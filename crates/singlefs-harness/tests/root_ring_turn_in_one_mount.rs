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
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::journal::back_chain_of;
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{
    publish_first_file, publish_without_units, FirstFile, PoolVersion, PoolWriter,
    ZeroUnitPublishPlan,
};
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, FloorTargetChoice, GeneratedHistory,
    HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation, HistoryRun, HistorySeed,
    HistoryStartingPoint, PerStepChecker, StepOutcome,
};
use singlefs_harness::SharedStream;

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
    execute_history_with(
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
        &SharedStream::new(),
        &mut |_| {},
    )
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

/// 可写挂载之后同一次挂载里连着覆盖写 37 次，再抬一次 F（目标取现行的 F）。修之前，这块小盘上根环转过之后谓词放行的槽挂载内不回收：
/// 第 37 次覆盖写的数据单元就拿不到落点，接着那次抬 F 的空发布连固定点都拿不到（`PlacementRefused`，每块盘上都没有）——
/// C517（固定点分配被拒之后同一进程里写不出去） 那一格从这里起（这几个结局在报告里的基线副本上逐步量过）。
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
}
