//! K2-L1：同一份库上第二趟全部复用，跑完那一刻叫停文件在：驱动行报 stopped_early、states_reused=0。
//! 照 tests/common_crash_points 与 run_from_environment 的入口跑（单机 CPU、空的多机配置、库在草稿目录、标成临时）。
use k2proto::flows::{content_of, record_flow, repository_root};
use singlefs_checker_tier::crash::layer0_plan_state_count;
use singlefs_checker_tier::crash_amplification::{
    reuse_and_scale_fields_of_the_line, run_from_environment, CrashFlow, ExpandUpToWritesPerSegment,
    PipelineJudgingCode,
};
use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;

fn main() {
    let scratch = std::path::PathBuf::from(std::env::var_os("K2_SCRATCH").expect("要给 K2_SCRATCH"))
        .join(format!("k2-stopped-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).expect("建草稿");
    let configuration = scratch.join("multi-host.env");
    std::fs::write(&configuration, "").expect("空配置");
    let library = scratch.join("library");
    let stop_file = scratch.join("stop");
    for variable in [
        "SINGLEFS_CRASH_AMPLIFICATION_SHARE", "SINGLEFS_CRASH_AMPLIFICATION_IMPORT",
        "SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS", "SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY",
        "SINGLEFS_GPU_CARDS_CONFIG", "SINGLEFS_CRASH_AMPLIFICATION_VERIFY",
    ] {
        std::env::remove_var(variable);
    }
    std::env::set_var("SINGLEFS_MULTI_HOST_CONFIG", &configuration);
    std::env::set_var("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY", &library);
    std::env::set_var("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY", "1");
    std::env::set_var("SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE", &stop_file);

    let recorded = record_flow("k2-stopped", &content_of(11, 4100), true);
    let limit = ExpandUpToWritesPerSegment(6);
    let expansion = move |_segment_index: usize, segment: &[usize]| limit.expansion_of(segment);
    let flow = CrashFlow {
        base: &recorded.base,
        writes: &recorded.writes,
        segments: &recorded.segments,
        judged_root_index: recorded.judged_root_index,
        versions: &recorded.versions,
        expansion: &expansion,
        crash_points: recorded.crash_points.clone(),
    };
    let states = layer0_plan_state_count(&recorded.base, &recorded.writes, &recorded.segments, &expansion);
    assert!(states <= 1_000_000);
    let root = repository_root();
    let judging = PipelineJudgingCode::of_the_judges(&root, &Layer0ToolchainIdentity::of_the_cargo_running_this_test())
        .expect("判法");

    let first = run_from_environment(&flow, &judging, &root, limit).expect("第一趟");
    println!(
        "K2L1 name=first_run states_of_the_flow={states} stopped_early={} states_checked_here={} states_already_judged_at_recording={} fields={}",
        first.stopped_early, first.checking.states_checked, first.recording.states_already_judged,
        reuse_and_scale_fields_of_the_line(&first).trim()
    );
    let second = run_from_environment(&flow, &judging, &root, limit).expect("第二趟");
    println!(
        "K2L1 name=second_run_not_stopped stopped_early={} states_checked_here={} states_already_judged_at_recording={} fields={}",
        second.stopped_early, second.checking.states_checked, second.recording.states_already_judged,
        reuse_and_scale_fields_of_the_line(&second).trim()
    );
    std::fs::write(&stop_file, "").expect("放叫停文件");
    let third = run_from_environment(&flow, &judging, &root, limit).expect("第三趟");
    println!(
        "K2L1 name=third_run_stop_file_present stopped_early={} states_checked_here={} states_already_judged_at_recording={} unjudged_blocks={} fields={}",
        third.stopped_early, third.checking.states_checked, third.recording.states_already_judged,
        third.unjudged_blocks, reuse_and_scale_fields_of_the_line(&third).trim()
    );
    let _ = std::fs::remove_dir_all(&scratch);
}
