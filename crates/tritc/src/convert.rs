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
                let mut err = 0f64;
                let mut norm = 0f64;
                for (w, t) in data.iter().zip(&trits) {
                    let d = (w - scale * *t as f32) as f64;
                    err += d * d;
                    norm += (*w as f64) * (*w as f64);
                }
                let recon = (err / norm.max(1e-12)).sqrt() as f32;
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
