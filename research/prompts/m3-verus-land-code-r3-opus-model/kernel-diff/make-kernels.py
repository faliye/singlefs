#!/usr/bin/env python3
"""从两份冻结副本取核，生成差分要的几份核到 <出口目录>（kernel-diff/src/kernels/）。每处替换的原文必须恰好命中一次，否则退 3。
用法：make-kernels.py <第二轮冻结副本根> <第三轮冻结副本根> <出口目录>"""
import pathlib
import sys

old_root, new_root, out = (pathlib.Path(argument) for argument in sys.argv[1:4])
KERNEL = "crates/singlefs-replay-prefix/src/lib.rs"
old_text = (old_root / KERNEL).read_text(encoding="utf-8")
new_text = (new_root / KERNEL).read_text(encoding="utf-8")


def replaced(text, original, replacement, label):
    hits = text.count(original)
    if hits != 1:
        print(f"ANCHOR {label} hits={hits}")
        sys.exit(3)
    return text.replace(original, replacement, 1)


variants = {
    "old.rs": old_text,
    "new.rs": new_text,
    # crates/mutations.tsv 第 1502 行改前的样子（第二轮冻结副本）：只跳过计数器恰好是下一号的那一条
    "old_k20.rs": replaced(old_text, "later.counter.0 >= next_counter {", "later.counter.0 > next_counter {", "old_k20"),
    # 同一行改后的样子（第三轮冻结副本）：从切片里隔一个位置起扫
    "new_k20.rs": replaced(new_text, "    let mut later_index = index + 1;", "    let mut later_index = index + 2;", "new_k20"),
    # H5-2 退回：锚点读不出那一支把「序号为 1」加回反向链回调之前
    "new_revert_h52.rs": replaced(
        new_text,
        "            if record.checkpoint_txg.0 != chain_start_txg_without_anchor.0 {\n                return CandidateVerdict::FailsACheckBeforeTheNamedUnits;",
        "            if record.checkpoint_txg.0 != chain_start_txg_without_anchor.0\n                || record.ordinal_within_publish.0 != 1 {\n                return CandidateVerdict::FailsACheckBeforeTheNamedUnits;",
        "new_revert_h52",
    ),
}
out.mkdir(parents=True, exist_ok=True)
for name, text in variants.items():
    (out / name).write_text(text, encoding="utf-8")
    print(f"WROTE {name}")
