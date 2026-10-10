#!/usr/bin/env python3
"""攻方零轮改法 F1（只在攻方副本上量过、被攻过零轮）：I-5.4 按版本那一层把「树表指针非零而树表读不出」的候选根也记成没走到，
报部分没判、点名那一版（今天这种根整版不进评估、别的版本的树走到了叶就报成立）。用法：i54_tree_table_fix.py <树根>。"""
import sys
path = sys.argv[1] + "/crates/singlefs-checker/src/walk.rs"
text = open(path, encoding="utf-8").read()
edits = [
    (
        "        let tree_table_pointer = parse_node_pointer(&root.record_bytes[36..122]);\n"
        "        if !tree_table_pointer.all_zero {\n"
        "            if let Some(tree_table) =\n"
        "                read_index_node_without_judging(reader, &tree_table_pointer, cache)\n"
        "            {\n",
        "        let tree_table_pointer = parse_node_pointer(&root.record_bytes[36..122]);\n"
        "        if !tree_table_pointer.all_zero {\n"
        "            let tree_table_read = read_index_node_without_judging(reader, &tree_table_pointer, cache);\n"
        "            if tree_table_read.is_none() {\n"
        "                versions_whose_tree_table_does_not_read_back\n"
        "                    .push(format!(\"根（实例 {}、txg {}）\", root.instance, root.checkpoint_txg));\n"
        "            }\n"
        "            if let Some(tree_table) = tree_table_read\n"
        "            {\n",
    ),
    (
        "    let mut versions_where_the_root_record_allocation_tree_root_is_zero: Vec<(String, bool)> =\n"
        "        Vec::new();\n",
        "    let mut versions_where_the_root_record_allocation_tree_root_is_zero: Vec<(String, bool)> =\n"
        "        Vec::new();\n"
        "    let mut versions_whose_tree_table_does_not_read_back: Vec<String> = Vec::new();\n",
    ),
    (
        "    if !versions_not_counted.is_empty() {\n",
        "    if !versions_whose_tree_table_does_not_read_back.is_empty() {\n"
        "        judgements.partially_judged(\n"
        "            \"I-5.4\",\n"
        "            format!(\n"
        "                \"树表读不出的版本，分配记录树认不出、一格没判（别的版本的树走到了叶不替它作保）：{}\",\n"
        "                versions_whose_tree_table_does_not_read_back.join(\"、\")\n"
        "            ),\n"
        "        );\n"
        "    }\n"
        "    if !versions_not_counted.is_empty() {\n",
    ),
]
for old, new in edits:
    if text.count(old) != 1:
        sys.exit(f"anchor hits {text.count(old)}: {old[:60]!r}")
    text = text.replace(old, new)
open(path, "w", encoding="utf-8").write(text)
print("applied F1")
