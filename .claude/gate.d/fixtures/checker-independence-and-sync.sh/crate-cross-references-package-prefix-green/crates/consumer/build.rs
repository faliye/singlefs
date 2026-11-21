// 样本：构建脚本为算判定的单元键逐份读提供包的文件，并报 rerun-if-changed；前缀行放行着色器与源码，清单本来就算数据。
fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=../provider/src/k.wgsl");
    println!("cargo::rerun-if-changed=../provider/src/shared.rs");
    println!("cargo::rerun-if-changed=../provider/Cargo.toml");
}
