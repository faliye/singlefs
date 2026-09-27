//! 实审 B3c-2（代码审阅第 2 条的收尾，用户 2026-09-27 定案「崩溃注入层补可写挂载」）：崩溃注入第二、三截交给记录核对器的记录流。
//!
//! 第二截（崩溃后镜像上可写挂载、再发一次布之后的池）与第三截（挂载途中的二次崩溃）交的都是整条历史录制流接上挂载那一段录制流，
//! 在接缝处断开（`crash::RecordStreamContinuity::ResumedAfterACrash`，`crash_injection::HistoryThenWritableMountRecords`）；
//! 崩溃后镜像是两份：崩溃态镜像判择根与前缀，实现恢复后的镜像判在不在（D13（验证路线） 已定项 7）。
//! 此前第三截只交挂载那一段到当前段为止的前缀，历史那几次发布在二次崩溃镜像上还在不在没人核（B3c-1 报告第六节第 2 条）；
//! 第二截只跑 checker，没接记录核对器（B3b 报告第二节第 5 条）；直接把历史写表接在挂载前面会假红（B3b 报告第二节第 4 条）。
//!
//! 这里钉三样：记录核对器的接口（两份镜像各判什么、接缝处怎么分发布，手摆的写表）；真实历史上两件活各一个会红的状态
//! （二次崩溃镜像上、挂载之后的池上丢了一次历史发布的单元）；不假红的对照（第一次崩溃截在一次发布中间、落到的那一版的实例表
//! 只在恢复后的镜像上、崩溃注入整段跑下来一条新发现都没有）。

use std::collections::BTreeMap;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::recovery::{recover, JournalPolicy, PoolReader};
use singlefs_harness::crash::{
    check_records_against, root_identity_written_by, some_publish_persisted_without_its_root,
    writes_and_segments, writes_and_segments_with_stream_indexes, CrashImage, MemoryPool,
    RecordCheck, RecordStreamContinuity, RetainedWrite, WrittenContents,
};
use singlefs_harness::crash_injection::{
    inject_crashes_into_history, materialized_crash_image,
    mount_and_publish_once_on_the_crash_image, CrashInjectionTally, CrashPoint, CrashPointDraw,
    HistoryThenWritableMountRecords, WritableMountOnTheCrashImage, WritableMountOutcome,
};
use singlefs_harness::history::{
    execute_history_with, generate_history, ContentChoice, ContentLength, GeneratedHistory,
    HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed,
    HistoryStartingPoint, PerStepChecker, StepPosition,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 历史本身不跑每一步的 checker，两块 4 GiB 的盘（单元区墙够不着，挂载与之后那一次发布都该成），与崩溃注入快档同一种跑法。
const UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::FourGibibytes,
    space_admission: SpaceAdmission::JudgedByTheFormula,
};

/// 手摆写表那几条用例用的盘：一块 1 MiB 的盘，写都是整扇区。
const HAND_LAID_DEVICE_BYTES: u64 = 1 << 20;
const HAND_LAID_WRITE_BYTES: usize = 512;

fn hand_laid_write(kind: StepKind, offset: u64, bytes: Vec<u8>) -> RetainedWrite {
    RetainedWrite {
        device: DeviceIdentity(0),
        kind,
        is_force_unit_access: kind == StepKind::RootRecordFua,
        offset: DeviceOffsetInBytes(offset),
        contents: WrittenContents::Bytes(bytes),
    }
}

/// 一次单元写：一个扇区，字节全是 `fill`。
fn hand_laid_unit(offset: u64, fill: u8) -> RetainedWrite {
    hand_laid_write(
        StepKind::UnitWrite,
        offset,
        vec![fill; HAND_LAID_WRITE_BYTES],
    )
}

/// 一次 journal 记录写：一个扇区，字节全是 `fill`。
fn hand_laid_record(offset: u64, fill: u8) -> RetainedWrite {
    hand_laid_write(
        StepKind::JournalRecord,
        offset,
        vec![fill; HAND_LAID_WRITE_BYTES],
    )
}

/// 一次根槽写：一个扇区，只填根身份（实例代号在偏移 24、checkpoint_txg 在偏移 28），别的全 0——实例表指针也是 0，
/// 这条根的实例表读不出，记录核对器一次发布都不按实例表判被抛弃（判据不放宽）。
fn hand_laid_root(offset: u64, instance: u32, checkpoint_txg: u64) -> RetainedWrite {
    let mut bytes = vec![0_u8; HAND_LAID_WRITE_BYTES];
    bytes[24..28].copy_from_slice(&instance.to_le_bytes());
    bytes[28..36].copy_from_slice(&checkpoint_txg.to_le_bytes());
    hand_laid_write(StepKind::RootRecordFua, offset, bytes)
}

/// 一块空盘上叠 `writes` 里 `landed` 为真的那几个写。
fn hand_laid_pool(writes: &[RetainedWrite], landed: &[bool]) -> MemoryPool {
    let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0)], HAND_LAID_DEVICE_BYTES);
    for (write, is_landed) in writes.iter().zip(landed) {
        if *is_landed {
            pool.apply_writes(std::slice::from_ref(write));
        }
    }
    pool
}

fn root_identity(instance: u32, checkpoint_txg: u64) -> (InstanceGeneration, CheckpointTxg) {
    (InstanceGeneration(instance), CheckpointTxg(checkpoint_txg))
}

/// 崩溃后镜像两份各判什么（D13（验证路线） 已定项 7）：一次发布 (1, 1) 的根槽写在盘上、它的记录在崩溃态镜像上没落，
/// 恢复之后记录在了、它的单元却没了。「根在而记录一条都不在」看崩溃态镜像（判红），「恢复自称新态而单元缺席」看恢复后的镜像（判红）；
/// 两份对调交进去，两条都不判——两条判据各读各的那一份，读错一份就少报一条。
#[test]
fn the_root_and_its_records_are_judged_on_the_crash_state_image_and_unit_presence_on_the_recovered_image(
) {
    let writes = vec![
        hand_laid_unit(0x1_0000, 0xa1),
        hand_laid_record(0x2_0000, 0xb1),
        hand_laid_root(0x3_0000, 1, 1),
    ];
    let persisted = vec![true, false, true];
    let crash_state_image = hand_laid_pool(&writes, &persisted);
    let recovered_image = hand_laid_pool(&writes, &[false, true, true]);
    assert_eq!(
        check_records_against(
            &crash_state_image,
            &recovered_image,
            &writes,
            &persisted,
            RecordStreamContinuity::OneRecording,
            Some(root_identity(1, 1)),
        ),
        RecordCheck {
            root_without_record: true,
            claimed_state_missing_unit: true,
        },
        "崩溃态镜像上 (1, 1) 的根在、记录不在；恢复后的镜像上 (1, 1) 的单元不在：两条都判"
    );
    assert_eq!(
        check_records_against(
            &recovered_image,
            &crash_state_image,
            &writes,
            &persisted,
            RecordStreamContinuity::OneRecording,
            Some(root_identity(1, 1)),
        ),
        RecordCheck::default(),
        "两份对调：崩溃态那一份上记录在、恢复后那一份上单元在，两条都不判"
    );
}

/// 接缝处只留崩溃之后那条时间线上的发布：截断的那条流里，(1, 1) 的根落了盘；(1, 2) 截在中间，单元没落、根槽写在截断处之后。
/// 崩溃之后另起实例 2，写出 (2, 2)，恢复（二次崩溃之后）落到它上。
/// - 第一次崩溃之后恢复落到 (1, 1)：(1, 2) 没发生过，不算 (2, 2) 该有的发布——它与挂载写的那一版同一个 txg，实例表读不出时
///   留着它就是假红；
/// - 第一次崩溃之后恢复落到由记录重建的 (1, 2)：它发生过，它的单元没落就判红；
/// - 根落了盘的 (1, 1) 照样核，恢复落到的不是它（落到由记录重建的 (1, 2)、(1, 2) 的单元都在）也核：它的单元没落就判红。
#[test]
fn publishes_cut_off_by_the_crash_are_on_the_resumed_timeline_only_if_their_root_landed_or_the_recovery_landed_on_them(
) {
    let writes = vec![
        hand_laid_unit(0x1_0000, 0xa1),
        hand_laid_root(0x3_0000, 1, 1),
        hand_laid_unit(0x1_1000, 0xa2),
        hand_laid_root(0x3_1000, 1, 2),
        hand_laid_unit(0x1_2000, 0xc1),
        hand_laid_root(0x3_2000, 2, 2),
    ];
    let first_write_after_the_crash = 4;
    let middle_publish_cut_off = vec![true, true, false, false, true, true];
    let image_with_the_middle_publish_cut_off = hand_laid_pool(&writes, &middle_publish_cut_off);
    let check_landing_after_the_first_crash_on =
        |landed: (InstanceGeneration, CheckpointTxg), persisted_set: &[bool], pool: &MemoryPool| {
            check_records_against(
                pool,
                pool,
                &writes,
                persisted_set,
                RecordStreamContinuity::ResumedAfterACrash {
                    first_write_after_the_crash,
                    version_landed_on_after_the_crash: Some(landed),
                },
                Some(root_identity(2, 2)),
            )
        };
    assert_eq!(
        check_landing_after_the_first_crash_on(
            root_identity(1, 1),
            &middle_publish_cut_off,
            &image_with_the_middle_publish_cut_off
        ),
        RecordCheck::default(),
        "恢复落到 (1, 1)：截在中间的 (1, 2) 没发生过，它没落的单元不算 (2, 2) 该有的"
    );
    assert!(
        check_landing_after_the_first_crash_on(
            root_identity(1, 2),
            &middle_publish_cut_off,
            &image_with_the_middle_publish_cut_off
        )
        .claimed_state_missing_unit,
        "恢复落到由记录重建的 (1, 2)：它在时间线上，它的单元没落照判红"
    );
    let first_unit_lost_second_unit_landed = vec![false, true, true, false, true, true];
    assert!(
        check_landing_after_the_first_crash_on(
            root_identity(1, 2),
            &first_unit_lost_second_unit_landed,
            &hand_laid_pool(&writes, &first_unit_lost_second_unit_landed)
        )
        .claimed_state_missing_unit,
        "根落了盘的 (1, 1) 不是恢复落到的那一版，也在时间线上，它的单元没落照判红"
    );
}

/// 截断的那条流末尾没等到根槽写的单元写（历史在一次发布中间停下），不归接缝之后的第一次发布：两条流接成一条来分，
/// 它就成了挂载写的 (2, 2) 的单元，(2, 2) 是恢复落到的那一版，它没落就判红（假红）。
#[test]
fn writes_left_without_a_root_at_the_end_of_the_cut_recording_do_not_join_the_first_publish_after_the_crash(
) {
    let writes = vec![
        hand_laid_unit(0x1_0000, 0xa1),
        hand_laid_root(0x3_0000, 1, 1),
        hand_laid_unit(0x1_1000, 0xa2),
        hand_laid_unit(0x1_2000, 0xc1),
        hand_laid_root(0x3_2000, 2, 2),
    ];
    let persisted = vec![true, true, false, true, true];
    let image = hand_laid_pool(&writes, &persisted);
    assert_eq!(
        check_records_against(
            &image,
            &image,
            &writes,
            &persisted,
            RecordStreamContinuity::ResumedAfterACrash {
                first_write_after_the_crash: 3,
                version_landed_on_after_the_crash: Some(root_identity(1, 1)),
            },
            Some(root_identity(2, 2)),
        ),
        RecordCheck::default(),
        "截断的那条流末尾没落的单元写不归 (2, 2)"
    );
}

fn overwrite(fill_seed: u64) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 2999 },
        fill_seed,
    })
}

/// 一段跑完的历史的录制流，切成写表与段；`candidate_segments` 是整段都落在 mkfs 之后的段（崩溃注入摆崩溃状态的同一条界）。
struct RecordedHistory {
    history: GeneratedHistory,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    candidate_segments: Vec<usize>,
}

fn record_history(history: GeneratedHistory) -> RecordedHistory {
    let stream = SharedStream::retaining_contents();
    let run = execute_history_with(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        &stream,
        &mut |_| {},
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "这段历史跑完");
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES
            .device_width
            .fixed_geometry(),
    );
    let candidate_segments = (0..segments.len())
        .filter(|segment_index| {
            segments[*segment_index]
                .iter()
                .all(|write| stream_indexes[*write] >= run.operations_written_by_make_filesystem)
        })
        .collect();
    RecordedHistory {
        history,
        writes,
        segments,
        candidate_segments,
    }
}

/// 一段写表里的每次发布：根身份、根槽写的下标、单元写（一次根槽写之前、上一次根槽写之后的单元写都归它）。
fn unit_writes_of_each_publish(
    writes: &[RetainedWrite],
) -> Vec<((InstanceGeneration, CheckpointTxg), usize, Vec<usize>)> {
    let mut publishes = Vec::new();
    let mut units = Vec::new();
    for (index, write) in writes.iter().enumerate() {
        match write.kind {
            StepKind::UnitWrite => units.push(index),
            StepKind::RootRecordFua => publishes.push((
                root_identity_written_by(write.bytes().expect("根槽写带着字节")),
                index,
                std::mem::take(&mut units),
            )),
            StepKind::JournalRecord
            | StepKind::ZeroFill
            | StepKind::SystemConfigurationSlot
            | StepKind::Barrier => {}
        }
    }
    publishes
}

/// 一个第一次崩溃状态（崩在 `crash_point` 那一段、段内按它落），与之后在那份崩溃后镜像上起的可写挂载加一次发布。
struct FirstCrashThenWritableMount {
    crash_point: CrashPoint,
    first_write_of_the_segment: usize,
    /// 与整条历史录制流逐条对应：更早的段整段持久、这一段按子集、更晚的段一个都没持久。
    persisted: Vec<bool>,
    /// 更早的段整段落了的池（崩溃镜像的基线）。
    base: MemoryPool,
    landed_version: Option<(InstanceGeneration, CheckpointTxg)>,
    mount_run: WritableMountOnTheCrashImage,
    mount_writes: Vec<RetainedWrite>,
    mount_segments: Vec<Vec<usize>>,
    /// 挂载之后那一次发布写出的写。
    publish_writes: Vec<RetainedWrite>,
    records: HistoryThenWritableMountRecords,
}

impl FirstCrashThenWritableMount {
    fn crash_image<'state>(&'state self, recorded: &'state RecordedHistory) -> CrashImage<'state> {
        CrashImage {
            base: &self.base,
            writes: &recorded.writes[self.first_write_of_the_segment
                ..self.first_write_of_the_segment
                    + self.crash_point.persisted_within_the_segment.len()],
            persisted: self.crash_point.persisted_within_the_segment.clone(),
        }
    }
}

fn crash_then_mount(
    recorded: &RecordedHistory,
    segment_index: usize,
    persisted_within_the_segment: Vec<bool>,
) -> FirstCrashThenWritableMount {
    let execution = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let segment = &recorded.segments[segment_index];
    assert_eq!(
        segment.len(),
        persisted_within_the_segment.len(),
        "段内每个写各一位"
    );
    let first_write_of_the_segment = segment[0];
    let mut persisted = vec![false; recorded.writes.len()];
    persisted[..first_write_of_the_segment].fill(true);
    persisted[first_write_of_the_segment..first_write_of_the_segment + segment.len()]
        .copy_from_slice(&persisted_within_the_segment);
    let mut base = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    base.apply_writes(&recorded.writes[..first_write_of_the_segment]);
    let crash_point = CrashPoint {
        segment_index,
        persisted_within_the_segment,
        step: StepPosition::StartingPoint,
        operation_kind: None,
    };
    let image = CrashImage {
        base: &base,
        writes: &recorded.writes
            [first_write_of_the_segment..first_write_of_the_segment + segment.len()],
        persisted: crash_point.persisted_within_the_segment.clone(),
    };
    let landed_version = recover(&image, JournalPolicy::Consult).effective_root;
    let mount_run = mount_and_publish_once_on_the_crash_image(
        &materialized_crash_image(&image),
        &execution.device_width.parameters(),
        execution,
        recorded.history.seed,
        &crash_point,
    );
    let geometry = execution.device_width.fixed_geometry();
    let (mount_writes, mount_segments) =
        writes_and_segments(&mount_run.mount_operations, &geometry);
    let (publish_writes, _publish_segments) =
        writes_and_segments(&mount_run.publish_operations, &geometry);
    let records = HistoryThenWritableMountRecords::new(
        &recorded.writes,
        &persisted,
        landed_version,
        &mount_writes,
        &publish_writes,
    );
    FirstCrashThenWritableMount {
        crash_point,
        first_write_of_the_segment,
        persisted,
        base,
        landed_version,
        mount_run,
        mount_writes,
        mount_segments,
        publish_writes,
        records,
    }
}

/// 挂载途中的一个二次崩溃：崩在挂载那一段的第 `segment_index` 段，段内按 `persisted_within_the_segment` 落。
struct SecondCrash {
    /// 与整条挂载流逐条对应：更早的段整段持久、这一段按子集、更晚的段一个都没持久。
    persisted_in_the_mount: Vec<bool>,
    /// 第一次崩溃之后的池加上挂载更早的段（二次崩溃镜像的基线）。
    base: MemoryPool,
    first_write_of_the_segment: usize,
    persisted_within_the_segment: Vec<bool>,
}

impl SecondCrash {
    fn image<'state>(
        &'state self,
        state: &'state FirstCrashThenWritableMount,
    ) -> CrashImage<'state> {
        self.image_on(&self.base, state)
    }

    fn image_on<'state>(
        &'state self,
        base: &'state MemoryPool,
        state: &'state FirstCrashThenWritableMount,
    ) -> CrashImage<'state> {
        CrashImage {
            base,
            writes: &state.mount_writes[self.first_write_of_the_segment
                ..self.first_write_of_the_segment + self.persisted_within_the_segment.len()],
            persisted: self.persisted_within_the_segment.clone(),
        }
    }
}

fn second_crash(
    recorded: &RecordedHistory,
    state: &FirstCrashThenWritableMount,
    segment_index: usize,
    persisted_within_the_segment: Vec<bool>,
) -> SecondCrash {
    let segment = &state.mount_segments[segment_index];
    assert_eq!(
        segment.len(),
        persisted_within_the_segment.len(),
        "段内每个写各一位"
    );
    let first_write_of_the_segment = segment[0];
    let mut persisted_in_the_mount = vec![false; state.mount_writes.len()];
    persisted_in_the_mount[..first_write_of_the_segment].fill(true);
    persisted_in_the_mount[first_write_of_the_segment..first_write_of_the_segment + segment.len()]
        .copy_from_slice(&persisted_within_the_segment);
    let mut base = materialized_crash_image(&state.crash_image(recorded));
    base.apply_writes(&state.mount_writes[..first_write_of_the_segment]);
    SecondCrash {
        persisted_in_the_mount,
        base,
        first_write_of_the_segment,
        persisted_within_the_segment,
    }
}

/// 段内两种摆法：只扣最后一个写，与只落第一个写（段只有一个写时两种都是「一个都没落」）。
fn two_subsets_of_a_segment(segment_length: usize) -> [Vec<bool>; 2] {
    [
        (0..segment_length)
            .map(|position| position + 1 < segment_length)
            .collect(),
        (0..segment_length)
            .map(|position| position == 0 && segment_length > 1)
            .collect(),
    ]
}

/// 这次写写下的字节还整份在不在 `reader` 上。
fn still_on_disk(reader: &dyn PoolReader, write: &RetainedWrite) -> bool {
    let length = usize::try_from(write.length_in_bytes()).expect("写长装得进 usize");
    reader
        .read(write.device, write.offset, length)
        .is_some_and(|bytes| write.contents.still_on_disk(&bytes))
}

/// 池级 checker 在这个镜像上一条都不判红。
fn checker_is_silent_on(image: &dyn singlefs_checker::image::ImageReader) -> bool {
    check_pool_image(image)
        .into_iter()
        .all(|(_, verdict)| match verdict {
            InvariantVerdict::Violated(_) => false,
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => true,
        })
}

/// 一次历史发布写在一个偏移上的单元的全部副本（写表下标）：按偏移分组。
fn copies_by_offset(writes: &[RetainedWrite], units: &[usize]) -> BTreeMap<u64, Vec<usize>> {
    let mut copies: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
    for unit in units {
        copies
            .entry(writes[*unit].offset.0)
            .or_default()
            .push(*unit);
    }
    copies
}

/// 在 `pool` 的副本上把这几份副本各翻一个字节（第一个扇区的第 0 个字节）。
fn with_copies_damaged(
    pool: &MemoryPool,
    writes: &[RetainedWrite],
    copies: &[usize],
) -> MemoryPool {
    let mut damaged = pool.clone();
    for copy in copies {
        damaged.flip_byte(writes[*copy].device, writes[*copy].offset, 0);
    }
    damaged
}

/// 只覆盖写的一段历史：起点（第一个文件）之后覆盖写 [`OVERWRITES_IN_THE_HISTORY`] 次，一个实例到底、不回退——第一次崩溃之后
/// 恢复落到的那一版以前的发布都在挂载写的那一版的时间线上（挂载写行 (1, 落到的 txg)，只抛弃比它新的）。
/// 次数要多过根环的槽数（这套几何 24 槽，池级 checker 的机理行里打着）：池级 checker 走遍根环里每一条有效根，早先那几次发布的根
/// 被挤出根环之后，它们写出而后来被换下、又没被复用的单元它不再走到，记录核对器照样要求它们在。
const OVERWRITES_IN_THE_HISTORY: u64 = 40;

/// 弄坏几处历史发布的单元（从最早的发布起）：每一处都要重跑一遍恢复（二次崩溃镜像上）或记录核对器（挂载之后的池上），
/// 几百处全弄一遍太慢；判的是弄过的每一处都红。挂载之后的池上另要找到一处池级 checker 不判的，找到之前不停。
const HISTORY_UNIT_DAMAGES_TRIED: usize = 12;

fn history_of_overwrites_only() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: (1..=OVERWRITES_IN_THE_HISTORY).map(overwrite).collect(),
    }
}

/// 第一次崩溃摆在最后那次覆盖写的根槽写所在的段上、扣下那次根槽写：恢复落到它前一版，之后的可写挂载做成、发成一次布。
fn crash_before_the_last_root_then_mount(
    recorded: &RecordedHistory,
) -> FirstCrashThenWritableMount {
    let last_root = recorded
        .writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("历史写过根槽");
    let segment_index = recorded
        .segments
        .iter()
        .position(|segment| segment.contains(&last_root))
        .expect("每个写都在某一段里");
    let persisted_within_the_segment = recorded.segments[segment_index]
        .iter()
        .map(|write| *write != last_root)
        .collect();
    let state = crash_then_mount(recorded, segment_index, persisted_within_the_segment);
    assert!(
        matches!(state.mount_run.outcome, WritableMountOutcome::Published),
        "崩溃之后的可写挂载与之后那一次发布都做成：{:?}",
        state.mount_run.outcome
    );
    state
}

/// 第一次崩溃之后恢复落到的那一版以前、根槽写落了盘的历史发布写的单元，按偏移：(发布的根身份, 偏移, 那个偏移上的全部副本)。
fn units_of_the_history_publishes_the_landed_version_keeps(
    recorded: &RecordedHistory,
    state: &FirstCrashThenWritableMount,
) -> Vec<((InstanceGeneration, CheckpointTxg), u64, Vec<usize>)> {
    let (landed_instance, landed_txg) = state.landed_version.expect("第一次崩溃之后恢复落到了一版");
    unit_writes_of_each_publish(&recorded.writes)
        .into_iter()
        .filter(|((instance, checkpoint_txg), root, _)| {
            *instance == landed_instance && *checkpoint_txg <= landed_txg && state.persisted[*root]
        })
        .flat_map(|(identity, _, units)| {
            copies_by_offset(&recorded.writes, &units)
                .into_iter()
                .map(move |(offset, copies)| (identity, offset, copies))
        })
        .collect()
}

/// 第 1 件（B3c-1 报告第六节第 2 条）：二次崩溃镜像上，一次历史发布（在挂载写的那一版的时间线上）写在某个偏移上的单元每一份都坏了，
/// 第三截要判红。二次崩溃摆在挂载最后一段、扣下最后一个写，恢复落到挂载写出的一版；只算干净的弄坏——弄坏之后恢复仍落到同一版。
/// 干净的弄坏至少一个，每个都判红；不弄坏时一条不判（对照）。只交挂载那一段时历史发布不在表里，这些一个都判不出（打出来给人看）。
#[test]
fn history_publishes_missing_units_on_the_second_crash_image_are_red_on_the_whole_stream() {
    let recorded = record_history(history_of_overwrites_only());
    let state = crash_before_the_last_root_then_mount(&recorded);
    let last_mount_segment = state.mount_segments.len() - 1;
    let second = second_crash(
        &recorded,
        &state,
        last_mount_segment,
        two_subsets_of_a_segment(state.mount_segments[last_mount_segment].len())[0].clone(),
    );
    let image = second.image(&state);
    let landed = recover(&image, JournalPolicy::Consult).effective_root;
    let (landed_instance, _) = landed.expect("二次崩溃之后恢复落到了一版");
    assert!(
        state
            .mount_writes
            .iter()
            .filter(|write| write.kind == StepKind::RootRecordFua)
            .any(
                |write| root_identity_written_by(write.bytes().expect("根槽写带着字节")).0
                    == landed_instance
            ),
        "二次崩溃之后恢复落到挂载写出的一版：{landed:?}"
    );
    assert_eq!(
        state.records.record_check_after_a_second_crash(
            &second.persisted_in_the_mount,
            &image,
            landed
        ),
        RecordCheck::default(),
        "对照：不弄坏时一条不判"
    );
    let mut clean_damages = 0usize;
    let mut mount_only_reports = 0usize;
    for (publish, offset, copies) in
        units_of_the_history_publishes_the_landed_version_keeps(&recorded, &state)
            .into_iter()
            .filter(|(_, _, copies)| {
                copies
                    .iter()
                    .all(|copy| still_on_disk(&image, &recorded.writes[*copy]))
            })
            .take(HISTORY_UNIT_DAMAGES_TRIED)
    {
        let damaged_base = with_copies_damaged(&second.base, &recorded.writes, &copies);
        let damaged_image = second.image_on(&damaged_base, &state);
        if recover(&damaged_image, JournalPolicy::Consult).effective_root != landed {
            continue;
        }
        clean_damages += 1;
        if check_records_against(
            &damaged_image,
            &damaged_image,
            &state.mount_writes,
            &second.persisted_in_the_mount,
            RecordStreamContinuity::OneRecording,
            landed,
        ) != RecordCheck::default()
        {
            mount_only_reports += 1;
        }
        assert!(
            state
                .records
                .record_check_after_a_second_crash(
                    &second.persisted_in_the_mount,
                    &damaged_image,
                    landed
                )
                .claimed_state_missing_unit,
            "历史发布 {publish:?} 写在偏移 {offset} 上的单元每一份（写表第 {copies:?} 项）在二次崩溃镜像上都坏了，恢复落到 {landed:?}，第三截照判红"
        );
    }
    println!(
        "── 二次崩溃落到 {landed:?}：干净地弄坏历史发布的单元 {clean_damages} 次，都判红；只交挂载那一段时判得出的 {mount_only_reports} 次 ──"
    );
    assert!(
        clean_damages >= 1,
        "一个干净的弄坏都没有（弄坏哪一个恢复都换了一版）：这一形没验到"
    );
}

/// 第 2 件（B3b 报告第二节第 5 条）：挂载与之后那一次发布之后的池（实现恢复后的镜像）上丢了单元，第二截要判红。
/// - 一次历史发布（在挂载写的那一版的时间线上）写在某个偏移上的单元每一份都坏了：判红；从最早的发布起找，至少有一处池级 checker
///   一条不判（那次发布的根已被挤出根环、单元已被换下）——第二截只跑 checker 时这一处报不出来（池级 checker 只跑到找到第一处为止）；
/// - 挂载之后那一次发布写的单元每一份都坏了：判红（恢复自称的那一版取挂载与之后那次发布写出的最新那条根，挂载之后写的发布也在里面）。
///
/// 不弄坏时一条不判（对照）。
#[test]
fn units_missing_on_the_pool_after_the_writable_mount_are_red_on_the_second_stage() {
    let recorded = record_history(history_of_overwrites_only());
    let state = crash_before_the_last_root_then_mount(&recorded);
    let crash_image = state.crash_image(&recorded);
    let pool_after = &state.mount_run.pool_after;
    assert_eq!(
        state
            .records
            .record_check_after_the_writable_mount(&crash_image, pool_after),
        RecordCheck::default(),
        "对照：挂载之后的池上不弄坏时一条不判"
    );
    let mut history_damages = 0usize;
    let mut history_damages_the_checker_misses = 0usize;
    for (publish, offset, copies) in
        units_of_the_history_publishes_the_landed_version_keeps(&recorded, &state)
    {
        if history_damages >= HISTORY_UNIT_DAMAGES_TRIED && history_damages_the_checker_misses == 1
        {
            break;
        }
        if !copies
            .iter()
            .all(|copy| still_on_disk(pool_after, &recorded.writes[*copy]))
        {
            continue;
        }
        let damaged_pool = with_copies_damaged(pool_after, &recorded.writes, &copies);
        history_damages += 1;
        if history_damages_the_checker_misses == 0 && checker_is_silent_on(&damaged_pool) {
            history_damages_the_checker_misses += 1;
            println!(
                "── 挂载之后的池：弄坏历史发布 {publish:?} 写在偏移 {offset} 上的单元，池级 checker 一条不判 ──"
            );
        }
        assert!(
            state
                .records
                .record_check_after_the_writable_mount(&crash_image, &damaged_pool)
                .claimed_state_missing_unit,
            "历史发布 {publish:?} 写在偏移 {offset} 上的单元每一份（写表第 {copies:?} 项）在挂载之后的池上都坏了，第二截照判红"
        );
    }
    let (_, publish_after_the_mount, units_of_the_publish_after_the_mount) =
        unit_writes_of_each_publish(&state.publish_writes)
            .pop()
            .expect("挂载之后发成了一次布");
    let mut damages_of_the_publish_after_the_mount = 0usize;
    for (offset, copies) in
        copies_by_offset(&state.publish_writes, &units_of_the_publish_after_the_mount)
    {
        let damaged_pool = with_copies_damaged(pool_after, &state.publish_writes, &copies);
        damages_of_the_publish_after_the_mount += 1;
        assert!(
            state
                .records
                .record_check_after_the_writable_mount(&crash_image, &damaged_pool)
                .claimed_state_missing_unit,
            "挂载之后那一次发布（根槽写在它自己那段写表第 {publish_after_the_mount} 项）写在偏移 {offset} 上的单元每一份都坏了，第二截照判红"
        );
    }
    println!(
        "── 挂载之后的池：弄坏历史发布的单元 {history_damages} 次、挂载之后那一次发布的单元 {damages_of_the_publish_after_the_mount} 次，都判红 ──"
    );
    assert!(history_damages >= 1, "历史发布的单元一个都没弄到");
    assert_eq!(
        history_damages_the_checker_misses, 1,
        "弄坏的历史单元池级 checker 每一个都判得出：这一形没有说明第二截接记录核对器多罩了什么"
    );
    assert!(
        damages_of_the_publish_after_the_mount >= 1,
        "挂载之后那一次发布一个单元都没写"
    );
}

/// 对照（B3b 报告第二节第 4 条那一形）：第一次崩溃截在一次发布中间（它写过东西、根还没落盘），恢复没落到它上；历史之后关掉重开，
/// 重开的实例号与崩溃之后的挂载取的相同、txg 也接在同一处。这段历史上每一个这样的崩溃状态（每个候选段两种摆法）之后：
/// 挂载之后的池、挂载每一段两种摆法的二次崩溃，交整条流、在接缝处断开，一条都不判。
/// 两种接成一条来分的交法在同一批状态上各有判红的（造的状态真在陷阱上）：接到截断处为止，截在中间那次发布没落的写归进挂载写行那次发布；
/// 整条接上，截断处之后历史重开写的发布与挂载写的撞了身份。
#[test]
fn first_crashes_cut_inside_publishes_leave_every_second_crash_and_the_pool_after_the_mount_clean()
{
    let recorded = record_history(GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            overwrite(1),
            overwrite(2),
            HistoryOperation::CloseAndMountWritable,
            overwrite(3),
            overwrite(4),
        ],
    });
    let mut first_crashes_inside_a_publish = 0usize;
    let mut judged = 0usize;
    let mut cut_at_the_crash_as_one_recording_red = 0usize;
    let mut whole_history_as_one_recording_red = 0usize;
    for segment_index in &recorded.candidate_segments {
        for persisted_within_the_segment in
            two_subsets_of_a_segment(recorded.segments[*segment_index].len())
        {
            let state = crash_then_mount(&recorded, *segment_index, persisted_within_the_segment);
            if !some_publish_persisted_without_its_root(&recorded.writes, &state.persisted)
                || state.landed_version.is_none()
            {
                continue;
            }
            first_crashes_inside_a_publish += 1;
            let crash_image = state.crash_image(&recorded);
            assert_eq!(
                state.records.record_check_after_the_writable_mount(
                    &crash_image,
                    &state.mount_run.pool_after
                ),
                RecordCheck::default(),
                "第一次崩溃在第 {segment_index} 段 {:?}、落到 {:?}：挂载之后的池上一条不判",
                state.crash_point.persisted_within_the_segment,
                state.landed_version
            );
            judged += 1;
            let end_of_the_crash_segment = state.first_write_of_the_segment
                + state.crash_point.persisted_within_the_segment.len();
            for (mount_segment_index, mount_segment) in state.mount_segments.iter().enumerate() {
                for persisted_within_the_mount_segment in
                    two_subsets_of_a_segment(mount_segment.len())
                {
                    let second = second_crash(
                        &recorded,
                        &state,
                        mount_segment_index,
                        persisted_within_the_mount_segment,
                    );
                    let image = second.image(&state);
                    let landed = recover(&image, JournalPolicy::Consult).effective_root;
                    assert_eq!(
                        state.records.record_check_after_a_second_crash(
                            &second.persisted_in_the_mount,
                            &image,
                            landed
                        ),
                        RecordCheck::default(),
                        "第一次崩溃在第 {segment_index} 段 {:?}（落到 {:?}），二次崩溃在挂载第 {mount_segment_index} 段 {:?}（落到 {landed:?}）：一条不判",
                        state.crash_point.persisted_within_the_segment,
                        state.landed_version,
                        second.persisted_within_the_segment
                    );
                    judged += 1;
                    let as_one_recording =
                        |history_writes: &[RetainedWrite],
                         history_persisted: &[bool],
                         mount_persisted: &[bool],
                         mount_writes: &[RetainedWrite]| {
                            let writes: Vec<RetainedWrite> =
                                history_writes.iter().chain(mount_writes).cloned().collect();
                            let persisted: Vec<bool> = history_persisted
                                .iter()
                                .chain(mount_persisted)
                                .copied()
                                .collect();
                            check_records_against(
                                &image,
                                &image,
                                &writes,
                                &persisted,
                                RecordStreamContinuity::OneRecording,
                                landed,
                            )
                        };
                    let end_of_the_mount_segment = second.first_write_of_the_segment
                        + second.persisted_within_the_segment.len();
                    if as_one_recording(
                        &recorded.writes[..end_of_the_crash_segment],
                        &state.persisted[..end_of_the_crash_segment],
                        &second.persisted_in_the_mount[..end_of_the_mount_segment],
                        &state.mount_writes[..end_of_the_mount_segment],
                    ) != RecordCheck::default()
                    {
                        cut_at_the_crash_as_one_recording_red += 1;
                    }
                    if as_one_recording(
                        &recorded.writes,
                        &state.persisted,
                        &second.persisted_in_the_mount,
                        &state.mount_writes,
                    ) != RecordCheck::default()
                    {
                        whole_history_as_one_recording_red += 1;
                    }
                }
            }
        }
    }
    println!(
        "── 第一次崩溃截在发布中间的状态 {first_crashes_inside_a_publish} 个，判了 {judged} 次都不红；接到截断处为止接成一条判红 {cut_at_the_crash_as_one_recording_red} 次，整条接成一条判红 {whole_history_as_one_recording_red} 次 ──"
    );
    assert!(
        first_crashes_inside_a_publish >= 1,
        "一个第一次崩溃截在发布中间的状态都没摆到"
    );
    assert!(
        cut_at_the_crash_as_one_recording_red >= 1,
        "接到截断处为止接成一条的交法在这批状态上一次都不红：造的状态不在 B3b 报告第二节第 4 条那个陷阱上"
    );
    assert!(
        whole_history_as_one_recording_red >= 1,
        "整条接成一条的交法在这批状态上一次都不红：造的状态没让截断处之后的历史发布与挂载的撞上"
    );
}

/// 对照：落到的那一版抛弃了一次单元已被后来的发布复用盖掉的发布（崩溃恢复抛弃最新那条根，之后又覆盖写几次，实七报告 (c) 那一形）。
/// 第一次崩溃摆在最后那次覆盖写的根槽写上、扣下它，挂载之后的池上恢复自称的是挂载写出的最新那一版，它的实例表只在挂载之后的池上：
/// 第二截从那里读它、豁免被抛弃的那次发布，一条不判。从崩溃态镜像上读，读不出那张表、豁免落空，判红（在同一个状态上现算给人看）。
#[test]
fn the_pool_after_the_mount_exempts_what_the_newest_mount_version_abandons_by_its_instance_table_on_the_recovered_image(
) {
    let recorded = record_history(GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            overwrite(1),
            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,
            overwrite(2),
            overwrite(3),
            overwrite(4),
        ],
    });
    let state = crash_before_the_last_root_then_mount(&recorded);
    let crash_image = state.crash_image(&recorded);
    assert_eq!(
        state
            .records
            .record_check_after_the_writable_mount(&crash_image, &state.mount_run.pool_after),
        RecordCheck::default(),
        "挂载写出的最新那一版的实例表抛弃 (1, 4)，它被复用盖掉的单元不算该在"
    );
    let newest_mount_root = state
        .mount_writes
        .iter()
        .chain(&state.publish_writes)
        .rev()
        .find(|write| write.kind == StepKind::RootRecordFua)
        .map(|write| root_identity_written_by(write.bytes().expect("根槽写带着字节")));
    let writes: Vec<RetainedWrite> = recorded
        .writes
        .iter()
        .chain(&state.mount_writes)
        .chain(&state.publish_writes)
        .cloned()
        .collect();
    let persisted: Vec<bool> = state
        .persisted
        .iter()
        .copied()
        .chain(std::iter::repeat_n(
            true,
            state.mount_writes.len() + state.publish_writes.len(),
        ))
        .collect();
    let instance_table_read_from_the_crash_state_image = check_records_against(
        &crash_image,
        &crash_image,
        &writes,
        &persisted,
        RecordStreamContinuity::ResumedAfterACrash {
            first_write_after_the_crash: recorded.writes.len(),
            version_landed_on_after_the_crash: state.landed_version,
        },
        newest_mount_root,
    );
    println!(
        "── 挂载之后恢复自称 {newest_mount_root:?}：实例表从崩溃态镜像上读时 {instance_table_read_from_the_crash_state_image:?} ──"
    );
    assert!(
        instance_table_read_from_the_crash_state_image.claimed_state_missing_unit,
        "实例表从崩溃态镜像上读不出来、豁免落空时判红：这一形真靠恢复后的镜像上那张表"
    );
}

/// 崩溃注入整段跑下来（两段短历史、每段两个崩溃状态）：每个崩溃状态之后挂载之后的池上跑过记录核对器、每个二次崩溃上跑过，
/// 一条新发现都没有（整条流、接缝处断开，在真实历史上不假红）。
#[test]
fn every_crash_state_runs_the_record_checker_after_the_writable_mount_and_on_every_second_crash_without_new_findings(
) {
    let mut tally = CrashInjectionTally::default();
    let mut new_findings = Vec::new();
    for seed in [3_u64, 11] {
        let injection = inject_crashes_into_history(
            &generate_history(HistorySeed(seed), 8),
            UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            CrashPointDraw::Sampled {
                crash_points_per_history: 2,
            },
            None,
        );
        tally.absorb(&injection.tally);
        new_findings.extend(injection.new_findings);
    }
    let rendered = tally.render();
    println!("{rendered}");
    assert!(
        tally.crash_points >= 2,
        "两段历史里摆出了崩溃状态：{rendered}"
    );
    assert_eq!(
        tally.record_checks_after_the_writable_mount, tally.crash_points,
        "每个崩溃状态之后挂载之后的池上都跑一次记录核对器：{rendered}"
    );
    assert!(
        tally.second_crash_points >= 1,
        "挂载途中一个二次崩溃都没摆出来：{rendered}"
    );
    assert_eq!(
        tally.second_crash_record_checks, tally.second_crash_points,
        "每个二次崩溃上都跑一次记录核对器：{rendered}"
    );
    assert!(
        new_findings.is_empty(),
        "新发现：{}",
        new_findings
            .iter()
            .map(singlefs_harness::crash_injection::CrashPointFinding::render)
            .collect::<String>()
    );
}
