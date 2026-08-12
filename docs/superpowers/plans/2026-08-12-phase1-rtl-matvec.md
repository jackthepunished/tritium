# tritium Phase 1 (RTL ternary matvec core) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A SystemVerilog ternary matvec core (`trit_matvec`) that matches `tritsim`'s integer select-accumulate path bit-for-bit under Verilator, driven by golden vectors generated from the Rust golden model.

**Architecture:** `tritsim vectors` (new subcommand) emits self-describing vector sets: an int8 activation vector, packed 2-bit weight beats, and expected i32 accumulators computed by `trit_core::matvec::ternary_matvec`. The RTL streams LANES=64 trits per beat (128-bit words), decodes each trit to add/subtract/skip against a preloaded activation memory, reduces through a combinational adder tree, and accumulates per row. A Verilator C++ testbench loads each vector set, streams the beats, and asserts exact equality on every row output. Columns are pre-padded to a LANES multiple at generation time with zero trits.

**Tech Stack:** SystemVerilog (synthesizable subset), Verilator 5.020, C++17 testbench, Rust for vector generation, GNU make.

**Explicitly deferred (recorded scope decision):** fixed-point RMSNorm/activation RTL units. Bit-exact verification of those requires migrating tritsim's f32 norm/activation math to matching fixed-point first (Q-format design); scheduled with week-3 board bring-up. This plan's deliverable is the week-2 headline: matmul core bit-exact vs golden model.

## Global Constraints

- Repo root `/mnt/d/dev/tritium`; `export CARGO_TARGET_DIR=$HOME/.cache/tritium-target` before cargo commands.
- Every task ends with its verification run green and a commit. No emojis anywhere.
- Bit-exact means exact i32 equality on every row of every vector set — no tolerances.
- RTL datapath rule (from ARCHITECTURE): no multipliers on weights — trit decode is a mux to {+x, -x, 0} feeding adders.
- Trit encoding as in `.trit`: `0b00 = 0`, `0b01 = +1`, `0b10 = -1`; `0b11` must not silently corrupt — RTL raises a sticky `err` flag (golden vectors never contain it; the testbench asserts `err == 0`, and one negative test asserts `err == 1`).
- Beat format: 64 trits per 128-bit little-endian word; trit for lane `l` (column `beat*64 + l`) sits at bits `[2l+1:2l]`. Hex files store one beat per line as 32 hex chars, most-significant nibble first.
- Vector sets are generated artifacts: `rtl/vectors/` is git-ignored; `make test` regenerates them via cargo.
- Simulation-first: single-cycle 64-term adder tree and 64 parallel activation-memory reads are acceptable now; FPGA timing/banking is week-3 work (note kept in the RTL header comment).

## File Structure

```
crates/tritsim/src/vectors.rs   # golden vector set generator (library + tests)
crates/tritsim/src/main.rs      # add `vectors` subcommand
rtl/trit_matvec.sv              # the core
rtl/tb/tb_trit_matvec.cpp       # Verilator harness: load set, stream, compare
rtl/Makefile                    # verilate, build, generate vectors, run all sets
.gitignore                      # + rtl/vectors/ + rtl/obj_dir/
```

---

### Task 1: Golden vector generator (`tritsim vectors`)

**Files:**
- Create: `crates/tritsim/src/vectors.rs`
- Modify: `crates/tritsim/src/lib.rs` (add `pub mod vectors;`), `crates/tritsim/src/main.rs` (add subcommand)

**Interfaces:**
- Produces: `vectors::write_set(dir: &Path, name: &str, rows: usize, cols: usize, trits: &[i8], x: &[i8]) -> anyhow::Result<()>` writing `<dir>/<name>/{meta.txt, x.hex, w.hex, y.hex}`; `vectors::generate_all(out: &Path) -> anyhow::Result<()>` emitting the standard sets; expected `y` computed with `trit_core::matvec::ternary_matvec` (the same function the model runs).
- File formats: `meta.txt` = `"<rows> <cols_padded>\n"`; `x.hex` = one 2-hex-char signed byte per line (`cols_padded` lines, padding bytes are `00`); `w.hex` = one 32-hex-char beat per line, `rows * cols_padded/64` lines, row-major; `y.hex` = one 8-hex-char two's-complement i32 per line.
- Standard sets (deterministic xorshift seed per set): `exact_64x64` (boundary fit), `padded_5x100` (padding path), `extremes_4x128` (x in {-128,127}, trits in {-1,1} — accumulator headroom), `zeros_3x64` (all-zero trits), `wide_2x6912` (real model width).

- [ ] **Step 1: Write the failing test**

In `crates/tritsim/src/vectors.rs`:

```rust
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
    fn generate_all_emits_standard_sets() {
        let dir = std::env::temp_dir().join("tritsim_vectors_all");
        let _ = std::fs::remove_dir_all(&dir);
        generate_all(&dir).unwrap();
        for set in ["exact_64x64", "padded_5x100", "extremes_4x128", "zeros_3x64", "wide_2x6912"] {
            assert!(dir.join(set).join("y.hex").exists(), "missing {set}");
        }
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p tritsim vectors`
Expected: COMPILE ERROR (module missing).

- [ ] **Step 3: Implement**

```rust
use anyhow::Result;
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
```

`main.rs`: add to `Cmd`:

```rust
    /// Emit golden vector sets for the RTL testbench
    Vectors {
        #[arg(long, default_value = "rtl/vectors")]
        out: PathBuf,
    },
```

and to the match: `Cmd::Vectors { out } => { tritsim::vectors::generate_all(&out)?; println!("vectors written to {}", out.display()); }`

- [ ] **Step 4: Run tests, then generate**

Run: `cargo test -p tritsim vectors` — expected 2 passed.
Run: `cargo run -p tritsim --release -- vectors` — expected `rtl/vectors/` with 5 sets.

- [ ] **Step 5: Commit**

```bash
git add crates/tritsim .gitignore
git commit -m "feat(tritsim): golden vector generator for RTL verification"
```

---

### Task 2: `trit_matvec.sv` core

**Files:**
- Create: `rtl/trit_matvec.sv`

**Interfaces (consumed by Task 3's testbench):**
- Params: `LANES=64`, `MAX_COLS=8192`, `ACCW=32`.
- Ports: `clk`, `rst_n`; activation write port `x_we/x_addr/x_data` (signed 8-bit); `num_cols` (must be a LANES multiple, latched on `start`); `start` pulse resets row/beat state; stream `w_valid`/`w_data[127:0]`/`w_ready` (always 1 in v1); outputs `y_valid` (1-cycle pulse), `y_data` (signed 32), sticky `err` for `0b11` codes.
- Behavior: each accepted beat adds `sum_{l}(decode(trit_l) * x[beat*LANES+l])` — implemented as mux to `{+x, -x, 0}` — into the row accumulator; after `num_cols/LANES` beats it pulses `y_valid` with the row total and resets for the next row.

- [ ] **Step 1: Write the RTL**

```systemverilog
// trit_matvec: streaming ternary matrix-vector core.
// Weights arrive as 64x 2-bit trits per beat; activations are preloaded int8.
// No multipliers: each lane muxes {+x, -x, 0} into a combinational adder tree.
// Simulation-first: the single-cycle 64-term reduction and 64 parallel x_mem
// reads are fine under Verilator; FPGA timing/banking is addressed in the
// board bring-up phase.
module trit_matvec #(
    parameter int LANES    = 64,
    parameter int MAX_COLS = 8192,
    parameter int ACCW     = 32
) (
    input  logic                          clk,
    input  logic                          rst_n,
    // activation preload
    input  logic                          x_we,
    input  logic [$clog2(MAX_COLS)-1:0]   x_addr,
    input  logic signed [7:0]             x_data,
    // control
    input  logic [$clog2(MAX_COLS):0]     num_cols,  // multiple of LANES
    input  logic                          start,
    // weight stream
    input  logic                          w_valid,
    input  logic [2*LANES-1:0]            w_data,
    output logic                          w_ready,
    // row results
    output logic                          y_valid,
    output logic signed [ACCW-1:0]        y_data,
    output logic                          err
);

    logic signed [7:0] x_mem[MAX_COLS];

    logic [$clog2(MAX_COLS):0] cols_q;
    logic [$clog2(MAX_COLS/LANES):0] beat_q, beats_per_row;
    logic signed [ACCW-1:0] acc_q;

    assign w_ready = 1'b1;

    // per-beat combinational reduction
    logic signed [ACCW-1:0] beat_sum;
    logic beat_err;
    always_comb begin
        beat_sum = '0;
        beat_err = 1'b0;
        for (int l = 0; l < LANES; l++) begin
            logic [1:0] code;
            logic signed [7:0] xv;
            code = w_data[2*l+:2];
            /* verilator lint_off WIDTHTRUNC */
            xv = x_mem[cols_q == '0 ? '0 : ($clog2(MAX_COLS))'(32'(beat_q) * LANES + l)];
            /* verilator lint_on WIDTHTRUNC */
            unique case (code)
                2'b01:   beat_sum += ACCW'(xv);
                2'b10:   beat_sum -= ACCW'(xv);
                2'b00:   ;
                default: beat_err = 1'b1;
            endcase
        end
    end

    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            cols_q <= '0;
            beat_q <= '0;
            beats_per_row <= '0;
            acc_q <= '0;
            y_valid <= 1'b0;
            y_data <= '0;
            err <= 1'b0;
        end else begin
            y_valid <= 1'b0;
            if (x_we) x_mem[x_addr] <= x_data;
            if (start) begin
                cols_q <= num_cols;
                beats_per_row <= ($clog2(MAX_COLS/LANES)+1)'(32'(num_cols) / LANES);
                beat_q <= '0;
                acc_q <= '0;
                err <= 1'b0;
            end else if (w_valid) begin
                if (beat_err) err <= 1'b1;
                if (32'(beat_q) == 32'(beats_per_row) - 1) begin
                    y_valid <= 1'b1;
                    y_data <= acc_q + beat_sum;
                    acc_q <= '0;
                    beat_q <= '0;
                end else begin
                    acc_q <= acc_q + beat_sum;
                    beat_q <= beat_q + 1'b1;
                end
            end
        end
    end

endmodule
```

- [ ] **Step 2: Lint-compile it**

Run: `verilator --lint-only -Wall rtl/trit_matvec.sv`
Expected: clean (iterate on lint warnings until silent; do not waive with pragmas except where noted).

- [ ] **Step 3: Commit**

```bash
git add rtl/trit_matvec.sv
git commit -m "feat(rtl): streaming ternary matvec core (64-lane, no multipliers)"
```

---

### Task 3: Verilator testbench + Makefile

**Files:**
- Create: `rtl/tb/tb_trit_matvec.cpp`, `rtl/Makefile`
- Modify: `.gitignore` (add `rtl/obj_dir/`, `rtl/vectors/`)

**Interfaces:**
- `./obj_dir/Vtrit_matvec <set_dir>...` runs every named vector set, exits 0 only if every row of every set matches exactly and `err` stayed low; prints `SET <name>: <rows> rows OK`.
- `make -C rtl test` = generate vectors (cargo) + verilate + run all sets.

- [ ] **Step 1: Write the testbench**

```cpp
// tb_trit_matvec: drives golden vector sets through Vtrit_matvec and
// requires exact i32 equality on every row output.
#include "Vtrit_matvec.h"
#include "verilated.h"

#include <cstdint>
#include <cstdio>
#include <fstream>
#include <string>
#include <vector>

namespace {

std::vector<std::string> lines(const std::string& path) {
    std::ifstream f(path);
    if (!f) { std::fprintf(stderr, "cannot open %s\n", path.c_str()); std::exit(2); }
    std::vector<std::string> out;
    for (std::string l; std::getline(f, l);)
        if (!l.empty()) out.push_back(l);
    return out;
}

struct Dut {
    Vtrit_matvec top;
    void tick() {
        top.clk = 0; top.eval();
        top.clk = 1; top.eval();
    }
    void reset() {
        top.rst_n = 0; tick(); tick();
        top.rst_n = 1; tick();
    }
};

bool run_set(Dut& d, const std::string& dir) {
    const auto meta = lines(dir + "/meta.txt");
    unsigned rows, cols;
    std::sscanf(meta.at(0).c_str(), "%u %u", &rows, &cols);
    const auto xh = lines(dir + "/x.hex");
    const auto wh = lines(dir + "/w.hex");
    const auto yh = lines(dir + "/y.hex");
    const unsigned beats = cols / 64;

    // preload activations
    for (unsigned c = 0; c < cols; c++) {
        d.top.x_we = 1;
        d.top.x_addr = c;
        d.top.x_data = static_cast<int8_t>(std::stoul(xh.at(c), nullptr, 16));
        d.tick();
    }
    d.top.x_we = 0;
    d.top.num_cols = cols;
    d.top.start = 1; d.tick(); d.top.start = 0;

    std::vector<int32_t> got;
    for (unsigned b = 0; b < rows * beats; b++) {
        const std::string& beat = wh.at(b); // 32 hex chars, MSB first
        for (int w = 0; w < 4; w++)         // w_data is 4x 32-bit words, LSW = chars 24..31
            d.top.w_data[w] = std::stoul(beat.substr(24 - 8 * w, 8), nullptr, 16);
        d.top.w_valid = 1;
        d.tick();
        if (d.top.y_valid) got.push_back(static_cast<int32_t>(d.top.y_data));
    }
    d.top.w_valid = 0;
    d.tick();
    if (d.top.y_valid) got.push_back(static_cast<int32_t>(d.top.y_data));

    bool ok = got.size() == rows && d.top.err == 0;
    for (unsigned r = 0; ok && r < rows; r++) {
        const auto want = static_cast<int32_t>(std::stoul(yh.at(r), nullptr, 16));
        if (got[r] != want) {
            std::fprintf(stderr, "%s row %u: got %d want %d\n", dir.c_str(), r, got[r], want);
            ok = false;
        }
    }
    if (got.size() != rows)
        std::fprintf(stderr, "%s: got %zu rows, want %u (err=%d)\n", dir.c_str(), got.size(), rows, (int)d.top.err);
    if (ok) std::printf("SET %s: %u rows OK\n", dir.c_str(), rows);
    return ok;
}

// invalid 0b11 code must raise the sticky err flag
bool run_invalid_code_check(Dut& d) {
    d.reset();
    d.top.x_we = 1; d.top.x_addr = 0; d.top.x_data = 1; d.tick(); d.top.x_we = 0;
    d.top.num_cols = 64;
    d.top.start = 1; d.tick(); d.top.start = 0;
    d.top.w_data[0] = 0x3; // code 0b11 in lane 0
    d.top.w_data[1] = d.top.w_data[2] = d.top.w_data[3] = 0;
    d.top.w_valid = 1; d.tick(); d.top.w_valid = 0;
    if (!d.top.err) { std::fprintf(stderr, "invalid-code check: err not raised\n"); return false; }
    std::printf("SET <invalid-code>: err raised OK\n");
    return true;
}

} // namespace

int main(int argc, char** argv) {
    Verilated::commandArgs(argc, argv);
    Dut d;
    bool ok = true;
    for (int i = 1; i < argc; i++) {
        d.reset();
        ok &= run_set(d, argv[i]);
    }
    ok &= run_invalid_code_check(d);
    d.top.final();
    return ok ? 0 : 1;
}
```

- [ ] **Step 2: Write the Makefile**

```makefile
# rtl/Makefile: verilate, build, and run the golden-vector regression.
VERILATOR ?= verilator
CARGO_TARGET_DIR ?= $(HOME)/.cache/tritium-target
SETS := exact_64x64 padded_5x100 extremes_4x128 zeros_3x64 wide_2x6912
SET_DIRS := $(addprefix vectors/,$(SETS))

.PHONY: test lint vectors clean

test: obj_dir/Vtrit_matvec vectors
	./obj_dir/Vtrit_matvec $(SET_DIRS)

lint:
	$(VERILATOR) --lint-only -Wall trit_matvec.sv

obj_dir/Vtrit_matvec: trit_matvec.sv tb/tb_trit_matvec.cpp
	$(VERILATOR) --cc --exe --build -Wall --top-module trit_matvec \
	    trit_matvec.sv tb/tb_trit_matvec.cpp -o Vtrit_matvec

vectors:
	cd .. && CARGO_TARGET_DIR=$(CARGO_TARGET_DIR) cargo run -p tritsim --release -- vectors --out rtl/vectors

clean:
	rm -rf obj_dir vectors
```

- [ ] **Step 3: Run the regression**

Run: `make -C rtl test`
Expected: `SET rtl/vectors/<name>: N rows OK` for all five sets plus the invalid-code check, exit 0. Any mismatch is a real bug in RTL or harness — debug with superpowers:systematic-debugging (the golden side is already validated against the real model).

- [ ] **Step 4: Commit**

```bash
git add rtl .gitignore
git commit -m "feat(rtl): verilator testbench and make regression, bit-exact vs golden vectors"
```

---

### Task 4: Real-model tile vectors (golden vectors from actual checkpoint weights)

**Files:**
- Modify: `crates/tritsim/src/vectors.rs` (add `model_tile_set`), `crates/tritsim/src/main.rs` (optional `--model` flag), `rtl/Makefile` (include set when present)

**Interfaces:**
- `tritsim vectors --out rtl/vectors --model models/bitnet-2b4t.trit` additionally writes `model_k_proj_l0` (first 8 rows of layer-0 k_proj against a deterministic random int8 activation) using the exact trits the model runs.

- [ ] **Step 1: Write the failing test** (uses a writer-built tiny .trit, not the real model)

```rust
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
```

- [ ] **Step 2: Implement**

```rust
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
```

(Add `use anyhow::Context;` to imports. In `main.rs`, `Vectors` gains `#[arg(long)] model: Option<PathBuf>`; when set, call `model_tile_set(&out, &model, "model_k_proj_l0", 8)?`.)

In `rtl/Makefile`, extend:

```makefile
MODEL ?= ../models/bitnet-2b4t.trit
vectors:
	cd .. && CARGO_TARGET_DIR=$(CARGO_TARGET_DIR) cargo run -p tritsim --release -- vectors --out rtl/vectors \
	    $$( [ -f $(MODEL) ] && echo --model $(MODEL) )
test: obj_dir/Vtrit_matvec vectors
	./obj_dir/Vtrit_matvec $(SET_DIRS) $$( [ -d vectors/model_k_proj_l0 ] && echo vectors/model_k_proj_l0 )
```

- [ ] **Step 3: Verify**

Run: `cargo test -p tritsim vectors` (3 passed) then `make -C rtl test` — expected all sets including `model_k_proj_l0` OK.

- [ ] **Step 4: Commit**

```bash
git add crates/tritsim rtl/Makefile
git commit -m "feat(tritsim): real-checkpoint tile vectors for RTL regression"
```

---

### Task 5: Docs + integration

**Files:**
- Modify: `README.md` (status + component table row for rtl/), `docs/02-ROADMAP.md` (tick week-2 items that are done, note the fixed-point-norm deferral)

- [ ] **Step 1: Update docs.** README status gains: RTL matvec core passes the bit-exact golden-vector regression (`make -C rtl test`); component table `tritcore` row points at `rtl/`. Roadmap week 2: mark `.trit` freeze (already done in Phase 0) and matmul-core items `[x]`; annotate the RMSNorm/SiLU fixed-point item as deferred to week 3 with the Q-format reasoning.

- [ ] **Step 2: Full verification.** `cargo test --workspace` green, `make -C rtl lint` clean, `make -C rtl test` all sets OK.

- [ ] **Step 3: Commit, push, PR.** Branch `phase1-rtl-matvec`, push, open PR against main; then superpowers:finishing-a-development-branch.

## Self-Review Notes

- Week-2 spec coverage: vector-driven bit-exact matmul core (Tasks 1-4) and testbench harness (Task 3) = the roadmap's headline artifact. `.trit` freeze already landed in Phase 0. Fixed-point norm units explicitly deferred with reasoning (preamble + Task 5 docs note).
- Type consistency: beat layout (trit l at bits 2l, 32-hex MSB-first lines) identical in `write_set`, RTL port comment, and TB `substr(24 - 8*w, 8)` parsing. `LANES=64` constant shared by generator and RTL param. `y.hex` two's-complement u32 formatting matches TB `stoul`-then-cast.
- The TB collects a trailing `y_valid` after the last beat because `y_valid` is registered (pulses the cycle after the final beat is accepted); the extra post-stream `tick()` covers it.
