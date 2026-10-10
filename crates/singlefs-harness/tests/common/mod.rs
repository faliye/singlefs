//! 步 6 / 步 7 验收共用的搭建：两个文件镜像上 mkfs → 取号 → 暖机 → 新池新建文件，录制流开内容保留。
#![allow(dead_code, reason = "每个测试文件各自只用到其中一部分")]

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{FileBackedBlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::journal::{back_chain_of, record_offset, JournalRecord};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, MakeFilesystemOutput,
    MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::recovery::{choose_system_configuration, readable_roots, PoolReader};
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, publish_sequential_write, warm_up,
    FirstFile, PoolWriter, PublishError, TransactionOutput, WarmUpOutput,
};
use singlefs_core::unit::{
    unit_filesystem_identifier, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
};
use singlefs_format::{
    DATA_UNIT_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT,
    NODE_BYTES, SLOT_BYTES,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};

static IMAGE_COUNTER: AtomicU64 = AtomicU64::new(0);
pub const IMAGE_BYTES: u64 = 4 << 30;
/// E142 的第一个文件 3000 字节（`name=config file_bytes=3000`）。
pub const FILE_BYTES: usize = 3000;
/// E142 装置里的固定 fsid（`FIXED_FSID`）。
pub const E142_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
/// E142 装置里的固定写入时间（`FIXED_WRITE_TIME_SECONDS`）。
pub const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;

pub type Recorded = RecordingBlockDevice<FileBackedBlockDevice>;

pub fn image_path(tag: &str, device: u32) -> PathBuf {
    let sequence = IMAGE_COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!(
        "singlefs-step67-{tag}-{}-{sequence}-dev{device}.img",
        std::process::id()
    ))
}

pub fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: E142_FILESYSTEM_IDENTIFIER,
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

pub fn file_content() -> Vec<u8> {
    (0..FILE_BYTES)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect()
}

pub struct BuiltPool {
    pub paths: Vec<PathBuf>,
    /// `take` 出去就是「进程退出、镜像关掉」：冷启动要重新打开文件。
    pub devices: Option<Vec<(DeviceIdentity, Recorded)>>,
    pub stream: SharedStream,
    pub genesis: MakeFilesystemOutput,
    pub warm_up: WarmUpOutput,
    pub output: TransactionOutput,
    pub allocator: PoolAllocator,
    pub mkfs_operation_count: usize,
    /// 暖机 `warm_up()` 已调完、`publish_first_file()` 还没调时录制流的长度：暖机与新池新建文件的分界，
    /// 不是从流尾往回数几步（里程碑「覆盖写、释放、回退与复用」收尾批「55 号装置步数」）。
    pub warm_up_operation_count: usize,
}

impl Drop for BuiltPool {
    fn drop(&mut self) {
        for path in &self.paths {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// 把一段录制流施加到两块空内存盘上：与文件镜像同一份字节。
fn memory_pool_of(operations: &[RetainedOperation]) -> MemoryPool {
    let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    pool.apply(operations);
    pool
}

/// 按路径重开镜像，写照旧录进同一条流。
fn reopen_recorded_images(
    paths: &[PathBuf],
    stream: &SharedStream,
) -> Vec<(DeviceIdentity, Recorded)> {
    paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let file = FileBackedBlockDevice::open_existing_image_file(
                path,
                IMAGE_BYTES,
                PhysicalBlockSizeInBytes(512),
            )
            .expect("重开镜像");
            let identity = DeviceIdentity(u32::try_from(index).expect("设备号"));
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, file, stream.clone()),
            )
        })
        .collect()
}

/// 按路径重开镜像，不录。
fn reopen_cold_images(paths: &[PathBuf]) -> Vec<(DeviceIdentity, FileBackedBlockDevice)> {
    paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let device = FileBackedBlockDevice::open_existing_image_file(
                path,
                IMAGE_BYTES,
                PhysicalBlockSizeInBytes(512),
            )
            .expect("重开镜像");
            (
                DeviceIdentity(u32::try_from(index).expect("设备号")),
                device,
            )
        })
        .collect()
}

/// 两个新建的全零文件镜像上 mkfs，录制流开内容保留：返回镜像路径、录着的设备、流、mkfs 写出的东西与 mkfs 占了流里几步。
fn formatted_recorded_images(
    tag: &str,
) -> (
    Vec<PathBuf>,
    Vec<(DeviceIdentity, Recorded)>,
    SharedStream,
    MakeFilesystemOutput,
    usize,
) {
    let stream = SharedStream::retaining_contents();
    let mut paths = Vec::new();
    let mut devices: Vec<(DeviceIdentity, Recorded)> = Vec::new();
    for device_number in 0..2u32 {
        let path = image_path(tag, device_number);
        let file = FileBackedBlockDevice::create_image_file_exclusively(
            &path,
            IMAGE_BYTES,
            PhysicalBlockSizeInBytes(512),
        )
        .expect("建镜像");
        devices.push((
            DeviceIdentity(device_number),
            RecordingBlockDevice::with_shared_stream(
                DeviceIdentity(device_number),
                file,
                stream.clone(),
            ),
        ));
        paths.push(path);
    }
    let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let mkfs_operation_count = stream.operations().len();
    (paths, devices, stream, genesis, mkfs_operation_count)
}

impl BuiltPool {
    /// 整条录制流带内容。
    pub fn retained_operations(&self) -> Vec<RetainedOperation> {
        self.stream.retained_operations()
    }
    /// 把整条流施加到内存镜像上：与文件镜像同一份字节。
    pub fn memory_pool(&self) -> MemoryPool {
        memory_pool_of(&self.retained_operations())
    }
    /// 只施加 mkfs 那 13 步：层 0 枚举的基线。
    pub fn memory_pool_after_mkfs(&self) -> MemoryPool {
        memory_pool_of(&self.retained_operations()[..self.mkfs_operation_count])
    }
    /// 重开镜像并继续录进同一条流：进程重开之后的可写挂载与发布都要进层 0 的整条流（里程碑「覆盖写、释放、回退与复用」步 0 / 步 3）。
    pub fn reopen_recorded(&mut self) -> Vec<(DeviceIdentity, Recorded)> {
        drop(self.devices.take());
        reopen_recorded_images(&self.paths, &self.stream)
    }

    /// 冷启动：丢掉进程内的设备句柄，按路径重新打开镜像。
    pub fn reopen_cold(&mut self) -> Vec<(DeviceIdentity, FileBackedBlockDevice)> {
        drop(self.devices.take());
        reopen_cold_images(&self.paths)
    }
}

/// 只做过 mkfs 的池：没取过号、一个文件都没发布（里程碑「覆盖写、释放、回退与复用」步 3：只做过 mkfs 的池也允许可写挂载，2026-09-17 用户定）。
pub struct FormattedPool {
    pub paths: Vec<PathBuf>,
    /// `take` 出去就是「进程退出、镜像关掉」。
    pub devices: Option<Vec<(DeviceIdentity, Recorded)>>,
    pub stream: SharedStream,
    pub genesis: MakeFilesystemOutput,
    pub mkfs_operation_count: usize,
}

impl Drop for FormattedPool {
    fn drop(&mut self) {
        for path in &self.paths {
            let _ = std::fs::remove_file(path);
        }
    }
}

impl FormattedPool {
    pub fn retained_operations(&self) -> Vec<RetainedOperation> {
        self.stream.retained_operations()
    }
    pub fn memory_pool(&self) -> MemoryPool {
        memory_pool_of(&self.retained_operations())
    }
    pub fn memory_pool_after_mkfs(&self) -> MemoryPool {
        memory_pool_of(&self.retained_operations()[..self.mkfs_operation_count])
    }
    pub fn reopen_recorded(&mut self) -> Vec<(DeviceIdentity, Recorded)> {
        drop(self.devices.take());
        reopen_recorded_images(&self.paths, &self.stream)
    }
    pub fn reopen_cold(&mut self) -> Vec<(DeviceIdentity, FileBackedBlockDevice)> {
        drop(self.devices.take());
        reopen_cold_images(&self.paths)
    }
}

/// 一个崩溃状态的两块内存盘：mkfs 之后的基线再施加持久了的那几条写，外面包录制器、录进一条新流（挂载之后流里多一步就是发了写或屏障）。
pub fn crash_state_devices(
    base: &MemoryPool,
    writes: &[singlefs_harness::memory_pool::RetainedWrite],
    persisted: &[bool],
    stream: &SharedStream,
) -> Vec<(
    DeviceIdentity,
    RecordingBlockDevice<singlefs_harness::memory_pool::SparseBlockDevice>,
)> {
    base.devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = singlefs_harness::memory_pool::SparseBlockDevice::new(
                IMAGE_BYTES,
                PhysicalBlockSizeInBytes(512),
            );
            device.image = sparse.clone();
            for (write, is_persisted) in writes.iter().zip(persisted) {
                if *is_persisted && write.device == *identity {
                    write.contents.apply_to(&mut device.image, write.offset);
                }
            }
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect()
}

/// 这几块内存盘此刻的整份镜像。
pub fn memory_pool_of_sparse_devices(
    devices: &[(
        DeviceIdentity,
        RecordingBlockDevice<singlefs_harness::memory_pool::SparseBlockDevice>,
    )],
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

/// 盘上可比的一份快照：两盘各两个系统配置槽的原样字节、根环里全部自证过的根、录制流里已有几步。挂载或抬 F 被拒之后与拒之前逐项相等，
/// 才算「在任何写之前拒绝」（录制流不多一步 = 一个写、一道屏障都没发）。
#[derive(Debug, PartialEq, Eq)]
pub struct DiskSnapshot {
    pub system_configuration_slots: Vec<Vec<u8>>,
    pub readable_roots: Vec<singlefs_core::root_record::RootRecord>,
    pub recorded_operations: usize,
}

pub fn disk_snapshot(image: &MemoryPool, stream: &SharedStream) -> DiskSnapshot {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes =
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut system_configuration_slots = Vec::new();
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for offset in [0, spacing] {
            system_configuration_slots.push(
                PoolReader::read(
                    image,
                    device,
                    singlefs_core::address::DeviceOffsetInBytes(offset),
                    slot_bytes,
                )
                .expect("系统配置槽读得到"),
            );
        }
    }
    let system_configuration =
        singlefs_core::recovery::choose_system_configuration(image).expect("系统配置");
    DiskSnapshot {
        system_configuration_slots,
        readable_roots: singlefs_core::recovery::readable_roots(
            image,
            &system_configuration.immutable.region_devices,
            &system_configuration.immutable.sizes,
            &system_configuration.immutable.filesystem_identifier,
        ),
        recorded_operations: stream.operations().len(),
    }
}

/// 新池新建文件之后，在同一个进程里对同一个文件覆盖写一次（发布 B 起的每一次覆盖写走这一条）：
/// 错误原样交回，要不要 `expect` 由调用方定。
pub fn publish_overwrite_in_process(
    pool: &mut BuiltPool,
    previous: &TransactionOutput,
    content: &[u8],
    write_time_seconds: u64,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        previous,
        FirstFile {
            content,
            write_time_seconds,
        },
        instance,
    )
}

/// 「A 的根落了、之后 B、C 两次单条发布只剩记录与单元」那段历史里 B、C 两版文件的字节数：两版都只切出一条记录，长度各不相同。
/// 在飞上限用例与挂载读盘次序用例共用这段历史（`journal_replay_applies_no_more_records_than_the_in_flight_limit.rs`、
/// `mounted_recovery_reads_each_candidate_once_in_order.rs`）。
pub const CONTENT_BYTES_OF_PUBLISH_B: usize = 4100;
pub const CONTENT_BYTES_OF_PUBLISH_C: usize = 2500;
/// 覆盖写那几版的写入时间比池里第一版晚这么多秒。
pub const OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE: u64 = 60;
/// 改坏根槽时翻的那个字节在根记录里的下标：落在树表指针那一段（根记录 magic 4 + fsid 16 + flags 4 + 实例代号 4 + checkpoint_txg 8 之后），
/// 翻了自证校验和就对不上。
pub const BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER: u64 = 100;

/// 同一个进程、实例 1 里再发布一版（一个数据单元的内容，一次发布一条记录），写入时间取第一版之后
/// [`OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE`] 秒；发布完把它记成池的当前版，下一次接在它后面。
pub fn overwrite_in_process_as_instance_one_and_advance_the_pool(
    pool: &mut BuiltPool,
    content: &[u8],
) -> TransactionOutput {
    let previous = pool.output.clone();
    let output = publish_overwrite_in_process(
        pool,
        &previous,
        content,
        FIXED_WRITE_TIME_SECONDS + OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
        InstanceGeneration(1),
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// 同一个进程、实例 1 里顺序写一版（N 个数据单元的内容，一次发布 N 条记录：前 N − 1 条各只点名自己那个数据单元），
/// 写入时间与发布完怎么记同 [`overwrite_in_process_as_instance_one_and_advance_the_pool`]。
pub fn sequential_write_in_process_as_instance_one_and_advance_the_pool(
    pool: &mut BuiltPool,
    content: &[u8],
) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish_sequential_write(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
        },
        InstanceGeneration(1),
    )
    .expect("顺序写");
    pool.output = output.clone();
    output
}

/// 把 txg 那一版的根槽改坏：只留记录与单元，模拟「根槽没持久」。
pub fn tear_the_root_slot_written_for_txg(image: &mut MemoryPool, txg: u64) {
    let target = target_for_publish(
        CheckpointTxg(txg),
        parameters().geometry.root_ring_slots_per_region,
    );
    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    image.flip_byte(
        device,
        slot_offset(target, parameters().geometry.fixed_structure_slot_spacing),
        BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER,
    );
}

/// 把默认环里 `counter` 那条 journal 记录的字段改一处，两盘各一份，`JournalRecord::to_bytes` 重封载荷与头部校验和；
/// 它后面一直到环里第一条解不出的记录，每一条的反向链按前一条改后的记录头重算、重封（D23（journal 的角色与格式） 已定项 8）：
/// 改过的记录照样自证得过、下一条的反向链也接得上，拦住它的只剩被改的那个字段。
pub fn rewrite_journal_record_and_rechain_the_records_after_it(
    image: &mut MemoryPool,
    counter: u64,
    change: impl Fn(&mut JournalRecord),
) {
    let filesystem_identifier = unit_filesystem_identifier(&parameters().filesystem_identifier);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut record = JournalRecord::parse(
            &journal_record_bytes_in_the_default_ring(image, device, counter),
            filesystem_identifier,
        )
        .expect("改之前这条记录自证过");
        change(&mut record);
        image.devices.get_mut(&device).expect("有这块盘").write(
            record_offset(counter, JOURNAL_RING_DEFAULT_BYTES),
            &record.to_bytes(),
        );
        // 迭代次数的上界是这段历史在 `counter` 之后写过的记录条数：环里下一格没写过、解不出就是出口。
        let mut later_counter = counter + 1;
        while let Some(mut later) = JournalRecord::parse(
            &journal_record_bytes_in_the_default_ring(image, device, later_counter),
            filesystem_identifier,
        ) {
            later.back_chain = back_chain_of(&journal_record_bytes_in_the_default_ring(
                image,
                device,
                later_counter - 1,
            ));
            image.devices.get_mut(&device).expect("有这块盘").write(
                record_offset(later_counter, JOURNAL_RING_DEFAULT_BYTES),
                &later.to_bytes(),
            );
            later_counter += 1;
        }
    }
}

/// 默认环里计数器 `counter` 那一格在 `device` 上的 4096 字节（没写过的格读出来是全 0）。
fn journal_record_bytes_in_the_default_ring(
    image: &MemoryPool,
    device: DeviceIdentity,
    counter: u64,
) -> Vec<u8> {
    PoolReader::read(
        image,
        device,
        record_offset(counter, JOURNAL_RING_DEFAULT_BYTES),
        usize::try_from(JOURNAL_RECORD_BYTES).expect("4096"),
    )
    .expect("环里那一格读得到")
}

/// 根环里每条自证过的根（`readable_roots`），把两块盘 journal 默认环里同 (实例, txg) 的记录的新根段改成照它的那七段
/// （树表指针、中央映射树根指针、树 ID 水位、回退下界 F、实例表指针、分配记录树根指针、卸载记号；I-8.10（记录新根段与同代根相同）
/// 判前四段，D23（journal 的角色与格式） 已定项 15 施加时七段都取记录里的，D23（journal 的角色与格式） 已定项 4：
/// 一次发布的每条记录带同一份新根段），`JournalRecord::to_bytes` 重封；再按计数器从小到大，前一条被改过、又与这一条同实例的，
/// 这一条的反向链按前一条改后的记录头重算、重封（I-8.6（反向链算法）只链本实例内逻辑前一条）。
/// 坏镜像改了根（新整单元校验和沿引用链补到根槽、改树表指针）之后调它：盘上留下的就是「写者从一开始就写出了这样的根」的样子，
/// 只改根不改记录的镜像每一份都会多红 I-8.10。同 (实例, txg) 有几条根时照第一条（I-8.10 那一行把它归 C88，这几份镜像上没有）；
/// 环里从计数器 1 起读到第一格解不出就停。什么都不用改时盘面逐字节不变。交回改了几条记录（按盘各算一条）。
///
/// # Panics
/// 镜像上择不出系统配置。
pub fn copy_each_root_into_the_new_root_segment_of_its_journal_records(
    image: &mut MemoryPool,
) -> usize {
    let system_configuration =
        choose_system_configuration(&*image).expect("坏镜像只改了单元与根槽，系统配置择得出");
    let mut root_of_each_instance_and_txg: std::collections::BTreeMap<
        (InstanceGeneration, CheckpointTxg),
        singlefs_core::root_record::RootRecord,
    > = std::collections::BTreeMap::new();
    for root in readable_roots(
        &*image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    ) {
        root_of_each_instance_and_txg
            .entry((root.instance, root.checkpoint_txg))
            .or_insert(root);
    }
    let filesystem_identifier = unit_filesystem_identifier(&parameters().filesystem_identifier);
    let record_slots_in_the_ring = JOURNAL_RING_DEFAULT_BYTES / JOURNAL_RECORD_BYTES;
    let mut records_rewritten = 0;
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        // 跨轮携带：计数器小 1 的那一条若被改过，记它的实例与改后的字节（下一条的反向链按它重算）。
        let mut rewritten_previous_record: Option<(InstanceGeneration, Vec<u8>)> = None;
        // 迭代次数的上界是环的槽数；环里第一格解不出的记录就是出口。
        for counter in 1..=record_slots_in_the_ring {
            let bytes = journal_record_bytes_in_the_default_ring(image, device, counter);
            let Some(record) = JournalRecord::parse(&bytes, filesystem_identifier) else {
                break;
            };
            let mut rewritten = record.clone();
            if let Some(root) =
                root_of_each_instance_and_txg.get(&(record.instance, record.checkpoint_txg))
            {
                rewritten.new_tree_table = root.tree_table;
                rewritten.new_mapping_root = root.mapping_root;
                rewritten.new_tree_identifier_watermark = root.tree_identifier_watermark;
                rewritten.new_rollback_floor = root.rollback_floor;
                rewritten.new_instance_table = root.instance_table;
                rewritten.new_allocation_record_tree_root = root.allocation_record_tree_root;
                rewritten.new_unmount_marker = root.unmount_marker;
            }
            if let Some((previous_instance, previous_bytes)) = &rewritten_previous_record {
                if *previous_instance == record.instance {
                    rewritten.back_chain = back_chain_of(previous_bytes);
                }
            }
            rewritten_previous_record = if rewritten == record {
                None
            } else {
                let rewritten_bytes = rewritten.to_bytes();
                image.devices.get_mut(&device).expect("有这块盘").write(
                    record_offset(counter, JOURNAL_RING_DEFAULT_BYTES),
                    &rewritten_bytes,
                );
                records_rewritten += 1;
                Some((record.instance, rewritten_bytes))
            };
        }
    }
    records_rewritten
}

/// 块层的一次读：哪块盘、从哪个字节起、多长。读盘次序的用例（只读挂载、可写挂载）逐次记这一项。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockLayerRead {
    pub device: DeviceIdentity,
    pub offset: DeviceOffsetInBytes,
    pub length_in_bytes: usize,
}

/// 反向链重读从 `device_identities()` 的第一块盘试起，健康镜像上第一份就对得上。
pub const FIRST_DEVICE_TRIED_BY_THE_BACK_CHAIN_REREAD: DeviceIdentity = DeviceIdentity(0);

/// 一个点名单元的整单元宽：数据单元与码 3 打包记录单元 32768，码 2 索引节点 16384（D23（journal 的角色与格式） 已定项 14 第四条按类读）。
pub fn named_unit_bytes(unit_class: u8) -> usize {
    let unit_bytes = match unit_class {
        UNIT_CLASS_DATA | UNIT_CLASS_PACKED => DATA_UNIT_BYTES,
        UNIT_CLASS_INDEX_NODE => NODE_BYTES,
        unknown_class => {
            panic!("发布路径只点名数据单元、索引节点与打包记录单元，读到类 {unknown_class}")
        }
    };
    usize::try_from(unit_bytes).expect("单元宽装得进 usize")
}

/// 候选 `record` 的反向链那一判在块层的读：在 0 号盘上重读它的上一条记录（`ring_bytes` 长的环里的落点）。
pub fn block_read_rereading_the_record_before(
    record: &JournalRecord,
    ring_bytes: u64,
) -> BlockLayerRead {
    BlockLayerRead {
        device: FIRST_DEVICE_TRIED_BY_THE_BACK_CHAIN_REREAD,
        offset: record_offset(record.counter - 1, ring_bytes),
        length_in_bytes: usize::try_from(JOURNAL_RECORD_BYTES).expect("4096"),
    }
}

/// 候选 `record` 的点名验证在块层的读：它点名的每个单元的两份，按点名次序。
pub fn block_reads_of_the_named_units_of(record: &JournalRecord) -> Vec<BlockLayerRead> {
    record
        .named
        .iter()
        .flat_map(|named| {
            named.locations.iter().map(|location| BlockLayerRead {
                device: location.device,
                offset: location.slot.to_device_offset(),
                length_in_bytes: named_unit_bytes(named.unit_class),
            })
        })
        .collect()
}

pub fn is_inside_the_journal_ring(offset: DeviceOffsetInBytes, ring_bytes: u64) -> bool {
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    offset.0 >= ring_start && offset.0 < ring_start + ring_bytes
}

/// 把计数器 `counters` 那几条记录从默认环的落点搬到 `ring_bytes` 长的环的落点（两盘各一份；先全部读出再写，搬过去的会盖掉更早的记录）。
/// 小环过不了 mkfs 与读者择系统配置，在飞上限那一形的用例在默认环建的池上这样摆记录，再拿环长改小的那份系统配置扫环。
pub fn image_with_the_records_relocated_onto_a_ring_of(
    image: &MemoryPool,
    counters: &[u64],
    ring_bytes: u64,
) -> MemoryPool {
    let mut relocated = image.clone();
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let records: Vec<(u64, Vec<u8>)> = counters
            .iter()
            .map(|counter| {
                (
                    *counter,
                    journal_record_bytes_in_the_default_ring(image, device, *counter),
                )
            })
            .collect();
        for (counter, bytes) in records {
            relocated
                .devices
                .get_mut(&device)
                .expect("有这块盘")
                .write(record_offset(counter, ring_bytes), &bytes);
        }
    }
    relocated
}

/// 把 [`system_configuration_slots_of_every_device`] 存下的系统配置槽字节写回镜像（盘在盘表里的位置 `index` 就是 `DeviceIdentity(index)`）：
/// 造「根槽没落、它之后的系统配置轮换也没落」的崩溃状态（D16（发布语义） 已定项 7：轮换在根槽 FUA 之后）。
pub fn write_back_system_configuration_slots(
    image: &mut MemoryPool,
    saved: &[(usize, DeviceOffsetInBytes, Vec<u8>)],
) {
    for (index, offset, bytes) in saved {
        image
            .devices
            .get_mut(&DeviceIdentity(
                u32::try_from(*index).expect("盘在盘表里的位置装得进 u32"),
            ))
            .expect("有这块盘")
            .write(*offset, bytes);
    }
}

pub fn format_pool(tag: &str) -> FormattedPool {
    let (paths, devices, stream, genesis, mkfs_operation_count) = formatted_recorded_images(tag);
    FormattedPool {
        paths,
        devices: Some(devices),
        stream,
        genesis,
        mkfs_operation_count,
    }
}

pub fn build_pool(tag: &str) -> BuiltPool {
    let parameters = parameters();
    let (paths, mut devices, stream, genesis, mkfs_operation_count) =
        formatted_recorded_images(tag);
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
    // mkfs 同一个进程里接着写的会话：装上 mkfs 刚写下的根环，转过根环之后照可再分配谓词回收（C551；不装的话记根就断言）。
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &parameters,
        &genesis,
    ));
    let content = file_content();
    let warm_up_operation_count;
    let (warm_up, output) = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        assert_eq!(instance, InstanceGeneration(1));
        let warm_up = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        // 暖机已调完、发布还没调：这里就是暖机与新池新建文件的分界，不从流尾往回数几步
        // （里程碑「覆盖写、释放、回退与复用」收尾批「55 号装置步数」）。
        warm_up_operation_count = stream.operations().len();
        let output = publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up.last_record_bytes,
        )
        .expect("新池新建文件");
        (warm_up, output)
    };
    BuiltPool {
        paths,
        devices: Some(devices),
        stream,
        genesis,
        warm_up,
        output,
        allocator,
        mkfs_operation_count,
        warm_up_operation_count,
    }
}

/// 崩溃恢复造出一条被抛弃的根（第一轮判决 H6 那一形；D23（journal 的角色与格式） 已定项 14 射程：影子账与按实例表判抛弃留着，
/// 理由是崩溃恢复）：进程退出之后，`newest`（根环里最新的那条根那一版）的根槽与它的数据单元（两份）暂时读不出（暂存、清零），
/// 它那次发布之后轮换写的系统配置槽（每块盘一槽）坏掉、系统配置没见证到它（清零、不写回；见证在时 C554 乙拒可写，造不出被抛弃的根），
/// 重开可写挂载——择根落到它前一条根，它那条记录施加前验点名单元失败、不施加；新实例写行与暖机。再把暂存的字节原样写回：
/// `newest` 的根又读得出，按新实例的实例表判是被抛弃的。池换成那次挂载交回的分配器与现行版本（要带文件），交回那次挂载——
/// 它看不见 `newest`，影子账隔离 0；之后的挂载才看得见。
pub fn abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(
    pool: &mut BuiltPool,
    newest: &TransactionOutput,
) -> singlefs_core::mount::Mounted {
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before_with_the_shadow_ledger(
        pool,
        newest,
        singlefs_core::mount::ShadowLedger::On,
    )
}

/// 同 [`abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`]，那次挂载的影子账那一臂由调用方给（只供测试的开关）。
/// `ShadowLedger::Off` 连 C694 的 C（水位之上没施加的记录点名的落点隔离，删除线第 33 批 T）一起关：那次挂载真看不见 `newest`，
/// 它的分配器把 `newest` 写出的单元当空闲槽——C 之后产品路径上 `newest` 的记录读得出时造不出这一形，checker 那几条要这一形的坏镜像用它造。
pub fn abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before_with_the_shadow_ledger(
    pool: &mut BuiltPool,
    newest: &TransactionOutput,
    shadow_ledger: singlefs_core::mount::ShadowLedger,
) -> singlefs_core::mount::Mounted {
    let data_unit_bytes = usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768");
    let data_units_of_the_newest = newest
        .data_pointers
        .iter()
        .flat_map(|pointer| pointer.locations)
        .map(|location| {
            (
                location.device,
                location.slot.to_device_offset(),
                data_unit_bytes,
            )
        })
        .collect();
    abandon_the_newest_root_by_a_recovery_hiding_its_root_slot_and(
        pool,
        newest,
        data_units_of_the_newest,
        shadow_ledger,
    )
}

/// 同 [`abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`]，暂时读不出的是 `newest` 的根槽与它那条末条记录点名的打包记录单元
/// （码 3，两份）：没写数据单元的发布（建空 inode 那一种）也被抛弃得了——它那条记录点名的 inode 叶容器两份都读不出，施加前验点名单元失败、不施加
/// （D23（journal 的角色与格式） 已定项 14 前缀第四条）。影子账开着；交回那次挂载。
pub fn abandon_the_newest_root_by_a_recovery_hiding_the_packed_units_its_record_names(
    pool: &mut BuiltPool,
    newest: &TransactionOutput,
) -> singlefs_core::mount::Mounted {
    let packed_units_named_by_the_newest = newest
        .record
        .named
        .iter()
        .filter(|named| named.unit_class == UNIT_CLASS_PACKED)
        .flat_map(|named| {
            named.locations.iter().map(|location| {
                (
                    location.device,
                    location.slot.to_device_offset(),
                    named_unit_bytes(named.unit_class),
                )
            })
        })
        .collect::<Vec<_>>();
    assert!(
        !packed_units_named_by_the_newest.is_empty(),
        "`newest` 那条记录点名了至少一片打包记录单元（建 inode 的发布重写 inode 叶容器）"
    );
    abandon_the_newest_root_by_a_recovery_hiding_its_root_slot_and(
        pool,
        newest,
        packed_units_named_by_the_newest,
        singlefs_core::mount::ShadowLedger::On,
    )
}

/// W3 那一段历史（D23（journal 的角色与格式） 已定项 14「水位」一格的 Opt2、D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」）：
/// 第一个文件那一版 A（实例 1，inode 号水位 2）之后建三个空 inode 的那一版 `abandoned`（号 2、3、4，水位 5）；崩溃恢复抛弃它
/// （[`abandon_the_newest_root_by_a_recovery_hiding_the_packed_units_its_record_names`]：恢复落到 A，实例 2 写行、暖机，再把它的根槽与
/// inode 叶容器写回——它的根读得出、从盘上重建得出水位 5，按实例 2 的实例表判是被抛弃的）；实例 2 再建一个空 inode 的那一版 `created_again`
/// （号 2 又发一次：被抛弃那段里发过的号，D8（核心索引结构） 已定项 6 射程允许重发；水位 3）；挂着的时候管理员回退到 A 的那一版 `rollback`。
pub struct RollbackAfterARecoveryAbandonedARootThatCreatedInodes {
    pub pool: BuiltPool,
    pub first_file_version: TransactionOutput,
    pub abandoned: TransactionOutput,
    pub created_again: TransactionOutput,
    pub rollback: TransactionOutput,
}

/// 搭 [`RollbackAfterARecoveryAbandonedARootThatCreatedInodes`] 那一段历史。搭建自证：回退之前被抛弃那一版的根在根环里读得出、
/// 从盘上重建得出水位 5（不然「回退只取没被抛弃的根」那一判在这段历史上量不出什么）。
pub fn rollback_after_a_recovery_abandoned_a_root_that_created_inodes(
    tag: &str,
) -> RollbackAfterARecoveryAbandonedARootThatCreatedInodes {
    const NEW_INODES_OF_THE_ABANDONED_PUBLISH: u64 = 3;
    let mut pool = build_pool(tag);
    let first_file_version = pool.output.clone();
    let publish_new_inodes_in_process =
        |pool_to_publish_on: &mut BuiltPool, count: u64, instance: InstanceGeneration| {
            let publish_parameters = parameters();
            let previous = pool_to_publish_on.output.clone();
            let devices = pool_to_publish_on.devices.as_mut().expect("镜像还开着");
            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
            let output = singlefs_core::transaction::publish_new_inodes(
                &mut writer,
                &mut pool_to_publish_on.allocator,
                &previous,
                count,
                FIXED_WRITE_TIME_SECONDS + 60,
                instance,
            )
            .expect("建空 inode");
            pool_to_publish_on.output = output.clone();
            output
        };
    let abandoned = publish_new_inodes_in_process(
        &mut pool,
        NEW_INODES_OF_THE_ABANDONED_PUBLISH,
        InstanceGeneration(1),
    );
    assert_eq!(
        abandoned.inode_number_watermark(),
        2 + NEW_INODES_OF_THE_ABANDONED_PUBLISH,
        "建三个空 inode（号 2、3、4）之后水位 5"
    );
    let mounted = abandon_the_newest_root_by_a_recovery_hiding_the_packed_units_its_record_names(
        &mut pool, &abandoned,
    );
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        (
            first_file_version.root.instance,
            first_file_version.root.checkpoint_txg
        ),
        "崩溃恢复落到 A，建 inode 那一版的记录不施加"
    );
    match singlefs_core::recovery::rebuild_version(
        &pool.memory_pool(),
        &abandoned.root,
        Some(abandoned.record.clone()),
    ) {
        Ok(
            singlefs_core::recovery::RebuiltVersion::WithFile(version)
            | singlefs_core::recovery::RebuiltVersion::WithTreesAndNoFile(version),
        ) => assert_eq!(
            version.inode_number_watermark(),
            2 + NEW_INODES_OF_THE_ABANDONED_PUBLISH,
            "被抛弃那一版从盘上重建得出水位 5"
        ),
        other => panic!("被抛弃那一版从盘上重建得出：{other:?}"),
    }
    let created_again = publish_new_inodes_in_process(&mut pool, 1, mounted.output.instance);
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    singlefs_core::mount::roll_back_by_a_forward_publish(
        &publish_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        singlefs_core::mount::RollbackTarget {
            instance: first_file_version.root.instance,
            checkpoint_txg: first_file_version.root.checkpoint_txg,
        },
    )
    .expect("挂着的时候回退到 A");
    let rollback = pool.output.clone();
    RollbackAfterARecoveryAbandonedARootThatCreatedInodes {
        pool,
        first_file_version,
        abandoned,
        created_again,
        rollback,
    }
}

/// 崩溃恢复抛弃 `newest` 的本体：根槽与 `units_hidden_besides_the_root_slot`（盘、字节偏移、长度）暂时读不出，其余见
/// [`abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`]。
fn abandon_the_newest_root_by_a_recovery_hiding_its_root_slot_and(
    pool: &mut BuiltPool,
    newest: &TransactionOutput,
    units_hidden_besides_the_root_slot: Vec<(DeviceIdentity, DeviceOffsetInBytes, usize)>,
    shadow_ledger: singlefs_core::mount::ShadowLedger,
) -> singlefs_core::mount::Mounted {
    use singlefs_core::block_device::{BlockDevice, WriteDurability};
    let publish_parameters = parameters();
    let target = singlefs_core::root_ring::target_for_publish(
        newest.root.checkpoint_txg,
        publish_parameters.geometry.root_ring_slots_per_region,
    );
    let root_slot_device =
        publish_parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_slot_offset = singlefs_core::root_ring::slot_offset(
        target,
        publish_parameters.geometry.fixed_structure_slot_spacing,
    );
    let root_slot_bytes =
        usize::try_from(publish_parameters.geometry.physical_block_size).expect("根槽宽");
    fn index_of_device(devices: &[(DeviceIdentity, Recorded)], identity: DeviceIdentity) -> usize {
        devices
            .iter()
            .position(|(candidate, _)| *candidate == identity)
            .expect("池里有这块盘")
    }
    let mut devices = pool.reopen_recorded();
    let places_to_hide = std::iter::once((root_slot_device, root_slot_offset, root_slot_bytes))
        .chain(units_hidden_besides_the_root_slot);
    let mut saved: Vec<(
        DeviceIdentity,
        singlefs_core::address::DeviceOffsetInBytes,
        Vec<u8>,
    )> = Vec::new();
    for (identity, offset, length) in places_to_hide {
        let index = index_of_device(&devices, identity);
        let mut bytes = vec![0u8; length];
        devices[index].1.read_at(offset, &mut bytes).expect("暂存");
        devices[index]
            .1
            .write_at(offset, &vec![0u8; length], WriteDurability::Plain)
            .expect("清零");
        saved.push((identity, offset, bytes));
    }
    // 系统配置没见证到 `newest`：C554 乙之后崩溃恢复还抛弃得了最新那条根的只剩这一形（系统配置在根槽 FUA 之后才轮换，D16（发布语义） 已定项 7；
    // 见证在时可写挂载重读一次仍读不出它就拒可写，`MountError::NewerStateStillUnreadableAfterOneReread`）：崩在 `newest` 的根槽 FUA 之后、
    // 它那次系统配置轮换写之前。轮换写那一槽（每块盘世代号最大的那一槽，槽号 = 世代号 mod 2）因此还是再前一次轮换留下的内容：
    // 世代号 g − 2、自证照过。录制流只记内容哈希、拿不回那份字节，这里拿另一槽（世代号 g − 1，见证的是 `newest` 前一次发布）的内容改世代号为 g − 2
    // 重新编码写回——见证值与那份真内容一样停在 `newest` 之前，两槽都自证得过。不清零：清零造的是撕裂轮换那一形（取甲时那一形拒可写；
    // C331（择根倒挂压过已确认的写） 改取 R2 之后不拒，但见证值取的是另一槽、与真内容不同）。
    let slot_spacing_in_bytes = u64::from(publish_parameters.geometry.fixed_structure_slot_spacing);
    let rotation_not_written_of_each_device: Vec<(
        singlefs_core::address::DeviceOffsetInBytes,
        Vec<u8>,
    )> = devices
        .iter()
        .map(|(identity, _)| {
            let mut verified_slots = singlefs_core::recovery::verified_system_configuration_slots(
                devices.as_slice(),
                *identity,
                slot_spacing_in_bytes,
                &publish_parameters.filesystem_identifier,
            );
            verified_slots.sort_by_key(|slot| slot.quantities.slot_generation);
            let [older, newest_slot] =
                <[_; 2]>::try_from(verified_slots).expect("造被抛弃的根之前每块盘两槽都自证得过");
            let newest_generation = newest_slot.quantities.slot_generation;
            let mut rotation_not_written = older;
            rotation_not_written.quantities.slot_generation = newest_generation
                .checked_sub(2)
                .expect("`newest` 之前至少还有两次轮换");
            (
                singlefs_core::address::DeviceOffsetInBytes(
                    (newest_generation % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
                        * slot_spacing_in_bytes,
                ),
                rotation_not_written.to_slot(),
            )
        })
        .collect();
    for ((_, device), (newest_slot, rotation_not_written)) in
        devices.iter_mut().zip(rotation_not_written_of_each_device)
    {
        device
            .write_at(newest_slot, &rotation_not_written, WriteDurability::Plain)
            .expect("见证 newest 的系统配置槽写回成轮换之前的那一代");
    }
    let mounted = singlefs_core::mount::mount_writable_with_test_only_switches(
        &publish_parameters,
        &mut devices,
        singlefs_core::admission::SpaceAdmission::JudgedByTheFormula,
        shadow_ledger,
    )
    .expect("最新那条根与它的数据单元读不出：择根落到前一条根，照常可写挂载");
    for (identity, offset, bytes) in &saved {
        let index = index_of_device(&devices, *identity);
        devices[index]
            .1
            .write_at(*offset, bytes, WriteDurability::Plain)
            .expect("原样写回");
    }
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator.clone();
    pool.output = mounted
        .current
        .file_version()
        .expect("落到的那一版带文件")
        .clone();
    mounted
}

/// 一段读坏的落点（C554 乙的用例用：暂时读不出的根槽、一次发布点名的单元、实例表那一片）：哪块盘、从哪个字节起、多长，这一段哪几次读坏。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnreadableRange {
    pub device: DeviceIdentity,
    pub offset_in_bytes: u64,
    pub length_in_bytes: u64,
    pub failing_reads: FailingReadsOfARange,
}

impl UnreadableRange {
    fn overlaps(&self, device: DeviceIdentity, offset_in_bytes: u64, length_in_bytes: u64) -> bool {
        self.device == device
            && offset_in_bytes < self.offset_in_bytes + self.length_in_bytes
            && self.offset_in_bytes < offset_in_bytes + length_in_bytes
    }
}

/// 一段在它那块盘上哪几次读坏：按这一段自己被读的次数从 1 数（一次读碰到这一段就算它被读一次）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailingReadsOfARange {
    /// 一次都不坏，只数它被读了几次。
    Never,
    /// 撤掉之前每次都坏。
    Every,
    /// 只有第一次坏，之后照读（只坏一次的瞬时读错）。
    OnlyTheFirst,
    /// 第 n 次起每次都坏（之前那几次照读）。
    FromTheNthOnward(u64),
}

/// 读坏的那一次交回什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnreadableRangeReadBack {
    /// 块设备报错（`BlockDeviceError::InputOutput`），缓冲区不动。
    DeviceError,
    /// 报成功，落在读坏那几段里的字节全是 0（历史执行器「崩溃恢复抛弃根」那一步的造法）；段外的字节照读。
    Zeros,
}

/// 读故障在不在：用例的钩子（`mount::BeforeTheOneReread::CallTheTestOnlyHookFirst`）里撤掉，之后每次读都照读。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnreadableRangesPresence {
    InPlace,
    Lifted,
}

struct UnreadableRangesState {
    ranges: Vec<UnreadableRange>,
    read_back: UnreadableRangeReadBack,
    presence: UnreadableRangesPresence,
    /// 与 `ranges` 同序：每一段被读了几次（撤掉之后的读照数）。
    reads_of_each_range: Vec<u64>,
    /// 读坏了几次（一次读碰到几段、只要有一段坏，算一次）。
    failed_reads: u64,
}

/// 几块盘共用的一份读故障：几段落点、读坏的那一次交回什么、撤没撤，每一段被读了几次。
#[derive(Clone)]
pub struct SharedUnreadableRanges(std::rc::Rc<std::cell::RefCell<UnreadableRangesState>>);

impl SharedUnreadableRanges {
    #[must_use]
    pub fn new(ranges: Vec<UnreadableRange>, read_back: UnreadableRangeReadBack) -> Self {
        let range_count = ranges.len();
        Self(std::rc::Rc::new(std::cell::RefCell::new(
            UnreadableRangesState {
                ranges,
                read_back,
                presence: UnreadableRangesPresence::InPlace,
                reads_of_each_range: vec![0; range_count],
                failed_reads: 0,
            },
        )))
    }

    /// 撤掉读故障：之后每次读都照读（计数照记）。
    pub fn lift(&self) {
        self.0.borrow_mut().presence = UnreadableRangesPresence::Lifted;
    }

    /// 第 `index` 段（按 `new` 交进来的次序）被读了几次。
    #[must_use]
    pub fn reads_of_range(&self, index: usize) -> u64 {
        self.0.borrow().reads_of_each_range[index]
    }

    /// 读坏了几次。
    #[must_use]
    pub fn failed_reads(&self) -> u64 {
        self.0.borrow().failed_reads
    }

    /// 记下这一次读碰到的每一段被读了一次，交回这一次读里读坏的那几段（读故障撤了、或碰到的段这一次都不坏时为空）。
    fn note_a_read(
        &self,
        device: DeviceIdentity,
        offset_in_bytes: u64,
        length_in_bytes: u64,
    ) -> Vec<UnreadableRange> {
        let mut state = self.0.borrow_mut();
        let UnreadableRangesState {
            ranges,
            presence,
            reads_of_each_range,
            failed_reads,
            ..
        } = &mut *state;
        let mut failing = Vec::new();
        for (range, reads_of_this_range) in ranges.iter().zip(reads_of_each_range.iter_mut()) {
            if !range.overlaps(device, offset_in_bytes, length_in_bytes) {
                continue;
            }
            *reads_of_this_range += 1;
            let fails_this_time = match range.failing_reads {
                FailingReadsOfARange::Never => false,
                FailingReadsOfARange::Every => true,
                FailingReadsOfARange::OnlyTheFirst => *reads_of_this_range == 1,
                FailingReadsOfARange::FromTheNthOnward(first_failing) => {
                    *reads_of_this_range >= first_failing
                }
            };
            if *presence == UnreadableRangesPresence::InPlace && fails_this_time {
                failing.push(*range);
            }
        }
        if !failing.is_empty() {
            *failed_reads += 1;
        }
        failing
    }

    fn read_back(&self) -> UnreadableRangeReadBack {
        self.0.borrow().read_back
    }
}

/// 读的时候按 [`SharedUnreadableRanges`] 读坏几段的盘；写、写零、屏障原样交给里面那块盘。
pub struct DeviceWithUnreadableRanges<Inner: singlefs_core::block_device::BlockDevice> {
    identity: DeviceIdentity,
    inner: Inner,
    ranges: SharedUnreadableRanges,
}

impl<Inner: singlefs_core::block_device::BlockDevice> singlefs_core::block_device::BlockDevice
    for DeviceWithUnreadableRanges<Inner>
{
    fn read_at(
        &self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        let length_in_bytes = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        let failing = self
            .ranges
            .note_a_read(self.identity, offset.0, length_in_bytes);
        if failing.is_empty() {
            return self.inner.read_at(offset, buffer);
        }
        match self.ranges.read_back() {
            UnreadableRangeReadBack::DeviceError => {
                Err(singlefs_core::block_device::BlockDeviceError::InputOutput(
                    std::io::Error::other("用例点名的这一段读坏"),
                ))
            }
            UnreadableRangeReadBack::Zeros => {
                self.inner.read_at(offset, buffer)?;
                for range in failing {
                    let start_in_buffer = range.offset_in_bytes.saturating_sub(offset.0);
                    let end_in_buffer = (range.offset_in_bytes + range.length_in_bytes - offset.0)
                        .min(length_in_bytes);
                    buffer[usize::try_from(start_in_buffer).expect("落在缓冲区里")
                        ..usize::try_from(end_in_buffer).expect("落在缓冲区里")]
                        .fill(0);
                }
                Ok(())
            }
        }
    }

    fn write_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        bytes: &[u8],
        durability: singlefs_core::block_device::WriteDurability,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 把一池盘逐块包上同一份读故障。
pub fn with_unreadable_ranges<Inner: singlefs_core::block_device::BlockDevice>(
    devices: Vec<(DeviceIdentity, Inner)>,
    ranges: &SharedUnreadableRanges,
) -> Vec<(DeviceIdentity, DeviceWithUnreadableRanges<Inner>)> {
    devices
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                DeviceWithUnreadableRanges {
                    identity,
                    inner,
                    ranges: ranges.clone(),
                },
            )
        })
        .collect()
}

/// 拆掉读故障，交回里面那几块盘。
pub fn without_unreadable_ranges<Inner: singlefs_core::block_device::BlockDevice>(
    devices: Vec<(DeviceIdentity, DeviceWithUnreadableRanges<Inner>)>,
) -> Vec<(DeviceIdentity, Inner)> {
    devices
        .into_iter()
        .map(|(identity, device)| (identity, device.inner))
        .collect()
}

/// txg 为 `checkpoint_txg` 的那次发布写的根槽（根环落点公式定的区域与槽，那一个区域住的盘），整个根槽一段。
#[must_use]
pub fn unreadable_root_slot_of(
    checkpoint_txg: singlefs_core::address::CheckpointTxg,
    failing_reads: FailingReadsOfARange,
) -> UnreadableRange {
    let publish_parameters = parameters();
    let target = singlefs_core::root_ring::target_for_publish(
        checkpoint_txg,
        publish_parameters.geometry.root_ring_slots_per_region,
    );
    UnreadableRange {
        device: publish_parameters.region_devices[usize::try_from(target.region).expect("区域号")],
        offset_in_bytes: singlefs_core::root_ring::slot_offset(
            target,
            publish_parameters.geometry.fixed_structure_slot_spacing,
        )
        .0,
        length_in_bytes: u64::from(publish_parameters.geometry.physical_block_size),
        failing_reads,
    }
}

/// 每块盘两个系统配置槽的全部字节，按 (盘在盘表里的位置, 偏移) 存。
pub fn system_configuration_slots_of_every_device(
    pool: &BuiltPool,
) -> Vec<(usize, singlefs_core::address::DeviceOffsetInBytes, Vec<u8>)> {
    use singlefs_core::block_device::BlockDevice;
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes =
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let devices = pool.devices.as_ref().expect("镜像开着");
    let mut saved = Vec::new();
    for (index, (_, device)) in devices.iter().enumerate() {
        for slot in 0..singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
            let offset = singlefs_core::address::DeviceOffsetInBytes(slot * spacing);
            let mut bytes = vec![0u8; slot_bytes];
            device.read_at(offset, &mut bytes).expect("读系统配置槽");
            saved.push((index, offset, bytes));
        }
    }
    saved
}

/// 崩溃恢复造出一条被抛弃的根的另一形（收口表第 43 行调查员的最小复现，`research/prompts/closeout-recheck-2026-09-28/row43-investigator-report.md`）：
/// `third` 的根槽 FUA 持久、系统配置轮换没持久就崩（`system_configuration_before_the_third` 是它那次发布之前每块盘两槽的字节，由
/// [`system_configuration_slots_of_every_device`] 取）；下一次可写挂载时 C 的根槽与数据单元一时读不出（挂载交回之后原样写回）：择根落到 B，
/// C 那条记录施加前验点名单元失败、不施加，新实例 2 写行、暖机。池换成那次挂载交回的分配器与现行版本。
pub fn abandon_the_third_version_by_a_crash_before_its_rotation(
    pool: &mut BuiltPool,
    third: &TransactionOutput,
    system_configuration_before_the_third: &[(
        usize,
        singlefs_core::address::DeviceOffsetInBytes,
        Vec<u8>,
    )],
) {
    use singlefs_core::block_device::BlockDevice;
    let publish_parameters = parameters();
    let target = singlefs_core::root_ring::target_for_publish(
        third.root.checkpoint_txg,
        publish_parameters.geometry.root_ring_slots_per_region,
    );
    let root_slot_device =
        publish_parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_slot_offset = singlefs_core::root_ring::slot_offset(
        target,
        publish_parameters.geometry.fixed_structure_slot_spacing,
    );
    let root_slot_bytes =
        usize::try_from(publish_parameters.geometry.physical_block_size).expect("根槽宽");
    let data_unit_bytes = usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768");
    let mut devices = pool.reopen_recorded();
    for (index, offset, bytes) in system_configuration_before_the_third {
        devices[*index]
            .1
            .write_at(
                *offset,
                bytes,
                singlefs_core::block_device::WriteDurability::Plain,
            )
            .expect("C 的轮换没落盘：退回轮换之前的字节");
    }
    let places_to_hide = std::iter::once((root_slot_device, root_slot_offset, root_slot_bytes))
        .chain(
            third
                .data_pointers
                .iter()
                .flat_map(|pointer| pointer.locations)
                .map(|location| {
                    (
                        location.device,
                        location.slot.to_device_offset(),
                        data_unit_bytes,
                    )
                }),
        );
    let mut hidden = Vec::new();
    for (identity, offset, length) in places_to_hide {
        let index = devices
            .iter()
            .position(|(candidate, _)| *candidate == identity)
            .expect("池里有这块盘");
        let mut bytes = vec![0u8; length];
        devices[index].1.read_at(offset, &mut bytes).expect("暂存");
        devices[index]
            .1
            .write_at(
                offset,
                &vec![0u8; length],
                singlefs_core::block_device::WriteDurability::Plain,
            )
            .expect("一时读不出：清零");
        hidden.push((index, offset, bytes));
    }
    let mounted = singlefs_core::mount::mount_writable(&publish_parameters, &mut devices);
    for (index, offset, bytes) in &hidden {
        devices[*index]
            .1
            .write_at(
                *offset,
                bytes,
                singlefs_core::block_device::WriteDurability::Plain,
            )
            .expect("原样写回");
    }
    let mounted = mounted.expect("择根落到 B，照常可写挂载");
    assert_eq!(
        mounted.output.effective_root.checkpoint_txg,
        singlefs_core::address::CheckpointTxg(4),
        "恢复落到 B"
    );
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator.clone();
    pool.output = mounted
        .current
        .file_version()
        .expect("落到的那一版带文件")
        .clone();
}
