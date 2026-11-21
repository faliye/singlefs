use super::*;
use crate::root_choice_fault_tests::cell_of;

#[test]
fn the_faults_profile_is_accepted_and_the_faults_family_name_is_recognised() {
    assert!(arguments_name_the_root_choice_faults_family(&[
        "--family".to_string(),
        "root_choice_faults".to_string()
    ]));
    assert!(!arguments_name_the_root_choice_faults_family(&[
        "--family".to_string(),
        "root_choice".to_string()
    ]));
    let selection = parse_the_root_choice_faults_command_line(&[
        "--bounds".to_string(),
        "segment1".to_string(),
        "--family".to_string(),
        "root_choice_faults".to_string(),
        "--only".to_string(),
        "KF,KL".to_string(),
    ])
    .expect("登记的命令行");
    assert_eq!(
        selection.only,
        Some(vec!["KF".to_string(), "KL".to_string()])
    );
    assert!(parse_the_root_choice_faults_command_line(&[
        "--bounds".to_string(),
        "segment1".to_string(),
        "--family".to_string(),
        "root_choice_faults".to_string(),
        "--only".to_string(),
        "KQ".to_string(),
    ])
    .is_err());
    let as_given = |profile: &str| {
        split_off_the_profile_options(&[
            "--profile".to_string(),
            profile.to_string(),
            "--bounds".to_string(),
            "segment1".to_string(),
        ])
    };
    assert_eq!(
        as_given("r4"),
        as_given("r3"),
        "r4 档的其余族与 r3 档相同：只多一族 root_choice_faults"
    );
}

/// 修1.八判别力自证第 1 条（门槛挪到两点之间：「SN0-TP 的历史全被拒」）：KF、KG 的 M = 2、U = 0 一格上，R2 的闸下 R2 那一条为真、取甲那一条为假；
/// 取甲的闸下反过来。9.2「R2 档的判别力自证仍用取甲那四条」点名这一条。
#[test]
fn the_read_error_rule_gate_refusal_proof_holds_under_the_read_error_rule_and_the_implementation_choice_proof_holds_only_under_the_implementation_choice(
) {
    for (witness_gate_rule, rule_label) in [
        (
            WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots,
            "R2",
        ),
        (
            WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable,
            "取甲",
        ),
    ] {
        let gate_on = fault_tally_of_a_cell_under_the_gate(
            FaultDomain::GateOn,
            cell_of(2, 0),
            witness_gate_rule,
            false,
            1,
        );
        let gate_off = fault_tally_of_a_cell_under_the_gate(
            FaultDomain::GateOff,
            cell_of(2, 0),
            witness_gate_rule,
            false,
            1,
        );
        let (_item, own_proof_holds, own_detail) =
            gate_refusal_proof_of(witness_gate_rule, &gate_on, &gate_off);
        assert!(
            own_proof_holds,
            "{rule_label} 的闸下它自己那一条为真：{own_detail}"
        );
        let other_proof_holds = match witness_gate_rule {
            WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
                gate_refusal_proof_under_the_implementation_choice(&gate_on, &gate_off).1
            }
            WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => {
                gate_refusal_proof_under_the_read_error_rule(&gate_on, &gate_off).1
            }
        };
        assert!(
            !other_proof_holds,
            "{rule_label} 的闸下另一种取法那一条为假"
        );
    }
}

/// `--gate r2` 只在 `--family root_choice_faults` 那一支认（别的族的参数解析报用法错），解出 R2；不给是取甲，config 行的闸字段与 r4 档逐字相同；
/// 别的值报错。9.2「--gate r2 认了但闸照取甲」点名这一条（config 行带 R2 的闸字段）。
#[test]
fn the_gate_option_names_the_read_error_rule_gate_in_the_configuration_row_only_on_the_faults_family(
) {
    let words = |list: &[&str]| {
        list.iter()
            .map(|word| (*word).to_string())
            .collect::<Vec<String>>()
    };
    let with_the_read_error_rule = parse_the_root_choice_faults_command_line(&words(&[
        "--bounds",
        "segment1",
        "--family",
        "root_choice_faults",
        "--gate",
        "r2",
    ]))
    .expect("登记的命令行");
    assert_eq!(
        with_the_read_error_rule.witness_gate_rule.label(),
        "r2_some_slot_reports_a_read_error_and_no_device_self_verified_both_slots",
        "--gate r2 的 config 行带 R2 的闸字段"
    );
    let without = parse_the_root_choice_faults_command_line(&words(&[
        "--bounds",
        "segment1",
        "--family",
        "root_choice_faults",
    ]))
    .expect("登记的命令行");
    assert_eq!(
        without.witness_gate_rule.label(),
        "the_implementation_choice_one_unreadable_or_torn_slot_means_undecidable",
        "不给 --gate 是取甲，config 行与 r4 档逐字相同"
    );
    assert!(parse_the_root_choice_faults_command_line(&words(&[
        "--bounds", "segment1", "--gate", "a"
    ]))
    .is_err());
    assert!(
        parse_the_command_line(&words(&[
            "--bounds", "segment1", "--family", "legal", "--gate", "r2"
        ]))
        .is_err(),
        "合法历史一族不认 --gate"
    );
}
