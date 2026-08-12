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

impl TritReader {
    pub fn open(path: &Path) -> Result<Self> {
        let f = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
        let mmap = unsafe { memmap2::Mmap::map(&f)? };
        let b = &mmap[..];
        if &b[0..4] != b"TRIT" {
            bail!("bad magic");
        }
        let mut p = 4usize;
        let rd_u32 = |b: &[u8], p: &mut usize| {
            let v = u32::from_le_bytes(b[*p..*p + 4].try_into().unwrap());
            *p += 4;
            v
        };
        let version = rd_u32(b, &mut p);
        if version != 0 {
            bail!("unsupported version {version}");
        }
        let clen = rd_u32(b, &mut p) as usize;
        let config = std::str::from_utf8(&b[p..p + clen])?.to_string();
        p += clen;
        let n = rd_u32(b, &mut p) as usize;
        let mut metas = Vec::with_capacity(n);
        for _ in 0..n {
            let nlen = u16::from_le_bytes(b[p..p + 2].try_into().unwrap()) as usize;
            p += 2;
            let name = std::str::from_utf8(&b[p..p + nlen])?.to_string();
            p += nlen;
            let dtype = match b[p] {
                0 => DType::F32,
                1 => DType::Trit,
                x => bail!("bad dtype {x}"),
            };
            p += 1;
            let ndim = b[p] as usize;
            p += 1;
            let mut shape = Vec::with_capacity(ndim);
            for _ in 0..ndim {
                shape.push(rd_u32(b, &mut p) as usize);
            }
            let scale = f32::from_le_bytes(b[p..p + 4].try_into().unwrap());
            p += 4;
            let offset = u64::from_le_bytes(b[p..p + 8].try_into().unwrap());
            p += 8;
            let byte_len = u64::from_le_bytes(b[p..p + 8].try_into().unwrap());
            p += 8;
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

    fn bytes(&self, m: &TensorMeta) -> &[u8] {
        let s = self.payload_start + m.offset as usize;
        &self.mmap[s..s + m.byte_len as usize]
    }

    pub fn read_f32(&self, name: &str) -> Result<Vec<f32>> {
        let m = self.meta(name)?;
        if m.dtype != DType::F32 {
            bail!("{name} is not F32");
        }
        Ok(self
            .bytes(m)
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
            .collect())
    }

    pub fn read_trit(&self, name: &str) -> Result<(Vec<i8>, f32)> {
        let m = self.meta(name)?;
        if m.dtype != DType::Trit {
            bail!("{name} is not Trit");
        }
        let n: usize = m.shape.iter().product();
        Ok((unpack_trits(self.bytes(m), n)?, m.scale))
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
}
