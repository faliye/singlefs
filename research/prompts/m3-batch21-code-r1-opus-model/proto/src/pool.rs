//! 内存稀疏盘上的池（攻方自己造，不从带 layer0 的用例拷）：mkfs → 取号 → 暖机 → 新池新建文件；之后的发布接在后面，每次发布后留一份整池镜像。
//! 录制流开内容保留，崩溃枚举从任一次发布之后的下标起取写流。
use singlefs_core::address::{DeviceIdentity, InodeNumber, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_sequential_write, publish_unlink_intent,
    publish_unlink_step, warm_up, FirstFile, PoolWriter, PublishError, TransactionOutput,
};
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

pub type Dev = RecordingBlockDevice<SparseBlockDevice>;
pub const INSTANCE_ONE: InstanceGeneration = InstanceGeneration(1);
pub const TIME: u64 = 1_788_000_000;
/// 与 harness 测试共用模块（`tests/common/mod.rs` 的 `E142_FILESYSTEM_IDENTIFIER`）同一个 fsid：坏镜像改法里找最新根按它读。
pub const FSID: [u8; 16] = [0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00];

pub fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FSID,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

pub fn geometry() -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        root_ring_slots_per_region: parameters().geometry.root_ring_slots_per_region,
    }
}

pub fn batch(units: u64) -> UnlinkBatchSize {
    if units >= 144 {
        UnlinkBatchSize::OneExtentLowerLeaf
    } else {
        UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(units).expect("1..=144"))
    }
}

/// 第 `seed` 份内容，`bytes` 字节。
pub fn content_of(bytes: usize, seed: usize) -> Vec<u8> {
    (0..bytes).map(|index| u8::try_from((index * 7 + seed * 29 + 3) % 251).expect("<256")).collect()
}

pub struct SparsePool {
    pub devices: Vec<(DeviceIdentity, Dev)>,
    pub stream: SharedStream,
    pub allocator: PoolAllocator,
    pub output: TransactionOutput,
    pub device_bytes: u64,
    /// 每次发布之后：名字、整池镜像、这一刻录制流的长度、这一版的 txg、这一版读得出的第一个文件内容（没有文件时 None）。
    pub published: Vec<Published>,
    pub content: Option<Vec<u8>>,
}

pub struct Published {
    pub name: String,
    pub image: MemoryPool,
    pub operations: usize,
    pub txg: u64,
    pub content: Option<Vec<u8>>,
}

impl SparsePool {
    pub fn build(device_bytes: u64, first_content: &[u8]) -> SparsePool {
        let parameters = parameters();
        let stream = SharedStream::retaining_contents();
        let mut devices: Vec<(DeviceIdentity, Dev)> = (0..2u32)
            .map(|number| {
                (
                    DeviceIdentity(number),
                    RecordingBlockDevice::with_shared_stream(
                        DeviceIdentity(number),
                        SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                )
            })
            .collect();
        let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
        let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
        let output = {
            let mut pool = PoolWriter::new(&parameters, &mut devices);
            let instance = acquire_instance(&mut pool).expect("取号");
            let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
            publish_first_file(
                &mut pool,
                &mut allocator,
                warm.roots.last().expect("暖机根"),
                FirstFile { content: first_content, write_time_seconds: TIME },
                instance,
                &warm.last_record_bytes,
            )
            .expect("新池新建文件")
        };
        let mut pool = SparsePool {
            devices,
            stream,
            allocator,
            output,
            device_bytes,
            published: Vec::new(),
            content: Some(first_content.to_vec()),
        };
        pool.snapshot("新池新建文件");
        pool
    }

    pub fn image(&self) -> MemoryPool {
        MemoryPool {
            devices: self.devices.iter().map(|(identity, device)| (*identity, device.wrapped_device().image.clone())).collect(),
            device_size_in_bytes: self.device_bytes,
        }
    }

    pub fn snapshot(&mut self, name: &str) {
        let image = self.image();
        let txg = self.output.root.checkpoint_txg.0;
        let operations = self.stream.operations().len();
        self.published.push(Published { name: format!("{name}@txg{txg}"), image, operations, txg, content: self.content.clone() });
    }

    fn publish(
        &mut self,
        name: &str,
        run: impl FnOnce(&mut PoolWriter<'_, Dev>, &mut PoolAllocator, &TransactionOutput) -> Result<TransactionOutput, PublishError>,
    ) -> Result<(), PublishError> {
        let parameters = parameters();
        let previous = self.output.clone();
        let output = {
            let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
            run(&mut writer, &mut self.allocator, &previous)?
        };
        self.output = output;
        self.snapshot(name);
        Ok(())
    }

    pub fn sequential_write(&mut self, content: &[u8]) {
        let time = TIME + 10;
        self.publish("顺序写", |writer, allocator, previous| {
            publish_sequential_write(writer, allocator, previous, FirstFile { content, write_time_seconds: time }, INSTANCE_ONE)
        })
        .expect("顺序写");
        self.content = Some(content.to_vec());
        let last = self.published.last_mut().expect("刚发");
        last.content = self.content.clone();
    }

    pub fn new_inodes(&mut self, count: u64) {
        self.publish("建空inode", |writer, allocator, previous| publish_new_inodes(writer, allocator, previous, count, TIME + 20, INSTANCE_ONE))
            .expect("建 inode");
    }

    pub fn intent(&mut self, inode: u64) -> Result<(), PublishError> {
        if inode == 1 {
            self.content = None;
        }
        self.publish(&format!("意图{inode}"), |writer, allocator, previous| {
            publish_unlink_intent(writer, allocator, previous, InodeNumber(inode), TIME + 30, INSTANCE_ONE)
        })
    }

    pub fn step(&mut self, units_per_batch: u64) -> Result<(), PublishError> {
        let inode = self.output.deleted_inodes.first().copied().expect("队列里有号");
        self.publish(&format!("删除步{}", inode.0), |writer, allocator, previous| {
            publish_unlink_step(writer, allocator, previous, inode, batch(units_per_batch), INSTANCE_ONE)
        })
    }

    pub fn queue_is_empty(&self) -> bool {
        self.output.deleted_inodes.is_empty()
    }

    pub fn inode_numbers(&self) -> Vec<u64> {
        self.output
            .inode_leaf_containers
            .iter()
            .flat_map(|container| container.contents.records.iter().map(|record| record.inode))
            .collect()
    }
}
