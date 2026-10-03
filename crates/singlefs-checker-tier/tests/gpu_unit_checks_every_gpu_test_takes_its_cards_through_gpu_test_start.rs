//! checker 档模块：无（只读这个包的源码文本，不调它的任何模块）
//! C604（代码三方 test-script-align-code-r2 判决 J3「补一条与硬件无关的源码扫描用例」）：GPU 用例取卡一律经
//! `gpu_unit_checks::gpu_test_start` 那一个入口（`usable_cards_or_report_not_run`、`first_usable_card_or_report_not_run`）：
//! 没卡的机器上它报「本次未跑」，有卡而用不上它判红。绕开它直接调 `usable_gpu_cards` 或 `gpu_cards_by_priority` 起卡的用例，
//! 在有卡的机器上与经入口走同一条路，跑用例分不出；这一条扫源码，不要显卡。
//! 扫的是 `tests/` 下每份 `.rs`（这一份除外）整份，与 `src/` 下每份 `.rs` 里顶格 `#[cfg(test)]` 下的模块与函数；
//! 产品路径（崩溃放量取卡、`gpu_unit_checks` 自己）不在测试代码里，不扫。行里 `//` 之后的注释不扫。

use std::path::{Path, PathBuf};

/// 这一份自己的文件名：它的常量里写着要找的调用，不扫。
const THIS_FILE_NAME: &str =
    "gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs";

/// 绕开入口直接起卡的调用。
const CARD_TAKING_CALLS_THAT_BYPASS_THE_ENTRY: [&str; 2] =
    ["usable_gpu_cards(", "gpu_cards_by_priority("];

/// 经入口取卡的调用。
const CARD_TAKING_CALLS_THROUGH_THE_ENTRY: [&str; 2] = [
    "usable_cards_or_report_not_run(",
    "first_usable_card_or_report_not_run(",
];

/// 经入口判过「有卡、照跑」之后再起卡、准许绕开的几处：(相对包根的文件, 那一行去掉前后空白的代码, 为什么准许)。
/// 每一项必须恰好命中一次，命中不到也判红：准许的那一行改了或删了，这里不许还留着。
const BYPASSES_ALLOWED_AFTER_THE_ENTRY_DECIDED: [(&str, &str, &str); 1] = [(
    "tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs",
    "for card in singlefs_checker_tier::gpu_unit_checks::gpu_cards_by_priority(&every_card) {",
    "量显存的用例先经入口取卡、判过照跑，放掉那份上下文再按整张卡的额度重起：量的是重起的那一份占多少显存",
)];

/// 一处绕开入口起卡的代码。
#[derive(Debug, PartialEq, Eq)]
struct BypassingLine {
    file_relative_to_the_package: String,
    line_number: usize,
    code: String,
}

/// `directory` 下（含子目录）每份 `.rs`，按路径排好。
fn rust_source_files_under(directory: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut directories_to_visit = vec![directory.to_path_buf()];
    while let Some(visiting) = directories_to_visit.pop() {
        let entries = std::fs::read_dir(&visiting)
            .unwrap_or_else(|error| panic!("{} 列不出：{error}", visiting.display()));
        for entry in entries {
            let path = entry.expect("目录项读得出").path();
            if path.is_dir() {
                directories_to_visit.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// 一份源码每一行的代码部分（`//` 之后的注释去掉），带 1 起的行号。
fn code_lines(source: &str) -> Vec<(usize, &str)> {
    source
        .lines()
        .enumerate()
        .map(|(index, line)| {
            (
                index + 1,
                line.split_once("//").map_or(line, |(code, _comment)| code),
            )
        })
        .collect()
}

/// `src/` 下一份源码里属于测试代码的行：顶格 `#[cfg(test)]`（与它下面紧跟的别的属性）之后是顶格的 `mod `、`fn `、`pub fn ` 起头的项，
/// 从项头起到下一个顶格的 `}` 为止；`#[cfg(test)]` 下的别的项（常量、`use`）不跑代码，不收。交回 (测试代码的行, 收了几项)。
fn test_code_lines_of_a_library_source<'source>(
    lines: &[(usize, &'source str)],
) -> (Vec<(usize, &'source str)>, usize) {
    let mut collected = Vec::new();
    let mut item_count = 0;
    let mut position = 0;
    while position < lines.len() {
        if lines[position].1.trim_end() != "#[cfg(test)]" {
            position += 1;
            continue;
        }
        let Some(header_offset) = lines[position + 1..]
            .iter()
            .position(|(_, code)| !code.starts_with("#["))
        else {
            break;
        };
        let header_position = position + 1 + header_offset;
        let header = lines[header_position].1;
        if !(header.starts_with("mod ")
            || header.starts_with("fn ")
            || header.starts_with("pub fn "))
        {
            position = header_position;
            continue;
        }
        item_count += 1;
        let end_position = lines[header_position..]
            .iter()
            .position(|(_, code)| code.trim_end() == "}")
            .map_or(lines.len(), |offset| header_position + offset + 1);
        collected.extend_from_slice(&lines[header_position..end_position]);
        position = end_position;
    }
    (collected, item_count)
}

/// 这几行里经入口取卡的调用有几处。
fn entry_call_count(lines: &[(usize, &str)]) -> usize {
    lines
        .iter()
        .map(|(_, code)| {
            CARD_TAKING_CALLS_THROUGH_THE_ENTRY
                .iter()
                .map(|call| code.matches(call).count())
                .sum::<usize>()
        })
        .sum()
}

/// 这几行里绕开入口起卡的每一行。
fn bypassing_lines(
    file_relative_to_the_package: &str,
    lines: &[(usize, &str)],
) -> Vec<BypassingLine> {
    lines
        .iter()
        .filter(|(_, code)| {
            CARD_TAKING_CALLS_THAT_BYPASS_THE_ENTRY
                .iter()
                .any(|call| code.contains(call))
        })
        .map(|(line_number, code)| BypassingLine {
            file_relative_to_the_package: file_relative_to_the_package.to_string(),
            line_number: *line_number,
            code: code.trim().to_string(),
        })
        .collect()
}

#[test]
fn every_gpu_test_takes_its_cards_through_gpu_test_start() {
    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let relative = |path: &Path| {
        path.strip_prefix(package_root)
            .expect("扫的文件都在包根下")
            .to_string_lossy()
            .into_owned()
    };
    let read = |path: &Path| {
        std::fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("{} 读不出：{error}", path.display()))
    };
    let mut bypasses = Vec::new();
    let mut scanned_test_file_count = 0_usize;
    let mut entry_calls_under_tests = 0_usize;
    for path in rust_source_files_under(&package_root.join("tests")) {
        if path.file_name().is_some_and(|name| name == THIS_FILE_NAME) {
            continue;
        }
        let source = read(&path);
        let lines = code_lines(&source);
        scanned_test_file_count += 1;
        entry_calls_under_tests += entry_call_count(&lines);
        bypasses.extend(bypassing_lines(&relative(&path), &lines));
    }
    let mut library_test_item_count = 0_usize;
    let mut entry_calls_in_library_test_code = 0_usize;
    for path in rust_source_files_under(&package_root.join("src")) {
        let source = read(&path);
        let (lines, item_count) = test_code_lines_of_a_library_source(&code_lines(&source));
        library_test_item_count += item_count;
        entry_calls_in_library_test_code += entry_call_count(&lines);
        bypasses.extend(bypassing_lines(&relative(&path), &lines));
    }
    assert!(
        scanned_test_file_count >= 20 && library_test_item_count >= 10,
        "扫到的 tests/ 文件 {scanned_test_file_count} 份、src/ 里的测试项 {library_test_item_count} 个，少得不对，扫法坏了"
    );
    assert!(
        entry_calls_under_tests >= 1 && entry_calls_in_library_test_code >= 1,
        "tests/ 里经入口取卡 {entry_calls_under_tests} 处、src/ 测试代码里 {entry_calls_in_library_test_code} 处：两边都该至少一处，扫法坏了"
    );
    for (allowed_file, allowed_code, _why_allowed) in BYPASSES_ALLOWED_AFTER_THE_ENTRY_DECIDED {
        let hits = bypasses
            .iter()
            .filter(|bypass| {
                bypass.file_relative_to_the_package == allowed_file && bypass.code == allowed_code
            })
            .count();
        assert_eq!(
            hits, 1,
            "准许绕开的那一行（{allowed_file}「{allowed_code}」）要恰好命中一次：那一行改了或删了，把这一项一起改或删"
        );
    }
    let not_allowed: Vec<&BypassingLine> = bypasses
        .iter()
        .filter(|bypass| {
            !BYPASSES_ALLOWED_AFTER_THE_ENTRY_DECIDED.iter().any(
                |(allowed_file, allowed_code, _why_allowed)| {
                    bypass.file_relative_to_the_package == *allowed_file
                        && bypass.code == *allowed_code
                },
            )
        })
        .collect();
    assert!(
        not_allowed.is_empty(),
        "这几处绕开 gpu_unit_checks::gpu_test_start 直接起卡（没卡的机器上报不出「本次未跑」、有卡而用不上判不了红），\
         改成经 usable_cards_or_report_not_run 或 first_usable_card_or_report_not_run 取卡：{not_allowed:#?}"
    );
}
