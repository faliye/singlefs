//! 第 4 次跑的跨臂比对：作废范围、产物行、句子记录与各项比对。

use super::*;

// ---------------------------------------------------------------------------
// 12.12 跨臂对拍（`r4-compare <产物前缀>…`）：读每个前缀下各臂的产物 `<前缀>-<臂>.out`，算跨臂的句子（与今天逐项相同、PC-多读、PC-多拒、
//       PC-22）、Q4（多拒、拦下）、Q5 两栏、7.3 第二三行、F11、起始镜像跨臂同一份，最后按 5.4 每一句的标签机械算 V1 的作废范围（`r4_void`）。
// ---------------------------------------------------------------------------

/// 一句对照的判定（在臂里打出的，或 compare 里跨臂算的）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct SentenceRecord {
    arm: String,
    control: String,
    name: String,
    tag: SentenceTag,
    verdict: SentenceVerdict,
}

/// 一条对照管哪几个量（登记 11.1 V1 末：PC-N-环、PC-N-盖、PC-O、PC-丢写 ⇒ Q1–Q3；PC-随 ⇒ H-随 那一栏的 Q1–Q3；PC-N0、PC-多拒 ⇒ Q4；
/// PC-多读 ⇒ Q5（(i) 全部、(ii) 同成、(iii) 拒）；PC-22 ⇒ Q6；PC-只读 ⇒ 甲那一句「只读照常」；PC-554 ⇒ F1 与那几臂的 Q4）。
fn quantities_of_a_control(control: &str) -> &'static [&'static str] {
    match control {
        "PC-N-ring" | "PC-N-cover" | "PC-O" | "PC-lost-write" => &["Q1-Q3"],
        "PC-random" => &["Q1-Q3-random-histories"],
        "PC-N0" | "PC-extra-refusal" | "PC-554" => &["Q4"],
        "PC-extra-reads-i" => &["Q5-both-mounted", "Q5-refused"],
        "PC-extra-reads-ii" => &["Q5-both-mounted"],
        "PC-extra-reads-iii" => &["Q5-refused"],
        "PC-22" => &["Q6"],
        "PC-read-only" => &["read-only"],
        _ => &[],
    }
}

/// 拿今天当参照的量（V1 ②：[测] 句在今天那一臂上打不中，别的臂只作废这几样）。
const QUANTITIES_REFERRING_TO_TODAY: [&str; 3] = ["Q4", "Q5-both-mounted", "Q5-refused"];
const EVERY_VOIDABLE_QUANTITY: [&str; 7] = [
    "Q1-Q3",
    "Q1-Q3-random-histories",
    "Q4",
    "Q5-both-mounted",
    "Q5-refused",
    "Q6",
    "read-only",
];

/// (臂, 量) → 作废的依据（空 = 不作废）。
type VoidedQuantities = BTreeMap<(String, &'static str), Vec<String>>;

/// V1（登记 11.1，按句子标签分五种）的作废范围：交回 (臂, 量) → 作废的依据（空 = 不作废），外加「装置缺陷（全部臂一起）」的那几句。
fn fourth_run_void_scope(
    sentences: &[SentenceRecord],
    arms: &[&str],
) -> (VoidedQuantities, Vec<String>) {
    let mut voided: VoidedQuantities = BTreeMap::new();
    for arm in arms {
        for quantity in EVERY_VOIDABLE_QUANTITY {
            voided.insert(((*arm).to_string(), quantity), Vec::new());
        }
    }
    let missed = |record: &SentenceRecord| {
        matches!(
            record.verdict,
            SentenceVerdict::Fail | SentenceVerdict::NotConstructible
        )
    };
    let mut device_defects = Vec::new();
    let mut measured_misses_by_sentence: BTreeMap<(String, String), BTreeSet<String>> =
        BTreeMap::new();
    for record in sentences.iter().filter(|record| missed(record)) {
        let basis = format!("{}:{}({})", record.control, record.name, record.tag.name());
        match record.tag {
            SentenceTag::TodayDescription => {}
            SentenceTag::ArmDefinition => {
                for quantity in quantities_of_a_control(&record.control) {
                    if let Some(list) = voided.get_mut(&(record.arm.clone(), *quantity)) {
                        list.push(basis.clone());
                    }
                }
            }
            SentenceTag::Measured => {
                measured_misses_by_sentence
                    .entry((record.control.clone(), record.name.clone()))
                    .or_default()
                    .insert(record.arm.clone());
                if record.arm == "today" {
                    for quantity in quantities_of_a_control(&record.control) {
                        if let Some(list) = voided.get_mut(&("today".to_string(), *quantity)) {
                            list.push(basis.clone());
                        }
                    }
                    for arm in arms.iter().filter(|arm| **arm != "today") {
                        for quantity in quantities_of_a_control(&record.control)
                            .iter()
                            .filter(|quantity| QUANTITIES_REFERRING_TO_TODAY.contains(quantity))
                        {
                            if let Some(list) = voided.get_mut(&((*arm).to_string(), *quantity)) {
                                list.push(format!("{basis}@today"));
                            }
                        }
                    }
                } else {
                    for quantity in quantities_of_a_control(&record.control) {
                        if let Some(list) = voided.get_mut(&(record.arm.clone(), *quantity)) {
                            list.push(basis.clone());
                        }
                    }
                }
            }
        }
    }
    for ((control, name), missing_arms) in &measured_misses_by_sentence {
        let asked_of: BTreeSet<&str> = sentences
            .iter()
            .filter(|record| &record.control == control && &record.name == name)
            .map(|record| record.arm.as_str())
            .collect();
        if asked_of.len() > 1 && asked_of.iter().all(|arm| missing_arms.contains(*arm)) {
            device_defects.push(format!("{control}:{name}"));
            for arm in arms {
                for quantity in quantities_of_a_control(control) {
                    if let Some(list) = voided.get_mut(&((*arm).to_string(), *quantity)) {
                        list.push(format!("{control}:{name}(measure,every_arm)"));
                    }
                }
            }
        }
    }
    (voided, device_defects)
}

/// PC-多读 (iii) 与 Q5-拒 的参照臂：同一种 N 的甲臂（登记 5.8 第二件：乙 的参照不再是今天那一臂；变异 M20 把它换回今天）。
fn extra_reads_reference_arm(arm: FourthRunArm) -> Option<&'static str> {
    arm.jia_arm_with_the_same_witness()
}

/// 产物里的一行（字段表）与它来自哪条臂。
struct ProductLine {
    fields: BTreeMap<String, String>,
}

impl ProductLine {
    pub(super) fn text(&self, name: &str) -> &str {
        self.fields.get(name).map_or("", String::as_str)
    }

    pub(super) fn number(&self, name: &str) -> Option<i64> {
        self.fields.get(name).and_then(|value| value.parse().ok())
    }

    pub(super) fn key(&self, coordinate_names: &[&str]) -> String {
        coordinate_names
            .iter()
            .filter_map(|name| {
                self.fields
                    .get(*name)
                    .map(|value| format!("{name}={value}"))
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

const LEGAL_OR_FAULT_STATE_CELL_COORDINATES: [&str; 14] = [
    "family", "geometry", "n1", "c", "k", "which", "form", "point", "of", "b", "page", "m",
    "duration", "seed",
];
const CONTROL_COORDINATES: [&str; 7] = ["control", "form", "duration", "n1", "c", "of", "b"];

/// 被测挂载那几行（Q4、Q5 的输入）：H1d 那样的族只取不断那一格，接尾巴的族只取第一个 m（A 的读数与 m 无关），L 族与第二段的每一次被测挂载全取。
fn tested_mount_lines(lines: &[ProductLine]) -> BTreeMap<String, &ProductLine> {
    let mut tested = BTreeMap::new();
    for line in lines {
        let key = match line.text("name") {
            "r4_cell"
                if line.text("cut") == "none"
                    || (line.text("cut").is_empty() && line.text("m") == "1") =>
            {
                format!(
                    "cell {}",
                    line.key(&[
                        "family", "geometry", "n1", "c", "n1a", "n1b", "a", "b", "form", "duration"
                    ])
                )
            }
            "r4_l_cell" if line.text("verdict").is_empty() => {
                format!("l {}", line.key(&LEGAL_OR_FAULT_STATE_CELL_COORDINATES))
            }
            "r4_random_mount" => format!(
                "random {}",
                line.key(&["family", "seed", "step", "attempt"])
            ),
            _ => continue,
        };
        tested.entry(key).or_insert(line);
    }
    tested
}

/// K0–K2 是挂载交回 `Ok`（只差空间准入怎么判），K3、K4 是拒（`mount_outcome_class`）。
pub(super) fn class_is_a_successful_mount(class: &str) -> bool {
    matches!(class, "K0" | "K1" | "K2")
}

fn class_is_a_refusal(class: &str) -> bool {
    matches!(class, "K3" | "K4")
}

fn duration_class(duration: &str) -> &'static str {
    if duration.starts_with('T') {
        "T"
    } else if duration.starts_with('O') {
        "O"
    } else {
        "W"
    }
}

/// Q5-同成 的一格：臂与今天那一臂都交回 `Ok`、起始镜像逐字节相同（变异 M18：把臂拒了的格也算进来）。
fn both_mounted_extra_reads(
    arm_cell: &ProductLine,
    today_cell: &ProductLine,
) -> Option<(i64, i64)> {
    let same_start = !arm_cell.text("base_fingerprint").is_empty()
        && arm_cell.text("base_fingerprint") == today_cell.text("base_fingerprint");
    let both_ok = class_is_a_successful_mount(arm_cell.text("a_class"))
        && class_is_a_successful_mount(today_cell.text("a_class"));
    if !(same_start && both_ok) {
        return None;
    }
    Some((
        arm_cell.number("a_reads")? - today_cell.number("a_reads")?,
        arm_cell.number("a_read_bytes")? - today_cell.number("a_read_bytes")?,
    ))
}

fn read_product_lines(path: &str) -> Result<Vec<ProductLine>, String> {
    let text = std::fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
    let finished = text
        .lines()
        .last()
        .is_some_and(|line| line.starts_with("E7RESULT name=done emitted="));
    if !finished {
        return Err(format!("{path}: 末行不是完成标记"));
    }
    Ok(text
        .lines()
        .filter(|line| line.starts_with("E7RESULT "))
        .map(|line| ProductLine {
            fields: fields_of_a_result_line(line),
        })
        .collect())
}

fn sentence_records_of(arm: &str, lines: &[ProductLine]) -> Vec<SentenceRecord> {
    let mut records = Vec::new();
    for line in lines
        .iter()
        .filter(|line| line.text("name") == "r4_positive_control")
    {
        for (field, value) in &line.fields {
            let Some(rest) = field.strip_prefix("sentence_") else {
                continue;
            };
            let (tag, name) = if let Some(name) = rest.strip_prefix("measure_") {
                (SentenceTag::Measured, name)
            } else if let Some(name) = rest.strip_prefix("arm_") {
                (SentenceTag::ArmDefinition, name)
            } else if let Some(name) = rest.strip_prefix("today_") {
                (SentenceTag::TodayDescription, name)
            } else {
                continue;
            };
            let verdict = match value.as_str() {
                "pass" => SentenceVerdict::Pass,
                "fail" => SentenceVerdict::Fail,
                "not_constructible" => SentenceVerdict::NotConstructible,
                _ => SentenceVerdict::Vacuous,
            };
            records.push(SentenceRecord {
                arm: arm.to_string(),
                control: line.text("control").to_string(),
                name: format!(
                    "{}[{}]",
                    name,
                    line.key(&["form", "duration", "n1", "c", "of"])
                ),
                tag,
                verdict,
            });
        }
    }
    records
}

fn emit_cross_arm_sentence(record: &SentenceRecord, detail: &str) {
    emit_result(&format!(
        "name=r4_cross_arm_sentence arm={} control={} sentence={} tag={} verdict={} {detail}",
        record.arm,
        record.control,
        record.name,
        record.tag.name(),
        record.verdict.name()
    ));
}

#[allow(
    clippy::too_many_lines,
    reason = "跨臂对拍的每一样按登记的次序写在一处，拆开只多几层参数搬运"
)]
pub(super) fn run_fourth_run_compare(prefixes: &[String]) {
    let mut lines_by_arm: BTreeMap<&'static str, Vec<ProductLine>> = BTreeMap::new();
    for arm in FOURTH_RUN_ARMS {
        let mut lines = Vec::new();
        for prefix in prefixes {
            let path = format!("{prefix}-{}.out", arm.name);
            if !std::path::Path::new(&path).exists() {
                continue;
            }
            match read_product_lines(&path) {
                Ok(read) => {
                    emit_result(&format!(
                        "name=r4_compare_input arm={} path={path} lines={} verdict=pass",
                        arm.name,
                        read.len()
                    ));
                    lines.extend(read);
                }
                Err(error) => emit_result(&format!(
                    "name=r4_compare_input arm={} verdict=stop reason={error:?}",
                    arm.name
                )),
            }
        }
        if !lines.is_empty() {
            lines_by_arm.insert(arm.name, lines);
        }
    }
    let Some(today_lines) = lines_by_arm.get("today") else {
        emit_result(
            "name=r4_compare_input arm=today verdict=stop reason=\"今天那一臂的产物一份都没有\"",
        );
        return;
    };
    let today_tested = tested_mount_lines(today_lines);
    let today_controls: BTreeMap<String, &ProductLine> = today_lines
        .iter()
        .filter(|line| line.text("name") == "r4_positive_control")
        .map(|line| (line.key(&CONTROL_COORDINATES), line))
        .collect();
    let mut sentences: Vec<SentenceRecord> = Vec::new();
    for (arm_name, lines) in &lines_by_arm {
        sentences.extend(sentence_records_of(arm_name, lines));
    }
    // PC-多读 (i)：今天那一臂减自己 = 0（每一格）。
    emit_result(&format!(
        "name=r4_section_seven_three row=4 arm=today cells={} nonzero_cells=0 verdict=pass",
        today_tested.len()
    ));
    sentences.push(SentenceRecord {
        arm: "today".to_string(),
        control: "PC-extra-reads-i".to_string(),
        name: "today_minus_itself_is_zero".to_string(),
        tag: SentenceTag::Measured,
        verdict: SentenceVerdict::Pass,
    });
    for (arm_name, lines) in &lines_by_arm {
        let Some(arm) = fourth_run_arm_named(arm_name) else {
            continue;
        };
        if arm.is_today() {
            continue;
        }
        let tested = tested_mount_lines(lines);
        // Q5 两栏、Q4（按族分报）。
        let mut both_mounted: BTreeMap<String, ExtraReads> = BTreeMap::new();
        let mut refused_minus_today: BTreeMap<String, ExtraReads> = BTreeMap::new();
        let mut extra_refusals: BTreeMap<String, u64> = BTreeMap::new();
        let mut intercepted: BTreeMap<String, u64> = BTreeMap::new();
        let mut compared: BTreeMap<String, u64> = BTreeMap::new();
        let mut diverged_mounts: BTreeMap<String, u64> = BTreeMap::new();
        let mut diverged_refusals: BTreeMap<String, u64> = BTreeMap::new();
        for (key, cell) in &tested {
            let family = format!("{}@{}", cell.text("family"), cell.text("geometry"));
            let today_cell = today_tested.get(key);
            // 第二段那几族整段在各臂上各跑一遍（登记 5.3）：与今天走岔之后的挂载（起始镜像不同，或今天那一臂没有这一格）没有
            // 「今天的同一格」，只按「臂拒 ∧ N_真 为假」记，单列一栏（r3 登记「读法写死」表「多拒」一行）。
            if cell.text("name") == "r4_random_mount"
                && today_cell.is_none_or(|today_cell| {
                    cell.text("base_fingerprint") != today_cell.text("base_fingerprint")
                })
            {
                *diverged_mounts.entry(family.clone()).or_insert(0) += 1;
                if class_is_a_refusal(cell.text("a_class")) && cell.text("truth_newer") != "true" {
                    *diverged_refusals.entry(family).or_insert(0) += 1;
                }
                continue;
            }
            let Some(today_cell) = today_cell else {
                continue;
            };
            *compared.entry(family.clone()).or_insert(0) += 1;
            if let Some((calls, bytes)) = both_mounted_extra_reads(cell, today_cell) {
                both_mounted
                    .entry(family.clone())
                    .or_default()
                    .add(calls, bytes);
            }
            let arm_refused = class_is_a_refusal(cell.text("a_class"));
            let today_refused = class_is_a_refusal(today_cell.text("a_class"));
            if arm_refused {
                if let (Some(calls), Some(today_calls)) =
                    (cell.number("a_reads"), today_cell.number("a_reads"))
                {
                    refused_minus_today.entry(family.clone()).or_default().add(
                        calls - today_calls,
                        cell.number("a_read_bytes").unwrap_or(0)
                            - today_cell.number("a_read_bytes").unwrap_or(0),
                    );
                }
            }
            if arm_refused && !today_refused {
                let truth_newer = cell.text("truth_newer") == "true"
                    || cell.text("truth_newer_on_the_twin") == "true";
                if truth_newer {
                    *intercepted
                        .entry(format!(
                            "{family}:{}",
                            duration_class(cell.text("duration"))
                        ))
                        .or_insert(0) += 1;
                } else {
                    *extra_refusals.entry(family.clone()).or_insert(0) += 1;
                }
            }
        }
        for (family, cells) in &compared {
            emit_result(&format!(
                "name=r4_q4 arm={arm_name} family={family} compared_tested_mounts={cells} extra_refusals={} intercepted={:?}",
                extra_refusals.get(family).copied().unwrap_or(0),
                intercepted
                    .iter()
                    .filter(|(key, _)| key.starts_with(&format!("{family}:")))
                    .map(|(key, count)| format!("{}={count}", key.rsplit(':').next().unwrap_or("")))
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
        for (family, mounts) in &diverged_mounts {
            emit_result(&format!(
                "name=r4_q4_after_divergence arm={arm_name} family={family} tested_mounts_after_divergence={mounts} refusals_with_truth_newer_false={}",
                diverged_refusals.get(family).copied().unwrap_or(0)
            ));
        }
        for (column, table) in [
            ("both_mounted", &both_mounted),
            ("refused_minus_today", &refused_minus_today),
        ] {
            for (family, extra) in table {
                emit_result(&format!(
                    "name=r4_q5 arm={arm_name} column={column} family={family} cells={} positive_cells={} total_extra_read_calls={} total_extra_read_bytes={} minimum_extra_read_calls={} maximum_extra_read_calls={}",
                    extra.cells,
                    extra.positive_cells,
                    extra.total_calls,
                    extra.total_bytes,
                    extra.minimum_calls.map_or("none".to_string(), |value| value.to_string()),
                    extra.maximum_calls.map_or("none".to_string(), |value| value.to_string())
                ));
            }
        }
        // 与今天逐项相同：PC-N0（F12，全部臂）与 PC-N-环 / PC-N-盖 的读回全零格（-槽 臂，N-槽 判假）。
        for line in lines
            .iter()
            .filter(|line| line.text("name") == "r4_positive_control")
        {
            let control = line.text("control");
            let needs_same_as_today = control == "PC-N0"
                || (matches!(control, "PC-N-ring" | "PC-N-cover")
                    && arm.witness == FourthRunWitness::UnreadableRootRingSlot
                    && line.text("form") == "reads_zeros");
            if !needs_same_as_today {
                continue;
            }
            let key = line.key(&CONTROL_COORDINATES);
            let differing: Vec<&str> = match today_controls.get(&key) {
                Some(today_line) => ["a_class", "a_chosen", "a_effective", "write_fingerprint"]
                    .into_iter()
                    .filter(|field| line.text(field) != today_line.text(field))
                    .collect(),
                None => vec!["today_line_missing"],
            };
            let record = SentenceRecord {
                arm: arm_name.to_string(),
                control: control.to_string(),
                name: format!(
                    "same_as_today[{}]",
                    line.key(&["form", "duration", "n1", "c"])
                ),
                tag: SentenceTag::ArmDefinition,
                verdict: SentenceVerdict::of(differing.is_empty()),
            };
            emit_cross_arm_sentence(
                &record,
                &format!(
                    "differing_fields={}",
                    if differing.is_empty() {
                        "none".to_string()
                    } else {
                        differing.join(",")
                    }
                ),
            );
            sentences.push(record);
        }
        // PC-随（登记 5.4）：-槽 各臂在 H-随 那三段上，到崩溃恢复抛弃根那一步为止与今天逐项相同（每一步做完时的镜像指纹）。
        if arm.witness == FourthRunWitness::UnreadableRootRingSlot {
            for line in lines.iter().filter(|line| {
                line.text("name") == "r4_positive_control" && line.text("control") == "PC-random"
            }) {
                let label = line.text("seed");
                let last = line.number("crash_recovery_order");
                let fingerprints_of = |step_lines: &[ProductLine]| -> BTreeMap<i64, String> {
                    step_lines
                        .iter()
                        .filter(|step| {
                            step.text("name") == "r4_random_step"
                                && step.text("family") == "h-random"
                                && step.text("seed") == label
                        })
                        .filter_map(|step| {
                            Some((
                                step.number("order")?,
                                step.text("fingerprint_after").to_string(),
                            ))
                        })
                        .collect()
                };
                let arm_fingerprints = fingerprints_of(lines);
                let today_fingerprints = fingerprints_of(today_lines);
                let differing: Vec<i64> = match last {
                    Some(last) => (0..=last)
                        .filter(|order| {
                            !arm_fingerprints.contains_key(order)
                                || arm_fingerprints.get(order) != today_fingerprints.get(order)
                        })
                        .collect(),
                    None => Vec::new(),
                };
                let record = SentenceRecord {
                    arm: arm_name.to_string(),
                    control: "PC-random".to_string(),
                    name: format!(
                        "slot_arm_same_as_today_up_to_the_crash_recovery_step[of={}]",
                        line.text("of")
                    ),
                    tag: SentenceTag::ArmDefinition,
                    verdict: if last.is_none() {
                        SentenceVerdict::NotConstructible
                    } else {
                        SentenceVerdict::of(differing.is_empty())
                    },
                };
                emit_cross_arm_sentence(
                    &record,
                    &format!("seed={label} differing_orders={differing:?}").replace(' ', ""),
                );
                sentences.push(record);
            }
        }
        // PC-多读 (ii)：L0 上 Q5-同成 的点值（甲 / 乙 / 乙-窄读 -配置 +4、-配置续 +8、-槽 +3S；丙-X = 甲-X 的点值减 3S + p）。
        for line in lines.iter().filter(|line| {
            line.text("name") == "r4_positive_control" && line.text("control") == "PC-N0"
        }) {
            let key = line.key(&CONTROL_COORDINATES);
            let Some(today_line) = today_controls.get(&key) else {
                continue;
            };
            let three_slots_per_region =
                i64::try_from(ROOT_RING_REGIONS_LOCAL * GEOMETRY_PRIMARY.slots_per_region)
                    .expect("24");
            let witness_point = match arm.witness {
                FourthRunWitness::NotJudged => 0,
                FourthRunWitness::SystemConfiguration => 4,
                FourthRunWitness::SystemConfigurationCarried => 8,
                FourthRunWitness::UnreadableRootRingSlot => three_slots_per_region,
            };
            let pages = today_line.number("effective_instance_table_pages");
            let expected = match arm.candidate {
                FourthRunCandidate::Bing => {
                    pages.map(|pages| witness_point - (three_slots_per_region + pages))
                }
                FourthRunCandidate::Jia
                | FourthRunCandidate::Yi
                | FourthRunCandidate::YiNarrow
                | FourthRunCandidate::Today => Some(witness_point),
            };
            let measured = both_mounted_extra_reads(line, today_line).map(|(calls, _)| calls);
            let record = SentenceRecord {
                arm: arm_name.to_string(),
                control: "PC-extra-reads-ii".to_string(),
                name: format!("point_value[{}]", line.key(&["n1", "c"])),
                tag: SentenceTag::ArmDefinition,
                verdict: match (measured, expected) {
                    (Some(measured), Some(expected)) => SentenceVerdict::of(measured == expected),
                    (None, _) | (_, None) => SentenceVerdict::NotConstructible,
                },
            };
            emit_cross_arm_sentence(&record, &format!("measured_extra_read_calls={measured:?} expected={expected:?} instance_table_pages={pages:?}").replace(' ', ""));
            sentences.push(record);
        }
        // PC-多读 (iii)：PC-N-环 与 PC-N-盖 的 W 格。
        if let Some(jia_name) = extra_reads_reference_arm(arm) {
            if arm.rereads() || arm.candidate == FourthRunCandidate::Bing {
                let jia_controls: BTreeMap<String, &ProductLine> = lines_by_arm
                    .get(jia_name)
                    .map(|jia_lines| {
                        jia_lines
                            .iter()
                            .filter(|line| line.text("name") == "r4_positive_control")
                            .map(|line| (line.key(&CONTROL_COORDINATES), line))
                            .collect()
                    })
                    .unwrap_or_default();
                for line in lines.iter().filter(|line| {
                    line.text("name") == "r4_positive_control"
                        && matches!(line.text("control"), "PC-N-ring" | "PC-N-cover")
                        && line.text("duration") == "W"
                }) {
                    if arm.witness == FourthRunWitness::UnreadableRootRingSlot
                        && line.text("form") != "read_fails"
                    {
                        continue;
                    }
                    let key = line.key(&CONTROL_COORDINATES);
                    let jia_total = jia_controls
                        .get(&key)
                        .and_then(|jia_line| jia_line.number("a_reads"));
                    let total = line.number("a_reads");
                    let coordinates = line.key(&["control", "form"]);
                    let mut push =
                        |name: &str, tag: SentenceTag, verdict: SentenceVerdict, detail: String| {
                            let record = SentenceRecord {
                                arm: arm_name.to_string(),
                                control: "PC-extra-reads-iii".to_string(),
                                name: format!("{name}[{coordinates}]"),
                                tag,
                                verdict,
                            };
                            emit_cross_arm_sentence(&record, &detail);
                            sentences.push(record);
                        };
                    if arm.rereads() {
                        let before = line.number("a_reads_before_first_wait");
                        let after = line.number("a_reads_after_first_wait");
                        let faults_before = line.number("a_faults_intercepted_before_first_wait");
                        push(
                            "first_pass_reads_equal_jia_total",
                            SentenceTag::ArmDefinition,
                            match (before, jia_total) {
                                (Some(before), Some(jia_total)) => {
                                    SentenceVerdict::of(before == jia_total)
                                }
                                (None, _) | (_, None) => SentenceVerdict::NotConstructible,
                            },
                            format!("before={before:?} jia_total={jia_total:?}").replace(' ', ""),
                        );
                        let rounds = i64::try_from(arm.reread_rounds).expect("R");
                        push(
                            "reads_after_the_mark_at_least_r_times_f_and_positive",
                            SentenceTag::ArmDefinition,
                            match (after, faults_before) {
                                (Some(after), Some(faults)) => {
                                    SentenceVerdict::of(after >= rounds * faults && after > 0)
                                }
                                (None, _) | (_, None) => SentenceVerdict::NotConstructible,
                            },
                            format!("after={after:?} f={faults_before:?} r={rounds}")
                                .replace(' ', ""),
                        );
                        push(
                            "total_is_before_plus_after",
                            SentenceTag::Measured,
                            match (total, before, after) {
                                (Some(total), Some(before), Some(after)) => {
                                    SentenceVerdict::of(total == before + after)
                                }
                                (None, _, _) | (_, None, _) | (_, _, None) => {
                                    SentenceVerdict::NotConstructible
                                }
                            },
                            format!("total={total:?}").replace(' ', ""),
                        );
                    } else if arm.witness != FourthRunWitness::UnreadableRootRingSlot {
                        push(
                            "bing_minus_jia_positive",
                            SentenceTag::ArmDefinition,
                            match (total, jia_total) {
                                (Some(total), Some(jia_total)) => {
                                    SentenceVerdict::of(total - jia_total > 0)
                                }
                                (None, _) | (_, None) => SentenceVerdict::NotConstructible,
                            },
                            format!("total={total:?} jia_total={jia_total:?}").replace(' ', ""),
                        );
                    }
                }
            }
        }
    }
    compare_extra_refusal_and_cell_22_controls(&lines_by_arm, &today_tested, &mut sentences);
    compare_row_two_three_and_clause_eleven(&lines_by_arm);
    compare_base_identity(&lines_by_arm);
    compare_losses(&lines_by_arm);
    let arm_names: Vec<&str> = lines_by_arm.keys().copied().collect();
    let (voided, device_defects) = fourth_run_void_scope(&sentences, &arm_names);
    for defect in &device_defects {
        emit_result(&format!("name=r4_device_defect sentence={defect:?} verdict=stop reason=\"同一句 [测] 在每一臂上都打不中（V1 ③）\""));
    }
    for ((arm, quantity), bases) in &voided {
        emit_result(&format!(
            "name=r4_void arm={arm} quantity={quantity} voided={} basis={}",
            !bases.is_empty(),
            if bases.is_empty() {
                "none".to_string()
            } else {
                bases.join(",")
            }
        ));
    }
    let missed_routes: Vec<String> = sentences
        .iter()
        .filter(|record| {
            record.tag != SentenceTag::Measured
                && matches!(
                    record.verdict,
                    SentenceVerdict::Fail | SentenceVerdict::NotConstructible
                )
        })
        .map(|record| {
            format!(
                "{}:{}:{}({})",
                record.arm,
                record.control,
                record.name,
                record.tag.name()
            )
        })
        .collect();
    emit_result(&format!(
        "name=r4_missed_arm_and_today_sentences count={} list={}",
        missed_routes.len(),
        if missed_routes.is_empty() {
            "none".to_string()
        } else {
            missed_routes.join(",")
        }
    ));
}

/// PC-多拒（-配置 各臂 L3(a) `read_fails` n1 = 1；-槽 各臂 L2 k = 2 `read_fails` n1 = 1；今天不 `Ok` 就顺延 n1）与 PC-22（L5）。
fn compare_extra_refusal_and_cell_22_controls(
    lines_by_arm: &BTreeMap<&'static str, Vec<ProductLine>>,
    today_tested: &BTreeMap<String, &ProductLine>,
    sentences: &mut Vec<SentenceRecord>,
) {
    for (arm_name, lines) in lines_by_arm {
        let Some(arm) = fourth_run_arm_named(arm_name) else {
            continue;
        };
        if arm.is_today() || arm.witness == FourthRunWitness::NotJudged {
            continue;
        }
        let tested = tested_mount_lines(lines);
        let (family, fixed) = match arm.witness {
            FourthRunWitness::UnreadableRootRingSlot => ("l2", "k=2"),
            FourthRunWitness::SystemConfiguration
            | FourthRunWitness::SystemConfigurationCarried
            | FourthRunWitness::NotJudged => ("l3", "which=a"),
        };
        let mut found = None;
        for overwrites in [1u64, 2, 3] {
            let key_part = format!(
                "family={family} geometry={} n1={overwrites} {fixed} form=read_fails",
                GEOMETRY_PRIMARY.label
            );
            let today_cell = today_tested
                .iter()
                .find(|(key, _)| key.starts_with("l ") && key.contains(&key_part))
                .map(|(_, cell)| *cell);
            if today_cell.is_some_and(|cell| class_is_a_successful_mount(cell.text("a_class"))) {
                let arm_cell = tested
                    .iter()
                    .find(|(key, _)| key.starts_with("l ") && key.contains(&key_part))
                    .map(|(_, cell)| *cell);
                found = Some((overwrites, arm_cell));
                break;
            }
        }
        let mut push = |name: &str, tag: SentenceTag, verdict: SentenceVerdict, detail: String| {
            let record = SentenceRecord {
                arm: arm_name.to_string(),
                control: "PC-extra-refusal".to_string(),
                name: name.to_string(),
                tag,
                verdict,
            };
            emit_cross_arm_sentence(&record, &detail);
            sentences.push(record);
        };
        match found {
            None => push(
                "today_ok_on_the_cell",
                SentenceTag::TodayDescription,
                SentenceVerdict::NotConstructible,
                "n1=1,2,3".to_string(),
            ),
            Some((overwrites, arm_cell)) => {
                push(
                    "today_ok_on_the_cell",
                    SentenceTag::TodayDescription,
                    SentenceVerdict::Pass,
                    format!("n1={overwrites}"),
                );
                let refused = arm_cell.is_some_and(|cell| cell.text("refused") == "true");
                let truth_newer =
                    arm_cell.is_some_and(|cell| cell.text("truth_newer_on_the_twin") == "true");
                push(
                    "arm_refuses",
                    SentenceTag::ArmDefinition,
                    SentenceVerdict::of(refused),
                    format!("n1={overwrites}"),
                );
                push(
                    "counted_as_extra_refusal",
                    SentenceTag::Measured,
                    SentenceVerdict::conditional(refused, !truth_newer),
                    format!("n1={overwrites} truth_newer={truth_newer}"),
                );
            }
        }
    }
    // PC-22：今天那一臂找一个 m（这次挂载 Ok、计数 0、隔离 0，而它自己的孪生镜像上隔离 > 0）；那个 m 上 甲 拒、乙 O(m 起) 拒 O(m 只) Ok 且隔离 = 孪生；
    // 丙：孪生镜像上那一片被读的次数 = 今天的 − 1、隔离 = 今天的。
    let cell_22_lines = |arm: &str| -> Vec<&ProductLine> {
        lines_by_arm
            .get(arm)
            .map(|lines| {
                lines
                    .iter()
                    .filter(|line| {
                        line.text("name") == "r4_q6_cell"
                            && line.text("geometry") == GEOMETRY_PRIMARY.label
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    let twins = |arm: &str| -> Vec<&ProductLine> {
        lines_by_arm
            .get(arm)
            .map(|lines| {
                lines
                    .iter()
                    .filter(|line| {
                        line.text("name") == "r4_l5_twin"
                            && line.text("geometry") == GEOMETRY_PRIMARY.label
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    let today_cells = cell_22_lines("today");
    let today_twins = twins("today");
    let chosen = today_cells.iter().find(|cell| {
        cell.text("ok") == "true"
            && cell.text("abandoned_roots_unreadable") == "0"
            && cell.text("isolated") == "0"
            && cell.number("twin_isolated").is_some_and(|twin| twin > 0)
    });
    let Some(chosen) = chosen else {
        let record = SentenceRecord {
            arm: "today".to_string(),
            control: "PC-22".to_string(),
            name: "an_m_with_ok_zero_count_zero_isolation".to_string(),
            tag: SentenceTag::TodayDescription,
            verdict: SentenceVerdict::NotConstructible,
        };
        emit_cross_arm_sentence(&record, "route=F2");
        sentences.push(record);
        return;
    };
    let chosen_key = chosen.key(&["n1", "b", "page", "m"]);
    let today_record = SentenceRecord {
        arm: "today".to_string(),
        control: "PC-22".to_string(),
        name: "an_m_with_ok_zero_count_zero_isolation".to_string(),
        tag: SentenceTag::TodayDescription,
        verdict: SentenceVerdict::Pass,
    };
    emit_cross_arm_sentence(&today_record, &chosen_key.replace(' ', ","));
    sentences.push(today_record);
    for arm in FOURTH_RUN_ARMS
        .iter()
        .filter(|arm| !arm.is_today() && lines_by_arm.contains_key(arm.name))
    {
        let cells = cell_22_lines(arm.name);
        let cell_at = |duration_suffix: &str| {
            cells.iter().find(|cell| {
                cell.key(&["n1", "b", "page", "m"]) == chosen_key
                    && cell.text("duration").ends_with(duration_suffix)
            })
        };
        let mut push = |name: &str, verdict: SentenceVerdict, detail: String| {
            let record = SentenceRecord {
                arm: arm.name.to_string(),
                control: "PC-22".to_string(),
                name: name.to_string(),
                tag: SentenceTag::ArmDefinition,
                verdict,
            };
            emit_cross_arm_sentence(&record, &detail);
            sentences.push(record);
        };
        let refused = |cell: Option<&&ProductLine>| {
            cell.map(|cell| SentenceVerdict::of(cell.text("ok") == "false"))
        };
        match arm.candidate {
            FourthRunCandidate::Jia => {
                for suffix in ["only", "from"] {
                    push(
                        &format!("refused_{suffix}"),
                        refused(cell_at(suffix)).unwrap_or(SentenceVerdict::NotConstructible),
                        chosen_key.replace(' ', ","),
                    );
                }
            }
            FourthRunCandidate::Yi | FourthRunCandidate::YiNarrow => {
                push(
                    "refused_from",
                    refused(cell_at("from")).unwrap_or(SentenceVerdict::NotConstructible),
                    chosen_key.replace(' ', ","),
                );
                let only_ok_and_equal = cell_at("only").map(|cell| {
                    SentenceVerdict::of(
                        cell.text("ok") == "true"
                            && cell.text("isolated") == cell.text("twin_isolated"),
                    )
                });
                push(
                    "ok_only_with_twin_isolation",
                    only_ok_and_equal.unwrap_or(SentenceVerdict::NotConstructible),
                    chosen_key.replace(' ', ","),
                );
            }
            FourthRunCandidate::Bing => {
                let base_key = chosen.key(&["n1", "b"]);
                let page: usize = chosen
                    .number("page")
                    .and_then(|page| usize::try_from(page).ok())
                    .unwrap_or(0);
                let own = twins(arm.name)
                    .into_iter()
                    .find(|twin| twin.key(&["n1", "b"]) == base_key);
                let today_twin = today_twins
                    .iter()
                    .find(|twin| twin.key(&["n1", "b"]) == base_key);
                let page_reads = |line: &ProductLine, field: &str| -> Option<i64> {
                    line.text(field)
                        .trim_matches(|character| character == '[' || character == ']')
                        .split(',')
                        .nth(page)
                        .and_then(|value| value.trim().parse().ok())
                };
                let verdict = match (own, today_twin) {
                    (Some(own), Some(today_twin)) => SentenceVerdict::of(
                        page_reads(own, "own_page_reads").is_some_and(|reads| {
                            Some(reads)
                                == page_reads(today_twin, "own_page_reads").map(|today| today - 1)
                        }) && own.text("twin_isolated") == today_twin.text("twin_isolated"),
                    ),
                    (None, _) | (_, None) => SentenceVerdict::NotConstructible,
                };
                push(
                    "page_read_once_less_and_same_isolation_as_today",
                    verdict,
                    chosen_key.replace(' ', ","),
                );
            }
            FourthRunCandidate::Today => {}
        }
    }
}

/// 7.3 第二行（今天那一臂上装置算的 N-配置 / N-槽 为真的被测挂载 = 甲-配置 / 甲-槽 交回「N 判真」拒绝成员的那几格）、第三行（T(δ) 对今天、甲、丙 与 W 逐格相同）、F11。
fn compare_row_two_three_and_clause_eleven(
    lines_by_arm: &BTreeMap<&'static str, Vec<ProductLine>>,
) {
    let Some(today_lines) = lines_by_arm.get("today") else {
        return;
    };
    let today_tested = tested_mount_lines(today_lines);
    for (witness_field, jia_name) in [("device_n_cfg", "jia-cfg"), ("device_n_slot", "jia-slot")] {
        let Some(jia_lines) = lines_by_arm.get(jia_name) else {
            continue;
        };
        let jia_tested = tested_mount_lines(jia_lines);
        let jia = fourth_run_arm_named(jia_name).expect("甲臂");
        let mut compared = 0u64;
        let mut mismatches = Vec::new();
        for (key, today_cell) in &today_tested {
            let Some(jia_cell) = jia_tested.get(key) else {
                continue;
            };
            if today_cell.text(witness_field).is_empty() || key.contains("family=h1g") {
                continue;
            }
            compared += 1;
            let device_says = today_cell.text(witness_field) == "true";
            let jia_refused = jia_cell.text("a_error") == jia.refusal_member();
            if device_says != jia_refused {
                mismatches.push(format!(
                    "{key}: 装置 {witness_field}={device_says}，{jia_name} 拒={jia_refused}"
                ));
            }
        }
        emit_result(&format!(
            "name=r4_section_seven_three row=2 witness={witness_field} jia_arm={jia_name} cells={compared} mismatches={} verdict={}",
            mismatches.len(),
            if mismatches.is_empty() { "pass" } else { "stop_s4" }
        ));
        for mismatch in &mismatches {
            emit_result(&format!(
                "name=r4_compare_finding kind=s4 detail={mismatch:?}"
            ));
        }
    }
    for (arm_name, lines) in lines_by_arm {
        let Some(arm) = fourth_run_arm_named(arm_name) else {
            continue;
        };
        if arm.rereads() {
            continue;
        }
        let tested = tested_mount_lines(lines);
        let mut compared = 0u64;
        let mut differences = Vec::new();
        for (key, cell) in tested.iter().filter(|(key, _)| key.contains("duration=T1")) {
            let Some(whole) = tested.get(&key.replace("duration=T1", "duration=W")) else {
                continue;
            };
            compared += 1;
            for field in [
                "a_class",
                "a_error",
                "a_writes",
                "a_reads",
                "a_chosen",
                "a_effective",
                "q3_read_back",
                "overwrite_cell",
                "q0_destination",
            ] {
                if cell.text(field) != whole.text(field) {
                    differences.push(format!(
                        "{key}: {field} T {} W {}",
                        cell.text(field),
                        whole.text(field)
                    ));
                }
            }
        }
        emit_result(&format!(
            "name=r4_section_seven_three row=3 arm={arm_name} cells={compared} differences={} verdict={}",
            differences.len(),
            if differences.is_empty() { "pass" } else { "void_v5" }
        ));
        for difference in &differences {
            emit_result(&format!(
                "name=r4_compare_finding arm={arm_name} kind=v5 detail={difference:?}"
            ));
        }
    }
    if let (Some(bing_lines), Some(jia_lines)) =
        (lines_by_arm.get("bing-cfg"), lines_by_arm.get("jia-cfg"))
    {
        let bing = tested_mount_lines(bing_lines);
        let jia = tested_mount_lines(jia_lines);
        let mut cells = 0u64;
        let mut differing = Vec::new();
        for (key, bing_cell) in &bing {
            if key.contains("duration=O1only")
                || key.contains("family=l5")
                || key.contains("family=pc554")
            {
                continue;
            }
            let Some(jia_cell) = jia.get(key) else {
                continue;
            };
            cells += 1;
            let jia_refused =
                jia_cell.text("a_error") == "R3NewerRootWitnessedByTheSystemConfiguration";
            let bing_refused =
                bing_cell.text("a_error") == "R3RebuildFromRecordsStoppedBelowTheWitness";
            let same = jia_refused == bing_refused
                && [
                    "a_class",
                    "a_chosen",
                    "a_effective",
                    "q3_read_back",
                    "overwrite_cell",
                ]
                .iter()
                .all(|field| bing_cell.text(field) == jia_cell.text(field));
            if !same {
                differing.push(key.clone());
            }
        }
        emit_result(&format!(
            "name=r4_f11 cells={cells} cells_differing_outside_o1_and_cell_22={} verdict={}",
            differing.len(),
            if differing.is_empty() {
                "inference_holds"
            } else {
                "inference_fails_f11"
            }
        ));
        for key in &differing {
            emit_result(&format!("name=r4_compare_finding kind=f11 detail={key:?}"));
        }
    }
}

/// 起始镜像跨臂同一份（「被测那次挂载之前的镜像一律在今天那一臂上造」的核：前缀在各臂自己进程里造的族，镜像指纹要与今天那一臂的相同；H1g 例外）。
fn compare_base_identity(lines_by_arm: &BTreeMap<&'static str, Vec<ProductLine>>) {
    let Some(today_lines) = lines_by_arm.get("today") else {
        return;
    };
    let today: BTreeMap<String, &str> = today_lines
        .iter()
        .filter(|line| line.text("name") == "r4_prefix")
        .map(|line| {
            (
                line.key(&["family", "geometry", "n1", "c", "n1a", "n1b", "a"]),
                line.text("base_fingerprint"),
            )
        })
        .collect();
    for (arm_name, lines) in lines_by_arm {
        let mut compared = 0u64;
        let mut differing = Vec::new();
        for line in lines
            .iter()
            .filter(|line| line.text("name") == "r4_prefix" && line.text("family") != "h1g")
        {
            let key = line.key(&["family", "geometry", "n1", "c", "n1a", "n1b", "a"]);
            let Some(today_fingerprint) = today.get(&key) else {
                continue;
            };
            compared += 1;
            if line.text("base_fingerprint") != *today_fingerprint {
                differing.push(key);
            }
        }
        emit_result(&format!(
            "name=r4_base_identity arm={arm_name} prefixes={compared} differing={} verdict={}",
            differing.len(),
            if differing.is_empty() {
                "pass"
            } else {
                "differs"
            }
        ));
        for key in &differing {
            emit_result(&format!(
                "name=r4_compare_finding arm={arm_name} kind=base_identity detail={key:?}"
            ));
        }
    }
}

/// 丢写格与 Q6 按臂 × 族汇总（臂自己的账；出局由主 agent 按岔路单原文判）。
fn compare_losses(lines_by_arm: &BTreeMap<&'static str, Vec<ProductLine>>) {
    for (arm_name, lines) in lines_by_arm {
        for line in lines.iter().filter(|line| {
            line.text("name") == "r4_family_summary" || line.text("name") == "r4_random_summary"
        }) {
            emit_result(&format!(
                "name=r4_loss arm={arm_name} family={} geometry={} cells={} lost_write_cells={} q1_overwrite_cells={} q2_rollback_cells={} q3_lost_cells={} q3c_lost_cells={} q3_last_confirmed_cells={}",
                line.text("family"),
                line.text("geometry"),
                line.text("cells"),
                line.text("lost_write_cells_q1_q2_q3"),
                line.text("q1_overwrite_cells"),
                line.text("q2_rollback_cells"),
                line.text("q3_lost_cells"),
                line.text("q3c_lost_cells"),
                line.text("q3_last_confirmed_cells")
            ));
        }
        let cell_22_verdicts: Vec<&ProductLine> = lines
            .iter()
            .filter(|line| line.text("name") == "r4_q6_cell")
            .collect();
        if !cell_22_verdicts.is_empty() {
            let mut by_geometry: BTreeMap<&str, (u64, u64)> = BTreeMap::new();
            for line in cell_22_verdicts {
                let entry = by_geometry.entry(line.text("geometry")).or_insert((0, 0));
                entry.0 += 1;
                if line.text("q6_not_covered") == "true" {
                    entry.1 += 1;
                }
            }
            for (geometry, (cells, not_covered)) in by_geometry {
                emit_result(&format!(
                    "name=r4_q6 arm={arm_name} geometry={geometry} cells={cells} not_covered_cells={not_covered} verdict={}",
                    if not_covered == 0 { "covered" } else { "not_covered" }
                ));
            }
        }
    }
}

#[cfg(test)]
mod fourth_run_tests;
