// 样本：同包 #[path]、读提供包的 Cargo.toml（数据）、读构建脚本 OUT_DIR 里生成的 .rs，另读提供包的着色器源码（放行表登了这一处）。
#[path = "c/mod.rs"]
mod c;

const PROVIDER_MANIFEST: &str = include_str!("../../provider/Cargo.toml");
const KERNEL: &str = include_str!("../../provider/src/k.wgsl");
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[test]
fn everything_read_is_there() {
    assert_eq!(c::shared_value() + GENERATED, 2);
    assert!(!PROVIDER_MANIFEST.is_empty() && !KERNEL.is_empty());
}
