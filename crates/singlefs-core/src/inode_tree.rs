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
    /// `a_packed_unit_holds_no_more_inode_records_than_the_leaf_record_limit` 盯着）。装到了说明这两道之一被绕过了，
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

/// 上一版就有的那几片身份与分隔 key 都没变（新开的只在它们右边）。
#[verifier::opaque]
pub open spec fn previous_leaves_keep_their_identities(before: Seq<InodeLeafContainer>, containers: Seq<InodeLeafContainer>) -> bool {
    &&& containers.len() >= before.len()
    &&& forall|position: int|
        0 <= position < before.len() ==> (#[trigger] containers[position]).identity == before[position].identity
            && containers[position].separator_key_in_the_parent_entry == before[position].separator_key_in_the_parent_entry
}

/// 上一版之外的那几片都是这次分裂出来的（[`leaf_container_is_split_off_by_this_publish`]）。
#[verifier::opaque]
pub open spec fn leaves_split_off_by_this_publish_are_named_after_their_first_record(
    before: Seq<InodeLeafContainer>,
    containers: Seq<InodeLeafContainer>,
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
) -> bool {
    forall|position: int|
        before.len() <= position < containers.len() ==> leaf_container_is_split_off_by_this_publish(
            #[trigger] containers[position],
            checkpoint_txg,
            inode_tree,
        )
}

/// 重写清单按叶序严格升序、每一项都在片数以内。
#[verifier::opaque]
pub open spec fn rewritten_leaves_are_listed_in_order(rewritten: Seq<InodeLeafContainerIndexInTree>, container_count: int) -> bool {
    &&& indices_strictly_ascend(rewritten)
    &&& forall|listed: int| 0 <= listed < rewritten.len() ==> (#[trigger] rewritten[listed]).0 < container_count
}

/// 没进重写清单的那几片都是上一版那一片原样。
pub open spec fn leaves_not_rewritten_are_carried(
    before: Seq<InodeLeafContainer>,
    containers: Seq<InodeLeafContainer>,
    rewritten: Seq<InodeLeafContainerIndexInTree>,
) -> bool {
    forall|position: int|
        #![trigger containers[position]]
        #![trigger indices_hold(rewritten, position)]
        0 <= position < containers.len() && !indices_hold(rewritten, position) ==> position < before.len()
            && leaf_containers_are_the_same(containers[position], before[position])
}

/// 写到第几条为止，叶容器与重写清单对上一版、已写的记录逐条成立的那几样（[`write_records_into_leaf_containers`] 的循环不变量与 `ensures`）：
/// 叶容器良好；记录连起来就是上一版的照已写的改写；上一版那几片身份不变、新开的照分裂纪律命名；重写清单升序；没进清单的原样。
pub open spec fn writing_progress(
    containers_before_this_publish: Seq<InodeLeafContainer>,
    written_so_far: Seq<InodeRecord>,
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
    containers: Seq<InodeLeafContainer>,
    rewritten: Seq<InodeLeafContainerIndexInTree>,
) -> bool {
    let before = containers_before_this_publish;
    &&& leaf_containers_are_well_formed(containers)
    &&& leaf_records(containers) == records_after_writing(leaf_records(before), written_so_far)
    &&& containers.len() >= before.len()
    &&& previous_leaves_keep_their_identities(before, containers)
    &&& leaves_split_off_by_this_publish_are_named_after_their_first_record(before, containers, checkpoint_txg, inode_tree)
    &&& rewritten_leaves_are_listed_in_order(rewritten, containers.len() as int)
    &&& leaves_not_rewritten_are_carried(before, containers, rewritten)
}

/// 这一片是这次分裂（或树空时新建）出来的：身份照参数、容器号与分隔 key 都是它第一条记录的 inode 号。
pub open spec fn leaf_container_is_split_off_by_this_publish(
    container: InodeLeafContainer,
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
) -> bool {
    &&& container.records@.len() >= 1
    &&& container.identity == (PackedIdentity {
        birth_tree: inode_tree,
        record_type: PACKED_TYPE_INODE,
        container: container.records@[0].inode,
        container_birth: checkpoint_txg,
    })
    &&& container.separator_key_in_the_parent_entry == container.records@[0].inode
}

/// 把一组 inode 记录写进 inode 树的叶容器，返回这次发布之后的全部容器与这次要重写的那几片。
///
/// 一条记录的落法只有两种：
/// - 它的 inode 号已经在树里 ⇒ **原地换掉那一条**（覆盖写改 size / 改动计数走这一支），记录数不变、容器身份不变，
///   只有那一片进重写清单；
/// - 它的 inode 号大于树里每一个 key ⇒ 追加到最右那片叶；最右那片已经装满 233 条就**在末尾分裂**：
///   左半（原来那片）一条记录都不动、身份不动、**不进重写清单**，右半是一片新容器，容器号 = 这条新记录的 inode 号、
///   出生代 = 这次发布的 checkpoint_txg、出生树 = `inode_tree`。
///
/// 树是空的（新池新建文件那一档）时第一条记录建第一片容器，容器号 = 它自己的 inode 号。
///
/// # Errors
/// [`InodeTreeWriteRefusal`] 的三格：中间插入、容器数超过一个根装得下的、给的记录不按 inode 号严格升序。
pub fn write_records_into_leaf_containers(
    containers_before_this_publish: &[InodeLeafContainer],
    records_to_write: &[InodeRecord],
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
) -> (after: Result<InodeLeafContainersAfterThisPublish, InodeTreeWriteRefusal>)
    requires
        leaf_containers_are_well_formed(containers_before_this_publish@),
        containers_before_this_publish.len() + records_to_write.len() <= u32::MAX,
    ensures
        match after {
            Ok(after) => {
                &&& inode_records_strictly_ascend(records_to_write@)
                &&& after.containers.len() <= 135
                &&& writing_progress(
                    containers_before_this_publish@,
                    records_to_write@,
                    checkpoint_txg,
                    inode_tree,
                    after.containers@,
                    after.rewritten@,
                )
                &&& after.origins.len() == after.containers.len()
                &&& forall|position: int|
                    0 <= position < after.origins.len() ==> #[trigger] after.origins@[position] == if indices_hold(
                        after.rewritten@,
                        position,
                    ) {
                        InodeLeafContainerOrigin::RewrittenThisPublish
                    } else {
                        InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree(position as u32))
                    }
                &&& after.retired.len() == 0
            },
            Err(InodeTreeWriteRefusal::RecordsNotInStrictlyAscendingInodeOrder { earlier, later }) => !inode_records_strictly_ascend(
                records_to_write@,
            ) && earlier.0 >= later.0,
            Err(InodeTreeWriteRefusal::InsertWouldNotLandInTheRightmostLeaf { inode, largest_inode_number_in_the_tree }) => inode.0
                <= largest_inode_number_in_the_tree.0,
            Err(InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds { containers, capacity }) => capacity == 135
                && containers > capacity,
            Err(InodeTreeWriteRefusal::RecordToRemoveIsNotInTheTree { inode: _ }) => false,
        },
{
    let ghost before = containers_before_this_publish@;
    let mut later: usize = 1;
    while later < records_to_write.len()
        invariant
            1 <= later,
            forall|checked: int|
                1 <= checked < later && checked < records_to_write.len() ==> (#[trigger] records_to_write@[checked - 1]).inode
                    < records_to_write@[checked].inode,
        decreases records_to_write.len() - later,
    {
        if records_to_write[later - 1].inode >= records_to_write[later].inode {
            return Err(
                InodeTreeWriteRefusal::RecordsNotInStrictlyAscendingInodeOrder {
                    earlier: InodeNumber(records_to_write[later - 1].inode),
                    later: InodeNumber(records_to_write[later].inode),
                },
            );
        }
        later += 1;
    }
    proof {
        lemma_adjacent_ascending_records_ascend(records_to_write@);
    }
    let mut containers = copy_of_the_leaf_containers(containers_before_this_publish);
    // 重写清单按叶序升序、同一片只进一次：同一次发布里改同一片叶两次（改一条记录、再往它里面追加一条）只重写一遍。
    let mut rewritten: Vec<InodeLeafContainerIndexInTree> = Vec::new();
    proof {
        assert(records_to_write@.take(0) =~= Seq::<InodeRecord>::empty());
        reveal(indices_hold);
        reveal(previous_leaves_keep_their_identities);
        reveal(leaves_split_off_by_this_publish_are_named_after_their_first_record);
        reveal(rewritten_leaves_are_listed_in_order);
                assert forall|position: int| 0 <= position < containers.len() implies 1 <= (#[trigger] containers@[position]).records@.len()
            <= 233 by {
            assert(leaf_containers_are_the_same(containers@[position], before[position]));
        }
        assert forall|position: int| 0 <= position < before.len() implies (#[trigger] containers@[position]).identity
            == before[position].identity && containers@[position].separator_key_in_the_parent_entry
            == before[position].separator_key_in_the_parent_entry by {
            assert(leaf_containers_are_the_same(containers@[position], before[position]));
        }
        assert forall|position: int| 0 <= position < containers.len() && !indices_hold(rewritten@, position) implies position
            < before.len() && leaf_containers_are_the_same(#[trigger] containers@[position], before[position]) by {}
        assert(indices_strictly_ascend(rewritten@));
    }
    let mut record_position: usize = 0;
    while record_position < records_to_write.len()
        invariant
            before == containers_before_this_publish@,
            containers_before_this_publish.len() + records_to_write.len() <= u32::MAX,
            record_position <= records_to_write.len(),
            containers.len() <= before.len() + record_position,
            writing_progress(
                before,
                records_to_write@.take(record_position as int),
                checkpoint_txg,
                inode_tree,
                containers@,
                rewritten@,
            ),
        decreases records_to_write.len() - record_position,
    {
        let record = records_to_write[record_position];
        let inode = InodeNumber(record.inode);
        let ghost containers_at_the_start = containers@;
        let ghost written_so_far = records_to_write@.take(record_position as int);
        proof {
            reveal(rewritten_leaves_are_listed_in_order);
            assert forall|position: int| 0 <= position < containers.len() implies inode_records_strictly_ascend(
                #[trigger] containers@[position].records@,
            ) by {
                lemma_each_container_of_well_formed_leaves_ascends(containers@, position);
            }
            assert(records_to_write@.take(record_position as int + 1) =~= written_so_far.push(record));
        }
        let existing = position_of_the_container_holding(&containers, inode);
        match existing {
            Some(index) => {
                let holding_position = index.position();
                let position_in_container = search_records_by_inode(&containers[holding_position].records, inode.0)
                    .expect("position_of_the_container_holding 找到的就是这条记录所在的容器");
                let ghost rewritten_at_the_start = rewritten@;
                let container = &mut containers[holding_position];
                container.records[position_in_container] = record;
                note_rewritten(index, &mut rewritten);
                proof {
                    lemma_overwriting_a_record_keeps_the_writing_progress(
                        before,
                        written_so_far,
                        checkpoint_txg,
                        inode_tree,
                        containers_at_the_start,
                        rewritten_at_the_start,
                        holding_position as int,
                        position_in_container as int,
                        record,
                        containers@,
                        rewritten@,
                    );
                }
            },
            None => {
                proof {
                    lemma_an_inode_absent_from_every_container_is_absent_from_the_leaf_records(containers_at_the_start, inode.0);
                }
                let largest = largest_inode_number_in_the_tree(&containers);
                if let Some(largest_inode_number_in_the_tree) = largest {
                    if inode.0 <= largest_inode_number_in_the_tree.0 {
                        return Err(
                            InodeTreeWriteRefusal::InsertWouldNotLandInTheRightmostLeaf {
                                inode,
                                largest_inode_number_in_the_tree,
                            },
                        );
                    }
                }
                let rightmost = rightmost_container_with_room(&containers);
                match rightmost {
                    Some(index) => {
                        let ghost rewritten_at_the_start = rewritten@;
                        proof {
                            lemma_the_last_leaf_record(containers_at_the_start);
                        }
                        let container = &mut containers[index.position()];
                        container.records.push(record);
                        note_rewritten(index, &mut rewritten);
                        proof {
                            lemma_appending_to_the_rightmost_leaf_keeps_the_writing_progress(
                                before,
                                written_so_far,
                                checkpoint_txg,
                                inode_tree,
                                containers_at_the_start,
                                rewritten_at_the_start,
                                record,
                                containers@,
                                rewritten@,
                            );
                        }
                    },
                    None => {
                        // 末尾分裂：左半（原来那片满的叶）一条记录、一段身份都不动，右半从这条新记录起。
                        let records = vec![record];
                        let split_off = InodeLeafContainer {
                            identity: PackedIdentity {
                                birth_tree: inode_tree,
                                record_type: PACKED_TYPE_INODE,
                                container: inode.0,
                                container_birth: checkpoint_txg,
                            },
                            separator_key_in_the_parent_entry: inode.0,
                            records,
                        };
                        let ghost rewritten_at_the_start = rewritten@;
                        let ghost split_off_view = split_off;
                        proof {
                            assert(split_off.records@ =~= seq![record]);
                            if containers_at_the_start.len() >= 1 {
                                lemma_the_last_leaf_record(containers_at_the_start);
                            }
                        }
                        containers.push(split_off);
                        note_rewritten(InodeLeafContainerIndexInTree::of_position(containers.len() - 1), &mut rewritten);
                        proof {
                            lemma_splitting_off_a_leaf_keeps_the_writing_progress(
                                before,
                                written_so_far,
                                checkpoint_txg,
                                inode_tree,
                                containers_at_the_start,
                                rewritten_at_the_start,
                                record,
                                split_off_view,
                                rewritten@,
                            );
                        }
                    },
                }
            },
        }
        record_position += 1;
    }
    proof {
        assert(records_to_write@.take(records_to_write.len() as int) =~= records_to_write@);
    }
    let capacity = leaf_containers_one_root_node_holds();
    if containers.len() > capacity {
        return Err(
            InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds { containers: containers.len(), capacity },
        );
    }
    let mut origins: Vec<InodeLeafContainerOrigin> = Vec::new();
    let mut position: usize = 0;
    while position < containers.len()
        invariant
            position <= containers.len() <= 135,
            origins.len() == position,
            forall|done: int|
                0 <= done < position ==> #[trigger] origins@[done] == if indices_hold(rewritten@, done) {
                    InodeLeafContainerOrigin::RewrittenThisPublish
                } else {
                    InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree(done as u32))
                },
        decreases containers.len() - position,
    {
        let index = InodeLeafContainerIndexInTree::of_position(position);
        if index_is_noted(&rewritten, index) {
            origins.push(InodeLeafContainerOrigin::RewrittenThisPublish);
        } else {
            origins.push(InodeLeafContainerOrigin::CarriedFrom(index));
        }
        position += 1;
    }
    Ok(InodeLeafContainersAfterThisPublish { containers, rewritten, origins, retired: Vec::new() })
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

/// 原地改了第 `changed_position` 片（身份、分隔 key、第一条的号都不变）、把它记进重写清单之后，四样账目照样成立。
proof fn lemma_changing_one_leaf_in_place_keeps_the_bookkeeping(
    before: Seq<InodeLeafContainer>,
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
    containers: Seq<InodeLeafContainer>,
    rewritten: Seq<InodeLeafContainerIndexInTree>,
    changed_position: int,
    containers_after: Seq<InodeLeafContainer>,
    rewritten_after: Seq<InodeLeafContainerIndexInTree>,
)
    requires
        previous_leaves_keep_their_identities(before, containers),
        leaves_split_off_by_this_publish_are_named_after_their_first_record(before, containers, checkpoint_txg, inode_tree),
        rewritten_leaves_are_listed_in_order(rewritten, containers.len() as int),
        leaves_not_rewritten_are_carried(before, containers, rewritten),
        0 <= changed_position < containers.len(),
        containers_after.len() == containers.len(),
        forall|position: int|
            0 <= position < containers.len() && position != changed_position ==> #[trigger] containers_after[position]
                == containers[position],
        containers_after[changed_position].identity == containers[changed_position].identity,
        containers_after[changed_position].separator_key_in_the_parent_entry
            == containers[changed_position].separator_key_in_the_parent_entry,
        containers[changed_position].records@.len() >= 1,
        containers_after[changed_position].records@.len() >= 1,
        containers_after[changed_position].records@[0].inode == containers[changed_position].records@[0].inode,
        indices_strictly_ascend(rewritten_after),
        forall|position: int|
            #[trigger] indices_hold(rewritten_after, position) <==> (indices_hold(rewritten, position) || position == changed_position),
    ensures
        previous_leaves_keep_their_identities(before, containers_after),
        leaves_split_off_by_this_publish_are_named_after_their_first_record(before, containers_after, checkpoint_txg, inode_tree),
        rewritten_leaves_are_listed_in_order(rewritten_after, containers_after.len() as int),
        leaves_not_rewritten_are_carried(before, containers_after, rewritten_after),
{
    reveal(previous_leaves_keep_their_identities);
    reveal(leaves_split_off_by_this_publish_are_named_after_their_first_record);
    reveal(rewritten_leaves_are_listed_in_order);
        assert forall|position: int| 0 <= position < before.len() implies (#[trigger] containers_after[position]).identity
        == before[position].identity && containers_after[position].separator_key_in_the_parent_entry
        == before[position].separator_key_in_the_parent_entry by {
        assert(containers[position].identity == before[position].identity);
    }
    assert forall|position: int| before.len() <= position < containers_after.len() implies leaf_container_is_split_off_by_this_publish(
        #[trigger] containers_after[position],
        checkpoint_txg,
        inode_tree,
    ) by {
        assert(leaf_container_is_split_off_by_this_publish(containers[position], checkpoint_txg, inode_tree));
    }
    assert forall|listed: int| 0 <= listed < rewritten_after.len() implies (#[trigger] rewritten_after[listed]).0
        < containers_after.len() by {
        reveal(indices_hold);
        let position = rewritten_after[listed].0 as int;
        assert(indices_hold(rewritten_after, position));
        if indices_hold(rewritten, position) {
            let earlier = choose|earlier: int| 0 <= earlier < rewritten.len() && (#[trigger] rewritten[earlier]).0 == position;
        }
    }
    assert forall|position: int|
        #![trigger containers_after[position]]
        #![trigger indices_hold(rewritten_after, position)]
        0 <= position < containers_after.len() && !indices_hold(rewritten_after, position) implies position < before.len()
            && leaf_containers_are_the_same(containers_after[position], before[position]) by {
        assert(!indices_hold(rewritten, position));
        assert(position != changed_position);
        assert(containers_after[position] == containers[position]);
        assert(0 <= position < containers.len() && !indices_hold(rewritten, position));
        assert(leaf_containers_are_the_same(containers[position], before[position]));
        assert(position < before.len());
    }
}

/// 在末尾添一片照分裂纪律命名的新容器、把它记进重写清单之后，四样账目照样成立。
proof fn lemma_adding_a_split_off_leaf_keeps_the_bookkeeping(
    before: Seq<InodeLeafContainer>,
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
    containers: Seq<InodeLeafContainer>,
    rewritten: Seq<InodeLeafContainerIndexInTree>,
    split_off: InodeLeafContainer,
    rewritten_after: Seq<InodeLeafContainerIndexInTree>,
)
    requires
        previous_leaves_keep_their_identities(before, containers),
        leaves_split_off_by_this_publish_are_named_after_their_first_record(before, containers, checkpoint_txg, inode_tree),
        rewritten_leaves_are_listed_in_order(rewritten, containers.len() as int),
        leaves_not_rewritten_are_carried(before, containers, rewritten),
        leaf_container_is_split_off_by_this_publish(split_off, checkpoint_txg, inode_tree),
        indices_strictly_ascend(rewritten_after),
        forall|position: int|
            #[trigger] indices_hold(rewritten_after, position) <==> (indices_hold(rewritten, position) || position == containers.len()),
    ensures
        previous_leaves_keep_their_identities(before, containers.push(split_off)),
        leaves_split_off_by_this_publish_are_named_after_their_first_record(
            before,
            containers.push(split_off),
            checkpoint_txg,
            inode_tree,
        ),
        rewritten_leaves_are_listed_in_order(rewritten_after, containers.len() as int + 1),
        leaves_not_rewritten_are_carried(before, containers.push(split_off), rewritten_after),
{
    reveal(previous_leaves_keep_their_identities);
    reveal(leaves_split_off_by_this_publish_are_named_after_their_first_record);
    reveal(rewritten_leaves_are_listed_in_order);
        let containers_after = containers.push(split_off);
    assert forall|position: int| 0 <= position < before.len() implies (#[trigger] containers_after[position]).identity
        == before[position].identity && containers_after[position].separator_key_in_the_parent_entry
        == before[position].separator_key_in_the_parent_entry by {
        assert(containers_after[position] == containers[position]);
    }
    assert forall|position: int| before.len() <= position < containers_after.len() implies leaf_container_is_split_off_by_this_publish(
        #[trigger] containers_after[position],
        checkpoint_txg,
        inode_tree,
    ) by {
        if position < containers.len() {
            assert(containers_after[position] == containers[position]);
        }
    }
    assert forall|listed: int| 0 <= listed < rewritten_after.len() implies (#[trigger] rewritten_after[listed]).0
        < containers_after.len() by {
        reveal(indices_hold);
        let position = rewritten_after[listed].0 as int;
        assert(indices_hold(rewritten_after, position));
        if indices_hold(rewritten, position) {
            let earlier = choose|earlier: int| 0 <= earlier < rewritten.len() && (#[trigger] rewritten[earlier]).0 == position;
        }
    }
    assert forall|position: int|
        #![trigger containers_after[position]]
        #![trigger indices_hold(rewritten_after, position)]
        0 <= position < containers_after.len() && !indices_hold(rewritten_after, position) implies position < before.len()
            && leaf_containers_are_the_same(containers_after[position], before[position]) by {
        assert(!indices_hold(rewritten, position));
        assert(position != containers.len());
        assert(containers_after[position] == containers[position]);
        assert(0 <= position < containers.len() && !indices_hold(rewritten, position));
        assert(leaf_containers_are_the_same(containers[position], before[position]));
        assert(position < before.len());
    }
}

/// 原地换掉第 `holding_position` 片的第 `position_in_container` 条（同号）、把这一片记进重写清单之后，[`writing_progress`] 照样成立。
proof fn lemma_overwriting_a_record_keeps_the_writing_progress(
    before: Seq<InodeLeafContainer>,
    written_so_far: Seq<InodeRecord>,
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
    containers: Seq<InodeLeafContainer>,
    rewritten: Seq<InodeLeafContainerIndexInTree>,
    holding_position: int,
    position_in_container: int,
    record: InodeRecord,
    containers_after: Seq<InodeLeafContainer>,
    rewritten_after: Seq<InodeLeafContainerIndexInTree>,
)
    requires
        writing_progress(before, written_so_far, checkpoint_txg, inode_tree, containers, rewritten),
        0 <= holding_position < containers.len(),
        0 <= position_in_container < containers[holding_position].records@.len(),
        containers[holding_position].records@[position_in_container].inode == record.inode,
        containers_after.len() == containers.len(),
        forall|position: int|
            0 <= position < containers.len() && position != holding_position ==> #[trigger] containers_after[position]
                == containers[position],
        containers_after[holding_position].identity == containers[holding_position].identity,
        containers_after[holding_position].separator_key_in_the_parent_entry
            == containers[holding_position].separator_key_in_the_parent_entry,
        containers_after[holding_position].records@ == containers[holding_position].records@.update(position_in_container, record),
        indices_strictly_ascend(rewritten_after),
        forall|position: int|
            #[trigger] indices_hold(rewritten_after, position) <==> (indices_hold(rewritten, position) || position == holding_position),
    ensures
        writing_progress(before, written_so_far.push(record), checkpoint_txg, inode_tree, containers_after, rewritten_after),
{
    let flattened = leaf_records(containers);
    let start = records_before_container(containers, holding_position);
    let global = start + position_in_container;
    assert(containers_after =~= containers.update(holding_position, containers_after[holding_position]));
    lemma_leaf_records_around_one_container(containers, holding_position, containers_after[holding_position]);
    lemma_a_record_inside_the_leaf_records(containers, holding_position, position_in_container);
    assert(leaf_records(containers_after) =~= flattened.update(global, record));
    lemma_first_position_of_an_inode_in_ascending_records(flattened, record.inode, global, 0);
    assert(written_so_far.push(record).drop_last() =~= written_so_far);
    lemma_updating_a_record_with_the_same_inode_keeps_the_records_ascending(flattened, global, record);
    assert forall|position: int| 0 <= position < containers_after.len() implies 1 <= (#[trigger] containers_after[position]).records@.len()
        <= 233 by {
        assert(1 <= containers[position].records@.len() <= 233);
    }
    assert(containers[holding_position].records@.len() >= 1);
    assert(containers_after[holding_position].records@[0].inode == containers[holding_position].records@[0].inode);
    lemma_changing_one_leaf_in_place_keeps_the_bookkeeping(
        before,
        checkpoint_txg,
        inode_tree,
        containers,
        rewritten,
        holding_position,
        containers_after,
        rewritten_after,
    );
}

/// 往最右那片（还装得下）末尾追加一条比树里都大的记录、把这一片记进重写清单之后，[`writing_progress`] 照样成立。
proof fn lemma_appending_to_the_rightmost_leaf_keeps_the_writing_progress(
    before: Seq<InodeLeafContainer>,
    written_so_far: Seq<InodeRecord>,
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
    containers: Seq<InodeLeafContainer>,
    rewritten: Seq<InodeLeafContainerIndexInTree>,
    record: InodeRecord,
    containers_after: Seq<InodeLeafContainer>,
    rewritten_after: Seq<InodeLeafContainerIndexInTree>,
)
    requires
        writing_progress(before, written_so_far, checkpoint_txg, inode_tree, containers, rewritten),
        containers.len() >= 1,
        containers.last().records@.len() < 233,
        !records_hold_inode(leaf_records(containers), record.inode),
        containers.last().records@.last().inode < record.inode,
        containers_after.len() == containers.len(),
        forall|position: int| 0 <= position < containers.len() - 1 ==> #[trigger] containers_after[position] == containers[position],
        containers_after.last().identity == containers.last().identity,
        containers_after.last().separator_key_in_the_parent_entry == containers.last().separator_key_in_the_parent_entry,
        containers_after.last().records@ == containers.last().records@.push(record),
        indices_strictly_ascend(rewritten_after),
        forall|position: int|
            #[trigger] indices_hold(rewritten_after, position) <==> (indices_hold(rewritten, position) || position == containers.len()
                - 1),
    ensures
        writing_progress(before, written_so_far.push(record), checkpoint_txg, inode_tree, containers_after, rewritten_after),
{
    let last = containers.len() - 1;
    let flattened = leaf_records(containers);
    assert(containers_after =~= containers.update(last, containers_after[last]));
    lemma_leaf_records_around_one_container(containers, last, containers_after[last]);
    assert(containers.subrange(last + 1, containers.len() as int) =~= Seq::<InodeLeafContainer>::empty());
    assert(leaf_records(containers_after) =~= flattened.push(record));
    lemma_first_position_of_an_absent_inode(flattened, record.inode, 0);
    assert(written_so_far.push(record).drop_last() =~= written_so_far);
    lemma_the_last_leaf_record(containers);
    lemma_pushing_a_larger_record_keeps_the_records_ascending(flattened, record);
    assert forall|position: int| 0 <= position < containers_after.len() implies 1 <= (#[trigger] containers_after[position]).records@.len()
        <= 233 by {
        assert(1 <= containers[position].records@.len() <= 233);
    }
    assert(containers[last].records@.len() >= 1);
    assert(containers_after[last].records@[0] == containers[last].records@[0]);
    lemma_changing_one_leaf_in_place_keeps_the_bookkeeping(
        before,
        checkpoint_txg,
        inode_tree,
        containers,
        rewritten,
        last,
        containers_after,
        rewritten_after,
    );
}

/// 最右那片满了（或树是空的）、在末尾分裂出一片只装这一条的新容器、把它记进重写清单之后，[`writing_progress`] 照样成立。
proof fn lemma_splitting_off_a_leaf_keeps_the_writing_progress(
    before: Seq<InodeLeafContainer>,
    written_so_far: Seq<InodeRecord>,
    checkpoint_txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
    containers: Seq<InodeLeafContainer>,
    rewritten: Seq<InodeLeafContainerIndexInTree>,
    record: InodeRecord,
    split_off: InodeLeafContainer,
    rewritten_after: Seq<InodeLeafContainerIndexInTree>,
)
    requires
        writing_progress(before, written_so_far, checkpoint_txg, inode_tree, containers, rewritten),
        !records_hold_inode(leaf_records(containers), record.inode),
        containers.len() >= 1 ==> containers.last().records@.last().inode < record.inode,
        split_off.records@ == seq![record],
        split_off.identity == (PackedIdentity {
            birth_tree: inode_tree,
            record_type: PACKED_TYPE_INODE,
            container: record.inode,
            container_birth: checkpoint_txg,
        }),
        split_off.separator_key_in_the_parent_entry == record.inode,
        indices_strictly_ascend(rewritten_after),
        forall|position: int|
            #[trigger] indices_hold(rewritten_after, position) <==> (indices_hold(rewritten, position) || position == containers.len()),
    ensures
        writing_progress(before, written_so_far.push(record), checkpoint_txg, inode_tree, containers.push(split_off), rewritten_after),
{
    let containers_after = containers.push(split_off);
    let flattened = leaf_records(containers);
    assert(containers_after.drop_last() =~= containers);
    assert(leaf_records(containers_after) =~= flattened.push(record));
    lemma_first_position_of_an_absent_inode(flattened, record.inode, 0);
    assert(written_so_far.push(record).drop_last() =~= written_so_far);
    if containers.len() >= 1 {
        lemma_the_last_leaf_record(containers);
    }
    lemma_pushing_a_larger_record_keeps_the_records_ascending(flattened, record);
    assert forall|position: int| 0 <= position < containers_after.len() implies 1 <= (#[trigger] containers_after[position]).records@.len()
        <= 233 by {
        if position < containers.len() {
            assert(1 <= containers[position].records@.len() <= 233);
        }
    }
    assert(leaf_container_is_split_off_by_this_publish(split_off, checkpoint_txg, inode_tree));
    lemma_adding_a_split_off_leaf_keeps_the_bookkeeping(
        before,
        checkpoint_txg,
        inode_tree,
        containers,
        rewritten,
        split_off,
        rewritten_after,
    );
}

// ===== 删除与合并 =====

/// 从 inode 树的叶容器里删掉 `inode` 那一条记录（unlink 最后一批那次发布：D8（核心索引结构） 已定项 16 第 2 条「删 inode 记录」），
/// 再照 E176 M3 那一臂合并（文件头那几条）。
///
/// - 删完没并、没借、没删空 ⇒ 只有那一片进重写清单，身份不变（COW 重写身份不变，D8 已定项 6）；
/// - 那一片删空了 ⇒ 整片不落盘（「记录数为 0 的类型 2 容器不落盘」），身份进 `retired`；左吸收右的那一对，左片进重写清单、右片身份进 `retired`；
///   左从右借的那一对两片都进重写清单、身份都不变，右片的分隔 key 改成它这一版的最小 key；
/// - 身份消失的那几片右边的每一片位置左移，内容没变的照抄（`origins` 记着上一版的叶序），不进重写清单。
///
/// 删的是树里最后一条时交回零片容器：inode 树写零条目根（key 区间 `[0, 0]`，D8 已定项 6），那一版是「有树、文件集合为空」
/// （D8 已定项 16 第 5 条），怎么装它归发布路径。
///
/// # Errors
/// 树里没有这条记录 ⇒ [`InodeTreeWriteRefusal::RecordToRemoveIsNotInTheTree`]（在任何落盘动作之前）。
pub fn remove_record_from_leaf_containers(
    containers_before_this_publish: &[InodeLeafContainer],
    inode: InodeNumber,
) -> (after: Result<InodeLeafContainersAfterThisPublish, InodeTreeWriteRefusal>)
    requires
        leaf_containers_are_well_formed(containers_before_this_publish@),
        containers_before_this_publish.len() <= u32::MAX,
    ensures
        match after {
            Ok(after) => {
                &&& records_hold_inode(leaf_records(containers_before_this_publish@), inode.0)
                &&& leaf_records(after.containers@) == leaf_records(containers_before_this_publish@).remove(
                    first_position_of_inode(leaf_records(containers_before_this_publish@), inode.0, 0),
                )
                &&& leaf_containers_are_well_formed(after.containers@)
                &&& removal_keeps_track_of_the_previous_version(containers_before_this_publish@, after)
            },
            Err(refusal) => refusal == (InodeTreeWriteRefusal::RecordToRemoveIsNotInTheTree { inode })
                && !records_hold_inode(leaf_records(containers_before_this_publish@), inode.0),
        },
{
    let ghost before = containers_before_this_publish@;
    proof {
        assert forall|position: int| 0 <= position < before.len() implies inode_records_strictly_ascend(
            #[trigger] before[position].records@,
        ) by {
            lemma_each_container_of_well_formed_leaves_ascends(before, position);
        }
    }
    let Some(index) = position_of_the_container_holding(containers_before_this_publish, inode)
    else {
        proof {
            lemma_an_inode_absent_from_every_container_is_absent_from_the_leaf_records(before, inode.0);
        }
        return Err(InodeTreeWriteRefusal::RecordToRemoveIsNotInTheTree { inode });
    };
    let mut tree = LeafContainersBeingMerged::from_the_previous_version(containers_before_this_publish);
    let holding_position = index.position();
    let ghost tree_at_the_start = tree.containers@;
    proof {
        assert(leaf_containers_are_the_same(tree_at_the_start[holding_position as int].contents, before[holding_position as int]));
    }
    let holding = &mut tree.containers[holding_position];
    let position_in_container = search_records_by_inode(&holding.contents.records, inode.0)
        .expect("position_of_the_container_holding 找到的就是这条记录所在的容器");
    holding.contents.records.remove(position_in_container);
    holding.is_changed_this_publish = true;
    let ghost records_after_removal = leaf_records(before).remove(first_position_of_inode(leaf_records(before), inode.0, 0));
    proof {
        let holding_after = tree.containers@[holding_position as int].contents;
        let removed = before.update(holding_position as int, holding_after);
        assert forall|position: int| 0 <= position < removed.len() implies #[trigger] leaf_containers_are_the_same(
            contents_being_merged(tree.containers@)[position],
            removed[position],
        ) by {
            assert(leaf_containers_are_the_same(tree_at_the_start[position].contents, before[position]));
            if position != holding_position {
                assert(tree.containers@[position] == tree_at_the_start[position]);
            }
        }
        lemma_the_same_leaf_containers_have_the_same_leaf_records(contents_being_merged(tree.containers@), removed);
        lemma_removing_a_record_keeps_the_rest_of_the_leaf_records(
            before,
            holding_position as int,
            position_in_container as int,
            holding_after,
        );
        lemma_a_record_inside_the_leaf_records(before, holding_position as int, position_in_container as int);
        lemma_first_position_of_an_inode_in_ascending_records(
            leaf_records(before),
            inode.0,
            records_before_container(before, holding_position as int) + position_in_container,
            0,
        );
        lemma_removing_a_record_keeps_the_records_ascending(
            leaf_records(before),
            records_before_container(before, holding_position as int) + position_in_container,
        );
        assert forall|position: int| 0 <= position < tree.containers@.len() implies {
            let container = #[trigger] tree.containers@[position];
            &&& container.previous_index.0 < before.len()
            &&& container.contents.identity == before[container.previous_index.0 as int].identity
            &&& !container.is_changed_this_publish ==> leaf_containers_are_the_same(
                container.contents,
                before[container.previous_index.0 as int],
            )
        } by {
            assert(leaf_containers_are_the_same(tree_at_the_start[position].contents, before[position]));
            if position != holding_position {
                assert(tree.containers@[position] == tree_at_the_start[position]);
            }
        }
        assert(records_hold_inode(leaf_records(before), inode.0)) by {
            assert(leaf_records(before)[records_before_container(before, holding_position as int) + position_in_container].inode
                == inode.0);
        }
    }
    if tree.containers[holding_position].contents.records.is_empty() {
        let ghost tree_before_retiring = tree.containers@;
        tree.retire(holding_position);
        proof {
            assert(contents_being_merged(tree.containers@) =~= contents_being_merged(tree_before_retiring).remove(
                holding_position as int,
            ));
            lemma_removing_an_empty_leaf_keeps_the_leaf_records(contents_being_merged(tree_before_retiring), holding_position as int);
            assert forall|position: int| 0 <= position < tree.containers@.len() implies 1 <= (
            #[trigger] tree.containers@[position]).contents.records@.len() <= 233 by {
                let in_the_old = if position < holding_position { position } else { position + 1 };
                assert(tree.containers@[position] == tree_before_retiring[in_the_old]);
                assert(tree_before_retiring[in_the_old] == tree_at_the_start[in_the_old]);
                assert(leaf_containers_are_the_same(tree_at_the_start[in_the_old].contents, before[in_the_old]));
            }
        }
        let has_new_neighbours = holding_position > 0 && holding_position < tree.containers.len();
        if has_new_neighbours {
            tree.absorb_until_no_pair_fits(holding_position - 1);
        }
    } else {
        proof {
            assert forall|position: int| 0 <= position < tree.containers@.len() implies 1 <= (
            #[trigger] tree.containers@[position]).contents.records@.len() <= 233 by {
                assert(leaf_containers_are_the_same(tree_at_the_start[position].contents, before[position]));
                if position != holding_position {
                    assert(tree.containers@[position] == tree_at_the_start[position]);
                }
            }
        }
        let absorbed_at_least_once = tree.absorb_until_no_pair_fits(holding_position);
        if !absorbed_at_least_once {
            tree.borrow_from_the_right_neighbour_if_below_the_floor(holding_position);
        }
    }
    let ghost tree_at_the_end = tree.containers@;
    let after = tree.after_this_publish();
    proof {
        assert forall|position: int| 0 <= position < after.containers@.len() implies 1 <= (#[trigger] after.containers@[position]).records@.len()
            <= 233 by {
            assert(after.containers@[position] == tree_at_the_end[position].contents);
        }
    }
    Ok(after)
}

/// [`remove_record_from_leaf_containers`] 交回的样子与上一版对得上的那几样：重写清单升序、与 `origins` 一一对应；照抄的那几片与上一版那一份
/// 逐字节相同；退役清单按上一版叶序不减、每一项是上一版那一片的身份。
pub open spec fn removal_keeps_track_of_the_previous_version(
    before: Seq<InodeLeafContainer>,
    after: InodeLeafContainersAfterThisPublish,
) -> bool {
    &&& after.origins.len() == after.containers.len()
    &&& indices_strictly_ascend(after.rewritten@)
    &&& forall|listed: int| 0 <= listed < after.rewritten.len() ==> (#[trigger] after.rewritten@[listed]).0 < after.containers.len()
    &&& forall|position: int|
        0 <= position < after.origins.len() ==> ((#[trigger] after.origins@[position]) is RewrittenThisPublish <==> indices_hold(
            after.rewritten@,
            position,
        ))
    &&& forall|position: int|
        0 <= position < after.origins.len() ==> match #[trigger] after.origins@[position] {
            InodeLeafContainerOrigin::CarriedFrom(previous_index) => previous_index.0 < before.len()
                && leaf_containers_are_the_same(after.containers@[position], before[previous_index.0 as int]),
            InodeLeafContainerOrigin::RewrittenThisPublish => true,
        }
    &&& forall|earlier: int, later: int|
        0 <= earlier < later < after.retired.len() ==> (#[trigger] after.retired@[earlier]).previous_index.0
            <= (#[trigger] after.retired@[later]).previous_index.0
    &&& forall|listed: int|
        0 <= listed < after.retired.len() ==> (#[trigger] after.retired@[listed]).previous_index.0 < before.len()
            && after.retired@[listed].identity == before[after.retired@[listed].previous_index.0 as int].identity
}

/// 一次删除发布里合并到一半的叶容器：每片带着它在上一版里的叶序与这次有没有改过；身份消失的那几片挪进 `retired`。
struct LeafContainersBeingMerged {
    containers: Vec<LeafContainerBeingMerged>,
    retired: Vec<RetiredInodeLeafContainer>,
    /// 上一版的叶容器（只给 Verus 看：每片记得的上一版叶序指的就是它）。
    #[allow(dead_code, reason = "只给 Verus 看的幽灵字段，产品构建里不读它")]
    previous_version: Ghost<Seq<InodeLeafContainer>>,
}

struct LeafContainerBeingMerged {
    contents: InodeLeafContainer,
    previous_index: InodeLeafContainerIndexInTree,
    is_changed_this_publish: bool,
}

/// 合并到一半的叶容器的内容（规格里用）。
spec fn contents_being_merged(containers: Seq<LeafContainerBeingMerged>) -> Seq<InodeLeafContainer> {
    containers.map_values(|container: LeafContainerBeingMerged| container.contents)
}

/// 合并到一半时每片都记得它在上一版里是哪一片：叶序在上一版以内、身份就是那一片的，没改过的与那一片逐字节相同；退役清单按上一版叶序不减、
/// 每一项是上一版那一片的身份。
spec fn merging_keeps_track_of_the_previous_version(
    containers: Seq<LeafContainerBeingMerged>,
    retired: Seq<RetiredInodeLeafContainer>,
    before: Seq<InodeLeafContainer>,
) -> bool {
    &&& containers.len() <= before.len()
    &&& forall|position: int|
        0 <= position < containers.len() ==> {
            let container = #[trigger] containers[position];
            &&& container.previous_index.0 < before.len()
            &&& container.contents.identity == before[container.previous_index.0 as int].identity
            &&& !container.is_changed_this_publish ==> leaf_containers_are_the_same(
                container.contents,
                before[container.previous_index.0 as int],
            )
        }
    &&& forall|earlier: int, later: int|
        0 <= earlier < later < retired.len() ==> (#[trigger] retired[earlier]).previous_index.0
            <= (#[trigger] retired[later]).previous_index.0
    &&& forall|listed: int|
        0 <= listed < retired.len() ==> (#[trigger] retired[listed]).previous_index.0 < before.len()
            && retired[listed].identity == before[retired[listed].previous_index.0 as int].identity
}

/// 合并到一半的叶容器的记录连起来（规格里用）。
spec fn records_being_merged(containers: Seq<LeafContainerBeingMerged>) -> Seq<InodeRecord> {
    leaf_records(contents_being_merged(containers))
}

/// 合并到一半的每片都装着 1 到 233 条。
spec fn every_leaf_being_merged_holds_one_to_a_full_container(containers: Seq<LeafContainerBeingMerged>) -> bool {
    forall|position: int|
        0 <= position < containers.len() ==> 1 <= (#[trigger] containers[position]).contents.records@.len() <= 233
}

/// 合并到一半时的不变量：每片 1 到 233 条，记录连起来严格升序且就是 `records`，每片记得上一版。
spec fn merging_is_on_track(
    containers: Seq<LeafContainerBeingMerged>,
    retired: Seq<RetiredInodeLeafContainer>,
    before: Seq<InodeLeafContainer>,
    records: Seq<InodeRecord>,
) -> bool {
    &&& every_leaf_being_merged_holds_one_to_a_full_container(containers)
    &&& leaf_records(contents_being_merged(containers)) == records
    &&& inode_records_strictly_ascend(records)
    &&& merging_keeps_track_of_the_previous_version(containers, retired, before)
}

impl LeafContainersBeingMerged {
    fn from_the_previous_version(containers: &[InodeLeafContainer]) -> (tree: Self)
        requires
            containers.len() <= u32::MAX,
        ensures
            tree.previous_version@ == containers@,
            tree.containers.len() == containers.len(),
            forall|position: int|
                0 <= position < tree.containers.len() ==> #[trigger] leaf_containers_are_the_same(
                    tree.containers@[position].contents,
                    containers@[position],
                ),
            tree.retired@.len() == 0,
            forall|position: int|
                0 <= position < tree.containers.len() ==> {
                    let container = #[trigger] tree.containers@[position];
                    &&& container.previous_index.0 == position
                    &&& !container.is_changed_this_publish
                },
    {
        let mut being_merged: Vec<LeafContainerBeingMerged> = Vec::new();
        let mut position: usize = 0;
        while position < containers.len()
            invariant
                position <= containers.len() <= u32::MAX,
                being_merged.len() == position,
                forall|done: int|
                    0 <= done < position ==> {
                        let container = #[trigger] being_merged@[done];
                        &&& leaf_containers_are_the_same(container.contents, containers@[done])
                        &&& container.previous_index.0 == done
                        &&& !container.is_changed_this_publish
                    },
            decreases containers.len() - position,
        {
            being_merged.push(
                LeafContainerBeingMerged {
                    contents: copy_of_one_leaf_container(&containers[position]),
                    previous_index: InodeLeafContainerIndexInTree::of_position(position),
                    is_changed_this_publish: false,
                },
            );
            position += 1;
        }
        Self { containers: being_merged, retired: Vec::new(), previous_version: Ghost(containers@) }
    }

    fn records_at(&self, position: usize) -> (records: usize)
        requires
            position < self.containers.len(),
        ensures
            records == self.containers@[position as int].contents.records@.len(),
    {
        self.containers[position].contents.records.len()
    }

    /// 这一对（`left_position` 与它右邻）合起来装得下一片（≤ 233 条）。
    fn pair_fits_in_one_container(&self, left_position: usize) -> (fits: bool)
        requires
            left_position + 1 < self.containers.len(),
            every_leaf_being_merged_holds_one_to_a_full_container(self.containers@),
        ensures
            fits == (self.containers@[left_position as int].contents.records@.len() + self.containers@[left_position + 1].contents.records@.len()
                <= 233),
    {
        proof {
            assert(self.containers@[left_position as int].contents.records@.len() <= 233);
            assert(self.containers@[left_position + 1].contents.records@.len() <= 233);
        }
        u64::try_from(self.records_at(left_position) + self.records_at(left_position + 1))
            .expect("两片之和至多 466 条")
            <= INODE_LEAF_RECORDS
    }

    /// 那一片的身份消失（删空摘掉或被吸收）：从这一版拿掉，身份连同上一版的叶序插进 `retired`（按上一版叶序，同序号的排在已有的之后）。
    fn retire(&mut self, position: usize) -> (retired: InodeLeafContainer)
        requires
            position < old(self).containers.len(),
            merging_keeps_track_of_the_previous_version(old(self).containers@, old(self).retired@, old(self).previous_version@),
        ensures
            final(self).previous_version == old(self).previous_version,
            final(self).containers@ == old(self).containers@.remove(position as int),
            retired == old(self).containers@[position as int].contents,
            final(self).retired@.len() == old(self).retired@.len() + 1,
            merging_keeps_track_of_the_previous_version(
                final(self).containers@,
                final(self).retired@,
                final(self).previous_version@,
            ),
    {
        let ghost before = self.previous_version@;
        let removed = self.containers.remove(position);
        let entry = RetiredInodeLeafContainer { previous_index: removed.previous_index, identity: removed.contents.identity };
        let ghost retired_before = self.retired@;
        let mut insertion_point: usize = 0;
        while insertion_point < self.retired.len() && self.retired[insertion_point].previous_index.0 <= entry.previous_index.0
            invariant
                self.retired@ == retired_before,
                insertion_point <= self.retired.len(),
                forall|earlier: int| 0 <= earlier < insertion_point ==> (#[trigger] self.retired@[earlier]).previous_index.0
                    <= entry.previous_index.0,
            decreases self.retired.len() - insertion_point,
        {
            insertion_point += 1;
        }
        self.retired.insert(insertion_point, entry);
        proof {
            let retired_after = self.retired@;
            assert forall|earlier: int, later: int| 0 <= earlier < later < retired_after.len() implies (#[trigger] retired_after[earlier]).previous_index.0
                <= (#[trigger] retired_after[later]).previous_index.0 by {
                if later < insertion_point {
                    assert(retired_after[earlier] == retired_before[earlier]);
                    assert(retired_after[later] == retired_before[later]);
                } else if later == insertion_point {
                    assert(retired_after[earlier] == retired_before[earlier]);
                } else if earlier < insertion_point {
                    assert(retired_after[earlier] == retired_before[earlier]);
                    assert(retired_after[later] == retired_before[later - 1]);
                    assert(retired_before[earlier].previous_index.0 <= entry.previous_index.0);
                    assert(retired_before[insertion_point as int].previous_index.0 > entry.previous_index.0);
                    if later - 1 > insertion_point {
                        assert(retired_before[insertion_point as int].previous_index.0 <= retired_before[later - 1].previous_index.0);
                    }
                } else if earlier == insertion_point {
                    assert(retired_after[later] == retired_before[later - 1]);
                    assert(retired_before[insertion_point as int].previous_index.0 > entry.previous_index.0);
                    if later - 1 > insertion_point {
                        assert(retired_before[insertion_point as int].previous_index.0 <= retired_before[later - 1].previous_index.0);
                    }
                } else {
                    assert(retired_after[earlier] == retired_before[earlier - 1]);
                    assert(retired_after[later] == retired_before[later - 1]);
                }
            }
            assert forall|listed: int| 0 <= listed < retired_after.len() implies (#[trigger] retired_after[listed]).previous_index.0
                < before.len() && retired_after[listed].identity == before[retired_after[listed].previous_index.0 as int].identity by {
                if listed < insertion_point {
                    assert(retired_after[listed] == retired_before[listed]);
                } else if listed > insertion_point {
                    assert(retired_after[listed] == retired_before[listed - 1]);
                }
            }
            assert forall|any_position: int| 0 <= any_position < self.containers@.len() implies {
                let container = #[trigger] self.containers@[any_position];
                &&& container.previous_index.0 < before.len()
                &&& container.contents.identity == before[container.previous_index.0 as int].identity
                &&& !container.is_changed_this_publish ==> leaf_containers_are_the_same(
                    container.contents,
                    before[container.previous_index.0 as int],
                )
            } by {
                let in_the_old = if any_position < position { any_position } else { any_position + 1 };
                assert(self.containers@[any_position] == old(self).containers@[in_the_old]);
            }
        }
        removed.contents
    }


    /// 左片吸收它右邻：左片保留身份与分隔 key，右片的记录接在后面，右片身份消失。
    fn absorb_the_right_neighbour(&mut self, left_position: usize)
        requires
            left_position + 1 < old(self).containers.len(),
            merging_is_on_track(
                old(self).containers@,
                old(self).retired@,
                old(self).previous_version@,
                records_being_merged(old(self).containers@),
            ),
            old(self).containers@[left_position as int].contents.records@.len() + old(self).containers@[left_position
                + 1].contents.records@.len() <= 233,
        ensures
            final(self).previous_version == old(self).previous_version,
            final(self).containers.len() == old(self).containers.len() - 1,
            merging_is_on_track(
                final(self).containers@,
                final(self).retired@,
                final(self).previous_version@,
                records_being_merged(old(self).containers@),
            ),
    {
        let ghost before = self.previous_version@;
        let ghost containers_before = self.containers@;
        let mut absorbed = self.retire(left_position + 1);
        let survivor = &mut self.containers[left_position];
        survivor.contents.records.append(&mut absorbed.records);
        survivor.is_changed_this_publish = true;
        proof {
            let containers_after = self.containers@;
            let merged = containers_after[left_position as int].contents;
            assert(contents_being_merged(containers_after) =~= contents_being_merged(containers_before).remove(
                left_position + 1,
            ).update(left_position as int, merged));
            lemma_merging_two_neighbours_keeps_the_leaf_records(
                contents_being_merged(containers_before),
                left_position as int,
                merged,
            );
            assert forall|position: int| 0 <= position < containers_after.len() implies 1 <= (
            #[trigger] containers_after[position]).contents.records@.len() <= 233 by {
                if position < left_position {
                    assert(containers_after[position] == containers_before[position]);
                } else if position > left_position {
                    assert(containers_after[position] == containers_before[position + 1]);
                }
            }
            assert forall|position: int| 0 <= position < containers_after.len() implies {
                let container = #[trigger] containers_after[position];
                &&& container.previous_index.0 < before.len()
                &&& container.contents.identity == before[container.previous_index.0 as int].identity
                &&& !container.is_changed_this_publish ==> leaf_containers_are_the_same(
                    container.contents,
                    before[container.previous_index.0 as int],
                )
            } by {
                if position < left_position {
                    assert(containers_after[position] == containers_before[position]);
                } else if position > left_position {
                    assert(containers_after[position] == containers_before[position + 1]);
                } else {
                    assert(containers_after[position].contents.identity == containers_before[position].contents.identity);
                    assert(containers_after[position].previous_index == containers_before[position].previous_index);
                }
            }
        }
    }

    /// 从 `start_position` 那片起先查它与右邻、再查左邻与它，两片之和 ≤ 233 就左吸收右；并完从幸存那片接着查，直到没有一对装得下
    /// （E176 M3 主取样点「先查右边」）。交回这一趟有没有并过；一次都没并过时什么都没动，且它与右邻装不进一片。
    ///
    /// 迭代上界是片数：每一轮要么并掉一片、要么退出；跨轮携带的是「当前那片」的位置，它只往左挪（左邻吸收它时换成左邻），
    /// 被摘掉的总在它右边或就是它被左邻吸收，所以位置一直指着同一片幸存的叶。
    fn absorb_until_no_pair_fits(&mut self, start_position: usize) -> (absorbed_at_least_once: bool)
        requires
            start_position < old(self).containers.len(),
            merging_is_on_track(
                old(self).containers@,
                old(self).retired@,
                old(self).previous_version@,
                records_being_merged(old(self).containers@),
            ),
        ensures
            final(self).previous_version == old(self).previous_version,
            merging_is_on_track(
                final(self).containers@,
                final(self).retired@,
                final(self).previous_version@,
                records_being_merged(old(self).containers@),
            ),
            !absorbed_at_least_once ==> final(self).containers@ == old(self).containers@ && final(self).retired@ == old(
                self,
            ).retired@ && !(start_position + 1 < old(self).containers.len() && old(self).containers@[start_position
                as int].contents.records@.len() + old(self).containers@[start_position + 1].contents.records@.len() <= 233),
    {
        let ghost containers_at_the_start = self.containers@;
        let ghost retired_at_the_start = self.retired@;
        let ghost records = records_being_merged(self.containers@);
        let mut current_position = start_position;
        let mut absorbed_at_least_once = false;
        loop
            invariant
                containers_at_the_start == old(self).containers@,
                retired_at_the_start == old(self).retired@,
                current_position < self.containers.len(),
                self.previous_version == old(self).previous_version,
                records == records_being_merged(containers_at_the_start),
                merging_is_on_track(self.containers@, self.retired@, self.previous_version@, records),
                !absorbed_at_least_once ==> current_position == start_position && self.containers@ == containers_at_the_start
                    && self.retired@ == retired_at_the_start,
            decreases self.containers.len(),
        {
            let right_pair_fits = current_position + 1 < self.containers.len() && self.pair_fits_in_one_container(
                current_position,
            );
            let left_pair_fits = current_position > 0 && self.pair_fits_in_one_container(current_position - 1);
            if right_pair_fits {
                self.absorb_the_right_neighbour(current_position);
            } else if left_pair_fits {
                self.absorb_the_right_neighbour(current_position - 1);
                current_position -= 1;
            } else {
                return absorbed_at_least_once;
            }
            absorbed_at_least_once = true;
        }
    }

    /// 左从右借（D8（核心索引结构） 已定项 6；E176 M3）：`borrower_position` 那片记录数小于下限且有右邻时，把右邻最小的
    /// ⌊(右 − 左) ÷ 2⌋ 条挪到它末尾（均分：借完两片条数相差不超过 1），两片都保留身份，右邻的分隔 key 改成它这一版的最小 key（N1）；
    /// 借完从右邻起照 [`Self::absorb_until_no_pair_fits`] 并。它是最右那片时不借（右从左借会破「记录号 ≥ 容器号」）。
    ///
    /// # Panics
    /// 要借的条数不在 [1, 右邻记录数) 里（`requires` 排除的那一种）：调用方只在「从这片起一次都没并过」之后调它，那时它与右邻之和 > 233，
    /// 它又少于 117 条 ⇒ 右邻多于 116 条、借的条数 ⌊(右 − 左) ÷ 2⌋ ≥ 117 − 左 ≥ 1，且小于右邻的条数。
    fn borrow_from_the_right_neighbour_if_below_the_floor(&mut self, borrower_position: usize)
        requires
            borrower_position < old(self).containers.len(),
            merging_is_on_track(
                old(self).containers@,
                old(self).retired@,
                old(self).previous_version@,
                records_being_merged(old(self).containers@),
            ),
            borrower_position + 1 < old(self).containers.len() ==> old(self).containers@[borrower_position as int].contents.records@.len()
                + old(self).containers@[borrower_position + 1].contents.records@.len() > 233,
        ensures
            final(self).previous_version == old(self).previous_version,
            merging_is_on_track(
                final(self).containers@,
                final(self).retired@,
                final(self).previous_version@,
                records_being_merged(old(self).containers@),
            ),
    {
        let ghost before = self.previous_version@;
        let lender_position = borrower_position + 1;
        if lender_position >= self.containers.len() {
            return;
        }
        let records_in_the_borrower = self.records_at(borrower_position);
        if records_in_the_borrower >= records_below_which_a_leaf_borrows_from_its_right_neighbour() {
            return;
        }
        let records_in_the_lender = self.records_at(lender_position);
        proof {
            assert(records_in_the_lender <= 233);
            assert(records_in_the_borrower >= 1);
        }
        let moved_count = match records_in_the_lender.checked_sub(records_in_the_borrower) {
            Some(difference) => difference / 2,
            None => 0,
        };
        assert!(
            moved_count >= 1 && moved_count < records_in_the_lender,
            "借的条数要在 [1, 右邻记录数) 里：借方 {records_in_the_borrower} 条、右邻 {records_in_the_lender} 条、借 {moved_count} 条（这一对没并成说明两片之和 > 233，借方又少于 117 条）"
        );
        let ghost containers_before = self.containers@;
        let lender = &mut self.containers[lender_position];
        let mut moved = lender.contents.records.split_off(moved_count);
        std::mem::swap(&mut lender.contents.records, &mut moved);
        lender.contents.separator_key_in_the_parent_entry = lender
            .contents
            .records
            .first()
            .expect("右邻借出的条数少于它的记录数：借完至少还剩一条")
            .inode;
        lender.is_changed_this_publish = true;
        let borrower = &mut self.containers[borrower_position];
        borrower.contents.records.append(&mut moved);
        borrower.is_changed_this_publish = true;
        proof {
            let containers_after = self.containers@;
            assert(contents_being_merged(containers_after) =~= contents_being_merged(containers_before).update(
                borrower_position as int,
                containers_after[borrower_position as int].contents,
            ).update(lender_position as int, containers_after[lender_position as int].contents));
            assert(containers_after[borrower_position as int].contents.records@ + containers_after[lender_position as int].contents.records@
                =~= containers_before[borrower_position as int].contents.records@ + containers_before[lender_position as int].contents.records@);
            lemma_moving_records_between_neighbours_keeps_the_leaf_records(
                contents_being_merged(containers_before),
                borrower_position as int,
                containers_after[borrower_position as int].contents,
                containers_after[lender_position as int].contents,
            );
            assert forall|position: int| 0 <= position < containers_after.len() implies 1 <= (
            #[trigger] containers_after[position]).contents.records@.len() <= 233 by {
                if position != borrower_position && position != lender_position {
                    assert(containers_after[position] == containers_before[position]);
                }
            }
            assert forall|position: int| 0 <= position < containers_after.len() implies {
                let container = #[trigger] containers_after[position];
                &&& container.previous_index.0 < before.len()
                &&& container.contents.identity == before[container.previous_index.0 as int].identity
                &&& !container.is_changed_this_publish ==> leaf_containers_are_the_same(
                    container.contents,
                    before[container.previous_index.0 as int],
                )
            } by {
                if position != borrower_position && position != lender_position {
                    assert(containers_after[position] == containers_before[position]);
                } else {
                    assert(containers_after[position].contents.identity == containers_before[position].contents.identity);
                    assert(containers_after[position].previous_index == containers_before[position].previous_index);
                }
            }
        }
        self.absorb_until_no_pair_fits(lender_position);
    }

    fn after_this_publish(self) -> (after: InodeLeafContainersAfterThisPublish)
        requires
            merging_keeps_track_of_the_previous_version(self.containers@, self.retired@, self.previous_version@),
            self.previous_version@.len() <= u32::MAX,
        ensures
            after.containers@ == contents_being_merged(self.containers@),
            removal_keeps_track_of_the_previous_version(self.previous_version@, after),
    {
        let ghost before = self.previous_version@;
        let ghost containers_at_the_start = self.containers@;
        let mut tree = self;
        let mut rewritten: Vec<InodeLeafContainerIndexInTree> = Vec::new();
        let mut origins: Vec<InodeLeafContainerOrigin> = Vec::new();
        let mut containers: Vec<InodeLeafContainer> = Vec::new();
        let mut position: usize = 0;
        while position < tree.containers.len()
            invariant
                tree.containers.len() == containers_at_the_start.len(),
                containers_at_the_start.len() <= before.len() <= u32::MAX,
                merging_keeps_track_of_the_previous_version(containers_at_the_start, tree.retired@, before),
                forall|later: int|
                    position <= later < tree.containers.len() ==> #[trigger] tree.containers@[later] == containers_at_the_start[later],
                position <= tree.containers.len(),
                containers.len() == position,
                origins.len() == position,
                forall|done: int| 0 <= done < position ==> #[trigger] containers@[done] == containers_at_the_start[done].contents,
                forall|done: int|
                    0 <= done < position ==> #[trigger] origins@[done] == if containers_at_the_start[done].is_changed_this_publish {
                        InodeLeafContainerOrigin::RewrittenThisPublish
                    } else {
                        InodeLeafContainerOrigin::CarriedFrom(containers_at_the_start[done].previous_index)
                    },
                indices_strictly_ascend(rewritten@),
                forall|listed: int| 0 <= listed < rewritten.len() ==> (#[trigger] rewritten@[listed]).0 < position,
                forall|done: int|
                    0 <= done < position ==> (indices_hold(rewritten@, done) <==> #[trigger] containers_at_the_start[done].is_changed_this_publish),
            decreases tree.containers.len() - position,
        {
            let is_changed = tree.containers[position].is_changed_this_publish;
            let previous_index = tree.containers[position].previous_index;
            let mut contents = InodeLeafContainer {
                identity: tree.containers[position].contents.identity,
                separator_key_in_the_parent_entry: 0,
                records: Vec::new(),
            };
            let ghost rewritten_before = rewritten@;
            let ghost containers_before_the_swap = tree.containers@;
            std::mem::swap(&mut tree.containers[position].contents, &mut contents);
            proof {
                assert(contents == containers_before_the_swap[position as int].contents);
                assert forall|later: int| position < later < tree.containers.len() implies #[trigger] tree.containers@[later]
                    == containers_at_the_start[later] by {
                    assert(tree.containers@[later] == containers_before_the_swap[later]);
                }
            }
            if is_changed {
                rewritten.push(InodeLeafContainerIndexInTree::of_position(position));
                origins.push(InodeLeafContainerOrigin::RewrittenThisPublish);
            } else {
                origins.push(InodeLeafContainerOrigin::CarriedFrom(previous_index));
            }
            containers.push(contents);
            position += 1;
            proof {
                reveal(indices_hold);
                assert forall|done: int| 0 <= done < position implies (indices_hold(rewritten@, done)
                    <==> #[trigger] containers_at_the_start[done].is_changed_this_publish) by {
                    if done < position - 1 {
                        if indices_hold(rewritten@, done) {
                            let listed = choose|listed: int| 0 <= listed < rewritten@.len() && (#[trigger] rewritten@[listed]).0 == done;
                            if listed < rewritten_before.len() {
                                assert(rewritten_before[listed] == rewritten@[listed]);
                            }
                        }
                        if containers_at_the_start[done].is_changed_this_publish {
                            let listed = choose|listed: int|
                                0 <= listed < rewritten_before.len() && (#[trigger] rewritten_before[listed]).0 == done;
                            assert(rewritten@[listed] == rewritten_before[listed]);
                        }
                    } else if is_changed {
                        assert(rewritten@[rewritten@.len() - 1].0 == done);
                    } else {
                        assert(rewritten@ == rewritten_before);
                        if indices_hold(rewritten@, done) {
                            let listed = choose|listed: int| 0 <= listed < rewritten@.len() && (#[trigger] rewritten@[listed]).0 == done;
                        }
                    }
                }
            }
        }
        proof {
            assert(containers@ =~= contents_being_merged(containers_at_the_start));
            assert forall|done: int| 0 <= done < origins@.len() implies match #[trigger] origins@[done] {
                InodeLeafContainerOrigin::CarriedFrom(previous_index) => previous_index.0 < before.len()
                    && leaf_containers_are_the_same(containers@[done], before[previous_index.0 as int]),
                InodeLeafContainerOrigin::RewrittenThisPublish => true,
            } by {
                assert(containers_at_the_start[done] == containers_at_the_start[done]);
            }
        }
        InodeLeafContainersAfterThisPublish { containers, rewritten, origins, retired: tree.retired }
    }
}

/// 两片邻居并成一片（左片的记录接上右片的），记录连起来不变。
proof fn lemma_merging_two_neighbours_keeps_the_leaf_records(
    containers: Seq<InodeLeafContainer>,
    left_position: int,
    merged: InodeLeafContainer,
)
    requires
        0 <= left_position,
        left_position + 1 < containers.len(),
        merged.records@ == containers[left_position].records@ + containers[left_position + 1].records@,
    ensures
        leaf_records(containers.remove(left_position + 1).update(left_position, merged)) == leaf_records(containers),
{
    let before_the_pair = containers.take(left_position);
    let after_the_pair = containers.subrange(left_position + 2, containers.len() as int);
    let pair = seq![containers[left_position], containers[left_position + 1]];
    assert(containers =~= before_the_pair + pair + after_the_pair);
    assert(containers.remove(left_position + 1).update(left_position, merged) =~= before_the_pair + seq![merged] + after_the_pair);
    lemma_leaf_records_of_three_parts(before_the_pair, pair, after_the_pair);
    lemma_leaf_records_of_three_parts(before_the_pair, seq![merged], after_the_pair);
    lemma_leaf_records_of_two_containers(containers[left_position], containers[left_position + 1]);
    lemma_leaf_records_of_one_container(merged);
}

/// 两片邻居之间挪记录（两片的记录接起来不变），记录连起来不变。
proof fn lemma_moving_records_between_neighbours_keeps_the_leaf_records(
    containers: Seq<InodeLeafContainer>,
    left_position: int,
    left_after: InodeLeafContainer,
    right_after: InodeLeafContainer,
)
    requires
        0 <= left_position,
        left_position + 1 < containers.len(),
        left_after.records@ + right_after.records@ == containers[left_position].records@ + containers[left_position + 1].records@,
    ensures
        leaf_records(containers.update(left_position, left_after).update(left_position + 1, right_after)) == leaf_records(containers),
{
    let before_the_pair = containers.take(left_position);
    let after_the_pair = containers.subrange(left_position + 2, containers.len() as int);
    let pair = seq![containers[left_position], containers[left_position + 1]];
    let pair_after = seq![left_after, right_after];
    assert(containers =~= before_the_pair + pair + after_the_pair);
    assert(containers.update(left_position, left_after).update(left_position + 1, right_after) =~= before_the_pair + pair_after
        + after_the_pair);
    lemma_leaf_records_of_three_parts(before_the_pair, pair, after_the_pair);
    lemma_leaf_records_of_three_parts(before_the_pair, pair_after, after_the_pair);
    lemma_leaf_records_of_two_containers(containers[left_position], containers[left_position + 1]);
    lemma_leaf_records_of_two_containers(left_after, right_after);
}

/// 第 `holding_position` 片去掉第 `position_in_container` 条之后，记录连起来就是原来的去掉那一条（它之前的条数加 `position_in_container`）。
proof fn lemma_removing_a_record_keeps_the_rest_of_the_leaf_records(
    containers: Seq<InodeLeafContainer>,
    holding_position: int,
    position_in_container: int,
    holding_after: InodeLeafContainer,
)
    requires
        0 <= holding_position < containers.len(),
        0 <= position_in_container < containers[holding_position].records@.len(),
        holding_after.records@ == containers[holding_position].records@.remove(position_in_container),
    ensures
        leaf_records(containers.update(holding_position, holding_after)) == leaf_records(containers).remove(
            records_before_container(containers, holding_position) + position_in_container,
        ),
{
    lemma_leaf_records_around_one_container(containers, holding_position, holding_after);
    assert(leaf_records(containers.take(holding_position)) + holding_after.records@ + leaf_records(
        containers.subrange(holding_position + 1, containers.len() as int),
    ) =~= (leaf_records(containers.take(holding_position)) + containers[holding_position].records@ + leaf_records(
        containers.subrange(holding_position + 1, containers.len() as int),
    )).remove(records_before_container(containers, holding_position) + position_in_container));
}

/// 删空的那一片拿掉，记录连起来不变。
proof fn lemma_removing_an_empty_leaf_keeps_the_leaf_records(containers: Seq<InodeLeafContainer>, position: int)
    requires
        0 <= position < containers.len(),
        containers[position].records@.len() == 0,
    ensures
        leaf_records(containers.remove(position)) == leaf_records(containers),
{
    let earlier = containers.take(position);
    let later = containers.subrange(position + 1, containers.len() as int);
    assert(containers =~= earlier + seq![containers[position]] + later);
    assert(containers.remove(position) =~= earlier + later);
    lemma_leaf_records_of_a_concatenation(earlier + seq![containers[position]], later);
    lemma_leaf_records_of_a_concatenation(earlier, seq![containers[position]]);
    lemma_leaf_records_of_a_concatenation(earlier, later);
    lemma_leaf_records_of_one_container(containers[position]);
    assert(leaf_records(earlier) + containers[position].records@ =~= leaf_records(earlier));
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;
    use singlefs_format::TREE_IDENTIFIER_INODE;

    const INODE_TREE: TreeIdentifier = TreeIdentifier(TREE_IDENTIFIER_INODE);
    /// 新池新建文件把树建起来那次发布的 checkpoint_txg（字节表四：容器 1、出生代 3）。
    const TREE_BIRTH_TXG: CheckpointTxg = CheckpointTxg(3);

    fn live_inode_record(inode: u64, size: u64, change_count: u64) -> InodeRecord {
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

    /// 新池新建文件那一档：树是空的，第一条记录建第一片容器，容器号 = 它自己的 inode 号、出生代 = 这次发布的 txg。
    #[test]
    fn the_first_record_creates_the_first_container_numbered_after_itself() {
        let after = write_records_into_leaf_containers(
            &[],
            &[live_inode_record(1, 3000, 3)],
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("建第一片容器");
        assert_eq!(after.containers.len(), 1);
        assert_eq!(
            after.containers[0].identity,
            PackedIdentity {
                birth_tree: INODE_TREE,
                record_type: PACKED_TYPE_INODE,
                container: 1,
                container_birth: TREE_BIRTH_TXG,
            },
            "容器号 = 第一条记录的 inode 号（D8 已定项 6）"
        );
        assert_eq!(
            after.containers[0].records,
            vec![live_inode_record(1, 3000, 3)]
        );
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::LEFTMOST]
        );
    }

    /// 覆盖写那一档：号已经在树里 ⇒ 原地换掉那一条，记录数不变、身份不变，只有那一片进重写清单。
    #[test]
    fn writing_a_number_already_in_the_tree_replaces_that_record_and_keeps_the_container_identity()
    {
        let before = write_records_into_leaf_containers(
            &[],
            &[live_inode_record(1, 3000, 3)],
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("建第一片容器")
        .containers;
        let after = write_records_into_leaf_containers(
            &before,
            &[live_inode_record(1, 4096, 4)],
            CheckpointTxg(4),
            INODE_TREE,
        )
        .expect("覆盖写");
        assert_eq!(after.containers.len(), 1);
        assert_eq!(
            after.containers[0].identity, before[0].identity,
            "COW 重写身份不变（D8 已定项 6）：出生代仍是建树那次的 3，不是这次发布的 4"
        );
        assert_eq!(
            after.containers[0].records,
            vec![live_inode_record(1, 4096, 4)]
        );
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::LEFTMOST]
        );
    }

    /// 一个码 3 单元按 inode 记录宽装得下的条数不多于一容器的上限：`free_record_slots` 那道断言对盘上读来的容器从盘上走不到，
    /// 靠的就是这条算术（读者判记录宽 = `INODE_RECORD_BYTES`、记录区不越过单元末尾）。改了记录宽、单元头宽或预留位而不重算
    /// `INODE_LEAF_RECORDS`，盘上一片就可能装得下第 234 条（C476 普查「缺口」表 `inode_tree.rs:88` 那一行）。
    #[test]
    fn a_packed_unit_holds_no_more_inode_records_than_the_leaf_record_limit() {
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

    /// 一容器 233 条：第 233 条还在第一片里，树只有一片叶。
    #[test]
    fn two_hundred_thirty_three_records_still_fit_one_container() {
        let records: Vec<InodeRecord> = (1..=233u64)
            .map(|inode| live_inode_record(inode, 0, 3))
            .collect();
        let after = write_records_into_leaf_containers(&[], &records, TREE_BIRTH_TXG, INODE_TREE)
            .expect("233 条");
        assert_eq!(INODE_LEAF_RECORDS, 233);
        assert_eq!(after.containers.len(), 1, "233 条装得下一片");
        assert_eq!(after.containers[0].records.len(), 233);
        assert_eq!(after.containers[0].free_record_slots(), 0);
    }

    /// 第 234 条触发末尾分裂：左半保留全部原有记录与身份、**一个字节都不重写**；
    /// 右半是新容器，容器号 = 触发分裂那条新记录的 inode 号、出生代 = 本次发布的 checkpoint_txg、出生树 = 执行分裂的那棵树。
    /// 这是 C116（叶容器的分裂 / 合并纪律没有会失败的检查） 那三条纪律的会红的量。
    #[test]
    fn the_two_hundred_thirty_fourth_record_splits_at_the_end_and_the_left_half_keeps_everything() {
        let before = write_records_into_leaf_containers(
            &[],
            &(1..=233u64)
                .map(|inode| live_inode_record(inode, 0, 3))
                .collect::<Vec<_>>(),
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("233 条")
        .containers;
        let split_txg = CheckpointTxg(9);
        let after = write_records_into_leaf_containers(
            &before,
            &[live_inode_record(234, 0, 9)],
            split_txg,
            INODE_TREE,
        )
        .expect("第 234 条");

        assert_eq!(after.containers.len(), 2, "分裂出第二片容器");
        assert_eq!(
            after.containers[0], before[0],
            "左半保留全部原有记录与身份：记录、容器号、出生代、出生树逐项不变"
        );
        assert_eq!(
            after.containers[0].records.len(),
            233,
            "分裂点取末尾：左半一条记录都不搬走"
        );
        assert_eq!(
            after.containers[1].identity,
            PackedIdentity {
                birth_tree: INODE_TREE,
                record_type: PACKED_TYPE_INODE,
                container: 234,
                container_birth: split_txg,
            },
            "右半容器号 = 触发分裂那条新记录的 inode 号，出生代 = 本次发布的 checkpoint_txg"
        );
        assert_eq!(
            after.containers[1].records,
            vec![live_inode_record(234, 0, 9)]
        );
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::of_position(1)],
            "只重写右半：左半的字节没变，连 COW 都不做"
        );
    }

    /// 分裂之后两片容器的分隔 key 与 key 区间：I-9.4（容器号不超最小 key） 与 I-9.12（分隔 key 落在孩子区间之外） 在树这一侧的形态。
    #[test]
    fn after_the_split_the_separator_keys_are_the_container_numbers_and_the_ranges_do_not_overlap()
    {
        let after = write_records_into_leaf_containers(
            &[],
            &(1..=234u64)
                .map(|inode| live_inode_record(inode, 0, 3))
                .collect::<Vec<_>>(),
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("234 条");
        let separators: Vec<u64> = after
            .containers
            .iter()
            .map(InodeLeafContainer::separator_key)
            .collect();
        assert_eq!(separators, vec![1, 234]);
        assert_eq!(
            after.containers[0].smallest_inode_number(),
            Some(InodeNumber(1))
        );
        assert_eq!(
            after.containers[0].largest_inode_number(),
            Some(InodeNumber(233))
        );
        assert_eq!(
            after.containers[1].smallest_inode_number(),
            Some(InodeNumber(234))
        );
        assert!(
            after.containers[0].largest_inode_number()
                < Some(after.containers[1].container_number()),
            "左容器的最大 key < 右容器号（I-9.4 那一句逼出的合并方向）"
        );
        for (container, separator) in after.containers.iter().zip(&separators) {
            assert!(
                container.smallest_inode_number() >= Some(InodeNumber(*separator)),
                "容器号不超它装的最小 key"
            );
        }
    }

    /// 一次写多条：新号接着最右叶追加，跨过 233 时中间那一片满着不动。
    #[test]
    fn appending_across_the_two_hundred_thirty_three_threshold_fills_left_to_right() {
        let after = write_records_into_leaf_containers(
            &[],
            &(1..=500u64)
                .map(|inode| live_inode_record(inode, 0, 3))
                .collect::<Vec<_>>(),
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("500 条");
        assert_eq!(after.containers.len(), 3, "500 = 233 + 233 + 34");
        assert_eq!(
            after
                .containers
                .iter()
                .map(|container| container.records.len())
                .collect::<Vec<_>>(),
            vec![233, 233, 34]
        );
        assert_eq!(
            after
                .containers
                .iter()
                .map(|container| container.identity.container)
                .collect::<Vec<_>>(),
            vec![1, 234, 467],
            "每片容器号 = 它诞生时装的第一条记录的 inode 号"
        );
        assert_eq!(
            after.rewritten,
            vec![
                InodeLeafContainerIndexInTree::of_position(0),
                InodeLeafContainerIndexInTree::of_position(1),
                InodeLeafContainerIndexInTree::of_position(2),
            ],
            "这一次三片都动过：第一片从空装到满，后两片是新建的"
        );
    }

    /// 中间插入（号不在树里、又不比树里每一个 key 大）在任何人动盘之前被拒：非末尾分裂的纪律没写。
    #[test]
    fn inserting_a_number_that_would_not_land_in_the_rightmost_leaf_is_refused() {
        let before = write_records_into_leaf_containers(
            &[],
            &[live_inode_record(1, 0, 3), live_inode_record(5, 0, 3)],
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("两条")
        .containers;
        let refusal = write_records_into_leaf_containers(
            &before,
            &[live_inode_record(3, 0, 4)],
            CheckpointTxg(4),
            INODE_TREE,
        )
        .expect_err("中间插入今天一律被拒");
        assert_eq!(
            refusal,
            InodeTreeWriteRefusal::InsertWouldNotLandInTheRightmostLeaf {
                inode: InodeNumber(3),
                largest_inode_number_in_the_tree: InodeNumber(5),
            }
        );
    }

    /// 容器数超过一个码 2 根装得下的 135 片：树要长高，怎么长没条款 ⇒ 拒。
    #[test]
    fn more_containers_than_one_root_node_holds_is_refused() {
        assert_eq!(leaf_containers_one_root_node_holds(), 135, "扇出 135");
        let records: Vec<InodeRecord> = (1..=(233 * 135 + 1))
            .map(|inode| live_inode_record(inode, 0, 3))
            .collect();
        let refusal = write_records_into_leaf_containers(&[], &records, TREE_BIRTH_TXG, INODE_TREE)
            .expect_err("第 136 片容器今天装不下");
        assert_eq!(
            refusal,
            InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds {
                containers: 136,
                capacity: 135,
            }
        );
    }

    /// 同一次写里的记录不按 inode 号严格升序 ⇒ 拒（升序取号是 D16 已定项 5 末段的纪律，这里不替调用方排）。
    #[test]
    fn records_not_in_ascending_inode_order_are_refused() {
        let refusal = write_records_into_leaf_containers(
            &[],
            &[live_inode_record(3, 0, 3), live_inode_record(2, 0, 3)],
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect_err("降序被拒");
        assert_eq!(
            refusal,
            InodeTreeWriteRefusal::RecordsNotInStrictlyAscendingInodeOrder {
                earlier: InodeNumber(3),
                later: InodeNumber(2),
            }
        );
        assert_eq!(
            write_records_into_leaf_containers(
                &[],
                &[live_inode_record(3, 0, 3), live_inode_record(3, 1, 3)],
                TREE_BIRTH_TXG,
                INODE_TREE,
            )
            .expect_err("同号两条也被拒"),
            InodeTreeWriteRefusal::RecordsNotInStrictlyAscendingInodeOrder {
                earlier: InodeNumber(3),
                later: InodeNumber(3),
            }
        );
    }

    /// 一次写里既改旧记录又追加新记录：改的那一片与新建的那一片都进重写清单，各进一次。
    #[test]
    fn one_write_that_both_replaces_and_appends_lists_each_touched_container_once() {
        let before = write_records_into_leaf_containers(
            &[],
            &(1..=233u64)
                .map(|inode| live_inode_record(inode, 0, 3))
                .collect::<Vec<_>>(),
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("233 条")
        .containers;
        let after = write_records_into_leaf_containers(
            &before,
            &[
                live_inode_record(1, 4096, 9),
                live_inode_record(234, 0, 9),
                live_inode_record(235, 0, 9),
            ],
            CheckpointTxg(9),
            INODE_TREE,
        )
        .expect("改一条、追加两条");
        assert_eq!(
            after.rewritten,
            vec![
                InodeLeafContainerIndexInTree::of_position(0),
                InodeLeafContainerIndexInTree::of_position(1),
            ]
        );
        assert_eq!(after.containers[1].records.len(), 2, "新容器装两条");
        assert_eq!(
            after.containers[1].identity.container, 234,
            "容器号是触发分裂那条（234），不是这次写的最后一条（235）"
        );
        assert_eq!(
            after.containers[0].records[0],
            live_inode_record(1, 4096, 9)
        );
    }

    /// 三片叶容器：第一片 1..=233、第二片 234..=466、第三片只有 467（分裂纪律写出来的形状）。
    fn three_containers() -> Vec<InodeLeafContainer> {
        let after = write_records_into_leaf_containers(
            &[],
            &(1..=467u64)
                .map(|inode| live_inode_record(inode, 0, 3))
                .collect::<Vec<_>>(),
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("467 条");
        assert_eq!(after.containers.len(), 3, "233 + 233 + 1");
        after.containers
    }

    /// `full_containers` 片装满 233 条的叶容器（inode 1 起连号；容器号 = 每片第一条的号、分隔 key 同容器号）。
    fn full_containers(full_containers: u64) -> Vec<InodeLeafContainer> {
        let after = write_records_into_leaf_containers(
            &[],
            &(1..=233 * full_containers)
                .map(|inode| live_inode_record(inode, 0, 3))
                .collect::<Vec<_>>(),
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("满的几片");
        assert_eq!(
            after.containers.len(),
            usize::try_from(full_containers).expect("片数")
        );
        after.containers
    }

    /// 把第 `position` 片的记录留成按号升序的前 `kept` 条（直接在内容上改：这里判的是删一条之后的落法，不是怎么删到这个样子）。
    fn keep_the_first_records(containers: &mut [InodeLeafContainer], position: usize, kept: usize) {
        containers[position].records.truncate(kept);
    }

    fn container_numbers(containers: &[InodeLeafContainer]) -> Vec<u64> {
        containers
            .iter()
            .map(|container| container.identity.container)
            .collect()
    }

    fn record_counts(containers: &[InodeLeafContainer]) -> Vec<usize> {
        containers
            .iter()
            .map(|container| container.records.len())
            .collect()
    }

    /// 删一条、那一片还有记录、与左右邻都并不了、不少于 117 条：只重写那一片，身份不变，别的片照抄，没有身份消失。
    #[test]
    fn removing_a_record_that_triggers_no_merge_rewrites_only_that_container() {
        let before = full_containers(3);
        let after = remove_record_from_leaf_containers(&before, InodeNumber(300)).expect("删 300");
        assert_eq!(record_counts(&after.containers), vec![233, 232, 233]);
        assert_eq!(
            after.containers[1].identity, before[1].identity,
            "COW 重写身份不变"
        );
        assert!(
            after.containers[1]
                .records
                .iter()
                .all(|kept| kept.inode != 300),
            "那一条不在了"
        );
        assert_eq!(after.containers[0], before[0]);
        assert_eq!(after.containers[2], before[2]);
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::of_position(1)]
        );
        assert_eq!(
            after.origins,
            vec![
                InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::of_position(
                    0
                )),
                InodeLeafContainerOrigin::RewrittenThisPublish,
                InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::of_position(
                    2
                )),
            ]
        );
        assert!(after.retired.is_empty());
        assert!(after.is_the_root_rewritten());
    }

    /// 删空最右那一片：整片不落盘、身份消失，重写清单是空的，根照样重写（少了一条条目）。它左边没有右邻可并。
    #[test]
    fn removing_the_only_record_of_the_rightmost_container_drops_it_and_still_rewrites_the_root() {
        let before = three_containers();
        let after = remove_record_from_leaf_containers(&before, InodeNumber(467)).expect("删 467");
        assert_eq!(after.containers, before[..2].to_vec(), "前两片原样");
        assert!(after.rewritten.is_empty(), "没有一片要重写");
        assert_eq!(
            after.retired,
            vec![RetiredInodeLeafContainer {
                previous_index: InodeLeafContainerIndexInTree::of_position(2),
                identity: before[2].identity,
            }]
        );
        assert!(after.is_the_root_rewritten(), "根少一条条目，要重写");
    }

    /// 删空最左一片（M1 写出来的树才有这种形状：左边一片只剩一条）：整片不落盘、身份消失；它没有左邻，不并；
    /// 右边每一片左移一格、内容没变，照抄、不进重写清单（`origins` 记着上一版的叶序）——20 片与 40 片两种几何重写清单都是空的，
    /// 不随右边片数涨。
    #[test]
    fn dropping_the_emptied_leftmost_container_carries_every_container_to_its_right_whatever_their_number(
    ) {
        for containers_in_the_tree in [20_u64, 40] {
            let mut before = full_containers(containers_in_the_tree);
            keep_the_first_records(&mut before, 0, 1);
            let after = remove_record_from_leaf_containers(&before, InodeNumber(1)).expect("删 1");
            assert_eq!(
                after.containers,
                before[1..].to_vec(),
                "{containers_in_the_tree} 片：右边各片内容、身份、分隔 key 原样"
            );
            assert!(
                after.rewritten.is_empty(),
                "{containers_in_the_tree} 片：挪了位置的一片都不重写"
            );
            assert_eq!(
                after.origins,
                (1..before.len())
                    .map(|previous_position| InodeLeafContainerOrigin::CarriedFrom(
                        InodeLeafContainerIndexInTree::of_position(previous_position)
                    ))
                    .collect::<Vec<_>>(),
                "{containers_in_the_tree} 片：这一版第 i 片照抄上一版第 i + 1 片"
            );
            assert_eq!(after.retired_identities(), vec![before[0].identity]);
        }
    }

    /// 删空中间一片（M1 写出来的形状）：身份消失；左邻与新右邻之和 > 233 不并；新右邻左移一格照抄，左边那片不动。
    #[test]
    fn dropping_an_emptied_middle_container_carries_the_ones_to_its_right_without_rewriting_them() {
        let mut before = three_containers();
        keep_the_first_records(&mut before, 1, 1);
        let after = remove_record_from_leaf_containers(&before, InodeNumber(234)).expect("删 234");
        assert_eq!(after.containers, vec![before[0].clone(), before[2].clone()]);
        assert!(
            after.rewritten.is_empty(),
            "左邻 233 + 新右邻 1 = 234 > 233，不并"
        );
        assert_eq!(
            after.origins[1],
            InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::of_position(2))
        );
        assert_eq!(after.retired_identities(), vec![before[1].identity]);
    }

    /// 左吸收右（M3）：第一片删完一条剩 99 条、第二片 100 条，之和 199 ≤ 233 ⇒ 第一片吸收第二片：保留左半身份与分隔 key，记录接在后面；
    /// 第二片身份消失；第三片左移一格照抄。前后叶序：容器号 [1, 234, 467] → [1, 467]。
    #[test]
    fn the_left_container_absorbs_its_right_neighbour_when_the_two_fit_in_one_and_keeps_its_identity(
    ) {
        let mut before = full_containers(3);
        keep_the_first_records(&mut before, 0, 100);
        keep_the_first_records(&mut before, 1, 100);
        let after = remove_record_from_leaf_containers(&before, InodeNumber(5)).expect("删 5");
        assert_eq!(container_numbers(&after.containers), vec![1, 467]);
        assert_eq!(record_counts(&after.containers), vec![199, 233]);
        assert_eq!(
            after.containers[0].identity, before[0].identity,
            "保留左半身份"
        );
        assert_eq!(after.containers[0].separator_key(), 1, "左半分隔 key 不变");
        assert_eq!(
            after.containers[0]
                .records
                .last()
                .map(|record| record.inode),
            Some(333),
            "右半的记录接在后面"
        );
        assert_eq!(after.containers[1], before[2], "第三片原样");
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::LEFTMOST]
        );
        assert_eq!(
            after.origins[1],
            InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::of_position(2))
        );
        assert_eq!(
            after.retired,
            vec![RetiredInodeLeafContainer {
                previous_index: InodeLeafContainerIndexInTree::of_position(1),
                identity: before[1].identity,
            }],
            "被吸收的右半身份消失"
        );
    }

    /// 左从右借（M3 均分借）：第一片删完一条剩 99 条、第二片 233 条，之和 > 233 不并；99 < 117 ⇒ 从第二片借最小的 ⌊(233 − 99) ÷ 2⌋ = 67 条，
    /// 借完两片 166 / 166（相差不超过 1）；两片都保留身份，第二片的分隔 key 改成它这一版的最小 key 301（N1）；没有身份消失，第三片照抄。
    #[test]
    fn a_container_below_the_floor_borrows_half_the_difference_from_its_right_neighbour() {
        let mut before = full_containers(3);
        keep_the_first_records(&mut before, 0, 100);
        let after = remove_record_from_leaf_containers(&before, InodeNumber(5)).expect("删 5");
        assert_eq!(container_numbers(&after.containers), vec![1, 234, 467]);
        assert_eq!(record_counts(&after.containers), vec![166, 166, 233]);
        assert_eq!(after.containers[0].identity, before[0].identity);
        assert_eq!(
            after.containers[1].identity, before[1].identity,
            "右片保留身份"
        );
        assert_eq!(
            after
                .containers
                .iter()
                .map(InodeLeafContainer::separator_key)
                .collect::<Vec<_>>(),
            vec![1, 301, 467],
            "右片的分隔 key = 它这一版的最小 key（N1），别的不变"
        );
        assert_eq!(
            after.containers[0]
                .records
                .last()
                .map(|record| record.inode),
            Some(300),
            "借来的是右片最小的 67 条：234..=300"
        );
        assert_eq!(after.containers[2], before[2]);
        assert_eq!(
            after.rewritten,
            vec![
                InodeLeafContainerIndexInTree::of_position(0),
                InodeLeafContainerIndexInTree::of_position(1)
            ]
        );
        assert!(after.retired.is_empty(), "借不退役");
    }

    /// 借了之后再删到 116、右片又借、……：同一对一路删 233 条，最后一条让两片之和恰好 233，左吸收右。
    /// 均分借每次都让两片条数相差不超过 1；中间每一版相邻两片之和都 ≥ 234（M3 那一臂的丙类断言）。
    #[test]
    fn deleting_from_the_left_of_a_pair_borrows_until_the_pair_fits_in_one_and_then_absorbs() {
        let mut containers = full_containers(3);
        let mut borrows = 0;
        let mut deletions = 0;
        let mut absorbed = false;
        // 迭代上界：这一对一共 466 条，每一轮删一条；吸收那一轮退出。
        while !absorbed {
            let inode = containers[0].records[1].inode;
            let after = remove_record_from_leaf_containers(&containers, InodeNumber(inode))
                .expect("删左片第二条");
            deletions += 1;
            absorbed = !after.retired.is_empty();
            if after.rewritten.len() == 2 {
                borrows += 1;
                let counts = record_counts(&after.containers);
                assert!(counts[0].abs_diff(counts[1]) <= 1, "均分借：{counts:?}");
            }
            if !absorbed {
                for pair in after.containers.windows(2) {
                    assert!(
                        pair[0].records.len() + pair[1].records.len() >= 234,
                        "相邻两片之和 ≥ 234"
                    );
                }
            }
            containers = after.containers;
        }
        assert_eq!(deletions, 233, "两片之和 466 删到 233 才装得下一片");
        assert_eq!(
            borrows, 7,
            "116 → 174 / 145 / 131 / 123 / 120 / 118 / 117 七次借"
        );
        assert_eq!(container_numbers(&containers), vec![1, 467]);
        assert_eq!(record_counts(&containers), vec![233, 233]);
    }

    /// 三片各 117 条、删中间那片的一条：
    /// 中间那片剩 116 条，与左邻、右邻之和都恰好 233，两对都装得下。M3「先查右邻」⇒ 中间那片吸收右邻，退役的是右邻（容器号 467），
    /// 左邻原样照抄；先查左邻的写法让左邻吸收中间那片、退役的是中间那片（容器号 234）。两种写法的镜像池级 checker 都不红，方向只有这里钉。
    #[test]
    fn removing_from_the_middle_of_three_half_full_containers_absorbs_the_right_neighbour_first() {
        let mut before = full_containers(3);
        for position in 0..3 {
            keep_the_first_records(&mut before, position, 117);
        }
        let after = remove_record_from_leaf_containers(&before, InodeNumber(300)).expect("删 300");
        assert_eq!(
            container_numbers(&after.containers),
            vec![1, 234],
            "中间那片吸收右邻：留下左邻与中间那片"
        );
        assert_eq!(record_counts(&after.containers), vec![117, 233]);
        assert_eq!(after.containers[0], before[0], "左邻一个字节都不动");
        assert_eq!(
            after.containers[1].identity, before[1].identity,
            "吸收方保留中间那片的身份"
        );
        assert_eq!(
            after.retired,
            vec![RetiredInodeLeafContainer {
                previous_index: InodeLeafContainerIndexInTree::of_position(2),
                identity: before[2].identity,
            }],
            "先查右邻：退役的是右邻，不是中间那片"
        );
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::of_position(1)]
        );
        assert_eq!(
            after.origins[0],
            InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::LEFTMOST)
        );
    }

    /// 删的是树里唯一一条（unlink 唯一的文件）：一片都不剩、没有要重写的片，那片身份消失，根照样重写成零条目根。
    #[test]
    fn removing_the_last_record_of_the_tree_leaves_no_container_and_rewrites_the_root() {
        let before = write_records_into_leaf_containers(
            &[],
            &[live_inode_record(1, 4096, 3)],
            TREE_BIRTH_TXG,
            INODE_TREE,
        )
        .expect("一条")
        .containers;
        let after = remove_record_from_leaf_containers(&before, InodeNumber(1)).expect("删 1");
        assert!(after.containers.is_empty(), "一片都不剩");
        assert!(after.rewritten.is_empty());
        assert_eq!(after.retired_identities(), vec![before[0].identity]);
        assert!(after.is_the_root_rewritten(), "根重写成零条目根");
    }

    /// 要删的号不在树里：在任何落盘动作之前拒。
    #[test]
    fn removing_a_number_that_is_not_in_the_tree_is_refused() {
        let before = three_containers();
        assert_eq!(
            remove_record_from_leaf_containers(&before, InodeNumber(468)),
            Err(InodeTreeWriteRefusal::RecordToRemoveIsNotInTheTree {
                inode: InodeNumber(468)
            })
        );
    }

    /// 等价性留档要比的两种改写（`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」第二行：在所有输入上与原式同值的变异，
    /// 把等价性写成测试留档）。：两条腿都推等价，
    /// 前提是「相邻两片之和 ≥ 234」——E176 M2、M3 的断言；追加只在最右那片满了才新开一片，删一条之后能并的一对之和恰好 233、
    /// 并完那片满 233 条，删空的那一片删之前只有 1 条、它的两个邻居都满着。
    #[derive(Clone, Copy, Debug)]
    enum MergeRuleRewriting {
        /// `k4-no-chain`：从某片起的每一趟并掉一对就停，不从幸存那片接着查（借完那一趟也是）。
        EveryPassStopsAfterAbsorbingOnePair,
        /// m4：删空的那一片摘掉之后不从它的左邻起并。
        NothingIsAbsorbedAfterDroppingTheEmptiedContainer,
    }

    /// 改写之下从 `start_position` 那片起的一趟并：交回并没并过。
    fn absorb_pass_under_the_rewriting(
        tree: &mut LeafContainersBeingMerged,
        start_position: usize,
        rewriting: MergeRuleRewriting,
    ) -> bool {
        match rewriting {
            MergeRuleRewriting::EveryPassStopsAfterAbsorbingOnePair => {
                if start_position + 1 < tree.containers.len()
                    && tree.pair_fits_in_one_container(start_position)
                {
                    tree.absorb_the_right_neighbour(start_position);
                    true
                } else if start_position > 0 && tree.pair_fits_in_one_container(start_position - 1)
                {
                    tree.absorb_the_right_neighbour(start_position - 1);
                    true
                } else {
                    false
                }
            }
            MergeRuleRewriting::NothingIsAbsorbedAfterDroppingTheEmptiedContainer => {
                tree.absorb_until_no_pair_fits(start_position)
            }
        }
    }

    /// 改写之下的左从右借：借的条数、挪哪几条、右片的分隔 key 与实现相同，借完那一趟并照改写走。
    fn borrow_under_the_rewriting(
        tree: &mut LeafContainersBeingMerged,
        borrower_position: usize,
        rewriting: MergeRuleRewriting,
    ) {
        match rewriting {
            MergeRuleRewriting::EveryPassStopsAfterAbsorbingOnePair => {
                let lender_position = borrower_position + 1;
                if lender_position >= tree.containers.len()
                    || tree.records_at(borrower_position)
                        >= records_below_which_a_leaf_borrows_from_its_right_neighbour()
                {
                    return;
                }
                let moved_count =
                    (tree.records_at(lender_position) - tree.records_at(borrower_position)) / 2;
                let moved_records: Vec<InodeRecord> = tree.containers[lender_position]
                    .contents
                    .records
                    .drain(..moved_count)
                    .collect();
                let smallest_kept_by_the_lender =
                    tree.containers[lender_position].contents.records[0].inode;
                tree.containers[lender_position]
                    .contents
                    .separator_key_in_the_parent_entry = smallest_kept_by_the_lender;
                tree.containers[lender_position].is_changed_this_publish = true;
                tree.containers[borrower_position]
                    .contents
                    .records
                    .extend(moved_records);
                tree.containers[borrower_position].is_changed_this_publish = true;
                absorb_pass_under_the_rewriting(tree, lender_position, rewriting);
            }
            MergeRuleRewriting::NothingIsAbsorbedAfterDroppingTheEmptiedContainer => {
                tree.borrow_from_the_right_neighbour_if_below_the_floor(borrower_position);
            }
        }
    }

    /// 改写之下删一条：删法、删空摘片、先并后借的次序与 [`remove_record_from_leaf_containers`] 相同，只有改写点到的那一处换掉。
    fn remove_record_under_the_rewriting(
        containers_before_this_publish: &[InodeLeafContainer],
        inode: InodeNumber,
        rewriting: MergeRuleRewriting,
    ) -> InodeLeafContainersAfterThisPublish {
        let holding_position =
            position_of_the_container_holding(containers_before_this_publish, inode)
                .expect("族里每一次删的号都取自那棵树")
                .position();
        let mut tree =
            LeafContainersBeingMerged::from_the_previous_version(containers_before_this_publish);
        tree.containers[holding_position]
            .contents
            .records
            .retain(|kept| kept.inode != inode.0);
        tree.containers[holding_position].is_changed_this_publish = true;
        if tree.records_at(holding_position) == 0 {
            tree.retire(holding_position);
            let has_new_neighbours_on_both_sides =
                holding_position > 0 && holding_position < tree.containers.len();
            match rewriting {
                MergeRuleRewriting::EveryPassStopsAfterAbsorbingOnePair => {
                    if has_new_neighbours_on_both_sides {
                        absorb_pass_under_the_rewriting(&mut tree, holding_position - 1, rewriting);
                    }
                }
                MergeRuleRewriting::NothingIsAbsorbedAfterDroppingTheEmptiedContainer => {}
            }
        } else if !absorb_pass_under_the_rewriting(&mut tree, holding_position, rewriting) {
            borrow_under_the_rewriting(&mut tree, holding_position, rewriting);
        }
        tree.after_this_publish()
    }

    /// 等价性那一族每片记录数的取值：1（删空）、2（借后吸收里的第三片）、116 / 117 / 118（删一条可并、借、借 1 条）、233（满片）。
    const RECORD_COUNTS_OF_THE_EQUIVALENCE_FAMILY: [usize; 6] = [1, 2, 116, 117, 118, 233];
    /// 那一族的树至多这么多片。
    const CONTAINERS_IN_THE_EQUIVALENCE_FAMILY_AT_MOST: usize = 4;

    /// 1 到 4 片、每片记录数取自 [`RECORD_COUNTS_OF_THE_EQUIVALENCE_FAMILY`]、相邻两片之和都 ≥ 234 的每一组记录数。
    fn record_counts_where_every_adjacent_pair_overflows_one_container() -> Vec<Vec<usize>> {
        let capacity = usize::try_from(INODE_LEAF_RECORDS).expect("233");
        let mut of_the_current_length: Vec<Vec<usize>> = vec![Vec::new()];
        let mut family = Vec::new();
        // 迭代上界：CONTAINERS_IN_THE_EQUIVALENCE_FAMILY_AT_MOST 轮，每轮多一片。
        for _ in 0..CONTAINERS_IN_THE_EQUIVALENCE_FAMILY_AT_MOST {
            of_the_current_length = of_the_current_length
                .iter()
                .flat_map(|prefix| {
                    RECORD_COUNTS_OF_THE_EQUIVALENCE_FAMILY
                        .iter()
                        .filter(|count| prefix.last().is_none_or(|last| last + **count > capacity))
                        .map(|count| {
                            let mut longer = prefix.clone();
                            longer.push(*count);
                            longer
                        })
                })
                .collect();
            family.extend(of_the_current_length.iter().cloned());
        }
        family
    }

    /// 记录数照 `record_counts` 的一棵树：inode 号从 1 起连号，每片容器号与分隔 key = 它的第一条（分裂纪律写出来的形状）。
    fn tree_of_record_counts(record_counts: &[usize]) -> Vec<InodeLeafContainer> {
        let mut first_inode_of_the_next_container = 1_u64;
        record_counts
            .iter()
            .map(|count| {
                let first_inode = first_inode_of_the_next_container;
                first_inode_of_the_next_container += u64::try_from(*count).expect("至多 233");
                InodeLeafContainer {
                    identity: PackedIdentity {
                        birth_tree: INODE_TREE,
                        record_type: PACKED_TYPE_INODE,
                        container: first_inode,
                        container_birth: TREE_BIRTH_TXG,
                    },
                    separator_key_in_the_parent_entry: first_inode,
                    records: (first_inode..first_inode_of_the_next_container)
                        .map(|inode| live_inode_record(inode, 0, 3))
                        .collect(),
                }
            })
            .collect()
    }

    /// 每片删第一条与最后一条（只有一条的那片删一次）：删哪一片决定落法，删片里哪一条只改挪走哪几个号。
    fn inodes_removed_from_each_container(containers: &[InodeLeafContainer]) -> Vec<InodeNumber> {
        let mut removed = Vec::new();
        for container in containers {
            let smallest = container.smallest_inode_number().expect("族里每片至少一条");
            let largest = container.largest_inode_number().expect("族里每片至少一条");
            removed.push(smallest);
            if largest != smallest {
                removed.push(largest);
            }
        }
        removed
    }

    /// 这次删除里有片的分隔 key 变了（只有借会改：右片改成它这一版的最小 key）。
    fn some_separator_key_moved(
        before: &[InodeLeafContainer],
        after: &InodeLeafContainersAfterThisPublish,
    ) -> bool {
        after.containers.iter().any(|kept| {
            before.iter().any(|earlier| {
                earlier.identity == kept.identity
                    && earlier.separator_key_in_the_parent_entry
                        != kept.separator_key_in_the_parent_entry
            })
        })
    }

    /// `k4-no-chain`（连锁只并一轮）的等价性留档：相邻两片之和都 ≥ 234 的每一棵树上删每一片的头尾一条，
    /// 实现与「每一趟并掉一对就停」逐片相同（容器、重写清单、来处、退役，整份比）——能并的一对之和恰好 233、并完那片满着，
    /// 再与哪个邻居相加都 > 233；借之后借方与借出片之和不变、仍 > 233，借出片吸收右邻之后同理。
    /// 族里删一条并、借后吸收、删空摘片后从左邻起那一趟各至少走到一次；删完之后相邻两片之和仍 ≥ 234（前提对删除封闭）。
    #[test]
    fn stopping_every_absorb_pass_after_one_pair_changes_nothing_when_every_adjacent_pair_overflows_one_container(
    ) {
        let rewriting = MergeRuleRewriting::EveryPassStopsAfterAbsorbingOnePair;
        let mut removals_that_absorbed_without_borrowing = 0;
        let mut removals_that_borrowed_and_then_absorbed = 0;
        let mut removals_that_dropped_a_container_between_two_neighbours = 0;
        for record_counts in record_counts_where_every_adjacent_pair_overflows_one_container() {
            let before = tree_of_record_counts(&record_counts);
            for inode in inodes_removed_from_each_container(&before) {
                let holding_position = position_of_the_container_holding(&before, inode)
                    .expect("取自这棵树")
                    .position();
                let implementation = remove_record_from_leaf_containers(&before, inode)
                    .expect("族里每一次删的号都在树里");
                assert_eq!(
                    implementation,
                    remove_record_under_the_rewriting(&before, inode, rewriting),
                    "各片 {record_counts:?} 删 {inode:?}：实现与「每一趟并掉一对就停」不同"
                );
                for pair in implementation.containers.windows(2) {
                    assert!(
                        pair[0].records.len() + pair[1].records.len() >= 234,
                        "各片 {record_counts:?} 删 {inode:?}：删完之后相邻两片之和 < 234（前提不再成立）"
                    );
                }
                let emptied = record_counts[holding_position] == 1;
                let borrowed = some_separator_key_moved(&before, &implementation);
                if emptied && holding_position > 0 && holding_position + 1 < record_counts.len() {
                    removals_that_dropped_a_container_between_two_neighbours += 1;
                }
                if !emptied && !implementation.retired.is_empty() {
                    if borrowed {
                        removals_that_borrowed_and_then_absorbed += 1;
                    } else {
                        removals_that_absorbed_without_borrowing += 1;
                    }
                }
            }
        }
        assert!(
            removals_that_absorbed_without_borrowing >= 1,
            "族里有删一条就并的"
        );
        assert!(
            removals_that_borrowed_and_then_absorbed >= 1,
            "族里有借完借出的那一片接着并的"
        );
        assert!(
            removals_that_dropped_a_container_between_two_neighbours >= 1,
            "族里有删空摘片之后两边还有邻居、从左邻起看一趟的"
        );
    }

    /// m4（删空摘片后不再并）的等价性留档：相邻两片之和都 ≥ 234 的每一棵树上删每一片的头尾一条，实现与「删空的那一片摘掉之后
    /// 不从它的左邻起并」逐片相同（整份比）——删空的那一片删之前只有 1 条，它的两个邻居都满 233 条，摘掉之后那一对之和 466，
    /// 左邻再与它的左邻之和也 > 233。族里删空摘片之后两边还有邻居的至少走到一次。
    #[test]
    fn not_absorbing_after_dropping_an_emptied_container_changes_nothing_when_every_adjacent_pair_overflows_one_container(
    ) {
        let rewriting = MergeRuleRewriting::NothingIsAbsorbedAfterDroppingTheEmptiedContainer;
        let mut removals_that_dropped_a_container_between_two_neighbours = 0;
        for record_counts in record_counts_where_every_adjacent_pair_overflows_one_container() {
            let before = tree_of_record_counts(&record_counts);
            for inode in inodes_removed_from_each_container(&before) {
                let holding_position = position_of_the_container_holding(&before, inode)
                    .expect("取自这棵树")
                    .position();
                assert_eq!(
                    remove_record_from_leaf_containers(&before, inode)
                        .expect("族里每一次删的号都在树里"),
                    remove_record_under_the_rewriting(&before, inode, rewriting),
                    "各片 {record_counts:?} 删 {inode:?}：实现与「删空摘片之后不再并」不同"
                );
                if record_counts[holding_position] == 1
                    && holding_position > 0
                    && holding_position + 1 < record_counts.len()
                {
                    removals_that_dropped_a_container_between_two_neighbours += 1;
                }
            }
        }
        assert!(
            removals_that_dropped_a_container_between_two_neighbours >= 1,
            "族里有删空摘片之后两边还有邻居的"
        );
    }
}
