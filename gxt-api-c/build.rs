extern crate cbindgen;

use std::env;

pub fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let target_dir = std::path::Path::new(&env::var("OUT_DIR").unwrap()).join("../../..");

    let c_bindings = target_dir.join("gxt.h");

    cbindgen::Builder::new()
        .with_cpp_compat(true)
        .with_language(cbindgen::Language::C)
        .with_crate(crate_dir)
        .with_item_prefix("gxt")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(c_bindings);
}
