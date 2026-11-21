//! I-3.20（空闲、已分配、不可回收之和等于容量，已承诺预留等于闭式）。

use super::*;

/// 不可回收（统计量 3）在第一版非 zoned 池上的值（D5（快照 / 空间记账机制） 已定项 4 第 3 项；D12（目标介质） 第一版只有非 zoned 布局）。
const UNRECLAIMABLE_BYTES_ON_A_POOL_OUTSIDE_ZONED: u64 = 0;

/// I-3.20 (b) 闭式要读的那一版的四个量（D3（空间分配） 已定项 2）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CommittedReservationBasis {
    /// 树表 0 条：标签 6 取 0（I-9.16 ④ 只在树表非空时要进树表的十二种各一条，`deleted_inodes` 那一条也只在树表非空时要在）。
    VersionWithAnEmptyTreeTable,
    /// 树表里有 `deleted_inodes` 条目：两棵树的内部层数（根节点码 2 头的层级减一，零根与只有根时 0）与 inode 树的两个记录数。
    VersionWithADeletedInodesTreeTableEntry {
        tombstone_tree_internal_levels: u64,
        retirement_tree_internal_levels: u64,
        live_objects: u64,
        inode_records: u64,
    },
}

/// 标签 6 的闭式（D3（空间分配） 已定项 2）：副本数 × ((2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录)；
/// 2 是一个开放容器的槽数（32768 ÷ 16384），1 是树根那一槽，41 = ⌈32768 ÷ 815⌉、35 = ⌈32768 ÷ 959⌉（每对象先取整再乘）。
/// 乘出来装不进 u64 交回 `None`（盘上读来的记录数大到这样，比的那一侧也装不下）。
fn committed_reservation_of_the_closed_form(
    basis: CommittedReservationBasis,
    replicas: u64,
) -> Option<u64> {
    match basis {
        CommittedReservationBasis::VersionWithAnEmptyTreeTable => Some(0),
        CommittedReservationBasis::VersionWithADeletedInodesTreeTableEntry {
            tombstone_tree_internal_levels,
            retirement_tree_internal_levels,
            live_objects,
            inode_records,
        } => {
            let open_container_slots = DATA_UNIT_BYTES / SLOT_BYTES;
            let root_slots = 1u64;
            let resident_slots = [
                open_container_slots,
                tombstone_tree_internal_levels,
                root_slots,
                open_container_slots,
                retirement_tree_internal_levels,
                root_slots,
            ]
            .into_iter()
            .try_fold(0u64, u64::checked_add)?;
            let tombstone_bytes_per_live_object =
                DATA_UNIT_BYTES.div_ceil(TOMBSTONE_CONTAINER_RECORDS);
            let retirement_bytes_per_inode_record =
                DATA_UNIT_BYTES.div_ceil(RETIREMENT_CONTAINER_RECORDS);
            resident_slots
                .checked_mul(SLOT_BYTES)?
                .checked_add(tombstone_bytes_per_live_object.checked_mul(live_objects)?)?
                .checked_add(retirement_bytes_per_inode_record.checked_mul(inode_records)?)?
                .checked_mul(replicas)
        }
    }
}

/// 一棵容器树（墓碑树、退役树）的内部层数：根节点码 2 头的层级减一，零根与只有根时 0。根读不出交回 `None`。
fn internal_levels_of_a_container_tree_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    central_mapping: &CentralMappingLocationsOfAVersion,
    cache: &mut IndexNodeCache,
) -> Option<u64> {
    if root_pointer.all_zero {
        return Some(0);
    }
    read_index_node_without_judging(
        reader,
        root_pointer,
        PositionOfASecondaryRead::TheCentralMappingOfTheVersion(central_mapping),
        cache,
    )
    .map(|root| u64::from(root.level).saturating_sub(1))
}

/// I-3.20 (b) 要的那一版的四个量：树表、两棵容器树的根、inode 树都读一遍（只读）。读不出的那一样交回说明；
/// 树表非空而一条种类 9（`deleted_inodes`）都没有（条目缺了，或那一条的种类码是没登记的）也交回说明：那块畸形由 I-9.16 ④ 判红，
/// 这一版该照「没有 `deleted_inodes` 条目取 0」还是照闭式算认不出
pub(super) fn committed_reservation_basis_of_a_version_without_judging(
    reader: &dyn ImageReader,
    tree_table_pointer: &[u8],
    mapping_root_pointer: &[u8],
    cache: &mut IndexNodeCache,
) -> Result<CommittedReservationBasis, String> {
    let tree_table_entries =
        tree_table_entries_of_a_version_without_judging(reader, tree_table_pointer, cache)
            .map_err(|gap| gap.reason().to_string())?;
    let kind_of = |entry: &Vec<u8>| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET);
    if !tree_table_entries
        .iter()
        .any(|entry| kind_of(entry) == TREE_KIND_DELETED_INODES)
    {
        if tree_table_entries.is_empty() {
            return Ok(CommittedReservationBasis::VersionWithAnEmptyTreeTable);
        }
        return Err(format!(
            "最新根的树表有 {} 条、一条种类 9（deleted_inodes）都没有（I-9.16 ④ 判它）：这一版的预留取 0 还是照闭式算认不出",
            tree_table_entries.len()
        ));
    }
    let central_mapping = CentralMappingLocationsOfAVersion::read_without_judging(
        reader,
        &parse_node_pointer(mapping_root_pointer),
        cache,
    );
    let mut internal_levels_of_the_tree_of_kind = |kind: u16, what: &str| {
        let roots: Vec<PointerView> = tree_table_entries
            .iter()
            .filter(|entry| kind_of(entry) == kind)
            .map(|entry| parse_node_pointer(&entry[14..100]))
            .collect();
        match roots.as_slice() {
            [] => Ok(0),
            [root] => internal_levels_of_a_container_tree_without_judging(
                reader,
                root,
                &central_mapping,
                cache,
            )
            .ok_or_else(|| format!("最新根的{what}根节点读不出：H 认不出")),
            [_, _, ..] => Err(format!(
                "最新根的树表里{what}的条目不止一条：H 认不出取哪一棵"
            )),
        }
    };
    let tombstone_tree_internal_levels =
        internal_levels_of_the_tree_of_kind(TREE_KIND_TOMBSTONE, "墓碑树")?;
    let retirement_tree_internal_levels =
        internal_levels_of_the_tree_of_kind(TREE_KIND_RETIREMENT, "退役树")?;
    let records = inode_records_of_the_single_inode_tree_without_judging(
        reader,
        &tree_table_entries,
        &central_mapping,
        cache,
    )
    .map_err(|gap| format!("最新根的 inode 树读不全（{gap:?}）：n_live、n_记录 数不全"))?;
    let live_objects = records.values().filter(|record| record.nlink != 0).count();
    Ok(
        CommittedReservationBasis::VersionWithADeletedInodesTreeTableEntry {
            tombstone_tree_internal_levels,
            retirement_tree_internal_levels,
            live_objects: u64::try_from(live_objects).expect("记录数"),
            inode_records: u64::try_from(records.len()).expect("记录数"),
        },
    )
}

/// [`FREE_ALLOCATED_AND_UNRECLAIMABLE_SUM_TO_THE_CAPACITY_AND_THE_COMMITTED_RESERVATION_MATCHES_ITS_CLOSED_FORM`]（I-3.20）(a) 那一盘：
/// 空闲 + 已分配 + 不可回收 == 单元区容量，不可回收那一行要在、而且是第一版非 zoned 池的 0。哪一行缺、加起来溢出、容量读不出都判红
/// （与 I-5.2 同一个写法）。
pub(super) fn judge_free_allocated_and_unreclaimable_of_a_device_sum_to_the_capacity(
    device: u32,
    free: Option<u64>,
    allocated: Option<u64>,
    unreclaimable: Option<u64>,
    capacity: Option<u64>,
    judgements: &mut Judgements,
) {
    let invariant = FREE_ALLOCATED_AND_UNRECLAIMABLE_SUM_TO_THE_CAPACITY_AND_THE_COMMITTED_RESERVATION_MATCHES_ITS_CLOSED_FORM;
    judgements.judge(
        invariant,
        unreclaimable == Some(UNRECLAIMABLE_BYTES_ON_A_POOL_OUTSIDE_ZONED),
        || {
            format!(
                "盘 {device}：记账的不可回收 {unreclaimable:?}，第一版非 zoned 池这一行要在、而且是 {UNRECLAIMABLE_BYTES_ON_A_POOL_OUTSIDE_ZONED}"
            )
        },
    );
    let sum = free
        .zip(allocated)
        .and_then(|(free, allocated)| free.checked_add(allocated))
        .zip(unreclaimable)
        .and_then(|(free_and_allocated, unreclaimable)| {
            free_and_allocated.checked_add(unreclaimable)
        });
    judgements.judge(invariant, sum.is_some() && sum == capacity, || {
        format!(
            "盘 {device}：空闲 {free:?} + 已分配 {allocated:?} + 不可回收 {unreclaimable:?} ≠ 单元区 {capacity:?}"
        )
    });
}

/// I-3.20 (b)：最新根记账里的标签 6 等于闭式现算的值（[`committed_reservation_of_the_closed_form`]）。行缺、闭式装不进 u64 都判红；
/// 闭式要读的树读不出、树表非空而没有 `deleted_inodes` 条目（[`committed_reservation_basis_of_a_version_without_judging`] 交回的说明）报部分没判。
pub(super) fn judge_the_committed_reservation_against_its_closed_form(
    recorded: Option<u64>,
    basis: Result<CommittedReservationBasis, String>,
    replicas: u64,
    newest_txg: u64,
    judgements: &mut Judgements,
) {
    let invariant = FREE_ALLOCATED_AND_UNRECLAIMABLE_SUM_TO_THE_CAPACITY_AND_THE_COMMITTED_RESERVATION_MATCHES_ITS_CLOSED_FORM;
    match basis {
        Ok(basis) => {
            let expected = committed_reservation_of_the_closed_form(basis, replicas);
            judgements.judge(invariant, expected.is_some() && recorded == expected, || {
                format!(
                    "最新根（txg {newest_txg}）记账的已承诺预留（标签 6）{recorded:?}，闭式按 {basis:?}、副本数 {replicas} 现算是 {expected:?}"
                )
            });
        }
        Err(why) => judgements
            .part_of_the_range_left_unjudged(invariant, format!("已承诺预留那一句没判：{why}")),
    }
}

/// 验证侧按一版根记录读取第一个文件的内容。外层 None 表示字段读不全，内层 None 表示该版没有活文件。
/// 不执行实现的恢复：只沿本模块独立解析的树表、inode、extent 与中央映射读取盘面。
#[must_use]
pub fn first_file_content_of_a_root_record(
    reader: &dyn ImageReader,
    root_record: &[u8],
) -> Option<Option<Vec<u8>>> {
    if root_record.len() < usize::try_from(singlefs_format::ROOT_RECORD_BYTES).ok()? {
        return None;
    }
    let mut index_cache = IndexNodeCache::new();
    let mut packed_cache = PackedUnitCache::new();
    let entries = tree_table_entries_of_a_version_without_judging(
        reader,
        &root_record[36..122],
        &mut index_cache,
    )
    .ok()?;
    if entries.is_empty() {
        return Some(None);
    }
    let mapping =
        central_mapping_of_a_root_record_without_judging(reader, root_record, &mut index_cache);
    let tree_of_kind = |kind| {
        let mut matching = entries.iter().filter(|entry| read_u16(entry, 10) == kind);
        let entry = matching.next()?;
        if matching.next().is_some() {
            return None;
        }
        Some(parse_node_pointer(&entry[14..100]))
    };
    let inodes = inode_record_bytes_without_judging(
        reader,
        &tree_of_kind(2)?,
        &mapping,
        &mut index_cache,
        &mut packed_cache,
    )
    .ok()?;
    let Some(inode) = inodes.get(&1) else {
        return Some(None);
    };
    if read_u32(inode, INODE_RECORD_NLINK_OFFSET) == 0 {
        return Some(None);
    }
    let size = usize::try_from(read_u64(inode, INODE_RECORD_SIZE_OFFSET)).ok()?;
    let extents = extent_tree_with_the_inode_of_each_data_pointer_without_judging(
        reader,
        &tree_of_kind(1)?,
        &mapping,
        &mut index_cache,
    )?;
    let declaration = extents.declarations_of_the_upper_leaf_entries.get(&1)?;
    let mut data_pointers_of_the_first_file: Vec<_> = extents
        .data_pointers_of_inodes
        .into_iter()
        .filter(|data_pointer| data_pointer.inode == 1)
        .collect();
    data_pointers_of_the_first_file.sort_by_key(|data_pointer| data_pointer.offset_in_bytes);
    let capacity = usize::try_from(data_unit_payload_capacity_in_bytes()).ok()?;
    let mut content = Vec::new();
    let mut expected_offset = 0u64;
    for data_pointer in data_pointers_of_the_first_file {
        if usize::try_from(data_pointer.offset_in_bytes).ok()? >= size && size != 0 {
            continue;
        }
        if data_pointer.offset_in_bytes != expected_offset {
            return None;
        }
        expected_offset = expected_offset.checked_add(u64::try_from(capacity).ok()?)?;
        let pointer = mapping.locate(
            &data_pointer.pointer,
            PositionAuthority::TheCentralMapping(MappingKeyUnitClass::DataUnit),
        );
        let bytes = read_located_unit_without_judging(
            reader,
            &pointer,
            usize::try_from(DATA_UNIT_BYTES).ok()?,
        )?;
        if crate::check_unit(&bytes).ok()? != 1 || bytes[42] != 1 {
            return None;
        }
        let start = usize::try_from(
            singlefs_format::DATA_UNIT_HEADER_BYTES + NONCE_MAC_ALGORITHM_RESERVED_BYTES,
        )
        .ok()?;
        let end = start.checked_add(usize::from(read_u16(&bytes, 8)))?;
        let payload = bytes.get(start..end)?;
        if bytes[end..].iter().any(|byte| *byte != 0) {
            return None;
        }
        let remaining = size.checked_sub(content.len())?;
        if payload.len() > remaining {
            return None;
        }
        content.extend_from_slice(payload);
    }
    if content.len() < size {
        if !matches!(
            declaration,
            DataUnitsDeclaredByTheUpperLeafEntry::NoDataUnit
                | DataUnitsDeclaredByTheUpperLeafEntry::WithDataUnits(
                    TailAfterTheLastDataUnit::HoleUpToTheSize
                )
        ) {
            return None;
        }
        content.try_reserve(size - content.len()).ok()?;
        content.resize(size, 0);
    }
    Some(Some(content))
}

#[cfg(test)]
mod tests;
