use super::*;

#[test]
fn range_that_reads_zero_is_cleared_for_its_own_disk_only_and_forgotten_once_written_over() {
    let mut inner = SparseBlockDevice::new(
        1 << 20,
        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES_LOCAL),
    );
    inner
        .write_at(
            DeviceOffsetInBytes(4096),
            &[0x77; 2048],
            WriteDurability::Plain,
        )
        .expect("内存盘写得进");
    let state = Rc::new(RefCell::new(ReadsZeroState::with_ranges(vec![
        ReadsZeroRange {
            device: DeviceIdentity(0),
            offset: 4608,
            length: 512,
            label: "test",
        },
    ])));
    let mut device = ReadsZeroDevice {
        identity: DeviceIdentity(0),
        inner,
        state: Rc::clone(&state),
    };
    let mut buffer = [0xFFu8; 2048];
    device
        .read_at(DeviceOffsetInBytes(4096), &mut buffer)
        .expect("读得出");
    assert!(
        buffer[..512].iter().all(|byte| *byte == 0x77),
        "区间之前读回真字节"
    );
    assert!(
        buffer[512..1024].iter().all(|byte| *byte == 0),
        "区间里读回全零"
    );
    assert!(
        buffer[1024..].iter().all(|byte| *byte == 0x77),
        "区间之后读回真字节"
    );
    assert_eq!(state.borrow().intercepted_reads, vec![1], "读拦了一次");
    let other_disk = ReadsZeroDevice {
        identity: DeviceIdentity(1),
        inner: SparseBlockDevice::new(
            1 << 20,
            PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES_LOCAL),
        ),
        state: Rc::clone(&state),
    };
    let mut untouched = [0xFFu8; 512];
    other_disk
        .read_at(DeviceOffsetInBytes(4608), &mut untouched)
        .expect("读得出");
    assert_eq!(
        state.borrow().intercepted_reads,
        vec![1],
        "别的盘上同一偏移不拦"
    );
    device
        .write_at(
            DeviceOffsetInBytes(4608),
            &[0x11; 512],
            WriteDurability::Plain,
        )
        .expect("写得进");
    device
        .read_at(DeviceOffsetInBytes(4608), &mut untouched)
        .expect("读得出");
    assert!(
        untouched.iter().all(|byte| *byte == 0x11),
        "写盖过之后读回写下的字节"
    );
}
