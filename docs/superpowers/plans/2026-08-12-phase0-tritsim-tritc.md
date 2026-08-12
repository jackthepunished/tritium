# tritium Phase 0 (tritsim + tritc) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A bit-accurate CPU reference for ternary (BitNet b1.58-style) LLM inference in Rust: `tritc` converts a Hugging Face checkpoint into the packed `.trit` format, and `tritsim` runs the full decode loop (no multiplies in the ternary hot path) and validates against reference logits.

**Architecture:** Cargo workspace with three crates. `trit-core` holds the shared math (absmean/absmax quantization, trit packing, select-accumulate matvec) and the `.trit` file format. `tritc` is a CLI that reads safetensors + config.json, quantizes weights to ternary, and writes `model.trit`. `tritsim` loads `model.trit` and runs a LLaMA-style transformer (RMSNorm, RoPE, GQA attention with KV cache, gated FFN, optional BitNet sub-norms) with per-token int8 activation quantization, greedy sampling, and a logit-comparison harness. Correctness comes from unit tests with hand-computed values plus property tests (determinism, causality) on a tiny synthetic model, then a manual logit comparison against the real checkpoint.

**Tech Stack:** Rust (edition 2021), safetensors 0.4, memmap2 0.9, half 2, serde/serde_json 1, clap 4 (derive), anyhow 1, tokenizers 0.21. Python 3 + transformers only for the reference-logit dump script.

## Global Constraints

- Repo root: `/mnt/d/dev/tritium` (Windows drive via WSL). Set `export CARGO_TARGET_DIR=$HOME/.cache/tritium-target` before any cargo command — building on `/mnt/d` (drvfs) is 10x slower.
- Every task ends with `cargo test --workspace` green and a commit. Commit messages: conventional (`feat:`, `test:`, `chore:`).
- No `unsafe` except via `memmap2`. No emojis anywhere (code, comments, commits, output).
- The ternary hot path (`ternary_matvec`) must contain no floating-point multiplies and no integer multiplies on weights — select/add/sub only. Scales are applied once per output element after accumulation.
- Trit encoding everywhere: 2 bits per trit, `0b00 = 0`, `0b01 = +1`, `0b10 = -1`, `0b11 = invalid (error on read)`. 4 trits per byte, little-endian within the byte (trit k at bits `2k..2k+2`).
- Target checkpoint: `microsoft/bitnet-b1.58-2B-4T-bf16` (master weights; the quant recipe below reproduces the QAT forward pass). Verify tensor names against the actual download in Task 5 — the layer code keys on LLaMA-style names plus optional `attn_sub_norm` / `ffn_sub_norm`.
- All buffers row-major `Vec<f32>` / `Vec<i8>`; matrices are `(rows, cols)` = (out_features, in_features), matching safetensors layout for `nn.Linear`.

## File Structure

```
Cargo.toml                      # workspace: crates/*
crates/trit-core/
  Cargo.toml
  src/lib.rs                    # pub mod quant; pub mod pack; pub mod matvec; pub mod tritfmt;
  src/quant.rs                  # absmean_quantize, absmax_quantize
  src/pack.rs                   # pack_trits, unpack_trits
  src/matvec.rs                 # ternary_matvec
  src/tritfmt.rs                # DType, TensorMeta, TritWriter, TritReader
crates/tritc/
  Cargo.toml
  src/main.rs                   # CLI: convert
  src/convert.rs                # safetensors -> model.trit
crates/tritsim/
  Cargo.toml
  src/main.rs                   # CLI: run | compare
  src/config.rs                 # ModelConfig (parsed from config.json embedded in .trit)
  src/math.rs                   # rmsnorm, softmax, rope, activations
  src/model.rs                  # Model, Layer, KvCache, forward()
  src/generate.rs               # tokenizer + greedy decode loop
  src/compare.rs                # logit comparison vs reference dump
  tests/tiny_model.rs           # end-to-end property tests on synthetic model
scripts/dump_logits.py          # reference logits via HF transformers
```

---

### Task 1: Workspace scaffold + trit-core quantization primitives

**Files:**
- Create: `Cargo.toml`, `crates/trit-core/Cargo.toml`, `crates/trit-core/src/lib.rs`, `crates/trit-core/src/quant.rs`
- Test: inline `#[cfg(test)]` in `crates/trit-core/src/quant.rs`

**Interfaces:**
- Consumes: nothing (first task).
- Produces: `trit_core::quant::absmean_quantize(w: &[f32]) -> (Vec<i8>, f32)` (trits in {-1,0,1}, scale = mean(|w|)+1e-8) and `trit_core::quant::absmax_quantize(x: &[f32]) -> (Vec<i8>, f32)` (int8 in [-127,127], scale = max(|x|)/127, all-zero input gives scale 1.0 and zeros).

- [ ] **Step 1: Create the workspace and crate**

Root `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = ["crates/trit-core"]
```

`crates/trit-core/Cargo.toml`:

```toml
[package]
name = "trit-core"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

`crates/trit-core/src/lib.rs`:

```rust
pub mod quant;
```

- [ ] **Step 2: Write the failing tests**

In `crates/trit-core/src/quant.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absmean_hand_computed() {
        // mean(|w|) = (0.5 + 1.2 + 0.1 + 2.0) / 4 = 0.95
        // w/s = [0.526, -1.263, 0.105, 2.105] -> round+clamp -> [1, -1, 0, 1]
        let (trits, scale) = absmean_quantize(&[0.5, -1.2, 0.1, 2.0]);
        assert_eq!(trits, vec![1, -1, 0, 1]);
        assert!((scale - 0.95).abs() < 1e-6);
    }

    #[test]
    fn absmean_all_zero_is_safe() {
        let (trits, scale) = absmean_quantize(&[0.0, 0.0]);
        assert_eq!(trits, vec![0, 0]);
        assert!(scale > 0.0);
    }

    #[test]
    fn absmax_hand_computed() {
        // max|x| = 2.54, scale = 2.54/127 = 0.02
        // xq = round([0, -127, 63.5]) = [0, -127, 64] (round half away from zero)
        let (xq, scale) = absmax_quantize(&[0.0, -2.54, 1.27]);
        assert_eq!(xq, vec![0, -127, 64]);
        assert!((scale - 0.02).abs() < 1e-6);
    }

    #[test]
    fn absmax_all_zero_is_safe() {
        let (xq, scale) = absmax_quantize(&[0.0; 4]);
        assert_eq!(xq, vec![0, 0, 0, 0]);
        assert_eq!(scale, 1.0);
    }
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cd /mnt/d/dev/tritium && export CARGO_TARGET_DIR=$HOME/.cache/tritium-target && cargo test -p trit-core`
Expected: COMPILE ERROR — `absmean_quantize` not found.

- [ ] **Step 4: Implement**

Top of `crates/trit-core/src/quant.rs`:

```rust
/// BitNet b1.58 weight quantization: scale = mean(|w|), trits = clamp(round(w/scale), -1, 1).
pub fn absmean_quantize(w: &[f32]) -> (Vec<i8>, f32) {
    let scale = w.iter().map(|v| v.abs()).sum::<f32>() / w.len() as f32 + 1e-8;
    let trits = w
        .iter()
        .map(|v| (v / scale).round().clamp(-1.0, 1.0) as i8)
        .collect();
    (trits, scale)
}

/// Per-token int8 activation quantization: scale = max(|x|)/127.
pub fn absmax_quantize(x: &[f32]) -> (Vec<i8>, f32) {
    let maxabs = x.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    if maxabs == 0.0 {
        return (vec![0; x.len()], 1.0);
    }
    let scale = maxabs / 127.0;
    let xq = x
        .iter()
        .map(|v| (v / scale).round().clamp(-127.0, 127.0) as i8)
        .collect();
    (xq, scale)
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p trit-core`
Expected: 4 passed.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml crates/trit-core
git commit -m "feat(trit-core): workspace scaffold + absmean/absmax quantization"
```

---

### Task 2: Trit packing + ternary select-accumulate matvec

**Files:**
- Create: `crates/trit-core/src/pack.rs`, `crates/trit-core/src/matvec.rs`
- Modify: `crates/trit-core/src/lib.rs` (add `pub mod pack; pub mod matvec;`)

**Interfaces:**
- Consumes: nothing.
- Produces: `trit_core::pack::pack_trits(trits: &[i8]) -> Vec<u8>`, `trit_core::pack::unpack_trits(bytes: &[u8], n: usize) -> anyhow::Result<Vec<i8>>` (errors on `0b11`), and `trit_core::matvec::ternary_matvec(trits: &[i8], rows: usize, cols: usize, xq: &[i8]) -> Vec<i32>` (row-major weights, i32 accumulators).

- [ ] **Step 1: Write the failing tests**

In `crates/trit-core/src/pack.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_including_partial_byte() {
        let trits: Vec<i8> = vec![1, -1, 0, 1, 0, -1]; // 6 trits -> 2 bytes
        let bytes = pack_trits(&trits);
        assert_eq!(bytes.len(), 2);
        // byte 0: trit0=01, trit1=10, trit2=00, trit3=01 -> 0b01_00_10_01 = 0x49
        assert_eq!(bytes[0], 0x49);
        assert_eq!(unpack_trits(&bytes, 6).unwrap(), trits);
    }

    #[test]
    fn invalid_encoding_is_an_error() {
        assert!(unpack_trits(&[0b0000_0011], 1).is_err());
    }
}
```

In `crates/trit-core/src/matvec.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_computed_2x3() {
        // W = [[1,-1,0],[0,1,1]], x = [10,20,30]
        // y = [10-20, 20+30] = [-10, 50]
        let w: Vec<i8> = vec![1, -1, 0, 0, 1, 1];
        assert_eq!(ternary_matvec(&w, 2, 3, &[10, 20, 30]), vec![-10, 50]);
    }

    #[test]
    fn matches_float_reference() {
        // vs naive f32 matmul of the same ternary weights
        let w: Vec<i8> = vec![1, 0, -1, -1, 1, 0, 0, 0, 1];
        let x: Vec<i8> = vec![-128i16 as i8, 127, 3]; // extremes are fine in i32 acc
        let y = ternary_matvec(&w, 3, 3, &x);
        for r in 0..3 {
            let expect: i32 = (0..3).map(|c| w[r * 3 + c] as i32 * x[c] as i32).sum();
            assert_eq!(y[r], expect);
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p trit-core`
Expected: COMPILE ERROR — modules missing.

- [ ] **Step 3: Implement**

`crates/trit-core/src/pack.rs`:

```rust
use anyhow::bail;

/// 2 bits/trit: 00 = 0, 01 = +1, 10 = -1. Trit k sits at bits 2k..2k+2 of byte k/4.
pub fn pack_trits(trits: &[i8]) -> Vec<u8> {
    let mut out = vec![0u8; trits.len().div_ceil(4)];
    for (i, &t) in trits.iter().enumerate() {
        let code: u8 = match t {
            0 => 0b00,
            1 => 0b01,
            -1 => 0b10,
            _ => panic!("non-ternary value {t}"),
        };
        out[i / 4] |= code << ((i % 4) * 2);
    }
    out
}

pub fn unpack_trits(bytes: &[u8], n: usize) -> anyhow::Result<Vec<i8>> {
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let code = (bytes[i / 4] >> ((i % 4) * 2)) & 0b11;
        out.push(match code {
            0b00 => 0,
            0b01 => 1,
            0b10 => -1,
            _ => bail!("invalid trit encoding 0b11 at index {i}"),
        });
    }
    Ok(out)
}
```

`crates/trit-core/src/matvec.rs`:

```rust
/// The ternary hot path: select-accumulate only, no multiplies on weights.
/// `trits` is row-major (rows x cols); returns i32 accumulators per row.
pub fn ternary_matvec(trits: &[i8], rows: usize, cols: usize, xq: &[i8]) -> Vec<i32> {
    assert_eq!(trits.len(), rows * cols);
    assert_eq!(xq.len(), cols);
    let mut y = vec![0i32; rows];
    for r in 0..rows {
        let row = &trits[r * cols..(r + 1) * cols];
        let mut acc = 0i32;
        for c in 0..cols {
            match row[c] {
                1 => acc += xq[c] as i32,
                -1 => acc -= xq[c] as i32,
                _ => {}
            }
        }
        y[r] = acc;
    }
    y
}
```

Update `lib.rs`: `pub mod matvec; pub mod pack; pub mod quant;`

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p trit-core`
Expected: 8 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/trit-core
git commit -m "feat(trit-core): trit packing and select-accumulate matvec kernel"
```

---

### Task 3: .trit file format (writer + reader)

**Files:**
- Create: `crates/trit-core/src/tritfmt.rs`
- Modify: `crates/trit-core/src/lib.rs` (add `pub mod tritfmt;`), `crates/trit-core/Cargo.toml` (add `memmap2 = "0.9"`)

**Interfaces:**
- Consumes: `pack::pack_trits`, `pack::unpack_trits`.
- Produces:
  - `tritfmt::DType` — `enum DType { F32, Trit }`
  - `tritfmt::TensorMeta { name: String, dtype: DType, shape: Vec<usize>, scale: f32 }`
  - `tritfmt::TritWriter::create(path: &Path, config_json: &str) -> Result<TritWriter>`, `.write_f32(name, shape, data: &[f32])`, `.write_trit(name, shape, trits: &[i8], scale: f32)`, `.finish()`
  - `tritfmt::TritReader::open(path: &Path) -> Result<TritReader>`, `.config_json() -> &str`, `.metas() -> &[TensorMeta]`, `.read_f32(name) -> Result<Vec<f32>>`, `.read_trit(name) -> Result<(Vec<i8>, f32)>`

File layout v0 (all integers little-endian):

```
magic b"TRIT" | version: u32 = 0 | config_len: u32 | config JSON bytes
n_tensors: u32
per tensor: name_len u16 | name | dtype u8 (0=F32, 1=Trit) | ndim u8 | dims u32[ndim]
            scale f32 | offset u64 (from payload start) | byte_len u64
payload bytes (tensors in manifest order, contiguous)
```

- [ ] **Step 1: Write the failing test**

In `crates/trit-core/src/tritfmt.rs`:

```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p trit-core tritfmt`
Expected: COMPILE ERROR.

- [ ] **Step 3: Implement**

```rust
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
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p trit-core`
Expected: all pass (9 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/trit-core
git commit -m "feat(trit-core): .trit v0 file format with writer and mmap reader"
```

---

### Task 4: tritc converter (safetensors -> model.trit)

**Files:**
- Create: `crates/tritc/Cargo.toml`, `crates/tritc/src/main.rs`, `crates/tritc/src/convert.rs`
- Modify: root `Cargo.toml` members (add `"crates/tritc"`)

**Interfaces:**
- Consumes: `trit_core::quant::absmean_quantize`, `trit_core::tritfmt::TritWriter`.
- Produces: binary `tritc` with `tritc convert --input <dir> --output model.trit`, and `tritc::convert::convert(input_dir: &Path, output: &Path) -> Result<Report>` where `Report { tensors: usize, ternary_tensors: usize, mean_zero_frac: f32, mean_recon_err: f32 }`.

Conversion rules:
- `<dir>` must contain `config.json` and one or more `*.safetensors` files.
- 2-D tensors whose name ends in one of `q_proj.weight`, `k_proj.weight`, `v_proj.weight`, `o_proj.weight`, `gate_proj.weight`, `up_proj.weight`, `down_proj.weight` are quantized with `absmean_quantize` per-tensor and stored as `Trit`.
- Everything else (embeddings, `lm_head.weight`, all norms) is stored as `F32` (bf16 converted up).
- Report per-tensor reconstruction error `||W - s*Wq|| / ||W||` and zero fraction; print a table.

- [ ] **Step 1: Write the failing test**

In `crates/tritc/src/convert.rs` (test builds a synthetic 1-layer checkpoint with the `safetensors` crate, converts it, reopens with `TritReader`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use safetensors::serialize;
    use safetensors::tensor::{Dtype, TensorView};
    use std::collections::HashMap;

    fn f32_bytes(v: &[f32]) -> Vec<u8> {
        v.iter().flat_map(|x| x.to_le_bytes()).collect()
    }

    #[test]
    fn converts_synthetic_checkpoint() {
        let dir = std::env::temp_dir().join("tritc_test");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("config.json"), r#"{"hidden_size":2}"#).unwrap();

        let q = f32_bytes(&[0.5, -1.2, 0.1, 2.0]); // 2x2, quantizes to [1,-1,0,1], scale 0.95
        let norm = f32_bytes(&[1.0, 1.0]);
        let mut tensors = HashMap::new();
        tensors.insert(
            "model.layers.0.self_attn.q_proj.weight".to_string(),
            TensorView::new(Dtype::F32, vec![2, 2], &q).unwrap(),
        );
        tensors.insert(
            "model.norm.weight".to_string(),
            TensorView::new(Dtype::F32, vec![2], &norm).unwrap(),
        );
        std::fs::write(dir.join("model.safetensors"), serialize(&tensors, &None).unwrap()).unwrap();

        let out = dir.join("model.trit");
        let report = convert(&dir, &out).unwrap();
        assert_eq!(report.tensors, 2);
        assert_eq!(report.ternary_tensors, 1);

        let r = trit_core::tritfmt::TritReader::open(&out).unwrap();
        assert_eq!(r.config_json(), r#"{"hidden_size":2}"#);
        let (trits, scale) = r.read_trit("model.layers.0.self_attn.q_proj.weight").unwrap();
        assert_eq!(trits, vec![1, -1, 0, 1]);
        assert!((scale - 0.95).abs() < 1e-6);
        assert_eq!(r.read_f32("model.norm.weight").unwrap(), vec![1.0, 1.0]);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p tritc`
Expected: COMPILE ERROR.

- [ ] **Step 3: Implement**

`crates/tritc/Cargo.toml`:

```toml
[package]
name = "tritc"
version = "0.1.0"
edition = "2021"

[dependencies]
trit-core = { path = "../trit-core" }
anyhow = "1"
clap = { version = "4", features = ["derive"] }
half = "2"
memmap2 = "0.9"
safetensors = "0.4"
```

`crates/tritc/src/convert.rs` core (plus the test above):

```rust
use anyhow::{Context, Result};
use half::bf16;
use safetensors::tensor::Dtype;
use safetensors::SafeTensors;
use std::path::Path;
use trit_core::quant::absmean_quantize;
use trit_core::tritfmt::TritWriter;

const TERNARY_SUFFIXES: &[&str] = &[
    "q_proj.weight", "k_proj.weight", "v_proj.weight", "o_proj.weight",
    "gate_proj.weight", "up_proj.weight", "down_proj.weight",
];

pub struct Report {
    pub tensors: usize,
    pub ternary_tensors: usize,
    pub mean_zero_frac: f32,
    pub mean_recon_err: f32,
}

fn to_f32(dtype: Dtype, data: &[u8]) -> Result<Vec<f32>> {
    Ok(match dtype {
        Dtype::F32 => data
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
            .collect(),
        Dtype::BF16 => data
            .chunks_exact(2)
            .map(|c| bf16::from_le_bytes(c.try_into().unwrap()).to_f32())
            .collect(),
        Dtype::F16 => data
            .chunks_exact(2)
            .map(|c| half::f16::from_le_bytes(c.try_into().unwrap()).to_f32())
            .collect(),
        d => anyhow::bail!("unsupported dtype {d:?}"),
    })
}

pub fn convert(input_dir: &Path, output: &Path) -> Result<Report> {
    let config = std::fs::read_to_string(input_dir.join("config.json"))
        .context("config.json missing")?;
    let mut writer = TritWriter::create(output, &config)?;

    let mut st_files: Vec<_> = std::fs::read_dir(input_dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "safetensors"))
        .collect();
    st_files.sort();
    anyhow::ensure!(!st_files.is_empty(), "no .safetensors files in {}", input_dir.display());

    let (mut n, mut n_tern, mut zsum, mut esum) = (0usize, 0usize, 0f32, 0f32);
    for path in st_files {
        let file = std::fs::File::open(&path)?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let st = SafeTensors::deserialize(&mmap)?;
        for (name, view) in st.tensors() {
            let shape: Vec<usize> = view.shape().to_vec();
            let data = to_f32(view.dtype(), view.data())?;
            let is_ternary = shape.len() == 2
                && TERNARY_SUFFIXES.iter().any(|s| name.ends_with(s));
            if is_ternary {
                let (trits, scale) = absmean_quantize(&data);
                let zeros = trits.iter().filter(|&&t| t == 0).count() as f32 / trits.len() as f32;
                let mut err = 0f32;
                let mut norm = 0f32;
                for (w, t) in data.iter().zip(&trits) {
                    let d = w - scale * *t as f32;
                    err += d * d;
                    norm += w * w;
                }
                let recon = (err / norm.max(1e-12)).sqrt();
                println!("{name:60} trit {shape:?} zeros={zeros:.3} err={recon:.4}");
                zsum += zeros;
                esum += recon;
                n_tern += 1;
                writer.write_trit(&name, &shape, &trits, scale)?;
            } else {
                writer.write_f32(&name, &shape, &data)?;
            }
            n += 1;
        }
    }
    writer.finish()?;
    Ok(Report {
        tensors: n,
        ternary_tensors: n_tern,
        mean_zero_frac: if n_tern > 0 { zsum / n_tern as f32 } else { 0.0 },
        mean_recon_err: if n_tern > 0 { esum / n_tern as f32 } else { 0.0 },
    })
}
```

`crates/tritc/src/main.rs`:

```rust
mod convert;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tritc", about = "tritium model converter")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Convert a HF checkpoint directory (config.json + *.safetensors) to .trit
    Convert {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().cmd {
        Cmd::Convert { input, output } => {
            let r = convert::convert(&input, &output)?;
            println!(
                "converted {} tensors ({} ternary), mean zero frac {:.3}, mean recon err {:.4}",
                r.tensors, r.ternary_tensors, r.mean_zero_frac, r.mean_recon_err
            );
        }
    }
    Ok(())
}
```

Add `"crates/tritc"` to workspace members.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p tritc`
Expected: 1 passed.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/tritc
git commit -m "feat(tritc): safetensors to .trit converter with quantization report"
```

---

### Task 5: Download the real checkpoint and convert it (manual validation gate)

**Files:**
- Create: `scripts/fetch_model.sh`
- No unit tests — this is a validation gate producing `models/` artifacts (git-ignored).

**Interfaces:**
- Consumes: `tritc convert`.
- Produces: `models/bitnet-2b4t/` (HF download) and `models/bitnet-2b4t.trit`; confirmed tensor-name list for Task 7.

- [ ] **Step 1: Write the fetch script and .gitignore entry**

`scripts/fetch_model.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
# Downloads the bf16 master weights for BitNet b1.58 2B4T (~5 GB).
# Requires: pip install "huggingface_hub[cli]"
MODEL_DIR="${1:-models/bitnet-2b4t}"
hf download microsoft/bitnet-b1.58-2B-4T-bf16 \
  --include "*.safetensors" "config.json" "tokenizer*" \
  --local-dir "$MODEL_DIR"
echo "downloaded to $MODEL_DIR"
```

Append to `.gitignore` (create if missing): `models/` and `*.trit`.

- [ ] **Step 2: Run the download**

Run: `chmod +x scripts/fetch_model.sh && ./scripts/fetch_model.sh`
Expected: `models/bitnet-2b4t/` contains `config.json`, `tokenizer.json`, and safetensors files. If the `-bf16` repo name has changed, find the current master-weights repo under `huggingface.co/microsoft` (search "bitnet") and update the script — do not silently substitute a packed/GGUF repo.

- [ ] **Step 3: Inspect tensor names and config**

Run: `python3 -c "import json,sys; d=json.load(open('models/bitnet-2b4t/config.json')); print(json.dumps(d,indent=1))"` and
`python3 -c "from safetensors import safe_open; f=safe_open([str(p) for p in __import__('pathlib').Path('models/bitnet-2b4t').glob('*.safetensors')][0],'np'); [print(k) for k in list(f.keys())[:40]]"`

Record in `docs/superpowers/plans/checkpoint-notes.md`: hidden_size, intermediate_size, num_hidden_layers, num_attention_heads, num_key_value_heads, rope_theta, rms_norm_eps, vocab_size, hidden_act, whether `attn_sub_norm` / `ffn_sub_norm` tensors exist, whether `lm_head.weight` exists (vs tied embeddings), and the exact projection tensor names. **If names differ from the `TERNARY_SUFFIXES` list in Task 4 or the names Task 7 loads, update those constants now.**

- [ ] **Step 4: Convert**

Run: `cargo run -p tritc --release -- convert --input models/bitnet-2b4t --output models/bitnet-2b4t.trit`
Expected: report prints; sanity thresholds: mean zero frac in 0.1–0.6, mean recon err < 0.15 (BitNet master weights sit close to their ternary points; a large error means wrong repo or wrong quant recipe — stop and investigate, do not proceed).

- [ ] **Step 5: Commit**

```bash
git add scripts/fetch_model.sh .gitignore docs/superpowers/plans/checkpoint-notes.md
git commit -m "chore: checkpoint fetch script and conversion notes"
```

---

### Task 6: tritsim math blocks (rmsnorm, softmax, rope, activations)

**Files:**
- Create: `crates/tritsim/Cargo.toml`, `crates/tritsim/src/main.rs` (stub), `crates/tritsim/src/math.rs`, `crates/tritsim/src/lib.rs`
- Modify: root `Cargo.toml` members (add `"crates/tritsim"`)

**Interfaces:**
- Consumes: nothing from other crates yet.
- Produces (all in `tritsim::math`):
  - `rmsnorm(x: &[f32], gain: &[f32], eps: f32) -> Vec<f32>`
  - `softmax_inplace(x: &mut [f32])`
  - `rope_inplace(v: &mut [f32], head_dim: usize, pos: usize, theta: f32)` — rotate-half convention (pairs `(v[i], v[i + head_dim/2])`), applied per head over a flat `[n_heads * head_dim]` buffer
  - `activate(act: Act, gate: f32) -> f32` with `enum Act { Silu, Relu2 }`

- [ ] **Step 1: Write the failing tests**

In `crates/tritsim/src/math.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rmsnorm_hand_computed() {
        // x=[3,4]: mean sq = 12.5, rms = 3.53553; y = x/rms * g
        let y = rmsnorm(&[3.0, 4.0], &[1.0, 2.0], 0.0);
        assert!((y[0] - 0.848528).abs() < 1e-5);
        assert!((y[1] - 2.262742).abs() < 1e-5);
    }

    #[test]
    fn softmax_sums_to_one_and_orders() {
        let mut x = vec![1.0, 2.0, 3.0];
        softmax_inplace(&mut x);
        assert!((x.iter().sum::<f32>() - 1.0).abs() < 1e-6);
        assert!(x[2] > x[1] && x[1] > x[0]);
    }

    #[test]
    fn rope_position_zero_is_identity() {
        let mut v = vec![1.0, 2.0, 3.0, 4.0]; // one head, dim 4
        rope_inplace(&mut v, 4, 0, 10000.0);
        assert_eq!(v, vec![1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn rope_hand_computed_dim2() {
        // head_dim=2: single pair (v[0], v[1]), inv_freq=1, angle=pos*1=1 rad
        let mut v = vec![1.0, 0.0];
        rope_inplace(&mut v, 2, 1, 10000.0);
        assert!((v[0] - 1f32.cos()).abs() < 1e-6);
        assert!((v[1] - 1f32.sin()).abs() < 1e-6);
    }

    #[test]
    fn activations() {
        assert_eq!(activate(Act::Relu2, -2.0), 0.0);
        assert_eq!(activate(Act::Relu2, 3.0), 9.0);
        // silu(1) = 1/(1+e^-1) = 0.731058
        assert!((activate(Act::Silu, 1.0) - 0.731058).abs() < 1e-5);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p tritsim`
Expected: COMPILE ERROR.

- [ ] **Step 3: Implement**

`crates/tritsim/Cargo.toml`:

```toml
[package]
name = "tritsim"
version = "0.1.0"
edition = "2021"

[dependencies]
trit-core = { path = "../trit-core" }
anyhow = "1"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokenizers = "0.21"

[lib]
name = "tritsim"
path = "src/lib.rs"
```

`src/lib.rs`: `pub mod math;` (modules added per task). `src/main.rs` stub: `fn main() {}`.

`crates/tritsim/src/math.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Act {
    Silu,
    Relu2,
}

pub fn rmsnorm(x: &[f32], gain: &[f32], eps: f32) -> Vec<f32> {
    let ms = x.iter().map(|v| v * v).sum::<f32>() / x.len() as f32;
    let r = 1.0 / (ms + eps).sqrt();
    x.iter().zip(gain).map(|(v, g)| v * r * g).collect()
}

pub fn softmax_inplace(x: &mut [f32]) {
    let m = x.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0;
    for v in x.iter_mut() {
        *v = (*v - m).exp();
        sum += *v;
    }
    for v in x.iter_mut() {
        *v /= sum;
    }
}

/// Rotate-half RoPE (HF LLaMA convention): pairs are (v[i], v[i + head_dim/2]).
pub fn rope_inplace(v: &mut [f32], head_dim: usize, pos: usize, theta: f32) {
    let half = head_dim / 2;
    for head in v.chunks_mut(head_dim) {
        for i in 0..half {
            let inv_freq = theta.powf(-2.0 * i as f32 / head_dim as f32);
            let angle = pos as f32 * inv_freq;
            let (sin, cos) = angle.sin_cos();
            let (a, b) = (head[i], head[i + half]);
            head[i] = a * cos - b * sin;
            head[i + half] = a * sin + b * cos;
        }
    }
}

pub fn activate(act: Act, x: f32) -> f32 {
    match act {
        Act::Silu => x / (1.0 + (-x).exp()),
        Act::Relu2 => {
            let r = x.max(0.0);
            r * r
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p tritsim`
Expected: 5 passed.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/tritsim
git commit -m "feat(tritsim): rmsnorm, softmax, rotate-half rope, activations"
```

---

### Task 7: Model loading + transformer forward pass

**Files:**
- Create: `crates/tritsim/src/config.rs`, `crates/tritsim/src/model.rs`
- Modify: `crates/tritsim/src/lib.rs` (add `pub mod config; pub mod model;`)

**Interfaces:**
- Consumes: `trit_core::tritfmt::TritReader`, `trit_core::quant::absmax_quantize`, `trit_core::matvec::ternary_matvec`, `tritsim::math::*`.
- Produces:
  - `config::ModelConfig { hidden_size: usize, intermediate_size: usize, num_layers: usize, num_heads: usize, num_kv_heads: usize, vocab_size: usize, rope_theta: f32, rms_eps: f32, act: Act, max_seq: usize }` with `ModelConfig::from_json(&str) -> Result<Self>` (serde on HF config.json field names: `num_hidden_layers`, `num_attention_heads`, `num_key_value_heads`, `rms_norm_eps`, `hidden_act` where `"silu" -> Silu`, `"relu2"|"relu-squared" -> Relu2`; `max_seq` fixed at 2048).
  - `model::Model::load(path: &Path) -> Result<Model>`, `model::KvCache::new(cfg: &ModelConfig) -> KvCache`, `Model::forward(&self, token: u32, pos: usize, cache: &mut KvCache) -> Vec<f32>` (returns logits, `vocab_size` long).
  - `model::BitLinear { trits: Vec<i8>, rows: usize, cols: usize, w_scale: f32 }` with `fn apply(&self, x: &[f32]) -> Vec<f32>` = absmax-quantize x, `ternary_matvec`, then `y[i] = acc[i] as f32 * w_scale * x_scale`.

Tensor names loaded (verified/corrected in Task 5): `model.embed_tokens.weight` (F32), per layer `model.layers.{i}.` + `input_layernorm.weight`, `self_attn.{q,k,v,o}_proj.weight`, `self_attn.attn_sub_norm.weight` (optional), `post_attention_layernorm.weight`, `mlp.{gate,up,down}_proj.weight`, `mlp.ffn_sub_norm.weight` (optional), then `model.norm.weight`, and `lm_head.weight` (fall back to `model.embed_tokens.weight` if absent).

Forward pass per layer (BitNet b1.58 structure):

```
x  += attn_block(rmsnorm(x, input_layernorm)):
        q,k,v = BitLinear each; rope(q), rope(k) at pos; cache.append(k, v)
        per q-head h (kv head = h * num_kv_heads / num_heads):
          scores[t] = dot(q_h, k_h[t]) / sqrt(head_dim)  for t <= pos; softmax; ctx_h = sum scores[t]*v_h[t]
        ctx = concat(ctx_h); if attn_sub_norm: ctx = rmsnorm(ctx, sub_norm); out = BitLinear_o(ctx)
x  += mlp_block(rmsnorm(x, post_attention_layernorm)):
        g = BitLinear_gate(h); u = BitLinear_up(h); a = activate(act, g) * u  (elementwise)
        if ffn_sub_norm: a = rmsnorm(a, sub_norm); out = BitLinear_down(a)
logits = matmul_f32(lm_head, rmsnorm(x, model.norm))
```

- [ ] **Step 1: Write the failing tests**

Config test in `crates/tritsim/src/config.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Act;

    #[test]
    fn parses_hf_config() {
        let cfg = ModelConfig::from_json(
            r#"{"hidden_size":64,"intermediate_size":128,"num_hidden_layers":2,
                "num_attention_heads":4,"num_key_value_heads":2,"vocab_size":100,
                "rope_theta":10000.0,"rms_norm_eps":1e-5,"hidden_act":"relu2"}"#,
        )
        .unwrap();
        assert_eq!(cfg.num_layers, 2);
        assert_eq!(cfg.num_kv_heads, 2);
        assert_eq!(cfg.act, Act::Relu2);
        assert_eq!(cfg.max_seq, 2048);
    }
}
```

BitLinear test in `crates/tritsim/src/model.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitlinear_matches_dequantized_float_matmul() {
        // W ternary [[1,-1],[0,1]] scale 0.5; x = [1.0, -2.0]
        // dequant W = [[0.5,-0.5],[0,0.5]]; exact y = [1.5, -1.0]
        // int path: absmax x -> scale 2/127, xq=[64,-127]
        // acc = [64+127, -127] = [191, -127]; y = acc * 0.5 * (2/127) = [1.50394, -1.0]
        let bl = BitLinear { trits: vec![1, -1, 0, 1], rows: 2, cols: 2, w_scale: 0.5 };
        let y = bl.apply(&[1.0, -2.0]);
        assert!((y[0] - 1.5).abs() < 0.01, "y0={}", y[0]);
        assert!((y[1] - -1.0).abs() < 0.01, "y1={}", y[1]);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p tritsim`
Expected: COMPILE ERROR.

- [ ] **Step 3: Implement config.rs**

```rust
use crate::math::Act;
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
struct Raw {
    hidden_size: usize,
    intermediate_size: usize,
    num_hidden_layers: usize,
    num_attention_heads: usize,
    num_key_value_heads: Option<usize>,
    vocab_size: usize,
    rope_theta: Option<f32>,
    rms_norm_eps: Option<f32>,
    hidden_act: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ModelConfig {
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub num_layers: usize,
    pub num_heads: usize,
    pub num_kv_heads: usize,
    pub vocab_size: usize,
    pub rope_theta: f32,
    pub rms_eps: f32,
    pub act: Act,
    pub max_seq: usize,
}

impl ModelConfig {
    pub fn from_json(s: &str) -> Result<Self> {
        let r: Raw = serde_json::from_str(s)?;
        let act = match r.hidden_act.as_deref() {
            Some("relu2") | Some("relu-squared") | Some("relu_squared") => Act::Relu2,
            _ => Act::Silu,
        };
        Ok(Self {
            hidden_size: r.hidden_size,
            intermediate_size: r.intermediate_size,
            num_layers: r.num_hidden_layers,
            num_heads: r.num_attention_heads,
            num_kv_heads: r.num_key_value_heads.unwrap_or(r.num_attention_heads),
            vocab_size: r.vocab_size,
            rope_theta: r.rope_theta.unwrap_or(10000.0),
            rms_eps: r.rms_norm_eps.unwrap_or(1e-5),
            act,
            max_seq: 2048,
        })
    }
    pub fn head_dim(&self) -> usize {
        self.hidden_size / self.num_heads
    }
}
```

- [ ] **Step 4: Implement model.rs**

```rust
use crate::config::ModelConfig;
use crate::math::{activate, rmsnorm, rope_inplace, softmax_inplace};
use anyhow::{Context, Result};
use std::path::Path;
use trit_core::matvec::ternary_matvec;
use trit_core::quant::absmax_quantize;
use trit_core::tritfmt::TritReader;

pub struct BitLinear {
    pub trits: Vec<i8>,
    pub rows: usize,
    pub cols: usize,
    pub w_scale: f32,
}

impl BitLinear {
    pub fn apply(&self, x: &[f32]) -> Vec<f32> {
        let (xq, x_scale) = absmax_quantize(x);
        ternary_matvec(&self.trits, self.rows, self.cols, &xq)
            .into_iter()
            .map(|acc| acc as f32 * self.w_scale * x_scale)
            .collect()
    }
}

pub struct Layer {
    pub input_norm: Vec<f32>,
    pub q: BitLinear,
    pub k: BitLinear,
    pub v: BitLinear,
    pub o: BitLinear,
    pub attn_sub_norm: Option<Vec<f32>>,
    pub post_norm: Vec<f32>,
    pub gate: BitLinear,
    pub up: BitLinear,
    pub down: BitLinear,
    pub ffn_sub_norm: Option<Vec<f32>>,
}

pub struct Model {
    pub cfg: ModelConfig,
    pub embed: Vec<f32>, // vocab x hidden
    pub layers: Vec<Layer>,
    pub final_norm: Vec<f32>,
    pub lm_head: Vec<f32>, // vocab x hidden
}

pub struct KvCache {
    // per layer: [max_seq * num_kv_heads * head_dim]
    pub k: Vec<Vec<f32>>,
    pub v: Vec<Vec<f32>>,
    pub len: usize,
}

impl KvCache {
    pub fn new(cfg: &ModelConfig) -> Self {
        let per = cfg.max_seq * cfg.num_kv_heads * cfg.head_dim();
        Self {
            k: (0..cfg.num_layers).map(|_| vec![0.0; per]).collect(),
            v: (0..cfg.num_layers).map(|_| vec![0.0; per]).collect(),
            len: 0,
        }
    }
}

impl Model {
    pub fn load(path: &Path) -> Result<Self> {
        let r = TritReader::open(path)?;
        let cfg = ModelConfig::from_json(r.config_json())?;
        let bl = |name: &str| -> Result<BitLinear> {
            let m = r
                .metas()
                .iter()
                .find(|m| m.name == name)
                .with_context(|| format!("missing {name}"))?;
            let (rows, cols) = (m.shape[0], m.shape[1]);
            let (trits, w_scale) = r.read_trit(name)?;
            Ok(BitLinear { trits, rows, cols, w_scale })
        };
        let f32_opt = |name: &str| r.read_f32(name).ok();

        let mut layers = Vec::with_capacity(cfg.num_layers);
        for i in 0..cfg.num_layers {
            let p = format!("model.layers.{i}.");
            layers.push(Layer {
                input_norm: r.read_f32(&format!("{p}input_layernorm.weight"))?,
                q: bl(&format!("{p}self_attn.q_proj.weight"))?,
                k: bl(&format!("{p}self_attn.k_proj.weight"))?,
                v: bl(&format!("{p}self_attn.v_proj.weight"))?,
                o: bl(&format!("{p}self_attn.o_proj.weight"))?,
                attn_sub_norm: f32_opt(&format!("{p}self_attn.attn_sub_norm.weight")),
                post_norm: r.read_f32(&format!("{p}post_attention_layernorm.weight"))?,
                gate: bl(&format!("{p}mlp.gate_proj.weight"))?,
                up: bl(&format!("{p}mlp.up_proj.weight"))?,
                down: bl(&format!("{p}mlp.down_proj.weight"))?,
                ffn_sub_norm: f32_opt(&format!("{p}mlp.ffn_sub_norm.weight")),
            });
        }
        let embed = r.read_f32("model.embed_tokens.weight")?;
        let lm_head = r.read_f32("lm_head.weight").unwrap_or_else(|_| embed.clone());
        Ok(Self {
            final_norm: r.read_f32("model.norm.weight")?,
            embed,
            lm_head,
            layers,
            cfg,
        })
    }

    pub fn forward(&self, token: u32, pos: usize, cache: &mut KvCache) -> Vec<f32> {
        let cfg = &self.cfg;
        let (hd, nh, nkv) = (cfg.head_dim(), cfg.num_heads, cfg.num_kv_heads);
        let h = cfg.hidden_size;
        let mut x = self.embed[token as usize * h..(token as usize + 1) * h].to_vec();

        for (li, layer) in self.layers.iter().enumerate() {
            // attention block
            let xn = rmsnorm(&x, &layer.input_norm, cfg.rms_eps);
            let mut q = layer.q.apply(&xn);
            let mut k = layer.k.apply(&xn);
            let v = layer.v.apply(&xn);
            rope_inplace(&mut q, hd, pos, cfg.rope_theta);
            rope_inplace(&mut k, hd, pos, cfg.rope_theta);
            let kc = &mut cache.k[li];
            let vc = &mut cache.v[li];
            kc[pos * nkv * hd..(pos + 1) * nkv * hd].copy_from_slice(&k);
            vc[pos * nkv * hd..(pos + 1) * nkv * hd].copy_from_slice(&v);

            let mut ctx = vec![0.0f32; nh * hd];
            let scale = 1.0 / (hd as f32).sqrt();
            for head in 0..nh {
                let kvh = head * nkv / nh;
                let qh = &q[head * hd..(head + 1) * hd];
                let mut scores: Vec<f32> = (0..=pos)
                    .map(|t| {
                        let kh = &kc[t * nkv * hd + kvh * hd..t * nkv * hd + (kvh + 1) * hd];
                        qh.iter().zip(kh).map(|(a, b)| a * b).sum::<f32>() * scale
                    })
                    .collect();
                softmax_inplace(&mut scores);
                let out = &mut ctx[head * hd..(head + 1) * hd];
                for (t, s) in scores.iter().enumerate() {
                    let vh = &vc[t * nkv * hd + kvh * hd..t * nkv * hd + (kvh + 1) * hd];
                    for d in 0..hd {
                        out[d] += s * vh[d];
                    }
                }
            }
            let ctx = match &layer.attn_sub_norm {
                Some(g) => rmsnorm(&ctx, g, cfg.rms_eps),
                None => ctx,
            };
            let attn_out = layer.o.apply(&ctx);
            for i in 0..h {
                x[i] += attn_out[i];
            }

            // mlp block
            let xn = rmsnorm(&x, &layer.post_norm, cfg.rms_eps);
            let g = layer.gate.apply(&xn);
            let u = layer.up.apply(&xn);
            let a: Vec<f32> = g
                .iter()
                .zip(&u)
                .map(|(gv, uv)| activate(cfg.act, *gv) * uv)
                .collect();
            let a = match &layer.ffn_sub_norm {
                Some(gain) => rmsnorm(&a, gain, cfg.rms_eps),
                None => a,
            };
            let mlp_out = layer.down.apply(&a);
            for i in 0..h {
                x[i] += mlp_out[i];
            }
        }
        cache.len = pos + 1;

        // logits: plain f32 matmul against lm_head (not ternary by design)
        let xn = rmsnorm(&x, &self.final_norm, cfg.rms_eps);
        (0..cfg.vocab_size)
            .map(|v| {
                self.lm_head[v * h..(v + 1) * h]
                    .iter()
                    .zip(&xn)
                    .map(|(a, b)| a * b)
                    .sum()
            })
            .collect()
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p tritsim`
Expected: all pass (7 tests).

- [ ] **Step 6: Commit**

```bash
git add crates/tritsim
git commit -m "feat(tritsim): model loading and full transformer forward pass"
```

---

### Task 8: End-to-end property tests on a tiny synthetic model

**Files:**
- Test: `crates/tritsim/tests/tiny_model.rs`

**Interfaces:**
- Consumes: `TritWriter` (to build the model file), `tritsim::model::{Model, KvCache}`, `tritsim::config::ModelConfig`.
- Produces: confidence that the decode loop is deterministic, causal, and numerically sane — the properties the RTL will later be held to.

- [ ] **Step 1: Write the failing tests**

`crates/tritsim/tests/tiny_model.rs`:

```rust
use std::path::PathBuf;
use trit_core::tritfmt::TritWriter;
use tritsim::model::{KvCache, Model};

/// Deterministic pseudo-random floats without rand: xorshift on a fixed seed.
struct Rng(u64);
impl Rng {
    fn next_f32(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        ((self.0 >> 40) as f32 / (1u64 << 24) as f32) - 0.5
    }
    fn vec(&mut self, n: usize) -> Vec<f32> {
        (0..n).map(|_| self.next_f32()).collect()
    }
    fn trits(&mut self, n: usize) -> Vec<i8> {
        (0..n)
            .map(|_| match (self.next_f32() * 3.0).abs() as u32 {
                0 => 0i8,
                1 => 1,
                _ => -1,
            })
            .collect()
    }
}

const CFG: &str = r#"{"hidden_size":16,"intermediate_size":32,"num_hidden_layers":2,
  "num_attention_heads":4,"num_key_value_heads":2,"vocab_size":32,
  "rope_theta":10000.0,"rms_norm_eps":1e-5,"hidden_act":"relu2"}"#;

fn build_tiny_model() -> PathBuf {
    let path = std::env::temp_dir().join("tritsim_tiny.trit");
    let mut rng = Rng(0x5eed_2026);
    let mut w = TritWriter::create(&path, CFG).unwrap();
    let (h, ff, kvh) = (16usize, 32usize, 8usize); // kvh = num_kv_heads * head_dim = 2*4
    w.write_f32("model.embed_tokens.weight", &[32, h], &rng.vec(32 * h)).unwrap();
    for i in 0..2 {
        let p = format!("model.layers.{i}.");
        let ones = vec![1.0f32; h];
        w.write_f32(&format!("{p}input_layernorm.weight"), &[h], &ones).unwrap();
        w.write_trit(&format!("{p}self_attn.q_proj.weight"), &[h, h], &rng.trits(h * h), 0.1).unwrap();
        w.write_trit(&format!("{p}self_attn.k_proj.weight"), &[kvh, h], &rng.trits(kvh * h), 0.1).unwrap();
        w.write_trit(&format!("{p}self_attn.v_proj.weight"), &[kvh, h], &rng.trits(kvh * h), 0.1).unwrap();
        w.write_trit(&format!("{p}self_attn.o_proj.weight"), &[h, h], &rng.trits(h * h), 0.1).unwrap();
        w.write_f32(&format!("{p}post_attention_layernorm.weight"), &[h], &ones).unwrap();
        w.write_trit(&format!("{p}mlp.gate_proj.weight"), &[ff, h], &rng.trits(ff * h), 0.1).unwrap();
        w.write_trit(&format!("{p}mlp.up_proj.weight"), &[ff, h], &rng.trits(ff * h), 0.1).unwrap();
        w.write_trit(&format!("{p}mlp.down_proj.weight"), &[h, ff], &rng.trits(h * ff), 0.1).unwrap();
    }
    w.write_f32("model.norm.weight", &[16], &vec![1.0; 16]).unwrap();
    w.finish().unwrap();
    path
}

#[test]
fn forward_is_deterministic_and_finite() {
    let path = build_tiny_model();
    let model = Model::load(&path).unwrap();
    let run = || {
        let mut cache = KvCache::new(&model.cfg);
        let mut out = Vec::new();
        for (pos, tok) in [1u32, 5, 9, 2].iter().enumerate() {
            out.push(model.forward(*tok, pos, &mut cache));
        }
        out
    };
    let (a, b) = (run(), run());
    assert_eq!(a, b, "two identical runs must produce identical logits");
    assert_eq!(a[0].len(), 32);
    assert!(a.iter().flatten().all(|v| v.is_finite()));
}

#[test]
fn causality_past_logits_unaffected_by_future_tokens() {
    let path = build_tiny_model();
    let model = Model::load(&path).unwrap();
    let logits_at = |tokens: &[u32], upto: usize| {
        let mut cache = KvCache::new(&model.cfg);
        let mut last = Vec::new();
        for (pos, tok) in tokens.iter().take(upto + 1).enumerate() {
            last = model.forward(*tok, pos, &mut cache);
        }
        last
    };
    // position-1 logits must be identical whatever comes at position 2+
    let a = logits_at(&[1, 5, 9, 2], 1);
    let b = logits_at(&[1, 5, 30, 30], 1);
    assert_eq!(a, b);
}

#[test]
fn greedy_argmax_is_stable() {
    let path = build_tiny_model();
    let model = Model::load(&path).unwrap();
    let mut cache = KvCache::new(&model.cfg);
    let logits = model.forward(3, 0, &mut cache);
    let argmax = |l: &[f32]| l.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).unwrap().0;
    let t1 = argmax(&logits);
    let mut cache2 = KvCache::new(&model.cfg);
    assert_eq!(argmax(&model.forward(3, 0, &mut cache2)), t1);
}
```

Note the tiny model omits `lm_head.weight` (exercises the tied-embedding fallback) and the optional sub-norms (exercises `None` paths).

- [ ] **Step 2: Run tests to verify they fail or pass honestly**

Run: `cargo test -p tritsim --test tiny_model`
Expected: PASS if Tasks 3–7 are correct; any failure here is a real bug — debug with `superpowers:systematic-debugging` before proceeding (do not weaken assertions).

- [ ] **Step 3: Commit**

```bash
git add crates/tritsim/tests
git commit -m "test(tritsim): tiny-model determinism, causality, and stability properties"
```

---

### Task 9: Tokenizer + greedy generation CLI

**Files:**
- Create: `crates/tritsim/src/generate.rs`
- Modify: `crates/tritsim/src/lib.rs` (add `pub mod generate;`), `crates/tritsim/src/main.rs` (real CLI)

**Interfaces:**
- Consumes: `Model`, `KvCache`, `tokenizers::Tokenizer`.
- Produces: `generate::generate(model: &Model, tokenizer: &tokenizers::Tokenizer, prompt: &str, steps: usize, eos_id: Option<u32>) -> Result<String>` and CLI `tritsim run --model <path> --tokenizer <path> --prompt <text> --steps <n>`.

- [ ] **Step 1: Write the failing test**

Append to `crates/tritsim/tests/tiny_model.rs` (uses a trivial whitespace tokenizer built in-test, since `tokenizer.json` files are large):

```rust
#[test]
fn generate_produces_requested_number_of_tokens() {
    use tritsim::generate::greedy_ids;
    let path = build_tiny_model();
    let model = Model::load(&path).unwrap();
    let out = greedy_ids(&model, &[1, 5], 8, None).unwrap();
    assert_eq!(out.len(), 8);
    assert!(out.iter().all(|&t| (t as usize) < 32));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p tritsim --test tiny_model`
Expected: COMPILE ERROR — `greedy_ids` not found.

- [ ] **Step 3: Implement**

`crates/tritsim/src/generate.rs`:

```rust
use crate::model::{KvCache, Model};
use anyhow::Result;

fn argmax(l: &[f32]) -> u32 {
    l.iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .unwrap()
        .0 as u32
}

/// Feed `prompt_ids`, then greedily decode `steps` tokens (stops early on eos).
pub fn greedy_ids(model: &Model, prompt_ids: &[u32], steps: usize, eos_id: Option<u32>) -> Result<Vec<u32>> {
    anyhow::ensure!(!prompt_ids.is_empty(), "empty prompt");
    let mut cache = KvCache::new(&model.cfg);
    let mut logits = Vec::new();
    for (pos, tok) in prompt_ids.iter().enumerate() {
        logits = model.forward(*tok, pos, &mut cache);
    }
    let mut out = Vec::new();
    let mut pos = prompt_ids.len();
    for _ in 0..steps {
        let next = argmax(&logits);
        if Some(next) == eos_id {
            break;
        }
        out.push(next);
        logits = model.forward(next, pos, &mut cache);
        pos += 1;
    }
    Ok(out)
}

pub fn generate(
    model: &Model,
    tokenizer: &tokenizers::Tokenizer,
    prompt: &str,
    steps: usize,
    eos_id: Option<u32>,
) -> Result<String> {
    let enc = tokenizer.encode(prompt, true).map_err(anyhow::Error::msg)?;
    let ids = greedy_ids(model, enc.get_ids(), steps, eos_id)?;
    tokenizer.decode(&ids, true).map_err(anyhow::Error::msg)
}
```

`crates/tritsim/src/main.rs`:

```rust
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tritsim::model::Model;

#[derive(Parser)]
#[command(name = "tritsim", about = "tritium golden-model inference")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run greedy generation
    Run {
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        tokenizer: PathBuf,
        #[arg(long)]
        prompt: String,
        #[arg(long, default_value_t = 64)]
        steps: usize,
    },
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Run { model, tokenizer, prompt, steps } => {
            let m = Model::load(&model)?;
            let tk = tokenizers::Tokenizer::from_file(&tokenizer).map_err(anyhow::Error::msg)?;
            let eos = tk.token_to_id("<|eot_id|>").or_else(|| tk.token_to_id("</s>"));
            let text = tritsim::generate::generate(&m, &tk, &prompt, steps, eos)?;
            println!("{text}");
        }
    }
    Ok(())
}
```

- [ ] **Step 4: Run tests, then run the real model**

Run: `cargo test --workspace`
Expected: all pass.

Run: `cargo run -p tritsim --release -- run --model models/bitnet-2b4t.trit --tokenizer models/bitnet-2b4t/tokenizer.json --prompt "The capital of France is" --steps 16`
Expected: coherent continuation mentioning Paris. Speed will be slow (naive scalar loop, ~5 GB of f32 embed/head in memory) — slow is acceptable, gibberish is not. Gibberish means a real bug (likely rope convention, sub-norm handling, or GQA head mapping) — stop and debug with `superpowers:systematic-debugging`.

- [ ] **Step 5: Commit**

```bash
git add crates/tritsim
git commit -m "feat(tritsim): tokenizer integration and greedy generation CLI"
```

---

### Task 10: Reference logit comparison harness

**Files:**
- Create: `scripts/dump_logits.py`, `crates/tritsim/src/compare.rs`
- Modify: `crates/tritsim/src/lib.rs` (add `pub mod compare;`), `crates/tritsim/src/main.rs` (add `Compare` subcommand)

**Interfaces:**
- Consumes: `Model`, `greedy_ids`-style forward loop.
- Produces: `scripts/dump_logits.py` writing `{"prompt_ids": [..], "logits": [[f32; vocab]; n_pos]}` JSON; `compare::compare(model: &Model, dump_path: &Path) -> Result<CompareStats>` with `CompareStats { positions: usize, mean_cosine: f32, top1_match_frac: f32 }`; CLI `tritsim compare --model <path> --dump logits.json`.

Acceptance thresholds (from BENCHMARKS doc discipline — set before measuring): `mean_cosine >= 0.98` and `top1_match_frac >= 0.90` over 8 positions. Below that: investigate, do not ship.

- [ ] **Step 1: Write the reference dump script**

`scripts/dump_logits.py`:

```python
#!/usr/bin/env python3
"""Dump per-position logits from the HF reference model for comparison with tritsim.

Usage: python3 scripts/dump_logits.py models/bitnet-2b4t "The capital of France is" logits.json
Requires: pip install torch transformers
"""
import json
import sys

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer

model_dir, prompt, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
tok = AutoTokenizer.from_pretrained(model_dir)
model = AutoModelForCausalLM.from_pretrained(
    model_dir, torch_dtype=torch.float32, trust_remote_code=True
)
model.eval()
ids = tok(prompt, return_tensors="pt").input_ids
with torch.no_grad():
    logits = model(ids).logits[0]  # [n_pos, vocab]
json.dump(
    {"prompt_ids": ids[0].tolist(), "logits": logits.tolist()},
    open(out_path, "w"),
)
print(f"wrote {out_path}: {logits.shape[0]} positions x {logits.shape[1]} vocab")
```

- [ ] **Step 2: Write the failing test for the comparison math**

In `crates/tritsim/src/compare.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_of_identical_vectors_is_one() {
        assert!((cosine(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_of_orthogonal_vectors_is_zero() {
        assert!(cosine(&[1.0, 0.0], &[0.0, 1.0]).abs() < 1e-6);
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p tritsim compare`
Expected: COMPILE ERROR.

- [ ] **Step 4: Implement**

`crates/tritsim/src/compare.rs`:

```rust
use crate::model::{KvCache, Model};
use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct Dump {
    prompt_ids: Vec<u32>,
    logits: Vec<Vec<f32>>,
}

pub struct CompareStats {
    pub positions: usize,
    pub mean_cosine: f32,
    pub top1_match_frac: f32,
}

pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (na * nb).max(1e-12)
}

fn argmax(l: &[f32]) -> usize {
    l.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).unwrap().0
}

pub fn compare(model: &Model, dump_path: &Path) -> Result<CompareStats> {
    let d: Dump = serde_json::from_str(&std::fs::read_to_string(dump_path)?)?;
    let mut cache = KvCache::new(&model.cfg);
    let (mut cos_sum, mut top1) = (0f32, 0usize);
    for (pos, tok) in d.prompt_ids.iter().enumerate() {
        let ours = model.forward(*tok, pos, &mut cache);
        let theirs = &d.logits[pos];
        let c = cosine(&ours, theirs);
        let m = argmax(&ours) == argmax(theirs);
        println!("pos {pos:3} cosine {c:.4} top1_match {m}");
        cos_sum += c;
        top1 += m as usize;
    }
    let n = d.prompt_ids.len();
    Ok(CompareStats {
        positions: n,
        mean_cosine: cos_sum / n as f32,
        top1_match_frac: top1 as f32 / n as f32,
    })
}
```

Add to `main.rs` `Cmd` enum and match:

```rust
    /// Compare per-position logits against a reference dump (scripts/dump_logits.py)
    Compare {
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        dump: PathBuf,
    },
```

```rust
        Cmd::Compare { model, dump } => {
            let m = Model::load(&model)?;
            let s = tritsim::compare::compare(&m, &dump)?;
            println!(
                "{} positions: mean cosine {:.4}, top1 match {:.1}%",
                s.positions, s.mean_cosine, s.top1_match_frac * 100.0
            );
            anyhow::ensure!(
                s.mean_cosine >= 0.98 && s.top1_match_frac >= 0.90,
                "below acceptance thresholds (cosine >= 0.98, top1 >= 0.90)"
            );
        }
```

- [ ] **Step 5: Run tests, then the real comparison**

Run: `cargo test --workspace`
Expected: all pass.

Run:
```bash
python3 scripts/dump_logits.py models/bitnet-2b4t "The capital of France is" logits.json
cargo run -p tritsim --release -- compare --model models/bitnet-2b4t.trit --dump logits.json
```
Expected: exits 0 with mean cosine >= 0.98 and top1 >= 90%. If it fails: the divergence position tells you which block to suspect (position 0 failing = embedding/norm/mlp path; later positions only = attention/rope/cache). Debug with `superpowers:systematic-debugging`.

- [ ] **Step 6: Commit**

```bash
git add scripts/dump_logits.py crates/tritsim
git commit -m "feat(tritsim): reference logit comparison harness with acceptance gates"
```

---

### Task 11: Phase 0 wrap-up

**Files:**
- Modify: `README.md` (status section), `docs/02-ROADMAP.md` (tick Week 1 checkboxes that are done)

- [ ] **Step 1: Update docs**

In `README.md`, replace the `## Status` body ("Pre-code. Docs-first....") first line with: `Phase 0 complete: tritsim generates text from the real checkpoint on CPU and matches reference logits (see docs/02-ROADMAP.md for gates passed).` Keep the reading-order list. In `docs/02-ROADMAP.md` Week 1, mark completed items `[x]` — only the ones actually verified.

- [ ] **Step 2: Full verification**

Run: `cargo test --workspace && cargo build --workspace --release`
Expected: everything green. Re-run the Task 10 compare command; confirm it still exits 0.

- [ ] **Step 3: Commit and push**

```bash
git add README.md docs/02-ROADMAP.md
git commit -m "docs: phase 0 complete"
git push origin main
```

---

## Self-Review Notes

- **Spec coverage:** Roadmap Week 1 items all map: workspace scaffold (Task 1), checkpoint load + ternary verify (Tasks 4–5), decode loop with all blocks (Tasks 6–9), reference-logit match (Task 10). The `.trit` freeze pulled forward from Week 2 is Tasks 3–4, per ARCHITECTURE Phase 0 definition. `tritd`/`tritbench` crates deliberately deferred (YAGNI for Phase 0).
- **Known uncertainty, handled explicitly:** exact 2B4T tensor names, `hidden_act` string, and sub-norm presence are pinned by inspection in Task 5 before the model code that depends on them (Task 7). The plan's names are the HF-LLaMA/BitNet defaults; Task 5 Step 3 is the correction point.
- **Type consistency check:** `absmean_quantize`/`absmax_quantize` signatures match across Tasks 1, 4, 7. `ternary_matvec(trits, rows, cols, xq)` matches Tasks 2 and 7. `TritWriter::write_trit(name, shape, trits, scale)` matches Tasks 3, 4, 8. `greedy_ids` matches Tasks 9 test and impl. `KvCache::new(&ModelConfig)` consistent in Tasks 7, 8, 10.
