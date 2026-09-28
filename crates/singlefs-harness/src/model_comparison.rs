//! 理想模型与实现之间那层胶水（里程碑「覆盖写、释放、回退与复用」增补 3 第 2 件）：把实现交回的东西（`singlefs_core` 的类型）换成模型的观测，
//! 把实现的错误成员映射到模型的拒绝理由。D13（验证路线） 已定项 5 管的是模型本身（`model.rs` 只 `use singlefs_format`）；
//! 拿实现结局与模型比的这一层可以用 core 的类型。
//!
//! 映射只做「这个成员说的是哪条理由」，不做判断，按成员与它的判别字段映射、不看给人看的文字：成员说得出条款理由的映射过去，
//! I/O、盘坏、走读失败、第一版不支持的池形状（小盘写满、各盘落点不一致）这类模型里没有的一律 `Unexplained`（不建崩溃与设备错、
//! 两块等大盘的历史里它们都不该出现）。

use std::collections::{BTreeMap, BTreeSet};

use singlefs_core::address::{CheckpointTxg, DataUnitIndexInFile, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::{AllocationRecord, PlacementRefusal};
use singlefs_core::block_device::BlockDeviceError;
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::instance_table::{
    InstanceTableChainRecord, InstanceTablePage, InstanceTablePageIndex,
};
use singlefs_core::mount::{
    InstanceRow, MountError, Mounted, RollbackCandidateExclusion, RollbackError,
    StillUnreadableAfterOneReread,
};
use singlefs_core::pointer::LocationEntry;
use singlefs_core::recovery::{RecoveryOutcome, RecoveryReport};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::RootRingSlot;
use singlefs_core::transaction::{
    AllocationRecordsOfTheVersionWithoutFile, PoolVersion, PublishError, PublishedUnit,
    TransactionOutput, TransactionUnit, VersionWithoutFilePublishOutput,
};
use singlefs_core::unit::{data_unit_payload, parse_data_unit};

use crate::model::{
    ModelCheckpointTxg, ModelDeviceIdentity, ModelInstanceGeneration, ModelInstanceRow,
    ModelJournalCounter, ModelRefusalReason, ModelRingPosition, ModelRootKey, ModelUnitRole,
    ObservedAllocationRecord, ObservedEffect, ObservedFile, ObservedInstanceTable,
    ObservedReadBack, ObservedRefusalReason, ObservedRoot, ObservedUnitAllocationRecords,
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

/// 实现的一个单元是模型的哪个角色。分配记录树根之下的节点不对应角色（`None`）：D8（核心索引结构） 已定项 14 按绝对槽号按位置寻址，
/// 一次发布重写根之下哪几片由这次动了哪些槽定，模型不记落点、不记是哪几片（占槽只取上界）；那几片的分配代、位置与记录归池级 checker 判
/// （I-1.1 位置、I-3.10 每个节点都有记录）。
#[must_use]
pub fn model_unit_role(unit: TransactionUnit) -> Option<ModelUnitRole> {
    let role = match unit {
        // 模型罩的发布（第一个文件版本 / 覆盖写、写行、暖机空发布）写的文件恒只有一个数据单元：多单元只会从
        // `publish_sequential_write` 那条路径出来，而随机历史与层 0 的固定脚本一次都不调它。
        TransactionUnit::Data(index) => {
            assert_eq!(
                index,
                DataUnitIndexInFile::FIRST,
                "模型今天只罩一个数据单元的文件"
            );
            ModelUnitRole::Data
        }
        // 模型罩的文件恒一个数据单元、只有 inode 1：extent 树上段只有根兼叶，那个单元内嵌在它的条目里（D8（核心索引结构） 已定项 14），
        // 没有下段、上段也没有根之下的节点。
        TransactionUnit::ExtentRoot => ModelUnitRole::ExtentRoot,
        TransactionUnit::ExtentLowerNode(position) => {
            panic!("模型今天只罩一个数据单元的文件（内嵌、没有下段）：{position:?}")
        }
        TransactionUnit::ExtentUpperNodeBelowTheRoot(position) => {
            panic!("模型今天只罩 inode 1 一个文件（上段只有根兼叶）：{position:?}")
        }
        // 模型罩的三种发布（第一个文件版本 / 覆盖写、写行、暖机空发布）里 inode 树恒只有最左那一片叶容器：
        // 第二片只会从 `publish_new_inodes` 那条路径出来，而随机历史与层 0 的固定脚本一次都不调它
        // （`history.rs` 的操作集合里没有「建 inode」，`ModelPublishKind` 也只有那三种）。
        TransactionUnit::InodeLeafContainer(index) => {
            assert_eq!(
                index,
                InodeLeafContainerIndexInTree::LEFTMOST,
                "模型今天只罩一片叶容器的那几种发布"
            );
            ModelUnitRole::InodeLeaf
        }
        TransactionUnit::InodeRoot => ModelUnitRole::InodeRoot,
        TransactionUnit::AllocationTree => ModelUnitRole::AllocationTree,
        TransactionUnit::AllocationTreeNodeBelowTheRoot(_) => return None,
        // 模型罩的池两块盘、15 行记账，映射条目恒 6 条：两棵树恒只有一个节点（根兼叶），根之下的节点只在压小容量的
        // 只供测试的开关下、或记账行装不下一个节点的池（80 块盘起）上出现，随机历史与层 0 的固定脚本都不装那个开关。
        TransactionUnit::AccountingTreeNodeBelowTheRoot(position) => {
            panic!("模型今天只罩记账树一个节点的池：{position:?}")
        }
        TransactionUnit::AccountingTree => ModelUnitRole::AccountingTree,
        TransactionUnit::MappingTreeNodeBelowTheRoot(position) => {
            panic!("模型今天只罩中央映射树一个节点的池：{position:?}")
        }
        TransactionUnit::MappingTree => ModelUnitRole::MappingTree,
        TransactionUnit::TreeTable => ModelUnitRole::TreeTable,
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
    }
}

/// 这一版的文件内容：文件第 0 个数据单元（模型只罩一个数据单元的文件）的载荷，按单元头里的声明长度取。
#[must_use]
pub fn observed_file_of_file_version(output: &TransactionOutput) -> ObservedFile {
    let Some(data_unit) = output
        .units
        .iter()
        .find(|unit| unit.identity == TransactionUnit::Data(DataUnitIndexInFile::FIRST))
    else {
        return ObservedFile::Undecodable {
            what: "这一版的 units 里没有文件第 0 个数据单元".to_string(),
        };
    };
    match parse_data_unit(&data_unit.bytes)
        .and_then(|header| data_unit_payload(&data_unit.bytes, header.declared_length))
    {
        Ok(payload) => ObservedFile::Decoded(payload.to_vec()),
        Err(error) => ObservedFile::Undecodable {
            what: format!("文件第 0 个数据单元解不开：{error:?}"),
        },
    }
}

/// 实例表链上一片的角色是第几片。
fn instance_table_page_of_role(identity: TransactionUnit) -> Option<InstanceTablePageIndex> {
    match identity {
        TransactionUnit::InstanceTable => Some(InstanceTablePageIndex::FIRST),
        TransactionUnit::InstanceTablePageAfterTheFirst(page) => Some(page),
        TransactionUnit::Data(_)
        | TransactionUnit::ExtentLowerNode(_)
        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
        | TransactionUnit::ExtentRoot
        | TransactionUnit::InodeLeafContainer(_)
        | TransactionUnit::InodeRoot
        | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
        | TransactionUnit::AllocationTree
        | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
        | TransactionUnit::AccountingTree
        | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        | TransactionUnit::MappingTree
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
        let Some(page) = InstanceTablePage::parse(bytes, page_index) else {
            return Err(InstanceTableNotDecodedFromTheUnits::PageUndecodable(
                page_index,
            ));
        };
        rows.extend(page.rows.iter().map(model_instance_row));
        match page.chain {
            InstanceTableChainRecord::LastPage => return Ok(rows),
            InstanceTableChainRecord::NextPage(_) => page_index = page_index.next(),
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
/// 带文件的一版与树表 0 条上写行的那一版共用这一处。
fn allocation_records_of_every_role(
    units: &[PublishedUnit],
    allocation_records: &[AllocationRecord],
    root: &RootRecord,
) -> Vec<(ModelUnitRole, Vec<ObservedAllocationRecord>)> {
    let mut records_of_roles: Vec<(ModelUnitRole, Vec<ObservedAllocationRecord>)> = units
        .iter()
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
    records_of_roles
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
    }
}

#[must_use]
pub fn observed_root_of_pool_version(version: &PoolVersion) -> ObservedRoot {
    match version {
        PoolVersion::WithFile(output) => observed_root_of_file_version(output),
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
        PoolVersion::WithFile(output) => (observed_root_of_file_version(output), None),
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
        PublishError::ContentExceedsDataUnit { .. } => {
            explained(ModelRefusalReason::ContentExceedsDataUnitPayload)
        }
        PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn { .. } => {
            explained(ModelRefusalReason::FirstFileVersionDoesNotFollowTheVersionItBuildsOn)
        }
        PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile { .. } => {
            explained(ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile)
        }
        // 释放判定路径的五种：上一版的映射或分配记录与上一版对不上、盘上那条指针的两条位置条目不同槽，健康的历史里不该出现。
        // 分配记录树重写集合迭代不收敛那一条同理：只有强制复用窗口为 0 的只供测试的开关下才可能走到，随机历史不装那个开关。
        // inode 树写入被拒那一条同样：随机历史一次都不调 `publish_new_inodes`，
        // 而它跑的那几种发布每次最多改一片叶容器、重写的角色最多九个。
        // 多层码 2 树算不出形状那一条同理：两棵树装不下一个节点时分裂、不拒，只有长到 257 层或上一版的形状按分隔 key 走不通才拒，
        // 随机历史里两棵树恒只有一个节点；它出现就是模型与实现对不上。
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
        // 第一个文件版本读不出那一版的树表、水位离 u64::MAX 不到八个号：健康的内存盘上都不该出现。
        | PublishError::TreeTableOfTheVersionToBuildOnUnreadable { .. }
        | PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(_)
        // 冻结着一次没重发的发布：随机历史里一次发布失败就整段停下、不接着发，健康的内存盘上不该出现。
        | PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
        // 一次发布切出来的记录多于在飞上限（随机历史每次最多写几个数据单元，远在上限之下）、要接的那一版 txg 已是 u64::MAX（坏镜像才有）：
        // 模型里没有它们的理由。
        | PublishError::JournalRecordsOfThePublishExceedTheLimit { .. }
        | PublishError::NextCheckpointTxgPastTheTopOfItsRange { .. }
        | PublishError::BlockDevice(_) => ObservedRefusalReason::Unexplained,
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
/// 候选集四条（D23（journal 的角色与格式） 已定项 14：根环里、txg ≥ F_生效、按现行那一版的实例表判仍然有效、带文件），各映射到自己那一条。
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
        | RollbackError::ResurrectedUnitCopyUnreadableOrMismatched { .. }
        // 判候选集读根环或算 F_生效 重读一次仍读坏（实审 A3b Q7）：读路径上注入故障、或同一个进程里一次根槽写被说谎的设备吞掉
        // （那一槽这个进程知道住着根、盘上却不是它写的那一份）才有，与 `MountError::Recovery(_)` 同一处置。
        | RollbackError::CandidateJudgementStillUnreadableAfterOneReread(_) => {
            ObservedRefusalReason::Unexplained
        }
    }
}

/// 零单元发布只会报块设备错：内存盘不报错，模型里没有它的理由。
#[must_use]
pub fn refusal_reason_of_block_device_error(_error: &BlockDeviceError) -> ObservedRefusalReason {
    ObservedRefusalReason::Unexplained
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
        // 树表 0 条、而实例表已经不是 mkfs 那一片：零故障走得到（写过行的那一版上再挂载一次），模型照代码今天的读法划进必须拒。
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
                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. } => {
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
        | MountError::Acquisition(_)
        | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
        | MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { .. }
        | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
        | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
        | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. }
        | MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_)
        | MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided { .. }
        // 盘表里同一个身份交了几次、调用方参数与盘上系统配置不一致、有盘落后于现行那一版又缺它的单元：随机历史每次都交整池两块盘
        // （没换过盘）、用建池的那一份参数，走不到。
        | MountError::DeviceIdentitiesHandedInMoreThanOnce { .. }
        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnits { .. }
        // 盘上的实例代号或 txg 已到顶：坏镜像才有。
        | MountError::SequenceNumberPastTheTopOfItsRange(_) => ObservedRefusalReason::Unexplained,
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
        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnits { .. }
        | MountError::SequenceNumberPastTheTopOfItsRange(_)
        | MountError::NewerStateStillUnreadableAfterOneReread(_) => None,
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
                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. } => None,
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
        | RollbackError::CurrentVersionUnitNotReleasable { .. }
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
        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnits { .. }
        | MountError::SequenceNumberPastTheTopOfItsRange(_)
        | MountError::NewerStateStillUnreadableAfterOneReread(_) => None,
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
/// 层 0 的 oracle（`crash::oracle_violation_for_versions`）判该读出哪一版拿的也是 `effective_root`。
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        IdealModel, ModelDeviceIdentity, ModelDisagreementAspect, ModelPoolGeometry,
        ObservedOutcome,
    };
    use singlefs_core::address::{DeviceIdentity, SlotNumber};
    use singlefs_core::allocator::CommitGeneratedDeviceAnswer;
    use singlefs_core::mount::RollbackTarget;
    use singlefs_core::transaction::TransactionUnit;
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

    /// 回退候选集的四条排除各映射到自己那一条理由、互不相同（此前「不在候选集里」映射成「低于 F、被抛弃」两条之一）；
    /// 「目标那一版树表 0 条」是第四条（D23（journal 的角色与格式） 已定项 14：候选集只收带文件的根）。
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

    /// 每一版的文件内容、整张实例表、全部角色的分配代都真的拿实现交回的东西比过（代码审阅第 12 条），不只在形状上对：
    /// 四段带挂载的短历史跑下来一条新发现都没有，四样计数都大于 0——胶水从实现的输出里解出了内容与实例表，
    /// 树表 0 条的那几版比过重写的角色集合。
    #[test]
    #[ignore = "harness 耗时用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
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
