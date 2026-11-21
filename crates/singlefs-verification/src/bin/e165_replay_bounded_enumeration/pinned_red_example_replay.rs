//! 钉住的红例逐行重放（`.claude/gate.d/e165-pinned-red-examples.tsv` 文件头的约定）：读表、按 ②③④ 找状态、重算 ①，与按工作单元合起来的计数。

use super::*;

// ── 钉住的红例逐行重放（`.claude/gate.d/e165-pinned-red-examples.tsv` 文件头的约定）：只按 ②③④ 找状态、重算 ①，不改任何判据

/// 钉住的表自己的段数（① 输入指纹、② 域、③ N、④ 序号、⑤ 出处、⑥ 变异、⑦ 原实现的逐态行、⑧ 变体的逐态行）。
const PINNED_TABLE_OWN_COLUMN_COUNT: usize = 8;

/// 红例账（`research/results/e165-*-red-examples.tsv`）一行的段数，列定义在账的表头。
const RED_EXAMPLE_LEDGER_COLUMN_COUNT: usize = 13;

/// 钉住的表一行有几段：自己的八段后面接红例账出红的那一行原样（⑨–㉑），⑨ 起各段用制表符接起来与账那一行逐字节相同。
const PINNED_TABLE_COLUMN_COUNT: usize =
    PINNED_TABLE_OWN_COLUMN_COUNT + RED_EXAMPLE_LEDGER_COLUMN_COUNT;

/// 钉住的表的段号写法，下标是段号减一。
const PINNED_TABLE_COLUMN_MARKS: [&str; PINNED_TABLE_COLUMN_COUNT] = [
    "①", "②", "③", "④", "⑤", "⑥", "⑦", "⑧", "⑨", "⑩", "⑪", "⑫", "⑬", "⑭", "⑮", "⑯", "⑰", "⑱", "⑲",
    "⑳", "㉑",
];

/// 抄来的账行里的一段与钉住的表自己的一段必须逐字相同：段号都从 1 数，账的段号按账自己数（钉表里是它加八）。
struct LedgerColumnThatMustEqualAPinnedColumn {
    ledger_column_number: usize,
    pinned_column_number: usize,
}

/// 按红例账表头的列定义：第 3、4、5 段是域、N、枚举次序里的序号，对 ②③④；第 12、13 段是原实现与变体的逐态行，对 ⑦⑧。
const LEDGER_COLUMNS_THAT_MUST_EQUAL_A_PINNED_COLUMN: [LedgerColumnThatMustEqualAPinnedColumn; 5] = [
    LedgerColumnThatMustEqualAPinnedColumn {
        ledger_column_number: 3,
        pinned_column_number: 2,
    },
    LedgerColumnThatMustEqualAPinnedColumn {
        ledger_column_number: 4,
        pinned_column_number: 3,
    },
    LedgerColumnThatMustEqualAPinnedColumn {
        ledger_column_number: 5,
        pinned_column_number: 4,
    },
    LedgerColumnThatMustEqualAPinnedColumn {
        ledger_column_number: 12,
        pinned_column_number: 7,
    },
    LedgerColumnThatMustEqualAPinnedColumn {
        ledger_column_number: 13,
        pinned_column_number: 8,
    },
];

/// 段数已经是 [`PINNED_TABLE_COLUMN_COUNT`] 的一行：抄来的账行（⑨ 起）逐对对着钉表自己的段核，不同的每一对都报出来、点名是哪一段。
fn check_that_the_copied_ledger_row_agrees_with_the_pinned_columns(
    columns: &[&str],
) -> Result<(), String> {
    assert_eq!(
        columns.len(),
        PINNED_TABLE_COLUMN_COUNT,
        "调用方 parse_a_pinned_red_example 先核过段数"
    );
    let disagreements: Vec<String> = LEDGER_COLUMNS_THAT_MUST_EQUAL_A_PINNED_COLUMN
        .iter()
        .filter_map(|pair| {
            let ledger_column_index_in_the_table =
                PINNED_TABLE_OWN_COLUMN_COUNT + pair.ledger_column_number - 1;
            let pinned_column_index = pair.pinned_column_number - 1;
            let ledger_value = columns[ledger_column_index_in_the_table];
            let pinned_value = columns[pinned_column_index];
            (ledger_value != pinned_value).then(|| {
                format!(
                    "{}（红例账第 {} 段）与 {} 不同：{} 是「{ledger_value}」，{} 是「{pinned_value}」",
                    PINNED_TABLE_COLUMN_MARKS[ledger_column_index_in_the_table],
                    pair.ledger_column_number,
                    PINNED_TABLE_COLUMN_MARKS[pinned_column_index],
                    PINNED_TABLE_COLUMN_MARKS[ledger_column_index_in_the_table],
                    PINNED_TABLE_COLUMN_MARKS[pinned_column_index],
                )
            })
        })
        .collect();
    if disagreements.is_empty() {
        Ok(())
    } else {
        Err(disagreements.join("；"))
    }
}

/// 钉住的表里的一行数据：① 与 ②③ 留原文（重算 ① 用原文），②③④ 另解出值来找状态。
#[derive(Clone, Debug, PartialEq, Eq)]
struct PinnedRedExample {
    line_number_in_the_table: usize,
    input_fingerprint: String,
    domain_text: String,
    size_text: String,
    domain: Domain,
    size: usize,
    ordinal: u64,
}

/// 表里一行数据 → [`PinnedRedExample`]：段数不是二十一（报要几段、这一行几段）、抄来的账行与 ②③④⑦⑧ 对不上、
/// 域不认识、N 不在这个域的范围、序号不是非负整数，都拒。
fn parse_a_pinned_red_example(
    line_number_in_the_table: usize,
    line: &str,
) -> Result<PinnedRedExample, String> {
    let columns: Vec<&str> = line.split('\t').collect();
    if columns.len() != PINNED_TABLE_COLUMN_COUNT {
        return Err(format!(
            "要 {PINNED_TABLE_COLUMN_COUNT} 段（制表符分隔：自己的 {PINNED_TABLE_OWN_COLUMN_COUNT} 段后面接红例账一行的 {RED_EXAMPLE_LEDGER_COLUMN_COUNT} 段），这一行是 {} 段",
            columns.len()
        ));
    }
    check_that_the_copied_ledger_row_agrees_with_the_pinned_columns(&columns)?;
    let (domain, size) = domain_and_size_of(columns[1], columns[2])?;
    let ordinal = columns[3]
        .parse::<u64>()
        .map_err(|error| format!("④ 序号要一个非负整数：{error}"))?;
    Ok(PinnedRedExample {
        line_number_in_the_table,
        input_fingerprint: columns[0].to_string(),
        domain_text: columns[1].to_string(),
        size_text: columns[2].to_string(),
        domain,
        size,
        ordinal,
    })
}

/// 输入指纹：`sha256(<域>|<N>|<逐态行里 state= 的值，截到 ;first= 之前>)` 的十六进制，与
/// `scripts/experiment-apparatus/e165-pin-red-examples.py` 的 `input_fingerprint` 同一个算法。逐态行里认不出 ` state=<不带空白的值>` 收尾交 `None`。
fn input_fingerprint_of(domain_text: &str, size_text: &str, state_row: &str) -> Option<String> {
    let (_row_before_the_state, state_text) = state_row.rsplit_once(" state=")?;
    if state_text.is_empty() || state_text.chars().any(char::is_whitespace) {
        return None;
    }
    let state_before_the_outcome = state_text
        .split_once(";first=")
        .map_or(state_text, |(before_the_outcome, _outcome)| {
            before_the_outcome
        });
    Some(sha256_hexadecimal(
        format!("{domain_text}|{size_text}|{state_before_the_outcome}").as_bytes(),
    ))
}

/// 一行钉住的红例在今天的装置上重放出来的样子。
#[derive(Clone, Debug, PartialEq, Eq)]
enum PinnedRedExampleReplay {
    /// 找到了状态，拿它的逐态行重算的指纹与 ① 相同。
    FingerprintMatches { state_row: String },
    /// 找到了状态，重算的指纹与 ① 不同：枚举次序或状态编码漂了。
    FingerprintDrifted {
        state_row: String,
        recomputed_fingerprint: String,
    },
    /// 找到了状态，逐态行里认不出 `state=` 那一段，算不出指纹。
    StateRowUnrecognized { state_row: String },
    /// 这个 (域, N) 走完了也没遇到这个序号。
    OrdinalOutsideTheDomainSize { states_walked: u64 },
}

/// 重放出来的逐态行（没遇到序号是 `None`）对着表里那一行判：纯函数，不碰探针。
fn judge_a_pinned_red_example_replay(
    example: &PinnedRedExample,
    captured_state_row: Option<String>,
    states_walked: u64,
) -> PinnedRedExampleReplay {
    let Some(state_row) = captured_state_row else {
        return PinnedRedExampleReplay::OrdinalOutsideTheDomainSize { states_walked };
    };
    match input_fingerprint_of(&example.domain_text, &example.size_text, &state_row) {
        None => PinnedRedExampleReplay::StateRowUnrecognized { state_row },
        Some(recomputed_fingerprint) if recomputed_fingerprint == example.input_fingerprint => {
            PinnedRedExampleReplay::FingerprintMatches { state_row }
        }
        Some(recomputed_fingerprint) => PinnedRedExampleReplay::FingerprintDrifted {
            state_row,
            recomputed_fingerprint,
        },
    }
}

/// 在今天的实现上重放一行：设记下模式的探针，单线程把这个 (域, N) 走一遍，交回记下的那一行与走过的状态数。
fn replay_a_pinned_red_example(
    example: &PinnedRedExample,
    run_profile: RunProfile,
) -> (Option<String>, u64) {
    set_the_state_probe(
        example.domain,
        example.size,
        StateProbe::CaptureOneStateRow {
            ordinal: example.ordinal,
        },
    );
    let _tally_of_the_one_judged_state = tally_of_a_domain_size(
        example.domain,
        example.size,
        Implementation::TodayReplayJournal,
        run_profile,
        1,
    );
    let states_walked = NEXT_STATE_ORDINAL.load(Ordering::Relaxed);
    let captured_state_row = CAPTURED_STATE_ROW
        .lock()
        .expect("拿着记下那一行的锁的地方都不 panic")
        .take();
    (captured_state_row, states_walked)
}

/// 钉住的表全文解出来的样子：数据行（非空、不以 `#` 起头）有几行，解得出的各行（按表里的次序），解不出的各行的问题（带行号）。
#[derive(Debug)]
struct PinnedTableParse {
    data_line_count: usize,
    examples: Vec<PinnedRedExample>,
    problems_of_the_rows_that_do_not_parse: Vec<String>,
}

/// 钉住的表全文 → [`PinnedTableParse`]：纯函数，不碰探针、不重放。
fn parse_the_pinned_table(table_text: &str) -> PinnedTableParse {
    let mut parse = PinnedTableParse {
        data_line_count: 0,
        examples: Vec::new(),
        problems_of_the_rows_that_do_not_parse: Vec::new(),
    };
    let data_lines = table_text
        .lines()
        .enumerate()
        .map(|(line_index, line)| (line_index + 1, line))
        .filter(|(_line_number, line)| !line.trim().is_empty() && !line.starts_with('#'));
    for (line_number, line) in data_lines {
        parse.data_line_count += 1;
        match parse_a_pinned_red_example(line_number, line) {
            Ok(example) => parse.examples.push(example),
            Err(problem) => parse
                .problems_of_the_rows_that_do_not_parse
                .push(format!("第 {line_number} 行：{problem}")),
        }
    }
    parse
}

/// `--pinned-cases <表>` 的主流程：按数据行的次序每行打一行 `name=state`；有行不对、找不到状态或指纹漂了，全部报完退 1。
pub(super) fn replay_every_pinned_red_example(table_path: &str, run_profile: RunProfile) {
    let table_text = match fs::read_to_string(table_path) {
        Ok(table_text) => table_text,
        Err(error) => {
            eprintln!("  ✗ 读不了钉住的表 {table_path}：{error}");
            eprintln!("  → 怎么办：{USAGE}");
            std::process::exit(2);
        }
    };
    install_the_panic_hook_that_stays_quiet_inside_the_replay_call();
    let PinnedTableParse {
        data_line_count,
        examples,
        problems_of_the_rows_that_do_not_parse,
    } = parse_the_pinned_table(&table_text);
    println!(
        "E7RESULT name=config experiment=E165 mode=pinned_cases rows={data_line_count} self_checks=not_run positive_controls=not_run threads=1"
    );
    let mut problems: Vec<String> = Vec::new();
    if data_line_count == 0 {
        problems.push(format!("{table_path} 一行数据都没有"));
    }
    problems.extend(problems_of_the_rows_that_do_not_parse);
    let mut state_rows_printed: u64 = 0;
    for example in examples {
        let (captured_state_row, states_walked) =
            replay_a_pinned_red_example(&example, run_profile);
        let place = format!(
            "第 {} 行（{}:{}:{}）",
            example.line_number_in_the_table,
            example.domain_text,
            example.size_text,
            example.ordinal
        );
        match judge_a_pinned_red_example_replay(&example, captured_state_row, states_walked) {
            PinnedRedExampleReplay::FingerprintMatches { state_row } => {
                println!("{state_row}");
                state_rows_printed += 1;
            }
            PinnedRedExampleReplay::FingerprintDrifted {
                state_row,
                recomputed_fingerprint,
            } => {
                println!("{state_row}");
                state_rows_printed += 1;
                problems.push(format!(
                    "{place}：拿这个状态重算的指纹是 {recomputed_fingerprint}，表里 ① 是 {}",
                    example.input_fingerprint
                ));
            }
            PinnedRedExampleReplay::StateRowUnrecognized { state_row } => {
                println!("{state_row}");
                state_rows_printed += 1;
                problems.push(format!("{place}：逐态行里认不出 state= 那一段"));
            }
            PinnedRedExampleReplay::OrdinalOutsideTheDomainSize { states_walked } => {
                problems.push(format!(
                    "{place}：这个 (域, N) 只有 {states_walked} 个状态，序号不在里面"
                ));
            }
        }
    }
    // 配置行一行、逐态行若干、收尾行一行。
    println!("E7RESULT name=done emitted={}", state_rows_printed + 2);
    if !problems.is_empty() {
        for problem in &problems {
            eprintln!("  ✗ {problem}");
        }
        eprintln!(
            "  → 怎么办：指纹漂了先查装置的枚举次序与状态编码变没变；行不对的照表头的二十一段格式（自己的八段后面接红例账出红的那一行原样）找回原行（表只由 scripts/experiment-apparatus/e165-pin-red-examples.py 写）"
        );
        std::process::exit(1);
    }
}

/// 把工作单元分给几条线程跑，再按工作单元的次序合并：计数是加法，见证取次序里最先的，结果与线程数、线程次序无关。
pub(super) fn tally_of_work_units<Unit: Sync>(
    units: &[Unit],
    thread_count: usize,
    work: &(dyn Fn(usize, &Unit) -> Tally + Sync),
) -> Tally {
    let next_unit_index = AtomicUsize::new(0);
    let finished_units: Mutex<Vec<(usize, Tally)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _worker in 0..thread_count.clamp(1, units.len().max(1)) {
            scope.spawn(|| loop {
                let unit_index = next_unit_index.fetch_add(1, Ordering::Relaxed);
                let Some(unit) = units.get(unit_index) else {
                    break;
                };
                let tally = work(unit_index, unit);
                finished_units
                    .lock()
                    .expect("别的线程没有在拿着这把锁的时候 panic")
                    .push((unit_index, tally));
            });
        }
    });
    let mut finished_units = finished_units
        .into_inner()
        .expect("线程都已经收回，没有谁在拿着这把锁的时候 panic");
    finished_units.sort_by_key(|(unit_index, _tally)| *unit_index);
    assert_eq!(finished_units.len(), units.len(), "每个工作单元恰好跑一遍");
    let mut total = Tally::new();
    for (_unit_index, tally) in &finished_units {
        total.merge(tally);
    }
    total
}

#[cfg(test)]
mod tests;
