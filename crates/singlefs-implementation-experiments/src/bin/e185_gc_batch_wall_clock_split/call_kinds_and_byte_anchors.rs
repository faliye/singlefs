use super::*;

// ───────────────────────────── 一次调用是哪一种、第七节乙类锚点 ─────────────────────────────

/// 一次 `run_compaction` 调用（发布预算压到 1）发成了什么（登记第一节「一批」那一行的口径）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CallKind {
    /// 签发：只把意图写进 logged_ops，不搬单元；单列，不算批。
    Issue,
    /// 一批（意图还在）。
    Batch,
    /// 最后一批：同一次发布删意图。
    FinalBatch,
    /// 删意图、丢意图而一个没搬：不算批。
    IntentDeletedWithoutABatch,
    /// 一次都没发（停了）。
    NoPublish,
}

impl CallKind {
    pub(super) const fn label(self) -> &'static str {
        match self {
            CallKind::Issue => "issue",
            CallKind::Batch => "batch",
            CallKind::FinalBatch => "final_batch",
            CallKind::IntentDeletedWithoutABatch => "intent_deleted_without_a_batch",
            CallKind::NoPublish => "no_publish",
        }
    }
}

/// 按发成的那一版判：logged_ops 还有没有意图、这一次搬了几个单元（同 harness 整理流前缀的判法）。
pub(super) fn kind_of_the_call(
    publishes: usize,
    intent_remains: bool,
    units_relocated: usize,
) -> CallKind {
    match (publishes, intent_remains, units_relocated > 0) {
        (0, _, _) => CallKind::NoPublish,
        (_, true, false) => CallKind::Issue,
        (_, true, true) => CallKind::Batch,
        (_, false, true) => CallKind::FinalBatch,
        (_, false, false) => CallKind::IntentDeletedWithoutABatch,
    }
}

/// 一批 = 搬了单元的那一次（会话 `observations.batches_published` 同一口径）；签发不算。
pub(super) fn counts_as_a_batch(kind: CallKind) -> bool {
    match kind {
        CallKind::Batch | CallKind::FinalBatch => true,
        CallKind::Issue | CallKind::IntentDeletedWithoutABatch | CallKind::NoPublish => false,
    }
}

/// 两盘同槽镜像：一个单元搬一次写两份。
pub(super) const DEVICES_HOLDING_A_COPY: u64 = 2;

/// R 一块盘的字节：64 槽 × 16384（锚点 1048576）。
pub(super) const fn region_bytes_per_device() -> u64 {
    LOCAL_CLUSTER_SEGMENT_SLOTS * LOCAL_SLOT_BYTES
}

/// 签发那一步扫被选中的 R 读到的字节：m 个数据单元（f = 0.25 → 262144，f = 0.75 → 786432）。
pub(super) const fn issue_scan_bytes(data_units: u64) -> u64 {
    data_units * LOCAL_DATA_UNIT_BYTES
}

/// 一条意图的批数 ⌈m ÷ K⌉。
pub(super) const fn batches_of_one_intent(data_units: u64, batch_units: u64) -> u64 {
    data_units.div_ceil(batch_units)
}

/// 一条意图签发加各批扫描读到的字节之和：每批之前对整个 R 重扫，第 j 批扫 m − jK 个单元。
pub(super) fn intent_scan_bytes(data_units: u64, batch_units: u64) -> u64 {
    let batch_scan_units: u64 = (0..batches_of_one_intent(data_units, batch_units))
        .map(|batch_index| data_units - batch_index * batch_units)
        .sum();
    (data_units + batch_scan_units) * LOCAL_DATA_UNIT_BYTES
}

/// 第一批写到这一批目的地的搬迁副本字节（两块盘）：2 × min(K, m) × 32768。
pub(super) const fn first_batch_copy_bytes(data_units: u64, batch_units: u64) -> u64 {
    let first_batch_units = if batch_units < data_units {
        batch_units
    } else {
        data_units
    };
    DEVICES_HOLDING_A_COPY * first_batch_units * LOCAL_DATA_UNIT_BYTES
}

/// 一条意图的搬迁副本字节之和：2 × m × 32768。
pub(super) const fn intent_copy_bytes(data_units: u64) -> u64 {
    DEVICES_HOLDING_A_COPY * data_units * LOCAL_DATA_UNIT_BYTES
}

/// 这一批扫描之前现行那一版 R 那块盘上未释放记录的 Σ 跨度 × 16384（停机条款第 2 条的右边）。
pub(super) fn expected_scan_read_bytes(
    records: &[AllocationRecord],
    region: CompactionRegion,
) -> u64 {
    records
        .iter()
        .filter(|record| {
            record.device == region.device()
                && !record.is_released
                && region.contains_slot(record.slot)
        })
        .map(|record| u64::from(record.span_slots) * LOCAL_SLOT_BYTES)
        .sum()
}

/// Q4 的点对 (K, 2K)：只取 2K ≤ m 的（登记第八节：f = 0.25 时 8→16 两边第一批都是 8 个单元，是取样点不敏感的一对）。
pub(super) fn doubling_pairs_within(data_units: u64) -> Vec<(u64, u64)> {
    BATCH_SIZES
        .iter()
        .flat_map(|smaller| {
            BATCH_SIZES
                .iter()
                .filter(move |larger| **larger == 2 * *smaller)
                .map(move |larger| (*smaller, *larger))
        })
        .filter(|(_smaller, larger)| *larger <= data_units)
        .collect()
}

/// 一批写到目的地的字节：每次 `write_at` 与这一批每个目的单元（盘、起点偏移、长度）重叠的字节之和；另交回每个目的单元是否被写满。
pub(super) fn relocation_copy_bytes(
    writes: &[RecordedWrite],
    destinations: &[(DeviceIdentity, u64, u64)],
) -> (u64, bool) {
    let mut copied = 0_u64;
    let mut is_every_destination_written_whole = true;
    for (device, start, length) in destinations {
        let end = start + length;
        let covered: u64 = writes
            .iter()
            .filter(|write| write.device == *device)
            .map(|write| {
                let overlap_start = write.offset.max(*start);
                let overlap_end = (write.offset + write.length).min(end);
                overlap_end.saturating_sub(overlap_start)
            })
            .sum();
        copied += covered;
        if covered != *length {
            is_every_destination_written_whole = false;
        }
    }
    (copied, is_every_destination_written_whole)
}

#[cfg(test)]
mod tests;
