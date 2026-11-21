// 样本：构建脚本只报提供包的着色器的 rerun-if-changed。
fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=../provider/src/k.wgsl");
}
