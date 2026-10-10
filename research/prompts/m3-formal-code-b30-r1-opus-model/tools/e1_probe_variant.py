#!/usr/bin/env python3
"""E1 那一句的探针臂（只为数「合法历史上走到了几次墓碑树 / 谱系树容器子指针」）：把 E1 新加的那一判换成恒判 I-2.4 不成立。
用法：e1_probe_variant.py <树根> apply|restore（restore 从 <冻结副本> 拷回，冻结副本由 FROZEN 给）。"""
import os, shutil, sys
root, action = sys.argv[1], sys.argv[2]
path = "crates/singlefs-checker/src/walk.rs"
anchor = "            judge_pointer_extent_offset_is_zero(&mut self.judgements, &child, &what);\n            if entry_type == 0 {\n"
probe = "            self.judgements.judge(\"I-2.4\", false, || format!(\"B30PROBE {what}\"));\n            if entry_type == 0 {\n"
if action == "restore":
    shutil.copyfile(os.path.join(os.environ["FROZEN"], path), os.path.join(root, path))
    sys.exit(0)
text = open(os.path.join(root, path), encoding="utf-8").read()
if text.count(anchor) != 1:
    sys.exit(f"anchor hits {text.count(anchor)}")
open(os.path.join(root, path), "w", encoding="utf-8").write(text.replace(anchor, probe))
print("applied e1 probe arm")
