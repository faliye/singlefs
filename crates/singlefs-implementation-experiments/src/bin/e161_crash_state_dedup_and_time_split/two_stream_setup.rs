//! 两条流的搭建（停机条款 S2）：新池新建文件流与第二条流的写表、段表与形状核对。

use super::*;

// ───────────────────────────── 两条流的搭建（S2） ─────────────────────────────

/// 哪一条流（登记第一节「读法写死」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum StreamName {
    /// 崩溃点重放的新池新建文件流：mkfs → 取号 → 暖机两次 → 新池新建文件。
    First,
    /// 崩溃点重放的覆盖写固定脚本流：固定脚本到 E 再正常卸载，取段 12–28。
    Second,
}

impl StreamName {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::First => "first",
            Self::Second => "second",
        }
    }
}

/// 一条流搭好之后的样子：基镜像（mkfs 之后）、录制流的写表与段、被判的根、版本表。
pub(super) struct PreparedStream {
    pub(super) name: StreamName,
    pub(super) base: MemoryPool,
    pub(super) writes: Vec<RetainedWrite>,
    pub(super) segments: Vec<Vec<usize>>,
    pub(super) judged_root_index: usize,
    pub(super) versions: Vec<PublishedVersion>,
    pub(super) root_write_count: usize,
}

/// 新池新建文件流：照 `tests/crash_enumeration_new_pool_file_creation_stream.rs` 的 `prepare`（S2 的段长与写数断言照它）。
pub(super) fn prepare_first_stream() -> PreparedStream {
    let pool = experiment_pool::build_pool("e161-first");
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments) = singlefs_verification::memory_pool::writes_and_segments(
        &operations[pool.mkfs_operation_count..],
        &experiment_pool::geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("写流里有根槽那一条");
    let (instance, checkpoint_txg) = root_identity_of_root_write(&writes[judged_root_index]);
    let root_write_count = writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .count();
    PreparedStream {
        name: StreamName::First,
        base,
        writes,
        segments,
        judged_root_index,
        versions: vec![PublishedVersion {
            instance,
            checkpoint_txg,
            content: experiment_pool::file_content(),
        }],
        root_write_count,
    }
}

/// 根槽写写下的根身份：实例代号在偏移 24、checkpoint_txg 在偏移 28（`crash.rs` 的 `root_identity_written_by` 同一读法）。
fn root_identity_of_root_write(write: &RetainedWrite) -> (InstanceGeneration, CheckpointTxg) {
    let bytes = write.bytes().expect("根槽写带着字节");
    (
        InstanceGeneration(u32::from_le_bytes(
            bytes[24..28].try_into().expect("4 字节"),
        )),
        CheckpointTxg(u64::from_le_bytes(
            bytes[28..36].try_into().expect("8 字节"),
        )),
    )
}

/// 覆盖写固定脚本流的内容与脚本照抄 `crash_enumeration_fixed_script_stream.rs` 里的同名常量与函数（`SECOND_FILE_BYTES` 到 `writes_and_segments_after_mkfs` 那一串）。
const SECOND_FILE_BYTES: usize = 4100;

fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

const THIRD_FILE_BYTES: usize = 2500;

fn later_content(seed: usize) -> Vec<u8> {
    (0..3000 + seed)
        .map(|index| u8::try_from((index * 5 + seed) % 251).expect("小于 256"))
        .collect()
}

fn third_content() -> Vec<u8> {
    (0..THIRD_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
        .collect()
}

fn overwrite(
    pool: &mut experiment_pool::BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> singlefs_core::transaction::TransactionOutput {
    let parameters = experiment_pool::parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer =
        singlefs_core::transaction::PoolWriter::new(&parameters, devices.as_mut_slice());
    singlefs_core::transaction::publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &pool.output,
        singlefs_core::transaction::FirstFile {
            content,
            write_time_seconds: experiment_pool::FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写")
}

fn unmount_recorded(
    pool: &mut experiment_pool::BuiltPool,
) -> singlefs_core::mount::UnmountRaisedTheFloor {
    let mut current =
        singlefs_core::transaction::PoolVersion::WithFile(std::sync::Arc::new(pool.output.clone()));
    let stream = pool.stream.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let allocator = &mut pool.allocator;
    let unmounted = stream
        .record_entry(singlefs_verification::RecordedPublishEntry::Unmount, || {
            singlefs_core::mount::unmount(
                &experiment_pool::parameters(),
                devices,
                allocator,
                &mut current,
                singlefs_core::mount::ShadowLedger::On,
            )
        })
        .expect("正常卸载");
    pool.output = current
        .into_file_version()
        .expect("带文件的一版卸载之后仍带文件");
    match unmounted {
        singlefs_core::mount::Unmounted::FloorRaisedToTheCurrentVersion(raised) => raised,
        singlefs_core::mount::Unmounted::NothingWrittenOnAVersionWithoutFile {
            current: reported_version,
        } => panic!("带文件的一版上卸载报成「一个字节都不写」：{reported_version:?}"),
    }
}

fn writes_and_segments_after_mkfs(
    pool: &experiment_pool::BuiltPool,
) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
    let operations = pool.retained_operations();
    let entry_spans: Vec<singlefs_verification::RecordedEntrySpan> = pool
        .stream
        .entry_spans()
        .into_iter()
        .map(|span| singlefs_verification::RecordedEntrySpan {
            entry: span.entry,
            operations: span.operations.start - pool.mkfs_operation_count
                ..span.operations.end - pool.mkfs_operation_count,
        })
        .collect();
    let (writes, segments, _stream_indexes) =
        singlefs_verification::memory_pool::writes_and_segments_with_stream_indexes_and_entries(
            &operations[pool.mkfs_operation_count..],
            &entry_spans,
            &experiment_pool::geometry(),
        );
    (writes, segments)
}

fn version(instance: u32, txg: u64, content: Vec<u8>) -> PublishedVersion {
    PublishedVersion {
        instance: InstanceGeneration(instance),
        checkpoint_txg: CheckpointTxg(txg),
        content,
    }
}

/// 覆盖写固定脚本流：`crash_enumeration_fixed_script_stream.rs` 的 `prepare`，`Script::ReuseAfterRaisingFloorThenNormalUnmount` 那一支。
/// 脚本里的断言（E 的数据单元落在 50176、卸载两次空发布是 txg 18、19）原样留着，S2 另核段长表、写数与根数。
pub(super) fn prepare_second_stream() -> PreparedStream {
    let mut pool = experiment_pool::build_pool("e161-second");
    let second = overwrite(&mut pool, &second_content(), InstanceGeneration(1));
    let mut versions = vec![
        version(1, 3, experiment_pool::file_content()),
        version(1, 4, second_content()),
    ];
    pool.output = second;
    let mut devices = pool.reopen_recorded();
    let mounted =
        singlefs_core::mount::mount_writable(&experiment_pool::parameters(), &mut devices)
            .expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    for txg in 5..=7 {
        versions.push(version(2, txg, second_content()));
    }
    let third = overwrite(&mut pool, &third_content(), InstanceGeneration(2));
    versions.push(version(2, 8, third_content()));
    pool.output = third;
    {
        let rollback_parameters = experiment_pool::parameters();
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        singlefs_core::mount::roll_back_by_a_forward_publish(
            &rollback_parameters,
            open_devices,
            &mut pool.allocator,
            &mut pool.output,
            singlefs_core::mount::RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
            },
        )
        .expect("挂着的时候回退到 A");
    }
    versions.push(version(2, 9, experiment_pool::file_content()));
    let mut latest = Vec::new();
    for (txg, seed) in [(10u64, 13usize), (11, 17), (12, 19), (13, 23), (14, 29)] {
        latest = later_content(seed);
        pool.output = overwrite(&mut pool, &latest, InstanceGeneration(2));
        versions.push(version(2, txg, latest.clone()));
    }
    let mut current = pool.output.clone();
    let raise_devices = pool.devices.as_mut().expect("镜像还开着");
    let raised = singlefs_core::mount::raise_rollback_floor(
        &experiment_pool::parameters(),
        raise_devices,
        &mut pool.allocator,
        &mut current,
        CheckpointTxg(11),
        singlefs_core::mount::ShadowLedger::On,
    )
    .expect("抬 F");
    assert_eq!(raised.publishes.len(), 2, "txg 15 落盘 0、txg 16 落盘 1");
    pool.output = current;
    for txg in 15..=16 {
        versions.push(version(2, txg, latest.clone()));
    }
    let reuse = overwrite(&mut pool, &later_content(31), InstanceGeneration(2));
    assert_eq!(
        reuse.data_pointers[0].locations[0].slot.0, 50176,
        "S2：E 的数据单元落回 50176"
    );
    versions.push(version(2, 17, later_content(31)));
    pool.output = reuse;
    let unmounted = unmount_recorded(&mut pool);
    assert_eq!(
        unmounted
            .publishes
            .iter()
            .map(|publish| publish.root.checkpoint_txg)
            .collect::<Vec<_>>(),
        vec![CheckpointTxg(18), CheckpointTxg(19)],
        "S2：卸载推两次空发布 txg 18、19"
    );
    for txg in 18..=19 {
        versions.push(version(2, txg, later_content(31)));
    }
    let base = pool.memory_pool_after_mkfs();
    let (writes, segments) = writes_and_segments_after_mkfs(&pool);
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    PreparedStream {
        name: StreamName::Second,
        base,
        judged_root_index: *root_indexes.last().expect("至少一条根槽写"),
        root_write_count: root_indexes.len(),
        writes,
        segments,
        versions,
    }
}

/// S2 的锚点（第 3 次重跑登记 A8，取号写两轮之后的段形：每次取号两轮、轮后各一道池屏障，各成一个 2 写段）：
/// 新池新建文件流 12 段的段长、43 次写
/// （`crash_enumeration_new_pool_file_creation_stream.rs` 的 `prepare` 里段长与写数那两条 `assert_eq!`）；
/// 覆盖写固定脚本流 80 段的段长表、485 次写、19 条根（`crash_enumeration_fixed_script_stream.rs` 的 `prepare` 里「固定脚本到 E 再正常卸载的段序列」那条断言、紧跟的写数断言与
/// `expected_roots`，到 E 再正常卸载那一支）。
const FIRST_STREAM_SEGMENT_LENGTHS: [usize; 12] = [2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2];
const FIRST_STREAM_WRITES: usize = 43;
/// 新池新建文件流的根槽写：暖机两次各一条、新池新建文件一条（段表里三个 1 写段，收严：登记 S2 没列这一格）。
const FIRST_STREAM_ROOTS: usize = 3;
const SECOND_STREAM_SEGMENT_LENGTHS: [usize; 80] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2, 16, 2, 1, 2,
    24, 2, 1, 2, 24, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 2,
    16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2,
];
pub(super) const SECOND_STREAM_WRITES: usize = 485;
const SECOND_STREAM_ROOTS: usize = 19;
/// 覆盖写固定脚本流取段 12–28（第 3 次重跑登记 1.3「第二条流的取样段」：第一条流整条之后，到 txg 7 的根槽为止）；
/// 段 0–11 与新池新建文件流段 0–11 逐字节相同（S2），不枚举。
pub(super) const SECOND_STREAM_FIRST_SAMPLED_SEGMENT: usize = 12;
pub(super) const SECOND_STREAM_LAST_SAMPLED_SEGMENT: usize = 28;
pub(super) const SHARED_PREFIX_SEGMENTS: usize = 12;

/// S2 的一行：每条判据各自一个布尔。
pub(super) struct StreamCheck {
    pub(super) segment_lengths_match: bool,
    pub(super) writes_match: bool,
    pub(super) roots_match: bool,
}

pub(super) fn check_stream_shape(stream: &PreparedStream) -> StreamCheck {
    let lengths: Vec<usize> = stream.segments.iter().map(Vec::len).collect();
    match stream.name {
        StreamName::First => StreamCheck {
            segment_lengths_match: lengths == FIRST_STREAM_SEGMENT_LENGTHS,
            writes_match: stream.writes.len() == FIRST_STREAM_WRITES,
            roots_match: stream.root_write_count == FIRST_STREAM_ROOTS,
        },
        StreamName::Second => StreamCheck {
            segment_lengths_match: lengths == SECOND_STREAM_SEGMENT_LENGTHS,
            writes_match: stream.writes.len() == SECOND_STREAM_WRITES,
            roots_match: stream.root_write_count == SECOND_STREAM_ROOTS,
        },
    }
}

/// S2：两条流的基镜像逐字节相同；覆盖写固定脚本流段 0–11 的写与新池新建文件流段 0–11 逐字节相同。
pub(super) fn shared_prefix_is_identical(
    first: &PreparedStream,
    second: &PreparedStream,
) -> (bool, bool) {
    let bases_identical = first.base == second.base;
    let prefix_of = |stream: &PreparedStream| -> Vec<RetainedWrite> {
        stream.segments[..SHARED_PREFIX_SEGMENTS]
            .iter()
            .flatten()
            .map(|write_index| stream.writes[*write_index].clone())
            .collect()
    };
    (bases_identical, prefix_of(first) == prefix_of(second))
}
