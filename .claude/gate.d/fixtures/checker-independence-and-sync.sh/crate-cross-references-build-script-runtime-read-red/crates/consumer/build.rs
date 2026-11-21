// 样本：构建脚本运行时读提供包的着色器源码与清单，并报 rerun-if-changed：着色器照别的包源码判红，清单算数据。
fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=../provider/src/k.wgsl");
    println!("cargo::rerun-if-changed=../provider/Cargo.toml");
    let kernel = std::fs::read_to_string("../provider/src/k.wgsl").unwrap();
    let manifest = std::fs::read_to_string("../provider/Cargo.toml").unwrap();
    assert!(!kernel.is_empty() && !manifest.is_empty());
}
