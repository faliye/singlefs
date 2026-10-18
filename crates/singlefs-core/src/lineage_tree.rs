//! 谱系树的叶容器（D18（块里携带什么信息） 已定项 24 第 5 条）：码 3 打包记录类型 5，一容器装 1359 条 24 字节谱系记录；
//! 根恒为码 2，条目同墓碑树（`crate::container_tree`）。随现行那一版、只增不删；管理员回退那次发布追加一条
//! （D23（journal 的角色与格式） 已定项 14）；容器号从 1 起，下一个号 = 树里最大号 + 1，零根时 1（规则同墓碑容器）。
//!
//! **根满长高**：根装满之后同样长一层（第 5 条「谱系树根满同样长高」），共用的那一半在 `crate::container_tree`。
//! 只供测试的两个旋钮（一片装几条 [`LineageContainerCapacity`]、一个码 2 节点装几条 `ContainerTreeNodeEntries`）只住内存、不落盘，
//! 「开下一片」与「长高」这两支按产品容量要 1359 次与 1359 × 135 次回退才走得到（`.claude/rules/fs-design.md` 五条硬要求第 2 条）。

use singlefs_format::LINEAGE_CONTAINER_RECORDS;

use crate::address::{CheckpointTxg, TreeIdentifier};
use crate::container_tree::{
    append_record_to_the_open_container, ContainerTreeAfterThisPublish, ContainerTreeNodeEntries,
    ContainerTreeShape, NextContainerNumberPastTheTopOfItsRange, RecordContainer,
};
use crate::records::LineageRecord;
use crate::unit::PACKED_TYPE_LINEAGE;

/// 谱系树里第几片叶容器，按叶序（= 容器号升序）从 0 数。它是树里的位置，不是容器号（容器号从 1 起）。
/// 只增不删 ⇒ 容器只在最右端出现，位置一旦定下就不再挪，可以当发布路径里的角色序号用。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineageContainerIndexInTree(pub u32);

impl LineageContainerIndexInTree {
    /// 当下标用。
    #[must_use]
    pub fn position(self) -> usize {
        usize::try_from(self.0).expect("u32 的谱系容器序号装得进 usize")
    }

    /// 第几片，从 0 数。
    ///
    /// # Panics
    /// 序号装不进 u32：要 2^32 片谱系容器（约 8.8 × 10^12 次回退），走不到。
    #[must_use]
    pub fn of_position(position: usize) -> Self {
        Self(u32::try_from(position).expect("谱系容器片数装得进 u32：要 2^32 片才装不下"))
    }
}

/// 一片谱系容器这一版装的东西：身份四段（出生树 = 谱系树、打包记录类型 5、容器号、容器出生代）与它的记录，按追加的先后。
pub type LineageContainer = RecordContainer<LineageRecord>;

/// 只供测试压小的每片谱系容器条数：至少 1、至多产品容量 1359；只住内存，不落盘（同墓碑容器的旋钮，D18（块里携带什么信息） 已定项 24 第 3 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineageRecordsPerContainerCappedForTests(u64);

impl LineageRecordsPerContainerCappedForTests {
    /// 0 条或多于产品容量 1359 交回 `None`。
    #[must_use]
    pub fn new(records: u64) -> Option<Self> {
        (1..=LINEAGE_CONTAINER_RECORDS)
            .contains(&records)
            .then_some(Self(records))
    }

    #[must_use]
    pub const fn records(self) -> u64 {
        self.0
    }
}

/// 一片谱系容器装满几条就关：产品路径按容器格式算（1359），只供测试的开关把它压小。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineageContainerCapacity {
    FromTheContainerFormat,
    CappedForTests(LineageRecordsPerContainerCappedForTests),
}

impl LineageContainerCapacity {
    /// 运行时报出走的是哪一条（`.claude/rules/fs-design.md` 五条硬要求第 4 条：分支必须可观测）。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            LineageContainerCapacity::FromTheContainerFormat => {
                "lineage_container_capacity_from_the_container_format"
            }
            LineageContainerCapacity::CappedForTests(_) => {
                "lineage_container_capacity_capped_for_tests"
            }
        }
    }

    /// 一片装满几条。
    #[must_use]
    pub const fn records_per_container(self) -> u64 {
        match self {
            LineageContainerCapacity::FromTheContainerFormat => LINEAGE_CONTAINER_RECORDS,
            LineageContainerCapacity::CappedForTests(capped) => capped.records(),
        }
    }
}

/// 这次发布之后谱系树的样子：叶容器、这次重写哪几片（叶序）、这次前后的形状（`crate::container_tree`）。
pub type LineageTreeAfterThisPublish = ContainerTreeAfterThisPublish<LineageRecord>;

/// 往谱系树的开放容器（最右那一片）追加一条谱系记录（管理员回退那次发布）：装得下就追加进最右那片；装满了（或零根）就开一片新容器，
/// 容器号 = 树里最大号 + 1（零根时 1），出生代 = 这次发布的 checkpoint_txg，出生树 = `lineage_tree`，打包记录类型 5；根满时长高。
///
/// # Errors
/// 要开新容器而最右那片的号（盘上读来的）已是 `u64::MAX` ⇒ [`NextContainerNumberPastTheTopOfItsRange`]（C841）。
pub fn append_lineage_record_to_the_open_container(
    containers_before_this_publish: &[LineageContainer],
    shape_before_this_publish: &ContainerTreeShape,
    record: LineageRecord,
    checkpoint_txg: CheckpointTxg,
    lineage_tree: TreeIdentifier,
    capacity: LineageContainerCapacity,
    node_entries: ContainerTreeNodeEntries,
) -> Result<LineageTreeAfterThisPublish, NextContainerNumberPastTheTopOfItsRange> {
    append_record_to_the_open_container(
        containers_before_this_publish,
        shape_before_this_publish,
        record,
        checkpoint_txg,
        lineage_tree,
        PACKED_TYPE_LINEAGE,
        usize::try_from(capacity.records_per_container()).expect("至多 1359 条"),
        node_entries,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit::PackedIdentity;
    use singlefs_format::TREE_IDENTIFIER_LINEAGE;

    const LINEAGE_TREE: TreeIdentifier = TreeIdentifier(TREE_IDENTIFIER_LINEAGE);

    /// 零根时第一条谱系记录开第一片容器：容器号 1、出生代 = 这次回退的 txg、打包记录类型 5、出生树 = 谱系树；
    /// 第二次回退追加进同一片（一片装 1359 条），只重写它、身份不变。
    #[test]
    fn the_first_lineage_record_opens_container_number_one_and_the_second_is_appended_to_it() {
        let first_record = LineageRecord {
            rollback_txg: CheckpointTxg(12),
            rollback_target_txg: CheckpointTxg(9),
            inode_number_watermark_of_the_version_rolled_back_from: 4,
        };
        let first = append_lineage_record_to_the_open_container(
            &[],
            &ContainerTreeShape::zero_root(),
            first_record,
            CheckpointTxg(12),
            LINEAGE_TREE,
            LineageContainerCapacity::FromTheContainerFormat,
            ContainerTreeNodeEntries::FromTheNodeFormat,
        )
        .expect("容器号从 1 起逐片加一，远在顶之下");
        let identity = PackedIdentity {
            birth_tree: LINEAGE_TREE,
            record_type: PACKED_TYPE_LINEAGE,
            container: 1,
            container_birth: CheckpointTxg(12),
        };
        assert_eq!(
            (first.containers.clone(), first.rewritten_containers.clone()),
            (
                vec![LineageContainer {
                    identity,
                    records: vec![first_record],
                }],
                vec![0],
            ),
            "D18 已定项 24 第 5 条：零根时容器号 1，类型 5"
        );
        let second_record = LineageRecord {
            rollback_txg: CheckpointTxg(15),
            rollback_target_txg: CheckpointTxg(12),
            inode_number_watermark_of_the_version_rolled_back_from: 4,
        };
        let second = append_lineage_record_to_the_open_container(
            &first.containers,
            &first.shape,
            second_record,
            CheckpointTxg(15),
            LINEAGE_TREE,
            LineageContainerCapacity::FromTheContainerFormat,
            ContainerTreeNodeEntries::FromTheNodeFormat,
        )
        .expect("容器号从 1 起逐片加一，远在顶之下");
        assert_eq!(
            second.containers,
            vec![LineageContainer {
                identity,
                records: vec![first_record, second_record],
            }],
            "第二条追加进开放容器，身份（出生代 12）不变"
        );
        assert_eq!(second.rewritten_containers, vec![0]);
    }

    /// 谱系记录 24 字节：T_rb、T_old、H_cur 依次各 8 字节，小端；读回逐字段相等，长度不是 24 读不出（短一字节、长一字节都不收）。
    #[test]
    fn a_lineage_record_is_twenty_four_bytes_rollback_txg_then_target_txg_then_the_watermark_rolled_back_from(
    ) {
        let record = LineageRecord {
            rollback_txg: CheckpointTxg(0x0102),
            rollback_target_txg: CheckpointTxg(0x0304),
            inode_number_watermark_of_the_version_rolled_back_from: 0x0506,
        };
        let bytes = record.to_bytes();
        assert_eq!(
            bytes,
            [
                [0x02, 0x01, 0, 0, 0, 0, 0, 0],
                [0x04, 0x03, 0, 0, 0, 0, 0, 0],
                [0x06, 0x05, 0, 0, 0, 0, 0, 0]
            ]
            .concat(),
            "T_rb 0、T_old 8、H_cur 16，各 8 字节小端"
        );
        assert_eq!(LineageRecord::parse(&bytes), Some(record));
        assert_eq!(LineageRecord::parse(&bytes[..23]), None);
        assert_eq!(
            LineageRecord::parse(&[bytes.as_slice(), &[0]].concat()),
            None
        );
    }
}
