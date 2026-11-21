//! 对拍参照一侧的入口登记表（`tests/pairing_units_entries.tsv` 的参照一侧那几行）逐组对测试支撑包（`singlefs-background-gpu-pairing-keys`）
//! 的构建脚本编进来的参照一侧键表（表的写法在 `singlefs-unit-graph` 的 `entry_groups` 文件头）：每组都算出了键、组里登记的入口都编进来了、
//! 每个入口在组的闭包里恰好是一个 fn，
//! 组名、入口、键各不相同。核法照 GPU 一侧那一张（`tests/pairing_conclusion_on_the_cards_is_reused_by_its_keys_and_judged_again_when_a_side_changes.rs`
//! 的 `every_gpu_side_group_is_keyed_and_closes_over_the_kernels_of_the_product_gpu_crate`）：只读编进来的那一份，跑时不读盘上的树。
//! 入口找不到、不唯一、在这组特性下不编进来的，构建脚本已经停下（`singlefs-unit-graph` 的 `EntryError`）；这里补的是构建脚本不停的那几种：
//! 组拒算或没编进来、一个入口没进闭包、两组共用一个组名或一个入口（一组登记成另一组那个编得过的入口，参照改了它的键不跟着变），
//! 与一组登记的入口不是它的调用点真交给对拍入口的参照函数（登记成另一个编得过、别组没用的函数，参照改了它的键同样不跟着变）：
//! 后一种对着编进来的调用点源文件（[`SOURCES_CALLING_THE_PAIRING_ENTRIES`]）逐组比。
//! 不要卡、不挂特性，`cargo test -p singlefs-background-gpu --test <本目标>` 就能跑。
mod common_pairing_side_units_entries;

use std::collections::{BTreeMap, BTreeSet};

use common_pairing_side_units_entries::{
    registered_groups_of_the_side, PairingSideOfTheEntries, PairingSideUnitsEntryGroup,
};
use singlefs_background_gpu_pairing_keys::REFERENCE_UNITS_KEYS_TEXT;

/// 入口 fn 路径起头的包名（构建脚本只拿它找包）。
const PACKAGE_PREFIX_OF_THE_ENTRIES: &str = "singlefs_background_gpu::";

/// 编进来的参照一侧键表里组 `group_name` 那几行：组那一行的结论（`keyed`、`refused`、`not-compiled`，没有这一组是空串）与键（算出了的才有），
/// 这组特性下没编进来的入口，组成清单里 fn 的限定名。
struct ReferenceGroupInTheCompiledTable {
    verdict: String,
    key_hexadecimal: Option<String>,
    entries_not_compiled: Vec<String>,
    qualified_names_of_the_functions: Vec<String>,
}

/// 编进来的参照一侧键表里组 `group_name` 那几行。
fn reference_group_in_the_compiled_table(group_name: &str) -> ReferenceGroupInTheCompiledTable {
    let mut verdict = String::new();
    let mut key_hexadecimal = None;
    let mut entries_not_compiled = Vec::new();
    let mut qualified_names_of_the_functions = Vec::new();
    for line in REFERENCE_UNITS_KEYS_TEXT.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        match fields.as_slice() {
            ["group", name, "keyed", key, ..] if *name == group_name => {
                verdict = "keyed".to_string();
                key_hexadecimal = Some((*key).to_string());
            }
            ["group", name, group_verdict, ..] if *name == group_name => {
                verdict = (*group_verdict).to_string();
            }
            ["not-compiled-entry", name, entry, ..] if *name == group_name => {
                entries_not_compiled.push((*entry).to_string());
            }
            ["unit", name, "fn", qualified_name, ..] if *name == group_name => {
                qualified_names_of_the_functions.push((*qualified_name).to_string());
            }
            _ => {}
        }
    }
    ReferenceGroupInTheCompiledTable {
        verdict,
        key_hexadecimal,
        entries_not_compiled,
        qualified_names_of_the_functions,
    }
}

/// 组成清单里的一个 fn 是不是登记的入口 `entry_path`：限定名在本包里（包名起头），去掉包名之后的模块路径与 fn 名是它的结尾
/// （限定名里包名与模块路径之间夹着目标的标记，例 `<test 目标>`、`<bin 名>`）。
fn is_the_function_of_the_entry(qualified_name: &str, entry_path: &str) -> bool {
    let path_in_the_package = entry_path
        .strip_prefix(PACKAGE_PREFIX_OF_THE_ENTRIES)
        .expect("登记表里的入口都写成 singlefs_background_gpu::<模块路径>::<fn>");
    qualified_name.starts_with(PACKAGE_PREFIX_OF_THE_ENTRIES)
        && qualified_name.ends_with(&format!("::{path_in_the_package}"))
}

/// 场景：读构建脚本编进来的参照一侧键表，逐组对登记表（`tests/pairing_units_entries.tsv` 的参照一侧那几行）里的组。
/// 预期：组名各不相同、两组不共用一个入口；每组都算出了键、组里登记的入口都编进来了、每个入口在这组的闭包里恰好是一个 fn；键两两不同。
/// 判别力（`crates/mutations.tsv`）：「参照一侧登记表：两组登记同一个组名」「参照一侧登记表：一组登记成另一组的入口」两行各让这一条红。
#[test]
fn every_reference_group_is_keyed_and_closes_over_each_of_its_own_entries_once() {
    let groups: Vec<PairingSideUnitsEntryGroup> =
        registered_groups_of_the_side(PairingSideOfTheEntries::Reference);
    let group_names: BTreeSet<&str> = groups.iter().map(|group| group.group_name).collect();
    assert_eq!(group_names.len(), groups.len(), "组名各不相同");
    let entries: Vec<&str> = groups
        .iter()
        .flat_map(|group| group.entry_functions.iter().copied())
        .collect();
    let distinct_entries: BTreeSet<&str> = entries.iter().copied().collect();
    assert_eq!(
        distinct_entries.len(),
        entries.len(),
        "两组不共用一个入口、一组里不重复登记一个入口：{entries:#?}"
    );
    let groups_falling_short: Vec<String> = groups
        .iter()
        .filter_map(|group| {
            let compiled = reference_group_in_the_compiled_table(group.group_name);
            let entries_not_found_once: Vec<(&str, usize)> = group
                .entry_functions
                .iter()
                .map(|entry| {
                    (
                        *entry,
                        compiled
                            .qualified_names_of_the_functions
                            .iter()
                            .filter(|qualified_name| is_the_function_of_the_entry(qualified_name, entry))
                            .count(),
                    )
                })
                .filter(|(_entry, functions_found)| *functions_found != 1)
                .collect();
            (compiled.verdict != "keyed"
                || !compiled.entries_not_compiled.is_empty()
                || !entries_not_found_once.is_empty())
            .then(|| {
                format!(
                    "{}：结论 {:?}，没编进来的入口 {:?}，在闭包里不是恰好一个 fn 的入口（入口，找到几个）{entries_not_found_once:?}",
                    group.group_name, compiled.verdict, compiled.entries_not_compiled
                )
            })
        })
        .collect();
    assert!(
        groups_falling_short.is_empty(),
        "每组都算出了键、入口都编进来了、每个入口在闭包里恰好是一个 fn：{groups_falling_short:#?}"
    );
    let distinct_keys: BTreeSet<String> = groups
        .iter()
        .filter_map(|group| reference_group_in_the_compiled_table(group.group_name).key_hexadecimal)
        .collect();
    assert_eq!(
        distinct_keys.len(),
        groups.len(),
        "每组一把键、两两不同：两组不落到同一把参照一侧的键上"
    );
}

/// 调参照一侧对拍入口的源文件（从包目录起的路径与原文，编进测试二进制）：每一处按组名交参照函数的调用都在这几份里。
/// 新写一处这样的调用，它所在的源文件要加进这张表；登记表里一组在这几份里一处调用点都没有，
/// [`every_reference_group_registers_exactly_the_functions_its_call_sites_pass_as_the_reference`] 判红。
const SOURCES_CALLING_THE_PAIRING_ENTRIES: [(&str, &str); 6] = [
    (
        "tests/common_checksum_batches_over_the_handed_over_gpus/mod.rs",
        include_str!("common_checksum_batches_over_the_handed_over_gpus/mod.rs"),
    ),
    (
        "tests/common_history_pool_through_the_library/mod.rs",
        include_str!("common_history_pool_through_the_library/mod.rs"),
    ),
    (
        "tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs",
        include_str!(
            "compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs"
        ),
    ),
    (
        "tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs",
        include_str!("live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs"),
    ),
    (
        "tests/mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu.rs",
        include_str!("mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu.rs"),
    ),
    (
        "tests/region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs",
        include_str!("region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs"),
    ),
];

/// 一个按组名收参照函数的对拍入口（`tests/common_pairing_input_through_the_library` 与 `tests/common_history_pool_through_the_library` 里的）：
/// 函数名，组名与参照函数各是它的哪个参数（从零起的下标）。
struct PairingEntryTakingAReferenceGroup {
    function_name: &'static str,
    group_argument_index: usize,
    reference_argument_index: usize,
}

/// 按组名收参照函数的对拍入口，参数次序照它们的签名。
const PAIRING_ENTRIES_TAKING_A_REFERENCE_GROUP: [PairingEntryTakingAReferenceGroup; 4] = [
    PairingEntryTakingAReferenceGroup {
        function_name: "pairing_concluded_on_the_cards_through_the_pairing_library",
        group_argument_index: 1,
        reference_argument_index: 3,
    },
    PairingEntryTakingAReferenceGroup {
        function_name: "pairing_input_and_reference_read_back_through_the_pairing_library",
        group_argument_index: 1,
        reference_argument_index: 3,
    },
    PairingEntryTakingAReferenceGroup {
        function_name: "history_pool_concluded_on_the_cards_through_the_pairing_library",
        group_argument_index: 1,
        reference_argument_index: 2,
    },
    PairingEntryTakingAReferenceGroup {
        function_name: "history_pool_and_reference_read_back_through_the_pairing_library",
        group_argument_index: 1,
        reference_argument_index: 2,
    },
];

/// 把组名与参照函数原样转交给下一层对拍入口的那几处（`tests/common_history_pool_through_the_library` 里的两个）写在组名参数上的名字：
/// 它们自己不点哪一组，组与参照函数由它们的调用方点，比的是调用方那一处。
const GROUP_PARAMETER_FORWARDED_BY_A_PAIRING_ENTRY: &str = "reference_units_group";

/// `source_text` 里每一处调 `function_name` 的顶层参数原文（去掉首尾空白；逗号只在调用自己那一层括号里分，括号、方括号、花括号、
/// 字符串与行注释里的不算）。`fn <名>(` 的定义与名字只是更长的标识符一截的不算调用。
///
/// # Errors
/// 一处调用的括号到源文件末尾都没合上。
fn arguments_of_each_call(
    source_text: &str,
    function_name: &str,
) -> Result<Vec<Vec<String>>, String> {
    let call_opening = format!("{function_name}(");
    let mut calls = Vec::new();
    for (call_start, _call_opening) in source_text.match_indices(&call_opening) {
        let text_before_the_call = &source_text[..call_start];
        let is_a_longer_identifier = text_before_the_call
            .chars()
            .next_back()
            .is_some_and(|character| character.is_alphanumeric() || character == '_');
        if is_a_longer_identifier || text_before_the_call.trim_end().ends_with("fn") {
            continue;
        }
        let mut arguments = Vec::new();
        let mut argument_text = String::new();
        let mut bracket_depth: usize = 1;
        let mut is_in_a_string = false;
        let mut characters = source_text[call_start + call_opening.len()..]
            .chars()
            .peekable();
        while bracket_depth > 0 {
            let Some(character) = characters.next() else {
                return Err(format!(
                    "{function_name} 的一处调用到源文件末尾括号都没合上"
                ));
            };
            if is_in_a_string {
                argument_text.push(character);
                if character == '\\' {
                    argument_text.extend(characters.next());
                } else if character == '"' {
                    is_in_a_string = false;
                }
                continue;
            }
            match character {
                '"' => {
                    is_in_a_string = true;
                    argument_text.push(character);
                }
                '/' if characters.peek() == Some(&'/') => {
                    for skipped in characters.by_ref() {
                        if skipped == '\n' {
                            break;
                        }
                    }
                    argument_text.push('\n');
                }
                '(' | '[' | '{' => {
                    bracket_depth += 1;
                    argument_text.push(character);
                }
                ')' | ']' | '}' => {
                    bracket_depth -= 1;
                    if bracket_depth > 0 {
                        argument_text.push(character);
                    }
                }
                ',' if bracket_depth == 1 => {
                    arguments.push(argument_text.trim().to_string());
                    argument_text.clear();
                }
                other_character => argument_text.push(other_character),
            }
        }
        let last_argument = argument_text.trim();
        if !last_argument.is_empty() {
            arguments.push(last_argument.to_string());
        }
        calls.push(arguments);
    }
    Ok(calls)
}

/// 调用点上组名那个参数写的组：字符串字面量照引号里的原样；全大写的常量名照同一份源文件里恰好一处 `const <名>: &str =` 之后的那个字面量。
/// 别的写法交回 `None`（认不出）。
fn group_name_written_in_the_argument(argument: &str, source_text: &str) -> Option<String> {
    if let Some(literal) = argument
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        return Some(literal.to_string());
    }
    let is_a_constant_name = !argument.is_empty()
        && argument.chars().all(|character| {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
        });
    if !is_a_constant_name {
        return None;
    }
    let constant_definition = format!("const {argument}: &str =");
    let mut definitions = source_text.match_indices(&constant_definition);
    let (definition_start, _definition) = definitions.next()?;
    if definitions.next().is_some() {
        return None;
    }
    let after_the_definition = &source_text[definition_start + constant_definition.len()..];
    let literal_and_rest = after_the_definition.split_once('"')?.1;
    Some(literal_and_rest.split_once('"')?.0.to_string())
}

/// 调用点上参照函数那个参数点的函数名（路径的末一段）：参数是一条路径（只有标识符字符与 `::`）才认，闭包与别的表达式交回 `None`。
fn function_named_in_the_argument(argument: &str) -> Option<String> {
    let is_a_path = !argument.is_empty()
        && argument
            .chars()
            .all(|character| character.is_alphanumeric() || character == '_' || character == ':');
    is_a_path.then(|| argument.rsplit("::").next().unwrap_or(argument).to_string())
}

/// 场景：读编进来的调用点源文件，找出每一处按组名交参照函数的对拍调用，记下每组的调用点交的参照函数；转交组名参数的那几处不算。
/// 预期：每一处调用的组名与参照函数都认得出（参照函数写成具名函数，不写闭包）；参照一侧登记表里每组登记的入口（按函数名）恰好是它的调用点交的那几个参照函数，
/// 调用点点的组都登记了：一组登记成另一个编得过、别组没用的函数时，参照改了它的键不跟着变，这一条判红。
/// 判别力（`crates/mutations.tsv`）：「参照一侧登记表：装置自数那一组登记成同一个装置里另一个存在的函数」让这一条红。
#[test]
fn every_reference_group_registers_exactly_the_functions_its_call_sites_pass_as_the_reference() {
    let mut functions_passed_per_group: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut call_sites_not_read: Vec<String> = Vec::new();
    for (source_path, source_text) in SOURCES_CALLING_THE_PAIRING_ENTRIES {
        for entry in &PAIRING_ENTRIES_TAKING_A_REFERENCE_GROUP {
            let calls = arguments_of_each_call(source_text, entry.function_name)
                .unwrap_or_else(|reason| panic!("{source_path}：{reason}"));
            for arguments in calls {
                let group_argument = arguments
                    .get(entry.group_argument_index)
                    .map(String::as_str);
                let reference_argument = arguments
                    .get(entry.reference_argument_index)
                    .map(String::as_str);
                if group_argument == Some(GROUP_PARAMETER_FORWARDED_BY_A_PAIRING_ENTRY) {
                    continue;
                }
                let group_name_read = group_argument
                    .and_then(|argument| group_name_written_in_the_argument(argument, source_text));
                let function_name_read =
                    reference_argument.and_then(function_named_in_the_argument);
                if let (Some(group_name), Some(function_name)) =
                    (&group_name_read, &function_name_read)
                {
                    functions_passed_per_group
                        .entry(group_name.clone())
                        .or_default()
                        .insert(function_name.clone());
                } else {
                    call_sites_not_read.push(format!(
                        "{source_path} 调 {}：组名 {group_argument:?} 认出 {group_name_read:?}，参照 {reference_argument:?} 认出 {function_name_read:?}",
                        entry.function_name
                    ));
                }
            }
        }
    }
    assert!(
        call_sites_not_read.is_empty(),
        "按组名交参照函数的调用，组名要写成字面量或本文件的常量、参照函数要写成具名函数：{call_sites_not_read:#?}"
    );
    let functions_registered_per_group: BTreeMap<String, BTreeSet<String>> =
        registered_groups_of_the_side(PairingSideOfTheEntries::Reference)
            .iter()
            .map(|group| {
                (
                    group.group_name.to_string(),
                    group
                        .entry_functions
                        .iter()
                        .map(|entry| entry.rsplit("::").next().unwrap_or(entry).to_string())
                        .collect(),
                )
            })
            .collect();
    assert_eq!(
        functions_registered_per_group, functions_passed_per_group,
        "参照一侧登记表每组登记的入口（左）要恰好是它的调用点交给对拍入口的参照函数（右）"
    );
}
