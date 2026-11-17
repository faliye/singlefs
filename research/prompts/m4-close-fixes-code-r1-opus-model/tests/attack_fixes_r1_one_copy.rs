//! m4-close-fixes-code-r1 攻方模型：把只坏一份那几批修补摆到 D4（校验和位置） 已定项 4 的字面旁边。
//! 每条用例断言的是今天的行为；观测行以 `attack-fixes-r1:` 起头。盘是内存稀疏盘，每个起点状态只建一次池，之后拷字节。
#![allow(dead_code, unused_imports)]

mod common;

use std::num::NonZeroU64;
use std::sync::Arc;
use std::time::Instant;

use common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    DataUnitIndexInFile, DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration,
    SlotNumber,
};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::code_two_tree::CodeTwoTreeNodeContents;
use singlefs_core::compaction_plan::{
    RelocationKeptForTheSnapshotsRefusedForSpace, RelocationKeptForTheSnapshotsRefusedNotForSpace,
};
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::mounted_read::open_pool_for_read;
use singlefs_core::mounted_session::{
    MountedSession, RelocationOfTheOldDataUnitsKeptForTheSnapshots, UserChange,
};
use singlefs_core::pointer::{LocationEntry, NodePointer};
use singlefs_core::records::TREE_KIND_SNAPSHOT_INODE_ROOT;
use singlefs_core::recovery::{
    read_the_first_file_of_a_snapshot, PoolReader, PoolReaderCountingUnitReadsWithOneCopyFailing,
    SnapshotFileReading,
};
use singlefs_core::transaction::{
    acquire_instance, central_mapping_locations_in_the_version,
    mapping_keys_of_the_units_at_these_locations, publish_first_file, publish_partial_write,
    publish_sequential_write, read_back_the_old_data_unit_telling_whether_a_copy_failed, warm_up,
    CopiesOfAUnitRead, FirstFile, MappingLookup, PartialWriteBytes, PoolWriter, TransactionOutput,
    TransactionUnit,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SLOT_BYTES;
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

const FIRST_FILE: InodeNumber = InodeNumber(1);
const DAMAGED_UNIT: DataUnitIndexInFile = DataUnitIndexInFile(1);
const FLIPPED_BYTE_IN_THE_UNIT: usize = 4000;
const BYTES_OF_A_NODE: usize = 16384;
const BYTES_OF_A_DATA_UNIT: usize = 32768;

fn payload_capacity() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634")
}

/// `units` 个单元的内容，最后一个不满。
fn content_of_units(units: u64) -> Vec<u8> {
    let length = usize::try_from(units * payload_capacity() - 1000).expect("装得进 usize");
    (0..length)
        .map(|index| u8::try_from((index * 37 + 11) % 253).expect("小于 256"))
        .collect()
}

struct Pool {
    devices: Devices,
    allocator: PoolAllocator,
    version: TransactionOutput,
    instance: InstanceGeneration,
}

/// 两块 4 GiB 内存盘上 mkfs → 取号 → 暖机 → 新池新建文件 → 第一个文件整份顺序写成 `content`。
fn pool_holding(content: &[u8]) -> Pool {
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
    let first_content = file_content();
    let (version, instance) = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &first_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件");
        let written = publish_sequential_write(
            &mut writer,
            &mut allocator,
            &first,
            FirstFile {
                content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            },
            instance,
        )
        .expect("顺序写");
        (written, instance)
    };
    Pool {
        devices,
        allocator,
        version,
        instance,
    }
}

fn devices_holding_the_same_bytes(devices: &Devices) -> Devices {
    devices
        .iter()
        .map(|(identity, device)| {
            let mut copy = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect()
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

/// 把 `location` 那块盘上那一份（从它的槽起 `unit_bytes` 字节）改一个字节，另一份不动。
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

fn copy_self_verifies(devices: &Devices, location: &LocationEntry, unit_bytes: usize) -> bool {
    let mut unit = vec![0u8; unit_bytes];
    devices
        .iter()
        .find(|(identity, _)| *identity == location.device)
        .expect("池里有这块盘")
        .1
        .read_at(location.slot.to_device_offset(), &mut unit)
        .expect("读得出那一份");
    crc32_castagnoli(&unit) == location.unit_checksum
}

fn session_of_a_writable_mount(devices: &mut Devices) -> MountedSession {
    let mounted = mount_writable(&parameters(), devices).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    session
}

fn current_version_of(session: &MountedSession) -> TransactionOutput {
    session
        .current
        .file_version()
        .expect("现行那一版带文件")
        .clone()
}

fn new_inode_change(write_time_offset: u64) -> UserChange<'static> {
    UserChange::NewInodes {
        count: 1,
        write_time_seconds: FIXED_WRITE_TIME_SECONDS + write_time_offset,
    }
}

fn outcome_name(outcome: &RelocationOfTheOldDataUnitsKeptForTheSnapshots) -> &'static str {
    match outcome {
        RelocationOfTheOldDataUnitsKeptForTheSnapshots::NotNeeded => "not_needed",
        RelocationOfTheOldDataUnitsKeptForTheSnapshots::Relocated { .. } => "relocated",
        RelocationOfTheOldDataUnitsKeptForTheSnapshots::NoSpaceAfterRaisingTheFloor { .. } => {
            "no_space_after_raising_the_floor"
        }
        RelocationOfTheOldDataUnitsKeptForTheSnapshots::RefusedNotForSpace { .. } => {
            "refused_not_for_space"
        }
        RelocationOfTheOldDataUnitsKeptForTheSnapshots::FloorRaiseFailedWhilePushingForSpace {
            ..
        } => "floor_raise_failed",
    }
}

fn mapping_key_of_role(version: &TransactionOutput, role: TransactionUnit) -> Vec<u8> {
    version
        .mapped_units
        .iter()
        .find(|(identity, _)| *identity == role)
        .expect("这个角色进映射")
        .1
        .clone()
}

fn mapped_locations(version: &TransactionOutput, key: &[u8]) -> Option<[LocationEntry; 2]> {
    match central_mapping_locations_in_the_version(version, key) {
        MappingLookup::Found(locations) => Some(locations),
        MappingLookup::NodeMalformedOrNoEntryWithThisKey
        | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => None,
    }
}

fn whole_first_file(image: &MemoryPool, version: &TransactionOutput) -> Vec<u8> {
    let mounted = open_pool_for_read(image, &version.root).expect("打开挂载态");
    let file = mounted.open_file(image, FIRST_FILE).expect("打开文件");
    let size = file.inode_record().size;
    file.read_at(image, FileOffsetInBytes(0), size)
        .expect("从 0 读到 size")
        .bytes
}

/// 池级 checker 判红的不变量名。
fn violated_invariants(image: &MemoryPool) -> Vec<&'static str> {
    check_pool_image(image)
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| name)
        .collect()
}

/// D4（校验和位置） 已定项 4 射程「豁免映射的三类（映射树节点、树表、实例表片）……它们在挂载这一串的发布里本来就换落点」：
/// 映射树多层时，挂载与头一次用户改动都不碰的那几片叶不换落点。先在没弄坏的一份上走一遍找出这样的叶，再在另一份上弄坏它一份：
/// 挂载计 1、认不出 key；头一次改动之后落点不变、坏的那一份还在；下一次挂载又计 1。
#[test]
fn a_mapping_tree_leaf_with_one_copy_damaged_stays_where_it_is_after_the_mount_and_the_first_change()
{
    let pool = pool_holding(&content_of_units(700));
    let before = &pool.version.central_mapping_tree;
    assert!(
        before.shape.nodes().len() > 1,
        "前提：映射树多层，节点数 {}",
        before.shape.nodes().len()
    );
    let mut rehearsal = devices_holding_the_same_bytes(&pool.devices);
    let mut rehearsal_session = session_of_a_writable_mount(&mut rehearsal);
    rehearsal_session
        .publish_user_change(&mut rehearsal, new_inode_change(3))
        .expect("没弄坏的那一份上建一个空 inode");
    let rehearsal_after = current_version_of(&rehearsal_session);
    let mut leaves_untouched: Vec<NodePointer> = Vec::new();
    let mut leaves_in_total = 0usize;
    for (node, pointer) in before.shape.nodes().iter().zip(&before.pointers) {
        if !matches!(node.contents, CodeTwoTreeNodeContents::Leaf { .. }) {
            continue;
        }
        leaves_in_total += 1;
        if rehearsal_after
            .central_mapping_tree
            .shape
            .nodes()
            .iter()
            .zip(&rehearsal_after.central_mapping_tree.pointers)
            .any(|(node_after, pointer_after)| {
                node_after.position == node.position && pointer_after == pointer
            })
        {
            leaves_untouched.push(*pointer);
        }
    }
    println!(
        "attack-fixes-r1: mapping_tree nodes={} leaves={} leaves_untouched_by_the_mount_and_the_first_change={}",
        before.shape.nodes().len(),
        leaves_in_total,
        leaves_untouched.len()
    );
    let leaf = *leaves_untouched
        .first()
        .expect("有一片叶挂载与头一次改动都不碰");
    let mut devices = devices_holding_the_same_bytes(&pool.devices);
    damage_the_copy_at(&mut devices, &leaf.locations[0], BYTES_OF_A_NODE);
    let mut session = session_of_a_writable_mount(&mut devices);
    let count_at_the_mount = session.unit_reads_with_one_copy_failing_while_mounting();
    let keys_at_the_mount = session
        .mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting_not_yet_relocated()
        .len();
    let published = session
        .publish_user_change(&mut devices, new_inode_change(3))
        .expect("建一个空 inode");
    let outcome = outcome_name(&published.relocation_of_the_old_data_units_kept_for_the_snapshots);
    let after = current_version_of(&session);
    let still_there = after
        .central_mapping_tree
        .pointers
        .iter()
        .any(|pointer| *pointer == leaf);
    let damaged_copy_verifies = copy_self_verifies(&devices, &leaf.locations[0], BYTES_OF_A_NODE);
    let count_at_the_next_mount =
        session_of_a_writable_mount(&mut devices).unit_reads_with_one_copy_failing_while_mounting();
    println!(
        "attack-fixes-r1: damaged=mapping_tree_leaf count_at_the_mount={count_at_the_mount} keys_at_the_mount={keys_at_the_mount} first_change_outcome={outcome} leaf_at_the_same_location_after_the_first_change={still_there} damaged_copy_self_verifies={damaged_copy_verifies} count_at_the_next_mount={count_at_the_next_mount}"
    );
    assert_eq!(count_at_the_mount, 1, "挂载读到只坏一份、计一次");
    assert_eq!(keys_at_the_mount, 0, "映射树节点认不出映射 key");
    assert!(still_there, "头一次改动之后这片叶落点没变");
    assert!(!damaged_copy_verifies, "坏的那一份还在现行那一版指着的落点上");
    assert_eq!(count_at_the_next_mount, 1, "下一次挂载同一份坏副本又计一次");
}

/// 同一句「旧落点走释放之前读盘核那一路」：树表、实例表第 0 片只坏一份，挂载的发布把它们换了落点，而释放之前读盘核不判豁免映射的角色
/// （`transaction::mapped_roles_released_by_this_publish`），这次挂载发的每一版的隔离清单里都没有它们。对照：数据单元经部分写释放时两份都隔离（入库用例已钉）。
#[test]
fn the_exempt_roles_replaced_by_the_mount_are_released_without_the_release_check() {
    let pool = pool_holding(&content_of_units(3));
    let root = pool.version.root;
    for (name, location, unit_bytes) in [
        ("tree_table", root.tree_table.locations[0], BYTES_OF_A_NODE),
        (
            "instance_table",
            root.instance_table.locations[0],
            BYTES_OF_A_DATA_UNIT,
        ),
    ] {
        let mut devices = devices_holding_the_same_bytes(&pool.devices);
        damage_the_copy_at(&mut devices, &location, unit_bytes);
        let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
        let mut versions: Vec<&TransactionOutput> = Vec::new();
        versions.extend(mounted.output.row_publish.version_of_trees());
        versions.extend(
            mounted
                .output
                .warm_up_publishes
                .iter()
                .filter_map(|version| version.version_of_trees()),
        );
        versions.extend(
            mounted
                .output
                .deletion_queue_continuation
                .iter()
                .map(|version| &**version),
        );
        versions.extend(mounted.current.version_of_trees());
        let exempt_copies_quarantined = versions
            .iter()
            .flat_map(|version| &version.quarantined_after_release_checksum_mismatch)
            .filter(|copy| {
                matches!(
                    copy.unit,
                    TransactionUnit::TreeTable
                        | TransactionUnit::InstanceTable
                        | TransactionUnit::InstanceTablePageAfterTheFirst(_)
                )
            })
            .count();
        let root_after = mounted.current.root();
        let moved = match name {
            "tree_table" => root_after.tree_table.locations[0].slot != location.slot,
            _ => root_after.instance_table.locations[0].slot != location.slot,
        };
        let versions_looked_at = versions.len();
        let (_, session) = MountedSession::of_the_mount(mounted);
        println!(
            "attack-fixes-r1: damaged={name} count_at_the_mount={} moved_by_the_mount={moved} versions_looked_at={versions_looked_at} exempt_copies_quarantined={exempt_copies_quarantined}",
            session.unit_reads_with_one_copy_failing_while_mounting()
        );
        assert!(moved, "{name}：挂载的发布换了它的落点");
        assert_eq!(exempt_copies_quarantined, 0, "{name}：旧落点没走释放之前读盘核");
    }
}

/// 已定项 4「可写挂载期间别的读」：今天会话没有读文件的入口，可写挂载期间读文件走 `open_pool_for_read`、`read_at`（只读挂载态读者）。
/// 挂载之后才坏的一份：读者自己的观测计 1，会话两个计数都是 0、没有要搬的 key，下一次用户改动不搬，坏的那一份还在映射落点上。
#[test]
fn a_file_read_during_the_writable_mount_is_counted_nowhere_in_the_session_and_not_relocated() {
    let pool = pool_holding(&content_of_units(3));
    let mut devices = devices_holding_the_same_bytes(&pool.devices);
    let mut session = session_of_a_writable_mount(&mut devices);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    damage_the_copy_at(&mut devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    let image = memory_pool_of(&devices);
    let mounted_for_read = open_pool_for_read(&image, &version.root).expect("打开挂载态");
    let file = mounted_for_read
        .open_file(&image, FIRST_FILE)
        .expect("打开文件");
    let size = file.inode_record().size;
    let output = file
        .read_at(&image, FileOffsetInBytes(0), size)
        .expect("读得出");
    let count_of_the_reader = output.observation.data_unit_reads_with_one_copy_failing;
    let keys_in_the_session = session
        .mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting_not_yet_relocated()
        .len();
    let counts_in_the_session = (
        session.unit_reads_with_one_copy_failing_while_mounting(),
        session.old_data_unit_reads_back_with_one_copy_failing_in_this_mount(),
    );
    let published = session
        .publish_user_change(&mut devices, new_inode_change(3))
        .expect("建一个空 inode");
    let outcome = outcome_name(&published.relocation_of_the_old_data_units_kept_for_the_snapshots);
    let after = current_version_of(&session);
    let locations_after = mapped_locations(&after, &key).expect("还在映射里");
    println!(
        "attack-fixes-r1: read_during_the_session count_of_the_reader={count_of_the_reader} counts_in_the_session={counts_in_the_session:?} keys_in_the_session={keys_in_the_session} next_change_outcome={outcome} same_location={} damaged_copy_self_verifies={}",
        locations_after == locations,
        copy_self_verifies(&devices, &locations[0], BYTES_OF_A_DATA_UNIT)
    );
    assert_eq!(count_of_the_reader, 1);
    assert_eq!(counts_in_the_session, (0, 0));
    assert_eq!(keys_in_the_session, 0);
    assert_eq!(outcome, "not_needed");
    assert_eq!(locations_after, locations);
    assert!(!copy_self_verifies(
        &devices,
        &locations[0],
        BYTES_OF_A_DATA_UNIT
    ));
}

/// 同一句的另一处：释放之前读盘核读到只坏一份（整份顺序写重写弄坏的单元）隔离两份，会话两个计数都不加。
#[test]
fn the_release_check_meeting_one_damaged_copy_quarantines_both_and_counts_nothing() {
    let pool = pool_holding(&content_of_units(3));
    let mut devices = devices_holding_the_same_bytes(&pool.devices);
    let mut session = session_of_a_writable_mount(&mut devices);
    let version = current_version_of(&session);
    let locations = mapped_locations(
        &version,
        &mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT)),
    )
    .expect("在映射里");
    damage_the_copy_at(&mut devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    let new_content: Vec<u8> = content_of_units(3).iter().map(|byte| byte ^ 0x5A).collect();
    session
        .publish_user_change(
            &mut devices,
            UserChange::SequentialWrite(FirstFile {
                content: &new_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            }),
        )
        .expect("整份顺序写");
    let quarantined = current_version_of(&session)
        .quarantined_after_release_checksum_mismatch
        .iter()
        .filter(|copy| matches!(copy.unit, TransactionUnit::Data(_)))
        .count();
    let counts = (
        session.unit_reads_with_one_copy_failing_while_mounting(),
        session.old_data_unit_reads_back_with_one_copy_failing_in_this_mount(),
    );
    println!(
        "attack-fixes-r1: release_check_on_a_damaged_unit data_copies_quarantined={quarantined} counts_in_the_session={counts:?}"
    );
    assert!(quarantined >= 2, "读盘核读到了那一份坏副本");
    assert_eq!(counts, (0, 0), "这一读哪个数都不进");
}

/// 同一句的又一处：会话里为空间推抬 F 时沿链读实例表（`mount::raise_the_floor_through` 的 `instance_table_chain_of_root(&*devices, …)`），
/// 读者是盘表本身，只坏一份那一报是默认的什么都不做。有活快照时部分写读回只坏一份的单元、另发那一批头一次被空间拒（只供测试的开关），
/// 那之前挂载之后坏了实例表一份：推一串抬 F 照成，会话的数里只有读回那一次。
/// 对照：两份都坏时抬 F 那一步读不出（读确实发生了）。
#[test]
fn the_instance_table_read_while_raising_the_floor_in_the_session_counts_nothing() {
    let pool = pool_holding(&content_of_units(3));
    for copies_damaged in [1usize, 2] {
        let mut devices = devices_holding_the_same_bytes(&pool.devices);
        let mut session = session_of_a_writable_mount(&mut devices);
        session
            .publish_user_change(&mut devices, UserChange::CreateSnapshot)
            .expect("建快照");
        let version = current_version_of(&session);
        let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
        let data_locations = mapped_locations(&version, &key).expect("在映射里");
        damage_the_copy_at(&mut devices, &data_locations[0], BYTES_OF_A_DATA_UNIT);
        let table = version.root.instance_table.locations;
        for location in table.iter().take(copies_damaged) {
            damage_the_copy_at(&mut devices, location, BYTES_OF_A_DATA_UNIT);
        }
        session
            .compaction
            .force_the_relocation_kept_for_the_snapshots_refused_for_space_for_tests(
                RelocationKeptForTheSnapshotsRefusedForSpace::ForcedForTheNextAttemptsForTests {
                    attempts: NonZeroU64::new(1).expect("拒一次：1 不是 0"),
                },
            );
        let patch = vec![0x5Au8; 64];
        let outcome = match session.publish_user_change(
            &mut devices,
            UserChange::WriteAtOffset {
                inode: FIRST_FILE,
                offset: FileOffsetInBytes(payload_capacity() + 100),
                bytes: PartialWriteBytes::new(&patch).expect("写的字节不是空的"),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            },
        ) {
            Ok(published) => outcome_name(
                &published.relocation_of_the_old_data_units_kept_for_the_snapshots,
            ),
            Err(_refused) => "change_refused",
        };
        let counts = (
            session.unit_reads_with_one_copy_failing_while_mounting(),
            session.old_data_unit_reads_back_with_one_copy_failing_in_this_mount(),
        );
        println!(
            "attack-fixes-r1: instance_table_copies_damaged_in_the_session={copies_damaged} outcome={outcome} counts_in_the_session={counts:?}"
        );
        match copies_damaged {
            1 => {
                assert_eq!(outcome, "relocated");
                assert_eq!(counts, (0, 1), "只有部分写读回那一次进数，实例表那一份不进任何一个数");
            }
            _ => assert_ne!(outcome, "relocated", "两份都坏时那一步读不出：读确实发到了盘上"),
        }
    }
}

/// 第 7 条的两个边：挂载记下的 key 只有用户改动发成才取走（被拒的改动不取）；同一把 key 同时在挂载的账与读回的账里，计划里只出一次。
#[test]
fn the_keys_kept_at_the_mount_survive_a_refused_change_and_are_taken_by_the_next_published_one() {
    let pool = pool_holding(&content_of_units(3));
    let key = mapping_key_of_role(&pool.version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&pool.version, &key).expect("在映射里");
    let mut devices = devices_holding_the_same_bytes(&pool.devices);
    damage_the_copy_at(&mut devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    let mut session = session_of_a_writable_mount(&mut devices);
    let refused = session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: InodeNumber(999_999),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            },
        )
        .is_err();
    let keys_after_the_refusal = session
        .mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting_not_yet_relocated()
        .len();
    let published = session
        .publish_user_change(&mut devices, new_inode_change(4))
        .expect("建一个空 inode");
    let outcome = outcome_name(&published.relocation_of_the_old_data_units_kept_for_the_snapshots);
    println!(
        "attack-fixes-r1: refused_change refused={refused} keys_after_the_refusal={keys_after_the_refusal} next_change_outcome={outcome}"
    );
    assert!(refused);
    assert_eq!(keys_after_the_refusal, 1);
    assert_eq!(outcome, "relocated");
}

#[test]
fn the_same_key_at_the_mount_and_in_the_read_back_under_a_snapshot_is_planned_once() {
    let pool = pool_holding(&content_of_units(3));
    let mut devices = devices_holding_the_same_bytes(&pool.devices);
    let mut first_session = session_of_a_writable_mount(&mut devices);
    first_session
        .publish_user_change(&mut devices, UserChange::CreateSnapshot)
        .expect("建快照");
    let version = current_version_of(&first_session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    damage_the_copy_at(&mut devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    let mut session = session_of_a_writable_mount(&mut devices);
    let patch = vec![0x5Au8; 64];
    let published = session
        .publish_user_change(
            &mut devices,
            UserChange::WriteAtOffset {
                inode: FIRST_FILE,
                offset: FileOffsetInBytes(payload_capacity() + 100),
                bytes: PartialWriteBytes::new(&patch).expect("写的字节不是空的"),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            },
        )
        .expect("部分写");
    let planned = match &published.relocation_of_the_old_data_units_kept_for_the_snapshots {
        RelocationOfTheOldDataUnitsKeptForTheSnapshots::Relocated { mapping_keys, .. } => {
            mapping_keys.clone()
        }
        other => panic!("应搬成，交回 {other:?}"),
    };
    let counts = (
        session.unit_reads_with_one_copy_failing_while_mounting(),
        session.old_data_unit_reads_back_with_one_copy_failing_in_this_mount(),
    );
    println!(
        "attack-fixes-r1: same_key_in_both_accounts planned={} counts_in_the_session={counts:?}",
        planned.len()
    );
    assert_eq!(planned, vec![key]);
    assert_eq!(counts, (1, 1), "同一份坏副本在挂载与读回各记一次");
}

/// 攻击面第 2 条：`hints_are_the_mapping_position` 记 false 的那一支。映射叶里这把 key 查不到（手改内存里那一版的叶字节，形同盘上一把 key 被换掉、条目数不变的镜像）：
/// 按提示读到只坏一份记成没有坏的一份；接着的部分写被拒（发布路径照 `ReleaseNotInMapping` 一类），写入口计 0。对照：同一块盘上不改叶字节记只坏一份。
#[test]
fn the_read_back_with_no_entry_in_the_mapping_drops_the_count_and_the_publish_is_refused_anyway() {
    let mut pool = pool_holding(&content_of_units(3));
    let key = mapping_key_of_role(&pool.version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&pool.version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    let mut crafted = pool.version.clone();
    let mapping_leaf = crafted
        .units
        .iter_mut()
        .find(|unit| unit.identity == TransactionUnit::MappingTree)
        .expect("三个单元的池映射树根兼叶");
    let mut leaf_bytes = mapping_leaf.bytes.to_vec();
    let at = leaf_bytes
        .windows(key.len())
        .position(|window| window == key.as_slice())
        .expect("叶字节里有这把 key");
    leaf_bytes[at + key.len() - 1] ^= 0xFF;
    mapping_leaf.bytes = Arc::from(leaf_bytes);
    assert!(mapped_locations(&crafted, &key).is_none(), "前提：查不到这把 key");
    let image = memory_pool_of(&pool.devices);
    let record_before = crafted.inode_record.expect("第一个文件那条记录");
    let copies_of = |version: &TransactionOutput| {
        read_back_the_old_data_unit_telling_whether_a_copy_failed(
            &image,
            version,
            FIRST_FILE,
            record_before,
            DAMAGED_UNIT,
            &parameters().filesystem_identifier,
        )
        .expect("按提示读得回")
        .copies
    };
    let copies_with_the_entry = copies_of(&pool.version);
    let copies_without_the_entry = copies_of(&crafted);
    let parameters_of_the_pool = parameters();
    let patch = vec![0x5Au8; 64];
    let mut writer = PoolWriter::new(&parameters_of_the_pool, pool.devices.as_mut_slice());
    let refusal = match publish_partial_write(
        &mut writer,
        &mut pool.allocator,
        &crafted,
        FIRST_FILE,
        FileOffsetInBytes(payload_capacity() + 100),
        PartialWriteBytes::new(&patch).expect("写的字节不是空的"),
        FIXED_WRITE_TIME_SECONDS + 2,
        pool.instance,
    ) {
        Ok(_) => "published".to_string(),
        Err(refusal) => format!("{refusal:?}"),
    };
    let count_of_the_writer = writer.old_data_unit_reads_back_with_one_copy_failing();
    println!(
        "attack-fixes-r1: no_entry_in_the_mapping copies_with_the_entry={copies_with_the_entry:?} copies_without_the_entry={copies_without_the_entry:?} count_of_the_writer={count_of_the_writer} publish={refusal}"
    );
    assert_eq!(copies_with_the_entry, CopiesOfAUnitRead::FromTheOtherCopyAfterOneFailed);
    assert_eq!(copies_without_the_entry, CopiesOfAUnitRead::WithoutAFailingCopy);
    assert_eq!(count_of_the_writer, 0);
    assert_ne!(refusal, "published", "这一版接不下去：照映射释放那一步查不到");
}

/// 扫一遍：`version` 里每个进映射的角色各弄坏头一条位置条目那一份，可写挂载 → 建一个空 inode（头一次用户改动，不碰文件内容）。每个角色打一行观测，
/// 判五样：挂载做得成；计数是 1；到头一次改动之后这把 key 还在映射里的，映射落点上两份都好（搬了或本来就换了）；文件读出原内容、快照读出原内容；
/// 池级 checker 不比没弄坏时多红。打破任何一样记一笔、最后整条红（红了就是新的打中）。
fn sweep_every_mapped_role(
    name: &str,
    devices_of_the_start: &Devices,
    version: &TransactionOutput,
    snapshot_expected: Option<(singlefs_core::address::TreeIdentifier, Vec<u8>)>,
) {
    let content_expected = whole_first_file(&memory_pool_of(devices_of_the_start), version);
    let baseline_violations = violated_invariants(&memory_pool_of(devices_of_the_start));
    let mut failures: Vec<String> = Vec::new();
    for (role, key) in &version.mapped_units {
        let unit_bytes = usize::try_from(role.span_slots() * SLOT_BYTES).expect("装得进 usize");
        let Some(locations) = mapped_locations(version, key) else {
            failures.push(format!("{role:?}：起点那一版映射里查不到"));
            continue;
        };
        let mut devices = devices_holding_the_same_bytes(devices_of_the_start);
        damage_the_copy_at(&mut devices, &locations[0], unit_bytes);
        let mounted = match mount_writable(&parameters(), &mut devices) {
            Ok(mounted) => mounted,
            Err(refusal) => {
                failures.push(format!("{role:?}：只坏一份却挂载不成：{refusal:?}"));
                continue;
            }
        };
        let (_, mut session) = MountedSession::of_the_mount(mounted);
        let count = session.unit_reads_with_one_copy_failing_while_mounting();
        let keys_kept = session
            .mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting_not_yet_relocated()
            .len();
        if count != 1 {
            failures.push(format!("{role:?}：挂载计 {count}，不是 1"));
        }
        let published = match session.publish_user_change(&mut devices, new_inode_change(20)) {
            Ok(published) => published,
            Err(refusal) => {
                failures.push(format!("{role:?}：头一次改动被拒：{refusal:?}"));
                continue;
            }
        };
        let outcome =
            outcome_name(&published.relocation_of_the_old_data_units_kept_for_the_snapshots);
        let after = current_version_of(&session);
        let locations_after = mapped_locations(&after, key);
        let both_good_after = locations_after.is_none_or(|locations_now| {
            locations_now
                .iter()
                .all(|location| copy_self_verifies(&devices, location, unit_bytes))
        });
        if !both_good_after {
            failures.push(format!(
                "{role:?}：头一次改动之后映射落点上还有坏的一份（outcome={outcome}）"
            ));
        }
        if outcome != "relocated" && outcome != "not_needed" {
            failures.push(format!("{role:?}：另发那一批结局 {outcome}"));
        }
        let image = memory_pool_of(&devices);
        if whole_first_file(&image, &after) != content_expected {
            failures.push(format!("{role:?}：文件内容变了"));
        }
        if let Some((tree, content_of_the_snapshot)) = &snapshot_expected {
            match read_the_first_file_of_a_snapshot(&image, &after.root, *tree) {
                Ok(SnapshotFileReading::File(read)) if read == *content_of_the_snapshot => {}
                other => failures.push(format!("{role:?}：快照读出 {:?}", other.map(|_| "别的"))),
            }
        }
        let new_violations: Vec<&str> = violated_invariants(&image)
            .into_iter()
            .filter(|invariant| !baseline_violations.contains(invariant))
            .collect();
        if !new_violations.is_empty() {
            failures.push(format!("{role:?}：池级 checker 多红 {new_violations:?}"));
        }
        let count_at_the_next_mount = session_of_a_writable_mount(&mut devices)
            .unit_reads_with_one_copy_failing_while_mounting();
        println!(
            "attack-fixes-r1: sweep={name} role={role:?} count_at_the_mount={count} keys_kept={keys_kept} outcome={outcome} key_still_mapped={} both_good_after={both_good_after} new_violations={new_violations:?} count_at_the_next_mount={count_at_the_next_mount}",
            locations_after.is_some()
        );
    }
    println!(
        "attack-fixes-r1: sweep={name} roles={} failures={}",
        version.mapped_units.len(),
        failures.len()
    );
    assert!(failures.is_empty(), "sweep={name}：{failures:#?}");
}

#[test]
fn sweep_every_mapped_role_of_a_plain_three_unit_pool() {
    let pool = pool_holding(&content_of_units(3));
    sweep_every_mapped_role("plain", &pool.devices, &pool.version, None);
}

/// 起点多一个快照、一条留存（部分写换下第 0 个单元）、一个空 inode：盘上有 deadlist 与快照两种条目。
#[test]
fn sweep_every_mapped_role_of_a_pool_with_a_snapshot_and_a_kept_unit() {
    let pool = pool_holding(&content_of_units(3));
    let mut devices = devices_holding_the_same_bytes(&pool.devices);
    let mut session = session_of_a_writable_mount(&mut devices);
    session
        .publish_user_change(&mut devices, UserChange::CreateSnapshot)
        .expect("建快照");
    let snapshot = current_version_of(&session)
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .expect("树表里有快照的码 14 条目")
        .tree;
    let patch = vec![0x5Au8; 64];
    session
        .publish_user_change(
            &mut devices,
            UserChange::WriteAtOffset {
                inode: FIRST_FILE,
                offset: FileOffsetInBytes(100),
                bytes: PartialWriteBytes::new(&patch).expect("写的字节不是空的"),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            },
        )
        .expect("部分写");
    session
        .publish_user_change(&mut devices, new_inode_change(4))
        .expect("建一个空 inode");
    let version = current_version_of(&session);
    sweep_every_mapped_role(
        "snapshot",
        &devices,
        &version,
        Some((snapshot, content_of_units(3))),
    );
}

/// 挂载那几处按位置条目去重与认 key 的代价随坏的单元数涨（数都是这一次跑量的，编译档照跑测试的那一档）：
/// 套着数的读者 `contains` 是坏的单元数的平方；认 key 是进映射的单元数乘坏的单元数。
#[test]
fn the_counting_reader_and_the_key_recognition_grow_with_the_units_read_with_one_copy_failing() {
    let image = MemoryPool::default();
    for units in [5_000u64, 10_000, 20_000] {
        let counting = PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&image);
        let started = Instant::now();
        for slot in 0..units {
            counting.note_that_a_unit_was_read_with_one_copy_failing(&[
                LocationEntry {
                    device: DeviceIdentity(0),
                    slot: SlotNumber(slot * 2),
                    unit_checksum: 7,
                },
                LocationEntry {
                    device: DeviceIdentity(1),
                    slot: SlotNumber(slot * 2),
                    unit_checksum: 7,
                },
            ]);
        }
        println!(
            "attack-fixes-r1: counting_reader units={units} milliseconds={}",
            started.elapsed().as_millis()
        );
        assert_eq!(counting.unit_reads_with_one_copy_failing(), units);
    }
    let pool = pool_holding(&content_of_units(700));
    for units in [1_000u64, 4_000, 16_000] {
        let locations: Vec<[LocationEntry; 2]> = (0..units)
            .map(|slot| {
                [DeviceIdentity(0), DeviceIdentity(1)].map(|device| LocationEntry {
                    device,
                    slot: SlotNumber(slot * 2 + 1),
                    unit_checksum: 7,
                })
            })
            .collect();
        let started = Instant::now();
        let keys = mapping_keys_of_the_units_at_these_locations(&pool.version, &locations);
        println!(
            "attack-fixes-r1: key_recognition mapped_units={} damaged_location_groups={units} milliseconds={} keys={}",
            pool.version.mapped_units.len(),
            started.elapsed().as_millis(),
            keys.len()
        );
    }
}

/// T3 与第 7 条「只试一次」：挂载记下的 key 走同一套被拒。头一次被空间拒（只供测试的开关）：观测推没推抬 F、搬没搬成。
#[test]
fn a_mount_key_refused_once_for_space_goes_through_the_same_floor_raise() {
    let pool = pool_holding(&content_of_units(3));
    let key = mapping_key_of_role(&pool.version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&pool.version, &key).expect("在映射里");
    let mut devices = devices_holding_the_same_bytes(&pool.devices);
    damage_the_copy_at(&mut devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    let mut session = session_of_a_writable_mount(&mut devices);
    session
        .compaction
        .force_the_relocation_kept_for_the_snapshots_refused_for_space_for_tests(
            RelocationKeptForTheSnapshotsRefusedForSpace::ForcedForTheNextAttemptsForTests {
                attempts: NonZeroU64::new(1).expect("拒一次：1 不是 0"),
            },
        );
    let published = session
        .publish_user_change(&mut devices, new_inode_change(3))
        .expect("建一个空 inode");
    let outcome = outcome_name(&published.relocation_of_the_old_data_units_kept_for_the_snapshots);
    let after = current_version_of(&session);
    let locations_after = mapped_locations(&after, &key).expect("还在映射里");
    println!(
        "attack-fixes-r1: mount_key_refused_once_for_space outcome={outcome} moved={} both_good_after={}",
        locations_after != locations,
        locations_after
            .iter()
            .all(|location| copy_self_verifies(&devices, location, BYTES_OF_A_DATA_UNIT))
    );
    assert_ne!(outcome, "not_needed");
}

/// 第 7 条「只试一次」：被拒不是空间不够之后，改动照成、key 清空，下一次改动不再搬、坏的那一份还在；下一次挂载重新读到、重新记下。
#[test]
fn a_mount_key_refused_not_for_space_is_tried_once_and_comes_back_at_the_next_mount() {
    let pool = pool_holding(&content_of_units(3));
    let key = mapping_key_of_role(&pool.version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&pool.version, &key).expect("在映射里");
    let mut devices = devices_holding_the_same_bytes(&pool.devices);
    damage_the_copy_at(&mut devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    let mut session = session_of_a_writable_mount(&mut devices);
    session
        .compaction
        .force_the_relocation_kept_for_the_snapshots_refused_not_for_space_for_tests(
            RelocationKeptForTheSnapshotsRefusedNotForSpace::ForcedOnEveryAttemptForTests,
        );
    let first = session
        .publish_user_change(&mut devices, new_inode_change(3))
        .expect("建一个空 inode");
    let first_outcome =
        outcome_name(&first.relocation_of_the_old_data_units_kept_for_the_snapshots);
    let keys_after_the_first = session
        .mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting_not_yet_relocated()
        .len();
    let second = session
        .publish_user_change(&mut devices, new_inode_change(4))
        .expect("再建一个空 inode");
    let second_outcome =
        outcome_name(&second.relocation_of_the_old_data_units_kept_for_the_snapshots);
    let damaged_copy_verifies = copy_self_verifies(&devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    let next_session = session_of_a_writable_mount(&mut devices);
    let keys_at_the_next_mount = next_session
        .mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting_not_yet_relocated()
        .to_vec();
    println!(
        "attack-fixes-r1: mount_key_refused_not_for_space first_outcome={first_outcome} keys_after_the_first={keys_after_the_first} second_outcome={second_outcome} damaged_copy_self_verifies={damaged_copy_verifies} keys_at_the_next_mount={}",
        keys_at_the_next_mount.len()
    );
    assert_eq!(first_outcome, "refused_not_for_space");
    assert_eq!(keys_after_the_first, 0);
    assert_eq!(second_outcome, "not_needed");
    assert!(!damaged_copy_verifies);
    assert_eq!(keys_at_the_next_mount, vec![key]);
}
