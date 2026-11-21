//! 一轮一轮地跑、父进程的选项、设备行与驱动核对、起一个子进程与整个父进程运行。

use super::*;

/// 父进程的续跑（登记修订 2 之五）：这一段每一格、轮号 1 到 5 + 3 逐个按轮键找库里的那一条，读得回的拼成 `ProductRounds` 交给 `pending_jobs`
/// （补轮规则不动）；要起的轮由 `run_child` 起，每做完一轮写进库（服务确认落盘才往下走）；都齐了（或 F1 停机）由库里这一段的轮拼出产物。
/// `device_line_of` 交一张卡的驱动与卡那一行（模型卡写 simulated）。
pub(super) fn run_rounds(
    plan: &RoundPlan<'_>,
    library: &RoundLibrary,
    device_line_of: &dyn Fn(Card) -> Result<String, String>,
    run_child: &mut dyn FnMut(&ChildJob) -> Result<String, String>,
) -> Result<RoundsOutcome, String> {
    let rounds_at_most = ROUNDS_REQUIRED_PER_CELL + MAXIMUM_REFILL_ROUNDS_PER_CELL;
    let candidate_jobs: Vec<ChildJob> = plan.cells.iter().flat_map(|cell| (1..=rounds_at_most).map(move |round| ChildJob { cell: *cell, round })).collect();
    let mut round_keys: BTreeMap<ChildJob, String> = BTreeMap::new();
    for job in &candidate_jobs {
        let round_key = round_key_of(&library.child_code_key, &library.arguments_code_key, job, &device_line_of(job.cell.card)?);
        round_keys.insert(*job, round_key);
    }
    for code_key in [&library.child_code_key, &library.arguments_code_key] {
        library.store.write(&library.composition_path(code_key), &library.composition);
    }
    let previous_index = read_the_round_index(library);
    let mut product: ProductRounds = BTreeMap::new();
    let mut texts: BTreeMap<ChildJob, String> = BTreeMap::new();
    let mut reused = 0_u64;
    for (job, round_key) in &round_keys {
        if let Some(text) = read_back_the_round(library.store.read(&library.round_path(round_key)), &round_header(round_key, library)) {
            add_round(&mut product, &mut texts, *job, text)?;
            reused += 1;
        }
    }
    let mismatched = count_mismatched_rounds(&previous_index, &round_keys);
    let changed_units = (mismatched > 0).then(|| changed_units_sentence(library, &previous_index, &round_keys));
    let write_index = |texts: &BTreeMap<ChildJob, String>| -> Result<(), String> {
        let mut index = previous_index.clone();
        for job in texts.keys() {
            index.insert(job.key_text(), round_keys.get(job).cloned().unwrap_or_default());
        }
        library.store.write(&library.index_path(), &index.iter().map(|(key_text, round_key)| format!("{key_text}\t{round_key}\n")).collect::<String>());
        Ok(())
    };
    let meta = format!("meta judged={} how={} host={}", library.judged, round_how(plan), library.host);
    let (mut computed, mut jobs_run, mut stopped_by) = (0_u64, 0_usize, "none");
    'passes: loop {
        let pending = pending_jobs(plan.cells, &product, plan.arm_count);
        if pending.is_empty() {
            break;
        }
        for job in pending {
            if plan.limit_jobs.is_some_and(|limit| jobs_run >= limit) {
                write_index(&texts)?;
                return Ok(RoundsOutcome { ending: RoundsEnding::StoppedAtTheLimit(jobs_run), computed, reused, mismatched, changed_units });
            }
            let child_text = run_child(&job)?;
            jobs_run += 1;
            computed += 1;
            let round_key = round_keys.get(&job).ok_or_else(|| format!("{} 的轮号超出 1 到 {rounds_at_most}", job.key_text()))?;
            library.store.write(&library.round_path(round_key), &round_file_text(&round_header(round_key, library), &job, &meta, &child_text));
            let hold_control_failed = child_output_shows_hold_control_failure(&job, &child_text);
            add_round(&mut product, &mut texts, job, child_text)?;
            if hold_control_failed {
                stopped_by = "f1_hold_control";
                break 'passes;
            }
        }
    }
    write_index(&texts)?;
    let mut text = format!(
        "E7RESULT name=config segment={} seed={SHUFFLE_SEED} driver_version={EXPECTED_DRIVER_VERSION} skip_light={} cells={} \
         rounds_required={ROUNDS_REQUIRED_PER_CELL} refills_at_most={MAXIMUM_REFILL_ROUNDS_PER_CELL} simulated={}\n",
        plan.segment.label(),
        plan.skip_light,
        plan.cells.len(),
        plan.simulated
    );
    let mut ordered: Vec<&ChildJob> = texts.keys().collect();
    ordered.sort_by_key(|job| (job_order_key(job, plan.arm_count), job.round));
    for job in ordered {
        text.push_str(&texts[job]);
    }
    let verdict = run_verdict_line(plan.segment, plan.cells, &product, stopped_by);
    let emitted = text.lines().filter(|line| line.starts_with("E7RESULT ")).count() + 2;
    text.push_str(&format!("{verdict}\nE7RESULT name=done emitted={emitted}\n"));
    let ending = if std::path::Path::new(plan.output_path).exists() {
        let (existing, _existing_rounds) = load_product(plan.output_path)?;
        if existing == text { RoundsEnding::AlreadyComplete } else { RoundsEnding::OutputDiffers }
    } else {
        write_text_atomically(std::path::Path::new(plan.output_path), &text)?;
        RoundsEnding::Assembled { stopped_by }
    };
    Ok(RoundsOutcome { ending, computed, reused, mismatched, changed_units })
}

// ============================== 父进程：按清单起子进程，续跑 ==============================

pub(super) struct ParentOptions {
    pub(super) segment: Segment,
    output_path: String,
    pub(super) skip_light: bool,
    /// 子进程都跑在模型卡上，不起卡、不读 `nvidia-smi`：只用来试父子流水线。
    pub(super) simulated: bool,
    /// 最多起这么多个子进程就停（不写产物）：冒烟用，库里已经做完的轮下一趟照读回。
    pub(super) limit_jobs: Option<usize>,
    /// 取键脚本打的组成清单（登记修订 2 之六第 5 条）：两个入口的代码键从这里读。
    code_key_file: Option<String>,
}

pub(super) fn parse_parent_options(arguments: &[String]) -> Result<ParentOptions, String> {
    let mut segment: Option<Segment> = None;
    let mut output_path: Option<String> = None;
    let mut skip_light = false;
    let mut simulated = false;
    let mut limit_jobs: Option<usize> = None;
    let mut code_key_file: Option<String> = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--limit-jobs" => {
                let text = arguments.get(index + 1).ok_or("--limit-jobs 后面没有值")?;
                limit_jobs = Some(text.parse().map_err(|_| format!("--limit-jobs 不是整数：{text}"))?);
                index += 2;
            }
            "--skip-light" => {
                skip_light = true;
                index += 1;
            }
            "--simulated" => {
                simulated = true;
                index += 1;
            }
            "--segment" => {
                let label = arguments.get(index + 1).ok_or("--segment 后面没有值")?;
                segment = Some(Segment::from_label(label).ok_or_else(|| format!("--segment 认不出：{label}"))?);
                index += 2;
            }
            "--output" => {
                output_path = Some(arguments.get(index + 1).ok_or("--output 后面没有值")?.clone());
                index += 2;
            }
            "--code-key-file" => {
                code_key_file = Some(arguments.get(index + 1).ok_or("--code-key-file 后面没有值")?.clone());
                index += 2;
            }
            other => return Err(format!("认不出的参数 {other}")),
        }
    }
    Ok(ParentOptions {
        segment: segment.ok_or("缺 --segment")?,
        output_path: output_path.ok_or("缺 --output")?,
        skip_light,
        simulated,
        limit_jobs,
        code_key_file,
    })
}

/// 父进程查驱动与卡那一行的 `nvidia-smi` 参数：每张卡的型号名、FB Total 与驱动版本，不带总线号（父进程不知道子进程会拿到哪一张）。
const DEVICE_LINE_QUERY_ARGUMENTS: [&str; 2] = ["--query-gpu=name,memory.total,driver_version", "--format=csv,noheader"];

/// 驱动与卡那一行（登记修订 2 之五「驱动与卡」）：`DEVICE_LINE_QUERY_ARGUMENTS` 那一种输出里 FB Total 等于这一型登记第七节 B1 的数的那一行，
/// 原样去掉首尾空白；同一型几张卡报的行相同算一行，不同（型号名或驱动不一）交 `Err`。
pub(super) fn device_lines_by_card(report: &str) -> Result<BTreeMap<Card, String>, String> {
    let mut lines: BTreeMap<Card, String> = BTreeMap::new();
    for line in report.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let Some(total) = line.split(',').nth(1) else { continue };
        for card in Card::EVERY_CARD {
            if total.trim() == format!("{} MiB", card.total_mebibytes()) {
                if let Some(earlier) = lines.insert(card, line.to_string()).filter(|earlier| earlier != line) {
                    return Err(format!("{} 那一型的几张卡报的驱动与卡那一行不同：「{earlier}」与「{line}」", card.label()));
                }
            }
        }
    }
    Ok(lines)
}

/// 登记第七节 B7、第十一节停机：驱动版本不是 595.84，停下，不作废也不当结果。
fn check_driver_version() -> Result<(), String> {
    let text = run_nvidia_smi(&["--query-gpu=driver_version", "--format=csv,noheader"])?;
    let versions: Vec<&str> = text.lines().map(str::trim).filter(|line| !line.is_empty()).collect();
    if versions.is_empty() || versions.iter().any(|version| *version != EXPECTED_DRIVER_VERSION) {
        return Err(format!("驱动版本不是 {EXPECTED_DRIVER_VERSION}：{versions:?}（登记第十一节停机）"));
    }
    Ok(())
}

fn raw_archive_path(output_path: &str) -> String {
    format!("{output_path}.raw.gz")
}

/// 把子进程的原文压成一个 gzip 成员追加到 `.raw.gz`（gzip 成员可以首尾相连，`zcat` 读得整份）。
fn append_gzip_member(raw_file: &str, archive_path: &str) -> Result<(), String> {
    let archive = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(archive_path)
        .map_err(|error| format!("打不开 {archive_path}：{error}"))?;
    let status = std::process::Command::new("gzip")
        .args(["-9", "-n", "-c", raw_file])
        .stdout(archive)
        .status()
        .map_err(|error| format!("起不了 gzip：{error}"))?;
    if status.success() { Ok(()) } else { Err(format!("gzip 退 {:?}", status.code())) }
}

/// 子进程照哪一份装置起：`scripts/experiments/device-code-key.py run` 起装置时把它编好、起的那一份的路径放进这个环境变量。
/// 不读自己的路径（`std::env::current_exe`）：单元键认不出读自己路径的起法，整个 main 闭包拒算。
const DEVICE_BINARY_VARIABLE: &str = "DEVICE_CODE_KEY_DEVICE_BINARY";

/// `run` 起子进程用的装置路径（`DEVICE_BINARY_VARIABLE`）；没设或是空的交错，`run` 一个子进程都不起。
fn child_executable_path() -> Result<std::path::PathBuf, String> {
    match std::env::var_os(DEVICE_BINARY_VARIABLE) {
        Some(path) if !path.is_empty() => Ok(std::path::PathBuf::from(path)),
        _ => Err(format!("没有 {DEVICE_BINARY_VARIABLE}：run 要经 scripts/experiments/device-code-key.py run 起，子进程照它交的装置路径起")),
    }
}

/// 起一个子进程，等它退出后整份读（不在转发输出的循环里打时间戳），交回要追加进产物的文本。
fn run_one_child(
    job: &ChildJob,
    executable: &std::path::Path,
    raw_archive: &str,
    scratch_directory: &std::path::Path,
    simulated: bool,
) -> Result<String, String> {
    let raw_file = scratch_directory.join(format!("e174-raw-{}.txt", std::process::id()));
    let mut command = std::process::Command::new(executable);
    command.args(job.child_arguments()).arg("--raw-output").arg(&raw_file);
    if simulated {
        command.arg("--simulated");
    }
    let output = command.output().map_err(|error| format!("起不了子进程：{error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let has_done_line = stdout.lines().any(|line| line.starts_with("E7RESULT name=child_done "));
    if output.status.success() && has_done_line {
        if raw_file.exists() {
            append_gzip_member(&raw_file.to_string_lossy(), raw_archive)?;
            let _ = std::fs::remove_file(&raw_file);
        }
        return Ok(stdout);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let tail: String = stderr.chars().rev().take(STDERR_TAIL_CHARACTERS).collect::<Vec<_>>().into_iter().rev().collect();
    let _ = std::fs::remove_file(&raw_file);
    Ok(format!(
        "E7RESULT name=child_failed {} exit={:?} has_done_line={has_done_line} detail={}\n",
        job.key_text(),
        output.status.code(),
        without_whitespace(&tail)
    ))
}

fn load_product(path: &str) -> Result<(String, ProductRounds), String> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("读不了 {path}：{error}")),
    };
    let rounds = parse_product(&text)?;
    Ok((text, rounds))
}

fn run_verdict_line(segment: Segment, cells: &[CellKey], product: &ProductRounds, stopped_by: &str) -> String {
    let required = usize::try_from(ROUNDS_REQUIRED_PER_CELL).expect("轮数装得进 usize");
    let cells_incomplete = cells
        .iter()
        .filter(|cell| {
            product.iter().filter(|(job, data)| job.cell == **cell && data.is_complete(cell.lease_count())).count() < required
        })
        .count();
    let children_complete = product.iter().filter(|(job, data)| data.is_complete(job.cell.lease_count())).count();
    let children_failed = product.values().filter(|data| data.failure_detail.is_some()).count();
    format!(
        "E7RESULT name=verdict scope=run segment={} cells_total={} cells_incomplete={cells_incomplete} children_total={} \
         children_complete={children_complete} children_failed={children_failed} children_void={} stopped_by={stopped_by}",
        segment.label(),
        cells.len(),
        product.len(),
        product.len() - children_complete - children_failed,
    )
}

/// 「扣住不丢」组的一个子进程里出现了 F1：整轮停（登记第十一节够判停机）。
pub(super) fn child_output_shows_hold_control_failure(job: &ChildJob, child_text: &str) -> bool {
    job.cell.is_hold_control() && child_text.contains(VoidReason::HoldControlDidNotHold.label())
}

pub(super) fn run_parent(arguments: &[String]) -> Result<i32, String> {
    let options = parse_parent_options(arguments)?;
    let child_executable = child_executable_path()?;
    let Some(code_key_path) = &options.code_key_file else {
        eprintln!("e174：run 要 --code-key-file（两个入口的代码键）：经 scripts/experiment-apparatus/device-code-key.py run 起");
        return Ok(2);
    };
    let composition = std::fs::read_to_string(code_key_path).map_err(|error| format!("读不了 --code-key-file {code_key_path}：{error}"))?;
    let store = match DeviceLibrary::from_environment() {
        Ok(store) => store,
        Err(problem) => {
            eprintln!("e174：{problem}");
            return Ok(2);
        }
    };
    let library = round_library_from_composition(store, &composition, options.simulated)?;
    if !options.simulated {
        check_driver_version()?;
    }
    let device_lines = if options.simulated {
        None
    } else {
        Some(device_lines_by_card(&run_nvidia_smi(&DEVICE_LINE_QUERY_ARGUMENTS)?)?)
    };
    let device_line_of = |card: Card| -> Result<String, String> {
        match &device_lines {
            None => Ok("simulated".to_string()),
            Some(lines) => lines.get(&card).cloned().ok_or_else(|| {
                format!("nvidia-smi 里没有 {} 那一型（FB Total {} MiB）的卡（驱动与卡那一行进轮键）", card.label(), card.total_mebibytes())
            }),
        }
    };
    let cells = cells_of_segment(options.segment, options.skip_light);
    let plan = RoundPlan {
        segment: options.segment,
        cells: &cells,
        arm_count: match options.segment {
            Segment::First => 2,
            Segment::Second => 3,
        },
        output_path: &options.output_path,
        skip_light: options.skip_light,
        simulated: options.simulated,
        limit_jobs: options.limit_jobs,
    };
    let scratch_directory = std::env::temp_dir();
    let raw_archive = raw_archive_path(&options.output_path);
    let mut run_child = |job: &ChildJob| run_one_child(job, &child_executable, &raw_archive, &scratch_directory, options.simulated);
    let outcome = run_rounds(&plan, &library, &device_line_of, &mut run_child)?;
    eprintln!(
        "run: 现判 {} 轮、复用 {} 轮、键对不上 {} 轮（段 {}，库 prune-server，经 device-code-key.py 转交）",
        outcome.computed,
        outcome.reused,
        outcome.mismatched,
        options.segment.label()
    );
    if let Some(sentence) = &outcome.changed_units {
        eprintln!("run: {sentence}");
    }
    Ok(match outcome.ending {
        RoundsEnding::StoppedAtTheLimit(jobs_run) => {
            println!("跑满 --limit-jobs {jobs_run} 个子进程，停在这里：没写产物，同一条命令去掉 --limit-jobs 就接着往下跑（做完的轮在库里）");
            0
        }
        RoundsEnding::Assembled { stopped_by } => i32::from(stopped_by != "none") * 4,
        RoundsEnding::AlreadyComplete => {
            println!("{} 已经齐了：与库里这一段拼出的产物逐字节相同，不再写", options.output_path);
            0
        }
        RoundsEnding::OutputDiffers => {
            eprintln!("e174：{} 已在、内容与库里这一段拼出的不同，一个字节没动；另给 --output", options.output_path);
            2
        }
    })
}
