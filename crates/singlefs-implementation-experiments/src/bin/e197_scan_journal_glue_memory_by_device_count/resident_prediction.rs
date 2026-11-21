//! 登记式子推的常驻量 R：环几何、`Vec` 的容量、结构大小与 glibc 的块大小，扫环峰值、读缓存与最后一盘收尾的预测。
use super::*;

/// A4：环 ≤ 设备容量 ÷ 4（D23 已定项 19 ③），且环末端不越过设备末尾，`scan_journal` 里 `ring_bytes_inside_the_device` 才取满环长。
pub(super) fn device_holds_the_ring(device_bytes: u64, ring_bytes: u64) -> bool {
    ring_bytes <= device_bytes / 4
        && JOURNAL_RING_START_SLOT * SLOT_BYTES + ring_bytes <= device_bytes
}

/// A5：`Vec::new()` 加逐条 push 到 `count` 条时的容量（元素不大于 1024 字节时最小 4，每次翻倍）。
pub(super) fn vec_capacity_after_pushes(count: u64) -> u64 {
    let mut capacity = 4;
    while capacity < count {
        capacity *= 2;
    }
    capacity
}

pub(super) fn structure_sizes() -> StructureSizes {
    let size = |bytes: usize| u64::try_from(bytes).expect("结构大小装得进 u64");
    StructureSizes {
        location_entry: size(std::mem::size_of::<LocationEntry>()),
        named_unit: size(std::mem::size_of::<NamedUnit>()),
        node_pointer: size(std::mem::size_of::<NodePointer>()),
        journal_record: size(std::mem::size_of::<JournalRecord>()),
    }
}

/// glibc 一块的大小：请求字节加 8 字节块头，向上取到 16 的倍数，最小 32（登记修订三第四节式子；推的，没量）。
#[cfg(test)]
pub(super) fn glibc_chunk_bytes(request: u64) -> u64 {
    (request + 8).next_multiple_of(16).max(32)
}

/// 登记修订三第七节 A7：每格 H 的常驻页推算 R（`model/resident_prediction.py` 的式子，按量到的 `size_of` 代入）。
/// 记录 Vec、键表过了 glibc 的 mmap 门槛之后按 `mremap` 长，没写过的容量页不常驻，所以只算 len × 元素大小，不算 2 的幂容量；
/// 每条记录的点名项堆算一块 chunk(K × 点名项大小)；两张 B 树按每叶 6 个键摊（叶 = 16 + 11 × 键 + 11 × 值，各算一块 chunk）。
/// baseline 与 split_pc = N·s·(记录 + 点名项块 + 16) + 树；baseline_pc 再多一份首份的点名项堆 s·块；
/// split 在 1 盘同 baseline，2 盘起 = 树 + 2·s·块 + s·记录 + N·s·16（取最后一盘那一刻：首份的点名项堆、这一盘的点名项堆与 Vec、到此为止的键表、两张树）。
#[cfg(test)]
fn predicted_resident_peak_bytes(
    arm: Arm,
    devices: u64,
    filled_slots: u64,
    named_per_record: u64,
    sizes: StructureSizes,
) -> f64 {
    let record = sizes.journal_record;
    let named_heap = glibc_chunk_bytes(named_per_record * sizes.named_unit);
    let leaf_of_records = glibc_chunk_bytes(16 + 11 * 16 + 11 * record);
    let leaf_of_positions = glibc_chunk_bytes(16 + 11 * 16 + 11 * 8);
    let trees = (filled_slots * (leaf_of_records + leaf_of_positions)) as f64
        / ASCENDING_INSERT_KEYS_PER_TREE_LEAF;
    let base = |device_count: u64| {
        (device_count * filled_slots * (record + named_heap + 16)) as f64 + trees
    };
    match arm {
        Arm::Baseline | Arm::SplitPc => base(devices),
        Arm::BaselinePc => base(devices) + (filled_slots * named_heap) as f64,
        Arm::Split => {
            if devices == 1 {
                base(1)
            } else {
                trees
                    + (2 * filled_slots * named_heap) as f64
                    + (filled_slots * record) as f64
                    + (devices * filled_slots * 16) as f64
            }
        }
        // 经读缓存的两臂调的是不改的 `scan_journal`：胶水那一截同基线；缓存那一截由 `predicted_cache_bytes` 另算（登记修订三第七节 A11）。
        Arm::ReadStageCache | Arm::ReadStageCacheKeepsZerosPc => base(devices),
        Arm::Empty | Arm::MeasurementPc => 0.0,
    }
}

/// 登记修订三第七节 A11、A12：读缓存每项 = chunk(4096) 字节的值 + 树叶的摊分（叶 = 16 + 11 × 24 + 11 × 24 = 544 字节的请求，一块 560，升序插入每叶留 6 个键）。
#[cfg(test)]
pub(super) fn predicted_cache_bytes(entries: u64) -> f64 {
    let leaf_of_the_cache = glibc_chunk_bytes(16 + 24 * 11 + 24 * 11);
    entries as f64
        * (glibc_chunk_bytes(JOURNAL_RECORD_BYTES) as f64
            + leaf_of_the_cache as f64 / ASCENDING_INSERT_KEYS_PER_TREE_LEAF)
}

/// 登记修订三第八节：最后一块盘最后一槽被读到那一刻的常驻页（轨迹的最后一个值）= 读到的记录数 × (记录 + 点名项块) + 环槽数 × 8（这一盘列出的偏移）；
/// 读到的记录数 = 盘数 × 每盘填满槽数，满槽时减去最后一槽（读到它的那一刻这一槽还没解）。
#[cfg(test)]
fn predicted_resident_at_the_last_device_end_bytes(
    devices: u64,
    filled_slots: u64,
    fill: Fill,
    ring_slots: u64,
    named_per_record: u64,
    sizes: StructureSizes,
) -> f64 {
    let read = devices * filled_slots - u64::from(fill == Fill::Full);
    let per_record = sizes.journal_record + glibc_chunk_bytes(named_per_record * sizes.named_unit);
    (read * per_record + ring_slots * 8) as f64
}

#[cfg(test)]
mod tests;
