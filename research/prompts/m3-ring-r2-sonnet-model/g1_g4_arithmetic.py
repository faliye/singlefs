#!/usr/bin/env python3
"""m3-ring-r2 辩方腿的两处纯算术：记录头加字段后一条记录装几个点名项（G1）、整轮清扫读多少个槽（G4）。
常量取冻结副本 crates/singlefs-format/src/lib.rs：JOURNAL_RECORD_BYTES 4096、JOURNAL_HEADER_BYTES 311、
JOURNAL_NAMED_ENTRY_BYTES 56、SLOT_BYTES 16384、NODE_POINTER_BYTES 86、INSTANCE_TABLE_PAGE_RECORDS 370。
不读盘、不编译、不碰 crates/。"""
RECORD = 4096
HEADER = 311
ENTRY = 56
POINTER = 86
SLOT = 16384


def capacity(extra_bytes):
    return (RECORD - HEADER - extra_bytes) // ENTRY


variants = [
    ("现行", 0),
    ("正推腿三字段：实例表指针 86 + 分配记录树根指针 86 + 记号 4", POINTER + POINTER + 4),
    ("只加 1 个标志字节（记号一位、分配记录树根一位）", 1),
    ("借记录标志字节里的两位（头不涨）", 0),
]
for name, extra in variants:
    print(f"{name}: 头 {HEADER + extra} 字节，装 {capacity(extra)} 个点名项")
print("多占多少字节点名项仍是现行那个数：至多",
      max(x for x in range(0, 200) if capacity(x) == capacity(0)))

for label, size in (("1 TiB", 1 << 40), ("4 TiB", 4 << 40)):
    slots = size // SLOT
    print(f"{label} 一块盘：{slots} 个槽；每槽读头 4096 字节共 {slots * 4096 / (1 << 30):.0f} GiB；两块盘各读一遍共 {2 * slots} 次读")

for m in (10_000, 1_000_000):
    pages = -(-(m + 3) // 369)
    print(f"M={m}: 片数 {pages}")
