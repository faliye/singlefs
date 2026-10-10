#!/usr/bin/env python3
"""腿自己的 N2 实验：在草稿副本的 core 里加两样（不进仓）：
 ① 记根回收出东西时 eprintln RECLAIM_AT_RECORD thread=<测试名> txg=<> n=<>；
 ② 环境变量 SIM_PRE31 设了时还原第 31 批之前的行为：测试 / 装置 / on_device_modes 里调 install_root_ring_occupancy 不装，记根没装表时静默返回。
用法：instrument_core.py <草稿副本根>"""
import sys

path = sys.argv[1] + "/crates/singlefs-core/src/allocator.rs"
source = open(path, encoding="utf-8").read()
pairs = [
    ('''    pub fn install_root_ring_occupancy(&mut self, occupancy: RootRingOccupancy) {
        self.root_ring = Some(occupancy);''',
     '''    #[track_caller]
    pub fn install_root_ring_occupancy(&mut self, occupancy: RootRingOccupancy) {
        if std::env::var_os("SIM_PRE31").is_some() {
            let file = std::panic::Location::caller().file();
            if file.contains("/tests/") || file.contains("/bin/") || file.contains("on_device_modes") {
                return;
            }
        }
        self.root_ring = Some(occupancy);'''),
    ('''        let occupancy = self.root_ring.as_mut().expect(''',
     '''        if self.root_ring.is_none() && std::env::var_os("SIM_PRE31").is_some() {
            return;
        }
        let occupancy = self.root_ring.as_mut().expect('''),
    ('''        let reclaimed =
            self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);
        self.isolate_reclaimed_slots_an_abandoned_root_still_references(''',
     '''        let reclaimed =
            self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);
        if !reclaimed.is_empty() {
            eprintln!("RECLAIM_AT_RECORD thread={:?} txg={} n={}", std::thread::current().name(), checkpoint_txg.0, reclaimed.len());
        }
        self.isolate_reclaimed_slots_an_abandoned_root_still_references('''),
]
for old, new in pairs:
    assert source.count(old) == 1, old
    source = source.replace(old, new)
open(path, "w", encoding="utf-8").write(source)
