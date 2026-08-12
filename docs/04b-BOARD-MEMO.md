# Board-selection memo

Decision requested: which FPGA board (if any) to buy for the first hardware
bring-up. Written after Phase 2; numbers below are measured, not guessed.
Following the ternoise doctrine: simulation-first, board chosen from synthesis
numbers.

## Measured inputs

- **Synthesis (Yosys 0.33, generic, 64 lanes, MAX_COLS=512):** 33.7k cells
  total, of which 4.1k DFF are the flattened activation memory (BRAM on any
  real device) — so the streaming datapath (trit decode + 64-term adder tree +
  control) is roughly 29.5k gate-level cells, order tens-of-k LUTs after
  packing, scaling ~linearly with lane count. **Zero multiplier cells,
  asserted by `make -C rtl synth` on every run.** No DSP slices needed for the
  matmul engine; norm folding (Phase 2) removed the per-element rsqrt too.
- **Bandwidth roofline (ARCHITECTURE section 2):** decode tok/s is
  memory-bound at `bandwidth / packed_model_size`. Packed 2B4T is ~600 MB;
  a 700M-class model ~175 MB.
- **Numerics (03b-NUMERICS.md):** q/k/v/ctx/norm-out stages fit int16;
  residual and the relu2 activation stage stay f32/wide in v1 — both live
  off the ternary streaming path, so they don't gate the board choice.

## Options

| Option | Cost | Usable BW | 700M model | 2B model | Notes |
|---|---|---|---|---|---|
| A. Budget Artix-7 (e.g. Alinx AX7035-class, DDR3) | ~$150-250 | ~1.6 GB/s | ~9 tok/s | ~2.6 tok/s | Proves the architecture on silicon; not conversational at 2B. 64 lanes fit trivially; fabric ~33-53k LUTs is the constraint to watch against the full layer sequencer. |
| B. Kria KV260 (Zynq US+, DDR4) | ~$250-400 | ~10-12 GB/s | ~60 tok/s | ~17-20 tok/s | Conversational 700M, usable 2B. PS runs tritd directly (Linux + AXI), which deletes the USB/host-transport workstream entirely. 117k LUTs, 144 BRAM36. |
| C. No purchase yet (extend Verilator, ternoise M5 style) | $0 | n/a | sim-speed | sim-speed | Verilate the full layer pipeline as a library tritd calls; demo exists end-to-end on desktop first. Zero risk, no tok/s/W numbers — and the f.inc demo eventually needs real silicon. |

## Recommendation

**B (Kria KV260), bought after option-C's layer sequencer runs under Verilator.**
Rationale: the roofline says DDR3 boards can never make the 2B model
conversational, and the Phase-0-validated stack deserves hardware that can hit
the week-5 benchmark targets (>= 8 tok/s on a ~700M model at <= $250 BOM is
achievable on a used KV260). The PS-side Linux host removes the largest
non-differentiating engineering risk (host transport). Sequence C -> B keeps
the ternoise discipline: the board arrives with the RTL already proven against
the golden model end-to-end.

Purchase decision and timing are Bahadir's; nothing here commits money.
