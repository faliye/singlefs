//! 第 4 次跑的常量回比与锚点、第一段与模式入口。

use super::*;

// ---------------------------------------------------------------------------
// 12.9 开跑检查（第 4 次跑）：5.6 常量回比（V3）、7.2 锚点（V4）、臂与副本对得上（装置编进的臂代码、R、读缓存收法与环境变量说的臂一致）。
// ---------------------------------------------------------------------------

pub(super) fn fourth_run_local_constants_checks() -> Vec<ConstantCheck> {
    let mut checks = third_run_local_constants_checks();
    checks.extend([
        ConstantCheck {
            name: "根环区域数（槽序点预言用的那一个）ROOT_RING_REGIONS",
            local_value: ROOT_RING_REGIONS_LOCAL,
            crates_value: singlefs_format::ROOT_RING_REGIONS,
        },
        ConstantCheck {
            name: "稀疏盘扇区宽（镜像指纹用的那一个）SECTOR_BYTES",
            local_value: SECTOR_BYTES_LOCAL,
            crates_value: singlefs_verification::memory_pool::SECTOR_BYTES,
        },
    ]);
    checks
}

/// 臂探针：mkfs → 可写挂载 → 首个文件 → 关闭 C → 再可写挂载（不注入），读臂的观测。乙、乙-窄读 每次可写挂载都走重读那一支、
/// 记下 R 与读缓存收法；别的臂两样都是缺省。交回 (探针那次挂载的 K 类, 观测到的 R, 收不收 journal 空槽)。
fn fourth_run_arm_probe() -> Result<(&'static str, Option<FourthRunArmExtras>), String> {
    let history =
        fourth_run_plain_prefix(&GEOMETRY_PRIMARY, 0, Closing::ProcessExit, "probe")?.history;
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let call = run_fourth_run_mount_writable(&history.pool, &parameters, &[], None, None);
    Ok((
        third_run_outcome_class(&call.call.outcome, call.call.write_calls),
        call.extras,
    ))
}

fn run_fourth_run_constants_and_anchors(arm_name: &str) -> bool {
    let mut all_passed = true;
    for check in fourth_run_local_constants_checks() {
        let passed = check.matches();
        all_passed &= passed;
        emit_result(&format!(
            "name=r4_local_constant_check arm={arm_name} item={:?} verdict={} local_value={} crates_value={}",
            check.name,
            verdict_word(passed),
            check.local_value,
            check.crates_value
        ));
    }
    let (regions_match, local_regions, crates_regions) = region_devices_matches_crates();
    all_passed &= regions_match;
    emit_result(&format!(
        "name=r4_local_constant_check arm={arm_name} item=region_devices verdict={} local_value={local_regions:?} crates_value={crates_regions:?}",
        verdict_word(regions_match)
    ));
    let arm = fourth_run_arm_named(arm_name);
    let arm_code_compiled_in = third_run_arm_hooks::arm_code_is_compiled_in();
    let probe = fourth_run_arm_probe();
    let arm_code_matches = match (arm, &probe) {
        (Some(arm), Ok((class, extras))) => {
            let extras_match = match extras {
                None => arm.is_today(),
                Some(extras) => {
                    !arm.is_today()
                        && u64::from(extras.reread_rounds_limit) == arm.reread_rounds
                        && extras.zeroed_journal_ring_slots_are_cached
                            == (arm.candidate == FourthRunCandidate::YiNarrow)
                }
            };
            *class == "K0" && arm.is_today() != arm_code_compiled_in && extras_match
        }
        (None, _) | (_, Err(_)) => false,
    };
    all_passed &= arm_code_matches;
    emit_result(&format!(
        "name=r4_arm_code_check arm={arm_name} verdict={} arm_known={} candidate={} witness={} registered_rounds={} arm_code_compiled_in={arm_code_compiled_in} probe={}",
        verdict_word(arm_code_matches),
        arm.is_some(),
        arm.map_or("none", FourthRunArm::candidate_name),
        arm.map_or("none", FourthRunArm::witness_name),
        arm.map_or(0, |arm| arm.reread_rounds),
        match &probe {
            Ok((class, extras)) => format!("{class}:{extras:?}").replace(' ', ""),
            Err(error) => format!("error:{}", error.replace(' ', "_")),
        }
    ));
    match decoded_tail_instance_and_floor_offsets() {
        Ok((tail, instance, floor)) => {
            let passed = tail == SYSTEM_CONFIGURATION_TAIL_OFFSET_LOCAL
                && instance == tail + 8
                && floor == SYSTEM_CONFIGURATION_TAIL_OFFSET_LOCAL + 8 + 4
                && instance == 477
                && floor == 481;
            all_passed &= passed;
            emit_result(&format!(
                "name=r4_section_seven_two_anchor arm={arm_name} item=\"journal 实例代号偏移 469 + 8；F 偏移 489 − 8\" verdict={} tail_offset={tail} instance_offset={instance} floor_offset={floor} registered_instance=477 registered_floor=481",
                verdict_word(passed)
            ));
        }
        Err(error) => {
            all_passed = false;
            emit_result(&format!(
                "name=r4_section_seven_two_anchor arm={arm_name} item=offsets verdict=fail reason={error:?}"
            ));
        }
    }
    for geometry in [
        &GEOMETRY_PRIMARY,
        &GEOMETRY_SMALLER_ROOT_RING,
        &GEOMETRY_LARGER_ROOT_RING,
    ] {
        let parameters = parameters_for(geometry);
        let mut devices = new_devices(IMAGE_BYTES);
        let anchor = make_filesystem(&parameters, &mut devices)
            .map_err(|error| format!("{error:?}"))
            .and_then(|_| independent_geometry(&memory_pool_of(&devices, IMAGE_BYTES)));
        match anchor {
            Ok(pool_geometry) => {
                let pool = memory_pool_of(&devices, IMAGE_BYTES);
                let root_slots = checker_image::root_slot_positions(&pool_geometry).len() as u64;
                let ring_slots = pool_geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES;
                let configuration_slots = device_decode(&pool, &pool_geometry)
                    .system_configuration_slots
                    .len() as u64;
                for (item, computed, registered) in [
                    (
                        "根槽总数 3×S（装置在这个几何上枚举的槽数）",
                        root_slots,
                        ROOT_RING_REGIONS_LOCAL * geometry.slots_per_region,
                    ),
                    (
                        "环槽数 环÷4096（装置在这个几何上的环槽数）",
                        ring_slots,
                        geometry.journal_ring_bytes / 4096,
                    ),
                    (
                        "每个池的系统配置槽数 2×2（装置读到的自证槽数）",
                        configuration_slots,
                        2 * 2,
                    ),
                ] {
                    let passed = computed == registered;
                    all_passed &= passed;
                    emit_result(&format!(
                        "name=r4_section_seven_two_anchor arm={arm_name} geometry={} item={item:?} verdict={} computed={computed} registered={registered}",
                        geometry.label,
                        verdict_word(passed)
                    ));
                }
            }
            Err(error) => {
                all_passed = false;
                emit_result(&format!(
                    "name=r4_section_seven_two_anchor arm={arm_name} geometry={} item=geometry verdict=fail reason={error:?}",
                    geometry.label
                ));
            }
        }
    }
    let registered_pieces = [
        (
            "G0 上 3×S（PC-多读 (ii) 里 -槽 臂的点值）",
            ROOT_RING_REGIONS_LOCAL * GEOMETRY_PRIMARY.slots_per_region,
            24u64,
        ),
        (
            "H1f 每臂前缀数 n1a×c×n1b",
            (FOURTH_RUN_REMOUNT_PREFIX_FIRST_OVERWRITES.len()
                * 2
                * FOURTH_RUN_REMOUNT_PREFIX_SECOND_OVERWRITES.len()) as u64,
            8,
        ),
        (
            "H1g 每臂格数 n1×a×m×b×造法",
            (FOURTH_RUN_ACQUISITION_CRASH_PREFIX_OVERWRITES.len()
                * FOURTH_RUN_ACQUISITION_CRASHES.len()
                * FOURTH_RUN_ACQUISITION_CRASH_TAIL_OVERWRITES.len()
                * 2
                * 2) as u64,
            32,
        ),
    ];
    for (item, computed, registered) in registered_pieces {
        let passed = computed == registered;
        all_passed &= passed;
        emit_result(&format!(
            "name=r4_section_seven_two_anchor arm={arm_name} item={item:?} verdict={} computed={computed} registered={registered}",
            verdict_word(passed)
        ));
    }
    all_passed
}

fn fourth_run_arm_label() -> String {
    env::var("SINGLEFS_E158_ARM").unwrap_or_else(|_| "today".to_string())
}

/// 第一段（登记 5.7）：阳性对照里不靠 L 族的那几条（PC-N-环、PC-N-盖、PC-O、PC-丢写、PC-只读，PC-多读 在 compare）、
/// PC-N0 与 PC-多读 (ii)（L0 那 8 格），H1d、H1e（G0），Q0 去向栏。
fn run_fourth_run_segment_one(arm: FourthRunArm) {
    run_fourth_run_positive_control_newer(
        &GEOMETRY_PRIMARY,
        arm,
        ReplayBreak::FirstCopyOfTheFirstNamedUnit,
    );
    run_fourth_run_positive_control_newer(&GEOMETRY_PRIMARY, arm, ReplayBreak::EveryRecord);
    run_fourth_run_positive_control_second_read(&GEOMETRY_PRIMARY, arm);
    run_fourth_run_positive_control_no_newer(&GEOMETRY_PRIMARY, arm);
    run_fourth_run_held_once_like_family(
        "h1d",
        &GEOMETRY_PRIMARY,
        arm,
        1,
        FourthRunContinuation::EveryCut,
    );
    run_fourth_run_held_once_like_family(
        "h1e",
        &GEOMETRY_PRIMARY,
        arm,
        1,
        FourthRunContinuation::Tail,
    );
}

/// H1d / H1e 在一个几何点上：前缀 P(n1, C)，n1 ∈ {0, 1, 2, 3}；H1e 的 m ∈ {1, 2, 3, 4}。
pub(super) fn run_fourth_run_held_once_like_family(
    name: &str,
    geometry: &Geometry,
    arm: FourthRunArm,
    hidden_count: usize,
    continuation: FourthRunContinuation,
) {
    let clause_checks = if hidden_count == 1
        && geometry.label == GEOMETRY_PRIMARY.label
        && matches!(name, "h1d" | "h1e")
    {
        ClauseChecksOnTheHiddenRoot::Checked
    } else {
        ClauseChecksOnTheHiddenRoot::NotRegisteredForThisFamily
    };
    let family = FourthRunFamily {
        name,
        geometry,
        hidden_count,
        continuation,
        tail_overwrites: &THIRD_RUN_TAIL_OVERWRITES,
        durations: FourthRunDurationPlan::OfTheArm,
        checks_the_clauses_on_the_hidden_root: clause_checks,
    };
    let prefixes = THIRD_RUN_OVERWRITES
        .iter()
        .map(|overwrites| {
            fourth_run_plain_prefix(geometry, *overwrites, Closing::ProcessExit, arm.name)
        })
        .collect();
    run_fourth_run_family(&family, arm, prefixes);
}

/// 第 4 次跑的模式：开跑检查不过就停（V3 / V4），过了跑点名的那一段。交回 true = 这个模式归第 4 次跑。
pub(super) fn run_fourth_run_mode(mode: &str, command_line_arguments: &[String]) -> bool {
    let known = [
        "r4-constants",
        "r4-seg1",
        "r4-pc",
        "r4-h1d",
        "r4-h1e",
        "r4-compare",
        "r4-bases",
        "r4-seg3",
        "r4-seg4",
        "r4-seg5",
        "r4-h1f",
        "r4-h1g",
        "r4-seg2",
    ];
    if !known.contains(&mode) {
        return false;
    }
    emit_input_header();
    if mode == "r4-compare" {
        let prefixes: Vec<String> = command_line_arguments.iter().skip(2).cloned().collect();
        run_fourth_run_compare(&prefixes);
        emit_result(&format!(
            "name=done emitted={}",
            emitted_result_line_count() + 1
        ));
        return true;
    }
    let arm_name = fourth_run_arm_label();
    if !run_fourth_run_constants_and_anchors(&arm_name) {
        emit_result(&format!(
            "name=stop arm={arm_name} reason=V3 detail=\"第 4 次跑的常量回比、第七节锚点或臂与副本的对应有不过的，整轮不开跑\""
        ));
        emit_result(&format!(
            "name=done emitted={}",
            emitted_result_line_count() + 1
        ));
        return true;
    }
    let arm = fourth_run_arm_named(&arm_name).expect("开跑检查里核过臂名");
    match mode {
        "r4-constants" => {}
        "r4-seg1" => run_fourth_run_segment_one(arm),
        "r4-pc" => {
            run_fourth_run_positive_control_newer(
                &GEOMETRY_PRIMARY,
                arm,
                ReplayBreak::FirstCopyOfTheFirstNamedUnit,
            );
            run_fourth_run_positive_control_newer(&GEOMETRY_PRIMARY, arm, ReplayBreak::EveryRecord);
            run_fourth_run_positive_control_second_read(&GEOMETRY_PRIMARY, arm);
            run_fourth_run_positive_control_no_newer(&GEOMETRY_PRIMARY, arm);
        }
        "r4-h1d" => run_fourth_run_held_once_like_family(
            "h1d",
            &GEOMETRY_PRIMARY,
            arm,
            1,
            FourthRunContinuation::EveryCut,
        ),
        "r4-h1e" => run_fourth_run_held_once_like_family(
            "h1e",
            &GEOMETRY_PRIMARY,
            arm,
            1,
            FourthRunContinuation::Tail,
        ),
        "r4-bases" => run_fourth_run_today_bases(arm),
        "r4-seg3" => run_fourth_run_segment_three(arm),
        "r4-seg4" => run_fourth_run_segment_four(arm),
        "r4-seg5" => run_fourth_run_segment_five(arm),
        "r4-h1f" => run_fourth_run_remount_families(arm),
        "r4-h1g" => run_fourth_run_acquisition_crash_family(arm),
        "r4-seg2" => fourth_run_segment_two::run_segment_two(arm),
        other => unreachable!("第 4 次跑的模式表 known 已经挡过：{other}"),
    }
    emit_result(&format!(
        "name=done emitted={}",
        emitted_result_line_count() + 1
    ));
    true
}

#[cfg(test)]
mod fourth_run_tests;
