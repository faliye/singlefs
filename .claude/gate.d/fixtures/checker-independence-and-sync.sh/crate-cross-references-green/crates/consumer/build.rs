// 样本里消费包的构建脚本：往 OUT_DIR 写 generated.rs（样本里的假 cargo 照 scenario.json 写好）。
fn main() {
    println!("cargo::rerun-if-changed=build.rs");
}
