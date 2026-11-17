//! 验证包模块：crash_point_replay_progress、crash_given_states、crash_amplification
//! 落后支缺单元那一半只算 (tail, 所选] 同一实例窗口里记录点名的单元（D18（块里携带什么信息） 已定项 11）：窗口之外缺的不判缺，按读到一份坏处理，进计数与下一次用户改动之后另发的那一批；窗口跨实例、窗口取不到照全部算缺。
//! 盘面手造，盘是两块内存稀疏盘，池在进程里造出来，不建镜像文件。挂得上的接着一次改动、读回、卸载，不放回字节判真盘。

mod common;

use common::{file_content, geometry, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber,
    InstanceGeneration, JournalSequenceNumber, SlotNumber,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::{record_offset, JournalRecord, JournalRecordPlaceInPublish};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable, roll_back_by_a_forward_publish, unmount, DeviceWithoutTheSelectedVersion,
    MountError, RollbackCandidateExclusion, RollbackError, RollbackTarget,
    SelectedVersionLackingOnDevice,
};
use singlefs_core::mounted_session::{
    MountedSession, RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing, UserChange,
    UserChangePublished,
};
use singlefs_core::pointer::LocationEntry;
use singlefs_core::recovery::verified_system_configuration_slots;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_sequential_write, warm_up, FirstFile,
    PoolVersion, PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::{
    JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, SLOT_BYTES,
};
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
use singlefs_verification::mount_transparency::{read_recovery_fields, ComparedVersion};
use singlefs_verification::{RecordingBlockDevice, RetainedOperation, SharedStream};

type RecordedSparse = RecordingBlockDevice<SparseBlockDevice>;
type SparseDevices = Vec<(DeviceIdentity, SparseBlockDevice)>;

const DEVICE_ZERO: DeviceIdentity = DeviceIdentity(0);
const DEVICE_ONE: DeviceIdentity = DeviceIdentity(1);
const FLIPPED_BYTE_IN_THE_UNIT: u64 = 4000;
const FLIPPED_BYTE_IN_THE_RECORD: u64 = 100;
const BYTES_OF_THE_FILE: usize = 200_000;
/// 一次顺序写切成不止一条记录的文件字节数（写的数据单元不止一个）。
const BYTES_OF_THE_FILE_WRITTEN_IN_ONE_PUBLISH_OF_SEVERAL_RECORDS: usize = 100_000;
/// 分叉那两条时间线各走到的改动数上限（`0..=` 这个数）。
const CHANGES_OF_THE_LONGEST_TIMELINE_OF_THE_FORK: u64 = 5;
/// 找所选那一版末条记录时扫到的计数器上限：这份用例的历史写不到这么多条记录。
const COUNTERS_SCANNED_FOR_THE_LAST_RECORD_OF_THE_VERSION: u64 = 512;

fn content_of_the_file() -> Vec<u8> {
    (0..BYTES_OF_THE_FILE)
        .map(|index| u8::try_from(index % 251).expect("小于 251"))
        .collect()
}

fn recorded_devices_of(
    image: &MemoryPool,
    stream: &SharedStream,
) -> Vec<(DeviceIdentity, RecordedSparse)> {
    sparse_devices_of(image)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect()
}

fn sparse_devices_of(image: &MemoryPool) -> SparseDevices {
    image
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                image.device_size_in_bytes,
                PhysicalBlockSizeInBytes(parameters().geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect()
}

fn image_of(devices: &SparseDevices, device_size_in_bytes: u64) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes,
    }
}

/// mkfs、取号、暖机、发第一个文件再整份顺序写一次：交回盘与第一个文件各数据单元的两条位置条目。
fn pool_with_a_file(content: &[u8]) -> (MemoryPool, Vec<[LocationEntry; 2]>) {
    let publish_parameters = parameters();
    let stream = SharedStream::retaining_contents();
    let empty = MemoryPool::with_devices(&[DEVICE_ZERO, DEVICE_ONE], IMAGE_BYTES);
    let mut devices = recorded_devices_of(&empty, &stream);
    let genesis = make_filesystem(&publish_parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DEVICE_ZERO, IMAGE_BYTES),
        DeviceFreeMap::new(DEVICE_ONE, IMAGE_BYTES),
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
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &publish_parameters,
        &genesis,
    ));
    let output = {
        let mut pool = PoolWriter::new(&publish_parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warm_up_output = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        let first_file_content = file_content();
        let first_file = publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up_output.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &first_file_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up_output.last_record_bytes,
        )
        .expect("新池新建文件");
        publish_sequential_write(
            &mut pool,
            &mut allocator,
            &first_file,
            FirstFile {
                content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            },
            instance,
        )
        .expect("顺序写")
    };
    let mut image = empty.clone();
    image.apply(&stream.retained_operations());
    let locations = output
        .data_pointers
        .iter()
        .map(|pointer| pointer.locations)
        .collect();
    (image, locations)
}

/// 一段录下来的历史：`base` 上可写挂载，再做几次改动，不卸载，所有写都落了。
struct RecordedHistory {
    base: MemoryPool,
    operations: Vec<RetainedOperation>,
    operations_of_the_mount: usize,
}

impl RecordedHistory {
    /// 挂载之后建 `changes` 个空 inode，一次改动一个。
    fn of_new_inodes(base: &MemoryPool, changes: u64, write_time_seconds: u64) -> Self {
        let stream = SharedStream::retaining_contents();
        let mut devices = recorded_devices_of(base, &stream);
        let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
        let operations_of_the_mount = stream.retained_operations().len();
        let (_, mut session): (_, MountedSession) = MountedSession::of_the_mount(mounted);
        // 迭代上界是 `changes`；跨轮携带的只有会话。
        for change in 0..changes {
            session
                .publish_user_change(
                    &mut devices,
                    UserChange::NewInodes {
                        count: 1,
                        write_time_seconds: write_time_seconds + change,
                    },
                )
                .expect("建一个空 inode");
        }
        Self {
            base: base.clone(),
            operations: stream.retained_operations(),
            operations_of_the_mount,
        }
    }

    /// 挂载之后给下一个 inode 顺序写一次：普通的多单元写，一次发布几条记录。
    fn of_a_sequential_write_of_several_records(base: &MemoryPool) -> Self {
        let content: Vec<u8> = (0..BYTES_OF_THE_FILE_WRITTEN_IN_ONE_PUBLISH_OF_SEVERAL_RECORDS)
            .map(|index| u8::try_from(index % 241).expect("小于 241"))
            .collect();
        let stream = SharedStream::retaining_contents();
        let mut devices = recorded_devices_of(base, &stream);
        let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
        let operations_of_the_mount = stream.retained_operations().len();
        let (_, mut session): (_, MountedSession) = MountedSession::of_the_mount(mounted);
        session
            .publish_user_change(
                &mut devices,
                UserChange::SequentialWriteOfAnInode {
                    inode: InodeNumber(FIRST_INODE_NUMBER + 1),
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 80,
                },
            )
            .expect("顺序写");
        Self {
            base: base.clone(),
            operations: stream.retained_operations(),
            operations_of_the_mount,
        }
    }

    fn image_after_everything(&self) -> MemoryPool {
        let mut image = self.base.clone();
        image.apply(&self.operations);
        image
    }

    fn image_after_the_mount(&self) -> MemoryPool {
        let mut image = self.base.clone();
        image.apply(&self.operations[..self.operations_of_the_mount]);
        image
    }

    fn writes_after_the_mount(&self) -> Vec<RetainedWrite> {
        writes_and_segments_with_stream_indexes(
            &self.operations[self.operations_of_the_mount..],
            &geometry(),
        )
        .0
    }

    fn writes_of_the_whole_history(&self) -> Vec<RetainedWrite> {
        writes_and_segments_with_stream_indexes(&self.operations, &geometry()).0
    }
}

/// 每条用例的起点：带文件的池、又一次挂载与两次改动之后的盘（实例新于发第一个文件的那一个），第一个文件各数据单元的两条位置条目。
struct PoolBeforeTheHistories {
    content: Vec<u8>,
    data_units: Vec<[LocationEntry; 2]>,
    image: MemoryPool,
}

fn pool_before_the_histories() -> PoolBeforeTheHistories {
    let content = content_of_the_file();
    let (image_with_the_file, data_units) = pool_with_a_file(&content);
    let image =
        RecordedHistory::of_new_inodes(&image_with_the_file, 2, FIXED_WRITE_TIME_SECONDS + 60)
            .image_after_everything();
    PoolBeforeTheHistories {
        content,
        data_units,
        image,
    }
}

fn is_a_journal_record_write(write: &RetainedWrite) -> bool {
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    (ring_start..ring_start + JOURNAL_RING_DEFAULT_BYTES).contains(&write.offset.0)
}

/// 写表里每个 journal 记录写的下标，按写的次序（每条记录盘 0 那一份在前）。
fn indexes_of_the_record_writes(writes: &[RetainedWrite]) -> Vec<usize> {
    let indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| is_a_journal_record_write(write))
        .map(|(index, _)| index)
        .collect();
    assert!(indexes.len() >= 2, "前提：写表里有记录写：{indexes:?}");
    for pair in indexes.as_chunks::<2>().0 {
        assert_eq!(
            writes[pair[0]].device, DEVICE_ZERO,
            "每条记录盘 0 那一份在前"
        );
        assert_eq!(
            writes[pair[1]].device, DEVICE_ONE,
            "每条记录盘 1 那一份在后"
        );
    }
    indexes
}

/// `image` 上施加 `index_on_device_zero` 之前的全部写，再加 `kept` 那块盘上这条记录那一份（另一份没落）：单元都落了、记录只落一块盘，根与轮换都没落。
fn crash_with_a_record_only_on(
    image: &MemoryPool,
    writes: &[RetainedWrite],
    index_on_device_zero: usize,
    kept: DeviceIdentity,
) -> MemoryPool {
    let mut crashed = image.clone();
    crashed.apply_writes(&writes[..index_on_device_zero]);
    let index_kept = if kept == DEVICE_ZERO {
        index_on_device_zero
    } else {
        index_on_device_zero + 1
    };
    crashed.apply_writes(std::slice::from_ref(&writes[index_kept]));
    crashed
}

fn location_on(locations: &[LocationEntry; 2], device: DeviceIdentity) -> LocationEntry {
    *locations
        .iter()
        .find(|location| location.device == device)
        .expect("两块盘各一份")
}

fn damaged(mut image: MemoryPool, location: &LocationEntry) -> MemoryPool {
    image.flip_byte(
        location.device,
        location.slot.to_device_offset(),
        FLIPPED_BYTE_IN_THE_UNIT,
    );
    image
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
                    "writable_mount_of_a_device_behind_the_version_judges_the_units_named_after_its_tail::{table_name}"
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

/// 现行那一版的单元里，这块盘上字节不同的有几个（字节是空的不算）。
fn units_of_the_current_version_differing_on(
    session: &MountedSession,
    devices: &SparseDevices,
    device: DeviceIdentity,
) -> usize {
    let version = session.current.version_of_trees().expect("现行那一版有树");
    let (_, disk) = devices
        .iter()
        .find(|(identity, _)| *identity == device)
        .expect("有这块盘");
    version
        .units
        .iter()
        .filter(|unit| !unit.bytes.is_empty())
        .filter(|unit| {
            disk.image
                .read(unit.slot.to_device_offset(), unit.bytes.len())[..]
                != unit.bytes[..]
        })
        .count()
}

fn assert_the_batch_relocated_one_unit(table_name: &str, published: &UserChangePublished) {
    match &published.relocation_after_the_change_of_the_units_read_with_one_copy_failing {
        RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::Relocated {
            mapping_keys, ..
        } => assert_eq!(mapping_keys.len(), 1, "{table_name}：那一批搬一个单元"),
        other @ (RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NoSpaceAfterRaisingTheFloor {
            ..
        }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace { .. }
        | RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::FloorRaiseFailedWhilePushingForSpace {
            ..
        }) => panic!("{table_name}：坏的那一份要另发一批搬走，交回 {other:?}"),
    }
}

/// 这张盘放行：挂载读到的只坏一份合计 1；接着一次改动，另发的那一批把坏的那一份搬走；读回第一个文件；卸载之后不放回字节判真盘一条违例都没有。
fn assert_the_device_behind_is_let_through_and_the_damaged_copy_is_moved(
    table_name: &str,
    image: &MemoryPool,
    content: &[u8],
) {
    let mut devices = sparse_devices_of(image);
    let mounted = mount_writable(&parameters(), &mut devices)
        .unwrap_or_else(|refusal| panic!("{table_name}：窗口之外缺的不判缺，要放行：{refusal:?}"));
    let (_, mut session): (_, MountedSession) = MountedSession::of_the_mount(mounted);
    assert_eq!(
        session.unit_reads_with_one_copy_failing_while_mounting()
            + session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "{table_name}：坏的那一份按读到一份坏进账，合计一次"
    );
    let published = session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 900,
            },
        )
        .unwrap_or_else(|refusal| panic!("{table_name}：改动被拒：{refusal:?}"));
    assert_the_batch_relocated_one_unit(table_name, &published);
    let read = session
        .read_the_file(
            &devices,
            InodeNumber(FIRST_INODE_NUMBER),
            FileOffsetInBytes(0),
            u64::try_from(content.len()).expect("文件字节数"),
        )
        .unwrap_or_else(|refusal| panic!("{table_name}：读文件被拒：{refusal:?}"));
    assert_eq!(read.bytes, content, "{table_name}：读回第一个文件");
    for device in [DEVICE_ZERO, DEVICE_ONE] {
        assert_eq!(
            units_of_the_current_version_differing_on(&session, &devices, device),
            0,
            "{table_name}：搬走之后盘 {device:?} 上现行那一版的单元都对得上"
        );
    }
    unmount(
        &parameters(),
        &mut devices,
        &mut session.allocator,
        &mut session.current,
        session.shadow_ledger,
    )
    .unwrap_or_else(|refusal| panic!("{table_name}：卸载被拒：{refusal:?}"));
    let after = image_of(&devices, image.device_size_in_bytes);
    if let Some(violated) = violated_invariants_of_the_image(table_name, &after) {
        assert_eq!(violated, Vec::<&str>::new(), "{table_name}：判红的不变量");
    }
}

/// 这张盘在取号之前被落后支缺单元那一半拒，点名 `device`，两块盘逐字节不变；交回 (所选那一版的 jsn, 那块盘的 tail, 缺的落点)。
fn refusal_naming_the_device_behind_and_missing_units(
    case: &str,
    image: &MemoryPool,
    device: DeviceIdentity,
) -> (
    JournalSequenceNumber,
    JournalSequenceNumber,
    Vec<SlotNumber>,
) {
    let mut devices = sparse_devices_of(image);
    let refusal = mount_writable(&parameters(), &mut devices)
        .err()
        .unwrap_or_else(|| panic!("{case}：要拒可写，却挂上了"));
    assert_eq!(
        image_of(&devices, image.device_size_in_bytes),
        *image,
        "{case}：拒在任何写之前，两块盘逐字节不变"
    );
    let MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion {
        selected_version_journal_position,
        devices: devices_without,
        ..
    } = refusal
    else {
        panic!("{case}：该拒成 WritableMountRefusedByDevicesWithoutTheSelectedVersion，实际 {refusal:?}");
    };
    let [DeviceWithoutTheSelectedVersion {
        device: device_named,
        lacking:
            SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndMissingItsUnits {
                newest_system_configuration_journal_tail,
                units_missing,
            },
    }] = devices_without.as_slice()
    else {
        panic!("{case}：只点名一块盘、缺的是单元：{devices_without:?}");
    };
    assert_eq!(*device_named, device, "{case}：点名的盘");
    (
        selected_version_journal_position,
        *newest_system_configuration_journal_tail,
        units_missing.iter().map(|(_, slot)| *slot).collect(),
    )
}

/// 记录只落盘 1、盘 0 上早先弄坏一份（盘 0 那一份在第一条位置条目，建上一版读得到）：盘 0 落后、环槽里没有末条记录、缺的不被 tail 之后的记录点名。放行，坏的那份被搬走。
/// 判别力：`crates/mutations.tsv`「落后支缺单元：不看 tail 之后的记录点名（窗口之外缺的照拒）」「窗口的记录只读落后那块盘自己的环槽」那两行。
#[test]
fn a_record_only_on_device_one_with_an_earlier_copy_damaged_on_device_zero_is_let_through_and_the_copy_is_moved(
) {
    let pool = pool_before_the_histories();
    let history = RecordedHistory::of_new_inodes(&pool.image, 1, FIXED_WRITE_TIME_SECONDS + 70);
    let writes = history.writes_after_the_mount();
    let crashed = crash_with_a_record_only_on(
        &history.image_after_the_mount(),
        &writes,
        indexes_of_the_record_writes(&writes)[0],
        DEVICE_ONE,
    );
    let image = damaged(crashed, &location_on(&pool.data_units[0], DEVICE_ZERO));
    assert_the_device_behind_is_let_through_and_the_damaged_copy_is_moved(
        "record-on-1-older-copy-on-0",
        &image,
        &pool.content,
    );
}

/// 记录只落盘 0、盘 1 上早先弄坏一份（盘 1 那一份在第二条位置条目，读单元读到第一条就停，只有落后支看得见它）：放行，那一份进别的读那一份账，被搬走。
/// 判别力：`crates/mutations.tsv`「落后支窗口之外缺的不报（第二条位置条目上那一份不计不搬）」「可写挂载取号之前逐盘核报的不进账」那两行。
#[test]
fn a_record_only_on_device_zero_with_an_earlier_copy_damaged_on_device_one_counts_the_copy_and_moves_it(
) {
    let pool = pool_before_the_histories();
    let history = RecordedHistory::of_new_inodes(&pool.image, 1, FIXED_WRITE_TIME_SECONDS + 70);
    let writes = history.writes_after_the_mount();
    let crashed = crash_with_a_record_only_on(
        &history.image_after_the_mount(),
        &writes,
        indexes_of_the_record_writes(&writes)[0],
        DEVICE_ZERO,
    );
    let image = damaged(crashed, &location_on(&pool.data_units[0], DEVICE_ONE));
    assert_the_device_behind_is_let_through_and_the_damaged_copy_is_moved(
        "record-on-0-older-copy-on-1",
        &image,
        &pool.content,
    );
}

/// 普通多单元写一次发布几条记录，末条只落盘 1、盘 0 上早先弄坏一份：窗口不止一条记录，放行不拒错。
/// 只放紧邻一版的读法（tail 加一不是末条计数器时退回整版比）在这一形上拒错。
#[test]
fn the_last_of_several_records_of_one_publish_only_on_device_one_with_an_earlier_copy_damaged_is_let_through(
) {
    let pool = pool_before_the_histories();
    let history = RecordedHistory::of_a_sequential_write_of_several_records(&pool.image);
    let writes = history.writes_after_the_mount();
    let record_writes = indexes_of_the_record_writes(&writes);
    assert!(
        record_writes.len() >= 4,
        "前提：这次写不止一条记录：{record_writes:?}"
    );
    let crashed = crash_with_a_record_only_on(
        &history.image_after_the_mount(),
        &writes,
        record_writes[record_writes.len() - 2],
        DEVICE_ONE,
    );
    let image = damaged(crashed, &location_on(&pool.data_units[0], DEVICE_ZERO));
    assert_the_device_behind_is_let_through_and_the_damaged_copy_is_moved(
        "several-records-last-on-1-older-copy-on-0",
        &image,
        &pool.content,
    );
}

/// 上一个实例的一次改动记录落了、轮换没落；下一次挂载取号写的 tail 是取号那一刻的见证值（新实例的实例代号、上一次轮换的计数器），
/// 写行那次发布的根也落了、轮换没落，它的末条记录只落盘 1。盘 0 落后、tail 与所选那一版同实例，窗口 (tail, 所选] 里有一个计数器只住着上一个实例那条记录。
/// 窗口只按计数器认，取得到；盘 0 上早先坏的那一份不被窗口点名，放行（D18（块里携带什么信息） 已定项 11）。
/// 只有末条记录落了、根没落的那一形不行：重放前缀按实例接链，新实例那条记录接不到上一个实例那条后面，所选那一版停在上一个实例，盘 0 不算落后、走不到窗口。
/// 判别力：`crates/mutations.tsv`「窗口的记录只认所选那一版的实例（夹着上一个实例那条记录时窗口取不到、拒错）」那一行。
#[test]
fn a_window_holding_the_record_of_the_previous_instance_is_let_through_with_an_earlier_copy_damaged(
) {
    let pool = pool_before_the_histories();
    let history = RecordedHistory::of_new_inodes(&pool.image, 1, FIXED_WRITE_TIME_SECONDS + 70);
    let writes = history.writes_after_the_mount();
    let record_writes = indexes_of_the_record_writes(&writes);
    let mut both_records_landed = history.image_after_the_mount();
    both_records_landed.apply_writes(&writes[..=record_writes[1]]);
    let next_mount = RecordedHistory::of_new_inodes(&both_records_landed, 0, 0);
    let writes_of_the_next_mount = next_mount.writes_of_the_whole_history();
    let record_writes_of_the_next_mount = indexes_of_the_record_writes(&writes_of_the_next_mount);
    let (record_on_device_zero, record_on_device_one) = (
        record_writes_of_the_next_mount[0],
        record_writes_of_the_next_mount[1],
    );
    let record_of_the_row_publish = JournalRecord::parse(
        writes_of_the_next_mount[record_on_device_one]
            .bytes()
            .expect("记录写带字节"),
        unit_filesystem_identifier(&parameters().filesystem_identifier),
    )
    .expect("解得开");
    assert_eq!(
        record_of_the_row_publish.place_in_publish,
        JournalRecordPlaceInPublish::LastRecordOfThePublish,
        "前提：写行那次发布只有一条记录"
    );
    let index_of_the_root_write = (record_on_device_one..writes_of_the_next_mount.len())
        .find(|index| writes_of_the_next_mount[*index].is_force_unit_access)
        .expect("根槽 FUA");
    let mut root_landed = both_records_landed;
    root_landed.apply_writes(&writes_of_the_next_mount[..record_on_device_zero]);
    root_landed.apply_writes(
        &writes_of_the_next_mount[record_on_device_zero + 1..=index_of_the_root_write],
    );
    let image = damaged(root_landed, &location_on(&pool.data_units[0], DEVICE_ZERO));
    let position = JournalSequenceNumber {
        instance_generation: record_of_the_row_publish.instance,
        counter: record_of_the_row_publish.counter,
    };
    let tail = tail_of_the_device(&image, DEVICE_ZERO);
    assert_eq!(
        tail.instance_generation, position.instance_generation,
        "前提：盘 0 的 tail 与所选那一版同实例"
    );
    assert!(
        tail < position,
        "前提：盘 0 落后：tail {tail:?}，所选 {position:?}"
    );
    assert!(
        record_in_the_ring_of(&image, DEVICE_ZERO, position.counter).is_none(),
        "前提：盘 0 环槽里没有末条记录"
    );
    let instances_held_by_each_counter_of_the_window: Vec<(u64, Vec<InstanceGeneration>)> =
        ((tail.counter + 1)..=position.counter)
            .map(|counter| {
                (
                    counter,
                    [DEVICE_ZERO, DEVICE_ONE]
                        .into_iter()
                        .filter_map(|device| record_in_the_ring_of(&image, device, counter))
                        .map(|record| record.instance)
                        .collect(),
                )
            })
            .collect();
    assert!(
        instances_held_by_each_counter_of_the_window
            .iter()
            .any(|(_, instances)| {
                !instances.is_empty()
                    && instances
                        .iter()
                        .all(|instance| *instance != position.instance_generation)
            }),
        "前提：窗口里有一个计数器只住着上一个实例的记录：{instances_held_by_each_counter_of_the_window:?}"
    );
    assert_the_device_behind_is_let_through_and_the_damaged_copy_is_moved(
        "window-holding-the-previous-instance-older-copy-on-0",
        &image,
        &pool.content,
    );
}

/// 两盘各走一条历史（盘 0 在旧实例上改了五次，盘 1 又挂了一次再改三次）：盘 0 的 tail 与所选那一版不同实例，盘 0 真缺单元，照拒。
/// 判别力：`crates/mutations.tsv`「窗口跨实例也放行（按计数器取窗口）」那一行。
#[test]
fn a_device_behind_across_instances_truly_missing_units_is_refused() {
    let pool = pool_before_the_histories();
    let older_timeline =
        RecordedHistory::of_new_inodes(&pool.image, 5, FIXED_WRITE_TIME_SECONDS + 100)
            .image_after_everything();
    let mounted_once = RecordedHistory::of_new_inodes(&pool.image, 0, 0).image_after_everything();
    let newer_timeline =
        RecordedHistory::of_new_inodes(&mounted_once, 3, FIXED_WRITE_TIME_SECONDS + 300)
            .image_after_everything();
    let image = with_device_zero_of(&newer_timeline, &older_timeline);
    let (position, tail, slots_missing) =
        refusal_naming_the_device_behind_and_missing_units("across-instances", &image, DEVICE_ZERO);
    assert_ne!(
        tail.instance_generation, position.instance_generation,
        "前提：tail 与所选那一版不同实例"
    );
    assert!(!slots_missing.is_empty(), "缺的单元");
}

/// 根落了、轮换没落，盘 0 上末条记录那一份坏、这次发布点名的单元盘 0 那一份也坏：盘 0 落后、同实例，缺的被 tail 之后的记录点名，照拒。
/// 判别力：`crates/mutations.tsv`「窗口的点名不进清单（窗口之内缺的也放行）」「窗口不含末条记录」那两行。
#[test]
fn a_device_behind_missing_a_unit_named_after_its_tail_is_refused() {
    let pool = pool_before_the_histories();
    let history = RecordedHistory::of_new_inodes(&pool.image, 1, FIXED_WRITE_TIME_SECONDS + 70);
    let writes = history.writes_after_the_mount();
    let record_writes = indexes_of_the_record_writes(&writes);
    let (record_on_device_zero, record_on_device_one) = (record_writes[0], record_writes[1]);
    let record = JournalRecord::parse(
        writes[record_on_device_one].bytes().expect("记录写带字节"),
        unit_filesystem_identifier(&parameters().filesystem_identifier),
    )
    .expect("解得开");
    let named_on_device_zero = location_on(&record.named[0].locations, DEVICE_ZERO);
    let index_of_the_root_write = (record_on_device_one..writes.len())
        .find(|index| writes[*index].is_force_unit_access)
        .expect("根槽 FUA");
    let mut root_landed = history.image_after_the_mount();
    root_landed.apply_writes(&writes[..=index_of_the_root_write]);
    root_landed.flip_byte(
        DEVICE_ZERO,
        writes[record_on_device_zero].offset,
        FLIPPED_BYTE_IN_THE_RECORD,
    );
    let image = damaged(root_landed, &named_on_device_zero);
    let (position, tail, slots_missing) = refusal_naming_the_device_behind_and_missing_units(
        "named-after-the-tail",
        &image,
        DEVICE_ZERO,
    );
    assert_eq!(
        tail.instance_generation, position.instance_generation,
        "前提：tail 与所选那一版同实例"
    );
    assert_eq!(
        slots_missing,
        vec![named_on_device_zero.slot],
        "缺的就是点名的那一个"
    );
}

/// 一次发布几条记录、根落了、轮换没落，末条记录盘 0 那一份坏，头一条记录点名的单元盘 0 那一份也坏：缺的被窗口里更早的一条记录点名，照拒。
/// 判别力：`crates/mutations.tsv`「窗口只取末条记录那一个计数器」那一行。
#[test]
fn a_device_behind_missing_a_unit_named_by_an_earlier_record_of_the_window_is_refused() {
    let pool = pool_before_the_histories();
    let history = RecordedHistory::of_a_sequential_write_of_several_records(&pool.image);
    let writes = history.writes_after_the_mount();
    let record_writes = indexes_of_the_record_writes(&writes);
    assert!(
        record_writes.len() >= 4,
        "前提：这次写不止一条记录：{record_writes:?}"
    );
    let record = JournalRecord::parse(
        writes[record_writes[1]].bytes().expect("记录写带字节"),
        unit_filesystem_identifier(&parameters().filesystem_identifier),
    )
    .expect("解得开");
    let named_on_device_zero = location_on(&record.named[0].locations, DEVICE_ZERO);
    let index_of_the_root_write = (record_writes[record_writes.len() - 1]..writes.len())
        .find(|index| writes[*index].is_force_unit_access)
        .expect("根槽 FUA");
    let mut root_landed = history.image_after_the_mount();
    root_landed.apply_writes(&writes[..=index_of_the_root_write]);
    let index_of_the_last_record_on_device_zero = record_writes[record_writes.len() - 2];
    root_landed.flip_byte(
        DEVICE_ZERO,
        writes[index_of_the_last_record_on_device_zero].offset,
        FLIPPED_BYTE_IN_THE_RECORD,
    );
    let image = damaged(root_landed, &named_on_device_zero);
    let (_, _, slots_missing) = refusal_naming_the_device_behind_and_missing_units(
        "named-by-an-earlier-record",
        &image,
        DEVICE_ZERO,
    );
    assert_eq!(
        slots_missing,
        vec![named_on_device_zero.slot],
        "缺的就是更早那条记录点名的那一个"
    );
}

/// 一次发布几条记录、根落了、轮换没落，窗口里头一条记录两块盘都坏，末条盘 0 那一份也坏，盘 0 上早先弄坏一份：窗口取不到，缺的全部算缺，照拒。
/// 判别力：`crates/mutations.tsv`「窗口里一个计数器哪块盘都解不出时跳过它（窗口取不到也放行）」那一行。
#[test]
fn a_window_with_a_record_lost_on_every_device_counts_every_differing_unit_as_missing() {
    let pool = pool_before_the_histories();
    let history = RecordedHistory::of_a_sequential_write_of_several_records(&pool.image);
    let writes = history.writes_after_the_mount();
    let record_writes = indexes_of_the_record_writes(&writes);
    assert!(
        record_writes.len() >= 4,
        "前提：这次写不止一条记录：{record_writes:?}"
    );
    let index_of_the_last_record_on_device_zero = record_writes[record_writes.len() - 2];
    let index_of_the_root_write = (record_writes[record_writes.len() - 1]..writes.len())
        .find(|index| writes[*index].is_force_unit_access)
        .expect("根槽 FUA");
    let mut root_landed = history.image_after_the_mount();
    root_landed.apply_writes(&writes[..=index_of_the_root_write]);
    for index_of_the_bad_record in [
        record_writes[0],
        record_writes[1],
        index_of_the_last_record_on_device_zero,
    ] {
        root_landed.flip_byte(
            writes[index_of_the_bad_record].device,
            writes[index_of_the_bad_record].offset,
            FLIPPED_BYTE_IN_THE_RECORD,
        );
    }
    let older_copy = location_on(&pool.data_units[0], DEVICE_ZERO);
    let image = damaged(root_landed, &older_copy);
    let (position, tail, slots_missing) = refusal_naming_the_device_behind_and_missing_units(
        "window-with-a-record-lost",
        &image,
        DEVICE_ZERO,
    );
    assert_eq!(
        tail.instance_generation, position.instance_generation,
        "前提：tail 与所选那一版同实例"
    );
    assert!(
        slots_missing.contains(&older_copy.slot),
        "窗口取不到：早先弄坏的那一份也算缺：{slots_missing:?}"
    );
}

/// 会话改动入口上的落后支：只剩一槽自证的盘（tail 停在上一次轮换）、环槽里那一版的末条记录也坏、早先的一个数据单元这块盘上那一份坏：放行，那一份进别的读那一份账，这次改动发成之后那一批搬走。
/// 判别力：`crates/mutations.tsv`「会话改动入口的落后支报的不进账」那一行。
#[test]
fn a_device_behind_at_the_change_entry_with_an_earlier_copy_damaged_counts_the_copy_and_moves_it() {
    let pool = pool_before_the_histories();
    let publish_parameters = parameters();
    let mut devices = sparse_devices_of(&pool.image);
    let mounted = mount_writable(&publish_parameters, &mut devices).expect("可写挂载");
    let (_, mut session): (_, MountedSession) = MountedSession::of_the_mount(mounted);
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 70,
            },
        )
        .expect("建一个空 inode");
    let counter_of_the_last_record = session
        .current
        .file_version()
        .expect("现行那一版带文件")
        .record
        .counter;
    let slot_spacing = u64::from(publish_parameters.geometry.fixed_structure_slot_spacing);
    let generation_of_the_newest_slot = verified_system_configuration_slots(
        devices.as_slice(),
        DEVICE_ZERO,
        slot_spacing,
        &publish_parameters.filesystem_identifier,
    )
    .into_iter()
    .map(|slot| slot.quantities.slot_generation)
    .max()
    .expect("盘 0 有自证过的系统配置");
    let older_copy = location_on(&pool.data_units[0], DEVICE_ZERO);
    let (_, device_zero) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == DEVICE_ZERO)
        .expect("有这块盘");
    device_zero.image.zero_fill(
        DeviceOffsetInBytes((generation_of_the_newest_slot % 2) * slot_spacing),
        JOURNAL_RECORD_BYTES,
    );
    device_zero.image.zero_fill(
        record_offset(
            counter_of_the_last_record,
            publish_parameters.geometry.journal_ring_bytes,
        ),
        JOURNAL_RECORD_BYTES,
    );
    let image_with_the_copy_damaged = damaged(image_of(&devices, IMAGE_BYTES), &older_copy);
    device_zero_takes_the_sectors_of(&mut devices, &image_with_the_copy_damaged);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        0,
        "前提：入口之前没有读到过那一份"
    );
    let published = session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 71,
            },
        )
        .unwrap_or_else(|refusal| panic!("窗口之外缺的不判缺，改动要放行：{refusal:?}"));
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "入口上看见的那一份进别的读那一份账"
    );
    assert_the_batch_relocated_one_unit("change-entry-older-copy-on-0", &published);
}

/// 同 [`a_device_behind_at_the_change_entry_with_an_earlier_copy_damaged_counts_the_copy_and_moves_it`] 那一形，造到入口之前：挂上、建一个空 inode，
/// 再让盘 0 只剩一槽自证（tail 停在上一次轮换）、环槽里现行那一版的末条记录也坏、第一个文件头一个数据单元在盘 0 上那一份坏。
/// 交回会话、盘与建 inode 之前那一版（挂载之后那一版）的回退目标。
fn session_with_device_zero_behind_and_an_earlier_copy_damaged(
    pool: &PoolBeforeTheHistories,
) -> (MountedSession, SparseDevices, RollbackTarget) {
    let publish_parameters = parameters();
    let mut devices = sparse_devices_of(&pool.image);
    let mounted = mount_writable(&publish_parameters, &mut devices).expect("可写挂载");
    let (_, mut session): (_, MountedSession) = MountedSession::of_the_mount(mounted);
    let root_of_the_mount = *session.current.root();
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 70,
            },
        )
        .expect("建一个空 inode");
    let counter_of_the_last_record = session
        .current
        .file_version()
        .expect("现行那一版带文件")
        .record
        .counter;
    let slot_spacing = u64::from(publish_parameters.geometry.fixed_structure_slot_spacing);
    let generation_of_the_newest_slot = verified_system_configuration_slots(
        devices.as_slice(),
        DEVICE_ZERO,
        slot_spacing,
        &publish_parameters.filesystem_identifier,
    )
    .into_iter()
    .map(|slot| slot.quantities.slot_generation)
    .max()
    .expect("盘 0 有自证过的系统配置");
    let older_copy = location_on(&pool.data_units[0], DEVICE_ZERO);
    let (_, device_zero) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == DEVICE_ZERO)
        .expect("有这块盘");
    device_zero.image.zero_fill(
        DeviceOffsetInBytes((generation_of_the_newest_slot % 2) * slot_spacing),
        JOURNAL_RECORD_BYTES,
    );
    device_zero.image.zero_fill(
        record_offset(
            counter_of_the_last_record,
            publish_parameters.geometry.journal_ring_bytes,
        ),
        JOURNAL_RECORD_BYTES,
    );
    let image_with_the_copy_damaged = damaged(image_of(&devices, IMAGE_BYTES), &older_copy);
    device_zero_takes_the_sectors_of(&mut devices, &image_with_the_copy_damaged);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        0,
        "前提：入口之前没有读到过那一份"
    );
    (
        session,
        devices,
        RollbackTarget {
            instance: root_of_the_mount.instance,
            checkpoint_txg: root_of_the_mount.checkpoint_txg,
        },
    )
}

/// 读文件入口上的落后支（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）：盘 0 落后、早先的一个数据单元它上面那一份坏，
/// 读第一个文件末尾那一个字节（不读那个数据单元）：读放行，落后支看见的那一份进别的读那一份账，下一次改动发成之后那一批搬走。
/// 判别力：`crates/mutations.tsv`「读文件入口的落后支经裸读者」那一行。
#[test]
fn a_device_behind_at_the_file_read_entry_with_an_earlier_copy_damaged_counts_the_copy_and_moves_it(
) {
    let pool = pool_before_the_histories();
    let (mut session, mut devices, _target_before_the_change) =
        session_with_device_zero_behind_and_an_earlier_copy_damaged(&pool);
    let offset_of_the_last_byte = pool.content.len() - 1;
    let read = session
        .read_the_file(
            &devices,
            InodeNumber(FIRST_INODE_NUMBER),
            FileOffsetInBytes(u64::try_from(offset_of_the_last_byte).expect("文件字节数")),
            1,
        )
        .unwrap_or_else(|refusal| panic!("窗口之外缺的不判缺，读要放行：{refusal:?}"));
    assert_eq!(
        read.bytes.as_slice(),
        &pool.content[offset_of_the_last_byte..],
        "读回第一个文件末尾那一个字节"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "读文件入口的落后支看见的那一份进别的读那一份账"
    );
    let published = session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 71,
            },
        )
        .unwrap_or_else(|refusal| panic!("窗口之外缺的不判缺，改动要放行：{refusal:?}"));
    assert_the_batch_relocated_one_unit("file-read-entry-older-copy-on-0", &published);
}

/// 挂着回退的落后支（`mount::caller_inputs_agreeing_with_the_disk`，抬 F、准入抬 F、推环、卸载与回退共用；D4（校验和位置） 已定项 4）：
/// 盘 0 落后、早先的一个数据单元它上面那一份坏，挂着回退到建 inode 之前那一版：回退放行，落后支看见的那一份进别的读那一份账，
/// 回退之后下一次改动发成之后那一批搬走。
/// 回退做成时，那一核之后经计数层的读也读到同一份（账按位置条目去重），这一条不分辨那一核记没记；分辨它的是
/// [`a_device_behind_at_a_rollback_refused_after_the_check_of_the_caller_inputs_counts_the_copy`]。
#[test]
fn a_device_behind_at_a_rollback_while_mounted_with_an_earlier_copy_damaged_counts_the_copy_and_moves_it(
) {
    let pool = pool_before_the_histories();
    let (mut session, mut devices, target_before_the_change) =
        session_with_device_zero_behind_and_an_earlier_copy_damaged(&pool);
    let mut current: TransactionOutput =
        TransactionOutput::clone(session.current.version_of_trees().expect("有树的一版"));
    roll_back_by_a_forward_publish(
        &parameters(),
        &mut devices,
        &mut session.allocator,
        &mut current,
        target_before_the_change,
    )
    .unwrap_or_else(|refusal| panic!("窗口之外缺的不判缺，回退要放行：{refusal:?}"));
    session.current = PoolVersion::of_a_version_of_trees(current);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "回退入口的落后支看见的那一份进别的读那一份账"
    );
    let published = session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 71,
            },
        )
        .unwrap_or_else(|refusal| panic!("回退之后改动要放行：{refusal:?}"));
    assert_the_batch_relocated_one_unit("rollback-entry-older-copy-on-0", &published);
}

/// 挂着之后收盘表那一核（`mount::caller_inputs_agreeing_with_the_disk`）自己的账：盘 0 落后、早先的一个数据单元它上面那一份坏，
/// 回退目标点名一个根环里没有的 txg。这一核放行之后，回退在候选那一判被拒（`NotInRing`），中间只读实例表与根环，不读那个数据单元；
/// 账里那一份只能是这一核的落后支记下的。被拒在任何写之前，两块盘逐字节不变。
/// 判别力：`crates/mutations.tsv`「挂着之后收盘表的入口落后支经裸读者」那一行。
#[test]
fn a_device_behind_at_a_rollback_refused_after_the_check_of_the_caller_inputs_counts_the_copy() {
    let pool = pool_before_the_histories();
    let (mut session, mut devices, target_before_the_change) =
        session_with_device_zero_behind_and_an_earlier_copy_damaged(&pool);
    let mut current: TransactionOutput =
        TransactionOutput::clone(session.current.version_of_trees().expect("有树的一版"));
    let target_not_in_the_ring = RollbackTarget {
        instance: target_before_the_change.instance,
        checkpoint_txg: CheckpointTxg(u64::MAX),
    };
    let image_before_the_rollback = image_of(&devices, IMAGE_BYTES);
    let refusal = roll_back_by_a_forward_publish(
        &parameters(),
        &mut devices,
        &mut session.allocator,
        &mut current,
        target_not_in_the_ring,
    )
    .expect_err("根环里没有那个 txg：回退要被拒");
    assert!(
        matches!(
            refusal,
            RollbackError::TargetNotACandidate {
                exclusion: RollbackCandidateExclusion::NotInRing,
                ..
            }
        ),
        "收盘表那一核放行之后在候选那一判被拒：{refusal:?}"
    );
    assert_eq!(
        image_of(&devices, IMAGE_BYTES),
        image_before_the_rollback,
        "被拒在任何写之前：两块盘逐字节不变"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "收盘表那一核的落后支看见的那一份进别的读那一份账"
    );
}

fn device_zero_takes_the_sectors_of(devices: &mut SparseDevices, image: &MemoryPool) {
    let (_, device_zero) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == DEVICE_ZERO)
        .expect("有这块盘");
    device_zero.image = image.devices[&DEVICE_ZERO].clone();
}

/// 把 `other` 的盘 0 换进 `image`。
fn with_device_zero_of(image: &MemoryPool, other: &MemoryPool) -> MemoryPool {
    let mut swapped = image.clone();
    swapped
        .devices
        .insert(DEVICE_ZERO, other.devices[&DEVICE_ZERO].clone());
    swapped
}

/// 这块盘世代号最大那份系统配置的 (实例代号, journal tail)。
fn tail_of_the_device(image: &MemoryPool, device: DeviceIdentity) -> JournalSequenceNumber {
    let publish_parameters = parameters();
    let newest = verified_system_configuration_slots(
        image,
        device,
        u64::from(publish_parameters.geometry.fixed_structure_slot_spacing),
        &publish_parameters.filesystem_identifier,
    )
    .into_iter()
    .max_by_key(|slot| slot.quantities.slot_generation)
    .expect("分叉的盘两块都有自证过的系统配置");
    JournalSequenceNumber {
        instance_generation: newest.quantities.journal_instance,
        counter: newest.quantities.journal_tail,
    }
}

/// `device` 的环槽里 `counter` 那一槽解得开的记录。
fn record_in_the_ring_of(
    image: &MemoryPool,
    device: DeviceIdentity,
    counter: u64,
) -> Option<JournalRecord> {
    let publish_parameters = parameters();
    let bytes = image.devices[&device].read(
        record_offset(counter, publish_parameters.geometry.journal_ring_bytes),
        usize::try_from(JOURNAL_RECORD_BYTES).expect("一条记录 4096 字节"),
    );
    JournalRecord::parse(
        &bytes,
        unit_filesystem_identifier(&publish_parameters.filesystem_identifier),
    )
    .filter(|record| record.counter == counter)
}

fn is_the_last_record_of_the_version(
    record: &JournalRecord,
    version: (InstanceGeneration, CheckpointTxg),
) -> bool {
    record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
        && (record.instance, record.checkpoint_txg) == version
}

/// 这张盘上哪几块盘落后于所选那一版、自己环槽里又没有它的末条记录：放行这样的盘靠的就是点名窗口。认不出所选那一版的末条记录时交空。
fn devices_behind_without_the_last_record_of_the_selected_version(
    image: &MemoryPool,
) -> Vec<DeviceIdentity> {
    let Some(version) = read_recovery_fields(image, ComparedVersion::TheRecoveredVersion)
        .report
        .effective_root
    else {
        return Vec::new();
    };
    let holds_the_last_record = |device: DeviceIdentity, counter: u64| {
        record_in_the_ring_of(image, device, counter)
            .is_some_and(|record| is_the_last_record_of_the_version(&record, version))
    };
    let Some(counter_of_the_last_record) =
        (1..=COUNTERS_SCANNED_FOR_THE_LAST_RECORD_OF_THE_VERSION).find(|counter| {
            [DEVICE_ZERO, DEVICE_ONE]
                .into_iter()
                .any(|device| holds_the_last_record(device, *counter))
        })
    else {
        return Vec::new();
    };
    let position = JournalSequenceNumber {
        instance_generation: version.0,
        counter: counter_of_the_last_record,
    };
    [DEVICE_ZERO, DEVICE_ONE]
        .into_iter()
        .filter(|device| {
            tail_of_the_device(image, *device) < position
                && !holds_the_last_record(*device, counter_of_the_last_record)
        })
        .collect()
}

/// 一种分叉的盘走到哪：拒在落后支缺单元那一半、别的拒（带拒因）、挂上（两块盘上现行那一版字节不同的单元数）。
#[derive(Debug)]
enum OutcomeOfAFork {
    RefusedAsMissingUnits,
    RefusedOtherwise(String),
    Mounted {
        differing_on_device_zero: usize,
        differing_on_device_one: usize,
    },
}

fn outcome_of_a_fork(image: &MemoryPool) -> OutcomeOfAFork {
    let mut devices = sparse_devices_of(image);
    match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => {
            let (_, session): (_, MountedSession) = MountedSession::of_the_mount(mounted);
            OutcomeOfAFork::Mounted {
                differing_on_device_zero: units_of_the_current_version_differing_on(
                    &session,
                    &devices,
                    DEVICE_ZERO,
                ),
                differing_on_device_one: units_of_the_current_version_differing_on(
                    &session, &devices, DEVICE_ONE,
                ),
            }
        }
        Err(refusal) => {
            assert_eq!(
                image_of(&devices, image.device_size_in_bytes),
                *image,
                "拒在任何写之前，两块盘逐字节不变"
            );
            let is_missing_units = matches!(
                &refusal,
                MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { devices: devices_without_the_version, .. }
                    if devices_without_the_version.iter().any(|device| matches!(
                        device.lacking,
                        SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndMissingItsUnits { .. }
                    ))
            );
            if is_missing_units {
                OutcomeOfAFork::RefusedAsMissingUnits
            } else {
                OutcomeOfAFork::RefusedOtherwise(format!("{refusal:?}"))
            }
        }
    }
}

/// 同实例分叉、两边改动数不等（两块盘同一个起点各挂一次、各改几次，再把盘 0 换成另一边的）：tail 落后、窗口非空，点名窗口的放宽真生效的就是这一支。
/// 这几张盘一份坏都没弄：挂上之后落后而环槽里没有末条记录的盘上字节不同的单元，都是从没落盘的，放过去就是判错。tail 不落后的盘放过去是今天就有的另一格（欠账「分叉盘今天就放真缺单元」），只打出来，不在这里判。
/// 每种取值一行 `SAME_INSTANCE_FORK`，至少一种拒在缺单元那一半才算这条用例走到了那一判。这一条红了是窗口判错，不放宽断言。
/// 判别力：`crates/mutations.tsv`「窗口的点名不进清单（分叉的盘放过去）」那一行。
// harness-test-granularity:one-scenario 同一个起点上两条时间线的各种接法，取值之间共用录好的时间线，判的是同一件事：这一族分叉有没有一种判错
#[test]
fn a_same_instance_fork_with_different_change_counts_never_lets_a_device_behind_through_lacking_units(
) {
    let pool = pool_before_the_histories();
    let timeline_of = |write_time_seconds: u64| -> Vec<MemoryPool> {
        (0..=CHANGES_OF_THE_LONGEST_TIMELINE_OF_THE_FORK)
            .map(|changes| {
                RecordedHistory::of_new_inodes(&pool.image, changes, write_time_seconds)
                    .image_after_everything()
            })
            .collect()
    };
    let timeline_on_device_zero = timeline_of(FIXED_WRITE_TIME_SECONDS + 400);
    let timeline_on_device_one = timeline_of(FIXED_WRITE_TIME_SECONDS + 600);
    let forks: Vec<(usize, usize)> = (0..timeline_on_device_zero.len())
        .flat_map(|changes_on_device_zero| {
            (0..timeline_on_device_one.len())
                .filter(move |changes_on_device_one| {
                    *changes_on_device_one != changes_on_device_zero
                })
                .map(move |changes_on_device_one| (changes_on_device_zero, changes_on_device_one))
        })
        .collect();
    let threads = singlefs_verification::heavy_threads::default_heavy_threads()
        .get()
        .min(forks.len());
    let outcomes: Vec<((usize, usize), Vec<DeviceIdentity>, OutcomeOfAFork)> =
        std::thread::scope(|scope| {
            let (timeline_on_device_zero, timeline_on_device_one) =
                (&timeline_on_device_zero, &timeline_on_device_one);
            // 迭代上界是线程数；每条线程走一段互不相交的取值，每种取值从录好的盘拷一份，跨线程不带状态，结果按段的次序接起来。
            let handles: Vec<_> = forks
                .chunks(forks.len().div_ceil(threads))
                .map(|forks_of_the_thread| {
                    scope.spawn(move || {
                        forks_of_the_thread
                            .iter()
                            .map(|(changes_on_device_zero, changes_on_device_one)| {
                                let image = with_device_zero_of(
                                    &timeline_on_device_one[*changes_on_device_one],
                                    &timeline_on_device_zero[*changes_on_device_zero],
                                );
                                (
                                    (*changes_on_device_zero, *changes_on_device_one),
                                    devices_behind_without_the_last_record_of_the_selected_version(
                                        &image,
                                    ),
                                    outcome_of_a_fork(&image),
                                )
                            })
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            handles
                .into_iter()
                .flat_map(|handle| handle.join().expect("线程里的断言照 panic 传上来"))
                .collect()
        });
    let mut refused_as_missing_units = 0usize;
    // 迭代上界是取值的个数；跨轮携带的只有这一个数。
    for ((changes_on_device_zero, changes_on_device_one), devices_behind, outcome) in &outcomes {
        println!(
            "SAME_INSTANCE_FORK device_zero_changes={changes_on_device_zero} device_one_changes={changes_on_device_one} behind_without_the_last_record={devices_behind:?} outcome={outcome:?}"
        );
        match outcome {
            OutcomeOfAFork::RefusedAsMissingUnits => refused_as_missing_units += 1,
            OutcomeOfAFork::RefusedOtherwise(refusal) => {
                assert!(
                    !refusal.contains("BehindTheSelectedVersionAndMissingItsUnits"),
                    "别的拒不该带落后支缺单元：{refusal}"
                );
            }
            OutcomeOfAFork::Mounted {
                differing_on_device_zero,
                differing_on_device_one,
            } => {
                for (device, differing) in [
                    (DEVICE_ZERO, *differing_on_device_zero),
                    (DEVICE_ONE, *differing_on_device_one),
                ] {
                    assert!(
                        !devices_behind.contains(&device) || differing == 0,
                        "分叉 {changes_on_device_zero}、{changes_on_device_one}：盘 {device:?} 落后、环槽里没有末条记录，一份坏都没弄却缺 {differing} 个单元，被放过去了"
                    );
                }
            }
        }
    }
    assert!(
        refused_as_missing_units >= 1,
        "至少一种取值走到缺单元那一判：{outcomes:?}"
    );
}
