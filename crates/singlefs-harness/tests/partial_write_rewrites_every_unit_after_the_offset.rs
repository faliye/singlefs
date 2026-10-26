//! 写到偏移 o 时把 o 之后的单元一起重写（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R4，用户 2026-10-26 定）、
//! 任意活 inode（R6）、快照下只把真换下的单元进 deadlist（R7、截断判决 K4），与会话里的按偏移写。
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
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::transaction::TransactionUnit;
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

/// 三个单元的内容（最后一个差 7 字节满），每个字节按位置算、彼此不同。
fn content_of_three_units(seed: u8) -> Vec<u8> {
    let length = usize::try_from(3 * payload_capacity() - 7).expect("长度");
    (0..length)
        .map(|position| u8::try_from((position * 7 + usize::from(seed)) % 251).expect("小于 256"))
        .collect()
}

fn content_with_the_patch(content: &[u8], offset: u64, patch: &[u8]) -> Vec<u8> {
    let start = usize::try_from(offset).expect("偏移");
    let mut patched = content.to_vec();
    patched[start..start + patch.len()].copy_from_slice(patch);
    patched
}

/// 判决第二节 R4（用户 2026-10-26 定「保留写序，写到 o 时把 o 之后的单元一起重写」）：三个单元的第一个文件在单元 0 写 16 字节，三个单元都换了落点、
/// 出生 txg 都是这次发布的（「出生 txg 随偏移不降」照旧成立）；扫描重建认领每个活单元（I-1.13 成立），池级 checker 一条都不红；读出改过的内容。
#[test]
fn a_partial_write_on_the_first_unit_rewrites_every_unit_after_it_and_the_scan_rebuild_claims_them_all(
) {
    let content = content_of_three_units(1);
    let mut pool = PoolOnObservedDevices::with_the_first_file_written_as(&content);
    let before = pool.output.clone();
    let patch = [0x5Au8; 16];
    let (published, _) = pool.write_at_offset(FIRST_FILE, 0, &patch);
    let after = published.expect("部分写发成");
    assert_eq!(
        after
            .data_pointers
            .iter()
            .map(|pointer| pointer.head.birth_txg)
            .collect::<Vec<_>>(),
        vec![after.root.checkpoint_txg; 3],
        "三个单元的出生 txg 都是这次发布的"
    );
    for unit in 0..3 {
        assert_ne!(
            slot_of_the_data_unit(&after, FIRST_FILE, unit),
            slot_of_the_data_unit(&before, FIRST_FILE, unit),
            "单元 {unit} 换了落点"
        );
    }
    let image = pool.image();
    let violated = invariants_violated(&image);
    assert!(
        violated.is_empty(),
        "池级 checker（含 I-1.13）：{violated:?}"
    );
    assert_eq!(
        whole_file_read_by_the_mounted_reader(&image, &after, FIRST_FILE),
        content_with_the_patch(&content, 0, &patch)
    );
}

/// 判决第二节 R6（任意活 inode）：第一个文件之外的一个 inode 整份写成三个单元，再在它单元 0 的偏移 100 写 16 字节——它的三个单元都重写、出生 txg
/// 都是这次发布的，第一个文件的单元一个不动；池级 checker 一条都不红，读出改过的内容。
#[test]
fn a_partial_write_on_an_inode_other_than_the_first_rewrites_its_units_after_the_offset_and_leaves_the_first_file(
) {
    let mut pool = PoolOnObservedDevices::with_the_first_file();
    let inode = pool.create_one_inode();
    let content = content_of_three_units(3);
    pool.sequential_write_of_the_inode(inode, &content);
    let first_file_pointers_before = pool.output.data_pointers.clone();
    let patch = [0x5Au8; 16];
    let (published, _) = pool.write_at_offset(inode, 100, &patch);
    let after = published.expect("部分写发成");
    let file = after
        .file_other_than_the_first(inode)
        .expect("这一版里有这个文件");
    assert_eq!(
        file.data_pointers
            .iter()
            .map(|pointer| pointer.head.birth_txg)
            .collect::<Vec<_>>(),
        vec![after.root.checkpoint_txg; 3]
    );
    assert_eq!(
        after.data_pointers, first_file_pointers_before,
        "第一个文件不动"
    );
    let image = pool.image();
    let violated = invariants_violated(&image);
    assert!(violated.is_empty(), "池级 checker：{violated:?}");
    assert_eq!(
        whole_file_read_by_the_mounted_reader(&image, &after, inode),
        content_with_the_patch(&content, 100, &patch)
    );
}

/// 判决第二节 R7 与截断判决 K4：有活快照时对三个单元的第一个文件的最后一个单元部分写，计划交 `NotRewrittenWhole`——进 deadlist 的数据单元恰是这次
/// 真换下的单元 2，单元 0、1 的映射 key 不进、落点不动；池级 checker 一条都不红。
#[test]
fn a_partial_write_under_a_snapshot_puts_only_the_data_units_it_replaced_on_the_deadlist() {
    let content = content_of_three_units(5);
    let mut pool = PoolOnObservedDevices::with_the_first_file_written_as(&content);
    pool.create_a_snapshot();
    let before = pool.output.clone();
    let key_of_the_data_unit = |unit: u64| {
        before
            .mapped_units
            .iter()
            .find(|(identity, _)| *identity == TransactionUnit::Data(DataUnitIndexInFile(unit)))
            .expect("数据单元进映射")
            .1
            .clone()
    };
    let patch = [0x5Au8; 16];
    let offset = 2 * payload_capacity() + 10;
    let (published, _) = pool.write_at_offset(FIRST_FILE, offset, &patch);
    let after = published.expect("快照下的部分写发成");
    let data_unit_keys_on_the_deadlist: Vec<u64> = (0..3)
        .filter(|unit| {
            let key = key_of_the_data_unit(*unit);
            after
                .deadlist_entries
                .iter()
                .any(|entry| entry.mapping_key == key)
        })
        .collect();
    assert_eq!(
        data_unit_keys_on_the_deadlist,
        vec![2],
        "只有这次换下的单元 2 进 deadlist"
    );
    assert_eq!(
        after.data_pointers[..2],
        before.data_pointers[..2],
        "单元 0、1 落点不动"
    );
    let image = pool.image();
    let violated = invariants_violated(&image);
    assert!(violated.is_empty(), "池级 checker：{violated:?}");
    assert_eq!(
        whole_file_read_by_the_mounted_reader(&image, &after, FIRST_FILE),
        content_with_the_patch(&content, offset, &patch)
    );
}

/// 会话里的按偏移写（`UserChange::WriteAtOffset`）：可写挂载之后经会话在第一个文件单元 1 的偏移 100 写 4 KiB，发成、读出改过的内容，池级 checker 不红。
#[test]
fn a_write_at_an_offset_through_the_mounted_session_publishes_the_patched_file() {
    let content = content_of_three_units(7);
    let pool = PoolOnObservedDevices::with_the_first_file_written_as(&content);
    let mut devices = pool.devices;
    let parameters = parameters();
    let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let patch = vec![0x9Du8; 4096];
    let offset = payload_capacity() + 100;
    session
        .publish_user_change(
            &mut devices,
            UserChange::WriteAtOffset {
                inode: FIRST_FILE,
                offset: FileOffsetInBytes(offset),
                bytes: PartialWriteBytes::new(&patch).expect("不是空的"),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            },
        )
        .expect("会话里按偏移写发成");
    let current = session
        .current
        .file_version()
        .expect("现行那一版带文件")
        .clone();
    let image = image_of(&devices);
    assert_eq!(
        whole_file_read_by_the_mounted_reader(&image, &current, FIRST_FILE),
        content_with_the_patch(&content, offset, &patch)
    );
    let violated = invariants_violated(&image);
    assert!(violated.is_empty(), "池级 checker：{violated:?}");
}
