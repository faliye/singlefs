//! 带读故障一族的阳性对照（登记 5.6）、产物行与本地常量回比（r4 新读的三样，不符走停机条款 TH2）。

use super::*;

// ── 阳性对照（登记 5.6）：假结局读数与钉值比，走同一段判据；产物里打 `name=control` 行

#[derive(Clone, Debug)]
pub(super) struct FaultControlReading {
    pub(super) criterion: FaultCriterion,
    pub(super) measured_where: String,
    pub(super) value: u64,
    pub(super) must_see: String,
    pub(super) is_red_as_required: bool,
}

fn fault_reading_that_must_equal(
    criterion: FaultCriterion,
    measured_where: String,
    value: u64,
    expected: u64,
) -> FaultControlReading {
    FaultControlReading {
        criterion,
        measured_where,
        value,
        must_see: format!("equals_{expected}"),
        is_red_as_required: value == expected,
    }
}

/// R2 的拒因行（登记 r5 修订 1 修1.五第 4 条、修1.七乙类「拒因分计」）：被拒的挂载按闸（重读那一遍仍真）、取号那一刻 (b)（某块盘一份自证过的都没有）、
/// 取号那一刻 (a)（判不出）三栏分；三栏之和等于被拒的挂载数（[`assert_the_pinned_fault_cell_under_the_gate`] 断言）。
pub(super) fn refusal_row_of_a_fault_cell(
    domain: FaultDomain,
    cell: RootChoiceCell,
    tally: &FaultTally,
) -> String {
    let columns: Vec<String> = [
        MountRefusalReason::TheGateAfterTheOneReread,
        MountRefusalReason::AcquisitionDeviceWithoutASelfVerifiedSlot,
        MountRefusalReason::AcquisitionWitnessUndecidable,
    ]
    .iter()
    .map(|reason| {
        format!(
            "refused_{}={}",
            reason.label(),
            tally.refusals_by_reason[reason.index()]
        )
    })
    .collect();
    format!(
        "name=root_choice_fault_refusals {} refused_mounts={} {}",
        fault_place_of(domain, cell),
        tally.refused_mounts,
        columns.join(" ")
    )
}

pub(super) fn fault_place_of(domain: FaultDomain, cell: RootChoiceCell) -> String {
    format!(
        "domain={} ring_slots={} slots_per_region={} mounts={} user_publishes={}",
        domain.label(),
        domain.geometry().ring().ring_slots,
        domain.geometry().slots_per_region(),
        cell.mounts,
        cell.user_publishes
    )
}

/// 一个合法历史或 KG 格上每条判据的对照读数（登记 5.6 表）：KJ0 在 KF M = 1、U = 0 指定一态上 panic（违例数 1）；KJ1、KJ1T、KJ2 在 KF1 下等于第七节 KF1 三列；
/// KJ1、KJ1T、KJ3 在 KF2 下各等于「有已确认的被判态数」，KJ2 等于第七节 KF2 KJ2 一列；KJ1、KJ1T 在 KF4 下等于第七节 KF4 两列。
/// 闸取甲（只给单测用，跑法里一律带闸的取法）。
#[cfg(test)]
fn fault_control_readings_of_a_cell(
    domain: FaultDomain,
    cell: RootChoiceCell,
    tally: &FaultTally,
) -> Vec<FaultControlReading> {
    fault_control_readings_of_a_cell_under_the_gate(
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable,
        domain,
        cell,
        tally,
    )
}

/// 一个合法历史或 KG 格上每条判据的对照读数，按闸的取法取钉值：取甲照 r4 登记第七节，R2 照 r5 登记修订 1 修1.七乙类（判据那一套同 r4 登记 5.6）。
pub(super) fn fault_control_readings_of_a_cell_under_the_gate(
    witness_gate_rule: WitnessGateRule,
    domain: FaultDomain,
    cell: RootChoiceCell,
    tally: &FaultTally,
) -> Vec<FaultControlReading> {
    let place = fault_place_of(domain, cell);
    let pinned = pinned_fault_cell_under_the_gate(witness_gate_rule, domain, cell)
        .expect("每个跑到的格都在登记第七节乙类的表里");
    let mut readings = Vec::new();
    if domain == FaultDomain::GateOn && cell.mounts == 1 && cell.user_publishes == 0 {
        readings.push(fault_reading_that_must_equal(
            FaultCriterion::Kj0Defined,
            format!("{place} fake_outcome=panics_on_the_designated_state"),
            tally.kj0_control_violations,
            1,
        ));
    }
    let oldest = tally.local_rules[FAULT_OLDEST_RULE_INDEX];
    for (criterion, position) in [
        (FaultCriterion::Kj1NotEarlierThanTheLastConfirmedPublish, 0),
        (FaultCriterion::Kj1tDescendsFromTheLastConfirmedPublish, 1),
        (FaultCriterion::Kj2NotEarlierThanAnyReadableRoot, 2),
    ] {
        readings.push(fault_reading_that_must_equal(
            criterion,
            format!("{place} fake_outcome=KF1_chooses_the_oldest_root"),
            oldest[position],
            pinned.kf1[position],
        ));
    }
    for (criterion, position, expected) in [
        (
            FaultCriterion::Kj1NotEarlierThanTheLastConfirmedPublish,
            0,
            pinned.confirmed_states,
        ),
        (
            FaultCriterion::Kj1tDescendsFromTheLastConfirmedPublish,
            1,
            pinned.confirmed_states,
        ),
        (
            FaultCriterion::Kj2NotEarlierThanAnyReadableRoot,
            2,
            pinned.kf2[2],
        ),
        (
            FaultCriterion::Kj3LandsOnTheIdealRecovery,
            3,
            pinned.confirmed_states,
        ),
    ] {
        readings.push(fault_reading_that_must_equal(
            criterion,
            format!("{place} fake_outcome=KF2_lands_on_the_parent_of_the_last_confirmed"),
            tally.kf2[position],
            expected,
        ));
    }
    for (criterion, position) in [
        (FaultCriterion::Kj1NotEarlierThanTheLastConfirmedPublish, 0),
        (FaultCriterion::Kj1tDescendsFromTheLastConfirmedPublish, 1),
    ] {
        readings.push(fault_reading_that_must_equal(
            criterion,
            format!("{place} fake_outcome=KF4_lands_on_the_newest_root_off_the_confirmed_chain"),
            tally.kf4[position],
            pinned.kf4[position],
        ));
    }
    readings
}

/// KL 上本地写的六种择根序（五种加平局取后者赢）的 KJ2 违例数必须等于第 7 次重跑登记 `research/prompts/e165-r7-prereg.md` 第七节乙类 B2：
/// 今天的序 0、平局取低 0、实例代号为主 6、比较不看实例 0、取第一个有效者 4、平局取后者赢 0。
const PINNED_KL_KJ2_OF_THE_LOCAL_RULES: [(FaultLocalRule, u64); 6] = [
    (FaultLocalRule::Today, 0),
    (FaultLocalRule::TieTakesTheLowerInstance, 0),
    (FaultLocalRule::InstanceFirstThenTxg, 6),
    (FaultLocalRule::IgnoresTheInstance, 0),
    (FaultLocalRule::FirstSelfVerifiedWins, 4),
    (FaultLocalRule::TieTakesTheLater, 0),
];

/// KL：取 r3 档同一个生成函数（[`landing_slot_adversarial_root_rings`]）交出的 27 个根环，本地写的一种择根序在上面 KJ2 违例几次。
fn kl_kj2_violations_of_a_local_rule(rule: FaultLocalRule) -> u64 {
    let mut violations = 0;
    for ring in landing_slot_adversarial_root_rings() {
        let roots_in_ring_order: Vec<Option<&RootChoicePublishMaterial>> =
            ring.root_slots.iter().map(|slot| slot.as_deref()).collect();
        let newest_version = roots_in_ring_order
            .iter()
            .copied()
            .flatten()
            .map(|material| material.version)
            .max();
        if let (Some(chosen), Some(newest)) =
            (rule.chooses_among(&roots_in_ring_order), newest_version)
        {
            violations += u64::from(chosen.version < newest);
        }
    }
    violations
}

/// KL 上每种本地择根序的对照读数（KJ2）：必须等于 [`PINNED_KL_KJ2_OF_THE_LOCAL_RULES`]。
pub(super) fn fault_kl_control_readings() -> Vec<FaultControlReading> {
    let place = fault_place_of(
        FaultDomain::AdversarialRootRing,
        RootChoiceCell {
            mounts: 0,
            user_publishes: 0,
        },
    );
    PINNED_KL_KJ2_OF_THE_LOCAL_RULES
        .iter()
        .map(|(rule, expected)| {
            fault_reading_that_must_equal(
                FaultCriterion::Kj2NotEarlierThanAnyReadableRoot,
                format!("{place} local_choice={}", rule.label()),
                kl_kj2_violations_of_a_local_rule(*rule),
                *expected,
            )
        })
        .collect()
}

// ── 产物行

/// 承诺内（历史罩着 ∧ 被判时罩着 ∧ 不是上限截掉）的违例数，从分格的行里算：罩着的格的违例数减去其中上限截掉那部分（登记 5.5 末段）。
fn promised_violations_from_the_split_rows(
    cells: &[FaultCellTally; FAULT_CELL_COUNT],
    criterion_index: usize,
) -> u64 {
    let mut promised = 0;
    for judged_fault in JudgedFault::ALL {
        if !judged_fault.is_inside_the_single_fault_promise() {
            continue;
        }
        let cell = &cells[fault_cell_index_of(*judged_fault, true)];
        promised += cell.violations[criterion_index] - cell.cut_violations[criterion_index];
    }
    promised
}

pub(super) fn emit_the_rows_of_a_fault_cell(
    emitter: &mut Emitter,
    domain: FaultDomain,
    cell: RootChoiceCell,
    tally: &FaultTally,
) {
    let place = fault_place_of(domain, cell);
    emitter.emit_result_line(&format!(
        "name=root_choice_fault_states {place} histories={} histories_with_an_uncovered_fault={} judged_states={} confirmed_states={} refused_mounts={} reread_mounts={} states_with_a_torn_system_slot={} same_txg_different_instance_roots={} higher_instance_lower_txg_roots={} states_without_a_readable_root={} above_newest_root_max={} in_flight_limit={} ideal_loss_by_limit={} ideal_loss_covered_other={} ideal_loss_uncovered={} promised_states={} confirmed_publishes_max={} largest_counter={} panicked={} choose_root_returned_none_with_a_readable_root={} choose_root_returned_none_without_a_readable_root={} replay_failed={} replay_applied={} kj1_and_kj1t_differing_states={} kj1_and_kj1t_differing_witness={} slot_readings_checked={} slot_readings_differing_from_the_truth={} slot_reading_witness={}",
        tally.histories,
        tally.histories_with_an_uncovered_fault,
        tally.judged_states,
        tally.confirmed_states,
        tally.refused_mounts,
        tally.reread_mounts,
        tally.states_with_a_torn_system_slot,
        tally.states_with_same_txg_different_instance_roots,
        tally.states_with_higher_instance_lower_txg_roots,
        tally.states_without_a_readable_root,
        tally.above_newest_root_maximum,
        domain.geometry().ring().local_in_flight_limit(),
        tally.ideal_loss_by_limit,
        tally.ideal_loss_covered_other,
        tally.ideal_loss_uncovered,
        tally.promised_states,
        tally.confirmed_publishes_maximum,
        tally.largest_counter,
        tally.panicked_states,
        tally.chose_nothing_with_a_readable_root,
        tally.chose_nothing_without_a_readable_root,
        tally.replay_failed_states,
        tally.replay_applied_states,
        tally.kj1_and_kj1t_differing_states,
        tally.kj1_and_kj1t_differing_witness.as_deref().unwrap_or("none"),
        tally.slot_readings_checked,
        tally.slot_readings_differing_from_the_truth,
        tally.slot_reading_witness.as_deref().unwrap_or("none"),
    ));
    let local_rule_fields: Vec<String> = FaultLocalRule::ALL
        .iter()
        .enumerate()
        .map(|(rule_index, rule)| {
            let reading = tally.local_rules[rule_index];
            format!(
                "{}={}/{}/{}",
                rule.label(),
                reading[0],
                reading[1],
                reading[2]
            )
        })
        .collect();
    emitter.emit_result_line(&format!(
        "name=fake_outcomes {place} local_rules_kj1_kj1t_kj2:{} kf2_kj1_kj1t_kj2_kj3={}/{}/{}/{} kf4_kj1_kj1t={}/{} kf4_states_without_a_root_off_the_confirmed_chain={}",
        local_rule_fields.join(","),
        tally.kf2[0],
        tally.kf2[1],
        tally.kf2[2],
        tally.kf2[3],
        tally.kf4[0],
        tally.kf4[1],
        tally.kf4_states_without_a_root_off_the_confirmed_chain,
    ));
    emit_the_split_and_promised_rows(emitter, &place, tally);
    if cell.mounts == 2 {
        for (menu_index, label) in MOUNT_FAULT_MENU_LABELS.iter().enumerate() {
            let count = tally.mount_two[menu_index];
            emitter.emit_result_line(&format!(
                "name=mount_two_fault {place} mount_two_fault={label} histories={} refused={} reread={}",
                count.histories, count.refused, count.reread
            ));
        }
    }
}

/// 每个 (被判时故障, 历史标签) 格每条判据一行，再加承诺内两行（KJ1、KJ1T，装置直接数的与从分格的行算的各报一遍）。
fn emit_the_split_and_promised_rows(emitter: &mut Emitter, place: &str, tally: &FaultTally) {
    for judged_fault in JudgedFault::ALL {
        for history_is_covered in [true, false] {
            let cell_index = fault_cell_index_of(*judged_fault, history_is_covered);
            let cell = &tally.cells[cell_index];
            for criterion in FaultCriterion::ALL {
                let criterion_index = criterion.index();
                let ideal_violations = if criterion_index == KJ1_INDEX {
                    cell.ideal_violations[0].to_string()
                } else if criterion_index == DESCENDS_FROM_THE_LAST_CONFIRMED_PUBLISH_INDEX {
                    cell.ideal_violations[1].to_string()
                } else {
                    "not_applicable".to_string()
                };
                emitter.emit_result_line(&format!(
                    "name=violations {place} judged_fault={} history={} criterion={} states={} confirmed_states={} cut_by_the_limit_states={} judged={} violations={} cut_by_the_limit_violations={} ideal_clause_violations={ideal_violations} witness={}",
                    judged_fault.label(),
                    if history_is_covered { "inside_the_promise" } else { "outside_the_promise" },
                    criterion.label(),
                    cell.states,
                    cell.confirmed_states,
                    cell.cut_states,
                    cell.judged[criterion_index],
                    cell.violations[criterion_index],
                    cell.cut_violations[criterion_index],
                    tally.witness_of_a_cell(cell_index, criterion_index)
                ));
            }
        }
    }
    for (position, criterion_index) in [KJ1_INDEX, DESCENDS_FROM_THE_LAST_CONFIRMED_PUBLISH_INDEX]
        .into_iter()
        .enumerate()
    {
        emitter.emit_result_line(&format!(
            "name=promised {place} criterion={} promised_states={} violations={} violations_from_the_split_rows={} witness={}",
            FaultCriterion::ALL[criterion_index].label(),
            tally.promised_states,
            tally.promised_violations[position],
            promised_violations_from_the_split_rows(&tally.cells, criterion_index),
            tally.promised_witnesses[position].as_deref().unwrap_or("none")
        ));
    }
}

/// 一个域扫完之后的判决行（各判据在这个域全部格上的合计），并记作废与停机。
pub(super) fn emit_the_verdict_of_a_fault_domain(
    emitter: &mut Emitter,
    findings: &mut RunFindings,
    domain: FaultDomain,
    cells: &[RootChoiceCell],
    total: &FaultTally,
    tier: BoundsTier,
) {
    let largest_mounts = cells.iter().map(|cell| cell.mounts).max().unwrap_or(0);
    let largest_user_publishes = cells
        .iter()
        .map(|cell| cell.user_publishes)
        .max()
        .unwrap_or(0);
    // 登记作废条款 TV10：域是空的（没有一个已确认发布的态；KG 上实例代号更高而 txg 更低同在为 0）。
    let domain_is_nonempty = total.confirmed_states > 0
        && (domain != FaultDomain::GateOff
            || total.states_with_higher_instance_lower_txg_roots > 0);
    if !domain_is_nonempty {
        findings.void_conditions.push(format!(
            "TV10 {} 域是空的（没有一个已确认发布的态，或 KG 上实例代号更高而 txg 更低同在为 0），这个域的 0 违例不算数",
            domain.label()
        ));
    }
    // 登记作废条款 TV6：承诺内那一行对不回分格的行。
    let promised_rows_equal_the_split_rows =
        [KJ1_INDEX, DESCENDS_FROM_THE_LAST_CONFIRMED_PUBLISH_INDEX]
            .into_iter()
            .enumerate()
            .all(|(position, criterion_index)| {
                total.promised_violations[position]
                    == promised_violations_from_the_split_rows(&total.cells, criterion_index)
            });
    if !promised_rows_equal_the_split_rows {
        findings.void_conditions.push(format!(
            "TV6 {} 域里「承诺内」那一行的违例数对不回分格的行",
            domain.label()
        ));
    }
    // 登记停机条款 TH4：读槽对拍不一致，不当 KJ 的违例记。
    if total.slot_readings_differing_from_the_truth > 0 {
        findings.halt_conditions.push(format!(
            "TH4 {} 域里 read_root_ring_slot 交回的与生成时的真相不同的槽数 {}（见证 {}）",
            domain.label(),
            total.slot_readings_differing_from_the_truth,
            total.slot_reading_witness.as_deref().unwrap_or("none")
        ));
    }
    // 登记第七节甲类、失败条款 TF9：合法历史的域里今天的 replay_journal 报错的被判态数是 0。
    if domain.is_a_legal_history() && total.replay_failed_states > 0 {
        findings.halt_conditions.push(format!(
            "TF9 {} 域里 replay_journal 报错的被判态数 {}（登记第七节甲类锚点是 0）",
            domain.label(),
            total.replay_failed_states
        ));
    }
    emitter.emit_result_line(&format!(
        "name=verdict domain={} ring_slots={} slots_per_region={} largest_mounts={largest_mounts} largest_user_publishes={largest_user_publishes} bounds={} criteria=root_choice_faults histories={} states={} confirmed_states={} kj0_violations={} panicked={} choose_root_returned_none_with_a_readable_root={} replay_failed={} kj1_violations={} kj1_judged={} kj1t_violations={} kj1t_judged={} kj2_violations={} kj2_judged={} kj3_violations={} kj3_judged={} promised_states={} promised_kj1_violations={} promised_kj1t_violations={} promised_rows_equal_the_split_rows={promised_rows_equal_the_split_rows} kj1_and_kj1t_differing_states={} slot_readings_differing_from_the_truth={} domain_is_nonempty={domain_is_nonempty}",
        domain.label(),
        domain.geometry().ring().ring_slots,
        domain.geometry().slots_per_region(),
        tier.label(),
        total.histories,
        total.judged_states,
        total.confirmed_states,
        total.violations_of_a_criterion(0),
        total.panicked_states,
        total.chose_nothing_with_a_readable_root,
        total.replay_failed_states,
        total.violations_of_a_criterion(KJ1_INDEX),
        total.judged_of_a_criterion(KJ1_INDEX),
        total.violations_of_a_criterion(DESCENDS_FROM_THE_LAST_CONFIRMED_PUBLISH_INDEX),
        total.judged_of_a_criterion(DESCENDS_FROM_THE_LAST_CONFIRMED_PUBLISH_INDEX),
        total.violations_of_a_criterion(KJ2_INDEX),
        total.judged_of_a_criterion(KJ2_INDEX),
        total.violations_of_a_criterion(KJ3_INDEX),
        total.judged_of_a_criterion(KJ3_INDEX),
        total.promised_states,
        total.promised_violations[0],
        total.promised_violations[1],
        total.kj1_and_kj1t_differing_states,
        total.slot_readings_differing_from_the_truth,
    ));
}

// ── 本地常量回比（登记第三节回比表，r4 新读的三样；不符走停机条款 TH2）

pub(super) fn cross_check_root_choice_fault_local_constants_against_the_implementation() {
    cross_check_root_choice_local_constants_against_the_implementation();
    assert_eq!(
        u64::try_from(FAULT_SYSTEM_SLOTS_PER_DEVICE).expect("2 装得进 u64"),
        singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
        "TH2：每块盘的系统配置槽数"
    );
    assert_eq!(
        FAULT_MKFS_SYSTEM_GENERATION,
        singlefs_core::make_filesystem::SYSTEM_CONFIGURATION_GENERATION_AT_MKFS,
        "TH2：mkfs 写下的系统配置世代号"
    );
    assert_eq!(
        FAULT_WRITABLE_DEVICE_COUNT_LOWER_BOUND,
        singlefs_core::mount::STRIPE_WIDTH_LOWER_BOUND.0,
        "TH2：可写挂载的设备数下限"
    );
    const {
        assert!(
            FAULT_DEVICE_COUNT >= FAULT_WRITABLE_DEVICE_COUNT_LOWER_BOUND,
            "TH2：两块盘装得下可写挂载的设备数下限（历史里的可写挂载在盘上都不是空操作）"
        );
    }
    assert_eq!(
        FAULT_SYSTEM_SLOT_COUNT_OF_THE_POOL,
        FAULT_DEVICE_COUNT * FAULT_SYSTEM_SLOTS_PER_DEVICE,
        "整池的系统配置槽数"
    );
}

#[cfg(test)]
mod root_choice_fault_tests;
