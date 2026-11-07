//! C700（三条判法各读各的根环没有跨装置检查）：池级 checker 的 I-3.5（引用区间的精确性）、I-3.1（已分配统计对得上）、
//! I-7.9（回退下界 F 不高于抬 F 的上限） 读的那一份根环（`singlefs_checker::walk::root_ring_the_ring_judgements_read`），逐位置与历史生成器的洞表
//! （`singlefs_verification::history::root_ring_hole_table_of_the_roots_written`：历史依次写进根环的根，按根环公式另算一份落点）比。
//! 读槽层读松（该有的根读成没有）、读严时三条判法一起偏、单看判定看不出，这一道比的是三套装置各算各的：实现写下的根、harness 的落点式子、
//! checker 的读槽。
mod common;

use common::{build_pool, parameters};
use singlefs_checker::walk::{root_ring_the_ring_judgements_read, RootRingSlotTheJudgementsRead};
use singlefs_verification::history::{
    root_ring_hole_table_of_the_roots_written, RootRingPositionContent,
};

/// 第一版根环的区域数 R（D22（单元原子性怎么合成） 已定项 2 / 已定项 16；D16（发布语义） 已定项 8 区域落在盘 0 / 1 / 0）。
const ROOT_RING_REGIONS_OF_THE_FIRST_VERSION: u64 = 3;

/// 写完新池新建文件的池：历史依次写进根环的是 mkfs 种的第 0 代根、暖机那几代根与新池新建文件那一版的根。洞表说哪几个位置有根、
/// 各是哪一条，其余位置是洞；池级 checker 那三条判法读的根环逐位置与它相同（有根的位置读出同一条根，洞读不出根）。
/// 判别力（`crates/mutations.tsv` 里点名这条的行）：池级 checker 读槽时丢掉一个区域的根，这条红。
#[test]
fn the_root_ring_the_three_ring_judgements_read_agrees_with_the_hole_table_of_a_new_pool_new_file_history(
) {
    let pool = build_pool("c700-root-ring-read-against-the-hole-table");
    let roots_written_in_order: Vec<(u32, u64)> = std::iter::once(&pool.genesis.root)
        .chain(pool.warm_up.roots.iter())
        .chain(std::iter::once(&pool.output.root))
        .map(|root| (root.instance.0, root.checkpoint_txg.0))
        .collect();
    let hole_table = root_ring_hole_table_of_the_roots_written(
        &roots_written_in_order,
        ROOT_RING_REGIONS_OF_THE_FIRST_VERSION,
        parameters().geometry.root_ring_slots_per_region.count(),
    );
    let expected: Vec<RootRingSlotTheJudgementsRead> = hole_table
        .content_of_each_position
        .iter()
        .filter_map(|(position, content)| match content {
            RootRingPositionContent::NeverWritten => None,
            RootRingPositionContent::LastRootWritten {
                instance,
                checkpoint_txg,
            } => Some(RootRingSlotTheJudgementsRead {
                region: position.region,
                slot_in_region: position.slot_in_region,
                instance: *instance,
                checkpoint_txg: *checkpoint_txg,
            }),
        })
        .collect();
    assert!(
        expected.len() >= 4,
        "洞表里至少第 0 代根、两代暖机根与新池新建文件那一版的根：{expected:?}"
    );
    assert!(
        hole_table
            .content_of_each_position
            .values()
            .any(|content| *content == RootRingPositionContent::NeverWritten),
        "根环还没转过一圈，洞表里有洞：{:?}",
        hole_table.content_of_each_position
    );
    let read =
        root_ring_the_ring_judgements_read(&pool.memory_pool()).expect("两块盘都择得到系统配置");
    assert_eq!(
        read, expected,
        "三条判法读的根环逐位置与历史的洞表相同（有根的位置是同一条根，洞读不出根）"
    );
}
