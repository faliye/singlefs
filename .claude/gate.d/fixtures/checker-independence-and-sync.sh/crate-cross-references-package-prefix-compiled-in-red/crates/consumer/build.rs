// 样本：构建脚本报提供包的着色器与 tests/c/mod.rs 的 rerun-if-changed；后者同时被测试目标当模块编进来。
fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=../provider/src/k.wgsl");
    println!("cargo::rerun-if-changed=../provider/tests/c/mod.rs");
}
