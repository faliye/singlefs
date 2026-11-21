//! 起点池（照第三轮攻方腿探针 `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-opus-model/probe/src/pool.rs` 拷来，加只读挂载读整个文件那一个函数）：
//! 两块内存稀疏盘（验证包 `memory_pool::SparseBlockDevice`）上 mkfs → 取号 → 暖机 → 新池新建文件 → 另一次挂载顺序写第一个文件 40 个数据单元
//! → 隔一个单元部分写一个字节（旧落点放掉）→ 建 30 次 inode（根环转过一圈，放掉的落点过了 defer 窗口），会话丢掉。
//! 数据段里有了洞，整理选得出段、签得出意图。起点只建一次，之后每段历史从它拷（`copied`）。
use std::collections::BTreeMap;

use singlefs_core::address::{DeviceIdentity, FileOffsetInBytes, InodeNumber};
use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator, UnitAreaStart};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
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
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};

pub const DEVICE_BYTES: u64 = 4 << 30;
const SECTOR_BYTES: u32 = 512;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-gpuwait";
pub const WRITE_TIME_SECONDS: u64 = 1_788_000_000;
const DATA_UNITS_OF_THE_FILE: usize = 40;
const DATA_UNITS_REWRITTEN_TO_LEAVE_HOLES: usize = 20;
const USER_CHANGES_TURNING_THE_ROOT_RING: usize = 30;

pub type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

pub fn parameters() -> MakeFilesystemParameters {
    let ring_bytes = UnitAreaStart::of_the_default_journal_ring()
        .slot()
        .to_device_offset()
        .0
        - JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION
            .to_device_offset()
            .0;
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: SECTOR_BYTES,
            minimum_input_output_bytes: SECTOR_BYTES,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(SECTOR_BYTES),
            journal_ring_bytes: ring_bytes,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn empty_device() -> SparseBlockDevice {
    SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(SECTOR_BYTES))
}

/// 同一份字节的另一个池：每段历史各拷一份，互不影响。
pub fn copied(devices: &Devices) -> Devices {
    devices
        .iter()
        .map(|(identity, device)| {
            let mut copy = empty_device();
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect()
}

/// 这几块盘此刻的字节当一张池镜像（池级 checker 读它）。
pub fn memory_pool_of(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect::<BTreeMap<_, _>>(),
        device_size_in_bytes: DEVICE_BYTES,
    }
}

/// 第一个文件里 `unit` 号数据单元的第一个字节的偏移。
pub fn offset_of_the_unit(unit: usize) -> FileOffsetInBytes {
    FileOffsetInBytes(u64::try_from(unit * data_unit_payload_capacity()).expect("偏移"))
}

/// 一次部分写：把 `byte` 写到第一个文件 `unit` 号数据单元的第一个字节（那个单元换一个新落点，旧落点放掉）。
pub fn write_one_byte(
    session: &mut MountedSession,
    devices: &mut Devices,
    unit: usize,
    byte: u8,
) -> Result<(), String> {
    let bytes = [byte];
    session
        .publish_user_change(
            devices,
            UserChange::WriteAtOffset {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                offset: offset_of_the_unit(unit),
                bytes: PartialWriteBytes::new(&bytes).expect("一个字节不是空的"),
                write_time_seconds: WRITE_TIME_SECONDS + 9,
            },
        )
        .map(|_published| ())
        .map_err(|refused| format!("{refused:?}").chars().take(160).collect())
}

/// 只读挂载（择根、扫环、施加）之后读 `inode` 整个文件；挂不上、打不开、读不出都交回说明（两次读的说明相同也算读回一致）。
pub fn whole_file_after_a_read_only_mount(devices: &Devices, inode: u64) -> Result<Vec<u8>, String> {
    let mounted = mount_read_only(devices).map_err(|failure| {
        format!("mount:{failure:?}").chars().take(120).collect::<String>()
    })?;
    let file = mounted
        .mounted
        .open_file(devices, InodeNumber(inode))
        .map_err(|failure| format!("open:{failure:?}").chars().take(120).collect::<String>())?;
    let size = file.inode_record().size;
    let read = file
        .read_at(devices, FileOffsetInBytes(0), size)
        .map_err(|failure| format!("read:{failure:?}").chars().take(120).collect::<String>())?;
    Ok(read.bytes)
}

/// 建起点池（文件头那一串）。
pub fn start_pool() -> Devices {
    let parameters = parameters();
    let mut devices: Devices = vec![
        (DeviceIdentity(0), empty_device()),
        (DeviceIdentity(1), empty_device()),
    ];
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                DeviceFreeMap::with_unit_area_start(identity, DEVICE_BYTES, genesis.unit_area_start)
            })
            .collect(),
    );
    allocator.mark_format_time_units(
        genesis.instance_table_placement(),
        genesis.tree_table_genesis_placement(),
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &parameters,
        &genesis,
    ));
    {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed_up = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first_content: Vec<u8> = (0..3000_usize)
            .map(|index| u8::try_from(index % 251).expect("小于 256"))
            .collect();
        publish_first_file(
            &mut writer,
            &mut allocator,
            warmed_up.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &first_content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            instance,
            &warmed_up.last_record_bytes,
        )
        .expect("新池新建文件");
    }
    let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let content: Vec<u8> = (0..DATA_UNITS_OF_THE_FILE * data_unit_payload_capacity())
        .map(|index| u8::try_from(index % 241).expect("小于 256"))
        .collect();
    session
        .publish_user_change(
            &mut devices,
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                content: &content,
                write_time_seconds: WRITE_TIME_SECONDS + 1,
            },
        )
        .expect("顺序写第一个文件");
    for rewritten in 0..DATA_UNITS_REWRITTEN_TO_LEAVE_HOLES {
        write_one_byte(&mut session, &mut devices, 2 * rewritten, 0x5A).expect("部分写一个数据单元");
    }
    for _turn in 0..USER_CHANGES_TURNING_THE_ROOT_RING {
        session
            .publish_user_change(
                &mut devices,
                UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: WRITE_TIME_SECONDS + 3,
                },
            )
            .expect("建一个 inode");
    }
    drop(session);
    devices
}
