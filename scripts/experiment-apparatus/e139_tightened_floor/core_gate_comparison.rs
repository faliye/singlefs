//! 成对行的取字段与比较、核心门臂的跨臂对照、到达与阳性对照与储备扫描的判定、核心门臂的整趟。

use super::*;

/// 第 3 次重跑第一节读法第 2 条：甲类行（随门槛变）的行名去掉族前缀之后的那一段。
const PAIRED_LINE_SUFFIXES: [&str; 17] = [
    "cell", "activation", "activation_h2", "near_full", "drain_failure", "broken_slot", "broken_slot_h1", "cost_growth", "crit1_push", "disk_stuck",
    "disk_stuck_h3", "stuck_growth", "broken_slot_growth", "broken_slot_h1_growth", "stuck_marks", "trajectory", "attribution",
];
/// 同一类行里认「同键」看的字段（另加行名、`line=` 与臂）：世界 / 变体、S、丢盘、失败组合、开销对、开销升到、占用、收尾、卡住的盘、种子、空发布开销。
const ROW_KEY_FIELD_NAMES: [&str; 14] =
    ["world", "variant", "s", "lost_disk", "failures", "c0", "c_max", "grown_to", "within_declared", "fill", "ending", "stuck_disk", "seed", "empty_cost"];
/// 两族臂名的对应：第 2 次（旧那一边）→ CG。前两对是第六节第 6 行的被测对，后两对是阳性对照。
const CORE_GATE_ARM_COUNTERPARTS: [(&str, &str); 4] = [
    ("RGall_ring_all", "RGall_core_gate"),
    ("RGall_Q_ring_all", "RGall_Q_core_gate"),
    ("RGall_short", "RGall_core_gate_short"),
    ("RGall_Q_short", "RGall_Q_core_gate_short"),
];
/// 五个判定位的计数字段（第一节读法第 3 条：C3f、C3u、C4、C5、C7）。
const JUDGEMENT_COUNT_FIELDS: [&str; 5] = ["fakes", "unrecoverable", "false_enospc_arm", "write_after_delete_failed", "retention_violations"];
const SECOND_RERUN_LINE_PREFIX: &str = "f_";
const CORE_GATE_LINE_PREFIX: &str = "core_gate_";
/// 没有恢复、没有回退的那几类行（第十一节 V7）：去掉族前缀之后的行名（轨迹与归因行看它们的 `line=`）。
const LINE_SUFFIXES_WITHOUT_RECOVERY: [&str; 8] =
    ["near_full", "drain_failure", "broken_slot", "broken_slot_h1", "cost_growth", "crit1_push", "broken_slot_growth", "broken_slot_h1_growth"];
const WORLDS_WITHOUT_RECOVERY: [&str; 2] = ["steady", "slot_write_fail"];

/// 一行产物拆成 (字段名, 值)；`E7RESULT` 那一段不带等号，不算字段。
fn result_line_fields(line: &str) -> Vec<(&str, &str)> {
    line.split(' ').filter_map(|token| token.split_once('=')).collect()
}

fn field_value<'line>(fields: &[(&'line str, &'line str)], field_name: &str) -> Option<&'line str> {
    fields.iter().find(|(name, _)| *name == field_name).map(|(_, value)| *value)
}

/// 两族之间配对的一行。
struct PairedLine<'line> {
    /// (去掉族前缀的行名, 去掉族前缀的 `line=`, 旧那一边的臂名, 行键字段) 拼成的键。
    key: String,
    base_arm: &'static str,
    suffix: &'line str,
    line_suffix: Option<&'line str>,
    fields: Vec<(&'line str, &'line str)>,
}

/// 甲类行、臂在对应表里才交回；`is_core_gate_side` 说这一行是 CG 那一边（臂名按对应表的右列认）还是旧那一边。
fn paired_line<'line>(line: &'line str, family_prefix: &str, is_core_gate_side: bool) -> Option<PairedLine<'line>> {
    let fields = result_line_fields(line);
    let suffix = field_value(&fields, "name")?.strip_prefix(family_prefix)?;
    if !PAIRED_LINE_SUFFIXES.contains(&suffix) {
        return None;
    }
    let arm = field_value(&fields, "arm")?;
    let base_arm = CORE_GATE_ARM_COUNTERPARTS.iter().find(|(old_arm, new_arm)| if is_core_gate_side { *new_arm == arm } else { *old_arm == arm }).map(|(old_arm, _)| *old_arm)?;
    let line_suffix = match field_value(&fields, "line") {
        Some(line_name) => Some(line_name.strip_prefix(family_prefix)?),
        None => None,
    };
    let key_fields: Vec<String> = fields.iter().filter(|(name, _)| ROW_KEY_FIELD_NAMES.contains(name)).map(|(name, value)| format!("{name}={value}")).collect();
    let key = format!("{suffix} line={} arm={base_arm} {}", line_suffix.unwrap_or("-"), key_fields.join(" "));
    Some(PairedLine { key, base_arm, suffix, line_suffix, fields })
}

/// 五个判定位：这一行判这一位、行里有这个字段时 `Some(计数 > 0)`，否则 `None`（第一节读法第 3 条）。
fn judged_bits(suffix: &str, fields: &[(&str, &str)]) -> [Option<bool>; 5] {
    let world = field_value(fields, "world");
    let failures = field_value(fields, "failures").and_then(|text| text.parse::<u64>().ok());
    let is_rollback_first_publish = world == Some("rollback_first_publish");
    let is_space_judged = match suffix {
        "near_full" | "broken_slot" | "broken_slot_h1" => true,
        "drain_failure" => failures.is_some_and(|count| count <= TOLERATED_ROOT_WRITE_FAILURES),
        "cost_growth" => field_value(fields, "within_declared") == Some("1"),
        _ => false,
    };
    let is_retention_judged = !is_rollback_first_publish && suffix != "disk_stuck" && suffix != "disk_stuck_h3" && !(suffix == "drain_failure" && failures == Some(3));
    let judged = [!is_rollback_first_publish, !is_rollback_first_publish, is_space_judged, is_space_judged, is_retention_judged];
    let mut bits = [None; 5];
    for (position, field_name) in JUDGEMENT_COUNT_FIELDS.iter().enumerate() {
        if judged[position] {
            bits[position] = field_value(fields, field_name).and_then(|text| text.parse::<u64>().ok()).map(|count| count > 0);
        }
    }
    bits
}

fn judgement_counts(fields: &[(&str, &str)]) -> [Option<u64>; 5] {
    JUDGEMENT_COUNT_FIELDS.map(|field_name| field_value(fields, field_name).and_then(|text| text.parse::<u64>().ok()))
}

/// 一对的五位：旧 0 → 新 1 的位数（翻面）、旧 1 → 新 0 的位数（反向）、两边同为 1 的位数（F9）。不判的位不数。
fn judged_bit_transitions(old_bits: &[Option<bool>; 5], new_bits: &[Option<bool>; 5]) -> (u64, u64, u64) {
    let mut flips = 0u64;
    let mut reverse = 0u64;
    let mut both_red = 0u64;
    for (old_bit, new_bit) in old_bits.iter().zip(new_bits.iter()) {
        if *old_bit == Some(false) && *new_bit == Some(true) {
            flips += 1;
        }
        if *old_bit == Some(true) && *new_bit == Some(false) {
            reverse += 1;
        }
        if *old_bit == Some(true) && *new_bit == Some(true) {
            both_red += 1;
        }
    }
    (flips, reverse, both_red)
}

/// 一条臂（按旧那一边的臂名）在逐行比里的计数（第六节第 5–8 行与 F9）。
#[derive(Clone, Copy, Default)]
struct CoreGateArmTally {
    pairs: u64,
    pairs_with_a_differing_field: u64,
    pairs_with_counts_changed: u64,
    flips: u64,
    pairs_with_a_flip: u64,
    reverse: u64,
    pairs_both_red: u64,
    /// 第六节第 7 行：CG 那一边判定位为 1 的行数，按五位分。
    new_red_rows_by_position: [u64; 5],
}

/// 五个计数变了的一对（第六节第 6 行「变了的每一对」）。
struct CoreGateChangedPair {
    core_gate_line_name: String,
    core_gate_arm: String,
    key_fields: String,
    old_counts: [Option<u64>; 5],
    new_counts: [Option<u64>; 5],
    old_bits: [Option<bool>; 5],
    new_bits: [Option<bool>; 5],
    flips: u64,
    /// 这一行的 `core_gate_reach` 里 dF ≠ 0 与钉值起作用的采样数（F8 归因）；找不到写 None。
    reach: Option<(String, String)>,
}

struct CoreGateComparison {
    by_arm: BTreeMap<&'static str, CoreGateArmTally>,
    /// (类, 旧那一边的臂) → (比了几对, 任一字段不同的对数)。
    by_class_and_arm: BTreeMap<(String, &'static str), (u64, u64)>,
    old_lines_without_partner: u64,
    core_gate_lines_without_partner: u64,
    duplicate_keys: u64,
    pairs_differing_in_worlds_without_recovery: u64,
    changed: Vec<CoreGateChangedPair>,
}

/// 第六节第 5、6、7、8 行与 V5、V7：旧那一边的甲类行与同键的 CG 行逐对比；只读两段记下来的行，不跑模型。
fn compare_core_gate_lines(old_lines: &[String], core_gate_lines: &[String]) -> CoreGateComparison {
    let mut comparison = CoreGateComparison {
        by_arm: BTreeMap::new(),
        by_class_and_arm: BTreeMap::new(),
        old_lines_without_partner: 0,
        core_gate_lines_without_partner: 0,
        duplicate_keys: 0,
        pairs_differing_in_worlds_without_recovery: 0,
        changed: Vec::new(),
    };
    let mut core_gate_by_key: HashMap<String, PairedLine> = HashMap::new();
    let mut reach_by_key: HashMap<String, (String, String)> = HashMap::new();
    for line in core_gate_lines {
        let fields = result_line_fields(line);
        if field_value(&fields, "name") == Some(CORE_GATE_LINES.core_gate_reach) {
            let key_fields: Vec<String> = fields.iter().filter(|(name, _)| *name == "line" || *name == "arm" || ROW_KEY_FIELD_NAMES.contains(name)).map(|(name, value)| format!("{name}={value}")).collect();
            let reach_fields = (field_value(&fields, "floor_term_difference_nonzero").unwrap_or("na").to_string(), field_value(&fields, "pin_effective").unwrap_or("na").to_string());
            reach_by_key.insert(key_fields.join(" "), reach_fields);
            continue;
        }
        if let Some(paired) = paired_line(line, CORE_GATE_LINE_PREFIX, true) {
            if core_gate_by_key.insert(paired.key.clone(), paired).is_some() {
                comparison.duplicate_keys += 1;
            }
        }
    }
    let mut old_keys_seen: HashSet<String> = HashSet::new();
    for line in old_lines {
        let Some(old) = paired_line(line, SECOND_RERUN_LINE_PREFIX, false) else {
            continue;
        };
        if !old_keys_seen.insert(old.key.clone()) {
            comparison.duplicate_keys += 1;
            continue;
        }
        let Some(new) = core_gate_by_key.get(&old.key) else {
            comparison.old_lines_without_partner += 1;
            continue;
        };
        let new_field_names: HashSet<&str> = new.fields.iter().map(|(name, _)| *name).collect();
        let is_differing = old.fields.iter()
            .filter(|(name, _)| *name != "name" && *name != "line" && *name != "arm" && new_field_names.contains(name))
            .any(|(name, value)| field_value(&new.fields, name) != Some(*value));
        let world = field_value(&old.fields, "world");
        let effective_suffix = old.line_suffix.unwrap_or(old.suffix);
        let class = match (old.line_suffix, world) {
            (Some(line_suffix), Some(world)) => format!("{}:{line_suffix}:{world}", old.suffix),
            (Some(line_suffix), None) => format!("{}:{line_suffix}", old.suffix),
            (None, Some(world)) => format!("{}:{world}", old.suffix),
            (None, None) => old.suffix.to_string(),
        };
        let class_tally = comparison.by_class_and_arm.entry((class, old.base_arm)).or_insert((0, 0));
        class_tally.0 += 1;
        class_tally.1 += u64::from(is_differing);
        let is_without_recovery = LINE_SUFFIXES_WITHOUT_RECOVERY.contains(&effective_suffix)
            || (effective_suffix == "cell" && world.is_some_and(|world| WORLDS_WITHOUT_RECOVERY.contains(&world)));
        comparison.pairs_differing_in_worlds_without_recovery += u64::from(is_differing && is_without_recovery);
        let old_bits = judged_bits(old.suffix, &old.fields);
        let new_bits = judged_bits(new.suffix, &new.fields);
        let old_counts = judgement_counts(&old.fields);
        let new_counts = judgement_counts(&new.fields);
        let (flips, reverse, both_red) = judged_bit_transitions(&old_bits, &new_bits);
        let tally = comparison.by_arm.entry(old.base_arm).or_default();
        tally.pairs += 1;
        tally.pairs_with_a_differing_field += u64::from(is_differing);
        tally.flips += flips;
        tally.pairs_with_a_flip += u64::from(flips > 0);
        tally.reverse += reverse;
        tally.pairs_both_red += u64::from(both_red > 0);
        for (position, bit) in new_bits.iter().enumerate() {
            tally.new_red_rows_by_position[position] += u64::from(*bit == Some(true));
        }
        if old_counts != new_counts {
            tally.pairs_with_counts_changed += 1;
            let core_gate_line_name = field_value(&new.fields, "name").unwrap_or("na").to_string();
            let core_gate_arm = field_value(&new.fields, "arm").unwrap_or("na").to_string();
            let key_fields: Vec<String> = new.fields.iter().filter(|(name, _)| ROW_KEY_FIELD_NAMES.contains(name)).map(|(name, value)| format!("{name}={value}")).collect();
            let key_fields = key_fields.join(" ");
            let reach = reach_by_key.get(&format!("line={core_gate_line_name} arm={core_gate_arm} {key_fields}")).cloned();
            comparison.changed.push(CoreGateChangedPair {
                core_gate_line_name, core_gate_arm, key_fields, old_counts, new_counts, old_bits, new_bits, flips, reach,
            });
        }
        core_gate_by_key.remove(&old.key);
    }
    comparison.core_gate_lines_without_partner = core_gate_by_key.values().filter(|paired| !is_core_gate_only_positive_control_line(paired)).count() as u64;
    comparison
}

/// CG 一族的阳性对照比第 2 次多进的格（torn_newest 与 activation_faults；第 2 次的短臂只进 steady），在旧那一边没有同键的行，不算「没配上」。
fn is_core_gate_only_positive_control_line(paired: &PairedLine) -> bool {
    (paired.base_arm == "RGall_short" || paired.base_arm == "RGall_Q_short") && field_value(&paired.fields, "world") != Some("steady")
}

fn counts_pair_text(old_count: Option<u64>, new_count: Option<u64>) -> String {
    let shown = |count: Option<u64>| count.map_or_else(|| "na".to_string(), |value| value.to_string());
    format!("{}/{}", shown(old_count), shown(new_count))
}

/// 第六节第 5–8 行的行与判决行：变了的每一对、翻面的每一对（不截断），按臂与按类的计数。
fn emit_core_gate_comparison(emitter: &mut Emitter, comparison: &CoreGateComparison) {
    for changed in &comparison.changed {
        let counts: Vec<String> = JUDGEMENT_COUNT_FIELDS.iter().enumerate()
            .map(|(position, field_name)| format!("{field_name}_old_new={}", counts_pair_text(changed.old_counts[position], changed.new_counts[position]))).collect();
        let (floor_term_nonzero, pin_effective) = changed.reach.clone().unwrap_or_else(|| ("na".to_string(), "na".to_string()));
        let body = format!(
            "line={} arm={} {} {} bits_old={} bits_new={}", changed.core_gate_line_name, changed.core_gate_arm, changed.key_fields, counts.join(" "),
            bits_text(&changed.old_bits), bits_text(&changed.new_bits)
        );
        println!("{}", emitter.emit_raw(&format!("name=core_gate_changed_row {body}")));
        if changed.flips > 0 {
            println!("{}", emitter.emit_raw(&format!("name=core_gate_flip_row {body} flipped_bits={} floor_term_difference_nonzero={floor_term_nonzero} pin_effective={pin_effective}", changed.flips)));
        }
    }
    for ((class, base_arm), (compared, differing)) in &comparison.by_class_and_arm {
        println!("{}", emitter.emit_raw(&format!("name=verdict_core_gate_discrimination_class class={class} old_arm={base_arm} pairs_compared={compared} pairs_differing={differing}")));
    }
    for (base_arm, tally) in &comparison.by_arm {
        println!("{}", emitter.emit_raw(&format!(
            "name=verdict_core_gate_flips old_arm={base_arm} pairs_compared={} pairs_with_a_differing_field={} pairs_with_counts_changed={} flipped_bits={} \
             pairs_with_a_flip={} reverse_bits={} pairs_both_red={} new_red_rows_c3f={} new_red_rows_c3u={} new_red_rows_c4={} new_red_rows_c5={} new_red_rows_c7={}",
            tally.pairs, tally.pairs_with_a_differing_field, tally.pairs_with_counts_changed, tally.flips, tally.pairs_with_a_flip, tally.reverse,
            tally.pairs_both_red, tally.new_red_rows_by_position[0], tally.new_red_rows_by_position[1], tally.new_red_rows_by_position[2],
            tally.new_red_rows_by_position[3], tally.new_red_rows_by_position[4]
        )));
    }
    println!("{}", emitter.emit_raw(&format!(
        "name=verdict_core_gate_pairing old_lines_without_partner={} core_gate_lines_without_partner={} duplicate_keys={} pairs_differing_in_worlds_without_recovery={}",
        comparison.old_lines_without_partner, comparison.core_gate_lines_without_partner, comparison.duplicate_keys, comparison.pairs_differing_in_worlds_without_recovery
    )));
}

/// 第六节第 4、10 行：CG 一族的行簿上数 CG 走到（按世界类合计）与阳性对照（每一行的假候选）。返回 (d ≠ 0, dF ≠ 0, 钉值起作用) 三个合计，
/// 与阳性对照为 0 的行数（steady，别的两类）。
fn emit_core_gate_reach_and_control_verdicts(emitter: &mut Emitter, rows: &[WindowRow]) -> ([u64; 3], u64, u64) {
    let mut by_class: BTreeMap<String, (u64, CoreGateObservations)> = BTreeMap::new();
    for row in rows {
        let entry = by_class.entry(row.world_class.clone()).or_insert((0, CoreGateObservations::fresh()));
        entry.0 += 1;
        entry.1.add(&row.metrics.core_gate);
    }
    let mut totals = CoreGateObservations::fresh();
    for (class, (row_count, observations)) in &by_class {
        println!("{}", emitter.emit_raw(&format!("name=core_gate_reach_class class={class} rows={row_count} {}", observations.render())));
        totals.add(observations);
    }
    let is_short = |arm: Arm| CORE_GATE_SHORT_ARMS.iter().any(|whole_ring_arm| arm == Arm::WholeRing(*whole_ring_arm));
    let mut control_rows: BTreeMap<String, (u64, u64)> = BTreeMap::new();
    for row in rows.iter().filter(|row| is_short(row.arm)) {
        let entry = control_rows.entry(row.world_class.clone()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += u64::from(row.metrics.fake_candidates == 0);
    }
    let mut steady_zero_rows = 0u64;
    let mut other_zero_rows = 0u64;
    for (class, (row_count, zero_rows)) in &control_rows {
        println!("{}", emitter.emit_raw(&format!("name=verdict_core_gate_positive_control class={class} rows={row_count} rows_with_zero_fake_candidates={zero_rows}")));
        if class == World::Steady.name() { steady_zero_rows += zero_rows } else { other_zero_rows += zero_rows }
    }
    println!("{}", emitter.emit_raw(&format!("name=verdict_core_gate_reach rows={} {}", rows.len(), totals.render())));
    (
        [totals.gate_difference_nonzero_samples, totals.floor_term_difference_nonzero_samples, totals.pin_effective_samples],
        steady_zero_rows,
        other_zero_rows,
    )
}

/// 第六节第 9 行：保留池扫描两族（CG 开、CG 关）同 (臂, 开销对, 保留池) 逐行比全部字段。返回不同的行数。
fn emit_core_gate_reserve_sweeps(emitter: &mut Emitter) -> u64 {
    let mut bodies: [Vec<String>; 2] = [Vec::new(), Vec::new()];
    for (family_index, (line_name, arms)) in [("core_gate_reserve_sweep", CORE_GATE_ARMS), ("core_gate_off_reserve_sweep", WHOLE_RING_ARMS)].iter().enumerate() {
        for whole_ring_arm in arms {
            let arm = Arm::WholeRing(*whole_ring_arm);
            for (mount_cost, declared_maximum_cost) in COST_PAIRS {
                let ring_depth = REGION_COUNT * RESERVE_SWEEP_SLOTS_PER_REGION;
                let formula = arm.reserve_blocks(ring_depth, arm.formula_cost(mount_cost, declared_maximum_cost));
                for reserve in (formula - RESERVE_SWEEP_BELOW)..=(formula + RESERVE_SWEEP_ABOVE) {
                    let total = fresh_total_over_seeds(RESERVE_SWEEP_SEED_COUNT, |seed| run_near_full(arm, RESERVE_SWEEP_SLOTS_PER_REGION, mount_cost, declared_maximum_cost, seed, Some(reserve)));
                    let body = format!(
                        "s={RESERVE_SWEEP_SLOTS_PER_REGION} c0={mount_cost} c_max={declared_maximum_cost} reserve={reserve} formula={formula} \
                         stalls={} false_enospc_arm={} write_after_delete_failed={} max_publications={} fakes={}",
                        total.stalls, total.false_enospc_arm, total.write_after_delete_failed, total.maximum_publications_per_admission, total.fake_candidates
                    );
                    println!("{}", emitter.emit_raw(&format!("name={line_name} arm={} {body}", arm.name())));
                    bodies[family_index].push(body);
                }
            }
        }
    }
    let compared = bodies[0].len().min(bodies[1].len());
    let differing = bodies[0].iter().zip(bodies[1].iter()).filter(|(core_gate_body, old_body)| core_gate_body != old_body).count() + bodies[0].len().abs_diff(bodies[1].len());
    println!("{}", emitter.emit_raw(&format!("name=verdict_core_gate_reserve_sweep rows_compared={compared} rows_differing={differing}")));
    differing as u64
}

/// 第 3 次重跑（`research/prompts/e139-r3-prereg.md`）：CG 一族进 RGall 一族进的每一格，保留池扫描两族，再与第 2 次那一族记下的行逐行比。
/// 接在第 2 次重跑的全部行之后、`name=done` 之前；`second_rerun_lines` 是第 2 次那一族打出的、记下来的行（同一次运行里重出的旧那一边）。
pub(super) fn run_core_gate_arms(emitter: &mut Emitter, second_rerun_lines: &[String]) {
    let family = core_gate_family();
    let family = &family;
    let arm_names = |arms: &[Arm]| arms.iter().map(|arm| arm.name()).collect::<Vec<&str>>().join("/");
    println!("{}", emitter.emit_raw(&format!(
        "name=core_gate_config core_gate_arms={} core_gate_short_arms={} short_worlds=steady/torn_newest/activation_faults short_slots={} disk_stuck_arms={} \
         reserve_sweep_arms={} off_reserve_sweep_arms={} pin_rule=oldest_txg_below_min_of_ring_length_and_first_txg floor_term_rule=min_of_effective_floor_and_newest_on_every_surviving_disk_held_until_rooted",
        arm_names(&family.arms), arm_names(&family.short_arms), format_set(&WINDOW_SHORT_SLOTS_SWEEP), arm_names(&family.disk_stuck_arms),
        arm_names(&family.arms), WHOLE_RING_ARMS.iter().map(|whole_ring_arm| whole_ring_arm.name()).collect::<Vec<&str>>().join("/")
    )));
    let mut rows: Vec<WindowRow> = Vec::new();
    let mut growth: Vec<StuckGrowthSummary> = Vec::new();
    start_recording_lines();
    let _ = emit_window_crit1_rows(emitter, family);
    emit_window_world_rows(emitter, &mut rows, family);
    emit_window_activation_rows(emitter, &mut rows, family, GeometryPoint::Original);
    emit_window_near_full_rows(emitter, &mut rows, family);
    emit_window_drain_failure_rows(emitter, &mut rows, family);
    emit_window_broken_slot_rows(emitter, &mut rows, &mut growth, family, GeometryPoint::Original);
    emit_window_cost_growth_rows(emitter, &mut rows, family);
    emit_window_activation_rows(emitter, &mut rows, family, GeometryPoint::Reversed);
    emit_window_broken_slot_rows(emitter, &mut rows, &mut growth, family, GeometryPoint::Reversed);
    emit_disk_stuck_rows(emitter, &mut rows, &mut growth, family);
    let core_gate_lines = stop_recording_lines();
    let reserve_sweep_rows_differing = emit_core_gate_reserve_sweeps(emitter);
    let ([gate_difference_nonzero, floor_term_difference_nonzero, pin_effective], steady_control_zero_rows, other_control_zero_rows) =
        emit_core_gate_reach_and_control_verdicts(emitter, &rows);
    let comparison = compare_core_gate_lines(second_rerun_lines, &core_gate_lines);
    emit_core_gate_comparison(emitter, &comparison);
    let tally_of = |base_arm: &str| comparison.by_arm.get(base_arm).copied().unwrap_or_default();
    let tested = tally_of("RGall_ring_all");
    let zero_counting_reading = tally_of("RGall_Q_ring_all");
    let discrimination_pairs_differing: u64 = comparison.by_arm.values().map(|tally| tally.pairs_with_a_differing_field).sum();
    let tested_red_rows: u64 = tested.new_red_rows_by_position.iter().sum();
    println!("{}", emitter.emit_raw(&format!(
        "name=verdict_core_gate_clauses tested_arm_pairs={} tested_arm_flipped_bits={} tested_arm_pairs_with_a_flip={} zero_counting_reading_pairs={} zero_counting_reading_pairs_with_a_flip={} \
         discrimination_pairs_differing={discrimination_pairs_differing} gate_difference_nonzero_samples={gate_difference_nonzero} \
         floor_term_difference_nonzero_samples={floor_term_difference_nonzero} pin_effective_samples={pin_effective} tested_arm_red_rows={tested_red_rows} \
         steady_control_rows_with_zero_fakes={steady_control_zero_rows} other_control_rows_with_zero_fakes={other_control_zero_rows} \
         tested_arm_pairs_both_red={} old_lines_without_partner={} core_gate_lines_without_partner={} duplicate_keys={} \
         reserve_sweep_rows_differing={reserve_sweep_rows_differing} pairs_differing_in_worlds_without_recovery={}",
        tested.pairs, tested.flips, tested.pairs_with_a_flip, zero_counting_reading.pairs, zero_counting_reading.pairs_with_a_flip, tested.pairs_both_red,
        comparison.old_lines_without_partner, comparison.core_gate_lines_without_partner, comparison.duplicate_keys, comparison.pairs_differing_in_worlds_without_recovery
    )));
}

fn bits_text(bits: &[Option<bool>; 5]) -> String {
    bits.iter().map(|bit| match bit {
        Some(true) => '1',
        Some(false) => '0',
        None => '-',
    }).collect()
}

#[cfg(test)]
#[path = "core_gate_comparison_tests.rs"]
mod tests;
