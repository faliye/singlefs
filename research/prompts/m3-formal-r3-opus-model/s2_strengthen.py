#!/usr/bin/env python3
"""m3-formal-r3 攻方腿：在冻结副本的核上写一版「S2 句法层补强」（施加的计数器从链首起逐一连续；锚点读得出时链首 = 锚点 + 1，
读不出时链首 txg = 根 + 1、序号 1），连证明一起，写到 <输出>；只在攻方副本里，零轮。
用法：s2_strengthen.py <冻结副本的 lib.rs> <输出 lib.rs>"""
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

# 1. spec：所选根那次发布带末条标志的一条；链首条件；连续。
rep("""pub open spec fn m2_holds<Segment>(""", """pub open spec fn is_the_root_publish_last_record<Segment>(
    record: CoreRecord<Segment>,
    root_instance: u32,
    root_checkpoint_txg: u64,
) -> bool {
    record.instance == root_instance && record.checkpoint_txg == root_checkpoint_txg
        && record.ends_its_publish
}

// 链首（已定项 14 注 1）：锚点读得出时链首的计数器 = 锚点 + 1；读不出时链首 txg = 根 + 1、序号 1。
pub open spec fn is_a_chain_head<Segment>(
    head: CoreRecord<Segment>,
    root_instance: u32,
    root_checkpoint_txg: u64,
    records: Seq<CoreRecord<Segment>>,
) -> bool {
    &&& forall|anchor: int|
        0 <= anchor < records.len() && is_the_root_publish_last_record(
            #[trigger] records[anchor],
            root_instance,
            root_checkpoint_txg,
        ) ==> head.counter == records[anchor].counter + 1
    &&& (forall|anchor: int|
        0 <= anchor < records.len() ==> !is_the_root_publish_last_record(
            #[trigger] records[anchor],
            root_instance,
            root_checkpoint_txg,
        )) ==> (head.checkpoint_txg == root_checkpoint_txg + 1 && head.ordinal_within_publish == 1)
}

// S2 句法层补强：施加的计数器从链首起逐一连续（I-8.3）。
pub open spec fn applied_counters_are_continuous_from_the_chain_head<Segment>(
    root_instance: u32,
    root_checkpoint_txg: u64,
    records: Seq<CoreRecord<Segment>>,
    applied: Seq<int>,
) -> bool {
    &&& forall|position: int|
        0 <= position < applied.len() ==> (#[trigger] records[applied[position]]).counter
            == records[applied[0]].counter + position
    &&& applied.len() > 0 ==> is_a_chain_head(
        records[applied[0]],
        root_instance,
        root_checkpoint_txg,
        records,
    )
}

pub open spec fn m2_holds<Segment>(""")
rep("""    &&& outcome.report.prefix_applied == applied.len()
    &&& outcome.rebuilt == if applied.len() == 0 {""", """    &&& outcome.report.prefix_applied == applied.len()
    &&& applied_counters_are_continuous_from_the_chain_head(
        root_instance,
        root_checkpoint_txg,
        records,
        applied,
    )
    &&& outcome.rebuilt == if applied.len() == 0 {""")

# 2. 锚点函数的 ensures 与循环不变式。
rep("""    ensures
        anchor matches Ok(Some(counter)) ==> counter < u64::MAX,
{""", """    ensures
        anchor matches Ok(Some(counter)) ==> counter < u64::MAX,
        anchor matches Ok(Some(counter)) ==> (forall|a: int|
            0 <= a < records@.len() && is_the_root_publish_last_record(
                #[trigger] records@[a],
                root_instance,
                root_checkpoint_txg,
            ) ==> records@[a].counter == counter),
        anchor matches Ok(Some(counter)) ==> (exists|a: int|
            0 <= a < records@.len() && is_the_root_publish_last_record(
                #[trigger] records@[a],
                root_instance,
                root_checkpoint_txg,
            ) && records@[a].counter == counter),
        anchor matches Ok(None) ==> (forall|a: int|
            0 <= a < records@.len() ==> !is_the_root_publish_last_record(
                #[trigger] records@[a],
                root_instance,
                root_checkpoint_txg,
            )),
{""")
rep("""            found matches Some(counter) ==> counter < u64::MAX,
        decreases records.len() - index,""", """            found matches Some(counter) ==> counter < u64::MAX,
            found matches Some(counter) ==> (forall|a: int|
                0 <= a < index && is_the_root_publish_last_record(
                    #[trigger] records@[a],
                    root_instance,
                    root_checkpoint_txg,
                ) ==> records@[a].counter == counter),
            found matches Some(counter) ==> (exists|a: int|
                0 <= a < index && is_the_root_publish_last_record(
                    #[trigger] records@[a],
                    root_instance,
                    root_checkpoint_txg,
                ) && records@[a].counter == counter),
            found is None ==> (forall|a: int|
                0 <= a < index ==> !is_the_root_publish_last_record(
                    #[trigger] records@[a],
                    root_instance,
                    root_checkpoint_txg,
                )),
        decreases records.len() - index,""")

# 3. 主循环：幽灵量与不变式。
rep("""    let ghost mut applied: Seq<int> = Seq::empty();""", """    let ghost mut applied: Seq<int> = Seq::empty();
    let ghost mut first_counter: int = 0;
    let ghost mut first_index: int = 0;""")
rep("""        invariant_except_break
            taken <= index,
            report.verification_passed <= index,
            report.verification_failed == 0,
        invariant""", """        invariant_except_break
            taken <= index,
            report.verification_passed <= index,
            report.verification_failed == 0,
            applied.len() + open@.len() > 0 ==> expected_next == Some(
                (root_instance, (first_counter + applied.len() + open@.len()) as u64),
            ),
            applied.len() + open@.len() == 0 ==> (match root_own_record_counter {
                Some(counter) => expected_next == Some((root_instance, (counter + 1) as u64)),
                None => expected_next is None,
            }),
            applied.len() + open@.len() > 0 ==> first_counter + applied.len() + open@.len()
                <= u64::MAX,
        invariant
            root_own_record_counter matches Some(counter) ==> (forall|a: int|
                0 <= a < records@.len() && is_the_root_publish_last_record(
                    #[trigger] records@[a],
                    root_instance,
                    root_checkpoint_txg,
                ) ==> records@[a].counter == counter),
            root_own_record_counter matches Some(counter) ==> (exists|a: int|
                0 <= a < records@.len() && is_the_root_publish_last_record(
                    #[trigger] records@[a],
                    root_instance,
                    root_checkpoint_txg,
                ) && records@[a].counter == counter),
            root_own_record_counter is None ==> (forall|a: int|
                0 <= a < records@.len() ==> !is_the_root_publish_last_record(
                    #[trigger] records@[a],
                    root_instance,
                    root_checkpoint_txg,
                )),
            forall|position: int|
                0 <= position < applied.len() ==> (#[trigger] records@[applied[position]]).counter
                    == first_counter + position,
            forall|position: int|
                0 <= position < open@.len() ==> (#[trigger] records@[open@[position] as int]).counter
                    == first_counter + applied.len() + position,
            applied.len() + open@.len() > 0 ==> 0 <= first_index < records@.len()
                && records@[first_index].counter == first_counter && is_a_chain_head(
                records@[first_index],
                root_instance,
                root_checkpoint_txg,
                records@,
            ),
            applied.len() > 0 ==> applied[0] == first_index,
            applied.len() == 0 && open@.len() > 0 ==> open@[0] as int == first_index,""")
# 接收这一条之前记链首（攻着的、施加的都空时）。
rep("""        // [原文 3198]
        let ghost open_before = open@;
        open.push(index);""", """        // [原文 3198]
        proof {
            if applied.len() + open@.len() == 0 {
                first_counter = records@[index as int].counter as int;
                first_index = index as int;
            }
        }
        let ghost open_before = open@;
        open.push(index);
        proof {
            assert forall|position: int|
                0 <= position < open@.len() implies (#[trigger] records@[open@[position] as int]).counter
                == first_counter + applied.len() + position by {
                if position < open_before.len() {
                    assert(open@[position] == open_before[position]);
                }
            }
        }""")
rep("""            assert(is_concatenation_of_whole_publishes(after));
            assert(applied.last() == index as int);""", """            assert(is_concatenation_of_whole_publishes(after));
            assert(applied.last() == index as int);
            assert forall|position: int|
                0 <= position < applied.len() implies (#[trigger] records@[applied[position]]).counter
                == first_counter + position by {
                if position < applied_before.len() {
                    assert(applied[position] == applied_before[position]);
                } else {
                    assert(applied[position] == this_publish[position - applied_before.len()]);
                    assert(this_publish[position - applied_before.len()] == open@[position - applied_before.len()] as int);
                }
            }
            if applied_before.len() == 0 {
                assert(applied[0] == this_publish[0]);
            } else {
                assert(applied[0] == applied_before[0]);
            }""")

# 4. 锚点映射成 expected_next：闭包带 ensures，Verus 才知道 map 之后是什么。
rep("""        |counter: u64| -> (u32, u64)
            requires
                counter < u64::MAX,
            { (root_instance, counter + 1) },
    );""", """        |counter: u64| -> (next: (u32, u64))
            requires
                counter < u64::MAX,
            ensures
                next == (root_instance, (counter + 1) as u64),
            { (root_instance, counter + 1) },
    );""")
Path(sys.argv[2]).parent.mkdir(parents=True, exist_ok=True)
Path(sys.argv[2]).write_text(text)
print("written", sys.argv[2])
