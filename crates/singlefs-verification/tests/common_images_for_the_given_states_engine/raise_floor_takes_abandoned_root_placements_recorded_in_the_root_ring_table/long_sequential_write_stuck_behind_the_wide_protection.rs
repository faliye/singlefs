//! 长顺序写被宽护卡住（攻方 K1 那一形）：推完放得下的列两条出路，推完照旧放不下的只列卸载再挂、照它走完放得下。

use super::*;

/// 攻方 K1 那一形（`research/prompts/m3-batch10z-pqrs-code-r1-opus-model/atk_part.rs` 的 `atk_k1_record`，`atk_k1_push_trace` 的 pre = 4、fill = 16）：
/// 第一个文件之后前缀覆盖写几次，文件每次长几个数据单元、长到几个，再录一次长几个单元的顺序写 R。
pub const OVERWRITES_BEFORE_GROWING_THE_FILE_UNTIL_THE_DEVICES_ARE_NEARLY_FULL: u64 = 4;

pub const DATA_UNITS_THE_FILE_GROWS_BY_EACH_TIME_BEFORE_THE_RECORDED_PUBLISH: usize = 5;

pub const DATA_UNITS_THE_FILE_GROWS_TO_BEFORE_THE_RECORDED_PUBLISH: usize = 16;

pub const DATA_UNITS_THE_RECORDED_PUBLISH_GROWS_THE_FILE_BY: usize = 3;

/// 攻方 K1 那一形崩在 R 第一条记录之后的盘面（不带任何读故障的自然崩溃点：可写挂载择 P，R 那几条记录在水位之上、没施加，
/// C 那一圈只有 R 那一槽，R 那一槽没有根）。
pub struct PoolCrashedAfterTheFirstRecordOfALongSequentialWrite {
    pub crashed: MemoryPool,
    pub content_of_each_root: BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>,
    pub write_time_seconds: u64,
    pub data_units_of_the_file: usize,
}

pub fn pool_crashed_after_the_first_record_of_a_long_sequential_write(
) -> PoolCrashedAfterTheFirstRecordOfALongSequentialWrite {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| {
            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
        },
    );
    for overwrite_number in 1..=OVERWRITES_BEFORE_GROWING_THE_FILE_UNTIL_THE_DEVICES_ARE_NEARLY_FULL
    {
        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!("前缀第 {overwrite_number} 次覆盖写落得下：{refusal:?}")
        });
    }
    let mut data_units_of_the_file = 1;
    // 迭代上界是长到的单元数；跨轮带着文件现在几个单元；出口：再长一步就越过长到的单元数。
    while data_units_of_the_file
        + DATA_UNITS_THE_FILE_GROWS_BY_EACH_TIME_BEFORE_THE_RECORDED_PUBLISH
        <= DATA_UNITS_THE_FILE_GROWS_TO_BEFORE_THE_RECORDED_PUBLISH
    {
        data_units_of_the_file +=
            DATA_UNITS_THE_FILE_GROWS_BY_EACH_TIME_BEFORE_THE_RECORDED_PUBLISH;
        pool.sequential_write(data_units_of_the_file * unit_bytes)
            .unwrap_or_else(|refusal| {
                panic!("文件长到 {data_units_of_the_file} 个单元落得下：{refusal:?}")
            });
    }
    assert_eq!(
        data_units_of_the_file, DATA_UNITS_THE_FILE_GROWS_TO_BEFORE_THE_RECORDED_PUBLISH,
        "文件长到攻方那一形的单元数"
    );
    let base = pool.image();
    let operations_before_the_recorded_publish = stream.operation_count();
    let recorded_publish_as_published = pool
        .sequential_write(
            (data_units_of_the_file + DATA_UNITS_THE_RECORDED_PUBLISH_GROWS_THE_FILE_BY)
                * unit_bytes,
        )
        .unwrap_or_else(|refusal| panic!("录的那次顺序写（R）落得下：{refusal:?}"));
    assert!(
        recorded_publish_as_published.floor_raises.is_empty(),
        "R 那次没推抬 F：录下来的写全是 R 自己的"
    );
    let operations = stream.retained_operations();
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_the_recorded_publish..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let end_of_the_first_record = writes
        .iter()
        .position(|write| write.kind == StepKind::JournalRecord)
        .expect("R 写了记录")
        + 1;
    let mut crashed = base;
    crashed.apply_writes(&writes[..end_of_the_first_record]);
    PoolCrashedAfterTheFirstRecordOfALongSequentialWrite {
        crashed,
        content_of_each_root: pool.content_of_each_root.clone(),
        write_time_seconds: pool.write_time_seconds + 100,
        data_units_of_the_file,
    }
}

impl PoolCrashedAfterTheFirstRecordOfALongSequentialWrite {
    /// 拿崩溃那一刻的盘面起两块新盘、可写挂载一次（空间准入判着）。
    pub fn mounted(&self) -> PoolUnderTest<SparseBlockDevice> {
        let mut pool = PoolUnderTest {
            parameters: parameters_of_the_384_slot_small_devices(),
            device_bytes: device_bytes_of_the_384_slot_small_devices(),
            devices: plain_devices_on(&self.crashed),
            session: None,
            write_time_seconds: self.write_time_seconds,
            content_of_the_current_version: Vec::new(),
            content_of_each_root: self.content_of_each_root.clone(),
        };
        pool.crash_and_mount_writable()
            .expect("崩在 R 第一条记录之后：可写挂载择 P、做得成");
        let holding = pool
            .session()
            .allocator
            .root_ring_occupancy()
            .expect("挂载建的分配器装着根环表")
            .ring_slots_holding_placements_named_by_unapplied_journal_records();
        assert_eq!(
            holding.len(),
            1,
            "C 那一圈只有 R 那一槽（阳性对照）：{:?}",
            holding.keys().collect::<Vec<_>>()
        );
        pool
    }

    /// 挂上之后写一次长 `data_units_grown_by` 个单元的顺序写：被拒、报「被宽护卡住」（这一次调用里会话先推一串抬 F、把 F 抬到上限）。
    /// 交回被拒之后挂着的池：再试同一个写时 F 已在上限，那一次调用一个写都不推。
    pub fn mounted_and_stuck_once_by_a_write_grown_by(
        &self,
        data_units_grown_by: usize,
    ) -> PoolUnderTest<SparseBlockDevice> {
        let mut pool = self.mounted();
        let refusal = pool
            .sequential_write(self.bytes_of_the_file_grown_by(data_units_grown_by))
            .expect_err("挂上之后第一次长文件：宽护挡着准入");
        stuck_behind_the_wide_protection_reported_by(
            refusal,
            &format!("挂上之后第一次长 {data_units_grown_by} 个单元"),
        );
        pool
    }

    /// 把文件长到比 P 多 `data_units` 个数据单元的顺序写的字节数（攻方的 g）。
    pub fn bytes_of_the_file_grown_by(&self, data_units: usize) -> usize {
        (self.data_units_of_the_file + data_units)
            * singlefs_core::unit::data_unit_payload_capacity()
    }
}

/// 攻方 K1 那一形里长到这几个单元的写：宽护挡着、推完放得下的那一档（攻方第一节：g ≤ 19 推完第一次就放得下）；推完照旧被拒的那一档（g = 20）；
/// 没有 C 那一圈的对照臂同一个长度也被拒的那一档（g = 36，对照臂 g = 36 起被拒）。
pub const DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY: usize = 13;

pub const DATA_UNITS_A_WRITE_STILL_REFUSED_AFTER_THE_PUSH_GROWS_THE_FILE_BY: usize = 20;

pub const DATA_UNITS_A_WRITE_REFUSED_EVEN_WITHOUT_THE_WIDE_PROTECTION_GROWS_THE_FILE_BY: usize = 36;

/// 甲（攻方 K1 那一形 g = 13）：要出路时预演推环与推完之后这次改动、推完放得下——两条出路都列：出路一说 R 那一槽没有读得出的有效根，
/// 出路二写明推几次、写掉几槽、R 那一槽、放出来的隔离槽数。挂上之后第一次被拒时会话推了一串抬 F、把 F 抬到上限；再试同一个写，
/// 那一次调用一串抬 F 都不推。报「被宽护卡住」那一次不预演（预演延到要出路时才做）：一块只进内存的盘都没建、一个改盘调用都没有；
/// 要出路那一次每块盘建一块、一个写都不发、会话的分配器不动。探物理块大小的只有三核，每核每块盘探一次，数探的次数分得出建没建只进内存的盘：
/// 会话发布用户改动的入口比探测值（`MountedSession::publish_user_change` 进循环之前），推抬 F 那一串核盘表
/// （`mount::push_one_floor_raise_within_the_admission_budget` 经 `caller_inputs_agreeing_with_the_disk`；F 已在上限的这一次也先核、再停），
/// 预演建只进内存的盘（`rehearse_the_push_then_the_remainder_of_the_change`）。
/// 照推环走完，放出来的隔离槽数就是推完前后每块盘隔离数之差，同一个写放得下，池级 checker 不红 I-7.4、I-4.8。
pub fn run_the_scenario_where_a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push(
    images: &mut PoolCheckerImages,
) {
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let stuck_once = crashed.mounted_and_stuck_once_by_a_write_grown_by(
        DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY,
    );
    let (mut pool, calls_of_the_pool) =
        pool_failing_one_changing_call_copied_from(&stuck_once, None);
    let length = crashed
        .bytes_of_the_file_grown_by(DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY);
    let image_before_the_write = pool.image();
    let allocator_before_the_write = format!("{:?}", pool.session().allocator);
    let refusal = pool
        .sequential_write(length)
        .expect_err("长 13 个单元再试：宽护照旧挡着准入");
    let WriteStuckBehindTheWideProtection {
        floor_raises_pushed,
        versions_published,
        stuck,
    } = stuck_behind_the_wide_protection_reported_by(refusal, "长 13 个单元再试");
    assert_eq!(
        (floor_raises_pushed, versions_published),
        (0, 0),
        "第一次被拒时已把 F 抬到上限：再试这一次一串抬 F 都没推、一版都没发（阳性对照：盘上的改动只可能来自预演）"
    );
    let devices_of_the_pool = u64::try_from(pool.devices.len()).expect("盘数");
    assert_eq!(
        (
            calls_of_the_pool.probes_of_the_physical_block_size.get(),
            calls_of_the_pool.calls.get()
        ),
        (2 * devices_of_the_pool, 0),
        "报「被宽护卡住」那一次不预演：每块盘只探两次——会话发布用户改动的入口比探测值一次、推抬 F 那一串核盘表一次\
         （F 已在上限，核完就停），一块只进内存的盘都没建（建一块要再探一次）、一个改盘调用都没有"
    );
    assert!(
        pool.image() == image_before_the_write,
        "报「被宽护卡住」那一次：两块盘逐字节不变"
    );
    let WaysOutOfTheWideProtection {
        pushing_the_root_ring,
        ways_out,
    } = ways_out_of(&pool, &stuck, "长 13 个单元");
    assert_eq!(
        (
            calls_of_the_pool.probes_of_the_physical_block_size.get(),
            calls_of_the_pool.calls.get()
        ),
        (3 * devices_of_the_pool, 0),
        "要出路那一次预演：每块盘建一块只进内存的盘、各再探一次（前两次是报「被宽护卡住」那一次的入口与推抬 F 那一串的核），\
         一个改盘调用都没发到底下的盘"
    );
    assert!(
        pool.image() == image_before_the_write,
        "要出路那一次预演推环与推完之后这次改动：两块盘逐字节不变"
    );
    assert_eq!(
        format!("{:?}", pool.session().allocator),
        allocator_before_the_write,
        "预演在分配器的拷贝上：会话的分配器整份不变"
    );
    assert_eq!(
        stuck.checkpoint_txg_of_the_last_ring_slot_the_push_reaches, None,
        "崩在 R 第一条记录之后：R 那一槽没有读得出的有效根（阳性对照：出路一要说的是这一种）"
    );
    assert_eq!(ways_out.len(), 2, "推完放得下：两条出路都列：{ways_out:?}");
    assert_the_way_out_by_unmounting_says_the_ring_slot_of_the_recorded_publish_holds_no_readable_root(
        &ways_out[0],
        &stuck,
        "长 13 个单元",
    );
    let released_in_the_rehearsal =
        released_by_the_push_if_the_change_is_admitted_after_it(&pushing_the_root_ring)
            .unwrap_or_else(|| {
                panic!("预演推完之后这次改动放得下，而预演的结局是 {pushing_the_root_ring:?}")
            });
    assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_the_recorded_publish_and_the_released_slots(
        &stuck,
        &ways_out[1],
        &released_in_the_rehearsal,
        WhatIsLeftAfterThePush::PublishTheChangeAgain,
        "长 13 个单元",
    );
    let isolated_before_the_push = isolated_slots_on_each_device_of(&pool.session().allocator);
    let pushed = push_the_root_ring_of(&mut pool)
        .unwrap_or_else(|push_refusal| panic!("照出路二推环做得成：{push_refusal:?}"));
    assert_eq!(
        u64::try_from(pushed.versions_published.len()).expect("次数"),
        stuck.empty_publishes_until_the_last_ring_slot_the_push_reaches_is_written,
        "推环发的空发布次数就是报里写的那个数"
    );
    let released_by_the_push = isolated_slots_released_between(
        &isolated_before_the_push,
        &isolated_slots_on_each_device_of(&pool.session().allocator),
    );
    assert_eq!(
        released_in_the_rehearsal, released_by_the_push,
        "出路二说放出来的隔离槽数就是推完前后每块盘隔离数之差"
    );
    pool.sequential_write(length)
        .unwrap_or_else(|refusal_after_the_push| {
            panic!("推完之后同一个写放得下：{refusal_after_the_push:?}")
        });
    images.hand_out(
        "推完写下之后 I-7.4、I-4.8 不红",
        || pool.image(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
    );
}

/// 甲（攻方 K1 那一形）：推完照旧放不下的写，被宽护卡住、要出路时只列卸载再挂（预演推完之后照旧被拒）；预演交回的拒绝与照出路二
/// 在一份拷贝上真推完、再发同一个写交回的拒绝逐字相同。照出路一走：正常卸载、再可写挂载，同一个写放得下，池级 checker 不红 I-7.4、I-4.8。
pub fn a_write_stuck_behind_the_wide_protection_the_push_does_not_let_through_lists_only_unmounting_which_lets_it_through(
    images: &mut PoolCheckerImages,
    data_units_grown_by: usize,
) {
    let why = format!("长 {data_units_grown_by} 个单元");
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let mut pool = crashed.mounted();
    let length = crashed.bytes_of_the_file_grown_by(data_units_grown_by);
    let refusal = pool.sequential_write(length).expect_err("宽护挡着准入");
    let WriteStuckBehindTheWideProtection { stuck, .. } =
        stuck_behind_the_wide_protection_reported_by(refusal, &why);
    let WaysOutOfTheWideProtection {
        pushing_the_root_ring,
        ways_out,
    } = ways_out_of(&pool, &stuck, &why);
    assert!(
        matches!(
            pushing_the_root_ring,
            RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. }
        ),
        "{why}：预演推环做得成、推完之后这次改动照旧被拒：{pushing_the_root_ring:?}"
    );
    assert_eq!(
        ways_out.len(),
        1,
        "{why}：推完放不下，只列卸载再挂：{ways_out:?}"
    );
    assert_the_way_out_by_unmounting_says_the_ring_slot_of_the_recorded_publish_holds_no_readable_root(
        &ways_out[0],
        &stuck,
        &why,
    );
    let content_of_the_refused_write = content_of(length, pool.write_time_seconds);
    assert_the_rehearsed_refusal_is_the_refusal_after_pushing_the_root_ring(
        &pool,
        &pushing_the_root_ring,
        UserChange::SequentialWrite(FirstFile {
            content: &content_of_the_refused_write,
            write_time_seconds: pool.write_time_seconds,
        }),
        &why,
    );
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
            panic!("{why}：现行那一版带文件，卸载要推一串抬 F：{current:?}")
        }
        Err(unmount_refusal) => panic!("{why}：照出路一正常卸载做得成：{unmount_refusal:?}"),
    }
    pool.crash_and_mount_writable()
        .unwrap_or_else(|mount_refusal| {
            panic!("{why}：卸载之后再可写挂载做得成：{mount_refusal:?}")
        });
    pool.sequential_write(length)
        .unwrap_or_else(|refusal_after_mounting_again| {
            panic!("{why}：卸载再挂之后同一个写放得下：{refusal_after_mounting_again:?}")
        });
    images.hand_out(
        &format!("{why}：卸载再挂写下之后 I-7.4、I-4.8 不红"),
        || pool.image(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
    );
}

/// 攻方 K1 那一形 g = 20：宽护确实挡着（对照臂同一个长度放得下），推完之后上限退到环里最旧的根，照旧被拒。
pub fn run_the_scenario_where_a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through(
    images: &mut PoolCheckerImages,
) {
    a_write_stuck_behind_the_wide_protection_the_push_does_not_let_through_lists_only_unmounting_which_lets_it_through(images,
        DATA_UNITS_A_WRITE_STILL_REFUSED_AFTER_THE_PUSH_GROWS_THE_FILE_BY,
    );
}

/// 攻方 K1 那一形 g = 36：没有 C 那一圈的对照臂同一个长度也被拒（宽护是挡着的之一、不是唯一），照样报「被宽护卡住」、只列卸载再挂。
pub fn run_the_scenario_where_a_write_refused_even_without_the_wide_protection_lists_only_unmounting_which_lets_it_through(
    images: &mut PoolCheckerImages,
) {
    a_write_stuck_behind_the_wide_protection_the_push_does_not_let_through_lists_only_unmounting_which_lets_it_through(images,
        DATA_UNITS_A_WRITE_REFUSED_EVEN_WITHOUT_THE_WIDE_PROTECTION_GROWS_THE_FILE_BY,
    );
}

/// 甲（攻方第一节「实现员自己的三形上……长到 60、80 个单元的推完照旧被拒」）：实现员三形卡住之后，把文件写成这么多个数据单元的顺序写
/// 被宽护卡住、要出路时只列卸载再挂；R 那一槽记着有效根，出路一照 R 的 txg 说它判成被抛弃、写明账永久读不出时的后果。
/// 预演交回的拒绝与照出路二在一份拷贝上真推完、再发同一个写交回的拒绝逐字相同。
pub fn a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
    images: &mut PoolCheckerImages,
    form: FormStuckBehindTheWideProtectionOnThe384SlotDevices,
    data_units: usize,
) {
    let why = format!("{form:?} 写成 {data_units} 个单元");
    let PoolStuckBehindTheWideProtection {
        recorded, mut pool, ..
    } = pool_stuck_behind_the_wide_protection(images, form);
    let length = data_units * singlefs_core::unit::data_unit_payload_capacity();
    let refusal = pool.sequential_write(length).expect_err("宽护挡着准入");
    let WriteStuckBehindTheWideProtection { stuck, .. } =
        stuck_behind_the_wide_protection_reported_by(refusal, &why);
    let WaysOutOfTheWideProtection {
        pushing_the_root_ring,
        ways_out,
    } = ways_out_of(&pool, &stuck, &why);
    assert!(
        matches!(
            pushing_the_root_ring,
            RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. }
        ),
        "{why}：预演推环做得成、推完之后这次改动照旧被拒：{pushing_the_root_ring:?}"
    );
    let content_of_the_refused_write = content_of(length, pool.write_time_seconds);
    assert_the_rehearsed_refusal_is_the_refusal_after_pushing_the_root_ring(
        &pool,
        &pushing_the_root_ring,
        UserChange::SequentialWrite(FirstFile {
            content: &content_of_the_refused_write,
            write_time_seconds: pool.write_time_seconds,
        }),
        &why,
    );
    assert_eq!(
        ways_out.len(),
        1,
        "{why}：推完放不下，只列卸载再挂：{ways_out:?}"
    );
    assert_eq!(
        stuck.checkpoint_txg_of_the_last_ring_slot_the_push_reaches,
        Some(recorded.key.1),
        "{why}：R 那一槽记着录下来的 R（阳性对照：出路一要说的是这一种）"
    );
    assert_the_way_out_by_unmounting_names_the_recorded_publish_abandoned_and_warns_of_a_permanently_unreadable_account(
        &ways_out[0],
        &stuck,
        recorded.key.1,
        &why,
    );
}

/// 长 60 个数据单元、80 个数据单元的顺序写（攻方 `atk_k1_implementer_forms_sweep` 推完仍拒的那两档）。
pub const DATA_UNITS_OF_A_LONG_WRITE_STILL_REFUSED_AFTER_THE_PUSH: usize = 60;

pub const DATA_UNITS_OF_A_LONGER_WRITE_STILL_REFUSED_AFTER_THE_PUSH: usize = 80;

pub fn run_the_scenario_where_a_sixty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting(
    images: &mut PoolCheckerImages,
) {
    a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
            overwrites_before: 20,
        },
        DATA_UNITS_OF_A_LONG_WRITE_STILL_REFUSED_AFTER_THE_PUSH,
    );
}

pub fn run_the_scenario_where_an_eighty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting(
    images: &mut PoolCheckerImages,
) {
    a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
            overwrites_before: 20,
        },
        DATA_UNITS_OF_A_LONGER_WRITE_STILL_REFUSED_AFTER_THE_PUSH,
    );
}

pub fn run_the_scenario_where_a_sixty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting(
    images: &mut PoolCheckerImages,
) {
    a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
            overwrites_before: 40,
        },
        DATA_UNITS_OF_A_LONG_WRITE_STILL_REFUSED_AFTER_THE_PUSH,
    );
}

pub fn run_the_scenario_where_an_eighty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting(
    images: &mut PoolCheckerImages,
) {
    a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
            overwrites_before: 40,
        },
        DATA_UNITS_OF_A_LONGER_WRITE_STILL_REFUSED_AFTER_THE_PUSH,
    );
}

pub fn run_the_scenario_where_a_sixty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting(
    images: &mut PoolCheckerImages,
) {
    a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::FirstPublishOfADeletion {
            overwrites_before: 160,
        },
        DATA_UNITS_OF_A_LONG_WRITE_STILL_REFUSED_AFTER_THE_PUSH,
    );
}

pub fn run_the_scenario_where_an_eighty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting(
    images: &mut PoolCheckerImages,
) {
    a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::FirstPublishOfADeletion {
            overwrites_before: 160,
        },
        DATA_UNITS_OF_A_LONGER_WRITE_STILL_REFUSED_AFTER_THE_PUSH,
    );
}
