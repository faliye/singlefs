"""攻方腿（第二份补丁，接在 patch_e165.py 之后打）：对抗一族里另数「同一实例里 checkpoint_txg 随计数器不减」（岔路单 K10）
成立的那些态上的 AS 三条与 A-互 三个数。原有判据一条不改。"""
import sys
path = sys.argv[1]
src = open(path, encoding="utf-8").read()
def once(old, new):
    global src
    n = src.count(old)
    if n != 1:
        sys.exit(f"锚点命中 {n} 次：{old[:80]!r}")
    src = src.replace(old, new)
once('        AttackLegalVm2SatisfiedButJ2 => "attack_legal_vm2_satisfied_but_j2",\n',
'''        AttackLegalVm2SatisfiedButJ2 => "attack_legal_vm2_satisfied_but_j2",
        AttackK10States => "attack_k10_states",
        AttackK10AsWholeTable => "attack_k10_as_whole_table",
        AttackK10AsGapContinues => "attack_k10_as_gap_continues",
        AttackK10AsGapStops => "attack_k10_as_gap_stops",
        AttackK10MutualWholeVsContinues => "attack_k10_whole_vs_continues",
        AttackK10MutualWholeVsStops => "attack_k10_whole_vs_stops",
        AttackK10MutualContinuesVsStops => "attack_k10_continues_vs_stops",
''')
once('''        // A-互：三种读法两两之间期望不同的态数。
        if expectations[0] != expectations[1] {''',
'''        // 攻方腿：K10 成立（同实例里 txg 随计数器不减）的态上另数一遍。
        let mut k10_previous_txg: Option<u64> = None;
        let mut k10_holds = true;
        for record in records.values() {
            if record.instance != self.chosen_root.instance {
                continue;
            }
            if k10_previous_txg.is_some_and(|previous| record.checkpoint_txg.0 < previous) {
                k10_holds = false;
                break;
            }
            k10_previous_txg = Some(record.checkpoint_txg.0);
        }
        if k10_holds {
            tally.add(Counter::AttackK10States);
            for (expectation, counter) in expectations.into_iter().zip([
                Counter::AttackK10AsWholeTable,
                Counter::AttackK10AsGapContinues,
                Counter::AttackK10AsGapStops,
            ]) {
                if !outcome_matches_the_expectation(&first, expectation) {
                    tally.add_violation(counter, &describe_witness);
                }
            }
            if expectations[0] != expectations[1] {
                tally.add_violation(Counter::AttackK10MutualWholeVsContinues, &describe_witness);
            }
            if expectations[0] != expectations[2] {
                tally.add_violation(Counter::AttackK10MutualWholeVsStops, &describe_witness);
            }
            if expectations[1] != expectations[2] {
                tally.add_violation(Counter::AttackK10MutualContinuesVsStops, &describe_witness);
            }
        }
        // A-互：三种读法两两之间期望不同的态数。
        if expectations[0] != expectations[1] {''')
once('''    if domain.family() == Family::Adversarial {
        emitter.result(&format!(
            "name=specification {place} expects_failure={}",''',
'''    if domain.family() == Family::Adversarial {
        emitter.result(&format!(
            "name=attack_k10 {place} states={} k10_states={} as_whole_table={} as_gap_continues={} as_gap_stops={} whole_vs_continues={} whole_vs_stops={} continues_vs_stops={} witness_as_gap_continues={} witness_as_whole_table={} witness_as_gap_stops={}",
            tally.get(Counter::States),
            tally.get(Counter::AttackK10States),
            tally.get(Counter::AttackK10AsWholeTable),
            tally.get(Counter::AttackK10AsGapContinues),
            tally.get(Counter::AttackK10AsGapStops),
            tally.get(Counter::AttackK10MutualWholeVsContinues),
            tally.get(Counter::AttackK10MutualWholeVsStops),
            tally.get(Counter::AttackK10MutualContinuesVsStops),
            witness_field(tally, Counter::AttackK10AsGapContinues),
            witness_field(tally, Counter::AttackK10AsWholeTable),
            witness_field(tally, Counter::AttackK10AsGapStops),
        ));
        emitter.result(&format!(
            "name=specification {place} expects_failure={}",''')
open(path, "w", encoding="utf-8").write(src)
print("patched", path)
