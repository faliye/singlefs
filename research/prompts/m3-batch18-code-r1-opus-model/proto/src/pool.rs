//! 内存稀疏盘上的池：mkfs → 取号 → 暖机 → 新池新建文件，录制流开内容保留；之后的发布接在后面。原型自己造，不从带 layer0 的用例拷。
use singlefs_core::address::{DeviceIdentity, InodeNumber, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_sequential_write,
    publish_unlink_intent, publish_unlink_step, warm_up, FirstFile, PoolWriter, PublishError,
    TransactionOutput,
};
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

use crate::common::{parameters, FIXED_WRITE_TIME_SECONDS};

pub type Dev = RecordingBlockDevice<SparseBlockDevice>;
pub const INSTANCE_ONE: InstanceGeneration = InstanceGeneration(1);

pub struct SparsePool {
    pub devices: Vec<(DeviceIdentity, Dev)>,
    pub stream: SharedStream,
    pub allocator: PoolAllocator,
    pub output: TransactionOutput,
    pub mkfs_operation_count: usize,
    pub device_bytes: u64,
    /// 每次发布之后的整池镜像（含新池新建文件那一版），按先后，带名字。
    pub published_images: Vec<(String, MemoryPool)>,
}

pub fn batch(units: u64) -> UnlinkBatchSize {
    if units >= 144 {
        UnlinkBatchSize::OneExtentLowerLeaf
    } else {
        UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(units).expect("1..=144"))
    }
}

pub fn content_of(bytes: usize) -> Vec<u8> {
    (0..bytes).map(|index| u8::try_from((index * 13 + 5) % 251).expect("<256")).collect()
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
        let mkfs_operation_count = stream.operations().len();
        let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
        let output = {
            let mut pool = PoolWriter::new(&parameters, &mut devices);
            let instance = acquire_instance(&mut pool).expect("取号");
            assert_eq!(instance, INSTANCE_ONE);
            let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
            publish_first_file(
                &mut pool,
                &mut allocator,
                warm.roots.last().expect("暖机根"),
                FirstFile { content: first_content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
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
            mkfs_operation_count,
            device_bytes,
            published_images: Vec::new(),
        };
        pool.snapshot("新池新建文件");
        pool
    }

    pub fn image(&self) -> MemoryPool {
        MemoryPool {
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
                .collect(),
            device_size_in_bytes: self.device_bytes,
        }
    }

    pub fn snapshot(&mut self, name: &str) {
        let image = self.image();
        let txg = self.output.root.checkpoint_txg.0;
        self.published_images.push((format!("{name}@txg{txg}"), image));
    }

    fn publish(
        &mut self,
        name: &str,
        run: impl FnOnce(&mut PoolWriter<'_, Dev>, &mut PoolAllocator, &TransactionOutput) -> Result<TransactionOutput, PublishError>,
    ) -> Result<TransactionOutput, PublishError> {
        let parameters = parameters();
        let previous = self.output.clone();
        let output = {
            let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
            run(&mut writer, &mut self.allocator, &previous)?
        };
        self.output = output.clone();
        self.snapshot(name);
        Ok(output)
    }

    pub fn sequential_write(&mut self, content: &[u8], time: u64) -> TransactionOutput {
        self.publish("顺序写", |writer, allocator, previous| {
            publish_sequential_write(writer, allocator, previous, FirstFile { content, write_time_seconds: time }, INSTANCE_ONE)
        })
        .expect("顺序写")
    }

    pub fn new_inodes(&mut self, count: u64, time: u64) -> TransactionOutput {
        self.publish("建空 inode", |writer, allocator, previous| {
            publish_new_inodes(writer, allocator, previous, count, time, INSTANCE_ONE)
        })
        .expect("建 inode")
    }

    pub fn intent(&mut self, inode: u64, time: u64) -> Result<TransactionOutput, PublishError> {
        self.publish(&format!("意图{inode}"), |writer, allocator, previous| {
            publish_unlink_intent(writer, allocator, previous, InodeNumber(inode), time, INSTANCE_ONE)
        })
    }

    pub fn step(&mut self, inode: u64, units_per_batch: u64) -> Result<TransactionOutput, PublishError> {
        self.publish(&format!("删除步{inode}"), |writer, allocator, previous| {
            publish_unlink_step(writer, allocator, previous, InodeNumber(inode), batch(units_per_batch), INSTANCE_ONE)
        })
    }

    /// 队首的号一步步删到队列空。
    pub fn steps_until_empty(&mut self, units_per_batch: u64) -> usize {
        let mut steps = 0;
        while let Some(inode) = self.output.deleted_inodes.first().copied() {
            self.step(inode.0, units_per_batch).expect("删除步");
            steps += 1;
            assert!(steps < 10_000, "走不空");
        }
        steps
    }
}
