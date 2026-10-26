#!/usr/bin/env bash
# 绿样本现场：data-names 那一格——样本自己的 .claude/abbreviations 登记了缩写 mac 与整类编号 e<数字>；缩写只出现在登记过的串里、注释里、
# 函数调用的实参里、kb 的围栏代码块与表格外的正文里；一个还带缩写的串登记在排除表里（还没改完名）。
set -e
mkdir -p .claude/kb scripts/experiment-apparatus
printf 'e<数字>  # 样本：实验编号\nmac  # 样本：message authentication code，校验标签\n' > .claude/abbreviations
printf 'scripts/experiment-apparatus/e1_demo.rs:ino_old  # 样本：还没改完名\n' > .claude/data-name-lint-exclude
cat > scripts/experiment-apparatus/e1_demo.rs <<'RS'
const FIELDS: &[(&str, u32)] = &[("system_configuration_mac", 16), ("e100_slot_count", 4), ("ino_old", 8)];
fn arm(selector: u8) -> &'static str {
    match selector {
        0 => "tail_count",
        _ => "tail_plain",
    }
}
fn report() {
    println!("name=demo item_count={}", 1);
    check("cnt_note", 1);
    let _text = "not a name";
}
// ("cnt_comment", 1) 和 => "cnt_comment" 写在注释里不判
RS
printf '| 字段 | 宽 |\n|---|---|\n| `count_total` | 8 |\n\n```\n| `cnt_fenced` | 8 |\n```\n\n正文里的 `cnt_prose` 不是表格单元格\n' > .claude/kb/demo.md
git init -q . && git add -A
