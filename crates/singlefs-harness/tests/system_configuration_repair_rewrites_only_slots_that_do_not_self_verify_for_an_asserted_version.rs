//! 离线出路（α'，`singlefs_core::system_configuration_repair`；C596（见证判不出的系统配置盘面只剩只读挂载）；用户 2026-10-19 定 S1「许，另加扫单元写序」，
//! 2026-10-20 再采「离线出路加 birth_txg 闸」；三方判决 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第二节 U2 与第三节）。
//!
//! 第三轮攻方 U2 那张表（`research/prompts/m3-close-sysconf-r3-opus-output.md`「## U2」）的九格各一条：起点 k2（单元区 384 槽那一档的两块内存小盘，
//! 第一个文件之后覆盖写两次，最新版 T = txg 5、根在盘 0），每块盘最新那一个系统配置槽读错、写过之后读得出（R2 判不出的那一形）；
//! 三种盘面：T 的根读得出 / T 的根槽读错、T 的记录读得出 / T 的根槽与 T 的记录（两块盘各一份）都读错；调用方断言 T、T−1、T+1。
//! 另一条是 C555（两盘最新系统配置槽都坏时取号撞号）那一形：取号写两轮落了、写行那次发布一个单元都没落就崩，两块盘最新那一槽读错。
mod common_admission_pool;

use common_admission_pool::{
    device_bytes_of_the_384_slot_small_devices, fixed_geometry_of_the_384_slot_small_devices,
    parameters_of_the_384_slot_small_devices, plain_device, plain_devices_on, PoolUnderTest,
    OVERWRITE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::record_offset;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::mount_writable_with_space_admission;
use singlefs_core::recovery::verified_system_configuration_slots;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::system_configuration_repair::{
    rewrite_system_configuration_slots_for_the_asserted_version, AssertedVersion,
    SystemConfigurationRepairRefused, SystemConfigurationSlotRewritten,
};
use singlefs_format::{JOURNAL_RECORD_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
const INSTANCE_OF_THE_POOL: InstanceGeneration = InstanceGeneration(1);
/// 起点 k2 最新的那一版：第一个文件 txg 3，覆盖写两次 txg 4、5。
const NEWEST_VERSION_TXG: CheckpointTxg = CheckpointTxg(5);

fn parameters() -> MakeFilesystemParameters {
    parameters_of_the_384_slot_small_devices()
}

fn slot_spacing_in_bytes() -> u64 {
    u64::from(parameters().geometry.fixed_structure_slot_spacing)
}

/// 一段读错：从哪个字节起、多长，写过之后还错不错。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReadErrorRange {
    offset_in_bytes: u64,
    length_in_bytes: u64,
    lasts: ReadErrorLasts,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadErrorLasts {
    /// 有一次写落在它上面之后读得出（离线出路重写了那一槽）。
    UntilWrittenOver,
    /// 一直读错。
    Always,
}

impl ReadErrorRange {
    fn overlaps(&self, offset_in_bytes: u64, length_in_bytes: u64) -> bool {
        offset_in_bytes < self.offset_in_bytes + self.length_in_bytes
            && self.offset_in_bytes < offset_in_bytes + length_in_bytes
    }
}

/// 读到点名的几段就报设备错的盘；写落在「写过之后读得出」的那一段上，那一段就撤掉。写、写零、屏障原样交给里面那块盘。
struct DeviceWithReadErrors {
    inner: SparseBlockDevice,
    ranges: std::cell::RefCell<Vec<ReadErrorRange>>,
}

impl DeviceWithReadErrors {
    fn clear_the_ranges_written_over(&self, offset_in_bytes: u64, length_in_bytes: u64) {
        self.ranges.borrow_mut().retain(|range| {
            range.lasts == ReadErrorLasts::Always
                || !range.overlaps(offset_in_bytes, length_in_bytes)
        });
    }
}

impl BlockDevice for DeviceWithReadErrors {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        if self
            .ranges
            .borrow()
            .iter()
            .any(|range| range.overlaps(offset.0, length_in_bytes))
        {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "用例点名的这一段读错",
            )));
        }
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)?;
        self.clear_the_ranges_written_over(
            offset.0,
            u64::try_from(bytes.len()).expect("一次写的长度装得进 u64"),
        );
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)?;
        self.clear_the_ranges_written_over(offset.0, length);
        Ok(())
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 一块盘两槽里世代号最大的那一份住在哪个偏移。
fn newest_system_configuration_slot_offset(image: &MemoryPool, device: DeviceIdentity) -> u64 {
    let newest_generation = verified_system_configuration_slots(
        image,
        device,
        slot_spacing_in_bytes(),
        &parameters().filesystem_identifier,
    )
    .iter()
    .map(|slot| slot.quantities.slot_generation)
    .max()
    .expect("这块盘有自证过的系统配置槽");
    (newest_generation % 2) * slot_spacing_in_bytes()
}

fn system_configuration_slot_range(offset_in_bytes: u64) -> ReadErrorRange {
    ReadErrorRange {
        offset_in_bytes,
        length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
        lasts: ReadErrorLasts::UntilWrittenOver,
    }
}

/// 每块盘最新那一槽读错、写过之后读得出，`more` 里另加的几段（每段带它在哪块盘上）。
fn devices_with_read_errors(
    image: &MemoryPool,
    more: &[(DeviceIdentity, ReadErrorRange)],
) -> Vec<(DeviceIdentity, DeviceWithReadErrors)> {
    plain_devices_on(image)
        .into_iter()
        .map(|(identity, inner)| {
            let mut ranges = vec![system_configuration_slot_range(
                newest_system_configuration_slot_offset(image, identity),
            )];
            ranges.extend(
                more.iter()
                    .filter(|(device, _)| *device == identity)
                    .map(|(_, range)| *range),
            );
            (
                identity,
                DeviceWithReadErrors {
                    inner,
                    ranges: std::cell::RefCell::new(ranges),
                },
            )
        })
        .collect()
}

fn image_of(
    devices: &[(DeviceIdentity, DeviceWithReadErrors)],
    device_size_in_bytes: u64,
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner.image.clone()))
            .collect(),
        device_size_in_bytes,
    }
}

/// T 那一版在盘面上读得出多少（第三轮攻方 U2 的三种盘面）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FaceOfTheNewestVersion {
    RootReadable,
    RecordReadableWithTheRootUnreadable,
    NeitherRootNorRecordReadable,
}

/// 起点 k2 的盘面、T 那条记录的计数器，与这一种盘面另加的读错（每块盘最新那一槽之外的）。
struct NewestVersionFace {
    image: MemoryPool,
    record_counter_of_the_newest_version: u64,
    more_read_errors: Vec<(DeviceIdentity, ReadErrorRange)>,
}

fn newest_version_face(face: FaceOfTheNewestVersion) -> NewestVersionFace {
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters(),
        device_bytes_of_the_384_slot_small_devices(),
        plain_device,
    );
    for _overwrite in 0..2 {
        pool.overwrite(OVERWRITE_BYTES)
            .expect("384 槽小盘上第一个文件之后覆盖写两次落得下");
    }
    let newest = pool.current_file_version();
    assert_eq!(
        (newest.root.instance, newest.root.checkpoint_txg),
        (INSTANCE_OF_THE_POOL, NEWEST_VERSION_TXG)
    );
    let record_counter_of_the_newest_version = newest.record.counter;
    let geometry = parameters().geometry;
    let root_slot = ReadErrorRange {
        offset_in_bytes: slot_offset(
            target_for_publish(NEWEST_VERSION_TXG, geometry.root_ring_slots_per_region),
            geometry.fixed_structure_slot_spacing,
        )
        .0,
        length_in_bytes: u64::from(geometry.physical_block_size),
        lasts: ReadErrorLasts::Always,
    };
    let record_slot = ReadErrorRange {
        offset_in_bytes: record_offset(
            record_counter_of_the_newest_version,
            geometry.journal_ring_bytes,
        )
        .0,
        length_in_bytes: JOURNAL_RECORD_BYTES,
        lasts: ReadErrorLasts::Always,
    };
    let device_of_the_newest_root = parameters().region_devices[2];
    let more_read_errors = match face {
        FaceOfTheNewestVersion::RootReadable => Vec::new(),
        FaceOfTheNewestVersion::RecordReadableWithTheRootUnreadable => {
            vec![(device_of_the_newest_root, root_slot)]
        }
        FaceOfTheNewestVersion::NeitherRootNorRecordReadable => vec![
            (device_of_the_newest_root, root_slot),
            (DeviceIdentity(0), record_slot),
            (DeviceIdentity(1), record_slot),
        ],
    };
    NewestVersionFace {
        image: pool.image(),
        record_counter_of_the_newest_version,
        more_read_errors,
    }
}

fn asserted(checkpoint_txg: u64) -> AssertedVersion {
    AssertedVersion {
        instance: INSTANCE_OF_THE_POOL,
        checkpoint_txg: CheckpointTxg(checkpoint_txg),
    }
}

/// 照写的那两格：每块盘最新那一槽（读错的那一槽）各重写一次、别的槽一个字节不动，写的实例代号是 1、tail 是 T 那条记录的计数器；
/// 之后带着剩下的读故障可写挂载一次，建在 T 上。
fn assert_rewritten_and_the_writable_mount_lands_on_the_newest_version(
    face: FaceOfTheNewestVersion,
) {
    let NewestVersionFace {
        image,
        record_counter_of_the_newest_version,
        more_read_errors,
    } = newest_version_face(face);
    let mut devices = devices_with_read_errors(&image, &more_read_errors);
    let repaired = rewrite_system_configuration_slots_for_the_asserted_version(
        &parameters(),
        &mut devices,
        asserted(NEWEST_VERSION_TXG.0),
    )
    .expect("断言 T、T 读得出：照写");
    let expected_rewritten: Vec<SystemConfigurationSlotRewritten> = DEVICES
        .iter()
        .map(|device| SystemConfigurationSlotRewritten {
            device: *device,
            slot_index: newest_system_configuration_slot_offset(&image, *device)
                / slot_spacing_in_bytes(),
        })
        .collect();
    assert_eq!(
        repaired.slots_rewritten, expected_rewritten,
        "只重写每块盘不自证的那一槽（最新那一槽）"
    );
    assert_eq!(
        repaired.journal_instance_written, INSTANCE_OF_THE_POOL,
        "实例代号取盘上的最大值、不加一"
    );
    assert_eq!(
        repaired.journal_tail_written, record_counter_of_the_newest_version,
        "tail 是 T 那条末条记录的计数器"
    );
    let after = image_of(&devices, image.device_size_in_bytes);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for device in DEVICES {
        let newest_offset = newest_system_configuration_slot_offset(&image, device);
        let other_offset = slot_spacing_in_bytes() - newest_offset;
        assert_eq!(
            after.devices[&device].read(DeviceOffsetInBytes(other_offset), slot_bytes),
            image.devices[&device].read(DeviceOffsetInBytes(other_offset), slot_bytes),
            "{device:?}：自证得过的那一槽一个字节不动"
        );
    }
    let mounted = mount_writable_with_space_admission(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("离线出路写过之后可写挂载做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        (INSTANCE_OF_THE_POOL, NEWEST_VERSION_TXG),
        "建在 T 上，确认过的 txg 5 不丢"
    );
}

/// 拒的那几格：交回 `expected` 那一种拒绝，两块盘逐字节不变。
fn assert_refused_without_writing(
    face: FaceOfTheNewestVersion,
    asserted_txg: u64,
    is_the_expected_refusal: impl Fn(&SystemConfigurationRepairRefused) -> bool,
) {
    let NewestVersionFace {
        image,
        more_read_errors,
        ..
    } = newest_version_face(face);
    let mut devices = devices_with_read_errors(&image, &more_read_errors);
    let refusal = rewrite_system_configuration_slots_for_the_asserted_version(
        &parameters(),
        &mut devices,
        asserted(asserted_txg),
    )
    .expect_err("这一格不该写");
    assert!(
        is_the_expected_refusal(&refusal),
        "拒的理由不对：{refusal:?}"
    );
    assert_eq!(
        image_of(&devices, image.device_size_in_bytes),
        image,
        "一个字节不写"
    );
}

fn is_neither_readable(refusal: &SystemConfigurationRepairRefused) -> bool {
    matches!(
        refusal,
        SystemConfigurationRepairRefused::AssertedVersionReadableNeitherInTheRootRingNorInTheJournalRing { .. }
    )
}

fn is_a_unit_born_after_the_asserted_version(refusal: &SystemConfigurationRepairRefused) -> bool {
    matches!(
        refusal,
        SystemConfigurationRepairRefused::UnitBornAfterTheAssertedVersion {
            highest_birth_txg,
            ..
        } if *highest_birth_txg == NEWEST_VERSION_TXG
    )
}

/// 断言 T、T 的根读得出：照写，可写挂载建在 T 上。前提另核一次：写之前可写挂载拒（R2 判不出）。
/// 判别力：只重写不自证的槽改成两槽都重写（变异「α' 两槽都重写（自证得过的那一槽也被覆写）」）时自证那一槽被覆写、重写的槽数是 4，这一条红。
#[test]
fn asserting_the_newest_version_whose_root_is_readable_rewrites_the_unreadable_slots_and_mounts_on_it(
) {
    let NewestVersionFace { image, .. } = newest_version_face(FaceOfTheNewestVersion::RootReadable);
    let mut devices = devices_with_read_errors(&image, &[]);
    mount_writable_with_space_admission(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect_err("前提：每块盘最新那一槽读错，系统配置见证判不出（R2），可写挂载拒");
    assert_rewritten_and_the_writable_mount_lands_on_the_newest_version(
        FaceOfTheNewestVersion::RootReadable,
    );
}

/// 断言 T、T 的根槽读错而 T 的记录读得出：凭记录照写，可写挂载由记录重建出 T。
#[test]
fn asserting_the_newest_version_whose_record_is_readable_rewrites_the_unreadable_slots_and_mounts_on_it(
) {
    assert_rewritten_and_the_writable_mount_lands_on_the_newest_version(
        FaceOfTheNewestVersion::RecordReadableWithTheRootUnreadable,
    );
}

/// 断言 T、T 的根与记录都读不出：查不到，一个字节不写。
#[test]
fn asserting_the_newest_version_whose_root_and_record_are_unreadable_writes_nothing() {
    assert_refused_without_writing(
        FaceOfTheNewestVersion::NeitherRootNorRecordReadable,
        NEWEST_VERSION_TXG.0,
        is_neither_readable,
    );
}

/// 断言 T−1、T 的根读得出：T−1 的根与记录都读得出，但单元区里有 T 写下的单元（birth_txg 5 > 4），一个字节不写。
/// 判别力：删掉 birth_txg 闸（变异「α' 不看单元区 birth_txg 的最大值（断言旧一版照写）」）时照写，这一条红。
#[test]
fn asserting_the_version_before_the_newest_while_its_root_is_readable_writes_nothing() {
    assert_refused_without_writing(
        FaceOfTheNewestVersion::RootReadable,
        NEWEST_VERSION_TXG.0 - 1,
        is_a_unit_born_after_the_asserted_version,
    );
}

/// 断言 T−1、T 的根读不出而记录读得出：同上，birth_txg 闸拒。
#[test]
fn asserting_the_version_before_the_newest_while_only_its_record_is_readable_writes_nothing() {
    assert_refused_without_writing(
        FaceOfTheNewestVersion::RecordReadableWithTheRootUnreadable,
        NEWEST_VERSION_TXG.0 - 1,
        is_a_unit_born_after_the_asserted_version,
    );
}

/// 断言 T−1、T 的根与记录都读不出：第三轮攻方打中的那一格（照写会丢已确认的 txg 5）。birth_txg 闸拒，一个字节不写，撤掉读故障之后仍能回到 T。
#[test]
fn asserting_the_version_before_the_newest_while_neither_is_readable_writes_nothing() {
    assert_refused_without_writing(
        FaceOfTheNewestVersion::NeitherRootNorRecordReadable,
        NEWEST_VERSION_TXG.0 - 1,
        is_a_unit_born_after_the_asserted_version,
    );
}

/// 断言 T+1、T 的根读得出：盘上没有 T+1，一个字节不写。判别力：删掉「查不到就不写」那一判（变异「α' 断言那一版查不到也照写」）时照写，
/// 这一条红。
#[test]
fn asserting_a_version_after_the_newest_while_its_root_is_readable_writes_nothing() {
    assert_refused_without_writing(
        FaceOfTheNewestVersion::RootReadable,
        NEWEST_VERSION_TXG.0 + 1,
        is_neither_readable,
    );
}

/// 断言 T+1、T 的根读不出而记录读得出：同上。
#[test]
fn asserting_a_version_after_the_newest_while_only_the_record_is_readable_writes_nothing() {
    assert_refused_without_writing(
        FaceOfTheNewestVersion::RecordReadableWithTheRootUnreadable,
        NEWEST_VERSION_TXG.0 + 1,
        is_neither_readable,
    );
}

/// 断言 T+1、T 的根与记录都读不出：同上。
#[test]
fn asserting_a_version_after_the_newest_while_neither_is_readable_writes_nothing() {
    assert_refused_without_writing(
        FaceOfTheNewestVersion::NeitherRootNorRecordReadable,
        NEWEST_VERSION_TXG.0 + 1,
        is_neither_readable,
    );
}

/// C555 那一形写完之后再可写挂载不撞号：第一个文件之后覆盖写一次（T = txg 4），可写挂载取号 2、取号写两轮落了、写行那次发布一个写都没落就崩；
/// 两块盘最新那一槽（取号第二轮写的）读错、写过之后读得出（判不出）。断言 T：离线出路把两槽读错的那一槽重写，写的实例代号取各盘自证槽里的号 2；
/// 之后可写挂载取 3。判别力：实例代号不并各盘自证槽（变异「α' 写的实例代号不取各盘自证槽里的最大值」）时写成 1，这一条红。
#[test]
fn after_an_acquisition_crash_with_both_newest_slots_unreadable_the_rewrite_keeps_the_acquired_instance_and_the_next_mount_takes_the_one_after(
) {
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters(),
        device_bytes_of_the_384_slot_small_devices(),
        plain_device,
    );
    pool.overwrite(OVERWRITE_BYTES)
        .expect("384 槽小盘上第一个文件之后覆盖写一次落得下");
    let asserted_version_txg = pool.current_file_version().root.checkpoint_txg;
    let base = pool.image();
    let stream = SharedStream::retaining_contents();
    let mut recorded_devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        plain_devices_on(&base)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
                )
            })
            .collect();
    let crashed_mount = mount_writable_with_space_admission(
        &parameters(),
        &mut recorded_devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("覆盖写之后的盘面可写挂载做得成");
    assert_eq!(crashed_mount.output.instance, InstanceGeneration(2));
    let (writes, _segments) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let first_write_after_the_acquisition = writes
        .iter()
        .position(|write| write.kind != StepKind::SystemConfigurationSlot)
        .expect("取号之后写了别的");
    assert_eq!(
        first_write_after_the_acquisition, 4,
        "取号写两轮、两块盘各一写一轮：四次系统配置写"
    );
    let mut crash_image = base.clone();
    crash_image.apply_writes(&writes[..first_write_after_the_acquisition]);
    let mut devices = devices_with_read_errors(&crash_image, &[]);
    mount_writable_with_space_admission(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect_err("前提：两块盘最新那一槽读错、判不出，可写挂载拒");
    let repaired = rewrite_system_configuration_slots_for_the_asserted_version(
        &parameters(),
        &mut devices,
        AssertedVersion {
            instance: INSTANCE_OF_THE_POOL,
            checkpoint_txg: asserted_version_txg,
        },
    )
    .expect("断言 T、T 的根读得出、单元区里没有比 T 晚的单元：照写");
    assert_eq!(
        repaired.journal_instance_written,
        InstanceGeneration(2),
        "实例代号取各盘自证槽里取号第一轮写的号 2"
    );
    let mounted = mount_writable_with_space_admission(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("离线出路写过之后可写挂载做得成");
    assert_eq!(
        mounted.output.instance,
        InstanceGeneration(3),
        "号 2 在盘上，这一次取 3，不撞号"
    );
}
