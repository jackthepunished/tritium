use std::path::PathBuf;

fn main() {
    if std::env::var_os("CARGO_FEATURE_RTL").is_none() {
        return; // default builds need no Verilator
    }
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let rtl = manifest.join("../../rtl").canonicalize().expect("rtl dir");
    let status = std::process::Command::new("make")
        .arg("-C")
        .arg(&rtl)
        .arg("lib")
        .status()
        .expect("run make -C rtl lib (verilator required for --features rtl)");
    assert!(status.success(), "make -C rtl lib failed");
    println!("cargo:rustc-link-search=native={}", rtl.join("obj_dir_lib").display());
    println!("cargo:rustc-link-lib=static=tritcore_rtl");
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rerun-if-changed={}", rtl.join("trit_matvec.sv").display());
    println!("cargo:rerun-if-changed={}", rtl.join("shim/trit_rtl_shim.cpp").display());
    println!("cargo:rerun-if-changed={}", rtl.join("shim/trit_rtl_shim.h").display());
}
