//! 计数（落后类的列与 `Tally`）与单状态重放、逐态判定行（登记修订第 12 条）。

use super::*;

/// 落后类别各自数的量（登记 r2 SQ9、SQ17：状态数与 J1、J2、J4、J6、J6Qn、J7 的违例数，各行照原样分开）。
/// r5 档另记 J8 五格（登记 r5 5.3：J7 五行换成 J8 五行）；产物只打这一档判的那几列（[`lagging_class_columns_shown_under`]）。
const LAGGING_CLASS_COLUMNS: [Counter; 18] = [
    Counter::States,
    Counter::ReturnedRootViolations,
    Counter::RootPlusDiskViolations,
    Counter::FarthestVersionViolations,
    Counter::FixedPointPremiseHoldsViolations,
    Counter::FixedPointPremiseFailsViolations,
    Counter::FixedPointQnPremiseHoldsViolations,
    Counter::FixedPointQnPremiseFailsViolations,
    Counter::J7aInstanceTableViolations,
    Counter::J7aAllocationRecordTreeRootViolations,
    Counter::J7aFilesystemIdentifierViolations,
    Counter::J7bViolations,
    Counter::J7cViolations,
    Counter::J8aInstanceTableViolations,
    Counter::J8aAllocationRecordTreeRootViolations,
    Counter::J8aUnmountMarkerViolations,
    Counter::J8bViolations,
    Counter::J8cViolations,
];

/// 落后类别的行里打哪几列：J7 五列只在判 J7 的档（r2、r3）打，J8 五列只在 r5 档打，其余列各档都打
/// （r2 档的落后类别行因此与加 J8 之前逐字节相同）。
pub(super) fn lagging_class_columns_shown_under(run_profile: RunProfile) -> Vec<Counter> {
    LAGGING_CLASS_COLUMNS
        .iter()
        .copied()
        .filter(|counter| {
            let is_a_copied_fields_column = matches!(
                counter,
                Counter::J7aInstanceTableViolations
                    | Counter::J7aAllocationRecordTreeRootViolations
                    | Counter::J7aFilesystemIdentifierViolations
                    | Counter::J7bViolations
                    | Counter::J7cViolations
            );
            let is_a_new_root_segment_fields_column = matches!(
                counter,
                Counter::J8aInstanceTableViolations
                    | Counter::J8aAllocationRecordTreeRootViolations
                    | Counter::J8aUnmountMarkerViolations
                    | Counter::J8bViolations
                    | Counter::J8cViolations
            );
            (!is_a_copied_fields_column || run_profile.judges_copied_fields())
                && (!is_a_new_root_segment_fields_column
                    || run_profile.judges_new_root_segment_fields())
        })
        .collect()
}

/// 两块盘的落盘集合（按记录下标置位）：相等；0 号盘的真包含于 1 号盘的是 0 号盘落后；反过来是 1 号盘落后；其余互不包含。
pub(super) fn lagging_class_of(first_device_mask: u32, second_device_mask: u32) -> LaggingClass {
    if first_device_mask == second_device_mask {
        LaggingClass::Equal
    } else if first_device_mask & second_device_mask == first_device_mask {
        LaggingClass::FirstDeviceBehind
    } else if first_device_mask & second_device_mask == second_device_mask {
        LaggingClass::SecondDeviceBehind
    } else {
        LaggingClass::Crossed
    }
}

#[derive(Clone, Debug)]
pub(super) struct Tally {
    pub(super) counts: Vec<u64>,
    pub(super) witnesses: Vec<Option<String>>,
    pair_counts: Vec<u64>,
    pub(super) above_water_maximum: usize,
    /// 下标 = 落后类别的下标 × [`LAGGING_CLASS_COLUMNS`] 的列数 + 列下标；单盘域恒为 0。
    lagging_class_counts: Vec<u64>,
}

impl Tally {
    pub(super) fn new() -> Self {
        Self {
            counts: vec![0; Counter::ALL.len()],
            witnesses: vec![None; Counter::ALL.len()],
            pair_counts: vec![0; PairSet::ALL.len() * PairColumn::ALL.len()],
            above_water_maximum: 0,
            lagging_class_counts: vec![0; LaggingClass::ALL.len() * LAGGING_CLASS_COLUMNS.len()],
        }
    }

    /// 一个多盘状态判完：把它让 [`LAGGING_CLASS_COLUMNS`] 各加的数记进它的落后类别（`counts_before` 是判这个状态之前的计数）。
    pub(super) fn add_to_the_lagging_class(&mut self, class: LaggingClass, counts_before: &[u64]) {
        for (column_index, counter) in LAGGING_CLASS_COLUMNS.iter().enumerate() {
            let added = self.counts[counter.index()] - counts_before[counter.index()];
            self.lagging_class_counts
                [class.index() * LAGGING_CLASS_COLUMNS.len() + column_index] += added;
        }
    }

    pub(super) fn lagging_class_count(&self, class: LaggingClass, counter: Counter) -> u64 {
        let column_index = LAGGING_CLASS_COLUMNS
            .iter()
            .position(|column| *column == counter)
            .expect("只问 LAGGING_CLASS_COLUMNS 里列了的量");
        self.lagging_class_counts[class.index() * LAGGING_CLASS_COLUMNS.len() + column_index]
    }

    pub(super) fn add(&mut self, counter: Counter) {
        self.counts[counter.index()] += 1;
    }

    /// 违例加一；这个量的第一条违例记一个见证（枚举次序里最先的那个，合并时按工作单元的次序保留最先的）。
    pub(super) fn add_violation(
        &mut self,
        counter: Counter,
        describe_witness: &dyn Fn() -> String,
    ) {
        let index = counter.index();
        self.counts[index] += 1;
        if self.witnesses[index].is_none() {
            self.witnesses[index] = Some(describe_witness());
        }
    }

    pub(super) fn count_of(&self, counter: Counter) -> u64 {
        self.counts[counter.index()]
    }

    pub(super) fn witness(&self, counter: Counter) -> Option<&str> {
        self.witnesses[counter.index()].as_deref()
    }

    pub(super) fn add_pair(&mut self, set: PairSet, column: PairColumn) {
        self.pair_counts[set.index() * PairColumn::ALL.len() + column.index()] += 1;
    }

    pub(super) fn pair(&self, set: PairSet, column: PairColumn) -> u64 {
        self.pair_counts[set.index() * PairColumn::ALL.len() + column.index()]
    }

    /// 把后一个工作单元的计数并进来：数相加，见证保留先到的，最大值取大。
    pub(super) fn merge(&mut self, later: &Self) {
        for (count, later_count) in self.counts.iter_mut().zip(&later.counts) {
            *count += later_count;
        }
        for (count, later_count) in self.pair_counts.iter_mut().zip(&later.pair_counts) {
            *count += later_count;
        }
        for (count, later_count) in self
            .lagging_class_counts
            .iter_mut()
            .zip(&later.lagging_class_counts)
        {
            *count += later_count;
        }
        for (witness, later_witness) in self.witnesses.iter_mut().zip(&later.witnesses) {
            if witness.is_none() {
                witness.clone_from(later_witness);
            }
        }
        self.above_water_maximum = self.above_water_maximum.max(later.above_water_maximum);
    }
}

// ── 单状态重放与逐态判定行（登记修订第 12 条）：只挑哪些状态要判、把判出来的量打出来，不改任何判据

/// 这一次跑只看哪些状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StateProbe {
    /// 只判枚举次序里序号等于它的那一个状态，别的状态跳过不判；打出那一行就收尾退出。
    ReplayOneState { ordinal: u64 },
    /// 每个状态都判，各打一行。
    ListEveryState,
    /// 只判序号等于它的那一个状态，别的状态跳过不判；那一行记进 [`CAPTURED_STATE_ROW`]，不打、不退（钉住的红例逐行重放用）。
    CaptureOneStateRow { ordinal: u64 },
}

/// 探针开没开：正常跑时每个状态只读这一个原子量，不碰 [`STATE_PROBE`] 那把锁。
static STATE_PROBE_IS_SET: AtomicBool = AtomicBool::new(false);
/// 探针要的 (域, N) 与模式；没设就是正常跑，两个判定函数里的探针什么都不做。钉住的红例逐行重放时每一行换一次。
static STATE_PROBE: Mutex<Option<(Domain, usize, StateProbe)>> = Mutex::new(None);
/// 探针模式下已经走过的状态数：单线程照工作单元的次序走，它就是状态在枚举次序里的序号。
pub(super) static NEXT_STATE_ORDINAL: AtomicU64 = AtomicU64::new(0);
/// 探针模式下打出去的 `name=state` 行数。
static STATE_ROWS_EMITTED: AtomicU64 = AtomicU64::new(0);
/// `CaptureOneStateRow` 记下的那一行（不带换行）；设探针时清空。
pub(super) static CAPTURED_STATE_ROW: Mutex<Option<String>> = Mutex::new(None);

/// 设探针：(域, N) 与模式换成新的，走过的序号、打出去的行数、记下的那一行都从头算。
pub(super) fn set_the_state_probe(domain: Domain, size: usize, probe: StateProbe) {
    *STATE_PROBE.lock().expect("拿着探针那把锁的地方都不 panic") = Some((domain, size, probe));
    NEXT_STATE_ORDINAL.store(0, Ordering::Relaxed);
    STATE_ROWS_EMITTED.store(0, Ordering::Relaxed);
    *CAPTURED_STATE_ROW
        .lock()
        .expect("拿着记下那一行的锁的地方都不 panic") = None;
    STATE_PROBE_IS_SET.store(true, Ordering::Release);
}

/// 此刻的探针；没设交 `None`。
fn current_state_probe() -> Option<(Domain, usize, StateProbe)> {
    if !STATE_PROBE_IS_SET.load(Ordering::Acquire) {
        return None;
    }
    *STATE_PROBE.lock().expect("拿着探针那把锁的地方都不 panic")
}

/// 一个状态开判之前探针的决定。
pub(super) enum ProbeDecision {
    NotProbing,
    SkipThisState,
    ReportThisState {
        ordinal: u64,
        counts_before: Vec<u64>,
    },
}

pub(super) fn begin_probing_a_state(tally: &Tally) -> ProbeDecision {
    let Some((_domain, _size, probe)) = current_state_probe() else {
        return ProbeDecision::NotProbing;
    };
    let ordinal = NEXT_STATE_ORDINAL.fetch_add(1, Ordering::Relaxed);
    match probe {
        StateProbe::ReplayOneState { ordinal: wanted }
        | StateProbe::CaptureOneStateRow { ordinal: wanted }
            if wanted != ordinal =>
        {
            ProbeDecision::SkipThisState
        }
        StateProbe::ReplayOneState { .. }
        | StateProbe::ListEveryState
        | StateProbe::CaptureOneStateRow { .. } => ProbeDecision::ReportThisState {
            ordinal,
            counts_before: tally.counts.clone(),
        },
    }
}

/// 一个状态让哪几个量加了一：照 [`Counter::ALL`] 的次序列名字，加了不止一的写 `名*次数`；一个都没加写 `none`。
pub(super) fn labels_of_the_counters_a_state_added(
    counts_before: &[u64],
    counts_after: &[u64],
) -> String {
    let labels: Vec<String> = Counter::ALL
        .iter()
        .zip(counts_before.iter().zip(counts_after))
        .filter_map(|(counter, (before, after))| match after - before {
            0 => None,
            1 => Some(counter.label().to_string()),
            added => Some(format!("{}*{added}", counter.label())),
        })
        .collect();
    if labels.is_empty() {
        "none".to_string()
    } else {
        labels.join(",")
    }
}

/// 一个状态判完之后：探针要报它的，打一行 `name=state`（记下模式只记不打）。重放模式下报完这一个就收尾退出。
pub(super) fn finish_probing_a_state(
    decision: ProbeDecision,
    tally: &Tally,
    describe_state: &dyn Fn() -> String,
) {
    let ProbeDecision::ReportThisState {
        ordinal,
        counts_before,
    } = decision
    else {
        return;
    };
    let (domain, size, probe) = current_state_probe().expect("探针报一个状态时探针必然设过");
    let state_row = format!(
        "E7RESULT name=state domain={} ring_slots={} n={size} ordinal={ordinal} counters={} state={}",
        domain.label(),
        domain.ring().ring_slots,
        labels_of_the_counters_a_state_added(&counts_before, &tally.counts),
        describe_state()
    );
    STATE_ROWS_EMITTED.fetch_add(1, Ordering::Relaxed);
    match probe {
        StateProbe::ReplayOneState { .. } => {
            println!("{state_row}");
            // 配置行一行、状态行一行、收尾行一行。
            println!("E7RESULT name=done emitted=3");
            std::process::exit(0);
        }
        StateProbe::ListEveryState => println!("{state_row}"),
        StateProbe::CaptureOneStateRow { .. } => {
            let earlier_row = CAPTURED_STATE_ROW
                .lock()
                .expect("拿着记下那一行的锁的地方都不 panic")
                .replace(state_row);
            assert!(
                earlier_row.is_none(),
                "一个序号只对应枚举次序里的一个状态，记下模式一次设探针只记一行"
            );
        }
    }
}

/// 域名与 N 的原文 → (域, N)：域名用产物里 `domain=` 的写法，N 要在这个域最小的 N 到 12 之间。
pub(super) fn domain_and_size_of(
    domain_text: &str,
    size_text: &str,
) -> Result<(Domain, usize), String> {
    let domain = Domain::ALL
        .iter()
        .copied()
        .find(|domain| domain.label() == domain_text)
        .ok_or_else(|| format!("不认识的域：{domain_text}"))?;
    let size = size_text
        .parse::<usize>()
        .map_err(|error| format!("N 要一个非负整数：{error}"))?;
    if size < domain.smallest_size() || size > 12 {
        return Err(format!(
            "{} 域的 N 要在 {} 到 12 之间，收到的是 {size}",
            domain.label(),
            domain.smallest_size()
        ));
    }
    Ok((domain, size))
}

/// `<域>:<N>:<序号>` 或 `<域>:<N>`：域名用产物里 `domain=` 的写法（U 域是 `U_verify_on`、`U_verify_off`）。
pub(super) fn parse_the_state_probe(
    arguments: &[String],
) -> Result<Option<(Domain, usize, StateProbe)>, String> {
    let [option, value] = arguments else {
        return Ok(None);
    };
    let wants_one_state = match option.as_str() {
        "--replay-state" => true,
        "--list-states" => false,
        _other_option => return Ok(None),
    };
    let parts: Vec<&str> = value.split(':').collect();
    let expected_parts = if wants_one_state { 3 } else { 2 };
    if parts.len() != expected_parts {
        return Err(format!(
            "{option} 要 {expected_parts} 段（冒号分隔），收到的是「{value}」"
        ));
    }
    let (domain, size) = domain_and_size_of(parts[0], parts[1])?;
    let probe = if wants_one_state {
        StateProbe::ReplayOneState {
            ordinal: parts[2]
                .parse::<u64>()
                .map_err(|error| format!("序号要一个非负整数：{error}"))?,
        }
    } else {
        StateProbe::ListEveryState
    };
    Ok(Some((domain, size, probe)))
}

/// 探针模式的主流程：不跑自检与阳性对照，单线程把这个 (域, N) 照工作单元的次序走一遍。
pub(super) fn run_the_state_probe(
    domain: Domain,
    size: usize,
    probe: StateProbe,
    run_profile: RunProfile,
) {
    install_the_panic_hook_that_stays_quiet_inside_the_replay_call();
    println!(
        "E7RESULT name=config experiment=E165 mode={} domain={} ring_slots={} n={size} self_checks=not_run positive_controls=not_run threads=1{}",
        match probe {
            StateProbe::ReplayOneState { .. } => "replay_state",
            StateProbe::ListEveryState => "list_states",
            StateProbe::CaptureOneStateRow { .. } => "capture_state",
        },
        domain.label(),
        domain.ring().ring_slots,
        profile_fields_of_the_run_rows(run_profile)
    );
    set_the_state_probe(domain, size, probe);
    let tally = tally_of_a_domain_size(
        domain,
        size,
        Implementation::TodayReplayJournal,
        run_profile,
        1,
    );
    let walked = NEXT_STATE_ORDINAL.load(Ordering::Relaxed);
    let captured_state_row = CAPTURED_STATE_ROW
        .lock()
        .expect("拿着记下那一行的锁的地方都不 panic")
        .take();
    match (probe, captured_state_row) {
        (StateProbe::CaptureOneStateRow { .. }, Some(state_row)) => {
            // 记下模式走完整个 (域, N) 才回到这里；与重放模式打同样的三行。
            println!("{state_row}");
            println!("E7RESULT name=done emitted=3");
        }
        (
            StateProbe::ReplayOneState { ordinal } | StateProbe::CaptureOneStateRow { ordinal },
            None,
        ) => {
            // 走到这里说明那个序号一直没遇到（重放模式遇到的那一刻已经退出）。
            eprintln!(
                "  ✗ {} 域 N = {size} 只有 {walked} 个状态，序号 {ordinal} 不在里面",
                domain.label()
            );
            eprintln!(
                "  → 怎么办：序号从 0 数；用 --list-states {}:{size} 看这个 (域, N) 的全部状态",
                domain.label()
            );
            std::process::exit(2);
        }
        (StateProbe::ReplayOneState { .. } | StateProbe::ListEveryState, Some(_state_row)) => {
            unreachable!("只有记下模式往 CAPTURED_STATE_ROW 里放行，设探针时清空过")
        }
        (StateProbe::ListEveryState, None) => {
            let rows = STATE_ROWS_EMITTED.load(Ordering::Relaxed);
            assert_eq!(rows, walked, "每个状态恰好一行");
            assert_eq!(
                tally.count_of(Counter::States),
                walked,
                "探针走过的状态数等于这个 (域, N) 数出来的状态数"
            );
            println!("E7RESULT name=done emitted={}", rows + 2);
        }
    }
}

#[cfg(test)]
mod tests;
