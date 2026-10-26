//! 第一版 mkfs 的默认每区槽数 S（D22（单元原子性怎么合成） 已定项 16 第 3 句，m3-close-c506 三轮判决第三节，用户 2026-10-22 定）：
//! 默认 S = min(8, ⌊P × chunk ÷ 固定结构槽距⌋)，小于 5 拒绝 mkfs，默认值不取区间 [4, 16] 的两条边。
//! 只管默认值：显式给 S 的建池入口（`MakeFilesystemParameters` 的 `geometry` 里直接放 S，测试与装置今天用的那条）照旧，只受读者那两判。
//! 三条用例：槽距 512 KiB 时默认写 6、两个读者都收下；槽距 1 MiB 时拒；格式允许的每一档槽距上默认值都落不到 4 或 16。
//! 期望的数用例自己写死（6、3、[5, 8]），不拿实现的式子再算一遍。

use singlefs_checker::check_system_configuration_slot;
use singlefs_checker::image::geometry_of;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::make_filesystem::{
    first_version_make_filesystem_geometry, make_filesystem, MakeFilesystemError,
    MakeFilesystemParameters,
};
use singlefs_core::recovery::choose_system_configuration;
use singlefs_core::root_ring::{
    DefaultSlotsPerRegionBelowTheFirstVersionFloor, RootRingSlotsPerRegion,
};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::memory_pool::SparseBlockDevice;

const IMAGE_BYTES: u64 = 4 << 30;
const PHYSICAL_BLOCK_SIZE: u32 = 512;
/// 系统配置字段表（`.claude/kb/layout/01-first-txn.md` 一）里「每区槽数 S」那一字节的偏移，用例自己写死。
const SLOTS_PER_REGION_OFFSET_IN_THE_SLOT: usize = 362;
/// 格式允许的固定结构槽距：4096 起、每 512 一档，到根环起点 1 MiB 减系统配置槽宽 4096 为止（D22（单元原子性怎么合成） 已定项 16
/// 第 5 句；D2（RAID 条带策略） 已定项 19 的槽距都是 512 的整数倍）。
const SMALLEST_SLOT_SPACING: u32 = 4096;
const LARGEST_SLOT_SPACING: u32 = (1 << 20) - 4096;
const SLOT_SPACING_STEP: usize = 512;

/// 槽距 512 KiB（io_min 512 KiB）：⌊3 MiB ÷ 512 KiB⌋ = 6，默认 S 写 6；6 × 512 KiB = 3 MiB 恰等于区域间距（写 ≤ 收），
/// mkfs 收下、core 择系统配置收下、池级 checker 收下，盘上偏移 362 那一字节是 6。
#[test]
fn the_first_version_default_at_a_512_kibibyte_spacing_writes_six_and_both_readers_accept_the_pool()
{
    let geometry = first_version_make_filesystem_geometry(
        PHYSICAL_BLOCK_SIZE,
        512 << 10,
        JOURNAL_RING_DEFAULT_BYTES,
    )
    .expect("槽距 512 KiB 时默认 S 是 6，不小于 5");
    assert_eq!(
        geometry.fixed_structure_slot_spacing,
        512 << 10,
        "槽距跟 io_min 走"
    );
    assert_eq!(
        geometry.root_ring_slots_per_region.count(),
        6,
        "默认 S = min(8, ⌊3 MiB ÷ 512 KiB⌋) = 6"
    );
    let parameters = MakeFilesystemParameters {
        filesystem_identifier: *b"singlefs-default",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry,
    };
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_SIZE)),
            )
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("6 × 512 KiB 恰等于区域间距，mkfs 收");
    let mut slot = vec![0u8; usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")];
    devices[0]
        .1
        .read_at(DeviceOffsetInBytes(0), &mut slot)
        .expect("盘 0 槽 0");
    assert_eq!(
        slot[SLOTS_PER_REGION_OFFSET_IN_THE_SLOT], 6,
        "盘上偏移 362 那一字节是 6"
    );
    let chosen = choose_system_configuration(&devices).expect("core 择系统配置收下这个池");
    assert_eq!(chosen.immutable.sizes.root_ring_slots_per_region.count(), 6);
    let view = check_system_configuration_slot(&slot).expect("mkfs 写的槽自证得过");
    assert_eq!(
        geometry_of(&slot, &view)
            .expect("池级 checker 收下这一槽")
            .slots_per_region,
        6
    );
}

/// 槽距 1 MiB（io_min 1 MiB）：⌊3 MiB ÷ 1 MiB⌋ = 3，小于 5，mkfs 在算几何那一步拒、报出算式两边的数。
#[test]
fn the_first_version_default_at_a_one_mebibyte_spacing_is_refused_by_mkfs() {
    let refusal = first_version_make_filesystem_geometry(
        PHYSICAL_BLOCK_SIZE,
        1 << 20,
        JOURNAL_RING_DEFAULT_BYTES,
    );
    assert!(
        matches!(
            refusal,
            Err(
                MakeFilesystemError::DefaultSlotsPerRegionBelowTheFirstVersionFloor(
                    DefaultSlotsPerRegionBelowTheFirstVersionFloor {
                        fixed_structure_slot_spacing: 1_048_576,
                        slots_fitting_in_the_region_stride: 3,
                        smallest_default_slots_per_region: 5,
                    }
                )
            )
        ),
        "槽距 1 MiB 时区域间距里只装得下 3 个槽，小于 5 拒：{refusal:?}"
    );
}

/// 格式允许的每一档槽距上，默认 S 要么被拒，要么落在 [5, 8] 里：落不到区间的两条边 4 与 16（「默认值不取两条边」）。
/// 两种结局都要出现：小槽距上是 8，槽距越过 3 MiB ÷ 5 之后拒。
#[test]
fn the_first_version_default_never_lands_on_either_edge_of_the_interval() {
    let mut accepted_spacings = 0usize;
    let mut refused_spacings = 0usize;
    for fixed_structure_slot_spacing in
        (SMALLEST_SLOT_SPACING..=LARGEST_SLOT_SPACING).step_by(SLOT_SPACING_STEP)
    {
        match RootRingSlotsPerRegion::first_version_make_filesystem_default_for_the_slot_spacing(
            fixed_structure_slot_spacing,
        ) {
            Ok(default_slots_per_region) => {
                accepted_spacings += 1;
                let count = default_slots_per_region.count();
                assert!(
                    (5..=8).contains(&count),
                    "槽距 {fixed_structure_slot_spacing} 时默认 S 是 {count}，不在 [5, 8] 里（碰到 4 或 16 就是取了边）"
                );
            }
            Err(refusal) => {
                refused_spacings += 1;
                assert!(
                    refusal.slots_fitting_in_the_region_stride < 5,
                    "槽距 {fixed_structure_slot_spacing} 被拒，而区域间距里装得下 {} 个槽",
                    refusal.slots_fitting_in_the_region_stride
                );
            }
        }
    }
    assert!(
        accepted_spacings > 0 && refused_spacings > 0,
        "两种结局都要出现：收 {accepted_spacings} 档、拒 {refused_spacings} 档"
    );
    assert_eq!(
        RootRingSlotsPerRegion::first_version_make_filesystem_default_for_the_slot_spacing(
            SMALLEST_SLOT_SPACING
        )
        .map(RootRingSlotsPerRegion::count),
        Ok(8),
        "最小槽距 4096 时默认 S 取上限 8"
    );
}
