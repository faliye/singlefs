//! 验证包模块：crash_point_replay_progress、crash_given_states、crash_amplification
//! 用户改动发成之后另发的那一批换下豁免映射的单元（D4（校验和位置） 已定项 4）：可写挂载的会话上，
//! 映射树最左那片叶读到只坏一份（挂载建上一版时）、实例表第 0 片读到只坏一份（头一次改动的发布写系统配置槽之前）；头一次改动（建一个空 inode）之后另发的那一批头一次被空间拒、推抬 F 之后发成，
//! 同一批从根到那片叶整条重写、实例表整条链重写，第二次改动（再建一个空 inode）不另发。这两次改动连同那一批的每一段按写表展开（少于十个写的段每个真子集一态，别的段按前缀），
//! 整张表交崩溃点重放的给定状态引擎判（录入段写进判定库、判段读回判，配了 GPU 走 GPU 判器，没配由 CPU 判器判）。
//! 弄坏的两份还在盘上、还被回退候选集里的旧根引用时 I-2.1、I-4.8、I-7.4 本来就该红，所以表的基线盘上先把弄坏的字节放回去，放回之后每个状态池级 checker 一条违例都没有；
//! 放回之后 checker 看不见现行版本还引用不引用坏的那一份，由用例里的断言钉住：那一批发完现行版本不再引用弄坏的那两个落点。
//! 录制流的每个前缀各恢复一次（盘上照旧是弄坏的那一份），读回第一个文件的内容；坏盘 × 部分落盘：同一张表上每个持久集合在不放回字节的盘上各恢复一次，同样读回，
//! 再各接着可写挂载一次：挂上的发一次改动、卸载，不放回字节判真盘；拒的判拒绝本身合不合法。
//! 那一批发完卸载，不放回字节判真盘（带不修的对照）。
//!
//! **比已有的流多罩了什么**：已有的流上映射树根之下的节点只在路上的 key 进出、搬迁时才重写，实例表只在写行、回退、清扫时才整条链重写。这里头一次枚举这几样：
//! 只为换落点、条目一个不变的映射树节点重写；行一个不变的实例表整条链重写；两者旧落点带隔离记号（一份已坏、两份一起隔离）落进分配记录。
//! 多跑的步：会话另发的这一批（新的一批发布，不是新的提交步骤，可写挂载的分段不变：`crates/singlefs-verification/src/writable_mount_phases.rs` 不动）；
//! 那一批写了一半、坏的两份还在盘上时接着走可写挂载、改动与卸载（已有的流上可写挂载的起点盘都没有弄坏的副本）。

mod common;

use std::num::NonZeroU64;

use common::{
    build_pool, geometry, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_core::address::{DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::code_two_tree::CodeTwoTreeNodeContents;
use singlefs_core::compaction_plan::RelocationBatchAfterTheChangeRefusedForSpace;
use singlefs_core::mount::{mount_writable, unmount};
use singlefs_core::mounted_session::{
    MountedSession, RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing, UserChange,
};
use singlefs_core::pointer::LocationEntry;
use singlefs_core::recovery::{
    instance_table_chain_of_root, recover, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::transaction::{
    central_mapping_locations_in_the_version, publish_sequential_write, FirstFile, MappingLookup,
    PoolWriter, TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_verification::crash_amplification::{
    one_test_changes_the_environment_at_a_time, GivenStatesEngine, GivenStatesItem,
};
use singlefs_verification::crash_given_states::{
    GivenStateConclusion, GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
};
use singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_verification::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_verification::mount_transparency::{
    judge_writable_mount_refusal_of_a_crash_state_of_the_pool,
    selected_version_of_the_pool_on_the_crash_image, DiskAfterTheRefusal,
    ReadFaultsInTheCrashState, ReadFaultsUnderTheRefusedMount, ReadOnlyReadBackOfTheCrashImage,
    WritableMountRefusalVerdict,
};
use singlefs_verification::RetainedOperation;

const BYTES_OF_A_NODE: usize = 16384;
const BYTES_OF_A_DATA_UNIT: usize = 32768;
const FLIPPED_BYTE_IN_THE_UNIT: usize = 4000;
/// 让中央映射树多层的文件有几个数据单元：一片映射叶约 290 条（16384 字节、条目 55 字节）。
const UNITS_OF_THE_FILE: usize = 400;
/// 少于这么多个写的段按每个真子集展开（同崩溃枚举用例「短段」的口径）。
const WRITES_OF_A_SEGMENT_EXPANDED_BY_SUBSET_BELOW: usize = 10;

fn content_of_the_file() -> Vec<u8> {
    let payload_capacity = data_unit_payload_capacity();
    (0..UNITS_OF_THE_FILE * payload_capacity - 1000)
        .map(|index| u8::try_from((index * 37 + 11) % 253).expect("小于 256"))
        .collect()
}

/// 弄坏一份时记下的：表的基线盘上把那个字节放回去要用（[`put_the_damaged_bytes_back_on_the_baseline`]）。
struct DamagedCopy {
    location: LocationEntry,
    unit_bytes: usize,
    byte_before_the_damage: u8,
}

/// 把 `location` 那块盘上那一份改一个字节（这一个写进录制流、落在被枚举的那几段之前，是基线的一部分），另一份不动。
fn damage_the_copy_at(
    devices: &mut [(DeviceIdentity, Recorded)],
    location: &LocationEntry,
    unit_bytes: usize,
) -> DamagedCopy {
    let (_, device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == location.device)
        .expect("池里有这块盘");
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

/// 表的基线盘上把弄坏的字节放回去。每一份都要放得回去：基线上那一槽还是弄坏时的内容，被枚举的那几段（`writes`）也没有一个写落在那一槽上，即那一槽没被复用过。
fn put_the_damaged_bytes_back_on_the_baseline(
    base: &mut MemoryPool,
    writes: &[RetainedWrite],
    damaged: &[DamagedCopy],
) {
    for damaged_copy in damaged {
        let location = damaged_copy.location;
        let unit_starts_at = location.slot.to_device_offset();
        let unit_ends_at =
            unit_starts_at.0 + u64::try_from(damaged_copy.unit_bytes).expect("单元字节数");
        assert!(
            !writes.iter().any(|write| write.device == location.device
                && write.offset.0 < unit_ends_at
                && unit_starts_at.0 < write.offset.0 + write.length_in_bytes()),
            "被枚举的那几段没有写落在弄坏的那一槽上：{location:?}"
        );
        let device = base.devices.get_mut(&location.device).expect("有这块盘");
        let mut unit = device.read(unit_starts_at, damaged_copy.unit_bytes);
        unit[FLIPPED_BYTE_IN_THE_UNIT] = damaged_copy.byte_before_the_damage;
        assert_eq!(
            crc32_castagnoli(&unit),
            location.unit_checksum,
            "放回之后那一份与位置条目对得上（基线上那一槽没被复用过）：{location:?}"
        );
        device.write(unit_starts_at, &unit);
    }
}

/// `version` 这一版经哪几条位置条目引用单元：根记录直接持有的树表与映射树根、中央映射树每个节点、实例表链每一片、每个进映射的单元、留存给快照的映射条目。
fn locations_the_version_references(
    version: &TransactionOutput,
    devices: &[(DeviceIdentity, Recorded)],
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
    for (role, key) in &version.mapped_units {
        match central_mapping_locations_in_the_version(version, key) {
            MappingLookup::Found(mapped_locations) => locations.extend(mapped_locations),
            lookup @ (MappingLookup::NodeMalformedOrNoEntryWithThisKey
            | MappingLookup::EntryNarrowerThanItsFieldTable { .. }) => {
                panic!("进映射的 {role:?} 在这一版的映射里，查到 {lookup:?}")
            }
        }
    }
    for (_, kept_locations) in &version.mapping_entries_kept_for_the_snapshots {
        locations.extend(*kept_locations);
    }
    locations
}

/// `version` 这一版引用不引用 `location` 那个落点（同一块盘的同一槽）。
fn version_references_the_location(
    version: &TransactionOutput,
    devices: &[(DeviceIdentity, Recorded)],
    location: &LocationEntry,
) -> bool {
    locations_the_version_references(version, devices)
        .iter()
        .any(|referenced| referenced.device == location.device && referenced.slot == location.slot)
}

/// 第一个文件整份顺序写成 [`UNITS_OF_THE_FILE`] 个单元（映射树多层）之后的池，第二个实例。
fn pool_with_a_multi_level_mapping_tree(tag: &str, content: &[u8]) -> BuiltPool {
    let mut pool = build_pool(tag);
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let output = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_sequential_write(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile {
                content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            InstanceGeneration(1),
        )
        .expect("顺序写")
    };
    pool.output = output;
    pool
}

/// 那一批换下的豁免映射的单元；不是发成的结局 panic。
fn units_replaced_by_the_batch(
    outcome: &RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing,
) -> &[TransactionUnit] {
    match outcome {
        RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::Relocated {
            units_exempt_from_the_mapping_replaced,
            ..
        } => units_exempt_from_the_mapping_replaced,
        other @ (RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NoSpaceAfterRaisingTheFloor {
            ..
        }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace { .. }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::FloorRaiseFailedWhilePushingForSpace {
            ..
        }) => panic!("应另发一批并发成，交回 {other:?}"),
    }
}

/// 各段的持久集合：前面的段全持久，这一段少于 [`WRITES_OF_A_SEGMENT_EXPANDED_BY_SUBSET_BELOW`] 个写的每个真子集一态，别的按前缀；再加全持久那一态。
fn persisted_sets_of_the_segments(writes: usize, segments: &[Vec<usize>]) -> Vec<Vec<bool>> {
    let mut persisted_sets: Vec<Vec<bool>> = Vec::new();
    let mut persisted_before_the_segment = vec![false; writes];
    // 迭代上界是段数；跨轮携带的是前面的段全持久那一份集合。
    for segment in segments {
        if segment.len() < WRITES_OF_A_SEGMENT_EXPANDED_BY_SUBSET_BELOW {
            for subset in 0..(1usize << segment.len()) - 1 {
                let mut persisted = persisted_before_the_segment.clone();
                for (position, write) in segment.iter().enumerate() {
                    persisted[*write] = subset & (1 << position) != 0;
                }
                persisted_sets.push(persisted);
            }
        } else {
            for prefix in 0..segment.len() {
                let mut persisted = persisted_before_the_segment.clone();
                for write in &segment[..prefix] {
                    persisted[*write] = true;
                }
                persisted_sets.push(persisted);
            }
        }
        for write in segment {
            persisted_before_the_segment[*write] = true;
        }
    }
    persisted_sets.push(persisted_before_the_segment);
    persisted_sets
}

/// 基线（`operations_before` 之前的盘，把 `damaged` 那几份的字节放回去）上后面那几段的每个状态（[`persisted_sets_of_the_segments`]）的池级 checker 违例，交崩溃点重放的给定状态引擎两段判。
/// 只录那一趟、表归别的机器判的那一趟交 `None`。
fn pool_checker_violations_of_the_states(
    operations: &[RetainedOperation],
    operations_before: usize,
    damaged: &[DamagedCopy],
) -> Option<Vec<Vec<(&'static str, String)>>> {
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    base.apply(&operations[..operations_before]);
    let (writes, segments, _stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations[operations_before..], &geometry());
    put_the_damaged_bytes_back_on_the_baseline(&mut base, &writes, damaged);
    let persisted_sets = persisted_sets_of_the_segments(writes.len(), &segments);
    let states = persisted_sets.len();
    let engine = {
        let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
        GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: format!(
                    "crash_points_of_the_repair_of_the_units_exempt_from_the_mapping::states={states}"
                ),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
    };
    let judged = engine
        .judge_the_given_states(&GivenStatesOfATable {
            table_name: "every-state-of-the-repair-of-the-units-exempt-from-the-mapping"
                .to_string(),
            base: &base,
            writes: &writes,
            segments: &segments,
            seams: &[],
            persisted_sets,
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
    assert_eq!(judged.states.len(), states, "判器逐状态交回结论");
    Some(
        judged
            .states
            .iter()
            .map(GivenStateConclusion::pool_checker_violations)
            .collect(),
    )
}

/// 弄坏映射树最左那片叶一份（挂载读到）、实例表第 0 片一份（头一次改动的发布读到），发那两次改动，钉住头一次改动另发的那一批换下了什么。
/// 交回会话、录着的盘、弄坏的两份、这两次改动之前录到第几个操作。
fn pool_with_the_batch_replacing_the_exempt_units_published(
    pool: &mut BuiltPool,
) -> (
    MountedSession,
    Vec<(DeviceIdentity, Recorded)>,
    [DamagedCopy; 2],
    usize,
) {
    let mut devices = pool.reopen_recorded();
    // 弄坏之前留下非空有效根：不足 4 条时上限是最旧的有效根，F 抬不动，「被空间拒之后抬 F」那几笔写走不到。
    let version_before_the_damage = {
        let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
        let (_, mut session): (_, MountedSession) = MountedSession::of_the_mount(mounted);
        for write_time_offset in [61, 62] {
            session
                .publish_user_change(
                    &mut devices,
                    UserChange::NewInodes {
                        count: 1,
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS + write_time_offset,
                    },
                )
                .expect("建一个空 inode，留下一条非空有效根");
        }
        session
            .current
            .file_version()
            .expect("现行那一版带文件")
            .clone()
    };
    let mapping_tree = &version_before_the_damage.central_mapping_tree;
    assert!(mapping_tree.shape.nodes().len() > 1, "前提：映射树多层");
    let leaf = mapping_tree
        .shape
        .nodes()
        .iter()
        .zip(&mapping_tree.pointers)
        .find(|(node, _)| matches!(node.contents, CodeTwoTreeNodeContents::Leaf { .. }))
        .map(|(_, pointer)| *pointer)
        .expect("多层的树有叶");
    let damaged_leaf_copy = damage_the_copy_at(&mut devices, &leaf.locations[0], BYTES_OF_A_NODE);
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session): (_, MountedSession) = MountedSession::of_the_mount(mounted);
    let version_before_the_batches: &TransactionOutput =
        session.current.file_version().expect("现行那一版带文件");
    let table_locations = version_before_the_batches.root.instance_table.locations;
    for location in [&leaf.locations[0], &table_locations[0]] {
        assert!(
            version_references_the_location(version_before_the_batches, &devices, location),
            "前提：那两批之前现行版本引用这个落点：{location:?}"
        );
    }
    let damaged_table_copy =
        damage_the_copy_at(&mut devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    session
        .compaction
        .force_the_relocation_batch_after_the_change_refused_for_space_for_tests(
            RelocationBatchAfterTheChangeRefusedForSpace::ForcedForTheNextAttemptsForTests {
                attempts: NonZeroU64::new(1).expect("拒一次：1 不是 0"),
            },
        );
    let operations_before = pool.retained_operations().len();
    let first_change = session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 63,
            },
        )
        .expect("建一个空 inode");
    let units_of_the_first_batch = units_replaced_by_the_batch(
        &first_change.relocation_after_the_change_of_the_units_read_with_one_copy_failing,
    );
    assert!(
        matches!(
            units_of_the_first_batch,
            [
                TransactionUnit::MappingTreeNodeBelowTheRoot(_),
                TransactionUnit::InstanceTable
            ]
        ),
        "那一批换下那片叶与实例表（一片）：{units_of_the_first_batch:?}"
    );
    let second_change = session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 64,
            },
        )
        .expect("再建一个空 inode");
    assert!(
        matches!(
            second_change.relocation_after_the_change_of_the_units_read_with_one_copy_failing,
            RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded
        ),
        "第二次改动不另发：{:?}",
        second_change.relocation_after_the_change_of_the_units_read_with_one_copy_failing
    );
    let version_after_the_batches: &TransactionOutput =
        session.current.file_version().expect("现行那一版带文件");
    assert_ne!(
        version_after_the_batches.root.instance_table.locations[0].slot, table_locations[0].slot,
        "实例表换到新槽"
    );
    assert!(
        !version_references_the_location(version_after_the_batches, &devices, &leaf.locations[0]),
        "那片叶换了落点：现行版本不再引用弄坏的那个落点"
    );
    assert!(
        !version_references_the_location(version_after_the_batches, &devices, &table_locations[0]),
        "现行版本不再引用弄坏的那个实例表落点"
    );
    (
        session,
        devices,
        [damaged_leaf_copy, damaged_table_copy],
        operations_before,
    )
}

/// 不放回字节的基线上那几段的每个持久集合（同 [`pool_checker_violations_of_the_states`] 那张表）各恢复一次：每个都读回 `content`。按盘的副本切到几条线程上（各状态互不相干）。
fn assert_every_persisted_set_recovers_the_file_on_the_disk_with_the_damage_still_on_it(
    operations: &[RetainedOperation],
    operations_before: usize,
    content: &[u8],
) {
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    base.apply(&operations[..operations_before]);
    let (writes, segments, _stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations[operations_before..], &geometry());
    let persisted_sets = persisted_sets_of_the_segments(writes.len(), &segments);
    let threads = singlefs_verification::heavy_threads::default_heavy_threads()
        .get()
        .min(persisted_sets.len());
    std::thread::scope(|scope| {
        // 迭代上界是线程数；每条线程判一段互不相交的持久集合，跨线程不带状态。
        for persisted_sets_of_the_thread in
            persisted_sets.chunks(persisted_sets.len().div_ceil(threads))
        {
            let (base, writes) = (&base, &writes);
            scope.spawn(move || {
                // 迭代上界是这一段的持久集合数；跨轮不带状态，每个状态从基线起。
                for persisted in persisted_sets_of_the_thread {
                    let mut image = base.clone();
                    let persisted_writes: Vec<RetainedWrite> = writes
                        .iter()
                        .zip(persisted)
                        .filter(|(_, is_persisted)| **is_persisted)
                        .map(|(write, _)| write.clone())
                        .collect();
                    image.apply_writes(&persisted_writes);
                    let report = recover(&image, JournalPolicy::Consult);
                    match &report.outcome {
                        RecoveryOutcome::FileRead {
                            content: content_read,
                            ..
                        } => assert_eq!(
                            content_read.as_slice(),
                            content,
                            "持久集合 {persisted:?}：读回第一个文件的内容"
                        ),
                        other @ (RecoveryOutcome::NoFile { .. }
                        | RecoveryOutcome::Failed { .. }) => {
                            panic!("持久集合 {persisted:?}：恢复要读回文件，交回 {other:?}")
                        }
                    }
                }
            });
        }
    });
}

/// 恢复的循环串行：镜像就地往前施加（前缀一个接一个），各前缀共用一份盘字节，切片要各建一份、每片从基线施加起，省不下时间；状态判定那一半在引擎里按它的切片并行。
// harness-test-granularity:one-scenario 同一条录制流上的两次改动与它们另发的两批，表里每个状态都是这一条流的崩溃点，不是新场景
// crash-case-check:not-a-crash-case 一个合成流、单跑几秒：这一批写的前缀与短段子集交给定状态引擎，不调枚举一族函数、不经块判定库
#[test]
fn every_crash_point_of_the_batches_replacing_the_units_exempt_from_the_mapping_recovers_the_file_and_is_green(
) {
    let content = content_of_the_file();
    let mut pool = pool_with_a_multi_level_mapping_tree(
        "crash-points-of-the-repair-of-the-units-exempt-from-the-mapping",
        &content,
    );
    let (session, devices, damaged, operations_before) =
        pool_with_the_batch_replacing_the_exempt_units_published(&mut pool);
    let operations = pool.retained_operations();
    drop((session, devices));
    let Some(violations_of_each_state) =
        pool_checker_violations_of_the_states(&operations, operations_before, &damaged)
    else {
        return;
    };
    for (state, violations) in violations_of_each_state.iter().enumerate() {
        assert!(
            violations.is_empty(),
            "状态 {state}：池级 checker 一条违例都没有：{violations:?}"
        );
    }
    let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    image.apply(&operations[..operations_before]);
    // 迭代上界是这两次改动与两批的写数；跨轮携带的只有往前施加的那一份盘。
    for prefix in operations_before..=operations.len() {
        if prefix > operations_before {
            image.apply(&operations[prefix - 1..prefix]);
        }
        let report = recover(&image, JournalPolicy::Consult);
        match &report.outcome {
            RecoveryOutcome::FileRead {
                content: content_read,
                ..
            } => assert_eq!(
                content_read, &content,
                "前缀 {prefix}：读回第一个文件的内容（建空 inode 与换落点都不动它）"
            ),
            other @ (RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. }) => {
                panic!("前缀 {prefix}：恢复要读回文件，交回 {other:?}")
            }
        }
    }
}

/// 坏盘 × 部分落盘：那一批写了一半、坏的那两份还在盘上（不放回字节）：表里每个持久集合恢复都读回第一个文件（读回落在另一份上）。可写挂载在下一条用例。
/// 判别力：`crates/mutations.tsv`「坏盘 × 部分落盘：读到头一份坏就当读不出」那一行。
// harness-test-granularity:one-scenario 同一条录制流上那两批的每个持久集合都是这一条流的崩溃点，不是新场景
// crash-case-check:not-a-crash-case 这一批写的短段子集与前缀交恢复读回，不调枚举一族函数、不造 CrashImage、不经块判定库
#[test]
fn every_persisted_set_of_the_batches_recovers_the_file_on_the_disk_with_the_damage_still_on_it() {
    let content = content_of_the_file();
    let mut pool = pool_with_a_multi_level_mapping_tree(
        "persisted-sets-of-the-repair-on-the-disk-with-the-damage",
        &content,
    );
    let (session, devices, _damaged, operations_before) =
        pool_with_the_batch_replacing_the_exempt_units_published(&mut pool);
    let operations = pool.retained_operations();
    drop((session, devices));
    assert_every_persisted_set_recovers_the_file_on_the_disk_with_the_damage_still_on_it(
        &operations,
        operations_before,
        &content,
    );
}

/// 一个持久集合的盘接着可写挂载走到哪：挂上的发一次改动（挂载读到的那片叶随它另发的那一批换下）再卸载，交回卸载之后的盘；拒的判拒绝本身合法（`mount_transparency`，U6 与 C670 那几条），
/// 这张盘注入过读坏（弄坏的两份），交 `None`。挂上之后经会话读第一个文件，读回 `content`。
fn disk_after_the_writable_mount_change_and_unmount_of_a_persisted_set(
    image: &MemoryPool,
    content: &[u8],
    persisted: &[bool],
) -> Option<MemoryPool> {
    let publish_parameters = parameters();
    let physical_block_size =
        PhysicalBlockSizeInBytes(publish_parameters.geometry.physical_block_size);
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device =
                SparseBlockDevice::new(image.device_size_in_bytes, physical_block_size);
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect();
    let pool_of = |devices_after_the_run: Vec<(DeviceIdentity, SparseBlockDevice)>| MemoryPool {
        devices: devices_after_the_run
            .into_iter()
            .map(|(identity, device)| (identity, device.image))
            .collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    };
    let mounted = match mount_writable(&publish_parameters, &mut devices) {
        Ok(mounted) => mounted,
        Err(refusal) => {
            let after = pool_of(devices);
            let verdict = judge_writable_mount_refusal_of_a_crash_state_of_the_pool(
                image,
                &publish_parameters.filesystem_identifier,
                selected_version_of_the_pool_on_the_crash_image(
                    image,
                    &publish_parameters.filesystem_identifier,
                ),
                &refusal,
                if after == *image {
                    DiskAfterTheRefusal::ByteForByteUnchanged
                } else {
                    DiskAfterTheRefusal::Changed
                },
                ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
                ReadFaultsInTheCrashState::Injected,
                ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults,
            );
            assert!(
                matches!(verdict, WritableMountRefusalVerdict::Legal(_)),
                "持久集合 {persisted:?}：拒绝不合法：{verdict:?}，拒因 {refusal:?}"
            );
            return None;
        }
    };
    let (_, mut session): (_, MountedSession) = MountedSession::of_the_mount(mounted);
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 70,
            },
        )
        .unwrap_or_else(|refusal| panic!("持久集合 {persisted:?}：改动被拒：{refusal:?}"));
    let read = session
        .read_the_file(
            &devices,
            InodeNumber(FIRST_INODE_NUMBER),
            FileOffsetInBytes(0),
            u64::try_from(content.len()).expect("文件字节数"),
        )
        .unwrap_or_else(|refusal| panic!("持久集合 {persisted:?}：读文件被拒：{refusal:?}"));
    assert_eq!(
        read.bytes, content,
        "持久集合 {persisted:?}：经会话读回第一个文件的内容"
    );
    unmount(
        &publish_parameters,
        &mut devices,
        &mut session.allocator,
        &mut session.current,
        session.shadow_ledger,
    )
    .unwrap_or_else(|refusal| panic!("持久集合 {persisted:?}：卸载被拒：{refusal:?}"));
    Some(pool_of(devices))
}

/// 坏盘 × 部分落盘接着可写挂载：那一批写了一半、坏的那两份还在盘上（不放回字节）。判法选「卸载之后判真盘」：挂载、改动与卸载之后那两份的槽可能已复用，放回字节放不回去，
/// 而卸载之后 F 抬到现行那一版、弄坏之前的根离开候选集，真盘上一条违例都没有才说明挂载读到的坏副本都换下了。
/// 每个持久集合都放行（弄坏的两份落在落后那块盘 tail 之前，不被 tail 之后的记录点名，D18（块里携带什么信息） 已定项 11），放行的每张盘都走到判真盘：
/// 这一趟判的、留给别的机器或判段那一趟的各几张打在 `DISKS_AFTER_THE_UNMOUNT` 那一行，两数之和对不上放行的张数判红。
/// 判别力：`crates/mutations.tsv`「坏盘 × 部分落盘接着可写挂载：建上一版读到的那几组不记下（映射树叶不换）」与「落后支缺单元：不看 tail 之后的记录点名（窗口之外缺的照拒）」那两行。
// harness-test-granularity:one-scenario 同一条录制流上那一批的每个持久集合都是这一条流的崩溃点，不是新场景
// crash-case-check:not-a-crash-case 这一批写的短段子集与前缀交可写挂载与给定状态引擎，不调枚举一族函数、不造 CrashImage、不经块判定库
#[test]
fn every_persisted_set_of_the_batches_on_the_disk_with_the_damage_still_on_it_mounts_writable_and_is_green_after_the_unmount(
) {
    let content = content_of_the_file();
    let mut pool = pool_with_a_multi_level_mapping_tree(
        "persisted-sets-of-the-repair-then-a-writable-mount",
        &content,
    );
    let (session, devices, _damaged, operations_before) =
        pool_with_the_batch_replacing_the_exempt_units_published(&mut pool);
    let operations = pool.retained_operations();
    drop((session, devices));
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    base.apply(&operations[..operations_before]);
    let (writes, segments, _stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations[operations_before..], &geometry());
    let persisted_sets = persisted_sets_of_the_segments(writes.len(), &segments);
    let states = persisted_sets.len();
    let threads = singlefs_verification::heavy_threads::default_heavy_threads()
        .get()
        .min(states);
    let disks_after_the_unmount: Vec<Option<MemoryPool>> = std::thread::scope(|scope| {
        let (base, writes, content) = (&base, &writes, &content);
        // 迭代上界是线程数；每条线程走一段互不相交的持久集合，跨线程不带状态，结果按段的次序接起来。
        let handles: Vec<_> = persisted_sets
            .chunks(states.div_ceil(threads))
            .map(|persisted_sets_of_the_thread| {
                scope.spawn(move || {
                    persisted_sets_of_the_thread
                        .iter()
                        .map(|persisted| {
                            let mut image = base.clone();
                            let persisted_writes: Vec<RetainedWrite> = writes
                                .iter()
                                .zip(persisted)
                                .filter(|(_, is_persisted)| **is_persisted)
                                .map(|(write, _)| write.clone())
                                .collect();
                            image.apply_writes(&persisted_writes);
                            disk_after_the_writable_mount_change_and_unmount_of_a_persisted_set(
                                &image, content, persisted,
                            )
                        })
                        .collect::<Vec<Option<MemoryPool>>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("线程里的断言照 panic 传上来"))
            .collect()
    });
    let states_refused = disks_after_the_unmount
        .iter()
        .filter(|disk| disk.is_none())
        .count();
    assert_eq!(
        states_refused, 0,
        "每个持久集合都放行：缺的都不被 tail 之后的记录点名"
    );
    let engine = {
        let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
        GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: format!(
                    "crash_points_of_the_repair_of_the_units_exempt_from_the_mapping::writable-mount-of-the-persisted-sets={states}"
                ),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
    };
    let mut disks_judged_on_this_host = 0usize;
    let mut disks_left_to_another_host_or_the_judging_stage = 0usize;
    // 迭代上界是挂得上的状态数；跨轮携带的只有这两个数，每张盘一张写表为空的表。
    for (state, disk) in disks_after_the_unmount.iter().enumerate() {
        let Some(disk) = disk else {
            continue;
        };
        let judged = engine
            .judge_the_given_states(&GivenStatesOfATable {
                table_name: format!("writable-mount-after-the-persisted-set-{state}"),
                base: disk,
                writes: &[],
                segments: &[],
                seams: &[],
                persisted_sets: vec![Vec::new()],
            })
            .unwrap_or_else(|reason| panic!("给定状态表判不了（不退回别处判）：{reason}"));
        match judged.source {
            GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost => {
                disks_judged_on_this_host += 1;
            }
            GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost
            | GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => {
                disks_left_to_another_host_or_the_judging_stage += 1;
                continue;
            }
        }
        assert_eq!(
            judged.states.len(),
            1,
            "持久集合 {state}：写表为空的表只有一个状态，判器交回它的结论"
        );
        for conclusion in &judged.states {
            let violations = conclusion.pool_checker_violations();
            assert!(
                violations.is_empty(),
                "持久集合 {state} 接着可写挂载、改动、卸载之后判真盘：{violations:?}"
            );
        }
    }
    assert_eq!(
        engine.finish().red_items(),
        Vec::<String>::new(),
        "GPU 判不了的状态判红"
    );
    println!(
        "DISKS_AFTER_THE_UNMOUNT states={states} refused={states_refused} judged_on_this_host={disks_judged_on_this_host} left_to_another_host_or_the_judging_stage={disks_left_to_another_host_or_the_judging_stage}"
    );
    assert_eq!(
        disks_judged_on_this_host + disks_left_to_another_host_or_the_judging_stage,
        states - states_refused,
        "放行的每张盘都走到判真盘"
    );
}

/// 这张盘（基线是它、写表为空）上池级 checker 判红的不变量，交崩溃点重放的给定状态引擎判；只录那一趟的交 `None`。
fn violated_invariants_of_the_image(
    table_name: &str,
    image: &MemoryPool,
) -> Option<Vec<&'static str>> {
    let engine = {
        let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
        GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: format!(
                    "crash_points_of_the_repair_of_the_units_exempt_from_the_mapping::{table_name}"
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
        .map(|violations| {
            violations
                .into_iter()
                .map(|(invariant, _)| invariant)
                .collect()
        })
}

/// 那一批发完卸载（F 抬到现行那一版，弄坏之前的根离开候选集），不放回字节判真盘：一条违例都没有。判的是用户手里那张盘。
/// 判别力：`crates/mutations.tsv`「卸载之后判真盘：映射树节点不进那一批（叶还在坏的落点）」那一行。
#[test]
fn the_disk_after_the_batches_and_the_unmount_with_the_damage_still_on_it_is_green() {
    let content = content_of_the_file();
    let mut pool =
        pool_with_a_multi_level_mapping_tree("disk-after-the-batches-and-the-unmount", &content);
    let (mut session, mut devices, _damaged, _operations_before) =
        pool_with_the_batch_replacing_the_exempt_units_published(&mut pool);
    unmount(
        &parameters(),
        &mut devices,
        &mut session.allocator,
        &mut session.current,
        session.shadow_ledger,
    )
    .expect("卸载");
    let operations = pool.retained_operations();
    drop(devices);
    let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    image.apply(&operations);
    if let Some(violated) =
        violated_invariants_of_the_image("disk-after-the-batches-and-the-unmount", &image)
    {
        assert_eq!(violated, Vec::<&str>::new(), "判红的不变量");
    }
}

/// 对照：同一条流、映射树叶弄坏一份而不发改动（没人换它），卸载之后判真盘报 I-2.1 这一族：这一判有牙。
#[test]
fn the_disk_after_the_unmount_with_the_damaged_leaf_not_repaired_is_red() {
    let content = content_of_the_file();
    let mut pool =
        pool_with_a_multi_level_mapping_tree("disk-after-the-unmount-leaf-not-repaired", &content);
    let mut devices = pool.reopen_recorded();
    let mapping_tree = &pool.output.central_mapping_tree;
    let leaf = mapping_tree
        .shape
        .nodes()
        .iter()
        .zip(&mapping_tree.pointers)
        .find(|(node, _)| matches!(node.contents, CodeTwoTreeNodeContents::Leaf { .. }))
        .map(|(_, pointer)| *pointer)
        .expect("多层的树有叶");
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session): (_, MountedSession) = MountedSession::of_the_mount(mounted);
    damage_the_copy_at(&mut devices, &leaf.locations[0], BYTES_OF_A_NODE);
    unmount(
        &parameters(),
        &mut devices,
        &mut session.allocator,
        &mut session.current,
        session.shadow_ledger,
    )
    .expect("卸载");
    let operations = pool.retained_operations();
    drop(devices);
    let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    image.apply(&operations);
    if let Some(violated) =
        violated_invariants_of_the_image("disk-after-the-unmount-leaf-not-repaired", &image)
    {
        assert!(
            ["I-2.1", "I-4.8", "I-7.4"]
                .iter()
                .any(|invariant| violated.contains(invariant)),
            "没修的那一份照报 I-2.1 这一族：{violated:?}"
        );
    }
}
