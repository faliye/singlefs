#!/usr/bin/env python3
"""elastic-ring-r1 云端攻方腿模型一：缩环之后，水位之上那一段还能重放几条。

只算术，不读盘。把今天代码里的两个函数原样抄成 python：

  crates/singlefs-core/src/journal.rs:109-114
      record_offset(counter, ring_bytes) = 环起点 + ((counter - 1) % (ring_bytes / 4096)) * 4096
  crates/singlefs-core/src/recovery.rs:843-874 scan_journal
      扫的范围是 [环起点, 环起点 + 系统配置里的 journal_ring_bytes)，按 (实例, counter) 收
  crates/singlefs-core/src/recovery.rs:896-921 replay_journal
      水位之上按 counter 排序，断号即止

缩环 = 系统配置里的 journal_ring_bytes 从 S 改成 S'（S' < S）。
盘上的字节一个都没动，动的只是「用哪个模数解读它们」与「扫到哪里为止」。
一条记录 c 在缩之后还找得到，当且仅当它的旧位置 (c-1) % S 落在 [0, S') 里，
且没有更晚的记录 c + k*S（k ≥ 1，c + k*S ≤ N）把那一格盖掉。
"""

import argparse
import statistics

RECORD_BYTES = 4096


def slots(ring_bytes: int) -> int:
    return ring_bytes // RECORD_BYTES


def position(counter: int, ring_slots: int) -> int:
    return (counter - 1) % ring_slots


def survivors(water: int, last: int, old_slots: int, new_slots: int):
    """水位之上 [water+1, last] 这一段里，缩环之后 scan_journal 还读得到的 counter。"""
    found = set()
    for counter in range(water + 1, last + 1):
        p = position(counter, old_slots)
        if p >= new_slots:
            continue  # 物理位置落在新环之外：再也不扫它
        overwritten = any(
            later <= last for later in range(counter + old_slots, last + 1, old_slots)
        )
        if not overwritten:
            found.add(counter)
    return found


def applied_prefix(water: int, last: int, old_slots: int, new_slots: int) -> int:
    """断号即止：从 water+1 起连续几条。"""
    found = survivors(water, last, old_slots, new_slots)
    applied = 0
    counter = water + 1
    while counter in found:
        applied += 1
        counter += 1
    return applied


def scan(old_bytes: int, new_bytes: int, depth: int, samples: int):
    """把「缩的时候水位停在环的哪一格」放开扫一遍（那一步由用户的写量决定，不由装置定）。"""
    old_slots, new_slots = slots(old_bytes), slots(new_bytes)
    results = []
    for index in range(samples):
        water = 1 + index * max(1, old_slots // samples)
        results.append(applied_prefix(water, water + depth, old_slots, new_slots))
    return old_slots, new_slots, results


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--samples", type=int, default=512)
    arguments = parser.parse_args()
    mib = 1024 * 1024
    print("环从 S 缩到 S'，水位之上 depth 条还没重放；断号即止之后真能施加几条")
    print("| 旧环 | 新环 | 在飞深度 | 取样点 | 施加 0 条的取样点 | 全部施加的取样点 | 施加条数均值 | 解析：施加 0 条的残差比例 | 解析：全部施加的残差比例 |")
    print("|---|---|---|---|---|---|---|---|---|")
    for old_mib, new_mib in [(768, 64), (768, 256), (768, 384), (768, 767), (256, 64), (64, 12 / 1024)]:
        for depth in (1, 4, 16):
            old_bytes = int(old_mib * mib)
            new_bytes = int(new_mib * mib)
            old_slots, new_slots, results = scan(old_bytes, new_bytes, depth, arguments.samples)
            zero = sum(1 for value in results if value == 0)
            full = sum(1 for value in results if value == depth)
            print(
                f"| {old_mib} MiB（{old_slots} 槽） | {new_mib} MiB（{new_slots} 槽） | {depth} | "
                f"{len(results)} | {zero} | {full} | {statistics.mean(results):.2f} | "
                f"{(old_slots - new_slots) / old_slots:.4f} | {max(0, new_slots - depth + 1) / old_slots:.4f} |"
            )


if __name__ == "__main__":
    main()
