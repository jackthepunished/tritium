use crate::pack::{pack_trits, unpack_trits};
use anyhow::{bail, Context, Result};
use std::io::{BufWriter, Write};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DType {
    F32,
    Trit,
}

#[derive(Clone, Debug)]
pub struct TensorMeta {
    pub name: String,
    pub dtype: DType,
    pub shape: Vec<usize>,
    pub scale: f32,
    offset: u64,
    byte_len: u64,
}

pub struct TritWriter {
    path: std::path::PathBuf,
    config: String,
    metas: Vec<TensorMeta>,
    payload: Vec<u8>,
}

impl TritWriter {
    pub fn create(path: &Path, config_json: &str) -> Result<Self> {
        Ok(Self {
            path: path.to_path_buf(),
            config: config_json.to_string(),
            metas: Vec::new(),
            payload: Vec::new(),
        })
    }

    fn push(&mut self, name: &str, dtype: DType, shape: &[usize], scale: f32, bytes: &[u8]) {
        self.metas.push(TensorMeta {
            name: name.to_string(),
            dtype,
            shape: shape.to_vec(),
            scale,
            offset: self.payload.len() as u64,
            byte_len: bytes.len() as u64,
        });
        self.payload.extend_from_slice(bytes);
    }

    pub fn write_f32(&mut self, name: &str, shape: &[usize], data: &[f32]) -> Result<()> {
        assert_eq!(shape.iter().product::<usize>(), data.len());
        let bytes: Vec<u8> = data.iter().flat_map(|v| v.to_le_bytes()).collect();
        self.push(name, DType::F32, shape, 1.0, &bytes);
        Ok(())
    }

    pub fn write_trit(&mut self, name: &str, shape: &[usize], trits: &[i8], scale: f32) -> Result<()> {
        assert_eq!(shape.iter().product::<usize>(), trits.len());
        self.push(name, DType::Trit, shape, scale, &pack_trits(trits));
        Ok(())
    }

    pub fn finish(self) -> Result<()> {
        let f = std::fs::File::create(&self.path)?;
        let mut w = BufWriter::new(f);
        w.write_all(b"TRIT")?;
        w.write_all(&0u32.to_le_bytes())?;
        w.write_all(&(self.config.len() as u32).to_le_bytes())?;
        w.write_all(self.config.as_bytes())?;
        w.write_all(&(self.metas.len() as u32).to_le_bytes())?;
        for m in &self.metas {
            w.write_all(&(m.name.len() as u16).to_le_bytes())?;
            w.write_all(m.name.as_bytes())?;
            w.write_all(&[if m.dtype == DType::F32 { 0u8 } else { 1u8 }])?;
            w.write_all(&[m.shape.len() as u8])?;
            for d in &m.shape {
                w.write_all(&(*d as u32).to_le_bytes())?;
            }
            w.write_all(&m.scale.to_le_bytes())?;
            w.write_all(&m.offset.to_le_bytes())?;
            w.write_all(&m.byte_len.to_le_bytes())?;
        }
        w.write_all(&self.payload)?;
        Ok(())
    }
}

pub struct TritReader {
    mmap: memmap2::Mmap,
    config: String,
    metas: Vec<TensorMeta>,
    payload_start: usize,
}

/// Bounds-checked read of `n` bytes at cursor `p`; the file is untrusted input,
/// so every field access must fail with Err rather than panic on truncation.
fn take<'a>(b: &'a [u8], p: &mut usize, n: usize) -> Result<&'a [u8]> {
    let end = p.checked_add(n).context("length overflow")?;
    let s = b.get(*p..end).context("truncated .trit file")?;
    *p = end;
    Ok(s)
}

fn take_u32(b: &[u8], p: &mut usize) -> Result<u32> {
    Ok(u32::from_le_bytes(take(b, p, 4)?.try_into().unwrap()))
}

impl TritReader {
    pub fn open(path: &Path) -> Result<Self> {
        let f = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
        let mmap = unsafe { memmap2::Mmap::map(&f)? };
        let b = &mmap[..];
        let mut p = 0usize;
        if take(b, &mut p, 4)? != b"TRIT" {
            bail!("bad magic");
        }
        let version = take_u32(b, &mut p)?;
        if version != 0 {
            bail!("unsupported version {version}");
        }
        let clen = take_u32(b, &mut p)? as usize;
        let config = std::str::from_utf8(take(b, &mut p, clen)?)?.to_string();
        let n = take_u32(b, &mut p)? as usize;
        // No with_capacity(n): n is file-controlled and preallocation would
        // let a corrupt header request gigabytes before any bounds check fires.
        let mut metas = Vec::new();
        for _ in 0..n {
            let nlen = u16::from_le_bytes(take(b, &mut p, 2)?.try_into().unwrap()) as usize;
            let name = std::str::from_utf8(take(b, &mut p, nlen)?)?.to_string();
            let dtype = match take(b, &mut p, 1)?[0] {
                0 => DType::F32,
                1 => DType::Trit,
                x => bail!("bad dtype {x}"),
            };
            let ndim = take(b, &mut p, 1)?[0] as usize;
            let mut shape = Vec::with_capacity(ndim);
            for _ in 0..ndim {
                shape.push(take_u32(b, &mut p)? as usize);
            }
            let scale = f32::from_le_bytes(take(b, &mut p, 4)?.try_into().unwrap());
            let offset = u64::from_le_bytes(take(b, &mut p, 8)?.try_into().unwrap());
            let byte_len = u64::from_le_bytes(take(b, &mut p, 8)?.try_into().unwrap());
            metas.push(TensorMeta { name, dtype, shape, scale, offset, byte_len });
        }
        Ok(Self { mmap, config, metas, payload_start: p })
    }

    pub fn config_json(&self) -> &str {
        &self.config
    }

    pub fn metas(&self) -> &[TensorMeta] {
        &self.metas
    }

    fn meta(&self, name: &str) -> Result<&TensorMeta> {
        self.metas
            .iter()
            .find(|m| m.name == name)
            .with_context(|| format!("tensor not found: {name}"))
    }

    fn bytes(&self, m: &TensorMeta) -> Result<&[u8]> {
        let start = self
            .payload_start
            .checked_add(usize::try_from(m.offset)?)
            .context("tensor offset overflow")?;
        let end = start
            .checked_add(usize::try_from(m.byte_len)?)
            .context("tensor length overflow")?;
        self.mmap
            .get(start..end)
            .with_context(|| format!("tensor {} data out of file bounds", m.name))
    }

    fn elem_count(m: &TensorMeta) -> Result<usize> {
        m.shape
            .iter()
            .try_fold(1usize, |a, &d| a.checked_mul(d))
            .with_context(|| format!("tensor {} shape overflow", m.name))
    }

    pub fn read_f32(&self, name: &str) -> Result<Vec<f32>> {
        let m = self.meta(name)?;
        if m.dtype != DType::F32 {
            bail!("{name} is not F32");
        }
        let bytes = self.bytes(m)?;
        let n = Self::elem_count(m)?;
        if bytes.len() != n * 4 {
            bail!("{name}: byte_len {} does not match shape ({n} f32s)", bytes.len());
        }
        Ok(bytes
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
            .collect())
    }

    pub fn read_trit(&self, name: &str) -> Result<(Vec<i8>, f32)> {
        let m = self.meta(name)?;
        if m.dtype != DType::Trit {
            bail!("{name} is not Trit");
        }
        let n = Self::elem_count(m)?;
        Ok((unpack_trits(self.bytes(m)?, n)?, m.scale))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_then_read_roundtrip() {
        let dir = std::env::temp_dir().join("tritfmt_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("m.trit");

        let mut w = TritWriter::create(&path, r#"{"hidden_size":4}"#).unwrap();
        w.write_f32("norm.weight", &[4], &[1.0, 2.0, 3.0, 4.0]).unwrap();
        w.write_trit("w.weight", &[2, 3], &[1, -1, 0, 0, 1, 1], 0.5).unwrap();
        w.finish().unwrap();

        let r = TritReader::open(&path).unwrap();
        assert_eq!(r.config_json(), r#"{"hidden_size":4}"#);
        assert_eq!(r.metas().len(), 2);
        assert_eq!(r.read_f32("norm.weight").unwrap(), vec![1.0, 2.0, 3.0, 4.0]);
        let (trits, scale) = r.read_trit("w.weight").unwrap();
        assert_eq!(trits, vec![1, -1, 0, 0, 1, 1]);
        assert_eq!(scale, 0.5);
        assert!(r.read_f32("nope").is_err());
        // dtype mismatch is an error
        assert!(r.read_trit("norm.weight").is_err());
    }

    #[test]
    fn truncated_files_error_instead_of_panicking() {
        let dir = std::env::temp_dir().join("tritfmt_trunc_test");
        std::fs::create_dir_all(&dir).unwrap();
        let full_path = dir.join("full.trit");
        let mut w = TritWriter::create(&full_path, r#"{"hidden_size":4}"#).unwrap();
        w.write_f32("norm.weight", &[4], &[1.0, 2.0, 3.0, 4.0]).unwrap();
        w.write_trit("w.weight", &[2, 3], &[1, -1, 0, 0, 1, 1], 0.5).unwrap();
        w.finish().unwrap();
        let full = std::fs::read(&full_path).unwrap();

        // Every strict prefix must produce Err somewhere, never a panic.
        let cut_path = dir.join("cut.trit");
        for cut in [0, 3, 4, 8, 12, 20, full.len() / 2, full.len() - 1] {
            std::fs::write(&cut_path, &full[..cut]).unwrap();
            let outcome = TritReader::open(&cut_path).and_then(|r| {
                r.read_f32("norm.weight")?;
                r.read_trit("w.weight")?;
                Ok(())
            });
            assert!(outcome.is_err(), "cut at {cut} bytes should error");
        }
    }
}
