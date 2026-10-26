//! 读—改—写读回旧单元读盘（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R3、R7；D4（校验和位置） 已定项 3、4；
//! C57（零填充顶掉原文））：部分写与截断、变长重写尾单元都读盘读回旧单元（提示位置、再中央映射，整单元 CRC-32C 与单元头对查找路径），
//! 读不回或校验不过整个写失败、盘上逐字节不变；不注入故障的同一个写发成。每一格旁边钉落在数据单元槽上的读序列（第一个写之前）。
//! 盘是两块内存稀疏盘，外面包一层记下整个池每一次读写、按偏移注入读故障的盘（读故障不改盘上的字节）。
#![allow(
    dead_code,
    reason = "三份部分写测试各抄一份同样的脚手架（不另开共用模块），各自只用到其中一部分"
)]

mod common;

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::DataUnitIndexInFile;
use singlefs_core::address::{
    DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber, InstanceGeneration,
    SlotNumber,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::open_pool_for_read;
use singlefs_core::transaction::{
    acquire_instance, publish_create_snapshot, publish_first_file, publish_new_inodes,
    publish_partial_write, publish_sequential_write, publish_sequential_write_of_an_inode,
    publish_size_change, warm_up, FirstFile, PartialWriteBytes, PoolWriter, PublishError,
    ResidueReadsOfTheWriter, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::transaction::{OldDataUnitReadBackFailure, TruncationRefusal};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

/// 块层上的一次读或写，按整个池一份次序记：读记盘、偏移、长度，写记盘与偏移。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BlockOperation {
    Read {
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length_in_bytes: usize,
    },
    Write {
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
    },
}

/// 读这块盘时注入的故障（写不受影响，盘上字节不变）：从点名的偏移起的那一读报读错，或读回来的最后一个字节翻转（整单元 CRC-32C 就对不上）。
#[derive(Clone, Debug, Default)]
struct ReadFaults {
    reads_failing_at: BTreeSet<u64>,
    reads_corrupted_at: BTreeSet<u64>,
}

/// 内存稀疏盘外面包一层：每一次读写按整个池一份次序记下来，读按 [`ReadFaults`] 注入故障。
struct ObservedDevice {
    identity: DeviceIdentity,
    disk: SparseBlockDevice,
    operations: Arc<Mutex<Vec<BlockOperation>>>,
    faults: ReadFaults,
}

impl ObservedDevice {
    fn record(&self, operation: BlockOperation) {
        self.operations
            .lock()
            .expect("测试单线程，记录的锁不会中毒")
            .push(operation);
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
            length_in_bytes: buffer.len(),
        });
        if self.faults.reads_failing_at.contains(&offset.0) {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "注入的读错",
            )));
        }
        self.disk.read_at(offset, buffer)?;
        if self.faults.reads_corrupted_at.contains(&offset.0) {
            if let Some(last_byte) = buffer.last_mut() {
                *last_byte ^= 0xFF;
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
        self.record(BlockOperation::Write {
            device: self.identity,
            offset,
        });
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
    instance: InstanceGeneration,
}

impl PoolOnObservedDevices {
    /// mkfs → 取号 → 暖机 → 新池新建文件（3000 字节的第一个文件），两块 4 GiB 内存稀疏盘。
    fn with_the_first_file() -> Self {
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
                        faults: ReadFaults::default(),
                    },
                )
            })
            .collect();
        let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
        let mut allocator = PoolAllocator::new(vec![
            DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
            DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
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
        let content = file_content();
        let output = {
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            let instance = acquire_instance(&mut writer).expect("取号");
            assert_eq!(instance, INSTANCE);
            let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
            publish_first_file(
                &mut writer,
                &mut allocator,
                warmed.roots.last().expect("暖机两代根"),
                FirstFile {
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS,
                },
                instance,
                &warmed.last_record_bytes,
            )
            .expect("新池新建文件")
        };
        PoolOnObservedDevices {
            devices,
            allocator,
            output,
            operations,
            next_write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            instance: INSTANCE,
        }
    }

    /// 第一个文件整份顺序写成 `content`。
    fn with_the_first_file_written_as(content: &[u8]) -> Self {
        let mut pool = PoolOnObservedDevices::with_the_first_file();
        let parameters = parameters();
        let write_time_seconds = pool.next_write_time();
        let previous = pool.output.clone();
        let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
        pool.output = publish_sequential_write(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile {
                content,
                write_time_seconds,
            },
            pool.instance,
        )
        .expect("顺序写");
        pool
    }

    fn next_write_time(&mut self) -> u64 {
        self.next_write_time_seconds += 1;
        self.next_write_time_seconds
    }

    /// 同一刻的一份拷贝（盘的字节、分配器、现行那一版），读写记录另起一份。
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
                            faults: device.faults.clone(),
                        },
                    )
                })
                .collect(),
            allocator: self.allocator.clone(),
            output: self.output.clone(),
            operations,
            next_write_time_seconds: self.next_write_time_seconds,
            instance: self.instance,
        }
    }

    /// 两块盘此刻的字节（池级 checker 与读者读的镜像）。
    fn image(&self) -> MemoryPool {
        image_of(&self.devices)
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

    fn faults_of(&mut self, device: DeviceIdentity) -> &mut ReadFaults {
        &mut self
            .devices
            .iter_mut()
            .find(|(identity, _)| *identity == device)
            .expect("池里有这块盘")
            .1
            .faults
    }

    /// 接着现行那一版按偏移写一次；发成了池接到它那一版上。交回发布的结局与这个写入口上读残留的账。
    fn write_at_offset(
        &mut self,
        inode: InodeNumber,
        offset: u64,
        bytes: &[u8],
    ) -> (
        Result<TransactionOutput, PublishError>,
        ResidueReadsOfTheWriter,
    ) {
        let parameters = parameters();
        let write_time_seconds = self.next_write_time();
        let previous = self.output.clone();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        let published = publish_partial_write(
            &mut writer,
            &mut self.allocator,
            &previous,
            inode,
            FileOffsetInBytes(offset),
            PartialWriteBytes::new(bytes).expect("用例写的字节不是空的"),
            write_time_seconds,
            self.instance,
        );
        let residue_reads = writer.residue_reads();
        if let Ok(version) = &published {
            self.output = version.clone();
        }
        (published, residue_reads)
    }
}

/// `version` 里 `inode` 那个文件第 `unit` 个数据单元的槽（两块盘上同一个槽号）。
fn slot_of_the_data_unit(version: &TransactionOutput, inode: InodeNumber, unit: u64) -> SlotNumber {
    let position = usize::try_from(unit).expect("单元序号");
    let pointer = if inode.0 == FIRST_INODE_NUMBER {
        version.data_pointers[position]
    } else {
        version
            .file_other_than_the_first(inode)
            .expect("这一版里有这个文件")
            .data_pointers[position]
    };
    assert_eq!(
        pointer.locations[0].slot, pointer.locations[1].slot,
        "两块盘同一个槽号"
    );
    pointer.locations[0].slot
}

/// 第一个写之前、落在 `watched` 这几个槽上的读，按次序交回（盘、槽）。读其它角色（树节点、系统配置、根环）的不在里面。
fn reads_of_the_watched_slots_before_the_first_write(
    operations: &[BlockOperation],
    watched: &[SlotNumber],
) -> Vec<(DeviceIdentity, SlotNumber)> {
    let mut reads = Vec::new();
    for operation in operations {
        match *operation {
            BlockOperation::Write { .. } => return reads,
            BlockOperation::Read { device, offset, .. } => {
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

/// 挂载态读者按 `version` 的根打开 `inode`、从 0 读到 size。
fn whole_file_read_by_the_mounted_reader(
    image: &MemoryPool,
    version: &TransactionOutput,
    inode: InodeNumber,
) -> Vec<u8> {
    let mounted = open_pool_for_read(image, &version.root).expect("打开挂载态");
    let file = mounted.open_file(image, inode).expect("打开文件");
    let size = file.inode_record().size;
    file.read_at(image, FileOffsetInBytes(0), size)
        .expect("从 0 读到 size")
        .bytes
}

/// 池级 checker 判这张镜像，违例的那几条不变量。
fn invariants_violated(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(name, verdict)| match verdict {
            InvariantVerdict::Violated(reason) => Some((name, reason)),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

const DEVICE_ZERO: DeviceIdentity = DeviceIdentity(0);
const DEVICE_ONE: DeviceIdentity = DeviceIdentity(1);

impl PoolOnObservedDevices {
    /// 接着现行那一版改 `inode` 的 size（N > S 是变长、N < S 是截断的意图那次）；发成了池接到它那一版上。
    fn change_the_size(
        &mut self,
        inode: InodeNumber,
        new_size: u64,
    ) -> Result<TransactionOutput, PublishError> {
        let parameters = parameters();
        let write_time_seconds = self.next_write_time();
        let previous = self.output.clone();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        let published = publish_size_change(
            &mut writer,
            &mut self.allocator,
            &previous,
            inode,
            new_size,
            write_time_seconds,
            self.instance,
        );
        if let Ok(version) = &published {
            self.output = version.clone();
        }
        published
    }

    /// 建一个 size 0、一个数据单元都没有的 inode，交回它的号。
    fn create_one_inode(&mut self) -> InodeNumber {
        let parameters = parameters();
        let write_time_seconds = self.next_write_time();
        let previous = self.output.clone();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        self.output = publish_new_inodes(
            &mut writer,
            &mut self.allocator,
            &previous,
            1,
            write_time_seconds,
            self.instance,
        )
        .expect("建 inode");
        let created = self
            .output
            .inode_leaf_container_contents()
            .iter()
            .flat_map(|container| container.records.iter())
            .map(|record| record.inode)
            .max()
            .expect("建过之后树里有号");
        InodeNumber(created)
    }

    /// 按 inode 号整份顺序写 `content`。
    fn sequential_write_of_the_inode(&mut self, inode: InodeNumber, content: &[u8]) {
        let parameters = parameters();
        let write_time_seconds = self.next_write_time();
        let previous = self.output.clone();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        self.output = publish_sequential_write_of_an_inode(
            &mut writer,
            &mut self.allocator,
            &previous,
            inode,
            content,
            write_time_seconds,
            self.instance,
        )
        .expect("按 inode 号顺序写");
    }

    /// 建一个只读快照。
    fn create_a_snapshot(&mut self) {
        let parameters = parameters();
        let previous = self.output.clone();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        self.output =
            publish_create_snapshot(&mut writer, &mut self.allocator, &previous, self.instance)
                .expect("建快照");
    }

    /// 在这两块盘上开一次可写挂载（不卸载就重开，同一个进程里接着用）：之后的发布接在挂载认下的现行那一版、用它的分配器与新实例。
    fn mounted_writable(mut self) -> Self {
        let parameters = parameters();
        let mounted = mount_writable(&parameters, &mut self.devices).expect("可写挂载");
        self.output = mounted
            .current
            .file_version()
            .expect("现行那一版带文件")
            .clone();
        self.allocator = mounted.allocator;
        self.instance = mounted.output.instance;
        self
    }
}

fn image_of(devices: &[(DeviceIdentity, ObservedDevice)]) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.disk.image.clone()))
            .collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);

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

/// 没注入故障（R7.1 F8）：三个单元的第一个文件在单元 1 中间写 4 KiB，单元 1、2 一起重写（R4），读回旧单元读盘（R3）：单元 1、2 各在第一份上读一次就对上，
/// 释放之前再核两个单元各两份（R7.2「读回次数不能只数某地址被读几次」）；一次残留都不读；挂载态读者读回改过的内容，池级 checker 一条都不红。
#[test]
fn an_unharmed_partial_write_in_the_middle_unit_reads_each_old_unit_back_once_before_the_release_check(
) {
    let content = content_of_three_units();
    let mut pool = PoolOnObservedDevices::with_the_first_file_written_as(&content);
    let old_slots: Vec<SlotNumber> = (0..3)
        .map(|unit| slot_of_the_data_unit(&pool.output, FIRST_FILE, unit))
        .collect();
    let patch = vec![0xC3u8; 4096];
    let offset = payload_capacity() + 100;
    pool.forget_the_operations();
    let (published, residue_reads) = pool.write_at_offset(FIRST_FILE, offset, &patch);
    let after = published.expect("没有注入故障：部分写发成");
    assert_eq!(
        reads_of_the_watched_slots_before_the_first_write(&pool.operations(), &old_slots),
        vec![
            (DEVICE_ZERO, old_slots[1]),
            (DEVICE_ZERO, old_slots[2]),
            (DEVICE_ZERO, old_slots[1]),
            (DEVICE_ONE, old_slots[1]),
            (DEVICE_ZERO, old_slots[2]),
            (DEVICE_ONE, old_slots[2]),
        ],
        "读回单元 1、2 各一读（第一份就对上，单元 0 不读），释放之前核单元 1、2 的两份"
    );
    assert_eq!(residue_reads, ResidueReadsOfTheWriter::NONE);
    let image = pool.image();
    assert_eq!(
        whole_file_read_by_the_mounted_reader(&image, &after, FIRST_FILE),
        content_with_the_patch(&content, offset, &patch)
    );
    let violated = invariants_violated(&image);
    assert!(violated.is_empty(), "池级 checker：{violated:?}");
}

/// C57（零填充顶掉原文）与 R3「读盘」：可写挂载之后、不重挂，单元 1 两份都读成坏字节（整单元校验和对不上），对它部分写失败：读两条提示、再按中央映射读两份，
/// 交回读回旧单元那一个成员，一个写都不发、盘上逐字节不变（D4（校验和位置） 已定项 4：不拿零填充凑一个合法的零）。
#[test]
fn a_partial_write_fails_before_any_write_when_both_copies_of_the_old_unit_turn_corrupt_after_the_writable_mount(
) {
    let content = content_of_three_units();
    let mut pool =
        PoolOnObservedDevices::with_the_first_file_written_as(&content).mounted_writable();
    let old_unit_one = slot_of_the_data_unit(&pool.output, FIRST_FILE, 1);
    for device in [DEVICE_ZERO, DEVICE_ONE] {
        pool.faults_of(device)
            .reads_corrupted_at
            .insert(old_unit_one.to_device_offset().0);
    }
    let image_before = pool.image();
    pool.forget_the_operations();
    let (published, _) = pool.write_at_offset(FIRST_FILE, payload_capacity() + 100, &[0xC3; 4096]);
    let refusal = published.expect_err("旧单元两份都坏：部分写失败");
    assert!(
        matches!(
            refusal,
            PublishError::TruncationRefused(TruncationRefusal::OldDataUnitToRewriteNotReadBack {
                inode: FIRST_FILE,
                unit: DataUnitIndexInFile(1),
                read_back: OldDataUnitReadBackFailure::UnreadableAtTheHintsAndAtTheMappedLocations,
            })
        ),
        "{refusal:?}"
    );
    assert_eq!(
        reads_of_the_watched_slots_before_the_first_write(&pool.operations(), &[old_unit_one]),
        vec![
            (DEVICE_ZERO, old_unit_one),
            (DEVICE_ONE, old_unit_one),
            (DEVICE_ZERO, old_unit_one),
            (DEVICE_ONE, old_unit_one),
        ],
        "两条提示各一读，中央映射的两条落点各一读"
    );
    assert!(
        pool.operations()
            .iter()
            .all(|operation| matches!(operation, BlockOperation::Read { .. })),
        "一个写都没发"
    );
    assert!(pool.image() == image_before, "盘上逐字节不变");
}

/// R7.1 F4：单元 1 只有第一份读成坏字节：读第二份对上，部分写发成；释放之前核出第一份对不上（重读一次仍不对），两份都留在已分配、照常发成。
#[test]
fn a_partial_write_reads_the_second_copy_when_only_the_first_copy_of_the_old_unit_is_corrupt() {
    let content = content_of_three_units();
    let mut pool = PoolOnObservedDevices::with_the_first_file_written_as(&content);
    let old_slots: Vec<SlotNumber> = (0..3)
        .map(|unit| slot_of_the_data_unit(&pool.output, FIRST_FILE, unit))
        .collect();
    pool.faults_of(DEVICE_ZERO)
        .reads_corrupted_at
        .insert(old_slots[1].to_device_offset().0);
    let patch = vec![0x3Cu8; 4096];
    let offset = payload_capacity() + 100;
    pool.forget_the_operations();
    let (published, _) = pool.write_at_offset(FIRST_FILE, offset, &patch);
    let after = published.expect("只坏一份：读第二份，部分写发成");
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
        "读回：单元 1 第一份坏、第二份对上，单元 2 一读；释放之前：单元 1 第一份核出坏、重读一次，第二份一读，单元 2 两份"
    );
    assert_eq!(
        whole_file_read_by_the_mounted_reader(&pool.image(), &after, FIRST_FILE),
        content_with_the_patch(&content, offset, &patch)
    );
}

/// R7.1 F1：单元 1 两份都读不出（读错）：提示两读、映射两读之后部分写失败，一个写都不发。
#[test]
fn a_partial_write_fails_before_any_write_when_both_copies_of_the_old_unit_are_unreadable() {
    let mut pool = PoolOnObservedDevices::with_the_first_file_written_as(&content_of_three_units());
    let old_unit_one = slot_of_the_data_unit(&pool.output, FIRST_FILE, 1);
    for device in [DEVICE_ZERO, DEVICE_ONE] {
        pool.faults_of(device)
            .reads_failing_at
            .insert(old_unit_one.to_device_offset().0);
    }
    let image_before = pool.image();
    pool.forget_the_operations();
    let (published, _) = pool.write_at_offset(FIRST_FILE, payload_capacity() + 100, &[0x11; 64]);
    let refusal = published.expect_err("旧单元两份都读不出：部分写失败");
    assert!(
        matches!(
            refusal,
            PublishError::TruncationRefused(TruncationRefusal::OldDataUnitToRewriteNotReadBack {
                inode: FIRST_FILE,
                unit: DataUnitIndexInFile(1),
                read_back: OldDataUnitReadBackFailure::UnreadableAtTheHintsAndAtTheMappedLocations,
            })
        ),
        "{refusal:?}"
    );
    assert_eq!(
        reads_of_the_watched_slots_before_the_first_write(&pool.operations(), &[old_unit_one])
            .len(),
        4,
        "两条提示、两条映射落点各一读"
    );
    assert!(pool.image() == image_before, "盘上逐字节不变");
}

/// 截断尾单元那条路一起改成读盘（判决第二节 R3）：3000 字节的第一个文件变长到 5000，单元 0 读盘读回一次（第一份），释放之前核两份；
/// 读出原内容后面补零。
#[test]
fn extending_the_first_file_reads_its_last_unit_back_from_the_disk_once_before_the_release_check() {
    let mut pool = PoolOnObservedDevices::with_the_first_file();
    let old_unit_zero = slot_of_the_data_unit(&pool.output, FIRST_FILE, 0);
    pool.forget_the_operations();
    let extended = pool
        .change_the_size(FIRST_FILE, 5000)
        .expect("没有注入故障：变长发成");
    assert_eq!(
        reads_of_the_watched_slots_before_the_first_write(&pool.operations(), &[old_unit_zero]),
        vec![
            (DEVICE_ZERO, old_unit_zero),
            (DEVICE_ZERO, old_unit_zero),
            (DEVICE_ONE, old_unit_zero),
        ],
        "读回尾单元一读，释放之前核两份"
    );
    let mut expected = file_content();
    expected.resize(5000, 0);
    assert_eq!(
        whole_file_read_by_the_mounted_reader(&pool.image(), &extended, FIRST_FILE),
        expected
    );
}

/// 截断尾单元读盘（判决第二节 R3）：可写挂载之后、不重挂，单元 1 两份都读成坏字节，截到单元 1 中间（要重写它）在任何写之前失败，盘上逐字节不变
/// （改之前读内存里的字节，照样发成）。
#[test]
fn truncating_into_a_unit_whose_both_copies_turn_corrupt_after_the_writable_mount_fails_before_any_write(
) {
    let mut pool = PoolOnObservedDevices::with_the_first_file_written_as(&content_of_three_units())
        .mounted_writable();
    let old_unit_one = slot_of_the_data_unit(&pool.output, FIRST_FILE, 1);
    for device in [DEVICE_ZERO, DEVICE_ONE] {
        pool.faults_of(device)
            .reads_corrupted_at
            .insert(old_unit_one.to_device_offset().0);
    }
    let image_before = pool.image();
    pool.forget_the_operations();
    let refusal = pool
        .change_the_size(FIRST_FILE, payload_capacity() + 100)
        .expect_err("要重写的尾单元两份都坏：截断失败");
    assert!(
        matches!(
            refusal,
            PublishError::TruncationRefused(TruncationRefusal::OldDataUnitToRewriteNotReadBack {
                inode: FIRST_FILE,
                unit: DataUnitIndexInFile(1),
                read_back: OldDataUnitReadBackFailure::UnreadableAtTheHintsAndAtTheMappedLocations,
            })
        ),
        "{refusal:?}"
    );
    assert!(
        pool.operations()
            .iter()
            .all(|operation| matches!(operation, BlockOperation::Read { .. })),
        "一个写都没发"
    );
    assert!(pool.image() == image_before, "盘上逐字节不变");
}
