#!/usr/bin/env python3
"""从运行库用例共用搭建（冻结副本的拷贝里那一份）生成探针自己的那一份小池搭建：盘 320 MiB、日志环 64 MiB，单元区 240 段。

mkfs 要环不超过盘的四分之一（make_filesystem.rs 的 JournalRingTooLargeForDevice），单元区从环末尾的下一个槽起，
所以默认 768 MiB 的环至少要 3 GiB 的盘（单元区 2288 段）。环放小之后 mkfs 同一个进程里接着写的那个分配器要照 mkfs 交回的起点建。
用法：make_the_small_pool_fixture.py <共用搭建> <小池搭建>。逐条旧串都要恰好一处，不是就退出、不写。
"""
import sys

source_path, target_path = sys.argv[1:]
with open(source_path, encoding="utf-8") as source:
    text = source.read()
for old, new in [
    (
        "pub const DEVICE_BYTES: u64 = 4 << 30;",
        "pub const DEVICE_BYTES: u64 = 320 << 20;",
    ),
    (
        "journal_ring_bytes: default_journal_ring_bytes(),",
        "journal_ring_bytes: 64 << 20,",
    ),
    (
        "DeviceFreeMap::new(DeviceIdentity(0), DEVICE_BYTES),",
        "DeviceFreeMap::with_unit_area_start(DeviceIdentity(0), DEVICE_BYTES, genesis.unit_area_start),",
    ),
    (
        "DeviceFreeMap::new(DeviceIdentity(1), DEVICE_BYTES),",
        "DeviceFreeMap::with_unit_area_start(DeviceIdentity(1), DEVICE_BYTES, genesis.unit_area_start),",
    ),
]:
    if text.count(old) != 1:
        sys.exit(f"{source_path}: {old} 不是恰好一处")
    text = text.replace(old, new)
with open(target_path, "x", encoding="utf-8") as target:
    target.write(text)
