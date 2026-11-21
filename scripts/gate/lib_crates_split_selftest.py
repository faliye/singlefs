"""crates-split-pure-move.py 的自证：在临时目录里现造小树（一个工作区、几个小包），不起 cargo、不读真 crates；各格互不依赖、并行跑。
弄坏开关打开时只跑各格（不再起开关轮），红的格集合与 common.BREAK_CELLS 点名的逐格相同；不带开关时各格跑完再起开关轮，逐个设上开关起 --selftest，
子进程用 --cell-workers 1 跑各格（开关轮外层并行 N 路、每路只一路线程，总并发不超过 N）。"""
import concurrent.futures
import os
import re
import subprocess
import sys
import tempfile

import lib_crates_split_cli as cli
import lib_crates_split_common as common
import lib_crates_split_layout as rustfmt_layout
import lib_crates_split_renames as test_renames
import lib_crates_split_tables as tables
import lib_crates_split_units as units
import lib_function_units as function_units

ROOT_MANIFEST = '[workspace]\nmembers = ["crates/*"]\n'
LEDGER = "crates/demo/src/ledger.rs"
LEDGER_ENTRY = "crates/demo/src/ledger/entry.rs"
LEDGER_TESTS = "crates/demo/src/ledger/tests.rs"
OTHER = "crates/demo/src/other.rs"
KERNEL = "crates/demo/src/kernel.wgsl"
REGISTRY = "crates/demo/src/ledger.rs"
ENTRY_SCRIPT = os.path.join(os.path.dirname(os.path.realpath(__file__)), "crates-split-pure-move.py")   # 自证起的入口：与这份模块同目录
BREAK_VARIABLE = [""]   # 弄坏开关的环境变量名，由入口经 run_selftest 交进来
CELL_WORKERS_OPTION = "--cell-workers"   # --selftest 的参数：各格并行的路数，缺省取每台默认重活线程数

LEDGER_BEFORE = r"""use std::fmt;

macro_rules! note_total {
    () => {};
}

const LIMIT: u32 = 10;

struct Entry {
    amount: u32,
}

pub struct Ledger {
    entries: Vec<Entry>,
}

impl Ledger {
    pub fn new() -> Self {
        Ledger { entries: Vec::new() }
    }

    fn total(&self) -> u32 {
        self.entries.iter().map(|entry| entry.amount).sum()
    }
}

impl fmt::Debug for Ledger {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ledger {}", self.total())
    }
}

fn limit() -> u32 {
    LIMIT
}

note_total!();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_ledger_totals_zero() {
        assert_eq!(Ledger::new().total(), 0);
    }
}
"""

LEDGER_AFTER_ROOT = r"""use self::entry::Entry;
use std::fmt;

mod entry;
#[cfg(test)]
mod tests;

macro_rules! note_total {
    () => {};
}

const LIMIT: u32 = 10;

pub struct Ledger {
    entries: Vec<Entry>,
}

impl Ledger {
    pub fn new() -> Self {
        Ledger { entries: Vec::new() }
    }

    pub(super) fn total(&self) -> u32 {
        self.entries.iter().map(|entry| entry.amount).sum()
    }
}

impl fmt::Debug for Ledger {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ledger {}", self.total())
    }
}

fn limit() -> u32 {
    LIMIT
}

note_total!();
"""

LEDGER_AFTER_ENTRY = "pub(super) struct Entry {\n    pub(super) amount: u32,\n}\n"
LEDGER_AFTER_TESTS = ("use super::*;\n\n#[test]\nfn empty_ledger_totals_zero() {\n    assert_eq!(Ledger::new().total(), 0);\n}\n")
OTHER_TEXT = "fn only_in_other_file() {}\n"

LEDGER_BEFORE_FILES = {LEDGER: LEDGER_BEFORE, OTHER: OTHER_TEXT}
LEDGER_AFTER_FILES = {LEDGER: LEDGER_AFTER_ROOT, LEDGER_ENTRY: LEDGER_AFTER_ENTRY, LEDGER_TESTS: LEDGER_AFTER_TESTS, OTHER: OTHER_TEXT}
SAME_NAME_TEXT = "fn same_name() -> u32 {\n    1\n}\n"
ARGUMENTS_COLUMN = "-p demo --lib -- some_case"
TEST_COLUMN = "some_case"


def write_tree(directory, files, crates=("demo",)):
    everything = {"Cargo.toml": ROOT_MANIFEST}
    for name in crates:
        everything[f"crates/{name}/Cargo.toml"] = f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2021"\n'
    everything.update(files)
    for path, text in everything.items():
        full = os.path.join(directory, path)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w", encoding="utf-8", newline="") as handle:
            handle.write(text)


def replaced(files, path, old, new):
    assert files[path].count(old) == 1, (path, old)
    return {**files, path: files[path].replace(old, new)}


def judged(work, before_files, after_files, crates=("demo",), before_crates=None, crate_filter=(), strict=False, tag="case"):
    before_root, after_root = os.path.join(work, f"before-{tag}"), os.path.join(work, f"after-{tag}")
    write_tree(before_root, before_files, before_crates or crates)
    write_tree(after_root, after_files, crates)
    return units.judge_pure_move(before_root, after_root, list(crate_filter), strict, False, 1)


def reference_units(work):
    """同一棵树自己比自己：算出这棵树里有几个单元。"""
    status, _lines, totals = judged(work, LEDGER_BEFORE_FILES, LEDGER_BEFORE_FILES, tag="same")
    return status, totals


def has_line(lines, *pieces):
    return any(all(piece in line for piece in pieces) for line in lines)


# ── 纯挪动检查 ───────────────────────────────────────────────────────────────

def cell_identical_tree_green(work):
    status, totals = reference_units(work)
    return (status == 0 and totals["units_before"] > 0 and totals["identical"] == totals["units_before"] == totals["units_after"]
            and totals["visibility"] == 0 and totals["scaffold"] == 0), f"同一棵树比自己该退 0、单元全部完全相同、可见性与脚手架都是 0，实得 {status} {totals}"


def cell_visibility_relaxed_green(work):
    status, lines, totals = judged(work, LEDGER_BEFORE_FILES, LEDGER_AFTER_FILES)
    return status == 0, f"纯挪动（挪出模块、只放宽可见性、加 mod 与 use）该退 0，实得 {status}、{totals}、{lines}"


def cell_scaffold_green(work):
    status, lines, totals = judged(work, LEDGER_BEFORE_FILES, LEDGER_AFTER_FILES)
    return status == 0 and totals["scaffold"] == 3 and has_line(lines, "脚手架 3 个"), f"新加的两个 mod 声明与一条 use 是脚手架、共 3 个且不算差异，实得 {status}、{totals}、{lines}"


def cell_inline_module_to_file_green(work):
    status, lines, totals = judged(work, LEDGER_BEFORE_FILES, LEDGER_AFTER_FILES)
    return status == 0 and totals["inline"] == 1, f"行内 mod tests 挪成 ledger/tests.rs（外属性同、里面的单元名不变）该判绿并数 1 个，实得 {status}、{totals}、{lines}"


def cell_visibility_counts(work):
    _status, reference = reference_units(work)
    status, _lines, totals = judged(work, LEDGER_BEFORE_FILES, LEDGER_AFTER_FILES)
    expected_identical = reference["units_before"] - 3
    return (status == 0 and totals["visibility"] == 2 and totals["identical"] == expected_identical,
            f"只因可见性变了的 2 个（fn total、struct Entry）要单报、完全相同的该是 {expected_identical}，实得 {totals}")


def cell_changed_body_red(work):
    limit_changed = replaced(LEDGER_AFTER_FILES, LEDGER, "fn limit() -> u32 {\n    LIMIT\n}", "fn limit() -> u32 {\n    LIMIT + 1\n}")
    status, lines, _totals = judged(work, LEDGER_BEFORE_FILES, limit_changed, tag="limit")
    moved_changed = replaced(LEDGER_AFTER_FILES, LEDGER_TESTS, "total(), 0)", "total(), 1)")
    moved_status, moved_lines, _moved_totals = judged(work, LEDGER_BEFORE_FILES, moved_changed, tag="moved")
    return (status == 1 and has_line(lines, "指纹变了", "fn limit", "crates/demo/src/ledger.rs:", "→")
            and moved_status == 1 and has_line(moved_lines, "指纹变了", "fn tests::empty_ledger_totals_zero", "crates/demo/src/ledger/tests.rs:"),
            f"改了函数体该退 1 并点名种类、名字、拆前与拆后的文件:行，实得 {status} {lines}；挪进子模块的函数体被改实得 {moved_status} {moved_lines}")


def cell_missing_unit_red(work):
    dropped = replaced(LEDGER_AFTER_FILES, LEDGER, "fn limit() -> u32 {\n    LIMIT\n}\n\n", "")
    status, lines, _totals = judged(work, LEDGER_BEFORE_FILES, dropped)
    return status == 1 and has_line(lines, "拆后没有", "fn limit"), f"拆后丢了 fn limit 该退 1 并点名，实得 {status} {lines}"


def cell_extra_unit_red(work):
    grown = {**LEDGER_AFTER_FILES, LEDGER: LEDGER_AFTER_ROOT + "\nfn extra_unit_added() {}\n"}
    status, lines, _totals = judged(work, LEDGER_BEFORE_FILES, grown)
    return status == 1 and has_line(lines, "拆后多出", "fn extra_unit_added"), f"拆后多出一个 fn 该退 1 并点名，实得 {status} {lines}"


def cell_duplicate_dropped_red(work):
    """两份文件里各有一个一样的 fn，两份文件拆后都动过（一份改了字节、一份没了），同一个键拆前两个、拆后一个：按多重集配才判得出少了一个。"""
    status, lines, _totals = judged(work, {"crates/demo/src/one.rs": SAME_NAME_TEXT, "crates/demo/src/two.rs": SAME_NAME_TEXT},
                                    {"crates/demo/src/one.rs": "// 这份文件的字节变了\n" + SAME_NAME_TEXT})
    return status == 1 and has_line(lines, "拆后没有", "fn same_name"), f"两份一样的 fn 拆后只剩一份该按多重集判红，实得 {status} {lines}"


def cell_duplicate_twin_unchanged_red(work):
    """一样的 fn 有一份留在没动过的文件里，另一份所在的文件没了：没动过的文件不进配对，少了的那一份照样判红。"""
    status, lines, _totals = judged(work, {"crates/demo/src/one.rs": SAME_NAME_TEXT, "crates/demo/src/two.rs": SAME_NAME_TEXT},
                                    {"crates/demo/src/one.rs": SAME_NAME_TEXT})
    return status == 1 and has_line(lines, "拆后没有", "fn same_name") and "two.rs" in " ".join(lines), f"孪生那份留在没动过的文件里、另一份的文件没了该退 1 并点名 two.rs，实得 {status} {lines}"


def cell_removed_use_green(work):
    removed = replaced(LEDGER_AFTER_FILES, LEDGER, "use std::fmt;\n", "")
    status, lines, totals = judged(work, LEDGER_BEFORE_FILES, removed)
    return status == 0 and totals["removed_uses"] == 1, f"拆后删掉一条 use 默认不算差异、单报 1 个，实得 {status} {totals} {lines}"


def cell_removed_use_strict_red(work):
    removed = replaced(LEDGER_AFTER_FILES, LEDGER, "use std::fmt;\n", "")
    status, lines, _totals = judged(work, LEDGER_BEFORE_FILES, removed, strict=True)
    return status == 1 and has_line(lines, "拆后没有", "use"), f"--strict-use 下删掉一条 use 该退 1，实得 {status} {lines}"


def cell_split_impl_green(work):
    before = {"crates/demo/src/impls.rs": "pub struct Meter;\n\nimpl Meter {\n    pub fn read(&self) -> u32 {\n        1\n    }\n\n    pub fn reset(&mut self) {}\n}\n"}
    after = {"crates/demo/src/impls.rs": "mod reset;\n\npub struct Meter;\n\nimpl Meter {\n    pub fn read(&self) -> u32 {\n        1\n    }\n}\n",
             "crates/demo/src/impls/reset.rs": "use super::Meter;\n\nimpl Meter {\n    pub fn reset(&mut self) {}\n}\n"}
    status, lines, totals = judged(work, before, after)
    return status == 0 and totals["scaffold"] == 3, f"一个 impl 块拆成两个文件里的两个块，多出的 impl 头、mod、use 是脚手架共 3 个，实得 {status} {totals} {lines}"


def cell_inline_module_attribute_dropped_red(work):
    dropped = replaced(LEDGER_AFTER_FILES, LEDGER, "#[cfg(test)]\nmod tests;", "mod tests;")
    status, lines, _totals = judged(work, LEDGER_BEFORE_FILES, dropped)
    return status == 1 and has_line(lines, "拆后没有", "mod_head tests"), f"挪成文件模块时丢了 #[cfg(test)] 该退 1、点名行内模块头，实得 {status} {lines}"


def cell_macro_call_moved_green(work):
    before = {"crates/demo/src/marks.rs": "macro_rules! mark {\n    () => {};\n}\n\nmark!();\n"}
    after = {"crates/demo/src/marks.rs": "// 位置变了\n// 位置变了\nmacro_rules! mark {\n    () => {};\n}\n\nmark!();\n"}
    status, lines, _totals = judged(work, before, after)
    return status == 0, f"条目级宏调用只是换了行号该判绿，实得 {status} {lines}"


def cell_crate_filter(work):
    files = {"crates/demo/src/stable.rs": "fn stable() -> u32 {\n    1\n}\n", "crates/other/src/changed.rs": "fn changed() -> u32 {\n    1\n}\n"}
    after = {**files, "crates/other/src/changed.rs": "fn changed() -> u32 {\n    2\n}\n"}
    crates = ("demo", "other")
    everything, _l, _t = judged(work, files, after, crates=crates, tag="all")
    narrowed, _lines, narrowed_totals = judged(work, files, after, crates=crates, crate_filter=("crates/demo",), tag="demo")
    other, _other_lines, _other_totals = judged(work, files, after, crates=crates, crate_filter=("crates/other",), tag="other")
    return (everything == 1 and narrowed == 0 and narrowed_totals["crates"] == 1 and other == 1,
            f"不收窄退 1、--crate crates/demo 只比 1 个包退 0、--crate crates/other 退 1，实得 {everything}、{narrowed} {narrowed_totals}、{other}")


def cell_crate_missing_after_red(work):
    files = {"crates/demo/src/stable.rs": "fn stable() -> u32 {\n    1\n}\n", "crates/gone/src/lost.rs": "fn lost() {}\n"}
    status, lines, _totals = judged(work, files, {"crates/demo/src/stable.rs": files["crates/demo/src/stable.rs"]}, crates=("demo",), before_crates=("demo", "gone"))
    return status == 1 and has_line(lines, "包只在拆前", "crates/gone"), f"拆后整个包没了该退 1 并点名，实得 {status} {lines}"


def cell_wgsl_changed_red(work):
    kernel = "fn square(x: u32) -> u32 {\n    return x * x;\n}\n\n@compute @workgroup_size(1)\nfn main() {\n    let value = square(2u);\n}\n"
    status, lines, _totals = judged(work, {KERNEL: kernel}, {KERNEL: kernel.replace("x * x", "x * x + 1u")})
    return status == 1 and has_line(lines, "指纹变了", "wgsl_fn square"), f"WGSL 函数体被改该退 1 并点名，实得 {status} {lines}"


def cell_wgsl_moved_green(work):
    kernel = "fn square(x: u32) -> u32 {\n    return x * x;\n}\n\n@compute @workgroup_size(1)\nfn main() {\n    let value = square(2u);\n}\n"
    entry = "@compute @workgroup_size(1)\nfn main() {\n    let value = square(2u);\n}\n"
    status, lines, _totals = judged(work, {KERNEL: kernel}, {KERNEL: entry, "crates/demo/src/square.wgsl": "fn square(x: u32) -> u32 {\n    return x * x;\n}\n"})
    return status == 0, f"WGSL 的函数拆到另一份 .wgsl 里原文不变该判绿，实得 {status} {lines}"


def cell_unknown_item_changed_red(work):
    status, lines, _totals = judged(work, {"crates/demo/src/odd.rs": "fn odd() {}\n"}, {"crates/demo/src/odd.rs": "fn odd() {}\n1 + 2;\n"})
    return status == 1 and has_line(lines, "拒算条目对不上"), f"拆后多出一处切不开的写法该退 1，实得 {status} {lines}"


def cell_exit_codes(work):
    red_after = replaced(LEDGER_AFTER_FILES, LEDGER, "fn limit() -> u32 {\n    LIMIT\n}", "fn limit() -> u32 {\n    LIMIT + 1\n}")
    roots = {}
    for name, files, crates in (("before", LEDGER_BEFORE_FILES, ("demo",)), ("green", LEDGER_AFTER_FILES, ("demo",)), ("red", red_after, ("demo",)), ("empty", {}, ())):
        roots[name] = os.path.join(work, name)
        write_tree(roots[name], files, crates)
    run = lambda after: cli.run_command(["pure-move", "--before", roots["before"], "--after", roots[after]], 1)[0]
    registry_root = os.path.join(work, "registry")
    write_tree(registry_root, {".claude/gate.d/pins.tsv": f"{REGISTRY}\tpin\n"})
    registry_status = cli.run_command(["registries", "--root", registry_root, "--split", REGISTRY], 1)[0]
    statuses = dict(green=run("green"), red=run("red"), empty=cli.run_command(["pure-move", "--before", roots["empty"], "--after", roots["empty"]], 1)[0], registries=registry_status)
    return statuses == dict(green=0, red=1, empty=77, registries=1), f"退出码该是 绿 0、红 1、没有包 77、登记表点名 1，实得 {statuses}"


# ── rustfmt 排版与没碰过的文件 ───────────────────────────────────────────────

FORMAT_FILE = "crates/demo/src/format.rs"
FORMAT_UNIT_NAMES = ("pair_sum", "table", "origin", "numbers", "doubled", "label")

FORMAT_BEFORE = r"""struct Point {
    x: u32,
    y: u32,
}

fn pair_sum(left: u32, right: u32) -> u32 {
    left + right
}

fn table() -> [u32; 3] {
    [1, 2, 3]
}

fn origin() -> Point {
    Point { x: 0, y: 0 }
}

fn numbers() -> HashMap<u32, String> {
    HashMap::new()
}

fn doubled(values: &[u32]) -> Vec<u32> {
    values.iter().map(|value| { value * 2 }).collect()
}

fn label(code: u32) -> u32 {
    match code {
        0 => {
            10
        }
        _ => {
            20
        }
    }
}
"""

FORMAT_AFTER = r"""struct Point {
    x: u32,
    y: u32,
}

fn pair_sum(
    left: u32,
    right: u32,
) -> u32 {
    left + right
}

fn table() -> [u32; 3] {
    [
        1,
        2,
        3,
    ]
}

fn origin() -> Point {
    Point {
        x: 0,
        y: 0,
    }
}

fn numbers() -> HashMap<
    u32,
    String,
> {
    HashMap::new()
}

fn doubled(values: &[u32]) -> Vec<u32> {
    values.iter().map(|value| value * 2).collect()
}

fn label(code: u32) -> u32 {
    match code {
        0 => 10,
        _ => 20,
    }
}
"""

COUNTS_BEFORE = "fn pair_sum(left: u32, right: u32) -> u32 {\n    left + right\n}\n\nfn solo() {}\n\nfn same() {}\n"
COUNTS_AFTER = "pub(super) fn pair_sum(\n    left: u32,\n    right: u32,\n) -> u32 {\n    left + right\n}\n\npub fn solo() {}\n\nfn same() {}\n"
REAL_BEFORE = "fn only() -> (u32,) {\n    (1,)\n}\n\nfn scaled(left: u32, right: u32) -> u32 {\n    left * right\n}\n"
REAL_AFTER = "fn only() -> (u32) {\n    (1)\n}\n\nfn scaled(\n    left: u32,\n    right: u32,\n) -> u32 {\n    left + right\n}\n"

# 词法单元一律是单个标点字符（=> 是 = 与 >），与 lib_function_units.py 的 rust_lex 同一种切法
SAME_AFTER_FORMATTING = [
    ("f(a, b)", "f(a, b,)"),
    ("[a, b]", "[a, b,]"),
    ("P { x: 1 }", "P { x: 1, }"),
    ("Vec<A, B>", "Vec<A, B,>"),
    ("x = (a, b);", "x = (a, b,);"),
    ("f(g(x),)", "f(g(x))"),
    ("struct W(u32,);", "struct W(u32);"),
    ("x = (HashMap<A, B>, c,);", "x = (HashMap<A, B>, c);"),
    ("m.map(|v| { v * 2 })", "m.map(|v| v * 2)"),
    ("m.map(|| { 1 })", "m.map(|| 1)"),
    ("m.map(|v| { (v, 1) })", "m.map(|v| (v, 1))"),
    ("fn f<T>(x: T,) -> T { x }", "fn f<T>(x: T) -> T { x }"),
    ("g::<u32>(x,)", "g::<u32>(x)"),
    ("match x { 0 => { 10 } _ => { 20 } }", "match x { 0 => 10, _ => 20, }"),
    ("match x { 0 => { 10 }, _ => 20 }", "match x { 0 => 10, _ => 20, }"),
]
DIFFERENT_AFTER_FORMATTING = [
    ("x = (a,);", "x = (a);"),
    ("fn f() -> (u32,) { (1,) }", "fn f() -> (u32) { (1) }"),
    ("let (a,) = t;", "let (a) = t;"),
    ("x = (HashMap<A, B>,);", "x = (HashMap<A, B>);"),
    ("m.map(|v| { let w = v; w })", "m.map(|v| let w = v; w)"),
    ("f(a, b)", "f(a b)"),
    ("f(a, b)", "f(a, c)"),
    ("fn f() { { x } }", "fn f() { x }"),
    ("match x { 0 => { 10 } }", "match x { 0 => 11, }"),
    ("match x { 0 => (a,) }", "match x { 0 => (a) }"),
]


def punctuation_tokens(text):
    return re.findall(r'"[^"]*"|[A-Za-z_][A-Za-z0-9_]*|\d+|\S', text)


def cell_rustfmt_normal_form(work):
    flatten = lambda text: rustfmt_layout.normalized(punctuation_tokens(text))
    not_equal = [pair for pair in SAME_AFTER_FORMATTING if flatten(pair[0]) != flatten(pair[1])]
    equal = [pair for pair in DIFFERENT_AFTER_FORMATTING if flatten(pair[0]) == flatten(pair[1])]
    return (not not_equal and not equal,
            f"尾逗号（紧挨 ) ] }} > 之前，单元素元组 (x,) 的除外）与只包单个表达式的闭包体、match 臂体花括号抹平之后相同才算只差排版，语句块的花括号与别的差不抹；"
            f"该相同却不同 {not_equal}，该不同却相同 {equal}")


def cell_rustfmt_only_green(work):
    status, lines, totals = judged(work, {FORMAT_FILE: FORMAT_BEFORE}, {FORMAT_FILE: FORMAT_AFTER})
    listed = [name for name in FORMAT_UNIT_NAMES if has_line(lines, "只差 rustfmt 排版", f"fn {name}", FORMAT_FILE)]
    return (status == 0 and totals["layout"] == len(FORMAT_UNIT_NAMES) and listed == list(FORMAT_UNIT_NAMES),
            f"尾逗号（) ] }} > 之前各一处）与闭包体、match 臂体的花括号差了 {len(FORMAT_UNIT_NAMES)} 个单元，该退 0、逐个列成「只差 rustfmt 排版」并数 {len(FORMAT_UNIT_NAMES)}，实得 {status}、{totals}、列出 {listed}、{lines}")


def cell_rustfmt_only_counts(work):
    status, lines, totals = judged(work, {FORMAT_FILE: COUNTS_BEFORE}, {FORMAT_FILE: COUNTS_AFTER})
    return (status == 0 and (totals["identical"], totals["visibility"], totals["layout"], totals["scaffold"]) == (1, 1, 1, 0)
            and has_line(lines, "完全相同 1 个", "只因可见性变了指纹 1 个", "只差 rustfmt 排版 1 个"),
            f"fn same 完全相同、fn solo 只因可见性变了、fn pair_sum 可见性与排版一起变（算只差 rustfmt 排版）：三类各 1 个并列报，实得 {status}、{totals}、{lines}")


def cell_rustfmt_real_changes_red(work):
    status, lines, totals = judged(work, {FORMAT_FILE: REAL_BEFORE}, {FORMAT_FILE: REAL_AFTER})
    return (status == 1 and totals["layout"] == 0 and has_line(lines, "指纹变了", "fn only") and has_line(lines, "指纹变了", "fn scaled"),
            f"单元素元组 (u32,) 去掉逗号、与尾逗号一起改了函数体都不是只差排版，该退 1 并点名 fn only 与 fn scaled、排版计 0，实得 {status}、{totals}、{lines}")


STEADY_ONE = "crates/demo/src/steady_one.rs"
STEADY_TWO = "crates/demo/src/steady_two.rs"
BIG = "crates/demo/src/big.rs"
BIG_FIRST = "crates/demo/src/big/first.rs"
BIG_SECOND = "crates/demo/src/big/second.rs"
STEADY_TEXTS = {STEADY_ONE: "#[cfg(test)]\nmod tests {\n    fn steady_one_check() {}\n}\n", STEADY_TWO: "#[cfg(test)]\nmod tests {\n    fn steady_two_check() {}\n}\n"}
BIG_BEFORE = "fn first_case() {}\n\nfn second_case() {}\n\n#[cfg(test)]\nmod tests {\n    fn first_check() {}\n\n    fn second_check() {}\n}\n"
BIG_FIRST_TEXT = "fn first_case() {}\n\n#[cfg(test)]\nmod tests {\n    fn first_check() {}\n}\n"
BIG_SECOND_TEXT = "fn second_case() {}\n\n#[cfg(test)]\nmod tests {\n    fn second_check() {}\n}\n"
SPLIT_WITH_STEADY_BEFORE = {BIG: BIG_BEFORE, **STEADY_TEXTS}
SPLIT_WITH_STEADY_AFTER = {BIG: "mod first;\nmod second;\n", BIG_FIRST: BIG_FIRST_TEXT, BIG_SECOND: BIG_SECOND_TEXT, **STEADY_TEXTS}


def cell_unchanged_files_silent(work):
    status, lines, _totals = judged(work, SPLIT_WITH_STEADY_BEFORE, SPLIT_WITH_STEADY_AFTER, tag="quiet")
    grown = replaced(SPLIT_WITH_STEADY_AFTER, BIG_FIRST, "fn first_check() {}", "fn first_check() { 1 }")
    red_status, red_lines_, _red_totals = judged(work, SPLIT_WITH_STEADY_BEFORE, grown, tag="loud")
    quiet = status == 0 and not has_line(lines, "steady_")
    loud = red_status == 1 and has_line(red_lines_, "指纹变了", "first_check", BIG_FIRST) and not has_line(red_lines_, "steady_")
    return (quiet and loud,
            f"拆前拆后逐字节相同的两份文件（各有一个同名同头的 mod tests）里不报任何东西：拆 big.rs 让 mod tests 头从 1 个变 2 个时该退 0 且没有一行点到它们，"
            f"改一个测试体该退 1、只点名 big/first.rs，实得 {status}、{lines}；{red_status}、{red_lines_}")


def cell_new_mod_heads_scaffold(work):
    status, lines, totals = judged(work, SPLIT_WITH_STEADY_BEFORE, SPLIT_WITH_STEADY_AFTER)
    return (status == 0 and totals["mod_heads"] == 1 and has_line(lines, "新加的 mod 头（脚手架）", f"被拆文件 {BIG} 的族里 mod 声明 2 个、行内 mod 头 1 个"),
            f"big.rs 拆成两份各带 mod tests 之后，多出的 1 个行内 mod 头与新加的 2 个 mod 声明都是脚手架、记在被拆文件 big.rs 的族里，实得 {status}、{totals}、{lines}")


def cell_module_attributes_head_added_red(work):
    status, lines, _totals = judged(work, {OTHER: "fn plain() {}\n"}, {OTHER: "#![allow(dead_code)]\n\nfn plain() {}\n"})
    return status == 1 and has_line(lines, "拆后多出", "mod_head <module attributes>"), f"拆后文件开头多出内属性是行为的变动、不是脚手架，该退 1 并点名，实得 {status} {lines}"


# ── 变异行重归 ───────────────────────────────────────────────────────────────

def table_row(name, path, anchor):
    return "\t".join([name, path, anchor, "替换文 \\n 带转义", ARGUMENTS_COLUMN, TEST_COLUMN])


MULTI_LINE_ANCHOR = "fn empty_ledger_totals_zero() {\\n    assert_eq!(Ledger::new().total(), 0);"
REMAP_ROWS = [
    table_row("zeta-anchor-stays-in-root", LEDGER, "const LIMIT: u32 = 10;"),
    table_row("moved-struct-field", LEDGER, "amount: u32,"),
    table_row("moved-test-assert", LEDGER, "assert_eq!(Ledger::new().total(), 0);"),
    table_row("multi-line-anchor", LEDGER, MULTI_LINE_ANCHOR),
    table_row("anchor-in-two-files", LEDGER, "use std::fmt;"),
    table_row("anchor-twice-in-one-file", LEDGER, "impl "),
    table_row("anchor-nowhere", LEDGER, "fn only_in_other_file() {}"),
    table_row("row-of-an-unsplit-file", OTHER, "fn only_in_other_file() {}"),
]
REMAP_TABLE = "# 样本表：每行六段\n# 样本表的另一行注释\n" + "\n".join(REMAP_ROWS) + "\n"


def remap_case(work, extra_arguments, table=REMAP_TABLE):
    """造拆前（LEDGER_BEFORE_FILES）、拆后（entry.rs 里再放一条 use std::fmt;，让那条锚点出在两份文件里）两棵树与变异表，起 remap-mutations。→ (退出码, 输出行, 改后的表文本)"""
    os.makedirs(work, exist_ok=True)
    before_root, after_root = os.path.join(work, "before"), os.path.join(work, "after")
    write_tree(before_root, LEDGER_BEFORE_FILES)
    write_tree(after_root, {**LEDGER_AFTER_FILES, LEDGER_ENTRY: "use std::fmt;\n" + LEDGER_AFTER_ENTRY})
    table_path, out_path = os.path.join(work, "mutations.tsv"), os.path.join(work, "mutations-out.tsv")
    with open(table_path, "w", encoding="utf-8", newline="") as handle:
        handle.write(table)
    status, lines = cli.run_command(["remap-mutations", "--before", before_root, "--after", after_root, "--table", table_path, "--out", out_path] + extra_arguments, 1)
    with open(out_path, encoding="utf-8", newline="") as handle:
        return status, lines, handle.read()


def rows_by_name(text):
    return {line.split("\t")[0]: line.split("\t") for line in text.split("\n") if line and not line.startswith("#")}


EXPLICIT_SPLIT = ["--split", LEDGER]


def cell_anchor_moved_into_submodule_changed(work):
    _status, _lines, text = remap_case(work, EXPLICIT_SPLIT)
    rows = rows_by_name(text)
    return (rows["moved-struct-field"][1] == LEDGER_ENTRY and rows["moved-test-assert"][1] == LEDGER_TESTS,
            f"锚点落进子模块的行文件列该改到子模块，实得 {rows['moved-struct-field'][1]}、{rows['moved-test-assert'][1]}")


def cell_multi_line_anchor_changed(work):
    _status, _lines, text = remap_case(work, EXPLICIT_SPLIT)
    return rows_by_name(text)["multi-line-anchor"][1] == LEDGER_TESTS, f"带 \\n 的多行锚点该按换行找、改到 {LEDGER_TESTS}，实得 {rows_by_name(text)['multi-line-anchor'][1]}"


def cell_anchor_in_two_files_listed(work):
    _status, lines, text = remap_case(work, EXPLICIT_SPLIT)
    return (has_line(lines, "没改 anchor-in-two-files", "几份文件里都有") and rows_by_name(text)["anchor-in-two-files"][1] == LEDGER,
            f"锚点在两份文件里都有该列出不改，实得 {lines}、文件列 {rows_by_name(text)['anchor-in-two-files'][1]}")


def cell_anchor_twice_in_one_file_listed(work):
    _status, lines, text = remap_case(work, EXPLICIT_SPLIT)
    return (has_line(lines, "没改 anchor-twice-in-one-file", "命中 2 次") and rows_by_name(text)["anchor-twice-in-one-file"][1] == LEDGER,
            f"锚点在一份文件里命中两次该列出不改，实得 {lines}")


def cell_anchor_nowhere_listed(work):
    _status, lines, _text = remap_case(work, EXPLICIT_SPLIT)
    return has_line(lines, "没改 anchor-nowhere", "0 处命中", "fn only_in_other_file"), f"锚点在族里 0 次命中该列出（带原文前 60 字），实得 {lines}"


def cell_anchor_only_in_split_family(work):
    _status, _lines, text = remap_case(work, EXPLICIT_SPLIT)
    rows = rows_by_name(text)
    return (rows["anchor-nowhere"][1] == LEDGER and rows["row-of-an-unsplit-file"][1] == OTHER,
            f"锚点只在无关文件（other.rs）里有的行不改，没点名被拆文件的行不碰，实得 {rows['anchor-nowhere'][1]}、{rows['row-of-an-unsplit-file'][1]}")


def cell_anchor_staying_in_root_unchanged(work):
    _status, lines, text = remap_case(work, EXPLICIT_SPLIT)
    return (rows_by_name(text)["zeta-anchor-stays-in-root"][1] == LEDGER and not has_line(lines, "zeta-anchor-stays-in-root")
            and has_line(lines, "仍在原文件命中一次 1 行")), f"锚点仍在根文件里的行不改、不列出、单数 1 行，实得 {lines}"


def cell_only_the_file_column_changes(work):
    status, _lines, text = remap_case(work, EXPLICIT_SPLIT)
    before_lines, after_lines = REMAP_TABLE.split("\n"), text.split("\n")
    if len(before_lines) != len(after_lines):
        return False, f"行数该不变：{len(before_lines)} 对 {len(after_lines)}"
    differing = []
    for original, rewritten in zip(before_lines, after_lines):
        if original == rewritten:
            continue
        original_fields, rewritten_fields = original.split("\t"), rewritten.split("\t")
        same_other_fields = (len(original_fields) == len(rewritten_fields) == 6
                             and all(original_fields[index] == rewritten_fields[index] for index in (0, 2, 3, 4, 5)))
        differing.append((original_fields[0], same_other_fields))
    names = sorted(name for name, _same in differing)
    return (names == ["moved-struct-field", "moved-test-assert", "multi-line-anchor"] and all(same for _name, same in differing) and text.endswith("\n"),
            f"只该有三行变、变的只有第 2 段、结尾换行还在，实得 {differing}")


def cell_row_order_kept(work):
    _status, _lines, text = remap_case(work, EXPLICIT_SPLIT)
    names = [line.split("\t")[0] for line in text.split("\n") if line]
    expected = ["# 样本表：每行六段", "# 样本表的另一行注释"] + [row.split("\t")[0] for row in REMAP_ROWS]
    return names == expected, f"行序（含注释行）该不变，实得 {names}"


def cell_remap_counts(work):
    _status, lines, _text = remap_case(work, EXPLICIT_SPLIT)
    return has_line(lines, "7 行里：改了 3 行，没改 4 行", "仍在原文件命中一次 1 行", "逐行列出 3 行"), f"点名被拆文件的 7 行里改了 3、没改 4（原文件 1、列出 3），实得 {lines}"


def cell_remap_exit_codes(work):
    status, _lines, _text = remap_case(work, EXPLICIT_SPLIT)
    good_table = "# 只有能归位的行\n" + "\n".join(REMAP_ROWS[:4]) + "\n"
    good_status, _good_lines, _good_text = remap_case(os.path.join(work, "good"), EXPLICIT_SPLIT, good_table)
    return status == 1 and good_status == 0, f"有行列出没改该退 1、行都归位该退 0，实得 {status}、{good_status}"


def cell_root_moved_to_main_detected(work):
    scenario = "#[test]\nfn scenario_runs() {\n    assert_eq!(1, 1);\n}\n"
    before_root, after_root = os.path.join(work, "before"), os.path.join(work, "after")
    write_tree(before_root, {"crates/demo/tests/scenario.rs": scenario})
    write_tree(after_root, {"crates/demo/tests/scenario/main.rs": scenario})
    table_path, out_path = os.path.join(work, "mutations.tsv"), os.path.join(work, "out.tsv")
    with open(table_path, "w", encoding="utf-8") as handle:
        handle.write(table_row("scenario-assert", "crates/demo/tests/scenario.rs", "assert_eq!(1, 1);") + "\n")
    status, lines = cli.run_command(["remap-mutations", "--before", before_root, "--after", after_root, "--table", table_path, "--out", out_path], 1)
    rewritten = open(out_path, encoding="utf-8").read().split("\t")[1] if os.path.exists(out_path) else ""
    return status == 0 and rewritten == "crates/demo/tests/scenario/main.rs", f"根挪成 tests/scenario/main.rs 该从两棵树认出来、文件列改过去，实得 {status} {lines} {rewritten!r}"


def cell_split_detected_from_trees(work):
    explicit_status, _lines, explicit_text = remap_case(os.path.join(work, "explicit"), EXPLICIT_SPLIT)
    detected_status, detected_lines, detected_text = remap_case(os.path.join(work, "detected"), [])
    return (detected_status == explicit_status == 1 and detected_text == explicit_text and LEDGER_ENTRY in detected_text,
            f"不给 --split 从两棵树认出 ledger.rs 被拆，结果与点名它一样，实得 {detected_status} {detected_lines}")


def cell_crate_root_siblings_searched(work):
    before_files = {"crates/demo/src/lib.rs": "mod ledger;\n\nconst TOTAL_CAP: u32 = 5;\n\nfn cap() -> u32 {\n    TOTAL_CAP\n}\n",
                    "crates/demo/src/ledger.rs": "const TOTAL_CAP: u32 = 5;\n"}
    after_files = {"crates/demo/src/lib.rs": "mod ledger;\nmod zones;\n\nfn cap() -> u32 {\n    zones::TOTAL_CAP\n}\n",
                   "crates/demo/src/zones.rs": "pub(super) const TOTAL_CAP: u32 = 5;\n", "crates/demo/src/ledger.rs": "const TOTAL_CAP: u32 = 5;\n"}
    before_root, after_root = os.path.join(work, "before"), os.path.join(work, "after")
    write_tree(before_root, before_files)
    write_tree(after_root, after_files)
    table_path, out_path = os.path.join(work, "mutations.tsv"), os.path.join(work, "out.tsv")
    with open(table_path, "w", encoding="utf-8") as handle:
        handle.write(table_row("cap-constant", "crates/demo/src/lib.rs", "const TOTAL_CAP: u32 = 5;") + "\n")
    status, lines = cli.run_command(["remap-mutations", "--before", before_root, "--after", after_root, "--table", table_path, "--out", out_path], 1)
    rewritten = open(out_path, encoding="utf-8").read().split("\t")[1] if os.path.exists(out_path) else ""
    return (status == 0 and rewritten == "crates/demo/src/zones.rs",
            f"crate 根拆出的同目录模块也在族里、拆前就有的 ledger.rs 不算，锚点只在 zones.rs 该改过去，实得 {status} {lines} {rewritten!r}")


# ── 测试改名表 ───────────────────────────────────────────────────────────────

SUMS = "ledger::tests::sums_two_entries"
EMPTY = "ledger::tests::empty_ledger_totals_zero"
EVERY = "ledger::tests::every_entry_is_counted"
RENAMES = [
    (EMPTY, "ledger::entry::tests::empty_ledger_totals_zero"),
    (SUMS, "ledger::entry::tests::sums_two_entries"),
    (f"{SUMS}_and_more", "ledger::totals::tests::sums_two_entries_and_more"),
    (EVERY, "ledger::totals::tests::every_entry_is_counted"),
]
NEW_NAMES = dict(RENAMES)
RENAMES_TEXT = "# 样本改名表\n" + "".join(f"{old}\t{new}\n" for old, new in RENAMES)


def test_row(name, arguments_tail, test_name):
    """变异表样本行：文件列是没被拆的 other.rs，第 5 段是 -p demo --lib -- 加 arguments_tail，第 6 段是 test_name。"""
    return "\t".join([name, OTHER, "fn only_in_other_file() {}", "替换文", f"-p demo --lib -- {arguments_tail}", test_name])


TEST_ROWS = [
    test_row("exact-whole-name", f"{SUMS}  {SUMS}_extra --exact", SUMS),
    test_row("substring-filter-kept", "sums_two_entries", SUMS),
    test_row("substring-filter-lost", "ledger::tests", EMPTY),
    test_row("whole-old-name-without-exact", SUMS, SUMS),
    test_row("prefix-filter-lost", "ledger::tests::every_", "ledger::tests::not_moved"),
    test_row("renamed-test-unselected", f"{SUMS} --exact", EMPTY),
    test_row("unknown-option", "sums_two_entries --skip other", EVERY),
    test_row("test-not-in-table", "other::tests::y --exact", "other::tests::y"),
]
TEST_ROW_FIELDS = {row.split("\t")[0]: row.split("\t") for row in TEST_ROWS}
TEST_TABLE = "# 样本表：每行六段\n" + "".join(f"{row}\n" for row in TEST_ROWS)
ARGUMENTS_INDEX, TEST_INDEX = 4, 5


def write_text(path, text):
    with open(path, "w", encoding="utf-8", newline="") as handle:
        handle.write(text)


def rename_case(work, table=TEST_TABLE, split=True, extra_arguments=()):
    """造拆前、拆后两棵树、变异表与改名表，起 remap-mutations --test-renames（split 为假时不给 --before 与 --split）。→ (退出码, 输出行, 改后的表文本；没写出来是空串)"""
    os.makedirs(work, exist_ok=True)
    before_root, after_root = os.path.join(work, "before"), os.path.join(work, "after")
    write_tree(before_root, LEDGER_BEFORE_FILES)
    write_tree(after_root, LEDGER_AFTER_FILES)
    table_path, renames_path, out_path = (os.path.join(work, name) for name in ("mutations.tsv", "test-renames.tsv", "mutations-out.tsv"))
    write_text(table_path, table)
    write_text(renames_path, RENAMES_TEXT)
    arguments = ["remap-mutations", "--after", after_root, "--table", table_path, "--out", out_path, "--test-renames", renames_path]
    if split:
        arguments += ["--before", before_root] + EXPLICIT_SPLIT
    status, lines = cli.run_command(arguments + list(extra_arguments), 1)
    if not os.path.exists(out_path):
        return status, lines, ""
    with open(out_path, encoding="utf-8", newline="") as handle:
        return status, lines, handle.read()


def table_field(text, name, index):
    row = rows_by_name(text).get(name)
    return row[index] if row and len(row) > index else ""


def cell_test_names_whole_replaced(work):
    _status, lines, text = rename_case(work)
    arguments, test_name = table_field(text, "exact-whole-name", ARGUMENTS_INDEX), table_field(text, "exact-whole-name", TEST_INDEX)
    expected = f"-p demo --lib -- {NEW_NAMES[SUMS]}  {SUMS}_extra --exact"
    return (arguments == expected and test_name == NEW_NAMES[SUMS] and rows_by_name(text).get("test-not-in-table") == TEST_ROW_FIELDS["test-not-in-table"]
            and not has_line(lines, "测试筛选 exact-whole-name"),
            f"带 --exact 的整名与第 6 段该换成新全名（两个空格照留；整名相等才换，{SUMS}_extra 不动），表里没有的名字那行不动、不列出，实得第 5 段 {arguments!r}、第 6 段 {test_name!r}、{lines}")


def cell_substring_filter_kept_untouched(work):
    _status, lines, text = rename_case(work)
    arguments = table_field(text, "substring-filter-kept", ARGUMENTS_INDEX)
    return (arguments == TEST_ROW_FIELDS["substring-filter-kept"][ARGUMENTS_INDEX] and not has_line(lines, "测试筛选 substring-filter-kept"),
            f"不带 --exact 的筛选词 sums_two_entries 拆前拆后都筛得到两条挪了位的测试，第 5 段不动、不列出，实得第 5 段 {arguments!r}、{lines}")


def cell_substring_filter_lost_listed(work):
    _status, lines, text = rename_case(work)
    listed = (has_line(lines, "测试筛选 substring-filter-lost", "筛选词 ledger::tests 拆前筛得到、拆后筛不到 4 个新全名", *NEW_NAMES.values())
              and has_line(lines, "测试筛选 whole-old-name-without-exact", f"筛选词 {SUMS} 拆前筛得到、拆后筛不到 2 个新全名", NEW_NAMES[SUMS], NEW_NAMES[f"{SUMS}_and_more"])
              and has_line(lines, "测试筛选 prefix-filter-lost", "拆后筛不到 1 个新全名", NEW_NAMES[EVERY]))
    untouched = all(table_field(text, name, ARGUMENTS_INDEX) == TEST_ROW_FIELDS[name][ARGUMENTS_INDEX]
                    for name in ("substring-filter-lost", "whole-old-name-without-exact", "prefix-filter-lost"))
    return listed and untouched, f"不带 --exact 的筛选词拆前筛得到、拆后筛不到的该列出（变异名、筛选词、筛不到的新全名），第 5 段一律不改（整名写法也不改），实得 {lines}、筛选词没动 {untouched}"


def cell_renamed_test_unselected_listed(work):
    _status, lines, _text = rename_case(work)
    return (has_line(lines, "测试筛选 renamed-test-unselected", f"第 6 段换名后是 {NEW_NAMES[EMPTY]}", "筛不到它")
            and has_line(lines, "测试筛选 substring-filter-lost", "第 6 段换名后是", "筛选词 ledger::tests 筛不到它"),
            f"第 6 段换名之后第 5 段的筛选（整名筛选词与子串筛选词各一行）筛不到它的该列出，实得 {lines}")


def cell_unknown_option_listed(work):
    _status, lines, text = rename_case(work)
    return (has_line(lines, "测试筛选 unknown-option", "不认得的选项 --skip")
            and table_field(text, "unknown-option", ARGUMENTS_INDEX) == TEST_ROW_FIELDS["unknown-option"][ARGUMENTS_INDEX],
            f"第 5 段 -- 之后有 --exact 以外的选项该列出、第 5 段不动，实得 {lines}")


def cell_test_columns_only_change(work):
    _status, _lines, text = rename_case(work)
    before_lines, after_lines = TEST_TABLE.split("\n"), text.split("\n")
    if len(before_lines) != len(after_lines):
        return False, f"行数该不变：{len(before_lines)} 对 {len(after_lines)}"
    differing = []
    for original, rewritten in zip(before_lines, after_lines):
        if original == rewritten:
            continue
        original_fields, rewritten_fields = original.split("\t"), rewritten.split("\t")
        same_other_fields = (len(original_fields) == len(rewritten_fields) == 6
                             and all(original_fields[index] == rewritten_fields[index] for index in (0, 1, 2, 3)))
        differing.append((original_fields[0], same_other_fields))
    names = [name for name, _same in differing]
    expected = ["exact-whole-name", "substring-filter-kept", "substring-filter-lost", "whole-old-name-without-exact", "renamed-test-unselected", "unknown-option"]
    return (names == expected and all(same for _name, same in differing) and text.endswith("\n"),
            f"变的只该有 {expected} 这六行、变的只有第 5、6 段、行序与结尾换行不变，实得 {differing}")


def cell_test_remap_counts(work):
    _status, lines, _text = rename_case(work)
    return (has_line(lines, f"照 {len(RENAMES)} 条测试改名，第 5、6 段改了 6 行，列出 5 行"),
            f"8 行样本里第 5 或第 6 段换了名的是 6 行、列出的是 5 行，成功句与失败句都带这两个数，实得 {lines}")


def cell_test_remap_exit_codes(work):
    status, _lines, _text = rename_case(work)
    consistent = "# 只有筛选都对的行\n" + "".join(f"{row}\n" for row in TEST_ROWS if row.split("\t")[0] in ("exact-whole-name", "substring-filter-kept", "test-not-in-table"))
    consistent_status, consistent_lines, _consistent_text = rename_case(os.path.join(work, "consistent"), table=consistent)
    return (status == 1 and consistent_status == 0 and has_line(consistent_lines, "✓ 变异行重归", "列出 0 行"),
            f"有测试筛选列出该退 1、筛选都对该退 0 并报列出 0 行，实得 {status}、{consistent_status} {consistent_lines}")


def cell_renames_without_split(work):
    row = test_row("exact-whole-name", f"{SUMS} --exact", SUMS)
    status, lines, text = rename_case(work, table=f"{row}\n", split=False)
    arguments, test_name = table_field(text, "exact-whole-name", ARGUMENTS_INDEX), table_field(text, "exact-whole-name", TEST_INDEX)
    return (status == 0 and arguments == f"-p demo --lib -- {NEW_NAMES[SUMS]} --exact" and test_name == NEW_NAMES[SUMS],
            f"给了改名表、没给 --before 与 --split 也要照表改测试名并退 0，实得 {status}、第 5 段 {arguments!r}、第 6 段 {test_name!r}、{lines}")


def cell_renames_and_file_column_together(work):
    row = "\t".join(["moved-struct-field-test", LEDGER, "amount: u32,", "替换文", f"-p demo --lib -- {SUMS} --exact", SUMS])
    status, lines, text = rename_case(work, table=f"{row}\n")
    expected = ["moved-struct-field-test", LEDGER_ENTRY, "amount: u32,", "替换文", f"-p demo --lib -- {NEW_NAMES[SUMS]} --exact", NEW_NAMES[SUMS]]
    return (status == 0 and rows_by_name(text).get("moved-struct-field-test") == expected,
            f"锚点落进子模块的行文件列、第 5 段、第 6 段该一起改、别的段不动，实得 {status}、{rows_by_name(text).get('moved-struct-field-test')}、{lines}")


def cell_renames_table_read(work):
    os.makedirs(work, exist_ok=True)
    paths = {name: os.path.join(work, f"{name}.tsv") for name in ("with-comments", "with-repeat", "conflict", "wrong-width", "empty-field", "absent")}
    write_text(paths["with-comments"], f"# 注释\n{EMPTY}\t{NEW_NAMES[EMPTY]}\n\n{SUMS}\t{NEW_NAMES[SUMS]}\n")
    write_text(paths["with-repeat"], f"{SUMS}\t{NEW_NAMES[SUMS]}\n{EVERY}\t{NEW_NAMES[EVERY]}\n")
    write_text(paths["conflict"], f"{SUMS}\tledger::elsewhere::tests::sums_two_entries\n")
    write_text(paths["wrong-width"], f"{SUMS}\n")
    write_text(paths["empty-field"], f"{SUMS}\t\n")
    union = test_renames.load_renames([paths["with-comments"], paths["with-repeat"]])
    expected_union = {EMPTY: NEW_NAMES[EMPTY], SUMS: NEW_NAMES[SUMS], EVERY: NEW_NAMES[EVERY]}
    statuses = {name: rename_case(os.path.join(work, name), extra_arguments=["--test-renames", paths[name]])[0]
                for name in ("conflict", "wrong-width", "empty-field", "absent")}
    return (union.loose == expected_union and not union.scoped and statuses == {"conflict": 2, "wrong-width": 2, "empty-field": 2, "absent": 3},
            f"几份改名表取并集（空行与注释跳过、同一对重复不算冲突）；同一个旧全名对到两个新全名、一行不是两段、有空段都退 2，读不了退 3，实得并集 {union.loose}、{statuses}")


# ── 测试改名表按测试目标分表 ─────────────────────────────────────────────────

SHARED_NAME = "ledger::tests::same_name"
SHARED_NAME_MOVED = "ledger::random::tests::same_name"
SHARED_NAME_ELSEWHERE = "ledger::elsewhere::tests::same_name"
SINGLE_NAME = "ledger::tests::only_here"
SINGLE_NAME_MOVED = "ledger::moved::tests::only_here"
UNLISTED_NAME = "ledger::tests::not_in_any_table"
LIBRARY_HEAD = "-p demo --lib"
INTEGRATION_HEAD = "-p demo --test random_histories"
INTEGRATION_TARGET = "demo:test:random_histories"
BINARY_HEAD = "-p demo --bin tool"
OTHER_PACKAGE_HEAD = "-p other --test random_histories"


def target_row(name, head, test_name):
    """变异表样本行：第 5 段是 head（cargo 参数）加 -- 与 --exact 点名的 test_name，第 6 段也是 test_name；文件列是没被拆的 other.rs。"""
    return "\t".join([name, OTHER, "fn only_in_other_file() {}", "替换文", f"{head} -- {test_name} --exact", test_name])


def targeted_case(work, rows, tables):
    """造拆前、拆后两棵树、变异表（rows 是样本行）与几份改名表（tables：[(文件名, 目标写法或 None, 表文本)]，目标写法是 <包>:<目标>），
    起 remap-mutations --test-renames。→ (退出码, 输出行, 改后的表文本；没写出来是空串)"""
    os.makedirs(work, exist_ok=True)
    before_root, after_root = os.path.join(work, "before"), os.path.join(work, "after")
    write_tree(before_root, LEDGER_BEFORE_FILES)
    write_tree(after_root, LEDGER_AFTER_FILES)
    table_path, out_path = os.path.join(work, "mutations.tsv"), os.path.join(work, "mutations-out.tsv")
    write_text(table_path, "".join(f"{row}\n" for row in rows))
    arguments = ["remap-mutations", "--before", before_root, "--after", after_root, "--split", LEDGER, "--table", table_path, "--out", out_path]
    for file_name, target, text in tables:
        path = os.path.join(work, file_name)
        write_text(path, text)
        arguments += ["--test-renames", path if target is None else f"{target}={path}"]
    status, lines = cli.run_command(arguments, 1)
    if not os.path.exists(out_path):
        return status, lines, ""
    with open(out_path, encoding="utf-8", newline="") as handle:
        return status, lines, handle.read()


def cell_target_renames_table_read(work):
    os.makedirs(work, exist_ok=True)
    tables = {"library.tsv": f"{SHARED_NAME}\t{SHARED_NAME_MOVED}\n", "integration.tsv": f"# 注释\n{SHARED_NAME}\t{SHARED_NAME_ELSEWHERE}\n",
              "conflict.tsv": f"{SHARED_NAME}\tledger::conflict::tests::same_name\n", "plain:name=table.tsv": f"{SINGLE_NAME}\t{SINGLE_NAME_MOVED}\n"}
    paths = {name: os.path.join(work, name) for name in tables}
    for name, text in tables.items():
        write_text(paths[name], text)

    def refused_as_usage_error(specs):
        try:
            test_renames.load_renames(specs)
        except common.UsageProblem as problem:
            return bool(problem.howto)
        except OSError:
            return False
        return False

    table = test_renames.load_renames([f"demo:lib={paths['library.tsv']}", f"{INTEGRATION_TARGET}={paths['integration.tsv']}",
                                       f"demo:bin:tool={paths['library.tsv']}", paths["plain:name=table.tsv"]])
    expected = {("demo", "lib"): {SHARED_NAME: SHARED_NAME_MOVED}, ("demo", "test:random_histories"): {SHARED_NAME: SHARED_NAME_ELSEWHERE},
                ("demo", "bin:tool"): {SHARED_NAME: SHARED_NAME_MOVED}}
    same_target_conflict = refused_as_usage_error([f"demo:lib={paths['library.tsv']}", f"demo:lib={paths['conflict.tsv']}"])
    malformed = {spec: refused_as_usage_error([spec]) for spec in (f"demo:tests={paths['library.tsv']}", f"demo:test:={paths['library.tsv']}", f"demo:test={paths['library.tsv']}",
                                                                    f":lib={paths['library.tsv']}", "demo:lib=")}
    try:
        test_renames.load_renames([f"demo:lib={os.path.join(work, 'absent.tsv')}"])
        absent_is_unreadable = False
    except OSError:
        absent_is_unreadable = True
    return (table.scoped == expected and table.loose == {SINGLE_NAME: SINGLE_NAME_MOVED} and len(table) == 4 and same_target_conflict
            and all(malformed.values()) and absent_is_unreadable,
            f"<包>:<目标>=<表> 按包与目标分表（不同目标里同一个旧全名可以对到不同的新全名），带 / 的路径照不带目标读；同一个包与目标里同一个旧全名对到两个新全名、"
            f"包或目标写不对退 2，表读不了照常抛 OSError，实得 {table.scoped}、{table.loose}、同目标冲突拒 {same_target_conflict}、写不对 {malformed}、读不了 {absent_is_unreadable}")


def cell_target_renames_by_target(work):
    originals = [target_row("lib-case", LIBRARY_HEAD, SHARED_NAME), target_row("integration-case", INTEGRATION_HEAD, SHARED_NAME),
                 target_row("binary-case", BINARY_HEAD, SHARED_NAME), target_row("other-package-case", OTHER_PACKAGE_HEAD, SHARED_NAME)]
    status, lines, text = targeted_case(work, originals, [("integration.tsv", INTEGRATION_TARGET, f"{SHARED_NAME}\t{SHARED_NAME_MOVED}\n")])
    expected = {row.split("\t")[0]: row.split("\t") for row in originals}
    expected["integration-case"] = target_row("integration-case", INTEGRATION_HEAD, SHARED_NAME_MOVED).split("\t")
    return (status == 0 and rows_by_name(text) == expected and has_line(lines, "第 5、6 段改了 1 行，列出 0 行"),
            f"同名测试在几个目标里时，{INTEGRATION_TARGET} 的改名表只换第 5 段点到 --test random_histories 的那一行（库、二进制、别的包的 random_histories 里同名的不动、不列出），"
            f"实得 {status}、{rows_by_name(text)}、{lines}")


def cell_untargeted_renames_ambiguous_listed(work):
    originals = [target_row("lib-case", LIBRARY_HEAD, SHARED_NAME), target_row("integration-case", INTEGRATION_HEAD, SHARED_NAME),
                 target_row("single-target-case", LIBRARY_HEAD, SINGLE_NAME)]
    untargeted = f"{SHARED_NAME}\t{SHARED_NAME_MOVED}\n{SINGLE_NAME}\t{SINGLE_NAME_MOVED}\n"
    status, lines, text = targeted_case(work, originals, [("untargeted.tsv", None, untargeted)])
    expected = {row.split("\t")[0]: row.split("\t") for row in originals}
    expected["single-target-case"] = target_row("single-target-case", LIBRARY_HEAD, SINGLE_NAME_MOVED).split("\t")
    listed = all(has_line(lines, f"测试筛选 {name}", f"旧全名 {SHARED_NAME}", "落在 2 个测试目标", "没换") for name in ("lib-case", "integration-case"))
    return (status == 1 and rows_by_name(text) == expected and listed and not has_line(lines, "测试筛选 single-target-case"),
            f"不带目标的改名表遇到同一个全名落在两个目标里时，点名它的两行都列出不改，只落在一个目标里的照名字换，实得 {status}、{rows_by_name(text)}、{lines}")


def cell_unrecognized_target_listed(work):
    originals = [target_row("no-package", "--test random_histories", SHARED_NAME), target_row("several-targets", f"{LIBRARY_HEAD} --test random_histories", SHARED_NAME),
                 target_row("no-target", "-p demo", SHARED_NAME), target_row("name-not-in-table", "-p demo", UNLISTED_NAME),
                 target_row("recognized", INTEGRATION_HEAD, SHARED_NAME)]
    status, lines, text = targeted_case(work, originals, [("integration.tsv", INTEGRATION_TARGET, f"{SHARED_NAME}\t{SHARED_NAME_MOVED}\n")])
    expected = {row.split("\t")[0]: row.split("\t") for row in originals}
    expected["recognized"] = target_row("recognized", INTEGRATION_HEAD, SHARED_NAME_MOVED).split("\t")
    listed = all(has_line(lines, f"测试筛选 {name}", "目标认不出", SHARED_NAME) for name in ("no-package", "several-targets", "no-target"))
    return (status == 1 and rows_by_name(text) == expected and listed and not has_line(lines, "测试筛选 name-not-in-table") and not has_line(lines, "测试筛选 recognized"),
            f"第 5 段认不出目标（没写 -p、写了两个目标、没写目标）而点名了按目标写的旧全名的行列出不改，没点名表里名字的行不列，认得出的行照换，实得 {status}、{rows_by_name(text)}、{lines}")


# ── 登记表扫描 ───────────────────────────────────────────────────────────────

REGISTRY_FILES = {
    ".claude/gate.d/pins.tsv": (f"{REGISTRY}\tpin\n# 注释里点名 {REGISTRY} 也算\n{REGISTRY}.bak\tbackup\nx{REGISTRY}\tprefixed\nnamed\t{REGISTRY}::limit\n{REGISTRY}x\tsuffixed\n"),
    ".claude/gate.d/fixtures/sample/inner.tsv": f"{REGISTRY}\tsample\n",
    ".claude/gate.d/notes.txt": f"{REGISTRY} 写在文本里\n",
}


def registry_hits(work):
    root = os.path.join(work, "root")
    write_tree(root, REGISTRY_FILES)
    status, lines = cli.run_command(["registries", "--root", root, "--split", REGISTRY], 1)
    return status, [line for line in lines if line.startswith("  ✗ 登记表点名被拆文件：")]


def cell_registry_path_boundary(work):
    status, hits = registry_hits(work)
    numbers = sorted(int(re.search(r"pins\.tsv:(\d+)", hit).group(1)) for hit in hits if "pins.tsv:" in hit)
    return status == 1 and numbers == [1, 2, 5], f"只有整条路径（前后不挨着路径字符）算点名：pins.tsv 的 1、2、5 行，实得 {numbers}"


def cell_registry_fixtures_excluded(work):
    _status, hits = registry_hits(work)
    return bool(hits) and not has_line(hits, "fixtures/"), f"fixtures/ 下的样本档不扫，实得 {hits}"


def cell_registry_only_tsv(work):
    _status, hits = registry_hits(work)
    return bool(hits) and not has_line(hits, "notes.txt"), f"只扫顶层 *.tsv，.txt 不扫，实得 {hits}"


REGISTRY_TEST_NAME_PINS = (f"{SUMS}\tpin\n# 注释里点名 {EMPTY} 也算\n{SUMS}_extra\tlonger\nx{SUMS}\tprefixed\nother::{SUMS}\tnested\n"
                           f"{NEW_NAMES[SUMS]}\tnew\nlib\t-p demo --lib -- {EVERY} --exact\n")


def cell_registry_test_names(work):
    root = os.path.join(work, "root")
    pins = ".claude/gate.d/pins.tsv"
    write_tree(root, {pins: REGISTRY_TEST_NAME_PINS})
    renames_path = os.path.join(work, "test-renames.tsv")
    write_text(renames_path, RENAMES_TEXT)
    status, lines = cli.run_command(["registries", "--root", root, "--test-renames", renames_path], 1)
    hits = [line for line in lines if line.startswith("  ✗ 登记表点名测试旧全名：")]
    numbers = sorted(int(re.search(r"pins\.tsv:(\d+)", hit).group(1)) for hit in hits)
    with open(os.path.join(root, pins), encoding="utf-8", newline="") as handle:
        unchanged = handle.read() == REGISTRY_TEST_NAME_PINS
    return (status == 1 and numbers == [1, 2, 7] and has_line(hits, "pins.tsv:1", f"{SUMS} 改名为 {NEW_NAMES[SUMS]}") and unchanged,
            f"旧全名整串出现、前后不紧挨标识符字符或 :: 的行才算点名：pins.tsv 的 1、2、7 行，列出旧全名改成什么、只报不改、退 1，实得 {status}、{numbers}、改动 {not unchanged}")


def cell_target_renames_registry(work):
    root = os.path.join(work, "root")
    pins = ".claude/gate.d/pins.tsv"
    text = (f"lib-line\t{LIBRARY_HEAD} -- {SHARED_NAME} --exact\n"
            f"integration-line\t{INTEGRATION_HEAD} -- {SHARED_NAME} --exact\n"
            f"bare-line\t{SHARED_NAME}\n"
            f"other-package-line\t{OTHER_PACKAGE_HEAD} -- {SHARED_NAME} --exact\n"
            f"name-not-in-table-line\t{INTEGRATION_HEAD} -- {UNLISTED_NAME} --exact\n")
    write_tree(root, {pins: text})
    renames_path = os.path.join(work, "integration.tsv")
    write_text(renames_path, f"{SHARED_NAME}\t{SHARED_NAME_MOVED}\n")
    status, lines = cli.run_command(["registries", "--root", root, "--test-renames", f"{INTEGRATION_TARGET}={renames_path}"], 1)
    hits = [line for line in lines if line.startswith("  ✗ 登记表点名测试旧全名：")]
    numbers = sorted(int(re.search(r"pins\.tsv:(\d+)", hit).group(1)) for hit in hits)
    with open(os.path.join(root, pins), encoding="utf-8", newline="") as handle:
        unchanged = handle.read() == text
    return (status == 1 and numbers == [2, 3] and has_line(hits, "pins.tsv:2", f"{SHARED_NAME} 改名为 {SHARED_NAME_MOVED}（{INTEGRATION_TARGET}）")
            and not has_line(hits, "pins.tsv:2", "目标认不出") and has_line(hits, "pins.tsv:3", "目标认不出") and unchanged,
            f"按目标写的改名表只列目标与它一致的行（pins.tsv 的 2 行）与目标认不出的行（3 行，注明目标认不出），库、别的包里同名的行不列，只报不改、退 1，实得 {status}、{numbers}、{hits}")


# ── 用法与开关 ───────────────────────────────────────────────────────────────

def cell_usage_errors(work):
    before_root, after_root = os.path.join(work, "before"), os.path.join(work, "after")
    write_tree(before_root, LEDGER_BEFORE_FILES)
    write_tree(after_root, LEDGER_AFTER_FILES)
    table_path = os.path.join(work, "mutations.tsv")
    with open(table_path, "w", encoding="utf-8") as handle:
        handle.write(REMAP_TABLE)
    attempts = {
        "no-arguments": [],
        "missing-after": ["pure-move", "--before", before_root],
        "unknown-crate": ["pure-move", "--before", before_root, "--after", after_root, "--crate", "crates/nope"],
        "out-is-the-table": ["remap-mutations", "--after", after_root, "--table", table_path, "--out", table_path, "--split", LEDGER],
        "no-way-to-find-splits": ["remap-mutations", "--after", after_root, "--table", table_path, "--out", os.path.join(work, "out.tsv")],
        "tree-missing": ["pure-move", "--before", os.path.join(work, "absent"), "--after", after_root],
    }
    statuses = {name: cli.run_command(arguments, 1)[0] for name, arguments in attempts.items()}
    return all(status == 2 for status in statuses.values()), f"用法不对都该退 2 并写下一步，实得 {statuses}"


def cell_break_outside_selftest(work):
    before_root, after_root = os.path.join(work, "before"), os.path.join(work, "after")
    write_tree(before_root, LEDGER_BEFORE_FILES)
    write_tree(after_root, LEDGER_AFTER_FILES)
    environment = dict(os.environ, **{BREAK_VARIABLE[0]: "changed-ignored"})
    completed = subprocess.run([sys.executable, ENTRY_SCRIPT, "pure-move", "--before", before_root, "--after", after_root],
                               capture_output=True, text=True, env=environment)
    return (completed.returncode == 2 and BREAK_VARIABLE[0] in completed.stderr and "自证" in completed.stderr,
            f"平常跑设了弄坏开关该退 2、stderr 点名它只在自证里认，实得 {completed.returncode}、{completed.stderr.strip()[:200]!r}")


def cell_switch_wheel_verdict(work):
    named = ("first-cell", "second-cell")
    cases = {"equal": (1, ["second-cell", "first-cell"]), "extra-red": (1, ["first-cell", "second-cell", "stray-cell"]),
             "missing-red": (1, ["first-cell"]), "none-red": (1, []), "exits-zero": (0, ["first-cell", "second-cell"])}
    verdicts = {name: wheel_verdict("sample-switch", named, status, red) for name, (status, red) in cases.items()}
    ok = {name: verdict[0] for name, verdict in verdicts.items()}
    return (ok == dict(equal=True, **{name: False for name in cases if name != "equal"})
            and "多红：stray-cell" in verdicts["extra-red"][1] and "少红" not in verdicts["extra-red"][1]
            and "少红：second-cell" in verdicts["missing-red"][1] and "多红" not in verdicts["missing-red"][1]
            and "少红：first-cell、second-cell" in verdicts["none-red"][1]
            and "退 0" in verdicts["exits-zero"][1]), f"红的格与点名的逐格相同才对，多红、少红、退 0 各自判错并点名，实得 {verdicts}"


def cell_selftest_arguments(work):
    problems = {}
    for name, rest in {"no-width": ["--cell-workers"], "zero": ["--cell-workers", "0"], "text": ["--cell-workers", "many"],
                       "unknown": ["--cells", "1"], "extra": ["--cell-workers", "1", "again"]}.items():
        try:
            problems[name] = parse_selftest_arguments(rest, "SAMPLE_BREAK")
        except common.UsageProblem as error:
            problems[name] = bool(error.howto)
    parsed = (parse_selftest_arguments([], "SAMPLE_BREAK"), parse_selftest_arguments([CELL_WORKERS_OPTION, "7"], "SAMPLE_BREAK"), parse_selftest_arguments(wheel_command()[3:], "SAMPLE_BREAK"))
    return (problems == {name: True for name in problems} and parsed == (None, 7, 1),
            f"--selftest 只认 --cell-workers <正整数>，别的抛用法错并带出路；没给是 None、开关轮起的子进程交 1，实得 {problems}、{parsed}")


CELLS = [
    ("identical-tree-green", cell_identical_tree_green),
    ("visibility-relaxed-green", cell_visibility_relaxed_green),
    ("scaffold-green", cell_scaffold_green),
    ("inline-module-to-file-green", cell_inline_module_to_file_green),
    ("visibility-counts", cell_visibility_counts),
    ("changed-body-red", cell_changed_body_red),
    ("missing-unit-red", cell_missing_unit_red),
    ("extra-unit-red", cell_extra_unit_red),
    ("duplicate-dropped-red", cell_duplicate_dropped_red),
    ("duplicate-twin-unchanged-red", cell_duplicate_twin_unchanged_red),
    ("removed-use-green", cell_removed_use_green),
    ("removed-use-strict-red", cell_removed_use_strict_red),
    ("split-impl-green", cell_split_impl_green),
    ("inline-module-attribute-dropped-red", cell_inline_module_attribute_dropped_red),
    ("macro-call-moved-green", cell_macro_call_moved_green),
    ("crate-filter", cell_crate_filter),
    ("crate-missing-after-red", cell_crate_missing_after_red),
    ("wgsl-changed-red", cell_wgsl_changed_red),
    ("wgsl-moved-green", cell_wgsl_moved_green),
    ("unknown-item-changed-red", cell_unknown_item_changed_red),
    ("exit-codes", cell_exit_codes),
    ("rustfmt-normal-form", cell_rustfmt_normal_form),
    ("rustfmt-only-green", cell_rustfmt_only_green),
    ("rustfmt-only-counts", cell_rustfmt_only_counts),
    ("rustfmt-real-changes-red", cell_rustfmt_real_changes_red),
    ("unchanged-files-silent", cell_unchanged_files_silent),
    ("new-mod-heads-scaffold", cell_new_mod_heads_scaffold),
    ("module-attributes-head-added-red", cell_module_attributes_head_added_red),
    ("anchor-moved-into-submodule-changed", cell_anchor_moved_into_submodule_changed),
    ("multi-line-anchor-changed", cell_multi_line_anchor_changed),
    ("anchor-in-two-files-listed", cell_anchor_in_two_files_listed),
    ("anchor-twice-in-one-file-listed", cell_anchor_twice_in_one_file_listed),
    ("anchor-nowhere-listed", cell_anchor_nowhere_listed),
    ("anchor-only-in-split-family", cell_anchor_only_in_split_family),
    ("anchor-staying-in-root-unchanged", cell_anchor_staying_in_root_unchanged),
    ("only-the-file-column-changes", cell_only_the_file_column_changes),
    ("row-order-kept", cell_row_order_kept),
    ("remap-counts", cell_remap_counts),
    ("remap-exit-codes", cell_remap_exit_codes),
    ("root-moved-to-main-detected", cell_root_moved_to_main_detected),
    ("split-detected-from-trees", cell_split_detected_from_trees),
    ("crate-root-siblings-searched", cell_crate_root_siblings_searched),
    ("test-names-whole-replaced", cell_test_names_whole_replaced),
    ("substring-filter-kept-untouched", cell_substring_filter_kept_untouched),
    ("substring-filter-lost-listed", cell_substring_filter_lost_listed),
    ("renamed-test-unselected-listed", cell_renamed_test_unselected_listed),
    ("unknown-option-listed", cell_unknown_option_listed),
    ("test-columns-only-change", cell_test_columns_only_change),
    ("test-remap-counts", cell_test_remap_counts),
    ("test-remap-exit-codes", cell_test_remap_exit_codes),
    ("renames-without-split", cell_renames_without_split),
    ("renames-and-file-column-together", cell_renames_and_file_column_together),
    ("renames-table-read", cell_renames_table_read),
    ("target-renames-table-read", cell_target_renames_table_read),
    ("target-renames-by-target", cell_target_renames_by_target),
    ("untargeted-renames-ambiguous-listed", cell_untargeted_renames_ambiguous_listed),
    ("unrecognized-target-listed", cell_unrecognized_target_listed),
    ("registry-path-boundary", cell_registry_path_boundary),
    ("registry-fixtures-excluded", cell_registry_fixtures_excluded),
    ("registry-only-tsv", cell_registry_only_tsv),
    ("registry-test-names", cell_registry_test_names),
    ("target-renames-registry", cell_target_renames_registry),
    ("usage-errors", cell_usage_errors),
    ("break-outside-selftest", cell_break_outside_selftest),
    ("switch-wheel-verdict", cell_switch_wheel_verdict),
    ("selftest-arguments", cell_selftest_arguments),
]


def run_cells(workers):
    """各格并行跑，每格一个自己的临时目录；→ [(格名, 过没过, 说明)]，按 CELLS 的次序。"""
    with tempfile.TemporaryDirectory() as parent:
        def run_one(entry):
            name, function = entry
            directory = os.path.join(parent, name)
            os.makedirs(directory)
            try:
                passed, message = function(directory)
            except Exception as error:  # noqa: BLE001 一格抛了就是那一格红，别的格照跑
                passed, message = False, f"抛了 {type(error).__name__}：{error}"
            return name, passed, message
        with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, min(workers, len(CELLS)))) as pool:
            return list(pool.map(run_one, CELLS))


def parse_selftest_arguments(rest, break_variable):
    """--selftest 后面的参数：只认 --cell-workers <正整数>（各格并行的路数）；→ 那个数，没给是 None。不对抛 common.UsageProblem。"""
    if not rest:
        return None
    if len(rest) == 2 and rest[0] == CELL_WORKERS_OPTION and re.fullmatch(r"[1-9][0-9]*", rest[1]):
        return int(rest[1])
    raise common.UsageProblem(f"--selftest 只认 {CELL_WORKERS_OPTION} <正整数>：{' '.join(rest)}",
                              f"python3 scripts/gate/crates-split-pure-move.py --selftest [{CELL_WORKERS_OPTION} <路数>]；要单跑一个开关，在命令前设 {break_variable}=<开关>")


def wheel_command():
    """开关轮给每个开关起的子进程：各格只用 1 路线程，开关轮外层并行几路总并发就是几路。"""
    return [sys.executable, ENTRY_SCRIPT, "--selftest", CELL_WORKERS_OPTION, "1"]


def wheel_verdict(switch, named, status, red_cells):
    """一个开关打开后的结局要退 1、红的格集合与点名的逐格相同；→ (对不对, 输出行)，不对的行点名多红与少红的格。"""
    named_set, red_set = set(named), set(red_cells)
    shown = f"红在「{'、'.join(red_cells) or '（没有）'}」"
    if status == 1 and red_set == named_set:
        return True, f"  · 开关轮：{switch} {shown}，与点名的逐格相同"
    problems = [f"退 {status}（该退 1）"] if status != 1 else []
    problems += [f"多红：{'、'.join(sorted(red_set - named_set))}"] if red_set - named_set else []
    problems += [f"少红：{'、'.join(sorted(named_set - red_set))}"] if named_set - red_set else []
    return False, f"  ✗ 开关轮：{switch} 打开后{shown}，点名的是「{'、'.join(named)}」；{'；'.join(problems)}"  # gate-lint:detail


def switch_wheel(workers):
    """逐个打开 common.BREAK_CELLS 的开关起 --selftest（外层并行 workers 路、子进程各格只 1 路），各自要退 1、红的格集合与点名的逐格相同；→ (没对上的个数, 输出行)。"""
    def run_switch(item):
        switch, named = item
        completed = subprocess.run(wheel_command(), capture_output=True, text=True, env=dict(os.environ, **{BREAK_VARIABLE[0]: switch}))
        return switch, named, completed.returncode, re.findall(r"✗ 自证 ([a-z0-9-]+)：", completed.stdout)
    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, min(workers, len(common.BREAK_CELLS)))) as pool:
        outcomes = list(pool.map(run_switch, list(common.BREAK_CELLS.items())))
    if len(outcomes) != len(common.BREAK_CELLS):
        return 1, [f"  ✗ 开关轮：收回 {len(outcomes)} 个结果，该有 {len(common.BREAK_CELLS)} 个"]
    verdicts = [wheel_verdict(switch, named, status, red_cells) for switch, named, status, red_cells in outcomes]
    return sum(1 for ok, _line in verdicts if not ok), [line for _ok, line in verdicts]


def run_selftest(workers, break_variable, cell_workers=None):
    """--selftest：弄坏开关没打开时，各格跑完再起开关轮；打开了一个或几个就只跑各格（红的格集合与点名的逐格相同是对的）。
    workers 是每台默认重活线程数：各格并行的路数缺省取它（cell_workers 给了就用它），开关轮外层并行度永远取它。break_variable 是入口读开关用的环境变量名。
    → (退出码, 格数, 开关轮的开关数)；成功句由调用方打"""
    BREAK_VARIABLE[0] = break_variable
    os.environ[function_units.RUST_LEX_CACHE_DIRECTORY_VARIABLE] = "off"
    names = {name for name, _function in CELLS}
    missing = sorted({cell for named in common.BREAK_CELLS.values() for cell in named} - names)
    if missing:
        print(f"  ✗ 自证 switch-table：开关表点名的格不存在：{'、'.join(missing)}")  # gate-lint:detail
        return 1, len(CELLS), 0
    results = run_cells(cell_workers or workers)
    if len(results) != len(CELLS):
        print(f"  ✗ --selftest：收回 {len(results)} 格的结果，该有 {len(CELLS)} 格")
        print("     → 怎么办：这一趟的结果不全，不当通过；整份重跑一次")
        return 1, len(CELLS), 0
    failed = [result for result in results if not result[1]]
    for name, _passed, message in failed:
        print(f"  ✗ 自证 {name}：{message}")  # gate-lint:detail
    if failed:
        print(f"  ✗ --selftest：{len(results)} 格里 {len(failed)} 格不对")
        print("     → 怎么办：照上面点名的格查判法；弄坏开关打开时红在文件头点名的格是对的")
        return 1, len(CELLS), 0
    if common.broken:
        print(f"  ✗ --selftest：弄坏开关 {'、'.join(sorted(common.broken))} 打开了，{len(results)} 格却一格都没红，开关没生效")
        print("     → 怎么办：照文件头点名的格查那个开关弄坏的那一处有没有真的接进判法")
        return 1, len(CELLS), 0
    failures, lines = switch_wheel(workers)
    for line in lines:
        print(line)  # gate-lint:detail
    if failures:
        print(f"  ✗ 开关轮 {len(common.BREAK_CELLS)} 个开关里 {failures} 个红的格与点名的不是逐格相同")
        print("     → 怎么办：改那一处判法，或照实改 lib_crates_split_common.py 的开关表与文件头")
        return 1, len(CELLS), len(common.BREAK_CELLS)
    return 0, len(CELLS), len(common.BREAK_CELLS)
