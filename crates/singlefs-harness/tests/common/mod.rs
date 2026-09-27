//! 步 6 / 步 7 验收共用的搭建：两个文件镜像上 mkfs → 取号 → 暖机 → 新池新建文件，录制流开内容保留。
#![allow(dead_code, reason = "每个测试文件各自只用到其中一部分")]

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{FileBackedBlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemOutput, MakeFilesystemParameters, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
    PublishError, TransactionOutput, WarmUpOutput,
};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
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
    use singlefs_core::recovery::PoolReader;
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
    let data_unit_bytes = usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768");
    fn index_of_device(devices: &[(DeviceIdentity, Recorded)], identity: DeviceIdentity) -> usize {
        devices
            .iter()
            .position(|(candidate, _)| *candidate == identity)
            .expect("池里有这块盘")
    }
    let mut devices = pool.reopen_recorded();
    let places_to_hide = std::iter::once((root_slot_device, root_slot_offset, root_slot_bytes))
        .chain(
            newest
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
    // 见证在时可写挂载重读一次仍读不出它就拒可写，`MountError::NewerStateStillUnreadableAfterOneReread`）。`newest` 那次发布之后轮换写的
    // 系统配置槽（每块盘世代号最大的那一槽，槽号 = 世代号 mod 2）坏掉：每块盘两槽轮换，容得下一槽坏（D22（单元原子性怎么合成） 已定项 8）。
    // 清零、不写回：这次挂载的取号写正好落回这一槽（世代号取读得出的最大 + 1）。
    let slot_spacing_in_bytes = u64::from(publish_parameters.geometry.fixed_structure_slot_spacing);
    let system_configuration_slot_bytes =
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let newest_system_configuration_slot_of_each_device: Vec<
        singlefs_core::address::DeviceOffsetInBytes,
    > = devices
        .iter()
        .map(|(identity, _)| {
            let newest_generation = singlefs_core::recovery::verified_system_configuration_slots(
                devices.as_slice(),
                *identity,
                slot_spacing_in_bytes,
                &publish_parameters.filesystem_identifier,
            )
            .into_iter()
            .map(|slot| slot.quantities.slot_generation)
            .max()
            .expect("每块盘都有自证过的系统配置");
            singlefs_core::address::DeviceOffsetInBytes(
                (newest_generation % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
                    * slot_spacing_in_bytes,
            )
        })
        .collect();
    for ((_, device), newest_slot) in devices
        .iter_mut()
        .zip(newest_system_configuration_slot_of_each_device)
    {
        device
            .write_at(
                newest_slot,
                &vec![0u8; system_configuration_slot_bytes],
                WriteDurability::Plain,
            )
            .expect("见证 newest 的系统配置槽清零");
    }
    let mounted = singlefs_core::mount::mount_writable(&publish_parameters, &mut devices)
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
