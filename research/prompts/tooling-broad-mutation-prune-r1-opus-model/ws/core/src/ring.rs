use crate::layout::REGION_SLOTS;

/// S2 被改文件：发布号落到环上的哪一槽（只绕一圈）。
pub fn slot_of(publish: u64) -> u64 {
    if publish >= REGION_SLOTS {
        publish - REGION_SLOTS
    } else {
        publish
    }
}
