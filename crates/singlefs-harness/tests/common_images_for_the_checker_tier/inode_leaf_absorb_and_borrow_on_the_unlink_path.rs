//! harness 测试文件 `crates/singlefs-harness/tests/inode_leaf_absorb_and_borrow_on_the_unlink_path.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::BuiltPool;
use crate::common_deletion_in_progress::{pool_with_new_inodes, unlink_to_the_end};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::records::RetirementRecord;
use singlefs_core::transaction::TransactionOutput;
use singlefs_core::unit::PackedIdentity;

/// 一片 inode 叶容器装 233 条（`singlefs_format::INODE_LEAF_RECORDS`）。
pub const RECORDS_PER_LEAF: usize = 233;

pub fn leaf_identities(version: &TransactionOutput) -> Vec<PackedIdentity> {
    version
        .inode_leaf_containers
        .iter()
        .map(|container| container.contents.identity)
        .collect()
}

pub fn leaf_record_counts(version: &TransactionOutput) -> Vec<usize> {
    version
        .inode_leaf_containers
        .iter()
        .map(|container| container.contents.records.len())
        .collect()
}

pub fn retirement_records_of(version: &TransactionOutput) -> Vec<RetirementRecord> {
    version
        .retirement_containers
        .iter()
        .flat_map(|container| container.contents.records.iter().copied())
        .collect()
}

/// 第一个文件之外再建 233 个空 inode：234 条记录装成 [1..=233]、[234] 两片。unlink 2 号：左片剩 232 条，与右邻合计 233 条装得下一片，
/// 左片吸收右片、保留左片身份；右片身份消失，最后一批那次发布写一条 (右片身份, 这次的 txg)。池级 checker 一条都不红，退役完备（I-9.35）成立。
pub fn the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_with_new_inodes("absorb-the-right-neighbour", 233);
    let before = pool.output.clone();
    assert_eq!(
        leaf_record_counts(&before),
        [RECORDS_PER_LEAF, 1],
        "234 条记录装成满的一片与一条的一片"
    );
    let [left, right] = leaf_identities(&before)[..] else {
        panic!("两片叶")
    };
    unlink_to_the_end(&mut pool, 2);
    let final_batch = pool.output.clone();
    assert_eq!(
        (
            leaf_identities(&final_batch),
            leaf_record_counts(&final_batch)
        ),
        (vec![left], vec![RECORDS_PER_LEAF]),
        "左片吸收右片：保留左片身份，232 + 1 = 233 条"
    );
    assert_eq!(
        retirement_records_of(&final_batch),
        [RetirementRecord {
            retired: right,
            death_txg: final_batch.root.checkpoint_txg,
        }],
        "被吸收的右片写一条退役记录，死亡 txg = 最后一批那次的 txg"
    );
    images.hand_out(
        "吸收之后一条都不红",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// 第一个文件之外再建 `new_inodes` 个空 inode（叶从左往右 233 条一片装满，前三片 [1..=233]、[234..=466]、[467..=699]，第四片从 700 起），
/// 再把第二片最小的 117 个号逐个 unlink 到底：前 116 次第二片剩 ≥ 117 条、它与两边哪一对都合不进一片（第四片至少一条，第三片与第四片
/// 合计 ≥ 234），不合不借；第 117 次剩 116 条，低于 ⌈233 ÷ 2⌉ = 117，与右邻合计 349 条合不进一片，从右邻借 ⌊(233 − 116) ÷ 2⌋ = 58 条最小的
/// （467..=524）：两片 174 / 175 条。交回池、建完那一版各片的身份与第 117 次 unlink 的最后一批那一版。
pub fn pool_unlinked_until_the_second_leaf_borrows(
    tag: &str,
    new_inodes: u64,
) -> (BuiltPool, Vec<PackedIdentity>, TransactionOutput) {
    let mut pool = pool_with_new_inodes(tag, new_inodes);
    let identities_before = leaf_identities(&pool.output);
    assert_eq!(
        leaf_record_counts(&pool.output)[..3],
        [RECORDS_PER_LEAF; 3],
        "前三片满"
    );
    // 迭代上界：第二片最小的 116 个号，234..=349。
    for inode in 234..=349 {
        unlink_to_the_end(&mut pool, inode);
        assert_eq!(
            leaf_identities(&pool.output),
            identities_before,
            "unlink {inode} 之后第二片还有 ≥ 117 条：不合不借"
        );
    }
    unlink_to_the_end(&mut pool, 350);
    let borrowed = pool.output.clone();
    (pool, identities_before, borrowed)
}

/// 第四片 60 条（[700..=759]）：借完右邻 175 条，与第四片合计 235 条合不进一片，借之后不再合。四片身份都在，右邻的分隔 key 改成它这一版的最小
/// key 525（N1）。借不写退役记录：退役树一直零根。池级 checker 一条都不红（右邻容器号 467 不大于左片最大 key 524，原 I-9.4 第三句会红的那一形）。
pub fn a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing(
    images: &mut PoolCheckerImages,
) {
    let (pool, identities_before, borrowed) =
        pool_unlinked_until_the_second_leaf_borrows("borrow-from-the-right-neighbour", 758);
    assert_eq!(
        (leaf_identities(&borrowed), leaf_record_counts(&borrowed)),
        (identities_before, vec![RECORDS_PER_LEAF, 174, 175, 60]),
        "第二片 116 条从右邻借 58 条：174 / 175，四片身份都在"
    );
    assert_eq!(
        borrowed.inode_leaf_containers[2]
            .contents
            .separator_key_in_the_parent_entry,
        525,
        "右邻的分隔 key 改成它这一版的最小 key（N1）"
    );
    assert_eq!(
        retirement_records_of(&borrowed),
        Vec::<RetirementRecord>::new(),
        "借不写退役记录，退役树零根"
    );
    images.hand_out(
        "借之后一条都不红",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// 第四片 1 条（[700]）：借完右邻 175 条，与第四片合计 176 条装得下一片，借出的那一片接着吸收它的右邻（E176 M3「借完再从借出的那一片看合并」）：
/// 剩三片 233 / 174 / 176，第四片身份消失，同一次发布写一条 (第四片身份, 这次的 txg)。池级 checker 一条都不红。
pub fn the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it(
    images: &mut PoolCheckerImages,
) {
    let (pool, identities_before, borrowed) =
        pool_unlinked_until_the_second_leaf_borrows("lender-absorbs-after-lending", 699);
    assert_eq!(
        (leaf_identities(&borrowed), leaf_record_counts(&borrowed)),
        (
            identities_before[..3].to_vec(),
            vec![RECORDS_PER_LEAF, 174, 176]
        ),
        "借完之后借出的第三片吸收第四片"
    );
    assert_eq!(
        retirement_records_of(&borrowed),
        [RetirementRecord {
            retired: identities_before[3],
            death_txg: borrowed.root.checkpoint_txg,
        }],
        "被吸收的第四片写一条退役记录"
    );
    images.hand_out(
        "借之后再吸收一条都不红",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}
