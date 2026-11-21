//! logged_ops 里的截断意图与整理意图（I-4.4（意图只描述目标态）、I-5.6（活整理意图的区域里没有签发之后分配的记录）），
//! 与 I-3.16（碎片度与全空段统计对得上）。

use super::*;

/// logged_ops 条目 44 里 key 之后、整理意图（种类 1）的 value 那几段相对 value 起点的偏移
/// key 是种类 2 | 签发 txg 8。
const LOGGED_OPERATION_ISSUED_TXG_OFFSET: usize = 2;
const COMPACTION_INTENT_REGION_DEVICE_OFFSET_IN_THE_VALUE: usize = 0;
const COMPACTION_INTENT_REGION_START_SLOT_OFFSET_IN_THE_VALUE: usize = 4;
const COMPACTION_INTENT_REGION_SLOT_COUNT_OFFSET_IN_THE_VALUE: usize = 10;
const COMPACTION_INTENT_ALLOCATION_GENERATION_AT_ISSUE_OFFSET_IN_THE_VALUE: usize = 12;
/// `segs0`（签发那次发布写下的记账第 11 项里 R 所在盘的值）之后那 6 字节补零，到 value 末尾。
const COMPACTION_INTENT_PADDING_OFFSET_IN_THE_VALUE: usize = 28;
/// logged_ops 条目的种类码：1 = 整理；2 = 截断；
/// 别的码保留，不认。
const LOGGED_OPERATION_KIND_COMPACTION: u16 = 1;
const LOGGED_OPERATION_KIND_TRUNCATION: u16 = 2;
/// 截断意图（种类 2）的 value 那几段相对 value 起点的偏移。
const TRUNCATION_INTENT_INODE_OFFSET_IN_THE_VALUE: usize = 0;
const TRUNCATION_INTENT_OBJECT_BIRTH_OFFSET_IN_THE_VALUE: usize = 8;
const TRUNCATION_INTENT_TARGET_SIZE_OFFSET_IN_THE_VALUE: usize = 16;
/// 目标 size 之后那 10 字节补零，到 value 末尾。
const TRUNCATION_INTENT_PADDING_OFFSET_IN_THE_VALUE: usize = 24;

fn logged_operations_key_bytes() -> usize {
    usize::try_from(LOGGED_OPERATIONS_KEY_BYTES).expect("10")
}

/// logged_ops 一条条目按字段表解出来。种类码是盘上读来的两字节，语义上允许未知值：做成显式成员。
pub(super) enum LoggedOperationOfTheScan {
    Compaction(CompactionIntentOfTheScan),
    Truncation(TruncationIntentOfTheScan),
    UnregisteredKind(u16),
}

/// 一条截断意图的字段：目标是值、不是增量（D8（核心索引结构） 已定项 5 硬要求 1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TruncationIntentOfTheScan {
    issued_txg: u64,
    pub(super) inode: u64,
    pub(super) object_birth: u64,
    pub(super) target_size_in_bytes: u64,
    padding_is_zero: bool,
}

/// 一条整理意图的字段（`segs0` 只作观测量，I-4.4 只判它在白名单里、不判它的值，这里不读）。
pub(super) struct CompactionIntentOfTheScan {
    issued_txg: u64,
    region_device: u32,
    region_start_slot: u64,
    region_slot_count: u16,
    allocation_generation_at_issue: u64,
    padding_is_zero: bool,
}

impl LoggedOperationOfTheScan {
    /// 调用方保证条目不窄于 44（`leaf_entries_of_a_multi_level_tree_without_judging` 按叶条目宽守过）。
    pub(super) fn of_entry(entry: &[u8]) -> Self {
        let kind_code = read_u16(entry, 0);
        let value = &entry[logged_operations_key_bytes()..logged_operations_entry_bytes()];
        match kind_code {
            LOGGED_OPERATION_KIND_COMPACTION => {}
            LOGGED_OPERATION_KIND_TRUNCATION => {
                return LoggedOperationOfTheScan::Truncation(TruncationIntentOfTheScan {
                    issued_txg: read_u64(entry, LOGGED_OPERATION_ISSUED_TXG_OFFSET),
                    inode: read_u64(value, TRUNCATION_INTENT_INODE_OFFSET_IN_THE_VALUE),
                    object_birth: read_u64(
                        value,
                        TRUNCATION_INTENT_OBJECT_BIRTH_OFFSET_IN_THE_VALUE,
                    ),
                    target_size_in_bytes: read_u64(
                        value,
                        TRUNCATION_INTENT_TARGET_SIZE_OFFSET_IN_THE_VALUE,
                    ),
                    padding_is_zero: value[TRUNCATION_INTENT_PADDING_OFFSET_IN_THE_VALUE..]
                        .iter()
                        .all(|byte| *byte == 0),
                });
            }
            unregistered_kind_code => {
                return LoggedOperationOfTheScan::UnregisteredKind(unregistered_kind_code);
            }
        }
        LoggedOperationOfTheScan::Compaction(CompactionIntentOfTheScan {
            issued_txg: read_u64(entry, LOGGED_OPERATION_ISSUED_TXG_OFFSET),
            region_device: read_u32(value, COMPACTION_INTENT_REGION_DEVICE_OFFSET_IN_THE_VALUE),
            region_start_slot: read_six_byte_unsigned(
                value,
                COMPACTION_INTENT_REGION_START_SLOT_OFFSET_IN_THE_VALUE,
            ),
            region_slot_count: read_u16(
                value,
                COMPACTION_INTENT_REGION_SLOT_COUNT_OFFSET_IN_THE_VALUE,
            ),
            allocation_generation_at_issue: read_u64(
                value,
                COMPACTION_INTENT_ALLOCATION_GENERATION_AT_ISSUE_OFFSET_IN_THE_VALUE,
            ),
            padding_is_zero: value[COMPACTION_INTENT_PADDING_OFFSET_IN_THE_VALUE..]
                .iter()
                .all(|byte| *byte == 0),
        })
    }
}

/// 一版树表里那一棵 inode 树（种类 2，第一版一版只有一棵）的全部记录，只读不判：不是恰好一条 inode 树条目、读不全的交回缺口。
pub(super) fn inode_records_of_the_single_inode_tree_without_judging(
    reader: &dyn ImageReader,
    tree_table_entries: &[Vec<u8>],
    central_mapping: &CentralMappingLocationsOfAVersion,
    cache: &mut IndexNodeCache,
) -> Result<BTreeMap<u64, InodeRecordOfTheDeletionScan>, DeletionScanGap> {
    let inode_tree_roots: Vec<PointerView> = tree_table_entries
        .iter()
        .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_INODE)
        .map(|entry| parse_node_pointer(&entry[14..100]))
        .collect();
    match inode_tree_roots.as_slice() {
        [inode_tree_root] => inode_records_without_judging(
            reader,
            inode_tree_root,
            central_mapping,
            cache,
            &mut PackedUnitCache::new(),
        ),
        [] | [_, _, ..] => Err(DeletionScanGap::InodeTreeOfTheVersionNotSingle),
    }
}

/// I-4.4 对一条截断条目：value 只装 inode 号、对象出生代、目标 size，
/// 末尾补零 10 字节全 0（「再截 N 个」这样的增量只能落在补零里）；签发 txg 不晚于这一版的 txg；它点名的 inode 在同一版的 inode 树里有记录、
/// nlink ≥ 1、对象出生代相等、目标 = 记录的 size（意图那次发布已把记录的 size 写成目标，判决 K2「先落目标」）。记录读不出的报射程里判不了。
fn judge_truncation_intent_against_its_record(
    version_name: &str,
    version_txg: u64,
    intent: &TruncationIntentOfTheScan,
    records: &Result<BTreeMap<u64, InodeRecordOfTheDeletionScan>, DeletionScanGap>,
    judgements: &mut Judgements,
) {
    let what = format!(
        "{version_name}的截断意图（签发 txg {}、inode {}）",
        intent.issued_txg, intent.inode
    );
    judgements.judge(INTENT_DESCRIBES_ONLY_THE_TARGET_STATE, intent.padding_is_zero, || {
        format!(
            "{what}：value 末尾的补零里有非零字节——字段白名单之外的东西（例如「再截 N 个」这样的增量）"
        )
    });
    judgements.judge(
        INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
        intent.issued_txg <= version_txg,
        || {
            format!(
                "{what}：签发 txg {} 晚于这一版的 txg {version_txg}",
                intent.issued_txg
            )
        },
    );
    let records = match records {
        Ok(records) => records,
        Err(gap) => {
            gap.note_as_a_part_of_the_range_left_unjudged(
                judgements,
                INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
            );
            return;
        }
    };
    let record = records.get(&intent.inode);
    let names_the_live_record_at_its_target = record.is_some_and(|record| {
        record.nlink >= 1
            && record.object_birth == intent.object_birth
            && record.size_in_bytes == intent.target_size_in_bytes
    });
    judgements.judge(
        INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
        names_the_live_record_at_its_target,
        || match record {
            None => format!("{what}：同一版的 inode 树里没有这个号的记录"),
            Some(record) => format!(
                "{what}：同一版的记录 nlink = {}、对象出生代 {}、size {}，条目的对象出生代 {}、目标 size {}——要 nlink ≥ 1、对象出生代相等、目标 = size",
                record.nlink,
                record.object_birth,
                record.size_in_bytes,
                intent.object_birth,
                intent.target_size_in_bytes
            ),
        },
    );
}

/// I-4.4（意图只描述目标态，N4）与 I-5.6（活整理意图的区域里没有签发之后分配的记录，N3）：每一版（[`VersionWithItsRootPointers`]）
/// 读自己树表里种类 12 那一条指着的 logged_ops 树（只读不判，树本身的结构在主走读里判），每条条目：
/// I-4.4 判种类码登记过（1 = 整理、2 = 截断）；截断条目照 [`judge_truncation_intent_against_its_record`] 判、有截断条目的那一版另判至多一条；
/// 整理条目判补零全 0（白名单之外的东西，例如「再搬 N 个」这样的增量，只能落在补零里）、R 对齐（起始槽 64 对齐、槽数 = 64）
/// 且在单元区内（设备是池里的盘、[起始槽, 起始槽 + 64) 在 [单元区起始槽号, 盘末) 里）、`c0` 与签发 txg 都不晚于这一版的 txg；
/// R 合法的整理意图再按 I-5.6 判这一版分配记录里罩进 R（任一块盘上的同号槽）的未释放记录，分配代都不大于 `c0`。
/// 没有一条条目、没有一条 R 合法的整理意图时两条各报不适用并带理由。
pub(super) fn judge_compaction_intents_of_the_versions(
    reader: &dyn ImageReader,
    versions: &[VersionWithItsRootPointers<'_>],
    unit_area_start_slot: u64,
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let pool_devices = reader.devices();
    let mut logged_operations_judged = 0u64;
    let mut compaction_intents_with_a_well_formed_region = 0u64;
    for version in versions {
        let tree_table_entries = match tree_table_entries_of_a_version_without_judging(
            reader,
            version.tree_table_pointer,
            cache,
        ) {
            Ok(entries) => entries,
            Err(gap) => {
                for invariant in [
                    INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
                    COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,
                ] {
                    gap.note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
                }
                continue;
            }
        };
        // 这一版自己的中央映射：logged_ops 树与 inode 树里豁免三类之外的单元按它认位置（读不出时是空表，照提示）。
        let central_mapping = CentralMappingLocationsOfAVersion::read_without_judging(
            reader,
            &parse_node_pointer(version.mapping_root_pointer),
            cache,
        );
        let mut logged_operations: Vec<Vec<u8>> = Vec::new();
        let mut logged_operations_read_in_full = true;
        for logged_operations_root in tree_table_entries
            .iter()
            .filter(|entry| {
                read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_LOGGED_OPERATIONS
            })
            .map(|entry| parse_node_pointer(&entry[14..100]))
            .filter(|pointer| !pointer.all_zero)
        {
            if leaf_entries_of_a_multi_level_tree_without_judging(
                reader,
                &logged_operations_root,
                KEY_SCHEMA_LOGGED_OPERATIONS,
                logged_operations_entry_bytes(),
                None,
                &central_mapping,
                cache,
                &mut logged_operations,
            )
            .is_none()
            {
                logged_operations_read_in_full = false;
            }
        }
        if !logged_operations_read_in_full {
            for invariant in [
                INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
                COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,
            ] {
                VersionReadingGap::LoggedOperationsTreeUnreadable
                    .note_as_a_part_of_the_range_left_unjudged(judgements, invariant);
            }
        }
        let mut allocation_records_of_this_version: Option<
            Result<AllocationRecordsOfAVersion, VersionReadingGap>,
        > = None;
        let mut inode_records_of_this_version: Option<
            Result<BTreeMap<u64, InodeRecordOfTheDeletionScan>, DeletionScanGap>,
        > = None;
        let mut truncation_intents_of_this_version = 0u64;
        for entry in &logged_operations {
            logged_operations_judged += 1;
            let intent = match LoggedOperationOfTheScan::of_entry(entry) {
                LoggedOperationOfTheScan::UnregisteredKind(kind_code) => {
                    judgements.judge(INTENT_DESCRIBES_ONLY_THE_TARGET_STATE, false, || {
                        format!(
                            "{}：logged_ops 条目的种类码 {kind_code} 不在登记表里（登记的只有 {LOGGED_OPERATION_KIND_COMPACTION} = 整理、{LOGGED_OPERATION_KIND_TRUNCATION} = 截断）",
                            version.name
                        )
                    });
                    continue;
                }
                LoggedOperationOfTheScan::Truncation(intent) => {
                    truncation_intents_of_this_version += 1;
                    let records = inode_records_of_this_version.get_or_insert_with(|| {
                        inode_records_of_the_single_inode_tree_without_judging(
                            reader,
                            &tree_table_entries,
                            &central_mapping,
                            cache,
                        )
                    });
                    judge_truncation_intent_against_its_record(
                        &version.name,
                        version.checkpoint_txg,
                        &intent,
                        records,
                        judgements,
                    );
                    continue;
                }
                LoggedOperationOfTheScan::Compaction(intent) => intent,
            };
            let version_txg = version.checkpoint_txg;
            let what = format!(
                "{}的整理意图（签发 txg {}）",
                version.name, intent.issued_txg
            );
            judgements.judge(
                INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
                intent.padding_is_zero,
                || {
                    format!(
                        "{what}：value 末尾的补零里有非零字节——字段白名单之外的东西（例如「再搬 N 个」这样的增量）"
                    )
                },
            );
            let region_is_aligned = intent
                .region_start_slot
                .is_multiple_of(CLUSTER_SEGMENT_SLOTS)
                && u64::from(intent.region_slot_count) == CLUSTER_SEGMENT_SLOTS;
            judgements.judge(
                INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
                region_is_aligned,
                || {
                    format!(
                        "{what}：R 不对齐（起始槽 {}、槽数 {}；要求起始槽 64 对齐、槽数 = 64）",
                        intent.region_start_slot, intent.region_slot_count
                    )
                },
            );
            let device_end_slot = pool_devices
                .contains(&intent.region_device)
                .then(|| reader.device_bytes(intent.region_device))
                .flatten()
                .map(|device_bytes| device_bytes / SLOT_BYTES);
            let region_lies_in_the_unit_area = device_end_slot.is_some_and(|end_slot| {
                intent.region_start_slot >= unit_area_start_slot
                    && intent
                        .region_start_slot
                        .checked_add(u64::from(intent.region_slot_count))
                        .is_some_and(|region_end| region_end <= end_slot)
            });
            judgements.judge(
                INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
                region_lies_in_the_unit_area,
                || {
                    format!(
                        "{what}：R（盘 {}、槽 {} 起 {} 槽）不在单元区里（单元区起始槽号 {unit_area_start_slot}、那块盘的槽数 {device_end_slot:?}）",
                        intent.region_device, intent.region_start_slot, intent.region_slot_count
                    )
                },
            );
            judgements.judge(
                INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
                intent.allocation_generation_at_issue <= version_txg
                    && intent.issued_txg <= version_txg,
                || {
                    format!(
                        "{what}：c0 {} 或签发 txg {} 晚于这一版的 txg {version_txg}",
                        intent.allocation_generation_at_issue, intent.issued_txg
                    )
                },
            );
            if !(region_is_aligned && region_lies_in_the_unit_area) {
                continue;
            }
            let records = allocation_records_of_this_version.get_or_insert_with(|| {
                allocation_records_of_a_version_without_judging(
                    reader,
                    version,
                    &tree_table_entries,
                    cache,
                )
            });
            let records = match records {
                Ok(account) => &account.records,
                Err(gap) => {
                    gap.note_as_a_part_of_the_range_left_unjudged(
                        judgements,
                        COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,
                    );
                    continue;
                }
            };
            compaction_intents_with_a_well_formed_region += 1;
            let region_end_slot = intent.region_start_slot + CLUSTER_SEGMENT_SLOTS;
            let records_reaching_into_the_region = records.iter().filter(|record| {
                !record.is_released
                    && record.slot < region_end_slot
                    && record.slot + record.span_slots > intent.region_start_slot
            });
            for record in records_reaching_into_the_region {
                judgements.judge(
                    COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,
                    record.generation <= intent.allocation_generation_at_issue,
                    || {
                        format!(
                            "{what}：R（槽 {} 起 64 槽）里盘 {} 槽 {} 的未释放记录分配代 {} 大于 c0 {}——R 排他被破",
                            intent.region_start_slot,
                            record.device,
                            record.slot,
                            record.generation,
                            intent.allocation_generation_at_issue
                        )
                    },
                );
            }
            // R 里一条未释放记录都没有（整理已把它搬空、删意图那一刻之前）也是判过：R 里没有分配代大于 c0 的记录。
            judgements.judge(
                COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,
                true,
                String::new,
            );
        }
        // 全池至多一条截断条目：只在有截断条目的那一版判，没有的版本 I-4.4 照旧没有对象。
        if truncation_intents_of_this_version > 0 {
            judgements.judge(
                INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
                truncation_intents_of_this_version <= 1,
                || {
                    format!(
                        "{}：logged_ops 里有 {truncation_intents_of_this_version} 条截断条目，全池至多一条",
                        version.name
                    )
                },
            );
        }
    }
    if logged_operations_judged == 0 {
        judgements.not_applicable(
            INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
            NO_INTENT_ENTRY_IN_THE_LOGGED_OPERATIONS_OF_ANY_WALKED_VERSION,
        );
    }
    if compaction_intents_with_a_well_formed_region == 0 {
        judgements.not_applicable(
            COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,
            NO_COMPACTION_INTENT_WITH_A_WELL_FORMED_REGION_IN_ANY_WALKED_VERSION,
        );
    }
}

/// [单元区起始槽号, 盘末) 里空闲槽的连续段数（D5（快照 / 空间记账机制） 已定项 4 第 10 项的口径），占着的槽按升序给（`occupied_slots`，
/// 都在这一段里）：段与段之间隔着占着的槽。全扫重算，不与分配器的增量维护共享代码（C413）。
fn free_runs_between_the_occupied_slots(
    unit_area_start_slot: u64,
    device_end_slot: u64,
    occupied_slots: &[u64],
) -> u64 {
    let mut runs = 0u64;
    let mut next_free_candidate = unit_area_start_slot;
    for occupied_slot in occupied_slots {
        if *occupied_slot > next_free_candidate {
            runs += 1;
        }
        next_free_candidate = occupied_slot + 1;
    }
    if device_end_slot > next_free_candidate {
        runs += 1;
    }
    runs
}

/// I-3.16（碎片度与全空段统计对得上），N5 = C413（碎片度与全空段统计没有不变量）：最新根记账里每块盘的 runs（第 10 项）与全空聚簇段数（第 11 项）
/// 等于全扫重算的值。占着的槽 `occupied_slots` 是 I-3.1（已分配统计对得上） 改成 RCW 之前那一份并集（全部走过的版本的引用、隔离豁免、
/// F 之下仍在 defer 的、钉带出来的），那块盘上它的槽数乘 16 KiB 不等于记账的已分配时占着的是哪几槽认不出，这块盘报部分没判。
/// 记账里缺 runs 或全空聚簇段数那一行判红（没有别的不变量判它，C701）。
/// runs：单元区里空闲槽的连续段数（[`free_runs_between_the_occupied_slots`]）。全空聚簇段数照 D3（空间分配） 已定项 10 ① 按记录口径——
/// 单元区里的整段、段内 64 槽都没有最新根未释放的分配记录（`newest_root_records`），defer 窗口里的已释放槽不算占用
/// （C372（全空聚簇段数两份定义相反） 按记录口径收口，分配器增量维护的也是这一个）；从记录现扫，不经占着的槽那一份并集。
pub(super) fn judge_fragmentation_and_empty_segment_statistics(
    reader: &dyn ImageReader,
    unit_area_start_slot: u64,
    accounting: &BTreeMap<(u16, u32), u64>,
    occupied_slots: &BTreeSet<(u32, u64)>,
    newest_root_records: &Result<AllocationRecordsOfAVersion, VersionReadingGap>,
    judgements: &mut Judgements,
) {
    for device in reader.devices() {
        let Some(allocated_bytes) = accounting
            .get(&(STATISTIC_ALLOCATED_BYTES, device))
            .copied()
        else {
            // 缺这一行由 I-3.1（已分配统计对得上）、I-5.2 判红（C701（别的判法静默跳过读不出的部分））。
            judgements.part_of_the_range_left_unjudged(
                FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
                format!("盘 {device}：最新根的记账里没有已分配那一行，占着哪几槽认不出"),
            );
            continue;
        };
        let occupied_on_this_device: Vec<u64> = occupied_slots
            .range((device, 0)..=(device, u64::MAX))
            .map(|(_, slot)| *slot)
            .collect();
        let occupied_slot_count = u64::try_from(occupied_on_this_device.len()).expect("槽数");
        if occupied_slot_count.checked_mul(SLOT_BYTES) != Some(allocated_bytes) {
            // 这一份是 I-3.1 改成 RCW 之前的并集，在合法状态上也可能与记账对不上（C697 那一形），不判红。
            judgements.part_of_the_range_left_unjudged(
                FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
                format!("盘 {device}：占着的槽那一份并集（{occupied_slot_count} 槽）乘 16 KiB 不等于记账的已分配 {allocated_bytes}，占着哪几槽认不出"),
            );
            continue;
        }
        let Some(device_end_slot) = reader
            .device_bytes(device)
            .map(|device_bytes| device_bytes / SLOT_BYTES)
            .filter(|end_slot| *end_slot > unit_area_start_slot)
        else {
            // 读不出容量、盘末不在单元区之后由 I-5.2 判红。
            judgements.part_of_the_range_left_unjudged(
                FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
                format!("盘 {device}：读不出容量、或盘末不在单元区起始槽号之后，没有单元区可数"),
            );
            continue;
        };
        let occupied_in_the_unit_area: Vec<u64> = occupied_on_this_device
            .into_iter()
            .filter(|slot| (unit_area_start_slot..device_end_slot).contains(slot))
            .collect();
        let runs = free_runs_between_the_occupied_slots(
            unit_area_start_slot,
            device_end_slot,
            &occupied_in_the_unit_area,
        );
        match accounting.get(&(STATISTIC_FRAGMENTATION_RUNS, device)) {
            Some(recorded_runs) => judgements.judge(
                FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
                *recorded_runs == runs,
                || {
                    format!(
                        "盘 {device}：记账的 runs {recorded_runs}，全扫重算单元区空闲槽的连续段数是 {runs}"
                    )
                },
            ),
            // 缺 runs 那一行没有别的不变量判红（配不上而别处不红就判红、不报部分没判）。
            None => judgements.judge(
                FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
                false,
                || format!("盘 {device}：最新根的记账里没有 runs 那一行（全扫重算是 {runs}）"),
            ),
        }
        let unit_area_slots = device_end_slot - unit_area_start_slot;
        let segment_of = |slot: u64| (slot - unit_area_start_slot) / CLUSTER_SEGMENT_SLOTS;
        let Ok(AllocationRecordsOfAVersion {
            records,
            allocation_record_trees_read,
            ..
        }) = newest_root_records
        else {
            judgements.part_of_the_range_left_unjudged(
                FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
                format!("盘 {device}：最新根的分配记录读不全，按记录口径的全空聚簇段数数不出"),
            );
            continue;
        };
        if *allocation_record_trees_read == 0 {
            judgements.part_of_the_range_left_unjudged(
                FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
                format!("盘 {device}：最新根那一版一棵分配记录树都没有（树表里没有种类 3 那一条、根记录也不持有，I-9.16 判它），按记录口径的全空聚簇段数数不出"),
            );
            continue;
        }
        let full_segments = unit_area_slots / CLUSTER_SEGMENT_SLOTS;
        let full_segments_holding_an_unreleased_record: BTreeSet<u64> = records
            .iter()
            .filter(|record| record.device == device && !record.is_released)
            .flat_map(|record| record.slot..record.slot.saturating_add(record.span_slots))
            .filter(|slot| (unit_area_start_slot..device_end_slot).contains(slot))
            .map(segment_of)
            .filter(|segment| *segment < full_segments)
            .collect();
        let empty_segments_by_the_records = full_segments
            - u64::try_from(full_segments_holding_an_unreleased_record.len()).expect("段数");
        match accounting.get(&(STATISTIC_EMPTY_CLUSTER_SEGMENTS, device)) {
            Some(recorded_empty_segments) => judgements.judge(
                FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
                *recorded_empty_segments == empty_segments_by_the_records,
                || {
                    format!(
                        "盘 {device}：记账的全空聚簇段数 {recorded_empty_segments}，全扫重算是 {empty_segments_by_the_records}"
                    )
                },
            ),
            // 缺全空聚簇段数那一行同 runs 那一行：没有别的不变量判红，判红。
            None => judgements.judge(
                FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
                false,
                || {
                    format!(
                        "盘 {device}：最新根的记账里没有全空聚簇段数那一行（全扫重算是 {empty_segments_by_the_records}）"
                    )
                },
            ),
        }
    }
}

#[cfg(test)]
mod tests;
