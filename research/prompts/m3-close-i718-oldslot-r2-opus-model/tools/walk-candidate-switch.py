"""在仓副本的 walk.rs 上装 I-7.18 候选开关（攻方腿原型，只在副本里）。"""
import sys
path = sys.argv[1]
src = open(path, encoding="utf-8").read()
old_judge = """    let (newest_instance, newest_txg) = (newest_root.instance, newest_root.checkpoint_txg);
    judgements.judge(
        ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD,
        newest_root.reclaim_threshold == recomputed_threshold,
"""
new_judge = """    let (newest_instance, newest_txg) = (newest_root.instance, newest_root.checkpoint_txg);
    // 攻方腿原型：I-7.18 的候选开关（甲、乙、丙与攻方提的变体），见 `i718_candidate_for_the_attack_probe`。
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
    let next_slot_unreadable = !slot_holds_a_self_verified_root(slot_of_txg(next_txg));
    let threshold_holds = match i718_candidate_for_the_attack_probe() {
        I718Candidate::CurrentOrBing => newest_root.reclaim_threshold == recomputed_threshold,
        I718Candidate::YiLiteral | I718Candidate::YiAfterTheFirstLap
            if next_slot_unreadable
                && (i718_candidate_for_the_attack_probe() == I718Candidate::YiLiteral
                    || next_txg > ring_length_in_txgs) =>
        {
            judgements.not_applicable(
                ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD,
                "候选乙：最新根之后那一槽读不出自证过的根",
            );
            I718_PROBE_COUNTERS.with(|counters| counters.borrow_mut().0 += 1);
            return judge_the_pinned_ring_slot_only(
                roots, newest_root, geometry, judgements,
            );
        }
        I718Candidate::YiLiteral | I718Candidate::YiAfterTheFirstLap => {
            newest_root.reclaim_threshold == recomputed_threshold
        }
        I718Candidate::Jia => {
            newest_root.reclaim_threshold == recomputed_threshold
                || (next_slot_unreadable
                    && next_txg > ring_length_in_txgs
                    && newest_root.reclaim_threshold
                        == recomputed_with_one_more_valid_txg(next_txg - ring_length_in_txgs))
        }
        I718Candidate::JiaAnyLapOfTheNextSlot => {
            newest_root.reclaim_threshold == recomputed_threshold
                || (next_slot_unreadable
                    && next_txg > ring_length_in_txgs
                    && (1..=next_txg / ring_length_in_txgs).any(|laps| {
                        newest_root.reclaim_threshold
                            == recomputed_with_one_more_valid_txg(next_txg - laps * ring_length_in_txgs)
                    }))
        }
        I718Candidate::JiaAnyUnreadableSlot => {
            newest_root.reclaim_threshold == recomputed_threshold
                || (0..ring_length_in_txgs).any(|slot_value| {
                    let slot = slot_of_txg(slot_value);
                    !slot_holds_a_self_verified_root(slot)
                        && (0..=newest_txg / ring_length_in_txgs).any(|laps| {
                            let txg = slot_value + laps * ring_length_in_txgs;
                            let latest_allowed = if slot == slot_of_txg(next_txg) {
                                next_txg.checked_sub(ring_length_in_txgs)
                            } else {
                                Some(newest_txg)
                            };
                            latest_allowed.is_some_and(|latest| txg <= latest)
                                && newest_root.reclaim_threshold == recomputed_with_one_more_valid_txg(txg)
                        })
                })
        }
    };
    I718_PROBE_COUNTERS.with(|counters| counters.borrow_mut().1 += 1);
    judgements.judge(
        ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD,
        threshold_holds,
"""
assert src.count(old_judge) == 1
src = src.replace(old_judge, new_judge)
anchor = "/// I-7.18（根记录的回收门槛等于现算的门槛） 与 I-7.19（钉着的槽不住本实例写的根）：只判最新根，只判建账的发布\n"
assert src.count(anchor) == 1
helpers = '''/// 攻方腿原型：I-7.18 的候选。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum I718Candidate {
    CurrentOrBing,
    Jia,
    YiLiteral,
    YiAfterTheFirstLap,
    JiaAnyLapOfTheNextSlot,
    JiaAnyUnreadableSlot,
}

static I718_CANDIDATE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(u8::MAX);

thread_local! {
    /// （乙报不适用的次数, I-7.18 真判的次数）
    pub static I718_PROBE_COUNTERS: std::cell::RefCell<(u64, u64)> = const { std::cell::RefCell::new((0, 0)) };
}

pub fn set_i718_candidate_for_the_attack_probe(candidate: I718Candidate) {
    I718_CANDIDATE.store(candidate as u8, std::sync::atomic::Ordering::SeqCst);
}

pub fn i718_candidate_for_the_attack_probe() -> I718Candidate {
    let stored = I718_CANDIDATE.load(std::sync::atomic::Ordering::SeqCst);
    let chosen = if stored == u8::MAX {
        match std::env::var("I718_CANDIDATE").as_deref() {
            Ok("jia") => I718Candidate::Jia,
            Ok("yi") => I718Candidate::YiLiteral,
            Ok("yi_after_first_lap") => I718Candidate::YiAfterTheFirstLap,
            Ok("jia_any_lap") => I718Candidate::JiaAnyLapOfTheNextSlot,
            Ok("jia_any_unreadable") => I718Candidate::JiaAnyUnreadableSlot,
            _ => I718Candidate::CurrentOrBing,
        }
    } else {
        [
            I718Candidate::CurrentOrBing,
            I718Candidate::Jia,
            I718Candidate::YiLiteral,
            I718Candidate::YiAfterTheFirstLap,
            I718Candidate::JiaAnyLapOfTheNextSlot,
            I718Candidate::JiaAnyUnreadableSlot,
        ][usize::from(stored)]
    };
    chosen
}

/// 候选乙报 I-7.18 不适用时，I-7.19 照原样判（抄原函数末尾那一段）。
fn judge_the_pinned_ring_slot_only(
    roots: &[(u64, u64, crate::RootView)],
    newest_root: &crate::RootView,
    geometry: &PoolGeometry,
    judgements: &mut Judgements,
) {
    let newest_instance = newest_root.instance;
    match pinned_ring_slot_of_the_root(newest_root.pinned_ring_slot, geometry) {
        PinnedRingSlotOfTheRoot::NoSlotPinned => {
            judgements.judge(PINNED_RING_SLOT_HOLDS_NO_ROOT_OF_THE_SAME_INSTANCE, true, String::new);
        }
        PinnedRingSlotOfTheRoot::OutsideTheRing(_) => {
            judgements.judge(PINNED_RING_SLOT_HOLDS_NO_ROOT_OF_THE_SAME_INSTANCE, false, || {
                "钉槽还原不成一个槽".to_string()
            });
        }
        PinnedRingSlotOfTheRoot::Pinned { region, slot, .. } => {
            let held = roots.iter().any(|(root_region, root_slot, root)| {
                (*root_region, *root_slot) == (region, slot) && root.instance == newest_instance
            });
            judgements.judge(PINNED_RING_SLOT_HOLDS_NO_ROOT_OF_THE_SAME_INSTANCE, !held, || {
                "钉槽指着本实例写的根".to_string()
            });
        }
    }
}

'''
src = src.replace(anchor, helpers + anchor)
open(path, "w", encoding="utf-8").write(src)
print("patched")
