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

/// 每区槽数 S：**系统配置字段，不是格式常量**（D22（单元原子性怎么合成） 已定项 2 参数表「每区槽数 S」那一格：
/// 格式承诺的是「S 是系统配置字段 + 挂载时判区间」，区间的两条边 4、16 才是格式常量）。
/// mkfs 把它写进系统配置槽偏移 362 那一字节，挂载时读回来；环几何的每一处都拿这个运行时的值算，
/// 不拿编译期常量算。读者另判 S × 固定结构槽距不超过 P × chunk（D22（单元原子性怎么合成） 已定项 16 第 1 句，
/// `recovery` 的 `system_configuration_values_this_reader_accepts`）：那一判要槽距，不在这个类型里。
///
/// 带不变量（落在 `[ROOT_RING_SLOTS_PER_REGION_MINIMUM, ROOT_RING_SLOTS_PER_REGION_MAXIMUM]` 闭区间里），
/// 所以字段私有、只从 [`RootRingSlotsPerRegion::from_system_configuration_field`]、
/// [`RootRingSlotsPerRegion::first_version_make_filesystem_default_for_the_slot_spacing`] 或
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

/// 第一版 mkfs 默认 S 的下限 5：小于它拒绝 mkfs（D22（单元原子性怎么合成） 已定项 16 第 3 句，用户 2026-10-22 定）。
/// 取区间下界加一：mkfs 的默认值不取两条边，下界那一条边是 [`ROOT_RING_SLOTS_PER_REGION_MINIMUM`]。
pub const FIRST_VERSION_MAKE_FILESYSTEM_SMALLEST_DEFAULT_SLOTS_PER_REGION: u64 =
    ROOT_RING_SLOTS_PER_REGION_MINIMUM + 1;

/// 第一版 mkfs 按固定结构槽距算出的默认 S 小于 5：区域间距 P × chunk 里按这个槽距放不下 5 个槽
/// （D22（单元原子性怎么合成） 已定项 16 第 3 句「小于 5 拒绝 mkfs」）。带上算式两边的数，调用方报得出拒的是什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DefaultSlotsPerRegionBelowTheFirstVersionFloor {
    pub fixed_structure_slot_spacing: u32,
    /// ⌊P × chunk ÷ 槽距⌋：这个槽距下区域间距里装得下几个槽。
    pub slots_fitting_in_the_region_stride: u64,
    /// [`FIRST_VERSION_MAKE_FILESYSTEM_SMALLEST_DEFAULT_SLOTS_PER_REGION`]。
    pub smallest_default_slots_per_region: u64,
}

impl RootRingSlotsPerRegion {
    /// 显式给 S 的建池入口（测试与装置）常取的那个 S，也是第一版 mkfs 默认值的上限
    /// （[`Self::first_version_make_filesystem_default_for_the_slot_spacing`] 的 min 里那个 8）。
    /// 在常量上下文里求值：碰到区间的边连编译都过不去（[`Self::strictly_between_the_edges_of_the_format_interval`]）。
    pub const AT_MAKE_FILESYSTEM: Self = Self::strictly_between_the_edges_of_the_format_interval(
        ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
    );

    /// mkfs 默认值进这个类型的口子：值严格落在区间两条边之间（D22（单元原子性怎么合成） 已定项 16 第 3 句「默认值不取两条边」）。
    /// [`Self::AT_MAKE_FILESYSTEM`] 在常量上下文里调它，断言在编译期判；[`Self::first_version_make_filesystem_default_for_the_slot_spacing`]
    /// 在运行时调它，同一条断言罩住按槽距算出来的每一个值。
    ///
    /// # Panics
    /// 值碰到或越过区间的边：mkfs 默认值的式子（min(8, …)、小于 5 拒）出了错。
    const fn strictly_between_the_edges_of_the_format_interval(slots_per_region: u64) -> Self {
        assert!(
            slots_per_region > ROOT_RING_SLOTS_PER_REGION_MINIMUM
                && slots_per_region < ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
            "mkfs 默认的 S 碰到了区间的边（D22 已定项 16 第 3 句：默认值不取 4、16 两条边）"
        );
        Self { slots_per_region }
    }

    /// 第一版 mkfs 的默认 S：min(8, ⌊P × chunk ÷ 固定结构槽距⌋)，小于 5 拒（D22（单元原子性怎么合成） 已定项 16 第 3 句，用户 2026-10-22 定）。
    /// 算出来的值落在 [5, 8] 里，不碰区间的两条边；这一条在运行时经 [`Self::strictly_between_the_edges_of_the_format_interval`] 判。
    /// 只管 mkfs 的默认取值：显式给 S 的建池入口（`MakeFilesystemParameters` 的 `geometry` 里直接放一个 S）不经这里，
    /// 只受读者的区间判与 S × 槽距判。
    ///
    /// # Errors
    /// ⌊P × chunk ÷ 槽距⌋ 小于 5 ⇒ [`DefaultSlotsPerRegionBelowTheFirstVersionFloor`]（槽距 1 MiB 时是 3）。
    ///
    /// # Panics
    /// 槽距是 0：mkfs 的槽距由 `SystemImmutableSizes::slot_spacing_for` 算，不小于 4096。
    pub fn first_version_make_filesystem_default_for_the_slot_spacing(
        fixed_structure_slot_spacing: u32,
    ) -> Result<Self, DefaultSlotsPerRegionBelowTheFirstVersionFloor> {
        let slots_fitting_in_the_region_stride = (ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES)
            .checked_div(u64::from(fixed_structure_slot_spacing))
            .expect("槽距不是 0：mkfs 的槽距由 slot_spacing_for 算，不小于 4096");
        let default_slots_per_region =
            slots_fitting_in_the_region_stride.min(ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM);
        if default_slots_per_region
            < FIRST_VERSION_MAKE_FILESYSTEM_SMALLEST_DEFAULT_SLOTS_PER_REGION
        {
            return Err(DefaultSlotsPerRegionBelowTheFirstVersionFloor {
                fixed_structure_slot_spacing,
                slots_fitting_in_the_region_stride,
                smallest_default_slots_per_region:
                    FIRST_VERSION_MAKE_FILESYSTEM_SMALLEST_DEFAULT_SLOTS_PER_REGION,
            });
        }
        Ok(Self::strictly_between_the_edges_of_the_format_interval(
            default_slots_per_region,
        ))
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

/// 读出来的一条根住在 `ring_slot` 这一槽合不合落点公式（D22（单元原子性怎么合成） 已定项 7 实例代号那一行、已定项 16；
/// 欠账 C743「重建分配器记新类时不核根住不住这一槽」）：合法写者只有发布路径（按 [`target_for_publish`] 写）与 mkfs
/// （第 0 代根种进三个区域各自的槽 0，[`crate::make_filesystem::genesis_root_ring_slots`]），所以第 0 代根
/// （txg 是 [`crate::make_filesystem::GENESIS_CHECKPOINT_TXG`]）照 mkfs 种根的槽另认，别的根只住公式算出的那一槽。
/// 不合的是错位读（读回别的槽里一条自证得过的根）：读根环的读者照「这一槽这一次读坏」记，不当这一槽住着那条根。
#[must_use]
pub fn root_sits_in_the_ring_slot_the_formula_gives(
    checkpoint_txg: CheckpointTxg,
    ring_slot: RootRingSlot,
    slots_per_region: RootRingSlotsPerRegion,
) -> bool {
    if checkpoint_txg == crate::make_filesystem::GENESIS_CHECKPOINT_TXG {
        return crate::make_filesystem::genesis_root_ring_slots()
            .any(|genesis_slot| genesis_slot == ring_slot);
    }
    target_for_publish(checkpoint_txg, slots_per_region) == ring_slot
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
            "新池新建文件 txg 3 = 区域 0 槽 1"
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

    /// 落点公式核（C743）：txg 4 的根只住公式算出的那一槽（S = 8：区域 1 槽 1），住在别的槽里（同区域的槽 2、别的区域的槽 1）判错位；
    /// 第 0 代根（txg 0）住三个区域各自的槽 0 都认（mkfs 种根的槽），住槽 1 判错位。
    /// 判别力：`crates/mutations.tsv`「根环一簇 R1：落点公式核恒认」那一行（谓词恒真）与「第 0 代根照公式只认区域 0」那一行，各让这一条红。
    #[test]
    fn a_root_sits_only_in_the_ring_slot_its_txg_targets_and_a_genesis_root_in_any_region_slot_zero(
    ) {
        let slots = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
        let formula_slot_of_txg_four = target_for_publish(CheckpointTxg(4), slots);
        assert_eq!(
            formula_slot_of_txg_four,
            RootRingSlot { region: 1, slot: 1 },
            "前提：S = 8 时 txg 4 落区域 1 槽 1"
        );
        assert!(
            root_sits_in_the_ring_slot_the_formula_gives(
                CheckpointTxg(4),
                formula_slot_of_txg_four,
                slots
            ),
            "txg 4 的根住在它的落点槽：认"
        );
        for misplaced in [
            RootRingSlot { region: 1, slot: 2 },
            RootRingSlot { region: 0, slot: 1 },
            RootRingSlot { region: 2, slot: 1 },
        ] {
            assert!(
                !root_sits_in_the_ring_slot_the_formula_gives(CheckpointTxg(4), misplaced, slots),
                "txg 4 的根住在 {misplaced:?}：错位读，不认"
            );
        }
        for region in 0..ROOT_RING_REGIONS {
            assert!(
                root_sits_in_the_ring_slot_the_formula_gives(
                    CheckpointTxg(0),
                    RootRingSlot { region, slot: 0 },
                    slots
                ),
                "第 0 代根住区域 {region} 的槽 0：mkfs 种根的槽，认"
            );
        }
        assert!(
            !root_sits_in_the_ring_slot_the_formula_gives(
                CheckpointTxg(0),
                RootRingSlot { region: 1, slot: 1 },
                slots
            ),
            "第 0 代根住区域 1 的槽 1：不是 mkfs 种根的槽，不认"
        );
    }

    /// 区间判（D22 已定项 2 参数表「每区槽数 S」那一格：下界 k_tol + 2 = 4，上界是格式常量 16）：
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
