//! checker 档模块：crash、crash_state_enumeration、writable_mount_phases、crash_amplification
//! 多次挂载、带读故障的流（欠账 C199（实例代号递增与 jsn 断号即止互相矛盾）两格与 C274（重放收养的对象不推计数器水位）一格；
//! 里程碑三二·5，2026-10-19 分诊定新写这条流、C274 并进来）。
//!
//! 流：mkfs → 实例 1：取号、暖机两次、第一个文件 A、覆盖写 B（`tests/common` 的 `build_pool` 与实例 1 覆盖写）→ 进程退出、重开镜像 →
//! 实例 2 可写挂载（取号、写实例 1 那一行、暖机两次）→ 实例 2 覆盖写 C → 实例 2 建快照 S（从树 ID 水位连发两个树号）。基镜像是实例 1 写完 B 之后的盘面，
//! 录的是实例 2 那次挂载与之后两次发布的全部写，按 D13（验证路线） 已定项 4 枚举崩溃状态；每个状态恢复两遍、多版本 oracle、池级 checker、记录核对器，
//! 另挂两个观察者：
//! - 新实例的记录施加到哪（计数观察者，在录入段对恢复报告下断言，豁免表 observer-assertions-on-the-recovery-report）：
//!   ① 这个状态里实例 2 有一条根槽写落了盘（所选根是新实例的根）⇒ 恢复落到实例 2 里记录齐了的最新一次发布
//!   （记录任一份落盘即在，D23（journal 的角色与格式） 已定项 14「两份镜像何时算记录在」）——水位之后新实例的记录全部施加；
//!   ② 同一个状态再读一遍，实例 2 落了盘的根槽全读不出（读者对那几段交「读不到」）⇒ 恢复退到实例 1 的最新根 B，实例 2 的记录一条都不施加、
//!   施加的最大事务号 0（可写挂载写上一个实例那一行时 W 取的就是它，`mount.rs` 的 `PreviousInstanceRow`；D16（发布语义） 已定项 7 那条代价）；
//!   实例 2 一条根都没落盘、记录落了的状态同样退在 B、一条都不施加（前缀不跨实例边界，已定项 14 注 1）。
//! - 可写挂载之后先建一棵树（交回镜像的观察者，豁免表 writable-mount）：在这个状态上可写挂载（实例 3），接着建快照，两个新树号与挂载落到的那一版
//!   树表里的每一个树号都不同、新树表的树号严格升序；挂完建完的盘交判段跑池级 checker。S 的记录落了盘、根没落的那几个状态上恢复由 S 的记录重建出
//!   S 那一版（在飞 checkpoint 里建的树随重放回来），这一判就是 C274 那一格。
//!
//! **比已有的流多罩了什么**：已有的流里实例切换之后新实例的根槽都读得出；崩溃注入那几场有读故障、多次挂载，但不在一条按全部崩溃状态枚举的流上
//! 判「新实例的根全读不出时它的记录一条都不施加」，也没有「重放补回建树的那次发布之后先建树」的状态。这一条在实例 2 那次挂载（取号写、写行那次
//! 十八个单元写、两次暖机）、C、S 的每个崩溃状态上各判一遍，S 那一段是建快照第一次进崩溃点重放的写表。
//! **多跑的一步**：每个状态再读一遍带读故障的恢复（只在实例 2 有根落盘的状态上），与每个状态上可写挂载、建快照。基线镜像、写表、段序列都与已有流不同，
//! 各自全量枚举。
//!
//! 快档按甲二展开（原地覆写取三态的子集、其余写全有或全无）；全量标 ignored，登记成崩溃枚举用例。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use std::collections::{BTreeMap, BTreeSet};

use common::{
    build_pool, file_content, geometry, overwrite_in_process_as_instance_one_and_advance_the_pool,
    parameters, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion, quick_tier_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_amplification::{
    ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver, ObserverDeduplication,
    ReadRangeOnTheCrashImage,
};
use singlefs_checker_tier::crash_point_replay_provenance::{
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow, RunOfTheFlow,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
    CrashPointReplayObserver, CrashPointReplayObserverHandingBackAnImage,
};
use singlefs_checker_tier::writable_mount_phases::materialized_crash_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::JournalRecord;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{recover, JournalPolicy, PoolReader, RecoveryReport};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{
    publish_create_snapshot, publish_overwrite, FirstFile, PoolWriter,
};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_harness::memory_pool::{
    writes_and_segments, CrashImage, MemoryPool, PublishedVersion, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_mounts_with_read_faults_stream.rs";

/// 崩溃点重放项名第一个 `::` 之前那一截（来历表里这条流的名字）。
const FLOW_NAME_AT_THE_ENUMERATION: &str = "mounts_with_read_faults";

/// 实例 1 的覆盖写 B 的内容。
const CONTENT_OF_THE_OVERWRITE_BY_THE_FIRST_INSTANCE: &[u8] =
    b"the first file as overwritten by the first instance before the process exits";

/// 实例 2 的覆盖写 C 的内容。
const CONTENT_OF_THE_OVERWRITE_BY_THE_SECOND_INSTANCE: &[u8] =
    b"the first file as overwritten by the second instance after its writable mount";

/// C 那次写进记录三个时间秒的值比第一个文件晚多少秒。
const SECONDS_OF_THE_SECOND_INSTANCE_OVERWRITE_AFTER_THE_FIRST_WRITE: u64 = 120;

/// 一次建快照从树 ID 水位连发几个树号（码 14 与码 15，`transaction::publish_create_snapshot`）。
const TREE_IDENTIFIERS_ISSUED_BY_A_SNAPSHOT: usize = 2;

/// 不判违例之外，在这条流的状态上至少真被评估过一次的不变量（C467（新补的四条不变量的覆盖率没被穷举用例钉住） 那三条：这条流有两个实例、
/// 每个实例好几次发布，同一 key 的几组、环、反向链都比得出来）。
const INVARIANTS_EVALUATED_ON_SOME_STATE: [&str; 3] = ["I-1.8", "I-7.3", "I-8.6"];

/// 计数名：① 实例 2 有根落盘、恢复落到它记录齐了的最新一次发布。
const NEW_INSTANCE_ROOT_CHOSEN_AND_ITS_COMPLETE_PUBLISHES_APPLIED: &str =
    "new_instance_root_chosen_and_its_complete_publishes_applied";
/// 计数名：② 实例 2 落了盘的根槽全读不出、恢复退到实例 1 的根、实例 2 的记录一条都不施加。
const NEW_INSTANCE_ROOTS_UNREADABLE_AND_NONE_OF_ITS_RECORDS_APPLIED: &str =
    "new_instance_roots_unreadable_and_none_of_its_records_applied";
/// 计数名：实例 2 一条根都没落盘而有记录落了盘、恢复留在实例 1 的根、实例 2 的记录一条都不施加。
const NO_NEW_INSTANCE_ROOT_PERSISTED_AND_NONE_OF_ITS_RECORDS_APPLIED: &str =
    "no_new_instance_root_persisted_and_none_of_its_records_applied";
/// 计数名：实例 2 的写一条记录都没落盘（取号那几次写之内）。
const NO_NEW_INSTANCE_RECORD_PERSISTED: &str = "no_new_instance_record_persisted";
/// 计数名：可写挂载之后建的快照的两个新树号与落到的那一版里的每个树号都不同。
const NEW_SNAPSHOT_TREES_DIFFER_FROM_EVERY_TREE_OF_THE_LANDED_VERSION: &str =
    "new_snapshot_trees_differ_from_every_tree_of_the_landed_version";
/// 计数名：S 的根没落盘、恢复由 S 的记录重建出 S 那一版（在飞 checkpoint 里建的树随重放回来）。
const SNAPSHOT_REPLAYED_FROM_ITS_RECORD: &str = "snapshot_replayed_from_its_record";
/// 判段在可写挂载、建快照之后的盘上判池级 checker、一条不变量都不判违例时加一的计数。
const POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_AND_THE_NEW_SNAPSHOT: &str =
    "pool_checker_clean_after_the_mount_and_the_new_snapshot";

/// 两个观察者另读的输入：实例 2 是哪个号、实例 1 最新那条根、录下的写里实例 2 的根槽写与记录写（按发布分）、S 那一版的根与它的两个树号。
#[derive(Debug)]
struct InputsOfTheObservers {
    new_instance: InstanceGeneration,
    newest_root_of_the_old_instance: (InstanceGeneration, CheckpointTxg),
    root_writes_of_the_new_instance: Vec<usize>,
    /// 实例 2 每次发布（按 txg）的每条记录（按计数器）落在录下的写里的哪几次写（两块盘各一份）。
    record_writes_of_the_new_instance: BTreeMap<CheckpointTxg, BTreeMap<u64, Vec<usize>>>,
    snapshot_root: (InstanceGeneration, CheckpointTxg),
    snapshot_root_write: usize,
    snapshot_trees: Vec<u64>,
}

impl InputsOfTheObservers {
    /// 进观察者的版本与去重键：调试写法取 SHA-256（字段全是这条流录出来的量，录法变了它跟着变）。
    fn digest(&self) -> [u8; DIGEST_BYTES] {
        sha256_digest(format!("{self:?}").as_bytes())
    }
}

/// 录下来的那一段。
struct RecordedMountsWithReadFaults {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    inputs: InputsOfTheObservers,
}

/// 录流：实例 1 的 A、B，之后实例 2 的可写挂载、C、S；基镜像取实例 1 写完 B 的盘面。
fn record_the_mounts_with_read_faults() -> RecordedMountsWithReadFaults {
    let publish_parameters = parameters();
    let mut pool = build_pool("crash-enumeration-mounts-with-read-faults");
    let first_file = pool.output.clone();
    let overwrite_by_the_first_instance = overwrite_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        CONTENT_OF_THE_OVERWRITE_BY_THE_FIRST_INSTANCE,
    );
    let base = pool.memory_pool();
    let operations_before_the_second_mount = pool.stream.retained_operations().len();
    let mut devices = pool.reopen_recorded();
    let mut mounted = mount_writable(&publish_parameters, &mut devices)
        .unwrap_or_else(|error| panic!("实例 2 的可写挂载：{error:?}"));
    let new_instance = mounted.output.instance;
    let landed = mounted
        .current
        .file_version()
        .expect("实例 2 挂载落到 B，带文件")
        .clone();
    let overwrite_by_the_second_instance = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut mounted.allocator,
            &landed,
            FirstFile {
                content: CONTENT_OF_THE_OVERWRITE_BY_THE_SECOND_INSTANCE,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS
                    + SECONDS_OF_THE_SECOND_INSTANCE_OVERWRITE_AFTER_THE_FIRST_WRITE,
            },
            new_instance,
        )
        .unwrap_or_else(|error| panic!("实例 2 覆盖写 C：{error:?}"))
    };
    let snapshot = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_create_snapshot(
            &mut writer,
            &mut mounted.allocator,
            &overwrite_by_the_second_instance,
            new_instance,
        )
        .unwrap_or_else(|error| panic!("实例 2 建快照 S：{error:?}"))
    };
    pool.devices = Some(devices);
    let recorded = pool.stream.retained_operations()[operations_before_the_second_mount..].to_vec();
    let (writes, segments) = writes_and_segments(&recorded, &geometry());
    let filesystem_identifier = publish_parameters.filesystem_identifier;
    let mut root_writes_of_the_new_instance = Vec::new();
    let mut versions = vec![
        PublishedVersion {
            instance: first_file.root.instance,
            checkpoint_txg: first_file.root.checkpoint_txg,
            content: file_content(),
        },
        PublishedVersion {
            instance: overwrite_by_the_first_instance.root.instance,
            checkpoint_txg: overwrite_by_the_first_instance.root.checkpoint_txg,
            content: CONTENT_OF_THE_OVERWRITE_BY_THE_FIRST_INSTANCE.to_vec(),
        },
    ];
    let order_of_the_second_instance_overwrite = (
        overwrite_by_the_second_instance.root.checkpoint_txg,
        overwrite_by_the_second_instance.root.instance,
    );
    let mut record_writes_of_the_new_instance: BTreeMap<CheckpointTxg, BTreeMap<u64, Vec<usize>>> =
        BTreeMap::new();
    let mut snapshot_root_write = None;
    for (index, write) in writes.iter().enumerate() {
        match write.kind {
            StepKind::RootRecordFua => {
                let root = RootRecord::parse_slot(
                    write.bytes().expect("根槽写带字节"),
                    &filesystem_identifier,
                )
                .expect("录下的根槽写自证得过");
                assert_eq!(root.instance, new_instance, "录下的根槽写都是实例 2 写的");
                root_writes_of_the_new_instance.push(index);
                if (root.instance, root.checkpoint_txg)
                    == (snapshot.root.instance, snapshot.root.checkpoint_txg)
                {
                    snapshot_root_write = Some(index);
                }
                let content = if (root.checkpoint_txg, root.instance)
                    >= order_of_the_second_instance_overwrite
                {
                    CONTENT_OF_THE_OVERWRITE_BY_THE_SECOND_INSTANCE
                } else {
                    CONTENT_OF_THE_OVERWRITE_BY_THE_FIRST_INSTANCE
                };
                versions.push(PublishedVersion {
                    instance: root.instance,
                    checkpoint_txg: root.checkpoint_txg,
                    content: content.to_vec(),
                });
            }
            StepKind::JournalRecord => {
                let record = JournalRecord::parse(
                    write.bytes().expect("记录写带字节"),
                    unit_filesystem_identifier(&filesystem_identifier),
                )
                .expect("录下的记录写自证得过");
                assert_eq!(record.instance, new_instance, "录下的记录写都是实例 2 写的");
                record_writes_of_the_new_instance
                    .entry(record.checkpoint_txg)
                    .or_default()
                    .entry(record.counter)
                    .or_default()
                    .push(index);
            }
            StepKind::ZeroFill
            | StepKind::Discard
            | StepKind::UnitWrite
            | StepKind::SystemConfigurationSlot
            | StepKind::Barrier => {}
        }
    }
    let judged_root_index = *root_writes_of_the_new_instance
        .last()
        .expect("实例 2 那次挂载与之后两次发布写了根槽");
    let snapshot_trees: Vec<u64> = snapshot
        .tree_table_entries
        .iter()
        .map(|entry| entry.tree.0)
        .filter(|tree| {
            !overwrite_by_the_second_instance
                .tree_table_entries
                .iter()
                .any(|entry| entry.tree.0 == *tree)
        })
        .collect();
    assert_eq!(
        snapshot_trees.len(),
        TREE_IDENTIFIERS_ISSUED_BY_A_SNAPSHOT,
        "建快照 S 从水位连发两个树号"
    );
    println!(
        "MOUNTS_WITH_READ_FAULTS new_instance={} segments={:?} root_writes={root_writes_of_the_new_instance:?} snapshot_trees={snapshot_trees:?}",
        new_instance.0,
        segments.iter().map(Vec::len).collect::<Vec<usize>>()
    );
    RecordedMountsWithReadFaults {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        inputs: InputsOfTheObservers {
            new_instance,
            newest_root_of_the_old_instance: (
                overwrite_by_the_first_instance.root.instance,
                overwrite_by_the_first_instance.root.checkpoint_txg,
            ),
            root_writes_of_the_new_instance,
            record_writes_of_the_new_instance,
            snapshot_root: (snapshot.root.instance, snapshot.root.checkpoint_txg),
            snapshot_root_write: snapshot_root_write.expect("S 的根槽写在录下的写里"),
            snapshot_trees,
        },
    }
}

/// 同一张崩溃镜像，几段读起来交「读不到」（块设备报读错时恢复看到的就是 `None`）：实例 2 落了盘的根槽。
struct CrashImageWithUnreadableRanges<'image> {
    image: &'image CrashImage<'image>,
    /// （盘，起始字节，结束字节）。
    unreadable_ranges: Vec<(DeviceIdentity, u64, u64)>,
}

impl PoolReader for CrashImageWithUnreadableRanges<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.image.device_identities()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.image.device_size_in_bytes(device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        let end = offset.0 + u64::try_from(length).expect("一次读的长度装得进 u64");
        let overlaps_an_unreadable_range =
            self.unreadable_ranges
                .iter()
                .any(|(unreadable_device, start, unreadable_end)| {
                    *unreadable_device == device && offset.0 < *unreadable_end && *start < end
                });
        if overlaps_an_unreadable_range {
            return None;
        }
        self.image.read(device, offset, length)
    }
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.image
            .journal_record_offsets_hint(device, ring_start, ring_bytes)
    }
}

/// 新实例的记录施加到哪那个观察者看一个状态（文件头 ① ② 两格）。只记计数，判不对就 panic（记成红项）。
///
/// # Panics
/// ① 恢复没落到实例 2 记录齐了的最新一次发布；② 读不出实例 2 的根之后恢复没退到实例 1 的最新根、或施加了实例 2 的记录；
/// 实例 2 没有根落盘时施加了它的记录。
// gpu-exempt: observer-assertions-on-the-recovery-report
fn observe_where_the_new_instance_records_are_applied(
    inputs: &InputsOfTheObservers,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    let is_persisted = |index: &usize| image.persisted[*index];
    let complete_publishes: Vec<CheckpointTxg> = inputs
        .record_writes_of_the_new_instance
        .iter()
        .filter(|(_, records)| {
            records
                .values()
                .all(|copies| copies.iter().any(is_persisted))
        })
        .map(|(txg, _)| *txg)
        .collect();
    let some_new_instance_record_persisted = inputs
        .record_writes_of_the_new_instance
        .values()
        .flat_map(BTreeMap::values)
        .flatten()
        .any(is_persisted);
    let new_instance_root_writes_persisted: Vec<&RetainedWrite> = inputs
        .root_writes_of_the_new_instance
        .iter()
        .filter(|&&index| image.persisted[index])
        .map(|&index| &image.writes[index])
        .collect();
    if new_instance_root_writes_persisted.is_empty() {
        if !some_new_instance_record_persisted {
            counts.add(NO_NEW_INSTANCE_RECORD_PERSISTED, 1);
            return;
        }
        assert_eq!(
            (
                report.effective_root,
                report.journal.prefix_applied,
                report.journal.maximum_applied_transaction
            ),
            (Some(inputs.newest_root_of_the_old_instance), 0, 0),
            "实例 2 一条根都没落盘、记录落了：恢复留在实例 1 的最新根，实例 2 的记录一条都不施加"
        );
        counts.add(
            NO_NEW_INSTANCE_ROOT_PERSISTED_AND_NONE_OF_ITS_RECORDS_APPLIED,
            1,
        );
        return;
    }
    let newest_complete_publish = complete_publishes
        .last()
        .copied()
        .expect("实例 2 有根落了盘：那次发布的记录在根槽写之前就落了盘");
    assert_eq!(
        report.effective_root,
        Some((inputs.new_instance, newest_complete_publish)),
        "① 所选根是实例 2 的根：水位之后实例 2 的记录全部施加，恢复落到记录齐了的最新一次发布"
    );
    counts.add(
        NEW_INSTANCE_ROOT_CHOSEN_AND_ITS_COMPLETE_PUBLISHES_APPLIED,
        1,
    );
    let faulted = CrashImageWithUnreadableRanges {
        image,
        unreadable_ranges: new_instance_root_writes_persisted
            .iter()
            .map(|write| {
                (
                    write.device,
                    write.offset.0,
                    write.offset.0 + write.length_in_bytes(),
                )
            })
            .collect(),
    };
    let report_with_the_new_instance_roots_unreadable = recover(&faulted, JournalPolicy::Consult);
    assert_eq!(
        (
            report_with_the_new_instance_roots_unreadable.effective_root,
            report_with_the_new_instance_roots_unreadable
                .journal
                .prefix_applied,
            report_with_the_new_instance_roots_unreadable
                .journal
                .maximum_applied_transaction
        ),
        (Some(inputs.newest_root_of_the_old_instance), 0, 0),
        "② 实例 2 落了盘的根全读不出：恢复退到实例 1 的最新根，实例 2 的记录一条都不施加、施加的最大事务号（写行的 W）是 0"
    );
    counts.add(
        NEW_INSTANCE_ROOTS_UNREADABLE_AND_NONE_OF_ITS_RECORDS_APPLIED,
        1,
    );
}

/// 可写挂载之后先建一棵树那个观察者看一个状态（文件头那一段，C274 那一格）：在崩溃镜像落成的盘的副本上可写挂载（实例 3），接着建快照；
/// 两个新树号不在落到的那一版的树表里、新树表的树号严格升序。挂载与建快照的写录在一条新的录制流上，连同盘交回：判段在盘上跑池级 checker。
///
/// # Panics
/// 可写挂载或建快照做不成，或新树号撞上落到的那一版里的树号。
// gpu-exempt: writable-mount
fn observe_a_new_tree_after_the_writable_mount(
    inputs: &InputsOfTheObservers,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) -> ImageAfterTheObserver {
    let mount_parameters = parameters();
    let materialized = materialized_crash_image(image);
    let stream = SharedStream::retaining_contents_and_reads();
    let mut devices: Vec<_> = materialized
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                materialized.device_size_in_bytes,
                PhysicalBlockSizeInBytes(mount_parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect();
    let mut mounted = mount_writable(&mount_parameters, &mut devices)
        .unwrap_or_else(|error| panic!("每个崩溃状态上可写挂载都做得成：{error:?}"));
    let landed = mounted
        .current
        .file_version()
        .expect("这条流里每一版都带文件")
        .clone();
    let trees_of_the_landed_version: BTreeSet<u64> = landed
        .tree_table_entries
        .iter()
        .map(|entry| entry.tree.0)
        .collect();
    if report.effective_root == Some(inputs.snapshot_root)
        && !image.persisted[inputs.snapshot_root_write]
    {
        assert!(
            inputs
                .snapshot_trees
                .iter()
                .all(|tree| trees_of_the_landed_version.contains(tree)),
            "S 的根没落盘、恢复由 S 的记录重建出 S 那一版：挂载落到的那一版带着 S 的两个树号 {:?}，实际 {trees_of_the_landed_version:?}",
            inputs.snapshot_trees
        );
        counts.add(SNAPSHOT_REPLAYED_FROM_ITS_RECORD, 1);
    }
    let created = {
        let mut writer = PoolWriter::new(&mount_parameters, devices.as_mut_slice());
        publish_create_snapshot(
            &mut writer,
            &mut mounted.allocator,
            &landed,
            mounted.output.instance,
        )
        .unwrap_or_else(|error| panic!("可写挂载之后建快照：{error:?}"))
    };
    let trees_after: Vec<u64> = created
        .tree_table_entries
        .iter()
        .map(|entry| entry.tree.0)
        .collect();
    assert!(
        trees_after.windows(2).all(|pair| pair[0] < pair[1]),
        "建快照之后树表的树号严格升序、两两不同：{trees_after:?}"
    );
    let new_trees: Vec<u64> = trees_after
        .iter()
        .copied()
        .filter(|tree| !trees_of_the_landed_version.contains(tree))
        .collect();
    assert_eq!(
        new_trees.len(),
        TREE_IDENTIFIERS_ISSUED_BY_A_SNAPSHOT,
        "可写挂载之后建的快照发了两个新树号，都不在落到的那一版（{:?}）里：落到的那一版 {trees_of_the_landed_version:?}，之后 {trees_after:?}",
        report.effective_root
    );
    counts.add(
        NEW_SNAPSHOT_TREES_DIFFER_FROM_EVERY_TREE_OF_THE_LANDED_VERSION,
        1,
    );
    let disks_after_the_observer = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: materialized.device_size_in_bytes,
    };
    let (writes_on_the_crash_image, _segments_of_the_mount) =
        writes_and_segments(&stream.retained_operations(), &geometry());
    ImageAfterTheObserver {
        writes_on_the_crash_image,
        disks_after_the_observer,
        ranges_read_on_the_crash_image: ReadRangeOnTheCrashImage::of_the_recorded_read_ranges(
            &stream.read_ranges(),
        ),
        invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
        counter_when_the_pool_checker_is_clean:
            POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_AND_THE_NEW_SNAPSHOT.to_string(),
    }
}

/// 按 `expansion` 枚举这条流，挂两个观察者；项名是 `<流名>::<标签>`。
fn enumerate_the_mounts_with_read_faults(
    recorded: &RecordedMountsWithReadFaults,
    item_label: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> Option<CrashPointReplayTally> {
    let inputs = &recorded.inputs;
    let observe_the_records =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_where_the_new_instance_records_are_applied(inputs, image, report, counts);
        };
    let observe_the_new_tree = |_ordinal: u64,
                                image: &CrashImage<'_>,
                                report: &RecoveryReport,
                                counts: &mut CrashPointReplayObserverCounts|
     -> ImageAfterTheObserver {
        observe_a_new_tree_after_the_writable_mount(inputs, image, report, counts)
    };
    let item_name = format!("{FLOW_NAME_AT_THE_ENUMERATION}::{item_label}");
    enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(&recorded.versions),
        expansion,
        // 两个观察者另读的输入都是 `InputsOfTheObservers`（实例号、实例 1 最新根、实例 2 的根槽写与记录写、S 的根与树号）
        &[CrashPointReplayObserver {
            name: "where_the_new_instance_records_are_applied",
            extra_inputs_digest: inputs.digest(),
            observe: &observe_the_records,
        }],
        &[CrashPointReplayObserverHandingBackAnImage {
            name: "a_new_tree_after_the_writable_mount",
            extra_inputs_digest: inputs.digest(),
            observe: &observe_the_new_tree,
            deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        }],
    )
    .judged()
}

/// 判器判的几样一个都不红、名单里的不变量都真被评估过；两个观察者看过每一个状态、各格都真跑到了。
fn assert_the_judges_and_the_observers_are_clean(tally: &CrashPointReplayTally) {
    assert_eq!(tally.violations, 0, "oracle：{:?}", tally.first_violation);
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0, "走读一次都不失败");
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant} 判违例：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
    for invariant in INVARIANTS_EVALUATED_ON_SOME_STATE {
        assert!(
            tally
                .checker_evaluated_states
                .get(invariant)
                .copied()
                .unwrap_or(0)
                > 0,
            "{invariant} 至少在一个状态上真被评估过：{}",
            tally.checker_counts_by_invariant()
        );
    }
    assert_eq!(tally.observed_states, tally.states, "每个状态都看过");
    let counts = &tally.observer_counts;
    assert_eq!(
        counts.count_named(NEW_INSTANCE_ROOT_CHOSEN_AND_ITS_COMPLETE_PUBLISHES_APPLIED)
            + counts.count_named(NO_NEW_INSTANCE_ROOT_PERSISTED_AND_NONE_OF_ITS_RECORDS_APPLIED)
            + counts.count_named(NO_NEW_INSTANCE_RECORD_PERSISTED),
        tally.states,
        "新实例的记录施加到哪：每个状态恰好落在一格：{counts:?}"
    );
    for counter in [
        NEW_INSTANCE_ROOT_CHOSEN_AND_ITS_COMPLETE_PUBLISHES_APPLIED,
        NEW_INSTANCE_ROOTS_UNREADABLE_AND_NONE_OF_ITS_RECORDS_APPLIED,
        NO_NEW_INSTANCE_ROOT_PERSISTED_AND_NONE_OF_ITS_RECORDS_APPLIED,
        SNAPSHOT_REPLAYED_FROM_ITS_RECORD,
    ] {
        assert!(
            counts.count_named(counter) > 0,
            "{counter} 那一格的状态真跑到了：{counts:?}"
        );
    }
    assert_eq!(
        counts.count_named(NEW_INSTANCE_ROOTS_UNREADABLE_AND_NONE_OF_ITS_RECORDS_APPLIED),
        counts.count_named(NEW_INSTANCE_ROOT_CHOSEN_AND_ITS_COMPLETE_PUBLISHES_APPLIED),
        "实例 2 有根落盘的每个状态都再读一遍带读故障的恢复：{counts:?}"
    );
    assert_eq!(
        counts.count_named(NEW_SNAPSHOT_TREES_DIFFER_FROM_EVERY_TREE_OF_THE_LANDED_VERSION),
        tally.states,
        "每个状态都可写挂载、建快照，新树号都不撞：{counts:?}"
    );
    assert_eq!(
        counts.count_named(POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_AND_THE_NEW_SNAPSHOT),
        tally.states,
        "每个状态挂完建完的盘池级 checker 0 违例：{counts:?}"
    );
}

/// 平时跑的那一份：每段按甲二展开（原地覆写取三态的子集，其余写全有或全无）；两个观察者都挂。
#[test]
fn every_crash_state_of_a_second_writable_mount_applies_the_new_instance_records_only_above_its_own_roots_and_issues_fresh_tree_identifiers(
) {
    let recorded = record_the_mounts_with_read_faults();
    let Some(tally) =
        enumerate_the_mounts_with_read_faults(&recorded, "fast", &quick_tier_expansion)
    else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_MOUNTS_WITH_READ_FAULTS_FAST states={} observer {:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_the_judges_and_the_observers_are_clean(&tally);
    assert_eq!(
        tally.states,
        crash_state_count_with_torn_in_place_overwrites(
            &recorded.base,
            &recorded.writes,
            &recorded.segments,
            &quick_tier_expansion,
        ),
        "甲二展开按崩溃点重放的枚举域数（原地覆写取三态）"
    );
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow(
        RunOfTheFlow::CrashPointReplayEnumeration,
        FLOW_NAME_AT_THE_ENUMERATION,
        &tally,
    );
}

/// 全量：每段都展开，每个状态两遍恢复 + checker + 两个观察者。登记成崩溃枚举用例，
/// checker-tier-crash-point-replay 认下面打印的 `CRASH_POINT_REPLAY_MOUNTS_WITH_READ_FAULTS` 行里 `exhaustive=true`。
#[test]
#[ignore = "全量约 1800 万个状态（按草稿副本录出的 22 段、最大段 24 写推的两态闭式，崩溃点重放的枚举域以用例打出的为准）、每个两遍恢复 + checker + 带读故障的恢复 + 可写挂载建快照；门禁 checker-tier-crash-point-replay --full 在 release 下跑，可双机分片"]
fn full_enumeration_of_the_mounts_with_read_faults_stream_is_exhaustive_and_clean() {
    let recorded = record_the_mounts_with_read_faults();
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &full_expansion,
    );
    let Some(tally) = enumerate_the_mounts_with_read_faults(&recorded, "full", &full_expansion)
    else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_MOUNTS_WITH_READ_FAULTS states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_the_judges_and_the_observers_are_clean(&tally);
    assert_eq!(tally.states, closed_form);
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow(
        RunOfTheFlow::CrashPointReplayEnumeration,
        FLOW_NAME_AT_THE_ENUMERATION,
        &tally,
    );
}
