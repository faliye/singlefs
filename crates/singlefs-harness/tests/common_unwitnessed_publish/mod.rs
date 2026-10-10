//! C694、C695 那一簇（删除线第 33 批 T）几份 harness 档用例共用的搭建：单元区 384 槽那一档小盘上第一个文件之后在会话里覆盖写 k 次，
//! 再经会话录一次发布 R（覆盖写一次，或顺序写三个数据单元、切成三条记录），崩在 R 的根槽落盘之后、它那次系统配置轮换之前
//! （系统配置没见证 R，D16（发布语义） 已定项 7 的持久顺序）。之后的一次可写挂载里让 R 的根槽、R 写出的单元、R 的某几条记录读回全零
//! （只这一次挂载：写到哪一段，那一段就不再藏），即攻方 `research/prompts/m3-c694-r2-opus-output.md` 第一、二、五节的 `ur` / `urM` / `urF` /
//! `urL` / `urJ` 那几形，在入库装置上重造。池本身（mkfs、取号、暖机、第一个文件、会话）在 `tests/common_admission_pool/mod.rs`，这里全部转出去。
#![allow(dead_code, reason = "几个测试文件各自只用到其中一部分")]

#[path = "../common_admission_pool/mod.rs"]
mod common_admission_pool;

pub use common_admission_pool::*;

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::{record_offset, JournalRecordPlaceInPublish};
use singlefs_core::mount::Mounted;
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, scan_journal,
    RootRingSlotReading,
};
use singlefs_core::root_ring::slot_offset;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 录的那一次发布 R 是哪一种。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublishRecordedAsR {
    /// 覆盖写一次（一个数据单元，一条记录）：第一轮攻方 K1-2 那一形的 R。
    Overwrite,
    /// 顺序写三个数据单元（每个数据单元一个事务、一条记录，末条另点名全部元数据单元）：第二轮攻方第二节那一形的 R。
    SequentialWriteOfThreeDataUnits,
}

/// 顺序写三个数据单元的内容长度：两个数据单元的净荷再加一千字节。
fn content_bytes_of_three_data_units() -> usize {
    2 * singlefs_core::unit::data_unit_payload_capacity() + 1000
}

/// 录下来的 R：R 之前那一刻的盘面（`base`）、R 那次发布的全部写（`writes`，按落盘次序）、R 的根槽写在写表里的下标、R 的 (实例, txg)
/// 与它带的文件内容，加起池那一份（参数、盘宽、每条根带的文件内容；会话停在 R 那一版）。
pub struct RecordedPublish {
    pub pool: PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>,
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub root_index: usize,
    pub key: (InstanceGeneration, CheckpointTxg),
    pub content: Vec<u8>,
    /// R 之前的最后一版（P）的 (实例, txg) 与文件内容。
    pub previous_key: (InstanceGeneration, CheckpointTxg),
    pub previous_content: Vec<u8>,
}

/// 起池（384 槽那一档、第一个文件），会话里覆盖写 `overwrites_before` 次，再经会话录一次 `publish`。
///
/// # Panics
/// 起池、覆盖写或录的那次发布被拒（这几档参数下都落得下）；录的那次发布不是一条根。
pub fn record_one_publish_after_overwrites(
    overwrites_before: u64,
    publish: PublishRecordedAsR,
) -> RecordedPublish {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| {
            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
        },
    );
    for _overwrite in 0..overwrites_before {
        pool.overwrite(OVERWRITE_BYTES)
            .expect("384 槽小盘上第一个文件之后覆盖写这几次都落得下");
    }
    let previous_root = *pool.session().current.root();
    let previous_content = pool.content_of_the_current_version.clone();
    let base = pool.image();
    let operations_before_r = stream.operation_count();
    match publish {
        PublishRecordedAsR::Overwrite => pool.overwrite(OVERWRITE_BYTES),
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits => {
            pool.sequential_write(content_bytes_of_three_data_units())
        }
    }
    .expect("录的那次发布落得下");
    let root = *pool.session().current.root();
    let operations = stream.retained_operations();
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_r..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let root_indices: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        root_indices.len(),
        1,
        "录的那一次发布只写一条根（准入够，没推抬 F）"
    );
    RecordedPublish {
        content: pool.content_of_the_current_version.clone(),
        pool,
        base,
        writes,
        root_index: root_indices[0],
        key: (root.instance, root.checkpoint_txg),
        previous_key: (previous_root.instance, previous_root.checkpoint_txg),
        previous_content,
    }
}

impl RecordedPublish {
    /// 崩在 R 的根槽落盘之后、系统配置轮换之前：R 的单元、记录与根槽都落了，轮换那两槽没落（写表里根槽之后的都不施加）。
    #[must_use]
    pub fn crash_image_with_the_root_persisted_and_the_rotation_lost(&self) -> MemoryPool {
        let mut image = self.base.clone();
        image.apply_writes(&self.writes[..=self.root_index]);
        image
    }

    /// R 的整次发布都落了（系统配置轮换与末尾屏障都过了：R 已确认）。
    #[must_use]
    pub fn image_with_the_whole_publish_persisted(&self) -> MemoryPool {
        let mut image = self.base.clone();
        image.apply_writes(&self.writes);
        image
    }

    /// R 写出的单元写（每块盘一份）。
    #[must_use]
    pub fn unit_writes(&self) -> Vec<&RetainedWrite> {
        self.writes
            .iter()
            .filter(|write| write.kind == StepKind::UnitWrite)
            .collect()
    }

    /// R 的第 `ordinal_from_one` 条记录的两份（两块盘同一个偏移）：记录写按落盘次序、按偏移归条，第一个偏移是第 1 条。
    ///
    /// # Panics
    /// R 没有那么多条记录。
    #[must_use]
    pub fn record_writes_of_the_record(&self, ordinal_from_one: usize) -> Vec<&RetainedWrite> {
        let mut offsets: Vec<DeviceOffsetInBytes> = Vec::new();
        for write in self
            .writes
            .iter()
            .filter(|write| write.kind == StepKind::JournalRecord)
        {
            if !offsets.contains(&write.offset) {
                offsets.push(write.offset);
            }
        }
        let offset = offsets[ordinal_from_one - 1];
        self.writes
            .iter()
            .filter(|write| write.kind == StepKind::JournalRecord && write.offset == offset)
            .collect()
    }

    /// R 一共几条记录。
    #[must_use]
    pub fn record_count(&self) -> usize {
        let mut offsets: Vec<DeviceOffsetInBytes> = Vec::new();
        for write in self
            .writes
            .iter()
            .filter(|write| write.kind == StepKind::JournalRecord)
        {
            if !offsets.contains(&write.offset) {
                offsets.push(write.offset);
            }
        }
        offsets.len()
    }

    /// R 的根槽写。
    #[must_use]
    pub fn root_slot_write(&self) -> &RetainedWrite {
        &self.writes[self.root_index]
    }
}

/// 一次挂载里读回全零的那几段（每段：盘、偏移、长度）。
#[derive(Clone, Debug, Default)]
pub struct RangesReadingBackAllZero(pub Vec<(DeviceIdentity, u64, u64)>);

impl RangesReadingBackAllZero {
    /// 加上这几次写各自落的那一段。
    #[must_use]
    pub fn with_the_writes(mut self, writes: &[&RetainedWrite]) -> Self {
        self.0.extend(
            writes
                .iter()
                .map(|write| (write.device, write.offset.0, write.length_in_bytes())),
        );
        self
    }

    /// 加上 `image` 上 (实例, txg) 为 `key` 的那条根所在的根槽（根环里自证过、正是这条根的那一槽，一块盘一份）。
    ///
    /// # Panics
    /// `image` 上择不到系统配置，或根环里没有自证过的这条根。
    #[must_use]
    pub fn with_the_root_slot_of(
        mut self,
        image: &MemoryPool,
        key: (InstanceGeneration, CheckpointTxg),
    ) -> Self {
        let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
        let sizes = &configuration.immutable.sizes;
        let ring_slot = every_root_ring_slot(sizes)
            .into_iter()
            .find(|ring_slot| {
                matches!(
                    read_root_ring_slot(
                        image,
                        &configuration.immutable.region_devices,
                        sizes,
                        &configuration.immutable.filesystem_identifier,
                        *ring_slot,
                    ),
                    RootRingSlotReading::SelfVerified(root) if (root.instance, root.checkpoint_txg) == key
                )
            })
            .expect("根环里有自证过的这条根");
        self.0.push((
            configuration.immutable.region_devices
                [usize::try_from(ring_slot.region).expect("区域号")],
            slot_offset(ring_slot, sizes.fixed_structure_slot_spacing).0,
            u64::from(sizes.physical_block_size),
        ));
        self
    }

    /// 加上 `image` 上 `key` 那次发布带末条标志的那条记录（每块盘同一个偏移各一份）。
    ///
    /// # Panics
    /// `image` 上择不到系统配置，或环里读不出那次发布带末条标志的记录。
    #[must_use]
    pub fn with_the_last_record_of_the_publish(
        mut self,
        image: &MemoryPool,
        key: (InstanceGeneration, CheckpointTxg),
    ) -> Self {
        let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
        let last_record = scan_journal(image, &configuration)
            .into_values()
            .find(|record| {
                (record.instance, record.checkpoint_txg) == key
                    && record.place_in_publish
                        == JournalRecordPlaceInPublish::LastRecordOfThePublish
            })
            .expect("环里读得出那次发布带末条标志的记录");
        let offset = record_offset(
            last_record.counter,
            configuration.immutable.sizes.journal_ring_bytes,
        );
        for device in image.devices.keys() {
            self.0
                .push((*device, offset.0, singlefs_format::JOURNAL_RECORD_BYTES));
        }
        self
    }

    fn on_device(&self, device: DeviceIdentity) -> Vec<(u64, u64)> {
        self.0
            .iter()
            .filter(|(range_device, _, _)| *range_device == device)
            .map(|(_, offset, length)| (*offset, *length))
            .collect()
    }
}

/// 几块盘共用的一个开关：撤了之后读回全零那一层整个不起作用，盘上的字节照读。用例在可写挂载「重读一次」之前的钩子里撤
/// （`mount::BeforeTheOneReread::CallTheTestOnlyHookFirst`），强制进入「重读读得出」那一支（`.claude/rules/fs-design.md` 五条硬要求第 2 条）。
#[derive(Clone, Debug, Default)]
pub struct ReadBackAllZeroLift(std::rc::Rc<std::cell::Cell<bool>>);

impl ReadBackAllZeroLift {
    /// 撤掉读故障：之后每块盘每一次读都照读盘上的字节。
    pub fn lift(&self) {
        self.0.set(true);
    }

    #[must_use]
    pub fn is_lifted(&self) -> bool {
        self.0.get()
    }
}

/// 读点名的几段读回全零（盘上的字节不动）；写到哪一段，那一段之后就照读；`lift` 撤了之后哪一段都照读。设备报读错不在这一层：
/// 故障模型是「读回全零」（`crates/singlefs-harness/src/history.rs` 注入的那一种）。
pub struct DeviceReadingBackAllZeroOnRanges<Inner: BlockDevice> {
    pub inner: Inner,
    ranges: Vec<(u64, u64)>,
    lift: ReadBackAllZeroLift,
}

fn overlaps(start: u64, length: u64, offset: u64, other_length: u64) -> bool {
    start < offset + other_length && offset < start + length
}

impl<Inner: BlockDevice> BlockDevice for DeviceReadingBackAllZeroOnRanges<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        if self.lift.is_lifted() {
            return Ok(());
        }
        let buffer_length = u64::try_from(buffer.len()).expect("读长装得进 u64");
        for (start, length) in &self.ranges {
            if overlaps(*start, *length, offset.0, buffer_length) {
                let from = usize::try_from(start.saturating_sub(offset.0)).expect("缓冲内偏移");
                let to = usize::try_from((start + length).min(offset.0 + buffer_length) - offset.0)
                    .expect("缓冲内偏移");
                buffer[from..to].fill(0);
            }
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let written_length = u64::try_from(bytes.len()).expect("写长装得进 u64");
        self.ranges
            .retain(|(start, length)| !overlaps(*start, *length, offset.0, written_length));
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.ranges
            .retain(|(start, range_length)| !overlaps(*start, *range_length, offset.0, length));
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

/// 一份镜像交给两块新的稀疏盘，`ranges` 那几段读回全零。
#[must_use]
pub fn devices_reading_back_all_zero_on(
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
) -> Vec<(
    DeviceIdentity,
    DeviceReadingBackAllZeroOnRanges<SparseBlockDevice>,
)> {
    devices_reading_back_all_zero_on_until_lifted(image, ranges, &ReadBackAllZeroLift::default())
}

/// 同 [`devices_reading_back_all_zero_on`]，几块盘共用 `lift` 这一个开关：调用方撤了它，`ranges` 那几段在每块盘上都不再读回全零。
#[must_use]
pub fn devices_reading_back_all_zero_on_until_lifted(
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
    lift: &ReadBackAllZeroLift,
) -> Vec<(
    DeviceIdentity,
    DeviceReadingBackAllZeroOnRanges<SparseBlockDevice>,
)> {
    plain_devices_on(image)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                DeviceReadingBackAllZeroOnRanges {
                    inner,
                    ranges: ranges.on_device(identity),
                    lift: lift.clone(),
                },
            )
        })
        .collect()
}

/// 同上，每块盘外面再录一层（录挂载自己的写，拿来摆「挂载写到一半断电」那几个状态）。
#[must_use]
pub fn recorded_devices_reading_back_all_zero_on(
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
    stream: &SharedStream,
) -> Vec<(
    DeviceIdentity,
    DeviceReadingBackAllZeroOnRanges<RecordingBlockDevice<SparseBlockDevice>>,
)> {
    plain_devices_on(image)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                DeviceReadingBackAllZeroOnRanges {
                    inner: RecordingBlockDevice::with_shared_stream(
                        identity,
                        inner,
                        stream.clone(),
                    ),
                    ranges: ranges.on_device(identity),
                    lift: ReadBackAllZeroLift::default(),
                },
            )
        })
        .collect()
}

/// 两块盘此刻的盘面（盘上的字节，读回全零那一层不算）。
#[must_use]
pub fn image_of_devices_reading_back_all_zero(
    devices: &[(
        DeviceIdentity,
        DeviceReadingBackAllZeroOnRanges<SparseBlockDevice>,
    )],
    device_bytes: u64,
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner.image.clone()))
            .collect(),
        device_size_in_bytes: device_bytes,
    }
}

/// 可写挂载做成之后的池：盘换成这次挂载之后的盘面（不再读回全零），会话是这次挂载的，每条根带的文件内容接着录的那一份记。
#[must_use]
pub fn pool_after_the_mount(
    recorded: &RecordedPublish,
    image_after_the_mount: &MemoryPool,
    mounted: Mounted,
) -> PoolUnderTest<SparseBlockDevice> {
    let (output, session) = MountedSession::of_the_mount(mounted);
    let content_of_the_current_version = recorded
        .pool
        .content_of_each_root
        .get(&(
            output.effective_root.instance,
            output.effective_root.checkpoint_txg,
        ))
        .cloned()
        .expect("挂载落到的那一版是录的池写过的一版");
    let mut content_of_each_root = recorded.pool.content_of_each_root.clone();
    for version in std::iter::once(&output.row_publish).chain(&output.warm_up_publishes) {
        let root = version.root();
        content_of_each_root.insert(
            (root.instance, root.checkpoint_txg),
            content_of_the_current_version.clone(),
        );
    }
    PoolUnderTest {
        parameters: recorded.pool.parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices: plain_devices_on(image_after_the_mount),
        session: Some(session),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version,
        content_of_each_root,
    }
}

/// R 写出的单元里，`image` 上逐字节已不是 R 写的那几份（盘、偏移）。
#[must_use]
pub fn units_of_r_overwritten_on(
    recorded: &RecordedPublish,
    image: &MemoryPool,
) -> Vec<(DeviceIdentity, u64)> {
    recorded
        .unit_writes()
        .into_iter()
        .filter(|write| {
            let length = usize::try_from(write.length_in_bytes()).expect("单元写长");
            let read_back = image.devices[&write.device].read(write.offset, length);
            !write.contents.still_on_disk(&read_back)
        })
        .map(|write| (write.device, write.offset.0))
        .collect()
}

impl RecordedPublish {
    /// R 的根记录（R 的根槽写解出来的）。
    ///
    /// # Panics
    /// 根槽写不带字节，或解不开、自证不过。
    #[must_use]
    pub fn root(&self) -> singlefs_core::root_record::RootRecord {
        singlefs_core::root_record::RootRecord::parse_slot(
            self.root_slot_write().bytes().expect("根槽写带字节"),
            &self.pool.parameters.filesystem_identifier,
        )
        .expect("R 的根自证")
    }

    /// R 写出的单元写里落在 `pointer` 那两条位置条目上的几份（R 自己写了那个单元时每块盘一份；那个单元是照抄上一版的就一份都没有）。
    #[must_use]
    pub fn unit_writes_at(
        &self,
        pointer: &singlefs_core::pointer::NodePointer,
    ) -> Vec<&RetainedWrite> {
        self.unit_writes()
            .into_iter()
            .filter(|write| {
                pointer.locations.iter().any(|location| {
                    location.device == write.device
                        && location.slot.to_device_offset() == write.offset
                })
            })
            .collect()
    }

    /// R 写出的单元按单元归组：同一个单元的几份字节相同、落在不同的盘上；组按第一份的落盘次序。
    ///
    /// # Panics
    /// 单元写不带字节（单元写按构造是普通写）。
    #[must_use]
    pub fn unit_writes_grouped_by_unit(&self) -> Vec<Vec<&RetainedWrite>> {
        let mut groups: Vec<Vec<&RetainedWrite>> = Vec::new();
        for write in self.unit_writes() {
            let bytes = write.bytes().expect("单元写带字节");
            match groups.iter_mut().find(|group| {
                group[0].bytes() == Some(bytes)
                    && group.iter().all(|copy| copy.device != write.device)
            }) {
                Some(group) => group.push(write),
                None => groups.push(vec![write]),
            }
        }
        groups
    }
}

/// `image` 的根环里有自证过的、(实例, txg) 为 `key` 的那条根。
///
/// # Panics
/// `image` 上择不到系统配置。
#[must_use]
pub fn root_is_in_the_ring(image: &MemoryPool, key: (InstanceGeneration, CheckpointTxg)) -> bool {
    let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
    let sizes = &configuration.immutable.sizes;
    every_root_ring_slot(sizes).into_iter().any(|ring_slot| {
        matches!(
            read_root_ring_slot(
                image,
                &configuration.immutable.region_devices,
                sizes,
                &configuration.immutable.filesystem_identifier,
                ring_slot,
            ),
            RootRingSlotReading::SelfVerified(root) if (root.instance, root.checkpoint_txg) == key
        )
    })
}

/// M1 之后依次做的三次用户动作（攻方 `research/prompts/m3-batch35-36-code-r1-opus-output.md` 第一节放开用户动作那一扫的 o、n、s 三种）。
#[derive(Clone, Copy, Debug)]
enum UserActionAfterTheMount {
    Overwrite,
    CreateOneInode,
    SequentialWriteOfTwoDataUnits,
}

/// 取新实例的那次可写挂载 M1（`crash_image` 上 `ranges` 那几段读回全零）：做成就核 R 的单元没被写掉——先核写行那次发布的根落盘之前那一刻
/// （`crash_image` 上施加 M1 写到那时的全部写）R 的单元一份不少；再在 M1 之后的池上依次覆盖写一次、建一个 inode、顺序写两个数据单元，
/// 每一步之后 R 的根还在根环里就核 R 的单元一份不少；交回 M1 落到的那一版。被拒就核拒在任何写之前（录下来的写一条都没有），原样交回拒因。
/// `cell` 只用在断言消息里，说是哪一格。
///
/// # Panics
/// 断言不成立（写掉了 R 的单元，或被拒时已经写了盘）；M1 没有写行那次发布的根槽写；三次用户动作有一次被拒（384 槽小盘上都落得下）。
pub fn mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
    recorded: &RecordedPublish,
    crash_image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
    cell: &str,
) -> Result<(InstanceGeneration, CheckpointTxg), singlefs_core::mount::MountError> {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(crash_image, ranges, &stream);
    let mounted =
        match singlefs_core::mount::mount_writable(&recorded.pool.parameters, &mut devices) {
            Ok(mounted) => mounted,
            Err(refusal) => {
                assert_eq!(
                    stream.operation_count(),
                    0,
                    "{cell}：可写挂载被拒（{refusal:?}），拒在任何写之前"
                );
                return Err(refusal);
            }
        };
    let effective = (
        mounted.output.effective_root.instance,
        mounted.output.effective_root.checkpoint_txg,
    );
    let (mount_writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let row_root = mount_writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("M1 写行那次发布有根槽写");
    let mut image_before_the_row_root = crash_image.clone();
    image_before_the_row_root.apply_writes(&mount_writes[..row_root]);
    assert_eq!(
        units_of_r_overwritten_on(recorded, &image_before_the_row_root),
        Vec::<(DeviceIdentity, u64)>::new(),
        "{cell}：写行那次发布的根落盘之前，M1 写下的一切都没碰 R 的单元"
    );
    let mut image_after_the_mount = crash_image.clone();
    image_after_the_mount.apply_writes(&mount_writes);
    let mut pool = pool_after_the_mount(recorded, &image_after_the_mount, mounted);
    for user_action in [
        UserActionAfterTheMount::Overwrite,
        UserActionAfterTheMount::CreateOneInode,
        UserActionAfterTheMount::SequentialWriteOfTwoDataUnits,
    ] {
        match user_action {
            UserActionAfterTheMount::Overwrite => pool.overwrite(OVERWRITE_BYTES).map(|_| ()),
            UserActionAfterTheMount::CreateOneInode => pool.create_one_inode().map(|_| ()),
            UserActionAfterTheMount::SequentialWriteOfTwoDataUnits => pool
                .sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())
                .map(|_| ()),
        }
        .unwrap_or_else(|refusal| panic!("{cell}：M1 之后{user_action:?}落得下：{refusal:?}"));
        let image = pool.image();
        if root_is_in_the_ring(&image, recorded.key) {
            assert_eq!(
                units_of_r_overwritten_on(recorded, &image),
                Vec::<(DeviceIdentity, u64)>::new(),
                "{cell}：M1 之后{user_action:?}，R 的根还在根环里，R 的单元一份都没被写掉"
            );
        }
    }
    Ok(effective)
}
