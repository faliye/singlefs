#!/usr/bin/env python3
"""m3-close-c506-r3 攻方腿：X3 前两轮报的数逐个换算式复核，X2 按 D2 已定项 19 取槽距时 mkfs 写得出哪些 S。

纯算术，不读仓、不调 crates：每个式子在注释里写出它照的是哪一句条款或哪一行代码（行号见报告）。
"""
MIB = 1 << 20
PRIME_STEP = 3                      # 被判形态钉的第一版常量 P
CHUNK_BYTES = MIB                   # 被判形态钉的第一版常量 chunk
REGION_STRIDE = PRIME_STEP * CHUNK_BYTES
ROOT_RING_BASE = MIB                # 根环起点 64 × 16384
JOURNAL_START = 1024 * 16384        # journal 环起点 1024 × 16384
JOURNAL_RING_BYTES = 768 * MIB      # journal 环长默认 768 MiB（layout 一）
SYSCONF_SLOT = 4096
SPACING_MAX = ROOT_RING_BASE - SYSCONF_SLOT   # D22 已定项 2 射程：槽距 ∈ [4096, 1 MiB − 4096]
S_MIN, S_MAX = 4, 16
REGION_DEVICE = (0, 1, 0)


def spacings(step):
    return range(4096, SPACING_MAX + 1, step)


def region_start(region):
    return ROOT_RING_BASE + region * REGION_STRIDE


def overlap(a, b):
    return a[0] < b[1] and b[0] < a[1]


def extents_per_device(slots, spacing, region_bytes):
    """0 / 1 / 0 归属下每块盘上真落的固定结构：系统配置两槽、本盘的根环区域、journal 环起点之后。"""
    per = {0: [], 1: []}
    for device in per:
        per[device].append((0, SYSCONF_SLOT))
        per[device].append((spacing, spacing + SYSCONF_SLOT))
        per[device].append((JOURNAL_START, JOURNAL_START + JOURNAL_RING_BYTES))
    for region, device in enumerate(REGION_DEVICE):
        start = region_start(region)
        per[device].append((start, start + region_bytes))
    return per


def any_overlap(per):
    for items in per.values():
        for i, a in enumerate(items):
            for b in items[i + 1:]:
                if overlap(a, b):
                    return True
    return False


def root_slots_collide(slots, spacing, pbs):
    per = {0: [], 1: []}
    for device in per:
        per[device].append((0, SYSCONF_SLOT))
        per[device].append((spacing, spacing + SYSCONF_SLOT))
        per[device].append((JOURNAL_START, JOURNAL_START + JOURNAL_RING_BYTES))
    for region, device in enumerate(REGION_DEVICE):
        for slot in range(slots):
            start = region_start(region) + slot * spacing
            per[device].append((start, start + pbs))
    return any_overlap(per)


def make_filesystem_today_accepts(slots, spacing, pbs):
    """照 make_filesystem.rs 的 fixed_structure_extents 与 check_geometry：六段不分盘两两比（区域长 = S × 槽距），根槽宽 ≤ 槽距。"""
    if pbs > spacing:
        return False
    segments = [(0, SYSCONF_SLOT), (spacing, spacing + SYSCONF_SLOT)]
    segments += [(region_start(r), region_start(r) + slots * spacing) for r in range(3)]
    segments.append((JOURNAL_START, JOURNAL_START + JOURNAL_RING_BYTES))
    for i, a in enumerate(segments):
        for b in segments[i + 1:]:
            if overlap(a, b):
                return False
    return True


def written_slots_per_region(spacing):
    return min(8, REGION_STRIDE // spacing)


def main():
    grand = 0
    for pbs in (512, 4096):
        shapes = [(s, sp) for s in range(S_MIN, S_MAX + 1) for sp in spacings(pbs)]
        grand += len(shapes)
        rule = {x for x in shapes if x[0] * x[1] <= REGION_STRIDE}
        mkfs = {x for x in shapes if make_filesystem_today_accepts(x[0], x[1], pbs)}
        full_clean = {x for x in shapes if not any_overlap(extents_per_device(x[0], x[1], x[0] * x[1]))}
        byte_clean = {x for x in shapes
                      if not any_overlap(extents_per_device(x[0], x[1], (x[0] - 1) * x[1] + pbs))}
        slot_clean = {x for x in shapes if not root_slots_collide(x[0], x[1], pbs)}
        print(f"name=shapes pbs={pbs} spacings={len(list(spacings(pbs)))} shapes={len(shapes)} "
              f"rule_accepts={len(rule)} mkfs_today_accepts={len(mkfs)} rule_minus_mkfs={len(rule - mkfs)} "
              f"mkfs_minus_rule={len(mkfs - rule)} extent_clean_full={len(full_clean)} "
              f"extent_clean_byte_exact={len(byte_clean)} root_slot_clean={len(slot_clean)} "
              f"rule_refuses_and_extent_clean_full={len(full_clean - rule)} "
              f"rule_refuses_and_root_slot_clean={len(slot_clean - rule)} rule_accepts_and_root_slot_collides={len(rule - slot_clean)}")
        bands = {}
        for sp in spacings(pbs):
            bands.setdefault(written_slots_per_region(sp), []).append(sp)
        for value in sorted(bands):
            sps = bands[value]
            print(f"name=min_formula pbs={pbs} written_S={value} spacings={len(sps)} from={sps[0]} to={sps[-1]}")
    print(f"name=shapes_total grand_total={grand}")
    largest_spacing_with_s4 = max(sp for sp in spacings(512) if S_MIN * sp <= REGION_STRIDE)
    print(f"name=effective_spacing_upper_bound bytes={largest_spacing_with_s4} kib={largest_spacing_with_s4 // 1024}")
    print(f"name=pinned_slot_byte largest_S={max(s for s in range(1, 256) if 3 * s - 1 <= 254)} "
          f"value_at_85={3 * 85 - 1} value_at_86={3 * 86 - 1}")
    # D2 已定项 19：槽距 = 4096 向上取整到 io_min 的整数倍；I-7.5 判定宽度 = max(pbs, io_min)（system_configuration.rs）
    outcomes = {}
    for io_min in range(512, 2 * MIB + 1, 512):
        spacing = -(-4096 // io_min) * io_min
        width = max(512, io_min)
        offsets = [0, spacing] + [region_start(r) for r in range(3)]
        aligned = all(o % width == 0 for o in offsets)
        in_range = spacing <= SPACING_MAX
        value = written_slots_per_region(spacing)
        if not aligned:
            key = "refused_by_alignment"
        elif not in_range:
            key = "refused_by_spacing_range"
        elif value < 5:
            key = "refused_only_by_below_five"
        else:
            key = f"writes_S={value}"
        outcomes.setdefault(key, []).append(io_min)
    for key in sorted(outcomes):
        values = outcomes[key]
        shown = values if len(values) <= 12 else values[:6] + ["..."] + values[-3:]
        print(f"name=d2_item19_io_min outcome={key} io_min_count={len(values)} io_min={shown}")


if __name__ == "__main__":
    main()
