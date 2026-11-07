//! 墓碑树、谱系树与退役树共用的那一半（D18（块里携带什么信息） 已定项 24 第 1、5 条；退役树照墓碑树，判决
//! `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条）：三棵树的根恒为码 2、条目照 inode 树内部节点的
//! 120 字节形态（分隔 key 容器号 8 + 身份引用 26 + 子指针 86），孩子是码 3 打包记录容器；容器号从 1 起、下一个号 = 树里最大号 + 1、
//! 零根时 1；容器只增不删。三棵树只差打包记录类型与一片装几条，各自的模块（`crate::tombstone_tree`、`crate::lineage_tree`、
//! `crate::retirement_tree`）把这两样填进来。
//!
//! 一次发布可以往开放容器里追加不止一条（退役树：一次删除连着吸收几片，[`append_records_to_the_open_container`]）：开放容器装满就开下一片，
//! 这次重写的码 2 节点是每片重写的容器的祖先的并（[`ContainerTreeShape::ancestors_of_the_rightmost_containers`]），只追加一条时它就是最右那一条路径。
//! 重写的容器恒是最右端连着的那几片（[`rewritten_containers_are_the_rightmost`]），所以每层重写的节点是这一层最右端的一段：发布路径上
//! 求祖先、判哪一片哪个节点重写了，都只读最右端那几段，不随容器总数涨（`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」
//! 运行时决策路径那一格）。最右端开一片按值原地改形状（[`ContainerTreeShape::after_a_container_is_opened_at_the_right_end`]），这次之前那一版
//! 只记每层节点数（[`ContainerTreeNodeCounts`]）。
//!
//! 根满时长高（D18（块里携带什么信息） 已定项 24 第 1、5 条「根满时长高，照 D8（核心索引结构） 已定项 6」）：
//! 码 2 节点装满时末尾分裂——左半保留全部原有条目、一个字节不动，右半从触发分裂的那条新条目起；根装满时长一层，新根 1 个码 2、
//! 分出的右半 1 个码 2（右半没有容器身份：父条目里类型段 0、容器号与出生代两段恒 0、出生树恒 0）。容器只在最右端出现，
//! 所以分裂只发生在最右那一条路径上，左边的节点一个都不重写。树的形状显式记着每个节点有几个孩子（[`ContainerTreeShape`]），
//! 不从容器数与扇出现算：只供测试的扇出旋钮不落盘（D18（块里携带什么信息） 已定项 24 第 3 条），压小扇出写出来的树换回产品扇出接着写，
//! 照着盘上那份形状往最右那条路径上加。
//!
//! 和 `crate::inode_tree` 一样，这里只算「这次发布之后树里是哪几片容器、各装哪些记录、哪几个节点要重写」，不发一个写、不动分配器、不碰落点。
//!
//! 这份文件的代码都在 `verus!` 里，由 Verus 直接验（里程碑四出口 5「core 直验」的容器小树那一簇，被验区域登记在
//! `.claude/gate.d/verus-pins.tsv`，必红世界在 `crates/singlefs-core/verus-must-red-worlds.tsv`）。重写保持「key 到条目」：
//! [`append_records_to_the_open_container`] 的 `ensures` 写明这次之后全部容器的记录依次连起来，就是上一版的记录依次连起来再接上这次给的记录，
//! 最右那片之前的容器一个字节不动，新开的容器号逐片加一；形状的不变量（每个节点至少一个孩子、每层孩子数之和等于下一层的节点数、
//! 最高那一层一个节点）是 [`ContainerTreeShape`] 的类型不变量。`requires` 只约束经 Verus 验过的调用方，没验的调用方违反时照旧在原来那一处 panic。

use std::ops::Range;

use singlefs_format::INODE_INTERNAL_ENTRY;
use vstd::prelude::*;

use crate::address::{CheckpointTxg, TreeIdentifier};
use crate::lineage_tree::LineageContainerCapacity;
use crate::retirement_tree::RetirementContainerCapacity;
use crate::tombstone_tree::TombstoneContainerCapacity;
use crate::unit::{index_node_entry_capacity, PackedIdentity};

verus! {

// ===== 这一簇用到的 singlefs-format 常量的规格 =====
// 写法照 `crate::verified_model`「singlefs-format 常量的规格」那一段：Verus 不认没进 verus! 的 crate 里的常量，被验区域用到的格式常量各写一条
// assume_specification，交回它在 singlefs-format 里的值，登记进信任清单 .claude/gate.d/verus-trust-list.tsv。差分测试是本文件 tests 模块的
// every_container_tree_format_constant_specification_returns_the_value_singlefs_format_defines：它读这份文件里每一条这样的声明，
// 与 singlefs-format 的真值逐个比。一个常量全 crate 只写一条：这一段只放这一簇要用、别处还没写的（INODE_INTERNAL_ENTRY 与
// `crate::unit::index_node_entry_capacity` 要的几个头宽常量在编解码那一簇的 records.rs、unit.rs 里）。
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::INODE_LEAF_RECORDS ] -> (inode_leaf_records: u64) returns 233u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::TOMBSTONE_CONTAINER_RECORDS ] -> (tombstone_container_records: u64) returns 815u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::LINEAGE_CONTAINER_RECORDS ] -> (lineage_container_records: u64) returns 1359u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::RETIREMENT_CONTAINER_RECORDS ] -> (retirement_container_records: u64) returns 959u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::DELETED_INODES_ENTRY_BYTES ] -> (deleted_inodes_entry_bytes: u64) returns 8u64;

/// 一片码 3 容器这一版装的东西：身份四段（出生树、打包记录类型、容器号、容器出生代）与它的记录，按追加的先后。
#[derive(Clone, Debug, PartialEq, Eq)]
#[verifier::allow(autoderive_clone_without_spec)]
pub struct RecordContainer<Record> {
    pub identity: PackedIdentity,
    pub records: Vec<Record>,
}

impl<Record> RecordContainer<Record> {
    /// 这片容器在父节点里的分隔 key：容器号（D18（块里携带什么信息） 已定项 24 第 1 条「分隔 key 容器号 8」）。
    #[must_use]
    pub fn separator_key(&self) -> (separator_key: u64)
        ensures
            separator_key == self.identity.container,
    {
        self.identity.container
    }
}

/// 墓碑树与谱系树码 2 节点条目的 key 宽：分隔 key = 容器号 8 字节（D18（块里携带什么信息） 已定项 24 第 1、5 条，三棵树的内部条目同一形态）。
pub const CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES: usize = 8;

/// 父条目指一个码 2 子节点时的身份引用（D8（核心索引结构） 已定项 6「类型段 0 ⇒ 子节点是码 2 索引节点（容器号、出生代两段恒 0）」、
/// 「类型段 0 条目身份引用里的出生树恒 0」）。
pub const IDENTITY_OF_A_CODE_TWO_CHILD: PackedIdentity = PackedIdentity {
    birth_tree: TreeIdentifier(0),
    record_type: 0,
    container: 0,
    container_birth: CheckpointTxg(0),
};

/// 一个码 2 节点按产品格式装得下几条容器树条目：与 inode 树内部节点同一形态（分隔 key 8 + 身份引用 26 + 子指针 86 = 120），扇出 135。
#[must_use]
pub fn containers_one_container_tree_root_holds() -> (capacity: usize)
    ensures
        capacity == 135,
{
    index_node_entry_capacity(
        CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES,
        usize::try_from(INODE_INTERNAL_ENTRY).expect("120"),
    )
}

/// 只供测试压小的码 2 节点条目数（`.claude/rules/fs-design.md` 五条硬要求第 2 条：「根满长高」这一支按产品扇出要 136 片容器才走得到）：
/// 至少 2（长高那次的新根要装得下左右两半）、至多产品扇出 135；只住内存，不落盘（D18（块里携带什么信息） 已定项 24 第 3 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContainerTreeNodeEntriesCappedForTests(usize);

impl ContainerTreeNodeEntriesCappedForTests {
    /// 类型不变量：至少 2、至多产品扇出 135（只经 [`Self::new`] 进来）。
    #[verifier::type_invariant]
    spec fn entries_lie_between_two_and_the_product_fanout(self) -> bool {
        2 <= self.0 <= 135
    }

    /// 压小之后一个节点装满几条（规格里用）。
    pub closed spec fn entries_in_the_specification(self) -> int {
        self.0 as int
    }

    /// 少于 2 条或多于产品扇出交回 `None`。
    #[allow(clippy::manual_range_contains, reason = "Verus 推不出 RangeInclusive::contains 的结果，写成两个比较")]
    #[must_use]
    pub fn new(entries: usize) -> (capped: Option<Self>)
        ensures
            capped is Some <==> 2 <= entries <= 135,
            capped matches Some(capped_entries) ==> capped_entries.entries_in_the_specification() == entries,
    {
        if 2 <= entries && entries <= containers_one_container_tree_root_holds() {
            Some(Self(entries))
        } else {
            None
        }
    }

    #[must_use]
    pub const fn entries(self) -> (entries: usize)
        ensures
            2 <= entries <= 135,
            entries == self.entries_in_the_specification(),
    {
        proof {
            use_type_invariant(&self);
        }
        self.0
    }
}

/// 容器树一个码 2 节点装满几条就分裂：产品路径按节点格式算（135），只供测试的开关把它压小。墓碑树与谱系树共用一个。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerTreeNodeEntries {
    FromTheNodeFormat,
    CappedForTests(ContainerTreeNodeEntriesCappedForTests),
}

impl ContainerTreeNodeEntries {
    /// 运行时报出走的是哪一条（`.claude/rules/fs-design.md` 五条硬要求第 4 条：分支必须可观测）。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            ContainerTreeNodeEntries::FromTheNodeFormat => {
                "container_tree_node_entries_from_the_node_format"
            }
            ContainerTreeNodeEntries::CappedForTests(_) => {
                "container_tree_node_entries_capped_for_tests"
            }
        }
    }

    /// 一个节点装满几条。
    #[must_use]
    pub fn entries_per_node(self) -> (entries: usize)
        ensures
            2 <= entries <= 135,
            self is FromTheNodeFormat ==> entries == 135,
            self matches ContainerTreeNodeEntries::CappedForTests(capped) ==> entries == capped.entries_in_the_specification(),
    {
        match self {
            ContainerTreeNodeEntries::FromTheNodeFormat => {
                containers_one_container_tree_root_holds()
            }
            ContainerTreeNodeEntries::CappedForTests(capped) => capped.entries(),
        }
    }
}

/// 写入口上墓碑树、谱系树与退役树的四个容量开关（都只供测试、只住内存、不落盘）：一片墓碑容器、一片谱系容器、一片退役容器各装满几条，
/// 一个码 2 节点装满几条。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContainerTreeCapacities {
    pub tombstone_container: TombstoneContainerCapacity,
    pub lineage_container: LineageContainerCapacity,
    pub retirement_container: RetirementContainerCapacity,
    pub node_entries: ContainerTreeNodeEntries,
}

impl ContainerTreeCapacities {
    /// 产品路径：四样都按格式算。
    pub const FROM_THE_FORMAT: ContainerTreeCapacities = ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container: RetirementContainerCapacity::FromTheContainerFormat,
        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    };
}

/// 容器树里一个码 2 节点的位置：层级（码 2 头里那一字节，指容器的节点是 1，根是最高那一层）与同层从左数第几个（从 0 数）。
/// 容器只在最右端出现、节点只在最右端新开，位置一旦定下就不再挪——上一版同一个位置上的节点照抄进来时就是它；
/// 长高那次旧根留在原位置（同一层级、同层第 0 个），只是不再是根。按 (层级, 同层序号) 排序就是 bump 次序（树内先叶后根）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContainerTreeNodePosition {
    pub level: u8,
    pub index_in_level: u32,
}


// ===== 形状的规格 =====
// 一版容器树的形状在规格里是「每一层（层级 1 在前）每个节点的孩子数」：层下标 level_index 从 0 起，对应层级 level_index + 1。

/// 一层里各节点孩子数之和。
pub open spec fn children_count_sum(children_of_each_node: Seq<usize>) -> int
    decreases children_of_each_node.len(),
{
    if children_of_each_node.len() == 0 {
        0
    } else {
        children_count_sum(children_of_each_node.drop_last()) + children_of_each_node.last()
    }
}

/// 形状里有几片容器：层级 1 的孩子数之和，零根 0。
pub open spec fn container_count(levels: Seq<Seq<usize>>) -> int {
    if levels.len() == 0 {
        0
    } else {
        children_count_sum(levels[0])
    }
}

/// 每一层至少一个节点、每个节点至少一个孩子。
pub open spec fn every_level_has_a_node_and_every_node_a_child(levels: Seq<Seq<usize>>) -> bool {
    &&& forall|level_index: int| 0 <= level_index < levels.len() ==> (#[trigger] levels[level_index]).len() >= 1
    &&& forall|level_index: int, node: int|
        0 <= level_index < levels.len() && 0 <= node < levels[level_index].len() ==> #[trigger] levels[level_index][node] >= 1
}

/// 层级 ℓ > 1 的孩子数之和等于层级 ℓ − 1 的节点数。
pub open spec fn each_upper_level_counts_the_nodes_below(levels: Seq<Seq<usize>>) -> bool {
    forall|upper_level_index: int|
        1 <= upper_level_index < levels.len() ==> children_count_sum(#[trigger] levels[upper_level_index])
            == levels[upper_level_index - 1].len()
}

/// 形状的不变量：每一层至少一个节点、每个节点至少一个孩子；层级 ℓ > 1 的孩子数之和等于层级 ℓ − 1 的节点数；最高那一层只有一个节点；
/// 容器数装得进 `usize`。
pub open spec fn container_tree_shape_is_well_formed(levels: Seq<Seq<usize>>) -> bool {
    &&& every_level_has_a_node_and_every_node_a_child(levels)
    &&& each_upper_level_counts_the_nodes_below(levels)
    &&& levels.len() >= 1 ==> levels.last().len() == 1
    &&& container_count(levels) <= usize::MAX
}

/// 形状装得进位置的两段：层数装得进码 2 头的一字节层级，容器数（因此每一层的节点数）装得进 u32 的同层序号。
pub open spec fn shape_fits_the_position_fields(levels: Seq<Seq<usize>>) -> bool {
    levels.len() <= 255 && container_count(levels) <= u32::MAX
}

/// 层下标 `level_index`、同层第 `node` 个那个位置。
pub open spec fn position_at(level_index: int, node: int) -> ContainerTreeNodePosition {
    ContainerTreeNodePosition { level: (level_index + 1) as u8, index_in_level: node as u32 }
}

/// 这个位置在这个形状里有节点。
pub open spec fn position_is_in_the_shape(levels: Seq<Seq<usize>>, position: ContainerTreeNodePosition) -> bool {
    1 <= position.level <= levels.len() && position.index_in_level < levels[position.level - 1].len()
}

/// 层下标 `level_index` 上有第 `node` 个节点。
pub open spec fn node_is_in_the_levels(levels: Seq<Seq<usize>>, level_index: int, node: int) -> bool {
    0 <= level_index < levels.len() && 0 <= node < levels[level_index].len()
}

/// 一层全部节点的位置，从左到右。
pub open spec fn positions_of_level(levels: Seq<Seq<usize>>, level_index: int) -> Seq<ContainerTreeNodePosition> {
    Seq::new(levels[level_index].len(), |node: int| position_at(level_index, node))
}

/// 前 `level_count` 层全部节点的位置，按 bump 次序（层级升序、同层从左到右）。
pub open spec fn bump_order_positions(levels: Seq<Seq<usize>>, level_count: int) -> Seq<ContainerTreeNodePosition>
    decreases level_count,
{
    if level_count <= 0 {
        Seq::empty()
    } else {
        bump_order_positions(levels, level_count - 1) + positions_of_level(levels, level_count - 1)
    }
}

/// 这一层的第 `node` 个节点罩住下一层（层下标 0 时是容器）的第 `child` 个：孩子下标落在它前面几个节点孩子数之和起的那一段里。
pub open spec fn child_lies_under_node(nodes_of_the_level: Seq<usize>, node: int, child: int) -> bool {
    &&& 0 <= node < nodes_of_the_level.len()
    &&& children_count_sum(nodes_of_the_level.take(node)) <= child
    &&& child < children_count_sum(nodes_of_the_level.take(node + 1))
}

/// 下一层第 `child` 个（层下标 0 时是容器）落在这一层哪个节点下：从第 `node` 个起往右找第一个前缀孩子数之和超过它的节点；
/// 都不超过时交回节点数。
pub open spec fn covering_node_from(nodes_of_the_level: Seq<usize>, child: int, node: int) -> int
    decreases nodes_of_the_level.len() - node,
{
    if node >= nodes_of_the_level.len() || child < children_count_sum(nodes_of_the_level.take(node + 1)) {
        node
    } else {
        covering_node_from(nodes_of_the_level, child, node + 1)
    }
}

/// 第 `container` 片容器在层下标 `level_index` 上的祖先是那一层的第几个节点。
pub open spec fn ancestor_index(levels: Seq<Seq<usize>>, level_index: nat, container: int) -> int
    decreases level_index,
{
    if level_index == 0 {
        covering_node_from(levels[0], container, 0)
    } else {
        covering_node_from(levels[level_index as int], ancestor_index(levels, (level_index - 1) as nat, container), 0)
    }
}

/// 层下标 `level_index` 上的第 `node` 个节点是第 `container` 片容器的祖先。
pub open spec fn node_is_an_ancestor_of_container(levels: Seq<Seq<usize>>, level_index: nat, node: int, container: int) -> bool {
    ancestor_index(levels, level_index, container) == node
}

/// 层下标 `level_index` 上的第 `node` 个节点是 `containers` 里某一片的祖先。
pub open spec fn node_is_an_ancestor_of_one_of(levels: Seq<Seq<usize>>, level_index: int, node: int, containers: Seq<usize>) -> bool {
    exists|listed: int|
        0 <= listed < containers.len() && #[trigger] node_is_an_ancestor_of_container(
            levels,
            level_index as nat,
            node,
            containers[listed] as int,
        )
}

/// 层下标 `level_index` 那一层之下的第 `child` 个被标过：层下标 0 之下是容器，标过就是它在 `containers` 里；
/// 再往上是下一层的节点，标过就是它是 `containers` 里某一片的祖先。
pub open spec fn child_is_marked(levels: Seq<Seq<usize>>, level_index: int, child: int, containers: Seq<usize>) -> bool {
    if level_index == 0 {
        exists|listed: int| 0 <= listed < containers.len() && #[trigger] containers[listed] == child
    } else {
        node_is_an_ancestor_of_one_of(levels, level_index - 1, child, containers)
    }
}

/// 层下标 `level_index` 上前 `node_count` 个节点里是 `containers` 某一片祖先的那几个的位置，从左到右。
pub open spec fn ancestors_in_level(
    levels: Seq<Seq<usize>>,
    level_index: int,
    containers: Seq<usize>,
    node_count: int,
) -> Seq<ContainerTreeNodePosition>
    decreases node_count,
{
    if node_count <= 0 {
        Seq::empty()
    } else if node_is_an_ancestor_of_one_of(levels, level_index, node_count - 1, containers) {
        ancestors_in_level(levels, level_index, containers, node_count - 1).push(position_at(level_index, node_count - 1))
    } else {
        ancestors_in_level(levels, level_index, containers, node_count - 1)
    }
}

/// 前 `level_count` 层里 `containers` 的全部祖先的位置，按 bump 次序。
pub open spec fn ancestors_below_level(
    levels: Seq<Seq<usize>>,
    containers: Seq<usize>,
    level_count: int,
) -> Seq<ContainerTreeNodePosition>
    decreases level_count,
{
    if level_count <= 0 {
        Seq::empty()
    } else {
        ancestors_below_level(levels, containers, level_count - 1) + ancestors_in_level(
            levels,
            level_count - 1,
            containers,
            levels[level_count - 1].len() as int,
        )
    }
}

/// 一层节点 `later_nodes` 是 `earlier_nodes` 在最右端长出来的：最右那个之前的孩子数逐个相同，最右那个不少，后面可以多出节点。
pub open spec fn nodes_extend(earlier_nodes: Seq<usize>, later_nodes: Seq<usize>) -> bool {
    &&& earlier_nodes.len() >= 1
    &&& later_nodes.len() >= earlier_nodes.len()
    &&& forall|node: int| 0 <= node < earlier_nodes.len() - 1 ==> #[trigger] later_nodes[node] == earlier_nodes[node]
    &&& later_nodes[earlier_nodes.len() - 1] >= earlier_nodes[earlier_nodes.len() - 1]
}

/// 形状 `later_levels` 是 `earlier_levels` 在最右端长出来的：每一层都是，上面可以多出几层。
pub open spec fn levels_extend(earlier_levels: Seq<Seq<usize>>, later_levels: Seq<Seq<usize>>) -> bool {
    &&& later_levels.len() >= earlier_levels.len()
    &&& forall|level_index: int|
        0 <= level_index < earlier_levels.len() ==> nodes_extend(#[trigger] earlier_levels[level_index], later_levels[level_index])
}

/// 最右端新开一片容器时一路往上挂到哪一层停：从 `level_index` 起第一个最右节点还装得下的层，都满就停在最高那一层。
pub open spec fn opening_stop_level_from(levels: Seq<Seq<usize>>, entries_per_node: int, level_index: int) -> int
    decreases levels.len() - level_index,
{
    if level_index + 1 >= levels.len() || levels[level_index].last() < entries_per_node {
        level_index
    } else {
        opening_stop_level_from(levels, entries_per_node, level_index + 1)
    }
}

/// 最右端新开一片容器之后的一层：停的那一层之下（与长高时停的那一层）最右端加一个只有一个孩子的节点，停的那一层装得下就给最右节点加一个孩子，
/// 再往上不动。
pub open spec fn level_after_opening(levels: Seq<Seq<usize>>, stop: int, grows: bool, level_index: int) -> Seq<usize> {
    if level_index < stop || (level_index == stop && grows) {
        levels[level_index].push(1usize)
    } else if level_index == stop {
        levels[level_index].update(levels[level_index].len() - 1, (levels[level_index].last() + 1) as usize)
    } else {
        levels[level_index]
    }
}

/// 最右端新开一片容器之后的形状（[`ContainerTreeShape::after_a_container_is_opened_at_the_right_end`] 的规格）：各层照 [`level_after_opening`]，
/// 停的那一层（只会是最高那一层）也满就再长一层两个孩子的新根；零根开一个层级 1 的根。
pub open spec fn levels_after_opening_at_the_right_end(levels: Seq<Seq<usize>>, entries_per_node: int) -> Seq<Seq<usize>> {
    if levels.len() == 0 {
        seq![seq![1usize]]
    } else {
        let stop = opening_stop_level_from(levels, entries_per_node, 0);
        let grows = levels[stop].last() >= entries_per_node;
        let modified = Seq::new(levels.len(), |level_index: int| level_after_opening(levels, stop, grows, level_index));
        if grows {
            modified.push(seq![2usize])
        } else {
            modified
        }
    }
}

/// 一版形状每一层有几个节点（层下标 0 在前）：上一版的形状在这次发布里只记这几个数（[`ContainerTreeNodeCounts`]）。
pub open spec fn node_counts_of(levels: Seq<Seq<usize>>) -> Seq<int> {
    Seq::new(levels.len(), |level_index: int| levels[level_index].len() as int)
}

/// 每层节点数是 `node_counts` 的那一版里，层下标 `level_index` 上有第 `node` 个节点。
pub open spec fn node_is_counted(node_counts: Seq<int>, level_index: int, node: int) -> bool {
    0 <= level_index < node_counts.len() && 0 <= node < node_counts[level_index]
}

/// 每层节点数是 `node_counts` 的那一版里这个位置上有节点。
pub open spec fn position_is_counted(node_counts: Seq<int>, position: ContainerTreeNodePosition) -> bool {
    1 <= position.level <= node_counts.len() && position.index_in_level < node_counts[position.level - 1]
}

/// `levels` 里不在上一版（每层节点数 `previous_node_counts`）的每个节点，都是 `rewritten_containers` 里某一片的祖先：照抄的节点上一版都有
/// （[`ContainerTreeAfterThisPublish::node_origins`]）。
pub open spec fn every_node_absent_from_the_previous_version_is_rewritten(
    previous_node_counts: Seq<int>,
    levels: Seq<Seq<usize>>,
    rewritten_containers: Seq<usize>,
) -> bool {
    forall|level_index: int, node: int|
        #[trigger] node_is_in_the_levels(levels, level_index, node) && !node_is_counted(previous_node_counts, level_index, node)
            ==> node_is_an_ancestor_of_one_of(levels, level_index, node, rewritten_containers)
}

/// 一共 `total` 片时最右端那 `count` 片容器，按叶序。
pub open spec fn rightmost_containers(total: int, count: int) -> Seq<usize> {
    Seq::new(count as nat, |listed: int| (total - count + listed) as usize)
}

/// 重写清单就是最右端那几片（一共 `total` 片）：按叶序一片挨一片、最后一片是最右那片。发布里重写的就是这样几片：
/// 追加只往最右那片里装、装满了在最右端开新的（[`append_records_to_the_open_container`]）。
pub open spec fn rewritten_containers_are_the_rightmost(rewritten_containers: Seq<usize>, total: int) -> bool {
    &&& rewritten_containers.len() <= total
    &&& forall|listed: int|
        0 <= listed < rewritten_containers.len() ==> #[trigger] rewritten_containers[listed] == total - rewritten_containers.len()
            + listed
}

/// 一组容器的记录按叶序依次连起来（「key 到条目」在规格里的样子）。
pub open spec fn records_of_containers<Record>(containers: Seq<RecordContainer<Record>>) -> Seq<Record>
    decreases containers.len(),
{
    if containers.len() == 0 {
        Seq::empty()
    } else {
        records_of_containers(containers.drop_last()) + containers.last().records@
    }
}

/// 每层每个节点的孩子数在规格里的样子。
pub open spec fn levels_of_children_counts(children_of_each_node_at_each_level: Seq<Vec<usize>>) -> Seq<Seq<usize>> {
    children_of_each_node_at_each_level.map_values(|nodes: Vec<usize>| nodes@)
}

// ===== 形状的引理 =====

/// 前 `count` 个之和加上第 `count` 个就是前 `count + 1` 个之和。
proof fn lemma_children_count_sum_of_one_more(children_of_each_node: Seq<usize>, count: int)
    requires
        0 <= count < children_of_each_node.len(),
    ensures
        children_count_sum(children_of_each_node.take(count + 1)) == children_count_sum(children_of_each_node.take(count))
            + children_of_each_node[count],
{
    assert(children_of_each_node.take(count + 1).drop_last() =~= children_of_each_node.take(count));
}

/// 取全部就是整层。
proof fn lemma_children_count_sum_of_the_whole_level(children_of_each_node: Seq<usize>)
    ensures
        children_count_sum(children_of_each_node.take(children_of_each_node.len() as int)) == children_count_sum(
            children_of_each_node,
        ),
{
    assert(children_of_each_node.take(children_of_each_node.len() as int) =~= children_of_each_node);
}

/// 孩子数之和不小于 0。
proof fn lemma_children_count_sum_is_not_negative(children_of_each_node: Seq<usize>)
    ensures
        children_count_sum(children_of_each_node) >= 0,
    decreases children_of_each_node.len(),
{
    if children_of_each_node.len() > 0 {
        lemma_children_count_sum_is_not_negative(children_of_each_node.drop_last());
    }
}

/// 前缀之和随前缀变长不减，不超过整层之和。
proof fn lemma_children_count_prefix_sum_is_monotone(children_of_each_node: Seq<usize>, shorter: int, longer: int)
    requires
        0 <= shorter <= longer <= children_of_each_node.len(),
    ensures
        children_count_sum(children_of_each_node.take(shorter)) <= children_count_sum(children_of_each_node.take(longer)),
        children_count_sum(children_of_each_node.take(longer)) <= children_count_sum(children_of_each_node),
    decreases children_of_each_node.len() - shorter,
{
    if shorter < children_of_each_node.len() {
        lemma_children_count_sum_of_one_more(children_of_each_node, shorter);
        if shorter < longer {
            lemma_children_count_prefix_sum_is_monotone(children_of_each_node, shorter + 1, longer);
        } else {
            lemma_children_count_prefix_sum_is_monotone(children_of_each_node, shorter + 1, shorter + 1);
        }
    } else {
        lemma_children_count_sum_of_the_whole_level(children_of_each_node);
    }
}

/// 每个节点至少一个孩子时，孩子数之和不小于节点数。
proof fn lemma_children_count_sum_is_at_least_the_node_count(children_of_each_node: Seq<usize>)
    requires
        forall|node: int| 0 <= node < children_of_each_node.len() ==> #[trigger] children_of_each_node[node] >= 1,
    ensures
        children_count_sum(children_of_each_node) >= children_of_each_node.len(),
    decreases children_of_each_node.len(),
{
    if children_of_each_node.len() > 0 {
        let shorter = children_of_each_node.drop_last();
        assert forall|node: int| 0 <= node < shorter.len() implies #[trigger] shorter[node] >= 1 by {
            assert(shorter[node] == children_of_each_node[node]);
        }
        lemma_children_count_sum_is_at_least_the_node_count(shorter);
        assert(children_of_each_node.last() >= 1);
    }
}

/// 末尾加一个，和加上它。
proof fn lemma_children_count_sum_after_a_push(children_of_each_node: Seq<usize>, value: usize)
    ensures
        children_count_sum(children_of_each_node.push(value)) == children_count_sum(children_of_each_node) + value,
{
    assert(children_of_each_node.push(value).drop_last() =~= children_of_each_node);
}

/// 改最右那一个，和随之变。
proof fn lemma_children_count_sum_after_an_update_of_the_last(children_of_each_node: Seq<usize>, value: usize)
    requires
        children_of_each_node.len() >= 1,
    ensures
        children_count_sum(children_of_each_node.update(children_of_each_node.len() - 1, value)) == children_count_sum(
            children_of_each_node,
        ) - children_of_each_node.last() + value,
{
    assert(children_of_each_node.update(children_of_each_node.len() - 1, value).drop_last()
        =~= children_of_each_node.drop_last());
}

/// 只有一个孩子数的一层，和就是它。
proof fn lemma_children_count_sum_of_a_single_node(value: usize)
    ensures
        children_count_sum(seq![value]) == value,
{
    reveal_with_fuel(children_count_sum, 2);
    assert(seq![value].drop_last() =~= Seq::<usize>::empty());
}

/// 形状良好时，每一层的节点数不超过容器数。
proof fn lemma_every_level_has_at_most_as_many_nodes_as_containers(levels: Seq<Seq<usize>>, level_index: int)
    requires
        container_tree_shape_is_well_formed(levels),
        0 <= level_index < levels.len(),
    ensures
        levels[level_index].len() <= container_count(levels),
    decreases level_index,
{
    assert forall|node: int| 0 <= node < levels[level_index].len() implies #[trigger] levels[level_index][node] >= 1 by {
        assert(levels[level_index][node] >= 1);
    }
    lemma_children_count_sum_is_at_least_the_node_count(levels[level_index]);
    if level_index > 0 {
        assert(children_count_sum(levels[level_index]) == levels[level_index - 1].len());
        lemma_every_level_has_at_most_as_many_nodes_as_containers(levels, level_index - 1);
    }
}

/// 一层里最右那个节点罩住下一层的最后一个。
proof fn lemma_the_rightmost_node_holds_the_last_child(nodes_of_the_level: Seq<usize>)
    requires
        nodes_of_the_level.len() >= 1,
        forall|node: int| 0 <= node < nodes_of_the_level.len() ==> #[trigger] nodes_of_the_level[node] >= 1,
    ensures
        child_lies_under_node(
            nodes_of_the_level,
            nodes_of_the_level.len() - 1,
            children_count_sum(nodes_of_the_level) - 1,
        ),
{
    let last = nodes_of_the_level.len() - 1;
    lemma_children_count_sum_of_one_more(nodes_of_the_level, last);
    lemma_children_count_sum_of_the_whole_level(nodes_of_the_level);
    assert(nodes_of_the_level[last] >= 1);
}

/// 一层里一个孩子至多落在一个节点下（前缀孩子数之和随节点不减）。
proof fn lemma_a_child_lies_under_at_most_one_node(nodes_of_the_level: Seq<usize>, node: int, other_node: int, child: int)
    requires
        child_lies_under_node(nodes_of_the_level, node, child),
        child_lies_under_node(nodes_of_the_level, other_node, child),
    ensures
        node == other_node,
{
    if node < other_node {
        lemma_children_count_prefix_sum_is_monotone(nodes_of_the_level, node + 1, other_node);
    } else if other_node < node {
        lemma_children_count_prefix_sum_is_monotone(nodes_of_the_level, other_node + 1, node);
    }
}

/// 从第 `node` 个起往右找，找到的就是罩住它的那一个节点。
proof fn lemma_covering_node_from(nodes_of_the_level: Seq<usize>, child: int, node: int)
    requires
        0 <= node <= nodes_of_the_level.len(),
        children_count_sum(nodes_of_the_level.take(node)) <= child,
        child < children_count_sum(nodes_of_the_level),
    ensures
        node <= covering_node_from(nodes_of_the_level, child, node) < nodes_of_the_level.len(),
        child_lies_under_node(nodes_of_the_level, covering_node_from(nodes_of_the_level, child, node), child),
    decreases nodes_of_the_level.len() - node,
{
    if node >= nodes_of_the_level.len() {
        lemma_children_count_sum_of_the_whole_level(nodes_of_the_level);
    } else if child >= children_count_sum(nodes_of_the_level.take(node + 1)) {
        lemma_covering_node_from(nodes_of_the_level, child, node + 1);
    }
}

/// 落在这一层孩子数之和以内的孩子，罩住它的节点就是从头找到的那一个，别的节点都不罩它。
proof fn lemma_covering_node(nodes_of_the_level: Seq<usize>, child: int)
    requires
        0 <= child < children_count_sum(nodes_of_the_level),
    ensures
        0 <= covering_node_from(nodes_of_the_level, child, 0) < nodes_of_the_level.len(),
        child_lies_under_node(nodes_of_the_level, covering_node_from(nodes_of_the_level, child, 0), child),
        forall|node: int| #[trigger] child_lies_under_node(nodes_of_the_level, node, child) ==> node == covering_node_from(
            nodes_of_the_level,
            child,
            0,
        ),
{
    assert(nodes_of_the_level.take(0) =~= Seq::<usize>::empty());
    lemma_covering_node_from(nodes_of_the_level, child, 0);
    assert forall|node: int| #[trigger] child_lies_under_node(nodes_of_the_level, node, child) implies node
        == covering_node_from(nodes_of_the_level, child, 0) by {
        lemma_a_child_lies_under_at_most_one_node(
            nodes_of_the_level,
            node,
            covering_node_from(nodes_of_the_level, child, 0),
            child,
        );
    }
}

/// 形状良好时，每一片容器在每一层上都有祖先：它落在那一层范围内，罩住它在下一层的祖先（层下标 0 时就是它自己）。
proof fn lemma_ancestor_index(levels: Seq<Seq<usize>>, level_index: nat, container: int)
    requires
        container_tree_shape_is_well_formed(levels),
        level_index < levels.len(),
        0 <= container < container_count(levels),
    ensures
        0 <= ancestor_index(levels, level_index, container) < levels[level_index as int].len(),
        child_lies_under_node(
            levels[level_index as int],
            ancestor_index(levels, level_index, container),
            if level_index == 0 {
                container
            } else {
                ancestor_index(levels, (level_index - 1) as nat, container)
            },
        ),
    decreases level_index,
{
    if level_index == 0 {
        lemma_covering_node(levels[0], container);
    } else {
        let below = (level_index - 1) as nat;
        lemma_ancestor_index(levels, below, container);
        assert(children_count_sum(levels[level_index as int]) == levels[below as int].len());
        lemma_covering_node(levels[level_index as int], ancestor_index(levels, below, container));
    }
}

/// 形状良好时，每一层最右那个节点都是最后一片容器的祖先（最右那一条路径）。
proof fn lemma_the_rightmost_path_holds_the_last_container(levels: Seq<Seq<usize>>, level_index: nat)
    requires
        container_tree_shape_is_well_formed(levels),
        level_index < levels.len(),
    ensures
        ancestor_index(levels, level_index, container_count(levels) - 1) == levels[level_index as int].len() - 1,
    decreases level_index,
{
    let nodes = levels[level_index as int];
    assert forall|node: int| 0 <= node < nodes.len() implies #[trigger] nodes[node] >= 1 by {
        assert(levels[level_index as int][node] >= 1);
    }
    assert(nodes.len() >= 1);
    lemma_the_rightmost_node_holds_the_last_child(nodes);
    if level_index == 0 {
        lemma_children_count_sum_is_at_least_the_node_count(nodes);
        lemma_covering_node(nodes, container_count(levels) - 1);
    } else {
        let below = (level_index - 1) as nat;
        lemma_the_rightmost_path_holds_the_last_container(levels, below);
        assert(children_count_sum(nodes) == levels[below as int].len());
        lemma_covering_node(nodes, levels[below as int].len() - 1);
    }
}

/// 一层在最右端长出来之后，原来罩住的孩子照样罩住。
proof fn lemma_extended_nodes_keep_their_children(earlier_nodes: Seq<usize>, later_nodes: Seq<usize>, node: int, child: int)
    requires
        nodes_extend(earlier_nodes, later_nodes),
        child_lies_under_node(earlier_nodes, node, child),
    ensures
        child_lies_under_node(later_nodes, node, child),
{
    let last = earlier_nodes.len() - 1;
    assert(later_nodes.take(last) =~= earlier_nodes.take(last));
    if node < last {
        assert(later_nodes.take(node) =~= earlier_nodes.take(node));
        assert(later_nodes.take(node + 1) =~= earlier_nodes.take(node + 1));
    } else {
        lemma_children_count_sum_of_one_more(earlier_nodes, last);
        lemma_children_count_sum_of_one_more(later_nodes, last);
    }
}

/// 形状在最右端长出来之后，原有的容器在原有层上的祖先不变。
proof fn lemma_extended_levels_keep_their_ancestors(
    earlier_levels: Seq<Seq<usize>>,
    later_levels: Seq<Seq<usize>>,
    level_index: nat,
    container: int,
)
    requires
        container_tree_shape_is_well_formed(earlier_levels),
        levels_extend(earlier_levels, later_levels),
        level_index < earlier_levels.len(),
        0 <= container < container_count(earlier_levels),
    ensures
        ancestor_index(later_levels, level_index, container) == ancestor_index(earlier_levels, level_index, container),
    decreases level_index,
{
    lemma_ancestor_index(earlier_levels, level_index, container);
    assert(nodes_extend(earlier_levels[level_index as int], later_levels[level_index as int]));
    let child = if level_index == 0 {
        container
    } else {
        ancestor_index(earlier_levels, (level_index - 1) as nat, container)
    };
    if level_index > 0 {
        lemma_ancestor_index(earlier_levels, (level_index - 1) as nat, container);
        lemma_extended_levels_keep_their_ancestors(earlier_levels, later_levels, (level_index - 1) as nat, container);
    }
    let node = ancestor_index(earlier_levels, level_index, container);
    lemma_extended_nodes_keep_their_children(earlier_levels[level_index as int], later_levels[level_index as int], node, child);
    lemma_children_count_prefix_sum_is_monotone(later_levels[level_index as int], node + 1, node + 1);
    lemma_covering_node(later_levels[level_index as int], child);
}

/// 在最右端长出来是传递的。
proof fn lemma_levels_extend_transitively(
    earliest_levels: Seq<Seq<usize>>,
    middle_levels: Seq<Seq<usize>>,
    latest_levels: Seq<Seq<usize>>,
)
    requires
        levels_extend(earliest_levels, middle_levels),
        levels_extend(middle_levels, latest_levels),
    ensures
        levels_extend(earliest_levels, latest_levels),
{
    assert forall|level_index: int| 0 <= level_index < earliest_levels.len() implies nodes_extend(
        #[trigger] earliest_levels[level_index],
        latest_levels[level_index],
    ) by {
        assert(nodes_extend(earliest_levels[level_index], middle_levels[level_index]));
        assert(nodes_extend(middle_levels[level_index], latest_levels[level_index]));
        let last = earliest_levels[level_index].len() - 1;
        assert forall|node: int| 0 <= node < last implies #[trigger] latest_levels[level_index][node]
            == earliest_levels[level_index][node] by {
            assert(middle_levels[level_index][node] == earliest_levels[level_index][node]);
        }
        if last < middle_levels[level_index].len() - 1 {
            assert(latest_levels[level_index][last] == middle_levels[level_index][last]);
        }
    }
}

/// 一路往上找停的那一层：停的那一层之下每层最右节点都满；不是最高那一层时停的那一层装得下。
proof fn lemma_opening_stop_level(levels: Seq<Seq<usize>>, entries_per_node: int, level_index: int)
    requires
        0 <= level_index < levels.len(),
    ensures
        level_index <= opening_stop_level_from(levels, entries_per_node, level_index) < levels.len(),
        forall|full_level_index: int|
            level_index <= full_level_index < opening_stop_level_from(levels, entries_per_node, level_index) ==> (
            #[trigger] levels[full_level_index]).last() >= entries_per_node,
        opening_stop_level_from(levels, entries_per_node, level_index) + 1 < levels.len() ==> levels[opening_stop_level_from(
            levels,
            entries_per_node,
            level_index,
        )].last() < entries_per_node,
    decreases levels.len() - level_index,
{
    if !(level_index + 1 >= levels.len() || levels[level_index].last() < entries_per_node) {
        lemma_opening_stop_level(levels, entries_per_node, level_index + 1);
    }
}

/// 最右端新开一片容器之后的一层：孩子数之和、节点数怎么变，在最右端长出来，每个节点至少一个孩子。
proof fn lemma_level_after_opening(levels: Seq<Seq<usize>>, entries_per_node: int, level_index: int)
    requires
        container_tree_shape_is_well_formed(levels),
        entries_per_node <= usize::MAX,
        0 <= level_index < levels.len(),
    ensures
        ({
            let stop = opening_stop_level_from(levels, entries_per_node, 0);
            let grows = levels[stop].last() >= entries_per_node;
            let opened = level_after_opening(levels, stop, grows, level_index);
            &&& children_count_sum(opened) == children_count_sum(levels[level_index]) + (if level_index <= stop {
                1int
            } else {
                0int
            })
            &&& opened.len() == levels[level_index].len() + (if level_index < stop || (level_index == stop && grows) {
                1int
            } else {
                0int
            })
            &&& nodes_extend(levels[level_index], opened)
            &&& forall|node: int| 0 <= node < opened.len() ==> #[trigger] opened[node] >= 1
        }),
{
    lemma_opening_stop_level(levels, entries_per_node, 0);
    let stop = opening_stop_level_from(levels, entries_per_node, 0);
    let grows = levels[stop].last() >= entries_per_node;
    let nodes = levels[level_index];
    let opened = level_after_opening(levels, stop, grows, level_index);
    assert(nodes.len() >= 1);
    assert forall|node: int| 0 <= node < nodes.len() implies #[trigger] nodes[node] >= 1 by {
        assert(levels[level_index][node] >= 1);
    }
    if level_index < stop || (level_index == stop && grows) {
        lemma_children_count_sum_after_a_push(nodes, 1usize);
        if level_index < stop {
            assert(levels[level_index].last() >= entries_per_node);
        }
        assert forall|node: int| 0 <= node < opened.len() implies #[trigger] opened[node] >= 1 by {
            if node < nodes.len() {
                assert(opened[node] == nodes[node]);
            }
        }
    } else if level_index == stop {
        assert(stop + 1 >= levels.len() ==> !grows);
        lemma_children_count_sum_after_an_update_of_the_last(nodes, (nodes.last() + 1) as usize);
        assert forall|node: int| 0 <= node < opened.len() implies #[trigger] opened[node] >= 1 by {
            if node < nodes.len() - 1 {
                assert(opened[node] == nodes[node]);
            }
        }
    }
}

/// 最右端新开一片容器之后形状照样良好、容器数加一。
proof fn lemma_opening_keeps_the_shape_well_formed(levels: Seq<Seq<usize>>, entries_per_node: int)
    requires
        container_tree_shape_is_well_formed(levels),
        container_count(levels) + 1 <= usize::MAX,
        2 <= entries_per_node <= usize::MAX,
    ensures
        container_tree_shape_is_well_formed(levels_after_opening_at_the_right_end(levels, entries_per_node)),
        container_count(levels_after_opening_at_the_right_end(levels, entries_per_node)) == container_count(levels) + 1,
{
    let opened = levels_after_opening_at_the_right_end(levels, entries_per_node);
    if levels.len() == 0 {
        lemma_children_count_sum_of_a_single_node(1usize);
        assert(opened[0] =~= seq![1usize]);
    } else {
        lemma_opening_stop_level(levels, entries_per_node, 0);
        let stop = opening_stop_level_from(levels, entries_per_node, 0);
        let grows = levels[stop].last() >= entries_per_node;
        assert(grows ==> stop == levels.len() - 1);
        assert forall|level_index: int| 0 <= level_index < levels.len() implies #[trigger] opened[level_index]
            == level_after_opening(levels, stop, grows, level_index) by {}
        if grows {
            lemma_children_count_sum_of_a_single_node(2usize);
            assert(opened[levels.len() as int] =~= seq![2usize]);
        }
        assert forall|level_index: int| 0 <= level_index < opened.len() implies (#[trigger] opened[level_index]).len() >= 1
            && forall|node: int| 0 <= node < opened[level_index].len() ==> #[trigger] opened[level_index][node] >= 1 by {
            if level_index < levels.len() {
                lemma_level_after_opening(levels, entries_per_node, level_index);
            }
        }
        assert forall|upper_level_index: int| 1 <= upper_level_index < opened.len() implies children_count_sum(
            #[trigger] opened[upper_level_index],
        ) == opened[upper_level_index - 1].len() by {
            lemma_level_after_opening(levels, entries_per_node, upper_level_index - 1);
            if upper_level_index < levels.len() {
                lemma_level_after_opening(levels, entries_per_node, upper_level_index);
                assert(children_count_sum(levels[upper_level_index]) == levels[upper_level_index - 1].len());
            } else {
                assert(levels[levels.len() - 1].len() == 1);
            }
        }
        lemma_level_after_opening(levels, entries_per_node, 0);
        lemma_level_after_opening(levels, entries_per_node, levels.len() - 1);
        assert(levels[levels.len() - 1].len() == 1);
    }
}

/// 最右端新开一片容器之后形状在最右端长出来；新出现的节点都在它那一层的最右端。
proof fn lemma_opening_extends_the_levels_at_the_right_end(levels: Seq<Seq<usize>>, entries_per_node: int)
    requires
        container_tree_shape_is_well_formed(levels),
        2 <= entries_per_node <= usize::MAX,
    ensures
        levels_extend(levels, levels_after_opening_at_the_right_end(levels, entries_per_node)),
        forall|level_index: int, node: int|
            #[trigger] node_is_in_the_levels(levels_after_opening_at_the_right_end(levels, entries_per_node), level_index, node)
                && !node_is_in_the_levels(levels, level_index, node) ==> node
                == levels_after_opening_at_the_right_end(levels, entries_per_node)[level_index].len() - 1,
{
    let opened = levels_after_opening_at_the_right_end(levels, entries_per_node);
    if levels.len() == 0 {
        assert(opened[0] =~= seq![1usize]);
    } else {
        lemma_opening_stop_level(levels, entries_per_node, 0);
        let stop = opening_stop_level_from(levels, entries_per_node, 0);
        let grows = levels[stop].last() >= entries_per_node;
        assert forall|level_index: int| 0 <= level_index < levels.len() implies #[trigger] opened[level_index]
            == level_after_opening(levels, stop, grows, level_index) by {}
        if grows {
            assert(opened[levels.len() as int] =~= seq![2usize]);
        }
        assert forall|level_index: int| 0 <= level_index < levels.len() implies nodes_extend(
            #[trigger] levels[level_index],
            opened[level_index],
        ) by {
            lemma_level_after_opening(levels, entries_per_node, level_index);
        }
        assert forall|level_index: int, node: int|
            #[trigger] node_is_in_the_levels(opened, level_index, node) && !node_is_in_the_levels(levels, level_index, node) implies node
                == opened[level_index].len() - 1 by {
            if level_index < levels.len() {
                lemma_level_after_opening(levels, entries_per_node, level_index);
            }
        }
    }
}

/// 多重写几片，罩住的照样罩住。
proof fn lemma_more_rewritten_containers_keep_every_new_node_rewritten(
    previous_node_counts: Seq<int>,
    levels: Seq<Seq<usize>>,
    rewritten_containers: Seq<usize>,
    added: usize,
)
    requires
        every_node_absent_from_the_previous_version_is_rewritten(previous_node_counts, levels, rewritten_containers),
    ensures
        every_node_absent_from_the_previous_version_is_rewritten(previous_node_counts, levels, rewritten_containers.push(added)),
{
    let longer = rewritten_containers.push(added);
    assert forall|level_index: int, node: int|
        #[trigger] node_is_in_the_levels(levels, level_index, node) && !node_is_counted(
            previous_node_counts,
            level_index,
            node,
        ) implies node_is_an_ancestor_of_one_of(levels, level_index, node, longer) by {
        let listed = choose|listed: int|
            0 <= listed < rewritten_containers.len() && #[trigger] node_is_an_ancestor_of_container(
                levels,
                level_index as nat,
                node,
                rewritten_containers[listed] as int,
            );
        assert(longer[listed] == rewritten_containers[listed]);
    }
}

/// 最右端新开一片容器、把它记进重写清单之后，新出现的节点照样都罩着重写的某一片。
proof fn lemma_opening_keeps_every_new_node_rewritten(
    previous_node_counts: Seq<int>,
    levels: Seq<Seq<usize>>,
    entries_per_node: int,
    rewritten_containers: Seq<usize>,
)
    requires
        container_tree_shape_is_well_formed(levels),
        container_count(levels) + 1 <= usize::MAX,
        2 <= entries_per_node <= usize::MAX,
        forall|listed: int| 0 <= listed < rewritten_containers.len() ==> #[trigger] rewritten_containers[listed] < container_count(levels),
        every_node_absent_from_the_previous_version_is_rewritten(previous_node_counts, levels, rewritten_containers),
    ensures
        every_node_absent_from_the_previous_version_is_rewritten(
            previous_node_counts,
            levels_after_opening_at_the_right_end(levels, entries_per_node),
            rewritten_containers.push(container_count(levels) as usize),
        ),
{
    let opened = levels_after_opening_at_the_right_end(levels, entries_per_node);
    let longer = rewritten_containers.push(container_count(levels) as usize);
    if levels.len() > 0 {
        lemma_children_count_sum_is_not_negative(levels[0]);
    }
    lemma_opening_keeps_the_shape_well_formed(levels, entries_per_node);
    lemma_opening_extends_the_levels_at_the_right_end(levels, entries_per_node);
    assert forall|level_index: int, node: int|
        #[trigger] node_is_in_the_levels(opened, level_index, node) && !node_is_counted(
            previous_node_counts,
            level_index,
            node,
        ) implies node_is_an_ancestor_of_one_of(opened, level_index, node, longer) by {
        if node_is_in_the_levels(levels, level_index, node) {
            let listed = choose|listed: int|
                0 <= listed < rewritten_containers.len() && #[trigger] node_is_an_ancestor_of_container(
                    levels,
                    level_index as nat,
                    node,
                    rewritten_containers[listed] as int,
                );
            lemma_extended_levels_keep_their_ancestors(levels, opened, level_index as nat, rewritten_containers[listed] as int);
            assert(longer[listed] == rewritten_containers[listed]);
            assert(node_is_an_ancestor_of_container(opened, level_index as nat, node, longer[listed] as int));
        } else {
            assert(node == opened[level_index].len() - 1);
            lemma_the_rightmost_path_holds_the_last_container(opened, level_index as nat);
            assert(longer[rewritten_containers.len() as int] == container_count(levels));
            assert(node_is_an_ancestor_of_container(
                opened,
                level_index as nat,
                node,
                longer[rewritten_containers.len() as int] as int,
            ));
        }
    }
}

/// 一个节点是不是 `containers` 某一片的祖先，就看它罩住的孩子里有没有被标过的。
proof fn lemma_node_is_marked_by_its_children(levels: Seq<Seq<usize>>, level_index: int, node: int, containers: Seq<usize>)
    requires
        container_tree_shape_is_well_formed(levels),
        0 <= level_index < levels.len(),
        forall|listed: int| 0 <= listed < containers.len() ==> #[trigger] containers[listed] < container_count(levels),
    ensures
        node_is_an_ancestor_of_one_of(levels, level_index, node, containers) <==> exists|child: int|
            child_lies_under_node(levels[level_index], node, child) && #[trigger] child_is_marked(
                levels,
                level_index,
                child,
                containers,
            ),
{
    if node_is_an_ancestor_of_one_of(levels, level_index, node, containers) {
        let listed = choose|listed: int|
            0 <= listed < containers.len() && #[trigger] node_is_an_ancestor_of_container(
                levels,
                level_index as nat,
                node,
                containers[listed] as int,
            );
        lemma_ancestor_index(levels, level_index as nat, containers[listed] as int);
        let child = if level_index == 0 {
            containers[listed] as int
        } else {
            ancestor_index(levels, (level_index - 1) as nat, containers[listed] as int)
        };
        assert(child_lies_under_node(levels[level_index], node, child));
        if level_index > 0 {
            assert(node_is_an_ancestor_of_container(levels, (level_index - 1) as nat, child, containers[listed] as int));
        }
        assert(child_is_marked(levels, level_index, child, containers));
    }
    if exists|child: int|
        child_lies_under_node(levels[level_index], node, child) && #[trigger] child_is_marked(
            levels,
            level_index,
            child,
            containers,
        ) {
        let child = choose|child: int|
            child_lies_under_node(levels[level_index], node, child) && #[trigger] child_is_marked(
                levels,
                level_index,
                child,
                containers,
            );
        lemma_a_node_over_a_marked_child_is_an_ancestor(levels, level_index, node, child, containers);
    }
}

/// 一个节点罩住的孩子里有一个被标过，它就是 `containers` 某一片的祖先：[`lemma_node_is_marked_by_its_children`] 从孩子推到节点的那一半，
/// 单列一个引理，那一个的查询只剩从节点推到孩子那一半（整 crate 一起验时两半挤在一个查询里，在登记的 rlimit 上验不过）。
proof fn lemma_a_node_over_a_marked_child_is_an_ancestor(
    levels: Seq<Seq<usize>>,
    level_index: int,
    node: int,
    child: int,
    containers: Seq<usize>,
)
    requires
        container_tree_shape_is_well_formed(levels),
        0 <= level_index < levels.len(),
        forall|listed: int| 0 <= listed < containers.len() ==> #[trigger] containers[listed] < container_count(levels),
        child_lies_under_node(levels[level_index], node, child),
        child_is_marked(levels, level_index, child, containers),
    ensures
        node_is_an_ancestor_of_one_of(levels, level_index, node, containers),
{
    lemma_children_count_prefix_sum_is_monotone(levels[level_index], node + 1, node + 1);
    lemma_children_count_sum_is_not_negative(levels[level_index].take(node));
    lemma_covering_node(levels[level_index], child);
    // 两支都在 `node_is_an_ancestor_of_one_of` 自己的触发式（层下标写成 `level_index as nat`）上给出见证。
    if level_index == 0 {
        let listed = choose|listed: int| 0 <= listed < containers.len() && #[trigger] containers[listed] == child;
        assert(node_is_an_ancestor_of_container(levels, level_index as nat, node, containers[listed] as int));
        assert(0 <= listed < containers.len());
    } else {
        let listed = choose|listed: int|
            0 <= listed < containers.len() && #[trigger] node_is_an_ancestor_of_container(
                levels,
                (level_index - 1) as nat,
                child,
                containers[listed] as int,
            );
        assert(node_is_an_ancestor_of_container(levels, level_index as nat, node, containers[listed] as int));
        assert(0 <= listed < containers.len());
    }
}

/// 两段接起来，含不含某一样就看两段各含不含。
proof fn lemma_concatenation_contains<Element>(earlier: Seq<Element>, later: Seq<Element>, element: Element)
    ensures
        (earlier + later).contains(element) <==> (earlier.contains(element) || later.contains(element)),
{
    let joined = earlier + later;
    if joined.contains(element) {
        let found = choose|found: int| 0 <= found < joined.len() && joined[found] == element;
        if found < earlier.len() {
            assert(earlier[found] == element);
        } else {
            assert(later[found - earlier.len()] == element);
        }
    }
    if earlier.contains(element) {
        let found = choose|found: int| 0 <= found < earlier.len() && earlier[found] == element;
        assert(joined[found] == element);
    }
    if later.contains(element) {
        let found = choose|found: int| 0 <= found < later.len() && later[found] == element;
        assert(joined[earlier.len() + found] == element);
    }
}

/// 一层里的祖先位置都在这一层上、序号小于前几个的个数，且「在这张单子里」就是「是某一片的祖先」。
proof fn lemma_ancestors_in_level(
    levels: Seq<Seq<usize>>,
    level_index: int,
    containers: Seq<usize>,
    node_count: int,
)
    requires
        0 <= level_index < 255,
        0 <= node_count <= u32::MAX + 1,
    ensures
        forall|listed: int|
            0 <= listed < ancestors_in_level(levels, level_index, containers, node_count).len() ==> {
                let position = #[trigger] ancestors_in_level(levels, level_index, containers, node_count)[listed];
                &&& position.level == level_index + 1
                &&& position.index_in_level < node_count
            },
        forall|node: int|
            0 <= node < node_count ==> (ancestors_in_level(levels, level_index, containers, node_count).contains(
                #[trigger] position_at(level_index, node),
            ) <==> node_is_an_ancestor_of_one_of(levels, level_index, node, containers)),
    decreases node_count,
{
    if node_count > 0 {
        lemma_ancestors_in_level(levels, level_index, containers, node_count - 1);
        let earlier = ancestors_in_level(levels, level_index, containers, node_count - 1);
        let last = position_at(level_index, node_count - 1);
        let listed_here = ancestors_in_level(levels, level_index, containers, node_count);
        let last_is_listed = node_is_an_ancestor_of_one_of(levels, level_index, node_count - 1, containers);
        assert(listed_here == if last_is_listed {
            earlier.push(last)
        } else {
            earlier
        });
        assert(earlier.push(last) =~= earlier + seq![last]);
        assert forall|node: int| 0 <= node < node_count implies (listed_here.contains(#[trigger] position_at(level_index, node))
            <==> node_is_an_ancestor_of_one_of(levels, level_index, node, containers)) by {
            let position = position_at(level_index, node);
            lemma_concatenation_contains(earlier, seq![last], position);
            if node < node_count - 1 {
                assert(position.index_in_level != last.index_in_level);
                assert(!seq![last].contains(position)) by {
                    if seq![last].contains(position) {
                        let found = choose|found: int| 0 <= found < 1 && seq![last][found] == position;
                    }
                }
            } else {
                assert(seq![last][0] == last);
                if earlier.contains(position) {
                    let found = choose|found: int| 0 <= found < earlier.len() && earlier[found] == position;
                    assert(earlier[found].index_in_level < node_count - 1);
                }
            }
        }
        assert forall|listed: int| 0 <= listed < listed_here.len() implies {
            let position = #[trigger] listed_here[listed];
            &&& position.level == level_index + 1
            &&& position.index_in_level < node_count
        } by {
            if listed < earlier.len() {
                assert(listed_here[listed] == earlier[listed]);
            }
        }
    }
}

/// 一层前 `node_count` 个节点都不是 `containers` 某一片的祖先时，那几个里的祖先位置是空的。
proof fn lemma_no_ancestor_among_the_first(levels: Seq<Seq<usize>>, level_index: int, containers: Seq<usize>, node_count: int)
    requires
        forall|node: int|
            0 <= node < node_count ==> !(#[trigger] node_is_an_ancestor_of_one_of(levels, level_index, node, containers)),
    ensures
        ancestors_in_level(levels, level_index, containers, node_count) == Seq::<ContainerTreeNodePosition>::empty(),
    decreases node_count,
{
    if node_count > 0 {
        assert(!node_is_an_ancestor_of_one_of(levels, level_index, node_count - 1, containers));
        lemma_no_ancestor_among_the_first(levels, level_index, containers, node_count - 1);
    }
}

/// 最右端那 `count` 片里有没有第 `child` 片：有就是它落在最右端那几片的头一片到最右那片之间（一共 `total` 片）。
proof fn lemma_rightmost_containers_hold_exactly_the_ones_from_their_first(total: int, count: int, child: int)
    requires
        0 <= count <= total <= usize::MAX,
    ensures
        (exists|listed: int|
            0 <= listed < rightmost_containers(total, count).len() && #[trigger] rightmost_containers(total, count)[listed] == child)
            <==> (total - count <= child < total),
{
    let containers = rightmost_containers(total, count);
    if total - count <= child < total {
        let listed = child - (total - count);
        assert(containers[listed] == child);
    }
    if exists|listed: int| 0 <= listed < containers.len() && #[trigger] containers[listed] == child {
        let listed = choose|listed: int| 0 <= listed < containers.len() && #[trigger] containers[listed] == child;
        assert(containers[listed] == total - count + listed);
    }
}

/// 重写清单是最右端那几片时，它就是 [`rightmost_containers`] 那一份。
proof fn lemma_rewritten_containers_are_the_rightmost_ones(rewritten_containers: Seq<usize>, total: int)
    requires
        rewritten_containers_are_the_rightmost(rewritten_containers, total),
        total <= usize::MAX,
    ensures
        rewritten_containers == rightmost_containers(total, rewritten_containers.len() as int),
{
    assert(rewritten_containers =~= rightmost_containers(total, rewritten_containers.len() as int));
}

/// 一片都没给时哪一层都没有祖先。
proof fn lemma_no_containers_have_no_ancestors(levels: Seq<Seq<usize>>, level_count: int)
    ensures
        ancestors_below_level(levels, Seq::<usize>::empty(), level_count) == Seq::<ContainerTreeNodePosition>::empty(),
    decreases level_count,
{
    if level_count > 0 {
        lemma_no_containers_have_no_ancestors(levels, level_count - 1);
        assert forall|node: int| 0 <= node < levels[level_count - 1].len() implies !(#[trigger] node_is_an_ancestor_of_one_of(
            levels,
            level_count - 1,
            node,
            Seq::<usize>::empty(),
        )) by {}
        lemma_no_ancestor_among_the_first(levels, level_count - 1, Seq::<usize>::empty(), levels[level_count - 1].len() as int);
        assert(Seq::<ContainerTreeNodePosition>::empty() + Seq::<ContainerTreeNodePosition>::empty()
            =~= Seq::<ContainerTreeNodePosition>::empty());
    }
}

/// 前几层的祖先位置里，「在这张单子里」就是「是某一片的祖先」；单子里的层级不超过层数。
pub proof fn lemma_ancestors_below_level_contains(levels: Seq<Seq<usize>>, containers: Seq<usize>, level_count: int)
    requires
        0 <= level_count <= levels.len(),
        shape_fits_the_position_fields(levels),
        container_tree_shape_is_well_formed(levels),
    ensures
        forall|listed: int|
            0 <= listed < ancestors_below_level(levels, containers, level_count).len() ==> (#[trigger] ancestors_below_level(
                levels,
                containers,
                level_count,
            )[listed]).level <= level_count,
        forall|level_index: int, node: int|
            #[trigger] node_is_in_the_levels(levels, level_index, node) && level_index < level_count ==> (
            ancestors_below_level(levels, containers, level_count).contains(position_at(level_index, node))
                <==> node_is_an_ancestor_of_one_of(levels, level_index, node, containers)),
    decreases level_count,
{
    if level_count > 0 {
        let level_index = level_count - 1;
        lemma_ancestors_below_level_contains(levels, containers, level_index);
        lemma_every_level_has_at_most_as_many_nodes_as_containers(levels, level_index);
        lemma_ancestors_in_level(levels, level_index, containers, levels[level_index].len() as int);
        let earlier = ancestors_below_level(levels, containers, level_index);
        let here = ancestors_in_level(levels, level_index, containers, levels[level_index].len() as int);
        let listed_now = ancestors_below_level(levels, containers, level_count);
        assert(listed_now == earlier + here);
        assert forall|listed: int| 0 <= listed < listed_now.len() implies (#[trigger] listed_now[listed]).level <= level_count by {
            if listed >= earlier.len() {
                assert(listed_now[listed] == here[listed - earlier.len()]);
            } else {
                assert(listed_now[listed] == earlier[listed]);
            }
        }
        assert forall|done_level_index: int, node: int|
            #[trigger] node_is_in_the_levels(levels, done_level_index, node) && done_level_index < level_count implies (
            listed_now.contains(position_at(done_level_index, node)) <==> node_is_an_ancestor_of_one_of(
                levels,
                done_level_index,
                node,
                containers,
            )) by {
            lemma_every_level_has_at_most_as_many_nodes_as_containers(levels, done_level_index);
            let position = position_at(done_level_index, node);
            lemma_concatenation_contains(earlier, here, position);
            if done_level_index < level_index {
                if here.contains(position) {
                    let found = choose|found: int| 0 <= found < here.len() && here[found] == position;
                    assert(here[found].level == level_index + 1);
                }
            } else {
                if earlier.contains(position) {
                    let found = choose|found: int| 0 <= found < earlier.len() && earlier[found] == position;
                    assert(earlier[found].level <= level_index);
                }
            }
        }
    }
}

/// bump 次序里的每个位置都是形状里某个节点的位置：层级在 1 到层数之间，序号小于那一层的节点数。
pub proof fn lemma_bump_order_positions_are_nodes(levels: Seq<Seq<usize>>, level_count: int)
    requires
        0 <= level_count <= levels.len(),
        shape_fits_the_position_fields(levels),
        container_tree_shape_is_well_formed(levels),
    ensures
        forall|listed: int|
            0 <= listed < bump_order_positions(levels, level_count).len() ==> {
                let position = #[trigger] bump_order_positions(levels, level_count)[listed];
                &&& 1 <= position.level <= level_count
                &&& position.index_in_level < levels[position.level - 1].len()
            },
    decreases level_count,
{
    if level_count > 0 {
        lemma_bump_order_positions_are_nodes(levels, level_count - 1);
        lemma_every_level_has_at_most_as_many_nodes_as_containers(levels, level_count - 1);
        let earlier = bump_order_positions(levels, level_count - 1);
        let here = positions_of_level(levels, level_count - 1);
        let listed_now = bump_order_positions(levels, level_count);
        assert(listed_now == earlier + here);
        assert forall|listed: int| 0 <= listed < listed_now.len() implies {
            let position = #[trigger] listed_now[listed];
            &&& 1 <= position.level <= level_count
            &&& position.index_in_level < levels[position.level - 1].len()
        } by {
            if listed < earlier.len() {
                assert(listed_now[listed] == earlier[listed]);
            } else {
                assert(listed_now[listed] == here[listed - earlier.len()]);
            }
        }
    }
}

/// 装得进位置两段的形状里，位置的两段就是层级与序号。
proof fn lemma_position_at_fits(levels: Seq<Seq<usize>>, level_index: int, node: int)
    requires
        shape_fits_the_position_fields(levels),
        container_tree_shape_is_well_formed(levels),
        node_is_in_the_levels(levels, level_index, node),
    ensures
        position_at(level_index, node).level == level_index + 1,
        position_at(level_index, node).index_in_level == node,
{
    lemma_every_level_has_at_most_as_many_nodes_as_containers(levels, level_index);
}

/// 一片接一片追加记录，记录依次连起来就是原来的接上这一条。
proof fn lemma_records_of_containers_after_a_push<Record>(containers: Seq<RecordContainer<Record>>, container: RecordContainer<Record>)
    ensures
        records_of_containers(containers.push(container)) == records_of_containers(containers) + container.records@,
{
    assert(containers.push(container).drop_last() =~= containers);
}

/// 最右那片换成记录多了一条的那一份，记录依次连起来就是原来的接上这一条。
proof fn lemma_records_of_containers_after_appending_to_the_last<Record>(
    containers: Seq<RecordContainer<Record>>,
    appended_to: Seq<RecordContainer<Record>>,
    record: Record,
)
    requires
        containers.len() >= 1,
        appended_to.len() == containers.len(),
        appended_to.drop_last() == containers.drop_last(),
        appended_to.last().records@ == containers.last().records@.push(record),
    ensures
        records_of_containers(appended_to) == records_of_containers(containers).push(record),
{
    assert(records_of_containers(containers).push(record) =~= records_of_containers(containers.drop_last())
        + containers.last().records@.push(record));
}

// ===== 形状 =====

/// 一版容器树的形状：每一层每个码 2 节点有几个孩子，自层级 1（孩子是容器）到根（最高那一层，恰好一个节点）。零根时一层都没有。
/// 不变量（类型不变量 [`container_tree_shape_is_well_formed`]）：每个节点至少一个孩子；层级 1 的孩子数之和 = 容器数，层级 ℓ > 1 的孩子数之和 =
/// 层级 ℓ − 1 的节点数；最高那一层只有一个节点；`container_total` 等于容器数。
///
/// 容器数记成一个字段、构造时算好、最右端开一片时加一：问片数（[`Self::containers`]）与求祖先（[`Self::ancestor_suffix_starts_of_the_rightmost_containers`]，
/// 从最右端往左减要知道层级 1 之下一共几片）都在发布路径上，现加层级 1 的孩子数要读层级 1 的每个节点，代价随容器总数涨
/// （`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」运行时决策路径那一格）。
#[derive(Debug, PartialEq, Eq)]
pub struct ContainerTreeShape {
    children_of_each_node_at_each_level: Vec<Vec<usize>>,
    container_total: usize,
}

impl Clone for ContainerTreeShape {
    fn clone(&self) -> (copy: Self)
        ensures
            copy.levels() == self.levels(),
    {
        proof {
            use_type_invariant(self);
        }
        Self {
            children_of_each_node_at_each_level: copy_of_the_children_counts(&self.children_of_each_node_at_each_level),
            container_total: self.container_total,
        }
    }
}

/// 逐层逐个照抄孩子数。
fn copy_of_the_children_counts(children_of_each_node_at_each_level: &[Vec<usize>]) -> (copy: Vec<Vec<usize>>)
    ensures
        levels_of_children_counts(copy@) == levels_of_children_counts(children_of_each_node_at_each_level@),
{
    let mut copy: Vec<Vec<usize>> = Vec::new();
    let mut level_index: usize = 0;
    while level_index < children_of_each_node_at_each_level.len()
        invariant
            level_index <= children_of_each_node_at_each_level.len(),
            copy.len() == level_index,
            forall|copied_level_index: int|
                0 <= copied_level_index < level_index ==> (#[trigger] copy@[copied_level_index])@
                    == children_of_each_node_at_each_level@[copied_level_index]@,
        decreases children_of_each_node_at_each_level.len() - level_index,
    {
        copy.push(copy_of_one_level(&children_of_each_node_at_each_level[level_index]));
        level_index += 1;
    }
    assert(levels_of_children_counts(copy@) =~= levels_of_children_counts(children_of_each_node_at_each_level@));
    copy
}

/// 照抄一层的孩子数。
fn copy_of_one_level(children_of_each_node: &[usize]) -> (copy: Vec<usize>)
    ensures
        copy@ == children_of_each_node@,
{
    let mut copy: Vec<usize> = Vec::new();
    let mut node: usize = 0;
    while node < children_of_each_node.len()
        invariant
            node <= children_of_each_node.len(),
            copy@ == children_of_each_node@.take(node as int),
        decreases children_of_each_node.len() - node,
    {
        copy.push(children_of_each_node[node]);
        node += 1;
        assert(copy@ =~= children_of_each_node@.take(node as int));
    }
    assert(children_of_each_node@.take(children_of_each_node.len() as int) =~= children_of_each_node@);
    copy
}

/// 只有一个孩子数的一层。
fn level_of_a_single_node(children: usize) -> (level: Vec<usize>)
    ensures
        level@ == seq![children],
{
    let level = vec![children];
    assert(level@ =~= seq![children]);
    level
}

/// 一层孩子数之和；越过 `usize` 交回 `None`。
fn checked_children_count_sum(children_of_each_node: &[usize]) -> (sum: Option<usize>)
    ensures
        sum is Some <==> children_count_sum(children_of_each_node@) <= usize::MAX,
        sum matches Some(total) ==> total == children_count_sum(children_of_each_node@),
{
    let mut total: usize = 0;
    let mut node: usize = 0;
    while node < children_of_each_node.len()
        invariant
            node <= children_of_each_node.len(),
            total == children_count_sum(children_of_each_node@.take(node as int)),
        decreases children_of_each_node.len() - node,
    {
        proof {
            lemma_children_count_sum_of_one_more(children_of_each_node@, node as int);
        }
        match total.checked_add(children_of_each_node[node]) {
            Some(next_total) => {
                total = next_total;
            },
            None => {
                proof {
                    lemma_children_count_prefix_sum_is_monotone(children_of_each_node@, node as int + 1, node as int + 1);
                }
                return None;
            },
        }
        node += 1;
    }
    proof {
        lemma_children_count_sum_of_the_whole_level(children_of_each_node@);
    }
    Some(total)
}

/// 每一层至少一个节点、每个节点至少一个孩子。
fn every_node_has_a_child(children_of_each_node_at_each_level: &[Vec<usize>]) -> (every_node_has_a_child: bool)
    ensures
        every_node_has_a_child == every_level_has_a_node_and_every_node_a_child(
            levels_of_children_counts(children_of_each_node_at_each_level@),
        ),
{
    let ghost levels = levels_of_children_counts(children_of_each_node_at_each_level@);
    let mut level_index: usize = 0;
    while level_index < children_of_each_node_at_each_level.len()
        invariant
            levels == levels_of_children_counts(children_of_each_node_at_each_level@),
            levels.len() == children_of_each_node_at_each_level.len(),
            level_index <= levels.len(),
            forall|checked_level_index: int|
                0 <= checked_level_index < level_index ==> (#[trigger] levels[checked_level_index]).len() >= 1,
            forall|checked_level_index: int, node: int|
                0 <= checked_level_index < level_index && 0 <= node < levels[checked_level_index].len()
                    ==> #[trigger] levels[checked_level_index][node] >= 1,
        decreases children_of_each_node_at_each_level.len() - level_index,
    {
        let nodes = &children_of_each_node_at_each_level[level_index];
        proof {
            assert(levels[level_index as int] == nodes@);
        }
        if nodes.is_empty() {
            proof {
                assert(levels[level_index as int].len() == 0);
            }
            return false;
        }
        let mut node: usize = 0;
        while node < nodes.len()
            invariant
                levels == levels_of_children_counts(children_of_each_node_at_each_level@),
                levels.len() == children_of_each_node_at_each_level.len(),
                node <= nodes.len(),
                level_index < levels.len(),
                levels[level_index as int] == nodes@,
                forall|checked_node: int| 0 <= checked_node < node ==> #[trigger] nodes@[checked_node] >= 1,
                forall|checked_level_index: int|
                    0 <= checked_level_index < level_index ==> (#[trigger] levels[checked_level_index]).len() >= 1,
                forall|checked_level_index: int, checked_node: int|
                    0 <= checked_level_index < level_index && 0 <= checked_node < levels[checked_level_index].len()
                        ==> #[trigger] levels[checked_level_index][checked_node] >= 1,
            decreases nodes.len() - node,
        {
            if nodes[node] == 0 {
                proof {
                    assert(levels[level_index as int][node as int] == 0);
                }
                return false;
            }
            node += 1;
        }
        proof {
            assert forall|checked_level_index: int, checked_node: int|
                0 <= checked_level_index < level_index + 1 && 0 <= checked_node < levels[checked_level_index].len()
                    implies #[trigger] levels[checked_level_index][checked_node] >= 1 by {
                if checked_level_index == level_index {
                    assert(nodes@[checked_node] >= 1);
                }
            }
        }
        level_index += 1;
    }
    true
}

/// 层级 ℓ > 1 的孩子数之和都等于层级 ℓ − 1 的节点数（算和越过 `usize` 的那一层算不等）。
fn each_level_counts_the_nodes_below(children_of_each_node_at_each_level: &[Vec<usize>]) -> (counts_the_nodes_below: bool)
    ensures
        counts_the_nodes_below == each_upper_level_counts_the_nodes_below(
            levels_of_children_counts(children_of_each_node_at_each_level@),
        ),
{
    let ghost levels = levels_of_children_counts(children_of_each_node_at_each_level@);
    let mut upper_level_index: usize = 1;
    while upper_level_index < children_of_each_node_at_each_level.len()
        invariant
            levels == levels_of_children_counts(children_of_each_node_at_each_level@),
            levels.len() == children_of_each_node_at_each_level.len(),
            1 <= upper_level_index,
            forall|checked_level_index: int|
                1 <= checked_level_index < upper_level_index && checked_level_index < levels.len() ==> children_count_sum(
                    #[trigger] levels[checked_level_index],
                ) == levels[checked_level_index - 1].len(),
        decreases children_of_each_node_at_each_level.len() - upper_level_index,
    {
        let nodes_below = children_of_each_node_at_each_level[upper_level_index - 1].len();
        proof {
            assert(levels[upper_level_index as int] == children_of_each_node_at_each_level@[upper_level_index as int]@);
            assert(levels[upper_level_index - 1] == children_of_each_node_at_each_level@[upper_level_index - 1]@);
        }
        match checked_children_count_sum(&children_of_each_node_at_each_level[upper_level_index]) {
            Some(sum) => {
                if sum != nodes_below {
                    return false;
                }
            },
            None => {
                return false;
            },
        }
        upper_level_index += 1;
    }
    true
}

impl ContainerTreeShape {
    /// 类型不变量：形状良好（[`container_tree_shape_is_well_formed`]），记着的容器数就是形状里的容器数。
    #[verifier::type_invariant]
    spec fn shape_is_well_formed(self) -> bool {
        container_tree_shape_is_well_formed(self.levels()) && self.container_total == container_count(self.levels())
    }

    /// 每层每个节点的孩子数（规格里用）。
    pub closed spec fn levels(self) -> Seq<Seq<usize>> {
        levels_of_children_counts(self.children_of_each_node_at_each_level@)
    }

    /// 零根（树表条目的根指针置零、一片容器都没有）。
    #[must_use]
    pub fn zero_root() -> (shape: Self)
        ensures
            shape.levels() == Seq::<Seq<usize>>::empty(),
    {
        let shape = Self { children_of_each_node_at_each_level: Vec::new(), container_total: 0 };
        assert(shape.levels() =~= Seq::<Seq<usize>>::empty());
        shape
    }

    /// 从盘上读回来的每层每个节点的孩子数（层级 1 在前）：不满足形状的不变量、层数装不进一字节交回 `None`。
    /// 层级 1 的孩子数之和越过 `usize` 的也交回 `None`（盘上一个节点的条目数是两字节，读不出这样的数）。
    #[must_use]
    pub fn from_children_of_each_node_at_each_level(children_of_each_node_at_each_level: Vec<Vec<usize>>) -> (shape: Option<Self>)
        ensures
            shape is Some <==> (container_tree_shape_is_well_formed(levels_of_children_counts(children_of_each_node_at_each_level@))
                && children_of_each_node_at_each_level.len() <= 255),
            shape matches Some(read_shape) ==> read_shape.levels() == levels_of_children_counts(
                children_of_each_node_at_each_level@,
            ),
    {
        let ghost levels = levels_of_children_counts(children_of_each_node_at_each_level@);
        let every_node_has_a_child = every_node_has_a_child(&children_of_each_node_at_each_level);
        let each_level_counts_the_nodes_below = each_level_counts_the_nodes_below(&children_of_each_node_at_each_level);
        let level_count = children_of_each_node_at_each_level.len();
        let the_top_level_is_one_root = level_count == 0 || children_of_each_node_at_each_level[level_count - 1].len() == 1;
        let levels_fit_one_byte = u8::try_from(level_count).is_ok();
        let container_total_if_it_fits_usize = if level_count == 0 {
            Some(0)
        } else {
            checked_children_count_sum(&children_of_each_node_at_each_level[0])
        };
        proof {
            if level_count > 0 {
                assert(levels[level_count - 1] == children_of_each_node_at_each_level@[level_count - 1]@);
                assert(levels[0] == children_of_each_node_at_each_level@[0]@);
            }
        }
        match container_total_if_it_fits_usize {
            Some(container_total) => {
                if every_node_has_a_child && each_level_counts_the_nodes_below && the_top_level_is_one_root && levels_fit_one_byte {
                    Some(Self { children_of_each_node_at_each_level, container_total })
                } else {
                    None
                }
            },
            None => None,
        }
    }

    /// 根节点码 2 头里的层级：零根 0，根的孩子是容器时 1，长高一次加一。
    ///
    /// # Panics
    /// 层数装不进一字节（`requires` 排除的那一种）：从盘上读来的在构造时判过；在 255 层上再长一层才会到这里。
    #[must_use]
    pub fn root_level(&self) -> (root_level: u8)
        requires
            self.levels().len() <= 255,
        ensures
            root_level == self.levels().len(),
    {
        u8::try_from(self.children_of_each_node_at_each_level.len())
            .expect("形状的层数在构造时判过装得进码 2 头的一字节层级")
    }

    /// 内部层数（根的层级减一，零根与只有根时 0）：记账标签 6 的 H（D3（空间分配） 已定项 2）与 ckpt_cost 谱系那一项（D28（挂载期承诺量） 已定项 4）读它。
    #[must_use]
    pub fn internal_levels(&self) -> (internal_levels: u64)
        requires
            self.levels().len() <= 255,
        ensures
            internal_levels == if self.levels().len() == 0 { 0 } else { self.levels().len() - 1 },
    {
        u64::from(self.root_level()).saturating_sub(1)
    }

    /// 这棵树有几片容器：读记着的那个数，不逐个加层级 1 的孩子数。
    #[must_use]
    pub fn containers(&self) -> (containers: usize)
        ensures
            containers == container_count(self.levels()),
    {
        proof {
            use_type_invariant(self);
        }
        self.container_total
    }

    /// 层级 `level` 上有几个节点（层级 0 是容器、超过根的层级 0 个）。
    #[must_use]
    pub fn nodes_at_level(&self, level: u8) -> (nodes: usize)
        ensures
            nodes == if level == 0 {
                container_count(self.levels())
            } else if level <= self.levels().len() {
                self.levels()[level - 1].len() as int
            } else {
                0int
            },
    {
        match level {
            0 => self.containers(),
            _ => {
                let level_index = usize::from(level) - 1;
                if level_index < self.children_of_each_node_at_each_level.len() {
                    self.children_of_each_node_at_each_level[level_index].len()
                } else {
                    0
                }
            },
        }
    }

    /// 全部码 2 节点的位置，按 bump 次序（层级升序、同层从左到右，根最末）。零根时空。
    #[must_use]
    pub fn positions_in_bump_order(&self) -> (positions: Vec<ContainerTreeNodePosition>)
        requires
            shape_fits_the_position_fields(self.levels()),
        ensures
            positions@ == bump_order_positions(self.levels(), self.levels().len() as int),
    {
        proof {
            use_type_invariant(self);
        }
        let ghost levels = self.levels();
        let mut positions: Vec<ContainerTreeNodePosition> = Vec::new();
        let mut level_index: usize = 0;
        while level_index < self.children_of_each_node_at_each_level.len()
            invariant
                levels == self.levels(),
                container_tree_shape_is_well_formed(levels),
                shape_fits_the_position_fields(levels),
                level_index <= levels.len(),
                positions@ == bump_order_positions(levels, level_index as int),
            decreases self.children_of_each_node_at_each_level.len() - level_index,
        {
            let level = u8::try_from(level_index + 1).expect("层数在构造时判过装得进一字节");
            let nodes = &self.children_of_each_node_at_each_level[level_index];
            proof {
                lemma_every_level_has_at_most_as_many_nodes_as_containers(levels, level_index as int);
                assert(nodes@ == levels[level_index as int]);
            }
            let ghost positions_below = positions@;
            let mut node: usize = 0;
            while node < nodes.len()
                invariant
                    nodes@ == levels[level_index as int],
                    level_index < levels.len(),
                    level == level_index + 1,
                    nodes@.len() <= u32::MAX,
                    node <= nodes.len(),
                    positions_below == bump_order_positions(levels, level_index as int),
                    positions@ == positions_below + positions_of_level(levels, level_index as int).take(node as int),
                decreases nodes.len() - node,
            {
                let position = ContainerTreeNodePosition {
                    level,
                    index_in_level: u32::try_from(node).expect("同层节点数装得进 u32"),
                };
                proof {
                    assert(position == position_at(level_index as int, node as int));
                }
                positions.push(position);
                node += 1;
                proof {
                    assert(positions@ =~= positions_below + positions_of_level(levels, level_index as int).take(node as int));
                }
            }
            proof {
                assert(positions_of_level(levels, level_index as int).take(nodes.len() as int) =~= positions_of_level(
                    levels,
                    level_index as int,
                ));
            }
            level_index += 1;
        }
        positions
    }

    /// 根的位置（最高那一层的第 0 个）；零根时 `None`。
    #[must_use]
    pub fn root_position(&self) -> (root_position: Option<ContainerTreeNodePosition>)
        requires
            self.levels().len() <= 255,
        ensures
            self.levels().len() == 0 ==> root_position is None,
            self.levels().len() > 0 ==> root_position == Some(position_at(self.levels().len() - 1, 0)),
    {
        if self.root_level() > 0 {
            Some(ContainerTreeNodePosition { level: self.root_level(), index_in_level: 0 })
        } else {
            None
        }
    }

    /// 这个位置在这一版里有没有节点。
    #[must_use]
    pub fn has_a_node_at(&self, position: ContainerTreeNodePosition) -> (has_a_node: bool)
        ensures
            has_a_node == position_is_in_the_shape(self.levels(), position),
    {
        position.level >= 1 && match usize::try_from(position.index_in_level) {
            Ok(index) => index < self.nodes_at_level(position.level),
            Err(_) => false,
        }
    }

    /// 这个位置上的节点是不是这一版的根。
    #[must_use]
    pub fn is_the_root(&self, position: ContainerTreeNodePosition) -> (is_the_root: bool)
        requires
            self.levels().len() <= 255,
        ensures
            is_the_root == (self.levels().len() > 0 && position == position_at(self.levels().len() - 1, 0)),
    {
        match self.root_position() {
            Some(root) => root.level == position.level && root.index_in_level == position.index_in_level,
            None => false,
        }
    }

    /// 每一层每个节点的孩子在下一层（层级 1 的孩子是容器，按叶序）里的下标区间：外层按层下标（层级 1 在前），里层同层从左到右。
    /// 每层从最左那个节点起顺着累加孩子数，一层只走一遍：装一棵容器树这一版的单元时每个节点都要它（`crate::transaction` 的
    /// `build_container_tree_units`），一个一个问、每个都从这一层最左加起，整层就是这一层节点数的平方。
    #[must_use]
    pub fn children_of_every_node(&self) -> (children: Vec<Vec<Range<usize>>>)
        ensures
            children@.len() == self.levels().len(),
            forall|level_index: int|
                0 <= level_index < children@.len() ==> (#[trigger] children@[level_index])@.len() == self.levels()[level_index].len(),
            forall|level_index: int, node: int|
                #[trigger] node_is_in_the_levels(self.levels(), level_index, node) ==> {
                    let range = children@[level_index]@[node];
                    &&& range.start == children_count_sum(self.levels()[level_index].take(node))
                    &&& range.end == range.start + self.levels()[level_index][node]
                },
    {
        proof {
            use_type_invariant(self);
        }
        let ghost levels = self.levels();
        let mut children: Vec<Vec<Range<usize>>> = Vec::new();
        let mut level_index: usize = 0;
        while level_index < self.children_of_each_node_at_each_level.len()
            invariant
                levels == self.levels(),
                container_tree_shape_is_well_formed(levels),
                level_index <= levels.len(),
                children@.len() == level_index,
                forall|done_level_index: int|
                    0 <= done_level_index < level_index ==> (#[trigger] children@[done_level_index])@.len()
                        == levels[done_level_index].len(),
                forall|done_level_index: int, node: int|
                    0 <= done_level_index < level_index && #[trigger] node_is_in_the_levels(levels, done_level_index, node) ==> {
                        let range = children@[done_level_index]@[node];
                        &&& range.start == children_count_sum(levels[done_level_index].take(node))
                        &&& range.end == range.start + levels[done_level_index][node]
                    },
            decreases self.children_of_each_node_at_each_level.len() - level_index,
        {
            let nodes = &self.children_of_each_node_at_each_level[level_index];
            proof {
                assert(nodes@ == levels[level_index as int]);
                if level_index == 0 {
                    assert(children_count_sum(nodes@) == container_count(levels));
                } else {
                    assert(children_count_sum(levels[level_index as int]) == levels[level_index - 1].len());
                    assert(levels[level_index - 1].len() == self.children_of_each_node_at_each_level@[level_index - 1].len());
                }
                assert(nodes@.take(0) =~= Seq::<usize>::empty());
            }
            // 迭代上界是这一层的节点数；跨轮携带的是下一个节点的头一个孩子（这一层前几个节点的孩子数之和）。
            let mut ranges: Vec<Range<usize>> = Vec::new();
            let mut first_child: usize = 0;
            let mut node: usize = 0;
            while node < nodes.len()
                invariant
                    nodes@ == levels[level_index as int],
                    children_count_sum(nodes@) <= usize::MAX,
                    node <= nodes.len(),
                    first_child == children_count_sum(nodes@.take(node as int)),
                    ranges@.len() == node,
                    forall|done_node: int|
                        0 <= done_node < node ==> {
                            let range = #[trigger] ranges@[done_node];
                            &&& range.start == children_count_sum(nodes@.take(done_node))
                            &&& range.end == range.start + nodes@[done_node]
                        },
                decreases nodes.len() - node,
            {
                proof {
                    lemma_children_count_sum_of_one_more(nodes@, node as int);
                    lemma_children_count_prefix_sum_is_monotone(nodes@, node as int + 1, node as int + 1);
                }
                let children_end = first_child + nodes[node];
                ranges.push(first_child..children_end);
                first_child = children_end;
                node += 1;
            }
            let ghost children_before = children@;
            children.push(ranges);
            proof {
                assert forall|done_level_index: int, done_node: int|
                    0 <= done_level_index < level_index + 1 && #[trigger] node_is_in_the_levels(levels, done_level_index, done_node)
                        implies {
                        let range = children@[done_level_index]@[done_node];
                        &&& range.start == children_count_sum(levels[done_level_index].take(done_node))
                        &&& range.end == range.start + levels[done_level_index][done_node]
                    } by {
                    if done_level_index < level_index {
                        assert(children@[done_level_index] == children_before[done_level_index]);
                    }
                }
            }
            level_index += 1;
        }
        children
    }

    /// 每一层有几个节点：这次发布把上一版的形状记成这几个数（[`ContainerTreeNodeCounts`]），不照抄每个节点的孩子数。
    #[must_use]
    pub fn node_counts(&self) -> (node_counts: ContainerTreeNodeCounts)
        ensures
            node_counts.counts() == node_counts_of(self.levels()),
    {
        let ghost levels = self.levels();
        let mut nodes_at_each_level: Vec<usize> = Vec::new();
        let mut level_index: usize = 0;
        while level_index < self.children_of_each_node_at_each_level.len()
            invariant
                levels == self.levels(),
                levels.len() == self.children_of_each_node_at_each_level@.len(),
                level_index <= levels.len(),
                nodes_at_each_level@.len() == level_index,
                forall|done_level_index: int|
                    0 <= done_level_index < level_index ==> #[trigger] nodes_at_each_level@[done_level_index] as int
                        == levels[done_level_index].len(),
            decreases self.children_of_each_node_at_each_level.len() - level_index,
        {
            proof {
                assert(levels[level_index as int] == self.children_of_each_node_at_each_level@[level_index as int]@);
            }
            nodes_at_each_level.push(self.children_of_each_node_at_each_level[level_index].len());
            level_index += 1;
        }
        let node_counts = ContainerTreeNodeCounts { nodes_at_each_level };
        proof {
            assert(node_counts.counts() =~= node_counts_of(levels));
        }
        node_counts
    }

    /// 最右端那 `rightmost_count` 片容器在每一层的祖先：每一层都是这一层最右端的一段（一段后缀），交回每层那一段的起点（层下标 0 在前）。
    /// 一层一层往上，每层被标过的是下一层从某一个起往右的全部（层级 1 之下就是这几片容器）：从这一层最右端往左逐个减去孩子数，减到不大于
    /// 被标过的头一个就停——停的那个节点罩住它，往右的每个节点都罩着被标过的，往左的一个都不罩——这一层的那一段就从停的那个节点起，
    /// 上一层被标过的也就从它起。每层只读最右端到那一段起点的几个节点：给的片数是 k、扇出是 f 时每层不超过 ⌈k ÷ f⌉ + 1 个，代价只随
    /// 给的片数与层数涨、不随容器总数涨（它在发布路径上，运行时决策路径，`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」）。
    /// 发布里重写的就是最右端连着的几片（[`rewritten_containers_are_the_rightmost`]），所以只收这样的几片、不收散在左边的。
    ///
    /// # Panics
    /// 给的片数为 0 或多于容器数（`requires` 排除的那一种）：调用方只拿这一版里重写了的几片问。
    #[must_use]
    pub fn ancestor_suffix_starts_of_the_rightmost_containers(&self, rightmost_count: usize) -> (starts: Vec<usize>)
        requires
            1 <= rightmost_count <= container_count(self.levels()),
        ensures
            starts@.len() == self.levels().len(),
            forall|level_index: int| 0 <= level_index < starts@.len() ==> #[trigger] starts@[level_index] < self.levels()[level_index].len(),
            forall|level_index: int, node: int|
                #[trigger] node_is_in_the_levels(self.levels(), level_index, node) ==> (node_is_an_ancestor_of_one_of(
                    self.levels(),
                    level_index,
                    node,
                    rightmost_containers(container_count(self.levels()), rightmost_count as int),
                ) <==> node >= starts@[level_index]),
    {
        proof {
            use_type_invariant(self);
        }
        let ghost levels = self.levels();
        let ghost containers = rightmost_containers(container_count(levels), rightmost_count as int);
        let container_total = self.containers();
        assert!(
            1 <= rightmost_count && rightmost_count <= container_total,
            "问祖先的是这一版最右端的 {rightmost_count} 片（{container_total} 片里）"
        );
        let mut first_marked: usize = container_total - rightmost_count;
        proof {
            assert forall|child: int| 0 <= child < container_total implies (#[trigger] child_is_marked(levels, 0, child, containers)
                <==> child >= first_marked) by {
                lemma_rightmost_containers_hold_exactly_the_ones_from_their_first(
                    container_count(levels),
                    rightmost_count as int,
                    child,
                );
            }
            assert forall|listed: int| 0 <= listed < containers.len() implies #[trigger] containers[listed] < container_count(levels) by {}
        }
        let mut starts: Vec<usize> = Vec::new();
        let mut width: usize = container_total;
        let mut level_index: usize = 0;
        while level_index < self.children_of_each_node_at_each_level.len()
            invariant
                levels == self.levels(),
                container_tree_shape_is_well_formed(levels),
                containers == rightmost_containers(container_count(levels), rightmost_count as int),
                forall|listed: int| 0 <= listed < containers.len() ==> #[trigger] containers[listed] < container_count(levels),
                level_index <= levels.len(),
                width == if level_index == 0 {
                    container_count(levels)
                } else {
                    levels[level_index - 1].len() as int
                },
                first_marked < width,
                forall|child: int|
                    0 <= child < width ==> (#[trigger] child_is_marked(levels, level_index as int, child, containers) <==> child
                        >= first_marked),
                starts@.len() == level_index,
                forall|done_level_index: int|
                    0 <= done_level_index < level_index ==> #[trigger] starts@[done_level_index] < levels[done_level_index].len(),
                forall|done_level_index: int, node: int|
                    0 <= done_level_index < level_index && #[trigger] node_is_in_the_levels(levels, done_level_index, node) ==> (
                    node_is_an_ancestor_of_one_of(levels, done_level_index, node, containers) <==> node >= starts@[done_level_index]),
            decreases self.children_of_each_node_at_each_level.len() - level_index,
        {
            let nodes = &self.children_of_each_node_at_each_level[level_index];
            proof {
                assert(nodes@ == levels[level_index as int]);
                if level_index == 0 {
                    assert(children_count_sum(nodes@) == width);
                } else {
                    assert(children_count_sum(levels[level_index as int]) == levels[level_index - 1].len());
                }
                assert(nodes@.take(nodes@.len() as int) =~= nodes@);
            }
            // 从最右端往左逐个减去孩子数，减到不大于被标过的头一个就停（迭代上界是这一层的节点数）。
            let mut first_scanned: usize = nodes.len();
            let mut first_child: usize = width;
            while first_child > first_marked
                invariant
                    nodes@ == levels[level_index as int],
                    children_count_sum(nodes@) == width,
                    first_marked < width,
                    first_scanned <= nodes@.len(),
                    first_child == children_count_sum(nodes@.take(first_scanned as int)),
                    first_scanned == nodes@.len() || first_marked < children_count_sum(nodes@.take(first_scanned as int + 1)),
                decreases first_scanned,
            {
                proof {
                    assert(nodes@.take(0) =~= Seq::<usize>::empty());
                    assert(first_scanned > 0);
                    lemma_children_count_sum_of_one_more(nodes@, first_scanned as int - 1);
                    lemma_children_count_sum_is_not_negative(nodes@.take(first_scanned as int - 1));
                }
                first_scanned -= 1;
                first_child -= nodes[first_scanned];
            }
            proof {
                assert(first_scanned < nodes@.len());
                assert forall|unscanned: int| 0 <= unscanned < first_scanned implies !(#[trigger] node_is_an_ancestor_of_one_of(
                    levels,
                    level_index as int,
                    unscanned,
                    containers,
                )) by {
                    lemma_node_is_marked_by_its_children(levels, level_index as int, unscanned, containers);
                    if node_is_an_ancestor_of_one_of(levels, level_index as int, unscanned, containers) {
                        let child = choose|child: int|
                            child_lies_under_node(levels[level_index as int], unscanned, child) && #[trigger] child_is_marked(
                                levels,
                                level_index as int,
                                child,
                                containers,
                            );
                        lemma_children_count_prefix_sum_is_monotone(nodes@, unscanned + 1, first_scanned as int);
                        lemma_children_count_sum_is_not_negative(nodes@.take(unscanned));
                        assert(0 <= child < width);
                        assert(child >= first_marked);
                    }
                }
                assert forall|scanned: int| first_scanned <= scanned < nodes@.len() implies #[trigger] node_is_an_ancestor_of_one_of(
                    levels,
                    level_index as int,
                    scanned,
                    containers,
                ) by {
                    lemma_node_is_marked_by_its_children(levels, level_index as int, scanned, containers);
                    lemma_children_count_sum_of_one_more(nodes@, scanned);
                    lemma_children_count_prefix_sum_is_monotone(nodes@, first_scanned as int + 1, scanned + 1);
                    lemma_children_count_prefix_sum_is_monotone(nodes@, scanned + 1, nodes@.len() as int);
                    assert(levels[level_index as int][scanned] >= 1);
                    let child = children_count_sum(nodes@.take(scanned + 1)) - 1;
                    assert(child_lies_under_node(levels[level_index as int], scanned, child));
                    assert(0 <= child < width);
                    assert(child >= first_marked);
                    assert(child_is_marked(levels, level_index as int, child, containers));
                }
            }
            let ghost starts_before = starts@;
            starts.push(first_scanned);
            proof {
                assert forall|done_level_index: int, node: int|
                    0 <= done_level_index < level_index + 1 && #[trigger] node_is_in_the_levels(levels, done_level_index, node) implies (
                    node_is_an_ancestor_of_one_of(levels, done_level_index, node, containers) <==> node >= starts@[done_level_index]) by {
                    if done_level_index < level_index {
                        assert(starts@[done_level_index] == starts_before[done_level_index]);
                    }
                }
                assert forall|done_level_index: int| 0 <= done_level_index < level_index + 1 implies #[trigger] starts@[done_level_index]
                    < levels[done_level_index].len() by {
                    if done_level_index < level_index {
                        assert(starts@[done_level_index] == starts_before[done_level_index]);
                    }
                }
            }
            first_marked = first_scanned;
            width = nodes.len();
            level_index += 1;
            proof {
                assert forall|child: int| 0 <= child < width implies (#[trigger] child_is_marked(
                    levels,
                    level_index as int,
                    child,
                    containers,
                ) <==> child >= first_marked) by {
                    assert(node_is_in_the_levels(levels, level_index as int - 1, child));
                }
            }
        }
        starts
    }

    /// 最右端那 `rightmost_count` 片容器的祖先的并，按 bump 次序（层级升序、每层那一段从左到右，根最末）：这几片被重写时 COW 的就是它们
    /// （叶 + 全部祖先）。只有最右那一片时就是最右那一条路径（每一层最右那一个节点）。每层那一段是一段后缀
    /// （[`Self::ancestor_suffix_starts_of_the_rightmost_containers`]），读的节点只有交回的这几个。
    /// 「在这张单子里就是某一片的祖先」由 [`lemma_ancestors_below_level_contains`] 推。
    ///
    /// # Panics
    /// 给的片数为 0 或多于容器数（`requires` 排除的那一种）。
    #[must_use]
    pub fn ancestors_of_the_rightmost_containers(&self, rightmost_count: usize) -> (ancestors: Vec<ContainerTreeNodePosition>)
        requires
            shape_fits_the_position_fields(self.levels()),
            1 <= rightmost_count <= container_count(self.levels()),
        ensures
            ancestors@ == ancestors_below_level(
                self.levels(),
                rightmost_containers(container_count(self.levels()), rightmost_count as int),
                self.levels().len() as int,
            ),
    {
        proof {
            use_type_invariant(self);
        }
        let ghost levels = self.levels();
        let ghost containers = rightmost_containers(container_count(levels), rightmost_count as int);
        let starts = self.ancestor_suffix_starts_of_the_rightmost_containers(rightmost_count);
        let mut ancestors: Vec<ContainerTreeNodePosition> = Vec::new();
        let mut level_index: usize = 0;
        while level_index < self.children_of_each_node_at_each_level.len()
            invariant
                levels == self.levels(),
                container_tree_shape_is_well_formed(levels),
                shape_fits_the_position_fields(levels),
                containers == rightmost_containers(container_count(levels), rightmost_count as int),
                starts@.len() == levels.len(),
                forall|any_level_index: int|
                    0 <= any_level_index < starts@.len() ==> #[trigger] starts@[any_level_index] < levels[any_level_index].len(),
                forall|any_level_index: int, node: int|
                    #[trigger] node_is_in_the_levels(levels, any_level_index, node) ==> (node_is_an_ancestor_of_one_of(
                        levels,
                        any_level_index,
                        node,
                        containers,
                    ) <==> node >= starts@[any_level_index]),
                level_index <= levels.len(),
                ancestors@ == ancestors_below_level(levels, containers, level_index as int),
            decreases self.children_of_each_node_at_each_level.len() - level_index,
        {
            let level = u8::try_from(level_index + 1).expect("层数在构造时判过装得进一字节");
            let node_count = self.children_of_each_node_at_each_level[level_index].len();
            let first_ancestor = starts[level_index];
            proof {
                lemma_every_level_has_at_most_as_many_nodes_as_containers(levels, level_index as int);
                assert(node_count == levels[level_index as int].len());
                assert forall|unlisted: int| 0 <= unlisted < first_ancestor implies !(#[trigger] node_is_an_ancestor_of_one_of(
                    levels,
                    level_index as int,
                    unlisted,
                    containers,
                )) by {
                    assert(node_is_in_the_levels(levels, level_index as int, unlisted));
                }
                lemma_no_ancestor_among_the_first(levels, level_index as int, containers, first_ancestor as int);
            }
            let ghost ancestors_below = ancestors@;
            proof {
                assert(ancestors@ =~= ancestors_below + ancestors_in_level(levels, level_index as int, containers, first_ancestor as int));
            }
            let mut node: usize = first_ancestor;
            while node < node_count
                invariant
                    levels == self.levels(),
                    shape_fits_the_position_fields(levels),
                    container_tree_shape_is_well_formed(levels),
                    level_index < levels.len(),
                    level == level_index + 1,
                    node_count == levels[level_index as int].len(),
                    node_count <= u32::MAX,
                    first_ancestor <= node <= node_count,
                    forall|any_node: int|
                        #[trigger] node_is_in_the_levels(levels, level_index as int, any_node) ==> (node_is_an_ancestor_of_one_of(
                            levels,
                            level_index as int,
                            any_node,
                            containers,
                        ) <==> any_node >= first_ancestor),
                    ancestors@ == ancestors_below + ancestors_in_level(levels, level_index as int, containers, node as int),
                decreases node_count - node,
            {
                let position = ContainerTreeNodePosition {
                    level,
                    index_in_level: u32::try_from(node).expect("同层节点数装得进 u32"),
                };
                proof {
                    assert(position == position_at(level_index as int, node as int));
                    assert(node_is_in_the_levels(levels, level_index as int, node as int));
                    assert(node_is_an_ancestor_of_one_of(levels, level_index as int, node as int, containers));
                }
                ancestors.push(position);
                node += 1;
                proof {
                    assert(ancestors@ =~= ancestors_below + ancestors_in_level(levels, level_index as int, containers, node as int));
                }
            }
            level_index += 1;
        }
        ancestors
    }

    /// 最右端新开一片容器之后的形状（末尾分裂、根满长高，D8（核心索引结构） 已定项 6）：最右那个层级 1 节点装得下（孩子少于
    /// `entries_per_node`）就挂进去；装满了就在这一层最右端新开一个只有它一个孩子的节点，再往上一层给最右那个节点添这个新孩子，
    /// 一层层往上；最高那一层（根）也满了就再长一层，新根两个孩子（旧根、新开的右半）。零根时第一片容器开一个层级 1 的根。
    /// 左边的节点一个都不动。
    ///
    /// 先从层级 1 往上找停的那一层（最右节点装得下的第一层，都满就停在根那一层），再只改停的那一层与它之下各层的最右端：之下各层末尾各加一个
    /// 只有一个孩子的节点，停的那一层装得下就给最右节点加一个孩子、装不下就也加一个节点并再长一层两个孩子的新根。形状按值交进来、原地改：
    /// 它在发布路径上（运行时决策路径，`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」：代价不许随盘容量增长），
    /// 每开一片只碰停的那一层及其之下各层的末尾，不逐层照抄全部节点，代价只随层数涨。
    ///
    /// # Panics
    /// `entries_per_node` 小于 2（新根装不下两个孩子）：开关的构造函数判过。
    #[must_use]
    pub fn after_a_container_is_opened_at_the_right_end(self, entries_per_node: usize) -> (opened: Self)
        requires
            entries_per_node >= 2,
            container_count(self.levels()) + 1 <= usize::MAX,
        ensures
            opened.levels() == levels_after_opening_at_the_right_end(self.levels(), entries_per_node as int),
            container_count(opened.levels()) == container_count(self.levels()) + 1,
    {
        proof {
            use_type_invariant(&self);
            lemma_opening_keeps_the_shape_well_formed(self.levels(), entries_per_node as int);
        }
        assert!(
            entries_per_node >= 2,
            "容器树一个码 2 节点至少装 2 条：长高那次的新根要装得下左右两半"
        );
        let ghost levels = self.levels();
        let Self { children_of_each_node_at_each_level: levels_before_opening, container_total: containers_before_opening } = self;
        let container_total = containers_before_opening;
        let mut levels_after = levels_before_opening;
        let level_count = levels_after.len();
        if level_count == 0 {
            // 零根：第一片容器开一个层级 1 的根。
            levels_after.push(level_of_a_single_node(1));
            proof {
                assert(levels_of_children_counts(levels_after@) =~= levels_after_opening_at_the_right_end(
                    levels,
                    entries_per_node as int,
                ));
            }
            return Self { children_of_each_node_at_each_level: levels_after, container_total: container_total + 1 };
        }
        proof {
            lemma_opening_stop_level(levels, entries_per_node as int, 0);
            assert(levels[0] == levels_after@[0]@);
        }
        // 迭代上界是层数：每一轮往上一层，到最右节点装得下的那一层、或最高那一层停。
        let mut stop: usize = 0;
        while stop + 1 < level_count && levels_after[stop][levels_after[stop].len() - 1] >= entries_per_node
            invariant
                levels == levels_of_children_counts(levels_after@),
                container_tree_shape_is_well_formed(levels),
                level_count == levels.len(),
                stop < level_count,
                opening_stop_level_from(levels, entries_per_node as int, stop as int) == opening_stop_level_from(
                    levels,
                    entries_per_node as int,
                    0,
                ),
            decreases level_count - stop,
        {
            proof {
                assert(levels[stop as int] == levels_after@[stop as int]@);
            }
            stop += 1;
            proof {
                assert(levels[stop as int] == levels_after@[stop as int]@);
            }
        }
        proof {
            assert(levels[stop as int] == levels_after@[stop as int]@);
            assert(levels[stop as int].len() >= 1);
        }
        let grows = levels_after[stop][levels_after[stop].len() - 1] >= entries_per_node;
        // 只改停的那一层与它之下各层：迭代上界是 stop + 1（不超过层数），每一轮只碰这一层的最右端；停的那一层之上原样留着。
        let mut level_index: usize = 0;
        while level_index <= stop
            invariant
                container_tree_shape_is_well_formed(levels),
                level_count == levels.len(),
                levels_after.len() == level_count,
                stop == opening_stop_level_from(levels, entries_per_node as int, 0),
                stop < level_count,
                levels[stop as int].len() >= 1,
                grows == (levels[stop as int].last() >= entries_per_node),
                level_index <= stop + 1,
                forall|done_level_index: int|
                    0 <= done_level_index < level_index ==> (#[trigger] levels_after@[done_level_index])@ == level_after_opening(
                        levels,
                        stop as int,
                        grows,
                        done_level_index,
                    ),
                forall|untouched_level_index: int|
                    level_index <= untouched_level_index < level_count ==> (#[trigger] levels_after@[untouched_level_index])@
                        == levels[untouched_level_index],
            decreases stop + 1 - level_index,
        {
            proof {
                assert(levels_after@[level_index as int]@ == levels[level_index as int]);
                assert(levels[level_index as int].len() >= 1);
            }
            if level_index < stop || grows {
                levels_after[level_index].push(1);
            } else {
                let rightmost = levels_after[level_index].len() - 1;
                proof {
                    assert(levels_after@[level_index as int]@[rightmost as int] < entries_per_node);
                }
                levels_after[level_index][rightmost] += 1;
            }
            proof {
                assert(levels_after@[level_index as int]@ =~= level_after_opening(levels, stop as int, grows, level_index as int));
            }
            level_index += 1;
        }
        proof {
            assert forall|untouched_level_index: int| stop < untouched_level_index < level_count implies (#[trigger] levels_after@[
                untouched_level_index
            ])@ == level_after_opening(levels, stop as int, grows, untouched_level_index) by {}
        }
        if grows {
            // 根也满了：长一层，新根挂两个孩子（旧根、刚在最右端新开的右半）。
            levels_after.push(level_of_a_single_node(2));
        }
        proof {
            assert(levels_of_children_counts(levels_after@) =~= levels_after_opening_at_the_right_end(
                levels,
                entries_per_node as int,
            ));
        }
        Self { children_of_each_node_at_each_level: levels_after, container_total: container_total + 1 }
    }
}

/// 一版容器树每一层有几个码 2 节点（层级 1 在前、根那一层在最后；零根时一层都没有）。这次发布对上一版的形状只问四样：某个位置上
/// 有没有节点、它是不是根、根在哪、根的层级（[`ContainerTreeAfterThisPublish::node_origins`]、
/// [`ContainerTreeAfterThisPublish::grew_a_level_this_publish`]、`crate::transaction` 按上一版的角色认照抄的节点），四样都只看层数与
/// 每层节点数，所以上一版只记这几个数，不照抄每个节点的孩子数：它在发布路径上（运行时决策路径，`.claude/rules/fs-design.md`
/// 「记账是事务的副产品，不是事后的遍历」），大小只随层数涨、不随节点总数涨。
#[derive(Clone, Debug, PartialEq, Eq)]
#[verifier::allow(autoderive_clone_without_spec)]
pub struct ContainerTreeNodeCounts {
    nodes_at_each_level: Vec<usize>,
}

impl ContainerTreeNodeCounts {
    /// 每层的节点数（规格里用）。
    pub closed spec fn counts(self) -> Seq<int> {
        Seq::new(self.nodes_at_each_level@.len(), |level_index: int| self.nodes_at_each_level@[level_index] as int)
    }

    /// 根节点码 2 头里的层级：零根 0，根的孩子是容器时 1，长高一次加一（层数）。
    ///
    /// # Panics
    /// 层数装不进一字节（`requires` 排除的那一种）：出自 [`ContainerTreeShape::node_counts`]，那份形状构造时判过。
    #[must_use]
    pub fn root_level(&self) -> (root_level: u8)
        requires
            self.counts().len() <= 255,
        ensures
            root_level == self.counts().len(),
    {
        u8::try_from(self.nodes_at_each_level.len()).expect("上一版形状的层数在那份形状构造时判过装得进码 2 头的一字节层级")
    }

    /// 根的位置（最高那一层的第 0 个）；零根时 `None`。
    #[must_use]
    pub fn root_position(&self) -> (root_position: Option<ContainerTreeNodePosition>)
        requires
            self.counts().len() <= 255,
        ensures
            self.counts().len() == 0 ==> root_position is None,
            self.counts().len() > 0 ==> root_position == Some(position_at(self.counts().len() - 1, 0)),
    {
        match self.root_level() {
            0 => None,
            root_level => Some(ContainerTreeNodePosition { level: root_level, index_in_level: 0 }),
        }
    }

    /// 这个位置上的节点是不是那一版的根。
    #[must_use]
    pub fn is_the_root(&self, position: ContainerTreeNodePosition) -> (is_the_root: bool)
        requires
            self.counts().len() <= 255,
        ensures
            is_the_root == (self.counts().len() > 0 && position == position_at(self.counts().len() - 1, 0)),
    {
        match self.root_position() {
            Some(root_of_that_version) => position.level == root_of_that_version.level && position.index_in_level
                == root_of_that_version.index_in_level,
            None => false,
        }
    }

    /// 这个位置在那一版里有没有节点。
    #[must_use]
    pub fn has_a_node_at(&self, position: ContainerTreeNodePosition) -> (has_a_node: bool)
        ensures
            has_a_node == position_is_counted(self.counts(), position),
    {
        let level = usize::from(position.level);
        if level < 1 || level > self.nodes_at_each_level.len() {
            return false;
        }
        match usize::try_from(position.index_in_level) {
            Ok(index) => index < self.nodes_at_each_level[level - 1],
            Err(_) => false,
        }
    }
}

/// 这个位置在不在这张单子里。
fn position_is_listed(positions: &[ContainerTreeNodePosition], position: ContainerTreeNodePosition) -> (is_listed: bool)
    ensures
        is_listed == positions@.contains(position),
{
    let mut listed: usize = 0;
    while listed < positions.len()
        invariant
            listed <= positions.len(),
            forall|checked: int| 0 <= checked < listed ==> #[trigger] positions@[checked] != position,
        decreases positions.len() - listed,
    {
        if positions[listed].level == position.level && positions[listed].index_in_level == position.index_in_level {
            proof {
                assert(positions@[listed as int] == position);
            }
            return true;
        }
        listed += 1;
    }
    false
}

/// 一版容器树里一个码 2 节点从哪来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerTreeNodeOrigin {
    RewrittenThisPublish,
    /// 上一版同一个位置上的那个节点，一个字节不动（长高那次的旧根也是这一种：位置不变、只是不再是根）。
    CarriedFromThePreviousVersion,
}

/// 这次发布之后一棵容器树的样子：叶容器（按叶序）、这次重写哪几片（叶序）、这次之后的形状与这次之前那一版每层的节点数。
/// 码 2 节点重写哪几个由它们推：有容器重写时是重写的那几片的全部祖先（COW 叶 + 祖先），没有时一个都不重写。
/// 这次之前那一版只记每层节点数（[`ContainerTreeNodeCounts`]）：发布对它只问哪个位置上有节点、哪个是根，不照抄它的形状。
#[derive(Clone, Debug, PartialEq, Eq)]
#[verifier::allow(autoderive_clone_without_spec)]
pub struct ContainerTreeAfterThisPublish<Record> {
    pub containers: Vec<RecordContainer<Record>>,
    pub rewritten_containers: Vec<usize>,
    pub shape: ContainerTreeShape,
    pub node_counts_before_this_publish: ContainerTreeNodeCounts,
}

/// 两组容器逐片相同：身份相同、记录逐条相同。
pub open spec fn containers_are_the_same<Record>(
    earlier: Seq<RecordContainer<Record>>,
    later: Seq<RecordContainer<Record>>,
) -> bool {
    &&& earlier.len() == later.len()
    &&& forall|position: int|
        0 <= position < earlier.len() ==> (#[trigger] later[position]).identity == earlier[position].identity
            && later[position].records@ == earlier[position].records@
}

/// 照抄一片容器的记录。
fn copy_of_the_records<Record: Copy>(records: &[Record]) -> (copy: Vec<Record>)
    ensures
        copy@ == records@,
{
    let mut copy: Vec<Record> = Vec::new();
    let mut position: usize = 0;
    while position < records.len()
        invariant
            position <= records.len(),
            copy@ == records@.take(position as int),
        decreases records.len() - position,
    {
        copy.push(records[position]);
        position += 1;
        assert(copy@ =~= records@.take(position as int));
    }
    assert(records@.take(records.len() as int) =~= records@);
    copy
}

/// 逐片照抄容器（记录逐条照抄）。
fn copy_of_the_containers<Record: Copy>(containers: &[RecordContainer<Record>]) -> (copy: Vec<RecordContainer<Record>>)
    ensures
        containers_are_the_same(containers@, copy@),
        records_of_containers(copy@) == records_of_containers(containers@),
{
    let mut copy: Vec<RecordContainer<Record>> = Vec::new();
    let mut position: usize = 0;
    while position < containers.len()
        invariant
            position <= containers.len(),
            containers_are_the_same(containers@.take(position as int), copy@),
            records_of_containers(copy@) == records_of_containers(containers@.take(position as int)),
        decreases containers.len() - position,
    {
        let container = &containers[position];
        let ghost copy_before = copy@;
        copy.push(RecordContainer { identity: container.identity, records: copy_of_the_records(&container.records) });
        position += 1;
        proof {
            assert(containers@.take(position as int).drop_last() =~= containers@.take(position as int - 1));
            assert(copy@.drop_last() =~= copy_before);
        }
    }
    proof {
        assert(containers@.take(containers.len() as int) =~= containers@);
    }
    copy
}

impl<Record: Copy> ContainerTreeAfterThisPublish<Record> {
    /// 这次不追加的发布：容器、形状都照抄上一版，这次之前那一版只记每层节点数。
    ///
    /// # Panics
    /// 形状里的容器数与容器的片数不等（`requires` 排除的那一种）：两样出自同一版。
    #[must_use]
    pub fn carried(containers: Vec<RecordContainer<Record>>, shape: ContainerTreeShape) -> (carried: Self)
        requires
            container_count(shape.levels()) == containers.len(),
        ensures
            carried.containers@ == containers@,
            carried.rewritten_containers@.len() == 0,
            carried.shape.levels() == shape.levels(),
            carried.node_counts_before_this_publish.counts() == node_counts_of(shape.levels()),
    {
        assert!(
            shape.containers() == containers.len(),
            "形状里层级 1 的孩子数之和就是容器数"
        );
        let node_counts_before_this_publish = shape.node_counts();
        Self { containers, rewritten_containers: Vec::new(), shape, node_counts_before_this_publish }
    }
}

impl<Record> ContainerTreeAfterThisPublish<Record> {
    /// 根跟着重写：有容器重写（COW 叶 + 祖先）。容器只增不减，没有「删空一片」那一格。
    #[must_use]
    pub fn is_the_root_rewritten(&self) -> (is_the_root_rewritten: bool)
        ensures
            is_the_root_rewritten == (self.rewritten_containers.len() > 0),
    {
        !self.rewritten_containers.is_empty()
    }

    /// 这次之后的内部层数（根的层级减一，零根与只有根时 0）。
    #[must_use]
    pub fn internal_levels(&self) -> (internal_levels: u64)
        requires
            self.shape.levels().len() <= 255,
        ensures
            internal_levels == if self.shape.levels().len() == 0 { 0 } else { self.shape.levels().len() - 1 },
    {
        self.shape.internal_levels()
    }

    /// 这次重写的容器从第几片起（重写的是最右端连着的那几片，[`rewritten_containers_are_the_rightmost`]）；一片都没重写时 `None`。
    /// 发布路径问「第几片这次重写了没有」按它比：序号不小于它就重写了，不在重写清单上逐个找（`crate::transaction` 列映射单元、装容器单元）。
    ///
    /// # Panics
    /// 重写清单不是最右端连着的那几片（`requires` 排除的那一种）：造它的只有 [`append_records_to_the_open_container`] 与 [`Self::carried`]，
    /// 两处的 `ensures` 都写着它是。逐片核一遍，代价是重写的片数。
    #[must_use]
    pub fn first_rewritten_container(&self) -> (first: Option<usize>)
        requires
            rewritten_containers_are_the_rightmost(self.rewritten_containers@, self.containers@.len() as int),
        ensures
            first is None <==> self.rewritten_containers@.len() == 0,
            first matches Some(first_position) ==> first_position == self.containers@.len() - self.rewritten_containers@.len()
                && forall|position: int|
                0 <= position < self.containers@.len() ==> (#[trigger] self.rewritten_containers@.contains(position as usize)
                    <==> position >= first_position),
    {
        let rightmost_count = self.rightmost_count_of_the_rewritten_containers(self.containers.len());
        if rightmost_count == 0 {
            return None;
        }
        let first_position = self.containers.len() - rightmost_count;
        proof {
            let rewritten = self.rewritten_containers@;
            assert forall|position: int| 0 <= position < self.containers@.len() implies (#[trigger] rewritten.contains(
                position as usize,
            ) <==> position >= first_position) by {
                if position >= first_position {
                    assert(rewritten[position - first_position] == position);
                }
                if rewritten.contains(position as usize) {
                    let found = choose|found: int| 0 <= found < rewritten.len() && rewritten[found] == position as usize;
                    assert(rewritten[found] == first_position + found);
                }
            }
        }
        Some(first_position)
    }

    /// 重写清单有几片，连带逐片核它就是最右端连着的那几片（一共 `total` 片）：代价是重写的片数。
    ///
    /// # Panics
    /// 重写清单不是最右端连着的那几片（`requires` 排除的那一种）：见 [`Self::first_rewritten_container`]。
    fn rightmost_count_of_the_rewritten_containers(&self, total: usize) -> (rightmost_count: usize)
        requires
            rewritten_containers_are_the_rightmost(self.rewritten_containers@, total as int),
        ensures
            rightmost_count == self.rewritten_containers@.len(),
    {
        let rightmost_count = self.rewritten_containers.len();
        assert!(
            rightmost_count <= total,
            "重写的 {rightmost_count} 片是这一版 {total} 片里最右端的那几片"
        );
        let mut listed: usize = 0;
        while listed < rightmost_count
            invariant
                rewritten_containers_are_the_rightmost(self.rewritten_containers@, total as int),
                rightmost_count == self.rewritten_containers@.len(),
                rightmost_count <= total,
                listed <= rightmost_count,
            decreases rightmost_count - listed,
        {
            assert!(
                self.rewritten_containers[listed] == total - rightmost_count + listed,
                "重写清单是最右端连着的那几片：造它的只有 append_records_to_the_open_container 与 carried"
            );
            listed += 1;
        }
        rightmost_count
    }

    /// 这次长了一层：上一版有根、这次之后的根层级比它高。那一次上一版的根不重写、换成根之下的节点留在原位置，不被这次换下。
    #[must_use]
    pub fn grew_a_level_this_publish(&self) -> (grew_a_level: bool)
        requires
            self.shape.levels().len() <= 255,
            self.node_counts_before_this_publish.counts().len() <= 255,
        ensures
            grew_a_level == (self.node_counts_before_this_publish.counts().len() > 0 && self.shape.levels().len()
                > self.node_counts_before_this_publish.counts().len()),
    {
        self.node_counts_before_this_publish.root_level() > 0
            && self.shape.root_level() > self.node_counts_before_this_publish.root_level()
    }

    /// 这次长了一层、而上一版的根一个字节不动地留在原位置当根之下的节点（它下面没有容器重写）：那一次新根在上一版里没有落点，
    /// 上一版的根不被这次换下。一次追加好几条、上一版的开放容器先装满再长层时，旧根下面有容器重写，它跟着重写、照样被换下。
    #[must_use]
    pub fn previous_root_is_carried_down_unchanged(&self) -> (is_carried_down_unchanged: bool)
        requires
            shape_fits_the_position_fields(self.shape.levels()),
            self.node_counts_before_this_publish.counts().len() <= 255,
            rewritten_containers_are_the_rightmost(self.rewritten_containers@, container_count(self.shape.levels())),
        ensures
            is_carried_down_unchanged == (self.node_counts_before_this_publish.counts().len() > 0 && self.shape.levels().len()
                > self.node_counts_before_this_publish.counts().len() && !ancestors_below_level(
                self.shape.levels(),
                self.rewritten_containers@,
                self.shape.levels().len() as int,
            ).contains(position_at(self.node_counts_before_this_publish.counts().len() - 1, 0))),
    {
        if !self.grew_a_level_this_publish() {
            return false;
        }
        match self.node_counts_before_this_publish.root_position() {
            Some(previous_root) => {
                let rewritten_nodes = self.rewritten_nodes();
                !position_is_listed(&rewritten_nodes, previous_root)
            },
            None => false,
        }
    }

    /// 这次重写的码 2 节点，按 bump 次序（层级升序，根最末）：重写的每片容器的全部祖先（COW 叶 + 祖先）。重写的是最右端连着的那几片，
    /// 每层的祖先是一段后缀（[`ContainerTreeShape::ancestors_of_the_rightmost_containers`]），只读那几段。
    ///
    /// # Panics
    /// 重写清单不是最右端连着的那几片（`requires` 排除的那一种）：见 [`Self::first_rewritten_container`]。
    #[must_use]
    pub fn rewritten_nodes(&self) -> (rewritten_nodes: Vec<ContainerTreeNodePosition>)
        requires
            shape_fits_the_position_fields(self.shape.levels()),
            rewritten_containers_are_the_rightmost(self.rewritten_containers@, container_count(self.shape.levels())),
        ensures
            rewritten_nodes@ == ancestors_below_level(
                self.shape.levels(),
                self.rewritten_containers@,
                self.shape.levels().len() as int,
            ),
    {
        let rightmost_count = self.rightmost_count_of_the_rewritten_containers(self.shape.containers());
        proof {
            lemma_rewritten_containers_are_the_rightmost_ones(self.rewritten_containers@, container_count(self.shape.levels()));
        }
        if rightmost_count == 0 {
            proof {
                assert(self.rewritten_containers@ =~= Seq::<usize>::empty());
                lemma_no_containers_have_no_ancestors(self.shape.levels(), self.shape.levels().len() as int);
            }
            return Vec::new();
        }
        self.shape.ancestors_of_the_rightmost_containers(rightmost_count)
    }

    /// 这一版每个码 2 节点从哪来，按 bump 次序。这次重写的是重写的那几片的祖先：每层是一段后缀，按每层那一段的起点判
    /// （[`ContainerTreeShape::ancestor_suffix_starts_of_the_rightmost_containers`]），不在重写的节点清单上逐个找。
    ///
    /// # Panics
    /// 一个照抄的位置在上一版里没有节点（`requires` 排除的那一种）：容器与节点都只在最右端新开，最右那一条路径之外的位置上一版都有。
    /// 重写清单不是最右端连着的那几片：见 [`Self::first_rewritten_container`]。
    #[must_use]
    pub fn node_origins(&self) -> (node_origins: Vec<(ContainerTreeNodePosition, ContainerTreeNodeOrigin)>)
        requires
            shape_fits_the_position_fields(self.shape.levels()),
            rewritten_containers_are_the_rightmost(self.rewritten_containers@, container_count(self.shape.levels())),
            every_node_absent_from_the_previous_version_is_rewritten(
                self.node_counts_before_this_publish.counts(),
                self.shape.levels(),
                self.rewritten_containers@,
            ),
        ensures
            node_origins.len() == bump_order_positions(self.shape.levels(), self.shape.levels().len() as int).len(),
            forall|listed: int|
                0 <= listed < node_origins.len() ==> {
                    let origin = #[trigger] node_origins@[listed];
                    &&& origin.0 == bump_order_positions(self.shape.levels(), self.shape.levels().len() as int)[listed]
                    &&& (origin.1 is RewrittenThisPublish <==> ancestors_below_level(
                        self.shape.levels(),
                        self.rewritten_containers@,
                        self.shape.levels().len() as int,
                    ).contains(origin.0))
                    &&& origin.1 is CarriedFromThePreviousVersion ==> position_is_counted(
                        self.node_counts_before_this_publish.counts(),
                        origin.0,
                    )
                },
    {
        proof {
            use_type_invariant(&self.shape);
            lemma_bump_order_positions_are_nodes(self.shape.levels(), self.shape.levels().len() as int);
            assert forall|level_index: int, node: int|
                #[trigger] node_is_in_the_levels(self.shape.levels(), level_index, node) implies level_index < self.shape.levels().len() by {}
            lemma_ancestors_below_level_contains(
                self.shape.levels(),
                self.rewritten_containers@,
                self.shape.levels().len() as int,
            );
            lemma_rewritten_containers_are_the_rightmost_ones(self.rewritten_containers@, container_count(self.shape.levels()));
        }
        let ghost levels = self.shape.levels();
        let ghost rewritten = ancestors_below_level(levels, self.rewritten_containers@, levels.len() as int);
        let rightmost_count = self.rightmost_count_of_the_rewritten_containers(self.shape.containers());
        // 每层重写的那一段从哪个节点起；一片都没重写时一个节点都不重写。
        let suffix_starts = if rightmost_count == 0 {
            None
        } else {
            Some(self.shape.ancestor_suffix_starts_of_the_rightmost_containers(rightmost_count))
        };
        proof {
            if rightmost_count == 0 {
                assert(self.rewritten_containers@ =~= Seq::<usize>::empty());
            }
        }
        let positions = self.shape.positions_in_bump_order();
        let mut node_origins: Vec<(ContainerTreeNodePosition, ContainerTreeNodeOrigin)> = Vec::new();
        let mut listed: usize = 0;
        while listed < positions.len()
            invariant
                levels == self.shape.levels(),
                container_tree_shape_is_well_formed(levels),
                shape_fits_the_position_fields(levels),
                positions@ == bump_order_positions(levels, levels.len() as int),
                rewritten == ancestors_below_level(levels, self.rewritten_containers@, levels.len() as int),
                self.rewritten_containers@ == rightmost_containers(container_count(levels), rightmost_count as int),
                rightmost_count == self.rewritten_containers@.len(),
                every_node_absent_from_the_previous_version_is_rewritten(
                    self.node_counts_before_this_publish.counts(),
                    levels,
                    self.rewritten_containers@,
                ),
                match suffix_starts {
                    Some(starts) => rightmost_count >= 1 && starts@.len() == levels.len() && forall|level_index: int, node: int|
                        #[trigger] node_is_in_the_levels(levels, level_index, node) ==> (node_is_an_ancestor_of_one_of(
                            levels,
                            level_index,
                            node,
                            self.rewritten_containers@,
                        ) <==> node >= starts@[level_index]),
                    None => rightmost_count == 0,
                },
                forall|any_listed: int|
                    0 <= any_listed < positions.len() ==> {
                        let position = #[trigger] positions@[any_listed];
                        &&& 1 <= position.level <= levels.len()
                        &&& position.index_in_level < levels[position.level - 1].len()
                    },
                forall|level_index: int, node: int|
                    #[trigger] node_is_in_the_levels(levels, level_index, node) && level_index < levels.len() ==> (
                    rewritten.contains(position_at(level_index, node)) <==> node_is_an_ancestor_of_one_of(
                        levels,
                        level_index,
                        node,
                        self.rewritten_containers@,
                    )),
                listed <= positions.len(),
                node_origins.len() == listed,
                forall|done: int|
                    0 <= done < listed ==> {
                        let origin = #[trigger] node_origins@[done];
                        &&& origin.0 == positions@[done]
                        &&& (origin.1 is RewrittenThisPublish <==> rewritten.contains(origin.0))
                        &&& origin.1 is CarriedFromThePreviousVersion ==> position_is_counted(
                            self.node_counts_before_this_publish.counts(),
                            origin.0,
                        )
                    },
            decreases positions.len() - listed,
        {
            let position = positions[listed];
            let level_index = usize::from(position.level) - 1;
            let index_in_level = usize::try_from(position.index_in_level).expect("同层序号装得进 usize");
            proof {
                assert(1 <= position.level <= levels.len());
                assert(node_is_in_the_levels(levels, level_index as int, index_in_level as int));
                assert(position == position_at(level_index as int, index_in_level as int));
            }
            let is_rewritten = match &suffix_starts {
                Some(starts) => index_in_level >= starts[level_index],
                None => false,
            };
            proof {
                if rightmost_count == 0 {
                    assert(!node_is_an_ancestor_of_one_of(levels, level_index as int, index_in_level as int, self.rewritten_containers@));
                }
                assert(is_rewritten == rewritten.contains(position));
            }
            if is_rewritten {
                node_origins.push((position, ContainerTreeNodeOrigin::RewrittenThisPublish));
            } else {
                proof {
                    assert(!node_is_an_ancestor_of_one_of(levels, level_index as int, index_in_level as int, self.rewritten_containers@));
                    assert(node_is_counted(self.node_counts_before_this_publish.counts(), level_index as int, index_in_level as int));
                }
                let level = position.level;
                let index_in_level_of_the_position = position.index_in_level;
                assert!(
                    self.node_counts_before_this_publish.has_a_node_at(position),
                    "照抄的容器树节点（层级 {level}、同层第 {index_in_level_of_the_position} 个）在上一版里有：只有最右那一条路径上的节点是新开的"
                );
                node_origins.push((position, ContainerTreeNodeOrigin::CarriedFromThePreviousVersion));
            }
            listed += 1;
        }
        node_origins
    }
}

/// 要开一片新容器时下一个容器号装不下：最右那片的号（盘上读来的 8 字节）已是 `u64::MAX`，加一越过顶（C841
/// 「写路径报类型化错误」：开新容器那一次发布报这个错、不 panic，读盘不变）。容器号从 1 起逐片加一，健康的树走不到顶，
/// 坏镜像、外来镜像才有。`tree` 是要开新容器的那棵树（墓碑树、谱系树或退役树），`rightmost_container` 是最右那片的号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NextContainerNumberPastTheTopOfItsRange {
    pub tree: TreeIdentifier,
    pub rightmost_container: u64,
}

/// 第 `position` 片是这次新开的：身份照参数、容器号比左邻大一（第 0 片是 1）、装了 1 到 `records_per_container` 条。
pub open spec fn container_is_opened_by_this_publish<Record>(
    containers: Seq<RecordContainer<Record>>,
    position: int,
    tree: TreeIdentifier,
    record_type: u16,
    checkpoint_txg: CheckpointTxg,
    records_per_container: int,
) -> bool {
    let identity = containers[position].identity;
    &&& identity.birth_tree == tree
    &&& identity.record_type == record_type
    &&& identity.container_birth == checkpoint_txg
    &&& identity.container == if position == 0 {
        1
    } else {
        containers[position - 1].identity.container + 1
    }
    &&& 1 <= containers[position].records@.len() <= records_per_container
}

/// 追加到这一条为止，容器、形状与重写清单对上一版与已追加的记录逐条成立的那几样（[`append_records_to_the_open_container`] 的循环不变量与 `ensures`）：
/// 「key 到条目」——全部记录依次连起来 = 上一版的接上已追加的；最右那片之前的容器原样；最右那片身份不变、原有记录是前缀；
/// 新开的容器身份照参数、容器号逐片加一；形状在最右端长出来、容器数等于片数；重写清单升序、罩住新开的每一片、最后一项是最右那片，
/// 而且就是最右端连着的那几片（[`rewritten_containers_are_the_rightmost`]：发布路径按它判第几片、哪个节点重写了，不在清单上逐个找）；
/// 一条都没追加时什么都没动；不在上一版的节点都罩着重写的某一片（照抄的节点上一版都有）。
pub open spec fn appending_progress<Record>(
    containers_before_this_publish: Seq<RecordContainer<Record>>,
    shape_before_this_publish: Seq<Seq<usize>>,
    appended_records: Seq<Record>,
    checkpoint_txg: CheckpointTxg,
    tree: TreeIdentifier,
    record_type: u16,
    records_per_container: int,
    containers: Seq<RecordContainer<Record>>,
    levels: Seq<Seq<usize>>,
    rewritten_containers: Seq<usize>,
) -> bool {
    let before = containers_before_this_publish;
    let before_count = before.len();
    &&& records_of_containers(containers) == records_of_containers(before) + appended_records
    &&& containers.len() >= before_count
    &&& forall|position: int|
        0 <= position < before_count - 1 ==> (#[trigger] containers[position]).identity == before[position].identity
            && containers[position].records@ == before[position].records@
    &&& before_count >= 1 ==> containers[before_count - 1].identity == before[before_count - 1].identity
    &&& before_count >= 1 ==> containers[before_count - 1].records@.take(before[before_count - 1].records@.len() as int)
        == before[before_count - 1].records@
    &&& before_count >= 1 ==> containers[before_count - 1].records@.len() >= before[before_count - 1].records@.len()
    &&& forall|position: int|
        before_count <= position < containers.len() ==> #[trigger] container_is_opened_by_this_publish(
            containers,
            position,
            tree,
            record_type,
            checkpoint_txg,
            records_per_container,
        )
    &&& container_count(levels) == containers.len()
    &&& levels_extend(shape_before_this_publish, levels)
    &&& forall|earlier: int, later: int|
        0 <= earlier < later < rewritten_containers.len() ==> #[trigger] rewritten_containers[earlier]
            < #[trigger] rewritten_containers[later]
    &&& forall|listed: int| 0 <= listed < rewritten_containers.len() ==> #[trigger] rewritten_containers[listed] < containers.len()
    &&& forall|position: int|
        before_count <= position < containers.len() ==> #[trigger] rewritten_containers.contains(position as usize)
    &&& appended_records.len() == 0 ==> rewritten_containers.len() == 0 && containers_are_the_same(before, containers)
        && levels == shape_before_this_publish
    &&& appended_records.len() > 0 ==> rewritten_containers.len() >= 1 && rewritten_containers.last() == containers.len() - 1
    &&& rewritten_containers_are_the_rightmost(rewritten_containers, containers.len() as int)
    &&& every_node_absent_from_the_previous_version_is_rewritten(node_counts_of(shape_before_this_publish), levels, rewritten_containers)
}

/// [`append_records_to_the_open_container`] 的 `ensures`：交回的样子对上一版、这次给的记录与参数照 [`appending_progress`] 成立，
/// 这次之前那一版记的每层节点数就是上一版形状的。
pub open spec fn appending_keeps_the_key_to_entry_mapping<Record>(
    containers_before_this_publish: Seq<RecordContainer<Record>>,
    shape_before_this_publish: Seq<Seq<usize>>,
    records: Seq<Record>,
    checkpoint_txg: CheckpointTxg,
    tree: TreeIdentifier,
    record_type: u16,
    records_per_container: int,
    after: ContainerTreeAfterThisPublish<Record>,
) -> bool {
    &&& after.node_counts_before_this_publish.counts() == node_counts_of(shape_before_this_publish)
    &&& appending_progress(
        containers_before_this_publish,
        shape_before_this_publish,
        records,
        checkpoint_txg,
        tree,
        record_type,
        records_per_container,
        after.containers@,
        after.shape.levels(),
        after.rewritten_containers@,
    )
}

/// 一条都还没追加时，照抄上一版就对上了 [`appending_progress`]。
proof fn lemma_appending_progress_at_the_start<Record>(
    containers_before_this_publish: Seq<RecordContainer<Record>>,
    shape_before_this_publish: Seq<Seq<usize>>,
    checkpoint_txg: CheckpointTxg,
    tree: TreeIdentifier,
    record_type: u16,
    records_per_container: int,
    containers: Seq<RecordContainer<Record>>,
)
    requires
        container_tree_shape_is_well_formed(shape_before_this_publish),
        container_count(shape_before_this_publish) == containers_before_this_publish.len(),
        containers_are_the_same(containers_before_this_publish, containers),
        records_of_containers(containers) == records_of_containers(containers_before_this_publish),
    ensures
        appending_progress(
            containers_before_this_publish,
            shape_before_this_publish,
            Seq::empty(),
            checkpoint_txg,
            tree,
            record_type,
            records_per_container,
            containers,
            shape_before_this_publish,
            Seq::empty(),
        ),
{
    let before = containers_before_this_publish;
    assert(records_of_containers(before) + Seq::<Record>::empty() =~= records_of_containers(before));
    if before.len() >= 1 {
        assert(containers[before.len() - 1].records@ == before[before.len() - 1].records@);
        assert(containers[before.len() - 1].records@.take(before[before.len() - 1].records@.len() as int)
            =~= before[before.len() - 1].records@);
    }
    assert forall|level_index: int| 0 <= level_index < shape_before_this_publish.len() implies nodes_extend(
        #[trigger] shape_before_this_publish[level_index],
        shape_before_this_publish[level_index],
    ) by {
        assert(shape_before_this_publish[level_index].len() >= 1);
    }
    assert forall|level_index: int, node: int|
        #[trigger] node_is_in_the_levels(shape_before_this_publish, level_index, node) implies node_is_counted(
        node_counts_of(shape_before_this_publish),
        level_index,
        node,
    ) by {}
}

/// 往还装得下的最右那片里追加一条之后，[`appending_progress`] 照样成立。
proof fn lemma_appending_into_the_open_container_keeps_the_progress<Record>(
    containers_before_this_publish: Seq<RecordContainer<Record>>,
    shape_before_this_publish: Seq<Seq<usize>>,
    appended_records: Seq<Record>,
    record: Record,
    checkpoint_txg: CheckpointTxg,
    tree: TreeIdentifier,
    record_type: u16,
    records_per_container: int,
    containers: Seq<RecordContainer<Record>>,
    levels: Seq<Seq<usize>>,
    rewritten_containers: Seq<usize>,
    containers_after: Seq<RecordContainer<Record>>,
    rewritten_after: Seq<usize>,
)
    requires
        appending_progress(
            containers_before_this_publish,
            shape_before_this_publish,
            appended_records,
            checkpoint_txg,
            tree,
            record_type,
            records_per_container,
            containers,
            levels,
            rewritten_containers,
        ),
        1 <= containers.len() <= usize::MAX,
        containers.last().records@.len() < records_per_container,
        containers_after.len() == containers.len(),
        containers_after.drop_last() == containers.drop_last(),
        containers_after.last().identity == containers.last().identity,
        containers_after.last().records@ == containers.last().records@.push(record),
        rewritten_after == if rewritten_containers.len() >= 1 && rewritten_containers.last() == containers.len() - 1 {
            rewritten_containers
        } else {
            rewritten_containers.push((containers.len() - 1) as usize)
        },
    ensures
        appending_progress(
            containers_before_this_publish,
            shape_before_this_publish,
            appended_records.push(record),
            checkpoint_txg,
            tree,
            record_type,
            records_per_container,
            containers_after,
            levels,
            rewritten_after,
        ),
{
    let before = containers_before_this_publish;
    let before_count = before.len();
    let last = containers.len() - 1;
    lemma_records_of_containers_after_appending_to_the_last(containers, containers_after, record);
    assert(records_of_containers(before) + appended_records.push(record) =~= (records_of_containers(before)
        + appended_records).push(record));
    assert forall|position: int| 0 <= position < last implies #[trigger] containers_after[position] == containers[position] by {
        assert(containers_after.drop_last()[position] == containers_after[position]);
        assert(containers.drop_last()[position] == containers[position]);
        assert(containers_after.drop_last()[position] == containers.drop_last()[position]);
    }
    assert(containers_after[last].identity == containers[last].identity);
    if before_count >= 1 {
        assert(containers_after[before_count - 1].identity == before[before_count - 1].identity);
        assert(containers_after[before_count - 1].records@.len() >= before[before_count - 1].records@.len());
    }
    assert forall|position: int| 0 <= position < before_count - 1 implies (#[trigger] containers_after[position]).identity
        == before[position].identity && containers_after[position].records@ == before[position].records@ by {
        assert(containers_after[position] == containers[position]);
    }
    assert(container_count(levels) == containers_after.len());
    if before_count >= 1 && last == before_count - 1 {
        let earlier_records = before[before_count - 1].records@;
        assert(containers_after[last].records@.take(earlier_records.len() as int) =~= containers[last].records@.take(
            earlier_records.len() as int,
        ));
    }
    assert forall|position: int| before_count <= position < containers_after.len() implies #[trigger] container_is_opened_by_this_publish(
        containers_after,
        position,
        tree,
        record_type,
        checkpoint_txg,
        records_per_container,
    ) by {
        assert(container_is_opened_by_this_publish(
            containers,
            position,
            tree,
            record_type,
            checkpoint_txg,
            records_per_container,
        ));
        if position >= 1 && position - 1 < last {
            assert(containers_after[position - 1] == containers[position - 1]);
        }
    }
    if appended_records.len() == 0 {
        assert(rewritten_containers.len() == 0);
        assert(rewritten_after =~= seq![last as usize]);
        assert(rewritten_after.last() == last as usize);
        assert(rewritten_after.len() == 1);
        lemma_more_rewritten_containers_keep_every_new_node_rewritten(
            node_counts_of(shape_before_this_publish),
            levels,
            rewritten_containers,
            last as usize,
        );
        assert forall|position: int| before_count <= position < containers_after.len() implies #[trigger] rewritten_after.contains(
            position as usize,
        ) by {
            assert(rewritten_after[0] == position as usize);
        }
    } else {
        assert(rewritten_after == rewritten_containers);
        assert(rewritten_after.last() == containers.len() - 1);
    }
    let appended_after = appended_records.push(record);
    assert(records_of_containers(containers_after) == records_of_containers(before) + appended_after);
    assert(containers_after.len() >= before_count);
    assert(before_count >= 1 ==> containers_after[before_count - 1].records@.take(before[before_count - 1].records@.len() as int)
        == before[before_count - 1].records@);
    assert(levels_extend(shape_before_this_publish, levels));
    assert(forall|earlier: int, later: int|
        0 <= earlier < later < rewritten_after.len() ==> #[trigger] rewritten_after[earlier] < #[trigger] rewritten_after[later]);
    assert(forall|listed: int| 0 <= listed < rewritten_after.len() ==> #[trigger] rewritten_after[listed] < containers_after.len());
    assert(appended_after.len() > 0 ==> rewritten_after.len() >= 1 && rewritten_after.last() == containers_after.len() - 1);
    assert(every_node_absent_from_the_previous_version_is_rewritten(node_counts_of(shape_before_this_publish), levels, rewritten_after));
    assert forall|listed: int| 0 <= listed < rewritten_after.len() implies #[trigger] rewritten_after[listed] == containers_after.len()
        - rewritten_after.len() + listed by {
        if appended_records.len() == 0 {
            assert(rewritten_after[0] == last as usize);
        } else {
            assert(rewritten_after[listed] == rewritten_containers[listed]);
        }
    }
    assert(rewritten_containers_are_the_rightmost(rewritten_after, containers_after.len() as int));
}

/// 最右那片装满（或树是空的）、开一片新容器追加一条之后，[`appending_progress`] 照样成立。
proof fn lemma_opening_a_container_keeps_the_progress<Record>(
    containers_before_this_publish: Seq<RecordContainer<Record>>,
    shape_before_this_publish: Seq<Seq<usize>>,
    appended_records: Seq<Record>,
    record: Record,
    checkpoint_txg: CheckpointTxg,
    tree: TreeIdentifier,
    record_type: u16,
    records_per_container: int,
    entries_per_node: int,
    containers: Seq<RecordContainer<Record>>,
    levels: Seq<Seq<usize>>,
    rewritten_containers: Seq<usize>,
    opened: RecordContainer<Record>,
    rewritten_after: Seq<usize>,
)
    requires
        appending_progress(
            containers_before_this_publish,
            shape_before_this_publish,
            appended_records,
            checkpoint_txg,
            tree,
            record_type,
            records_per_container,
            containers,
            levels,
            rewritten_containers,
        ),
        container_tree_shape_is_well_formed(levels),
        container_count(levels) + 1 <= usize::MAX,
        2 <= entries_per_node <= usize::MAX,
        records_per_container >= 1,
        opened.identity == (PackedIdentity {
            birth_tree: tree,
            record_type,
            container: if containers.len() == 0 {
                1
            } else {
                (containers.last().identity.container + 1) as u64
            },
            container_birth: checkpoint_txg,
        }),
        containers.len() >= 1 ==> containers.last().identity.container + 1 <= u64::MAX,
        opened.records@ == seq![record],
        rewritten_after == if rewritten_containers.len() >= 1 && rewritten_containers.last() == containers.len() {
            rewritten_containers
        } else {
            rewritten_containers.push(containers.len() as usize)
        },
    ensures
        appending_progress(
            containers_before_this_publish,
            shape_before_this_publish,
            appended_records.push(record),
            checkpoint_txg,
            tree,
            record_type,
            records_per_container,
            containers.push(opened),
            levels_after_opening_at_the_right_end(levels, entries_per_node),
            rewritten_after,
        ),
{
    let before = containers_before_this_publish;
    let before_count = before.len();
    let containers_after = containers.push(opened);
    let levels_after = levels_after_opening_at_the_right_end(levels, entries_per_node);
    lemma_records_of_containers_after_a_push(containers, opened);
    assert(records_of_containers(before) + appended_records.push(record) =~= (records_of_containers(before)
        + appended_records) + seq![record]);
    assert forall|position: int| 0 <= position < containers.len() implies #[trigger] containers_after[position]
        == containers[position] by {}
    assert forall|position: int| before_count <= position < containers_after.len() implies #[trigger] container_is_opened_by_this_publish(
        containers_after,
        position,
        tree,
        record_type,
        checkpoint_txg,
        records_per_container,
    ) by {
        if position < containers.len() {
            assert(container_is_opened_by_this_publish(
                containers,
                position,
                tree,
                record_type,
                checkpoint_txg,
                records_per_container,
            ));
            if position >= 1 {
                assert(containers_after[position - 1] == containers[position - 1]);
            }
        } else {
            if position >= 1 {
                assert(containers_after[position - 1] == containers.last());
            }
        }
    }
    lemma_opening_keeps_the_shape_well_formed(levels, entries_per_node);
    lemma_opening_extends_the_levels_at_the_right_end(levels, entries_per_node);
    lemma_levels_extend_transitively(shape_before_this_publish, levels, levels_after);
    assert forall|listed: int| 0 <= listed < rewritten_containers.len() implies #[trigger] rewritten_containers[listed]
        < container_count(levels) by {}
    lemma_opening_keeps_every_new_node_rewritten(
        node_counts_of(shape_before_this_publish),
        levels,
        entries_per_node,
        rewritten_containers,
    );
    assert(rewritten_after == rewritten_containers.push(containers.len() as usize));
    assert forall|position: int| before_count <= position < containers_after.len() implies #[trigger] rewritten_after.contains(
        position as usize,
    ) by {
        if position < containers.len() {
            assert(rewritten_containers.contains(position as usize));
            let found = choose|found: int|
                0 <= found < rewritten_containers.len() && rewritten_containers[found] == position as usize;
            assert(rewritten_after[found] == position as usize);
        } else {
            assert(rewritten_after[rewritten_containers.len() as int] == position as usize);
        }
    }
    assert forall|earlier: int, later: int| 0 <= earlier < later < rewritten_after.len() implies #[trigger] rewritten_after[earlier]
        < #[trigger] rewritten_after[later] by {
        if later < rewritten_containers.len() {
            assert(rewritten_after[earlier] == rewritten_containers[earlier]);
            assert(rewritten_after[later] == rewritten_containers[later]);
        } else {
            assert(rewritten_after[earlier] == rewritten_containers[earlier]);
        }
    }
    assert forall|listed: int| 0 <= listed < rewritten_after.len() implies #[trigger] rewritten_after[listed] == containers_after.len()
        - rewritten_after.len() + listed by {
        if listed < rewritten_containers.len() {
            assert(rewritten_after[listed] == rewritten_containers[listed]);
        } else {
            assert(rewritten_after[listed] == containers.len());
        }
    }
    assert(rewritten_containers_are_the_rightmost(rewritten_after, containers_after.len() as int));
    if appended_records.len() == 0 {
        assert(levels == shape_before_this_publish);
    }
}

/// 往容器树的开放容器（最右那一片）追加一条记录：[`append_records_to_the_open_container`] 只给一条的那一格。
///
/// # Errors
/// 同 [`append_records_to_the_open_container`]。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：上一版的容器与形状、这条记录、txg、树、记录类型、一片装几条、一个节点装几条"
)]
pub fn append_record_to_the_open_container<Record: Copy>(
    containers_before_this_publish: &[RecordContainer<Record>],
    shape_before_this_publish: &ContainerTreeShape,
    record: Record,
    checkpoint_txg: CheckpointTxg,
    tree: TreeIdentifier,
    record_type: u16,
    records_per_container: usize,
    node_entries: ContainerTreeNodeEntries,
) -> (after: Result<ContainerTreeAfterThisPublish<Record>, NextContainerNumberPastTheTopOfItsRange>)
    requires
        container_count(shape_before_this_publish.levels()) == containers_before_this_publish.len(),
        records_per_container >= 1,
    ensures
        match after {
            Ok(after) => appending_keeps_the_key_to_entry_mapping(
                containers_before_this_publish@,
                shape_before_this_publish.levels(),
                seq![record],
                checkpoint_txg,
                tree,
                record_type,
                records_per_container as int,
                after,
            ),
            Err(error) => error.tree == tree && error.rightmost_container == u64::MAX,
        },
{
    let records = vec![record];
    proof {
        assert(records@ =~= seq![record]);
    }
    append_records_to_the_open_container(
        containers_before_this_publish,
        shape_before_this_publish,
        records.as_slice(),
        checkpoint_txg,
        tree,
        record_type,
        records_per_container,
        node_entries,
    )
}

/// 往容器树的开放容器（最右那一片）按给的次序逐条追加记录：最右那片还装得下（记录数小于 `records_per_container`）就追加进去；装满了
/// （或树是空的）就开一片新容器：容器号 = 树里最大号 + 1（零根时 1），出生代 = 这次发布的 checkpoint_txg，出生树 = `tree`，
/// 打包记录类型 = `record_type`，形状照 [`ContainerTreeShape::after_a_container_is_opened_at_the_right_end`] 往上挂（根满长高）。
/// 这次追加过的每一片进重写清单（叶序升序、各一次）；别的容器一个字节都不动。`records` 为空时整棵照抄（[`ContainerTreeAfterThisPublish::carried`]）。
///
/// # Errors
/// 要开新容器而最右那片的号已是 `u64::MAX` ⇒ [`NextContainerNumberPastTheTopOfItsRange`]（C841）：在开那一片之前交回，
/// 只算、不写，调用方照样一个字节都没写。最右那片还装得下时照常追加，不看它的号。
///
/// # Panics
/// `shape_before_this_publish` 里的容器数与 `containers_before_this_publish` 不等：两样出自同一版。`records_per_container` 为 0：
/// 三棵树的容量开关的构造函数都判过至少 1。两样都是 `requires` 排除的。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：上一版的容器与形状、这几条记录、txg、树、记录类型、一片装几条、一个节点装几条"
)]
pub fn append_records_to_the_open_container<Record: Copy>(
    containers_before_this_publish: &[RecordContainer<Record>],
    shape_before_this_publish: &ContainerTreeShape,
    records: &[Record],
    checkpoint_txg: CheckpointTxg,
    tree: TreeIdentifier,
    record_type: u16,
    records_per_container: usize,
    node_entries: ContainerTreeNodeEntries,
) -> (after: Result<ContainerTreeAfterThisPublish<Record>, NextContainerNumberPastTheTopOfItsRange>)
    requires
        container_count(shape_before_this_publish.levels()) == containers_before_this_publish.len(),
        records_per_container >= 1,
    ensures
        match after {
            Ok(after) => appending_keeps_the_key_to_entry_mapping(
                containers_before_this_publish@,
                shape_before_this_publish.levels(),
                records@,
                checkpoint_txg,
                tree,
                record_type,
                records_per_container as int,
                after,
            ),
            Err(error) => error.tree == tree && error.rightmost_container == u64::MAX,
        },
{
    assert!(
        shape_before_this_publish.containers() == containers_before_this_publish.len(),
        "上一版容器树的形状与它的容器出自同一版"
    );
    assert!(
        records_per_container >= 1,
        "一片容器至少装一条：容量开关的构造函数判过"
    );
    let ghost before = containers_before_this_publish@;
    let ghost levels_before = shape_before_this_publish.levels();
    let mut containers = copy_of_the_containers(containers_before_this_publish);
    let mut shape = shape_before_this_publish.clone();
    let mut rewritten_containers: Vec<usize> = Vec::new();
    let entries_per_node = node_entries.entries_per_node();
    proof {
        use_type_invariant(shape_before_this_publish);
        lemma_appending_progress_at_the_start(
            before,
            levels_before,
            checkpoint_txg,
            tree,
            record_type,
            records_per_container as int,
            containers@,
        );
        assert(records@.take(0) =~= Seq::<Record>::empty());
        assert(rewritten_containers@ =~= Seq::<usize>::empty());
    }
    // 迭代次数 = 这次追加的条数；跨轮携带的是容器与形状（形状里的容器数恒等于容器数）与升序不重复的重写清单。
    let mut record_position: usize = 0;
    while record_position < records.len()
        invariant
            record_position <= records.len(),
            before == containers_before_this_publish@,
            levels_before == shape_before_this_publish.levels(),
            2 <= entries_per_node <= 135,
            records_per_container >= 1,
            appending_progress(
                before,
                levels_before,
                records@.take(record_position as int),
                checkpoint_txg,
                tree,
                record_type,
                records_per_container as int,
                containers@,
                shape.levels(),
                rewritten_containers@,
            ),
        decreases records.len() - record_position,
    {
        proof {
            use_type_invariant(&shape);
        }
        let record = records[record_position];
        let ghost containers_at_the_start = containers@;
        let ghost levels_at_the_start = shape.levels();
        let ghost rewritten_at_the_start = rewritten_containers@;
        let open_container_has_room = !containers.is_empty()
            && containers[containers.len() - 1].records.len() < records_per_container;
        if open_container_has_room {
            let mut open = containers.pop().expect("上一句判过最右那片在");
            open.records.push(record);
            containers.push(open);
        } else {
            // 容器号沿叶序严格递增、从 1 起（0 不用），下一个号 = 树里最大号 + 1：最右那片的号就是最大号。
            // 最右那片的号是盘上读来的 8 字节：加一越过顶的交回类型化错误（C841），不 panic。
            let next_container_number = if containers.is_empty() {
                1
            } else {
                let rightmost_container = containers[containers.len() - 1].identity.container;
                rightmost_container.checked_add(1).ok_or(
                    NextContainerNumberPastTheTopOfItsRange { tree, rightmost_container },
                )?
            };
            proof {
                assert(container_count(levels_at_the_start) == containers_at_the_start.len());
            }
            containers.push(
                RecordContainer {
                    identity: PackedIdentity {
                        birth_tree: tree,
                        record_type,
                        container: next_container_number,
                        container_birth: checkpoint_txg,
                    },
                    records: vec_of_one(record),
                },
            );
            proof {
                assert(containers@.len() == containers_at_the_start.len() + 1);
                assert(containers.len() <= usize::MAX);
            }
            shape = shape.after_a_container_is_opened_at_the_right_end(entries_per_node);
        }
        let rewritten_position = containers.len() - 1;
        if rewritten_containers.is_empty() || rewritten_containers[rewritten_containers.len() - 1] != rewritten_position {
            rewritten_containers.push(rewritten_position);
        }
        proof {
            if open_container_has_room {
                assert(containers@.drop_last() =~= containers_at_the_start.drop_last());
                assert(containers_at_the_start.len() == containers.len());
                lemma_appending_into_the_open_container_keeps_the_progress(
                    before,
                    levels_before,
                    records@.take(record_position as int),
                    record,
                    checkpoint_txg,
                    tree,
                    record_type,
                    records_per_container as int,
                    containers_at_the_start,
                    levels_at_the_start,
                    rewritten_at_the_start,
                    containers@,
                    rewritten_containers@,
                );
            } else {
                assert(containers@ == containers_at_the_start.push(containers@.last()));
                lemma_opening_a_container_keeps_the_progress(
                    before,
                    levels_before,
                    records@.take(record_position as int),
                    record,
                    checkpoint_txg,
                    tree,
                    record_type,
                    records_per_container as int,
                    entries_per_node as int,
                    containers_at_the_start,
                    levels_at_the_start,
                    rewritten_at_the_start,
                    containers@.last(),
                    rewritten_containers@,
                );
            }
            assert(records@.take(record_position as int).push(record) =~= records@.take(record_position as int + 1));
        }
        assert!(
            shape.containers() == containers.len(),
            "形状里层级 1 的孩子数之和就是容器数"
        );
        record_position += 1;
    }
    proof {
        assert(records@.take(records.len() as int) =~= records@);
    }
    Ok(
        ContainerTreeAfterThisPublish {
            containers,
            rewritten_containers,
            shape,
            node_counts_before_this_publish: shape_before_this_publish.node_counts(),
        },
    )
}

/// 只装一条的一组记录（新开容器的第一条）。
fn vec_of_one<Record: Copy>(record: Record) -> (records: Vec<Record>)
    ensures
        records@ == seq![record],
{
    let records = vec![record];
    assert(records@ =~= seq![record]);
    records
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;

    fn container_tree_node_entries_capped_at(entries: usize) -> ContainerTreeNodeEntries {
        ContainerTreeNodeEntries::CappedForTests(
            ContainerTreeNodeEntriesCappedForTests::new(entries).expect("2..=135"),
        )
    }

    fn shape_after_opening(containers: usize, entries: usize) -> ContainerTreeShape {
        (0..containers).fold(ContainerTreeShape::zero_root(), |shape, _| {
            shape.after_a_container_is_opened_at_the_right_end(
                container_tree_node_entries_capped_at(entries).entries_per_node(),
            )
        })
    }

    fn position(level: u8, index_in_level: u32) -> ContainerTreeNodePosition {
        ContainerTreeNodePosition {
            level,
            index_in_level,
        }
    }

    /// 一个节点的孩子在下一层里的下标区间：取 [`ContainerTreeShape::children_of_every_node`] 里那一项。
    fn children_of_the_node(
        shape: &ContainerTreeShape,
        node: ContainerTreeNodePosition,
    ) -> Range<usize> {
        let level_index = usize::from(node.level) - 1;
        let index_in_level =
            usize::try_from(node.index_in_level).expect("测试里的同层序号装得进 usize");
        shape.children_of_every_node()[level_index][index_in_level].clone()
    }

    /// 扇出压到 2：第 1、2 片挂在层级 1 的根下；第 3 片根满、长一层（旧根留在 (1, 0)、右半 (1, 1) 一个孩子、新根 (2, 0) 两个孩子）；
    /// 第 5 片两层都满、再长一层：层级 1 的 (1, 1) 满了开 (1, 2)；它的父亲是根 (2, 0)，也满了，旧根留在 (2, 0)、右半 (2, 1) 一个孩子 (1, 2)、
    /// 新根 (3, 0) 两个孩子 (2, 0) 与 (2, 1)。
    #[test]
    fn a_full_root_grows_a_level_with_the_old_root_kept_in_place_and_a_new_right_half() {
        assert_eq!(shape_after_opening(0, 2).root_level(), 0);
        assert_eq!(
            shape_after_opening(2, 2).positions_in_bump_order(),
            vec![position(1, 0)]
        );
        let three = shape_after_opening(3, 2);
        assert_eq!(
            three.positions_in_bump_order(),
            vec![position(1, 0), position(1, 1), position(2, 0)],
            "第 3 片：根满长一层，旧根在 (1, 0)、右半在 (1, 1)"
        );
        assert_eq!(children_of_the_node(&three, position(1, 0)), 0..2);
        assert_eq!(children_of_the_node(&three, position(1, 1)), 2..3);
        assert_eq!(children_of_the_node(&three, position(2, 0)), 0..2);
        assert_eq!(three.internal_levels(), 1);
        let five = shape_after_opening(5, 2);
        assert_eq!(
            five.positions_in_bump_order(),
            vec![
                position(1, 0),
                position(1, 1),
                position(1, 2),
                position(2, 0),
                position(2, 1),
                position(3, 0)
            ],
            "第 5 片：层级 1、2 都满，长到层级 3"
        );
        assert_eq!(children_of_the_node(&five, position(3, 0)), 0..2);
        assert_eq!(children_of_the_node(&five, position(2, 1)), 2..3);
        assert_eq!(five.containers(), 5);
    }

    /// 产品扇出 135：第 135 片还挂在层级 1 的根下，第 136 片长一层。
    #[test]
    fn the_product_fanout_grows_the_root_at_the_hundred_and_thirty_sixth_container() {
        let mut shape = ContainerTreeShape::zero_root();
        for _ in 0..135 {
            shape = shape.after_a_container_is_opened_at_the_right_end(
                ContainerTreeNodeEntries::FromTheNodeFormat.entries_per_node(),
            );
        }
        assert_eq!(shape.root_level(), 1);
        shape = shape.after_a_container_is_opened_at_the_right_end(
            ContainerTreeNodeEntries::FromTheNodeFormat.entries_per_node(),
        );
        assert_eq!(shape.root_level(), 2);
        assert_eq!(shape.nodes_at_level(1), 2);
    }

    /// 长高那次重写的是新开的容器、右半与新根（最右那一条路径），旧根照抄、位置不变；从盘上读回来的孩子数换回原样的形状。
    #[test]
    fn the_growth_publish_rewrites_the_rightmost_path_and_carries_the_old_root() {
        let before = shape_after_opening(2, 2);
        let containers: Vec<RecordContainer<u64>> = (1..=2)
            .map(|number| RecordContainer {
                identity: PackedIdentity {
                    birth_tree: TreeIdentifier(65545),
                    record_type: 1,
                    container: number,
                    container_birth: CheckpointTxg(5),
                },
                records: vec![number],
            })
            .collect();
        let after = append_record_to_the_open_container(
            &containers,
            &before,
            3_u64,
            CheckpointTxg(9),
            TreeIdentifier(65545),
            1,
            1,
            container_tree_node_entries_capped_at(2),
        )
        .expect("最右那片的号 2，加一装得下");
        assert!(after.grew_a_level_this_publish());
        assert_eq!(after.containers[2].identity.container, 3);
        assert_eq!(
            after.node_origins(),
            vec![
                (
                    position(1, 0),
                    ContainerTreeNodeOrigin::CarriedFromThePreviousVersion
                ),
                (
                    position(1, 1),
                    ContainerTreeNodeOrigin::RewrittenThisPublish
                ),
                (
                    position(2, 0),
                    ContainerTreeNodeOrigin::RewrittenThisPublish
                ),
            ]
        );
        assert_eq!(
            ContainerTreeShape::from_children_of_each_node_at_each_level(vec![vec![2, 1], vec![2]]),
            Some(after.shape.clone())
        );
        assert_eq!(
            ContainerTreeShape::from_children_of_each_node_at_each_level(vec![vec![2, 1], vec![1]]),
            None,
            "根的孩子数之和要等于下一层的节点数"
        );
    }

    /// 一次追加三条、一片装两条、扇出压到 2：上一版两片（第二片装着一条、还开着）。第一条装满第二片，第二、三条开第三片（这时根满长一层）。
    /// 这次重写的是第二、三片与它们的全部祖先：旧根 (1, 0) 下面的第二片被重写，旧根跟着重写、照样被这次换下。
    #[test]
    fn appending_several_records_rewrites_every_touched_container_and_all_their_ancestors() {
        let before_shape = shape_after_opening(2, 2);
        let identity = |number: u64| PackedIdentity {
            birth_tree: TreeIdentifier(65548),
            record_type: 3,
            container: number,
            container_birth: CheckpointTxg(5),
        };
        let before = vec![
            RecordContainer {
                identity: identity(1),
                records: vec![10_u64, 11],
            },
            RecordContainer {
                identity: identity(2),
                records: vec![12_u64],
            },
        ];
        let after = append_records_to_the_open_container(
            &before,
            &before_shape,
            &[13_u64, 14, 15],
            CheckpointTxg(9),
            TreeIdentifier(65548),
            3,
            2,
            container_tree_node_entries_capped_at(2),
        )
        .expect("最右那片的号 2，加一装得下");
        assert_eq!(after.containers.len(), 3);
        assert_eq!(after.containers[0], before[0], "装满了的第一片原样");
        assert_eq!(after.containers[1].records, vec![12, 13], "第二片装满");
        assert_eq!(
            after.containers[2].identity,
            PackedIdentity {
                birth_tree: TreeIdentifier(65548),
                record_type: 3,
                container: 3,
                container_birth: CheckpointTxg(9),
            },
            "第三片：容器号 = 最大号 + 1，出生代 = 这次的 txg"
        );
        assert_eq!(after.containers[2].records, vec![14, 15]);
        assert_eq!(after.rewritten_containers, vec![1, 2]);
        assert!(after.grew_a_level_this_publish());
        assert!(
            !after.previous_root_is_carried_down_unchanged(),
            "旧根下面的第二片重写了：旧根跟着重写"
        );
        assert_eq!(
            after.rewritten_nodes(),
            vec![position(1, 0), position(1, 1), position(2, 0)]
        );
        assert_eq!(after.first_rewritten_container(), Some(1));
        assert_eq!(
            after.shape.ancestors_of_the_rightmost_containers(1),
            vec![position(1, 1), position(2, 0)],
            "只看最右那一片时祖先就是最右那一条路径：右半 (1, 1) 与根 (2, 0)，旧根 (1, 0) 不在其内"
        );
        let carried = append_records_to_the_open_container(
            &after.containers,
            &after.shape,
            &[],
            CheckpointTxg(10),
            TreeIdentifier(65548),
            3,
            2,
            container_tree_node_entries_capped_at(2),
        )
        .expect("一条都不追加，不开新容器");
        assert!(carried.rewritten_containers.is_empty() && carried.rewritten_nodes().is_empty());
        assert_eq!(carried.first_rewritten_container(), None);
        assert_eq!(
            carried.containers, after.containers,
            "一条都不追加时整棵照抄"
        );
    }

    /// 求祖先只读最右端那几片的路径，不按容器总数开表、不逐层扫全层（`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」
    /// 运行时决策路径那一格：代价不随容器总数涨）。这份形状只有最右端那一截是真的：记着 2^20 片容器，层级 1 最右两个节点罩住最后 3 + 2 片、
    /// 层级 2 最右那个节点罩住层级 1 的最后两个、根罩住层级 2 的两个；层级 1、2 最左那个节点的孩子数写成 `usize::MAX`，故意不满足形状的不变量——
    /// 从最左读起的写法（按全部容器数开一张标记表，再逐层从最左扫全层；或从最右往左一路减到最左）一读到它就越过标记表或加减溢出、panic。
    /// 问最右那两片：每层从最右端往左读到那一段起点就停的写法交回最右那一条路径，每层的那一段起点也只到那里。
    #[test]
    fn ancestors_of_the_rightmost_containers_read_only_the_nodes_on_their_paths() {
        const CONTAINER_TOTAL: usize = 1 << 20;
        let only_the_right_end_is_readable = ContainerTreeShape {
            children_of_each_node_at_each_level: vec![
                vec![usize::MAX, 3, 2],
                vec![usize::MAX, 2],
                vec![2],
            ],
            container_total: CONTAINER_TOTAL,
        };
        assert_eq!(
            only_the_right_end_is_readable.ancestors_of_the_rightmost_containers(2),
            vec![position(1, 2), position(2, 1), position(3, 0)],
            "最右那两片的祖先：层级 1 的 (1, 2)、层级 2 的 (2, 1) 与根 (3, 0)；读不到最左那两个节点"
        );
        assert_eq!(
            only_the_right_end_is_readable.ancestor_suffix_starts_of_the_rightmost_containers(2),
            vec![2, 1, 0],
            "每层那一段从最右那条路径上的节点起"
        );
    }

    /// 对拍的参照：按全部容器数（层级 1 的孩子数逐个加起来）开一张标记表、标出给的容器，再逐层从最左扫全层，
    /// 看每个节点罩住的那一段里有没有标过的。
    fn ancestors_by_marking_every_container(
        shape: &ContainerTreeShape,
        containers: &[usize],
    ) -> Vec<ContainerTreeNodePosition> {
        let container_total: usize = shape
            .children_of_each_node_at_each_level
            .first()
            .map_or(0, |level_one| level_one.iter().sum());
        let mut marked_below = vec![false; container_total];
        for &container in containers {
            marked_below[container] = true;
        }
        let mut ancestors = Vec::new();
        for (level_index, nodes) in shape.children_of_each_node_at_each_level.iter().enumerate() {
            let level = u8::try_from(level_index + 1).expect("测试里的形状不到 255 层");
            let mut marked_here = Vec::with_capacity(nodes.len());
            let mut first_child = 0;
            for (node, &children) in nodes.iter().enumerate() {
                let is_marked = marked_below[first_child..first_child + children]
                    .iter()
                    .any(|&marked| marked);
                if is_marked {
                    ancestors.push(position(
                        level,
                        u32::try_from(node).expect("测试里的形状同层不到 2^32 个节点"),
                    ));
                }
                marked_here.push(is_marked);
                first_child += children;
            }
            marked_below = marked_here;
        }
        ancestors
    }

    /// 对拍用的一批形状：扇出 2、3 与产品扇出各开 1 到 40 片（151 份里的 120 份）；再加先按扇出 2 开 9 片、换回产品扇出接着开到 40 片的 31 份
    /// （只供测试的扇出旋钮不落盘，换回产品扇出照盘上那份形状往最右那条路径上加，左边留着扇出 2 的满节点）。
    fn shapes_for_the_ancestor_differential() -> Vec<ContainerTreeShape> {
        let mut shapes = Vec::new();
        for entries in [2, 3, 135] {
            for containers in 1..=40 {
                shapes.push(shape_after_opening(containers, entries));
            }
        }
        let mut switched_to_the_product_fanout = shape_after_opening(9, 2);
        for _ in 10..=40 {
            switched_to_the_product_fanout = switched_to_the_product_fanout
                .after_a_container_is_opened_at_the_right_end(
                    ContainerTreeNodeEntries::FromTheNodeFormat.entries_per_node(),
                );
            shapes.push(switched_to_the_product_fanout.clone());
        }
        shapes
    }

    /// 新旧实现对拍：同一批形状上，最右端 1 到 n 片（n 是容器数）各问一次，每层只读那一段后缀的求祖先与按全部容器数开标记表、逐层扫全层的
    /// 求祖先交回的位置逐项相同（集合与次序）。151 份形状各问 n 组：扇出 2、3、产品扇出各 Σ(n = 1..=40) = 820 组，换扇出那 31 份
    /// Σ(n = 10..=40) = 775 组，合计 3235 组。
    #[test]
    fn ancestors_read_from_the_right_end_equal_the_ancestors_found_by_marking_every_container() {
        let mut compared: usize = 0;
        for shape in shapes_for_the_ancestor_differential() {
            let container_total = shape.containers();
            for rightmost_count in 1..=container_total {
                let containers: Vec<usize> =
                    (container_total - rightmost_count..container_total).collect();
                assert_eq!(
                    shape.ancestors_of_the_rightmost_containers(rightmost_count),
                    ancestors_by_marking_every_container(&shape, &containers),
                    "形状 {shape:?} 上问最右端 {rightmost_count} 片"
                );
                compared += 1;
            }
        }
        assert_eq!(compared, 3_235, "对拍了 151 份形状上最右端的每一组");
    }

    /// 同一批形状上：最右端 1 到 n 片重写时，`node_origins` 按每层那一段起点判出的「这次重写」恰好是按全部容器数开标记表找出的祖先，
    /// 别的都是照抄（上一版取同一份形状，每个位置上一版都有）。一片都不重写时一个节点都不重写。合计 3235 + 151 组。
    #[test]
    fn node_origins_mark_exactly_the_ancestors_of_the_rightmost_rewritten_containers() {
        let mut compared: usize = 0;
        for shape in shapes_for_the_ancestor_differential() {
            let container_total = shape.containers();
            for rightmost_count in 0..=container_total {
                let rewritten_containers: Vec<usize> =
                    (container_total - rightmost_count..container_total).collect();
                let expected_rewritten =
                    ancestors_by_marking_every_container(&shape, &rewritten_containers);
                let after: ContainerTreeAfterThisPublish<u64> = ContainerTreeAfterThisPublish {
                    containers: (0..container_total)
                        .map(|_| RecordContainer {
                            identity: IDENTITY_OF_A_CODE_TWO_CHILD,
                            records: Vec::new(),
                        })
                        .collect(),
                    rewritten_containers,
                    node_counts_before_this_publish: shape.node_counts(),
                    shape: shape.clone(),
                };
                let origins = after.node_origins();
                assert_eq!(
                    origins.iter().map(|(node, _)| *node).collect::<Vec<_>>(),
                    shape.positions_in_bump_order(),
                    "形状 {shape:?}：按 bump 次序列全部节点"
                );
                for (node, origin) in origins {
                    let expected_origin = if expected_rewritten.contains(&node) {
                        ContainerTreeNodeOrigin::RewrittenThisPublish
                    } else {
                        ContainerTreeNodeOrigin::CarriedFromThePreviousVersion
                    };
                    assert_eq!(
                        origin, expected_origin,
                        "形状 {shape:?} 上最右端 {rightmost_count} 片重写时节点 {node:?}"
                    );
                }
                compared += 1;
            }
        }
        assert_eq!(
            compared, 3_386,
            "对拍了 151 份形状上最右端的每一组与一片都不重写的那一组"
        );
    }

    /// 同一批形状上，每一层顺着累加出的孩子区间与从这一层最左把前面几个节点的孩子数加起来的区间逐项相同。
    #[test]
    fn children_of_every_node_equal_the_prefix_sums_of_each_level() {
        for shape in shapes_for_the_ancestor_differential() {
            let children = shape.children_of_every_node();
            assert_eq!(
                children.len(),
                shape.children_of_each_node_at_each_level.len()
            );
            for (level_index, nodes) in shape.children_of_each_node_at_each_level.iter().enumerate()
            {
                let expected: Vec<Range<usize>> = (0..nodes.len())
                    .map(|node| {
                        let first_child: usize = nodes[..node].iter().sum();
                        first_child..first_child + nodes[node]
                    })
                    .collect();
                assert_eq!(
                    children[level_index], expected,
                    "形状 {shape:?} 层下标 {level_index}"
                );
            }
        }
    }

    /// 最右端开一片原地改形状、不逐层照抄：三层的形状，层级 1 有 13500 个满节点（容器 1822500 片）、层级 2 有 100 个满节点、根 100 个孩子，
    /// 开第 1822501 片时层级 1、2 末尾各加一个节点、根加一个孩子。每层的 `Vec` 都预留了多一个的余量，原地改不重新分配：开片之后每层还是原来那一块
    /// 内存。逐层照抄的写法（每层 `copy_of_one_level` 抄一份新的那种）每层都换了一块，这里就红。
    #[test]
    fn opening_a_container_changes_the_levels_in_place_without_copying_them() {
        const LEVEL_ONE_NODES: usize = 13_500;
        const LEVEL_TWO_NODES: usize = 100;
        let mut level_one = Vec::with_capacity(LEVEL_ONE_NODES + 1);
        level_one.resize(LEVEL_ONE_NODES, 135);
        let mut level_two = Vec::with_capacity(LEVEL_TWO_NODES + 1);
        level_two.resize(LEVEL_TWO_NODES, 135);
        let root = vec![LEVEL_TWO_NODES];
        let mut levels = Vec::with_capacity(4);
        levels.extend([level_one, level_two, root]);
        let shape = ContainerTreeShape::from_children_of_each_node_at_each_level(levels)
            .expect("每层孩子数之和等于下一层的节点数、根一个节点");
        let buffers_before: Vec<*const usize> = shape
            .children_of_each_node_at_each_level
            .iter()
            .map(|nodes| nodes.as_ptr())
            .collect();
        let opened = shape.after_a_container_is_opened_at_the_right_end(
            ContainerTreeNodeEntries::FromTheNodeFormat.entries_per_node(),
        );
        assert_eq!(opened.containers(), LEVEL_ONE_NODES * 135 + 1);
        assert_eq!(opened.nodes_at_level(1), LEVEL_ONE_NODES + 1);
        assert_eq!(opened.nodes_at_level(2), LEVEL_TWO_NODES + 1);
        assert_eq!(
            opened.children_of_each_node_at_each_level[2],
            vec![LEVEL_TWO_NODES + 1]
        );
        let buffers_after: Vec<*const usize> = opened
            .children_of_each_node_at_each_level
            .iter()
            .map(|nodes| nodes.as_ptr())
            .collect();
        assert_eq!(
            buffers_after, buffers_before,
            "开片之后每一层还是原来那一块内存：原地改，不逐层照抄"
        );
    }

    /// 这次之前那一版只记每层节点数：一次追加长了一层，记着的是上一版的两层（层级 1 两个节点、根一个），据它判哪个位置上一版有节点、哪个是上一版的根。
    #[test]
    fn the_version_before_this_publish_is_kept_as_the_node_count_of_each_level() {
        let before_shape = shape_after_opening(3, 2);
        let containers: Vec<RecordContainer<u64>> = (1..=3)
            .map(|number| RecordContainer {
                identity: PackedIdentity {
                    birth_tree: TreeIdentifier(65545),
                    record_type: 1,
                    container: number,
                    container_birth: CheckpointTxg(5),
                },
                records: vec![number],
            })
            .collect();
        let after = append_records_to_the_open_container(
            &containers,
            &before_shape,
            &[4_u64, 5],
            CheckpointTxg(9),
            TreeIdentifier(65545),
            1,
            1,
            container_tree_node_entries_capped_at(2),
        )
        .expect("最右那片的号 3，加一装得下");
        assert_eq!(
            after.node_counts_before_this_publish,
            ContainerTreeNodeCounts {
                nodes_at_each_level: vec![2, 1]
            }
        );
        assert_eq!(
            after.node_counts_before_this_publish,
            before_shape.node_counts()
        );
        assert!(after
            .node_counts_before_this_publish
            .has_a_node_at(position(1, 1)));
        assert!(!after
            .node_counts_before_this_publish
            .has_a_node_at(position(1, 2)));
        assert!(!after
            .node_counts_before_this_publish
            .has_a_node_at(position(3, 0)));
        assert!(after
            .node_counts_before_this_publish
            .is_the_root(position(2, 0)));
        assert_eq!(
            after.node_counts_before_this_publish.root_position(),
            Some(position(2, 0))
        );
        assert_eq!(after.rewritten_containers, vec![3, 4]);
        assert_eq!(after.first_rewritten_container(), Some(3));
        assert!(after.grew_a_level_this_publish());
    }

    /// C841：最右那片的号是 `u64::MAX`（坏镜像上读来的）。它还装得下时照常追加进它；装满了、要开新容器时交回
    /// `NextContainerNumberPastTheTopOfItsRange`、点名树与最右那片的号，不 panic。
    #[test]
    fn opening_a_container_after_one_numbered_at_the_top_of_its_range_returns_an_error() {
        let rightmost_at_the_top = RecordContainer {
            identity: PackedIdentity {
                birth_tree: TreeIdentifier(65545),
                record_type: 1,
                container: u64::MAX,
                container_birth: CheckpointTxg(5),
            },
            records: vec![10_u64],
        };
        let before = vec![rightmost_at_the_top];
        let shape_before = shape_after_opening(1, 2);
        let appended_into_the_open_container = append_records_to_the_open_container(
            &before,
            &shape_before,
            &[11_u64],
            CheckpointTxg(9),
            TreeIdentifier(65545),
            1,
            2,
            container_tree_node_entries_capped_at(2),
        )
        .expect("最右那片还装得下：不开新容器，不看它的号");
        assert_eq!(
            appended_into_the_open_container.containers[0].records,
            vec![10, 11]
        );
        assert_eq!(
            append_records_to_the_open_container(
                &before,
                &shape_before,
                &[11_u64],
                CheckpointTxg(9),
                TreeIdentifier(65545),
                1,
                1,
                container_tree_node_entries_capped_at(2),
            ),
            Err(NextContainerNumberPastTheTopOfItsRange {
                tree: TreeIdentifier(65545),
                rightmost_container: u64::MAX,
            }),
            "最右那片装满、要开新容器：号加一越过顶，交回类型化错误"
        );
    }

    /// 只供测试的扇出至少 2、至多产品扇出 135。
    #[test]
    fn the_node_entries_capped_for_tests_lie_between_two_and_the_product_fanout() {
        assert_eq!(ContainerTreeNodeEntriesCappedForTests::new(1), None);
        assert_eq!(ContainerTreeNodeEntriesCappedForTests::new(136), None);
        assert_eq!(
            ContainerTreeNodeEntriesCappedForTests::new(135).map(|capped| capped.entries()),
            Some(135)
        );
    }

    /// 这份文件 `verus!` 里写了 `assume_specification` 的格式常量：名字与 `singlefs-format` 里的真值。
    /// 新加一条 `assume_specification` 时在这里加它的名字，没配上的那一边测试照样红。
    const CONTAINER_TREE_FORMAT_CONSTANTS_WITH_A_VERUS_SPECIFICATION: [(&str, u64); 5] = [
        ("INODE_LEAF_RECORDS", singlefs_format::INODE_LEAF_RECORDS),
        (
            "TOMBSTONE_CONTAINER_RECORDS",
            singlefs_format::TOMBSTONE_CONTAINER_RECORDS,
        ),
        (
            "LINEAGE_CONTAINER_RECORDS",
            singlefs_format::LINEAGE_CONTAINER_RECORDS,
        ),
        (
            "RETIREMENT_CONTAINER_RECORDS",
            singlefs_format::RETIREMENT_CONTAINER_RECORDS,
        ),
        (
            "DELETED_INODES_ENTRY_BYTES",
            singlefs_format::DELETED_INODES_ENTRY_BYTES,
        ),
    ];

    /// 这份文件 `verus!` 里每一条 `pub assume_specification[ singlefs_format::<名> ] -> (…) returns <值>;`：名字与 `returns` 后面的值。
    /// 只读 `tests` 模块之前那一截，这里写的模式与字符串不算。
    fn format_constant_specifications_in_this_file() -> Vec<(String, u64)> {
        let source = include_str!("container_tree.rs");
        let (verified_region, _tests) = source
            .split_once("\n#[cfg(test)]\n")
            .expect("这份文件里有 tests 模块，它的 #[cfg(test)] 单独占一行");
        verified_region
            .lines()
            .map(str::trim)
            .filter_map(|line| line.strip_prefix("pub assume_specification[ singlefs_format::"))
            .map(|rest| {
                let (name, after_name) = rest
                    .split_once(" ]")
                    .expect("格式常量的 assume_specification 写成「[ singlefs_format::<名> ]」");
                let (_returned_binding, value_text) = after_name
                    .split_once(" returns ")
                    .expect("格式常量的 assume_specification 用 returns 交回值");
                let digits: String = value_text
                    .trim_end_matches(';')
                    .trim_end_matches("u64")
                    .chars()
                    .filter(|character| *character != '_')
                    .collect();
                let value = digits
                    .parse::<u64>()
                    .expect("returns 后面是一个带 u64 后缀的十进制整数");
                (name.to_owned(), value)
            })
            .collect()
    }

    #[test]
    fn every_container_tree_format_constant_specification_returns_the_value_singlefs_format_defines(
    ) {
        let specified = format_constant_specifications_in_this_file();
        let mut specified_names: Vec<&str> = specified
            .iter()
            .map(|(name, _value)| name.as_str())
            .collect();
        specified_names.sort_unstable();
        let mut listed_names: Vec<&str> =
            CONTAINER_TREE_FORMAT_CONSTANTS_WITH_A_VERUS_SPECIFICATION
                .iter()
                .map(|(name, _value)| *name)
                .collect();
        listed_names.sort_unstable();
        assert_eq!(
            specified_names, listed_names,
            "verus! 里写了 assume_specification 的格式常量与测试里列的名字要逐个相同"
        );
        for (name, specified_value) in &specified {
            let (_listed_name, actual_value) =
                CONTAINER_TREE_FORMAT_CONSTANTS_WITH_A_VERUS_SPECIFICATION
                    .iter()
                    .find(|(listed_name, _value)| listed_name == name)
                    .expect("上面判过两边名字逐个相同");
            assert_eq!(
                specified_value, actual_value,
                "singlefs_format::{name} 的 assume_specification 交回 {specified_value}，singlefs-format 里定义的是 {actual_value}"
            );
        }
    }
}
