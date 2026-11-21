//! 片文件：转义、写与读、各合计的写出与读回、片头、合并与原子写。

use super::*;

// ---------------------------------------------------------------------------------------------------------------------
// 片文件：一片 = 一个几何格里的一族（`reading --geometry <几何名> --family <族名> --shard-output <文件>`），`merge` 把齐了的片合成产物
// ---------------------------------------------------------------------------------------------------------------------

/// 片文件的格式标记：改了片文件的写法就换这个值，合并认不得别的写法的片。
const SHARD_FORMAT: &str = "e180-reading-shard-v1";
/// 片文件末行的键：值是整份的行数（含这一行）；合并与长跑脚本按它认这一片做完了。
const SHARD_DONE_KEY: &str = "shard_done";
/// 三条轨迹的名字（产物 `trajectory` 行的 `quantity`），次序同 [`FamilyTally`] 里三个轨迹字段。
pub(super) const TRAJECTORY_NAMES: [&str; 3] = [
    "closed_minus_ring_window",
    "closed_alone",
    "closed_minus_all_common_commitments",
];

fn escape_shard_value(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '\t' => escaped.push_str("\\t"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            other => escaped.push(other),
        }
    }
    escaped
}

fn unescape_shard_value(text: &str) -> Result<String, String> {
    let mut plain = String::with_capacity(text.len());
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            plain.push(character);
            continue;
        }
        match characters.next() {
            Some('\\') => plain.push('\\'),
            Some('t') => plain.push('\t'),
            Some('n') => plain.push('\n'),
            Some('r') => plain.push('\r'),
            other => return Err(format!("bad_escape_after_backslash={other:?}")),
        }
    }
    Ok(plain)
}

/// 片文件的写端：一条一行「键 TAB 转义过的值」，次序就是读端 [`ShardReader`] 取的次序。
#[derive(Default)]
struct ShardWriter {
    pub(super) lines: Vec<String>,
}

impl ShardWriter {
    fn put(&mut self, key: &str, value: impl std::fmt::Display) {
        self.lines
            .push(format!("{key}\t{}", escape_shard_value(&value.to_string())));
    }

    fn put_optional<Value: std::fmt::Display>(&mut self, key: &str, value: Option<Value>) {
        match value {
            Some(present) => self.put(key, present),
            None => self.put(key, "none"),
        }
    }

    /// 先写条数，再一条一行。
    fn put_texts(&mut self, count_key: &str, item_key: &str, texts: &[String]) {
        self.put(count_key, texts.len());
        for text in texts {
            self.put(item_key, text);
        }
    }

    /// 整份文本：末行是 `shard_done TAB <行数>`，每行以换行收尾。
    pub(super) fn finish(mut self) -> String {
        let line_count = self.lines.len() + 1;
        self.lines.push(format!("{SHARD_DONE_KEY}\t{line_count}"));
        let mut text = self.lines.join("\n");
        text.push('\n');
        text
    }
}

/// 片文件的读端：按写端的次序逐条取，键对不上、条数不够、末行不是做完标记都交回错误（合并据此停机）。
struct ShardReader {
    pub(super) records: std::collections::VecDeque<(String, String)>,
}

impl ShardReader {
    pub(super) fn new(text: &str) -> Result<Self, String> {
        if !text.ends_with('\n') {
            return Err("shard_text_does_not_end_with_a_newline".to_string());
        }
        let lines: Vec<&str> = text.lines().collect();
        let expected_done_line = format!("{SHARD_DONE_KEY}\t{}", lines.len());
        match lines.last() {
            Some(last) if *last == expected_done_line => {}
            Some(last) => {
                return Err(format!(
                    "last_line_is_not_the_done_marker last={last:?} expected={expected_done_line:?}"
                ))
            }
            None => return Err("empty_shard".to_string()),
        }
        let mut records = std::collections::VecDeque::new();
        for line in &lines[..lines.len() - 1] {
            let (key, value) = line
                .split_once('\t')
                .ok_or_else(|| format!("line_without_a_tab={line:?}"))?;
            records.push_back((key.to_string(), unescape_shard_value(value)?));
        }
        Ok(ShardReader { records })
    }

    pub(super) fn take(&mut self, key: &str) -> Result<String, String> {
        match self.records.pop_front() {
            Some((found, value)) if found == key => Ok(value),
            Some((found, _)) => Err(format!("expected_key={key} found_key={found}")),
            None => Err(format!("expected_key={key} found_the_end")),
        }
    }

    fn take_parsed<Value: std::str::FromStr>(&mut self, key: &str) -> Result<Value, String> {
        let text = self.take(key)?;
        text.parse::<Value>()
            .map_err(|_| format!("key={key} unparsable_value={text:?}"))
    }

    fn take_optional<Value: std::str::FromStr>(
        &mut self,
        key: &str,
    ) -> Result<Option<Value>, String> {
        let text = self.take(key)?;
        if text == "none" {
            return Ok(None);
        }
        text.parse::<Value>()
            .map(Some)
            .map_err(|_| format!("key={key} unparsable_value={text:?}"))
    }

    fn take_texts(&mut self, count_key: &str, item_key: &str) -> Result<Vec<String>, String> {
        let count: usize = self.take_parsed(count_key)?;
        (0..count).map(|_| self.take(item_key)).collect()
    }

    pub(super) fn finish(self) -> Result<(), String> {
        match self.records.front() {
            None => Ok(()),
            Some((key, _)) => Err(format!("records_left_over_from_key={key}")),
        }
    }
}

fn put_arm_tally(writer: &mut ShardWriter, tally: &ArmTally) {
    let ArmTally {
        over_states,
        not_over_states,
        not_over_below_lower_bound_states,
        undetermined_states,
        maximum_over_bytes,
        maximum_over_at,
        maximum_under_bytes,
        maximum_under_at,
        sum_under_bytes,
        under_states,
    } = tally;
    writer.put("over_states", over_states);
    writer.put("not_over_states", not_over_states);
    writer.put(
        "not_over_below_lower_bound_states",
        not_over_below_lower_bound_states,
    );
    writer.put("undetermined_states", undetermined_states);
    writer.put("max_over_bytes", maximum_over_bytes);
    writer.put("max_over_at", maximum_over_at);
    writer.put("max_under_bytes", maximum_under_bytes);
    writer.put("max_under_at", maximum_under_at);
    writer.put("sum_under_bytes", sum_under_bytes);
    writer.put("under_states", under_states);
}

fn take_arm_tally(reader: &mut ShardReader) -> Result<ArmTally, String> {
    Ok(ArmTally {
        over_states: reader.take_parsed("over_states")?,
        not_over_states: reader.take_parsed("not_over_states")?,
        not_over_below_lower_bound_states: reader
            .take_parsed("not_over_below_lower_bound_states")?,
        undetermined_states: reader.take_parsed("undetermined_states")?,
        maximum_over_bytes: reader.take_parsed("max_over_bytes")?,
        maximum_over_at: reader.take("max_over_at")?,
        maximum_under_bytes: reader.take_parsed("max_under_bytes")?,
        maximum_under_at: reader.take("max_under_at")?,
        sum_under_bytes: reader.take_parsed("sum_under_bytes")?,
        under_states: reader.take_parsed("under_states")?,
    })
}

fn put_residual_tally(writer: &mut ShardWriter, tally: &ResidualTally) {
    let ResidualTally {
        states,
        rewritten_check_residual_nonzero_states,
        original_formula_residual_nonzero_states,
        maximum_absolute_rewritten_check_residual,
        maximum_absolute_original_formula_residual,
        states_with_overlap,
        states_with_held_ledger,
        control_without_overlap_term_stayed_zero,
        first_state_with_a_nonzero_rewritten_check_residual,
        first_state_with_a_nonzero_original_formula_residual,
    } = tally;
    writer.put("residual_states", states);
    writer.put(
        "rewritten_check_residual_nonzero_states",
        rewritten_check_residual_nonzero_states,
    );
    writer.put(
        "original_formula_residual_nonzero_states",
        original_formula_residual_nonzero_states,
    );
    writer.put(
        "maximum_absolute_rewritten_check_residual",
        maximum_absolute_rewritten_check_residual,
    );
    writer.put(
        "maximum_absolute_original_formula_residual",
        maximum_absolute_original_formula_residual,
    );
    writer.put("states_with_overlap", states_with_overlap);
    writer.put("states_with_held_ledger", states_with_held_ledger);
    writer.put(
        "control_without_overlap_term_stayed_zero",
        control_without_overlap_term_stayed_zero,
    );
    writer.put(
        "first_state_with_a_nonzero_rewritten_check_residual",
        first_state_with_a_nonzero_rewritten_check_residual,
    );
    writer.put(
        "first_state_with_a_nonzero_original_formula_residual",
        first_state_with_a_nonzero_original_formula_residual,
    );
}

fn take_residual_tally(reader: &mut ShardReader) -> Result<ResidualTally, String> {
    Ok(ResidualTally {
        states: reader.take_parsed("residual_states")?,
        rewritten_check_residual_nonzero_states: reader
            .take_parsed("rewritten_check_residual_nonzero_states")?,
        original_formula_residual_nonzero_states: reader
            .take_parsed("original_formula_residual_nonzero_states")?,
        maximum_absolute_rewritten_check_residual: reader
            .take_parsed("maximum_absolute_rewritten_check_residual")?,
        maximum_absolute_original_formula_residual: reader
            .take_parsed("maximum_absolute_original_formula_residual")?,
        states_with_overlap: reader.take_parsed("states_with_overlap")?,
        states_with_held_ledger: reader.take_parsed("states_with_held_ledger")?,
        control_without_overlap_term_stayed_zero: reader
            .take_parsed("control_without_overlap_term_stayed_zero")?,
        first_state_with_a_nonzero_rewritten_check_residual: reader
            .take("first_state_with_a_nonzero_rewritten_check_residual")?,
        first_state_with_a_nonzero_original_formula_residual: reader
            .take("first_state_with_a_nonzero_original_formula_residual")?,
    })
}

fn put_trajectory_tally(writer: &mut ShardWriter, tally: &TrajectoryTally) {
    let TrajectoryTally {
        states,
        first_value,
        peak_after_the_first,
        positive_states_after_the_first,
        last_value,
    } = tally;
    writer.put("trajectory_states", states);
    writer.put("first_value", first_value);
    writer.put_optional("peak_after_the_first", *peak_after_the_first);
    writer.put(
        "positive_states_after_the_first",
        positive_states_after_the_first,
    );
    writer.put("last_value", last_value);
}

fn take_trajectory_tally(reader: &mut ShardReader) -> Result<TrajectoryTally, String> {
    Ok(TrajectoryTally {
        states: reader.take_parsed("trajectory_states")?,
        first_value: reader.take_parsed("first_value")?,
        peak_after_the_first: reader.take_optional("peak_after_the_first")?,
        positive_states_after_the_first: reader.take_parsed("positive_states_after_the_first")?,
        last_value: reader.take_parsed("last_value")?,
    })
}

fn put_trajectories(writer: &mut ShardWriter, name: &str, trajectories: &[TrajectoryTally]) {
    writer.put("trajectory_devices", name);
    writer.put("devices", trajectories.len());
    for trajectory in trajectories {
        put_trajectory_tally(writer, trajectory);
    }
}

fn take_trajectories(reader: &mut ShardReader, name: &str) -> Result<Vec<TrajectoryTally>, String> {
    let found = reader.take("trajectory_devices")?;
    if found != name {
        return Err(format!("expected_trajectory={name} found={found}"));
    }
    let devices: usize = reader.take_parsed("devices")?;
    (0..devices)
        .map(|_| take_trajectory_tally(reader))
        .collect()
}

fn put_family_tally(writer: &mut ShardWriter, tally: &FamilyTally) {
    let FamilyTally {
        states,
        exact_states,
        censored_states,
        unreadable_states,
        arms,
        residuals,
        closed_minus_ring_window,
        closed_alone,
        closed_minus_all_common_commitments,
        control_failures,
        shortcut_pairs_checked,
        shortcut_differences,
        records_disagree_with_the_disk_states,
        decomposition_failure_states,
        not_whole_slot_available_states,
        lag_unattributed_states,
        non_monotone_states,
        probes_run,
        stop_clause_held_ledger_mismatch_states,
        unmount_pushes_checked,
        unmount_pushes_differing,
        panics,
    } = tally;
    writer.put("states", states);
    writer.put("exact_states", exact_states);
    writer.put("censored_states", censored_states);
    writer.put("unreadable_states", unreadable_states);
    writer.put("arms", arms.len());
    for (arm, arm_tally) in arms {
        writer.put("arm", arm.label());
        put_arm_tally(writer, arm_tally);
    }
    writer.put("residual_devices", residuals.len());
    for residual in residuals {
        put_residual_tally(writer, residual);
    }
    put_trajectories(writer, TRAJECTORY_NAMES[0], closed_minus_ring_window);
    put_trajectories(writer, TRAJECTORY_NAMES[1], closed_alone);
    put_trajectories(
        writer,
        TRAJECTORY_NAMES[2],
        closed_minus_all_common_commitments,
    );
    writer.put_texts("control_failures", "control_failure", control_failures);
    writer.put("shortcut_pairs_checked", shortcut_pairs_checked);
    writer.put_texts(
        "shortcut_differences",
        "shortcut_difference",
        shortcut_differences,
    );
    writer.put(
        "records_disagree_with_the_disk_states",
        records_disagree_with_the_disk_states,
    );
    writer.put("decomposition_failure_states", decomposition_failure_states);
    writer.put(
        "not_whole_slot_available_states",
        not_whole_slot_available_states,
    );
    writer.put("lag_unattributed_states", lag_unattributed_states);
    writer.put("non_monotone_states", non_monotone_states);
    writer.put("probes_run", probes_run);
    writer.put(
        "stop_clause_held_ledger_mismatch_states",
        stop_clause_held_ledger_mismatch_states,
    );
    writer.put("unmount_pushes_checked", unmount_pushes_checked);
    writer.put_texts(
        "unmount_pushes_differing",
        "unmount_push_differing",
        unmount_pushes_differing,
    );
    writer.put_texts("panics", "panic", panics);
}

fn take_family_tally(reader: &mut ShardReader) -> Result<FamilyTally, String> {
    let states = reader.take_parsed("states")?;
    let exact_states = reader.take_parsed("exact_states")?;
    let censored_states = reader.take_parsed("censored_states")?;
    let unreadable_states = reader.take_parsed("unreadable_states")?;
    let arm_count: usize = reader.take_parsed("arms")?;
    let mut arms = BTreeMap::new();
    for _ in 0..arm_count {
        let label = reader.take("arm")?;
        let arm = all_arms()
            .into_iter()
            .find(|arm| arm.label() == label)
            .ok_or_else(|| format!("unknown_arm={label}"))?;
        arms.insert(arm, take_arm_tally(reader)?);
    }
    let residual_devices: usize = reader.take_parsed("residual_devices")?;
    let residuals = (0..residual_devices)
        .map(|_| take_residual_tally(reader))
        .collect::<Result<Vec<_>, String>>()?;
    let closed_minus_ring_window = take_trajectories(reader, TRAJECTORY_NAMES[0])?;
    let closed_alone = take_trajectories(reader, TRAJECTORY_NAMES[1])?;
    let closed_minus_all_common_commitments = take_trajectories(reader, TRAJECTORY_NAMES[2])?;
    Ok(FamilyTally {
        states,
        exact_states,
        censored_states,
        unreadable_states,
        arms,
        residuals,
        closed_minus_ring_window,
        closed_alone,
        closed_minus_all_common_commitments,
        control_failures: reader.take_texts("control_failures", "control_failure")?,
        shortcut_pairs_checked: reader.take_parsed("shortcut_pairs_checked")?,
        shortcut_differences: reader.take_texts("shortcut_differences", "shortcut_difference")?,
        records_disagree_with_the_disk_states: reader
            .take_parsed("records_disagree_with_the_disk_states")?,
        decomposition_failure_states: reader.take_parsed("decomposition_failure_states")?,
        not_whole_slot_available_states: reader.take_parsed("not_whole_slot_available_states")?,
        lag_unattributed_states: reader.take_parsed("lag_unattributed_states")?,
        non_monotone_states: reader.take_parsed("non_monotone_states")?,
        probes_run: reader.take_parsed("probes_run")?,
        stop_clause_held_ledger_mismatch_states: reader
            .take_parsed("stop_clause_held_ledger_mismatch_states")?,
        unmount_pushes_checked: reader.take_parsed("unmount_pushes_checked")?,
        unmount_pushes_differing: reader
            .take_texts("unmount_pushes_differing", "unmount_push_differing")?,
        panics: reader.take_texts("panics", "panic")?,
    })
}

/// 片头：这一片是哪一格、跑它的那一次的总线程数与探测线程数、本地常量回比了几个；合并核每片的格对得上、几片的这几样一致。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ShardHeader {
    pub(super) geometry_name: String,
    pub(super) family: String,
    pub(super) total_threads: usize,
    pub(super) probe_threads: usize,
    pub(super) local_constants_checked: usize,
}

/// 一片的片文件：片头、这一族的耗时与单元区槽数、锚点、三条轨迹名、产物行、累计；次序就是 [`parse_shard`] 读的次序。
pub(super) fn shard_text(header: &ShardHeader, outcome: &FamilyOutcome) -> String {
    let FamilyOutcome {
        family,
        lines,
        tally,
        anchors,
        seconds,
        unit_area_slots_per_device,
        trajectory_names,
    } = outcome;
    assert_eq!(header.family, *family, "片头的族与这一族的结果一致");
    let mut writer = ShardWriter::default();
    writer.put("shard_format", SHARD_FORMAT);
    writer.put("geometry", &header.geometry_name);
    writer.put("family", &header.family);
    writer.put("total_threads", header.total_threads);
    writer.put("probe_threads", header.probe_threads);
    writer.put("local_constants_checked", header.local_constants_checked);
    writer.put("seconds", seconds);
    writer.put("unit_area_devices", unit_area_slots_per_device.len());
    for slots in unit_area_slots_per_device {
        writer.put("unit_area_slots", slots);
    }
    let FirstFileAnchors {
        open_segment_start_slot,
        closed_free_slots_in_the_open_segment_on_device_zero,
        first_data_unit_slot,
    } = anchors;
    writer.put_optional("open_segment_start_slot", *open_segment_start_slot);
    writer.put_optional(
        "closed_free_slots_in_the_open_segment_on_device_zero",
        *closed_free_slots_in_the_open_segment_on_device_zero,
    );
    writer.put_optional("first_data_unit_slot", *first_data_unit_slot);
    for name in trajectory_names {
        writer.put("trajectory_name", name);
    }
    writer.put_texts("product_lines", "product_line", lines);
    put_family_tally(&mut writer, tally);
    writer.finish()
}

fn parse_shard(text: &str) -> Result<(ShardHeader, FamilyOutcome), String> {
    let mut reader = ShardReader::new(text)?;
    let format = reader.take("shard_format")?;
    if format != SHARD_FORMAT {
        return Err(format!("shard_format={format} expected={SHARD_FORMAT}"));
    }
    let geometry_name = reader.take("geometry")?;
    let family_text = reader.take("family")?;
    let family = PROBED_FAMILIES
        .iter()
        .copied()
        .chain([LONG_MOUNT_FAMILY])
        .find(|known| *known == family_text)
        .ok_or_else(|| format!("unknown_family={family_text}"))?;
    let header = ShardHeader {
        geometry_name,
        family: family.to_string(),
        total_threads: reader.take_parsed("total_threads")?,
        probe_threads: reader.take_parsed("probe_threads")?,
        local_constants_checked: reader.take_parsed("local_constants_checked")?,
    };
    let seconds = reader.take_parsed("seconds")?;
    let unit_area_devices: usize = reader.take_parsed("unit_area_devices")?;
    let unit_area_slots_per_device = (0..unit_area_devices)
        .map(|_| reader.take_parsed::<u64>("unit_area_slots"))
        .collect::<Result<Vec<u64>, String>>()?;
    let anchors = FirstFileAnchors {
        open_segment_start_slot: reader.take_optional("open_segment_start_slot")?,
        closed_free_slots_in_the_open_segment_on_device_zero: reader
            .take_optional("closed_free_slots_in_the_open_segment_on_device_zero")?,
        first_data_unit_slot: reader.take_optional("first_data_unit_slot")?,
    };
    for expected in TRAJECTORY_NAMES {
        let found = reader.take("trajectory_name")?;
        if found != expected {
            return Err(format!("trajectory_name={found} expected={expected}"));
        }
    }
    let lines = reader.take_texts("product_lines", "product_line")?;
    let tally = take_family_tally(&mut reader)?;
    reader.finish()?;
    let outcome = FamilyOutcome {
        family,
        lines,
        tally,
        anchors,
        seconds,
        unit_area_slots_per_device,
        trajectory_names: TRAJECTORY_NAMES,
    };
    Ok((header, outcome))
}

/// 片文件（键是文件名）合成产物的全部行：全部片要齐（[`ALL_GEOMETRIES`] × [`families_of_the_geometry`]），每片装的格与文件名对得上，
/// 几片的总线程数与本地常量回比个数一致、探测线程数等于 [`probe_threads_of_the_geometry`]，不许有多出来的片；产物经 [`reading_product_lines`] 出。
pub(super) fn merge_shard_texts(texts: &BTreeMap<String, String>) -> Result<Vec<String>, String> {
    let mut cells = Vec::new();
    let mut expected_names = BTreeSet::new();
    let mut common_threads_and_constants: Option<(usize, usize)> = None;
    for geometry in ALL_GEOMETRIES {
        let mut outcomes = Vec::new();
        for family in families_of_the_geometry(geometry, None) {
            let name = shard_file_name(geometry, family);
            let text = texts
                .get(&name)
                .ok_or_else(|| format!("missing_shard={name}"))?;
            let (header, outcome) =
                parse_shard(text).map_err(|problem| format!("shard={name} {problem}"))?;
            if header.geometry_name != geometry.name || header.family != family {
                return Err(format!(
                    "shard={name} holds_geometry={} holds_family={}",
                    header.geometry_name, header.family
                ));
            }
            let this_shard = (header.total_threads, header.local_constants_checked);
            let common = *common_threads_and_constants.get_or_insert(this_shard);
            if common != this_shard {
                return Err(format!(
                    "shard={name} total_threads_and_local_constants={this_shard:?} others={common:?}"
                ));
            }
            let expected_probe_threads =
                probe_threads_of_the_geometry(geometry, header.total_threads);
            if header.probe_threads != expected_probe_threads {
                return Err(format!(
                    "shard={name} probe_threads={} expected={expected_probe_threads}",
                    header.probe_threads
                ));
            }
            expected_names.insert(name);
            outcomes.push(outcome);
        }
        cells.push(GeometryCellOutcome { geometry, outcomes });
    }
    if let Some(unexpected) = texts.keys().find(|name| !expected_names.contains(*name)) {
        return Err(format!("unexpected_shard={unexpected}"));
    }
    let (_, local_constants_checked) =
        common_threads_and_constants.expect("ALL_GEOMETRIES 非空，至少读过一片");
    Ok(reading_product_lines(local_constants_checked, &cells))
}

/// 片文件原子落盘：先写同目录的 `<路径>.partial-<进程号>`、刷盘，再换名成 `<路径>`；跑到一半被打断的片不会以 `.shard` 结尾留下。
pub(super) fn write_shard_atomically(path: &str, text: &str) {
    let temporary = format!("{path}.partial-{}", std::process::id());
    let written = std::fs::File::create(&temporary)
        .and_then(|mut file| {
            use std::io::Write;
            file.write_all(text.as_bytes())?;
            file.sync_all()
        })
        .and_then(|()| std::fs::rename(&temporary, path));
    if let Err(error) = written {
        // 删临时文件失败也照停：停机行里已经写明是哪一片没写成，留下的临时文件不以 `.shard` 结尾，合并不认它。
        let _ = std::fs::remove_file(&temporary);
        stop_with("shard_write_failed", &format!("path={path} error={error}"));
    }
}

#[cfg(test)]
mod shard_tests;
