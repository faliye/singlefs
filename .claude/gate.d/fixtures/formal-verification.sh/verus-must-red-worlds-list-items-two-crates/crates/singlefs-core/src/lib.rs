//! 样本的 core：被验区域在 compaction_plan 里，别处照常
pub mod compaction_plan;

pub fn slot_limit() -> u64 {
    singlefs_format::SLOT_NUMBER_LIMIT
}
