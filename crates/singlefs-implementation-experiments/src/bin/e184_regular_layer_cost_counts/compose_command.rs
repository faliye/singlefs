use super::*;

// ───────────────────────────── compose：J2-乙，提交那一趟的挂钟落在哪一档（纯算术） ─────────────────────────────

/// 挂钟档：≤ 2 小时 / 2–24 小时 / > 24 小时（界的出处在常量上）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WallClockBand {
    WithinTwoHours,
    BetweenTwoHoursAndADay,
    OverADay,
}

impl WallClockBand {
    pub(super) fn name(self) -> &'static str {
        match self {
            WallClockBand::WithinTwoHours => "within_2h",
            WallClockBand::BetweenTwoHoursAndADay => "between_2h_and_24h",
            WallClockBand::OverADay => "over_24h",
        }
    }
}

fn band_of_seconds(seconds: f64) -> WallClockBand {
    if seconds <= SECONDS_OF_TWO_HOURS {
        WallClockBand::WithinTwoHours
    } else if seconds <= SECONDS_OF_A_DAY {
        WallClockBand::BetweenTwoHoursAndADay
    } else {
        WallClockBand::OverADay
    }
}

/// 一行的字段（行首可以有 `E7RESULT `）。
fn fields_of_the_line(line: &str) -> BTreeMap<String, String> {
    fields_of(line.strip_prefix("E7RESULT ").unwrap_or(line))
}

/// 文本里 `name=<name>` 的每一行的字段。
fn lines_named(text: &str, name: &str) -> Vec<BTreeMap<String, String>> {
    text.lines()
        .map(fields_of_the_line)
        .filter(|fields| fields.get("name").is_some_and(|found| found == name))
        .collect()
}

fn float_field(fields: &BTreeMap<String, String>, name: &str) -> Result<f64, String> {
    required_field(fields, name)?
        .parse::<f64>()
        .map_err(|error| format!("{name}：{error}"))
}

fn u64_field(fields: &BTreeMap<String, String>, name: &str) -> Result<u64, String> {
    required_field(fields, name)?
        .parse::<u64>()
        .map_err(|error| format!("{name}：{error}"))
}

/// 崩溃放量一项冷跑的日志读出来的数（5.4：`CRASH_AMPLIFICATION_ITEM` 行与 `CRASH_AMPLIFICATION_GPU_TOTAL` 行）。
#[derive(Clone, Debug, PartialEq)]
struct ColdRun {
    pub(super) item: String,
    pub(super) exit_code: u64,
    /// 现判的状态数（`states=`）。
    pub(super) states: u64,
    pub(super) reused: u64,
    /// 这一项从起到收的秒数（入口自己计的）。
    seconds: f64,
    /// 判段：几台并行时各取最大（它们同时在判）。
    compile_seconds: f64,
    judge_seconds: f64,
    seconds_storing: f64,
    gpu_total_lines: usize,
}

impl ColdRun {
    /// 读一项的日志。ITEM 行恰好一行；GPU_TOTAL 行至少一行。
    ///
    /// # Errors
    /// ITEM 行不是恰好一行，或 GPU_TOTAL 一行都没有，或字段读不出：读不到不等于读到 0。
    fn from_log(text: &str) -> Result<Self, String> {
        let item_lines: Vec<BTreeMap<String, String>> = text
            .lines()
            .filter(|line| line.starts_with("CRASH_AMPLIFICATION_ITEM "))
            .map(fields_of)
            .collect();
        let [item_line] = item_lines.as_slice() else {
            return Err(format!(
                "CRASH_AMPLIFICATION_ITEM 行要恰好一行，读到 {} 行",
                item_lines.len()
            ));
        };
        let gpu_lines: Vec<BTreeMap<String, String>> = text
            .lines()
            .filter(|line| line.starts_with("CRASH_AMPLIFICATION_GPU_TOTAL "))
            .map(fields_of)
            .collect();
        if gpu_lines.is_empty() {
            return Err("一行 CRASH_AMPLIFICATION_GPU_TOTAL 都没有：判段用时读不到".to_string());
        }
        let largest = |name: &str| -> Result<f64, String> {
            gpu_lines
                .iter()
                .map(|fields| float_field(fields, name))
                .try_fold(0.0f64, |largest, value| {
                    value.map(|seconds| largest.max(seconds))
                })
        };
        Ok(Self {
            item: required_field(item_line, "item")?.to_string(),
            exit_code: u64_field(item_line, "exit")?,
            states: u64_field(item_line, "states")?,
            reused: u64_field(item_line, "reused")?,
            seconds: float_field(item_line, "seconds")?,
            compile_seconds: largest("compile_seconds")?,
            judge_seconds: largest("judge_seconds")?,
            seconds_storing: largest("seconds_storing")?,
            gpu_total_lines: gpu_lines.len(),
        })
    }

    /// 录入段用时（推的）：项的 `seconds` 减去编译、判段、存结论三段。
    fn recording_seconds(&self) -> f64 {
        self.seconds - self.compile_seconds - self.judge_seconds - self.seconds_storing
    }

    /// 包含自检：外层 `seconds` 包得住里层三段，包不住的这一项不进统计。
    fn outer_seconds_cover_the_inner_ones(&self) -> bool {
        self.recording_seconds() >= 0.0
    }

    /// 总吞吐：现判态数 ÷ 这一项的挂钟（录入加判）。
    fn rate_total(&self) -> f64 {
        self.states as f64 / self.seconds
    }

    /// 判段吞吐：现判态数 ÷ 判段用时（被测进程里计的 `judge_seconds`）。
    fn rate_judge(&self) -> f64 {
        self.states as f64 / self.judge_seconds
    }
}

/// 两点拟合：秒 = 固定 + 态数 ÷ 斜率。两点的秒数不同才拟合得出。
fn fit_through_two_points(
    small_states: u64,
    small_seconds: f64,
    medium_states: u64,
    medium_seconds: f64,
) -> Option<(f64, f64)> {
    let seconds_apart = medium_seconds - small_seconds;
    if seconds_apart <= 0.0 || medium_states <= small_states {
        return None;
    }
    let states_per_second = (medium_states - small_states) as f64 / seconds_apart;
    Some((
        small_seconds - small_states as f64 / states_per_second,
        states_per_second,
    ))
}

/// `du -s -b` 的输出里路径以 `suffix` 结尾的那一行的字节数。
fn library_bytes_of(du_text: &str, suffix: &str) -> Result<u64, String> {
    du_text
        .lines()
        .filter_map(|line| line.split_once(char::is_whitespace))
        .find(|(_bytes, path)| path.trim().ends_with(suffix))
        .ok_or_else(|| format!("du 的输出里没有以 {suffix} 结尾的路径"))
        .and_then(|(bytes, _path)| {
            bytes
                .parse::<u64>()
                .map_err(|error| format!("{suffix} 的字节数：{error}"))
        })
}

/// `crash_library_reader keys` 的输出里库目录以 `suffix` 结尾的那一行的 `recorded_blocks=`。
fn recorded_blocks_of(reader_text: &str, suffix: &str) -> Result<u64, String> {
    reader_text
        .lines()
        .filter(|line| line.starts_with("CRASH_LIBRARY_KEYS\t"))
        .map(fields_of)
        .find(|fields| {
            fields
                .get("library")
                .is_some_and(|library| library.ends_with(suffix))
        })
        .ok_or_else(|| {
            format!("读库程序的输出里没有库目录以 {suffix} 结尾的 CRASH_LIBRARY_KEYS 行")
        })
        .and_then(|fields| u64_field(&fields, "recorded_blocks"))
}

/// J3-乙的判定（登记第六节）：还差 = still_missing，写路径提交大多作废过半 = more_than_half_invalidated，块级复用省下大半 = mostly_reused，
/// 其间 = partial。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CommitPairsVerdict {
    StillMissing,
    MoreThanHalfInvalidated,
    MostlyReused,
    Partial,
}

impl CommitPairsVerdict {
    pub(super) fn name(self) -> &'static str {
        match self {
            CommitPairsVerdict::StillMissing => "still_missing",
            CommitPairsVerdict::MoreThanHalfInvalidated => "more_than_half_invalidated",
            CommitPairsVerdict::MostlyReused => "mostly_reused",
            CommitPairsVerdict::Partial => "partial",
        }
    }
}

fn commit_pairs_verdict(fractions: &[f64]) -> CommitPairsVerdict {
    let Some(spread) =
        Spread::of(fractions).filter(|_| fractions.len() >= COMMIT_PAIRS_NEEDED_FOR_ENOUGH)
    else {
        return CommitPairsVerdict::StillMissing;
    };
    if spread.median >= INVALIDATED_MEDIAN_AT_LEAST_FOR_MORE_THAN_HALF {
        CommitPairsVerdict::MoreThanHalfInvalidated
    } else if spread.median <= INVALIDATED_MEDIAN_AT_MOST_FOR_MOST_REUSED {
        CommitPairsVerdict::MostlyReused
    } else {
        CommitPairsVerdict::Partial
    }
}

/// 提交对的作废占比轨迹（8.1：被条款当谓词输入的量报轨迹，不只报中位）：算得出的对数、为正的对数、峰值、中位、期末值。
#[derive(Clone, Debug, PartialEq)]
struct CommitPairTrajectory {
    computable: usize,
    positive: usize,
    peak: f64,
    pub(super) median: f64,
    final_value: Option<f64>,
}

/// 作废占比大于 0 的提交对数（占比为 0 的那一对一块都没作废，不算为正）。
fn pairs_with_a_positive_fraction(fractions: &[f64]) -> usize {
    fractions.iter().filter(|fraction| **fraction > 0.0).count()
}

/// `fractions_in_order` 按提交的时间次序；`final_index` 是期末那一对在其中的下标（主工作区对最近一次提交那一对）。
fn commit_pair_trajectory(
    fractions_in_order: &[f64],
    final_index: Option<usize>,
) -> Option<CommitPairTrajectory> {
    let spread = Spread::of(fractions_in_order)?;
    Some(CommitPairTrajectory {
        computable: fractions_in_order.len(),
        positive: pairs_with_a_positive_fraction(fractions_in_order),
        peak: spread.maximum,
        median: spread.median,
        final_value: final_index.and_then(|index| fractions_in_order.get(index).copied()),
    })
}

/// 一对提交在 `compare` 的汇总行里读出来的作废占比（同一对的几组合起来按块数、态数合算）。
#[derive(Clone, Debug, PartialEq)]
struct CommitPairReading {
    pub(super) pair: String,
    pub(super) block_fraction: f64,
    pub(super) state_fraction: f64,
    judging_unit_key_changed: String,
}

/// `compare` 输出里 `kind=commit-pair` 的汇总行，按 `pair` 合算，按首次出现的次序。
///
/// # Errors
/// 汇总行的字段读不出。
fn commit_pairs_of(compare_text: &str) -> Result<Vec<CommitPairReading>, String> {
    let mut order: Vec<String> = Vec::new();
    let mut pooled: BTreeMap<String, (u64, u64, u64, u64, String)> = BTreeMap::new();
    for fields in lines_named(compare_text, "compare_summary") {
        if required_field(&fields, "kind")? != "commit-pair" {
            continue;
        }
        let pair = required_field(&fields, "pair")?.to_string();
        let entry = pooled.entry(pair.clone()).or_insert_with(|| {
            order.push(pair.clone());
            (0, 0, 0, 0, String::new())
        });
        entry.0 += u64_field(&fields, "blocks_after")?;
        entry.1 += u64_field(&fields, "invalidated_blocks")?;
        entry.2 += u64_field(&fields, "states_after")?;
        entry.3 += u64_field(&fields, "invalidated_states")?;
        entry.4 = required_field(&fields, "judging_unit_key_changed")?.to_string();
    }
    Ok(order
        .into_iter()
        .map(|pair| {
            let (blocks, invalidated_blocks, states, invalidated_states, key_changed) =
                &pooled[&pair];
            CommitPairReading {
                pair,
                block_fraction: *invalidated_blocks as f64 / *blocks as f64,
                state_fraction: *invalidated_states as f64 / *states as f64,
                judging_unit_key_changed: key_changed.clone(),
            }
        })
        .collect())
}

/// 提交那一趟要现判的秒数：查库的 `keys_seconds` 之和，加上作废的态数 ÷ 判段吞吐。
fn commit_cell_seconds(keys_seconds: f64, states_to_judge: f64, states_per_second: f64) -> f64 {
    keys_seconds + states_to_judge / states_per_second
}

/// 预算 `budget_seconds` 下按点的态数升序开：开得下几个点、几个状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OpenUnderBudget {
    pub(super) points: usize,
    pub(super) states: u64,
}

fn open_under_budget(
    point_states: &[u64],
    budget_seconds: f64,
    states_per_second: f64,
) -> OpenUnderBudget {
    let mut sorted = point_states.to_vec();
    sorted.sort_unstable();
    let mut open = OpenUnderBudget {
        points: 0,
        states: 0,
    };
    for states in sorted {
        let after = open.states.saturating_add(states);
        if after as f64 / states_per_second > budget_seconds {
            break;
        }
        open = OpenUnderBudget {
            points: open.points + 1,
            states: after,
        };
    }
    open
}

/// 按点的态数升序，每 10% 点数一个刻度的累计秒数（8.1 的累计曲线）。
fn cumulative_curve(point_states: &[u64], states_per_second: f64) -> Vec<(f64, f64)> {
    let mut sorted = point_states.to_vec();
    sorted.sort_unstable();
    (1..=10u64)
        .map(|tenth| {
            let points = usize::try_from((sorted.len() as u64 * tenth).div_ceil(10))
                .expect("点数装得进 usize");
            let states: u64 = sorted.iter().take(points).sum();
            (tenth as f64 / 10.0, states as f64 / states_per_second)
        })
        .collect()
}

/// 常规层名义负载 N（登记 5.4 前的 `compose` 定义）：随机历史 BROAD、FourGibibytes、100 步档的 20 段，加覆盖写链三档。
fn is_in_the_nominal_regular_layer_load(row: &FlowRow) -> bool {
    label_is_in_the_nominal_regular_layer_load(&row.label)
}

fn label_is_in_the_nominal_regular_layer_load(label: &str) -> bool {
    label.starts_with("random_history-broad-FourGibibytes-steps0100-")
        || label.starts_with("overwrite_chain-")
}

/// 合算要的 N 一侧的量。
#[derive(Clone, Debug, PartialEq)]
struct ComposeInputs {
    keys_seconds_total: f64,
    full_total: u64,
    in_place_subsets_total: u64,
    /// N 里每一条流的每个崩溃点的全量态数（全部流的点放在一起）。
    point_states: Vec<u64>,
    write_path_median_fraction: Option<f64>,
    write_path_upper_bound_fraction: Option<f64>,
}

/// 一次冷跑量出来的两种吞吐（态每秒）。
#[derive(Clone, Copy, Debug, PartialEq)]
struct Rates {
    pub(super) total: f64,
    judge: f64,
}

/// 每个 G2 候选 × 提交类别的格子。丙只查库，另列一格「主 agent 起的全量」；丁在预算下开得下多少，另行列出。
fn cell_bodies(inputs: &ComposeInputs, rate_source: &str, rates: &Rates) -> Vec<String> {
    let classes = [
        ("write_path_median", inputs.write_path_median_fraction),
        (
            "write_path_upper_bound",
            inputs.write_path_upper_bound_fraction,
        ),
        ("judge_only", Some(1.0)),
    ];
    let full_states = inputs.full_total as f64;
    let in_place_subsets_states = inputs.in_place_subsets_total as f64;
    let mut bodies = Vec::new();
    for (class, fraction) in classes {
        let Some(fraction) = fraction else {
            bodies.push(format!("name=compose_cell commit_class={class} rate_source={rate_source} unavailable=no_invalidated_fraction"));
            continue;
        };
        let keys_seconds = inputs.keys_seconds_total;
        let seconds_when_every_commit_judges_the_full_domain =
            commit_cell_seconds(keys_seconds, fraction * full_states, rates.judge);
        let seconds_when_commits_judge_the_block_reuse_domain_only = commit_cell_seconds(
            keys_seconds,
            fraction * in_place_subsets_states,
            rates.judge,
        );
        let seconds_when_commits_query_the_library_only = keys_seconds;
        for (candidate, seconds) in [
            (
                "every_commit_judges_the_full_domain_by_block_keys",
                seconds_when_every_commit_judges_the_full_domain,
            ),
            (
                "commit_judges_the_block_reuse_domain_only",
                seconds_when_commits_judge_the_block_reuse_domain_only,
            ),
            (
                "commit_queries_the_library_only",
                seconds_when_commits_query_the_library_only,
            ),
        ] {
            bodies.push(format!(
                "name=compose_cell candidate={candidate} commit_class={class} rate_source={rate_source} invalidated_state_fraction={} seconds={seconds:.1} hours={:.3} band={}",
                fraction_text(fraction),
                seconds / 3600.0,
                band_of_seconds(seconds).name()
            ));
        }
        if candidate_is_over_a_day(seconds_when_every_commit_judges_the_full_domain) {
            bodies.push(format!("name=compose_trigger item=every_commit_judging_the_full_domain_takes_over_a_day commit_class={class} rate_source={rate_source} seconds={seconds_when_every_commit_judges_the_full_domain:.1}"));
        }
    }
    let full_by_the_main_agent = full_states / rates.total;
    bodies.push(format!(
        "name=compose_cell candidate=commit_queries_the_library_only_full_by_the_main_agent commit_class=any rate_source={rate_source} seconds={full_by_the_main_agent:.1} hours={:.3} band={}",
        full_by_the_main_agent / 3600.0,
        band_of_seconds(full_by_the_main_agent).name()
    ));
    for (hours, budget_seconds) in [(2u32, SECONDS_OF_TWO_HOURS), (24, SECONDS_OF_A_DAY)] {
        let open = open_under_budget(&inputs.point_states, budget_seconds, rates.judge);
        let total_states: u64 = inputs.point_states.iter().sum();
        bodies.push(format!(
            "name=compose_budget candidate=static_ignore_under_a_wall_clock_budget budget_hours={hours} rate_source={rate_source} points_total={} open_points={} open_point_fraction={} states_total={total_states} open_states={} open_state_fraction={}",
            inputs.point_states.len(),
            open.points,
            fraction_text(open.points as f64 / inputs.point_states.len() as f64),
            open.states,
            fraction_text(open.states as f64 / total_states as f64),
        ));
    }
    for (point_fraction, seconds) in cumulative_curve(&inputs.point_states, rates.judge) {
        bodies.push(format!(
            "name=compose_curve candidate=static_ignore_under_a_wall_clock_budget rate_source={rate_source} point_fraction={point_fraction:.1} cumulative_hours={:.3}",
            seconds / 3600.0
        ));
    }
    bodies
}

/// F4 的触发观测：甲的格子越过一天。
fn candidate_is_over_a_day(seconds: f64) -> bool {
    band_of_seconds(seconds) == WallClockBand::OverADay
}

/// J2 小项（建快照流）的全量态数：值抄自 `crash_enumeration_snapshot_creation_stream.rs` 文件头「崩溃点重放的枚举域 16842807 个状态」
/// （锚 A7，停机 T5 拿它与装置重录的建快照流的全量态数回比）。
/// 取号写两轮之后那份文件头已是 16842815；这个数同时是跑前登记作废条款 X3 写死的数，第一段的 J2 冷跑按它判过，改不改由登记定。
const SNAPSHOT_CREATION_FULL_STATES_PINNED_BY_THE_TEST: u64 = 16_842_807;

/// J2 中项（覆盖写与 unlink 在快照之下的流）的全量态数：值抄自 `crash_enumeration_overwrite_and_unlink_under_a_snapshot_stream.rs`
/// 文件头与全量用例的 `ignore` 说明（「崩溃点重放的枚举域 167837792 个状态」，锚 A8）；对不上不作废，只在 `name=anchor` 行报不符。
/// 取号写两轮之后测试文件改成 167837792（取号写一轮时 167837784），这里照新值重抄。
const OVERWRITE_AND_UNLINK_FULL_STATES_PINNED_BY_THE_TEST: u64 = 167_837_792;

/// `keys` 的输出文本里的每一行 `name=flow`。
///
/// # Errors
/// 某一行的字段读不出。
fn flow_rows_of(text: &str) -> Result<Vec<FlowRow>, String> {
    text.lines()
        .map(|line| line.strip_prefix("E7RESULT ").unwrap_or(line))
        .filter(|line| line.starts_with("name=flow "))
        .map(FlowRow::from_body)
        .collect()
}

/// N 一侧的量：N 里每条流的 `keys_seconds`、全量与甲二态数、每个点的全量态数。
///
/// # Errors
/// N 里有流的全量态数装不进 u64（合算不出来）；N 里一条流都没有；N 里某条流没有块键文件。
fn compose_inputs_of(
    rows: &[FlowRow],
    keys_files: &[KeysFile],
    write_path_median_fraction: Option<f64>,
    write_path_upper_bound_fraction: Option<f64>,
) -> Result<ComposeInputs, String> {
    let nominal: Vec<&FlowRow> = rows
        .iter()
        .filter(|row| is_in_the_nominal_regular_layer_load(row))
        .collect();
    if nominal.is_empty() {
        return Err("常规层名义负载 N 里一条流都没有".to_string());
    }
    let mut inputs = ComposeInputs {
        keys_seconds_total: 0.0,
        full_total: 0,
        in_place_subsets_total: 0,
        point_states: Vec::new(),
        write_path_median_fraction,
        write_path_upper_bound_fraction,
    };
    for row in nominal {
        inputs.keys_seconds_total += row.keys_seconds;
        inputs.full_total += row
            .full
            .ok_or_else(|| format!("{} 的全量态数装不进 u64，合算不出", row.label))?;
        inputs.in_place_subsets_total += row
            .in_place_subsets
            .ok_or_else(|| format!("{} 的甲二态数装不进 u64，合算不出", row.label))?;
        let keys = keys_files
            .iter()
            .find(|file| file.row.label == row.label)
            .ok_or_else(|| format!("N 里的 {} 没有块键文件", row.label))?;
        let mut states_by_point: BTreeMap<usize, u64> = BTreeMap::new();
        for block in &keys.blocks {
            *states_by_point.entry(block.point_index).or_default() += block.states;
        }
        inputs.point_states.extend(states_by_point.into_values());
    }
    Ok(inputs)
}

/// `compare` 输出里 `kind=patch`、补丁位是 `pair` 的汇总行，只取属于 N 的组，按块数与态数合算出作废态数占比。
fn patch_state_fraction_in_the_nominal_load(
    compare_text: &str,
    pair: &str,
) -> Result<Option<f64>, String> {
    let (mut states, mut invalidated) = (0u64, 0u64);
    for fields in lines_named(compare_text, "compare_summary") {
        let group = required_field(&fields, "group")?;
        let in_the_load = group == "random_history-broad-FourGibibytes-steps0100"
            || group.starts_with("overwrite_chain-");
        if required_field(&fields, "kind")? == "patch"
            && required_field(&fields, "pair")? == pair
            && in_the_load
        {
            states += u64_field(&fields, "states_after")?;
            invalidated += u64_field(&fields, "invalidated_states")?;
        }
    }
    Ok((states > 0).then(|| invalidated as f64 / states as f64))
}

/// `compose` 的文本输入。
struct ComposeTexts {
    keys_output: String,
    compare_output: String,
    small_log: String,
    medium_log: String,
    library_bytes: String,
    pub(super) recorded_blocks: String,
    final_pair: Option<String>,
}

/// 登记 X3：库不冷（`reused` 不为 0）、包含自检没过、中项的 `seconds` 不大于小项、小项 `states` 不是 16842807，任何一样都让那一项（自检、单调是整个 J2）作废。
fn voided_cold_runs(small: &ColdRun, medium: &ColdRun) -> Vec<String> {
    let mut reasons = Vec::new();
    for run in [small, medium] {
        if run.reused != 0 {
            reasons.push(format!("{}：reused={}，库不冷", run.item, run.reused));
        }
        if !run.outer_seconds_cover_the_inner_ones() {
            reasons.push(format!(
                "{}：seconds 包不住 compile_seconds、judge_seconds、seconds_storing 之和",
                run.item
            ));
        }
        if run.exit_code != 0 {
            reasons.push(format!("{}：驱动退出码 {}", run.item, run.exit_code));
        }
    }
    if medium.seconds <= small.seconds {
        reasons.push("中项的 seconds 不大于小项：两点分不出规模".to_string());
    }
    if small.states != SNAPSHOT_CREATION_FULL_STATES_PINNED_BY_THE_TEST {
        reasons.push(format!(
            "小项 states={}，不是 {SNAPSHOT_CREATION_FULL_STATES_PINNED_BY_THE_TEST}",
            small.states
        ));
    }
    reasons
}

/// `compose` 的全部输出行。交回退出码：0；停机条款没过 3（T5）；作废 4（X3）。
fn compose_lines(
    texts: &ComposeTexts,
    keys_files: &[KeysFile],
    output: &mut Output,
) -> Result<i32, String> {
    let rows = flow_rows_of(&texts.keys_output)?;
    let small =
        ColdRun::from_log(&texts.small_log).map_err(|error| format!("小项日志：{error}"))?;
    let medium =
        ColdRun::from_log(&texts.medium_log).map_err(|error| format!("中项日志：{error}"))?;
    let snapshot_recorded = rows
        .iter()
        .find(|row| row.label == "smoke-snapshot_creation_stream")
        .ok_or("J1 的输出里没有 smoke-snapshot_creation_stream 那一行")?;
    if snapshot_recorded.full != Some(small.states) {
        output.emit(&format!(
            "name=stop_check item=small_item_states_match_the_recorded_snapshot_full result=fail small_item_states={} recorded_full={:?}",
            small.states, snapshot_recorded.full
        ));
        output.emit_closing_line(
            "compose_done",
            "stopped=small_item_states_match_the_recorded_snapshot_full",
        );
        return Ok(EXIT_CODE_STOP_CLAUSE_FAILED);
    }
    output.emit(&format!(
        "name=stop_check item=small_item_states_match_the_recorded_snapshot_full result=pass small_item_states={}",
        small.states
    ));
    let voided = voided_cold_runs(&small, &medium);
    for reason in &voided {
        output.emit(&format!(
            "name=void item=cold_library_runs reason={}",
            reason.replace(' ', "_")
        ));
    }
    if !voided.is_empty() {
        output.emit_closing_line("compose_done", "voided=cold_library_runs");
        return Ok(EXIT_CODE_POSITIVE_CONTROL_FAILED);
    }
    for (identifier, expected, actual) in [
        (
            "small_item_states",
            snapshot_recorded
                .full
                .map_or_else(|| BEYOND_U64.to_string(), |full| full.to_string()),
            small.states.to_string(),
        ),
        (
            "medium_item_states",
            OVERWRITE_AND_UNLINK_FULL_STATES_PINNED_BY_THE_TEST.to_string(),
            medium.states.to_string(),
        ),
    ] {
        output.emit(&anchor_body(identifier, &expected, &actual).0);
    }
    let library_bytes = [
        library_bytes_of(&texts.library_bytes, "library-small")?,
        library_bytes_of(&texts.library_bytes, "library-medium")?,
    ];
    let recorded_blocks = [
        recorded_blocks_of(&texts.recorded_blocks, "library-small")?,
        recorded_blocks_of(&texts.recorded_blocks, "library-medium")?,
    ];
    for (source, run, bytes, blocks) in [
        ("small", &small, library_bytes[0], recorded_blocks[0]),
        ("medium", &medium, library_bytes[1], recorded_blocks[1]),
    ] {
        output.emit(&format!(
            "name=compose_rate rate_source={source} item={} states={} seconds={:.1} compile_seconds={:.1} judge_seconds={:.1} seconds_storing={:.1} recording_seconds={:.1} gpu_total_lines={} states_per_second_of_recording_and_judging={:.1} states_per_second_of_judging={:.1} library_bytes={bytes} library_bytes_per_state={:.4} recorded_blocks={blocks} library_bytes_per_recorded_block={:.1}",
            run.item,
            run.states,
            run.seconds,
            run.compile_seconds,
            run.judge_seconds,
            run.seconds_storing,
            run.recording_seconds(),
            run.gpu_total_lines,
            run.rate_total(),
            run.rate_judge(),
            bytes as f64 / run.states as f64,
            bytes as f64 / blocks as f64,
        ));
    }
    for (metric, small_seconds, medium_seconds) in [
        ("seconds", small.seconds, medium.seconds),
        ("judge_seconds", small.judge_seconds, medium.judge_seconds),
    ] {
        match fit_through_two_points(small.states, small_seconds, medium.states, medium_seconds) {
            Some((fixed, slope)) => output.emit(&format!("name=compose_fit metric={metric} fixed_seconds={fixed:.1} states_per_second={slope:.1}")),
            None => output.emit(&format!("name=compose_fit metric={metric} unavailable=the_two_points_do_not_rise")),
        }
    }
    let pairs = commit_pairs_of(&texts.compare_output)?;
    for pair in &pairs {
        output.emit(&format!(
            "name=commit_pair pair={} invalidated_block_fraction={} invalidated_state_fraction={} judging_unit_key_changed={}",
            pair.pair,
            fraction_text(pair.block_fraction),
            fraction_text(pair.state_fraction),
            pair.judging_unit_key_changed
        ));
    }
    let final_index = texts
        .final_pair
        .as_ref()
        .and_then(|name| pairs.iter().position(|pair| &pair.pair == name));
    let mut median_state_fraction = None;
    for (basis, fractions) in [
        (
            "blocks",
            pairs
                .iter()
                .map(|pair| pair.block_fraction)
                .collect::<Vec<f64>>(),
        ),
        (
            "states",
            pairs
                .iter()
                .map(|pair| pair.state_fraction)
                .collect::<Vec<f64>>(),
        ),
    ] {
        let Some(trajectory) = commit_pair_trajectory(&fractions, final_index) else {
            output.emit(&format!(
                "name=commit_pairs_trajectory basis={basis} computable=0 verdict={}",
                CommitPairsVerdict::StillMissing.name()
            ));
            continue;
        };
        if basis == "states" {
            median_state_fraction = Some(trajectory.median);
        }
        output.emit(&format!(
            "name=commit_pairs_trajectory basis={basis} computable={} positive={} peak={} median={} final={} verdict={}",
            trajectory.computable,
            trajectory.positive,
            fraction_text(trajectory.peak),
            fraction_text(trajectory.median),
            trajectory.final_value.map_or_else(|| "none".to_string(), fraction_text),
            commit_pairs_verdict(&fractions).name()
        ));
    }
    let upper_bound =
        patch_state_fraction_in_the_nominal_load(&texts.compare_output, "journal-mac")?;
    let inputs = compose_inputs_of(&rows, keys_files, median_state_fraction, upper_bound)?;
    output.emit(&format!(
        "name=compose_input flows_in_the_nominal_load={} keys_seconds_total={:.1} full_total={} in_place_subsets_total={} points_total={}",
        rows.iter().filter(|row| is_in_the_nominal_regular_layer_load(row)).count(),
        inputs.keys_seconds_total,
        inputs.full_total,
        inputs.in_place_subsets_total,
        inputs.point_states.len()
    ));
    for (source, run) in [("small", &small), ("medium", &medium)] {
        for body in cell_bodies(
            &inputs,
            source,
            &Rates {
                total: run.rate_total(),
                judge: run.rate_judge(),
            },
        ) {
            output.emit(&body);
        }
    }
    output.emit_closing_line(
        "compose_done",
        &format!("commit_pairs_computable={}", pairs.len()),
    );
    Ok(EXIT_CODE_OK)
}

fn read_texts(files: &[String]) -> Result<String, String> {
    let mut text = String::new();
    for file in files {
        text.push_str(
            &std::fs::read_to_string(file).map_err(|error| format!("读 {file} 失败：{error}"))?,
        );
        text.push('\n');
    }
    Ok(text)
}

pub(super) fn run_compose_command(arguments: &[String]) -> i32 {
    let parsed = match ParsedOptions::parse(
        arguments,
        &[
            "--j1",
            "--keys",
            "--j3",
            "--j2-small",
            "--j2-medium",
            "--library-bytes",
            "--recorded-blocks",
            "--final-pair",
            "--out",
        ],
        &[],
    ) {
        Ok(parsed) => parsed,
        Err(message) => return usage_error(&message),
    };
    let loaded = (|| -> Result<(ComposeTexts, Vec<KeysFile>, PathBuf), String> {
        parsed.paths("--j1")?;
        parsed.paths("--j3")?;
        let texts = ComposeTexts {
            keys_output: read_texts(parsed.every_value_given_for("--j1"))?,
            compare_output: read_texts(parsed.every_value_given_for("--j3"))?,
            small_log: read_texts(&[parsed.single("--j2-small")?.to_string()])?,
            medium_log: read_texts(&[parsed.single("--j2-medium")?.to_string()])?,
            library_bytes: read_texts(&[parsed.single("--library-bytes")?.to_string()])?,
            recorded_blocks: read_texts(&[parsed.single("--recorded-blocks")?.to_string()])?,
            final_pair: parsed
                .every_value_given_for("--final-pair")
                .first()
                .cloned(),
        };
        Ok((
            texts,
            keys_file_paths(&parsed.paths("--keys")?, Expansion::Full)?
                .iter()
                .filter(|path| {
                    label_is_in_the_nominal_regular_layer_load(&label_of_the_keys_file(path))
                })
                .map(|path| read_keys_file(path, Expansion::Full))
                .collect::<Result<Vec<KeysFile>, String>>()?,
            PathBuf::from(parsed.single("--out")?),
        ))
    })();
    let (texts, keys_files, out_file) = match loaded {
        Ok(loaded) => loaded,
        Err(message) => return usage_error(&message),
    };
    let mut output = Output::new();
    let exit_code = match compose_lines(&texts, &keys_files, &mut output) {
        Ok(exit_code) => exit_code,
        Err(message) => {
            eprintln!("合算失败：{message}");
            return EXIT_CODE_STOP_CLAUSE_FAILED;
        }
    };
    if let Err(message) = output.write_to(&out_file) {
        eprintln!("写 {} 失败：{message}", out_file.display());
        return EXIT_CODE_USAGE;
    }
    exit_code
}

#[cfg(test)]
mod tests;
