//! 系统配置槽里每区槽数 S（偏移 362，一字节）越界、单元大小（偏移 305）又不是格式常量，两格同错、整槽校验和重封：两个读者都整池拒、
//! 点名的却是各自先判的那一格，照两边各自的判定次序钉（I-7.13（系统配置池级字段在读者收的范围里） 对根环起点那一格的写法，
//! `crates/singlefs-checker/invariants/02-根环.md` 里「照两边各自的判定次序钉」那一段）。core 解析系统配置槽时先判 S（`system_configuration`
//! 的 `parse_slot`），单元几何四格在解析之后的「读者收不收」那几判里（`recovery` 的 `system_configuration_values_this_reader_accepts`），
//! 所以择系统配置与可写挂载都拒成 S 越界、点名先读到的盘 0，盘上逐字节不变；池级 checker 在 `geometry_of`（判 S）之前判单元几何四格，
//! 两块盘四个槽都读成拒在单元大小那一格的不收之值，I-7.13 判红。
//! 偏移、常量与改成的值用例自己写死，不从实现里取：取了的话，实现把字段写到别处去这一条也跟着走。

use singlefs_checker::image::{
    judge_system_configuration_values_the_reader_accepts, system_configuration_slot_readings,
    ImageReader, Judgements, RefusedSystemConfigurationValue, SystemConfigurationSlotReading,
    UnitGeometryFieldThatDisagrees, SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::recovery::{choose_system_configuration, RecoveryFailure};
use singlefs_core::root_ring::{RootRingSlotsPerRegion, RootRingSlotsPerRegionOutOfRange};
use singlefs_core::system_configuration::{
    SystemImmutableSizes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_verification::memory_pool::SparseBlockDevice;

const IMAGE_BYTES: u64 = 4 << 30;
const PHYSICAL_BLOCK_SIZE: u32 = 512;
/// 系统配置槽 1 的偏移：io_min 不声明时固定结构槽距是 4096。
const SLOT_ONE_OFFSET: u64 = 4096;
/// 系统配置字段表（`.claude/kb/layout/01-first-txn.md` 一）：单元大小 305，4 字节；每区槽数 S 362，1 字节。
const UNIT_BYTES_OFFSET_IN_THE_SLOT: usize = 305;
const ROOT_RING_SLOTS_PER_REGION_OFFSET_IN_THE_SLOT: usize = 362;
/// 改成的单元大小：格式常量 32768 的两倍。
const UNIT_BYTES_REWRITTEN: u32 = 65536;
/// 改成的 S：格式上界 16 加一。
const SLOTS_PER_REGION_REWRITTEN: u8 = 17;
/// S 的格式区间：下界 k_tol + 2 = 4，上界 16（D22（单元原子性怎么合成） 已定项 2 参数表「每区槽数 S」那一格）。
const SLOTS_PER_REGION_MINIMUM: u64 = 4;
const SLOTS_PER_REGION_MAXIMUM: u64 = 16;

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: *b"singlefs-twocell",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_SIZE,
            minimum_input_output_bytes: 0,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(0),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

/// mkfs 之后把两块盘四个系统配置槽的单元大小改成 [`UNIT_BYTES_REWRITTEN`]、S 改成 [`SLOTS_PER_REGION_REWRITTEN`]、整槽校验和重封：
/// 改出来的槽 magic、校验和、incompat 三关照样过，拒它的只能是读者收不收那几判。
fn pool_with_every_system_configuration_slot_carrying_both_rewritten_fields(
) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_SIZE)),
            )
        })
        .collect();
    make_filesystem(&parameters(), &mut devices).expect("mkfs 收这一份几何");
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for (_, device) in &mut devices {
        for offset in [0, SLOT_ONE_OFFSET] {
            let mut slot = vec![0u8; slot_bytes];
            device
                .read_at(DeviceOffsetInBytes(offset), &mut slot)
                .expect("系统配置槽在盘内");
            slot[UNIT_BYTES_OFFSET_IN_THE_SLOT..UNIT_BYTES_OFFSET_IN_THE_SLOT + 4]
                .copy_from_slice(&UNIT_BYTES_REWRITTEN.to_le_bytes());
            slot[ROOT_RING_SLOTS_PER_REGION_OFFSET_IN_THE_SLOT] = SLOTS_PER_REGION_REWRITTEN;
            let digest = wide_checksum_with_field_zeroed(
                &slot,
                slot_bytes,
                SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            );
            slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            device
                .write_at(DeviceOffsetInBytes(offset), &slot, WriteDurability::Plain)
                .expect("改完的槽写得回去");
        }
    }
    devices
}

/// 池级 checker 按字节读这几块盘：只给系统配置那几判用，扫描方向的候选一个都不给。
struct DevicesReadByThePoolChecker<'pool>(&'pool [(DeviceIdentity, SparseBlockDevice)]);

impl ImageReader for DevicesReadByThePoolChecker<'_> {
    fn devices(&self) -> Vec<u32> {
        self.0.iter().map(|(identity, _)| identity.0).collect()
    }
    fn device_bytes(&self, device: u32) -> Option<u64> {
        self.0
            .iter()
            .find(|(identity, _)| identity.0 == device)
            .map(|(_, sparse)| sparse.size_in_bytes())
    }
    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        let (_, sparse) = self.0.iter().find(|(identity, _)| identity.0 == device)?;
        let mut bytes = vec![0u8; length];
        sparse
            .read_at(DeviceOffsetInBytes(offset), &mut bytes)
            .ok()?;
        Some(bytes)
    }
    fn candidate_unit_slots(&self, _device: u32) -> Option<Vec<u64>> {
        Some(Vec::new())
    }
    fn candidate_journal_slots(&self, _device: u32) -> Option<Vec<u64>> {
        Some(Vec::new())
    }
}

/// 两块盘前 16 MiB（系统配置两槽 + 整条根环 + 一段余量）的字节：拒绝挂载那一格拿它比「盘上逐字节不变」。
fn front_of_every_device(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> Vec<Vec<u8>> {
    devices
        .iter()
        .map(|(_, device)| {
            let mut front = vec![0u8; 16 << 20];
            device
                .read_at(DeviceOffsetInBytes(0), &mut front)
                .expect("前 16 MiB 在盘内");
            front
        })
        .collect()
}

/// 两格同错：core 择系统配置与可写挂载都拒成 S 越界（盘 0 先读到），可写挂载之后盘上逐字节不变；池级 checker 四个槽都拒在单元大小那一格，I-7.13 判红。
#[test]
fn slots_per_region_out_of_range_with_a_unit_size_other_than_the_format_constant_is_refused_as_slots_per_region_by_core_and_as_unit_size_by_the_pool_checker(
) {
    let mut devices = pool_with_every_system_configuration_slot_carrying_both_rewritten_fields();
    let expected_by_core = RecoveryFailure::RootRingSlotsPerRegionOutOfRange {
        device: DeviceIdentity(0),
        out_of_range: RootRingSlotsPerRegionOutOfRange {
            declared_slots_per_region: u64::from(SLOTS_PER_REGION_REWRITTEN),
            minimum: SLOTS_PER_REGION_MINIMUM,
            maximum: SLOTS_PER_REGION_MAXIMUM,
        },
    };
    let chosen = choose_system_configuration(&devices);
    assert_eq!(
        chosen.as_ref().err(),
        Some(&expected_by_core),
        "core 择系统配置先判 S：整池拒成 S 越界、点名盘 0，不是单元大小，实际是 {chosen:?}"
    );
    let before = front_of_every_device(&devices);
    let mounted = mount_writable(&parameters(), &mut devices);
    assert!(
        matches!(&mounted, Err(MountError::Recovery(failure)) if *failure == expected_by_core),
        "可写挂载同样拒成 S 越界、点名盘 0，实际是 {:?}",
        mounted.err()
    );
    assert_eq!(
        front_of_every_device(&devices),
        before,
        "拒绝挂载在任何写之前：盘上逐字节不变"
    );
    let readings = system_configuration_slot_readings(&DevicesReadByThePoolChecker(&devices));
    assert_eq!(readings.len(), 2, "两块盘各两槽");
    for (device, slots) in &readings {
        assert_eq!(slots.len(), 2, "盘 {device} 两槽");
        for (slot_index, slot) in slots.iter().enumerate() {
            assert_eq!(
                slot,
                &SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses {
                    offset_in_bytes: if slot_index == 0 { 0 } else { SLOT_ONE_OFFSET },
                    refused: RefusedSystemConfigurationValue::UnitGeometryFieldDisagrees(
                        UnitGeometryFieldThatDisagrees::UnitBytesNotTheFormatConstant
                    ),
                },
                "池级 checker 先判单元几何四格：盘 {device} 槽 {slot_index} 拒在单元大小那一格，不是 S"
            );
        }
    }
    let mut judgements = Judgements::default();
    assert!(
        judge_system_configuration_values_the_reader_accepts(&readings, &mut judgements),
        "池级 checker 交回「有带越界值的槽」：整池不作保"
    );
    assert!(
        judgements
            .first_violation_of(SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS)
            .is_some(),
        "I-7.13 判红"
    );
}
