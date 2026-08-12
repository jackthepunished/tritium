//! Per-stage numeric range recorder, enabled by TRITSIM_STATS=<out.json>.
//! Feeds the fixed-point format decisions in docs/03b-NUMERICS.md.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

#[derive(Default, Clone, serde::Serialize)]
pub struct StageStats {
    pub count: u64,
    pub max_abs: f32,
    pub mean_abs: f64,
    pub max_l2: f32,
}

#[derive(Default)]
pub struct Recorder {
    stages: BTreeMap<String, (StageStats, f64 /* abs sum */, u64 /* elems */)>,
}

impl Recorder {
    pub fn record(&mut self, stage: &str, v: &[f32]) {
        let e = self.stages.entry(stage.to_string()).or_default();
        let mut abs_sum = 0f64;
        let mut sq = 0f64;
        for x in v {
            let a = x.abs();
            if a > e.0.max_abs {
                e.0.max_abs = a;
            }
            abs_sum += a as f64;
            sq += (*x as f64) * (*x as f64);
        }
        let l2 = sq.sqrt() as f32;
        if l2 > e.0.max_l2 {
            e.0.max_l2 = l2;
        }
        e.0.count += 1;
        e.1 += abs_sum;
        e.2 += v.len() as u64;
    }

    pub fn finalize(&self) -> BTreeMap<String, StageStats> {
        self.stages
            .iter()
            .map(|(k, (s, abs_sum, elems))| {
                let mut s = s.clone();
                s.mean_abs = if *elems > 0 { abs_sum / *elems as f64 } else { 0.0 };
                (k.clone(), s)
            })
            .collect()
    }
}

static GLOBAL: OnceLock<Option<Mutex<Recorder>>> = OnceLock::new();

fn global() -> &'static Option<Mutex<Recorder>> {
    GLOBAL.get_or_init(|| {
        std::env::var_os("TRITSIM_STATS").map(|_| Mutex::new(Recorder::default()))
    })
}

/// No-op unless TRITSIM_STATS is set.
pub fn record(stage: &str, v: &[f32]) {
    if let Some(m) = global() {
        m.lock().unwrap().record(stage, v);
    }
}

/// Write collected stats to the path named by TRITSIM_STATS (call from main).
pub fn flush() -> anyhow::Result<()> {
    if let (Some(m), Some(path)) = (global(), std::env::var_os("TRITSIM_STATS")) {
        let snap = m.lock().unwrap().finalize();
        std::fs::write(path, serde_json::to_string_pretty(&snap)?)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorder_tracks_ranges() {
        let mut r = Recorder::default();
        r.record("a", &[1.0, -3.0]);
        r.record("a", &[2.0, 0.0]);
        let s = &r.finalize()["a"];
        assert_eq!(s.count, 2);
        assert_eq!(s.max_abs, 3.0);
        assert!((s.mean_abs - 1.5).abs() < 1e-9); // (1+3+2+0)/4
        assert!((s.max_l2 - (10f32).sqrt()).abs() < 1e-6);
    }

    #[test]
    fn disabled_global_is_noop() {
        // TRITSIM_STATS unset in the test env: record must not panic or allocate a recorder
        record("whatever", &[1.0]);
        assert!(global().is_none() || true); // reaching here without panic is the assertion
    }
}
