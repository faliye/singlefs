#!/usr/bin/env python3
"""把运行库用例共用搭建（冻结副本的拷贝里那一份）的根环每区槽数改成探针能摆的：parameters() 读一个静态量，默认 8。只改交进来的那一份文件。"""
import sys

path = sys.argv[1]
with open(path, encoding="utf-8") as source:
    text = source.read()
old = "RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM"
if text.count(old) != 1:
    sys.exit(f"{path}: {old} 不是恰好一处")
text = text.replace(old, "root_ring_slots_of_the_probe()")
text += """
/// 探针摆的根环每区槽数（5–8）。
static ROOT_RING_SLOTS_OF_THE_PROBE: AtomicU64 = AtomicU64::new(8);

pub fn set_root_ring_slots_of_the_probe(slots_per_region: u64) {
    ROOT_RING_SLOTS_OF_THE_PROBE.store(slots_per_region, Ordering::SeqCst);
}

fn root_ring_slots_of_the_probe() -> RootRingSlotsPerRegion {
    RootRingSlotsPerRegion::from_system_configuration_field(
        ROOT_RING_SLOTS_OF_THE_PROBE.load(Ordering::SeqCst),
    )
    .expect("探针摆的槽数在区间里")
}
"""
with open(path, "w", encoding="utf-8") as target:
    target.write(text)
