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

/// 经会话读文件读到只坏一份：内容照出、计 1；同一份坏副本等着下一次改动的这一程里再读不再加；下一次改动（建一个空 inode，不碰这个单元）发成之后
/// 另发的那一批把它搬到新槽、两份新写，旧落点两份隔离，现行版本不再引用弄坏的那个落点；账上清掉，再下一次改动不另发；放回弄坏的字节之后池级 checker 过。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：会话读文件」「现行版本不再引用弄坏的落点：搬迁那一批映射条目不换到目的地」两行。
#[test]
fn a_file_read_through_the_session_counts_the_damaged_copy_and_the_next_change_relocates_the_unit()
{
    let content = content_of_units(3);
    let mut pool = pool_holding(&content);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    assert!(
        version_references_the_location(&version, &pool.devices, &locations[0]),
        "前提：弄坏之前现行版本引用这个落点"
    );
    let damaged_copy = damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    let read = whole_first_file_read_through_the_session(&session, &pool.devices);
    assert_eq!(read.bytes, content, "读出原内容");
    assert_eq!(
        read.observation.data_unit_reads_with_one_copy_failing, 1,
        "这一次读读到只坏一份"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "进这次挂载别的读那个数"
    );
    whole_first_file_read_through_the_session(&session, &pool.devices);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "同一份坏副本等着下一次改动的这一程里只算一次"
    );
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let (mapping_keys, units_replaced, floor_raises) = relocated_of(&published);
    assert_eq!(mapping_keys, std::slice::from_ref(&key), "搬的就是那个单元");
    assert!(units_replaced.is_empty(), "没有豁免映射的单元要换");
    assert_eq!(floor_raises, 0, "那一批一次都没被拒");
    let after = current_version_of(&session);
    assert!(
        !version_references_the_location(&after, &pool.devices, &locations[0]),
        "搬之后现行版本不再引用弄坏的那个落点"
    );
    let new_locations = mapped_locations(&after, &key).expect("搬之后映射里查得到");
    for (new_location, old_location) in new_locations.iter().zip(&locations) {
        assert_ne!(new_location.slot, old_location.slot, "落点换到新槽");
        assert!(
            copy_self_verifies(&pool.devices, new_location, BYTES_OF_A_DATA_UNIT),
            "新槽上那一份新写"
        );
    }
    assert_eq!(
        quarantined_copies_of(&after, |unit| matches!(unit, TransactionUnit::Data(_))),
        device_and_slot_of_both_copies(&locations),
        "旧落点两份都隔离"
    );
    assert_eq!(whole_first_file(&pool.devices, &after), content);
    let next = session
        .publish_user_change(&mut pool.devices, new_inode_change(4))
        .expect("再建一个空 inode");
    assert_nothing_more_to_relocate(&next);
    assert!(
        !version_references_the_location(
            &current_version_of(&session),
            &pool.devices,
            &locations[0]
        ),
        "再下一次改动之后现行版本也不引用弄坏的那个落点"
    );
    assert_the_pool_checker_is_green_with_the_damaged_bytes_put_back(
        "file-read-relocated",
        &mut pool.devices,
        &[damaged_copy],
    );
}

/// `open_pool_for_read` 那条不挂在会话上的路是只读语义：读者自己的观测计 1，会话的数不加、下一次改动不搬、坏的那一份还在。
#[test]
fn a_file_read_off_the_session_during_the_mount_only_counts_in_its_own_observation() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    let image = memory_pool_of(&pool.devices);
    let mounted_for_read = open_pool_for_read(&image, &version.root).expect("打开挂载态");
    let file = mounted_for_read
        .open_file(&image, FIRST_FILE)
        .expect("打开文件");
    let size = file.inode_record().size;
    let output = file
        .read_at(&image, FileOffsetInBytes(0), size)
        .expect("读得出");
    assert_eq!(output.observation.data_unit_reads_with_one_copy_failing, 1);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        0
    );
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    assert_nothing_more_to_relocate(&published);
    let after = current_version_of(&session);
    assert_eq!(mapped_locations(&after, &key), Some(locations));
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

/// 实例表片读到只坏一份（可写挂载之后才坏）：有活快照时部分写读回只坏一份的单元，这次改动的发布写系统配置槽之前算 F 生效值读到实例表那一份，
/// 进别的读那个数（连同那一批搬留存单元读源那一次，共 2）；另发那一批头一次被空间拒（只供测试的开关）、推抬 F 之后发成，同一批搬那个单元、整条链重写实例表：
/// 行不变、落点换到新槽、两份新写，旧落点两份隔离，现行版本（连同留存给快照的映射条目）不再引用弄坏的那两个落点，放回弄坏的字节之后池级 checker 过。
/// 判别力：`crates/mutations.tsv`「主动修复：实例表不进那一批」那一行。
#[test]
fn an_instance_table_page_read_with_one_copy_damaged_by_the_publish_of_a_change_is_replaced_by_its_batch_after_raising_the_floor(
) {
    let content = content_of_units(3);
    let mut pool = pool_holding(&content);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, UserChange::CreateSnapshot)
        .expect("建快照");
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let data_locations = mapped_locations(&version, &key).expect("在映射里");
    let table_locations = version.root.instance_table.locations;
    for location in [&data_locations[0], &table_locations[0]] {
        assert!(
            version_references_the_location(&version, &pool.devices, location),
            "前提：弄坏之前现行版本引用这个落点：{location:?}"
        );
    }
    let damaged_data_copy =
        damage_the_copy_at(&mut pool.devices, &data_locations[0], BYTES_OF_A_DATA_UNIT);
    let damaged_table_copy =
        damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    let rows_before = instance_table_chain_of_root(&memory_pool_of(&pool.devices), &version.root)
        .expect("另一份读得出")
        .records
        .rows;
    session
        .compaction
        .force_the_relocation_batch_after_the_change_refused_for_space_for_tests(
            RelocationBatchAfterTheChangeRefusedForSpace::ForcedForTheNextAttemptsForTests {
                attempts: NonZeroU64::new(1).expect("拒一次：1 不是 0"),
            },
        );
    let patch = vec![0x5Au8; 64];
    let published = session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_damaged_unit(&patch, 3),
        )
        .expect("部分写");
    let (mapping_keys, units_replaced, floor_raises) = relocated_of(&published);
    assert_eq!(mapping_keys, std::slice::from_ref(&key));
    assert_eq!(
        units_replaced,
        [TransactionUnit::InstanceTable],
        "那次发布读到实例表那一份，这一批的计划定下之前就在账上"
    );
    assert!(floor_raises >= 1, "被空间拒过、推过抬 F");
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        2,
        "写系统配置槽之前读实例表一次、搬留存单元读源一次"
    );
    let after = current_version_of(&session);
    assert!(
        !version_references_the_location(&after, &pool.devices, &data_locations[0]),
        "现行版本（连同留存给快照的映射条目）不再引用弄坏的那个数据单元落点"
    );
    assert!(
        !version_references_the_location(&after, &pool.devices, &table_locations[0]),
        "现行版本不再引用弄坏的那个实例表落点"
    );
    for (new_location, old_location) in after
        .root
        .instance_table
        .locations
        .iter()
        .zip(&table_locations)
    {
        assert_ne!(
            new_location.slot, old_location.slot,
            "实例表第 0 片换到新槽"
        );
        assert!(
            copy_self_verifies(&pool.devices, new_location, BYTES_OF_A_DATA_UNIT),
            "新槽上那一份新写"
        );
    }
    assert_eq!(
        quarantined_copies_of(&after, |unit| matches!(
            unit,
            TransactionUnit::InstanceTable
        )),
        device_and_slot_of_both_copies(&table_locations),
        "旧落点两份都隔离"
    );
    assert_eq!(
        instance_table_chain_of_root(&memory_pool_of(&pool.devices), &after.root)
            .expect("读得回")
            .records
            .rows,
        rows_before,
        "行不变"
    );
    let next = session
        .publish_user_change(&mut pool.devices, new_inode_change(4))
        .expect("建一个空 inode");
    assert_nothing_more_to_relocate(&next);
    assert_the_pool_checker_is_green_with_the_damaged_bytes_put_back(
        "instance-table-replaced",
        &mut pool.devices,
        &[damaged_data_copy, damaged_table_copy],
    );
}

/// 释放之前读盘核读到只坏一份（整份顺序写重写弄坏的单元）：进别的读那个数；这个单元这次发布就换了落点、旧落点两份隔离，另发的那一批没事可做。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：释放之前读盘核」那一行。
#[test]
fn the_release_check_reading_one_damaged_copy_counts_it_and_the_unit_is_replaced_by_that_publish() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let locations = mapped_locations(
        &version,
        &mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT)),
    )
    .expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    let new_content: Vec<u8> = content_of_units(3).iter().map(|byte| byte ^ 0x5A).collect();
    let published = session
        .publish_user_change(
            &mut pool.devices,
            UserChange::SequentialWrite(FirstFile {
                content: &new_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            }),
        )
        .expect("整份顺序写");
    assert_nothing_more_to_relocate(&published);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "释放之前读盘核那一读进数"
    );
    let after = current_version_of(&session);
    assert_eq!(
        quarantined_copies_of(&after, |unit| matches!(unit, TransactionUnit::Data(_))),
        device_and_slot_of_both_copies(&locations),
        "读盘核读到了那一份坏副本，两份一起隔离"
    );
    assert_eq!(whole_first_file(&pool.devices, &after), new_content);
}

/// 搬留存单元读源读到只坏一份：另发的那一批被开关拒成不是空间不够、单元还钉在旧落点上；照它发一批搬迁，第二项计划对不上（这一批在任何写之前拒，走不到释放之前读盘核），
/// 头一项按留存条目读源那一次已经进别的读那个数。判别力：`crates/mutations.tsv`「可写挂载别的读：搬留存单元读源」那一行。
#[test]
fn reading_the_source_of_a_kept_unit_with_one_copy_damaged_counts_it_even_when_the_batch_is_refused(
) {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, UserChange::CreateSnapshot)
        .expect("建快照");
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    session
        .compaction
        .force_the_relocation_batch_after_the_change_refused_not_for_space_for_tests(
            RelocationBatchAfterTheChangeRefusedNotForSpace::ForcedOnEveryAttemptForTests,
        );
    let patch = vec![0x5Au8; 64];
    session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_damaged_unit(&patch, 3),
        )
        .expect("部分写");
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        0,
        "开关在读源之前就拒了那一批"
    );
    let after_the_partial_write = current_version_of(&session);
    let (_, kept_locations) = after_the_partial_write
        .mapping_entries_kept_for_the_snapshots
        .iter()
        .find(|(kept_key, _)| *kept_key == key)
        .expect("映射条目留存");
    let source = Placement {
        slot: kept_locations[0].slot,
        span: 2,
    };
    let planned = [
        PlannedRelocation {
            mapping_key: key.clone(),
            source,
        },
        PlannedRelocation {
            mapping_key: vec![0u8; 27],
            source,
        },
    ];
    let pool_parameters = parameters();
    let mut writer = PoolWriter::new(&pool_parameters, pool.devices.as_mut_slice());
    let refused = publish_relocation_batch(
        &mut writer,
        &mut session.allocator,
        &after_the_partial_write,
        &planned,
        session.instance,
    );
    assert!(
        matches!(
            refused,
            Err(RelocationBatchRefused::PlannedUnitNotInTheVersion { .. })
        ),
        "第二项计划对不上：{refused:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "头一项读源那一次进数"
    );
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

/// 映射树叶挂载读到只坏一份：挂载计 1、认不出映射 key；挂载这一串不碰它；头一次改动（建一个空 inode，也不碰它）发成之后另发的那一批把从根到它的路整条重写：
/// 叶换到新槽、两份新写，旧落点两份隔离，现行版本不再引用弄坏的那个落点；账上清掉，再下一次改动不另发；下一次挂载读不到坏副本（计 0），文件读出原内容，放回弄坏的字节之后池级 checker 过。
/// 判别力：`crates/mutations.tsv`「主动修复：映射树节点」两行。
#[test]
fn a_mapping_tree_leaf_read_with_one_copy_damaged_while_mounting_is_rewritten_after_the_next_change(
) {
    let content = content_of_units(UNITS_OF_A_FILE_WITH_A_MULTI_LEVEL_MAPPING_TREE);
    let mut pool = pool_holding(&content);
    let leaf = leftmost_mapping_tree_leaf_of(&pool.version);
    let damaged_copy = damage_the_copy_at(&mut pool.devices, &leaf.locations[0], BYTES_OF_A_NODE);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    assert_eq!(session.unit_reads_with_one_copy_failing_while_mounting(), 1);
    assert!(
        session
            .mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting_not_yet_relocated()
            .is_empty(),
        "映射树节点认不出映射 key"
    );
    assert!(
        current_version_of(&session)
            .central_mapping_tree
            .pointers
            .contains(&leaf),
        "前提：挂载这一串没碰这片叶"
    );
    assert!(
        version_references_the_location(
            &current_version_of(&session),
            &pool.devices,
            &leaf.locations[0]
        ),
        "前提：头一次改动之前现行版本引用这个落点"
    );
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let (mapping_keys, units_replaced, floor_raises) = relocated_of(&published);
    assert!(mapping_keys.is_empty(), "没有进映射的单元要搬");
    assert_eq!(
        units_replaced.len(),
        1,
        "换下的就是那片叶：{units_replaced:?}"
    );
    assert!(
        matches!(
            units_replaced[0],
            TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        ),
        "{units_replaced:?}"
    );
    assert_eq!(floor_raises, 0);
    let after = current_version_of(&session);
    assert!(
        !version_references_the_location(&after, &pool.devices, &leaf.locations[0]),
        "那片叶换了落点：现行版本不再引用弄坏的那个落点"
    );
    assert!(
        !after.central_mapping_tree.pointers.contains(&leaf),
        "那片叶的指针换了"
    );
    for pointer in &after.central_mapping_tree.pointers {
        for location in &pointer.locations {
            assert!(
                copy_self_verifies(&pool.devices, location, BYTES_OF_A_NODE),
                "映射树每个节点的每一份都读得回：{location:?}"
            );
        }
    }
    assert_eq!(
        quarantined_copies_of(&after, |unit| matches!(
            unit,
            TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        )),
        device_and_slot_of_both_copies(&leaf.locations),
        "旧落点两份都隔离"
    );
    let next = session
        .publish_user_change(&mut pool.devices, new_inode_change(4))
        .expect("再建一个空 inode");
    assert_nothing_more_to_relocate(&next);
    assert!(
        !version_references_the_location(
            &current_version_of(&session),
            &pool.devices,
            &leaf.locations[0]
        ),
        "再下一次改动之后现行版本也不引用弄坏的那个落点"
    );
    assert_eq!(
        session_of_a_writable_mount(&mut pool.devices)
            .unit_reads_with_one_copy_failing_while_mounting(),
        0,
        "下一次挂载读不到坏副本"
    );
    assert_eq!(
        whole_first_file_read_through_the_session(
            &session_of_a_writable_mount(&mut pool.devices),
            &pool.devices
        )
        .bytes,
        content
    );
    assert_the_pool_checker_is_green_with_the_damaged_bytes_put_back(
        "mapping-tree-leaf-rewritten",
        &mut pool.devices,
        &[damaged_copy],
    );
}

/// 同上，那一批头一次被空间拒（只供测试的开关）：推一串抬 F 之后再发同一批，叶照样换了落点。抬 F 的上限是第 4 新的非空有效根，不足 4 条取最旧的有效根
/// （D16（发布语义） 已定项 1）：新建文件、顺序写与弄坏之后那次改动只有 3 条，上限等于今天的 F、没有可抬的，所以弄坏之前先发两次建 inode，留下可抬的 F。
#[test]
fn a_mapping_tree_leaf_rewrite_refused_once_for_space_is_published_after_raising_the_floor() {
    let mut pool = pool_holding(&content_of_units(
        UNITS_OF_A_FILE_WITH_A_MULTI_LEVEL_MAPPING_TREE,
    ));
    let version_before_the_damage = {
        let mut session = session_of_a_writable_mount(&mut pool.devices);
        for write_time_offset in [3, 4] {
            session
                .publish_user_change(&mut pool.devices, new_inode_change(write_time_offset))
                .expect("建一个空 inode，留下一条非空有效根");
        }
        current_version_of(&session)
    };
    let leaf = leftmost_mapping_tree_leaf_of(&version_before_the_damage);
    damage_the_copy_at(&mut pool.devices, &leaf.locations[0], BYTES_OF_A_NODE);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .compaction
        .force_the_relocation_batch_after_the_change_refused_for_space_for_tests(
            RelocationBatchAfterTheChangeRefusedForSpace::ForcedForTheNextAttemptsForTests {
                attempts: NonZeroU64::new(1).expect("拒一次：1 不是 0"),
            },
        );
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(5))
        .expect("建一个空 inode");
    let (_, units_replaced, floor_raises) = relocated_of(&published);
    assert_eq!(units_replaced.len(), 1, "{units_replaced:?}");
    assert!(floor_raises >= 1, "被空间拒过、推过抬 F");
    assert!(
        !current_version_of(&session)
            .central_mapping_tree
            .pointers
            .contains(&leaf),
        "那片叶换了落点"
    );
}

/// 只试一次：那一批被拒、不是空间不够（只供测试的开关）时结局随那次改动报出，叶还在旧落点上；账上清掉，下一次改动不接着换。
/// 判别力：`crates/mutations.tsv`「主动修复：被拒之后接着试」那一行。
#[test]
fn a_mapping_tree_leaf_rewrite_refused_not_for_space_is_not_tried_again_by_the_next_change() {
    let mut pool = pool_holding(&content_of_units(
        UNITS_OF_A_FILE_WITH_A_MULTI_LEVEL_MAPPING_TREE,
    ));
    let leaf = leftmost_mapping_tree_leaf_of(&pool.version);
    damage_the_copy_at(&mut pool.devices, &leaf.locations[0], BYTES_OF_A_NODE);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .compaction
        .force_the_relocation_batch_after_the_change_refused_not_for_space_for_tests(
            RelocationBatchAfterTheChangeRefusedNotForSpace::ForcedOnEveryAttemptForTests,
        );
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    assert!(
        matches!(
            published.relocation_after_the_change_of_the_units_read_with_one_copy_failing,
            RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace { .. }
        ),
        "结局随那次改动报出：{:?}",
        published.relocation_after_the_change_of_the_units_read_with_one_copy_failing
    );
    assert!(
        current_version_of(&session)
            .central_mapping_tree
            .pointers
            .contains(&leaf),
        "叶还在旧落点上"
    );
    let next = session
        .publish_user_change(&mut pool.devices, new_inode_change(4))
        .expect("再建一个空 inode");
    assert_nothing_more_to_relocate(&next);
}

/// 树表读到只坏一份（挂载之后才坏、经会话读文件读到）：计 1；树表每次发布都换下，下一次改动自己就把它换了落点、两份新写，旧落点两份照释放之前读盘核隔离，
/// 另发的那一批没事可做，现行版本不再引用弄坏的那个落点；放回弄坏的字节之后池级 checker 过。判别力：`crates/mutations.tsv`「主动修复：记下位置的豁免单元不走释放之前读盘核」那一行。
#[test]
fn a_tree_table_read_with_one_copy_damaged_is_replaced_by_the_next_change_and_its_old_copies_are_quarantined(
) {
    let content = content_of_units(3);
    let mut pool = pool_holding(&content);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let table_locations = version.root.tree_table.locations;
    assert!(
        version_references_the_location(&version, &pool.devices, &table_locations[0]),
        "前提：弄坏之前现行版本引用这个落点"
    );
    let damaged_copy = damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_NODE);
    let read = whole_first_file_read_through_the_session(&session, &pool.devices);
    assert_eq!(read.bytes, content);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "树表那一份进数"
    );
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    assert_nothing_more_to_relocate(&published);
    let after = current_version_of(&session);
    for old_location in &table_locations {
        let old_record = after
            .allocation_records
            .iter()
            .find(|record| record.device == old_location.device && record.slot == old_location.slot)
            .expect("释放之前读盘核隔离的旧落点留在分配记录里");
        assert_eq!(
            old_record.quarantine_mark,
            AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck,
            "旧落点带隔离记号：{old_location:?}"
        );
    }
    for (new_location, old_location) in after.root.tree_table.locations.iter().zip(&table_locations)
    {
        assert_ne!(new_location.slot, old_location.slot, "树表换到新槽");
        assert!(
            copy_self_verifies(&pool.devices, new_location, BYTES_OF_A_NODE),
            "新槽上那一份新写"
        );
    }
    assert_eq!(
        quarantined_copies_of(&after, |unit| matches!(unit, TransactionUnit::TreeTable)),
        device_and_slot_of_both_copies(&table_locations),
        "旧落点两份都隔离"
    );
    assert_the_pool_checker_is_green_with_the_damaged_bytes_put_back(
        "tree-table-replaced",
        &mut pool.devices,
        &[damaged_copy],
    );
}

/// 这把数据单元 key 的事务号加一（尾段是事务号低 48 位，小端）。交的是这个文件 key 序里最末的单元时，新 key 不撞别的单元（事务号按单元序连着发）、仍小于后面别的类的 key。
fn mapping_key_with_the_transaction_number_raised_by_one(key: &[u8]) -> Vec<u8> {
    let mut transaction_number_bytes = [0u8; 8];
    transaction_number_bytes[..BYTES_OF_THE_TAIL_OF_A_MAPPING_KEY]
        .copy_from_slice(&key[key.len() - BYTES_OF_THE_TAIL_OF_A_MAPPING_KEY..]);
    let raised = u64::from_le_bytes(transaction_number_bytes) + 1;
    let mut raised_key = key.to_vec();
    let tail_starts_at = raised_key.len() - BYTES_OF_THE_TAIL_OF_A_MAPPING_KEY;
    raised_key[tail_starts_at..]
        .copy_from_slice(&raised.to_le_bytes()[..BYTES_OF_THE_TAIL_OF_A_MAPPING_KEY]);
    raised_key
}

/// 造一张映射里查不到 `key` 的盘：`version` 那一版的映射树根兼叶上 `key` 那一条换成 `replacement`（位置条目不动），条目仍严格升序，叶按写者的写法重装（两道校验和都对），
/// 两份写回原槽，根记录里映射根的整单元校验和换成新字节的，根槽重封写回。日志记录里那份新根段不动：它只用来施加水位之上的发布，这一条根不在水位之上。交回盘上那条根。
fn replace_the_key_in_the_mapping_root_on_disk(
    devices: &mut Devices,
    version: &TransactionOutput,
    key: &[u8],
    replacement: &[u8],
) -> RootRecord {
    let mut root = version.root;
    let forged = mapping_root_with_the_key_replaced(version, key, replacement);
    let forged_checksum = crc32_castagnoli(&forged);
    for location in &mut root.mapping_root.locations {
        device_named(devices, location.device)
            .write_at(
                location.slot.to_device_offset(),
                &forged,
                WriteDurability::Plain,
            )
            .expect("写回映射树根");
        location.unit_checksum = forged_checksum;
    }
    let pool_parameters = parameters();
    let ring_slot = target_for_publish(
        root.checkpoint_txg,
        pool_parameters.geometry.root_ring_slots_per_region,
    );
    let device_of_the_ring_slot =
        pool_parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    device_named(devices, device_of_the_ring_slot)
        .write_at(
            slot_offset(
                ring_slot,
                pool_parameters.geometry.fixed_structure_slot_spacing,
            ),
            &root.to_slot(
                usize::try_from(pool_parameters.geometry.physical_block_size).expect("根槽宽"),
            ),
            WriteDurability::Plain,
        )
        .expect("写回根槽");
    root
}

/// `version` 那一版的映射树根兼叶字节，`key` 那一条换成 `replacement`（位置条目不动），条目仍严格升序，按写者的写法重装（两道校验和都对）。
fn mapping_root_with_the_key_replaced(
    version: &TransactionOutput,
    key: &[u8],
    replacement: &[u8],
) -> Vec<u8> {
    let original = parse_index_node(
        &version
            .units
            .iter()
            .find(|unit| unit.identity == TransactionUnit::MappingTree)
            .expect("三个单元的池映射树根兼叶")
            .bytes,
    )
    .expect("映射树根解得开");
    let entries: Vec<(Vec<u8>, [LocationEntry; 2])> = original
        .entries
        .iter()
        .map(|entry| {
            let (entry_key, locations) = parse_mapping_entry(entry).expect("映射条目切得开");
            if entry_key == key {
                (replacement.to_vec(), locations)
            } else {
                (entry_key, locations)
            }
        })
        .collect();
    assert!(
        entries.windows(2).all(|pair| {
            CodeTwoTreeKey::new(&pair[0].0, CodeTwoKeyFieldWidths::CENTRAL_MAPPING)
                < CodeTwoTreeKey::new(&pair[1].0, CodeTwoKeyFieldWidths::CENTRAL_MAPPING)
        }),
        "前提：换了 key 之后条目仍严格升序"
    );
    build_index_node(
        original.tree,
        original.level,
        original.key_width,
        &original.smallest_key,
        &original.largest_key,
        original.birth_txg,
        &version.root.filesystem_identifier,
        original.instance,
        original.birth_sequence,
        u16::try_from(original.entry_width).expect("条目宽"),
        &entries
            .iter()
            .map(|(entry_key, locations)| build_mapping_entry(entry_key, *locations))
            .collect::<Vec<Vec<u8>>>(),
    )
}

/// 一个数只坏一份的读者沿 `version` 的链读一遍数到几份：对照，证那条链上还有没有一份坏的。
fn instance_table_reads_with_one_copy_failing(
    devices: &Devices,
    version: &TransactionOutput,
) -> u64 {
    let counting_reader =
        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(devices);
    instance_table_chain_of_root(&counting_reader, &version.root).expect("另一份读得出");
    counting_reader.unit_reads_with_one_copy_failing()
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

/// F 抬不动（非空根不足 4 条，上限就是今天的 F）：改动的发布读到实例表那一份、那一批带着它被空间拒、停在抬不动上；试过的那两组从账上去掉，
/// 下一次改动的发布再读到、再记一次，那一批把实例表整条链换掉（数据单元那一组不再试）。算上限那一读进不进账在
/// [`whichever_read_of_the_instance_table_page_fails_while_raising_the_floor_it_is_counted`]。
#[test]
fn an_instance_table_page_in_a_batch_refused_with_the_floor_stuck_is_replaced_by_the_next_change() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    damage_the_data_unit_and_read_it_through_the_session(&session, &mut pool.devices);
    let table_locations = current_version_of(&session).root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    force_one_space_refusal_of_the_batch(&mut session);
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(20))
        .expect("建一个空 inode");
    assert!(
        matches!(
            published.relocation_after_the_change_of_the_units_read_with_one_copy_failing,
            RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NoSpaceAfterRaisingTheFloor { .. }
        ),
        "前提：F 抬不动，停在算上限那一步：{:?}",
        published.relocation_after_the_change_of_the_units_read_with_one_copy_failing
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        2,
        "数据单元经会话读一次、改动的发布读实例表一次"
    );
    let next = session
        .publish_user_change(&mut pool.devices, new_inode_change(21))
        .expect("再建一个空 inode");
    let (mapping_keys, units_replaced, _) = relocated_of(&next);
    assert!(mapping_keys.is_empty(), "数据单元那一组试过一次了");
    assert_eq!(units_replaced, [TransactionUnit::InstanceTable]);
    let after = current_version_of(&session);
    assert_ne!(
        after.root.instance_table.locations[0].slot, table_locations[0].slot,
        "实例表换到新槽"
    );
    assert_eq!(
        quarantined_copies_of(&after, |unit| matches!(
            unit,
            TransactionUnit::InstanceTable
        )),
        device_and_slot_of_both_copies(&table_locations),
        "旧落点两份都隔离"
    );
}

/// 算上限读每条有效根的树表（`mount::admission_ceilings_not_passing_ring_slots_bad_since_the_mount`）：改动之前那一版的树表弄坏一份，只有这一读读到，进别的读那个数。
/// 改动已经把现行树表换了，这一组到下一次改动认不出、不另发。判别力：`crates/mutations.tsv`「可写挂载别的读：准入算上限读树表不进账」那一行。
#[test]
fn the_tree_table_read_for_the_admission_ceiling_counts_the_damaged_copy() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    damage_the_data_unit_and_read_it_through_the_session(&session, &mut pool.devices);
    let tree_table_locations = current_version_of(&session).root.tree_table.locations;
    damage_the_copy_at(&mut pool.devices, &tree_table_locations[0], BYTES_OF_A_NODE);
    force_one_space_refusal_of_the_batch(&mut session);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(20))
        .expect("建一个空 inode");
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        2,
        "算上限读树表读到的那一份进数"
    );
}

/// 改动的发布每块盘写系统配置槽之前各读一次实例表（两块盘两次）：头两次都读得好，账上只有这次改动换下的数据单元落点。
/// 第三次读是另发那一批认位置时自己沿链读实例表；从这一次起那一份读坏，只有这一读及时读到。
const READ_OF_THE_INSTANCE_TABLE_PAGE_BY_THE_CHAIN_READ_OF_THE_REPAIR: u64 = 3;

/// 另发那一批认位置时自己沿链读实例表：账上只有一组这次改动换下的数据单元落点时走这一读。读到的那一片进别的读那个数、同一批接着换：
/// 下一次改动之前链上已经没有坏的。判别力：`crates/mutations.tsv`「主动修复：认位置沿链读实例表读到的不进账」「主动修复：认位置读到的片不进那一批」两行。
#[test]
fn the_chain_read_of_the_repair_counts_the_damaged_page_and_the_same_batch_replaces_it() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let data_locations = mapped_locations(&version, &key).expect("在映射里");
    let table_locations = version.root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &data_locations[0], BYTES_OF_A_DATA_UNIT);
    let reads_of_the_page = reads_of_the_copy(
        &table_locations[0],
        FailingReadsOfARange::FromTheNthOnward(
            READ_OF_THE_INSTANCE_TABLE_PAGE_BY_THE_CHAIN_READ_OF_THE_REPAIR,
        ),
    );
    let mut devices = with_unreadable_ranges(pool.devices, &reads_of_the_page);
    let new_content: Vec<u8> = content_of_units(3).iter().map(|byte| byte ^ 0x5A).collect();
    let published = session
        .publish_user_change(
            &mut devices,
            UserChange::SequentialWrite(FirstFile {
                content: &new_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            }),
        )
        .expect("整份顺序写");
    let (mapping_keys, units_replaced, _) = relocated_of(&published);
    assert!(mapping_keys.is_empty(), "数据单元那次改动已经重写");
    assert_eq!(units_replaced, [TransactionUnit::InstanceTable]);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        2,
        "释放之前读盘核一次、认位置沿链读实例表一次"
    );
    assert!(
        reads_of_the_page.failed_reads() >= 1,
        "前提：真有一次读坏：那一份一共读了 {} 次",
        reads_of_the_page.reads_of_range(0)
    );
    let mut devices = without_unreadable_ranges(devices);
    let after = current_version_of(&session);
    assert_eq!(
        instance_table_reads_with_one_copy_failing(&devices, &after),
        0,
        "同一批换了那一片：链上没有坏的了"
    );
    assert_eq!(
        quarantined_copies_of(&after, |unit| matches!(
            unit,
            TransactionUnit::InstanceTable
        )),
        device_and_slot_of_both_copies(&table_locations),
        "旧落点两份都隔离"
    );
    let next = session
        .publish_user_change(&mut devices, new_inode_change(4))
        .expect("建一个空 inode");
    assert_nothing_more_to_relocate(&next);
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

/// 账的口径：同一份坏副本挂载读到一次，数据单元（认得出 key）与映射树叶（认不出）一样：挂载那个数 1，之后再读、或另发那一批换下时释放之前读盘核，别的读那个数都不加。
/// 判别力：`crates/mutations.tsv`「账的口径：挂载交回之前把认成 key 的那几组从账上去掉」那一行。
#[test]
fn the_other_reads_count_of_a_copy_read_while_mounting_is_the_same_for_a_data_unit_and_a_mapping_tree_leaf(
) {
    let mut data_pool = pool_holding(&content_of_units(3));
    let key = mapping_key_of_role(&data_pool.version, TransactionUnit::Data(DAMAGED_UNIT));
    let data_locations = mapped_locations(&data_pool.version, &key).expect("在映射里");
    damage_the_copy_at(
        &mut data_pool.devices,
        &data_locations[0],
        BYTES_OF_A_DATA_UNIT,
    );
    let mut devices_read_again = copy_of_the_devices(&data_pool.devices);
    let session_reading_again = session_of_a_writable_mount(&mut devices_read_again);
    let data_mount = session_reading_again.unit_reads_with_one_copy_failing_while_mounting();
    whole_first_file_read_through_the_session(&session_reading_again, &devices_read_again);
    let data_read_again =
        session_reading_again.other_unit_reads_with_one_copy_failing_in_this_mount();
    let mut devices_not_read_again = copy_of_the_devices(&data_pool.devices);
    let mut session_not_reading_again = session_of_a_writable_mount(&mut devices_not_read_again);
    let published = session_not_reading_again
        .publish_user_change(&mut devices_not_read_again, new_inode_change(3))
        .expect("建一个空 inode");
    let (data_mapping_keys, _, _) = relocated_of(&published);
    assert_eq!(data_mapping_keys, std::slice::from_ref(&key));
    let data_relocated_without_another_read =
        session_not_reading_again.other_unit_reads_with_one_copy_failing_in_this_mount();

    let mut leaf_pool = pool_holding(&content_of_units(
        UNITS_OF_A_FILE_WITH_A_MULTI_LEVEL_MAPPING_TREE,
    ));
    let leaf = leftmost_mapping_tree_leaf_of(&leaf_pool.version);
    damage_the_copy_at(&mut leaf_pool.devices, &leaf.locations[0], BYTES_OF_A_NODE);
    let mut leaf_session = session_of_a_writable_mount(&mut leaf_pool.devices);
    let leaf_mount = leaf_session.unit_reads_with_one_copy_failing_while_mounting();
    whole_first_file_read_through_the_session(&leaf_session, &leaf_pool.devices);
    let leaf_read_again = leaf_session.other_unit_reads_with_one_copy_failing_in_this_mount();
    let leaf_published = leaf_session
        .publish_user_change(&mut leaf_pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let (_, leaf_units_replaced, _) = relocated_of(&leaf_published);
    assert_eq!(leaf_units_replaced.len(), 1, "{leaf_units_replaced:?}");
    let leaf_replaced = leaf_session.other_unit_reads_with_one_copy_failing_in_this_mount();

    assert_eq!(
        (
            data_mount,
            data_read_again,
            data_relocated_without_another_read
        ),
        (1, 0, 0),
        "数据单元：挂载、再读、换下时读盘核"
    );
    assert_eq!(
        (leaf_mount, leaf_read_again, leaf_replaced),
        (1, 0, 0),
        "映射树叶：挂载、再读、换下时读盘核"
    );
}

/// 账上有一组这次改动已换下的数据单元落点、实例表两份都读不出：认不出那一组是不是实例表片，实例表那一半跳过；与实例表无关的好 key 照搬，结局报明实例表没换。
/// 判别力：`crates/mutations.tsv`「主动修复：实例表读不出整批不发」「主动修复：结局不报实例表没换」两行。
#[test]
fn an_unreadable_instance_table_leaves_the_table_in_place_and_still_relocates_an_unrelated_unit() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key_of_the_unit_kept =
        mapping_key_of_role(&version, TransactionUnit::Data(DataUnitIndexInFile(0)));
    let key_of_the_unit_rewritten =
        mapping_key_of_role(&version, TransactionUnit::Data(LAST_UNIT_IN_THE_KEY_ORDER));
    let locations_of_the_unit_kept =
        mapped_locations(&version, &key_of_the_unit_kept).expect("在映射里");
    let locations_of_the_unit_rewritten =
        mapped_locations(&version, &key_of_the_unit_rewritten).expect("在映射里");
    damage_the_copy_at(
        &mut pool.devices,
        &locations_of_the_unit_kept[0],
        BYTES_OF_A_DATA_UNIT,
    );
    damage_the_copy_at(
        &mut pool.devices,
        &locations_of_the_unit_rewritten[0],
        BYTES_OF_A_DATA_UNIT,
    );
    whole_first_file_read_through_the_session(&session, &pool.devices);
    let table_locations = version.root.instance_table.locations;
    for location in &table_locations {
        damage_the_copy_at(&mut pool.devices, location, BYTES_OF_A_DATA_UNIT);
    }
    let patch = vec![0x5Au8; 64];
    let published = session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_damaged_unit(&patch, 3),
        )
        .expect("部分写");
    match &published.relocation_after_the_change_of_the_units_read_with_one_copy_failing {
        RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::Relocated {
            mapping_keys,
            units_exempt_from_the_mapping_replaced,
            instance_table_chain_unreadable,
            ..
        } => {
            assert_eq!(mapping_keys, std::slice::from_ref(&key_of_the_unit_kept));
            assert!(units_exempt_from_the_mapping_replaced.is_empty());
            assert!(
                matches!(
                    instance_table_chain_unreadable,
                    Some(RecoveryFailure::UnitUnreadable { .. })
                ),
                "结局报明实例表没换：{instance_table_chain_unreadable:?}"
            );
        }
        other @ (RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NoSpaceAfterRaisingTheFloor {
            ..
        }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace { .. }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::FloorRaiseFailedWhilePushingForSpace {
            ..
        }) => panic!("与实例表无关的好 key 照搬，交回 {other:?}"),
    }
    let after = current_version_of(&session);
    assert_ne!(
        mapped_locations(&after, &key_of_the_unit_kept),
        Some(locations_of_the_unit_kept),
        "好 key 搬了"
    );
    assert_eq!(
        after.root.instance_table.locations, table_locations,
        "实例表没换"
    );
}

/// 同上，一样要发的都没有（账上只有这次改动已换下的那一组）：不发，读不出照报在结局里，不报成不用发。
/// 判别力：`crates/mutations.tsv`「主动修复：实例表读不出、没别的要发时报成不用发」那一行。
#[test]
fn an_unreadable_instance_table_with_nothing_else_to_relocate_is_reported_in_the_outcome() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    whole_first_file_read_through_the_session(&session, &pool.devices);
    for location in &version.root.instance_table.locations {
        damage_the_copy_at(&mut pool.devices, location, BYTES_OF_A_DATA_UNIT);
    }
    let patch = vec![0x5Au8; 64];
    let published = session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_damaged_unit(&patch, 3),
        )
        .expect("部分写重写弄坏的单元");
    match &published.relocation_after_the_change_of_the_units_read_with_one_copy_failing {
        RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace {
            refusal,
            ..
        } => assert!(
            matches!(
                **refusal,
                RelocationBatchRefused::InstanceTableOfTheVersionUnreadable(_)
            ),
            "{refusal:?}"
        ),
        other @ (RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::Relocated { .. }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NoSpaceAfterRaisingTheFloor {
            ..
        }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::FloorRaiseFailedWhilePushingForSpace {
            ..
        }) => panic!("读不出报在结局里，交回 {other:?}"),
    }
}

/// 「只试一次」的射程：这一组记下的这一次只试一次；账上清掉之后同一组再读到就再记、下一次改动再试。
/// 数据单元这一组是那一批计划里的源落点，账上清掉它的有「试过的组」「计划里的源落点」两支；豁免映射的单元只有前一支，在下面那一条用例。
/// 判别力：`crates/mutations.tsv`「主动修复：那一批发完账上一组都不去掉」那一行。
#[test]
fn a_batch_refused_not_for_space_is_tried_again_once_the_unit_is_read_again() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    session
        .compaction
        .force_the_relocation_batch_after_the_change_refused_not_for_space_for_tests(
            RelocationBatchAfterTheChangeRefusedNotForSpace::ForcedOnEveryAttemptForTests,
        );
    // 迭代上界是下面这三遍；跨轮不带状态，每一遍都读一次、发一次改动。
    for attempt in 0..3u64 {
        whole_first_file_read_through_the_session(&session, &pool.devices);
        let published = session
            .publish_user_change(&mut pool.devices, new_inode_change(3 + attempt))
            .expect("建一个空 inode");
        assert!(
            matches!(
                published.relocation_after_the_change_of_the_units_read_with_one_copy_failing,
                RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace { .. }
            ),
            "每一次读过之后的那次改动都再试：{:?}",
            published.relocation_after_the_change_of_the_units_read_with_one_copy_failing
        );
    }
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        3
    );
}

/// 实例表片那一组不是那一批计划里的源落点（实例表豁免映射），账上清掉它的只有「试过的组」那一支：每一遍经一次被拒的回退读一次实例表，再记一次，下一次改动再试。
/// 判别力：`crates/mutations.tsv`「主动修复：试过的组再读到不再记」那一行。
#[test]
fn a_batch_refused_not_for_space_for_the_instance_table_is_tried_again_once_the_page_is_read_again()
{
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let table_locations = current_version_of(&session).root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    session
        .compaction
        .force_the_relocation_batch_after_the_change_refused_not_for_space_for_tests(
            RelocationBatchAfterTheChangeRefusedNotForSpace::ForcedOnEveryAttemptForTests,
        );
    // 迭代上界是下面这三遍；跨轮不带状态，每一遍都读一次、发一次改动。
    for attempt in 0..3u64 {
        read_the_current_instance_table_through_a_refused_rollback(&mut session, &mut pool.devices);
        assert_eq!(
            session.other_unit_reads_with_one_copy_failing_in_this_mount(),
            attempt + 1,
            "账上清掉之后同一组再读到再记一次"
        );
        let published = session
            .publish_user_change(&mut pool.devices, new_inode_change(3 + attempt))
            .expect("建一个空 inode");
        assert!(
            matches!(
                published.relocation_after_the_change_of_the_units_read_with_one_copy_failing,
                RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace { .. }
            ),
            "每一次读过之后的那次改动都再试：{:?}",
            published.relocation_after_the_change_of_the_units_read_with_one_copy_failing
        );
    }
    assert_eq!(
        current_version_of(&session).root.instance_table.locations,
        table_locations,
        "那一批每次都被拒，实例表还在旧落点上"
    );
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

/// 发布在读盘核之后才被拒（整份顺序写重写弄坏的单元，落盘中途写失败、冻结）：读盘核读到的那一份已经进别的读那个数，改动没发成也不退；
/// 原样重发之后旧落点两份隔离，再下一次改动不另发。判别力：`crates/mutations.tsv`「可写挂载别的读：释放之前读盘核读到只坏一份不记（发布后来被拒）」那一行。
#[test]
fn a_publish_failing_after_the_release_check_keeps_the_count_and_the_resent_version_quarantines_both_copies(
) {
    let pool = pool_holding(&content_of_units(3));
    let version = pool.version.clone();
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    let mut failing = pool_failing_on_request(pool.devices);
    let mounted = mount_writable(&parameters(), &mut failing.devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let device_of_the_damaged_copy = &mut failing
        .devices
        .iter_mut()
        .find(|(identity, _)| *identity == locations[0].device)
        .expect("池里有这块盘")
        .1
        .inner;
    let mut damaged_copy = vec![0u8; BYTES_OF_A_DATA_UNIT];
    device_of_the_damaged_copy
        .read_at(locations[0].slot.to_device_offset(), &mut damaged_copy)
        .expect("读得出那一份");
    damaged_copy[FLIPPED_BYTE_IN_THE_UNIT] ^= 0xFF;
    device_of_the_damaged_copy
        .write_at(
            locations[0].slot.to_device_offset(),
            &damaged_copy,
            WriteDurability::Plain,
        )
        .expect("写回改过的那一份");
    let new_content: Vec<u8> = content_of_units(3).iter().map(|byte| byte ^ 0x5A).collect();
    failing.writes_fail.store(true, Ordering::SeqCst);
    let refused = session.publish_user_change(
        &mut failing.devices,
        UserChange::SequentialWrite(FirstFile {
            content: &new_content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
        }),
    );
    assert!(
        matches!(refused, Err(UserChangeRefused::Publish { .. })),
        "落盘中途写失败：{refused:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "读盘核读到的那一份进数，改动被拒不退"
    );
    failing.writes_fail.store(false, Ordering::SeqCst);
    assert!(
        session
            .resend_the_frozen_publish(&mut failing.devices)
            .expect("写不再失败，重发得成"),
        "有冻结着的那一次"
    );
    let after = current_version_of(&session);
    assert_eq!(
        quarantined_copies_of(&after, |unit| matches!(unit, TransactionUnit::Data(_))),
        device_and_slot_of_both_copies(&locations),
        "读盘核读到了那一份坏副本，重发的那一版两份一起隔离"
    );
    let next = session
        .publish_user_change(&mut failing.devices, new_inode_change(4))
        .expect("建一个空 inode");
    assert_nothing_more_to_relocate(&next);
}

/// 第二份坏：读路径头一份对上就停、读不到它，别的读那个数不加；释放之前读盘核两份都读，读到它：进数，两份一起隔离。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：释放之前读盘核只看头一份读得好不好（第二份坏不报）」那一行。
#[test]
fn a_second_copy_damaged_is_not_seen_by_the_file_read_and_is_counted_by_the_release_check() {
    let content = content_of_units(3);
    let mut pool = pool_holding(&content);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let locations = mapped_locations(
        &version,
        &mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT)),
    )
    .expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[1], BYTES_OF_A_DATA_UNIT);
    assert_eq!(
        whole_first_file_read_through_the_session(&session, &pool.devices).bytes,
        content
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        0,
        "头一份对上就停，读不到第二份"
    );
    let new_content: Vec<u8> = content.iter().map(|byte| byte ^ 0x5A).collect();
    let published = session
        .publish_user_change(
            &mut pool.devices,
            UserChange::SequentialWrite(FirstFile {
                content: &new_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            }),
        )
        .expect("整份顺序写");
    assert_nothing_more_to_relocate(&published);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "释放之前读盘核读到第二份"
    );
    assert_eq!(
        quarantined_copies_of(&current_version_of(&session), |unit| matches!(
            unit,
            TransactionUnit::Data(_)
        )),
        device_and_slot_of_both_copies(&locations),
        "两份一起隔离"
    );
}

/// 头一份读回 `None`（设备报读错，不是校验和对不上）：与读成而对不上同一支：内容照出、进别的读那个数，下一次改动发成之后另发的那一批搬到新槽。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：读回 None 不算只坏一份（只认读成而对不上）」那一行。
#[test]
fn a_copy_reading_back_none_counts_like_a_damaged_copy_and_the_next_change_relocates_the_unit() {
    let content = content_of_units(3);
    let pool = pool_holding(&content);
    let mut failing = pool_failing_on_request(pool.devices);
    let mounted = mount_writable(&parameters(), &mut failing.devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    *failing
        .copy_whose_reads_fail
        .lock()
        .expect("这条用例单线程，锁没有中毒") =
        Some((locations[0].device, locations[0].slot.to_device_offset().0));
    let size = version.inode_record.expect("第一个文件那条记录").size;
    let read = session
        .read_the_file(&failing.devices, FIRST_FILE, FileOffsetInBytes(0), size)
        .expect("另一份读得出");
    assert_eq!(read.bytes, content);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
    let published = session
        .publish_user_change(&mut failing.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let (mapping_keys, _, _) = relocated_of(&published);
    assert_eq!(mapping_keys, std::slice::from_ref(&key));
    assert_ne!(
        mapped_locations(&current_version_of(&session), &key),
        Some(locations),
        "搬到新槽"
    );
}

/// [`publish_relocation_batch`] 的计划头一项（现行那一版的单元）读盘核读到只坏一份，第二项（留存单元，内存里那一版把两条位置项改成同一块盘）在读盘核那一圈被拒、提前返回：
/// 返回之前读到的那一份照样进账。判别力：`crates/mutations.tsv`「可写挂载别的读：读盘核那一圈提前返回不记账」那一行。
#[test]
fn a_release_check_refused_part_way_keeps_what_it_read_before_the_refusal_in_the_count() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, UserChange::CreateSnapshot)
        .expect("建快照");
    session
        .compaction
        .force_the_relocation_batch_after_the_change_refused_not_for_space_for_tests(
            RelocationBatchAfterTheChangeRefusedNotForSpace::ForcedOnEveryAttemptForTests,
        );
    let key_of_the_kept_unit = mapping_key_of_role(
        &current_version_of(&session),
        TransactionUnit::Data(DAMAGED_UNIT),
    );
    let patch = vec![0x5Au8; 64];
    session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_damaged_unit(&patch, 3),
        )
        .expect("部分写");
    let mut forged_version = current_version_of(&session);
    let key_of_the_unit_of_this_version = mapping_key_of_role(
        &forged_version,
        TransactionUnit::Data(DataUnitIndexInFile(0)),
    );
    let locations_of_the_unit_of_this_version =
        mapped_locations(&forged_version, &key_of_the_unit_of_this_version).expect("在映射里");
    damage_the_copy_at(
        &mut pool.devices,
        &locations_of_the_unit_of_this_version[0],
        BYTES_OF_A_DATA_UNIT,
    );
    let kept_locations = &mut forged_version
        .mapping_entries_kept_for_the_snapshots
        .iter_mut()
        .find(|(kept_key, _)| *kept_key == key_of_the_kept_unit)
        .expect("映射条目留存")
        .1;
    kept_locations[1].device = kept_locations[0].device;
    let source_of_the_kept_unit = Placement {
        slot: kept_locations[0].slot,
        span: 2,
    };
    let planned = [
        PlannedRelocation {
            mapping_key: key_of_the_unit_of_this_version,
            source: Placement {
                slot: locations_of_the_unit_of_this_version[0].slot,
                span: 2,
            },
        },
        PlannedRelocation {
            mapping_key: key_of_the_kept_unit,
            source: source_of_the_kept_unit,
        },
    ];
    let pool_parameters = parameters();
    let mut writer = PoolWriter::new(&pool_parameters, pool.devices.as_mut_slice());
    let refused = publish_relocation_batch(
        &mut writer,
        &mut session.allocator,
        &forged_version,
        &planned,
        session.instance,
    );
    assert!(
        matches!(
            refused,
            Err(RelocationBatchRefused::Publish(
                PublishError::MappingEntryLocationsOnTheSameDevice { .. }
            ))
        ),
        "第二项在读盘核那一圈被拒：{refused:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "头一项读盘核读到的那一份进数"
    );
}

/// 后台入口的调用方：钟停在起点，前台没有 fsync 在等。
struct CallerWithoutAForegroundFsync;

impl CallerOfTheBackgroundCompaction for CallerWithoutAForegroundFsync {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(Duration::ZERO)
    }
    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch {
        AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting
    }
}

/// 挂载之后头一次叫后台入口读根环与每条有效根的树表、现行那一版的实例表（`mount::rollback_states_of_the_ring`）：读到只坏一份的实例表片进别的读那个数。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：后台入口读根环不进账」那一行。
#[test]
fn the_ring_read_of_the_background_compaction_counts_the_damaged_instance_table_page() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let table_locations = current_version_of(&session).root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    let _call_outcome_is_not_what_this_case_judges = session
        .run_compaction_in_the_background(
            &mut pool.devices,
            CompactionPacing::BY_DEFAULT,
            &CallerWithoutAForegroundFsync,
        )
        .expect("盘表是挂载的那一份、现行那一版有树");
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
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

/// 清扫判要删哪几行时读现行那一版的实例表（`instance_table_sweep::judge_the_instance_table_sweep`）：根环每一槽都自证过才走到这一读，所以先把环转满。
/// 判通过之后抹头那一步（`erase_the_heads_and_publish_the_sweep`）还要沿同一条链读一遍，账按位置只算一次，判那一读进不进账就看不出来；
/// 所以让判自己停在这一读之后：现行那一版的分配记录树根两份都坏，判读账读不出、一个头都不抹，这一读是唯一读到坏的那一片的读。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：清扫判的时候读实例表不进账」那一行。
#[test]
fn the_instance_table_sweep_reading_the_table_counts_the_damaged_page() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let root_ring_slots = 3 * parameters().geometry.root_ring_slots_per_region.count();
    // 迭代上界是根环的槽数；跨轮不带状态，每一次发一条根。
    for write_time_offset in 0..root_ring_slots {
        session
            .publish_user_change(&mut pool.devices, new_inode_change(3 + write_time_offset))
            .expect("建一个空 inode，转一格根环");
    }
    let version = current_version_of(&session);
    damage_the_copy_at(
        &mut pool.devices,
        &version.root.instance_table.locations[0],
        BYTES_OF_A_DATA_UNIT,
    );
    damage_both_copies_of_the_allocation_record_tree_root(&version, &mut pool.devices);
    let parameters_of_the_pool = parameters();
    let outcome_of_the_sweep = sweep_the_instance_table_while_mounted(
        &parameters_of_the_pool,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
    );
    assert!(
        matches!(
            outcome_of_the_sweep,
            InstanceTableSweepOutcome::NoRowDeleted(
                NoRowDeletedThisRound::AccountOfARingRootUnreadable { .. }
            )
        ),
        "判停在读账，走不到抹头之前那一读：{outcome_of_the_sweep:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
}

/// 两次改动（每次建一个空 inode）之后的会话与头一次改动那一版的根：回退的目标。
fn session_after_two_changes_and_the_rollback_target(
    pool: &mut Pool,
) -> (MountedSession, RollbackTarget, TransactionOutput) {
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let version_of_the_target = current_version_of(&session);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(4))
        .expect("再建一个空 inode");
    (
        session,
        RollbackTarget {
            instance: version_of_the_target.root.instance,
            checkpoint_txg: version_of_the_target.root.checkpoint_txg,
        },
        version_of_the_target,
    )
}

/// 回退释放现行那一版换下的单元之前读盘核（只有现行那一版引用的数据单元）：读到只坏一份进别的读那个数。
/// 回退算 inode 号水位还要重建环里每条根（`mount::highest_inode_number_watermark_in_the_ring`），现行那一版引用的单元它都读，账按位置只算一次，读盘核进不进账就看不出来；
/// 所以把现行那一版的 extent 树根两份都弄坏：重建这条根在读数据单元之前就读不出（水位不算这条根），读盘核是唯一读到坏的那一份数据单元的读。两份都坏的 extent 树根不进这个数。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：回退的释放之前读盘核不进账」那一行。
#[test]
fn the_release_check_of_a_rollback_counts_the_damaged_copy() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let version_of_the_target = current_version_of(&session);
    let patch = vec![0x5Au8; 64];
    session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_damaged_unit(&patch, 4),
        )
        .expect("部分写");
    let mut current = current_version_of(&session);
    let data_locations = mapped_locations(
        &current,
        &mapping_key_of_role(&current, TransactionUnit::Data(DAMAGED_UNIT)),
    )
    .expect("数据单元在映射里");
    let extent_root_locations = mapped_locations(
        &current,
        &mapping_key_of_role(&current, TransactionUnit::ExtentRoot),
    )
    .expect("extent 树根在映射里");
    for location in [&data_locations[0], &extent_root_locations[0]] {
        assert!(
            !version_references_the_location(&version_of_the_target, &pool.devices, location),
            "前提：只有现行那一版引用这个落点：{location:?}"
        );
    }
    damage_the_copy_at(&mut pool.devices, &data_locations[0], BYTES_OF_A_DATA_UNIT);
    for location in &extent_root_locations {
        damage_the_copy_at(&mut pool.devices, location, BYTES_OF_A_NODE);
    }
    roll_back_by_a_forward_publish(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        &mut current,
        RollbackTarget {
            instance: version_of_the_target.root.instance,
            checkpoint_txg: version_of_the_target.root.checkpoint_txg,
        },
    )
    .expect("回退到头一次改动那一版");
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
}

/// 回退重建目标那一版读到只坏一份进别的读那个数。坏的是只有目标那一版引用的记账树根：它不随回退取回（`mount::is_a_unit_taken_from_the_rollback_target`），只有重建读它。
/// 取回的单元（inode 树根这一类）进复活集，坏一份回退就拒（D23（journal 的角色与格式） 已定项 14「在任何写之前拒」，`rollback_by_a_forward_publish` 目标钉着），走不到这个数。
/// 这一条钉回退照样做成、算水位再重建这一版时同一份坏副本不再加次数；重建那一读自己进不进账在下一条用例。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：回退做成时同一份坏副本再读又加次数（重建目标那一版，账上不记）」那一行。
#[test]
fn rebuilding_the_target_of_a_rollback_counts_the_damaged_copy() {
    let mut pool = pool_holding(&content_of_units(3));
    let (mut session, target, version_of_the_target) =
        session_after_two_changes_and_the_rollback_target(&mut pool);
    let mut current = current_version_of(&session);
    let accounting_root_key =
        mapping_key_of_role(&version_of_the_target, TransactionUnit::AccountingTree);
    assert_eq!(
        mapped_locations(&current, &accounting_root_key),
        None,
        "前提：现行那一版不再引用目标那一版的记账树根，回退的别处读不到它"
    );
    let accounting_root_locations =
        mapped_locations(&version_of_the_target, &accounting_root_key).expect("在映射里");
    damage_the_copy_at(
        &mut pool.devices,
        &accounting_root_locations[0],
        BYTES_OF_A_NODE,
    );
    roll_back_by_a_forward_publish(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        &mut current,
        target,
    )
    .expect("回退到头一次改动那一版");
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
}

/// 回退重建目标那一版之后接着读现行那一版的账：账读不出（分配记录树根两份都坏，不进这个数）就拒，走不到算水位那一遍重建，重建目标那一版是唯一读到坏的那一份的读。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：回退重建目标那一版不进账」那一行。
#[test]
fn rebuilding_the_target_of_a_rollback_refused_right_after_it_counts_the_damaged_copy() {
    let mut pool = pool_holding(&content_of_units(3));
    let (mut session, target, version_of_the_target) =
        session_after_two_changes_and_the_rollback_target(&mut pool);
    let mut current = current_version_of(&session);
    let accounting_root_key =
        mapping_key_of_role(&version_of_the_target, TransactionUnit::AccountingTree);
    assert_eq!(
        mapped_locations(&current, &accounting_root_key),
        None,
        "前提：现行那一版不再引用目标那一版的记账树根"
    );
    let accounting_root_locations =
        mapped_locations(&version_of_the_target, &accounting_root_key).expect("在映射里");
    damage_the_copy_at(
        &mut pool.devices,
        &accounting_root_locations[0],
        BYTES_OF_A_NODE,
    );
    damage_both_copies_of_the_allocation_record_tree_root(&current, &mut pool.devices);
    let refused = roll_back_by_a_forward_publish(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        &mut current,
        target,
    )
    .err();
    assert!(
        matches!(
            refused,
            Some(RollbackError::CurrentAccountUnreadable { .. })
        ),
        "重建目标那一版之后读现行那一版的账拒：{refused:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
}

/// 回退读现行那一版的实例表（挂载写行之后那张，目标那一版在上一次挂载，指的是另一张）：读到只坏一份进别的读那个数；回退照样做成，判候选与算水位再读这一片时不再加次数。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：回退做成时同一份坏副本再读又加次数（读现行实例表，判重比不上）」那一行。
#[test]
fn a_rollback_reading_the_current_instance_table_counts_the_damaged_page() {
    let mut pool = pool_holding(&content_of_units(3));
    let version_of_the_target = {
        let mut session = session_of_a_writable_mount(&mut pool.devices);
        session
            .publish_user_change(&mut pool.devices, new_inode_change(3))
            .expect("建一个空 inode");
        current_version_of(&session)
    };
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(4))
        .expect("再建一个空 inode");
    let mut current = current_version_of(&session);
    assert_ne!(
        current.root.instance_table.locations[0].slot,
        version_of_the_target.root.instance_table.locations[0].slot,
        "前提：挂载写行换了实例表"
    );
    let table_locations = current.root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    roll_back_by_a_forward_publish(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        &mut current,
        RollbackTarget {
            instance: version_of_the_target.root.instance,
            checkpoint_txg: version_of_the_target.root.checkpoint_txg,
        },
    )
    .expect("回退到上一次挂载那一版");
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
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

/// 回退读过现行那一版的实例表、判候选时目标不在根环里就拒：算水位那一遍重建走不到，这一读是唯一读到坏的那一片的读。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：回退读现行实例表与判候选不进账」那一行。
#[test]
fn a_rollback_refused_right_after_reading_the_current_instance_table_counts_the_damaged_page() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let table_locations = current_version_of(&session).root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    read_the_current_instance_table_through_a_refused_rollback(&mut session, &mut pool.devices);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
}

/// 这张盘卸载之后不放回字节判真盘：卸载把 F 抬到现行那一版、改动之前的根离开候选集，坏的那一份还在盘上而没有候选根引用它。交回判红的不变量；只录那一趟的交 `None`。
fn violated_invariants_of_the_disk_after_the_unmount(
    table_name: &str,
    session: &mut MountedSession,
    devices: &mut Devices,
) -> Option<Vec<&'static str>> {
    unmount(
        &parameters(),
        devices,
        &mut session.allocator,
        &mut session.current,
        session.shadow_ledger,
    )
    .expect("卸载");
    pool_checker_violations_of_the_image(table_name, &memory_pool_of(devices)).map(|violations| {
        violations
            .into_iter()
            .map(|(invariant, _)| invariant)
            .collect()
    })
}

fn assert_the_disk_after_the_unmount_is_green(
    table_name: &str,
    session: &mut MountedSession,
    devices: &mut Devices,
) {
    if let Some(violated) =
        violated_invariants_of_the_disk_after_the_unmount(table_name, session, devices)
    {
        assert_eq!(violated, Vec::<&str>::new(), "{table_name}：判红的不变量");
    }
}

/// 对照：坏的那一份没修、现行那一版还引用它，卸载之后判真盘报 I-2.1 这一族（这一判有牙）。
#[test]
fn the_disk_after_the_unmount_with_the_data_unit_not_repaired_is_red() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let locations = mapped_locations(
        &version,
        &mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT)),
    )
    .expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    if let Some(violated) = violated_invariants_of_the_disk_after_the_unmount(
        "disk-after-the-unmount-data-unit-not-repaired",
        &mut session,
        &mut pool.devices,
    ) {
        assert!(
            ["I-2.1", "I-4.8", "I-7.4"]
                .iter()
                .any(|invariant| violated.contains(invariant)),
            "没修的那一份照报 I-2.1 这一族：{violated:?}"
        );
    }
}

/// 数据单元经会话读到、下一次改动之后搬走：卸载之后判真盘（弄坏的字节没放回）一条违例都没有。
/// 判别力：`crates/mutations.tsv`「卸载之后判真盘：搬迁那一批不发（坏的那一份现行版本还引用）」那一行。
#[test]
fn the_disk_after_the_unmount_with_the_data_unit_relocated_is_green() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    damage_the_data_unit_and_read_it_through_the_session(&session, &mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    assert_the_disk_after_the_unmount_is_green(
        "disk-after-the-unmount-data-unit-relocated",
        &mut session,
        &mut pool.devices,
    );
}

/// 树表读到只坏一份、下一次改动换下：卸载之后判真盘一条违例都没有。
#[test]
fn the_disk_after_the_unmount_with_the_tree_table_replaced_is_green() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let table_locations = current_version_of(&session).root.tree_table.locations;
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_NODE);
    whole_first_file_read_through_the_session(&session, &pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    assert_the_disk_after_the_unmount_is_green(
        "disk-after-the-unmount-tree-table-replaced",
        &mut session,
        &mut pool.devices,
    );
}

/// 实例表读到只坏一份（挂载建上一版时）、挂载写行就换了：卸载之后判真盘一条违例都没有。
#[test]
fn the_disk_after_the_unmount_with_the_instance_table_replaced_by_the_mount_is_green() {
    let mut pool = pool_holding(&content_of_units(3));
    let table_locations = pool.version.root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    assert_the_disk_after_the_unmount_is_green(
        "disk-after-the-unmount-instance-table-replaced-by-the-mount",
        &mut session,
        &mut pool.devices,
    );
}

/// 映射树叶挂载读到只坏一份、头一次改动之后那一批重写：卸载之后判真盘一条违例都没有。
#[test]
fn the_disk_after_the_unmount_with_the_mapping_tree_leaf_rewritten_is_green() {
    let mut pool = pool_holding(&content_of_units(
        UNITS_OF_A_FILE_WITH_A_MULTI_LEVEL_MAPPING_TREE,
    ));
    let leaf = leftmost_mapping_tree_leaf_of(&pool.version);
    damage_the_copy_at(&mut pool.devices, &leaf.locations[0], BYTES_OF_A_NODE);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    assert_the_disk_after_the_unmount_is_green(
        "disk-after-the-unmount-mapping-tree-leaf-rewritten",
        &mut session,
        &mut pool.devices,
    );
}

/// 这张盘可写挂载被拒在映射 key 集合那一判（`recovery::central_mapping_keys_are_those_of_the_mapped_units_and_the_entries_kept_for_the_snapshots`），盘上逐字节不变。
fn assert_the_writable_mount_refuses_the_key_set_of_the_mapping(devices: &mut Devices) {
    let image_before_the_mount = memory_pool_of(devices);
    match mount_writable(&parameters(), devices) {
        Err(MountError::Recovery(RecoveryFailure::InvariantViolated { invariant, .. })) => {
            assert_eq!(invariant, CENTRAL_MAPPING_KEY_SET_INVARIANT);
        }
        Err(other) => panic!("拒在映射 key 集合那一判，交回 {other:?}"),
        Ok(_) => panic!("映射 key 集合对不上的盘不许挂上"),
    }
    assert_eq!(
        memory_pool_of(devices),
        image_before_the_mount,
        "拒在任何写之前"
    );
}

/// 映射 key 被换（封得对、仍升序）、提示没过期：可写挂载拒。判别力：`crates/mutations.tsv`「映射 key 集合：建上一版不判」「映射 key 集合：只比条数」两行。
#[test]
fn a_disk_with_a_key_replaced_in_the_mapping_is_refused_by_the_writable_mount() {
    let mut pool = pool_holding(&content_of_units(3));
    let key = mapping_key_of_role(
        &pool.version,
        TransactionUnit::Data(LAST_UNIT_IN_THE_KEY_ORDER),
    );
    let locations = mapped_locations(&pool.version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    let replacement = mapping_key_with_the_transaction_number_raised_by_one(&key);
    let version = pool.version.clone();
    replace_the_key_in_the_mapping_root_on_disk(&mut pool.devices, &version, &key, &replacement);
    assert_the_writable_mount_refuses_the_key_set_of_the_mapping(&mut pool.devices);
}

/// 映射 key 被换、提示过期（那一批搬过这个单元，父节点里的提示还指隔离着的旧落点）：不拒的话第一次发布把这把 key 盖回旧落点、搬到的新槽没人引用；可写挂载拒。
/// 判别力：`crates/mutations.tsv`「映射 key 集合：建上一版不判（提示过期那一形）」那一行。
#[test]
fn a_key_replaced_after_a_relocation_is_refused_by_the_writable_mount() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(LAST_UNIT_IN_THE_KEY_ORDER));
    let old_locations = mapped_locations(&version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &old_locations[0], BYTES_OF_A_DATA_UNIT);
    whole_first_file_read_through_the_session(&session, &pool.devices);
    let relocated = session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let (mapping_keys, _, _) = relocated_of(&relocated);
    assert_eq!(mapping_keys, std::slice::from_ref(&key));
    let after = current_version_of(&session);
    drop(session);
    let replacement = mapping_key_with_the_transaction_number_raised_by_one(&key);
    replace_the_key_in_the_mapping_root_on_disk(&mut pool.devices, &after, &key, &replacement);
    assert_the_writable_mount_refuses_the_key_set_of_the_mapping(&mut pool.devices);
}

/// 直接重建同一张盘：拒在同一判。判别力：`crates/mutations.tsv`「映射 key 集合：不等也放行（拒因写反）」那一行。
#[test]
fn rebuilding_a_version_with_a_key_replaced_in_the_mapping_is_refused() {
    let mut pool = pool_holding(&content_of_units(3));
    let key = mapping_key_of_role(
        &pool.version,
        TransactionUnit::Data(LAST_UNIT_IN_THE_KEY_ORDER),
    );
    let replacement = mapping_key_with_the_transaction_number_raised_by_one(&key);
    let version = pool.version.clone();
    let root_on_disk = replace_the_key_in_the_mapping_root_on_disk(
        &mut pool.devices,
        &version,
        &key,
        &replacement,
    );
    let image = memory_pool_of(&pool.devices);
    match rebuild_version(&image, &root_on_disk, Some(version.record.clone())) {
        Err(RebuildVersionFailure::Walk(RecoveryFailure::InvariantViolated {
            invariant, ..
        })) => assert_eq!(invariant, CENTRAL_MAPPING_KEY_SET_INVARIANT),
        other @ (Ok(
            RebuiltVersion::WithFile(_)
            | RebuiltVersion::WithoutFile
            | RebuiltVersion::WithTreesAndNoFile(_),
        )
        | Err(_)) => panic!("拒在映射 key 集合那一判，交回 {other:?}"),
    }
}

/// 读回用的那一版（内存里）映射查不到这把 key：判不出提示过期，按提示读到的只坏一份照计。盘上的这一形可写挂载已经拒了，这一支只有内存里那一版对不上才走得到。
/// 判别力：`crates/mutations.tsv`「查不到 key 改记没坏：部分写读回映射查不到时把真的一份改记没坏」那一行。
#[test]
fn a_read_back_on_a_version_whose_mapping_has_no_entry_for_the_key_still_counts_the_damaged_copy() {
    let mut pool = pool_holding(&content_of_units(3));
    let key = mapping_key_of_role(
        &pool.version,
        TransactionUnit::Data(LAST_UNIT_IN_THE_KEY_ORDER),
    );
    let locations = mapped_locations(&pool.version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    let mut version_with_the_key_replaced = pool.version.clone();
    let replacement = mapping_key_with_the_transaction_number_raised_by_one(&key);
    let forged_root =
        mapping_root_with_the_key_replaced(&version_with_the_key_replaced, &key, &replacement);
    version_with_the_key_replaced
        .units
        .iter_mut()
        .find(|unit| unit.identity == TransactionUnit::MappingTree)
        .expect("三个单元的池映射树根兼叶")
        .bytes = Arc::from(forged_root);
    assert!(
        mapped_locations(&version_with_the_key_replaced, &key).is_none(),
        "前提：查不到这把 key"
    );
    let read_back = read_back_the_old_data_unit_telling_whether_a_copy_failed(
        &memory_pool_of(&pool.devices),
        &version_with_the_key_replaced,
        FIRST_FILE,
        pool.version.inode_record.expect("第一个文件那条记录"),
        LAST_UNIT_IN_THE_KEY_ORDER,
        &parameters().filesystem_identifier,
    )
    .expect("按提示读得回");
    assert_eq!(
        read_back.copies,
        CopiesOfAUnitRead::FromTheOtherCopyAfterOneFailed,
        "查不到 key 判不出提示过期：照计只坏一份"
    );
}

/// 只数（或按 `failing_reads` 读坏）`location` 那一份的读：从交出去那一刻起数。
fn reads_of_the_copy(
    location: &LocationEntry,
    failing_reads: FailingReadsOfARange,
) -> SharedUnreadableRanges {
    SharedUnreadableRanges::new(
        vec![UnreadableRange {
            device: location.device,
            offset_in_bytes: location.slot.to_device_offset().0,
            length_in_bytes: 512,
            failing_reads,
        }],
        UnreadableRangeReadBack::DeviceError,
    )
}

/// 写系统配置槽之前算 F 生效值（`transaction::PoolWriter` 轮换系统配置那一步，每次发布每块盘都走）读最新那条根的实例表：实例表第 0 片弄坏一份，发一次建 inode，
/// 这一读是这次改动里唯一读到那一片的读。进别的读那个数；这次改动另发的那一批整条链重写，旧落点两份隔离；再下一次改动不另发。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：写系统配置槽之前读实例表不进账」「可写挂载别的读：写入口上的账发布之后不转进分配器」两行。
#[test]
fn the_read_before_the_system_configuration_writes_counts_the_damaged_page_and_the_batch_of_that_change_replaces_it(
) {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let table_locations = current_version_of(&session).root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(10))
        .expect("建一个空 inode");
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
    let (mapping_keys, units_replaced, _) = relocated_of(&published);
    assert!(mapping_keys.is_empty());
    assert_eq!(units_replaced, [TransactionUnit::InstanceTable]);
    let after = current_version_of(&session);
    assert_ne!(
        after.root.instance_table.locations[0].slot, table_locations[0].slot,
        "实例表换到新槽"
    );
    assert_eq!(
        quarantined_copies_of(&after, |unit| matches!(
            unit,
            TransactionUnit::InstanceTable
        )),
        device_and_slot_of_both_copies(&table_locations),
        "旧落点两份都隔离"
    );
    let next = session
        .publish_user_change(&mut pool.devices, new_inode_change(11))
        .expect("再建一个空 inode");
    assert_nothing_more_to_relocate(&next);
}

/// 这段历史之后抬 F 抬得动：非空有效根够 4 条。
const CHANGES_BEFORE_THE_FLOOR_RAISE: u64 = 8;

/// 抬 F 那一次的读数（[`floor_raise_with_reads_of_the_instance_table_page`]）。
struct FloorRaiseReadings {
    reads_of_the_page: u64,
    failed_reads: u64,
    other_reads: u64,
    is_raised: bool,
}

/// 抬 F 那一串的写报不报错。
#[derive(Clone, Copy)]
enum WritesOfTheFloorRaise {
    Succeed,
    FailOnEveryDevice,
}

/// 第一个文件 3 个单元、[`CHANGES_BEFORE_THE_FLOOR_RAISE`] 次建 inode 之后抬 F 到上限，现行实例表第 0 片那一份照 `failing_reads` 读坏；`writes` 定抬 F 一开始写报不报错。
fn floor_raise_with_reads_of_the_instance_table_page(
    failing_reads: FailingReadsOfARange,
    writes: WritesOfTheFloorRaise,
) -> FloorRaiseReadings {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    // 迭代上界是这个常量；跨轮不带状态，每一次发一条非空有效根。
    for change in 0..CHANGES_BEFORE_THE_FLOOR_RAISE {
        session
            .publish_user_change(&mut pool.devices, new_inode_change(10 + change))
            .expect("建一个空 inode");
    }
    let page = current_version_of(&session).root.instance_table.locations[0];
    let failing_on_request = pool_failing_on_request(pool.devices);
    failing_on_request.writes_fail.store(
        matches!(writes, WritesOfTheFloorRaise::FailOnEveryDevice),
        Ordering::SeqCst,
    );
    let reads_of_the_page = reads_of_the_copy(&page, failing_reads);
    let mut devices = with_unreadable_ranges(failing_on_request.devices, &reads_of_the_page);
    let PoolVersion::WithFile(current) = &mut session.current else {
        panic!("现行那一版带文件");
    };
    let raise = raise_rollback_floor_to_the_admission_ceiling(
        &parameters(),
        &mut devices,
        &mut session.allocator,
        Arc::make_mut(current),
        ShadowLedger::On,
    );
    FloorRaiseReadings {
        reads_of_the_page: reads_of_the_page.reads_of_range(0),
        failed_reads: reads_of_the_page.failed_reads(),
        other_reads: session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        is_raised: raise.is_ok(),
    }
}

/// 抬 F 那一串读现行实例表的每一读（算上限、读候选集那张表、算 F 生效值、先写系统配置与每次空发布写系统配置槽之前）都进账：哪一读单独读坏一次，别的读那个数都是 1。
/// 先空跑一遍数这一串读了几次，再对每一次各跑一遍。判别力：`crates/mutations.tsv`「可写挂载别的读：准入算上限沿链读实例表不进账」「可写挂载别的读：推抬 F 读实例表读到只坏一份不进账」
/// 「可写挂载别的读：推抬 F 算生效值不进账」三行。
// harness-test-granularity:one-scenario 同一段历史上同一次抬 F，取值只差读坏的是第几读，「每一读都进账」要在同一条用例里数全
#[test]
fn whichever_read_of_the_instance_table_page_fails_while_raising_the_floor_it_is_counted() {
    let dry_run = floor_raise_with_reads_of_the_instance_table_page(
        FailingReadsOfARange::Never,
        WritesOfTheFloorRaise::Succeed,
    );
    assert!(dry_run.is_raised, "前提：抬 F 抬得动");
    assert_eq!(dry_run.other_reads, 0, "一读都没坏就不记");
    assert!(
        dry_run.reads_of_the_page >= 4,
        "前提：这一串读过算上限、候选集那张表、算生效值、写系统配置槽之前：{}",
        dry_run.reads_of_the_page
    );
    // 迭代上界是空跑数到的读次数；跨轮不带状态，每一轮新造一个池。
    for failing_read in 1..=dry_run.reads_of_the_page {
        let readings = floor_raise_with_reads_of_the_instance_table_page(
            FailingReadsOfARange::OnlyTheNth(failing_read),
            WritesOfTheFloorRaise::Succeed,
        );
        assert_eq!(readings.failed_reads, 1, "第 {failing_read} 读读坏了");
        assert!(
            readings.is_raised,
            "第 {failing_read} 读读坏：另一份读得出，抬 F 照样做成"
        );
        assert_eq!(readings.other_reads, 1, "第 {failing_read} 读读坏");
    }
}

/// 先写系统配置那一步写失败、这一串的根一条都不发：写系统配置槽之前那一读同样进账（写入口随错丢掉之前转交）。哪一读单独读坏一次，别的读那个数都是 1。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：抬 F 先写系统配置失败时写入口上的账不转交」那一行。
// harness-test-granularity:one-scenario 同上：取值只差读坏的是第几读
#[test]
fn whichever_read_of_the_instance_table_page_fails_before_a_floor_raise_whose_write_fails_it_is_counted(
) {
    let dry_run = floor_raise_with_reads_of_the_instance_table_page(
        FailingReadsOfARange::Never,
        WritesOfTheFloorRaise::FailOnEveryDevice,
    );
    assert!(!dry_run.is_raised, "前提：写报错，抬 F 做不成");
    assert!(
        dry_run.reads_of_the_page >= 1,
        "前提：写之前读过实例表：{}",
        dry_run.reads_of_the_page
    );
    // 迭代上界是空跑数到的读次数；跨轮不带状态，每一轮新造一个池。
    for failing_read in 1..=dry_run.reads_of_the_page {
        let readings = floor_raise_with_reads_of_the_instance_table_page(
            FailingReadsOfARange::OnlyTheNth(failing_read),
            WritesOfTheFloorRaise::FailOnEveryDevice,
        );
        assert_eq!(readings.failed_reads, 1, "第 {failing_read} 读读坏了");
        assert_eq!(readings.other_reads, 1, "第 {failing_read} 读读坏");
    }
}

/// 抬 F 那一次跑下来的盘、会话与抬之前后的那两版（[`run_with_a_floor_raise`]）。
struct FloorRaiseRun {
    pool: Pool,
    session: MountedSession,
    before: TransactionOutput,
    after: TransactionOutput,
}

/// 400 个单元的文件（分配记录树多片叶）、[`CHANGES_BEFORE_THE_FLOOR_RAISE`] 次建 inode 之后抬 F 到上限，`damaged` 给了就在抬 F 之前弄坏那个分配记录树节点的头一份。
fn run_with_a_floor_raise(damaged: Option<AllocationRecordTreeNode>) -> FloorRaiseRun {
    let mut pool = pool_holding(&content_of_units(
        UNITS_OF_A_FILE_WITH_A_MULTI_LEVEL_MAPPING_TREE,
    ));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    // 迭代上界是这个常量；跨轮不带状态，每一次发一条非空有效根。
    for change in 0..CHANGES_BEFORE_THE_FLOOR_RAISE {
        session
            .publish_user_change(&mut pool.devices, new_inode_change(10 + change))
            .expect("建一个空 inode");
    }
    let before = current_version_of(&session);
    if let Some(node) = damaged {
        let location = before
            .allocation_record_tree
            .pointer_of(node)
            .expect("节点在树里")
            .locations[0];
        damage_the_copy_at(&mut pool.devices, &location, BYTES_OF_A_NODE);
    }
    let PoolVersion::WithFile(current) = &mut session.current else {
        panic!("现行那一版带文件");
    };
    let _raised_floor = raise_rollback_floor_to_the_admission_ceiling(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        Arc::make_mut(current),
        ShadowLedger::On,
    )
    .expect("抬 F 抬得动");
    let after = current_version_of(&session);
    FloorRaiseRun {
        pool,
        session,
        before,
        after,
    }
}

/// 抬 F 重算影子账读每条候选根的账（`mount::isolate_slots_referenced_only_by_abandoned_roots`），现行那一版也在候选里，读的是现行那一版的分配记录树：
/// 取一片抬 F 那一串没动的叶，弄坏头一份。这一读读到，进别的读那个数；下一次改动发成之后现行版本不再引用弄坏的那个落点。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：抬 F 重算影子账读各根的账不进账」那一行。
#[test]
fn the_floor_raise_reading_the_account_of_the_current_version_counts_the_damaged_leaf_and_the_next_change_moves_it(
) {
    let dry_run = run_with_a_floor_raise(None);
    let leaves_the_raise_does_not_touch: BTreeSet<AllocationRecordTreeNode> = dry_run
        .before
        .allocation_record_tree
        .nodes
        .iter()
        .filter(|(node, pointer)| {
            dry_run.after.allocation_record_tree.pointer_of(*node) == Some(*pointer)
        })
        .map(|(node, _)| *node)
        .filter(|node| {
            matches!(
                node,
                AllocationRecordTreeNode::BelowTheRoot(AllocationRecordTreeNodePosition {
                    level: 0,
                    ..
                })
            )
        })
        .collect();
    let node = *leaves_the_raise_does_not_touch
        .iter()
        .next()
        .expect("前提：抬 F 那一串有没动的叶");
    let mut run = run_with_a_floor_raise(Some(node));
    let location = run
        .before
        .allocation_record_tree
        .pointer_of(node)
        .expect("节点在树里")
        .locations[0];
    assert_eq!(
        run.session
            .other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
    run.session
        .publish_user_change(&mut run.pool.devices, new_inode_change(40))
        .expect("再建一个空 inode");
    assert!(
        !version_references_the_location(
            &current_version_of(&run.session),
            &run.pool.devices,
            &location
        ),
        "现行版本不再引用弄坏的那个落点"
    );
    assert_ne!(
        current_version_of(&run.session)
            .allocation_record_tree
            .pointer_of(node),
        run.before.allocation_record_tree.pointer_of(node),
        "那片叶换了落点"
    );
}

/// 可写挂载重建分配器读每条候选根的账：只有更早那一版引用的分配记录树根弄坏一份，只有这一读读到。进别的读那个数；读的是别的版本，下一次改动发成之后那一批认不出、不搬。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：重建分配器读各根的账不进账」那一行。
#[test]
fn the_mount_reading_the_account_of_another_candidate_root_counts_the_damaged_copy_and_nothing_moves(
) {
    let mut pool = pool_holding(&content_of_units(3));
    let (earlier, current) = {
        let mut session = session_of_a_writable_mount(&mut pool.devices);
        session
            .publish_user_change(&mut pool.devices, new_inode_change(3))
            .expect("建一个空 inode");
        let earlier = current_version_of(&session);
        session
            .publish_user_change(&mut pool.devices, new_inode_change(4))
            .expect("再建一个空 inode");
        (earlier, current_version_of(&session))
    };
    let key = mapping_key_of_role(&earlier, TransactionUnit::AllocationTree);
    assert_eq!(
        mapped_locations(&current, &key),
        None,
        "前提：现行那一版不再引用更早那一版的分配记录树根"
    );
    let locations = mapped_locations(&earlier, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_NODE);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    assert_eq!(session.unit_reads_with_one_copy_failing_while_mounting(), 0);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(5))
        .expect("建一个空 inode");
    assert_nothing_more_to_relocate(&published);
}

/// 可写挂载建上一版读到的那一份（现行树表），重建分配器读候选根的账又读到一遍：归挂载那个数，别的读那个数不加。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：重建分配器读到建上一版已读到的那一份又进别的读」那一行。
#[test]
fn a_copy_read_by_building_the_previous_version_is_not_counted_again_by_the_read_of_the_accounts() {
    let mut pool = pool_holding(&content_of_units(3));
    let tree_table_locations = pool.version.root.tree_table.locations;
    damage_the_copy_at(&mut pool.devices, &tree_table_locations[0], BYTES_OF_A_NODE);
    let session = session_of_a_writable_mount(&mut pool.devices);
    assert_eq!(session.unit_reads_with_one_copy_failing_while_mounting(), 1);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        0
    );
}

/// 回退取环里各根的 inode 号水位（`mount::highest_inode_number_watermark_in_the_ring`）：环里一条既不是现行也不是目标的根引用的单元，只有这一读读到。回退照样做成，别的读那个数是 1。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：回退取水位不进账」那一行。
#[test]
fn only_the_watermark_read_of_a_rollback_sees_a_unit_of_a_root_between_the_target_and_the_current_version(
) {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let target = current_version_of(&session);
    let patch_of_the_middle = vec![0x5Au8; 64];
    session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_damaged_unit(&patch_of_the_middle, 4),
        )
        .expect("部分写");
    let middle = current_version_of(&session);
    let middle_locations = mapped_locations(
        &middle,
        &mapping_key_of_role(&middle, TransactionUnit::Data(DAMAGED_UNIT)),
    )
    .expect("在映射里");
    let patch_of_the_current = vec![0xA5u8; 64];
    session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_damaged_unit(&patch_of_the_current, 5),
        )
        .expect("再部分写");
    let mut current = current_version_of(&session);
    damage_the_copy_at(
        &mut pool.devices,
        &middle_locations[0],
        BYTES_OF_A_DATA_UNIT,
    );
    roll_back_by_a_forward_publish(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        &mut current,
        RollbackTarget {
            instance: target.root.instance,
            checkpoint_txg: target.root.checkpoint_txg,
        },
    )
    .expect("回退照样做成");
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "只有水位那一读读到中间那一版的单元"
    );
}

/// 部分写之前那一版是回退目标、部分写把 [`DAMAGED_UNIT`] 换到新槽的会话，连同现行那一版、目标那一版那个单元（回退复活它，逐份验）的两条位置条目。
fn session_after_a_partial_write_and_the_resurrected_unit_of_the_target(
    pool: &mut Pool,
) -> (
    MountedSession,
    TransactionOutput,
    RollbackTarget,
    [LocationEntry; 2],
) {
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let target = current_version_of(&session);
    let patch = vec![0x5Au8; 64];
    session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_damaged_unit(&patch, 4),
        )
        .expect("部分写");
    let current = current_version_of(&session);
    let unit_of_the_target = mapped_locations(
        &target,
        &mapping_key_of_role(&target, TransactionUnit::Data(DAMAGED_UNIT)),
    )
    .expect("在映射里");
    (
        session,
        current,
        RollbackTarget {
            instance: target.root.instance,
            checkpoint_txg: target.root.checkpoint_txg,
        },
        unit_of_the_target,
    )
}

/// 回退重建目标那一版把复活单元读回来：前一条位置条目那一份坏，这一读读到只坏一份进别的读那个数；复活单元那一判逐份读盘、拒在这一份，不经按位置条目读，不再加次数。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：回退重建目标那一版读到复活单元不进账」那一行。
#[test]
fn a_rollback_refused_at_a_resurrected_unit_counts_its_damaged_copy_once_for_rebuilding_the_target()
{
    let mut pool = pool_holding(&content_of_units(3));
    let (mut session, mut current, target, unit_of_the_target) =
        session_after_a_partial_write_and_the_resurrected_unit_of_the_target(&mut pool);
    damage_the_copy_at(
        &mut pool.devices,
        &unit_of_the_target[0],
        BYTES_OF_A_DATA_UNIT,
    );
    let refused = roll_back_by_a_forward_publish(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        &mut current,
        target,
    )
    .err();
    assert!(
        matches!(
            refused,
            Some(RollbackError::ResurrectedUnitCopyUnreadableOrMismatched { .. })
        ),
        "拒在复活单元那一判：{refused:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "重建目标那一版读到它一次，复活单元那一判不进这个数"
    );
}

/// 回退读现行那一版的账（从盘上沿根读一遍分配记录树）：现行那一版的分配记录树根弄坏一份，目标那一版的一个复活单元弄坏后一条位置条目那一份，回退拒在复活单元那一判，走不到别的读。
/// 重建目标那一版读复活单元读到前一条就对上，碰不到后一条（前一条那一份坏时它计一次，`a_rollback_refused_at_a_resurrected_unit_counts_its_damaged_copy_once_for_rebuilding_the_target`）；复活单元那一判不进这个数；两个单元的位置条目不同，一把 key 只计一次也并不到一起。
/// 所以别的读那个数的 1 只能是读现行账记的。判别力：`crates/mutations.tsv`「可写挂载别的读：回退读现行那一版的账不进账」那一行。
#[test]
fn a_rollback_refused_after_reading_the_current_account_counts_its_damaged_root() {
    let mut pool = pool_holding(&content_of_units(3));
    let (mut session, mut current, target, unit_of_the_target) =
        session_after_a_partial_write_and_the_resurrected_unit_of_the_target(&mut pool);
    damage_the_copy_at(
        &mut pool.devices,
        &unit_of_the_target[1],
        BYTES_OF_A_DATA_UNIT,
    );
    let account_root_of_the_current_version = mapped_locations(
        &current,
        &mapping_key_of_role(&current, TransactionUnit::AllocationTree),
    )
    .expect("在映射里");
    damage_the_copy_at(
        &mut pool.devices,
        &account_root_of_the_current_version[0],
        BYTES_OF_A_NODE,
    );
    let refused = roll_back_by_a_forward_publish(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        &mut current,
        target,
    )
    .err();
    assert!(
        matches!(
            refused,
            Some(RollbackError::ResurrectedUnitCopyUnreadableOrMismatched { .. })
        ),
        "拒在复活单元那一判：{refused:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
}

/// 可写挂载末尾那一判数实例表的行（`instance_table_sweep::sweep_the_instance_table_at_the_end_of_the_mount`）：阈值之下不扫，这一读是唯一读到那一片的读。
/// 直接对会话调它：挂载的那一次在写行之后读新表，盘上还没有坏的。判别力：`crates/mutations.tsv`「可写挂载别的读：挂载末尾数实例表行数不进账」那一行。
#[test]
fn the_read_of_the_row_count_at_the_end_of_the_mount_counts_the_damaged_page() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let table_locations = current_version_of(&session).root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    let outcome = sweep_the_instance_table_at_the_end_of_the_mount(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        InstanceTableSweepConfiguration {
            switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
            row_count_threshold: InstanceTableRowCountThreshold(u64::MAX),
            sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
        },
    );
    assert!(
        matches!(
            outcome,
            InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { .. }
        ),
        "阈值之下不扫：{outcome:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
}

/// 落盘中途写失败、这次发布冻结，轮换系统配置那一步没走到；原样重发时写系统配置槽之前那一读读到实例表一份读坏：重发那个写入口上的读进别的读那个数，下一次改动发成之后另发的那一批换掉那一片。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：重发冻结的发布时写入口上的账不转交」那一行。
#[test]
fn the_read_before_the_system_configuration_writes_of_a_resent_publish_counts_the_page_that_fails()
{
    let pool = pool_holding(&content_of_units(3));
    let mut failing = pool_failing_on_request(pool.devices);
    let mounted = mount_writable(&parameters(), &mut failing.devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let page = current_version_of(&session).root.instance_table.locations[0];
    failing.writes_fail.store(true, Ordering::SeqCst);
    let refused = session.publish_user_change(&mut failing.devices, new_inode_change(3));
    assert!(
        matches!(refused, Err(UserChangeRefused::Publish { .. })),
        "落盘中途写失败：{refused:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        0,
        "前提：头一个写就失败，走不到写系统配置槽之前那一读"
    );
    failing.writes_fail.store(false, Ordering::SeqCst);
    *failing
        .copy_whose_reads_fail
        .lock()
        .expect("用例单线程改这个开关，锁没有中毒") =
        Some((page.device, page.slot.to_device_offset().0));
    assert!(
        session
            .resend_the_frozen_publish(&mut failing.devices)
            .expect("写不再失败，重发得成"),
        "有冻结着的那一次"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1
    );
    *failing
        .copy_whose_reads_fail
        .lock()
        .expect("用例单线程改这个开关，锁没有中毒") = None;
    let next = session
        .publish_user_change(&mut failing.devices, new_inode_change(4))
        .expect("建一个空 inode");
    let (_, units_replaced, _) = relocated_of(&next);
    assert_eq!(units_replaced, [TransactionUnit::InstanceTable]);
}

/// 一次可写挂载里读 `location` 那一份的读数：那一份一共读了几次、读坏几次，挂载那个数与别的读那个数。
struct ReadingsOfAMount {
    reads_of_the_copy: u64,
    failed_reads: u64,
    unit_reads_while_mounting: u64,
    other_unit_reads: u64,
}

/// 在 `devices` 的一份拷贝上可写挂载一次，`location` 那一份照 `failing_reads` 读坏（报块设备错，另一份读得出）。
fn writable_mount_with_reads_of_the_copy(
    devices: &Devices,
    location: &LocationEntry,
    failing_reads: FailingReadsOfARange,
) -> ReadingsOfAMount {
    let reads = reads_of_the_copy(location, failing_reads);
    let mut devices_with_the_reads = with_unreadable_ranges(copy_of_the_devices(devices), &reads);
    let mounted = mount_writable(&parameters(), &mut devices_with_the_reads)
        .expect("另一份读得出，可写挂载做得成");
    let (_, session) = MountedSession::of_the_mount(mounted);
    ReadingsOfAMount {
        reads_of_the_copy: reads.reads_of_range(0),
        failed_reads: reads.failed_reads(),
        unit_reads_while_mounting: session.unit_reads_with_one_copy_failing_while_mounting(),
        other_unit_reads: session.other_unit_reads_with_one_copy_failing_in_this_mount(),
    }
}

/// 先不读坏挂一遍数这一次挂载读了那一份几次，再对每一次各挂一遍：哪一读单独读坏一次，挂载那个数与别的读那个数合起来都是 1。
fn assert_every_read_of_the_copy_during_a_writable_mount_is_counted(
    devices: &Devices,
    location: &LocationEntry,
) {
    let dry_run =
        writable_mount_with_reads_of_the_copy(devices, location, FailingReadsOfARange::Never);
    assert_eq!(
        dry_run.unit_reads_while_mounting + dry_run.other_unit_reads,
        0,
        "一读都没坏就不记"
    );
    assert!(dry_run.reads_of_the_copy >= 1, "前提：挂载读过那一份");
    // 迭代上界是空跑数到的读次数；跨轮不带状态，每一轮在一份新拷贝上挂载。
    for failing_read in 1..=dry_run.reads_of_the_copy {
        let readings = writable_mount_with_reads_of_the_copy(
            devices,
            location,
            FailingReadsOfARange::OnlyTheNth(failing_read),
        );
        assert_eq!(readings.failed_reads, 1, "第 {failing_read} 读读坏了");
        assert_eq!(
            readings.unit_reads_while_mounting + readings.other_unit_reads,
            1,
            "第 {failing_read} 读读坏（一共读 {} 次）：挂载那个数 {}、别的读那个数 {}",
            dry_run.reads_of_the_copy,
            readings.unit_reads_while_mounting,
            readings.other_unit_reads
        );
    }
}

/// mkfs 之后头一次可写挂载（树表 0 条那一版，要写的行为空，写行那次是零单元发布）：哪一读单独读坏一次都记 1。这一版独有的一读是取号之前取单元清单（`mount::PreviousVersion::units`，带文件的一版用重建出来的、不读盘）：
/// 它是这次挂载共用的那一份第一次读实例表那一片，之后取号写、零单元发布与暖机写系统配置槽之前读的都取记下的那一份，读不到盘，读到盘的只剩挂载末尾数行数那一读。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：取号之前取树表 0 条那一版的单元清单不进账」那一行。
// harness-test-granularity:one-scenario 同一个池上同一次挂载，取值只差读坏的是第几读，「每一读都进账」要在同一条用例里数全
#[test]
fn whichever_read_of_the_instance_table_page_fails_during_the_mount_of_a_version_without_a_file_it_is_counted(
) {
    let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
    assert_every_read_of_the_copy_during_a_writable_mount_is_counted(
        &devices,
        &genesis.root.instance_table.locations[0],
    );
}

/// 带文件那一版的可写挂载：重建分配器开头读最新那条根的实例表（`mount::instance_table_of_the_newest_root_read_at_most_twice`）与之后读这张表的每一读都进账；
/// 建上一版读好了、只有这一读读坏时归别的读那个数。判别力：`crates/mutations.tsv`「可写挂载别的读：重建分配器读最新那条根的实例表不进账」那一行。
// harness-test-granularity:one-scenario 同上：取值只差读坏的是第几读
#[test]
fn whichever_read_of_the_instance_table_page_fails_during_the_mount_of_a_version_with_a_file_it_is_counted(
) {
    let pool = pool_holding(&content_of_units(3));
    assert_every_read_of_the_copy_during_a_writable_mount_is_counted(
        &pool.devices,
        &pool.version.root.instance_table.locations[0],
    );
}
