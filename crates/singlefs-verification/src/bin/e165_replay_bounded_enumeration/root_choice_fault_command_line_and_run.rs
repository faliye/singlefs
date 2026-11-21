//! 带读故障一族的命令行与主流程（`--family root_choice_faults`，只在 `--profile r4`）：对照行、自证与按界的整趟。

use super::*;

// ── 命令行与主流程（`--family root_choice_faults`，只在 `--profile r4`）

pub(super) struct FaultSelection {
    /// 这一趟判的登记界，按次序逐段判（[`bounds_to_judge_of_the_option`]）。
    pub(super) bounds_to_judge: Vec<BoundsTier>,
    pub(super) only: Option<Vec<String>>,
    pub(super) largest_user_publishes_cap: Option<usize>,
    /// `--gate r2` 时 R2，不给时取甲（登记 r5 修订 1）。
    pub(super) witness_gate_rule: WitnessGateRule,
}

pub(super) fn arguments_name_the_root_choice_faults_family(arguments: &[String]) -> bool {
    arguments
        .windows(2)
        .any(|pair| pair[0] == "--family" && pair[1] == FAULT_FAMILY_LABEL)
}

/// `--bounds segment1|segment2`（这一族 segment1 与 segment2 同界；不给是不指定范围，两段都判，[`bounds_to_judge_of_the_option`]）、`--family root_choice_faults`、
/// `--only KF,KF3,KF9,KFW,KG,KL`、`--max-u <U>`（每个格的 U 封顶，只给调试用）。
pub(super) fn parse_the_root_choice_faults_command_line(
    arguments: &[String],
) -> Result<FaultSelection, String> {
    let mut bounds: Option<BoundsTier> = None;
    let mut only: Option<Vec<String>> = None;
    let mut largest_user_publishes_cap: Option<usize> = None;
    let mut witness_gate_rule = WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable;
    let mut position = 0;
    while position < arguments.len() {
        let option = arguments[position].as_str();
        let value = arguments
            .get(position + 1)
            .ok_or_else(|| format!("{option} 后面缺一个值"))?
            .as_str();
        match option {
            "--bounds" => {
                bounds = Some(match value {
                    "segment1" => BoundsTier::SegmentOne,
                    "segment2" => BoundsTier::SegmentTwo,
                    unknown_tier => return Err(format!("不认识的界：{unknown_tier}")),
                });
            }
            "--gate" => {
                witness_gate_rule = match value {
                    "r2" => {
                        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots
                    }
                    unknown_gate => {
                        return Err(format!(
                        "不认识的闸：{unknown_gate}（只认 r2；不给 --gate 是取甲，登记 r5 修订 1）"
                    ))
                    }
                };
            }
            "--family" => {
                if value != FAULT_FAMILY_LABEL {
                    return Err(format!("不认识的族：{value}"));
                }
            }
            "--only" => {
                let names: Vec<String> = value.split(',').map(str::to_string).collect();
                for name in &names {
                    if !FaultDomain::ALL.iter().any(|domain| domain.label() == name) {
                        return Err(format!(
                            "不认识的域：{name}（这一族的域是 KF、KF3、KF9、KFW、KG、KL）"
                        ));
                    }
                }
                only = Some(names);
            }
            "--max-u" => {
                largest_user_publishes_cap = Some(
                    value
                        .parse::<usize>()
                        .map_err(|error| format!("--max-u 要一个非负整数：{error}"))?,
                );
            }
            unknown_option => return Err(format!("不认识的参数：{unknown_option}")),
        }
        position += 2;
    }
    Ok(FaultSelection {
        bounds_to_judge: bounds_to_judge_of_the_option(bounds),
        only,
        largest_user_publishes_cap,
        witness_gate_rule,
    })
}

/// 跑中途收集的、收尾要用的几样读数（判别力自证、C331 的判别力自证）。
#[derive(Default)]
struct FaultRunWitnesses {
    mount_two_by_domain: Vec<(FaultDomain, FaultTally)>,
    /// KF 的 M = 2 每一格（U, 取号那一刻因「某块盘一份自证过的都没有」被拒的挂载数），R2 的判别力自证第 3 条用。
    acquisition_refusals_without_a_self_verified_slot_on_kf_two_mounts: Vec<(usize, u64)>,
    torn_states_of_the_gate_on_domain_at_two_mounts_without_user_publishes: Option<u64>,
    kg_kf4_at_two_mounts_and_three_user_publishes: Option<[u64; 2]>,
    kg_local_today_kj1t_without_a_judged_fault: u64,
    kg_implementation_kj1t_without_a_judged_fault: u64,
    kg_implementation_kj3_without_a_judged_fault: u64,
    kg_was_run: bool,
}

fn emit_the_control_rows(
    emitter: &mut Emitter,
    findings: &mut RunFindings,
    readings: &[FaultControlReading],
) {
    for reading in readings {
        emitter.emit_result_line(&format!(
            "name=control criterion={} measured={} value={} must_see={} red_as_required={}",
            reading.criterion.label(),
            reading.measured_where.replace(' ', ","),
            reading.value,
            reading.must_see,
            reading.is_red_as_required
        ));
        if !reading.is_red_as_required {
            findings.void_conditions.push(format!(
                "TV3 判据 {} 的阳性对照没红：{} 量到 {}，必须 {}",
                reading.criterion.label(),
                reading.measured_where,
                reading.value,
                reading.must_see
            ));
        }
    }
}

/// 登记第八节「判别力自证」四条与 C331 的判别力自证（5.6）：读数都来自这一次跑出的格，不另跑。
/// 挂载 2 开头那项故障在菜单里的下标。
fn mount_fault_menu_index_of(label: &str) -> usize {
    MOUNT_FAULT_MENU_LABELS
        .iter()
        .position(|candidate| *candidate == label)
        .expect("故障名在菜单之内")
}

/// 取甲的判别力自证第 1 条（r4 登记 5.6）：KF 的 M = 2 各格合起来，SN0-TP、SN1-TP、SB-TP、RNSB-TP 四项每一项的历史全被拒，KG 同四项一个不拒。
fn gate_refusal_proof_under_the_implementation_choice(
    gate_on: &FaultTally,
    gate_off: &FaultTally,
) -> (String, bool, String) {
    let mut gate_refusal_proof_passed = true;
    let mut gate_refusal_proof_detail = Vec::new();
    for label in ["SN0-TP", "SN1-TP", "SB-TP", "RNSB-TP"] {
        let with_the_gate = gate_on.mount_two[mount_fault_menu_index_of(label)];
        let without_the_gate = gate_off.mount_two[mount_fault_menu_index_of(label)];
        gate_refusal_proof_passed &= with_the_gate.histories > 0
            && with_the_gate.refused == with_the_gate.histories
            && without_the_gate.refused == 0;
        gate_refusal_proof_detail.push(format!(
            "{label}:KF_histories={}_KF_refused={}_KG_refused={}",
            with_the_gate.histories, with_the_gate.refused, without_the_gate.refused
        ));
    }
    (
        "gate_refuses_every_history_of_the_four_tp_faults_and_the_gate_off_domain_refuses_none"
            .to_string(),
        gate_refusal_proof_passed,
        gate_refusal_proof_detail.join(","),
    )
}

/// R2 的判别力自证第 1 条（r5 登记修订 1 修1.八）：KF 的 M = 2 各格合起来，SN0-TP、SN1-TP 两项有历史、有被拒、而不是全拒（取甲下全拒）；
/// SB-TP、RNSB-TP 两项历史全被拒；KG 同四项一个不拒。把门槛挪到两点之间（「SN0-TP 的历史全被拒」）：取甲那一点为真、R2 这一点为假。
fn gate_refusal_proof_under_the_read_error_rule(
    gate_on: &FaultTally,
    gate_off: &FaultTally,
) -> (String, bool, String) {
    let mut gate_refusal_proof_passed = true;
    let mut gate_refusal_proof_detail = Vec::new();
    for (label, every_history_is_refused) in [
        ("SN0-TP", false),
        ("SN1-TP", false),
        ("SB-TP", true),
        ("RNSB-TP", true),
    ] {
        let with_the_gate = gate_on.mount_two[mount_fault_menu_index_of(label)];
        let without_the_gate = gate_off.mount_two[mount_fault_menu_index_of(label)];
        let refusal_matches_the_rule = if every_history_is_refused {
            with_the_gate.refused == with_the_gate.histories
        } else {
            with_the_gate.refused > 0 && with_the_gate.refused < with_the_gate.histories
        };
        gate_refusal_proof_passed &= with_the_gate.histories > 0
            && refusal_matches_the_rule
            && without_the_gate.refused == 0;
        gate_refusal_proof_detail.push(format!(
            "{label}:KF_histories={}_KF_refused={}_KG_refused={}",
            with_the_gate.histories, with_the_gate.refused, without_the_gate.refused
        ));
    }
    (
        "r2_gate_lets_part_of_the_sn_tp_histories_through_refuses_every_sb_and_rnsb_tp_history_and_the_gate_off_domain_refuses_none"
            .to_string(),
        gate_refusal_proof_passed,
        gate_refusal_proof_detail.join(","),
    )
}

/// 判别力自证第 1 条照闸的取法选（取甲那一条与 R2 那一条判的是相反的两点）。
fn gate_refusal_proof_of(
    witness_gate_rule: WitnessGateRule,
    gate_on: &FaultTally,
    gate_off: &FaultTally,
) -> (String, bool, String) {
    match witness_gate_rule {
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => {
            gate_refusal_proof_under_the_implementation_choice(gate_on, gate_off)
        }
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
            gate_refusal_proof_under_the_read_error_rule(gate_on, gate_off)
        }
    }
}

/// 判别力自证第 2 条（T1 生效）：取甲照 r4 登记 5.6（SN0-T1、SN1-T1、SB-T1 走第二遍的挂载数都不是 0）；R2 照 r5 登记修订 1 修1.八
/// （SB-T1、RNSB-T1 走第二遍的挂载数等于那一项的历史数，SN0-T1、SN1-T1 不是 0）。
fn second_read_proof_of(
    witness_gate_rule: WitnessGateRule,
    gate_on: &FaultTally,
) -> (String, bool, String) {
    let mut second_read_proof_passed = true;
    let mut second_read_proof_detail = Vec::new();
    match witness_gate_rule {
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => {
            for label in ["SN0-T1", "SN1-T1", "SB-T1"] {
                let with_the_gate = gate_on.mount_two[mount_fault_menu_index_of(label)];
                second_read_proof_passed &= with_the_gate.reread > 0;
                second_read_proof_detail
                    .push(format!("{label}:KF_reread={}", with_the_gate.reread));
            }
            (
                "t1_takes_the_second_read_in_the_gate_on_domain".to_string(),
                second_read_proof_passed,
                second_read_proof_detail.join(","),
            )
        }
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
            for (label, every_history_takes_the_second_read) in [
                ("SN0-T1", false),
                ("SN1-T1", false),
                ("SB-T1", true),
                ("RNSB-T1", true),
            ] {
                let with_the_gate = gate_on.mount_two[mount_fault_menu_index_of(label)];
                second_read_proof_passed &= with_the_gate.reread > 0
                    && (!every_history_takes_the_second_read
                        || with_the_gate.reread == with_the_gate.histories);
                second_read_proof_detail.push(format!(
                    "{label}:KF_histories={}_KF_reread={}",
                    with_the_gate.histories, with_the_gate.reread
                ));
            }
            (
                "r2_t1_takes_the_second_read_on_every_sb_and_rnsb_t1_history_and_on_some_sn_t1_histories"
                    .to_string(),
                second_read_proof_passed,
                second_read_proof_detail.join(","),
            )
        }
    }
}

fn emit_the_self_proofs(
    emitter: &mut Emitter,
    findings: &mut RunFindings,
    witnesses: &FaultRunWitnesses,
    witness_gate_rule: WitnessGateRule,
) {
    let mount_two_of = |domain: FaultDomain| {
        witnesses
            .mount_two_by_domain
            .iter()
            .find(|(candidate, _tally)| *candidate == domain)
            .map(|(_domain, tally)| tally)
    };
    let mut proofs: Vec<(String, bool, String)> = Vec::new();
    if let (Some(gate_on), Some(gate_off)) = (
        mount_two_of(FaultDomain::GateOn),
        mount_two_of(FaultDomain::GateOff),
    ) {
        proofs.push(gate_refusal_proof_of(witness_gate_rule, gate_on, gate_off));
        proofs.push(second_read_proof_of(witness_gate_rule, gate_on));
    }
    // R2 的判别力自证第 3 条（r5 登记修订 1 修1.八）：取号那一刻第二道拒（某块盘一份自证过的都没有）在 KF 的 M = 2 每一格都拒过挂载。
    match witness_gate_rule {
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => {}
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
            let cells =
                &witnesses.acquisition_refusals_without_a_self_verified_slot_on_kf_two_mounts;
            if !cells.is_empty() {
                proofs.push((
                    "r2_acquisition_refuses_a_device_without_a_self_verified_slot_on_every_kf_two_mount_cell"
                        .to_string(),
                    cells.iter().all(|(_user_publishes, refused)| *refused > 0),
                    cells
                        .iter()
                        .map(|(user_publishes, refused)| format!("KF_M2_U{user_publishes}:refused={refused}"))
                        .collect::<Vec<String>>()
                        .join(","),
                ));
            }
        }
    }
    if let Some([kj1, kj1t]) = witnesses.kg_kf4_at_two_mounts_and_three_user_publishes {
        proofs.push((
            "kf4_parts_kj1_from_kj1t_on_the_gate_off_domain".to_string(),
            kj1 != kj1t,
            format!("KF4_kj1={kj1},KF4_kj1t={kj1t}"),
        ));
    }
    if let Some(torn) =
        witnesses.torn_states_of_the_gate_on_domain_at_two_mounts_without_user_publishes
    {
        proofs.push((
            "torn_third_state_is_enumerated".to_string(),
            torn > 0,
            format!("KF_M2_U0_states_with_a_torn_system_slot={torn}"),
        ));
    }
    for (item, passed, detail) in &proofs {
        emitter.emit_result_line(&format!(
            "name=self_proof item={item} passed={passed} detail={detail}"
        ));
        if !passed {
            findings
                .void_conditions
                .push(format!("TV3 判别力自证没过：{item}（{detail}）"));
        }
    }
    if witnesses.kg_was_run {
        emitter.emit_result_line(&format!(
            "name=c331_proof domain=KG local_today_kj1t_without_a_judged_fault={} pinned={PINNED_KG_TODAY_DESCENT_VIOLATIONS_WITHOUT_A_JUDGED_FAULT} implementation_kj1t_without_a_judged_fault={} implementation_kj3_without_a_judged_fault={}",
            witnesses.kg_local_today_kj1t_without_a_judged_fault,
            witnesses.kg_implementation_kj1t_without_a_judged_fault,
            witnesses.kg_implementation_kj3_without_a_judged_fault,
        ));
        if witnesses
            .kg_kf4_at_two_mounts_and_three_user_publishes
            .is_some()
            && witnesses.kg_local_today_kj1t_without_a_judged_fault == 0
        {
            findings.void_conditions.push(
                "TV3 KG 没造出 C331 那一形：本地理想恢复在被判时无故障的态上 KJ1T 合计为 0（只作废 KG 那一行与依它的「拿掉保护会丢」）"
                    .to_string(),
            );
        }
    }
}

/// KL（与 r3 档同一个生成函数）：三步在 27 个照落点摆根的对抗根环上，行与判决用 r3 的写法；对照是本地写的六种择根序。
fn run_the_adversarial_domain_of_the_faults_family(
    emitter: &mut Emitter,
    findings: &mut RunFindings,
    control_readings: &mut Vec<FaultControlReading>,
) -> u64 {
    let geometry = RootChoiceDomain::AdversarialRootRing;
    let cell = RootChoiceCell {
        mounts: 0,
        user_publishes: 0,
    };
    let tally =
        root_choice_tally_of_the_adversarial_root_rings(RootChoiceImplementation::TodayThreeSteps);
    assert_the_pinned_landing_slot_adversarial_shape(&tally);
    emit_the_rows_of_a_root_choice_cell(emitter, geometry, cell, &tally);
    let readings = fault_kl_control_readings();
    emit_the_control_rows(emitter, findings, &readings);
    control_readings.extend(readings);
    emit_the_verdict_of_a_root_choice_domain(emitter, findings, geometry, &[cell], &tally);
    tally.count_of(RootChoiceCounter::States)
}

pub(super) fn run_the_root_choice_faults_family(arguments: &[String]) {
    let selection = match parse_the_root_choice_faults_command_line(arguments) {
        Ok(selection) => selection,
        Err(problem) => {
            eprintln!("  ✗ {problem}");
            eprintln!("  → 怎么办：{USAGE}");
            std::process::exit(2);
        }
    };
    install_the_panic_hook_that_stays_quiet_inside_the_replay_call();
    let outcomes: Vec<OutcomeOfTheBounds> = selection
        .bounds_to_judge
        .iter()
        .map(|bounds| run_the_root_choice_faults_family_under_the_bounds(&selection, *bounds))
        .collect();
    exit_when_some_bounds_ended_void_or_halted(&outcomes);
}

/// 带读故障的择根一族在一段登记界下跑一趟（[`run_the_root_choice_faults_family`] 逐段调它）。
fn run_the_root_choice_faults_family_under_the_bounds(
    selection: &FaultSelection,
    bounds: BoundsTier,
) -> OutcomeOfTheBounds {
    let thread_count = thread_count_from_the_environment();
    let run_started = Instant::now();
    let mut emitter = Emitter { emitted: 0 };
    let mut findings = RunFindings::default();
    let selected_cells: Vec<(FaultDomain, Vec<RootChoiceCell>)> = FaultDomain::ALL
        .iter()
        .copied()
        .filter(|domain| {
            selection
                .only
                .as_ref()
                .is_none_or(|names| names.iter().any(|name| name == domain.label()))
        })
        .map(|domain| {
            (
                domain,
                fault_cells_of(domain, bounds, selection.largest_user_publishes_cap),
            )
        })
        .collect();
    emitter.emit_result_line(&format!(
        "name=config experiment=E165 bounds={} family={FAULT_FAMILY_LABEL} only={} max_u={} root_slot_bytes={ROOT_CHOICE_ROOT_SLOT_BYTES} region_devices=0,1,0 system_slots_per_device={FAULT_SYSTEM_SLOTS_PER_DEVICE} witness_gate={} profile=r4{}",
        bounds.label(),
        selection
            .only
            .as_ref()
            .map_or_else(|| "every_domain".to_string(), |names| names.join(",")),
        selection
            .largest_user_publishes_cap
            .map_or_else(|| "none".to_string(), |cap| cap.to_string()),
        selection.witness_gate_rule.label(),
        narrowing_field_of_the_configuration_line(selection.largest_user_publishes_cap, "max_u"),
    ));
    // ① 常量回比断言（不过当场 panic，进程以非 0 退出，一行结果都不算）。
    cross_check_root_choice_fault_local_constants_against_the_implementation();
    emitter.emit_result_line(
        "name=self_check check=root_choice_fault_local_constants_equal_the_implementation passed=true",
    );
    let self_checks_passed = 1;
    for ring in [RingGeometry::BIG, RingGeometry::SMALL_NINE] {
        emitter.emit_result_line(&format!(
            "name=in_flight_limit ring_slots={} local={} implementation={}",
            ring.ring_slots,
            ring.local_in_flight_limit(),
            journal_in_flight_record_limit(ring.ring_bytes())
        ));
    }
    let mut witnesses = FaultRunWitnesses::default();
    let mut control_readings: Vec<FaultControlReading> = Vec::new();
    let mut total_states: u64 = 0;
    let the_whole_registered_bound_is_run = selection.largest_user_publishes_cap.is_none()
        && matches!(bounds, BoundsTier::SegmentOne | BoundsTier::SegmentTwo);
    for (domain, cells) in &selected_cells {
        if !domain.has_histories() {
            total_states += run_the_adversarial_domain_of_the_faults_family(
                &mut emitter,
                &mut findings,
                &mut control_readings,
            );
            continue;
        }
        let mut domain_total = FaultTally::new();
        let mut mount_two_merged = FaultTally::new();
        for cell in cells {
            let cell_started = Instant::now();
            let tally = fault_tally_of_a_cell_under_the_gate(
                *domain,
                *cell,
                selection.witness_gate_rule,
                true,
                thread_count,
            );
            findings
                .halt_conditions
                .extend(assert_the_pinned_fault_cell_under_the_gate(
                    selection.witness_gate_rule,
                    *domain,
                    *cell,
                    &tally,
                ));
            assert_eq!(
                tally.chose_nothing_without_a_readable_root,
                pinned_fault_cell_under_the_gate(selection.witness_gate_rule, *domain, *cell)
                    .expect("每个跑到的格都在登记第七节乙类的表里")
                    .states_without_a_readable_root,
                "TV4：{} 读视图里一条根都没有的被判态里 choose_root 交回空的态数",
                fault_place_of(*domain, *cell)
            );
            emit_the_rows_of_a_fault_cell(&mut emitter, *domain, *cell, &tally);
            match selection.witness_gate_rule {
                WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => {}
                WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
                    emitter.emit_result_line(&refusal_row_of_a_fault_cell(*domain, *cell, &tally));
                }
            }
            if *domain == FaultDomain::GateOn && cell.mounts == 2 {
                witnesses
                    .acquisition_refusals_without_a_self_verified_slot_on_kf_two_mounts
                    .push((
                        cell.user_publishes,
                        tally.refusals_by_reason
                            [MountRefusalReason::AcquisitionDeviceWithoutASelfVerifiedSlot.index()],
                    ));
            }
            let readings = fault_control_readings_of_a_cell_under_the_gate(
                selection.witness_gate_rule,
                *domain,
                *cell,
                &tally,
            );
            emit_the_control_rows(&mut emitter, &mut findings, &readings);
            control_readings.extend(readings);
            println!(
                "E7TIMING name=elapsed scope=cell domain={} mounts={} user_publishes={} states={} elapsed_seconds={:.3} threads={thread_count}",
                domain.label(),
                cell.mounts,
                cell.user_publishes,
                tally.judged_states,
                cell_started.elapsed().as_secs_f64()
            );
            if *domain == FaultDomain::GateOn && cell.mounts == 2 && cell.user_publishes == 0 {
                witnesses.torn_states_of_the_gate_on_domain_at_two_mounts_without_user_publishes =
                    Some(tally.states_with_a_torn_system_slot);
            }
            if *domain == FaultDomain::GateOff {
                witnesses.kg_was_run = true;
                witnesses.kg_local_today_kj1t_without_a_judged_fault +=
                    tally.local_kj1t_by_judged_fault[0][JudgedFault::Nothing.index()];
                for history_is_covered in [true, false] {
                    let none_cell =
                        &tally.cells[fault_cell_index_of(JudgedFault::Nothing, history_is_covered)];
                    witnesses.kg_implementation_kj1t_without_a_judged_fault +=
                        none_cell.violations[DESCENDS_FROM_THE_LAST_CONFIRMED_PUBLISH_INDEX];
                    witnesses.kg_implementation_kj3_without_a_judged_fault +=
                        none_cell.violations[KJ3_INDEX];
                }
                if cell.mounts == 2 && cell.user_publishes == 3 {
                    witnesses.kg_kf4_at_two_mounts_and_three_user_publishes = Some(tally.kf4);
                }
            }
            domain_total.merge(&tally);
            if cell.mounts == 2 {
                mount_two_merged.merge(&tally);
            }
        }
        if the_whole_registered_bound_is_run && domain.largest_mounts(bounds) >= 2 {
            assert_the_pinned_mount_two_table_under_the_gate(
                selection.witness_gate_rule,
                *domain,
                &mount_two_merged,
            );
        }
        if cells.iter().any(|cell| cell.mounts == 2) {
            witnesses
                .mount_two_by_domain
                .push((*domain, mount_two_merged));
        }
        total_states += domain_total.judged_states;
        emit_the_verdict_of_a_fault_domain(
            &mut emitter,
            &mut findings,
            *domain,
            cells,
            &domain_total,
            bounds,
        );
    }
    emit_the_self_proofs(
        &mut emitter,
        &mut findings,
        &witnesses,
        selection.witness_gate_rule,
    );
    for condition in &findings.void_conditions {
        emitter.emit_result_line(&format!(
            "name=void condition={}",
            condition.replace(' ', "_")
        ));
    }
    for condition in &findings.halt_conditions {
        emitter.emit_result_line(&format!(
            "name=halt condition={}",
            condition.replace(' ', "_")
        ));
    }
    emitter.emit_result_line(&format!(
        "name=verdict scope=run bounds={} family={FAULT_FAMILY_LABEL} states={total_states} self_checks_passed={self_checks_passed} control_readings={} controls_not_red={} void_conditions={} halt_conditions={} profile=r4{}",
        bounds.label(),
        control_readings.len(),
        control_readings
            .iter()
            .filter(|reading| !reading.is_red_as_required)
            .count(),
        findings.void_conditions.len(),
        findings.halt_conditions.len(),
        match selection.witness_gate_rule {
            WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => String::new(),
            WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
                format!(" witness_gate={}", selection.witness_gate_rule.label())
            }
        },
    ));
    println!(
        "E7TIMING name=elapsed scope=run states={total_states} elapsed_seconds={:.3} threads={thread_count}",
        run_started.elapsed().as_secs_f64()
    );
    emitter.done();
    OutcomeOfTheBounds::of_the_findings(&findings)
}

#[cfg(test)]
mod root_choice_fault_tests;
