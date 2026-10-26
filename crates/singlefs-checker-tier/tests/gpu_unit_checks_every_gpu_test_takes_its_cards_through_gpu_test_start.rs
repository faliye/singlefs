//! checker 档模块：crash_library
//! C604（代码三方 test-script-align-code-r2 判决 J3「补一条与硬件无关的源码扫描用例」）：GPU 用例取卡一律经
//! `gpu_unit_checks::gpu_test_start` 那一个入口（`usable_cards_or_report_not_run`、`first_usable_card_or_report_not_run`，
//! 与进程里共用一池卡的 `gpu_card_pool_or_report_not_run`）：
//! 没卡的机器上它报「本次未跑」，有卡而用不上它判红。绕开它直接调 `usable_gpu_cards` 或 `gpu_cards_by_priority` 起卡的用例，
//! 在有卡的机器上与经入口走同一条路，跑用例分不出；这一条扫源码，不要显卡。
//! 扫的是 `tests/` 下每份 `.rs`（这一份除外）整份，与 `src/` 下每份 `.rs` 里的测试代码：`#[test]`、条件含 `test`（不在 `not(…)` 里）的
//! `#[cfg(…)]` 下的项，顶格不顶格、与别的属性写在同一行、`cfg` 与括号之间带空白都算（`#[cfg(test)]`、`#[cfg(all(test, feature = "gpu"))]`、
//! `impl` 块里缩进的 `#[cfg(test)]`、`#[allow(…)] #[cfg (test)]`），`#[cfg_attr(条件, 属性…)]` 里带着这两种属性的同样算
//! （`#[cfg_attr(test, test)]`，第 10x 批代码轮判决 K3-2），文件级的 `#![cfg(test)]` 整份算，测试代码里 `mod 名字;` 引进来的
//! 那份文件也整份算（第 10t 批代码轮判决 K4-2）；产品路径（崩溃放量取卡、`gpu_unit_checks` 自己）不在测试代码里，不扫。
//! 注释（`//` 到行尾、`/* */`）不扫，字符串（普通、`b"…"`、`c"…"`、原始的 `r#"…"#`、`br#"…"#`、`cr#"…"#`）与字符字面量的内容也不扫
//! （字符串里的 `//` 不当注释起头，同一行后面的代码照扫）。
//! 判的是名字出现没出现，不只是带括号的调用：`use … as 别名` 那一行、拿函数当值、包一层改名都留得下这个名字（第 10t 批 K4-4）。
//! 不扫的（交 review）：`include!` 与 `#[path = "…"]` 引进来的文件、`benches/` 与 `examples/`、产品代码里包一层再从测试调的。
//! 改走卡池的几份测试文件（[`FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL`]）另判一层：只许经 `gpu_card_pool_or_report_not_run` 取卡、
//! 按量借，不许再各自拿第一张能用的卡、拿整份卡表或直接在卡上建判器（各按空闲显存的比例开，几个用例并行时互抢显存）。
//! 名单之外的 `tests/` 文件同样扫这一层：还绕开卡池的要登记在 [`TEST_FILES_NOT_YET_MOVED_ONTO_THE_CARD_POOL`]、写明为什么还没改，
//! 登记了的要真还在绕（第 10x 批代码轮判决 K3-2：名单之外的文件不扫，新写的 GPU 用例绕开卡池没人拦）。

use std::path::{Path, PathBuf};

use singlefs_checker_tier::crash_library::ExclusiveTemporaryDirectory;

/// 这一份自己的文件名：它的常量与样本里写着要找的名字，不扫。
const THIS_FILE_NAME: &str =
    "gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs";

/// 绕开入口直接起卡的函数名：代码里出现就算（调用、`use` 引进来、改名、当值传）。后三个是直接用 wgpu 起设备
/// （第 10t 批代码轮判决 K4-2）：`gpu_unit_checks` 自己的产品代码起设备用它们，测试代码里不该有。
const CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY: [&str; 5] = [
    "usable_gpu_cards",
    "gpu_cards_by_priority",
    "request_adapter",
    "request_device",
    "enumerate_adapters",
];

/// 经入口取卡的函数名（数调用）。
const CARD_TAKING_FUNCTIONS_THROUGH_THE_ENTRY: [&str; 3] = [
    "usable_cards_or_report_not_run",
    "first_usable_card_or_report_not_run",
    "gpu_card_pool_or_report_not_run",
];

/// 改走卡池的测试文件（相对包根）：每份至少一处调 [`CARD_POOL_FUNCTION`]，[`NAMES_THAT_BYPASS_THE_POOL`] 一处都不许有。
const FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL: [&str; 10] = [
    "tests/crash_amplification_pipeline_records_checks_and_reuses.rs",
    "tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs",
    "tests/crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs",
    "tests/crash_judge_fall_back_on_the_union_of_readers_matches_the_walk_alone.rs",
    "tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_root_ring_stride_refusals.rs",
    "tests/gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker.rs",
    "tests/gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs",
    "tests/gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core.rs",
    "tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs",
    "tests/record_checker_judges_absence_by_the_persisted_set.rs",
];

/// 绕开卡池拿卡、建判器的名字：经入口拿第一张能用的卡、拿整份卡表，或拿一张卡直接建判器（`GpuCrashJudge` 的每个构造都照那张卡的
/// 空闲显存或额度开草稿区，不记账）。代码里出现就算。
const NAMES_THAT_BYPASS_THE_POOL: [&str; 3] = [
    "first_usable_card_or_report_not_run",
    "usable_cards_or_report_not_run",
    "GpuCrashJudge",
];

/// 改走卡池的文件里经卡池取卡的函数名。
const CARD_POOL_FUNCTION: &str = "gpu_card_pool_or_report_not_run";

/// [`FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL`] 之外、还绕开卡池拿卡或建判器（[`NAMES_THAT_BYPASS_THE_POOL`]）的测试文件（相对包根）
/// 与为什么还没改。每一项要真还在绕：改完了就挪进名单、删这一项（只缩不涨）。
const TEST_FILES_NOT_YET_MOVED_ONTO_THE_CARD_POOL: [(&str, &str); 1] = [(
    "tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs",
    "GPU 对拍那一份：量显存的用例与几条按整张卡起的对拍还经入口拿整份卡表、第一张卡或直接建判器；这份文件在删除线第 34 批 U 手里改\
     （第 10x 批代码轮判决第三节末段：GPU 用例自己的 Out of Memory 与对拍流归它）",
)];

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

fn is_an_identifier_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

/// 从 `opening_position` 那个 `"` 起的一个字符串字面量：引号照抄，内容换成空格（换行照留，行号不变）；交回字面量之后的位置。
/// `closing_hashes` 是原始字符串（`r#"…"#`）收尾的 `#` 个数，普通字符串是 0 且认 `\` 转义。
fn blank_a_string_literal(
    characters: &[char],
    opening_position: usize,
    closing_hashes: Option<usize>,
    code: &mut String,
) -> usize {
    code.push('"');
    let mut position = opening_position + 1;
    while position < characters.len() {
        let character = characters[position];
        match closing_hashes {
            None if character == '\\' => {
                code.push(' ');
                match characters.get(position + 1) {
                    Some('\n') => code.push('\n'),
                    Some(_) => code.push(' '),
                    None => {}
                }
                position += 2;
                continue;
            }
            None if character == '"' => {
                code.push('"');
                return position + 1;
            }
            Some(hashes)
                if character == '"'
                    && characters[position + 1..]
                        .iter()
                        .take(hashes)
                        .filter(|following| **following == '#')
                        .count()
                        == hashes =>
            {
                code.push('"');
                code.extend(std::iter::repeat_n('#', hashes));
                return position + 1 + hashes;
            }
            None | Some(_) => {}
        }
        code.push(if character == '\n' { '\n' } else { ' ' });
        position += 1;
    }
    position
}

/// 一份源码每一行的代码部分，带 1 起的行号：注释（`//` 到行尾、可嵌套的 `/* */`）去掉，字符串（普通、`b"…"`、`c"…"`、`r#"…"#`、
/// `br#"…"#`、`cr#"…"#`）与字符字面量的内容换成空格（引号留着）。字符串里的 `//` 不当注释，写在字符串里的名字也不当代码里的名字。
fn code_lines(source: &str) -> Vec<(usize, String)> {
    let characters: Vec<char> = source.chars().collect();
    let mut code = String::with_capacity(source.len());
    let mut position = 0;
    while position < characters.len() {
        let character = characters[position];
        let next = characters.get(position + 1).copied();
        let follows_an_identifier =
            position > 0 && is_an_identifier_character(characters[position - 1]);
        let raw_string_hashes = |after_the_prefix: usize| -> Option<usize> {
            let hashes = characters[after_the_prefix..]
                .iter()
                .take_while(|following| **following == '#')
                .count();
            (characters.get(after_the_prefix + hashes) == Some(&'"')).then_some(hashes)
        };
        match (character, next) {
            ('/', Some('/')) => {
                while position < characters.len() && characters[position] != '\n' {
                    position += 1;
                }
            }
            ('/', Some('*')) => {
                let mut depth = 1_usize;
                position += 2;
                while position < characters.len() && depth > 0 {
                    match (characters[position], characters.get(position + 1)) {
                        ('/', Some('*')) => {
                            depth += 1;
                            position += 2;
                        }
                        ('*', Some('/')) => {
                            depth -= 1;
                            position += 2;
                        }
                        ('\n', _) => {
                            code.push('\n');
                            position += 1;
                        }
                        (_, _) => position += 1,
                    }
                }
            }
            ('"', _) => {
                position = blank_a_string_literal(&characters, position, None, &mut code);
            }
            ('r' | 'b' | 'c', Some(_)) if !follows_an_identifier => {
                let prefix_length = if (character == 'b' || character == 'c') && next == Some('r') {
                    2
                } else {
                    1
                };
                let is_raw = character == 'r' || prefix_length == 2;
                match (is_raw, raw_string_hashes(position + prefix_length)) {
                    (true, Some(hashes)) => {
                        code.extend(&characters[position..position + prefix_length + hashes]);
                        position = blank_a_string_literal(
                            &characters,
                            position + prefix_length + hashes,
                            Some(hashes),
                            &mut code,
                        );
                    }
                    (false, Some(0)) => {
                        code.push(character);
                        position =
                            blank_a_string_literal(&characters, position + 1, None, &mut code);
                    }
                    (true | false, Some(_) | None) => {
                        code.push(character);
                        position += 1;
                    }
                }
            }
            ('\'', Some('\\')) => {
                // 带转义的字符字面量（'\n'、'\''、'\u{…}'）：被转义的那一个之后找收尾的引号
                let closing = characters
                    .get(position + 3..)
                    .unwrap_or_default()
                    .iter()
                    .position(|following| *following == '\'')
                    .map_or(characters.len(), |offset| position + 3 + offset);
                code.push_str("' '");
                position = closing + 1;
            }
            ('\'', Some(_)) if characters.get(position + 2) == Some(&'\'') => {
                code.push_str("' '");
                position += 3;
            }
            (_, _) => {
                // 生命周期、标签与别的字符照抄
                code.push(character);
                position += 1;
            }
        }
    }
    code.split('\n')
        .take(source.split('\n').count())
        .enumerate()
        .map(|(index, line)| (index + 1, line.to_string()))
        .collect()
}

/// `code` 里名字 `identifier` 出现几次（前后都不是名字字符）。
fn mentions_of(code: &str, identifier: &str) -> usize {
    code.match_indices(identifier)
        .filter(|(start, _)| {
            let before = code[..*start].chars().next_back();
            let after = code[start + identifier.len()..].chars().next();
            !before.is_some_and(is_an_identifier_character)
                && !after.is_some_and(is_an_identifier_character)
        })
        .count()
}

/// `code` 里调 `function` 几次：名字之后（隔着空白）紧跟 `(`。
fn calls_of(code: &str, function: &str) -> usize {
    code.match_indices(function)
        .filter(|(start, _)| {
            let before = code[..*start].chars().next_back();
            let after = code[start + function.len()..].trim_start().chars().next();
            !before.is_some_and(is_an_identifier_character) && after == Some('(')
        })
        .count()
}

/// 一条 `cfg(…)` 属性的条件（`attribute` 是去掉注释、字符串内容之后的属性原文）里有没有一处不在 `not(…)` 里面的 `test`：
/// 有，这条属性下的项只在测试里编，是测试代码。
fn cfg_predicate_includes_test_outside_a_not(attribute: &str) -> bool {
    let mut enclosing_functions: Vec<String> = Vec::new();
    let mut word = String::new();
    let mut pending_word: Option<String> = None;
    for character in attribute.chars().chain(std::iter::once(' ')) {
        if is_an_identifier_character(character) {
            word.push(character);
            continue;
        }
        if !word.is_empty() {
            pending_word = Some(std::mem::take(&mut word));
        }
        if character.is_whitespace() {
            continue;
        }
        if character == '(' {
            enclosing_functions.push(pending_word.take().unwrap_or_default());
            continue;
        }
        if pending_word.take().as_deref() == Some("test")
            && !enclosing_functions.iter().any(|function| function == "not")
        {
            return true;
        }
        if character == ')' {
            enclosing_functions.pop();
        }
    }
    false
}

/// 项的关键字：属性下面的东西头上（属性之后、第一个 `{`、`;`、`=`、`,` 之前）有其中之一就是一项（函数、模块、`impl` 块……），
/// 项到它的 `{…}` 收尾或最外层的 `;` 为止，中间的逗号（泛型参数、`where` 子句）不算收尾。
const ITEM_KEYWORDS: [&str; 12] = [
    "fn",
    "mod",
    "impl",
    "struct",
    "enum",
    "trait",
    "const",
    "static",
    "type",
    "use",
    "union",
    "macro_rules",
];

/// 一条属性（去掉注释、字符串内容之后的原文里，从 `#` 起）：是不是内层属性（`#![…]`）、`[` 与配对的 `]` 之间的原文、那个 `]` 的位置。
struct AttributeInTheCode {
    is_inner: bool,
    inside: String,
    closing_position: usize,
}

/// 从 `hash_position` 那个 `#` 起是不是一条属性（`#` 与 `!`、`[` 之间可以有空白）；是就交回它。
fn attribute_at(characters: &[(usize, char)], hash_position: usize) -> Option<AttributeInTheCode> {
    let after = |position: usize| {
        (position..characters.len())
            .find(|candidate| !characters[*candidate].1.is_whitespace())
            .unwrap_or(characters.len())
    };
    let mut position = after(hash_position + 1);
    let is_inner = characters.get(position).map(|(_, character)| *character) == Some('!');
    if is_inner {
        position = after(position + 1);
    }
    if characters.get(position).map(|(_, character)| *character) != Some('[') {
        return None;
    }
    let mut bracket_depth = 0_usize;
    for (offset, (_, character)) in characters[position..].iter().enumerate() {
        match character {
            '[' => bracket_depth += 1,
            ']' => {
                bracket_depth -= 1;
                if bracket_depth == 0 {
                    let closing_position = position + offset;
                    return Some(AttributeInTheCode {
                        is_inner,
                        inside: characters[position + 1..closing_position]
                            .iter()
                            .map(|(_, inside_character)| *inside_character)
                            .collect(),
                        closing_position,
                    });
                }
            }
            _ => {}
        }
    }
    None
}

/// `cfg_attr(…)` 括号里那一段（去掉注释、字符串内容之后的原文）按最外层的逗号切开：第一段是条件，后面每段是一条属性。
fn arguments_of_a_cfg_attr(inside_the_parentheses: &str) -> Vec<&str> {
    let mut arguments = Vec::new();
    let mut depth = 0_usize;
    let mut start = 0;
    for (offset, character) in inside_the_parentheses.char_indices() {
        match character {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                arguments.push(&inside_the_parentheses[start..offset]);
                start = offset + 1;
            }
            _ => {}
        }
    }
    arguments.push(&inside_the_parentheses[start..]);
    arguments
}

/// 属性（`[` 与 `]` 之间的原文）让它下面的东西只在测试里编：裸的 `test`（`#[test]`），或条件含 `test`（不在 `not(…)` 里，
/// [`cfg_predicate_includes_test_outside_a_not`]）的 `cfg(…)`（`cfg` 与括号之间可以有空白），或 `cfg_attr(条件, 属性…)` 里
/// 带着这两种之一的（`#[cfg_attr(test, test)]`：测试里编成测试函数，第 10x 批代码轮判决 K3-2）。
fn attribute_marks_test_code(attribute_inside: &str) -> bool {
    let trimmed = attribute_inside.trim();
    if trimmed == "test" {
        return true;
    }
    if let Some(after_the_name) = trimmed.strip_prefix("cfg_attr") {
        return after_the_name
            .trim_start()
            .strip_prefix('(')
            .and_then(|rest| rest.trim_end().strip_suffix(')'))
            .is_some_and(|inside_the_parentheses| {
                arguments_of_a_cfg_attr(inside_the_parentheses)
                    .iter()
                    .skip(1)
                    .any(|attribute| attribute_marks_test_code(attribute))
            });
    }
    let Some(after_the_name) = trimmed.strip_prefix("cfg") else {
        return false;
    };
    let predicate = after_the_name.trim_start();
    predicate.starts_with('(') && cfg_predicate_includes_test_outside_a_not(predicate)
}

/// 从 `start` 起的那一项（之前的属性已经读过）到哪一个字符收尾，与它是不是 `mod 名字;`（交回名字）：头上有项的关键字的
/// （[`ITEM_KEYWORDS`]，头上先跳过别的属性）到它的 `{…}` 收尾或最外层的 `;` 为止；别的（字段、枚举成员、语句、表达式、match 的一支）
/// 另在最外层遇到 `,`、或多出来的右括号时收尾。
fn the_item_after(characters: &[(usize, char)], start: usize) -> (usize, Option<String>) {
    let mut head_start = start;
    loop {
        head_start = (head_start..characters.len())
            .find(|candidate| !characters[*candidate].1.is_whitespace())
            .unwrap_or(characters.len());
        match characters.get(head_start).map(|(_, character)| *character) {
            Some('#') => match attribute_at(characters, head_start) {
                Some(attribute) => head_start = attribute.closing_position + 1,
                None => break,
            },
            Some(_) | None => break,
        }
    }
    let head_end = characters[head_start..]
        .iter()
        .position(|(_, character)| matches!(character, '{' | ';' | '=' | ','))
        .map_or(characters.len(), |offset| head_start + offset);
    let head: String = characters[head_start..head_end]
        .iter()
        .map(|(_, character)| *character)
        .collect();
    let is_an_item = ITEM_KEYWORDS
        .iter()
        .any(|keyword| mentions_of(&head, keyword) > 0);
    let module_name = || {
        let mut words = head
            .split(|character: char| !is_an_identifier_character(character))
            .filter(|word| !word.is_empty());
        words.find(|word| *word == "mod")?;
        words.next().map(str::to_string)
    };
    let mut depth = 0_usize;
    let mut body_started = false;
    for (offset, (_, character)) in characters[start..].iter().enumerate() {
        let here = start + offset;
        match character {
            '(' | '[' | '{' => {
                if *character == '{' && depth == 0 {
                    body_started = true;
                }
                depth += 1;
            }
            ')' | ']' | '}' => {
                if depth == 0 {
                    return (here, None);
                }
                depth -= 1;
                if depth == 0 && *character == '}' && body_started {
                    return (here, None);
                }
            }
            ';' if depth == 0 => {
                return (here, if is_an_item { module_name() } else { None });
            }
            ',' if depth == 0 && !is_an_item => return (here, None),
            _ => {}
        }
    }
    (characters.len() - 1, None)
}

/// 一份库源码里属于测试代码的部分（[`test_code_of_a_library_source`]）。
struct TestCodeOfALibrarySource {
    /// 测试代码的行。
    lines: Vec<(usize, String)>,
    /// 收了几项（整份都是测试代码的算一项）。
    item_count: usize,
    /// 测试代码里 `mod 名字;` 引进来的另一份文件的模块名：那份文件整份都是测试代码（[`out_of_line_module_file`]）。
    out_of_line_module_names: Vec<String>,
}

/// 一份源码的代码行里每一处 `mod 名字;`（模块放在另一份文件里）的名字。
fn out_of_line_module_names(lines: &[(usize, String)]) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    for (_, code) in lines {
        let mut word = String::new();
        for character in code.chars().chain(std::iter::once(' ')) {
            if is_an_identifier_character(character) {
                word.push(character);
                continue;
            }
            if !word.is_empty() {
                tokens.push(std::mem::take(&mut word));
            }
            if !character.is_whitespace() {
                tokens.push(character.to_string());
            }
        }
    }
    tokens
        .windows(3)
        .filter(|window| {
            window[0] == "mod"
                && window[2] == ";"
                && window[1].chars().all(is_an_identifier_character)
        })
        .map(|window| window[1].clone())
        .collect()
}

/// `src/` 下一份源码里属于测试代码的部分：标着测试的属性（[`attribute_marks_test_code`]：`#[test]`、条件含 `test` 的 `#[cfg(…)]`，
/// 写在行首、行中、与别的属性同一行、`cfg` 后面带空白都算）连同它下面的那一项（[`the_item_after`]）；文件级的 `#![cfg(test)]` 让整份都是测试代码。
/// 测试代码里的 `mod 名字;` 记下名字，那份文件另由调用方整份当测试代码扫。
fn test_code_of_a_library_source(lines: &[(usize, String)]) -> TestCodeOfALibrarySource {
    let characters: Vec<(usize, char)> = lines
        .iter()
        .enumerate()
        .flat_map(|(line_position, (_, code))| {
            code.chars()
                .chain(std::iter::once('\n'))
                .map(move |character| (line_position, character))
        })
        .collect();
    let mut collected_line_positions = std::collections::BTreeSet::new();
    let mut item_count = 0;
    let mut module_names = Vec::new();
    let mut position = 0;
    while position < characters.len() {
        if characters[position].1 != '#' {
            position += 1;
            continue;
        }
        let Some(attribute) = attribute_at(&characters, position) else {
            position += 1;
            continue;
        };
        if !attribute_marks_test_code(&attribute.inside) {
            // 同一行后面可以还有属性（`#[allow(…)] #[cfg(test)]`）：从这一条之后接着找
            position = attribute.closing_position + 1;
            continue;
        }
        if attribute.is_inner {
            // 文件级的 `#![cfg(test)]`：整份都是测试代码
            return TestCodeOfALibrarySource {
                lines: lines.to_vec(),
                item_count: item_count + 1,
                out_of_line_module_names: out_of_line_module_names(lines),
            };
        }
        let (item_end, module_name) = the_item_after(&characters, attribute.closing_position + 1);
        item_count += 1;
        collected_line_positions.extend(characters[position].0..=characters[item_end].0);
        module_names.extend(module_name);
        position = item_end + 1;
    }
    TestCodeOfALibrarySource {
        lines: collected_line_positions
            .into_iter()
            .map(|line_position| lines[line_position].clone())
            .collect(),
        item_count,
        out_of_line_module_names: module_names,
    }
}

/// `parent` 这份源码里 `mod 名字;` 引进来的那份文件：`lib.rs`、`main.rs`、`mod.rs` 里的在同一目录，别的在与它同名的子目录；
/// `名字.rs` 或 `名字/mod.rs`，都不在交 `None`（编不过的写法，不归这里判）。`#[path = "…"]` 指的位置不认（属性的字符串内容扫之前已经抹掉）。
fn out_of_line_module_file(parent: &Path, module_name: &str) -> Option<PathBuf> {
    let directory = match parent.file_name().and_then(|name| name.to_str()) {
        Some("lib.rs" | "main.rs" | "mod.rs") => parent.parent()?.to_path_buf(),
        Some(_) | None => parent.with_extension(""),
    };
    [
        directory.join(format!("{module_name}.rs")),
        directory.join(module_name).join("mod.rs"),
    ]
    .into_iter()
    .find(|candidate| candidate.is_file())
}

/// `src/` 下扫出来的：绕开入口起卡的那几行、测试代码收了几项、测试代码里经入口取卡几处。
struct LibraryScan {
    bypasses: Vec<BypassingLine>,
    test_item_count: usize,
    entry_calls_in_test_code: usize,
}

/// 扫 `source_directory`（`src/`）下每份 `.rs` 的测试代码（[`test_code_of_a_library_source`]），连同测试代码里 `mod 名字;` 引进来的文件
/// （整份都是测试代码，它们再引的照样）；文件名照 `package_root` 起的相对路径记。同一行只记一次。
fn scan_the_library_sources(source_directory: &Path, package_root: &Path) -> LibraryScan {
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
    let mut scan = LibraryScan {
        bypasses: Vec::new(),
        test_item_count: 0,
        entry_calls_in_test_code: 0,
    };
    let mut module_files_that_are_test_code: Vec<PathBuf> = Vec::new();
    for path in rust_source_files_under(source_directory) {
        let test_code = test_code_of_a_library_source(&code_lines(&read(&path)));
        scan.test_item_count += test_code.item_count;
        scan.entry_calls_in_test_code += entry_call_count(&test_code.lines);
        scan.bypasses.extend(lines_mentioning(
            &relative(&path),
            &test_code.lines,
            &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
        ));
        module_files_that_are_test_code.extend(
            test_code
                .out_of_line_module_names
                .iter()
                .filter_map(|module_name| out_of_line_module_file(&path, module_name)),
        );
    }
    let mut visited = std::collections::BTreeSet::new();
    while let Some(module_file) = module_files_that_are_test_code.pop() {
        if !visited.insert(module_file.clone()) {
            continue;
        }
        let lines = code_lines(&read(&module_file));
        scan.test_item_count += 1;
        scan.entry_calls_in_test_code += entry_call_count(&lines);
        scan.bypasses.extend(lines_mentioning(
            &relative(&module_file),
            &lines,
            &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
        ));
        module_files_that_are_test_code.extend(
            out_of_line_module_names(&lines)
                .iter()
                .filter_map(|module_name| out_of_line_module_file(&module_file, module_name)),
        );
    }
    scan.bypasses.sort_by(|left, right| {
        (&left.file_relative_to_the_package, left.line_number)
            .cmp(&(&right.file_relative_to_the_package, right.line_number))
    });
    scan.bypasses.dedup();
    scan
}

/// 这几行里经入口取卡的调用有几处。
fn entry_call_count(lines: &[(usize, String)]) -> usize {
    lines
        .iter()
        .map(|(_, code)| {
            CARD_TAKING_FUNCTIONS_THROUGH_THE_ENTRY
                .iter()
                .map(|function| calls_of(code, function))
                .sum::<usize>()
        })
        .sum()
}

/// 这几行里出现了 `names` 里某个名字的每一行。
fn lines_mentioning(
    file_relative_to_the_package: &str,
    lines: &[(usize, String)],
    names: &[&str],
) -> Vec<BypassingLine> {
    lines
        .iter()
        .filter(|(_, code)| names.iter().any(|name| mentions_of(code, name) > 0))
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
        bypasses.extend(lines_mentioning(
            &relative(&path),
            &lines,
            &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
        ));
    }
    let library = scan_the_library_sources(&package_root.join("src"), package_root);
    let library_test_item_count = library.test_item_count;
    let entry_calls_in_library_test_code = library.entry_calls_in_test_code;
    bypasses.extend(library.bypasses);
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
         改成经 gpu_card_pool_or_report_not_run（共用一池卡、按量借）、usable_cards_or_report_not_run 或 first_usable_card_or_report_not_run 取卡：{not_allowed:#?}"
    );
    println!(
        "GPU_CARD_ENTRY_SCAN test_files={scanned_test_file_count} library_test_items={library_test_item_count} bypasses_allowed={}",
        BYPASSES_ALLOWED_AFTER_THE_ENTRY_DECIDED.len()
    );
}

#[test]
fn every_test_file_moved_onto_the_card_pool_takes_its_cards_only_from_the_pool() {
    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut bypasses_of_the_pool = Vec::new();
    let mut files_without_a_pool_call = Vec::new();
    for file in FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL {
        let path = package_root.join(file);
        let source = std::fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!(
                "{} 读不出：{error}（改了名就把这一项一起改）",
                path.display()
            )
        });
        let lines = code_lines(&source);
        if !lines
            .iter()
            .any(|(_, code)| calls_of(code, CARD_POOL_FUNCTION) > 0)
        {
            files_without_a_pool_call.push(file);
        }
        bypasses_of_the_pool.extend(lines_mentioning(file, &lines, &NAMES_THAT_BYPASS_THE_POOL));
    }
    assert!(
        files_without_a_pool_call.is_empty(),
        "这几份改走卡池的文件里一处 {CARD_POOL_FUNCTION}(…) 都没有：{files_without_a_pool_call:?}"
    );
    assert!(
        bypasses_of_the_pool.is_empty(),
        "这几处在改走卡池的文件里绕开卡池拿卡或直接建判器（不记账，几个用例并行时互抢显存）：\
         改成经 gpu_card_pool_or_report_not_run 取池、建好表之后按量借（GpuCardPool::lease_card_or_report_not_run，\
         判器用 crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool）：{bypasses_of_the_pool:#?}"
    );
    println!(
        "CARD_POOL_ONLY_FILES checked={} bypasses=0",
        FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL.len()
    );
}

/// 第 10x 批代码轮判决 K3-2：名单（[`FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL`]）之外的 `tests/` 文件（含子目录，这一份除外）也扫
/// 绕开卡池的名字：扫出来的文件要与登记的（[`TEST_FILES_NOT_YET_MOVED_ONTO_THE_CARD_POOL`]）逐个相同：新写的绕开卡池要红，
/// 登记了而已经不绕的也红（挪进名单）。
#[test]
fn every_test_file_outside_the_pool_list_that_bypasses_the_pool_is_registered_with_a_reason() {
    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut scanned_file_count = 0_usize;
    let mut bypassing_files = Vec::new();
    for path in rust_source_files_under(&package_root.join("tests")) {
        if path.file_name().is_some_and(|name| name == THIS_FILE_NAME) {
            continue;
        }
        let relative = path
            .strip_prefix(package_root)
            .expect("扫的文件都在包根下")
            .to_string_lossy()
            .into_owned();
        if FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL.contains(&relative.as_str()) {
            continue;
        }
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} 读不出：{error}", path.display()));
        scanned_file_count += 1;
        if !lines_mentioning(&relative, &code_lines(&source), &NAMES_THAT_BYPASS_THE_POOL)
            .is_empty()
        {
            bypassing_files.push(relative);
        }
    }
    assert!(
        scanned_file_count >= 20,
        "名单之外扫到的 tests/ 文件只有 {scanned_file_count} 份，少得不对，扫法坏了"
    );
    let registered: Vec<String> = TEST_FILES_NOT_YET_MOVED_ONTO_THE_CARD_POOL
        .iter()
        .map(|(file, _why_not_yet)| (*file).to_string())
        .collect();
    assert_eq!(
        bypassing_files, registered,
        "名单之外还绕开卡池拿卡或建判器的测试文件要与登记的逐个相同：新扫出来的改成经 gpu_card_pool_or_report_not_run 取池、按量借\
         （改不了的登记进 TEST_FILES_NOT_YET_MOVED_ONTO_THE_CARD_POOL、写明为什么）；登记了而已经不绕的挪进 FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL"
    );
    println!(
        "CARD_POOL_BYPASSES_OUTSIDE_THE_LIST scanned={scanned_file_count} registered={}",
        registered.len()
    );
}

/// 第 10x 批代码轮判决 K3-2 红样本：库源码里 `#[cfg_attr(test, test)]` 下的函数在测试里编成测试函数，里面的起卡调用照扫；
/// `cfg_attr` 只带别的属性（`allow`）的不算测试代码。
#[test]
fn test_code_under_a_cfg_attr_carrying_test_in_a_library_source_is_caught() {
    let sample = "#[cfg_attr(test, test)]\n\
                  fn takes_cards() {\n\
                  \x20   let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
                  }\n\
                  #[cfg_attr(feature = \"gpu\", allow(dead_code))]\n\
                  fn product() {\n\
                  \x20   let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
                  }\n\
                  #[cfg_attr(feature = \"gpu\", cfg(test))]\n\
                  fn helper() {\n\
                  \x20   let _cards = crate::gpu_unit_checks::gpu_cards_by_priority(&[]);\n\
                  }\n";
    assert_eq!(
        bypass_line_numbers_in_a_library_sample(sample),
        vec![3, 11],
        "cfg_attr 带着 test 或 cfg(test) 的扫；只带 allow 的是产品代码、不扫"
    );
}

/// 第 10t 批 K4-4 红样本一：`use … as 别名` 引进起卡的函数、再调别名，调用那一行没有原名，`use` 那一行有：扫得出；
/// 改走卡池的文件里给入口函数起别名同样扫得出。
#[test]
fn a_card_taking_function_brought_in_under_an_alias_is_caught() {
    let sample = "use singlefs_checker_tier::gpu_unit_checks::usable_gpu_cards as cards_now;\n\
                  use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::first_usable_card_or_report_not_run as first_card;\n\
                  fn takes_cards() {\n    let _cards = cards_now();\n    let _card = first_card(\"sample\");\n}\n";
    let lines = code_lines(sample);
    let entry_bypasses = lines_mentioning(
        "tests/sample.rs",
        &lines,
        &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
    );
    assert_eq!(
        entry_bypasses
            .iter()
            .map(|bypass| bypass.line_number)
            .collect::<Vec<_>>(),
        vec![1],
        "改名引进 usable_gpu_cards 的那一行要扫得出：{entry_bypasses:#?}"
    );
    let pool_bypasses = lines_mentioning("tests/sample.rs", &lines, &NAMES_THAT_BYPASS_THE_POOL);
    assert_eq!(
        pool_bypasses.iter().map(|bypass| bypass.line_number).collect::<Vec<_>>(),
        vec![2],
        "改走卡池的文件里改名引进 first_usable_card_or_report_not_run 的那一行要扫得出：{pool_bypasses:#?}"
    );
}

/// 第 10t 批 K4-4 红样本二：同一行前面有个带 `//` 的字符串字面量，后面的起卡调用照扫；字符串里写的名字不当调用。
#[test]
fn a_card_taking_call_after_a_string_holding_two_slashes_on_the_same_line_is_caught() {
    let sample = "fn takes_cards() {\n    let _address = \"http://example\"; let _cards = singlefs_checker_tier::gpu_unit_checks::usable_gpu_cards();\n    let _message = \"usable_gpu_cards() 写在字符串里不算\";\n    let _raw = r#\"// 原始字符串里的也不算\"#; let _card = singlefs_checker_tier::gpu_unit_checks::gpu_cards_by_priority(&[]);\n}\n";
    let bypasses = lines_mentioning(
        "tests/sample.rs",
        &code_lines(sample),
        &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
    );
    assert_eq!(
        bypasses
            .iter()
            .map(|bypass| bypass.line_number)
            .collect::<Vec<_>>(),
        vec![2, 4],
        "字符串里的 // 之后的调用要扫得出，字符串里写的名字不算：{bypasses:#?}"
    );
}

/// 第 10t 批 K4-4 红样本三：库源码里条件不只是 `test` 的 `#[cfg(all(test, feature = "gpu"))]` 与 `impl` 块里缩进的 `#[cfg(test)]`
/// 下的项都是测试代码、照扫；`#[cfg(not(test))]` 下的是产品代码、不扫。
#[test]
fn test_code_under_a_compound_or_indented_cfg_test_in_a_library_source_is_caught() {
    let sample = "pub fn product() {}\n\
                  \n\
                  #[cfg(all(test, feature = \"gpu\"))]\n\
                  mod gpu_tests {\n\
                  \x20   #[test]\n\
                  \x20   fn takes_cards() {\n\
                  \x20       let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
                  \x20   }\n\
                  }\n\
                  \n\
                  impl Something {\n\
                  \x20   #[cfg(test)]\n\
                  \x20   fn helper() {\n\
                  \x20       let _cards = crate::gpu_unit_checks::gpu_cards_by_priority(&[]);\n\
                  \x20   }\n\
                  \n\
                  \x20   #[cfg(not(test))]\n\
                  \x20   fn product_only() {\n\
                  \x20       let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
                  \x20   }\n\
                  }\n";
    let test_code = test_code_of_a_library_source(&code_lines(sample));
    assert_eq!(
        test_code.item_count, 2,
        "两项测试代码：{:#?}",
        test_code.lines
    );
    let bypasses = lines_mentioning(
        "src/sample.rs",
        &test_code.lines,
        &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
    );
    assert_eq!(
        bypasses.iter().map(|bypass| bypass.line_number).collect::<Vec<_>>(),
        vec![7, 14],
        "cfg(all(test, …)) 与缩进的 cfg(test) 下的起卡调用要扫得出，cfg(not(test)) 下的不扫：{bypasses:#?}"
    );
}

/// 一份库源码样本里扫得出的起卡那几行的行号（[`test_code_of_a_library_source`]）。
fn bypass_line_numbers_in_a_library_sample(sample: &str) -> Vec<usize> {
    lines_mentioning(
        "src/sample.rs",
        &test_code_of_a_library_source(&code_lines(sample)).lines,
        &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
    )
    .iter()
    .map(|bypass| bypass.line_number)
    .collect()
}

/// 第 10t 批代码轮判决 K4-2 红样本一：C 字符串 `c"…"` 与原始 C 字符串 `cr#"…"#` 之后同一行的起卡调用照扫
/// （以前 `cr` 的 `r` 跟在 `c` 后面不认成原始串，把 `\` 当转义、吞到下一个引号）。
#[test]
fn a_card_taking_call_after_a_raw_c_string_on_the_same_line_is_caught() {
    let sample = r###"fn takes_cards() {
    let _a = cr#"x" "#; let _cards = usable_gpu_cards(); let _b = cr#"y"#;
    let _c = c"\\"; let _card = gpu_cards_by_priority(&[]);
    let _d = cr"\"; let _more = usable_gpu_cards();
}
"###;
    let bypasses = lines_mentioning(
        "tests/sample.rs",
        &code_lines(sample),
        &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
    );
    assert_eq!(
        bypasses
            .iter()
            .map(|bypass| bypass.line_number)
            .collect::<Vec<_>>(),
        vec![2, 3, 4],
        "C 字符串之后的起卡调用要扫得出：{bypasses:#?}"
    );
}

/// 第 10t 批代码轮判决 K4-2 红样本二：库源码里 `#[cfg(test)]` 下的函数带泛型参数（`<A, B>`）或 `where` 子句，里面的逗号不让这一项在签名里收尾，
/// 函数体照扫。
#[test]
fn test_code_whose_signature_holds_commas_is_scanned_to_the_end_of_its_body() {
    let sample = "#[cfg(test)]\n\
                  fn helper<A, B>() {\n\
                  \x20   let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
                  }\n\
                  \n\
                  #[cfg(test)]\n\
                  fn bounded<T>()\n\
                  where\n\
                  \x20   T: Copy,\n\
                  {\n\
                  \x20   let _cards = crate::gpu_unit_checks::gpu_cards_by_priority(&[]);\n\
                  }\n";
    assert_eq!(
        bypass_line_numbers_in_a_library_sample(sample),
        vec![3, 11],
        "泛型参数、where 子句里的逗号之后的函数体要扫"
    );
}

/// 第 10t 批代码轮判决 K4-2 红样本三：库源码里没有 `#[cfg(test)]`、只有裸 `#[test]` 的函数也是测试代码（只在测试里编），照扫。
#[test]
fn a_bare_test_function_in_a_library_source_is_scanned() {
    let sample = "pub fn product() {}\n\
                  \n\
                  #[test]\n\
                  fn takes_cards() {\n\
                  \x20   let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
                  }\n";
    assert_eq!(bypass_line_numbers_in_a_library_sample(sample), vec![5]);
}

/// 第 10t 批代码轮判决 K4-2 红样本四：`cfg(test)` 的几种不在行首、不紧挨着的写法：文件级的 `#![cfg(test)]`（整份都是测试代码，
/// 下面第二项也算）、与别的属性写在同一行、`cfg` 与括号之间带空白。
#[test]
fn cfg_test_written_off_the_start_of_the_line_or_spaced_or_at_the_file_level_is_caught() {
    let file_level = "#![cfg(test)]\n\
                      fn first() {\n\
                      \x20   let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
                      }\n\
                      fn second() {\n\
                      \x20   let _cards = crate::gpu_unit_checks::gpu_cards_by_priority(&[]);\n\
                      }\n";
    assert_eq!(
        bypass_line_numbers_in_a_library_sample(file_level),
        vec![3, 6],
        "文件级的 #![cfg(test)]：整份都扫"
    );
    let same_line = "#[allow(dead_code)] #[cfg(test)] fn helper() {\n\
                     \x20   let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
                     }\n";
    assert_eq!(
        bypass_line_numbers_in_a_library_sample(same_line),
        vec![2],
        "与别的属性同一行的 #[cfg(test)]"
    );
    let spaced = "#[cfg (test)]\n\
                  fn helper() {\n\
                  \x20   let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
                  }\n";
    assert_eq!(
        bypass_line_numbers_in_a_library_sample(spaced),
        vec![3],
        "cfg 与括号之间带空白"
    );
}

/// 第 10t 批代码轮判决 K4-2 红样本五：不经那两个名字、直接用 wgpu 起设备（`request_adapter`、`request_device`、`enumerate_adapters`）照扫。
#[test]
fn a_test_that_starts_a_device_straight_through_wgpu_is_caught() {
    let sample = "fn takes_cards(instance: &wgpu::Instance) {\n\
                  \x20   let adapter = pollster::block_on(instance.request_adapter(&options));\n\
                  \x20   let device = pollster::block_on(adapter.request_device(&descriptor));\n\
                  \x20   let adapters = instance.enumerate_adapters(wgpu::Backends::VULKAN);\n\
                  }\n";
    let bypasses = lines_mentioning(
        "tests/sample.rs",
        &code_lines(sample),
        &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
    );
    assert_eq!(
        bypasses
            .iter()
            .map(|bypass| bypass.line_number)
            .collect::<Vec<_>>(),
        vec![2, 3, 4],
        "直接用 wgpu 起设备的每一行都要扫得出：{bypasses:#?}"
    );
}

/// 一份临时的包目录（这一份一次性临时目录，丢掉时连同整棵删掉）。
struct TemporaryPackage {
    root: ExclusiveTemporaryDirectory,
}

impl TemporaryPackage {
    /// 建好包根与 `src/`，按 `(相对 src/ 的路径, 内容)` 写进去。
    fn with_sources(test_name: &str, sources: &[(&str, &str)]) -> Self {
        let root = ExclusiveTemporaryDirectory::create(&format!("gpu-entry-scan-{test_name}"))
            .expect("建得了一次性临时目录");
        for (relative_path, content) in sources {
            let path = root.join("src").join(relative_path);
            std::fs::create_dir_all(path.parent().expect("src 下的文件有上一层"))
                .expect("建得了临时目录");
            std::fs::write(&path, content).expect("写得了临时文件");
        }
        Self { root }
    }
}

/// 第 10t 批代码轮判决 K4-2 红样本六：库源码里 `#[cfg(test)] mod gpu_tests;` 把测试放进另一份文件，那份文件里没有 cfg 属性：
/// 整份当测试代码扫（它再引的 `mod deeper;` 照样），不当产品代码放过；`lib.rs` 里不在测试代码里的 `mod product;` 不扫。
#[test]
fn a_test_module_kept_in_another_file_is_scanned_whole() {
    let package = TemporaryPackage::with_sources(
        "test-module-in-another-file",
        &[
            ("lib.rs", "mod product;\n#[cfg(test)]\nmod gpu_tests;\n"),
            (
                "product.rs",
                "pub fn product() {\n    let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n}\n",
            ),
            (
                "gpu_tests.rs",
                "mod deeper;\nfn helper() {\n    let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n}\n",
            ),
            (
                "gpu_tests/deeper.rs",
                "fn deeper() {\n    let _cards = crate::gpu_unit_checks::gpu_cards_by_priority(&[]);\n}\n",
            ),
        ],
    );
    let scan = scan_the_library_sources(&package.root.join("src"), &package.root);
    assert_eq!(
        scan.bypasses
            .iter()
            .map(|bypass| (
                bypass.file_relative_to_the_package.as_str(),
                bypass.line_number
            ))
            .collect::<Vec<_>>(),
        vec![("src/gpu_tests.rs", 3), ("src/gpu_tests/deeper.rs", 2)],
        "另一份文件里的测试模块整份要扫，产品代码的 mod product; 不扫：{:#?}",
        scan.bypasses
    );
}
