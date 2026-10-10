#!/usr/bin/env python3
"""patch_model2.py 之后的第三刀：环境变量 W3_OPS（只用这几个字母，逗号分隔，例 create,rollback1,empty,mount）、W3_LONGEST、W3_RING 覆盖配置，
用来把历史拉长到超过环长（C1b 的环窗口那一形：回退之后环转了一圈才挂载）。不设这些变量时行为与第二刀之后完全相同。
用法：patch_model3.py <输入 main.rs> <输出 main.rs>"""
import sys
src, out = sys.argv[1], sys.argv[2]
t = open(src, encoding="utf-8").read()
def once(old, new):
    global t
    n = t.count(old)
    if n != 1:
        sys.stderr.write(f"旧串命中 {n} 次：{old[:90]!r}\n"); sys.exit(3)
    t = t.replace(old, new)
once("""    for (index, op) in Op::ALL.iter().enumerate() {
        let mut next = state.clone();""", """    for (index, op) in Op::ALL.iter().enumerate() {
        if !op_allowed(*op) {
            continue;
        }
        let mut next = state.clone();""")
once("""fn run(config: Config) {""", """fn op_allowed(op: Op) -> bool {
    static ALLOWED: std::sync::OnceLock<Option<Vec<String>>> = std::sync::OnceLock::new();
    let allowed = ALLOWED.get_or_init(|| {
        std::env::var("W3_OPS").ok().map(|list| list.split(',').map(|name| name.trim().to_string()).collect())
    });
    allowed.as_ref().is_none_or(|names| names.contains(&op.code()))
}

fn run(mut config: Config) {
    if let Ok(value) = std::env::var("W3_LONGEST") {
        config.longest = value.parse().expect("W3_LONGEST 是整数");
    }
    if let Ok(value) = std::env::var("W3_RING") {
        config.ring_slots = value.parse().expect("W3_RING 是整数");
    }""")
open(out, "w", encoding="utf-8").write(t)
