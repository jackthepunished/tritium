//! Every kernel against the reference, on the same bytes, demanding **exact**
//! equality.
//!
//! Exactness is not optimism. The kernels accumulate in `i32`, integer addition
//! is associative and exact, and the widest tensor in the target model bounds
//! each accumulator at `6912 * 128 = 884_736` -- three orders of magnitude
//! inside `i32`. So lane order, thread count and the RTL's beat-serial order all
//! have to produce identical bits. A tolerance here would be hiding a bug.

use trit_core::matvec::ternary_matvec_planes_vec;
use trit_core::planes::{beats_per_row, pack_planes, TritPlanes, LANES};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn trit(&mut self) -> i8 {
        [-1i8, 0, 1][(self.next() % 3) as usize]
    }
    /// Full i8 range, so -128 occurs.
    fn i8(&mut self) -> i8 {
        (self.next() & 0xff) as u8 as i8
    }
}

fn pad(x: &[i8], cols: usize) -> Vec<i8> {
    let mut v = vec![0i8; beats_per_row(cols) * LANES];
    v[..x.len()].copy_from_slice(x);
    v
}

/// One case: a weight matrix, an activation vector, and a label.
struct Case {
    name: String,
    rows: usize,
    cols: usize,
    trits: Vec<i8>,
    x: Vec<i8>,
}

fn corpus() -> Vec<Case> {
    let mut rng = Rng(0xA11CE);
    let mut cases = Vec::new();

    // Shapes either side of the beat boundary, plus every real BitNet 2B4T
    // width: 2560 (hidden), 640 (kv heads), 6912 (intermediate).
    for &(rows, cols) in &[
        (1usize, 1usize),
        (1, 63),
        (1, 64),
        (1, 65),
        (2, 100),
        (3, 127),
        (5, 129),
        (7, 640),
        (16, 61),
        (64, 64),
        (2, 2560),
        (2, 6912),
        (640, 2560),
        (513, 64),
    ] {
        let trits: Vec<i8> = (0..rows * cols).map(|_| rng.trit()).collect();
        let x: Vec<i8> = (0..cols).map(|_| rng.i8()).collect();
        cases.push(Case {
            name: format!("random_{rows}x{cols}"),
            rows,
            cols,
            trits,
            x,
        });
    }

    // The worst case for an i8-domain negation: every weight -1, every
    // activation -128. A kernel that computes -x in i8 returns -128 per column
    // instead of +128 and fails here by a factor of -1.
    cases.push(Case {
        name: "all_neg_weights_x_minus128".into(),
        rows: 4,
        cols: 128,
        trits: vec![-1; 4 * 128],
        x: vec![-128; 128],
    });
    cases.push(Case {
        name: "all_pos_weights_x_minus128".into(),
        rows: 4,
        cols: 128,
        trits: vec![1; 4 * 128],
        x: vec![-128; 128],
    });
    // The golden vector set the RTL testbench uses: alternating extremes, no
    // zeros anywhere, worst-case accumulator magnitude.
    {
        let mut r = Rng(0xFFF);
        let trits: Vec<i8> = (0..4 * 128)
            .map(|_| if r.next() & 1 == 0 { 1 } else { -1 })
            .collect();
        let x: Vec<i8> = (0..128)
            .map(|_| if r.next() & 1 == 0 { -128 } else { 127 })
            .collect();
        cases.push(Case {
            name: "extremes_4x128".into(),
            rows: 4,
            cols: 128,
            trits,
            x,
        });
    }
    // Degenerate planes.
    cases.push(Case {
        name: "all_zero_weights".into(),
        rows: 3,
        cols: 64,
        trits: vec![0; 3 * 64],
        x: (0..64).map(|i| i as i8).collect(),
    });
    cases.push(Case {
        name: "widest_row_all_nonzero".into(),
        rows: 1,
        cols: 6912,
        trits: vec![-1; 6912],
        x: vec![-128; 6912],
    });
    // Sparse: one bit set in the last beat, exercising the tail mask.
    {
        let cols = 6912;
        let mut trits = vec![0i8; cols];
        trits[cols - 1] = -1;
        cases.push(Case {
            name: "single_weight_in_final_beat".into(),
            rows: 1,
            cols,
            trits,
            x: vec![-128; cols],
        });
    }
    cases
}

/// Run the whole corpus through one kernel and compare against the reference.
fn check_kernel(name: &str) {
    // force_kernel must ERROR, never fall back: a silent fallback would make a
    // CI runner without AVX-512 report green while testing the scalar path.
    let forced = match trit_cpu::force_kernel(name) {
        Ok(k) => k,
        Err(e) => panic!("cannot test {name}: {e}"),
    };
    assert_eq!(forced, name, "force_kernel resolved to a different kernel");

    for c in corpus() {
        let beats = pack_planes(&c.trits, c.rows, c.cols).unwrap();
        let planes = TritPlanes::new(&beats, c.rows, c.cols, 1.0).unwrap();
        let x = pad(&c.x, c.cols);

        let expect = ternary_matvec_planes_vec(&planes, &x);
        let got = trit_cpu::ternary_matvec_planes_vec(&planes, &x);
        assert_eq!(got, expect, "kernel {name} disagrees on case {}", c.name);

        // And against a naive dense dot product, so a shared bug in both plane
        // implementations cannot pass.
        let naive: Vec<i32> = (0..c.rows)
            .map(|r| {
                (0..c.cols)
                    .map(|j| c.trits[r * c.cols + j] as i32 * c.x[j] as i32)
                    .sum()
            })
            .collect();
        assert_eq!(
            got, naive,
            "kernel {name} disagrees with the naive dot on {}",
            c.name
        );
    }
}

/// The dispatcher's own choice -- what a real run uses.
#[test]
fn default_kernel_matches_the_reference() {
    let name = trit_cpu::kernel_name();
    println!("dispatched kernel: {name}");
    println!("available on this CPU: {:?}", trit_cpu::available_kernels());
    for c in corpus() {
        let beats = pack_planes(&c.trits, c.rows, c.cols).unwrap();
        let planes = TritPlanes::new(&beats, c.rows, c.cols, 1.0).unwrap();
        let x = pad(&c.x, c.cols);
        assert_eq!(
            trit_cpu::ternary_matvec_planes_vec(&planes, &x),
            ternary_matvec_planes_vec(&planes, &x),
            "default kernel {name} disagrees on {}",
            c.name
        );
    }
}

/// The kernel under test is chosen by `TRIT_CPU_KERNEL`, because the choice is
/// cached process-wide and one test binary can only exercise one. CI runs this
/// binary once per kernel in `available_kernels()`.
#[test]
fn forced_kernel_matches_the_reference() {
    let Ok(name) = std::env::var("TRIT_CPU_KERNEL") else {
        eprintln!("TRIT_CPU_KERNEL unset; covered by default_kernel_matches_the_reference");
        return;
    };
    check_kernel(&name);
}

#[test]
fn unsupported_kernel_errors_rather_than_falling_back() {
    let err = trit_cpu::force_kernel("definitely-not-a-kernel").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("not available"), "{msg}");
    // The message must name what IS available, or a CI failure is unactionable.
    assert!(msg.contains("scalar"), "{msg}");
}

#[test]
fn scalar_is_always_available() {
    assert!(trit_cpu::available_kernels().contains(&"scalar"));
}

/// Threading splits rows, never a reduction, so the result must not depend on
/// thread count at all.
#[test]
fn threading_does_not_change_a_single_bit() {
    use trit_core::backend::MatvecBackend;
    let mut rng = Rng(0xB0B);
    let (rows, cols) = (1024, 2560);
    let trits: Vec<i8> = (0..rows * cols).map(|_| rng.trit()).collect();
    let x: Vec<i8> = (0..cols).map(|_| rng.i8()).collect();
    let beats = pack_planes(&trits, rows, cols).unwrap();
    let planes = TritPlanes::new(&beats, rows, cols, 1.0).unwrap();
    let xp = pad(&x, cols);

    let expect = ternary_matvec_planes_vec(&planes, &xp);
    for threads in [1usize, 2, 3, 8, 16] {
        let be = trit_cpu::CpuBackend::new(threads);
        let mut y = vec![0i32; rows];
        be.matvec(&planes, &xp, &mut y);
        assert_eq!(y, expect, "{threads} threads changed the result");
    }
}
