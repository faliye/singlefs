//! checker 档模块：crash_injection
//! 代码审阅第 2 条（崩溃注入层补可写挂载）：崩溃注入在每个崩溃状态上，只读恢复与判定照旧之后，
//! 在同一份崩溃后镜像上真的起一次可写挂载（取号、写行、暖机）、再发一次布、跑池级 checker；挂载途中再崩一次（二次崩溃），
//! 取号、写行、暖机、续做各段各摆一个，每个二次崩溃状态上只读恢复、问模型、池级 checker、记录核对器。
//! 实现在 `singlefs_checker_tier::crash_injection`；随机崩溃注入的快档与大档（`crash_injection_campaign.rs`）
//! 走的是同一个入口，这里用两段短历史与一段 unlink 的历史钉「每一截都跑到了、四段都摆到了、一条新发现都没有」，
//! 再加一段崩溃后那次可写挂载删得了实例表行的历史（有一行就扫）：清扫发布那一段的二次崩溃从它来；
//! 再加一段开头那次覆盖写掉电在 journal 记录之前、留下实例 1 的孤儿的历史（同一种摆法）：抹头那一段的二次崩溃从它来。

use singlefs_checker_tier::crash_injection::{
    inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row,
    inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans,
    inject_crashes_into_history, CrashInjectionTally, CrashPointDraw, WritableMountPhase,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    generate_history, GeneratedHistory, HistoryDeviceWidth, HistoryExecution, HistoryOperation,
    HistorySeed, HistoryStartingPoint, PerStepChecker,
};

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
    );
    tally.absorb(&unlinking_the_first_file.tally);
    new_findings.extend(unlinking_the_first_file.new_findings);
    let deleting_an_instance_table_row =
        inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row(
            UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        );
    tally.absorb(&deleting_an_instance_table_row.tally);
    new_findings.extend(deleting_an_instance_table_row.new_findings);
    let erasing_heads_of_orphans =
        inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans(
            UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
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
