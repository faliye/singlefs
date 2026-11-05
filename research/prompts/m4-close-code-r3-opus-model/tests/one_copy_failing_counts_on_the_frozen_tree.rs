//! 代码轮 m4-close-code-r3 攻方腿 T2 的模型（只在冻结副本的拷贝上跑，不进仓）：只坏一份的计数在三形历史上的读数。
//! - 可写挂载建上一版：同一张实例表第 0 片的同一份坏副本，经数「只坏一份」的那一层读两遍（`recovery` 的
//!   `bootstrap_units_of_the_root_read_and_judged` 一遍、`mount::rebuild_previous_version` 里 `instance_table_chain_of_root` 又一遍），
//!   一次挂载计两次；树表坏一份对照计一次。
//! - 位置提示过期（整理搬走了单元，盘上 extent 叶里的提示仍指旧槽）、旧槽一块盘上那一份被盖掉（一次没发布成的写、只落了一块盘），
//!   映射位置上两份都好：可写挂载建上一版与只读挂载 `read_at` 都把它计成只坏一份；对照不盖那一份计 0。
//! - 只读挂载一次 `read_at` 先读到只坏一份的单元、后面一个单元两份都坏而整次读报错：这一次读的观测整份不进打开之后的累计，
//!   只坏一份的那一次丢了；再单读那个单元才计上。
//! 每条用例断言的是今天的行为：绿 = 攻方说的那一形在冻结树上成立。观测行以 `attack-r3:` 开头。
mod common;

use common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_core::address::{
    DataUnitIndexInFile, DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration,
};
use singlefs_core::allocator::{Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::compaction_plan::PlannedRelocation;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::pointer::{DataPointer, LocationEntry};
use singlefs_core::transaction::{
    acquire_instance, central_mapping_locations_in_the_version, publish_first_file,
    publish_relocation_batch, publish_sequential_write, warm_up, FirstFile, MappingLookup,
    PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

const FIRST_FILE: InodeNumber = InodeNumber(1);
/// 弄坏的是第一个文件第 1 个数据单元（三个单元里中间那一个）。
const DAMAGED_UNIT: DataUnitIndexInFile = DataUnitIndexInFile(1);
/// 弄坏那一份时翻的是单元里第几个字节（树表一个槽 16384 字节，也落在里面）。
const FLIPPED_BYTE_IN_THE_UNIT: usize = 4000;

fn payload_capacity() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634")
}

fn two_slots() -> usize {
    usize::try_from(2 * SLOT_BYTES).expect("两槽")
}

fn one_slot() -> usize {
    usize::try_from(SLOT_BYTES).expect("一槽")
}

/// 三个单元的内容，最后一个不满。
fn content_of_three_units() -> Vec<u8> {
    let length = usize::try_from(3 * payload_capacity() - 1000).expect("三个单元以内");
    (0..length)
        .map(|index| u8::try_from((index * 37 + 11) % 253).expect("小于 256"))
        .collect()
}

struct PoolOfThreeUnits {
    devices: Devices,
    allocator: PoolAllocator,
    version: TransactionOutput,
    instance: InstanceGeneration,
}

/// 两块 4 GiB 内存盘上 mkfs → 取号 → 暖机 → 新池新建文件 → 第一个文件整份顺序写成三个单元。
fn pool_of_three_units() -> PoolOfThreeUnits {
    let parameters = parameters();
    let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let content = file_content();
    let (version, instance) = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first = publish_first_file(
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
        .expect("新池新建文件");
        let three_units = publish_sequential_write(
            &mut writer,
            &mut allocator,
            &first,
            FirstFile {
                content: &content_of_three_units(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            },
            instance,
        )
        .expect("顺序写成三个单元");
        (three_units, instance)
    };
    assert_eq!(version.data_pointers.len(), 3, "前提：第一个文件三个单元");
    PoolOfThreeUnits {
        devices,
        allocator,
        version,
        instance,
    }
}

fn memory_pool_of(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

/// 把 `location` 那块盘上那一份（从它的槽起 `length` 字节）改一个字节：那一份整单元 CRC-32C 与位置条目对不上，另一份不动。
fn damage_the_copy_at(devices: &mut Devices, location: &LocationEntry, length: usize) {
    let device = &mut devices
        .iter_mut()
        .find(|(identity, _)| *identity == location.device)
        .expect("池里有这块盘")
        .1;
    let mut unit = vec![0u8; length];
    device
        .read_at(location.slot.to_device_offset(), &mut unit)
        .expect("读得出那一份");
    unit[FLIPPED_BYTE_IN_THE_UNIT] ^= 0xFF;
    device
        .write_at(location.slot.to_device_offset(), &unit, WriteDurability::Plain)
        .expect("写回改过的那一份");
    assert_ne!(
        crc32_castagnoli(&unit),
        location.unit_checksum,
        "前提：改过的那一份与位置条目对不上"
    );
}

fn copy_at_the_location_self_verifies(devices: &Devices, location: &LocationEntry, length: usize) -> bool {
    let mut unit = vec![0u8; length];
    devices
        .iter()
        .find(|(identity, _)| *identity == location.device)
        .expect("池里有这块盘")
        .1
        .read_at(location.slot.to_device_offset(), &mut unit)
        .expect("读得出那一份");
    crc32_castagnoli(&unit) == location.unit_checksum
}

fn pointer_of_the_damaged_unit(version: &TransactionOutput) -> DataPointer {
    version.data_pointers[usize::try_from(DAMAGED_UNIT.0).expect("单元序号")]
}

fn mount_count_while_mounting(pool: &mut PoolOfThreeUnits) -> u64 {
    let mounted = mount_writable(&parameters(), &mut pool.devices).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    session.unit_reads_with_one_copy_failing_while_mounting()
}

/// 只读挂载、打开第一个文件、从 0 读到 size：交回读出的字节与打开之后累计的只坏一份次数。
fn read_only_whole_file_and_its_count(devices: &Devices) -> (Vec<u8>, u64) {
    let image = memory_pool_of(devices);
    let read_only = mount_read_only(&image).expect("只读挂载");
    let file = read_only.mounted.open_file(&image, FIRST_FILE).expect("打开文件");
    let size = file.inode_record().size;
    let output = file
        .read_at(&image, FileOffsetInBytes(0), size)
        .expect("从 0 读到 size");
    (
        output.bytes,
        read_only
            .mounted
            .observation_since_open()
            .data_unit_reads_with_one_copy_failing,
    )
}

/// 实例表第 0 片（根记录直接持有的那一片，32 KiB）头一条位置条目那一份坏、另一份好：可写挂载照常做成，挂载时读到的只坏一份计两次。
#[test]
fn a_writable_mount_counts_one_damaged_copy_of_the_instance_table_twice() {
    let mut pool = pool_of_three_units();
    let location = pool.version.root.instance_table.locations[0];
    damage_the_copy_at(&mut pool.devices, &location, two_slots());
    let counted = mount_count_while_mounting(&mut pool);
    println!("attack-r3: damaged=instance_table_first_page_one_copy unit_reads_with_one_copy_failing_while_mounting={counted}");
    assert!(counted >= 2, "同一份坏副本一次挂载里计了不止一次：{counted}");
}

/// 对照：树表那一个单元（16 KiB）头一条位置条目那一份坏、另一份好：一次挂载计一次。
#[test]
fn a_writable_mount_counts_one_damaged_copy_of_the_tree_table_once() {
    let mut pool = pool_of_three_units();
    let location = pool.version.root.tree_table.locations[0];
    damage_the_copy_at(&mut pool.devices, &location, one_slot());
    let counted = mount_count_while_mounting(&mut pool);
    println!("attack-r3: damaged=tree_table_one_copy unit_reads_with_one_copy_failing_while_mounting={counted}");
    assert_eq!(counted, 1, "树表只经那一层读一遍");
}

/// 整理那一批（`publish_relocation_batch`）把第 1 个数据单元搬走：交回搬之后那一版、旧落点（盘上 extent 叶里的提示还指着它）与映射位置。
fn relocate_the_damaged_unit(pool: &mut PoolOfThreeUnits) -> ([LocationEntry; 2], [LocationEntry; 2]) {
    let unit = TransactionUnit::Data(DAMAGED_UNIT);
    let mapping_key = pool
        .version
        .mapped_units
        .iter()
        .find(|(role, _)| *role == unit)
        .map(|(_, key)| key.clone())
        .expect("第 1 个数据单元进映射");
    let MappingLookup::Found(source) =
        central_mapping_locations_in_the_version(&pool.version, &mapping_key)
    else {
        panic!("第 1 个数据单元在映射里查得到");
    };
    let planned = PlannedRelocation {
        mapping_key: mapping_key.clone(),
        source: Placement {
            slot: source[0].slot,
            span: unit.span_slots(),
        },
    };
    let parameters = parameters();
    let relocated = {
        let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
        publish_relocation_batch(
            &mut writer,
            &mut pool.allocator,
            &pool.version,
            std::slice::from_ref(&planned),
            pool.instance,
        )
        .expect("搬一批")
    };
    let MappingLookup::Found(destination) =
        central_mapping_locations_in_the_version(&relocated, &mapping_key)
    else {
        panic!("搬之后映射里查得到");
    };
    assert_ne!(destination[0].slot, source[0].slot, "前提：搬到了别的槽");
    assert_eq!(
        pointer_of_the_damaged_unit(&pool.version).locations,
        source,
        "前提：搬之前指针里的位置就是映射位置（盘上 extent 叶不随这一批重写，提示仍指它）"
    );
    pool.version = relocated;
    (source, destination)
}

/// 对照：搬走之后不盖旧槽那一份：只读挂载与可写挂载都计 0。
#[test]
fn after_a_relocation_with_the_old_slot_untouched_nothing_is_counted() {
    let mut pool = pool_of_three_units();
    let (_source, destination) = relocate_the_damaged_unit(&mut pool);
    let (bytes, read_only_count) = read_only_whole_file_and_its_count(&pool.devices);
    assert_eq!(bytes, content_of_three_units(), "读出原内容");
    let mount_count = mount_count_while_mounting(&mut pool);
    println!("attack-r3: relocated old_slot=untouched read_only_count={read_only_count} mount_count={mount_count} destination={destination:?}");
    assert_eq!(read_only_count, 0, "没有坏的一份");
    assert_eq!(mount_count, 0, "没有坏的一份");
}

/// 搬走之后旧槽在头一条位置条目那块盘上的那一份被盖掉（一次没发布成、只落了一块盘的写），映射位置上两份都好：
/// 只读挂载 `read_at` 与可写挂载建上一版都按过期的提示读、头一份对不上、第二份（旧字节）对上，各计一次只坏一份。
#[test]
fn a_stale_hint_whose_old_slot_lost_one_copy_is_counted_although_both_mapped_copies_are_intact() {
    let mut pool = pool_of_three_units();
    let (source, destination) = relocate_the_damaged_unit(&mut pool);
    damage_the_copy_at(&mut pool.devices, &source[0], two_slots());
    assert!(
        destination
            .iter()
            .all(|location| copy_at_the_location_self_verifies(&pool.devices, location, two_slots())),
        "前提：映射位置上两份都好"
    );
    let (bytes, read_only_count) = read_only_whole_file_and_its_count(&pool.devices);
    assert_eq!(bytes, content_of_three_units(), "读出原内容");
    let mount_count = mount_count_while_mounting(&mut pool);
    println!("attack-r3: relocated old_slot=one_copy_overwritten read_only_count={read_only_count} mount_count={mount_count} source={source:?} destination={destination:?}");
    assert_eq!(read_only_count, 1, "只读挂载按过期提示读，计成只坏一份");
    assert_eq!(mount_count, 1, "可写挂载建上一版按过期提示读，计成只坏一份");
}

/// 只读挂载一次 `read_at` 从 0 读到 size：第 1 个单元头一份坏（只坏一份），第 2 个单元两份都坏（整次读报错）。
/// 今天这一次读的观测整份不进打开之后的累计：只坏一份那一次丢了；再单读第 1 个单元才计上 1。
#[test]
fn a_read_failing_on_a_later_unit_drops_the_one_copy_failing_count_of_an_earlier_unit() {
    let mut pool = pool_of_three_units();
    // 挂载与打开在没弄坏的盘上做（挂载重放 journal 时核点名单元，第 2 个单元两份都坏会让那条记录不施加、落到更早的一版），
    // 之后盘上坏掉（挂着的时候设备出错）：`read_at` 每次读的是交进来的读者。
    let image_at_the_mount = memory_pool_of(&pool.devices);
    let read_only = mount_read_only(&image_at_the_mount).expect("只读挂载");
    let file = read_only
        .mounted
        .open_file(&image_at_the_mount, FIRST_FILE)
        .expect("打开文件");
    let damaged = pointer_of_the_damaged_unit(&pool.version);
    damage_the_copy_at(&mut pool.devices, &damaged.locations[0], two_slots());
    let last = pool.version.data_pointers[2];
    damage_the_copy_at(&mut pool.devices, &last.locations[0], two_slots());
    damage_the_copy_at(&mut pool.devices, &last.locations[1], two_slots());
    let image = memory_pool_of(&pool.devices);
    let size = file.inode_record().size;
    let whole = file.read_at(&image, FileOffsetInBytes(0), size);
    let after_the_failed_read = read_only.mounted.observation_since_open();
    let unit_one = file.read_at(
        &image,
        FileOffsetInBytes(payload_capacity()),
        payload_capacity(),
    );
    let after_reading_unit_one = read_only.mounted.observation_since_open();
    println!(
        "attack-r3: failed_whole_read_is_err={} count_after_the_failed_read={} device_reads_after_the_failed_read={} unit_one_read_is_ok={} count_after_reading_unit_one={}",
        whole.is_err(),
        after_the_failed_read.data_unit_reads_with_one_copy_failing,
        after_the_failed_read.device_reads_issued,
        unit_one.is_ok(),
        after_reading_unit_one.data_unit_reads_with_one_copy_failing
    );
    assert!(whole.is_err(), "第 2 个单元两份都坏：整次读报错");
    assert_eq!(
        after_the_failed_read.data_unit_reads_with_one_copy_failing, 0,
        "报错那一次读里第 1 个单元的只坏一份没进累计"
    );
    assert!(unit_one.is_ok(), "单读第 1 个单元读得出");
    assert_eq!(
        after_reading_unit_one.data_unit_reads_with_one_copy_failing, 1,
        "单读那一次计上"
    );
}
