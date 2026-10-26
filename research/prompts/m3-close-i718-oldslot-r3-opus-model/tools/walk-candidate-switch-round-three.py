"""在已装第一、二轮开关的副本 walk.rs 上加第三轮两个候选（攻方腿原型，只在副本里）。

- DingYiRoundThree（丁乙，第三轮正文第三节）：最新根之后那一槽读不出自证过的根、journal 里有任一实例 txg = N+1 的记录、N+1 >= 3S 时
  I-7.18 报不适用；I-7.19 照原样判（第一轮 DingYiWithTheNextRecord 的写法，只把前提换成任一实例、>=）。
  正文「照 I-7.18 条文里「不判」那一类的写法记」连 I-7.19 一起不判的那一读法不另装：探针按「I-7.18 报不适用」那几张把 I-7.19 改记成不适用。
- DingJiaGenerationZero（攻方第三轮提的，零轮）：丁甲（任一实例、>=、第 2 种按补进环之后现算），下一槽是某个区域的槽 0 时，
  补进去的候选另加 mkfs 的第 0 代根（txg 0）。
"""
import sys
path = sys.argv[1]
s = open(path, encoding="utf-8").read()
def once(old, new):
    global s
    assert s.count(old) == 1, old[:80]
    s = s.replace(old, new)
once("""    DingRoundTwoAnyInstanceRawValues,
}
""", """    DingRoundTwoAnyInstanceRawValues,
    DingYiRoundThree,
    DingJiaGenerationZero,
}
""")
once("""            I718Candidate::DingRoundTwoAnyInstanceRawValues,
        ][usize::from(stored)]""", """            I718Candidate::DingRoundTwoAnyInstanceRawValues,
            I718Candidate::DingYiRoundThree,
            I718Candidate::DingJiaGenerationZero,
        ][usize::from(stored)]""")
once("""            Ok("ding_r2_any_instance_raw") => I718Candidate::DingRoundTwoAnyInstanceRawValues,""", """            Ok("ding_r2_any_instance_raw") => I718Candidate::DingRoundTwoAnyInstanceRawValues,
            Ok("ding_yi_r3") => I718Candidate::DingYiRoundThree,
            Ok("ding_jia_gen0") => I718Candidate::DingJiaGenerationZero,""")
once("""        I718Candidate::DingRoundTwoAnyInstanceRawValues => {""", """        I718Candidate::DingYiRoundThree
            if next_slot_unreadable
                && I718_NEXT_RECORD_PRESENT_OF_ANY_INSTANCE.with(std::cell::Cell::get)
                && next_txg >= ring_length_in_txgs =>
        {
            judgements.not_applicable(
                ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD,
                "候选丁乙（第三轮）：最新根之后那一槽读不出自证过的根，journal 里有任一实例点名 N+1 的记录，N+1 >= 3S",
            );
            return judge_the_pinned_ring_slot_only(roots, newest_root, geometry, judgements);
        }
        I718Candidate::DingYiRoundThree => newest_root.reclaim_threshold == recomputed_threshold,
        I718Candidate::DingJiaGenerationZero => {
            let generation_zero_may_have_lived_there = slot_of_txg(next_txg).1 == 0;
            newest_root.reclaim_threshold == recomputed_threshold
                || (next_slot_unreadable
                    && I718_NEXT_RECORD_PRESENT_OF_ANY_INSTANCE.with(std::cell::Cell::get)
                    && next_txg >= ring_length_in_txgs
                    && ((1..=next_txg / ring_length_in_txgs).any(|laps| {
                        newest_root.reclaim_threshold
                            == recomputed_with_one_more_valid_txg(next_txg - laps * ring_length_in_txgs)
                    }) || (generation_zero_may_have_lived_there
                        && newest_root.reclaim_threshold == recomputed_with_one_more_valid_txg(0))))
        }
        I718Candidate::DingRoundTwoAnyInstanceRawValues => {""")
open(path, "w", encoding="utf-8").write(s)
print("patched round three")
