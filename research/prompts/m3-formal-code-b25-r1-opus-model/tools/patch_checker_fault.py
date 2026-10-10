#!/usr/bin/env python3
"""攻方副本里给池级 checker（O2）注入一个只在环境变量下生效的 panic：OPUS_O2_FAULT_SLOT=<槽号> 时，
盘 0 的候选单元槽里有这一槽、盘 1 的没有（那个单元的两盘镜像写只落了盘 0 那一份），check_pool_image 当场 panic。
不设环境变量时行为与原文逐字节相同的路径一致（只多一次读环境变量）。用法：patch_checker_fault.py <副本根>"""
import sys, pathlib
p = pathlib.Path(sys.argv[1]) / "crates/singlefs-checker/src/walk.rs"
s = p.read_text()
a = "pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {\n"
b = a + """    if let Some(slot) = std::env::var("OPUS_O2_FAULT_SLOT").ok().and_then(|value| value.parse::<u64>().ok()) {
        let landed_on = |device: u32| reader.candidate_unit_slots(device).is_some_and(|slots| slots.contains(&slot));
        assert!(!(landed_on(0) && !landed_on(1)), "攻方注入：O2 在槽 {slot} 的镜像对只落了盘 0 那一份时 panic");
    }
"""
assert s.count(a) == 1; s = s.replace(a, b)
tmp = pathlib.Path(str(p) + ".tmp"); tmp.write_text(s); tmp.replace(p)
print("patched")
