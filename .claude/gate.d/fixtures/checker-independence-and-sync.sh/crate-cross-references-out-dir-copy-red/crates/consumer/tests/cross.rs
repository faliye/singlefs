// 样本：把构建脚本 OUT_DIR 里的两份都 include! 进来；copied.rs 与提供包的 src/x.rs 逐字节相同，照别的包文件判红。
include!(concat!(env!("OUT_DIR"), "/copied.rs"));
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[test]
fn copied_and_generated_are_compiled() {
    assert_eq!(copied() + GENERATED, 8);
}
