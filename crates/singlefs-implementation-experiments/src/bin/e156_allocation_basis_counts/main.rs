// task-server-gpu-arguments: *
// 不分子命令，每一趟都经 `verdict` 把盘交 `GivenStatesEngine` 判；带 gpu 编好的二进制直接交统一任务服务时，由服务清场、分卡。
//! E156（alloc-basis 四条岔路的代价数） 第 4 次重跑（登记 `research/prompts/e156-r4-prereg.md`）：回退改成挂着时的一次向前发布、
//! 挂载内回收、准入改式之后，岔路单第 1 行（记账第 1 项与 I-3.1（已分配统计对得上） 的形态，环上有洞怎么办）的代价数，
//! 连同问题单第一节三个前提在这个装置上的最小复现。
//!
//! - 第一段（登记第五节 5.5）：PQ1-前 / PQ1-后（前提 1：记录已持久、根槽没持久的崩溃之后可写挂载，环上出洞）与 PC-c2、
//!   PQ2（前提 2：抬 F 那一串里 F_生效 什么时候取到新值）、G-adm（前提 3 的间接影响）、U11 与 U13 的新形态（管理员回退是挂着时的
//!   一次向前发布）。第一段的停机条款（SP1、S1、S12）任一触发，第二段不跑。
//! - 第二段：Hh(k, 位置, S, ρ) 38 条排得下的历史，每条在「实」（真实分配器）与「每」（每次发布之前按谓词现算）两个回收时点上
//!   量甲-T1 与 G12 两条臂的多扣（Q1a、Q1b）与它们的差 Δ，按洞的计数分组判（Q1c、Q1c-洞、Q1c-前、Q1c-后、Q1d）。
//!
//! 只读驱动 `singlefs-core` / `singlefs-checker` 的公开入口，不改这两个 crate 的生产代码；产物按 `E7RESULT` 行打印，复跑登记在
//! `scripts/experiment-apparatus/replay.sh`。洞的 txg、挂载做完的 txg、终点 txg 从本地 S 与装置里的锚点模型（[`anchor_model`]，登记第七节 7.2
//! 命令四 `anchors_e156_r4.py` 的逐行移植，门槛那一处照第 6 次重跑登记 `research/prompts/e156-r6-prereg.md` 第十三节命令四
//! `anchors_e156_r6.py` 改）算，不从 `crates/` 算；装置跑的时候从盘上现量，两边比。
//!
//! 第 6 次重跑（登记 `research/prompts/e156-r6-prereg.md` 第一节 W15–W18、第五节 E7–E11）：回收门槛照乙读——
//! 挂载那一刻读回全零的根环槽钉在它可能住过的最旧 txg，直到这个进程写掉那一槽、或读出一条根（[`read_hh_state`] 的
//! `all_zero_slots_still_pinned`）；K10 守门、「每」列的旁路门槛与锚点模型三处吃同一个读数。
//!
//! 第 7 次重跑（登记 `research/prompts/e156-r7-prereg.md`，问题单 H1）：洞数档扫到环满一圈并跨圈——Hh(k, 前) 的 k 扫到 `k_满 + 1`、Hh(k, 后) 扫到
//! `k_满后`、Hc（同一间距一直造到跨过两圈、三圈）与 Hs（洞钉在同一个环槽、每圈一个），排不下的格照跑；每个统计单位 (量, S, ρ, 回收时点) 的档序列
//! 按 W28 判「仍在涨 / 已平 / 不单调」并报上界数（`name=r7_tier_peaks`、`name=r7_verdict`）。锚点模型加了按给定洞 txg 造历史的入口
//! （[`anchor_model::HoleSchedule`]），族表按 S 现算并与登记第七节 A20 的本地常量表回比；锚点的摘要与逐观测点两种行（`name=anchor_r7_summary`、
//! `name=anchor_r7_dump`）去掉前缀之后与登记第十三节命令一的 `anchors_e156_r7.py` 输出逐行比（停机 S20；摘要行的五个键名改过，比之前先照
//! [`emit_hole_family_anchor_cells`] 文档里那张对照把脚本输出换名）。
//!
//! 判定写在装置里（[`judge_peak_against_threshold`] 那一组函数，单测 U18），不另写 Python 判定脚本：执行员的写范围不含
//! `scripts/` 与门禁 47 号，理由与改动记在登记第十二节。

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::OnceLock;

use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::{AllocationRecord, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemOutput,
    MakeFilesystemParameters,
};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling, roll_back_by_a_forward_publish,
    MountSpaceAdmission, RaiseToTheAdmissionCeiling, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_system_configuration, effective_rollback_floor,
    instance_table_chain_of_root, read_root_ring_slot, readable_roots,
    readable_roots_with_ring_slots, root_is_abandoned_by_the_instance_table, PoolReader,
    RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{
    slot_offset, target_for_publish, RootRingSlot, RootRingSlotsPerRegion,
};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, publish_version, warm_up, FirstFile,
    InstanceTablePlan, PoolWriter, PublishError, PublishPlan, TransactionOutput, TransactionUnit,
};
use singlefs_format::{
    JOURNAL_RING_DEFAULT_BYTES, ROOT_RING_REGIONS, SLOT_BYTES, UNIT_AREA_START_SLOT,
};
use singlefs_verification::crash_amplification::{GivenStatesEngine, GivenStatesItem};
use singlefs_verification::crash_given_states::{
    GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
};
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice, SECTOR_BYTES};
use singlefs_verification::segments::{FixedGeometry, StepKind};
use singlefs_verification::{RecordingBlockDevice, SharedStream};

mod accounting_rows_and_device_corruption;
mod allocation_record_tree_closed_forms;
mod anchor_cells_and_hole_history_verdicts;
mod checker_verdicts;
mod crash_before_the_root_persists;
mod floor_raise_trace;
mod forward_rollback_scenarios;
#[cfg(test)]
mod helpers_shared_by_the_tests;
mod hh_cell_summary;
mod hh_history_run;
mod hh_observations_and_root_ring_slots;
mod hh_state_reading;
mod hole_family_anchor_cells;
mod hole_family_verdict;
mod hole_positions_and_hh_cell_shapes;
mod local_constants_and_pool_parameters;
mod memory_pool_snapshots;
mod peak_and_group_judgements;
mod tier_growth_and_units;
mod workload_publishes;

use accounting_rows_and_device_corruption::*;
use allocation_record_tree_closed_forms::*;
use anchor_cells_and_hole_history_verdicts::*;
use checker_verdicts::*;
use crash_before_the_root_persists::*;
use floor_raise_trace::*;
use forward_rollback_scenarios::*;
#[cfg(test)]
use helpers_shared_by_the_tests::*;
use hh_cell_summary::*;
use hh_history_run::*;
use hh_observations_and_root_ring_slots::*;
use hh_state_reading::*;
use hole_family_anchor_cells::*;
use hole_family_verdict::*;
use hole_positions_and_hh_cell_shapes::*;
use local_constants_and_pool_parameters::*;
use memory_pool_snapshots::*;
use peak_and_group_judgements::*;
use tier_growth_and_units::*;
use workload_publishes::*;

/// 覆盖写第 `step` 次的文件内容：定长 3000 字节，字节序列随 `step` 变，保证每次覆盖写都改变用户可见状态（骨架发布 的 O）。
fn overwrite_content(step: u64) -> Vec<u8> {
    let salt = usize::try_from(step % 251).expect("对 251 取模落在 usize 里");
    (0..3000usize)
        .map(|index| u8::try_from((index + salt * 7 + 3) % 251).expect("小于 256"))
        .collect()
}

struct Emitter {
    emitted: u64,
}
impl Emitter {
    fn emit(&mut self, body: &str) {
        self.emitted += 1;
        println!("E7RESULT {body}");
    }
    fn finish(&mut self) {
        self.emitted += 1;
        println!("E7RESULT name=done emitted={}", self.emitted);
    }
}

// ============================================================================================
// 锚点模型：第 4 次重跑登记第十三节命令四 `anchors_e156_r4.py`（第三版）的逐行移植。不 import 装置别的部分、不读 `crates/`，
// 只用脚本头那几句条款：根环 txg u 落槽 u mod 3S；可再分配 = 已释放 ∧ 释放代 ≤ 环里最旧有效根（Hh 里 F_生效 = 0）；
// 每一次发布（连空发布）都重写树表单元，mkfs 之后两次暖机是零单元发布；暖机直到两块盘上各有一条本实例的根；
// c2 之后新实例第一个 txg = max(环里最高, 记录最高) + 1。产物里 `name=anchor_dump` 那几行去掉前缀之后与 Python 脚本
// `--dump` 的 `DUMP` 行逐行比（报告里的命令）。第 6 次重跑登记第五节 E11：门槛那一处照第十三节命令四 `anchors_e156_r6.py` 改，
// 并进这次挂载读回全零、还没被写掉的槽（槽号即落到它的最小 txg；定乙），`h_缺`、`h_洞` 与两条闭式照旧读环里最旧的根。
// ============================================================================================
mod anchor_model;

#[allow(
    clippy::too_many_lines,
    reason = "第一段、第二段与全部判定行按登记第五节 5.5 的次序连着跑，拆开反而难与登记逐行对"
)]
fn main() {
    let mut emitter = Emitter { emitted: 0 };
    emit_static_anchors(&mut emitter);

    // 锚点模型（命令四的移植）：逐观测点那几行，去掉前缀之后与 Python 脚本的 `--dump` 逐行比（报告里的命令）。
    let mut anchors: BTreeMap<u64, AnchorCellsOfOneSlotsPerRegion> = BTreeMap::new();
    for slots_per_region in E156_SLOTS_PER_REGION_SAMPLING_POINTS {
        for line in anchor_model::dump_lines(slots_per_region) {
            emitter.emit(&format!("name=anchor_dump {line}"));
        }
        let cells = anchor_cells_of(slots_per_region);
        let lags = anchor_model::tree_table_reclaim_lags(slots_per_region, cells.without_holes.end);
        // 命令四的另两样：排得下的格上 r2 的两条闭式与按环现数逐点相同（A11），k = 0 的 Q1a / Q1b 与全部格的 Q1b 恒 0（A12、A13）。
        let fitting_rows = || {
            cells
                .cells
                .iter()
                .filter(|(_, _, cell)| cell.fits)
                .flat_map(|(_, _, cell)| cell.rows.iter())
        };
        let closed_form_mismatches = fitting_rows()
            .filter(|row| {
                row.live_holes != row.live_holes_closed_form
                    || row.missing_txgs != row.missing_txgs_closed_form
            })
            .count();
        let interval_rule_maximum = fitting_rows()
            .chain(cells.without_holes.rows.iter())
            .map(|row| row.interval_rule_tree_table_over_withheld)
            .max()
            .unwrap_or(0);
        let without_holes_floor_rule_maximum = cells
            .without_holes
            .rows
            .iter()
            .map(|row| row.tree_table_lower_bound)
            .max()
            .unwrap_or(0);
        emitter.emit(&format!(
            "name=anchor_model s={slots_per_region} k0_end={} a10_lags={} a11_closed_form_mismatches_on_fitting_cells={closed_form_mismatches} a12_k0_max_q1a={without_holes_floor_rule_maximum} a13_max_q1b={interval_rule_maximum} fits={}",
            cells.without_holes.end,
            set_text(&lags),
            cells
                .cells
                .iter()
                .map(|(count, position, cell)| format!("{count}{}:{}", position.label(), cell.fits))
                .collect::<Vec<_>>()
                .join(",")
        ));
        anchors.insert(slots_per_region, cells);
    }

    // 第 7 次重跑的锚点模型（登记第七节 7.2 的 `anchors_e156_r7.py` 的移植）：k_满 / 排得下的最大洞数与本地常量表回比（A20，不等即停机 S20），
    // 摘要与逐观测点两种行去掉前缀之后与脚本的输出逐行比（报告里的命令；摘要行的五个键名先照 `emit_hole_family_anchor_cells` 文档里那张对照换名）。
    let mut hole_family_anchors: BTreeMap<u64, HoleFamilyAnchorCellsOfOneSlotsPerRegion> =
        BTreeMap::new();
    for slots_per_region in E156_SLOTS_PER_REGION_SAMPLING_POINTS {
        let cells = hole_family_anchor_cells_of(slots_per_region);
        emit_hole_family_anchor_cells(&mut emitter, slots_per_region, &cells);
        hole_family_anchors.insert(slots_per_region, cells);
    }

    // ===================== 第一段 =====================
    let shape_of = |slots_per_region: u64,
                    density: WorkloadDensity,
                    hole_count: u64,
                    position: Option<HolePosition>| HhCellShape {
        slots_per_region,
        density,
        hole_count,
        position,
    };
    let premise_front_shape = shape_of(
        4,
        WorkloadDensity::EveryPublishOverwrites,
        1,
        Some(HolePosition::Front),
    );
    let premise_back_shape = shape_of(
        4,
        WorkloadDensity::EveryPublishOverwrites,
        1,
        Some(HolePosition::Back),
    );
    let premise_front = run_hh_history(
        &mut emitter,
        &hh_request(
            "Hh",
            premise_front_shape,
            anchor_cell_for(&anchors, 4, 1, Some(HolePosition::Front)),
        ),
    );
    let premise_back = run_hh_history(
        &mut emitter,
        &hh_request(
            "Hh",
            premise_back_shape,
            anchor_cell_for(&anchors, 4, 1, Some(HolePosition::Back)),
        ),
    );
    for (label, outcome) in [("front", &premise_front), ("back", &premise_back)] {
        for recovery in &outcome.recoveries {
            emitter.emit(&format!(
                "name=pq1 position={label} hole_txg={} prefix_applied={} first_txg_of_new_instance={} ring_has_root_at_hole_txg={} hole_slot_content={} a16_applicable={}",
                recovery.hole_txg,
                recovery.prefix_applied,
                recovery.first_txg_of_the_new_instance,
                recovery.ring_has_a_root_at_the_hole_txg,
                recovery.hole_slot_content,
                recovery.prefix_applied > 0
            ));
        }
    }
    let positive_control_crash = {
        let mut request = hh_request(
            "PC-c2",
            premise_back_shape,
            anchor_cell_for(&anchors, 4, 1, Some(HolePosition::Back)),
        );
        request.cut_point = CrashCutPoint::AfterTheRootSlotWritePositiveControl;
        request.emit_positive_control_no_reclaim = false;
        run_hh_history(&mut emitter, &request)
    };
    let positive_control_crash_seen = positive_control_crash
        .recoveries
        .first()
        .is_some_and(|recovery| recovery.ring_has_a_root_at_the_hole_txg)
        && positive_control_crash
            .observations
            .iter()
            .find(|observation| observation.point == ObservationPoint::MountReturn)
            .is_some_and(|observation| observation.reading.missing_txgs == 0)
        && judge_hole_history_outcome(&positive_control_crash, None).threshold_anchor_mismatches
            > 0;
    emitter.emit(&format!(
        "name=pc_c2 ring_has_root_at_hole_txg={} mount_return_h_que={} a14_mismatches={} seen={positive_control_crash_seen}",
        positive_control_crash
            .recoveries
            .first()
            .is_some_and(|recovery| recovery.ring_has_a_root_at_the_hole_txg),
        positive_control_crash
            .observations
            .iter()
            .find(|observation| observation.point == ObservationPoint::MountReturn)
            .map_or_else(|| "none".to_string(), |observation| observation.reading.missing_txgs.to_string()),
        judge_hole_history_outcome(&positive_control_crash, None).threshold_anchor_mismatches
    ));

    let floor_raise = run_floor_raise_trace();
    let pushed_roots_text: Vec<String> = floor_raise
        .pushed_roots
        .iter()
        .map(|(txg, root_floor, effective)| {
            format!("txg{txg}:root_floor{root_floor}:effective{effective}")
        })
        .collect();
    let premise_two_merged = floor_raise.the_two_readings_merge();
    emitter.emit(&format!(
        "name=pq2 raised={} floor_before={} new_floor={} effective_floor_before_any_write={} system_configuration_writes={} effective_floor_after_last_system_configuration_write={} system_configuration_writes_after_first_root={} pushed_roots={} publishes={} reclaimed_placements={} the_two_readings_merge={premise_two_merged}",
        floor_raise.raised,
        floor_raise.floor_before,
        floor_raise.new_floor,
        floor_raise.effective_floor_before_any_write,
        floor_raise.system_configuration_writes,
        floor_raise
            .effective_floor_after_the_last_system_configuration_write
            .map_or_else(|| "none".to_string(), |floor| floor.to_string()),
        floor_raise.system_configuration_writes_after_the_first_root,
        pushed_roots_text.join(","),
        floor_raise.publishes,
        floor_raise.reclaimed_placements,
    ));

    let forward_rollback = run_forward_rollback_scenario();
    let forward_rollback_structure_holds = forward_rollback.txg_of_the_rollback_publish
        == forward_rollback.txg_before_the_rollback + 1
        && forward_rollback.instance_after == forward_rollback.instance_before
        && forward_rollback.instance_table_rows_after
            == forward_rollback.instance_table_rows_before
        && forward_rollback.isolated_after_the_plain_remount
            == vec![(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)]
        && forward_rollback.abandoned_roots_after_the_plain_remount == 0;
    emitter.emit(&format!(
        "name=u11_forward_rollback txg_before={} txg_of_rollback_publish={} instance_before={} instance_after={} instance_table_rows_before={} instance_table_rows_after={} remount_instance={} isolated_after_plain_remount={:?} abandoned_roots_after_plain_remount={} forward_rollback_structure_holds={forward_rollback_structure_holds}",
        forward_rollback.txg_before_the_rollback,
        forward_rollback.txg_of_the_rollback_publish,
        forward_rollback.instance_before,
        forward_rollback.instance_after,
        forward_rollback.instance_table_rows_before,
        forward_rollback.instance_table_rows_after,
        forward_rollback.remount_instance,
        forward_rollback
            .isolated_after_the_plain_remount
            .iter()
            .map(|(device, slots)| (device.0, *slots))
            .collect::<Vec<_>>(),
        forward_rollback.abandoned_roots_after_the_plain_remount,
    ));
    let oldest_rollback = run_rollback_to_the_oldest_candidate();
    let u13a_structure_holds = oldest_rollback.txg_after == oldest_rollback.txg_before + 1
        && oldest_rollback.instance_after == oldest_rollback.instance_before
        && oldest_rollback.instance_table_rows_after == oldest_rollback.instance_table_rows_before;
    emitter.emit(&format!(
        "name=u13a_rollback_to_oldest_candidate overwrites={} target_txg={} txg_before={} txg_after={} instance_before={} instance_after={} instance_table_rows_before={} instance_table_rows_after={} item1_slots={} item5_slots={} i311={} structure_holds={u13a_structure_holds}",
        oldest_rollback.overwrites,
        oldest_rollback.target_txg,
        oldest_rollback.txg_before,
        oldest_rollback.txg_after,
        oldest_rollback.instance_before,
        oldest_rollback.instance_after,
        oldest_rollback.instance_table_rows_before,
        oldest_rollback.instance_table_rows_after,
        oldest_rollback.item1_slots,
        oldest_rollback.item5_slots,
        verdict_label(&oldest_rollback.allocated_minus_deferred_verdict),
    ));
    let synthetic = synthetic_base_with_zero_deferred();
    let (synthetic_flip1, synthetic_flip2) = q7c_self_test(
        &mut emitter,
        "beta_syn",
        &synthetic.pool,
        synthetic.accounting_slot,
        synthetic.referenced,
    );

    let premise_totals = {
        let mut totals = judge_hole_history_outcome(&premise_front, None);
        totals.add(judge_hole_history_outcome(&premise_back, None));
        totals
    };
    let premise_one_holds =
        premise_one_reproduced(&premise_front) && premise_one_reproduced(&premise_back);
    let stage_one_stop_reasons: Vec<&str> = [
        (!premise_one_holds, "sp1_premise_one_not_reproduced"),
        (
            premise_totals.device_and_crates_disagreements > 0,
            "s1_device_and_crates_disagree",
        ),
        (
            premise_totals.root_ring_missing > 0,
            "s1h_root_ring_not_installed",
        ),
        (
            premise_totals.slots_per_region_read_back_mismatches > 0,
            "k8_slots_per_region_read_back",
        ),
        (
            premise_totals.admission_touches > 0,
            "s12_admission_touched_the_history",
        ),
        (
            premise_totals.hole_count_anchor_mismatches > 0,
            "v6_hole_counts_differ_from_the_anchor",
        ),
        (
            premise_totals.threshold_anchor_mismatches > 0,
            "v2_threshold_differs_from_the_anchor",
        ),
        (!positive_control_crash_seen, "v2_pc_c2_not_seen"),
    ]
    .into_iter()
    .filter(|(triggered, _)| *triggered)
    .map(|(_, reason)| reason)
    .collect();
    emitter.emit(&format!(
        "name=verdict stage=first premise_one_reproduced={premise_one_holds} premise_two_the_two_readings_merge={premise_two_merged} premise_two_answer={} fork_three={} g_adm_nonzero={} s1_failures={} a11_mismatches={} a14_mismatches={} pc_c2_seen={positive_control_crash_seen} k11_holds={} forward_rollback_structure_holds={forward_rollback_structure_holds} u13a_structure_holds={u13a_structure_holds} u13b_flip1={synthetic_flip1} u13b_flip2_not_applicable={} stop_reasons={}",
        if premise_two_merged { "not_holds" } else { "holds" },
        if premise_two_merged { "void_not_measured" } else { "sp2_needs_a_new_registration" },
        premise_totals.admission_touches,
        premise_totals.device_and_crates_disagreements,
        premise_totals.hole_count_anchor_mismatches,
        premise_totals.threshold_anchor_mismatches,
        floor_raise.effective_floor_after_the_last_system_configuration_write == Some(floor_raise.new_floor),
        !synthetic_flip2,
        if stage_one_stop_reasons.is_empty() { "none".to_string() } else { stage_one_stop_reasons.join(",") },
    ));
    if !stage_one_stop_reasons.is_empty() {
        emitter.emit("name=stage_two_not_run reason=stage_one_stop");
        emitter.finish();
        return;
    }

    // ===================== 第二段 =====================
    let judge_cases = judge_positive_control_cases();
    let judge_wrong = judge_cases
        .iter()
        .filter(|(_, actual, expected)| actual != expected)
        .count();
    for (case, actual, expected) in &judge_cases {
        emitter.emit(&format!(
            "name=pc_judge case={case} actual={actual} expected={expected} holds={}",
            actual == expected
        ));
    }
    // PC-档序列（第 7 次重跑登记 5.3）：W28 的判定函数在五个合成序列上逐条判对，错一条即作废 V20。
    let tier_sequence_cases = tier_sequence_positive_control_cases();
    let tier_sequence_cases_wrong = tier_sequence_cases
        .iter()
        .filter(|(_, actual, expected)| actual != expected)
        .count();
    for (case, actual, expected) in &tier_sequence_cases {
        emitter.emit(&format!(
            "name=pc_tier_sequence case={case} actual={actual} expected={expected} holds={}",
            actual == expected
        ));
    }

    let mut outcomes: Vec<HhRunOutcome> = Vec::new();
    let mut grid_totals = HoleHistoryVerdictTotals::default();
    let mut every_publish_tree_table_lag_mismatch_cells = 0u64;
    let mut cells_run_without_fitting_the_ring = 0u64;
    let mut all_zero_pin_cells: Vec<AllZeroPinCellTotals> = Vec::new();
    // 第 7 次重跑登记 5.2 / W30 / W31：族表按 S 现算（Hh 前 k = 1 … k_满 + 1、Hh 后 k = 1 … k_满后、Hc 两档、Hs 六格，加 k = 0 的基准族），排不下的格照跑。
    for slots_per_region in E156_SLOTS_PER_REGION_SAMPLING_POINTS {
        let hole_family_cells = &hole_family_anchors[&slots_per_region];
        let expected_lags = anchor_model::tree_table_reclaim_lags(
            slots_per_region,
            hole_family_cells.without_holes.end,
        );
        for density in [
            WorkloadDensity::EveryPublishOverwrites,
            WorkloadDensity::OneOverwriteInFourPublishes,
        ] {
            let mut grid_cells: Vec<(&'static str, HhCellShape, &anchor_model::AnchorCell)> =
                vec![(
                    E156_FAMILY_HOLE_COUNT_SWEEP,
                    shape_of(slots_per_region, density, 0, None),
                    &hole_family_cells.without_holes,
                )];
            for cell in &hole_family_cells.cells {
                grid_cells.push((
                    cell.family,
                    shape_of(
                        slots_per_region,
                        density,
                        cell.hole_count,
                        Some(cell.position),
                    ),
                    &cell.anchor,
                ));
            }
            for (family, shape, anchor) in grid_cells {
                if !anchor.fits {
                    cells_run_without_fitting_the_ring += 1;
                }
                let reuse = if slots_per_region == 4
                    && density == WorkloadDensity::EveryPublishOverwrites
                    && family == E156_FAMILY_HOLE_COUNT_SWEEP
                    && shape.hole_count == 1
                {
                    match shape.position {
                        Some(HolePosition::Front) => Some(premise_front.clone()),
                        Some(HolePosition::Back) => Some(premise_back.clone()),
                        None => None,
                    }
                } else {
                    None
                };
                let outcome = match reuse {
                    Some(outcome) => {
                        emitter.emit(&format!(
                            "name=q1_cell_reused {} from=stage_one_premise_history",
                            shape.fields()
                        ));
                        outcome
                    }
                    None => run_hh_history(&mut emitter, &hh_request(family, shape, anchor)),
                };
                let lags = if shape.hole_count == 0 {
                    Some(&expected_lags)
                } else {
                    None
                };
                let totals = judge_hole_history_outcome(&outcome, lags);
                every_publish_tree_table_lag_mismatch_cells +=
                    totals.tree_table_lag_every_publish_mismatch;
                emit_cell_checks(&mut emitter, &outcome, &totals);
                all_zero_pin_cells.push(emit_all_zero_pin_cell(&mut emitter, &outcome));
                grid_totals.add(totals);
                outcomes.push(outcome);
            }
        }
    }

    // PC-钉门槛（第 5 次重跑登记 5.3、5.4、5.5 E2；替 PC-分配器）：格表里本来就跑的 Hh(1, 后, S, ρ) 六格，洞让 txg t − 3S 的旧根
    // 留在 t 的槽里，真实分配器的回收门槛钉在它上面直到 t + 3S（K15、L1），释放代 6 的树表滞后 = 6S − 1。把这一格交给同一个
    // k = 0 的 A10 判定（期望 = 同一 S 的 {3S}）：判定判「不符」只证判定不是恒绿，「含 6S − 1」证量法读到了真实分配器的推迟，两句各报各的。
    // 六格不进 grid_totals；6S − 1 用本地的 S 现算（2 × 区域数 × S − 1）。
    let mut pinned_floor_cells = 0u64;
    let mut pinned_floor_seen_real_cells = 0u64;
    let mut pinned_floor_seen_every_cells = 0u64;
    for outcome in outcomes.iter().filter(|outcome| {
        outcome.family == E156_FAMILY_HOLE_COUNT_SWEEP
            && outcome.shape.hole_count == 1
            && outcome.shape.position == Some(HolePosition::Back)
    }) {
        let slots_per_region = outcome.shape.slots_per_region;
        let expected_lags = anchor_model::tree_table_reclaim_lags(
            slots_per_region,
            anchors[&slots_per_region].without_holes.end,
        );
        let totals = judge_hole_history_outcome(outcome, Some(&expected_lags));
        let bound = 2 * E156_ROOT_RING_REGIONS * slots_per_region - 1;
        let contains_bound_real = outcome.tree_table_lags_real.contains(&bound);
        let contains_bound_every = outcome.tree_table_lags_every_publish.contains(&bound);
        let seen_real = totals.tree_table_lag_real_mismatch == 1 && contains_bound_real;
        let seen_every = totals.tree_table_lag_every_publish_mismatch == 1 && contains_bound_every;
        pinned_floor_cells += 1;
        pinned_floor_seen_real_cells += u64::from(seen_real);
        pinned_floor_seen_every_cells += u64::from(seen_every);
        emitter.emit(&format!(
            "name=pc_pinned_floor {} expected={} lags_real={} lags_every={} judge_real_mismatch={} judge_every_mismatch={} bound={bound} contains_bound_real={contains_bound_real} contains_bound_every={contains_bound_every} seen_real={seen_real} seen_every={seen_every}",
            outcome.shape.fields(),
            set_text(&expected_lags),
            set_text(&outcome.tree_table_lags_real),
            set_text(&outcome.tree_table_lags_every_publish),
            totals.tree_table_lag_real_mismatch,
            totals.tree_table_lag_every_publish_mismatch,
        ));
    }
    emitter.emit(&format!(
        "name=pc_pinned_floor_summary cells={pinned_floor_cells} seen_real_cells={pinned_floor_seen_real_cells} seen_every_cells={pinned_floor_seen_every_cells}"
    ));
    // 登记写死六格：S ∈ {4, 8, 16} × ρ ∈ {1, 1/4}，全都排得下。
    let pinned_floor_seen =
        pinned_floor_cells == 6 && pinned_floor_seen_every_cells == pinned_floor_cells;

    // PC-多扣：每个 (S, ρ) 的 Hh(0, —) 与 Hh(2, 后)，四个组合的期末值都要 > 0。
    let mut over_withheld_controls_void = 0u64;
    for outcome in outcomes.iter().filter(|outcome| {
        outcome.family == E156_FAMILY_HOLE_COUNT_SWEEP
            && (outcome.shape.hole_count == 0
                || (outcome.shape.hole_count == 2
                    && outcome.shape.position == Some(HolePosition::Back)))
    }) {
        let last = outcome
            .observations
            .last()
            .map(|observation| observation.reading.device_zero);
        let finals = last.map_or([0u64; 4], |device_zero| {
            [
                device_zero.no_reclaim_floor_rule_real,
                device_zero.no_reclaim_floor_rule_every_publish,
                device_zero.no_reclaim_interval_rule_real,
                device_zero.no_reclaim_interval_rule_every_publish,
            ]
        });
        let void = finals.iter().filter(|value| **value == 0).count();
        over_withheld_controls_void += u64::try_from(void).expect("落在 u64 内");
        emitter.emit(&format!(
            "name=pc_over_withheld {} final_floor_rule_real={} final_floor_rule_every={} final_interval_rule_real={} final_interval_rule_every={} seen={}",
            outcome.shape.fields(),
            finals[0],
            finals[1],
            finals[2],
            finals[3],
            void == 0
        ));
    }

    // PC-洞：h_缺、h_洞 两个计数器改成恒 0，A11 在每一条 k ≥ 1 的族上都要判红。
    let mut hole_controls_not_seen = 0u64;
    for outcome in outcomes
        .iter()
        .filter(|outcome| outcome.shape.hole_count >= 1)
    {
        let red_points = outcome
            .observations
            .iter()
            .filter(|observation| {
                observation
                    .anchor
                    .is_some_and(|row| row.missing_txgs != 0 || row.live_holes != 0)
            })
            .count();
        if red_points == 0 {
            hole_controls_not_seen += 1;
        }
        emitter.emit(&format!(
            "name=pc_hole_counters_zeroed {} a11_red_points={red_points} seen={}",
            outcome.shape.fields(),
            red_points > 0
        ));
    }

    // PC-同槽（第 7 次重跑登记 5.3、E25）：每个 (S, ρ) 的 Hs(2, 后) 再跑一遍，「每」列的旁路门槛把第一个洞的环槽当作这个进程写过（去掉那一槽里的旧根重算，
    // 不碰真实分配器）；「每」列 Q1a 峰值必须比不改时低，否则 V21（Hs 两族的读数分不出同槽钉住与别的多扣来源）。对照格不进 grid_totals、不进统计单位。
    let mut same_slot_controls: Vec<SameSlotControlReading> = Vec::new();
    for slots_per_region in E156_SLOTS_PER_REGION_SAMPLING_POINTS {
        for density in [
            WorkloadDensity::EveryPublishOverwrites,
            WorkloadDensity::OneOverwriteInFourPublishes,
        ] {
            let unchanged = outcomes
                .iter()
                .find(|outcome| {
                    outcome.family == E156_FAMILY_SAME_RING_SLOT
                        && outcome.shape.slots_per_region == slots_per_region
                        && outcome.shape.density == density
                        && outcome.shape.position == Some(HolePosition::Back)
                        && outcome.shape.hole_count == 2
                })
                .expect("格表里有 Hs(2, 后)");
            let anchor = &hole_family_anchors[&slots_per_region]
                .cells
                .iter()
                .find(|cell| {
                    cell.family == E156_FAMILY_SAME_RING_SLOT
                        && cell.position == HolePosition::Back
                        && cell.hole_count == 2
                })
                .expect("族表里有 Hs(2, 后)")
                .anchor;
            let mut request = hh_request(E156_FAMILY_SAME_SLOT_CONTROL, unchanged.shape, anchor);
            request.treat_the_hole_ring_slot_as_rewritten = true;
            let control = run_hh_history(&mut emitter, &request);
            let reading = SameSlotControlReading {
                unchanged_peak_every_publish: q1a_peak_of(unchanged, ReclaimTiming::EveryPublish).0,
                control_peak_every_publish: q1a_peak_of(&control, ReclaimTiming::EveryPublish).0,
            };
            emitter.emit(&format!(
                "name=pc_same_slot {} unchanged_q1a_every_peak={} control_q1a_every_peak={} control_q1a_real_peak={} seen={}",
                unchanged.shape.fields(),
                reading.unchanged_peak_every_publish,
                reading.control_peak_every_publish,
                q1a_peak_of(&control, ReclaimTiming::Real).0,
                reading.control_peak_every_publish < reading.unchanged_peak_every_publish
            ));
            same_slot_controls.push(reading);
        }
    }

    // R7a–R7e：每个统计单位 (量, S, ρ, 回收时点) 的档序列判 W28、报上界数；R7f（每档的多扣）在 `name=q1_trajectory ... quantity=q1a_real` 与 `q1a_every` 行。
    let tier_units = judge_and_emit_tier_units(&mut emitter, &outcomes, &hole_family_anchors);
    emit_hole_family_verdict(
        &mut emitter,
        &tier_units,
        tier_sequence_cases_wrong,
        &same_slot_controls,
    );

    // 分组、判定。
    let mut groups: BTreeMap<GroupKey, Vec<DeltaSample>> = BTreeMap::new();
    for outcome in &outcomes {
        if outcome.truncation.is_some() {
            emitter.emit(&format!(
                "name=q1_cell_truncated {} note=only_points_before_truncation_are_used",
                outcome.shape.fields()
            ));
        }
        // 第 6 次重跑以来的 Q1c / Q1d 分组只收那一次跑的 Hh(0, 1, 2, 4) 格，Hc、Hs 与 k = 3、5 … 的格不混进去（它们的判定在 `name=r7_*` 行）。
        let feeds_the_groups_of_the_earlier_reruns = outcome.family == E156_FAMILY_HOLE_COUNT_SWEEP
            && matches!(outcome.shape.hole_count, 0 | 1 | 2 | 4);
        if !feeds_the_groups_of_the_earlier_reruns {
            continue;
        }
        for timing in [ReclaimTiming::Real, ReclaimTiming::EveryPublish] {
            let key = (
                outcome.shape.slots_per_region,
                density_key(outcome.shape.density),
                timing,
            );
            let entry = groups.entry(key).or_default();
            for observation in &outcome.observations {
                entry.push(DeltaSample {
                    shape: outcome.shape,
                    txg: observation.txg,
                    workload_publishes: observation.workload_publishes,
                    missing_txgs: observation.reading.missing_txgs,
                    live_holes: observation.reading.live_holes,
                    delta: observation.delta(timing),
                });
            }
        }
    }
    let mut judgements: BTreeMap<GroupKey, GroupJudgements> = BTreeMap::new();
    for (key, samples) in &groups {
        judgements.insert(*key, judge_group(&mut emitter, key, samples));
    }
    let sensitivity = |pick: &dyn Fn(&GroupJudgements) -> PeakJudgement| {
        let values: Vec<PeakJudgement> = judgements.values().map(pick).collect();
        let listing: Vec<String> = judgements
            .iter()
            .map(|(key, judgement)| {
                format!(
                    "{}:{}",
                    group_label(key).replace(' ', "_"),
                    pick(judgement).label()
                )
            })
            .collect();
        (judge_geometry_sensitivity(&values), listing.join(","))
    };
    for (quantity, pick) in [
        (
            "q1c",
            &(|judgement: &GroupJudgements| judgement.all_points)
                as &dyn Fn(&GroupJudgements) -> PeakJudgement,
        ),
        ("q1c_on_holes", &|judgement: &GroupJudgements| {
            judgement.on_holes
        }),
        ("q1c_front", &|judgement: &GroupJudgements| {
            judgement.front_holes
        }),
        ("q1c_back", &|judgement: &GroupJudgements| {
            judgement.back_holes
        }),
    ] {
        let (result, listing) = sensitivity(pick);
        emitter.emit(&format!(
            "name=geometry_sensitivity quantity={quantity} result={} per_sampling_point={listing}",
            result.label()
        ));
    }
    let growth_listing: Vec<String> = judgements
        .iter()
        .map(|(key, judgement)| {
            format!(
                "{}:{}",
                group_label(key).replace(' ', "_"),
                judgement.growth.label()
            )
        })
        .collect();
    let growth_values: BTreeSet<&str> = judgements
        .values()
        .map(|judgement| judgement.growth.label())
        .collect();
    emitter.emit(&format!(
        "name=geometry_sensitivity quantity=q1d_growth result={} per_sampling_point={}",
        if growth_values.len() == 1 {
            format!("consistent_on_{}_sampling_points", judgements.len())
        } else {
            "unstable".to_string()
        },
        growth_listing.join(",")
    ));

    // 失败条款（第十节）。
    let front_above_two_groups: Vec<String> = judgements
        .iter()
        .filter(|(_, judgement)| judgement.front_holes == PeakJudgement::AboveTheThreshold)
        .map(|(key, _)| group_label(key).replace(' ', "_"))
        .collect();
    let no_observable_difference = judgements.values().all(|judgement| {
        judgement.front_peak_is_zero_everywhere && judgement.back_peak_is_zero_everywhere
    });
    let front_back_differ_groups: Vec<String> = judgements
        .iter()
        .filter(|(_, judgement)| {
            judgement.front_holes != PeakJudgement::NothingObserved
                && judgement.back_holes != PeakJudgement::NothingObserved
                && judgement.front_holes != judgement.back_holes
        })
        .map(|(key, _)| group_label(key).replace(' ', "_"))
        .collect();
    let back_at_most_two_groups: Vec<String> = judgements
        .iter()
        .filter(|(_, judgement)| judgement.back_holes == PeakJudgement::AtMostTheThreshold)
        .map(|(key, _)| group_label(key).replace(' ', "_"))
        .collect();
    let front_one_slot: Vec<String> = judgements
        .iter()
        .map(|(key, judgement)| {
            format!(
                "{}:{}",
                group_label(key).replace(' ', "_"),
                judgement.front_holes_one_slot.label()
            )
        })
        .collect();
    let not_on_holes: Vec<String> = judgements
        .iter()
        .map(|(key, judgement)| {
            format!(
                "{}:{}",
                group_label(key).replace(' ', "_"),
                judgement.not_on_holes.label()
            )
        })
        .collect();
    let list_or_none = |values: &[String]| {
        if values.is_empty() {
            "none".to_string()
        } else {
            values.join(",")
        }
    };
    emitter.emit(&format!(
        "name=failure_clauses f1_front_above_two_groups={} f2_no_observable_difference={no_observable_difference} f22_front_and_back_differ_groups={} f23_back_at_most_two_groups={} f24_real_every_differ_points={} q1c_front_one_slot={} q1c_not_on_holes={}",
        list_or_none(&front_above_two_groups),
        list_or_none(&front_back_differ_groups),
        list_or_none(&back_at_most_two_groups),
        grid_totals.real_every_differ,
        front_one_slot.join(","),
        not_on_holes.join(","),
    ));
    // 第 6 次重跑登记 E10：定乙钉住的汇总（逐格的和）。
    let sum_of = |pick: fn(&AllZeroPinCellTotals) -> u64| -> u64 {
        all_zero_pin_cells.iter().map(pick).sum()
    };
    emitter.emit(&format!(
        "name=all_zero_pin_summary cells={} cells_with_pins={} pins_installed={} threshold_lowered_points={} released_by_reread_only={} bad_slots_at_mount={} bad_slots_still_pinned={}",
        all_zero_pin_cells.len(),
        all_zero_pin_cells
            .iter()
            .filter(|cell| cell.pins_installed > 0)
            .count(),
        sum_of(|cell| cell.pins_installed),
        sum_of(|cell| cell.threshold_lowered_points),
        sum_of(|cell| cell.released_by_reread_only),
        sum_of(|cell| cell.bad_slots_at_mount),
        sum_of(|cell| cell.bad_slots_still_pinned),
    ));

    // 作废与停机（第十一节）。
    let cells_run = outcomes.len();
    emitter.emit(&format!(
        "name=verdict stage=second cells_run={cells_run} cells_run_without_fitting_the_ring={cells_run_without_fitting_the_ring} truncated_cells={} v1_over_withheld_controls_void={over_withheld_controls_void} v2_a10_every_mismatch_cells={every_publish_tree_table_lag_mismatch_cells} v2_a12_every_nonzero_points={} v2_a14_mismatch_points={} v2_a15_every_below_points={} v2_pc_pinned_floor_seen={pinned_floor_seen} v2_pc_c2_seen={positive_control_crash_seen} v6_a11_mismatch_points={} v6_devices_unequal_points={} v6_pc_hole_not_seen_cells={hole_controls_not_seen} v16_judge_cases_wrong={judge_wrong} v17_a13_every_nonzero_points={} s1_failures={} s1h_root_ring_missing_points={} s11_a10_real_mismatch_cells={} s11_a12_real_nonzero_points={} s11_a15_real_below_points={} s11_k10_violation_points={} s12_g_adm_nonzero={} k8_mismatches={} referenced_but_free_points={} abandoned_or_unknown_root_points={} anchor_rows_missing={} anchor_lag_mismatch_points={} anchor_pinned_slot_mismatch_points={}",
        outcomes.iter().filter(|outcome| outcome.truncation.is_some()).count(),
        grid_totals.every_publish_nonzero_without_holes,
        grid_totals.threshold_anchor_mismatches,
        grid_totals.every_publish_below_tree_table_bound,
        grid_totals.hole_count_anchor_mismatches,
        grid_totals.devices_unequal,
        grid_totals.interval_rule_every_publish_nonzero,
        grid_totals.device_and_crates_disagreements,
        grid_totals.root_ring_missing,
        grid_totals.tree_table_lag_real_mismatch,
        grid_totals.real_nonzero_without_holes,
        grid_totals.real_below_tree_table_bound,
        grid_totals.reclaim_rule_violation_points,
        grid_totals.admission_touches,
        grid_totals.slots_per_region_read_back_mismatches,
        grid_totals.q1a_plus_positive,
        grid_totals.abandoned_or_unknown,
        grid_totals.anchor_rows_missing,
        grid_totals.lag_anchor_mismatches,
        grid_totals.pinned_anchor_mismatches,
    ));
    for (key, judgement) in &judgements {
        emitter.emit(&format!(
            "name=verdict stage=second_group {} q1c={} q1c_on_holes={} q1c_front={} q1c_back={} q1d_growth={}",
            group_label(key),
            judgement.all_points.label(),
            judgement.on_holes.label(),
            judgement.front_holes.label(),
            judgement.back_holes.label(),
            judgement.growth.label(),
        ));
    }
    emitter.finish();
}

#[cfg(test)]
mod tests;
