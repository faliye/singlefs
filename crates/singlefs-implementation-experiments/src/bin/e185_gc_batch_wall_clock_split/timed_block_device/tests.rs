use super::*;

#[test]
fn device_wait_subtracts_the_cpu_spent_inside_the_call() {
    assert_eq!(device_wait_nanoseconds(5_000_000, 2_000_000), 3_000_000);
    assert_eq!(
        device_wait_nanoseconds(10_000, 1_000_000),
        -990_000,
        "纯 CPU 的设备调用里 CPU 读数包着挂钟读数，可以是负的"
    );
    let mut ledger = CallLedger::new(WriteInjection::Nothing);
    ledger.note_the_call(
        DeviceMethod::Write,
        4096,
        Duration::from_millis(5),
        Some(10_000_000),
        Some(12_000_000),
    );
    let totals = ledger.totals_of(DeviceMethod::Write);
    assert_eq!(
        (
            totals.calls,
            totals.bytes,
            totals.cpu_nanoseconds,
            totals.wait_nanoseconds
        ),
        (1, 4096, 2_000_000, 3_000_000)
    );
    ledger.note_the_call(
        DeviceMethod::Read,
        512,
        Duration::from_millis(1),
        None,
        Some(1),
    );
    assert!(
        ledger.is_the_cpu_unreadable_in_this_window,
        "读不到线程 CPU 时钟记下来，那一轮作废"
    );
}

#[test]
fn timed_device_records_reads_before_the_first_write_and_the_injected_sleep() {
    let ledger = Rc::new(RefCell::new(CallLedger::new(
        WriteInjection::SleepTwoMilliseconds,
    )));
    let mut devices = timed_devices(
        vec![(
            DeviceIdentity(0),
            SparseBlockDevice::new(8 << 20, PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES)),
        )],
        &ledger,
    );
    let device = &mut devices[0].1;
    let mut buffer = vec![0_u8; 32_768];
    device
        .read_at(DeviceOffsetInBytes(0), &mut buffer)
        .expect("读");
    device
        .write_at(
            DeviceOffsetInBytes(32_768),
            &[1_u8; 4096],
            WriteDurability::Plain,
        )
        .expect("写");
    device
        .read_at(DeviceOffsetInBytes(32_768), &mut buffer[..4096])
        .expect("读");
    let window = ledger.borrow();
    assert_eq!(
        window
            .reads
            .iter()
            .map(|read| read.is_before_the_first_write)
            .collect::<Vec<_>>(),
        vec![true, false],
        "第一次写之前那一读算扫描读"
    );
    let write_totals = window.totals_of(DeviceMethod::Write);
    assert_eq!((write_totals.calls, write_totals.bytes), (1, 4096));
    assert!(
        write_totals.wall_nanoseconds >= 2_000_000,
        "写里多睡的 2 ms 计进挂钟"
    );
    assert!(window
        .chunks_written_since_the_copy
        .contains(&(DeviceIdentity(0), 0)));
}
