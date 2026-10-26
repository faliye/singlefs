#!/usr/bin/env python3
"""m3-close-c506-r3 攻方腿 X3：第一轮「S = 16、槽距 512 KiB 暖机 24 / 24 判 I-7.17 红、对照 96 / 96 干净」换一条独立的路复核。

不调 crates：只按条款的落点公式在两块盘上摆根，逐状态照 I-7.17 的字面判。
- 区域 r 的起点 = 1 MiB + r × 3 MiB，归属 0 / 1 / 0（D22 已定项 16 第 1、4 句）；槽 j 在 起点 + j × 槽距（第 2 句）。
- mkfs：三个区域整段写 0，再在每个区域槽 0 种第 0 代根（已定项 8 第 3 条）。
- 第 n 次发布写区域 n mod 3 的槽 (n div 3) mod S（已定项 16）。
- I-7.17：读者按每个 (区域, 槽) 的偏移去读，读到的根 txg ≥ 1 时要 r == txg mod 3 且 s == (txg div 3) mod S；txg == 0 只判 s == 0。
"""
import sys

MIB = 1 << 20
REGION_DEVICE = (0, 1, 0)


def region_start(region):
    return MIB + region * 3 * MIB


def run_arm(slots, spacing, publishes):
    disks = {0: {}, 1: {}}
    for region, device in enumerate(REGION_DEVICE):
        start = region_start(region)
        for slot in range(slots):
            disks[device].pop(start + slot * spacing, None)
    for region, device in enumerate(REGION_DEVICE):
        disks[device][region_start(region)] = 0
    aliases = []
    seen = {}
    for region, device in enumerate(REGION_DEVICE):
        for slot in range(slots):
            key = (device, region_start(region) + slot * spacing)
            if key in seen:
                aliases.append(f"d{device}@{key[1]}:r{seen[key][0]}s{seen[key][1]}=r{region}s{slot}")
            seen[key] = (region, slot)
    red_states = 0
    states = 0
    first_red_txg = None
    for txg in range(0, publishes + 1):
        if txg >= 1:
            region = txg % 3
            slot = (txg // 3) % slots
            disks[REGION_DEVICE[region]][region_start(region) + slot * spacing] = txg
        states += 1
        red = False
        for region, device in enumerate(REGION_DEVICE):
            for slot in range(slots):
                root = disks[device].get(region_start(region) + slot * spacing)
                if root is None:
                    continue
                if root == 0:
                    ok = slot == 0
                else:
                    ok = region == root % 3 and slot == (root // 3) % slots
                if not ok:
                    red = True
        if red:
            red_states += 1
            if first_red_txg is None:
                first_red_txg = txg
    return states, red_states, first_red_txg, aliases


def main():
    publishes = int(sys.argv[1]) if len(sys.argv) > 1 else 200
    arms = [("aliased_S16_spacing512KiB", 16, 512 * 1024),
            ("control_S8_spacing512KiB", 8, 512 * 1024),
            ("control_S16_spacing256KiB", 16, 256 * 1024),
            ("control_S8_spacing4KiB", 8, 4096),
            ("control_S16_spacing4KiB", 16, 4096)]
    for name, slots, spacing in arms:
        states, red, first, aliases = run_arm(slots, spacing, publishes)
        print(f"name=i717_arm arm={name} S={slots} spacing={spacing} states={states} red_states={red} "
              f"first_red_txg={first} aliased_slot_pairs={aliases}")


if __name__ == "__main__":
    main()
