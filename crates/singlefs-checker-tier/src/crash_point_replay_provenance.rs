//! 崩溃点重放各条流的来历表与按来历统一挂的观察者（G4 第三轮判决丙-置，用户 2026-10-22 定「按流登记+相对基镜像」；
//! 主 agent 对 G4 实现员停下的五问的定案：来历登在一张独立的表里，观察者按来历在跑流的那一处统一挂，不在各设点文件里一份一份挂）。
//!
//! 跑流的两处都经这里挂：崩溃枚举用例的枚举入口（`crate::crash_state_enumeration` 的 `enumerate_crash_states*`，名字是崩溃点重放项名
//! `CrashPointReplayItem.name`）与崩溃放量（`crate::crash_amplification::run_from_environment_with_observers`，名字是流名）。两处按
//! [`observers_on_the_nodes_with_the_ones_attached_to_the_flow`] 查来历表：查不到、查到不止一行都在录任何东西之前拒（这一项整条红），
//! 查到的那一行定挂哪几个观察者（[`observers_attached_by_the_provenance`]）。
//!
//! 表里一行是一条流：项名第一个 `::` 之前那一截（之后是档位、标签，同一条流），或放量的流名。项名与流名在测试里多半是现拼的
//! （`format!("<流>::quick_tier::{label}")`、`stream.flow_name()`），登记位上没有它们，所以名字的完整由跑的那一处兜底：没登的名字一跑就拒；
//! 每一行另写它在哪一处登记位上（崩溃点重放合表的项、崩溃放量入口 ITEMS 的分项，或两处都不登），登记完整性用例
//! （`tests/crash_points_observer_registration_is_complete.rs`）现读两处登记位，核每一项、每一分项在这张表里都有行、表里点名的项都还在登记位上。
//!
//! 挂哪几个：登记位上的流（崩溃点重放项与放量分项，GPU 判的崩溃态全域）里来历是 mkfs 返回之后的挂两槽全废盘面观察者，录 mkfs 自己的流与
//! mkfs 中途崩溃之后的挂载不挂。两处都不登的流（引擎、流水线与对拍自己的单测，实验装置）不挂：它们钉着自己挂了哪几个观察者、库里存了几块观察者结论。
//!
//! 两槽全废盘面观察者在判段判（用户 2026-10-22 定「你做你的 GPU版让GPU去做」）：录入段对它什么都不跑；判器逐状态交回逐盘自证过的系统配置槽数
//! （配了 GPU 是走读内核第 25 字，没配是 CPU 参照 `crate::crash_judge_conclusions::self_verified_system_configuration_slots_of_each_device`，
//! 池级 checker 的逐盘数），判段写结论块时连同基镜像那一份存进库，观察者的判段读回来对基镜像比
//! （`crate::crash_amplification::ObserverCode::FromTheSelfVerifiedSystemConfigurationSlotsTheJudgeHandsBack`）。
//!
//! 并集对走读观察者（F6）撤了（主 agent 2026-10-22 定）：GPU 恢复内核自己判三样读者的并集，它要核的那件事由 GPU 与 CPU 判器的逐状态对拍管。
//! 并集判法照留在这里（[`fall_back_of_the_state`]、[`disagreement_of_the_state`]），当 GPU 并集对拍的 CPU 参照（标 `cpu-reference-for-gpu-comparison`）。

use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::journal::{JournalRecord, JournalRecordPlaceInPublish};
use singlefs_core::mounted_read::{
    open_pool_for_read_judging_the_truncation_fields_as, OpenPoolForReadFailure,
};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration_naming_devices_without_a_mountable_slot,
    fall_back_judged_on_the_three_readers, instance_table_chain_of_root,
    newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to,
    rebuild_version_judging_the_truncation_fields_as, replay_journal, scan_journal,
    walk_to_file_judging_the_truncation_fields_as, FailureKindForTheFallBack,
    NamedUnitVerification, PoolReader, RebuildVersionFailure, RecoveryFailure,
    SystemConfigurationChoice,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_harness::sha256::DIGEST_BYTES;
use std::collections::BTreeMap;

use crate::crash::{CrashPointReplayObserverCounts, CrashPointReplayTally};
use crate::crash_amplification::{
    CrashFlow, NodeOfAnObserver, ObserveTheSelfVerifiedSystemConfigurationSlotsOfACrashState,
    ObserverCode, ObserverOnTheNodes, END_OF_FLOW_NODE_NAME,
};
use crate::crash_identity::ObserverName;
use crate::crash_judge_conclusions::devices_without_a_self_verified_system_configuration_slot_against_the_base_image_from_the_slot_counts;

/// 这一份的仓内路径：按来历挂的观察者的观察者代码文件（`crate::judging_units_entries::OBSERVER_ENTRIES` 按它登一行，目标 `lib`）。
pub const THIS_FILE: &str = "crates/singlefs-checker-tier/src/crash_point_replay_provenance.rs";

// ============================== 来历表 ==============================

/// 一条流从哪里录起（封闭集合：加一种就加一个成员，[`observers_attached_by_the_provenance`] 的 `match` 跟着补）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowProvenance {
    /// 基镜像是 mkfs 返回之后的池（或在它上面接着录过的池），录的是之后的历史：取号、暖机、发布、挂载、删除、整理……
    RecordedAfterTheMakeFilesystemReturned,
    /// 录的就是 mkfs 自己：基镜像是 mkfs 之前的盘面。
    RecordingTheMakeFilesystemItself,
    /// 基镜像是 mkfs 中途崩溃留下的盘面，录的是之后的挂载。
    MountAfterACrashInTheMiddleOfTheMakeFilesystem,
}

/// 跑这条流的是哪一处（两处的名字各是一个名字空间）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunOfTheFlow {
    /// 崩溃枚举用例的枚举入口（`crate::crash_state_enumeration` 的 `enumerate_crash_states*`）：名字是崩溃点重放项名第一个 `::` 之前那一截。
    CrashPointReplayEnumeration,
    /// 崩溃放量（`crate::crash_amplification::run_from_environment_with_observers`）：名字是流名。
    CrashAmplification,
}

/// 这条流在哪一处登记位上（登记完整性用例现读这两处）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistrationOfTheFlow {
    /// 崩溃点重放合表 `.claude/gate.d/crash-point-replay-items.tsv` 的一项（第一列项名）：它第二列点名的崩溃枚举用例跑的就是这条流。
    CrashPointReplayItemsTable(&'static str),
    /// 崩溃放量入口 `scripts/crash/crash-amplification.sh` 的 ITEMS 里一个分项（第一列分项名）。
    CrashAmplificationItems(&'static str),
    /// 两处都不登：引擎、流水线与对拍自己的单测，实验装置。不是崩溃点重放项也不是放量分项，不挂按来历挂的观察者。
    NeitherRegistry,
}

/// 来历表的一行：哪一处跑、跑的时候认的名字、登记在哪、来历。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProvenanceOfAFlow {
    pub run: RunOfTheFlow,
    /// 崩溃枚举用例是项名第一个 `::` 之前那一截（[`flow_name_of_the_name_at_the_run`]），崩溃放量是流名。
    pub flow_name: &'static str,
    pub registration: RegistrationOfTheFlow,
    pub provenance: FlowProvenance,
}

/// 写表用的简写：崩溃枚举用例跑的、登在合表上的一条流。
const fn enumeration_of_the_item(
    flow_name: &'static str,
    item_in_the_table: &'static str,
    provenance: FlowProvenance,
) -> ProvenanceOfAFlow {
    ProvenanceOfAFlow {
        run: RunOfTheFlow::CrashPointReplayEnumeration,
        flow_name,
        registration: RegistrationOfTheFlow::CrashPointReplayItemsTable(item_in_the_table),
        provenance,
    }
}

/// 写表用的简写：崩溃放量跑的、登在 ITEMS 上的一条流。
const fn amplification_of_the_item(
    flow_name: &'static str,
    item_in_the_registry: &'static str,
    provenance: FlowProvenance,
) -> ProvenanceOfAFlow {
    ProvenanceOfAFlow {
        run: RunOfTheFlow::CrashAmplification,
        flow_name,
        registration: RegistrationOfTheFlow::CrashAmplificationItems(item_in_the_registry),
        provenance,
    }
}

/// 写表用的简写：两处登记位都不登的一条流。
const fn unregistered(
    run: RunOfTheFlow,
    flow_name: &'static str,
    provenance: FlowProvenance,
) -> ProvenanceOfAFlow {
    ProvenanceOfAFlow {
        run,
        flow_name,
        registration: RegistrationOfTheFlow::NeitherRegistry,
        provenance,
    }
}

/// 来历表。一行一条流，（跑的那一处，名字）恰好一行；跑的那一处查不到、查到几行都拒。
pub const PROVENANCE_OF_THE_FLOWS: &[ProvenanceOfAFlow] = &[
    // 崩溃枚举用例：登在合表上的那几项（合表第二列的崩溃枚举用例跑的流）
    enumeration_of_the_item(
        "new_pool_file_creation",
        "new-pool-file-creation",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "fixed_script",
        "fixed-script-through-the-unmount",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "floor_raise_pushed_by_the_session",
        "floor-raise-pushed-by-the-session",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "multi_record_publish",
        "multi-record-publish",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "unlink_after_multi_record",
        "unlink-after-the-multi-record-publish",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "unlink_merging_inode_leaf_containers",
        "unlink-merging-inode-leaf-containers",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "unlink_after_the_fixed_script",
        "unlink-after-the-fixed-script",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "snapshot_creation",
        "snapshot-creation",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "overwrite_and_unlink_under_a_snapshot",
        "overwrite-and-unlink-under-a-snapshot",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "snapshot_destruction_resumed_after_a_crash",
        "snapshot-destruction-resumed-after-a-crash",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "compaction_relocation",
        "compaction-relocation",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "compaction_relocating_a_block_shared_with_a_snapshot",
        "compaction-relocating-a-block-shared-with-a-snapshot",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "deletion_interleaved_with_compaction_across_a_rollback",
        "deletion-interleaved-with-compaction-across-a-rollback",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "lifecycle_closure",
        "lifecycle-closure",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "unlink_under_a_snapshot",
        "unlink-under-a-snapshot",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "truncate_to_zero",
        "truncate-to-zero",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "truncate_into_a_unit_then_extend",
        "truncate-into-a-unit-then-extend",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "truncate_under_a_snapshot",
        "truncate-under-a-snapshot",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "truncate_an_inode_other_than_the_first_to_zero",
        "truncate-an-inode-other-than-the-first-to-zero",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "truncate_an_inode_other_than_the_first_into_a_unit_then_extend",
        "truncate-an-inode-other-than-the-first-into-a-unit-then-extend",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "truncate_an_inode_other_than_the_first_under_a_snapshot",
        "truncate-an-inode-other-than-the-first-under-a-snapshot",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "partial_write_in_the_middle_unit",
        "partial-write-in-the-middle-unit",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "partial_write_across_the_end_then_into_the_hole",
        "partial-write-across-the-end-then-into-the-hole",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "partial_write_under_a_snapshot",
        "partial-write-under-a-snapshot",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "batched_requests_not_yet_published",
        "batched-requests-not-yet-published",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "automatic_publish_interrupted",
        "automatic-publish-interrupted",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "tree_split",
        "tree-split-streams",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "position_addressed",
        "position-addressed-trees",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "io_min_8192_mount",
        "writable-mount-of-an-io-min-8192-pool",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "formatted_pool_remounted_without_a_file",
        "formatted-pool-remounted-without-a-file",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "make_filesystem_over_a_used_pool",
        "make-filesystem-over-a-used-pool",
        FlowProvenance::RecordingTheMakeFilesystemItself,
    ),
    enumeration_of_the_item(
        "crash_in_the_middle_of_the_acquisition_write",
        "crash-in-the-middle-of-the-acquisition-write",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "instance_table_sweep",
        "instance-table-sweep",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "one_file_overwritten_across_two_writable_remounts",
        "one-file-overwritten-across-two-writable-remounts",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "several_inodes_overwritten_in_turn",
        "several-inodes-overwritten-in-turn",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "large_file_overwritten_across_lower_leaves",
        "large-file-overwritten-across-lower-leaves",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "other_file_deleted_before_the_first_file",
        "unlink-of-two-files-holding-data",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    enumeration_of_the_item(
        "first_file_deleted_before_the_other_file",
        "unlink-of-two-files-holding-data",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    // 基镜像是 mkfs、第一个文件、崩了再挂、顺序写到贴边之后的盘面，录的是之后的截断三步
    enumeration_of_the_item(
        "truncate_to_zero_on_a_nearly_full_pool",
        "truncate-to-zero-on-a-nearly-full-pool",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    // 基镜像是实例 1 写完 A、B 之后的盘面，录的是实例 2 那次挂载与之后的覆盖写、建快照
    enumeration_of_the_item(
        "mounts_with_read_faults",
        "mounts-with-read-faults",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    // 基镜像是 A 被落到暖机根的崩溃恢复抛弃、实例 2 写行暖机之后的盘面，录的是再发第一个文件版本与之后一次覆盖写
    enumeration_of_the_item(
        "first_file_version_after_falling_back_to_a_warm_up_root",
        "first-file-version-after-falling-back-to-a-warm-up-root",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    // 崩溃枚举用例：两处都不登的（快档里的流与引擎、流水线、装置自己的）
    unregistered(
        RunOfTheFlow::CrashPointReplayEnumeration,
        "record_spill_over",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    unregistered(
        RunOfTheFlow::CrashPointReplayEnumeration,
        "acquisition_barrier",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    unregistered(
        RunOfTheFlow::CrashPointReplayEnumeration,
        "formatted_pool_mount",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    unregistered(
        RunOfTheFlow::CrashPointReplayEnumeration,
        "segments_per_device",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    unregistered(
        RunOfTheFlow::CrashPointReplayEnumeration,
        "findings_new_pool_file_creation_stream",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    unregistered(
        RunOfTheFlow::CrashPointReplayEnumeration,
        "crash_enumeration_gpu_without_the_feature",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    unregistered(
        RunOfTheFlow::CrashPointReplayEnumeration,
        "e161_crates_comparison",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    // 崩溃放量：登在 ITEMS 上的那几项
    amplification_of_the_item(
        "fixed-script-through-the-rollback",
        "fixed-script-through-rollback",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "fixed-script-through-the-unmount",
        "fixed-script-through-unmount",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "unlink-after-the-fixed-script",
        "unlink-after-the-fixed-script",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "unlink-merging-inode-leaf-containers",
        "unlink-merging-inode-leaf-containers",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "multi-record-publish",
        "multi-record-publish",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "unlink-after-the-multi-record-publish",
        "unlink-after-the-multi-record-publish",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "tree_split_CentralMappingRootSplit",
        "tree-split-streams",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "tree_split_CentralMappingLeafSplit",
        "tree-split-streams",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "tree_split_CentralMappingTwoLevelsSplitInARow",
        "tree-split-streams",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "tree_split_CentralMappingEmptyLeafDropped",
        "tree-split-streams",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "tree_split_CentralMappingRootLowered",
        "tree-split-streams",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "tree_split_AccountingRootSplit",
        "tree-split-streams",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "tree_split_AccountingLeafSplit",
        "tree-split-streams",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "position_addressed_ExtentInlineToLowerSegment",
        "position-addressed-trees",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "position_addressed_ExtentLowerSegmentBackToInline",
        "position-addressed-trees",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "position_addressed_ExtentLowerSegmentGrowsToTwoLevels",
        "position-addressed-trees",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "position_addressed_AllocationRecordTreeTwoLeavesPerDevice",
        "position-addressed-trees",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "position_addressed_VersionWithoutFileRowPublishWithItsOwnTree",
        "position-addressed-trees",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "floor-raise-pushed-by-the-session",
        "floor-raise-pushed-by-the-session",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "sigma-of-the-misaligned-reuse",
        "sigma-of-the-misaligned-reuse",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "compaction-relocation",
        "compaction-relocation",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "compaction-relocating-a-block-shared-with-a-snapshot",
        "compaction-relocating-a-block-shared-with-a-snapshot",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "lifecycle-closure",
        "lifecycle-closure",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "unlink-under-a-snapshot",
        "unlink-under-a-snapshot",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "truncate-to-zero",
        "truncate-to-zero",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "truncate-into-a-unit-then-extend",
        "truncate-into-a-unit-then-extend",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "truncate-under-a-snapshot",
        "truncate-under-a-snapshot",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "snapshot-creation",
        "snapshot-creation",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "overwrite-and-unlink-under-a-snapshot",
        "overwrite-and-unlink-under-a-snapshot",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "snapshot-destruction-resumed-after-a-crash",
        "snapshot-destruction-resumed-after-a-crash",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "one_file_overwritten_across_two_writable_remounts",
        "one-file-overwritten-across-two-writable-remounts",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "several_inodes_overwritten_in_turn",
        "several-inodes-overwritten-in-turn",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "large_file_overwritten_across_lower_leaves",
        "large-file-overwritten-across-lower-leaves",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "other_file_deleted_before_the_first_file",
        "unlink-of-two-files-holding-data",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    amplification_of_the_item(
        "first_file_deleted_before_the_other_file",
        "unlink-of-two-files-holding-data",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    // 与同名崩溃枚举用例同一段录法：贴边那一版落盘之后的盘面是起点，只录截断三步
    amplification_of_the_item(
        "truncate-to-zero-on-a-nearly-full-pool",
        "truncate-to-zero-on-a-nearly-full-pool",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    // 崩溃放量：两处都不登的（流水线、产品路径与 GPU 导入那几条单测）
    unregistered(
        RunOfTheFlow::CrashAmplification,
        "line-fields",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    unregistered(
        RunOfTheFlow::CrashAmplification,
        "stopped-reuse",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    unregistered(
        RunOfTheFlow::CrashAmplification,
        "import-reuse",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    unregistered(
        RunOfTheFlow::CrashAmplification,
        "refused-observer",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    unregistered(
        RunOfTheFlow::CrashAmplification,
        "import-planned",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    unregistered(
        RunOfTheFlow::CrashAmplification,
        "placement-reuse",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    unregistered(
        RunOfTheFlow::CrashAmplification,
        "gpu-without-the-feature",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    unregistered(
        RunOfTheFlow::CrashAmplification,
        "gpu-pipeline-import",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    unregistered(
        RunOfTheFlow::CrashAmplification,
        "gpu-observer-images",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    unregistered(
        RunOfTheFlow::CrashAmplification,
        "gpu-observer-rewriting-one-location",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    unregistered(
        RunOfTheFlow::CrashAmplification,
        "the_product_path_judges_on_scratch_it_booked_on_the_card_pool",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
    unregistered(
        RunOfTheFlow::CrashAmplification,
        "the_product_path_books_the_facts_verifier_buffers_on_the_card_pool",
        FlowProvenance::RecordedAfterTheMakeFilesystemReturned,
    ),
];

/// 跑的时候的名字对到来历表里的流名：项名第一个 `::` 之前那一截（之后是档位与标签），没有 `::` 的整个名字就是。
#[must_use]
pub fn flow_name_of_the_name_at_the_run(name_at_the_run: &str) -> &str {
    name_at_the_run
        .split("::")
        .next()
        .expect("split 至少交回一截")
}

/// 跑的那一处查来历表查不出一行。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProvenanceRefusal {
    /// 表里没有这条流：先在 [`PROVENANCE_OF_THE_FLOWS`] 登一行（写明它登在哪一处登记位上、来历是哪一种），再跑。
    NoRowForTheFlow {
        run: RunOfTheFlow,
        name_at_the_run: String,
    },
    /// 表里这条流有几行：（跑的那一处，流名）只许一行。
    SeveralRowsForTheFlow {
        run: RunOfTheFlow,
        name_at_the_run: String,
        rows: usize,
    },
    /// 调用方自己挂的观察者里有一个与按来历挂的同名：同名的两个观察者计数并成一份、分不开是谁看的。
    ObserverNamedLikeOneAttachedByTheProvenance { observer_name: String },
}

impl ProvenanceRefusal {
    #[must_use]
    pub fn sentence(&self) -> String {
        match self {
            Self::NoRowForTheFlow {
                run,
                name_at_the_run,
            } => format!(
                "来历表（crate::crash_point_replay_provenance::PROVENANCE_OF_THE_FLOWS）里没有 {run:?} 的流「{}」（名字 {name_at_the_run}）：\
                 先登一行，写明它登在哪一处登记位上、来历是哪一种，再跑",
                flow_name_of_the_name_at_the_run(name_at_the_run)
            ),
            Self::SeveralRowsForTheFlow {
                run,
                name_at_the_run,
                rows,
            } => format!(
                "来历表里 {run:?} 的流「{}」（名字 {name_at_the_run}）有 {rows} 行：一条流只许一行",
                flow_name_of_the_name_at_the_run(name_at_the_run)
            ),
            Self::ObserverNamedLikeOneAttachedByTheProvenance { observer_name } => format!(
                "调用方挂的观察者「{observer_name}」与按来历挂的观察者同名：按来历挂的由跑流的那一处挂，设点文件不再手挂"
            ),
        }
    }
}

/// 查来历表：（跑的那一处，名字对到的流名）恰好一行。
///
/// # Errors
/// 表里没有这条流，或有几行。
pub fn provenance_row_of_the_flow(
    run: RunOfTheFlow,
    name_at_the_run: &str,
) -> Result<&'static ProvenanceOfAFlow, ProvenanceRefusal> {
    let flow_name = flow_name_of_the_name_at_the_run(name_at_the_run);
    let rows: Vec<&'static ProvenanceOfAFlow> = PROVENANCE_OF_THE_FLOWS
        .iter()
        .filter(|row| row.run == run && row.flow_name == flow_name)
        .collect();
    match rows.as_slice() {
        [row] => Ok(row),
        [] => Err(ProvenanceRefusal::NoRowForTheFlow {
            run,
            name_at_the_run: name_at_the_run.to_string(),
        }),
        [_, _, ..] => Err(ProvenanceRefusal::SeveralRowsForTheFlow {
            run,
            name_at_the_run: name_at_the_run.to_string(),
            rows: rows.len(),
        }),
    }
}

// ============================== 按来历挂哪几个观察者 ==============================

/// 按来历挂的观察者（封闭集合：加一个就加一个成员，几处 `match` 跟着补）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverAttachedByTheProvenance {
    /// 两槽全废盘面观察者（[`observe_the_devices_without_a_self_verified_system_configuration_slot`]，判段看）。
    DevicesWithoutASelfVerifiedSystemConfigurationSlot,
}

impl ObserverAttachedByTheProvenance {
    /// 观察者名：进它那一组判法单元的组名（`OBSERVER_ENTRIES` 按 [`THIS_FILE`] 与这个名字各登一行）。
    #[must_use]
    pub fn observer_name(self) -> &'static str {
        match self {
            Self::DevicesWithoutASelfVerifiedSystemConfigurationSlot => {
                DEVICES_WITHOUT_A_SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOT_OBSERVER_NAME
            }
        }
    }

    /// 观察者的代码：入口就是这个 fn（`OBSERVER_ENTRIES` 那一行的入口）；挂上去是判段从判器交回的逐盘自证槽数看的那一种
    /// （`ObserverCode::FromTheSelfVerifiedSystemConfigurationSlotsTheJudgeHandsBack`）。
    pub fn observe(
        self,
    ) -> &'static ObserveTheSelfVerifiedSystemConfigurationSlotsOfACrashState<'static> {
        match self {
            Self::DevicesWithoutASelfVerifiedSystemConfigurationSlot => {
                &observe_the_devices_without_a_self_verified_system_configuration_slot
            }
        }
    }

    /// 看过的每个状态加一的那个计数（挂了它的流每个状态都看过：这个计数等于状态数）。
    #[must_use]
    pub fn states_seen_counter(self) -> &'static str {
        match self {
            Self::DevicesWithoutASelfVerifiedSystemConfigurationSlot => {
                SYSTEM_CONFIGURATION_SLOTS_JUDGED_AGAINST_THE_BASE_IMAGE_COUNTER
            }
        }
    }
}

/// 一条流按它登在哪、来历是哪一种挂哪几个观察者。登记位上的流（崩溃点重放项与放量分项）来历是 mkfs 返回之后的挂两槽全废盘面观察者
/// （mkfs 自己的流、mkfs 中途崩溃之后的挂载不挂：那两种盘面上一块盘两槽都不自证是允许的）。两处都不登的一个都不挂。
#[must_use]
pub fn observers_attached_by_the_provenance(
    registration: RegistrationOfTheFlow,
    provenance: FlowProvenance,
) -> Vec<ObserverAttachedByTheProvenance> {
    match registration {
        RegistrationOfTheFlow::NeitherRegistry => Vec::new(),
        RegistrationOfTheFlow::CrashPointReplayItemsTable(_)
        | RegistrationOfTheFlow::CrashAmplificationItems(_) => match provenance {
            FlowProvenance::RecordedAfterTheMakeFilesystemReturned => vec![
                ObserverAttachedByTheProvenance::DevicesWithoutASelfVerifiedSystemConfigurationSlot,
            ],
            FlowProvenance::RecordingTheMakeFilesystemItself
            | FlowProvenance::MountAfterACrashInTheMiddleOfTheMakeFilesystem => Vec::new(),
        },
    }
}

/// 跑的那一处要挂的那几个：查来历表（[`provenance_row_of_the_flow`]），按那一行定（[`observers_attached_by_the_provenance`]）。
///
/// # Errors
/// 同 [`provenance_row_of_the_flow`]。
pub fn observers_attached_to_the_flow(
    run: RunOfTheFlow,
    name_at_the_run: &str,
) -> Result<Vec<ObserverAttachedByTheProvenance>, ProvenanceRefusal> {
    let row = provenance_row_of_the_flow(run, name_at_the_run)?;
    Ok(observers_attached_by_the_provenance(
        row.registration,
        row.provenance,
    ))
}

/// 跑流的两处统一挂观察者的那一步：调用方自己挂的照原样在前，按来历挂的接在后面，挂在 `nodes` 上（崩溃枚举用例是它那个点与末节点，
/// 崩溃放量是流的每个点与末节点，[`nodes_of_every_point_of_the_flow`]）。观察者代码文件是 [`THIS_FILE`]、没有另读的输入。
///
/// # Errors
/// 同 [`provenance_row_of_the_flow`]；调用方挂的观察者与按来历挂的同名。
pub fn observers_on_the_nodes_with_the_ones_attached_to_the_flow<'observer>(
    run: RunOfTheFlow,
    name_at_the_run: &str,
    observers_of_the_caller: &[ObserverOnTheNodes<'observer>],
    nodes: &[NodeOfAnObserver],
) -> Result<Vec<ObserverOnTheNodes<'observer>>, ProvenanceRefusal> {
    let attached = observers_attached_to_the_flow(run, name_at_the_run)?;
    if let Some(clashing) = observers_of_the_caller.iter().find(|observer| {
        attached
            .iter()
            .any(|attached| attached.observer_name() == observer.observer_name.as_str())
    }) {
        return Err(
            ProvenanceRefusal::ObserverNamedLikeOneAttachedByTheProvenance {
                observer_name: clashing.observer_name.as_str().to_string(),
            },
        );
    }
    let mut observers = observers_of_the_caller.to_vec();
    observers.extend(
        attached.into_iter().map(|attached| ObserverOnTheNodes {
            observer_name: ObserverName::new(attached.observer_name())
                .expect("按来历挂的观察者名是写死的非空短串"),
            observer_code_file: THIS_FILE.to_string(),
            extra_inputs_digest: [0; DIGEST_BYTES],
            nodes: nodes.to_vec(),
            observe: ObserverCode::FromTheSelfVerifiedSystemConfigurationSlotsTheJudgeHandsBack(
                attached.observe(),
            ),
        }),
    );
    Ok(observers)
}

/// 崩溃放量那一路一条流的全部节点：每个崩溃点一个（设点文件与点名），加末节点（设点文件取最后一个点的，同 `crash_amplification` 排计划的取法）。
/// 流里一个崩溃点都没有时交回空的（排计划那一步自己停下，这里不抢先）。
#[must_use]
pub fn nodes_of_every_point_of_the_flow(flow: &CrashFlow<'_>) -> Vec<NodeOfAnObserver> {
    let Some(last_point) = flow.crash_points.last() else {
        return Vec::new();
    };
    flow.crash_points
        .iter()
        .map(|point| NodeOfAnObserver {
            setting_file: point.code_file_in_repository.clone(),
            entry_name: point.name.clone(),
        })
        .chain(std::iter::once(NodeOfAnObserver {
            setting_file: last_point.code_file_in_repository.clone(),
            entry_name: END_OF_FLOW_NODE_NAME.to_string(),
        }))
        .collect()
}

/// 崩溃枚举用例跑完之后核跑流那一处按来历挂上了：这条流在来历表里那一行要挂的观察者（至少一个）每个都看过每一个状态（看过的状态数计数
/// 等于状态数）。判红的状态整条用例已在判段收尾时停下，走到这里的一个都没红。设点文件只调这一句核，不手挂；只给登在登记位上、来历是
/// mkfs 返回之后的流调（别的流按来历一个都不挂）。
///
/// # Panics
/// 来历表里查不出这条流；这条流按来历一个都不挂；有一个按来历挂的观察者没看全。
pub fn assert_every_state_was_seen_by_the_observers_attached_to_the_flow(
    run: RunOfTheFlow,
    name_at_the_run: &str,
    tally: &CrashPointReplayTally,
) {
    let attached = observers_attached_to_the_flow(run, name_at_the_run)
        .unwrap_or_else(|refusal| panic!("{}", refusal.sentence()));
    assert!(
        !attached.is_empty(),
        "{run:?} 的流 {name_at_the_run} 按来历一个观察者都不挂：这一句只给登在登记位上、来历是 mkfs 返回之后的流核"
    );
    for observer in attached {
        assert_eq!(
            tally
                .observer_counts
                .count_named(observer.states_seen_counter()),
            tally.states,
            "按来历挂的观察者 {} 看过这条流的每一个状态：{:?}",
            observer.observer_name(),
            tally.observer_counts
        );
    }
}

// ============================== 两槽全废盘面观察者（G4 第三轮判决丙-置；用户 2026-10-22 定「按流登记+相对基镜像」与「你做你的 GPU版让GPU去做」） ==============================
//
// 判决 `research/prompts/m3-close-g4-rewrite-r3-main-verification.md` 第三节与「用户定案（2026-10-22，弹窗）」：无读故障的崩溃态上每块盘至少一份
// 自证过的系统配置，这一性质由崩溃点重放 mkfs 返回之后各条流上的观察者判（D13（验证路线） 已定项 6）。按来历挂，不按段认；撕裂域在内（判器逐状态交回
// 逐盘槽数，原地覆写的第三态也在）。判槽自证是池级 checker 的逐盘数：判器配了 GPU 是走读内核第 25 字，没配是 CPU 参照
// （`crate::crash_judge_conclusions::self_verified_system_configuration_slots_of_each_device`），不调核心层的解析。观察者在判段看，录入段对它什么都不跑。

/// 两槽全废盘面观察者的名字。
pub const DEVICES_WITHOUT_A_SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOT_OBSERVER_NAME: &str =
    "devices_without_a_self_verified_system_configuration_slot";

/// 这个观察者看过的状态数（每个状态加一，全池皆废的那几态也算）。
pub const SYSTEM_CONFIGURATION_SLOTS_JUDGED_AGAINST_THE_BASE_IMAGE_COUNTER: &str =
    "system_configuration_slots_judged_against_the_base_image";

/// 全池每块盘都一个自证过的系统配置槽都没有、留给 I-8.5 判的状态数（观察者不重记）。
pub const EVERY_DEVICE_WITHOUT_A_SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOT_COUNTER: &str =
    "every_device_without_a_self_verified_system_configuration_slot";

/// 一个崩溃状态上一个自证过的系统配置槽都没有的盘，相对这条流的基镜像怎样。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DevicesWithoutASelfVerifiedSystemConfigurationSlotAgainstTheBaseImage {
    /// 没有新出现的：崩溃镜像上这样的盘，基镜像上已经是这样（或一块都没有）。
    NoneNew,
    /// 池里每块盘都这样（全池皆废）：I-8.5 判红，观察者不重记。
    EveryDeviceLeftToTheTailSlotInvariant,
    /// 基镜像上还有自证过的槽、这个崩溃镜像上一个都没有的盘（盘号升序，池级 checker 读盘的次序）。
    New(Vec<u32>),
}

/// 两槽全废盘面观察者在判段看一个状态（`crate::crash_amplification::ObserverCode::FromTheSelfVerifiedSystemConfigurationSlotsTheJudgeHandsBack`）：
/// 判器交回的这个状态逐盘自证过的系统配置槽数对这条流基镜像那一份（判段写结论块时连同存下的）比
/// （`crate::crash_judge_conclusions::devices_without_a_self_verified_system_configuration_slot_against_the_base_image_from_the_slot_counts`），
/// 有新出现的、一个自证过的系统配置槽都没有的盘就判红（panic，判段接住记成红项）——mkfs 返回之后、没有读故障的历史留不下这份盘面
/// （正确实现在它上面可写挂载永远拒；G4 拒绝那一臂在崩溃注入里判同一形，`singlefs_harness::mount_transparency`）；基镜像上已经死的盘不记
/// （补写流的起点就有一块死盘）；全池皆废交 I-8.5、只数进它那一项。
/// 观察者的身份是这个 fn 的闭包键（`crate::judging_units_entries::OBSERVER_ENTRIES` 按 [`THIS_FILE`] 登一行）；没有另读的输入（基镜像那一份随判器交回的存）。
pub fn observe_the_devices_without_a_self_verified_system_configuration_slot(
    slots_of_each_device_on_the_crash_image: &[(u32, u32)],
    slots_of_each_device_on_the_base_image: &[(u32, u32)],
    counts: &mut CrashPointReplayObserverCounts,
) {
    counts.add(
        SYSTEM_CONFIGURATION_SLOTS_JUDGED_AGAINST_THE_BASE_IMAGE_COUNTER,
        1,
    );
    match devices_without_a_self_verified_system_configuration_slot_against_the_base_image_from_the_slot_counts(
        slots_of_each_device_on_the_crash_image,
        slots_of_each_device_on_the_base_image,
    ) {
        DevicesWithoutASelfVerifiedSystemConfigurationSlotAgainstTheBaseImage::NoneNew => {}
        DevicesWithoutASelfVerifiedSystemConfigurationSlotAgainstTheBaseImage::EveryDeviceLeftToTheTailSlotInvariant => {
            counts.add(
                EVERY_DEVICE_WITHOUT_A_SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOT_COUNTER,
                1,
            );
        }
        DevicesWithoutASelfVerifiedSystemConfigurationSlotAgainstTheBaseImage::New(devices) => {
            panic!(
                "{DEVICES_WITHOUT_A_SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOT_OBSERVER_NAME}：盘 {devices:?} 在这个崩溃状态上一个自证过的系统配置槽都没有，\
                 流的基镜像上它们还有（mkfs 返回之后、没有读故障的历史留不下这份盘面，D13（验证路线） 已定项 6）"
            );
        }
    }
}

// ============================== 判退看三样读者的并集：GPU 并集对拍的 CPU 参照（F6） ==============================
//
// 判退看三样读者的并集（C755（恢复与两种挂载触发退回的失败不同），`recovery::fall_back_judged_on_the_three_readers`）与只看走读
// （走读失败在字节丢了的失败点上才问 F2 判法：GPU 恢复内核改判并集之前那一判）在崩溃态上落不落到同一版。两边都在 CPU 上算，只比判退那一步：
// - 并集：核心层那一份原样调（① 系统配置没见证所选那一版、② 三样读者的并集里有字节丢了而没有内容拒收、③ 触发之后重扫一遍环再问 F2 判法）。
// - 只看走读：① 与 ③ 同一份，② 换成冷走读失败、失败种类是字节丢了（`RecoveryFailure::failure_kind_for_the_fall_back`）。
// 退到的那一版之后的读两边是同一段代码，判退那一步落到同一版，恢复就落到同一版；所以只比判退。
// GPU 恢复内核今天判并集（`crash_judge_recovery.wgsl` 的 `selected_version_under_the_three_readers`），按来历统一挂的并集对走读观察者撤了
// （主 agent 2026-10-22 定）；这一份是参照：快档用例 `tests/crash_judge_fall_back_on_the_union_of_readers_matches_the_walk_alone.rs` 拿它钉两边分得开、
// 拿核心层的恢复比 GPU 并集。

/// 一版的身份：(实例代号, checkpoint_txg)，择根的序就是它的序。
pub type VersionKey = (InstanceGeneration, CheckpointTxg);

/// journal 环里读出来的记录，按 (实例代号, 计数器)（`recovery::scan_journal` 交回的那一份）。
pub type RecordsOfTheRing = BTreeMap<(InstanceGeneration, u64), JournalRecord>;

#[must_use]
pub fn key_of(root: &RootRecord) -> VersionKey {
    (root.instance, root.checkpoint_txg)
}

/// 一种判退：所选根、所选那一版（一条记录都没施加时就是所选根）、这一处扫过的那一份记录 → 退到哪一版（`None` 是不退）。
/// 两种判退签名相同，核对只换这一个函数。
type FallBackJudgement = fn(
    &dyn PoolReader,
    &SystemConfiguration,
    &RootRecord,
    &RootRecord,
    &RecordsOfTheRing,
) -> Option<VersionKey>;

/// 判退看三样读者的并集：核心层那一份原样调（恢复、只读挂载、可写挂载三处共用的那一个函数）。
// cpu-reference-for-gpu-comparison
fn fall_back_under_the_union_of_the_three_readers(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
    chosen_root: &RootRecord,
    selected_version: &RootRecord,
    records_of_the_site: &RecordsOfTheRing,
) -> Option<VersionKey> {
    fall_back_judged_on_the_three_readers(
        reader,
        system_configuration,
        chosen_root,
        selected_version,
        records_of_the_site,
    )
    .map(|fall_back| key_of(&fall_back.version_fallen_back_to))
}

/// 判退只看走读：GPU 恢复内核那一判。① 与 ③ 与并集那一份同一个核心层函数（GPU 恢复内核照它另写的那一份，见
/// `crash_judge_recovery.wgsl` 的同名函数）；② 只看冷走读——走读失败、失败种类是字节丢了才往下判。
// cpu-reference-for-gpu-comparison
fn fall_back_under_the_walk_alone(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
    chosen_root: &RootRecord,
    selected_version: &RootRecord,
    records_of_the_site: &RecordsOfTheRing,
) -> Option<VersionKey> {
    newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
        reader,
        system_configuration,
        chosen_root,
        selected_version,
        records_of_the_site,
    )?;
    let failure_of_the_walk =
        first_failure_of_the_walk(reader, system_configuration, selected_version)?;
    match failure_of_the_walk.failure_kind_for_the_fall_back() {
        FailureKindForTheFallBack::BytesLost => {}
        FailureKindForTheFallBack::ContentRefused => return None,
    }
    let records_rescanned_after_the_trigger = scan_journal(reader, system_configuration);
    newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
        reader,
        system_configuration,
        chosen_root,
        selected_version,
        &records_rescanned_after_the_trigger,
    )
    .map(|root| key_of(&root))
}

/// 择到的系统配置下的单元区起点（核心层 `recovery::unit_area_start_of_the_chosen_system_configuration` 同一个算法：环末尾的下一个槽）。
fn unit_area_start_of(system_configuration: &SystemConfiguration) -> UnitAreaStart {
    UnitAreaStart::following_the_journal_ring(
        system_configuration.immutable.sizes.journal_ring_bytes,
    )
    .expect("择系统配置时读者判过环长算出的单元区起点在聚簇段边界上")
}

/// 冷走读在 `version` 上停在的第一个失败（走得通是 `None`）。
fn first_failure_of_the_walk(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
    version: &RootRecord,
) -> Option<RecoveryFailure> {
    let mut mapping_fallbacks = 0;
    walk_to_file_judging_the_truncation_fields_as(
        reader,
        unit_area_start_of(system_configuration),
        system_configuration
            .immutable
            .inode_record_truncation_fields,
        version,
        &mut mapping_fallbacks,
    )
    .err()
}

/// 三样读者在所选那一版上各自停在的第一个失败（读通是 `None`），两边不一致时打出来看是哪一样分了家。
#[derive(Debug)]
pub struct FirstFailureOfEachReader {
    pub walk: Option<RecoveryFailure>,
    pub opening: Option<OpenPoolForReadFailure>,
    pub rebuild: Option<RebuildVersionFailure>,
    pub instance_table_of_the_rebuild: Option<RecoveryFailure>,
}

/// 建上一版要的那条记录：同实例、同 checkpoint_txg、带末条标志的记录里计数器最大的那条，没有就拿环里计数器最大的那条
/// （核心层 `mount::record_standing_for_the_version` 的取法；它不公开，这里照抄一份只用来打失败）。
fn record_standing_for_the_version(
    records: &RecordsOfTheRing,
    version: &RootRecord,
) -> Option<JournalRecord> {
    records
        .values()
        .filter(|record| {
            record.instance == version.instance
                && record.checkpoint_txg == version.checkpoint_txg
                && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .max_by_key(|record| record.counter)
        .or_else(|| records.values().max_by_key(|record| record.counter))
        .cloned()
}

/// 三样读者（连同建上一版之后的实例表链）在所选那一版上各停在哪个失败上。
#[must_use]
pub fn first_failure_of_each_reader(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
    selected_version: &RootRecord,
    records_of_the_site: &RecordsOfTheRing,
) -> FirstFailureOfEachReader {
    let truncation_fields_of_the_pool = system_configuration
        .immutable
        .inode_record_truncation_fields;
    FirstFailureOfEachReader {
        walk: first_failure_of_the_walk(reader, system_configuration, selected_version),
        opening: open_pool_for_read_judging_the_truncation_fields_as(
            reader,
            selected_version,
            truncation_fields_of_the_pool,
        )
        .err(),
        rebuild: rebuild_version_judging_the_truncation_fields_as(
            reader,
            unit_area_start_of(system_configuration),
            truncation_fields_of_the_pool,
            selected_version,
            record_standing_for_the_version(records_of_the_site, selected_version),
        )
        .err(),
        instance_table_of_the_rebuild: instance_table_chain_of_root(reader, selected_version).err(),
    }
}

/// 一个崩溃态上恢复走到判退那一步时手里的东西，与两种判退各退到哪一版。
pub struct FallBackJudgedOnTheState {
    pub system_configuration: SystemConfiguration,
    pub chosen_root: RootRecord,
    pub selected_version: RootRecord,
    pub records_of_the_site: RecordsOfTheRing,
    pub under_the_union_of_the_three_readers: Option<VersionKey>,
    pub under_the_walk_alone: Option<VersionKey>,
}

/// 一个崩溃态上判退走没走到。
pub enum FallBackOfTheState {
    /// 恢复在判退之前就停下了（择不到系统配置、择不到根、重放在施加任何记录之前停下）：两种判退都走不到，GPU 恢复内核同样在那里停。
    StoppedBeforeTheFallBack,
    Judged(Box<FallBackJudgedOnTheState>),
}

/// 照恢复看 journal 那一遍（`recovery::recover` 的 `JournalPolicy::Consult`）走到判退那一步，两种判退各判一次。
// cpu-reference-for-gpu-comparison
#[must_use]
pub fn fall_back_of_the_state(reader: &dyn PoolReader) -> FallBackOfTheState {
    let Ok(SystemConfigurationChoice {
        system_configuration,
        devices_without_a_mountable_system_configuration_slot: _,
    }) = choose_system_configuration_naming_devices_without_a_mountable_slot(reader)
    else {
        return FallBackOfTheState::StoppedBeforeTheFallBack;
    };
    let Some(chosen_root) = choose_root(reader, &system_configuration) else {
        return FallBackOfTheState::StoppedBeforeTheFallBack;
    };
    let records_of_the_site = scan_journal(reader, &system_configuration);
    let Ok((_report, selected_version)) = replay_journal(
        reader,
        &chosen_root,
        system_configuration.immutable.sizes.journal_ring_bytes,
        &records_of_the_site,
        NamedUnitVerification::Verify,
    ) else {
        return FallBackOfTheState::StoppedBeforeTheFallBack;
    };
    let judge_under = |judgement: FallBackJudgement| {
        judgement(
            reader,
            &system_configuration,
            &chosen_root,
            &selected_version,
            &records_of_the_site,
        )
    };
    let under_the_union_of_the_three_readers =
        judge_under(fall_back_under_the_union_of_the_three_readers);
    let under_the_walk_alone = judge_under(fall_back_under_the_walk_alone);
    FallBackOfTheState::Judged(Box::new(FallBackJudgedOnTheState {
        system_configuration,
        chosen_root,
        selected_version,
        records_of_the_site,
        under_the_union_of_the_three_readers,
        under_the_walk_alone,
    }))
}

/// 两种判退不一致时那一行：状态身份、两边各退到哪一版、三样读者各停在哪个失败上。一致（或恢复走不到判退）交回 `None`。
#[must_use]
pub fn disagreement_of_the_state(state_identity: &str, reader: &dyn PoolReader) -> Option<String> {
    let FallBackOfTheState::Judged(judged) = fall_back_of_the_state(reader) else {
        return None;
    };
    disagreement_of_the_judged_state(state_identity, reader, &judged)
}

fn disagreement_of_the_judged_state(
    state_identity: &str,
    reader: &dyn PoolReader,
    judged: &FallBackJudgedOnTheState,
) -> Option<String> {
    if judged.under_the_union_of_the_three_readers == judged.under_the_walk_alone {
        return None;
    }
    Some(format!(
        "{state_identity}：所选根 {:?}、所选那一版 {:?}；并集退到 {:?}，只看走读退到 {:?}；三样读者各停在：{:?}",
        key_of(&judged.chosen_root),
        key_of(&judged.selected_version),
        judged.under_the_union_of_the_three_readers,
        judged.under_the_walk_alone,
        first_failure_of_each_reader(
            reader,
            &judged.system_configuration,
            &judged.selected_version,
            &judged.records_of_the_site
        )
    ))
}
