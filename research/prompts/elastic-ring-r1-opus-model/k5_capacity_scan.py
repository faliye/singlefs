#!/usr/bin/env python3
"""elastic-ring-r1 云端攻方腿模型二：逐个容量点问「甲今天做不到、乙做得到」的是什么。

写死的输入（每行标来源）：
  环默认 768 MiB、上界 环 ≤ 容量 ÷ 4、越界拒绝 mkfs
      .claude/kb/decisions/23-journal的角色与格式.md 已定项 19 ③
  F = 3                    crates/singlefs-format/src/lib.rs 的 JOURNAL_SAFETY_FACTOR
  有效 T_dirty = min(字段 2 GiB, 环 ÷ F)
      .claude/kb/decisions/16-发布语义.md 已定项 5
  I-8.1 按「任一事务」读的最坏占用 12 KiB ⇒ 硬下界 F × 12 KiB = 36 KiB
      .claude/kb/checks-owed.md 的 C310 那一行
  单元区起点 = 槽 50176（784 MiB），格式常量
      crates/singlefs-format/src/lib.rs:194-195

甲 = 今天：默认 768 MiB，用户可在 mkfs 传任意 r ≤ 容量 ÷ 4。
乙 = 弹性环：mkfs 按容量算初值（这里取「容量 ÷ 64，夹进 [36 KiB, 容量 ÷ 4]」这一种），运行时还能缩。
问的是：哪一个容量点上，甲传得出的那组 r 里一个都达不到乙达到的效果。
"""

KIB = 1024
MIB = 1024 * KIB
GIB = 1024 * MIB
SAFETY_FACTOR = 3
DEFAULT_RING = 768 * MIB
DIRTY_FIELD = 2 * GIB
WORST_CASE_PER_TRANSACTION = 12 * KIB
HARD_FLOOR = SAFETY_FACTOR * WORST_CASE_PER_TRANSACTION
UNIT_AREA_START = 50176 * 16384


def effective_dirty(ring: int) -> int:
    return min(DIRTY_FIELD, ring // SAFETY_FACTOR)


def human(bytes_value: float) -> str:
    for unit, size in (("GiB", GIB), ("MiB", MIB), ("KiB", KIB)):
        if bytes_value >= size:
            return f"{bytes_value / size:.4g} {unit}"
    return f"{bytes_value:.0f} B"


def main() -> None:
    capacities = [
        512 * MIB, 1 * GIB, 2 * GIB, 3 * GIB, 4 * GIB, 16 * GIB, 64 * GIB,
        200 * GIB, 500 * GIB, 2 * 1024 * GIB,
    ]
    print("| 容量 | 甲默认（768 MiB）能不能 mkfs | 甲默认有效 T_dirty | 甲能传的最大环（容量÷4） | 甲传满时有效 T_dirty | 乙自动初值（容量÷64 夹取） | 乙有效 T_dirty | 乙买到甲传不出的东西吗 | 缩到硬下界能腾出 | 占容量 |")
    print("|---|---|---|---|---|---|---|---|---|---|")
    for capacity in capacities:
        ceiling = capacity // 4
        default_ok = DEFAULT_RING <= ceiling
        default_dirty = effective_dirty(DEFAULT_RING) if default_ok else 0
        elastic = min(max(capacity // 64, HARD_FLOOR), ceiling)
        reachable_by_parameter = elastic <= ceiling  # 甲传得出同一个数吗
        freed = max(0, elastic - HARD_FLOOR)
        print(
            f"| {human(capacity)} | {'能' if default_ok else '不能（越界拒绝）'} | "
            f"{human(default_dirty) if default_ok else '—'} | {human(ceiling)} | "
            f"{human(effective_dirty(ceiling))} | {human(elastic)} | {human(effective_dirty(elastic))} | "
            f"{'没有：同一个环长甲传得出' if reachable_by_parameter else '有'} | "
            f"{human(freed)} | {freed / capacity * 100:.2f}% |"
        )
    print()
    print("单元区起点是格式常量 50176 槽 =", human(UNIT_AREA_START), "，与环长无关：")
    print("| 环长 | 环末尾槽 | 单元区起点常量 | 重叠 / 空洞 |")
    print("|---|---|---|---|")
    for ring in [12 * KIB, 64 * MIB, 256 * MIB, DEFAULT_RING, 1 * GIB, 6 * GIB]:
        end_slot = 1024 + ring // 16384
        delta = end_slot - 50176
        kind = "重叠" if delta > 0 else "空洞"
        print(f"| {human(ring)} | {end_slot} | 50176 | {kind} {human(abs(delta) * 16384)} |")


if __name__ == "__main__":
    main()
