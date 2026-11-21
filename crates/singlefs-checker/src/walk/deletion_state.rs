//! 删除态扫描：删除队列、inode 记录、数据单元与截断意图，extent 上段形状；判 I-9.5（删除队列里的号仍在树里）、I-4.6（中间态其余不变量成立） 与 I-4.7（删除中 inode 有账） 一类。

use super::*;

/// 删除态扫描要判的一版（I-9.5（删除队列里的号仍在树里） 那一行的共用判法：回退候选集里每条根，加由记录施加出来的那几版）：
/// 称呼（判红与判不了时写进说明）、它的 `checkpoint_txg`（「在谱系上」从这一版自己的 txg 起算，I-9.20 比死亡 txg 的上界）
/// 与它的树表指针、中央映射根指针（根环里的根取根记录偏移 36 与 256 起的各 86 字节，由记录施加出来的那一版取记录新根段里的那两条）：
/// 几棵树里豁免三类之外的单元按这一版的映射认位置（D19（块指针的结构与宽度预算） 已定项 5）。
pub(super) struct VersionToScanTheDeletionStateOf<'record> {
    pub(super) name: String,
    pub(super) checkpoint_txg: u64,
    pub(super) tree_table_pointer: &'record [u8],
    pub(super) mapping_root_pointer: &'record [u8],
}

/// 删除态扫描在一版上读出的一条 inode 记录里要的五段（D8（核心索引结构） 已定项 6 字段表：对象出生代偏移 8、nlink 偏移 36、size 偏移 40；
/// 截断字段 T 偏移 120、N_w 偏移 128）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct InodeRecordOfTheDeletionScan {
    pub(super) object_birth: u64,
    pub(super) nlink: u32,
    pub(super) size_in_bytes: u64,
    truncation_field_txg: u64,
    truncation_field_size_in_bytes: u64,
}

/// 删除态扫描在一版上读不全、配不上的是哪一样：那一版在射程里判不了，报部分没判、逐条列出（[`DeletionScanGap::note_as_a_part_of_the_range_left_unjudged`]）、不判红
/// 读不出、解不开、形状不对由主走读判
/// （I-2.1、I-1.1、I-1.7、I-1.10、I-9.1、I-9.27 等），这一遍只如实记「判不了」，不进 `walk_failures`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DeletionScanGap {
    TreeTablePointerAllZero,
    TreeTableUnreadable,
    TreeTableEntryShorterThanTheFieldTable,
    DeletedInodesHeadIdentifierNamesNoSingleInodeTree,
    InodeTreeNamedByNoSingleDeletedInodesEntry,
    ExtentTreeHeadIdentifierNamesNoSingleInodeTree,
    DeletedInodesRootUnreadable,
    InodeTreeUnreadable,
    ExtentTreeOfTheInodeTreeNotSingle,
    ExtentTreeUnreadable,
    AccountingTreeUnreadable,
    TombstoneTreeUnreadable,
    TombstoneTreeOfTheVersionUnreadable,
    TombstoneTreeHeadIdentifierNamesNoSingleInodeTree,
    InodeTreeNamedByNoSingleTombstoneTreeEntry,
    LineageTreeEntryNotSingle,
    LineageTreeUnreadable,
    LineageRecordsInconsistent,
    QueuedInodeWithoutARecord,
    RetirementTreeUnreadable,
    RetirementTreeHeadIdentifierNamesNoSingleInodeTree,
    ParentVersionNotSingle,
    LoggedOperationsTreeUnreadable,
    DataUnitWithinTheSizeUnreadable,
    DataUnitBeyondTheTruncationFieldUnreadable,
    InodeTreeOfTheVersionNotSingle,
}

impl DeletionScanGap {
    /// 把这一样记成 `invariant` 射程里判不了的一部分：这一版在射程里、在镜像上，它要的那一样读不出、读不全或配不上，报部分没判、
    /// 逐条列出没判到的是哪一样（[`Judgements::part_of_the_range_left_unjudged`]；C701（别的判法静默跳过读不出的部分）
    /// 「一律报部分没判」，不报不适用）。
    pub(super) fn note_as_a_part_of_the_range_left_unjudged(
        self,
        judgements: &mut Judgements,
        invariant: &'static str,
    ) {
        judgements.part_of_the_range_left_unjudged(invariant, self.reason().to_string());
    }

    pub(super) fn reason(self) -> &'static str {
        match self {
            DeletionScanGap::TreeTablePointerAllZero => {
                "走读到的某一版树表指针全零（core 照它读盘 0 槽 0、读不出）：那一版的删除态射程里判不了"
            }
            DeletionScanGap::TreeTableUnreadable => {
                "走读到的某一版树表读不出（两份都对不上整单元校验和或解不开）：那一版的删除态射程里判不了"
            }
            DeletionScanGap::TreeTableEntryShorterThanTheFieldTable => {
                "走读到的某一版树表条目宽不到字段表的 200 字节：那一版的删除态射程里判不了"
            }
            DeletionScanGap::DeletedInodesHeadIdentifierNamesNoSingleInodeTree => {
                "走读到的某一版 deleted_inodes 条目偏移 116 的头 ID 配不上同一张树表里恰好一条 inode 树条目：那一版判不了"
            }
            DeletionScanGap::InodeTreeNamedByNoSingleDeletedInodesEntry => {
                "走读到的某一版树表里有一棵 inode 树，同一张表里偏移 116 的头 ID 指着它的 deleted_inodes 条目不是恰好一条：那一版这棵树的队列与记录判不了"
            }
            DeletionScanGap::ExtentTreeHeadIdentifierNamesNoSingleInodeTree => {
                "走读到的某一版树表里有一棵 extent 树，偏移 116 的头 ID 配不上同一张表里恰好一条 inode 树条目：那一版的数据单元数判不了"
            }
            DeletionScanGap::DeletedInodesRootUnreadable => {
                "走读到的某一版 deleted_inodes 根读不出，或不是层级 0、key 宽与条目宽 8 的码 2 根：那一版的队列判不了"
            }
            DeletionScanGap::InodeTreeUnreadable => {
                "走读到的某一版 inode 树读不全（节点或叶容器读不出、层级或条目宽不对、类型段不认得）：那一版的记录判不了"
            }
            DeletionScanGap::ExtentTreeOfTheInodeTreeNotSingle => {
                "走读到的某一版树表里头 ID 指着这棵 inode 树的 extent 树条目不是恰好一条：那一版的数据单元数判不了"
            }
            DeletionScanGap::ExtentTreeUnreadable => {
                "走读到的某一版 extent 树读不全：那一版每个号的数据单元数判不了"
            }
            DeletionScanGap::AccountingTreeUnreadable => {
                "走读到的某一版记账树读不全（节点读不出、层级或条目宽不对）：那一版的记账行判不了"
            }
            DeletionScanGap::TombstoneTreeUnreadable => {
                "所选根的树表或墓碑树读不全：水位大于墓碑对象 ID 那一半判不了"
            }
            DeletionScanGap::TombstoneTreeOfTheVersionUnreadable => {
                "走读到的某一版墓碑树读不全（节点或容器读不出、层级或条目宽不对、类型段不认得、容器的打包记录类型或记录宽不是登记的）：那一版的墓碑判不了"
            }
            DeletionScanGap::TombstoneTreeHeadIdentifierNamesNoSingleInodeTree => {
                "走读到的某一版墓碑树条目偏移 116 的头 ID 配不上同一张树表里恰好一条 inode 树条目：那一版墓碑对 inode 记录的那几判判不了"
            }
            DeletionScanGap::InodeTreeNamedByNoSingleTombstoneTreeEntry => {
                "走读到的某一版树表里有一棵 inode 树，同一张表里偏移 116 的头 ID 指着它的墓碑树条目不是恰好一条：那一版这棵树的队列号判不了"
            }
            DeletionScanGap::LineageTreeEntryNotSingle => {
                "走读到的某一版树表里谱系树条目不是恰好一条：那一版的「在谱系上」判不了"
            }
            DeletionScanGap::LineageTreeUnreadable => {
                "走读到的某一版谱系树读不全（节点或容器读不出、层级或条目宽不对、类型段不认得、容器的打包记录类型或记录宽不是登记的）：那一版的「在谱系上」判不了"
            }
            DeletionScanGap::LineageRecordsInconsistent => {
                "走读到的某一版谱系记录不合法（有一条 T_old 不小于 T_rb，或同一个 T_rb 有两条）：那一版的父链接不上，「在谱系上」判不了"
            }
            DeletionScanGap::QueuedInodeWithoutARecord => {
                "走读到的某一版 deleted_inodes 里有一个号在同一版 inode 树里没有记录：那个号要的对象出生代没有，它的谱系上墓碑判不了（这一形由 I-9.5 判红）"
            }
            DeletionScanGap::RetirementTreeUnreadable => {
                "走读到的某一版退役树读不全（节点或容器读不出、层级或条目宽不对、类型段不认得、容器的打包记录类型或记录宽不是登记的）：那一版的退役记录判不了"
            }
            DeletionScanGap::RetirementTreeHeadIdentifierNamesNoSingleInodeTree => {
                "走读到的某一版退役树条目偏移 116 的头 ID 配不上同一张树表里恰好一条 inode 树条目：那一版退役记录对叶容器的那两判判不了"
            }
            DeletionScanGap::ParentVersionNotSingle => {
                "走读到的某一版的父版（那个 txg）在回退候选集里不止一版、树表不同：拿哪一版的叶身份集比判不了"
            }
            DeletionScanGap::LoggedOperationsTreeUnreadable => {
                "走读到的某一版 logged_ops 树读不全（节点读不出、层级、key 宽或条目宽不对）：那一版的截断条目数不全，size 之外还挂着的单元有没有账判不了"
            }
            DeletionScanGap::DataUnitWithinTheSizeUnreadable => {
                "走读到的某一版有一个 size 之内的数据单元两份都读不出或整单元校验和对不上（读不出归 I-2.1）：它的声明长度判不了"
            }
            DeletionScanGap::DataUnitBeyondTheTruncationFieldUnreadable => {
                "走读到的某一版有一个截断字段之外、size 之内的数据单元两份都读不出或整单元校验和对不上（读不出归 I-2.1）：它的诞生代号判不了"
            }
            DeletionScanGap::InodeTreeOfTheVersionNotSingle => {
                "走读到的某一版树表里 inode 树条目不是恰好一条：截断条目点名的那条记录在哪棵树里认不出"
            }
        }
    }
}

/// 删除态扫描读 I-9.5、I-4.7、I-4.6、I-4.10、I-9.38、I-9.39 六条要的那一组：树表、`deleted_inodes` 与 inode 树那几样一版里任一样判不了（树表读不出、条目窄、
/// `deleted_inodes` 与 inode 树两个方向配不上、队列或记录读不出），六条在那一版都判不了；extent 树那一侧的三种（extent 树指着这棵 inode 树的
/// 不是恰好一条、extent 树读不全、extent 树的头 ID 配不上 inode 树）只记读 extent 树的那四条（[`DELETION_SCAN_INVARIANTS_READING_THE_EXTENT_TREE`]），
/// I-9.5 不读 extent 树、照判（`research/prompts/m3-batch21-code-r1-main-verification.md` 第二节 K1 那一行）。
const DELETION_QUEUE_SCAN_INVARIANTS: [&str; 6] = [
    DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
    INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
    DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
    DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT,
    DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE,
    LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT,
];

/// 删除态扫描里读 extent 树的那五条（I-4.7、I-4.6、I-4.10、I-9.38、I-9.39）：extent 树那一侧判不了时一起记射程里判不了。
const DELETION_SCAN_INVARIANTS_READING_THE_EXTENT_TREE: [&str; 5] = [
    INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
    DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
    DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT,
    DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE,
    LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT,
];

/// 删除态扫描读过的数据单元头里要的两段：偏移 8 的声明长度（I-9.38）与诞生代号（I-9.39）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DataUnitHeaderOfTheDeletionScan {
    declared_length: u16,
    pub(super) birth_txg: u64,
}

/// 删除态扫描读过的数据单元头：(设备身份, 起点槽, 位置条目里的整单元校验和) → 头里那两段（两份都读不出记 None），
/// 同 [`PackedUnitCache`]：候选集里几版常共用同一个数据单元，按认过位置的落点记一份（[`read_through_a_cache_keyed_by_the_located_placement`]）。
type DataUnitHeaderCache = BTreeMap<(u32, u64, u32), Option<DataUnitHeaderOfTheDeletionScan>>;

/// 数据单元头里声明长度那一段的偏移（D18（块里携带什么信息） 已定项 7 字段表：码 1 偏移 8，2 字节），按字段表另写一份。
const DATA_UNIT_DECLARED_LENGTH_OFFSET: usize = 8;

/// 读一个数据单元的头、不判：数据单元（码 1）不在豁免三类里，一律按这一版的中央映射认位置（`central_mapping`，
/// [`read_located_unit_without_judging`]）。
fn data_unit_header_without_judging(
    reader: &dyn ImageReader,
    pointer: &PointerView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    cache: &mut DataUnitHeaderCache,
) -> Option<DataUnitHeaderOfTheDeletionScan> {
    let located = PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping)
        .locate(pointer, MappingKeyUnitClass::DataUnit);
    read_through_a_cache_keyed_by_the_located_placement(cache, &located, || {
        read_located_unit_without_judging(reader, &located, data_unit_bytes()).map(|unit| {
            DataUnitHeaderOfTheDeletionScan {
                declared_length: read_u16(&unit, DATA_UNIT_DECLARED_LENGTH_OFFSET),
                birth_txg: read_u64(&unit, ContentUnitClass::Data.header_offsets().birth_txg),
            }
        })
    })
}

/// 删除态扫描读过的码 3 单元：解开的容器与头里写序的实例代号（I-9.6 墓碑那一半按已发布谓词滤容器要它）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PackedUnitOfTheDeletionScan {
    pub(super) view: crate::PackedUnitView,
    pub(super) write_order_instance: u32,
}

/// 删除态扫描里读过的码 3 单元：(设备身份, 起点槽, 位置条目里的整单元校验和) → 解开的单元（解不开记 None），同 [`IndexNodeCache`]：
/// 候选集里几版常共用同一片 inode 叶容器，按认过位置的落点记一份（[`read_through_a_cache_keyed_by_the_located_placement`]），省掉重复的整单元 CRC。
pub(super) type PackedUnitCache = BTreeMap<(u32, u64, u32), Option<PackedUnitOfTheDeletionScan>>;

/// 读一个码 3 单元、不判：码 3（inode 叶容器、墓碑 / 谱系 / 退役容器）不在豁免三类里，一律按这一版的中央映射认位置
/// （`central_mapping`，[`read_located_unit_without_judging`]）。
pub(super) fn read_packed_unit_without_judging(
    reader: &dyn ImageReader,
    pointer: &PointerView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    cache: &mut PackedUnitCache,
) -> Option<PackedUnitOfTheDeletionScan> {
    if pointer.all_zero {
        return None;
    }
    let located = PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping)
        .locate(pointer, MappingKeyUnitClass::PackedRecordUnit);
    read_through_a_cache_keyed_by_the_located_placement(cache, &located, || {
        read_located_unit_without_judging(reader, &located, data_unit_bytes()).and_then(|unit| {
            packed_unit_view(&unit)
                .ok()
                .map(|view| PackedUnitOfTheDeletionScan {
                    view,
                    write_order_instance: read_u32(&unit, PACKED_UNIT_WRITE_ORDER_INSTANCE_OFFSET),
                })
        })
    })
}

/// 一版的 `deleted_inodes` 号集，只读不判（D8（核心索引结构） 已定项 16：码 2、层级 0 的一个根，key = inode 号 8 字节、无 value；
/// 队列空时树表条目的根指针置零）。
pub(super) fn deleted_inodes_queue_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    cache: &mut IndexNodeCache,
) -> Result<BTreeSet<u64>, DeletionScanGap> {
    if root_pointer.all_zero {
        return Ok(BTreeSet::new());
    }
    let root = read_index_node_without_judging(
        reader,
        root_pointer,
        PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
        cache,
    )
    .ok_or(DeletionScanGap::DeletedInodesRootUnreadable)?;
    if root.level != 0
        || root.key_width != DELETED_INODES_ENTRY_BYTES
        || root.entry_width < DELETED_INODES_ENTRY_BYTES
    {
        return Err(DeletionScanGap::DeletedInodesRootUnreadable);
    }
    Ok(root
        .entries
        .iter()
        .map(|entry| read_u64(entry, 0))
        .collect())
}

/// 一版 inode 树的全部记录（号 → nlink 与 size），只读不判：读法见 [`inode_record_bytes_without_judging`]。
pub(super) fn inode_records_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
) -> Result<BTreeMap<u64, InodeRecordOfTheDeletionScan>, DeletionScanGap> {
    Ok(inode_record_bytes_without_judging(
        reader,
        root_pointer,
        central_mapping,
        index_cache,
        packed_cache,
    )?
    .into_iter()
    .map(|(inode, record)| {
        (
            inode,
            InodeRecordOfTheDeletionScan {
                object_birth: read_u64(&record, INODE_RECORD_OBJECT_BIRTH_OFFSET),
                nlink: read_u32(&record, INODE_RECORD_NLINK_OFFSET),
                size_in_bytes: read_u64(&record, INODE_RECORD_SIZE_OFFSET),
                truncation_field_txg: read_u64(&record, INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET),
                truncation_field_size_in_bytes: read_u64(
                    &record,
                    INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET,
                ),
            },
        )
    })
    .collect())
}

/// 一版 inode 树的全部记录（号 → 记录原样），只读不判：根层级 0 是零条目根（一条记录都没有，D8（核心索引结构） 已定项 6、16 第 5 条），
/// 层级 ≥ 1 的往下走（[`collect_inode_records_below`]）。删除态扫描取其中三段，扫描重建的比对（I-1.13）整条比。
pub(super) fn inode_record_bytes_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
) -> Result<BTreeMap<u64, Vec<u8>>, DeletionScanGap> {
    let mut records = BTreeMap::new();
    if root_pointer.all_zero {
        return Ok(records);
    }
    let root = read_index_node_without_judging(
        reader,
        root_pointer,
        PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
        index_cache,
    )
    .ok_or(DeletionScanGap::InodeTreeUnreadable)?;
    if root.level == 0 {
        return if root.entries.is_empty() {
            Ok(records)
        } else {
            Err(DeletionScanGap::InodeTreeUnreadable)
        };
    }
    collect_inode_records_below(
        reader,
        &root,
        central_mapping,
        index_cache,
        packed_cache,
        &mut records,
    )?;
    Ok(records)
}

/// inode 树一个内部节点下面的记录逐条收进 `records`：类型段 2 的叶容器（码 3 打包记录类型 2，记录宽 140）读它的每条记录，
/// 类型段 0 的内部节点（层级 = 父层级 − 1，且 ≥ 1）往下走。读不出、形状不对的交回 [`DeletionScanGap::InodeTreeUnreadable`]。
/// 同一个号出现两条时后读到的那条盖掉先读到的：号的唯一性由主走读的 I-9.4 判——同一片叶里同号两条、乱序由第四句（同一片容器里
/// 记录号沿记录序严格递增 取甲），跨叶的由第二、三句
/// （沿叶序容器号严格递增、左容器最大 key < 右容器号），这一遍只读不判。递归的上界：每下一层层级减一。
fn collect_inode_records_below(
    reader: &dyn ImageReader,
    node: &crate::IndexNodeView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
    records: &mut BTreeMap<u64, Vec<u8>>,
) -> Result<(), DeletionScanGap> {
    if node.entry_width < inode_internal_entry_bytes() {
        return Err(DeletionScanGap::InodeTreeUnreadable);
    }
    let inode_record_bytes =
        registered_record_width(PACKED_TYPE_INODE).expect("类型 2 登记了记录宽 140");
    for entry in &node.entries {
        let child_pointer = parse_node_pointer(&entry[34..120]);
        match InodeInternalEntryType::of(read_u16(entry, 16)) {
            InodeInternalEntryType::LeafContainer => {
                let container = read_packed_unit_without_judging(
                    reader,
                    &child_pointer,
                    central_mapping,
                    packed_cache,
                )
                .ok_or(DeletionScanGap::InodeTreeUnreadable)?;
                if container.view.record_type != PACKED_TYPE_INODE
                    || container.view.record_width < inode_record_bytes
                {
                    return Err(DeletionScanGap::InodeTreeUnreadable);
                }
                for record in &container.view.records {
                    records.insert(read_u64(record, 0), record.clone());
                }
            }
            InodeInternalEntryType::InternalNode => {
                if child_pointer.all_zero {
                    return Err(DeletionScanGap::InodeTreeUnreadable);
                }
                let child = read_index_node_without_judging(
                    reader,
                    &child_pointer,
                    PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
                    index_cache,
                )
                .ok_or(DeletionScanGap::InodeTreeUnreadable)?;
                if node.level.checked_sub(1) != Some(child.level) || child.level == 0 {
                    return Err(DeletionScanGap::InodeTreeUnreadable);
                }
                collect_inode_records_below(
                    reader,
                    &child,
                    central_mapping,
                    index_cache,
                    packed_cache,
                    records,
                )?;
            }
            InodeInternalEntryType::Unregistered(_) => {
                return Err(DeletionScanGap::InodeTreeUnreadable);
            }
        }
    }
    Ok(())
}

/// 删除态扫描在一版 extent 树里读出的一个 inode 号的数据单元：上段叶条目声明了什么（没有上段叶条目时 `None`），
/// 与它存在的每个数据单元（字节偏移、数据指针），按偏移升序。
struct DataUnitsOfAnInodeOfTheDeletionScan {
    declared_by_the_upper_leaf_entry: Option<DataUnitsDeclaredByTheUpperLeafEntry>,
    data_pointers_by_offset: Vec<(u64, PointerView)>,
}

impl DataUnitsOfAnInodeOfTheDeletionScan {
    fn data_unit_count(&self) -> u64 {
        u64::try_from(self.data_pointers_by_offset.len()).expect("单元数装得进 u64")
    }
}

/// 一版 extent 树里每个 inode 号的数据单元，只读不判：上段叶条目内联的那一个、下段叶的每条 extent 叶记录各算一个；
/// 全零的数据指针不指单元，不算。上段叶条目的标签照记（标签 0 的号也记，单元是空的）。根指针全零的树一个单元都没有。
fn data_units_of_each_inode_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    cache: &mut IndexNodeCache,
) -> Result<BTreeMap<u64, DataUnitsOfAnInodeOfTheDeletionScan>, DeletionScanGap> {
    let mut data_units_of_each_inode: BTreeMap<u64, DataUnitsOfAnInodeOfTheDeletionScan> =
        BTreeMap::new();
    if root_pointer.all_zero {
        return Ok(data_units_of_each_inode);
    }
    let extent_tree = extent_tree_with_the_inode_of_each_data_pointer_without_judging(
        reader,
        root_pointer,
        central_mapping,
        cache,
    )
    .ok_or(DeletionScanGap::ExtentTreeUnreadable)?;
    for (inode, declaration) in &extent_tree.declarations_of_the_upper_leaf_entries {
        data_units_of_each_inode.insert(
            *inode,
            DataUnitsOfAnInodeOfTheDeletionScan {
                declared_by_the_upper_leaf_entry: Some(*declaration),
                data_pointers_by_offset: Vec::new(),
            },
        );
    }
    for data_pointer_of_an_inode in extent_tree
        .data_pointers_of_inodes
        .iter()
        .filter(|data_pointer_of_an_inode| !data_pointer_of_an_inode.pointer.all_zero)
    {
        data_units_of_each_inode
            .entry(data_pointer_of_an_inode.inode)
            .or_insert_with(|| DataUnitsOfAnInodeOfTheDeletionScan {
                declared_by_the_upper_leaf_entry: None,
                data_pointers_by_offset: Vec::new(),
            })
            .data_pointers_by_offset
            .push((
                data_pointer_of_an_inode.offset_in_bytes,
                data_pointer_of_an_inode.pointer,
            ));
    }
    for data_units in data_units_of_each_inode.values_mut() {
        data_units
            .data_pointers_by_offset
            .sort_by_key(|(offset_in_bytes, _)| *offset_in_bytes);
    }
    Ok(data_units_of_each_inode)
}

/// 一版 extent 树上段读出来的形状（I-9.40 的输入），只读不判：根的层级（根节点头里的）、每个节点的 (层级, 序号)、上段叶条目的 inode 号集合
/// （任一标签都算：叶里有它的条目，这片叶就要在）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ExtentUpperSegmentShapeRead {
    root_level: u8,
    positions: BTreeSet<(u8, u64)>,
    inodes_with_an_entry: BTreeSet<u64>,
}

/// 读一版 extent 树的上段（不进下段）：从根按位置往下读每个内部条目指的孩子，记位置与叶条目的号。读不出、层级或 key 宽对不上、
/// 条目解不开、同一个落点读到两次的交回 `None`（主走读的 I-1.1 那几条判它们）。
fn extent_upper_segment_shape_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    cache: &mut IndexNodeCache,
) -> Option<ExtentUpperSegmentShapeRead> {
    let internal_entry_bytes = usize::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110");
    let upper_leaf_entry_bytes = usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113");
    let key_width = KEY_SCHEMA_EXTENT.width();
    let mut positions: BTreeSet<(u8, u64)> = BTreeSet::new();
    let mut inodes_with_an_entry: BTreeSet<u64> = BTreeSet::new();
    let mut root_level: Option<u8> = None;
    let mut seen: BTreeSet<(u32, u64)> = BTreeSet::new();
    let mut pending: Vec<(PointerView, Option<(u8, u64)>)> = vec![(*root_pointer, None)];
    // 迭代上界：每个落点只读一次（`seen`），读到的节点数不多于镜像里的码 2 单元数；跨轮带的是待读的孩子。
    while let Some((pointer, position)) = pending.pop() {
        if pointer.all_zero
            || !seen.insert((pointer.locations[0].device, pointer.locations[0].slot))
        {
            return None;
        }
        let node = read_index_node_without_judging(
            reader,
            &pointer,
            PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
            cache,
        )?;
        let (level, index) = position.unwrap_or((node.level, 0));
        if position.is_none() {
            root_level = Some(node.level);
        }
        if node.level != level || node.key_width != key_width {
            return None;
        }
        positions.insert((level, index));
        if level == 0 {
            if node.entry_width < upper_leaf_entry_bytes {
                return None;
            }
            for entry in &node.entries {
                match extent_upper_leaf_entry_view(entry) {
                    ExtentUpperLeafEntryView::NoDataUnit { inode }
                    | ExtentUpperLeafEntryView::LowerSegmentRoot { inode, .. }
                    | ExtentUpperLeafEntryView::InlineDataUnit { inode, .. } => {
                        inodes_with_an_entry.insert(inode);
                    }
                    ExtentUpperLeafEntryView::Malformed => return None,
                }
            }
            continue;
        }
        if node.entry_width < internal_entry_bytes {
            return None;
        }
        for entry in &node.entries {
            let child = extent_upper_child_of_entry_key(level, index, &entry[..key_width])?;
            pending.push((
                parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]),
                Some(child),
            ));
        }
    }
    Some(ExtentUpperSegmentShapeRead {
        root_level: root_level?,
        positions,
        inodes_with_an_entry,
    })
}

/// 有条目的 inode 号集合定的上段规范形（I-9.40）：根层级是罩得住最大号的最低一层（罩的号数大于它），节点是每个号在每一层上罩着它的那一格；
/// 集合空时是 (0, 0) 一个。与实现的写法各写各的（D13（验证路线） 已定项 5），只共用按位置寻址的几何（`position_addressed`）。
fn canonical_extent_upper_segment_of(
    inodes_with_an_entry: &BTreeSet<u64>,
) -> (u8, BTreeSet<(u8, u64)>) {
    let Some(largest_inode) = inodes_with_an_entry.last().copied() else {
        return (0, BTreeSet::from([(0, 0)]));
    };
    let mut root_level: u8 = 0;
    // 迭代上界：罩的号数逐层乘扇出、饱和在 u64 的最大值，至多十来层就大于任何不是 u64::MAX 的号。
    while extent_upper_span_in_inodes(root_level) <= largest_inode && root_level < u8::MAX {
        root_level += 1;
    }
    let positions = inodes_with_an_entry
        .iter()
        .flat_map(|inode| {
            (0..=root_level).map(move |level| (level, inode / extent_upper_span_in_inodes(level)))
        })
        .collect();
    (root_level, positions)
}

/// 一版上段的形状离规范形差在哪（I-9.40）：根层级不是最低的那一层、或节点位置集合多出或少了几个，写成一句；是规范形交回 `None`。
fn departure_of_the_extent_upper_segment_from_its_canonical_shape(
    shape: &ExtentUpperSegmentShapeRead,
) -> Option<String> {
    let (canonical_root_level, canonical_positions) =
        canonical_extent_upper_segment_of(&shape.inodes_with_an_entry);
    let extra: Vec<&(u8, u64)> = shape.positions.difference(&canonical_positions).collect();
    let missing: Vec<&(u8, u64)> = canonical_positions.difference(&shape.positions).collect();
    if shape.root_level == canonical_root_level && extra.is_empty() && missing.is_empty() {
        return None;
    }
    Some(format!(
        "上段根层级 {}（有条目的 {} 个号定的最低一层是 {canonical_root_level}），多出的节点（层级, 序号）{extra:?}，少了的节点 {missing:?}",
        shape.root_level,
        shape.inodes_with_an_entry.len(),
    ))
}

/// I-9.40 在一版上判：这一版树表里每一棵 extent 树（种类 1，根指针不全零）读它的上段（[`extent_upper_segment_shape_without_judging`]），
/// 与有条目的号集合定的规范形比；读不全的报射程里判不了。
fn judge_extent_upper_segment_shape_of_a_version(
    reader: &dyn ImageReader,
    version_name: &str,
    tree_table_entries: &[Vec<u8>],
    central_mapping: &CentralMappingLocationsOfAVersion,
    index_cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let invariant = EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY;
    for extent_entry in tree_table_entries
        .iter()
        .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_EXTENT)
    {
        let tree = read_u64(extent_entry, 0);
        let root_pointer = parse_node_pointer(&extent_entry[14..100]);
        if root_pointer.all_zero {
            continue;
        }
        let Some(shape) = extent_upper_segment_shape_without_judging(
            reader,
            &root_pointer,
            central_mapping,
            index_cache,
        ) else {
            judgements.part_of_the_range_left_unjudged(
                invariant,
                format!("{version_name}的 extent 树 {tree} 上段读不全（读不出、层级或条目解不开、一个落点读到两次）"),
            );
            continue;
        };
        let departure = departure_of_the_extent_upper_segment_from_its_canonical_shape(&shape);
        judgements.judge(invariant, departure.is_none(), || {
            format!(
                "{version_name}的 extent 树（树 {tree}）上段不是有条目的 inode 号集合定的规范形：{}",
                departure.unwrap_or_default()
            )
        });
    }
}

/// 一版 logged_ops 树里的截断条目（种类 2），只读不判：
/// 树表里种类 12 那几条指着的根逐棵读到叶，种类码不是 2 的条目不收（它们由 I-4.4 判）。根指针全零的没有条目。
fn truncation_intents_of_a_version_without_judging(
    reader: &dyn ImageReader,
    tree_table_entries: &[Vec<u8>],
    central_mapping: &CentralMappingLocationsOfAVersion,
    cache: &mut IndexNodeCache,
) -> Result<Vec<TruncationIntentOfTheScan>, DeletionScanGap> {
    let mut logged_operations: Vec<Vec<u8>> = Vec::new();
    for logged_operations_root in tree_table_entries
        .iter()
        .filter(|entry| {
            read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_LOGGED_OPERATIONS
        })
        .map(|entry| parse_node_pointer(&entry[14..100]))
        .filter(|pointer| !pointer.all_zero)
    {
        leaf_entries_of_a_multi_level_tree_without_judging(
            reader,
            &logged_operations_root,
            KEY_SCHEMA_LOGGED_OPERATIONS,
            logged_operations_entry_bytes(),
            None,
            central_mapping,
            cache,
            &mut logged_operations,
        )
        .ok_or(DeletionScanGap::LoggedOperationsTreeUnreadable)?;
    }
    Ok(logged_operations
        .iter()
        .filter_map(|entry| match LoggedOperationOfTheScan::of_entry(entry) {
            LoggedOperationOfTheScan::Truncation(intent) => Some(intent),
            LoggedOperationOfTheScan::Compaction(_)
            | LoggedOperationOfTheScan::UnregisteredKind(_) => None,
        })
        .collect())
}

/// 一版记账树的全部行，只读不判（D8（核心索引结构） 已定项 11：多层码 2 树，叶条目 34、内部条目 = key 22 + 子指针 86）：
/// 走全了才交回，有一个节点读不出、key 宽或条目宽不对、孩子层级不是父层级减一都交回 [`DeletionScanGap::AccountingTreeUnreadable`]。
/// 递归的上界：每下一层层级减一。
fn collect_accounting_rows_without_judging(
    reader: &dyn ImageReader,
    node_pointer: &PointerView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    cache: &mut IndexNodeCache,
    rows: &mut Vec<Vec<u8>>,
) -> Result<(), DeletionScanGap> {
    let node = read_index_node_without_judging(
        reader,
        node_pointer,
        PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
        cache,
    )
    .ok_or(DeletionScanGap::AccountingTreeUnreadable)?;
    let key_width = KEY_SCHEMA_ACCOUNTING.width();
    if node.key_width != key_width {
        return Err(DeletionScanGap::AccountingTreeUnreadable);
    }
    if node.level == 0 {
        if node.entry_width < accounting_entry_bytes() {
            return Err(DeletionScanGap::AccountingTreeUnreadable);
        }
        rows.extend(node.entries.iter().cloned());
        return Ok(());
    }
    if node.entry_width < internal_entry_bytes(KEY_SCHEMA_ACCOUNTING) {
        return Err(DeletionScanGap::AccountingTreeUnreadable);
    }
    for entry in &node.entries {
        let child_pointer = parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]);
        if child_pointer.all_zero {
            return Err(DeletionScanGap::AccountingTreeUnreadable);
        }
        let child_level = read_index_node_without_judging(
            reader,
            &child_pointer,
            PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
            cache,
        )
        .ok_or(DeletionScanGap::AccountingTreeUnreadable)?
        .level;
        if node.level.checked_sub(1) != Some(child_level) {
            return Err(DeletionScanGap::AccountingTreeUnreadable);
        }
        collect_accounting_rows_without_judging(
            reader,
            &child_pointer,
            central_mapping,
            cache,
            rows,
        )?;
    }
    Ok(())
}

/// 删除态扫描：要判的每一版（回退候选集里每条根，
/// 加由记录施加出来的那几版；树表单元相同的几版内容相同，只扫一次）各自重读一遍树表与几棵树，只读、不进 `walk_failures`，
/// 判 I-3.4（[`judge_pending_deletion_occupancy_of_a_version`]）与 I-9.5、I-4.7、I-4.6（[`judge_deletion_queue_of_a_version`]）。
/// 主走读按 `visited_units` 只走一遍共享子树，这几条要的是「这一版自己的」队列、记录与单元数，所以另走一遍（与 [`references_of_root`] 同一个缘故）。
/// 树表 0 条的那一版没有这几棵树，不判；树表指针全零的那一版照 core 的根记录读者读盘 0 槽 0、读不出，报射程里判不了
pub(super) fn judge_deletion_state_of_the_versions(
    reader: &dyn ImageReader,
    versions: &[VersionToScanTheDeletionStateOf<'_>],
    index_cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let mut packed_cache = PackedUnitCache::new();
    let mut data_unit_header_cache = DataUnitHeaderCache::new();
    let mut tree_tables_scanned: BTreeSet<(u32, u64, u32)> = BTreeSet::new();
    for version in versions {
        let tree_table_pointer = parse_node_pointer(version.tree_table_pointer);
        if tree_table_pointer.all_zero {
            for invariant in DELETION_QUEUE_SCAN_INVARIANTS.into_iter().chain([
                PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
                DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
                EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY,
            ]) {
                DeletionScanGap::TreeTablePointerAllZero
                    .note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
            }
            continue;
        }
        let first_location = tree_table_pointer.locations[0];
        if !tree_tables_scanned.insert((
            first_location.device,
            first_location.slot,
            first_location.checksum,
        )) {
            continue;
        }
        // 这一版自己的中央映射：几棵树里豁免三类之外的节点、码 3 容器与数据单元按它认位置（读不出时是空表，照提示）。
        // 树表单元相同的几版只扫第一版：照抄树表的那几版（零单元发布、卸载与抬 F 的空发布）也照抄映射根。
        let central_mapping = CentralMappingLocationsOfAVersion::read_without_judging(
            reader,
            &parse_node_pointer(version.mapping_root_pointer),
            index_cache,
        );
        let gap = match read_index_node_without_judging(
            reader,
            &tree_table_pointer,
            PositionOfASecondaryRead::ThePointerItself,
            index_cache,
        ) {
            None => Some(DeletionScanGap::TreeTableUnreadable),
            Some(tree_table) if tree_table.entries.is_empty() => None,
            Some(tree_table)
                if tree_table
                    .entries
                    .iter()
                    .any(|entry| entry.len() < tree_table_entry_bytes()) =>
            {
                Some(DeletionScanGap::TreeTableEntryShorterThanTheFieldTable)
            }
            Some(tree_table) => {
                judge_pending_deletion_occupancy_of_a_version(
                    reader,
                    &version.name,
                    &tree_table.entries,
                    &central_mapping,
                    index_cache,
                    judgements,
                );
                judge_deletion_queue_of_a_version(
                    reader,
                    &version.name,
                    &tree_table.entries,
                    &central_mapping,
                    index_cache,
                    &mut packed_cache,
                    &mut data_unit_header_cache,
                    judgements,
                );
                judge_extent_upper_segment_shape_of_a_version(
                    reader,
                    &version.name,
                    &tree_table.entries,
                    &central_mapping,
                    index_cache,
                    judgements,
                );
                None
            }
        };
        if let Some(gap) = gap {
            for invariant in DELETION_QUEUE_SCAN_INVARIANTS.into_iter().chain([
                PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
                DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
                EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY,
            ]) {
                gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
            }
        }
    }
    // 一次都没判到时的理由（判到过的照判定报，`Judgements::not_applicable` 不改已经评估过的）。
    judgements.not_applicable(
        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
        "走读到的每一版树表里都没有 deleted_inodes 条目（树表 0 条）：没有队列可比",
    );
    judgements.not_applicable(
        PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
        "走读到的每一版树表里都没有记账树（树表 0 条）",
    );
    judgements.not_applicable(
        INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
        "走读到的每一版里都没有处于删除中的 inode（nlink = 0 的记录、有数据单元而没有 nlink ≥ 1 记录的号）",
    );
    judgements.not_applicable(
        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
        NO_INODE_RECORD_IN_ANY_WALKED_VERSION,
    );
    judgements.not_applicable(
        DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT,
        NO_INODE_RECORD_IN_ANY_WALKED_VERSION,
    );
    judgements.not_applicable(
        DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE,
        NO_DATA_UNIT_WITHIN_THE_SIZE_OF_A_RECORD_IN_ANY_WALKED_VERSION,
    );
    judgements.not_applicable(
        LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT,
        NO_INODE_RECORD_IN_ANY_WALKED_VERSION,
    );
    judgements.not_applicable(
        DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
        "走读到的每一版树表里都没有 deleted_inodes 条目（树表 0 条）：没有队列可判",
    );
    judgements.not_applicable(
        EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY,
        "走读到的每一版树表里都没有根指针不全零的 extent 树（树表 0 条）：没有上段可判",
    );
}

/// I-3.4（可用空间扣待删占用） 只落「记账标签 4 恒 0」：这一版树表里每一棵记账树（种类 4）走全了，
/// 要有 (4, 树, 0xFFFF_FFFF, 代) 那一行（缺行判红），每一行标签 4 的值都是 0；读不全的报射程里判不了。根指针全零的不判（没有记账树）。
fn judge_pending_deletion_occupancy_of_a_version(
    reader: &dyn ImageReader,
    version_name: &str,
    tree_table_entries: &[Vec<u8>],
    central_mapping: &CentralMappingLocationsOfAVersion,
    index_cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let invariant = PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO;
    for accounting_entry in tree_table_entries
        .iter()
        .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_ACCOUNTING)
    {
        let tree = read_u64(accounting_entry, 0);
        let root_pointer = parse_node_pointer(&accounting_entry[14..100]);
        if root_pointer.all_zero {
            continue;
        }
        let mut rows = Vec::new();
        if let Err(gap) = collect_accounting_rows_without_judging(
            reader,
            &root_pointer,
            central_mapping,
            index_cache,
            &mut rows,
        ) {
            gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
            continue;
        }
        let pending_deletion_rows: Vec<&Vec<u8>> = rows
            .iter()
            .filter(|row| read_u16(row, 0) == STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES)
            .collect();
        let has_the_pending_deletion_row = pending_deletion_rows.iter().any(|row| {
            read_u32(row, ACCOUNTING_ROW_DEVICE_OFFSET) == STATISTIC_NO_DEVICE_DIMENSION
        });
        judgements.judge(invariant, has_the_pending_deletion_row, || {
            format!(
                "{version_name}的记账树（树 {tree}）走全了，没有待删除但意图未完成占用（标签 {STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES}、设备段 {STATISTIC_NO_DEVICE_DIMENSION:#x}）那一行"
            )
        });
        for row in pending_deletion_rows {
            let value = read_u64(row, ACCOUNTING_ROW_VALUE_OFFSET);
            judgements.judge(invariant, value == 0, || {
                format!(
                    "{version_name}的记账树（树 {tree}）里待删除但意图未完成占用（标签 {STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES}、设备段 {:#x}）那一行的值是 {value}，不是 0",
                    read_u32(row, ACCOUNTING_ROW_DEVICE_OFFSET)
                )
            });
        }
    }
}

/// I-9.5（两个方向）、I-4.7（对象取丙）、I-4.6（取乙）在一版上判：这一版树表里每一条 `deleted_inodes` 条目经偏移 116 的头 ID 配同一张表里
/// 树 ID 等于它的那一条 inode 树条目，extent 树同样按头 ID 配这棵 inode 树；
/// 反方向也配：每一棵 inode 树都要恰好一条 `deleted_inodes` 条目指着它，每一棵 extent 树的头 ID 都要配上恰好一条 inode 树，
/// 不是恰好一条的记判不了。
/// I-9.5 每一版每个方向各判一次（队列与 nlink = 0 的号集合两两比，干净的一版也真比过）；I-4.7 每个处于删除中的号判一次、
/// I-4.6 每条 nlink = 0 的记录判一次（一个都没有时没有对象）。
/// I-9.24（删除队列至多一条 key）：每一条 `deleted_inodes` 条目都判，读队列在配对之前，配不配得上 inode 树都判；空队列报成立
#[allow(
    clippy::too_many_arguments,
    reason = "版本称呼、树表条目、这一版的中央映射、三份缓存与判定累加器各有名字与类型，不为凑短签名收成结构体"
)]
fn judge_deletion_queue_of_a_version(
    reader: &dyn ImageReader,
    version_name: &str,
    tree_table_entries: &[Vec<u8>],
    central_mapping: &CentralMappingLocationsOfAVersion,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
    data_unit_header_cache: &mut DataUnitHeaderCache,
    judgements: &mut Judgements,
) {
    let kind_of = |entry: &&Vec<u8>| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET);
    let head_identifier_of =
        |entry: &&Vec<u8>| read_u64(entry, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
    for deleted_inodes_entry in tree_table_entries
        .iter()
        .filter(|entry| kind_of(entry) == TREE_KIND_DELETED_INODES)
    {
        let inode_tree = head_identifier_of(&deleted_inodes_entry);
        let queue_read_back = deleted_inodes_queue_without_judging(
            reader,
            &parse_node_pointer(&deleted_inodes_entry[14..100]),
            central_mapping,
            index_cache,
        );
        match &queue_read_back {
            Ok(queue) => {
                judgements.judge(
                    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
                    queue.len() <= 1,
                    || {
                        format!(
                        "{version_name}：deleted_inodes（树 {}）里有 {} 条 key {queue:?}，多于一条",
                        read_u64(deleted_inodes_entry, 0),
                        queue.len()
                    )
                    },
                );
            }
            Err(gap) => {
                gap.note_as_a_part_of_the_range_left_unjudged(
                    judgements,
                    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
                );
            }
        }
        let inode_tree_entries: Vec<&Vec<u8>> = tree_table_entries
            .iter()
            .filter(|entry| kind_of(entry) == TREE_KIND_INODE && read_u64(entry, 0) == inode_tree)
            .collect();
        let read_back = match inode_tree_entries.as_slice() {
            [inode_tree_entry] => queue_read_back.and_then(|queue| {
                inode_records_without_judging(
                    reader,
                    &parse_node_pointer(&inode_tree_entry[14..100]),
                    central_mapping,
                    index_cache,
                    packed_cache,
                )
                .map(|records| (queue, records))
            }),
            [] | [_, _, ..] => {
                Err(DeletionScanGap::DeletedInodesHeadIdentifierNamesNoSingleInodeTree)
            }
        };
        let (queue, records) = match read_back {
            Ok(queue_and_records) => queue_and_records,
            Err(gap) => {
                for invariant in DELETION_QUEUE_SCAN_INVARIANTS {
                    gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
                }
                continue;
            }
        };
        judge_the_deletion_queue_against_the_inode_records(
            version_name,
            inode_tree,
            &queue,
            &records,
            judgements,
        );
        let extent_tree_entries: Vec<&Vec<u8>> = tree_table_entries
            .iter()
            .filter(|entry| {
                kind_of(entry) == TREE_KIND_EXTENT && head_identifier_of(entry) == inode_tree
            })
            .collect();
        let data_units_of_each_inode = match extent_tree_entries.as_slice() {
            [extent_tree_entry] => data_units_of_each_inode_without_judging(
                reader,
                &parse_node_pointer(&extent_tree_entry[14..100]),
                central_mapping,
                index_cache,
            ),
            [] | [_, _, ..] => Err(DeletionScanGap::ExtentTreeOfTheInodeTreeNotSingle),
        };
        match data_units_of_each_inode {
            Ok(data_units_of_each_inode) => {
                let data_unit_count_of_each_inode: BTreeMap<u64, u64> = data_units_of_each_inode
                    .iter()
                    .map(|(inode, data_units)| (*inode, data_units.data_unit_count()))
                    .collect();
                judge_inodes_being_deleted_against_the_queue_and_their_data_units(
                    version_name,
                    &queue,
                    &records,
                    &data_unit_count_of_each_inode,
                    judgements,
                );
                match truncation_intents_of_a_version_without_judging(
                    reader,
                    tree_table_entries,
                    central_mapping,
                    index_cache,
                ) {
                    Ok(truncation_intents) => {
                        judge_data_unit_counts_of_the_inodes_against_their_size(
                            version_name,
                            &records,
                            &data_units_of_each_inode,
                            &truncation_intents,
                            judgements,
                        );
                        judge_data_units_beyond_the_size_against_the_truncation_intents(
                            version_name,
                            &records,
                            &data_unit_count_of_each_inode,
                            &truncation_intents,
                            judgements,
                        );
                    }
                    Err(gap) => {
                        for invariant in [
                            DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
                            DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT,
                        ] {
                            gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
                        }
                    }
                }
                judge_declared_lengths_of_the_data_units_within_the_size(
                    reader,
                    version_name,
                    &records,
                    &data_units_of_each_inode,
                    central_mapping,
                    data_unit_header_cache,
                    judgements,
                );
                judge_birth_txgs_of_the_live_data_units_beyond_the_truncation_field(
                    reader,
                    version_name,
                    &records,
                    &data_units_of_each_inode,
                    central_mapping,
                    data_unit_header_cache,
                    judgements,
                );
            }
            Err(gap) => {
                for invariant in DELETION_SCAN_INVARIANTS_READING_THE_EXTENT_TREE {
                    gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
                }
            }
        }
    }
    // 反方向也配：上面那一遍只从
    // deleted_inodes 条目一侧去配，树表里没有指着某棵 inode 树的 deleted_inodes 条目时那棵树一次都不判、也不记判不了。这里每一棵 inode 树
    // 都要恰好一条 deleted_inodes 条目的头 ID 指着它，不是恰好一条的记判不了。放在上面那一遍之后：
    // deleted_inodes 头 ID 配不上 inode 树的那一形同时也是这一形，先记上面那一句更近的理由。
    for inode_tree_entry in tree_table_entries
        .iter()
        .filter(|entry| kind_of(entry) == TREE_KIND_INODE)
    {
        let inode_tree = read_u64(inode_tree_entry, 0);
        let deleted_inodes_entries_naming_the_inode_tree = tree_table_entries
            .iter()
            .filter(|entry| {
                kind_of(entry) == TREE_KIND_DELETED_INODES
                    && head_identifier_of(entry) == inode_tree
            })
            .count();
        if deleted_inodes_entries_naming_the_inode_tree != 1 {
            for invariant in DELETION_QUEUE_SCAN_INVARIANTS {
                DeletionScanGap::InodeTreeNamedByNoSingleDeletedInodesEntry
                    .note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
            }
        }
    }
    // extent 树那一侧也反方向配：
    // 上面那一遍只读头 ID 等于这棵 inode 树的 extent 树，头 ID 配不上 inode 树的 extent 树一次都不读、也不记判不了——一条空根条目
    // 冒充这棵 inode 树的 extent 树、真 extent 树头 ID 指别处时，I-4.7、I-4.6 按零个单元判、报没有对象。这里每一棵 extent 树的头 ID
    // 都要配上恰好一条 inode 树，配不上的记判不了（设计轮判决 F4-b「配不上记判不了」）。放在上面那一遍之后：没有 extent 树指着
    // inode 树的那一形同时也是这一形，先记上面那一句更近的理由。I-9.5 不读 extent 树，不记。
    for extent_tree_entry in tree_table_entries
        .iter()
        .filter(|entry| kind_of(entry) == TREE_KIND_EXTENT)
    {
        let inode_tree = head_identifier_of(&extent_tree_entry);
        let inode_trees_named_by_the_extent_tree = tree_table_entries
            .iter()
            .filter(|entry| kind_of(entry) == TREE_KIND_INODE && read_u64(entry, 0) == inode_tree)
            .count();
        if inode_trees_named_by_the_extent_tree != 1 {
            for invariant in DELETION_SCAN_INVARIANTS_READING_THE_EXTENT_TREE {
                DeletionScanGap::ExtentTreeHeadIdentifierNamesNoSingleInodeTree
                    .note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
            }
        }
    }
}

/// I-9.5 两个方向，一版各判一次：① 队列里的每个号在这棵 inode 树里有记录且 nlink = 0；② nlink = 0 的每条记录，号在队列里。
fn judge_the_deletion_queue_against_the_inode_records(
    version_name: &str,
    inode_tree: u64,
    queue: &BTreeSet<u64>,
    records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>,
    judgements: &mut Judgements,
) {
    let invariant = DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES;
    let first_queued_inode_without_a_record_of_nlink_zero = queue
        .iter()
        .find(|inode| records.get(inode).is_none_or(|record| record.nlink != 0));
    judgements.judge(
        invariant,
        first_queued_inode_without_a_record_of_nlink_zero.is_none(),
        || {
            let inode = first_queued_inode_without_a_record_of_nlink_zero
                .expect("判红时队列里有一个号没有 nlink = 0 的记录");
            match records.get(inode) {
                None => format!(
                    "{version_name}：deleted_inodes 里的号 {inode} 在同一版的 inode 树（树 {inode_tree}）里没有记录"
                ),
                Some(record) => format!(
                    "{version_name}：deleted_inodes 里的号 {inode} 在同一版的 inode 树（树 {inode_tree}）里的记录 nlink = {}，不是 0",
                    record.nlink
                ),
            }
        },
    );
    let first_record_of_nlink_zero_outside_the_queue = records
        .iter()
        .find(|(inode, record)| record.nlink == 0 && !queue.contains(inode))
        .map(|(inode, _)| *inode);
    judgements.judge(
        invariant,
        first_record_of_nlink_zero_outside_the_queue.is_none(),
        || {
            let inode = first_record_of_nlink_zero_outside_the_queue
                .expect("判红时有一条 nlink = 0 的记录不在队列里");
            format!(
                "{version_name}：inode 树（树 {inode_tree}）里 inode {inode} 的记录 nlink = 0，号不在同一版的 deleted_inodes 里"
            )
        },
    );
}

/// I-4.7，对象取丙：nlink = 0 的记录的号，加上 extent 树里有数据单元、而 inode 树里没有 nlink ≥ 1 记录的号；
/// 每个这样的号要么在队列里，要么一个数据单元都不剩。
fn judge_inodes_being_deleted_against_the_queue_and_their_data_units(
    version_name: &str,
    queue: &BTreeSet<u64>,
    records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>,
    data_units_of_each_inode: &BTreeMap<u64, u64>,
    judgements: &mut Judgements,
) {
    let mut inodes_being_deleted: BTreeSet<u64> = records
        .iter()
        .filter(|(_, record)| record.nlink == 0)
        .map(|(inode, _)| *inode)
        .collect();
    inodes_being_deleted.extend(
        data_units_of_each_inode
            .iter()
            .filter(|(inode, data_units)| {
                **data_units > 0 && !records.get(inode).is_some_and(|record| record.nlink >= 1)
            })
            .map(|(inode, _)| *inode),
    );
    for inode in inodes_being_deleted {
        let data_units_left = data_units_of_each_inode.get(&inode).copied().unwrap_or(0);
        judgements.judge(
            INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
            queue.contains(&inode) || data_units_left == 0,
            || {
                let why_being_deleted = match records.get(&inode) {
                    Some(_) => "记录 nlink = 0",
                    None => "inode 树里没有它的记录",
                };
                format!(
                    "{version_name}：inode {inode} 处于删除中（{why_being_deleted}），extent 树里还有 {data_units_left} 个数据单元，号也不在 deleted_inodes 里"
                )
            },
        );
    }
}

/// 一条记录的 size 要几个数据单元：⌈size ÷ 净荷容量⌉.max(1)（size 为 0 也是一个声明长度 0 的单元；实现那一侧同一式在
/// `crates/singlefs-core/src/write_request_split.rs` 的 `data_unit_count_of_a_sequential_write` 与 `crates/singlefs-core/src/truncate.rs`）。
/// 净荷容量 checker 自己算一份（[`data_unit_payload_capacity_in_bytes`]）。
pub(super) fn data_units_the_size_asks_for(size_in_bytes: u64) -> u64 {
    size_in_bytes
        .div_ceil(data_unit_payload_capacity_in_bytes())
        .max(1)
}

/// 这一版的截断条目里有没有一条对得上这条记录：inode 号、对象出生代相同，目标 = 记录的 size。
fn truncation_intent_names_the_record(
    inode: u64,
    record: &InodeRecordOfTheDeletionScan,
    truncation_intents: &[TruncationIntentOfTheScan],
) -> bool {
    truncation_intents.iter().any(|intent| {
        intent.inode == inode
            && intent.object_birth == record.object_birth
            && intent.target_size_in_bytes == record.size_in_bytes
    })
}

/// I-4.6（中间态其余不变量成立） 取乙之后合并 C711（单元数对 size 只判了 nlink = 0 的记录），判决
/// `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K7 T-3 与第六节 S1：每条记录（nlink = 0 与 ≥ 1 都判）各判两句——
/// ① 存在的数据单元从 0 起连号（按偏移升序第 i 个的偏移是 i × 净荷容量）；
/// ② 单元数对 size 与上段叶条目的标签：标签 1、2（之后没有洞）等于 ⌈size ÷ 净荷⌉.max(1)，有一条对得上的截断条目时 size 之外可以多；
///    标签 3、4（之后到 size 是洞）至少一个、少于它（至少一个洞单元；截断中间态 size 之外挂着的单元不算洞，这时它不会少于）；
///    标签 0 的号一个单元都没有；没有上段叶条目的号一个单元都没有、而且 size 是 0（判决
///    `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S2 与第三节第 4 句：size > 0 的号写者照写标签 0，
///    「有记录、没有条目、size > 0」读者拒、这里红）。
fn judge_data_unit_counts_of_the_inodes_against_their_size(
    version_name: &str,
    records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>,
    data_units_of_each_inode: &BTreeMap<u64, DataUnitsOfAnInodeOfTheDeletionScan>,
    truncation_intents: &[TruncationIntentOfTheScan],
    judgements: &mut Judgements,
) {
    let payload_capacity_in_bytes = data_unit_payload_capacity_in_bytes();
    let no_data_unit = DataUnitsOfAnInodeOfTheDeletionScan {
        declared_by_the_upper_leaf_entry: None,
        data_pointers_by_offset: Vec::new(),
    };
    for (inode, record) in records {
        let data_units = data_units_of_each_inode.get(inode).unwrap_or(&no_data_unit);
        let first_offset_out_of_sequence = data_units
            .data_pointers_by_offset
            .iter()
            .zip(0u64..)
            .find(|((offset_in_bytes, _), position)| {
                position.checked_mul(payload_capacity_in_bytes) != Some(*offset_in_bytes)
            })
            .map(|((offset_in_bytes, _), position)| (*offset_in_bytes, position));
        judgements.judge(
            DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
            first_offset_out_of_sequence.is_none(),
            || {
                let (offset_in_bytes, position) = first_offset_out_of_sequence
                    .expect("判红时有一个单元的偏移不是它的位次乘净荷容量");
                format!(
                    "{version_name}：inode {inode} 存在的数据单元不从 0 起连号：按偏移升序第 {position} 个的偏移是 {offset_in_bytes}，要 {position} × 净荷容量 {payload_capacity_in_bytes}"
                )
            },
        );
        let data_units_existing = data_units.data_unit_count();
        let asked_for = data_units_the_size_asks_for(record.size_in_bytes);
        let names_a_truncation_intent =
            truncation_intent_names_the_record(*inode, record, truncation_intents);
        let (count_matches_the_tag, what_the_tag_allows) = match (
            data_units.declared_by_the_upper_leaf_entry,
            names_a_truncation_intent,
        ) {
            (None, _) => (
                data_units_existing == 0 && record.size_in_bytes == 0,
                "没有上段叶条目：一个单元都没有、size 0".to_string(),
            ),
            (Some(DataUnitsDeclaredByTheUpperLeafEntry::NoDataUnit), _) => (
                data_units_existing == 0,
                "上段叶条目标签 0：一个单元都没有".to_string(),
            ),
            (
                Some(DataUnitsDeclaredByTheUpperLeafEntry::WithDataUnits(
                    TailAfterTheLastDataUnit::NoHole,
                )),
                true,
            ) => (
                data_units_existing >= asked_for,
                format!("标签 1 或 2（之后没有洞）、有对得上的截断条目：不少于 {asked_for} 个"),
            ),
            (
                Some(DataUnitsDeclaredByTheUpperLeafEntry::WithDataUnits(
                    TailAfterTheLastDataUnit::NoHole,
                )),
                false,
            ) => (
                data_units_existing == asked_for,
                format!("标签 1 或 2（之后没有洞）、没有对得上的截断条目：恰好 {asked_for} 个"),
            ),
            (
                Some(DataUnitsDeclaredByTheUpperLeafEntry::WithDataUnits(
                    TailAfterTheLastDataUnit::HoleUpToTheSize,
                )),
                _,
            ) => (
                data_units_existing >= 1 && data_units_existing < asked_for,
                format!("标签 3 或 4（之后到 size 是洞）：至少 1 个、少于 {asked_for} 个"),
            ),
        };
        judgements.judge(
            DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
            count_matches_the_tag,
            || {
                format!(
                    "{version_name}：inode {inode} 的记录 nlink = {}、size {}，按净荷容量 {payload_capacity_in_bytes} 要 {asked_for} 个数据单元，extent 树里有 {data_units_existing} 个；{what_the_tag_allows}",
                    record.nlink, record.size_in_bytes
                )
            },
        );
    }
}

/// I-4.10（截断中间态有账），T-2：每条记录，存在的数据单元多于
/// ⌈size ÷ 净荷⌉.max(1)（size 之外还挂着单元）时，同一版 logged_ops 里有一条对得上的截断条目（inode 号、对象出生代相同、目标 = 记录的 size）。
/// 只判这一个方向：截在最后一个单元之内的那次意图之后 size 之外一个单元都没有、条目照挂着（下一步是放 0 个单元的最后一批，判决 K11），
/// 那一版是合法中间态；条目对不对得上记录由 I-4.4 判。
fn judge_data_units_beyond_the_size_against_the_truncation_intents(
    version_name: &str,
    records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>,
    data_unit_count_of_each_inode: &BTreeMap<u64, u64>,
    truncation_intents: &[TruncationIntentOfTheScan],
    judgements: &mut Judgements,
) {
    for (inode, record) in records {
        let data_units_existing = data_unit_count_of_each_inode
            .get(inode)
            .copied()
            .unwrap_or(0);
        let asked_for = data_units_the_size_asks_for(record.size_in_bytes);
        judgements.judge(
            DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT,
            data_units_existing <= asked_for
                || truncation_intent_names_the_record(*inode, record, truncation_intents),
            || {
                format!(
                    "{version_name}：inode {inode} 的记录 size {} 要 {asked_for} 个数据单元，extent 树里挂着 {data_units_existing} 个（size 之外 {} 个），同一版 logged_ops 里没有一条截断条目对得上它（inode 号、对象出生代 {}、目标 = size）",
                    record.size_in_bytes,
                    data_units_existing - asked_for,
                    record.object_birth
                )
            },
        );
    }
}

/// I-9.38（数据单元声明长度对 size），T-4：每条记录，每个存在而在 size 之内的
/// 数据单元（偏移 ÷ 净荷 < ⌈size ÷ 净荷⌉.max(1)），单元头偏移 8 的声明长度 = min(size − 起点, 净荷容量)。size 之外还挂着的单元
/// （截断中间态）不判；单元读不出的报射程里判不了（读不出归 I-2.1）。
fn judge_declared_lengths_of_the_data_units_within_the_size(
    reader: &dyn ImageReader,
    version_name: &str,
    records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>,
    data_units_of_each_inode: &BTreeMap<u64, DataUnitsOfAnInodeOfTheDeletionScan>,
    central_mapping: &CentralMappingLocationsOfAVersion,
    data_unit_header_cache: &mut DataUnitHeaderCache,
    judgements: &mut Judgements,
) {
    let payload_capacity_in_bytes = data_unit_payload_capacity_in_bytes();
    for (inode, record) in records {
        let Some(data_units) = data_units_of_each_inode.get(inode) else {
            continue;
        };
        let asked_for = data_units_the_size_asks_for(record.size_in_bytes);
        for (offset_in_bytes, pointer) in data_units
            .data_pointers_by_offset
            .iter()
            .filter(|(offset_in_bytes, _)| offset_in_bytes / payload_capacity_in_bytes < asked_for)
        {
            let Some(declared_length) = data_unit_header_without_judging(
                reader,
                pointer,
                central_mapping,
                data_unit_header_cache,
            )
            .map(|header| header.declared_length) else {
                DeletionScanGap::DataUnitWithinTheSizeUnreadable
                    .note_as_a_part_of_the_range_left_unjudged(
                        judgements,
                        DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE,
                    );
                continue;
            };
            let expected = record
                .size_in_bytes
                .saturating_sub(*offset_in_bytes)
                .min(payload_capacity_in_bytes);
            judgements.judge(
                DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE,
                u64::from(declared_length) == expected,
                || {
                    format!(
                        "{version_name}：inode {inode} 偏移 {offset_in_bytes} 的数据单元声明长度 {declared_length}，记录 size {} 要 min(size − 起点, 净荷容量 {payload_capacity_in_bytes}) = {expected}",
                        record.size_in_bytes
                    )
                },
            );
        }
    }
}

/// I-9.39（截断字段之外的活单元不早于它），方向照第四节第 2 条改写：
/// 每条记录判一次（照 I-4.10 每条记录都判的写法，截断字段 T = 0 的记录没有对象、照判成立）：T ≠ 0 时，extent 树里存在、在 size 之内
/// （单元号 < ⌈size ÷ 净荷⌉.max(1)）、单元号不小于 ⌈N_w ÷ 净荷⌉.max(1) 的数据单元，单元头诞生代号都不早于 T。早于 T 的单元扫描重建按
/// 读法 Z 第四条（R3）判死、不认领，遍历方向却还读得到它。
/// 「活单元」取 extent 树里、size 之内的：截断中间态 size 之外还挂着的单元出生都早于意图那次写的 T，它们本来就等着放、扫描重建按第二条不认领，
/// 不判。
/// 单元两份都读不出的报射程里判不了、不拿它判（读不出归 I-2.1）。
fn judge_birth_txgs_of_the_live_data_units_beyond_the_truncation_field(
    reader: &dyn ImageReader,
    version_name: &str,
    records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>,
    data_units_of_each_inode: &BTreeMap<u64, DataUnitsOfAnInodeOfTheDeletionScan>,
    central_mapping: &CentralMappingLocationsOfAVersion,
    data_unit_header_cache: &mut DataUnitHeaderCache,
    judgements: &mut Judgements,
) {
    let payload_capacity_in_bytes = data_unit_payload_capacity_in_bytes();
    for (inode, record) in records {
        let data_units_the_truncation_field_keeps =
            data_units_the_size_asks_for(record.truncation_field_size_in_bytes);
        let data_units_the_size_asks_for_now = data_units_the_size_asks_for(record.size_in_bytes);
        let mut first_data_unit_born_before_the_truncation_field: Option<(u64, u64)> = None;
        if record.truncation_field_txg != 0 {
            for (offset_in_bytes, pointer) in data_units_of_each_inode
                .get(inode)
                .map_or(
                    &[][..],
                    |data_units| &data_units.data_pointers_by_offset[..],
                )
                .iter()
                .filter(|(offset_in_bytes, _)| {
                    let unit_number = offset_in_bytes / payload_capacity_in_bytes;
                    unit_number >= data_units_the_truncation_field_keeps
                        && unit_number < data_units_the_size_asks_for_now
                })
            {
                let Some(header) = data_unit_header_without_judging(
                    reader,
                    pointer,
                    central_mapping,
                    data_unit_header_cache,
                ) else {
                    DeletionScanGap::DataUnitBeyondTheTruncationFieldUnreadable
                        .note_as_a_part_of_the_range_left_unjudged(
                            judgements,
                            LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT,
                        );
                    continue;
                };
                if header.birth_txg < record.truncation_field_txg
                    && first_data_unit_born_before_the_truncation_field.is_none()
                {
                    first_data_unit_born_before_the_truncation_field =
                        Some((*offset_in_bytes, header.birth_txg));
                }
            }
        }
        judgements.judge(
            LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT,
            first_data_unit_born_before_the_truncation_field.is_none(),
            || {
                let (offset_in_bytes, birth_txg) = first_data_unit_born_before_the_truncation_field
                    .expect("判红时有一个截断字段之外、size 之内的单元诞生代号早于 T");
                format!(
                    "{version_name}：inode {inode} 的记录截断字段 T = {}、N_w = {}（留下 {data_units_the_truncation_field_keeps} 个单元），size {} 之内偏移 {offset_in_bytes} 的数据单元诞生代号 {birth_txg} 早于 T：扫描重建按读法 Z 第四条判死它，遍历方向还读得到",
                    record.truncation_field_txg,
                    record.truncation_field_size_in_bytes,
                    record.size_in_bytes
                )
            },
        );
    }
}

#[cfg(test)]
mod tests;
