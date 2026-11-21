use super::*;
use crate::root_choice_fault_tests::cell_of;

#[test]
fn the_pinned_text_tables_hold_the_registered_number_of_rows() {
    assert_eq!(pinned_fault_cells().len(), 49);
    assert_eq!(
        PINNED_MOUNT_TWO_FAULT_TEXT.lines().count(),
        4 * MOUNT_FAULT_MENU_LENGTH
    );
}

/// 登记 9.2 里生成器一侧那几条变异的取样点：闸恒假、T1 当 TP、写行 txg 只看根、不枚举撕裂第三态、最后一次挂载不并系统配置那 9 个组合、c_见证 = 0 判真、
/// 已确认算进崩着的那一次、RI 算不罩、盘不在时两块盘的记录都读不出、KJ2 比真相里的根、RN 取键最小的根，都落在这几个格的计数上。
#[test]
fn the_generator_counts_of_the_sampling_cells_equal_the_pinned_values() {
    for (domain, mounts, user_publishes) in [
        (FaultDomain::GateOn, 1, 0),
        (FaultDomain::GateOn, 1, 1),
        (FaultDomain::GateOn, 1, 2),
        (FaultDomain::GateOn, 2, 0),
        (FaultDomain::GateOn, 2, 1),
        (FaultDomain::GateOnWrappingRootRing, 1, 11),
        (FaultDomain::GateOff, 1, 2),
    ] {
        let cell = cell_of(mounts, user_publishes);
        let tally = fault_tally_of_a_cell(domain, cell, false, 1);
        let findings = assert_the_pinned_fault_cell(domain, cell, &tally);
        assert!(findings.is_empty(), "{findings:?}");
    }
}

/// `--gate r2` 解析出的闸（登记 r5 修订 1）上，KF M = 2、U = 0 那一格的生成器计数与拒因三栏等于修1.七乙类（历史 3 094、被拒 99、
/// 走了重读 1 130、带撕裂 245；拒因 91、8、0）。9.2「R2 后一半拿掉」「两槽都自证过不看撕裂」「取号那一刻不判 (b)」「c_见证 = 0 判真」
/// 「--gate r2 认了但闸照取甲」点名这一条。
#[test]
fn the_read_error_rule_counts_of_the_kf_two_mount_cell_equal_the_read_error_rule_pinned_values() {
    let selection = parse_the_root_choice_faults_command_line(
        &[
            "--bounds",
            "segment1",
            "--family",
            "root_choice_faults",
            "--gate",
            "r2",
        ]
        .iter()
        .map(|word| (*word).to_string())
        .collect::<Vec<String>>(),
    )
    .expect("登记的命令行");
    let cell = cell_of(2, 0);
    let tally = fault_tally_of_a_cell_under_the_gate(
        FaultDomain::GateOn,
        cell,
        selection.witness_gate_rule,
        false,
        1,
    );
    assert_eq!(
        (tally.histories, tally.refused_mounts, tally.reread_mounts),
        (3_094, 99, 1_130),
        "KF M = 2、U = 0 在 R2 下的历史数、被拒数、走了重读的数"
    );
    let findings = assert_the_pinned_fault_cell_under_the_gate(
        selection.witness_gate_rule,
        FaultDomain::GateOn,
        cell,
        &tally,
    );
    assert!(findings.is_empty(), "{findings:?}");
}

/// 修1.七乙类「拒因分计」KF M = 2、U = 0 那一格：闸拒 91、取号那一刻 (b) 8、取号那一刻 (a) 0，三栏之和等于被拒的 99。
/// 9.2「拒因记错：取号那一刻 (b) 的拒记进闸那一栏」点名这一条。
#[test]
fn the_read_error_rule_refusals_of_the_kf_two_mount_cell_split_into_91_at_the_gate_8_without_a_self_verified_slot_and_0_undecidable(
) {
    let tally = fault_tally_of_a_cell_under_the_gate(
        FaultDomain::GateOn,
        cell_of(2, 0),
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots,
        false,
        1,
    );
    assert_eq!(
        [
            tally.refusals_by_reason[MountRefusalReason::TheGateAfterTheOneReread.index()],
            tally.refusals_by_reason
                [MountRefusalReason::AcquisitionDeviceWithoutASelfVerifiedSlot.index()],
            tally.refusals_by_reason[MountRefusalReason::AcquisitionWitnessUndecidable.index()],
            tally.refusals_by_reason
                [MountRefusalReason::AcquisitionUnderTheImplementationChoice.index()],
        ],
        [91, 8, 0, 0]
    );
    assert_eq!(tally.refused_mounts, 91 + 8);
}
