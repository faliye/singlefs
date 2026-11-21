use super::*;

// ============================================================================
// 四、全知的历史日志：每条发布过的根（按 (实例代号, txg) 认）引用的每个单元，在这条根的根槽写落下那一刻盘上的字节。
//    「被盖」就是恢复择到的根引用的某个单元，在用到的盘上的字节与这份日志里的不同（逐字节比，经读）。
// ============================================================================

pub(super) type RootKey = (u32, u64);

pub(super) fn root_key_of(root: &RootRecord) -> RootKey {
    (root.instance.0, root.checkpoint_txg.0)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct UnitLog {
    pub(super) device: u32,
    pub(super) offset_in_bytes: u64,
    pub(super) bytes: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct RootLog {
    pub(super) units: Vec<UnitLog>,
    /// 这条根的分配记录树读得出（读不出时只记根记录直接指着的几个单元）。
    pub(super) account_readable: bool,
}

/// 一条根引用的落点：它的账里没释放的分配记录（`recovery::allocation_records_under_root`），加根记录直接指着的树表、映射树根、
/// 分配记录树根与实例表链认得出的各片。账读不出时只有后一半，`account_readable` 记假。
fn placements_referenced_by_root(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> (BTreeSet<(u32, u64, u64)>, bool) {
    let mut placements = BTreeSet::new();
    let account = allocation_records_under_root(reader, root);
    let account_readable = account.is_ok();
    if let Ok(records) = account {
        for record in records.iter().filter(|record| !record.is_released) {
            placements.insert((record.device.0, record.slot.0, u64::from(record.span_slots)));
        }
    }
    let instance_table_pages = instance_table_page_pointers_as_far_as_readable(reader, root);
    for pointer in [
        &root.tree_table,
        &root.mapping_root,
        &root.allocation_record_tree_root,
    ]
    .into_iter()
    .chain(instance_table_pages.iter())
    {
        for location in &pointer.locations {
            if location.slot.0 != 0 {
                placements.insert((location.device.0, location.slot.0, 1));
            }
        }
    }
    (placements, account_readable)
}

fn unit_log_of_root(pool: &MemoryPool, root: &RootRecord) -> RootLog {
    let (placements, account_readable) = placements_referenced_by_root(pool, root);
    let units = placements
        .into_iter()
        .filter_map(|(device, slot, span)| {
            let offset_in_bytes = slot * SLOT_BYTES_LOCAL;
            let length = usize::try_from(span * SLOT_BYTES_LOCAL).expect("单元长度装得进 usize");
            PoolReader::read(
                pool,
                DeviceIdentity(device),
                DeviceOffsetInBytes(offset_in_bytes),
                length,
            )
            .map(|bytes| UnitLog {
                device,
                offset_in_bytes,
                bytes,
            })
        })
        .collect();
    RootLog {
        units,
        account_readable,
    }
}

pub(super) fn is_a_root_slot_write(operation: &RetainedOperation, fixed: &FixedGeometry) -> bool {
    matches!(
        operation.operation.kind,
        RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess
    ) && fixed.classify(&operation.operation) == StepKind::RootRecordFua
}

pub(super) fn root_written_by(operation: &RetainedOperation) -> Option<RootRecord> {
    RootRecord::parse_slot(operation.contents.as_ref()?, &FILESYSTEM_IDENTIFIER)
}

/// 在 `start` 上依次施加录制流，每遇到一次根槽写就记下这条根引用的每个单元此刻的字节。
pub(super) fn log_the_roots_of_a_stream(
    start: &MemoryPool,
    operations: &[RetainedOperation],
) -> BTreeMap<RootKey, RootLog> {
    let fixed = fixed_geometry_of_the_pool();
    let mut pool = start.clone();
    let mut log = BTreeMap::new();
    for operation in operations {
        pool.apply(std::slice::from_ref(operation));
        if !is_a_root_slot_write(operation, &fixed) {
            continue;
        }
        if let Some(root) = root_written_by(operation) {
            log.entry(root_key_of(&root))
                .or_insert_with(|| unit_log_of_root(&pool, &root));
        }
    }
    log
}
