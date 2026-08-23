//! Frozen reader for `.trit` **v0**, used only by `tritc upgrade`.
//!
//! v0 stored ternary weights as interleaved 2-bit codes -- `00` = 0, `01` = +1,
//! `10` = -1, `11` illegal -- four trits per byte, trit `k` at bits `2k..2k+2`
//! of byte `k/4`. v1 replaced that with bit planes.
//!
//! This module exists so someone holding only a v0 `.trit`, without the original
//! safetensors checkpoint, can still migrate. It is **frozen**: it is never
//! extended, and `trit-core` contains no v0 code at all, so the runtime cannot
//! accidentally grow a legacy path. The upgrade is a pure re-encoding of the
//! same trits and scales, which `upgrade_matches_a_fresh_convert` proves by
//! requiring the output to be byte-identical to a fresh `tritc convert`.

use anyhow::{bail, Context, Result};
use std::path::Path;

pub struct V0Tensor {
    pub name: String,
    pub shape: Vec<usize>,
    pub scale: f32,
    /// `None` for f32 tensors, whose values are in `f32`.
    pub trits: Option<Vec<i8>>,
    pub f32: Option<Vec<f32>>,
}

pub struct V0File {
    pub config: String,
    pub tensors: Vec<V0Tensor>,
}

fn take<'a>(b: &'a [u8], p: &mut usize, n: usize) -> Result<&'a [u8]> {
    let end = p.checked_add(n).context("length overflow")?;
    let s = b.get(*p..end).context("truncated v0 .trit file")?;
    *p = end;
    Ok(s)
}
fn take_u32(b: &[u8], p: &mut usize) -> Result<u32> {
    Ok(u32::from_le_bytes(take(b, p, 4)?.try_into().unwrap()))
}
fn take_u64(b: &[u8], p: &mut usize) -> Result<u64> {
    Ok(u64::from_le_bytes(take(b, p, 8)?.try_into().unwrap()))
}

/// Decode v0's interleaved 2-bit codes.
fn unpack_v0_trits(bytes: &[u8], n: usize) -> Result<Vec<i8>> {
    let required = n.div_ceil(4);
    if bytes.len() < required {
        bail!(
            "truncated trit data: need {required} bytes for {n} trits, got {}",
            bytes.len()
        );
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        out.push(match (bytes[i / 4] >> ((i % 4) * 2)) & 0b11 {
            0b00 => 0,
            0b01 => 1,
            0b10 => -1,
            _ => bail!("invalid trit encoding 0b11 at index {i}"),
        });
    }
    Ok(out)
}

pub fn read(path: &Path) -> Result<V0File> {
    let raw = std::fs::read(path).with_context(|| format!("open {}", path.display()))?;
    let b = &raw[..];
    let mut p = 0usize;

    if take(b, &mut p, 4)? != b"TRIT" {
        bail!("bad magic: not a .trit file");
    }
    let version = take_u32(b, &mut p)?;
    if version != 0 {
        bail!("this is .trit v{version}, not v0; nothing to upgrade");
    }
    let clen = take_u32(b, &mut p)? as usize;
    let config = std::str::from_utf8(take(b, &mut p, clen)?)?.to_string();
    let n = take_u32(b, &mut p)? as usize;

    struct Meta {
        name: String,
        is_trit: bool,
        shape: Vec<usize>,
        scale: f32,
        offset: u64,
        byte_len: u64,
    }
    let mut metas = Vec::new();
    for _ in 0..n {
        let nlen = u16::from_le_bytes(take(b, &mut p, 2)?.try_into().unwrap()) as usize;
        let name = std::str::from_utf8(take(b, &mut p, nlen)?)?.to_string();
        let is_trit = match take(b, &mut p, 1)?[0] {
            0 => false,
            1 => true,
            x => bail!("bad dtype {x}"),
        };
        let ndim = take(b, &mut p, 1)?[0] as usize;
        let mut shape = Vec::with_capacity(ndim);
        for _ in 0..ndim {
            shape.push(take_u32(b, &mut p)? as usize);
        }
        let scale = f32::from_le_bytes(take(b, &mut p, 4)?.try_into().unwrap());
        let offset = take_u64(b, &mut p)?;
        let byte_len = take_u64(b, &mut p)?;
        metas.push(Meta {
            name,
            is_trit,
            shape,
            scale,
            offset,
            byte_len,
        });
    }
    let payload_start = p;

    let mut tensors = Vec::new();
    for m in metas {
        let start = payload_start
            .checked_add(usize::try_from(m.offset)?)
            .context("tensor offset overflow")?;
        let end = start
            .checked_add(usize::try_from(m.byte_len)?)
            .context("tensor length overflow")?;
        let data = b
            .get(start..end)
            .with_context(|| format!("tensor {} out of bounds", m.name))?;
        let elems: usize = m.shape.iter().product();
        if m.is_trit {
            tensors.push(V0Tensor {
                trits: Some(unpack_v0_trits(data, elems)?),
                f32: None,
                name: m.name,
                shape: m.shape,
                scale: m.scale,
            });
        } else {
            if data.len() != elems * 4 {
                bail!(
                    "{}: byte_len {} does not match shape ({elems} f32s)",
                    m.name,
                    data.len()
                );
            }
            tensors.push(V0Tensor {
                trits: None,
                f32: Some(
                    data.chunks_exact(4)
                        .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
                        .collect(),
                ),
                name: m.name,
                shape: m.shape,
                scale: m.scale,
            });
        }
    }
    Ok(V0File { config, tensors })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_the_v0_code_layout() {
        // byte 0: trit0=01(+1), trit1=10(-1), trit2=00(0), trit3=01(+1) -> 0x49
        assert_eq!(unpack_v0_trits(&[0x49], 4).unwrap(), vec![1, -1, 0, 1]);
    }

    #[test]
    fn rejects_the_illegal_code_and_truncation() {
        assert!(unpack_v0_trits(&[0b0000_0011], 1).is_err());
        assert!(unpack_v0_trits(&[0x49], 6).is_err());
    }
}
