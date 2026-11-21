use super::*;

// ───────────────────────────── share：J1-乙（态数区间重叠）与 J1-丙（共用块键） ─────────────────────────────

/// J1-丙的判定（登记第六节）：换名 = renamed、独立 = independent、部分 = partial。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShareVerdict {
    Renamed,
    Independent,
    Partial,
}

impl ShareVerdict {
    pub(super) fn name(self) -> &'static str {
        match self {
            ShareVerdict::Renamed => "renamed",
            ShareVerdict::Independent => "independent",
            ShareVerdict::Partial => "partial",
        }
    }
}

/// 族中位不小于门槛记换名；族最大不大于门槛记独立；其余记部分。
fn share_verdict(median: f64, maximum: f64) -> ShareVerdict {
    if median >= SHARE_MEDIAN_AT_LEAST_FOR_RENAMED {
        ShareVerdict::Renamed
    } else if maximum <= SHARE_MAXIMUM_AT_MOST_FOR_INDEPENDENT {
        ShareVerdict::Independent
    } else {
        ShareVerdict::Partial
    }
}

/// J1-乙的判定：重叠 = overlap、分开 = separate、部分 = partial。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OverlapVerdict {
    Overlap,
    Separate,
    Partial,
}

impl OverlapVerdict {
    pub(super) fn name(self) -> &'static str {
        match self {
            OverlapVerdict::Overlap => "overlap",
            OverlapVerdict::Separate => "separate",
            OverlapVerdict::Partial => "partial",
        }
    }
}

fn overlap_verdict(inside_fraction: f64) -> OverlapVerdict {
    if inside_fraction >= INSIDE_FRACTION_AT_LEAST_FOR_OVERLAP {
        OverlapVerdict::Overlap
    } else if inside_fraction <= INSIDE_FRACTION_AT_MOST_FOR_SEPARATE {
        OverlapVerdict::Separate
    } else {
        OverlapVerdict::Partial
    }
}

/// 候选流共用占比（按块）：候选的块里块键在比较集里的块数 ÷ 候选自己的块数。
fn block_share_fraction(candidate: &[BlockOfAPlan], against: &BTreeSet<VerdictBlockKey>) -> f64 {
    let (shared_blocks, _shared_states) = shared_blocks_and_states(candidate, against);
    shared_blocks as f64 / candidate.len() as f64
}

/// 候选流共用占比（按态数加权）：共用块里的态数 ÷ 候选的态数。
fn weighted_share_fraction(candidate: &[BlockOfAPlan], against: &BTreeSet<VerdictBlockKey>) -> f64 {
    let (_shared_blocks, shared_states) = shared_blocks_and_states(candidate, against);
    let states: u64 = candidate.iter().map(|block| block.states).sum();
    shared_states as f64 / states as f64
}

/// 复用键共用占比（块键共用的上界，不含计划哈希）：复用键在比较集里的块数 ÷ 候选的块数。
pub(super) fn reuse_key_share_fraction(
    candidate: &[BlockOfAPlan],
    against_reuse_keys: &BTreeSet<[u8; 32]>,
) -> f64 {
    let shared = candidate
        .iter()
        .filter(|block| against_reuse_keys.contains(&block.key.node_input_digest.0))
        .count();
    shared as f64 / candidate.len() as f64
}

/// 冒烟层态数区间 S：最小与最大的两态闭式，和最长段的最小与最大（登记 1.1 节「冒烟层态数区间 S」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SmokeInterval {
    two_state_minimum: u64,
    two_state_maximum: u64,
    longest_segment_minimum: usize,
    longest_segment_maximum: usize,
}

/// 冒烟层里钉了常量的一条流：流名、两态闭式、最长段（执行员 grep 测试文件取常量、照闭式现算核过再写进 `--pinned-smoke` 文件）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct PinnedSmokeFlow {
    pub(super) label: String,
    pub(super) two_state: u64,
    pub(super) longest_segment: usize,
}

/// `--pinned-smoke` 文件：一行一条流，`<流名> <两态闭式> <最长段>`，空行与 `#` 开头的行不算。
fn parse_pinned_smoke(text: &str) -> Result<Vec<PinnedSmokeFlow>, String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let words: Vec<&str> = line.split_whitespace().collect();
            let [label, two_state, longest_segment] = words.as_slice() else {
                return Err(format!(
                    "钉住的冒烟流要三段（流名 两态闭式 最长段），读到 {line:?}"
                ));
            };
            Ok(PinnedSmokeFlow {
                label: (*label).to_string(),
                two_state: two_state
                    .parse::<u64>()
                    .map_err(|error| format!("{line:?} 的两态闭式：{error}"))?,
                longest_segment: longest_segment
                    .parse::<usize>()
                    .map_err(|error| format!("{line:?} 的最长段：{error}"))?,
            })
        })
        .collect()
}

/// S：现录的冒烟层流（`baseline` 层）加钉住的那几条。
///
/// # Errors
/// 一条流都没有；现录的冒烟层流两态闭式算不出（有段不少于 64 写）。
fn smoke_interval(
    baseline_rows: &[FlowRow],
    pinned: &[PinnedSmokeFlow],
) -> Result<SmokeInterval, String> {
    let mut two_states = Vec::new();
    let mut longest_segments = Vec::new();
    for row in baseline_rows {
        two_states.push(
            row.two_state
                .ok_or_else(|| format!("冒烟层流 {} 的两态闭式算不出", row.label))?,
        );
        longest_segments.push(row.longest_segment);
    }
    for flow in pinned {
        two_states.push(flow.two_state);
        longest_segments.push(flow.longest_segment);
    }
    Ok(SmokeInterval {
        two_state_minimum: *two_states
            .iter()
            .min()
            .ok_or("一条冒烟层的流都没有，区间 S 定不出")?,
        two_state_maximum: *two_states
            .iter()
            .max()
            .ok_or("一条冒烟层的流都没有，区间 S 定不出")?,
        longest_segment_minimum: *longest_segments
            .iter()
            .min()
            .ok_or("一条冒烟层的流都没有，区间 S 定不出")?,
        longest_segment_maximum: *longest_segments
            .iter()
            .max()
            .ok_or("一条冒烟层的流都没有，区间 S 定不出")?,
    })
}

/// 一个值对区间 S 的位置。区间两端都算在里面（闭区间）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlaceAgainstInterval {
    Below,
    Inside,
    Above,
}

fn place_against(value: Option<u64>, minimum: u64, maximum: u64) -> PlaceAgainstInterval {
    match value {
        None => PlaceAgainstInterval::Above,
        Some(known) if known < minimum => PlaceAgainstInterval::Below,
        Some(known) if known > maximum => PlaceAgainstInterval::Above,
        Some(_) => PlaceAgainstInterval::Inside,
    }
}

/// 候选流的分组：每个（族，组）一个范围，另加两个主取样的合并范围（随机历史：比重 BROAD、盘宽 FourGibibytes 的各步数档；
/// 覆盖写链：每次覆盖写 4 个数据单元那一档）。敏感性取样点（盘宽、比重、步数、每次覆盖写的数据单元数）各自是单独的组，
/// 同一个门槛在两个取样点上各判一次（登记 8.2）。冒烟层的流（`baseline` 层）不在候选里。
fn scopes_of_the_candidates(rows: &[FlowRow]) -> Vec<(String, Vec<usize>)> {
    let mut scopes: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, row) in rows.iter().enumerate() {
        if row.tier != "candidate" {
            continue;
        }
        let group = row.group();
        if group.starts_with("random_history-broad-FourGibibytes-steps") {
            scopes
                .entry("family:random_history_main".to_string())
                .or_default()
                .push(index);
        }
        if group == "overwrite_chain-04_units_per_overwrite" {
            scopes
                .entry("family:overwrite_chain_main".to_string())
                .or_default()
                .push(index);
        }
        scopes
            .entry(format!("group:{group}"))
            .or_default()
            .push(index);
    }
    scopes.into_iter().collect()
}

/// 一个范围里写数不少于 1000 的流对区间 S 的位置。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
struct OverlapCounts {
    pub(super) flows: usize,
    flows_with_writes_at_least_1000: usize,
    inside: usize,
    above: usize,
    below: usize,
    longest_segment_inside: usize,
}

impl OverlapCounts {
    pub(super) fn of(members: &[&FlowRow], interval: &SmokeInterval) -> Self {
        let mut counts = Self {
            flows: members.len(),
            ..Self::default()
        };
        for row in members
            .iter()
            .filter(|row| row.writes >= WRITES_OF_A_THOUSAND_WRITE_STREAM)
        {
            counts.flows_with_writes_at_least_1000 += 1;
            match place_against(
                row.two_state,
                interval.two_state_minimum,
                interval.two_state_maximum,
            ) {
                PlaceAgainstInterval::Inside => counts.inside += 1,
                PlaceAgainstInterval::Above => counts.above += 1,
                PlaceAgainstInterval::Below => counts.below += 1,
            }
            if (interval.longest_segment_minimum..=interval.longest_segment_maximum)
                .contains(&row.longest_segment)
            {
                counts.longest_segment_inside += 1;
            }
        }
        counts
    }

    /// 占写数不少于 1000 的流的比例；一条都没有交 `None`。
    fn fraction_of(self, count: usize) -> Option<f64> {
        (self.flows_with_writes_at_least_1000 > 0)
            .then(|| count as f64 / self.flows_with_writes_at_least_1000 as f64)
    }

    pub(super) fn body(self, scope: &str, interval: &SmokeInterval) -> String {
        let fraction_text = |fraction: Option<f64>| {
            fraction.map_or_else(|| "none".to_string(), |value| format!("{value:.6}"))
        };
        let verdict = self
            .fraction_of(self.inside)
            .map_or("no_flow_with_writes_at_least_1000", |fraction| {
                overlap_verdict(fraction).name()
            });
        format!(
            "name=overlap scope={scope} smoke_interval_two_state_minimum={} smoke_interval_two_state_maximum={} smoke_interval_longest_segment_minimum={} smoke_interval_longest_segment_maximum={} flows={} flows_with_writes_at_least_1000={} inside={} above={} below={} inside_fraction={} above_fraction={} at_most_smoke_interval_maximum_fraction={} longest_segment_inside_fraction={} verdict={verdict}",
            interval.two_state_minimum,
            interval.two_state_maximum,
            interval.longest_segment_minimum,
            interval.longest_segment_maximum,
            self.flows,
            self.flows_with_writes_at_least_1000,
            self.inside,
            self.above,
            self.below,
            fraction_text(self.fraction_of(self.inside)),
            fraction_text(self.fraction_of(self.above)),
            fraction_text(self.fraction_of(self.inside + self.below)),
            fraction_text(self.fraction_of(self.longest_segment_inside)),
        )
    }
}

/// `share` 命令行读出来的输入：候选目录、比较集目录、输出文件、钉住的冒烟流。
type ShareInputs = (Vec<PathBuf>, Vec<PathBuf>, PathBuf, Vec<PinnedSmokeFlow>);

/// `share`：候选各流对比较集（冒烟层四条流的块键并集）的共用，加 J1-乙的区间重叠与自比对照。
pub(super) fn run_share_command(arguments: &[String]) -> i32 {
    let parsed = match ParsedOptions::parse(
        arguments,
        &["--candidates", "--against", "--out", "--pinned-smoke"],
        &[],
    ) {
        Ok(parsed) => parsed,
        Err(message) => return usage_error(&message),
    };
    let inputs = (|| -> Result<ShareInputs, String> {
        let pinned = match parsed.every_value_given_for("--pinned-smoke") {
            [] => Vec::new(),
            [file] => parse_pinned_smoke(
                &std::fs::read_to_string(file)
                    .map_err(|error| format!("读 {file} 失败：{error}"))?,
            )?,
            _ => return Err("--pinned-smoke 只许给一次".to_string()),
        };
        Ok((
            parsed.paths("--candidates")?,
            parsed.paths("--against")?,
            PathBuf::from(parsed.single("--out")?),
            pinned,
        ))
    })();
    let (candidate_directories, against_directories, out_file, pinned) = match inputs {
        Ok(inputs) => inputs,
        Err(message) => return usage_error(&message),
    };
    let (candidate_paths, against) = match (
        keys_file_paths(&candidate_directories, Expansion::Full),
        read_keys_files(&against_directories, Expansion::Full),
    ) {
        (Ok(candidate_paths), Ok(against)) => (candidate_paths, against),
        (Err(message), _) | (_, Err(message)) => {
            eprintln!("读块键文件失败：{message}");
            return EXIT_CODE_STOP_CLAUSE_FAILED;
        }
    };
    let mut output = Output::new();
    let candidates = candidate_paths
        .iter()
        .map(|path| read_keys_file(path, Expansion::Full));
    let exit_code = match share_lines(candidates, &against, &pinned, &mut output) {
        Ok(exit_code) => exit_code,
        Err(message) => {
            eprintln!("读块键文件失败：{message}");
            return EXIT_CODE_STOP_CLAUSE_FAILED;
        }
    };
    if let Err(message) = output.write_to(&out_file) {
        eprintln!("写 {} 失败：{message}", out_file.display());
        return EXIT_CODE_USAGE;
    }
    exit_code
}

pub(super) fn fraction_text(fraction: f64) -> String {
    format!("{fraction:.6}")
}

/// 自比对照：一条流对「只含它自己」的比较集，共用占比恰好 1（共用块数等于它的块数）。
fn self_share_control_body(file: &KeysFile) -> (String, bool) {
    let own_keys = block_keys_of(&file.blocks);
    let (shared_blocks, _shared_states) = shared_blocks_and_states(&file.blocks, &own_keys);
    let passed = shared_blocks == file.blocks.len()
        && (block_share_fraction(&file.blocks, &own_keys) - 1.0).abs() < f64::EPSILON;
    (
        format!(
            "name=control kind=self_share flow={} passed={passed} blocks={} shared_blocks={shared_blocks} expected_shared_blocks={}",
            file.row.label,
            file.blocks.len(),
            file.blocks.len()
        ),
        passed,
    )
}

/// `share` 的全部输出行。候选一条一条读（最大的流有上千万块，不一起放进内存），只留每条流的行和三个占比。
/// 交回退出码：0；区间 S 定不出 3；自比对照没过 4。
///
/// # Errors
/// 某一份候选的块键文件读不回。
fn share_lines(
    candidates: impl Iterator<Item = Result<KeysFile, String>>,
    against: &[KeysFile],
    pinned: &[PinnedSmokeFlow],
    output: &mut Output,
) -> Result<i32, String> {
    // 比较集只收冒烟层（baseline 层）的流，候选只收 candidate 层的流：两边的目录里混了别层的流时，不静默当成别的层
    let against: Vec<KeysFile> = against
        .iter()
        .filter(|file| file.row.tier == "baseline")
        .cloned()
        .collect();
    let against = against.as_slice();
    let against_blocks: Vec<BlockOfAPlan> = against
        .iter()
        .flat_map(|file| file.blocks.iter().cloned())
        .collect();
    let against_keys = block_keys_of(&against_blocks);
    let against_reuse_keys = reuse_keys_of(&against_blocks);
    let mut controls_failed = 0usize;
    for file in against.iter().filter(|file| !file.blocks.is_empty()) {
        let (body, passed) = self_share_control_body(file);
        controls_failed += usize::from(!passed);
        output.emit(&body);
    }
    let mut rows: Vec<FlowRow> = Vec::new();
    let mut share_fractions: Vec<Option<(f64, f64, f64)>> = Vec::new();
    for candidate in candidates {
        let file = candidate?;
        if file.row.tier != "candidate" {
            output.emit(&format!(
                "name=share_ignored flow={} reason=tier_is_{}",
                file.row.label, file.row.tier
            ));
            continue;
        }
        rows.push(file.row.clone());
        if file.blocks.is_empty() {
            output.emit(&format!(
                "name=share_not_planned flow={} reason=no_blocks_planned",
                file.row.label
            ));
            share_fractions.push(None);
            continue;
        }
        let (body, passed) = self_share_control_body(&file);
        controls_failed += usize::from(!passed);
        output.emit(&body);
        let (shared_blocks, shared_states) = shared_blocks_and_states(&file.blocks, &against_keys);
        let block_fraction = block_share_fraction(&file.blocks, &against_keys);
        let weighted_fraction = weighted_share_fraction(&file.blocks, &against_keys);
        let reuse_fraction = reuse_key_share_fraction(&file.blocks, &against_reuse_keys);
        output.emit(&format!(
            "name=share flow={} family={} tier={} group={} blocks={} shared_blocks={shared_blocks} share_fraction={} states={} shared_states={shared_states} weighted_fraction={} reuse_fraction={}",
            file.row.label,
            file.row.family,
            file.row.tier,
            file.row.group(),
            file.blocks.len(),
            fraction_text(block_fraction),
            file.state_total(),
            fraction_text(weighted_fraction),
            fraction_text(reuse_fraction),
        ));
        share_fractions.push(Some((block_fraction, weighted_fraction, reuse_fraction)));
    }
    let baseline_rows: Vec<FlowRow> = against.iter().map(|file| file.row.clone()).collect();
    let interval = match smoke_interval(&baseline_rows, pinned) {
        Ok(interval) => interval,
        Err(message) => {
            output.emit(&format!(
                "name=stop_check item=S_interval result=fail detail={}",
                message.replace(' ', "_")
            ));
            output.emit_closing_line(
                "share_done",
                &format!(
                    "candidates={} against={} controls_failed={controls_failed}",
                    rows.len(),
                    against.len()
                ),
            );
            return Ok(EXIT_CODE_STOP_CLAUSE_FAILED);
        }
    };
    for (scope, members) in scopes_of_the_candidates(&rows) {
        let planned: Vec<(f64, f64, f64)> = members
            .iter()
            .filter_map(|index| share_fractions[*index])
            .collect();
        let spread_of = |pick: &dyn Fn(&(f64, f64, f64)) -> f64| {
            Spread::of(&planned.iter().map(pick).collect::<Vec<f64>>())
        };
        if let (Some(blocks), Some(weighted), Some(reuse)) = (
            spread_of(&|reading| reading.0),
            spread_of(&|reading| reading.1),
            spread_of(&|reading| reading.2),
        ) {
            output.emit(&format!(
                "name=share_group scope={scope} flows={} planned_flows={} median_share_fraction={} maximum_share_fraction={} median_weighted_fraction={} median_reuse_fraction={} maximum_reuse_fraction={} verdict={}",
                members.len(),
                planned.len(),
                fraction_text(blocks.median),
                fraction_text(blocks.maximum),
                fraction_text(weighted.median),
                fraction_text(reuse.median),
                fraction_text(reuse.maximum),
                share_verdict(blocks.median, blocks.maximum).name(),
            ));
        } else {
            output.emit(&format!(
                "name=share_group scope={scope} flows={} planned_flows=0 verdict=not_planned",
                members.len()
            ));
        }
        let member_rows: Vec<&FlowRow> = members.iter().map(|index| &rows[*index]).collect();
        output.emit(&OverlapCounts::of(&member_rows, &interval).body(&scope, &interval));
    }
    output.emit_closing_line(
        "share_done",
        &format!(
            "candidates={} against={} controls_failed={controls_failed}",
            rows.len(),
            against.len()
        ),
    );
    Ok(if controls_failed > 0 {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else {
        EXIT_CODE_OK
    })
}

#[cfg(test)]
mod tests;
