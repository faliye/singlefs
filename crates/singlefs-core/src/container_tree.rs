//! 墓碑树与谱系树共用的那一半（D18（块里携带什么信息） 已定项 24 第 1、5 条）：两棵树的根恒为码 2、条目照 inode 树内部节点的
//! 120 字节形态（分隔 key 容器号 8 + 身份引用 26 + 子指针 86），孩子是码 3 打包记录容器；容器号从 1 起、下一个号 = 树里最大号 + 1、
//! 零根时 1；容器只增不删。两棵树只差打包记录类型与一片装几条，各自的模块（`crate::tombstone_tree`、`crate::lineage_tree`）把这两样填进来。
//!
//! 根满时长高（D18（块里携带什么信息） 已定项 24 第 1、5 条「根满时长高，照 D8（核心索引结构） 已定项 6」，用户定案 K-R3-3）：
//! 码 2 节点装满时末尾分裂——左半保留全部原有条目、一个字节不动，右半从触发分裂的那条新条目起；根装满时长一层，新根 1 个码 2、
//! 分出的右半 1 个码 2（右半没有容器身份：父条目里类型段 0、容器号与出生代两段恒 0、出生树恒 0）。容器只在最右端出现，
//! 所以分裂只发生在最右那一条路径上，左边的节点一个都不重写。树的形状显式记着每个节点有几个孩子（[`ContainerTreeShape`]），
//! 不从容器数与扇出现算：只供测试的扇出旋钮不落盘（D18（块里携带什么信息） 已定项 24 第 3 条），压小扇出写出来的树换回产品扇出接着写，
//! 照着盘上那份形状往最右那条路径上加。
//!
//! 和 `crate::inode_tree` 一样，这里只算「这次发布之后树里是哪几片容器、各装哪些记录、哪几个节点要重写」，不发一个写、不动分配器、不碰落点。

use std::ops::Range;

use singlefs_format::INODE_INTERNAL_ENTRY;

use crate::address::{CheckpointTxg, TreeIdentifier};
use crate::lineage_tree::LineageContainerCapacity;
use crate::tombstone_tree::TombstoneContainerCapacity;
use crate::unit::{index_node_entry_capacity, PackedIdentity};

/// 一片码 3 容器这一版装的东西：身份四段（出生树、打包记录类型、容器号、容器出生代）与它的记录，按追加的先后。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordContainer<Record> {
    pub identity: PackedIdentity,
    pub records: Vec<Record>,
}

impl<Record> RecordContainer<Record> {
    /// 这片容器在父节点里的分隔 key：容器号（D18（块里携带什么信息） 已定项 24 第 1 条「分隔 key 容器号 8」）。
    #[must_use]
    pub fn separator_key(&self) -> u64 {
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
pub fn containers_one_container_tree_root_holds() -> usize {
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
    /// 少于 2 条或多于产品扇出交回 `None`。
    #[must_use]
    pub fn new(entries: usize) -> Option<Self> {
        (2..=containers_one_container_tree_root_holds())
            .contains(&entries)
            .then_some(Self(entries))
    }

    #[must_use]
    pub const fn entries(self) -> usize {
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
    pub fn entries_per_node(self) -> usize {
        match self {
            ContainerTreeNodeEntries::FromTheNodeFormat => {
                containers_one_container_tree_root_holds()
            }
            ContainerTreeNodeEntries::CappedForTests(capped) => capped.entries(),
        }
    }
}

/// 写入口上墓碑树与谱系树的三个容量开关（都只供测试、只住内存、不落盘）：一片墓碑容器、一片谱系容器各装满几条，一个码 2 节点装满几条。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContainerTreeCapacities {
    pub tombstone_container: TombstoneContainerCapacity,
    pub lineage_container: LineageContainerCapacity,
    pub node_entries: ContainerTreeNodeEntries,
}

impl ContainerTreeCapacities {
    /// 产品路径：三样都按格式算。
    pub const FROM_THE_FORMAT: ContainerTreeCapacities = ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
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

/// 一版容器树的形状：每一层每个码 2 节点有几个孩子，自层级 1（孩子是容器）到根（最高那一层，恰好一个节点）。零根时一层都没有。
/// 不变量：每个节点至少一个孩子；层级 1 的孩子数之和 = 容器数，层级 ℓ > 1 的孩子数之和 = 层级 ℓ − 1 的节点数；最高那一层只有一个节点。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContainerTreeShape {
    children_of_each_node_at_each_level: Vec<Vec<usize>>,
}

impl ContainerTreeShape {
    /// 零根（树表条目的根指针置零、一片容器都没有）。
    #[must_use]
    pub fn zero_root() -> Self {
        Self {
            children_of_each_node_at_each_level: Vec::new(),
        }
    }

    /// 从盘上读回来的每层每个节点的孩子数（层级 1 在前）：不满足形状的不变量交回 `None`。
    #[must_use]
    pub fn from_children_of_each_node_at_each_level(
        children_of_each_node_at_each_level: Vec<Vec<usize>>,
    ) -> Option<Self> {
        let every_node_has_a_child = children_of_each_node_at_each_level
            .iter()
            .all(|level| !level.is_empty() && level.iter().all(|children| *children >= 1));
        let each_level_counts_the_nodes_below = children_of_each_node_at_each_level
            .windows(2)
            .all(|pair| pair[1].iter().sum::<usize>() == pair[0].len());
        let the_top_level_is_one_root = children_of_each_node_at_each_level
            .last()
            .is_none_or(|top| top.len() == 1);
        let levels_fit_one_byte = u8::try_from(children_of_each_node_at_each_level.len()).is_ok();
        (every_node_has_a_child
            && each_level_counts_the_nodes_below
            && the_top_level_is_one_root
            && levels_fit_one_byte)
            .then_some(Self {
                children_of_each_node_at_each_level,
            })
    }

    /// 根节点码 2 头里的层级：零根 0，根的孩子是容器时 1，长高一次加一。
    ///
    /// # Panics
    /// 层数装不进一字节：构造时判过。
    #[must_use]
    pub fn root_level(&self) -> u8 {
        u8::try_from(self.children_of_each_node_at_each_level.len())
            .expect("形状的层数在构造时判过装得进码 2 头的一字节层级")
    }

    /// 内部层数（根的层级减一，零根与只有根时 0）：记账标签 6 的 H（D3（空间分配） 已定项 2）与 ckpt_cost 谱系那一项（D28（挂载期承诺量） 已定项 4）读它。
    #[must_use]
    pub fn internal_levels(&self) -> u64 {
        u64::from(self.root_level()).saturating_sub(1)
    }

    /// 这棵树有几片容器。
    #[must_use]
    pub fn containers(&self) -> usize {
        self.children_of_each_node_at_each_level
            .first()
            .map_or(0, |level_one| level_one.iter().sum())
    }

    /// 层级 `level` 上有几个节点（层级 0 是容器、超过根的层级 0 个）。
    #[must_use]
    pub fn nodes_at_level(&self, level: u8) -> usize {
        match level {
            0 => self.containers(),
            _ => self
                .children_of_each_node_at_each_level
                .get(usize::from(level) - 1)
                .map_or(0, Vec::len),
        }
    }

    /// 全部码 2 节点的位置，按 bump 次序（层级升序、同层从左到右，根最末）。零根时空。
    #[must_use]
    pub fn positions_in_bump_order(&self) -> Vec<ContainerTreeNodePosition> {
        self.children_of_each_node_at_each_level
            .iter()
            .enumerate()
            .flat_map(|(level_index, nodes)| {
                let level = u8::try_from(level_index + 1).expect("层数在构造时判过装得进一字节");
                (0..nodes.len()).map(move |index| ContainerTreeNodePosition {
                    level,
                    index_in_level: u32::try_from(index).expect("同层节点数装得进 u32"),
                })
            })
            .collect()
    }

    /// 根的位置（最高那一层的第 0 个）；零根时 `None`。
    #[must_use]
    pub fn root_position(&self) -> Option<ContainerTreeNodePosition> {
        (self.root_level() > 0).then_some(ContainerTreeNodePosition {
            level: self.root_level(),
            index_in_level: 0,
        })
    }

    /// 这个位置在这一版里有没有节点。
    #[must_use]
    pub fn has_a_node_at(&self, position: ContainerTreeNodePosition) -> bool {
        position.level >= 1
            && usize::try_from(position.index_in_level)
                .is_ok_and(|index| index < self.nodes_at_level(position.level))
    }

    /// 这个位置上的节点是不是这一版的根。
    #[must_use]
    pub fn is_the_root(&self, position: ContainerTreeNodePosition) -> bool {
        self.root_position() == Some(position)
    }

    /// 这个节点的孩子在下一层（层级 1 的孩子是容器，按叶序）里的下标区间。
    ///
    /// # Panics
    /// 这个位置上没有节点：调用方只拿 [`Self::positions_in_bump_order`] 里的位置问。
    #[must_use]
    pub fn children_of(&self, position: ContainerTreeNodePosition) -> Range<usize> {
        let nodes = &self.children_of_each_node_at_each_level[usize::from(position.level) - 1];
        let index = usize::try_from(position.index_in_level).expect("同层序号装得进 usize");
        let first_child: usize = nodes[..index].iter().sum();
        first_child..first_child + nodes[index]
    }

    /// 最右那一条路径上的节点（每一层最右那一个），按 bump 次序（层级 1 先、根最末）：最右那片容器被重写时 COW 的祖先就是它们。
    #[must_use]
    pub fn rightmost_path(&self) -> Vec<ContainerTreeNodePosition> {
        self.children_of_each_node_at_each_level
            .iter()
            .enumerate()
            .map(|(level_index, nodes)| ContainerTreeNodePosition {
                level: u8::try_from(level_index + 1).expect("层数在构造时判过装得进一字节"),
                index_in_level: u32::try_from(nodes.len() - 1).expect("同层节点数装得进 u32"),
            })
            .collect()
    }

    /// 最右端新开一片容器之后的形状（末尾分裂、根满长高，D8（核心索引结构） 已定项 6）：最右那个层级 1 节点装得下（孩子少于
    /// `entries_per_node`）就挂进去；装满了就在这一层最右端新开一个只有它一个孩子的节点，再往上一层给最右那个节点添这个新孩子，
    /// 一层层往上；最高那一层（根）也满了就再长一层，新根两个孩子（旧根、新开的右半）。零根时第一片容器开一个层级 1 的根。
    /// 左边的节点一个都不动。
    ///
    /// # Panics
    /// `entries_per_node` 小于 2（新根装不下两个孩子）：开关的构造函数判过。
    #[must_use]
    pub fn after_a_container_is_opened_at_the_right_end(&self, entries_per_node: usize) -> Self {
        assert!(
            entries_per_node >= 2,
            "容器树一个码 2 节点至少装 2 条：长高那次的新根要装得下左右两半"
        );
        let mut levels = self.children_of_each_node_at_each_level.clone();
        // 迭代上界是这一版的层数 + 1：每一轮往上一层，到挂得进去的那一层停；最高那一层也满了就长一层停。
        let mut level_index = 0;
        loop {
            let is_the_top_level = level_index + 1 == levels.len();
            let Some(nodes) = levels.get_mut(level_index) else {
                // 只有零根走到这里（有根时最高那一层在下面就停了）：第一片容器开一个层级 1 的根。
                levels.push(vec![1]);
                break;
            };
            let rightmost = nodes
                .last_mut()
                .expect("形状的每一层至少一个节点（构造时判过）");
            if *rightmost < entries_per_node {
                *rightmost += 1;
                break;
            }
            nodes.push(1);
            if is_the_top_level {
                // 根也满了：长一层，新根挂两个孩子（旧根、刚在最右端新开的右半）。
                levels.push(vec![2]);
                break;
            }
            level_index += 1;
        }
        Self {
            children_of_each_node_at_each_level: levels,
        }
    }
}

/// 一版容器树里一个码 2 节点从哪来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerTreeNodeOrigin {
    RewrittenThisPublish,
    /// 上一版同一个位置上的那个节点，一个字节不动（长高那次的旧根也是这一种：位置不变、只是不再是根）。
    CarriedFromThePreviousVersion,
}

/// 这次发布之后一棵容器树的样子：叶容器（按叶序）、这次重写哪几片（叶序）、这次之后与这次之前的形状。
/// 码 2 节点重写哪几个由它们推：有容器重写时最右那一条路径（COW 叶 + 祖先），没有时一个都不重写。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContainerTreeAfterThisPublish<Record> {
    pub containers: Vec<RecordContainer<Record>>,
    pub rewritten_containers: Vec<usize>,
    pub shape: ContainerTreeShape,
    pub shape_before_this_publish: ContainerTreeShape,
}

impl<Record> ContainerTreeAfterThisPublish<Record> {
    /// 这次不追加的发布：容器、形状都照抄上一版。
    #[must_use]
    pub fn carried(containers: Vec<RecordContainer<Record>>, shape: ContainerTreeShape) -> Self {
        assert_eq!(
            shape.containers(),
            containers.len(),
            "形状里层级 1 的孩子数之和就是容器数"
        );
        Self {
            containers,
            rewritten_containers: Vec::new(),
            shape_before_this_publish: shape.clone(),
            shape,
        }
    }

    /// 根跟着重写：有容器重写（COW 叶 + 祖先）。容器只增不减，没有「删空一片」那一格。
    #[must_use]
    pub fn is_the_root_rewritten(&self) -> bool {
        !self.rewritten_containers.is_empty()
    }

    /// 这次之后的内部层数（根的层级减一，零根与只有根时 0）。
    #[must_use]
    pub fn internal_levels(&self) -> u64 {
        self.shape.internal_levels()
    }

    /// 这次长了一层：上一版有根、这次之后的根层级比它高。那一次上一版的根不重写、换成根之下的节点留在原位置，不被这次换下。
    #[must_use]
    pub fn grew_a_level_this_publish(&self) -> bool {
        self.shape_before_this_publish.root_level() > 0
            && self.shape.root_level() > self.shape_before_this_publish.root_level()
    }

    /// 这次重写的码 2 节点，按 bump 次序（层级升序，根最末）。
    #[must_use]
    pub fn rewritten_nodes(&self) -> Vec<ContainerTreeNodePosition> {
        if self.is_the_root_rewritten() {
            self.shape.rightmost_path()
        } else {
            Vec::new()
        }
    }

    /// 这一版每个码 2 节点从哪来，按 bump 次序。
    ///
    /// # Panics
    /// 一个照抄的位置在上一版里没有节点：容器与节点都只在最右端新开，最右那一条路径之外的位置上一版都有。
    #[must_use]
    pub fn node_origins(&self) -> Vec<(ContainerTreeNodePosition, ContainerTreeNodeOrigin)> {
        let rewritten = self.rewritten_nodes();
        self.shape
            .positions_in_bump_order()
            .into_iter()
            .map(|position| {
                if rewritten.contains(&position) {
                    (position, ContainerTreeNodeOrigin::RewrittenThisPublish)
                } else {
                    assert!(
                        self.shape_before_this_publish.has_a_node_at(position),
                        "照抄的容器树节点 {position:?} 在上一版里有：只有最右那一条路径上的节点是新开的"
                    );
                    (position, ContainerTreeNodeOrigin::CarriedFromThePreviousVersion)
                }
            })
            .collect()
    }
}

/// 往容器树的开放容器（最右那一片）追加一条记录：最右那片还装得下（记录数小于 `records_per_container`）就追加进去、形状不变；装满了
/// （或树是空的）就开一片新容器：容器号 = 树里最大号 + 1（零根时 1），出生代 = 这次发布的 checkpoint_txg，出生树 = `tree`，
/// 打包记录类型 = `record_type`，形状照 [`ContainerTreeShape::after_a_container_is_opened_at_the_right_end`] 往上挂（根满长高）。
/// 别的容器一个字节都不动。
///
/// # Panics
/// `shape_before_this_publish` 里的容器数与 `containers_before_this_publish` 不等：两样出自同一版。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：上一版的容器与形状、这条记录、txg、树、记录类型、一片装几条、一个节点装几条"
)]
#[must_use]
pub fn append_record_to_the_open_container<Record: Clone>(
    containers_before_this_publish: &[RecordContainer<Record>],
    shape_before_this_publish: &ContainerTreeShape,
    record: Record,
    checkpoint_txg: CheckpointTxg,
    tree: TreeIdentifier,
    record_type: u16,
    records_per_container: usize,
    node_entries: ContainerTreeNodeEntries,
) -> ContainerTreeAfterThisPublish<Record> {
    assert_eq!(
        shape_before_this_publish.containers(),
        containers_before_this_publish.len(),
        "上一版容器树的形状与它的容器出自同一版"
    );
    let mut containers = containers_before_this_publish.to_vec();
    let open_container_has_room = containers
        .last()
        .is_some_and(|open| open.records.len() < records_per_container);
    let shape = if open_container_has_room {
        containers
            .last_mut()
            .expect("上一句判过最右那片在")
            .records
            .push(record);
        shape_before_this_publish.clone()
    } else {
        // 容器号沿叶序严格递增、从 1 起（0 不用），下一个号 = 树里最大号 + 1：最右那片的号就是最大号。
        let next_container_number = containers
            .last()
            .map_or(1, |rightmost| rightmost.identity.container + 1);
        containers.push(RecordContainer {
            identity: PackedIdentity {
                birth_tree: tree,
                record_type,
                container: next_container_number,
                container_birth: checkpoint_txg,
            },
            records: vec![record],
        });
        shape_before_this_publish
            .after_a_container_is_opened_at_the_right_end(node_entries.entries_per_node())
    };
    let rewritten_position = containers.len() - 1;
    ContainerTreeAfterThisPublish {
        containers,
        rewritten_containers: vec![rewritten_position],
        shape,
        shape_before_this_publish: shape_before_this_publish.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capped(entries: usize) -> ContainerTreeNodeEntries {
        ContainerTreeNodeEntries::CappedForTests(
            ContainerTreeNodeEntriesCappedForTests::new(entries).expect("2..=135"),
        )
    }

    fn shape_after_opening(containers: usize, entries: usize) -> ContainerTreeShape {
        (0..containers).fold(ContainerTreeShape::zero_root(), |shape, _| {
            shape.after_a_container_is_opened_at_the_right_end(capped(entries).entries_per_node())
        })
    }

    fn position(level: u8, index_in_level: u32) -> ContainerTreeNodePosition {
        ContainerTreeNodePosition {
            level,
            index_in_level,
        }
    }

    /// 扇出压到 2：第 1、2 片挂在层级 1 的根下；第 3 片根满、长一层（旧根留在 (1, 0)、右半 (1, 1) 一个孩子、新根 (2, 0) 两个孩子）；
    /// 第 5 片两层都满、再长一层：(1, 2) 满了开 (1, 3)、(2, 1) 也满了开 (2, 2)，根 (2, 0) 那一层原先只有两个节点、新根 (3, 0)。
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
        assert_eq!(three.children_of(position(1, 0)), 0..2);
        assert_eq!(three.children_of(position(1, 1)), 2..3);
        assert_eq!(three.children_of(position(2, 0)), 0..2);
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
        assert_eq!(five.children_of(position(3, 0)), 0..2);
        assert_eq!(five.children_of(position(2, 1)), 2..3);
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
            capped(2),
        );
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
}
