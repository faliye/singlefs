//! unlink 最后一批删掉一条 inode 记录之后 inode 树叶容器的合并与借（E176 M3 那一臂，第 50 批；判决
//! `research/prompts/m3-retire-r1-main-verification.md` 第三节第 3 条）在发布路径上：左吸收右那一次给被吸收的那一片写一条退役记录、
//! 死亡 txg = 这次发布的 txg；左从右借不写退役记录，右片保留身份、分隔 key 改成它这一版的最小 key（N1）；两种发布之后池级 checker 一条都不红
//! （I-9.4 原来的第三句「左容器的最大 key < 右容器号」随 N1 删掉，借之后右片容器号不大于左片最大 key 也不红）。
//! 空 inode 一次建好（一次发布写进全部记录，叶从左往右装满、233 条一片），再逐个按产品批大小 unlink 到底。
mod common;
mod common_deletion_in_progress;
mod common_images_for_the_checker_tier;
mod common_pointer_head_corruption;
#[path = "common_images_for_the_checker_tier/inode_leaf_absorb_and_borrow_on_the_unlink_path.rs"]
mod scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path;

use common_deletion_in_progress::{pool_with_new_inodes, unlink_to_the_end};
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::{
    leaf_identities, leaf_record_counts, retirement_records_of, RECORDS_PER_LEAF,
};
use singlefs_core::transaction::TransactionOutput;
use singlefs_format::SLOT_BYTES;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one() {
    scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing()
{
    scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it() {
    scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it(&mut PoolCheckerImages::left_to_the_checker_tier());
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
