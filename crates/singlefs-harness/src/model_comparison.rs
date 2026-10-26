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
                .push((index.0, unit.bytes.as_slice()));
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
fn observed_node_mapping_key(class: u8, pointer: NodePointer) -> Vec<u8> {
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
                (index < data_units_within_the_size).then_some((index, unit.bytes.as_slice()))
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
            instance_table_page_of_role(unit.identity).map(|page| (page, unit.bytes.as_slice()))
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

fn explained(reason: ModelRefusalReason) -> ObservedRefusalReason {
    ObservedRefusalReason::Explained(reason)
}

/// 发布的错误成员说的是哪条理由。
#[must_use]
pub fn refusal_reason_of_publish_error(error: &PublishError) -> ObservedRefusalReason {
    match error {
        PublishError::PlacementRefused { refusal, .. } => {
            refusal_reason_of_placement_refusal(refusal)
        }
        // 空间准入（D28（挂载期承诺量） 已定项 1 的式子）判这次的普通分配不够：模型的「单元区装不下」就是这一条准入（模型答允许拒绝的区间）。
        PublishError::SpaceAdmissionRefused(_) => explained(ModelRefusalReason::UnitAreaWall),
        // 合取 7（全空聚簇段少于 `N₀`，只拦前台写入类发布）：此前借空间准入不够那一个成员报，模型照单元区墙认；分开报之后照旧认它。
        PublishError::EmptyClusterSegmentsBelowTheFloor(_) => {
            explained(ModelRefusalReason::UnitAreaWall)
        }
        PublishError::ContentExceedsDataUnit { .. } => {
            explained(ModelRefusalReason::ContentExceedsDataUnitPayload)
        }
        // 第一版同时活着的只读快照至多 10 个（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 1）。
        PublishError::SnapshotCreationRefused(
            SnapshotCreationRefusal::MoreSnapshotsThanTheFirstVersionAllows { .. },
        ) => explained(ModelRefusalReason::MoreSnapshotsThanTheFirstVersionAllows),
        PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn { .. } => {
            explained(ModelRefusalReason::FirstFileVersionDoesNotFollowTheVersionItBuildsOn)
        }
        PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile { .. } => {
            explained(ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile)
        }
        // 释放判定路径的五种：上一版的映射或分配记录与上一版对不上、盘上那条指针的两条位置条目不同槽，健康的历史里不该出现。
        // 分配记录树重写集合迭代不收敛那一条同理：只有强制复用窗口为 0 的只供测试的开关下才可能走到，随机历史不装那个开关。
        // inode 树写入被拒那一条同样：随机历史建 inode 一次一个（`HistoryOperation::CreateInode`）或一次几个、不跨叶容器
        // （`HistoryOperation::CreateInodes`），号从水位起发、恒大于树里的每个号，新记录恒追加在最右那片叶、满 233 条在末尾分裂；叶容器数到一个根装得下的 135 片之前（三万多次建 inode）不拒，
        // 它跑的那几种发布每次最多改一片叶容器、重写的角色最多九个。
        // 多层码 2 树算不出形状那一条同理：两棵树装不下一个节点时分裂、不拒，只有长到 257 层或上一版的形状按分隔 key 走不通才拒，
        // 随机历史里两棵树至多长一两层（记账树在建几百个 inode 之后、映射树在跨三片下段叶的文件上）；它出现就是模型与实现对不上。
        PublishError::MultiLevelCodeTwoTreeRefused { .. }
        | PublishError::InodeTreeWriteRefused(_)
        | PublishError::AllocationRecordTreeRewriteSetDidNotSettle { .. }
        | PublishError::ReleaseNotInMapping { .. }
        | PublishError::ReleaseTargetNotAllocated { .. }
        | PublishError::ReleaseTargetAlreadyReleased { .. }
        | PublishError::ReleaseTargetLocationsOnDifferentSlots { .. }
        | PublishError::ReleaseSpanMismatch { .. }
        | PublishError::MappingEntryNarrowerThanItsFieldTable { .. }
        // 映射条目的位置项指池外的盘、或两条指同一块盘（当映射条目损坏）：只有坏镜像、外来镜像上有；
        // 健康的内存盘上都不该出现，模型里没有它们的理由。
        | PublishError::MappingEntryLocationOnADeviceOutsideThePool { .. }
        | PublishError::MappingEntryLocationsOnTheSameDevice { .. }
        // 第一个文件版本读不出那一版的树表、水位离 u64::MAX 不到十二个号（第一个文件版本一次连号发十二棵树的号）：健康的内存盘上都不该出现。
        | PublishError::TreeTableOfTheVersionToBuildOnUnreadable { .. }
        | PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(_)
        // 冻结着一次没重发的发布：随机历史里一次发布失败就整段停下、不接着发，健康的内存盘上不该出现。
        | PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
        // 一次发布切出来的记录多于在飞上限（随机历史每次最多写几个数据单元，远在上限之下）、要接的那一版 txg 已是 u64::MAX（坏镜像才有）：
        // 模型里没有它们的理由。
        | PublishError::JournalRecordsOfThePublishExceedTheLimit { .. }
        | PublishError::NextCheckpointTxgPastTheTopOfItsRange { .. }
        // 墓碑树、谱系树或退役树最右那片的容器号已到顶（C841）：容器号从 1 起逐片加一，健康的内存盘上走不到，坏镜像才有。
        | PublishError::NextContainerNumberPastTheTopOfItsRange(_)
        // 写删除中的 inode、写删完的 inode、unlink 不是活记录的号、deleted_inodes 装不下：执行器只在实现报带文件时调覆盖写与 unlink，
        // 队列不空时那两种改动先走空、再拒成带着走空那几步的 `NoFileVersionToChange`（代码轮 U10、U11），不报这几种；
        // 队列至多 inode 1 一个号，造不出这几格。
        | PublishError::WriteToAnInodeBeingDeleted { .. }
        | PublishError::WriteToAnInodeThatIsNotInTheTree { .. }
        | PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { .. }
        | PublishError::DeletedInodesQueueRefused(_)
        // 建快照时有快照在销毁中、水位加二越过 u64::MAX：会话先把销毁中的快照走完（判决第七节岔路 2），水位只在坏镜像上到顶，健康的内存盘上不该出现。
        | PublishError::SnapshotCreationRefused(
            SnapshotCreationRefusal::SnapshotBeingDestroyed { .. }
            | SnapshotCreationRefusal::TreeIdentifierWatermarkLeavesNoRoomForTheSnapshotTrees { .. },
        )
        // 删快照点名的快照不在或正在删、deadlist 条目留存的映射条目没有或落点核不过：执行器只点名模型里活着的快照、一次删完，
        // 健康的内存盘上不该出现。
        | PublishError::SnapshotDestructionRefused(_)
        | PublishError::DeadlistEntryNotReleasable { .. }
        | PublishError::BlockDevice(_)
        // 可写挂载取号之前预演读错、读坏那两拒：健康的内存盘上不出读错、读坏，模型里没有它们的理由。
        | PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(_)
        | PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(_) => {
            ObservedRefusalReason::Unexplained
        }
        PublishError::TruncationRefused(refusal) => refusal_reason_of_truncation_refusal(*refusal),
        // 删完第一个文件之后给建的 inode 写数据（D8（核心索引结构） 已定项 16 射程 K-U5，用户 2026-10-19 定「拦」）。
        PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion { .. } => {
            explained(ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted)
        }
    }
}

/// 改 size 与按偏移写被拒说的是哪条理由（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K12；
/// `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R3）。
#[must_use]
pub fn refusal_reason_of_truncation_refusal(refusal: TruncationRefusal) -> ObservedRefusalReason {
    match refusal {
        TruncationRefusal::TargetSizeRoundedUpToAWholeDataUnitOverflows { .. } => {
            explained(ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow)
        }
        // 读—改—写读不回旧单元（D4（校验和位置） 已定项 4）：模型有这条理由、健康的内存盘上从不要求它（每个单元都读得回），
        // 实现在健康的历史里拒在这一格，对拍报「模型说该成、实现拒了」，不归「说不出理由」。
        TruncationRefusal::OldDataUnitToRewriteNotReadBack { .. } => {
            explained(ModelRefusalReason::OldDataUnitToRewriteNotReadBack)
        }
        // 执行器只点名树里活着的记录（任意活 inode 都能缩短与变长，判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md`
        // 第二节 S6）；截断条目只经会话的走空续做，走空只在有一条对得上的条目时调：健康的内存盘上这几种都不该出现，模型里没有它们的理由。
        TruncationRefusal::InodeIsNotALiveRecord { .. }
        | TruncationRefusal::TruncationIntentDoesNotNameTheLiveRecord { .. }
        | TruncationRefusal::NoTruncationIntentToContinue => ObservedRefusalReason::Unexplained,
    }
}

/// 分配器拒落点的原因说的是哪条理由：只有每块盘上都没有合政策的落点（容量不够）是单元区墙。小盘写满、各盘落点不一致是第一版不支持的
/// 池形状，不是容量墙；模型的池是两块等大的盘、两盘的空闲图同样地变，模型里没有它的理由，出现就对不上（增补 3 第 2 件代码三方第一轮判决
/// 第三节第 2 条：此前 `NoSpaceFor` 装着这四种，一律映射成单元区墙，单元区墙的区间一开就被接走）。
#[must_use]
pub fn refusal_reason_of_placement_refusal(refusal: &PlacementRefusal) -> ObservedRefusalReason {
    match refusal {
        PlacementRefusal::NoFreeSlotOnAnyDevice => explained(ModelRefusalReason::UnitAreaWall),
        PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { .. }
        | PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported { .. }
        | PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
            ..
        } => ObservedRefusalReason::Unexplained,
    }
}

/// 回退目标不在候选集里的那一条说的是哪条理由：按字段一对一映射，不看给人看的文字（增补 3 第 2 件代码三方第一轮判决第三节第 2 条）。
/// 候选集四条（D23（journal 的角色与格式） 已定项 14：根环里、txg ≥ F_生效、按现行那一版的实例表判仍然有效、带文件；D16（发布语义） 已定项 1
/// 「回退候选集」原来另有「txg 落在一圈窗口里」一条，用户 2026-10-04 定 RGall 拿掉），各映射到自己那一条。
#[must_use]
pub fn refusal_reason_of_rollback_candidate_exclusion(
    exclusion: RollbackCandidateExclusion,
) -> ObservedRefusalReason {
    explained(match exclusion {
        RollbackCandidateExclusion::NotInRing => ModelRefusalReason::RollbackTargetNotInRing,
        RollbackCandidateExclusion::BelowEffectiveFloor => {
            ModelRefusalReason::RollbackTargetBelowEffectiveFloor
        }
        RollbackCandidateExclusion::OnAbandonedTimeline => {
            ModelRefusalReason::RollbackTargetOnAbandonedTimeline
        }
        RollbackCandidateExclusion::VersionWithoutFile => {
            ModelRefusalReason::RollbackTargetWithoutFile
        }
    })
}

/// 管理员回退（挂着时的一次向前发布）的错误成员说的是哪条理由：候选集那四条按字段映射；回退那次发布自己的错照发布路径映射；
/// 账读不出、账对不上、单元读不出、冻结着没重发、实例表或系统配置读不出、两版树的号不同：健康的内存盘上都不该出现。
#[must_use]
pub fn refusal_reason_of_rollback_error(error: &RollbackError) -> ObservedRefusalReason {
    match error {
        RollbackError::TargetNotACandidate { exclusion, .. } => {
            refusal_reason_of_rollback_candidate_exclusion(*exclusion)
        }
        RollbackError::Publish(failed) => refusal_reason_of_publish_error(&failed.cause),
        RollbackError::CurrentVersionUnitNotReleasable { cause } => {
            refusal_reason_of_publish_error(cause)
        }
        // 第一版有快照就拒回退（判决 S8、岔路 3）：模型按它记着的活快照判同一条。
        RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses { .. } => {
            explained(ModelRefusalReason::RollbackWithSnapshotsWhichTheFirstVersionRefuses)
        }
        RollbackError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
        | RollbackError::Recovery(_)
        // 调用方的参数或盘表与盘上不一致（随机历史每次都交整池两块盘、用建池的那一份参数）、现行那一版 txg 已是 u64::MAX（坏镜像才有）。
        | RollbackError::CallerInputsDisagreeWithTheDisk(_)
        | RollbackError::NextCheckpointTxgPastTheTopOfItsRange { .. }
        | RollbackError::CurrentInstanceTableMalformed
        | RollbackError::TargetVersionUnreadable { .. }
        | RollbackError::CurrentAccountUnreadable { .. }
        | RollbackError::TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
            ..
        }
        | RollbackError::UserVisibleUnitWithoutItsRecordInTheCurrentAccount { .. }
        | RollbackError::UserVisibleUnitStillAllocatedUnderAnotherGeneration { .. }
        // 回退目标引用的单元（复活集里的、隔离的）在一块盘上的那一份读不出或对不上：健康的内存盘上不该出现；故障注入里说谎的设备吞了落在
        // 那一份上的写才有（`research/prompts/code-backlog-b2-code-r1-main-verification.md`「更正」一节 A4），那一格不在这里映射成理由，
        // 由故障注入按实现点名的那一份认（`unit_copy_unreadable_or_mismatched_of_rollback_error`，`fault_injection::left_by_the_lying_device`）。
        | RollbackError::ResurrectedUnitCopyUnreadableOrMismatched { .. }
        | RollbackError::IsolatedUnitCopyUnreadableOrMismatched { .. }
        // 判候选集读根环或算 F_生效 重读一次仍读坏（实审 A3b Q7）：读路径上注入故障、或同一个进程里一次根槽写被说谎的设备吞掉
        // （那一槽这个进程知道住着根、盘上却不是它写的那一份）才有，与 `MountError::Recovery(_)` 同一处置。
        | RollbackError::CandidateJudgementStillUnreadableAfterOneReread(_) => {
            ObservedRefusalReason::Unexplained
        }
    }
}

/// 挂载、抬 F 的错误成员说的是哪条理由。
#[must_use]
pub fn refusal_reason_of_mount_error(error: &MountError) -> ObservedRefusalReason {
    match error {
        MountError::Publish(failed) | MountError::RaiseFloorSequencePublishFailed(failed) => {
            refusal_reason_of_publish_error(&failed.cause)
        }
        MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { cause, .. }
        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { cause, .. }
        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { cause, .. } => {
            refusal_reason_of_publish_error(cause)
        }
        // 可写挂载写行与暖机之后推抬 F、抬 F 自己报错（实审 A1b Q5）：理由就是抬 F 那个错的理由（改之前挂载原样交回它）。
        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {
            refusal_reason_of_mount_error(&failed.cause)
        }
        MountError::RollbackFloorAboveCeiling { .. } => {
            explained(ModelRefusalReason::FloorAboveCeiling)
        }
        // 取号之后那一串自己的落点在取号之前就取不到：说的是分配器那一条原因（每块盘上都没有 = 单元区墙）。
        MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
            refusal, ..
        } => refusal_reason_of_placement_refusal(refusal),
        // 取号之前空间准入不够（实例切换的预留拿不到）：同发布那一条，是模型的单元区墙。
        MountError::SpaceAdmissionRefusedBeforeAcquisition { .. } => {
            explained(ModelRefusalReason::UnitAreaWall)
        }
        // 树表或实例表不是 mkfs 写的那一版、而这一版又没有自己的分配记录树：手造镜像才走得到。映射成模型那一条理由，模型的答案里一条都不列
        // （C512（树表 0 条的一版上被换下的单元记在哪） 2026-09-23 定案之后这一格不再拒，`IdealModel::answer_establishing_an_instance`），
        // 实现要是在随机历史里拒在这一格，对拍报「模型说该成、实现拒了」。
        MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. } => {
            explained(ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem)
        }
        // 读阶段判出系统配置见证过比所选那一版新的发布、重读一次仍判真（C554 乙）：崩溃恢复抛弃根那一步把最新那条根读成全 0 就走到，
        // 模型那一步答必须拒（`IdealModel::answer_mount_writable_with_the_newest_root_unreadable`）。
        MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) => {
            match **still_unreadable {
                StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { .. } => {
                    explained(
                        ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread,
                    )
                }
                // 重建分配器时最新那条根的实例表重读仍读不出（代码审阅第 22 条）：读阶段判完、判据为假之后才读它，崩溃恢复抛弃根
                // 那一步在读阶段就拒了走不到；只在读路径上注入故障、正好落在那一片上才有，模型没有这一条。
                StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger { .. }
                // 挂着时抬 F 算 F 生效值读最新那条根的实例表、重算影子账读根环里这个进程知道住着根的槽，重读一次仍读坏（实审 A3b Q4，
                // 改之前经 `MountError::Recovery` 交出、同样记成没理由）：只在读路径上注入故障才有，模型没有这一条。
                | StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. }
                | StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { .. }
                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. }
                // 没施加的发布的账读不出、读得出的记录不齐、映射也现算不成（C_MAPPING）：随机历史不藏 journal 记录，模型没有这一条。
                | StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords { .. }
                // 水位之上一次缺记录、读得出的记录点名的单元又验不过的发布（F1 乙）：随机历史不藏 journal 记录，模型没有这一条。
                | StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => {
                    ObservedRefusalReason::Unexplained
                }
            }
        }
        // 恢复失败、记录读不出、表解不开、取号失败、坏盘上才有的根、判定与取号之间号变了、有盘不带所选那一版（空盘、停在旧状态）、
        // 交进来的盘少于 w 的下限（随机历史每次都交整池两块盘）、抬 F 先写系统配置那一步的块设备错、要抬到的 F 低于盘上的生效值
        // （同一个进程里上一次先写系统配置只写进一部分盘才有）、算抬 F 上限时一个知道住着根的根环槽读坏又重读仍坏（读路径上注入故障才有）：
        // 健康的内存盘上都不该出现。
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::InstanceTableUnreadable
        | MountError::Acquisition(_)
        | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
        | MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { .. }
        | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
        | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
        | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. }
        | MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_)
        | MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided { .. }
        // 盘表里同一个身份交了几次、调用方参数与盘上系统配置不一致、有盘落后于现行那一版又缺它的单元或它的根：随机历史每次都交整池两块盘
        // （没换过盘）、用建池的那一份参数，走不到。
        | MountError::DeviceIdentitiesHandedInMoreThanOnce { .. }
        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
        // 盘上的实例代号或 txg 已到顶：坏镜像才有。
        | MountError::SequenceNumberPastTheTopOfItsRange(_)
        // 重建分配器读根环有一块盘没读出有效根、整环重读一遍，两遍逐槽并起来整个环仍一条不被抛弃的自证有效根都没有（Q3 另起的拒因；
        // R1 按槽并两遍之后，第一遍读出过一条的不拒）：读阶段择得到根，只在读路径上注入故障才有，模型没有这一条。
        | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread
        // 读阶段判完之后（判据 N-配置 为假）逐盘核或取号那一刻系统配置又读不出（C664（挂载拒绝合法复算把两种拒因报成一个）之前并在 N-配置 里、
        // 映射成模型那一条理由）：两遍读之间的瞬时读错或一块盘两槽都坏才有，模型没有这一条，不映射成「见证过比所选那一版新的发布」。
        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => {
            ObservedRefusalReason::Unexplained
        }
    }
}

/// 实现的根环槽 (区域, 槽) 换成模型的同一个槽。
#[must_use]
pub fn model_ring_position(ring_slot: RootRingSlot) -> ModelRingPosition {
    ModelRingPosition {
        region: ring_slot.region,
        slot_in_region: ring_slot.slot,
    }
}

/// 算抬 F 的上限时一个知道住着根的根环槽读坏、重读仍坏（`MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`）
/// 时实现点名的那个槽；别的成员 None。
#[must_use]
pub fn root_ring_slot_still_bad_after_one_reread_of_mount_error(
    error: &MountError,
) -> Option<ModelRingPosition> {
    match error {
        MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {
            ring_slot, ..
        } => Some(model_ring_position(*ring_slot)),
        // 可写挂载推的抬 F 报的错装在里面（实审 A1b Q5）：点名的槽照抬 F 那个错取。
        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {
            root_ring_slot_still_bad_after_one_reread_of_mount_error(&failed.cause)
        }
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::InstanceTableUnreadable
        | MountError::Acquisition(_)
        | MountError::Publish(_)
        | MountError::RaiseFloorSequencePublishFailed(_)
        | MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. }
        | MountError::RollbackFloorAboveCeiling { .. }
        | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
        | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
        | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
        | MountError::SpaceAdmissionRefusedBeforeAcquisition { .. }
        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
        | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. }
        | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
        | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. }
        | MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_)
        | MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided { .. }
        | MountError::DeviceIdentitiesHandedInMoreThanOnce { .. }
        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
        | MountError::SequenceNumberPastTheTopOfItsRange(_)
        | MountError::NewerStateStillUnreadableAfterOneReread(_)
        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_)
        | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => None,
    }
}

/// 挂着时回退判候选集读根环，一个知道住着根的根环槽读坏、重读仍坏
/// （`RollbackError::CandidateJudgementStillUnreadableAfterOneReread` 装着 `RootRingSlotKnownToHoldARoot`）时实现点名的那个槽；别的成员 None。
#[must_use]
pub fn root_ring_slot_still_bad_after_one_reread_of_rollback_error(
    error: &RollbackError,
) -> Option<ModelRingPosition> {
    match error {
        RollbackError::CandidateJudgementStillUnreadableAfterOneReread(still_unreadable) => {
            match **still_unreadable {
                StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot {
                    ring_slot, ..
                } => Some(model_ring_position(ring_slot)),
                StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { .. }
                | StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger { .. }
                | StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. }
                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. }
                | StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords { .. }
                | StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => None,
            }
        }
        RollbackError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
        | RollbackError::Recovery(_)
        | RollbackError::CallerInputsDisagreeWithTheDisk(_)
        | RollbackError::NextCheckpointTxgPastTheTopOfItsRange { .. }
        | RollbackError::CurrentInstanceTableMalformed
        | RollbackError::TargetNotACandidate { .. }
        | RollbackError::TargetVersionUnreadable { .. }
        | RollbackError::CurrentAccountUnreadable { .. }
        | RollbackError::TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
            ..
        }
        | RollbackError::UserVisibleUnitWithoutItsRecordInTheCurrentAccount { .. }
        | RollbackError::UserVisibleUnitStillAllocatedUnderAnotherGeneration { .. }
        | RollbackError::ResurrectedUnitCopyUnreadableOrMismatched { .. }
        | RollbackError::IsolatedUnitCopyUnreadableOrMismatched { .. }
        | RollbackError::CurrentVersionUnitNotReleasable { .. }
        | RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses { .. }
        | RollbackError::Publish(_) => None,
    }
}

/// 管理员回退拒在「回退目标引用的一个单元在一块盘上的那一份读不出或整单元 CRC-32C 对不上」上时实现点名的那一份
/// （复活集里的 `ResurrectedUnitCopyUnreadableOrMismatched` 与隔离单元的 `IsolatedUnitCopyUnreadableOrMismatched` 同一形，
/// `research/prompts/code-backlog-b2-code-r1-main-verification.md`「更正」一节 A4）；别的成员 None。
#[must_use]
pub fn unit_copy_unreadable_or_mismatched_of_rollback_error(
    error: &RollbackError,
) -> Option<ModelUnitCopyLocation> {
    match error {
        RollbackError::ResurrectedUnitCopyUnreadableOrMismatched { unit, device, slot }
        | RollbackError::IsolatedUnitCopyUnreadableOrMismatched { unit, device, slot } => {
            Some(ModelUnitCopyLocation {
                device: ModelDeviceIdentity(device.0),
                span_in_slots: u32::try_from(unit.span_slots())
                    .expect("一个单元两槽以内，装得进 u32"),
                first_slot: slot.0,
            })
        }
        RollbackError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
        | RollbackError::Recovery(_)
        | RollbackError::CallerInputsDisagreeWithTheDisk(_)
        | RollbackError::NextCheckpointTxgPastTheTopOfItsRange { .. }
        | RollbackError::CurrentInstanceTableMalformed
        | RollbackError::TargetNotACandidate { .. }
        | RollbackError::TargetVersionUnreadable { .. }
        | RollbackError::CurrentAccountUnreadable { .. }
        | RollbackError::TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
            ..
        }
        | RollbackError::UserVisibleUnitWithoutItsRecordInTheCurrentAccount { .. }
        | RollbackError::UserVisibleUnitStillAllocatedUnderAnotherGeneration { .. }
        | RollbackError::CurrentVersionUnitNotReleasable { .. }
        | RollbackError::CandidateJudgementStillUnreadableAfterOneReread(_)
        | RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses { .. }
        | RollbackError::Publish(_) => None,
    }
}

/// 抬 F 被上限拒时实现报的上限。
#[must_use]
pub fn reported_ceiling_of_mount_error(error: &MountError) -> Option<ModelCheckpointTxg> {
    match error {
        MountError::RollbackFloorAboveCeiling { ceiling, .. } => Some(model_txg(*ceiling)),
        // 可写挂载推的抬 F 报的错装在里面（实审 A1b Q5）：上限照抬 F 那个错取。
        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {
            reported_ceiling_of_mount_error(&failed.cause)
        }
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::InstanceTableUnreadable
        | MountError::Acquisition(_)
        | MountError::Publish(_)
        | MountError::RaiseFloorSequencePublishFailed(_)
        | MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. }
        | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
        | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
        | MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { .. }
        | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
        | MountError::SpaceAdmissionRefusedBeforeAcquisition { .. }
        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
        | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. }
        | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
        | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. }
        | MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_)
        | MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided { .. }
        | MountError::DeviceIdentitiesHandedInMoreThanOnce { .. }
        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
        | MountError::SequenceNumberPastTheTopOfItsRange(_)
        | MountError::NewerStateStillUnreadableAfterOneReread(_)
        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_)
        | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => None,
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
    use crate::model::{
        IdealModel, ModelDeviceIdentity, ModelDisagreementAspect, ModelPoolGeometry,
        ObservedOutcome,
    };
    use singlefs_core::address::{DataUnitIndexInFile, DeviceIdentity, InodeNumber, SlotNumber};
    use singlefs_core::allocator::CommitGeneratedDeviceAnswer;
    use singlefs_core::mount::RollbackTarget;
    use singlefs_core::transaction::{OldDataUnitReadBackFailure, TransactionUnit};
    use singlefs_format::{SLOT_BYTES, UNIT_AREA_START_SLOT};

    fn every_placement_refusal() -> [PlacementRefusal; 4] {
        [
            PlacementRefusal::NoFreeSlotOnAnyDevice,
            PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined {
                full_devices: vec![DeviceIdentity(1)],
            },
            PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported {
                slot_per_device: vec![
                    (DeviceIdentity(0), SlotNumber(50_184)),
                    (DeviceIdentity(1), SlotNumber(50_182)),
                ],
            },
            PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
                answer_per_device: vec![
                    (
                        DeviceIdentity(0),
                        CommitGeneratedDeviceAnswer::OpenEmptySegment(SlotNumber(196_672)),
                    ),
                    (
                        DeviceIdentity(1),
                        CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(SlotNumber(196_638)),
                    ),
                ],
            },
        ]
    }

    /// 单元区墙的区间开着时（单元区只有 640 槽的小盘，第一个文件之后占槽上界 17 × 64 > 640），发布的落点被拒：每块盘上都没有（容量不够）
    /// 映射成单元区墙、模型放行；小盘写满、各盘落点不一致是第一版不支持的池形状，映射成模型没有的理由、判「模型说该成、实现拒了」
    /// （增补 3 第 2 件代码三方第一轮判决第三节第 2 条：此前 `NoSpaceFor` 装着这四种、一律映射成单元区墙，区间一开就被接走）。
    #[test]
    fn only_a_placement_refused_on_every_device_passes_as_the_unit_area_wall() {
        let mut model = IdealModel::after_make_filesystem(ModelPoolGeometry {
            devices: vec![ModelDeviceIdentity(0), ModelDeviceIdentity(1)],
            device_size_in_bytes: (UNIT_AREA_START_SLOT + 640) * SLOT_BYTES,
        });
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let answer = model
            .answer_publish_first_file(&[3])
            .expect("会话开着、现行 txg 2");
        assert!(
            answer.required_refusals.is_empty(),
            "条款不要求拒：{answer:?}"
        );
        for refusal in every_placement_refusal() {
            let is_capacity = refusal == PlacementRefusal::NoFreeSlotOnAnyDevice;
            let error = PublishError::PlacementRefused {
                unit: TransactionUnit::Data(DataUnitIndexInFile::FIRST),
                refusal,
            };
            let observed = ObservedOutcome::Refused {
                member: format!("{error:?}"),
                reason: refusal_reason_of_publish_error(&error),
                publishes_completed: 0,
                wrote_anything: false,
                reported_ceiling: None,
                reported_root_ring_slot_still_bad_after_one_reread: None,
            };
            let judged = model
                .clone()
                .judge_and_advance(&answer, &observed)
                .map(|_| ())
                .map_err(|disagreement| disagreement.aspect);
            if is_capacity {
                assert_eq!(judged, Ok(()), "容量不够是单元区墙：{error:?}");
            } else {
                assert_eq!(
                    judged,
                    Err(ModelDisagreementAspect::RefusedWhenModelRequiresSuccess),
                    "不是容量墙：{error:?}"
                );
            }
        }
    }

    /// 可写挂载写行与暖机之后推的抬 F 报错时，挂载交回的成员装着抬 F 的错（实审 A1b Q5）：理由、报的上限与点名的根环槽都照里面那个错取，
    /// 与改之前挂载原样交回它时相同。
    #[test]
    fn a_floor_raise_failure_after_the_mounts_publishes_is_judged_by_the_floor_raise_error_inside()
    {
        use singlefs_core::mount::FloorRaiseFailedAfterTheMountsPublishes;
        use singlefs_core::recovery::BadRootRingSlotReading;
        let wrapped = |cause: MountError| {
            MountError::FloorRaiseFailedAfterTheMountsPublishes(Box::new(
                FloorRaiseFailedAfterTheMountsPublishes {
                    cause,
                    writes_of_persisted_publishes: Vec::new(),
                    floor_raises: Vec::new(),
                },
            ))
        };
        let above_the_ceiling = || MountError::RollbackFloorAboveCeiling {
            requested: CheckpointTxg(9),
            ceiling: CheckpointTxg(7),
        };
        assert_eq!(
            refusal_reason_of_mount_error(&wrapped(above_the_ceiling())),
            ObservedRefusalReason::Explained(ModelRefusalReason::FloorAboveCeiling)
        );
        assert_eq!(
            reported_ceiling_of_mount_error(&wrapped(above_the_ceiling())),
            Some(model_txg(CheckpointTxg(7)))
        );
        let ring_slot = RootRingSlot { region: 1, slot: 2 };
        let still_bad = MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {
            ring_slot,
            first_reading: BadRootRingSlotReading::Unreadable,
            reread: BadRootRingSlotReading::Unreadable,
        };
        assert_eq!(
            root_ring_slot_still_bad_after_one_reread_of_mount_error(&wrapped(still_bad)),
            Some(model_ring_position(ring_slot))
        );
    }

    /// 挂着时回退判候选集读根环、一个知道住着根的槽重读仍坏：回退交回的成员点名的槽原样交给模型比对（故障注入按它认被吞的根槽写）；
    /// 同一个成员里装的另外三样、回退别的成员都不点名槽。
    #[test]
    fn a_rollback_refused_on_a_root_ring_slot_known_to_hold_a_root_names_that_slot_and_no_other_refusal_names_one(
    ) {
        use singlefs_core::mount::{
            NewerPublishWitness, SelectedVersionAgainstTheWitness, StillUnreadableAfterOneReread,
            WitnessedCounterComparison,
        };
        use singlefs_core::recovery::BadRootRingSlotReading;
        let candidate_judgement = |still_unreadable: StillUnreadableAfterOneReread| {
            RollbackError::CandidateJudgementStillUnreadableAfterOneReread(Box::new(
                still_unreadable,
            ))
        };
        let root_ring_slot_still_bad = candidate_judgement(
            StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot {
                ring_slot: RootRingSlot { region: 0, slot: 6 },
                first_reading: BadRootRingSlotReading::NotSelfVerified,
                reread: BadRootRingSlotReading::NotSelfVerified,
            },
        );
        assert_eq!(
            root_ring_slot_still_bad_after_one_reread_of_rollback_error(&root_ring_slot_still_bad),
            Some(ModelRingPosition {
                region: 0,
                slot_in_region: 6
            }),
            "回退点名的槽要原样交给模型比对"
        );
        let newest_root = RollbackTarget {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(18),
        };
        let against_the_witness = SelectedVersionAgainstTheWitness {
            selected_version: newest_root,
            witness: NewerPublishWitness {
                witnessed_journal_counter: 5,
                comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                    selected_version_last_record_counter: 4,
                },
            },
        };
        let refusals_naming_no_slot = [
            candidate_judgement(
                StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
                    first_read: against_the_witness,
                    reread: against_the_witness,
                },
            ),
            candidate_judgement(
                StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger {
                    newest_root_on_the_reread: Some(newest_root),
                },
            ),
            candidate_judgement(
                StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor {
                    newest_root_on_the_reread: Some(newest_root),
                },
            ),
            RollbackError::CurrentInstanceTableMalformed,
            RollbackError::TargetNotACandidate {
                target: newest_root,
                exclusion: RollbackCandidateExclusion::NotInRing,
            },
        ];
        for error in &refusals_naming_no_slot {
            assert_eq!(
                root_ring_slot_still_bad_after_one_reread_of_rollback_error(error),
                None,
                "不是「知道住着根的根环槽重读仍坏」的回退拒绝不点名槽：{error:?}"
            );
        }
    }

    /// 管理员回退拒在「回退目标引用的一个单元在一块盘上的那一份读不出或对不上」上（`research/prompts/code-backlog-b2-code-r1-main-verification.md`
    /// 「更正」一节 A4）：复活集里的与隔离单元的两个成员照同一个判据交出点名的那一份（哪块盘、从哪个槽起、几个槽）；
    /// 别的回退拒绝一份都不交，同样点名了盘与槽的「账里没有它的记录」也不交（说的不是那一份读不出）。
    #[test]
    fn both_unit_copy_refusals_of_a_rollback_name_the_copy_and_no_other_refusal_does() {
        let unit = TransactionUnit::Data(DataUnitIndexInFile::FIRST);
        let named_copy = ModelUnitCopyLocation {
            device: ModelDeviceIdentity(1),
            span_in_slots: 2,
            first_slot: 51_000,
        };
        for error in [
            RollbackError::ResurrectedUnitCopyUnreadableOrMismatched {
                unit,
                device: DeviceIdentity(1),
                slot: SlotNumber(51_000),
            },
            RollbackError::IsolatedUnitCopyUnreadableOrMismatched {
                unit,
                device: DeviceIdentity(1),
                slot: SlotNumber(51_000),
            },
        ] {
            assert_eq!(
                unit_copy_unreadable_or_mismatched_of_rollback_error(&error),
                Some(named_copy),
                "数据单元占两槽：{error:?}"
            );
        }
        for error in [
            RollbackError::CurrentInstanceTableMalformed,
            RollbackError::UserVisibleUnitWithoutItsRecordInTheCurrentAccount {
                unit,
                device: DeviceIdentity(1),
                slot: SlotNumber(51_000),
            },
        ] {
            assert_eq!(
                unit_copy_unreadable_or_mismatched_of_rollback_error(&error),
                None,
                "不是「那一份读不出或对不上」的回退拒绝不交那一份：{error:?}"
            );
        }
    }

    /// C554 乙那一拒（`MountError::NewerStateStillUnreadableAfterOneReread`）按它说的是哪一样分：系统配置见证过比所选那一版新的发布、
    /// 重读一次仍判真，映射成模型那一条理由（崩溃恢复抛弃根那一步模型答必须拒的就是它）；重建分配器时最新那条根的实例表重读仍读不出，
    /// 模型没有这一条，照 Unexplained——两样不许混成一条。
    #[test]
    fn only_the_witnessed_newer_publish_still_unreadable_after_one_reread_maps_to_the_models_reason(
    ) {
        use singlefs_core::mount::{
            NewerPublishWitness, SelectedVersionAgainstTheWitness, StillUnreadableAfterOneReread,
            WitnessedCounterComparison,
        };
        let against_the_witness = SelectedVersionAgainstTheWitness {
            selected_version: RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
            },
            witness: NewerPublishWitness {
                witnessed_journal_counter: 5,
                comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                    selected_version_last_record_counter: 4,
                },
            },
        };
        let witnessed_newer_publish = MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
            StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
                first_read: against_the_witness,
                reread: against_the_witness,
            },
        ));
        let instance_table_of_the_newest_root =
            MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
                StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger {
                    newest_root_on_the_reread: None,
                },
            ));
        assert_eq!(
            (
                refusal_reason_of_mount_error(&witnessed_newer_publish),
                refusal_reason_of_mount_error(&instance_table_of_the_newest_root)
            ),
            (
                ObservedRefusalReason::Explained(
                    ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread
                ),
                ObservedRefusalReason::Unexplained
            )
        );
    }

    /// C664（挂载拒绝合法复算把两种拒因报成一个）：读阶段判完之后逐盘核或取号那一刻系统配置又读不出
    /// （`MountError::SystemConfigurationUnreadableAfterTheReadStage`，三处各一个）映射成 Unexplained，不映射成读阶段判据 N-配置 那一条模型理由——
    /// 两种拒因分开报之后，模型那一侧也不许再把它们混成一条。
    /// 判别力：这一成员映射成「见证过比所选那一版新的发布、重读仍判真」（`crates/mutations.tsv` 一行）时这一条红。
    #[test]
    fn a_system_configuration_unreadable_after_the_read_stage_does_not_map_to_the_witnessed_newer_publish(
    ) {
        use singlefs_core::mount::{
            NewerPublishWitness, SelectedVersionAgainstTheWitness,
            SystemConfigurationUnreadableAfterTheReadStage, SystemConfigurationUnreadableWhen,
            WitnessedCounterComparison,
        };
        let read_stage = SelectedVersionAgainstTheWitness {
            selected_version: RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
            },
            witness: NewerPublishWitness {
                witnessed_journal_counter: 4,
                comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                    selected_version_last_record_counter: 4,
                },
            },
        };
        for unreadable in [
            SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
                device: DeviceIdentity(1),
            },
            SystemConfigurationUnreadableWhen::AcquisitionReadsAnUndecidableWitness {
                first_device_reporting_a_read_error: DeviceIdentity(0),
            },
            SystemConfigurationUnreadableWhen::AcquisitionFindsADeviceWithoutAnySelfVerifiedSlot {
                device: DeviceIdentity(1),
            },
        ] {
            let refusal = MountError::SystemConfigurationUnreadableAfterTheReadStage(Box::new(
                SystemConfigurationUnreadableAfterTheReadStage {
                    read_stage,
                    unreadable,
                },
            ));
            assert_eq!(
                refusal_reason_of_mount_error(&refusal),
                ObservedRefusalReason::Unexplained,
                "读阶段之后系统配置又读不出（{unreadable:?}）模型没有这一条理由，不并进判据 N-配置 那一条"
            );
        }
    }

    /// 回退候选集的四条排除各映射到自己那一条理由、互不相同（此前「不在候选集里」映射成「低于 F、被抛弃」两条之一）；
    /// 「目标那一版树表 0 条」是第四条（D23（journal 的角色与格式） 已定项 14：候选集只收带文件的根）。原来的「落在一圈窗口外」那一条
    /// 用户 2026-10-04 定 RGall 拿掉（D16（发布语义） 已定项 1「回退候选集」）。
    #[test]
    fn each_rollback_candidate_exclusion_maps_to_its_own_reason() {
        let target = RollbackTarget {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(7),
        };
        let mapped: Vec<ObservedRefusalReason> = [
            RollbackCandidateExclusion::NotInRing,
            RollbackCandidateExclusion::BelowEffectiveFloor,
            RollbackCandidateExclusion::OnAbandonedTimeline,
            RollbackCandidateExclusion::VersionWithoutFile,
        ]
        .into_iter()
        .map(|exclusion| RollbackError::TargetNotACandidate { target, exclusion })
        .map(|error| refusal_reason_of_rollback_error(&error))
        .collect();
        assert_eq!(
            mapped,
            vec![
                ObservedRefusalReason::Explained(ModelRefusalReason::RollbackTargetNotInRing),
                ObservedRefusalReason::Explained(
                    ModelRefusalReason::RollbackTargetBelowEffectiveFloor
                ),
                ObservedRefusalReason::Explained(
                    ModelRefusalReason::RollbackTargetOnAbandonedTimeline
                ),
                ObservedRefusalReason::Explained(ModelRefusalReason::RollbackTargetWithoutFile),
            ]
        );
    }

    /// 改 size 被拒（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K12）：N 按整单元取整溢出映射到模型自己那一条理由；
    /// 别的几种（健康的内存盘上走不到）是说不清的理由。第一个文件之外的 inode 改 size 不再有拒（判决
    /// `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S6 删了那个成员）。
    /// 读—改—写读不回要重写的旧单元（部分写与截断、变长重写尾单元同一个成员，判决 `research/prompts/m4-partial-write-r1-main-verification.md`
    /// 第二节 R3）：两样读回失败都映射到模型「读不回旧单元」那一条理由，不归「说不出理由」。判别力：映射改回「说不出理由」时这一条红
    /// （`crates/mutations.tsv` 一行）。
    #[test]
    fn an_old_data_unit_not_read_back_maps_to_the_models_reason_for_it_and_not_to_an_unexplained_one(
    ) {
        let mapped: Vec<ObservedRefusalReason> = [
            OldDataUnitReadBackFailure::UnreadableAtTheHintsAndAtTheMappedLocations,
            OldDataUnitReadBackFailure::UnreadableAtTheHintsAndMissingFromTheMapping,
            OldDataUnitReadBackFailure::DisagreesWithTheLookupPath {
                detail: "单元头解不开",
            },
        ]
        .into_iter()
        .map(|read_back| {
            refusal_reason_of_publish_error(&PublishError::TruncationRefused(
                TruncationRefusal::OldDataUnitToRewriteNotReadBack {
                    inode: InodeNumber(7),
                    unit: DataUnitIndexInFile(2),
                    read_back,
                },
            ))
        })
        .collect();
        assert_eq!(
            mapped,
            vec![
                ObservedRefusalReason::Explained(
                    ModelRefusalReason::OldDataUnitToRewriteNotReadBack
                );
                3
            ],
            "三种读回失败都是「读不回旧单元」那一条理由"
        );
    }

    #[test]
    fn each_truncation_refusal_the_model_has_a_reason_for_maps_to_its_own_reason() {
        let mapped: Vec<ObservedRefusalReason> = [
            TruncationRefusal::TargetSizeRoundedUpToAWholeDataUnitOverflows {
                target_size_in_bytes: u64::MAX,
            },
            TruncationRefusal::InodeIsNotALiveRecord { inode: 1 },
            TruncationRefusal::NoTruncationIntentToContinue,
        ]
        .into_iter()
        .map(|refusal| refusal_reason_of_publish_error(&PublishError::TruncationRefused(refusal)))
        .collect();
        assert_eq!(
            mapped,
            vec![
                ObservedRefusalReason::Explained(
                    ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow
                ),
                ObservedRefusalReason::Unexplained,
                ObservedRefusalReason::Unexplained,
            ]
        );
    }

    /// 每一版的文件内容、整张实例表、全部角色的分配代都真的拿实现交回的东西比过（代码审阅第 12 条），不只在形状上对：
    /// 四段带挂载的短历史跑下来一条新发现都没有，四样计数都大于 0——胶水从实现的输出里解出了内容与实例表，
    /// 树表 0 条的那几版比过重写的角色集合。
    #[test]
    #[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
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

    /// 胶水把实现交回的 inode 叶容器序号映到模型角色那一句，逐片单测（第 30 批判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md`
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

    /// 模型模块只用格式常量那一个 crate（D13（验证路线） 已定项 5）：`model.rs` 里注释之外的每一行都不提 `singlefs_core`、`singlefs_checker`，
    /// `use` 只有 `std` 与 `singlefs_format`（测试模块的 `use super::*` 除外）。胶水（这个文件）可以用 core。
    #[test]
    fn the_model_module_uses_only_the_standard_library_and_the_format_constants() {
        let source = include_str!("model.rs");
        let code_lines: Vec<&str> = source
            .lines()
            .map(str::trim_start)
            .filter(|line| !line.starts_with("//"))
            .collect();
        assert!(code_lines.len() > 100, "读到的是 model.rs 本身");
        for line in &code_lines {
            assert!(
                !line.contains("singlefs_core") && !line.contains("singlefs_checker"),
                "模型模块里有一行提到了实现或 checker：{line}"
            );
            if line.starts_with("use ") {
                assert!(
                    line.starts_with("use std::")
                        || line.starts_with("use singlefs_format::")
                        || *line == "use super::*;",
                    "模型模块的 use 只许 std 与 singlefs_format：{line}"
                );
            }
        }
    }
}
