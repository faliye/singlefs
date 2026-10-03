"""攻方腿给仓副本里的 E165 装置加的几个量（只加计数与一个生成器开关，原有判据一条不改）。
- attack_vm2_*：Verus 核 spec 段 m2_holds 的可执行替身——返回的根逐字段等于第 j 版、prefix_applied = p 时，
  存不存在一组下标满足 m2_holds（每一段是表里读得出的一次整发布、都在水位之上、末一段是第 j 版、条数合计 p）。
- attack_full_root_mismatch：返回的根与第 j 版的整条根（十个字段）不等。
- ATTACK_ROW_ROOT=1：第 0 版（所选根 R_0）照写行那次发布带一条非零的分配记录树根，之后各版照带文件那一版写全零
  （transaction.rs 的写行发布与带文件发布两处各自怎么填这一项）。
- attack_legal_*：C 域里只数合法崩溃状态（甲、乙两种切法任一种下停在某一段）的 J6 与在飞上限。"""
import sys
path = sys.argv[1]
src = open(path, encoding="utf-8").read()

def once(old, new):
    global src
    n = src.count(old)
    if n != 1:
        sys.exit(f"锚点命中 {n} 次：{old[:80]!r}")
    src = src.replace(old, new)

once('        StatesHoldingARecordRejectedByParse => "states_holding_a_record_rejected_by_parse",\n',
'''        StatesHoldingARecordRejectedByParse => "states_holding_a_record_rejected_by_parse",
        AttackVm2Judged => "attack_vm2_judged",
        AttackVm2Unsatisfiable => "attack_vm2_unsatisfiable",
        AttackVm2SatisfiedButJ2 => "attack_vm2_satisfied_but_j2",
        AttackVm2SatisfiedButJ4 => "attack_vm2_satisfied_but_j4",
        AttackFullRootMismatch => "attack_full_root_mismatch",
        AttackLegalStates => "attack_legal_states",
        AttackLegalJ6PremiseFailsJudged => "attack_legal_j6_premise_fails_judged",
        AttackLegalJ6PremiseFailsViolations => "attack_legal_j6_premise_fails",
        AttackLegalJ6PremiseHoldsViolations => "attack_legal_j6_premise_holds",
        AttackLegalAboveWaterOverLimit => "attack_legal_above_water_over_limit",
        AttackLegalJ1 => "attack_legal_j1",
        AttackLegalJ2 => "attack_legal_j2",
        AttackLegalJ3 => "attack_legal_j3",
        AttackLegalJ4 => "attack_legal_j4",
        AttackLegalVm2SatisfiedButJ2 => "attack_legal_vm2_satisfied_but_j2",
''')

once("fn judge_a_legal_state(\n",
'''fn attack_row_root() -> bool {
    static ROW_ROOT: OnceLock<bool> = OnceLock::new();
    *ROW_ROOT.get_or_init(|| env::var("ATTACK_ROW_ROOT").is_ok_and(|text| text == "1"))
}

/// 第 `version` 版的整条根；ATTACK_ROW_ROOT=1 时第 0 版带写行发布那条非零的分配记录树根。
fn attack_full_root_of_version(history: &History, version: usize) -> RootRecord {
    let mut root = history.version_fields(version).to_root();
    if attack_row_root() && version == 0 {
        root.allocation_record_tree_root = node_pointer_at_slot(68_000, root.checkpoint_txg.0);
    }
    root
}

/// Verus 核 m2_holds 的可执行替身（见文件头）。
fn attack_verus_m2_satisfiable(state: &LegalState<'_>, landing: usize, prefix_applied: usize) -> bool {
    let chosen = state.chosen_version;
    if landing < chosen {
        return false;
    }
    if landing == chosen {
        return prefix_applied == 0;
    }
    let wholly_in_the_table = |version: usize| {
        state
            .history
            .record_indexes_of_publish(version)
            .all(|record_index| state.presence[record_index] != RecordPresence::NotOnDisk)
    };
    if !wholly_in_the_table(landing) {
        return false;
    }
    let last = state.history.publishes[landing].record_count;
    let Some(target) = prefix_applied.checked_sub(last) else {
        return false;
    };
    let mut reachable = vec![false; target + 1];
    reachable[0] = true;
    for version in chosen + 1..landing {
        if !wholly_in_the_table(version) {
            continue;
        }
        let count = state.history.publishes[version].record_count;
        for sum in (count..=target).rev() {
            if reachable[sum - count] {
                reachable[sum] = true;
            }
        }
    }
    reachable[target]
}

fn judge_a_legal_state(
''')

once("    let chosen_root = history.version_fields(chosen_version).to_root();\n    let first = first_outcome_on_a_legal_state(",
"    let chosen_root = attack_full_root_of_version(history, chosen_version);\n    let first = first_outcome_on_a_legal_state(")

once("    // J4 最远一版：j = j*。\n    if landing != Some(truth.farthest_version) {\n        tally.add_violation(Counter::J4Violations, &describe_witness);\n    }\n",
'''    // J4 最远一版：j = j*。
    if landing != Some(truth.farthest_version) {
        tally.add_violation(Counter::J4Violations, &describe_witness);
    }
    // 攻方腿加的量：Verus m2_holds 的替身、整条根。
    if let (ReplayOutcome::Returned(returned), Some(version)) = (&first, landing) {
        if state.residue.is_none() {
            tally.add(Counter::AttackVm2Judged);
            if attack_verus_m2_satisfiable(state, version, returned.prefix_applied) {
                if !root_plus_disk_holds {
                    tally.add_violation(Counter::AttackVm2SatisfiedButJ2, &describe_witness);
                }
                if landing != Some(truth.farthest_version) {
                    tally.add_violation(Counter::AttackVm2SatisfiedButJ4, &describe_witness);
                }
            } else {
                tally.add_violation(Counter::AttackVm2Unsatisfiable, &describe_witness);
            }
        }
        if returned.root != attack_full_root_of_version(history, version) {
            tally.add_violation(Counter::AttackFullRootMismatch, &describe_witness);
        }
    }
''')

# C 域：先建段，再逐状态判；合法崩溃状态另数 J6 与在飞上限
old_c_loop_head = "    // 状态掩码：低 Nc 位是 P_0 之后各条记录在不在盘上，高 m 位是各次发布的根槽写落没落。\n    for state_mask in every_subset_mask(state_bit_count) {\n"
once(old_c_loop_head,
'''    let mut attack_segments_own: Vec<u32> = Vec::new();
    let mut attack_segments_with: Vec<u32> = Vec::new();
    for version in 1..=root_write_count {
        let record_bits = history
            .record_indexes_of_publish(version)
            .fold(0u32, |bits, record_index| bits | (1 << (record_index - 1)));
        let root_bit = 1u32 << (later_record_count + version - 1);
        attack_segments_own.push(record_bits);
        attack_segments_own.push(root_bit);
        attack_segments_with.push(record_bits | root_bit);
    }
''' + old_c_loop_head)

once('''        let today_interval = judge_a_legal_state(
            &state,
            implementation,
            SpecificationSelfCheck::NotJudged,
            unit_index == 0 && state_mask == 0 && later_record_count == 1,
            &mut tally,
        );
''',
'''        let mut state_tally = Tally::new();
        let today_interval = judge_a_legal_state(
            &state,
            implementation,
            SpecificationSelfCheck::NotJudged,
            unit_index == 0 && state_mask == 0 && later_record_count == 1,
            &mut state_tally,
        );
        let is_a_legal_crash_state = stop_segment_mask(state_mask, &attack_segments_own) != 0
            || stop_segment_mask(state_mask, &attack_segments_with) != 0;
        if is_a_legal_crash_state {
            state_tally.add(Counter::AttackLegalStates);
            for (from, to) in [
                (Counter::J6PremiseFailsJudged, Counter::AttackLegalJ6PremiseFailsJudged),
                (Counter::J6PremiseFailsViolations, Counter::AttackLegalJ6PremiseFailsViolations),
                (Counter::J6PremiseHoldsViolations, Counter::AttackLegalJ6PremiseHoldsViolations),
                (Counter::AboveWaterOverLimit, Counter::AttackLegalAboveWaterOverLimit),
                (Counter::J1Violations, Counter::AttackLegalJ1),
                (Counter::J2Violations, Counter::AttackLegalJ2),
                (Counter::J3Violations, Counter::AttackLegalJ3),
                (Counter::J4Violations, Counter::AttackLegalJ4),
                (Counter::AttackVm2SatisfiedButJ2, Counter::AttackLegalVm2SatisfiedButJ2),
            ] {
                let times = state_tally.get(from);
                for _ in 0..times {
                    state_tally.add(to);
                }
            }
        }
        tally.merge(&state_tally);
''')

once('''    if domain.specification_self_check() == SpecificationSelfCheck::Judged {
        for counter in [
            Counter::SpecificationWholeTableMismatch,''',
'''    emitter.result(&format!(
        "name=attack {place} states={} J1={} J2={} J3={} J4={} vm2_judged={} vm2_unsatisfiable={} vm2_satisfied_but_j2={} vm2_satisfied_but_j4={} full_root_mismatch={} legal_states={} legal_j6_premise_fails_judged={} legal_j6_premise_fails={} legal_j6_premise_holds={} legal_above_water_over_limit={} legal_j1={} legal_j2={} legal_j3={} legal_j4={} legal_vm2_satisfied_but_j2={} witness_vm2_satisfied_but_j2={} witness_vm2_unsatisfiable={} witness_full_root_mismatch={}",
        tally.get(Counter::States),
        tally.get(Counter::J1Violations),
        tally.get(Counter::J2Violations),
        tally.get(Counter::J3Violations),
        tally.get(Counter::J4Violations),
        tally.get(Counter::AttackVm2Judged),
        tally.get(Counter::AttackVm2Unsatisfiable),
        tally.get(Counter::AttackVm2SatisfiedButJ2),
        tally.get(Counter::AttackVm2SatisfiedButJ4),
        tally.get(Counter::AttackFullRootMismatch),
        tally.get(Counter::AttackLegalStates),
        tally.get(Counter::AttackLegalJ6PremiseFailsJudged),
        tally.get(Counter::AttackLegalJ6PremiseFailsViolations),
        tally.get(Counter::AttackLegalJ6PremiseHoldsViolations),
        tally.get(Counter::AttackLegalAboveWaterOverLimit),
        tally.get(Counter::AttackLegalJ1),
        tally.get(Counter::AttackLegalJ2),
        tally.get(Counter::AttackLegalJ3),
        tally.get(Counter::AttackLegalJ4),
        tally.get(Counter::AttackLegalVm2SatisfiedButJ2),
        witness_field(tally, Counter::AttackVm2SatisfiedButJ2),
        witness_field(tally, Counter::AttackVm2Unsatisfiable),
        witness_field(tally, Counter::AttackFullRootMismatch),
    ));
    if domain.specification_self_check() == SpecificationSelfCheck::Judged {
        for counter in [
            Counter::SpecificationWholeTableMismatch,''')
open(path, "w", encoding="utf-8").write(src)
print("patched", path)
