//! Backend selection.
//!
//! `cpu` is always present. `rtl` exists only in a build that opted into the
//! Verilator dependency, and asking for it otherwise is a clear error rather
//! than a silent fallback -- running on the CPU while believing you are running
//! on the RTL would invalidate exactly the comparison the RTL path exists for.

use std::sync::Arc;

use anyhow::Result;
use trit_core::backend::MatvecBackend;

/// Backends compiled into this binary.
pub fn compiled_backends() -> &'static [&'static str] {
    #[cfg(feature = "rtl")]
    {
        &["cpu", "rtl"]
    }
    #[cfg(not(feature = "rtl"))]
    {
        &["cpu"]
    }
}

pub fn make_backend(name: &str, threads: usize) -> Result<Arc<dyn MatvecBackend>> {
    match name {
        "cpu" => Ok(Arc::new(trit_cpu::CpuBackend::new(threads))),
        #[cfg(feature = "rtl")]
        "rtl" => {
            if threads > 1 {
                eprintln!(
                    "note: the RTL backend serializes on one Verilated core; --threads {threads} \
                     affects only the dense f32 path"
                );
            }
            Ok(Arc::new(trit_rtl::RtlBackend::new()))
        }
        #[cfg(not(feature = "rtl"))]
        "rtl" => anyhow::bail!(
            "this build has no RTL backend; rebuild with --features rtl (requires verilator)"
        ),
        other => anyhow::bail!(
            "unknown backend {other:?} (available in this build: {})",
            compiled_backends().join(", ")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_is_always_available() {
        assert!(compiled_backends().contains(&"cpu"));
        assert!(make_backend("cpu", 1).is_ok());
    }

    #[test]
    fn unknown_backend_names_what_is_available() {
        let msg = make_backend("gpu", 1).unwrap_err().to_string();
        assert!(msg.contains("unknown backend"), "{msg}");
        assert!(msg.contains("cpu"), "{msg}");
    }

    #[cfg(not(feature = "rtl"))]
    #[test]
    fn rtl_without_the_feature_says_how_to_get_it() {
        let msg = make_backend("rtl", 1).unwrap_err().to_string();
        assert!(msg.contains("--features rtl"), "{msg}");
        assert!(msg.contains("verilator"), "{msg}");
    }

    #[cfg(feature = "rtl")]
    #[test]
    fn rtl_is_available_when_compiled_in() {
        assert!(compiled_backends().contains(&"rtl"));
        assert_eq!(make_backend("rtl", 1).unwrap().name(), "rtl/verilator");
    }
}
