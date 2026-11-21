//! 分配记录与实例表链：分配记录落在池的几何里、落点在它那块盘的单元区里，按根读分配记录（带文件的一版与树表 0 条的一版），
//! 沿链读实例表，树表条目数。

use super::*;

/// 盘上读来的分配记录逐条对这个池的几何判一遍：这几个字段可以是任何值，它们不是不变量。
/// 读分配记录树时两道判分得开：前一道是**字段表**与**位置**（条目宽够不够装下 20 字节的记录、记录落不落在它所在叶里，
/// `crate::allocation_record_tree::read_allocation_record_tree`），这一道是**几何**（设备身份、槽号、跨度、两条记录罩不罩同一个槽）。
/// 分配器那一侧的下标、位图长度与两条断言全部按这两道已经判过来写（`allocator::DeviceFreeMap::index` / `mark_allocated` 的消息指着这里）。
///
/// 四样各对着 panic 面普查里的一条：设备身份不在池里（R9，`PoolAllocator::rebuild_from_records` 的 `expect`）、
/// 槽号在单元区起点之下（R6，`DeviceFreeMap::index` 的减法）、跨度越过单元区末尾与同一块盘上两条记录罩住同一个槽
/// （R6 / R8，`mark_allocated` 的两条断言）。
///
/// 单元区槽数只有 [`crate::allocator::unit_area_slots_of_device_starting_at`] 一处定义，分配器的位图长度读的也是它。
/// 前三样逐条走 [`placement_lies_in_the_unit_area_of_its_device`]；mkfs 那一版的两个单元进分配器之前也走这一道
/// （`mount::format_time_allocator`，代码审阅第 32 条）。
///
/// # Errors
/// 四样里任一样不成立 ⇒ [`RecoveryFailure::AllocationRecordOutsideThePoolGeometry`]，`what` 说清是哪一样；
/// 记录所在那块盘末尾在单元区起点之前 ⇒ [`RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart`]。
pub(crate) fn allocation_records_fit_the_pool_geometry(
    reader: &dyn PoolReader,
    unit_area_start: UnitAreaStart,
    records: &[AllocationRecord],
) -> Result<(), RecoveryFailure> {
    let mut covered_slots: BTreeSet<(DeviceIdentity, u64)> = BTreeSet::new();
    for record in records {
        let span = u64::from(record.span_slots);
        placement_lies_in_the_unit_area_of_its_device(
            reader,
            unit_area_start,
            record.device,
            record.slot,
            span,
        )?;
        for slot in record.slot.0..record.slot.0 + span {
            if !covered_slots.insert((record.device, slot)) {
                return Err(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
                    what: "同一块盘上两条分配记录罩住同一个槽",
                });
            }
        }
    }
    Ok(())
}

/// 盘上读来的一个落点 `(device, [slot, slot + span))` 落不落在那块盘的单元区里（单元区从 `unit_area_start` 起：择到的系统配置按环长现算的那个，
/// [`unit_area_start_of_the_chosen_system_configuration`]）：设备身份在池里、槽号不在单元区起点之下、跨度不越过单元区末尾
/// （[`allocation_records_fit_the_pool_geometry`] 逐条记录判的前三样；影子账拿被抛弃根的指针槽号进分配器之前也走这一处，代码审阅第 32 条）。
/// 分配器的位图下标与「跨度越过单元区末尾」那几句断言按这一判已经做过来写（`allocator::DeviceFreeMap::index`、`isolate`、`mark_allocated`）。
///
/// # Errors
/// 设备身份不在池里、槽号在单元区起点之下、跨度越过单元区末尾 ⇒ [`RecoveryFailure::AllocationRecordOutsideThePoolGeometry`]；
/// 那块盘末尾在单元区起点之前 ⇒ [`RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart`]。
pub(crate) fn placement_lies_in_the_unit_area_of_its_device(
    reader: &dyn PoolReader,
    unit_area_start: UnitAreaStart,
    device: DeviceIdentity,
    slot: SlotNumber,
    span_slots: u64,
) -> Result<(), RecoveryFailure> {
    // 「这块盘在不在池里」只这一处判：`PoolReader::device_size_in_bytes` 对池外的盘交 `None`
    // （每个实现都从 `device_identities` 那张表里找），另写一句 `device_identities().contains(…)`
    // 是同一条判定的第二份手抄，两份会分叉（`code-discipline.md`「重复要生成，不许手抄」）。
    let device_bytes = reader.device_size_in_bytes(device).ok_or(
        RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
            what: "分配记录的设备身份不在池里",
        },
    )?;
    if slot.0 < unit_area_start.slot().0 {
        return Err(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
            what: "分配记录的槽号落在单元区起点之下",
        });
    }
    let unit_area_slots = unit_area_slots_of_device_starting_at(device_bytes, unit_area_start)
        .map_err(|DeviceEndsBeforeTheUnitAreaStart { device_bytes, .. }| {
            RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart {
                device,
                device_bytes,
            }
        })?;
    // 槽号是盘上的 6 字节（< 2⁴⁸）、跨度 ≤ 2¹⁶ 由调用方的字段宽给出，两个加法都装得进 u64。
    let unit_area_end_slot = unit_area_start.slot().0 + unit_area_slots;
    if slot.0 + span_slots > unit_area_end_slot {
        return Err(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
            what: "分配记录的跨度越过单元区末尾",
        });
    }
    Ok(())
}

/// 池里每块盘的完整槽数都到得了单元区起点 `unit_area_start`（择到的系统配置按环长现算的那个；代码审阅第 35 条）：
/// 可写挂载建空闲图（`allocator::DeviceFreeMap::with_unit_area_start`）的前置条件，在择系统配置之后、读根环之前逐盘判。
/// 只读盘的字节数，不读盘。
///
/// # Errors
/// 按 [`PoolReader::device_identities`] 次序第一块末尾在单元区起点之前的盘 ⇒ [`RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart`]。
pub fn every_device_reaches_the_unit_area_start(
    reader: &dyn PoolReader,
    unit_area_start: UnitAreaStart,
) -> Result<(), RecoveryFailure> {
    for device in reader.device_identities() {
        let device_bytes = reader
            .device_size_in_bytes(device)
            .expect("device_identities 列出的盘，device_size_in_bytes 交得出它的字节数（PoolReader 的契约）");
        unit_area_slots_of_device_starting_at(device_bytes, unit_area_start).map_err(
            |_ends_before| RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart {
                device,
                device_bytes,
            },
        )?;
    }
    Ok(())
}

/// 一条根引用的分配记录（树表 → 分配记录树根节点）：回退的影子账要读每条被抛弃根的账。第 0 代树表（没有分配记录树）给空。
/// 分配记录树根的位置提示读不出时经这条根的中央映射回退（D19（块指针的结构与宽度预算） 已定项 8，与冷走读同一条
/// `read_mapped_tree_node_via_hint_then_central_mapping`）；树表是自举豁免，只按根记录里的位置条目读。
/// 回退的次数这里不交出去：调用方（影子账、重建分配器）今天没有接多跳观测点的口子。
///
/// # Errors
/// 树表读不到、解不开，同一种树有两条（[`TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE`]）；
/// 分配记录树根提示读不出且映射里没有它（`MappingMiss`）、映射落点也读不出（`MappingStillUnreadable`）、
/// 映射树根读不出或解不开、分配记录树根解不开；条目宽判红（见 [`read_allocation_record_tree`]）或结构值判红
/// （见 `allocation_records_fit_the_pool_geometry`）。
///
/// 单元区从 `unit_area_start` 起（择到的系统配置按环长现算的那个，[`unit_area_start_of_the_chosen_system_configuration`]）：
/// 分配记录落不落在单元区里按它判。
pub fn allocation_records_under_root_in_the_unit_area_starting_at(
    reader: &dyn PoolReader,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
) -> Result<Vec<AllocationRecord>, RecoveryFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    let tree_table_entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)?;
    let Some(allocation_entry) = tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_ALLOCATION)
    else {
        return Ok(Vec::new());
    };
    // 读的是别的根（影子账、抬 F 的上限）的账：映射树只按「拼得成一棵树」核，与这里读分配记录树同一个口径。
    let central_mapping_root = CentralMappingTreeWithBytesReadOnFirstUse::new(
        reader,
        root,
        CodeTwoTreeHeaderJudgement::OnlyWhatTheShapeNeeds,
    );
    let mut stale_location_hint_hops_not_exposed_by_this_reader = 0usize;
    // 分配记录树按绝对槽号按位置寻址、可以多层（D8（核心索引结构） 已定项 14）：整棵读回来，节点进映射、提示读不出经这条根的中央映射回退。
    let tree = read_allocation_record_tree(
        &allocation_entry.root,
        &AllocationRecordTreeGeometry::of_reader(reader),
        allocation_entry.tree,
        AllocationRecordTreeHeaderJudgement::OnlyWhatThePositionsNeed,
        root,
        unit_filesystem_identifier(&root.filesystem_identifier),
        &mut |pointer: &NodePointer| {
            read_mapped_tree_node_via_hint_then_central_mapping(
                reader,
                pointer,
                MappedTreeNodeClass::IndexNode,
                &|mapping_key: &[u8]| central_mapping_root.locations_of_key(mapping_key),
                &mut stale_location_hint_hops_not_exposed_by_this_reader,
            )
        },
    )?;
    allocation_records_fit_the_pool_geometry(reader, unit_area_start, &tree.records)?;
    Ok(tree.records)
}

/// 同 [`allocation_records_under_root_in_the_unit_area_starting_at`]，单元区起点先择一次系统配置现算
/// （[`choose_system_configuration`]：多读每块盘的两个系统配置槽）。给手里没有择好的系统配置的调用方（用例、实验装置）。
///
/// # Errors
/// 择不到系统配置（[`choose_system_configuration`] 的各个成员）；其余同带起点的那一份。
pub fn allocation_records_under_root(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Result<Vec<AllocationRecord>, RecoveryFailure> {
    allocation_records_under_root_in_the_unit_area_starting_at(
        reader,
        unit_area_start_of_the_pool(reader)?,
        root,
    )
}

/// 树表 0 条的那一版自己那棵分配记录树（根指针住根记录那一项，C512（树表 0 条的一版上被换下的单元记在哪））：整棵读回来
/// （D8（核心索引结构） 已定项 14：挂载时分配记录树整棵读），交回它的节点与指针、全部记录。树号 0、节点豁免映射，只按位置条目读。
/// 指针全零 ⇒ `None`，那是 mkfs 的第 0 代（那一版的账由实例表与树表两条指针直接算）。
///
/// # Errors
/// 分配记录树的节点读不到、解不开、位置对不上、节点头里的自描述与指着它的指针对不上（`crate::allocation_record_tree::read_allocation_record_tree`，
/// `EveryHeaderAgainstItsReference`）；结构值判红（`allocation_records_fit_the_pool_geometry`）。
///
/// 单元区从 `unit_area_start` 起（择到的系统配置按环长现算的那个）：分配记录落不落在单元区里按它判。
pub fn allocation_records_of_version_without_file_in_the_unit_area_starting_at(
    reader: &dyn PoolReader,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
) -> Result<Option<AllocationRecordTreeOfTheVersionWithoutFile>, RecoveryFailure> {
    if root.allocation_record_tree_root == NodePointer::empty_root() {
        return Ok(None);
    }
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    // 这是可写挂载要接着写的那一版自己的账：节点头照冷走读判全（树 ID、key 宽、出生身份、fsid，代码审阅第 24 / 33 条）。
    let tree = read_allocation_record_tree(
        &root.allocation_record_tree_root,
        &AllocationRecordTreeGeometry::of_reader(reader),
        TreeIdentifier(crate::transaction::TREE_IDENTIFIER_NONE),
        AllocationRecordTreeHeaderJudgement::EveryHeaderAgainstItsReference,
        root,
        unit_filesystem_identifier(&root.filesystem_identifier),
        &mut |pointer: &NodePointer| {
            read_unit_via_locations(reader, &pointer.locations, node_bytes)
        },
    )?;
    allocation_records_fit_the_pool_geometry(reader, unit_area_start, &tree.records)?;
    // 这棵树的节点指针下一次发布照抄或换下时写回内部条目：按 I-2.5 判（`pointers_of_the_rebuilt_version_ascend_by_device` 同一道）。
    for (_, pointer, _) in &tree.nodes {
        pointer
            .location_entries_ascend_by_device()
            .map_err(|_not_ascending| location_entries_not_ascending())?;
    }
    Ok(Some(AllocationRecordTreeOfTheVersionWithoutFile {
        version: tree.version(),
        records: tree.records,
    }))
}

/// 同 [`allocation_records_of_version_without_file_in_the_unit_area_starting_at`]；根记录那一项全零（没有分配记录树）时直接交 `None`、
/// 不读系统配置，否则单元区起点先择一次系统配置现算（[`choose_system_configuration`]）。给手里没有择好的系统配置的调用方。
///
/// # Errors
/// 择不到系统配置（[`choose_system_configuration`] 的各个成员）；其余同带起点的那一份。
pub fn allocation_records_of_version_without_file(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Result<Option<AllocationRecordTreeOfTheVersionWithoutFile>, RecoveryFailure> {
    if root.allocation_record_tree_root == NodePointer::empty_root() {
        return Ok(None);
    }
    allocation_records_of_version_without_file_in_the_unit_area_starting_at(
        reader,
        unit_area_start_of_the_pool(reader)?,
        root,
    )
}

/// 一条根指着的整张实例表（全部行）：沿链读到「无下一片」为止（[`instance_table_chain_of_root`]）；
/// 任一片读不出或解不开都是 `None`（读不出的由走读另报）。
#[must_use]
pub fn instance_table_of_root(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Option<InstanceTableRecords> {
    instance_table_chain_of_root(reader, root)
        .ok()
        .map(|chain| chain.records)
}

/// 一条根指着的整张实例表：各片的行按链上的次序接起来，连同每一片的指针（D18（块里携带什么信息） 已定项 11）。
pub struct InstanceTableChain {
    pub records: InstanceTableRecords,
    /// 每一片的指针，按片序号：第 0 片是根记录里那一条，第 k 片是第 k − 1 片链指针记录里那一条。至少一片。
    pub page_pointers: Vec<NodePointer>,
}

/// 沿链读的结果：读到哪儿认得出哪几片的指针，与整张表读没读成。影子账要前一样（读不全的链也要把认得出的那几片隔离），
/// 别的读者要后一样。
struct InstanceTableChainRead {
    /// 认得出的每一片的指针，按片序号（第 0 片恒在）：一片读不出、解不开时，它自己的指针也在（上一片的链指针记录给的），
    /// 它后面的认不出。
    page_pointers: Vec<NodePointer>,
    outcome: Result<InstanceTableRecords, RecoveryFailure>,
}

/// 从根记录里那条指针读第 0 片，按每一片的链指针记录读下一片，读到「无下一片」为止。
///
/// 迭代上界：第 k 轮要求读到的单元身份里容器号是 k（[`InstanceTablePage::parse`]），两轮读到同一个单元就要它的容器号
/// 同时等于两个数 ⇒ 每一轮读的是盘上不同的单元，轮数不超过一块盘装得下的 32 KiB 单元数；链指回前面任何一片（成环）
/// 在身份那一判上断掉。跨轮带的是认得出的各片指针与已经接起来的行；提前出口只有读不出、解不开两个。
fn read_instance_table_chain<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    root: &RootRecord,
    chain_pointer_head_fields: PointerHeadFieldsJudgement,
) -> InstanceTableChainRead {
    let data_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    let mut page_pointers = vec![root.instance_table];
    let mut page_index = InstanceTablePageIndex::FIRST;
    let mut rows = Vec::new();
    loop {
        let pointer = *page_pointers.last().expect("第 0 片那一条恒在");
        let bytes = match read_unit_via_locations(reader, &pointer.locations, data_bytes) {
            Ok(bytes) => bytes,
            Err(failure) => {
                return InstanceTableChainRead {
                    page_pointers,
                    outcome: Err(failure),
                }
            }
        };
        let Some(page) = InstanceTablePage::parse_with_chain_pointer_head_fields_judgement(
            &bytes,
            page_index,
            chain_pointer_head_fields,
        ) else {
            return InstanceTableChainRead {
                page_pointers,
                outcome: Err(RecoveryFailure::UnitMalformed {
                    what: "实例表的一片（身份不是 (0, 4, 片序号, 0)、链指针记录或行不合法）",
                }),
            };
        };
        rows.extend(page.rows);
        match page.chain {
            InstanceTableChainRecord::LastPage => {
                return InstanceTableChainRead {
                    page_pointers,
                    outcome: Ok(InstanceTableRecords { rows }),
                }
            }
            InstanceTableChainRecord::NextPage(next_page) => {
                page_pointers.push(next_page);
                page_index = page_index.next();
            }
        }
    }
}

/// 沿链读一条根指着的整张实例表（D18（块里携带什么信息） 已定项 11：根记录直接持有第 0 片，第 k 片的链指针记录指着第 k + 1 片）。
/// 一片读不出或解不开，整张表就不可读（D18（块里携带什么信息） 已定项 11「表不可读时」：指针指不到或校验不过就是表不可读）。
///
/// # Errors
/// 某一片两条位置条目都读不到校验和相符的那份 ⇒ `UnitUnreadable`；某一片解不开（身份不是 (0, 4, 片序号, 0)、
/// 链指针记录或行不合法）⇒ `UnitMalformed`。
pub fn instance_table_chain_of_root<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    root: &RootRecord,
) -> Result<InstanceTableChain, RecoveryFailure> {
    let read = read_instance_table_chain(
        reader,
        root,
        PointerHeadFieldsJudgement::JudgedAsTheStructureIsCorruptOutsideTheFirstVersion,
    );
    read.outcome.map(|records| InstanceTableChain {
        records,
        page_pointers: read.page_pointers,
    })
}

/// 一条根指着的实例表链上认得出的每一片的指针（第 0 片恒在）：读得出的就沿链往下认，读不出、解不开的那一片自己的指针也在，
/// 它后面的认不出。影子账拿它隔离一条被抛弃根的整条链——读不全的链少隔离一片，被抛弃根在环里被覆写之前那一片就可能被发出去。
/// 所以链指针头部第一版之外的取值（加密、压缩那几段与 extent 偏移）这里不判（[`PointerHeadFieldsJudgement::NotJudgedByTheShadowLedger`]）：
/// 指针里的位置条目照样认下一片；沿链读整张表（[`instance_table_chain_of_root`]）照判。
#[must_use]
pub fn instance_table_page_pointers_as_far_as_readable<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    root: &RootRecord,
) -> Vec<NodePointer> {
    read_instance_table_chain(
        reader,
        root,
        PointerHeadFieldsJudgement::NotJudgedByTheShadowLedger,
    )
    .page_pointers
}

/// 一条根指着的树表是不是 0 条（mkfs 的第 0 代树表，第一次可写挂载的暖机根照抄它）：这条根下面还没有发布过文件版本。
///
/// # Errors
/// 树表单元读不到、解不开。
pub fn tree_table_has_no_entries(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Result<bool, RecoveryFailure> {
    Ok(tree_table_entry_count(reader, root)? == 0)
}

/// 一条根指着的树表有几条条目：0 条就是 mkfs 的第 0 代树表（这条根下面还没有发布过文件版本）。
/// 发布路径（`transaction::publish_first_file`）拿写者手里那串盘直接读，所以读者按泛型收、不要求有大小。
///
/// # Errors
/// 树表单元读不到、解不开。
pub fn tree_table_entry_count<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    root: &RootRecord,
) -> Result<usize, RecoveryFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    Ok(tree_table.entries.len())
}

/// 盘上读来的分配记录对这个池的几何判的那四样（panic 面普查 R6 / R8 / R9）：设备身份、槽号下界、跨度上界、
/// 同一块盘上两条记录罩不罩同一个槽。四样各钉一格，加一格合法的。
///
/// 为什么在这一层钉而不是只钉在坏镜像上：跨度上界那一条，坏盘输入那一档的盘是 4 GiB（单元区到槽 262144），
/// 而那条坏法能写进跨度字段的最大值是 0x7FFF = 32767 槽——从单元区起点起也够不到末尾，
/// 它实际打中的是「两条记录罩住同一个槽」。跨度上界那一条因此在镜像那一层零覆盖，只能在这里钉。
#[cfg(test)]
mod allocation_record_pool_geometry_tests {
    use super::{allocation_records_fit_the_pool_geometry, PoolReader, RecoveryFailure};
    use crate::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
    use crate::allocator::{AllocationRecord, UnitAreaStart};
    use singlefs_format::{SLOT_BYTES, UNIT_AREA_START_SLOT};

    /// 每块盘 4 GiB（与坏盘输入那一档的盘同宽）：单元区从 [`UNIT_AREA_START_SLOT`] 起到槽
    /// 4 GiB ÷ 16 KiB = 262144。
    const DEVICE_BYTES: u64 = 4 << 30;
    const UNIT_AREA_END_SLOT: u64 = DEVICE_BYTES / SLOT_BYTES;

    /// 只交得出「池里有哪几块盘、每块多大」的读者。
    struct PoolOfTwoDevicesOfFourGibibytes;

    impl PoolReader for PoolOfTwoDevicesOfFourGibibytes {
        fn device_identities(&self) -> Vec<DeviceIdentity> {
            vec![DeviceIdentity(0), DeviceIdentity(1)]
        }
        fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
            self.device_identities()
                .contains(&device)
                .then_some(DEVICE_BYTES)
        }
        fn physical_block_size_probed_on_the_device(
            &self,
            _device: DeviceIdentity,
        ) -> Option<crate::block_device::PhysicalBlockSizeInBytes> {
            unreachable!("allocation_records_fit_the_pool_geometry 不探测块宽")
        }
        fn read(
            &self,
            _device: DeviceIdentity,
            _offset: DeviceOffsetInBytes,
            _length: usize,
        ) -> Option<Vec<u8>> {
            // 这一道判只读盘的身份与大小：它拿到的是已经解好的记录，一个字节都不再读。
            unreachable!("allocation_records_fit_the_pool_geometry 不读盘上的字节")
        }
        fn journal_record_offsets_hint(
            &self,
            _device: DeviceIdentity,
            _ring_start: DeviceOffsetInBytes,
            _ring_bytes: u64,
        ) -> Option<Vec<DeviceOffsetInBytes>> {
            // 同上：这一道判不碰 journal 环。
            unreachable!("allocation_records_fit_the_pool_geometry 不扫 journal 环")
        }
    }

    fn record(device: u32, slot: u64, span_slots: u16) -> AllocationRecord {
        AllocationRecord {
            device: DeviceIdentity(device),
            slot: SlotNumber(slot),
            span_slots,
            generation: CheckpointTxg(3),
            is_released: false,
            quarantine_mark: crate::allocator::AllocationRecordQuarantineMark::NotQuarantined,
        }
    }

    /// 默认环长的池：单元区从 [`UNIT_AREA_START_SLOT`] 起（这组用例按那个常量摆记录）。
    fn judge(records: &[AllocationRecord]) -> Result<(), RecoveryFailure> {
        allocation_records_fit_the_pool_geometry(
            &PoolOfTwoDevicesOfFourGibibytes,
            UnitAreaStart::of_the_default_journal_ring(),
            records,
        )
    }

    fn outside_the_geometry(what: &'static str) -> Result<(), RecoveryFailure> {
        Err(RecoveryFailure::AllocationRecordOutsideThePoolGeometry { what })
    }

    #[test]
    fn records_inside_the_geometry_are_accepted() {
        assert_eq!(
            judge(&[
                record(0, UNIT_AREA_START_SLOT, 2),
                record(1, UNIT_AREA_START_SLOT, 2),
                record(0, UNIT_AREA_START_SLOT + 2, 2),
                record(0, UNIT_AREA_END_SLOT - 2, 2),
            ]),
            Ok(()),
            "槽号在单元区内、跨度贴着末尾也不越过、两块盘各算各的：这几条都合法"
        );
    }

    #[test]
    fn record_on_the_device_outside_the_pool_is_refused() {
        assert_eq!(
            judge(&[record(7, UNIT_AREA_START_SLOT, 2)]),
            outside_the_geometry("分配记录的设备身份不在池里"),
            "池里只有盘 0 与盘 1（R9：不判就喂进 rebuild_from_records 的 expect）"
        );
    }

    #[test]
    fn record_whose_slot_is_below_the_unit_area_is_refused() {
        assert_eq!(
            judge(&[record(0, UNIT_AREA_START_SLOT - 1, 2)]),
            outside_the_geometry("分配记录的槽号落在单元区起点之下"),
            "R6：不判就在 DeviceFreeMap::index 的减法上下溢"
        );
    }

    #[test]
    fn record_whose_span_runs_past_the_end_of_the_unit_area_is_refused() {
        assert_eq!(
            judge(&[record(0, UNIT_AREA_END_SLOT - 1, 2)]),
            outside_the_geometry("分配记录的跨度越过单元区末尾"),
            "R6：不判就在 mark_allocated 的「跨度越过单元区末尾」那条断言上"
        );
    }

    #[test]
    fn two_records_covering_the_same_slot_on_one_device_are_refused() {
        assert_eq!(
            judge(&[
                record(0, UNIT_AREA_START_SLOT, 4),
                record(0, UNIT_AREA_START_SLOT + 2, 2),
            ]),
            outside_the_geometry("同一块盘上两条分配记录罩住同一个槽"),
            "R8：不判就在 mark_allocated 的「跨度里有已分配的槽」那条断言上（I-5.4 的读路径形态）"
        );
        assert_eq!(
            judge(&[
                record(0, UNIT_AREA_START_SLOT, 2),
                record(1, UNIT_AREA_START_SLOT, 2),
            ]),
            Ok(()),
            "两盘同槽是发布路径自己的写法（D2 已定项 10），不是重叠"
        );
    }

    /// 盘上读来的带隔离记号的记录（跨度段 0x4002、0x4001，D19（块指针的结构与宽度预算） 已定项 5）按去掉记号的跨度判几何：
    /// 起在单元区起点、跨度段 0x4002 的那条只罩两槽，紧挨着它的 0x4001 那条与它不重叠。记号当成跨度读的话，头一条罩到一万六千多槽之外、
    /// 与后一条罩住同一个槽。
    #[test]
    fn records_read_with_the_quarantine_mark_are_judged_by_the_span_in_the_low_bits() {
        let read_from_disk = |slot: u64, span_field: u16| {
            let mut bytes = record(0, slot, 1).to_bytes();
            bytes[10..12].copy_from_slice(&span_field.to_le_bytes());
            AllocationRecord::parse(&bytes).expect("带隔离记号、未释放的记录解得开")
        };
        let quarantined_unit = read_from_disk(UNIT_AREA_START_SLOT, 0x4002);
        let quarantined_node = read_from_disk(UNIT_AREA_START_SLOT + 2, 0x4001);
        assert_eq!(
            (quarantined_unit.span_slots, quarantined_node.span_slots),
            (2, 1),
            "跨度取低位"
        );
        assert_eq!(
            [quarantined_unit.quarantine_mark, quarantined_node.quarantine_mark],
            [crate::allocator::AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck; 2],
            "两条都带隔离记号"
        );
        assert_eq!(
            judge(&[quarantined_unit, quarantined_node]),
            Ok(()),
            "带记号的两条按跨度 2、1 判，挨着而不重叠"
        );
    }
}
