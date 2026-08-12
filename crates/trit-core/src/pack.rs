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
    let required = n.div_ceil(4);
    if bytes.len() < required {
        bail!("truncated trit data: need {required} bytes for {n} trits, got {}", bytes.len());
    }
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

    #[test]
    fn truncated_data_is_an_error() {
        assert!(unpack_trits(&[0x49], 6).is_err()); // 6 trits need 2 bytes
        assert!(unpack_trits(&[], 1).is_err());
        assert!(unpack_trits(&[], 0).unwrap().is_empty());
    }
}
