//! 退役树的叶容器（D18（块里携带什么信息） 已定项 11 打包记录类型 3；判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 2–5 条）：
//! 码 3 打包记录类型 3，一容器装 959 条 34 字节退役记录；根恒为码 2，条目照 inode 树内部节点的 120 字节形态，形态照墓碑树
//! （共用的那一半在 `crate::container_tree`）。
//!
//! 压着它的条款，逐字（判决第三节）：
//! - 「死亡 txg = 身份消失那次发布的 checkpoint_txg（删空摘掉、合并被吸收那次）」（第 3 条）；
//! - 「写者只追加：同一身份跨回退可以有多条，不去重、不覆盖」（第 4 条）；
//! - 「容器号从 1 起、下一个号 = 树里最大号 + 1；有新记录的发布才重写开放容器；满了才关；第一版不回收；根满长高」（第 5 条）。
//!
//! 一次发布的退役记录条数不设上限（第二节 K4「条数不设上界」那一行）：一次删除连着吸收几片时一次追加几条，开放容器装满就开下一片
//! （[`append_retirement_records_to_the_open_container`]）。和 `crate::tombstone_tree` 一样，这里只算「这次发布之后树里是哪几片容器、
//! 各装哪些记录、哪几片要重写」，不发一个写、不动分配器、不碰落点。

use singlefs_format::RETIREMENT_CONTAINER_RECORDS;

use crate::address::{CheckpointTxg, TreeIdentifier};
use crate::container_tree::{
    append_records_to_the_open_container, ContainerTreeAfterThisPublish, ContainerTreeNodeEntries,
    ContainerTreeShape, RecordContainer,
};
use crate::records::RetirementRecord;
use crate::unit::{PackedIdentity, PACKED_TYPE_RETIREMENT};

/// 退役树里第几片叶容器，按叶序（= 容器号升序）从 0 数。它是树里的位置，不是容器号（容器号从 1 起）。
/// 第一版不回收 ⇒ 容器只在最右端出现、一片都不删，位置一旦定下就不再挪，可以当发布路径里的角色序号用。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RetirementContainerIndexInTree(pub u32);

impl RetirementContainerIndexInTree {
    /// 当下标用。
    #[must_use]
    pub fn position(self) -> usize {
        usize::try_from(self.0).expect("u32 的退役容器序号装得进 usize")
    }

    /// 第几片，从 0 数。
    ///
    /// # Panics
    /// 序号装不进 u32：要 2^32 片退役容器（约 4 × 10^12 次叶容器退役），走不到。
    #[must_use]
    pub fn of_position(position: usize) -> Self {
        Self(u32::try_from(position).expect("退役容器片数装得进 u32：要 2^32 片才装不下"))
    }
}

/// 一片退役容器这一版装的东西：身份四段（出生树 = 退役树、打包记录类型 3、容器号、容器出生代）与它的记录，按追加的先后。
pub type RetirementContainer = RecordContainer<RetirementRecord>;

/// 只供测试压小的每片退役容器条数（`.claude/rules/fs-design.md` 五条硬要求第 2 条：「满了才关、开下一片」这一支按产品容量
/// 要 959 次叶容器退役才走得到）。至少 1、至多产品容量 959；只住内存，不落盘（同墓碑容器的旋钮）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetirementRecordsPerContainerCappedForTests(u64);

impl RetirementRecordsPerContainerCappedForTests {
    /// 0 条或多于产品容量 959 交回 `None`。
    #[must_use]
    pub fn new(records: u64) -> Option<Self> {
        (1..=RETIREMENT_CONTAINER_RECORDS)
            .contains(&records)
            .then_some(Self(records))
    }

    #[must_use]
    pub const fn records(self) -> u64 {
        self.0
    }
}

/// 一片退役容器装满几条就关：产品路径按容器格式算（959），只供测试的开关把它压小。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetirementContainerCapacity {
    FromTheContainerFormat,
    CappedForTests(RetirementRecordsPerContainerCappedForTests),
}

impl RetirementContainerCapacity {
    /// 运行时报出走的是哪一条（`.claude/rules/fs-design.md` 五条硬要求第 4 条：分支必须可观测）。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            RetirementContainerCapacity::FromTheContainerFormat => {
                "retirement_container_capacity_from_the_container_format"
            }
            RetirementContainerCapacity::CappedForTests(_) => {
                "retirement_container_capacity_capped_for_tests"
            }
        }
    }

    /// 一片装满几条。
    #[must_use]
    pub const fn records_per_container(self) -> u64 {
        match self {
            RetirementContainerCapacity::FromTheContainerFormat => RETIREMENT_CONTAINER_RECORDS,
            RetirementContainerCapacity::CappedForTests(capped) => capped.records(),
        }
    }
}

/// 这次发布之后退役树的样子：叶容器、这次重写哪几片（叶序）、这次前后的形状（`crate::container_tree`）。
pub type RetirementTreeAfterThisPublish = ContainerTreeAfterThisPublish<RetirementRecord>;

/// 这次发布要追加的退役记录：`retired_identities` 里每个身份一条，死亡 txg 都是这次发布的 checkpoint_txg，次序照给的次序
/// （调用方按叶容器在上一版里的叶序给，同一个参数两次发布逐字节相同）。不去重：写者只追加（判决第三节第 4 条）。
#[must_use]
pub fn retirement_records_of_this_publish(
    retired_identities: &[PackedIdentity],
    checkpoint_txg: CheckpointTxg,
) -> Vec<RetirementRecord> {
    retired_identities
        .iter()
        .map(|retired| RetirementRecord {
            retired: *retired,
            death_txg: checkpoint_txg,
        })
        .collect()
}

/// 往退役树的开放容器（最右那一片）追加这次发布的退役记录（删空摘掉、被左邻吸收的叶容器身份，各一条）：
/// 装得下就追加进最右那片；装满了（或零根）就开一片新容器，容器号 = 树里最大号 + 1（零根时 1），出生代 = 这次发布的 checkpoint_txg，
/// 出生树 = `retirement_tree`，打包记录类型 3；根满时长高。一条都没有时整棵照抄（这次不重写退役树）。
#[must_use]
pub fn append_retirement_records_to_the_open_container(
    containers_before_this_publish: &[RetirementContainer],
    shape_before_this_publish: &ContainerTreeShape,
    records: &[RetirementRecord],
    checkpoint_txg: CheckpointTxg,
    retirement_tree: TreeIdentifier,
    capacity: RetirementContainerCapacity,
    node_entries: ContainerTreeNodeEntries,
) -> RetirementTreeAfterThisPublish {
    append_records_to_the_open_container(
        containers_before_this_publish,
        shape_before_this_publish,
        records,
        checkpoint_txg,
        retirement_tree,
        PACKED_TYPE_RETIREMENT,
        usize::try_from(capacity.records_per_container()).expect("至多 959 条"),
        node_entries,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container_tree::{
        containers_one_container_tree_root_holds, ContainerTreeNodeEntriesCappedForTests,
    };
    use crate::unit::PACKED_TYPE_INODE;
    use singlefs_format::{TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_RETIREMENT};

    const RETIREMENT_TREE: TreeIdentifier = TreeIdentifier(TREE_IDENTIFIER_RETIREMENT);

    fn retired_leaf(container: u64, death_txg: u64) -> RetirementRecord {
        RetirementRecord {
            retired: PackedIdentity {
                birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
                record_type: PACKED_TYPE_INODE,
                container,
                container_birth: CheckpointTxg(3),
            },
            death_txg: CheckpointTxg(death_txg),
        }
    }

    fn capped(records: u64) -> RetirementContainerCapacity {
        RetirementContainerCapacity::CappedForTests(
            RetirementRecordsPerContainerCappedForTests::new(records).expect("1..=959"),
        )
    }

    /// 一路追加：第 txg 次发布追加一条，交回最后一次之后的样子。
    fn after_appending_one_per_publish(
        publishes: u64,
        capacity: RetirementContainerCapacity,
        node_entries: ContainerTreeNodeEntries,
    ) -> RetirementTreeAfterThisPublish {
        let mut after =
            RetirementTreeAfterThisPublish::carried(Vec::new(), ContainerTreeShape::zero_root());
        for txg in 1..=publishes {
            after = append_retirement_records_to_the_open_container(
                &after.containers,
                &after.shape,
                &[retired_leaf(txg * 1000, txg)],
                CheckpointTxg(txg),
                RETIREMENT_TREE,
                capacity,
                node_entries,
            );
        }
        after
    }

    /// 零根时第一条退役记录开第一片容器：容器号 1、出生代 = 这次发布的 txg、打包记录类型 3、出生树 = 退役树；
    /// 下一次发布追加进同一片（开放容器），身份不变、只重写它。
    #[test]
    fn the_first_retirement_record_opens_container_one_and_the_next_publish_appends_to_the_open_container(
    ) {
        let first = append_retirement_records_to_the_open_container(
            &[],
            &ContainerTreeShape::zero_root(),
            &[retired_leaf(234, 7)],
            CheckpointTxg(7),
            RETIREMENT_TREE,
            RetirementContainerCapacity::FromTheContainerFormat,
            ContainerTreeNodeEntries::FromTheNodeFormat,
        );
        let identity = PackedIdentity {
            birth_tree: RETIREMENT_TREE,
            record_type: PACKED_TYPE_RETIREMENT,
            container: 1,
            container_birth: CheckpointTxg(7),
        };
        assert_eq!(
            first.containers,
            vec![RetirementContainer {
                identity,
                records: vec![retired_leaf(234, 7)],
            }],
            "判决第三节第 5 条：容器号从 1 起，类型 3，出生树 = 退役树"
        );
        assert_eq!(first.rewritten_containers, vec![0]);
        assert_eq!(first.shape.root_level(), 1);
        let second = append_retirement_records_to_the_open_container(
            &first.containers,
            &first.shape,
            &[retired_leaf(467, 9)],
            CheckpointTxg(9),
            RETIREMENT_TREE,
            RetirementContainerCapacity::FromTheContainerFormat,
            ContainerTreeNodeEntries::FromTheNodeFormat,
        );
        assert_eq!(
            second.containers,
            vec![RetirementContainer {
                identity,
                records: vec![retired_leaf(234, 7), retired_leaf(467, 9)],
            }],
            "有新记录的发布重写开放容器，身份（出生代 7）不变"
        );
        assert_eq!(second.rewritten_containers, vec![0]);
        assert_eq!(second.shape, first.shape, "没开新容器：形状不变");
    }

    /// 没有新退役记录的发布整棵照抄：一片都不重写，根也不重写。
    #[test]
    fn a_publish_without_retirement_records_carries_the_whole_tree() {
        let before = after_appending_one_per_publish(
            2,
            RetirementContainerCapacity::FromTheContainerFormat,
            ContainerTreeNodeEntries::FromTheNodeFormat,
        );
        let after = append_retirement_records_to_the_open_container(
            &before.containers,
            &before.shape,
            &[],
            CheckpointTxg(3),
            RETIREMENT_TREE,
            RetirementContainerCapacity::FromTheContainerFormat,
            ContainerTreeNodeEntries::FromTheNodeFormat,
        );
        assert_eq!(after.containers, before.containers);
        assert!(!after.is_the_root_rewritten());
        assert!(after.rewritten_nodes().is_empty());
    }

    /// 开放容器满了就关、开下一片（产品容量 959，这里压到 2）：容器号 = 最大号 + 1、出生代 = 这次的 txg；满了的那片一个字节都不动。
    /// 一次发布追加两条、开放容器只剩一格时：第一条装满它，第二条开新片，两片都重写。
    #[test]
    fn a_full_open_container_is_closed_and_a_publish_with_two_records_can_span_two_containers() {
        let two = after_appending_one_per_publish(
            2,
            capped(2),
            ContainerTreeNodeEntries::FromTheNodeFormat,
        );
        let third = append_retirement_records_to_the_open_container(
            &two.containers,
            &two.shape,
            &[retired_leaf(5000, 11)],
            CheckpointTxg(11),
            RETIREMENT_TREE,
            capped(2),
            ContainerTreeNodeEntries::FromTheNodeFormat,
        );
        assert_eq!(third.containers.len(), 2, "第一片满 2 条 ⇒ 开第二片");
        assert_eq!(third.containers[0], two.containers[0], "满了的那片原样");
        assert_eq!(third.containers[1].identity.container, 2, "最大号 1 + 1");
        assert_eq!(
            third.containers[1].identity.container_birth,
            CheckpointTxg(11)
        );
        assert_eq!(third.rewritten_containers, vec![1], "只重写新开的那一片");
        let spanning = append_retirement_records_to_the_open_container(
            &third.containers,
            &third.shape,
            &[retired_leaf(6000, 12), retired_leaf(7000, 12)],
            CheckpointTxg(12),
            RETIREMENT_TREE,
            capped(2),
            ContainerTreeNodeEntries::FromTheNodeFormat,
        );
        assert_eq!(
            spanning
                .containers
                .iter()
                .map(|container| container.records.len())
                .collect::<Vec<_>>(),
            vec![2, 2, 1],
            "第二片装满、第三片开出来装一条"
        );
        assert_eq!(spanning.rewritten_containers, vec![1, 2]);
        assert_eq!(spanning.containers[2].identity.container, 3);
    }

    /// 根装满之后再开一片：长一层——产品扇出 135 片之后的第 136 片；长高那次旧根留在 (1, 0) 照抄，新开的容器、右半与新根重写。
    #[test]
    fn opening_a_container_past_what_one_root_holds_grows_the_retirement_tree_one_level() {
        assert_eq!(containers_one_container_tree_root_holds(), 135);
        let full_root = after_appending_one_per_publish(
            135,
            capped(1),
            ContainerTreeNodeEntries::FromTheNodeFormat,
        );
        assert_eq!(full_root.shape.root_level(), 1);
        let grown = append_retirement_records_to_the_open_container(
            &full_root.containers,
            &full_root.shape,
            &[retired_leaf(999_999, 136)],
            CheckpointTxg(136),
            RETIREMENT_TREE,
            capped(1),
            ContainerTreeNodeEntries::FromTheNodeFormat,
        );
        assert_eq!(grown.shape.root_level(), 2, "第 136 片：根满长一层");
        assert!(grown.grew_a_level_this_publish());
        assert!(grown.previous_root_is_carried_down_unchanged());
        assert_eq!(grown.internal_levels(), 1);
        let node_entries_of_two = ContainerTreeNodeEntries::CappedForTests(
            ContainerTreeNodeEntriesCappedForTests::new(2).expect("2..=135"),
        );
        let three = after_appending_one_per_publish(3, capped(1), node_entries_of_two);
        assert_eq!(three.shape.root_level(), 2, "扇出 2 的第 3 片长一层");
    }

    /// 写者只追加：同一身份退役两次（删 → 回退到删之前 → 再删）两条都在，不去重、不覆盖。
    #[test]
    fn the_same_identity_retired_twice_keeps_both_records() {
        let first = append_retirement_records_to_the_open_container(
            &[],
            &ContainerTreeShape::zero_root(),
            &[retired_leaf(234, 10)],
            CheckpointTxg(10),
            RETIREMENT_TREE,
            RetirementContainerCapacity::FromTheContainerFormat,
            ContainerTreeNodeEntries::FromTheNodeFormat,
        );
        let second = append_retirement_records_to_the_open_container(
            &first.containers,
            &first.shape,
            &[retired_leaf(234, 12)],
            CheckpointTxg(12),
            RETIREMENT_TREE,
            RetirementContainerCapacity::FromTheContainerFormat,
            ContainerTreeNodeEntries::FromTheNodeFormat,
        );
        assert_eq!(
            second.containers[0].records,
            vec![retired_leaf(234, 10), retired_leaf(234, 12)]
        );
    }

    /// 只供测试的容量至少 1、至多产品容量 959。
    #[test]
    fn the_capacity_capped_for_tests_lies_between_one_and_the_product_capacity() {
        assert_eq!(RetirementRecordsPerContainerCappedForTests::new(0), None);
        assert_eq!(RetirementRecordsPerContainerCappedForTests::new(960), None);
        assert_eq!(
            RetirementRecordsPerContainerCappedForTests::new(959).map(|capped| capped.records()),
            Some(959)
        );
        assert_eq!(
            retirement_records_of_this_publish(
                &[retired_leaf(1, 4).retired, retired_leaf(234, 4).retired],
                CheckpointTxg(8)
            ),
            vec![retired_leaf(1, 8), retired_leaf(234, 8)],
            "死亡 txg 都取这次发布的 txg，次序照给的次序"
        );
    }
}
