// 样本：构建脚本把提供包的 src/x.rs 原样拷进 OUT_DIR（样本里的假 cargo 照 scenario.json 写出同样的内容），另写一份自己生成的 generated.rs。
fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    let out = std::env::var("OUT_DIR").unwrap();
    std::fs::copy("../provider/src/x.rs", format!("{out}/copied.rs")).unwrap();
    std::fs::write(format!("{out}/generated.rs"), "pub const GENERATED: u32 = 1;\n").unwrap();
}
