#!/usr/bin/env python3
"""在草稿副本上装 V242u 开关、接线变体开关与追踪输出（只改副本，不碰仓内与冻结副本）。"""
import sys, pathlib
root = pathlib.Path(sys.argv[1])

def replace_once(path, old, new):
    p = root / path
    text = p.read_text()
    n = text.count(old)
    if n != 1:
        sys.exit(f"{path}: 旧串命中 {n} 次（要恰好 1 次）：{old[:80]!r}")
    p.write_text(text.replace(old, new))

# 1. V242u（crates/mutations.tsv 第 2283 行那一条）照原样做成运行时开关：B18_V242U 设了且这次发布带单元，先轮换、后根槽 FUA、再屏障。
replace_once("crates/singlefs-core/src/transaction.rs",
"""    writer.perform_commit_step(CommitStep::Barrier)?;
    persist_the_root_then_rotate_the_system_configuration(
        writer,
        writes.checkpoint_txg,
        &writes.root_slot,
        writes.journal_tail,
        writes.journal_instance,
    )
}""",
"""    writer.perform_commit_step(CommitStep::Barrier)?;
    if std::env::var_os("B18_V242U").is_some() && !writes.units.is_empty() {
        writer.perform_commit_step(CommitStep::RotateSystemConfigurationSlots {
            journal_tail: writes.journal_tail,
            journal_instance: writes.journal_instance,
        })?;
        writer.perform_commit_step(CommitStep::WriteRootRecordForceUnitAccess {
            checkpoint_txg: writes.checkpoint_txg,
            root_slot: &writes.root_slot,
        })?;
        return writer.perform_commit_step(CommitStep::Barrier);
    }
    persist_the_root_then_rotate_the_system_configuration(
        writer,
        writes.checkpoint_txg,
        &writes.root_slot,
        writes.journal_tail,
        writes.journal_instance,
    )
}""")

F = "crates/singlefs-checker-tier/src/crash_injection.rs"
# 2. 变体开关：B18_VARIANT=<名>。
replace_once(F,
"""const SECOND_CRASH_POINT_SEED_SALT: u64""",
"""fn b18_variant(name: &str) -> bool {
    std::env::var("B18_VARIANT").is_ok_and(|value| value.split(',').any(|v| v == name))
}
fn b18_trace() -> bool {
    std::env::var_os("B18_TRACE").is_some()
}
const SECOND_CRASH_POINT_SEED_SALT: u64""")

# 第二截：读故障认定的下标（s2_index_write：拿写表下标当录制流下标）。
replace_once(F,
"""        let read_faults_in_the_crash_state = marks.read_faults_in_a_crash_state(
            &step_kinds,
            stream_indexes[first_write_of_the_segment + writes_in_the_segment - 1],
        );""",
"""        let read_faults_in_the_crash_state = marks.read_faults_in_a_crash_state(
            &step_kinds,
            if b18_variant("s2_index_write") {
                first_write_of_the_segment + writes_in_the_segment - 1
            } else {
                stream_indexes[first_write_of_the_segment + writes_in_the_segment - 1]
            },
        );""")

# 第二截：所选那一版（s2_version_none）。
replace_once(F,
"""            RefusalOnTheCrashImage {
                crash_image_materialized: &crash_image_materialized,
                version_landed_on_after_the_crash: report.effective_root,""",
"""            RefusalOnTheCrashImage {
                crash_image_materialized: &crash_image_materialized,
                version_landed_on_after_the_crash: if b18_variant("s2_version_none") {
                    None
                } else {
                    report.effective_root
                },""")

# 第三截调用点：镜像、所选那一版、读故障认定。
replace_once(F,
"""        let second_crash_image_materialized = materialized_crash_image(&image);
        judge_the_remount_after_a_second_crash(
            tally,
            findings,
            &content_after_the_uninterrupted_mount,
            &RefusalOnTheCrashImage {
                crash_image_materialized: &second_crash_image_materialized,
                version_landed_on_after_the_crash: report.effective_root,
                read_only_read_back: if disagreement.is_none() {
                    ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet
                } else {
                    ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet
                },
                read_faults: read_faults_in_the_first_crash_state,
            },""",
"""        let second_crash_image_materialized = materialized_crash_image(&image);
        let illegal_before = tally.remount_refusals_with_the_witness_criterion_true_without_read_faults;
        let judged_before = tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration;
        let refused_before = tally
            .remount_idempotence_not_judged
            .get(&RemountIdempotenceNotJudged::TheRemountWasRefused)
            .copied()
            .unwrap_or(0);
        judge_the_remount_after_a_second_crash(
            tally,
            findings,
            &content_after_the_uninterrupted_mount,
            &RefusalOnTheCrashImage {
                crash_image_materialized: if b18_variant("s3_call_first_image") {
                    crash_image_materialized
                } else if b18_variant("s3_call_base_without_segment") {
                    &second_base
                } else {
                    &second_crash_image_materialized
                },
                version_landed_on_after_the_crash: if b18_variant("s3_call_first_version") {
                    fields_read_after_the_first_crash.report.effective_root
                } else {
                    report.effective_root
                },
                read_only_read_back: if disagreement.is_none() {
                    ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet
                } else {
                    ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet
                },
                read_faults: if b18_variant("s3_call_injected") {
                    ReadFaultsInTheCrashState::Injected
                } else {
                    read_faults_in_the_first_crash_state
                },
            },""")

# 追踪：每个二次崩溃一行。
replace_once(F,
"""            second.phase,
            seed,
            crash_point,
            image_directory,
        );
        tally.second_crash_checker_runs += 1;""",
"""            second.phase,
            seed,
            crash_point,
            image_directory,
        );
        if b18_trace() {
            let row_publish_writes = phase_of_each_write
                .iter()
                .filter(|phase| **phase == WritableMountPhase::RowPublish)
                .count();
            let bits = |v: &[bool]| v.iter().map(|b| if *b { '1' } else { '0' }).collect::<String>();
            let kinds: Vec<&str> = mount_segments[second.segment_index]
                .iter()
                .map(|w| mount_writes[*w].kind.name())
                .collect();
            eprintln!(
                "B18TRACE first_seg={} first_bits={} first_version={:?} mount_writes={} mount_segments={} row_publish_writes={} second_seg={} second_kinds={:?} second_bits={} phase={} second_version={:?} remount_refused={} c670_judged={} c670_illegal={}",
                crash_point.segment_index,
                bits(&crash_point.persisted_within_the_segment),
                fields_read_after_the_first_crash.report.effective_root,
                mount_writes.len(),
                mount_segments.iter().map(|s| s.len().to_string()).collect::<Vec<_>>().join("+"),
                row_publish_writes,
                second.segment_index,
                kinds,
                bits(&second.persisted_within_the_segment),
                second.phase.name(),
                report.effective_root,
                tally.remount_idempotence_not_judged.get(&RemountIdempotenceNotJudged::TheRemountWasRefused).copied().unwrap_or(0) - refused_before,
                tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration - judged_before,
                tally.remount_refusals_with_the_witness_criterion_true_without_read_faults - illegal_before,
            );
        }
        tally.second_crash_checker_runs += 1;""")
print("patched")
