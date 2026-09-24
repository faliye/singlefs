//! 根环几何（D22（单元原子性怎么合成） 已定项 2 / 已定项 16）：R = 3 区域，区域 r 的起点 = 基址 + r × P × chunk（设备内偏移），
//! 区域内槽 j 的偏移 = 区域起点 + j × 固定结构槽距；第 n 次发布写区域 `n mod R` 的槽 `(n div R) mod S`。

use singlefs_format::{
    ROOT_RING_BASE_SLOT, ROOT_RING_CHUNK_BYTES, ROOT_RING_PRIME_STEP, ROOT_RING_REGIONS,
    ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM, ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
    ROOT_RING_SLOTS_PER_REGION_MINIMUM, SLOT_BYTES,
};

use crate::address::{CheckpointTxg, DeviceOffsetInBytes};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RootRingSlot {
    pub region: u64,
    pub slot: u64,
}

/// 每区槽数 S：**系统配置字段，不是格式常量**（D22（单元原子性怎么合成） 已定项 1 的字段表逐字：
/// 「具体取多少不是格式决策——格式承诺的是『S 是系统配置字段 + 挂载时判区间』」）。
/// mkfs 把它写进系统配置槽偏移 362 那一字节，挂载时读回来；环几何的每一处都拿这个运行时的值算，
/// 不拿编译期常量算。
///
/// 带不变量（落在 `[ROOT_RING_SLOTS_PER_REGION_MINIMUM, ROOT_RING_SLOTS_PER_REGION_MAXIMUM]` 闭区间里），
/// 所以字段私有、只从 [`RootRingSlotsPerRegion::from_system_configuration_field`] 或
/// [`RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM`] 进来，也不实现 `Default`（`code-discipline.md`
/// 「带不变量的类型不实现 `Default`」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootRingSlotsPerRegion {
    slots_per_region: u64,
}

/// 盘上读来的那一字节落在格式承诺的区间之外：带上读到的值与两条边，调用方报得出拒的是什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootRingSlotsPerRegionOutOfRange {
    /// 系统配置槽偏移 362 那一字节的值，原样带出来。
    pub declared_slots_per_region: u64,
    pub minimum: u64,
    pub maximum: u64,
}

impl RootRingSlotsPerRegion {
    /// mkfs 第一版写进系统配置的 S。`const fn` 里的断言是编译期的：碰到区间的边连编译都过不去
    /// （字段表逐字「4..16 之间取值不碰任何一条边」）。
    pub const AT_MAKE_FILESYSTEM: Self = Self::at_make_filesystem_checked_at_compile_time(
        ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
    );

    const fn at_make_filesystem_checked_at_compile_time(slots_per_region: u64) -> Self {
        assert!(
            slots_per_region > ROOT_RING_SLOTS_PER_REGION_MINIMUM
                && slots_per_region < ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
            "mkfs 写的 S 碰到了区间的边（D22 已定项 1 字段表：4..16 之间取值不碰任何一条边）"
        );
        Self { slots_per_region }
    }

    /// 盘上读来的那一字节进这个类型的唯一入口：落在区间外就交不出这个类型，于是「按越界的 S 算环几何」
    /// 写不出来（`code-discipline.md`「让非法状态写不出来」）。
    ///
    /// # Errors
    /// 值小于下界或大于上界 ⇒ [`RootRingSlotsPerRegionOutOfRange`]。
    pub fn from_system_configuration_field(
        declared_slots_per_region: u64,
    ) -> Result<Self, RootRingSlotsPerRegionOutOfRange> {
        if !(ROOT_RING_SLOTS_PER_REGION_MINIMUM..=ROOT_RING_SLOTS_PER_REGION_MAXIMUM)
            .contains(&declared_slots_per_region)
        {
            return Err(RootRingSlotsPerRegionOutOfRange {
                declared_slots_per_region,
                minimum: ROOT_RING_SLOTS_PER_REGION_MINIMUM,
                maximum: ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
            });
        }
        Ok(Self {
            slots_per_region: declared_slots_per_region,
        })
    }

    /// 一个区域有几个槽。
    #[must_use]
    pub fn count(self) -> u64 {
        self.slots_per_region
    }

    /// 写进系统配置槽的那一字节。上界 16 保证它装得进 `u8`。
    #[must_use]
    pub fn to_system_configuration_field(self) -> u8 {
        u8::try_from(self.slots_per_region)
            .expect("S 装得进一字节：from_system_configuration_field 判过它不超过上界 16")
    }
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

/// 发布 checkpoint_txg 时写哪个区域的哪个槽。S 由调用方从系统配置里读来给（挂载时判过区间）。
#[must_use]
pub fn target_for_publish(
    checkpoint_txg: CheckpointTxg,
    slots_per_region: RootRingSlotsPerRegion,
) -> RootRingSlot {
    RootRingSlot {
        region: checkpoint_txg.0 % ROOT_RING_REGIONS,
        slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),
    }
}

/// 一个区域占多少字节：S 个槽 × 槽距。区域之间隔着 P × chunk，这一段之外的字节不归根环。
/// mkfs 按它清根环、`ring_end` 按它算末端，两处同一个定义。
#[must_use]
pub fn region_length_in_bytes(
    fixed_structure_slot_spacing: u32,
    slots_per_region: RootRingSlotsPerRegion,
) -> u64 {
    slots_per_region.count() * u64::from(fixed_structure_slot_spacing)
}

/// 三个区域的末端：mkfs 越界校验用（基址 + (R − 1) × P × chunk + S × 槽距 ≤ 设备可用字节）。
#[must_use]
pub fn ring_end(
    fixed_structure_slot_spacing: u32,
    slots_per_region: RootRingSlotsPerRegion,
) -> u64 {
    region_start(ROOT_RING_REGIONS - 1).0
        + region_length_in_bytes(fixed_structure_slot_spacing, slots_per_region)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slots_per_region(declared: u64) -> RootRingSlotsPerRegion {
        RootRingSlotsPerRegion::from_system_configuration_field(declared)
            .expect("这几条用例给的都是区间里的值")
    }

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
            target_for_publish(CheckpointTxg(3), RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM),
            RootRingSlot { region: 0, slot: 1 },
            "第一个事务 txg 3 = 区域 0 槽 1"
        );
        assert_eq!(
            target_for_publish(CheckpointTxg(1), RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM),
            RootRingSlot { region: 1, slot: 0 }
        );
        assert_eq!(
            target_for_publish(CheckpointTxg(2), RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM),
            RootRingSlot { region: 2, slot: 0 }
        );
        assert_eq!(
            region_length_in_bytes(4096, RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM),
            8 * 4096,
            "一个区域就是 S = 8 个槽，mkfs 清根环清的是这一段"
        );
        assert_eq!(
            ring_end(4096, RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM),
            (7 << 20) + 8 * 4096
        );
    }

    /// 环几何按**传进来的那个 S** 算，不按 mkfs 那一档算：S = 4 与 S = 16 各自的环长、落点、区域长度
    /// 都与 S = 8 不同。这一条是「运行时 S 真的在用」在纯算这一层的形态。
    #[test]
    fn ring_geometry_follows_the_slots_per_region_it_is_given() {
        assert_eq!(
            target_for_publish(CheckpointTxg(12), slots_per_region(4)),
            RootRingSlot { region: 0, slot: 0 },
            "S = 4 时环长 3 × 4 = 12，txg 12 绕回区域 0 槽 0"
        );
        assert_eq!(
            target_for_publish(
                CheckpointTxg(12),
                RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM
            ),
            RootRingSlot { region: 0, slot: 4 },
            "S = 8 时环长 24，txg 12 还没绕回去"
        );
        assert_eq!(
            target_for_publish(CheckpointTxg(12), slots_per_region(16)),
            RootRingSlot { region: 0, slot: 4 },
            "S = 16 时环长 48"
        );
        assert_eq!(
            target_for_publish(CheckpointTxg(24), slots_per_region(16)),
            RootRingSlot { region: 0, slot: 8 },
            "S = 16 时 txg 24 落槽 8——S = 8 时这一条会绕回槽 0"
        );
        assert_eq!(region_length_in_bytes(4096, slots_per_region(4)), 4 * 4096);
        assert_eq!(
            region_length_in_bytes(4096, slots_per_region(16)),
            16 * 4096
        );
        assert_eq!(
            ring_end(4096, slots_per_region(16)),
            (7 << 20) + 16 * 4096,
            "S 大一档，根环末端就远一段：mkfs 的越界校验按它判"
        );
    }

    /// 区间判（D22 已定项 1 字段表：下界 k_tol + 2 = 4，上界今天算不出来、按 16 收着）：
    /// 两条边上的值收，边外的值拒，拒的时候把盘上读到的值与两条边一起带出来。
    #[test]
    fn slots_per_region_outside_the_format_interval_is_refused_with_the_value_it_read() {
        for declared in [4u64, 8, 16] {
            assert_eq!(
                RootRingSlotsPerRegion::from_system_configuration_field(declared)
                    .expect("区间闭区间，两条边上的值收")
                    .count(),
                declared
            );
        }
        assert_eq!(
            RootRingSlotsPerRegion::from_system_configuration_field(3),
            Err(RootRingSlotsPerRegionOutOfRange {
                declared_slots_per_region: 3,
                minimum: 4,
                maximum: 16,
            }),
            "下界之下拒，并报出盘上读到的 3"
        );
        assert_eq!(
            RootRingSlotsPerRegion::from_system_configuration_field(17),
            Err(RootRingSlotsPerRegionOutOfRange {
                declared_slots_per_region: 17,
                minimum: 4,
                maximum: 16,
            }),
            "上界之上拒，并报出盘上读到的 17"
        );
        assert_eq!(
            RootRingSlotsPerRegion::from_system_configuration_field(0),
            Err(RootRingSlotsPerRegionOutOfRange {
                declared_slots_per_region: 0,
                minimum: 4,
                maximum: 16,
            }),
            "全 0 的槽也拒：S = 0 会让环长变 0、取模除零"
        );
        assert_eq!(
            RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM.to_system_configuration_field(),
            8,
            "mkfs 写进偏移 362 那一字节的就是 8"
        );
    }
}
