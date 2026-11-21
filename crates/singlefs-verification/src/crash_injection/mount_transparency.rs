//! 挂载透明：崩溃镜像的只读恢复与只挂载、挂载加一次发布之后的只读恢复逐字段比，扫实例表删的行照放回去比。

use super::*;

/// 挂载透明整个动作没判的理由（计数名）：崩溃后镜像上的可写挂载被拒，判的是拒绝本身（[`judge_the_writable_mount_on_the_crash_image`]）。
const MOUNT_TRANSPARENCY_NOT_JUDGED_THE_MOUNT_WAS_REFUSED: &str = "可写挂载被拒（判拒绝本身）";
/// 挂载做成、之后那一次发布被拒：没有「挂载加一次发布」那一版可比。
const MOUNT_TRANSPARENCY_NOT_JUDGED_THE_PUBLISH_AFTER_THE_MOUNT_WAS_REFUSED: &str =
    "挂载之后那一次发布被拒";
/// 二次崩溃镜像上：没崩过的那一次可写挂载被拒，没有挂载交回值（推没推抬 F）。
const MOUNT_TRANSPARENCY_NOT_JUDGED_THE_UNINTERRUPTED_MOUNT_WAS_REFUSED: &str =
    "没崩过的那一次可写挂载被拒";
/// 第一次崩溃那个态注入过读故障（[`StreamMarks::read_faults_in_a_crash_state`]）：挂载透明的射程只管不带故障的状态
/// （D13（验证路线） 已定项 1），崩溃镜像上那一次挂载整个不判，它途中的每个二次崩溃也不判（各记一次）。
pub(super) const MOUNT_TRANSPARENCY_NOT_JUDGED_ON_A_READ_FAULTED_CRASH_STATE: &str =
    "崩溃态注入过读故障（挂载透明只管不带故障的状态）";

/// 挂载透明的一判记进计数：整个动作没判的按理由、某一样没判的按理由、判红的按字段（名字里带量过或推的）；交回判红的那几样，
/// 一样一条违例（名字是 [`singlefs_verification::mount_transparency::MountTransparencyField::violation_name`]）。
fn violations_of_the_mount_transparency(
    tally: &mut CrashInjectionTally,
    transparency: &MountTransparency,
) -> Vec<(&'static str, String)> {
    let verdicts = match transparency {
        MountTransparency::NotJudged(reason) => {
            *tally
                .mount_transparency_not_judged
                .entry(reason.name())
                .or_insert(0) += 1;
            return Vec::new();
        }
        MountTransparency::Judged(verdicts) => verdicts,
    };
    let mut violations = Vec::new();
    for verdict in verdicts {
        match &verdict.outcome {
            FieldOutcome::Holds => {}
            FieldOutcome::NotJudged(reason) => {
                *tally
                    .mount_transparency_fields_not_judged
                    .entry(reason.name())
                    .or_insert(0) += 1;
            }
            FieldOutcome::Violated { expected, observed } => {
                let name = verdict.field.violation_name(verdict.basis);
                *tally
                    .mount_transparency_violations_by_field
                    .entry(name)
                    .or_insert(0) += 1;
                violations.push((name, format!("期望 {expected}；读出 {observed}")));
            }
        }
    }
    violations
}

/// 挂载透明判红时记进违例的名字：R2 那一版是清扫发布那一版，它的实例表里却仍有清扫说删掉了的行（C704（挂载透明不认清扫发布））。
pub const MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH: &str =
    "挂载透明·实例表行（清扫发布那一版里仍有它删掉的行）";

/// R2 那一版的实例表行对清扫发布删的行是什么样（C704（挂载透明不认清扫发布）的处置：判法先跟上，再让战役带清扫配置）。
#[derive(Debug)]
enum InstanceTableRowsAgainstTheSweepPublish {
    /// 挂载没删行、R2 那一版不是清扫发布那一版，或它的实例表读不出（读不出照原样交给逐字段判，实例表行那一样照判红）：照原样比。
    ComparedAsRead,
    /// R2 那一版是清扫发布那一版、被删的行一条都不在：把它们放回去之后的 R2（其余的行照 D18 已定项 11「恢复写的行」判）。
    DeletedRowsPutBack(Box<RecoveryFieldsRead>),
    /// R2 那一版是清扫发布那一版，被删的行还在：判红。
    DeletedRowsStillPresent(Vec<InstanceRow>),
}

/// 按清扫发布那一版的根身份认 R2（只挂载与二次崩溃比恢复落到的那一版、挂载加一次发布比发布之前那一版——清扫是挂载交回之前最后一件事，
/// 两种比法拿到的都可能是它），判被删的行在不在。
fn instance_table_rows_against_the_sweep_publish(
    after_the_action: &RecoveryFieldsRead,
    rows_deleted_by_the_sweep: Option<&RowsDeletedByTheInstanceTableSweepPublish>,
) -> InstanceTableRowsAgainstTheSweepPublish {
    let Some(rows_deleted_by_the_sweep) = rows_deleted_by_the_sweep else {
        return InstanceTableRowsAgainstTheSweepPublish::ComparedAsRead;
    };
    let Some(compared) = &after_the_action.compared_version else {
        return InstanceTableRowsAgainstTheSweepPublish::ComparedAsRead;
    };
    if (compared.root.instance, compared.root.checkpoint_txg)
        != rows_deleted_by_the_sweep.sweep_publish
    {
        return InstanceTableRowsAgainstTheSweepPublish::ComparedAsRead;
    }
    let Some(rows_read) = &compared.instance_table_rows else {
        return InstanceTableRowsAgainstTheSweepPublish::ComparedAsRead;
    };
    let deleted_instances: BTreeSet<InstanceGeneration> = rows_deleted_by_the_sweep
        .rows_deleted
        .iter()
        .map(|row| row.instance)
        .collect();
    let still_present: Vec<InstanceRow> = rows_read
        .iter()
        .filter(|row| deleted_instances.contains(&row.instance))
        .copied()
        .collect();
    if !still_present.is_empty() {
        return InstanceTableRowsAgainstTheSweepPublish::DeletedRowsStillPresent(still_present);
    }
    let rows_with_the_deleted_put_back: Vec<InstanceRow> = rows_read
        .iter()
        .chain(&rows_deleted_by_the_sweep.rows_deleted)
        .copied()
        .collect();
    let mut put_back = after_the_action.clone();
    put_back
        .compared_version
        .as_mut()
        .expect("上面判过 R2 那一版在")
        .instance_table_rows = Some(rows_with_the_deleted_put_back);
    InstanceTableRowsAgainstTheSweepPublish::DeletedRowsPutBack(Box::new(put_back))
}

/// 挂载透明逐字段判（`mount_transparency::judge_mount_transparency`）之前，挂载末尾的清扫发布删了行时按 C704（挂载透明不认清扫发布）先处置 R2：
/// 那里的逐字段判法按「恢复写的行」比实例表行，不认清扫发布删行。R2 那一版是清扫发布那一版而被删的行一条都不在 ⇒ 交回放回之后的 R2；
/// 被删的行还在 ⇒ R2 照原样交回、另交一条违例（[`MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH`]，
/// 计进按字段分的判红计数）；别的照原样交回。被删的是哪几行取挂载交回值（与 F 取挂载交回的抬 F、续做取挂载交回的根指针同一个做法），
/// 判法不另算哪几行该删。
fn after_the_action_with_the_rows_deleted_by_the_sweep_put_back<'read>(
    tally: &mut CrashInjectionTally,
    after_the_action: &'read RecoveryFieldsRead,
    rows_deleted_by_the_sweep: Option<&RowsDeletedByTheInstanceTableSweepPublish>,
) -> (Cow<'read, RecoveryFieldsRead>, Vec<(&'static str, String)>) {
    match instance_table_rows_against_the_sweep_publish(after_the_action, rows_deleted_by_the_sweep)
    {
        InstanceTableRowsAgainstTheSweepPublish::ComparedAsRead => {
            (Cow::Borrowed(after_the_action), Vec::new())
        }
        InstanceTableRowsAgainstTheSweepPublish::DeletedRowsPutBack(put_back) => {
            tally.mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back += 1;
            (Cow::Owned(*put_back), Vec::new())
        }
        InstanceTableRowsAgainstTheSweepPublish::DeletedRowsStillPresent(still_present) => {
            *tally
                .mount_transparency_violations_by_field
                .entry(MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH)
                .or_insert(0) += 1;
            (
                Cow::Borrowed(after_the_action),
                vec![(
                    MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH,
                    format!(
                        "清扫发布 {:?} 删了 {:?}；那一版的实例表里仍有 {still_present:?}",
                        rows_deleted_by_the_sweep.map(|deleted| deleted.sweep_publish),
                        rows_deleted_by_the_sweep.map(|deleted| &deleted.rows_deleted)
                    ),
                )],
            )
        }
    }
}

/// 第一次崩溃镜像上一个动作的挂载透明：R1（`before_the_mount`）与动作之后的池（`pool_after_the_action`）上的只读恢复（`after_the_action_as_read`；挂载末尾的清扫发布删了行时先按 C704 处置）逐字段比；
/// 判红的落成这一截（[`CrashStateStage::MountTransparencyAfterTheCrash`]）的新发现，镜像留的是动作之后的池。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：计数与发现、哪个动作、两次恢复与动作之后的池、动作与挂载续做那几步的交回值、清扫发布删的行、这一个崩溃状态落在哪"
)]
fn judge_mount_transparency_of_one_comparison(
    tally: &mut CrashInjectionTally,
    findings: &mut FindingsOfOneHistory,
    comparison: MountTransparencyComparison,
    before_the_mount: &RecoveryFieldsRead,
    pool_after_the_action: &MemoryPool,
    after_the_action_as_read: &RecoveryFieldsRead,
    action: MountTransparencyAction,
    continuation: &UserVisibleTreeRootPointersOfTheContinuation,
    rows_deleted_by_the_sweep: Option<&RowsDeletedByTheInstanceTableSweepPublish>,
    seed: HistorySeed,
    crash_point: &CrashPoint,
    image_directory: Option<&Path>,
) {
    let (compared_after_the_action, rows_deleted_by_the_sweep_still_present) =
        after_the_action_with_the_rows_deleted_by_the_sweep_put_back(
            tally,
            after_the_action_as_read,
            rows_deleted_by_the_sweep,
        );
    let after_the_action: &RecoveryFieldsRead = &compared_after_the_action;
    let transparency =
        judge_mount_transparency(before_the_mount, after_the_action, action, continuation);
    match &transparency {
        MountTransparency::Judged(_) => {
            *tally
                .mount_transparency_judged
                .entry(comparison)
                .or_insert(0) += 1;
        }
        MountTransparency::NotJudged(_) => {}
    }
    let mut violations = violations_of_the_mount_transparency(tally, &transparency);
    violations.extend(rows_deleted_by_the_sweep_still_present);
    if violations.is_empty() {
        return;
    }
    let image = CrashImage {
        base: pool_after_the_action,
        writes: &[],
        persisted: Vec::new(),
    };
    let detail = violations
        .iter()
        .map(|(name, detail)| format!("{name}：{detail}"))
        .collect::<Vec<_>>()
        .join("；");
    let observation = crash_state_observation(
        &image,
        crash_point,
        violations,
        None,
        RecordCheck::default(),
    );
    findings.settle(
        tally,
        FailureAtAStage {
            seed,
            crash_point,
            stage: CrashStateStage::MountTransparencyAfterTheCrash(comparison),
            observation,
            read_back: detail,
            image: &image,
        },
        image_directory,
    );
}

/// 挂载做成了（`floor_raised_by_the_mount` 是 Some）时一起交回的续做那几步的两根指针。
fn continuation_of_a_mount_that_was_made(
    mount_run: &WritableMountOnTheCrashImage,
) -> &UserVisibleTreeRootPointersOfTheContinuation {
    mount_run
        .user_visible_tree_root_pointers_of_the_continuation
        .as_ref()
        .expect("挂载做成时抬 F 与续做那几步的根指针一起交回（mount_and_publish_once_on_the_crash_image 里同一个 Ok 取的两样）")
}

/// 第一次崩溃镜像上的挂载透明：只挂载（v2a，`after_the_uninterrupted_mount` 是那个池与它上面的只读恢复）、
/// 挂载加一次发布（比发布之前那一版）各判一次。挂载被拒的不判（判的是拒绝本身），发布被拒的不判「挂载加一次发布」那一个。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：计数与发现、R1、挂载那一趟与 v2a、这一个崩溃状态落在哪"
)]
pub(super) fn judge_mount_transparency_on_the_crash_image(
    tally: &mut CrashInjectionTally,
    findings: &mut FindingsOfOneHistory,
    before_the_mount: &RecoveryFieldsRead,
    mount_run: &WritableMountOnTheCrashImage,
    after_the_uninterrupted_mount: Option<(&MemoryPool, &RecoveryFieldsRead)>,
    seed: HistorySeed,
    crash_point: &CrashPoint,
    image_directory: Option<&Path>,
) {
    match (
        after_the_uninterrupted_mount,
        mount_run.floor_raised_by_the_mount,
    ) {
        (Some((pool, after_the_mount)), Some(floor)) => judge_mount_transparency_of_one_comparison(
            tally,
            findings,
            MountTransparencyComparison::TheMountAlone,
            before_the_mount,
            pool,
            after_the_mount,
            MountTransparencyAction::MountAlone(floor),
            continuation_of_a_mount_that_was_made(mount_run),
            mount_run.rows_deleted_by_the_instance_table_sweep.as_ref(),
            seed,
            crash_point,
            image_directory,
        ),
        (None, _) | (_, None) => {
            *tally
                .mount_transparency_not_judged
                .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_THE_MOUNT_WAS_REFUSED)
                .or_insert(0) += 1;
        }
    }
    match (&mount_run.outcome, mount_run.floor_raised_by_the_mount) {
        (WritableMountOutcome::Published, Some(floor)) => {
            let after_the_publish = read_recovery_fields(
                &mount_run.pool_after,
                ComparedVersion::TheVersionBeforeTheRecoveredOne,
            );
            judge_mount_transparency_of_one_comparison(
                tally,
                findings,
                MountTransparencyComparison::TheMountAndOnePublish,
                before_the_mount,
                &mount_run.pool_after,
                &after_the_publish,
                MountTransparencyAction::MountAndOnePublish(floor),
                continuation_of_a_mount_that_was_made(mount_run),
                mount_run.rows_deleted_by_the_instance_table_sweep.as_ref(),
                seed,
                crash_point,
                image_directory,
            );
        }
        (WritableMountOutcome::PublishRefused { .. }, _) => {
            *tally
                .mount_transparency_not_judged
                .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_THE_PUBLISH_AFTER_THE_MOUNT_WAS_REFUSED)
                .or_insert(0) += 1;
        }
        (WritableMountOutcome::MountRefused { .. } | WritableMountOutcome::Published, _) => {
            *tally
                .mount_transparency_not_judged
                .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_THE_MOUNT_WAS_REFUSED)
                .or_insert(0) += 1;
        }
    }
}

/// 二次崩溃镜像上的挂载透明（F6「加二次」）：只读恢复读出的与 R1 比。挂载推了抬 F 而二次崩溃镜像上
/// 抬 F 的写有持久的（从第一串把新 F 写进系统配置的那个写起，`persisted_in_the_mount` 与挂载那一段写表逐条对应）时，F 那一样不判
/// （条款没写落在哪一处）；暖机那一次发布的写崩出来的态照 r5 判决 U5 前半句判 F 不变。抬 F 的界在挂载写表里取不出的
/// 这一次挂载的二次崩溃整段 F 不判、计数。没崩过的那一次挂载被拒的不判。
/// 没崩过的那一次挂载末尾的清扫发布删了行时，实例表行那一样照 C704（挂载透明不认清扫发布）的处置判
/// （[`after_the_action_with_the_rows_deleted_by_the_sweep_put_back`]）。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：计数、R1 与二次崩溃镜像上的恢复、没崩过的那一次挂载的抬 F 界与续做根指针与清扫删的行、二次崩溃的持久集合与段"
)]
pub(super) fn mount_transparency_violations_of_a_second_crash(
    tally: &mut CrashInjectionTally,
    fields_read_after_the_first_crash: &RecoveryFieldsRead,
    fields_read_after_the_second_crash: &RecoveryFieldsRead,
    floor_raises_in_the_uninterrupted_mount_writes: Option<FloorRaisesInTheMountWrites>,
    continuation_of_the_uninterrupted_mount: Option<&UserVisibleTreeRootPointersOfTheContinuation>,
    rows_deleted_by_the_sweep_of_the_uninterrupted_mount: Option<
        &RowsDeletedByTheInstanceTableSweepPublish,
    >,
    persisted_in_the_mount: &[bool],
    phase: WritableMountPhase,
) -> Vec<(&'static str, String)> {
    let Some(floor_raises_in_the_mount_writes) = floor_raises_in_the_uninterrupted_mount_writes
    else {
        *tally
            .mount_transparency_not_judged
            .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_THE_UNINTERRUPTED_MOUNT_WAS_REFUSED)
            .or_insert(0) += 1;
        return Vec::new();
    };
    let continuation = continuation_of_the_uninterrupted_mount.expect(
        "没崩过的那一次挂载做成时抬 F 的界与续做那几步的根指针一起交回（mount_and_publish_once_on_the_crash_image 里同一个 Ok 取的两样）",
    );
    let floor_raises = floor_raises_in_the_mount_writes.at_a_second_crash(persisted_in_the_mount);
    let (fields_read_after_the_second_crash, rows_deleted_by_the_sweep_still_present) =
        after_the_action_with_the_rows_deleted_by_the_sweep_put_back(
            tally,
            fields_read_after_the_second_crash,
            rows_deleted_by_the_sweep_of_the_uninterrupted_mount,
        );
    let transparency = judge_mount_transparency(
        fields_read_after_the_first_crash,
        &fields_read_after_the_second_crash,
        MountTransparencyAction::SecondCrashInsideTheMount(floor_raises),
        continuation,
    );
    match &transparency {
        MountTransparency::Judged(_) => {
            *tally
                .mount_transparency_judged_on_second_crashes_by_phase
                .entry(phase)
                .or_insert(0) += 1;
        }
        MountTransparency::NotJudged(_) => {}
    }
    let mut violations = violations_of_the_mount_transparency(tally, &transparency);
    violations.extend(rows_deleted_by_the_sweep_still_present);
    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 挂载透明在崩溃注入里真的判了：与上一条同样两段历史、同一种摆法，每个挂载做成的崩溃状态都判了只挂载那一个，
    /// 挂载与之后那次发布都做成的都判了挂载加一次发布那一个（删完那一版上那次发布建 inode），二次崩溃取号、写行、暖机、续做四段各有判过的；
    /// 一样都没红（挂载续做过时用户可见树根指针按续做那几步比、F 认续做推的抬 F：「续做算挂载的一部分」）。
    #[test]
    fn mount_transparency_is_judged_after_every_writable_mount_and_inside_every_mount_phase_without_a_red_field(
    ) {
        let (tally, injections) = crash_injections_into_histories_reaching_every_mount_phase();
        let tally = &tally;
        assert_eq!(
            tally.recoveries_failed, 0,
            "前提：崩溃镜像上的只读恢复都落到了一版：{tally:?}"
        );
        assert!(
            tally.writable_mounts_after_the_crash_succeeded >= 1,
            "前提：崩溃后镜像上的可写挂载做成过：{tally:?}"
        );
        assert_eq!(
            tally
                .mount_transparency_judged
                .get(&MountTransparencyComparison::TheMountAlone)
                .copied()
                .unwrap_or(0),
            tally.writable_mounts_after_the_crash_succeeded,
            "每个挂载做成的崩溃状态都判了只挂载那一个：{:?}",
            tally.mount_transparency_not_judged
        );
        assert_eq!(
            tally
                .mount_transparency_judged
                .get(&MountTransparencyComparison::TheMountAndOnePublish)
                .copied()
                .unwrap_or(0),
            tally.publishes_after_the_writable_mount,
            "每个挂载与发布都做成的崩溃状态都判了挂载加一次发布那一个：{:?}",
            tally.mount_transparency_not_judged
        );
        for phase in WritableMountPhase::ALL {
            assert!(
                tally
                    .mount_transparency_judged_on_second_crashes_by_phase
                    .get(&phase)
                    .copied()
                    .unwrap_or(0)
                    >= 1,
                "崩在挂载的 {} 里的二次崩溃镜像一次都没判过挂载透明：{:?}",
                phase.name(),
                tally.mount_transparency_judged_on_second_crashes_by_phase
            );
        }
        assert!(
            tally.mount_transparency_violations_by_field.is_empty(),
            "挂载透明判红的字段 {:?}：{}",
            tally.mount_transparency_violations_by_field,
            injections
                .iter()
                .flat_map(|injection| injection.new_findings.iter())
                .map(CrashPointFinding::render)
                .collect::<String>()
        );
    }

    /// 清扫发布那一版的根身份（实例, checkpoint_txg），两条用例共用：R2 那一版就是它。
    const SWEEP_PUBLISH_OF_THE_C704_CASES: (InstanceGeneration, CheckpointTxg) =
        (InstanceGeneration(3), CheckpointTxg(9));
    /// 清扫删掉的那一行与留下的那一行。
    const ROW_DELETED_BY_THE_SWEEP: InstanceRow = InstanceRow {
        instance: InstanceGeneration(1),
        selected_root_txg: CheckpointTxg(4),
        applied_transaction_high_water: 2,
    };
    const ROW_KEPT_BY_THE_SWEEP: InstanceRow = InstanceRow {
        instance: InstanceGeneration(2),
        selected_root_txg: CheckpointTxg(7),
        applied_transaction_high_water: 5,
    };

    /// 一次只读恢复的字段，拿来比的那一版（R2）的根身份是 `root_identity`、实例表行是 `instance_table_rows`。恢复报告与其余字段取自一对空盘上的
    /// 只读恢复（择不到系统配置，那几样都是 None）：C704 那一臂只看 R2 的根身份与实例表行。
    fn recovery_fields_read_comparing_a_version_with_these_instance_table_rows(
        root_identity: (InstanceGeneration, CheckpointTxg),
        instance_table_rows: Vec<InstanceRow>,
    ) -> RecoveryFieldsRead {
        let blank_pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], 4 << 30);
        let mut read = read_recovery_fields(&blank_pool, ComparedVersion::TheRecoveredVersion);
        let empty_pointer = singlefs_core::pointer::NodePointer::empty_root();
        read.compared_version = Some(crate::mount_transparency::VersionFieldsRead {
            root: singlefs_core::root_record::RootRecord {
                filesystem_identifier: [7u8; 16],
                unmount_marker:
                    singlefs_core::root_record::UnmountMarker::NotWrittenByTheUnmountSequence,
                instance: root_identity.0,
                checkpoint_txg: root_identity.1,
                tree_table: empty_pointer,
                tree_identifier_watermark: 0,
                rollback_floor: CheckpointTxg(0),
                instance_table: empty_pointer,
                mapping_root: empty_pointer,
                allocation_record_tree_root: empty_pointer,
                reclaim_threshold: singlefs_core::address::CheckpointTxg(0),
                pinned_ring_slot: singlefs_core::root_record::PinnedRingSlot::NO_SLOT_PINNED,
            },
            root_record_bytes: Vec::new(),
            content: crate::mount_transparency::PosixContentRead::NoFile,
            instance_table_rows: Some(instance_table_rows),
            tree_table_entries: None,
        });
        read
    }

    fn rows_deleted_by_the_sweep_of_the_c704_cases() -> RowsDeletedByTheInstanceTableSweepPublish {
        RowsDeletedByTheInstanceTableSweepPublish {
            sweep_publish: SWEEP_PUBLISH_OF_THE_C704_CASES,
            rows_deleted: vec![ROW_DELETED_BY_THE_SWEEP],
            heads_erased: 0,
        }
    }

    /// C704（挂载透明不认清扫发布）处置那一臂：R2 那一版是清扫发布那一版，它的实例表里却仍有清扫说删掉了的行 ⇒ 交回一条违例，
    /// 名字是 [`MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH`]，按字段分的判红计数里它加 1；
    /// R2 照原样交回、不记放回。判别力（`crates/mutations.tsv` 里点名这条的行）：被删的行还在也当成都不在、放回去再比时一条违例都没有；
    /// 判红不进计数时那一格是 None。
    #[test]
    fn sweep_publish_still_holding_the_rows_it_deleted_is_a_mount_transparency_violation_counted_once(
    ) {
        let read = recovery_fields_read_comparing_a_version_with_these_instance_table_rows(
            SWEEP_PUBLISH_OF_THE_C704_CASES,
            vec![ROW_KEPT_BY_THE_SWEEP, ROW_DELETED_BY_THE_SWEEP],
        );
        let rows_deleted = rows_deleted_by_the_sweep_of_the_c704_cases();
        let mut tally = CrashInjectionTally::default();
        let (compared, violations) = after_the_action_with_the_rows_deleted_by_the_sweep_put_back(
            &mut tally,
            &read,
            Some(&rows_deleted),
        );
        assert_eq!(
            violations.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
            [MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH],
            "清扫发布那一版里仍有它删掉的那一行：判红一条：{violations:?}"
        );
        assert_eq!(
            tally
                .mount_transparency_violations_by_field
                .get(MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH)
                .copied(),
            Some(1),
            "按字段分的判红计数里这一样加 1：{}",
            tally.render()
        );
        assert_eq!(
            tally.mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back, 0,
            "被删的行还在：不放回"
        );
        assert_eq!(*compared, read, "R2 照原样交回给逐字段判");
    }

    /// 对照：R2 那一版是清扫发布那一版，被删的行一条都不在 ⇒ 不判红，把它们放回去之后的 R2 交给逐字段判，放回计数加 1。
    /// 判别力（`crates/mutations.tsv` 里点名这条的行）：放回时不接上被删的行，交回的 R2 里只有留下的那一行。
    #[test]
    fn sweep_publish_without_the_rows_it_deleted_is_compared_with_them_put_back() {
        let read = recovery_fields_read_comparing_a_version_with_these_instance_table_rows(
            SWEEP_PUBLISH_OF_THE_C704_CASES,
            vec![ROW_KEPT_BY_THE_SWEEP],
        );
        let rows_deleted = rows_deleted_by_the_sweep_of_the_c704_cases();
        let mut tally = CrashInjectionTally::default();
        let (compared, violations) = after_the_action_with_the_rows_deleted_by_the_sweep_put_back(
            &mut tally,
            &read,
            Some(&rows_deleted),
        );
        assert!(
            violations.is_empty(),
            "被删的行都不在：不判红：{violations:?}"
        );
        assert!(
            tally.mount_transparency_violations_by_field.is_empty(),
            "按字段分的判红计数一样都没有：{}",
            tally.render()
        );
        assert_eq!(
            tally.mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back, 1,
            "放回一次"
        );
        assert_eq!(
            compared
                .compared_version
                .as_ref()
                .expect("R2 那一版在")
                .instance_table_rows,
            Some(vec![ROW_KEPT_BY_THE_SWEEP, ROW_DELETED_BY_THE_SWEEP]),
            "放回之后的 R2：留下的那一行在前，被删的接在后面"
        );
    }
}
