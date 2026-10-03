#!/usr/bin/env python3
"""m3-formal-r3 攻方腿：给冻结副本的拷贝打运行时开关与计数（只在副本里用；判据原文一条不改）。

用法：python3 patch_attack.py <仓副本根>
每处替换要求原文在文件里恰好命中一次，否则退出码 3、不写。
开关（环境变量，0 或不设 = 原文）：
  ATTACK_CORE  取前缀核 crates/singlefs-replay-prefix/src/lib.rs：
     1 jsn 对不上不断（I-8.3 那一判，七·2 的必红世界）  2 表第 36 行（无锚点不看 txg）
     3 Z1 一条都不施加  4 Z2 不看在飞上限  5 表第 406 行（每条都当末条）  6 水位之上两条以上就一条都不施加
     7 表第 516 行（末条不带提交标记也施加）  8 表第 1003 行（txg 顶格不判）  9 表第 1004 行（txg 差一格就报顶格）
  ATTACK_GLUE  recovery.rs 拼根：1 实例表指针取记录的映射根指针  2 卸载记号照抄所选根  3 分配记录树根清零
  ATTACK_GEN   E165 生成器（位）：1 第 0 版是写行发布、分配记录树根非零，其余版全零  2 每一版的根都带卸载记号
  ATTACK_SKIP_PRELUDE=1  跳过自检与阳性对照（变异跑用；计数不受影响）
"""
import sys
from pathlib import Path

root = Path(sys.argv[1])
CORE = root / "crates/singlefs-replay-prefix/src/lib.rs"
GLUE = root / "crates/singlefs-core/src/recovery.rs"
E165 = root / "crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs"

edits = {CORE: [], GLUE: [], E165: []}

def rep(path, old, new):
    edits[path].append((old, new))

# ---------- 核 ----------
rep(CORE, "use vstd::prelude::*;\n",
"""use vstd::prelude::*;

/// 攻方第三轮的运行时开关（只在副本里）：0 是原文。
pub fn attack_core_switch() -> u32 {
    static SWITCH: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *SWITCH.get_or_init(|| {
        std::env::var("ATTACK_CORE").ok().and_then(|value| value.parse().ok()).unwrap_or(0)
    })
}
""")
rep(CORE, """        decreases records.len() - index,
    {
        let record = &records[index];
        // [原文 3078–3080] above 的筛选条件，同上面那一遍
""", """        decreases records.len() - index,
    {
        if attack_core_switch() == 3 || (attack_core_switch() == 6 && report.above_water >= 2) {
            break ;
        }
        let record = &records[index];
        // [原文 3078–3080] above 的筛选条件，同上面那一遍
""")
rep(CORE, "        if taken >= in_flight_limit {\n", "        if taken >= in_flight_limit && attack_core_switch() != 4 {\n")
rep(CORE, "                if record.instance != expected_key.0 || record.counter != expected_key.1 {\n",
          "                if (record.instance != expected_key.0 || record.counter != expected_key.1) && attack_core_switch() != 1 {\n")
rep(CORE, "                if record.checkpoint_txg != chain_start_txg_without_anchor\n",
          "                if (attack_core_switch() != 2 && record.checkpoint_txg != chain_start_txg_without_anchor)\n")
rep(CORE, "        if !record_ends_its_publish(record) {\n", "        if !record_ends_its_publish(record) && attack_core_switch() != 5 {\n")
rep(CORE, "        if !record.is_commit && record_ends_its_publish(record) {\n",
          "        if !record.is_commit && record_ends_its_publish(record) && attack_core_switch() != 7 {\n")
rep(CORE, "    if root_checkpoint_txg == u64::MAX {\n",
          "    if (attack_core_switch() != 8 && root_checkpoint_txg == u64::MAX) || (attack_core_switch() == 9 && root_checkpoint_txg >= u64::MAX - 1) {\n")

# ---------- 胶水 ----------
rep(GLUE, "/// 所选根的 checkpoint_txg 是 u64 的最大值时 [`replay_journal`] 报",
"""/// 攻方第三轮的运行时开关（只在副本里）：0 是原文。
fn attack_glue_switch() -> u32 {
    static SWITCH: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *SWITCH.get_or_init(|| {
        std::env::var("ATTACK_GLUE").ok().and_then(|value| value.parse().ok()).unwrap_or(0)
    })
}

/// 所选根的 checkpoint_txg 是 u64 的最大值时 [`replay_journal`] 报""")
rep(GLUE, "                unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,\n",
          "                unmount_marker: if attack_glue_switch() == 2 { root.unmount_marker } else { UnmountMarker::NotWrittenByTheUnmountSequence },\n")
rep(GLUE, "                instance_table: root.instance_table,\n",
          "                instance_table: if attack_glue_switch() == 1 { record.new_mapping_root } else { root.instance_table },\n")
rep(GLUE, "                allocation_record_tree_root: root.allocation_record_tree_root,\n",
          "                allocation_record_tree_root: if attack_glue_switch() == 3 { NodePointer::empty_root() } else { root.allocation_record_tree_root },\n")

# ---------- E165 ----------
rep(E165, """            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance: InstanceGeneration(self.instance),""",
"""            unmount_marker: if attack_gen_mode() & 2 != 0 {
                UnmountMarker::WrittenByTheUnmountSequence
            } else {
                UnmountMarker::NotWrittenByTheUnmountSequence
            },
            instance: InstanceGeneration(self.instance),""")
rep(E165, "            allocation_record_tree_root: NodePointer::empty_root(),\n        }\n    }\n}\n",
"""            allocation_record_tree_root: if attack_gen_mode() & 1 != 0
                && self.segment.tree_identifier_watermark == 1000
            {
                node_pointer_at_slot(68_000, 1)
            } else {
                NodePointer::empty_root()
            },
        }
    }
}
""")
rep(E165, """    // J4 最远一版：j = j*。
    if landing != Some(truth.farthest_version) {
        tally.add_violation(Counter::J4Violations, &describe_witness);
    }
""", """    // J4 最远一版：j = j*。
    if landing != Some(truth.farthest_version) {
        tally.add_violation(Counter::J4Violations, &describe_witness);
    }
    attack_judge_a_legal_state(
        state,
        &first,
        landing,
        &chosen_root,
        &records,
        &disk,
        &truth,
        [returned_root_holds, root_plus_disk_holds, within_the_in_flight_limit],
        &describe_witness,
        tally,
    );
""")
rep(E165, """        StatesHoldingARecordRejectedByParse => "states_holding_a_record_rejected_by_parse",
    }""", """        StatesHoldingARecordRejectedByParse => "states_holding_a_record_rejected_by_parse",
        AtkFullRootMismatch => "atk_full_root_mismatch",
        AtkS1Violations => "atk_s1_violations",
        AtkS1OtherFieldsDiffer => "atk_s1_other_fields_differ",
        AtkS1MarkerDiffers => "atk_s1_marker_differs",
        AtkS1AllocDiffers => "atk_s1_alloc_differs",
        AtkS1InstanceTableDiffers => "atk_s1_instance_table_differs",
        AtkUnmountShapeStates => "atk_unmount_shape_states",
        AtkUnmountShapeMarkerDiffers => "atk_unmount_shape_marker_differs",
        AtkLegalStates => "atk_legal_states",
        AtkLegalOwnStates => "atk_legal_own_states",
        AtkLegalJ1 => "atk_legal_j1",
        AtkLegalJ2 => "atk_legal_j2",
        AtkLegalJ3 => "atk_legal_j3",
        AtkLegalJ4 => "atk_legal_j4",
        AtkLegalOwnJ1 => "atk_legal_own_j1",
        AtkLegalOwnJ2 => "atk_legal_own_j2",
        AtkLegalOwnJ3 => "atk_legal_own_j3",
        AtkLegalOwnJ4 => "atk_legal_own_j4",
        AtkLegalS1MarkerDiffers => "atk_legal_s1_marker_differs",
        AtkLegalOwnS1MarkerDiffers => "atk_legal_own_s1_marker_differs",
        AtkLegalFullRootMismatch => "atk_legal_full_root_mismatch",
        AtkLegalJ6PremiseFails => "atk_legal_j6_premise_fails",
        AtkJ6WritingAJudged => "atk_j6_writing_a_judged",
        AtkJ6WritingAViolations => "atk_j6_writing_a_violations",
        AtkJ6WritingBJudged => "atk_j6_writing_b_judged",
        AtkJ6WritingBViolations => "atk_j6_writing_b_violations",
        AtkK10States => "atk_k10_states",
        AtkK10AsWholeTable => "atk_k10_as_whole_table",
        AtkK10AsGapContinues => "atk_k10_as_gap_continues",
        AtkK10AsGapStops => "atk_k10_as_gap_stops",
        AtkK10J6PremiseHolds => "atk_k10_j6_premise_holds",
        AtkDiffWcImplW => "atk_diff_wc_impl_whole",
        AtkDiffWcImplC => "atk_diff_wc_impl_continues",
        AtkDiffWcImplNone => "atk_diff_wc_impl_neither",
        AtkDiffCsImplC => "atk_diff_cs_impl_continues",
        AtkDiffCsImplS => "atk_diff_cs_impl_stops",
        AtkDiffCsImplNone => "atk_diff_cs_impl_neither",
        AtkDiffWsImplW => "atk_diff_ws_impl_whole",
        AtkDiffWsImplS => "atk_diff_ws_impl_stops",
        AtkDiffWsImplNone => "atk_diff_ws_impl_neither",
        AtkK10DiffWcImplW => "atk_k10_diff_wc_impl_whole",
        AtkK10DiffWcImplC => "atk_k10_diff_wc_impl_continues",
        AtkK10DiffWcImplNone => "atk_k10_diff_wc_impl_neither",
        AtkK10DiffCsImplC => "atk_k10_diff_cs_impl_continues",
        AtkK10DiffCsImplS => "atk_k10_diff_cs_impl_stops",
        AtkK10DiffCsImplNone => "atk_k10_diff_cs_impl_neither",
        AtkK10DiffWsImplW => "atk_k10_diff_ws_impl_whole",
        AtkK10DiffWsImplS => "atk_k10_diff_ws_impl_stops",
        AtkK10DiffWsImplNone => "atk_k10_diff_ws_impl_neither",
    }""")
rep(E165, "    let mut presence = vec![RecordPresence::OnDisk(UnitState::BothCopiesGood); history.slots.len()];\n",
"""    let mut presence = vec![RecordPresence::OnDisk(UnitState::BothCopiesGood); history.slots.len()];
    let (attack_segments_own, attack_segments_with) = {
        let mut own: Vec<u32> = Vec::new();
        let mut with: Vec<u32> = Vec::new();
        for version in 1..=root_write_count {
            let record_bits = history
                .record_indexes_of_publish(version)
                .fold(0u32, |bits, record_index| bits | (1 << (record_index - 1)));
            let root_bit = 1u32 << (later_record_count + version - 1);
            own.push(record_bits);
            own.push(root_bit);
            with.push(record_bits | root_bit);
        }
        (own, with)
    };
""")
rep(E165, "        let today_interval = judge_a_legal_state(\n",
"""        ATTACK_LEGAL_CRASH_STATE.with(|flag| {
            flag.set(
                u8::from(stop_segment_mask(state_mask, &attack_segments_own) != 0)
                    | (u8::from(stop_segment_mask(state_mask, &attack_segments_with) != 0) << 1),
            );
        });
        let today_interval = judge_a_legal_state(
""")
rep(E165, "        if today_interval.is_none() {\n            tally.add(Counter::PairStatesWithoutAnInterval);\n",
"""        ATTACK_LEGAL_CRASH_STATE.with(|flag| flag.set(0));
        if today_interval.is_none() {
            tally.add(Counter::PairStatesWithoutAnInterval);
""")
rep(E165, """        match &first {
            ReplayOutcome::Panicked => tally.add_violation(Counter::Panicked, &describe_witness),
""", """        attack_judge_adversarial(&records, &first, &expectations, tally);
        match &first {
            ReplayOutcome::Panicked => tally.add_violation(Counter::Panicked, &describe_witness),
""")
rep(E165, """                judge_the_fixed_point(
                    self.implementation,
                    returned,
                    None,
                    above_water_record_count <= in_flight_limit,
                    &records,
                    &disk,
                    true,
                    &describe_witness,
                    tally,
                );
""", """                let attack_j6_before = tally.get(Counter::J6PremiseHoldsViolations);
                judge_the_fixed_point(
                    self.implementation,
                    returned,
                    None,
                    above_water_record_count <= in_flight_limit,
                    &records,
                    &disk,
                    true,
                    &describe_witness,
                    tally,
                );
                if attack_k10_holds(&records)
                    && tally.get(Counter::J6PremiseHoldsViolations) > attack_j6_before
                {
                    tally.add_violation(Counter::AtkK10J6PremiseHolds, &describe_witness);
                }
""")
rep(E165, """    let place = format!(
        "domain={} ring_slots={} n={size}",
        domain.label(),
        domain.ring().ring_slots
    );
""", """    let place = format!(
        "domain={} ring_slots={} n={size}",
        domain.label(),
        domain.ring().ring_slots
    );
    emitter.result(&attack_row(&place, tally));
""")
rep(E165, "    for (check_name, check) in SELF_CHECKS {\n",
          "    for (check_name, check) in if attack_skip_prelude() { &[][..] } else { SELF_CHECKS } {\n")
rep(E165, "    let control_readings = positive_control_readings(&selection.families, thread_count);\n",
"""    let control_readings = if attack_skip_prelude() {
        Vec::new()
    } else {
        positive_control_readings(&selection.families, thread_count)
    };
""")
rep(E165, "fn main() {\n", (Path(__file__).parent / "e165_attack_functions.rs").read_text() + "\nfn main() {\n")

# ---------- 写 ----------
for path, items in edits.items():
    text = path.read_text()
    for old, new in items:
        count = text.count(old)
        if count != 1:
            print(f"✗ {path}: 原文命中 {count} 次（要恰好 1 次）：{old[:80]!r}", file=sys.stderr)
            sys.exit(3)
        text = text.replace(old, new)
    path.write_text(text)
    print(f"patched {path} ({len(items)} 处)")
