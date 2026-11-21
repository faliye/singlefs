//! 跑着的流与备好的流：两条流从建池到收尾的驱动，与流的形状回比 crates的单测。

use super::*;

/// 两条流驱动完，`baseline` 回比的段长表、写数、根槽写数、盘数与盘字节数（`prepare_stream` 那一批 S1）逐条成立：
/// 本地表与现在录到的流分叉时这里红，用不着跑一趟 `baseline`。闭式与写数都对而段的次序不对的表只有这条抓得到。
#[test]
fn stop_checks_of_both_streams_hold_against_the_streams_recorded_now() {
    for name in STREAM_NAMES {
        let stream = prepare_stream(name, RunSwitches::default(), true);
        assert_eq!(
            stream.ending,
            StreamEnding::Completed,
            "{} 没有驱动到头",
            name.directory_name()
        );
        let broken: Vec<String> = stream
            .checks
            .iter()
            .filter(|check| !check.holds())
            .map(|check| {
                format!(
                    "{} local={} crates={}",
                    check.name, check.local, check.crates
                )
            })
            .collect();
        assert!(
            broken.is_empty(),
            "{} 的 S1 没过：{broken:?}",
            name.directory_name()
        );
    }
}

/// 真池上走一遍取号、暖机：O1-对拍那一栏没有红。照攻方 K4-c 的造法（`crates/singlefs-core/src/transaction.rs` 取号逐盘核 `<` 写成 `<=`，
/// 不带故障的盘上每次取号都拒），核心一改这一条就红，因为装置把被拒的取号记成了 O1-对拍的红；装置不记红（回到修之前）时这一条不红。
/// 只断言对拍一栏、不断言取号成功（断言取号成功的是下一条，那一条在装置不记红时也红，分不出装置记没记）。
#[test]
fn an_acquire_and_a_warm_up_on_a_fresh_pool_leave_the_comparison_column_without_a_red() {
    let mut live = LiveStream::start(
        StreamName::NewPoolFileCreation,
        "e171-unit-acquire-and-warm-up-comparison",
        RunSwitches::default(),
    );
    let _ = live.step_acquire_and_warm_up();
    let calls_red_in_the_comparison: Vec<&str> = live
        .driver
        .calls
        .iter()
        .filter(|call| matches!(call.model, ModelVerdictOfCall::Disagrees { .. }))
        .map(|call| call.label.as_str())
        .collect();
    assert!(
        calls_red_in_the_comparison.is_empty(),
        "取号或暖机被拒、O1-对拍记了红：{calls_red_in_the_comparison:?}"
    );
}

/// 真池上取号、暖机的绝对值：两次调用都成功、模型都不比（这一步模型不交答）、模型没停、第一次取到的号是 1。
#[test]
fn an_acquire_and_a_warm_up_on_a_fresh_pool_succeed_and_the_model_is_not_asked() {
    let mut live = LiveStream::start(
        StreamName::NewPoolFileCreation,
        "e171-unit-acquire-and-warm-up-absolute",
        RunSwitches::default(),
    );
    assert!(live.step_acquire_and_warm_up());
    let labels: Vec<&str> = live
        .driver
        .calls
        .iter()
        .map(|call| call.label.as_str())
        .collect();
    assert_eq!(labels, vec!["acquire", "warm_up"]);
    assert!(live
        .driver
        .calls
        .iter()
        .all(|call| call.outcome == CallOutcome::Succeeded
            && call.model == ModelVerdictOfCall::NotModelled));
    assert_eq!(live.driver.model_stopped_at_call, None);
    assert_eq!(live.instance, Some(InstanceGeneration(1)));
}
