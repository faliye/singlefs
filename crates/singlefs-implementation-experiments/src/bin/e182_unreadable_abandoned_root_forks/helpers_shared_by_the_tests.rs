use super::*;
use std::sync::OnceLock;

pub(super) fn small_specification() -> HistorySpecification {
    HistorySpecification {
        family: Family::HiddenRoot(Confirmation::Unconfirmed),
        geometry: DiskGeometry::Default,
        first_file_units: 1,
        overwrites_before_the_hidden_publish: 5,
        overwrite_shape: OverwriteShape::OneUnit,
    }
}

/// 一条小历史（G默认、一个单元、5 次覆盖写、未确认档）：几条用例共用，建一次。
pub(super) fn small_history() -> &'static RecordedHistory {
    static HISTORY: OnceLock<RecordedHistory> = OnceLock::new();
    HISTORY.get_or_init(|| record_the_history(&small_specification()).expect("小历史建得起来"))
}

pub(super) fn two_device_pool_with_pattern_at(offset: u64, length: usize) -> MemoryPool {
    let mut pool = empty_pool(1 << 22);
    for device in device_identities() {
        pool.devices
            .get_mut(&device)
            .expect("池里有这块盘")
            .write(DeviceOffsetInBytes(offset), &vec![0xA5; length]);
    }
    pool
}

// ------------------------------------------------------------------------
// 覆盖写保持 s 个单元的补臂（登记修订 12.11）。
// ------------------------------------------------------------------------

/// 补臂的 specification：G默认、第一个文件 64 个单元、2 次覆盖写、未确认档（用例里只取实例 1 与一条小历史，p 取 2 省时间）。
pub(super) fn specification_of_the_shape(shape: OverwriteShape) -> HistorySpecification {
    HistorySpecification {
        first_file_units: 64,
        overwrites_before_the_hidden_publish: 2,
        overwrite_shape: shape,
        ..small_specification()
    }
}
