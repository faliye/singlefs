//! 取号时补写两槽全废的盘（丙'，C463（一块盘的系统配置两份全废之后静默成功）；用户 2026-10-19 定 S4「开例外，四件闸」，2026-10-20 再采
//! 「闸三加单元区扫描」「环槽读错就不补写」；三方判决 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第二节 U4 与第三节）。
//! 闸都过才补写：本盘带所选那一版的全部单元；根槽那一半不缺；这块盘根环与 journal 环里没有比所选那一版新的根与记录；
//! 单元区里没有写序实例大于所选那一版实例的单元；根环与 journal 环里没有读错的槽；补写失败按甲' 每块盘回卷一次（取号写那一份，
//! `acquisition_writes_two_rounds_and_rolls_each_device_back_once.rs`）。有一件不过照今天拒：取号之前的逐盘核报
//! `SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition`，盘上逐字节不变。
//!
//! 都在单元区 384 槽那一档的两块内存小盘上（`tests/common_admission_pool/mod.rs`），两槽全废的都是盘 1。所选那一版 V 多数用例是第一个文件之后
//! 覆盖写一次的 txg 4（根落区域 1 的盘 1）；空盘与别的池的盘那两条换掉整块盘 1，用覆盖写两次的 txg 5（根落区域 2 的盘 0），盘 1 换掉之后择根、
//! 重放与见证都不缺东西，拒才落在补写闸上。
mod common;
mod common_unwitnessed_publish;

use common::{
    with_unreadable_ranges, FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange,
    UnreadableRangeReadBack,
};
use common_unwitnessed_publish::{
    device_bytes_of_the_384_slot_small_devices, fixed_geometry_of_the_384_slot_small_devices,
    parameters_of_the_384_slot_small_devices, plain_device, plain_devices_on,
    record_one_publish_after_overwrites, PoolUnderTest, PublishRecordedAsR, OVERWRITE_BYTES,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::record_offset;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{
    mount_writable_with_space_admission, MountError, Mounted, SystemConfigurationUnreadableWhen,
};
use singlefs_core::recovery::verified_system_configuration_slots;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_format::{SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::memory_pool::{
    writes_and_segments, MemoryPool, SparseBlockDevice, SparseDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const DEVICE_LOSING_BOTH_SLOTS: DeviceIdentity = DeviceIdentity(1);
const VERSION: (InstanceGeneration, CheckpointTxg) = (InstanceGeneration(1), CheckpointTxg(4));
/// 覆盖写两次那一版：根落区域 2（盘 0），换掉整块盘 1 不动它。
const VERSION_WITH_ITS_ROOT_ON_DEVICE_ZERO: (InstanceGeneration, CheckpointTxg) =
    (InstanceGeneration(1), CheckpointTxg(5));

fn parameters() -> MakeFilesystemParameters {
    parameters_of_the_384_slot_small_devices()
}

/// 第一个文件之后覆盖写一次（V = txg 4，确认过）的池。
fn pool_at_the_version() -> PoolUnderTest<SparseBlockDevice> {
    pool_after_overwrites(1, VERSION)
}

/// 第一个文件之后覆盖写 `overwrites` 次的池，最新那一版是 `version`。
fn pool_after_overwrites(
    overwrites: u64,
    version: (InstanceGeneration, CheckpointTxg),
) -> PoolUnderTest<SparseBlockDevice> {
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters(),
        device_bytes_of_the_384_slot_small_devices(),
        plain_device,
    );
    for _overwrite in 0..overwrites {
        pool.overwrite(OVERWRITE_BYTES)
            .expect("384 槽小盘上第一个文件之后覆盖写几次落得下");
    }
    assert_eq!(
        (
            pool.session().current.root().instance,
            pool.session().current.root().checkpoint_txg
        ),
        version
    );
    pool
}

/// `image` 上盘 `device` 两个系统配置槽整槽写 0（两槽全废：读得出、自证不过）。
fn zero_both_system_configuration_slots(image: &mut MemoryPool, device: DeviceIdentity) {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let device_image = image.devices.get_mut(&device).expect("池里有这块盘");
    for slot_index in 0..2 {
        device_image.write(
            DeviceOffsetInBytes(slot_index * spacing),
            &vec![0u8; slot_bytes],
        );
    }
}

fn image_of(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    device_size_in_bytes: u64,
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes,
    }
}

/// 在 `image` 的拷贝上可写挂载一次：结局与挂载之后的盘面。
fn mount_writable_on(image: &MemoryPool) -> (Result<Mounted, MountError>, MemoryPool) {
    let mut devices = plain_devices_on(image);
    let outcome = mount_writable_with_space_admission(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    );
    (outcome, image_of(&devices, image.device_size_in_bytes))
}

/// 取号之前的逐盘核拒了两槽全废的盘 1（补写闸有一件不过），盘上逐字节不变。
fn assert_refused_before_the_acquisition_naming_device_one(
    outcome: Result<Mounted, MountError>,
    before: &MemoryPool,
    after: &MemoryPool,
) {
    let refusal = outcome.expect_err("补写闸有一件不过：拒可写");
    assert!(
        matches!(
            &refusal,
            MountError::SystemConfigurationUnreadableAfterTheReadStage(unreadable)
                if unreadable.unreadable
                    == SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
                        device: DEVICE_LOSING_BOTH_SLOTS,
                    }
        ),
        "拒在取号之前的逐盘核、点名盘 1，实际 {refusal:?}"
    );
    assert_eq!(after, before, "拒在任何写之前：两块盘逐字节不变");
}

fn violations_of(image: &MemoryPool) -> Vec<(&'static str, String)> {
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

/// 合格的盘：V 之后盘 1 两个系统配置槽整槽写 0，别的都在（V 的单元、根、记录）。可写挂载照常做成，取号写两轮把盘 1 两槽补写成新号 2；
/// 挂上之后的盘面池级 checker 一条都不红。判别力：取号不认补写的盘（变异「丙' 取号那一刻的见证读不认过了补写闸的盘（照旧拒）」）时
/// 取号拒成 `AcquisitionFindsADeviceWithoutAnySelfVerifiedSlot`，这一条红。
#[test]
fn a_device_holding_the_version_whose_both_slots_were_zeroed_is_rewritten_by_the_acquisition_and_the_checker_stays_green(
) {
    let mut image = pool_at_the_version().image();
    zero_both_system_configuration_slots(&mut image, DEVICE_LOSING_BOTH_SLOTS);
    let (outcome, after) = mount_writable_on(&image);
    let mounted = outcome.expect("盘 1 过了补写闸：可写挂载做成");
    assert_eq!(mounted.output.instance, InstanceGeneration(2));
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        VERSION,
        "建在 V 上"
    );
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let slots_of_device_one = verified_system_configuration_slots(
        &after,
        DEVICE_LOSING_BOTH_SLOTS,
        spacing,
        &parameters().filesystem_identifier,
    );
    assert_eq!(
        slots_of_device_one.len(),
        2,
        "盘 1 两槽都补写回来、自证得过"
    );
    assert!(
        slots_of_device_one
            .iter()
            .all(|slot| slot.quantities.journal_instance == InstanceGeneration(2)),
        "两槽都带这次挂载的号 2（取号写两轮，之后的轮换照写）：{slots_of_device_one:?}"
    );
    assert_eq!(
        violations_of(&after),
        Vec::new(),
        "补写之后挂上的盘面池级 checker 不红"
    );
}

/// 空盘：盘 1 换成一块一个字节都没写过的盘（所选那一版 txg 5 的根在盘 0）。闸一（本盘带所选那一版的全部单元）不过，拒。
/// 判别力：删掉闸一（变异「丙' 补写闸一不看本盘带不带所选那一版的单元」）时空盘过了别的几件、被补写、挂载往下走、盘上被写，这一条红。
#[test]
fn a_blank_device_is_not_rewritten_and_the_writable_mount_is_refused_before_any_write() {
    let mut image = pool_after_overwrites(2, VERSION_WITH_ITS_ROOT_ON_DEVICE_ZERO).image();
    image
        .devices
        .insert(DEVICE_LOSING_BOTH_SLOTS, SparseDevice::default());
    let (outcome, after) = mount_writable_on(&image);
    assert_refused_before_the_acquisition_naming_device_one(outcome, &image, &after);
}

/// 别的池的盘：盘 1 换成另一个池（fsid 不同、别的都同）第一个文件之后的盘 1（所选那一版 txg 5 的根在盘 0）。它两槽里没有一份是本池的，
/// 单元头的 fsid 也不是本池的：闸一不过，拒。
#[test]
fn a_device_of_another_pool_is_not_rewritten_and_the_writable_mount_is_refused_before_any_write() {
    let mut image = pool_after_overwrites(2, VERSION_WITH_ITS_ROOT_ON_DEVICE_ZERO).image();
    let mut parameters_of_another_pool = parameters();
    parameters_of_another_pool.filesystem_identifier[0] ^= 0xff;
    let another_pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_another_pool,
        device_bytes_of_the_384_slot_small_devices(),
        plain_device,
    )
    .image();
    image.devices.insert(
        DEVICE_LOSING_BOTH_SLOTS,
        another_pool.devices[&DEVICE_LOSING_BOTH_SLOTS].clone(),
    );
    let (outcome, after) = mount_writable_on(&image);
    let refusal = outcome.as_ref().expect_err("别的池的盘：拒可写");
    assert!(
        matches!(
            refusal,
            MountError::SystemConfigurationUnreadableAfterTheReadStage(unreadable)
                if unreadable.unreadable
                    == SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
                        device: DEVICE_LOSING_BOTH_SLOTS,
                    }
        ),
        "别的池的槽筛掉之后盘 1 两槽全废，过不了闸一，实际 {refusal:?}"
    );
    assert_eq!(after, image, "拒在任何写之前：两块盘逐字节不变");
}

/// 同 fsid 的克隆盘、克隆那边又发布过（带更新的根与记录）：V 之后会话再覆盖写一次（txg 5，克隆那边；它的根落区域 2，在克隆的盘 0 上），
/// 取那时的盘 1、两槽清零，与 V 那一刻的盘 0 拼成一池。所选那一版仍是 V；克隆盘的 journal 环里有 txg 5 的记录，比 V 新：补写闸三不过，
/// 取号之前拒、点名盘 1，补写不发生，盘上逐字节不变（第二轮攻方打中的那一形，闸三收掉它）。
#[test]
fn a_cloned_device_carrying_a_newer_root_and_record_is_not_rewritten_and_the_writable_mount_is_refused_before_any_write(
) {
    let mut pool = pool_at_the_version();
    let image_at_the_version = pool.image();
    pool.overwrite(OVERWRITE_BYTES)
        .expect("克隆那边再覆盖写一次落得下");
    let mut clone = pool.image();
    zero_both_system_configuration_slots(&mut clone, DEVICE_LOSING_BOTH_SLOTS);
    let mut image = image_at_the_version;
    image.devices.insert(
        DEVICE_LOSING_BOTH_SLOTS,
        clone.devices[&DEVICE_LOSING_BOTH_SLOTS].clone(),
    );
    let (outcome, after) = mount_writable_on(&image);
    assert_refused_before_the_acquisition_naming_device_one(outcome, &image, &after);
}

/// 闸三：盘 1 的 journal 环里有一条比 V 新的记录。V 之后会话录一次三条记录的顺序写 R（txg 5），只落了 R 的单元与盘 1 上 R 的第 1 条记录
/// （R 没有末条、重放不施加它，所选那一版仍是 V）；盘 1 两槽清零。判别力：删掉闸三（变异「丙' 补写闸三不看环里比所选那一版新的根与记录」）
/// 时盘 1 过了别的几件（R 的单元与 V 同一个实例）、被补写、挂载做成，这一条红。
#[test]
fn a_device_whose_journal_ring_holds_a_record_newer_than_the_version_is_not_rewritten() {
    let recorded =
        record_one_publish_after_overwrites(1, PublishRecordedAsR::SequentialWriteOfThreeDataUnits);
    assert_eq!(recorded.previous_key, VERSION, "R 之前那一版是 V");
    let mut image = recorded.base.clone();
    let unit_writes: Vec<_> = recorded.unit_writes().into_iter().cloned().collect();
    image.apply_writes(&unit_writes);
    let first_record_on_device_one: Vec<_> = recorded
        .record_writes_of_the_record(1)
        .into_iter()
        .filter(|write| write.device == DEVICE_LOSING_BOTH_SLOTS)
        .cloned()
        .collect();
    assert_eq!(
        first_record_on_device_one.len(),
        1,
        "R 的第 1 条记录盘 1 那一份"
    );
    image.apply_writes(&first_record_on_device_one);
    zero_both_system_configuration_slots(&mut image, DEVICE_LOSING_BOTH_SLOTS);
    let (outcome, after) = mount_writable_on(&image);
    assert_refused_before_the_acquisition_naming_device_one(outcome, &image, &after);
}

/// 闸四：同 fsid 的克隆盘、克隆那边可写挂载取号 2、只写了单元就崩（C555 那一形挂在克隆盘上）。V 那一刻的盘面拷一份当克隆，在它上面可写挂载一次、
/// 录下写流，取写行那次发布的单元落了、记录还没落的那一刻的盘 1，两槽清零（取号写两轮写的号 2 也一起没了），与 V 那一刻的盘 0 拼成一池。
/// 盘 1 的根环与 journal 环里没有比 V 新的，单元区里有号 2 写下的单元：闸四不过，拒。判别力：删掉闸四（变异「丙' 补写闸四不扫单元区的写序」）时
/// 盘 1 被补写、这一池接着取号 2，与克隆盘上号 2 的单元撞号，挂载做成，这一条红。
#[test]
fn a_cloned_device_holding_units_written_by_a_later_instance_is_not_rewritten() {
    let image_at_the_version = pool_at_the_version().image();
    let stream = SharedStream::retaining_contents();
    let mut clone_devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        plain_devices_on(&image_at_the_version)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
                )
            })
            .collect();
    let clone_mount = mount_writable_with_space_admission(
        &parameters(),
        &mut clone_devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("克隆那边可写挂载做得成");
    assert_eq!(
        clone_mount.output.instance,
        InstanceGeneration(2),
        "克隆那边取号 2"
    );
    let (writes, _segments) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let first_record = writes
        .iter()
        .position(|write| write.kind == StepKind::JournalRecord)
        .expect("写行那次发布写了记录");
    let mut clone = image_at_the_version.clone();
    clone.apply_writes(&writes[..first_record]);
    zero_both_system_configuration_slots(&mut clone, DEVICE_LOSING_BOTH_SLOTS);
    let mut image = image_at_the_version;
    image.devices.insert(
        DEVICE_LOSING_BOTH_SLOTS,
        clone.devices[&DEVICE_LOSING_BOTH_SLOTS].clone(),
    );
    let (outcome, after) = mount_writable_on(&image);
    assert_refused_before_the_acquisition_naming_device_one(outcome, &image, &after);
}

/// 闸五：合格的盘之外，盘 1 上 V 那条记录所在的 journal 环槽读一次报一次设备错（那一槽在盘 0 上那一份照读）。读错的槽里可能藏着更新的记录：
/// 不补写，拒。判别力：删掉闸五（变异「丙' 补写闸五不看环槽读错」）时盘 1 被补写、挂载做成，这一条红。
#[test]
fn a_device_with_a_journal_ring_slot_reporting_a_read_error_is_not_rewritten() {
    let pool = pool_at_the_version();
    let record_counter_of_the_version = pool.current_file_version().record.counter;
    let mut image = pool.image();
    zero_both_system_configuration_slots(&mut image, DEVICE_LOSING_BOTH_SLOTS);
    let record_slot = record_offset(
        record_counter_of_the_version,
        parameters().geometry.journal_ring_bytes,
    );
    let ranges = SharedUnreadableRanges::new(
        vec![UnreadableRange {
            device: DEVICE_LOSING_BOTH_SLOTS,
            offset_in_bytes: record_slot.0,
            length_in_bytes: singlefs_format::JOURNAL_RECORD_BYTES,
            failing_reads: FailingReadsOfARange::Every,
        }],
        UnreadableRangeReadBack::DeviceError,
    );
    let mut devices = with_unreadable_ranges(plain_devices_on(&image), &ranges);
    let refusal = mount_writable_with_space_admission(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect_err("盘 1 的 journal 环有一槽读错：拒可写");
    assert!(ranges.failed_reads() > 0, "那一槽真被读过、报了设备错");
    assert!(
        matches!(
            &refusal,
            MountError::SystemConfigurationUnreadableAfterTheReadStage(unreadable)
                if unreadable.unreadable
                    == SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
                        device: DEVICE_LOSING_BOTH_SLOTS,
                    }
        ),
        "拒在取号之前的逐盘核、点名盘 1，实际 {refusal:?}"
    );
    let after = MemoryPool {
        devices: common::without_unreadable_ranges(devices)
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    };
    assert_eq!(after, image, "拒在任何写之前：两块盘逐字节不变");
}

/// 盘 1 读到它单元区最末一槽之后，V 的根槽（落在盘 1）读回全零：只有补写闸四扫单元区会读到那一槽，闸二（根槽那一半）排在它之后，
/// 读的是全零。读阶段择根时那一槽照读。
struct DeviceZeroingTheVersionRootSlotAfterItsUnitAreaEndIsRead {
    inner: SparseBlockDevice,
    zeroes_the_version_root_slot: bool,
    root_slot_offset: u64,
    root_slot_bytes: u64,
    last_unit_area_slot_offset: u64,
    is_triggered: std::cell::Cell<bool>,
}

impl BlockDevice for DeviceZeroingTheVersionRootSlotAfterItsUnitAreaEndIsRead {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        if !self.zeroes_the_version_root_slot {
            return Ok(());
        }
        let read_end = offset.0 + u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        if offset.0 <= self.last_unit_area_slot_offset && self.last_unit_area_slot_offset < read_end
        {
            self.is_triggered.set(true);
        }
        let root_slot_end = self.root_slot_offset + self.root_slot_bytes;
        if self.is_triggered.get() && offset.0 < root_slot_end && self.root_slot_offset < read_end {
            let start_in_buffer = self.root_slot_offset.saturating_sub(offset.0);
            let end_in_buffer = (root_slot_end - offset.0).min(read_end - offset.0);
            buffer[usize::try_from(start_in_buffer).expect("落在缓冲区里")
                ..usize::try_from(end_in_buffer).expect("落在缓冲区里")]
                .fill(0);
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
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

/// 闸二：合格的盘之外，V 的根槽（落在盘 1）在补写闸扫完单元区之后读回全零（只在补写闸里读，读阶段择到 V 的根照常）。根槽那一半缺：
/// 读一次、重读一次都不是 V，拒。判别力：删掉闸二（变异「丙' 补写闸二不看根槽那一半」）时盘 1 被补写、挂载往下走，盘上被写，这一条红。
#[test]
fn a_device_whose_ring_slot_of_the_version_root_no_longer_holds_it_is_not_rewritten() {
    let mut image = pool_at_the_version().image();
    zero_both_system_configuration_slots(&mut image, DEVICE_LOSING_BOTH_SLOTS);
    let geometry = parameters().geometry;
    let root_slot_offset = slot_offset(
        target_for_publish(VERSION.1, geometry.root_ring_slots_per_region),
        geometry.fixed_structure_slot_spacing,
    );
    let device_bytes = image.device_size_in_bytes;
    let mut devices: Vec<(
        DeviceIdentity,
        DeviceZeroingTheVersionRootSlotAfterItsUnitAreaEndIsRead,
    )> = plain_devices_on(&image)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                DeviceZeroingTheVersionRootSlotAfterItsUnitAreaEndIsRead {
                    inner,
                    zeroes_the_version_root_slot: identity == DEVICE_LOSING_BOTH_SLOTS,
                    root_slot_offset: root_slot_offset.0,
                    root_slot_bytes: u64::from(geometry.physical_block_size),
                    last_unit_area_slot_offset: (device_bytes / SLOT_BYTES - 1) * SLOT_BYTES,
                    is_triggered: std::cell::Cell::new(false),
                },
            )
        })
        .collect();
    let refusal = mount_writable_with_space_admission(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect_err("V 的根槽在补写闸里读不出 V：拒可写");
    assert!(
        devices[1].1.is_triggered.get(),
        "补写闸扫到了盘 1 单元区最末一槽"
    );
    assert!(
        matches!(
            &refusal,
            MountError::SystemConfigurationUnreadableAfterTheReadStage(unreadable)
                if unreadable.unreadable
                    == SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
                        device: DEVICE_LOSING_BOTH_SLOTS,
                    }
        ),
        "拒在取号之前的逐盘核、点名盘 1，实际 {refusal:?}"
    );
    let after = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner.image.clone()))
            .collect(),
        device_size_in_bytes: device_bytes,
    };
    assert_eq!(after, image, "拒在任何写之前：两块盘逐字节不变");
}
