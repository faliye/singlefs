//! 全池没有可用的系统配置槽 0 时，池级 checker 与恢复找槽 1 的读法相同、结论相同（C651（全池没有可用的系统配置槽 0 时 checker
//! 与实现找槽 1 的读法不同），第一轮判决 `research/prompts/m3-c651-r1-main-verification.md` 第四节三条与「用户定案」W1）：
//! 在格式允许的每个槽距上各读一槽（不小于 4096、512 的整数倍、不越过根环起点 × 16384 − 4096），只收自证过、且自述的固定结构槽距
//! 等于所在偏移的槽；几档都收得到时取世代号最大的，世代号相同取偏移小的；读到带读者不收之值的槽整池拒（读到可用的一档也先判
//! I-8.1 三个字段等式、再核偏移，代码轮第二轮判决 K3-1 `research/prompts/m3-formal-code-r2-main-verification.md`）；
//! mkfs 在写任何元数据之前把每块盘的 [0, 根环基址) 整段写 0，在用过的盘上换一个 io_min 重做 mkfs，旧池别的槽距上的槽 1 不再被择中。
//! 盘都是内存稀疏盘（`singlefs_harness::memory_pool::SparseBlockDevice`），两边读的是同一份 `MemoryPool`。

use singlefs_checker::image::{
    chosen_system_configurations, system_configuration_slot_readings, InvariantVerdict,
    JournalRingGeometryFieldThatDisagrees, RefusedSystemConfigurationValue,
    SystemConfigurationSlotReading, SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
    SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{
    choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryFailure,
    RecoveryOutcome, SystemConfigurationValueOutsideWhatThisReaderAccepts,
};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::{
    SystemConfiguration, SystemImmutableSizes,
    INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT,
    SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

const DEVICE_BYTES: u64 = 4 << 30;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-c651-r1";
/// 内存稀疏盘按 512 字节扇区存：物理块宽 512。
const SECTOR_PHYSICAL_BLOCK_SIZE: PhysicalBlockSizeInBytes = PhysicalBlockSizeInBytes(512);
/// io_min 512 的池：槽距 4096（D2（RAID 条带策略） 已定项 19 的式子）。
const MINIMUM_INPUT_OUTPUT_BYTES_OF_A_4096_SPACING: u32 = 512;
/// io_min 8192 的池：槽距 8192，槽 1 在 8192 处（C651 的那一格）。
const MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING: u32 = 8192;
/// 系统配置槽里 incompat 位图第一个字节的偏移：magic 4 + 格式版本 2（与 `core_review_tree_table_duplicates_and_slot_one_search.rs` 同一处）。
const FIRST_INCOMPAT_BYTE_OFFSET: usize = 6;
/// journal 环起点字段写成第一版常量 1024 之外的一个槽号：自证得过、读者不收（D22（单元原子性怎么合成） 已定项 16 第 1 句）。
const JOURNAL_RING_START_SLOT_THE_READER_REFUSES: u64 = 1025;
/// 系统配置槽里在飞上限那 4 字节的偏移（与 `journal_ring_geometry_fields_are_recomputed_when_choosing_the_system_configuration.rs` 同一处）。
const IN_FLIGHT_RECORD_LIMIT_OFFSET_IN_THE_SLOT: usize = 345;
/// 代码轮第二轮判决 K3-a 攻方原型放槽的偏移：io_min 8192 的池里，8192 之后下一个不是 8192 整数倍的 512 档（槽距 12288 的槽 1 也在这里）。
const OFFSET_OF_THE_PLANTED_SLOT: u64 = 12288;

fn geometry_with_minimum_input_output_bytes(
    minimum_input_output_bytes: u32,
) -> SystemImmutableSizes {
    SystemImmutableSizes {
        physical_block_size: 512,
        minimum_input_output_bytes,
        fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(
            minimum_input_output_bytes,
        ),
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
    }
}

fn parameters_with_minimum_input_output_bytes(
    minimum_input_output_bytes: u32,
) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: geometry_with_minimum_input_output_bytes(minimum_input_output_bytes),
    }
}

fn two_sparse_devices() -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(DEVICE_BYTES, SECTOR_PHYSICAL_BLOCK_SIZE),
            )
        })
        .collect()
}

fn image_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: DEVICE_BYTES,
    }
}

fn slot_bytes() -> usize {
    usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")
}

/// 按 `minimum_input_output_bytes` 算的槽距做一次 mkfs（没有文件），交回内存池。
fn formatted_image_with_minimum_input_output_bytes(minimum_input_output_bytes: u32) -> MemoryPool {
    let mut devices = two_sparse_devices();
    make_filesystem(
        &parameters_with_minimum_input_output_bytes(minimum_input_output_bytes),
        &mut devices,
    )
    .expect("mkfs");
    image_of(&devices)
}

/// 两块盘偏移 `offset` 那一个系统配置槽整槽抹成 0：读得出、magic 不对、自证不过。
fn wipe_the_slot_at_on_every_device(image: &mut MemoryPool, offset: u64) {
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        image
            .devices
            .get_mut(&device)
            .expect("池里有这块盘")
            .write(DeviceOffsetInBytes(offset), &vec![0u8; slot_bytes()]);
    }
}

fn slot_at(image: &MemoryPool, device: DeviceIdentity, offset: u64) -> SystemConfiguration {
    SystemConfiguration::parse_slot(
        &PoolReader::read(image, device, DeviceOffsetInBytes(offset), slot_bytes())
            .expect("槽读得到"),
    )
    .expect("这一槽自证得过")
}

fn plant_slot(
    image: &mut MemoryPool,
    device: DeviceIdentity,
    offset: u64,
    system_configuration: &SystemConfiguration,
) {
    image
        .devices
        .get_mut(&device)
        .expect("池里有这块盘")
        .write(DeviceOffsetInBytes(offset), &system_configuration.to_slot());
}

/// 整槽校验和重算：改过字节的槽 magic 与校验和两关照样过，拒不拒只看被改的那一格
/// （与 `core_review_tree_table_duplicates_and_slot_one_search.rs` 同一个做法）。
fn reseal_system_configuration_slot(slot: &mut [u8]) {
    let digest =
        wide_checksum_with_field_zeroed(slot, slot_bytes(), SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
    slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
        .copy_from_slice(&digest);
}

/// 恢复择到的那一份：（盘、固定结构槽距、世代号）。
fn chosen_by_recovery(image: &MemoryPool) -> (u32, u64, u64) {
    let chosen = choose_system_configuration(image).expect("恢复择得到系统配置");
    (
        chosen.immutable.this_device.0,
        u64::from(chosen.immutable.sizes.fixed_structure_slot_spacing),
        chosen.quantities.slot_generation,
    )
}

/// 池级 checker 每块盘择到的那一份：（盘、固定结构槽距、世代号），一块盘择不到就是 None。
fn chosen_by_the_checker(image: &MemoryPool) -> Vec<(u32, Option<(u64, u64)>)> {
    chosen_system_configurations(image)
        .into_iter()
        .map(|(device, chosen)| {
            (
                device,
                chosen.map(|(view, geometry)| (geometry.slot_spacing, view.slot_generation)),
            )
        })
        .collect()
}

fn violated_invariants(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

fn verdict_of(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(name, _)| *name == invariant)
        .map(|(_, verdict)| verdict)
        .expect("池级 checker 报这条不变量")
}

/// C651 那一格：io_min 8192 的池（槽距 8192），两块盘的槽 0 都抹掉。恢复逐档择到盘 0 在 8192 处的槽 1；
/// checker 也逐档读，两块盘都择到 8192 处 mkfs 写下的世代号 1，I-8.5（tail 槽有效集非空） 成立、一条不变量都不红。
/// checker 按最小槽距 4096 读时 4096 处是 mkfs 清过的 0，两块盘都择不到、I-8.5 判红——与恢复结论相反。
#[test]
fn an_io_min_8192_pool_whose_slot_zeros_are_wiped_has_the_checker_and_recovery_both_choose_slot_one_at_8192(
) {
    let spacing = u64::from(MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING);
    let mut image = formatted_image_with_minimum_input_output_bytes(
        MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING,
    );
    wipe_the_slot_at_on_every_device(&mut image, 0);
    assert_eq!(
        chosen_by_recovery(&image),
        (0, spacing, 1),
        "恢复择到盘 0 在 8192 处的槽 1（mkfs 写下的世代号 1）"
    );
    assert_eq!(
        recover(&image, JournalPolicy::Consult).outcome,
        RecoveryOutcome::NoFile {
            root: (InstanceGeneration(0), CheckpointTxg(0)),
        },
        "恢复走到 mkfs 的第 0 代根"
    );
    assert_eq!(
        chosen_by_the_checker(&image),
        vec![(0, Some((spacing, 1))), (1, Some((spacing, 1)))],
        "checker 两块盘都择到 8192 处世代号 1 的槽 1"
    );
    assert_eq!(
        verdict_of(
            &image,
            SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
        ),
        InvariantVerdict::Holds,
        "两块盘的槽 1 自证过：I-8.5 成立"
    );
    assert_eq!(
        violated_invariants(&image),
        Vec::<(&str, String)>::new(),
        "恢复挂得上的池，checker 一条不变量都不红"
    );
}

/// 只收自述的固定结构槽距等于所在偏移的槽（第四节第 1 条）：io_min 8192 的池，4096 处放一份自证得过、世代号 9、记着槽距 8192 的槽
/// （它不在自己记的槽距上），两块盘的槽 0 都抹掉：恢复与 checker 都不收它，都择 8192 处世代号 1 的那一份。
#[test]
fn a_self_verified_slot_whose_recorded_spacing_is_not_its_own_offset_is_taken_by_neither_the_checker_nor_recovery(
) {
    let spacing = u64::from(MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING);
    let mut image = formatted_image_with_minimum_input_output_bytes(
        MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING,
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut misplaced = slot_at(&image, device, spacing);
        misplaced.quantities.slot_generation = 9;
        plant_slot(&mut image, device, 4096, &misplaced);
    }
    wipe_the_slot_at_on_every_device(&mut image, 0);
    assert_eq!(
        chosen_by_recovery(&image),
        (0, spacing, 1),
        "4096 处那一份记的槽距是 8192，恢复不收；择到 8192 处世代号 1 的"
    );
    assert_eq!(
        chosen_by_the_checker(&image),
        vec![(0, Some((spacing, 1))), (1, Some((spacing, 1)))],
        "checker 同样不收 4096 处那一份"
    );
}

/// 几档都收得到时取世代号最大的（第四节第 2 条）：槽距 4096 的池，8192 处放一份同 fsid、记着槽距 8192、世代号 5 的槽，
/// 两块盘的槽 0 都抹掉。4096 处世代号 1、8192 处世代号 5 都自证过、记的槽距都等于所在偏移：恢复与 checker 都择世代号 5 的那一份。
#[test]
fn slots_found_at_several_spacings_resolve_to_the_highest_generation_in_both_the_checker_and_recovery(
) {
    let mut image = formatted_image_with_minimum_input_output_bytes(
        MINIMUM_INPUT_OUTPUT_BYTES_OF_A_4096_SPACING,
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut at_8192 = slot_at(&image, device, 4096);
        at_8192.immutable.sizes.fixed_structure_slot_spacing = 8192;
        at_8192.quantities.slot_generation = 5;
        plant_slot(&mut image, device, 8192, &at_8192);
    }
    wipe_the_slot_at_on_every_device(&mut image, 0);
    assert_eq!(
        chosen_by_recovery(&image),
        (0, 8192, 5),
        "恢复择到 8192 处世代号 5 的那一份"
    );
    assert_eq!(
        chosen_by_the_checker(&image),
        vec![(0, Some((8192, 5))), (1, Some((8192, 5)))],
        "checker 与恢复择同一份：8192 处世代号 5"
    );
}

/// 世代号相同时取偏移小的（第四节第 2 条）：槽距 4096 的池，8192 处放一份同 fsid、记着槽距 8192、世代号同为 1 的槽，
/// 两块盘的槽 0 都抹掉：恢复与 checker 都择 4096 处本池的那一份。
#[test]
fn slots_of_equal_generation_found_at_several_spacings_resolve_to_the_smaller_offset_in_both_the_checker_and_recovery(
) {
    let mut image = formatted_image_with_minimum_input_output_bytes(
        MINIMUM_INPUT_OUTPUT_BYTES_OF_A_4096_SPACING,
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut at_8192 = slot_at(&image, device, 4096);
        at_8192.immutable.sizes.fixed_structure_slot_spacing = 8192;
        plant_slot(&mut image, device, 8192, &at_8192);
    }
    wipe_the_slot_at_on_every_device(&mut image, 0);
    assert_eq!(
        chosen_by_recovery(&image),
        (0, 4096, 1),
        "世代号都是 1：恢复取偏移小的 4096 那一份"
    );
    assert_eq!(
        chosen_by_the_checker(&image),
        vec![(0, Some((4096, 1))), (1, Some((4096, 1)))],
        "checker 与恢复择同一份：4096 处世代号 1"
    );
}

/// 逐档读到一份带读者不收之值的槽（8192 处，journal 环起点写成 1025、整槽校验和重算过）：恢复整池拒、点名盘 0；
/// checker 报 I-7.13（系统配置池级字段在读者收的范围里） 违例、整池不作保（D22（单元原子性怎么合成） 已定项 16 第 5 句「越界即拒」）。
/// 4096 处本池的槽 1 照样自证得过、可择，两边都不因为它而跳过那一份。
#[test]
fn a_slot_carrying_a_value_the_reader_refuses_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery(
) {
    let mut image = formatted_image_with_minimum_input_output_bytes(
        MINIMUM_INPUT_OUTPUT_BYTES_OF_A_4096_SPACING,
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut refused = slot_at(&image, device, 4096);
        refused.immutable.journal_ring_start_slot =
            SlotNumber(JOURNAL_RING_START_SLOT_THE_READER_REFUSES);
        plant_slot(&mut image, device, 8192, &refused);
    }
    wipe_the_slot_at_on_every_device(&mut image, 0);
    assert!(
        matches!(
            choose_system_configuration(&image),
            Err(RecoveryFailure::SystemConfigurationValueRefused {
                device: DeviceIdentity(0),
                ..
            })
        ),
        "恢复整池拒、点名盘 0：{:?}",
        choose_system_configuration(&image).err()
    );
    let readings = system_configuration_slot_readings(&image);
    assert!(
        readings.iter().all(|(_, slots)| matches!(
            slots[1],
            SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses {
                offset_in_bytes: 8192,
                ..
            }
        )),
        "checker 两块盘的槽 1 都读成 8192 处那份带不收之值的槽：{readings:?}"
    );
    assert!(
        matches!(
            verdict_of(
                &image,
                SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS
            ),
            InvariantVerdict::Violated(_)
        ),
        "checker 报 I-7.13 违例"
    );
}

/// io_min 8192 的池（槽距 8192），两块盘的槽 0 整槽写 0，盘 0 在 12288 处放一份由 mkfs 写下的槽 1 改出来的槽：在飞上限 + 1、
/// `edit_the_slot` 再改几格、整槽校验和重封（代码轮第二轮判决 K3-a 攻方原型 `k3slot` 那两种槽的造法）。交回这份镜像。
fn io_min_8192_pool_with_an_in_flight_limit_plus_one_slot_planted_at_12288_on_device_zero(
    edit_the_slot: impl Fn(&mut SystemConfiguration),
) -> MemoryPool {
    let mut image = formatted_image_with_minimum_input_output_bytes(
        MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING,
    );
    let mut planted = slot_at(
        &image,
        DeviceIdentity(0),
        u64::from(MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING),
    );
    edit_the_slot(&mut planted);
    let mut planted_bytes = planted.to_slot();
    let in_flight_record_limit_field = IN_FLIGHT_RECORD_LIMIT_OFFSET_IN_THE_SLOT
        ..IN_FLIGHT_RECORD_LIMIT_OFFSET_IN_THE_SLOT + std::mem::size_of::<u32>();
    let recorded_in_flight_record_limit = u32::from_le_bytes(
        planted_bytes[in_flight_record_limit_field.clone()]
            .try_into()
            .expect("在飞上限是 4 字节"),
    );
    planted_bytes[in_flight_record_limit_field]
        .copy_from_slice(&(recorded_in_flight_record_limit + 1).to_le_bytes());
    reseal_system_configuration_slot(&mut planted_bytes);
    image
        .devices
        .get_mut(&DeviceIdentity(0))
        .expect("池里有这块盘")
        .write(
            DeviceOffsetInBytes(OFFSET_OF_THE_PLANTED_SLOT),
            &planted_bytes,
        );
    wipe_the_slot_at_on_every_device(&mut image, 0);
    image
}

/// 同一份镜像上恢复与 checker 的结局：恢复整池拒、点名盘 0、拒的是在飞上限那一格；checker 盘 0 的槽 1 读成 12288 处那份带不收之值的槽、
/// 报 I-7.13（系统配置池级字段在读者收的范围里） 违例，盘 1 照旧择到 8192 处 mkfs 写下的世代号 1。
fn assert_recovery_refuses_the_pool_and_the_checker_reads_the_planted_slot_as_carrying_a_refused_value(
    image: &MemoryPool,
) {
    let recovery_choice = choose_system_configuration(image);
    assert!(
        matches!(
            recovery_choice,
            Err(RecoveryFailure::SystemConfigurationValueRefused {
                device: DeviceIdentity(0),
                value: SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalInFlightRecordLimitNotRecomputedFromTheRingLength { .. },
            })
        ),
        "恢复逐档读到 12288 处那一份，先判值、整池拒、点名盘 0 的在飞上限：{recovery_choice:?}"
    );
    let readings = system_configuration_slot_readings(image);
    assert_eq!(
        readings
            .iter()
            .find(|(device, _)| *device == 0)
            .map(|(_, slots)| slots[1].clone()),
        Some(SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses {
            offset_in_bytes: OFFSET_OF_THE_PLANTED_SLOT,
            refused: RefusedSystemConfigurationValue::JournalRingGeometryFieldDisagrees(
                JournalRingGeometryFieldThatDisagrees::InFlightRecordLimitNotRecomputedFromTheRingLength,
            ),
        }),
        "checker 盘 0 逐档读到 12288 处那一份，先判 I-8.1 三格、当场交回带不收之值，不核偏移、不比世代号：{readings:?}"
    );
    assert_eq!(
        chosen_by_the_checker(image),
        vec![
            (0, None),
            (
                1,
                Some((u64::from(MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING), 1))
            )
        ],
        "checker 盘 0 择不到；盘 1 没有那一份，照旧择到 8192 处世代号 1"
    );
    assert!(
        matches!(
            verdict_of(
                image,
                SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS
            ),
            InvariantVerdict::Violated(_)
        ),
        "checker 报 I-7.13 违例、整池不作保，与恢复整池拒同一个结局"
    );
}

/// 攻方原型 `inflight_plus1_spacing_8192_at_12288`：12288 处那一份自述槽距 8192（不在自己记的槽距上）、世代号 1、在飞上限 + 1。
/// 实现先判值、整池拒；checker 改前先核偏移，把它当「不在自己槽距上」跳过，择 8192 处世代号 1、全绿。
#[test]
fn a_slot_whose_in_flight_limit_is_off_by_one_and_whose_spacing_is_not_its_offset_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery(
) {
    let image =
        io_min_8192_pool_with_an_in_flight_limit_plus_one_slot_planted_at_12288_on_device_zero(
            |_unchanged| {},
        );
    assert_recovery_refuses_the_pool_and_the_checker_reads_the_planted_slot_as_carrying_a_refused_value(
        &image,
    );
}

/// 攻方原型 `inflight_plus1_spacing_equals_offset_gen0`：12288 处那一份自述槽距 12288（等于所在偏移）、世代号 0、在飞上限 + 1。
/// 实现先判值、整池拒；checker 改前核过偏移之后比世代号，0 比 8192 处的 1 小，择 8192 处世代号 1、全绿。
#[test]
fn a_slot_whose_in_flight_limit_is_off_by_one_and_whose_generation_is_lower_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery(
) {
    let image =
        io_min_8192_pool_with_an_in_flight_limit_plus_one_slot_planted_at_12288_on_device_zero(
            |planted| {
                planted.immutable.sizes.fixed_structure_slot_spacing =
                    u32::try_from(OFFSET_OF_THE_PLANTED_SLOT).expect("12288 装得进 4 字节");
                planted.quantities.slot_generation = 0;
            },
        );
    assert_recovery_refuses_the_pool_and_the_checker_reads_the_planted_slot_as_carrying_a_refused_value(
        &image,
    );
}

/// 逐档读到的自证过的槽都不可择（4096 处本池的槽 1 也抹掉，8192 处放一份 incompat 位图不认识的槽）：恢复报 incompat 位图不认识、
/// 点名盘 0；checker 两块盘都择不到：这不是一个挂得上的镜像，每一条不变量都报不适用、带理由。I-8.5（tail 槽有效集非空） 也报不适用
/// （那一槽自证过，全池皆废那一格不成立，不判红；而一块盘都交不出可择的系统配置，也不判成立：`InvariantVerdict` 的「不适用必须带理由，
/// 不许静默算成立」）。改之前 I-8.5 在早退之前判成成立，坏盘输入战役（checker 档 `bad_disk_input_campaign`）把这类镜像算成三个读者都没看见。
#[test]
fn only_self_verified_slots_that_are_not_choosable_found_while_reading_every_spacing_choose_nothing_in_both_the_checker_and_recovery(
) {
    let mut image = formatted_image_with_minimum_input_output_bytes(
        MINIMUM_INPUT_OUTPUT_BYTES_OF_A_4096_SPACING,
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut not_choosable_slot = slot_at(&image, device, 4096).to_slot();
        not_choosable_slot[FIRST_INCOMPAT_BYTE_OFFSET] =
            INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT;
        reseal_system_configuration_slot(&mut not_choosable_slot);
        image
            .devices
            .get_mut(&device)
            .expect("池里有这块盘")
            .write(DeviceOffsetInBytes(8192), &not_choosable_slot);
    }
    wipe_the_slot_at_on_every_device(&mut image, 0);
    wipe_the_slot_at_on_every_device(&mut image, 4096);
    assert!(
        matches!(
            choose_system_configuration(&image),
            Err(
                RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized {
                    first_device_carrying_them: DeviceIdentity(0),
                    ..
                }
            )
        ),
        "恢复报 incompat 位图不认识、点名盘 0：{:?}",
        choose_system_configuration(&image).err()
    );
    assert_eq!(
        chosen_by_the_checker(&image),
        vec![(0, None), (1, None)],
        "checker 两块盘都择不到"
    );
    assert!(
        matches!(
            verdict_of(
                &image,
                SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
            ),
            InvariantVerdict::NotApplicable(_)
        ),
        "8192 处那一槽自证过、而一块盘都择不到：I-8.5 报不适用：{:?}",
        verdict_of(
            &image,
            SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
        )
    );
    let judged: Vec<(&str, InvariantVerdict)> = check_pool_image(&image)
        .into_iter()
        .filter(|(_, verdict)| !matches!(verdict, InvariantVerdict::NotApplicable(_)))
        .collect();
    assert_eq!(
        judged,
        Vec::new(),
        "一块盘都择不到系统配置：每一条不变量都报不适用"
    );
}

/// mkfs 清 [0, 根环基址)（W1）：一块盘先 mkfs 一次槽距 4096 的池、可写挂载一次（取号与两次发布各写一次系统配置，槽 1 世代号到 3），
/// 再拿同一个 fsid mkfs 一次 io_min 8192 的池，之后两块盘的槽 0 都抹掉。旧池 4096 处的槽 1 记着槽距 4096、世代号比新池的 1 大，
/// mkfs 不清它时逐档读的两个读者都会择中它；清了之后 4096 处是 0，恢复与 checker 都择到新池 8192 处的槽 1。
#[test]
fn remaking_the_filesystem_with_another_io_min_leaves_no_slot_one_of_the_earlier_pool_for_either_reader(
) {
    let mut devices = two_sparse_devices();
    let earlier_parameters =
        parameters_with_minimum_input_output_bytes(MINIMUM_INPUT_OUTPUT_BYTES_OF_A_4096_SPACING);
    make_filesystem(&earlier_parameters, &mut devices).expect("先 mkfs 一次槽距 4096 的池");
    mount_writable(&earlier_parameters, &mut devices).expect("旧池可写挂载一次");
    let earlier_image = image_of(&devices);
    let earlier_slot_one_generation = slot_at(&earlier_image, DeviceIdentity(0), 4096)
        .quantities
        .slot_generation;
    assert!(
        earlier_slot_one_generation > 1,
        "旧池 4096 处的槽 1 写过系统配置，世代号 {earlier_slot_one_generation} 比新池 mkfs 写下的 1 大"
    );

    make_filesystem(
        &parameters_with_minimum_input_output_bytes(MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING),
        &mut devices,
    )
    .expect("同一个 fsid 再 mkfs 一次 io_min 8192 的池");
    let mut image = image_of(&devices);
    wipe_the_slot_at_on_every_device(&mut image, 0);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let bytes_where_the_earlier_slot_one_was =
            PoolReader::read(&image, device, DeviceOffsetInBytes(4096), slot_bytes())
                .expect("4096 处读得到");
        assert!(
            bytes_where_the_earlier_slot_one_was
                .iter()
                .all(|byte| *byte == 0),
            "盘 {device:?}：mkfs 之后旧池槽 1 所在的 4096 处整槽是 0"
        );
    }
    assert_eq!(
        chosen_by_recovery(&image),
        (0, 8192, 1),
        "恢复择到新池 8192 处世代号 1 的槽 1，不是旧池 4096 处世代号 {earlier_slot_one_generation} 的那一份"
    );
    assert_eq!(
        chosen_by_the_checker(&image),
        vec![(0, Some((8192, 1))), (1, Some((8192, 1)))],
        "checker 与恢复择同一份：新池 8192 处世代号 1"
    );
}
