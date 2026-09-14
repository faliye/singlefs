//! 把第一遍的模型 `../src/main.rs` 原样搬进 OUT_DIR：只去掉 `//!` 行（include! 不收内部文档注释），把入口 `fn main() {` 改名。
//! 搬完回读，确认恰好改了一处入口、其余行一字未动（行数 = 原行数 − `//!` 行数）。

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_directory = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cargo 给 CARGO_MANIFEST_DIR"));
    let source_path = manifest_directory.join("../src/main.rs");
    println!("cargo:rerun-if-changed={}", source_path.display());
    let source = fs::read_to_string(&source_path).expect("读得到第一遍的 ../src/main.rs");
    let mut entry_points_renamed = 0usize;
    let mut inner_doc_lines = 0usize;
    let mut kept_lines = Vec::new();
    for line in source.lines() {
        if line.starts_with("//!") {
            inner_doc_lines += 1;
            continue;
        }
        if line == "fn main() {" {
            entry_points_renamed += 1;
            kept_lines.push("fn first_pass_main() {".to_string());
        } else {
            kept_lines.push(line.to_string());
        }
    }
    assert_eq!(entry_points_renamed, 1, "../src/main.rs 里要恰好一行 `fn main() {{`");
    let output_path = PathBuf::from(env::var("OUT_DIR").expect("cargo 给 OUT_DIR")).join("model.rs");
    fs::write(&output_path, kept_lines.join("\n")).expect("写得进 OUT_DIR");
    let written_back = fs::read_to_string(&output_path).expect("回读 OUT_DIR/model.rs");
    assert_eq!(written_back.lines().count() + inner_doc_lines, source.lines().count(), "搬运之后行数对不上");
    assert!(written_back.contains("fn first_pass_main() {"), "入口没改成 first_pass_main");
}
