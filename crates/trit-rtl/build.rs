//! Builds and links the Verilated `tritcore` static library.
//!
//! This crate exists only to be linked deliberately. Depending on it requires
//! Verilator, a C++ toolchain and GNU make on PATH, which is why `tritd` gates
//! it behind an off-by-default cargo feature -- a normal build of the runtime
//! must not need an RTL simulator.

use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let rtl = manifest.join("../../rtl").canonicalize().expect("rtl directory");

    let status = std::process::Command::new("make")
        .arg("-C")
        .arg(&rtl)
        .arg("lib")
        .status()
        .expect("run `make -C rtl lib` (verilator and a C++ toolchain are required)");
    assert!(status.success(), "make -C rtl lib failed");

    println!("cargo:rustc-link-search=native={}", rtl.join("obj_dir_lib").display());
    println!("cargo:rustc-link-lib=static=tritcore_rtl");
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rerun-if-changed={}", rtl.join("trit_matvec.sv").display());
    println!("cargo:rerun-if-changed={}", rtl.join("shim/trit_rtl_shim.cpp").display());
    println!("cargo:rerun-if-changed={}", rtl.join("shim/trit_rtl_shim.h").display());
}
