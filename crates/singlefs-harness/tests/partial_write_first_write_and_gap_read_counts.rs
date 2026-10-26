//! 首次写读残留与空隙补零（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R2、R5；C385（首次写要不要读回，两条已定项说反话）
//! 用户 2026-10-26 定「首次写也读回」「残留读出错记数、不拒写」）：没有旧单元的单元读这次发布分到的新槽残留一次、字节丢掉、旧内容按零，
//! 空隙补整单元零、不读；写跨末尾、写进洞里换 size 与标签。读回次数钉成落在数据单元槽上的读序列（第一个写之前）的绝对值。
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

/// `bytes` 前面补 `zeros_before` 个零、后面补零到 `size`。
fn zeros_then_the_bytes(zeros_before: u64, bytes: &[u8], size: u64) -> Vec<u8> {
    let mut content = vec![0u8; usize::try_from(zeros_before).expect("长度")];
    content.extend_from_slice(bytes);
    content.resize(usize::try_from(size).expect("长度"), 0);
    content
}

/// 先在同一刻的一份拷贝上照样写一次，取 `inode` 第 `unit` 个单元分到的槽（分配器对同一份输入取同一个槽）。
fn slot_the_write_lands_on(
    pool: &PoolOnObservedDevices,
    inode: InodeNumber,
    offset: u64,
    bytes: &[u8],
    unit: u64,
) -> SlotNumber {
    let mut rehearsal = pool.duplicate();
    let (published, _) = rehearsal.write_at_offset(inode, offset, bytes);
    slot_of_the_data_unit(&published.expect("拷贝上照样写发成"), inode, unit)
}

/// C385（首次写要不要读回，用户 2026-10-26 定「首次写也读回」）落在 0 个单元的 inode 上（判决第二节 R2 攻方 ①）：新建的 inode 写 4 KiB，
/// 落在数据单元槽上的读恰好一次——这次给它分到的新槽、第一条位置项那块盘（残留，字节丢掉）；第一个文件的单元不读；读残留的账一次、没读错。
#[test]
fn a_four_kibibyte_write_into_an_inode_without_data_units_reads_only_the_residue_of_its_new_slot() {
    let mut pool = PoolOnObservedDevices::with_the_first_file();
    let inode = pool.create_one_inode();
    let first_file_unit = slot_of_the_data_unit(&pool.output, FIRST_FILE, 0);
    let bytes = vec![0x5Au8; 4096];
    pool.forget_the_operations();
    let (published, residue_reads) = pool.write_at_offset(inode, 0, &bytes);
    let after = published.expect("首次写发成");
    let new_slot = slot_of_the_data_unit(&after, inode, 0);
    assert_eq!(
        reads_of_the_watched_slots_before_the_first_write(
            &pool.operations(),
            &[first_file_unit, new_slot]
        ),
        vec![(DEVICE_ZERO, new_slot)],
        "读回次数 1：新槽的残留一读，没有旧单元可读、没有释放核"
    );
    assert_eq!(
        residue_reads,
        ResidueReadsOfTheWriter {
            issued: 1,
            failed: 0
        }
    );
    let image = pool.image();
    assert_eq!(
        whole_file_read_by_the_mounted_reader(&image, &after, inode),
        bytes
    );
    let violated = invariants_violated(&image);
    assert!(violated.is_empty(), "池级 checker：{violated:?}");
}

/// 残留字节一个都不进新单元（判决第二节 R2：残留常是同一文件放掉的旧单元）：这次要分到的那个槽两块盘上先写满 0xA5，再往新建的 inode 偏移 100 写
/// 10 字节——读出前 100 字节是零，补齐为零（I-2.3，池级 checker 不红）。
#[test]
fn a_poisoned_new_slot_leaves_no_residue_byte_in_the_new_data_unit() {
    let mut pool = PoolOnObservedDevices::with_the_first_file();
    let inode = pool.create_one_inode();
    let bytes = [0x5Au8; 10];
    let slot = slot_the_write_lands_on(&pool, inode, 100, &bytes, 0);
    let poison = vec![0xA5u8; 32768];
    for (_, device) in &mut pool.devices {
        device.disk.image.write(slot.to_device_offset(), &poison);
    }
    let (published, residue_reads) = pool.write_at_offset(inode, 100, &bytes);
    let after = published.expect("首次写发成");
    assert_eq!(
        slot_of_the_data_unit(&after, inode, 0),
        slot,
        "拷贝上分到的就是这一个槽"
    );
    assert_eq!(residue_reads.issued, 1);
    let image = pool.image();
    assert_eq!(
        whole_file_read_by_the_mounted_reader(&image, &after, inode),
        zeros_then_the_bytes(100, &bytes, 110)
    );
    let violated = invariants_violated(&image);
    assert!(violated.is_empty(), "池级 checker：{violated:?}");
}

/// R7.1 F9：新槽的残留那一读报读错——用户 2026-10-26 定「残留读出错记数、不拒写」：写发成，读残留的账一次、读错一次。
#[test]
fn a_failing_residue_read_is_counted_and_the_first_write_still_succeeds() {
    let mut pool = PoolOnObservedDevices::with_the_first_file();
    let inode = pool.create_one_inode();
    let bytes = [0x77u8; 10];
    let slot = slot_the_write_lands_on(&pool, inode, 0, &bytes, 0);
    pool.faults_of(DEVICE_ZERO)
        .reads_failing_at
        .insert(slot.to_device_offset().0);
    let (published, residue_reads) = pool.write_at_offset(inode, 0, &bytes);
    let after = published.expect("残留读出错不拒写");
    assert_eq!(
        residue_reads,
        ResidueReadsOfTheWriter {
            issued: 1,
            failed: 1
        }
    );
    assert_eq!(
        whole_file_read_by_the_mounted_reader(&pool.image(), &after, inode),
        bytes.to_vec()
    );
}

/// 判决第二节 R5：起点越过末尾时前面的空隙补整单元零、不读：新建的 inode 在第三个单元里写 10 字节，三个单元，落在数据单元槽上的读只有单元 2 的残留一读；
/// 读出前面全零。
#[test]
fn a_write_past_a_gap_zeroes_the_gap_units_without_reading_them() {
    let mut pool = PoolOnObservedDevices::with_the_first_file();
    let inode = pool.create_one_inode();
    let bytes = [0x42u8; 10];
    let offset = 2 * payload_capacity() + 5;
    pool.forget_the_operations();
    let (published, residue_reads) = pool.write_at_offset(inode, offset, &bytes);
    let after = published.expect("跨空隙的写发成");
    let new_slots: Vec<SlotNumber> = (0..3)
        .map(|unit| slot_of_the_data_unit(&after, inode, unit))
        .collect();
    assert_eq!(
        reads_of_the_watched_slots_before_the_first_write(&pool.operations(), &new_slots),
        vec![(DEVICE_ZERO, new_slots[2])],
        "空隙的两个单元不读，写到的单元 2 读残留一次"
    );
    assert_eq!(residue_reads.issued, 1);
    let image = pool.image();
    assert_eq!(
        whole_file_read_by_the_mounted_reader(&image, &after, inode),
        zeros_then_the_bytes(offset, &bytes, offset + 10)
    );
    let violated = invariants_violated(&image);
    assert!(violated.is_empty(), "池级 checker：{violated:?}");
}

/// 判决第二节 R5：写跨末尾同时是一次变长——3000 字节的第一个文件从 2000 起写 40000 字节，size 换成 42000、两个单元；单元 0 读回旧单元
/// （没盖满，读盘一次、释放之前核两份），新加的单元 1 读残留一次；读出前 2000 字节是原内容。
#[test]
fn a_write_across_the_end_of_the_file_extends_it_reading_the_old_last_unit_back() {
    let mut pool = PoolOnObservedDevices::with_the_first_file();
    let old_unit_zero = slot_of_the_data_unit(&pool.output, FIRST_FILE, 0);
    let bytes = vec![0x66u8; 40000];
    pool.forget_the_operations();
    let (published, residue_reads) = pool.write_at_offset(FIRST_FILE, 2000, &bytes);
    let after = published.expect("跨末尾的写发成");
    assert_eq!(after.data_pointers.len(), 2);
    let new_unit_one = slot_of_the_data_unit(&after, FIRST_FILE, 1);
    assert_eq!(
        reads_of_the_watched_slots_before_the_first_write(
            &pool.operations(),
            &[old_unit_zero, new_unit_one]
        ),
        vec![
            (DEVICE_ZERO, old_unit_zero),
            (DEVICE_ZERO, old_unit_zero),
            (DEVICE_ONE, old_unit_zero),
            (DEVICE_ZERO, new_unit_one),
        ],
        "单元 0 读回一读、释放之前核两份；单元 1 读残留一读"
    );
    assert_eq!(residue_reads.issued, 1);
    let mut expected = file_content()[..2000].to_vec();
    expected.extend_from_slice(&bytes);
    let image = pool.image();
    assert_eq!(
        whole_file_read_by_the_mounted_reader(&image, &after, FIRST_FILE),
        expected
    );
    let violated = invariants_violated(&image);
    assert!(violated.is_empty(), "池级 checker：{violated:?}");
}

/// 判决第二节 R5：写进变长留下的洞里——第一个文件变长到三个净荷（单元 0 补满，之后是洞），再在单元 2 里写 10 字节：单元 0 满、不重写不读，
/// 单元 1 补零不读，单元 2 读残留一次；三个单元都在了、洞没了，size 不变；读出原内容、零、写的字节、零。
#[test]
fn a_write_into_the_hole_after_an_extension_fills_the_gap_and_reads_only_the_residue() {
    let mut pool = PoolOnObservedDevices::with_the_first_file();
    let size = 3 * payload_capacity();
    pool.change_the_size(FIRST_FILE, size).expect("变长");
    let unit_zero = slot_of_the_data_unit(&pool.output, FIRST_FILE, 0);
    let bytes = [0x24u8; 10];
    let offset = 2 * payload_capacity() + 1;
    pool.forget_the_operations();
    let (published, residue_reads) = pool.write_at_offset(FIRST_FILE, offset, &bytes);
    let after = published.expect("洞里的写发成");
    assert_eq!(after.data_pointers.len(), 3);
    assert_eq!(
        slot_of_the_data_unit(&after, FIRST_FILE, 0),
        unit_zero,
        "单元 0 照抄"
    );
    let new_slots = [
        slot_of_the_data_unit(&after, FIRST_FILE, 1),
        slot_of_the_data_unit(&after, FIRST_FILE, 2),
    ];
    assert_eq!(
        reads_of_the_watched_slots_before_the_first_write(
            &pool.operations(),
            &[unit_zero, new_slots[0], new_slots[1]]
        ),
        vec![(DEVICE_ZERO, new_slots[1])],
        "单元 0、1 不读，单元 2 读残留一次"
    );
    assert_eq!(residue_reads.issued, 1);
    let mut expected = file_content();
    expected.resize(usize::try_from(offset).expect("偏移"), 0);
    expected.extend_from_slice(&bytes);
    expected.resize(usize::try_from(size).expect("长度"), 0);
    let image = pool.image();
    assert_eq!(
        whole_file_read_by_the_mounted_reader(&image, &after, FIRST_FILE),
        expected
    );
    let violated = invariants_violated(&image);
    assert!(violated.is_empty(), "池级 checker：{violated:?}");
}
