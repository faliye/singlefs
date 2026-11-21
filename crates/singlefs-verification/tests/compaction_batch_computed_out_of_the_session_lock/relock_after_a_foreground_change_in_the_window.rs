//! 窗口里有前台改动之后回锁：覆盖写 R 里的单元、别处写、建 inode、unlink 有单元在 R 里的文件之后，回锁搬的与锁里重算搬的相同；窗口里只换了隔离标记的被隔离单元在回锁时剔掉。

use super::*;

/// 起池、可写挂载、强制起跑、一批罩得住一整段，在单元区头一段（第一个文件的数据单元在里面，实例表钉住）签一条意图，再建一个 inode（「该跑了」，
/// 也是「写别处」那一格要写的另一个 inode）。
fn pool_with_a_live_intent_on_the_segment_of_the_first_data_unit(
    tag: &str,
) -> (BuiltPool, MountedSessionWithPendingChanges, Devices) {
    let mut pool = build_pool(tag);
    let (mut pending, mut devices) = session_with_pending_changes_of(&mut pool);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::ForcedToStartForTests,
        None,
        UNITS_OF_A_BATCH_COVERING_A_SEGMENT,
    );
    issue_an_intent_on(
        &mut pending,
        &mut devices,
        region_starting_at(SEGMENT_OF_THE_FIRST_DATA_UNIT),
    );
    create_one_inode(
        &mut pending,
        &mut devices,
        &CallerForTests::with_nothing_waiting(),
    );
    (pool, pending, devices)
}

/// 有活意图那一支回锁重排（D26（后台整理与放置回收） 已定项 13 第 5 条）：同一份起点两个池，一个取输入之后锁外期间做 `change`、回锁发这一批；
/// 一个先做 `change` 再在锁里推一批（在现行那一版上锁内重算）。两边搬走的单元相同、不发成「没发成」，回锁那一个池池级 checker 在 `judge` 上判不红，
/// 第一个文件读回相同。
fn the_relock_moves_the_units_a_recomputation_in_the_lock_moves_after(
    change: ForegroundChangeInTheWindow,
    tag: &str,
    judge: JudgeOfAPoolCheckerJudgement,
) {
    let caller = CallerForTests::with_nothing_waiting();
    let (pool_relocked, mut pending_relocked, mut devices_relocked) =
        pool_with_a_live_intent_on_the_segment_of_the_first_data_unit(&format!("{tag}-relocked"));
    let ticket = ticket_taken(&mut pending_relocked, &mut devices_relocked, &caller);
    make_the_foreground_change(
        change,
        &mut pending_relocked,
        &mut devices_relocked,
        &caller,
    );
    let computed = computed_reading_in_the_session(*ticket, &pending_relocked, &devices_relocked);
    let txg_before_the_relock = current_of(pending_relocked.session()).root.checkpoint_txg;
    let run_relocked = run_of(relocked(
        &mut pending_relocked,
        &mut devices_relocked,
        computed,
        &caller,
    ));
    if let Some(version_published_at_the_relock) = run_relocked.versions_published.first() {
        assert_eq!(
            version_published_at_the_relock.root.checkpoint_txg.0,
            txg_before_the_relock.0 + 1,
            "回锁发的那一版接在现行那一版后面（{change:?}）"
        );
    }
    let (_pool_recomputed, mut pending_recomputed, mut devices_recomputed) =
        pool_with_a_live_intent_on_the_segment_of_the_first_data_unit(&format!(
            "{tag}-recomputed-in-the-lock"
        ));
    make_the_foreground_change(
        change,
        &mut pending_recomputed,
        &mut devices_recomputed,
        &caller,
    );
    let run_recomputed = run_of(
        pending_recomputed
            .run_compaction_in_the_background_for_at_most_one_publish(
                &mut devices_recomputed,
                pacing_with_the_timer_fallback_off(),
                &caller,
            )
            .expect("盘表是挂载的那一份、现行那一版有树"),
    );
    let _stop_of_the_relock = stop_of(&run_relocked);
    let _stop_of_the_recomputation = stop_of(&run_recomputed);
    assert_eq!(
        mapping_keys_relocated_by(&run_relocked),
        mapping_keys_relocated_by(&run_recomputed),
        "回锁重排搬的单元与锁内在现行那一版上重算的相同（{change:?}）"
    );
    if let Some(violated) =
        invariants_violated_on(&pool_relocked, &format!("{tag}-relocked"), judge)
    {
        assert_eq!(
            violated,
            Vec::<&str>::new(),
            "回锁重排之后池级 checker 不红（{change:?}，{judge:?}）"
        );
    }
    let first_file_of = |pending: &MountedSessionWithPendingChanges, devices: &Devices| {
        current_of(pending.session())
            .inode_record
            .map(|_| first_file_read_back(pending, devices))
    };
    assert_eq!(
        first_file_of(&pending_relocked, &devices_relocked),
        first_file_of(&pending_recomputed, &devices_recomputed),
        "第一个文件读回相同（{change:?}）"
    );
}

#[test]
fn a_relock_after_an_overwrite_of_a_unit_in_the_region_moves_what_a_recomputation_in_the_lock_moves(
) {
    the_relock_moves_the_units_a_recomputation_in_the_lock_moves_after(
        ForegroundChangeInTheWindow::OverwriteOfTheFirstFile,
        "relock-after-an-overwrite-in-the-region",
        JudgeOfAPoolCheckerJudgement::JudgeOfTheMultiHostConfigurationOfTheCaller,
    );
}

#[test]
fn a_relock_after_an_overwrite_of_a_unit_in_the_region_moves_what_a_recomputation_in_the_lock_moves_on_the_cpu_judge(
) {
    the_relock_moves_the_units_a_recomputation_in_the_lock_moves_after(
        ForegroundChangeInTheWindow::OverwriteOfTheFirstFile,
        "relock-after-an-overwrite-in-the-region",
        JudgeOfAPoolCheckerJudgement::CpuJudgeUnderACpuOnlyMultiHostConfiguration,
    );
}

#[test]
fn a_relock_after_a_write_elsewhere_moves_what_a_recomputation_in_the_lock_moves() {
    the_relock_moves_the_units_a_recomputation_in_the_lock_moves_after(
        ForegroundChangeInTheWindow::WriteOfAnotherInode,
        "relock-after-a-write-elsewhere",
        JudgeOfAPoolCheckerJudgement::JudgeOfTheMultiHostConfigurationOfTheCaller,
    );
}

#[test]
fn a_relock_after_a_write_elsewhere_moves_what_a_recomputation_in_the_lock_moves_on_the_cpu_judge()
{
    the_relock_moves_the_units_a_recomputation_in_the_lock_moves_after(
        ForegroundChangeInTheWindow::WriteOfAnotherInode,
        "relock-after-a-write-elsewhere",
        JudgeOfAPoolCheckerJudgement::CpuJudgeUnderACpuOnlyMultiHostConfiguration,
    );
}

#[test]
fn a_relock_after_a_new_inode_moves_what_a_recomputation_in_the_lock_moves() {
    the_relock_moves_the_units_a_recomputation_in_the_lock_moves_after(
        ForegroundChangeInTheWindow::NewInode,
        "relock-after-a-new-inode",
        JudgeOfAPoolCheckerJudgement::JudgeOfTheMultiHostConfigurationOfTheCaller,
    );
}

#[test]
fn a_relock_after_a_new_inode_moves_what_a_recomputation_in_the_lock_moves_on_the_cpu_judge() {
    the_relock_moves_the_units_a_recomputation_in_the_lock_moves_after(
        ForegroundChangeInTheWindow::NewInode,
        "relock-after-a-new-inode",
        JudgeOfAPoolCheckerJudgement::CpuJudgeUnderACpuOnlyMultiHostConfiguration,
    );
}

#[test]
fn a_relock_after_an_unlink_of_a_file_with_a_unit_in_the_region_moves_what_a_recomputation_in_the_lock_moves(
) {
    the_relock_moves_the_units_a_recomputation_in_the_lock_moves_after(
        ForegroundChangeInTheWindow::UnlinkOfTheFirstFile,
        "relock-after-an-unlink-with-a-unit-in-the-region",
        JudgeOfAPoolCheckerJudgement::JudgeOfTheMultiHostConfigurationOfTheCaller,
    );
}

#[test]
fn a_relock_after_an_unlink_of_a_file_with_a_unit_in_the_region_moves_what_a_recomputation_in_the_lock_moves_on_the_cpu_judge(
) {
    the_relock_moves_the_units_a_recomputation_in_the_lock_moves_after(
        ForegroundChangeInTheWindow::UnlinkOfTheFirstFile,
        "relock-after-an-unlink-with-a-unit-in-the-region",
        JudgeOfAPoolCheckerJudgement::CpuJudgeUnderACpuOnlyMultiHostConfiguration,
    );
}

/// 隔离记号那一格：锁外期间覆盖写第一个文件，它在 R 里的数据单元第二块盘上那一份被改坏，释放之前读盘核没过——两块盘的记录留在已分配、分配代不变，
/// 只多了隔离记号。回锁时这个单元剔掉（计划里只有它，剔完一个不剩，这一批不发），会话的常驻隔离集重取进它的槽。
/// 池级 checker 那一判：覆盖写之前那一版的根还在回退候选集里、引用着改坏的那一份，回锁之前就判红 I-2.1、I-4.8、I-7.4 三条
/// （改坏是用例注入的，I-2.1 点名的正是第二块盘上那一份）；回锁之后判红的还是这三条、一条不多。池级 checker 在 `judge` 上判，
/// I-2.1 的细节只在 CPU 判器给的那一份上核。
fn the_unit_quarantined_in_the_window_with_only_its_quarantine_mark_changed_is_pruned_at_the_relock_judged_by(
    judge: JudgeOfAPoolCheckerJudgement,
) {
    let caller = CallerForTests::with_nothing_waiting();
    let (pool, mut pending, mut devices) =
        pool_with_a_live_intent_on_the_segment_of_the_first_data_unit(
            "relock-prunes-a-unit-quarantined-in-the-window",
        );
    let data_slot = current_of(pending.session()).data_pointers[0].locations[0].slot;
    assert_eq!(
        data_slot.0 / 64 * 64,
        SEGMENT_OF_THE_FIRST_DATA_UNIT,
        "前提：第一个文件的数据单元在 R 里"
    );
    let ticket = ticket_taken(&mut pending, &mut devices, &caller);
    corrupt_the_copy_on(&mut devices, DeviceIdentity(1), data_slot);
    make_the_foreground_change(
        ForegroundChangeInTheWindow::OverwriteOfTheFirstFile,
        &mut pending,
        &mut devices,
        &caller,
    );
    let records_of_the_data_slot: Vec<_> = current_of(pending.session())
        .allocation_records
        .iter()
        .filter(|record| record.slot == data_slot)
        .copied()
        .collect();
    assert!(
        records_of_the_data_slot.len() == 2
            && records_of_the_data_slot
                .iter()
                .all(|record| !record.is_released
                    && record.quarantine_mark
                        == AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck),
        "前提：两块盘的记录留在已分配、带隔离记号：{records_of_the_data_slot:?}"
    );
    let copy_corrupted = format!("盘 1 槽 {} 的那一份", data_slot.0);
    let violated_before_the_relock = violations_on(
        &pool,
        "relock-prunes-a-unit-quarantined-in-the-window-before-the-relock",
        judge,
    );
    if let Some(violated_before_the_relock) = &violated_before_the_relock {
        assert_eq!(
            violated_before_the_relock
                .iter()
                .map(|(name, _)| *name)
                .collect::<Vec<_>>(),
            INVARIANTS_RED_ON_THE_COPY_CORRUPTED_UNDER_A_CANDIDATE_ROOT,
            "前提：回锁之前判红的只有改坏的那一份牵出的三条（{judge:?}）：{violated_before_the_relock:?}"
        );
        // 违例细节只有 CPU 判器给：只开 CPU 的那一判照原样核细节；照调用方配置的那一判由 GPU 判器判时只核违例位（上一句已核 I-2.1 红）
        let detail_names_the_copy_corrupted = |detail: &Option<String>| match judge {
            JudgeOfAPoolCheckerJudgement::CpuJudgeUnderACpuOnlyMultiHostConfiguration => {
                detail.as_ref().is_some_and(|detail_of_the_cpu_judge| {
                    detail_of_the_cpu_judge.contains(&copy_corrupted)
                })
            }
            JudgeOfAPoolCheckerJudgement::JudgeOfTheMultiHostConfigurationOfTheCaller => {
                detail.as_ref().is_none_or(|detail_of_the_cpu_judge| {
                    detail_of_the_cpu_judge.contains(&copy_corrupted)
                })
            }
        };
        assert!(
            violated_before_the_relock
                .iter()
                .any(|(name, detail)| *name == "I-2.1" && detail_names_the_copy_corrupted(detail)),
            "前提：I-2.1 点名的是改坏的那一份（{copy_corrupted}，{judge:?}）：{violated_before_the_relock:?}"
        );
    }
    let computed = computed_reading_in_the_session(*ticket, &pending, &devices);
    let run = run_of(relocked(&mut pending, &mut devices, computed, &caller));
    assert_eq!(
        stop_of(&run),
        CompactionRunStop::BatchPrunedToNothingAtTheRelock { units_pruned: 1 },
        "只多了隔离记号的那个单元剔掉，剔完一个不剩不发"
    );
    assert!(run.versions_published.is_empty(), "一个写都没发");
    assert!(
        pending
            .session()
            .compaction
            .quarantined_slots()
            .contains(&data_slot),
        "回锁时重取了隔离集：那个槽进了会话的常驻隔离集"
    );
    assert!(
        pending
            .session()
            .compaction
            .has_a_background_trip_under_way(),
        "剔完不发的这一趟还在途"
    );
    if let (Some(violated_before_the_relock), Some(violated_after_the_relock)) = (
        violated_before_the_relock,
        violations_on(
            &pool,
            "relock-prunes-a-unit-quarantined-in-the-window-after-the-relock",
            judge,
        ),
    ) {
        assert_eq!(
            violated_after_the_relock, violated_before_the_relock,
            "回锁之后池级 checker 判红的与回锁之前相同（{judge:?}）"
        );
    }
}

/// 照调用方的多机配置判池级 checker（配了 GPU 而这个进程拿不到卡记「本次未跑」；GPU 判器判的只核违例位）。
#[test]
fn a_unit_quarantined_in_the_window_with_only_its_quarantine_mark_changed_is_pruned_at_the_relock()
{
    the_unit_quarantined_in_the_window_with_only_its_quarantine_mark_changed_is_pruned_at_the_relock_judged_by(
        JudgeOfAPoolCheckerJudgement::JudgeOfTheMultiHostConfigurationOfTheCaller,
    );
}

/// 在只开 CPU 的多机配置下由 CPU 判器判池级 checker（不打「本次未跑」，I-2.1 的细节照原样核）。
#[test]
fn a_unit_quarantined_in_the_window_with_only_its_quarantine_mark_changed_is_pruned_at_the_relock_on_the_cpu_judge(
) {
    the_unit_quarantined_in_the_window_with_only_its_quarantine_mark_changed_is_pruned_at_the_relock_judged_by(
        JudgeOfAPoolCheckerJudgement::CpuJudgeUnderACpuOnlyMultiHostConfiguration,
    );
}
