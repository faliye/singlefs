//! 代码审阅第 2 条（用户 2026-09-27 定案「崩溃注入层补可写挂载」）：崩溃注入在每个崩溃状态上，只读恢复与判定照旧之后，
//! 在同一份崩溃后镜像上真的起一次可写挂载（取号、写行、暖机）、再发一次布、跑池级 checker；挂载途中再崩一次（二次崩溃），
//! 取号、写行、暖机三段各摆一个，每个二次崩溃状态上只读恢复、问模型、池级 checker、记录核对器。
//! 实现在 `singlefs_checker_tier::crash_injection`；随机崩溃注入的快档与大档（`second_transaction_supplement_three_crash_injection.rs`）
//! 走的是同一个入口，这里用两段短历史钉「每一截都跑到了、三段都摆到了、一条新发现都没有」。

use singlefs_checker_tier::crash_injection::{
    inject_crashes_into_history, CrashInjectionTally, CrashPointDraw, WritableMountPhase,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    generate_history, HistoryDeviceWidth, HistoryExecution, HistorySeed, PerStepChecker,
};

/// 历史本身不跑每一步的 checker（活盘面由随机历史那一路罩着），两块 4 GiB 的盘：单元区墙在这里够不着，挂载与之后那一次发布都该成。
const UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::FourGibibytes,
    space_admission: SpaceAdmission::JudgedByTheFormula,
};

/// 两段短历史、每段摆两个崩溃状态：每个崩溃状态之后都起了可写挂载、发了一次布、跑了 checker，
/// 挂载途中的二次崩溃取号、写行、暖机三段都摆到了，二次崩溃状态上恢复、问模型、checker、记录核对器各跑了一次；
/// 两截上一条新发现都没有，挂载与那次发布一次都没被拒。
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
    let rendered = tally.render();
    println!("{rendered}");
    assert!(
        tally.crash_points >= 2,
        "两段历史里摆出了崩溃状态：{rendered}"
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
