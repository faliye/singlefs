//! checker 档模块：crash_point_replay_progress、crash_injection、crash_given_states、crash_amplification
//! 代码审阅第 2 条（崩溃注入层补可写挂载）：崩溃注入在每个崩溃状态上，只读恢复与判定照旧之后，
//! 在同一份崩溃后镜像上真的起一次可写挂载（取号、写行、暖机）、再发一次布、跑池级 checker；挂载途中再崩一次（二次崩溃），
//! 取号、写行、暖机、续做各段各摆一个，每个二次崩溃状态上只读恢复、问模型、池级 checker、记录核对器。
//! 实现在 `singlefs_checker_tier::crash_injection`；随机崩溃注入的快档与大档（`crash_injection_campaign.rs`）
//! 走的是同一个入口，这里用两段短历史与一段 unlink 的历史钉「每一截都跑到了、四段都摆到了、一条新发现都没有」，
//! 再加一段崩溃后那次可写挂载删得了实例表行的历史（有一行就扫）：清扫发布那一段的二次崩溃从它来；
//! 再加一段开头那次覆盖写掉电在 journal 记录之前、留下实例 1 的孤儿的历史（同一种摆法）：抹头那一段的二次崩溃从它来。

use singlefs_checker_tier::crash_amplification::{GivenStatesEngine, GivenStatesItem};
use singlefs_checker_tier::crash_injection::{
    inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row,
    inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans,
    inject_crashes_into_history, CrashInjectionTally, CrashPointDraw,
    GivenStatesAfterTheFirstCrash, HistoryAfterTheMakeFilesystem,
    StateOfTheTableAfterTheWritableMount, WritableMountPhase,
};
use singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_harness::history::{
    execute_history_with, generate_history, ContentChoice, ContentLength, GeneratedHistory,
    HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed,
    HistoryStartingPoint, PerStepChecker,
};
use singlefs_harness::memory_pool::{
    publishes_in, writes_and_segments_with_stream_indexes, CrashImage, RecordStreamContinuity,
    RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 给定状态表交崩溃点重放的引擎判：表与结论进家目录下给定状态表那一类的判定库（判过的复用），配了 GPU 且编进了 `gpu` 特性上卡、否则 CPU。
fn given_states_engine(seeds_and_scale: &str) -> GivenStatesEngine {
    GivenStatesEngine::start_from_environment(
        GivenStatesItem {
            item_name: format!(
                "crash-injection-writable-mount-after-the-crash::{}",
                seeds_and_scale
            ),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
}

/// 历史本身不跑每一步的 checker（活盘面由随机历史那一路罩着），两块 4 GiB 的盘：单元区墙在这里够不着，挂载与之后那一次发布都该成。
const UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::FourGibibytes,
    space_admission: SpaceAdmission::JudgedByTheFormula,
};

/// 两段短历史、每段摆两个崩溃状态，再加第一个文件之后经挂着的会话 unlink 它的一段（意图那次 + 最后一批，短段整段枚举：
/// 崩溃状态有落在意图那一版上的，那一版上的可写挂载续做，U4，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：
/// 每个崩溃状态之后都起了可写挂载、发了一次布（删完那一版上建 inode）、跑了 checker，
/// 挂载途中的二次崩溃取号、写行、暖机、续做四段都摆到了，二次崩溃状态上恢复、问模型、checker、记录核对器各跑了一次；
/// 两截上一条新发现都没有，挂载与那次发布一次都没被拒。第四段在一段覆盖写到根环里只剩实例 2 的根的历史最后一步开头崩一次，
/// 那次可写挂载与二次崩溃之后的再挂有一行就扫（实例 1 那一行删得掉）：清扫发布那一段摆到二次崩溃。第五段同一种摆法，那段历史开头那次
/// 覆盖写掉电在 journal 记录之前、留下实例 1 的孤儿：清扫先抹它们的头，抹头那一段摆到二次崩溃。
#[test]
fn every_crash_state_is_followed_by_a_writable_mount_one_publish_the_checker_and_second_crashes_in_each_phase(
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
            &given_states_engine("writable_mount_after_the_crash"),
        );
        tally.absorb(&injection.tally);
        new_findings.extend(injection.new_findings);
    }
    let unlinking_the_first_file = inject_crashes_into_history(
        &GeneratedHistory {
            seed: HistorySeed(3),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: vec![HistoryOperation::Unlink],
        },
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::EveryProperSubsetOfShortSegments {
            segment_writes: 2,
            sampled_in_longer_segments: 2,
        },
        None,
        &given_states_engine("writable_mount_after_the_crash"),
    );
    tally.absorb(&unlinking_the_first_file.tally);
    new_findings.extend(unlinking_the_first_file.new_findings);
    let deleting_an_instance_table_row =
        inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row(
            UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            &given_states_engine("writable_mount_after_the_crash"),
        );
    tally.absorb(&deleting_an_instance_table_row.tally);
    new_findings.extend(deleting_an_instance_table_row.new_findings);
    let erasing_heads_of_orphans =
        inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans(
            UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            &given_states_engine("writable_mount_after_the_crash"),
        );
    tally.absorb(&erasing_heads_of_orphans.tally);
    new_findings.extend(erasing_heads_of_orphans.new_findings);
    let rendered = tally.render();
    println!("{rendered}");
    assert!(
        tally.crash_points >= 2,
        "五段历史里摆出了崩溃状态：{rendered}"
    );
    assert!(
        tally.writable_mounts_after_the_crash_deleting_instance_table_rows >= 1,
        "前提：有一次崩溃后的可写挂载末尾的清扫发布删了行：{rendered}"
    );
    assert!(
        tally.heads_erased_by_the_instance_table_sweep_after_the_crash >= 1,
        "前提：有一次崩溃后的可写挂载末尾的清扫抹了孤儿的头：{rendered}"
    );
    assert_eq!(
        tally.writable_mounts_after_the_crash, tally.crash_points,
        "每个崩溃状态之后都起一次可写挂载：{rendered}"
    );
    assert_eq!(
        tally.writable_mounts_after_the_crash_succeeded, tally.crash_points,
        "两块 4 GiB 的盘上崩溃之后的可写挂载都该成：{rendered}"
    );
    assert_eq!(
        tally.publishes_after_the_writable_mount, tally.crash_points,
        "每次挂载之后都发成一次布：{rendered}"
    );
    assert_eq!(
        tally.checker_runs_after_the_writable_mount, tally.crash_points,
        "每次挂载与发布之后都跑一次 checker：{rendered}"
    );
    for phase in WritableMountPhase::ALL {
        assert!(
            tally
                .second_crash_points_by_phase
                .get(&phase)
                .copied()
                .unwrap_or(0)
                >= 1,
            "挂载途中的二次崩溃一次都没崩在 {}：{rendered}",
            phase.name()
        );
    }
    assert_eq!(
        tally.second_crash_points,
        tally.second_crash_points_by_phase.values().sum::<u64>(),
        "二次崩溃按段分的计数加起来是总数：{rendered}"
    );
    for (what, count) in [
        ("问模型", tally.second_crash_model_judgements),
        ("池级 checker", tally.second_crash_checker_runs),
        ("记录核对器", tally.second_crash_record_checks),
        (
            "恢复",
            tally.second_crash_recoveries_reading_a_file
                + tally.second_crash_recoveries_without_a_file
                + tally.second_crash_recoveries_failed,
        ),
    ] {
        assert_eq!(
            count, tally.second_crash_points,
            "每个二次崩溃状态上都跑一次{what}：{rendered}"
        );
    }
    assert_eq!(
        tally.second_crash_recoveries_failed, 0,
        "二次崩溃之后恢复失败过：{rendered}"
    );
    assert!(
        new_findings.is_empty(),
        "崩溃后可写挂载与二次崩溃上的新发现：{}",
        new_findings
            .iter()
            .map(singlefs_checker_tier::crash_injection::CrashPointFinding::render)
            .collect::<String>()
    );
}

/// 第二截经库（用户 2026-10-13 定「能走两段的都走两段」）：每个崩溃状态上可写挂载、再发一次布之后的盘与第一次崩溃那份镜像当一张表的
/// 两个给定状态交判器判（池级 checker 与记录核对器，记录核对器两份镜像各判一条，D13（验证路线） 已定项 7；判器是崩溃点重放的引擎时
/// 表先写进库、从库里读回判，配了 GPU 走 GPU），一个崩溃状态一张 `after-the-mount` 表，挂完的盘不在 CPU 上就地判。判器换成数表的替身
/// （照 CPU 判器判）：替身看到的 `after-the-mount` 表数等于崩溃状态数、每张两个状态，第二截的记录核对器次数也等于崩溃状态数。
/// 判别力（`crates/mutations.tsv`）：第二截改回在内存里就地判、不交判器，这条红。
// crash-case-check:not-a-crash-case 一段 8 步的短历史只摆两个崩溃状态，只数判器看到的表，单跑几秒
#[test]
fn every_crash_state_hands_the_disks_after_the_mount_and_its_publish_to_the_given_states_judge() {
    use singlefs_checker_tier::crash_injection::{
        judge_the_given_states_on_the_cpu, GivenStatesConclusionsOfATable, GivenStatesJudge,
        GivenStatesOfATable,
    };
    /// 数它判过哪几张表（表名、状态数），照 CPU 判器判。
    struct TablesSeenByTheJudge(std::sync::Mutex<Vec<(String, usize)>>);
    impl GivenStatesJudge for TablesSeenByTheJudge {
        fn stop_is_requested(&self) -> bool {
            false
        }
        fn judge_the_given_states(
            &self,
            table: &GivenStatesOfATable<'_>,
        ) -> Result<GivenStatesConclusionsOfATable, String> {
            self.0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push((table.table_name.clone(), table.persisted_sets.len()));
            Ok(judge_the_given_states_on_the_cpu(table))
        }
    }
    let judge = TablesSeenByTheJudge(std::sync::Mutex::new(Vec::new()));
    let injection = inject_crashes_into_history(
        &generate_history(HistorySeed(3), 8),
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::Sampled {
            crash_points_per_history: 2,
        },
        None,
        &judge,
    );
    let tables = judge
        .0
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let after_the_mount: Vec<&(String, usize)> = tables
        .iter()
        .filter(|(table_name, _)| table_name.contains("/after-the-mount/"))
        .collect();
    assert!(
        injection.tally.crash_points >= 1,
        "前提：这段历史摆出了崩溃状态：{}",
        injection.tally.render()
    );
    assert_eq!(
        u64::try_from(after_the_mount.len()).expect("装得进 u64"),
        injection.tally.crash_points,
        "每个崩溃状态一张挂载之后的给定状态表：{tables:?}"
    );
    assert!(
        after_the_mount
            .iter()
            .all(|(_, states)| *states == StateOfTheTableAfterTheWritableMount::ALL.len()),
        "挂载之后那张表恰好两个状态（挂载与之后那一次发布全落的池、第一次崩溃那份镜像）：{after_the_mount:?}"
    );
    assert_eq!(
        injection.tally.record_checks_after_the_writable_mount, injection.tally.crash_points,
        "第二截的记录核对器每个崩溃状态一次"
    );
}

fn overwrite(fill_seed: u64) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 2999 },
        fill_seed,
    })
}

/// 第二截的记录核对器读两份镜像（D13（验证路线） 已定项 7「崩溃后镜像是两份」，欠账 C832，用户 2026-10-13 定「过渡后补回两份镜像」）：
/// 第一条判据（根在而记录一条都不在）读第一次崩溃那份镜像，第二条读挂载之后的池。手摆一个第一次崩溃状态：第一个文件之后覆盖写两次，
/// 最后一次带 journal 记录的那次发布，根槽写落了、记录写一个都没落；挂载那一段只有一个写，把那次发布的根槽盖成同一块盘上更早写的一条根
/// （历史那次发布的根槽被挂载的写覆盖）。只读挂载之后的池时那条根已不在，第一条判据放过；第一次崩溃那份镜像上根在、记录一条都不在，
/// 读两份镜像的第二截判红。表交崩溃点重放的引擎判（两段：表先写进判定库、读回再判，配了 GPU 走 GPU）。
/// 判别力（`crates/mutations.tsv`）：第二截第一条判据改回读挂载之后的池那个状态，这条红。
// crash-case-check:not-a-crash-case 一段两步的短历史上手摆一个第一次崩溃状态，不枚举崩溃状态
#[test]
fn the_record_checker_after_the_writable_mount_finds_a_history_root_overwritten_by_the_mount_on_the_first_crash_image(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![overwrite(1), overwrite(2)],
    };
    let execution = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let run = execute_history_with(&history, execution, &stream, &mut |_| {});
    assert_eq!(run.ending, HistoryEnding::Completed, "前提：这段历史跑完");
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
        &writes,
        &segments,
        &stream_indexes,
        run.operations_written_by_make_filesystem,
        execution,
    );
    let every_write = vec![true; writes.len()];
    let last_publish_with_records =
        publishes_in(&writes, &every_write, RecordStreamContinuity::OneRecording)
            .into_iter()
            .rev()
            .find(|publish| !publish.records.is_empty())
            .expect("前提：覆盖写那几次发布写了 journal 记录");
    let overwritten_root = &writes[last_publish_with_records.root];
    let earlier_root_on_the_same_device = writes[..last_publish_with_records.root]
        .iter()
        .rev()
        .find(|write| {
            write.kind == StepKind::RootRecordFua
                && write.device == overwritten_root.device
                && write.length_in_bytes() == overwritten_root.length_in_bytes()
        })
        .expect("前提：那次发布之前同一块盘上写过别的根");
    let mut persisted = every_write;
    for record in &last_publish_with_records.records {
        persisted[*record] = false;
    }
    let mount_writes = vec![RetainedWrite {
        device: overwritten_root.device,
        kind: StepKind::RootRecordFua,
        is_force_unit_access: true,
        offset: overwritten_root.offset,
        contents: earlier_root_on_the_same_device.contents.clone(),
    }];
    let mount_segments = vec![vec![0]];
    let version_landed_on_after_the_first_crash = recover(
        &CrashImage {
            base: history_after_the_make_filesystem.base_pool(),
            writes: history_after_the_make_filesystem.writes(),
            persisted: history_after_the_make_filesystem.persisted_set_of_the_table(&persisted),
        },
        JournalPolicy::Consult,
    )
    .effective_root;
    let engine = given_states_engine("writable_mount_after_the_crash");
    let given = GivenStatesAfterTheFirstCrash::new(
        &engine,
        &history_after_the_make_filesystem,
        &persisted,
        &[],
        version_landed_on_after_the_first_crash,
        &[],
        &[],
    );
    let conclusions = given.judge_the_pool_after_the_writable_mount(
        &mount_writes,
        &mount_segments,
        "first-file-and-two-overwrites/root-of-the-last-publish-with-records-overwritten-by-the-mount"
            .to_string(),
    );
    {
        use singlefs_checker_tier::crash_given_states::GivenStatesConclusionsSource;
        if conclusions.source == GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly {
            // 配置开了 GPU 而这份二进制没编进 gpu 特性：引擎只录不判、交占位结论（开跑时已打本次未跑那一行），判归带特性那一趟
            return;
        }
    }
    assert!(
        !conclusions
            .on_the_pool_after_the_mount
            .record_check()
            .root_without_record,
        "只读挂载之后的池：那次发布的根槽被挂载的写盖掉，第一条判据放过"
    );
    assert!(
        conclusions
            .on_the_first_crash_image
            .record_check()
            .root_without_record,
        "第一次崩溃那份镜像上那次发布的根在、记录一条都不在"
    );
    assert!(
        conclusions.record_check().root_without_record,
        "读两份镜像：第二截的第一条判据读第一次崩溃那份镜像，判红"
    );
}
