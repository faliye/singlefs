//! 崩溃全量积压登记表（`crates/singlefs-checker-tier/crash-full-run-backlog.tsv`，判法在 `singlefs_checker_tier::crash_full_run_backlog`）的
//! 读写小程序：崩溃点重放入口与门禁格经它核表、取件、写回，不各写一份表的读法。路径都由命令行给，不认仓根与当前目录。
//!
//! 用法（选项一律 `--<名> <值>`，每个只给一次）：
//! - `crash_full_run_backlog check --table <表> --merged-items <合表> --amplification-items <ITEMS 列表> [--previous-table <上一版的表>]`：
//!   表对着合表（`.claude/gate.d/crash-point-replay-items.tsv`）与 ITEMS 列表（`bash scripts/crash/crash-amplification.sh --list-items` 的输出存成的文件）
//!   展开出来的件核（少登、多登、重登、次序、序号、全绿要有态数、跑过要有判法键）；给了上一版就再核上一版的件次序不变、新件追加在末尾。
//!   每处问题一行 `CRASH_FULL_RUN_BACKLOG_PROBLEM <TAB> <句子>`，末了一行
//!   `CRASH_FULL_RUN_BACKLOG_CHECK <TAB> rows=<行数> <TAB> expansion_parts=<展开的件数> <TAB> problems=<问题数>`。
//! - `crash_full_run_backlog today-judging-key`：先打 `CRASH_FULL_RUN_BACKLOG_TODAYS_JUDGING_KEY_GROUPS <TAB> groups=<并进的组数> <TAB> not_compiled=<其中不编进来的组数>`，
//!   再打 `CRASH_FULL_RUN_BACKLOG_TODAYS_JUDGING_KEY <TAB> key=<64 位小写十六进制>`：判法单元入口登记表登记的全部组（镜像判定、给定状态、
//!   每一个观察者与别的几组）在这一份二进制的判器键表里的合判法键（`crash_full_run_backlog::combined_judging_key_of_the_groups`）；
//!   键里有构建档、特性与工具链，要与全量那一趟同一种编法（`cargo run --release`，特性同那一趟）取，不然对不上。有一组拒算、或登记了而表里没有，退 1。
//! - `crash_full_run_backlog plan --table <表> --today-judging-key <64 位十六进制> --current-milestone <m4 这类> [--backlog-states-at-least <态数>] [--states-at-least <态数>]`：
//!   游标一行 `CRASH_FULL_RUN_BACKLOG_CURSOR <TAB> sequence=<序号或 none>`；取到的件照跑的次序每件一行
//!   `CRASH_FULL_RUN_BACKLOG_TAKE <TAB> group=<new-or-judging-key-changed 或 backlog> <TAB> sequence=<序号> <TAB> item=<项名> <TAB> part=<部分> <TAB> key=<键> <TAB> states=<态数或 not-counted>`；
//!   末了一行 `CRASH_FULL_RUN_BACKLOG_PLAN <TAB> new_or_judging_key_changed_states=<态数> <TAB> backlog_states=<态数> <TAB> taken_without_a_count=<件数> <TAB> lower_bounds=<both 或 every-uncovered-row-below-the-bounds>`。
//!   两条下限不给取出口 10 的 50 亿、100 亿。
//! - `crash_full_run_backlog record --table <表> --sequence <序号> --judging-key <判法键格> --progress <进度格> --last-run <最后一次格> [--states <态数格>]`：
//!   把跑完一件的结果写进那一行（格的写法同表里；先写同目录的一份再换上），打 `CRASH_FULL_RUN_BACKLOG_RECORDED <TAB> sequence=<序号>`。
//! - `crash_full_run_backlog append-missing --table <表> --merged-items <合表> --amplification-items <ITEMS 列表> --registered-in <m4 这类> --provenance <态数的出处>`：
//!   展开里有、表里没有的件追加到末尾（未数、未记、未跑、无），打 `CRASH_FULL_RUN_BACKLOG_APPENDED <TAB> parts=<件数>`。
//!
//! 退出码：0 做完（`check` 没有问题）；1 `check` 有问题、`today-judging-key` 拒算；2 用法错、读不成、写不成。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use singlefs_checker_tier::crash_full_run_backlog::{
    amplification_item_names_of_the_list, append_the_parts_missing_from_the_table,
    backlog_of_the_table_text, backlog_table_text, parts_in_the_expansion_order,
    parts_to_run_under_todays_judging_key, problems_against_the_expansion,
    problems_against_the_previous_table, todays_judging_key_of_the_compiled_table,
    write_the_run_into_the_row, BacklogRow, CrashFullRunBacklog, CrashFullRunPart,
    JudgingKeyOfThePart, JudgingUnitsGroupKeyAtTheBuild, LastRunOfThePart, LowerBoundsReached,
    ProgressOfThePart, RegisteredInMilestone, RowAfterARun, RowSequence,
    SeparatelyRegisteredMilestone, StateLowerBounds, StatesOfThePart, TodaysJudgingKey,
};
use singlefs_checker_tier::crash_identity::compiled_judging_units_keys;
use singlefs_harness::hexadecimal::hexadecimal_text;

/// 一次调用读到的选项：名（不带 `--`）→ 值。
struct OptionsOfTheCall {
    values: BTreeMap<String, String>,
}

impl OptionsOfTheCall {
    /// `--<名> <值>` 成对读；`allowed` 之外的名、同一个名给两次、少了值都拒。
    fn of_the_arguments(arguments: &[String], allowed: &[&str]) -> Result<Self, String> {
        let mut values = BTreeMap::new();
        let mut remaining = arguments.iter();
        while let Some(flag) = remaining.next() {
            let Some(name) = flag.strip_prefix("--") else {
                return Err(format!("「{flag}」不是 --<名>"));
            };
            if !allowed.contains(&name) {
                return Err(format!(
                    "不认 --{name}（这个子命令认：{}）",
                    allowed.join("、")
                ));
            }
            let Some(value) = remaining.next() else {
                return Err(format!("--{name} 后面少了值"));
            };
            if values.insert(name.to_string(), value.clone()).is_some() {
                return Err(format!("--{name} 给了两次"));
            }
        }
        Ok(Self { values })
    }

    fn required(&self, name: &str) -> Result<&str, String> {
        self.values
            .get(name)
            .map(String::as_str)
            .ok_or_else(|| format!("少了 --{name}"))
    }

    fn optional(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(String::as_str)
    }
}

fn text_of_the_file(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| format!("读不了 {}：{error}", path.display()))
}

fn backlog_of_the_file(path: &Path) -> Result<CrashFullRunBacklog, String> {
    backlog_of_the_table_text(&text_of_the_file(path)?)
        .map_err(|unreadable| format!("{}：{}", path.display(), unreadable.sentence()))
}

fn expansion_of_the_files(
    merged_items: &Path,
    amplification_items: &Path,
) -> Result<Vec<CrashFullRunPart>, String> {
    let amplification_item_names =
        amplification_item_names_of_the_list(&text_of_the_file(amplification_items)?);
    parts_in_the_expansion_order(&text_of_the_file(merged_items)?, &amplification_item_names)
        .map_err(|unreadable| format!("{}：{}", merged_items.display(), unreadable.sentence()))
}

/// 先写同目录的一份、再换上（写到一半被打断，表还是原来那一份）。
fn write_the_backlog_file(path: &Path, backlog: &CrashFullRunBacklog) -> Result<(), String> {
    let mut writing = path.as_os_str().to_owned();
    writing.push(".writing");
    let writing = PathBuf::from(writing);
    std::fs::write(&writing, backlog_table_text(backlog))
        .map_err(|error| format!("写不了 {}：{error}", writing.display()))?;
    std::fs::rename(&writing, path).map_err(|error| {
        format!(
            "换不上 {}（{} 留着）：{error}",
            path.display(),
            writing.display()
        )
    })
}

fn decimal_option(options: &OptionsOfTheCall, name: &str, default: u64) -> Result<u64, String> {
    match options.optional(name) {
        None => Ok(default),
        Some(text) => text
            .parse::<u64>()
            .map_err(|_unreadable| format!("--{name} 「{text}」不是十进制整数")),
    }
}

/// `check`：交问题数（0 是没有问题）。
fn check_the_table(options: &OptionsOfTheCall) -> Result<usize, String> {
    let backlog = backlog_of_the_file(Path::new(options.required("table")?))?;
    let expansion = expansion_of_the_files(
        Path::new(options.required("merged-items")?),
        Path::new(options.required("amplification-items")?),
    )?;
    let mut problems = problems_against_the_expansion(&backlog, &expansion);
    if let Some(previous_table) = options.optional("previous-table") {
        let previous = backlog_of_the_file(Path::new(previous_table))?;
        problems.extend(problems_against_the_previous_table(&previous, &backlog));
    }
    for problem in &problems {
        println!("CRASH_FULL_RUN_BACKLOG_PROBLEM\t{}", problem.sentence());
    }
    println!(
        "CRASH_FULL_RUN_BACKLOG_CHECK\trows={}\texpansion_parts={}\tproblems={}",
        backlog.rows.len(),
        expansion.len(),
        problems.len()
    );
    Ok(problems.len())
}

fn row_of_the_sequence(backlog: &CrashFullRunBacklog, sequence: RowSequence) -> &BacklogRow {
    backlog
        .rows
        .iter()
        .find(|row| row.sequence == sequence)
        .expect("取件只交表里有的序号")
}

fn take_line(group: &str, row: &BacklogRow) -> String {
    let states = row
        .states
        .counted_states()
        .map_or_else(|| "not-counted".to_string(), |states| states.to_string());
    format!(
        "CRASH_FULL_RUN_BACKLOG_TAKE\tgroup={group}\tsequence={}\titem={}\tpart={}\tkey={}\tstates={states}",
        row.sequence.0,
        row.part.item_name,
        row.part.part.text(),
        row.part.key
    )
}

/// `plan`：游标与取到的件。
fn plan_the_run(options: &OptionsOfTheCall) -> Result<(), String> {
    let backlog = backlog_of_the_file(Path::new(options.required("table")?))?;
    let key_text = options.required("today-judging-key")?;
    let today = TodaysJudgingKey::of_the_hexadecimal_text(key_text)
        .ok_or_else(|| format!("--today-judging-key「{key_text}」不是 64 位小写十六进制"))?;
    let milestone_text = options.required("current-milestone")?;
    let current_milestone = SeparatelyRegisteredMilestone::of_the_text(milestone_text)
        .ok_or_else(|| format!("--current-milestone「{milestone_text}」不是 m<不小于 4 的号>"))?;
    let bounds = StateLowerBounds {
        backlog_states_at_least: decimal_option(
            options,
            "backlog-states-at-least",
            StateLowerBounds::OF_THE_MILESTONE_EXIT.backlog_states_at_least,
        )?,
        states_at_least: decimal_option(
            options,
            "states-at-least",
            StateLowerBounds::OF_THE_MILESTONE_EXIT.states_at_least,
        )?,
    };
    let selection =
        parts_to_run_under_todays_judging_key(&backlog, &today, current_milestone, bounds);
    println!(
        "CRASH_FULL_RUN_BACKLOG_CURSOR\tsequence={}",
        selection
            .cursor
            .map_or_else(|| "none".to_string(), |sequence| sequence.0.to_string())
    );
    for sequence in &selection.new_or_judging_key_changed {
        println!(
            "{}",
            take_line(
                "new-or-judging-key-changed",
                row_of_the_sequence(&backlog, *sequence)
            )
        );
    }
    for sequence in &selection.backlog_from_the_cursor {
        println!(
            "{}",
            take_line("backlog", row_of_the_sequence(&backlog, *sequence))
        );
    }
    let lower_bounds = match selection.lower_bounds {
        LowerBoundsReached::BothBounds => "both",
        LowerBoundsReached::EveryUncoveredRowTakenBelowTheBounds => {
            "every-uncovered-row-below-the-bounds"
        }
    };
    println!(
        "CRASH_FULL_RUN_BACKLOG_PLAN\tnew_or_judging_key_changed_states={}\tbacklog_states={}\ttaken_without_a_count={}\tlower_bounds={lower_bounds}",
        selection.counted_states_of_the_new_or_judging_key_changed,
        selection.counted_states_of_the_backlog,
        selection.taken_without_a_count.len()
    );
    Ok(())
}

/// `record`：跑完一件写回。
fn record_the_run(options: &OptionsOfTheCall) -> Result<(), String> {
    let table = Path::new(options.required("table")?);
    let mut backlog = backlog_of_the_file(table)?;
    let sequence_text = options.required("sequence")?;
    let sequence = sequence_text
        .parse::<u64>()
        .map(RowSequence)
        .map_err(|_unreadable| format!("--sequence「{sequence_text}」不是十进制整数"))?;
    let run = RowAfterARun {
        states: options
            .optional("states")
            .map(StatesOfThePart::of_the_text)
            .transpose()?,
        judging_key: JudgingKeyOfThePart::of_the_text(options.required("judging-key")?)?,
        progress: ProgressOfThePart::of_the_text(options.required("progress")?)?,
        last_run: LastRunOfThePart::of_the_text(options.required("last-run")?)?,
    };
    write_the_run_into_the_row(&mut backlog, sequence, run)?;
    write_the_backlog_file(table, &backlog)?;
    println!("CRASH_FULL_RUN_BACKLOG_RECORDED\tsequence={}", sequence.0);
    Ok(())
}

/// `append-missing`：展开里有、表里没有的件追加到末尾。
fn append_the_missing_parts(options: &OptionsOfTheCall) -> Result<(), String> {
    let table = Path::new(options.required("table")?);
    let mut backlog = backlog_of_the_file(table)?;
    let expansion = expansion_of_the_files(
        Path::new(options.required("merged-items")?),
        Path::new(options.required("amplification-items")?),
    )?;
    let registered_text = options.required("registered-in")?;
    let registered_in = RegisteredInMilestone::of_the_text(registered_text)
        .ok_or_else(|| format!("--registered-in「{registered_text}」认不出"))?;
    let appended = append_the_parts_missing_from_the_table(
        &mut backlog,
        &expansion,
        registered_in,
        options.required("provenance")?,
    );
    write_the_backlog_file(table, &backlog)?;
    println!("CRASH_FULL_RUN_BACKLOG_APPENDED\tparts={appended}");
    Ok(())
}

/// `today-judging-key`：拒算交错。
fn print_todays_judging_key() -> Result<(), String> {
    let (key, groups) = todays_judging_key_of_the_compiled_table(compiled_judging_units_keys())
        .map_err(|refusal| refusal.sentence())?;
    let not_compiled = groups
        .iter()
        .filter(|group| group.key == JudgingUnitsGroupKeyAtTheBuild::NotCompiledInThisBuild)
        .count();
    println!(
        "CRASH_FULL_RUN_BACKLOG_TODAYS_JUDGING_KEY_GROUPS\tgroups={}\tnot_compiled={not_compiled}",
        groups.len()
    );
    println!(
        "CRASH_FULL_RUN_BACKLOG_TODAYS_JUDGING_KEY\tkey={}",
        hexadecimal_text(&key.0)
    );
    Ok(())
}

const USAGE: &str = "用法：crash_full_run_backlog check|today-judging-key|plan|record|append-missing --<名> <值>…（文件头逐个子命令写了选项）";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some((subcommand, rest)) = arguments.split_first() else {
        eprintln!("  ✗ 没给子命令。{USAGE}");
        return ExitCode::from(2);
    };
    let allowed: &[&str] = match subcommand.as_str() {
        "check" => &[
            "table",
            "merged-items",
            "amplification-items",
            "previous-table",
        ],
        "today-judging-key" => &[],
        "plan" => &[
            "table",
            "today-judging-key",
            "current-milestone",
            "backlog-states-at-least",
            "states-at-least",
        ],
        "record" => &[
            "table",
            "sequence",
            "judging-key",
            "progress",
            "last-run",
            "states",
        ],
        "append-missing" => &[
            "table",
            "merged-items",
            "amplification-items",
            "registered-in",
            "provenance",
        ],
        unknown => {
            eprintln!("  ✗ 不认子命令 {unknown}。{USAGE}");
            return ExitCode::from(2);
        }
    };
    let options = match OptionsOfTheCall::of_the_arguments(rest, allowed) {
        Ok(options) => options,
        Err(refusal) => {
            eprintln!("  ✗ {subcommand}：{refusal}。{USAGE}");
            return ExitCode::from(2);
        }
    };
    let outcome = match subcommand.as_str() {
        "check" => check_the_table(&options).map(|problems| {
            if problems == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }),
        "today-judging-key" => {
            return match print_todays_judging_key() {
                Ok(()) => ExitCode::SUCCESS,
                Err(refusal) => {
                    eprintln!("  ✗ {refusal}");
                    ExitCode::from(1)
                }
            };
        }
        "plan" => plan_the_run(&options).map(|()| ExitCode::SUCCESS),
        "record" => record_the_run(&options).map(|()| ExitCode::SUCCESS),
        "append-missing" => append_the_missing_parts(&options).map(|()| ExitCode::SUCCESS),
        unknown => unreachable!("子命令 {unknown} 在上面认选项时已经拒了"),
    };
    match outcome {
        Ok(exit_code) => exit_code,
        Err(refusal) => {
            eprintln!("  ✗ {subcommand}：{refusal}");
            ExitCode::from(2)
        }
    }
}
