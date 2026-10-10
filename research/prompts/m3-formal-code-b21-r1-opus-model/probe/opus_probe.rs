//! 攻方副本探针（m3-formal-code-b21-r1 opus）：只在副本里，不进仓。挂在 crash_injection 下面当子模块，用它的私有项。
use super::*;
use singlefs_harness::history::{
    ContentChoice, ContentLength, HistoryDeviceWidth, HistoryStartingPoint, PerStepChecker,
};

fn segment_containing(segments: &[Vec<usize>], write: usize) -> usize {
    segments
        .iter()
        .position(|segment| segment.contains(&write))
        .expect("写在某一段里")
}

fn kinds(writes: &[RetainedWrite], range: std::ops::Range<usize>) -> String {
    range
        .map(|index| format!("{index}:{}", writes[index].kind.name()))
        .collect::<Vec<_>>()
        .join(" ")
}

/// 与 tests 里 `history_of_the_first_file_and_one_overwrite` 同一段历史。
#[must_use]
pub fn history_ff_ow() -> (GeneratedHistory, HistoryExecution) {
    (
        GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: vec![HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 0 },
                fill_seed: 1,
            })],
        },
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
    )
}

fn writes_of(history: &GeneratedHistory, execution: HistoryExecution, mark_after: StepPosition) -> (Vec<RetainedWrite>, Vec<Vec<usize>>, Vec<usize>, usize) {
    let stream = SharedStream::retaining_contents();
    let mut operations_after = None;
    let run = execute_history_with(history, execution, &stream, &mut |observation| {
        if observation.position == mark_after {
            operations_after = Some(stream.operation_count());
        }
    });
    assert_eq!(run.ending, HistoryEnding::Completed, "前提：这段历史跑完");
    let operations_after = operations_after.expect("登记过");
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let first_after = stream_indexes
        .iter()
        .position(|stream_index| *stream_index >= operations_after)
        .expect("之后写过东西");
    (writes, segments, stream_indexes, first_after)
}

/// K3：第一次崩溃怎么摆（`control`：与入库靶向用例同一个；`ff-record-lost`：第一个文件那次发布的记录扣下、它的根与轮换落了、
/// 覆盖写的单元记录根都落了、覆盖写的轮换没落），二次崩溃怎么摆（`with-rotation`：与入库用例同；`no-rotation`：写行的记录扣下、根落了、轮换没落）。
#[must_use]
pub fn k3_run(first_case: &str, second_case: &str) -> String {
    let mut out = String::new();
    let (history, execution) = history_ff_ow();
    let (writes, segments, _stream_indexes, first_ow) =
        writes_of(&history, execution, StepPosition::StartingPoint);
    let ow_root = (first_ow..writes.len())
        .find(|w| writes[*w].kind == StepKind::RootRecordFua)
        .expect("覆盖写的根");
    let ff_root = (0..first_ow)
        .rev()
        .find(|w| writes[*w].kind == StepKind::RootRecordFua)
        .expect("第一个文件的根");
    let previous_root = (0..ff_root)
        .rev()
        .find(|w| writes[*w].kind == StepKind::RootRecordFua)
        .expect("第一个文件之前那次的根");
    let ff_records: Vec<usize> = (previous_root + 1..ff_root)
        .filter(|w| writes[*w].kind == StepKind::JournalRecord)
        .collect();
    let _ = writeln!(
        out,
        "history writes={} first_ow={first_ow} ow_root={ow_root} ff_root={ff_root} previous_root={previous_root} ff_records={ff_records:?}",
        writes.len()
    );
    let _ = writeln!(out, "kinds[{}..{}]: {}", previous_root, (ow_root + 3).min(writes.len()), kinds(&writes, previous_root..(ow_root + 3).min(writes.len())));
    let (first_segment, first_count, first_withheld): (usize, usize, Vec<usize>) = match first_case {
        "control" => {
            let segment = segments
                .iter()
                .position(|segment| segment[0] == first_ow)
                .expect("覆盖写从一段的第一个写起");
            (segment, 1, vec![0])
        }
        "ff-record-lost" => {
            let segment = segment_containing(&segments, ff_records[0]);
            let first = segments[segment][0];
            (
                segment,
                ow_root + 1 - first,
                ff_records.iter().map(|w| w - first).collect(),
            )
        }
        other => panic!("不认的第一次崩溃 {other}"),
    };
    let first_write_of_the_first_segment = segments[first_segment][0];
    let mut first_crash_image = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    first_crash_image.apply_writes(&writes[..first_write_of_the_first_segment]);
    let persisted_first: Vec<RetainedWrite> = (0..first_count)
        .filter(|k| !first_withheld.contains(k))
        .map(|k| writes[first_write_of_the_first_segment + k].clone())
        .collect();
    first_crash_image.apply_writes(&persisted_first);
    let _ = writeln!(
        out,
        "first crash: segment={first_segment} first_write={first_write_of_the_first_segment} count={first_count} withheld={first_withheld:?}"
    );
    let crash_point = CrashPoint {
        segment_index: first_segment,
        persisted_within_the_segment: (0..first_count).map(|k| !first_withheld.contains(&k)).collect(),
        step: StepPosition::Operation(0),
        operation_kind: Some(HistoryOperationKind::PublishOverwrite),
    };
    let mount_run = mount_and_publish_once_on_the_crash_image(
        &first_crash_image,
        &execution.device_width.parameters(),
        execution,
        history.seed,
        &crash_point,
    );
    let _ = writeln!(out, "first-crash mount outcome: {:?}", mount_run.outcome);
    let (mount_writes, mount_segments, _) = writes_and_segments_with_stream_indexes(
        &mount_run.mount_operations,
        &execution.device_width.fixed_geometry(),
    );
    let phase_of_each_write = writable_mount_phase_of_each_write(
        &mount_writes,
        mount_run.publishes_before_the_deletion_queue_continuation,
        mount_run.publishes_before_the_instance_table_sweep,
    );
    let root_of_the_row_publish = (0..mount_writes.len())
        .find(|w| {
            phase_of_each_write[*w] == WritableMountPhase::RowPublish
                && mount_writes[*w].kind == StepKind::RootRecordFua
        })
        .expect("写行的根");
    let records_of_the_row_publish: Vec<usize> = (0..root_of_the_row_publish)
        .filter(|w| {
            phase_of_each_write[*w] == WritableMountPhase::RowPublish
                && mount_writes[*w].kind == StepKind::JournalRecord
        })
        .collect();
    let rotation_after_the_root: Vec<usize> = (root_of_the_row_publish + 1..mount_writes.len())
        .take_while(|w| mount_writes[*w].kind == StepKind::SystemConfigurationSlot)
        .collect();
    let _ = writeln!(
        out,
        "mount writes={} kinds: {}",
        mount_writes.len(),
        kinds(&mount_writes, 0..mount_writes.len().min(root_of_the_row_publish + 4))
    );
    let second_segment = segment_containing(&mount_segments, records_of_the_row_publish[0]);
    let first_write_of_the_second_segment = mount_segments[second_segment][0];
    let last_persisted = match second_case {
        "with-rotation" => rotation_after_the_root[rotation_after_the_root.len() - 1],
        "no-rotation" => root_of_the_row_publish,
        other => panic!("不认的二次崩溃 {other}"),
    };
    let second_count = last_persisted + 1 - first_write_of_the_second_segment;
    let second_withheld: Vec<usize> = records_of_the_row_publish
        .iter()
        .map(|record| record - first_write_of_the_second_segment)
        .collect();
    let mut second_crash_image = first_crash_image.clone();
    second_crash_image.apply_writes(&mount_writes[..first_write_of_the_second_segment]);
    let persisted_second: Vec<RetainedWrite> = (first_write_of_the_second_segment..=last_persisted)
        .filter(|w| !records_of_the_row_publish.contains(w))
        .map(|w| mount_writes[w].clone())
        .collect();
    second_crash_image.apply_writes(&persisted_second);
    let version_after_the_first_crash =
        recover(&first_crash_image, JournalPolicy::Consult).effective_root;
    let version_after_the_second_crash =
        recover(&second_crash_image, JournalPolicy::Consult).effective_root;
    let _ = writeln!(
        out,
        "second crash: segment={second_segment} first_write={first_write_of_the_second_segment} count={second_count} withheld={second_withheld:?}; Vo={version_after_the_first_crash:?} Vn={version_after_the_second_crash:?}"
    );
    for wiring in ["second", "first"] {
        std::env::set_var("OPUS_K3_WIRING", wiring);
        let injection = inject_crashes_into_history_drawing_second_crashes(
            &history,
            execution,
            CrashPointDraw::ExactlyOneCrashState {
                segment_index: first_segment,
                writes_from_the_segment_on: first_count,
                withheld_among_them: first_withheld.clone().leak(),
            },
            SecondCrashPointDraw::ExactlyOneSecondCrashState {
                segment_index: second_segment,
                writes_from_the_segment_on: second_count,
                withheld_among_them: second_withheld.clone().leak(),
            },
            None,
        );
        let tally = &injection.tally;
        let remount_findings: Vec<String> = injection
            .new_findings
            .iter()
            .filter(|finding| {
                matches!(
                    finding.stage,
                    CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(_)
                )
            })
            .map(|finding| {
                let answer = finding
                    .observation
                    .model_disagreement
                    .as_ref()
                    .map_or_else(|| finding.render(), |d| d.implementation_answer.clone());
                let branch = ["WitnessedCounterAboveTheLastRecordOfTheSelectedVersion",
                    "RecordAtTheWitnessedCounterNewerThanTheSelectedVersion",
                    "NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable",
                    "CriterionFalseWhenRecomputedOnTheCrashImage"]
                    .iter()
                    .filter(|name| answer.contains(*name))
                    .copied()
                    .collect::<Vec<_>>()
                    .join("+");
                branch
            })
            .collect();
        let other_stages: Vec<String> = injection
            .new_findings
            .iter()
            .filter(|finding| {
                !matches!(
                    finding.stage,
                    CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(_)
                )
            })
            .map(|finding| format!("{:?}", finding.stage))
            .collect();
        let _ = writeln!(
            out,
            "wiring={wiring}: (crash_points, mounts_ok, read_fault_states)=({}, {}, {}) second_crash_points={} (judged, illegal)=({}, {}) remounts_after_second={} remount_findings_branches={remount_findings:?} other_stage_findings={other_stages:?}",
            tally.crash_points,
            tally.writable_mounts_after_the_crash_succeeded,
            tally.crash_points_after_a_step_reading_under_injected_faults,
            tally.second_crash_points,
            tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
            tally.remount_refusals_with_the_witness_criterion_true_without_read_faults,
            tally.remounts_after_a_second_crash,
        );
    }
    std::env::remove_var("OPUS_K3_WIRING");
    out
}

/// K2 改法 a：入库靶向用例那一段历史与崩溃态，第二截那一支按 `OPUS_K2A_VARIANT` 换成坏发布，量改法 a 的断言、第二截新发现与池级 checker。
#[must_use]
pub fn k2a_run(variant: &str) -> String {
    let mut out = String::new();
    if variant == "correct" {
        std::env::remove_var("OPUS_K2A_VARIANT");
    } else {
        std::env::set_var("OPUS_K2A_VARIANT", variant);
    }
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![HistoryOperation::Unlink, HistoryOperation::CreateInode],
    };
    let (writes, segments, _stream_indexes, first_after_unlink) =
        writes_of(&history, execution, StepPosition::Operation(0));
    let root_of_the_inode_creation = (first_after_unlink..writes.len())
        .rev()
        .find(|w| writes[*w].kind == StepKind::RootRecordFua)
        .expect("建 inode 的根");
    let segment_of_the_root = segment_containing(&segments, root_of_the_inode_creation);
    let first_write_of_the_segment = segments[segment_of_the_root][0];
    let injection = inject_crashes_into_history(
        &history,
        execution,
        CrashPointDraw::ExactlyOneCrashState {
            segment_index: segment_of_the_root,
            writes_from_the_segment_on: root_of_the_inode_creation + 2 - first_write_of_the_segment,
            withheld_among_them: vec![root_of_the_inode_creation + 1 - first_write_of_the_segment]
                .leak(),
        },
        None,
    );
    let tally = &injection.tally;
    let findings_after: Vec<String> = injection
        .new_findings
        .iter()
        .filter(|finding| finding.stage == CrashStateStage::AfterTheWritableMountAndOnePublish)
        .map(|finding| {
            let rendered = finding.render();
            rendered.chars().take(400).collect()
        })
        .collect();
    let mut crash_image = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    crash_image.apply_writes(&writes[..=root_of_the_inode_creation]);
    let mount_run = mount_and_publish_once_on_the_crash_image(
        &crash_image,
        &execution.device_width.parameters(),
        execution,
        history.seed,
        &injection.crash_points[0],
    );
    let read_back = |pool: &MemoryPool| -> Vec<(u64, u64, u32, u64)> {
        singlefs_core::mounted_read::mount_read_only(pool)
            .map(|mounted| {
                mounted
                    .mounted
                    .inode_records()
                    .iter()
                    .map(|record| (record.inode, record.object_birth.0, record.nlink, record.size))
                    .collect()
            })
            .unwrap_or_default()
    };
    let crashed = read_back(&crash_image);
    let after = read_back(&mount_run.pool_after);
    let crashed_numbers: BTreeSet<u64> = crashed.iter().map(|r| r.0).collect();
    let after_numbers: BTreeSet<u64> = after.iter().map(|r| r.0).collect();
    let assertion_a = crashed_numbers.is_subset(&after_numbers)
        && after_numbers.len() == crashed_numbers.len() + 1;
    let violations: Vec<&'static str> = checker_violations_of(&mount_run.pool_after)
        .into_iter()
        .map(|(invariant, _)| invariant)
        .collect();
    let _ = writeln!(
        out,
        "variant={variant}: outcome={:?} (publishes, refused)=({}, {}) findings_after_mount={} assertion_a={assertion_a} crashed(inode,birth,nlink,size)={crashed:?} after={after:?} checker_violations_on_pool_after={violations:?}",
        mount_run.outcome,
        tally.publishes_after_the_writable_mount,
        tally.publishes_after_the_writable_mount_refused,
        findings_after.len(),
    );
    for finding in &findings_after {
        let _ = writeln!(out, "  finding: {finding}");
    }
    std::env::remove_var("OPUS_K2A_VARIANT");
    out
}

/// F3：用真的 `draw_second_crash_points`，在一份真挂载写表（k3 control 那一次挂载）上，对第一次崩溃段长 `segment_length` 的全部真子集各摆一次，
/// 比混指纹与不混（持久集合交空，等于变异「指纹换成空子集的」，与 F3 之前只混段号同一串随机数）两种摆法：不同二次崩溃态的并集多大、种子有几个不同。
#[must_use]
pub fn f3_run(seed: u64, segment_index: usize, segment_length: usize) -> String {
    let (history, execution) = history_ff_ow();
    let (writes, segments, _stream_indexes, first_ow) =
        writes_of(&history, execution, StepPosition::StartingPoint);
    let segment = segments
        .iter()
        .position(|segment| segment[0] == first_ow)
        .expect("覆盖写从一段的第一个写起");
    let mut first_crash_image = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    first_crash_image.apply_writes(&writes[..first_ow]);
    let mount_run = mount_and_publish_once_on_the_crash_image(
        &first_crash_image,
        &execution.device_width.parameters(),
        execution,
        history.seed,
        &CrashPoint {
            segment_index: segment,
            persisted_within_the_segment: vec![false],
            step: StepPosition::Operation(0),
            operation_kind: Some(HistoryOperationKind::PublishOverwrite),
        },
    );
    let (mount_writes, mount_segments, _) = writes_and_segments_with_stream_indexes(
        &mount_run.mount_operations,
        &execution.device_width.fixed_geometry(),
    );
    let phase_of_each_write = writable_mount_phase_of_each_write(
        &mount_writes,
        mount_run.publishes_before_the_deletion_queue_continuation,
        mount_run.publishes_before_the_instance_table_sweep,
    );
    let mut with_fingerprint: BTreeSet<SecondCrashPoint> = BTreeSet::new();
    let mut without_fingerprint: BTreeSet<SecondCrashPoint> = BTreeSet::new();
    let mut seeds: BTreeSet<u64> = BTreeSet::new();
    let mut distinct_draws_with: BTreeSet<Vec<SecondCrashPoint>> = BTreeSet::new();
    let subsets = (1u64 << segment_length) - 1;
    for mask in 0..subsets {
        let persisted = bits_of(mask, segment_length);
        seeds.insert(
            seed ^ SECOND_CRASH_POINT_SEED_SALT
                ^ u64::try_from(segment_index).expect("段号")
                ^ fingerprint_of_the_persisted_subset(&persisted),
        );
        let crash_point = CrashPoint {
            segment_index,
            persisted_within_the_segment: persisted,
            step: StepPosition::Operation(0),
            operation_kind: None,
        };
        let drawn = draw_second_crash_points(
            HistorySeed(seed),
            &crash_point,
            &mount_segments,
            &phase_of_each_write,
        );
        with_fingerprint.extend(drawn.iter().cloned());
        distinct_draws_with.insert(drawn);
        let without = CrashPoint {
            persisted_within_the_segment: Vec::new(),
            ..crash_point
        };
        without_fingerprint.extend(draw_second_crash_points(
            HistorySeed(seed),
            &without,
            &mount_segments,
            &phase_of_each_write,
        ));
    }
    let mut per_phase_writes: BTreeMap<WritableMountPhase, usize> = BTreeMap::new();
    for phase in &phase_of_each_write {
        *per_phase_writes.entry(*phase).or_insert(0) += 1;
    }
    format!(
        "f3 seed={seed} segment_index={segment_index} segment_length={segment_length} first_crash_states={subsets} mount_writes={} mount_segments={} writes_per_phase={per_phase_writes:?} distinct_seeds={} distinct_draw_sets_with_fingerprint={} union_second_crash_states_with_fingerprint={} union_without_fingerprint={}",
        mount_writes.len(),
        mount_segments.len(),
        seeds.len(),
        distinct_draws_with.len(),
        with_fingerprint.len(),
        without_fingerprint.len()
    )
}

/// 指纹函数本身（给 main 里的算术模型逐位对拍用）。
#[must_use]
pub fn fingerprint(persisted: &[bool]) -> u64 {
    fingerprint_of_the_persisted_subset(persisted)
}

#[must_use]
pub const fn salts() -> (u64, u64) {
    (CRASH_POINT_SEED_SALT, SECOND_CRASH_POINT_SEED_SALT)
}

fn subsets_of(items: &[usize]) -> Vec<Vec<usize>> {
    (1u32..(1 << items.len()))
        .map(|mask| {
            items
                .iter()
                .enumerate()
                .filter(|(bit, _)| mask & (1 << bit) != 0)
                .map(|(_, item)| *item)
                .collect()
        })
        .collect()
}

/// K3 扫：第一次崩溃从第一个文件那次的记录起、摆到覆盖写的根（或连覆盖写的轮换），扣下第一个文件那次的记录与轮换的任一非空子集；
/// 二次崩溃从写行的记录起、摆到写行的根或它后面的一槽、两槽轮换，扣下写行的记录与（摆到了的）轮换的任一非空子集。每个组合两种接线各跑一次。
#[must_use]
pub fn k3_sweep(last_choice: usize, subset_from: usize, subset_to: usize) -> String {
    let mut out = String::new();
    let (history, execution) = history_ff_ow();
    let (writes, segments, _stream_indexes, first_ow) =
        writes_of(&history, execution, StepPosition::StartingPoint);
    let ow_root = (first_ow..writes.len())
        .find(|w| writes[*w].kind == StepKind::RootRecordFua)
        .expect("覆盖写的根");
    let ff_root = (0..first_ow)
        .rev()
        .find(|w| writes[*w].kind == StepKind::RootRecordFua)
        .expect("第一个文件的根");
    let previous_root = (0..ff_root)
        .rev()
        .find(|w| writes[*w].kind == StepKind::RootRecordFua)
        .expect("前一次的根");
    let ff_records: Vec<usize> = (previous_root + 1..ff_root)
        .filter(|w| writes[*w].kind == StepKind::JournalRecord)
        .collect();
    let ff_rotation: Vec<usize> = (ff_root + 1..first_ow)
        .filter(|w| writes[*w].kind == StepKind::SystemConfigurationSlot)
        .collect();
    let first_segment = segment_containing(&segments, ff_records[0]);
    let first_write = segments[first_segment][0];
    let candidates: Vec<usize> = ff_records.iter().chain(&ff_rotation).map(|w| w - first_write).collect();
    let (mut combos, mut same, mut branch_only, mut counts_differ, mut not_judged) = (0, 0, 0, 0, 0);
    for last in [[ow_root, writes.len() - 1][last_choice]] {
        let first_count = last + 1 - first_write;
        for first_withheld in subsets_of(&candidates).into_iter().skip(subset_from).take(subset_to - subset_from) {
            let mut first_crash_image = MemoryPool::with_devices(
                &[DeviceIdentity(0), DeviceIdentity(1)],
                execution.device_width.device_bytes(),
            );
            first_crash_image.apply_writes(&writes[..first_write]);
            let persisted: Vec<RetainedWrite> = (0..first_count)
                .filter(|k| !first_withheld.contains(k))
                .map(|k| writes[first_write + k].clone())
                .collect();
            first_crash_image.apply_writes(&persisted);
            let mount_run = mount_and_publish_once_on_the_crash_image(
                &first_crash_image,
                &execution.device_width.parameters(),
                execution,
                history.seed,
                &CrashPoint {
                    segment_index: first_segment,
                    persisted_within_the_segment: (0..first_count).map(|k| !first_withheld.contains(&k)).collect(),
                    step: StepPosition::Operation(0),
                    operation_kind: Some(HistoryOperationKind::PublishOverwrite),
                },
            );
            if !matches!(mount_run.outcome, WritableMountOutcome::Published) {
                not_judged += 1;
                let _ = writeln!(out, "first last={last} withheld={first_withheld:?}: first-crash mount {:?}", mount_run.outcome);
                continue;
            }
            let (mount_writes, mount_segments, _) = writes_and_segments_with_stream_indexes(
                &mount_run.mount_operations,
                &execution.device_width.fixed_geometry(),
            );
            let phase_of_each_write = writable_mount_phase_of_each_write(
                &mount_writes,
                mount_run.publishes_before_the_deletion_queue_continuation,
                mount_run.publishes_before_the_instance_table_sweep,
            );
            let root_row = (0..mount_writes.len())
                .find(|w| phase_of_each_write[*w] == WritableMountPhase::RowPublish && mount_writes[*w].kind == StepKind::RootRecordFua)
                .expect("写行的根");
            let row_records: Vec<usize> = (0..root_row)
                .filter(|w| phase_of_each_write[*w] == WritableMountPhase::RowPublish && mount_writes[*w].kind == StepKind::JournalRecord)
                .collect();
            let rotation: Vec<usize> = (root_row + 1..mount_writes.len())
                .take_while(|w| mount_writes[*w].kind == StepKind::SystemConfigurationSlot)
                .collect();
            let second_segment = segment_containing(&mount_segments, row_records[0]);
            let second_first = mount_segments[second_segment][0];
            for extra in 0..=rotation.len() {
                let second_last = if extra == 0 { root_row } else { rotation[extra - 1] };
                let second_count = second_last + 1 - second_first;
                let second_candidates: Vec<usize> = row_records
                    .iter()
                    .chain(rotation[..extra].iter())
                    .map(|w| w - second_first)
                    .collect();
                for second_withheld in subsets_of(&second_candidates) {
                    combos += 1;
                    let mut results = Vec::new();
                    for wiring in ["second", "first"] {
                        std::env::set_var("OPUS_K3_WIRING", wiring);
                        let injection = inject_crashes_into_history_drawing_second_crashes(
                            &history,
                            execution,
                            CrashPointDraw::ExactlyOneCrashState {
                                segment_index: first_segment,
                                writes_from_the_segment_on: first_count,
                                withheld_among_them: first_withheld.clone().leak(),
                            },
                            SecondCrashPointDraw::ExactlyOneSecondCrashState {
                                segment_index: second_segment,
                                writes_from_the_segment_on: second_count,
                                withheld_among_them: second_withheld.clone().leak(),
                            },
                            None,
                        );
                        let remount: Vec<String> = injection
                            .new_findings
                            .iter()
                            .filter(|f| matches!(f.stage, CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(_)))
                            .map(|f| f.observation.model_disagreement.as_ref().map_or_else(String::new, |d| {
                                ["WitnessedCounterAboveTheLastRecordOfTheSelectedVersion",
                                 "RecordAtTheWitnessedCounterNewerThanTheSelectedVersion",
                                 "NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable"]
                                    .iter().filter(|n| d.implementation_answer.contains(*n)).copied().collect::<Vec<_>>().join("+")
                            }))
                            .collect();
                        results.push((
                            injection.tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
                            injection.tally.remount_refusals_with_the_witness_criterion_true_without_read_faults,
                            remount,
                        ));
                    }
                    let (a, b) = (&results[0], &results[1]);
                    if (a.0, a.1) != (b.0, b.1) {
                        counts_differ += 1;
                        let _ = writeln!(out, "COUNTS-DIFFER first(last={last},withheld={first_withheld:?}) second(last={second_last},withheld={second_withheld:?}): correct={:?} mutant={:?}", (a.0, a.1, &a.2), (b.0, b.1, &b.2));
                    } else if a.2 != b.2 {
                        branch_only += 1;
                    } else {
                        same += 1;
                    }
                }
            }
        }
    }
    std::env::remove_var("OPUS_K3_WIRING");
    let _ = writeln!(out, "k3-sweep combos={combos} same={same} branch_only={branch_only} counts_differ={counts_differ} first_crash_mount_not_published={not_judged} (ff_records={ff_records:?} ff_rotation={ff_rotation:?} ow_root={ow_root} writes={})", writes.len());
    out
}

/// F3 内联用例原样的输入（种子 7、段号 5、写行 4 个写 + 暖机 8 个写两段，子集 [真,假,假] 与 [假,真,假]）：两份摆法同不同。
#[must_use]
pub fn f3_inline() -> String {
    let phase_of_each_write: Vec<WritableMountPhase> = (0..12)
        .map(|w| if w < 4 { WritableMountPhase::RowPublish } else { WritableMountPhase::WarmUp })
        .collect();
    let segments: Vec<Vec<usize>> = vec![(0..4).collect(), (4..12).collect()];
    let with = |persisted: Vec<bool>| CrashPoint {
        segment_index: 5,
        persisted_within_the_segment: persisted,
        step: StepPosition::Operation(0),
        operation_kind: Some(HistoryOperationKind::PublishOverwrite),
    };
    let a = draw_second_crash_points(HistorySeed(7), &with(vec![true, false, false]), &segments, &phase_of_each_write);
    let b = draw_second_crash_points(HistorySeed(7), &with(vec![false, true, false]), &segments, &phase_of_each_write);
    format!(
        "f3-inline fingerprint={} draws_differ={} (inline test assert_ne {})",
        std::env::var("OPUS_F3_FP").unwrap_or_else(|_| "fnv1a".to_string()),
        a != b,
        if a != b { "green" } else { "red" }
    )
}
