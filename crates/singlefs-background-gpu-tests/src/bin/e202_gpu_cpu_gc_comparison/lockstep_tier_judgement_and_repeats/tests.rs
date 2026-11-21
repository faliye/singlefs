use super::*;

#[test]
fn the_numbers_of_a_run_line_read_back_as_written() {
    let line = "E202 name=run tier=4GiB arm=gpu_one_card_rejudged repeat=3 session=2 batches=144 gc_wall_nanoseconds=900 \
                    process_cpu_nanoseconds=700 inner_rejudge_cpu_nanoseconds=50 void=no";
    let (name, fields) = fields_of_a_result_line(line).expect("是本装置的结果行");
    assert_eq!(name, "run");
    assert_eq!(
        RunNumbers::of_the_fields(&fields),
        Some(RunNumbers {
            arm: LOCKSTEP_ARMS[4],
            repeat: 3,
            batches: 144,
            gc_wall_nanoseconds: 900.0,
            process_cpu_nanoseconds: 700.0,
            inner_rejudge_cpu_nanoseconds: 50.0,
        })
    );
    assert_eq!(fields_of_a_result_line("E185 name=run"), None);
}

#[test]
fn the_rotation_of_a_repeat_starts_at_the_arm_with_the_repeat_number() {
    let rotation: Vec<String> = arms_in_the_rotation_of_the_repeat(&LOCKSTEP_ARMS, 2)
        .iter()
        .map(|arm| arm.name())
        .collect();
    assert_eq!(
        rotation,
        [
            "gpu_every_card",
            "cpu_rejudged",
            "gpu_one_card_rejudged",
            "gpu_every_card_rejudged",
            "cpu",
            "gpu_one_card"
        ]
    );
}

/// 前提 N：批数都相等且不少于 64 才够。
#[test]
fn batches_equal_and_at_least_sixty_four_are_enough_and_nothing_else_is() {
    assert!(batches_are_equal_and_at_least_the_minimum(&[64, 64, 64]));
    assert!(batches_are_equal_and_at_least_the_minimum(&[144]));
    assert!(!batches_are_equal_and_at_least_the_minimum(&[63, 63]));
    assert!(!batches_are_equal_and_at_least_the_minimum(&[64, 65]));
    assert!(!batches_are_equal_and_at_least_the_minimum(&[]));
}

fn checks_where_everything_holds() -> ChecksOfARepeat {
    ChecksOfARepeat {
        busiest_host_other_cores: 1.0,
        is_identical: true,
        has_enough_batches: true,
        class_one_anchors_hold: true,
        class_two_anchors_hold: true,
    }
}

/// 停机条款第 4 条与作废第 3 条：甲类不成立停机；机器忙（> 16 核）整组作废、排在别的核之前；都成立才计数。
#[test]
fn the_repeat_checks_void_a_busy_host_first_and_stop_on_a_broken_class_one_anchor() {
    assert_eq!(
        verdict_of_the_repeat_checks(checks_where_everything_holds()),
        VerdictOfTheRepeatChecks::Counted
    );
    let broken_class_one = ChecksOfARepeat {
        class_one_anchors_hold: false,
        ..checks_where_everything_holds()
    };
    assert_eq!(
        verdict_of_the_repeat_checks(broken_class_one),
        VerdictOfTheRepeatChecks::Stopped("a_class_one_anchor_does_not_hold")
    );
    let busy_and_broken = ChecksOfARepeat {
        busiest_host_other_cores: 16.5,
        ..broken_class_one
    };
    assert_eq!(
        verdict_of_the_repeat_checks(busy_and_broken),
        VerdictOfTheRepeatChecks::Voided("the_host_was_busy")
    );
    let sixteen_cores = ChecksOfARepeat {
        busiest_host_other_cores: 16.0,
        ..checks_where_everything_holds()
    };
    assert_eq!(
        verdict_of_the_repeat_checks(sixteen_cores),
        VerdictOfTheRepeatChecks::Counted
    );
    let not_identical = ChecksOfARepeat {
        is_identical: false,
        ..checks_where_everything_holds()
    };
    assert_eq!(
        verdict_of_the_repeat_checks(not_identical),
        VerdictOfTheRepeatChecks::Stopped("the_arms_of_the_repeat_did_not_write_and_read_the_same")
    );
    let broken_class_two = ChecksOfARepeat {
        class_two_anchors_hold: false,
        ..checks_where_everything_holds()
    };
    assert_eq!(
        verdict_of_the_repeat_checks(broken_class_two),
        VerdictOfTheRepeatChecks::Stopped("a_class_two_anchor_does_not_hold_the_tier_is_void")
    );
}

/// 这一趟所在那一组的 `name=saving` 那一行（交替跑判读，[`judgement_of_a_lockstep_tier`]），判的是 `run_lines` 里的数。
fn saving_line_judged_from(run_lines: &[BTreeMap<String, String>]) -> String {
    let numbers: Vec<RunNumbers> = run_lines
        .iter()
        .filter_map(RunNumbers::of_the_fields)
        .collect();
    let arms_run: Vec<LockstepArm> = ["cpu", "gpu_one_card"]
        .into_iter()
        .map(|name| LockstepArm::of_the_name(name).expect("交替跑的臂"))
        .collect();
    judgement_of_a_lockstep_tier(UnitAreaTier::FourGibibytes, &numbers, &arms_run)
        .lines
        .into_iter()
        .find(|(name, _fields)| name == "saving")
        .map(|(_name, fields)| fields)
        .expect("点名一张卡臂跑了：有 saving 那一行")
}

/// 阳性对照（登记第十二节修订第 16 条新加的失败条款「判读分组看得见硬件组之间的差」）：一份合成的交替跑产物，组甲（驱动信息 alpha）
/// 与组乙（beta）各做完 5 次重复，组甲 5 个 S 全 > 0，组乙全 < 0。
/// 预期：这一趟在组甲判出「省」（sign=positive），在组乙判出「多占」（sign=negative）；两组合起来判是「不稳定」（sign=unstable）——
/// 这一对照分得出分没分组。
#[test]
fn the_two_hardware_groups_of_a_synthetic_product_are_judged_apart() {
    let alpha = group_of_the_one_card_with_the_driver_information("alpha");
    let beta = group_of_the_one_card_with_the_driver_information("beta");
    let mut lines = lines_of_a_synthetic_lockstep_session(1, &alpha, 0, 100_000);
    lines.extend(lines_of_a_synthetic_lockstep_session(2, &beta, 5, -100_000));
    let product = lines_already_in_a_product_of_the_owned(&lines);
    let saving_of_the_group = |driver_information: &str| {
        let plan = plan_of_a_lockstep_session(
            &product,
            3,
            &lockstep_arms_requested_named(&["cpu", "gpu_one_card"]),
            CardsOfTheEveryCardArm::EveryHandedOverCard,
            &one_card_handed_over_with_the_driver_information(driver_information),
            &host_hardware_of_the_tests(),
        )
        .expect("两组的产物：归组接着判");
        saving_line_judged_from(&plan.run_lines_of_the_group)
    };
    let (saving_of_alpha, saving_of_beta) =
        (saving_of_the_group("alpha"), saving_of_the_group("beta"));
    assert!(
        saving_of_alpha.ends_with(" sign=positive"),
        "组甲判「省」：{saving_of_alpha}"
    );
    assert!(
        saving_of_beta.ends_with(" sign=negative"),
        "组乙判「多占」：{saving_of_beta}"
    );
    let every_run_line_of_the_product: Vec<BTreeMap<String, String>> = product
        .iter()
        .filter(|(name, _fields)| name == "run")
        .map(|(_name, fields)| fields.clone())
        .collect();
    let pooled = saving_line_judged_from(&every_run_line_of_the_product);
    assert!(
        pooled.ends_with(" sign=unstable"),
        "两组合起来判是「不稳定」：{pooled}"
    );
}
