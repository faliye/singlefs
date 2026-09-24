//! 每区槽数 S 住系统配置（D22（单元原子性怎么合成） 已定项 1 的字段表逐字：「具体取多少不是格式决策——
//! 格式承诺的是『S 是系统配置字段 + 挂载时判区间』」，区间 4..16）。C506（每区槽数 S 写成编译期常量，条文说它住系统配置）
//! 之前 S 是 `singlefs-format` 里的一个编译期常量，全仓零处区间判。
//!
//! 两条用例各钉一半：
//! 一、**区间判**：盘上自述 S = 3 或 17 的池挂不上，报点名的成员、盘上逐字节不变；S = 4 / 8 / 16 三档都挂得上。
//! 二、**运行时 S 真的在用**：同一份实现，S 只从盘上那一字节来——改它，mkfs 清的那一段长度跟着变、
//!    挂载时环里看得见的根跟着变。
//!
//! ⚠️ 这两条读的都是**绝对数与盘上的字节**（清零段的末端偏移写死在用例里、根的 txg 逐条列出），
//! 不是「用实现算一遍再和实现比」：2026-09-23 定义轮那次实测，只改编译期常量时装置与实现一起动，
//! 26 行输出逐字不变——那正是共用一个数、逐格自洽的形状。

use singlefs_checker::image::geometry_of;
use singlefs_checker::{check_system_configuration_slot, Verdict};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, readable_roots, PoolReader, RecoveryFailure,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{region_start, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::{
    SystemImmutableSizes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::crash::SparseBlockDevice;

const IMAGE_BYTES: u64 = 4 << 30;
const PHYSICAL_BLOCK_SIZE: u32 = 512;
/// 固定结构槽距：mkfs 参数那一份（`u32`）与用例自己按偏移算位置那一份（`u64`）各写一遍，
/// 这一条要证明的就是「区域有多长由 S 定」，不借实现的 `region_length_in_bytes` 算。
const SLOT_SPACING: u32 = 4096;
const SLOT_SPACING_IN_BYTES: u64 = 4096;
/// 系统配置字段表（`.claude/kb/layout/01-first-txn.md` 一）里「每区槽数 S」那一字节的偏移：
/// R 在 361、S 在 362。用例自己写死这个数，不从实现里取——取了的话，实现把 S 写到别处去这两条也跟着走。
const SLOTS_PER_REGION_OFFSET_IN_THE_SLOT: usize = 362;
/// 每盘两个系统配置槽的盘内偏移（D22（单元原子性怎么合成） 已定项 16：槽 0 在 0、槽 1 在槽距处）。
const SYSTEM_CONFIGURATION_SLOT_OFFSETS: [u64; 2] = [0, SLOT_SPACING_IN_BYTES];

fn parameters_with(slots_per_region: u64) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: *b"singlefs-slots-S",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_SIZE,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: SLOT_SPACING,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                slots_per_region,
            )
            .expect("用例给的都是区间里的值"),
        },
    }
}

fn empty_devices() -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    (0..2u32)
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_SIZE)),
            )
        })
        .collect()
}

/// 建一个每区 `slots_per_region` 个槽的池；`dirty_from` 起的那一段在 mkfs 之前撒上非 0，
/// 好让「mkfs 清了多长」在盘上看得出来。
fn formatted_pool(
    slots_per_region: u64,
    dirty_region_zero_bytes: u64,
) -> (
    MakeFilesystemParameters,
    Vec<(DeviceIdentity, SparseBlockDevice)>,
    RootRecord,
) {
    let parameters = parameters_with(slots_per_region);
    let mut devices = empty_devices();
    if dirty_region_zero_bytes > 0 {
        let dirty =
            vec![0xA5u8; usize::try_from(dirty_region_zero_bytes).expect("撒脏的长度")];
        for (_, device) in devices.iter_mut() {
            device
                .write_at(region_start(0), &dirty, WriteDurability::Plain)
                .expect("撒脏写得进去");
        }
    }
    let output = make_filesystem(&parameters, &mut devices).expect("mkfs");
    (parameters, devices, output.root)
}

fn read_bytes(device: &SparseBlockDevice, offset: u64, length: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; length];
    device
        .read_at(DeviceOffsetInBytes(offset), &mut buffer)
        .expect("这几段都在盘内");
    buffer
}

fn read_slot(device: &SparseBlockDevice, offset: u64) -> Vec<u8> {
    read_bytes(
        device,
        offset,
        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096"),
    )
}

/// 把两块盘四个系统配置槽里「每区槽数 S」那一字节改成 `declared`，整槽校验和按改完的字节重算：
/// 改出来的槽 magic、校验和、incompat 三关照样过，拒它的只能是区间判。
fn declare_slots_per_region_on_disk(
    devices: &mut [(DeviceIdentity, SparseBlockDevice)],
    declared: u8,
) {
    for (_, device) in devices.iter_mut() {
        for offset in SYSTEM_CONFIGURATION_SLOT_OFFSETS {
            let mut slot = read_slot(device, offset);
            slot[SLOTS_PER_REGION_OFFSET_IN_THE_SLOT] = declared;
            let digest = wide_checksum_with_field_zeroed(
                &slot,
                usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096"),
                SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            );
            slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            device
                .write_at(DeviceOffsetInBytes(offset), &slot, WriteDurability::Plain)
                .expect("改完的槽写得回去");
        }
    }
}

/// 两块盘前 16 MiB（系统配置两槽 + 整条根环 + 一段余量）的字节：拒绝挂载那几格拿它比「盘上逐字节不变」。
fn front_of_every_device(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> Vec<Vec<u8>> {
    devices
        .iter()
        .map(|(_, device)| read_bytes(device, 0, 16 << 20))
        .collect()
}

/// 一、区间判：S = 3 与 17 两次都必须被拒，报点名的成员、带出盘上读到的值，盘上逐字节不变；
/// S = 4 / 8 / 16 三档都要挂得上，而且读回来的 S 就是各自写下去的那个。
#[test]
fn mounting_refuses_a_slots_per_region_outside_the_interval_and_accepts_four_eight_and_sixteen() {
    for declared in [3u8, 17] {
        let (parameters, mut devices, _) = formatted_pool(8, 0);
        declare_slots_per_region_on_disk(&mut devices, declared);
        let before = front_of_every_device(&devices);

        let refusal = mount_writable(&parameters, &mut devices);
        let Err(MountError::Recovery(RecoveryFailure::RootRingSlotsPerRegionOutOfRange {
            out_of_range,
            ..
        })) = refusal
        else {
            panic!("自述 S = {declared} 的池必须被点名拒绝，实际是 {refusal:?}");
        };
        assert_eq!(
            (
                out_of_range.declared_slots_per_region,
                out_of_range.minimum,
                out_of_range.maximum
            ),
            (u64::from(declared), 4, 16),
            "拒的时候要说得出盘上读到的是几、两条边是几"
        );
        assert_eq!(
            front_of_every_device(&devices),
            before,
            "拒绝挂载在任何写之前：盘上逐字节不变"
        );
    }

    for declared in [4u64, 8, 16] {
        let (parameters, mut devices, _) = formatted_pool(declared, 0);
        let mounted = mount_writable(&parameters, &mut devices);
        assert!(
            mounted.is_ok(),
            "S = {declared} 在区间里，池要挂得上：{:?}",
            mounted.err()
        );
        let system_configuration =
            choose_system_configuration(&devices).expect("挂得上的池择得出系统配置");
        assert_eq!(
            system_configuration
                .immutable
                .sizes
                .root_ring_slots_per_region
                .count(),
            declared,
            "挂载读回来的 S 就是 mkfs 写下去的那个"
        );
        assert_eq!(
            read_slot(&devices[0].1, 0)[SLOTS_PER_REGION_OFFSET_IN_THE_SLOT],
            u8::try_from(declared).expect("三档都装得进一字节"),
            "盘上偏移 362 那一字节就是它"
        );
    }
}

/// 同一条区间判，checker 那一侧也要有：它读的是同一字节，走读根环时按它枚举 R × S 个槽。
/// 两侧各写一份、不共用代码（D13（验证路线） 已定项 5），所以要各自钉一条。
#[test]
fn the_pool_checker_refuses_the_same_out_of_range_slots_per_region_the_mount_refuses() {
    let (_, mut devices, _) = formatted_pool(8, 0);
    for declared in [3u8, 17] {
        declare_slots_per_region_on_disk(&mut devices, declared);
        let slot = read_slot(&devices[0].1, 0);
        let view = check_system_configuration_slot(&slot).expect("这一槽自证得过");
        assert_eq!(
            geometry_of(&slot, &view).err(),
            Some(Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval),
            "checker 也拒自述 S = {declared} 的槽，不夹到区间里往下走"
        );
    }
    for declared in [4u8, 8, 16] {
        declare_slots_per_region_on_disk(&mut devices, declared);
        let slot = read_slot(&devices[0].1, 0);
        let view = check_system_configuration_slot(&slot).expect("这一槽自证得过");
        assert_eq!(
            geometry_of(&slot, &view)
                .expect("区间里的值收")
                .slots_per_region,
            u64::from(declared),
            "checker 取的 S 就是盘上那一字节"
        );
    }
}

/// 二、运行时 S 真的在用：S 只从盘上那一字节来，改它两处读数就变——
/// ① mkfs 清根环清的那一段长度（在盘上按绝对偏移读：`S × 4096` 之内是 0、之外还是撒下去的脏字节）；
/// ② 挂载时环里看得见的根（区域 0 槽 5 上那条根，S = 8 的池看得见、S = 4 的池看不见）。
///
/// ⚠️ 两处读数都是**绝对的**：清零段的末端按用例自己写的 `S × 4096` 算，根的 txg 与它落的槽逐条列出。
/// 换成「用实现的 `region_length_in_bytes` 再算一遍」的话，实现把 S 取错时装置跟着取错，两边逐格相等。
#[test]
fn the_ring_geometry_follows_the_slots_per_region_on_disk_not_a_compile_time_constant() {
    // ① mkfs 清的那一段：S 越大清得越长。区域 0 起点之后撒 64 KiB 脏字节，mkfs 之后逐档读。
    let dirty_bytes = 64u64 << 10;
    for (declared, cleared_bytes) in [(4u64, 16_384u64), (8, 32_768), (16, 65_536)] {
        let (_, devices, _) = formatted_pool(declared, dirty_bytes);
        let region_zero = region_start(0).0;
        for (identity, device) in &devices {
            // 槽 0 上有这次写下的第 0 代根（区域 0 归盘 0），从槽 1 起到清零段末尾必须全 0。
            let first_zero = region_zero + SLOT_SPACING_IN_BYTES;
            let zeroed = read_bytes(
                device,
                first_zero,
                usize::try_from(cleared_bytes - SLOT_SPACING_IN_BYTES).expect("清零段长度"),
            );
            assert!(
                zeroed.iter().all(|byte| *byte == 0),
                "S = {declared}：盘 {} 区域 0 的槽 1..{declared} 该被 mkfs 清成 0",
                identity.0
            );
            if cleared_bytes < dirty_bytes {
                let beyond = read_bytes(device, region_zero + cleared_bytes, 4096);
                assert!(
                    beyond.iter().all(|byte| *byte == 0xA5),
                    "S = {declared}：区域 0 第 {cleared_bytes} 字节之后不归根环，mkfs 不该碰它"
                );
            }
        }
    }

    // ② 挂载时环里看得见的根：在区域 0 槽 5 上种一条 txg 15 的根。
    //    S = 8 的池里槽 5 在环内，择根择得中它；把系统配置改成 S = 4（合法）之后槽 5 在环外，择根看不见它。
    let (_, mut devices, genesis_root) = formatted_pool(8, 0);
    let planted = RootRecord {
        checkpoint_txg: CheckpointTxg(15),
        ..genesis_root
    };
    let slot_five_offset = region_start(0).0 + 5 * SLOT_SPACING_IN_BYTES;
    let root_slot_bytes = usize::try_from(PHYSICAL_BLOCK_SIZE).expect("根槽宽");
    devices[0]
        .1
        .write_at(
            DeviceOffsetInBytes(slot_five_offset),
            &planted.to_slot(root_slot_bytes),
            WriteDurability::ForceUnitAccess,
        )
        .expect("种根写得进去");

    let with_eight = choose_system_configuration(&devices).expect("S = 8 的池择得出系统配置");
    assert_eq!(
        choose_root(&devices, &with_eight).map(|root| root.checkpoint_txg),
        Some(CheckpointTxg(15)),
        "S = 8：区域 0 槽 5 在环内，择根择得中种下去的那条"
    );
    assert_eq!(
        readable_roots(
            &devices,
            &with_eight.immutable.region_devices,
            &with_eight.immutable.sizes,
            &with_eight.immutable.filesystem_identifier,
        )
        .len(),
        4,
        "S = 8：mkfs 的三条第 0 代根 + 种下去的那条"
    );

    declare_slots_per_region_on_disk(&mut devices, 4);
    let with_four = choose_system_configuration(&devices).expect("S = 4 也在区间里");
    assert_eq!(
        with_four.immutable.sizes.root_ring_slots_per_region.count(),
        4
    );
    assert_eq!(
        choose_root(&devices, &with_four).map(|root| root.checkpoint_txg),
        Some(CheckpointTxg(0)),
        "S = 4：槽 5 在环外，择根只看得见 mkfs 那三条第 0 代根——同一份盘、同一份实现，只有那一字节不同"
    );
    assert_eq!(
        readable_roots(
            &devices,
            &with_four.immutable.region_devices,
            &with_four.immutable.sizes,
            &with_four.immutable.filesystem_identifier,
        )
        .len(),
        3,
        "S = 4：环里只有 mkfs 那三条"
    );
    assert_eq!(
        PoolReader::read(
            &devices,
            DeviceIdentity(0),
            DeviceOffsetInBytes(slot_five_offset),
            root_slot_bytes,
        )
        .expect("槽 5 的字节还在盘上"),
        planted.to_slot(root_slot_bytes),
        "看不见它不是因为它被抹了，是因为 S = 4 的环不罩这一槽"
    );
}
