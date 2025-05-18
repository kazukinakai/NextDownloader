use std::env;
use std::path::PathBuf;

fn main() {
    // C FFIのヘッダーファイル生成
    if cfg!(feature = "c-ffi") {
        let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let header_path = out_dir.join("nextdownloader.h");

        let config = cbindgen::Config::from_file("cbindgen.toml")
            .expect("cbindgen.tomlの読み込みに失敗しました");

        cbindgen::Builder::new()
            .with_crate(crate_dir)
            .with_config(config)
            .generate()
            .expect("C/C++ヘッダーの生成に失敗しました")
            .write_to_file(header_path);

        println!("cargo:rerun-if-changed=src/c_ffi.rs");
        println!("cargo:rerun-if-changed=cbindgen.toml");
    }

    // UniFFIのバインディング生成
    if cfg!(feature = "uniffi") {
        uniffi::generate_scaffolding("src/nextdownloader.udl")
            .expect("UniFFIスカフォールディングの生成に失敗しました");

        println!("cargo:rerun-if-changed=src/uniffi.rs");
        println!("cargo:rerun-if-changed=src/nextdownloader.udl");
    }
}