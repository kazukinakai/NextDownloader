fn main() {
    // UDLファイルからバインディングを生成
    uniffi::generate_scaffolding("src/nextdownloader.udl").unwrap();
}