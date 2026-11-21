use super::*;

/// 每条判据在每个被判的格上各有对照读数、都红；读数条数照格数逐条判据数（删掉某条判据的某一行对照，条数对不上）。
#[test]
fn every_root_choice_criterion_has_a_positive_control_and_every_control_is_red() {
    let thread_count = thread_count_from_the_environment();
    let big_ring_cells: Vec<RootChoiceCell> = [(1, 0), (1, 2), (1, 3)]
        .into_iter()
        .map(|(mounts, user_publishes)| RootChoiceCell {
            mounts,
            user_publishes,
        })
        .collect();
    let selected_cells = vec![
        (RootChoiceDomain::BigRing, big_ring_cells.clone()),
        (
            RootChoiceDomain::AdversarialRootRing,
            vec![RootChoiceCell {
                mounts: 0,
                user_publishes: 0,
            }],
        ),
    ];
    let readings = root_choice_control_readings(&selected_cells, thread_count);
    for reading in &readings {
        assert!(reading.is_red_as_required, "对照没红：{reading:?}");
    }
    let readings_of = |criterion: RootChoiceCriterion| {
        readings
            .iter()
            .filter(|reading| reading.criterion == criterion)
            .count()
    };
    let cell_count = big_ring_cells.len();
    let local_rule_count = LocalRootChoiceRule::ALL.len();
    assert_eq!(readings_of(RootChoiceCriterion::Kj0Defined), 1);
    assert_eq!(
        readings_of(RootChoiceCriterion::Kj1NotEarlierThanTheLastConfirmedPublish),
        2 * cell_count,
        "KJ1 在每个格上 KF1、KF2 各一条对照"
    );
    assert_eq!(
        readings_of(RootChoiceCriterion::Kj2NotEarlierThanAnySelfVerifiedRoot),
        2 * cell_count + local_rule_count,
        "KJ2 在每个格上 KF1、KF2 各一条，在 KL 上每种本地择根序一条"
    );
    assert_eq!(
        readings_of(RootChoiceCriterion::Kj3LandsOnTheIdealRecovery),
        cell_count,
        "KJ3 在每个格上 KF2 一条"
    );
    for criterion in RootChoiceCriterion::ALL {
        assert!(
            readings
                .iter()
                .any(|reading| reading.criterion == *criterion),
            "判据 {} 没有对照的读数",
            criterion.label()
        );
    }
}

#[test]
fn the_root_choice_command_line_takes_the_family_and_rejects_unknown_domains_and_options() {
    let arguments =
        |text: &str| -> Vec<String> { text.split_whitespace().map(str::to_string).collect() };
    assert!(arguments_name_the_root_choice_family(&arguments(
        "--bounds segment1 --family root_choice"
    )));
    assert!(!arguments_name_the_root_choice_family(&arguments(
        "--bounds segment1 --family legal"
    )));
    let selection = parse_the_root_choice_command_line(&arguments(
        "--family root_choice --only K9,KL --max-u 1",
    ))
    .expect("认得的参数");
    assert_eq!(
        selection.bounds_to_judge,
        BoundsTier::REGISTERED.to_vec(),
        "不给 --bounds 是不指定范围：登记的每一段都判"
    );
    assert_eq!(
        selection.only,
        Some(vec!["K9".to_string(), "KL".to_string()])
    );
    assert_eq!(selection.largest_user_publishes_cap, Some(1));
    assert!(parse_the_root_choice_command_line(&arguments(
        "--bounds segment1 --family root_choice --only L"
    ))
    .is_err());
    assert!(parse_the_root_choice_command_line(&arguments(
        "--bounds segment1 --family root_choice --max-n 3"
    ))
    .is_err());
    assert!(parse_the_root_choice_command_line(&arguments(
        "--bounds segment3 --family root_choice"
    ))
    .is_err());
}
