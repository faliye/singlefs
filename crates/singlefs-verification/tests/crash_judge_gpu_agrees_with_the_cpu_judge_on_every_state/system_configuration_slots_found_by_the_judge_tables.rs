//! 根环里的系统配置槽（I-7 类）：判表照 CPU 择系统配置的找法取落点、逐档读到的槽 1 带读者不收之值、逐盘自证过的槽数。

use super::*;

/// 基线里盘 0 的两个系统配置槽与盘 1 的槽 0 都抹成 0，只剩盘 1 的槽 1 自证得过（CPU 恢复与池级 checker 都择它；流里之后的系统配置槽写
/// 再把轮到的那一槽写回来）：判表的落点几何要从盘 1 的槽 1 取，只读第一块盘槽 0 的取法在这条流上建不出表。
fn record_new_pool_with_only_the_slot_one_of_the_second_device_self_verified_in_the_base(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(
        tag,
        "new-pool-only-slot-one-of-device-one-self-verified-in-the-base",
    );
    let spacing = u64::from(geometry().fixed_structure_slot_spacing);
    for (device, offset) in [(0, 0), (0, spacing), (1, 0)] {
        flow.base
            .devices
            .get_mut(&DeviceIdentity(device))
            .expect("基线里有这块盘")
            .write(
                singlefs_core::address::DeviceOffsetInBytes(offset),
                &[0u8; 4096],
            );
    }
    flow
}

/// 新池新建文件那条流只留盘 0：基线里没有盘 1（系统配置记的根环区域 1 在盘 1 上），写表只录盘 0 上的写（发布那次 txg 3 的根落在区域 0、盘 0 上）。
/// 判表只在池里的盘上建落点；给区域盘一律建落点的取法从基线读盘 1 读不出、建表时 panic。
fn record_new_pool_file_creation_on_device_zero_alone(tag: &str) -> RecordedFlow {
    let pool = build_pool(tag);
    let operations_on_device_zero: Vec<_> = pool.retained_operations()[pool.mkfs_operation_count..]
        .iter()
        .filter(|retained| retained.operation.device == DeviceIdentity(0))
        .cloned()
        .collect();
    let (writes, segments, _stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations_on_device_zero, &geometry());
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布那次的根槽写在盘 0 上（txg 3 落在区域 0）");
    let mut base = pool.memory_pool_after_mkfs();
    base.devices
        .remove(&DeviceIdentity(1))
        .expect("mkfs 之后基线里有盘 1");
    RecordedFlow {
        name: "new-pool-file-creation-on-device-zero-alone",
        base,
        writes,
        segments,
        judged_root_index,
        versions: vec![PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content: file_content(),
        }],
    }
}

/// 一条流的判表（展开上限照 [`expand_up_to`]，与 [`compare_on_every_state`] 同一种展开）：取卡之前在 CPU 上建，没有卡的机器上这一半照样判。
pub(super) fn judge_tables_of_the_flow(flow: &RecordedFlow) -> JudgeTables {
    let limit = expand_up_to();
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    })
    .unwrap_or_else(|refusal| panic!("流 {} 建得出判表：{refusal:?}", flow.name))
}

/// 一条流上 GPU 与 CPU 逐状态比，一个状态都不许不一致、判不了。
pub(super) fn assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_the_flow(
    test_name: &str,
    flow: &RecordedFlow,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, flow, expand_up_to()) else {
        return;
    };
    assert!(comparison.states > 0, "流 {} 有状态可判", flow.name);
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// 基线里只剩盘 1 的槽 1 自证得过：判表照 CPU 择系统配置的找法从那一槽取落点几何、建得出（取卡之前判），每个崩溃状态 GPU 与 CPU 逐状态相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_only_the_slot_one_of_the_second_device_self_verifies_in_the_base(
) {
    let flow =
        record_new_pool_with_only_the_slot_one_of_the_second_device_self_verified_in_the_base(
            "crash-judge-gpu-only-slot-one-of-device-one",
        );
    let tables = judge_tables_of_the_flow(&flow);
    let spacing = u64::from(geometry().fixed_structure_slot_spacing);
    let system_configuration_locations: Vec<(u32, u64)> = tables
        .locations
        .iter()
        .filter(|location| location.kind == JudgeLocationKind::SystemConfiguration)
        .map(|location| (location.device, location.offset))
        .collect();
    assert_eq!(
        system_configuration_locations,
        vec![(0, 0), (0, spacing), (1, 0), (1, spacing)],
        "流 {}：落点几何取自盘 1 的槽 1，两块盘各有两个系统配置槽的落点",
        flow.name
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_the_flow(
        "the_gpu_judge_agrees_with_the_cpu_when_only_the_slot_one_of_the_second_device_self_verifies_in_the_base",
        &flow,
    );
}

/// 池里少一块盘（只剩盘 0，系统配置记的根环区域 1 在丢掉的盘 1 上）：判表建得出（取卡之前判）、盘 1 上一个落点都不建，
/// 每个崩溃状态 GPU 与 CPU 逐状态相同（CPU 读盘 1 交 None、GPU 在盘 1 上找不到落点，两边都当那几个根槽读不出）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_a_pool_that_lost_a_device_holding_a_root_ring_region() {
    let flow = record_new_pool_file_creation_on_device_zero_alone(
        "crash-judge-gpu-new-pool-on-device-zero-alone",
    );
    let tables = judge_tables_of_the_flow(&flow);
    assert_eq!(tables.devices, vec![0], "流 {}：池里只有盘 0", flow.name);
    assert!(
        tables.locations.iter().all(|location| location.device == 0),
        "流 {}：落点都在盘 0 上",
        flow.name
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_the_flow(
        "the_gpu_judge_agrees_with_the_cpu_on_a_pool_that_lost_a_device_holding_a_root_ring_region",
        &flow,
    );
}

/// journal 环起点字段写成第一版常量 1024 之外的一个槽号：自证得过、读者不收（与 harness 场景
/// `system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs` 的同名常量同一个值）。
const JOURNAL_RING_START_SLOT_THE_READER_REFUSES: u64 = 1025;

/// 基线里 `device` 上偏移 `offset` 那一个系统配置槽（自证得过）。
fn system_configuration_slot_in_the_base(
    flow: &RecordedFlow,
    device: DeviceIdentity,
    offset: u64,
) -> singlefs_core::system_configuration::SystemConfiguration {
    let slot_bytes =
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    singlefs_core::system_configuration::SystemConfiguration::parse_slot(
        &singlefs_core::recovery::PoolReader::read(
            &flow.base,
            device,
            singlefs_core::address::DeviceOffsetInBytes(offset),
            slot_bytes,
        )
        .expect("基线里这一槽读得到"),
    )
    .expect("基线里这一槽自证得过")
}

/// 基线里 `device` 上偏移 `offset` 起写 `bytes`。
fn write_into_the_base(flow: &mut RecordedFlow, device: DeviceIdentity, offset: u64, bytes: &[u8]) {
    flow.base
        .devices
        .get_mut(&device)
        .expect("基线里有这块盘")
        .write(singlefs_core::address::DeviceOffsetInBytes(offset), bytes);
}

/// 两块盘的槽 0 都抹成 0（全池没有一个可择的槽 0：恢复与池级 checker 逐档读槽 1，直到这条流把槽 0 写回）。
fn wipe_the_slot_zeros_in_the_base(flow: &mut RecordedFlow) {
    let slot_bytes =
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        write_into_the_base(flow, device, 0, &vec![0u8; slot_bytes]);
    }
}

/// 新池新建文件那条流的基线：两块盘的槽 0 都抹掉，槽距处本池的槽 1 照旧，再远一档（两倍槽距）放一份本池槽 1 的拷贝、journal 环起点写成 1025
/// （自证得过、读者不收，记的槽距仍是本池的，不等于它所在的偏移；harness 场景「8192 处那份槽带不收之值」的造法）。
fn record_new_pool_with_a_refused_slot_one_farther_than_the_pool_slot_one_in_the_base(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(
        tag,
        "new-pool-refused-slot-one-farther-than-the-pool-slot-one-in-the-base",
    );
    let spacing = u64::from(geometry().fixed_structure_slot_spacing);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut refused = system_configuration_slot_in_the_base(&flow, device, spacing);
        refused.immutable.journal_ring_start_slot =
            singlefs_core::address::SlotNumber(JOURNAL_RING_START_SLOT_THE_READER_REFUSES);
        write_into_the_base(&mut flow, device, 2 * spacing, &refused.to_slot());
    }
    wipe_the_slot_zeros_in_the_base(&mut flow);
    flow
}

/// 新池新建文件那条流的基线：两块盘的槽 0 都抹掉；逐档读到的第一个自证得过的槽 1（槽距处、记的槽距就是它所在的偏移）带读者不收之值
/// （journal 环起点 1025），更远一档（两倍槽距）有一份读者收的槽 1（本池槽 1 的拷贝、记的槽距改成两倍槽距、等于它所在的偏移）。
fn record_new_pool_whose_first_self_verified_slot_one_carries_a_refused_value_and_a_farther_one_is_accepted(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(
        tag,
        "new-pool-first-self-verified-slot-one-refused-and-a-farther-one-accepted-in-the-base",
    );
    let spacing = u64::from(geometry().fixed_structure_slot_spacing);
    let farther_spacing = 2 * spacing;
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let pool_slot_one = system_configuration_slot_in_the_base(&flow, device, spacing);
        let mut refused = pool_slot_one;
        refused.immutable.journal_ring_start_slot =
            singlefs_core::address::SlotNumber(JOURNAL_RING_START_SLOT_THE_READER_REFUSES);
        let mut farther = pool_slot_one;
        farther.immutable.sizes.fixed_structure_slot_spacing =
            u32::try_from(farther_spacing).expect("两倍槽距装得进 4 字节");
        write_into_the_base(&mut flow, device, spacing, &refused.to_slot());
        write_into_the_base(&mut flow, device, farther_spacing, &farther.to_slot());
    }
    wipe_the_slot_zeros_in_the_base(&mut flow);
    flow
}

/// 一条流的判表里系统配置槽那一类落点在哪几处（盘、偏移），升序。
fn system_configuration_locations_of(tables: &JudgeTables) -> Vec<(u32, u64)> {
    tables
        .locations
        .iter()
        .filter(|location| location.kind == JudgeLocationKind::SystemConfiguration)
        .map(|location| (location.device, location.offset))
        .collect()
}

/// 两块盘的槽 0 都废、槽距处本池的槽 1 照旧、再远一档有一份带读者不收之值的槽（hand_out 那张「8192 处那份槽带不收之值」的形，搬到新池新建文件这条流上）：
/// 判表给每块盘的两倍槽距处建系统配置落点（取卡之前判，没有卡的机器上这一半照样判）；这条流把槽 0 写回之前的状态 CPU 逐档读到那一份、判 I-7.13，
/// GPU 逐档读同一处，每个状态逐位相同。只建偏移 0 与几何槽距两处落点的取法在取卡之前那一判就红（落点表少了两倍槽距那两处），
/// 上卡时 GPU 读那一处读成不自证、不红 I-7.13。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_a_refused_slot_one_found_farther_than_the_pool_slot_one_while_reading_every_spacing(
) {
    let flow = record_new_pool_with_a_refused_slot_one_farther_than_the_pool_slot_one_in_the_base(
        "crash-judge-gpu-refused-slot-one-farther",
    );
    let tables = judge_tables_of_the_flow(&flow);
    let spacing = u64::from(geometry().fixed_structure_slot_spacing);
    assert_eq!(
        system_configuration_locations_of(&tables),
        vec![
            (0, 0),
            (0, spacing),
            (0, 2 * spacing),
            (1, 0),
            (1, spacing),
            (1, 2 * spacing)
        ],
        "流 {}：两块盘各有偏移 0、槽距与逐档读到的那一处三个系统配置槽的落点",
        flow.name
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_on_a_refused_slot_one_found_farther_than_the_pool_slot_one_while_reading_every_spacing",
        &flow,
        "I-7.13",
    );
}

/// 两块盘的槽 0 都废、逐档读到的第一个自证得过的槽 1 带读者不收之值、更远一档槽距上有一份读者收的槽 1：落点几何取第一个那一份
/// （记的槽距就是这条流的槽距，根槽写都对得上落点；不看读者收不收），更远那一份另建落点（取卡之前判）；这条流把槽 0 写回之前的状态
/// CPU 逐档读到第一个那一份就判 I-7.13，GPU 逐位相同。GPU 走读内核把 journal 环起点不是 1024 的槽读成不自证时，GPU 逐档读到更远那一份、
/// 照常判，不红 I-7.13，这条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_first_self_verified_slot_one_carries_a_refused_value_and_a_farther_one_is_accepted(
) {
    let flow = record_new_pool_whose_first_self_verified_slot_one_carries_a_refused_value_and_a_farther_one_is_accepted(
        "crash-judge-gpu-first-slot-one-refused-farther-accepted",
    );
    let tables = judge_tables_of_the_flow(&flow);
    let spacing = u64::from(geometry().fixed_structure_slot_spacing);
    assert_eq!(
        system_configuration_locations_of(&tables),
        vec![
            (0, 0),
            (0, spacing),
            (0, 2 * spacing),
            (1, 0),
            (1, spacing),
            (1, 2 * spacing)
        ],
        "流 {}：几何取槽距处那一份，两块盘各有偏移 0、槽距与更远那一份三个系统配置槽的落点",
        flow.name
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_first_self_verified_slot_one_carries_a_refused_value_and_a_farther_one_is_accepted",
        &flow,
        "I-7.13",
    );
}

// ─── 里程碑三收尾（G4 丙-置的 GPU 一侧）：逐盘自证过的系统配置槽数上 GPU ───

/// 新池新建文件那条流，流尾另起一段：盘 0 的槽 0 原地覆写成盘 0 槽 1 那一份自证过的字节、盘 0 的槽 1 整槽清零（攻方坏实现 MUT-B1 那一形：
/// 一段里一块盘的系统配置槽写不止一个）。槽 0 那次写撕裂、槽 1 清零落了的那一态，盘 0 一个自证过的系统配置槽都没有，基镜像上它两槽都有。
fn record_new_pool_with_both_system_configuration_slots_of_device_zero_lost_in_one_segment(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(
        tag,
        "new-pool-both-system-configuration-slots-of-device-zero-lost-in-one-segment",
    );
    let spacing = u64::from(geometry().fixed_structure_slot_spacing);
    let mut landed = flow.base.clone();
    landed.apply_writes(&flow.writes);
    let slot_one_of_device_zero = singlefs_core::recovery::PoolReader::read(
        &landed,
        singlefs_core::address::DeviceIdentity(0),
        singlefs_core::address::DeviceOffsetInBytes(spacing),
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096"),
    )
    .expect("新池新建文件之后盘 0 的槽 1 读得出");
    let slot_write = |offset: u64, contents: WrittenContents| RetainedWrite {
        device: singlefs_core::address::DeviceIdentity(0),
        kind: StepKind::SystemConfigurationSlot,
        is_force_unit_access: false,
        offset: singlefs_core::address::DeviceOffsetInBytes(offset),
        contents,
    };
    append_a_segment_of_writes(
        &mut flow,
        vec![
            slot_write(0, WrittenContents::Bytes(slot_one_of_device_zero)),
            slot_write(
                spacing,
                WrittenContents::Bytes(vec![
                    0u8;
                    usize::try_from(
                        singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES
                    )
                    .expect("4096")
                ]),
            ),
        ],
    );
    flow
}

/// 场景：[`record_new_pool_with_both_system_configuration_slots_of_device_zero_lost_in_one_segment`] 那条流的每个状态（撕裂那一态在内）。
/// 预期：GPU 走读内核交的逐盘自证过的系统配置槽数（`crash_judge_gpu::SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOTS_OF_EACH_DEVICE_WORD`）与 CPU 参照
/// （`crash_judge_conclusions::self_verified_system_configuration_slots_of_each_device`，池级 checker 的逐盘数）逐状态逐盘相同；判段那一判
/// （`devices_without_a_self_verified_system_configuration_slot_against_the_base_image_from_the_slot_counts`）拿 GPU 那一份与拿 CPU 参照那一份（没配 GPU 时判段
/// CPU 那一路交的）对基镜像比，逐状态结局相同，且至少一个状态记盘 0 是新出现的死盘；对拍字逐状态相同。
/// 判别力：`crates/mutations.tsv` 里「GPU 逐盘自证槽数恒数满两槽」那一行让这一条红（撕裂那一态上 GPU 报 2、CPU 报 0）。
// crash-case-check:not-a-crash-case GPU 走读内核交的逐盘自证过的系统配置槽数与 CPU 参照逐状态逐盘对拍，对拍按块判时每个状态上另看这一样，不调枚举一族函数、不自己造崩溃镜像
#[test]
fn the_gpu_judge_hands_back_the_self_verified_system_configuration_slots_of_each_device_like_the_pool_checker_on_every_state(
) {
    let test_name = "the_gpu_judge_hands_back_the_self_verified_system_configuration_slots_of_each_device_like_the_pool_checker_on_every_state";
    let flow =
        record_new_pool_with_both_system_configuration_slots_of_device_zero_lost_in_one_segment(
            "crash-judge-gpu-self-verified-system-configuration-slots",
        );
    let limit = expand_up_to();
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let judge_tables = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    })
    .expect("这条流的 GPU 判表建得出");
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        &flow,
        limit,
        &[
            StateExamination::SelfVerifiedSystemConfigurationSlotsOfEachDevice {
                slots_on_the_base_image: self_verified_system_configuration_slots_of_each_device(
                    &flow.base,
                ),
                device_newly_dead: 0,
            },
        ],
    ) else {
        return;
    };
    assert_eq!(
        comparison.states, judge_tables.state_count,
        "前提：对拍那一趟与这里的判表同一份计划（状态数）"
    );
    let slots_of_each_device = &comparison.examined[0];
    let states_with_a_new_dead_device_zero = slots_of_each_device.states_counted;
    let slot_mismatches = &slots_of_each_device.disagreements;
    eprintln!(
        "CRASH_JUDGE_GPU_SELF_VERIFIED_SLOTS flow={} states={} states_with_a_new_dead_device_zero={states_with_a_new_dead_device_zero} slot_mismatches={}",
        flow.name,
        comparison.states,
        slot_mismatches.state_count()
    );
    assert!(
        slot_mismatches.is_empty(),
        "流 {}：{} 个状态上逐盘自证槽数或判段那一判两边不同（前几个）：\n{}",
        flow.name,
        slot_mismatches.state_count(),
        slot_mismatches
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        states_with_a_new_dead_device_zero > 0,
        "流 {}：槽 0 撕裂、槽 1 清零落了的状态上盘 0 是新出现的死盘",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}
