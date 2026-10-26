//! 回退候选集的四条排除（`singlefs_core::mount::RollbackCandidateExclusion`，D23（journal 的角色与格式） 已定项 14「候选集」：
//! 根环里有它 ∧ 按现行那一版的实例表判仍然有效 ∧ txg ≥ F_生效 ∧ 带文件）里，「不在根环里」与「树表 0 条」两条各一段固定历史，
//! 最后一步回退被拒、报的就是那一条，理想模型要求拒、理由对上（C493（回退候选集条文与实现说反话） 欠的判别力自证）。
//! 另两条各自的用例在 `random_histories.rs`：低于 F_生效 的是 `rolling_back_below_the_effective_floor_is_refused_for_being_below_the_floor`，
//! 被抛弃时间线上的是 `rolling_back_to_a_root_crash_recovery_abandoned_without_a_system_configuration_witness_is_refused_on_the_abandoned_timeline`。
//! 一段历史每一步接着上一步的盘面，不切片并行。
//! 「树表 0 条」另有一条不走历史的用例：那一版的实例表读坏时照样报这一条，钉住它由候选集那一判报、不靠后面重建那一版时的同一个结论。

mod common;

use common::{
    parameters, with_unreadable_ranges, FailingReadsOfARange, SharedUnreadableRanges,
    UnreadableRange, UnreadableRangeReadBack, FILE_BYTES, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::mount::{
    mount_writable, roll_back_by_a_forward_publish, RollbackCandidateExclusion, RollbackError,
    RollbackTarget,
};
use singlefs_core::recovery::{instance_table_chain_of_root, tree_table_has_no_entries};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter,
};
use singlefs_format::{DATA_UNIT_BYTES, SLOT_BYTES};
use singlefs_harness::history::{
    execute_history_with, GeneratedHistory, HistoryEnding, HistoryExecution, HistoryOperation,
    HistoryRun, HistorySeed, HistoryStartingPoint, RollbackTargetChoice, StepOutcome,
};
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::SharedStream;

fn execute_history_without_the_pool_checker(history: &GeneratedHistory) -> HistoryRun {
    execute_history_with(
        history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    )
}

/// 第一个文件之后可写挂载（实例 2 写行、暖机），挂着的时候回退到最新那条根的实例、txg 比最新的大 1 的目标：根环里没有它，
/// 报「不在根环里」，模型要求拒、理由对上。
/// 判别力：回退找目标那一步只比实例、不比 txg（拿掉「根环里有它」那一判）时，目标落到同实例的一条根上，不再报这一条。
#[test]
fn rolling_back_to_a_txg_beyond_the_newest_root_is_refused_for_not_being_in_the_ring() {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::BeyondNewestRoot {
                txg_beyond_newest: 0,
            }),
        ],
    };
    let run = execute_history_without_the_pool_checker(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert_eq!(
        run.outcomes[1],
        StepOutcome::Refused {
            member: "RollbackError::TargetNotACandidate(NotInRing)".to_string()
        },
        "根环里没有这个 (实例, txg)"
    );
    assert_eq!(
        run.tally.model_counts.required_refusals_matched, 1,
        "模型要求拒、理由对上"
    );
}

/// 第一个文件（txg 3）之后可写挂载（实例 2 写行 txg 4、暖机 txg 5），挂着的时候回退到根环从新到旧第 4 条：第一个文件之前那条
/// 树表 0 条的根。它在根环里、F 是 0、没有被抛弃，只因那一版还没写过文件被挡，报「树表 0 条」，模型要求拒、理由对上。
/// 这一格判不出拿掉「带文件」那一判：拿掉之后目标进了候选集，后面重建那一版交回「树表 0 条」，报的是同一条。
/// 判别力在 `rolling_back_to_a_version_without_file_whose_instance_table_reads_bad_is_refused_for_a_version_without_file`。
#[test]
fn rolling_back_to_the_root_make_filesystem_wrote_before_any_file_is_refused_for_a_version_without_file(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 3,
            }),
        ],
    };
    let run = execute_history_without_the_pool_checker(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert_eq!(
        run.outcomes[1],
        StepOutcome::Refused {
            member: "RollbackError::TargetNotACandidate(VersionWithoutFile)".to_string()
        },
        "建池时那条根在根环里、不低于 F、没被抛弃，只是那一版树表 0 条"
    );
    assert_eq!(
        run.tally.model_counts.required_refusals_matched, 1,
        "模型要求拒、理由对上"
    );
}

/// 新池新建文件之后可写挂载（实例 2 写行、暖机），挂着的时候回退到暖机那条树表 0 条的根（txg 2），那一版的实例表两份在每次读上都读坏。
/// 候选集那一判只读目标那一版的树表（`recovery::tree_table_has_no_entries`），树表 0 条就报「树表 0 条」，不去读那一版的实例表。
/// 判别力（C493 那一行变异在历史用例上没红的取样点）：拿掉「带文件」那一判时目标进了候选集，后面重建那一版
/// （`recovery::rebuild_version_in_the_unit_area_starting_at` 先读实例表、树表）读实例表读坏，报「目标那一版读不出」，不再报这一条；
/// 实例表读得出时重建交回「树表 0 条」、报同一条，历史用例那一格分不出来。
#[test]
fn rolling_back_to_a_version_without_file_whose_instance_table_reads_bad_is_refused_for_a_version_without_file(
) {
    let pool_parameters = parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                )
            })
            .collect();
    let genesis = make_filesystem(&pool_parameters, &mut devices).expect("mkfs");
    let mut allocator_after_the_make_filesystem =
        allocator_after_make_filesystem(&pool_parameters, &devices, &genesis);
    let content: Vec<u8> = (0..FILE_BYTES)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect();
    let target_root = {
        let mut writer = PoolWriter::new(&pool_parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号 1");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let last_warm_up_root = *warmed.roots.last().expect("暖机写了两条根");
        publish_first_file(
            &mut writer,
            &mut allocator_after_the_make_filesystem,
            &last_warm_up_root,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件");
        last_warm_up_root
    };
    let mounted = mount_writable(&pool_parameters, &mut devices).expect("可写挂载");
    let mut allocator_of_the_writable_mount = mounted.allocator;
    let mut current = mounted
        .current
        .into_file_version()
        .expect("新池新建文件之后每一版都带文件");
    let unreadable_ranges = SharedUnreadableRanges::new(
        target_root
            .instance_table
            .locations
            .iter()
            .map(|location| UnreadableRange {
                device: location.device,
                offset_in_bytes: location.slot.0 * SLOT_BYTES,
                length_in_bytes: DATA_UNIT_BYTES,
                failing_reads: FailingReadsOfARange::Every,
            })
            .collect(),
        UnreadableRangeReadBack::DeviceError,
    );
    let mut devices = with_unreadable_ranges(devices, &unreadable_ranges);
    assert!(
        instance_table_chain_of_root(&devices, &target_root).is_err(),
        "前提：目标那一版的实例表读坏"
    );
    assert!(
        instance_table_chain_of_root(&devices, &current.root).is_ok(),
        "前提：现行那一版的实例表不是那两份、读得出"
    );
    assert_eq!(
        tree_table_has_no_entries(&devices, &target_root),
        Ok(true),
        "前提：目标那一版树表读得出、0 条"
    );
    let refused = roll_back_by_a_forward_publish(
        &pool_parameters,
        &mut devices,
        &mut allocator_of_the_writable_mount,
        &mut current,
        RollbackTarget {
            instance: target_root.instance,
            checkpoint_txg: target_root.checkpoint_txg,
        },
    );
    assert!(
        matches!(
            refused,
            Err(RollbackError::TargetNotACandidate {
                exclusion: RollbackCandidateExclusion::VersionWithoutFile,
                ..
            })
        ),
        "暖机那条根树表 0 条：候选集那一判报「树表 0 条」，不读它的实例表；实际 {refused:?}"
    );
}
