use super::*;

// ------------------------------------------------------------------------
// 第 2 次跑（`research/prompts/e158-r2-prereg.md`）那一节的单测。
// ------------------------------------------------------------------------

fn device_with_bytes(identity: DeviceIdentity, offset: u64, bytes: &[u8]) -> SparseBlockDevice {
    let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
    device
        .write_at(DeviceOffsetInBytes(offset), bytes, WriteDurability::Plain)
        .expect("内存盘写不报错");
    let _ = identity;
    device
}

fn instrumented(
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    state: &Rc<RefCell<InstrumentationState>>,
) -> InstrumentedDevice {
    InstrumentedDevice {
        identity,
        inner,
        state: Rc::clone(state),
    }
}

/// 读落点：读错那一类读就报错、清零那一类读回 0，两类都按区间罩（读的起点不必等于落点起点），落点外的读原样；
/// 每个落点被拦下几次单独记；这次调用自己写到落点上之后不再拦。
#[test]
fn instrumented_device_fails_or_zeroes_only_reads_over_the_named_ranges_and_counts_them() {
    let identity = DeviceIdentity(0);
    let inner = device_with_bytes(identity, 8192, &[7u8; 4096]);
    let state = Rc::new(RefCell::new(InstrumentationState::new(
        vec![
            UnreadableRange {
                device: identity,
                offset: 8192,
                length: 512,
                form: UnreadableForm::ReadFails,
                label: "fails".to_string(),
            },
            UnreadableRange {
                device: identity,
                offset: 9216,
                length: 512,
                form: UnreadableForm::ReadsZeros,
                label: "zeros".to_string(),
            },
        ],
        None,
        None,
    )));
    let mut device = instrumented(identity, inner, &state);
    let mut buffer = vec![0u8; 512];
    assert!(
        device
            .read_at(DeviceOffsetInBytes(7936), &mut buffer)
            .is_err(),
        "起点在落点之前、区间罩到落点的读也要报错"
    );
    let mut wide = vec![0u8; 2048];
    assert!(device.read_at(DeviceOffsetInBytes(8704), &mut wide).is_ok());
    assert!(wide[..512].iter().all(|byte| *byte == 7));
    assert!(
        wide[512..1024].iter().all(|byte| *byte == 0),
        "清零那一段读回 0"
    );
    assert!(wide[1024..1536].iter().all(|byte| *byte == 7));
    let mut outside = vec![0u8; 512];
    assert!(device
        .read_at(DeviceOffsetInBytes(12288), &mut outside)
        .is_ok());
    assert_eq!(state.borrow().intercepted_reads, vec![1, 1]);
    device
        .write_at(
            DeviceOffsetInBytes(8192),
            &[9u8; 512],
            WriteDurability::Plain,
        )
        .expect("写");
    assert!(
        device
            .read_at(DeviceOffsetInBytes(8192), &mut buffer)
            .is_ok(),
        "写过之后不再拦"
    );
    assert!(buffer.iter().all(|byte| *byte == 9));
    assert_eq!(state.borrow().intercepted_reads, vec![1, 1]);
}

/// 注入写失败：整池第 n 次写（从 0 数）报错、不落盘、不进落了的写的记录；第 n 次屏障同理。
#[test]
fn instrumented_device_fails_exactly_the_named_write_and_barrier_calls() {
    let identity = DeviceIdentity(0);
    let state = Rc::new(RefCell::new(InstrumentationState::new(
        Vec::new(),
        Some(1),
        Some(0),
    )));
    let mut device = instrumented(
        identity,
        SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
        &state,
    );
    assert!(device
        .write_at(DeviceOffsetInBytes(0), &[1u8; 512], WriteDurability::Plain)
        .is_ok());
    assert!(device
        .write_at(
            DeviceOffsetInBytes(512),
            &[2u8; 512],
            WriteDurability::Plain
        )
        .is_err());
    assert!(device
        .write_at(
            DeviceOffsetInBytes(1024),
            &[3u8; 512],
            WriteDurability::Plain
        )
        .is_ok());
    assert!(device.barrier().is_err());
    assert!(device.barrier().is_ok());
    let state = state.borrow();
    assert_eq!(state.write_calls, 3);
    assert_eq!(state.barrier_calls, 2);
    assert!(state.injected_failure_fired);
    let written_offsets: Vec<u64> = state
        .applied
        .iter()
        .filter_map(|step| match step {
            AppliedDeviceStep::Write { offset, .. } => Some(*offset),
            AppliedDeviceStep::ZeroFill { .. } | AppliedDeviceStep::Barrier { .. } => None,
        })
        .collect();
    assert_eq!(written_offsets, vec![0, 1024]);
    let mut buffer = vec![0u8; 512];
    device
        .inner
        .read_at(DeviceOffsetInBytes(512), &mut buffer)
        .expect("读");
    assert!(buffer.iter().all(|byte| *byte == 0), "报了错的那次写不落盘");
}

fn write_step(device: u32, offset: u64, fill: u8) -> AppliedDeviceStep {
    AppliedDeviceStep::Write {
        device: DeviceIdentity(device),
        offset,
        bytes: vec![fill; 512],
    }
}

/// 崩溃状态（前缀）：只施加前 n 个写，屏障不算写、也不改字节。
#[test]
fn image_with_the_first_writes_applies_only_the_prefix() {
    let base = memory_pool_of(&new_devices(IMAGE_BYTES), IMAGE_BYTES);
    let steps = vec![
        write_step(0, 0, 1),
        AppliedDeviceStep::Barrier {
            device: DeviceIdentity(0),
        },
        write_step(1, 512, 2),
        write_step(0, 1024, 3),
    ];
    let image = image_with_the_first_writes(&base, &steps, 2);
    assert_eq!(image.read(0, 0, 512), Some(vec![1; 512]));
    assert_eq!(image.read(1, 512, 512), Some(vec![2; 512]));
    assert_eq!(
        image.read(0, 1024, 512),
        Some(vec![0; 512]),
        "第 3 个写没施加"
    );
    let empty = image_with_the_first_writes(&base, &steps, 0);
    assert_eq!(empty.read(0, 0, 512), Some(vec![0; 512]));
}

/// 第一个根槽写的下标按写数（屏障不算），它之前的屏障数只数在它之前发的。
#[test]
fn index_of_the_first_root_slot_write_and_the_barriers_before_it() {
    let steps = vec![
        write_step(0, 4096, 1),
        AppliedDeviceStep::Barrier {
            device: DeviceIdentity(0),
        },
        write_step(1, 900_000, 2),
        AppliedDeviceStep::Barrier {
            device: DeviceIdentity(1),
        },
        write_step(1, 4_198_400, 3),
        AppliedDeviceStep::Barrier {
            device: DeviceIdentity(0),
        },
        write_step(0, 4_198_400, 4),
    ];
    let root_slots = vec![(DeviceIdentity(1), 4_198_400u64)];
    assert_eq!(
        index_of_the_first_root_slot_write(&steps, &root_slots, 512),
        Some(2)
    );
    assert_eq!(barriers_before_write(&steps, 3), 2);
    assert_eq!(barriers_before_write(&steps, 1), 0);
    let barrier_first = vec![
        AppliedDeviceStep::Barrier {
            device: DeviceIdentity(0),
        },
        write_step(0, 4096, 1),
    ];
    assert_eq!(
        barriers_before_write(&barrier_first, 0),
        0,
        "「前 0 个写」之前一个屏障都不算"
    );
    assert_eq!(
        barriers_before_write(&barrier_first, 1),
        1,
        "第 1 个写之前那一次屏障算"
    );
    assert_eq!(
        index_of_the_first_root_slot_write(&steps, &[(DeviceIdentity(0), 12)], 512),
        None
    );
}

/// K0–K4（前提 3 的答案）：报错的按这次调用设备一层写没写分 K3 / K4，不按错误成员分（登记第九节 M17）；做成的按准入那一栏分。
#[test]
fn mount_outcome_class_splits_refusals_by_the_writes_of_the_call() {
    let refused: Result<Mounted, singlefs_core::mount::MountError> =
        Err(singlefs_core::mount::MountError::InstanceTableMalformed);
    assert_eq!(mount_outcome_class(&refused, 0), "K3");
    assert_eq!(mount_outcome_class(&refused, 1), "K4");
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let mut devices = new_devices(IMAGE_BYTES);
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mounted = mount_writable(&parameters, &mut devices);
    assert_eq!(mount_outcome_class(&mounted, 30), "K0");
}
