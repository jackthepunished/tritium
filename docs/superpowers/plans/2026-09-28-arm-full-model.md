# ARM full-model measurement implementation plan

**Goal:** validate and measure BitNet 2B4T on native ARM, comparing pre-shuffle
Tritium, merged Tritium and pinned bitnet.cpp I2_S.

**Architecture:** a path-filtered CI workflow builds all binaries and converts
pinned public weights before timing. A Python harness writes every raw result,
checks output parity, alternates runtime order, and profiles decode separately.

**Constraints:** no emulation performance claims; no concurrent builds while
timing; exact model/toolchain/source identities; honest TTFT-minimum and
short-prompt vs empty-context labels; no energy estimates. Preserve the normal
CI suite and require the real checkpoint to exist before the ignored oracle test.

- [ ] Add pinned model acquisition, hash verification and native build workflow.
- [ ] Add measurement harness and tests for parsers, ordering, errors and parity.
- [ ] Validate locally, then run a draft PR on the native ARM runner.
- [ ] Require oracle cosine >0.999999 and matching top-1 at 24 positions;
      verify greedy text before/after at each measured thread count.
- [ ] Record paired full-model measurements, phase profiles and empty-dispatch
      diagnostics at 1/2/4 threads; compare bitnet.cpp on the same runner.
- [ ] Publish results, qualifications and next bottleneck in the roadmap/report;
      finish CI and review without merging.
