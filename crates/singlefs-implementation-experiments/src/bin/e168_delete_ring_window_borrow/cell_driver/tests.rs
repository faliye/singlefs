//! 一格的驱动：跑一格、阳性对照的等式与删完之后的核对的单测。

use super::*;

thread_local! {
    /// 假的删除入口用的开关与计数：代替副本里补丁 R11 的 `FORMULA_JUDGEMENT_SKIPPED_FOR_TESTS`、R7 的判数与 R2 的 nlink 开关。
    static FAKE_FORMULA_JUDGEMENT_SKIPPED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static FAKE_FORMULA_JUDGEMENTS_MADE: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static FAKE_INODE_WRITTEN_WITH_ZERO_LINKS: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
    /// 假的删除入口每被调一次，记下那一刻 (式子那一判关没关, 写 nlink = 0 的 inode)。
    static FAKE_SWITCHES_SEEN_BY_EACH_UNLINK_BATCH: RefCell<Vec<(bool, Option<u64>)>> = const { RefCell::new(Vec::new()) };
}

/// 假的删除入口，照副本里补丁 R6、R7 的样子：式子那一判开着时真判一次并拒（r5 停机那一格删完之后每盘空闲 66 槽、保留量 86 槽，
/// 式子可用为负），关着时不判、交回上一版（落点那一判不在假入口里）。
fn fake_publish_unlink_batch(
    _writer: &mut PoolWriter<'_, SparseBlockDevice>,
    _allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    _kept_data_units: u64,
    _size_after_the_batch: u64,
    _instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let is_formula_judgement_skipped = FAKE_FORMULA_JUDGEMENT_SKIPPED.with(std::cell::Cell::get);
    let inode_written_with_zero_links =
        FAKE_INODE_WRITTEN_WITH_ZERO_LINKS.with(std::cell::Cell::get);
    FAKE_SWITCHES_SEEN_BY_EACH_UNLINK_BATCH.with(|cell| {
        cell.borrow_mut()
            .push((is_formula_judgement_skipped, inode_written_with_zero_links));
    });
    if is_formula_judgement_skipped {
        Ok(previous.clone())
    } else {
        FAKE_FORMULA_JUDGEMENTS_MADE.with(|cell| cell.set(cell.get() + 1));
        Err(PublishError::SpaceAdmissionRefused(
            singlefs_core::admission::AdmissionRefusedOnSomeDevices {
                short_devices: Vec::new(),
            },
        ))
    }
}

fn fake_unlink_batch_change(
    _kept_data_units: u64,
    _size_after_the_batch: u64,
) -> UserChange<'static> {
    panic!("S7 那一次直发（A 臂的发法），不经会话");
}

fn fake_set_formula_judgement_skipped(is_skipped: bool) {
    FAKE_FORMULA_JUDGEMENT_SKIPPED.with(|cell| cell.set(is_skipped));
}

fn fake_formula_judgements_made() -> u64 {
    FAKE_FORMULA_JUDGEMENTS_MADE.with(std::cell::Cell::get)
}

fn fake_set_inode_written_with_zero_links(inode: Option<u64>) {
    FAKE_INODE_WRITTEN_WITH_ZERO_LINKS.with(|cell| cell.set(inode));
}

fn fake_set_reclaim_switched_off(_is_switched_off: bool) {}

fn fake_allocation_record_tree_positions(
    _version: &TransactionOutput,
    _allocator: &PoolAllocator,
) -> u64 {
    panic!("S7 那一次发布不读 K1");
}

fn fake_delete_path_hooks() -> DeletePathHooks {
    DeletePathHooks {
        publish_unlink_batch: fake_publish_unlink_batch,
        unlink_batch_change: fake_unlink_batch_change,
        set_inode_written_with_zero_links: fake_set_inode_written_with_zero_links,
        set_formula_judgement_skipped: fake_set_formula_judgement_skipped,
        formula_judgements_made: fake_formula_judgements_made,
        set_reclaim_switched_off: fake_set_reclaim_switched_off,
        take_free_slots_recorded_at_each_root: take_the_fake_free_slots_recorded_at_each_root,
        set_ledger: leave_the_ledger_as_it_is,
        allocation_record_tree_positions: fake_allocation_record_tree_positions,
    }
}

#[test]
fn the_ledger_rewrite_before_the_final_checker_is_published_with_the_formula_judgement_skipped_like_every_delete_publish(
) {
    let PoolWithAFile::Written(mut pool) = pool_with_a_file(
        narrow_geometry_with_the_ring_window_reserved(ROOT_RING_SLOTS_PER_REGION_MINIMUM),
        1,
    ) else {
        panic!("单元区 790 槽、S = 4 上写得进一个单元的文件");
    };
    let hooks = fake_delete_path_hooks();
    let (ledger_rewrite, formula_judgements) =
        publish_the_ledger_rewrite_as_a_delete_publish(&mut pool, &hooks);
    assert_eq!(
        FAKE_SWITCHES_SEEN_BY_EACH_UNLINK_BATCH
            .with(|cell| std::mem::take(&mut *cell.borrow_mut())),
        vec![(true, Some(1))],
        "S7 那一次删除发布只发一次，发的那一刻式子那一判关着、inode 1 写 nlink = 0（登记 5.1 第 4 步：删除类发布不判式子那一判）"
    );
    assert_eq!(formula_judgements, 0, "其间式子那一判一次都没判");
    assert_eq!(
        ledger_rewrite.refusal, None,
        "式子那一判关着就不被它拒（2026-10-02 第一段 r5 那一行 `refusal=landing filled_slots=[0, 0]` 是式子那一判开着拒的，被记成了落点）"
    );
    assert!(
        !FAKE_FORMULA_JUDGEMENT_SKIPPED.with(std::cell::Cell::get),
        "发完把式子那一判开回来"
    );
    assert_eq!(
        FAKE_INODE_WRITTEN_WITH_ZERO_LINKS.with(std::cell::Cell::get),
        None,
        "发完 nlink 开关恢复"
    );
}
