//! 读—改—写的故障格（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R7「故障格照正推 R7.1，每一格的成对用例旁钉读序列」；
//! 正推腿 R7.1 的 F2、F5、F6、F7、F11，`partial_write_reads_the_old_unit_from_disk.rs` 罩了 F1、F3、F4、F8 与截断尾单元那一对）：每一格同一个写在
//! 注入与不注入两份同一刻的池上各做一次（[`PoolOnObservedDevices::duplicate`]），注入的那一份照格的要求成或败、败的盘上逐字节不变，不注入的那一份
//! 发成；旁边钉落在数据单元槽上的读序列（第一个写之前）。盘是两块内存稀疏盘，外面包一层记下整个池每一次读写、按偏移注入读故障与吞写的盘。
//! 提示与映射分开的两格（F5、F6）改的是交给写入口的那一版（`TransactionOutput` 的数据指针与映射 key）：那一版就是从一份提示过时、映射缺条目的
//! 镜像上重建出来的样子，写入口只认交进来的那一版。

mod common;

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_core::address::{
    DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber,
    InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mounted_read::open_pool_for_read;
use singlefs_core::recovery::PoolReader;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_partial_write, publish_sequential_write,
    read_back_the_old_payload_of_a_data_unit, warm_up, FirstFile, OldDataUnitReadBackFailure,
    PartialWriteBytes, PoolWriter, PublishError, TransactionOutput, TransactionUnit,
    TruncationRefusal, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{data_unit_payload_capacity, seal_header_checksum};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

/// 块层上的一次读或写，按整个池一份次序记：读记盘、偏移、长度，写记盘与偏移。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BlockOperation {
    Read {
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
    },
    Write {
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
    },
}

/// 这块盘上注入的故障：从点名的偏移起的那一读报读错；与点名的字节段有交叠的写报成功、一个字节都不落（说谎的设备，正推 R7.1 F11
/// 「`WriteIsSwallowed`」那一形）。
#[derive(Clone, Debug, Default)]
struct DeviceFaults {
    reads_failing_at: BTreeSet<u64>,
    writes_swallowed_over: Vec<std::ops::Range<u64>>,
}

/// 内存稀疏盘外面包一层：每一次读写按整个池一份次序记下来，读写按 [`DeviceFaults`] 注入故障。
struct ObservedDevice {
    identity: DeviceIdentity,
    disk: SparseBlockDevice,
    operations: Arc<Mutex<Vec<BlockOperation>>>,
    faults: DeviceFaults,
}

impl ObservedDevice {
    fn record(&self, operation: BlockOperation) {
        self.operations
            .lock()
            .expect("测试单线程，记录的锁不会中毒")
            .push(operation);
    }

    fn swallows_a_write_over(&self, offset: DeviceOffsetInBytes, length: u64) -> bool {
        let written = offset.0..offset.0 + length;
        self.faults
            .writes_swallowed_over
            .iter()
            .any(|swallowed| swallowed.start < written.end && written.start < swallowed.end)
    }
}

impl BlockDevice for ObservedDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.record(BlockOperation::Read {
            device: self.identity,
            offset,
        });
        if self.faults.reads_failing_at.contains(&offset.0) {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "注入的读错",
            )));
        }
        self.disk.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.record(BlockOperation::Write {
            device: self.identity,
            offset,
        });
        if self.swallows_a_write_over(offset, u64::try_from(bytes.len()).expect("写的字节数"))
        {
            return Ok(());
        }
        self.disk.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.record(BlockOperation::Write {
            device: self.identity,
            offset,
        });
        if self.swallows_a_write_over(offset, length) {
            return Ok(());
        }
        self.disk.write_zeroes_at(offset, length)
    }
    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.record(BlockOperation::Write {
            device: self.identity,
            offset,
        });
        self.disk.discard_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.disk.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.disk.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.disk.size_in_bytes()
    }
}

const INSTANCE: InstanceGeneration = InstanceGeneration(1);
const DEVICE_ZERO: DeviceIdentity = DeviceIdentity(0);
const DEVICE_ONE: DeviceIdentity = DeviceIdentity(1);
const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);
/// 一个数据单元占几个槽、几个字节（两槽 32 KiB）。
const DATA_UNIT_SLOTS: u64 = 2;
/// 一个谁都没占过的槽（4 GiB 的盘上靠后那一片，读回全 0）：F5、F6 让提示指到这里，模拟提示过时。
const SLOT_NO_UNIT_WAS_EVER_WRITTEN_TO: SlotNumber = SlotNumber(200_000);

fn payload_capacity() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634")
}

/// 一个池：两块盘、分配器、现行那一版、整个池的读写记录。
struct PoolOnObservedDevices {
    devices: Vec<(DeviceIdentity, ObservedDevice)>,
    allocator: PoolAllocator,
    output: TransactionOutput,
    operations: Arc<Mutex<Vec<BlockOperation>>>,
    next_write_time_seconds: u64,
}

impl PoolOnObservedDevices {
    /// mkfs → 取号 → 暖机 → 新池新建文件 → 第一个文件整份顺序写成 `content`，两块 4 GiB 内存稀疏盘。
    fn with_the_first_file_written_as(content: &[u8]) -> Self {
        let parameters = parameters();
        let operations = Arc::new(Mutex::new(Vec::new()));
        let mut devices: Vec<(DeviceIdentity, ObservedDevice)> = (0..2u32)
            .map(|number| {
                (
                    DeviceIdentity(number),
                    ObservedDevice {
                        identity: DeviceIdentity(number),
                        disk: SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                        operations: Arc::clone(&operations),
                        faults: DeviceFaults::default(),
                    },
                )
            })
            .collect();
        let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
        let mut allocator = PoolAllocator::new(vec![
            DeviceFreeMap::new(DEVICE_ZERO, IMAGE_BYTES),
            DeviceFreeMap::new(DEVICE_ONE, IMAGE_BYTES),
        ]);
        allocator.mark_format_time_units(
            Placement {
                slot: INSTANCE_TABLE_SLOT,
                span: 2,
            },
            Placement {
                slot: TREE_TABLE_GENESIS_SLOT,
                span: 1,
            },
        );
        allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
            &parameters,
            &genesis,
        ));
        let first_content = file_content();
        let output = {
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            let instance = acquire_instance(&mut writer).expect("取号");
            assert_eq!(instance, INSTANCE);
            let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
            let created = publish_first_file(
                &mut writer,
                &mut allocator,
                warmed.roots.last().expect("暖机两代根"),
                FirstFile {
                    content: &first_content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS,
                },
                instance,
                &warmed.last_record_bytes,
            )
            .expect("新池新建文件");
            publish_sequential_write(
                &mut writer,
                &mut allocator,
                &created,
                FirstFile {
                    content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
                },
                instance,
            )
            .expect("顺序写")
        };
        PoolOnObservedDevices {
            devices,
            allocator,
            output,
            operations,
            next_write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
        }
    }

    /// 同一刻的一份拷贝（盘的字节、分配器、现行那一版），读写记录另起一份，故障不带过去。
    fn duplicate(&self) -> Self {
        let operations = Arc::new(Mutex::new(Vec::new()));
        PoolOnObservedDevices {
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| {
                    let mut disk =
                        SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
                    disk.image = device.disk.image.clone();
                    (
                        *identity,
                        ObservedDevice {
                            identity: *identity,
                            disk,
                            operations: Arc::clone(&operations),
                            faults: DeviceFaults::default(),
                        },
                    )
                })
                .collect(),
            allocator: self.allocator.clone(),
            output: self.output.clone(),
            operations,
            next_write_time_seconds: self.next_write_time_seconds,
        }
    }

    /// 两块盘此刻的字节。
    fn image(&self) -> MemoryPool {
        MemoryPool {
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, device.disk.image.clone()))
                .collect(),
            device_size_in_bytes: IMAGE_BYTES,
        }
    }

    fn forget_the_operations(&self) {
        self.operations
            .lock()
            .expect("测试单线程，记录的锁不会中毒")
            .clear();
    }

    fn operations(&self) -> Vec<BlockOperation> {
        self.operations
            .lock()
            .expect("测试单线程，记录的锁不会中毒")
            .clone()
    }

    fn device_mut(&mut self, device: DeviceIdentity) -> &mut ObservedDevice {
        &mut self
            .devices
            .iter_mut()
            .find(|(identity, _)| *identity == device)
            .expect("池里有这块盘")
            .1
    }

    /// 接着现行那一版按偏移写一次；发成了池接到它那一版上。
    fn write_at_offset(
        &mut self,
        inode: InodeNumber,
        offset: u64,
        bytes: &[u8],
    ) -> Result<TransactionOutput, PublishError> {
        let parameters = parameters();
        self.next_write_time_seconds += 1;
        let previous = self.output.clone();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        let published = publish_partial_write(
            &mut writer,
            &mut self.allocator,
            &previous,
            inode,
            FileOffsetInBytes(offset),
            PartialWriteBytes::new(bytes).expect("用例写的字节不是空的"),
            self.next_write_time_seconds,
            INSTANCE,
        );
        if let Ok(version) = &published {
            self.output = version.clone();
        }
        published
    }

    /// 两块盘的 `slot` 起一个数据单元换成 `bytes`（直写底下的稀疏盘，不进读写记录）。
    fn overwrite_the_data_unit_on_both_devices(&mut self, slot: SlotNumber, bytes: &[u8]) {
        for device in [DEVICE_ZERO, DEVICE_ONE] {
            self.device_mut(device)
                .disk
                .write_at(slot.to_device_offset(), bytes, WriteDurability::Plain)
                .expect("内存盘写得进");
        }
    }

    /// 两块盘里 `slot` 起一个数据单元的字节（第一块盘那一份）。
    fn data_unit_bytes_at(&self, slot: SlotNumber) -> Vec<u8> {
        let mut bytes = vec![
            0u8;
            usize::try_from(DATA_UNIT_SLOTS * SLOT_BYTES)
                .expect("32768 装得进 usize")
        ];
        self.devices[0]
            .1
            .disk
            .read_at(slot.to_device_offset(), &mut bytes)
            .expect("内存盘读得出");
        bytes
    }
}

/// `version` 里第一个文件第 `unit` 个数据单元的槽（两块盘上同一个槽号）。
fn slot_of_the_data_unit(version: &TransactionOutput, unit: u64) -> SlotNumber {
    let pointer = version.data_pointers[usize::try_from(unit).expect("单元序号")];
    assert_eq!(
        pointer.locations[0].slot, pointer.locations[1].slot,
        "两块盘同一个槽号"
    );
    pointer.locations[0].slot
}

/// 第一个写之前、落在 `watched` 这几个槽上的读，按次序交回（盘、槽）。
fn reads_of_the_watched_slots_before_the_first_write(
    operations: &[BlockOperation],
    watched: &[SlotNumber],
) -> Vec<(DeviceIdentity, SlotNumber)> {
    let mut reads = Vec::new();
    for operation in operations {
        match *operation {
            BlockOperation::Write { .. } => return reads,
            BlockOperation::Read { device, offset } => {
                if let Some(slot) = watched
                    .iter()
                    .find(|slot| slot.to_device_offset() == offset)
                {
                    reads.push((device, *slot));
                }
            }
        }
    }
    reads
}

/// 挂载态读者按 `version` 的根打开第一个文件、从 0 读到 size。
fn whole_first_file_read_by_the_mounted_reader(
    image: &MemoryPool,
    version: &TransactionOutput,
) -> Vec<u8> {
    let mounted = open_pool_for_read(image, &version.root).expect("打开挂载态");
    let file = mounted.open_file(image, FIRST_FILE).expect("打开文件");
    let size = file.inode_record().size;
    file.read_at(image, FileOffsetInBytes(0), size)
        .expect("从 0 读到 size")
        .bytes
}

/// 三个单元的内容（最后一个差 7 字节满），每个字节按位置算、彼此不同。
fn content_of_three_units() -> Vec<u8> {
    let length = usize::try_from(3 * payload_capacity() - 7).expect("长度");
    (0..length)
        .map(|position| u8::try_from((position * 7 + 1) % 251).expect("小于 256"))
        .collect()
}

/// `content` 在偏移 `offset` 起换成 `patch`。
fn content_with_the_patch(content: &[u8], offset: u64, patch: &[u8]) -> Vec<u8> {
    let start = usize::try_from(offset).expect("偏移");
    let mut patched = content.to_vec();
    patched[start..start + patch.len()].copy_from_slice(patch);
    patched
}

/// 拒的是读—改—写读不回第一个文件第 `unit` 个旧单元、原因是 `read_back`。
fn refused_as_an_old_unit_not_read_back(
    refusal: &PublishError,
    unit: u64,
    read_back: OldDataUnitReadBackFailure,
) -> bool {
    matches!(
        refusal,
        PublishError::TruncationRefused(TruncationRefusal::OldDataUnitToRewriteNotReadBack {
            inode,
            unit: refused_unit,
            read_back: refused_read_back,
        }) if *inode == FIRST_FILE
            && *refused_unit == DataUnitIndexInFile(unit)
            && *refused_read_back == read_back
    )
}

/// 一个写都没发。
fn no_write_was_issued(operations: &[BlockOperation]) -> bool {
    operations
        .iter()
        .all(|operation| matches!(operation, BlockOperation::Read { .. }))
}

/// 一个数据单元的字节改过之后重封：载荷校验和（偏移 101，罩头之后到单元末尾）与头校验和照改过的字节重算（`singlefs_core::unit` 的封法）。
fn reseal_data_unit(bytes: &mut [u8]) {
    let header_end = 105;
    let payload_checksum = crc32_castagnoli(&bytes[header_end..]);
    bytes[101..105].copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(bytes, header_end);
}

/// 第一个文件第 `unit` 个单元的数据指针两条位置条目的整单元校验和换成 `bytes` 的（指针随单元字节改过，模拟「单元自洽、指针也对得上」的坏单元）。
fn point_the_data_unit_at_bytes(version: &mut TransactionOutput, unit: u64, bytes: &[u8]) {
    let checksum = crc32_castagnoli(bytes);
    for location in &mut version.data_pointers[usize::try_from(unit).expect("单元序号")].locations
    {
        location.unit_checksum = checksum;
    }
}

/// R7.1 F2：单元 1 只有第一块盘那一份读错（第二块盘照常）：读回旧单元先读第一份失败、再读第二份对上，部分写发成；不注入的同一个写第一份就对上。
/// 判别力：读回只试第一条位置提示时注入的那一份失败（`crates/mutations.tsv` 里「m4 部分写 R7 F4」那一行同样让这一条红）。
#[test]
fn a_partial_write_reads_the_second_copy_when_the_first_copy_of_the_old_unit_fails_to_read() {
    let content = content_of_three_units();
    let mut pool = PoolOnObservedDevices::with_the_first_file_written_as(&content);
    let old_slots: Vec<SlotNumber> = (0..3)
        .map(|unit| slot_of_the_data_unit(&pool.output, unit))
        .collect();
    let patch = vec![0x5Au8; 4096];
    let offset = payload_capacity() + 100;
    let mut without_the_fault = pool.duplicate();
    pool.device_mut(DEVICE_ZERO)
        .faults
        .reads_failing_at
        .insert(old_slots[1].to_device_offset().0);
    pool.forget_the_operations();
    let after = pool
        .write_at_offset(FIRST_FILE, offset, &patch)
        .expect("只有一份读错：读第二份，部分写发成");
    assert_eq!(
        reads_of_the_watched_slots_before_the_first_write(&pool.operations(), &old_slots),
        vec![
            (DEVICE_ZERO, old_slots[1]),
            (DEVICE_ONE, old_slots[1]),
            (DEVICE_ZERO, old_slots[2]),
            (DEVICE_ZERO, old_slots[1]),
            (DEVICE_ZERO, old_slots[1]),
            (DEVICE_ONE, old_slots[1]),
            (DEVICE_ZERO, old_slots[2]),
            (DEVICE_ONE, old_slots[2]),
        ],
        "读回：单元 1 第一份读错、第二份对上，单元 2 第一份一读；释放之前：单元 1 第一份读错、重读一次，第二份一读，单元 2 两份"
    );
    assert_eq!(
        whole_first_file_read_by_the_mounted_reader(&pool.image(), &after),
        content_with_the_patch(&content, offset, &patch)
    );
    without_the_fault.forget_the_operations();
    without_the_fault
        .write_at_offset(FIRST_FILE, offset, &patch)
        .expect("不注入：部分写发成");
    assert_eq!(
        reads_of_the_watched_slots_before_the_first_write(
            &without_the_fault.operations(),
            &old_slots
        )[..2],
        [(DEVICE_ZERO, old_slots[1]), (DEVICE_ZERO, old_slots[2])],
        "不注入：单元 1、2 各在第一份上读一次就对上"
    );
}

/// R7.1 F5：单元 1 的提示过时（交给写入口的那一版里它的数据指针指到一个谁都没写过的槽，读回全 0、整单元校验和对不上），中央映射里的落点读得出：
/// 两条提示各一读之后按映射读第一份对上，部分写发成、读回改过的内容；不注入的同一个写不读提示之外的槽。
/// 判别力：提示都不算数时不再按映射读（直接拒）时这一条红（`crates/mutations.tsv` 一行）。
#[test]
fn a_partial_write_reads_the_old_unit_at_the_mapped_location_when_its_hint_is_stale() {
    let content = content_of_three_units();
    let mut pool = PoolOnObservedDevices::with_the_first_file_written_as(&content);
    let old_slots: Vec<SlotNumber> = (0..3)
        .map(|unit| slot_of_the_data_unit(&pool.output, unit))
        .collect();
    let watched = [SLOT_NO_UNIT_WAS_EVER_WRITTEN_TO, old_slots[1], old_slots[2]];
    let patch = vec![0x6Bu8; 4096];
    let offset = payload_capacity() + 100;
    let mut without_the_fault = pool.duplicate();
    for location in &mut pool.output.data_pointers[1].locations {
        location.slot = SLOT_NO_UNIT_WAS_EVER_WRITTEN_TO;
    }
    pool.forget_the_operations();
    let after = pool
        .write_at_offset(FIRST_FILE, offset, &patch)
        .expect("提示过时、映射读得出：部分写发成");
    assert_eq!(
        reads_of_the_watched_slots_before_the_first_write(&pool.operations(), &watched),
        vec![
            (DEVICE_ZERO, SLOT_NO_UNIT_WAS_EVER_WRITTEN_TO),
            (DEVICE_ONE, SLOT_NO_UNIT_WAS_EVER_WRITTEN_TO),
            (DEVICE_ZERO, old_slots[1]),
            (DEVICE_ZERO, old_slots[2]),
            (DEVICE_ZERO, old_slots[1]),
            (DEVICE_ONE, old_slots[1]),
            (DEVICE_ZERO, old_slots[2]),
            (DEVICE_ONE, old_slots[2]),
        ],
        "读回：单元 1 两条提示各一读都不算数，按映射读第一份对上，单元 2 第一份一读；释放之前按映射核单元 1、2 各两份"
    );
    assert_eq!(
        whole_first_file_read_by_the_mounted_reader(&pool.image(), &after),
        content_with_the_patch(&content, offset, &patch)
    );
    without_the_fault.forget_the_operations();
    without_the_fault
        .write_at_offset(FIRST_FILE, offset, &patch)
        .expect("不注入：部分写发成");
    assert!(
        !reads_of_the_watched_slots_before_the_first_write(
            &without_the_fault.operations(),
            &watched
        )
        .iter()
        .any(|(_, slot)| *slot == SLOT_NO_UNIT_WAS_EVER_WRITTEN_TO),
        "不注入：不读提示之外的槽"
    );
}

/// R7.1 F6：单元 1 的提示过时、中央映射里又没有它那把 key（交给写入口的那一版里它的映射 key 改了一个字节）：两条提示各一读之后部分写失败、
/// 原因是「提示读不出、映射里没有它」，不拿零当旧内容接着写，一个写都不发、盘上逐字节不变；不注入的同一个写发成。
/// 判别力：读不回时拿零基底接着写（`crates/mutations.tsv` 里「m4 部分写 R7 F1」那一行同样让这一条红）。
#[test]
fn a_partial_write_fails_before_any_write_when_the_hint_is_stale_and_the_mapping_has_no_entry_for_the_old_unit(
) {
    let content = content_of_three_units();
    let mut pool = PoolOnObservedDevices::with_the_first_file_written_as(&content);
    let old_slots: Vec<SlotNumber> = (0..3)
        .map(|unit| slot_of_the_data_unit(&pool.output, unit))
        .collect();
    let watched = [SLOT_NO_UNIT_WAS_EVER_WRITTEN_TO, old_slots[1], old_slots[2]];
    let patch = vec![0x7Cu8; 64];
    let offset = payload_capacity() + 100;
    let mut without_the_fault = pool.duplicate();
    for location in &mut pool.output.data_pointers[1].locations {
        location.slot = SLOT_NO_UNIT_WAS_EVER_WRITTEN_TO;
    }
    let (_, mapping_key) = pool
        .output
        .mapped_units
        .iter_mut()
        .find(|(role, _)| *role == TransactionUnit::Data(DataUnitIndexInFile(1)))
        .expect("单元 1 进映射");
    let last_byte = mapping_key.last_mut().expect("映射 key 不空");
    *last_byte ^= 0xFF;
    let image_before = pool.image();
    pool.forget_the_operations();
    let refusal = pool
        .write_at_offset(FIRST_FILE, offset, &patch)
        .expect_err("提示读不出、映射里没有它：部分写失败");
    assert!(
        refused_as_an_old_unit_not_read_back(
            &refusal,
            1,
            OldDataUnitReadBackFailure::UnreadableAtTheHintsAndMissingFromTheMapping
        ),
        "{refusal:?}"
    );
    assert_eq!(
        reads_of_the_watched_slots_before_the_first_write(&pool.operations(), &watched),
        vec![
            (DEVICE_ZERO, SLOT_NO_UNIT_WAS_EVER_WRITTEN_TO),
            (DEVICE_ONE, SLOT_NO_UNIT_WAS_EVER_WRITTEN_TO),
        ],
        "两条提示各一读，映射里查不到就不再读"
    );
    assert!(no_write_was_issued(&pool.operations()), "一个写都没发");
    assert!(pool.image() == image_before, "盘上逐字节不变");
    without_the_fault
        .write_at_offset(FIRST_FILE, offset, &patch)
        .expect("不注入：部分写发成");
}

/// 记下每一读的读者，读点名的偏移时把读回的字节截短（正推 R7.1 F7 (a)「读口子替身返回短 `Vec`」：块设备注入不出短读，这里换掉读口子）。
struct ReaderRecordingEveryReadAndShorteningSome<'image> {
    image: &'image MemoryPool,
    shortened_at: BTreeSet<u64>,
    reads: Mutex<Vec<(DeviceIdentity, DeviceOffsetInBytes)>>,
}

impl PoolReader for ReaderRecordingEveryReadAndShorteningSome<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.image.device_identities()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.image.device_size_in_bytes(device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.reads
            .lock()
            .expect("测试单线程，记录的锁不会中毒")
            .push((device, offset));
        let mut bytes = self.image.read(device, offset, length)?;
        if self.shortened_at.contains(&offset.0) {
            bytes.truncate(length / 2);
        }
        Some(bytes)
    }
    fn journal_record_offsets_hint(
        &self,
        _device: DeviceIdentity,
        _ring_start: DeviceOffsetInBytes,
        _ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        None
    }
}

/// R7.1 F7 (a)：读回旧单元的那几读读回来的字节短了一半（两条提示、映射两条落点都是同一个槽）：整单元校验和对不上，四读之后读回失败、原因是
/// 「提示与映射的落点都读不出」；同一个读者不截短时第一读就读回单元 1 的净荷。
/// 判别力：读回不核整单元校验和时短读被当成读到（`crates/mutations.tsv` 里「m4 部分写 R3：读回旧单元不核整单元 CRC-32C」那一行同样让这一条红）。
#[test]
fn reading_the_old_unit_back_fails_when_every_read_of_it_comes_back_short() {
    let content = content_of_three_units();
    let pool = PoolOnObservedDevices::with_the_first_file_written_as(&content);
    let slot = slot_of_the_data_unit(&pool.output, 1);
    let image = pool.image();
    let record = pool
        .output
        .inode_record
        .expect("这一版有第一个文件那条记录");
    let filesystem_identifier = parameters().filesystem_identifier;
    let shortening = ReaderRecordingEveryReadAndShorteningSome {
        image: &image,
        shortened_at: BTreeSet::from([slot.to_device_offset().0]),
        reads: Mutex::new(Vec::new()),
    };
    let failure = read_back_the_old_payload_of_a_data_unit(
        &shortening,
        &pool.output,
        FIRST_FILE,
        record,
        DataUnitIndexInFile(1),
        &filesystem_identifier,
    )
    .expect_err("每一读都短了：读回失败");
    assert_eq!(
        failure,
        OldDataUnitReadBackFailure::UnreadableAtTheHintsAndAtTheMappedLocations
    );
    assert_eq!(
        shortening
            .reads
            .lock()
            .expect("测试单线程，记录的锁不会中毒")
            .clone(),
        vec![
            (DEVICE_ZERO, slot.to_device_offset()),
            (DEVICE_ONE, slot.to_device_offset()),
            (DEVICE_ZERO, slot.to_device_offset()),
            (DEVICE_ONE, slot.to_device_offset()),
        ],
        "两条提示、映射的两条落点各一读"
    );
    let unharmed = ReaderRecordingEveryReadAndShorteningSome {
        image: &image,
        shortened_at: BTreeSet::new(),
        reads: Mutex::new(Vec::new()),
    };
    let payload = read_back_the_old_payload_of_a_data_unit(
        &unharmed,
        &pool.output,
        FIRST_FILE,
        record,
        DataUnitIndexInFile(1),
        &filesystem_identifier,
    )
    .expect("不截短：读回单元 1");
    let unit_start = usize::try_from(payload_capacity()).expect("净荷");
    assert_eq!(payload, content[unit_start..2 * unit_start].to_vec());
    assert_eq!(
        unharmed
            .reads
            .lock()
            .expect("测试单线程，记录的锁不会中毒")
            .len(),
        1,
        "不截短：第一读就对上"
    );
}

/// R7.1 F7 (b)：单元 1 盘上那两份的声明长度改成少一个字节、载荷校验和与头校验和照改过的字节重封，数据指针的整单元校验和也换成改过的
/// （单元自洽、指针对得上）：读回第一读就对上校验和，单元头对查找路径时声明长度与按写之前 size 算的不符，部分写失败、一个写都不发、盘上逐字节不变；
/// 不改的同一个写发成。判别力：单元头不对声明长度时这一条红（`crates/mutations.tsv` 一行）。
#[test]
fn a_partial_write_fails_before_any_write_when_the_old_unit_declares_a_length_the_size_does_not_give(
) {
    let content = content_of_three_units();
    let mut pool = PoolOnObservedDevices::with_the_first_file_written_as(&content);
    let old_slots: Vec<SlotNumber> = (0..3)
        .map(|unit| slot_of_the_data_unit(&pool.output, unit))
        .collect();
    let patch = vec![0x2Du8; 64];
    let offset = payload_capacity() + 100;
    let mut without_the_fault = pool.duplicate();
    let mut forged = pool.data_unit_bytes_at(old_slots[1]);
    let declared_length = u16::from_le_bytes([forged[8], forged[9]]);
    forged[8..10].copy_from_slice(&(declared_length - 1).to_le_bytes());
    reseal_data_unit(&mut forged);
    pool.overwrite_the_data_unit_on_both_devices(old_slots[1], &forged);
    point_the_data_unit_at_bytes(&mut pool.output, 1, &forged);
    let image_before = pool.image();
    pool.forget_the_operations();
    let refusal = pool
        .write_at_offset(FIRST_FILE, offset, &patch)
        .expect_err("声明长度与 size 不符：部分写失败");
    assert!(
        refused_as_an_old_unit_not_read_back(
            &refusal,
            1,
            OldDataUnitReadBackFailure::DisagreesWithTheLookupPath {
                detail: "声明长度与写之前的 size 按净荷算出来的不符",
            }
        ),
        "{refusal:?}"
    );
    assert_eq!(
        reads_of_the_watched_slots_before_the_first_write(&pool.operations(), &old_slots),
        vec![(DEVICE_ZERO, old_slots[1])],
        "第一读就对上整单元校验和，单元头不对查找路径就拒"
    );
    assert!(no_write_was_issued(&pool.operations()), "一个写都没发");
    assert!(pool.image() == image_before, "盘上逐字节不变");
    without_the_fault
        .write_at_offset(FIRST_FILE, offset, &patch)
        .expect("不改：部分写发成");
}

/// R7.1 F7 (c)：单元 2（声明长度比净荷容量少 7 字节）盘上那两份的末尾那一个补齐字节改成非 0、两样校验和重封、数据指针跟着换：读回第一读对上校验和、
/// 单元头也对得上查找路径，补齐字节非 0（I-2.3）就拒，部分写失败、一个写都不发、盘上逐字节不变；不改的同一个写发成。
/// 判别力：读回不核补齐字节时这一条红（`crates/mutations.tsv` 一行）。
#[test]
fn a_partial_write_fails_before_any_write_when_the_old_unit_has_a_non_zero_padding_byte() {
    let content = content_of_three_units();
    let mut pool = PoolOnObservedDevices::with_the_first_file_written_as(&content);
    let old_slots: Vec<SlotNumber> = (0..3)
        .map(|unit| slot_of_the_data_unit(&pool.output, unit))
        .collect();
    let patch = vec![0x3Eu8; 64];
    let offset = 2 * payload_capacity() + 100;
    let mut without_the_fault = pool.duplicate();
    let mut forged = pool.data_unit_bytes_at(old_slots[2]);
    let last = forged.len() - 1;
    forged[last] = 0x01;
    reseal_data_unit(&mut forged);
    pool.overwrite_the_data_unit_on_both_devices(old_slots[2], &forged);
    point_the_data_unit_at_bytes(&mut pool.output, 2, &forged);
    let image_before = pool.image();
    pool.forget_the_operations();
    let refusal = pool
        .write_at_offset(FIRST_FILE, offset, &patch)
        .expect_err("补齐字节非 0：部分写失败");
    assert!(
        refused_as_an_old_unit_not_read_back(
            &refusal,
            2,
            OldDataUnitReadBackFailure::DisagreesWithTheLookupPath {
                detail: "声明长度之后的补齐字节非零",
            }
        ),
        "{refusal:?}"
    );
    assert_eq!(
        reads_of_the_watched_slots_before_the_first_write(&pool.operations(), &old_slots),
        vec![(DEVICE_ZERO, old_slots[2])],
        "只重写单元 2：第一读就对上整单元校验和，补齐字节非 0 就拒"
    );
    assert!(no_write_was_issued(&pool.operations()), "一个写都没发");
    assert!(pool.image() == image_before, "盘上逐字节不变");
    without_the_fault
        .write_at_offset(FIRST_FILE, offset, &patch)
        .expect("不改：部分写发成");
}

/// R7.1 F11：说谎的设备吞了上一次部分写的新单元 1（两块盘上那两槽的写报成功、一个字节都没落），接着再对单元 1 部分写：新指针点名的槽上是那两槽
/// 原来的字节、整单元校验和对不上，两条提示、映射两条落点各一读之后失败，一个写都不发；不吞的同一对写第二次照常发成。读内存里字节的读法在这一格
/// 会成（内存里是新字节），读盘的分得开（R3）。判别力：读—改—写不读回旧单元、拿零当旧内容时第二次照样发成（`crates/mutations.tsv` 一行）。
#[test]
fn a_second_partial_write_fails_when_a_lying_device_swallowed_the_first_one() {
    let content = content_of_three_units();
    let mut pool = PoolOnObservedDevices::with_the_first_file_written_as(&content);
    let first_patch = vec![0x11u8; 4096];
    let second_patch = vec![0x22u8; 64];
    let offset = payload_capacity() + 100;
    let mut without_the_fault = pool.duplicate();
    let after_the_first = without_the_fault
        .write_at_offset(FIRST_FILE, offset, &first_patch)
        .expect("不吞：第一次部分写发成");
    let new_slot_of_unit_one = slot_of_the_data_unit(&after_the_first, 1);
    let swallowed = new_slot_of_unit_one.to_device_offset().0
        ..new_slot_of_unit_one.to_device_offset().0 + DATA_UNIT_SLOTS * SLOT_BYTES;
    for device in [DEVICE_ZERO, DEVICE_ONE] {
        pool.device_mut(device)
            .faults
            .writes_swallowed_over
            .push(swallowed.clone());
    }
    let swallowed_first = pool
        .write_at_offset(FIRST_FILE, offset, &first_patch)
        .expect("吞写的设备报成功：第一次部分写发成");
    assert_eq!(
        slot_of_the_data_unit(&swallowed_first, 1),
        new_slot_of_unit_one,
        "两份同一刻的池上第一次写分到同一个槽"
    );
    let image_before = pool.image();
    pool.forget_the_operations();
    let refusal = pool
        .write_at_offset(FIRST_FILE, offset + 100, &second_patch)
        .expect_err("新单元 1 被吞了：第二次部分写失败");
    assert!(
        refused_as_an_old_unit_not_read_back(
            &refusal,
            1,
            OldDataUnitReadBackFailure::UnreadableAtTheHintsAndAtTheMappedLocations
        ),
        "{refusal:?}"
    );
    assert_eq!(
        reads_of_the_watched_slots_before_the_first_write(
            &pool.operations(),
            &[new_slot_of_unit_one]
        ),
        vec![
            (DEVICE_ZERO, new_slot_of_unit_one),
            (DEVICE_ONE, new_slot_of_unit_one),
            (DEVICE_ZERO, new_slot_of_unit_one),
            (DEVICE_ONE, new_slot_of_unit_one),
        ],
        "两条提示、映射的两条落点各一读"
    );
    assert!(no_write_was_issued(&pool.operations()), "一个写都没发");
    assert!(pool.image() == image_before, "盘上逐字节不变");
    let after_the_second = without_the_fault
        .write_at_offset(FIRST_FILE, offset + 100, &second_patch)
        .expect("不吞：第二次部分写发成");
    assert_eq!(
        whole_first_file_read_by_the_mounted_reader(&without_the_fault.image(), &after_the_second),
        content_with_the_patch(
            &content_with_the_patch(&content, offset, &first_patch),
            offset + 100,
            &second_patch
        )
    );
}
