//! 会话已走过被拒那一版、盘表次序对调或多一块少一块时，要出路与推环在预演之前拒、一个盘都不读。

use super::*;

/// 要出路时会话已不在报「被宽护卡住」时被拒的那一版上（之后推过环）：按哪一刻算条款没写、第一版不支持，拒成点名这一形的那一种、
/// 带被拒那一版与现行那一版的实例代号与 txg；不预演：一块只进内存的盘都没建、一个盘都没读、一个改盘调用都没有，两块盘逐字节不变。
pub fn run_the_scenario_where_asking_for_the_ways_out_after_the_session_moved_past_the_version_refused_on_is_refused_without_reading_a_device(
    images: &mut PoolCheckerImages,
) {
    let PoolStuckBehindTheWideProtection {
        mut pool, stuck, ..
    } = pool_stuck_behind_the_wide_protection(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
            overwrites_before: 40,
        },
    );
    let refused_on = *pool.session().current.root();
    push_the_root_ring_of(&mut pool).unwrap_or_else(|refusal| panic!("推环做得成：{refusal:?}"));
    let (counting_pool, calls_of_the_pool) =
        pool_failing_one_changing_call_copied_from(&pool, None);
    let current_after_the_push = *counting_pool.session().current.root();
    assert!(
        current_after_the_push.checkpoint_txg > refused_on.checkpoint_txg,
        "推环之后会话已在后面的一版上（阳性对照）"
    );
    let image_before = counting_pool.image();
    match counting_pool
        .session()
        .ways_out_of_the_wide_protection(&counting_pool.devices, &stuck)
    {
        Err(
            WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion {
                instance_refused_on,
                checkpoint_txg_refused_on,
                instance_of_the_current_version,
                checkpoint_txg_of_the_current_version,
            },
        ) => assert_eq!(
            (
                instance_refused_on,
                checkpoint_txg_refused_on,
                instance_of_the_current_version,
                checkpoint_txg_of_the_current_version
            ),
            (
                refused_on.instance,
                refused_on.checkpoint_txg,
                current_after_the_push.instance,
                current_after_the_push.checkpoint_txg
            ),
            "带被拒那一版与现行那一版的实例代号与 txg"
        ),
        other => panic!("会话已不在被拒的那一版上：不预演，而交回 {other:?}"),
    }
    assert_eq!(
        (
            calls_of_the_pool.probes_of_the_physical_block_size.get(),
            calls_of_the_pool.reads.get(),
            calls_of_the_pool.calls.get()
        ),
        (0, 0, 0),
        "不预演：一块只进内存的盘都没建、一个盘都没读、一个改盘调用都没有"
    );
    assert!(counting_pool.image() == image_before, "两块盘逐字节不变");
}

/// 盘表怎么换（要出路时交进去的那一份）。
#[derive(Clone, Copy, Debug)]
pub enum DeviceTableChange {
    /// 两块盘次序对调（身份与盘不拆开）：攻方 `research/prompts/m3-batch10z-vw-code-r1-opus-model/atk_vw_part3.rs` 的 `table-reversed`。
    Reversed,
    /// 多交一块本池之外的空盘。
    OneDeviceMore,
    /// 少交最后一块盘。
    OneDeviceFewer,
}

/// 池里交进去的盘表的设备身份，按交进去的次序。
pub fn device_identities_in_the_order_handed_in(
    pool: &PoolUnderTest<DeviceFailingOneChangingCall>,
) -> Vec<DeviceIdentity> {
    pool.devices.iter().map(|(identity, _)| *identity).collect()
}

/// 照 `table_change` 换池里交进去的盘表（多交的那一块是本池之外的空盘，改盘调用一样数进 `calls_of_the_pool`），交回少交的那一块
/// （别的换法交 `None`）；换完的盘表与挂载时的不同（阳性对照，当场判）。
pub fn change_the_device_table_of(
    pool: &mut PoolUnderTest<DeviceFailingOneChangingCall>,
    table_change: DeviceTableChange,
    calls_of_the_pool: &Rc<ChangingCallsOfThePool>,
) -> Option<(DeviceIdentity, DeviceFailingOneChangingCall)> {
    let device_table_of_the_mount = device_identities_in_the_order_handed_in(pool);
    let mut device_left_out = None;
    match table_change {
        DeviceTableChange::Reversed => pool.devices.reverse(),
        DeviceTableChange::OneDeviceMore => pool.devices.push((
            DeviceIdentity(u32::try_from(device_table_of_the_mount.len()).expect("几块盘")),
            DeviceFailingOneChangingCall {
                inner: SparseBlockDevice::new(pool.device_bytes, PhysicalBlockSizeInBytes(512)),
                calls_of_the_pool: Rc::clone(calls_of_the_pool),
            },
        )),
        DeviceTableChange::OneDeviceFewer => device_left_out = pool.devices.pop(),
    }
    assert_ne!(
        device_identities_in_the_order_handed_in(pool),
        device_table_of_the_mount,
        "{table_change:?}：交进去的盘表与挂载时的不同（阳性对照）"
    );
    device_left_out
}

/// [`change_the_device_table_of`] 换回去：盘表回到挂载时那一份。
pub fn restore_the_device_table_of(
    pool: &mut PoolUnderTest<DeviceFailingOneChangingCall>,
    table_change: DeviceTableChange,
    device_left_out: Option<(DeviceIdentity, DeviceFailingOneChangingCall)>,
) {
    match table_change {
        DeviceTableChange::Reversed => pool.devices.reverse(),
        DeviceTableChange::OneDeviceMore => {
            pool.devices.pop();
        }
        DeviceTableChange::OneDeviceFewer => pool
            .devices
            .push(device_left_out.expect("少交的那一块留着")),
    }
}

/// 被宽护卡住（建 inode、前缀 40 次那一形）之后，在这一刻盘面与会话的一份拷贝上照 `table_change` 换盘表要出路：在任何读之前拒成
/// 盘表那一种、带两份盘表，一块只进内存的盘都没建、一个盘都没读、一个改盘调用都没有，盘逐字节不变；换回挂载时那一份盘表照常预演
/// （同一个会话原样照旧）。不核盘表次序时，次序对调的盘表在预演里推环那一串照样过（核盘表不看次序）、列推环，照出路二推完再发却被拒成盘表那一种
/// （攻方「打中二」）；多一块、少一块在预演里推环被拒，但已经读过盘。
pub fn assert_the_ways_out_with_the_device_table_changed_are_refused_without_reading_a_device(
    images: &mut PoolCheckerImages,
    table_change: DeviceTableChange,
) {
    let PoolStuckBehindTheWideProtection { pool, stuck, .. } =
        pool_stuck_behind_the_wide_protection(
            images,
            FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
                overwrites_before: 40,
            },
        );
    let (mut counting_pool, calls_of_the_pool) =
        pool_failing_one_changing_call_copied_from(&pool, None);
    let device_table_of_the_mount = device_identities_in_the_order_handed_in(&counting_pool);
    let device_left_out =
        change_the_device_table_of(&mut counting_pool, table_change, &calls_of_the_pool);
    let device_table_handed_in = device_identities_in_the_order_handed_in(&counting_pool);
    let image_before = counting_pool.image();
    match counting_pool
        .session()
        .ways_out_of_the_wide_protection(&counting_pool.devices, &stuck)
    {
        Err(WaysOutOfTheWideProtectionRefused::DeviceTableOtherThanTheOneOfTheMount {
            device_table_of_the_mount: refused_table_of_the_mount,
            device_table_handed_in: refused_table_handed_in,
        }) => assert_eq!(
            (refused_table_of_the_mount, refused_table_handed_in),
            (device_table_of_the_mount.clone(), device_table_handed_in),
            "{table_change:?}：带挂载时那一份盘表与交进去的那一份"
        ),
        other => panic!(
            "{table_change:?}：盘表与挂载时的不同：在预演之前拒成盘表那一种，而交回 {other:?}"
        ),
    }
    assert_eq!(
        (
            calls_of_the_pool.probes_of_the_physical_block_size.get(),
            calls_of_the_pool.reads.get(),
            calls_of_the_pool.calls.get()
        ),
        (0, 0, 0),
        "{table_change:?}：不预演：一块只进内存的盘都没建、一个盘都没读、一个改盘调用都没有"
    );
    assert!(
        counting_pool.image() == image_before,
        "{table_change:?}：盘逐字节不变"
    );
    restore_the_device_table_of(&mut counting_pool, table_change, device_left_out);
    ways_out_of(
        &counting_pool,
        &stuck,
        &format!("{table_change:?} 之后换回挂载时那一份盘表"),
    );
}

pub fn run_the_scenario_where_asking_for_the_ways_out_with_the_two_devices_in_reversed_order_is_refused_without_reading_a_device(
    images: &mut PoolCheckerImages,
) {
    assert_the_ways_out_with_the_device_table_changed_are_refused_without_reading_a_device(
        images,
        DeviceTableChange::Reversed,
    );
}

pub fn run_the_scenario_where_asking_for_the_ways_out_with_one_device_more_than_the_mount_is_refused_without_reading_a_device(
    images: &mut PoolCheckerImages,
) {
    assert_the_ways_out_with_the_device_table_changed_are_refused_without_reading_a_device(
        images,
        DeviceTableChange::OneDeviceMore,
    );
}

pub fn run_the_scenario_where_asking_for_the_ways_out_with_one_device_fewer_than_the_mount_is_refused_without_reading_a_device(
    images: &mut PoolCheckerImages,
) {
    assert_the_ways_out_with_the_device_table_changed_are_refused_without_reading_a_device(
        images,
        DeviceTableChange::OneDeviceFewer,
    );
}

/// 推环入口先核盘表次序（也先比次序）：被宽护卡住（建 inode、前缀 40 次那一形）之后，在这一刻盘面与会话的
/// 一份拷贝上照 `table_change` 换盘表推环：在任何读写之前拒成盘表那一种、带挂载时那一份与交进去的那一份，物理块大小没探、一个盘都没读、
/// 一个改盘调用都没有，盘逐字节不变、分配器与会话不动；换回挂载时那一份盘表照常推环做成（推到 R 那一槽）。不核这一判时，
/// 次序对调的盘表照样推成（攻方 `research/prompts/m3-batch10z-xy-code-r1-opus-output.md` 第二节：写的先后跟着盘表次序变）；
/// 多一块、少一块在核盘表那一套里读过盘才拒成 `DevicesHandedIn`。
pub fn assert_the_push_with_the_device_table_changed_is_refused_without_reading_a_device(
    images: &mut PoolCheckerImages,
    table_change: DeviceTableChange,
) {
    let PoolStuckBehindTheWideProtection { pool, stuck, .. } =
        pool_stuck_behind_the_wide_protection(
            images,
            FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
                overwrites_before: 40,
            },
        );
    let (mut counting_pool, calls_of_the_pool) =
        pool_failing_one_changing_call_copied_from(&pool, None);
    let device_table_of_the_mount = device_identities_in_the_order_handed_in(&counting_pool);
    let device_left_out =
        change_the_device_table_of(&mut counting_pool, table_change, &calls_of_the_pool);
    let device_table_handed_in = device_identities_in_the_order_handed_in(&counting_pool);
    let image_before = counting_pool.image();
    let allocator_before = format!("{:?}", counting_pool.session().allocator);
    let current_before = *counting_pool.session().current.root();
    match push_the_root_ring_of(&mut counting_pool) {
        Err(RootRingPushRefused::DeviceTableOtherThanTheOneOfTheMount {
            device_table_of_the_mount: refused_table_of_the_mount,
            device_table_handed_in: refused_table_handed_in,
        }) => assert_eq!(
            (refused_table_of_the_mount, refused_table_handed_in),
            (device_table_of_the_mount, device_table_handed_in),
            "{table_change:?}：带挂载时那一份盘表与交进去的那一份"
        ),
        other => panic!(
            "{table_change:?}：盘表与挂载时的不同：推环在任何读写之前拒成盘表那一种，而交回 {other:?}"
        ),
    }
    assert_eq!(
        (
            calls_of_the_pool.probes_of_the_physical_block_size.get(),
            calls_of_the_pool.reads.get(),
            calls_of_the_pool.calls.get()
        ),
        (0, 0, 0),
        "{table_change:?}：在任何读写之前拒：物理块大小没探、一个盘都没读、一个改盘调用都没有"
    );
    assert!(
        counting_pool.image() == image_before,
        "{table_change:?}：盘逐字节不变"
    );
    assert_eq!(
        format!("{:?}", counting_pool.session().allocator),
        allocator_before,
        "{table_change:?}：分配器整份不变"
    );
    assert_eq!(
        *counting_pool.session().current.root(),
        current_before,
        "{table_change:?}：会话的现行那一版不变"
    );
    restore_the_device_table_of(&mut counting_pool, table_change, device_left_out);
    let pushed = push_the_root_ring_of(&mut counting_pool).unwrap_or_else(|refusal| {
        panic!("{table_change:?} 之后换回挂载时那一份盘表：推环照常做成，而被拒：{refusal:?}")
    });
    assert_eq!(
        u64::try_from(pushed.versions_published.len()).expect("几次"),
        stuck.empty_publishes_until_the_last_ring_slot_the_push_reaches_is_written,
        "{table_change:?} 之后换回挂载时那一份盘表：推到 R 那一槽，发的次数是报里的"
    );
    assert!(
        pushed
            .ring_slots_written
            .contains(&stuck.last_ring_slot_the_push_reaches),
        "{table_change:?} 之后换回挂载时那一份盘表：写掉了 R 那一槽：{:?}",
        pushed.ring_slots_written
    );
}

pub fn run_the_scenario_where_pushing_the_root_ring_with_a_device_table_other_than_the_mount_in_reversed_order_is_refused_without_reading_a_device(
    images: &mut PoolCheckerImages,
) {
    assert_the_push_with_the_device_table_changed_is_refused_without_reading_a_device(
        images,
        DeviceTableChange::Reversed,
    );
}

pub fn run_the_scenario_where_pushing_the_root_ring_with_a_device_table_other_than_the_mount_with_one_device_more_is_refused_without_reading_a_device(
    images: &mut PoolCheckerImages,
) {
    assert_the_push_with_the_device_table_changed_is_refused_without_reading_a_device(
        images,
        DeviceTableChange::OneDeviceMore,
    );
}

pub fn run_the_scenario_where_pushing_the_root_ring_with_a_device_table_other_than_the_mount_with_one_device_fewer_is_refused_without_reading_a_device(
    images: &mut PoolCheckerImages,
) {
    assert_the_push_with_the_device_table_changed_is_refused_without_reading_a_device(
        images,
        DeviceTableChange::OneDeviceFewer,
    );
}

/// 会话已动（推过环，不在被拒的那一版上）又交次序对调的盘表要出路：先比版本、再比盘表，拒成 `SessionMoved…`、带被拒那一版与现行那一版的
/// 实例代号与 txg，不拒成盘表那一种（攻方 N1 / 正推 S2：盘表那一道挪到版本那一道之前时拒成盘表那一种，调用方换回盘表再要一次才拿到
/// `SessionMoved…`）。不预演：物理块大小没探、一个盘都没读、一个改盘调用都没有，盘逐字节不变。
pub fn run_the_scenario_where_asking_for_the_ways_out_after_the_session_moved_with_the_two_devices_in_reversed_order_is_refused_as_the_session_moved(
    images: &mut PoolCheckerImages,
) {
    let PoolStuckBehindTheWideProtection {
        mut pool, stuck, ..
    } = pool_stuck_behind_the_wide_protection(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
            overwrites_before: 40,
        },
    );
    let refused_on = *pool.session().current.root();
    push_the_root_ring_of(&mut pool).unwrap_or_else(|refusal| panic!("推环做得成：{refusal:?}"));
    let (mut counting_pool, calls_of_the_pool) =
        pool_failing_one_changing_call_copied_from(&pool, None);
    let current_after_the_push = *counting_pool.session().current.root();
    assert!(
        current_after_the_push.checkpoint_txg > refused_on.checkpoint_txg,
        "推环之后会话已在后面的一版上（阳性对照）"
    );
    let device_left_out = change_the_device_table_of(
        &mut counting_pool,
        DeviceTableChange::Reversed,
        &calls_of_the_pool,
    );
    assert!(device_left_out.is_none(), "次序对调不少交盘");
    let image_before = counting_pool.image();
    match counting_pool
        .session()
        .ways_out_of_the_wide_protection(&counting_pool.devices, &stuck)
    {
        Err(
            WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion {
                instance_refused_on,
                checkpoint_txg_refused_on,
                instance_of_the_current_version,
                checkpoint_txg_of_the_current_version,
            },
        ) => assert_eq!(
            (
                instance_refused_on,
                checkpoint_txg_refused_on,
                instance_of_the_current_version,
                checkpoint_txg_of_the_current_version
            ),
            (
                refused_on.instance,
                refused_on.checkpoint_txg,
                current_after_the_push.instance,
                current_after_the_push.checkpoint_txg
            ),
            "带被拒那一版与现行那一版的实例代号与 txg"
        ),
        other => panic!(
            "会话已不在被拒的那一版上、盘表又次序对调：先比版本，拒成 SessionMoved…，而交回 {other:?}"
        ),
    }
    assert_eq!(
        (
            calls_of_the_pool.probes_of_the_physical_block_size.get(),
            calls_of_the_pool.reads.get(),
            calls_of_the_pool.calls.get()
        ),
        (0, 0, 0),
        "不预演：物理块大小没探、一个盘都没读、一个改盘调用都没有"
    );
    assert!(counting_pool.image() == image_before, "盘逐字节不变");
}

/// 卸载再挂之后拿再挂之前的报要出路（被宽护卡住的回退那一形，同 `run_the_scenario_where_unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_the_recorded_publish_and_lets_writes_through`）：
/// 再挂取新实例，拒成 `SessionMoved…`，被拒那一版的两个数是报里的，现行那一版的实例代号与 txg 是再挂之后那一版的（新实例）
/// （正推 S4：现行那一版的实例代号取成报里的时照绿，入库用例里只有这一形分得开）。不预演：物理块大小没探、一个盘都没读、一个改盘调用都没有，
/// 盘逐字节不变。
pub fn run_the_scenario_where_asking_for_the_ways_out_after_unmounting_and_mounting_again_with_the_report_from_before_is_refused_as_the_session_moved_naming_the_new_instance(
    images: &mut PoolCheckerImages,
) {
    let PoolStuckBehindTheWideProtection {
        mut pool, stuck, ..
    } = pool_stuck_behind_the_wide_protection(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
            overwrites_before: OVERWRITES_BEFORE_THE_ROLLBACK_SIX_VERSIONS_BACK,
        },
    );
    let refused_on = *pool.session().current.root();
    let mut session = pool.session.take().expect("挂着的会话");
    let unmounted = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    );
    match unmounted {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
            for version in &raised.publishes {
                pool.note_root_of_the_current_content(&version.root);
            }
        }
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
            panic!("现行那一版带文件，卸载要推一串抬 F：{current:?}")
        }
        Err(refusal) => panic!("被宽护卡住时正常卸载做得成：{refusal:?}"),
    }
    pool.crash_and_mount_writable()
        .expect("卸载之后再可写挂载做得成（R 的账读得出）");
    let (counting_pool, calls_of_the_pool) =
        pool_failing_one_changing_call_copied_from(&pool, None);
    let current_after_mounting_again = *counting_pool.session().current.root();
    assert_ne!(
        current_after_mounting_again.instance, refused_on.instance,
        "再挂取新实例（阳性对照）"
    );
    let image_before = counting_pool.image();
    match counting_pool
        .session()
        .ways_out_of_the_wide_protection(&counting_pool.devices, &stuck)
    {
        Err(
            WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion {
                instance_refused_on,
                checkpoint_txg_refused_on,
                instance_of_the_current_version,
                checkpoint_txg_of_the_current_version,
            },
        ) => assert_eq!(
            (
                instance_refused_on,
                checkpoint_txg_refused_on,
                instance_of_the_current_version,
                checkpoint_txg_of_the_current_version
            ),
            (
                refused_on.instance,
                refused_on.checkpoint_txg,
                current_after_mounting_again.instance,
                current_after_mounting_again.checkpoint_txg
            ),
            "被拒那一版是报里的、现行那一版是再挂之后那一版（实例代号是新实例的）"
        ),
        other => panic!("卸载再挂之后拿再挂之前的报：拒成 SessionMoved…，而交回 {other:?}"),
    }
    assert_eq!(
        (
            calls_of_the_pool.probes_of_the_physical_block_size.get(),
            calls_of_the_pool.reads.get(),
            calls_of_the_pool.calls.get()
        ),
        (0, 0, 0),
        "不预演：物理块大小没探、一个盘都没读、一个改盘调用都没有"
    );
    assert!(counting_pool.image() == image_before, "盘逐字节不变");
}
