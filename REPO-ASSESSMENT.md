# Tritium repository assessment — 22 September 2026

Assessment of `a5-arm-throughput` at `6ea2939223d0b4abe98df23a0c8af319929a6474`, including the roadmap, implementation, committed measurements, live GitHub status, and fresh local verification. This is an assessment, not an implementation change.

## What you are building

Tritium is an inference stack for offline language capability on inexpensive, power-constrained devices. The longer-term product thesis is a ternary hardware module, eventually an ASIC, supported by its own model format, runtime, numerical reference, and measurement infrastructure. The CPU runtime provides something useful today while reducing the risk of the eventual hardware implementation. The positioning documents identify robotics, drones, handheld tools and other embedded devices as the intended customers, with the public development record also supporting a Founders Inc application.

The central engineering decision is shared weight layout. `.trit` v1 stores 64 ternary weights in a 16-byte beat containing positive and negative bit masks. CPU SIMD kernels and the RTL consume the same beat layout. Model weights are memory-mapped and addressed through validated spans; they are not expanded into a separate runtime representation. The 1.58-bit description refers to the information content of three weight states; the stored ternary projections use two bits per weight. Embeddings and the output head are dense bf16/f32.

Your numerical discipline is the other major asset. There is a deliberately straightforward model implementation (`tritsim`), a production model implementation (`trit-core` plus a backend), and a SystemVerilog matvec implementation. Integer ternary accumulations must agree exactly. Floating-point reductions have separate tolerances and model-level checks. This lets you change the CPU implementation or hardware backend while retaining a common correctness contract.

The claim to prove next is **system-level efficiency on the intended hardware**, particularly joules per token. A multiplier-free ternary core does not make the entire transformer multiplier-free: attention, normalization, rescaling and the dense head still involve floating-point arithmetic. Similarly, operation-level energy estimates do not establish whole-device savings. The $50 / single-digit-watt positioning is an intended product outcome, not a demonstrated result in this repository.

## Implementation map

| Component | What is implemented | Practical boundary |
|---|---|---|
| `tritc` | Pinned HF checkpoint workflow; deterministic, globally sorted conversion from safetensors; per-tensor ternary quantization; bf16 retention for dense matrices; verification and v0 migration | Validated principally on BitNet b1.58 2B4T; this is not a general converter for arbitrary transformer architectures |
| `trit-core` | Container validation, mmap-backed weights, transformer, GQA attention, RoPE, sampler, tokenizer interfaces, reusable session/scratch, KV cache | Single-token forward calls; f32 KV storage; 2048-position cap |
| `trit-cpu` | Scalar, AVX2, AVX-512BW, AVX-512 VNNI, NEON and NEON dot-product ternary kernels; dense SIMD kernels; persistent row worker pool | Global pool and kernel selection introduce process-wide constraints; scaling and ARM measurement remain open |
| `tritd` | CLI `run`, `info`, `bench`, `serve`; SSE completions; model/health/metrics endpoints; C static/shared libraries and header | A small synchronous, single-session HTTP server, not a concurrent serving platform |
| `tritsim` | Independent transformer/oracle, reference logit comparison, numerical modes, golden-vector generation | Deliberately slow and memory-heavy; major real-model checks need ignored local assets |
| `trit-rtl` / `rtl` | Verilator C++ bridge, Rust backend, 64-lane streaming ternary matvec, vector tests and generic synthesis | A matvec accelerator under simulation; no placed full transformer accelerator, board transport/DMA, or timing closure |
| `benches` | Fixed prompt suites, runtime orchestration, baseline adapters, committed CSVs and reporting | Short-context CPU comparisons exist; baseline energy measurement and ARM end-to-end results do not |

The production forward path is embedding lookup → normalized/quantized q/k/v projections → RoPE and KV attention → output projection/residual → gated MLP/residual → final normalization and dense vocabulary projection. `MatvecBackend` handles both ternary and dense projections. Sessions own KV state and scratch; models share weights. The numerical ladder is `Reference`, `Folded`, and `IntMlp`; the latter applies to squared-ReLU models with the required FFN sub-norms. Prefill currently loops over ordinary token forwards, including output-head computation, rather than amortizing a weight read across a token batch.

## Where the roadmap actually stands

| Milestone | Assessment |
|---|---|
| Original simulation/numerics/RTL phases | Implemented. The repository contains the oracle, exact integer MLP path, hardware-in-the-loop backend, and their regression checks. |
| Pivot to a hardware-agnostic runtime | Implemented. Shared v1 planes, SIMD, mmap, daemon, C API and comparative harness exist. |
| A1 — run the baselines | Complete for the documented x86 short-context comparison. bitnet.cpp uses the same checkpoint; llama.cpp uses a different model/quality point. |
| A2 — parallel scaling | **Partially complete**, as the detailed roadmap says. The pool is merged, but throughput still peaks at four threads and falls beyond that in the committed measurements. |
| A3 — dense head storage | Complete. Source bf16 values remain bf16 in the file and widen in registers. No additional weight quantization was required. |
| A4 — energy/token | Open. No counter is exposed here. There is also measurement work to do: baseline energy capture and a consistent measurement window. |
| A5 — ARM throughput | Started, not complete. This branch adds dense-head microbenchmarks and invokes kernelbench in CI. The gate requires real-model decode on ARM with the kernel identified. |
| A6 — KV precision/context | Open. 314,572,800 bytes of f32 KV capacity at the 2048 cap. Measure both RSS and longer-context behavior; allocated zero-filled capacity is not the same as resident memory at a short prompt. |
| A7 — batched prefill | Open. No batched matrix path or continuous batching exists. |
| B1–B4 — silicon | Open. The simulation milestone unblocks a purchase decision; it does not supply a board, achieved bandwidth, full-model device measurement, or timing closure. |

The active branch is exactly one commit ahead of `origin/main` (`18a2b86`). That commit changes only `.github/workflows/ci.yml` and `crates/trit-cpu/examples/kernelbench.rs`: 70 insertions and three deletions. GitHub reported no open PR and no CI run for this branch at inspection time. The latest main workflow succeeded: [main CI run](https://github.com/jackthepunished/tritium/actions/runs/32898699661). The workflow triggers on main pushes and pull requests, so merely pushing this branch does not run its new ARM timing step.

## What the existing measurements establish

The frozen post-A2/A3 reference on the Ryzen 9 8945HX is 27.90 decode tok/s at four threads, 239 ms reported TTFT, about 1.19 GiB peak RSS, and 1,177,681,920 weight bytes per token. Those bytes split into 521,011,200 ternary bytes and 656,670,720 dense-head bytes. The dense head is still 56% of the weight stream. This explains why ternary-kernel speedups alone cannot deliver the full system improvement.

The committed same-checkpoint comparison reports Tritium ahead of bitnet.cpp at one, two and four threads, but behind at eight and sixteen. At eight threads it is 25.93 versus 32.49 tok/s. The shell harness invokes each thread configuration in a separate process, so the kernelbench process-state defect described below does not by itself invalidate that published table.

These are historical, short-context, host-specific measurements. They establish useful CPU inference and a scaling limitation. They do not establish ARM throughput, wall-plug efficiency, FPGA throughput, or broad task quality. The uncentered logit cosine and a handful of token positions are numerical regression evidence rather than an application-level evaluation suite.

## Findings that matter before the next milestone

**1. A5's thread-scaling benchmark silently measures the wrong execution mode.** `kernelbench.rs:120` tries 1/2/4/8/16/32 threads in one process. `pool.rs:240` initializes one `OnceLock<Pool>` and returns `None` for any later requested width that differs. `CpuBackend::matvec` and the dense path then run serially. The first parallel request fixes the pool at two threads; the later rows labelled four or more are serial, including the new four-thread dense benchmark. A standalone probe using the actual pool source reproduced the mismatch; running the example also exhibited it. Use one process per thread configuration, or explicitly support multiple pool widths, before collecting A5 results. The same constraint affects library consumers that load models with different thread counts in one process. The pool also allocates a parts vector per dispatch, despite the architecture document's allocation-free wording.

**2. Baseline NEON has an uncovered overflow counterexample.** In `aarch64.rs:94`, an all-zero beat executes `continue` before the periodic i16-to-i32 flush at line 106. If beats 16, 32 and 48 are zero, the accumulators can span 45 active beats without a flush. With 3,072 columns, +1 weights in all other beats and activations of -128, the exact dot is -368,640; emulating the kernel's wrapping i16 arithmetic gives +155,648. The all-nonzero and isolated-sparse tests do not exercise this combination. This was established by source analysis and exact integer emulation on x86, **not execution of NEON hardware**. Add the counterexample to the ARM corpus and make flushing independent of skipped weight beats. The dot-product kernel uses i32 accumulators and does not have this particular defect.

**3. The headline HF “top-1 100%” result accepts near ties.** `tritsim/src/compare.rs:68` counts the reference's second-ranked token as a match if its top two logits are within 0.25. The fresh six-position comparison reproduced cosine 0.9991 and reported 100%, but position zero selects token 279 while HF selects 11. Strict argmax agreement is therefore 5/6 (83.3%) for this dump; near-tie-aware agreement is 6/6. This is a reporting distinction, not evidence of a newly introduced regression. It should be explicit wherever the result is quoted. Oracle-versus-production G8 is a separate check with actual identical top-1 IDs.

**4. Measurement definitions need alignment before energy/latency claims expand.** The benchmark returns a median over all prompt/run decode rates, but TTFT is the minimum across them (`bench.rs:274`), and `prefill_tok_per_s` is hardcoded to zero. The methodology document specifies a fixed 256-token TTFT prompt and thermal/60-second warm-up procedures that the current default harness does not implement. The RAPL interval includes prefill/session overhead while its denominator counts generated tokens. Baseline rows always write energy as absent. Package RAPL also measures a different boundary from wall-plug/12V-rail power. Define and implement the same energy window and source for both runtimes before interpreting A4 as a hardware-only blocker.

**5. Hardware planning and public positioning contain stale assumptions.** The board memo still budgets ~600 MB for the 2B model, while current weight traffic is 1.178 GB/token. Even retaining its assumed 10–12 GB/s bandwidth, the weight-only ceiling would be about 8.5–10.2 tok/s rather than 17–20, before host/attention/transfer costs. Recalculate before choosing hardware; the older memo is not a current throughput forecast. The positioning document still uses the original six-week timeline, and the ignored application source predates A2/A3. The architecture's final open-question section also still refers to the f32 head. None should override the newer roadmap, gates and code.

**6. Fresh-clone and CI coverage need a little maintenance.** `rtl/obj_dir_lib` contains tracked generated C++, object files and static archives. Rebuild from RTL source to avoid relying on their provenance or timestamps. Both shell entry points are tracked without executable bits, so use `bash scripts/fetch_model.sh` and `bash benches/run.sh` on Linux until modes are corrected. Real-model, tokenizer and C integration checks require local assets; some return early rather than failing when prerequisites are missing. A green fixture-only workflow does not mean those real-model paths ran. The branch's new benchmark output is printed to CI logs rather than committed as a reproducible ARM results artifact. There is no pinned Rust toolchain file, although the current local toolchain builds successfully.

## Recommended continuation

First, make A5 trustworthy: correct the benchmark's process/pool interaction, cover the NEON sparse-flush case, and arrange a CI trigger for the branch. Then collect named-kernel ARM microbenchmarks and full-model decode at separate 1/2/4-thread settings. CI ARM throughput is useful evidence, but the final edge story also needs the actual intended device's memory, power and thermal constraints.

In parallel with hardware access, finish A4's comparable measurement window and baseline capture, and correct the near-tie/TTFT labels. These are small credibility improvements relative to changing the engine. Keep the observed A2 scaling gap visible; a merged pool is not a closed scaling problem.

For silicon, refresh the complete-system bandwidth budget, including the dense head and host boundary, before resuming the board decision. Treat timing closure and activation-memory banking as early bring-up risks. The existing 64-read activation interface will need deliberate banking/replication or a revised lane schedule; generic synthesis does not decide its placement.

A6 is the next natural memory feature once those measurements are credible. A7 remains lower priority for this single-session edge product. Application-level task evaluation on the intended workload should accompany hardware claims; a numerically faithful engine can still be running a model that does not solve the customer's task.

## WSL workspace and verification

The independent working clone is `/home/bahadir/dev/tritium`, on WSL's Linux filesystem. `origin` remains `https://github.com/jackthepunished/tritium.git`, with `a5-arm-throughput` tracking its matching remote branch. The original `/mnt/d/dev/tritium` checkout remains untouched. The seven ignored reference/working files and complete model directory were copied locally; all 24 copied files (7,889,459,756 bytes) match their source SHA-256 digests. The raw manifest and validation logs are in `/home/bahadir/.cache/tritium-analysis-20260922`.

Tools available here: Rust/Cargo 1.95.0, Verilator 5.020, Yosys 0.33, C/C++ compilers and the aarch64 Rust cross-check target. Runtime binaries and C libraries were built on the Linux filesystem. No ARM hardware or exposed RAPL zones are available in this WSL environment.

Fresh verification results are recorded below. RTL rebuilding uses an isolated source snapshot under the validation directory so generated tracked files do not dirty the working clone. This assessment and `AGENTS.md` contain the local review and working constraints; no implementation, roadmap, gate, or benchmark CSV was edited.


| Fresh check | Result |
|---|---|
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --exclude trit-rtl --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --exclude trit-rtl --release --locked` | 137 passed, zero failed, three explicitly ignored |
| Forced ternary differential tests | Scalar, AVX2, AVX-512BW and AVX-512 VNNI all passed in separate processes |
| Forced dense differential tests | Scalar, AVX2 and AVX-512F all passed in separate processes |
| Optional bitserial differential check | Passed with the feature enabled and the kernel forced |
| ARM cross-check | `cargo check -p trit-cpu --target aarch64-unknown-linux-gnu --locked` passed; compilation, not ARM execution |
| Real checkpoint G8 | Passed: eight positions in each of Reference/Folded/IntMlp; displayed cosine 1.000000 and identical top-1 IDs throughout |
| HF comparison G2 | Reproduced 0.9991 mean cosine and 100% near-tie-aware match; strict argmax 5/6 as explained above |
| CPU greedy G3 | Frozen 16-token text reproduced exactly in all three numerical modes, four threads, AVX-512 VNNI |
| C ABI and tokenizer | Four tests passed with the copied model/tokenizer present, including actual C consumer generation |
| RTL lint | Passed |
| RTL golden vectors | Five synthetic sets plus the real layer-zero projection tile passed; overlap error and all-zero-beat checks passed |
| RTL-feature tests after rebuilding from source | 38 passed, zero failed, one explicitly ignored real-checkpoint test in the isolated snapshot |
| Generic synthesis | Completed at 33,659 cells; configured post-synthesis zero `$mul` / `$macc` assertions passed; this is not device placement/timing evidence |
| Copied assets | All 24 files verified byte-identical by SHA-256 |

A fresh benchmark ran after the analysis's heavy validation processes completed, from the WSL-native clone:

```sh
cd /home/bahadir/dev/tritium
target/release/tritd bench --model models/bitnet-2b4t.trit \
  --suite benches/prompts/short.jsonl --tokens 32 --threads 4 --runs 3 \
  --json /home/bahadir/.cache/tritium-analysis-20260922/wsl-bench.json
```

It measured **27.02 tok/s**, reported **239.3 ms TTFT**, and **1,220 MiB peak RSS**. Effective weight throughput was **31.82 GB/s** against a **57.93 GB/s** streaming probe in the same invocation (**54.93%**). Energy remained absent. This is consistent with the historical operating range; it is not a paired filesystem speedup experiment or a new baseline comparison. The TTFT aggregation caveat above still applies. The benchmark's `peak_rss_mb` value is actually MiB because the implementation divides KiB by 1024.

Not repeated here: full-model generation through the RTL backend, execution on real ARM hardware, fresh HF logit-dump generation, the external llama.cpp/bitnet.cpp comparison, board measurements, or application-level quality evaluation. HF comparison uses the copied reference dump. Kernelbench was used diagnostically while other verification was active; its timings are not a new publishable performance baseline.

For the next shell session, use `cd /home/bahadir/dev/tritium`. Commands in this assessment were directed explicitly to this clone. Cloning does not change the Codex app's saved project directory; select the WSL clone as the project when starting future tasks.


## User-confirmed constraint — 22 September 2026

Bahadir currently has no access to a real FPGA board or other target hardware. The immediate plan must therefore use the existing development machine: fix and validate software, improve measurement tooling, cross-compile ARM code, and exercise the RTL under simulation. Physical ARM throughput, FPGA bring-up, and device energy measurements remain blocked on hardware access. Their preparation can proceed, but acquiring hardware is not a prerequisite for continuing useful work. Revisit availability when Bahadir reports a change.


## Follow-up: first fixes implemented — 22 September 2026

The NEON sparse-flush overflow and kernelbench thread-count defects identified
above have now been fixed in the WSL working tree, with regressions. The actual
compiled ARM kernel was tested red and green under QEMU; this improves the
initial assessment's integer-emulation evidence without claiming physical ARM
validation. Kernelbench now uses one explicit thread count per process and CI
runs its CLI checks. The global pool's behavior for library callers requesting
multiple widths remains unchanged. Fresh checks and the local thread-scaling
baseline are recorded in [the validation report](benches/results/WSL-20260922.md).
The earlier assessment describes the repository before these fixes.


## Follow-up: A2 closed — 22 September 2026

The assessment above lists A2 as partially complete, with decode peaking at
four threads and declining beyond it. That decline is now diagnosed and fixed.
It was neither the memory system nor the serial fraction: a fork-once static
partitioning of the same work holds 42-43 GB/s from four threads to
thirty-two, while per-matvec dispatch collapses to 16.9, and the serial scalar
remainder is a flat 3.3-3.6 ms/token. The cause was the pool's spin window
expiring inside decode's own inter-matvec gaps, so most of the 210 dispatches
per token paid a futex wake.

Replacing the iteration count with a 200 us deadline moves decode from
26.5/24.3/20.3 to 33.4/34.8/33.3 tok/s at 4/8/16 threads in interleaved pairs,
and the peak moves to eight threads. It spends 14-43% more CPU per token above
four threads and slightly less at or below four; energy remains unmeasured.

Two defects the diagnosis exposed are recorded and unfixed: dispatch still
costs 218 us at sixteen slots with workers hot, and the `k`/`v` projections
fall below the slot threshold and never parallelise. Neither would raise the
peak. Details, method and limitations in
[the A2 report](benches/results/WSL-20260922-A2.md).

The assessment's point 4 still stands unaddressed: TTFT is still a minimum
rather than a median, and `prefill_tok_per_s` is still hardcoded to zero.
