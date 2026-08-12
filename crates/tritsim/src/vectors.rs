use anyhow::{Context, Result};
use std::fmt::Write as _;
use std::path::Path;
use trit_core::matvec::ternary_matvec;

/// Pad columns to a LANES multiple with zero trits / zero activations so the
/// RTL always consumes whole 64-trit beats. Zero-padding is exact: padded
/// terms contribute nothing to the accumulator.
pub const LANES: usize = 64;

pub fn write_set(dir: &Path, name: &str, rows: usize, cols: usize, trits: &[i8], x: &[i8]) -> Result<()> {
    assert_eq!(trits.len(), rows * cols);
    assert_eq!(x.len(), cols);
    let cp = cols.div_ceil(LANES) * LANES; // cols padded
    let d = dir.join(name);
    std::fs::create_dir_all(&d)?;
    std::fs::write(d.join("meta.txt"), format!("{rows} {cp}\n"))?;

    let mut xh = String::new();
    for c in 0..cp {
        let v = if c < cols { x[c] } else { 0 };
        writeln!(xh, "{:02x}", v as u8)?;
    }
    std::fs::write(d.join("x.hex"), xh)?;

    let mut wh = String::new();
    for r in 0..rows {
        for beat in 0..cp / LANES {
            let mut word: u128 = 0;
            for l in 0..LANES {
                let c = beat * LANES + l;
                let t = if c < cols { trits[r * cols + c] } else { 0 };
                let code: u128 = match t {
                    0 => 0b00,
                    1 => 0b01,
                    -1 => 0b10,
                    _ => panic!("non-ternary {t}"),
                };
                word |= code << (2 * l);
            }
            writeln!(wh, "{word:032x}")?;
        }
    }
    std::fs::write(d.join("w.hex"), wh)?;

    let y = ternary_matvec(trits, rows, cols, x);
    let mut yh = String::new();
    for v in y {
        writeln!(yh, "{:08x}", v as u32)?;
    }
    std::fs::write(d.join("y.hex"), yh)?;
    Ok(())
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn trit(&mut self) -> i8 {
        [(-1i8), 0, 1][(self.next() % 3) as usize]
    }
    fn i8(&mut self) -> i8 {
        (self.next() & 0xff) as u8 as i8
    }
}

fn rand_set(dir: &Path, name: &str, rows: usize, cols: usize, seed: u64) -> Result<()> {
    let mut rng = Rng(seed);
    let trits: Vec<i8> = (0..rows * cols).map(|_| rng.trit()).collect();
    let x: Vec<i8> = (0..cols).map(|_| rng.i8()).collect();
    write_set(dir, name, rows, cols, &trits, &x)
}

pub fn generate_all(out: &Path) -> Result<()> {
    rand_set(out, "exact_64x64", 64, 64, 0xA11CE)?;
    rand_set(out, "padded_5x100", 5, 100, 0xB0B)?;
    rand_set(out, "wide_2x6912", 2, 6912, 0x5EED)?;
    // extremes: worst-case accumulator magnitude, no zeros anywhere
    let mut rng = Rng(0xFFF);
    let trits: Vec<i8> = (0..4 * 128).map(|_| if rng.next() & 1 == 0 { 1 } else { -1 }).collect();
    let x: Vec<i8> = (0..128).map(|_| if rng.next() & 1 == 0 { -128 } else { 127 }).collect();
    write_set(out, "extremes_4x128", 4, 128, &trits, &x)?;
    write_set(out, "zeros_3x64", 3, 64, &vec![0i8; 3 * 64], &(0..64).map(|i| i as i8).collect::<Vec<_>>())?;
    Ok(())
}

/// Vectors cut from the actual checkpoint: first `max_rows` of layer-0 k_proj
/// against a deterministic random activation, using the exact trits the model
/// runs. Guards against generator-side packing bugs that random sets share.
pub fn model_tile_set(out: &Path, model: &Path, name: &str, max_rows: usize) -> Result<()> {
    let r = trit_core::tritfmt::TritReader::open(model)?;
    let tname = "model.layers.0.self_attn.k_proj.weight";
    let meta = r
        .metas()
        .iter()
        .find(|m| m.name == tname)
        .with_context(|| format!("{tname} not in model"))?;
    let (all, _scale) = r.read_trit(tname)?;
    let cols = meta.shape[1];
    let rows = meta.shape[0].min(max_rows);
    let trits = &all[..rows * cols];
    let mut rng = Rng(0xD1CE);
    let x: Vec<i8> = (0..cols).map(|_| rng.i8()).collect();
    write_set(out, name, rows, cols, trits, &x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_roundtrips_and_matches_golden() {
        let dir = std::env::temp_dir().join("tritsim_vectors_test");
        std::fs::create_dir_all(&dir).unwrap();
        // 2x5 with padding to 64: y = [10-20, 30+50] = [-10, 80]
        let trits: Vec<i8> = vec![1, -1, 0, 0, 0, 0, 0, 1, 0, 1];
        let x: Vec<i8> = vec![10, 20, 30, 40, 50];
        write_set(&dir, "t", 2, 5, &trits, &x).unwrap();

        let meta = std::fs::read_to_string(dir.join("t/meta.txt")).unwrap();
        assert_eq!(meta.trim(), "2 64");
        let xh: Vec<String> = std::fs::read_to_string(dir.join("t/x.hex"))
            .unwrap().lines().map(String::from).collect();
        assert_eq!(xh.len(), 64);
        assert_eq!(xh[0], "0a");
        assert_eq!(xh[1], "14");
        assert_eq!(xh[5], "00"); // padding
        let wh: Vec<String> = std::fs::read_to_string(dir.join("t/w.hex"))
            .unwrap().lines().map(String::from).collect();
        assert_eq!(wh.len(), 2); // one beat per row
        // row 0: trit0=+1 (bits 1:0 = 01), trit1=-1 (bits 3:2 = 10) -> low byte 0b1001 = 0x09
        assert!(wh[0].ends_with("09"), "beat {}", wh[0]);
        let yh: Vec<String> = std::fs::read_to_string(dir.join("t/y.hex"))
            .unwrap().lines().map(String::from).collect();
        assert_eq!(yh, vec!["fffffff6", "00000050"]); // -10, 80
    }

    #[test]
    fn model_tile_set_uses_stored_trits() {
        use trit_core::tritfmt::TritWriter;
        let dir = std::env::temp_dir().join("tritsim_vectors_model");
        std::fs::create_dir_all(&dir).unwrap();
        let mp = dir.join("m.trit");
        let mut w = TritWriter::create(&mp, "{}").unwrap();
        let trits: Vec<i8> = (0..2 * 64).map(|i| [(1i8), 0, -1][i % 3]).collect();
        w.write_trit("model.layers.0.self_attn.k_proj.weight", &[2, 64], &trits, 0.5).unwrap();
        w.finish().unwrap();
        model_tile_set(&dir, &mp, "model_k_proj_l0", 8).unwrap();
        let meta = std::fs::read_to_string(dir.join("model_k_proj_l0/meta.txt")).unwrap();
        assert_eq!(meta.trim(), "2 64"); // capped at available rows
    }

    #[test]
    fn generate_all_emits_standard_sets() {
        let dir = std::env::temp_dir().join("tritsim_vectors_all");
        let _ = std::fs::remove_dir_all(&dir);
        generate_all(&dir).unwrap();
        for set in ["exact_64x64", "padded_5x100", "extremes_4x128", "zeros_3x64", "wide_2x6912"] {
            assert!(dir.join(set).join("y.hex").exists(), "missing {set}");
        }
    }
}
