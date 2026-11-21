use super::*;

use singlefs_core::admission::SpaceAdmission;
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::allocator_after_make_filesystem;
use singlefs_core::mount::{
    mount_writable_with_space_admission, ParametersAndDeviceTableOfTheMount,
};
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::recovery::RecoveryReport;
use singlefs_core::transaction::{publish_without_units, ZeroUnitPublishPlan};
use singlefs_verification::crash_amplification::GivenStatesEngine;
use singlefs_verification::crash_given_states::GivenStatesOfATable;
use singlefs_verification::crash_injection::{inject_crashes_into_history, CrashPointDraw};
use singlefs_verification::fault_injection::{
    inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_given_states_engine,
    FaultCallCounts,
};
use singlefs_verification::history::{
    execute_history_handing_out_every_step_image, execute_history_with_faults,
    generate_history_with_weights, with_panic_capture, ContentChoice, ContentLength,
    FloorTargetChoice, GeneratedHistory, GenerationWeights, HistoryDevice, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
    PerStepChecker, RollbackTargetChoice, StepPosition,
};
use singlefs_verification::memory_pool::{writes_and_segments_with_stream_indexes, RetainedWrite};
use singlefs_verification::scenario::{
    first_file_content, FIXED_WRITE_TIME_SECONDS as WRITE_TIME_SECONDS_OF_THE_HARNESS_HISTORIES,
};
use singlefs_verification::{RecordingBlockDevice, SharedStream};

mod observed_device_and_random_ledger;
mod random_cells_and_branches;
mod random_executor;
mod random_fault_and_crash_families;
mod random_histories_and_source_checks;
mod rendering_tallies_and_today_points;

use self::observed_device_and_random_ledger::*;
use self::random_cells_and_branches::*;
use self::random_executor::*;
use self::random_fault_and_crash_families::*;
use self::random_histories_and_source_checks::*;
use self::rendering_tallies_and_today_points::*;

// ------------------------------------------------------------------------
// 13.9 结果行。
// ------------------------------------------------------------------------

fn optional_number_text(number: Option<u64>) -> String {
    number.map_or("none".to_string(), |value| value.to_string())
}

/// 模式 `r4-seg2` 的入口（开跑检查 `run_fourth_run_constants_and_anchors` 已在调用方过了）。
pub(super) fn run_segment_two(arm: FourthRunArm) {
    let report = investigator_report_text();
    let checks = random_history_source_checks(&report);
    let mut random_sources_hold = true;
    for one in &checks {
        let stops = !one.item.starts_with("5.6");
        if stops && !one.passed {
            random_sources_hold = false;
        }
        emit_result(&format!(
            "name=r4_segment_two_check arm={} item=\"{}\" verdict={} stops_the_family={} detail=\"{}\"",
            arm.name,
            one.item,
            if one.passed { "pass" } else { "fail" },
            stops,
            one.detail
        ));
    }
    let today_points = if arm.is_today() {
        TodayPoints::default()
    } else {
        match read_today_points() {
            Ok(points) => points,
            Err(error) => {
                emit_result(&format!(
                    "name=stop arm={} reason=\"今天那一臂的第二段产物读不出：{error}\"",
                    arm.name
                ));
                return;
            }
        }
    };
    emit_result(&format!(
        "name=r4_segment_two_start arm={} stop_at_the_first_checker_red={RANDOM_HISTORIES_STOP_AT_THE_FIRST_CHECKER_RED} fault_points_from_today={} crash_points_from_today={} stopped_families_from_today={:?}",
        arm.name,
        today_points.fault_points.len(),
        today_points.crash_points.values().map(Vec::len).sum::<usize>(),
        today_points.stopped_families
    ));
    // 逐步的池级 checker 与 harness 那一侧第一次判红，五族共用一个给定状态引擎（一份判定库，判过的镜像跨族、跨臂复用）
    let checker_engine = given_states_engine_of_this_experiment("e158_segment_two_checker_reading");
    for family in [
        SegmentTwoFamily::RandomRed,
        SegmentTwoFamily::RandomAll,
        SegmentTwoFamily::SeventhFault,
        SegmentTwoFamily::SeventhCrash,
        SegmentTwoFamily::EighthCrash,
    ] {
        if today_points.stopped_families.contains(family.name()) {
            emit_result(&format!(
                "name=r4_segment_two_family_skipped family={} arm={} reason=\"今天那一臂的产物里这一族停了\"",
                family.name(),
                arm.name
            ));
            continue;
        }
        let random_family = matches!(
            family,
            SegmentTwoFamily::RandomRed | SegmentTwoFamily::RandomAll
        );
        if random_family && !random_sources_hold {
            emit_result(&format!(
                "name=stop family={} arm={} clause=S9 reason=\"种子或写死的两条历史与调查员报告、测试源码对不上\"",
                family.name(),
                arm.name
            ));
            continue;
        }
        let run = match family {
            SegmentTwoFamily::RandomRed => {
                run_the_random_families(arm, family, random_red_histories(), &checker_engine)
            }
            SegmentTwoFamily::RandomAll => {
                run_the_random_families(arm, family, random_all_histories(), &checker_engine)
            }
            SegmentTwoFamily::SeventhFault => {
                run_the_seventh_fault_family(arm, &today_points, &checker_engine)
            }
            SegmentTwoFamily::SeventhCrash => run_a_crash_family(
                arm,
                family,
                &seventh_crash_history(),
                &today_points,
                &checker_engine,
            ),
            SegmentTwoFamily::EighthCrash => run_a_crash_family(
                arm,
                family,
                &eighth_crash_history(),
                &today_points,
                &checker_engine,
            ),
        };
        emit_family(family, run);
    }
}

#[cfg(test)]
mod tests;
