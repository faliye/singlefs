//! 一片叶容器、合并与借记录的各种策略、一次发布里的改动与重写、整棵 inode 树、发布之后的结构核对与内容记法的上界。

use super::*;

/// 一片叶容器：容器号（首次插入它的那条记录的 inode 号，合并、借都不改）与按号升序的记录号。模型只存号，不存 140 字节记录。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeafContainer {
    pub container_number: u64,
    pub inode_numbers: Vec<u64>,
}

impl LeafContainer {
    #[must_use]
    pub fn smallest_inode_number(&self) -> u64 {
        *self.inode_numbers.first().expect("记录数为 0 的容器不留在树里（删空就摘）")
    }

    #[must_use]
    pub fn largest_inode_number(&self) -> u64 {
        *self.inode_numbers.last().expect("记录数为 0 的容器不留在树里（删空就摘）")
    }
}

/// M2、M3 删完一条之后先查哪一对（跑前登记第五节；主取样点先查右边，第二段扫先查左边）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeighbourCheckOrder {
    RightPairFirst,
    LeftPairFirst,
}

/// M3 借多少（跑前登记第五节；主取样点均分，第二段扫借到刚好够下限）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorrowAmount {
    HalfTheDifference,
    JustEnoughForTheFloor,
}

/// 三条臂在删除那一次发布里不同的那一步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MergePolicy {
    /// M1：删空即摘，别的什么都不做。
    DropEmptyOnly,
    /// M2：相邻两片之和 ≤ 233 就左吸收右，并到没有一对装得下为止。
    AbsorbRightWhenTheyFit { order: NeighbourCheckOrder },
    /// M3：先照 M2 并；这次一次都没并、删的那片没删空、它的记录数 < 下限、它有右邻时，从右邻借最小的几条。
    AbsorbRightAndBorrowFromRight { order: NeighbourCheckOrder, floor: usize, amount: BorrowAmount },
}

/// 叶容器数设不设 135 的上限：只有 C1 那一格不设（D8 已定项 6 末尾「9.98 倍」那一句说的是不设上限的树）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeafContainerLimit {
    OneRootNode,
    Unlimited,
}

/// 一次插入的结局。被拒时树不变（`crates/` 拒在任何落盘之前，`inode_tree.rs:258` 起）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsertOutcome {
    Accepted,
    RefusedMoreLeafContainersThanOneRootNodeHolds { containers_it_would_take: usize },
}

/// 一次删除（unlink 最后一批那一次发布）之后两种记法下重写了哪几片（这次发布之后的叶序位置，升序），以及这次摘了几片、并了几次、借了几次。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemovalRewrites {
    /// 内容记法：内容变了、且这次发布之后还在的叶。
    pub content_rewritten_positions: Vec<usize>,
    /// 位置记法（`crates/` 今天的记法）：内容记法那几片，加上第一个被摘掉的位置及其右边每一片留下的叶。
    pub position_rewritten_positions: Vec<usize>,
    pub containers_dropped: usize,
    pub merges: u64,
    pub borrows: u64,
}

impl RemovalRewrites {
    #[must_use]
    pub fn content_notation_count(&self) -> usize {
        self.content_rewritten_positions.len()
    }

    #[must_use]
    pub fn position_notation_count(&self) -> usize {
        self.position_rewritten_positions.len()
    }
}

/// 一次删除发布里攒着的改动：内容变了的容器号、每次摘片时它在当时叶序里的位置。
struct ChangesInThisPublish {
    changed_container_numbers: BTreeSet<u64>,
    dropped_positions_at_the_time_of_drop: Vec<usize>,
    pub(super) merges: u64,
    pub(super) borrows: u64,
}

/// 合并时查到的是哪一对：当前那片与它右邻，还是左邻与它。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PairToAbsorb {
    WithRightNeighbour,
    WithLeftNeighbour,
}

/// 一棵 inode 树的叶容器表（按叶序），连同它走哪条臂、设不设上限，以及累计并了几次、借了几次。
#[derive(Clone, Debug)]
pub struct InodeLeafTree {
    pub containers: Vec<LeafContainer>,
    pub policy: MergePolicy,
    pub limit: LeafContainerLimit,
    pub merges: u64,
    pub borrows: u64,
}

impl InodeLeafTree {
    #[must_use]
    pub fn empty(policy: MergePolicy, limit: LeafContainerLimit) -> Self {
        Self { containers: Vec::new(), policy, limit, merges: 0, borrows: 0 }
    }

    /// 从给定的叶容器表起一棵树（只给单测造 `crates/` 单测那几种形态用）。
    #[must_use]
    pub fn from_containers(policy: MergePolicy, limit: LeafContainerLimit, containers: Vec<LeafContainer>) -> Self {
        Self { containers, policy, limit, merges: 0, borrows: 0 }
    }

    #[must_use]
    pub fn live_inode_count(&self) -> u64 {
        self.containers.iter().map(|container| u64::try_from(container.inode_numbers.len()).expect("一片至多 233 条")).sum()
    }

    /// 建一个 inode：号要大于树里每一个 key（号从 1 起单调递增）；最右片不满就追加，满了就末尾分裂出一片新容器、容器号 = 这条的号
    /// （`crates/singlefs-core/src/inode_tree.rs:181` 起的落法）；新建之后片数超过 135 就拒，树不变。
    pub fn insert(&mut self, inode: u64) -> InsertOutcome {
        if let Some(rightmost) = self.containers.last_mut() {
            assert!(inode > rightmost.largest_inode_number(), "插入只落在最右叶：号 {inode} 要大于树里每一个 key");
            if rightmost.inode_numbers.len() < LEAF_RECORD_CAPACITY {
                rightmost.inode_numbers.push(inode);
                return InsertOutcome::Accepted;
            }
        }
        let containers_it_would_take = self.containers.len() + 1;
        let is_limited_to_one_root_node = match self.limit {
            LeafContainerLimit::OneRootNode => true,
            LeafContainerLimit::Unlimited => false,
        };
        if is_limited_to_one_root_node && containers_it_would_take > ROOT_NODE_LEAF_CONTAINER_CAPACITY {
            return InsertOutcome::RefusedMoreLeafContainersThanOneRootNodeHolds { containers_it_would_take };
        }
        self.containers.push(LeafContainer { container_number: inode, inode_numbers: vec![inode] });
        InsertOutcome::Accepted
    }

    /// 装着这个号的那片在叶序里的位置。三条臂上叶与叶的 key 区间都不交叠、沿叶序升序（丙类断言盯着），所以二分找得到。
    #[must_use]
    pub fn position_of_the_container_holding(&self, inode: u64) -> Option<usize> {
        let candidate = self.containers.partition_point(|container| container.largest_inode_number() < inode);
        let container = self.containers.get(candidate)?;
        container.inode_numbers.binary_search(&inode).ok().map(|_| candidate)
    }

    /// 容器号沿叶序严格递增（I-9.4 第二句，丙类断言盯着），所以二分找。
    fn position_of_container_number(&self, container_number: u64) -> usize {
        self.containers
            .binary_search_by_key(&container_number, |container| container.container_number)
            .expect("这次发布里点名的容器号还在树里：摘掉的容器号先从改动集合里拿掉")
    }

    fn drop_container(&mut self, position: usize, changes: &mut ChangesInThisPublish) -> LeafContainer {
        let dropped = self.containers.remove(position);
        changes.dropped_positions_at_the_time_of_drop.push(position);
        changes.changed_container_numbers.remove(&dropped.container_number);
        dropped
    }

    fn pair_fits_in_one_container(&self, left_position: usize) -> bool {
        self.containers[left_position].inode_numbers.len() + self.containers[left_position + 1].inode_numbers.len() <= LEAF_RECORD_CAPACITY
    }

    /// 左片吸收它右边那片：左片保留身份，记录接在后面，右片摘掉（退役）。
    fn absorb_right_neighbour(&mut self, left_position: usize, changes: &mut ChangesInThisPublish) {
        let absorbed = self.drop_container(left_position + 1, changes);
        let survivor = &mut self.containers[left_position];
        survivor.inode_numbers.extend(absorbed.inode_numbers);
        changes.changed_container_numbers.insert(survivor.container_number);
        changes.merges += 1;
        self.merges += 1;
    }

    /// 从容器号为 `start_container_number` 的那片起，按 `order` 查它与左右邻两对，和 ≤ 233 就并；并完从幸存那片接着查，直到没有一对装得下。
    /// 返回这一趟有没有并过。
    fn merge_until_no_pair_fits(&mut self, start_container_number: u64, order: NeighbourCheckOrder, changes: &mut ChangesInThisPublish) -> bool {
        let mut current_container_number = start_container_number;
        let mut merged_at_least_once = false;
        loop {
            let position = self.position_of_container_number(current_container_number);
            let right_fits = position + 1 < self.containers.len() && self.pair_fits_in_one_container(position);
            let left_fits = position > 0 && self.pair_fits_in_one_container(position - 1);
            let pair = match order {
                NeighbourCheckOrder::RightPairFirst if right_fits => Some(PairToAbsorb::WithRightNeighbour),
                NeighbourCheckOrder::RightPairFirst if left_fits => Some(PairToAbsorb::WithLeftNeighbour),
                NeighbourCheckOrder::LeftPairFirst if left_fits => Some(PairToAbsorb::WithLeftNeighbour),
                NeighbourCheckOrder::LeftPairFirst if right_fits => Some(PairToAbsorb::WithRightNeighbour),
                NeighbourCheckOrder::RightPairFirst | NeighbourCheckOrder::LeftPairFirst => None,
            };
            match pair {
                None => return merged_at_least_once,
                Some(PairToAbsorb::WithRightNeighbour) => self.absorb_right_neighbour(position, changes),
                Some(PairToAbsorb::WithLeftNeighbour) => {
                    self.absorb_right_neighbour(position - 1, changes);
                    current_container_number = self.containers[position - 1].container_number;
                }
            }
            merged_at_least_once = true;
        }
    }

    /// M3 的借：容器号为 `borrower_container_number` 的那片记录数 < 下限且有右邻时，把右邻最小的几条移到它末尾，两片都保留身份；
    /// 借完右邻变少，右邻与它右邻那一对照 M2 的规则查。它是最右那片时不借（右从左借违背 D8 已定项 6）。
    fn borrow_from_right_if_below_floor(
        &mut self,
        borrower_container_number: u64,
        order: NeighbourCheckOrder,
        floor: usize,
        amount: BorrowAmount,
        changes: &mut ChangesInThisPublish,
    ) {
        let position = self.position_of_container_number(borrower_container_number);
        if position + 1 >= self.containers.len() {
            return;
        }
        let records_in_the_borrower = self.containers[position].inode_numbers.len();
        if records_in_the_borrower >= floor {
            return;
        }
        let records_in_the_lender = self.containers[position + 1].inode_numbers.len();
        let moved_count = match amount {
            BorrowAmount::HalfTheDifference => (records_in_the_lender - records_in_the_borrower) / 2,
            BorrowAmount::JustEnoughForTheFloor => floor - records_in_the_borrower,
        };
        assert!(
            moved_count >= 1 && moved_count < records_in_the_lender,
            "借的条数要在 [1, 右邻记录数) 里：借方 {records_in_the_borrower} 条、右邻 {records_in_the_lender} 条、借 {moved_count} 条（相邻两片之和 ≥ 234 保证这一条）"
        );
        let moved_inode_numbers: Vec<u64> = self.containers[position + 1].inode_numbers.drain(..moved_count).collect();
        self.containers[position].inode_numbers.extend(moved_inode_numbers);
        let lender_container_number = self.containers[position + 1].container_number;
        changes.changed_container_numbers.insert(borrower_container_number);
        changes.changed_container_numbers.insert(lender_container_number);
        changes.borrows += 1;
        self.borrows += 1;
        self.merge_until_no_pair_fits(lender_container_number, order, changes);
    }

    /// 删一个 inode（unlink 最后一批那一次发布）：从装着它的那片里删掉；删空就摘（三条臂一样）。M2、M3 摘完之后查新相邻的那一对，
    /// 没删空就从那片起并；M3 这次一次都没并、那片没删空时再看借不借。
    pub fn remove(&mut self, inode: u64) -> RemovalRewrites {
        let holding_position = self.position_of_the_container_holding(inode).expect("删的号在树里：生成器只删它自己的活集合里的号");
        let mut changes = ChangesInThisPublish {
            changed_container_numbers: BTreeSet::new(),
            dropped_positions_at_the_time_of_drop: Vec::new(),
            merges: 0,
            borrows: 0,
        };
        let holding = &mut self.containers[holding_position];
        let position_in_container = holding.inode_numbers.binary_search(&inode).expect("上面按二分找到的就是装着它的那片");
        holding.inode_numbers.remove(position_in_container);
        let holding_container_number = holding.container_number;
        let holding_is_empty = holding.inode_numbers.is_empty();
        if holding_is_empty {
            self.drop_container(holding_position, &mut changes);
            let has_new_neighbours = holding_position > 0 && holding_position < self.containers.len();
            match self.policy {
                MergePolicy::DropEmptyOnly => {}
                MergePolicy::AbsorbRightWhenTheyFit { order } | MergePolicy::AbsorbRightAndBorrowFromRight { order, .. } => {
                    if has_new_neighbours {
                        let left_neighbour_container_number = self.containers[holding_position - 1].container_number;
                        self.merge_until_no_pair_fits(left_neighbour_container_number, order, &mut changes);
                    }
                }
            }
        } else {
            changes.changed_container_numbers.insert(holding_container_number);
            match self.policy {
                MergePolicy::DropEmptyOnly => {}
                MergePolicy::AbsorbRightWhenTheyFit { order } => {
                    self.merge_until_no_pair_fits(holding_container_number, order, &mut changes);
                }
                MergePolicy::AbsorbRightAndBorrowFromRight { order, floor, amount } => {
                    let merged = self.merge_until_no_pair_fits(holding_container_number, order, &mut changes);
                    if !merged {
                        self.borrow_from_right_if_below_floor(holding_container_number, order, floor, amount, &mut changes);
                    }
                }
            }
        }
        self.rewrites_of_this_publish(&changes)
    }

    fn rewrites_of_this_publish(&self, changes: &ChangesInThisPublish) -> RemovalRewrites {
        let content_rewritten: BTreeSet<usize> = changes
            .changed_container_numbers
            .iter()
            .map(|container_number| self.position_of_container_number(*container_number))
            .collect();
        let mut position_rewritten = content_rewritten.clone();
        if let Some(first_dropped_position) = changes.dropped_positions_at_the_time_of_drop.iter().min() {
            position_rewritten.extend(*first_dropped_position..self.containers.len());
        }
        RemovalRewrites {
            content_rewritten_positions: content_rewritten.into_iter().collect(),
            position_rewritten_positions: position_rewritten.into_iter().collect(),
            containers_dropped: changes.dropped_positions_at_the_time_of_drop.len(),
            merges: changes.merges,
            borrows: changes.borrows,
        }
    }

    /// I-9.4 第三句（左容器的最大 key < 右容器号）为假的相邻对数。M3 借过之后会为正（C536 那一句），M1、M2 上恒断言为 0。
    #[must_use]
    pub fn adjacent_pairs_where_left_largest_reaches_right_container_number(&self) -> u64 {
        let pairs = self.containers.windows(2).filter(|pair| pair[0].largest_inode_number() >= pair[1].container_number).count();
        u64::try_from(pairs).expect("至多 134 对")
    }

    /// 走的是不是会并的臂（M2、M3）。
    #[must_use]
    pub fn merges_adjacent_containers(&self) -> bool {
        match self.policy {
            MergePolicy::DropEmptyOnly => false,
            MergePolicy::AbsorbRightWhenTheyFit { .. } | MergePolicy::AbsorbRightAndBorrowFromRight { .. } => true,
        }
    }

    /// 走的是不是会借的臂（M3）。
    #[must_use]
    pub fn borrows_from_the_right(&self) -> bool {
        match self.policy {
            MergePolicy::DropEmptyOnly | MergePolicy::AbsorbRightWhenTheyFit { .. } => false,
            MergePolicy::AbsorbRightAndBorrowFromRight { .. } => true,
        }
    }
}

/// 丙类断言（跑前登记第七节）在一次发布之后：片内记录号严格升序（只查这次动过的那几片，没动的上次动它时查过）、
/// 每片 1 … 233 条、容器号 ≤ 片内最小号、沿叶序容器号严格递增、相邻两片左片最大号 < 右片最小号（三条臂都断言）；
/// I-9.4 第三句（左片最大号 < 右片容器号）在 M1、M2 上断言、在 M3 上只计数；M2、M3 任意相邻两片之和 ≥ 234。
/// 红了交回一句话（哪一条、哪一对、两片记录数），由调用方带上序列名与步号停下（第十节 F4）；绿了交回 I-9.4 第三句为假的对数。
pub fn check_structure_after_publish(tree: &InodeLeafTree, touched_positions: &[usize]) -> Result<u64, String> {
    for position in touched_positions {
        let Some(container) = tree.containers.get(*position) else { continue };
        if container.inode_numbers.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(format!("第 {position} 片（容器号 {}）片内记录号不严格升序", container.container_number));
        }
    }
    for (position, container) in tree.containers.iter().enumerate() {
        let records = container.inode_numbers.len();
        if records == 0 || records > LEAF_RECORD_CAPACITY {
            return Err(format!("第 {position} 片（容器号 {}）记录数 {records} 不在 1 … 233 里", container.container_number));
        }
        if container.container_number > container.smallest_inode_number() {
            return Err(format!("第 {position} 片容器号 {} 大于片内最小号 {}（I-9.4 第一句）", container.container_number, container.smallest_inode_number()));
        }
    }
    let mut third_sentence_false_pairs = 0_u64;
    for (left_position, pair) in tree.containers.windows(2).enumerate() {
        let (left, right) = (&pair[0], &pair[1]);
        if left.container_number >= right.container_number {
            return Err(format!("第 {left_position}、{} 片容器号 {} ≥ {}（I-9.4 第二句）", left_position + 1, left.container_number, right.container_number));
        }
        if left.largest_inode_number() >= right.smallest_inode_number() {
            return Err(format!(
                "第 {left_position}、{} 片 key 区间交叠：左片最大号 {} ≥ 右片最小号 {}",
                left_position + 1,
                left.largest_inode_number(),
                right.smallest_inode_number()
            ));
        }
        if left.largest_inode_number() >= right.container_number {
            third_sentence_false_pairs += 1;
            if !tree.borrows_from_the_right() {
                return Err(format!(
                    "第 {left_position}、{} 片左片最大号 {} ≥ 右片容器号 {}（I-9.4 第三句，只许在 M3 上为假）",
                    left_position + 1,
                    left.largest_inode_number(),
                    right.container_number
                ));
            }
        }
        if tree.merges_adjacent_containers() && left.inode_numbers.len() + right.inode_numbers.len() < LEAF_RECORD_CAPACITY + 1 {
            return Err(format!(
                "相邻两片之和 ≥ 234 不成立：第 {left_position}、{} 片记录数 {} + {}",
                left_position + 1,
                left.inode_numbers.len(),
                right.inode_numbers.len()
            ));
        }
    }
    Ok(third_sentence_false_pairs)
}

/// 内容记法下每次删除重写片数的上界（丙类，第四节推的）：M1、M2 ≤ 1，M3 ≤ 2。
#[must_use]
pub fn content_notation_bound_per_removal(tree: &InodeLeafTree) -> usize {
    if tree.borrows_from_the_right() {
        2
    } else {
        1
    }
}

#[cfg(test)]
#[path = "leaf_containers_and_the_tree_tests.rs"]
mod tests;
