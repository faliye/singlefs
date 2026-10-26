//! 本包端到端用例共用的搭建：两块内存里的稀疏盘上 mkfs → 取号 → 暖机 → 新池新建文件（同 `crates/singlefs-harness/tests/common/mod.rs` 的
//! `build_pool` 那一串，参数也照它的 `parameters`），另一次挂载里顺序写第一个文件 [`DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT`] 个数据单元、
//! 会话丢掉（「进程退出」）。这份字节拷成几个池，各自重挂、装上整理只供测试的开关，推一趟整理到停，用例逐项比。
//! 本包不依赖 harness 档（门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格第 ⑧ 条判产品 GPU 包与 checker 档两边闭包的交集，
//! harness 档在 checker 档的闭包里），所以盘与搭建在这里另写一份，只用核心层。
//! 盘按 512 字节扇区存（没写过的读回 0），改盘的每个动作（写、写零、discard、屏障）按先后记进这个池两块盘共用的录制流；重挂之前换一条空的流，
//! 用例比的是重挂之后的那一段。
#![allow(dead_code, reason = "几份测试文件各自 mod 进来，只用到其中一部分")]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InodeNumber};
use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator, UnitAreaStart};
use singlefs_core::background_compute::{BackgroundComputeSettings, BackgroundGpuComputeHandle};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::compaction_plan::{
    CompactionBatchSize, CompactionBatchUnits, CompactionRunForcing, CompactionSwitches,
};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{mount_writable, mount_writable_with_background_compute_settings};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::{
    SystemImmutableSizes, JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;

/// 每块盘的字节数（同 harness 档 `build_pool` 的镜像）。
pub const DEVICE_BYTES: u64 = 4 << 30;
/// 内存盘的扇区，也是物理块大小。
const SECTOR_BYTES: u64 = 512;
pub const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-bggpu-1";
pub const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 新池新建文件那一份内容的字节数（同 harness 档 `build_pool` 的 3000 字节）。
const FIRST_FILE_BYTES: usize = 3000;
/// 另一次挂载里顺序写第一个文件的数据单元数（同 harness 档 `compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs`）。
pub const DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT: usize = 40;
/// 整理一批至多搬几个（只供测试的开关；产品路径批大小条款没写）。
pub const UNITS_PER_BATCH: usize = 2;

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
    sectors: BTreeMap<u64, Vec<u8>>,
    stream: RecordedStream,
}

impl RecordedMemoryDevice {
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
            .is_none_or(|end| end > DEVICE_BYTES)
        {
            return Err(BlockDeviceError::OutOfRange {
                offset,
                length: length_in_bytes,
                device_size: DEVICE_BYTES,
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
        self.check_request(offset, u64::try_from(bytes.len()).expect("写长装得进 u64"))?;
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
        self.note(RecordedDeviceOperation::Barrier {
            device: self.identity,
        });
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        PhysicalBlockSizeInBytes(u32::try_from(SECTOR_BYTES).expect("512"))
    }

    fn size_in_bytes(&self) -> u64 {
        DEVICE_BYTES
    }
}

/// 一个内存里的池：两块盘与它们共用的录制流。
pub struct PoolOnMemoryDevices {
    pub devices: Vec<(DeviceIdentity, RecordedMemoryDevice)>,
    pub stream: RecordedStream,
}

impl PoolOnMemoryDevices {
    /// 同一份字节的另一个池，录制流是空的：「同一历史」的另一个副本。
    pub fn copied_with_an_empty_stream(&self) -> PoolOnMemoryDevices {
        let stream: RecordedStream = Arc::new(Mutex::new(Vec::new()));
        let devices = self
            .devices
            .iter()
            .map(|(identity, device)| {
                (
                    *identity,
                    RecordedMemoryDevice {
                        identity: *identity,
                        sectors: device.sectors.clone(),
                        stream: Arc::clone(&stream),
                    },
                )
            })
            .collect();
        PoolOnMemoryDevices { devices, stream }
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

    /// 重挂、开会话，装上整理只供测试的开关：一批至多 [`UNITS_PER_BATCH`] 个、起跑强制开着、greedy 打分选段。
    /// 后台计算的设置连同可选的 GPU 实现交挂载；挂载拒了就 panic（用例给的都是全合法的那一组）。
    pub fn remounted_with_compaction_forced_to_start(
        &mut self,
        background_compute_settings: BackgroundComputeSettings,
        gpu_compute: Option<BackgroundGpuComputeHandle>,
    ) -> MountedSession {
        let mounted = mount_writable_with_background_compute_settings(
            &parameters(),
            &mut self.devices,
            background_compute_settings,
            gpu_compute,
        )
        .expect("全合法的设置连同实现：挂载照常做成");
        let (_, mut session) = MountedSession::of_the_mount(mounted);
        let CompactionSwitches {
            batch_size: _,
            forced_region,
            run_forcing: _,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor,
            relocation_publish_order,
        } = CompactionSwitches::PRODUCT;
        session
            .allocator
            .set_compaction_switches(CompactionSwitches {
                batch_size: CompactionBatchSize::ForcedForTests(
                    CompactionBatchUnits::new(UNITS_PER_BATCH).expect("至少 1"),
                ),
                forced_region,
                run_forcing: CompactionRunForcing::ForcedToStartForTests,
                region_exclusion,
                destination_policy,
                release_checksum_mismatch_after_issue,
                empty_cluster_segments_floor,
                relocation_publish_order,
            });
        session
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

/// mkfs 参数（同 harness 档 `common::parameters`，fsid 换成本包的）。
pub fn parameters() -> MakeFilesystemParameters {
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
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
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
    let parameters = parameters();
    let stream: RecordedStream = Arc::new(Mutex::new(Vec::new()));
    let mut devices: Vec<(DeviceIdentity, RecordedMemoryDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    RecordedMemoryDevice {
                        identity,
                        sectors: BTreeMap::new(),
                        stream: Arc::clone(&stream),
                    },
                )
            })
            .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), DEVICE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), DEVICE_BYTES),
    ]);
    allocator.mark_format_time_units(
        genesis.instance_table_placement(),
        genesis.tree_table_genesis_placement(),
    );
    // mkfs 同一个进程里接着写的会话：装上 mkfs 刚写下的根环（同 harness 档 `build_pool`）。
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &parameters,
        &genesis,
    ));
    {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
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
    let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
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
    PoolOnMemoryDevices { devices, stream }
}
