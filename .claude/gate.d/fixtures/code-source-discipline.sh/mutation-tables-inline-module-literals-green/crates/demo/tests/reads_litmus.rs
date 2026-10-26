// 门禁 code-source-discipline 的判别力样本：逃出 crates/ 的相对路径指到 litmus/（门禁 checker-tier-crates-mutation-replay 的拷贝范围里有它），不判红。
// 注释里写的 "../../fixture-data/answer.txt" 不算；#[path = …] 指的是编译期带进来的源文件，也不算。
#[path = "../../helpers-outside-the-copy/mod.rs"]
mod helpers_outside_the_copy;

#[test]
fn reads_the_litmus_directory() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../litmus").join("sample.litmus");
    assert!(std::fs::read_to_string(path).unwrap().contains("exists (x=1)"));
}
