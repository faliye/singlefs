//! 第 4 次跑的影子账家族、被抛弃根槽的阳性对照与第四段、第五段。

use super::*;

/// 被抛弃根独占的槽（起始镜像上：它引用的落点里，不被任何一条没被抛弃的可读根引用的那几个）。
fn exclusive_slots_of_an_abandoned_root(
    pool: &MemoryPool,
    abandoned: TimelineRoot,
) -> Result<u64, String> {
    let geometry = independent_geometry(pool)?;
    let abandoned_set = abandoned_roots_on(pool)?;
    let record_of = |root: TimelineRoot| {
        root_record_of(pool, &geometry, root).ok_or(format!("根 {root:?} 的记录读不出"))
    };
    let referenced_by_it = placements_referenced_by_root_on(pool, &record_of(abandoned)?)?;
    let mut referenced_by_others = BTreeSet::new();
    for root in readable_roots_independent(pool, &geometry) {
        if root == abandoned || abandoned_set.contains(&root) {
            continue;
        }
        if let Ok(placements) = placements_referenced_by_root_on(pool, &record_of(root)?) {
            referenced_by_others.extend(placements);
        }
    }
    Ok(referenced_by_it.difference(&referenced_by_others).count() as u64)
}

/// L5 与 PC-554（起始镜像同一份，今天那一臂上造）：每一片的两份按 O(m 只)、O(m 起) 各一格（m = 1..M），另打这一臂自己在孪生镜像上
/// 那次不注入的挂载（隔离总数与每一片被读的次数，Q6 与 PC-22 丙 那一句要）；PC-554：那条被抛弃根自己的根槽 W 读坏。
fn run_fourth_run_shadow_ledger_families(
    geometry: &Geometry,
    arm: FourthRunArm,
    summary: &mut FourthRunLegalAndFaultStateTally,
) {
    let parameters = parameters_for(geometry);
    let mut constructed_bases = 0u64;
    let mut pc_554_done = false;
    for overwrites in FOURTH_RUN_CELL_22_OVERWRITES {
        for replay_break in [
            ReplayBreak::EveryRecord,
            ReplayBreak::FirstCopyOfTheFirstNamedUnit,
        ] {
            let coordinates = format!("n1={overwrites} b={}", replay_break.name());
            let key = cell_22_base_key(geometry, overwrites, replay_break);
            let adopted =
                legal_or_fault_state_prefix(geometry, overwrites, arm).and_then(|mut history| {
                    let page_reads = adopt_today_base(&mut history, &key, arm)?;
                    Ok((history, page_reads))
                });
            let (history, page_reads) = match adopted {
                Ok(adopted) => adopted,
                Err(error) => {
                    emit_result(&format!(
                        "name=r4_l_cell family=l5 arm={} geometry={} {coordinates} verdict=stop reason={error:?}",
                        arm.name, geometry.label
                    ));
                    continue;
                }
            };
            constructed_bases += 1;
            let base = history.pool.clone();
            let counting_faults = match instance_table_page_faults_of_the_newest_root(
                &base,
                FaultDuration::FromTheNthRead(u64::MAX),
            ) {
                Ok(faults) => faults,
                Err(error) => {
                    emit_result(&format!(
                        "name=r4_l_cell family=l5 arm={} geometry={} {coordinates} verdict=stop reason={error:?}",
                        arm.name, geometry.label
                    ));
                    continue;
                }
            };
            let flat: Vec<ThirdRunFault> = counting_faults.iter().flatten().cloned().collect();
            let twin = run_fourth_run_mount_writable(&base, &parameters, &flat, None, None);
            let twin_summary = TestedMountSummary::of(&twin.call);
            let own_page_reads: Vec<u64> = counting_faults
                .iter()
                .scan(0usize, |start, copies| {
                    let reads: u64 = twin.reads_of_the_targets[*start..*start + copies.len()]
                        .iter()
                        .sum();
                    *start += copies.len();
                    Some(reads)
                })
                .collect();
            let twin_isolated = twin_summary.isolated;
            emit_result(&format!(
                "name=r4_l5_twin arm={} geometry={} {coordinates} base_fingerprint={:016x} today_page_reads={page_reads:?} own_page_reads={own_page_reads:?} twin_class={} twin_isolated={} twin_abandoned_roots_unreadable={}",
                arm.name,
                geometry.label,
                image_fingerprint(&base),
                twin_summary.class,
                twin_isolated.map_or("none".to_string(), |count| count.to_string()),
                twin_summary
                    .abandoned_roots_unreadable
                    .map_or("none".to_string(), |count| count.to_string())
            ));
            for (page, reads) in page_reads.iter().enumerate() {
                for nth in 1..=*reads {
                    for (duration_name, duration) in [
                        ("only", FaultDuration::OnlyTheNthRead(nth)),
                        ("from", FaultDuration::FromTheNthRead(nth)),
                    ] {
                        let Ok(page_faults) =
                            instance_table_page_faults_of_the_newest_root(&base, duration)
                        else {
                            continue;
                        };
                        let Some(faults) = page_faults.get(page) else {
                            continue;
                        };
                        let cell_coordinates = format!(
                            "{coordinates} page={page} m={nth} duration=O{nth}{duration_name}"
                        );
                        if let Some(tested) = run_fourth_run_legal_or_fault_state_cell(
                            "l5",
                            arm,
                            geometry,
                            &cell_coordinates,
                            &base,
                            faults,
                            &format!(
                                "twin_isolated={}",
                                twin_isolated.map_or("none".to_string(), |count| count.to_string())
                            ),
                            summary,
                        ) {
                            let not_covered = tested.call.call.outcome.is_ok()
                                && matches!((tested.summary.isolated, twin_isolated), (Some(isolated), Some(isolated_on_the_twin)) if isolated < isolated_on_the_twin);
                            emit_result(&format!(
                                "name=r4_q6_cell arm={} geometry={} {cell_coordinates} ok={} isolated={} twin_isolated={} abandoned_roots_unreadable={} q6_not_covered={not_covered}",
                                arm.name,
                                geometry.label,
                                tested.call.call.outcome.is_ok(),
                                tested.summary.isolated.map_or("none".to_string(), |count| count.to_string()),
                                twin_isolated.map_or("none".to_string(), |count| count.to_string()),
                                tested
                                    .summary
                                    .abandoned_roots_unreadable
                                    .map_or("none".to_string(), |count| count.to_string())
                            ));
                        }
                    }
                }
            }
            if !pc_554_done && geometry.label == GEOMETRY_PRIMARY.label {
                pc_554_done = run_fourth_run_positive_control_abandoned_root_slot(
                    arm,
                    &history,
                    &coordinates,
                    summary,
                );
            }
        }
    }
    let registered = 3 * 2;
    emit_result(&format!(
        "name=r4_section_seven_two_anchor arm={} geometry={} item=\"L5 起始镜像 3×2\" computed={constructed_bases} registered={registered} verdict={}",
        arm.name,
        geometry.label,
        if constructed_bases == registered { "pass" } else { "fail_v4" }
    ));
    if !pc_554_done && geometry.label == GEOMETRY_PRIMARY.label {
        emit_fourth_run_positive_control(
            arm.name,
            "PC-554",
            "n1=none b=none",
            &[sentence(
                SentenceTag::TodayDescription,
                "an_abandoned_root_with_exclusive_slots",
                SentenceVerdict::NotConstructible,
                "F1",
            )],
            "reason=\"L5 的六份起始镜像上都找不到独占槽个数 > 0 的被抛弃根\"",
        );
    }
}

/// PC-554（第 3 次跑 5.4 PC-554 行，标签照第 4 次跑 5.4）：起始镜像同 L5；被测挂载时那条被抛弃根自己的根槽 W 读坏（`read_fails`）。
/// 独占的槽个数 = 0 就交回 false（顺延到下一个 n1 与 b）。
fn run_fourth_run_positive_control_abandoned_root_slot(
    arm: FourthRunArm,
    history: &ThirdRunHistory,
    coordinates: &str,
    summary: &mut FourthRunLegalAndFaultStateTally,
) -> bool {
    let base = &history.pool;
    let Ok(abandoned) = abandoned_roots_on(base) else {
        return false;
    };
    let Some(abandoned_root) = abandoned
        .iter()
        .copied()
        .max_by_key(|(instance, txg)| (*txg, *instance))
    else {
        return false;
    };
    let exclusive = exclusive_slots_of_an_abandoned_root(base, abandoned_root).unwrap_or(0);
    if exclusive == 0 {
        return false;
    }
    let Ok(pool_geometry) = independent_geometry(base) else {
        return false;
    };
    let Some((device, offset)) = root_slot_of(base, &pool_geometry, abandoned_root) else {
        return false;
    };
    let faults = vec![ThirdRunFault {
        device,
        offset,
        length: u64::from(pool_geometry.physical_block_size),
        form: UnreadableForm::ReadFails,
        duration: FaultDuration::WholeCall,
        label: format!("abandoned_root{}", key_text(Some(abandoned_root))),
    }];
    let Some(tested) = run_fourth_run_legal_or_fault_state_cell(
        "pc554",
        arm,
        &GEOMETRY_PRIMARY,
        coordinates,
        base,
        &faults,
        "",
        summary,
    ) else {
        return false;
    };
    let ok_count_zero_isolated_zero = tested.call.call.outcome.is_ok()
        && tested.summary.abandoned_roots_unreadable == Some(0)
        && tested.summary.isolated == Some(0);
    let sentences = vec![match (arm.candidate, arm.witness) {
        (FourthRunCandidate::Today, _) | (_, FourthRunWitness::NotJudged) => sentence(
            SentenceTag::TodayDescription,
            "ok_count_zero_isolated_zero",
            SentenceVerdict::of(ok_count_zero_isolated_zero),
            "F1",
        ),
        (_, FourthRunWitness::UnreadableRootRingSlot) => sentence(
            SentenceTag::ArmDefinition,
            "slot_arm_refused",
            SentenceVerdict::of(
                tested.summary.member == arm.refusal_member() && tested.summary.writes == 0,
            ),
            "F9",
        ),
        (
            _,
            FourthRunWitness::SystemConfiguration | FourthRunWitness::SystemConfigurationCarried,
        ) => sentence(
            SentenceTag::ArmDefinition,
            "same_outcome_as_today_ok_count_zero_isolated_zero",
            SentenceVerdict::of(ok_count_zero_isolated_zero),
            "F12",
        ),
    }];
    emit_fourth_run_positive_control(
        arm.name,
        "PC-554",
        coordinates,
        &sentences,
        &format!(
            "abandoned_root={} exclusive_slots={exclusive} truth_newer_on_the_twin={} {}",
            key_text(Some(abandoned_root)),
            tested.judgement.truth_newer,
            render_fourth_run_tested_mount(&tested)
        ),
    );
    true
}

pub(super) fn run_fourth_run_segment_four(arm: FourthRunArm) {
    let mut summary = FourthRunLegalAndFaultStateTally::default();
    run_fourth_run_legal_state_families(&GEOMETRY_PRIMARY, arm, &mut summary);
    run_fourth_run_fault_state_families(
        &GEOMETRY_PRIMARY,
        arm,
        &["l2", "l3", "l4", "l6"],
        &mut summary,
    );
    run_fourth_run_shadow_ledger_families(&GEOMETRY_PRIMARY, arm, &mut summary);
    emit_fourth_run_scratch_findings("l", arm.name, &summary.counts);
    // 7.2（第 3 次跑那一行）：L0 格数 4 × 2；L3 4 × 3 × 2；L4 4 × 2 × 2；L6 4 × 2（按实际跑到被测挂载的格数）。
    for (family, registered) in [
        ("l0", 4 * 2),
        ("l3", 4 * 3 * 2),
        ("l4", 4 * 2 * 2),
        ("l6", 4 * 2),
    ] {
        let computed = summary
            .cells
            .get(&format!("{family}:{}", GEOMETRY_PRIMARY.label))
            .copied()
            .unwrap_or(0);
        emit_result(&format!(
            "name=r4_section_seven_two_anchor arm={} item=\"{family} 格数\" computed={computed} registered={registered} verdict={}",
            arm.name,
            if computed == registered { "pass" } else { "fail_v4" }
        ));
    }
}

// ---------------------------------------------------------------------------
// 12.10 第五段：8.2 的敏感性点（G_S4、G_S16、G_默认、G_小环、k = 2、R = 3），只重跑判决格（登记 8.2 的取样点表）。
// ---------------------------------------------------------------------------

/// mkfs 默认环长的几何点 G_默认 = (S = 8, 768 MiB, 4 GiB)（登记 8.2）。
const GEOMETRY_DEFAULT_RING: Geometry = Geometry {
    label: "G_default(S=8,ring=768MiB)",
    slots_per_region: 8,
    journal_ring_bytes: 768 * 1024 * 1024,
};
/// 小环的取法（第 3 次跑登记 8.2 末）：从 48 KiB 起、每次加 16 KiB。上界是装置自己的停手线（取不到记「这一点取不到」）。
const SMALL_RING_START_BYTES: u64 = 48 * 1024;
const SMALL_RING_STEP_BYTES: u64 = 16 * 1024;
const SMALL_RING_LAST_TRIED_BYTES: u64 = 3 * 1024 * 1024;

/// G_小环：第一个 mkfs 收、且 P(3, U) 接 H1e(m = 4)（不注入）整段跑得通的环长。
fn small_ring_geometry() -> Result<Geometry, String> {
    let mut ring_bytes = SMALL_RING_START_BYTES;
    while ring_bytes <= SMALL_RING_LAST_TRIED_BYTES {
        let label: &'static str =
            Box::leak(format!("G_small_ring(S=8,ring={ring_bytes})").into_boxed_str());
        let geometry = Geometry {
            label,
            slots_per_region: 8,
            journal_ring_bytes: ring_bytes,
        };
        let parameters = parameters_for(&geometry);
        let runs_through =
            fourth_run_plain_prefix(&geometry, 3, Closing::NormalUnmount, "small-ring-search")
                .and_then(|prefix| {
                    let mut history = prefix.history;
                    fourth_run_mount_writable_into_the_history(&mut history, &parameters)?;
                    for _ in 0..4 {
                        third_run_change_content(&mut history, &parameters)?;
                    }
                    Ok(())
                });
        emit_result(&format!(
            "name=r4_small_ring_try ring_bytes={ring_bytes} runs_through={}",
            runs_through.as_ref().map_or_else(
                |error| format!("no:{}", error_member_of_debug(error)),
                |()| "yes".to_string()
            )
        ));
        if runs_through.is_ok() {
            return Ok(geometry);
        }
        ring_bytes += SMALL_RING_STEP_BYTES;
    }
    Err(format!(
        "{SMALL_RING_START_BYTES}..={SMALL_RING_LAST_TRIED_BYTES} 字节里没有一个环长跑得通"
    ))
}

pub(super) fn run_fourth_run_segment_five(arm: FourthRunArm) {
    if arm.rereads() && arm.reread_rounds > 1 {
        // R = 3（登记 8.2）：乙、乙-窄读 的重读轮数加到 3，时长 T(δ)–T(4δ)，H1d 的 `none`、H1e。
        run_fourth_run_held_once_like_family(
            "h1d-uncut-r3",
            &GEOMETRY_PRIMARY,
            arm,
            1,
            FourthRunContinuation::UncutOnly,
        );
        run_fourth_run_held_once_like_family(
            "h1e-r3",
            &GEOMETRY_PRIMARY,
            arm,
            1,
            FourthRunContinuation::Tail,
        );
        return;
    }
    let mut summary = FourthRunLegalAndFaultStateTally::default();
    for geometry in [&GEOMETRY_SMALLER_ROOT_RING, &GEOMETRY_LARGER_ROOT_RING] {
        run_fourth_run_held_once_like_family(
            "h1d-uncut",
            geometry,
            arm,
            1,
            FourthRunContinuation::UncutOnly,
        );
        run_fourth_run_held_once_like_family("h1e", geometry, arm, 1, FourthRunContinuation::Tail);
        run_fourth_run_fault_state_families(geometry, arm, &["l2"], &mut summary);
        run_fourth_run_shadow_ledger_families(geometry, arm, &mut summary);
    }
    run_fourth_run_held_once_like_family(
        "h1d-uncut",
        &GEOMETRY_DEFAULT_RING,
        arm,
        1,
        FourthRunContinuation::UncutOnly,
    );
    run_fourth_run_held_once_like_family(
        "h1e",
        &GEOMETRY_DEFAULT_RING,
        arm,
        1,
        FourthRunContinuation::Tail,
    );
    run_fourth_run_fault_state_families(&GEOMETRY_DEFAULT_RING, arm, &["l3"], &mut summary);
    match small_ring_geometry() {
        Ok(geometry) => {
            emit_result(&format!(
                "name=r4_small_ring arm={} geometry={} verdict=found",
                arm.name, geometry.label
            ));
            run_fourth_run_held_once_like_family(
                "h1e",
                &geometry,
                arm,
                1,
                FourthRunContinuation::Tail,
            );
            run_fourth_run_fault_state_families(&geometry, arm, &["l3"], &mut summary);
        }
        Err(error) => emit_result(&format!(
            "name=r4_small_ring arm={} verdict=not_obtainable reason={error:?}",
            arm.name
        )),
    }
    // k = 2（登记 8.2）：A 藏最新 2 条根的根槽，断链落在两条里较旧那一次发布上；H1d 的 `none`、H1e。
    run_fourth_run_held_once_like_family(
        "h1d-uncut-k2",
        &GEOMETRY_PRIMARY,
        arm,
        2,
        FourthRunContinuation::UncutOnly,
    );
    run_fourth_run_held_once_like_family(
        "h1e-k2",
        &GEOMETRY_PRIMARY,
        arm,
        2,
        FourthRunContinuation::Tail,
    );
    emit_fourth_run_scratch_findings("seg5", arm.name, &summary.counts);
}
