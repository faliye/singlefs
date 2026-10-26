"""在已装候选开关的副本 walk.rs 上再加两个丁（下一次发布的记录在盘上才放宽）。"""
import sys
path = sys.argv[1]
s = open(path, encoding="utf-8").read()
def once(old, new):
    global s
    assert s.count(old) == 1, old[:80]
    s = s.replace(old, new)
once("""    JiaAnyUnreadableSlot,
}
""", """    JiaAnyUnreadableSlot,
    DingJiaAnyLapWithTheNextRecord,
    DingYiWithTheNextRecord,
}

thread_local! {
    /// 丁：journal 里有没有最新根同一实例、txg = 最新根 + 1 的自证过的记录（调用处现算、放进来）。
    pub static I718_NEXT_RECORD_PRESENT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
""")
once("""            I718Candidate::JiaAnyUnreadableSlot,
        ][usize::from(stored)]""", """            I718Candidate::JiaAnyUnreadableSlot,
            I718Candidate::DingJiaAnyLapWithTheNextRecord,
            I718Candidate::DingYiWithTheNextRecord,
        ][usize::from(stored)]""")
once("""            Ok("jia_any_unreadable") => I718Candidate::JiaAnyUnreadableSlot,""", """            Ok("jia_any_unreadable") => I718Candidate::JiaAnyUnreadableSlot,
            Ok("ding_jia") => I718Candidate::DingJiaAnyLapWithTheNextRecord,
            Ok("ding_yi") => I718Candidate::DingYiWithTheNextRecord,""")
once("""    let threshold_holds = match i718_candidate_for_the_attack_probe() {""", """    let next_record_present = I718_NEXT_RECORD_PRESENT.with(std::cell::Cell::get);
    let threshold_holds = match i718_candidate_for_the_attack_probe() {
        I718Candidate::DingYiWithTheNextRecord
            if next_slot_unreadable && next_txg > ring_length_in_txgs && next_record_present =>
        {
            judgements.not_applicable(
                ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD,
                "候选丁乙：最新根之后那一槽读不出自证过的根，而那一次发布的记录在盘上",
            );
            return judge_the_pinned_ring_slot_only(roots, newest_root, geometry, judgements);
        }
        I718Candidate::DingYiWithTheNextRecord => newest_root.reclaim_threshold == recomputed_threshold,
        I718Candidate::DingJiaAnyLapWithTheNextRecord => {
            newest_root.reclaim_threshold == recomputed_threshold
                || (next_slot_unreadable
                    && next_record_present
                    && next_txg > ring_length_in_txgs
                    && (1..=next_txg / ring_length_in_txgs).any(|laps| {
                        newest_root.reclaim_threshold
                            == recomputed_with_one_more_valid_txg(next_txg - laps * ring_length_in_txgs)
                    }))
        }""")
once("""    // I-7.18（根记录的回收门槛等于现算的门槛）、I-7.19（钉着的槽不住本实例写的根）：只判最新根、只判建账的发布（C773（读坏与全零根槽的回收门槛钉只在内存里））。
    judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root(""", """    // 攻方腿原型（丁）：journal 里有没有最新根同一实例、txg + 1 的记录。
    {
        let newest_view = &roots[newest_index].2;
        let present = scanned_journal_records_by_device(reader, &geometry, filesystem_identifier_low)
            .iter()
            .any(|(_, records)| {
                records.values().any(|record| {
                    record.instance == newest_view.instance
                        && record.checkpoint_txg == newest_view.checkpoint_txg + 1
                })
            });
        I718_NEXT_RECORD_PRESENT.with(|cell| cell.set(present));
    }
    // I-7.18（根记录的回收门槛等于现算的门槛）、I-7.19（钉着的槽不住本实例写的根）：只判最新根、只判建账的发布（C773（读坏与全零根槽的回收门槛钉只在内存里））。
    judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root(""")
open(path, "w", encoding="utf-8").write(s)
print("patched ding")
