use super::*;

/// 场景：交来三张，全部卡臂照 `--cards` 只取了其中两张。
/// 预期：全部卡臂取卡、数张数、写型号都从挑过的两张里取；点名一张卡臂与 CPU 臂仍从交来的三张里取，`name=run` 行的型号串照旧是三张。
#[test]
fn only_the_every_card_arm_takes_its_cards_from_those_the_cards_option_named() {
    let three = three_discrete_gpus_seen_on_this_host();
    let two = &three[1..];
    assert_eq!(
        pci_bus_addresses_of(discrete_gpus_of_the_arm(
            ComputeOfTheArm::GpuOnEveryCard,
            &three,
            two
        )),
        ["0000:02:00.0", "0000:03:00.0"]
    );
    for compute in [
        ComputeOfTheArm::Cpu,
        ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
    ] {
        assert_eq!(
            discrete_gpus_of_the_arm(compute, &three, two).len(),
            3,
            "{compute:?}"
        );
    }
    assert_eq!(
        gpu_models_in_the_run_line(&three),
        "model_at_0000:01:00.0+model_at_0000:02:00.0+model_at_0000:03:00.0"
    );
}

/// 这一台一张卡都没交的情形（取卡模块交回 `None`）。
fn no_card_handed_to_this_host() -> CardsSeenByThisProcess {
    CardsSeenByThisProcess {
        discrete_gpus: Vec::new(),
        handed_over_pci_bus_addresses_in_the_handed_over_order: Vec::new(),
        vulkan_instance: None,
    }
}

/// 场景：交来三张（交来的先后 03、01、02），点了 CPU 臂、点名一张卡臂与全部卡臂（`--cards 2`）。
/// 预期：先打两行记卡（点名一张卡臂那一张 01；全部卡臂开交来的头两张 03、01，按 PCI 地址次序记；各带驱动与驱动信息，记卡行带趟号），
/// 再打 `name=hardware_group`，最后 `name=resume`；三条臂都跑。
#[test]
fn the_card_lines_carry_the_session_and_the_drivers_and_come_before_the_hardware_group_and_the_resume(
) {
    let plan = plan_of_a_lockstep_session(
        &[],
        3,
        &lockstep_arms_requested_named(&["cpu", "gpu_one_card", "gpu_every_card"]),
        CardsOfTheEveryCardArm::FirstHandedOverCards(NonZeroUsize::new(2).expect("2 不是 0")),
        &three_cards_handed_over_out_of_the_pci_order(),
        &host_hardware_of_the_tests(),
    )
    .expect("交来三张、--cards 2：接着跑");
    assert_eq!(plan.indices_of_the_arms_run, [0, 1, 2]);
    assert_eq!(
        plan.lines
            .iter()
            .map(|(kind, _fields)| *kind)
            .collect::<Vec<&str>>(),
        [
            "one_card_arm_card",
            "every_card_arm_cards",
            "hardware_group",
            "resume"
        ]
    );
    assert_eq!(
        plan.lines[0].1,
        "session=3 tier=4GiB pci_bus_address=0000:01:00.0 model=model_at_0000:01:00.0 driver=driver \
             driver_information=driver_information_at_0000:01:00.0"
    );
    assert_eq!(
        plan.lines[1].1,
        "session=3 tier=4GiB cards_named=2 named_by=cards_option pci_bus_addresses=0000:01:00.0,0000:03:00.0 \
             models=model_at_0000:01:00.0,model_at_0000:03:00.0 drivers=driver,driver \
             driver_information_of_each_card=driver_information_at_0000:01:00.0,driver_information_at_0000:03:00.0"
    );
    assert_eq!(
        plan.lines[2].1,
        plan.group
            .fields_of_the_line(3, UnitAreaTier::FourGibibytes)
    );
    assert_eq!(
        (plan.group.one_card.as_str(), plan.group.every_card.as_str()),
        (
            "0000:01:00.0|model_at_0000:01:00.0|driver|driver_information_at_0000:01:00.0",
            "2|0000:01:00.0|model_at_0000:01:00.0|driver|driver_information_at_0000:01:00.0;\
                 0000:03:00.0|model_at_0000:03:00.0|driver|driver_information_at_0000:03:00.0"
        )
    );
}

/// 场景：产物里组甲（驱动信息 alpha）做完 5 次重复；这一趟换了驱动信息（beta，组乙）续跑，产物里另有一趟记了卡就断、没出结果行的（组乙）。
/// 预期：不停、接着跑：组乙做完 0 次、读回 0 行，下一个重复号照全产物往后编（5）；`name=resume` 列两组与各自做完的数。
#[test]
fn resuming_in_another_hardware_group_goes_on_with_none_of_the_repeats_of_the_other_group() {
    let alpha = group_of_the_one_card_with_the_driver_information("alpha");
    let beta = group_of_the_one_card_with_the_driver_information("beta");
    let mut lines = lines_of_a_synthetic_lockstep_session(1, &alpha, 0, 100_000);
    lines.push("name=session_start session=2 command=lockstep".to_string());
    lines.push(format!(
        "name=hardware_group {}",
        beta.fields_of_the_line(2, UnitAreaTier::FourGibibytes)
    ));
    let plan = plan_of_a_lockstep_session(
        &lines_already_in_a_product_of_the_owned(&lines),
        3,
        &lockstep_arms_requested_named(&["cpu", "gpu_one_card"]),
        CardsOfTheEveryCardArm::EveryHandedOverCard,
        &one_card_handed_over_with_the_driver_information("beta"),
        &host_hardware_of_the_tests(),
    )
    .expect("换了驱动信息的一趟：归到组乙接着跑，不停");
    assert_eq!(plan.group.identifier, beta.identifier);
    assert_eq!(
        (
            plan.repeats_of_the_group_done.len(),
            plan.run_lines_of_the_group.len(),
            plan.next_repeat
        ),
        (0, 0, 5)
    );
    let (alpha_listed, beta_listed) = (
        format!("{}:5", alpha.identifier),
        format!("{}:0", beta.identifier),
    );
    let mut listed = [alpha_listed, beta_listed];
    listed.sort();
    assert_eq!(
        plan.lines.last(),
        Some(&(
            "resume",
            format!(
                "tier=4GiB repeats_already_done=5 next_repeat=5 group={} repeats_of_the_group_already_done=0 groups_in_the_product={}",
                beta.identifier,
                listed.join(",")
            )
        ))
    );
}

/// 场景：产物里组甲做完 5 次重复；这一趟还是组甲。
/// 预期：这一组做完 5 次（k ≥ 5 不再跑、照判），读回 10 条结果行。
#[test]
fn resuming_in_the_same_hardware_group_takes_its_five_repeats() {
    let alpha = group_of_the_one_card_with_the_driver_information("alpha");
    let plan = plan_of_a_lockstep_session(
        &lines_already_in_a_product_of_the_owned(&lines_of_a_synthetic_lockstep_session(
            1, &alpha, 0, 100_000,
        )),
        2,
        &lockstep_arms_requested_named(&["cpu", "gpu_one_card"]),
        CardsOfTheEveryCardArm::EveryHandedOverCard,
        &one_card_handed_over_with_the_driver_information("alpha"),
        &host_hardware_of_the_tests(),
    )
    .expect("同一组：接着判");
    assert_eq!(
        (
            plan.repeats_of_the_group_done.len(),
            plan.run_lines_of_the_group.len(),
            plan.next_repeat
        ),
        (5, 10, 5)
    );
}

/// 场景：产物里第 1 趟有结果行（`name=run`）而没有 `name=hardware_group` 行。
/// 预期：拒、退 2，那一行点名那一趟、出路是换一个 `--product`；一行 `not_run`、记卡、`hardware_group` 都不打。
#[test]
fn a_session_with_runs_and_without_a_hardware_group_refuses_the_resume() {
    assert_eq!(
        plan_of_a_lockstep_session(
            &lines_already_in_a_product_of(&[
                "name=session_start session=1 command=lockstep",
                "name=run tier=4GiB arm=cpu repeat=0 session=1 void=no batches=144",
            ]),
            2,
            &lockstep_arms_requested_named(&["cpu", "gpu_one_card"]),
            CardsOfTheEveryCardArm::EveryHandedOverCard,
            &one_card_handed_over_with_the_driver_information("alpha"),
            &host_hardware_of_the_tests(),
        ),
        Err(RefusalBeforeTheRun {
            lines: vec![(
                "stopped",
                "clause=product_session_without_a_hardware_group session=1 way_out=换一个--product"
                    .to_string()
            )],
            exit_code: EXIT_USAGE,
        })
    );
}

/// 场景：只交来一张卡、没给 `--cards`，点了 CPU 臂与全部卡臂。
/// 预期：全部卡臂报本次未跑，不打 `name=every_card_arm_cards`，组键里全部卡臂那一段是 none（与只点 CPU 臂的那一组相同）。
#[test]
fn an_every_card_arm_not_run_records_no_card_line_and_leaves_its_segment_none_in_the_group_key() {
    let plan = plan_of_a_lockstep_session(
        &[],
        1,
        &lockstep_arms_requested_named(&["cpu", "gpu_every_card"]),
        CardsOfTheEveryCardArm::EveryHandedOverCard,
        &one_card_handed_over_with_the_driver_information("alpha"),
        &host_hardware_of_the_tests(),
    )
    .expect("CPU 臂照跑");
    assert_eq!(plan.indices_of_the_arms_run, [0]);
    assert_eq!(
        plan.lines
            .iter()
            .map(|(kind, _fields)| *kind)
            .collect::<Vec<&str>>(),
        ["not_run", "hardware_group", "resume"]
    );
    assert_eq!(
        plan.group,
        hardware_group_of(&host_hardware_of_the_tests(), None, None)
    );
}

/// 场景：这一台一张卡都没交，点了 CPU 臂与点名一张卡臂；另一趟 `--cards 3` 而交来的只有一张。
/// 预期：头一趟两条臂都报本次未跑、退 77，不记卡、不打硬件组；后一趟拒、退 2，那一行带出路「--cards改小或不给」。
#[test]
fn no_card_handed_over_refuses_with_seventy_seven_and_too_many_cards_named_with_two() {
    let refusal_without_cards = plan_of_a_lockstep_session(
        &[],
        1,
        &lockstep_arms_requested_named(&["cpu", "gpu_one_card"]),
        CardsOfTheEveryCardArm::EveryHandedOverCard,
        &no_card_handed_to_this_host(),
        &host_hardware_of_the_tests(),
    )
    .expect_err("一张卡都没交：一条臂都不跑");
    assert_eq!(
        (
            refusal_without_cards.exit_code,
            refusal_without_cards
                .lines
                .iter()
                .map(|(kind, _fields)| *kind)
                .collect::<Vec<&str>>()
        ),
        (EXIT_NO_ARM_RAN, vec!["not_run", "not_run"])
    );
    let refusal_of_the_cards_option = plan_of_a_lockstep_session(
        &[],
        1,
        &lockstep_arms_requested_named(&["cpu", "gpu_every_card"]),
        CardsOfTheEveryCardArm::FirstHandedOverCards(NonZeroUsize::new(3).expect("3 不是 0")),
        &one_card_handed_over_with_the_driver_information("alpha"),
        &host_hardware_of_the_tests(),
    )
    .expect_err("--cards 3 多于交来的一张");
    assert_eq!(refusal_of_the_cards_option.exit_code, EXIT_USAGE);
    assert!(
        refusal_of_the_cards_option.lines.len() == 1
            && refusal_of_the_cards_option.lines[0].0 == "stopped"
            && refusal_of_the_cards_option.lines[0]
                .1
                .starts_with("clause=cards_option_exceeds_the_cards_handed_over ")
            && refusal_of_the_cards_option.lines[0]
                .1
                .ends_with(" way_out=--cards改小或不给"),
        "{:?}",
        refusal_of_the_cards_option.lines
    );
}
