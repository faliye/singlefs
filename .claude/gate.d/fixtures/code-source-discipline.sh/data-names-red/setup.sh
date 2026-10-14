#!/usr/bin/env bash
# 红样本现场：data-names 那一格——一份实验源码里元组首列带没登记的缩写、=> 右侧带缩写、格式串里键= 的键名带缩写、元组首列是单字母，
# 一份 kb 表格里反引号包着的单元格带缩写；注释里和函数调用实参里的同形态串不判；排除表一行指向已经改好名的串、一行指向不存在的文件。
# 样本自己带 .claude/abbreviations（只登记 e<数字> 这一类编号，没登记 mac）。
set -e
mkdir -p .claude/kb scripts/experiments
printf 'e<数字>  # 样本：实验编号\n' > .claude/abbreviations
printf 'scripts/experiments/e1_demo.rs:renamed_long  # 样本：已经改好名还登记着\nscripts/experiments/gone.rs:cnt_total  # 样本：指向不存在的文件\n' > .claude/data-name-lint-exclude
cat > scripts/experiments/e1_demo.rs <<'RS'
const FIELDS: &[(&str, u32)] = &[("system_configuration_mac", 16), ("slot_count", 4)];
fn arm(selector: u8) -> &'static str {
    match selector {
        0 => "tail_cnt",
        _ => "tail_plain",
    }
}
fn report() {
    println!("name=demo num_items={}", 1);
    let _single = ("x", 1);
    check("cnt_note", 1);
}
// ("cnt_comment", 1) 写在注释里不判
RS
printf '| 字段 | 宽 |\n|---|---|\n| `cnt_total` | 8 |\n| `count_total` | 8 |\n' > .claude/kb/demo.md
git init -q . && git add -A
