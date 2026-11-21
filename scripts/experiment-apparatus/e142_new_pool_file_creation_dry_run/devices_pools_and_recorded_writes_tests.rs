use super::*;

/// 臂 N19M 的 ⑫（登记 R46，R19E-4 第 5 条的合成取样点）：同一块盘上连着的屏障并成一道；
/// 中间有任何一块盘上的写就照记——盘 1 的一次写同样让盘 0 下一道屏障照记（变异 M203 只在这一格变）。
#[test]
fn consecutive_barriers_on_one_device_merge_and_any_write_in_between_keeps_them_apart() {
    let steps = |pool: &RecordingPool| -> Vec<Option<u32>> {
        pool.operations
            .iter()
            .map(|operation| match operation {
                RecordedOperation::Barrier(device) => Some(device.0),
                RecordedOperation::Write(_) => None,
            })
            .collect()
    };
    let empty_pool = |device_count: usize| RecordingPool { pool: Pool { devices: vec![SparseDevice::default(); device_count], device_bytes: DEVICE_BYTES }, operations: Vec::new() };

    let mut back_to_back = empty_pool(2);
    back_to_back.barrier();
    back_to_back.barrier();
    assert_eq!(steps(&back_to_back), vec![Some(0), Some(1)], "两道池屏障之间没有写：第二道两块盘都并掉，只剩两步");

    let mut write_on_device_one_in_between = empty_pool(2);
    write_on_device_one_in_between.barrier();
    write_on_device_one_in_between.write(DeviceIdentity(1), DeviceOffset(0), &[1u8; 4096], StepKind::SystemConfigurationSlot);
    write_on_device_one_in_between.barrier();
    assert_eq!(
        steps(&write_on_device_one_in_between),
        vec![Some(0), Some(1), None, Some(0), Some(1)],
        "中间有盘 1 的一次写：盘 0 那一步也照记（「任何一次写」取任何一块盘上的写），共四步屏障"
    );

    let mut one_device = empty_pool(1);
    one_device.barrier();
    one_device.barrier();
    assert_eq!(steps(&one_device), vec![Some(0)], "一块盘上连着两道：只记一步");
}

/// 臂 N19 的 ①：一道池屏障按设备号升序每块盘记一步，一块盘时恰一步。
#[test]
fn pool_barrier_records_one_step_per_device_in_ascending_order() {
    for (device_count, expected) in [(2usize, vec![DeviceIdentity(0), DeviceIdentity(1)]), (1, vec![DeviceIdentity(0)])] {
        let mut pool = RecordingPool { pool: Pool { devices: vec![SparseDevice::default(); device_count], device_bytes: DEVICE_BYTES }, operations: Vec::new() };
        pool.barrier();
        let recorded: Vec<DeviceIdentity> = pool
            .operations
            .iter()
            .map(|operation| match operation {
                RecordedOperation::Barrier(device) => *device,
                RecordedOperation::Write(_) => panic!("屏障只记屏障"),
            })
            .collect();
        assert_eq!(recorded, expected);
    }
}
