//! The `.trit` container, version 1.
//!
//! ```text
//! off  size  field
//!   0     4  magic "TRIT"
//!   4     4  u32 version = 1
//!   8     4  u32 header_flags (reserved, must be 0)
//!  12     8  u64 payload_hash (FNV-1a-64; 0 = not computed)
//!  20     4  u32 config_len
//!  24     n  config (UTF-8 JSON, the HF config.json verbatim)
//!   .     4  u32 tensor_count
//!   .     .  tensor records
//!   .     .  zero padding so payload_start % 64 == 0
//!   .     .  payload
//! ```
//!
//! Record: `u16 name_len`, name, `u8 dtype`, `u8 ndim`, `u8 reserved0`,
//! `u8 reserved1`, `u32 dims[ndim]`, `f32 scale`, `u32 flags`, `u64 offset`,
//! `u64 byte_len`. All little-endian; offsets relative to payload start and
//! themselves 64-byte aligned.
//!
//! Ternary payloads are the bit-plane beat stream defined in [`crate::planes`].
//! Because `payload_start` and every tensor offset are 64-byte aligned and the
//! mapping base is page-aligned, a tensor's bytes begin on a cache line -- which
//! is what lets a dense f32 tensor be borrowed from the map with no copy, and
//! what a future DMA descriptor chain will want.
//!
//! The file is untrusted input. Every field access is bounds-checked and every
//! structural relation is validated at `open()`, so the accessors cannot panic
//! on a truncated or hostile file. The two full-payload checks -- the plane
//! invariants and the payload hash -- are O(file) and therefore opt-in via
//! [`TritFile::open_verified`] or [`TritFile::verify`].

use crate::planes::{self, TritPlanes};
use anyhow::{bail, ensure, Context, Result};
use std::io::{BufWriter, Write};
use std::path::Path;

/// Payload and tensor-offset alignment.
pub const ALIGN: usize = 64;

/// The version this build reads and writes.
pub const VERSION: u32 = 1;

const MAGIC: &[u8; 4] = b"TRIT";
const MAX_CONFIG_LEN: usize = 16 << 20;
const MAX_TENSORS: usize = 1 << 20;
const MAX_NAME_LEN: usize = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DType {
    F32,
    Trit,
    Bf16,
}

impl DType {
    fn code(self) -> u8 {
        match self {
            DType::F32 => 0,
            DType::Trit => 1,
            DType::Bf16 => 2,
        }
    }
    fn from_code(c: u8) -> Result<Self> {
        Ok(match c {
            0 => DType::F32,
            1 => DType::Trit,
            2 => DType::Bf16,
            x => bail!("bad dtype {x}"),
        })
    }
    /// Bytes per element for the dense dtypes.
    fn elem_size(self) -> Option<usize> {
        match self {
            DType::F32 => Some(4),
            DType::Bf16 => Some(2),
            DType::Trit => None,
        }
    }
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

impl TensorMeta {
    pub fn elem_count(&self) -> Result<usize> {
        self.shape
            .iter()
            .try_fold(1usize, |a, &d| a.checked_mul(d))
            .with_context(|| format!("tensor {} shape overflow", self.name))
    }
    pub fn byte_len(&self) -> u64 {
        self.byte_len
    }
}

/// FNV-1a-64 over the payload. Cheap, dependency-free, and enough to catch a
/// truncated download or a half-written file; it is not a security primitive.
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    h
}

// ---------------------------------------------------------------- writer

pub struct TritWriter {
    path: std::path::PathBuf,
    config: String,
    metas: Vec<TensorMeta>,
    payload: Vec<u8>,
}

impl TritWriter {
    pub fn create(path: &Path, config_json: &str) -> Result<Self> {
        ensure!(config_json.len() <= MAX_CONFIG_LEN, "config JSON too large");
        Ok(Self {
            path: path.to_path_buf(),
            config: config_json.to_string(),
            metas: Vec::new(),
            payload: Vec::new(),
        })
    }

    fn push(&mut self, name: &str, dtype: DType, shape: &[usize], scale: f32, bytes: &[u8]) -> Result<()> {
        ensure!(!name.is_empty() && name.len() <= MAX_NAME_LEN, "bad tensor name length: {name}");
        ensure!(!shape.is_empty() && shape.len() <= 4, "tensor {name}: ndim must be 1..=4");
        ensure!(scale.is_finite() && scale > 0.0, "tensor {name}: scale {scale} must be positive and finite");
        ensure!(
            !self.metas.iter().any(|m| m.name == name),
            "duplicate tensor name: {name}"
        );
        // Pad so this tensor starts on an aligned offset.
        let pad = (ALIGN - self.payload.len() % ALIGN) % ALIGN;
        self.payload.resize(self.payload.len() + pad, 0);
        self.metas.push(TensorMeta {
            name: name.to_string(),
            dtype,
            shape: shape.to_vec(),
            scale,
            offset: self.payload.len() as u64,
            byte_len: bytes.len() as u64,
        });
        self.payload.extend_from_slice(bytes);
        Ok(())
    }

    pub fn write_f32(&mut self, name: &str, shape: &[usize], data: &[f32]) -> Result<()> {
        ensure!(shape.iter().product::<usize>() == data.len(), "tensor {name}: shape/data mismatch");
        let bytes: Vec<u8> = data.iter().flat_map(|v| v.to_le_bytes()).collect();
        self.push(name, DType::F32, shape, 1.0, &bytes)
    }

    /// Store a dense tensor as bfloat16 (round-to-nearest-even), halving its
    /// footprint and its per-token bandwidth.
    pub fn write_bf16(&mut self, name: &str, shape: &[usize], data: &[f32]) -> Result<()> {
        ensure!(shape.iter().product::<usize>() == data.len(), "tensor {name}: shape/data mismatch");
        let bytes: Vec<u8> = data.iter().flat_map(|v| f32_to_bf16_bits(*v).to_le_bytes()).collect();
        self.push(name, DType::Bf16, shape, 1.0, &bytes)
    }

    pub fn write_trit(&mut self, name: &str, shape: &[usize], trits: &[i8], scale: f32) -> Result<()> {
        ensure!(shape.len() == 2, "ternary tensor {name} must be 2-D, got {shape:?}");
        ensure!(shape.iter().product::<usize>() == trits.len(), "tensor {name}: shape/data mismatch");
        let beats = planes::pack_planes(trits, shape[0], shape[1])
            .with_context(|| format!("packing {name}"))?;
        self.push(name, DType::Trit, shape, scale, &beats)
    }

    pub fn finish(self) -> Result<()> {
        let f = std::fs::File::create(&self.path)
            .with_context(|| format!("create {}", self.path.display()))?;
        let mut w = BufWriter::new(f);

        // Header size must be known before writing, to compute the padding that
        // puts payload_start on an ALIGN boundary.
        let mut head_len = 4 + 4 + 4 + 8 + 4 + self.config.len() + 4;
        for m in &self.metas {
            head_len += 2 + m.name.len() + 1 + 1 + 1 + 1 + 4 * m.shape.len() + 4 + 4 + 8 + 8;
        }
        let head_pad = (ALIGN - head_len % ALIGN) % ALIGN;

        w.write_all(MAGIC)?;
        w.write_all(&VERSION.to_le_bytes())?;
        w.write_all(&0u32.to_le_bytes())?; // header_flags
        w.write_all(&fnv1a64(&self.payload).to_le_bytes())?;
        w.write_all(&(self.config.len() as u32).to_le_bytes())?;
        w.write_all(self.config.as_bytes())?;
        w.write_all(&(self.metas.len() as u32).to_le_bytes())?;
        for m in &self.metas {
            w.write_all(&(m.name.len() as u16).to_le_bytes())?;
            w.write_all(m.name.as_bytes())?;
            w.write_all(&[m.dtype.code(), m.shape.len() as u8, 0, 0])?;
            for d in &m.shape {
                w.write_all(&(*d as u32).to_le_bytes())?;
            }
            w.write_all(&m.scale.to_le_bytes())?;
            w.write_all(&0u32.to_le_bytes())?; // per-tensor flags
            w.write_all(&m.offset.to_le_bytes())?;
            w.write_all(&m.byte_len.to_le_bytes())?;
        }
        w.write_all(&vec![0u8; head_pad])?;
        w.write_all(&self.payload)?;
        w.flush()?;
        Ok(())
    }
}

/// f32 -> bf16 bits, round-to-nearest-even. NaN payloads are preserved as a
/// quiet NaN rather than silently becoming infinity.
fn f32_to_bf16_bits(v: f32) -> u16 {
    let bits = v.to_bits();
    if v.is_nan() {
        return ((bits >> 16) as u16) | 0x0040;
    }
    let rounding = 0x7fff + ((bits >> 16) & 1);
    ((bits.wrapping_add(rounding)) >> 16) as u16
}

fn bf16_bits_to_f32(b: u16) -> f32 {
    f32::from_bits((b as u32) << 16)
}

// ---------------------------------------------------------------- spans

/// A validated handle to a ternary tensor.
///
/// Spans are resolved once at load and are plain `Copy` values, so a `Model` can
/// hold them in its layer structs without borrowing the file. That is what keeps
/// `Model` free of a lifetime parameter -- and therefore `Send + Sync`, storable
/// behind an `Arc`, and usable as a `'static` handle across the C FFI.
#[derive(Clone, Copy, Debug)]
pub struct TritSpan {
    start: usize,
    len: usize,
    rows: usize,
    cols: usize,
    scale: f32,
}

impl TritSpan {
    pub fn rows(&self) -> usize {
        self.rows
    }
    pub fn cols(&self) -> usize {
        self.cols
    }
    pub fn scale(&self) -> f32 {
        self.scale
    }
    /// Bytes this tensor occupies -- its exact contribution to per-token
    /// weight bandwidth.
    pub fn bytes(&self) -> usize {
        self.len
    }
}

/// A validated handle to a dense (F32 or BF16) tensor.
#[derive(Clone, Copy, Debug)]
pub struct DenseSpan {
    start: usize,
    len: usize,
    dtype: DType,
    elems: usize,
}

impl DenseSpan {
    pub fn elems(&self) -> usize {
        self.elems
    }
    pub fn dtype(&self) -> DType {
        self.dtype
    }
    pub fn bytes(&self) -> usize {
        self.len
    }
}

// ---------------------------------------------------------------- reader

pub struct TritFile {
    mmap: memmap2::Mmap,
    config: String,
    metas: Vec<TensorMeta>,
    payload_start: usize,
    payload_hash: u64,
}

/// Prints the header summary rather than the mapping, which is routinely
/// gigabytes.
impl std::fmt::Debug for TritFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TritFile")
            .field("tensors", &self.metas.len())
            .field("payload_start", &self.payload_start)
            .field("mapped_bytes", &self.mmap.len())
            .finish()
    }
}

/// Bounds-checked read of `n` bytes at cursor `p`; the file is untrusted input,
/// so every field access must fail with Err rather than panic on truncation.
fn take<'a>(b: &'a [u8], p: &mut usize, n: usize) -> Result<&'a [u8]> {
    let end = p.checked_add(n).context("length overflow")?;
    let s = b.get(*p..end).context("truncated .trit file")?;
    *p = end;
    Ok(s)
}

fn take_u16(b: &[u8], p: &mut usize) -> Result<u16> {
    Ok(u16::from_le_bytes(take(b, p, 2)?.try_into().unwrap()))
}
fn take_u32(b: &[u8], p: &mut usize) -> Result<u32> {
    Ok(u32::from_le_bytes(take(b, p, 4)?.try_into().unwrap()))
}
fn take_u64(b: &[u8], p: &mut usize) -> Result<u64> {
    Ok(u64::from_le_bytes(take(b, p, 8)?.try_into().unwrap()))
}

impl TritFile {
    /// Open and validate the header. O(tensor_count); does not touch the payload.
    pub fn open(path: &Path) -> Result<std::sync::Arc<Self>> {
        let f = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
        // SAFETY: the only unsafe operation in this crate's read path. The map is
        // owned by the returned TritFile and outlives every view handed out. A
        // concurrent truncation of the backing file would fault -- documented as
        // unsupported; use --preload for deployments that rotate models in place.
        let mmap = unsafe { memmap2::Mmap::map(&f)? };
        Self::from_mmap(mmap).with_context(|| format!("reading {}", path.display()))
    }

    /// Open, then walk the whole payload: plane invariants and the payload hash.
    pub fn open_verified(path: &Path) -> Result<std::sync::Arc<Self>> {
        let file = Self::open(path)?;
        file.verify()?;
        Ok(file)
    }

    fn from_mmap(mmap: memmap2::Mmap) -> Result<std::sync::Arc<Self>> {
        let b = &mmap[..];
        let mut p = 0usize;

        if take(b, &mut p, 4)? != MAGIC {
            bail!("bad magic: not a .trit file");
        }
        match take_u32(b, &mut p)? {
            VERSION => {}
            0 => bail!(
                ".trit v0 (2-bit interleaved codes) is no longer supported; \
                 re-run 'tritc convert', or 'tritc upgrade --input <old> --output <new>'"
            ),
            v => bail!("unsupported .trit version {v} (this build reads v{VERSION})"),
        }
        let flags = take_u32(b, &mut p)?;
        ensure!(flags == 0, "unknown header flags 0x{flags:08x}");
        let payload_hash = take_u64(b, &mut p)?;

        let clen = take_u32(b, &mut p)? as usize;
        ensure!(clen <= MAX_CONFIG_LEN, "config JSON too large: {clen} bytes");
        let config = std::str::from_utf8(take(b, &mut p, clen)?)
            .context("config JSON is not valid UTF-8")?
            .to_string();

        let n = take_u32(b, &mut p)? as usize;
        ensure!(n <= MAX_TENSORS, "implausible tensor count {n}");
        // No with_capacity(n): n is file-controlled and preallocation would
        // let a corrupt header request gigabytes before any bounds check fires.
        let mut metas = Vec::new();
        for _ in 0..n {
            let nlen = take_u16(b, &mut p)? as usize;
            ensure!(nlen > 0 && nlen <= MAX_NAME_LEN, "bad tensor name length {nlen}");
            let name = std::str::from_utf8(take(b, &mut p, nlen)?)
                .context("tensor name is not valid UTF-8")?
                .to_string();
            let hdr = take(b, &mut p, 4)?;
            let (dtype, ndim, r0, r1) = (DType::from_code(hdr[0])?, hdr[1] as usize, hdr[2], hdr[3]);
            ensure!(r0 == 0 && r1 == 0, "tensor {name}: reserved bytes must be zero");
            ensure!((1..=4).contains(&ndim), "tensor {name}: ndim {ndim} out of range 1..=4");
            let mut shape = Vec::with_capacity(ndim);
            for _ in 0..ndim {
                shape.push(take_u32(b, &mut p)? as usize);
            }
            let scale = f32::from_le_bytes(take(b, &mut p, 4)?.try_into().unwrap());
            ensure!(
                scale.is_finite() && scale > 0.0,
                "tensor {name}: scale {scale} is not a positive finite number"
            );
            let tflags = take_u32(b, &mut p)?;
            ensure!(tflags == 0, "tensor {name}: unknown flags 0x{tflags:08x}");
            let offset = take_u64(b, &mut p)?;
            let byte_len = take_u64(b, &mut p)?;
            ensure!(
                offset % ALIGN as u64 == 0,
                "tensor {name}: offset {offset} is not {ALIGN}-byte aligned"
            );
            metas.push(TensorMeta { name, dtype, shape, scale, offset, byte_len });
        }

        // Header padding to the payload boundary.
        let payload_start = p.next_multiple_of(ALIGN);
        ensure!(payload_start <= b.len(), "truncated .trit file: no payload");

        let file = Self { mmap, config, metas, payload_start, payload_hash };
        file.validate_layout()?;
        Ok(std::sync::Arc::new(file))
    }

    /// Structural checks that need every meta in hand: name uniqueness, extents
    /// inside the file, declared sizes matching declared shapes, and no two
    /// tensors claiming the same bytes.
    fn validate_layout(&self) -> Result<()> {
        let payload_len = self.mmap.len() - self.payload_start;
        let mut seen = std::collections::HashSet::new();
        for m in &self.metas {
            ensure!(seen.insert(m.name.as_str()), "duplicate tensor name: {}", m.name);
            let off = usize::try_from(m.offset).context("tensor offset overflow")?;
            let len = usize::try_from(m.byte_len).context("tensor length overflow")?;
            let end = off.checked_add(len).context("tensor extent overflow")?;
            ensure!(end <= payload_len, "tensor {} data out of file bounds", m.name);

            let elems = m.elem_count()?;
            match m.dtype {
                DType::Trit => {
                    ensure!(m.shape.len() == 2, "ternary tensor {} must be 2-D", m.name);
                    let (rows, cols) = (m.shape[0], m.shape[1]);
                    ensure!(cols > 0, "ternary tensor {} has zero columns", m.name);
                    let want = planes::payload_len(rows, cols);
                    ensure!(
                        len == want,
                        "tensor {}: byte_len {len} does not match {rows}x{cols} \
                         ({} beats of {} bytes = {want})",
                        m.name,
                        rows * planes::beats_per_row(cols),
                        planes::BEAT_BYTES
                    );
                }
                dense => {
                    let es = dense.elem_size().unwrap();
                    ensure!(
                        len == elems * es,
                        "tensor {}: byte_len {len} does not match shape ({elems} x {es} bytes)",
                        m.name
                    );
                }
            }
        }
        // Overlap detection: sort spans once rather than comparing all pairs.
        let mut spans: Vec<(u64, u64, &str)> =
            self.metas.iter().filter(|m| m.byte_len > 0).map(|m| (m.offset, m.byte_len, m.name.as_str())).collect();
        spans.sort_unstable_by_key(|s| s.0);
        for w in spans.windows(2) {
            let (o0, l0, n0) = w[0];
            let (o1, _, n1) = w[1];
            ensure!(o0 + l0 <= o1, "tensor {n0} overlaps tensor {n1} in the payload");
        }
        Ok(())
    }

    pub fn config_json(&self) -> &str {
        &self.config
    }
    pub fn metas(&self) -> &[TensorMeta] {
        &self.metas
    }
    pub fn has(&self, name: &str) -> bool {
        self.metas.iter().any(|m| m.name == name)
    }
    /// Byte offset where the payload begins. Always a multiple of [`ALIGN`].
    pub fn payload_start(&self) -> usize {
        self.payload_start
    }

    fn meta(&self, name: &str) -> Result<&TensorMeta> {
        self.metas
            .iter()
            .find(|m| m.name == name)
            .with_context(|| format!("tensor not found: {name}"))
    }

    /// Payload bytes of a tensor. Infallible in practice -- `validate_layout`
    /// already proved the extent is inside the map -- but kept checked so a
    /// future edit cannot turn it into a panic.
    fn bytes(&self, m: &TensorMeta) -> Result<&[u8]> {
        let start = self.payload_start + m.offset as usize;
        let end = start + m.byte_len as usize;
        self.mmap
            .get(start..end)
            .with_context(|| format!("tensor {} data out of file bounds", m.name))
    }

    /// Resolve a ternary tensor to a `Copy` span. Validates shape and dtype once.
    pub fn trit_span(&self, name: &str) -> Result<TritSpan> {
        let m = self.meta(name)?;
        ensure!(m.dtype == DType::Trit, "{name} is not a ternary tensor");
        Ok(TritSpan {
            start: self.payload_start + m.offset as usize,
            len: m.byte_len as usize,
            rows: m.shape[0],
            cols: m.shape[1],
            scale: m.scale,
        })
    }

    /// Resolve a dense tensor to a `Copy` span.
    pub fn dense_span(&self, name: &str) -> Result<DenseSpan> {
        let m = self.meta(name)?;
        ensure!(m.dtype != DType::Trit, "{name} is ternary, not dense");
        Ok(DenseSpan {
            start: self.payload_start + m.offset as usize,
            len: m.byte_len as usize,
            dtype: m.dtype,
            elems: m.elem_count()?,
        })
    }

    /// Borrow a ternary tensor's beat stream. Infallible: the span was validated
    /// when it was resolved, so this cannot fail at call time.
    pub fn planes(&self, s: TritSpan) -> TritPlanes<'_> {
        TritPlanes::new_unchecked_bits(&self.mmap[s.start..s.start + s.len], s.rows, s.cols, s.scale)
            .expect("span validated at resolve time")
    }

    /// Borrow (F32) or decode (BF16) a dense tensor.
    ///
    /// F32 tensors are returned borrowed straight from the map when the mapping
    /// happens to be 4-byte aligned there -- which the format's 64-byte offset
    /// alignment guarantees. That is the difference between a 1.3 GB copy of
    /// `lm_head` and none at all.
    pub fn dense(&self, s: DenseSpan) -> std::borrow::Cow<'_, [f32]> {
        let raw = &self.mmap[s.start..s.start + s.len];
        match s.dtype {
            DType::F32 => {
                // SAFETY: every bit pattern is a valid f32, the slice length is a
                // multiple of 4 (checked in validate_layout), and align_to hands
                // back any unaligned head/tail rather than assuming alignment.
                let (head, mid, tail) = unsafe { raw.align_to::<f32>() };
                if head.is_empty() && tail.is_empty() {
                    std::borrow::Cow::Borrowed(mid)
                } else {
                    // Format guarantees alignment, so this is unreachable for a
                    // well-formed file; decode rather than fail if it happens.
                    std::borrow::Cow::Owned(
                        raw.chunks_exact(4)
                            .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
                            .collect(),
                    )
                }
            }
            DType::Bf16 => std::borrow::Cow::Owned(
                raw.chunks_exact(2)
                    .map(|c| bf16_bits_to_f32(u16::from_le_bytes(c.try_into().unwrap())))
                    .collect(),
            ),
            DType::Trit => unreachable!("dense_span rejects ternary tensors"),
        }
    }

    /// One row of a dense 2-D tensor -- the embedding lookup, without touching
    /// the other 128k rows.
    pub fn dense_row(&self, s: DenseSpan, row: usize, cols: usize) -> Result<std::borrow::Cow<'_, [f32]>> {
        let es = s.dtype.elem_size().context("not a dense tensor")?;
        let start = s.start + row * cols * es;
        let end = start + cols * es;
        ensure!(end <= s.start + s.len, "row {row} out of bounds");
        let sub = DenseSpan { start, len: cols * es, dtype: s.dtype, elems: cols };
        Ok(self.dense(sub))
    }

    /// Full payload walk: plane invariants for every ternary tensor, plus the
    /// stored payload hash. O(file size).
    pub fn verify(&self) -> Result<VerifyReport> {
        let mut rep = VerifyReport::default();
        for m in &self.metas {
            rep.tensors += 1;
            if m.dtype != DType::Trit {
                continue;
            }
            let (rows, cols) = (m.shape[0], m.shape[1]);
            let p = TritPlanes::new(self.bytes(m)?, rows, cols, m.scale)
                .with_context(|| format!("tensor {}", m.name))?;
            rep.ternary_tensors += 1;
            rep.total_trits += (rows as u64) * (cols as u64);
            rep.zeros += p.zero_count();
        }
        rep.zero_fraction =
            if rep.total_trits > 0 { rep.zeros as f64 / rep.total_trits as f64 } else { 0.0 };
        rep.hash_ok = if self.payload_hash == 0 {
            None
        } else {
            Some(fnv1a64(&self.mmap[self.payload_start..]) == self.payload_hash)
        };
        if rep.hash_ok == Some(false) {
            bail!("payload hash mismatch: the file is corrupt or was truncated");
        }
        Ok(rep)
    }
}

#[derive(Debug, Default)]
pub struct VerifyReport {
    pub tensors: usize,
    pub ternary_tensors: usize,
    pub total_trits: u64,
    pub zeros: u64,
    pub zero_fraction: f64,
    /// `None` when the file stores no hash.
    pub hash_ok: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("tritfmt_v1_tests");
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    fn write_sample(path: &Path) {
        let mut w = TritWriter::create(path, r#"{"hidden_size":4}"#).unwrap();
        w.write_f32("norm.weight", &[4], &[1.0, 2.0, 3.0, 4.0]).unwrap();
        w.write_trit("w.weight", &[2, 3], &[1, -1, 0, 0, 1, 1], 0.5).unwrap();
        w.finish().unwrap();
    }

    #[test]
    fn write_then_read_roundtrip() {
        let path = tmp("m.trit");
        write_sample(&path);

        let r = TritFile::open(&path).unwrap();
        assert_eq!(r.config_json(), r#"{"hidden_size":4}"#);
        assert_eq!(r.metas().len(), 2);
        assert_eq!(&*r.dense(r.dense_span("norm.weight").unwrap()), &[1.0, 2.0, 3.0, 4.0]);

        let s = r.trit_span("w.weight").unwrap();
        assert_eq!((s.rows(), s.cols()), (2, 3));
        assert_eq!(s.scale(), 0.5);
        assert_eq!(r.planes(s).to_trits(), vec![1, -1, 0, 0, 1, 1]);

        assert!(r.dense_span("nope").is_err());
        assert!(r.trit_span("norm.weight").is_err(), "dtype mismatch is an error");
        assert!(r.dense_span("w.weight").is_err(), "dtype mismatch is an error");
    }

    #[test]
    fn payload_and_every_tensor_are_aligned() {
        let path = tmp("align.trit");
        write_sample(&path);
        let r = TritFile::open(&path).unwrap();
        assert_eq!(r.payload_start() % ALIGN, 0);
        for m in r.metas() {
            assert_eq!(m.offset % ALIGN as u64, 0, "{} unaligned", m.name);
        }
    }

    #[test]
    fn f32_tensors_are_borrowed_not_copied() {
        let path = tmp("borrow.trit");
        write_sample(&path);
        let r = TritFile::open(&path).unwrap();
        let got = r.dense(r.dense_span("norm.weight").unwrap());
        assert!(matches!(got, std::borrow::Cow::Borrowed(_)), "alignment should permit borrowing");
    }

    #[test]
    fn verify_reports_the_zero_fraction() {
        let path = tmp("verify.trit");
        write_sample(&path);
        let rep = TritFile::open(&path).unwrap().verify().unwrap();
        assert_eq!(rep.ternary_tensors, 1);
        assert_eq!(rep.total_trits, 6);
        assert_eq!(rep.zeros, 2); // [1,-1,0,0,1,1]
        assert_eq!(rep.hash_ok, Some(true));
    }

    #[test]
    fn v0_files_name_the_migration_command() {
        let path = tmp("v0.trit");
        let mut b = Vec::new();
        b.extend_from_slice(b"TRIT");
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&2u32.to_le_bytes());
        b.extend_from_slice(b"{}");
        b.extend_from_slice(&0u32.to_le_bytes());
        std::fs::write(&path, &b).unwrap();
        let err = format!("{:#}", TritFile::open(&path).unwrap_err());
        assert!(err.contains("no longer supported"), "{err}");
        assert!(err.contains("tritc upgrade"), "{err}");
    }

    #[test]
    fn future_versions_and_unknown_flags_are_refused() {
        for (ver, flags, needle) in [(2u32, 0u32, "unsupported .trit version"), (VERSION, 1, "unknown header flags")] {
            let path = tmp(&format!("ver{ver}_{flags}.trit"));
            let mut b = Vec::new();
            b.extend_from_slice(b"TRIT");
            b.extend_from_slice(&ver.to_le_bytes());
            b.extend_from_slice(&flags.to_le_bytes());
            b.extend_from_slice(&0u64.to_le_bytes());
            b.extend_from_slice(&0u32.to_le_bytes());
            b.extend_from_slice(&0u32.to_le_bytes());
            std::fs::write(&path, &b).unwrap();
            let err = format!("{:#}", TritFile::open(&path).unwrap_err());
            assert!(err.contains(needle), "{err}");
        }
    }

    #[test]
    fn truncated_files_error_instead_of_panicking() {
        let full_path = tmp("full.trit");
        write_sample(&full_path);
        let full = std::fs::read(&full_path).unwrap();

        // Every strict prefix must produce Err somewhere, never a panic.
        let cut_path = tmp("cut.trit");
        let mut cuts: Vec<usize> = vec![0, 3, 4, 8, 12, 20, 24, full.len() / 2, full.len() - 1];
        // and every byte of the header region, where the padding lives
        cuts.extend(0..full.len().min(200));
        for cut in cuts {
            std::fs::write(&cut_path, &full[..cut]).unwrap();
            let outcome = TritFile::open(&cut_path).and_then(|r| {
                r.dense(r.dense_span("norm.weight")?);
                r.planes(r.trit_span("w.weight")?).to_trits();
                r.verify()?;
                Ok(())
            });
            assert!(outcome.is_err(), "cut at {cut} bytes should error");
        }
    }

    #[test]
    fn corrupt_payload_is_caught_by_the_hash() {
        let path = tmp("corrupt.trit");
        write_sample(&path);
        let mut bytes = std::fs::read(&path).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xff;
        std::fs::write(&path, &bytes).unwrap();
        let err = format!("{:#}", TritFile::open(&path).unwrap().verify().unwrap_err());
        assert!(err.contains("hash mismatch") || err.contains("padding bits"), "{err}");
    }

    #[test]
    fn non_positive_or_nan_scale_is_refused() {
        // A NaN scale would silently poison every logit with no error anywhere.
        let path = tmp("nanscale.trit");
        let mut w = TritWriter::create(&path, "{}").unwrap();
        assert!(w.write_trit("w", &[1, 1], &[1], f32::NAN).is_err());
        assert!(w.write_trit("w", &[1, 1], &[1], 0.0).is_err());
        assert!(w.write_trit("w", &[1, 1], &[1], -1.0).is_err());
    }

    #[test]
    fn duplicate_tensor_names_are_refused_at_write_time() {
        let path = tmp("dup.trit");
        let mut w = TritWriter::create(&path, "{}").unwrap();
        w.write_f32("a", &[1], &[1.0]).unwrap();
        assert!(w.write_f32("a", &[1], &[2.0]).is_err());
    }

    #[test]
    fn bf16_roundtrips_within_its_precision() {
        let path = tmp("bf16.trit");
        let vals: Vec<f32> = vec![0.0, 1.0, -1.0, 0.5, 3.14159, -2.71828, 1e-8, 1e8];
        let mut w = TritWriter::create(&path, "{}").unwrap();
        w.write_bf16("e", &[vals.len()], &vals).unwrap();
        w.finish().unwrap();
        let r = TritFile::open(&path).unwrap();
        let got = r.dense(r.dense_span("e").unwrap());
        for (a, b) in vals.iter().zip(got.iter()) {
            // bf16 keeps 8 mantissa bits: ~2^-8 relative
            assert!((a - b).abs() <= a.abs() * 0.004 + 1e-30, "{a} -> {b}");
        }
    }
}
