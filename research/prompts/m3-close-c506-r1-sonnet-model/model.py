#!/usr/bin/env python3
"""C506 正推腿的纯算术模型：每区槽数 S 的上界 16 与根环几何、环窗口常驻扣项的关系。

只做算术，不跑 crates 的代码。常量是本地抄的（出处在每行后面），开头 `check_against_source` 用正则回读 crates 源码里同名 const，
对不上就退 1：装置的数与实现对不上时两边都查，不默认哪边对。
用法：python3 -I model.py --repo <仓根>
"""
import argparse
import math
import re
import sys

# ---- 本地常量（抄自 crates/，出处 = 文件::常量名）----
LOCAL = {
    "ROOT_RING_REGIONS": 3,                      # singlefs-format/src/lib.rs
    "ROOT_RING_PRIME_STEP": 3,                   # 同上
    "ROOT_RING_CHUNK_BYTES": 1_048_576,          # 同上
    "ROOT_RING_BASE_SLOT": 64,                   # 同上
    "SLOT_BYTES": 16384,                         # 同上
    "JOURNAL_RING_START_SLOT": 1024,             # 同上
    "ROOT_RING_SLOTS_PER_REGION_MINIMUM": 4,     # 同上
    "ROOT_RING_SLOTS_PER_REGION_MAXIMUM": 16,    # 同上
    "ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM": 8,  # 同上
    "FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES": 4096,  # 同上
    "SYSTEM_CONFIGURATION_SLOT_BYTES": 4096,     # 同上
    "ALLOCATION_RECORD_TREE_LEAF_SLOTS": 812,    # 同上
    "ALLOCATION_RECORD_TREE_INTERNAL_FANOUT": 169,  # 同上
    "EXTENT_TREE_LOWER_LEAF_DATA_UNITS": 144,    # 同上
}
# admission.rs 里 W 式子的三个常数（admission.rs::DELETION_PUBLISH_*）
W_OUTSIDE_TREE = 24
W_RETIREMENT_BEYOND_LEVELS = 6
W_LEVEL_FACTOR = 2
W_LEVEL_UNITS_BEYOND_BATCH = 26

MIB = 1 << 20
GIB = 1 << 30


def check_against_source(repo):
    text = open(f"{repo}/crates/singlefs-format/src/lib.rs", encoding="utf-8").read()
    bad = []
    for name, want in LOCAL.items():
        m = re.search(rf"^pub const {name}: u\d+ = ([0-9_]+);", text, re.M)
        if not m:
            bad.append(f"{name}: 源码里找不到")
            continue
        got = int(m.group(1).replace("_", ""))
        if got != want:
            bad.append(f"{name}: 本地 {want} 源码 {got}")
    for name, want in (("DELETION_PUBLISH_SLOTS_OUTSIDE_THE_ALLOCATION_RECORD_TREE_AT_MOST", W_OUTSIDE_TREE),
                       ("DELETION_PUBLISH_RETIREMENT_TREE_SLOTS_BEYOND_ITS_INTERNAL_LEVELS_AT_MOST", W_RETIREMENT_BEYOND_LEVELS),
                       ("DELETION_PUBLISH_ALLOCATION_RECORD_TREE_LEVEL_BOUND_FACTOR", W_LEVEL_FACTOR),
                       ("DELETION_PUBLISH_ALLOCATION_RECORD_TREE_LEVEL_BOUND_UNITS_BEYOND_THE_BATCH", W_LEVEL_UNITS_BEYOND_BATCH)):
        admission = open(f"{repo}/crates/singlefs-core/src/admission.rs", encoding="utf-8").read()
        m = re.search(rf"pub const {name}: u64 = ([0-9]+);", admission)
        if not m or int(m.group(1)) != want:
            bad.append(f"{name}: 源码 {m.group(1) if m else '找不到'}，本地 {want}")
    if bad:
        print("name=source_check result=MISMATCH " + "; ".join(bad))
        sys.exit(1)
    print(f"name=source_check result=ok checked={len(LOCAL) + 4}")


def region_geometry_table():
    """S × 槽距 的区域落点表：区域 r 起点 = base + r × P × chunk（root_ring.rs::region_start），
    区域长 = S × 槽距（root_ring.rs::region_length_in_bytes）。mkfs 判两两不重叠（make_filesystem.rs::check_geometry
    里 fixed_structure_extents + overlaps）；读者一侧（recovery.rs::system_configuration_values_this_reader_accepts、
    image.rs::geometry_of）没有这一判。"""
    L = LOCAL
    base = L["ROOT_RING_BASE_SLOT"] * L["SLOT_BYTES"]
    stride = L["ROOT_RING_PRIME_STEP"] * L["ROOT_RING_CHUNK_BYTES"]
    journal_start = L["JOURNAL_RING_START_SLOT"] * L["SLOT_BYTES"]
    spacing_max = L["ROOT_RING_BASE_SLOT"] * L["SLOT_BYTES"] - L["SYSTEM_CONFIGURATION_SLOT_BYTES"]
    print(f"name=region_geometry base={base} stride={stride} journal_start={journal_start} spacing_max={spacing_max}")
    for s in (4, 8, 16):
        print(f"name=largest_spacing_that_keeps_regions_apart S={s} bytes={stride // s} kib={stride // s // 1024} "
              f"largest_spacing_inside_one_chunk_bytes={L['ROOT_RING_CHUNK_BYTES'] // s}")
    spacings = [4096, 8192, 65536, 131072, 196608, 262144, 393216, 524288, 786432, spacing_max]
    for spacing in spacings:
        for s in (4, 8, 16):
            length = s * spacing
            starts = [base + r * stride for r in range(L["ROOT_RING_REGIONS"])]
            ends = [st + length for st in starts]
            region_to_region_overlap = any(ends[r] > starts[r + 1] for r in range(len(starts) - 1))
            hits_journal = ends[-1] > journal_start
            mkfs_refuses = region_to_region_overlap or hits_journal
            print(f"name=cell spacing={spacing} S={s} region_bytes={length} fits_one_chunk={length <= L['ROOT_RING_CHUNK_BYTES']} "
                  f"fits_stride={length <= stride} last_region_end={ends[-1]} hits_journal={hits_journal} "
                  f"mkfs_refuses_overlap={mkfs_refuses} reader_table_accepts=True")


def pin_byte_and_capacity_limits():
    """根记录钉槽 p 是 1 字节、255 = 没有钉着（root_record.rs::PinnedRingSlot）；p 小于环长 3S ⇒ 3S ≤ 255 ⇒ S ≤ 85。
    GPU 判器的根表容量 MAX_ROOTS = 48（crash_judge_common.wgsl::MAX_ROOTS），= 3 × 16。"""
    r = LOCAL["ROOT_RING_REGIONS"]
    print(f"name=other_limits pin_byte_largest_S={255 // r} gpu_max_roots=48 gpu_max_S={48 // r} "
          f"system_configuration_byte_domain=0..255")


def root_level(slots_of_devices):
    leaf = LOCAL["ALLOCATION_RECORD_TREE_LEAF_SLOTS"]
    fan = LOCAL["ALLOCATION_RECORD_TREE_INTERNAL_FANOUT"]
    for level in range(1, 256):
        child_span = leaf * fan ** (level - 1)
        if sum(math.ceil(s / child_span) for s in slots_of_devices) <= fan:
            return level
    raise SystemExit("root level not found")


def ring_window_slots(device_bytes_each, devices, journal_ring_bytes, s, batch=144, retirement_levels=0):
    """admission.rs::ring_window_on_one_device 的算术复写：(3S+1) × W，W = 24 + (6 + H) + Σ_{ℓ<根层级} min(N_ℓ, 2(K+26))。
    N_ℓ 按 admission.rs::allocation_record_tree_positions_at_each_level_below_the_root（全部盘合计）。
    单元区起点 = 环末尾下一个槽，向上取整到 64 槽聚簇段（make_filesystem.rs 文档里的口径，这里按它复写，推的）。"""
    L = LOCAL
    abs_slots = device_bytes_each // L["SLOT_BYTES"]
    first = L["JOURNAL_RING_START_SLOT"] + math.ceil(journal_ring_bytes / L["SLOT_BYTES"])
    first = math.ceil(first / 64) * 64
    if abs_slots <= first:
        return None
    last = abs_slots - 1
    level_root = root_level([abs_slots] * devices)
    bound = W_LEVEL_FACTOR * (batch + W_LEVEL_UNITS_BEYOND_BATCH)
    total = W_OUTSIDE_TREE + W_RETIREMENT_BEYOND_LEVELS + retirement_levels
    for level in range(level_root):
        span = L["ALLOCATION_RECORD_TREE_LEAF_SLOTS"] * L["ALLOCATION_RECORD_TREE_INTERNAL_FANOUT"] ** level
        n = (last // span - first // span + 1) * devices
        total += min(n, bound)
    return (L["ROOT_RING_REGIONS"] * s + 1) * total, total, level_root, first


def ring_window_table():
    print("name=ring_window_note derived_by=arithmetic_copy_of_admission_rs (not the implementation run); K=144, H_retirement=0, two equal devices")
    for gib, ring_mib in ((1, 64), (2, 64), (4, 768), (16, 768), (100, 768), (1000, 768)):
        dev = gib * GIB
        for s in (4, 8, 16):
            res = ring_window_slots(dev, 2, ring_mib * MIB, s)
            if res is None:
                print(f"name=ring_window device_gib={gib} S={s} device_too_small_for_the_unit_area")
                continue
            slots, w, level, first = res
            window_bytes = slots * LOCAL["SLOT_BYTES"]
            print(f"name=ring_window device_gib={gib} journal_ring_mib={ring_mib} S={s} W={w} root_level={level} "
                  f"window_slots={slots} window_mib={window_bytes / MIB:.1f} share_of_one_device_percent={100 * window_bytes / dev:.2f}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", required=True)
    args = ap.parse_args()
    check_against_source(args.repo)
    region_geometry_table()
    pin_byte_and_capacity_limits()
    ring_window_table()


if __name__ == "__main__":
    main()
