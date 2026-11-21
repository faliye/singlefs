//! 理想模型与实现之间那层胶水（里程碑「覆盖写、释放、回退与复用」增补 3 第 2 件）：把实现交回的东西（`singlefs_core` 的类型）换成模型的观测，
//! 把实现的错误成员映射到模型的拒绝理由。D13（验证路线） 已定项 5 管的是模型本身（`model.rs` 只 `use singlefs_format`）；
//! 拿实现结局与模型比的这一层可以用 core 的类型。
//!
//! 拒因映射只读成员与它的判别字段、不看给人看的文字；盘上字节的观测由验证侧独立解析，不调 core 的编解码器：成员说得出条款理由的映射过去，
//! I/O、盘坏、走读失败、第一版不支持的池形状（小盘写满、各盘落点不一致）这类模型里没有的一律 `Unexplained`（不建崩溃与设备错、
//! 两块等大盘的历史里它们都不该出现）。

use std::collections::{BTreeMap, BTreeSet};

use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::{AllocationRecord, PlacementRefusal};
use singlefs_core::extent_tree::TailOfAFile;
use singlefs_core::instance_table::InstanceTablePageIndex;
use singlefs_core::logged_operations::{CompactionRegion, LoggedOperation};
use singlefs_core::mount::{
    InstanceRow, MountError, Mounted, RollbackCandidateExclusion, RollbackError,
    StillUnreadableAfterOneReread, Unmounted,
};
use singlefs_core::pointer::{LocationEntry, NodePointer};
use singlefs_core::records::{
    DeadlistEntry, NLINK_OF_AN_INODE_BEING_DELETED, TREE_KIND_EXTENT, TREE_KIND_INODE,
    TREE_KIND_SNAPSHOT_EXTENT_ROOT, TREE_KIND_SNAPSHOT_INODE_ROOT,
};
use singlefs_core::recovery::{
    RecoveryFailure, RecoveryOutcome, RecoveryReport, SnapshotFileReading,
};
use singlefs_core::retirement_tree::RetirementContainerIndexInTree;
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::RootRingSlot;
use singlefs_core::tombstone_tree::TombstoneContainerIndexInTree;
use singlefs_core::transaction::{
    AllocationRecordsOfTheVersionWithoutFile, PoolVersion, PublishError, PublishedUnit,
    SnapshotCreationRefusal, TransactionOutput, TransactionUnit, TruncationRefusal,
    VersionWithoutFilePublishOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::UNIT_CLASS_INDEX_NODE;

use crate::model::{
    ModelCheckpointTxg, ModelCompactionIntent, ModelCompactionRegion, ModelDeadlistEntry,
    ModelDeviceIdentity, ModelExtentUpperNodePosition, ModelInstanceGeneration, ModelInstanceRow,
    ModelJournalCounter, ModelRefusalReason, ModelRingPosition, ModelRootKey,
    ModelUnitCopyLocation, ModelUnitRole, ModelUserVisibleRootUnit, ModelUserVisibleTree,
    ObservedAllocationRecord, ObservedAllocationRecordsOfEveryUnit, ObservedEffect, ObservedFile,
    ObservedInstanceTable, ObservedReadBack, ObservedRefusalReason, ObservedRoot, ObservedSnapshot,
    ObservedSnapshotReadBack, ObservedSnapshotState, ObservedTruncationField,
    ObservedUnitAllocationRecords,
};

mod refusal_reasons_of_errors;

pub use self::refusal_reasons_of_errors::*;

#[must_use]
pub fn model_instance(instance: InstanceGeneration) -> ModelInstanceGeneration {
    ModelInstanceGeneration(u64::from(instance.0))
}

#[must_use]
pub fn model_txg(checkpoint_txg: CheckpointTxg) -> ModelCheckpointTxg {
    ModelCheckpointTxg(checkpoint_txg.0)
}

#[must_use]
pub fn model_root_key(instance: InstanceGeneration, checkpoint_txg: CheckpointTxg) -> ModelRootKey {
    ModelRootKey {
        checkpoint_txg: model_txg(checkpoint_txg),
        instance: model_instance(instance),
    }
}

/// 实现的一个单元是模型的哪个角色。不对应角色（`None`）的几种各臂写着为什么；分配记录树根之下的节点是这样：D8（核心索引结构） 已定项 14 按绝对槽号按位置寻址，
/// 一次发布重写根之下哪几片由这次动了哪些槽定，模型不记落点、不记是哪几片（占槽只取上界）；那几片的分配代、位置与记录归池级 checker 判
/// （I-1.1 位置、I-3.10 每个节点都有记录）。
#[must_use]
pub fn model_unit_role(unit: TransactionUnit) -> Option<ModelUnitRole> {
    let role = match unit {
        // 文件的每个数据单元在「这次重写了哪几个角色」里都是这一个角色；分配记录不走这里，按单元号逐个交
        // （`allocation_records_of_every_role` 的 `data_units`，判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K4：
        // 截断只重写尾单元之后各单元的分配代不同）。
        TransactionUnit::Data(_) => ModelUnitRole::Data,
        // extent 树上段的根（D8（核心索引结构） 已定项 14）；一个单元的文件那个指针内嵌在它的条目里，多个单元的文件条目指着下段根。
        TransactionUnit::ExtentRoot => ModelUnitRole::ExtentRoot,
        // 下段的节点不对应角色（同分配记录树根之下的节点）：一次删除批只重写被截掉那一片下段叶与它的祖先、别的照抄，
        // 一版里下段各节点的分配代不止一个，模型不记是哪几片；它们的分配代、位置与记录归池级 checker 判，占槽模型按层数现算上界。
        TransactionUnit::ExtentLowerNode(_) => return None,
        // 第一个文件之外的文件（按 inode 号写数据写出来的，里程碑「删除与多次 COW」一·1）：数据单元按 (inode 号, 单元号) 另交
        // （`allocation_records_of_every_role` 的 `data_units_of_the_files_other_than_the_first`），下段节点同第一个文件的下段不对应角色。
        TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
        | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. } => return None,
        // 上段根之下的节点（写的号落在上段第 0 片叶之外时上段长层）按位置各是一个角色：形状只由有条目的 inode 号集合定，一次发布重写哪几个、
        // 照抄哪几个由模型自己的规范形计划答（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2、M4），逐个比分配代。
        TransactionUnit::ExtentUpperNodeBelowTheRoot(position) => {
            ModelUnitRole::ExtentUpperNodeBelowTheRoot(ModelExtentUpperNodePosition {
                level: u64::from(position.level),
                index_in_level: position.index,
            })
        }
        // inode 树的每片叶容器按它在树里左起的位置各是一个角色（最左那片是 `InodeLeaf`，之后每片各一个）：删完第一个文件之后
        // 一次建一个 inode（`history.rs` 的 `HistoryOperation::CreateInode`），满 233 条在末尾分裂、左半不动（D8（核心索引结构） 已定项 6），
        // 各片的分配代不同，模型逐片记、逐片比。位置是实现发布路径里的角色序号（`inode_tree::InodeLeafContainerIndexInTree`）。
        TransactionUnit::InodeLeafContainer(index) => {
            ModelUnitRole::inode_leaf_container_at(u64::from(index.0))
        }
        TransactionUnit::InodeRoot => ModelUnitRole::InodeRoot,
        TransactionUnit::AllocationTree => ModelUnitRole::AllocationTree,
        TransactionUnit::AllocationTreeNodeBelowTheRoot(_) => return None,
        // 记账树根之下的节点不对应角色（同中央映射树根之下的节点）：inode 号水位的台阶行多到一片叶装不下时（两块盘第 463 级台阶起，
        // D5（快照 / 空间记账机制） 已定项 2 射程）记账树从中间切开、长出根之下的节点（D8（核心索引结构） 已定项 11）；哪几片这次重写由
        // 这次删插了哪些 key 定，模型不记 key、不记是哪几片。它们的分配代、位置与记录归池级 checker 判，占槽与映射条目模型沿来路带
        // 节点数的上界（`model::accounting_tree_nodes_upper_bound`）。
        TransactionUnit::AccountingTreeNodeBelowTheRoot(_) => return None,
        TransactionUnit::AccountingTree => ModelUnitRole::AccountingTree,
        // deadlist 树（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）：只在有快照时有节点，模型不记它的落点与分配代
        // （同记账树根之下的节点）；它们的分配代、位置与记录归池级 checker 判。
        TransactionUnit::DeadlistTreeNodeBelowTheRoot(_) | TransactionUnit::DeadlistTree => {
            return None
        }
        // 中央映射树根之下的节点不对应角色（同分配记录树根之下的节点）：每个数据单元一条映射条目，数据单元加别的进映射的单元
        // 多于一片叶装的 294 条时（随机历史里是跨三片下段叶的那一档），映射树从中间切开、长出根之下的节点（D8（核心索引结构） 已定项 11）；
        // 哪几片这次重写由这次删插了哪些 key 定，模型不记 key、不记是哪几片。它们的分配代、位置与记录归池级 checker 判，
        // 占槽模型沿来路带节点数的上界（`model::central_mapping_tree_nodes_upper_bound`）。
        TransactionUnit::MappingTreeNodeBelowTheRoot(_) => return None,
        TransactionUnit::MappingTree => ModelUnitRole::MappingTree,
        TransactionUnit::TreeTable => ModelUnitRole::TreeTable,
        // 删除那两棵树（里程碑三·2 第 8 条）：`deleted_inodes` 至多一条 key（会话每次用户改动先把队列走空、unlink 一次调用走完全部批，D8（核心索引结构） 已定项 16 第 6 条；号可以是任一 inode），根兼叶一个节点；墓碑一次 unlink 一条、
        // 一片容器装几百条，模型罩的历史里墓碑树恒只有容器 1 那一片与根兼索引的根，根之下的节点只在只供测试的容量旋钮压小时出现。
        TransactionUnit::DeletedInodesRoot => ModelUnitRole::DeletedInodesRoot,
        TransactionUnit::TombstoneLeafContainer(index) => {
            assert_eq!(
                index,
                TombstoneContainerIndexInTree(0),
                "模型今天只罩墓碑树一片容器"
            );
            ModelUnitRole::TombstoneLeaf
        }
        TransactionUnit::TombstoneRoot => ModelUnitRole::TombstoneRoot,
        TransactionUnit::TombstoneNodeBelowTheRoot(position) => {
            panic!("模型今天只罩墓碑树一片容器（根之下没有节点）：{position:?}")
        }
        // 谱系树（管理员回退那次追加一条，D18（块里携带什么信息） 已定项 24 第 5 条）：模型的回退不记谱系容器与谱系树根的落点与分配代，
        // 同分配记录树根之下的节点，不对应角色；它们的分配代、位置与记录归池级 checker 判。
        TransactionUnit::LineageLeafContainer(_)
        | TransactionUnit::LineageRoot
        | TransactionUnit::LineageNodeBelowTheRoot(_) => return None,
        // logged_ops 根（整理意图，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1）：模型罩的历史里至多一条意图、根兼叶一个节点，
        // 签发那次写、之后照抄、删意图那次没了，模型逐版记它的分配代（= 签发那次的 txg）。
        TransactionUnit::LoggedOperationsRoot => ModelUnitRole::LoggedOperationsRoot,
        // 退役树（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条，照墓碑树）：模型罩的历史里只有删第一个文件那次最后一批
        // 让唯一那片叶容器身份消失，退役树恒只有容器 1 那一片与根（回退之后再删一次追加进同一片）。
        TransactionUnit::RetirementLeafContainer(index) => {
            assert_eq!(
                index,
                RetirementContainerIndexInTree(0),
                "模型今天只罩退役树一片容器"
            );
            ModelUnitRole::RetirementLeaf
        }
        TransactionUnit::RetirementRoot => ModelUnitRole::RetirementRoot,
        TransactionUnit::RetirementNodeBelowTheRoot(position) => {
            panic!("模型今天只罩退役树一片容器（根之下没有节点）：{position:?}")
        }
        // 模型把实例表链当一个角色：各片的落点与分配记录按这次之后的行数现算片数（`model::instance_table_pages_for_rows`）。
        TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
            ModelUnitRole::InstanceTable
        }
    };
    Some(role)
}

#[must_use]
pub fn model_instance_row(row: &InstanceRow) -> ModelInstanceRow {
    ModelInstanceRow {
        instance: model_instance(row.instance),
        selected_root_txg: model_txg(row.selected_root_txg),
        applied_transaction_high_water: row.applied_transaction_high_water,
    }
}

/// 带文件的一版：根的身份、jsn、F，这一版的文件内容与整张实例表（从 `units` 里那几个单元的字节解出来），
/// 和这一版每个角色的单元在这一版分配记录里的那几条（按槽号找，每块盘一条）。
#[must_use]
pub fn observed_root_of_file_version(output: &TransactionOutput) -> ObservedRoot {
    ObservedRoot {
        key: model_root_key(output.root.instance, output.root.checkpoint_txg),
        journal_counter: ModelJournalCounter(output.record.counter),
        rollback_floor: model_txg(output.root.rollback_floor),
        file: observed_file_of_file_version(output),
        instance_table: observed_instance_table_of_file_version(output),
        unit_allocation_records: ObservedUnitAllocationRecords::EveryRoleOfTheVersion(
            allocation_records_of_every_role(
                &output.units,
                &output.allocation_records,
                &output.root,
            ),
        ),
        snapshot_state: observed_snapshot_state_of_file_version(output),
        logged_compaction_intent: observed_logged_compaction_intent(output),
        first_file_record_truncation_field: observed_truncation_field_of_file_version(output),
        inodes_other_than_the_first: observed_inodes_other_than_the_first_of_file_version(output),
        truncation_fields_of_the_inodes_other_than_the_first:
            observed_truncation_fields_of_the_inodes_other_than_the_first_of_file_version(output),
    }
}

/// 这一版第一个文件之外的每个活着的 inode（nlink 不是 0）读回的内容（里程碑「删除与多次 COW」一·1）：那个文件的数据单元
/// （`TransactionUnit::DataOfAFileOtherThanTheFirst`）按单元号接起来，各按单元头里的声明长度取载荷，按上段叶条目的标签与 logged_ops 里的
/// 截断条目读（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S5「胶水按标签与截断条目读别的文件」）：
/// 单元号要从 0 起连着、每个号恰好一个；标签 1、2（尾部没有洞）的头 ⌈size ÷ 净荷⌉.max(1) 个接起来恰好 size 字节，多出来的只许是一条对得上的
/// 截断条目（号、对象出生代、目标 = size）点名的截断中间态、读者不读；标签 3、4 的单元都在 size 之内、个个满净荷，之后到 size 读 0；
/// 标签 0（上段有条目、一个单元都没有）整个文件读 0。上段里没有它的条目的要 size 0（读回空）。
#[must_use]
pub fn observed_inodes_other_than_the_first_of_file_version(
    output: &TransactionOutput,
) -> BTreeMap<u64, ObservedFile> {
    let mut data_units_by_inode: BTreeMap<u64, Vec<(u64, &[u8])>> = BTreeMap::new();
    for unit in &output.units {
        if let TransactionUnit::DataOfAFileOtherThanTheFirst { inode, unit: index } = unit.identity
        {
            data_units_by_inode
                .entry(inode.0)
                .or_default()
                .push((index.0, &unit.bytes[..]));
        }
    }
    let tail_of_the_entry: BTreeMap<u64, TailOfAFile> = output
        .files_other_than_the_first
        .iter()
        .map(|file| (file.inode.0, file.tail))
        .collect();
    let truncation_intent =
        singlefs_core::logged_operations::live_truncation_intent_of_the_entries(
            &output.logged_operations,
        )
        .map(|(_, intent)| intent);
    output
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .filter(|record| {
            record.inode != FIRST_INODE_NUMBER && record.nlink != NLINK_OF_AN_INODE_BEING_DELETED
        })
        .map(|record| {
            let mut data_units = data_units_by_inode
                .remove(&record.inode)
                .unwrap_or_default();
            data_units.sort_by_key(|(index, _)| *index);
            let names_a_truncation_intent = truncation_intent.is_some_and(|intent| {
                intent.inode.0 == record.inode
                    && intent.object_birth == record.object_birth
                    && intent.target_size_in_bytes == record.size
            });
            (
                record.inode,
                content_of_a_file_other_than_the_first(
                    record.size,
                    tail_of_the_entry.get(&record.inode).copied(),
                    names_a_truncation_intent,
                    &data_units,
                ),
            )
        })
        .collect()
}

/// 第一个文件之外一个文件读回的内容（[`observed_inodes_other_than_the_first_of_file_version`]）：`data_units` 按单元号升序，
/// `tail_of_the_entry` 是它上段叶条目标签声明的尾部（没有条目为 `None`），`names_a_truncation_intent` 说 logged_ops 里有没有一条截断条目对得上它。
fn content_of_a_file_other_than_the_first(
    size_in_bytes: u64,
    tail_of_the_entry: Option<TailOfAFile>,
    names_a_truncation_intent: bool,
    data_units: &[(u64, &[u8])],
) -> ObservedFile {
    let payload_capacity =
        u64::try_from(crate::verification_read::data_payload_capacity()).expect("净荷容量");
    let data_units_the_size_asks_for = size_in_bytes.div_ceil(payload_capacity).max(1);
    let data_units_existing = u64::try_from(data_units.len()).expect("单元数装得进 u64");
    let mut payloads: Vec<&[u8]> = Vec::with_capacity(data_units.len());
    // 迭代上界：这个文件的数据单元数；跨轮只往末尾追加；提前出口两个：单元号断了或重了、一个单元解不开。
    for (expected_index, (index, bytes)) in (0_u64..).zip(data_units) {
        if *index != expected_index {
            return ObservedFile::Undecodable {
                what: format!(
                    "数据单元号不是从 0 起连着的：第 {expected_index} 个位置上是 {index}"
                ),
            };
        }
        match crate::verification_read::data_payload(bytes) {
            Ok(payload) => payloads.push(payload),
            Err(error) => {
                return ObservedFile::Undecodable {
                    what: format!("第 {index} 个数据单元解不开：{error:?}"),
                }
            }
        }
    }
    let content_of = |within: &[&[u8]]| -> Vec<u8> { within.concat() };
    let content_read_back = match (tail_of_the_entry, data_units_existing) {
        (None, 0) if size_in_bytes == 0 => Vec::new(),
        (None, _) => {
            return ObservedFile::Undecodable {
                what: format!(
                    "上段里没有它的条目、有 {data_units_existing} 个数据单元，记录的 size 是 {size_in_bytes}（要 size 0、没有单元）"
                ),
            };
        }
        // 标签 0：整个文件是洞。
        (Some(_), 0) => vec![0; usize::try_from(size_in_bytes).expect("size 装得进 usize")],
        (Some(TailOfAFile::NoHoleAfterTheLastDataUnit), _) => {
            if data_units_existing < data_units_the_size_asks_for
                || (data_units_existing > data_units_the_size_asks_for
                    && !names_a_truncation_intent)
            {
                return ObservedFile::Undecodable {
                    what: format!(
                        "标签 1、2：{data_units_existing} 个数据单元，记录的 size {size_in_bytes} 要 {data_units_the_size_asks_for} 个（多出来的只许是截断条目点名的中间态）"
                    ),
                };
            }
            content_of(&payloads[..usize::try_from(data_units_the_size_asks_for).expect("单元数")])
        }
        (Some(TailOfAFile::HoleAfterTheLastDataUnit), _) => {
            if data_units_existing >= data_units_the_size_asks_for
                || payloads.iter().any(|payload| {
                    u64::try_from(payload.len()).expect("载荷长度") != payload_capacity
                })
            {
                return ObservedFile::Undecodable {
                    what: format!(
                        "标签 3、4：{data_units_existing} 个数据单元要都满净荷、少于 size {size_in_bytes} 要的 {data_units_the_size_asks_for} 个"
                    ),
                };
            }
            let mut content = content_of(&payloads);
            content.resize(
                usize::try_from(size_in_bytes).expect("size 装得进 usize"),
                0,
            );
            content
        }
    };
    if u64::try_from(content_read_back.len()).expect("内容长度装得进 u64") != size_in_bytes {
        return ObservedFile::Undecodable {
            what: format!(
                "{} 个数据单元接起来 {} 字节，记录的 size 是 {size_in_bytes}",
                data_units.len(),
                content_read_back.len()
            ),
        };
    }
    ObservedFile::Decoded(content_read_back)
}

/// 截断观测优先从发布输出的 inode 容器原字节读取，偏移按字段表独立定义。
/// 输出不带任何 inode 容器时只观察内存结果的字段，不调用实现的序列化器制造参照字节。
fn observed_truncation_field_of_file_version(
    output: &TransactionOutput,
) -> Option<ObservedTruncationField> {
    let record = output.inode_record.as_ref()?;
    let containers: Vec<_> = output
        .units
        .iter()
        .filter(|unit| matches!(unit.identity, TransactionUnit::InodeLeafContainer(_)))
        .collect();
    if containers.is_empty() {
        return Some(ObservedTruncationField {
            truncation_txg: model_txg(record.truncation_field.truncation_txg()),
            size_in_bytes: record.truncation_field.truncated_size_in_bytes(),
        });
    }
    for container in containers {
        let page = singlefs_checker::packed_unit_view(&container.bytes).ok()?;
        if page.record_type != 2 || page.record_width < 136 {
            return None;
        }
        for bytes in page.records {
            if u64::from_le_bytes(bytes[..8].try_into().ok()?) == record.inode {
                return Some(ObservedTruncationField {
                    truncation_txg: ModelCheckpointTxg(u64::from_le_bytes(
                        bytes[120..128].try_into().ok()?,
                    )),
                    size_in_bytes: u64::from_le_bytes(bytes[128..136].try_into().ok()?),
                });
            }
        }
    }
    None
}

/// 这一版第一个文件之外每条 inode 记录（活着的与删除中的）里截断字段那两段（判决
/// `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S5「截断字段按 inode 比」）：同第一个文件那一份
/// （[`observed_truncation_field_of_file_version`]）优先从发布输出的 inode 容器原字节读，偏移按字段表独立定义（偏移 0 是号、120 是 T、128 是 N_w，
/// 各 8 字节小端）；输出不带任何 inode 容器时只观察内存结果的字段。解不开的容器、记录宽不够的那一页不交（模型那一侧有它就判对不上）。
fn observed_truncation_fields_of_the_inodes_other_than_the_first_of_file_version(
    output: &TransactionOutput,
) -> BTreeMap<u64, ObservedTruncationField> {
    let containers: Vec<_> = output
        .units
        .iter()
        .filter(|unit| matches!(unit.identity, TransactionUnit::InodeLeafContainer(_)))
        .collect();
    if containers.is_empty() {
        return output
            .inode_leaf_containers
            .iter()
            .flat_map(|container| container.contents.records.iter())
            .filter(|record| record.inode != FIRST_INODE_NUMBER)
            .map(|record| {
                (
                    record.inode,
                    ObservedTruncationField {
                        truncation_txg: model_txg(record.truncation_field.truncation_txg()),
                        size_in_bytes: record.truncation_field.truncated_size_in_bytes(),
                    },
                )
            })
            .collect();
    }
    let little_endian_word_at = |bytes: &[u8], offset: usize| {
        let mut word = [0u8; 8];
        word.copy_from_slice(&bytes[offset..offset + 8]);
        u64::from_le_bytes(word)
    };
    let mut fields = BTreeMap::new();
    // 迭代上界：这一版交回的 inode 叶容器数乘每页的记录数；跨轮只往表里加；解不开的那一页跳过（见文档）。
    for container in containers {
        let Ok(page) = singlefs_checker::packed_unit_view(&container.bytes) else {
            continue;
        };
        if page.record_type != 2 || page.record_width < 136 {
            continue;
        }
        for bytes in page.records {
            let inode = little_endian_word_at(&bytes, 0);
            if inode == FIRST_INODE_NUMBER {
                continue;
            }
            fields.insert(
                inode,
                ObservedTruncationField {
                    truncation_txg: ModelCheckpointTxg(little_endian_word_at(&bytes, 120)),
                    size_in_bytes: little_endian_word_at(&bytes, 128),
                },
            );
        }
    }
    fields
}

/// 整理区域换成模型那一份（一个聚簇段：盘、起始槽、槽数）。
#[must_use]
pub fn model_compaction_region(region: CompactionRegion) -> ModelCompactionRegion {
    ModelCompactionRegion {
        device: ModelDeviceIdentity(region.device().0),
        start_slot: region.start_slot().0,
        slot_count: region.slot_count(),
    }
}

/// 模型那一份整理区域换回实现的（执行器拿它判实现交回的一版 R 按记录全空没有）：起始槽不对齐、装不进 6 字节、槽数不是一个聚簇段的交回 `None`
/// （模型记的 R 都是从实现的 R 换过来的，走不到）。
#[must_use]
pub fn compaction_region_of_the_model(region: ModelCompactionRegion) -> Option<CompactionRegion> {
    let of_one_segment = CompactionRegion::of_one_cluster_segment(
        DeviceIdentity(region.device.0),
        SlotNumber(region.start_slot),
    )
    .ok()?;
    (of_one_segment.slot_count() == region.slot_count).then_some(of_one_segment)
}

/// 有树的一版 logged_ops 里那一条活整理意图（签发 txg、R、`c0`；`segs0` 模型不比），没有为 `None`。
#[must_use]
pub fn observed_logged_compaction_intent(
    output: &TransactionOutput,
) -> Option<ModelCompactionIntent> {
    let mut intents = output
        .logged_operations
        .iter()
        .filter_map(|entry| match entry.operation {
            LoggedOperation::Compaction(intent) => Some((entry.issued_txg, intent)),
            LoggedOperation::Truncation(_) => None,
        });
    let live = intents.next();
    assert!(intents.next().is_none(), "模型观测里活整理意图至多一条");
    live.map(|(issued_txg, intent)| ModelCompactionIntent {
        issued_txg: model_txg(issued_txg),
        region: model_compaction_region(intent.region),
        allocation_generation_at_issue: model_txg(intent.allocation_generation_at_issue),
    })
}

/// 这一次整理发布搬走的单元里，有哪几个是这一版树表引用着的用户可见树的根单元（活头的 inode 树、extent 树那两条，与每个快照的码 14、码 15）：
/// 树表条目的根指针按它的映射 key（码 2，`records::mapping_key_for_node`：出生树、出生 txg、实例代号、出生序号，搬迁不改）去搬迁表里找。
/// 一个单元被几条条目引用（快照抄的是活头那一个）只交一次。按（树、出生 txg）升序。
#[must_use]
pub fn user_visible_root_units_relocated_by_this_publish(
    version: &TransactionOutput,
) -> Vec<ModelUserVisibleRootUnit> {
    let relocated_keys: BTreeSet<&[u8]> = version
        .units_relocated_by_this_publish
        .iter()
        .map(|relocated| relocated.mapping_key.as_slice())
        .collect();
    let units: BTreeSet<ModelUserVisibleRootUnit> = version
        .tree_table_entries
        .iter()
        .filter(|entry| entry.root != NodePointer::empty_root())
        .filter_map(|entry| {
            let tree = match entry.kind {
                TREE_KIND_INODE | TREE_KIND_SNAPSHOT_INODE_ROOT => ModelUserVisibleTree::InodeTree,
                TREE_KIND_EXTENT | TREE_KIND_SNAPSHOT_EXTENT_ROOT => {
                    ModelUserVisibleTree::ExtentTree
                }
                _ => return None,
            };
            relocated_keys
                .contains(observed_node_mapping_key(UNIT_CLASS_INDEX_NODE, entry.root).as_slice())
                .then_some(ModelUserVisibleRootUnit {
                    tree,
                    birth_txg: model_txg(entry.root.head.birth_txg),
                })
        })
        .collect();
    units.into_iter().collect()
}

/// 有树的一版的快照那几样（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S7）：活头 inode 树那条条目的
/// `previous_snapshot_txg`、树表里的码 14 条目按（诞生 txg, 树号）升序（派生链的次序，与 `transaction::next_side_of_the_snapshot` 同一个排法）
/// 连同与它配对的码 15（头 ID 是它的树号）、deadlist 条目数、留存的映射条目数。
///
/// # Panics
/// 一条码 14 没有配对的码 15：发布路径成对写、重建时配对判过（`recovery::snapshot_tree_table_entries_paired_one_to_one`）。
#[must_use]
pub fn observed_snapshot_state_of_file_version(
    output: &TransactionOutput,
) -> ObservedSnapshotState {
    let mut snapshot_inode_roots: Vec<_> = output
        .tree_table_entries
        .iter()
        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .collect();
    snapshot_inode_roots.sort_by_key(|entry| (entry.birth_txg, entry.tree));
    let snapshots = snapshot_inode_roots
        .into_iter()
        .map(|inode_root| {
            let extent_root = output
                .tree_table_entries
                .iter()
                .find(|entry| {
                    entry.kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT
                        && entry.head_identifier == inode_root.tree.0
                })
                .expect("每条码 14 配一条头 ID 是它树号的码 15（发布路径成对写，重建时配对判过）");
            ObservedSnapshot {
                birth_txg: model_txg(inode_root.birth_txg),
                previous_snapshot_txg: model_txg(inode_root.previous_snapshot_txg),
                is_being_destroyed: output.snapshots_being_destroyed.contains(&inode_root.tree),
                inode_tree_written_by: model_root_key(
                    inode_root.root.instance,
                    inode_root.root.head.birth_txg,
                ),
                extent_root_written_at: model_txg(extent_root.root.head.birth_txg),
            }
        })
        .collect();
    ObservedSnapshotState {
        live_head_previous_snapshot_txg: Some(model_txg(
            output.previous_snapshot_txg_of_the_live_head(),
        )),
        snapshots,
        deadlist_entries: output
            .deadlist_entries
            .iter()
            .map(observed_deadlist_entry)
            .collect(),
        pinned_mapping_entries: u64::try_from(output.mapping_entries_kept_for_the_snapshots.len())
            .expect("条数"),
    }
}

/// 模型观测用的节点身份：按映射 key 字段表编码，不调用实现的编码器。
pub(crate) fn observed_node_mapping_key(class: u8, pointer: NodePointer) -> Vec<u8> {
    let mut key = vec![class];
    key.extend_from_slice(&pointer.head.birth_tree.0.to_le_bytes());
    key.extend_from_slice(&pointer.head.birth_txg.0.to_le_bytes());
    key.extend_from_slice(&pointer.instance.0.to_le_bytes());
    key.extend_from_slice(&pointer.birth_sequence.0.to_le_bytes());
    key.extend_from_slice(&[0, 0]);
    key
}

/// 一条 deadlist 条目换成模型观测；直接读取 key 的类标签、出生树与出生 txg，末段模型不比。
fn observed_deadlist_entry(entry: &DeadlistEntry) -> ModelDeadlistEntry {
    let key = &entry.mapping_key;
    let unit_class = key[0];
    let birth_tree = u64::from_le_bytes(key[1..9].try_into().expect("映射 key 出生树"));
    let birth_txg = u64::from_le_bytes(key[9..17].try_into().expect("映射 key 出生代"));
    ModelDeadlistEntry {
        epoch: model_txg(entry.epoch),
        mapping_key_unit_class: unit_class,
        mapping_key_birth_tree: birth_tree,
        birth_txg: ModelCheckpointTxg(birth_txg),
    }
}

/// 读一个快照读出了什么（`recovery::read_the_first_file_of_a_snapshot` 的结局），换成模型的观测。
#[must_use]
pub fn observed_snapshot_read_back(
    reading: &Result<SnapshotFileReading, RecoveryFailure>,
) -> ObservedSnapshotReadBack {
    match reading {
        Ok(SnapshotFileReading::NoFile) => ObservedSnapshotReadBack::NoFile,
        Ok(SnapshotFileReading::File(content)) => ObservedSnapshotReadBack::File(content.clone()),
        Ok(SnapshotFileReading::NoSnapshotWithThisTree) => {
            ObservedSnapshotReadBack::NoSnapshotWithThisTree
        }
        Err(failure) => ObservedSnapshotReadBack::Failed {
            what: format!("{failure:?}"),
        },
    }
}

/// 这一版的文件内容：文件第 0、1、2…… 个数据单元的载荷（各按单元头里的声明长度取）按单元号接起来，接到记录的 size 为止；单元号要从 0 起连着、
/// 每个号恰好一个单元，不然解不出。size 罩的单元（⌈size ÷ 净荷⌉.max(1) 个）之外还挂着的不接（截断途中，判决
/// `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2：读者不读它们）；接出来的短于 size 时之后是尾部的洞，补 0 到 size（K5：读洞答 0）；
/// 长于 size 的解不出。
/// inode 1 不是活记录（删除中的记录 nlink = 0、删完树里没有它，D8（核心索引结构） 已定项 16 第 4 条）时这一版没有读得出的文件：
/// 意图那次之后数据单元还在，按单元取会读出被删的内容。
#[must_use]
pub fn observed_file_of_file_version(output: &TransactionOutput) -> ObservedFile {
    let size_in_bytes = match &output.inode_record {
        None => return ObservedFile::NoFile,
        Some(record) if record.nlink == NLINK_OF_AN_INODE_BEING_DELETED => {
            return ObservedFile::NoFile
        }
        Some(record) => record.size,
    };
    let payload_capacity = u64::try_from(crate::verification_read::data_payload_capacity())
        .expect("净荷容量装得进 u64");
    let data_units_within_the_size = size_in_bytes.div_ceil(payload_capacity).max(1);
    let mut data_units: Vec<(u64, &[u8])> = output
        .units
        .iter()
        .filter_map(|unit| {
            if let TransactionUnit::Data(DataUnitIndexInFile(index)) = unit.identity {
                (index < data_units_within_the_size).then_some((index, &unit.bytes[..]))
            } else {
                None
            }
        })
        .collect();
    data_units.sort_by_key(|(index, _)| *index);
    if data_units.is_empty() {
        return ObservedFile::Undecodable {
            what: "这一版的 units 里没有文件第 0 个数据单元".to_string(),
        };
    }
    let mut content = Vec::new();
    // 迭代上界：这一版的数据单元数；跨轮只往内容末尾追加；提前出口两个：单元号断了或重了、一个单元解不开。
    for (expected_index, (index, bytes)) in (0_u64..).zip(&data_units) {
        if *index != expected_index {
            return ObservedFile::Undecodable {
                what: format!(
                    "这一版的数据单元号不是从 0 起连着的：第 {expected_index} 个位置上是 {index}"
                ),
            };
        }
        match crate::verification_read::data_payload(bytes) {
            Ok(payload) => content.extend_from_slice(payload),
            Err(error) => {
                return ObservedFile::Undecodable {
                    what: format!("文件第 {index} 个数据单元解不开：{error:?}"),
                }
            }
        }
    }
    let size = usize::try_from(size_in_bytes).expect("模型罩的历史里 size 装得进 usize");
    if content.len() > size {
        return ObservedFile::Undecodable {
            what: format!(
                "size 罩的 {data_units_within_the_size} 个数据单元接起来 {} 字节，长于记录的 size {size_in_bytes}",
                content.len()
            ),
        };
    }
    content.resize(size, 0);
    ObservedFile::Decoded(content)
}

/// 实例表链上一片的角色是第几片。
fn instance_table_page_of_role(identity: TransactionUnit) -> Option<InstanceTablePageIndex> {
    match identity {
        TransactionUnit::InstanceTable => Some(InstanceTablePageIndex::FIRST),
        TransactionUnit::InstanceTablePageAfterTheFirst(page) => Some(page),
        TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
        | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
        | TransactionUnit::Data(_)
        | TransactionUnit::ExtentLowerNode(_)
        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
        | TransactionUnit::ExtentRoot
        | TransactionUnit::InodeLeafContainer(_)
        | TransactionUnit::InodeRoot
        | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
        | TransactionUnit::AllocationTree
        | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
        | TransactionUnit::AccountingTree
        | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
        | TransactionUnit::DeadlistTree
        | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        | TransactionUnit::MappingTree
        | TransactionUnit::DeletedInodesRoot
        | TransactionUnit::LoggedOperationsRoot
        | TransactionUnit::TombstoneLeafContainer(_)
        | TransactionUnit::TombstoneRoot
        | TransactionUnit::TombstoneNodeBelowTheRoot(_)
        | TransactionUnit::LineageLeafContainer(_)
        | TransactionUnit::LineageRoot
        | TransactionUnit::LineageNodeBelowTheRoot(_)
        | TransactionUnit::RetirementLeafContainer(_)
        | TransactionUnit::RetirementRoot
        | TransactionUnit::RetirementNodeBelowTheRoot(_)
        | TransactionUnit::TreeTable => None,
    }
}

/// 从 `units` 里解不出整张实例表的原因（[`instance_table_rows_of_units`]）。
enum InstanceTableNotDecodedFromTheUnits {
    /// `units` 里一片实例表都没有。
    NoPageInTheUnits,
    /// 链指着的那一片不在 `units` 里。
    ChainedPageNotInTheUnits,
    /// 这一片的字节解不开。
    PageUndecodable(InstanceTablePageIndex),
}

/// `units` 里实例表链的各片，从第 0 片起按每片末尾的链指针记录往下接，接到「无下一片」为止，交回整张表的行（各片的行按链上的次序）。
/// 带文件的一版与树表 0 条上写行的那一版共用这一处：两处各解一份，片的接法会分叉。
///
/// 迭代上界是 `units` 里实例表的片数；跨轮带的是已接起来的行与下一片的片序号；提前出口是链断在 `units` 之外、一片解不开。
fn instance_table_rows_of_units(
    units: &[PublishedUnit],
) -> Result<Vec<ModelInstanceRow>, InstanceTableNotDecodedFromTheUnits> {
    let pages: BTreeMap<InstanceTablePageIndex, &[u8]> = units
        .iter()
        .filter_map(|unit| {
            instance_table_page_of_role(unit.identity).map(|page| (page, &unit.bytes[..]))
        })
        .collect();
    if pages.is_empty() {
        return Err(InstanceTableNotDecodedFromTheUnits::NoPageInTheUnits);
    }
    let mut rows = Vec::new();
    let mut page_index = InstanceTablePageIndex::FIRST;
    for _ in 0..pages.len() {
        let Some(bytes) = pages.get(&page_index) else {
            return Err(InstanceTableNotDecodedFromTheUnits::ChainedPageNotInTheUnits);
        };
        let Some(page) = crate::verification_read::instance_table_page(bytes, page_index.0) else {
            return Err(InstanceTableNotDecodedFromTheUnits::PageUndecodable(
                page_index,
            ));
        };
        rows.extend(page.rows.iter().map(model_instance_row));
        match page.next {
            None => return Ok(rows),
            Some(_) => {
                page_index = InstanceTablePageIndex(
                    page_index
                        .0
                        .checked_add(1)
                        .ok_or(InstanceTableNotDecodedFromTheUnits::ChainedPageNotInTheUnits)?,
                )
            }
        }
    }
    Err(InstanceTableNotDecodedFromTheUnits::ChainedPageNotInTheUnits)
}

/// 带文件的一版的整张实例表：`units` 里实例表链的各片接起来（[`instance_table_rows_of_units`]）。
/// `units` 里一片都没有（mkfs 之后第一次重写之前，实例表还是 mkfs 那一片、`units` 不带它），或链指着的下一片不在 `units` 里
/// （从盘上重建的一版只带第 0 片），交回 [`ObservedInstanceTable::NotInTheOutput`]：比不了，不当成空表。
#[must_use]
pub fn observed_instance_table_of_file_version(
    output: &TransactionOutput,
) -> ObservedInstanceTable {
    match instance_table_rows_of_units(&output.units) {
        Ok(rows) => ObservedInstanceTable::Rows(rows),
        Err(InstanceTableNotDecodedFromTheUnits::NoPageInTheUnits) => {
            ObservedInstanceTable::NotInTheOutput {
                why: "这一版的 units 里没有实例表（mkfs 之后第一次重写之前实例表还是 mkfs 那一片，输出不带它）",
            }
        }
        Err(InstanceTableNotDecodedFromTheUnits::ChainedPageNotInTheUnits) => {
            ObservedInstanceTable::NotInTheOutput {
                why: "实例表链指着的那一片不在这一版的 units 里（从盘上重建的一版只带第 0 片）",
            }
        }
        Err(InstanceTableNotDecodedFromTheUnits::PageUndecodable(page_index)) => {
            ObservedInstanceTable::Undecodable {
                what: format!("实例表第 {} 片解不开", page_index.0),
            }
        }
    }
}

/// 树表 0 条的一版上写行那次发布写出的整张实例表：写行 COW 重写整条链（D18（块里携带什么信息） 已定项 11），
/// 这次写出的 `units` 里就是整条链。一片都没有、链接不上、一片解不开，都是实现交出的东西与它自己的根对不上：
/// 交回 [`ObservedInstanceTable::Undecodable`]（模型报这一版的实例表），不当成比不了。
fn observed_instance_table_written_by_a_row_publish(
    units: &[PublishedUnit],
) -> ObservedInstanceTable {
    let what = match instance_table_rows_of_units(units) {
        Ok(rows) => return ObservedInstanceTable::Rows(rows),
        Err(InstanceTableNotDecodedFromTheUnits::NoPageInTheUnits) => {
            "写行那次发布重写整条实例表链，交回的 units 里一片实例表都没有".to_string()
        }
        Err(InstanceTableNotDecodedFromTheUnits::ChainedPageNotInTheUnits) => {
            "写行那次发布重写整条实例表链，链指着的那一片不在交回的 units 里".to_string()
        }
        Err(InstanceTableNotDecodedFromTheUnits::PageUndecodable(page_index)) => {
            format!("实例表第 {} 片解不开", page_index.0)
        }
    };
    ObservedInstanceTable::Undecodable { what }
}

fn observed_allocation_record(record: &AllocationRecord) -> ObservedAllocationRecord {
    ObservedAllocationRecord {
        device: ModelDeviceIdentity(record.device.0),
        generation: model_txg(record.generation),
        is_released: record.is_released,
    }
}

/// 根记录直接指着的一个单元在这一版分配记录里的那几条：按那条指针的两条位置条目（盘、槽）找。
fn observed_allocation_records_at_the_locations(
    allocation_records: &[AllocationRecord],
    locations: &[LocationEntry],
) -> Vec<ObservedAllocationRecord> {
    allocation_records
        .iter()
        .filter(|record| {
            locations
                .iter()
                .any(|location| location.device == record.device && location.slot == record.slot)
        })
        .map(observed_allocation_record)
        .collect()
}

/// 这次写出（或照抄进来）的一个单元在这一版分配记录里的那几条：落点在它的槽上的（两盘同槽，每块盘一条）。
fn observed_allocation_records_on_the_slot(
    allocation_records: &[AllocationRecord],
    slot: SlotNumber,
) -> Vec<ObservedAllocationRecord> {
    allocation_records
        .iter()
        .filter(|record| record.slot == slot)
        .map(observed_allocation_record)
        .collect()
}

/// 这一版每个角色的单元在这一版分配记录里的那几条：`units` 里的每个角色按单元的槽号找；根记录直接指着、而 `units` 里没有的
/// 两个角色按根记录里那条指针的两条位置条目找——实例表（带文件的一版在 mkfs 之后第一次重写之前）与树表（树表 0 条的一版从不重写
/// mkfs 写的那一片）。这一版仍然有这两个角色，模型那一侧照样问它们（代码审阅第 12 条：两个方向都比，漏交的角色也报）。
/// 带文件的一版与树表 0 条上写行的那一版共用这一处。文件的数据单元按单元号另交（`data_units`，判决
/// `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K4：截断只重写尾单元之后各单元的分配代不同，模型按单元号逐个比）。
fn allocation_records_of_every_role(
    units: &[PublishedUnit],
    allocation_records: &[AllocationRecord],
    root: &RootRecord,
) -> ObservedAllocationRecordsOfEveryUnit {
    let data_units: Vec<(u64, Vec<ObservedAllocationRecord>)> = units
        .iter()
        .filter_map(|unit| {
            if let TransactionUnit::Data(DataUnitIndexInFile(index)) = unit.identity {
                Some((
                    index,
                    observed_allocation_records_on_the_slot(allocation_records, unit.slot),
                ))
            } else {
                None
            }
        })
        .collect();
    let data_units_of_the_files_other_than_the_first: Vec<(
        (u64, u64),
        Vec<ObservedAllocationRecord>,
    )> = units
        .iter()
        .filter_map(|unit| {
            if let TransactionUnit::DataOfAFileOtherThanTheFirst { inode, unit: index } =
                unit.identity
            {
                Some((
                    (inode.0, index.0),
                    observed_allocation_records_on_the_slot(allocation_records, unit.slot),
                ))
            } else {
                None
            }
        })
        .collect();
    let mut records_of_roles: Vec<(ModelUnitRole, Vec<ObservedAllocationRecord>)> = units
        .iter()
        .filter(|unit| !matches!(unit.identity, TransactionUnit::Data(_)))
        .filter_map(|unit| {
            let role = model_unit_role(unit.identity)?;
            Some((
                role,
                observed_allocation_records_on_the_slot(allocation_records, unit.slot),
            ))
        })
        .collect();
    if !records_of_roles
        .iter()
        .any(|(role, _)| *role == ModelUnitRole::InstanceTable)
    {
        let records = observed_allocation_records_at_the_locations(
            allocation_records,
            &root.instance_table.locations,
        );
        records_of_roles.push((ModelUnitRole::InstanceTable, records));
    }
    if !records_of_roles
        .iter()
        .any(|(role, _)| *role == ModelUnitRole::TreeTable)
    {
        let records = observed_allocation_records_at_the_locations(
            allocation_records,
            &root.tree_table.locations,
        );
        records_of_roles.push((ModelUnitRole::TreeTable, records));
    }
    ObservedAllocationRecordsOfEveryUnit {
        roles: records_of_roles,
        data_units,
        data_units_of_the_files_other_than_the_first,
    }
}

/// 一次发布重写的角色（实现交回的 `rewritten`），换成模型的角色；分配记录树根之下的节点不对应角色，不进。
#[must_use]
pub fn rewritten_model_roles(rewritten: &[TransactionUnit]) -> BTreeSet<ModelUnitRole> {
    rewritten
        .iter()
        .filter_map(|unit| model_unit_role(*unit))
        .collect()
}

/// 树表 0 条的一版（零单元发布、树表 0 条上写行）：没有文件。写行那次发布交回这次写出的单元（整条实例表链、分配记录树重写的节点）
/// 与这一版的全部分配记录：同带文件的一版，比整张实例表与每个角色的分配代（两个方向，代码审阅第 12 条）。
/// 零单元发布一个字节都不写、不经分配器，输出不带这一版的实例表与分配记录：实例表比不了（照计数），分配代只比这次重写了哪几个角色。
#[must_use]
pub fn observed_root_of_version_without_file(
    output: &VersionWithoutFilePublishOutput,
) -> ObservedRoot {
    let (instance_table, unit_allocation_records) = match &output.allocation_records {
        AllocationRecordsOfTheVersionWithoutFile::WrittenIntoTheAllocationRecordTreeByThisPublish(
            allocation_records,
        ) => (
            observed_instance_table_written_by_a_row_publish(&output.units),
            ObservedUnitAllocationRecords::EveryRoleOfTheVersion(allocation_records_of_every_role(
                &output.units,
                allocation_records,
                &output.root,
            )),
        ),
        AllocationRecordsOfTheVersionWithoutFile::SameAsThePreviousVersionNotHandedInByAZeroUnitPublish => (
            ObservedInstanceTable::NotInTheOutput {
                why: "零单元发布一个字节都不写：这一版的实例表是上一版那一条（根记录里实例表指针照抄），输出不带它的字节",
            },
            ObservedUnitAllocationRecords::RewrittenRolesOnly(rewritten_model_roles(
                &output.rewritten,
            )),
        ),
    };
    ObservedRoot {
        key: model_root_key(output.root.instance, output.root.checkpoint_txg),
        journal_counter: ModelJournalCounter(output.record.counter),
        rollback_floor: model_txg(output.root.rollback_floor),
        file: ObservedFile::NoFile,
        instance_table,
        unit_allocation_records,
        snapshot_state: ObservedSnapshotState::OF_A_VERSION_WITHOUT_TREES,
        logged_compaction_intent: None,
        first_file_record_truncation_field: None,
        inodes_other_than_the_first: BTreeMap::new(),
        truncation_fields_of_the_inodes_other_than_the_first: BTreeMap::new(),
    }
}

#[must_use]
pub fn observed_root_of_pool_version(version: &PoolVersion) -> ObservedRoot {
    match version {
        PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => {
            observed_root_of_file_version(output)
        }
        PoolVersion::WithoutFile(output) => observed_root_of_version_without_file(output),
    }
}

/// 零单元发布从同一次挂载里前一版往下带着比的那一份（实审 B3c-4，代码审阅第 12 条「每一版比内容和实例表」）：树表 0 条的一版上
/// 写行那次发布交回的单元（整条实例表链、分配记录树重写的节点）与这一版的全部分配记录，连同同一次挂载里最近那一版的根。
/// 零单元发布一个字节都不写、不经分配器，根记录照抄上一版的实例表指针与分配记录树根指针：两条指针与前一版逐字段相同，
/// 这一版的实例表与分配记录就是写行那次交回的那一份。比的是实现交出的东西，不另读盘、不从挂着的分配器取。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentsCarriedToZeroUnitPublishes {
    /// 同一次挂载里最近那一版的根：下一次零单元发布的两条指针拿它比。
    root_of_the_previous_version: RootRecord,
    /// 写行那次发布交回的单元。
    units: Vec<PublishedUnit>,
    /// 写行那次发布交回的这一版全部分配记录。
    allocation_records: Vec<AllocationRecord>,
}

/// 一版树表 0 条的发布，`carried` 是同一次挂载里前一版交回（或往下带下来）的那一份（挂载里第一版、前一版没有可带的是 None）。
/// 交回这一版给模型比的样子，与这一版之后的零单元发布接着往下带的那一份：
/// - 写行那次（交回了单元与全部分配记录）：照 [`observed_root_of_version_without_file`] 比，之后从它带；
/// - 零单元发布、有可带的：根记录里的实例表指针、分配记录树根指针与前一版的逐字段相同 ⇒ 拿带下来的那一份比整张实例表与每个角色的
///   分配代（两个方向，与写行那一版同一套判法）；有一条不同 ⇒ 交回模型判红的那一样（零单元发布改了指针），不算比不了；
/// - 零单元发布、没有可带的（一次挂载里第一版就是零单元：取号之后第一次发布接的是从盘上恢复的那一版，没有单元字节）：
///   照旧交比不了的两臂，模型照计数。
#[must_use]
pub fn observed_root_of_version_without_file_carrying(
    output: &VersionWithoutFilePublishOutput,
    carried: Option<&ContentsCarriedToZeroUnitPublishes>,
) -> (ObservedRoot, Option<ContentsCarriedToZeroUnitPublishes>) {
    match (&output.allocation_records, carried) {
        (
            AllocationRecordsOfTheVersionWithoutFile::WrittenIntoTheAllocationRecordTreeByThisPublish(
                allocation_records,
            ),
            Some(_) | None,
        ) => (
            observed_root_of_version_without_file(output),
            Some(ContentsCarriedToZeroUnitPublishes {
                root_of_the_previous_version: output.root,
                units: output.units.clone(),
                allocation_records: allocation_records.clone(),
            }),
        ),
        (
            AllocationRecordsOfTheVersionWithoutFile::SameAsThePreviousVersionNotHandedInByAZeroUnitPublish,
            None,
        ) => (observed_root_of_version_without_file(output), None),
        (
            AllocationRecordsOfTheVersionWithoutFile::SameAsThePreviousVersionNotHandedInByAZeroUnitPublish,
            Some(carried),
        ) => observed_zero_unit_publish_carried_from_the_version_before(output, carried),
    }
}

/// 零单元发布拿同一次挂载里前一版交回（或往下带下来）的那一份比：两条指针与前一版逐字段相同才带，之后接着带同一份；
/// 有一条不同，那一样交回模型判红的那一臂、之后不再带（模型判红，这段历史停在这一步）。
fn observed_zero_unit_publish_carried_from_the_version_before(
    output: &VersionWithoutFilePublishOutput,
    carried: &ContentsCarriedToZeroUnitPublishes,
) -> (ObservedRoot, Option<ContentsCarriedToZeroUnitPublishes>) {
    let previous_root = &carried.root_of_the_previous_version;
    let instance_table_pointer_carried = output.root.instance_table == previous_root.instance_table;
    let allocation_record_tree_root_carried =
        output.root.allocation_record_tree_root == previous_root.allocation_record_tree_root;
    let instance_table = if instance_table_pointer_carried {
        observed_instance_table_written_by_a_row_publish(&carried.units)
    } else {
        ObservedInstanceTable::Undecodable {
            what: format!(
                "零单元发布该照抄同一次挂载里前一版的实例表指针，交回的根换了一条：前一版 {:?}，这一版 {:?}",
                previous_root.instance_table, output.root.instance_table
            ),
        }
    };
    let unit_allocation_records = if allocation_record_tree_root_carried {
        ObservedUnitAllocationRecords::EveryRoleOfTheVersion(allocation_records_of_every_role(
            &carried.units,
            &carried.allocation_records,
            &output.root,
        ))
    } else {
        ObservedUnitAllocationRecords::AllocationRecordTreeRootChangedByAZeroUnitPublish {
            what: format!(
                "交回的根换了分配记录树根指针：前一版 {:?}，这一版 {:?}",
                previous_root.allocation_record_tree_root, output.root.allocation_record_tree_root
            ),
        }
    };
    let observed = ObservedRoot {
        key: model_root_key(output.root.instance, output.root.checkpoint_txg),
        journal_counter: ModelJournalCounter(output.record.counter),
        rollback_floor: model_txg(output.root.rollback_floor),
        file: ObservedFile::NoFile,
        instance_table,
        unit_allocation_records,
        snapshot_state: ObservedSnapshotState::OF_A_VERSION_WITHOUT_TREES,
        logged_compaction_intent: None,
        first_file_record_truncation_field: None,
        inodes_other_than_the_first: BTreeMap::new(),
        truncation_fields_of_the_inodes_other_than_the_first: BTreeMap::new(),
    };
    let carried_on =
        (instance_table_pointer_carried && allocation_record_tree_root_carried).then(|| {
            ContentsCarriedToZeroUnitPublishes {
                root_of_the_previous_version: output.root,
                units: carried.units.clone(),
                allocation_records: carried.allocation_records.clone(),
            }
        });
    (observed, carried_on)
}

/// 同 [`observed_root_of_version_without_file_carrying`]，带文件的一版照 [`observed_root_of_file_version`] 比，之后没有可带的
/// （零单元发布只接在树表 0 条的一版后面）。
#[must_use]
pub fn observed_root_of_pool_version_carrying(
    version: &PoolVersion,
    carried: Option<&ContentsCarriedToZeroUnitPublishes>,
) -> (ObservedRoot, Option<ContentsCarriedToZeroUnitPublishes>) {
    match version {
        PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => {
            (observed_root_of_file_version(output), None)
        }
        PoolVersion::WithoutFile(output) => {
            observed_root_of_version_without_file_carrying(output, carried)
        }
    }
}

/// 一次挂载做成了什么（同 [`observed_mount`]），暖机的零单元发布从这次挂载里前一版往下带着比（实审 B3c-4）：写行那一版交回过
/// 单元与分配记录的，之后的暖机拿它比整张实例表与每个角色的分配代。交回挂载做成了什么，与这次挂载之后的零单元发布接着往下带的那一份。
/// 执行器（`history`）用这一处；[`observed_mount`] 一次挂载单独看、不往下带。
#[must_use]
pub fn observed_mount_carrying_to_zero_unit_publishes(
    mounted: &Mounted,
) -> (ObservedEffect, Option<ContentsCarriedToZeroUnitPublishes>) {
    let mut carried: Option<ContentsCarriedToZeroUnitPublishes> = None;
    let mut roots = Vec::new();
    // 迭代上界是写行一次加暖机至多根环区域数那么多次；跨轮带的是可带的那一份；没有提前出口。
    for version in
        std::iter::once(&mounted.output.row_publish).chain(mounted.output.warm_up_publishes.iter())
    {
        let (observed, next) = observed_root_of_pool_version_carrying(version, carried.as_ref());
        roots.push(observed);
        carried = next;
    }
    (
        ObservedEffect::Mount {
            instance: model_instance(mounted.output.instance),
            rows_written: mounted
                .output
                .rows_written
                .iter()
                .map(model_instance_row)
                .collect(),
            roots,
        },
        carried,
    )
}

/// 一次挂载做成了什么：取到的号、写的行、写行与暖机那几条根。一次挂载单独看、零单元发布不往下带
/// （往下带的是 [`observed_mount_carrying_to_zero_unit_publishes`]）。
#[must_use]
pub fn observed_mount(mounted: &Mounted) -> ObservedEffect {
    ObservedEffect::Mount {
        instance: model_instance(mounted.output.instance),
        rows_written: mounted
            .output
            .rows_written
            .iter()
            .map(model_instance_row)
            .collect(),
        roots: std::iter::once(&mounted.output.row_publish)
            .chain(mounted.output.warm_up_publishes.iter())
            .map(observed_root_of_pool_version)
            .collect(),
    }
}

/// 一次正常卸载（`mount::unmount`）做成了什么：推了那一串的，是那几次带新 F 的空发布写出的根（每一条照带文件的一版比，F 读根上带的）；
/// 现行那一版树表 0 条、一个字节都没写的，是零条根。卸载不判上限，不交上限。
#[must_use]
pub fn observed_unmount(unmounted: &Unmounted) -> ObservedEffect {
    let roots = match unmounted {
        Unmounted::FloorRaisedToTheCurrentVersion(raised) => raised
            .publishes
            .iter()
            .map(observed_root_of_file_version)
            .collect(),
        Unmounted::NothingWrittenOnAVersionWithoutFile { .. } => Vec::new(),
    };
    ObservedEffect::Publishes {
        roots,
        reported_ceiling: None,
    }
}

/// 冷启动读回的结局：所选根、读回的内容或失败的成员。
#[must_use]
pub fn observed_read_back(outcome: &RecoveryOutcome) -> ObservedReadBack {
    match outcome {
        RecoveryOutcome::NoFile { root } => ObservedReadBack::NoFile {
            root: model_root_key(root.0, root.1),
        },
        RecoveryOutcome::FileRead { root, content } => ObservedReadBack::FileRead {
            root: model_root_key(root.0, root.1),
            content: content.clone(),
        },
        RecoveryOutcome::Failed { failure, root } => ObservedReadBack::Failed {
            what: format!("{failure:?}（所选根 {root:?}）"),
        },
    }
}

/// 崩溃之后的冷启动读回（增补 3 第 3 件）：根取施加 journal 记录前缀之后**实际走的**那条根（`RecoveryReport::effective_root`），
/// 不取 `RecoveryOutcome` 里带的那条。两者的差别只在崩溃状态上看得见：`crates/singlefs-core/src/recovery.rs` 的 `recover` 里
/// `root_key` 取的是 `choose_root`（施加之前所选的根），`walk_to_file` 走的却是 `effective_root`（施加之后的根）——
/// 不建崩溃时记录都在水位之下、两者相等，崩溃状态上记录前缀一施加就不等了，读回的内容属于后者。
/// 崩溃点重放的 oracle（`crash::oracle_violation_for_versions`）判该读出哪一版拿的也是 `effective_root`。
#[must_use]
pub fn observed_read_back_after_a_crash(report: &RecoveryReport) -> ObservedReadBack {
    let Some((instance, checkpoint_txg)) = report.effective_root else {
        return ObservedReadBack::Failed {
            what: format!("没择到根（{:?}）", report.outcome),
        };
    };
    let root = model_root_key(instance, checkpoint_txg);
    match &report.outcome {
        RecoveryOutcome::NoFile { .. } => ObservedReadBack::NoFile { root },
        RecoveryOutcome::FileRead { content, .. } => ObservedReadBack::FileRead {
            root,
            content: content.clone(),
        },
        RecoveryOutcome::Failed { failure, .. } => ObservedReadBack::Failed {
            what: format!("{failure:?}（实际走的根 {root:?}）"),
        },
    }
}

/// 攒批的会话报的一次排空待发的发布与模型比（判决 `research/prompts/m4-publish-trigger-r1-main-verification.md` 第二节「返回即落盘」那一格：
/// 模型对拍的点只放在会话报告的发布之后，待发期间模型里「盘上读得到」的仍是上一次发布的那一份）：`version` 是这次发布发成的那一版
/// （会话的现行那一版），`first_file_read_back` 是这一刻盘上恢复读出的第一个文件。比三样：模型里每个 inode 号在这一版的 inode 树里、size 是它的内容长、
/// 不在删除中；inode 号水位相同；读出的第一个文件与模型里那一份逐字节相同。
///
/// # Errors
/// 对不上的那一样，写明模型里是什么、实现那一版是什么。
pub fn compare_a_publish_of_the_pending_changes_with_the_model(
    version: &TransactionOutput,
    first_file_read_back: Option<&[u8]>,
    model: &crate::model::ModelOfThePendingChanges,
) -> Result<(), String> {
    let published = model.published();
    for (inode, content) in &published.contents_by_inode {
        let Some(record) = version.inode_record_of(singlefs_core::address::InodeNumber(*inode))
        else {
            return Err(format!(
                "模型里发布之后有 inode {inode}（{} 字节），实现那一版（txg {}）的 inode 树里没有它",
                content.len(),
                version.root.checkpoint_txg.0
            ));
        };
        let size_in_the_model = u64::try_from(content.len()).expect("内容长度装得进 u64");
        if record.size != size_in_the_model || record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {
            return Err(format!(
                "inode {inode}：模型里 size {size_in_the_model}、活着，实现那一版（txg {}）记 size {}、nlink {}",
                version.root.checkpoint_txg.0, record.size, record.nlink
            ));
        }
    }
    if version.inode_number_watermark() != published.inode_number_watermark {
        return Err(format!(
            "inode 号水位：模型 {}，实现那一版（txg {}）{}",
            published.inode_number_watermark,
            version.root.checkpoint_txg.0,
            version.inode_number_watermark()
        ));
    }
    let first_file_in_the_model = published
        .contents_by_inode
        .get(&FIRST_INODE_NUMBER)
        .map(Vec::as_slice);
    if first_file_read_back != first_file_in_the_model {
        return Err(format!(
            "恢复读出的第一个文件 {:?} 字节，模型里发布之后是 {:?} 字节（内容不同或一边没有）",
            first_file_read_back.map(<[u8]>::len),
            first_file_in_the_model.map(<[u8]>::len)
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use singlefs_core::transaction::TransactionUnit;

    /// 每一版的文件内容、整张实例表、全部角色的分配代都真的拿实现交回的东西比过（代码审阅第 12 条），不只在形状上对：
    /// 四段带挂载的短历史跑下来一条新发现都没有，四样计数都大于 0——胶水从实现的输出里解出了内容与实例表，
    /// 树表 0 条的那几版比过重写的角色集合。
    #[test]
    fn every_published_version_is_compared_by_content_instance_table_and_every_role_both_ways() {
        use crate::history::{
            execute_history_with, generate_history, HistoryDeviceWidth, HistoryEnding,
            HistoryExecution, HistorySeed, PerStepChecker,
        };
        use singlefs_core::admission::SpaceAdmission;
        let mut counts = crate::model::ModelJudgementCounts::default();
        for seed in 0_u64..4 {
            let run = execute_history_with(
                &generate_history(HistorySeed(seed), 16),
                HistoryExecution {
                    per_step_checker: PerStepChecker::Skipped,
                    device_width: HistoryDeviceWidth::FourGibibytes,
                    space_admission: SpaceAdmission::JudgedByTheFormula,
                },
                &crate::SharedStream::new(),
                &mut |_| {},
            );
            assert!(
                !matches!(run.ending, HistoryEnding::NewFinding { .. }),
                "种子 {seed}：{:?}",
                run.ending
            );
            counts.add(&run.tally.model_counts);
        }
        assert!(counts.file_contents_compared >= 1, "{counts:?}");
        assert!(counts.instance_tables_compared >= 1, "{counts:?}");
        assert!(counts.rewritten_role_sets_compared >= 1, "{counts:?}");
        assert!(counts.allocation_records_compared >= 1, "{counts:?}");
    }

    /// 胶水把实现交回的 inode 叶容器序号映到模型角色那一句，逐片单测（`research/prompts/m3-formal-code-b30-r1-main-verification.md`
    /// 第二节 M1「第 k 片都对」那一行的改法 B3）：一个码 2 根装得下的 135 片（D8（核心索引结构） 已定项 6；实现的
    /// `leaf_containers_one_root_node_holds`，再多一片实现就拒）逐片映到这一片自己的角色——第 0 片是最左那片，第 k 片是左起第 k 片——
    /// 而且 135 片映到 135 个互不相同的角色。按取样点逐片往外加历史用例追不完（胶水封顶在第 k 片，只有建到第 k + 1 片的历史抓得到），
    /// 这一条不跑历史，封顶在任何一片都红。
    #[test]
    fn every_inode_leaf_container_one_root_holds_maps_to_its_own_model_role_and_no_two_share_one() {
        const LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS: usize = 135;
        assert_eq!(
            singlefs_core::inode_tree::leaf_containers_one_root_node_holds(),
            LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS,
            "实现的码 2 根扇出是 135（D8（核心索引结构） 已定项 6）"
        );
        let mut roles_seen = BTreeSet::new();
        for position_from_the_leftmost in 0..LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS {
            let role = model_unit_role(TransactionUnit::InodeLeafContainer(
                singlefs_core::inode_tree::InodeLeafContainerIndexInTree::of_position(
                    position_from_the_leftmost,
                ),
            ))
            .expect("inode 叶容器每一片都对应一个角色");
            let position_the_role_names = match role {
                ModelUnitRole::InodeLeaf => 0,
                ModelUnitRole::InodeLeafRightOfTheLeftmost(leaf_container) => {
                    usize::try_from(leaf_container.position_from_the_leftmost())
                        .expect("位置小于 135")
                }
                ModelUnitRole::Data
                | ModelUnitRole::ExtentRoot
                | ModelUnitRole::ExtentUpperNodeBelowTheRoot(_)
                | ModelUnitRole::InodeRoot
                | ModelUnitRole::AllocationTree
                | ModelUnitRole::AccountingTree
                | ModelUnitRole::MappingTree
                | ModelUnitRole::TreeTable
                | ModelUnitRole::InstanceTable
                | ModelUnitRole::DeletedInodesRoot
                | ModelUnitRole::LoggedOperationsRoot
                | ModelUnitRole::TombstoneLeaf
                | ModelUnitRole::TombstoneRoot
                | ModelUnitRole::RetirementLeaf
                | ModelUnitRole::RetirementRoot => {
                    panic!(
                        "第 {position_from_the_leftmost} 片 inode 叶容器映到了别的角色：{role:?}"
                    )
                }
            };
            assert_eq!(
                position_the_role_names, position_from_the_leftmost,
                "第 {position_from_the_leftmost} 片映到的角色说的是第 {position_the_role_names} 片"
            );
            assert_eq!(
                role,
                ModelUnitRole::inode_leaf_container_at(
                    u64::try_from(position_from_the_leftmost).expect("位置小于 135")
                ),
                "第 {position_from_the_leftmost} 片映到模型落点函数的第 {position_from_the_leftmost} 片"
            );
            assert!(
                roles_seen.insert(role),
                "第 {position_from_the_leftmost} 片与前面某一片映到同一个角色：{role:?}"
            );
        }
        assert_eq!(
            roles_seen.len(),
            LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS,
            "135 片映到 135 个互不相同的角色"
        );
    }

    /// 模型模块的每一份源文件（路径相对 `src/`，根 `model.rs` 在最前），编进测试二进制：源码自查判的就是这几份。
    /// 表与 `model.rs` 起逐层 `mod` 声明到的文件逐个相同，由源码自查那条测试核。
    static MODEL_MODULE_SOURCE_FILES: [(&str, &str); 18] = [
        ("model.rs", include_str!("model.rs")),
        (
            "model/allocation_generation_judging.rs",
            include_str!("model/allocation_generation_judging.rs"),
        ),
        (
            "model/answers_and_observed_outcomes.rs",
            include_str!("model/answers_and_observed_outcomes.rs"),
        ),
        (
            "model/answers_to_mounts_and_rollbacks.rs",
            include_str!("model/answers_to_mounts_and_rollbacks.rs"),
        ),
        (
            "model/answers_to_publishes_and_inode_changes.rs",
            include_str!("model/answers_to_publishes_and_inode_changes.rs"),
        ),
        (
            "model/answers_to_snapshots_and_compaction.rs",
            include_str!("model/answers_to_snapshots_and_compaction.rs"),
        ),
        (
            "model/answers_to_truncations.rs",
            include_str!("model/answers_to_truncations.rs"),
        ),
        (
            "model/crash_recovery_judging.rs",
            include_str!("model/crash_recovery_judging.rs"),
        ),
        (
            "model/disagreements_and_judgement_counts.rs",
            include_str!("model/disagreements_and_judgement_counts.rs"),
        ),
        (
            "model/judging_of_observed_outcomes.rs",
            include_str!("model/judging_of_observed_outcomes.rs"),
        ),
        (
            "model/model_scenarios_for_tests.rs",
            include_str!("model/model_scenarios_for_tests.rs"),
        ),
        (
            "model/next_root_of_a_publish.rs",
            include_str!("model/next_root_of_a_publish.rs"),
        ),
        (
            "model/pending_changes.rs",
            include_str!("model/pending_changes.rs"),
        ),
        (
            "model/publish_changes.rs",
            include_str!("model/publish_changes.rs"),
        ),
        (
            "model/ring_positions_and_tree_node_bounds.rs",
            include_str!("model/ring_positions_and_tree_node_bounds.rs"),
        ),
        (
            "model/rollback_floor_and_ceiling.rs",
            include_str!("model/rollback_floor_and_ceiling.rs"),
        ),
        (
            "model/unit_roles_and_publish_kinds.rs",
            include_str!("model/unit_roles_and_publish_kinds.rs"),
        ),
        (
            "model/versions_and_roots.rs",
            include_str!("model/versions_and_roots.rs"),
        ),
    ];

    /// 条目前面可以带的可见性，不带的那一种在最前。
    const VISIBILITIES_BEFORE_AN_ITEM: [&str; 4] = ["", "pub ", "pub(super) ", "pub(crate) "];

    /// 一行去掉开头空白之后，是 `<可见性><关键字>` 起头的条目就交回关键字之后的那一段。
    fn item_after_the_visibility_and_the_keyword<'line>(
        line: &'line str,
        keyword_with_the_space: &str,
    ) -> Option<&'line str> {
        VISIBILITIES_BEFORE_AN_ITEM.iter().find_map(|visibility| {
            line.strip_prefix(visibility)?
                .strip_prefix(keyword_with_the_space)
        })
    }

    /// 一份模型模块源文件里 `mod <名>;` 声明的子模块文件，路径相对 `src/`：`model.rs` 的在 `model/` 下，`model/<名>.rs` 的在 `model/<名>/` 下。
    fn submodule_files_declared_in(path: &str, source: &str) -> Vec<String> {
        let directory_of_the_submodules = path
            .strip_suffix(".rs")
            .expect("模型模块的源文件表里每条路径都以 .rs 结尾");
        source
            .lines()
            .map(str::trim_start)
            .filter_map(|line| item_after_the_visibility_and_the_keyword(line, "mod "))
            .filter_map(|declaration| declaration.strip_suffix(';'))
            .map(|submodule| format!("{directory_of_the_submodules}/{submodule}.rs"))
            .collect()
    }

    /// 模型模块只用格式常量那一个 crate（D13（验证路线） 已定项 5）：`model.rs` 与 `model/` 下每一份里注释之外的每一行都不提
    /// `singlefs_core`、`singlefs_checker`，`use`（带不带可见性）只有 `std`、`singlefs_format`、模块内的 `self::` 与 `use super::*;`。
    /// 判的那几份恰是 `model.rs` 起逐层 `mod` 声明到的全部文件，少判一份就红。胶水（这个文件）可以用 core。
    #[test]
    fn the_model_module_uses_only_the_standard_library_and_the_format_constants() {
        let judged_files: &[(&str, &str)] = &MODEL_MODULE_SOURCE_FILES;
        let judged_paths: BTreeSet<String> = judged_files
            .iter()
            .map(|(path, _)| (*path).to_owned())
            .collect();
        let declared_paths: BTreeSet<String> = std::iter::once("model.rs".to_owned())
            .chain(
                judged_files
                    .iter()
                    .flat_map(|(path, source)| submodule_files_declared_in(path, source)),
            )
            .collect();
        assert_eq!(
            judged_paths, declared_paths,
            "源码自查判的文件与 model.rs 起逐层 mod 声明到的文件不一样"
        );
        for (path, source) in judged_files {
            let code_lines: Vec<&str> = source
                .lines()
                .map(str::trim_start)
                .filter(|line| !line.starts_with("//"))
                .collect();
            assert!(!code_lines.is_empty(), "{path} 读到了注释之外的行");
            for line in &code_lines {
                assert!(
                    !line.contains("singlefs_core") && !line.contains("singlefs_checker"),
                    "{path} 里有一行提到了实现或 checker：{line}"
                );
                if let Some(used_path) = item_after_the_visibility_and_the_keyword(line, "use ") {
                    assert!(
                        used_path.starts_with("std::")
                            || used_path.starts_with("singlefs_format::")
                            || used_path.starts_with("self::")
                            || used_path == "super::*;",
                        "{path} 的 use 只许 std、singlefs_format 与模型模块自己的子模块：{line}"
                    );
                }
            }
        }
    }
}
