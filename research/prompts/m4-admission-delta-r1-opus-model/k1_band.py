#!/usr/bin/env python3
"""m4-admission-delta-r1 攻方腿的纯算术模型（只在攻方模型上算过、被攻过零轮）。

照冻结副本 crates/singlefs-core/src/admission.rs 的 allocation_record_tree_positions_over_the_unit_areas 与
crates/singlefs-core/src/allocation_record_tree.rs 的 AllocationRecordTreeGeometry::of_devices 抄式子（S_L = 812 × 169^L，
根层级取每块盘一格装得进 169 的最低一层），算各几何上分配记录树的 K1（D28（挂载期承诺量） 已定项 4 的整棵树上界）：
- 候选 A3 把这棵树的固定点挪到准入之后、准入判上界时，这一项只能取 K1（固定点自己的分配改自己的形状，
  「进出 key 数 × 树高」那一类式子对它是循环的：x = (k + x) × h 在 h ≥ 1 时没有有限解）；
- 今天式子那一判读的是走到固定点的实际重写数；两者之差就是 A3 比今天多拒的那一带的下界。

单元区起点各几何都取两块 4 GiB 盘那一份（50176 槽，admission.rs 那段文档里写的），大盘上的真起点没查（推的）。
「今天的实际重写数」取 D28 已定项 4 引的实审 A4c「一块盘改 3–4 片叶」那一句，按每块盘 4 片叶连到根算（推的）。
"""
import sys

LEAF_SLOTS = 812
FANOUT = 169
SLOT_BYTES = 16384
UNIT_AREA_START = 50176
DEVICES = 2


def span(level):
    return LEAF_SLOTS * FANOUT ** level


def root_level(slots_of_each_device):
    for candidate in range(1, 256):
        if sum(-(-slots // span(candidate - 1)) for slots in slots_of_each_device) <= FANOUT:
            return candidate
    raise SystemExit("根层级超过 255")


def k1(device_bytes):
    slots = device_bytes // SLOT_BYTES
    level_of_the_root = root_level([slots] * DEVICES)
    positions = [(slots - 1) // span(level) - UNIT_AREA_START // span(level) + 1 for level in range(level_of_the_root)]
    return slots, level_of_the_root, positions, 1 + DEVICES * sum(positions)


def todays_settled_allocation_record_nodes(level_of_the_root, leaves_per_device=4):
    # 每块盘 leaves_per_device 片叶，叶之上每层至多 leaves_per_device 个祖先（取大），加根。
    return 1 + DEVICES * leaves_per_device * level_of_the_root


def circular_keys_times_height(changed_records, height, cap, rounds=12):
    # x_{n+1} = (k + x_n) × h：固定点自己的分配算进「进出的 key」时的迭代，封顶在 K1。
    trail, x = [], 0
    for _ in range(rounds):
        x = min((changed_records + x) * height, cap)
        trail.append(x)
    return trail


def main():
    for name, device_bytes in [("4GiB", 4 << 30), ("64GiB", 64 << 30), ("1TiB", 1 << 40), ("16TiB", 16 << 40)]:
        slots, level_of_the_root, positions, whole_tree = k1(device_bytes)
        settled = todays_settled_allocation_record_nodes(level_of_the_root)
        band_blocks = whole_tree - settled
        band_mib = band_blocks * SLOT_BYTES / 2 ** 20
        trail = circular_keys_times_height(changed_records=4, height=level_of_the_root + 1, cap=whole_tree)
        print(
            f"name=k1_band geometry=2x{name} slots_per_device={slots} root_level={level_of_the_root} "
            f"positions_per_device_by_level={positions} k1={whole_tree} todays_settled_nodes_assumed={settled} "
            f"band_blocks_at_least={band_blocks} band_per_device_mib={band_mib:.1f} "
            f"band_share_of_device_percent={100 * band_blocks * SLOT_BYTES / device_bytes:.3f} "
            f"circular_trail={trail}"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
