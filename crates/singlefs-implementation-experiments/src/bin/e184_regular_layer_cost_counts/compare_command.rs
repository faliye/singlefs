use super::*;

// ───────────────────────────── compare：J3，前后两份写流的第一处不同与作废块数 ─────────────────────────────

/// 两份写流的第一处不同。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FirstDifference {
    Identical,
    /// 基镜像就不同（mkfs 写的东西变了）。
    Base,
    /// 写表里第一条不同的写的下标（一边比另一边长时，是较短那边的写数）。
    Write(usize),
}

impl FirstDifference {
    pub(super) fn text(self) -> String {
        match self {
            FirstDifference::Identical => "none".to_string(),
            FirstDifference::Base => "base".to_string(),
            FirstDifference::Write(index) => format!("write:{index}"),
        }
    }
}

pub(super) fn first_difference(before: &FlowDump, after: &FlowDump) -> FirstDifference {
    if before.base_sectors_digest() != after.base_sectors_digest() {
        return FirstDifference::Base;
    }
    let shared = before.writes.len().min(after.writes.len());
    match (0..shared).find(|index| before.writes[*index] != after.writes[*index]) {
        Some(index) => FirstDifference::Write(index),
        None if before.writes.len() == after.writes.len() => FirstDifference::Identical,
        None => FirstDifference::Write(shared),
    }
}

/// 写下标落在第几段（前一份流的段）；不在任何段里交 `None`。
fn segment_of_write(segments: &[Vec<usize>], write_index: usize) -> Option<usize> {
    segments
        .iter()
        .position(|segment| segment.contains(&write_index))
}

/// 写下标落在崩溃枚举用例那一路切出来的第几个点里；不在任何点里交 `None`。
fn point_of_write(dump: &FlowDump, write_index: usize) -> Option<usize> {
    crash_points_of_the_steps(default_item(), &dump.writes, &dump.segments)
        .iter()
        .position(|span| span.writes.contains(&write_index))
}

/// 作废的块：后一份流里块键不在前一份流块键集合里的块（下一趟要现判的）。
pub(super) fn invalidated_blocks<'blocks>(
    before: &[BlockOfAPlan],
    after: &'blocks [BlockOfAPlan],
) -> Vec<&'blocks BlockOfAPlan> {
    let before_keys = block_keys_of(before);
    after
        .iter()
        .filter(|block| !before_keys.contains(&block.key))
        .collect()
}

/// 一条流前后对比的结果。
#[derive(Clone, Debug)]
pub(super) struct ComparedFlow {
    pub(super) label: String,
    pub(super) group: String,
    pub(super) first_difference: FirstDifference,
    pub(super) segment: Option<usize>,
    pub(super) point: Option<usize>,
    pub(super) first_journal_record_write: Option<usize>,
    pub(super) blocks_before: usize,
    pub(super) blocks_after: usize,
    pub(super) invalidated_blocks: usize,
    pub(super) states_after: u64,
    pub(super) invalidated_states: u64,
}

impl ComparedFlow {
    pub(super) fn block_fraction(&self) -> f64 {
        self.invalidated_blocks as f64 / self.blocks_after as f64
    }

    pub(super) fn state_fraction(&self) -> f64 {
        self.invalidated_states as f64 / self.states_after as f64
    }

    pub(super) fn body(&self) -> String {
        let optional = |value: Option<usize>| {
            value.map_or_else(|| "none".to_string(), |number| number.to_string())
        };
        format!(
            "name=compare flow={} group={} first_difference={} segment={} point={} first_journal_record_write={} blocks_before={} blocks_after={} invalidated_blocks={} states_after={} invalidated_states={} invalidated_block_fraction={} invalidated_state_fraction={}",
            self.label,
            self.group,
            self.first_difference.text(),
            if self.first_difference == FirstDifference::Base { "base".to_string() } else { optional(self.segment) },
            if self.first_difference == FirstDifference::Base { "base".to_string() } else { optional(self.point) },
            optional(self.first_journal_record_write),
            self.blocks_before,
            self.blocks_after,
            self.invalidated_blocks,
            self.states_after,
            self.invalidated_states,
            fraction_text(self.block_fraction()),
            fraction_text(self.state_fraction()),
        )
    }
}

fn compare_one_flow(
    before_dump: &FlowDump,
    after_dump: &FlowDump,
    before_keys: &KeysFile,
    after_keys: &KeysFile,
) -> ComparedFlow {
    let first = first_difference(before_dump, after_dump);
    let invalidated = invalidated_blocks(&before_keys.blocks, &after_keys.blocks);
    ComparedFlow {
        label: after_keys.row.label.clone(),
        group: after_keys.row.group(),
        first_difference: first,
        segment: match first {
            FirstDifference::Write(index) => segment_of_write(&before_dump.segments, index),
            FirstDifference::Identical | FirstDifference::Base => None,
        },
        point: match first {
            FirstDifference::Write(index) => point_of_write(before_dump, index),
            FirstDifference::Identical | FirstDifference::Base => None,
        },
        first_journal_record_write: before_dump
            .writes
            .iter()
            .position(|write| write.kind == StepKind::JournalRecord),
        blocks_before: before_keys.blocks.len(),
        blocks_after: after_keys.blocks.len(),
        invalidated_blocks: invalidated.len(),
        states_after: after_keys.state_total(),
        invalidated_states: invalidated.iter().map(|block| block.states).sum(),
    }
}

/// 同一组的几条流汇总成一行：作废块占后一份流块数的比例（几条流的块合起来算），按态数也算一遍，另报逐条流占比的中位与最大。
pub(super) fn compare_summary_body(
    kind: &str,
    pair: &str,
    group: &str,
    flows: &[&ComparedFlow],
    unavailable: usize,
    key_unit_changed: &str,
) -> String {
    let blocks_after: usize = flows.iter().map(|flow| flow.blocks_after).sum();
    let invalidated: usize = flows.iter().map(|flow| flow.invalidated_blocks).sum();
    let states_after: u64 = flows.iter().map(|flow| flow.states_after).sum();
    let invalidated_states: u64 = flows.iter().map(|flow| flow.invalidated_states).sum();
    let per_flow = Spread::of(
        &flows
            .iter()
            .map(|flow| flow.block_fraction())
            .collect::<Vec<f64>>(),
    );
    format!(
        "name=compare_summary kind={kind} pair={pair} group={group} flows_compared={} flows_unavailable={unavailable} flows_with_a_difference={} blocks_after={blocks_after} invalidated_blocks={invalidated} invalidated_block_fraction={} states_after={states_after} invalidated_states={invalidated_states} invalidated_state_fraction={} per_flow_median_block_fraction={} per_flow_maximum_block_fraction={} judging_unit_key_changed={key_unit_changed}",
        flows.len(),
        flows.iter().filter(|flow| flow.first_difference != FirstDifference::Identical).count(),
        fraction_text(invalidated as f64 / blocks_after as f64),
        fraction_text(invalidated_states as f64 / states_after as f64),
        per_flow.as_ref().map_or_else(|| "none".to_string(), |spread| fraction_text(spread.median)),
        per_flow.as_ref().map_or_else(|| "none".to_string(), |spread| fraction_text(spread.maximum)),
    )
}

/// 补丁位该造出什么样的第一处不同（阳性对照 A9、A10、A11）：`journal-record` = 基镜像就不同，或第一处不同的写就是写表里第一条 journal 记录写；
/// `none` = 两份写流逐字相同、一块都不作废。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExpectedFirstDifference {
    JournalRecord,
    Identical,
}

impl ExpectedFirstDifference {
    pub(super) fn parse(text: &str) -> Result<Self, String> {
        match text {
            "journal-record" => Ok(Self::JournalRecord),
            "none" => Ok(Self::Identical),
            other => Err(format!(
                "--expect-first-difference 要写 journal-record 或 none，读到 {other:?}"
            )),
        }
    }

    fn holds_for(self, flow: &ComparedFlow) -> bool {
        match self {
            ExpectedFirstDifference::Identical => {
                flow.first_difference == FirstDifference::Identical && flow.invalidated_blocks == 0
            }
            ExpectedFirstDifference::JournalRecord => match flow.first_difference {
                FirstDifference::Base => true,
                FirstDifference::Write(index) => Some(index) == flow.first_journal_record_write,
                FirstDifference::Identical => false,
            },
        }
    }
}

/// 目录里这条流的块键文件与转储；两样有一样不在交 `None`（这条流这一侧取不出），在而读不回交回说明（不当没有）。
fn flow_files_of(directory: &Path, label: &str) -> Result<Option<(KeysFile, FlowDump)>, String> {
    let keys_path = directory.join(format!("{label}{FULL_KEYS_FILE_SUFFIX}"));
    let dump_path = directory.join(format!("{label}{DUMP_FILE_SUFFIX}"));
    if !keys_path.exists() || !dump_path.exists() {
        return Ok(None);
    }
    Ok(Some((
        read_keys_file(&keys_path, Expansion::Full)?,
        FlowDump::read_from(&dump_path)?,
    )))
}

pub(super) fn run_compare_command(arguments: &[String]) -> i32 {
    let parsed = match ParsedOptions::parse(
        arguments,
        &[
            "--before",
            "--after",
            "--kind",
            "--pair",
            "--out",
            "--judging-unit-key-changed",
            "--expect-first-difference",
        ],
        &[],
    ) {
        Ok(parsed) => parsed,
        Err(message) => return usage_error(&message),
    };
    let optional_single = |option: &str| parsed.every_value_given_for(option).first().cloned();
    let inputs = (|| -> Result<_, String> {
        let kind = parsed.single("--kind")?;
        if kind != "commit-pair" && kind != "patch" {
            return Err(format!("--kind 要写 commit-pair 或 patch，读到 {kind:?}"));
        }
        let expectation = optional_single("--expect-first-difference")
            .map(|text| ExpectedFirstDifference::parse(&text))
            .transpose()?;
        Ok((
            PathBuf::from(parsed.single("--before")?),
            PathBuf::from(parsed.single("--after")?),
            kind.to_string(),
            parsed.single("--pair")?.to_string(),
            PathBuf::from(parsed.single("--out")?),
            optional_single("--judging-unit-key-changed")
                .unwrap_or_else(|| "unavailable".to_string()),
            expectation,
        ))
    })();
    let (before_directory, after_directory, kind, pair, out_file, key_unit_changed, expectation) =
        match inputs {
            Ok(inputs) => inputs,
            Err(message) => return usage_error(&message),
        };
    let after_paths = match keys_file_paths(std::slice::from_ref(&after_directory), Expansion::Full)
    {
        Ok(after_paths) => after_paths,
        Err(message) => {
            eprintln!("读前后两份流失败：{message}");
            return EXIT_CODE_STOP_CLAUSE_FAILED;
        }
    };
    let mut output = Output::new();
    let (mut compared, mut unavailable, mut controls_failed) = (Vec::new(), 0usize, 0usize);
    for after_path in &after_paths {
        let label = label_of_the_keys_file(after_path);
        let pair_of_sides = flow_files_of(&before_directory, &label)
            .and_then(|before| Ok((before, flow_files_of(&after_directory, &label)?)));
        let (Some((before, before_dump)), Some((after, after_dump))) = (match pair_of_sides {
            Ok(sides) => sides,
            Err(message) => {
                eprintln!("读前后两份流失败：{message}");
                return EXIT_CODE_STOP_CLAUSE_FAILED;
            }
        }) else {
            unavailable += 1;
            output.emit(&format!(
                "name=compare_unavailable flow={label} reason=missing_on_one_side"
            ));
            continue;
        };
        if before.blocks.is_empty() || after.blocks.is_empty() {
            unavailable += 1;
            output.emit(&format!(
                "name=compare_unavailable flow={label} reason=no_blocks_planned"
            ));
            continue;
        }
        let flow = compare_one_flow(&before_dump, &after_dump, &before, &after);
        output.emit(&flow.body());
        if let Some(expected) = expectation {
            let passed = expected.holds_for(&flow);
            controls_failed += usize::from(!passed);
            output.emit(&format!("name=control kind=expected_first_difference flow={label} passed={passed} expected={expected:?} first_difference={}", flow.first_difference.text()));
        }
        compared.push(flow);
    }
    let mut groups: BTreeMap<String, Vec<&ComparedFlow>> = BTreeMap::new();
    for flow in &compared {
        groups.entry(flow.group.clone()).or_default().push(flow);
    }
    for (group, flows) in &groups {
        output.emit(&compare_summary_body(
            &kind,
            &pair,
            group,
            flows,
            unavailable,
            &key_unit_changed,
        ));
    }
    output.emit_closing_line("compare_done", &format!("kind={kind} pair={pair} flows_compared={} flows_unavailable={unavailable} controls_failed={controls_failed}", compared.len()));
    if let Err(message) = output.write_to(&out_file) {
        eprintln!("写 {} 失败：{message}", out_file.display());
        return EXIT_CODE_USAGE;
    }
    if controls_failed > 0 {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else {
        EXIT_CODE_OK
    }
}

#[cfg(test)]
mod tests;
