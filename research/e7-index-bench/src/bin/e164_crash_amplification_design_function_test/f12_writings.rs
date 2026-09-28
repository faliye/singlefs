// E164（崩溃放量新设计的小范围功能测试）F12 的五种写法（跑前登记 5.7），每种一段，以「// writing N:」一行起头；
// 装置把 `#![forbid(unsafe_code)]`、一种类型形（design 模块）与其中一段拼成一份 .rs。
// writing 1: 标签直接当库的键传进去
pub fn store_verdict_under_the_label(store: &mut design::VerdictStore, label: design::NodeLabel) {
    store.put(label, 1);
}
// writing 2: 标签放进 HashMap 当键
pub fn map_verdict_under_the_label(label: design::NodeLabel) -> usize {
    let mut verdicts = std::collections::HashMap::new();
    verdicts.insert(label, 1u8);
    verdicts.len()
}
// writing 3: 拿标签造复用键
pub fn reuse_key_from_the_label(label: &design::NodeLabel) -> design::ReuseKey {
    design::ReuseKey::from(label)
}
// writing 4: 拿标签的文字当内存里的键
pub fn map_verdict_under_the_label_text(label: &design::NodeLabel) -> usize {
    let mut verdicts = std::collections::HashMap::new();
    verdicts.insert(label.to_string(), 1u8);
    verdicts.len()
}
// writing 5: 合法写法：把标签交给覆盖报告
pub fn hand_the_label_to_the_coverage_report(report: &mut design::CoverageReport, label: design::NodeLabel) {
    report.record(label);
}
