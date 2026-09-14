//! 根环几何（D22（单元原子性怎么合成） 已定项 2 / 已定项 16）：R = 3 区域，区域 r 的起点 = 基址 + r × P × chunk（设备内偏移），
//! 区域内槽 j 的偏移 = 区域起点 + j × 固定结构槽距；第 n 次发布写区域 `n mod R` 的槽 `(n div R) mod S`。

use singlefs_format::{
    ROOT_RING_BASE_SLOT, ROOT_RING_CHUNK_BYTES, ROOT_RING_PRIME_STEP, ROOT_RING_REGIONS,
    ROOT_RING_SLOTS_PER_REGION, SLOT_BYTES,
};

use crate::address::{CheckpointTxg, DeviceOffsetInBytes};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootRingSlot {
    pub region: u64,
    pub slot: u64,
}

#[must_use]
pub fn region_start(region: u64) -> DeviceOffsetInBytes {
    DeviceOffsetInBytes(
        ROOT_RING_BASE_SLOT * SLOT_BYTES + region * ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES,
    )
}

#[must_use]
pub fn slot_offset(target: RootRingSlot, fixed_structure_slot_spacing: u32) -> DeviceOffsetInBytes {
    DeviceOffsetInBytes(
        region_start(target.region).0 + target.slot * u64::from(fixed_structure_slot_spacing),
    )
}

/// 发布 checkpoint_txg 时写哪个区域的哪个槽。
#[must_use]
pub fn target_for_publish(checkpoint_txg: CheckpointTxg) -> RootRingSlot {
    RootRingSlot {
        region: checkpoint_txg.0 % ROOT_RING_REGIONS,
        slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % ROOT_RING_SLOTS_PER_REGION,
    }
}

/// 三个区域的末端：mkfs 越界校验用（基址 + (R − 1) × P × chunk + S × 槽距 ≤ 设备可用字节）。
#[must_use]
pub fn ring_end(fixed_structure_slot_spacing: u32) -> u64 {
    region_start(ROOT_RING_REGIONS - 1).0
        + ROOT_RING_SLOTS_PER_REGION * u64::from(fixed_structure_slot_spacing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regions_sit_at_1_4_7_mebibytes_and_publishes_rotate_across_regions() {
        assert_eq!(region_start(0), DeviceOffsetInBytes(1 << 20));
        assert_eq!(region_start(1), DeviceOffsetInBytes(4 << 20));
        assert_eq!(region_start(2), DeviceOffsetInBytes(7 << 20));
        assert_eq!(
            slot_offset(RootRingSlot { region: 0, slot: 1 }, 4096),
            DeviceOffsetInBytes((1 << 20) + 4096)
        );
        assert_eq!(
            target_for_publish(CheckpointTxg(3)),
            RootRingSlot { region: 0, slot: 1 },
            "第一个事务 txg 3 = 区域 0 槽 1"
        );
        assert_eq!(
            target_for_publish(CheckpointTxg(1)),
            RootRingSlot { region: 1, slot: 0 }
        );
        assert_eq!(
            target_for_publish(CheckpointTxg(2)),
            RootRingSlot { region: 2, slot: 0 }
        );
        assert_eq!(ring_end(4096), (7 << 20) + 8 * 4096);
    }
}
