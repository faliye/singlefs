//! 扫源码的单测用的读法：越过注释与字面量读行、切掉测试模块、找交着叫停请求的那几层。

// 扫源码的用例里，串里的花括号一律写成 \u 转义：门禁 code-source-discipline 的 mutation-tables 格配花括号时不跳字符串，
// 串里落单的半边括号会让它把后面的用例算到别的模块下。
/// 崩溃放量模块族的根（[`ROOT_OF_THE_FAMILY`]）里测试模块的头两行（扫源码的用例从这里切掉测试模块）。
pub(super) const TESTS_MODULE_OPENING: &str = "#[cfg(test)]\nmod tests \u{7b}";
/// 顶格的右括号自成一行（测试模块在这里收尾）。
pub(super) const TOP_LEVEL_CLOSING_BRACE_LINE: &str = "\n\u{7d}\n";

/// `code` 里 `const <constant_name>: &str = "…";` 那个串（前面带不带 `pub`、等号后面换不换行都认）。
///
/// # Panics
/// 没有这个常量的定义、定义不止一处，或值不是一个串字面量。
pub(super) fn value_of_the_string_constant(code: &str, constant_name: &str) -> String {
    let declaration = format!("const {constant_name}: &str =");
    let mut declarations = code.match_indices(declaration.as_str());
    let (position, _) = declarations.next().unwrap_or_else(|| {
            panic!(
                "按常量 {constant_name} 读的环境变量，这份源码测试模块之外没有 `{declaration}` 那一处定义：扫法认不出"
            )
        });
    assert!(
        declarations.next().is_none(),
        "常量 {constant_name} 的定义不止一处"
    );
    let value = code[position + declaration.len()..]
        .trim_start()
        .strip_prefix('"')
        .unwrap_or_else(|| panic!("常量 {constant_name} 的值不是串字面量"));
    value[..value.find('"').expect("常量的串收得了尾")].to_string()
}

/// `source`（族的根、`crash_library.rs` 这类测试模块写在自己里面的源文件，或用例在它上面改过的一份）去掉测试模块之后的文本：从测试模块的头两行起，到与它配对的顶格右括号那一行止，
/// 连同这两处一起去掉。头两行只认这一种，与卡池扫源码那一份（`gpu_unit_checks.rs` 的 `code_before_the_tests_module`）同一口径、
/// 按行逐字：
/// - 相邻两整行逐字是 [`TESTS_MODULE_OPENING`] 那两行（`#[cfg(test)]`，与 `mod tests`、一个空格、一个左花括号；行首起、行尾止，不带别的空白与注释，换行是 `\n`），在代码里（注释与字面量之外）；
/// - 在花括号的第 0 层，也在圆括号、方括号的第 0 层；
/// - 前一个词（空白、注释、字面量都跳过）是分号、右花括号、右方括号或文件开头。
///
/// 照原文找第一处时，块注释里写一份就把切口挪到注释那里，注释与真测试模块之间的代码被当成测试切掉；
/// 拿一个吃掉 `#[cfg(test)]` 的宏把测试模块包进一次宏调用的圆括号里，那个「测试模块」在非测试构建里照编、里面藏的层照样可调用，
/// 头两行却仍在花括号第 0 层，宏调用的括号里头两行前一个词是 `(`，前面先放一条以分号收尾的
/// 条目也还在圆括号里。这一份按行逐字比：这两份文件是 rustfmt 出的，头两行就是这一种写法，
/// 别的写法（`mod tests` 后面不空格就接左花括号、写在一行、`#[ cfg ( test ) ]`、多空格、缩进、行尾带空白或注释、CRLF 换行）一律判红，是故意拒。
///
/// # Panics
/// 照上面那样认的头两行不是恰好一处，或它之后没有收尾的顶格右括号行。
pub(super) fn text_outside_the_tests_module(source: &str) -> String {
    let lines: Vec<&str> = source.split('\n').collect();
    let read_lines = lines_read_past_comments_and_literals(&lines);
    let (attribute_line, module_line) = TESTS_MODULE_OPENING
        .split_once('\n')
        .expect("测试模块的头两行是两行");
    let code_before_the_line = |line_index: usize| -> String {
        read_lines[..line_index]
            .iter()
            .map(|read_line| read_line.without_comments_and_literals.as_str())
            .collect::<Vec<&str>>()
            .join("\n")
    };
    let openings: Vec<usize> = read_lines
        .windows(2)
        .enumerate()
        .filter(|(line_index, pair)| {
            pair[0].without_comments_and_literals == attribute_line
                && pair[1].without_comments_and_literals == module_line
                && pair[0].brace_depth_at_the_start == 0
                && parenthesis_and_bracket_depth_at_the_end_of(&code_before_the_line(*line_index))
                    == 0
                && {
                    let code_before_the_attribute_line = code_before_the_line(*line_index);
                    let last_code = code_before_the_attribute_line.trim_end();
                    last_code.is_empty() || last_code.ends_with([';', '\u{7d}', ']'])
                }
        })
        .map(|(line_index, _pair)| line_index)
        .collect();
    assert_eq!(
            openings.len(),
            1,
            "代码里测试模块的头两行恰好一处（两整行逐字、括号第 0 层、前一个词是条目收尾；注释、字符串、宏调用括号里的不算），读到的行号（从 0 数）：{openings:?}"
        );
    let opening = openings[0];
    let closing = opening
        + 1
        + read_lines[opening + 1..]
            .iter()
            .position(|read_line| {
                read_line.brace_depth_at_the_start == 1
                    && read_line.without_comments_and_literals.trim_end()
                        == TOP_LEVEL_CLOSING_BRACE_LINE.trim()
            })
            .expect("测试模块在顶格的右括号行收尾");
    lines[..opening]
        .iter()
        .chain(&lines[closing + 1..])
        .copied()
        .collect::<Vec<&str>>()
        .join("\n")
}

/// `code`（注释与字面量已换成空格）读到结尾时还没收尾的圆括号与方括号有几层。
///
/// # Panics
/// 右圆括号、右方括号多于左的。
fn parenthesis_and_bracket_depth_at_the_end_of(code: &str) -> usize {
    code.chars().fold(0_usize, |depth, character| {
        if matches!(character, '(' | '[') {
            depth + 1
        } else if matches!(character, ')' | ']') {
            depth.checked_sub(1).expect("右圆括号、右方括号不多于左的")
        } else {
            depth
        }
    })
}

/// `line` 里 `stop_is_requested` 作为整个名字出现了几次（前后不连着字母、数字、下划线：`stop_is_requested_by_the_environment`、
/// `_stop_is_requested` 不算）。
fn whole_name_uses_of_the_stop_request(line: &str) -> usize {
    whole_word_positions(line, STOP_REQUEST_PARAMETER_NAME).len()
}

/// 收叫停的那几层给叫停参数起的名字。
const STOP_REQUEST_PARAMETER_NAME: &str = concat!("stop_is_", "requested");

/// `text` 里 `word` 作为整个名字出现的每一处起点（前后不连着字母、数字、下划线）。
fn whole_word_positions(text: &str, word: &str) -> Vec<usize> {
    let is_part_of_a_name = |neighbour: Option<char>| {
        neighbour.is_some_and(|neighbour_character| {
            neighbour_character.is_alphanumeric() || neighbour_character == '_'
        })
    };
    text.match_indices(word)
        .filter(|(position, _word)| {
            !is_part_of_a_name(text[..*position].chars().next_back())
                && !is_part_of_a_name(text[position + word.len()..].chars().next())
        })
        .map(|(position, _word)| position)
        .collect()
}

/// 叫停参数的类型，去掉空白。
const STOP_REQUEST_TYPE_WITHOUT_SPACES: &str = "&(dynFn()->bool+Sync)";

/// `text` 跳过空白读下去恰以 `pattern_without_spaces` 起头时，那一截（连同中间的空白）收尾之后的位置；不是就交 `None`。
fn end_of_a_prefix_ignoring_whitespace(text: &str, pattern_without_spaces: &str) -> Option<usize> {
    let mut pattern_characters = pattern_without_spaces.chars().peekable();
    for (offset, character) in text.char_indices() {
        let Some(expected_character) = pattern_characters.peek() else {
            return Some(offset);
        };
        if character.is_whitespace() {
            continue;
        }
        if character != *expected_character {
            return None;
        }
        pattern_characters.next();
    }
    pattern_characters.peek().is_none().then_some(text.len())
}

/// 源码里写着叫停类型的一处（参数、`let` 的类型标注、字段都算）。
struct PlaceTakingTheStopRequest<'code> {
    /// 往前最近的 `fn` 后面那个名字（往前没有 `fn` 的交空串）。
    layer: &'code str,
    /// 类型前面 `名字:` 的那个名字（前面不是 `名字:` 的交空串）。
    parameter: &'code str,
    /// 参数名前面紧挨着的是不是属性的右方括号（`#[cfg(any())] stop_is_requested: …`）。
    is_the_parameter_under_an_attribute: bool,
    /// 类型起头（`&`）的位置。
    start_of_the_type: usize,
    /// 类型收尾之后的位置。
    end_of_the_type: usize,
}

/// `code`（注释与字面量已换成空格）里每一处叫停类型，照源码次序：类型里的空白随意、可以跨行，参数与别的参数挤在签名那一行、
/// 参数换了名字的都找得到。
fn places_taking_the_stop_request(code: &str) -> Vec<PlaceTakingTheStopRequest<'_>> {
    let is_identifier_character = |character: char| character.is_alphanumeric() || character == '_';
    let identifier_at_the_end_of = |text: &str| -> usize {
        text.char_indices()
            .rev()
            .take_while(|(_index, character)| is_identifier_character(*character))
            .last()
            .map_or(text.len(), |(index, _character)| index)
    };
    code.match_indices('&')
        .filter_map(|(ampersand, _ampersand)| {
            let end_of_the_type = ampersand
                + end_of_a_prefix_ignoring_whitespace(
                    &code[ampersand..],
                    STOP_REQUEST_TYPE_WITHOUT_SPACES,
                )?;
            let before_the_type = code[..ampersand].trim_end();
            let before_the_colon = before_the_type
                .strip_suffix(':')
                .filter(|before_the_colon| !before_the_colon.ends_with(':'))
                .map(str::trim_end);
            let parameter = before_the_colon.map_or("", |before_the_name| {
                &before_the_name[identifier_at_the_end_of(before_the_name)..]
            });
            let is_the_parameter_under_an_attribute =
                before_the_colon.is_some_and(|before_the_name| {
                    before_the_name[..identifier_at_the_end_of(before_the_name)]
                        .trim_end()
                        .ends_with(']')
                });
            let layer = whole_word_positions(&code[..ampersand], "fn")
                .last()
                .map_or("", |fn_keyword| {
                    let after_the_keyword = code[fn_keyword + 2..ampersand].trim_start();
                    let name_length = after_the_keyword
                        .find(|character: char| !is_identifier_character(character))
                        .unwrap_or(after_the_keyword.len());
                    &after_the_keyword[..name_length]
                });
            Some(PlaceTakingTheStopRequest {
                layer,
                parameter,
                is_the_parameter_under_an_attribute,
                start_of_the_type: ampersand,
                end_of_the_type,
            })
        })
        .collect()
}

/// 叫停那种闭包的形（不问参数）到返回类型之前那一截，去掉空白；`FnMut`、`FnOnce` 与函数指针 `fn() -> …` 也算。返回类型交 `bool`
/// 的才算叫停（[`end_of_a_bool_return_type`]）。
const STOP_REQUEST_SHAPES_UP_TO_THE_RETURN_TYPE_WITHOUT_SPACES: [&str; 4] =
    ["Fn()->", "FnMut()->", "FnOnce()->", "fn()->"];

/// 叫停那种闭包交的 `bool` 的写法，去掉空白：按路径写到 `core`、`std` 里那个原始类型的也算（N01）。
const WAYS_TO_WRITE_BOOL_WITHOUT_SPACES: [&str; 5] = [
    "bool",
    "::core::primitive::bool",
    "core::primitive::bool",
    "::std::primitive::bool",
    "std::primitive::bool",
];

/// `text`（叫停那种闭包的形 `->` 之后那一截）以交 `bool` 的返回类型起头时，那个 `bool` 收尾之后的位置；不是就交 `None`。
/// 空白随意，外面套几层括号的（`(bool)` N02）、按路径写的（[`WAYS_TO_WRITE_BOOL_WITHOUT_SPACES`]）都算，
/// 后面紧跟标识符的字（`boolean`）不算。
fn end_of_a_bool_return_type(text: &str) -> Option<usize> {
    let is_identifier_character = |character: char| character.is_alphanumeric() || character == '_';
    let after_the_parentheses =
        text.trim_start_matches(|character: char| character == '(' || character.is_whitespace());
    let parentheses_length = text.len() - after_the_parentheses.len();
    WAYS_TO_WRITE_BOOL_WITHOUT_SPACES
        .iter()
        .find_map(|way_to_write_bool| {
            let bool_length =
                end_of_a_prefix_ignoring_whitespace(after_the_parentheses, way_to_write_bool)?;
            (!after_the_parentheses[bool_length..]
                .chars()
                .next()
                .is_some_and(is_identifier_character))
            .then_some(parentheses_length + bool_length)
        })
}

/// `code`（注释与字面量已换成空格）里写着叫停那种闭包的形、却不在 `places` 哪一处钉住的类型里的地方，一处一句（往前最近的 `fn`
/// 后面那个名字连同那一行）：叫停写成 `&(dyn Fn() -> bool + Sync)` 之外的类型（多加 `Send`、`'static`，`&mut`、`&'static`，
/// `Box<dyn …>`，`dyn Sync + Fn() -> bool`，`&dyn Fn() -> bool`，`impl`、泛型、被扫的那一份源文件里的类型别名）收叫停的层，按钉住的类型找不到，
/// 照这个形找得到。
fn stop_request_shapes_outside_the_pinned_type(
    code: &str,
    places: &[PlaceTakingTheStopRequest<'_>],
) -> Vec<String> {
    let is_identifier_character = |character: char| character.is_alphanumeric() || character == '_';
    let mut found = Vec::new();
    for shape in STOP_REQUEST_SHAPES_UP_TO_THE_RETURN_TYPE_WITHOUT_SPACES {
        let first_word = &shape[..shape.find('(').expect("形里有左括号")];
        for position in whole_word_positions(code, first_word) {
            let Some(shape_length) = end_of_a_prefix_ignoring_whitespace(&code[position..], shape)
            else {
                continue;
            };
            if end_of_a_bool_return_type(&code[position + shape_length..]).is_none() {
                continue;
            }
            if places
                .iter()
                .any(|place| (place.start_of_the_type..place.end_of_the_type).contains(&position))
            {
                continue;
            }
            let layer =
                whole_word_positions(&code[..position], "fn")
                    .last()
                    .map_or("", |fn_keyword| {
                        let after_the_keyword = code[fn_keyword + 2..position].trim_start();
                        let name_length = after_the_keyword
                            .find(|character: char| !is_identifier_character(character))
                            .unwrap_or(after_the_keyword.len());
                        &after_the_keyword[..name_length]
                    });
            let line_start = code[..position]
                .rfind('\n')
                .map_or(0, |newline| newline + 1);
            let line_end = code[position..]
                .find('\n')
                .map_or(code.len(), |newline| position + newline);
            found.push(format!(
                    "{layer} 往后有一处叫停写成钉住的 &(dyn Fn() -> bool + Sync) 之外的类型，按钉住的类型找不到这一层：{}",
                    code[line_start..line_end].trim()
                ));
        }
    }
    found
}

/// 没有行为用例的那几层，扫源码钉住它们怎么用收到的叫停（`every_layer_handed_the_stop_request_hands_it_on_or_asks_it`）。
enum StopRequestUseThatIsPinned {
    /// 函数体短：参数行之后到函数收尾（返回类型与函数体，去掉注释行与空白）整个等于这一句。
    WholeBodyAfterTheParameter(&'static str),
    /// 函数体太长、整句比不了：用到叫停整名的只有 `line` 这一行（去掉首尾空白），它是函数体顶层的一句；
    /// `binding` 在这一层只由它 `let` 绑一次，`type_name` 在这一层只出现在它里面。
    TopLevelStatementOfALongBody {
        line: &'static str,
        binding: &'static str,
        type_name: &'static str,
    },
}

/// 数花括号时，读到的这个字在源码的哪一种地方（跨行时带到下一行）。
#[derive(Clone, Copy)]
enum PlaceInTheSource {
    Code,
    InsideAString,
    /// 原始字符串（`r"…"`、`r#"…"#`、`br"…"`、`cr"…"`）：收尾是一个 `"` 后面跟这么多个 `#`。
    InsideARawString {
        closing_hashes: usize,
    },
    /// 块注释，可以嵌套：`nesting` 是还没收尾的 `/*` 有几层。
    InsideABlockComment {
        nesting: usize,
    },
}

/// 读过的一个字算源码里的哪一类（`lines_read_past_comments_and_literals`）。
#[derive(Clone, Copy)]
enum KindOfACharacterInTheSource {
    Code,
    /// 行注释、块注释里的字，连同 `//`、`/*`、`*/`。
    Comment,
    /// 字符串（普通的、原始的）与字符字面量里的字，连同引号与原始字符串的 `r`、`br`、`cr` 和 `#`。
    Literal,
}

/// 一行读过注释与字面量之后的样子（`lines_read_past_comments_and_literals`）。
pub(super) struct LineReadPastCommentsAndLiterals {
    /// 这一行起头处在第几层花括号里。
    brace_depth_at_the_start: usize,
    /// 注释里的字换成空格（一个字换成与它的 UTF-8 字节数一样多的空格，字节位置不变），字面量照原样。
    pub(super) without_comments: String,
    /// 注释与字面量里的字都换成空格，字节位置不变：扫源码的用例照它找名字、数 `let`、数花括号，注释与字符串里写的不算。
    without_comments_and_literals: String,
}

/// `lines` 每一行起头处在第几层花括号里（第一行起头算第 0 层）：字符串（普通的、原始的）、字符字面量、行注释与块注释（可以嵌套）里的
/// 花括号不算，字符串、块注释跨行时照跨着数。
///
/// # Panics
/// 右花括号多于左花括号。
pub(super) fn brace_depth_at_the_start_of_each_line(lines: &[&str]) -> Vec<usize> {
    lines_read_past_comments_and_literals(lines)
        .iter()
        .map(|read_line| read_line.brace_depth_at_the_start)
        .collect()
}

/// `lines` 逐行读过注释与字面量：每一行起头处的花括号层数，连同去掉注释、去掉注释与字面量的两份（字节位置与原行相同）。
/// 字符串（普通的、原始的 `r"…"`、`r#"…"#`、`br`、`cr`）、字符字面量、行注释与块注释（可以嵌套）里的花括号不算，
/// 字符串、块注释跨行时照跨着读。与卡池扫源码那一份分词（`gpu_unit_checks.rs` 的 `rust_tokens_without_comments_and_literals`）
/// 认原始字符串的写法相同：`r`、`br`、`cr` 后面紧跟若干个 `#` 与一个 `"`。
///
/// # Panics
/// 右花括号多于左花括号。
pub(super) fn lines_read_past_comments_and_literals(
    lines: &[&str],
) -> Vec<LineReadPastCommentsAndLiterals> {
    let is_identifier_character = |character: char| character.is_alphanumeric() || character == '_';
    let mut depth = 0_usize;
    let mut place = PlaceInTheSource::Code;
    let mut read_lines = Vec::with_capacity(lines.len());
    for line in lines {
        let brace_depth_at_the_start = depth;
        let characters: Vec<char> = line.chars().collect();
        let mut kinds = vec![KindOfACharacterInTheSource::Code; characters.len()];
        let mut index = 0;
        while index < characters.len() {
            let character = characters[index];
            let next_character = characters.get(index + 1).copied();
            let read_from = index;
            let place_before = place;
            match place {
                PlaceInTheSource::InsideAString => {
                    if character == '\\' {
                        index += 1;
                    } else if character == '"' {
                        place = PlaceInTheSource::Code;
                    }
                }
                PlaceInTheSource::InsideARawString { closing_hashes } => {
                    let closing_hashes_follow = characters
                        .get(index + 1..index + 1 + closing_hashes)
                        .is_some_and(|after_the_quote| {
                            after_the_quote.iter().all(|after| *after == '#')
                        });
                    if character == '"' && closing_hashes_follow {
                        index += closing_hashes;
                        place = PlaceInTheSource::Code;
                    }
                }
                PlaceInTheSource::InsideABlockComment { nesting } => {
                    if character == '/' && next_character == Some('*') {
                        index += 1;
                        place = PlaceInTheSource::InsideABlockComment {
                            nesting: nesting + 1,
                        };
                    } else if character == '*' && next_character == Some('/') {
                        index += 1;
                        place = match nesting.checked_sub(1) {
                            Some(0) | None => PlaceInTheSource::Code,
                            Some(nesting_left) => PlaceInTheSource::InsideABlockComment {
                                nesting: nesting_left,
                            },
                        };
                    }
                }
                PlaceInTheSource::Code => {
                    if character == '"' {
                        place = PlaceInTheSource::InsideAString;
                    } else if character == '\'' {
                        // 字符字面量（`'x'`、`'\''`、`'\u{7b}'`）整个跳过；生命周期（`'static`）只跳这个引号
                        if next_character == Some('\\') {
                            index += 2;
                            while characters.get(index + 1).is_some_and(|next| *next != '\'') {
                                index += 1;
                            }
                            index += 1;
                        } else if characters.get(index + 2) == Some(&'\'') {
                            index += 2;
                        }
                    } else if character == '/' && next_character == Some('/') {
                        kinds[index..].fill(KindOfACharacterInTheSource::Comment);
                        break;
                    } else if character == '/' && next_character == Some('*') {
                        index += 1;
                        place = PlaceInTheSource::InsideABlockComment { nesting: 1 };
                    } else if is_identifier_character(character) {
                        // 一整个词跳过；词是 r、br、cr 而后面紧跟若干个 # 与一个 " 的，是原始字符串的开头
                        let word_end = characters[index..]
                            .iter()
                            .position(|after| !is_identifier_character(*after))
                            .map_or(characters.len(), |word_length| index + word_length);
                        let word: String = characters[index..word_end].iter().collect();
                        let opening_hashes = characters[word_end..]
                            .iter()
                            .take_while(|after| **after == '#')
                            .count();
                        if matches!(word.as_str(), "r" | "br" | "cr")
                            && characters.get(word_end + opening_hashes) == Some(&'"')
                        {
                            index = word_end + opening_hashes;
                            place = PlaceInTheSource::InsideARawString {
                                closing_hashes: opening_hashes,
                            };
                        } else {
                            index = word_end - 1;
                        }
                    } else if character == '\u{7b}' {
                        depth += 1;
                    } else if character == '\u{7d}' {
                        depth = depth.checked_sub(1).expect("右花括号不多于左花括号");
                    }
                }
            }
            index += 1;
            let kind_read = match place_before {
                PlaceInTheSource::InsideAString | PlaceInTheSource::InsideARawString { .. } => {
                    KindOfACharacterInTheSource::Literal
                }
                PlaceInTheSource::InsideABlockComment { .. } => {
                    KindOfACharacterInTheSource::Comment
                }
                PlaceInTheSource::Code => match place {
                    PlaceInTheSource::InsideAString | PlaceInTheSource::InsideARawString { .. } => {
                        KindOfACharacterInTheSource::Literal
                    }
                    PlaceInTheSource::InsideABlockComment { .. } => {
                        KindOfACharacterInTheSource::Comment
                    }
                    // 一次读过不止一个字的引号是整个字符字面量；生命周期只读过那一个引号
                    PlaceInTheSource::Code if character == '\'' && index > read_from + 1 => {
                        KindOfACharacterInTheSource::Literal
                    }
                    PlaceInTheSource::Code => KindOfACharacterInTheSource::Code,
                },
            };
            kinds[read_from..index.min(characters.len())].fill(kind_read);
        }
        let mut without_comments = String::with_capacity(line.len());
        let mut without_comments_and_literals = String::with_capacity(line.len());
        for (character, kind) in characters.iter().zip(&kinds) {
            let spaces = " ".repeat(character.len_utf8());
            match kind {
                KindOfACharacterInTheSource::Code => {
                    without_comments.push(*character);
                    without_comments_and_literals.push(*character);
                }
                KindOfACharacterInTheSource::Comment => {
                    without_comments.push_str(&spaces);
                    without_comments_and_literals.push_str(&spaces);
                }
                KindOfACharacterInTheSource::Literal => {
                    without_comments.push(*character);
                    without_comments_and_literals.push_str(&spaces);
                }
            }
        }
        read_lines.push(LineReadPastCommentsAndLiterals {
            brace_depth_at_the_start,
            without_comments,
            without_comments_and_literals,
        });
    }
    read_lines
}

/// `family`（崩溃放量模块族编进库的那几份，照文件表的次序：[`library_files_of_the_family`]，或用例在其中一份上改过的）测试部分之外，
/// 收叫停的层没照钉住的那样往下交或问的地方，一处一句（前面带那一份的路径）；一处没有交空。逐份读
/// （[`ways_a_layer_in_one_file_handed_the_stop_request_could_drop_it`]），收叫停的层照文件表的次序一份接一份、每份里照源码次序列成名单，
/// 与钉住的名单整个比。判法与名单在 `every_layer_handed_the_stop_request_hands_it_on_or_asks_it`。
pub(super) fn ways_a_layer_handed_the_stop_request_could_drop_it(
    family: &[(&str, &str)],
) -> Vec<String> {
    let mut found = Vec::new();
    let mut layers_read_from_the_files: Vec<(String, String)> = Vec::new();
    for (path, source) in family {
        let (found_in_the_file, layers_in_the_file) =
            ways_a_layer_in_one_file_handed_the_stop_request_could_drop_it(
                &text_outside_the_tests_of_a_family_file(path, source),
            );
        found.extend(
            found_in_the_file
                .into_iter()
                .map(|finding| format!("{path}：{finding}")),
        );
        layers_read_from_the_files.extend(layers_in_the_file);
    }
    let layers_handed_the_stop_request: Vec<(&str, &str)> = layers_read_from_the_files
        .iter()
        .map(|(layer, parameter)| (layer.as_str(), parameter.as_str()))
        .collect();
    // 照文件表的次序（路径的字节序）一份接一份，每份里照源码次序
    let expected_layers = [
        ("check_blocks_where_until", STOP_REQUEST_PARAMETER_NAME),
        (
            "check_blocks_where_until_with_threads",
            STOP_REQUEST_PARAMETER_NAME,
        ),
        (
            "check_blocks_where_until_judging_each_piece_with",
            STOP_REQUEST_PARAMETER_NAME,
        ),
        (
            "check_pending_blocks_sharing_one_enumeration_table_of_the_flow",
            STOP_REQUEST_PARAMETER_NAME,
        ),
        (
            "check_pending_blocks_in_pieces",
            STOP_REQUEST_PARAMETER_NAME,
        ),
        (
            "verify_my_blocks_through_the_facts_verifier_pending_rows",
            STOP_REQUEST_PARAMETER_NAME,
        ),
        ("verify_my_blocks_on_gpu", STOP_REQUEST_PARAMETER_NAME),
        ("verify_my_blocks_on_gpu", concat!("_stop_is_", "requested")),
        ("judge_my_blocks_on_gpu_cards", STOP_REQUEST_PARAMETER_NAME),
        (
            "judge_my_blocks_on_the_cards_started_for_this_run",
            STOP_REQUEST_PARAMETER_NAME,
        ),
        (
            "wait_for_the_verdict_words_of_the_packed_tables",
            STOP_REQUEST_PARAMETER_NAME,
        ),
        (
            "judge_packed_tables_through_the_gpu_verification_server",
            STOP_REQUEST_PARAMETER_NAME,
        ),
        (
            "judge_my_blocks_through_the_pending_rows",
            STOP_REQUEST_PARAMETER_NAME,
        ),
        ("judge_my_blocks_on_gpu", STOP_REQUEST_PARAMETER_NAME),
        ("judge_my_blocks_on_gpu", concat!("_stop_is_", "requested")),
        (
            "judge_the_observers_then_the_reuse_sentinels",
            STOP_REQUEST_PARAMETER_NAME,
        ),
        (
            "record_the_observers_of_the_flow",
            STOP_REQUEST_PARAMETER_NAME,
        ),
        (
            "record_the_observers_of_the_flow_with_threads",
            STOP_REQUEST_PARAMETER_NAME,
        ),
        (
            "record_the_observers_of_the_forced_block_again",
            STOP_REQUEST_PARAMETER_NAME,
        ),
        (
            "record_the_observers_kept_out_of_the_library_again",
            STOP_REQUEST_PARAMETER_NAME,
        ),
        ("check_selected_blocks", STOP_REQUEST_PARAMETER_NAME),
    ];
    if layers_handed_the_stop_request != expected_layers {
        found.push(format!(
                "收叫停的是这二十层（CPU 判段判一段交调用方给的判器那一层，观察者的录入段两层，取核对线程数那一层原样交下去，录入那一层每录一块问一次；复用哨兵抽中的块回录入段重录一层，原样交给录入段；判段补录照判不入库的观察者一层，原样交给录入段；观察者与哨兵的判段一层，原样交给补录那一层；写待判行、等 GPU 批处理服务判完那一层，每现算一遍问一次，与它的入口 judge_my_blocks_on_gpu；没交卡的判段写判表待判行、等 GPU 批处理服务写回判表结论字那一层，每现算一遍问一次，与登记临时库、包它的那一层，原样交下去；第 ② 段写核对待判行、等服务核完那一层，每现算一遍问一次，与它的入口 verify_my_blocks_on_gpu），连同没编进 gpu 特性时那两份 judge_my_blocks_on_gpu、verify_my_blocks_on_gpu（多一层少一层、参数换了名字都要回来看这条用例罩没罩到），照文件表的次序一份接一份、每份里照源码次序，读到的是 {layers_handed_the_stop_request:?}"
            ));
    }
    found
}

/// 一份源文件测试部分之外的文本（`outside_the_tests_of_the_file`，[`text_outside_the_tests_of_a_family_file`] 交的）里，
/// 收叫停的层没照钉住的那样往下交或问的地方（一处一句），连同照源码次序读到的收叫停的层（层名、叫停参数名）。
fn ways_a_layer_in_one_file_handed_the_stop_request_could_drop_it(
    outside_the_tests_of_the_file: &str,
) -> (Vec<String>, Vec<(String, String)>) {
    let pinned_layers_without_a_behaviour_test: [(&str, &str, StopRequestUseThatIsPinned); 9] = [
            (
                "judge_my_blocks_on_gpu_cards",
                STOP_REQUEST_PARAMETER_NAME,
                StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
                    ")->Result<(CheckingTally,GpuJudgeOutcome),String>\u{7b}",
                    "lettables=crate::crash_library::record_the_judge_tables_and_read_them_back(store,&packed_judge_tables_of_the_plan(flow,plan)?,)?;",
                    "judge_my_blocks_on_the_cards_started_for_this_run(",
                    "&LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(store),flow,plan,&tables,is_mine,&ConclusionsStoredUnder::TheJudgingUnitsFingerprint(version),cards,stop_is_",
                    "requested,)"
                )),
            ),
            (
                "judge_my_blocks_on_the_cards_started_for_this_run",
                STOP_REQUEST_PARAMETER_NAME,
                StopRequestUseThatIsPinned::TopLevelStatementOfALongBody {
                    line: concat!(
                        "let stop_request = StopRequestRememberedInThisRun::new(stop_is_",
                        "requested);"
                    ),
                    binding: "stop_request",
                    type_name: "StopRequestRememberedInThisRun",
                },
            ),
            (
                "check_selected_blocks",
                STOP_REQUEST_PARAMETER_NAME,
                StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
                    ")->Result<CheckingTally,VerdictStoreError>\u{7b}",
                    "letthreads=checking_threads();",
                    "check_blocks_where_until_with_threads(library,flow,plan,",
                    "&|block_sequence|selected_block_sequences.contains(&block_sequence),",
                    "conclusions,stop_is_",
                    "requested,threads,)"
                )),
            ),
            (
                "record_the_observers_of_the_flow",
                STOP_REQUEST_PARAMETER_NAME,
                StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
                    ")->Result<ObserverRecordingTally,String>\u{7b}",
                    "letthreads=checking_threads();",
                    "record_the_observers_of_the_flow_with_threads(library,flow,plan,observers,versions,is_mine,stop_is_",
                    "requested,threads,)"
                )),
            ),
            (
                "record_the_observers_of_the_forced_block_again",
                STOP_REQUEST_PARAMETER_NAME,
                StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
                    ")->Result<bool,String>\u{7b}",
                    "letSome((forced,_stored_before_judging))=&sentinels.forcedelse\u{7b}returnOk(false);\u{7d};",
                    "ifobserver_inputs_of_the_block_are_stored(&library.take(),&observers_for_the_reuse_sentinels(plan,observers,versions),forced.point_index,&forced.key.0,)",
                    ".map_err(|error|format!(\"复用哨兵：读强制现判那一块的录入段失败：\u{7b}error:?\u{7d}\"))?\u{7b}returnOk(false);\u{7d}",
                    "letforced_block=forced.block_sequence;",
                    "record_the_observers_of_the_flow(library,flow,plan,observers,versions,&|block|block==forced_block,stop_is_",
                    "requested,)?;Ok(true)"
                )),
            ),
            (
                "judge_my_blocks_on_gpu",
                STOP_REQUEST_PARAMETER_NAME,
                StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
                    ")->Result<(CheckingTally,GpuJudgeOutcome),String>\u{7b}lettemporary_library=temporary_library_of_the_judging_stage_registered_with_the_real_service(library,PendingRowFamilyOfTheGpuVerificationServer::ImageJudgement,)?;",
                    "letwaiting_started=Instant::now();",
                    "lettally=judge_my_blocks_through_the_pending_rows(library,plan,recorded_flow_digest,&sha256_digest(&tables.to_bytes()),is_mine,conclusions,&WaitingForTheGpuVerificationServer\u{7b}progress_interval:GPU_VERIFICATION_SERVER_WAITING_PROGRESS_INTERVAL,between_two_reckonings:&|_library|\u{7b}std::thread::sleep(GPU_VERIFICATION_SERVER_RECKONING_INTERVAL);",
                    "\u{7d},\u{7d},stop_is_",
                    "requested,)?;drop(temporary_library);",
                    "letoutcome=GpuJudgeOutcome\u{7b}cards_used:0,blocks_judged:tally.blocks_checked,states_judged:tally.states_checked,red_states:tally.red_states,compile_seconds:0.0,judge_seconds:waiting_started.elapsed().as_secs_f64(),seconds_storing:0.0,seconds_decoding:0.0,cards:Vec::new(),\u{7d};",
                    "Ok((tally,outcome))"
                )),
            ),
            (
                "judge_my_blocks_on_gpu",
                concat!("_stop_is_", "requested"),
                StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
                    ")->Result<(CheckingTally,GpuJudgeOutcome),String>\u{7b}",
                    "Err(\"配置写了ENABLE_GPU=1，这份二进制没有编进gpu特性（cargo…--featuresgpu）\".to_string())"
                )),
            ),
            (
                "verify_my_blocks_on_gpu",
                STOP_REQUEST_PARAMETER_NAME,
                StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
                    ")->Result<VerificationTally,String>\u{7b}lettemporary_library=temporary_library_of_the_judging_stage_registered_with_the_real_service(library,PendingRowFamilyOfTheGpuVerificationServer::FactsVerifier,)?;",
                    "lettally=verify_my_blocks_through_the_facts_verifier_pending_rows(library,plan,recorded_flow_digest,facts_digest,is_mine,&WaitingForTheGpuVerificationServer\u{7b}progress_interval:GPU_VERIFICATION_SERVER_WAITING_PROGRESS_INTERVAL,between_two_reckonings:&|_library|\u{7b}std::thread::sleep(GPU_VERIFICATION_SERVER_RECKONING_INTERVAL);",
                    "\u{7d},\u{7d},stop_is_",
                    "requested,)?;drop(temporary_library);Ok(tally)"
                )),
            ),
            (
                "verify_my_blocks_on_gpu",
                concat!("_stop_is_", "requested"),
                StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
                    ")->Result<VerificationTally,String>\u{7b}",
                    "Err(\"配置写了ENABLE_GPU=1，这份二进制没有编进gpu特性（cargo…--featuresgpu）\".to_string())"
                )),
            ),
        ];
    let outside_the_tests = format!("{outside_the_tests_of_the_file}\n");
    let outside_the_tests_lines: Vec<&str> = outside_the_tests.split('\n').collect();
    let read_lines = lines_read_past_comments_and_literals(&outside_the_tests_lines);
    // 两份与 outside_the_tests 逐字节同位：找名字、数 let 与花括号照 code（注释与字面量都不算），整句、整行比照 without_comments
    let without_comments = read_lines
        .iter()
        .map(|read_line| read_line.without_comments.as_str())
        .collect::<Vec<&str>>()
        .join("\n");
    let code = read_lines
        .iter()
        .map(|read_line| read_line.without_comments_and_literals.as_str())
        .collect::<Vec<&str>>()
        .join("\n");
    let mut found = Vec::new();
    let mut layers_handed_the_stop_request = Vec::new();
    let places = places_taking_the_stop_request(&code);
    for place in &places {
        let PlaceTakingTheStopRequest {
            layer,
            parameter,
            is_the_parameter_under_an_attribute,
            start_of_the_type: _,
            end_of_the_type,
        } = *place;
        if is_the_parameter_under_an_attribute {
            found.push(format!(
                    "{layer} 的叫停参数 {parameter} 前面挂着属性（#[cfg(any())] 让参数编译时去掉，函数体里的名字就落到别处同名的条目上）"
                ));
        }
        let layer_length = code[end_of_the_type..]
            .find(TOP_LEVEL_CLOSING_BRACE_LINE)
            .unwrap_or(code.len() - end_of_the_type);
        let rest_of_the_layer: Vec<&str> = without_comments
            [end_of_the_type..end_of_the_type + layer_length]
            .lines()
            .collect();
        let rest_of_the_layer_code: Vec<&str> = code
            [end_of_the_type..end_of_the_type + layer_length]
            .lines()
            .collect();
        match pinned_layers_without_a_behaviour_test
            .iter()
            .find(|(pinned_layer, pinned_parameter, _pinned)| {
                *pinned_layer == layer && *pinned_parameter == parameter
            })
            .map(|(_layer, _parameter, pinned)| pinned)
        {
            Some(StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(expected_body)) => {
                let body_without_spaces: String = rest_of_the_layer
                    .iter()
                    .flat_map(|line| line.chars())
                    .filter(|character| !character.is_whitespace())
                    .collect();
                // 参数后面的逗号（参数单独占一行时有）不算函数体
                let body_after_the_parameter = body_without_spaces
                    .strip_prefix(',')
                    .unwrap_or(&body_without_spaces);
                if body_after_the_parameter != *expected_body {
                    found.push(format!(
                            "{layer} 参数之后整句等于钉住的那一句（死分支里留一份原样的调用、真调用换实参，都多出或换掉一截）：{rest_of_the_layer:#?}"
                        ));
                }
            }
            Some(StopRequestUseThatIsPinned::TopLevelStatementOfALongBody {
                line: pinned_line,
                binding,
                type_name,
            }) => {
                let lines_using_it: Vec<(usize, &str)> = rest_of_the_layer_code
                    .iter()
                    .enumerate()
                    .filter(|(_offset, line)| whole_name_uses_of_the_stop_request(line) > 0)
                    .map(|(offset, _line)| (offset, rest_of_the_layer[offset].trim()))
                    .collect();
                if lines_using_it
                    .iter()
                    .map(|(_offset, line)| *line)
                    .collect::<Vec<&str>>()
                    != [*pinned_line]
                {
                    found.push(format!(
                            "{layer} 用到叫停的只有钉住的那一行、原样（只提一次名字而永不叫停的写法多一行或那一行变了）：{rest_of_the_layer:#?}"
                        ));
                }
                let depths = brace_depth_at_the_start_of_each_line(&rest_of_the_layer_code);
                for (pinned_offset, _line) in &lines_using_it {
                    if depths[*pinned_offset] != 1 {
                        found.push(format!(
                                "{layer} 的 {pinned_line} 是函数体顶层的一句，不在 `if false` 一类的块里（参数之后那一行开函数体，算第 1 层），读到的是第 {} 层",
                                depths[*pinned_offset]
                            ));
                    }
                    let code_before_the_pinned_line =
                        rest_of_the_layer_code[..*pinned_offset].join("\n");
                    if !code_before_the_pinned_line
                        .trim_end()
                        .ends_with([';', '\u{7b}', '\u{7d}'])
                    {
                        found.push(format!(
                                "{layer} 的 {pinned_line} 前面紧挨着的是上一句的收尾（分号、左右花括号），不是属性：#[cfg(any())] 挂在它上面编译时整句去掉，下面用的 {binding} 就落到别处同名的条目上"
                            ));
                    }
                }
                let rest_of_the_layer_text = rest_of_the_layer_code.join("\n");
                let let_bindings = whole_word_positions(&rest_of_the_layer_text, "let")
                    .into_iter()
                    .filter(|let_keyword| {
                        let after_the_keyword = &rest_of_the_layer_text[let_keyword + 3..];
                        let pattern_end = after_the_keyword
                            .find(['=', ';'])
                            .unwrap_or(after_the_keyword.len());
                        !whole_word_positions(&after_the_keyword[..pattern_end], binding).is_empty()
                    })
                    .count();
                if let_bindings != 1 {
                    found.push(format!(
                            "{layer} 里 {binding} 只由钉住的那一行 let 绑一次（后面再 let 一次、解构、if let 就换掉了记住的叫停），读到 {let_bindings} 次"
                        ));
                }
                let lines_naming_the_type = rest_of_the_layer_code
                    .iter()
                    .filter(|line| line.contains(type_name))
                    .count();
                if lines_naming_the_type != 1 {
                    found.push(format!(
                            "{layer} 里只有钉住的那一行提到 {type_name}（块里 use 或另起同名的类型会遮住它），读到 {lines_naming_the_type} 行"
                        ));
                }
            }
            None => {}
        }
        layers_handed_the_stop_request.push((layer, parameter));
    }
    found.extend(stop_request_shapes_outside_the_pinned_type(&code, &places));
    let layers_in_the_file = layers_handed_the_stop_request
        .into_iter()
        .map(|(layer, parameter)| (layer.to_owned(), parameter.to_owned()))
        .collect();
    (found, layers_in_the_file)
}

/// 崩溃放量模块族的根（路径相对 `src/`）：族里只有它把测试模块写在自己里面（[`TESTS_MODULE_OPENING`]）。
pub(super) const ROOT_OF_THE_FAMILY: &str = "crash_amplification.rs";

/// 子模块里声明测试模块的那两整行：子模块的测试写在 `<子模块>/tests.rs` 里，只给测试编。
const TESTS_MODULE_DECLARATION: &str = "#[cfg(test)]\nmod tests;";

/// 崩溃放量模块族里一份源文件归哪一种。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum RoleInTheFamily {
    /// 编进库：扫源码的用例扫它测试部分之外的那一截（[`text_outside_the_tests_of_a_family_file`]）。
    CompiledIntoTheLibrary,
    /// 只给测试编（`#[cfg(test)]` 声明的模块，连同它往下声明的）：各子模块的 `tests.rs`、几处测试共用的搭建、扫源码的读法（这一份）。扫源码的用例不扫。
    CompiledOnlyForTests,
}

/// 崩溃放量模块族的每一份源文件：路径（相对 `src/`）、归哪一种、源码（编进测试二进制）。照路径的字节序排，根在最前；
/// 扫源码的用例照这张表的次序读族里编进库的那几份（[`library_files_of_the_family`]）。表与根起逐层 `mod` 声明到的文件逐个相同、
/// 归的那一种也相同，由 `the_family_table_holds_every_file_the_module_declarations_reach_from_the_root` 核。
pub(super) static CRASH_AMPLIFICATION_FAMILY_FILES: &[(&str, RoleInTheFamily, &str)] = &[
    (
        "crash_amplification.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("../crash_amplification.rs"),
    ),
    (
        "crash_amplification/block_ranges_and_parts.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("block_ranges_and_parts.rs"),
    ),
    (
        "crash_amplification/block_ranges_and_parts/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("block_ranges_and_parts/tests.rs"),
    ),
    (
        "crash_amplification/block_recording_and_storage.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("block_recording_and_storage.rs"),
    ),
    (
        "crash_amplification/block_recording_and_storage/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("block_recording_and_storage/tests.rs"),
    ),
    (
        "crash_amplification/conclusion_versions_and_judging_units.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("conclusion_versions_and_judging_units.rs"),
    ),
    (
        "crash_amplification/conclusion_versions_and_judging_units/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("conclusion_versions_and_judging_units/tests.rs"),
    ),
    (
        "crash_amplification/conclusions_read_at_the_end.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("conclusions_read_at_the_end.rs"),
    ),
    (
        "crash_amplification/conclusions_read_at_the_end/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("conclusions_read_at_the_end/tests.rs"),
    ),
    (
        "crash_amplification/cpu_checking.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("cpu_checking.rs"),
    ),
    (
        "crash_amplification/cpu_checking/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("cpu_checking/tests.rs"),
    ),
    (
        "crash_amplification/crash_point_plan.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("crash_point_plan.rs"),
    ),
    (
        "crash_amplification/crash_point_plan/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("crash_point_plan/tests.rs"),
    ),
    (
        "crash_amplification/crash_point_recorder.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("crash_point_recorder.rs"),
    ),
    (
        "crash_amplification/crash_point_recorder/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("crash_point_recorder/tests.rs"),
    ),
    (
        "crash_amplification/facts_verification.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("facts_verification.rs"),
    ),
    (
        "crash_amplification/facts_verification/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("facts_verification/tests.rs"),
    ),
    (
        "crash_amplification/facts_verifier_pending_rows.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("facts_verifier_pending_rows.rs"),
    ),
    (
        "crash_amplification/facts_verifier_pending_rows/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("facts_verifier_pending_rows/tests.rs"),
    ),
    (
        "crash_amplification/given_states_engine.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("given_states_engine.rs"),
    ),
    (
        "crash_amplification/given_states_engine/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("given_states_engine/tests.rs"),
    ),
    (
        "crash_amplification/gpu_card_judging.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("gpu_card_judging.rs"),
    ),
    (
        "crash_amplification/gpu_card_judging/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("gpu_card_judging/tests.rs"),
    ),
    (
        "crash_amplification/gpu_pending_rows_and_packed_tables.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("gpu_pending_rows_and_packed_tables.rs"),
    ),
    (
        "crash_amplification/gpu_pending_rows_and_packed_tables/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("gpu_pending_rows_and_packed_tables/tests.rs"),
    ),
    (
        "crash_amplification/gpu_verification_server.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("gpu_verification_server.rs"),
    ),
    (
        "crash_amplification/gpu_verification_server/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("gpu_verification_server/tests.rs"),
    ),
    (
        "crash_amplification/images_after_the_observers.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("images_after_the_observers.rs"),
    ),
    (
        "crash_amplification/images_after_the_observers/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("images_after_the_observers/tests.rs"),
    ),
    (
        "crash_amplification/observer_definitions.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("observer_definitions.rs"),
    ),
    (
        "crash_amplification/observer_definitions/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("observer_definitions/tests.rs"),
    ),
    (
        "crash_amplification/observer_judging.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("observer_judging.rs"),
    ),
    (
        "crash_amplification/observer_judging/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("observer_judging/tests.rs"),
    ),
    (
        "crash_amplification/observer_recording.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("observer_recording.rs"),
    ),
    (
        "crash_amplification/observer_recording/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("observer_recording/tests.rs"),
    ),
    (
        "crash_amplification/observer_records.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("observer_records.rs"),
    ),
    (
        "crash_amplification/observer_records/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("observer_records/tests.rs"),
    ),
    (
        "crash_amplification/read_set_witnesses.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("read_set_witnesses.rs"),
    ),
    (
        "crash_amplification/read_set_witnesses/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("read_set_witnesses/tests.rs"),
    ),
    (
        "crash_amplification/recorded_flow.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("recorded_flow.rs"),
    ),
    (
        "crash_amplification/recorded_flow/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("recorded_flow/tests.rs"),
    ),
    (
        "crash_amplification/registered_segments_and_slices.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("registered_segments_and_slices.rs"),
    ),
    (
        "crash_amplification/registered_segments_and_slices/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("registered_segments_and_slices/tests.rs"),
    ),
    (
        "crash_amplification/reuse_sentinels.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("reuse_sentinels.rs"),
    ),
    (
        "crash_amplification/reuse_sentinels/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("reuse_sentinels/tests.rs"),
    ),
    (
        "crash_amplification/run_from_the_environment.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("run_from_the_environment.rs"),
    ),
    (
        "crash_amplification/run_from_the_environment/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("run_from_the_environment/tests.rs"),
    ),
    (
        "crash_amplification/run_modes_and_cards.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("run_modes_and_cards.rs"),
    ),
    (
        "crash_amplification/run_modes_and_cards/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("run_modes_and_cards/tests.rs"),
    ),
    (
        "crash_amplification/shared_test_fixtures.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("shared_test_fixtures.rs"),
    ),
    (
        "crash_amplification/source_reading_for_the_scans.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("source_reading_for_the_scans.rs"),
    ),
];

/// 族里路径为 `path` 的一份编进库的源文件（`source`，或用例在它上面改过的一份）去掉测试部分之后的文本。族的根
/// （[`ROOT_OF_THE_FAMILY`]）照 [`text_outside_the_tests_module`] 切掉写在它里面的测试模块；子模块的测试在 `<子模块>/tests.rs` 里、不扫，
/// 这里去掉的是声明它的那两整行（[`TESTS_MODULE_DECLARATION`]：两整行逐字、在代码里、花括号第 0 层），没有这两行的原样交回。
///
/// # Panics
/// 根照 [`text_outside_the_tests_module`] 那几条；子模块里声明测试模块的两整行不止一处。
pub(super) fn text_outside_the_tests_of_a_family_file(path: &str, source: &str) -> String {
    if path == ROOT_OF_THE_FAMILY {
        text_outside_the_tests_module(source)
    } else {
        let lines: Vec<&str> = source.split('\n').collect();
        let read_lines = lines_read_past_comments_and_literals(&lines);
        let (attribute_line, declaration_line) = TESTS_MODULE_DECLARATION
            .split_once('\n')
            .expect("声明测试模块的是两行");
        let declarations: Vec<usize> = read_lines
            .windows(2)
            .enumerate()
            .filter(|(_line_index, two_lines)| {
                two_lines[0].brace_depth_at_the_start == 0
                    && two_lines[0].without_comments_and_literals == attribute_line
                    && two_lines[1].without_comments_and_literals == declaration_line
            })
            .map(|(line_index, _pair)| line_index)
            .collect();
        assert!(
            declarations.len() <= 1,
            "{path} 里声明测试模块的两整行至多一处，读到的行号（从 0 数）：{declarations:?}"
        );
        lines
            .iter()
            .enumerate()
            .filter(|(line_index, _line)| {
                !declarations
                    .iter()
                    .any(|declaration| (*declaration..*declaration + 2).contains(line_index))
            })
            .map(|(_line_index, line)| *line)
            .collect::<Vec<&str>>()
            .join("\n")
    }
}

/// 族里一份源文件（路径 `path` 相对 `src/`，归 `role`，源码 `source`）里花括号第 0 层 `mod <名>;` 声明的子模块文件与它归的那一种：
/// 路径相对 `src/`，`<x>.rs` 声明的在 `<x>/` 下；`path` 只给测试编，或声明前面（上一条条目收尾之后）挂着 `#[cfg(test)]` 的，只给测试编，
/// 别的编进库。注释与字面量里写的不算，行内模块（`mod <名>` 后面接左花括号）不声明文件。
///
/// # Panics
/// 花括号第 0 层之外有 `mod <名>;`，或声明前面挂着 `#[path`：这两种声明的文件落在哪，这一读法认不出（不悄悄跳过）。
pub(super) fn submodule_files_declared_in(
    path: &str,
    role: RoleInTheFamily,
    source: &str,
) -> Vec<(String, RoleInTheFamily)> {
    let lines: Vec<&str> = source.split('\n').collect();
    let code = lines_read_past_comments_and_literals(&lines)
        .iter()
        .map(|read_line| read_line.without_comments_and_literals.as_str())
        .collect::<Vec<&str>>()
        .join("\n");
    let directory_of_the_submodules = path.strip_suffix(".rs").expect("族里每条路径都以 .rs 结尾");
    let is_identifier_character = |character: char| character.is_alphanumeric() || character == '_';
    let module_keyword = "mod";
    whole_word_positions(&code, module_keyword)
        .into_iter()
        .filter_map(|keyword_position| {
            let after_the_keyword = code[keyword_position + module_keyword.len()..].trim_start();
            let name_length = after_the_keyword
                .find(|character: char| !is_identifier_character(character))
                .unwrap_or(after_the_keyword.len());
            if !after_the_keyword[name_length..]
                .trim_start()
                .starts_with(';')
            {
                return None;
            }
            let name = &after_the_keyword[..name_length];
            let code_before = &code[..keyword_position];
            let brace_depth = code_before
                .matches('\u{7b}')
                .count()
                .checked_sub(code_before.matches('\u{7d}').count())
                .expect("右花括号不多于左花括号");
            assert_eq!(
                brace_depth, 0,
                "{path} 里花括号第 {brace_depth} 层有 mod {name};：行内模块里声明的文件落在哪，这一读法认不出"
            );
            let start_of_the_item = code_before
                .rfind([';', '\u{7b}', '\u{7d}'])
                .map_or(0, |end_of_the_item_before| end_of_the_item_before + 1);
            let attributes: String = code_before[start_of_the_item..]
                .chars()
                .filter(|character| !character.is_whitespace())
                .collect();
            assert!(
                !attributes.contains("#[path"),
                "{path} 里 mod {name}; 挂着 #[path]：声明的文件落在哪，这一读法认不出"
            );
            let declared_role = match role {
                RoleInTheFamily::CompiledOnlyForTests => RoleInTheFamily::CompiledOnlyForTests,
                RoleInTheFamily::CompiledIntoTheLibrary if attributes.contains("#[cfg(test)]") => {
                    RoleInTheFamily::CompiledOnlyForTests
                }
                RoleInTheFamily::CompiledIntoTheLibrary => RoleInTheFamily::CompiledIntoTheLibrary,
            };
            Some((
                format!("{directory_of_the_submodules}/{name}.rs"),
                declared_role,
            ))
        })
        .collect()
}

/// 崩溃放量模块族编进库的那几份（照文件表的次序）：路径与源码。扫源码的用例扫的就是这几份。
pub(super) fn library_files_of_the_family() -> Vec<(&'static str, &'static str)> {
    CRASH_AMPLIFICATION_FAMILY_FILES
        .iter()
        .filter(|(_path, role, _source)| *role == RoleInTheFamily::CompiledIntoTheLibrary)
        .map(|(path, _role, source)| (*path, *source))
        .collect()
}

/// `family` 每一份去掉测试部分之后的文本（[`text_outside_the_tests_of_a_family_file`]），照 `family` 的次序。
pub(super) fn family_text_outside_the_tests(family: &[(&str, &str)]) -> Vec<String> {
    family
        .iter()
        .map(|(path, source)| text_outside_the_tests_of_a_family_file(path, source))
        .collect()
}

/// 编进库的那几份，`path` 那一份换成 `replacement`、别的照原样：扫源码的用例在一份上改过之后再扫整族。
///
/// # Panics
/// 编进库的那几份里没有 `path`。
pub(super) fn library_files_with_one_replaced<'replacement>(
    path: &str,
    replacement: &'replacement str,
) -> Vec<(&'static str, &'replacement str)> {
    let family = library_files_of_the_family();
    assert!(
        family
            .iter()
            .any(|(library_path, _source)| *library_path == path),
        "编进库的那几份里有 {path}"
    );
    family
        .into_iter()
        .map(|(library_path, source)| {
            if library_path == path {
                (library_path, replacement)
            } else {
                (library_path, source)
            }
        })
        .collect()
}

/// 编进库的那几份里照文件表的次序头一份原文含 `anchor` 的：路径与源码；一份都没有交 `None`。
pub(super) fn first_library_file_holding(anchor: &str) -> Option<(&'static str, &'static str)> {
    library_files_of_the_family()
        .into_iter()
        .find(|(_path, source)| source.contains(anchor))
}

/// `anchor` 在编进库的那几份原文里一共出现几处。
pub(super) fn occurrences_in_the_library_files(anchor: &str) -> usize {
    library_files_of_the_family()
        .iter()
        .map(|(_path, source)| source.matches(anchor).count())
        .sum()
}

/// 族的根那一份的源码（测试模块写在它里面）。
pub(super) fn source_of_the_family_root() -> &'static str {
    CRASH_AMPLIFICATION_FAMILY_FILES
        .iter()
        .find(|(path, _role, _source)| *path == ROOT_OF_THE_FAMILY)
        .map(|(_path, _role, source)| *source)
        .expect("族的文件表里有根")
}
