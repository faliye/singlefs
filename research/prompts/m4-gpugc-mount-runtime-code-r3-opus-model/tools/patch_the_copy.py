#!/usr/bin/env python3
"""只在腿的拷贝里加两处，产品判法一个字节不动：
1. 核心层会话加一个只读方法，交回回退门读的那一份根环状态（同 stop_of_the_rollback_gate 那一读）。
2. 运行库用例共用搭建加一个入口：照给的 mkfs 参数建 320 MiB 的小池。
用法：patch_the_copy.py <mounted_session.rs> <common_pool_on_memory_devices/mod.rs>。文件不是预期的样子就退出、不写。
"""
import sys

session_path, fixture_path = sys.argv[1:]
with open(session_path, encoding="utf-8") as source:
    session = source.read()
for expected in (
    "rollback_states_of_the_ring(reader, parameters, current.root())",
    "PoolReaderOfOneReadPass::reading_for_the_first_time(devices)",
):
    if expected not in session:
        sys.exit(f"{session_path}: 没有 {expected}")
if "probe_reading_of_the_rollback_states" in session:
    sys.exit(f"{session_path}: 已经打过")
session += """
impl MountedSession {
    /// 只在攻方腿的拷贝里：(此刻的不同状态数, 走掉下一次与余量之后还剩几个, 最新状态之后的根数)；读不通交回 `None`。同 `stop_of_the_rollback_gate` 那一读。
    pub fn probe_reading_of_the_rollback_states<Device: BlockDevice>(
        &self,
        devices: &[(DeviceIdentity, Device)],
    ) -> Option<(u64, u64, u64)> {
        let parameters = self.parameters_and_device_table.parameters_on_disk();
        self.allocator
            .reading_and_counting_the_other_unit_reads_with_one_copy_failing(
                &PoolReaderOfOneReadPass::reading_for_the_first_time(devices),
                |reader| rollback_states_of_the_ring(reader, parameters, self.current.root()),
            )
            .ok()
            .map(|states| {
                (
                    states.states_now,
                    states.states_after_the_next_publish_and_the_margin,
                    states.publishes_after_the_newest_state,
                )
            })
    }
}
"""
with open(session_path, "w", encoding="utf-8") as target:
    target.write(session)
with open(fixture_path, encoding="utf-8") as source:
    fixture = source.read()
if "fn pool_after_a_sequential_write_on_devices_of(" not in fixture or "DEVICE_BYTES_OF_THE_SMALL_POOL" not in fixture:
    sys.exit(f"{fixture_path}: 不是预期的样子")
fixture += """
/// 只在攻方腿的拷贝里：照给的 mkfs 参数建写得满的小池。
pub fn small_pool_under_for_the_probe(parameters: &MakeFilesystemParameters) -> PoolOnMemoryDevices {
    pool_after_a_sequential_write_on_devices_of(parameters, DEVICE_BYTES_OF_THE_SMALL_POOL)
}
"""
with open(fixture_path, "w", encoding="utf-8") as target:
    target.write(fixture)
