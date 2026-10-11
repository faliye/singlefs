//! inode 树叶容器的合并与借在发布路径上逐次对一份独立参照（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F1；
//! 历史补段、比分隔 key、各形至少走到一次照 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第三节 G4）：
//! 参照按条款另写一份，不引 `singlefs_core::inode_tree` 的合并、借、摘片代码，只从发布出来的那一版读记录、身份与分隔 key——
//! D8（核心索引结构） 已定项 6：插入落在最右的叶、满 233 条就在末尾分裂，右半容器号 = 那条新记录的 inode 号、出生代 = 这次发布的 txg，
//! 新片的分隔 key = 它的容器号，左从右借的那次发布右片的分隔 key 改成它这一版的最小 key；
//! 合并时机照 E176（inode 树叶合并三候选的容量与写代价） M3 那一臂（`scripts/experiments/e176_inode_leaf_merge_capacity_and_rewrites.rs`
//! 的 `remove`）：删空的整片摘掉、身份消失，它的新左右邻从左邻起并；没删空就从它起并——先查它与右邻、再查左邻与它，两片之和 ≤ 233 就左吸收右、
//! 被吸收的那一片身份消失，并完从幸存那片接着查；从它起一次都没并、它少于 ⌈233 ÷ 2⌉ 条、它有右邻时从右邻借最小的 ⌊(右 − 左) ÷ 2⌋ 条，
//! 两片都保留身份，再从借出的那一片起并。
//! 历史（种子固定）分五段：
//! 一、随机切几批建 583–599 个空 inode（叶 [233, 233, ≥ 117]），再一次次 unlink「可删条数最多的那片叶」（并列取最左）里随机的一个号，
//!    直到参照见到一次「删完之后与左邻、右邻两对都装得下」——M3 的方向只有这一形分得出来（攻方第二次抽样 K4-4：三片各 117 条删中间一条，
//!    两种写法退役不同的叶、池级 checker 都不红）；先右之后剩 [117, 233]。
//! 二、删空摘片：再建 1–8 个（最右那片满着，新开一片），一次次 unlink 最大的活号，直到最右那片删空摘掉（它的左邻满 233 条，删到最后一条之前
//!    左邻那一对都装不下）；之后回到 [117, 233]。
//! 三、借与借后吸收：再建 1–58 个（新开第三片），unlink 第一片里随机的一个号：第一片剩 116 条、从第二片借 ⌊(233 − 116) ÷ 2⌋ = 58 条，
//!    第二片剩 175 条，与第三片之和 ≤ 233，借出的那一片接着吸收第三片。
//! 四、再随机 unlink 几个活着的号。
//! 五、左吸收：一次次 unlink 最大的活号，直到最右那片与左邻之和恰好 233（它没有右邻，只有左邻那一对装得下），左邻吸收它。
//! 每次用户改动之后逐片比记录号、身份与分隔 key，逐次比这次退役的身份集与死亡 txg；末了断言两对都装得下（先右）、删空摘片、借、借后吸收、
//! 左吸收这五形参照里各至少走到一次。

mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::{build_pool, BuiltPool};
use common_deletion_in_progress::{publish_new_inodes_and_advance_the_pool, unlink_to_the_end};
use singlefs_core::transaction::TransactionOutput;
use singlefs_format::INODE_LEAF_RECORDS;
use singlefs_harness::history::SeededRandomSource;

/// 第一个文件那条 inode 记录（号 1）：历史里不 unlink 它，它在最左那片叶里。
const FIRST_FILE_INODE: u64 = 1;
/// inode 叶容器的打包记录类型（D8（核心索引结构） 已定项 6：类型 2）。
const PACKED_RECORD_TYPE_INODE: u16 = 2;
/// 这段历史的种子。
const SEED_OF_THE_HISTORY: u64 = 53;
/// 建 inode 那几批合起来至少建这么多个：叶装成 [233, 233, 117]，删到三片各 117 条的那一刻正是「两对都装得下」。
const NEW_INODES_AT_LEAST: u64 = 582;
/// 再往上随机多建 0..这么多个（叶的第三片 117..=133 条，仍删得到「两对都装得下」那一形）。
const NEW_INODES_ABOVE_THE_LEAST_BELOW: u64 = 17;
/// 一批至多建这么多个。
const NEW_INODES_PER_BATCH_AT_MOST: u64 = 260;
/// 删的那一段至多这么多次 unlink：三片从 [233, 233, 133] 删到各 117 条至多 250 次（参照的推算），见到那一形之后另删几次。
const UNLINKS_UNTIL_BOTH_PAIRS_FIT_AT_MOST: u64 = 260;
/// 第二段新开的那一片至多装这么多条（1 起随机）；一条一条删最大的活号，删这么多次它就删空。
const RECORDS_OF_THE_LEAF_EMPTIED_BY_UNLINKING_THE_LARGEST_AT_MOST: u64 = 8;
/// 第三段新开的第三片至多装这么多条（1 起随机）：第一片 116 条从第二片 233 条借 ⌊(233 − 116) ÷ 2⌋ = 58 条，第二片剩 175 条，
/// 与第三片之和 ≤ 233 要第三片 ≤ 233 − 175 = 58。
const RECORDS_OF_THE_LEAF_THE_LENDER_ABSORBS_AT_MOST: u64 = 58;
/// 第四段随机 unlink 几个活着的号。
const RANDOM_UNLINKS_BEFORE_ONLY_THE_LEFT_PAIR_FITS: u64 = 6;
/// 第五段至多这么多次 unlink 最大的活号：最右那片至多 233 条，删到与左邻之和恰好 233 之前删不完它。
const UNLINKS_UNTIL_ONLY_THE_LEFT_PAIR_FITS_AT_MOST: u64 = 233;

/// 一片叶容器的身份四段（出生树、打包记录类型、容器号、容器出生代）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ReferenceLeafIdentity {
    birth_tree: u64,
    record_type: u16,
    container: u64,
    container_birth: u64,
}

/// 参照里的一片叶：身份、它在父节点条目里的分隔 key 与它装的 inode 号（升序）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ReferenceLeaf {
    identity: ReferenceLeafIdentity,
    separator_key: u64,
    inode_numbers: Vec<u64>,
}

/// 参照：按叶序的叶，加上这次改动里身份消失的那几片（按消失的先后），与五形各走到几次。
struct ReferenceLeafShape {
    inode_tree: u64,
    leaves: Vec<ReferenceLeaf>,
    retired_in_this_change: Vec<ReferenceLeafIdentity>,
    /// 删完之后与左邻、右邻两对都装得下（M3 先右）。
    times_both_neighbour_pairs_fit: u64,
    /// 右邻那一对装不下（或没有右邻）、左邻那一对装得下，左邻吸收它。
    times_only_the_left_neighbour_pair_fit: u64,
    /// 删空的那一片整片摘掉。
    times_an_emptied_leaf_was_dropped: u64,
    /// 少于下限的那一片从右邻借。
    times_a_leaf_borrowed_from_its_right_neighbour: u64,
    /// 借完从借出的那一片起并，并掉了至少一对。
    times_the_lender_absorbed_after_lending: u64,
}

fn leaf_capacity() -> usize {
    usize::try_from(INODE_LEAF_RECORDS).expect("233")
}

/// 均分借的下限 ⌈233 ÷ 2⌉（E176 M3 主取样点）。
fn borrow_floor() -> usize {
    leaf_capacity().div_ceil(2)
}

impl ReferenceLeafShape {
    /// 从发布出来的那一版读起：每片叶的身份、分隔 key 与记录号。
    fn of_the_version(version: &TransactionOutput) -> Self {
        let leaves = leaves_of_the_version(version);
        let inode_tree = leaves
            .first()
            .expect("第一个文件之后 inode 树至少一片叶")
            .identity
            .birth_tree;
        Self {
            inode_tree,
            leaves,
            retired_in_this_change: Vec::new(),
            times_both_neighbour_pairs_fit: 0,
            times_only_the_left_neighbour_pair_fit: 0,
            times_an_emptied_leaf_was_dropped: 0,
            times_a_leaf_borrowed_from_its_right_neighbour: 0,
            times_the_lender_absorbed_after_lending: 0,
        }
    }

    /// 一条新号追加到最右那片；最右那片满 233 条（或树是空的）就新开一片：容器号与分隔 key = 这个号、出生代 = 这次发布的 txg。
    fn append(&mut self, inode: u64, txg_of_the_publish: u64) {
        let rightmost_is_full = self
            .leaves
            .last()
            .is_none_or(|leaf| leaf.inode_numbers.len() == leaf_capacity());
        if rightmost_is_full {
            self.leaves.push(ReferenceLeaf {
                identity: ReferenceLeafIdentity {
                    birth_tree: self.inode_tree,
                    record_type: PACKED_RECORD_TYPE_INODE,
                    container: inode,
                    container_birth: txg_of_the_publish,
                },
                separator_key: inode,
                inode_numbers: Vec::new(),
            });
        }
        self.leaves
            .last_mut()
            .expect("上面保证了至少一片")
            .inode_numbers
            .push(inode);
    }

    fn records_at(&self, position: usize) -> usize {
        self.leaves[position].inode_numbers.len()
    }

    fn record_counts(&self) -> Vec<usize> {
        self.leaves
            .iter()
            .map(|leaf| leaf.inode_numbers.len())
            .collect()
    }

    fn drop_leaf(&mut self, position: usize) -> ReferenceLeaf {
        let dropped = self.leaves.remove(position);
        self.retired_in_this_change.push(dropped.identity);
        dropped
    }

    /// 左片吸收右邻：左片的身份与分隔 key 不变，右邻的记录接在后面。
    fn left_absorbs_its_right_neighbour(&mut self, left_position: usize) {
        let absorbed = self.drop_leaf(left_position + 1);
        self.leaves[left_position]
            .inode_numbers
            .extend(absorbed.inode_numbers);
    }

    /// 从 `start_position` 那片起：先查它与右邻、再查左邻与它，和 ≤ 233 就左吸收右；并完从幸存那片接着查。交回并没并过。
    /// 迭代上界是片数：每一轮并掉一片或退出。
    fn merge_from(&mut self, start_position: usize) -> bool {
        let mut position = start_position;
        let mut merged = false;
        loop {
            let right_pair_fits = position + 1 < self.leaves.len()
                && self.records_at(position) + self.records_at(position + 1) <= leaf_capacity();
            let left_pair_fits = position > 0
                && self.records_at(position - 1) + self.records_at(position) <= leaf_capacity();
            if right_pair_fits && left_pair_fits {
                self.times_both_neighbour_pairs_fit += 1;
            }
            if right_pair_fits {
                self.left_absorbs_its_right_neighbour(position);
            } else if left_pair_fits {
                self.times_only_the_left_neighbour_pair_fit += 1;
                self.left_absorbs_its_right_neighbour(position - 1);
                position -= 1;
            } else {
                return merged;
            }
            merged = true;
        }
    }

    /// 均分借：`position` 那片少于下限、有右邻时，右邻最小的 ⌊(右 − 左) ÷ 2⌋ 条挪到它末尾，右邻的分隔 key 改成它这一版的最小 key，
    /// 再从右邻起并。
    fn borrow_from_the_right_if_below_the_floor(&mut self, position: usize) {
        if position + 1 >= self.leaves.len() || self.records_at(position) >= borrow_floor() {
            return;
        }
        self.times_a_leaf_borrowed_from_its_right_neighbour += 1;
        let moved_count = (self.records_at(position + 1) - self.records_at(position)) / 2;
        let moved: Vec<u64> = self.leaves[position + 1]
            .inode_numbers
            .drain(..moved_count)
            .collect();
        self.leaves[position].inode_numbers.extend(moved);
        let lender = &mut self.leaves[position + 1];
        lender.separator_key = *lender
            .inode_numbers
            .first()
            .expect("借出的条数少于右邻的条数（两片之和 > 233、左片 < 117）：右邻借完还有记录");
        if self.merge_from(position + 1) {
            self.times_the_lender_absorbed_after_lending += 1;
        }
    }

    /// unlink 最后一批删掉 `inode` 那条记录。
    fn remove(&mut self, inode: u64) {
        let position = self
            .leaves
            .iter()
            .position(|leaf| leaf.inode_numbers.contains(&inode))
            .expect("删的号在参照里");
        self.leaves[position]
            .inode_numbers
            .retain(|kept| *kept != inode);
        if self.leaves[position].inode_numbers.is_empty() {
            self.times_an_emptied_leaf_was_dropped += 1;
            self.drop_leaf(position);
            if position > 0 && position < self.leaves.len() {
                self.merge_from(position - 1);
            }
        } else if !self.merge_from(position) {
            self.borrow_from_the_right_if_below_the_floor(position);
        }
    }

    /// 可删条数（不算第一个文件那条）最多的那片叶的位置，并列取最左。
    fn position_of_the_leaf_with_the_most_removable_records(&self) -> usize {
        let removable = |leaf: &ReferenceLeaf| {
            leaf.inode_numbers
                .iter()
                .filter(|inode| **inode != FIRST_FILE_INODE)
                .count()
        };
        let most = self.leaves.iter().map(removable).max().expect("至少一片");
        self.leaves
            .iter()
            .position(|leaf| removable(leaf) == most)
            .expect("取到的最大值来自其中一片")
    }

    /// 第 `position` 片里随机一个可删的号（不是第一个文件那条）。
    fn random_removable_inode_of_the_leaf(
        &self,
        position: usize,
        random: &mut SeededRandomSource,
    ) -> u64 {
        let removable: Vec<u64> = self.leaves[position]
            .inode_numbers
            .iter()
            .copied()
            .filter(|inode| *inode != FIRST_FILE_INODE)
            .collect();
        let pick =
            usize::try_from(random.below(u64::try_from(removable.len()).expect("一片至多 233 条")))
                .expect("小于 233");
        removable[pick]
    }

    fn live_removable_inodes(&self) -> Vec<u64> {
        self.leaves
            .iter()
            .flat_map(|leaf| leaf.inode_numbers.iter().copied())
            .filter(|inode| *inode != FIRST_FILE_INODE)
            .collect()
    }

    /// 最大的活号：最右那片的最后一条。
    fn largest_live_inode(&self) -> u64 {
        let largest = *self
            .leaves
            .last()
            .and_then(|leaf| leaf.inode_numbers.last())
            .expect("树里至少一条记录");
        assert_ne!(
            largest, FIRST_FILE_INODE,
            "最大的活号是第一个文件那条：历史不删它"
        );
        largest
    }
}

fn identity_of(identity: &singlefs_core::unit::PackedIdentity) -> ReferenceLeafIdentity {
    ReferenceLeafIdentity {
        birth_tree: identity.birth_tree.0,
        record_type: identity.record_type,
        container: identity.container,
        container_birth: identity.container_birth.0,
    }
}

/// 发布出来的那一版里按叶序的每片叶：身份、父节点条目里的分隔 key 与记录号。
fn leaves_of_the_version(version: &TransactionOutput) -> Vec<ReferenceLeaf> {
    version
        .inode_leaf_containers
        .iter()
        .map(|container| ReferenceLeaf {
            identity: identity_of(&container.contents.identity),
            separator_key: container.contents.separator_key_in_the_parent_entry,
            inode_numbers: container
                .contents
                .records
                .iter()
                .map(|record| record.inode)
                .collect(),
        })
        .collect()
}

/// 这一版退役树里按叶序的全部退役记录：(被退役的身份, 死亡 txg)。
fn retirement_records_of(version: &TransactionOutput) -> Vec<(ReferenceLeafIdentity, u64)> {
    version
        .retirement_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .map(|record| (identity_of(&record.retired), record.death_txg.0))
        .collect()
}

/// 一次用户改动之后：发布路径那一版的每片叶（身份、分隔 key、记录号）与参照逐片相同；这次改动追加的退役记录（退役树只追加）与参照
/// 这次身份消失的那几片逐个相同（身份按集合比、条数也比），死亡 txg 都是这一版的 txg。
fn assert_the_version_matches_the_reference(
    what_changed: &str,
    retirement_records_before: &[(ReferenceLeafIdentity, u64)],
    version: &TransactionOutput,
    reference: &mut ReferenceLeafShape,
) {
    let leaves = leaves_of_the_version(version);
    assert_eq!(
        leaves
            .iter()
            .map(|leaf| leaf.inode_numbers.len())
            .collect::<Vec<_>>(),
        reference.record_counts(),
        "{what_changed}：每片叶的记录数与参照不同"
    );
    assert_eq!(
        leaves
            .iter()
            .map(|leaf| leaf.separator_key)
            .collect::<Vec<_>>(),
        reference
            .leaves
            .iter()
            .map(|leaf| leaf.separator_key)
            .collect::<Vec<_>>(),
        "{what_changed}：每片叶的分隔 key 与参照不同"
    );
    assert_eq!(
        leaves, reference.leaves,
        "{what_changed}：叶的身份或记录号与参照不同"
    );
    let retirement_records_after = retirement_records_of(version);
    assert!(
        retirement_records_after.starts_with(retirement_records_before),
        "{what_changed}：退役树只追加"
    );
    let appended = &retirement_records_after[retirement_records_before.len()..];
    let mut retired_by_the_publish_path: Vec<ReferenceLeafIdentity> =
        appended.iter().map(|(identity, _)| *identity).collect();
    let mut retired_by_the_reference = std::mem::take(&mut reference.retired_in_this_change);
    retired_by_the_publish_path.sort_unstable();
    retired_by_the_reference.sort_unstable();
    assert_eq!(
        retired_by_the_publish_path, retired_by_the_reference,
        "{what_changed}：这次退役的身份集与参照不同"
    );
    let txg = version.root.checkpoint_txg.0;
    assert!(
        appended.iter().all(|(_, death_txg)| *death_txg == txg),
        "{what_changed}：这次追加的退役记录死亡 txg 都是这一版的 txg {txg}：{appended:?}"
    );
}

/// 建一批 `count` 个空 inode（号从水位起），参照照条款追加，再比。
fn create_and_compare(pool: &mut BuiltPool, count: u64, reference: &mut ReferenceLeafShape) {
    let retirement_records_before = retirement_records_of(&pool.output);
    let first_new_inode = pool.output.inode_number_watermark();
    let version = publish_new_inodes_and_advance_the_pool(pool, count);
    let txg = version.root.checkpoint_txg.0;
    // 迭代上界：这一批的条数。
    for inode in first_new_inode..first_new_inode + count {
        reference.append(inode, txg);
    }
    assert_the_version_matches_the_reference(
        &format!("建 {count} 个（{first_new_inode} 起，txg {txg}）"),
        &retirement_records_before,
        &version,
        reference,
    );
}

/// unlink `inode` 到底，参照在最后一批删掉那条记录，再比最后一批那一版。
fn unlink_and_compare(pool: &mut BuiltPool, inode: u64, reference: &mut ReferenceLeafShape) {
    let retirement_records_before = retirement_records_of(&pool.output);
    let counts_before = reference.record_counts();
    unlink_to_the_end(pool, inode);
    reference.remove(inode);
    assert_the_version_matches_the_reference(
        &format!(
            "unlink {inode}（删之前各片 {counts_before:?}，最后一批 txg {}）",
            pool.output.root.checkpoint_txg.0
        ),
        &retirement_records_before,
        &pool.output,
        reference,
    );
}

#[test]
fn every_publish_of_a_random_history_leaves_the_leaf_shape_and_the_retired_leaves_of_the_independent_reference(
) {
    let mut pool = build_pool("inode-leaf-merge-and-borrow-against-an-independent-reference");
    let mut random = SeededRandomSource::from_seed(SEED_OF_THE_HISTORY);
    let mut reference = ReferenceLeafShape::of_the_version(&pool.output);

    // 一、建到 [233, 233, ≥ 117]，删可删条数最多的那片，直到两对都装得下（先右）。
    let mut left_to_create = NEW_INODES_AT_LEAST + random.below(NEW_INODES_ABOVE_THE_LEAST_BELOW);
    // 迭代上界：每一批至少建 1 个，left_to_create 至多 598。
    while left_to_create > 0 {
        let count = (1 + random.below(NEW_INODES_PER_BATCH_AT_MOST)).min(left_to_create);
        create_and_compare(&mut pool, count, &mut reference);
        left_to_create -= count;
    }
    let mut unlinks_until_both_pairs_fit = 0;
    // 迭代上界：UNLINKS_UNTIL_BOTH_PAIRS_FIT_AT_MOST 次；跨轮携带的是参照与池，每轮各删一个号。
    while reference.times_both_neighbour_pairs_fit == 0 {
        assert!(
            unlinks_until_both_pairs_fit < UNLINKS_UNTIL_BOTH_PAIRS_FIT_AT_MOST,
            "删了 {unlinks_until_both_pairs_fit} 次还没走到「两对都装得下」那一形：各片 {:?}",
            reference.record_counts()
        );
        let position = reference.position_of_the_leaf_with_the_most_removable_records();
        let inode = reference.random_removable_inode_of_the_leaf(position, &mut random);
        unlink_and_compare(&mut pool, inode, &mut reference);
        unlinks_until_both_pairs_fit += 1;
    }

    // 二、最右那片后面新开一片 1–8 条，删最大的活号直到它删空摘掉。
    let records_of_the_leaf_to_empty =
        1 + random.below(RECORDS_OF_THE_LEAF_EMPTIED_BY_UNLINKING_THE_LARGEST_AT_MOST);
    let free_slots_of_the_rightmost_leaf =
        leaf_capacity() - reference.records_at(reference.leaves.len() - 1);
    create_and_compare(
        &mut pool,
        u64::try_from(free_slots_of_the_rightmost_leaf).expect("至多 233")
            + records_of_the_leaf_to_empty,
        &mut reference,
    );
    let mut unlinks_of_the_largest_until_a_drop = 0;
    // 迭代上界：records_of_the_leaf_to_empty 次；每轮删最右那片的最后一条。
    while reference.times_an_emptied_leaf_was_dropped == 0 {
        assert!(
            unlinks_of_the_largest_until_a_drop < records_of_the_leaf_to_empty,
            "删了 {unlinks_of_the_largest_until_a_drop} 次最大的活号，{records_of_the_leaf_to_empty} 条的那一片还没删空：各片 {:?}",
            reference.record_counts()
        );
        let largest_live_inode = reference.largest_live_inode();
        unlink_and_compare(&mut pool, largest_live_inode, &mut reference);
        unlinks_of_the_largest_until_a_drop += 1;
    }

    // 三、[117, 233] 后面新开第三片 1–58 条，删第一片的一条：借 58 条，借出的那一片再吸收第三片。
    assert_eq!(
        reference.record_counts(),
        vec![borrow_floor(), leaf_capacity()],
        "第三段从 [117, 233] 起：先右吸收之后、新开的那一片删空摘掉之后"
    );
    let records_of_the_leaf_the_lender_absorbs =
        1 + random.below(RECORDS_OF_THE_LEAF_THE_LENDER_ABSORBS_AT_MOST);
    create_and_compare(
        &mut pool,
        records_of_the_leaf_the_lender_absorbs,
        &mut reference,
    );
    let inode_unlinked_from_the_first_leaf =
        reference.random_removable_inode_of_the_leaf(0, &mut random);
    unlink_and_compare(
        &mut pool,
        inode_unlinked_from_the_first_leaf,
        &mut reference,
    );
    assert_eq!(
        (
            reference.times_a_leaf_borrowed_from_its_right_neighbour,
            reference.times_the_lender_absorbed_after_lending
        ),
        (1, 1),
        "第一片 116 条从第二片借 58 条，第二片剩 175 条再吸收第三片 {records_of_the_leaf_the_lender_absorbs} 条：各片 {:?}",
        reference.record_counts()
    );

    // 四、随机 unlink 几个活着的号。
    // 迭代上界：常量 RANDOM_UNLINKS_BEFORE_ONLY_THE_LEFT_PAIR_FITS。
    for _ in 0..RANDOM_UNLINKS_BEFORE_ONLY_THE_LEFT_PAIR_FITS {
        let live = reference.live_removable_inodes();
        let pick = usize::try_from(random.below(u64::try_from(live.len()).expect("装得进 u64")))
            .expect("小于活着的号数");
        unlink_and_compare(&mut pool, live[pick], &mut reference);
    }

    // 五、删最大的活号，直到最右那片与左邻之和恰好 233、只有左邻那一对装得下。
    let rightmost_position = reference.leaves.len() - 1;
    assert!(
        rightmost_position > 0 && reference.records_at(rightmost_position - 1) < leaf_capacity(),
        "第五段要最右那片有一个没装满的左邻（左邻满着就删到最右那片删空、走不到左吸收）：各片 {:?}",
        reference.record_counts()
    );
    let mut unlinks_of_the_largest_until_the_left_pair_fits = 0;
    // 迭代上界：UNLINKS_UNTIL_ONLY_THE_LEFT_PAIR_FITS_AT_MOST 次；每轮删最右那片的最后一条。
    while reference.times_only_the_left_neighbour_pair_fit == 0 {
        assert!(
            unlinks_of_the_largest_until_the_left_pair_fits
                < UNLINKS_UNTIL_ONLY_THE_LEFT_PAIR_FITS_AT_MOST,
            "删了 {unlinks_of_the_largest_until_the_left_pair_fits} 次最大的活号还没走到左吸收：各片 {:?}",
            reference.record_counts()
        );
        let largest_live_inode = reference.largest_live_inode();
        unlink_and_compare(&mut pool, largest_live_inode, &mut reference);
        unlinks_of_the_largest_until_the_left_pair_fits += 1;
    }

    assert!(
        reference.times_both_neighbour_pairs_fit >= 1,
        "两对都装得下（先右）至少走到一次"
    );
    assert!(
        reference.times_an_emptied_leaf_was_dropped >= 1,
        "删空摘片至少走到一次"
    );
    assert!(
        reference.times_a_leaf_borrowed_from_its_right_neighbour >= 1,
        "借至少走到一次"
    );
    assert!(
        reference.times_the_lender_absorbed_after_lending >= 1,
        "借后吸收至少走到一次"
    );
    assert!(
        reference.times_only_the_left_neighbour_pair_fit >= 1,
        "左吸收（只有左邻那一对装得下）至少走到一次"
    );
}
