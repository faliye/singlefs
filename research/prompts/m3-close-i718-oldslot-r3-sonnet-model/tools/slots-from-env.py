#!/usr/bin/env python3
"""副本里把环洞池的每区槽数改成读环境变量 I718_S（没设取原来的 4）。只在三方腿的副本里用，不动主工作区。
用法：slots-from-env.py <副本里的 tests/common_root_ring_holes/mod.rs>"""
import sys

path = sys.argv[1]
text = open(path, encoding="utf-8").read()
old = """            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS,
            )"""
new = """            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                std::env::var("I718_S").ok().and_then(|v| v.parse().ok()).unwrap_or(SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS),
            )"""
if text.count(old) != 1:
    sys.exit("找不到唯一的一处要改的写法")
open(path, "w", encoding="utf-8").write(text.replace(old, new))
