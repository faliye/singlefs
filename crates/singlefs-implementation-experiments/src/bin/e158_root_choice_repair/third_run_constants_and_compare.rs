//! 第 3 次跑的常量回比与锚点、跨臂比对与模式入口。

use super::*;

// ---------------------------------------------------------------------------
// 11.8 开跑检查：5.6 常量回比（V3）、7.2 锚点（V4）、臂与副本对得上（装置编进的臂代码与环境变量说的臂一致）。
// ---------------------------------------------------------------------------

pub(super) fn third_run_local_constants_checks() -> Vec<ConstantCheck> {
    let mut checks = second_run_local_constants_checks();
    checks.extend([
        ConstantCheck {
            name: "单元类标签 数据 UNIT_CLASS_DATA（装置重放取单元宽）",
            local_value: u64::from(UNIT_CLASS_DATA_LOCAL),
            crates_value: u64::from(singlefs_core::unit::UNIT_CLASS_DATA),
        },
        ConstantCheck {
            name: "单元类标签 索引节点 UNIT_CLASS_INDEX_NODE",
            local_value: u64::from(UNIT_CLASS_INDEX_NODE_LOCAL),
            crates_value: u64::from(singlefs_core::unit::UNIT_CLASS_INDEX_NODE),
        },
        ConstantCheck {
            name: "单元类标签 打包 UNIT_CLASS_PACKED",
            local_value: u64::from(UNIT_CLASS_PACKED_LOCAL),
            crates_value: u64::from(singlefs_core::unit::UNIT_CLASS_PACKED),
        },
        ConstantCheck {
            name: "数据单元宽（装置重放用的那一个）DATA_UNIT_BYTES",
            local_value: DATA_UNIT_BYTES_LOCAL as u64,
            crates_value: singlefs_format::DATA_UNIT_BYTES,
        },
        ConstantCheck {
            name: "节点宽（装置重放用的那一个）NODE_BYTES",
            local_value: NODE_BYTES_LOCAL as u64,
            crates_value: singlefs_format::NODE_BYTES,
        },
        ConstantCheck {
            name: "系统配置整槽校验和偏移（PC-N0 指纹抹掉的那一段）",
            local_value: SYSTEM_CONFIGURATION_CHECKSUM_OFFSET_LOCAL as u64,
            crates_value: u64::try_from(
                singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            )
            .expect("155"),
        },
    ]);
    checks
}

fn run_third_run_constants_and_anchors(arm: &str) -> bool {
    let mut all_passed = true;
    for check in third_run_local_constants_checks() {
        let passed = check.matches();
        all_passed &= passed;
        emit_result(&format!(
            "name=r3_local_constant_check arm={arm} item={:?} verdict={} local_value={} crates_value={}",
            check.name,
            verdict_word(passed),
            check.local_value,
            check.crates_value
        ));
    }
    let (regions_match, local_regions, crates_regions) = region_devices_matches_crates();
    all_passed &= regions_match;
    emit_result(&format!(
        "name=r3_local_constant_check arm={arm} item=region_devices verdict={} local_value={local_regions:?} crates_value={crates_regions:?}",
        verdict_word(regions_match)
    ));
    let arm_known = THIRD_RUN_ARMS.contains(&arm);
    let arm_code_matches =
        arm_known && (arm == "today") != third_run_arm_hooks::arm_code_is_compiled_in();
    all_passed &= arm_code_matches;
    emit_result(&format!(
        "name=r3_arm_code_check arm={arm} verdict={} arm_known={arm_known} arm_code_compiled_in={}",
        verdict_word(arm_code_matches),
        third_run_arm_hooks::arm_code_is_compiled_in()
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
                "name=r3_section_seven_two_anchor arm={arm} item=\"journal 实例代号偏移 469 + 8；F 偏移 489 − 8\" verdict={} tail_offset={tail} instance_offset={instance} floor_offset={floor} registered_instance=477 registered_floor=481",
                verdict_word(passed)
            ));
        }
        Err(error) => {
            all_passed = false;
            emit_result(&format!(
                "name=r3_section_seven_two_anchor arm={arm} item=offsets verdict=fail reason={error:?}"
            ));
        }
    }
    // 7.2：根槽总数 3 × S 与环槽数（按装置在 G0 上现枚举的数），系统配置槽数 2 × 2。
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let mut devices = new_devices(IMAGE_BYTES);
    let geometry_anchor = make_filesystem(&parameters, &mut devices)
        .map_err(|error| format!("{error:?}"))
        .and_then(|_| independent_geometry(&memory_pool_of(&devices, IMAGE_BYTES)));
    match geometry_anchor {
        Ok(geometry) => {
            let pool = memory_pool_of(&devices, IMAGE_BYTES);
            let root_slots = checker_image::root_slot_positions(&geometry).len() as u64;
            let ring_slots = geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES;
            let configuration_slots = device_decode(&pool, &geometry)
                .system_configuration_slots
                .len() as u64;
            for (item, computed, registered) in [
                (
                    "根槽总数 3×S，S=8（装置在 G0 上枚举的槽数）",
                    root_slots,
                    24u64,
                ),
                ("环槽数 3MiB÷4096（装置在 G0 上的环槽数）", ring_slots, 768),
                (
                    "每个池的系统配置槽数 2×2（装置读到的自证槽数）",
                    configuration_slots,
                    4,
                ),
            ] {
                let passed = computed == registered;
                all_passed &= passed;
                emit_result(&format!(
                    "name=r3_section_seven_two_anchor arm={arm} item={item:?} verdict={} computed={computed} registered={registered}",
                    verdict_word(passed)
                ));
            }
        }
        Err(error) => {
            all_passed = false;
            emit_result(&format!(
                "name=r3_section_seven_two_anchor arm={arm} item=geometry verdict=fail reason={error:?}"
            ));
        }
    }
    all_passed
}

// ---------------------------------------------------------------------------
// 11.9 跨臂对拍（`r3-compare`）：读九条臂各自的产物，算 Q5（多读）、PC-多读、PC-N0 的逐项相同（F12）、
//      7.3 第二行（装置在今天那一臂上算的 N 与甲臂的拒一一对应，S4）、第三行（T(δ) 对今天、甲、丙与 W 逐格相同，V5）、
//      第四行（今天减自己 = 0）、F11（丙-配置 与 甲-配置 只在 O(1 只) 那几格不同）。
// ---------------------------------------------------------------------------

/// 产物里一行 `E7RESULT name=… k=v …`：按空格切，值里不带空格的那几个字段取得出（带引号的说明字段不取）。
pub(super) fn fields_of_a_result_line(line: &str) -> BTreeMap<String, String> {
    line.trim_start_matches("E7RESULT ")
        .split(' ')
        .filter_map(|token| token.split_once('='))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

/// 一格的键：族与坐标（不含臂）。
fn cell_key_of(fields: &BTreeMap<String, String>, coordinate_names: &[&str]) -> String {
    coordinate_names
        .iter()
        .map(|name| format!("{name}={}", fields.get(*name).map_or("", String::as_str)))
        .collect::<Vec<_>>()
        .join(" ")
}

struct ComparedCell {
    fields: BTreeMap<String, String>,
}

/// 从一条臂的产物里取 H1d 不断那一格、H1e（每个 (n1, b, 造法, 时长) 只取 m = 1 那一行，A 的读数与 m 无关）、PC-N、PC-O、PC-N0 的行，按格的键收。
fn compared_cells_of(text: &str) -> BTreeMap<String, ComparedCell> {
    let mut cells = BTreeMap::new();
    for line in text.lines() {
        let fields = fields_of_a_result_line(line);
        let name = fields.get("name").map_or("", String::as_str);
        let key = match name {
            "r3_h1d_cell" if fields.get("cut").map(String::as_str) == Some("none") => format!(
                "h1d {}",
                cell_key_of(&fields, &["n1", "b", "form", "duration"])
            ),
            "r3_h1e_cell" if fields.get("m").map(String::as_str) == Some("1") => format!(
                "h1e {}",
                cell_key_of(&fields, &["n1", "b", "form", "duration"])
            ),
            "r3_positive_control" => {
                let control = fields.get("control").map_or("", String::as_str);
                match control {
                    "PC-N" | "PC-O" => format!("{control} {}", cell_key_of(&fields, &["duration"])),
                    "PC-N0" => format!("{control} {}", cell_key_of(&fields, &["n1", "c"])),
                    _ => continue,
                }
            }
            _ => continue,
        };
        cells.insert(key, ComparedCell { fields });
    }
    cells
}

fn field_u64(cell: &ComparedCell, name: &str) -> Option<u64> {
    cell.fields.get(name).and_then(|value| value.parse().ok())
}

fn field_text<'cell>(cell: &'cell ComparedCell, name: &str) -> &'cell str {
    cell.fields.get(name).map_or("", String::as_str)
}

#[derive(Default)]
pub(super) struct ExtraReads {
    pub(super) cells: u64,
    pub(super) positive_cells: u64,
    pub(super) total_calls: i64,
    pub(super) total_bytes: i64,
    pub(super) minimum_calls: Option<i64>,
    pub(super) maximum_calls: Option<i64>,
}

impl ExtraReads {
    pub(super) fn add(&mut self, calls: i64, bytes: i64) {
        self.cells += 1;
        if calls > 0 {
            self.positive_cells += 1;
        }
        self.total_calls += calls;
        self.total_bytes += bytes;
        self.minimum_calls = Some(
            self.minimum_calls
                .map_or(calls, |current| current.min(calls)),
        );
        self.maximum_calls = Some(
            self.maximum_calls
                .map_or(calls, |current| current.max(calls)),
        );
    }
}

fn run_third_run_compare(prefix: &str) -> bool {
    let mut texts = BTreeMap::new();
    for arm in THIRD_RUN_ARMS {
        let path = format!("{prefix}-{arm}.out");
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                texts.insert(arm, text);
            }
            Err(error) => {
                emit_result(&format!(
                    "name=r3_compare_input arm={arm} verdict=stop reason={:?}",
                    format!("{path}: {error}")
                ));
                return false;
            }
        }
    }
    for (arm, text) in &texts {
        let finished = text
            .lines()
            .last()
            .is_some_and(|line| line.starts_with("E7RESULT name=done emitted="));
        emit_result(&format!(
            "name=r3_compare_input arm={arm} verdict={} lines={}",
            if finished { "pass" } else { "stop_unfinished" },
            text.lines().count()
        ));
        if !finished {
            return false;
        }
    }
    let cells: BTreeMap<&str, BTreeMap<String, ComparedCell>> = texts
        .iter()
        .map(|(arm, text)| (*arm, compared_cells_of(text)))
        .collect();
    let today = &cells["today"];
    // 7.3 第四行：今天减自己 = 0（每一格，装置自检）。
    let self_difference_cells = today
        .values()
        .filter(|cell| field_u64(cell, "a_reads").is_some())
        .count();
    emit_result(&format!(
        "name=r3_section_seven_three arm=today row=4 item=\"今天那一臂减自己的多读 = 0\" cells={self_difference_cells} nonzero_cells=0 verdict=pass"
    ));
    for arm in THIRD_RUN_ARMS.into_iter().filter(|arm| *arm != "today") {
        let arm_cells = &cells[arm];
        // Q5 多读：H1d 不断那一格、H1e、PC-N（不断），臂减今天，同一格、同一份起始镜像。
        let mut by_family: BTreeMap<&str, ExtraReads> = BTreeMap::new();
        for (key, cell) in arm_cells {
            let Some(today_cell) = today.get(key) else {
                continue;
            };
            let family = key.split(' ').next().unwrap_or("");
            if !matches!(family, "h1d" | "h1e" | "PC-N") {
                continue;
            }
            let (Some(calls), Some(today_calls), Some(bytes), Some(today_bytes)) = (
                field_u64(cell, "a_reads"),
                field_u64(today_cell, "a_reads"),
                field_u64(cell, "a_read_bytes"),
                field_u64(today_cell, "a_read_bytes"),
            ) else {
                continue;
            };
            by_family.entry(family).or_default().add(
                i64::try_from(calls).expect("读次数") - i64::try_from(today_calls).expect("读次数"),
                i64::try_from(bytes).expect("读字节") - i64::try_from(today_bytes).expect("读字节"),
            );
        }
        for (family, extra) in &by_family {
            emit_result(&format!(
                "name=r3_q5_extra_reads arm={arm} family={family} cells={} positive_cells={} total_extra_read_calls={} total_extra_read_bytes={} minimum_extra_read_calls={} maximum_extra_read_calls={}",
                extra.cells,
                extra.positive_cells,
                extra.total_calls,
                extra.total_bytes,
                extra.minimum_calls.map_or("none".to_string(), |value| value.to_string()),
                extra.maximum_calls.map_or("none".to_string(), |value| value.to_string())
            ));
        }
        // PC-多读（PC-N，W）：乙 ≥ R ×（第一遍里读失败的落点个数）且 > 0；丙 > 0。
        let whole_call_key = "PC-N duration=W".to_string();
        if let (Some(cell), Some(today_cell)) =
            (arm_cells.get(&whole_call_key), today.get(&whole_call_key))
        {
            let extra = i64::try_from(field_u64(cell, "a_reads").unwrap_or(0)).expect("读次数")
                - i64::try_from(field_u64(today_cell, "a_reads").unwrap_or(0)).expect("读次数");
            let faults = i64::try_from(field_u64(cell, "faults").unwrap_or(0)).expect("落点数");
            let (expectation, passed) = match ThirdRunCandidate::of(arm) {
                Some(ThirdRunCandidate::Yi) => (
                    "extra_reads_at_least_r_times_faults_and_positive",
                    extra >= i64::try_from(THIRD_RUN_REREAD_ROUNDS).expect("R") * faults
                        && extra > 0,
                ),
                Some(ThirdRunCandidate::Bing) => ("extra_reads_positive", extra > 0),
                Some(ThirdRunCandidate::Jia | ThirdRunCandidate::Today) | None => {
                    ("not_registered_for_this_arm", true)
                }
            };
            emit_result(&format!(
                "name=r3_positive_control arm={arm} control=PC-extra-reads verdict={} expectation={expectation} extra_read_calls={extra} faults={faults}",
                verdict_word(passed)
            ));
        } else {
            emit_result(&format!(
                "name=r3_positive_control arm={arm} control=PC-extra-reads verdict=not_constructible reason=\"PC-N W 那一行缺\""
            ));
        }
        // PC-N0 与今天逐项相同（F12）：K 类、所选根、E、写的指纹（-配置续 取号那 8 个字节已抹）。
        let mut no_newer_root_cells = 0u64;
        let mut no_newer_root_differences = Vec::new();
        for (key, cell) in arm_cells
            .iter()
            .filter(|(cell_key, _)| cell_key.starts_with("PC-N0"))
        {
            no_newer_root_cells += 1;
            let Some(today_cell) = today.get(key) else {
                no_newer_root_differences.push(format!("{key}: 今天那一臂没有这一格"));
                continue;
            };
            for field in ["a_class", "a_chosen", "a_effective", "write_fingerprint"] {
                if field_text(cell, field) != field_text(today_cell, field) {
                    no_newer_root_differences.push(format!(
                        "{key}: {field} 臂 {} 今天 {}",
                        field_text(cell, field),
                        field_text(today_cell, field)
                    ));
                }
            }
        }
        emit_result(&format!(
            "name=r3_positive_control arm={arm} control=PC-N0-same-as-today verdict={} cells={no_newer_root_cells} differences={}",
            verdict_word(no_newer_root_differences.is_empty() && no_newer_root_cells == 8),
            no_newer_root_differences.len()
        ));
        for difference in &no_newer_root_differences {
            emit_result(&format!(
                "name=r3_compare_finding arm={arm} kind=f12 detail={difference:?}"
            ));
        }
    }
    // 7.3 第二行：今天那一臂上装置算的 N-配置 / N-槽 为真的格 = 甲-配置 / 甲-槽 交回「N 判真」拒绝成员的格。
    for (witness_field, jia_arm) in [("device_n_cfg", "jia-cfg"), ("device_n_slot", "jia-slot")] {
        let mut compared = 0u64;
        let mut mismatches = Vec::new();
        for (key, today_cell) in today {
            let Some(jia_cell) = cells[jia_arm].get(key) else {
                continue;
            };
            if field_text(today_cell, witness_field).is_empty() {
                continue;
            }
            compared += 1;
            let device_says = field_text(today_cell, witness_field) == "true";
            let jia_refused = field_text(jia_cell, "a_error") == third_run_refusal_member(jia_arm);
            if device_says != jia_refused {
                mismatches.push(format!(
                    "{key}: 装置 {witness_field}={device_says}，{jia_arm} 拒={jia_refused}"
                ));
            }
        }
        emit_result(&format!(
            "name=r3_section_seven_three row=2 witness={witness_field} jia_arm={jia_arm} cells={compared} mismatches={} verdict={}",
            mismatches.len(),
            if mismatches.is_empty() { "pass" } else { "stop_s4" }
        ));
        for mismatch in &mismatches {
            emit_result(&format!(
                "name=r3_compare_finding kind=s4 detail={mismatch:?}"
            ));
        }
    }
    // 7.3 第三行：T(δ) 对今天、甲、丙各臂与 W 逐格相同（H1d 不断那一格与 PC-N）。
    for arm in THIRD_RUN_ARMS.into_iter().filter(|arm| {
        matches!(
            ThirdRunCandidate::of(arm),
            Some(ThirdRunCandidate::Today | ThirdRunCandidate::Jia | ThirdRunCandidate::Bing)
        )
    }) {
        let mut compared = 0u64;
        let mut differences = Vec::new();
        for (key, cell) in cells[arm]
            .iter()
            .filter(|(key, _)| key.contains("duration=T1"))
        {
            let whole_key = key.replace("duration=T1", "duration=W");
            let Some(whole_cell) = cells[arm].get(&whole_key) else {
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
            ] {
                if field_text(cell, field) != field_text(whole_cell, field) {
                    differences.push(format!(
                        "{key}: {field} T {} W {}",
                        field_text(cell, field),
                        field_text(whole_cell, field)
                    ));
                }
            }
        }
        emit_result(&format!(
            "name=r3_section_seven_three row=3 arm={arm} cells={compared} differences={} verdict={}",
            differences.len(),
            if differences.is_empty() { "pass" } else { "void_v5" }
        ));
        for difference in &differences {
            emit_result(&format!(
                "name=r3_compare_finding arm={arm} kind=v5 detail={difference:?}"
            ));
        }
    }
    // F11：丙-配置 与 甲-配置 只在第 22 条那几格与 O(1 只) 那几格不同（第四节推的 ③）。
    let mut failure_clause_eleven_cells = 0u64;
    let mut failure_clause_eleven_differences = Vec::new();
    for (key, bing_cell) in &cells["bing-cfg"] {
        if key.contains("duration=O1only") || key.starts_with("PC-O") {
            continue;
        }
        let Some(jia_cell) = cells["jia-cfg"].get(key) else {
            continue;
        };
        failure_clause_eleven_cells += 1;
        let jia_refused = field_text(jia_cell, "a_error") == third_run_refusal_member("jia-cfg");
        let bing_refused = field_text(bing_cell, "a_error") == third_run_refusal_member("bing-cfg");
        let same = jia_refused == bing_refused
            && field_text(bing_cell, "a_class") == field_text(jia_cell, "a_class")
            && field_text(bing_cell, "a_chosen") == field_text(jia_cell, "a_chosen")
            && field_text(bing_cell, "a_effective") == field_text(jia_cell, "a_effective")
            && field_text(bing_cell, "q3_read_back") == field_text(jia_cell, "q3_read_back")
            && field_text(bing_cell, "overwrite_cell") == field_text(jia_cell, "overwrite_cell");
        if !same {
            failure_clause_eleven_differences.push(key.clone());
        }
    }
    emit_result(&format!(
        "name=r3_f11 cells={failure_clause_eleven_cells} cells_differing_outside_o1_and_cell_22={} verdict={}",
        failure_clause_eleven_differences.len(),
        if failure_clause_eleven_differences.is_empty() { "inference_holds" } else { "inference_fails_f11" }
    ));
    for difference in &failure_clause_eleven_differences {
        emit_result(&format!(
            "name=r3_compare_finding kind=f11 detail={difference:?}"
        ));
    }
    true
}

/// 第 3 次跑的模式：开跑检查不过就停（V3 / V4），过了跑点名的那一段。交回 true = 这个模式归第 3 次跑。
pub(super) fn run_third_run_mode(mode: &str, command_line_arguments: &[String]) -> bool {
    let known = [
        "r3-seg1",
        "r3-constants",
        "r3-pc",
        "r3-h1d",
        "r3-h1e",
        "r3-compare",
    ];
    if !known.contains(&mode) {
        return false;
    }
    emit_input_header();
    if mode == "r3-compare" {
        let prefix = command_line_arguments
            .get(2)
            .cloned()
            .unwrap_or_else(|| "research/results/e158-root-choice-repair-r3-seg1".to_string());
        run_third_run_compare(&prefix);
        emit_result(&format!(
            "name=done emitted={}",
            emitted_result_line_count() + 1
        ));
        return true;
    }
    let arm = third_run_arm_label();
    if !run_third_run_constants_and_anchors(&arm) {
        emit_result(&format!(
            "name=stop arm={arm} reason=V3 detail=\"第 3 次跑的常量回比、第七节锚点或臂与副本的对应有不过的，整轮不开跑\""
        ));
        emit_result(&format!(
            "name=done emitted={}",
            emitted_result_line_count() + 1
        ));
        return true;
    }
    match mode {
        "r3-seg1" => {
            run_third_run_positive_control_newer(&GEOMETRY_PRIMARY, &arm);
            run_third_run_positive_control_second_read(&GEOMETRY_PRIMARY, &arm);
            run_third_run_positive_control_no_newer(&GEOMETRY_PRIMARY, &arm);
            run_third_run_held_once_family(&GEOMETRY_PRIMARY, &arm);
            run_third_run_tail_family(&GEOMETRY_PRIMARY, &arm);
        }
        "r3-constants" => {}
        "r3-pc" => {
            run_third_run_positive_control_newer(&GEOMETRY_PRIMARY, &arm);
            run_third_run_positive_control_second_read(&GEOMETRY_PRIMARY, &arm);
            run_third_run_positive_control_no_newer(&GEOMETRY_PRIMARY, &arm);
        }
        "r3-h1d" => run_third_run_held_once_family(&GEOMETRY_PRIMARY, &arm),
        "r3-h1e" => run_third_run_tail_family(&GEOMETRY_PRIMARY, &arm),
        other => unreachable!("第 3 次跑的模式表 known 已经挡过：{other}"),
    }
    emit_result(&format!(
        "name=done emitted={}",
        emitted_result_line_count() + 1
    ));
    true
}

/// 产物文件头（登记开头「产物文件头第一行写这个汇总 sha256」）：快照的汇总 sha256 由跑的人经环境变量交进来；
/// `E7INPUT` 开头，`scripts/experiment-apparatus/replay.sh` 比对前删掉，`name=done` 不数它。
pub(super) fn emit_input_header() {
    println!(
        "E7INPUT name=crates_snapshot key=E158 sha256={}",
        env::var("SINGLEFS_E158_SNAPSHOT_SHA256").unwrap_or_else(|_| "unset".to_string())
    );
}

#[cfg(test)]
mod third_run_tests;
