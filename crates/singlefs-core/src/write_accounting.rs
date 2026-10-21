//! 每次发布按结构种类计的写（里程碑「覆盖写、释放、回退与复用」增补 1 第 1 件）：写调用数与写字节。
//!
//! 种类由发布路径在构造提交步骤时给出：单元写带着它的角色，journal 记录、根槽、系统配置槽由步骤成员本身定。写入口把一次写交给设备、
//! 设备报成功之后记一笔。记账不发写、不改写的次序、屏障与段序列，也不加提交步骤成员或块设备动作（D17（实现分层与第三方管道）
//! 已定项 2 / 已定项 5）。一次写调用 = 交给一块盘的一次 `write_at`：镜像的两份各算一次，与设备一层数的口径相同，
//! 所以一次发布按种类的合计要与设备一层数的逐次相等。

use std::collections::BTreeMap;

use crate::transaction::TransactionUnit;

/// 一次写调用写的是哪一种盘上结构：单元按角色分到每棵树各自的节点、树表单元、实例表单元，固定结构分 journal 记录、根槽、系统配置槽。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum WrittenStructureKind {
    DataUnit,
    ExtentTreeNode,
    InodeTreeLeafContainer,
    InodeTreeRoot,
    AllocationRecordTreeNode,
    AccountingTreeNode,
    /// deadlist 树的节点（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3：第三棵多层码 2 树）：按树数，不按层数。
    DeadlistTreeNode,
    /// `deleted_inodes` 树的根（D8（核心索引结构） 已定项 16）。
    DeletedInodesTreeNode,
    /// 墓碑树的容器与根（D18（块里携带什么信息） 已定项 24）：按树数，不按容器与根分。
    TombstoneTreeNode,
    /// 谱系树的容器与根（D18（块里携带什么信息） 已定项 24 第 5 条）：另立一种、不并进墓碑树那一种（用户 2026-10-02 定「另立一种」），按树数。
    LineageTreeNode,
    /// logged_ops 树的根（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1：单树、只有一个码 2 根）：另立一种，按树数。
    LoggedOperationsTreeNode,
    /// 退役树的容器与根（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条）：照谱系树另立一种，按树数。
    RetirementTreeNode,
    CentralMappingTreeNode,
    TreeTableUnit,
    InstanceTableUnit,
    JournalRecord,
    RootSlot,
    SystemConfigurationSlot,
}

impl WrittenStructureKind {
    /// 报数的次序：单元按 bump 次序（实例表单元在末尾，同 `TransactionOutput::units`），固定结构按持久顺序（D16（发布语义） 已定项 7）。
    /// 合计不经过这张清单（`WritesByStructureKind::total` 只从记过的账里加）：清单漏一种，报出的各种之和就对不上合计。
    pub const IN_REPORT_ORDER: [WrittenStructureKind; 18] = [
        WrittenStructureKind::DataUnit,
        WrittenStructureKind::ExtentTreeNode,
        WrittenStructureKind::InodeTreeLeafContainer,
        WrittenStructureKind::InodeTreeRoot,
        WrittenStructureKind::AllocationRecordTreeNode,
        WrittenStructureKind::AccountingTreeNode,
        WrittenStructureKind::DeadlistTreeNode,
        WrittenStructureKind::DeletedInodesTreeNode,
        WrittenStructureKind::TombstoneTreeNode,
        WrittenStructureKind::LineageTreeNode,
        WrittenStructureKind::LoggedOperationsTreeNode,
        WrittenStructureKind::RetirementTreeNode,
        WrittenStructureKind::CentralMappingTreeNode,
        WrittenStructureKind::TreeTableUnit,
        WrittenStructureKind::InstanceTableUnit,
        WrittenStructureKind::JournalRecord,
        WrittenStructureKind::RootSlot,
        WrittenStructureKind::SystemConfigurationSlot,
    ];

    /// 一个单元角色的写归哪一种。
    #[must_use]
    pub const fn of_unit(identity: TransactionUnit) -> WrittenStructureKind {
        match identity {
            TransactionUnit::Data(_) | TransactionUnit::DataOfAFileOtherThanTheFirst { .. } => {
                WrittenStructureKind::DataUnit
            }
            // 第一个文件之外的文件那一棵下段同第一个文件的下段：extent 树节点，按树数、不按文件数。
            TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. } => {
                WrittenStructureKind::ExtentTreeNode
            }
            // 按 key 空间定形状的两棵树同样按树数、不按段与层数（extent 树上段与下段、分配记录树每一层都是这一种）。
            TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot => WrittenStructureKind::ExtentTreeNode,
            TransactionUnit::InodeLeafContainer(_) => WrittenStructureKind::InodeTreeLeafContainer,
            TransactionUnit::InodeRoot => WrittenStructureKind::InodeTreeRoot,
            TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            | TransactionUnit::AllocationTree => WrittenStructureKind::AllocationRecordTreeNode,
            // 多层之后根之下的节点与根同一种：按树数，不按层数（增补 1 的账按结构种类记）。
            TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree => WrittenStructureKind::AccountingTreeNode,
            TransactionUnit::DeadlistTreeNodeBelowTheRoot(_) | TransactionUnit::DeadlistTree => {
                WrittenStructureKind::DeadlistTreeNode
            }
            TransactionUnit::DeletedInodesRoot => WrittenStructureKind::DeletedInodesTreeNode,
            TransactionUnit::TombstoneLeafContainer(_) | TransactionUnit::TombstoneRoot => {
                WrittenStructureKind::TombstoneTreeNode
            }
            TransactionUnit::LineageLeafContainer(_) | TransactionUnit::LineageRoot => {
                WrittenStructureKind::LineageTreeNode
            }
            // 根满长高之后根之下的码 2 节点与根同一种：按树数，不按层数。
            TransactionUnit::TombstoneNodeBelowTheRoot(_) => {
                WrittenStructureKind::TombstoneTreeNode
            }
            TransactionUnit::LineageNodeBelowTheRoot(_) => WrittenStructureKind::LineageTreeNode,
            TransactionUnit::LoggedOperationsRoot => WrittenStructureKind::LoggedOperationsTreeNode,
            TransactionUnit::RetirementLeafContainer(_)
            | TransactionUnit::RetirementRoot
            | TransactionUnit::RetirementNodeBelowTheRoot(_) => {
                WrittenStructureKind::RetirementTreeNode
            }
            TransactionUnit::MappingTreeNodeBelowTheRoot(_) | TransactionUnit::MappingTree => {
                WrittenStructureKind::CentralMappingTreeNode
            }
            TransactionUnit::TreeTable => WrittenStructureKind::TreeTableUnit,
            TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                WrittenStructureKind::InstanceTableUnit
            }
        }
    }

    /// 结果行里这一种的字段名前缀。
    #[must_use]
    pub const fn report_name(self) -> &'static str {
        match self {
            WrittenStructureKind::DataUnit => "data_unit",
            WrittenStructureKind::ExtentTreeNode => "extent_tree_node",
            WrittenStructureKind::InodeTreeLeafContainer => "inode_tree_leaf_container",
            WrittenStructureKind::InodeTreeRoot => "inode_tree_root",
            WrittenStructureKind::AllocationRecordTreeNode => "allocation_record_tree_node",
            WrittenStructureKind::AccountingTreeNode => "accounting_tree_node",
            WrittenStructureKind::DeadlistTreeNode => "deadlist_tree_node",
            WrittenStructureKind::DeletedInodesTreeNode => "deleted_inodes_tree_node",
            WrittenStructureKind::TombstoneTreeNode => "tombstone_tree_node",
            WrittenStructureKind::LineageTreeNode => "lineage_tree_node",
            WrittenStructureKind::LoggedOperationsTreeNode => "logged_operations_tree_node",
            WrittenStructureKind::RetirementTreeNode => "retirement_tree_node",
            WrittenStructureKind::CentralMappingTreeNode => "central_mapping_tree_node",
            WrittenStructureKind::TreeTableUnit => "tree_table_unit",
            WrittenStructureKind::InstanceTableUnit => "instance_table_unit",
            WrittenStructureKind::JournalRecord => "journal_record",
            WrittenStructureKind::RootSlot => "root_slot",
            WrittenStructureKind::SystemConfigurationSlot => "system_configuration_slot",
        }
    }
}

/// 写调用数与写字节。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriteCallsAndBytes {
    pub write_calls: u64,
    pub written_bytes: u64,
}

impl WriteCallsAndBytes {
    pub const NONE: WriteCallsAndBytes = WriteCallsAndBytes {
        write_calls: 0,
        written_bytes: 0,
    };

    #[must_use]
    pub const fn plus(self, other: WriteCallsAndBytes) -> WriteCallsAndBytes {
        WriteCallsAndBytes {
            write_calls: self.write_calls + other.write_calls,
            written_bytes: self.written_bytes + other.written_bytes,
        }
    }
}

/// 按结构种类记的写：只有写过的种类有条目。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WritesByStructureKind {
    counted: BTreeMap<WrittenStructureKind, WriteCallsAndBytes>,
}

impl WritesByStructureKind {
    /// 一笔都没记：新开的写入口，以及从盘上重建、不是这个进程写出的版本。
    pub const NOTHING_WRITTEN: WritesByStructureKind = WritesByStructureKind {
        counted: BTreeMap::new(),
    };

    /// 记一次写调用：这一种的写调用加 1、写字节加这次写的长度。
    pub fn count_write_call(&mut self, kind: WrittenStructureKind, bytes: &[u8]) {
        let entry = self.counted.entry(kind).or_insert(WriteCallsAndBytes::NONE);
        *entry = entry.plus(WriteCallsAndBytes {
            write_calls: 1,
            written_bytes: u64::try_from(bytes.len()).expect("一次写的长度装得进 u64"),
        });
    }

    /// 某一种的写；没写过是 `NONE`。
    #[must_use]
    pub fn of(&self, kind: WrittenStructureKind) -> WriteCallsAndBytes {
        self.counted
            .get(&kind)
            .copied()
            .unwrap_or(WriteCallsAndBytes::NONE)
    }

    /// 全部种类的合计：从记过的每一笔加起来，不经过 `IN_REPORT_ORDER`。
    #[must_use]
    pub fn total(&self) -> WriteCallsAndBytes {
        self.counted
            .values()
            .fold(WriteCallsAndBytes::NONE, |sum, kind_writes| {
                sum.plus(*kind_writes)
            })
    }

    /// `earlier` 之后多记的：一次发布的账 = 发布结束时的累计减去发布开始时的累计；这一段里没多写的种类不留条目。
    ///
    /// # Panics
    /// `earlier` 里某一种比这里多：它不是同一个写入口更早的快照（累计只增不减）。
    #[must_use]
    pub fn since(&self, earlier: &WritesByStructureKind) -> WritesByStructureKind {
        for (kind, earlier_writes) in &earlier.counted {
            let later_writes = self.of(*kind);
            assert!(
                later_writes.write_calls >= earlier_writes.write_calls
                    && later_writes.written_bytes >= earlier_writes.written_bytes,
                "累计只增不减：{kind:?} 的更早快照 {earlier_writes:?} 大于现在的 {later_writes:?}，传进来的不是同一个写入口更早的快照"
            );
        }
        let counted = self
            .counted
            .iter()
            .map(|(kind, later_writes)| {
                let earlier_writes = earlier.of(*kind);
                (
                    *kind,
                    WriteCallsAndBytes {
                        write_calls: later_writes.write_calls - earlier_writes.write_calls,
                        written_bytes: later_writes.written_bytes - earlier_writes.written_bytes,
                    },
                )
            })
            .filter(|(_, difference)| *difference != WriteCallsAndBytes::NONE)
            .collect();
        WritesByStructureKind { counted }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::DataUnitIndexInFile;
    use crate::inode_tree::InodeLeafContainerIndexInTree;

    #[test]
    fn since_keeps_only_the_later_writes_and_total_adds_every_counted_kind() {
        let mut cumulative = WritesByStructureKind::NOTHING_WRITTEN;
        cumulative.count_write_call(WrittenStructureKind::SystemConfigurationSlot, &[0u8; 4096]);
        cumulative.count_write_call(WrittenStructureKind::SystemConfigurationSlot, &[0u8; 4096]);
        let before_publish = cumulative.clone();
        cumulative.count_write_call(WrittenStructureKind::DataUnit, &[0u8; 32768]);
        cumulative.count_write_call(WrittenStructureKind::RootSlot, &[0u8; 512]);
        cumulative.count_write_call(WrittenStructureKind::SystemConfigurationSlot, &[0u8; 4096]);
        let publish = cumulative.since(&before_publish);
        assert_eq!(
            publish.of(WrittenStructureKind::SystemConfigurationSlot),
            WriteCallsAndBytes {
                write_calls: 1,
                written_bytes: 4096
            },
            "发布之前的两次系统配置槽写不算进这次发布"
        );
        assert_eq!(
            publish.of(WrittenStructureKind::DataUnit),
            WriteCallsAndBytes {
                write_calls: 1,
                written_bytes: 32768
            }
        );
        assert_eq!(
            publish.of(WrittenStructureKind::JournalRecord),
            WriteCallsAndBytes::NONE,
            "没写过的种类是零"
        );
        assert_eq!(
            publish.total(),
            WriteCallsAndBytes {
                write_calls: 3,
                written_bytes: 37376
            }
        );
        assert_eq!(
            cumulative.total(),
            WriteCallsAndBytes {
                write_calls: 5,
                written_bytes: 45568
            }
        );
        assert_eq!(
            before_publish.since(&before_publish),
            WritesByStructureKind::NOTHING_WRITTEN,
            "两次快照之间没写：一个条目都不留"
        );
    }

    #[test]
    fn report_order_lists_every_kind_once_so_the_reported_kinds_add_up_to_the_total() {
        let mut every_kind_once = WritesByStructureKind::NOTHING_WRITTEN;
        let mut next_length_in_bytes = 1usize;
        for identity in [
            TransactionUnit::Data(DataUnitIndexInFile::FIRST),
            TransactionUnit::ExtentRoot,
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST),
            TransactionUnit::InodeRoot,
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::DeadlistTree,
            TransactionUnit::DeletedInodesRoot,
            TransactionUnit::TombstoneRoot,
            TransactionUnit::LineageRoot,
            TransactionUnit::LoggedOperationsRoot,
            TransactionUnit::RetirementRoot,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
            TransactionUnit::InstanceTable,
        ] {
            every_kind_once.count_write_call(
                WrittenStructureKind::of_unit(identity),
                &vec![0u8; next_length_in_bytes],
            );
            next_length_in_bytes *= 2;
        }
        for fixed_structure in [
            WrittenStructureKind::JournalRecord,
            WrittenStructureKind::RootSlot,
            WrittenStructureKind::SystemConfigurationSlot,
        ] {
            every_kind_once.count_write_call(fixed_structure, &vec![0u8; next_length_in_bytes]);
            next_length_in_bytes *= 2;
        }
        for kind in WrittenStructureKind::IN_REPORT_ORDER {
            assert_eq!(
                every_kind_once.of(kind).write_calls,
                1,
                "{kind:?}：十五个单元角色各归一种、三种固定结构各一种，每种恰好一笔"
            );
        }
        let reported = WrittenStructureKind::IN_REPORT_ORDER
            .iter()
            .fold(WriteCallsAndBytes::NONE, |sum, kind| {
                sum.plus(every_kind_once.of(*kind))
            });
        assert_eq!(
            reported,
            every_kind_once.total(),
            "报数清单漏了一种或重复了一种：各种长度互不相同（2 的幂），和对不上"
        );
        assert_eq!(
            every_kind_once.total(),
            WriteCallsAndBytes {
                write_calls: 18,
                written_bytes: 262_143
            },
            "十八笔，长度 1 到 131072 各一次"
        );
        let mut names: Vec<&str> = WrittenStructureKind::IN_REPORT_ORDER
            .iter()
            .map(|kind| kind.report_name())
            .collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 18, "结果行的字段名前缀互不相同");
    }
}
