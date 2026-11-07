//! 只坏一份的读计数（D4（校验和位置） 已定项 4「读到只坏一份时计数，并经已有的搬迁路径把这个单元整个搬到新槽、两份新写，旧落点照释放前核隔离」；
//! C946（只坏一份的读没有计数也没有搬迁修复））：只把第一个文件第 1 个数据单元在它头一条位置条目那块盘上的那一份弄坏（盘上字节改一个），
//! 部分写读回、截断尾单元读回、挂载态 `read_at`、冷恢复读各跑一次，读出的都是原内容、计数各加一；不弄坏时计数为零。
//! 部分写与截断读回的那个单元这一次发布就整个重写：落在新槽、两份新写、旧落点两份都照释放之前读盘核隔离。
//! 计数只在内存里：写入口一份（`PoolWriter::old_data_unit_reads_back_with_one_copy_failing`），挂载态一份
//! （`ReadPathObservation::data_unit_reads_with_one_copy_failing`，一次读的与打开之后累计的），冷恢复读不挂载、只报给读者
//! （`recovery::PoolReaderCountingUnitReadsWithOneCopyFailing` 套着数）。挂载态 `read_at` 是只读挂载，没有写者，不搬。
//! 盘是两块内存稀疏盘，池在进程里 mkfs、取号、暖机、第一个文件、顺序写成三个单元造出来，不建镜像文件。
//!
//! 可写挂载的会话按一次挂载累计写入口上的数（`mounted_session::MountedSession::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`，
//! 各次发布之和，新挂载从 0 起）。有活快照时读回的旧单元换下来进 deadlist、不释放（判决 m4-partial-write-r1 R7），坏的那一份留在旧落点上：
//! 会话照已有的搬迁路径另发一批，把快照里的这个单元搬到新槽、两份新写，旧落点照释放之前读盘核隔离。
//! 只读挂载只计数、不修：盘上逐字节不变。
//! 另发的那一批被空间拒时推一串抬 F、回收之后再发同一批，推满仍放不下交回那一种结局、这一批一个字节都不写，
//! 用户改动在每一种结局下都已发成；被拒那一支靠只供测试的开关（`compaction_plan::RelocationKeptForTheSnapshotsRefusedForSpace`）强制进入。
//! 被拒不是空间不够（不推抬 F、不再试）与为空间推的抬 F 自己报错那两支各靠一个只供测试的开关强制进入
//! （`compaction_plan::RelocationKeptForTheSnapshotsRefusedNotForSpace`、`compaction_plan::FloorRaiseForTheRelocationKeptForTheSnapshotsFails`；
//! `.claude/rules/fs-design.md`「五条硬要求」第 2 条）；被拒不是空间不够的块设备错那一形另由故障注入设备在那一批的头一次写上报错造出来，照发布路径冻结。
//! 截断到弄坏的那个单元中间与部分写同一套断言。可写挂载建上一版时读到的只坏一份另记一个数
//! （`mounted_session::MountedSession::unit_reads_with_one_copy_failing_while_mounting`，与会话里读—改—写那一份按读的地方分开报）。
//!
//! 判决 `research/prompts/m4-close-code-r3-main-verification.md` 第三节第 3–7 条（攻方模型用例
//! `research/prompts/m4-close-code-r3-opus-model/tests/one_copy_failing_counts_on_the_frozen_tree.rs` 挪进来、断言翻成修完之后的期望）：
//! 同一份坏副本一次挂载只计一次（实例表第 0 片经两处读两遍那一形计 1）；按这个单元此刻的权威落点判（整理搬走之后盘上的提示还指旧槽、
//! 旧槽一块盘上那一份被盖掉、映射里两份都好：只读挂载、可写挂载建上一版、部分写读回都计 0）；`read_at` 中途报错时报错之前读到的只坏一份
//! 照样进打开之后的累计（与写入口「被拒也算」同一种口径）；可写挂载建上一版时读到只坏一份的单元，挂载记下它、下一次用户改动发成之后照快照下
//! 另发那批同一条路径搬到新槽、两份新写、旧落点两块盘都隔离。
mod common;
mod common_unwitnessed_publish;

use std::num::NonZeroU64;

use common::{file_content, geometry, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, record_one_publish_after_overwrites, PublishRecordedAsR,
    RangesReadingBackAllZero,
};
use singlefs_core::address::{
    DataUnitIndexInFile, DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration,
    SlotNumber, TreeIdentifier,
};
use singlefs_core::admission::admission_reading_before_a_publish;
use singlefs_core::allocator::{Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::compaction_plan::{
    FloorRaiseForTheRelocationKeptForTheSnapshotsFails, PlannedRelocation,
    RelocationKeptForTheSnapshotsRefusedForSpace, RelocationKeptForTheSnapshotsRefusedNotForSpace,
};
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::mounted_read::{mount_read_only, open_pool_for_read};
use singlefs_core::mounted_session::{
    FloorRaisesPushedForTheRelocation, MountedSession,
    RelocationOfTheOldDataUnitsKeptForTheSnapshots, UserChange, UserChangePublished,
};
use singlefs_core::pointer::{DataPointer, LocationEntry};
use singlefs_core::records::TREE_KIND_SNAPSHOT_INODE_ROOT;
use singlefs_core::recovery::{
    read_the_first_file_of_a_snapshot, walk_to_file, PoolReaderCountingUnitReadsWithOneCopyFailing,
    SnapshotFileReading,
};
use singlefs_core::transaction::{
    acquire_instance, central_mapping_locations_in_the_version, publish_first_file,
    publish_partial_write, publish_relocation_batch, publish_sequential_write, publish_size_change,
    read_back_the_old_data_unit_telling_whether_a_copy_failed, warm_up, CopiesOfAUnitRead,
    FirstFile, MappingLookup, PartialWriteBytes, PoolWriter, PublishError, RelocationBatchRefused,
    TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::{data_unit_payload_capacity, parse_data_unit};
use singlefs_format::SLOT_BYTES;
use singlefs_verification::fault_injection::{
    FaultInjectingBlockDevice, FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

const FIRST_FILE: InodeNumber = InodeNumber(1);
/// 弄坏的是第一个文件第 1 个数据单元（三个单元里中间那一个）。
const DAMAGED_UNIT: DataUnitIndexInFile = DataUnitIndexInFile(1);
/// 弄坏那一份时翻的是单元里第几个字节（头之后、净荷里）。
const FLIPPED_BYTE_IN_THE_UNIT: usize = 4000;
/// 部分写写在第 1 个单元起点之后多少字节、写多长。
const PARTIAL_WRITE_OFFSET_IN_THE_UNIT: u64 = 100;
const PARTIAL_WRITE_BYTES: usize = 64;
/// 截断截到第 1 个单元起点之后多少字节（截在单元中间：尾单元读回、重写）。
const TRUNCATION_OFFSET_IN_THE_UNIT: u64 = 500;

fn payload_capacity() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634")
}

fn data_unit_bytes() -> usize {
    usize::try_from(2 * SLOT_BYTES).expect("一个数据单元两槽")
}

/// 三个单元的内容，最后一个不满。
fn content_of_three_units() -> Vec<u8> {
    let length = usize::try_from(3 * payload_capacity() - 1000).expect("三个单元以内");
    (0..length)
        .map(|index| u8::try_from((index * 37 + 11) % 253).expect("小于 256"))
        .collect()
}

/// 一个池：两块盘、分配器、现行那一版（第一个文件三个单元）。
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

/// `version` 里第一个文件第 [`DAMAGED_UNIT`] 个数据单元的指针。
fn pointer_of_the_damaged_unit(version: &TransactionOutput) -> DataPointer {
    version.data_pointers[usize::try_from(DAMAGED_UNIT.0).expect("单元序号")]
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

fn device_named(devices: &mut Devices, device: DeviceIdentity) -> &mut SparseBlockDevice {
    &mut devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘")
        .1
}

/// 把 `pointer` 头一条位置条目那块盘上的那一份改一个字节（盘上字节真改了，另一份不动）：那一份整单元 CRC-32C 与位置条目对不上。
fn damage_the_copy_at_the_first_location_of(devices: &mut Devices, pointer: &DataPointer) {
    damage_the_copy_at(devices, &pointer.locations[0], data_unit_bytes());
}

/// 把 `location` 那块盘上那一份（从它的槽起 `unit_bytes` 字节）改一个字节：那一份整单元 CRC-32C 与位置条目对不上，另一份不动。
fn damage_the_copy_at(devices: &mut Devices, location: &LocationEntry, unit_bytes: usize) {
    let device = device_named(devices, location.device);
    let mut unit = vec![0u8; unit_bytes];
    device
        .read_at(location.slot.to_device_offset(), &mut unit)
        .expect("读得出那一份");
    unit[FLIPPED_BYTE_IN_THE_UNIT] ^= 0xFF;
    device
        .write_at(
            location.slot.to_device_offset(),
            &unit,
            WriteDurability::Plain,
        )
        .expect("写回改过的那一份");
    assert_ne!(
        crc32_castagnoli(&unit),
        location.unit_checksum,
        "前提：改过的那一份与位置条目对不上"
    );
}

/// 一份按位置条目读出来、整单元 CRC-32C 与位置条目相符。
fn copy_at_the_location_self_verifies(devices: &Devices, location: &LocationEntry) -> bool {
    let mut unit = vec![0u8; data_unit_bytes()];
    devices
        .iter()
        .find(|(identity, _)| *identity == location.device)
        .expect("池里有这块盘")
        .1
        .read_at(location.slot.to_device_offset(), &mut unit)
        .expect("读得出那一份");
    crc32_castagnoli(&unit) == location.unit_checksum
}

/// 挂载态读者按 `version` 的根打开第一个文件、从 0 读到 size。
fn whole_first_file_read_by_the_mounted_reader(
    image: &MemoryPool,
    version: &TransactionOutput,
) -> Vec<u8> {
    let mounted = open_pool_for_read(image, &version.root).expect("打开挂载态");
    let file = mounted.open_file(image, FIRST_FILE).expect("打开文件");
    let size = file.inode_record().size;
    file.read_at(image, FileOffsetInBytes(0), size)
        .expect("从 0 读到 size")
        .bytes
}

/// 读回的那个单元在这一次发布里换了新槽、两份新写（两份都与新位置条目相符），旧落点两块盘上那两份都照释放之前读盘核隔离。
fn the_damaged_unit_moved_to_new_slots_and_its_old_copies_are_quarantined(
    pool: &PoolOfThreeUnits,
    old_pointer: &DataPointer,
    published: &TransactionOutput,
) {
    let new_pointer = pointer_of_the_damaged_unit(published);
    for (new_location, old_location) in new_pointer.locations.iter().zip(&old_pointer.locations) {
        assert_ne!(
            new_location.slot, old_location.slot,
            "读回的单元落在新槽（盘 {:?}）",
            new_location.device
        );
        assert!(
            copy_at_the_location_self_verifies(&pool.devices, new_location),
            "新槽上那一份新写、与位置条目相符（盘 {:?}）",
            new_location.device
        );
    }
    let mut quarantined: Vec<(DeviceIdentity, u64)> = published
        .quarantined_after_release_checksum_mismatch
        .iter()
        .filter(|copy| copy.unit == TransactionUnit::Data(DAMAGED_UNIT))
        .map(|copy| (copy.device, copy.placement.slot.0))
        .collect();
    quarantined.sort_unstable();
    let mut expected: Vec<(DeviceIdentity, u64)> = old_pointer
        .locations
        .iter()
        .map(|location| (location.device, location.slot.0))
        .collect();
    expected.sort_unstable();
    assert_eq!(
        quarantined, expected,
        "旧落点两块盘上那两份都隔离（一份核出对不上，两份一起留）"
    );
}

/// 接着现行那一版按偏移写一次：交回发布出来的那一版与这个写入口上只坏一份的读回次数。
fn partial_write_into_the_damaged_unit(pool: &mut PoolOfThreeUnits) -> (TransactionOutput, u64) {
    let parameters = parameters();
    let patch = vec![0x5Au8; PARTIAL_WRITE_BYTES];
    let previous = pool.version.clone();
    let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
    let published = publish_partial_write(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FIRST_FILE,
        FileOffsetInBytes(payload_capacity() + PARTIAL_WRITE_OFFSET_IN_THE_UNIT),
        PartialWriteBytes::new(&patch).expect("写的字节不是空的"),
        FIXED_WRITE_TIME_SECONDS + 2,
        pool.instance,
    )
    .expect("部分写发成");
    (
        published,
        writer.old_data_unit_reads_back_with_one_copy_failing(),
    )
}

fn content_after_the_partial_write() -> Vec<u8> {
    let mut content = content_of_three_units();
    let start =
        usize::try_from(payload_capacity() + PARTIAL_WRITE_OFFSET_IN_THE_UNIT).expect("文件偏移");
    content[start..start + PARTIAL_WRITE_BYTES].fill(0x5A);
    content
}

/// 截到第 1 个单元中间：交回发布出来的那一版与这个写入口上只坏一份的读回次数。
fn truncation_into_the_damaged_unit(pool: &mut PoolOfThreeUnits) -> (TransactionOutput, u64) {
    let parameters = parameters();
    let previous = pool.version.clone();
    let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
    let published = publish_size_change(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FIRST_FILE,
        payload_capacity() + TRUNCATION_OFFSET_IN_THE_UNIT,
        FIXED_WRITE_TIME_SECONDS + 2,
        pool.instance,
    )
    .expect("截断的意图那次发成");
    (
        published,
        writer.old_data_unit_reads_back_with_one_copy_failing(),
    )
}

fn content_after_the_truncation() -> Vec<u8> {
    let mut content = content_of_three_units();
    content.truncate(
        usize::try_from(payload_capacity() + TRUNCATION_OFFSET_IN_THE_UNIT).expect("文件长度"),
    );
    content
}

/// 部分写读回第 1 个单元时第一份坏、第二份好：写照成，内容是原内容加这次写的字节，写入口计一次；那个单元换了新槽、两份新写，旧落点隔离。
/// 判别力：读回不计数（`crates/mutations.tsv` 一行）时计数是 0，这一条红。
#[test]
fn a_partial_write_reading_back_a_unit_with_one_copy_damaged_counts_it_and_rewrites_the_unit_to_new_slots(
) {
    let mut pool = pool_of_three_units();
    let old_pointer = pointer_of_the_damaged_unit(&pool.version);
    damage_the_copy_at_the_first_location_of(&mut pool.devices, &old_pointer);
    let (published, reads_with_one_copy_failing) = partial_write_into_the_damaged_unit(&mut pool);
    assert_eq!(
        reads_with_one_copy_failing, 1,
        "读回第 1 个单元时只坏一份：计一次"
    );
    assert_eq!(
        whole_first_file_read_by_the_mounted_reader(&memory_pool_of(&pool.devices), &published),
        content_after_the_partial_write(),
        "读回的是原内容：写之后是原内容加这次写的字节"
    );
    the_damaged_unit_moved_to_new_slots_and_its_old_copies_are_quarantined(
        &pool,
        &old_pointer,
        &published,
    );
}

/// 不弄坏时同一个部分写：计数 0、一份都不隔离。
#[test]
fn a_partial_write_reading_back_intact_units_counts_nothing_and_quarantines_nothing() {
    let mut pool = pool_of_three_units();
    let (published, reads_with_one_copy_failing) = partial_write_into_the_damaged_unit(&mut pool);
    assert_eq!(reads_with_one_copy_failing, 0, "没有坏的一份：计数 0");
    assert!(
        published
            .quarantined_after_release_checksum_mismatch
            .is_empty(),
        "没有坏的一份：一份都不隔离"
    );
    assert_eq!(
        whole_first_file_read_by_the_mounted_reader(&memory_pool_of(&pool.devices), &published),
        content_after_the_partial_write()
    );
}

/// 截到第 1 个单元中间、读回这个尾单元时第一份坏、第二份好：截断的意图那次照成，size 之内是原内容，写入口计一次；尾单元换了新槽、两份新写，
/// 旧落点隔离。判别力：截断读回不计数（`crates/mutations.tsv` 一行，与部分写同一处）时这一条红。
#[test]
fn a_truncation_reading_back_a_tail_unit_with_one_copy_damaged_counts_it_and_rewrites_the_unit_to_new_slots(
) {
    let mut pool = pool_of_three_units();
    let old_pointer = pointer_of_the_damaged_unit(&pool.version);
    damage_the_copy_at_the_first_location_of(&mut pool.devices, &old_pointer);
    let (published, reads_with_one_copy_failing) = truncation_into_the_damaged_unit(&mut pool);
    assert_eq!(
        reads_with_one_copy_failing, 1,
        "读回尾单元时只坏一份：计一次"
    );
    assert_eq!(
        whole_first_file_read_by_the_mounted_reader(&memory_pool_of(&pool.devices), &published),
        content_after_the_truncation(),
        "读回的是原内容：size 之内是截断之前的那一段"
    );
    the_damaged_unit_moved_to_new_slots_and_its_old_copies_are_quarantined(
        &pool,
        &old_pointer,
        &published,
    );
}

/// 不弄坏时同一个截断：计数 0、一份都不隔离。
#[test]
fn a_truncation_reading_back_an_intact_tail_unit_counts_nothing_and_quarantines_nothing() {
    let mut pool = pool_of_three_units();
    let (published, reads_with_one_copy_failing) = truncation_into_the_damaged_unit(&mut pool);
    assert_eq!(reads_with_one_copy_failing, 0, "没有坏的一份：计数 0");
    assert!(
        published
            .quarantined_after_release_checksum_mismatch
            .is_empty(),
        "没有坏的一份：一份都不隔离"
    );
}

/// 挂载态 `read_at` 读整个文件、第 1 个单元第一份坏：读出原内容，这一次读与打开之后累计的都计一次。
/// 判别力：挂载态按位置条目读不计数（`crates/mutations.tsv` 一行）时这一条红。
#[test]
fn a_mounted_read_meeting_a_unit_with_one_copy_damaged_reads_the_content_and_counts_it_once() {
    let mut pool = pool_of_three_units();
    let old_pointer = pointer_of_the_damaged_unit(&pool.version);
    damage_the_copy_at_the_first_location_of(&mut pool.devices, &old_pointer);
    let image = memory_pool_of(&pool.devices);
    let mounted = open_pool_for_read(&image, &pool.version.root).expect("打开挂载态");
    let file = mounted.open_file(&image, FIRST_FILE).expect("打开文件");
    let size = file.inode_record().size;
    let output = file
        .read_at(&image, FileOffsetInBytes(0), size)
        .expect("从 0 读到 size");
    assert_eq!(output.bytes, content_of_three_units(), "读出的是原内容");
    assert_eq!(
        output.observation.data_unit_reads_with_one_copy_failing, 1,
        "这一次读：只坏一份计一次"
    );
    assert_eq!(
        mounted
            .observation_since_open()
            .data_unit_reads_with_one_copy_failing,
        1,
        "打开之后累计：计一次"
    );
}

/// 不弄坏时同一次读：计数 0。
#[test]
fn a_mounted_read_of_intact_units_counts_nothing() {
    let pool = pool_of_three_units();
    let image = memory_pool_of(&pool.devices);
    let mounted = open_pool_for_read(&image, &pool.version.root).expect("打开挂载态");
    let file = mounted.open_file(&image, FIRST_FILE).expect("打开文件");
    let size = file.inode_record().size;
    let output = file
        .read_at(&image, FileOffsetInBytes(0), size)
        .expect("从 0 读到 size");
    assert_eq!(output.bytes, content_of_three_units());
    assert_eq!(output.observation.data_unit_reads_with_one_copy_failing, 0);
    assert_eq!(
        mounted
            .observation_since_open()
            .data_unit_reads_with_one_copy_failing,
        0
    );
}

/// 冷恢复读（不挂载）走到第一个文件、第 1 个单元第一份坏：读出原内容，报给读者一次（套着数的读者计一次）。
/// 判别力：按位置条目读不报只坏一份（`crates/mutations.tsv` 一行）时这一条红。
#[test]
fn a_cold_read_meeting_a_unit_with_one_copy_damaged_reads_the_content_and_reports_it_once() {
    let mut pool = pool_of_three_units();
    let old_pointer = pointer_of_the_damaged_unit(&pool.version);
    damage_the_copy_at_the_first_location_of(&mut pool.devices, &old_pointer);
    let image = memory_pool_of(&pool.devices);
    let counting = PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&image);
    let content = walk_to_file(&counting, &pool.version.root, &mut 0)
        .expect("冷恢复读读通")
        .expect("有第一个文件");
    assert_eq!(content, content_of_three_units(), "读出的是原内容");
    assert_eq!(
        counting.unit_reads_with_one_copy_failing(),
        1,
        "只坏一份的那一个单元报一次"
    );
}

/// 不弄坏时同一次冷恢复读：一次都不报。
#[test]
fn a_cold_read_of_intact_units_reports_nothing() {
    let pool = pool_of_three_units();
    let image = memory_pool_of(&pool.devices);
    let counting = PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&image);
    let content = walk_to_file(&counting, &pool.version.root, &mut 0)
        .expect("冷恢复读读通")
        .expect("有第一个文件");
    assert_eq!(content, content_of_three_units());
    assert_eq!(counting.unit_reads_with_one_copy_failing(), 0);
}

/// 只读挂载（`mounted_read::mount_read_only`）读整个文件、第 1 个单元第一份坏：读出原内容，打开之后累计计一次，盘上逐字节不变（只计数、不修，
///）。判别力：挂载态按位置条目读不计数（`crates/mutations.tsv` 一行）时这一条红。
#[test]
fn a_read_only_mount_meeting_a_unit_with_one_copy_damaged_counts_it_and_leaves_every_disk_byte_unchanged(
) {
    let mut pool = pool_of_three_units();
    let old_pointer = pointer_of_the_damaged_unit(&pool.version);
    damage_the_copy_at_the_first_location_of(&mut pool.devices, &old_pointer);
    let image = memory_pool_of(&pool.devices);
    let image_before_the_read = image.clone();
    let read_only = mount_read_only(&image).expect("只读挂载");
    let file = read_only
        .mounted
        .open_file(&image, FIRST_FILE)
        .expect("打开文件");
    let size = file.inode_record().size;
    let output = file
        .read_at(&image, FileOffsetInBytes(0), size)
        .expect("从 0 读到 size");
    assert_eq!(output.bytes, content_of_three_units(), "读出的是原内容");
    assert_eq!(
        read_only
            .mounted
            .observation_since_open()
            .data_unit_reads_with_one_copy_failing,
        1,
        "打开之后累计：计一次"
    );
    // 不用 assert_eq：两份 4 GiB 稀疏盘的 Debug 输出太大，红了只说哪一样不对。
    assert!(
        image == image_before_the_read,
        "只读挂载只计数、不修：盘上逐字节不变"
    );
    assert!(
        !copy_at_the_location_self_verifies(&pool.devices, &old_pointer.locations[0]),
        "坏的那一份还在原处、没被改写"
    );
}

/// 第一个文件三个单元的池可写挂载，交回会话。
fn session_of_a_writable_mount(pool: &mut PoolOfThreeUnits) -> MountedSession {
    let parameters = parameters();
    let mounted = mount_writable(&parameters, &mut pool.devices).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    session
}

/// 会话现行那一版。
fn current_version_of(session: &MountedSession) -> TransactionOutput {
    session
        .current
        .file_version()
        .expect("现行那一版带文件")
        .clone()
}

/// 经会话在第一个文件第 [`DAMAGED_UNIT`] 个单元里按偏移写一次（同 [`partial_write_into_the_damaged_unit`] 的偏移与长度）。
fn write_into_the_damaged_unit_through_the_session(
    session: &mut MountedSession,
    pool: &mut PoolOfThreeUnits,
    write_time_seconds: u64,
) -> UserChangePublished {
    write_into_the_damaged_unit_through_the_session_on_the_devices(
        session,
        &mut pool.devices,
        write_time_seconds,
    )
}

/// 同 [`write_into_the_damaged_unit_through_the_session`]，盘表是交进来的这一份（外面包了故障注入的那一份也行：会话只核盘的身份）。
fn write_into_the_damaged_unit_through_the_session_on_the_devices<Device: BlockDevice>(
    session: &mut MountedSession,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    write_time_seconds: u64,
) -> UserChangePublished {
    let patch = vec![0x5Au8; PARTIAL_WRITE_BYTES];
    session
        .publish_user_change(
            devices,
            UserChange::WriteAtOffset {
                inode: FIRST_FILE,
                offset: FileOffsetInBytes(payload_capacity() + PARTIAL_WRITE_OFFSET_IN_THE_UNIT),
                bytes: PartialWriteBytes::new(&patch).expect("写的字节不是空的"),
                write_time_seconds,
            },
        )
        .expect("会话里按偏移写发成")
}

/// 一次可写挂载里连做三次部分写，每次之前都把现行那一版第 1 个单元头一条位置条目那一份弄坏：每次读回都只坏一份，会话的累计数逐次是 1、2、3
/// （各次之和），挂上时是 0。
/// 判别力：会话不把写入口上的数并进来、或并成取大不相加（`crates/mutations.tsv` 两行）时这一条红。同一个会话上按次序做三次，是一个场景。
#[test]
fn a_writable_mount_sums_the_one_copy_failing_read_backs_of_every_partial_write_in_it() {
    let mut pool = pool_of_three_units();
    let mut session = session_of_a_writable_mount(&mut pool);
    assert_eq!(
        session.old_data_unit_reads_back_with_one_copy_failing_in_this_mount(),
        0,
        "挂上时 0"
    );
    // 迭代三次；跨轮携带的是会话与盘：每次部分写把第 1 个单元整个重写到新槽，下一轮弄坏的是新槽上那一份。
    for round in 1..=3_u64 {
        let pointer = pointer_of_the_damaged_unit(&current_version_of(&session));
        damage_the_copy_at_the_first_location_of(&mut pool.devices, &pointer);
        write_into_the_damaged_unit_through_the_session(
            &mut session,
            &mut pool,
            FIXED_WRITE_TIME_SECONDS + 2 + round,
        );
        assert_eq!(
            session.old_data_unit_reads_back_with_one_copy_failing_in_this_mount(),
            round,
            "第 {round} 次部分写之后：累计数是前面各次之和"
        );
    }
}

/// 一次可写挂载里读回只坏一份计了一次；同一组盘再可写挂载一次，新会话的累计数从 0 起（只在内存里、不落盘）。
/// 判别力：挂载期状态起步不是 0（`crates/mutations.tsv` 一行）时这一条红。
#[test]
fn a_new_writable_mount_counts_the_one_copy_failing_read_backs_from_zero() {
    let mut pool = pool_of_three_units();
    let mut session = session_of_a_writable_mount(&mut pool);
    let pointer = pointer_of_the_damaged_unit(&current_version_of(&session));
    damage_the_copy_at_the_first_location_of(&mut pool.devices, &pointer);
    write_into_the_damaged_unit_through_the_session(
        &mut session,
        &mut pool,
        FIXED_WRITE_TIME_SECONDS + 3,
    );
    assert_eq!(
        session.old_data_unit_reads_back_with_one_copy_failing_in_this_mount(),
        1,
        "前提：头一次挂载里计了一次"
    );
    drop(session);
    let session_of_the_next_mount = session_of_a_writable_mount(&mut pool);
    assert_eq!(
        session_of_the_next_mount.old_data_unit_reads_back_with_one_copy_failing_in_this_mount(),
        0,
        "新挂载从 0 起"
    );
}

/// 没有活快照时经会话部分写、读回第 1 个单元只坏一份：行为与写入口那一条相同——改动那次发布就把这个单元重写到新槽、旧落点两份都隔离，
/// 不另发搬迁（这次调用只发成改动那一版）。判别力：计划为空也照发那一批（`crates/mutations.tsv` 一行，发布入口断言计划非空）时这一条红。
#[test]
fn a_partial_write_without_a_snapshot_through_the_session_relocates_nothing_more() {
    let mut pool = pool_of_three_units();
    let mut session = session_of_a_writable_mount(&mut pool);
    let before = current_version_of(&session);
    let old_pointer = pointer_of_the_damaged_unit(&before);
    damage_the_copy_at_the_first_location_of(&mut pool.devices, &old_pointer);
    let published = write_into_the_damaged_unit_through_the_session(
        &mut session,
        &mut pool,
        FIXED_WRITE_TIME_SECONDS + 3,
    );
    assert!(
        matches!(
            published.relocation_of_the_old_data_units_kept_for_the_snapshots,
            RelocationOfTheOldDataUnitsKeptForTheSnapshots::NotNeeded
        ),
        "没有活快照钉着：不另发搬迁，{:?}",
        published.relocation_of_the_old_data_units_kept_for_the_snapshots
    );
    assert_eq!(
        published.versions_published.len(),
        1,
        "这次调用只发成改动那一版"
    );
    the_damaged_unit_moved_to_new_slots_and_its_old_copies_are_quarantined(
        &pool,
        &old_pointer,
        &current_version_of(&session),
    );
}

/// 有活快照、第 1 个单元头一条位置条目那一份已弄坏的会话（可写挂载、建快照、再弄坏），连同核搬迁与读快照要的几样。
struct SessionUnderALiveSnapshotWithTheUnitDamaged {
    session: MountedSession,
    snapshot: TreeIdentifier,
    /// 建快照之后那一版里第 1 个单元的指针（弄坏的是它头一条位置条目那一份）。
    old_pointer: DataPointer,
    key_of_the_damaged_unit: Vec<u8>,
}

/// 第一个文件三个单元的池可写挂载、经会话建一个快照，再把现行那一版第 1 个单元头一条位置条目那一份弄坏。
fn session_under_a_live_snapshot_with_the_unit_damaged(
    pool: &mut PoolOfThreeUnits,
) -> SessionUnderALiveSnapshotWithTheUnitDamaged {
    let mut session = session_of_a_writable_mount(pool);
    session
        .publish_user_change(&mut pool.devices, UserChange::CreateSnapshot)
        .expect("建快照");
    let before = current_version_of(&session);
    let snapshot = before
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .expect("树表里有快照的码 14 条目")
        .tree;
    let old_pointer = pointer_of_the_damaged_unit(&before);
    let key_of_the_damaged_unit = before
        .mapped_units
        .iter()
        .find(|(identity, _)| *identity == TransactionUnit::Data(DAMAGED_UNIT))
        .expect("数据单元进映射")
        .1
        .clone();
    damage_the_copy_at_the_first_location_of(&mut pool.devices, &old_pointer);
    SessionUnderALiveSnapshotWithTheUnitDamaged {
        session,
        snapshot,
        old_pointer,
        key_of_the_damaged_unit,
    }
}

/// 另发的那一批搬成之后的会话现行那一版上：那个单元仍被快照钉着、留存条目换到新槽，两份都与留存条目的新位置项相符，旧落点两块盘上那两份都照
/// 释放之前读盘核隔离（一份核出对不上，两份一起留）；快照读出原内容。
fn the_unit_kept_for_the_snapshot_moved_to_new_slots_and_its_old_copies_are_quarantined(
    pool: &PoolOfThreeUnits,
    under_the_snapshot: &SessionUnderALiveSnapshotWithTheUnitDamaged,
) {
    let after = current_version_of(&under_the_snapshot.session);
    let (_, kept_locations) = after
        .mapping_entries_kept_for_the_snapshots
        .iter()
        .find(|(kept_key, _)| *kept_key == under_the_snapshot.key_of_the_damaged_unit)
        .expect("那个单元仍被快照钉着、映射条目留存");
    for (new_location, old_location) in kept_locations
        .iter()
        .zip(&under_the_snapshot.old_pointer.locations)
    {
        assert_ne!(
            new_location.slot, old_location.slot,
            "留存条目换到新槽（盘 {:?}）",
            new_location.device
        );
        assert!(
            copy_at_the_location_self_verifies(&pool.devices, new_location),
            "新槽上那一份新写、与留存条目的位置项相符（盘 {:?}）",
            new_location.device
        );
    }
    let mut quarantined: Vec<(DeviceIdentity, u64)> = after
        .quarantined_after_release_checksum_mismatch
        .iter()
        .filter(|copy| matches!(copy.unit, TransactionUnit::Data(_)))
        .map(|copy| (copy.device, copy.placement.slot.0))
        .collect();
    quarantined.sort_unstable();
    let mut expected: Vec<(DeviceIdentity, u64)> = under_the_snapshot
        .old_pointer
        .locations
        .iter()
        .map(|location| (location.device, location.slot.0))
        .collect();
    expected.sort_unstable();
    assert_eq!(
        quarantined, expected,
        "旧落点两块盘上那两份都隔离（一份核出对不上，两份一起留）"
    );
    assert_eq!(
        read_the_first_file_of_a_snapshot(
            &memory_pool_of(&pool.devices),
            &after.root,
            under_the_snapshot.snapshot
        )
        .expect("读快照"),
        SnapshotFileReading::File(content_of_three_units()),
        "快照读出原内容"
    );
}

/// 另发的那一批搬成了：交回搬的那几把 key 与为它推的几串抬 F。别的结局 panic，带着交回的那一种。
fn relocated_keys_and_floor_raises(
    published: &UserChangePublished,
) -> (&[Vec<u8>], &FloorRaisesPushedForTheRelocation) {
    match &published.relocation_of_the_old_data_units_kept_for_the_snapshots {
        RelocationOfTheOldDataUnitsKeptForTheSnapshots::Relocated {
            mapping_keys,
            floor_raises,
        } => (mapping_keys.as_slice(), floor_raises),
        RelocationOfTheOldDataUnitsKeptForTheSnapshots::NotNeeded
        | RelocationOfTheOldDataUnitsKeptForTheSnapshots::NoSpaceAfterRaisingTheFloor { .. }
        | RelocationOfTheOldDataUnitsKeptForTheSnapshots::RefusedNotForSpace { .. }
        | RelocationOfTheOldDataUnitsKeptForTheSnapshots::FloorRaiseFailedWhilePushingForSpace {
            ..
        } => panic!(
            "有活快照钉着只坏一份的旧单元：应另发一批搬迁、搬成，交回的是 {:?}",
            published.relocation_of_the_old_data_units_kept_for_the_snapshots
        ),
    }
}

/// 有活快照时经会话部分写、读回第 1 个单元只坏一份：旧单元换下来进 deadlist、映射条目留存，会话另发一批搬迁——快照里的这个单元搬到新槽、
/// 两份都与留存条目的新位置项相符，旧落点两块盘上那两份都照释放之前读盘核隔离；快照读出原内容。会话计一次。一次都没被拒，一串抬 F 都不推。
/// 判别力：写入口不记只坏一份那个单元的映射 key、或会话发成那一批之后不换现行那一版（`crates/mutations.tsv` 两行）时这一条红。
#[test]
fn a_partial_write_under_a_live_snapshot_relocates_the_pinned_unit_read_with_one_copy_damaged() {
    let mut pool = pool_of_three_units();
    let mut under_the_snapshot = session_under_a_live_snapshot_with_the_unit_damaged(&mut pool);
    let published = write_into_the_damaged_unit_through_the_session(
        &mut under_the_snapshot.session,
        &mut pool,
        FIXED_WRITE_TIME_SECONDS + 3,
    );
    assert_eq!(
        under_the_snapshot
            .session
            .old_data_unit_reads_back_with_one_copy_failing_in_this_mount(),
        1,
        "读回第 1 个单元时只坏一份：计一次"
    );
    let (mapping_keys, floor_raises) = relocated_keys_and_floor_raises(&published);
    assert_eq!(
        mapping_keys,
        &[under_the_snapshot.key_of_the_damaged_unit.clone()],
        "另发的那一批搬的就是读回时只坏一份的那个单元"
    );
    assert!(
        floor_raises.floor_raises.is_empty(),
        "那一批一次都没被拒：一串抬 F 都不推"
    );
    the_unit_kept_for_the_snapshot_moved_to_new_slots_and_its_old_copies_are_quarantined(
        &pool,
        &under_the_snapshot,
    );
}

/// 有活快照时经会话截到第 1 个单元中间、读回这个尾单元只坏一份：与部分写那一条同一套断言——会话计一次、另发一批搬迁、快照里的这个单元搬到新槽、
/// 两份都与留存条目的新位置项相符，旧落点两块盘上那两份都隔离；快照读出原内容，现行那一版读出截断之后的内容。
/// 判别力：截断读回尾单元那一处不记只坏一份的映射 key（`crates/mutations.tsv` 一行）时这一条红。
#[test]
fn a_truncation_under_a_live_snapshot_relocates_the_pinned_tail_unit_read_with_one_copy_damaged() {
    let mut pool = pool_of_three_units();
    let mut under_the_snapshot = session_under_a_live_snapshot_with_the_unit_damaged(&mut pool);
    let published = under_the_snapshot
        .session
        .publish_user_change(
            &mut pool.devices,
            UserChange::Truncate {
                inode: FIRST_FILE,
                new_size: payload_capacity() + TRUNCATION_OFFSET_IN_THE_UNIT,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            },
        )
        .expect("会话里截断发成");
    assert_eq!(
        under_the_snapshot
            .session
            .old_data_unit_reads_back_with_one_copy_failing_in_this_mount(),
        1,
        "读回尾单元时只坏一份：计一次"
    );
    let (mapping_keys, _) = relocated_keys_and_floor_raises(&published);
    assert_eq!(
        mapping_keys,
        &[under_the_snapshot.key_of_the_damaged_unit.clone()],
        "另发的那一批搬的就是读回时只坏一份的那个尾单元"
    );
    the_unit_kept_for_the_snapshot_moved_to_new_slots_and_its_old_copies_are_quarantined(
        &pool,
        &under_the_snapshot,
    );
    assert_eq!(
        whole_first_file_read_by_the_mounted_reader(
            &memory_pool_of(&pool.devices),
            &current_version_of(&under_the_snapshot.session)
        ),
        content_after_the_truncation(),
        "现行那一版读出截断之后的内容"
    );
}

/// 有活快照时经会话部分写、读回第 1 个单元只坏一份，另发的那一批头一次被空间拒（只供测试的开关强制拒一次）：推一串抬 F、回收之后再发同一批，
/// 这一次搬成——快照里的这个单元搬到新槽、两份新写、与留存条目的新位置项相符，旧落点两份都隔离；改动照成，现行那一版读出写之后的内容。
/// 判别力：被拒不当作空间不够（不推抬 F、不再试）、或推的那几串不记进结局（`crates/mutations.tsv` 两行）时这一条红。
#[test]
fn a_relocation_under_a_live_snapshot_refused_once_for_space_relocates_after_raising_the_floor() {
    let mut pool = pool_of_three_units();
    let mut under_the_snapshot = session_under_a_live_snapshot_with_the_unit_damaged(&mut pool);
    under_the_snapshot
        .session
        .compaction
        .force_the_relocation_kept_for_the_snapshots_refused_for_space_for_tests(
            RelocationKeptForTheSnapshotsRefusedForSpace::ForcedForTheNextAttemptsForTests {
                attempts: NonZeroU64::new(1).expect("拒一次：1 不是 0"),
            },
        );
    let published = write_into_the_damaged_unit_through_the_session(
        &mut under_the_snapshot.session,
        &mut pool,
        FIXED_WRITE_TIME_SECONDS + 3,
    );
    let (mapping_keys, floor_raises) = relocated_keys_and_floor_raises(&published);
    assert_eq!(
        mapping_keys,
        &[under_the_snapshot.key_of_the_damaged_unit.clone()],
        "抬 F 之后再发的那一批搬的就是读回时只坏一份的那个单元"
    );
    assert_eq!(
        floor_raises.floor_raises.len(),
        1,
        "被拒一次：推一串抬 F 再试"
    );
    assert_eq!(
        floor_raises.refusals_that_pushed_the_floor_raises.len(),
        1,
        "那一串是因为那一次被拒才推的"
    );
    the_unit_kept_for_the_snapshot_moved_to_new_slots_and_its_old_copies_are_quarantined(
        &pool,
        &under_the_snapshot,
    );
    assert_eq!(
        whole_first_file_read_by_the_mounted_reader(
            &memory_pool_of(&pool.devices),
            &current_version_of(&under_the_snapshot.session)
        ),
        content_after_the_partial_write(),
        "改动照成：现行那一版读出写之后的内容"
    );
}

/// 有活快照时经会话部分写、另发的那一批每一次都被空间拒（只供测试的开关）：推抬 F 推到推满、交回推满仍放不下的结局；这一批一个字节都没写——
/// 现行那一版是推的最后一串的最后一次空发布，留存条目仍在旧落点、坏的那一份还在旧落点上没被改写、另一份照旧核得上，旧落点没进隔离；快照照旧读出原内容。
/// 判别力：推了抬 F 之后会话不换现行那一版、或开关不拒（`crates/mutations.tsv` 两行）时这一条红。
#[test]
fn a_relocation_under_a_live_snapshot_refused_for_space_every_time_writes_nothing_after_raising_the_floor_to_the_limit(
) {
    let mut pool = pool_of_three_units();
    let mut under_the_snapshot = session_under_a_live_snapshot_with_the_unit_damaged(&mut pool);
    under_the_snapshot
        .session
        .compaction
        .force_the_relocation_kept_for_the_snapshots_refused_for_space_for_tests(
            RelocationKeptForTheSnapshotsRefusedForSpace::ForcedOnEveryAttemptForTests,
        );
    let published = write_into_the_damaged_unit_through_the_session(
        &mut under_the_snapshot.session,
        &mut pool,
        FIXED_WRITE_TIME_SECONDS + 3,
    );
    let floor_raises = match &published.relocation_of_the_old_data_units_kept_for_the_snapshots {
        RelocationOfTheOldDataUnitsKeptForTheSnapshots::NoSpaceAfterRaisingTheFloor {
            floor_raises,
            ..
        } => floor_raises,
        RelocationOfTheOldDataUnitsKeptForTheSnapshots::NotNeeded
        | RelocationOfTheOldDataUnitsKeptForTheSnapshots::Relocated { .. }
        | RelocationOfTheOldDataUnitsKeptForTheSnapshots::RefusedNotForSpace { .. }
        | RelocationOfTheOldDataUnitsKeptForTheSnapshots::FloorRaiseFailedWhilePushingForSpace {
            ..
        } => panic!(
            "那一批每一次都被空间拒：应推满仍放不下，交回的是 {:?}",
            published.relocation_of_the_old_data_units_kept_for_the_snapshots
        ),
    };
    let last_empty_publish = floor_raises
        .floor_raises
        .last()
        .expect("推满之前至少推成一串抬 F")
        .publishes
        .last()
        .expect("推成的一串至少一次空发布");
    let current = current_version_of(&under_the_snapshot.session);
    assert_eq!(
        (current.root.instance, current.root.checkpoint_txg),
        (
            last_empty_publish.root.instance,
            last_empty_publish.root.checkpoint_txg
        ),
        "现行那一版是推的最后一串的最后一次空发布：那一批没发成"
    );
    let (_, kept_locations) = current
        .mapping_entries_kept_for_the_snapshots
        .iter()
        .find(|(kept_key, _)| *kept_key == under_the_snapshot.key_of_the_damaged_unit)
        .expect("那个单元仍被快照钉着、映射条目留存");
    assert_eq!(
        kept_locations, &under_the_snapshot.old_pointer.locations,
        "留存条目仍在旧落点：那一批没搬"
    );
    assert!(
        !copy_at_the_location_self_verifies(
            &pool.devices,
            &under_the_snapshot.old_pointer.locations[0]
        ),
        "坏的那一份还在旧落点上、没被改写"
    );
    assert!(
        copy_at_the_location_self_verifies(
            &pool.devices,
            &under_the_snapshot.old_pointer.locations[1]
        ),
        "另一份照旧核得上"
    );
    assert!(
        !current
            .quarantined_after_release_checksum_mismatch
            .iter()
            .any(|copy| under_the_snapshot
                .old_pointer
                .locations
                .iter()
                .any(|location| location.device == copy.device
                    && location.slot == copy.placement.slot)),
        "旧落点没进隔离：那一批没释放它"
    );
    assert_eq!(
        read_the_first_file_of_a_snapshot(
            &memory_pool_of(&pool.devices),
            &current.root,
            under_the_snapshot.snapshot
        )
        .expect("读快照"),
        SnapshotFileReading::File(content_of_three_units()),
        "快照照旧读出原内容（读的是另一份）"
    );
}

/// 有活快照时经会话部分写、另发的那一批每一次都被空间拒：用户改动照样发成——调用交回 `Ok`、这次调用只发成改动那一版、会话累计计一次，
/// 现行那一版读出写之后的内容。判别力：开关不拒（`crates/mutations.tsv` 一行，与上一条同一处）时交回的是搬成的结局，这一条红在结局那一判。
#[test]
fn a_partial_write_under_a_live_snapshot_is_published_while_its_relocation_stays_refused_for_space()
{
    let mut pool = pool_of_three_units();
    let mut under_the_snapshot = session_under_a_live_snapshot_with_the_unit_damaged(&mut pool);
    under_the_snapshot
        .session
        .compaction
        .force_the_relocation_kept_for_the_snapshots_refused_for_space_for_tests(
            RelocationKeptForTheSnapshotsRefusedForSpace::ForcedOnEveryAttemptForTests,
        );
    let published = write_into_the_damaged_unit_through_the_session(
        &mut under_the_snapshot.session,
        &mut pool,
        FIXED_WRITE_TIME_SECONDS + 3,
    );
    assert!(
        matches!(
            published.relocation_of_the_old_data_units_kept_for_the_snapshots,
            RelocationOfTheOldDataUnitsKeptForTheSnapshots::NoSpaceAfterRaisingTheFloor { .. }
        ),
        "前提：那一批推满仍放不下，{:?}",
        published.relocation_of_the_old_data_units_kept_for_the_snapshots
    );
    assert_eq!(
        published.versions_published.len(),
        1,
        "这次调用只发成改动那一版（推的抬 F 不在这里）"
    );
    assert_eq!(
        under_the_snapshot
            .session
            .old_data_unit_reads_back_with_one_copy_failing_in_this_mount(),
        1,
        "读回第 1 个单元时只坏一份：计一次"
    );
    assert_eq!(
        whole_first_file_read_by_the_mounted_reader(
            &memory_pool_of(&pool.devices),
            &current_version_of(&under_the_snapshot.session)
        ),
        content_after_the_partial_write(),
        "改动照成：现行那一版读出写之后的内容"
    );
    assert_eq!(
        whole_first_file_read_by_the_mounted_reader(
            &memory_pool_of(&pool.devices),
            &published.versions_published[0]
        ),
        content_after_the_partial_write(),
        "发成的那一版就是这次改动"
    );
}

/// 可写挂载建上一版时读第一个文件第 1 个单元、它头一条位置条目那一份坏、另一份好：挂载照常做成，挂载时读到的只坏一份计一次；会话里读—改—写那一份
/// 还是 0（按读的地方分开报）。判别力：建上一版不经数「只坏一份」的那一层读、或挂载数出来的不记进这次挂载的状态（`crates/mutations.tsv` 两行）时这一条红。
#[test]
fn a_writable_mount_building_on_a_unit_with_one_copy_damaged_counts_it_while_mounting() {
    let mut pool = pool_of_three_units();
    let old_pointer = pointer_of_the_damaged_unit(&pool.version);
    damage_the_copy_at_the_first_location_of(&mut pool.devices, &old_pointer);
    let session = session_of_a_writable_mount(&mut pool);
    assert_eq!(
        session.unit_reads_with_one_copy_failing_while_mounting(),
        1,
        "挂载建上一版时读第 1 个单元只坏一份：计一次"
    );
    assert_eq!(
        session.old_data_unit_reads_back_with_one_copy_failing_in_this_mount(),
        0,
        "会话里还没有读—改—写：那一份是 0"
    );
}

/// 不弄坏时同一次可写挂载：挂载时读到的只坏一份是 0。
#[test]
fn a_writable_mount_building_on_intact_units_counts_nothing_while_mounting() {
    let mut pool = pool_of_three_units();
    let session = session_of_a_writable_mount(&mut pool);
    assert_eq!(
        session.unit_reads_with_one_copy_failing_while_mounting(),
        0,
        "没有坏的一份：挂载时计数 0"
    );
}

/// 有活快照时经会话部分写、读回第 1 个单元只坏一份，另发的那一批被拒、不是空间不够（只供测试的开关强制：计划对不上那一形）：不推抬 F、不再试，
/// 交回被拒不是空间不够的结局、原样带着那一拒；这一批一个字节都没写——现行那一版停在改动发成的那一版上，留存条目仍在旧落点、坏的那一份还在
/// 旧落点上；用户改动照样发成——这次调用只发成改动那一版，现行那一版读出写之后的内容。
/// 判别力：开关不拒、或开关拒成了空间不够（`crates/mutations.tsv` 两行）时这一条红；新挂载上这个开关不是关的（一行）时
/// `a_partial_write_under_a_live_snapshot_relocates_the_pinned_unit_read_with_one_copy_damaged` 红。
#[test]
fn a_relocation_under_a_live_snapshot_refused_not_for_space_is_not_retried_while_the_change_stays_published(
) {
    let mut pool = pool_of_three_units();
    let mut under_the_snapshot = session_under_a_live_snapshot_with_the_unit_damaged(&mut pool);
    under_the_snapshot
        .session
        .compaction
        .force_the_relocation_kept_for_the_snapshots_refused_not_for_space_for_tests(
            RelocationKeptForTheSnapshotsRefusedNotForSpace::ForcedOnEveryAttemptForTests,
        );
    let published = write_into_the_damaged_unit_through_the_session(
        &mut under_the_snapshot.session,
        &mut pool,
        FIXED_WRITE_TIME_SECONDS + 3,
    );
    let (refusal, floor_raises) = match &published
        .relocation_of_the_old_data_units_kept_for_the_snapshots
    {
        RelocationOfTheOldDataUnitsKeptForTheSnapshots::RefusedNotForSpace {
            refusal,
            floor_raises,
        } => (refusal, floor_raises),
        RelocationOfTheOldDataUnitsKeptForTheSnapshots::NotNeeded
        | RelocationOfTheOldDataUnitsKeptForTheSnapshots::Relocated { .. }
        | RelocationOfTheOldDataUnitsKeptForTheSnapshots::NoSpaceAfterRaisingTheFloor { .. }
        | RelocationOfTheOldDataUnitsKeptForTheSnapshots::FloorRaiseFailedWhilePushingForSpace {
            ..
        } => panic!(
            "那一批被拒、不是空间不够：应交回 RefusedNotForSpace，交回的是 {:?}",
            published.relocation_of_the_old_data_units_kept_for_the_snapshots
        ),
    };
    assert!(
        matches!(
            &**refusal,
            RelocationBatchRefused::PlannedUnitNotInTheVersion { mapping_key }
                if *mapping_key == under_the_snapshot.key_of_the_damaged_unit
        ),
        "交回的是开关造的那一拒原样（计划对不上那一形，点名那个单元的 key），{refusal:?}"
    );
    assert!(
        floor_raises.floor_raises.is_empty(),
        "不是空间不够：一串抬 F 都不推"
    );
    assert!(
        floor_raises
            .refusals_that_pushed_the_floor_raises
            .is_empty(),
        "不是空间不够：没有推抬 F 的那一次被拒"
    );
    assert_eq!(
        published.versions_published.len(),
        1,
        "这次调用只发成改动那一版"
    );
    let current = current_version_of(&under_the_snapshot.session);
    assert_eq!(
        (current.root.instance, current.root.checkpoint_txg),
        (
            published.versions_published[0].root.instance,
            published.versions_published[0].root.checkpoint_txg
        ),
        "现行那一版停在改动发成的那一版上：那一批没发成、不再试"
    );
    let (_, kept_locations) = current
        .mapping_entries_kept_for_the_snapshots
        .iter()
        .find(|(kept_key, _)| *kept_key == under_the_snapshot.key_of_the_damaged_unit)
        .expect("那个单元仍被快照钉着、映射条目留存");
    assert_eq!(
        kept_locations, &under_the_snapshot.old_pointer.locations,
        "留存条目仍在旧落点：那一批没搬"
    );
    assert!(
        !copy_at_the_location_self_verifies(
            &pool.devices,
            &under_the_snapshot.old_pointer.locations[0]
        ),
        "坏的那一份还在旧落点上、没被改写"
    );
    assert_eq!(
        whole_first_file_read_by_the_mounted_reader(&memory_pool_of(&pool.devices), &current),
        content_after_the_partial_write(),
        "改动照成：现行那一版读出写之后的内容"
    );
}

/// 有活快照时经会话部分写、读回第 1 个单元只坏一份，另发的那一批头一次被空间拒（只供测试的开关强制拒一次），为它推的那一串抬 F 自己报错
/// （只供测试的开关强制：实例表读不出那一种）：交回抬 F 自己报错的结局，错原样、推它的是那一次空间准入拒、之前一串都没推成；不当作空间不够、
/// 不再试——现行那一版停在改动发成的那一版上，留存条目仍在旧落点；用户改动照样发成，现行那一版读出写之后的内容。
/// 判别力：开关不让抬 F 报错（`crates/mutations.tsv` 一行）时那一串真推成、再发搬成，这一条红；新挂载上这个开关不是关的（一行）时
/// `a_relocation_under_a_live_snapshot_refused_once_for_space_relocates_after_raising_the_floor` 红。
#[test]
fn a_floor_raise_failing_while_pushing_for_the_relocation_under_a_live_snapshot_is_handed_back_while_the_change_stays_published(
) {
    let mut pool = pool_of_three_units();
    let mut under_the_snapshot = session_under_a_live_snapshot_with_the_unit_damaged(&mut pool);
    under_the_snapshot
        .session
        .compaction
        .force_the_relocation_kept_for_the_snapshots_refused_for_space_for_tests(
            RelocationKeptForTheSnapshotsRefusedForSpace::ForcedForTheNextAttemptsForTests {
                attempts: NonZeroU64::new(1).expect("拒一次：1 不是 0"),
            },
        );
    under_the_snapshot
        .session
        .compaction
        .force_the_floor_raise_for_the_relocation_kept_for_the_snapshots_to_fail_for_tests(
            FloorRaiseForTheRelocationKeptForTheSnapshotsFails::ForcedOnEveryRaiseForTests,
        );
    let published = write_into_the_damaged_unit_through_the_session(
        &mut under_the_snapshot.session,
        &mut pool,
        FIXED_WRITE_TIME_SECONDS + 3,
    );
    let (cause, refusal_that_pushed_the_failed_raise, floor_raises) = match &published
        .relocation_of_the_old_data_units_kept_for_the_snapshots
    {
        RelocationOfTheOldDataUnitsKeptForTheSnapshots::FloorRaiseFailedWhilePushingForSpace {
            cause,
            refusal_that_pushed_the_failed_raise,
            floor_raises,
        } => (cause, refusal_that_pushed_the_failed_raise, floor_raises),
        RelocationOfTheOldDataUnitsKeptForTheSnapshots::NotNeeded
        | RelocationOfTheOldDataUnitsKeptForTheSnapshots::Relocated { .. }
        | RelocationOfTheOldDataUnitsKeptForTheSnapshots::NoSpaceAfterRaisingTheFloor { .. }
        | RelocationOfTheOldDataUnitsKeptForTheSnapshots::RefusedNotForSpace { .. } => panic!(
            "为那一批推的抬 F 自己报错：应交回 FloorRaiseFailedWhilePushingForSpace，交回的是 {:?}",
            published.relocation_of_the_old_data_units_kept_for_the_snapshots
        ),
    };
    assert!(
        matches!(&**cause, MountError::InstanceTableUnreadable),
        "抬 F 的错原样交回（开关造的那一种），{cause:?}"
    );
    assert!(
        matches!(
            &**refusal_that_pushed_the_failed_raise,
            RelocationBatchRefused::Publish(PublishError::SpaceAdmissionRefused(_))
        ),
        "推那一串抬 F 的是那一次空间准入拒，{refusal_that_pushed_the_failed_raise:?}"
    );
    assert!(
        floor_raises.floor_raises.is_empty(),
        "头一串就报错：之前一串都没推成"
    );
    assert_eq!(
        published.versions_published.len(),
        1,
        "这次调用只发成改动那一版"
    );
    let current = current_version_of(&under_the_snapshot.session);
    assert_eq!(
        (current.root.instance, current.root.checkpoint_txg),
        (
            published.versions_published[0].root.instance,
            published.versions_published[0].root.checkpoint_txg
        ),
        "现行那一版停在改动发成的那一版上：抬 F 一次都没落盘、那一批没发成"
    );
    let (_, kept_locations) = current
        .mapping_entries_kept_for_the_snapshots
        .iter()
        .find(|(kept_key, _)| *kept_key == under_the_snapshot.key_of_the_damaged_unit)
        .expect("那个单元仍被快照钉着、映射条目留存");
    assert_eq!(
        kept_locations, &under_the_snapshot.old_pointer.locations,
        "留存条目仍在旧落点：那一批没搬"
    );
    assert_eq!(
        whole_first_file_read_by_the_mounted_reader(&memory_pool_of(&pool.devices), &current),
        content_after_the_partial_write(),
        "改动照成：现行那一版读出写之后的内容"
    );
}

/// 两块盘外面各包一层故障注入、共用一份计划。
type FaultInjectingDevices = Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)>;

/// 第一个文件三个单元的池走到「有活快照、第 1 个单元那一份已弄坏」的会话（[`session_under_a_live_snapshot_with_the_unit_damaged`]），
/// 再把池里两块盘拿出来、各包一层故障注入（计划先不上膛）：会话之后的改动发在包过的那一份盘表上。
fn session_under_a_live_snapshot_with_the_unit_damaged_behind_fault_injection(
    pool: &mut PoolOfThreeUnits,
) -> (
    SessionUnderALiveSnapshotWithTheUnitDamaged,
    FaultInjectingDevices,
    SharedFaultPlan,
) {
    let under_the_snapshot = session_under_a_live_snapshot_with_the_unit_damaged(pool);
    let plan = SharedFaultPlan::unarmed(geometry());
    let devices = std::mem::take(&mut pool.devices)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, device, plan.clone()),
            )
        })
        .collect();
    (under_the_snapshot, devices, plan)
}

/// 在一份同样造出来的池上不注入地数一遍：有活快照时经会话部分写那一次调用里，改动自己（另发的那一批之前）往盘上发了几次写。
/// 那一批由开关强制拒成不是空间不够、一个写都不发，这次调用里的写就只剩改动与之后的 discard
/// （一段一次，`FaultInjectingBlockDevice` 的 discard 记进整池写数），减掉 discard 那几次。
fn writes_of_the_partial_write_before_the_relocation_under_a_live_snapshot() -> u64 {
    let mut pool = pool_of_three_units();
    let (mut under_the_snapshot, mut devices, plan) =
        session_under_a_live_snapshot_with_the_unit_damaged_behind_fault_injection(&mut pool);
    under_the_snapshot
        .session
        .compaction
        .force_the_relocation_kept_for_the_snapshots_refused_not_for_space_for_tests(
            RelocationKeptForTheSnapshotsRefusedNotForSpace::ForcedOnEveryAttemptForTests,
        );
    let writes_before_the_call = plan.calls().writes;
    let published = write_into_the_damaged_unit_through_the_session_on_the_devices(
        &mut under_the_snapshot.session,
        &mut devices,
        FIXED_WRITE_TIME_SECONDS + 3,
    );
    let writes_in_the_call = plan.calls().writes - writes_before_the_call;
    assert!(
        matches!(
            published.relocation_of_the_old_data_units_kept_for_the_snapshots,
            RelocationOfTheOldDataUnitsKeptForTheSnapshots::RefusedNotForSpace { .. }
        ),
        "数的那一遍：那一批被开关拒成不是空间不够、一个写都没发，{:?}",
        published.relocation_of_the_old_data_units_kept_for_the_snapshots
    );
    let discards = &published.discards_after_the_change;
    let discard_writes =
        u64::try_from(discards.applied.len() + discards.unsupported.len() + discards.failed.len())
            .expect("discard 的段数装得进 u64");
    writes_in_the_call - discard_writes
}

/// 有活快照时经会话部分写、读回第 1 个单元只坏一份，另发的那一批在它的头一次写上报块设备错（故障注入：改动那几次写之后的下一次写，次数先在
/// 一份同样的池上数出来）：照发布路径冻结——落盘中途失败的那一次冻结在分配器上、改动之后一段 discard 都不发；被拒不是空间不够——交回
/// 被拒不是空间不够的结局、原样带着块设备错，不推抬 F、不再试；用户改动照样发成——这次调用只发成改动那一版、现行那一版就是它、读出写之后的内容。
/// 判别力：发布路径的错一律当作空间不够（`crates/mutations.tsv` 一行）时为它推抬 F，这一条红。
#[test]
fn a_block_device_error_on_the_first_write_of_the_relocation_under_a_live_snapshot_freezes_that_publish_and_is_not_retried(
) {
    let writes_before_the_relocation =
        writes_of_the_partial_write_before_the_relocation_under_a_live_snapshot();
    let mut pool = pool_of_three_units();
    let (mut under_the_snapshot, mut devices, plan) =
        session_under_a_live_snapshot_with_the_unit_damaged_behind_fault_injection(&mut pool);
    plan.arm(FaultSchedule::the_nth_call_across_the_pool(
        InjectedFault::WriteFails,
        writes_before_the_relocation + 1,
    ));
    let published = write_into_the_damaged_unit_through_the_session_on_the_devices(
        &mut under_the_snapshot.session,
        &mut devices,
        FIXED_WRITE_TIME_SECONDS + 3,
    );
    plan.disarm();
    assert_eq!(
        plan.fired_count(),
        1,
        "前提：注入真的发生了一次（改动那几次写之后的下一次写）"
    );
    let (refusal, floor_raises) = match &published
        .relocation_of_the_old_data_units_kept_for_the_snapshots
    {
        RelocationOfTheOldDataUnitsKeptForTheSnapshots::RefusedNotForSpace {
            refusal,
            floor_raises,
        } => (refusal, floor_raises),
        RelocationOfTheOldDataUnitsKeptForTheSnapshots::NotNeeded
        | RelocationOfTheOldDataUnitsKeptForTheSnapshots::Relocated { .. }
        | RelocationOfTheOldDataUnitsKeptForTheSnapshots::NoSpaceAfterRaisingTheFloor { .. }
        | RelocationOfTheOldDataUnitsKeptForTheSnapshots::FloorRaiseFailedWhilePushingForSpace {
            ..
        } => panic!(
            "那一批落盘时块设备错：应交回 RefusedNotForSpace，交回的是 {:?}",
            published.relocation_of_the_old_data_units_kept_for_the_snapshots
        ),
    };
    assert!(
        matches!(
            &**refusal,
            RelocationBatchRefused::Publish(PublishError::BlockDevice(_))
        ),
        "交回的是发布路径的块设备错原样，{refusal:?}"
    );
    assert!(
        floor_raises.floor_raises.is_empty(),
        "块设备错不是空间不够：一串抬 F 都不推"
    );
    assert!(
        under_the_snapshot
            .session
            .allocator
            .frozen_publish()
            .is_some(),
        "照发布路径冻结：落盘中途失败的那一次冻结在分配器上，等原样重发"
    );
    assert!(
        published
            .discards_after_the_change
            .txg_of_the_frozen_publish_withholding_every_discard
            .is_some(),
        "冻结着：改动之后一段 discard 都不发"
    );
    assert_eq!(
        published.versions_published.len(),
        1,
        "这次调用只发成改动那一版"
    );
    let current = current_version_of(&under_the_snapshot.session);
    assert_eq!(
        (current.root.instance, current.root.checkpoint_txg),
        (
            published.versions_published[0].root.instance,
            published.versions_published[0].root.checkpoint_txg
        ),
        "现行那一版停在改动发成的那一版上：那一批没发成"
    );
    pool.devices = devices
        .into_iter()
        .map(|(identity, device)| (identity, device.into_inner()))
        .collect();
    assert_eq!(
        whole_first_file_read_by_the_mounted_reader(&memory_pool_of(&pool.devices), &current),
        content_after_the_partial_write(),
        "改动照成：现行那一版读出写之后的内容"
    );
}

/// `version` 里第一个文件第 [`DAMAGED_UNIT`] 个数据单元的映射 key。
fn mapping_key_of_the_damaged_unit(version: &TransactionOutput) -> Vec<u8> {
    version
        .mapped_units
        .iter()
        .find(|(identity, _)| *identity == TransactionUnit::Data(DAMAGED_UNIT))
        .expect("数据单元进映射")
        .1
        .clone()
}

/// 第一个文件三个单元的池可写挂载：挂载时读到只坏一份的次数。
fn unit_reads_with_one_copy_failing_while_mounting(pool: &mut PoolOfThreeUnits) -> u64 {
    session_of_a_writable_mount(pool).unit_reads_with_one_copy_failing_while_mounting()
}

/// 实例表第 0 片（根记录直接持有的那一片，32 KiB）头一条位置条目那一份坏、另一份好：可写挂载照常做成，挂载时经数「只坏一份」的那一层把这一片读两遍
/// （建上一版一遍、读实例表链一遍），同一份坏副本只计一次（攻方「damaged=instance_table_first_page_one_copy」那一形，修之前计 2）。
/// 判别力：套着数的读者不按位置条目去重（`crates/mutations.tsv` 一行）时这一条红。
#[test]
fn a_writable_mount_counts_one_damaged_copy_of_the_instance_table_once() {
    let mut pool = pool_of_three_units();
    let location = pool.version.root.instance_table.locations[0];
    damage_the_copy_at(&mut pool.devices, &location, data_unit_bytes());
    assert_eq!(
        unit_reads_with_one_copy_failing_while_mounting(&mut pool),
        1,
        "同一份坏副本一次挂载只计一次"
    );
}

/// 整理那一批把第 1 个数据单元搬走之后：池、搬之前那条数据指针（它的两条位置条目就是盘上 extent 叶里的提示，指旧落点）、映射里的新落点。
struct PoolWithTheDamagedUnitRelocated {
    pool: PoolOfThreeUnits,
    pointer_before_the_relocation: DataPointer,
    destination: [LocationEntry; 2],
}

/// 整理那一批（`transaction::publish_relocation_batch`）把第 1 个数据单元搬走：交回搬之前那条数据指针（旧落点）与映射里的新落点。
/// 内存里交回的那一版记的是映射位置（`transaction::take_the_mapping_positions_over_the_location_hints`，判决 G3「写路径改成以映射为准」），
/// 这一批不重写 extent 树，盘上 extent 叶里的提示还是旧落点（同一个函数的文档「盘上没重写的父节点里仍是旧提示」）。
fn relocate_the_damaged_unit(pool: &mut PoolOfThreeUnits) -> (DataPointer, [LocationEntry; 2]) {
    let pointer_before_the_relocation = pointer_of_the_damaged_unit(&pool.version);
    let mapping_key = mapping_key_of_the_damaged_unit(&pool.version);
    let MappingLookup::Found(source) =
        central_mapping_locations_in_the_version(&pool.version, &mapping_key)
    else {
        panic!("第 1 个数据单元在映射里查得到");
    };
    let planned = PlannedRelocation {
        mapping_key: mapping_key.clone(),
        source: Placement {
            slot: source[0].slot,
            span: TransactionUnit::Data(DAMAGED_UNIT).span_slots(),
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
        pointer_before_the_relocation.locations, source,
        "前提：搬之前指针里的位置条目就是映射里的源落点"
    );
    assert_eq!(
        relocated.extent_tree, pool.version.extent_tree,
        "前提：这一批不重写 extent 树（各节点指针搬前搬后相同），盘上 extent 叶里这个单元的提示还是旧落点"
    );
    pool.version = relocated;
    (pointer_before_the_relocation, destination)
}

/// 搬走第 1 个数据单元之后，旧槽在头一条位置条目那块盘上的那一份被盖掉（一次没发布成、只落了一块盘的写），旧槽另一份与映射里的新落点两份都好。
fn pool_whose_relocated_unit_lost_one_copy_at_its_old_slot() -> PoolWithTheDamagedUnitRelocated {
    let mut pool = pool_of_three_units();
    let (pointer_before_the_relocation, destination) = relocate_the_damaged_unit(&mut pool);
    damage_the_copy_at(
        &mut pool.devices,
        &pointer_before_the_relocation.locations[0],
        data_unit_bytes(),
    );
    assert!(
        copy_at_the_location_self_verifies(
            &pool.devices,
            &pointer_before_the_relocation.locations[1]
        ),
        "前提：旧槽另一份还好（按提示读到的是只坏一份）"
    );
    assert!(
        destination
            .iter()
            .all(|location| copy_at_the_location_self_verifies(&pool.devices, location)),
        "前提：映射里的新落点两份都好"
    );
    PoolWithTheDamagedUnitRelocated {
        pool,
        pointer_before_the_relocation,
        destination,
    }
}

/// 只读挂载、打开第一个文件、从 0 读到 size：交回读出的字节与打开之后累计的只坏一份次数。
fn read_only_whole_file_and_its_count(devices: &Devices) -> (Vec<u8>, u64) {
    let image = memory_pool_of(devices);
    let read_only = mount_read_only(&image).expect("只读挂载");
    let file = read_only
        .mounted
        .open_file(&image, FIRST_FILE)
        .expect("打开文件");
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

/// 提示过期、旧槽一份被盖掉：只读挂载 `read_at` 按提示读到只坏一份，而提示不是这个单元此刻在映射里的落点，读出原内容、计 0
/// （攻方「relocated old_slot=one_copy_overwritten」那一形，修之前计 1）。判别力：挂载态读不判提示是不是映射里的落点（`crates/mutations.tsv` 一行）时这一条红。
#[test]
fn a_read_only_mount_reading_through_a_stale_hint_whose_old_slot_lost_one_copy_counts_nothing() {
    let relocated = pool_whose_relocated_unit_lost_one_copy_at_its_old_slot();
    let (bytes, read_only_count) = read_only_whole_file_and_its_count(&relocated.pool.devices);
    assert_eq!(bytes, content_of_three_units(), "读出原内容");
    assert_eq!(read_only_count, 0, "旧槽上坏的那一份不归这个单元计");
}

/// 同一形：可写挂载建上一版按提示读到只坏一份，提示不是映射里的落点，挂载时计 0、也不记下要搬的 key。
/// 判别力：建上一版不判提示是不是映射里的落点（`crates/mutations.tsv` 一行）时这一条红。
#[test]
fn a_writable_mount_building_through_a_stale_hint_whose_old_slot_lost_one_copy_counts_nothing() {
    let mut relocated = pool_whose_relocated_unit_lost_one_copy_at_its_old_slot();
    let session = session_of_a_writable_mount(&mut relocated.pool);
    assert_eq!(
        session.unit_reads_with_one_copy_failing_while_mounting(),
        0,
        "旧槽上坏的那一份不归这个单元计"
    );
    assert!(
        session
            .mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting_not_yet_relocated()
            .is_empty(),
        "没有要搬的单元"
    );
}

/// 同一形：部分写读回第 1 个单元（`transaction::read_back_the_old_data_unit_telling_whether_a_copy_failed`，部分写、截断、变长读回旧单元都走它），
/// 上一版里这条数据指针还是盘上 extent 叶里那条提示（指旧落点），而上一版映射里是新落点：按提示读到只坏一份，提示不是映射里的落点，
/// 读出原内容、记成没有坏的一份（写入口因此计 0）。发布路径交回的一版在内存里记的已是映射位置（`take_the_mapping_positions_over_the_location_hints`，
/// 整理那一批与从盘上重建一版都调），整份部分写接在它后面读的是新落点、走不到这一判，所以这里把搬之前那条指针放回搬之后那一版，直接调读回那一步。
/// 判别力：读回不判提示是不是映射里的落点（`crates/mutations.tsv` 一行）时这一条红。
#[test]
fn a_partial_write_reading_back_through_a_stale_hint_whose_old_slot_lost_one_copy_counts_nothing() {
    let relocated = pool_whose_relocated_unit_lost_one_copy_at_its_old_slot();
    let mut version_holding_the_stale_hint = relocated.pool.version.clone();
    version_holding_the_stale_hint.data_pointers
        [usize::try_from(DAMAGED_UNIT.0).expect("单元序号")] =
        relocated.pointer_before_the_relocation;
    let MappingLookup::Found(mapped_locations) = central_mapping_locations_in_the_version(
        &version_holding_the_stale_hint,
        &mapping_key_of_the_damaged_unit(&version_holding_the_stale_hint),
    ) else {
        panic!("第 1 个数据单元在映射里查得到");
    };
    assert_eq!(
        mapped_locations, relocated.destination,
        "前提：映射里是新落点，指针里的提示是旧落点"
    );
    let record_before = version_holding_the_stale_hint
        .inode_record
        .expect("这一版有第一个文件那条记录");
    let read_back = read_back_the_old_data_unit_telling_whether_a_copy_failed(
        &memory_pool_of(&relocated.pool.devices),
        &version_holding_the_stale_hint,
        FIRST_FILE,
        record_before,
        DAMAGED_UNIT,
        &parameters().filesystem_identifier,
    )
    .expect("按提示读得回第 1 个单元");
    assert_eq!(
        read_back.pointer.locations, relocated.pointer_before_the_relocation.locations,
        "前提：读回按的是指旧落点的那条提示"
    );
    assert_eq!(
        read_back.copies,
        CopiesOfAUnitRead::WithoutAFailingCopy,
        "旧槽上坏的那一份不归这个单元计"
    );
    let unit_start = usize::try_from(payload_capacity()).expect("净荷");
    assert_eq!(
        read_back.payload,
        content_of_three_units()[unit_start..2 * unit_start].to_vec(),
        "读出原内容"
    );
}

/// 只读挂载一次 `read_at` 从 0 读到 size：第 1 个单元头一份坏（只坏一份），第 2 个单元两份都坏（整次读报错）。报错之前读到的只坏一份照样进打开之后的累计：
/// 报错那一次之后计 1；再单读第 1 个单元又计一次、到 2（攻方那一形，修之前报错之后是 0）。挂载与打开在没弄坏的盘上做（挂载重放 journal 时核点名单元），
/// 之后盘上坏掉：`read_at` 每次读的是交进来的读者。判别力：`read_at` 报错时不把这一次的观测并进累计（`crates/mutations.tsv` 一行）时这一条红。
#[test]
fn a_read_failing_on_a_later_unit_keeps_the_one_copy_failing_count_of_an_earlier_unit() {
    let mut pool = pool_of_three_units();
    let image_at_the_mount = memory_pool_of(&pool.devices);
    let read_only = mount_read_only(&image_at_the_mount).expect("只读挂载");
    let file = read_only
        .mounted
        .open_file(&image_at_the_mount, FIRST_FILE)
        .expect("打开文件");
    let damaged = pointer_of_the_damaged_unit(&pool.version);
    damage_the_copy_at(&mut pool.devices, &damaged.locations[0], data_unit_bytes());
    let last = pool.version.data_pointers[2];
    damage_the_copy_at(&mut pool.devices, &last.locations[0], data_unit_bytes());
    damage_the_copy_at(&mut pool.devices, &last.locations[1], data_unit_bytes());
    let image = memory_pool_of(&pool.devices);
    let size = file.inode_record().size;
    assert!(
        file.read_at(&image, FileOffsetInBytes(0), size).is_err(),
        "第 2 个单元两份都坏：整次读报错"
    );
    assert_eq!(
        read_only
            .mounted
            .observation_since_open()
            .data_unit_reads_with_one_copy_failing,
        1,
        "报错那一次里第 1 个单元的只坏一份进了累计"
    );
    let unit_one = file
        .read_at(
            &image,
            FileOffsetInBytes(payload_capacity()),
            payload_capacity(),
        )
        .expect("单读第 1 个单元读得出");
    assert_eq!(
        unit_one.observation.data_unit_reads_with_one_copy_failing, 1,
        "单读那一次计一次"
    );
    assert_eq!(
        read_only
            .mounted
            .observation_since_open()
            .data_unit_reads_with_one_copy_failing,
        2,
        "累计再加一次"
    );
}

/// 可写挂载建上一版时读第 1 个单元只坏一份：挂载记下它的映射 key；下一次用户改动（建一个空 inode，不碰这个单元）发成之后，会话照快照下另发那批
/// 同一条路径另发一批，把它搬到新槽、两份都与映射里的新位置条目相符，旧落点两块盘上那两份都照释放之前读盘核隔离；记着的 key 取走、清空；
/// 文件读出原内容。判别力：挂载不记下要搬的 key、或另发那一批不并进挂载时读到的那几个（`crates/mutations.tsv` 两行）时这一条红。
#[test]
fn a_unit_read_with_one_copy_damaged_while_mounting_is_relocated_after_the_next_user_change() {
    let mut pool = pool_of_three_units();
    let old_pointer = pointer_of_the_damaged_unit(&pool.version);
    let key_of_the_damaged_unit = mapping_key_of_the_damaged_unit(&pool.version);
    damage_the_copy_at_the_first_location_of(&mut pool.devices, &old_pointer);
    let mut session = session_of_a_writable_mount(&mut pool);
    assert_eq!(
        session
            .mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting_not_yet_relocated(
            ),
        std::slice::from_ref(&key_of_the_damaged_unit),
        "挂载交回之前记下读到只坏一份的那个单元"
    );
    let published = session
        .publish_user_change(
            &mut pool.devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            },
        )
        .expect("建一个空 inode");
    let (mapping_keys, floor_raises) = relocated_keys_and_floor_raises(&published);
    assert_eq!(
        mapping_keys,
        std::slice::from_ref(&key_of_the_damaged_unit),
        "另发的那一批搬的就是挂载时读到只坏一份的那个单元"
    );
    assert!(floor_raises.floor_raises.is_empty(), "那一批一次都没被拒");
    assert!(
        session
            .mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting_not_yet_relocated()
            .is_empty(),
        "交给那一批之后清空"
    );
    let after = current_version_of(&session);
    let MappingLookup::Found(new_locations) =
        central_mapping_locations_in_the_version(&after, &key_of_the_damaged_unit)
    else {
        panic!("搬之后映射里查得到那个单元");
    };
    for (new_location, old_location) in new_locations.iter().zip(&old_pointer.locations) {
        assert_ne!(
            new_location.slot, old_location.slot,
            "映射里的落点换到新槽（盘 {:?}）",
            new_location.device
        );
        assert!(
            copy_at_the_location_self_verifies(&pool.devices, new_location),
            "新槽上那一份新写、与映射里的位置条目相符（盘 {:?}）",
            new_location.device
        );
    }
    let mut quarantined: Vec<(DeviceIdentity, u64)> = after
        .quarantined_after_release_checksum_mismatch
        .iter()
        .filter(|copy| matches!(copy.unit, TransactionUnit::Data(_)))
        .map(|copy| (copy.device, copy.placement.slot.0))
        .collect();
    quarantined.sort_unstable();
    let mut expected: Vec<(DeviceIdentity, u64)> = old_pointer
        .locations
        .iter()
        .map(|location| (location.device, location.slot.0))
        .collect();
    expected.sort_unstable();
    assert_eq!(
        quarantined, expected,
        "旧落点两块盘上那两份都隔离（一份核出对不上，两份一起留）"
    );
    assert_eq!(
        whole_first_file_read_by_the_mounted_reader(&memory_pool_of(&pool.devices), &after),
        content_of_three_units(),
        "文件读出原内容"
    );
}

/// 推抬 F 重发那一形之前经会话建几次空 inode：每次换下的固定点单元进 defer，等推的抬 F 回收（照
/// `compaction_off_the_calling_thread_and_admission.rs`「一次用户改动被准入拒、推一串抬 F、回收之后再判发成」那一条的造法，它建 12 次）。
const NEW_INODE_CHANGES_BEFORE_THE_PARTIAL_WRITE_REFUSED_FOR_SPACE: u64 = 12;

/// 那次部分写的写入时间：接在建空 inode 那几次之后。
const WRITE_TIME_OF_THE_PARTIAL_WRITE_REFUSED_FOR_SPACE: u64 =
    FIXED_WRITE_TIME_SECONDS + 3 + NEW_INODE_CHANGES_BEFORE_THE_PARTIAL_WRITE_REFUSED_FOR_SPACE;

/// 第一个文件三个单元的池可写挂载、经会话建 [`NEW_INODE_CHANGES_BEFORE_THE_PARTIAL_WRITE_REFUSED_FOR_SPACE`] 次空 inode，再把现行那一版第 1 个单元
/// 头一条位置条目那一份弄坏（挂载之后才弄坏：挂载时读不到它，不记下挂载之后搬它）。
fn session_after_new_inode_changes_with_the_unit_damaged(
    pool: &mut PoolOfThreeUnits,
) -> MountedSession {
    let mut session = session_of_a_writable_mount(pool);
    for creation in 0..NEW_INODE_CHANGES_BEFORE_THE_PARTIAL_WRITE_REFUSED_FOR_SPACE {
        session
            .publish_user_change(
                &mut pool.devices,
                UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3 + creation,
                },
            )
            .expect("建一个空 inode");
    }
    let pointer = pointer_of_the_damaged_unit(&current_version_of(&session));
    damage_the_copy_at_the_first_location_of(&mut pool.devices, &pointer);
    session
}

/// 准入读数里每块盘的可用(d)（字节），按分配器里盘的次序：会话现行那一版之上、下一次发布之前的那一份。
fn available_bytes_on_each_device(session: &MountedSession) -> Vec<(DeviceIdentity, i128)> {
    admission_reading_before_a_publish(&session.allocator, session.current.file_version())
        .available_on_each_device()
        .into_iter()
        .map(|(device, available)| (device, available.0))
        .collect()
}

/// 在每块盘单元区的最高处隔离槽（只动分配器的隔离位，D28（挂载期承诺量） 已定项 1 第九项，不写盘），让每块盘的可用(d) 落进
/// [`lower_bound_in_bytes`, `lower_bound_in_bytes` + 16384)（照 `compaction_off_the_calling_thread_and_admission.rs` 同名的那一个）。
fn isolate_the_highest_slots_until_the_available_is_within_one_slot_above(
    session: &mut MountedSession,
    lower_bound_in_bytes: i128,
) {
    let slot_bytes = i128::from(SLOT_BYTES);
    for (device, available) in available_bytes_on_each_device(session) {
        let excess_in_bytes = available - lower_bound_in_bytes;
        assert!(
            excess_in_bytes >= 0,
            "{device:?}：可用 {available} 本来就低于要压到的 {lower_bound_in_bytes}"
        );
        let slots_to_isolate =
            u64::try_from(excess_in_bytes / slot_bytes).expect("非负、装得进 u64");
        if slots_to_isolate == 0 {
            continue;
        }
        let device_map = session
            .allocator
            .devices
            .iter()
            .find(|device_map| device_map.device == device)
            .expect("读数里的盘就是分配器的盘");
        let unit_area_end = device_map.unit_area_start().slot().0 + device_map.unit_area_slots();
        session.allocator.isolate_abandoned(
            device,
            SlotNumber(unit_area_end - slots_to_isolate),
            slots_to_isolate,
        );
    }
    for (device, available) in available_bytes_on_each_device(session) {
        assert!(
            available >= lower_bound_in_bytes && available < lower_bound_in_bytes + slot_bytes,
            "{device:?}：隔离之后可用 {available} 落在 [{lower_bound_in_bytes}, +16384) 里"
        );
    }
}

/// 推抬 F 重发那一形（判决 `research/prompts/m4-close-code-r3-main-verification.md` 第三节第 6 条「每次用户改动只计一次」的端到端）：
/// 先在同样建、同样弄坏的另一个池上量出这次部分写重写的槽数；本池把每块盘的可用(d) 压到它之下一个槽以内，经会话部分写第 1 个单元：
/// 头一次发布读回第 1 个单元（只坏一份）、被准入拒（读回在任何分配与写之前），会话推一串抬 F、回收之后重发同一个改动，又读回同一份坏副本，发成。
/// 这次挂载的累计只加一次；写照成，内容是原内容加这次写的字节。判别力：一次改动里读回过的旧单元重读也再加次数（`crates/mutations.tsv` 一行）时计 2，这一条红。
#[test]
fn a_partial_write_refused_for_space_and_published_after_raising_the_floor_counts_the_unit_read_back_with_one_copy_damaged_once(
) {
    let mut pool_measuring_the_demand = pool_of_three_units();
    let mut session_measuring_the_demand =
        session_after_new_inode_changes_with_the_unit_damaged(&mut pool_measuring_the_demand);
    let unconstrained = write_into_the_damaged_unit_through_the_session(
        &mut session_measuring_the_demand,
        &mut pool_measuring_the_demand,
        WRITE_TIME_OF_THE_PARTIAL_WRITE_REFUSED_FOR_SPACE,
    );
    assert!(
        unconstrained.floor_raises.is_empty() && unconstrained.versions_published.len() == 1,
        "量需求的那个池不压余量：一次就发成"
    );
    let demand_in_slots: u64 = unconstrained.versions_published[0]
        .rewritten
        .iter()
        .map(|role| role.span_slots())
        .sum();

    let mut pool = pool_of_three_units();
    let mut session = session_after_new_inode_changes_with_the_unit_damaged(&mut pool);
    let slot_bytes = i128::from(SLOT_BYTES);
    isolate_the_highest_slots_until_the_available_is_within_one_slot_above(
        &mut session,
        i128::from(demand_in_slots) * slot_bytes - slot_bytes,
    );
    let published = write_into_the_damaged_unit_through_the_session(
        &mut session,
        &mut pool,
        WRITE_TIME_OF_THE_PARTIAL_WRITE_REFUSED_FOR_SPACE,
    );
    assert!(
        !published.refusals_that_pushed_the_floor_raises.is_empty()
            && !published.floor_raises.is_empty(),
        "前提：可用压到需求之下，头一次发布被拒、推了抬 F 再发"
    );
    assert_eq!(
        session.old_data_unit_reads_back_with_one_copy_failing_in_this_mount(),
        1,
        "被拒那一次与重发那一次读回的是同一份坏副本：这次改动只计一次"
    );
    assert_eq!(
        whole_first_file_read_by_the_mounted_reader(
            &memory_pool_of(&pool.devices),
            &current_version_of(&session)
        ),
        content_after_the_partial_write(),
        "写照成"
    );
}

/// 挂载退版那一形之前在会话里覆盖写几次（照 `fall_back_judged_on_the_union_of_the_three_readers_and_only_for_lost_bytes.rs` 的盘面：
/// 384 槽小盘上第一个文件之后覆盖写 5 次是 P，再录一次覆盖写 R）。
const OVERWRITES_BEFORE_THE_UNWITNESSED_PUBLISH: u64 = 5;

/// 挂载退版那一形（判决 `research/prompts/m4-close-code-r3-main-verification.md` 第三节第 6 条「挂载退版」的端到端）：R 崩在它的根槽之后、
/// 系统配置轮换之前（系统配置没见证 R）。这次可写挂载里 R 那一个数据单元两份读回全零（冷走读字节丢了，判退到 P）、P 的树表两份读回全零
/// （P 建不出来，留在 R、照建 R）、P 与 R 共用的实例表第 0 片头一条位置条目那一份读回全零（只坏一份）。建 P 那一趟先读实例表（读到那一份坏）、
/// 再读树表建不出，建 R 那一趟又读到同一份：同一份坏副本两趟各读一次，挂载落在 R、挂载时只计一次。
/// 判别力：套着数的读者遇到计过的位置条目照样再计（`crates/mutations.tsv` 一行）时这一条红。
#[test]
fn a_writable_mount_falling_back_to_a_version_it_cannot_build_counts_the_damaged_copy_read_by_both_builds_once(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_UNWITNESSED_PUBLISH,
        PublishRecordedAsR::Overwrite,
    );
    let root_of_r = recorded.root();
    let root_of_p = mount_read_only(&recorded.base)
        .expect("R 之前那一刻的盘面只读挂得上")
        .effective_root;
    assert_eq!(
        (root_of_p.instance, root_of_p.checkpoint_txg),
        recorded.previous_key,
        "前提：R 之前那一刻择到的是 P"
    );
    assert_eq!(
        root_of_p.instance_table.locations, root_of_r.instance_table.locations,
        "前提：P 与 R 共用实例表（同一次挂载里的覆盖写不重写它）"
    );
    assert_ne!(
        root_of_p.tree_table.locations, root_of_r.tree_table.locations,
        "前提：R 重写了树表，P 的树表只归 P"
    );
    let data_unit_groups: Vec<_> = recorded
        .unit_writes_grouped_by_unit()
        .into_iter()
        .filter(|copies| parse_data_unit(copies[0].bytes().expect("单元写带字节")).is_ok())
        .collect();
    assert_eq!(
        data_unit_groups.len(),
        1,
        "前提：覆盖写 R 只写了一个数据单元"
    );
    let mut ranges = RangesReadingBackAllZero::default().with_the_writes(&data_unit_groups[0]);
    for location in &root_of_p.tree_table.locations {
        ranges.0.push((
            location.device,
            location.slot.to_device_offset().0,
            SLOT_BYTES,
        ));
    }
    let damaged_copy_of_the_instance_table = root_of_r.instance_table.locations[0];
    ranges.0.push((
        damaged_copy_of_the_instance_table.device,
        damaged_copy_of_the_instance_table.slot.to_device_offset().0,
        SLOT_BYTES,
    ));
    let image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut devices = devices_reading_back_all_zero_on(&image, &ranges);
    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect("P 建不出来、留在 R：可写挂载照建 R 做成");
    let (output, session) = MountedSession::of_the_mount(mounted);
    assert_eq!(
        (
            output.effective_root.instance,
            output.effective_root.checkpoint_txg
        ),
        recorded.key,
        "前提：判退到 P 而 P 建不出来，挂载落在 R"
    );
    assert_eq!(
        session.unit_reads_with_one_copy_failing_while_mounting(),
        1,
        "同一份坏副本建 P、建 R 两趟各读一次：挂载时只计一次"
    );
}
