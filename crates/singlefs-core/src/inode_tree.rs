//! inode 树的叶容器：一容器 233 条 140 字节记录，满了就在末尾分裂（里程碑「覆盖写、释放、回退与复用」并行线三的写路径）。
//!
//! 压着它的条款是 D8（核心索引结构） 已定项 6 的三条写路径纪律，逐字：
//! - 「inode 号在每一条时间线上单调（回退回到 R_old 的树与它的水位，之后发的号仍大于那棵树里每一个 key）⇒
//!   插入永远落在最右的叶 ⇒ 分裂只发生在最右叶，**分裂点取末尾**」；
//! - 「左半保留全部原有记录与身份，右半从触发分裂的那条新记录起，**右半容器号 = 那条新记录的 inode 号**，
//!   出生代 = 本次发布的 checkpoint_txg，**出生树 = 执行这次分裂的那棵树**（分裂是新建，与「COW 重写身份不变」是两件事）」；
//! - 「第一个容器的号 = 第一条记录的 inode 号」。
//!
//! 这三条今天没有会失败的检查（C116（叶容器的分裂 / 合并纪律没有会失败的检查）），
//! [09-inode树结构.md](../../singlefs-checker/invariants/09-inode树结构.md) 逐字把它们排除在 I-9（inode 树结构） 之外：
//! 「分裂 / 合并的身份传递……与合并时机是操作断言，对着一个镜像判不了，走单测 / 模型对拍与崩溃点重放」。
//! 这个模块就是那个能判的地方：它只算「这次发布之后树里是哪几片容器、各装哪些记录、哪几片要重写」，
//! 不发一个写、不动分配器、不碰落点，所以三条纪律在这里是纯函数的后置条件，单测判得了。
//!
//! **删除的落法**（[`remove_record_from_leaf_containers`]，unlink 最后一批那次发布）：删掉那一条；之后照 E176（inode 树叶合并三候选的容量与写代价）
//! M3 那一臂的写路径策略合并（D8（核心索引结构） 已定项 6「左吸收右与左从右借两样都做」，合并时机是写路径策略）：
//! - 那一片删空了就整片不落盘（「记录数为 0 的类型 2 容器不落盘」），它的左右邻都在时从左邻起按「先查右邻、再查左邻，两片之和 ≤ 233 就左吸收右」
//!   一直并到没有一对装得下；
//! - 没删空就从它起照同一条规则并；一次都没并、它的记录数 < 117（⌈233 ÷ 2⌉）、它有右邻时，从右邻借最小的 ⌊(右 − 左) ÷ 2⌋ 条
//!   （均分：借完两片条数相差不超过 1），右片保留身份、分隔 key 改成它这一版的最小 key（N1），再从右片起照同一条规则并。
//!
//! 合并保留左半身份；被吸收的右半与删空的那一片身份消失，交回给发布路径写退役记录（[`InodeLeafContainersAfterThisPublish::retired`]）。
//! **只重写内容变了的叶**：身份消失的那几片右边的每一片位置左移，但内容、身份、分隔 key 都没变，这次照抄上一版那一份（落点、字节、指针都不动），
//! 只是在这一版里换到左边的叶序上（[`InodeLeafContainerOrigin::CarriedFrom`]）——删空最左一片那次新写的槽数不随右边片数涨
//!
//! 这份文件的代码都在 `verus!` 里，由 Verus 直接验（里程碑四出口 5 容器小树那一簇）。重写保持「key 到条目」：
//! [`write_records_into_leaf_containers`] 的 `ensures` 写明这次之后全部叶容器的记录依次连起来，就是上一版的记录逐条照这次给的记录改写
//! （号已在树里的原地换掉、比树里都大的接在末尾，[`records_after_writing`]）；[`remove_record_from_leaf_containers`] 的 `ensures` 写明它就是上一版的记录
//! 去掉那一条（合并、借、删空摘掉都不改记录的次序与内容）。两样都还写明这次之后记录按 inode 号严格升序、每片 1 到 233 条，照抄的那几片与上一版那一份逐字节相同。
//! 树内找号、在重写清单里找位置用同一语义的手写查找（严格升序时找到的位置与插入点唯一，与 std 的 `binary_search` 交回相同）；
//! 退役清单每退役一片就插进它的位置（同序号的排在已有的之后），交回的清单与在末尾按上一版叶序 `sort_by_key`（稳定排序）排出来的相同。
//! `requires` 只约束经 Verus 验过的调用方，没验的调用方违反时照旧在原来那一处 panic 或得到与原来相同的结果。

use singlefs_format::{INODE_INTERNAL_ENTRY, INODE_LEAF_RECORDS};
use vstd::prelude::*;

use crate::address::{CheckpointTxg, InodeNumber, TreeIdentifier};
use crate::records::InodeRecord;
use crate::unit::{index_node_entry_capacity, PackedIdentity, PACKED_TYPE_INODE};

mod removing_a_record_from_leaf_containers;
mod writing_records_into_leaf_containers;
pub use removing_a_record_from_leaf_containers::*;
pub use writing_records_into_leaf_containers::*;

verus! {

/// inode 树里第几片叶容器，按叶序（= key 升序）从 0 数。
///
/// 它是**树里的位置**，不是容器号：容器号是首次插入它的那条记录的 inode 号（D8（核心索引结构） 已定项 6），
/// 位置随左边有没有别的容器而变，两者不是一个量。新容器只在最右端出现；一片删空或被吸收时它右边的每一片左移，
/// 那一次发布照抄挪了位置的几片、把它们换到这一版的叶序上（[`InodeLeafContainerOrigin::CarriedFrom`] 记着上一版的叶序），
/// 所以位置照样当这一版里的角色序号用，照抄时按上一版的叶序取上一版那一份。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InodeLeafContainerIndexInTree(pub u32);

impl InodeLeafContainerIndexInTree {
    /// 最左那一片。树只有一片叶时就是它（新池新建文件那一档）。
    pub const LEFTMOST: Self = Self(0);

    /// 当下标用：一棵树的容器数不会超过一个码 2 根装得下的条目数（135），`usize` 一定装得下。
    #[must_use]
    pub fn position(self) -> (position: usize)
        ensures
            position == self.0,
    {
        usize::try_from(self.0).expect("叶容器序号小于一个码 2 根的条目数")
    }

    /// 第几片，从 0 数。
    ///
    /// # Panics
    /// 序号装不进 u32（`requires` 排除的那一种）：一棵树的容器数不超过一个码 2 根的条目数。
    #[must_use]
    pub fn of_position(position: usize) -> (index: Self)
        requires
            position <= u32::MAX,
        ensures
            index.0 == position,
    {
        Self(u32::try_from(position).expect("叶容器序号小于一个码 2 根的条目数"))
    }
}

/// 一片 inode 叶容器这一版装的东西：身份四段（出生树、打包记录类型、容器号、容器出生代）、它在父节点条目里的分隔 key、
/// 它的记录，记录按 inode 号升序。
#[derive(Clone, Debug, PartialEq, Eq)]
#[verifier::allow(autoderive_clone_without_spec)]
pub struct InodeLeafContainer {
    pub identity: PackedIdentity,
    /// 父节点里这片容器那一条条目的分隔 key（D8（核心索引结构） 已定项 6「分隔 key = 该孩子创建时的最小 key，左从右借的那次发布改成
    /// 被借的右孩子这一版的最小 key」）：诞生时是容器号，只有被左邻借走最小那几条的那次发布改成它这一版的最小 key，之后照抄。
    pub separator_key_in_the_parent_entry: u64,
    pub records: Vec<InodeRecord>,
}

impl InodeLeafContainer {
    /// 容器号（身份里那 8 字节）：首次插入它的那条记录的 inode 号。
    #[must_use]
    pub fn container_number(&self) -> (container_number: InodeNumber)
        ensures
            container_number.0 == self.identity.container,
    {
        InodeNumber(self.identity.container)
    }

    /// 它装的最小 inode 号；空容器没有（记录数为 0 的类型 2 容器不落盘，D8（核心索引结构） 已定项 6）。
    #[allow(clippy::manual_map, reason = "Verus 推不出 Option::map 里闭包交回的值，写成 match")]
    #[must_use]
    pub fn smallest_inode_number(&self) -> (smallest: Option<InodeNumber>)
        ensures
            self.records.len() == 0 ==> smallest is None,
            self.records.len() > 0 ==> smallest == Some(InodeNumber(self.records@[0].inode)),
    {
        match self.records.first() {
            Some(record) => Some(InodeNumber(record.inode)),
            None => None,
        }
    }

    /// 它装的最大 inode 号。
    #[allow(clippy::manual_map, reason = "Verus 推不出 Option::map 里闭包交回的值，写成 match")]
    #[must_use]
    pub fn largest_inode_number(&self) -> (largest: Option<InodeNumber>)
        ensures
            self.records.len() == 0 ==> largest is None,
            self.records.len() > 0 ==> largest == Some(InodeNumber(self.records@.last().inode)),
    {
        match self.records.last() {
            Some(record) => Some(InodeNumber(record.inode)),
            None => None,
        }
    }

    /// 这片容器在父节点里的分隔 key：D8（核心索引结构） 已定项 6 逐字「**分隔 key** = 该孩子创建时的最小 key，左从右借的那次发布改成
    /// 被借的右孩子这一版的最小 key」。创建时的最小 key 就是容器号——第一个容器的号 = 第一条记录的 inode 号，分裂出来的右半容器号 =
    /// 触发分裂那条新记录的 inode 号。**别的发布不取今天的最小 key**：容器被 COW 重写、记录增删之后今天的最小 key 可以比上一次定下的大，
    /// 而分隔 key 不许跟着变（跟着变就等于改父节点里的路由）；只有被借的那一次随借走的那几条改（[`remove_record_from_leaf_containers`]）。
    #[must_use]
    pub fn separator_key(&self) -> (separator_key: u64)
        ensures
            separator_key == self.separator_key_in_the_parent_entry,
    {
        self.separator_key_in_the_parent_entry
    }

    /// 这片容器还能再装几条：一容器 233 条（D8（核心索引结构） 已定项 6，`INODE_LEAF_RECORDS`）。
    ///
    /// # Panics
    /// 这片容器已经装了多于 233 条（`requires` 排除的那一种）。容器里的记录只有两个来处，两处都装不到第 234 条：
    /// 写者往里追加只经 [`write_records_into_leaf_containers`]，它只在这个函数报「还装得下」时才追加；
    /// 盘上读来的（可写挂载重建上一版，`recovery::rebuild_version`）由读者判过记录宽 140
    /// （`recovery::inode_leaf_container_judged_against_its_entry`）、记录区不越过单元末尾（`unit::parse_packed_unit`），
    /// 一片至多 ⌊(32768 − 107 − 29) ÷ 140⌋ = 233 条（C476 普查「缺口」表 `inode_tree.rs:88` 那一行；这条算术由单测
    /// `packed_unit_holds_no_more_inode_records_than_the_leaf_record_limit` 盯着）。装到了说明这两道之一被绕过了，
    /// 不许当成「还能装 0 条」往下走。
    #[must_use]
    pub fn free_record_slots(&self) -> (free_record_slots: u64)
        requires
            self.records.len() <= 233,
        ensures
            free_record_slots == 233 - self.records.len(),
    {
        INODE_LEAF_RECORDS
            .checked_sub(u64::try_from(self.records.len()).expect("一容器 233 条"))
            .expect(
                "一容器最多 233 条记录（D8 已定项 6）：写者只经 write_records_into_leaf_containers 追加（分裂纪律），盘上读来的由 \
                 recovery::inode_leaf_container_judged_against_its_entry 判过记录宽 140、unit::parse_packed_unit 判过记录区不越过单元末尾",
            )
    }
}

/// inode 树的 key 宽：inode 号 8 字节（D8（核心索引结构） 已定项 6：叶记录的 key、根条目的分隔 key 都是 inode 号）。
/// 写侧装根、读侧核根（`crate::recovery` 的 `key_width_for_kind`、`crate::mounted_read`）都引这一个。
pub const INODE_KEY_WIDTH_IN_BYTES: usize = 8;

/// 一个码 2 根装得下几条 inode 内部条目：扇出 135（D8（核心索引结构） 已定项 6：分隔 key 8 + 身份引用 26 + 子指针 86 = 120）。
#[must_use]
pub fn leaf_containers_one_root_node_holds() -> (capacity: usize)
    ensures
        capacity == 135,
{
    index_node_entry_capacity(
        INODE_KEY_WIDTH_IN_BYTES,
        usize::try_from(INODE_INTERNAL_ENTRY).expect("120"),
    )
}

/// 这一版的一片叶容器从哪来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InodeLeafContainerOrigin {
    /// 这次重写（内容、或分隔 key 变了，或是新分裂出来的）。
    RewrittenThisPublish,
    /// 上一版那一片一个字节不动，带的是它在上一版里的叶序（左边有片身份消失时比这一版的大）。
    CarriedFrom(InodeLeafContainerIndexInTree),
}

/// 这次身份消失的一片叶容器（删空摘掉，或被左邻吸收）：它在上一版里的叶序与它的身份。发布路径给它的身份写一条退役记录、
/// 按上一版的叶序换下它那一份（D8（核心索引结构） 已定项 6「死亡分三层」第 ② 层）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetiredInodeLeafContainer {
    pub previous_index: InodeLeafContainerIndexInTree,
    pub identity: PackedIdentity,
}

/// 这次发布之后 inode 树的叶容器，以及这次要重写哪几片、每片从哪来、哪几片身份消失。
///
/// 没被点到的那几片一个字节都不变（分裂时的左半就是这样：它的记录与身份都没动 ⇒ 不进这次的重写清单，
/// 连 COW 都不做），调用方按 `origins` 里上一版的叶序照抄上一版的落点与指针。
#[derive(Clone, Debug, PartialEq, Eq)]
#[verifier::allow(autoderive_clone_without_spec)]
pub struct InodeLeafContainersAfterThisPublish {
    /// 左起按 key 序的全部叶容器。
    pub containers: Vec<InodeLeafContainer>,
    /// 这次要重写的那几片，按叶序升序（就是 `origins` 里 `RewrittenThisPublish` 的那几个叶序）。
    pub rewritten: Vec<InodeLeafContainerIndexInTree>,
    /// 每片从哪来，与 `containers` 一一对应。
    pub origins: Vec<InodeLeafContainerOrigin>,
    /// 这次身份消失的那几片，按它们在上一版里的叶序升序（只有 [`remove_record_from_leaf_containers`] 会有）：
    /// 删空的是最右那一片时重写清单是空的，根照样要重写（少了一条条目）。
    pub retired: Vec<RetiredInodeLeafContainer>,
}

impl InodeLeafContainersAfterThisPublish {
    /// 这次要不要重写 inode 树根：有叶容器重写（COW 叶 + 全部祖先，D23（journal 的角色与格式） 已定项 1 甲），或有一片身份消失、不再落盘。
    #[must_use]
    pub fn is_the_root_rewritten(&self) -> (is_the_root_rewritten: bool)
        ensures
            is_the_root_rewritten == (self.rewritten.len() > 0 || self.retired.len() > 0),
    {
        !self.rewritten.is_empty() || !self.retired.is_empty()
    }

    /// 这次身份消失的那几片的身份，按上一版的叶序：这次发布的退役集合（上一版的叶身份集 − 这一版的叶身份集）。
    #[must_use]
    pub fn retired_identities(&self) -> (identities: Vec<PackedIdentity>)
        ensures
            identities.len() == self.retired.len(),
            forall|position: int|
                0 <= position < identities.len() ==> #[trigger] identities@[position] == self.retired@[position].identity,
    {
        let mut identities: Vec<PackedIdentity> = Vec::new();
        let mut position: usize = 0;
        while position < self.retired.len()
            invariant
                position <= self.retired.len(),
                identities.len() == position,
                forall|done: int| 0 <= done < position ==> #[trigger] identities@[done] == self.retired@[done].identity,
            decreases self.retired.len() - position,
        {
            identities.push(self.retired[position].identity);
            position += 1;
        }
        identities
    }
}

/// 左从右借的下限：一片叶删完一条之后记录数少于它、这次又没并过、它有右邻，就从右邻借（E176 M3 那一臂的主取样点
/// f = ⌈233 ÷ 2⌉ = 117，`scripts/experiment-apparatus/e176_inode_leaf_merge_capacity_and_rewrites.rs` 的 `BORROW_FLOOR_MAIN`）。
/// 向上取整写成 (233 + 1) ÷ 2（与 `u64::div_ceil` 同值）。
#[must_use]
#[allow(clippy::manual_div_ceil, reason = "vstd 没有 u64::div_ceil 的规格，写成 (233 + 1) ÷ 2")]
pub fn records_below_which_a_leaf_borrows_from_its_right_neighbour() -> (floor: usize)
    ensures
        floor == 117,
{
    usize::try_from((INODE_LEAF_RECORDS + 1) / 2).expect("117")
}

/// 往 inode 树里写记录时走得到、而条款没写的那几格。三样都在**任何落盘动作之前**交回，盘上逐字节不变。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InodeTreeWriteRefusal {
    /// 要写的 inode 号既不在树里、又不大于树里每一个 key：这条记录要插进中间某片叶。
    ///
    /// D8（核心索引结构） 已定项 6 的分裂纪律整条建在「inode 号在每一条时间线上单调 ⇒ 插入永远落在最右的叶 ⇒
    /// 分裂只发生在最右叶，分裂点取末尾」上；中间插入要的是非末尾分裂（左右两半各装哪些记录、身份给谁、
    /// 分隔 key 取什么），仓里一个字都没写。
    InsertWouldNotLandInTheRightmostLeaf {
        inode: InodeNumber,
        largest_inode_number_in_the_tree: InodeNumber,
    },
    /// 这次写之后的叶容器数超过一个码 2 根装得下的条目数：inode 树要在根之下再长一层内部节点。
    ///
    /// D8（核心索引结构） 已定项 6 只写了「内部节点同样在右端溢出、末尾分裂，右半是码 2、没有容器身份」，
    /// 没写树长高那一步——新的根从哪来、它的 key 区间与出生序号怎么取、两层内部节点的条目怎么指
    /// （类型段 0 那一支今天盘上一条都没有）。里程碑「覆盖写、释放、回退与复用」并行线三把
    /// 「内部节点条目 120、扇出 135 ⇒ 十万个文件约 430 片叶、树高 2」整句标成**预想**。
    MoreLeafContainersThanOneRootNodeHolds { containers: usize, capacity: usize },
    /// unlink 最后一批要删的那条记录不在树里（[`remove_record_from_leaf_containers`]）：调用方按现行那一版的队列取号，
    /// 号在队列里而记录不在树里说明那一版的 `deleted_inodes` 与 inode 树对不上。
    RecordToRemoveIsNotInTheTree { inode: InodeNumber },
    /// 同一次写里的记录没有按 inode 号严格升序排。
    ///
    /// 升序是调用方的义务，不是这里替它排：一次请求内按 key 升序取号是 D16（发布语义） 已定项 5 末段的纪律
    /// （「同一次请求切出的若干事务按其单元的 key 升序取号」），这里替它排就把那条纪律的违反盖掉了。
    RecordsNotInStrictlyAscendingInodeOrder {
        earlier: InodeNumber,
        later: InodeNumber,
    },
}

// ===== 叶容器的规格 =====

/// 叶容器的记录按叶序依次连起来（「key 到条目」在规格里的样子）。
pub open spec fn leaf_records(containers: Seq<InodeLeafContainer>) -> Seq<InodeRecord>
    decreases containers.len(),
{
    if containers.len() == 0 {
        Seq::empty()
    } else {
        leaf_records(containers.drop_last()) + containers.last().records@
    }
}

/// 记录按 inode 号严格升序。
pub open spec fn inode_records_strictly_ascend(records: Seq<InodeRecord>) -> bool {
    forall|earlier: int, later: int|
        0 <= earlier < later < records.len() ==> (#[trigger] records[earlier]).inode < (#[trigger] records[later]).inode
}

/// 这几条记录里有 inode 号是 `inode` 的那一条。
pub open spec fn records_hold_inode(records: Seq<InodeRecord>, inode: u64) -> bool {
    exists|position: int| 0 <= position < records.len() && (#[trigger] records[position]).inode == inode
}

/// 叶容器良好：记录依次连起来按 inode 号严格升序，每片 1 到 233 条。
pub open spec fn leaf_containers_are_well_formed(containers: Seq<InodeLeafContainer>) -> bool {
    &&& inode_records_strictly_ascend(leaf_records(containers))
    &&& forall|position: int|
        0 <= position < containers.len() ==> 1 <= (#[trigger] containers[position]).records@.len() <= 233
}

/// 从第 `from` 条起第一条 inode 号是 `inode` 的记录的位置；没有交回条数。
pub open spec fn first_position_of_inode(records: Seq<InodeRecord>, inode: u64, from: int) -> int
    decreases records.len() - from,
{
    if from >= records.len() {
        records.len() as int
    } else if records[from].inode == inode {
        from
    } else {
        first_position_of_inode(records, inode, from + 1)
    }
}

/// 把 `written` 逐条写进这组记录：号已在里面的原地换掉那一条，不在的接在末尾（[`write_records_into_leaf_containers`] 交回 `Ok`
/// 时，不在的那几条的号都大于里面的每一个）。
pub open spec fn records_after_writing(records: Seq<InodeRecord>, written: Seq<InodeRecord>) -> Seq<InodeRecord>
    decreases written.len(),
{
    if written.len() == 0 {
        records
    } else {
        let earlier = records_after_writing(records, written.drop_last());
        let record = written.last();
        let position = first_position_of_inode(earlier, record.inode, 0);
        if position < earlier.len() {
            earlier.update(position, record)
        } else {
            earlier.push(record)
        }
    }
}

/// 第 `position` 片之前一共几条记录（它的记录在连起来的那一串里从这里起）。
pub open spec fn records_before_container(containers: Seq<InodeLeafContainer>, position: int) -> int {
    leaf_records(containers.take(position)).len() as int
}

// ===== 叶容器的引理 =====

/// 两组叶容器接起来，记录也接起来。
proof fn lemma_leaf_records_of_a_concatenation(earlier: Seq<InodeLeafContainer>, later: Seq<InodeLeafContainer>)
    ensures
        leaf_records(earlier + later) == leaf_records(earlier) + leaf_records(later),
    decreases later.len(),
{
    if later.len() == 0 {
        assert(earlier + later =~= earlier);
        assert(leaf_records(earlier) + Seq::<InodeRecord>::empty() =~= leaf_records(earlier));
    } else {
        assert((earlier + later).drop_last() =~= earlier + later.drop_last());
        assert((earlier + later).last() == later.last());
        lemma_leaf_records_of_a_concatenation(earlier, later.drop_last());
        assert(leaf_records(earlier) + leaf_records(later.drop_last()) + later.last().records@ =~= leaf_records(earlier)
            + (leaf_records(later.drop_last()) + later.last().records@));
    }
}

/// 一片叶容器的记录就是它自己的记录。
proof fn lemma_leaf_records_of_one_container(container: InodeLeafContainer)
    ensures
        leaf_records(seq![container]) == container.records@,
{
    reveal_with_fuel(leaf_records, 2);
    assert(seq![container].drop_last() =~= Seq::<InodeLeafContainer>::empty());
    assert(Seq::<InodeRecord>::empty() + container.records@ =~= container.records@);
}

/// 第 `position` 片的记录在连起来的那一串里占着从它之前的条数起的那一段；把这一片换成另一片，那一段换成另一片的记录。
proof fn lemma_leaf_records_around_one_container(
    containers: Seq<InodeLeafContainer>,
    position: int,
    replacement: InodeLeafContainer,
)
    requires
        0 <= position < containers.len(),
    ensures
        leaf_records(containers) == leaf_records(containers.take(position)) + containers[position].records@ + leaf_records(
            containers.subrange(position + 1, containers.len() as int),
        ),
        leaf_records(containers.update(position, replacement)) == leaf_records(containers.take(position))
            + replacement.records@ + leaf_records(containers.subrange(position + 1, containers.len() as int)),
{
    let before = containers.take(position);
    let after = containers.subrange(position + 1, containers.len() as int);
    assert(containers =~= before + seq![containers[position]] + after);
    assert(containers.update(position, replacement) =~= before + seq![replacement] + after);
    lemma_leaf_records_of_a_concatenation(before + seq![containers[position]], after);
    lemma_leaf_records_of_a_concatenation(before, seq![containers[position]]);
    lemma_leaf_records_of_a_concatenation(before + seq![replacement], after);
    lemma_leaf_records_of_a_concatenation(before, seq![replacement]);
    lemma_leaf_records_of_one_container(containers[position]);
    lemma_leaf_records_of_one_container(replacement);
}

/// 第 `position` 片的第 `record_position` 条就是连起来那一串里它之前的条数加 `record_position` 那一条。
proof fn lemma_a_record_inside_the_leaf_records(containers: Seq<InodeLeafContainer>, position: int, record_position: int)
    requires
        0 <= position < containers.len(),
        0 <= record_position < containers[position].records@.len(),
    ensures
        records_before_container(containers, position) + record_position < leaf_records(containers).len(),
        leaf_records(containers)[records_before_container(containers, position) + record_position]
            == containers[position].records@[record_position],
{
    lemma_leaf_records_around_one_container(containers, position, containers[position]);
}

/// 严格升序的一串里，号相同的只有一条：从头找到的就是它。
proof fn lemma_first_position_of_an_inode_in_ascending_records(records: Seq<InodeRecord>, inode: u64, position: int, from: int)
    requires
        inode_records_strictly_ascend(records),
        0 <= from <= position < records.len(),
        records[position].inode == inode,
    ensures
        first_position_of_inode(records, inode, from) == position,
    decreases position - from,
{
    if from < position {
        assert(records[from].inode < records[position].inode);
        lemma_first_position_of_an_inode_in_ascending_records(records, inode, position, from + 1);
    }
}

/// 一条都没有这个号时，从头找交回条数。
proof fn lemma_first_position_of_an_absent_inode(records: Seq<InodeRecord>, inode: u64, from: int)
    requires
        0 <= from <= records.len(),
        !records_hold_inode(records, inode),
    ensures
        first_position_of_inode(records, inode, from) == records.len(),
    decreases records.len() - from,
{
    if from < records.len() {
        assert(records[from].inode != inode);
        lemma_first_position_of_an_absent_inode(records, inode, from + 1);
    }
}

/// 严格升序的一串，截出一段照样严格升序。
proof fn lemma_a_piece_of_ascending_records_ascends(records: Seq<InodeRecord>, start: int, end: int)
    requires
        inode_records_strictly_ascend(records),
        0 <= start <= end <= records.len(),
    ensures
        inode_records_strictly_ascend(records.subrange(start, end)),
{
    let piece = records.subrange(start, end);
    assert forall|earlier: int, later: int| 0 <= earlier < later < piece.len() implies (#[trigger] piece[earlier]).inode
        < (#[trigger] piece[later]).inode by {
        assert(piece[earlier] == records[start + earlier]);
        assert(piece[later] == records[start + later]);
    }
}

/// 叶容器良好时，每一片的记录自己也严格升序，且每片都在连起来那一串里。
proof fn lemma_each_container_of_well_formed_leaves_ascends(containers: Seq<InodeLeafContainer>, position: int)
    requires
        inode_records_strictly_ascend(leaf_records(containers)),
        0 <= position < containers.len(),
    ensures
        inode_records_strictly_ascend(containers[position].records@),
{
    lemma_leaf_records_around_one_container(containers, position, containers[position]);
    let start = records_before_container(containers, position);
    let end = start + containers[position].records@.len();
    assert(leaf_records(containers).subrange(start, end) =~= containers[position].records@);
    lemma_a_piece_of_ascending_records_ascends(leaf_records(containers), start, end);
}

/// 连起来那一串里有这个号，就有一片装着它。
proof fn lemma_an_inode_in_the_leaf_records_is_in_one_container(containers: Seq<InodeLeafContainer>, inode: u64)
    requires
        records_hold_inode(leaf_records(containers), inode),
    ensures
        exists|position: int| 0 <= position < containers.len() && records_hold_inode(#[trigger] containers[position].records@, inode),
    decreases containers.len(),
{
    let shorter = containers.drop_last();
    let records = leaf_records(containers);
    let found = choose|found: int| 0 <= found < records.len() && (#[trigger] records[found]).inode == inode;
    if found < leaf_records(shorter).len() {
        assert(leaf_records(shorter)[found] == records[found]);
        lemma_an_inode_in_the_leaf_records_is_in_one_container(shorter, inode);
        let position = choose|position: int| 0 <= position < shorter.len() && records_hold_inode(#[trigger] shorter[position].records@, inode);
        assert(containers[position] == shorter[position]);
    } else {
        let last = containers.len() - 1;
        assert(containers.last().records@[found - leaf_records(shorter).len()] == records[found]);
        assert(records_hold_inode(containers[last].records@, inode));
    }
}

/// 最后一片的最后一条就是连起来那一串的最后一条。
proof fn lemma_the_last_leaf_record(containers: Seq<InodeLeafContainer>)
    requires
        containers.len() >= 1,
        containers.last().records@.len() >= 1,
    ensures
        leaf_records(containers).len() >= 1,
        leaf_records(containers).last() == containers.last().records@.last(),
{
}

/// 严格升序的一串末尾接一条比末尾那条大的，照样严格升序。
proof fn lemma_pushing_a_larger_record_keeps_the_records_ascending(records: Seq<InodeRecord>, record: InodeRecord)
    requires
        inode_records_strictly_ascend(records),
        records.len() >= 1 ==> records.last().inode < record.inode,
    ensures
        inode_records_strictly_ascend(records.push(record)),
{
    let longer = records.push(record);
    assert forall|earlier: int, later: int| 0 <= earlier < later < longer.len() implies (#[trigger] longer[earlier]).inode
        < (#[trigger] longer[later]).inode by {
        if later == records.len() {
            if earlier < records.len() - 1 {
                assert(records[earlier].inode < records[records.len() - 1].inode);
            }
        } else {
            assert(longer[earlier] == records[earlier]);
            assert(longer[later] == records[later]);
        }
    }
}

/// 严格升序的一串换掉一条为同号的，照样严格升序。
proof fn lemma_updating_a_record_with_the_same_inode_keeps_the_records_ascending(
    records: Seq<InodeRecord>,
    position: int,
    record: InodeRecord,
)
    requires
        inode_records_strictly_ascend(records),
        0 <= position < records.len(),
        records[position].inode == record.inode,
    ensures
        inode_records_strictly_ascend(records.update(position, record)),
{
    let updated = records.update(position, record);
    assert forall|earlier: int, later: int| 0 <= earlier < later < updated.len() implies (#[trigger] updated[earlier]).inode
        < (#[trigger] updated[later]).inode by {
        assert(records[earlier].inode < records[later].inode);
    }
}

/// 严格升序的一串去掉一条，照样严格升序。
proof fn lemma_removing_a_record_keeps_the_records_ascending(records: Seq<InodeRecord>, position: int)
    requires
        inode_records_strictly_ascend(records),
        0 <= position < records.len(),
    ensures
        inode_records_strictly_ascend(records.remove(position)),
{
    let shorter = records.remove(position);
    assert forall|earlier: int, later: int| 0 <= earlier < later < shorter.len() implies (#[trigger] shorter[earlier]).inode
        < (#[trigger] shorter[later]).inode by {
        let earlier_in_records = if earlier < position { earlier } else { earlier + 1 };
        let later_in_records = if later < position { later } else { later + 1 };
        assert(shorter[earlier] == records[earlier_in_records]);
        assert(shorter[later] == records[later_in_records]);
    }
}


/// 两组叶容器逐片相同（身份、分隔 key、记录都相同），记录连起来也相同。
proof fn lemma_the_same_leaf_containers_have_the_same_leaf_records(earlier: Seq<InodeLeafContainer>, later: Seq<InodeLeafContainer>)
    requires
        earlier.len() == later.len(),
        forall|position: int| 0 <= position < earlier.len() ==> #[trigger] leaf_containers_are_the_same(earlier[position], later[position]),
    ensures
        leaf_records(earlier) == leaf_records(later),
    decreases earlier.len(),
{
    if earlier.len() > 0 {
        assert forall|position: int| 0 <= position < earlier.drop_last().len() implies #[trigger] leaf_containers_are_the_same(
            earlier.drop_last()[position],
            later.drop_last()[position],
        ) by {
            assert(leaf_containers_are_the_same(earlier[position], later[position]));
        }
        lemma_the_same_leaf_containers_have_the_same_leaf_records(earlier.drop_last(), later.drop_last());
        assert(leaf_containers_are_the_same(earlier[earlier.len() - 1], later[later.len() - 1]));
    }
}

/// 三段叶容器接起来，记录也三段接起来。
proof fn lemma_leaf_records_of_three_parts(
    earlier: Seq<InodeLeafContainer>,
    middle: Seq<InodeLeafContainer>,
    later: Seq<InodeLeafContainer>,
)
    ensures
        leaf_records(earlier + middle + later) == leaf_records(earlier) + leaf_records(middle) + leaf_records(later),
{
    lemma_leaf_records_of_a_concatenation(earlier + middle, later);
    lemma_leaf_records_of_a_concatenation(earlier, middle);
}

/// 两片叶容器的记录就是两片的记录接起来。
proof fn lemma_leaf_records_of_two_containers(left: InodeLeafContainer, right: InodeLeafContainer)
    ensures
        leaf_records(seq![left, right]) == left.records@ + right.records@,
{
    assert(seq![left, right] =~= seq![left] + seq![right]);
    lemma_leaf_records_of_a_concatenation(seq![left], seq![right]);
    lemma_leaf_records_of_one_container(left);
    lemma_leaf_records_of_one_container(right);
}
// ===== 查找 =====

/// 重写清单（叶序）严格升序。
pub open spec fn indices_strictly_ascend(indices: Seq<InodeLeafContainerIndexInTree>) -> bool {
    forall|earlier: int, later: int|
        0 <= earlier < later < indices.len() ==> (#[trigger] indices[earlier]).0 < (#[trigger] indices[later]).0
}

/// 重写清单里有叶序 `position`。
#[verifier::opaque]
pub open spec fn indices_hold(indices: Seq<InodeLeafContainerIndexInTree>, position: int) -> bool {
    exists|listed: int| 0 <= listed < indices.len() && (#[trigger] indices[listed]).0 == position
}

/// 两片叶容器逐字节相同：身份、分隔 key、记录都相同。
pub open spec fn leaf_containers_are_the_same(earlier: InodeLeafContainer, later: InodeLeafContainer) -> bool {
    &&& earlier.identity == later.identity
    &&& earlier.separator_key_in_the_parent_entry == later.separator_key_in_the_parent_entry
    &&& earlier.records@ == later.records@
}

/// 在严格升序的记录里按 inode 号找：找到交回它的位置，找不到交回插入点（严格升序时两样都唯一，与 `binary_search_by_key` 交回的相同）。
fn search_records_by_inode(records: &[InodeRecord], inode: u64) -> (found: Result<usize, usize>)
    requires
        inode_records_strictly_ascend(records@),
    ensures
        match found {
            Ok(position) => position < records.len() && records@[position as int].inode == inode,
            Err(position) => position <= records.len() && !records_hold_inode(records@, inode),
        },
{
    let mut low: usize = 0;
    let mut high: usize = records.len();
    while low < high
        invariant
            low <= high <= records.len(),
            inode_records_strictly_ascend(records@),
            forall|earlier: int| 0 <= earlier < low ==> (#[trigger] records@[earlier]).inode < inode,
            forall|later: int| high <= later < records.len() ==> (#[trigger] records@[later]).inode > inode,
        decreases high - low,
    {
        let middle = low + (high - low) / 2;
        let middle_inode = records[middle].inode;
        if middle_inode == inode {
            return Ok(middle);
        } else if middle_inode < inode {
            proof {
                assert forall|earlier: int| 0 <= earlier < middle + 1 implies (#[trigger] records@[earlier]).inode < inode by {
                    if earlier < middle {
                        assert(records@[earlier].inode < records@[middle as int].inode);
                    }
                }
            }
            low = middle + 1;
        } else {
            proof {
                assert forall|later: int| middle <= later < records.len() implies (#[trigger] records@[later]).inode > inode by {
                    if later > middle {
                        assert(records@[middle as int].inode < records@[later].inode);
                    }
                }
            }
            high = middle;
        }
    }
    proof {
        assert forall|position: int| 0 <= position < records.len() implies (#[trigger] records@[position]).inode != inode by {
            if position < low {
                assert(records@[position].inode < inode);
            } else {
                assert(records@[position].inode > inode);
            }
        }
    }
    Err(low)
}

/// 重写清单里有没有这一片。
fn index_is_noted(noted_so_far: &[InodeLeafContainerIndexInTree], index: InodeLeafContainerIndexInTree) -> (is_noted: bool)
    ensures
        is_noted == indices_hold(noted_so_far@, index.0 as int),
{
    proof {
        reveal(indices_hold);
    }
    let mut listed: usize = 0;
    while listed < noted_so_far.len()
        invariant
            listed <= noted_so_far.len(),
            forall|checked: int| 0 <= checked < listed ==> (#[trigger] noted_so_far@[checked]).0 != index.0,
        decreases noted_so_far.len() - listed,
    {
        if noted_so_far[listed].0 == index.0 {
            proof {
                reveal(indices_hold);
                assert(noted_so_far@[listed as int].0 == index.0);
            }
            return true;
        }
        listed += 1;
    }
    proof {
        reveal(indices_hold);
    }
    false
}

/// 把这一片记进重写清单：清单按叶序升序、同一片只进一次（同一次发布里改同一片叶两次只重写一遍）。插在第一个不小于它的位置，已有就不动。
fn note_rewritten(index: InodeLeafContainerIndexInTree, noted_so_far: &mut Vec<InodeLeafContainerIndexInTree>)
    requires
        indices_strictly_ascend(old(noted_so_far)@),
    ensures
        indices_strictly_ascend(final(noted_so_far)@),
        forall|position: int| indices_hold(final(noted_so_far)@, position) <==> (indices_hold(old(noted_so_far)@, position) || position == index.0),
{
    proof {
        reveal(indices_hold);
    }
    let ghost noted_before = noted_so_far@;
    let mut insertion_point: usize = 0;
    while insertion_point < noted_so_far.len() && noted_so_far[insertion_point].0 < index.0
        invariant
            noted_so_far@ == noted_before,
            insertion_point <= noted_so_far.len(),
            forall|earlier: int| 0 <= earlier < insertion_point ==> (#[trigger] noted_so_far@[earlier]).0 < index.0,
        decreases noted_so_far.len() - insertion_point,
    {
        insertion_point += 1;
    }
    if insertion_point < noted_so_far.len() && noted_so_far[insertion_point].0 == index.0 {
        proof {
            assert forall|position: int| indices_hold(noted_so_far@, position) <==> (indices_hold(noted_before, position) || position
                == index.0) by {
                if position == index.0 {
                    assert(noted_so_far@[insertion_point as int].0 == position);
                }
            }
        }
        return;
    }
    noted_so_far.insert(insertion_point, index);
    proof {
        let noted_after = noted_so_far@;
        assert forall|later: int| insertion_point < later < noted_after.len() implies (#[trigger] noted_after[later]).0 > index.0 by {
            assert(noted_after[later] == noted_before[later - 1]);
            if insertion_point < noted_before.len() {
                assert(noted_before[insertion_point as int].0 >= index.0);
                if later - 1 > insertion_point {
                    assert(noted_before[insertion_point as int].0 < noted_before[later - 1].0);
                }
            }
        }
        assert forall|earlier: int, later: int| 0 <= earlier < later < noted_after.len() implies (#[trigger] noted_after[earlier]).0
            < (#[trigger] noted_after[later]).0 by {
            if later < insertion_point {
                assert(noted_after[earlier] == noted_before[earlier]);
                assert(noted_after[later] == noted_before[later]);
            } else if later == insertion_point {
                assert(noted_after[earlier] == noted_before[earlier]);
            } else if earlier < insertion_point {
                assert(noted_after[earlier] == noted_before[earlier]);
            } else if earlier > insertion_point {
                assert(noted_after[earlier] == noted_before[earlier - 1]);
                assert(noted_after[later] == noted_before[later - 1]);
            }
        }
        assert forall|position: int| indices_hold(noted_after, position) <==> (indices_hold(noted_before, position) || position
            == index.0) by {
            if indices_hold(noted_after, position) {
                let listed = choose|listed: int| 0 <= listed < noted_after.len() && (#[trigger] noted_after[listed]).0 == position;
                if listed < insertion_point {
                    assert(noted_before[listed] == noted_after[listed]);
                } else if listed > insertion_point {
                    assert(noted_before[listed - 1] == noted_after[listed]);
                }
            }
            if indices_hold(noted_before, position) {
                let listed = choose|listed: int| 0 <= listed < noted_before.len() && (#[trigger] noted_before[listed]).0 == position;
                if listed < insertion_point {
                    assert(noted_after[listed] == noted_before[listed]);
                } else {
                    assert(noted_after[listed + 1] == noted_before[listed]);
                }
            }
            if position == index.0 {
                assert(noted_after[insertion_point as int] == index);
            }
        }
    }
}

/// 哪一片容器装着这个 inode 号；一条都没装着是 `None`。
fn position_of_the_container_holding(
    containers: &[InodeLeafContainer],
    inode: InodeNumber,
) -> (holding: Option<InodeLeafContainerIndexInTree>)
    requires
        forall|position: int|
            0 <= position < containers.len() ==> inode_records_strictly_ascend(#[trigger] containers[position].records@),
        containers.len() <= u32::MAX,
    ensures
        match holding {
            Some(index) => index.0 < containers.len() && records_hold_inode(containers[index.0 as int].records@, inode.0),
            None => forall|position: int|
                0 <= position < containers.len() ==> !records_hold_inode(#[trigger] containers[position].records@, inode.0),
        },
{
    let mut position: usize = 0;
    while position < containers.len()
        invariant
            position <= containers.len(),
            containers.len() <= u32::MAX,
            forall|any_position: int|
                0 <= any_position < containers.len() ==> inode_records_strictly_ascend(#[trigger] containers[any_position].records@),
            forall|checked: int| 0 <= checked < position ==> !records_hold_inode(#[trigger] containers[checked].records@, inode.0),
        decreases containers.len() - position,
    {
        match search_records_by_inode(&containers[position].records, inode.0) {
            Ok(_found) => {
                return Some(InodeLeafContainerIndexInTree::of_position(position));
            },
            Err(_insertion_point) => {},
        }
        position += 1;
    }
    None
}

/// 树里最大的 inode 号；树是空的是 `None`。容器按 key 序、容器内记录按 inode 号升序 ⇒ 最后一片的最后一条。
fn largest_inode_number_in_the_tree(containers: &[InodeLeafContainer]) -> (largest: Option<InodeNumber>)
    ensures
        containers.len() == 0 ==> largest is None,
        containers.len() > 0 ==> largest == containers@.last().largest_inode_number_in_the_specification(),
{
    match containers.last() {
        Some(last) => last.largest_inode_number(),
        None => None,
    }
}

impl InodeLeafContainer {
    /// 它装的最大 inode 号（规格里用）。
    pub open spec fn largest_inode_number_in_the_specification(self) -> Option<InodeNumber> {
        if self.records@.len() == 0 {
            None
        } else {
            Some(InodeNumber(self.records@.last().inode))
        }
    }
}

/// 最右那片叶还装得下就给它的序号；树是空的、或最右那片已经满 233 条，都是 `None`（两种情况都要新建一片容器）。
fn rightmost_container_with_room(containers: &[InodeLeafContainer]) -> (rightmost: Option<InodeLeafContainerIndexInTree>)
    requires
        containers.len() <= u32::MAX + 1,
        containers.len() > 0 ==> containers@.last().records@.len() <= 233,
    ensures
        rightmost is Some <==> (containers.len() > 0 && containers@.last().records@.len() < 233),
        rightmost matches Some(index) ==> index.0 == containers.len() - 1,
{
    let last = containers.last()?;
    if last.free_record_slots() == 0 {
        return None;
    }
    Some(InodeLeafContainerIndexInTree::of_position(containers.len() - 1))
}

/// 一组记录相邻两条逐对严格升序，整组就严格升序。
proof fn lemma_adjacent_ascending_records_ascend(records: Seq<InodeRecord>)
    requires
        forall|later: int| 1 <= later < records.len() ==> (#[trigger] records[later - 1]).inode < records[later].inode,
    ensures
        inode_records_strictly_ascend(records),
{
    assert forall|earlier: int, later: int| 0 <= earlier < later < records.len() implies (#[trigger] records[earlier]).inode
        < (#[trigger] records[later]).inode by {
        lemma_adjacent_ascending_records_ascend_between(records, earlier, later);
    }
}

proof fn lemma_adjacent_ascending_records_ascend_between(records: Seq<InodeRecord>, earlier: int, later: int)
    requires
        forall|any_later: int| 1 <= any_later < records.len() ==> (#[trigger] records[any_later - 1]).inode < records[any_later].inode,
        0 <= earlier < later < records.len(),
    ensures
        records[earlier].inode < records[later].inode,
    decreases later - earlier,
{
    assert(records[later - 1].inode < records[later].inode);
    if earlier < later - 1 {
        lemma_adjacent_ascending_records_ascend_between(records, earlier, later - 1);
    }
}

/// 逐片照抄叶容器（记录逐条照抄）。
fn copy_of_the_leaf_containers(containers: &[InodeLeafContainer]) -> (copy: Vec<InodeLeafContainer>)
    ensures
        copy.len() == containers.len(),
        forall|position: int| 0 <= position < copy.len() ==> #[trigger] leaf_containers_are_the_same(copy@[position], containers@[position]),
        leaf_records(copy@) == leaf_records(containers@),
{
    let mut copy: Vec<InodeLeafContainer> = Vec::new();
    let mut position: usize = 0;
    while position < containers.len()
        invariant
            position <= containers.len(),
            copy.len() == position,
            forall|done: int| 0 <= done < position ==> #[trigger] leaf_containers_are_the_same(copy@[done], containers@[done]),
        decreases containers.len() - position,
    {
        copy.push(copy_of_one_leaf_container(&containers[position]));
        position += 1;
    }
    proof {
        lemma_the_same_leaf_containers_have_the_same_leaf_records(copy@, containers@);
    }
    copy
}

/// 照抄一片叶容器。
fn copy_of_one_leaf_container(container: &InodeLeafContainer) -> (copy: InodeLeafContainer)
    ensures
        leaf_containers_are_the_same(copy, *container),
{
    let mut records: Vec<InodeRecord> = Vec::new();
    let mut position: usize = 0;
    while position < container.records.len()
        invariant
            position <= container.records.len(),
            records@ == container.records@.take(position as int),
        decreases container.records.len() - position,
    {
        records.push(container.records[position]);
        position += 1;
        proof {
            assert(records@ =~= container.records@.take(position as int));
        }
    }
    proof {
        assert(records@ =~= container.records@);
    }
    InodeLeafContainer {
        identity: container.identity,
        separator_key_in_the_parent_entry: container.separator_key_in_the_parent_entry,
        records,
    }
}

/// 哪一片都不装着这个号，连起来那一串里也没有它。
proof fn lemma_an_inode_absent_from_every_container_is_absent_from_the_leaf_records(
    containers: Seq<InodeLeafContainer>,
    inode: u64,
)
    requires
        forall|position: int| 0 <= position < containers.len() ==> !records_hold_inode(#[trigger] containers[position].records@, inode),
    ensures
        !records_hold_inode(leaf_records(containers), inode),
{
    if records_hold_inode(leaf_records(containers), inode) {
        lemma_an_inode_in_the_leaf_records_is_in_one_container(containers, inode);
    }
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;
    use singlefs_format::TREE_IDENTIFIER_INODE;

    pub(super) const INODE_TREE: TreeIdentifier = TreeIdentifier(TREE_IDENTIFIER_INODE);
    /// 新池新建文件把树建起来那次发布的 checkpoint_txg（字节表四：容器 1、出生代 3）。
    pub(super) const TREE_BIRTH_TXG: CheckpointTxg = CheckpointTxg(3);

    pub(super) fn live_inode_record(inode: u64, size: u64, change_count: u64) -> InodeRecord {
        InodeRecord {
            inode,
            object_birth: TREE_BIRTH_TXG,
            nlink: crate::records::NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
            size,
            change_count,
            write_time_seconds: 1_788_000_000,
            truncation_field: crate::records::InodeRecordTruncationField::NOT_WRITTEN,
        }
    }

    /// 一个码 3 单元按 inode 记录宽装得下的条数不多于一容器的上限：`free_record_slots` 那道断言对盘上读来的容器从盘上走不到，
    /// 靠的就是这条算术（读者判记录宽 = `INODE_RECORD_BYTES`、记录区不越过单元末尾）。改了记录宽、单元头宽或预留位而不重算
    /// `INODE_LEAF_RECORDS`，盘上一片就可能装得下第 234 条（C476 普查「缺口」表 `inode_tree.rs:88` 那一行）。
    #[test]
    fn packed_unit_holds_no_more_inode_records_than_the_leaf_record_limit() {
        use singlefs_format::{
            DATA_UNIT_BYTES, INODE_RECORD_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES,
            PACKED_UNIT_HEADER_BYTES,
        };
        let records_a_packed_unit_holds =
            (DATA_UNIT_BYTES - PACKED_UNIT_HEADER_BYTES - NONCE_MAC_ALGORITHM_RESERVED_BYTES)
                / INODE_RECORD_BYTES;
        assert!(
            records_a_packed_unit_holds <= INODE_LEAF_RECORDS,
            "一个码 3 单元装得下 {records_a_packed_unit_holds} 条 inode 记录，多于一容器的上限 {INODE_LEAF_RECORDS}"
        );
    }
}
