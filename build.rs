//! Kopiert passendes `memory.x` ins Build-Verzeichnis.

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let v7 = env::var("CARGO_FEATURE_SD_V7").is_ok();
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let memory_name = if v7 {
        "memory-s140-v7.x"
    } else {
        "memory-s140-v6.x"
    };
    let memory_path = format!("{manifest}/{memory_name}");
    let memory_x = std::fs::read(&memory_path).unwrap_or_else(|e| panic!("{memory_path}: {e}"));
    let layout = if v7 { "v7" } else { "v6" };

    println!("cargo:warning=CPB memory layout: {layout} ({memory_name})");
    println!("cargo:rustc-cfg=cpb_memory_{layout}");
    println!("cargo:rustc-check-cfg=cfg(cpb_memory_v6)");
    println!("cargo:rustc-check-cfg=cfg(cpb_memory_v7)");

    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(&memory_x)
        .unwrap();

    println!("cargo:rerun-if-changed=memory-s140-v6.x");
    println!("cargo:rerun-if-changed=memory-s140-v7.x");
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
}
