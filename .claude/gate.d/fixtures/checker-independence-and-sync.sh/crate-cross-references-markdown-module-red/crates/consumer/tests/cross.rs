// 样本：#[path] 把提供包的一份 .md 当模块编进来（后缀是数据后缀也照源码判），另 include_str! 读提供包的一份 .md（只当数据读，放行）。
#[path = "../../provider/src/code.md"]
mod from_markdown;

const NOTES: &str = include_str!("../../provider/src/notes.md");

#[test]
fn markdown_module_compiles() {
    assert_eq!(from_markdown::from_markdown(), 1);
    assert!(!NOTES.is_empty());
}
