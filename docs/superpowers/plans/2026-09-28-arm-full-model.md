# ARM full-model measurement implementation plan

**Goal:** validate and measure BitNet 2B4T on native ARM, comparing pre-shuffle
Tritium, merged Tritium and pinned bitnet.cpp I2_S.

**Architecture:** a manually dispatched CI workflow builds all binaries and converts
pinned public weights before timing. A Python harness writes every raw result,
checks output parity, alternates runtime order, and profiles decode separately.

**Constraints:** no emulation performance claims; no concurrent builds while
timing; exact model/toolchain/source identities; honest TTFT-minimum and
short-prompt vs empty-context labels; no energy estimates. Preserve the normal
CI suite and require the real checkpoint to exist before the ignored oracle test.

- [x] Add pinned model acquisition, hash verification and native build workflow.
- [x] Add measurement harness and tests for parsers, ordering, errors and parity.
- [x] Validate locally, then run a draft PR on the native ARM runner.
- [x] Require oracle cosine >0.999999 and matching top-1 at 24 positions;
      verify greedy text before/after at each measured thread count.
- [x] Record paired full-model measurements, phase profiles and empty-dispatch
      diagnostics at 1/2/4 threads; attempt bitnet.cpp on the same runner.
      Its pinned ARM I2_S path fails the arithmetic audit, so those raw rates
      are diagnostic only and excluded from competitive claims. A validated
      I2_S revision or ARM TL1 comparator remains follow-up work.
- [x] Publish results, qualifications and next bottleneck in the roadmap/report.
      Keep the PR unmerged; final CI and review status is tracked on the PR.

The PR's bootstrap runs used a path-filtered pull-request trigger. After native
validation, retain the expensive checkpoint run as an explicit workflow dispatch
and run the seven harness regressions on every PR. Documentation-only pushes do
not need another full-model experiment. Independent review found that
`--pool-probe` exits before model loading; model profiles and empty dispatch
diagnostics now use separate processes, with a regression requiring phase rows.
The confirmation profiles put ternary work at 81.7% of four-thread decode and
the still-serial K/V calls at 9.71 ms/token (13.5% of total). The next runtime
experiment is the existing ARM work-per-slot knob, not a change made here.
