#!/usr/bin/env python3
"""S2 句法层补强的原型 1：m2_holds 加「施加的计数器从链首起逐一连续」，把锚点写成 spec，循环不变量带链条。"""
import pathlib, sys
src = pathlib.Path("/tmp/claude-1000/m3-formal-r3-sonnet/verus/base-crate/src/lib.rs").read_text()
out_dir = pathlib.Path(sys.argv[1]); (out_dir / "src").mkdir(parents=True, exist_ok=True)

def sub(old, new):
    global src
    assert src.count(old) == 1, (src.count(old), old[:70])
    src = src.replace(old, new)

# 1. spec：锚点与链
sub("""pub open spec fn m2_holds<Segment>(""",
"""// 所选根那次发布读得出的记录里带末条标志的那一条（spec 层照 counter_of_the_last_record_the_root_covers 的字面）
pub open spec fn root_publish_last_flagged<Segment>(
    records: Seq<CoreRecord<Segment>>,
    root_instance: u32,
    root_checkpoint_txg: u64,
    position: int,
) -> bool {
    &&& 0 <= position < records.len()
    &&& records[position].instance == root_instance
    &&& records[position].checkpoint_txg == root_checkpoint_txg
    &&& records[position].ends_its_publish
}

// 锚点 = 那一条的计数器；一条都没有 = None；多于一条 = 没有锚点（核报错，走不到 Ok）
pub open spec fn anchor_is<Segment>(
    root_instance: u32,
    root_checkpoint_txg: u64,
    records: Seq<CoreRecord<Segment>>,
    anchor: Option<u64>,
) -> bool {
    match anchor {
        Some(counter) => exists|position: int|
            (#[trigger] root_publish_last_flagged(records, root_instance, root_checkpoint_txg, position))
                && records[position].counter == counter && (forall|other: int|
                (#[trigger] root_publish_last_flagged(records, root_instance, root_checkpoint_txg, other))
                    ==> other == position),
        None => forall|position: int|
            !(#[trigger] root_publish_last_flagged(records, root_instance, root_checkpoint_txg, position)),
    }
}

// 施加的计数器从链首起逐一连续：锚点读得出时首条 = 锚点 + 1；读不出时首条 txg = 根 + 1、序号 1；之后每条 = 前一条 + 1
pub open spec fn chain_from_the_head<Segment>(
    root_checkpoint_txg: u64,
    records: Seq<CoreRecord<Segment>>,
    anchor: Option<u64>,
    chain: Seq<int>,
) -> bool {
    &&& chain.len() > 0 ==> (match anchor {
        Some(counter) => records[chain[0]].counter == counter + 1,
        None => records[chain[0]].checkpoint_txg == root_checkpoint_txg + 1
            && records[chain[0]].ordinal_within_publish == 1,
    })
    &&& forall|position: int|
        0 <= position && position + 1 < chain.len() ==> records[(#[trigger] chain[position + 1])].counter
            == records[chain[position]].counter + 1
}

pub open spec fn m2_holds<Segment>(""")

# 2. m2_holds 末加一条
sub("""    &&& outcome.rebuilt == if applied.len() == 0 {
        root_segment
    } else {
        records[applied.last()].new_root_segment
    }
}""",
"""    &&& outcome.rebuilt == if applied.len() == 0 {
        root_segment
    } else {
        records[applied.last()].new_root_segment
    }
    &&& exists|anchor: Option<u64>|
        (#[trigger] anchor_is(root_instance, root_checkpoint_txg, records, anchor))
            && chain_from_the_head(root_checkpoint_txg, records, anchor, applied)
}""")

# 3. 锚点函数：ensures 与循环不变量
sub("""    ensures
        anchor matches Ok(Some(counter)) ==> counter < u64::MAX,
{
    let mut found: Option<u64> = None;""",
"""    ensures
        anchor matches Ok(Some(counter)) ==> counter < u64::MAX,
        anchor matches Ok(found_anchor) ==> anchor_is(
            root_instance,
            root_checkpoint_txg,
            records@,
            found_anchor,
        ),
{
    let mut found: Option<u64> = None;""")
sub("""            found matches Some(counter) ==> counter < u64::MAX,
        decreases records.len() - index,
    {
        let record = &records[index];
        // [原文 2970–2972]""",
"""            found matches Some(counter) ==> counter < u64::MAX,
            found is None ==> forall|position: int|
                0 <= position < index ==> !(#[trigger] root_publish_last_flagged(
                    records@,
                    root_instance,
                    root_checkpoint_txg,
                    position,
                )),
            found matches Some(counter) ==> exists|position: int|
                0 <= position < index
                    && (#[trigger] root_publish_last_flagged(
                    records@,
                    root_instance,
                    root_checkpoint_txg,
                    position,
                )) && records@[position].counter == counter && (forall|other: int|
                    0 <= other < index && (#[trigger] root_publish_last_flagged(
                        records@,
                        root_instance,
                        root_checkpoint_txg,
                        other,
                    )) ==> other == position),
        decreases records.len() - index,
    {
        let record = &records[index];
        // [原文 2970–2972]""")

# 4. 初值：map 换成显式 match
sub("""    let mut expected_next: Option<(u32, u64)> = root_own_record_counter.map(
        |counter: u64| -> (u32, u64)
            requires
                counter < u64::MAX,
            { (root_instance, counter + 1) },
    );""",
"""    let mut expected_next: Option<(u32, u64)> = match root_own_record_counter {
        Some(counter) => Some((root_instance, counter + 1)),
        None => None,
    };""")

# 5. 主循环不变量：链条（任何 break 处都成立）与期望键（只在回到循环头时成立）
sub("""            // 还没接上链首（锚点读不出、一条都没收）时攒着的一定是空的：tsv-448 那一判冗余靠这一条
            expected_next is None ==> open@.len() == 0,
        decreases records.len() - index,""",
"""            // 还没接上链首（锚点读不出、一条都没收）时攒着的一定是空的：tsv-448 那一判冗余靠这一条
            expected_next is None ==> open@.len() == 0,
            // S2 补强：已施加加攒着的合起来就是从链首起逐一连续的一串
            anchor_is(root_instance, root_checkpoint_txg, records@, root_own_record_counter),
            chain_from_the_head(
                root_checkpoint_txg,
                records@,
                root_own_record_counter,
                applied + as_ints(open@),
            ),
        decreases records.len() - index,""")
sub("""        invariant_except_break
            taken <= index,""",
"""        invariant_except_break
            match expected_next {
                Some(key) => key.0 == root_instance && ((applied + as_ints(open@)).len() == 0 ==> (
                root_own_record_counter matches Some(first_counter) && key.1 == first_counter + 1))
                    && ((applied + as_ints(open@)).len() > 0 ==> key.1 == records@[(applied
                    + as_ints(open@)).last()].counter + 1),
                None => root_own_record_counter is None && (applied + as_ints(open@)).len() == 0,
            },
            taken <= index,""")

# 6. 取下标进 open 之后的证明：新链条
sub("""        // [原文 3198]
        let ghost open_before = open@;
        open.push(index);
        proof {""",
"""        // [原文 3198]
        let ghost open_before = open@;
        let ghost consumed_before = applied + as_ints(open_before);
        proof {
            // 走到这里 expected_next 那一判没断：本条的计数器就是期望的（Some），或本条是无锚点时的链首（None）
            match expected_next {
                Some(expected_key) => {
                    assert(record.instance == expected_key.0 && record.counter == expected_key.1);
                },
                None => {
                    assert(record.checkpoint_txg == chain_start_txg_without_anchor && record.ordinal_within_publish == 1);
                },
            }
        }
        open.push(index);
        proof {
            assert(as_ints(open@) =~= as_ints(open_before).push(index as int));
            assert(applied + as_ints(open@) =~= consumed_before.push(index as int));
            let chain_after = consumed_before.push(index as int);
            assert(chain_from_the_head(
                root_checkpoint_txg,
                records@,
                root_own_record_counter,
                chain_after,
            )) by {
                assert forall|position: int|
                    0 <= position && position + 1 < chain_after.len() implies records@[(
                #[trigger] chain_after[position + 1])].counter == records@[chain_after[position]].counter + 1 by {
                    if position + 1 < consumed_before.len() {
                        assert(chain_after[position + 1] == consumed_before[position + 1]);
                        assert(chain_after[position] == consumed_before[position]);
                    }
                }
            }
        }
        proof {""")
(out_dir / "src/lib.rs").write_text(src)
print("wrote", out_dir)
