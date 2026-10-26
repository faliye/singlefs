"""辩方腿原型：把环洞池的每区槽数 S 改成读环境变量 I718_S（没设取原来的 4）。只在副本里用。用法：mod-slots-from-env.py <common_root_ring_holes/mod.rs>"""
import sys
path = sys.argv[1]
s = open(path, encoding="utf-8").read()
old = """            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS,
            )
            .expect("S = 4 在区间里"),"""
new = """            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                std::env::var("I718_S").ok().and_then(|v| v.parse().ok()).unwrap_or(SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS),
            )
            .expect("S 在区间里"),"""
assert s.count(old) == 1
open(path, "w", encoding="utf-8").write(s.replace(old, new))
print("patched slots from env")
