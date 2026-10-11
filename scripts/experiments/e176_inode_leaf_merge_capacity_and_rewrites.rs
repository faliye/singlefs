//! E176（inode 树叶合并三候选的容量与写代价）：跑前登记 `research/prompts/e176-preregistration.md`。
//!
//! 独立手写模型，不与 `crates/` 共用代码（`implementation-first.md` 第 4 条）。三条臂：
//! M1 只删空即摘（`crates/` 今天）、M2 只做左吸收右（相邻两片合起来装得下就并）、M3 左吸收右加左从右借。
//! M1 那一臂另由 `e176_inode_leaf_merge_crates_oracle.rs`（特性 `e176-crates-oracle`）逐步对拍 `crates/` 的两个纯函数；
//! 那个 bin 以 `#[path]` 把这份文件当模块引进去，所以对拍 bin 用到的项都写成 `pub`。
//!
//! 用法：`e176-inode-leaf-merge-capacity-and-rewrites segment-one`（第一段）或 `segment-two`（第二段，登记第五节与修订 3）；线程数读 `SINGLEFS_THREAD_CAP`（`scripts/hosts/capped.sh` 设），没设取 1。
//! 产物是 `E7RESULT` 行，收尾 `E7RESULT name=done emitted=N`（N 把 config 行与 done 行都算进去）。
//! 确定性模型：跑两遍逐字节相同只说明没有隐藏状态，不是统计上稳定。

use std::collections::BTreeSet;
use std::fmt::Write as _;

/// 一片叶至多几条记录：D8（核心索引结构） 已定项 6「一叶 233 条 140 字节记录」。
/// 值抄自 `crates/singlefs-format/src/lib.rs:110` 的 `INODE_LEAF_RECORDS`，对拍 bin 拿它回比 `crates/`。
pub const LEAF_RECORD_CAPACITY: usize = 233;
/// 一个码 2 根至多几条叶条目：D8（核心索引结构） 已定项 6「扇出 135」「⌊(16384 − 131) / 120⌋ = 135」。
/// 值抄自 `crates/singlefs-core/src/inode_tree.rs:109` 的 `leaf_containers_one_root_node_holds()`，对拍 bin 拿它回比 `crates/`。
pub const ROOT_NODE_LEAF_CONTAINER_CAPACITY: usize = 135;
/// 每条序列每条臂最多吃几个操作（跑前登记第六节）：到了还没被拒就报「未撞到」。
pub const OPERATIONS_PER_ARM_AT_MOST: u64 = 20_000_000;
/// 公共窗口里的最后一批少于这个次数，Q2、Q3 两行只报数、标「样本不足」（跑前登记第六节）。
pub const UNLINKS_IN_THE_WINDOW_FOR_A_JUDGEMENT_AT_LEAST: u64 = 100;
/// M3 主取样点的下限 f = ⌈233 ÷ 2⌉ = 117；方向相反的取样点 59 = ⌈233 ÷ 4⌉（跑前登记第五、八节）。
pub const BORROW_FLOOR_MAIN: usize = 117;
pub const BORROW_FLOOR_SMALL: usize = 59;

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
    merges: u64,
    borrows: u64,
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

/// 逐块喂的 SHA-256（FIPS 180-4），给操作流前缀算指纹；单测拿公开测试向量钉住。
#[derive(Clone, Debug)]
pub struct StreamingSha256 {
    hash_words: [u32; 8],
    pending_block: [u8; 64],
    pending_bytes: usize,
    total_bytes: u64,
}

const SHA256_INITIAL_HASH: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];

const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

impl StreamingSha256 {
    #[must_use]
    pub fn new() -> Self {
        Self { hash_words: SHA256_INITIAL_HASH, pending_block: [0; 64], pending_bytes: 0, total_bytes: 0 }
    }

    pub fn update(&mut self, bytes: &[u8]) {
        self.total_bytes += u64::try_from(bytes.len()).expect("一次喂的字节数装得进 u64");
        for byte in bytes {
            self.pending_block[self.pending_bytes] = *byte;
            self.pending_bytes += 1;
            if self.pending_bytes == 64 {
                let block = self.pending_block;
                self.compress(&block);
                self.pending_bytes = 0;
            }
        }
    }

    fn compress(&mut self, block: &[u8; 64]) {
        let mut schedule = [0u32; 64];
        for (word_index, word) in schedule.iter_mut().take(16).enumerate() {
            *word = u32::from_be_bytes(block[word_index * 4..word_index * 4 + 4].try_into().expect("切了 4 字节"));
        }
        for word_index in 16..64 {
            let sigma_zero = schedule[word_index - 15].rotate_right(7) ^ schedule[word_index - 15].rotate_right(18) ^ (schedule[word_index - 15] >> 3);
            let sigma_one = schedule[word_index - 2].rotate_right(17) ^ schedule[word_index - 2].rotate_right(19) ^ (schedule[word_index - 2] >> 10);
            schedule[word_index] = schedule[word_index - 16].wrapping_add(sigma_zero).wrapping_add(schedule[word_index - 7]).wrapping_add(sigma_one);
        }
        let mut registers = self.hash_words;
        for round in 0..64 {
            let big_sigma_one = registers[4].rotate_right(6) ^ registers[4].rotate_right(11) ^ registers[4].rotate_right(25);
            let choose = (registers[4] & registers[5]) ^ ((!registers[4]) & registers[6]);
            let temporary_one = registers[7]
                .wrapping_add(big_sigma_one)
                .wrapping_add(choose)
                .wrapping_add(SHA256_ROUND_CONSTANTS[round])
                .wrapping_add(schedule[round]);
            let big_sigma_zero = registers[0].rotate_right(2) ^ registers[0].rotate_right(13) ^ registers[0].rotate_right(22);
            let majority = (registers[0] & registers[1]) ^ (registers[0] & registers[2]) ^ (registers[1] & registers[2]);
            let temporary_two = big_sigma_zero.wrapping_add(majority);
            registers = [
                temporary_one.wrapping_add(temporary_two),
                registers[0],
                registers[1],
                registers[2],
                registers[3].wrapping_add(temporary_one),
                registers[4],
                registers[5],
                registers[6],
            ];
        }
        for (slot, value) in self.hash_words.iter_mut().zip(registers) {
            *slot = slot.wrapping_add(value);
        }
    }

    /// 收尾（补 0x80、零、位长），交回十六进制摘要。拿的是一份拷贝，原状态可以接着喂。
    #[must_use]
    pub fn hexadecimal_digest(&self) -> String {
        let mut finishing = self.clone();
        let bit_length = self.total_bytes * 8;
        finishing.update(&[0x80]);
        while finishing.pending_bytes != 56 {
            finishing.update(&[0]);
        }
        finishing.update(&bit_length.to_be_bytes());
        assert_eq!(finishing.pending_bytes, 0, "补齐之后正好一整块");
        finishing.hash_words.iter().fold(String::new(), |mut text, word| {
            write!(text, "{word:08x}").expect("写进 String 不会失败");
            text
        })
    }
}

impl Default for StreamingSha256 {
    fn default() -> Self {
        Self::new()
    }
}

/// 负载序列里的一个操作：建一个 inode（写一条记录），或 unlink 最后一批（删一条记录）。一次发布一个操作。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Create(u64),
    Unlink(u64),
}

impl Operation {
    /// 进操作流指纹的 9 字节：一个标记字节加小端 8 字节号。
    #[must_use]
    pub fn fingerprint_bytes(self) -> [u8; 9] {
        let (marker, inode) = match self {
            Operation::Create(inode) => (b'C', inode),
            Operation::Unlink(inode) => (b'U', inode),
        };
        let mut bytes = [0u8; 9];
        bytes[0] = marker;
        bytes[1..].copy_from_slice(&inode.to_le_bytes());
        bytes
    }
}

/// splitmix64 伪随机源，种子即初值（跑前登记第六节 W-R 那一行）。
#[derive(Clone, Debug)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    #[must_use]
    pub fn seeded(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_value(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }
}

/// W-S 每轮删的次序：升序删最小的（最老的），降序删最大的（最新的）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeletionOrder {
    SmallestFirst,
    LargestFirst,
}

/// W-A 每批留下哪些：留 1/s 的删 (n − b) mod s ≠ 0 的；留 3/4 的删 (n − b) mod 4 = 3 的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeptInEachBatch {
    OneIn(u64),
    ThreeInFour,
}

/// 三族负载（跑前登记第六节「负载序列」那张表）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Workload {
    /// W-R：每一步取一个 53 位均匀数 u，活集合为空或 u < p 就插入，否则删活集合（按插入次序追加的 Vec）第 next() mod 长度 个、swap_remove 取出。
    /// p 写成百分数，比较按整数做：u × 100 < p_percent × 2^53。
    RandomInsertDelete { insert_probability_percent: u64 },
    /// W-S：一轮插入 A 条，再按次序删活集合里最小（或最大）的 D 个。
    SequentialDelete { inserted_per_round: u64, deleted_per_round: u64, order: DeletionOrder },
    /// W-A：一轮插入 A 条连号 b … b + A − 1，再按号升序删掉这一批里的一部分。
    AlternateDelete { batch_size: u64, kept: KeptInEachBatch },
}

/// 一个登记的取样点：名字、负载、种子（只 W-R 用）、是不是主取样点。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegisteredSequence {
    pub name: &'static str,
    pub family: &'static str,
    pub workload: Workload,
    pub seed: u64,
    pub is_main_sampling_point: bool,
}

/// 第一段的 22 条序列，次序就是产物里的次序。
#[must_use]
pub fn registered_sequences() -> Vec<RegisteredSequence> {
    let mut sequences = Vec::new();
    let random_points: [(u64, &'static [&'static str; 5], bool); 3] = [
        (55, &["wr_p055_seed1", "wr_p055_seed2", "wr_p055_seed3", "wr_p055_seed4", "wr_p055_seed5"], true),
        (75, &["wr_p075_seed1", "wr_p075_seed2", "wr_p075_seed3", "wr_p075_seed4", "wr_p075_seed5"], false),
        (95, &["wr_p095_seed1", "wr_p095_seed2", "wr_p095_seed3", "wr_p095_seed4", "wr_p095_seed5"], false),
    ];
    for (insert_probability_percent, names, is_main_sampling_point) in random_points {
        for (seed_offset, name) in names.iter().enumerate() {
            sequences.push(RegisteredSequence {
                name,
                family: "W-R",
                workload: Workload::RandomInsertDelete { insert_probability_percent },
                seed: u64::try_from(seed_offset).expect("五个种子") + 1,
                is_main_sampling_point,
            });
        }
    }
    let sequential_points = [
        ("ws_a1000_d500_smallest_first", 500, DeletionOrder::SmallestFirst, true),
        ("ws_a1000_d950_smallest_first", 950, DeletionOrder::SmallestFirst, false),
        ("ws_a1000_d500_largest_first", 500, DeletionOrder::LargestFirst, false),
    ];
    for (name, deleted_per_round, order, is_main_sampling_point) in sequential_points {
        sequences.push(RegisteredSequence {
            name,
            family: "W-S",
            workload: Workload::SequentialDelete { inserted_per_round: 1000, deleted_per_round, order },
            seed: 0,
            is_main_sampling_point,
        });
    }
    let alternate_points = [
        ("wa_keep1of2_a1000", 1000, KeptInEachBatch::OneIn(2), true),
        ("wa_keep1of2_a2330", 2330, KeptInEachBatch::OneIn(2), false),
        ("wa_keep1of233_a2330", 2330, KeptInEachBatch::OneIn(233), false),
        ("wa_keep3of4_a1000", 1000, KeptInEachBatch::ThreeInFour, false),
    ];
    for (name, batch_size, kept, is_main_sampling_point) in alternate_points {
        sequences.push(RegisteredSequence {
            name,
            family: "W-A",
            workload: Workload::AlternateDelete { batch_size, kept },
            seed: 0,
            is_main_sampling_point,
        });
    }
    sequences
}

/// W-S、W-A 一轮里走到哪一步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RoundPhase {
    Creating { remaining: u64 },
    Deleting { remaining: u64 },
    DeletingFromBatch { batch_start: u64, next_offset: u64 },
}

/// 序列生成器：自己维护活 inode 集合，不看任何一条臂的状态；号从 1 起单调递增。
#[derive(Clone, Debug)]
pub struct WorkloadGenerator {
    workload: Workload,
    random: SplitMix64,
    next_inode: u64,
    live_in_creation_order: Vec<u64>,
    live_ascending: std::collections::VecDeque<u64>,
    phase: RoundPhase,
}

impl WorkloadGenerator {
    #[must_use]
    pub fn new(workload: Workload, seed: u64) -> Self {
        let phase = match workload {
            Workload::RandomInsertDelete { .. } => RoundPhase::Creating { remaining: 0 },
            Workload::SequentialDelete { inserted_per_round, .. } => RoundPhase::Creating { remaining: inserted_per_round },
            Workload::AlternateDelete { batch_size, .. } => RoundPhase::Creating { remaining: batch_size },
        };
        Self {
            workload,
            random: SplitMix64::seeded(seed),
            next_inode: 1,
            live_in_creation_order: Vec::new(),
            live_ascending: std::collections::VecDeque::new(),
            phase,
        }
    }

    fn create(&mut self) -> Operation {
        let inode = self.next_inode;
        self.next_inode += 1;
        Operation::Create(inode)
    }

    pub fn next_operation(&mut self) -> Operation {
        match self.workload {
            Workload::RandomInsertDelete { insert_probability_percent } => {
                let uniform_53_bits = self.random.next_value() >> 11;
                let inserts = self.live_in_creation_order.is_empty()
                    || u128::from(uniform_53_bits) * 100 < u128::from(insert_probability_percent) << 53;
                if inserts {
                    let operation = self.create();
                    let Operation::Create(inode) = operation else { unreachable!("create 只交回 Create") };
                    self.live_in_creation_order.push(inode);
                    operation
                } else {
                    let live_count = u64::try_from(self.live_in_creation_order.len()).expect("活数装得进 u64");
                    let pick = usize::try_from(self.random.next_value() % live_count).expect("下标小于活数");
                    Operation::Unlink(self.live_in_creation_order.swap_remove(pick))
                }
            }
            Workload::SequentialDelete { inserted_per_round, deleted_per_round, order } => loop {
                match self.phase {
                    RoundPhase::Creating { remaining: 0 } => self.phase = RoundPhase::Deleting { remaining: deleted_per_round },
                    RoundPhase::Deleting { remaining: 0 } => self.phase = RoundPhase::Creating { remaining: inserted_per_round },
                    RoundPhase::Creating { remaining } => {
                        self.phase = RoundPhase::Creating { remaining: remaining - 1 };
                        let operation = self.create();
                        let Operation::Create(inode) = operation else { unreachable!("create 只交回 Create") };
                        self.live_ascending.push_back(inode);
                        return operation;
                    }
                    RoundPhase::Deleting { remaining } => {
                        self.phase = RoundPhase::Deleting { remaining: remaining - 1 };
                        let inode = match order {
                            DeletionOrder::SmallestFirst => self.live_ascending.pop_front(),
                            DeletionOrder::LargestFirst => self.live_ascending.pop_back(),
                        };
                        return Operation::Unlink(inode.expect("每轮先插 A 条再删 D < A 条，活集合不空"));
                    }
                    RoundPhase::DeletingFromBatch { .. } => unreachable!("W-S 不走按批删"),
                }
            },
            Workload::AlternateDelete { batch_size, kept } => loop {
                match self.phase {
                    RoundPhase::Creating { remaining: 0 } => {
                        self.phase = RoundPhase::DeletingFromBatch { batch_start: self.next_inode - batch_size, next_offset: 0 };
                    }
                    RoundPhase::Creating { remaining } => {
                        self.phase = RoundPhase::Creating { remaining: remaining - 1 };
                        return self.create();
                    }
                    RoundPhase::DeletingFromBatch { batch_start, next_offset } => {
                        let deleted_offset = (next_offset..batch_size).find(|offset| match kept {
                            KeptInEachBatch::OneIn(kept_one_in) => offset % kept_one_in != 0,
                            KeptInEachBatch::ThreeInFour => offset % 4 == 3,
                        });
                        match deleted_offset {
                            Some(offset) => {
                                self.phase = RoundPhase::DeletingFromBatch { batch_start, next_offset: offset + 1 };
                                return Operation::Unlink(batch_start + offset);
                            }
                            None => self.phase = RoundPhase::Creating { remaining: batch_size },
                        }
                    }
                    RoundPhase::Deleting { .. } => unreachable!("W-A 不走按次序删"),
                }
            },
        }
    }
}

/// 一条臂的变体：名字（产物里的臂字段）与它的合并策略。第一段跑 M1、M2 主、M3 主、M3 f = 59 四个；第二段跑 M1、M2 主、M3 主与另三个变体。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArmVariant {
    pub name: &'static str,
    pub policy: MergePolicy,
}

pub const ARM_M1: ArmVariant = ArmVariant { name: "M1", policy: MergePolicy::DropEmptyOnly };
pub const ARM_M2_MAIN: ArmVariant =
    ArmVariant { name: "M2", policy: MergePolicy::AbsorbRightWhenTheyFit { order: NeighbourCheckOrder::RightPairFirst } };
pub const ARM_M3_MAIN: ArmVariant = ArmVariant {
    name: "M3",
    policy: MergePolicy::AbsorbRightAndBorrowFromRight {
        order: NeighbourCheckOrder::RightPairFirst,
        floor: BORROW_FLOOR_MAIN,
        amount: BorrowAmount::HalfTheDifference,
    },
};
pub const ARM_M3_FLOOR_59: ArmVariant = ArmVariant {
    name: "M3_floor59",
    policy: MergePolicy::AbsorbRightAndBorrowFromRight {
        order: NeighbourCheckOrder::RightPairFirst,
        floor: BORROW_FLOOR_SMALL,
        amount: BorrowAmount::HalfTheDifference,
    },
};
pub const ARM_M3_JUST_ENOUGH_FOR_THE_FLOOR: ArmVariant = ArmVariant {
    name: "M3_just_enough",
    policy: MergePolicy::AbsorbRightAndBorrowFromRight {
        order: NeighbourCheckOrder::RightPairFirst,
        floor: BORROW_FLOOR_MAIN,
        amount: BorrowAmount::JustEnoughForTheFloor,
    },
};
pub const ARM_M2_LEFT_PAIR_FIRST: ArmVariant =
    ArmVariant { name: "M2_left_first", policy: MergePolicy::AbsorbRightWhenTheyFit { order: NeighbourCheckOrder::LeftPairFirst } };
/// 第二段「先查哪一对」那个旋钮在 M3 上的取值：下限与借多少取主旋钮，只把先查的那一对换成左边（跑前登记第五节、第八节）。
pub const ARM_M3_LEFT_PAIR_FIRST: ArmVariant = ArmVariant {
    name: "M3_left_first",
    policy: MergePolicy::AbsorbRightAndBorrowFromRight {
        order: NeighbourCheckOrder::LeftPairFirst,
        floor: BORROW_FLOOR_MAIN,
        amount: BorrowAmount::HalfTheDifference,
    },
};

/// 第一段的四个臂变体，次序就是产物里的次序；下标 0 是 M1。
pub const SEGMENT_ONE_ARMS: [ArmVariant; 4] = [ARM_M1, ARM_M2_MAIN, ARM_M3_MAIN, ARM_M3_FLOOR_59];
/// 第二段的六个臂变体：主窗口那三条（M1、M2 主、M3 主）当参照，加登记第五节第二段那三个变体；次序就是产物里的次序，下标 0 是 M1、1 是 M2 主、2 是 M3 主。
pub const SEGMENT_TWO_ARMS: [ArmVariant; 6] =
    [ARM_M1, ARM_M2_MAIN, ARM_M3_MAIN, ARM_M3_JUST_ENOUGH_FOR_THE_FLOOR, ARM_M2_LEFT_PAIR_FIRST, ARM_M3_LEFT_PAIR_FIRST];

/// 一条臂在一条序列上为什么停下。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArmStop {
    /// 第 `refused_operation_number` 个操作（从 1 数）是一次被拒的插入（叶容器数会超过 135）。
    Refused { refused_operation_number: u64 },
    /// 吃满 2 × 10⁷ 个操作还没被拒：「未撞到」。
    ReachedOperationCap,
}

/// 一条臂在一条序列上的轨迹（第八节），到它自己停下为止。填充率存成 (活数, 片数 × 233) 两个整数，比较按交叉相乘。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArmTrajectory {
    pub arm: &'static str,
    pub stop: ArmStop,
    pub operations_accepted: u64,
    pub live_inodes_at_stop: u64,
    pub containers_peak: usize,
    pub publishes_at_root_capacity: u64,
    pub containers_final: usize,
    pub fill_smallest: (u64, u64),
    pub fill_final: (u64, u64),
    pub merges: u64,
    pub borrows: u64,
    pub third_sentence_false_peak: u64,
    pub third_sentence_false_positive_publishes: u64,
    pub third_sentence_false_final: u64,
    pub consumed_prefix_sha256: String,
}

/// 公共窗口里一条臂每次 unlink 的重写片数：两种记法的和、单次最大、分布（0、1、2、3–9、10–134 各几次）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WindowRewrites {
    pub content_sum: u64,
    pub position_sum: u64,
    pub content_largest: usize,
    pub position_largest: usize,
    pub content_histogram: [u64; 5],
    pub position_histogram: [u64; 5],
}

fn histogram_bucket(rewritten: usize) -> usize {
    match rewritten {
        0 => 0,
        1 => 1,
        2 => 2,
        3..=9 => 3,
        10..=134 => 4,
        _ => panic!("一次发布重写 {rewritten} 片：摘过片之后至多 134 片，没摘时内容记法至多 2 片"),
    }
}

/// 累计重写差 S(t) = Σ(臂 − M1)（第八节），t 走公共窗口里的最后一批：峰值、单次差 > 0 的次数、期末值。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CumulativeDifference {
    pub running: i64,
    pub peak: i64,
    pub positive_single_differences: u64,
}

impl CumulativeDifference {
    fn add(&mut self, single_difference: i64) {
        self.running += single_difference;
        self.peak = self.peak.max(self.running);
        if single_difference > 0 {
            self.positive_single_differences += 1;
        }
    }
}

/// 一个公共窗口：三条臂（下标指进这条序列的臂表，第 0 个是 M1）从头到三条里最早一次不接受操作之前。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComparisonWindow {
    pub label: &'static str,
    pub arm_indices: [usize; 3],
    pub is_open: bool,
    pub operations: u64,
    pub unlinks: u64,
    pub ended_by_arm: Option<&'static str>,
    pub prefix_sha256: [String; 3],
    pub operations_identical: bool,
    pub rewrites: [WindowRewrites; 3],
    /// [第 1 条臂 − M1 位置记法, 第 1 条 − M1 内容记法, 第 2 条 − M1 位置记法, 第 2 条 − M1 内容记法]
    pub cumulative: [CumulativeDifference; 4],
}

impl ComparisonWindow {
    fn opened(label: &'static str, arm_indices: [usize; 3]) -> Self {
        Self {
            label,
            arm_indices,
            is_open: true,
            operations: 0,
            unlinks: 0,
            ended_by_arm: None,
            prefix_sha256: [String::new(), String::new(), String::new()],
            operations_identical: true,
            rewrites: [WindowRewrites::default(), WindowRewrites::default(), WindowRewrites::default()],
            cumulative: [CumulativeDifference::default(); 4],
        }
    }
}

/// 一条序列跑完的全部结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SequenceResult {
    pub sequence: RegisteredSequence,
    pub trajectories: Vec<ArmTrajectory>,
    pub windows: Vec<ComparisonWindow>,
}

/// 一条臂在一步里吃到了什么。
#[derive(Clone, Debug)]
enum StepOutcome {
    NotTaken,
    Refused(Operation),
    Created(Operation),
    Unlinked(Operation, usize, usize),
}

/// 一条臂跑的时候带着的状态。
struct ArmRun {
    variant: ArmVariant,
    tree: InodeLeafTree,
    generator: WorkloadGenerator,
    hasher: StreamingSha256,
    hasher_before_this_step: StreamingSha256,
    trajectory: ArmTrajectory,
    is_running: bool,
}

/// 读法 1（跑前登记第一节）：撞上上限时的活数 = 最后一个被接受的版本里的记录条数，被拒那一条不算。序列与锚点格共用这一处。
#[must_use]
#[allow(clippy::let_and_return, reason = "中间那个名字是变异表 V4 的锚点：它说清这里不算被拒那一条")]
pub fn live_inodes_at_refusal(tree_after_the_refused_insert: &InodeLeafTree) -> u64 {
    let live_inodes_not_counting_the_refused_insert = tree_after_the_refused_insert.live_inode_count();
    live_inodes_not_counting_the_refused_insert
}

fn fill_is_smaller(candidate: (u64, u64), current: (u64, u64)) -> bool {
    u128::from(candidate.0) * u128::from(current.1) < u128::from(current.0) * u128::from(candidate.1)
}

/// 把一个操作喂给一条臂：插入被拒就记停下（树不变、被拒那一条不算）；接受了就进操作流指纹、跑丙类断言、更新轨迹。
fn apply_operation(run: &mut ArmRun, operation: Operation, sequence_name: &str, step: u64) -> StepOutcome {
    let (outcome, touched_positions) = match operation {
        Operation::Create(inode) => match run.tree.insert(inode) {
            InsertOutcome::Accepted => (StepOutcome::Created(operation), vec![run.tree.containers.len() - 1]),
            InsertOutcome::RefusedMoreLeafContainersThanOneRootNodeHolds { .. } => {
                let live_inodes_when_refused = live_inodes_at_refusal(&run.tree);
                run.is_running = false;
                run.trajectory.stop = ArmStop::Refused { refused_operation_number: step + 1 };
                run.trajectory.live_inodes_at_stop = live_inodes_when_refused;
                return StepOutcome::Refused(operation);
            }
        },
        Operation::Unlink(inode) => {
            let rewrites = run.tree.remove(inode);
            let bound = content_notation_bound_per_removal(&run.tree);
            assert!(
                rewrites.content_notation_count() <= bound,
                "F4 丙类断言红：序列 {sequence_name} 臂 {} 第 {} 个操作 {operation:?} 内容记法重写 {} 片，上界 {bound}",
                run.variant.name,
                step + 1,
                rewrites.content_notation_count()
            );
            let content = rewrites.content_notation_count();
            let position = rewrites.position_notation_count();
            (StepOutcome::Unlinked(operation, content, position), rewrites.content_rewritten_positions)
        }
    };
    run.hasher.update(&operation.fingerprint_bytes());
    run.trajectory.operations_accepted += 1;
    let third_sentence_false_pairs = check_structure_after_publish(&run.tree, &touched_positions).unwrap_or_else(|violation| {
        panic!("F4 丙类断言红：序列 {sequence_name} 臂 {} 第 {} 个操作 {operation:?}：{violation}", run.variant.name, step + 1)
    });
    let containers = run.tree.containers.len();
    let trajectory = &mut run.trajectory;
    trajectory.containers_peak = trajectory.containers_peak.max(containers);
    if containers == ROOT_NODE_LEAF_CONTAINER_CAPACITY {
        trajectory.publishes_at_root_capacity += 1;
    }
    if containers > 0 {
        let fill = (run.tree.live_inode_count(), u64::try_from(containers * LEAF_RECORD_CAPACITY).expect("片数 × 233 装得进 u64"));
        if fill_is_smaller(fill, trajectory.fill_smallest) {
            trajectory.fill_smallest = fill;
        }
        trajectory.fill_final = fill;
    }
    trajectory.third_sentence_false_peak = trajectory.third_sentence_false_peak.max(third_sentence_false_pairs);
    if third_sentence_false_pairs > 0 {
        trajectory.third_sentence_false_positive_publishes += 1;
    }
    trajectory.third_sentence_false_final = third_sentence_false_pairs;
    outcome
}

fn operation_of(outcome: &StepOutcome) -> Option<Operation> {
    match outcome {
        StepOutcome::NotTaken => None,
        StepOutcome::Refused(operation) | StepOutcome::Created(operation) | StepOutcome::Unlinked(operation, _, _) => Some(*operation),
    }
}

fn accepted(outcome: &StepOutcome) -> bool {
    match outcome {
        StepOutcome::NotTaken | StepOutcome::Refused(_) => false,
        StepOutcome::Created(_) | StepOutcome::Unlinked(..) => true,
    }
}

/// 窗口还开着时记一步：有一条臂没接受这一步的操作就关窗（这一步不算），记三条臂此前的操作流指纹；否则三条臂都删了就记重写片数。
fn record_step_in_window(window: &mut ComparisonWindow, runs: &[ArmRun], outcomes: &[StepOutcome], step: u64) {
    let triple_outcomes = window.arm_indices.map(|arm_index| &outcomes[arm_index]);
    if let Some(slot) = triple_outcomes.iter().position(|outcome| !accepted(outcome)) {
        window.is_open = false;
        window.operations = step;
        window.ended_by_arm = Some(runs[window.arm_indices[slot]].variant.name);
        for (slot_index, arm_index) in window.arm_indices.iter().enumerate() {
            let hasher = if accepted(&outcomes[*arm_index]) { &runs[*arm_index].hasher_before_this_step } else { &runs[*arm_index].hasher };
            window.prefix_sha256[slot_index] = hasher.hexadecimal_digest();
        }
        return;
    }
    let operations = triple_outcomes.map(operation_of);
    if operations[0] != operations[1] || operations[0] != operations[2] {
        window.operations_identical = false;
    }
    let StepOutcome::Unlinked(_, m1_content, m1_position) = triple_outcomes[0] else { return };
    window.unlinks += 1;
    for (slot_index, outcome) in triple_outcomes.iter().enumerate() {
        let StepOutcome::Unlinked(_, content, position) = outcome else {
            window.operations_identical = false;
            continue;
        };
        let rewrites = &mut window.rewrites[slot_index];
        rewrites.content_sum += u64::try_from(*content).expect("片数装得进 u64");
        rewrites.position_sum += u64::try_from(*position).expect("片数装得进 u64");
        rewrites.content_largest = rewrites.content_largest.max(*content);
        rewrites.position_largest = rewrites.position_largest.max(*position);
        rewrites.content_histogram[histogram_bucket(*content)] += 1;
        rewrites.position_histogram[histogram_bucket(*position)] += 1;
        if slot_index > 0 {
            let position_difference = i64::try_from(*position).expect("片数") - i64::try_from(*m1_position).expect("片数");
            let content_difference = i64::try_from(*content).expect("片数") - i64::try_from(*m1_content).expect("片数");
            window.cumulative[(slot_index - 1) * 2].add(position_difference);
            window.cumulative[(slot_index - 1) * 2 + 1].add(content_difference);
        }
    }
}

/// 第一段的两个公共窗口：主窗口（M1、M2 主、M3 主）与 f = 59 那一个（M1、M2 主、M3 f = 59），下标指进 [`SEGMENT_ONE_ARMS`]。
pub const SEGMENT_ONE_WINDOWS: [(&str, [usize; 3]); 2] = [("main", [0, 1, 2]), ("floor59", [0, 1, 3])];
/// 第二段的三个公共窗口（登记修订 3）：主窗口（M1、M2 主、M3 主）；借到刚好够下限那一个（M1、M2 主、M3 借到刚好够下限）；
/// 先查左边那一个（M1、M2 先查左边、M3 先查左边）。下标指进 [`SEGMENT_TWO_ARMS`]。
pub const SEGMENT_TWO_WINDOWS: [(&str, [usize; 3]); 3] = [("main", [0, 1, 2]), ("just_enough", [0, 1, 3]), ("left_first", [0, 4, 5])];

/// 一条序列喂给一组臂变体，逐步同走：每条臂各用一个生成器实例、各算各的操作流指纹，直到每条臂都停下（被拒或吃满上限）。
#[must_use]
pub fn run_sequence(
    sequence: RegisteredSequence,
    arms: &[ArmVariant],
    window_triples: &[(&'static str, [usize; 3])],
    operations_per_arm_at_most: u64,
) -> SequenceResult {
    let mut runs: Vec<ArmRun> = arms
        .iter()
        .map(|arm_variant| ArmRun {
            variant: *arm_variant,
            tree: InodeLeafTree::empty(arm_variant.policy, LeafContainerLimit::OneRootNode),
            generator: WorkloadGenerator::new(sequence.workload, sequence.seed),
            hasher: StreamingSha256::new(),
            hasher_before_this_step: StreamingSha256::new(),
            trajectory: ArmTrajectory {
                arm: arm_variant.name,
                stop: ArmStop::ReachedOperationCap,
                operations_accepted: 0,
                live_inodes_at_stop: 0,
                containers_peak: 0,
                publishes_at_root_capacity: 0,
                containers_final: 0,
                fill_smallest: (1, 1),
                fill_final: (0, 1),
                merges: 0,
                borrows: 0,
                third_sentence_false_peak: 0,
                third_sentence_false_positive_publishes: 0,
                third_sentence_false_final: 0,
                consumed_prefix_sha256: String::new(),
            },
            is_running: true,
        })
        .collect();
    let mut windows: Vec<ComparisonWindow> = window_triples.iter().map(|(label, arm_indices)| ComparisonWindow::opened(label, *arm_indices)).collect();
    let mut step: u64 = 0;
    while runs.iter().any(|run| run.is_running) {
        let any_window_open = windows.iter().any(|window| window.is_open);
        let mut outcomes: Vec<StepOutcome> = Vec::with_capacity(runs.len());
        for run in &mut runs {
            if !run.is_running {
                outcomes.push(StepOutcome::NotTaken);
                continue;
            }
            if run.trajectory.operations_accepted == operations_per_arm_at_most {
                run.is_running = false;
                run.trajectory.stop = ArmStop::ReachedOperationCap;
                run.trajectory.live_inodes_at_stop = run.tree.live_inode_count();
                outcomes.push(StepOutcome::NotTaken);
                continue;
            }
            if any_window_open {
                run.hasher_before_this_step = run.hasher.clone();
            }
            let operation = run.generator.next_operation();
            outcomes.push(apply_operation(run, operation, sequence.name, step));
        }
        for window in windows.iter_mut().filter(|window| window.is_open) {
            record_step_in_window(window, &runs, &outcomes, step);
        }
        step += 1;
    }
    let trajectories = runs
        .into_iter()
        .map(|run| {
            let mut trajectory = run.trajectory;
            trajectory.containers_final = run.tree.containers.len();
            trajectory.merges = run.tree.merges;
            trajectory.borrows = run.tree.borrows;
            trajectory.consumed_prefix_sha256 = run.hasher.hexadecimal_digest();
            trajectory
        })
        .collect();
    SequenceResult { sequence, trajectories, windows }
}

/// 第十一节作废条款：公共窗口内三条臂的操作流前缀指纹必须相同，逐步的操作也必须相同。不同就当场停下（这一段产物不算）。
pub fn assert_window_prefixes_agree(result: &SequenceResult) {
    for window in &result.windows {
        assert!(
            window.operations_identical
                && window.prefix_sha256[0] == window.prefix_sha256[1]
                && window.prefix_sha256[0] == window.prefix_sha256[2],
            "作废：序列 {} 窗口 {} 三条臂的操作流前缀不同：{:?}",
            result.sequence.name,
            window.label,
            window.prefix_sha256
        );
    }
}

/// 一个锚点格的读数：格名、臂、读到的（规范写法）、登记的答案（第七节；没有登记答案的写 None，只报不判）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnchorReading {
    pub cell: &'static str,
    pub arm: &'static str,
    pub observed: String,
    pub expected: Option<&'static str>,
}

impl AnchorReading {
    #[must_use]
    pub fn matches(&self) -> bool {
        self.expected.is_none_or(|expected| expected == self.observed)
    }
}

fn joined_sizes(tree: &InodeLeafTree) -> String {
    tree.containers.iter().map(|container| container.inode_numbers.len().to_string()).collect::<Vec<_>>().join("/")
}

/// 插入 `first_inode`、`first_inode` + 1 … 直到被拒，交回被拒的那个号。
fn insert_until_refused(tree: &mut InodeLeafTree, first_inode: u64) -> u64 {
    let mut inode = first_inode;
    loop {
        match tree.insert(inode) {
            InsertOutcome::Accepted => inode += 1,
            InsertOutcome::RefusedMoreLeafContainersThanOneRootNodeHolds { .. } => return inode,
        }
    }
}

/// A1：插入 1, 2, … 直到被拒。
#[must_use]
pub fn anchor_a1(arm: ArmVariant) -> String {
    let mut tree = InodeLeafTree::empty(arm.policy, LeafContainerLimit::OneRootNode);
    let refused_inode = insert_until_refused(&mut tree, 1);
    format!("refused_insert={refused_inode} live={} containers={}", live_inodes_at_refusal(&tree), tree.containers.len())
}

/// A2：插入 1 … 31 455；对 k = 0 … 134 按号升序删 233k + 2 … 233k + 233；再插入直到被拒。
#[must_use]
pub fn anchor_a2(arm: ArmVariant) -> String {
    let mut tree = InodeLeafTree::empty(arm.policy, LeafContainerLimit::OneRootNode);
    for inode in 1..=31_455_u64 {
        assert_eq!(tree.insert(inode), InsertOutcome::Accepted, "A2 建满 135 片之前不会被拒");
    }
    for leaf in 0..135_u64 {
        for inode in (233 * leaf + 2)..=(233 * leaf + 233) {
            let rewrites = tree.remove(inode);
            check_structure_after_publish(&tree, &rewrites.content_rewritten_positions).unwrap_or_else(|violation| panic!("A2 臂 {} 删 {inode}：{violation}", arm.name));
        }
    }
    let refused_inode = insert_until_refused(&mut tree, 31_456);
    format!(
        "refused_insert={refused_inode} live={} containers={} merges={} borrows={}",
        live_inodes_at_refusal(&tree),
        tree.containers.len(),
        tree.merges,
        tree.borrows
    )
}

/// A3–A6 那种手造场景：插入 1 … `inserted`，再按给定次序删；交回两种记法的合计、末态片数、并 / 借次数、末态 I-9.4 第三句为假的对数、末态各片记录数。
#[must_use]
pub fn anchor_scenario_tree(arm: ArmVariant, inserted: u64, deleted: &[u64]) -> (String, InodeLeafTree) {
    let mut tree = InodeLeafTree::empty(arm.policy, LeafContainerLimit::OneRootNode);
    for inode in 1..=inserted {
        assert_eq!(tree.insert(inode), InsertOutcome::Accepted, "手造场景插不到 135 片");
    }
    let (mut content_total, mut position_total) = (0_usize, 0_usize);
    for inode in deleted {
        let rewrites = tree.remove(*inode);
        check_structure_after_publish(&tree, &rewrites.content_rewritten_positions).unwrap_or_else(|violation| panic!("场景 臂 {} 删 {inode}：{violation}", arm.name));
        content_total += rewrites.content_notation_count();
        position_total += rewrites.position_notation_count();
    }
    let observed = format!(
        "content={content_total} position={position_total} containers={} merges={} borrows={} third_sentence_false={} sizes={}",
        tree.containers.len(),
        tree.merges,
        tree.borrows,
        tree.adjacent_pairs_where_left_largest_reaches_right_container_number(),
        joined_sizes(&tree)
    );
    (observed, tree)
}

/// C1：不设 135 上限的 M1，按号 1 … 10⁶ 建满，再按号升序删 (n − 1) mod 10 ≠ 0 的每一条；交回叶数、活数、⌈活数 ÷ 233⌉、比值（四舍五入到两位）。
#[must_use]
pub fn anchor_c1() -> String {
    let mut tree = InodeLeafTree::empty(MergePolicy::DropEmptyOnly, LeafContainerLimit::Unlimited);
    for inode in 1..=1_000_000_u64 {
        assert_eq!(tree.insert(inode), InsertOutcome::Accepted, "C1 不设上限");
    }
    for inode in (1..=1_000_000_u64).filter(|inode| (inode - 1) % 10 != 0) {
        let _ = tree.remove(inode);
    }
    let leaves = u64::try_from(tree.containers.len()).expect("叶数装得进 u64");
    let live = tree.live_inode_count();
    let ideal = live.div_ceil(u64::try_from(LEAF_RECORD_CAPACITY).expect("233"));
    let ratio_in_hundredths = (leaves * 200 + ideal) / (ideal * 2);
    format!("leaves={leaves} live={live} ideal={ideal} ratio={}.{:02}", ratio_in_hundredths / 100, ratio_in_hundredths % 100)
}

fn ascending_run(first: u64, last: u64) -> Vec<u64> {
    (first..=last).collect()
}

/// `crates/` 单测钉住的数（第七节乙类末段），模型这边写本地常量，对拍 bin 回比 `crates/`：
/// 插入 1 … 500 各片记录数与容器号；插入 1 … 233 × 135 + 1 拒在第 136 片、上限 135；三片 [1 … 233][234][467] 删 234 的重写清单与摘片数。
#[must_use]
pub fn crates_unit_test_readings() -> Vec<AnchorReading> {
    let mut five_hundred = InodeLeafTree::empty(MergePolicy::DropEmptyOnly, LeafContainerLimit::OneRootNode);
    for inode in 1..=500_u64 {
        assert_eq!(five_hundred.insert(inode), InsertOutcome::Accepted, "500 条装得下");
    }
    let container_numbers = five_hundred.containers.iter().map(|container| container.container_number.to_string()).collect::<Vec<_>>().join("/");
    let mut full = InodeLeafTree::empty(MergePolicy::DropEmptyOnly, LeafContainerLimit::OneRootNode);
    let mut refusal = String::from("never");
    for inode in 1..=(233 * 135 + 1_u64) {
        if let InsertOutcome::RefusedMoreLeafContainersThanOneRootNodeHolds { containers_it_would_take } = full.insert(inode) {
            refusal = format!("refused_inode={inode} refused_containers={containers_it_would_take} capacity={ROOT_NODE_LEAF_CONTAINER_CAPACITY}");
            break;
        }
    }
    let mut three = InodeLeafTree::from_containers(
        MergePolicy::DropEmptyOnly,
        LeafContainerLimit::OneRootNode,
        vec![
            LeafContainer { container_number: 1, inode_numbers: ascending_run(1, 233) },
            LeafContainer { container_number: 234, inode_numbers: vec![234] },
            LeafContainer { container_number: 467, inode_numbers: vec![467] },
        ],
    );
    let rewrites = three.remove(234);
    vec![
        AnchorReading {
            cell: "crates_insert_500",
            arm: "M1",
            observed: format!("sizes={} container_numbers={container_numbers}", joined_sizes(&five_hundred)),
            expected: Some("sizes=233/233/34 container_numbers=1/234/467"),
        },
        AnchorReading { cell: "crates_insert_31456", arm: "M1", observed: refusal, expected: Some("refused_inode=31456 refused_containers=136 capacity=135") },
        AnchorReading {
            cell: "crates_drop_middle",
            arm: "M1",
            observed: format!("position_rewritten={:?} dropped={}", rewrites.position_rewritten_positions, rewrites.containers_dropped),
            expected: Some("position_rewritten=[1] dropped=1"),
        },
    ]
}

/// 第七节甲、乙类全部锚点格，加上第五节「阳性对照对每条臂、每个旋钮取值都跑」里 f = 59 那一个变体（登记没给它答案，只报不判）。
#[must_use]
pub fn anchor_readings() -> Vec<AnchorReading> {
    let mut readings = Vec::new();
    for arm in [ARM_M1, ARM_M2_MAIN, ARM_M3_MAIN, ARM_M3_FLOOR_59] {
        readings.push(AnchorReading { cell: "A1", arm: arm.name, observed: anchor_a1(arm), expected: Some("refused_insert=31456 live=31455 containers=135") });
    }
    let a2_expected: [(ArmVariant, Option<&'static str>); 4] = [
        (ARM_M1, Some("refused_insert=31688 live=367 containers=135 merges=0 borrows=0")),
        (ARM_M2_MAIN, Some("refused_insert=62776 live=31455 containers=135 merges=134 borrows=0")),
        (ARM_M3_MAIN, Some("refused_insert=62776 live=31455 containers=135 merges=134 borrows=918")),
        (ARM_M3_FLOOR_59, None),
    ];
    for (arm, expected) in a2_expected {
        readings.push(AnchorReading { cell: "A2", arm: arm.name, observed: anchor_a2(arm), expected });
    }
    let a3_deleted = ascending_run(1, 117);
    let a4_deleted = ascending_run(1, 233);
    let a5_deleted: Vec<u64> = ascending_run(234, 350).into_iter().chain(ascending_run(467, 583)).collect();
    let a6_deleted: Vec<u64> = ascending_run(1, 117).into_iter().chain(ascending_run(234, 349)).collect();
    let scenarios: [(&'static str, u64, &[u64], [(ArmVariant, Option<&'static str>); 4]); 4] = [
        ("A3", 467, &a3_deleted, [
            (ARM_M1, Some("content=117 position=117 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=116/233/1")),
            (ARM_M2_MAIN, Some("content=117 position=117 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=116/233/1")),
            (ARM_M3_MAIN, Some("content=118 position=118 containers=2 merges=1 borrows=1 third_sentence_false=1 sizes=174/176")),
            (ARM_M3_FLOOR_59, None),
        ]),
        ("A4", 700, &a4_deleted, [
            (ARM_M1, Some("content=232 position=235 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=233/233/1")),
            (ARM_M2_MAIN, Some("content=232 position=235 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=233/233/1")),
            (ARM_M3_MAIN, Some("content=240 position=242 containers=3 merges=1 borrows=7 third_sentence_false=0 sizes=233/233/1")),
            (ARM_M3_FLOOR_59, None),
        ]),
        ("A5", 933, &a5_deleted, [
            (ARM_M1, Some("content=234 position=234 containers=5 merges=0 borrows=0 third_sentence_false=0 sizes=233/116/116/233/1")),
            (ARM_M2_MAIN, Some("content=234 position=236 containers=4 merges=1 borrows=0 third_sentence_false=0 sizes=233/232/233/1")),
            (ARM_M3_MAIN, Some("content=241 position=243 containers=4 merges=1 borrows=7 third_sentence_false=0 sizes=233/232/233/1")),
            (ARM_M3_FLOOR_59, None),
        ]),
        ("A6", 466, &a6_deleted, [
            (ARM_M1, Some("content=233 position=233 containers=2 merges=0 borrows=0 third_sentence_false=0 sizes=116/117")),
            (ARM_M2_MAIN, Some("content=233 position=233 containers=1 merges=1 borrows=0 third_sentence_false=0 sizes=233")),
            (ARM_M3_MAIN, Some("content=240 position=240 containers=1 merges=1 borrows=7 third_sentence_false=0 sizes=233")),
            (ARM_M3_FLOOR_59, None),
        ]),
    ];
    for (cell, inserted, deleted, arms) in scenarios {
        for (arm, expected) in arms {
            let (observed, tree) = anchor_scenario_tree(arm, inserted, deleted);
            readings.push(AnchorReading { cell, arm: arm.name, observed, expected });
            if cell == "A3" && arm == ARM_M3_MAIN {
                let boundary = format!(
                    "leaf0_largest={} leaf1_smallest={}",
                    tree.containers[0].largest_inode_number(),
                    tree.containers[1].smallest_inode_number()
                );
                readings.push(AnchorReading { cell: "A3_boundary", arm: arm.name, observed: boundary, expected: Some("leaf0_largest=291 leaf1_smallest=292") });
            }
        }
    }
    let (a3_just_enough, _) = anchor_scenario_tree(ARM_M3_JUST_ENOUGH_FOR_THE_FLOOR, 467, &a3_deleted);
    readings.push(AnchorReading {
        cell: "A3",
        arm: ARM_M3_JUST_ENOUGH_FOR_THE_FLOOR.name,
        observed: a3_just_enough,
        expected: Some("content=118 position=118 containers=2 merges=1 borrows=1 third_sentence_false=1 sizes=117/233"),
    });
    let (a5_left_first, _) = anchor_scenario_tree(ARM_M2_LEFT_PAIR_FIRST, 933, &a5_deleted);
    readings.push(AnchorReading {
        cell: "A5",
        arm: ARM_M2_LEFT_PAIR_FIRST.name,
        observed: a5_left_first,
        expected: Some("content=234 position=236 containers=4 merges=1 borrows=0 third_sentence_false=0 sizes=233/232/233/1"),
    });
    readings.push(AnchorReading { cell: "C1", arm: "M1_unlimited", observed: anchor_c1(), expected: Some("leaves=4292 live=100000 ideal=430 ratio=9.98") });
    readings.extend(crates_unit_test_readings());
    readings
}

/// 第二段的三个臂变体（登记第五节第二段那一行），次序就是锚点行里的次序。
pub const SEGMENT_TWO_VARIANTS: [ArmVariant; 3] = [ARM_M3_JUST_ENOUGH_FOR_THE_FLOOR, ARM_M2_LEFT_PAIR_FIRST, ARM_M3_LEFT_PAIR_FIRST];

/// 第二段的阳性对照（登记修订 3）：第一段的全部锚点格（M1、M2 主、M3 主在第二段照样进窗口），加第二段三个变体在 A1 … A6 上还没有读数的格。
/// A1 只插入、臂不起作用，三个变体都按 A1 那一格的登记答案判；A3 借到刚好够下限、A5 M2 先查左边两格第一段里已有、带登记答案；其余只报不判。
#[must_use]
pub fn segment_two_anchor_readings() -> Vec<AnchorReading> {
    let mut readings = anchor_readings();
    for arm in SEGMENT_TWO_VARIANTS {
        readings.push(AnchorReading { cell: "A1", arm: arm.name, observed: anchor_a1(arm), expected: Some("refused_insert=31456 live=31455 containers=135") });
    }
    for arm in SEGMENT_TWO_VARIANTS {
        readings.push(AnchorReading { cell: "A2", arm: arm.name, observed: anchor_a2(arm), expected: None });
    }
    let a3_deleted = ascending_run(1, 117);
    let a4_deleted = ascending_run(1, 233);
    let a5_deleted: Vec<u64> = ascending_run(234, 350).into_iter().chain(ascending_run(467, 583)).collect();
    let a6_deleted: Vec<u64> = ascending_run(1, 117).into_iter().chain(ascending_run(234, 349)).collect();
    let scenarios: [(&'static str, u64, &[u64]); 4] = [("A3", 467, &a3_deleted), ("A4", 700, &a4_deleted), ("A5", 933, &a5_deleted), ("A6", 466, &a6_deleted)];
    for (cell, inserted, deleted) in scenarios {
        for arm in SEGMENT_TWO_VARIANTS {
            let has_a_segment_one_reading = (cell == "A3" && arm == ARM_M3_JUST_ENOUGH_FOR_THE_FLOOR) || (cell == "A5" && arm == ARM_M2_LEFT_PAIR_FIRST);
            if has_a_segment_one_reading {
                continue;
            }
            let (observed, _) = anchor_scenario_tree(arm, inserted, deleted);
            readings.push(AnchorReading { cell, arm: arm.name, observed, expected: None });
        }
    }
    readings
}

/// 精确分数（分母恒正），判定一律按交叉相乘比，不经浮点。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rational {
    numerator: i128,
    denominator: i128,
}

impl Rational {
    #[must_use]
    pub fn new(numerator: i128, denominator: i128) -> Self {
        assert!(denominator != 0, "分母不为 0");
        if denominator < 0 {
            Self { numerator: -numerator, denominator: -denominator }
        } else {
            Self { numerator, denominator }
        }
    }

    #[must_use]
    pub fn compare(self, other: Self) -> std::cmp::Ordering {
        (self.numerator * other.denominator).cmp(&(other.numerator * self.denominator))
    }

    #[must_use]
    pub fn midpoint(self, other: Self) -> Self {
        Self::new(self.numerator * other.denominator + other.numerator * self.denominator, 2 * self.denominator * other.denominator)
    }

    /// 四舍五入（远离 0）到 `places` 位小数的十进制写法。
    #[must_use]
    pub fn decimal(self, places: u32) -> String {
        let scale = 10_i128.pow(places);
        let magnitude = self.numerator.abs() * scale;
        let rounded = (2 * magnitude + self.denominator) / (2 * self.denominator);
        let sign = if self.numerator < 0 && rounded != 0 { "-" } else { "" };
        let width = usize::try_from(places).expect("位数很小");
        format!("{sign}{}.{:0width$}", rounded / scale, rounded % scale)
    }
}

/// Q1r 的门槛（第六节）：R12 ≥ 0.90 判「不合并的容量代价小」，R12 < 0.50 判「合并买到成倍容量」。
pub const CAPACITY_RATIO_SMALL_COST_AT_LEAST: Rational = Rational { numerator: 90, denominator: 100 };
pub const CAPACITY_RATIO_DOUBLED_BELOW: Rational = Rational { numerator: 50, denominator: 100 };
/// Q2、Q3 的门槛（第六节）：E ≤ 0.10 判「可忽略」，E ≥ 1.00 判「显著」。
pub const WRITE_COST_NEGLIGIBLE_AT_MOST: Rational = Rational { numerator: 10, denominator: 100 };
pub const WRITE_COST_SIGNIFICANT_AT_LEAST: Rational = Rational { numerator: 100, denominator: 100 };

/// 登记的判定行（第六节「量与判据」那张表）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JudgedRow {
    /// Q1r：L_M1 ÷ L_M2。
    CapacityRatio,
    /// Q2p / Q2c / Q3：两条臂（窗口里的槽位）每次 unlink 平均重写片数之差，`minuend_slot` − `subtrahend_slot`。
    WriteCost { name: &'static str, minuend_slot: usize, subtrahend_slot: usize, notation: RewriteNotation, significant_side_counts: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewriteNotation {
    Position,
    Content,
}

impl JudgedRow {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            JudgedRow::CapacityRatio => "Q1r",
            JudgedRow::WriteCost { name, .. } => name,
        }
    }
}

/// 第六节的七行，次序就是产物里的次序。Q2c 的 Δ2c 那一行「≥ 1.00」一侧由臂的定义限死，不算判据（第四节）。
pub const JUDGED_ROWS: [JudgedRow; 7] = [
    JudgedRow::CapacityRatio,
    JudgedRow::WriteCost { name: "Q2p_delta2", minuend_slot: 1, subtrahend_slot: 0, notation: RewriteNotation::Position, significant_side_counts: true },
    JudgedRow::WriteCost { name: "Q2p_delta3", minuend_slot: 2, subtrahend_slot: 0, notation: RewriteNotation::Position, significant_side_counts: true },
    JudgedRow::WriteCost { name: "Q2c_delta2", minuend_slot: 1, subtrahend_slot: 0, notation: RewriteNotation::Content, significant_side_counts: false },
    JudgedRow::WriteCost { name: "Q2c_delta3", minuend_slot: 2, subtrahend_slot: 0, notation: RewriteNotation::Content, significant_side_counts: true },
    JudgedRow::WriteCost { name: "Q3p_delta32", minuend_slot: 2, subtrahend_slot: 1, notation: RewriteNotation::Position, significant_side_counts: true },
    JudgedRow::WriteCost { name: "Q3c_delta32", minuend_slot: 2, subtrahend_slot: 1, notation: RewriteNotation::Content, significant_side_counts: true },
];

/// 一个取样点上一行的读数：值（读不到或不判时是 None）与旁注。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointValue {
    pub value: Option<Rational>,
    pub note: &'static str,
}

/// 一行的门槛（自证判别力时会把其中一个挪到两个取样点的值之间）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Thresholds {
    pub lower: Rational,
    pub upper: Rational,
}

#[must_use]
pub fn registered_thresholds(row: JudgedRow) -> Thresholds {
    match row {
        JudgedRow::CapacityRatio => Thresholds { lower: CAPACITY_RATIO_DOUBLED_BELOW, upper: CAPACITY_RATIO_SMALL_COST_AT_LEAST },
        JudgedRow::WriteCost { .. } => Thresholds { lower: WRITE_COST_NEGLIGIBLE_AT_MOST, upper: WRITE_COST_SIGNIFICANT_AT_LEAST },
    }
}

/// 一个取样点上一行的判定（第六节门槛，按点各判一次）。
#[must_use]
pub fn point_verdict(row: JudgedRow, point: PointValue, thresholds: Thresholds) -> &'static str {
    let Some(value) = point.value else { return point.note };
    if point.blocks_judgement() {
        return point.note;
    }
    match row {
        JudgedRow::CapacityRatio => {
            if value.compare(thresholds.upper).is_ge() {
                "no_merge_capacity_cost_small"
            } else if value.compare(thresholds.lower).is_lt() {
                "merge_buys_double_capacity"
            } else {
                "no_flip"
            }
        }
        JudgedRow::WriteCost { significant_side_counts, .. } => {
            if value.compare(thresholds.lower).is_le() {
                "negligible"
            } else if significant_side_counts && value.compare(thresholds.upper).is_ge() {
                "significant"
            } else {
                "no_flip"
            }
        }
    }
}

impl PointValue {
    /// 「样本不足」（公共窗口里最后一批不足 100 次）与「M1 未撞到」（F6）报数不判；「上界」（M2 吃满上限没被拒）照判、带着旁注。
    #[must_use]
    pub fn blocks_judgement(self) -> bool {
        matches!(self.note, "insufficient_sample" | "m1_not_reached")
    }
}

/// 一条序列上一行的读数。`window_index` 0 是主窗口（M3 主），1 是 f = 59 那个窗口（第 2 槽换成 M3 f = 59）。
#[must_use]
pub fn point_value(result: &SequenceResult, row: JudgedRow, window_index: usize) -> PointValue {
    match row {
        JudgedRow::CapacityRatio => {
            let m1 = &result.trajectories[0];
            let m2 = &result.trajectories[1];
            if m1.stop == ArmStop::ReachedOperationCap {
                return PointValue { value: None, note: "m1_not_reached" };
            }
            let note = if m2.stop == ArmStop::ReachedOperationCap { "upper_bound" } else { "" };
            PointValue { value: Some(Rational::new(i128::from(m1.live_inodes_at_stop), i128::from(m2.live_inodes_at_stop))), note }
        }
        JudgedRow::WriteCost { minuend_slot, subtrahend_slot, notation, .. } => {
            let window = &result.windows[window_index];
            if window.unlinks == 0 {
                return PointValue { value: None, note: "insufficient_sample" };
            }
            let sum_of = |slot: usize| -> i128 {
                match notation {
                    RewriteNotation::Position => i128::from(window.rewrites[slot].position_sum),
                    RewriteNotation::Content => i128::from(window.rewrites[slot].content_sum),
                }
            };
            let value = Rational::new(sum_of(minuend_slot) - sum_of(subtrahend_slot), i128::from(window.unlinks));
            let note = if window.unlinks < UNLINKS_IN_THE_WINDOW_FOR_A_JUDGEMENT_AT_LEAST { "insufficient_sample" } else { "" };
            PointValue { value: Some(value), note }
        }
    }
}

/// 一行在全部登记点上的总判定（第六节）：Q1r 任一主取样点 < 0.50 ⇒ 成倍；每一个登记点都 ≥ 0.90 ⇒ 代价小；其余不翻。
/// Q2、Q3 任一主取样点 ≥ 1.00 ⇒ 显著（Δ2c 不算这一侧）；每一个判得了的登记点都 ≤ 0.10 ⇒ 可忽略；其余不翻。样本不足的点不进判定。
#[must_use]
pub fn overall_verdict(results: &[SequenceResult], row: JudgedRow, window_index: usize) -> &'static str {
    let thresholds = registered_thresholds(row);
    let points: Vec<(bool, PointValue)> = results.iter().map(|result| (result.sequence.is_main_sampling_point, point_value(result, row, window_index))).collect();
    match row {
        JudgedRow::CapacityRatio => {
            let doubled = points.iter().any(|(is_main, point)| {
                *is_main && !point.blocks_judgement() && point.value.is_some_and(|value| value.compare(thresholds.lower).is_lt())
            });
            let all_small = points.iter().all(|(_, point)| !point.blocks_judgement() && point.value.is_some_and(|value| value.compare(thresholds.upper).is_ge()));
            if doubled {
                "merge_buys_double_capacity"
            } else if all_small {
                "no_merge_capacity_cost_small"
            } else {
                "no_flip"
            }
        }
        JudgedRow::WriteCost { significant_side_counts, .. } => {
            let judged: Vec<&(bool, PointValue)> = points.iter().filter(|(_, point)| !point.blocks_judgement() && point.value.is_some()).collect();
            let significant = significant_side_counts
                && judged.iter().any(|(is_main, point)| *is_main && point.value.is_some_and(|value| value.compare(thresholds.upper).is_ge()));
            let all_negligible = !judged.is_empty() && judged.iter().all(|(_, point)| point.value.is_some_and(|value| value.compare(thresholds.lower).is_le()));
            if significant {
                "significant"
            } else if all_negligible {
                "negligible"
            } else {
                "no_flip"
            }
        }
    }
}

/// 判别力自证（第八节末）：两点判定相同时，把一个门槛挪到两点的值之间，判定必须转成不同。交回挪后的门槛与两点挪后的判定。
#[must_use]
pub fn moved_threshold_verdicts(row: JudgedRow, main: PointValue, opposite: PointValue) -> Option<(Thresholds, &'static str, &'static str)> {
    let (main_value, opposite_value) = (main.value?, opposite.value?);
    if main_value.compare(opposite_value).is_eq() {
        return None;
    }
    let registered = registered_thresholds(row);
    let between = main_value.midpoint(opposite_value);
    let smaller = if main_value.compare(opposite_value).is_lt() { main_value } else { opposite_value };
    let moved = match row {
        JudgedRow::CapacityRatio => {
            if smaller.compare(registered.lower).is_lt() && main_value.max_with(opposite_value).compare(registered.lower).is_lt() {
                Thresholds { lower: between, upper: registered.upper }
            } else {
                Thresholds { lower: registered.lower, upper: between }
            }
        }
        JudgedRow::WriteCost { significant_side_counts, .. } => {
            let larger = main_value.max_with(opposite_value);
            if significant_side_counts && smaller.compare(registered.upper).is_ge() && larger.compare(registered.upper).is_ge() {
                Thresholds { lower: registered.lower, upper: between }
            } else {
                Thresholds { lower: between, upper: registered.upper }
            }
        }
    };
    Some((moved, point_verdict(row, main, moved), point_verdict(row, opposite, moved)))
}

impl Rational {
    #[must_use]
    pub fn max_with(self, other: Self) -> Self {
        if self.compare(other).is_ge() {
            self
        } else {
            other
        }
    }
}

fn histogram_text(histogram: &[u64; 5]) -> String {
    histogram.iter().map(u64::to_string).collect::<Vec<_>>().join("/")
}

fn stop_text(stop: ArmStop) -> String {
    match stop {
        ArmStop::Refused { refused_operation_number } => format!("stop=refused refused_operation={refused_operation_number}"),
        ArmStop::ReachedOperationCap => format!("stop=not_reached operation_cap={OPERATIONS_PER_ARM_AT_MOST}"),
    }
}

fn value_text(point: PointValue) -> String {
    point.value.map_or_else(|| "unreadable".to_string(), |value| value.decimal(4))
}

/// 一条序列的全部产物行（不带 `E7RESULT ` 前缀）。
#[must_use]
pub fn sequence_lines(result: &SequenceResult) -> Vec<String> {
    let sequence = result.sequence;
    let mut lines = Vec::new();
    for trajectory in &result.trajectories {
        let full_tree = i128::try_from(LEAF_RECORD_CAPACITY * ROOT_NODE_LEAF_CONTAINER_CAPACITY).expect("31455");
        lines.push(format!(
            "name=arm_run sequence={} family={} main_sampling_point={} arm={} {} operations_accepted={} live_at_stop={} live_over_31455={} \
             containers_peak={} publishes_at_135={} containers_final={} fill_smallest={} fill_final={} merges={} borrows={} \
             i94_third_false_peak={} i94_third_false_positive_publishes={} i94_third_false_final={} consumed_prefix_sha256={}",
            sequence.name,
            sequence.family,
            sequence.is_main_sampling_point,
            trajectory.arm,
            stop_text(trajectory.stop),
            trajectory.operations_accepted,
            trajectory.live_inodes_at_stop,
            Rational::new(i128::from(trajectory.live_inodes_at_stop), full_tree).decimal(4),
            trajectory.containers_peak,
            trajectory.publishes_at_root_capacity,
            trajectory.containers_final,
            Rational::new(i128::from(trajectory.fill_smallest.0), i128::from(trajectory.fill_smallest.1)).decimal(4),
            Rational::new(i128::from(trajectory.fill_final.0), i128::from(trajectory.fill_final.1)).decimal(4),
            trajectory.merges,
            trajectory.borrows,
            trajectory.third_sentence_false_peak,
            trajectory.third_sentence_false_positive_publishes,
            trajectory.third_sentence_false_final,
            trajectory.consumed_prefix_sha256
        ));
    }
    for (window_index, window) in result.windows.iter().enumerate() {
        let arm_names = window.arm_indices.map(|arm_index| result.trajectories[arm_index].arm);
        let prefixes_equal = window.operations_identical && window.prefix_sha256[0] == window.prefix_sha256[1] && window.prefix_sha256[0] == window.prefix_sha256[2];
        lines.push(format!(
            "name=window sequence={} window={} arms={} operations={} unlinks={} ended_by={} prefix_sha256={} prefixes_equal={prefixes_equal}",
            sequence.name,
            window.label,
            arm_names.join(","),
            window.operations,
            window.unlinks,
            window.ended_by_arm.unwrap_or("none"),
            window.prefix_sha256.join(",")
        ));
        for (slot, rewrites) in window.rewrites.iter().enumerate() {
            let unlinks = i128::from(window.unlinks.max(1));
            lines.push(format!(
                "name=window_rewrites sequence={} window={} arm={} unlinks={} content_sum={} position_sum={} content_mean={} position_mean={} \
                 content_largest={} position_largest={} content_histogram={} position_histogram={}",
                sequence.name,
                window.label,
                arm_names[slot],
                window.unlinks,
                rewrites.content_sum,
                rewrites.position_sum,
                Rational::new(i128::from(rewrites.content_sum), unlinks).decimal(4),
                Rational::new(i128::from(rewrites.position_sum), unlinks).decimal(4),
                rewrites.content_largest,
                rewrites.position_largest,
                histogram_text(&rewrites.content_histogram),
                histogram_text(&rewrites.position_histogram)
            ));
        }
        for (pair_index, cumulative) in window.cumulative.iter().enumerate() {
            let notation = if pair_index % 2 == 0 { "position" } else { "content" };
            lines.push(format!(
                "name=cumulative_difference sequence={} window={} pair={}-M1 notation={notation} peak={} positive_single_differences={} final={}",
                sequence.name,
                window.label,
                arm_names[pair_index / 2 + 1],
                cumulative.peak,
                cumulative.positive_single_differences,
                cumulative.running
            ));
        }
        for row in JUDGED_ROWS {
            if row == JudgedRow::CapacityRatio && window_index > 0 {
                continue;
            }
            let point = point_value(result, row, window_index);
            lines.push(format!(
                "name=point_verdict sequence={} window={} row={} value={} note={} verdict={}",
                sequence.name,
                window.label,
                row.name(),
                value_text(point),
                if point.note.is_empty() { "none" } else { point.note },
                point_verdict(row, point, registered_thresholds(row))
            ));
        }
    }
    lines
}

/// 第八节「几何 / 旋钮敏感性」里第一段跑的负载旋钮：(旋钮, 主取样点或参照点, 方向相反的取样点)。W-R 按种子逐个配对。
pub const WORKLOAD_KNOB_PAIRS: [(&str, &str, &str); 17] = [
    ("insert_probability_0.95", "wr_p055_seed1", "wr_p095_seed1"),
    ("insert_probability_0.95", "wr_p055_seed2", "wr_p095_seed2"),
    ("insert_probability_0.95", "wr_p055_seed3", "wr_p095_seed3"),
    ("insert_probability_0.95", "wr_p055_seed4", "wr_p095_seed4"),
    ("insert_probability_0.95", "wr_p055_seed5", "wr_p095_seed5"),
    ("insert_probability_0.75", "wr_p055_seed1", "wr_p075_seed1"),
    ("insert_probability_0.75", "wr_p055_seed2", "wr_p075_seed2"),
    ("insert_probability_0.75", "wr_p055_seed3", "wr_p075_seed3"),
    ("insert_probability_0.75", "wr_p055_seed4", "wr_p075_seed4"),
    ("insert_probability_0.75", "wr_p055_seed5", "wr_p075_seed5"),
    ("deleted_fraction_0.95", "ws_a1000_d500_smallest_first", "ws_a1000_d950_smallest_first"),
    ("deletion_order_largest_first", "ws_a1000_d500_smallest_first", "ws_a1000_d500_largest_first"),
    ("kept_fraction_3of4", "wa_keep1of2_a1000", "wa_keep3of4_a1000"),
    ("kept_fraction_1of233", "wa_keep1of2_a2330", "wa_keep1of233_a2330"),
    ("batch_size_2330", "wa_keep1of2_a1000", "wa_keep1of2_a2330"),
    ("borrow_floor_59", "", ""),
    ("rewrite_notation_content", "", ""),
];

/// 两点各按自己那一行的规则判（记法旋钮那一对是 Q2p 对 Q2c，规则只差 Δ2c 不算「显著」一侧）；挪门槛自证按方向相反那一点的行做。
fn sensitivity_line(knob: &str, main_label: &str, opposite_label: &str, rows: (JudgedRow, JudgedRow), main: PointValue, opposite: PointValue) -> String {
    let (main_row, row) = rows;
    let registered = registered_thresholds(row);
    let main_verdict = point_verdict(main_row, main, registered);
    let opposite_verdict = point_verdict(row, opposite, registered);
    let comparable = !main.blocks_judgement() && !opposite.blocks_judgement() && main.value.is_some() && opposite.value.is_some();
    let result = if !comparable {
        "not_comparable"
    } else if main_verdict == opposite_verdict {
        "two_points_same"
    } else {
        "unstable"
    };
    let mut line = format!(
        "name=sensitivity knob={knob} row={} main={main_label} main_value={} main_verdict={main_verdict} opposite={opposite_label} opposite_value={} opposite_verdict={opposite_verdict} result={result}",
        if main_row == row { row.name().to_string() } else { format!("{}/{}", main_row.name(), row.name()) },
        value_text(main),
        value_text(opposite)
    );
    if result == "two_points_same" {
        match moved_threshold_verdicts(row, main, opposite) {
            Some((moved, moved_main, moved_opposite)) => {
                let moved_result = if moved_main == moved_opposite { "two_points_same" } else { "unstable" };
                write!(
                    line,
                    " moved_lower={} moved_upper={} moved_main_verdict={moved_main} moved_opposite_verdict={moved_opposite} moved_result={moved_result}",
                    moved.lower.decimal(6),
                    moved.upper.decimal(6)
                )
                .expect("写进 String 不会失败");
            }
            None => line.push_str(" moved_result=equal_values_cannot_be_separated"),
        }
    }
    line
}

/// 第八节的敏感性：负载旋钮按点配对；f 旋钮在每条序列上比主窗口与 f = 59 窗口（只比涉及 M3 的四行）；记法旋钮在每个主取样点上比位置记法与内容记法的同一对臂。
#[must_use]
pub fn sensitivity_lines(results: &[SequenceResult]) -> Vec<String> {
    let by_name = |name: &str| results.iter().find(|result| result.sequence.name == name).expect("旋钮配对点名的序列都在登记里");
    let mut lines = Vec::new();
    for (knob, main_name, opposite_name) in WORKLOAD_KNOB_PAIRS.iter().filter(|(_, main_name, _)| !main_name.is_empty()) {
        let (main_result, opposite_result) = (by_name(main_name), by_name(opposite_name));
        for row in JUDGED_ROWS {
            lines.push(sensitivity_line(knob, main_name, opposite_name, (row, row), point_value(main_result, row, 0), point_value(opposite_result, row, 0)));
        }
    }
    for result in results {
        for row in JUDGED_ROWS.iter().filter(|row| matches!(row, JudgedRow::WriteCost { minuend_slot: 2, .. })) {
            let main_label = format!("{}@M3", result.sequence.name);
            let opposite_label = format!("{}@M3_floor59", result.sequence.name);
            lines.push(sensitivity_line("borrow_floor_59", &main_label, &opposite_label, (*row, *row), point_value(result, *row, 0), point_value(result, *row, 1)));
        }
    }
    let notation_pairs = [(JUDGED_ROWS[1], JUDGED_ROWS[3]), (JUDGED_ROWS[2], JUDGED_ROWS[4]), (JUDGED_ROWS[5], JUDGED_ROWS[6])];
    for result in results.iter().filter(|result| result.sequence.is_main_sampling_point) {
        for (position_row, content_row) in notation_pairs {
            let main_label = format!("{}@{}", result.sequence.name, position_row.name());
            let opposite_label = format!("{}@{}", result.sequence.name, content_row.name());
            let line = sensitivity_line(
                "rewrite_notation_content",
                &main_label,
                &opposite_label,
                (position_row, content_row),
                point_value(result, position_row, 0),
                point_value(result, content_row, 0),
            );
            lines.push(line);
        }
    }
    lines
}

/// 第二段两个旋钮（第八节末两行）的敏感性：「借多少」在每条序列上比主窗口与借到刚好够下限那个窗口里涉及 M3 的四行；
/// 「先查哪一对」在每条序列上比主窗口与先查左边那个窗口的六行（Q2p、Q2c、Q3 两种记法；M2、M3 都换了先查的那一对）。
#[must_use]
pub fn segment_two_sensitivity_lines(results: &[SequenceResult]) -> Vec<String> {
    let window_index_of = |result: &SequenceResult, label: &str| -> usize {
        result.windows.iter().position(|window| window.label == label).expect("第二段的三个窗口每条序列都开")
    };
    let mut lines = Vec::new();
    for result in results {
        let (main_window, just_enough_window) = (window_index_of(result, "main"), window_index_of(result, "just_enough"));
        for row in JUDGED_ROWS.iter().filter(|row| matches!(row, JudgedRow::WriteCost { minuend_slot: 2, .. })) {
            let main_label = format!("{}@M3", result.sequence.name);
            let opposite_label = format!("{}@M3_just_enough", result.sequence.name);
            let points = (point_value(result, *row, main_window), point_value(result, *row, just_enough_window));
            lines.push(sensitivity_line("borrow_amount_just_enough", &main_label, &opposite_label, (*row, *row), points.0, points.1));
        }
    }
    for result in results {
        let (main_window, left_first_window) = (window_index_of(result, "main"), window_index_of(result, "left_first"));
        for row in JUDGED_ROWS.iter().filter(|row| matches!(row, JudgedRow::WriteCost { .. })) {
            let main_label = format!("{}@main", result.sequence.name);
            let opposite_label = format!("{}@left_first", result.sequence.name);
            let points = (point_value(result, *row, main_window), point_value(result, *row, left_first_window));
            lines.push(sensitivity_line("check_order_left_first", &main_label, &opposite_label, (*row, *row), points.0, points.1));
        }
    }
    lines
}

/// 线程数：`SINGLEFS_THREAD_CAP`（`scripts/hosts/capped.sh` 设），没设取 1；写错就停。
fn thread_count_from_environment() -> usize {
    match std::env::var("SINGLEFS_THREAD_CAP") {
        Ok(text) => text.parse::<usize>().ok().filter(|threads| *threads >= 1).expect("SINGLEFS_THREAD_CAP 要是正整数"),
        Err(_) => 1,
    }
}

/// 按序列分线程跑，每条序列喂给同一组臂变体、开同一组窗口；结果按登记次序排，与单线程逐字相同（单测盯着）。工作线程 panic（丙类断言红、作废断言红）整段停下。
#[must_use]
pub fn run_sequences(
    sequences: &[RegisteredSequence],
    arms: &[ArmVariant],
    window_triples: &[(&'static str, [usize; 3])],
    threads: usize,
    operations_per_arm_at_most: u64,
) -> Vec<SequenceResult> {
    let next_sequence = std::sync::atomic::AtomicUsize::new(0);
    let mut collected: Vec<(usize, SequenceResult)> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads.max(1))
            .map(|_| {
                scope.spawn(|| {
                    let mut finished = Vec::new();
                    loop {
                        let sequence_index = next_sequence.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        let Some(sequence) = sequences.get(sequence_index) else { break };
                        finished.push((sequence_index, run_sequence(*sequence, arms, window_triples, operations_per_arm_at_most)));
                    }
                    finished
                })
            })
            .collect();
        workers.into_iter().flat_map(|worker| worker.join().expect("工作线程 panic：丙类断言或作废断言红，这一段不出产物")).collect()
    });
    collected.sort_by_key(|(sequence_index, _)| *sequence_index);
    assert_eq!(collected.len(), sequences.len(), "派出去几条序列就收回几条");
    collected.into_iter().map(|(_, result)| result).collect()
}

/// 窗口指纹那一行（第十一节作废条款的读数）与 F6 那一行，两段共用。
fn window_prefix_and_f6_verdict_lines(results: &[SequenceResult]) -> Vec<String> {
    let windows: Vec<&ComparisonWindow> = results.iter().flat_map(|result| result.windows.iter()).collect();
    let equal_windows = windows
        .iter()
        .filter(|window| window.operations_identical && window.prefix_sha256[0] == window.prefix_sha256[1] && window.prefix_sha256[0] == window.prefix_sha256[2])
        .count();
    let m1_not_reached = results.iter().filter(|result| result.trajectories[0].stop == ArmStop::ReachedOperationCap).count();
    vec![
        format!(
            "name=verdict scope=window_prefixes windows={} windows_with_equal_prefixes={equal_windows} all_window_prefixes_equal={}",
            windows.len(),
            equal_windows == windows.len()
        ),
        format!("name=verdict scope=f6_m1_reached points={} m1_not_reached_points={m1_not_reached} m1_reached_on_every_point={}", results.len(), m1_not_reached == 0),
    ]
}

/// 每个窗口每一行的总判定（Q1r 只在第 0 个窗口判：它比的是 M1 与 M2 主，与窗口里第三条臂无关），两段共用。
fn row_verdict_lines(results: &[SequenceResult], window_labels: &[&str]) -> Vec<String> {
    let mut lines = Vec::new();
    for (window_index, window_label) in window_labels.iter().enumerate() {
        for row in JUDGED_ROWS {
            if row == JudgedRow::CapacityRatio && window_index > 0 {
                continue;
            }
            let judged_points = results.iter().filter(|result| !point_value(result, row, window_index).blocks_judgement()).count();
            lines.push(format!(
                "name=verdict scope=row row={} window={window_label} points={} judged_points={judged_points} verdict={}",
                row.name(),
                results.len(),
                overall_verdict(results, row, window_index)
            ));
        }
    }
    lines
}

/// 敏感性行之后那一行挪门槛自证的计数：两点同判的对数、判定不同的对数、挪门槛之后转成不同的对数、两点值逐位相等挪不开的对数（单列，不并进别的计数）。
fn sensitivity_self_proof_line(sensitivity: &[String]) -> String {
    let same_pairs = sensitivity.iter().filter(|line| line.contains(" result=two_points_same")).count();
    let moved_to_unstable = sensitivity.iter().filter(|line| line.contains(" moved_result=unstable")).count();
    let equal_values = sensitivity.iter().filter(|line| line.contains(" moved_result=equal_values_cannot_be_separated")).count();
    let unstable_pairs = sensitivity.iter().filter(|line| line.contains(" result=unstable")).count();
    format!(
        "name=verdict scope=sensitivity_self_proof same_pairs={same_pairs} unstable_pairs={unstable_pairs} moved_to_unstable={moved_to_unstable} equal_values_not_separable={equal_values} every_separable_same_pair_moved_to_unstable={}",
        moved_to_unstable + equal_values == same_pairs
    )
}

/// 第二段序列跑完之后的总判定行（不带 `E7RESULT ` 前缀）：窗口指纹、F6、三个窗口每行总判定、两个旋钮的敏感性与它的挪门槛自证。
#[must_use]
pub fn segment_two_summary_lines(results: &[SequenceResult]) -> Vec<String> {
    let mut lines = window_prefix_and_f6_verdict_lines(results);
    let window_labels: Vec<&str> = SEGMENT_TWO_WINDOWS.iter().map(|(label, _)| *label).collect();
    lines.extend(row_verdict_lines(results, &window_labels));
    let sensitivity = segment_two_sensitivity_lines(results);
    let self_proof = sensitivity_self_proof_line(&sensitivity);
    lines.extend(sensitivity);
    lines.push(self_proof);
    lines
}

/// 序列跑完之后的总判定行（不带 `E7RESULT ` 前缀）：窗口指纹、F5、F6、每行总判定、敏感性与它的挪门槛自证。
#[must_use]
pub fn summary_lines(results: &[SequenceResult]) -> Vec<String> {
    let mut lines = window_prefix_and_f6_verdict_lines(results);
    let main_point_borrows: u64 = results.iter().filter(|result| result.sequence.is_main_sampling_point).map(|result| result.trajectories[2].borrows).sum();
    lines.push(format!("name=verdict scope=f5_m3_borrows_on_main_points main_point_m3_borrows={main_point_borrows} m3_borrowed_on_some_main_point={}", main_point_borrows > 0));
    let window_labels: Vec<&str> = SEGMENT_ONE_WINDOWS.iter().map(|(label, _)| *label).collect();
    lines.extend(row_verdict_lines(results, &window_labels));
    let sensitivity = sensitivity_lines(results);
    let self_proof = sensitivity_self_proof_line(&sensitivity);
    lines.extend(sensitivity);
    lines.push(self_proof);
    lines
}

/// 第一段整段：config、锚点与阳性对照（不全对就到此为止）、22 条序列、总判定。交回产物行（不带前缀）与阳性对照过没过。
#[must_use]
pub fn segment_one_output(threads: usize) -> (Vec<String>, bool) {
    let mut lines = vec![format!(
        "name=config experiment=E176 segment=one sequences={} arms={} leaf_record_capacity={LEAF_RECORD_CAPACITY} root_node_leaf_container_capacity={ROOT_NODE_LEAF_CONTAINER_CAPACITY} operations_per_arm_at_most={OPERATIONS_PER_ARM_AT_MOST} unlinks_for_a_judgement_at_least={UNLINKS_IN_THE_WINDOW_FOR_A_JUDGEMENT_AT_LEAST}",
        registered_sequences().len(),
        SEGMENT_ONE_ARMS.map(|arm| arm.name).join(",")
    )];
    let anchors_ok = push_anchor_lines(&mut lines, &anchor_readings());
    if !anchors_ok {
        return (lines, false);
    }
    let results = run_sequences(&registered_sequences(), &SEGMENT_ONE_ARMS, &SEGMENT_ONE_WINDOWS, threads, OPERATIONS_PER_ARM_AT_MOST);
    for result in &results {
        assert_window_prefixes_agree(result);
        lines.extend(sequence_lines(result));
    }
    lines.extend(summary_lines(&results));
    (lines, true)
}

/// 锚点行与阳性对照那一行判决，两段共用；交回登记了答案的格是不是全对。
fn push_anchor_lines(lines: &mut Vec<String>, readings: &[AnchorReading]) -> bool {
    for reading in readings {
        let expected = match reading.expected {
            Some(expected) if expected == reading.observed => "matches=true".to_string(),
            Some(expected) => format!("matches=false expected={}", expected.replace(' ', ",")),
            None => "matches=unregistered".to_string(),
        };
        lines.push(format!("name=anchor cell={} arm={} {} {expected}", reading.cell, reading.arm, reading.observed));
    }
    let registered = readings.iter().filter(|reading| reading.expected.is_some()).count();
    let matching = readings.iter().filter(|reading| reading.expected.is_some() && reading.matches()).count();
    let anchors_ok = registered == matching;
    lines.push(format!("name=verdict scope=positive_controls anchors_registered={registered} anchors_matching={matching} all_registered_anchors_match={anchors_ok}"));
    anchors_ok
}

/// 第二段整段（跑前登记第五节第二段那一行、修订 3）：config、锚点与阳性对照（不全对就到此为止）、22 条序列 × 六个臂变体三个窗口、总判定。
#[must_use]
pub fn segment_two_output(threads: usize) -> (Vec<String>, bool) {
    let mut lines = vec![format!(
        "name=config experiment=E176 segment=two sequences={} arms={} windows={} leaf_record_capacity={LEAF_RECORD_CAPACITY} root_node_leaf_container_capacity={ROOT_NODE_LEAF_CONTAINER_CAPACITY} operations_per_arm_at_most={OPERATIONS_PER_ARM_AT_MOST} unlinks_for_a_judgement_at_least={UNLINKS_IN_THE_WINDOW_FOR_A_JUDGEMENT_AT_LEAST}",
        registered_sequences().len(),
        SEGMENT_TWO_ARMS.map(|arm| arm.name).join(","),
        SEGMENT_TWO_WINDOWS.map(|(label, _)| label).join(",")
    )];
    let anchors_ok = push_anchor_lines(&mut lines, &segment_two_anchor_readings());
    if !anchors_ok {
        return (lines, false);
    }
    let results = run_sequences(&registered_sequences(), &SEGMENT_TWO_ARMS, &SEGMENT_TWO_WINDOWS, threads, OPERATIONS_PER_ARM_AT_MOST);
    for result in &results {
        assert_window_prefixes_agree(result);
        lines.extend(sequence_lines(result));
    }
    lines.extend(segment_two_summary_lines(&results));
    (lines, true)
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    let threads = thread_count_from_environment();
    let (lines, positive_controls_ok) = match mode.as_str() {
        "segment-one" => {
            eprintln!("E176 第一段：{threads} 个线程（线程数不进产物）");
            segment_one_output(threads)
        }
        "segment-two" => {
            eprintln!("E176 第二段：{threads} 个线程（线程数不进产物）");
            segment_two_output(threads)
        }
        _ => {
            eprintln!("用法：e176-inode-leaf-merge-capacity-and-rewrites segment-one | segment-two（线程数读 SINGLEFS_THREAD_CAP）");
            std::process::exit(2);
        }
    };
    let emitted = lines.len() + 1;
    for line in lines {
        println!("E7RESULT {line}");
    }
    println!("E7RESULT name=done emitted={emitted}");
    if !positive_controls_ok {
        eprintln!("阳性对照或锚点有一格对不上（第十一节作废）：见 name=anchor 那几行的 matches=false");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_shot_digest(bytes: &[u8]) -> String {
        let mut hasher = StreamingSha256::new();
        hasher.update(bytes);
        hasher.hexadecimal_digest()
    }

    #[test]
    fn sha256_matches_the_published_test_vectors_whether_fed_at_once_or_in_pieces() {
        assert_eq!(one_shot_digest(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(one_shot_digest(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        let two_blocks = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        assert_eq!(one_shot_digest(two_blocks), "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");
        let mut in_pieces = StreamingSha256::new();
        for piece in two_blocks.chunks(9) {
            in_pieces.update(piece);
        }
        assert_eq!(in_pieces.hexadecimal_digest(), one_shot_digest(two_blocks), "分块喂与一次喂同一个摘要");
    }

    #[test]
    fn every_registered_anchor_matches_the_independent_reference() {
        let readings = anchor_readings();
        let registered = readings.iter().filter(|reading| reading.expected.is_some()).count();
        assert_eq!(registered, 26, "第七节登记的锚点格：A1 四臂、A2 三臂、A3 五格、A4 三臂、A5 四格、A6 三臂、C1、crates 单测三格");
        assert_eq!(readings.len(), 26 + 5, "另有 f = 59 那个变体在 A2 … A6 上只报不判的五格");
        for reading in &readings {
            assert!(reading.matches(), "锚点 {} 臂 {}：读到 {}，登记 {:?}", reading.cell, reading.arm, reading.observed, reading.expected);
        }
    }

    #[test]
    fn a1_refuses_the_insert_that_would_open_the_136th_leaf() {
        let observed = anchor_a1(ARM_M1);
        assert_eq!(observed, format!("refused_insert={} live={} containers={}", 31_455 + 1, 233 * 135, 135));
    }

    #[test]
    fn a2_one_record_per_leaf_leaves_m1_only_134_plus_233_live_inodes() {
        let observed = anchor_a2(ARM_M1);
        assert_eq!(observed, format!("refused_insert={} live={} containers=135 merges=0 borrows=0", 31_455 + 233, 134 + 233));
    }

    #[test]
    fn main_random_sequence_feeds_every_arm_the_same_operation_prefix() {
        let sequence = registered_sequences()[0];
        assert_eq!(sequence.name, "wr_p055_seed1");
        let result = run_sequence(sequence, &SEGMENT_ONE_ARMS, &SEGMENT_ONE_WINDOWS, OPERATIONS_PER_ARM_AT_MOST);
        assert_window_prefixes_agree(&result);
        let main_window = &result.windows[0];
        assert!(main_window.unlinks > 0, "主窗口里有删除（读得到的数，不是空窗口）");
        assert!(matches!(result.trajectories[0].stop, ArmStop::Refused { .. }), "W-R p = 0.55 上 M1 撞得到上限");
    }

    fn assert_three_threads_render_the_same_lines_as_one(arms: &[ArmVariant], windows: &[(&'static str, [usize; 3])]) {
        let sequences: Vec<RegisteredSequence> = registered_sequences()
            .into_iter()
            .filter(|sequence| ["wr_p055_seed2", "ws_a1000_d500_smallest_first", "wa_keep1of2_a1000", "wa_keep3of4_a1000"].contains(&sequence.name))
            .collect();
        assert_eq!(sequences.len(), 4);
        let render = |results: Vec<SequenceResult>| results.iter().flat_map(sequence_lines).collect::<Vec<_>>().join("\n");
        let single = render(run_sequences(&sequences, arms, windows, 1, 200_000));
        let multi = render(run_sequences(&sequences, arms, windows, 3, 200_000));
        assert!(!single.is_empty());
        assert_eq!(single, multi, "按序列分线程跑，合并结果与单线程逐字相同（{} 个臂变体）", arms.len());
    }

    #[test]
    fn multithreaded_and_single_threaded_runs_are_byte_identical() {
        assert_three_threads_render_the_same_lines_as_one(&SEGMENT_ONE_ARMS, &SEGMENT_ONE_WINDOWS);
    }

    #[test]
    fn segment_two_multithreaded_and_single_threaded_runs_are_byte_identical() {
        assert_three_threads_render_the_same_lines_as_one(&SEGMENT_TWO_ARMS, &SEGMENT_TWO_WINDOWS);
    }

    #[test]
    fn sequential_workload_creates_a_round_then_deletes_the_oldest() {
        let mut generator = WorkloadGenerator::new(Workload::SequentialDelete { inserted_per_round: 3, deleted_per_round: 2, order: DeletionOrder::SmallestFirst }, 0);
        let operations: Vec<Operation> = (0..7).map(|_| generator.next_operation()).collect();
        assert_eq!(
            operations,
            vec![Operation::Create(1), Operation::Create(2), Operation::Create(3), Operation::Unlink(1), Operation::Unlink(2), Operation::Create(4), Operation::Create(5)]
        );
        let mut newest_first = WorkloadGenerator::new(Workload::SequentialDelete { inserted_per_round: 3, deleted_per_round: 2, order: DeletionOrder::LargestFirst }, 0);
        let newest: Vec<Operation> = (0..5).map(|_| newest_first.next_operation()).collect();
        assert_eq!(newest[3..], [Operation::Unlink(3), Operation::Unlink(2)]);
    }

    #[test]
    fn alternate_workload_deletes_the_registered_share_of_each_batch() {
        let mut half = WorkloadGenerator::new(Workload::AlternateDelete { batch_size: 4, kept: KeptInEachBatch::OneIn(2) }, 0);
        let operations: Vec<Operation> = (0..8).map(|_| half.next_operation()).collect();
        assert_eq!(operations[4..], [Operation::Unlink(2), Operation::Unlink(4), Operation::Create(5), Operation::Create(6)]);
        let mut three_quarters = WorkloadGenerator::new(Workload::AlternateDelete { batch_size: 8, kept: KeptInEachBatch::ThreeInFour }, 0);
        let operations: Vec<Operation> = (0..11).map(|_| three_quarters.next_operation()).collect();
        assert_eq!(operations[8..], [Operation::Unlink(4), Operation::Unlink(8), Operation::Create(9)]);
    }

    #[test]
    fn random_workload_starts_with_a_create_and_only_unlinks_live_inodes() {
        let mut generator = WorkloadGenerator::new(Workload::RandomInsertDelete { insert_probability_percent: 55 }, 1);
        let mut live = BTreeSet::new();
        for _ in 0..10_000 {
            match generator.next_operation() {
                Operation::Create(inode) => assert!(live.insert(inode), "号不复用"),
                Operation::Unlink(inode) => assert!(live.remove(&inode), "只删活着的号"),
            }
        }
        assert!(!live.is_empty());
    }

    fn two_leaf_tree(policy: MergePolicy, left: Vec<u64>, right_number: u64, right: Vec<u64>) -> InodeLeafTree {
        InodeLeafTree::from_containers(
            policy,
            LeafContainerLimit::OneRootNode,
            vec![LeafContainer { container_number: left[0], inode_numbers: left }, LeafContainer { container_number: right_number, inode_numbers: right }],
        )
    }

    #[test]
    fn structural_check_turns_red_on_each_broken_shape() {
        let overlapping = two_leaf_tree(MergePolicy::DropEmptyOnly, vec![1, 5], 3, vec![4, 6]);
        assert!(check_structure_after_publish(&overlapping, &[]).is_err(), "左片最大号 5 ≥ 右片最小号 4 要红");
        let third_sentence = two_leaf_tree(MergePolicy::DropEmptyOnly, vec![1, 5], 3, vec![6, 7]);
        assert!(check_structure_after_publish(&third_sentence, &[]).is_err(), "M1 上左片最大号 5 ≥ 右片容器号 3 要红");
        let borrowed = two_leaf_tree(ARM_M3_MAIN.policy, ascending_run(1, 120), 100, ascending_run(121, 240));
        assert_eq!(check_structure_after_publish(&borrowed, &[]), Ok(1), "M3 上同一形态只计数：1 对");
        let thin_pair = two_leaf_tree(ARM_M2_MAIN.policy, ascending_run(1, 100), 101, ascending_run(101, 233));
        assert!(check_structure_after_publish(&thin_pair, &[]).is_err(), "M2 上相邻两片之和 233 < 234 要红");
        let unsorted = two_leaf_tree(MergePolicy::DropEmptyOnly, vec![1, 3, 2], 10, vec![10]);
        assert!(check_structure_after_publish(&unsorted, &[0]).is_err(), "动过的那片片内不升序要红");
    }

    #[test]
    fn verdict_thresholds_are_inclusive_where_the_registration_says_so() {
        let point = |numerator: i128, denominator: i128| PointValue { value: Some(Rational::new(numerator, denominator)), note: "" };
        let capacity = registered_thresholds(JudgedRow::CapacityRatio);
        assert_eq!(point_verdict(JudgedRow::CapacityRatio, point(90, 100), capacity), "no_merge_capacity_cost_small");
        assert_eq!(point_verdict(JudgedRow::CapacityRatio, point(89, 100), capacity), "no_flip");
        assert_eq!(point_verdict(JudgedRow::CapacityRatio, point(50, 100), capacity), "no_flip");
        assert_eq!(point_verdict(JudgedRow::CapacityRatio, point(49, 100), capacity), "merge_buys_double_capacity");
        let delta2p = JUDGED_ROWS[1];
        let delta2c = JUDGED_ROWS[3];
        let write_cost = registered_thresholds(delta2p);
        assert_eq!(point_verdict(delta2p, point(10, 100), write_cost), "negligible");
        assert_eq!(point_verdict(delta2p, point(-3, 1), write_cost), "negligible");
        assert_eq!(point_verdict(delta2p, point(11, 100), write_cost), "no_flip");
        assert_eq!(point_verdict(delta2p, point(1, 1), write_cost), "significant");
        assert_eq!(point_verdict(delta2c, point(1, 1), write_cost), "no_flip", "Δ2c 的「≥ 1.00」一侧不算判据");
        let thin = PointValue { value: Some(Rational::new(5, 1)), note: "insufficient_sample" };
        assert_eq!(point_verdict(delta2p, thin, write_cost), "insufficient_sample");
    }

    #[test]
    fn moving_a_threshold_between_two_same_verdict_points_splits_them() {
        let point = |numerator: i128, denominator: i128| PointValue { value: Some(Rational::new(numerator, denominator)), note: "" };
        let cases = [
            (JudgedRow::CapacityRatio, point(95, 100), point(99, 100)),
            (JudgedRow::CapacityRatio, point(60, 100), point(70, 100)),
            (JudgedRow::CapacityRatio, point(10, 100), point(30, 100)),
            (JUDGED_ROWS[1], point(1, 100), point(5, 100)),
            (JUDGED_ROWS[1], point(30, 100), point(60, 100)),
            (JUDGED_ROWS[1], point(2, 1), point(3, 1)),
            (JUDGED_ROWS[3], point(2, 1), point(3, 1)),
        ];
        for (row, main, opposite) in cases {
            let registered = registered_thresholds(row);
            assert_eq!(point_verdict(row, main, registered), point_verdict(row, opposite, registered), "取样的两点在登记门槛下同判");
            let (_, moved_main, moved_opposite) = moved_threshold_verdicts(row, main, opposite).expect("两点值不同");
            assert_ne!(moved_main, moved_opposite, "{} 挪门槛之后两点要分开", row.name());
        }
        assert!(moved_threshold_verdicts(JUDGED_ROWS[1], point(1, 10), point(1, 10)).is_none(), "两点值相等时挪不开");
    }

    #[test]
    fn rational_decimal_rounds_half_away_from_zero() {
        assert_eq!(Rational::new(4292 * 100, 430 * 100).decimal(2), "9.98");
        assert_eq!(Rational::new(1, 8).decimal(2), "0.13");
        assert_eq!(Rational::new(-1, 8).decimal(2), "-0.13");
        assert_eq!(Rational::new(0, 7).decimal(4), "0.0000");
        assert_eq!(Rational::new(367, 31_455).decimal(4), "0.0117");
    }

    #[test]
    fn merge_and_borrow_arms_keep_adjacent_pairs_above_one_leaf_through_a_random_run() {
        for arm in [ARM_M2_MAIN, ARM_M3_MAIN, ARM_M3_FLOOR_59, ARM_M3_JUST_ENOUGH_FOR_THE_FLOOR, ARM_M2_LEFT_PAIR_FIRST, ARM_M3_LEFT_PAIR_FIRST] {
            let mut tree = InodeLeafTree::empty(arm.policy, LeafContainerLimit::OneRootNode);
            let mut generator = WorkloadGenerator::new(Workload::RandomInsertDelete { insert_probability_percent: 60 }, 7);
            for _ in 0..50_000 {
                let touched = match generator.next_operation() {
                    Operation::Create(inode) => {
                        assert_eq!(tree.insert(inode), InsertOutcome::Accepted, "五万步到不了 135 片");
                        vec![tree.containers.len() - 1]
                    }
                    Operation::Unlink(inode) => {
                        let rewrites = tree.remove(inode);
                        assert!(rewrites.content_notation_count() <= content_notation_bound_per_removal(&tree));
                        rewrites.content_rewritten_positions
                    }
                };
                check_structure_after_publish(&tree, &touched).unwrap_or_else(|violation| panic!("臂 {}：{violation}", arm.name));
            }
        }
    }

    #[test]
    fn segment_two_windows_put_each_registered_variant_against_m1() {
        let names = |triple: [usize; 3]| triple.map(|arm_index| SEGMENT_TWO_ARMS[arm_index].name);
        assert_eq!(SEGMENT_TWO_WINDOWS.map(|(label, _)| label), ["main", "just_enough", "left_first"]);
        assert_eq!(names(SEGMENT_TWO_WINDOWS[0].1), ["M1", "M2", "M3"]);
        assert_eq!(names(SEGMENT_TWO_WINDOWS[1].1), ["M1", "M2", "M3_just_enough"]);
        assert_eq!(names(SEGMENT_TWO_WINDOWS[2].1), ["M1", "M2_left_first", "M3_left_first"]);
        assert_eq!(
            ARM_M3_LEFT_PAIR_FIRST.policy,
            MergePolicy::AbsorbRightAndBorrowFromRight { order: NeighbourCheckOrder::LeftPairFirst, floor: 117, amount: BorrowAmount::HalfTheDifference }
        );
        assert_eq!(
            ARM_M3_JUST_ENOUGH_FOR_THE_FLOOR.policy,
            MergePolicy::AbsorbRightAndBorrowFromRight { order: NeighbourCheckOrder::RightPairFirst, floor: 117, amount: BorrowAmount::JustEnoughForTheFloor }
        );
        assert_eq!(ARM_M2_LEFT_PAIR_FIRST.policy, MergePolicy::AbsorbRightWhenTheyFit { order: NeighbourCheckOrder::LeftPairFirst });
    }

    #[test]
    fn segment_two_anchors_judge_a1_for_the_three_variants_and_only_report_the_rest() {
        let readings = segment_two_anchor_readings();
        let registered = readings.iter().filter(|reading| reading.expected.is_some()).count();
        assert_eq!(registered, 26 + 3, "第一段登记的 26 格，加三个变体的 A1");
        assert_eq!(readings.len(), 31 + 3 + 3 + 2 + 3 + 2 + 3, "第一段 31 格，加三个变体 A1、A2、A3（两格）、A4、A5（两格）、A6");
        for reading in &readings {
            assert!(reading.matches(), "锚点 {} 臂 {}：读到 {}，登记 {:?}", reading.cell, reading.arm, reading.observed, reading.expected);
        }
    }

    fn three_leaf_tree_whose_middle_leaf_fits_with_either_neighbour(arm: ArmVariant) -> InodeLeafTree {
        InodeLeafTree::from_containers(
            arm.policy,
            LeafContainerLimit::OneRootNode,
            vec![
                LeafContainer { container_number: 1, inode_numbers: ascending_run(1, 117) },
                LeafContainer { container_number: 234, inode_numbers: ascending_run(234, 350) },
                LeafContainer { container_number: 467, inode_numbers: ascending_run(467, 583) },
            ],
        )
    }

    fn removal_outcome(arm: ArmVariant) -> (Vec<usize>, Vec<u64>, Vec<usize>, Vec<usize>) {
        let mut tree = three_leaf_tree_whose_middle_leaf_fits_with_either_neighbour(arm);
        let rewrites = tree.remove(350);
        check_structure_after_publish(&tree, &rewrites.content_rewritten_positions).unwrap_or_else(|violation| panic!("臂 {}：{violation}", arm.name));
        let sizes = tree.containers.iter().map(|container| container.inode_numbers.len()).collect();
        let numbers = tree.containers.iter().map(|container| container.container_number).collect();
        (sizes, numbers, rewrites.content_rewritten_positions, rewrites.position_rewritten_positions)
    }

    /// 右边那一对先查：中间那片吸收右邻，摘的是最右一格，位置记法只记中间那片。
    fn assert_the_middle_leaf_absorbs_its_right_neighbour(arm: ArmVariant) {
        assert_eq!(removal_outcome(arm), (vec![117, 233], vec![1, 234], vec![1], vec![1]), "臂 {}", arm.name);
    }

    /// 左边那一对先查：左邻吸收中间那片，摘的是中间一格，右邻位置左移，位置记法比右边先查多 1 片。
    fn assert_the_left_neighbour_absorbs_the_middle_leaf(arm: ArmVariant) {
        assert_eq!(removal_outcome(arm), (vec![233, 117], vec![1, 467], vec![0], vec![0, 1]), "臂 {}", arm.name);
    }

    #[test]
    fn m2_checking_the_right_pair_first_lets_the_middle_leaf_absorb_its_right_neighbour() {
        assert_the_middle_leaf_absorbs_its_right_neighbour(ARM_M2_MAIN);
    }

    #[test]
    fn m3_checking_the_right_pair_first_lets_the_middle_leaf_absorb_its_right_neighbour() {
        assert_the_middle_leaf_absorbs_its_right_neighbour(ARM_M3_MAIN);
    }

    #[test]
    fn m2_checking_the_left_pair_first_lets_the_left_neighbour_absorb_the_middle_leaf_and_shifts_one_more_position() {
        assert_the_left_neighbour_absorbs_the_middle_leaf(ARM_M2_LEFT_PAIR_FIRST);
    }

    #[test]
    fn m3_checking_the_left_pair_first_lets_the_left_neighbour_absorb_the_middle_leaf_and_shifts_one_more_position() {
        assert_the_left_neighbour_absorbs_the_middle_leaf(ARM_M3_LEFT_PAIR_FIRST);
    }

    #[test]
    fn segment_two_sensitivity_pairs_four_m3_rows_for_the_borrow_amount_and_six_rows_for_the_check_order() {
        let sequence = registered_sequences().into_iter().find(|sequence| sequence.name == "wa_keep1of2_a1000").expect("登记里有这一点");
        let result = run_sequence(sequence, &SEGMENT_TWO_ARMS, &SEGMENT_TWO_WINDOWS, 200_000);
        let lines = segment_two_sensitivity_lines(std::slice::from_ref(&result));
        let knob_and_row = |line: &String| -> (String, String) {
            let field = |key: &str| line.split(' ').find_map(|part| part.strip_prefix(key)).expect("每一行都有这个字段").to_string();
            (field("knob="), field("row="))
        };
        let observed: Vec<(String, String)> = lines.iter().map(knob_and_row).collect();
        let expected: Vec<(String, String)> = [
            ("borrow_amount_just_enough", "Q2p_delta3"),
            ("borrow_amount_just_enough", "Q2c_delta3"),
            ("borrow_amount_just_enough", "Q3p_delta32"),
            ("borrow_amount_just_enough", "Q3c_delta32"),
            ("check_order_left_first", "Q2p_delta2"),
            ("check_order_left_first", "Q2p_delta3"),
            ("check_order_left_first", "Q2c_delta2"),
            ("check_order_left_first", "Q2c_delta3"),
            ("check_order_left_first", "Q3p_delta32"),
            ("check_order_left_first", "Q3c_delta32"),
        ]
        .map(|(knob, row)| (knob.to_string(), row.to_string()))
        .to_vec();
        assert_eq!(observed, expected);
        let left_first_delta2 = point_value(&result, JUDGED_ROWS[1], 2);
        assert!(lines[4].contains(&format!(" opposite_value={} ", value_text(left_first_delta2))), "先查左边那一点取的是 left_first 窗口：{}", lines[4]);
        assert!(lines[4].contains(" opposite=wa_keep1of2_a1000@left_first "), "{}", lines[4]);
    }
}
