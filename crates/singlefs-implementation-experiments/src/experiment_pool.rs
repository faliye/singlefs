//! 实验装置造的池：两块录进同一条流（录制流留内容）的文件镜像上 mkfs（[`format_pool`]），或接着取号、暖机、新池新建文件（[`build_pool`]）。
//!
//! 池的样子全取验证包库的公开项，这个包不另定：mkfs 参数是 `known_bad_images::parameters_of_the_common_test_pool`，盘宽是
//! `history::HistoryDeviceWidth::FourGibibytes`（库里写明与验证包测试搭建的文件镜像同宽），固定写时刻与第一个文件的内容是
//! `scenario` 那一份；同一个进程接着写的分配器照核心层 `allocator_after_make_filesystem` 建。录出的流因此与验证包库的新池新建文件那条路
//! （`scenario::run_new_pool_file_creation`，[`new_pool_file_creation_of_the_verification_library`]）逐条相同：单测
//! `the_built_pool_records_the_stream_of_the_new_pool_file_creation_of_the_verification_library` 核这一条，
//! E170（O3规约执行器第一版）、E171（三oracle合谋测试） 的停机 S1 拿装置自己驱动出的流与那条路比。
//!
//! 名字与形状照装置的用法定（E184（常规层与大负载层的代价计数） 照抄的崩溃枚举用例建流函数原样调 `build_pool`、`file_content`、
//! `geometry`、`parameters` 与池的字段）；实现是这个包自己写的，不经 `#[path]` 引验证包 `tests/` 下的共用模块
//! （`.claude/rules/directory-layout.md`「crates/：各包放什么」）。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::{FileBackedBlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemOutput,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_verification::history::HistoryDeviceWidth;
pub use singlefs_verification::known_bad_images::{
    parameters_of_the_common_test_pool as parameters, E142_FILESYSTEM_IDENTIFIER,
};
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};
pub use singlefs_verification::scenario::{
    first_file_content as file_content, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_verification::scenario::{run_new_pool_file_creation, ScenarioPoint};
use singlefs_verification::segments::FixedGeometry;
use singlefs_verification::temporary_files::TemporaryPathRemovedOnDrop;
use singlefs_verification::{RecordingBlockDevice, RetainedOperation, SharedStream};

/// 两块盘各多宽：验证包库给这一类池定的那一档（随机历史的 4 GiB 盘宽）。
const POOL_DEVICE_WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::FourGibibytes;

/// 池的两块盘：[`parameters`] 的三个区域落在这两块上。
const DEVICES_OF_THE_POOL: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 同一个进程里每一对镜像各取一个号，写进文件名：几条流在几个线程里同时造池也不撞名。
static NEXT_IMAGE_PAIR_NUMBER: AtomicU64 = AtomicU64::new(0);

/// 一块文件镜像上开的设备，写录进池的那条流。
pub type RecordedImageDevice = RecordingBlockDevice<FileBackedBlockDevice>;

/// 写表切段用的几何：槽距、journal 环长与根环每区槽数照 [`parameters`] 取。
#[must_use]
pub fn geometry() -> FixedGeometry {
    let sizes_of_the_pool = parameters().geometry;
    FixedGeometry {
        fixed_structure_slot_spacing: sizes_of_the_pool.fixed_structure_slot_spacing,
        journal_ring_bytes: sizes_of_the_pool.journal_ring_bytes,
        root_ring_slots_per_region: sizes_of_the_pool.root_ring_slots_per_region,
    }
}

/// 只做过 mkfs 的池：两块镜像上 mkfs 写完，没取过号、一个文件都没发布。
pub struct FormattedPool {
    /// 开着的两块镜像；`take` 出去（或 [`FormattedPool::reopen_recorded`] 关掉）就是进程退出、镜像关掉。
    pub devices: Option<Vec<(DeviceIdentity, RecordedImageDevice)>>,
    pub stream: SharedStream,
    pub genesis: MakeFilesystemOutput,
    /// mkfs 在录制流里占的步数（含末尾那道屏障）。
    pub mkfs_operation_count: usize,
    /// 两块镜像文件，由守卫认领：池丢掉时删（要留现场照 `singlefs_verification::temporary_files` 的开关）。
    pub image_files: Vec<TemporaryPathRemovedOnDrop>,
}

/// 走完 mkfs、取号、暖机、新池新建文件的池。
pub struct BuiltPool {
    /// 开着的两块镜像；`take` 出去（或 [`BuiltPool::reopen_recorded`] 关掉）就是进程退出、镜像关掉。
    pub devices: Option<Vec<(DeviceIdentity, RecordedImageDevice)>>,
    pub stream: SharedStream,
    /// mkfs 在录制流里占的步数（含末尾那道屏障）。
    pub mkfs_operation_count: usize,
    /// 暖机调完、新池新建文件还没调时录制流的长度：暖机与新池新建文件的分界。
    pub warm_up_operation_count: usize,
    /// 现行那一版：走完时是新池新建文件那一版，装置接着发布时换成新的一版。
    pub output: TransactionOutput,
    /// 同一个进程接着写用的分配器。
    pub allocator: PoolAllocator,
    /// 两块镜像文件，由守卫认领：池丢掉时删。
    pub image_files: Vec<TemporaryPathRemovedOnDrop>,
}

/// mkfs 之后接着走完的取号、暖机、新池新建文件交回的三样：同一个进程接着写用的分配器、新池新建文件那一版、暖机调完时录制流的长度。
pub struct AfterTheNewPoolFileCreation {
    pub allocator: PoolAllocator,
    pub output: TransactionOutput,
    pub warm_up_operation_count: usize,
}

/// 两块镜像文件怎么打开：新建（文件已存在就拒），或按路径重开已有的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImageFileOpening {
    CreatedExclusively,
    Reopened,
}

/// 一块镜像文件建在临时目录的哪：`singlefs-experiment-pool-<tag>-<进程号>-<这一对的号>-device<盘号>.img`。
fn image_file_path(tag: &str, pair_number: u64, device: DeviceIdentity) -> PathBuf {
    std::env::temp_dir().join(format!(
        "singlefs-experiment-pool-{tag}-{}-{pair_number}-device{}.img",
        std::process::id(),
        device.0
    ))
}

/// 每块镜像文件上开一个设备（第几份文件对第几块盘），写录进 `stream`。
///
/// # Panics
/// 镜像文件不是恰好两份；新建时文件已存在，或打不开。
fn recorded_devices_on_the_image_files(
    image_files: &[TemporaryPathRemovedOnDrop],
    stream: &SharedStream,
    opening: ImageFileOpening,
) -> Vec<(DeviceIdentity, RecordedImageDevice)> {
    assert_eq!(
        image_files.len(),
        DEVICES_OF_THE_POOL.len(),
        "池的每块盘一份镜像文件"
    );
    let physical_block_size = PhysicalBlockSizeInBytes(parameters().geometry.physical_block_size);
    DEVICES_OF_THE_POOL
        .iter()
        .zip(image_files)
        .map(|(device, image_file)| {
            let opened = match opening {
                ImageFileOpening::CreatedExclusively => {
                    FileBackedBlockDevice::create_image_file_exclusively(
                        image_file.path(),
                        POOL_DEVICE_WIDTH.device_bytes(),
                        physical_block_size,
                    )
                }
                ImageFileOpening::Reopened => FileBackedBlockDevice::open_existing_image_file(
                    image_file.path(),
                    POOL_DEVICE_WIDTH.device_bytes(),
                    physical_block_size,
                ),
            };
            let image_device = opened.unwrap_or_else(|error| {
                panic!(
                    "镜像文件 {} 打不开（{opening:?}）：{error:?}",
                    image_file.path().display()
                )
            });
            (
                *device,
                RecordingBlockDevice::with_shared_stream(*device, image_device, stream.clone()),
            )
        })
        .collect()
}

/// 把一段录制流施加到两块空内存盘上：与文件镜像同一份字节。
fn memory_pool_after(operations: &[RetainedOperation]) -> MemoryPool {
    let mut memory_pool =
        MemoryPool::with_devices(&DEVICES_OF_THE_POOL, POOL_DEVICE_WIDTH.device_bytes());
    memory_pool.apply(operations);
    memory_pool
}

/// 两块新建的镜像上 mkfs（`tag` 是镜像文件名里的那一段）。镜像文件建之前就由守卫认领：mkfs 半路 panic 也删掉已建的那几块。
///
/// # Panics
/// 镜像建不了；mkfs 被拒。
#[must_use]
pub fn format_pool(tag: &str) -> FormattedPool {
    let pair_number = NEXT_IMAGE_PAIR_NUMBER.fetch_add(1, Ordering::SeqCst);
    let image_files: Vec<TemporaryPathRemovedOnDrop> = DEVICES_OF_THE_POOL
        .iter()
        .map(|device| {
            TemporaryPathRemovedOnDrop::claiming(image_file_path(tag, pair_number, *device))
        })
        .collect();
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_on_the_image_files(
        &image_files,
        &stream,
        ImageFileOpening::CreatedExclusively,
    );
    let genesis = make_filesystem(&parameters(), &mut devices).expect("新建的两块镜像上 mkfs");
    let mkfs_operation_count = stream.operation_count();
    FormattedPool {
        devices: Some(devices),
        stream,
        genesis,
        mkfs_operation_count,
        image_files,
    }
}

/// 两块新建的镜像上走 mkfs、取号、暖机、新池新建文件（[`FormattedPool::through_the_new_pool_file_creation`]）。
///
/// # Panics
/// 哪一步被拒。
#[must_use]
pub fn build_pool(tag: &str) -> BuiltPool {
    format_pool(tag).through_the_new_pool_file_creation()
}

impl FormattedPool {
    /// 接着取号、暖机、新池新建文件，交出走完的池：分配器照 mkfs 刚写下的样子建（`allocator_after_make_filesystem`，同一个进程接着写），
    /// 新池第一次取号取 1，第一个文件与写时刻取 [`file_content`]、[`FIXED_WRITE_TIME_SECONDS`]。
    ///
    /// # Panics
    /// 镜像已经关掉；哪一步被拒；新池第一次取号不是 1。
    #[must_use]
    pub fn through_the_new_pool_file_creation(mut self) -> BuiltPool {
        let parameters_of_the_pool = parameters();
        let mut open_devices = self.devices.take().expect("mkfs 之后镜像还开着");
        let mut allocator =
            allocator_after_make_filesystem(&parameters_of_the_pool, &open_devices, &self.genesis);
        let first_file = file_content();
        let mut writer = PoolWriter::new(&parameters_of_the_pool, &mut open_devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        assert_eq!(instance, InstanceGeneration(1), "新池第一次取号取 1");
        let warmed = warm_up(&mut writer, &self.genesis.root, instance).expect("暖机");
        let warm_up_operation_count = self.stream.operation_count();
        let output = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机写下两代根"),
            FirstFile {
                content: &first_file,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件");
        self.devices = Some(open_devices);
        self.into_built_pool(AfterTheNewPoolFileCreation {
            allocator,
            output,
            warm_up_operation_count,
        })
    }

    /// 自己驱动取号、暖机、新池新建文件的装置（E170 每一步返回时记流长）把走完的三样交进来，镜像、设备与流转给 [`BuiltPool`]。
    #[must_use]
    pub fn into_built_pool(self, after: AfterTheNewPoolFileCreation) -> BuiltPool {
        let FormattedPool {
            devices,
            stream,
            genesis: _,
            mkfs_operation_count,
            image_files,
        } = self;
        BuiltPool {
            devices,
            stream,
            mkfs_operation_count,
            warm_up_operation_count: after.warm_up_operation_count,
            output: after.output,
            allocator: after.allocator,
            image_files,
        }
    }
}

/// [`FormattedPool`] 与 [`BuiltPool`] 共有的三个方法：两者都持着镜像、设备、流与 mkfs 的步数，写法相同，由这个宏一次写成、给两个类型各生成一份。
macro_rules! methods_of_a_recorded_pool {
    ($pool:ty) => {
        impl $pool {
            /// 整条录制流（带内容，从 mkfs 起）。
            ///
            /// # Panics
            /// 流比 mkfs 写下的还短（录制流只增不减，短了是流被换过）。
            #[must_use]
            pub fn retained_operations(&self) -> Vec<RetainedOperation> {
                let operations = self.stream.retained_operations();
                assert!(
                    operations.len() >= self.mkfs_operation_count,
                    "录制流 {} 步，比 mkfs 写下的 {} 步还短",
                    operations.len(),
                    self.mkfs_operation_count
                );
                operations
            }

            /// 只施加 mkfs 那几步的内存镜像：崩溃点重放枚举的起点。
            #[must_use]
            pub fn memory_pool_after_mkfs(&self) -> MemoryPool {
                memory_pool_after(&self.retained_operations()[..self.mkfs_operation_count])
            }

            /// 关掉开着的设备（进程退出、镜像关掉），按路径重开两块镜像，写照旧录进同一条流：进程重开之后的可写挂载与发布都进这条流。
            ///
            /// # Panics
            /// 镜像文件打不开。
            pub fn reopen_recorded(&mut self) -> Vec<(DeviceIdentity, RecordedImageDevice)> {
                self.devices = None;
                recorded_devices_on_the_image_files(
                    &self.image_files,
                    &self.stream,
                    ImageFileOpening::Reopened,
                )
            }
        }
    };
}

methods_of_a_recorded_pool!(FormattedPool);
methods_of_a_recorded_pool!(BuiltPool);

impl BuiltPool {
    /// 整条录制流施加到两块空内存盘上：与文件镜像同一份字节。
    #[must_use]
    pub fn memory_pool(&self) -> MemoryPool {
        memory_pool_after(&self.retained_operations())
    }
}

/// 验证包库的新池新建文件那条路在两块空内存盘上录下的流：E170、E171 的停机 S1 拿装置自己驱动出的流与它逐条比。
pub struct NewPoolFileCreationOfTheVerificationLibrary {
    /// 整条录制流（带内容，从 mkfs 起）。
    pub operations: Vec<RetainedOperation>,
    /// mkfs 在录制流里占的步数（那条路交回的 `mkfs_operation_count`）。
    pub mkfs_operation_count: usize,
    /// 暖机调完、新池新建文件还没开始时录制流的长度（那条路在 `ScenarioPoint::BeforeNewPoolFileCreation` 叫调用方的那一刻）。
    pub warm_up_operation_count: usize,
}

/// 验证包库的新池新建文件那条路（`singlefs_verification::scenario::run_new_pool_file_creation`）走一遍：两块空内存盘
/// （`singlefs_verification::memory_pool::SparseBlockDevice`），盘宽、物理块与 mkfs 参数与 [`build_pool`] 同一组，都取验证包库的。
///
/// # Panics
/// 那条路没走完；或走完了却没在暖机之后叫调用方。
#[must_use]
pub fn new_pool_file_creation_of_the_verification_library(
) -> NewPoolFileCreationOfTheVerificationLibrary {
    let parameters_of_the_pool = parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        DEVICES_OF_THE_POOL
            .iter()
            .map(|device| {
                (
                    *device,
                    RecordingBlockDevice::with_shared_stream(
                        *device,
                        SparseBlockDevice::new(
                            POOL_DEVICE_WIDTH.device_bytes(),
                            PhysicalBlockSizeInBytes(
                                parameters_of_the_pool.geometry.physical_block_size,
                            ),
                        ),
                        stream.clone(),
                    ),
                )
            })
            .collect();
    let mut warm_up_operation_count = None;
    let run = run_new_pool_file_creation(
        &parameters_of_the_pool,
        &mut devices,
        &stream,
        |point, _devices| match point {
            ScenarioPoint::BeforeNewPoolFileCreation => {
                warm_up_operation_count = Some(stream.operation_count());
            }
            ScenarioPoint::AfterMakeFilesystem | ScenarioPoint::AfterInstanceAcquisition => {}
        },
    )
    .unwrap_or_else(|failure| panic!("验证包库的新池新建文件那条路没走完：{}", failure.render()));
    NewPoolFileCreationOfTheVerificationLibrary {
        operations: stream.retained_operations(),
        mkfs_operation_count: run.mkfs_operation_count,
        warm_up_operation_count: warm_up_operation_count
            .expect("那条路在暖机之后、新池新建文件之前叫调用方一次"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 场景：这个包造的池（两块文件镜像上 mkfs、取号、暖机、新池新建文件）与验证包库的新池新建文件那条路（两块内存盘）各录一条流。
    /// 预期：mkfs 的步数、暖机调完时的流长相同；两条流一样长、逐条相同（带内容），第一处不同的下标是没有。
    /// 判别力（`crates/mutations.tsv`）：造池时第一个文件的写时刻晚一秒，这条红。
    #[test]
    fn the_built_pool_records_the_stream_of_the_new_pool_file_creation_of_the_verification_library()
    {
        let built = build_pool("compared-with-the-verification-library");
        let of_the_library = new_pool_file_creation_of_the_verification_library();
        assert_eq!(
            built.mkfs_operation_count, of_the_library.mkfs_operation_count,
            "mkfs 在录制流里占的步数"
        );
        assert_eq!(
            built.warm_up_operation_count, of_the_library.warm_up_operation_count,
            "暖机调完时录制流的长度"
        );
        let built_operations = built.retained_operations();
        let first_difference = built_operations
            .iter()
            .zip(&of_the_library.operations)
            .position(|(built_operation, library_operation)| built_operation != library_operation);
        assert_eq!(
            (built_operations.len(), first_difference),
            (of_the_library.operations.len(), None),
            "两条流的长度，与第一处不同的下标"
        );
    }
}
