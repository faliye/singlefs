#!/usr/bin/env python3
"""在冻结副本的拷贝里（参数：拷贝的 tree 目录）给 crates/singlefs-core/src/recovery.rs 加 r5probe 的副本变异开关，
并把 r5probe 加进工作区成员。只改拷贝，不碰仓。"""
import pathlib
import sys

tree = pathlib.Path(sys.argv[1])
recovery = tree / "crates/singlefs-core/src/recovery.rs"
text = recovery.read_text()
old = "                tree_table: record.new_tree_table,\n"
assert text.count(old) == 1, "recovery.rs 里那一行不是恰好一处"
new = ("                // r5probe 副本变异（只在攻方草稿副本里）：R5_MUTATION=keep_tree_table 时施加记录只换身份与别的字段、树表指针照抄所选根。\n"
       "                tree_table: if r5_mutation_keeps_the_chosen_tree_table() { root.tree_table } else { record.new_tree_table },\n")
text = text.replace(old, new)
text += ("\n/// r5probe 副本变异开关（只在攻方草稿副本里）。\n"
         "fn r5_mutation_keeps_the_chosen_tree_table() -> bool {\n"
         "    static FLAG: std::sync::OnceLock<bool> = std::sync::OnceLock::new();\n"
         "    *FLAG.get_or_init(|| std::env::var(\"R5_MUTATION\").as_deref() == Ok(\"keep_tree_table\"))\n}\n")
recovery.write_text(text)
cargo = tree / "Cargo.toml"
members = cargo.read_text()
anchor = '    "crates/singlefs-checker-tier",\n]'
assert anchor in members, "工作区成员表的末尾不是预期的样子"
cargo.write_text(members.replace(anchor, '    "crates/singlefs-checker-tier",\n    "crates/r5probe",\n]'))
