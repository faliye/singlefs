"""辩方腿原型：在仓副本的 walk.rs 上，I-7.18 那一判照今天的判（真判不动），同时把各候选的判定一次算出来、放进线程局部变量，探针读。
候选（下标顺序，探针与 tally 按这个顺序读）：
  0 today        今天的现算（丙就是它）
  1 jia_gt       投稿的甲：下一槽读不出、N+1 > 3S 时再认 N+1-3S 那一种现算（单值）
  2 jia_seed     甲含种子边：同上，N+1 >= 3S
  3 yi_literal   乙照正文字面：下一槽读不出就报不适用
  4 yi_gate      乙加闸：下一槽读不出且 N+1 >= 3S 才报不适用
  5 any_lap      甲-任一圈（不设 journal 前提）：下一槽读不出、N+1 >= 3S 时，旧内容取 N+1-k*3S（k >= 1）任一圈
  6 ding_jia     丁甲：any_lap 再加前提「journal 里有点名 N+1 的记录」（任一实例）
  7 ding_jia_same 同上，前提收成「记录的实例与最新根同一实例」
  8 ding_jia_single 前提同 6，旧内容只认 N+1-3S 一种（单值）
  9 ding_yi      丁乙：下一槽读不出、N+1 >= 3S、有前提记录（任一实例）时报不适用
 10 ding_yi_same 同上，前提收成同一实例
取值：1 成立，0 判红，-1 不适用。
"""
import sys
path = sys.argv[1]
s = open(path, encoding="utf-8").read()


def once(old, new):
    global s
    assert s.count(old) == 1, old[:100]
    s = s.replace(old, new)


once("""    let (newest_instance, newest_txg) = (newest_root.instance, newest_root.checkpoint_txg);
    judgements.judge(
        ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD,
        newest_root.reclaim_threshold == recomputed_threshold,
""", """    let (newest_instance, newest_txg) = (newest_root.instance, newest_root.checkpoint_txg);
    {
        let ring_length_in_txgs = geometry.regions * geometry.slots_per_region;
        let slot_of_txg = |txg: u64| (txg % geometry.regions, (txg / geometry.regions) % geometry.slots_per_region);
        let slot_holds_a_self_verified_root =
            |slot: (u64, u64)| roots.iter().any(|(region, ring_slot, _)| (*region, *ring_slot) == slot);
        let recomputed_with_one_more_valid_txg = |extra_txg: u64| {
            deferred_slots_reclaim_threshold(
                roots
                    .iter()
                    .map(|(_, _, root)| (standing_in_the_newest_roots_table(root), root.checkpoint_txg))
                    .chain(
                        pinned_txg_counted_as_a_valid_root
                            .map(|pinned_txg| (RootStandingInTheNewestRootsTable::Valid, pinned_txg)),
                    )
                    .chain(std::iter::once((RootStandingInTheNewestRootsTable::Valid, extra_txg))),
                newest_root.rollback_floor,
            )
        };
        let next_txg = newest_txg + 1;
        let unreadable = !slot_holds_a_self_verified_root(slot_of_txg(next_txg));
        let t = newest_root.reclaim_threshold;
        let today = t == recomputed_threshold;
        let gate_seed = next_txg >= ring_length_in_txgs;
        let gate_gt = next_txg > ring_length_in_txgs;
        let single = gate_seed && t == recomputed_with_one_more_valid_txg(next_txg - ring_length_in_txgs);
        let any_lap = gate_seed
            && (1..=next_txg / ring_length_in_txgs)
                .any(|laps| t == recomputed_with_one_more_valid_txg(next_txg - laps * ring_length_in_txgs));
        let (premise_any, premise_same) = I718_PREMISE.with(std::cell::Cell::get);
        let v = |b: bool| if b { 1i8 } else { 0i8 };
        let all: [i8; 11] = [
            v(today),
            v(today || (unreadable && gate_gt && single)),
            v(today || (unreadable && gate_seed && single)),
            if unreadable { -1 } else { v(today) },
            if unreadable && gate_seed { -1 } else { v(today) },
            v(today || (unreadable && any_lap)),
            v(today || (unreadable && premise_any && any_lap)),
            v(today || (unreadable && premise_same && any_lap)),
            v(today || (unreadable && premise_any && single)),
            if unreadable && gate_seed && premise_any { -1 } else { v(today) },
            if unreadable && gate_seed && premise_same { -1 } else { v(today) },
        ];
        I718_ALL.with(|cell| *cell.borrow_mut() = Some(all));
    }
    judgements.judge(
        ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD,
        newest_root.reclaim_threshold == recomputed_threshold,
""")

anchor = "/// I-7.18（根记录的回收门槛等于现算的门槛） 与 I-7.19（钉着的槽不住本实例写的根）：只判最新根，只判建账的发布\n"
once(anchor, """thread_local! {
    /// 辩方腿原型：各候选在最近一次 I-7.18 判里的判定（1 成立、0 判红、-1 不适用）；没走到那一判（不适用在更前面）时是 None。
    pub static I718_ALL: std::cell::RefCell<Option<[i8; 11]>> = const { std::cell::RefCell::new(None) };
    /// 辩方腿原型：journal 里有没有点名「最新根的 txg + 1」的记录（任一实例，最新根同一实例）。调用处现算、放进来。
    pub static I718_PREMISE: std::cell::Cell<(bool, bool)> = const { std::cell::Cell::new((false, false)) };
}

""" + anchor)

once("""    // I-7.18（根记录的回收门槛等于现算的门槛）、I-7.19（钉着的槽不住本实例写的根）：只判最新根、只判建账的发布（C773（读坏与全零根槽的回收门槛钉只在内存里））。
    judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root(""", """    // 辩方腿原型：journal 里有没有点名 txg + 1 的记录。
    {
        let newest_view = &roots[newest_index].2;
        let mut any = false;
        let mut same = false;
        for (_, records) in scanned_journal_records_by_device(reader, &geometry, filesystem_identifier_low) {
            for record in records.values() {
                if record.checkpoint_txg == newest_view.checkpoint_txg + 1 {
                    any = true;
                    if record.instance == newest_view.instance {
                        same = true;
                    }
                }
            }
        }
        I718_PREMISE.with(|cell| cell.set((any, same)));
    }
    I718_ALL.with(|cell| *cell.borrow_mut() = None);
    // I-7.18（根记录的回收门槛等于现算的门槛）、I-7.19（钉着的槽不住本实例写的根）：只判最新根、只判建账的发布（C773（读坏与全零根槽的回收门槛钉只在内存里））。
    judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root(""")
open(path, "w", encoding="utf-8").write(s)
print("patched defense switch")
