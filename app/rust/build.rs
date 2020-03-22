extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let build = dst.join("build");

    let mut cfg = cc::Build::new();

    cfg.out_dir(&build)
        .cpp(true)
        .flag_if_supported("-w")
        .flag_if_supported("-Wno-unused-parameter")
        .include("libnode/include/node")
        .file("build/util/util.cpp")
        .file("build/v8_jni/wrapper.cpp")
        .file("build/api.cpp")
        .compile("api");
}
