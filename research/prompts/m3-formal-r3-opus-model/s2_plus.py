#!/usr/bin/env python3
"""m3-formal-r3 攻方腿：在 s2_strengthen.py 的输出上再加两句（攻方自己提的收严，零轮）：
施加的条数不超过在飞上限；一次整的发布末条带提交标记、末条之前不带提交标记的那条之后不换事务号（已定项 7、已定项 14 第三条）。
用法：s2_plus.py <s2_strengthen 的输出> <输出 lib.rs>"""
import sys
from pathlib import Path

text = Path(sys.argv[1]).read_text()

def rep(old, new):
    global text
    count = text.count(old)
    if count != 1:
        print(f"✗ 原文命中 {count} 次：{old[:90]!r}", file=sys.stderr)
        sys.exit(3)
    text = text.replace(old, new)

# is_whole_publish：末条带提交标记；不带提交标记的那条之后同一事务。
rep("""    &&& forall|position: int|
        0 <= position < publish.len() ==> ((#[trigger] publish[position]).ends_its_publish
            <==> position == publish.len() - 1)
}""", """    &&& forall|position: int|
        0 <= position < publish.len() ==> ((#[trigger] publish[position]).ends_its_publish
            <==> position == publish.len() - 1)
    &&& publish[publish.len() - 1].is_commit
    &&& forall|position: int|
        0 <= position < publish.len() - 1 ==> (#[trigger] publish[position]).is_commit
            || publish[position + 1].transaction == publish[position].transaction
}""")
# is_open_publish：同一句。
rep("""    &&& forall|position: int|
        0 <= position < publish.len() ==> !(#[trigger] publish[position]).ends_its_publish
}""", """    &&& forall|position: int|
        0 <= position < publish.len() ==> !(#[trigger] publish[position]).ends_its_publish
    &&& forall|position: int|
        0 <= position < publish.len() - 1 ==> (#[trigger] publish[position]).is_commit
            || publish[position + 1].transaction == publish[position].transaction
}""")
# replay_core 的 ensures：施加条数不超过在飞上限。
rep("""            Ok(outcome) => m2_holds(
                root_instance,
                root_checkpoint_txg,
                root_segment,
                records@,
                outcome,
            ),""", """            Ok(outcome) => m2_holds(
                root_instance,
                root_checkpoint_txg,
                root_segment,
                records@,
                outcome,
            ) && outcome.applied@.len() <= in_flight_limit,""")
rep("""        invariant
            root_own_record_counter matches Some(counter) ==> (forall|a: int|""", """        invariant
            taken <= in_flight_limit,
            applied.len() + open@.len() <= taken,
            root_own_record_counter matches Some(counter) ==> (forall|a: int|""")
# 冒烟：两条的整次发布要带提交标记。
rep("""        !first.ends_its_publish,
        second.ends_its_publish,
    ensures
        is_concatenation_of_whole_publishes(seq![first, second]),""", """        !first.ends_its_publish,
        second.ends_its_publish,
        second.is_commit,
        first.is_commit || second.transaction == first.transaction,
    ensures
        is_concatenation_of_whole_publishes(seq![first, second]),""")
# 接收这一条时的提交标记那一句。
rep("""        let ghost open_before = open@;
        open.push(index);
        proof {""", """        let ghost open_before = open@;
        open.push(index);
        proof {
            let after_commit = records_at(records@, as_ints(open@));
            assert forall|position: int|
                0 <= position < after_commit.len() - 1 implies (#[trigger] after_commit[position]).is_commit
                || after_commit[position + 1].transaction == after_commit[position].transaction by {
                if position + 1 < open_before.len() {
                    assert(open@[position] == open_before[position]);
                    assert(open@[position + 1] == open_before[position + 1]);
                    let before_commit = records_at(records@, as_ints(open_before));
                    assert(before_commit[position] == after_commit[position]);
                    assert(before_commit[position + 1] == after_commit[position + 1]);
                } else {
                    assert(open@[position] == open_before[open_before.len() - 1]);
                    assert(open@[position + 1] == index);
                }
            }""")
Path(sys.argv[2]).parent.mkdir(parents=True, exist_ok=True)
Path(sys.argv[2]).write_text(text)
print("written", sys.argv[2])
