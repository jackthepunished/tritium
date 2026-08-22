//! Compile a real C program against `include/tritium.h`, link it against the
//! real library, and run it.
//!
//! This is what keeps a hand-written header honest: if a declaration here drifts
//! from the Rust signature, the C compiler rejects it, and if the semantics
//! drift, the program produces the wrong answer.

use std::path::PathBuf;
use std::process::Command;

fn out_dir() -> PathBuf {
    let d = std::env::temp_dir().join("tritd_c_abi");
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Locate the staticlib cargo just built. It sits beside the test binary.
fn find_staticlib() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    // .../target/<profile>/deps/c_abi-<hash>
    let deps = exe.parent()?;
    let profile = deps.parent()?;
    for dir in [profile, deps] {
        let p = dir.join("libtritd.a");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// The header must be valid C on its own, with no other includes and warnings
/// as errors -- an embedder including it should never see a diagnostic.
#[test]
fn header_compiles_standalone_as_strict_c() {
    let dir = out_dir();
    let src = dir.join("hdr_only.c");
    std::fs::write(
        &src,
        r#"#include "tritium.h"
int main(void) { return 0; }
"#,
    )
    .unwrap();

    let out = Command::new("cc")
        .args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-c"])
        .arg("-I")
        .arg(manifest().join("include"))
        .arg(&src)
        .arg("-o")
        .arg(dir.join("hdr_only.o"))
        .output();

    let Ok(out) = out else {
        eprintln!("no C compiler available; skipping");
        return;
    };
    assert!(
        out.status.success(),
        "header did not compile as strict C:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Load a model through the C API and generate tokens, exactly as an embedder
/// would.
#[test]
fn c_program_loads_a_model_and_generates() {
    let Some(lib) = find_staticlib() else {
        eprintln!("libtritd.a not built (needs crate-type staticlib); skipping");
        return;
    };
    // The real checkpoint: a tokenizer and a model have to agree on vocabulary,
    // and the tiny test fixtures have no tokenizer of their own. Skipped when
    // the checkpoint is not present (it is gitignored and 1.8 GB), so this test
    // is a no-op on a fresh clone rather than a failure.
    let model = manifest().join("../../models/bitnet-2b4t.trit");
    let tokenizer = manifest().join("../../models/bitnet-2b4t/tokenizer.json");
    if !model.exists() || !tokenizer.exists() {
        eprintln!(
            "models/bitnet-2b4t.trit or its tokenizer is absent; skipping the end-to-end C run"
        );
        return;
    }

    let dir = out_dir();
    let src = dir.join("consumer.c");
    std::fs::write(
        &src,
        r#"#include "tritium.h"
#include <stdio.h>
#include <string.h>

int main(int argc, char **argv) {
    if (argc < 3) { fprintf(stderr, "usage: consumer <model> <tokenizer>\n"); return 2; }

    if (!(trit_backend_mask() & TRIT_BACKEND_CPU)) {
        fprintf(stderr, "no cpu backend\n"); return 1;
    }
    if (trit_version() == NULL || trit_kernel_name() == NULL) {
        fprintf(stderr, "null introspection string\n"); return 1;
    }

    TritModel *m = trit_model_load(argv[1], argv[2], "cpu", 1);
    if (!m) { fprintf(stderr, "load failed: %s\n", trit_last_error()); return 1; }

    if (trit_model_vocab_size(m) == 0) { fprintf(stderr, "vocab 0\n"); return 1; }
    if (trit_model_weight_bytes(m) == 0) { fprintf(stderr, "weight bytes 0\n"); return 1; }

    TritSamplerParams p = {0};
    p.temperature = 0.0f;          /* greedy: reproducible */
    TritSession *s = trit_session_new(m, &p);
    if (!s) { fprintf(stderr, "session failed: %s\n", trit_last_error()); return 1; }

    if (trit_session_prefill(s, "hello", 0) != TRIT_OK) {
        fprintf(stderr, "prefill failed: %s\n", trit_last_error()); return 1;
    }

    char buf[256];
    uint32_t id = 0;
    size_t len = 0, produced = 0;
    int rc;
    while ((rc = trit_session_next(s, &id, buf, sizeof buf, &len)) == 1) {
        if (len >= sizeof buf) { fprintf(stderr, "length out of range\n"); return 1; }
        if (buf[len] != '\0')  { fprintf(stderr, "not NUL terminated\n"); return 1; }
        produced++;
        if (produced >= 8) break;
    }
    if (rc < 0) { fprintf(stderr, "next failed: %s\n", trit_last_error()); return 1; }
    if (produced == 0) { fprintf(stderr, "no tokens produced\n"); return 1; }

    /* reset must succeed and let the session run again */
    if (trit_session_reset(s) != TRIT_OK) { fprintf(stderr, "reset failed\n"); return 1; }
    if (trit_session_prefill(s, "hello", 0) != TRIT_OK) { fprintf(stderr, "re-prefill\n"); return 1; }

    trit_session_free(s);
    trit_model_free(m);
    printf("OK %zu tokens\n", produced);
    return 0;
}
"#,
    )
    .unwrap();

    let bin = dir.join("consumer");
    let build = Command::new("cc")
        .args(["-std=c11", "-Wall", "-Wextra", "-Werror"])
        .arg("-I")
        .arg(manifest().join("include"))
        .arg(&src)
        .arg(&lib)
        // -lstdc++ is required: the tokenizer dependency (esaxx-rs) is C++.
        .args(["-lpthread", "-ldl", "-lm", "-lstdc++", "-o"])
        .arg(&bin)
        .output();

    let Ok(build) = build else {
        eprintln!("no C compiler available; skipping");
        return;
    };
    assert!(
        build.status.success(),
        "C consumer did not build:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );

    let run = Command::new(&bin).arg(&model).arg(&tokenizer).output().unwrap();
    assert!(
        run.status.success(),
        "C consumer failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(String::from_utf8_lossy(&run.stdout).contains("OK"));
}
