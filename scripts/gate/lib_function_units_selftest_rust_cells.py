"""自证的 Rust 与 WGSL 格：文档测试、与 unit-graph 对拍、闭包规则与边、cfg、manifest、词法缓存、unit-graph 输入与夹具名字的格。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import ast
import os
import re
import types
from lib_function_units_lexer import (
    RUST_LEX_CACHE_DIRECTORY_VARIABLE, RUST_LEX_CACHE_STATS, active_breaks, rust_lex, rust_lex_cache_path, rust_lex_cached,
)
from lib_function_units_base import DEFAULT_SCRIPT_DIRECTORIES, LIBRARY_ROOT, SKIPPED_DIRECTORIES, files_under, sha256_bytes
from lib_function_units_registry import EXCLUDED_CALL_EDGES
from lib_function_units_shell_index import ShellIndex
from lib_function_units_rust_file import standard_macros_evaluating_their_arguments_in_place
from lib_function_units_unit_graph import unit_graph_inputs_digest
from lib_function_units_rust_workspace import RustWorkspace
from lib_function_units_rust_closure import rust_closure_of
from lib_function_units_script_closure import Options, closure_of, parse_entry
from lib_function_units_closure_keys import closure_key, dump_lines, edit_file, unit_graph_key, write_tree
from lib_function_units_selftest_fixture_trees import (
    alignment_dump, alignment_kind_names_cell, compare_cell, fixture_key, line_with, profile_unkeyed_cell, real_tree, repository_copy_module,
    rust_string_list_cell, with_breaks_added,
)


def alignment_qualified_names_cell(work):
    """dump 的限定名带 crate、目标与模块路径前缀（lib 不写目标，bin 写 <bin 名>，mod 声明连进来的文件带模块路径）；文档测试写 <模块>::<doctest line 行>。"""
    _base, lines = alignment_dump(work)
    wanted = ("fn\tfalign::helpers::help\tcrates/falign/src/helpers.rs:1-3\t", "fn\tfalign::<bin falign>::main\tcrates/falign/src/main.rs:1-3\t",
              "fn\tfalign::<bin tool>::main\tcrates/falign/src/bin/tool.rs:1-1\t", "fn\tfalign::nested::inside\tcrates/falign/src/lib.rs:10-12\t",
              "doctest\tfalign::<doctest line 4>\tcrates/falign/src/lib.rs:4-7\t", "wgsl_compute\tfalign::kernel_count.wgsl::main\t")
    missing = [prefix for prefix in wanted if not line_with(lines, prefix)]
    return not missing, f"dump 里缺 {missing}"


def doctest_fingerprint(base):
    units = [unit for unit in RustWorkspace(base).file(ALIGNMENT_ENTRY_FILE).units if unit.kind == "doctest"]
    return units[0].fingerprint if len(units) == 1 else f"文档测试单元 {len(units)} 个"


def alignment_doctest_fingerprint_cell(work):
    """文档测试的指纹按代码块重新切词、去注释：代码块里只改注释不变，改代码变（不随原文各行变）。"""
    base = os.path.join(work, "tree")
    write_tree(base, ALIGNMENT_FIXTURE)
    before = doctest_fingerprint(base)
    edit_file(base, ALIGNMENT_ENTRY_FILE, "//! // 只是注释\n", "//! // 只是注释，改了措辞\n")
    commented = doctest_fingerprint(base)
    edit_file(base, ALIGNMENT_ENTRY_FILE, "falign::width(), 4);", "falign::width(), 5);")
    changed = doctest_fingerprint(base)
    return before == commented and commented != changed and len(before) == 64, f"改注释前 {before}、改注释后 {commented}、改代码后 {changed}"


def alignment_module_head_cell(work):
    """模块头：文件开头的 #![…] 与行内模块头（连内属性）各切成一个 mod_head 单元；模块里的单元连它，改内属性作废。"""
    base, lines = alignment_dump(work)
    wanted = ("mod_head\tfalign::<module attributes>\tcrates/falign/src/lib.rs:1-1\t", "mod_head\tfalign::nested\tcrates/falign/src/lib.rs:8-9\t")
    missing = [prefix for prefix in wanted if not line_with(lines, prefix)]
    before, _ = fixture_key(base, f"{ALIGNMENT_ENTRY_FILE}::nested::inside")
    edit_file(base, ALIGNMENT_ENTRY_FILE, "#![allow(unused)]", "#![allow(unused_variables)]")
    after, _ = fixture_key(base, f"{ALIGNMENT_ENTRY_FILE}::nested::inside")
    return not missing and before is not None and before != after, f"dump 里缺 {missing}；nested::inside 的键改内属性前 {before}、后 {after}"


def alignment_data_dump_cell(work):
    """dump 按读它的那一处列数据单元：data<TAB>数据文件<TAB>读它的文件:行-行<TAB>字节 sha256（.wgsl 不列，按 WGSL 单元列）。"""
    _base, lines = alignment_dump(work)
    want = f"data\tcrates/falign/src/table.txt\tcrates/falign/src/lib.rs:33-33\t{sha256_bytes(b'row' + bytes([10]))}"
    wgsl_listed = [line for line in lines if line.startswith("data\t") and ".wgsl" in line.split("\t")[1]]
    return want in lines and not wgsl_listed, f"要有 {want}，读 .wgsl 的不列：{wgsl_listed}"


def alignment_external_prefix_cell(work):
    """WGSL 外名的首段连 _ 落在闭包里 macro_rules! 体的词法单元里（FAILURE_POINT_NONE 取 FAILURE_）或串字面量里（T_HEADER 取 T_）就有着落、不拒算。"""
    base = os.path.join(work, "tree")
    write_tree(base, ALIGNMENT_FIXTURE)
    key, forms = fixture_key(base, f"{ALIGNMENT_ENTRY_FILE}::kernel_source")
    return key is not None, f"不该拒算，实际拒算 {sorted(forms)}"


def alignment_external_identifier_cell(work):
    """WGSL 外名只出现在闭包里别的 Rust 单元的长标识符里（let COUNT_LIMIT）、不在串字面量与宏体里：拒算（不再按「标识符长于 3 个字符」认）。"""
    base = os.path.join(work, "tree")
    write_tree(base, ALIGNMENT_FIXTURE)
    closure = closure_of(base, f"{ALIGNMENT_ENTRY_FILE}::count_kernel_source")
    refused = [refusal for refusal in closure.refusals if refusal.form == "wgsl-undefined-name"]
    return [refusal.snippet for refusal in refused] == ["COUNT_LIMIT"], f"要恰好拒算 COUNT_LIMIT，实际 {[refusal.text() for refusal in closure.refusals]}"


def alignment_external_edge_cell(work):
    """外名的着落做成边：WGSL 入口（--rust-strings-entry 给拼内核的 Rust 函数）的闭包带上给名字着落的 Rust 单元与它的闭包，改那个单元键变。"""
    base = os.path.join(work, "tree")
    write_tree(base, ALIGNMENT_FIXTURE)
    options = Options()
    options.rust_strings_entry = f"{ALIGNMENT_ENTRY_FILE}::kernel_source"
    entry = "crates/falign/src/kernel.wgsl::main"
    before, before_refusals = closure_key(closure_of(base, entry, options))
    edit_file(base, ALIGNMENT_ENTRY_FILE, "const T_{}: u32 = 1u;", "const T_{}: u32 = 2u;")
    after, _ = closure_key(closure_of(base, entry, options))
    return before is not None and before != after, f"改给 T_ 着落的那一串，WGSL 入口的键改前 {before}（拒算 {[r.text() for r in before_refusals]}）、改后 {after}"


def alignment_union_cell(work):
    """union 是上下文关键字：declare_named!(union) 这条条目级宏调用可能定义 union，调 union() 的函数连到它。"""
    base = os.path.join(work, "tree")
    write_tree(base, ALIGNMENT_FIXTURE)
    closure = closure_of(base, f"{ALIGNMENT_ENTRY_FILE}::call_union")
    macros = [unit.name for unit in closure.order if unit.kind == "item_macro"]
    return macros == ["declare_named!@26"], f"call_union 的闭包里条目级宏调用：{macros}"


def alignment_cells():
    return [
        ("dump：种类名照 crates/singlefs-unit-graph 的拼法（macro_rules、item_macro、mod、mod_head、wgsl_compute）", alignment_kind_names_cell),
        ("dump：限定名带 crate、目标与模块路径前缀，文档测试写 <模块>::<doctest line 行>", alignment_qualified_names_cell),
        ("文档测试：代码块里只改注释指纹不变、改代码指纹变", alignment_doctest_fingerprint_cell),
        ("模块头：#![…] 与行内模块头切成单元，模块里的单元连它，改内属性作废", alignment_module_head_cell),
        ("dump：include_str! 读的数据文件按读它的那一处列一行 data", alignment_data_dump_cell),
        ("WGSL 外名：首段连 _ 落在 macro_rules! 体或串字面量里有着落", alignment_external_prefix_cell),
        ("WGSL 外名：只在别的单元的长标识符里出现的拒算", alignment_external_identifier_cell),
        ("WGSL 外名的着落做成边：给名字的 Rust 单元改了 WGSL 入口作废", alignment_external_edge_cell),
        ("format! 串里 {{{名}}}（转义的 {{ 后紧跟的捕获）改了作废", compare_cell(ALIGNMENT_FIXTURE, f"{ALIGNMENT_ENTRY_FILE}::wrapped", ALIGNMENT_ENTRY_FILE,
                                                         "pub const WIDTH: u32 = 4;", "pub const WIDTH: u32 = 5;", "changed")),
        ("条目级宏调用里的 union 算它可能定义的名字", alignment_union_cell),
    ]



def closure_member_names(base, entry):
    return {f"{unit.kind} {unit.name}" for unit in closure_of(base, entry).order if not unit.data}


def closure_rules_file_head_cell(work):
    """改文件开头的 #![…]：经 mod sub; 连进来的 sub.rs 里的入口作废（入口连声明它所在文件的 mod，mod 再连 lib.rs 的模块头）。"""
    base = os.path.join(work, "tree")
    write_tree(base, CLOSURE_RULES_FIXTURE)
    entry = "crates/frules/src/sub.rs::in_sub"
    members = closure_member_names(base, entry)
    wanted = {"mod sub", "mod_head <module attributes>"}
    before, _ = fixture_key(base, entry)
    edit_file(base, CLOSURE_RULES_FILE, "#![allow(dead_code)]", "#![allow(dead_code, unused_imports)]")
    after, _ = fixture_key(base, entry)
    return wanted <= members and before is not None and before != after, f"in_sub 的闭包缺 {sorted(wanted - members)}；改 #![…] 前键 {before}、后 {after}"


def closure_rules_inline_head_cell(work):
    """改行内模块头的内属性：只那个模块里的入口作废，模块外同一份文件里的入口不变。"""
    base = os.path.join(work, "tree")
    write_tree(base, CLOSURE_RULES_FIXTURE)
    inside, outside = f"{CLOSURE_RULES_FILE}::inline_area::inside", f"{CLOSURE_RULES_FILE}::outside"
    inside_before, outside_before = fixture_key(base, inside)[0], fixture_key(base, outside)[0]
    edit_file(base, CLOSURE_RULES_FILE, "#![allow(unused)]", "#![allow(unused_variables)]")
    inside_after, outside_after = fixture_key(base, inside)[0], fixture_key(base, outside)[0]
    ok = None not in (inside_before, outside_before) and inside_before != inside_after and outside_before == outside_after
    return ok, f"inline_area::inside 改前 {inside_before}、后 {inside_after}；outside 改前 {outside_before}、后 {outside_after}"


def closure_rules_glob_cell(work):
    """use tools::* 之后只用到 tools 里一个 fn：闭包有那条 use 与那个 fn，没有 tools 里别的 fn；改没用到的 fn 不作废。"""
    base = os.path.join(work, "tree")
    write_tree(base, CLOSURE_RULES_FIXTURE)
    entry = f"{CLOSURE_RULES_FILE}::glob_caller"
    members = closure_member_names(base, entry)
    uses = sorted(name for name in members if name.startswith("use "))
    extra = sorted({"fn tools::unused", "fn tools::original"} & members)
    before, _ = fixture_key(base, entry)
    edit_file(base, CLOSURE_RULES_FILE, "        4\n", "        40\n")
    after, _ = fixture_key(base, entry)
    ok = len(uses) == 1 and "fn tools::used" in members and not extra and before is not None and before == after
    return ok, f"glob_caller 的闭包里 use {uses}、有没有 tools::used {'fn tools::used' in members}、多出 {extra}；改 unused 前键 {before}、后 {after}"


def closure_rules_listed_cell(work):
    """use crate::tools::{used, unused} 只用 used：use 只结构上连、引进的名字不往下解，闭包里没有 unused，改它不作废。"""
    base = os.path.join(work, "tree")
    write_tree(base, CLOSURE_RULES_FIXTURE)
    entry = f"{CLOSURE_RULES_FILE}::listing::list_caller"
    members = closure_member_names(base, entry)
    before, _ = fixture_key(base, entry)
    edit_file(base, CLOSURE_RULES_FILE, "        4\n", "        40\n")
    after, _ = fixture_key(base, entry)
    uses = sorted(name for name in members if name.startswith("use "))
    ok = uses and "fn tools::unused" not in members and before is not None and before == after
    return ok, f"list_caller 的闭包里 use {uses}、有没有 tools::unused {'fn tools::unused' in members}；改 unused 前键 {before}、后 {after}"


def closure_rules_alias_cell(work):
    """use crate::tools::original as renamed 之后调 renamed()：按原名 original 再解，闭包含 tools::original，改它作废。"""
    base = os.path.join(work, "tree")
    write_tree(base, CLOSURE_RULES_FIXTURE)
    entry = f"{CLOSURE_RULES_FILE}::renaming::alias_caller"
    members = closure_member_names(base, entry)
    before, _ = fixture_key(base, entry)
    edit_file(base, CLOSURE_RULES_FILE, "        5\n", "        50\n")
    after, _ = fixture_key(base, entry)
    return "fn tools::original" in members and before is not None and before != after, \
        f"alias_caller 的闭包里有没有 tools::original {'fn tools::original' in members}；改 original 前键 {before}、后 {after}"


def closure_rules_trait_argument_cell(work):
    """R-隐式按自身类型认 trait impl：只点名 impl From<Inner> for Outer 里 trait 参数的 Inner，这个 impl 不进闭包；点名 Outer 的进。"""
    base = os.path.join(work, "tree")
    write_tree(base, CLOSURE_RULES_FIXTURE)
    impl_name = "impl <impl From < Inner > for Outer>"
    inner, outer = closure_member_names(base, f"{CLOSURE_RULES_FILE}::make_inner"), closure_member_names(base, f"{CLOSURE_RULES_FILE}::make_outer")
    return impl_name not in inner and impl_name in outer, f"make_inner 的闭包里有没有那个 impl {impl_name in inner}，make_outer 的 {impl_name in outer}"


def closure_rule_cells():
    return [
        ("R-结构：改文件开头的 #![…]，经 mod 声明连进来的子模块里的入口作废", closure_rules_file_head_cell),
        ("R-结构：改行内模块头的内属性，只那个模块里的入口作废", closure_rules_inline_head_cell),
        ("R-结构：use a::* 只用到 a 里一个 fn，闭包有那条 use 与那个 fn、没有 a 里别的 fn", closure_rules_glob_cell),
        ("R-结构：use 引进的名字不往下解，没用到的那个改了不作废", closure_rules_listed_cell),
        ("R-结构：use a::b as c 之后调 c()，按 b 再解", closure_rules_alias_cell),
        ("R-隐式：只点名 trait 参数里的类型，那个 trait impl 不进闭包", closure_rules_trait_argument_cell),
    ]



def outside_crate_edit_verdicts(work, edits):
    """OUTSIDE_CRATE_NAME_FIXTURE 上入口 served 的键：逐个改 edits 里的 (说明, 旧, 新)，每改一处跟前一把比 → ([(说明, changed / same / 拒算)], 改动前的键)。"""
    base = os.path.join(work, "tree")
    write_tree(base, OUTSIDE_CRATE_NAME_FIXTURE)
    previous, _ = fixture_key(base, OUTSIDE_CRATE_NAME_ENTRY)
    first, verdicts = previous, []
    for label, old, new in edits:
        edit_file(base, OUTSIDE_CRATE_NAME_FILE, old, new)
        current, refused = fixture_key(base, OUTSIDE_CRATE_NAME_ENTRY)
        if previous is None or current is None:
            verdicts.append((label, f"拒算 {sorted(refused)}"))
        else:
            verdicts.append((label, "changed" if current != previous else "same"))
        previous = current
    return verdicts, first


def outside_import_cell(work):
    """别的包按裸名连进工作区外的 crate 要经 use 或路径：halted 只是 fuser 的字段名，改 fservice 的 fn halted 键不变；
    use 引进的 restarts、经 use 引进的模块 control 写成 control::paused、写成 fservice::stopped_connections 的，改了键都变。"""
    verdicts, first = outside_crate_edit_verdicts(work, (("fn halted", "    6\n", "    60\n"), ("fn restarts", "    4\n", "    40\n"),
                                                         ("control::paused", "        5\n", "        50\n"),
                                                         ("fn stopped_connections", "Worker.launch() + Log", "Worker.launch() + 1 + Log")))
    wanted = [("fn halted", "same"), ("fn restarts", "changed"), ("control::paused", "changed"), ("fn stopped_connections", "changed")]
    return first is not None and verdicts == wanted, f"期望 {wanted}，实际 {verdicts}（改前键 {first}）"


def outside_module_cell(work):
    """工作区外 crate 的同一个目标里按模块连：stopped_connections 经 use log::Log 用 Log，bench 模块里同名的 Log 不连，改它键不变；
    Worker::launch 自己的名字不连到 bench 模块里的 fn launch，改它键不变；改 log::Log、改 Worker::launch 的体，键都变。"""
    verdicts, first = outside_crate_edit_verdicts(work, (("bench::Log", "pub entries: u64,", "pub entries: u16,"), ("bench::launch", "        7\n", "        70\n"),
                                                         ("log::Log", "pub entries: u32,", "pub entries: u8,"), ("Worker::launch", "        2\n", "        20\n")))
    wanted = [("bench::Log", "same"), ("bench::launch", "same"), ("log::Log", "changed"), ("Worker::launch", "changed")]
    return first is not None and verdicts == wanted, f"期望 {wanted}，实际 {verdicts}（改前键 {first}）"



def closure_edge_cells():
    return [
        (MACRO_TYPE_EDGE_CELL, compare_cell(MACRO_EDGE_FIXTURE, f"{MACRO_EDGE_FILE}::heavy_amount", MACRO_EDGE_FILE,
                                            "Weight::Heavy => 9,", "Weight::Heavy => 8,", "changed")),
        (MACRO_LITERAL_EDGE_CELL, compare_cell(MACRO_EDGE_FIXTURE, f"{MACRO_EDGE_FILE}::code_count", MACRO_EDGE_FILE,
                                               "CODE_FULL = 7,", "CODE_FULL = 8,", "changed")),
        (ALLOCATOR_EDGE_CELL, compare_cell(ALLOCATOR_EDGE_FIXTURE, "crates/falloc/src/main.rs::growths", "crates/falloc/src/main.rs",
                                           "GROWTHS.fetch_add(1, Ordering::Relaxed);", "GROWTHS.fetch_add(2, Ordering::Relaxed);", "changed")),
        (FOREIGN_IMPL_EDGE_CELL, compare_cell(FOREIGN_IMPL_EDGE_FIXTURE, "crates/fforeign/src/lib.rs::measured_second", "crates/fforeign/src/lib.rs",
                                              "self.as_secs() + 1", "self.as_secs() + 2", "changed")),
        (BUILD_ENVIRONMENT_EDGE_CELL, compare_cell(BUILD_ENVIRONMENT_EDGE_FIXTURE, "crates/fbuild/src/lib.rs::reported", "crates/fbuild/build.rs",
                                                   'String::from("plain")', 'String::from("gpu")', "changed")),
        (EXCLUDED_PATH_DEPENDENCY_EDGE_CELL, compare_cell(EXCLUDED_PATH_DEPENDENCY_EDGE_FIXTURE, "crates/fuser/src/lib.rs::stop_count", "fservice/src/lib.rs",
                                                          "    3\n", "    4\n", "changed")),
        (OUTSIDE_IMPORT_CELL, outside_import_cell),
        (OUTSIDE_MODULE_CELL, outside_module_cell),
        (LET_SHADOW_BOUND_CELL, compare_cell(LET_SHADOW_FIXTURE, "crates/fshadow/src/lib.rs::bound", "crates/fshadow/src/lib.rs", "    11\n", "    12\n", "same")),
        (LET_SHADOW_PARAMETER_CELL, compare_cell(LET_SHADOW_FIXTURE, "crates/fshadow/src/lib.rs::parameter", "crates/fshadow/src/lib.rs", "    11\n", "    12\n", "same")),
        (LET_SHADOW_CAPTURE_CELL, compare_cell(LET_SHADOW_FIXTURE, "crates/fshadow/src/lib.rs::captured", "crates/fshadow/src/lib.rs", "    11\n", "    12\n", "same")),
        (LET_SHADOW_LISTED_MACRO_CELL, compare_cell(LET_SHADOW_FIXTURE, "crates/fshadow/src/lib.rs::named_in_a_listed_macro", "crates/fshadow/src/lib.rs", "    11\n", "    12\n", "same")),
        (LET_SHADOW_UNLISTED_MACRO_CELL, compare_cell(LET_SHADOW_FIXTURE, "crates/fshadow/src/lib.rs::named_in_a_macro_outside_the_list", "crates/fshadow/src/lib.rs",
                                                      "    11\n", "    12\n", "changed")),
        (LET_SHADOW_NESTED_MACRO_CELL, compare_cell(LET_SHADOW_FIXTURE, "crates/fshadow/src/lib.rs::named_in_a_listed_macro_inside_an_unlisted_macro", "crates/fshadow/src/lib.rs",
                                                    "    11\n", "    12\n", "changed")),
        (STANDARD_MACRO_LIST_CELL, rust_string_list_cell("STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE", standard_macros_evaluating_their_arguments_in_place)),
        (LET_SHADOW_BEFORE_CELL, compare_cell(LET_SHADOW_FIXTURE, "crates/fshadow/src/lib.rs::before", "crates/fshadow/src/lib.rs", "    11\n", "    12\n", "changed")),
        (LET_SHADOW_OTHER_BLOCK_CELL, compare_cell(LET_SHADOW_FIXTURE, "crates/fshadow/src/lib.rs::other_block", "crates/fshadow/src/lib.rs", "    11\n", "    12\n", "changed")),
        (LET_SHADOW_CONSTANT_CELL, compare_cell(LET_SHADOW_FIXTURE, "crates/fshadow/src/lib.rs::constant_pattern", "crates/fshadow/src/lib.rs", "= 21;", "= 22;", "changed")),
        (PROFILE_UNKEYED_CELL, profile_unkeyed_cell),
        (USE_PATH_CONDITIONAL_CELL, compare_cell(USE_PATH_CONDITIONAL_FIXTURE, "crates/fswap/src/lib.rs::entry_point", "crates/fswap/src/lib.rs",
                                                 ".map(|byte| u32::from(*byte)).sum()", ".map(|byte| u32::from(*byte) + 1).sum()", "changed")),
        (USE_PATH_RE_EXPORT_CELL, compare_cell(USE_PATH_RE_EXPORT_FIXTURE, "crates/freexport/src/lib.rs::entry_point", "crates/freexport/src/lib.rs",
                                               ".map(|byte| u32::from(*byte) * 2).sum()", ".map(|byte| u32::from(*byte) * 3).sum()", "changed")),
        (USE_PATH_FIRST_SEGMENT_CELL, compare_cell(USE_PATH_FIRST_SEGMENT_FIXTURE, "crates/falias/src/lib.rs::entry_point", "crates/falias/src/lib.rs",
                                                   "            7\n", "            70\n", "changed")),
        (USE_PATH_SAME_USE_CELL, compare_cell(USE_PATH_SAME_USE_FIXTURE, "crates/fsameuse/src/lib.rs::through_one_use::entry_point", "crates/fsameuse/src/lib.rs",
                                              "            7\n", "            70\n", "changed")),
        (USE_PATH_SELF_IMPORT_CELL, compare_cell(USE_PATH_SAME_USE_FIXTURE, "crates/fsameuse/src/lib.rs::through_self_import::entry_point", "crates/fsameuse/src/lib.rs",
                                                 "            7\n", "            70\n", "changed")),
        (USE_PATH_ITEM_MACRO_CELL, compare_cell(USE_PATH_SAME_USE_FIXTURE, "crates/fsameuse/src/lib.rs::through_a_macro::entry_point", "crates/fsameuse/src/lib.rs",
                                                "            7\n", "            70\n", "changed")),
        (USE_PATH_ALIAS_LEAF_CELL, compare_cell(USE_PATH_ALIAS_LEAF_FIXTURE, "crates/fdiverge/src/lib.rs::entry_point", "crates/fdiverge/src/lib.rs",
                                                "pub mod second {\n    pub fn helper_total() -> u32 {\n        3", "pub mod second {\n    pub fn helper_total() -> u32 {\n        30", "changed")),
        (USE_PATH_EXACT_CELL, compare_cell(USE_PATH_EXACT_FIXTURE, "crates/fprelude/src/user.rs::width", "crates/fprelude/src/deep.rs", "= 64;", "= 65;", "changed")),
        (CRATES_MODULE_PRIVATE_CELL, compare_cell(CRATES_MODULE_FIXTURE, "crates/fmodules/src/cpu_checks.rs::cpu_slots", "crates/fmodules/src/gpu_checks.rs",
                                                  "usize = 32;", "usize = 33;", "same")),
        (CRATES_MODULE_IMPORTED_CELL, compare_cell(CRATES_MODULE_FIXTURE, "crates/fmodules/src/cpu_checks.rs::cpu_slots", "crates/fmodules/src/format.rs",
                                                   "usize = 64;", "usize = 65;", "changed")),
    ]



def cfg_keys(base, entry, features):
    """入口在给的特性下 → (unit_graph_key, closure_key)；拒算的那一把是 None。"""
    workspace = RustWorkspace(base)
    language, path, qualified = parse_entry(base, entry)
    unit, target_id = workspace.find_entry(path, qualified, None)
    closure = rust_closure_of(workspace, unit, target_id, None, False, list(features), False)
    return unit_graph_key(closure, ["host: x86_64-unknown-linux-gnu"])[0], closure_key(closure)[0]


def cfg_unreferenced_cell(work):
    """闭包里没有单元引到 gpu 的入口：带不带 gpu 特性两种目标下两把键（unit_graph_key 与 closure_key）都相同。"""
    base = os.path.join(work, "tree")
    write_tree(base, CFG_ATOMS_FIXTURE)
    without = cfg_keys(base, f"{CFG_ATOMS_FILE}::plain_entry", [])
    with_gpu = cfg_keys(base, f"{CFG_ATOMS_FILE}::plain_entry", ["fcfg/gpu"])
    return None not in without + with_gpu and without == with_gpu, f"不带 gpu {without}、带 gpu {with_gpu}"


def cfg_referenced_cell(work):
    """闭包里有单元引到 gpu（函数体里的 cfg!、语句上的 #[cfg]、入口自己的 #[cfg_attr]）：带不带 gpu 特性两把键都不同。"""
    base = os.path.join(work, "tree")
    write_tree(base, CFG_ATOMS_FIXTURE)
    same = []
    for name in ("macro_entry", "statement_entry", "attribute_only_entry"):
        without = cfg_keys(base, f"{CFG_ATOMS_FILE}::{name}", [])
        with_gpu = cfg_keys(base, f"{CFG_ATOMS_FILE}::{name}", ["fcfg/gpu"])
        if None in without + with_gpu or without[0] == with_gpu[0] or without[1] == with_gpu[1]:
            same.append(f"{name}：不带 {without}、带 {with_gpu}")
    return not same, "；".join(same)


# 外名由 Rust 现拼的三种写法（与 crates/singlefs-unit-graph 的 tests/unit_graph_wgsl_modules_close_over_compute_entries.rs 的三份夹具同形）
WGSL_SUPPLY_FIXTURE = {
    "crates/fsupply/Cargo.toml": '[package]\nname = "fsupply"\n',
    "crates/fsupply/src/lib.rs": '''pub const CODE_TABLE_KERNEL_SOURCE: &str = include_str!("failure_points.wgsl");
pub const UNRELATED_KERNEL_SOURCE: &str = include_str!("unrelated.wgsl");
pub const SUPPLIED_KERNEL_SOURCE: &str = include_str!("supplied.wgsl");
pub const PRELUDE_SOURCE: &str = include_str!("prelude.wgsl");
pub const SPLIT_KERNEL_SOURCE: &str = include_str!("split.wgsl");
macro_rules! failure_points {
    ($($constant:ident = $code:literal,)*) => {
        $(pub const $constant: u32 = $code;)*
        pub const GPU_FAILURE_POINTS: &[(&str, u32)] = &[$((stringify!($constant), $code),)*];
    };
}
failure_points! {
    GPU_FAILURE_POINT_NONE = 0,
    GPU_FAILURE_POINT_UNREADABLE = 9,
}
pub fn failure_point_constants() -> String {
    GPU_FAILURE_POINTS
        .iter()
        .map(|(name, code)| format!("const {}: u32 = {code}u;\\n", name.strip_prefix("GPU_").expect("都以 GPU_ 起头")))
        .collect()
}
pub fn failure_point_kernel_text() -> String {
    assert_eq!(GPU_FAILURE_POINT_NONE, 0, "无失败点的码是 0");
    format!("{}{CODE_TABLE_KERNEL_SOURCE}", failure_point_constants())
}
pub fn supplied_kernel_text() -> String {
    format!("{}{SUPPLIED_KERNEL_SOURCE}", failure_point_constants())
}
pub fn unrelated_kernel_text() -> String {
    let note = "FOO_ 起头的说明串，与内核里的名字无关";
    format!("// {note}\\n{UNRELATED_KERNEL_SOURCE}")
}
pub fn prelude_text() -> &'static str {
    PRELUDE_SOURCE
}
pub fn split_kernel_text() -> String {
    format!("{}{SPLIT_KERNEL_SOURCE}", prelude_text())
}
''',
    "crates/fsupply/src/failure_points.wgsl": '''@group(0) @binding(0) var<storage, read_write> words: array<u32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    words[id.x] = FAILURE_POINT_NONE + FAILURE_POINT_UNREADABLE;
}
''',
    "crates/fsupply/src/unrelated.wgsl": '''@group(0) @binding(0) var<storage, read_write> words: array<u32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    words[id.x] = FOO_BAR;
}
''',
    "crates/fsupply/src/supplied.wgsl": '''@group(0) @binding(0) var<storage, read_write> words: array<u32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    words[id.x] = FAILURE_POINT_UNREADABLE;
}
''',
    "crates/fsupply/src/prelude.wgsl": '''fn doubled(value: u32) -> u32 {
    return value * 2u;
}
''',
    "crates/fsupply/src/split.wgsl": '''@group(0) @binding(0) var<storage, read_write> words: array<u32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    words[id.x] = doubled(id.x);
}
''',
}
WGSL_SUPPLY_FILE = "crates/fsupply/src/lib.rs"
# 同一份样本再加一个测试模块：入口是里面一条测试（按测试编，测试单元都编），另一条测试的串里写着外名、函数体读运行期的清单目录（拒算 runtime-path）
WGSL_TEST_SUPPLIER_FIXTURE = dict(WGSL_SUPPLY_FIXTURE)
WGSL_TEST_SUPPLIER_FIXTURE[WGSL_SUPPLY_FILE] = WGSL_SUPPLY_FIXTURE[WGSL_SUPPLY_FILE] + '''#[cfg(test)]
mod tests {
    #[test]
    fn the_supplied_kernel_text_is_built() {
        assert!(!super::supplied_kernel_text().is_empty());
    }
    #[test]
    fn the_words_of_the_failure_points_are_listed() {
        let words = "FAILURE_POINT_UNREADABLE FAILURE_POINT_NONE";
        let _root = std::env::var_os("CARGO_MANIFEST_DIR");
        assert!(!words.is_empty());
    }
}
'''


def wgsl_defined_suffix_cell(work):
    """条目级宏调用定义的 GPU_FAILURE_POINT_NONE 以 _ 加外名收尾：FAILURE_POINT_NONE 有着落、不拒算（不靠首段规则：其余那一段 POINT_NONE 哪儿都没有）。"""
    base = os.path.join(work, "tree")
    write_tree(base, WGSL_SUPPLY_FIXTURE)
    key, forms = fixture_key(base, f"{WGSL_SUPPLY_FILE}::failure_point_kernel_text")
    return key is not None, f"不该拒算，实际拒算 {sorted(forms)}"


def wgsl_prefix_remainder_cell(work):
    """外名 FOO_BAR 的首段 FOO_ 只落在一个无关的说明串里、其余那一段 BAR 哪儿都没有：拒算 wgsl-undefined-name。"""
    base = os.path.join(work, "tree")
    write_tree(base, WGSL_SUPPLY_FIXTURE)
    closure = closure_of(base, f"{WGSL_SUPPLY_FILE}::unrelated_kernel_text")
    refused = [refusal.snippet for refusal in closure.refusals if refusal.form == "wgsl-undefined-name"]
    return refused == ["FOO_BAR"], f"要恰好拒算 FOO_BAR，实际 {[refusal.text() for refusal in closure.refusals]}"


def wgsl_supplier_edge_cell(work):
    """供外名的条目级宏调用不经调用体里的名字点到（拼内核的函数只用宏体里定义的 GPU_FAILURE_POINTS）：着落做成边，宏调用并进闭包，改它里面的码键变。
    登记时套 with_breaks_added 关掉宏体字面名字那条边，只看着落。"""
    base = os.path.join(work, "tree")
    write_tree(base, WGSL_SUPPLY_FIXTURE)
    entry = f"{WGSL_SUPPLY_FILE}::supplied_kernel_text"
    members = closure_member_names(base, entry)
    before, before_refused = fixture_key(base, entry)
    edit_file(base, WGSL_SUPPLY_FILE, "GPU_FAILURE_POINT_UNREADABLE = 9,", "GPU_FAILURE_POINT_UNREADABLE = 99,")
    after, _ = fixture_key(base, entry)
    macro_in = any(name.startswith("item_macro failure_points!@") for name in members)
    return macro_in and before is not None and after is not None and before != after, \
        f"闭包里有没有 failure_points! 调用 {macro_in}；改码前键 {before}（拒算 {sorted(before_refused)}）、后 {after}"


def wgsl_test_supplier_cell(work):
    """入口是一条测试，WGSL 外名只有宏调用真给着落；同一模块里另一条测试的串里也写着这两个外名、函数体读运行期的清单目录：
    测试函数不当名字供给者收（不并进闭包、不带进它的 runtime-path 拒算），键算得出；改那条测试的函数体键不变；改宏调用里的码键变。
    登记时套 with_breaks_added 关掉宏体字面名字那条边：开着它，宏调用经 GPU_FAILURE_POINTS 进闭包、外名都有着落，供给者一个都不拉，测试函数收不收判不出。"""
    base = os.path.join(work, "tree")
    write_tree(base, WGSL_TEST_SUPPLIER_FIXTURE)
    entry = f"{WGSL_SUPPLY_FILE}::tests::the_supplied_kernel_text_is_built"
    pulled = "fn tests::the_words_of_the_failure_points_are_listed" in closure_member_names(base, entry)
    before, before_refused = fixture_key(base, entry)
    edit_file(base, WGSL_SUPPLY_FILE, 'let words = "FAILURE_POINT_UNREADABLE FAILURE_POINT_NONE";', 'let words = "FAILURE_POINT_UNREADABLE FAILURE_POINT_NONE，改了";')
    outside, _ = fixture_key(base, entry)
    edit_file(base, WGSL_SUPPLY_FILE, "GPU_FAILURE_POINT_UNREADABLE = 9,", "GPU_FAILURE_POINT_UNREADABLE = 99,")
    inside, _ = fixture_key(base, entry)
    ok = not pulled and before is not None and outside == before and inside is not None and inside != before
    return ok, (f"测试函数并进了闭包 {pulled}（要没有）；键 {before}（拒算 {sorted(before_refused)}）、改测试函数体后 {outside}（要同前）、改宏调用里的码后 {inside}（要不同）")


def wgsl_single_module_cell(work):
    """闭包读到的几份 .wgsl 合成一个模块：split.wgsl 用的 doubled 在 prelude.wgsl 里（拼它的函数与点名 PRELUDE_SOURCE 的函数是两个），不拒算、doubled 进闭包。"""
    base = os.path.join(work, "tree")
    write_tree(base, WGSL_SUPPLY_FIXTURE)
    closure = closure_of(base, f"{WGSL_SUPPLY_FILE}::split_kernel_text")
    names = sorted(unit.name for unit in closure.order if unit.language == "wgsl")
    return not closure.refusals and "doubled" in names, f"WGSL 成员 {names}，拒算 {[refusal.text() for refusal in closure.refusals]}"



def trait_self_path_cell(work):
    """trait 成员里的 Self::scale() 落到这个 trait 自己的成员 scale：改 scale 的默认实现，measure 的键变。"""
    base = os.path.join(work, "tree")
    write_tree(base, TRAIT_SELF_FIXTURE)
    entry = "crates/ftrait/src/lib.rs::measure"
    members = closure_member_names(base, entry)
    before, _ = fixture_key(base, entry)
    edit_file(base, "crates/ftrait/src/lib.rs", "        3\n", "        4\n")
    after, _ = fixture_key(base, entry)
    return "fn <trait Shape>::scale" in members and before is not None and before != after, \
        f"measure 的闭包里有没有 <trait Shape>::scale {'fn <trait Shape>::scale' in members}；改 scale 前键 {before}、后 {after}"



def doctest_structure_cell(work):
    """文档测试是另编的一个 crate：不连它挂的条目、也不连那个模块的 use；改所挂条目 owner 的函数体，文档测试入口的键不变，改它调的 shown 键变。"""
    base = os.path.join(work, "tree")
    write_tree(base, DOCTEST_STRUCTURE_FIXTURE)
    path = "crates/fdoc/src/lib.rs"
    doctests = [unit.name for unit in RustWorkspace(base).file(path).units if unit.kind == "doctest"]
    if len(doctests) != 1:
        return False, f"要恰好一个文档测试单元，实际 {doctests}"
    entry = f"{path}::{doctests[0]}"
    members = closure_member_names(base, entry)
    before, _ = fixture_key(base, entry)
    edit_file(base, path, "    10\n", "    11\n")
    owner_edited, _ = fixture_key(base, entry)
    edit_file(base, path, "    1\n", "    2\n")
    shown_edited, _ = fixture_key(base, entry)
    linked = sorted(name for name in members if name in ("fn owner",) or name.startswith("use "))
    ok = not linked and None not in (before, owner_edited, shown_edited) and before == owner_edited and owner_edited != shown_edited
    return ok, f"文档测试的闭包里连上了 {linked}；键：改前 {before}、改 owner 后 {owner_edited}、改 shown 后 {shown_edited}"



def use_self_import_cell(work):
    """use crate::store::{self} 引进名字 store：只写 store::open() 的单元连上这条 use，改这条 use 作废。"""
    base = os.path.join(work, "tree")
    write_tree(base, USE_SELF_FIXTURE)
    entry = "crates/fuse/src/lib.rs::user::reader"
    members = closure_member_names(base, entry)
    uses = sorted(name for name in members if name.startswith("use "))
    before, _ = fixture_key(base, entry)
    edit_file(base, "crates/fuse/src/lib.rs", "use crate::store::{self};", "use crate::store::{self, open};")
    after, _ = fixture_key(base, entry)
    return uses == ["use user::use@7"] and before is not None and before != after, f"reader 的闭包里 use {uses}；改那条 use 前键 {before}、后 {after}"



def open_trait_argument_cell(work):
    """R-开放只按 trait 自己的名字找 impl：入口签名里的 dyn Sink 带进 impl Sink for Meter（改它键变），不带只在 trait 参数里写了 Sink 的 impl Wrap<dyn Sink> for Holder（改它键不变）。"""
    base = os.path.join(work, "tree")
    write_tree(base, OPEN_TRAIT_ARGUMENT_FIXTURE)
    entry = f"{OPEN_TRAIT_ARGUMENT_FILE}::feed"
    members = closure_member_names(base, entry)
    before, _ = fixture_key(base, entry)
    edit_file(base, OPEN_TRAIT_ARGUMENT_FILE, "        2\n", "        3\n")
    holder_edited, _ = fixture_key(base, entry)
    edit_file(base, OPEN_TRAIT_ARGUMENT_FILE, "        1\n", "        4\n")
    meter_edited, _ = fixture_key(base, entry)
    holder_in = sorted(name for name in members if "Holder" in name)
    ok = not holder_in and None not in (before, holder_edited, meter_edited) and before == holder_edited and holder_edited != meter_edited
    return ok, f"feed 的闭包里有 Holder 的 {holder_in}；键：改前 {before}、改 Holder 的 wrap 后 {holder_edited}、改 Meter 的 level 后 {meter_edited}"



def blanket_method_cell(work):
    """blanket impl（impl<T: Copy> Shout for T）够得到的方法算边：yell 调 value.shout()，闭包里有 blanket impl 的 shout，改它键变。"""
    base = os.path.join(work, "tree")
    write_tree(base, BLANKET_FIXTURE)
    entry = f"{BLANKET_FILE}::yell"
    members = closure_member_names(base, entry)
    before, _ = fixture_key(base, entry)
    edit_file(base, BLANKET_FILE, "        6\n", "        7\n")
    after, _ = fixture_key(base, entry)
    impl_members = sorted(name for name in members if name.startswith("fn <impl"))
    return bool(impl_members) and before is not None and after is not None and before != after, \
        f"yell 的闭包里 impl 成员 {impl_members}；改 blanket impl 的 shout 前键 {before}、后 {after}"



def blanket_shapes_cell(work):
    """heaviest 调 pair.weigh()：六份 impl 的自身类型里都出现了 impl 的泛型参数，都按 blanket 放行、都进闭包；改 Sealed<T> 那一份的 weigh 键变。"""
    base = os.path.join(work, "tree")
    write_tree(base, BLANKET_SHAPES_FIXTURE)
    entry = f"{BLANKET_SHAPES_FILE}::heaviest"
    closure = closure_of(base, entry)
    heads = {unit.name for unit in closure.order if unit.kind == "impl"}
    missing = [shape for shape in BLANKET_SHAPES_SELF_TYPES
               if not any("for" + shape.replace(" ", "") in name.replace(" ", "") for name in heads)]
    before, _ = fixture_key(base, entry)
    edit_file(base, BLANKET_SHAPES_FILE, "        5\n", "        6\n")
    after, _ = fixture_key(base, entry)
    return not missing and before is not None and after is not None and before != after, \
        f"闭包里的 impl 头 {sorted(heads)}，缺 {missing}；改 Sealed<T> 那一份的 weigh 前键 {before}、后 {after}"



def wgsl_other_file_definition_cell(work):
    """kernel_text 的闭包只读 kernel.wgsl，它用的 SHARED_LIMIT 定义在 shared.wgsl：读 shared.wgsl 的 SHARED_SOURCE 并进闭包、shared.wgsl 进模块，
    不拒算；改 shared.wgsl 里那个常量键变。"""
    base = os.path.join(work, "tree")
    write_tree(base, WGSL_OTHER_FILE_FIXTURE)
    entry = f"{WGSL_OTHER_FILE_RUST}::kernel_text"
    closure = closure_of(base, entry)
    names = {f"{unit.language} {unit.name}" for unit in closure.order}
    refused = sorted(refusal.text() for refusal in closure.refusals)
    before, _ = fixture_key(base, entry)
    edit_file(base, "crates/fkernels/src/shared.wgsl", "7u", "8u")
    after, _ = fixture_key(base, entry)
    pulled = {"rust SHARED_SOURCE", "wgsl SHARED_LIMIT"} <= names
    return pulled and not refused and before is not None and after is not None and before != after, \
        f"闭包里有 SHARED_SOURCE 与 SHARED_LIMIT：{pulled}（成员 {sorted(names)}）；拒算 {refused}；改常量前键 {before}、后 {after}"



def underscore_const_cell(work):
    """const _ 没有名字：pick 里的 let _ 与 _ => 不把 gpu 特性下的 const _ 拉进闭包，带不带 gpu 特性两种目标下两把键都相同、闭包里没有 const _。"""
    base = os.path.join(work, "tree")
    write_tree(base, UNDERSCORE_FIXTURE)
    entry = f"{UNDERSCORE_FILE}::pick"
    without = cfg_keys(base, entry, [])
    with_gpu = cfg_keys(base, entry, ["funderscore/gpu"])
    workspace = RustWorkspace(base)
    _language, path, qualified = parse_entry(base, entry)
    unit, target_id = workspace.find_entry(path, qualified, None)
    closure = rust_closure_of(workspace, unit, target_id, None, False, ["funderscore/gpu"], False)
    underscores = sorted(f"{member.kind} {member.name}" for member in closure.order if not member.data and member.short == "_")
    return None not in without + with_gpu and without == with_gpu and not underscores, \
        f"不带 gpu {without}、带 gpu {with_gpu}；带 gpu 时闭包里的 _ 单元 {underscores}"



def outside_include_cell(work):
    """根是 <树>/workspace：include_str! 读根外 <树>/thread-default 的入口出键、改那份文件键变；读根外不在的文件拒算 include-missing。"""
    tree = os.path.join(work, "tree")
    write_tree(tree, OUTSIDE_INCLUDE_FIXTURE)
    base = os.path.join(tree, "workspace")
    before, before_refused = fixture_key(base, f"{OUTSIDE_INCLUDE_FILE}::threads")
    edit_file(tree, "thread-default", "31\n", "16\n")
    after, after_refused = fixture_key(base, f"{OUTSIDE_INCLUDE_FILE}::threads")
    missing, missing_refused = fixture_key(base, f"{OUTSIDE_INCLUDE_FILE}::missing")
    ok = None not in (before, after) and before != after and missing is None and "include-missing" in missing_refused
    return ok, (f"threads：改前键 {before}（拒算 {sorted(before_refused)}）、改根外文件后 {after}（拒算 {sorted(after_refused)}）；"
                f"missing：键 {missing}、拒算 {sorted(missing_refused)}")


def concatenated_includes_cell(work):
    """CONCATENATED_INCLUDES_CELL：concat!(include_str!("header.txt"), include_str!("body.txt")) 的两份文件在 dump 里各按读它的那一行列一行 data
    （data<TAB>数据文件<TAB>读它的文件:行-行<TAB>字节 sha256），改 body.txt 那一份 sources 的键变。弄坏开关 rust-concatenated-includes-skipped 下两行都没有、键不变。"""
    base = os.path.join(work, "tree")
    write_tree(base, CONCATENATED_INCLUDES_FIXTURE)
    lines = dump_lines(base, "rust")[0]
    wanted = [f"data\t{path}\t{CONCATENATED_INCLUDES_FILE}:{line}-{line}\t{sha256_bytes(content)}" for path, line, content in CONCATENATED_INCLUDES_LINES]
    missing = [line for line in wanted if line not in lines]
    before, before_refused = fixture_key(base, f"{CONCATENATED_INCLUDES_FILE}::sources")
    edit_file(base, "crates/fconcat/src/body.txt", "body\n", "body changed\n")
    after, after_refused = fixture_key(base, f"{CONCATENATED_INCLUDES_FILE}::sources")
    ok = not missing and None not in (before, after) and before != after
    return ok, (f"dump 里缺 {missing}；sources 的键改 body.txt 前 {before}（拒算 {sorted(before_refused)}）、"
                f"改后 {after}（拒算 {sorted(after_refused)}）")


def unit_divergence_cells():
    return [
        ("cfg 成分：闭包没引到 gpu 的入口，带不带 gpu 特性两种目标下键相同", cfg_unreferenced_cell),
        ("cfg 成分：闭包里有 cfg!、语句上的 #[cfg] 或 #[cfg_attr] 引到 gpu 的入口，带不带 gpu 特性键不同", cfg_referenced_cell),
        ("WGSL 外名：条目级宏调用定义的名字以 _ 加外名收尾就有着落", wgsl_defined_suffix_cell),
        ("WGSL 外名：首段只落在无关的串里、其余那一段哪儿都没有的拒算", wgsl_prefix_remainder_cell),
        # 这两格只看着落那条边：GPU_FAILURE_POINTS 是宏体展开式顶层按字面写出的名字，宏体字面名字那条边（文件头 Rust 闭包「条目级宏调用生成的条目」）
        # 也让 failure_points! 调用进闭包，开着它两格的弄坏开关红不了；crates/singlefs-unit-graph 的 unit_graph_wgsl_modules_close_over_compute_entries.rs 同样关它
        ("WGSL 外名的着落做成边：只靠外名点到的条目级宏调用并进闭包，改它里面的码键变",
         with_breaks_added(wgsl_supplier_edge_cell, "rust-macro-rules-literal-names-unlinked")),
        (WGSL_TEST_SUPPLIER_CELL, with_breaks_added(wgsl_test_supplier_cell, "rust-macro-rules-literal-names-unlinked")),
        ("WGSL：闭包读到的几份 .wgsl 合成一个模块，名字跨文件解", wgsl_single_module_cell),
        ("R-路径：trait 成员里的 Self::x 落到这个 trait 自己的成员 x", trait_self_path_cell),
        ("R-结构：文档测试不连所挂的条目与那个模块的 use", doctest_structure_cell),
        ("R-结构：use a::{self} 引进名字 a，只写 a::f() 的单元连上这条 use", use_self_import_cell),
        ("R-开放：dyn Trait 只带 Trait 自己的 impl，不带只在 trait 参数里写了它的 impl", open_trait_argument_cell),
        ("R-方法：blanket impl 够得到的方法算边，改它作废", blanket_method_cell),
        ("R-裸名：const _ 没有名字，let _ 与 _ => 不把 gpu 特性下的 const _ 拉进闭包，带不带 gpu 特性键相同", underscore_const_cell),
        ("include_str! 读根外的文件：读得到收成数据单元、改它键变，读不到拒算 include-missing", outside_include_cell),
        ("R-方法：自身类型里出现泛型参数的 impl（(T, u32)、[T]、[T; N]、Box<T>、Option<Box<T>>、Sealed<T>）都按 blanket 放行，改 Sealed<T> 那一份作废",
         blanket_shapes_cell),
        ("WGSL 外名在链接集里别的 include_str! 读进来的 .wgsl 里有定义：读它的 Rust 单元与那份 .wgsl 拉进闭包，改那个常量作废", wgsl_other_file_definition_cell),
        (CONCATENATED_INCLUDES_CELL, concatenated_includes_cell),
    ]


def rust_unbalanced_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, RUST_FIXTURE)
    edit_file(base, "crates/fextra/src/lib.rs", "    Shade::Dark as u32 + 1\n}", "    Shade::Dark as u32 + 1\n")
    key, forms = fixture_key(base, f"{RUST_FIXTURE_ENTRY_FILE}::judge_listing")
    return key is None and "unbalanced-brackets" in forms, f"要拒算 unbalanced-brackets，实际 {sorted(forms)}"


def wgsl_undefined_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, RUST_FIXTURE)
    edit_file(base, "crates/fextra/src/common.wgsl", "x + GENERATED_LIMIT", "x + MISSING_EVERYWHERE")
    key, forms = fixture_key(base, f"{RUST_FIXTURE_ENTRY_FILE}::judge_kernel")
    return key is None and "wgsl-undefined-name" in forms, f"要拒算 wgsl-undefined-name，实际 {sorted(forms)}"


def python_rta_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, PYTHON_FIXTURE)
    closure = closure_of(base, "pkg/a.py::use_reader")
    names = {unit.name for unit in closure.order}
    return "Reader.read" in names and "Writer.read" not in names, f"闭包里方法：{sorted(name for name in names if '.' in name)}"


def python_spec_cell(work):
    ok, detail = compare_cell(PYTHON_FIXTURE, "pkg/a.py::entry", "pkg/b.py", "    return 2", "    return 20", "changed")(work)
    return ok, detail


def shell_heredoc_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, SHELL_FIXTURE)
    index = ShellIndex(base, ["s/main.sh"])
    shell_file = index.files["s/main.sh"]
    emits = shell_file.functions.get("emit", [])
    if len(emits) != 1 or "second" not in emits[0].body or "second" in shell_file.script_body.body:
        return False, f"emit 单元 {len(emits)} 个，含 second：{[('second' in unit.body) for unit in emits]}"
    before, _ = fixture_key(base, "s/main.sh::emit")
    beta_before, _ = fixture_key(base, "s/main.sh::cell_beta")
    edit_file(base, "s/main.sh", "second\n", "second changed\n")
    after, _ = fixture_key(base, "s/main.sh::emit")
    beta_after, _ = fixture_key(base, "s/main.sh::cell_beta")
    return before != after and beta_before == beta_after and before is not None, "here-doc 正文改了：emit 的键要变、cell_beta 的不变"


def shell_names_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, SHELL_FIXTURE)
    closure = closure_of(base, "s/main.sh")
    names = {unit.name for unit in closure.order if unit.file == "s/main.sh"}
    return "cell_alpha" in names and "cell_beta" not in names, f"main.sh 的闭包里：{sorted(names)}"


def shell_folding_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, SHELL_FIXTURE)
    closure = closure_of(base, "s/main.sh")
    files = {unit.file for unit in closure.order}
    key, refusals = closure_key(closure)
    return {"s/lib.sh", "s/child.sh"} <= files and key is not None, f"闭包碰到的文件 {sorted(files)}，拒算 {[refusal.form for refusal in refusals]}"



def shell_data_edit_trip(work, entry, path, old, new):
    """→ (改动前的键, 改后键, 改动前闭包里经 shell-source 收进来的文件, 改动前与改后的 unit_graph_key)。"""
    base = os.path.join(work, "tree")
    write_tree(base, SHELL_DATA_FIXTURE)
    closure = closure_of(base, f"{SHELL_DATA_FILE}::{entry}")
    sourced = sorted(unit.file for unit in closure.order if closure.via.get(id(unit), (None, ""))[1] == "shell-source")
    graph_before = unit_graph_key(closure, [SHELL_DATA_HOST_LINE])[0]
    before = closure_key(closure)[0]
    edit_file(base, path, old, new)
    changed = closure_of(base, f"{SHELL_DATA_FILE}::{entry}")
    return before, closure_key(changed)[0], sourced, (graph_before, unit_graph_key(changed, [SHELL_DATA_HOST_LINE])[0])


def shell_data_source_cell(work):
    before, after, sourced, graphs = shell_data_edit_trip(work, "runs_the_sourcing_script", "scripts/lib/first.sh", "  echo first\n", "  echo first, changed\n")
    base = os.path.join(work, "tree")
    plain_after = fixture_key(base, f"{SHELL_DATA_FILE}::runs_the_plain_script")[0]
    edit_file(base, "scripts/lib/first.sh", "  echo first, changed\n", "  echo first\n")
    plain_before = fixture_key(base, f"{SHELL_DATA_FILE}::runs_the_plain_script")[0]
    # 数据位置（data_sites：读它的文件、行、种类）：被 source 的两份都落在 lib.rs 里读 sourcing.sh 的那一处（crates/singlefs-unit-graph 的 follow_shell_sources 同）
    sites = {site[2]: (site[0], site[1], site[5]) for site in closure_of(base, f"{SHELL_DATA_FILE}::runs_the_sourcing_script").data_sites}
    reader = sites.get("scripts/sourcing.sh")
    sited = reader is not None and all(sites.get(path) == reader for path in ("scripts/lib/first.sh", "scripts/lib/second.sh"))
    ok = ("scripts/lib/first.sh" in sourced and before is not None and after is not None and before != after
          and plain_before is not None and plain_before == plain_after and sited and graphs[0] is not None and graphs[1] is not None and graphs[0] != graphs[1])
    return ok, (f"经 shell-source 收进来的 {sourced}（要有 scripts/lib/first.sh）；改它前后键 {before and before[:12]} → {after and after[:12]}（要都有、不同）；"
                f"读 plain.sh 的入口改前改后 {plain_before and plain_before[:12]} / {plain_after and plain_after[:12]}（要都有、相同）；"
                f"数据位置 {dict(sorted(sites.items()))}（被 source 的 first.sh、second.sh 要与 sourcing.sh 落在同一处）；"
                f"unit_graph_key {graphs[0] and graphs[0][:12]} → {graphs[1] and graphs[1][:12]}（要都有、不同）")


def shell_data_dot_cell(work):
    before, after, sourced, _graphs = shell_data_edit_trip(work, "runs_the_dotted_script", "scripts/lib/dot-only.sh", "  echo dot\n", "  echo dot, changed\n")
    ok = sourced == ["scripts/lib/dot-only.sh"] and before is not None and after is not None and before != after
    return ok, f"经 shell-source 收进来的 {sourced}（要 [scripts/lib/dot-only.sh]）；改它前后键 {before and before[:12]} → {after and after[:12]}（要都有、不同）"


def shell_data_nested_cell(work):
    before, after, sourced, _graphs = shell_data_edit_trip(work, "runs_the_sourcing_script", "scripts/lib/second.sh", "  echo second\n", "  echo second, changed\n")
    ok = sourced == ["scripts/lib/first.sh", "scripts/lib/second.sh"] and before is not None and after is not None and before != after
    return ok, (f"经 shell-source 收进来的 {sourced}（要 [scripts/lib/first.sh, scripts/lib/second.sh]）；改里层 second.sh 前后键 "
                f"{before and before[:12]} → {after and after[:12]}（要都有、不同）")



def manifest_root_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, MANIFEST_ROOT_FIXTURE)
    directory_key, directory_forms = fixture_key(base, "crates/fbench/src/lib.rs::root_by_parent_directories")
    before, named_forms = fixture_key(base, "crates/fbench/src/lib.rs::root_by_named_source")
    edit_file(base, "crates/fbench/src/lib.rs", "// 只是注释\n", "// 只是注释，改了措辞\n")
    after, _ = fixture_key(base, "crates/fbench/src/lib.rs::root_by_named_source")
    ok = directory_key is None and "runtime-path" in directory_forms and before is not None and after is not None and before != after
    return ok, (f"往上拼目录的：键{'有' if directory_key else '无'}、拒算 {sorted(directory_forms)}（要拒算 runtime-path）；"
                f"点名文件的：拒算 {sorted(named_forms)}，改那份文件里一句注释前后键{'相同' if before == after else '不同'}（要算得出且不同）")



def current_exe_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, CURRENT_EXE_FIXTURE)
    file = "crates/fexe/src/lib.rs"
    kept = {name: fixture_key(base, f"{file}::tests::{name}") for name in CURRENT_EXE_KEYED}
    refused = {name: fixture_key(base, f"{file}::tests::{name}")[1] for name in CURRENT_EXE_REFUSED}
    reached = {}
    for name in CURRENT_EXE_KEYED:
        found = closure_of(base, f"{file}::tests::{name}")
        reached[name] = {unit.name: found.via.get(id(unit), (None, ""))[1] for unit in found.order}.get("tests::the_child_runs_alone")
    edit_file(base, file, "        let note = 5;\n        assert_eq!(note, 5);", "        let note = 6;\n        assert_eq!(note, 6);")
    outside = {name: fixture_key(base, f"{file}::tests::{name}")[0] for name in CURRENT_EXE_KEYED}
    edit_file(base, file, "        let value = 1;\n        assert_eq!(value, 1);", "        let value = 2;\n        assert_eq!(value, 2);")
    inside = {name: fixture_key(base, f"{file}::tests::{name}")[0] for name in CURRENT_EXE_KEYED}
    ok = (all(kept[name][0] is not None for name in CURRENT_EXE_KEYED) and all(rule == "current-exe-child" for rule in reached.values())
          and all(forms == {"current-exe"} for forms in refused.values())
          and all(outside[name] == kept[name][0] for name in CURRENT_EXE_KEYED)
          and all(inside[name] is not None and inside[name] != kept[name][0] for name in CURRENT_EXE_KEYED))
    return ok, (f"算得出键的入口：{ {name: kept[name][0] is not None for name in CURRENT_EXE_KEYED} }，拒算 {[forms for key, forms in kept.values() if key is None]}；"
                f"子进程测试进闭包的规则 {reached}（要都是 current-exe-child）；点名不存在的、名字在 --exact 前面的拒算 {refused}（要都是 current-exe）；"
                f"改无关的测试后键同前：{[outside[name] == kept[name][0] for name in CURRENT_EXE_KEYED]}；改子进程测试的函数体后键都变：{[inside[name] != kept[name][0] for name in CURRENT_EXE_KEYED]}")



def current_exe_constant_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, CURRENT_EXE_CONSTANT_FIXTURE)
    file = "crates/fexec/src/lib.rs"
    keyed, ambiguous = f"{file}::tests::the_parent_names_the_child_by_a_constant_from_another_file", f"{file}::tests::the_parent_names_a_constant_defined_twice_with_different_values"
    kept, kept_forms = fixture_key(base, keyed)
    ambiguous_key, ambiguous_forms = fixture_key(base, ambiguous)
    found = closure_of(base, keyed)
    reached = {unit.name: found.via.get(id(unit), (None, ""))[1] for unit in found.order}.get("tests::the_child_runs_alone")
    edit_file(base, file, "        let note = 5;\n        assert_eq!(note, 5);", "        let note = 6;\n        assert_eq!(note, 6);")
    outside = fixture_key(base, keyed)[0]
    edit_file(base, file, "        let value = 1;\n        assert_eq!(value, 1);", "        let value = 2;\n        assert_eq!(value, 2);")
    inside = fixture_key(base, keyed)[0]
    ok = (kept is not None and reached == "current-exe-child" and ambiguous_key is None and ambiguous_forms == {"current-exe"}
          and outside == kept and inside is not None and inside != kept)
    return ok, (f"常量在另一份文件里的入口：键{'有' if kept else '无'}（拒算 {sorted(kept_forms)}）、子进程测试进闭包的规则 {reached}（要 current-exe-child）；"
                f"同名常量两份值不同的：键{'有' if ambiguous_key else '无'}、拒算 {sorted(ambiguous_forms)}（要无键、拒算 current-exe）；"
                f"改无关的测试后键同前：{outside == kept}；改子进程测试的函数体后键变：{inside is not None and inside != kept}")



def current_exe_path_only_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, CURRENT_EXE_PATH_ONLY_FIXTURE)
    file = "crates/fexp/src/lib.rs"
    kept = {name: fixture_key(base, f"{file}::tests::{name}") for name in CURRENT_EXE_PATH_ONLY_ENTRIES}
    edit_file(base, file, "        let note = 5;\n        assert_eq!(note, 5);", "        let note = 6;\n        assert_eq!(note, 6);")
    outside = {name: fixture_key(base, f"{file}::tests::{name}")[0] for name in CURRENT_EXE_PATH_ONLY_ENTRIES}
    edit_file(base, file, "        assert!(std::env::current_exe().is_ok());", "        assert!(std::env::current_exe().is_err());")
    edit_file(base, file, "expect(\"读得到\").len();\n        assert!(size > 0);", "expect(\"读得到\").len();\n        assert!(size > 1);")
    inside = {name: fixture_key(base, f"{file}::tests::{name}")[0] for name in CURRENT_EXE_PATH_ONLY_ENTRIES}
    ok = (all(kept[name][0] is not None and not kept[name][1] for name in CURRENT_EXE_PATH_ONLY_ENTRIES)
          and all(outside[name] == kept[name][0] for name in CURRENT_EXE_PATH_ONLY_ENTRIES)
          and all(inside[name] is not None and inside[name] != kept[name][0] for name in CURRENT_EXE_PATH_ONLY_ENTRIES))
    return ok, (f"算得出键的入口：{ {name: kept[name][0] is not None for name in CURRENT_EXE_PATH_ONLY_ENTRIES} }，拒算 {[sorted(kept[name][1]) for name in CURRENT_EXE_PATH_ONLY_ENTRIES]}（要都不拒算）；"
                f"改无关的测试后键同前：{[outside[name] == kept[name][0] for name in CURRENT_EXE_PATH_ONLY_ENTRIES]}；"
                f"改它们自己的函数体后键都变：{[inside[name] != kept[name][0] for name in CURRENT_EXE_PATH_ONLY_ENTRIES]}")



def current_exe_spawn_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, CURRENT_EXE_SPAWN_FIXTURE)
    file = "crates/fexs/src/lib.rs"
    kept, kept_forms = fixture_key(base, f"{file}::tests::{CURRENT_EXE_SPAWN_KEYED}")
    refused = {name: fixture_key(base, f"{file}::tests::{name}") for name in CURRENT_EXE_SPAWN_REFUSED}
    found = closure_of(base, f"{file}::tests::{CURRENT_EXE_SPAWN_KEYED}")
    reached = {unit.name: found.via.get(id(unit), (None, ""))[1] for unit in found.order}.get("tests::the_child_runs_alone")
    edit_file(base, file, "        let value = 1;\n        assert_eq!(value, 1);", "        let value = 2;\n        assert_eq!(value, 2);")
    inside = fixture_key(base, f"{file}::tests::{CURRENT_EXE_SPAWN_KEYED}")[0]
    ok = (kept is not None and reached == "current-exe-child" and inside is not None and inside != kept
          and all(key is None and forms == {"current-exe"} for key, forms in refused.values()))
    return ok, (f"点名了测试的入口：键{'有' if kept else '无'}（拒算 {sorted(kept_forms)}）、子进程测试进闭包的规则 {reached}（要 current-exe-child）、改子进程测试的函数体后键变：{inside is not None and inside != kept}；"
                f"没点名、路径先存变量、路径改过、点名不存在的：{ {name: (key is not None, sorted(forms)) for name, (key, forms) in refused.items()} }（要都无键、拒算 current-exe）")



def manifest_directory_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, MANIFEST_DIRECTORY_FIXTURE)
    file = "crates/fdir/src/lib.rs"
    kept, problems = {}, []
    for name in MANIFEST_DIRECTORY_ENTRIES:
        closure = closure_of(base, f"{file}::{name}")
        key, _forms = fixture_key(base, f"{file}::{name}")
        kept[name] = key
        if key is None or "data-directory" not in {unit.kind for unit in closure.order}:
            problems.append(f"{name}：键{'有' if key else '无'}、闭包里{'有' if any(unit.kind == 'data-directory' for unit in closure.order) else '没有'}数据目录")
    refused = {name: fixture_key(base, f"{file}::{name}")[1] for name in ("count_every_crate", "count_by_climbing_from_a_variable")}
    edit_file(base, file, "    1\n}", "    2\n}")
    outside = {name: fixture_key(base, f"{file}::{name}")[0] for name in MANIFEST_DIRECTORY_ENTRIES}
    edit_file(base, "crates/fdir/rules/nested/second.txt", "rule two", "rule two, changed")
    inside = {name: fixture_key(base, f"{file}::{name}")[0] for name in MANIFEST_DIRECTORY_ENTRIES}
    ok = (not problems and all(forms == {"runtime-path"} for forms in refused.values()) and outside == kept
          and all(inside[name] is not None and inside[name] != kept[name] for name in MANIFEST_DIRECTORY_ENTRIES))
    return ok, (f"没算出键或闭包里没有数据目录：{problems}；crates/ 整个与变量上接 .parent() 的拒算 {refused}（要都是 runtime-path）；"
                f"改闭包外的函数后键同前：{outside == kept}；改目录里一份文件后三个入口的键都变：{[inside[name] != kept[name] for name in MANIFEST_DIRECTORY_ENTRIES]}")



def manifest_binding_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, MANIFEST_BINDING_FIXTURE)
    file = "crates/fpre/src/lib.rs"
    digest = sha256_bytes(MANIFEST_BINDING_FIXTURE["crates/fpre/tools/check.py"].encode("utf-8"))
    reads = sorted(line for line in dump_lines(base, "rust")[0] if line.startswith(("data\t", "data-directory\t")))
    want_reads = sorted(f"data\tcrates/fpre/tools/check.py\t{file}:{line}-{line}\t{digest}" for line, _refused in MANIFEST_BINDING_ENTRIES.values())
    keys = {name: fixture_key(base, f"{file}::{name}") for name in MANIFEST_BINDING_ENTRIES}
    wrong = {name: (key is not None, sorted(forms)) for name, (key, forms) in keys.items()
             if (key is None, forms) != (MANIFEST_BINDING_ENTRIES[name][1], {"runtime-path"} if MANIFEST_BINDING_ENTRIES[name][1] else set())}
    ok = reads == want_reads and not wrong
    return ok, (f"dump 的数据读 {reads}（要 {want_reads}）；键与拒算对不上的入口 (有没有键, 拒算) {wrong}"
                f"（要 check_from_the_manifest_directory、check_from_the_crates_directory 无键、拒算 runtime-path，check_through_an_alias 有键、不拒算）")



def rust_lex_cache_cell(work):
    """缓存放在这一格自己的目录里（FUNCTION_UNITS_LEX_CACHE_DIRECTORY 临时换成它）：同一份文本头一趟现切、第二趟取缓存且与 rust_lex 现切的逐项相同；
    换一份同样长、差一个字符的文本：现切、与它的 rust_lex 逐项相同、与头一份不同（弄坏开关 rust-lex-cache-unverified 下文件名只取长度，这里沿用了头一份）；
    缓存文件写坏了：照常现切、结果同前。"""
    first_text = "pub fn answer() -> u32 {\n    // note\n    /// documented\n    41 + 1\n}\n"
    second_text = first_text.replace("41", "43")
    previous = os.environ.get(RUST_LEX_CACHE_DIRECTORY_VARIABLE)
    os.environ[RUST_LEX_CACHE_DIRECTORY_VARIABLE] = os.path.join(work, "rust-lex-cache")

    def trip(text):
        before = dict(RUST_LEX_CACHE_STATS)
        result = rust_lex_cached(text)
        return result, RUST_LEX_CACHE_STATS["lexed"] - before["lexed"], RUST_LEX_CACHE_STATS["reused"] - before["reused"]
    try:
        first, second = trip(first_text), trip(first_text)
        changed, changed_again = trip(second_text), trip(second_text)
        with open(rust_lex_cache_path(first_text), "w", encoding="utf-8") as handle:
            handle.write("{broken")   # 不是 zlib 压缩的 json
        damaged = trip(first_text)
    finally:
        if previous is None:
            os.environ.pop(RUST_LEX_CACHE_DIRECTORY_VARIABLE, None)
        else:
            os.environ[RUST_LEX_CACHE_DIRECTORY_VARIABLE] = previous
    direct_first, direct_second = rust_lex(first_text), rust_lex(second_text)
    ok = (direct_second != direct_first and first == (direct_first, 1, 0) and second == (direct_first, 0, 1) and changed == (direct_second, 1, 0)
          and changed_again == (direct_second, 0, 1) and damaged == (direct_first, 1, 0))
    return ok, (f"头一趟 (现切 {first[1]}、取缓存 {first[2]}，同 rust_lex {first[0] == direct_first})、第二趟 ({second[1]}、{second[2]}，{second[0] == direct_first})、"
                f"换文本 ({changed[1]}、{changed[2]}，同它的 rust_lex {changed[0] == direct_second})、再一趟 ({changed_again[1]}、{changed_again[2]}，{changed_again[0] == direct_second})、"
                f"缓存文件写坏 ({damaged[1]}、{damaged[2]}，{damaged[0] == direct_first})（要 头一趟 (1、0)、第二趟 (0、1)、换文本 (1、0)、再一趟 (0、1)、写坏 (1、0)，结果都同现切）")


def library_variant_namespace(work, label, edits):
    """这份库此刻的原文照 edits（(说明, 原文, 改成)，原文要恰好命中一次）改过，另执行成一份模块（不进 sys.modules、不跑它的自证）
    → (它算出的词法缓存目录名, None)；原文命中不是一次的 → (None, 说明)。"""
    with open(os.path.join(LIBRARY_ROOT, "scripts", "gate", "lib_function_units_lexer.py"), encoding="utf-8") as handle:
        text = handle.read()
    for what, old, new in edits:
        if text.count(old) != 1:
            return None, f"{what}：「{old.strip()}」在这份库里命中 {text.count(old)} 次（要恰好一次）：原文改了，照新原文改那一张改法表"
        text = text.replace(old, new)
    module = types.ModuleType(f"lib_function_units_variant_{label}")
    module.__file__ = os.path.join(work, label, "lib_function_units_lexer.py")
    exec(compile(text, module.__file__, "exec"), module.__dict__)
    return module.rust_lex_cache_namespace(), None


def rust_lex_namespace_cell(edits, want):
    """RUST_LEX_NAMESPACE_CHANGED_CELL / RUST_LEX_NAMESPACE_KEPT_CELL：这份库原样一份、照 edits 里每一处各改一份，各另执行成模块算词法缓存的目录名，
    每一份与原样那一份比，变没变都要是 want（changed 或 same）。"""
    def judge(work):
        base, problem = library_variant_namespace(work, "base", ())
        if base is None:
            return False, problem
        wrong = []
        for index, edit in enumerate(edits):
            namespace, problem = library_variant_namespace(work, f"edit-{index}", (edit,))
            if namespace is None:
                return False, problem
            got = "changed" if namespace != base else "same"
            if got != want:
                wrong.append(f"{edit[0]}：目录名{'换了' if got == 'changed' else '没换'}")
        return not wrong, f"要{'都换' if want == 'changed' else '都不换'}，对不上的 {wrong}（目录名照 rust_lex_cache_namespace 与 live_closure_fingerprint 改）"
    return judge


def rust_lex_namespace_closure_cell(work):
    """RUST_LEX_NAMESPACE_CLOSURE_CELL：真仓拷一份（real_tree），照这份库自己的规则收 rust_lex_cached 的闭包：没有这份库整份的 source-text 与 data-file，
    load_rust_lex、store_rust_lex、rust_lex、RUST_LEX_CACHE_FORM、IDENT 在里面；再照 RUST_LEX_NAMESPACE_CLOSURE_EDITS 在拷贝上逐处改（逐个累加），
    改注释那一处键不变、改 load_rust_lex 那一处键变。弄坏开关 rust-lex-cache-namespace-reads-own-file：在拷贝上给 rust_lex_cache_namespace
    加回一句读这份库自己原文的，这一格必红。"""
    tree = real_tree(work)
    library = "scripts/gate/lib_function_units_lexer.py"
    entry = f"{library}::rust_lex_cached"
    if "rust-lex-cache-namespace-reads-own-file" in active_breaks():
        edit_file(tree, library, *RUST_LEX_NAMESPACE_OWN_FILE_EDIT)
    closure = closure_of(tree, entry)
    members = {(unit.kind, unit.name) for unit in closure.order if unit.file == library}
    whole = sorted(f"{kind} {name}" for kind, name in members if kind in ("source-text", "data-file"))
    wanted = ("load_rust_lex", "store_rust_lex", "rust_lex", "RUST_LEX_CACHE_FORM", "IDENT")
    missing = [name for name in wanted if not any(member_name == name for _kind, member_name in members)]
    previous, wrong = closure_key(closure, allow_refused=True)[0], []
    for what, old, new, want in RUST_LEX_NAMESPACE_CLOSURE_EDITS:
        edit_file(tree, library, old, new)
        key = closure_key(closure_of(tree, entry), allow_refused=True)[0]
        if ("changed" if key != previous else "same") != want:
            wrong.append(f"{what}：键{'变了' if key != previous else '没变'}")
        previous = key
    return not whole and not missing and not wrong, (f"闭包里这份库的单元 {len(members)} 个：整份的 {whole}（要没有），该在而不在的 {missing}；"
                                                     f"键对不上的 {wrong}（整份的沿 closure --explain 的 via 找读这份库原文的那一处）")


def excluded_call_edge_cell(work):
    """EXCLUDED_CALL_EDGE_CELL：自证期间临时把夹具里 entry 调 locate 那一条边登进 EXCLUDED_CALL_EDGES（EXCLUDED_CALL_EDGE_FIXTURE_EDGES）：
    改只经 locate 读的 pkg/locator.txt 键不变，改 judge_input 读的 pkg/judged.txt 键变；跑完把表还原。"""
    registered = dict(EXCLUDED_CALL_EDGES)
    EXCLUDED_CALL_EDGES.update(EXCLUDED_CALL_EDGE_FIXTURE_EDGES)
    got = {}
    try:
        for path, old, new in (("pkg/locator.txt", "locator\n", "locator changed\n"), ("pkg/judged.txt", "judged\n", "judged changed\n")):
            base = os.path.join(work, path.replace("/", "-"))
            write_tree(base, EXCLUDED_CALL_EDGE_FIXTURE)
            before = closure_key(closure_of(base, "pkg/tool.py::entry"), allow_refused=True)[0]
            edit_file(base, path, old, new)
            after = closure_key(closure_of(base, "pkg/tool.py::entry"), allow_refused=True)[0]
            got[path] = "changed" if before != after else "same"
    finally:
        EXCLUDED_CALL_EDGES.clear()
        EXCLUDED_CALL_EDGES.update(registered)
    want = {"pkg/locator.txt": "same", "pkg/judged.txt": "changed"}
    return got == want, f"改了键变没变 {got}（要 {want}：登记了的边不连，同一个单元的别的边照连）"


def unit_graph_inputs_cell(work):
    """UNIT_GRAPH_INPUTS_CELL：真仓拷一份（real_tree），crates/singlefs-unit-graph 与 Cargo.lock 经 scripts/hosts/lib_repository_copy.py 拷进去
    （copy_closure 只拷那一个工作区、声明 Cargo.lock，再 copy_entries；两样不在拷出来的树里就判红），照这份库自己的规则收 unit_graph_directory 的闭包：
    built_unit_graph、build_unit_graph、unit_graph_inputs_digest 不在；Cargo.lock 头一个包的版本、.claude/heavy-thread-default、crates/singlefs-unit-graph/src
    里头一份 .rs 各改一处（逐个累加），键都不变。弄坏开关 call-edge-exclusions-ignored 下那一条边照连，这一格必红；
    unit-graph-inputs-lock-undeclared 下不声明 Cargo.lock（拷出来的树里没有它），这一格必红。"""
    tree = real_tree(work)
    repository_copy = repository_copy_module()
    declared = [] if "unit-graph-inputs-lock-undeclared" in active_breaks() else [("Cargo.lock", "改它头一个包的版本，判 unit_graph_directory 的键不变")]
    copied = repository_copy.copy_closure(LIBRARY_ROOT, declared, workspace=os.path.join("crates", "singlefs-unit-graph"))
    repository_copy.copy_entries(LIBRARY_ROOT, tree, copied.entries)
    absent = [path for path in ("Cargo.lock", "crates/singlefs-unit-graph/Cargo.toml") if not os.path.isfile(os.path.join(tree, path))]
    if absent:
        return False, f"经 lib_repository_copy 拷出来的树里缺 {absent}（拷的条目 {sorted(copied.entries)}）"
    library = "scripts/gate/lib_function_units_unit_graph.py"
    entry = f"{library}::unit_graph_directory"
    closure = closure_of(tree, entry)
    base_key = closure_key(closure, allow_refused=True)[0]
    present = sorted(unit.name for unit in closure.order
                     if unit.file == library and unit.name in ("built_unit_graph", "build_unit_graph", "unit_graph_inputs_digest"))
    source = files_under(tree, (".rs",), ["crates/singlefs-unit-graph/src"])[0]
    changed = []
    for path, edit in (("Cargo.lock", lambda text: text.replace('\nversion = "', '\nversion = "0.0.0-', 1)),
                       (".claude/heavy-thread-default", lambda text: text + "# 自证追加的一行\n"),
                       (source, lambda text: text + "// 自证追加的一行\n")):
        absolute = os.path.join(tree, path)
        with open(absolute, encoding="utf-8") as handle:
            text = handle.read()
        with open(absolute, "w", encoding="utf-8") as handle:
            handle.write(edit(text))
        if closure_key(closure_of(tree, entry), allow_refused=True)[0] != base_key:
            changed.append(path)
    return not present and not changed, (f"{entry} 的闭包里有 {present}（要没有）；改了键就变的 {changed}（要没有）：编 unit-graph 那一步照"
                                         "「不走的调用边」EXCLUDED_CALL_EDGES 那一行不进判法闭包")


def unit_graph_digest_cell(work):
    """UNIT_GRAPH_DIGEST_CELL：临时目录里摆一份只有 crates/singlefs-unit-graph（Cargo.toml、src/main.rs）、Cargo.lock、.claude/heavy-thread-default 的小仓，
    unit_graph_inputs_digest 先算一遍，再三样各改一处（逐个累加）各算一遍：每一次都与上一次不同（编 unit-graph 读的三样都罩在缓存目录名里）。"""
    repository = os.path.join(work, "repository")
    write_tree(repository, UNIT_GRAPH_DIGEST_FIXTURE)
    source = os.path.join(repository, "crates", "singlefs-unit-graph")
    thread_registry, lock_file = os.path.join(repository, ".claude", "heavy-thread-default"), os.path.join(repository, "Cargo.lock")
    previous, unchanged = unit_graph_inputs_digest(repository, source, thread_registry, lock_file), []
    for path in ("Cargo.lock", ".claude/heavy-thread-default", "crates/singlefs-unit-graph/src/main.rs"):
        with open(os.path.join(repository, path), "a", encoding="utf-8") as handle:
            handle.write("# 自证追加的一行\n" if not path.endswith(".rs") else "// 自证追加的一行\n")
        digest = unit_graph_inputs_digest(repository, source, thread_registry, lock_file)
        if digest == previous:
            unchanged.append(path)
        previous = digest
    return not unchanged, f"改了缓存目录名却没换的 {unchanged}（要没有：编 unit-graph 读的每一样都要进 unit_graph_inputs_digest）"


def selftest_fixtures_outside_cell(work):
    """SELFTEST_FIXTURES_OUTSIDE_CELL：真仓拷一份（real_tree），收 rust_lex_cached 的闭包（这份库 import 时执行的语句都在里面）：
    lib_function_units_selftest_fixtures.py 的单元一个都不在；那一份末尾加一句常量之后键不变。弄坏开关 selftest-fixtures-loaded-at-import：
    在拷贝上给这份库加一句模块级的赋值点名 load_selftest_fixtures（import 时就载入那一份），这一格必红。"""
    tree = real_tree(work)
    library, fixtures = "scripts/gate/lib_function_units_lexer.py", "scripts/gate/lib_function_units_selftest_fixtures.py"
    entry = f"{library}::rust_lex_cached"
    if "selftest-fixtures-loaded-at-import" in active_breaks():
        edit_file(tree, library, *SELFTEST_FIXTURES_IMPORT_EDIT)
    closure = closure_of(tree, entry)
    inside = sorted(unit.name for unit in closure.order if unit.file.startswith(fixtures[:-len(".py")]))
    before = closure_key(closure, allow_refused=True)[0]
    with open(os.path.join(tree, fixtures), "a", encoding="utf-8") as handle:
        handle.write(SELFTEST_FIXTURES_APPENDED)
    after = closure_key(closure_of(tree, entry), allow_refused=True)[0]
    return not inside and before == after, (f"{entry} 的闭包里有那一份的单元 {len(inside)} 个（头几个 {inside[:5]}，要没有）；"
                                            f"那一份加一句之后键{'变了' if before != after else '没变'}（要没变）：照文件头「自证夹具」只在自证里载入它")


LIBRARY_IMPORT_ALIAS = re.compile(r"^\s*import\s+lib_function_units\s+as\s+(\w+)", re.M)
LIBRARY_IMPORT_PLAIN = re.compile(r"^\s*import\s+lib_function_units\s*(?:#.*)?$", re.M)
LIBRARY_CALL_ALIAS = re.compile(r"^\s*(\w+)\s*=\s*[\w.]*\bfunction_units\(\)", re.M)
LIBRARY_LOAD_ALIAS = re.compile(r"""^\s*(\w+)\s*=\s*\w+\(\s*['"]lib_function_units['"]\s*\)""", re.M)
LIBRARY_SPEC_ALIAS = re.compile(r"^\s*(\w+)\s*=\s*importlib\.util\.spec_from_file_location\([^\n]*function_units", re.M)
LIBRARY_MODULE_FROM_SPEC = re.compile(r"^\s*(\w+)\s*=\s*importlib\.util\.module_from_spec\(\s*(\w+)\s*\)", re.M)
LIBRARY_FROM_IMPORT = re.compile(r"^\s*from\s+lib_function_units\s+import\s+(\([^)]*\)|[^\n]*)", re.M)


def library_aliases(text):
    """一份脚本里指这份库的名字：import lib_function_units [as X]、X = …function_units()（按路径载入这份库的取法）、X = load('lib_function_units')、
    spec_from_file_location 的路径带着 function_units 的那一份 module_from_spec 交的名字。"""
    found = set(LIBRARY_IMPORT_ALIAS.findall(text)) | set(LIBRARY_CALL_ALIAS.findall(text)) | set(LIBRARY_LOAD_ALIAS.findall(text))
    if LIBRARY_IMPORT_PLAIN.search(text):
        found.add("lib_function_units")
    specifications = set(LIBRARY_SPEC_ALIAS.findall(text))
    found |= {module for module, specification in LIBRARY_MODULE_FROM_SPEC.findall(text) if specification in specifications}
    return found


def library_external_calls(root, names):
    """names 里哪几个被别的脚本当这份库的属性点名：提到 lib_function_units 的 .py / .sh 里写着 <指这份库的名字>.<名字>（library_aliases）、
    …function_units().<名字> 或 from lib_function_units import … <名字>。别的模块上的同名属性（trait_impl_lag.selftest_cells 这类）不算。
    弄坏开关 selftest-fixture-names-receivers-ignored 下照旧不看 . 前面是谁：写着 .<名字> 或 import … <名字> 就算。"""
    found = set()
    loose = "selftest-fixture-names-receivers-ignored" in active_breaks()
    for top in DEFAULT_SCRIPT_DIRECTORIES + ("task-server", "prune-server"):
        for directory, subdirectories, files in os.walk(os.path.join(root, top)):
            subdirectories[:] = sorted(name for name in subdirectories if name not in SKIPPED_DIRECTORIES and name != "fixtures")
            for name in sorted(files):
                if not name.endswith((".py", ".sh")) or name.startswith("lib_function_units"):
                    continue
                with open(os.path.join(directory, name), encoding="utf-8", errors="replace") as handle:
                    text = handle.read()
                if "lib_function_units" not in text:
                    continue
                if loose:
                    found |= {called for called in names if re.search(rf"\.{re.escape(called)}\b|import[^\n]*\b{re.escape(called)}\b", text)}
                    continue
                receivers = [re.escape(alias) for alias in sorted(library_aliases(text))] + [r"function_units\(\)"]
                pattern = re.compile(rf"\b(?:{'|'.join(receivers)})\.(\w+)")
                found |= {called for called in pattern.findall(text) if called in names}
                for group in LIBRARY_FROM_IMPORT.findall(text):
                    found |= {called for called in re.findall(r"\w+", group.split("#")[0]) if called in names}
    return found


def selftest_fixture_name_problems(library_text, fixtures_text, external_calls):
    """fixtures_text 定义的名字被 library_text 运行期那一边读到的几处 → 说明的列表（空是对的）。运行期那一边：import 时执行的语句、
    从 selftest 经名字够不到的顶层函数与类、external_calls 里的函数，经函数体、类体与模块级常量的值里读的名字往下够（selftest 那一条边不走）；
    library_text 自己也定义了的同样点名。"""
    library, fixtures = ast.parse(library_text), ast.parse(fixtures_text)

    def read(node):
        return {inner.id for inner in ast.walk(node) if isinstance(inner, ast.Name) and isinstance(inner.ctx, (ast.Load, ast.Del))}
    # 模块级常量（两份里的）→ 它的值读到的名字；那一份定义的名字
    moved, values = set(), {}
    for tree in (library, fixtures):
        for statement in tree.body:
            if isinstance(statement, (ast.Assign, ast.AnnAssign)) and statement.value is not None:
                for target in (statement.targets if isinstance(statement, ast.Assign) else [statement.target]):
                    for node in ast.walk(target):
                        if isinstance(node, ast.Name):
                            values.setdefault(node.id, set()).update(read(statement.value))
                            if tree is fixtures:
                                moved.add(node.id)
    bodies, import_time, defined = {}, set(), set()

    def executed_at_import(definition):
        """def / class 在 import 时求值的那几块：装饰器、缺省值；类另有基类、关键字与类体里不是 def 的语句（类体里 def 的装饰器与缺省值）。"""
        if isinstance(definition, ast.ClassDef):
            parts = list(definition.decorator_list) + list(definition.bases) + [keyword.value for keyword in definition.keywords]
            for inner in definition.body:
                parts += executed_at_import(inner) if isinstance(inner, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)) else [inner]
            return parts
        return list(definition.decorator_list) + list(definition.args.defaults) + [value for value in definition.args.kw_defaults if value is not None]
    for statement in library.body:
        if isinstance(statement, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            bodies[statement.name] = read(statement)
            defined.add(statement.name)
            for part in executed_at_import(statement):
                import_time |= read(part)
        else:
            import_time |= read(statement)
            defined |= {inner.id for inner in ast.walk(statement) if isinstance(inner, ast.Name) and isinstance(inner.ctx, ast.Store)}
            defined |= {(alias.asname or alias.name).split(".")[0] for inner in ast.walk(statement) if isinstance(inner, (ast.Import, ast.ImportFrom))
                        for alias in inner.names}

    def reach(roots):
        seen, pending = set(), list(roots)
        while pending:
            name = pending.pop()
            if name not in seen and name != "selftest":
                seen.add(name)
                pending += list(bodies.get(name, ())) + list(values.get(name, ()))
        return seen
    selftest_side = reach(bodies.get("selftest", ()))
    runtime = reach((set(bodies) - selftest_side - {"selftest"}) | import_time | set(external_calls))
    problems = [f"import 时执行的语句读 {name}" for name in sorted(import_time & moved)]
    problems += [f"运行期的 {name} 读 {sorted(bodies[name] & moved)}" for name in sorted(runtime) if name in bodies and bodies[name] & moved]
    problems += [f"这份库自己也定义 {name}" for name in sorted(defined & moved)]
    return problems


def selftest_fixture_names_cell(_work):
    """SELFTEST_FIXTURE_NAMES_CELL：读本仓这份库与 lib_function_units_selftest_fixtures.py 的原文（selftest_fixture_name_problems），别的脚本点名的函数现扫
    （library_external_calls）。弄坏开关 selftest-fixtures-runtime-name-moved：在读进来的原文上把 ROW_KEYED_TABLES（admission.py 调的 row_keyed_closure 读它）
    从这份库挪进那一份，这一格必红。"""
    gate_directory = os.path.join(LIBRARY_ROOT, "scripts", "gate")
    library_texts, fixtures_texts = [], []
    for name in sorted(os.listdir(gate_directory)):
        if name.startswith("lib_function_units") and name.endswith(".py"):
            with open(os.path.join(gate_directory, name), encoding="utf-8") as handle:
                (fixtures_texts if name.startswith("lib_function_units_selftest_fixtures") else library_texts).append(handle.read())
    library_text, fixtures_text = "\n".join(library_texts), "\n".join(fixtures_texts)
    if "selftest-fixtures-runtime-name-moved" in active_breaks():
        statement = next(node for node in ast.parse(library_text).body if isinstance(node, ast.Assign)
                         and any(isinstance(target, ast.Name) and target.id == "ROW_KEYED_TABLES" for target in node.targets))
        rows = library_text.split("\n")
        moved = "\n".join(rows[statement.lineno - 1:statement.end_lineno])
        library_text = "\n".join(rows[:statement.lineno - 1] + rows[statement.end_lineno:])
        fixtures_text += "\n" + moved + "\n"
    names = {node.name for node in ast.parse(library_text).body if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef))}
    problems = selftest_fixture_name_problems(library_text, fixtures_text, library_external_calls(LIBRARY_ROOT, names))
    return not problems, f"{len(problems)} 处不对：{problems[:8]}（运行期也读的常量留在这份库里，文件头「自证夹具」）"
