use super::*;

// ───────────────────────────── keys：逐份转储排计划、取块键 ─────────────────────────────

const CONTROLS_FILE_SUFFIX: &str = ".controls";
const CONTROLS_FILE_TRAILER_PREFIX: &str = "# controls_done control_lines=";

/// 一份转储这一趟做完的结局。
enum KeysOutcome {
    Done {
        row: Box<FlowRow>,
        reused: bool,
        /// （`name=control` 行的内容，过没过）。
        controls: Vec<(String, bool)>,
        count_disagreements: Vec<String>,
    },
    Failed {
        label: String,
        message: String,
    },
}

fn label_of_the_file(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
        .trim_end_matches(DUMP_FILE_SUFFIX)
        .to_string()
}

pub(super) fn label_of_the_keys_file(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
        .trim_end_matches(FULL_KEYS_FILE_SUFFIX)
        .to_string()
}

fn controls_file_text(controls: &[ControlReading]) -> String {
    let mut text: String = controls
        .iter()
        .map(|control| format!("{}\n", control.body()))
        .collect();
    text.push_str(&format!(
        "{CONTROLS_FILE_TRAILER_PREFIX}{}\n",
        controls.len()
    ));
    text
}

/// 对照文件读回来的 `name=control` 行（每行一条）；缺末行（没写完）交 `None`。
fn control_lines_of(text: &str) -> Option<Vec<String>> {
    let (lines, trailer) = text
        .trim_end()
        .rsplit_once('\n')
        .map_or((None, text.trim_end()), |(head, last)| (Some(head), last));
    let declared = trailer
        .strip_prefix(CONTROLS_FILE_TRAILER_PREFIX)?
        .parse::<usize>()
        .ok()?;
    let kept: Vec<String> =
        lines.map_or_else(Vec::new, |head| head.lines().map(str::to_string).collect());
    (kept.len() == declared).then_some(kept)
}

/// 做一份转储：已经有写完的块键文件（与需要的对照）就整份跳过；否则读转储、排计划、写块键文件、跑对照。
fn keys_of_one_dump(path: &Path, out_directory: &Path, with_controls: bool) -> KeysOutcome {
    let label = label_of_the_file(path);
    let full_path = out_directory.join(format!("{label}{FULL_KEYS_FILE_SUFFIX}"));
    let in_place_subsets_path =
        out_directory.join(format!("{label}{IN_PLACE_SUBSETS_KEYS_FILE_SUFFIX}"));
    let controls_path = out_directory.join(format!("{label}{CONTROLS_FILE_SUFFIX}"));
    let failed = |message: String| KeysOutcome::Failed {
        label: label.clone(),
        message,
    };
    let dump = match FlowDump::read_from(path) {
        Ok(dump) => dump,
        Err(message) => return failed(message),
    };
    let needs_controls = with_controls && controls_apply_to(&dump);
    if let Ok((_, existing_row)) = KeysFile::read_row_of_a_finished_file(&full_path) {
        let in_place_subsets_done = existing_row.blocks_in_place_subsets.is_none()
            || KeysFile::read_row_of_a_finished_file(&in_place_subsets_path).is_ok();
        let control_lines = std::fs::read_to_string(&controls_path)
            .ok()
            .and_then(|text| control_lines_of(&text));
        if in_place_subsets_done && (!needs_controls || control_lines.is_some()) {
            return KeysOutcome::Done {
                row: Box::new(existing_row),
                reused: true,
                controls: control_lines
                    .unwrap_or_default()
                    .into_iter()
                    .map(|line| {
                        let passed = line.contains(" passed=true ");
                        (line, passed)
                    })
                    .collect(),
                count_disagreements: Vec::new(),
            };
        }
    }
    let base = dump.base_pool();
    let outcome = catch_unwind(AssertUnwindSafe(
        || -> Result<(FlowReading, Vec<ControlReading>), String> {
            let reading = read_the_flow(&dump, &base)?;
            let controls = if needs_controls {
                controls_of_the_flow(&dump, &base, &reading)?
            } else {
                Vec::new()
            };
            Ok((reading, controls))
        },
    ));
    let (reading, controls) = match outcome {
        Err(payload) => return failed(format!("panic：{}", panic_text_of(payload.as_ref()))),
        Ok(Err(message)) => return failed(message),
        Ok(Ok(pair)) => pair,
    };
    let no_blocks: &[BlockOfAPlan] = &[];
    let full_blocks = reading.full.as_ref().map_or(no_blocks, |plan| &plan.blocks);
    if let Err(message) =
        write_keys_file_atomically(&full_path, &reading.row, Expansion::Full, full_blocks)
    {
        return failed(message);
    }
    if let Some(in_place_subsets) = &reading.in_place_subsets {
        if let Err(message) = write_keys_file_atomically(
            &in_place_subsets_path,
            &reading.row,
            Expansion::InPlaceSubsets,
            &in_place_subsets.blocks,
        ) {
            return failed(message);
        }
    }
    if needs_controls {
        if let Err(message) = write_dump::write_bytes_atomically(
            &controls_path,
            controls_file_text(&controls).as_bytes(),
        ) {
            return failed(message);
        }
    }
    KeysOutcome::Done {
        row: Box::new(reading.row),
        reused: false,
        controls: controls
            .iter()
            .map(|control| (control.body(), control.passed))
            .collect(),
        count_disagreements: reading.count_disagreements,
    }
}

pub(super) fn panic_text_of(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|text| (*text).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "panic 的内容不是文本".to_string())
        .replace('\n', "\\n")
}

/// 每族每组一行 `name=family_summary`：各量的最小 / 中位 / 最大，与写数不少于 1000 的流数（登记第六节 J1-甲）。
fn family_summary_bodies(rows: &[FlowRow]) -> Vec<String> {
    let mut groups: BTreeMap<(String, String), Vec<&FlowRow>> = BTreeMap::new();
    for row in rows {
        groups
            .entry((row.family.clone(), row.group()))
            .or_default()
            .push(row);
    }
    groups
        .into_iter()
        .map(|((family, group), members)| {
            let spread_of = |value_of: &dyn Fn(&FlowRow) -> Option<f64>| {
                Spread::text(Spread::of(&members.iter().filter_map(|row| value_of(row)).collect::<Vec<f64>>()).as_ref())
            };
            let log2_of = |count: Option<u64>| count.map(|value| (value as f64).log2());
            format!(
                "name=family_summary family={family} group={group} flows={} flows_with_writes_at_least_1000={} flows_beyond_u64={} writes=[{}] segments=[{}] longest_segment=[{}] log2_two_state=[{}] log2_full=[{}] log2_in_place_subsets=[{}] blocks_full=[{}]",
                members.len(),
                members.iter().filter(|row| row.writes >= WRITES_OF_A_THOUSAND_WRITE_STREAM).count(),
                members.iter().filter(|row| row.full.is_none()).count(),
                spread_of(&|row| Some(row.writes as f64)),
                spread_of(&|row| Some(row.segments as f64)),
                spread_of(&|row| Some(row.longest_segment as f64)),
                spread_of(&|row| log2_of(row.two_state)),
                spread_of(&|row| log2_of(row.full)),
                spread_of(&|row| log2_of(row.in_place_subsets)),
                spread_of(&|row| row.blocks_full.map(|blocks| blocks as f64)),
            )
        })
        .collect()
}

/// `keys`：按流并行（线程数取转储模块里那一份读法），每份转储一个片，片与片之间不共享状态。
pub(super) fn run_keys_command(arguments: &[String]) -> i32 {
    let parsed = match ParsedOptions::parse(arguments, &["--dump", "--out"], &["--controls"]) {
        Ok(parsed) => parsed,
        Err(message) => return usage_error(&message),
    };
    let (dump_directories, out_directory) = match (parsed.paths("--dump"), parsed.single("--out")) {
        (Ok(dumps), Ok(out)) => (dumps, PathBuf::from(out)),
        (Err(message), _) | (_, Err(message)) => return usage_error(&message),
    };
    if let Err(error) = std::fs::create_dir_all(&out_directory) {
        return usage_error(&format!("建 {} 失败：{error}", out_directory.display()));
    }
    let dump_files = match files_with_suffix(&dump_directories, DUMP_FILE_SUFFIX, None) {
        Ok(files) => files,
        Err(message) => return usage_error(&message),
    };
    let with_controls = parsed.flag("--controls");
    let output = Mutex::new(Output::new());
    let outcomes: Mutex<BTreeMap<usize, KeysOutcome>> = Mutex::new(BTreeMap::new());
    let next_file = AtomicUsize::new(0);
    let started = Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..write_dump::worker_threads_from_the_environment().min(dump_files.len().max(1)) {
            scope.spawn(|| loop {
                let file_index = next_file.fetch_add(1, Ordering::SeqCst);
                let Some(path) = dump_files.get(file_index) else {
                    break;
                };
                let outcome = keys_of_one_dump(path, &out_directory, with_controls);
                let mut finished = outcomes.lock().expect("结局表的锁没被毒化");
                finished.insert(file_index, outcome);
                let done = finished.len();
                drop(finished);
                output.lock().expect("打印器的锁没被毒化").emit(&format!(
                    "name=progress flow={} done={done}/{} seconds={:.1}",
                    label_of_the_file(path),
                    dump_files.len(),
                    started.elapsed().as_secs_f64()
                ));
            });
        }
    });
    let mut output = output.into_inner().expect("打印器的锁没被毒化");
    let (mut rows, mut failed_flows, mut failed_controls, mut disagreements_total) =
        (Vec::new(), 0usize, 0usize, 0usize);
    for outcome in outcomes
        .into_inner()
        .expect("结局表的锁没被毒化")
        .into_values()
    {
        match outcome {
            KeysOutcome::Failed { label, message } => {
                failed_flows += 1;
                output.emit(&format!("name=flow_failed flow={label} reason={message}"));
            }
            KeysOutcome::Done {
                row,
                reused,
                controls,
                count_disagreements,
            } => {
                output.emit(&format!("{} reused={reused}", row.body()));
                for (body, passed) in controls {
                    failed_controls += usize::from(!passed);
                    output.emit(&body);
                }
                for disagreement in &count_disagreements {
                    output.emit(&format!(
                        "name=stop_check item=two_counting_routes_agree result=fail detail={}",
                        disagreement.replace(' ', "_")
                    ));
                }
                disagreements_total += count_disagreements.len();
                rows.push(*row);
            }
        }
    }
    for body in family_summary_bodies(&rows) {
        output.emit(&body);
    }
    output.emit_closing_line(
        "keys_done",
        &format!("flows={} failed={failed_flows} controls_failed={failed_controls} count_disagreements={disagreements_total}", rows.len()),
    );
    if disagreements_total > 0 || failed_flows > 0 {
        EXIT_CODE_STOP_CLAUSE_FAILED
    } else if failed_controls > 0 {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else {
        EXIT_CODE_OK
    }
}

pub(super) fn usage_error(message: &str) -> i32 {
    eprintln!("命令行不对：{message}");
    EXIT_CODE_USAGE
}

#[cfg(test)]
mod tests;
