//! unlink 最后一批删掉一条 inode 记录之后 inode 树叶容器的合并与借（E176 M3 那一臂，第 50 批；判决
//! `research/prompts/m3-retire-r1-main-verification.md` 第三节第 3 条）在发布路径上：左吸收右那一次给被吸收的那一片写一条退役记录、
//! 死亡 txg = 这次发布的 txg；左从右借不写退役记录，右片保留身份、分隔 key 改成它这一版的最小 key（N1）；两种发布之后池级 checker 一条都不红
//! （I-9.4 原来的第三句「左容器的最大 key < 右容器号」随 N1 删掉，借之后右片容器号不大于左片最大 key 也不红）。
//! 空 inode 一次建好（一次发布写进全部记录，叶从左往右装满、233 条一片），再逐个按产品批大小 unlink 到底。

mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::BuiltPool;
use common_deletion_in_progress::{pool_with_new_inodes, unlink_to_the_end};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::records::RetirementRecord;
use singlefs_core::transaction::TransactionOutput;
use singlefs_core::unit::PackedIdentity;
use singlefs_format::SLOT_BYTES;

/// 一片 inode 叶容器装 233 条（`singlefs_format::INODE_LEAF_RECORDS`）。
const RECORDS_PER_LEAF: usize = 233;

fn leaf_identities(version: &TransactionOutput) -> Vec<PackedIdentity> {
    version
        .inode_leaf_containers
        .iter()
        .map(|container| container.contents.identity)
        .collect()
}

fn leaf_record_counts(version: &TransactionOutput) -> Vec<usize> {
    version
        .inode_leaf_containers
        .iter()
        .map(|container| container.contents.records.len())
        .collect()
}

fn retirement_records_of(version: &TransactionOutput) -> Vec<RetirementRecord> {
    version
        .retirement_containers
        .iter()
        .flat_map(|container| container.contents.records.iter().copied())
        .collect()
}

fn violated_invariants(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<&'static str> {
    verdicts
        .iter()
        .filter(|(_, found)| matches!(found, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect()
}

/// 第一个文件之外再建 233 个空 inode：234 条记录装成 [1..=233]、[234] 两片。unlink 2 号：左片剩 232 条，与右邻合计 233 条装得下一片，
/// 左片吸收右片、保留左片身份；右片身份消失，最后一批那次发布写一条 (右片身份, 这次的 txg)。池级 checker 一条都不红，退役完备（I-9.35）成立。
#[test]
fn the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one() {
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
    let verdicts = check_pool_image(&pool.memory_pool());
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "吸收之后一条都不红：{verdicts:?}"
    );
}

/// 第一个文件之外再建 `new_inodes` 个空 inode（叶从左往右 233 条一片装满，前三片 [1..=233]、[234..=466]、[467..=699]，第四片从 700 起），
/// 再把第二片最小的 117 个号逐个 unlink 到底：前 116 次第二片剩 ≥ 117 条、它与两边哪一对都合不进一片（第四片至少一条，第三片与第四片
/// 合计 ≥ 234），不合不借；第 117 次剩 116 条，低于 ⌈233 ÷ 2⌉ = 117，与右邻合计 349 条合不进一片，从右邻借 ⌊(233 − 116) ÷ 2⌋ = 58 条最小的
/// （467..=524）：两片 174 / 175 条。交回池、建完那一版各片的身份与第 117 次 unlink 的最后一批那一版。
fn pool_unlinked_until_the_second_leaf_borrows(
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
#[test]
fn a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing()
{
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
    let verdicts = check_pool_image(&pool.memory_pool());
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "借之后一条都不红：{verdicts:?}"
    );
}

/// 第四片 1 条（[700]）：借完右邻 175 条，与第四片合计 176 条装得下一片，借出的那一片接着吸收它的右邻（E176 M3「借完再从借出的那一片看合并」）：
/// 剩三片 233 / 174 / 176，第四片身份消失，同一次发布写一条 (第四片身份, 这次的 txg)。池级 checker 一条都不红。
#[test]
fn the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it() {
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
    let verdicts = check_pool_image(&pool.memory_pool());
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "借之后再吸收一条都不红：{verdicts:?}"
    );
}

/// 一次发布每块盘新写的槽：这次真正写出的角色（`rewritten`）各自的单元按槽宽取整加起来（第一版每个单元落池里每一块盘，各盘一样多）。
fn slots_newly_written_on_each_device(publish: &TransactionOutput) -> u64 {
    publish
        .units
        .iter()
        .filter(|unit| publish.rewritten.contains(&unit.identity))
        .map(|unit| {
            u64::try_from(unit.bytes.len())
                .expect("单元字节数装得进 u64")
                .div_ceil(SLOT_BYTES)
        })
        .sum()
}

/// R3（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 7 条，用户定「并进 M3 改」）在发布路径上的几何：第一个文件之外再建
/// `leaves × 233 − 1` 个空 inode，叶装成 `leaves` 片满的（第 k 片是 [233(k − 1) + 1, 233k]）。先把第一片的 2..=117、第二片的 234..=349 逐个 unlink 到底
/// （两片各剩 117 条：不低于借的下限 117，两片之和 234 合不进一片，一次都不并不借），再 unlink 118：第一片剩 116 条、与右邻合计 233 条，
/// 第一片吸收第二片。交回吸收之前那一版与吸收那一次（最后一批）发布。
fn the_first_leaf_absorbing_the_second_in_a_tree_of(
    tag: &str,
    leaves: u64,
) -> (TransactionOutput, TransactionOutput) {
    let records_per_leaf = u64::try_from(RECORDS_PER_LEAF).expect("233");
    let mut pool = pool_with_new_inodes(tag, leaves * records_per_leaf - 1);
    assert_eq!(
        leaf_record_counts(&pool.output),
        vec![RECORDS_PER_LEAF; usize::try_from(leaves).expect("片数装得进 usize")],
        "{leaves} 片满的叶"
    );
    // 迭代上界：两段各 116 个号。
    for inode in (2..=117).chain(234..=349) {
        unlink_to_the_end(&mut pool, inode);
    }
    let before = pool.output.clone();
    assert_eq!(
        leaf_record_counts(&before)[..3],
        [117, 117, RECORDS_PER_LEAF],
        "第一、二片各剩 117 条，一次都没并没借"
    );
    unlink_to_the_end(&mut pool, 118);
    (before, pool.output.clone())
}

/// R3 验收（第 50 批补做：纯 M3 下删空最左一片走不到——低于 117 条就从右邻借，删不空——改成用左吸收右）：20 片与 40 片两种几何上，
/// 第一片吸收第二片那一次每块盘新写的槽数相同，钉 14 槽；第二片右边那几片位置左移一格而内容没变，指针照抄上一版（不进重写清单）。
/// 判别力：删除类发布改回按位置重写、按位置换下右边每一片（`crates/mutations.tsv` 一行「第 50 批补做 R3：…」）时第二片右边那几片都换了新落点，
/// 先红在「指针照抄上一版」那一判（证红时量过）；只把挪了位置的片记成重写、不跟着改换下的叶序时发布路径同一个落点释放两次，红在分配器的断言上。
#[test]
fn the_first_leaf_absorbing_the_second_writes_as_many_slots_in_a_tree_of_twenty_leaves_as_in_a_tree_of_forty(
) {
    let (twenty_before, twenty) =
        the_first_leaf_absorbing_the_second_in_a_tree_of("absorb-the-second-of-twenty-leaves", 20);
    let (forty_before, forty) =
        the_first_leaf_absorbing_the_second_in_a_tree_of("absorb-the-second-of-forty-leaves", 40);
    for (leaves, before, absorbed) in [(20, &twenty_before, &twenty), (40, &forty_before, &forty)] {
        let identities_before = leaf_identities(before);
        assert_eq!(
            (
                leaf_identities(absorbed),
                retirement_records_of(absorbed)
                    .iter()
                    .map(|record| record.retired)
                    .collect::<Vec<_>>(),
            ),
            (
                [&identities_before[..1], &identities_before[2..]].concat(),
                vec![identities_before[1]],
            ),
            "{leaves} 片：第一片吸收第二片、保留自己的身份，第二片写一条退役记录"
        );
        assert_eq!(
            absorbed.inode_leaf_containers[1..]
                .iter()
                .map(|container| container.pointer)
                .collect::<Vec<_>>(),
            before.inode_leaf_containers[2..]
                .iter()
                .map(|container| container.pointer)
                .collect::<Vec<_>>(),
            "{leaves} 片：第二片右边的每一片挪了位置、内容没变，指针照抄上一版"
        );
    }
    assert_eq!(
        (
            slots_newly_written_on_each_device(&twenty),
            slots_newly_written_on_each_device(&forty),
        ),
        (14, 14),
        "吸收那一次每块盘新写的槽数不随右边的片数涨"
    );
}
