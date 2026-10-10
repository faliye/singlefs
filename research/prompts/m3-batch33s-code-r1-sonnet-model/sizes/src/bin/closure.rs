//! 判法闭包收了哪几份文件（第 33 批 S5 的「旧判定不复用」：被判的几份是不是都在闭包里；不入库）
use singlefs_checker_tier::crash_identity::judging_code_of_the_judges;
use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;
fn main() {
    let root = std::env::args().nth(1).expect("仓根");
    let toolchain = Layer0ToolchainIdentity {
        rustc_version_lines: "x".into(),
        cargo_version: "x".into(),
        target_triple: "x".into(),
    };
    let code = judging_code_of_the_judges(std::path::Path::new(&root), &toolchain).expect("闭包");
    for file in &code.files {
        let raw = if code.raw_files.contains(file) { " raw" } else { "" };
        println!("FILE {file}{raw}");
    }
    for file in &code.stopped_at {
        println!("STOPPED {file}");
    }
}
