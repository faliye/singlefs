#!/usr/bin/env python3
"""第 33 批攻方 P3：在攻方副本的 crates/ 上施加一个回收错法（恰好命中一次才写，否则退 3）。

用法：p3_mutants.py <副本根> <名字>
  行号形：row<N> —— 取副本里 crates/mutations.tsv 第 N 行的文件、原文、替换文（\\n、\\t 照字面转回）。
  另起的几种（攻方自己写的，不在变异表里）：
    floor_plus_one   进程内记根时回收门槛多一（多回收一代）
    floor_minus_one  进程内记根时回收门槛少一（少回收一代）
    reuse_keeps_old_generation  复用已回收落点改写记录时不换分配代（留着释放代）
    reclaim_device_zero_only    回收只在盘 0 上做（盘 1 那条记录照旧已释放、槽不回空闲）
"""
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
name = sys.argv[2]
FLOOR_ANCHOR = (
    "        let reclaim_floor = crate::mount::reclaim_floor(\n"
    "            occupancy.effective_floor,\n"
    "            occupancy.oldest_valid_root_in_the_ring(),\n"
    "        );"
)
CUSTOM = {
    "floor_plus_one": (
        "crates/singlefs-core/src/allocator.rs",
        FLOOR_ANCHOR,
        FLOOR_ANCHOR + "\n        let reclaim_floor = CheckpointTxg(reclaim_floor.0 + 1);",
    ),
    "floor_minus_one": (
        "crates/singlefs-core/src/allocator.rs",
        FLOOR_ANCHOR,
        FLOOR_ANCHOR + "\n        let reclaim_floor = CheckpointTxg(reclaim_floor.0.saturating_sub(1));",
    ),
    "reuse_keeps_old_generation": (
        "crates/singlefs-core/src/allocator.rs",
        "            existing.generation = generation;\n",
        "",
    ),
    "reclaim_device_zero_only": (
        "crates/singlefs-core/src/allocator.rs",
        "            .filter(|record| record.is_released && record.generation <= floor)\n",
        "            .filter(|record| record.is_released && record.generation <= floor && record.device == DeviceIdentity(0))\n",
    ),
}


def unescape(text):
    return text.replace("\\n", "\n").replace("\\t", "\t")


if name.startswith("row"):
    line_number = int(name[3:])
    row = (root / "crates/mutations.tsv").read_text().split("\n")[line_number - 1].split("\t")
    relative, anchor, replacement = row[1], unescape(row[2]), unescape(row[3])
else:
    relative, anchor, replacement = CUSTOM[name]
target = root / relative
text = target.read_text()
hits = text.count(anchor)
if hits != 1:
    print(f"{name}: 原文在 {relative} 命中 {hits} 次", file=sys.stderr)
    sys.exit(3)
target.write_text(text.replace(anchor, replacement))
print(f"MUTANT {name} applied to {relative}")
