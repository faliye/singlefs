//! 运行库用例共用的搭建：两块内存里的稀疏盘上 mkfs → 取号 → 暖机 → 新池新建文件（同验证包 `build_pool` 那一串），另一次挂载里顺序写第一个文件，
//! 会话丢掉（「进程退出」）。运行库只依赖核心层，所以盘与搭建在这里另写一份。盘是 `Send` 的（录制流在 `Arc<Mutex>` 里）：维护线程带得走。
//! 盘按 512 字节扇区存（没写过的读回 0），改盘的每个动作按先后记；另带用例摆的几样：数读、下几个写报错、下几个屏障报错。
#![allow(dead_code, reason = "几份测试文件各自 mod 进来，只用到其中一部分")]

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber};
use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator, UnitAreaStart};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::{
    SystemImmutableSizes, JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PartialWriteBytes, PoolWriter,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;

/// 每块盘的字节数（同验证包 `build_pool` 的镜像）。
pub const DEVICE_BYTES: u64 = 4 << 30;
/// 写得满的小池每块盘的字节数：前台写到被拒只要几分钟。
const DEVICE_BYTES_OF_THE_SMALL_POOL: u64 = 320 << 20;
/// 写得满的小池的日志环字节数：单元区起点随它算，别的几何同 [`parameters`]。
const JOURNAL_RING_BYTES_OF_THE_SMALL_POOL: u64 = 64 << 20;
/// 内存盘的扇区，也是物理块大小。
const SECTOR_BYTES: u64 = 512;
pub const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-runtime";
pub const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 新池新建文件那一份内容的字节数（同验证包 `build_pool` 的 3000 字节）。
const FIRST_FILE_BYTES: usize = 3000;
/// 另一次挂载里顺序写第一个文件的数据单元数（同验证包 `compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs`）。
pub const DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT: usize = 40;
/// 带洞的池：部分写这么多个数据单元（隔一个写一个），旧落点放掉之后成洞。
const DATA_UNITS_REWRITTEN_TO_LEAVE_HOLES: usize = 20;
/// 带洞的池：之后再建这么多次 inode，根环转过一圈（默认几何 3 × 8 = 24 槽），放掉的落点过了 defer 窗口。
const USER_CHANGES_TURNING_THE_ROOT_RING: usize = 30;

/// 用例摆的故障，一个池两块盘共用一份。
#[derive(Clone, Debug, Default)]
pub struct DeviceFaults {
    /// 两块盘读过几次。
    pub reads: Arc<AtomicU64>,
    /// 写报错之前先放过几个写。
    pub writes_let_through_before_the_faults: Arc<AtomicU64>,
    /// 接下来还有几个写报错（写之前报，盘上不变）。
    pub writes_left_to_fail: Arc<AtomicU64>,
    /// 屏障报错之前先放过几个屏障。
    pub barriers_let_through_before_the_faults: Arc<AtomicU64>,
    /// 接下来还有几个屏障报错（盘上不变、不记进录制流）。
    pub barriers_left_to_fail: Arc<AtomicU64>,
}

/// 改盘的一个动作，按先后记。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordedDeviceOperation {
    Write {
        device: DeviceIdentity,
        offset_in_bytes: u64,
        bytes: Vec<u8>,
        durability: WriteDurability,
    },
    WriteZeroes {
        device: DeviceIdentity,
        offset_in_bytes: u64,
        length_in_bytes: u64,
    },
    Discard {
        device: DeviceIdentity,
        offset_in_bytes: u64,
        length_in_bytes: u64,
    },
    Barrier {
        device: DeviceIdentity,
    },
}

/// 一个池两块盘共用的录制流。
pub type RecordedStream = Arc<Mutex<Vec<RecordedDeviceOperation>>>;

/// 内存里的一块稀疏盘：写过的扇区（扇区号 → 512 字节），连同它记进的录制流。
pub struct RecordedMemoryDevice {
    identity: DeviceIdentity,
    device_bytes: u64,
    sectors: BTreeMap<u64, Vec<u8>>,
    stream: RecordedStream,
    faults: DeviceFaults,
}

impl RecordedMemoryDevice {
    /// 这一个写该不该报错（用例摆的）。
    fn fault_of_this_write(
        &self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        let is_let_through = self
            .faults
            .writes_let_through_before_the_faults
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |left| {
                left.checked_sub(1)
            })
            .is_ok();
        let fails = !is_let_through
            && self
                .faults
                .writes_left_to_fail
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |left| {
                    left.checked_sub(1)
                })
                .is_ok();
        if fails {
            return Err(BlockDeviceError::OutOfRange {
                offset,
                length,
                device_size: 0,
            });
        }
        Ok(())
    }

    fn note(&self, operation: RecordedDeviceOperation) {
        self.stream
            .lock()
            .expect("录制流只在这一条线程上记，没在别处 panic 过")
            .push(operation);
    }

    fn check_request(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes: u64,
    ) -> Result<(), BlockDeviceError> {
        if !offset.0.is_multiple_of(SECTOR_BYTES) || !length_in_bytes.is_multiple_of(SECTOR_BYTES) {
            return Err(BlockDeviceError::Unaligned {
                offset,
                length: length_in_bytes,
                physical_block_size: u32::try_from(SECTOR_BYTES).expect("512"),
            });
        }
        if offset
            .0
            .checked_add(length_in_bytes)
            .is_none_or(|end| end > self.device_bytes)
        {
            return Err(BlockDeviceError::OutOfRange {
                offset,
                length: length_in_bytes,
                device_size: self.device_bytes,
            });
        }
        Ok(())
    }

    fn zero_fill(&mut self, offset: DeviceOffsetInBytes, length_in_bytes: u64) {
        let first_sector = offset.0 / SECTOR_BYTES;
        let past_the_last_sector = first_sector + length_in_bytes / SECTOR_BYTES;
        let written_in_the_range: Vec<u64> = self
            .sectors
            .range(first_sector..past_the_last_sector)
            .map(|(sector, _bytes)| *sector)
            .collect();
        for sector in written_in_the_range {
            self.sectors.remove(&sector);
        }
    }
}

impl BlockDevice for RecordedMemoryDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(buffer.len()).expect("读长装得进 u64");
        self.check_request(offset, length_in_bytes)?;
        self.faults.reads.fetch_add(1, Ordering::SeqCst);
        let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
        buffer.fill(0);
        let first_sector = offset.0 / SECTOR_BYTES;
        for (sector, bytes) in self
            .sectors
            .range(first_sector..first_sector + length_in_bytes / SECTOR_BYTES)
        {
            let start = usize::try_from(sector - first_sector).expect("扇区下标") * sector_bytes;
            buffer[start..start + sector_bytes].copy_from_slice(bytes);
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(bytes.len()).expect("写长装得进 u64");
        self.check_request(offset, length_in_bytes)?;
        self.fault_of_this_write(offset, length_in_bytes)?;
        let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
        let first_sector = offset.0 / SECTOR_BYTES;
        for (sector_in_the_write, sector_contents) in bytes.chunks(sector_bytes).enumerate() {
            self.sectors.insert(
                first_sector + u64::try_from(sector_in_the_write).expect("扇区下标"),
                sector_contents.to_vec(),
            );
        }
        self.note(RecordedDeviceOperation::Write {
            device: self.identity,
            offset_in_bytes: offset.0,
            bytes: bytes.to_vec(),
            durability,
        });
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, length)?;
        self.zero_fill(offset, length);
        self.note(RecordedDeviceOperation::WriteZeroes {
            device: self.identity,
            offset_in_bytes: offset.0,
            length_in_bytes: length,
        });
        Ok(())
    }

    /// 内存盘认 discard，把它当整段清零应用（崩溃点重放「持久 → 0」那一态），交回「交给了底层」。
    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.check_request(offset, length)?;
        self.zero_fill(offset, length);
        self.note(RecordedDeviceOperation::Discard {
            device: self.identity,
            offset_in_bytes: offset.0,
            length_in_bytes: length,
        });
        Ok(DiscardOutcome::Applied)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        let is_let_through = self
            .faults
            .barriers_let_through_before_the_faults
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |left| {
                left.checked_sub(1)
            })
            .is_ok();
        if !is_let_through
            && self
                .faults
                .barriers_left_to_fail
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |left| {
                    left.checked_sub(1)
                })
                .is_ok()
        {
            return Err(BlockDeviceError::OutOfRange {
                offset: DeviceOffsetInBytes(0),
                length: 0,
                device_size: 0,
            });
        }
        self.note(RecordedDeviceOperation::Barrier {
            device: self.identity,
        });
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        PhysicalBlockSizeInBytes(u32::try_from(SECTOR_BYTES).expect("512"))
    }

    fn size_in_bytes(&self) -> u64 {
        self.device_bytes
    }
}

/// 一个内存里的池：两块盘与它们共用的录制流。
pub struct PoolOnMemoryDevices {
    pub devices: Vec<(DeviceIdentity, RecordedMemoryDevice)>,
    pub stream: RecordedStream,
    pub faults: DeviceFaults,
}

impl PoolOnMemoryDevices {
    /// 同一份字节的另一个池，录制流是空的：「同一历史」的另一个副本。
    pub fn copied_with_an_empty_stream(&self) -> PoolOnMemoryDevices {
        let stream: RecordedStream = Arc::new(Mutex::new(Vec::new()));
        let faults = DeviceFaults::default();
        let devices = self
            .devices
            .iter()
            .map(|(identity, device)| {
                (
                    *identity,
                    RecordedMemoryDevice {
                        identity: *identity,
                        device_bytes: device.device_bytes,
                        sectors: device.sectors.clone(),
                        stream: Arc::clone(&stream),
                        faults: faults.clone(),
                    },
                )
            })
            .collect();
        PoolOnMemoryDevices {
            devices,
            stream,
            faults,
        }
    }

    /// 两块盘此刻写过的扇区：两个池逐字节比。
    pub fn written_sectors_of_every_device(&self) -> Vec<(DeviceIdentity, BTreeMap<u64, Vec<u8>>)> {
        self.devices
            .iter()
            .map(|(identity, device)| (*identity, device.sectors.clone()))
            .collect()
    }

    /// 录制流此刻记下的每个动作，按先后。
    pub fn recorded_operations(&self) -> Vec<RecordedDeviceOperation> {
        self.stream
            .lock()
            .expect("录制流只在这一条线程上记，没在别处 panic 过")
            .clone()
    }
}

/// 默认环长：从第一版的环起点到默认环长下单元区起点（环末尾的下一个槽）之间那么多字节。照核心层的两个起点算，不另抄一份格式常量。
fn default_journal_ring_bytes() -> u64 {
    UnitAreaStart::of_the_default_journal_ring()
        .slot()
        .to_device_offset()
        .0
        - JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION
            .to_device_offset()
            .0
}

/// mkfs 参数（同验证包 `common::parameters`，fsid 换成本包的）。
pub fn parameters() -> MakeFilesystemParameters {
    parameters_with_the_root_ring_slots(RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM)
}

/// 写得满的小池的 mkfs 参数：同 [`parameters`]，日志环换成 [`JOURNAL_RING_BYTES_OF_THE_SMALL_POOL`]。
pub fn parameters_of_the_small_pool() -> MakeFilesystemParameters {
    let mut small_pool_parameters = parameters();
    small_pool_parameters.geometry.journal_ring_bytes = JOURNAL_RING_BYTES_OF_THE_SMALL_POOL;
    small_pool_parameters
}

/// 同 [`parameters`]，每区根槽数照给（扫根环几何的用例）。
pub fn parameters_with_the_root_ring_slots(
    root_ring_slots_per_region: RootRingSlotsPerRegion,
) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: u32::try_from(SECTOR_BYTES).expect("512"),
            minimum_input_output_bytes: u32::try_from(SECTOR_BYTES).expect("512"),
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(
                u32::try_from(SECTOR_BYTES).expect("512"),
            ),
            journal_ring_bytes: default_journal_ring_bytes(),
            root_ring_slots_per_region,
        },
    }
}

fn contents_of_the_first_file() -> Vec<u8> {
    (0..FIRST_FILE_BYTES)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect()
}

/// 新池新建文件之后，另一次挂载里顺序写第一个文件 [`DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT`] 个数据单元、会话丢掉。交回那一刻的池
/// （录制流带着这一串的全部动作；拷池时换空流）。
pub fn pool_after_a_sequential_write_in_an_earlier_mount() -> PoolOnMemoryDevices {
    pool_after_a_sequential_write_in_an_earlier_mount_under(&parameters())
}

/// 写得满的小池（[`parameters_of_the_small_pool`]，每块盘 [`DEVICE_BYTES_OF_THE_SMALL_POOL`]）：同 [`pool_after_a_sequential_write_in_an_earlier_mount`]。
pub fn small_pool_after_a_sequential_write_in_an_earlier_mount() -> PoolOnMemoryDevices {
    pool_after_a_sequential_write_on_devices_of(
        &parameters_of_the_small_pool(),
        DEVICE_BYTES_OF_THE_SMALL_POOL,
    )
}

/// 同 [`pool_after_a_sequential_write_in_an_earlier_mount`]，mkfs 参数照给。
pub fn pool_after_a_sequential_write_in_an_earlier_mount_under(
    parameters: &MakeFilesystemParameters,
) -> PoolOnMemoryDevices {
    pool_after_a_sequential_write_on_devices_of(parameters, DEVICE_BYTES)
}

/// 同 [`pool_after_a_sequential_write_in_an_earlier_mount`]，mkfs 参数与每块盘的字节数照给。
fn pool_after_a_sequential_write_on_devices_of(
    parameters: &MakeFilesystemParameters,
    device_bytes: u64,
) -> PoolOnMemoryDevices {
    let stream: RecordedStream = Arc::new(Mutex::new(Vec::new()));
    let faults = DeviceFaults::default();
    let mut devices: Vec<(DeviceIdentity, RecordedMemoryDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    RecordedMemoryDevice {
                        identity,
                        device_bytes,
                        sectors: BTreeMap::new(),
                        stream: Arc::clone(&stream),
                        faults: faults.clone(),
                    },
                )
            })
            .collect();
    let genesis = make_filesystem(parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::with_unit_area_start(
            DeviceIdentity(0),
            device_bytes,
            genesis.unit_area_start,
        ),
        DeviceFreeMap::with_unit_area_start(
            DeviceIdentity(1),
            device_bytes,
            genesis.unit_area_start,
        ),
    ]);
    allocator.mark_format_time_units(
        genesis.instance_table_placement(),
        genesis.tree_table_genesis_placement(),
    );
    // mkfs 同一个进程里接着写的会话：装上 mkfs 刚写下的根环（同验证包 `build_pool`）。
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        parameters, &genesis,
    ));
    {
        let mut writer = PoolWriter::new(parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed_up = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut writer,
            &mut allocator,
            warmed_up.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &contents_of_the_first_file(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed_up.last_record_bytes,
        )
        .expect("新池新建文件");
    }
    let mounted = mount_writable(parameters, &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let content: Vec<u8> = (0..DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT
        * data_unit_payload_capacity())
        .map(|index| u8::try_from(index % 241).expect("小于 256"))
        .collect();
    session
        .publish_user_change(
            &mut devices,
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            },
        )
        .expect("顺序写第一个文件");
    drop(session);
    PoolOnMemoryDevices {
        devices,
        stream,
        faults,
    }
}

/// 带洞的池：同 [`pool_after_a_sequential_write_in_an_earlier_mount`]，接着隔一个数据单元部分写一个（旧落点放掉）、再建几次 inode 让根环转过一圈（放掉的落点过了 defer 窗口），
/// 会话丢掉。数据段里有了洞：产品路径选段之后「R 外的空洞够不够」那一判过得了（只顺序写的池上段都是满的，判不过）。
pub fn pool_with_holes_in_the_data_segments() -> PoolOnMemoryDevices {
    pool_with_holes_in_the_data_segments_under(&parameters())
}

/// 同 [`pool_with_holes_in_the_data_segments`]，mkfs 参数照给。
pub fn pool_with_holes_in_the_data_segments_under(
    parameters: &MakeFilesystemParameters,
) -> PoolOnMemoryDevices {
    let mut pool = pool_after_a_sequential_write_in_an_earlier_mount_under(parameters);
    let mounted = mount_writable(parameters, &mut pool.devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let one_byte = [0x5A_u8];
    for rewritten in 0..DATA_UNITS_REWRITTEN_TO_LEAVE_HOLES {
        let data_unit_index = u64::try_from(2 * rewritten).expect("单元下标");
        let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
        session
            .publish_user_change(
                &mut pool.devices,
                UserChange::WriteAtOffset {
                    inode: InodeNumber(FIRST_INODE_NUMBER),
                    offset: FileOffsetInBytes(data_unit_index * payload),
                    bytes: PartialWriteBytes::new(&one_byte).expect("一个字节不是空的"),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
                },
            )
            .expect("部分写一个数据单元");
    }
    for _turn in 0..USER_CHANGES_TURNING_THE_ROOT_RING {
        session
            .publish_user_change(
                &mut pool.devices,
                UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
                },
            )
            .expect("建一个 inode");
    }
    drop(session);
    pool
}
