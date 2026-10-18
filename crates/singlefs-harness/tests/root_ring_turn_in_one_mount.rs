//! 里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 5 行转来的 C518（一次挂载之内环转过一圈之后不回收） 与 C517（固定点分配被拒之后同一进程里写不出去）：
//! D16（发布语义） 已定项 1 的可再分配谓词（已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)）在一次挂载之内照样兑现——
//! 这个进程写的根盖掉环里最旧的有效根之后，谓词放行的槽在同一次挂载里回到空闲、下一次分配拿得到，不用重挂。
//! 分配器那张根环表（`allocator::RootRingOccupancy`）挂载时从盘上读、mkfs 同一个进程那条会话里按 mkfs 刚写下的样子装，
//! 之后这个进程每写一条根记一条。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/root_ring_turn_in_one_mount.rs"]
mod scenarios_of_root_ring_turn_in_one_mount;

use common::{format_pool, parameters, FIXED_WRITE_TIME_SECONDS};
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_root_ring_turn_in_one_mount::content_of;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::journal::back_chain_of;
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{
    publish_first_file, publish_without_units, FirstFile, PoolVersion, PoolWriter,
    ZeroUnitPublishPlan,
};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::history::{
    execute_history_with, AppliedEffect, ContentChoice, ContentLength, FloorTargetChoice,
    GeneratedHistory, HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation,
    HistoryRun, HistorySeed, HistoryStartingPoint, PerStepChecker, StepOutcome, StepPosition,
};
use singlefs_harness::{RecordedOperation, RecordedOperationKind, SharedStream};
use std::collections::{BTreeMap, BTreeSet};

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_root_ring_turn_in_one_mount::turning_the_root_ring_in_one_writable_mount_reclaims_what_the_predicate_releases_and_the_next_publish_takes_it`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn turning_the_root_ring_in_one_writable_mount_reclaims_what_the_predicate_releases_and_the_next_publish_takes_it(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_root_ring_turn_in_one_mount::turning_the_root_ring_in_one_writable_mount_reclaims_what_the_predicate_releases_and_the_next_publish_takes_it,
    );
}

/// 只做过 mkfs 的池可写挂载（实例 1：写行与暖机都是零单元发布，txg 1、2）之后，调用方自己又发一次零单元发布（txg 3，不经分配器），
/// 再在它上面发第一个文件版本（txg 4）：`publish_first_file` 先把 txg 3 那条根补记进分配器的根环表
/// （`PoolAllocator::record_zero_unit_roots_leading_to`），表照旧跟得上；不补记，记到 txg 4 时表判出跳号、断言失败。
#[test]
fn zero_unit_publish_the_caller_issued_before_the_first_file_version_is_recorded_into_the_root_ring(
) {
    let mut formatted = format_pool("root-ring-records-a-caller-zero-unit-publish");
    let mut devices = formatted.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("只做过 mkfs 的池可写挂载");
    formatted.devices = Some(devices);
    let mut allocator = mounted.allocator;
    let PoolVersion::WithoutFile(warm_up) = mounted.current else {
        panic!("树表 0 条：现行那一版没有文件");
    };
    assert_eq!(warm_up.root.checkpoint_txg, CheckpointTxg(2));
    let publish_parameters = parameters();
    let open_devices = formatted.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
    let zero_unit = publish_without_units(
        &mut writer,
        &warm_up.root,
        ZeroUnitPublishPlan {
            txg: CheckpointTxg(3),
            counter: warm_up.record.counter + 1,
            instance: InstanceGeneration(1),
            back_chain: back_chain_of(&warm_up.record_bytes),
            rollback_floor: warm_up.root.rollback_floor,
            tree_identifier_watermark: warm_up.root.tree_identifier_watermark,
        },
    )
    .expect("调用方自己发的零单元发布");
    let first = publish_first_file(
        &mut writer,
        &mut allocator,
        &zero_unit.root,
        FirstFile {
            content: &content_of(3000, 1),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        },
        InstanceGeneration(1),
        &zero_unit.record_bytes,
    )
    .expect("第一个文件版本：txg 3 那条根先补记进根环表，记 txg 4 时不跳号");
    assert_eq!(first.root.checkpoint_txg, CheckpointTxg(4));
    assert!(
        allocator.root_ring_occupancy().is_some(),
        "挂载装了根环表：补记与跳号断言都作用在它上面"
    );
}

/// 覆盖写的内容：长度与填充都固定，同一段历史逐字节复现。
const OVERWRITE_IN_THE_SMALL_POOL: HistoryOperation =
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 2999 },
        fill_seed: 1,
    });

/// 两块单元区 384 槽的小盘上跑一段历史（起点：mkfs 同一个进程里发完第一个文件），不跑池级 checker——这里只比每一步入口的结局，
/// 转环之后的记账由 `turning_the_root_ring_in_one_writable_mount_reclaims_what_the_predicate_releases_and_the_next_publish_takes_it`
/// 在 4 GiB 的盘上跑 checker 判。空间准入关掉（只供测试的开关 `SpaceAdmission::SkippedByTheTestOnlySwitch`）：
/// 判着准入时这块小盘上式子在挂载之后第 11 次覆盖写就拒（D28（挂载期承诺量） 已定项 1 的式子扣切换预留与保留池、defer 按读法甲扣两次），
/// 走不到这里要比的「根环转过之后落点取不到」那一格。
fn run_on_the_small_pool(operations: Vec<HistoryOperation>) -> HistoryRun {
    run_on_the_small_pool_keeping_the_stream(operations).run
}

/// [`run_on_the_small_pool`] 连同录制流：每一步操作之后流里有几步，按操作的次序（第 0 个是起点之后）。
struct SmallPoolRunWithItsStream {
    run: HistoryRun,
    stream_operations: Vec<RecordedOperation>,
    stream_length_after_the_starting_point_and_each_operation: Vec<usize>,
}

fn run_on_the_small_pool_keeping_the_stream(
    operations: Vec<HistoryOperation>,
) -> SmallPoolRunWithItsStream {
    let stream = SharedStream::new();
    let stream_seen_by_the_observer = stream.clone();
    let mut stream_length_after_the_starting_point_and_each_operation: Vec<usize> = Vec::new();
    let run = execute_history_with(
        &GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations,
        },
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
            space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
        },
        &stream,
        &mut |observation| {
            let expected_position = match observation.position {
                StepPosition::StartingPoint => 0,
                StepPosition::Operation(operation_index) => operation_index + 1,
            };
            assert_eq!(
                stream_length_after_the_starting_point_and_each_operation.len(),
                expected_position,
                "观察者按次序见到起点与每一步"
            );
            stream_length_after_the_starting_point_and_each_operation
                .push(stream_seen_by_the_observer.operation_count());
        },
    );
    SmallPoolRunWithItsStream {
        run,
        stream_operations: stream.operations(),
        stream_length_after_the_starting_point_and_each_operation,
    }
}

/// 小盘单元区的起始槽号：小盘那条 6 MiB journal 环末尾的下一个槽。
fn unit_area_start_slot_of_the_small_pool() -> u64 {
    UnitAreaStart::following_the_journal_ring(
        HistoryDeviceWidth::UnitAreaOf384Slots
            .parameters()
            .geometry
            .journal_ring_bytes,
    )
    .expect("小盘的 6 MiB 环末尾的下一个槽落在聚簇段边界上")
    .slot()
    .0
}

/// 小盘每块盘的单元区槽数：盘宽减去单元区起点，即环窗口补足（`singlefs_harness::history` 的 `SMALL_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS`）
/// 加名字里的 384 槽。环窗口补足变了，这个数跟着变，下面推出来的覆盖写次数也跟着变。
fn unit_area_slots_on_each_device_of_the_small_pool() -> u64 {
    HistoryDeviceWidth::UnitAreaOf384Slots.device_bytes() / SLOT_BYTES
        - unit_area_start_slot_of_the_small_pool()
}

/// 一次覆盖写（[`OVERWRITE_IN_THE_SMALL_POOL`]）每块盘至少新落几槽（数据单元与它改到的树节点、分配记录树叶）：只用来推下面的次数；
/// 推少了，用例里「这一段新落的槽数多过单元区」那条前提先红，不会悄悄失敏。
const SLOTS_EACH_OVERWRITE_PLACES_ON_EACH_DEVICE_AT_LEAST: u64 = 10;

/// 同一次挂载里、不夹抬 F 连着覆盖写几次：单元区槽数 ÷ 每次至少新落的槽数再多一次，这一段新落的槽数就多过单元区。
fn overwrites_past_the_wall_without_reclaim_in_the_mount() -> usize {
    usize::try_from(
        unit_area_slots_on_each_device_of_the_small_pool()
            / SLOTS_EACH_OVERWRITE_PLACES_ON_EACH_DEVICE_AT_LEAST
            + 1,
    )
    .expect("覆盖写次数装得进 usize")
}

/// 录制流里 `[stream_lengths 相邻两项)` 那几段（一段一步操作）每块盘写进单元区的槽：一段之内去重，段与段之间相加。
/// 不经挂载内回收，同一次挂载里新落的槽只能取挂载那一刻空着的槽、每槽至多落一次，所以这个和不超过单元区槽数。
fn unit_area_slots_placed_step_by_step_on_each_device(
    stream_operations: &[RecordedOperation],
    stream_lengths: &[usize],
) -> BTreeMap<DeviceIdentity, u64> {
    let unit_area_start_slot = unit_area_start_slot_of_the_small_pool();
    let mut placed_on_each_device: BTreeMap<DeviceIdentity, u64> = BTreeMap::new();
    for step_range in stream_lengths.windows(2) {
        let mut slots_written_in_this_step: BTreeSet<(DeviceIdentity, u64)> = BTreeSet::new();
        for operation in &stream_operations[step_range[0]..step_range[1]] {
            match operation.kind {
                RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {}
                RecordedOperationKind::WriteZeroes | RecordedOperationKind::Barrier => continue,
            }
            let first_slot = operation.offset.0 / SLOT_BYTES;
            if first_slot < unit_area_start_slot || operation.length == 0 {
                continue;
            }
            let last_slot = (operation.offset.0 + operation.length - 1) / SLOT_BYTES;
            slots_written_in_this_step
                .extend((first_slot..=last_slot).map(|slot| (operation.device, slot)));
        }
        for (device, _) in slots_written_in_this_step {
            *placed_on_each_device.entry(device).or_insert(0) += 1;
        }
    }
    placed_on_each_device
}

fn outcome_names(run: &HistoryRun) -> Vec<String> {
    run.outcomes
        .iter()
        .map(|outcome| match outcome {
            StepOutcome::Applied(_) => "Applied".to_string(),
            StepOutcome::Refused { member } => format!("Refused({member})"),
            StepOutcome::NotApplicable(missing) => format!("NotApplicable({missing:?})"),
        })
        .collect()
}

/// 可写挂载之后同一次挂载里连着覆盖写 37 次，再抬一次 F（目标取现行的 F）。修之前、小盘还没有环窗口补足时，根环转过之后谓词放行的槽挂载内不回收：
/// 第 37 次覆盖写的数据单元就拿不到落点，接着那次抬 F 的空发布连固定点都拿不到（`PlacementRefused`，每块盘上都没有）——
/// C517（固定点分配被拒之后同一进程里写不出去） 那一格从这里起（这几个结局在报告里的基线副本上逐步量过）。
/// 单元区放大了环窗口补足之后，37 次覆盖写离挂载内不回收时的墙还远，抬 F 的回收又把没回收的补上：挂载内回收由
/// [`overwrites_past_the_wall_without_reclaim_in_the_mount`] 那一段长覆盖写判。
fn up_to_the_raise_that_was_refused_its_fixed_points_before_the_fix() -> Vec<HistoryOperation> {
    std::iter::once(HistoryOperation::CloseAndMountWritable)
        .chain(std::iter::repeat_n(OVERWRITE_IN_THE_SMALL_POOL, 37))
        .chain([HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
            steps_above_current_floor: 0,
        })])
        .collect()
}

/// C517 的验收：同一份盘面上，重挂之后发得出去的那次覆盖写，不重挂也发得出去；不重挂的那一路接着再发十次也都发得出去。
/// 修之前重挂那一路发得出去（重建分配器按谓词回收），不重挂那一路从第 37 次覆盖写起每一步都被同一个拒绝挡住。
/// 再从挂载起、不夹抬 F 连着覆盖写 [`overwrites_past_the_wall_without_reclaim_in_the_mount`] 次：这一段新落的槽多过单元区，
/// 挂载内不回收时一定撞墙，每一次都发得出去只能靠挂载内回收。
#[test]
fn an_overwrite_that_goes_out_after_a_remount_goes_out_in_the_same_mount_as_well() {
    let prefix = up_to_the_raise_that_was_refused_its_fixed_points_before_the_fix();

    let after_a_remount = run_on_the_small_pool(
        prefix
            .iter()
            .copied()
            .chain([
                HistoryOperation::CloseAndMountWritable,
                OVERWRITE_IN_THE_SMALL_POOL,
            ])
            .collect(),
    );
    let names_after_a_remount = outcome_names(&after_a_remount);
    assert_eq!(
        names_after_a_remount.last().map(String::as_str),
        Some("Applied"),
        "重挂之后那次覆盖写发得出去：{names_after_a_remount:?}"
    );

    let in_the_same_mount = run_on_the_small_pool(
        prefix
            .iter()
            .copied()
            .chain(std::iter::repeat_n(OVERWRITE_IN_THE_SMALL_POOL, 11))
            .collect(),
    );
    let names_in_the_same_mount = outcome_names(&in_the_same_mount);
    assert_eq!(
        names_in_the_same_mount[prefix.len()..],
        vec!["Applied".to_string(); 11],
        "不重挂：同一次覆盖写发得出去，接着再发十次也都发得出去：{names_in_the_same_mount:?}"
    );
    assert_eq!(
        names_in_the_same_mount[..prefix.len()],
        vec!["Applied".to_string(); prefix.len()],
        "挂载、37 次覆盖写与那次抬 F 也都做成：挂载内回收跟得上，这块小盘上根本撞不到墙"
    );
    assert_eq!(in_the_same_mount.ending, HistoryEnding::Completed);

    let overwrites_past_the_wall = overwrites_past_the_wall_without_reclaim_in_the_mount();
    let long_stretch = run_on_the_small_pool_keeping_the_stream(
        std::iter::once(HistoryOperation::CloseAndMountWritable)
            .chain(std::iter::repeat_n(
                OVERWRITE_IN_THE_SMALL_POOL,
                overwrites_past_the_wall,
            ))
            .collect(),
    );
    let names_of_the_long_stretch = outcome_names(&long_stretch.run);
    assert_eq!(
        names_of_the_long_stretch,
        vec!["Applied".to_string(); 1 + overwrites_past_the_wall],
        "挂载之后不夹抬 F 连着覆盖写 {overwrites_past_the_wall} 次，每一次都发得出去：{names_of_the_long_stretch:?}"
    );
    assert_eq!(long_stretch.run.ending, HistoryEnding::Completed);
    for (overwrite_index, outcome) in long_stretch.run.outcomes[1..].iter().enumerate() {
        assert!(
            matches!(
                outcome,
                StepOutcome::Applied(AppliedEffect::Published {
                    floor_raises_pushed_by_the_session: 0,
                    ..
                })
            ),
            "第 {overwrite_index} 次覆盖写发布之前会话没推抬 F（抬 F 的回收会把挂载内回收补上）：{outcome:?}"
        );
    }
    let placed_on_each_device = unit_area_slots_placed_step_by_step_on_each_device(
        &long_stretch.stream_operations,
        &long_stretch.stream_length_after_the_starting_point_and_each_operation[1..],
    );
    let unit_area_slots = unit_area_slots_on_each_device_of_the_small_pool();
    assert!(
        placed_on_each_device.len() == 2
            && placed_on_each_device
                .values()
                .all(|placed| *placed > unit_area_slots),
        "前提：挂载之后这一段每块盘新落的槽（逐步去重再相加）多过单元区的 {unit_area_slots} 槽，挂载内不回收时走不完：{placed_on_each_device:?}"
    );
}
