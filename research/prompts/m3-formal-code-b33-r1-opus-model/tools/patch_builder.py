#!/usr/bin/env python3
"""攻方副本里改 checker_known_bad_images.rs：搭建结构多带一版、末尾追加 src/x_records_applied_versions.rs。

用法：patch_builder.py <副本根> <模型目录>。每处旧串必须恰好命中一次，否则退出 3、一个字节不写。
"""
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
model = pathlib.Path(sys.argv[2])
target = root / "crates/singlefs-harness/tests/checker_known_bad_images.rs"
text = target.read_text()
replacements = [
    (
        "    tree_table_slot_of_the_version_applied_only_by_its_record: u64,\n}",
        "    tree_table_slot_of_the_version_applied_only_by_its_record: u64,\n"
        "    version_applied_only_by_its_record: singlefs_core::transaction::TransactionOutput,\n}",
    ),
    (
        "        tree_table_slot_of_the_version_applied_only_by_its_record: applied_only_by_its_record\n",
        "        version_applied_only_by_its_record: applied_only_by_its_record.clone(),\n"
        "        tree_table_slot_of_the_version_applied_only_by_its_record: applied_only_by_its_record\n",
    ),
]
for old, new in replacements:
    hits = text.count(old)
    if hits != 1:
        print(f"旧串命中 {hits} 次：{old[:60]!r}", file=sys.stderr)
        sys.exit(3)
    text = text.replace(old, new)
if "b33x_line" in text:
    print("已经打过", file=sys.stderr)
    sys.exit(3)
text += "\n" + (model / "src/x_records_applied_versions.rs").read_text()
target.write_text(text)
print(f"patched {target}")
