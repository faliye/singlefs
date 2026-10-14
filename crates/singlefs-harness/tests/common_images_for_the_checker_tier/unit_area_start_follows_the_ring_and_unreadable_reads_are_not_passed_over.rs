//! harness 测试文件 `crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_checker::image::ImageReader;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::{PoolAllocator, UnitAreaStart};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::recovery::{
    readable_roots_with_ring_slots, recover, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, RootRingSlot, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::{
    unit_area_start_slot_recorded_in_the_slot, SystemImmutableSizes,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
    TransactionOutput,
};
use singlefs_format::{
    JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, SLOT_BYTES,
    SYSTEM_CONFIGURATION_SLOT_BYTES,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

pub const FOUR_GIBIBYTES: u64 = 4 << 30;

pub const MEBIBYTE: u64 = 1 << 20;

pub const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-rev-a3b";

pub const WRITE_TIME_SECONDS: u64 = 1_788_000_000;

/// 内存稀疏盘按 512 字节扇区存：物理块宽 512。
pub const SECTOR_PHYSICAL_BLOCK_SIZE: PhysicalBlockSizeInBytes = PhysicalBlockSizeInBytes(512);

pub const FIRST_FILE_BYTES: usize = 3000;

pub type SparseDevices = Vec<(DeviceIdentity, SparseBlockDevice)>;

pub fn geometry_with_journal_ring_bytes(journal_ring_bytes: u64) -> SystemImmutableSizes {
    SystemImmutableSizes {
        physical_block_size: 512,
        minimum_input_output_bytes: 512,
        fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
        journal_ring_bytes,
        root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
    }
}

pub fn parameters_with_journal_ring_bytes(journal_ring_bytes: u64) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: geometry_with_journal_ring_bytes(journal_ring_bytes),
    }
}

pub fn content_of(length_in_bytes: usize, seed: u64) -> Vec<u8> {
    (0..length_in_bytes)
        .map(|index| {
            let index = u64::try_from(index).expect("长度装得进 u64");
            u8::try_from((index * 131 + seed * 7 + 1) % 251).expect("小于 256")
        })
        .collect()
}

pub fn sparse_devices(device_bytes: u64) -> SparseDevices {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(device_bytes, SECTOR_PHYSICAL_BLOCK_SIZE),
            )
        })
        .collect()
}

pub fn memory_pool_of(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    device_bytes: u64,
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: device_bytes,
    }
}

/// 两块内存盘上 mkfs → 取号 1 → 暖机 → 第一个文件（txg 3），分配器是 mkfs 同一个进程那一条（`allocator_after_make_filesystem`）。
pub struct PoolAfterTheFirstFile {
    pub parameters: MakeFilesystemParameters,
    pub device_bytes: u64,
    pub devices: SparseDevices,
    pub unit_area_start: UnitAreaStart,
    pub allocator: PoolAllocator,
    pub instance: InstanceGeneration,
    pub first: TransactionOutput,
}

pub fn pool_after_the_first_file(
    journal_ring_bytes: u64,
    device_bytes: u64,
) -> PoolAfterTheFirstFile {
    let parameters = parameters_with_journal_ring_bytes(journal_ring_bytes);
    let mut devices = sparse_devices(device_bytes);
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let content = content_of(FIRST_FILE_BYTES, 0);
    let (instance, first) = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号 1");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机写了两条根"),
            FirstFile {
                content: &content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个文件");
        (instance, first)
    };
    PoolAfterTheFirstFile {
        parameters,
        device_bytes,
        devices,
        unit_area_start: genesis.unit_area_start,
        allocator,
        instance,
        first,
    }
}

/// 一块盘系统配置槽 `slot_index`（0 或 1）的原样字节。
pub fn system_configuration_slot_bytes(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    device: DeviceIdentity,
    slot_index: u64,
    parameters: &MakeFilesystemParameters,
) -> Vec<u8> {
    let offset = slot_index * u64::from(parameters.geometry.fixed_structure_slot_spacing);
    devices
        .iter()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘")
        .1
        .image
        .read(
            DeviceOffsetInBytes(offset),
            usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096"),
        )
}

/// 一条非默认环长上走完 mkfs → 第一个文件 → 可写挂载 → 冷走读 → 池级 checker，逐处核单元区起点都是环长现算的那个 `expected_start`。
pub fn assert_the_unit_area_follows_a_journal_ring_of(
    images: &mut PoolCheckerImages,
    journal_ring_bytes: u64,
    device_bytes: u64,
    expected_start: u64,
) {
    let mut pool = pool_after_the_first_file(journal_ring_bytes, device_bytes);
    assert_eq!(
        pool.unit_area_start.slot(),
        SlotNumber(expected_start),
        "mkfs 按环长现算的单元区起点"
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        assert_eq!(
            unit_area_start_slot_recorded_in_the_slot(&system_configuration_slot_bytes(
                &pool.devices,
                device,
                0,
                &pool.parameters,
            )),
            SlotNumber(expected_start),
            "系统配置偏移 417 的 8 字节写的是现算的起点（盘 {device:?}）"
        );
    }
    let genesis_instance_table_slot = pool.first.root.instance_table.locations[0].slot;
    assert_eq!(
        genesis_instance_table_slot,
        SlotNumber(expected_start),
        "mkfs 的实例表第 0 片就写在单元区起点（第一个文件照抄它的指针）"
    );
    assert!(
        pool.first
            .allocation_records
            .iter()
            .all(|record| record.slot.0 >= expected_start),
        "第一个文件的每一条分配记录都落在单元区起点之后：{:?}",
        pool.first.allocation_records
    );
    let journal_ring_end_slot =
        (JOURNAL_RING_START_SLOT * SLOT_BYTES + journal_ring_bytes).div_ceil(SLOT_BYTES);
    assert!(
        journal_ring_end_slot <= expected_start,
        "单元区起点不在 journal 环里"
    );
    let image_after_the_first_file = memory_pool_of(&pool.devices, pool.device_bytes);
    for device in [0, 1] {
        let candidates = ImageReader::candidate_unit_slots(&image_after_the_first_file, device)
            .expect("内存镜像交得出单元扫描候选");
        assert!(
            candidates.contains(&expected_start)
                && candidates.iter().all(|slot| *slot >= expected_start),
            "崩溃镜像的单元扫描候选从盘上记着的单元区起点起、罩住 mkfs 的实例表（盘 {device}）：{:?}",
            candidates.iter().take(4).collect::<Vec<_>>()
        );
    }
    // 池级 checker 按偏移 417 那 8 字节划单元区，一条不红
    images.hand_out(
        "第一个文件之后",
        || image_after_the_first_file.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    let mounted = mount_writable(&pool.parameters, &mut pool.devices).expect("可写挂载");
    for device_map in &mounted.allocator.devices {
        assert_eq!(
            device_map.unit_area_start().slot(),
            SlotNumber(expected_start),
            "挂载按盘上系统配置建的空闲图从现算的起点起"
        );
    }
    let report = recover(&pool.devices, JournalPolicy::Consult);
    assert!(
        matches!(report.outcome, RecoveryOutcome::FileRead { ref content, .. } if *content == content_of(FIRST_FILE_BYTES, 0)),
        "冷走读读回第一个文件：{:?}",
        report.outcome
    );
    images.hand_out(
        "可写挂载写行、暖机之后",
        || memory_pool_of(&pool.devices, pool.device_bytes),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// 环 1 GiB（改之前 mkfs 按「环末端越过编译期的单元区起点」拒）：单元区从 1024 + 65536 = 66560 起，mkfs、第一个文件、可写挂载、冷走读、checker 都按它。
pub fn a_one_gibibyte_journal_ring_puts_the_unit_area_right_after_the_ring_through_mount_and_checker(
    images: &mut PoolCheckerImages,
) {
    assert_the_unit_area_follows_a_journal_ring_of(images, 1 << 30, FOUR_GIBIBYTES, 66_560);
}

/// 环 128 MiB（改之前单元区照旧从 50176 起、白扔 640 MiB）：单元区从 1024 + 8192 = 9216 起。
pub fn a_128_mebibyte_journal_ring_puts_the_unit_area_right_after_the_ring_through_mount_and_checker(
    images: &mut PoolCheckerImages,
) {
    assert_the_unit_area_follows_a_journal_ring_of(images, 128 * MEBIBYTE, FOUR_GIBIBYTES, 9216);
}

/// 这个文件自己的读故障：几段落点，每一段按它自己被读的次数（从 1 数，一次读碰到这一段就算一次）点名哪几次读坏。
/// 两块盘共用一份。
pub struct ChosenReadFaults {
    pub ranges: Vec<ChosenReadFaultsOfARange>,
}

/// 点名的那几次读怎么坏。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChosenReadFault {
    /// 块设备报 `InputOutput`（读不出）。
    Fails,
    /// 读得出，这一段落在这次读里的第一个字节取反（根槽里是 magic 那一字节：自证不过）。
    ReturnsTheFirstByteOfTheRangeFlipped,
}

pub struct ChosenReadFaultsOfARange {
    pub device: DeviceIdentity,
    pub offset_in_bytes: u64,
    pub length_in_bytes: u64,
    pub failing_read_numbers: BTreeSet<u64>,
    pub fault: ChosenReadFault,
    pub reads_so_far: u64,
}

/// 这一次读里点名要坏的那一段：它怎么坏，和它在这次读里从第几个字节起。
pub struct ChosenReadGoingBad {
    pub fault: ChosenReadFault,
    pub first_byte_of_the_range_in_the_read: usize,
}

/// 按 [`ChosenReadFaults`] 读坏的内存盘；写、写零、屏障原样交给里面那块盘。
pub struct DeviceFailingChosenReads {
    pub identity: DeviceIdentity,
    pub inner: SparseBlockDevice,
    pub faults: Rc<RefCell<ChosenReadFaults>>,
}

impl BlockDevice for DeviceFailingChosenReads {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        let going_bad = self
            .faults
            .borrow_mut()
            .note_a_read_and_tell_how_it_goes_bad(self.identity, offset.0, length_in_bytes);
        match going_bad {
            None => self.inner.read_at(offset, buffer),
            Some(ChosenReadGoingBad {
                fault: ChosenReadFault::Fails,
                ..
            }) => Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "用例点名的这一次读坏",
            ))),
            Some(ChosenReadGoingBad {
                fault: ChosenReadFault::ReturnsTheFirstByteOfTheRangeFlipped,
                first_byte_of_the_range_in_the_read,
            }) => {
                self.inner.read_at(offset, buffer)?;
                buffer[first_byte_of_the_range_in_the_read] ^= 0xff;
                Ok(())
            }
        }
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

/// 一段读故障：哪块盘、从哪个字节起、多长，这一段第几次读坏（读不出）。
pub fn chosen_read_faults_of(
    device: DeviceIdentity,
    offset_in_bytes: u64,
    length_in_bytes: u64,
    failing_read_numbers: &[u64],
) -> ChosenReadFaultsOfARange {
    ChosenReadFaultsOfARange {
        device,
        offset_in_bytes,
        length_in_bytes,
        failing_read_numbers: failing_read_numbers.iter().copied().collect(),
        fault: ChosenReadFault::Fails,
        reads_so_far: 0,
    }
}

pub fn with_chosen_read_faults(
    devices: SparseDevices,
    ranges: Vec<ChosenReadFaultsOfARange>,
) -> Vec<(DeviceIdentity, DeviceFailingChosenReads)> {
    let faults = Rc::new(RefCell::new(ChosenReadFaults { ranges }));
    devices
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                DeviceFailingChosenReads {
                    identity,
                    inner,
                    faults: faults.clone(),
                },
            )
        })
        .collect()
}

pub fn without_chosen_read_faults(
    devices: Vec<(DeviceIdentity, DeviceFailingChosenReads)>,
) -> SparseDevices {
    devices
        .into_iter()
        .map(|(identity, device)| (identity, device.inner))
        .collect()
}

/// 管理员回退算水位的树 ID 那一半（D23（journal 的角色与格式） 已定项 14「水位」、D8（核心索引结构） 已定项 8 ②「号永不重发」；
/// C586（回退取树 ID 水位的那一半没有用例），攻方 `research/prompts/m2-closeout-code-r3-opus-output.md`「X1-e」）：
/// 第一个文件（txg 3）十三棵树发 65536..65549、水位 65549，覆盖写一次（txg 4）照带 65549；环里别的根（mkfs 与暖机）都带 mkfs 种下的 65536。
/// 回退到 txg 3 时这两条根的槽判候选那两遍（各自第 1、2 次读）读得出，算水位那一遍（第 3 次，`readable_roots`，不重读）都瞬时读不出：
/// 环里读得出的根都是 65536，只有内存里的现行那一版带 65549。回退那次发布的树 ID 水位仍是 65549——树表里 65536..65549 那几棵树都在水位之下（I-7.8 不红），
/// 之后再发树 ID 从 65549 起、不重发 65536..65549。只取环里读得出的根时是 65536——`crates/mutations.tsv` 里 C586 那一行证这条断言红。
pub fn the_rollback_takes_the_tree_identifier_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_every_root_carrying_it(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_after_the_first_file(JOURNAL_RING_DEFAULT_BYTES, FOUR_GIBIBYTES);
    let sizes = pool.parameters.geometry;
    let overwritten = {
        let content = content_of(2097, 1);
        let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &pool.first,
            FirstFile {
                content: &content,
                write_time_seconds: WRITE_TIME_SECONDS + 60,
            },
            pool.instance,
        )
        .expect("覆盖写")
    };
    assert_eq!(overwritten.root.checkpoint_txg, CheckpointTxg(4));
    assert_eq!(overwritten.root.tree_identifier_watermark, 65549);
    let roots_in_the_ring = readable_roots_with_ring_slots(
        &pool.devices,
        &pool.parameters.region_devices,
        &sizes,
        &pool.parameters.filesystem_identifier,
    );
    let slots_of_the_roots_carrying_the_first_file_version_watermark: Vec<RootRingSlot> =
        roots_in_the_ring
            .iter()
            .filter(|(_, root)| root.tree_identifier_watermark == 65549)
            .map(|(ring_slot, _)| *ring_slot)
            .collect();
    assert_eq!(
        roots_in_the_ring
            .iter()
            .filter(|(_, root)| root.tree_identifier_watermark == 65549)
            .map(|(_, root)| root.checkpoint_txg)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([CheckpointTxg(3), CheckpointTxg(4)]),
        "带 65549 的只有第一个文件与覆盖写那两条根"
    );
    assert!(
        roots_in_the_ring
            .iter()
            .all(|(_, root)| root.tree_identifier_watermark == 65549
                || root.tree_identifier_watermark == 65536),
        "环里别的根都带 mkfs 种下的 65536"
    );
    let parameters = pool.parameters.clone();
    let mut faulty = with_chosen_read_faults(
        std::mem::take(&mut pool.devices),
        slots_of_the_roots_carrying_the_first_file_version_watermark
            .iter()
            .map(|ring_slot| {
                chosen_read_faults_of(
                    parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")],
                    slot_offset(*ring_slot, sizes.fixed_structure_slot_spacing).0,
                    u64::from(sizes.physical_block_size),
                    &[3],
                )
            })
            .collect(),
    );
    let mut current = overwritten;
    roll_back_by_a_forward_publish(
        &parameters,
        &mut faulty,
        &mut pool.allocator,
        &mut current,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
    )
    .expect("判候选两遍读得出，回退做成");
    pool.devices = without_chosen_read_faults(faulty);
    assert_eq!(
        current.root.checkpoint_txg,
        CheckpointTxg(5),
        "回退那次发布"
    );
    assert_eq!(
        current.root.tree_identifier_watermark, 65549,
        "回退那次发布的树 ID 水位 = max(现行那一版内存里的 65549, 环里读得出的根的 65536)"
    );
    // 回退之后池级 checker 不红：树表里的树 ID 都在回退那一版的水位之下（I-7.8）
    images.hand_out(
        "回退之后",
        || memory_pool_of(&pool.devices, pool.device_bytes),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

impl ChosenReadFaults {
    /// 记下这一次读碰到的每一段被读了一次；这一次读里有一段点名这一次坏，就交回第一段点名的那一个。
    pub fn note_a_read_and_tell_how_it_goes_bad(
        &mut self,
        device: DeviceIdentity,
        offset_in_bytes: u64,
        length_in_bytes: u64,
    ) -> Option<ChosenReadGoingBad> {
        let mut going_bad = None;
        for range in &mut self.ranges {
            let overlaps = range.device == device
                && offset_in_bytes < range.offset_in_bytes + range.length_in_bytes
                && range.offset_in_bytes < offset_in_bytes + length_in_bytes;
            if !overlaps {
                continue;
            }
            range.reads_so_far += 1;
            if going_bad.is_none() && range.failing_read_numbers.contains(&range.reads_so_far) {
                going_bad = Some(ChosenReadGoingBad {
                    fault: range.fault,
                    first_byte_of_the_range_in_the_read: usize::try_from(
                        range.offset_in_bytes.saturating_sub(offset_in_bytes),
                    )
                    .expect("一次读里的偏移装得进 usize"),
                });
            }
        }
        going_bad
    }
}
