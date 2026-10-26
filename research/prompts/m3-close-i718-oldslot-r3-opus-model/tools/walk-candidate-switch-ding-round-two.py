"""在已装第一轮开关（walk-candidate-switch.py、walk-candidate-switch-ding.py）的副本 walk.rs 上再加第二轮丁甲的几种读法（攻方腿原型，只在副本里）。

- DingRoundTwoSameInstance：记录要与最新根同一实例、txg = N+1；N+1 >= 3S；第 1、2 种现算（第 2 种按「补进环之后的现算」，与环上别的根取小再与 F 取大）；
  第 3 种按「那一槽没有根时的现算」读，与第 1 种相同，不另加。
- DingRoundTwoAnyInstance：同上，记录只看 txg = N+1，不看实例。
- DingRoundTwoAnyInstanceRawValues：记录只看 txg；第 2 种按值读（t 等于 max(F, N+1-k*3S) 就收，不与环上别的根取小），
  第 3 种按值读（t 等于 max(F, N+2-3S) 就收）。
"""
import sys
path = sys.argv[1]
s = open(path, encoding="utf-8").read()
def once(old, new):
    global s
    assert s.count(old) == 1, old[:80]
    s = s.replace(old, new)
once("""    DingYiWithTheNextRecord,
}
""", """    DingYiWithTheNextRecord,
    DingRoundTwoSameInstance,
    DingRoundTwoAnyInstance,
    DingRoundTwoAnyInstanceRawValues,
}

thread_local! {
    /// 第二轮丁甲：journal 里有没有 txg = 最新根 + 1 的自证过的记录（不看实例）。
    pub static I718_NEXT_RECORD_PRESENT_OF_ANY_INSTANCE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
""")
once("""            I718Candidate::DingYiWithTheNextRecord,
        ][usize::from(stored)]""", """            I718Candidate::DingYiWithTheNextRecord,
            I718Candidate::DingRoundTwoSameInstance,
            I718Candidate::DingRoundTwoAnyInstance,
            I718Candidate::DingRoundTwoAnyInstanceRawValues,
        ][usize::from(stored)]""")
once("""            Ok("ding_yi") => I718Candidate::DingYiWithTheNextRecord,""", """            Ok("ding_yi") => I718Candidate::DingYiWithTheNextRecord,
            Ok("ding_r2_same_instance") => I718Candidate::DingRoundTwoSameInstance,
            Ok("ding_r2_any_instance") => I718Candidate::DingRoundTwoAnyInstance,
            Ok("ding_r2_any_instance_raw") => I718Candidate::DingRoundTwoAnyInstanceRawValues,""")
once("""        I718Candidate::DingYiWithTheNextRecord => newest_root.reclaim_threshold == recomputed_threshold,""", """        I718Candidate::DingYiWithTheNextRecord => newest_root.reclaim_threshold == recomputed_threshold,
        I718Candidate::DingRoundTwoSameInstance | I718Candidate::DingRoundTwoAnyInstance => {
            let record_present = if i718_candidate_for_the_attack_probe() == I718Candidate::DingRoundTwoSameInstance {
                next_record_present
            } else {
                I718_NEXT_RECORD_PRESENT_OF_ANY_INSTANCE.with(std::cell::Cell::get)
            };
            newest_root.reclaim_threshold == recomputed_threshold
                || (next_slot_unreadable
                    && record_present
                    && next_txg >= ring_length_in_txgs
                    && (1..=next_txg / ring_length_in_txgs).any(|laps| {
                        newest_root.reclaim_threshold
                            == recomputed_with_one_more_valid_txg(next_txg - laps * ring_length_in_txgs)
                    }))
        }
        I718Candidate::DingRoundTwoAnyInstanceRawValues => {
            let floor = newest_root.rollback_floor;
            newest_root.reclaim_threshold == recomputed_threshold
                || (next_slot_unreadable
                    && I718_NEXT_RECORD_PRESENT_OF_ANY_INSTANCE.with(std::cell::Cell::get)
                    && next_txg >= ring_length_in_txgs
                    && ((1..=next_txg / ring_length_in_txgs).any(|laps| {
                        newest_root.reclaim_threshold == (next_txg - laps * ring_length_in_txgs).max(floor)
                    }) || newest_root.reclaim_threshold == (next_txg + 1 - ring_length_in_txgs).max(floor)))
        }""")
once("""        I718_NEXT_RECORD_PRESENT.with(|cell| cell.set(present));""", """        I718_NEXT_RECORD_PRESENT.with(|cell| cell.set(present));
        let present_of_any_instance = scanned_journal_records_by_device(reader, &geometry, filesystem_identifier_low)
            .iter()
            .any(|(_, records)| records.values().any(|record| record.checkpoint_txg == newest_view.checkpoint_txg + 1));
        I718_NEXT_RECORD_PRESENT_OF_ANY_INSTANCE.with(|cell| cell.set(present_of_any_instance));""")
open(path, "w", encoding="utf-8").write(s)
print("patched ding round two")
