# tritium Phase 3 (hardware-in-the-loop under Verilator) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The week-3b purchase gate: the real BitNet 2B4T model decodes end-to-end with every ternary matvec executed by the Verilated `trit_matvec` RTL, producing logits and greedy text bit-identical to the CPU backend.

**Architecture:** Verilator compiles `trit_matvec.sv` into a static library plus a small C shim (`trit_rtl_new/free/matvec`) that drives the core exactly like the Phase-1 testbench (load x, start, stream 16-byte beats, collect row accumulators). A cargo feature `rtl` in tritsim builds and links that library via `build.rs` (shelling to `make -C rtl lib`) and adds a runtime `--backend rtl` switch: `BitLinear` routes its matvec through a process-wide RTL handle instead of `ternary_matvec`. Because the accumulator math is exact integer arithmetic, CPU and RTL paths must agree bit-for-bit — the end-to-end gate is exact equality, not tolerance. Beat layout equals the `.trit` packed byte layout (2 bits/trit, 4/byte, little-endian; 16 bytes/beat), and all model widths are multiples of 64, so the RTL consumes the exact bytes tritc wrote.

**Tech Stack:** Verilator 5.020 (C++ harness), Rust FFI (feature-gated), GNU make, ar.

## Global Constraints

- Repo root `/mnt/d/dev/tritium`; `CARGO_TARGET_DIR=$HOME/.cache/tritium-target`. No emojis. Task = green + commit.
- Default builds must NOT require Verilator: everything RTL-linked lives behind `--features rtl` (build.rs skips the make call when the feature is off).
- The gate standard is exact equality (identical logits bytes, identical greedy text) between `--backend cpu` and `--backend rtl` on the real checkpoint — integer datapath, no excuses.
- The Rust wrapper pads non-multiple-of-64 widths (zero trits / zero x), so the API stays general even though this model never needs it.
- Wrapper concurrency: one global core behind a `Mutex` (model forward is sequential; parallel tests just serialize).

## File Structure

```
rtl/shim/trit_rtl_shim.cpp    # extern "C" driver around Vtrit_matvec
rtl/shim/trit_rtl_shim.h      # C API
rtl/Makefile                  # + `lib` target -> obj_dir_lib/libtritcore_rtl.a
crates/tritsim/build.rs       # feature rtl: make -C rtl lib, link static + stdc++
crates/tritsim/src/rtl.rs     # unsafe FFI + safe rtl_matvec(trits,rows,cols,xq)
crates/tritsim/src/backend.rs # Backend::{Cpu,Rtl}; matvec dispatch used by BitLinear
crates/tritsim/src/model.rs   # BitLinear matvec calls go through backend
crates/tritsim/src/main.rs    # --backend flag on run/compare
```

---

### Task 1: C shim + `make lib`

`trit_rtl_shim.h`:

```c
#pragma once
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
void* trit_rtl_new(void);
void trit_rtl_free(void* h);
/* beats: rows * (cols/64) * 16 bytes, .trit packed layout; cols % 64 == 0.
   Returns 0 on success, 1 if the sticky err flag rose (invalid trit code). */
int trit_rtl_matvec(void* h, const uint8_t* beats, const int8_t* x,
                    uint32_t rows, uint32_t cols, int32_t* y_out);
#ifdef __cplusplus
}
#endif
```

`trit_rtl_shim.cpp` drives the DUT exactly like `tb_trit_matvec.cpp` (reset once in `trit_rtl_new`; per call: write x, latch config with `start`, stream beats one per tick assembling `w_data` words from 16 little-endian bytes, collect `y_valid` pulses including the trailing one, return err flag state). Keep a `reset()` before each matvec call to clear the sticky err and counters.

Makefile `lib` target: `verilator --cc trit_matvec.sv -Mdir obj_dir_lib --build -CFLAGS -fPIC`, compile the shim with the Verilator include dirs (`-Iobj_dir_lib -I$(VERILATOR_ROOT)/include -I$(VERILATOR_ROOT)/include/vltstd`), then `ar rcs obj_dir_lib/libtritcore_rtl.a` over the shim object, `Vtrit_matvec__ALL.a` members (extract with `ar x` or pass `.o` files), and `verilated.o`-family objects. Verify with a tiny C smoke test compiled and run by the task (hand case: W=[[1,-1,0...],[0,1,1...]] padded to 64 cols, x=[10,20,30,0...], expect [-10, 50]).

- [ ] Implement shim + header + Makefile target; C smoke test binary runs and matches.
- [ ] Commit: `feat(rtl): C shim and static library target for hardware-in-the-loop`.

### Task 2: Rust FFI + backend dispatch (feature `rtl`)

- `build.rs`: when `CARGO_FEATURE_RTL` is set, run `make -C ../../rtl lib` (path via `CARGO_MANIFEST_DIR`), emit `cargo:rustc-link-search=native=<rtl>/obj_dir_lib`, `cargo:rustc-link-lib=static=tritcore_rtl`, `cargo:rustc-link-lib=stdc++`; rerun-if-changed on the RTL and shim.
- `rtl.rs` (behind `#[cfg(feature = "rtl")]`): RAII handle over the C API in a `OnceLock<Mutex<...>>`; `pub fn rtl_matvec(trits: &[i8], rows: usize, cols: usize, xq: &[i8]) -> Vec<i32>` — pads cols to 64, packs trits with `trit_core::pack::pack_trits` row-by-row (padded), calls the shim, panics on err flag (golden data never triggers it).
- `backend.rs`: `pub enum Backend { Cpu, #[cfg(feature = "rtl")] Rtl }` + a process-wide selected backend (`set_backend`, default Cpu) + `pub fn matvec(...)` dispatching to `ternary_matvec` or `rtl_matvec`. `BitLinear::apply/apply_prequantized` call `backend::matvec`.
- `main.rs`: `--backend cpu|rtl` on `Run` and `Compare` (rejects `rtl` unless compiled with the feature).
- Tests (feature-gated, `cargo test -p tritsim --features rtl`): the Phase-1 hand case via `rtl_matvec`; a property test of 20 random (rows, cols) shapes — including non-multiples of 64 (padding path) and one 2x6912 — asserting `rtl_matvec == ternary_matvec` exactly.

- [ ] Failing tests first, then implement; `cargo test -p tritsim --features rtl` green AND plain `cargo test --workspace` (no feature, no Verilator needed) green.
- [ ] Commit: `feat(tritsim): RTL matvec backend behind cargo feature, exact-match property tests`.

### Task 3: The purchase gate — real model through the RTL

- [ ] `tritsim compare --backend rtl` (folded norm on, both backends) on `logits.json`: metrics identical to the CPU backend run, and add `--dump-argmax` style proof via existing output; then greedy: `run --backend rtl --steps 8` on the France prompt — text identical to CPU backend. Long-running (tens of minutes in Verilator): run in background, wall-clock logged.
- [ ] Record in `docs/02-ROADMAP.md` week 3b: gate PASSED with the measured sim wall-clock per token (context for why silicon is next).
- [ ] Commit: `feat: week-3b gate passed — real model end-to-end through Verilated RTL`.

### Task 4: Docs + PR

- [ ] README status; board memo note that the purchase gate is satisfied.
- [ ] Full verification: `cargo test --workspace`, `--features rtl` suite, `make -C rtl lint test synth`.
- [ ] Branch `phase3-verilator-loop`, push, PR, finishing-a-development-branch.

## Self-Review Notes

- The bit-exact standard is justified: both paths are integer select-accumulate; any deviation is a harness bug, not noise. f32 scale application happens outside the backend boundary (identical code both paths).
- Beat layout compatibility with `.trit` bytes is asserted by the Task 2 property tests (pack_trits is the shared packer) — no silent format fork.
- Default build stays Verilator-free (feature gate), so contributors without EDA tools build and test everything else.
