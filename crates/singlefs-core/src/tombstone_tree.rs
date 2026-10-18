//! 墓碑树的叶容器（D18（块里携带什么信息） 已定项 24）：码 3 打包记录类型 1，一容器装 815 条 40 字节墓碑记录；
//! 根恒为码 2，条目照 inode 树内部节点的 120 字节形态（分隔 key 容器号 8 + 身份引用 26 + 子指针 86）。
//!
//! 压着它的条款，逐字：
//! - 「容器号从 1 起，下一个号 = 树里最大号 + 1，零根时 1」（已定项 24 第 1 条）；
//! - 「有新记录的那次发布才重写，不删除的发布照抄；满了才关；关闭路径加只供测试的容量旋钮，旋钮不落盘」（第 3 条）；
//! - 「第一版不回收墓碑」（第 4 条）：容器只增不减，一片都不删。
//!
//! 和 `crate::inode_tree` 一样，这里只算「这次发布之后树里是哪几片容器、各装哪些记录、哪几片要重写」，
//! 不发一个写、不动分配器、不碰落点，几条纪律是纯函数的后置条件，单测判得了。
//!
//! **根满长高**：根装满之后长一层（D8（核心索引结构） 已定项 6、用户定案 K-R3-3），共用的那一半在 `crate::container_tree`；
//! 只供测试的扇出旋钮（`crate::container_tree::ContainerTreeNodeEntries`）与这里的容器容量旋钮一样不落盘。

use singlefs_format::TOMBSTONE_CONTAINER_RECORDS;

use crate::address::{CheckpointTxg, TreeIdentifier};
use crate::container_tree::{
    append_record_to_the_open_container, ContainerTreeAfterThisPublish, ContainerTreeNodeEntries,
    ContainerTreeShape, NextContainerNumberPastTheTopOfItsRange, RecordContainer,
};
use crate::records::TombstoneRecord;
use crate::unit::PACKED_TYPE_TOMBSTONE;

pub use crate::container_tree::CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES;

/// 墓碑树里第几片叶容器，按叶序（= 容器号升序）从 0 数。它是树里的位置，不是容器号：容器号从 1 起、叶序从 0 起。
/// 第一版不回收墓碑 ⇒ 容器只在最右端出现、一片都不删，位置一旦定下就不再挪，可以当发布路径里的角色序号用。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TombstoneContainerIndexInTree(pub u32);

impl TombstoneContainerIndexInTree {
    /// 当下标用。
    #[must_use]
    pub fn position(self) -> usize {
        usize::try_from(self.0).expect("u32 的墓碑容器序号装得进 usize")
    }

    /// 第几片，从 0 数。
    ///
    /// # Panics
    /// 序号装不进 u32：要 2^32 片墓碑容器（约 3.5 × 10^12 次 unlink），走不到。
    #[must_use]
    pub fn of_position(position: usize) -> Self {
        Self(u32::try_from(position).expect("墓碑容器片数装得进 u32：要 2^32 片才装不下"))
    }
}

/// 一片墓碑容器这一版装的东西：身份四段（出生树 = 墓碑树、打包记录类型 1、容器号、容器出生代）与它的记录，按追加的先后。
pub type TombstoneContainer = RecordContainer<TombstoneRecord>;

/// 只供测试压小的每片容器条数（`.claude/rules/fs-design.md` 五条硬要求第 2 条：「满了才关、开下一片」这一支按产品容量
/// 要 815 次 unlink 才走得到）。至少 1、至多产品容量 815；只住内存，不落盘（D18（块里携带什么信息） 已定项 24 第 3 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TombstoneRecordsPerContainerCappedForTests(u64);

impl TombstoneRecordsPerContainerCappedForTests {
    /// 0 条（容器永远装不下一条）或多于产品容量 815（一片 32 KiB 装不下）交回 `None`。
    #[must_use]
    pub fn new(records: u64) -> Option<Self> {
        (1..=TOMBSTONE_CONTAINER_RECORDS)
            .contains(&records)
            .then_some(Self(records))
    }

    #[must_use]
    pub const fn records(self) -> u64 {
        self.0
    }
}

/// 一片墓碑容器装满几条就关：产品路径按容器格式算（815），只供测试的开关把它压小。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TombstoneContainerCapacity {
    FromTheContainerFormat,
    CappedForTests(TombstoneRecordsPerContainerCappedForTests),
}

impl TombstoneContainerCapacity {
    /// 运行时报出走的是哪一条（`.claude/rules/fs-design.md` 五条硬要求第 4 条：分支必须可观测）。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            TombstoneContainerCapacity::FromTheContainerFormat => {
                "tombstone_container_capacity_from_the_container_format"
            }
            TombstoneContainerCapacity::CappedForTests(_) => {
                "tombstone_container_capacity_capped_for_tests"
            }
        }
    }

    /// 一片装满几条。
    #[must_use]
    pub const fn records_per_container(self) -> u64 {
        match self {
            TombstoneContainerCapacity::FromTheContainerFormat => TOMBSTONE_CONTAINER_RECORDS,
            TombstoneContainerCapacity::CappedForTests(capped) => capped.records(),
        }
    }
}

/// 这次发布之后墓碑树的样子：叶容器、这次重写哪几片（叶序）、这次前后的形状（`crate::container_tree`）。
pub type TombstoneTreeAfterThisPublish = ContainerTreeAfterThisPublish<TombstoneRecord>;

/// 往墓碑树的开放容器（最右那一片）追加一条墓碑（意图那次发布，D18（块里携带什么信息） 已定项 24 第 1、3 条）：
/// 最右那片还装得下（记录数小于 `capacity`）就追加进去；装满了（或树是空的）就开一片新容器：
/// 容器号 = 树里最大号 + 1（零根时 1），出生代 = 这次发布的 checkpoint_txg，出生树 = `tombstone_tree`，打包记录类型 1；
/// 根满时长高（`node_entries` 是一个码 2 节点装满几条）。别的容器一个字节都不动。
///
/// # Errors
/// 要开新容器而最右那片的号（盘上读来的）已是 `u64::MAX` ⇒ [`NextContainerNumberPastTheTopOfItsRange`]（C841）。
pub fn append_tombstone_to_the_open_container(
    containers_before_this_publish: &[TombstoneContainer],
    shape_before_this_publish: &ContainerTreeShape,
    record: TombstoneRecord,
    checkpoint_txg: CheckpointTxg,
    tombstone_tree: TreeIdentifier,
    capacity: TombstoneContainerCapacity,
    node_entries: ContainerTreeNodeEntries,
) -> Result<TombstoneTreeAfterThisPublish, NextContainerNumberPastTheTopOfItsRange> {
    append_record_to_the_open_container(
        containers_before_this_publish,
        shape_before_this_publish,
        record,
        checkpoint_txg,
        tombstone_tree,
        PACKED_TYPE_TOMBSTONE,
        usize::try_from(capacity.records_per_container()).expect("至多 815 条"),
        node_entries,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::InodeNumber;
    use crate::container_tree::{
        containers_one_container_tree_root_holds, ContainerTreeNodeEntriesCappedForTests,
        ContainerTreeNodePosition,
    };
    use crate::unit::PackedIdentity;
    use singlefs_format::TREE_IDENTIFIER_TOMBSTONE;

    const TOMBSTONE_TREE: TreeIdentifier = TreeIdentifier(TREE_IDENTIFIER_TOMBSTONE);

    fn tombstone(inode: u64, death_txg: u64) -> TombstoneRecord {
        TombstoneRecord::of_the_whole_object(
            InodeNumber(inode),
            CheckpointTxg(3),
            CheckpointTxg(death_txg),
        )
    }

    fn capped(records: u64) -> TombstoneContainerCapacity {
        TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(records).expect("1..=815"),
        )
    }

    /// 一路追加墓碑：每条 txg 取它的序号。交回最后一次追加之后的样子。
    fn after_appending(
        tombstones: u64,
        capacity: TombstoneContainerCapacity,
        node_entries: ContainerTreeNodeEntries,
    ) -> TombstoneTreeAfterThisPublish {
        let mut after =
            TombstoneTreeAfterThisPublish::carried(Vec::new(), ContainerTreeShape::zero_root());
        for txg in 1..=tombstones {
            after = append_tombstone_to_the_open_container(
                &after.containers,
                &after.shape,
                tombstone(txg + 1, txg),
                CheckpointTxg(txg),
                TOMBSTONE_TREE,
                capacity,
                node_entries,
            )
            .expect("容器号从 1 起逐片加一，远在顶之下");
        }
        after
    }

    /// 零根时第一条墓碑开第一片容器：容器号 1（不是 0）、出生代 = 这次发布的 txg、打包记录类型 1、出生树 = 墓碑树。
    #[test]
    fn the_first_tombstone_opens_container_number_one_born_in_this_publish() {
        let after = append_tombstone_to_the_open_container(
            &[],
            &ContainerTreeShape::zero_root(),
            tombstone(2, 7),
            CheckpointTxg(7),
            TOMBSTONE_TREE,
            TombstoneContainerCapacity::FromTheContainerFormat,
            ContainerTreeNodeEntries::FromTheNodeFormat,
        )
        .expect("容器号从 1 起逐片加一，远在顶之下");
        assert_eq!(
            after.containers,
            vec![TombstoneContainer {
                identity: PackedIdentity {
                    birth_tree: TOMBSTONE_TREE,
                    record_type: PACKED_TYPE_TOMBSTONE,
                    container: 1,
                    container_birth: CheckpointTxg(7),
                },
                records: vec![tombstone(2, 7)],
            }],
            "D18 已定项 24 第 1 条：零根时容器号 1"
        );
        assert_eq!(after.rewritten_containers, vec![0]);
        assert_eq!(after.shape.root_level(), 1, "第一片容器开一个层级 1 的根");
    }

    /// 开放容器还装得下：追加进最右那一片、身份不变、只重写它。
    #[test]
    fn a_tombstone_is_appended_to_the_open_container_while_it_has_room() {
        let first = after_appending(1, capped(2), ContainerTreeNodeEntries::FromTheNodeFormat);
        let after = append_tombstone_to_the_open_container(
            &first.containers,
            &first.shape,
            tombstone(3, 9),
            CheckpointTxg(9),
            TOMBSTONE_TREE,
            capped(2),
            ContainerTreeNodeEntries::FromTheNodeFormat,
        )
        .expect("容器号从 1 起逐片加一，远在顶之下");
        assert_eq!(after.containers.len(), 1, "压到 2 条一片：第二条还装得下");
        assert_eq!(
            after.containers[0].identity, first.containers[0].identity,
            "COW 重写身份不变：出生代仍是开它那次的 1"
        );
        assert_eq!(
            after.containers[0].records,
            vec![tombstone(2, 1), tombstone(3, 9)]
        );
        assert_eq!(after.rewritten_containers, vec![0]);
        assert_eq!(after.shape, first.shape, "没开新容器：形状不变");
    }

    /// 开放容器满了就关、开下一片：容器号 = 最大号 + 1、出生代 = 这次的 txg；满了的那片一个字节都不动、不进重写清单。
    #[test]
    fn a_full_open_container_is_closed_and_the_next_one_takes_the_largest_number_plus_one() {
        let two = after_appending(2, capped(2), ContainerTreeNodeEntries::FromTheNodeFormat);
        let after = append_tombstone_to_the_open_container(
            &two.containers,
            &two.shape,
            tombstone(4, 11),
            CheckpointTxg(11),
            TOMBSTONE_TREE,
            capped(2),
            ContainerTreeNodeEntries::FromTheNodeFormat,
        )
        .expect("容器号从 1 起逐片加一，远在顶之下");
        assert_eq!(after.containers.len(), 2, "第一片满 2 条 ⇒ 开第二片");
        assert_eq!(after.containers[0], two.containers[0], "满了的那片原样");
        assert_eq!(after.containers[1].identity.container, 2, "最大号 1 + 1");
        assert_eq!(
            after.containers[1].identity.container_birth,
            CheckpointTxg(11)
        );
        assert_eq!(after.containers[1].records, vec![tombstone(4, 11)]);
        assert_eq!(after.rewritten_containers, vec![1], "只重写新开的那一片");
    }

    /// 根装满之后再开一片：长一层（D8 已定项 6、用户定案 K-R3-3）——产品扇出 135 片之后的第 136 片；扇出压到 2 时第 3 片。
    /// 长高那次旧根留在 (层级 1, 第 0 个) 照抄，新开的容器、右半 (1, 1) 与新根 (2, 0) 重写。
    #[test]
    fn opening_a_container_past_what_one_root_holds_grows_the_tree_one_level() {
        let capacity_of_the_root = containers_one_container_tree_root_holds();
        assert_eq!(capacity_of_the_root, 135, "与 inode 树根同一形态：扇出 135");
        let full_root =
            after_appending(135, capped(1), ContainerTreeNodeEntries::FromTheNodeFormat);
        assert_eq!(full_root.shape.root_level(), 1);
        let grown = append_tombstone_to_the_open_container(
            &full_root.containers,
            &full_root.shape,
            tombstone(1000, 136),
            CheckpointTxg(136),
            TOMBSTONE_TREE,
            capped(1),
            ContainerTreeNodeEntries::FromTheNodeFormat,
        )
        .expect("容器号从 1 起逐片加一，远在顶之下");
        assert_eq!(grown.shape.root_level(), 2, "第 136 片：根满长一层");
        assert!(grown.grew_a_level_this_publish());
        let node_entries_of_two = ContainerTreeNodeEntries::CappedForTests(
            ContainerTreeNodeEntriesCappedForTests::new(2).expect("2..=135"),
        );
        let three = after_appending(3, capped(1), node_entries_of_two);
        assert_eq!(
            three.rewritten_nodes(),
            vec![
                ContainerTreeNodePosition {
                    level: 1,
                    index_in_level: 1
                },
                ContainerTreeNodePosition {
                    level: 2,
                    index_in_level: 0
                },
            ],
            "扇出 2 的第 3 片：重写右半与新根，旧根照抄"
        );
        assert_eq!(three.internal_levels(), 1, "长高之后内部层数 1");
    }

    /// 只供测试的容量至少 1、至多产品容量 815。
    #[test]
    fn the_capacity_capped_for_tests_lies_between_one_and_the_product_capacity() {
        assert_eq!(TombstoneRecordsPerContainerCappedForTests::new(0), None);
        assert_eq!(TombstoneRecordsPerContainerCappedForTests::new(816), None);
        assert_eq!(
            TombstoneRecordsPerContainerCappedForTests::new(815).map(|capped| capped.records()),
            Some(815)
        );
    }
}
