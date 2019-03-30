extern crate cc;

use std::path::{PathBuf, Path};
use std::env;

fn main() {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!(
        "cargo:rustc-link-search=native={}",
        Path::new(&dir).join("libnode/bin/x86").display()
    );
    println!("cargo:rustc-link-search=dylib={}", "node");

    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let build = dst.join("build");

    let mut cfg = cc::Build::new();

    cfg.out_dir(&build)
        .cpp(true)
        .flag_if_supported("-Wno-unused-parameter")
        .include("libnode/include/node")
        .file("api.cpp")
        .compile("api");
}
