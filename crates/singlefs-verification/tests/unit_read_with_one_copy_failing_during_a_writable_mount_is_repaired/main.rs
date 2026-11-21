//! 验证包模块：crash_point_replay_progress、crash_given_states、crash_amplification
//! 可写挂载期间别的读读到只坏一份（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬，包括……可写挂载期间别的读」）：
//! 经会话读文件（`MountedSession::read_the_file`）、写系统配置槽之前算 F 生效值、推抬 F 读根环与实例表（F 抬不动时也算）与重算影子账读各根的账、
//! 重建分配器读各根的账、取号之前取树表 0 条那一版的单元清单、落后支窗口之外坏了的那一份（用例在 `writable_mount_of_a_device_behind_the_version_judges_the_units_named_after_its_tail.rs`）、整理回退门读根环、回退、清扫、释放之前读盘核、搬留存单元读源、另发那一批认位置时沿链读实例表，都进这次挂载别的读那个数
//! （`MountedSession::other_unit_reads_with_one_copy_failing_in_this_mount`），下一次用户改动发成之后另发的那一批把那一份换掉。
//! 发 discard 之前沿链读实例表那一处的用例在 `compaction_runs_stop_and_discard_after_the_publish.rs`（要它那一套整理与转环的搭建）。
//! 豁免映射的三类（映射树节点、树表、实例表片）读到只坏一份时的主动修复：记下位置，下一次用户改动发成之后，
//! 那次改动本来就换下的（树表、映射树根）旧落点照释放之前读盘核隔离，没换下的（映射树根之下的节点、实例表片）另发的那一批换落点、两份新写，只试一次，
//! 被空间拒照推抬 F 再试一遍，结局随那次改动报出。映射里的 key 集合对不上这一版进映射的单元（一条 key 被换、封得对的盘），可写挂载拒。
//! 修好之后卸载、不放回字节判真盘（带不修的对照）。盘是两块内存稀疏盘，池在进程里造出来，不建镜像文件。

#[path = "../common/mod.rs"]
mod common;

use std::collections::BTreeSet;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::{
    file_content, parameters, with_unreadable_ranges, without_unreadable_ranges,
    FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
    FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, FileOffsetInBytes, InodeNumber,
};
use singlefs_core::allocation_record_tree::{
    AllocationRecordTreeNode, AllocationRecordTreeNodePosition,
};
use singlefs_core::allocator::{AllocationRecordQuarantineMark, Placement};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::code_two_tree::{
    CodeTwoKeyFieldWidths, CodeTwoTreeKey, CodeTwoTreeNodeContents,
};
use singlefs_core::compaction_plan::{
    AnswerOfTheCallerBeforeTheNextBatch, CallerMonotonicClockReading,
    CallerOfTheBackgroundCompaction, CompactionPacing, PlannedRelocation,
    RelocationBatchAfterTheChangeRefusedForSpace, RelocationBatchAfterTheChangeRefusedNotForSpace,
};
use singlefs_core::instance_table_sweep::{
    sweep_the_instance_table_at_the_end_of_the_mount, sweep_the_instance_table_while_mounted,
    InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
    InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
    NoRowDeletedThisRound, SweepPublishPlacements,
};
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling, roll_back_by_a_forward_publish,
    unmount, MountError, RollbackCandidateExclusion, RollbackError, RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_read::open_pool_for_read;
use singlefs_core::mounted_session::{
    MountedSession, RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing, UserChange,
    UserChangePublished, UserChangeRefused,
};
use singlefs_core::pointer::{LocationEntry, NodePointer};
use singlefs_core::records::{build_mapping_entry, parse_mapping_entry};
use singlefs_core::recovery::{
    instance_table_chain_of_root, rebuild_version, PoolReaderCountingUnitReadsWithOneCopyFailing,
    RebuildVersionFailure, RebuiltVersion, RecoveryFailure, CENTRAL_MAPPING_KEY_SET_INVARIANT,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    acquire_instance, central_mapping_locations_in_the_version, publish_first_file,
    publish_relocation_batch, publish_sequential_write,
    read_back_the_old_data_unit_telling_whether_a_copy_failed, warm_up, CopiesOfAUnitRead,
    FirstFile, MappingLookup, PartialWriteBytes, PoolVersion, PoolWriter, PublishError,
    RelocationBatchRefused, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::{build_index_node, data_unit_payload_capacity, parse_index_node};
use singlefs_verification::crash_amplification::{
    one_test_changes_the_environment_at_a_time, GivenStatesEngine, GivenStatesItem,
};
use singlefs_verification::crash_given_states::{
    GivenStateConclusion, GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
};
use singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};

mod damaged_copies_of_data_units_and_mapping_tree_leaves;
mod damaged_copies_of_instance_table_pages;
mod damaged_copies_of_tree_tables_and_accounts;
mod damaged_copies_read_by_rollbacks;
mod damaged_copies_read_by_the_release_check;
mod disk_judged_after_the_unmount;
mod keys_replaced_in_the_mapping;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

const FIRST_FILE: InodeNumber = InodeNumber(1);

const DAMAGED_UNIT: DataUnitIndexInFile = DataUnitIndexInFile(1);

/// 三个单元的文件里映射 key 最大的单元：事务号按单元序发，这个单元最大。
const LAST_UNIT_IN_THE_KEY_ORDER: DataUnitIndexInFile = DataUnitIndexInFile(2);

/// 数据单元映射 key 的尾段（事务号低 48 位）有几字节。
const BYTES_OF_THE_TAIL_OF_A_MAPPING_KEY: usize = 6;

const FLIPPED_BYTE_IN_THE_UNIT: usize = 4000;

const BYTES_OF_A_NODE: usize = 16384;

const BYTES_OF_A_DATA_UNIT: usize = 32768;

/// 让中央映射树多层的文件有几个数据单元：一片映射叶约 290 条（16384 字节、条目 55 字节），400 个单元装不进一片。
const UNITS_OF_A_FILE_WITH_A_MULTI_LEVEL_MAPPING_TREE: u64 = 400;

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
    version: TransactionOutput,
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
    let version = {
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
        publish_sequential_write(
            &mut writer,
            &mut allocator,
            &first,
            FirstFile {
                content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            },
            instance,
        )
        .expect("顺序写")
    };
    Pool { devices, version }
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

/// 弄坏一份时记下的：判池级 checker 之前把那个字节放回去要用（[`put_the_damaged_byte_back`]）。
struct DamagedCopy {
    location: LocationEntry,
    unit_bytes: usize,
    byte_before_the_damage: u8,
}

/// 把 `location` 那块盘上那一份（从它的槽起 `unit_bytes` 字节）改一个字节，另一份不动。
fn damage_the_copy_at(
    devices: &mut Devices,
    location: &LocationEntry,
    unit_bytes: usize,
) -> DamagedCopy {
    let device = device_named(devices, location.device);
    let mut unit = vec![0u8; unit_bytes];
    device
        .read_at(location.slot.to_device_offset(), &mut unit)
        .expect("读得出那一份");
    let byte_before_the_damage = unit[FLIPPED_BYTE_IN_THE_UNIT];
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
    DamagedCopy {
        location: *location,
        unit_bytes,
        byte_before_the_damage,
    }
}

/// 把弄坏的那个字节放回去。放回之后那一份与位置条目对不上（那一槽已经不是弄坏时的内容，被复用过）时不写、交回 `false`。
fn put_the_damaged_byte_back(devices: &mut Devices, damaged: &DamagedCopy) -> bool {
    let device = device_named(devices, damaged.location.device);
    let mut unit = vec![0u8; damaged.unit_bytes];
    device
        .read_at(damaged.location.slot.to_device_offset(), &mut unit)
        .expect("读得出那一份");
    unit[FLIPPED_BYTE_IN_THE_UNIT] = damaged.byte_before_the_damage;
    if crc32_castagnoli(&unit) != damaged.location.unit_checksum {
        return false;
    }
    device
        .write_at(
            damaged.location.slot.to_device_offset(),
            &unit,
            WriteDurability::Plain,
        )
        .expect("写回放好的那一份");
    true
}

/// `version` 这一版经哪几条位置条目引用单元：根记录直接持有的树表与映射树根、中央映射树每个节点、实例表链每一片、每个进映射的单元、留存给快照的映射条目。
fn locations_the_version_references(
    version: &TransactionOutput,
    devices: &Devices,
) -> Vec<LocationEntry> {
    let mut locations: Vec<LocationEntry> = Vec::new();
    locations.extend(version.root.tree_table.locations);
    locations.extend(version.root.mapping_root.locations);
    for pointer in &version.central_mapping_tree.pointers {
        locations.extend(pointer.locations);
    }
    for pointer in instance_table_chain_of_root(devices, &version.root)
        .expect("这一版的实例表至少有一份读得出")
        .page_pointers
    {
        locations.extend(pointer.locations);
    }
    for (_, key) in &version.mapped_units {
        locations.extend(mapped_locations(version, key).expect("进映射的单元在这一版的映射里"));
    }
    for (_, kept_locations) in &version.mapping_entries_kept_for_the_snapshots {
        locations.extend(*kept_locations);
    }
    locations
}

/// `version` 这一版引用不引用 `location` 那个落点（同一块盘的同一槽）。
fn version_references_the_location(
    version: &TransactionOutput,
    devices: &Devices,
    location: &LocationEntry,
) -> bool {
    locations_the_version_references(version, devices)
        .iter()
        .any(|referenced| referenced.device == location.device && referenced.slot == location.slot)
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

fn whole_first_file(devices: &Devices, version: &TransactionOutput) -> Vec<u8> {
    let image = memory_pool_of(devices);
    let mounted = open_pool_for_read(&image, &version.root).expect("打开挂载态");
    let file = mounted.open_file(&image, FIRST_FILE).expect("打开文件");
    let size = file.inode_record().size;
    file.read_at(&image, FileOffsetInBytes(0), size)
        .expect("从 0 读到 size")
        .bytes
}

fn whole_first_file_read_through_the_session(
    session: &MountedSession,
    devices: &Devices,
) -> singlefs_core::mounted_read::FileReadOutput {
    let size = current_version_of(session)
        .inode_record
        .expect("第一个文件那条记录")
        .size;
    session
        .read_the_file(devices, FIRST_FILE, FileOffsetInBytes(0), size)
        .expect("经会话读得出")
}

/// 这张盘（基线是它、写表为空、一个空的持久集合，交崩溃点重放的给定状态引擎判）上池级 checker 判红的那几条；只录那一趟、表归别的机器判的交 `None`。
fn pool_checker_violations_of_the_image(
    table_name: &str,
    image: &MemoryPool,
) -> Option<Vec<(&'static str, String)>> {
    let engine = {
        let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
        GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: format!(
                    "unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired::{table_name}"
                ),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
    };
    let judged = engine
        .judge_the_given_states(&GivenStatesOfATable {
            table_name: table_name.to_string(),
            base: image,
            writes: &[],
            segments: &[],
            seams: &[],
            persisted_sets: vec![Vec::new()],
        })
        .unwrap_or_else(|reason| panic!("给定状态表判不了（不退回别处判）：{reason}"));
    assert_eq!(
        engine.finish().red_items(),
        Vec::<String>::new(),
        "GPU 判不了的状态判红"
    );
    match judged.source {
        GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost => {}
        GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost
        | GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => return None,
    }
    judged
        .states
        .iter()
        .map(GivenStateConclusion::pool_checker_violations)
        .next()
}

/// 把 `damaged` 那几份的字节放回去之后，池级 checker 一条违例都没有。
/// 坏的那一份还在盘上、还被回退候选集里的旧根引用时，I-2.1、I-4.8、I-7.4 本来就该红，所以判之前放回；每一份都要放得回去，即那一槽没被复用过。
/// 放回之后 checker 看不见现行版本还引用不引用坏的那一份：调用方另用 [`version_references_the_location`] 钉住。
fn assert_the_pool_checker_is_green_with_the_damaged_bytes_put_back(
    table_name: &str,
    devices: &mut Devices,
    damaged: &[DamagedCopy],
) {
    for damaged_copy in damaged {
        assert!(
            put_the_damaged_byte_back(devices, damaged_copy),
            "{table_name}：弄坏的那一槽没被复用过：{:?}",
            damaged_copy.location
        );
    }
    if let Some(violations) =
        pool_checker_violations_of_the_image(table_name, &memory_pool_of(devices))
    {
        assert!(
            violations.is_empty(),
            "{table_name}：池级 checker 一条违例都没有：{violations:?}"
        );
    }
}

/// 交回 `published` 那一批搬的 key、换下的豁免映射的单元、为它推了几串抬 F；别的结局 panic。
fn relocated_of(published: &UserChangePublished) -> (&[Vec<u8>], &[TransactionUnit], usize) {
    match &published.relocation_after_the_change_of_the_units_read_with_one_copy_failing {
        RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::Relocated {
            mapping_keys,
            units_exempt_from_the_mapping_replaced,
            instance_table_chain_unreadable: None,
            floor_raises,
        } => (
            mapping_keys.as_slice(),
            units_exempt_from_the_mapping_replaced.as_slice(),
            floor_raises.floor_raises.len(),
        ),
        other @ (RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::Relocated {
            instance_table_chain_unreadable: Some(_),
            ..
        }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NoSpaceAfterRaisingTheFloor {
            ..
        }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace { .. }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::FloorRaiseFailedWhilePushingForSpace {
            ..
        }) => panic!("应另发一批并发成、实例表读得出，交回 {other:?}"),
    }
}

fn assert_nothing_more_to_relocate(published: &UserChangePublished) {
    assert!(
        matches!(
            published.relocation_after_the_change_of_the_units_read_with_one_copy_failing,
            RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded
        ),
        "不另发，交回 {:?}",
        published.relocation_after_the_change_of_the_units_read_with_one_copy_failing
    );
}

/// `version` 这一版隔离的份里，`is_the_unit` 判真的那几份：(盘, 槽) 升序。
fn quarantined_copies_of(
    version: &TransactionOutput,
    is_the_unit: impl Fn(&TransactionUnit) -> bool,
) -> Vec<(DeviceIdentity, u64)> {
    let mut copies: Vec<(DeviceIdentity, u64)> = version
        .quarantined_after_release_checksum_mismatch
        .iter()
        .filter(|copy| is_the_unit(&copy.unit))
        .map(|copy| (copy.device, copy.placement.slot.0))
        .collect();
    copies.sort_unstable();
    copies
}

fn device_and_slot_of_both_copies(locations: &[LocationEntry; 2]) -> Vec<(DeviceIdentity, u64)> {
    let mut copies: Vec<(DeviceIdentity, u64)> = locations
        .iter()
        .map(|location| (location.device, location.slot.0))
        .collect();
    copies.sort_unstable();
    copies
}

/// 部分写：有活快照时写 `DAMAGED_UNIT` 那个单元（读回只坏一份、换下来钉在快照上）。
fn partial_write_into_the_damaged_unit<'content>(
    patch: &'content [u8],
    write_time_offset: u64,
) -> UserChange<'content> {
    UserChange::WriteAtOffset {
        inode: FIRST_FILE,
        offset: FileOffsetInBytes(payload_capacity() + 100),
        bytes: PartialWriteBytes::new(patch).expect("写的字节不是空的"),
        write_time_seconds: FIXED_WRITE_TIME_SECONDS + write_time_offset,
    }
}

/// 中央映射树最左那片叶（key 最小，首字节是类标签，数据单元的类最小：只装数据单元的 key，建 inode、发空发布都不碰）：指针与字节数。
fn leftmost_mapping_tree_leaf_of(version: &TransactionOutput) -> NodePointer {
    let mapping_tree = &version.central_mapping_tree;
    assert!(
        mapping_tree.shape.nodes().len() > 1,
        "前提：映射树多层，节点数 {}",
        mapping_tree.shape.nodes().len()
    );
    mapping_tree
        .shape
        .nodes()
        .iter()
        .zip(&mapping_tree.pointers)
        .find(|(node, _)| matches!(node.contents, CodeTwoTreeNodeContents::Leaf { .. }))
        .map(|(_, pointer)| *pointer)
        .expect("多层的树有叶")
}

fn force_one_space_refusal_of_the_batch(session: &mut MountedSession) {
    session
        .compaction
        .force_the_relocation_batch_after_the_change_refused_for_space_for_tests(
            RelocationBatchAfterTheChangeRefusedForSpace::ForcedForTheNextAttemptsForTests {
                attempts: NonZeroU64::new(1).expect("拒一次：1 不是 0"),
            },
        );
}

/// 弄坏数据单元一份、经会话读到（别的读 = 1，账上有一组、下一次改动要发那一批）：下面几条用例拿它把那一批推到被空间拒那一支。
fn damage_the_data_unit_and_read_it_through_the_session(
    session: &MountedSession,
    devices: &mut Devices,
) {
    let version = current_version_of(session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    damage_the_copy_at(devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    whole_first_file_read_through_the_session(session, devices);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "前提：账上有那一组"
    );
}

fn copy_of_the_devices(devices: &Devices) -> Devices {
    devices
        .iter()
        .map(|(identity, device)| {
            let mut copy = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect()
}

/// 发布在读盘核之后才被拒（落盘中途写失败、冻结）那一形用的盘：照转给内存盘，打开开关之后每次写都报 I/O 错、不落盘；点名的偏移读报错。
struct DeviceFailingOnRequest {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    writes_fail: Arc<AtomicBool>,
    copy_whose_reads_fail: Arc<Mutex<Option<(DeviceIdentity, u64)>>>,
}

impl BlockDevice for DeviceFailingOnRequest {
    fn read_at(
        &self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        if *self
            .copy_whose_reads_fail
            .lock()
            .expect("用例单线程改这个开关，锁没有中毒")
            == Some((self.identity, offset.0))
        {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "用例设的读失败",
            )));
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        if self.writes_fail.load(Ordering::SeqCst) {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "用例设的写失败",
            )));
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn discard_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.inner.discard_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
    fn journal_record_offsets_hint(
        &self,
        ring_start: singlefs_core::address::DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<singlefs_core::address::DeviceOffsetInBytes>> {
        self.inner
            .journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

/// [`DeviceFailingOnRequest`] 那两块盘与它们共用的两个开关。读失败的开关连盘一起点名：一个单元的两份在两块盘的同一个槽上，只比偏移就两份一起读不出。
struct PoolFailingOnRequest {
    devices: Vec<(DeviceIdentity, DeviceFailingOnRequest)>,
    writes_fail: Arc<AtomicBool>,
    copy_whose_reads_fail: Arc<Mutex<Option<(DeviceIdentity, u64)>>>,
}

fn pool_failing_on_request(devices: Devices) -> PoolFailingOnRequest {
    let writes_fail = Arc::new(AtomicBool::new(false));
    let copy_whose_reads_fail = Arc::new(Mutex::new(None));
    PoolFailingOnRequest {
        devices: devices
            .into_iter()
            .map(|(identity, inner)| {
                (
                    identity,
                    DeviceFailingOnRequest {
                        identity,
                        inner,
                        writes_fail: Arc::clone(&writes_fail),
                        copy_whose_reads_fail: Arc::clone(&copy_whose_reads_fail),
                    },
                )
            })
            .collect(),
        writes_fail,
        copy_whose_reads_fail,
    }
}

/// 把 `version` 这一版的分配记录树根两份都弄坏：读这一版的账读不出，两份都坏不算只坏一份，不进别的读那个数。
fn damage_both_copies_of_the_allocation_record_tree_root(
    version: &TransactionOutput,
    devices: &mut Devices,
) {
    let locations = mapped_locations(
        version,
        &mapping_key_of_role(version, TransactionUnit::AllocationTree),
    )
    .expect("分配记录树根在映射里");
    for location in &locations {
        damage_the_copy_at(devices, location, BYTES_OF_A_NODE);
    }
}

/// 对现行那一版发一次必被拒的回退（目标的 txg 是下一次发布才有的，根环里没有）：拒之前读过现行那一版的实例表，走不到判候选之后的读；盘与分配器都不动。
fn read_the_current_instance_table_through_a_refused_rollback(
    session: &mut MountedSession,
    devices: &mut Devices,
) {
    let mut current = current_version_of(session);
    let target = RollbackTarget {
        instance: current.root.instance,
        checkpoint_txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
    };
    let refused = roll_back_by_a_forward_publish(
        &parameters(),
        devices,
        &mut session.allocator,
        &mut current,
        target,
    )
    .err();
    assert!(
        matches!(
            refused,
            Some(RollbackError::TargetNotACandidate {
                exclusion: RollbackCandidateExclusion::NotInRing,
                ..
            })
        ),
        "目标不在根环里：{refused:?}"
    );
}

/// 这段历史之后抬 F 抬得动：非空有效根够 4 条。
const CHANGES_BEFORE_THE_FLOOR_RAISE: u64 = 8;
