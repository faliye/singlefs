//! 一条流的驱动：逐次调用实现、跟着推理想模型（O1-对拍），连同内存对象的摘要的单测。

use super::*;

/// 模型没有会话时问发布第一个文件：答案本身就是一格对不上，驱动把它记成 `Disagrees`、模型停在这次调用，下一次调用不比
/// （X5：`judge_and_advance` 的不同丢掉，这里红）。
#[test]
fn comparison_the_model_cannot_answer_is_recorded_as_a_disagreement_and_stops_the_model() {
    let mut driver = StreamDriver::new(
        StreamName::NewPoolFileCreation,
        SharedStream::retaining_contents(),
        0,
        RunSwitches::default(),
    );
    let refused: Result<(), &str> = Err("拒");
    assert!(driver.call("publish", unit_text, || refused).is_err());
    driver.judge(
        |model| model.answer_publish_first_file(b"x"),
        ObservedOutcome::Succeeded(ObservedEffect::Publishes {
            roots: Vec::new(),
            reported_ceiling: None,
        }),
    );
    assert!(matches!(
        driver.calls[0].model,
        ModelVerdictOfCall::Disagrees { .. }
    ));
    assert_eq!(driver.model_stopped_at_call, Some(0));
    let accepted: Result<(), &str> = Ok(());
    assert!(driver.call("publish", unit_text, || accepted).is_ok());
    driver.judge(
        |model| model.answer_publish_first_file(b"x"),
        ObservedOutcome::Succeeded(ObservedEffect::Publishes {
            roots: Vec::new(),
            reported_ceiling: None,
        }),
    );
    assert_eq!(
        driver.calls[1].model,
        ModelVerdictOfCall::SkippedAfterTheModelStopped
    );
}

/// 模型这一步要求成功、实现却交回一个说不出理由的错误：`judge_and_advance` 自己判出一格对不上，驱动记成 `Disagrees` 并让模型停在这次调用
/// （X5：`judge_and_advance` 的不同丢掉，这里红）。
#[test]
fn an_implementation_refusal_the_model_does_not_allow_is_recorded_as_a_disagreement() {
    let mut driver = StreamDriver::new(
        StreamName::NewPoolFileCreation,
        SharedStream::retaining_contents(),
        0,
        RunSwitches::default(),
    );
    driver.advance_the_model_past_the_acquire_and_the_warm_up();
    let refused: Result<(), &str> = Err("拒");
    assert!(driver.call("publish", unit_text, || refused).is_err());
    let observed = driver.refusal(
        &"拒",
        singlefs_verification::model::ObservedRefusalReason::Unexplained,
        0,
        0,
        None,
        None,
    );
    driver.judge(|model| model.answer_publish_first_file(b"x"), observed);
    assert!(
        matches!(driver.calls[0].model, ModelVerdictOfCall::Disagrees { .. }),
        "模型要求成功、实现拒了：{:?}",
        driver.calls[0].model
    );
    assert_eq!(driver.model_stopped_at_call, Some(0));
}

/// 取号交回错误：这条流不带故障，模型对取号只有「往前走」一种走法，驱动把它记成「模型说该成、实现拒了」，模型停在这次调用
/// （装置一侧变异：记红丢掉，这里红）。
#[test]
fn refused_acquire_on_a_stream_without_faults_is_recorded_as_a_comparison_red_and_stops_the_model()
{
    let mut driver = StreamDriver::new(
        StreamName::NewPoolFileCreation,
        SharedStream::retaining_contents(),
        0,
        RunSwitches::default(),
    );
    let refused_acquire: Result<(), RefusedForTheTest> = Err(
        RefusedForTheTest::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness,
    );
    let acquire_failure = driver
        .call("acquire", unit_text, || refused_acquire)
        .expect_err("取号被拒");
    driver.record_a_refusal_the_model_only_goes_forward_on(acquire_failure);
    assert_eq!(
        driver.calls[0].model,
        ModelVerdictOfCall::Disagrees {
            aspect: "模型说该成、实现拒了",
            model_answer: "往前走".to_string(),
            implementation_answer:
                "拒了:DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness"
                    .to_string(),
        }
    );
    assert_eq!(
        driver.calls[0].model.word(),
        "disagrees:模型说该成、实现拒了"
    );
    assert_eq!(driver.model_stopped_at_call, Some(0));
    // 暖机也一样：取号成功之后暖机被拒，记在第二次调用上，第一次调用不动。
    let mut after_a_good_acquire = StreamDriver::new(
        StreamName::NewPoolFileCreation,
        SharedStream::retaining_contents(),
        0,
        RunSwitches::default(),
    );
    let accepted: Result<(), RefusedForTheTest> = Ok(());
    assert!(after_a_good_acquire
        .call("acquire", unit_text, || accepted)
        .is_ok());
    let refused_warm_up: Result<(), RefusedForTheTest> = Err(
        RefusedForTheTest::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness,
    );
    let warm_up_failure = after_a_good_acquire
        .call("warm_up", unit_text, || refused_warm_up)
        .expect_err("暖机被拒");
    after_a_good_acquire.record_a_refusal_the_model_only_goes_forward_on(warm_up_failure);
    assert_eq!(
        after_a_good_acquire.calls[0].model,
        ModelVerdictOfCall::NotModelled
    );
    assert_eq!(
        after_a_good_acquire.calls[1].model.word(),
        "disagrees:模型说该成、实现拒了"
    );
    assert_eq!(after_a_good_acquire.model_stopped_at_call, Some(1));
}

/// 取号 panic：与交回错误同样截断（读法 5），结局记 `panic`、O1-对拍记「实现自崩、不比」（不红不绿），模型停在这次调用，
/// 这条流的结局是 panic 截断（照导出照判）；`record_a_refusal_the_model_only_goes_forward_on` 收到 `None` 不记红。
#[test]
fn panic_in_the_acquire_is_an_implementation_crash_and_not_a_comparison_red() {
    let mut driver = StreamDriver::new(
        StreamName::NewPoolFileCreation,
        SharedStream::retaining_contents(),
        0,
        RunSwitches::default(),
    );
    let failure = driver
        .call("acquire", unit_text, || -> Result<(), RefusedForTheTest> {
            panic!("取号自崩")
        })
        .expect_err("panic 交回 Err(None)");
    assert!(failure.is_none());
    driver.record_a_refusal_the_model_only_goes_forward_on(failure);
    assert_eq!(
        driver.calls[0].model,
        ModelVerdictOfCall::NotComparedBecauseTheImplementationCrashed
    );
    assert!(!driver.calls[0].model.was_judged_by_the_comparison());
    assert_eq!(driver.calls[0].outcome.calls_file_word(), "err");
    assert_eq!(driver.model_stopped_at_call, Some(0));
    assert_eq!(
        ending_of_the_driver(&driver),
        StreamEnding::StoppedByAPanic {
            call_index: 0,
            message: "取号自崩".to_string()
        }
    );
}

/// 关一条（Q3-核）关掉 O1 时每次调用都不比：被拒的取号也不记红，O1-对拍那一栏留在 `not_modelled`。
#[test]
fn with_the_model_switched_off_a_refused_acquire_is_not_a_comparison_red() {
    let mut driver = StreamDriver::new(
        StreamName::NewPoolFileCreation,
        SharedStream::retaining_contents(),
        0,
        RunSwitches {
            positive_control: None,
            disabled_oracle: Some(DisabledOracle::ComparisonAndCrashJudge),
            writable_mount_probe: false,
        },
    );
    let refused: Result<(), RefusedForTheTest> = Err(
        RefusedForTheTest::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness,
    );
    let failure = driver
        .call("acquire", unit_text, || refused)
        .expect_err("取号被拒");
    driver.record_a_refusal_the_model_only_goes_forward_on(failure);
    assert_eq!(driver.calls[0].model, ModelVerdictOfCall::NotModelled);
}

/// 内存对象五样、次序固定、各自是 `{:?}` 文本（V3：不记分配器那一项，这里红）。
#[test]
fn the_memory_objects_are_the_allocator_the_instance_the_warm_up_the_current_version_and_the_mount_output(
) {
    let allocator = PoolAllocator::new(vec![DeviceFreeMap::new(DeviceIdentity(0), 1 << 30)]);
    let objects = memory_objects_of(
        &allocator,
        Some(InstanceGeneration(2)),
        None,
        None,
        Some("挂载"),
    );
    assert_eq!(
        objects.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
        vec![
            "allocator",
            "instance",
            "warm_up_output",
            "current_version",
            "mount_output"
        ]
    );
    assert_eq!(objects[0].1, format!("{allocator:?}"));
    assert_eq!(objects[1].1, "Some(InstanceGeneration(2))");
    assert_eq!(objects[4].1, "挂载");
}
