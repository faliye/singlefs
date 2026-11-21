//! 块的类别与判定、一条路径的核对计划、整库的核对与核对角色。

use super::*;

// ============================== 逐块核（登记 5.1 的表，两路共 N 块） ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockClass {
    Confirmed,
    InFlight,
    NotStarted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadKind {
    MatchingBytes,
    DifferentBytes,
    NotFound,
    ReadError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockVerdict {
    Correct,
    /// Q3d：收到过确认（直写：提交返回过）而读不到。
    LostConfirmed,
    /// Q3e：读到而字节不同。
    SilentlyCorrupted,
    /// Q3f 的一半：已确认而读报错。
    ConfirmedReadError,
    /// Q3f 的另一半：在途或未开始而读报错。
    UnconfirmedReadError,
    /// Q3g：读到未开始的块。
    Phantom,
}

fn classify_block(class: BlockClass, read: ReadKind) -> BlockVerdict {
    match (class, read) {
        (BlockClass::Confirmed, ReadKind::MatchingBytes) => BlockVerdict::Correct,
        (BlockClass::Confirmed, ReadKind::DifferentBytes) => BlockVerdict::SilentlyCorrupted,
        (BlockClass::Confirmed, ReadKind::NotFound) => BlockVerdict::LostConfirmed,
        (BlockClass::Confirmed, ReadKind::ReadError) => BlockVerdict::ConfirmedReadError,
        (BlockClass::InFlight, ReadKind::MatchingBytes) => BlockVerdict::Correct,
        (BlockClass::InFlight, ReadKind::DifferentBytes) => BlockVerdict::SilentlyCorrupted,
        (BlockClass::InFlight, ReadKind::NotFound) => BlockVerdict::Correct,
        (BlockClass::InFlight, ReadKind::ReadError) => BlockVerdict::UnconfirmedReadError,
        (BlockClass::NotStarted, ReadKind::MatchingBytes | ReadKind::DifferentBytes) => BlockVerdict::Phantom,
        (BlockClass::NotStarted, ReadKind::NotFound) => BlockVerdict::Correct,
        (BlockClass::NotStarted, ReadKind::ReadError) => BlockVerdict::UnconfirmedReadError,
    }
}

/// 两路各自哪些块已确认、在途、未开始：直写是顺序的，已确认是前缀 [0, confirmed)、在途是 [confirmed, started)；
/// 网络那一路按送块进程报的确认区间与「发出过的块序号」[0, sent)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PathVerificationPlan {
    pub(super) states_per_block: u64,
    pub(super) direct_blocks: u64,
    pub(super) direct_confirmed: u64,
    pub(super) direct_started: u64,
    pub(super) network_blocks: u64,
    pub(super) network_acknowledged: Vec<bool>,
    pub(super) network_sent: u64,
}

impl PathVerificationPlan {
    fn direct_class(&self, direct_position: u64) -> BlockClass {
        if direct_position < self.direct_confirmed {
            BlockClass::Confirmed
        } else if direct_position < self.direct_started {
            BlockClass::InFlight
        } else {
            BlockClass::NotStarted
        }
    }

    fn network_class(&self, network_position: u64) -> BlockClass {
        let position_as_index = usize::try_from(network_position).expect("块序号装得进 usize");
        if self.network_acknowledged.get(position_as_index).copied().unwrap_or(false) {
            BlockClass::Confirmed
        } else if network_position < self.network_sent {
            BlockClass::InFlight
        } else {
            BlockClass::NotStarted
        }
    }

    pub(super) fn to_arguments(&self, candidate: Candidate, library: &Path) -> Vec<String> {
        vec![
            "verify".to_string(),
            "--arm".to_string(),
            candidate.label().to_string(),
            "--library".to_string(),
            library.display().to_string(),
            "--states-per-block".to_string(),
            self.states_per_block.to_string(),
            "--direct-blocks".to_string(),
            self.direct_blocks.to_string(),
            "--direct-confirmed".to_string(),
            self.direct_confirmed.to_string(),
            "--direct-started".to_string(),
            self.direct_started.to_string(),
            "--network-blocks".to_string(),
            self.network_blocks.to_string(),
            "--network-acknowledged".to_string(),
            compress_true_positions(&self.network_acknowledged),
            "--network-sent".to_string(),
            self.network_sent.to_string(),
        ]
    }

    pub(super) fn parse(values: &HashMap<String, String>) -> Result<PathVerificationPlan, String> {
        let network_blocks = required_number(values, "--network-blocks")?;
        let acknowledged_text = values.get("--network-acknowledged").ok_or("缺 --network-acknowledged")?;
        Ok(PathVerificationPlan {
            states_per_block: required_number(values, "--states-per-block")?,
            direct_blocks: required_number(values, "--direct-blocks")?,
            direct_confirmed: required_number(values, "--direct-confirmed")?,
            direct_started: required_number(values, "--direct-started")?,
            network_blocks,
            network_acknowledged: expand_true_positions(acknowledged_text, network_blocks).ok_or("确认区间写坏了")?,
            network_sent: required_number(values, "--network-sent")?,
        })
    }
}

/// `compress_true_positions` 的反过来：给出总数，还原成布尔串；写坏了返回 None。
pub(super) fn expand_true_positions(text: &str, total: u64) -> Option<Vec<bool>> {
    let mut flags = vec![false; usize::try_from(total).ok()?];
    if text == "none" {
        return Some(flags);
    }
    for range in text.split(',') {
        let (start_text, end_text) = range.split_once('-')?;
        let start: usize = start_text.parse().ok()?;
        let end: usize = end_text.parse().ok()?;
        if start > end || end >= flags.len() {
            return None;
        }
        for flag in &mut flags[start..=end] {
            *flag = true;
        }
    }
    Some(flags)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct VerificationReport {
    pub(super) open_failed: bool,
    pub(super) open_error: String,
    /// 只有 K：第一次打开失败之后调 DB::repair 再开，能开就是 Some(true)。
    pub(super) opened_after_repair: Option<bool>,
    pub(super) open_microseconds: u64,
    pub(super) confirmed_checked: u64,
    pub(super) in_flight_checked: u64,
    pub(super) not_started_checked: u64,
    pub(super) lost_confirmed: Vec<u64>,
    pub(super) silently_corrupted: Vec<u64>,
    pub(super) confirmed_read_errors: Vec<u64>,
    pub(super) unconfirmed_read_errors: Vec<u64>,
    pub(super) phantom_keys: BTreeSet<String>,
    pub(super) enumerated_key_count: u64,
    pub(super) enumerated_value_bytes: u64,
    pub(super) enumeration_error: String,
    pub(super) first_read_error: String,
    /// 核对子进程超时或没交回结果行。
    pub(super) verification_missing: bool,
}

fn verify_library(candidate: Candidate, library: &Path, plan: &PathVerificationPlan) -> VerificationReport {
    let mut report = VerificationReport::default();
    let open_started = Instant::now();
    let opened = open_existing_store(candidate, library);
    report.open_microseconds = u64::try_from(open_started.elapsed().as_micros()).expect("微秒装得进 u64");
    let store = match opened {
        Ok(store) => store,
        Err(open_error) => {
            report.open_failed = true;
            report.open_error = open_error.to_string();
            match candidate {
                Candidate::RocksDatabase => {
                    let repaired = rocksdb::DB::repair(&rocksdb_options(), library)
                        .map_err(rocksdb_error)
                        .and_then(|()| open_existing_store(candidate, library));
                    report.opened_after_repair = Some(repaired.is_ok());
                    match repaired {
                        Ok(store) => store,
                        Err(_still_broken) => return report,
                    }
                }
                Candidate::RedbQuickRepair | Candidate::RedbDefault | Candidate::HardenedFile => return report,
            }
        }
    };
    let fingerprint = input_fingerprint();
    let mut checked: Vec<(u64, BlockClass)> = Vec::new();
    for direct_position in 0..plan.direct_blocks {
        checked.push((2 * direct_position, plan.direct_class(direct_position)));
    }
    for network_position in 0..plan.network_blocks {
        checked.push((network_path_block_index(network_position), plan.network_class(network_position)));
    }
    for (block_index, class) in checked {
        match class {
            BlockClass::Confirmed => report.confirmed_checked += 1,
            BlockClass::InFlight => report.in_flight_checked += 1,
            BlockClass::NotStarted => report.not_started_checked += 1,
        }
        let key = interleaved_block_key(block_index, plan.states_per_block, &fingerprint);
        let read = match store.read_block(&key) {
            ReadOutcome::Found(found) => {
                let expected = block_value(ValuePattern::Random, SEED_NETWORK_BLOCKS, block_index, plan.states_per_block);
                if found.len() == expected.len() && found == expected {
                    ReadKind::MatchingBytes
                } else {
                    ReadKind::DifferentBytes
                }
            }
            ReadOutcome::NotFound => ReadKind::NotFound,
            ReadOutcome::ReadError(reason) => {
                if report.first_read_error.is_empty() {
                    report.first_read_error = format!("block={block_index}:{reason}");
                }
                ReadKind::ReadError
            }
        };
        match classify_block(class, read) {
            BlockVerdict::Correct => {}
            BlockVerdict::LostConfirmed => report.lost_confirmed.push(block_index),
            BlockVerdict::SilentlyCorrupted => report.silently_corrupted.push(block_index),
            BlockVerdict::ConfirmedReadError => report.confirmed_read_errors.push(block_index),
            BlockVerdict::UnconfirmedReadError => report.unconfirmed_read_errors.push(block_index),
            BlockVerdict::Phantom => {
                report.phantom_keys.insert(hexadecimal(&key));
            }
        }
    }
    let started_keys: HashSet<BlockKey> = (0..plan.direct_started)
        .map(|direct_position| interleaved_block_key(2 * direct_position, plan.states_per_block, &fingerprint))
        .chain((0..plan.network_sent).map(|network_position| {
            interleaved_block_key(network_path_block_index(network_position), plan.states_per_block, &fingerprint)
        }))
        .collect();
    match store.enumerate_keys_with_value_lengths() {
        Ok(entries) => {
            for (key, value_length) in entries {
                report.enumerated_key_count += 1;
                report.enumerated_value_bytes += value_length;
                let is_started = <[u8; BLOCK_KEY_BYTES]>::try_from(key.as_slice()).is_ok_and(|fixed_key| started_keys.contains(&fixed_key));
                if !is_started {
                    report.phantom_keys.insert(hexadecimal(&key));
                }
            }
        }
        Err(error) => report.enumeration_error = error.to_string(),
    }
    report
}

fn first_numbers(numbers: &[u64]) -> String {
    if numbers.is_empty() {
        "none".to_string()
    } else {
        numbers.iter().take(10).map(u64::to_string).collect::<Vec<_>>().join(",")
    }
}

impl VerificationReport {
    pub(super) fn encode(&self) -> String {
        format!(
            "V open_failed={} open_error={} opened_after_repair={} open_microseconds={} confirmed_checked={} in_flight_checked={} not_started_checked={} lost_confirmed={} lost_confirmed_first={} silently_corrupted={} silently_corrupted_first={} confirmed_read_errors={} unconfirmed_read_errors={} read_errors_first={} phantom_keys={} enumerated_keys={} enumerated_value_bytes={} enumeration_error={} first_read_error={}",
            self.open_failed,
            text_field(&self.open_error),
            self.opened_after_repair.map_or("not_applicable".to_string(), |opened| opened.to_string()),
            self.open_microseconds,
            self.confirmed_checked,
            self.in_flight_checked,
            self.not_started_checked,
            self.lost_confirmed.len(),
            first_numbers(&self.lost_confirmed),
            self.silently_corrupted.len(),
            first_numbers(&self.silently_corrupted),
            self.confirmed_read_errors.len(),
            self.unconfirmed_read_errors.len(),
            first_numbers(&self.confirmed_read_errors.iter().chain(self.unconfirmed_read_errors.iter()).copied().collect::<Vec<_>>()),
            self.phantom_keys.len(),
            self.enumerated_key_count,
            self.enumerated_value_bytes,
            text_field(&self.enumeration_error),
            text_field(&self.first_read_error),
        )
    }
}

pub(super) fn run_verify_role(arguments: &[String]) -> i32 {
    let values = match parse_flag_values(arguments) {
        Ok(values) => values,
        Err(problem) => {
            eprintln!("  ✗ verify 的参数不对：{problem}");
            return 2;
        }
    };
    let Some(candidate) = values.get("--arm").and_then(|text| Candidate::parse(text)) else {
        eprintln!("  ✗ verify 缺 --arm 或认不出");
        return 2;
    };
    let Some(library) = values.get("--library").map(PathBuf::from) else {
        eprintln!("  ✗ verify 缺 --library");
        return 2;
    };
    let plan = match PathVerificationPlan::parse(&values) {
        Ok(plan) => plan,
        Err(problem) => {
            eprintln!("  ✗ verify 的计划写坏了：{problem}");
            return 2;
        }
    };
    let report = verify_library(candidate, &library, &plan);
    print_flushed(&report.encode());
    0
}

#[cfg(test)]
#[path = "block_verification_tests.rs"]
mod tests;
