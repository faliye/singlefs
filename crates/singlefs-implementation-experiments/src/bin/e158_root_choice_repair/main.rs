// task-server-gpu-arguments: r2-all r2-seventh-batch r4-seg2
// 只有这几个模式起 `GivenStatesEngine`（`run_seventh_batch_reproductions` 与 `fourth_run_segment_two::run_segment_two`）判盘，
// 别的模式不判崩溃状态、不要卡；带 gpu 编好的二进制直接交统一任务服务时，由服务按 task-server-gpu-arguments 的声明清场、分卡。
//! E158（择根与修复四岔路）跑前登记第一段：入库装置。
//!
//! 派发范围只跑第一段（`research/prompts/e158-preregistration.md` 5.8）：
//! 停机条款 S1–S5 的逐项对拍、5.7 常量回比、第七节全部锚点、H3（岔路 3 的历史族）不注入全跑出 N_w 与轨迹、
//! Q3-2、Q2-3 的算术、PC-Nw、PC3。岔路 1（C393）、岔路 2（C331）、岔路 4（C334）与它们的臂表、副本、
//! Q1 / Q2 / Q4 全部量都不在这一段，本文件不实现。
//!
//! 只读、只驱动、只观测：不改 `crates/singlefs-core`、`crates/singlefs-checker`；几何全部走本文件的本地常量，
//! 不 `use` `crates/singlefs-verification/src/segments.rs` 的 `FixedGeometry` 默认值或别的写死几何来决定本轮的工作量 /
//! 几何 / 判据门槛——本地常量的值抄自 `research/prompts/e158-preregistration.md` 5.7 与第七节，注明来源，
//! 另加回比断言与 `crates::` 同名常量比对（S1–S5 与 5.7 表，`run_constants_and_anchors`）。
//! `FixedGeometry` 类型本身在 PC3 的 `SharedFaultPlan::armed` 里用到一次：那是故障注入 API 的必经参数类型，
//! 填的是本文件自己算出的本地几何值，不是从它的默认值读几何——这一处判断留给交回报告，请主 agent 核。

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::env;

use singlefs_checker::image as checker_image;
use singlefs_checker::image::{ImageReader, PoolGeometry};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::allocator::{AllocationRecord, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::{record_offset, JournalRecord};
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, ring_slots_whose_reread_still_bad_refuses_the_ceiling_by,
    roll_back_by_a_forward_publish, rollback_floor_ceiling, Mounted, RollbackTarget, ShadowLedger,
};
use singlefs_core::pointer::LocationEntry;
use singlefs_core::records::{
    parse_mapping_entry, TreeTableEntry, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION,
    TREE_KIND_DEADLIST, TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LIVELIST,
    TREE_KIND_SPARSE_SIDE_TABLE,
};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_system_configuration, instance_table_chain_of_root,
    recover, replay_journal, scan_journal, tree_table_has_no_entries, JournalPolicy,
    NamedUnitVerification, RecoveryOutcome, RecoveryReport,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{RootRingSlot, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion,
    PoolWriter, TransactionOutput,
};
use singlefs_core::unit::{parse_index_node, unit_filesystem_identifier};
use singlefs_core::write_accounting::{WritesByStructureKind, WrittenStructureKind};
use singlefs_device_library::DeviceLibrary;
use singlefs_format::JOURNAL_RECORD_BYTES;
use singlefs_verification::fault_injection::{
    injected_block_device_error, FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice,
    FaultOccurrence, FaultPlacement, FaultSchedule, InjectedFault, NamedRootRingSlots,
    PoolReaderWithUnreadableRootRingSlots, RootRingSlotTarget, SharedFaultPlan,
};
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use singlefs_verification::segments::FixedGeometry;

mod fault_set_violation_checkpoints;
mod fifth_run_domain_and_units;
mod fifth_run_domain_runs;
mod fifth_run_judgement_library;
mod fifth_run_pair_construction_and_variants;
mod fifth_run_tallies_and_quantity_lines;
mod fourth_run_arms_and_device;
mod fourth_run_compare;
mod fourth_run_constants_and_mode;
mod fourth_run_hidden_root_and_sentences;
mod fourth_run_legal_and_fault_state_families;
mod fourth_run_positive_controls;
mod fourth_run_prefixes_and_families;
mod fourth_run_shadow_ledger_and_later_segments;
mod fourth_run_today_bases;
mod geometry_and_simulated_history;
mod independent_decode;
mod ledger_fault_family;
mod ledger_fault_probes;
mod local_constants_and_anchors;
mod pool_checker_through_the_given_states_engine;
mod publication_bytes_by_structure_kind;
mod root_choice_enumeration;
mod rootback_tolerance_search;
mod second_run_constants_and_anchors;
mod second_run_families;
mod second_run_history;
mod second_run_instrumented_device;
mod third_run_arms_and_device;
mod third_run_constants_and_compare;
mod third_run_device_decode;
mod third_run_families_and_positive_controls;
mod third_run_history_and_cell_loss;
mod watermark_and_system_configuration_controls;

use self::fault_set_violation_checkpoints::*;
use self::fifth_run_domain_and_units::*;
use self::fifth_run_domain_runs::*;
use self::fifth_run_judgement_library::*;
use self::fifth_run_pair_construction_and_variants::*;
use self::fifth_run_tallies_and_quantity_lines::*;
use self::fourth_run_arms_and_device::*;
use self::fourth_run_compare::*;
use self::fourth_run_constants_and_mode::*;
use self::fourth_run_hidden_root_and_sentences::*;
use self::fourth_run_legal_and_fault_state_families::*;
use self::fourth_run_positive_controls::*;
use self::fourth_run_prefixes_and_families::*;
use self::fourth_run_shadow_ledger_and_later_segments::*;
use self::fourth_run_today_bases::*;
use self::geometry_and_simulated_history::*;
use self::independent_decode::*;
use self::ledger_fault_family::*;
use self::ledger_fault_probes::*;
use self::local_constants_and_anchors::*;
use self::pool_checker_through_the_given_states_engine::*;
use self::publication_bytes_by_structure_kind::*;
use self::root_choice_enumeration::*;
use self::rootback_tolerance_search::*;
use self::second_run_constants_and_anchors::*;
use self::second_run_families::*;
use self::second_run_history::*;
use self::second_run_instrumented_device::*;
use self::third_run_arms_and_device::*;
use self::third_run_constants_and_compare::*;
use self::third_run_device_decode::*;
use self::third_run_families_and_positive_controls::*;
use self::third_run_history_and_cell_loss::*;
use self::watermark_and_system_configuration_controls::*;

/// （`m2-rootchoice-repair-r1-forks.md` 岔路单第 2 行还差项②）：旧的计数上限
/// （`SUBSET_ENUMERATION_CAP=20_000`）会在权重层中途停手、不报「完整空间多大」，判「负结果」时
/// 分不清是穷举完了还是撞了计数——按登记要求撤掉，换成下面两个按权重层边界决定停不停的常量。
///
/// 完整证据空间的子集数 ≤ 这个预算 ⇒ `weight_ceiling` 传 `None` 时穷举到 `maximum_weight`（真正的
/// 完整空间）；超过 ⇒ 退到 `DEFAULT_WEIGHT_CEILING_WHEN_INFEASIBLE`（权重 12，与丙的 k=12 可比，
/// 岔路单原文「某个 n1 的完整空间大到按本机挂钟算不完的，按权重从小到大穷举到权重 12」）。100_000
/// 个子集按 H-C1 下界探针实测速率（22558 个子集 real 2m45s ≈ 7.3ms/个，
/// `research/results/e158-root-choice-repair-2026-09-24-q2-1-hc1-lower-bound.out`）折算约 12 分钟，
/// 在一次前台或后台调用里可接受；调用方显式传 `Some(w)` 时这个自动判断不生效，直接用调用方给的
/// 权重（例如 H-C1 下界探针自己要精确探到某一档）。
const FEASIBLE_FULL_SEARCH_SUBSET_BUDGET: u64 = 100_000;
/// 完整空间太大时退到的权重上限：丙（今天）在 n1=4 上实测恰好在权重 12 打中（H-C1 直接构造 +
/// 下界探针），选它是为了让各修法在同一档权重上可比。
const DEFAULT_WEIGHT_CEILING_WHEN_INFEASIBLE: u64 = 12;

/// 两块盘各自的字节数：够装全部取样点的几何（最大 journal 环 3 MiB、S 至多 16），4 GiB 与
/// `crates/singlefs-verification/tests/common/mod.rs` 的 `IMAGE_BYTES` 同一个量级，不共用那份代码。
const IMAGE_BYTES: u64 = 4 << 30;
/// 固定 fsid：与 E142 装置那份不同，避免两个进程各自建的镜像撞出同一份「历史上出现过」的巧合。
const FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x45, 0x31, 0x35, 0x38, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00, 0x00, 0x00, 0x00, 0x00,
];
/// 装置自己发的文件很小：H3 只关心根环上的 (实例, txg) 与「读得出哪些根」，不关心内容字节数，
/// 小文件让每次发布的挂钟代价降到最低（大规模穷举需要）。
const FIRST_FILE_BYTES: usize = 96;
const OVERWRITE_FILE_BYTES: usize = 96;
const FIXED_WRITE_TIME_SECONDS: u64 = 1_800_000_000;

/// 项目全仓统一的产物完整性闸口径（`scripts/experiment-apparatus/replay.sh` 文件头「收尾行必须是
/// `name=done emitted=N`」那一条）：每一条结果行都以 `E7RESULT` 起头，`main` 收尾打一条
/// `E7RESULT name=done emitted=N`，N = 本进程打过的 `E7RESULT` 行数（含收尾这一条）。
static EMITTED_RESULT_LINES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn emit_result(line: &str) {
    EMITTED_RESULT_LINES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    println!("E7RESULT {line}");
}

fn emitted_result_line_count() -> u64 {
    EMITTED_RESULT_LINES.load(std::sync::atomic::Ordering::SeqCst)
}
use std::cell::RefCell;
use std::rc::Rc;

/// 等待钩子与虚拟时钟（登记「读法写死」表「时长」一行）：时钟在每次入口调用开始时归 0，只在臂调「等一个间隔」的钩子时前进 δ。
/// 臂的代码只在草稿副本里有（臂表 r3 行），装置源码各臂同一份：副本的 singlefs-implementation-experiments 声明特性 `e158-r3-arm` 且缺省开，
/// 主工作区与今天那一臂的副本不声明，这几个函数退成空。
#[allow(
    unexpected_cfgs,
    reason = "特性 e158-r3-arm 只在草稿副本的 singlefs-implementation-experiments/Cargo.toml 里声明（scripts/mutation/tables/e158_arms.tsv 收尾那一行）"
)]
mod third_run_arm_hooks {
    use std::cell::Cell;

    thread_local! {
        static VIRTUAL_CLOCK: Cell<u64> = const { Cell::new(0) };
        static WAITS: Cell<u64> = const { Cell::new(0) };
    }

    pub fn reset_the_virtual_clock() {
        VIRTUAL_CLOCK.with(|clock| clock.set(0));
        WAITS.with(|waits| waits.set(0));
    }

    pub fn virtual_now() -> u64 {
        VIRTUAL_CLOCK.with(Cell::get)
    }

    pub fn waits() -> u64 {
        WAITS.with(Cell::get)
    }

    #[allow(
        dead_code,
        reason = "只在臂的副本（特性 e158-r3-arm 开）里被等待钩子调；主工作区与今天那一臂里只有单测调它"
    )]
    pub fn advance_one_interval(interval: u64) {
        VIRTUAL_CLOCK.with(|clock| clock.set(clock.get() + interval));
        WAITS.with(|waits| waits.set(waits.get() + 1));
    }

    pub fn arm_code_is_compiled_in() -> bool {
        cfg!(feature = "e158-r3-arm")
    }

    #[cfg(feature = "e158-r3-arm")]
    pub fn install_the_wait_hook(interval: u64) {
        singlefs_core::mount::r3_install_wait_hook(Some(Box::new(move || {
            advance_one_interval(interval);
        })));
    }

    #[cfg(not(feature = "e158-r3-arm"))]
    pub fn install_the_wait_hook(_interval: u64) {}

    #[cfg(feature = "e158-r3-arm")]
    pub fn take_the_arm_observation() -> Option<super::ArmObservation> {
        let observation = singlefs_core::mount::r3_take_observation();
        Some(super::ArmObservation {
            witness_judged: observation.witness_judged,
            newer_root_on_the_first_pass: observation.newer_root_on_the_first_pass,
            witnessed_tail: observation.witnessed_tail,
            selected_version_last_counter: observation.selected_version_last_counter,
            undecidable: observation.undecidable,
            unreadable_ring_slots_on_the_first_pass: observation
                .unreadable_ring_slots_on_the_first_pass,
            reread_rounds: observation.reread_rounds,
            rebuilt_from_records: observation.rebuilt_from_records,
            shadow_ledger_table_unreadable_on_the_first_read: observation
                .shadow_ledger_table_unreadable_on_the_first_read,
            shadow_ledger_table_rereads: observation.shadow_ledger_table_rereads,
        })
    }

    #[cfg(not(feature = "e158-r3-arm"))]
    pub fn take_the_arm_observation() -> Option<super::ArmObservation> {
        None
    }
}

/// 等待钩子（第 4 次跑）：推进虚拟时钟之前先调 `on_wait`（记第一个等待标记）。臂的代码只在副本里有，见 `third_run_arm_hooks`。
#[allow(
    unexpected_cfgs,
    reason = "特性 e158-r3-arm 只在草稿副本的 singlefs-implementation-experiments/Cargo.toml 里声明（scripts/mutation/tables/e158_arms.tsv 的 r4 收尾那一行）"
)]
mod fourth_run_arm_hooks {
    #[cfg(feature = "e158-r3-arm")]
    pub fn install_the_marking_wait_hook(interval: u64, on_wait: Box<dyn Fn()>) {
        singlefs_core::mount::r3_install_wait_hook(Some(Box::new(move || {
            on_wait();
            super::third_run_arm_hooks::advance_one_interval(interval);
        })));
    }

    #[cfg(not(feature = "e158-r3-arm"))]
    pub fn install_the_marking_wait_hook(_interval: u64, _on_wait: Box<dyn Fn()>) {}

    #[cfg(feature = "e158-r3-arm")]
    pub fn take_the_arm_observation_and_its_extras(
    ) -> Option<(super::ArmObservation, super::FourthRunArmExtras)> {
        let observation = singlefs_core::mount::r3_take_observation();
        Some((
            super::ArmObservation {
                witness_judged: observation.witness_judged,
                newer_root_on_the_first_pass: observation.newer_root_on_the_first_pass,
                witnessed_tail: observation.witnessed_tail,
                selected_version_last_counter: observation.selected_version_last_counter,
                undecidable: observation.undecidable,
                unreadable_ring_slots_on_the_first_pass: observation
                    .unreadable_ring_slots_on_the_first_pass,
                reread_rounds: observation.reread_rounds,
                rebuilt_from_records: observation.rebuilt_from_records,
                shadow_ledger_table_unreadable_on_the_first_read: observation
                    .shadow_ledger_table_unreadable_on_the_first_read,
                shadow_ledger_table_rereads: observation.shadow_ledger_table_rereads,
            },
            super::FourthRunArmExtras {
                reread_rounds_limit: observation.reread_rounds_limit,
                zeroed_journal_ring_slots_are_cached: observation
                    .zeroed_journal_ring_slots_are_cached,
            },
        ))
    }

    #[cfg(not(feature = "e158-r3-arm"))]
    pub fn take_the_arm_observation_and_its_extras(
    ) -> Option<(super::ArmObservation, super::FourthRunArmExtras)> {
        None
    }
}

// ============================================================================
// 十三、第 4 次跑第二段（`research/prompts/e158-r4-prereg.md` 5.3 实七-甲、实七-乙、实八、H-随，5.5 主 agent 认定第 9 项 H-随全，
//      5.4 PC-随，5.6 H-随 的常量回比，11.2 S5a、S5b、S7、S9，第九节 M22）：模式 `r4-seg2`。
//      harness 按种子生成的历史在每一臂的副本上逐步执行。执行器是装置自己的：八种操作照 harness 执行器（`history.rs` 的 `apply_*`）
//      同一串入口、同一套前提各写一遍，盘照 harness 那一叠（故障注入包录制器包内存盘）；checker 每一步的判定原样记下，判红不截断。
//      harness 的注入点与崩溃状态按「同一步、步内同一序号」映到各臂自己的调用与写序列上（第十二节第二段修订）。
//      今天那一臂上每一步与 harness 自己的执行器比镜像与调用数：对得上，映射才站得住（对不上走 S9 / S7）。
// ============================================================================

mod fourth_run_segment_two;

fn main() {
    let command_line_arguments: Vec<String> = env::args().collect();
    let mode = command_line_arguments
        .get(1)
        .map(String::as_str)
        .unwrap_or("all");
    if run_second_run_mode(mode) {
        return;
    }
    if run_third_run_mode(mode, &command_line_arguments) {
        return;
    }
    if run_fourth_run_mode(mode, &command_line_arguments) {
        return;
    }
    if run_fifth_run_mode(mode, &command_line_arguments) {
        return;
    }

    let constants_ok = run_constants_and_anchors();
    if !constants_ok {
        emit_result("name=stop reason=V3 detail=\"5.7 常量回比或第七节算术有不等，整轮不开跑\"");
        std::process::exit(1);
    }
    if mode == "constants" {
        emit_result(&format!(
            "name=done emitted={}",
            emitted_result_line_count() + 1
        ));
        return;
    }

    let independent_decode_check_passed = run_independent_decode_cross_check();
    if !independent_decode_check_passed {
        emit_result(
            "name=stop reason=S5 detail=\"装置自己的解码与 crates::recovery 的输出对不上\"",
        );
        std::process::exit(1);
    }
    if mode == "s5" {
        emit_result(&format!(
            "name=done emitted={}",
            emitted_result_line_count() + 1
        ));
        return;
    }

    let initial_overwrite_counts = [1u64, 2, 3];
    let mut overall_peak_events_live = 0u64;
    let mut pc3_node: Option<(SimNode, MakeFilesystemParameters, PoolGeometry)> = None;

    if mode == "all" || mode == "h3-g0" || mode == "pc3" {
        let summary = run_root_choice_geometry(&GEOMETRY_PRIMARY, &initial_overwrite_counts, 3);
        overall_peak_events_live = overall_peak_events_live.max(summary.peak_events_live);
        if let Some(node) = summary.first_events_live_node {
            let parameters = parameters_for(&GEOMETRY_PRIMARY);
            if let Ok(geometry_view) = independent_geometry(&node.pool) {
                pc3_node = Some((node, parameters, geometry_view));
            }
        }
    }
    if mode == "all" || mode == "h3-s16" {
        let summary =
            run_root_choice_geometry(&GEOMETRY_LARGER_ROOT_RING, &initial_overwrite_counts, 3);
        overall_peak_events_live = overall_peak_events_live.max(summary.peak_events_live);
    }
    if mode == "all" || mode == "h3-small-ring" {
        let small_ring_bytes = find_small_ring();
        emit_result(&format!(
            "name=small_ring_search found_ring_bytes={small_ring_bytes} search_started_at_bytes=49152 step_bytes=16384"
        ));
        let small_ring = Geometry {
            label: "小环",
            slots_per_region: 8,
            journal_ring_bytes: small_ring_bytes,
        };
        let summary = run_root_choice_geometry(&small_ring, &initial_overwrite_counts, 3);
        overall_peak_events_live = overall_peak_events_live.max(summary.peak_events_live);
    }
    if mode == "all" || mode == "h3-s4" {
        let summary =
            run_root_choice_geometry(&GEOMETRY_SMALLER_ROOT_RING, &initial_overwrite_counts, 4);
        overall_peak_events_live = overall_peak_events_live.max(summary.peak_events_live);
    }

    if mode == "all"
        || mode == "h3-g0"
        || mode == "h3-s16"
        || mode == "h3-small-ring"
        || mode == "h3-s4"
    {
        emit_result(&format!(
            "name=peak_live_rollback_events_across_the_family value={overall_peak_events_live}"
        ));
        report_witness_width_and_slot_overlap_arithmetic(overall_peak_events_live);
    }

    if mode == "all" || mode == "pc3" {
        match pc3_node {
            Some((node, parameters, geometry_view)) => {
                let pc3_ok = run_pc3(&node, &parameters, &geometry_view);
                emit_result(&format!(
                    "name=pc3_overall_verdict verdict={}",
                    if pc3_ok { "pass" } else { "fail" }
                ));
            }
            None => emit_result(
                "name=pc3_overall_verdict verdict=not_applicable reason=\"GEOMETRY_PRIMARY 全族里没有找到 events_live≥1 的节点\"",
            ),
        }
    }

    // Q3-1（跑前登记第二段，岔路 3 候选 3 那一半）：今天那一臂的 H3 × Φ3 违例比例。
    // 不并进 "all"：这一段代价数量级不同（每个 H3 节点还要按 |F| ≤ 2 的组合各做一次 (a)，
    // 打中省法门槛之外的还要各做一次 (b)(c)），独立跑、独立记时。
    if mode == "q3-1-g0" {
        run_fault_set_violation_family(&GEOMETRY_PRIMARY, &initial_overwrite_counts, 3);
    }
    // 只跑最浅的一层，用来估算每个节点的挂钟代价（不是登记里要的产物，交回报告说明为什么有这一档）。
    if mode == "q3-1-g0-timing-probe" {
        run_fault_set_violation_family(&GEOMETRY_PRIMARY, &[1], 1);
    }
    if mode == "q3-1-g0-timing-probe-2" {
        run_fault_set_violation_family(&GEOMETRY_PRIMARY, &[1], 2);
    }
    if mode == "q3-1-g0-timing-probe-3" {
        run_fault_set_violation_family(&GEOMETRY_PRIMARY, &[1], 3);
    }
    if mode == "q3-1-s16" {
        run_fault_set_violation_family(&GEOMETRY_LARGER_ROOT_RING, &initial_overwrite_counts, 3);
    }
    if mode == "q3-1-small-ring" {
        let small_ring_bytes = find_small_ring();
        emit_result(&format!(
            "name=small_ring_search found_ring_bytes={small_ring_bytes} search_started_at_bytes=49152 step_bytes=16384"
        ));
        let small_ring = Geometry {
            label: "小环",
            slots_per_region: 8,
            journal_ring_bytes: small_ring_bytes,
        };
        run_fault_set_violation_family(&small_ring, &initial_overwrite_counts, 3);
    }
    if mode == "q3-1-s4" {
        run_fault_set_violation_family(&GEOMETRY_SMALLER_ROOT_RING, &initial_overwrite_counts, 4);
    }

    // Q1（跑前登记岔路单第 1 行，C393）：H1 家族 + Φ1 故障注入 + PC1-a/PC1-b。
    if mode == "q1-g0" {
        let summary = run_ledger_fault_family(&GEOMETRY_PRIMARY);
        emit_ledger_fault_family_summary(&GEOMETRY_PRIMARY, &summary);
        run_ledger_fault_positive_controls(&GEOMETRY_PRIMARY);
    }

    // 第八节几何敏感性（岔路单第 1 行判决格 = Q1-1a 的 N_trig）：
    // S16（根环大一倍）与 S4（根环小一半）两个方向相反的取样点，复用同一套 H1 装置代码。
    if mode == "q1-s16" {
        let summary = run_ledger_fault_family(&GEOMETRY_LARGER_ROOT_RING);
        emit_ledger_fault_family_summary(&GEOMETRY_LARGER_ROOT_RING, &summary);
    }
    if mode == "q1-s4" {
        let summary = run_ledger_fault_family(&GEOMETRY_SMALLER_ROOT_RING);
        emit_ledger_fault_family_summary(&GEOMETRY_SMALLER_ROOT_RING, &summary);
    }

    // Q2-1（跑前登记岔路单第 2 行 ①，C331 修法）：H2 主族在 G0 上、n1 ∈ {0,1,2,3}、n2=1 的穷举下界。
    // 只在这一份 crates/ 上跑（同一个二进制在不同臂的副本目录里各编各的，命令与产物文件名分臂）。
    if mode == "q2-1-g0" {
        run_rootback_tolerance_family(&GEOMETRY_PRIMARY, 1, false, 0..=6, None);
    }

    // Q2-1，Φ2 加系统配置槽这一类证据（跑前登记岔路单第 2 行 ①）：只在读系统配置的
    // 臂（乙-留环、丁-留环、丁-只配置）上有意义；跑在不读系统配置的臂上也不会红，只是白扩大搜索
    // 空间，所以只在这三份副本上跑这个 mode。n1 区间可选从第二、三个命令行参数读（默认 0..=6，
    // 缺省不改变行为）——只为了能把一次挂钟预算装不下的全范围拆成几次跑，不改判据。
    if mode == "q2-1-g0-configuration-evidence" {
        let range_start = command_line_arguments
            .get(2)
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(0);
        let range_end = command_line_arguments
            .get(3)
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(6);
        run_rootback_tolerance_family(&GEOMETRY_PRIMARY, 1, true, range_start..=range_end, None);
    }

    // H-C2 直接构造 + 两个负对照（Φ2 系统配置槽这一类证据的独立构造，见十三点六节）。
    if mode == "q2-1-hc2" {
        run_direct_construction_for_the_system_configuration_evidence_class(&GEOMETRY_PRIMARY);
        run_negative_control_older_slot_for_the_system_configuration_evidence_class(
            &GEOMETRY_PRIMARY,
        );
        run_negative_control_other_device_for_the_system_configuration_evidence_class(
            &GEOMETRY_PRIMARY,
        );
    }

    // Q2-2a（跑前登记岔路单第 2 行 ②）：固定脚本上这一份 crates/ 每次发布按结构种类写的字节。
    if mode == "q2-2a-g0" {
        run_fixed_publication_byte_script(&GEOMETRY_PRIMARY);
    }

    // 阳性对照 PC2（岔路单第 2 行 ①）：直接复现判决 K2 的两个具体构造，
    // 不靠穷举——见十三点五节顶部注释。两个函数在没有对应候选字段/改法的臂上应当报 hit=false /
    // matches_k2_table=false，这也是证据的一部分（见交回报告）。
    if mode == "q2-1-pc2" {
        run_positive_control_for_the_root_ring_txg_only_candidate(&GEOMETRY_PRIMARY);
        run_positive_control_for_the_configuration_published_txg_candidate(&GEOMETRY_PRIMARY);
    }

    // H-C1 直接构造（岔路单第 2 行 ①，丙的具体历史）：判决 K2 说丙需要
    // 12 个故障（4 根槽 + 4 条记录各两块盘）才打得中；这一段不靠穷举（n1∈{4,5,6} 的完整空间太大，
    // 按权重上限 12 穷举，见下面「H-C1 下界探针」），直接按这个具体构造跑一次，
    // 附一个只挡根槽的负对照。
    if mode == "q2-1-hc1" {
        run_direct_construction_for_the_record_scan_watermark_candidate(&GEOMETRY_PRIMARY);
        run_negative_control_root_slots_only_for_the_record_scan_watermark_candidate(
            &GEOMETRY_PRIMARY,
        );
    }

    // H-C1 下界探针：权重上限从第二个命令行参数读，缺省 `None`（按 `FEASIBLE_FULL_SEARCH_SUBSET_BUDGET`
    // 自动判断——n1=4 这个节点的完整空间只有 2^15=32768，在预算内，所以缺省就是穷举到完整空间的顶）。
    if mode == "q2-1-hc1-lower-bound" {
        let weight_ceiling = command_line_arguments
            .get(2)
            .and_then(|value| value.parse::<u64>().ok());
        run_record_scan_watermark_lower_bound_probe(&GEOMETRY_PRIMARY, weight_ceiling);
    }

    emit_result(&format!(
        "name=done emitted={}",
        emitted_result_line_count() + 1
    ));
}

/// 抛弃步这一组判法的入口（device-code-key.py 按它取这一组的代码键）：节点镜像 → 构造成立与否、被抛弃根集合，交判定值文本。
fn abandonment_judgement_entry(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
    replay_break: ReplayBreak,
    construction_cell: &std::cell::OnceCell<Result<PairConstruction, String>>,
) -> Result<String, String> {
    let construction = constructed_or_error(construction_cell, || {
        construct_pair(node, parameters, replay_break)
    })?;
    Ok(outcome_of_the_construction(construction).to_text())
}

/// 孪生这一组判法的入口。
fn twin_judgement_entry(job: &VariantJob) -> Result<String, String> {
    variant_judgement_text(job, VariantSpecification::Twin)
}

/// 账读不出这一组判法的入口。
fn account_unreadable_judgement_entry(
    job: &VariantJob,
    specification: VariantSpecification,
) -> Result<String, String> {
    match specification {
        VariantSpecification::Account { .. } => variant_judgement_text(job, specification),
        VariantSpecification::Twin | VariantSpecification::RootSlot { .. } => {
            Err(format!("{} 不属于账读不出这一组", specification.name()))
        }
    }
}

/// 根槽读不出这一组判法的入口。
fn root_slot_unreadable_judgement_entry(
    job: &VariantJob,
    specification: VariantSpecification,
) -> Result<String, String> {
    match specification {
        VariantSpecification::RootSlot { .. } => variant_judgement_text(job, specification),
        VariantSpecification::Twin | VariantSpecification::Account { .. } => {
            Err(format!("{} 不属于根槽读不出这一组", specification.name()))
        }
    }
}

#[cfg(test)]
mod tests;
