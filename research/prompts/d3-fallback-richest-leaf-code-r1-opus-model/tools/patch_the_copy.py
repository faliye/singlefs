#!/usr/bin/env python3
"""把攻方腿模型的一条臂装进副本：从冻结副本读分配器原文，按臂做定点替换（每个旧串必须恰好命中一次），写进副本；再把探针测试拷进副本的验证包。

用法：patch_the_copy.py <冻结副本> <副本> <探针目录> <臂>
  臂：as-coded（不改）、prefer-then-lowest（先重写集、不够退回各盘槽号最小的空槽）、lowest-only（改前的政策：各盘槽号最小的空槽）
退出码：0 装好；2 用法错或旧串命中次数不是 1。
两处替换的原文与替换文逐字取自 crates/mutations.tsv 里点名「重写集的叶不够时回落退回各盘槽号最小的空槽」与「提交内生块回落不先取重写集的叶」的那两行。
"""
import pathlib
import shutil
import sys

PROBE_FILE = "fallback_settling_on_forged_disks_without_a_rich_leaf.rs"
ALLOCATOR = pathlib.PurePosixPath("crates/singlefs-core/src/allocator.rs")

RICHEST_LEAF_STEP = (
    "            .or_else(|| {\n"
    "                self.lowest_slot_in_the_leaf_with_the_most_slots_open_on_every_device(\n"
    "                    footprint,\n"
    "                    excluded_region,\n"
    "                )\n"
    "            })",
    "            .or(None)",
)
REWRITE_SET_STEP = (
    "        rewrite_set_leaves\n"
    "            .leaf_indices\n"
    "            .iter()\n"
    "            .find_map(|leaf_index| {",
    "        RewriteSetLeaves::none()\n"
    "            .leaf_indices\n"
    "            .iter()\n"
    "            .find_map(|leaf_index| {",
)
REPLACEMENTS_OF_THE_ARM = {
    "as-coded": [],
    "prefer-then-lowest": [RICHEST_LEAF_STEP],
    "lowest-only": [RICHEST_LEAF_STEP, REWRITE_SET_STEP],
}


def main(arguments: list[str]) -> int:
    if len(arguments) != 4 or arguments[3] not in REPLACEMENTS_OF_THE_ARM:
        print("用法：patch_the_copy.py <冻结副本> <副本> <探针目录> <臂>；臂取 "
              + "、".join(REPLACEMENTS_OF_THE_ARM), file=sys.stderr)
        return 2
    frozen, copy, probe, arm = (pathlib.Path(arguments[0]), pathlib.Path(arguments[1]),
                                pathlib.Path(arguments[2]), arguments[3])
    text = (frozen / ALLOCATOR).read_text(encoding="utf-8")
    for original, replacement in REPLACEMENTS_OF_THE_ARM[arm]:
        hits = text.count(original)
        if hits != 1:
            print(f"  ✗ 臂 {arm}：旧串在 {ALLOCATOR} 里命中 {hits} 次（要恰好 1 次）：{original[:60]!r}", file=sys.stderr)
            return 2
        text = text.replace(original, replacement)
    (copy / ALLOCATOR).write_text(text, encoding="utf-8")
    shutil.copyfile(probe / PROBE_FILE, copy / "crates/singlefs-verification/tests" / PROBE_FILE)
    print(f"name=arm_installed arm={arm} replacements={len(REPLACEMENTS_OF_THE_ARM[arm])}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
