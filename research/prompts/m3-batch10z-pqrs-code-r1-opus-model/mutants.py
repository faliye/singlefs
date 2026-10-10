#!/usr/bin/env python3
"""攻方腿自己的几个变异（不是入库变异表）：mutants.py <仓副本根> <变异名>；原文在文件里恰好命中一次才改，否则退 3。"""
import sys, pathlib
root = pathlib.Path(sys.argv[1]); name = sys.argv[2]
MS = 'crates/singlefs-core/src/mounted_session.rs'
MO = 'crates/singlefs-core/src/mount.rs'
TABLE = {
    # 真发失败那一支不把会话的 current 挪到最后一次落盘的那一版
    'M1-current-not-moved-on-failure': (MS,
        "                    *current = PoolVersion::of_a_version_of_trees(last_published);\n                    return Err(RootRingPushRefused::PublishFailedAfterTheRehearsal(",
        "                    let _ = &last_published;\n                    return Err(RootRingPushRefused::PublishFailedAfterTheRehearsal("),
    # 推环入口收盘表不核（直接用会话持着的参数）
    'M2-devices-not-checked': (MO,
        "    Ok(\n        caller_inputs_agreeing_with_the_disk(caller_parameters, devices, current)?\n            .parameters_of_the_pool,\n    )",
        "    let _ = (devices, current);\n    Ok(caller_parameters.clone())"),
    # 报的挡着的槽数取两块盘全部隔离（不限 C 那一圈罩着的）
    'M7-count-all-isolation': (MS,
        "    let isolated_slots_on_each_device = allocator.isolated_slots_covered_by(&target.placements);",
        "    let isolated_slots_on_each_device: Vec<(DeviceIdentity, u64)> = allocator.devices.iter().map(|map| (map.device, map.isolated_slots())).collect();"),
    # 报的 R 那一槽取 C 那一圈里最先轮到的一槽（次数照最先轮到的算）
    'M12-r-slot-is-the-first': (MS,
        "        .max()?;\n    Some(RootRingPushTarget {",
        "        .min()?;\n    Some(RootRingPushTarget {"),
}
path, old, new = TABLE[name]
p = root / path
s = p.read_text()
n = s.count(old)
if n != 1:
    print(f'{name}: 原文命中 {n} 次', file=sys.stderr); sys.exit(3)
tmp = p.with_suffix(p.suffix + '.tmp')
tmp.write_text(s.replace(old, new))
tmp.replace(p)
print(f'{name}: 改了 {path}')
