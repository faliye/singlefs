//! 汇总：把结果文件里的历史行按族与臂加起来，查重复、冲突与没写完的行，写汇总文件。
use super::*;

/// 结果行里一个字段的值。
fn field_of<'line>(line: &'line str, name: &str) -> Option<&'line str> {
    let prefix = format!("{name}=");
    line.split(' ')
        .find_map(|field| field.strip_prefix(prefix.as_str()))
}

/// 不进汇总求和的字段：身份与标签、挂钟（挂钟另起一行）。
const FIELDS_NOT_SUMMED: [&str; 3] = ["index", "elapsed_ns", "label"];

/// 汇总（登记第五节「产物」的汇总表）：按（族、臂）收结果行，每个数值字段报合计与「大于 0 的段数」，`*_peak` 另报最大值；
/// 同一（族、臂、序号）出现两行时内容相同记重复、不同记冲突（登记第十一节「合并不确定」作废那一条）；没写完的行另数。
fn summarize(text: &str) -> Vec<String> {
    let mut groups: BTreeMap<(String, String), BTreeMap<u64, String>> = BTreeMap::new();
    let mut duplicates: BTreeMap<(String, String), u64> = BTreeMap::new();
    let mut conflicts: BTreeMap<(String, String), u64> = BTreeMap::new();
    let mut events: BTreeMap<(String, String, String), u64> = BTreeMap::new();
    let mut complete: BTreeSet<(String, String)> = BTreeSet::new();
    let mut unfinished_lines = 0_u64;
    for line in text.lines() {
        let family = field_of(line, "family").unwrap_or("").to_string();
        let arm = field_of(line, "arm").unwrap_or("").to_string();
        if line.starts_with("E7RESULT name=family_complete ") {
            complete.insert((family, arm));
        } else if line.starts_with("E7RESULT name=event ") {
            let kind = field_of(line, "kind").unwrap_or("").to_string();
            *events.entry((family, arm, kind)).or_default() += 1;
        } else if line.starts_with("E7RESULT name=history ") {
            if !line.ends_with(" line_end=ok") {
                unfinished_lines += 1;
                continue;
            }
            let index: u64 = field_of(line, "index")
                .and_then(|index| index.parse().ok())
                .expect("结果行带序号");
            let without_timing: String = line
                .split(' ')
                .filter(|field| !field.starts_with("elapsed_ns="))
                .collect::<Vec<_>>()
                .join(" ");
            let key = (family, arm);
            let group = groups.entry(key.clone()).or_default();
            match group.get(&index) {
                Some(kept) if *kept == without_timing => *duplicates.entry(key).or_default() += 1,
                Some(_) => *conflicts.entry(key).or_default() += 1,
                None => {
                    group.insert(index, without_timing);
                }
            }
        } else if line.starts_with("E7RESULT ") && !line.starts_with("E7RESULT name=done ") {
            unfinished_lines += 1;
        }
    }
    let mut output = Vec::new();
    for ((family, arm), lines) in &groups {
        let mut sums: BTreeMap<String, u64> = BTreeMap::new();
        let mut positive_segments: BTreeMap<String, u64> = BTreeMap::new();
        let mut peaks: BTreeMap<String, u64> = BTreeMap::new();
        let mut endings: BTreeMap<String, u64> = BTreeMap::new();
        let mut segments_issuing_at_least_two = 0_u64;
        for line in lines.values() {
            let ending = field_of(line, "ending")
                .unwrap_or("")
                .split(':')
                .next()
                .unwrap_or("")
                .to_string();
            *endings.entry(ending).or_default() += 1;
            if field_of(line, "issuances_at_operations")
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(0)
                >= 2
            {
                segments_issuing_at_least_two += 1;
            }
            for field in line.split(' ').skip(1) {
                let Some((name, value)) = field.split_once('=') else {
                    continue;
                };
                if FIELDS_NOT_SUMMED.contains(&name) {
                    continue;
                }
                let Ok(number) = value.parse::<u64>() else {
                    continue;
                };
                *sums.entry(name.to_string()).or_default() += number;
                if number > 0 {
                    *positive_segments.entry(name.to_string()).or_default() += 1;
                }
                if name.ends_with("_peak") {
                    let peak = peaks.entry(name.to_string()).or_default();
                    *peak = (*peak).max(number);
                }
            }
        }
        let key = (family.clone(), arm.clone());
        let mut line = format!(
            "E7RESULT name=summary family={family} arm={arm} histories={} complete={} duplicates={} conflicts={} q10_segments_issuing_at_least_two={segments_issuing_at_least_two}",
            lines.len(),
            if complete.contains(&key) { "yes" } else { "no" },
            duplicates.get(&key).copied().unwrap_or(0),
            conflicts.get(&key).copied().unwrap_or(0),
        );
        for (ending, count) in &endings {
            line.push_str(&format!(" ending_{ending}={count}"));
        }
        for (name, sum) in &sums {
            line.push_str(&format!(
                " sum_{name}={sum} segments_{name}={}",
                positive_segments.get(name).copied().unwrap_or(0)
            ));
        }
        for (name, peak) in &peaks {
            line.push_str(&format!(" max_{name}={peak}"));
        }
        output.push(line);
    }
    for ((family, arm, kind), count) in &events {
        output.push(format!(
            "E7RESULT name=event_count family={family} arm={arm} kind={kind} events={count}"
        ));
    }
    output.push(format!(
        "E7RESULT name=unfinished_lines count={unfinished_lines}"
    ));
    let emitted = output.len() + 1;
    output.push(format!("E7RESULT name=done emitted={emitted}"));
    output
}

/// 代码标记：`E175_CODE_BASELINE` 没设时 `not-given`（文件头写明）。
pub(super) fn code_baseline_from_the_variables(read_variable: VariableReader<'_>) -> String {
    read_variable("E175_CODE_BASELINE").unwrap_or_else(|| "not-given".to_string())
}

/// 读几份结果文件、核代码标记、把汇总写进 `target`（新文件，已有就拒）。
fn write_summary_of_the_result_files(
    input_paths: &[String],
    code_baseline_of_this_run: &str,
    target: &Path,
) {
    assert!(
        !target.exists(),
        "{} 已存在：汇总写新文件，不覆盖",
        target.display()
    );
    let mut text = String::new();
    let mut texts: Vec<(String, String)> = Vec::new();
    for path in input_paths {
        let file_text =
            std::fs::read_to_string(path).unwrap_or_else(|error| panic!("读不了 {path}：{error}"));
        text.push_str(&file_text);
        if !text.ends_with('\n') {
            text.push('\n');
        }
        texts.push((path.clone(), file_text));
    }
    let code_baseline =
        single_code_baseline_of_result_texts(&texts).unwrap_or_else(|reason| panic!("{reason}"));
    assert_eq!(
        code_baseline, code_baseline_of_this_run,
        "输入的代码标记与 E175_CODE_BASELINE 不同"
    );
    let mut output = code_baseline_header(&code_baseline) + "\n";
    for line in summarize(&text) {
        output.push_str(&format!("{line} code_baseline={code_baseline}\n"));
    }
    std::fs::write(target, output).expect("写汇总文件");
}

/// `summarize` 那条入口：`E175_SUMMARY_INPUTS` 不给就先跑一趟大规模，汇总它的结果文件。
pub(super) fn summarize_the_result_files_from_the_variables(read_variable: VariableReader<'_>) {
    let target = PathBuf::from(
        read_variable("E175_SUMMARY_FILE").expect("E175_SUMMARY_FILE 没设：汇总要写到点名的新文件"),
    );
    let input_paths: Vec<String> = match read_variable("E175_SUMMARY_INPUTS") {
        Some(list) => list.split(',').map(str::to_string).collect(),
        None => {
            let request = RunRequest::from_variables(read_variable);
            run_families_into_the_result_file(&request);
            vec![request.result_file.display().to_string()]
        }
    };
    write_summary_of_the_result_files(
        &input_paths,
        &code_baseline_from_the_variables(read_variable),
        &target,
    );
}

#[cfg(test)]
mod tests;
