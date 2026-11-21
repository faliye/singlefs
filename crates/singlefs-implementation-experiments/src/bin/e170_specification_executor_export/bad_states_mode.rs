//! 第三段：`bad-states` 模式的导出、报告与命令行。

use super::*;

/// 一条流造完、判完的坏状态：底态（带崩溃状态序号对照）、行、实现那一侧的观测、回比。
struct BadStreamExport {
    pub(super) name: StreamName,
    pub(super) bases: Vec<(BaseState, Option<u64>, usize)>,
    pub(super) rows: Vec<BadRow>,
    pub(super) observations: Vec<BadObservation>,
    pub(super) checks: Vec<StopCheck>,
    pub(super) files: Vec<WrittenFile>,
}

fn implementation_verdict_line(row_identifier: usize, observation: &BadObservation) -> String {
    match &observation.verdict {
        Ok(verdict) => format!(
            "{row_identifier}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            verdict.consulted_root,
            verdict.consulted_outcome,
            verdict.ignored_root,
            verdict.ignored_outcome,
            field_text(&list_text(&verdict.red_items)),
            list_text(&verdict.pool_checker_red),
            verdict.root_without_record,
            verdict.claimed_state_missing_unit,
            verdict.pool_checker_and_record_checker_both_pass()
        ),
        Err(panic) => format!(
            "{row_identifier}\tpanicked:{}\t-\t-\t-\t-\t-\t-\t-\tfalse\n",
            field_text(panic)
        ),
    }
}

/// 一条流：重驱动、核导出、造表、判、写五个文件（写之前 `run_bad_states` 已核过五个文件都不存在）。
fn bad_states_of_stream(
    name: StreamName,
    dump_root: &Path,
    seed_hex: &str,
    seed: u64,
    generator: &mut SplitMix64,
    table_version: BadTableVersion,
) -> BadStreamExport {
    let stream = prepare_stream(name);
    let directory = dump_root.join(name.directory_name());
    let mut checks = stream.checks.clone();
    checks.extend(dump_agreement_checks(&stream, &directory));
    let tables = crash_state_enumeration_tables(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let flow_facts = extract_flow_facts(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let torn_image_index_by_write: BTreeMap<usize, usize> = tables
        .torn_images
        .iter()
        .map(|row| (row.write_index, row.torn_image_index))
        .collect();
    let facts = bad_stream_facts(&stream);
    checks.extend(bad_local_constant_checks(&facts, &facts.all_persisted()));
    let (bases, rows) = bad_rows_of_stream(&facts, generator, table_version, &mut checks);
    let mut matched_bases: Vec<(BaseState, Option<u64>, usize)> = Vec::new();
    for base in bases.iter().chain(rows.iter().map(|row| &row.base)) {
        if matched_bases
            .iter()
            .any(|(seen, _, _)| seen.label == base.label)
        {
            continue;
        }
        let (found, candidates) = crash_state_ordinal_with_the_same_landing(
            stream.writes.len(),
            &tables,
            &flow_facts,
            &torn_image_index_by_write,
            &base.persisted,
        );
        matched_bases.push((base.clone(), found, candidates));
    }
    let observations: Vec<BadObservation> = rows
        .iter()
        .map(|row| observe_bad_row(&stream, row))
        .collect();
    if table_version == BadTableVersion::SecondRun {
        checks.extend(lower_bound_row_base_checks(
            name.directory_name(),
            &rows,
            &observations,
        ));
    }
    let mut states_text = String::new();
    let mut patches_bytes = Vec::new();
    let mut expected_text = String::new();
    let mut verdicts_text = String::new();
    for (row_identifier, (row, observation)) in rows.iter().zip(&observations).enumerate() {
        let landing: String = row
            .base
            .persisted
            .iter()
            .map(|is_persisted| if *is_persisted { '2' } else { '0' })
            .collect();
        let patch_list: Vec<String> = row
            .patches
            .iter()
            .map(|patch| {
                let start = patches_bytes.len();
                patches_bytes.extend_from_slice(&patch.bytes);
                format!(
                    "{}:{}:{start}:{}",
                    patch.device.0,
                    patch.offset,
                    patch.bytes.len()
                )
            })
            .collect();
        states_text.push_str(&format!(
            "{row_identifier}\t{landing}\t{}\t{}\t{}\n",
            observation.base_digest,
            observation.patched_digest,
            patch_list.join(",")
        ));
        let crash_state_ordinal_of_the_matched_base = matched_bases
            .iter()
            .find(|(seen, _, _)| seen.label == row.base.label)
            .and_then(|(_, found, _)| *found)
            .map_or_else(|| "none".to_string(), |ordinal| ordinal.to_string());
        expected_text.push_str(&format!(
            "{row_identifier}\t{}\t{}\t{}\t{}\t{}\t{}\t{crash_state_ordinal_of_the_matched_base}\t{}\n",
            row.name,
            row.family.label(),
            row.base.label,
            row.base
                .call
                .map_or_else(|| "-".to_string(), |call| call.to_string()),
            row.expected_verdict,
            row.expected_landing.text(),
            row.base.notes
        ));
        verdicts_text.push_str(&implementation_verdict_line(row_identifier, observation));
    }
    let seed_text = format!("seed_hex\t{seed_hex}\nseed_u64\t{seed}\n");
    let files = vec![
        write_export_file(&directory, BAD_STATES_FILE, states_text.as_bytes()),
        write_export_file(&directory, BAD_PATCHES_FILE, &patches_bytes),
        write_export_file(&directory, BAD_EXPECTED_FILE, expected_text.as_bytes()),
        write_export_file(
            &directory,
            BAD_IMPLEMENTATION_VERDICTS_FILE,
            verdicts_text.as_bytes(),
        ),
        write_export_file(&directory, BAD_SEED_FILE, seed_text.as_bytes()),
    ];
    BadStreamExport {
        name,
        bases: matched_bases,
        rows,
        observations,
        checks,
        files,
    }
}

/// 一条流的坏状态结果行；交回 (回比没过的条数, 实现判定 panic 的行数)。
fn report_bad_stream(lines: &mut ResultLines, export: &BadStreamExport) -> (usize, usize) {
    let label = export.name.directory_name();
    let mut failed_checks = 0usize;
    for check in &export.checks {
        failed_checks += usize::from(!check.holds());
        lines.emit_result_line(&format!(
            "name=bad_s1 stream={label} check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    for (base, found, candidates) in &export.bases {
        lines.emit_result_line(&format!(
            "name=bad_base stream={label} base={} call={} persisted_writes={} layer0_match={} layer0_candidates={candidates} notes={}",
            base.label,
            base.call.map_or_else(|| "-".to_string(), |call| call.to_string()),
            base.persisted.iter().filter(|is_persisted| **is_persisted).count(),
            found.map_or_else(|| "none".to_string(), |ordinal| ordinal.to_string()),
            field_text(&base.notes)
        ));
    }
    let mut panics = 0usize;
    let mut both_pass_by_family: BTreeMap<&'static str, (usize, usize)> = BTreeMap::new();
    for (row_identifier, (row, observation)) in
        export.rows.iter().zip(&export.observations).enumerate()
    {
        let (verdict_text, both_pass) = match &observation.verdict {
            Ok(verdict) => (
                format!(
                    "impl_consulted_root={} impl_consulted_outcome={} impl_ignored_root={} impl_pool_checker_red={} impl_root_without_record={} impl_claimed_state_missing_unit={}",
                    verdict.consulted_root,
                    verdict.consulted_outcome,
                    verdict.ignored_root,
                    field_text(&list_text(&verdict.pool_checker_red)),
                    verdict.root_without_record,
                    verdict.claimed_state_missing_unit
                ),
                verdict.pool_checker_and_record_checker_both_pass(),
            ),
            Err(panic) => {
                panics += 1;
                (format!("impl_panicked={}", field_text(panic)), false)
            }
        };
        let tally = both_pass_by_family
            .entry(row.family.label())
            .or_insert((0, 0));
        tally.0 += 1;
        tally.1 += usize::from(both_pass);
        lines.emit_result_line(&format!(
            "name=bad_row stream={label} row={row_identifier} name={} family={} base={} patches={} patched_bytes={} expected={} expected_landing={} base_digest={} patched_digest={} {verdict_text} impl_both_pass={both_pass}",
            row.name,
            row.family.label(),
            row.base.label,
            row.patches.len(),
            row.patches.iter().map(|patch| patch.bytes.len()).sum::<usize>(),
            row.expected_verdict,
            row.expected_landing.text(),
            observation.base_digest,
            observation.patched_digest
        ));
    }
    for (family, (rows, both_pass)) in &both_pass_by_family {
        lines.emit_result_line(&format!(
            "name=bad_family stream={label} family={family} rows={rows} impl_both_pass={both_pass}"
        ));
    }
    for file in &export.files {
        lines.emit_result_line(&format!(
            "name=bad_file stream={label} file={} bytes={} sha256={}",
            file.name, file.bytes, file.sha256
        ));
    }
    (failed_checks, panics)
}

pub(super) struct BadStatesCommandLine {
    pub(super) table_version: BadTableVersion,
    dump_directory: PathBuf,
    seed_hex: String,
}

/// 第一个词认两个模式：`bad-states`（第一次的表）、`bad-states-second-run`（第 1 次重跑的表）；别的词不认。
pub(super) fn parse_bad_states_command_line(arguments: &[String]) -> Option<BadStatesCommandLine> {
    let mut words = arguments.iter();
    let table_version = match words.next().map(String::as_str) {
        Some("bad-states") => BadTableVersion::FirstRun,
        Some("bad-states-second-run") => BadTableVersion::SecondRun,
        Some(_) | None => return None,
    };
    let mut dump_directory = None;
    let mut seed_hex = None;
    while let Some(word) = words.next() {
        match word.as_str() {
            "--dump-directory" => dump_directory = Some(PathBuf::from(words.next()?)),
            "--seed-hex" => seed_hex = Some(words.next()?.clone()),
            _ => return None,
        }
    }
    let seed_hex = seed_hex?;
    (seed_hex.len() == 16
        && seed_hex
            .chars()
            .all(|character| character.is_ascii_hexdigit()))
    .then_some(BadStatesCommandLine {
        table_version,
        dump_directory: dump_directory?,
        seed_hex,
    })
}

/// `bad-states` 与 `bad-states-second-run` 模式（`BadTableVersion` 区分）：两条流各造各判，流一先（盲测族的 splitmix64 一串接着抽）。
/// 退出码：0 正常；2 命令行不对或五个文件里有已存在的；3 回比没过（产物照打，交主 agent）。`config` 行的 `mode=` 两个模式各写各的名字，
/// 第一次的 `bad-states` 模式除此之外输出一个字节不变。
pub(super) fn run_bad_states(command_line: &BadStatesCommandLine) -> i32 {
    for name in STREAM_NAMES {
        for file_name in BAD_FILES {
            let path = command_line
                .dump_directory
                .join(name.directory_name())
                .join(file_name);
            if path.exists() {
                eprintln!(
                    "{} 已存在：坏状态文件不覆盖，换一份导出目录",
                    path.display()
                );
                return 2;
            }
        }
    }
    let seed =
        u64::from_str_radix(&command_line.seed_hex, 16).expect("16 个十六进制字符，命令行判过");
    let mut generator = SplitMix64 { state: seed };
    let mut lines = ResultLines { emitted: 0 };
    lines.emit_result_line(&format!(
        "name=config segment=s3 mode={} seed_hex={} seed_u64={seed} blind_rows_per_stream={BLIND_ROWS_PER_STREAM} patched_writes_kind=system_configuration_slot judge_threads=1",
        command_line.table_version.configuration_mode_text(),
        command_line.seed_hex
    ));
    let mut failed_checks = 0usize;
    let mut panics = 0usize;
    let mut rows = 0usize;
    for name in STREAM_NAMES {
        let export = bad_states_of_stream(
            name,
            &command_line.dump_directory,
            &command_line.seed_hex,
            seed,
            &mut generator,
            command_line.table_version,
        );
        let (stream_failed, stream_panics) = report_bad_stream(&mut lines, &export);
        failed_checks += stream_failed;
        panics += stream_panics;
        rows += export.rows.len();
    }
    lines.emit_result_line(&format!(
        "name=verdict bad_s1_failed_checks={failed_checks} bad_rows={rows} impl_panics={panics}"
    ));
    lines.finish();
    if failed_checks > 0 {
        3
    } else {
        0
    }
}

#[cfg(test)]
mod tests;
