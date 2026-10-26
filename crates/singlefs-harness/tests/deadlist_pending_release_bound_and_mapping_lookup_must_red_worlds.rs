//! 池级 checker 的 I-3.6（deadlist 紧界） 新并进的三格在写路径写出的 deadlist 上的必红世界与不许红的合法态：
//! - 待放集的上界（判决 `research/prompts/m4-c145-destruction-bound-r1-main-verification.md` 第二节 B2）：下一侧是活头时上界取被判那一版的
//!   checkpoint txg——意图与最后一批之间把一条待放条目的纪元抬到这一版 txg 以上必红（上界取 ∞ 时它照算待放、不红）；先删掉的快照留下的、
//!   纪元大于 lo 的合法待放条目不许红（上界收成 lo + 1 时它红）。
//! - 每个纪元恰好落进一份区间（`.claude/kb/checks-owed.md` C145 第一条，用户 2026-10-26 定并进 I-3.6）：前驱链首尾相接而一条条目落不进任何一侧必红。
//!   写路径上最早那一侧的前驱恒为 0（I-9.25 那一行「没有为 0」），落不进任何一侧要连树表里那条前驱一起改：这一格的必红世界同时让 I-9.25 红，
//!   用例只断言 I-3.6 红、红在哪一句。
//! - deadlist key 在同一版中央映射里查得到（C300 的 checker 一半，用户 2026-10-26 定第一版判损坏停住）：删掉一条被 deadlist 记着的留存映射条目、
//!   重封到根必红；合法的销毁中间态（意图那一版）与完成态（最后一批之后）照旧一条违例都没有。
//!
//! 搭建在 `tests/common_deadlist_pending_release/mod.rs`（checker 档的 GPU 对拍经 `#[path]` 引同一份）。
mod common;
mod common_deadlist_pending_release;
mod common_pointer_head_corruption;

use common_deadlist_pending_release::{
    change_the_epoch_of_the_data_unit_entry,
    image_whose_deadlist_epoch_falls_below_every_side_of_a_contiguous_chain,
    image_whose_kept_mapping_entry_of_a_deadlist_key_is_deleted,
    pool_after_the_destruction_intent_under_an_overwrite,
    pool_whose_pending_entries_lie_above_the_lower_bound_of_the_intent,
    walk_the_destruction_to_the_end_and_advance, TXG_OF_THE_FIRST_SNAPSHOT,
    TXG_OF_THE_SECOND_SNAPSHOT,
};
use singlefs_checker::image::{
    InvariantVerdict, DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_harness::memory_pool::MemoryPool;

fn verdict_of_the_tight_bound(image: &MemoryPool) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(candidate, _)| {
            *candidate
                == DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE
        })
        .expect("清单里有这一条")
        .1
}

fn violations(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, found)| match found {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

fn assert_the_tight_bound_violated_mentioning(image: &MemoryPool, fragment: &str, what: &str) {
    match verdict_of_the_tight_bound(image) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains(fragment),
            "{what}：I-3.6 的违例说明里要有「{fragment}」，实际 {detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("{what}：I-3.6 要判红，实际 {other:?}")
        }
    }
}

/// 必红世界（判决 B2「纪元抬到本版 txg 以上」）：快照下覆盖写、写删快照的意图（意图与最后一批之间，四条条目都在待放集里、还没放）；
/// 钉数据单元的那一条纪元由 S.txg 4 抬到意图那一版的 txg + 1。下一侧是活头、上界取这一版的 txg，它不再算待放：没有活快照钉着它，
/// birth 3 大于活头那一侧的前驱 0。上界取 ∞ 时它照算待放，I-3.6 与 I-3.5 逆向都不红。
#[test]
fn a_pending_entry_whose_epoch_is_raised_above_the_txg_of_the_version_reddens_the_tight_bound() {
    let pool =
        pool_after_the_destruction_intent_under_an_overwrite("deadlist-pending-epoch-raised");
    let intent_txg = pool.output.root.checkpoint_txg.0;
    let mut image = pool.memory_pool();
    assert_eq!(
        verdict_of_the_tight_bound(&image),
        InvariantVerdict::Holds,
        "改之前：意图那一版的待放条目豁免于紧界"
    );
    change_the_epoch_of_the_data_unit_entry(&mut image, TXG_OF_THE_FIRST_SNAPSHOT, intent_txg + 1);
    assert_the_tight_bound_violated_mentioning(
        &image,
        "也不在未完成销毁意图的待放集里",
        "纪元抬到意图那一版 txg 以上的条目不算待放",
    );
}

/// 合法态（判决 B2「纪元 > lo 的合法待放条目」不许红）：建快照 X（txg 4）、Y（txg 5），快照下覆盖写，删 Y 到最后一批，再写删 X 的意图：
/// 四条条目纪元 5 都大于 lo 4、小于这一版的 txg，都在待放集里。一条违例都没有；上界收成 lo + 1 时四条都不算待放，I-3.6 与 I-3.5 逆向红。
#[test]
fn pending_entries_whose_epoch_is_above_the_lower_bound_of_the_intent_hold_every_invariant() {
    let pool = pool_whose_pending_entries_lie_above_the_lower_bound_of_the_intent(
        "deadlist-pending-epoch-above-the-lower-bound",
    );
    let epochs: Vec<u64> = pool
        .output
        .deadlist_entries
        .iter()
        .map(|entry| entry.epoch.0)
        .collect();
    assert!(
        !epochs.is_empty() && epochs.iter().all(|epoch| *epoch == TXG_OF_THE_SECOND_SNAPSHOT),
        "删 X 的意图那一版：deadlist 里留着的条目纪元都是 Y 的 S.txg 5（大于 lo 4），实际 {epochs:?}"
    );
    assert!(
        pool.output.root.checkpoint_txg.0 > TXG_OF_THE_SECOND_SNAPSHOT,
        "纪元 5 小于这一版的 txg"
    );
    let image = pool.memory_pool();
    assert_eq!(
        violations(&image),
        [],
        "纪元大于 lo 的待放条目：一条违例都没有"
    );
    assert_eq!(
        verdict_of_the_tight_bound(&image),
        InvariantVerdict::Holds,
        "I-3.6 判到且成立"
    );
}

/// 必红世界（C145 第一条：每个纪元恰好落进一份区间）：快照下覆盖写之后，钉数据单元的那一条 deadlist 条目纪元由 4 改成 1，最新那一版树表里
/// 快照那条码 14 的前驱由 0 改成 2（前驱链照样首尾相接）：纪元 1 比最早那一侧的前驱 2 小，落不进任何一侧。
#[test]
fn a_deadlist_entry_whose_epoch_falls_below_every_side_of_a_contiguous_chain_reddens_the_tight_bound(
) {
    let image = image_whose_deadlist_epoch_falls_below_every_side_of_a_contiguous_chain(
        "deadlist-epoch-below-every-side",
    );
    assert_the_tight_bound_violated_mentioning(
        &image,
        "落不进任何一侧",
        "纪元 1 比最早那一侧的前驱 2 小",
    );
}

/// 必红世界（C300：deadlist key 在同一版中央映射里查得到）：快照下覆盖写之后，钉数据单元的那一条 deadlist 条目的留存映射条目从最新那一版的
/// 中央映射叶里删掉、重封到根。
#[test]
fn a_kept_mapping_entry_deleted_from_the_central_mapping_reddens_the_tight_bound() {
    let image = image_whose_kept_mapping_entry_of_a_deadlist_key_is_deleted(
        "deadlist-kept-mapping-entry-deleted",
    );
    assert_the_tight_bound_violated_mentioning(
        &image,
        "中央映射里查不到",
        "被 deadlist 记着的留存映射条目没了",
    );
}

/// 合法的销毁中间态（C300 那一格不许红）：快照下覆盖写、写删快照的意图：四条条目都在、留存的映射条目都在，一条违例都没有、I-3.6 判到且成立。
#[test]
fn the_destruction_intent_version_holds_every_invariant_with_the_mapping_lookup_of_the_deadlist() {
    let pool =
        pool_after_the_destruction_intent_under_an_overwrite("deadlist-lookup-after-the-intent");
    let image = pool.memory_pool();
    assert_eq!(violations(&image), [], "写意图那一版一条违例都没有");
    assert_eq!(
        verdict_of_the_tight_bound(&image),
        InvariantVerdict::Holds,
        "I-3.6 判到且成立"
    );
}

/// 合法的销毁完成态（C300 那一格不许红）：再走到最后一批（同一次发布放掉条目、删掉留存的映射条目与那一对树表条目）：环里还留着意图那条根，
/// 两版都判，一条违例都没有。
#[test]
fn the_version_after_the_last_destruction_batch_holds_every_invariant_with_the_mapping_lookup_of_the_deadlist(
) {
    let mut pool = pool_after_the_destruction_intent_under_an_overwrite(
        "deadlist-lookup-after-the-last-batch",
    );
    walk_the_destruction_to_the_end_and_advance(&mut pool);
    assert!(
        pool.output.deadlist_entries.is_empty(),
        "最后一批之后 deadlist 空了"
    );
    let image = pool.memory_pool();
    assert_eq!(violations(&image), [], "最后一批之后一条违例都没有");
    assert_eq!(
        verdict_of_the_tight_bound(&image),
        InvariantVerdict::Holds,
        "I-3.6 判到且成立"
    );
}
