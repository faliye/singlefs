//! 里程碑「第二个事务」增补 2 收口表第 5 行转来的 C518（一次挂载之内环转过一圈之后不回收） 与 C517（固定点分配被拒之后同一进程里写不出去）：
//! D16（发布语义） 已定项 1 的可再分配谓词（已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)）在一次挂载之内照样兑现——
//! 这个进程写的根盖掉环里最旧的有效根之后，谓词放行的槽在同一次挂载里回到空闲、下一次分配拿得到，不用重挂。
//! 分配器那张根环表（`allocator::RootRingOccupancy`）挂载时从盘上读、mkfs 同一个进程那条会话里按 mkfs 刚写下的样子装，
//! 之后这个进程每写一条根记一条。

mod common;

use common::{build_pool, format_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DataUnitIndexInFile, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::TREE_TABLE_GENESIS_SLOT;
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, publish_without_units, FirstFile, PoolVersion,
    PoolWriter, TransactionOutput, TransactionUnit, ZeroUnitPublishPlan,
};
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, FloorTargetChoice, GeneratedHistory,
    HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation, HistoryRun, HistorySeed,
    HistoryStartingPoint, PerStepChecker, StepOutcome,
};
use singlefs_harness::SharedStream;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn overwrite_in_process(pool: &mut BuiltPool, instance: InstanceGeneration) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let seed = usize::try_from(previous.root.checkpoint_txg.0).expect("txg 很小");
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content: &content_of(2000 + seed, seed),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// 镜像上判红的不变量（判不适用的不算）。
fn violated_invariants(pool: &BuiltPool) -> Vec<&'static str> {
    check_pool_image(&pool.memory_pool())
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| invariant)
        .collect()
}

/// C518 的验收：第一个事务（txg 3）之后可写挂载一次（实例 2：写行 txg 4、暖机 txg 5），同一次挂载里连着覆盖写（txg 6 起）。
/// 根环 R × S = 24 槽：txg 24、25、26 依次盖掉第 0 代根与两条暖机根。txg 2 的暖机根是最后一条引用 mkfs 那片第 0 版树表
/// （50178，1 槽，第一个文件版本在 txg 3 换下）的根；txg 26 盖掉它之后环里最旧有效根是 3，那一槽按谓词回收——
/// 同一次发布的记账行就按回收之后的数写，checker 的 I-3.1（已分配统计对得上） 在这一版上成立（修之前这一步判红）；
/// txg 27 的数据单元（用户数据取最低的两槽空位）落回 50178。
#[test]
fn turning_the_root_ring_in_one_writable_mount_reclaims_what_the_predicate_releases_and_the_next_publish_takes_it(
) {
    let mut pool = build_pool("root-ring-turn-in-one-mount");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("第一个文件之后重开，现行那一版带文件");
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(5),
        "写行 txg 4、暖机 txg 5"
    );
    while pool.output.root.checkpoint_txg < CheckpointTxg(25) {
        overwrite_in_process(&mut pool, InstanceGeneration(2));
    }
    for device_map in &pool.allocator.devices {
        assert!(
            !device_map.is_free(TREE_TABLE_GENESIS_SLOT),
            "盘 {:?}：txg 25 之后 txg 2 的暖机根还在环里、还引用着 mkfs 那片树表",
            device_map.device
        );
    }
    let deferred_before_txg_26: Vec<u64> = pool
        .allocator
        .devices
        .iter()
        .map(|device_map| device_map.deferred_slots())
        .collect();

    let txg_26 = overwrite_in_process(&mut pool, InstanceGeneration(2));
    assert_eq!(txg_26.root.checkpoint_txg, CheckpointTxg(26));
    let released_by_txg_26: u64 = txg_26.released.iter().map(|placement| placement.span).sum();
    for (device_map, deferred_before) in pool.allocator.devices.iter().zip(&deferred_before_txg_26)
    {
        assert!(
            device_map.is_free(TREE_TABLE_GENESIS_SLOT),
            "盘 {:?}：txg 26 盖掉 txg 2 之后 mkfs 那片树表按谓词回收，同一次挂载里就回到空闲",
            device_map.device
        );
        assert_eq!(
            device_map.deferred_slots() + 1,
            deferred_before + released_by_txg_26,
            "盘 {:?}：defer 队列加上 txg 26 换下的，减去回收的那 1 槽",
            device_map.device
        );
    }
    assert_eq!(
        violated_invariants(&pool),
        Vec::<&str>::new(),
        "txg 26 这一版的记账按回收之后的数写：checker 一条都不红"
    );

    let txg_27 = overwrite_in_process(&mut pool, InstanceGeneration(2));
    assert_eq!(
        txg_27
            .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST))
            .slot,
        TREE_TABLE_GENESIS_SLOT,
        "回收回来的那一槽与旁边从没分配过的 50179 是全池最低的两槽空位：txg 27 的数据单元落在那里"
    );
    assert_eq!(violated_invariants(&pool), Vec::<&str>::new());
}

/// 只做过 mkfs 的池可写挂载（实例 1：写行与暖机都是零单元发布，txg 1、2）之后，调用方自己又发一次零单元发布（txg 3，不经分配器），
/// 再在它上面发第一个文件版本（txg 4）：`publish_first_file` 先把 txg 3 那条根补记进分配器的根环表
/// （`PoolAllocator::record_zero_unit_roots_leading_to`），表照旧跟得上；不补记，记到 txg 4 时表判出跳号、断言失败。
#[test]
fn a_zero_unit_publish_the_caller_issued_before_the_first_file_version_is_recorded_into_the_root_ring(
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
