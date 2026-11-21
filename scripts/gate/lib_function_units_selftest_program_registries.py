"""自证的登记类格：不拒算的程序词、起仓内程序、按尾巴读的登记、记录里的尾巴读、调用方给的命令、内联拒算行、own 程序与 extern 块、工作区成员。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import ast
import errno
import os
import posixpath
import re
import subprocess
import threading
import types
import lib_lag_split_family as split_family
from lib_function_units_lexer import active_breaks
from lib_function_units_base import FunctionUnitsError, INLINE_FORM, LIBRARY_ROOT, UsageError, directory_fingerprint, files_under
from lib_function_units_registry import (
    BUILT_TOOL_LAUNCHES, BUILT_TOOL_SOURCES, CALLER_SUPPLIED_COMMANDS, EXCLUDED_CALL_EDGES, EXCLUDED_PROGRAM_EDGES, LAUNCHED_REPOSITORY_PROGRAMS,
    REGISTRY_COMMAND_TEXTS, RUNTIME_DATA_FILES, SPEC_PATH_ARGUMENTS, TAIL_READ_RECORD_PREFIXES, TAIL_READ_ROOT_COPY, TAIL_READ_SCOPES, TEST_SERVICE_CODE_ROWS,
    UNKEYED_PROGRAM_WORDS, UNSCOPED_TAIL_READS, in_tail_read_scope, is_test_service_code, registered_function_name, test_service_code_rows,
)
from lib_function_units_shell_text import tree_copy_file
from lib_function_units_shell_index import ShellIndex, language_of_file, shell_files_under
from lib_function_units_python_module import PythonIndex
import lib_function_units_unit_graph as unit_graph_module
from lib_function_units_unit_graph import UNIT_GRAPH_TEXT_BUSY_PAUSE_SECONDS, run_program_waiting_out_text_busy
from lib_function_units_script_closure import Options, SHARED_SCRIPT_INDICES, closure_of, closure_unit_lines
from lib_function_units_closure_keys import edit_file, write_tree
from lib_function_units_selftest_fixture_trees import (
    SHELL_FOLDS_FIXTURE, fixture_key, judging_entry_refusal_workers, prepare_shared_script_index, registered_judging_entries,
)


def directory_closure_line_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, MANIFEST_DIRECTORY_FIXTURE)
    file = "crates/fdir/src/lib.rs"
    closure = closure_of(base, f"{file}::count_rules_directly")
    got = [line for line in closure_unit_lines(closure) if line.startswith("data-directory\t")]
    line = next(number for number, text in enumerate(MANIFEST_DIRECTORY_FIXTURE[file].splitlines(), 1) if 'join("rules")' in text)
    want = [f"data-directory\tcrates/fdir/rules\t{file}:{line}-{line}\t{directory_fingerprint(base, 'crates/fdir/rules')[0]}"]
    return got == want, f"打出的数据目录行 {got}（要 {want}）"


def program_words_cell(work):
    """PROGRAM_WORDS_CELL：不登记时四处都拒算；自证期间临时登记两处（PROGRAM_WORDS_FIXTURE_ROWS），那两处不拒算、另两处照拒算；跑完把表还原。"""
    base = os.path.join(work, "tree")
    write_tree(base, PROGRAM_WORDS_FIXTURE)
    seam, other = ("tool/seam.sh::run_seam", "tool/launch.py::launch_seam"), ("tool/seam.sh::run_other", "tool/launch.py::launch_other")

    def refused(entries):
        return [fixture_key(base, entry)[0] is None for entry in entries]

    before = refused(seam + other)
    registered = dict(UNKEYED_PROGRAM_WORDS)
    UNKEYED_PROGRAM_WORDS.update(PROGRAM_WORDS_FIXTURE_ROWS)
    try:
        after_seam, after_other = refused(seam), refused(other)
    finally:
        UNKEYED_PROGRAM_WORDS.clear()
        UNKEYED_PROGRAM_WORDS.update(registered)
    return all(before) and not any(after_seam) and all(after_other), (f"不登记时拒算 {before}（要全 True）；登记之后登记的那两处拒算 {after_seam}（要全 False）、"
                                                                      f"没登记的那两处拒算 {after_other}（要全 True）")



def launched_programs_cell(work):
    """LAUNCHED_PROGRAMS_CELL：不登记时两处都拒算；自证期间临时登记，两处都不拒算、改被起那一份里只有脚本体走到的 drive 键变；跑完把表还原。"""
    base = os.path.join(work, "tree")
    write_tree(base, LAUNCHED_PROGRAMS_FIXTURE)
    entries = ("tool/driver.py::launch_self", "tool/driver.py::launch_template")
    before = [fixture_key(base, entry)[0] is None for entry in entries]
    registered = dict(LAUNCHED_REPOSITORY_PROGRAMS)
    LAUNCHED_REPOSITORY_PROGRAMS.update(LAUNCHED_PROGRAMS_FIXTURE_ROWS)
    try:
        first = [fixture_key(base, entry)[0] for entry in entries]
        edit_file(base, "tool/driver.py", "return 7", "return 70")
        second = [fixture_key(base, entry)[0] for entry in entries]
    finally:
        LAUNCHED_REPOSITORY_PROGRAMS.clear()
        LAUNCHED_REPOSITORY_PROGRAMS.update(registered)
    changed = [one is not None and two is not None and one != two for one, two in zip(first, second)]
    return all(before) and all(changed), f"不登记时拒算 {before}（要全 True）；登记之后算得出键、改 drive 键变 {changed}（要全 True）"



def registered_inline_launch_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, REGISTERED_INLINE_LAUNCH_FIXTURE)
    before = fixture_key(base, "tool/threads.py::launch")
    after = with_program_word_rows(REGISTERED_INLINE_LAUNCH_ROWS, lambda: fixture_key(base, "tool/threads.py::launch"))
    return before[0] is None and after[0] is not None, f"不登记时拒算 {sorted(before[1])}（要拒算）；登记之后拒算 {sorted(after[1])}（要不拒算）"



def launched_shell_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, LAUNCHED_SHELL_FIXTURE)
    before = fixture_key(base, "tool/source_lib.sh")[0]
    registered = dict(LAUNCHED_REPOSITORY_PROGRAMS)
    LAUNCHED_REPOSITORY_PROGRAMS.update(LAUNCHED_SHELL_FIXTURE_ROWS)
    try:
        first = fixture_key(base, "tool/source_lib.sh")[0]
        edit_file(base, "tool/lib.sh", "echo 7", "echo 70")
        second = fixture_key(base, "tool/source_lib.sh")[0]
    finally:
        LAUNCHED_REPOSITORY_PROGRAMS.clear()
        LAUNCHED_REPOSITORY_PROGRAMS.update(registered)
    return before is None and first is not None and second is not None and first != second, (
        f"不登记时{'拒算' if before is None else '没拒算'}（要拒算）；登记之后{'算得出键' if first else '拒算'}、改 tool/lib.sh 键{'变' if first != second else '不变'}（要变）")



def refusal_lines_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, REFUSAL_LINES_FIXTURE)
    got = {file: sorted((refusal.form, refusal.line) for refusal in closure_of(base, file).refusals) for file in sorted(REFUSAL_LINES_FIXTURE)}
    wanted = {"s/command_word.sh": [("shell-command-unfolded", 4)], "s/inline_word.sh": [(INLINE_FORM, 4)]}
    return got == wanted, f"拒算落在 {got}（要 {wanted}）"


def refusal_word_lines_cell(work):
    """REFUSAL_WORD_LINES_CELL：s/word_bounds.sh 起不在的 join（前一行 bash s/tools/joined.sh 里有 join 这一截）、s/positional_caller.sh 照 bash 剪
    s/positional_interpreter.sh（"$1" join 代成 bash join，前一行 echo join）：shell-path-missing 各恰好一条，都落在第 3 行。
    弄坏开关 shell-refusal-lines-by-substring 下前一份落到第 2 行，shell-refusal-lines-positional-unmatched 下后一份落到第 2 行。"""
    base = os.path.join(work, "tree")
    write_tree(base, REFUSAL_WORD_LINES_FIXTURE)
    entries = ("s/word_bounds.sh", "s/positional_caller.sh")
    got = {entry: sorted((refusal.form, refusal.file, refusal.line) for refusal in closure_of(base, entry).refusals) for entry in entries}
    wanted = {"s/word_bounds.sh": [("shell-path-missing", "s/word_bounds.sh", 3)],
              "s/positional_caller.sh": [("shell-path-missing", "s/positional_interpreter.sh", 3)]}
    return got == wanted, f"拒算落在 {got}（要 {wanted}）"


def with_program_word_rows(rows, judge):
    """自证期间临时把 rows 并进 UNKEYED_PROGRAM_WORDS 再判，跑完还原。"""
    registered = dict(UNKEYED_PROGRAM_WORDS)
    UNKEYED_PROGRAM_WORDS.update(rows)
    try:
        return judge()
    finally:
        UNKEYED_PROGRAM_WORDS.clear()
        UNKEYED_PROGRAM_WORDS.update(registered)


def program_words_qualified_cell(work):
    """PROGRAM_WORDS_QUALIFIED_CELL：tool/launchers.py 里 TripLauncher.run 与 OtherLauncher.run 各起调用方交的折不出的命令（入口 trip、other）。
    不登记时两条都拒算；自证期间只临时登记 PROGRAM_WORDS_QUALIFIED_ROWS（TripLauncher.run 那一处）：trip 不拒算、other 照拒算（登记按限定名认）；跑完把表还原。
    弄坏开关 registered-functions-short-names 下照方法名 run 查、登的限定名认不上，trip 也拒算，这一格必红。"""
    base = os.path.join(work, "tree")
    write_tree(base, PROGRAM_WORDS_QUALIFIED_FIXTURE)

    def refused():
        return [fixture_key(base, entry)[0] is None for entry in PROGRAM_WORDS_QUALIFIED_ENTRIES]

    before = refused()
    after = with_program_word_rows(PROGRAM_WORDS_QUALIFIED_ROWS, refused)
    return before == [True, True] and after == [False, True], (f"不登记时 trip、other 拒算 {before}（要 [True, True]）；登记 TripLauncher.run 之后拒算 {after}"
                                                                "（要 [False, True]：登的那一个方法不拒算，同一份文件里别的类的同名方法照拒算）")



def nested_program_words_registered_cell(work):
    """NESTED_PROGRAM_WORDS_REGISTERED_CELL：临时登记 NESTED_PROGRAM_WORDS_INNER_ROWS，外层当入口改 tool/child.sh 前后各算一把键：都算得出、改了变。"""
    base = os.path.join(work, "tree")
    write_tree(base, NESTED_PROGRAM_WORDS_FIXTURE)

    def keys():
        before = fixture_key(base, NESTED_PROGRAM_WORDS_OUTER)
        edit_file(base, "tool/child.sh", "echo child", "echo child changed")
        return before, fixture_key(base, NESTED_PROGRAM_WORDS_OUTER)
    (before, before_refused), (after, after_refused) = with_program_word_rows(NESTED_PROGRAM_WORDS_INNER_ROWS, keys)
    ok = None not in (before, after) and before != after
    return ok, (f"按内层短名登记之后外层当入口：改前拒算 {sorted(before_refused)}、改后拒算 {sorted(after_refused)}（都要不拒算）；"
                f"改 tool/child.sh 之后键{'变了' if ok else '没变或算不出'}（要变）")


def nested_program_words_unregistered_cell(work):
    """NESTED_PROGRAM_WORDS_UNREGISTERED_CELL：不登记时内层当入口拒算 shell-command-unfolded；只按外层的短名登记（NESTED_PROGRAM_WORDS_OUTER_ROWS）时外层当入口照拒算。"""
    base = os.path.join(work, "tree")
    write_tree(base, NESTED_PROGRAM_WORDS_FIXTURE)
    inner, inner_refused = fixture_key(base, NESTED_PROGRAM_WORDS_INNER)
    outer, outer_refused = with_program_word_rows(NESTED_PROGRAM_WORDS_OUTER_ROWS, lambda: fixture_key(base, NESTED_PROGRAM_WORDS_OUTER))
    ok = inner is None and "shell-command-unfolded" in inner_refused and outer is None and "shell-command-unfolded" in outer_refused
    return ok, (f"不登记时内层当入口拒算 {sorted(inner_refused)}（要有 shell-command-unfolded）；"
                f"只按外层短名登记时外层当入口拒算 {sorted(outer_refused)}（要有 shell-command-unfolded）")


def registered_missing_path_cell(work):
    """REGISTERED_MISSING_PATH_CELL：不登记时入口拒算 shell-path-missing；临时登记 REGISTERED_MISSING_PATH_ROWS 之后不拒算，改 tool/child.sh 前后各算一把键、变了。"""
    base = os.path.join(work, "tree")
    write_tree(base, REGISTERED_MISSING_PATH_FIXTURE)
    entry = "tool/template_launch.sh::run_cells"
    unregistered, unregistered_refused = fixture_key(base, entry)

    def keys():
        before = fixture_key(base, entry)
        edit_file(base, "tool/child.sh", "echo child", "echo child changed")
        return before, fixture_key(base, entry)
    (before, before_refused), (after, after_refused) = with_program_word_rows(REGISTERED_MISSING_PATH_ROWS, keys)
    ok = unregistered is None and "shell-path-missing" in unregistered_refused and None not in (before, after) and before != after
    return ok, (f"不登记时拒算 {sorted(unregistered_refused)}（要有 shell-path-missing）；登记之后改前拒算 {sorted(before_refused)}、改后拒算 {sorted(after_refused)}"
                f"（都要不拒算），改 tool/child.sh 之后键{'变了' if before is not None and after is not None and before != after else '没变或算不出'}（要变）")


def spec_path_arguments_cell(work):
    """SPEC_PATH_ARGUMENTS_CELL：不登记时拒算 spec-path-unfolded；自证期间临时登记夹具里那一处，不拒算、改载入的那一份键变；跑完把表还原。"""
    base = os.path.join(work, "tree")
    write_tree(base, SPEC_PATH_FIXTURE)
    entry = "tool/check.py::load"
    unregistered, forms = fixture_key(base, entry)
    registered = dict(SPEC_PATH_ARGUMENTS)
    SPEC_PATH_ARGUMENTS[("tool/check.py", "anchors_path")] = "tool/anchors.py"
    try:
        before, before_refused = fixture_key(base, entry)
        edit_file(base, "tool/anchors.py", "VALUE = 1", "VALUE = 2")
        after, after_refused = fixture_key(base, entry)
        # 登记值是元组（载入的是几份里的哪一份折不出）：每一份都连，改元组里不是头一份的那一份键也变
        SPEC_PATH_ARGUMENTS[("tool/check.py", "anchors_path")] = ("tool/anchors.py", "tool/anchors_other.py")
        tuple_before, tuple_before_refused = fixture_key(base, entry)
        edit_file(base, "tool/anchors_other.py", "VALUE = 3", "VALUE = 4")
        tuple_after, tuple_after_refused = fixture_key(base, entry)
    finally:
        SPEC_PATH_ARGUMENTS.clear()
        SPEC_PATH_ARGUMENTS.update(registered)
    return (unregistered is None and "spec-path-unfolded" in forms and None not in (before, after) and before != after
            and None not in (tuple_before, tuple_after) and tuple_before != tuple_after,
            f"不登记时拒算 {sorted(forms)}（要 spec-path-unfolded）；登记之后改前 {before}（拒算 {sorted(before_refused)}）、改后 {after}（拒算 {sorted(after_refused)}），要两把都有、不相等；"
            f"登记元组之后改第二份：改前 {tuple_before}（拒算 {sorted(tuple_before_refused)}）、改后 {tuple_after}（拒算 {sorted(tuple_after_refused)}），要两把都有、不相等")


def tail_read_scopes_cell(work):
    """TAIL_READ_SCOPES_CELL：不登记时 scan 照尾巴收全仓的 Manifest.toml（前缀外多一份键变）、climb 改 elsewhere 那一份键变、
    rows 照 tree_copy_file 收根里的 tables/rows.tsv（改它键变）；自证期间临时登记 scan 那一处只留 members、climb 与 rows 那两处是调用方的输入：
    前缀外多一份、改 elsewhere、改 tables/rows.tsv 键都不变，改 members 底下那一份 scan 键照变；跑完把表还原。"""
    base = os.path.join(work, "tree")
    write_tree(base, TAIL_READ_SCOPES_FIXTURE)
    scan, climb, rows = "pkg/scoped.py::scan", "pkg/scoped.py::climb", "pkg/scoped.py::rows"
    scan_read = ("pkg/scoped.py", "member_texts", "os.path.join(directory, 'Manifest.toml')")
    climb_read = ("pkg/scoped.py", "nearest", "os.path.join(current, 'Manifest.toml')")
    rows_read = ("pkg/scoped.py", "table_rows", "os.path.join(tree, 'tables', 'rows.tsv')")
    observed = {}
    observed["scan 不登记"] = fixture_key(base, scan)[0]
    observed["climb 不登记"] = fixture_key(base, climb)[0]
    observed["rows 不登记"] = fixture_key(base, rows)[0]
    write_tree(base, {"outside/Manifest.toml": "name = outside\n"})
    observed["scan 不登记、前缀外多一份"] = fixture_key(base, scan)[0]
    edit_file(base, "elsewhere/Manifest.toml", "name = elsewhere", "name = moved")
    observed["climb 不登记、改 elsewhere"] = fixture_key(base, climb)[0]
    edit_file(base, "tables/rows.tsv", "row", "row moved")
    observed["rows 不登记、改 tables/rows.tsv"] = fixture_key(base, rows)[0]
    registered = dict(TAIL_READ_SCOPES)
    TAIL_READ_SCOPES[scan_read] = (("members",), "自证夹具")
    TAIL_READ_SCOPES[climb_read] = ((), "自证夹具")
    TAIL_READ_SCOPES[rows_read] = ((), "自证夹具")
    try:
        observed["scan 登记"] = fixture_key(base, scan)[0]
        observed["climb 登记"] = fixture_key(base, climb)[0]
        observed["rows 登记"] = fixture_key(base, rows)[0]
        write_tree(base, {"another/Manifest.toml": "name = another\n"})
        edit_file(base, "elsewhere/Manifest.toml", "name = moved", "name = moved again")
        edit_file(base, "tables/rows.tsv", "row moved", "row moved again")
        observed["scan 登记、前缀外多一份"] = fixture_key(base, scan)[0]
        observed["climb 登记、改 elsewhere"] = fixture_key(base, climb)[0]
        observed["rows 登记、改 tables/rows.tsv"] = fixture_key(base, rows)[0]
        edit_file(base, "members/one/Manifest.toml", "name = one", "name = first")
        observed["scan 登记、改 members 底下那一份"] = fixture_key(base, scan)[0]
    finally:
        TAIL_READ_SCOPES.clear()
        TAIL_READ_SCOPES.update(registered)
    held = (None not in observed.values()
            and observed["scan 不登记"] != observed["scan 不登记、前缀外多一份"]
            and observed["climb 不登记"] != observed["climb 不登记、改 elsewhere"]
            and observed["rows 不登记"] != observed["rows 不登记、改 tables/rows.tsv"]
            and observed["scan 登记"] == observed["scan 登记、前缀外多一份"] != observed["scan 登记、改 members 底下那一份"]
            and observed["climb 登记"] == observed["climb 登记、改 elsewhere"]
            and observed["rows 登记"] == observed["rows 登记、改 tables/rows.tsv"])
    return held, (f"各趟的键 {observed}；要：不登记时 scan 前缀外多一份键变、climb 改 elsewhere 键变、rows 改 tables/rows.tsv 键变；"
                  "登记之后 scan 前缀外多一份键不变、改 members 底下那一份键变，climb 改 elsewhere 键不变，rows 改 tables/rows.tsv 键不变；没有一把拒算（None）")


def tail_read_root_scope_cell(work):
    """TAIL_READ_ROOT_SCOPE_CELL：自证期间临时登记 root_manifest 那一处只留根上那一份（TAIL_READ_ROOT_COPY）：research/ 下多一份、改 members 底下那一份键不变，
    改根上的 Manifest.toml 键变；再临时登成空元组对照：改根上那一份键不变（"." 收回的正是空元组漏掉的那一份）。跑完把表还原。
    弄坏开关 tail-read-scopes-root-ignored 下 "." 一份都不留，改根上那一份键不变，判红。"""
    base = os.path.join(work, "tree")
    write_tree(base, TAIL_READ_ROOT_SCOPE_FIXTURE)
    entry = "pkg/rooted.py::head"
    root_read = ("pkg/rooted.py", "root_manifest", "os.path.join(tree, 'Manifest.toml')")
    registered = dict(TAIL_READ_SCOPES)
    observed = {}
    try:
        TAIL_READ_SCOPES[root_read] = ((TAIL_READ_ROOT_COPY,), "自证夹具")
        observed["登根上那一份"] = fixture_key(base, entry)[0]
        write_tree(base, {"research/round/Manifest.toml": "name = research\n"})
        observed["登根上那一份、research 下多一份"] = fixture_key(base, entry)[0]
        edit_file(base, "members/one/Manifest.toml", "name = one", "name = first")
        observed["登根上那一份、改 members 底下那一份"] = fixture_key(base, entry)[0]
        edit_file(base, "Manifest.toml", "name = root", "name = moved")
        observed["登根上那一份、改根上那一份"] = fixture_key(base, entry)[0]
        TAIL_READ_SCOPES[root_read] = ((), "自证夹具")
        observed["登空元组"] = fixture_key(base, entry)[0]
        edit_file(base, "Manifest.toml", "name = moved", "name = moved again")
        observed["登空元组、改根上那一份"] = fixture_key(base, entry)[0]
    finally:
        TAIL_READ_SCOPES.clear()
        TAIL_READ_SCOPES.update(registered)
    held = (None not in observed.values()
            and observed["登根上那一份"] == observed["登根上那一份、research 下多一份"] == observed["登根上那一份、改 members 底下那一份"]
            != observed["登根上那一份、改根上那一份"]
            and observed["登空元组"] == observed["登空元组、改根上那一份"])
    return held, (f"各趟的键 {observed}；要：登根上那一份之后 research 下多一份、改 members 底下那一份键都不变，改根上那一份键变；"
                  "登空元组之后改根上那一份键不变；没有一把拒算（None）")


def function_name_pieces(function):
    """登记表函数一格（registered_function_name 的写法）里要逐字出现在源码里的几段：限定名按 . 切开，<locals> 与 <script-body>、<module-body>、<main-guard>
    这类尖括号记号不是源码里写的名字，不核。"""
    return [piece for piece in function.split(".") if piece and not piece.startswith("<")]


def registered_words_real_cell(work):
    """REGISTERED_WORDS_REAL_CELL：几张登记表点名的文件在真仓都在、函数名与词都还写在那份文件里（引号不比）；SPEC_PATH_ARGUMENTS 点名载入的那一份也在；
    TAIL_READ_SCOPES 那一处的函数名与路径表达式都还写在那份文件里，登记的前缀是真仓里的目录；BUILT_TOOL_LAUNCHES、CALLER_SUPPLIED_COMMANDS 的函数与词
    都还写在那份文件里，BUILT_TOOL_LAUNCHES 点名的编译函数登在 BUILT_TOOL_SOURCES 里。"""
    tree = LIBRARY_ROOT   # 只读不改，登记的文件也在 scripts/ 与 .claude/ 之外（task-server/、prune-server/），不用拷贝

    def plain(text):
        return text.replace('"', "").replace("'", "")

    def source_of(file):
        path = os.path.join(tree, file)
        if not os.path.isfile(path):
            return None
        with open(path, encoding="utf-8", errors="replace") as handle:
            return plain(handle.read())

    unparsed = {}

    def unparsed_of(file):
        """python 文件里每个列表、f 串、二元式照 ast.unparse 的写法（去引号）：跨行写的、相邻字面量拼起来的词在源码里逐字找不到，照这个认。解不开交空集合。"""
        if file not in unparsed:
            try:
                with open(os.path.join(tree, file), encoding="utf-8", errors="replace") as handle:
                    parsed = ast.parse(handle.read())
                unparsed[file] = {plain(ast.unparse(node)) for node in ast.walk(parsed) if isinstance(node, (ast.List, ast.JoinedStr, ast.BinOp))}
            except (SyntaxError, ValueError):
                unparsed[file] = set()
        return unparsed[file]

    rows = dict(UNKEYED_PROGRAM_WORDS)
    if "registered-words-drifted" in active_breaks():
        rows[("scripts/gate/admission.py", "no_such_function", "no_such_word")] = "弄坏开关"
    wanted = [(file, [*function_name_pieces(function), word]) for file, function, word in rows]
    wanted += [(file, function_name_pieces(function)) for file, function in REGISTRY_COMMAND_TEXTS]
    wanted += [(table, []) for table, _column in REGISTRY_COMMAND_TEXTS.values()]
    wanted += [(file, [expression]) for file, expression in SPEC_PATH_ARGUMENTS]
    wanted += [(target, []) for value in SPEC_PATH_ARGUMENTS.values() for target in ((value,) if isinstance(value, str) else value)]
    wanted += [(file, function_name_pieces(function)) for file, function in BUILT_TOOL_SOURCES]
    wanted += [(file, [*function_name_pieces(function), word]) for file, function, word in BUILT_TOOL_LAUNCHES]
    wanted += [(file, [*function_name_pieces(function), argument]) for file, function, argument in CALLER_SUPPLIED_COMMANDS]
    wanted += [(file, [*function_name_pieces(function), word]) for file, function, word in LAUNCHED_REPOSITORY_PROGRAMS]
    wanted += [(target, []) for targets in LAUNCHED_REPOSITORY_PROGRAMS.values() for target in targets]
    # 函数一格写的是内联 python 正文的模块体、main-guard 的（<module-body>、<main-guard>），不是源码里写的名字：只核表达式
    wanted += [(file, [*function_name_pieces(function), expression]) for file, function, expression in TAIL_READ_SCOPES]
    problems = [f"{directory} 不在" for directory in BUILT_TOOL_SOURCES.values() if not os.path.isdir(os.path.join(LIBRARY_ROOT, directory))]
    problems += [f"BUILT_TOOL_LAUNCHES 的 {launch} 点名的编译函数 {builder} 不在 BUILT_TOOL_SOURCES 里" for launch, builder in BUILT_TOOL_LAUNCHES.items()
                 if builder not in BUILT_TOOL_SOURCES]
    problems += [f"按尾巴读的登记前缀 {prefix} 不在" for prefixes, _why in TAIL_READ_SCOPES.values() for prefix in prefixes
                 if not os.path.isdir(os.path.join(LIBRARY_ROOT, prefix))]
    for file, pieces in wanted:
        source = source_of(file)
        if source is None:
            problems.append(f"{file} 不在")
            continue
        problems += [f"{file} 里没有「{piece}」" for piece in pieces
                     if piece != "<script-body>" and plain(piece) not in source and plain(piece) not in unparsed_of(file)]
    return not problems, f"对不上的 {problems}（文件搬了、函数或那一处写法改了：照现状改登记表那一行，那一处不再起就删那一行）"


def registered_functions():
    """几张按（文件, 函数）登记的表里每一行的（文件, 函数, 哪张表）：UNKEYED_PROGRAM_WORDS、LAUNCHED_REPOSITORY_PROGRAMS、BUILT_TOOL_SOURCES、
    BUILT_TOOL_LAUNCHES（键与它点名的编译函数）、CALLER_SUPPLIED_COMMANDS、REGISTRY_COMMAND_TEXTS、TAIL_READ_SCOPES、EXCLUDED_PROGRAM_EDGES、EXCLUDED_CALL_EDGES。
    弄坏开关 registered-functions-drifted 下多一行函数已经挪走的登记（task-server 拆出去的 read_gpu_cards_failed 还登在入口上）。"""
    rows = [(file, function, "UNKEYED_PROGRAM_WORDS") for file, function, _word in UNKEYED_PROGRAM_WORDS]
    rows += [(file, function, "LAUNCHED_REPOSITORY_PROGRAMS") for file, function, _word in LAUNCHED_REPOSITORY_PROGRAMS]
    rows += [(file, function, "BUILT_TOOL_SOURCES") for file, function in BUILT_TOOL_SOURCES]
    rows += [(file, function, "BUILT_TOOL_LAUNCHES") for file, function, _word in BUILT_TOOL_LAUNCHES]
    rows += [(file, function, "BUILT_TOOL_LAUNCHES") for file, function in BUILT_TOOL_LAUNCHES.values()]
    rows += [(file, function, "CALLER_SUPPLIED_COMMANDS") for file, function, _argument in CALLER_SUPPLIED_COMMANDS]
    rows += [(file, function, "REGISTRY_COMMAND_TEXTS") for file, function in REGISTRY_COMMAND_TEXTS]
    rows += [(file, function, "TAIL_READ_SCOPES") for file, function, _expression in TAIL_READ_SCOPES]
    rows += [(file, function, "EXCLUDED_PROGRAM_EDGES") for file, function, _program in EXCLUDED_PROGRAM_EDGES]
    rows += [(file, function, "EXCLUDED_CALL_EDGES") for file, function, _called in EXCLUDED_CALL_EDGES]
    if "registered-functions-drifted" in active_breaks():
        rows.append(("task-server/task_server.py", "read_gpu_cards_failed", "弄坏开关"))
    return rows


def defined_unit_names(python, file):
    """一份文件里单元的名字，照登记表里函数一格的写法（registered_function_name）：python 的 def、方法（<类>.<方法>）、类、模块级赋值、<module-body>、<main-guard>
    与套在函数里的函数（<外层>.<locals>.<内层>）取限定名；shell 的函数（套着的 <外层>::<名> 取 <名>）、顶层变量、<script-body>，
    连同它各单元起的内联 python 正文里的那几样（tail_read_scope_modules）。文件不在交 None。"""
    if not os.path.isfile(os.path.join(python.root, file)):
        return None
    names, modules = set(), []
    if language_of_file(python.root, file) == "shell":
        shell = ShellIndex(python.root, [file])
        shell_file = shell.files.get(file)
        names |= {unit.name.rsplit("::", 1)[-1] for unit in (shell_file.units if shell_file is not None else ()) if not unit.data}
        modules = tail_read_scope_modules(python, file, True)
    elif python.module(file) is not None:
        modules = [python.module(file)]
    for module in modules:
        names |= {registered_function_name(unit) for unit in module.units if not unit.data}
        # 套在函数里的函数不进单元表（PythonModule.nested_unit），起子进程的那一处照它的限定名 <外层>.<locals>.<内层> 登记
        names |= qualified_function_names(module.tree) if module.tree is not None else set()
    return names


def qualified_function_names(tree):
    """一份 python 模块里每个 def 的限定名（PythonModule 的写法：类里的 <类>.<方法>，套在函数里的 <外层>.<locals>.<内层>）。"""
    names = set()

    def visit(node, prefix):
        for child in ast.iter_child_nodes(node):
            if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef)):
                names.add(prefix + child.name)
                visit(child, f"{prefix}{child.name}.<locals>.")
            elif isinstance(child, ast.ClassDef):
                visit(child, f"{prefix}{child.name}.")
            else:
                visit(child, prefix)
    visit(tree, "")
    return names


def registered_functions_real_cell(_work):
    """REGISTERED_FUNCTIONS_REAL_CELL：真仓里逐行核 registered_functions：登记的文件在，登记的函数是那份文件里一个单元的限定名（defined_unit_names）。
    只核文件里写着这个词不够（调用处还留在入口里、定义已挪进拆出的模块的，词照样在）：要定义在这一份里。"""
    python = PythonIndex(LIBRARY_ROOT)   # 只读不改，同 registered_words_real_cell
    names_of, problems = {}, []
    for file, function, table in registered_functions():
        if file not in names_of:
            names_of[file] = defined_unit_names(python, file)
        names = names_of[file]
        if names is None:
            problems.append(f"{table}：{file} 不在")
        elif function not in names:
            problems.append(f"{table}：{file} 里没有单元 {function}")
    return not problems, (f"对不上的 {problems}（函数挪进了拆出的文件：照它此刻定义所在的那一份改登记表那一行的文件，那一处不再起就删那一行）"
                          if problems else f"核了 {len(registered_functions())} 行，登记的函数都定义在登记的文件里")


def tail_read_scope_modules(index, file, inline_read):
    """TAIL_READ_SCOPES_REAL_CELL 核 file 那几行时去哪几份模块里找那一处：.py 是那份文件自己；别的（shell 脚本）是正文写在它里面的内联 python
    （ShellIndex.links 的 python-inline，同 dump 的认法；内联模块的 source_file 算在正文写在的那份 shell 脚本上，与算键时问到的是同一把键）：
    它自己与它（传递地）source 的、source 了它的、和它一起被 source 的那几份里各单元起的，正文出处是它的那几份（拆成入口加库：入口把程序装进变量、库里起它），
    连同这些内联 python（与它们再载入的）exec 进模块对象的库正文里出处是它的那几份（module_object_exec）。
    不在、解不开、一份内联正文都没认出交空表。弄坏开关 tail-read-scopes-inline-unread 下 shell 脚本也照 .py 读（解不开）；
    registered-functions-module-object-texts-unread 下不往 exec 进模块对象的库正文里找。"""
    if file.endswith(".py") or not inline_read:
        module = index.module(file)
        return [] if module is None or module.tree is None else [module]
    if not os.path.isfile(os.path.join(index.root, file)):
        return []
    shell = index.__dict__.get("_tail_read_shell_index")
    if shell is None:
        shell = index._tail_read_shell_index = ShellIndex(index.root, shell_files_under(index.root))   # 这一格的几份 shell 脚本共用一份索引
    shell.add([file])
    if file not in shell.files:
        return []
    related = sorted({file} | shell.downward([file]) | shell.upward(file) | shell.siblings(file))
    scripts = dict.fromkeys(script for path in related if path in shell.files for unit in shell.files[path].units if not unit.data
                            for kind, found, _refusal in shell.links(unit) if kind == "python-inline" for script in found)
    modules = [module for script in scripts for module in index.inline_modules(script) if module.tree is not None]
    if "registered-functions-module-object-texts-unread" not in active_breaks():
        # 内联 python exec 进模块对象的库正文（文件头「exec 进模块对象」）：正文出自这份文件里一份 here-doc 的同样算在这份文件上（source_file），接着往它 exec 的找
        exec_texts = index.__dict__.setdefault("_module_object_text_modules", {})   # 载入方 → 它 exec 进模块对象的库正文的模块（几份文件共用，一份载入方只扫一遍）
        seen, position = {id(module) for module in modules}, 0
        while position < len(modules):
            loader, position = modules[position], position + 1
            if id(loader) not in exec_texts:
                found = []
                for unit in loader.units:
                    for node in ast.walk(ast.Module(body=list(unit.nodes or []), type_ignores=[])):
                        loaded = loader.module_object_exec(node, unit) if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) \
                            and node.func.id == "exec" else None
                        if loaded is not None and loaded[0] == "texts":
                            found += [module for module in index.inline_modules(loaded[1]) if module.tree is not None]
                exec_texts[id(loader)] = found
            for module in exec_texts[id(loader)]:
                if id(module) not in seen:
                    seen.add(id(module))
                    modules.append(module)
    return [module for module in modules if module.source_file() == file]


def tail_read_scope_statements(unit, inline_read):
    """一个 python 单元里去哪几句找登记的那一处：函数单元是那个 def；模块级赋值、模块体、main-guard（TAIL_READ_SCOPE_STATEMENT_KINDS）是它们的那几句。
    弄坏开关 tail-read-scopes-inline-unread 下只认函数单元。"""
    node = getattr(unit, "node", None)
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
        return [node]
    if inline_read and getattr(unit, "kind", None) in TAIL_READ_SCOPE_STATEMENT_KINDS:
        return list(getattr(unit, "nodes", None) or ())
    return []


def tail_read_scopes_real_cell(work):
    """TAIL_READ_SCOPES_REAL_CELL：真仓里逐行核 TAIL_READ_SCOPES（文件头「按尾巴读的登记」）：登记的文件里有限定名（registered_function_name）是登记函数名的单元（函数，或模块级语句：
    tail_read_scope_statements；shell 脚本到它起的内联 python 正文里找：tail_read_scope_modules），单元里有一处表达式
    的 ast.unparse 与登记的逐字相同；从那几处起 data_read_origins，算的时候真问到了这一行（PythonModule.registered_tail_scope 对这把键交回的前缀与登记的相同），
    收进来的文件都落在登记前缀下（TAIL_READ_ROOT_COPY 只认根上那一份：in_tail_read_scope；空元组一份都不收）。几份模块先全部换上记问询的那一层再起算：一处读经调用处问到别的登记模块时也记得到。
    弄坏开关 tail-read-scopes-drifted 下多一行真仓那个函数里没有的表达式；tail-read-scopes-inline-unread 下只认 .py 文件里的函数单元。"""
    rows = dict(TAIL_READ_SCOPES)
    if "tail-read-scopes-drifted" in active_breaks():
        rows[("task-server/task_server_lib_commands.py", "cargo_manifest_scope", "os.path.join(current, 'no-such-manifest.toml')")] = ((), "弄坏开关")
    index = PythonIndex(LIBRARY_ROOT)   # 只读不改，同 registered_words_real_cell
    asked = []   # (文件, 函数的限定名, 表达式, 交回的前缀)：registered_tail_scope 每问一次记一行

    def record_questions(module):
        answer = module.registered_tail_scope

        def recorded_tail_scope(node, unit):
            scope = answer(node, unit)
            asked.append((module.source_file(), registered_function_name(unit), ast.unparse(node), scope))
            return scope
        module.registered_tail_scope = recorded_tail_scope

    inline_read = "tail-read-scopes-inline-unread" not in active_breaks()
    problems, modules = [], {}
    for file in sorted({file for file, _function, _expression in rows}):
        found = tail_read_scope_modules(index, file, inline_read)
        if not found:
            problems.append(f"{file} 不在或解不开" + ("（shell 脚本里一份内联 python 正文都没认出）" if inline_read and not file.endswith(".py") else ""))
            continue
        for module in found:
            record_questions(module)
        modules[file] = found
    for (file, function, expression), (prefixes, _why) in sorted(rows.items()):
        if file not in modules:
            continue
        sites = [(node, unit) for module in modules[file] for unit in module.units
                 if registered_function_name(unit) == function
                 for statement in tail_read_scope_statements(unit, inline_read)
                 for node in ast.walk(statement) if isinstance(node, ast.expr) and ast.unparse(node) == expression]
        if not sites:
            problems.append(f"{file} 的 {function} 里没有一处「{expression}」")
            continue
        wanted = tuple(prefix.strip("/") for prefix in prefixes)
        collected = set()
        for node, unit in sites:
            for origin in unit.module.data_read_origins(node, unit):
                if origin[0] in ("file", "directory"):
                    collected.add(origin[1])
                elif origin[0] == "param-tail":
                    collected |= set(origin[3])
        answers = {scope for asked_file, asked_function, asked_expression, scope in asked
                   if (asked_file, asked_function, asked_expression) == (file, function, expression)}
        if wanted not in answers:
            problems.append(f"{file} 的 {function}「{expression}」：算键时问到这一行交回的前缀是 {sorted(answers, key=repr) or '一次都没问到'}，登记的是 {wanted}")
        # TAIL_READ_ROOT_COPY 认的那截尾巴（同 data_read_origins_once）：照尾巴收的取 path_tails，tree_copy_file 那一种取它收的根里那一份
        tails = set()
        for node, unit in sites:
            tails |= {tail.strip("/") for tail in (unit.module.path_tails(node, unit) or ())}
            literal = unit.module.literal_tail(node, unit)
            copied = tree_copy_file(index.root, literal) if literal else None
            if copied is not None:
                tails.add(copied)
        outside = sorted(path for path in collected if not in_tail_read_scope(path, wanted, tails, active_breaks()))
        if outside:
            problems.append(f"{file} 的 {function}「{expression}」：收进了登记前缀 {wanted or '（空元组：一份都不该收）'} 外的 {outside[:5]}"
                            + (f" 等 {len(outside)} 份" if len(outside) > 5 else ""))
    return not problems, (f"核了 {len(rows)} 行，对不上的 {problems}（那一处搬了、写法改了、换了函数：照算键时真问到的那一处（参数照调用处的实参认）改登记表那一行；"
                          "前缀外收进了文件：登记的前缀不是那一处真走到的地方，照它真读的范围改）")


SELFTEST_REGISTRY = ".claude/hooks/selftest-registry.tsv"


def selftest_registry_scripts(root):
    """root 里 .claude/hooks/selftest-registry.tsv 第二列正好是 selftest 的那几行的路径（samples 那几种是门禁样本档，不算）；表不在交空表。"""
    path = os.path.join(root, SELFTEST_REGISTRY)
    if not os.path.isfile(path):
        return []
    scripts = []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            fields = line.rstrip("\n").split("\t")
            if line.strip() and not line.lstrip().startswith("#") and len(fields) >= 2 and fields[1].strip() == "selftest":
                scripts.append(fields[0].strip())
    return scripts


def records_tail_reads_of(job):
    """(入口, {(文件, 行, 函数的限定名, 表达式): [文件…]}, 收不成的原因或 None)：一个入口的闭包里，按尾巴读而没登记 TAIL_READ_SCOPES 的那几处（UNSCOPED_TAIL_READS 记的）
    收进来的、落在 records/、research/ 下的文件——读它的函数在这个闭包里（内联脚本认它所在的那份文件），记下的文件也收进了这个闭包的才算。"""
    root, entry, arguments = job
    options = Options()
    options.program_arguments = arguments
    try:
        closure = closure_of(root, entry, options)
    except (FunctionUnitsError, UsageError) as error:
        return entry, {}, str(error)
    absolute = os.path.abspath(root)
    members = {(unit.file, registered_function_name(unit)) for unit in closure.order if not unit.data}
    member_files = {file for file, _short in members}
    collected = {unit.file for unit in closure.order if unit.data and unit.file.startswith(TAIL_READ_RECORD_PREFIXES)}
    hits = {}
    for (site_root, file, line, short, expression, inline), files in UNSCOPED_TAIL_READS.items():
        if site_root != absolute or not ((file, short) in members or (inline and file in member_files)):
            continue
        inside = sorted(files & collected)
        if inside:
            hits[(file, line, short, expression)] = inside
    return entry, hits, None


def records_tail_reads(root, jobs):
    """jobs 是 [(入口, 程序入口的参数或 None)] → ({(文件, 行, 函数的限定名, 表达式): (收进来的文件的集合, 入口的集合)}, {入口: 收不成的原因})。
    按每台默认重活线程数并行（judging_entry_refusal_workers）；同 judging_entry_refusals，父进程先建好脚本索引、冻进永久代，子进程共用。
    开跑前清掉这个根之前记下的那几处：同一个进程里前一趟（登记之前）记的不带到这一趟。"""
    import concurrent.futures
    import gc
    import multiprocessing
    absolute = os.path.abspath(root)
    for stale in [key for key in UNSCOPED_TAIL_READS if key[0] == absolute]:
        del UNSCOPED_TAIL_READS[stale]
    workers = judging_entry_refusal_workers(os.environ)
    _prepared, key = prepare_shared_script_index(root)
    gc.collect()
    gc.freeze()
    try:
        with concurrent.futures.ProcessPoolExecutor(max_workers=max(1, min(workers, len(jobs))), mp_context=multiprocessing.get_context("fork")) as pool:
            results = list(pool.map(records_tail_reads_of, [(root, entry, arguments) for entry, arguments in jobs]))
    finally:
        gc.unfreeze()
        SHARED_SCRIPT_INDICES.pop(key, None)
    if len(results) != len(jobs):
        return {}, {"（整趟）": f"派出去 {len(jobs)} 个入口、收回 {len(results)} 个"}
    sites, problems = {}, {}
    for entry, hits, problem in results:
        if problem is not None:
            problems[entry] = problem
        for site, files in hits.items():
            known_files, entries = sites.get(site, (set(), set()))
            sites[site] = (known_files | set(files), entries | {entry})
    return sites, problems


def tail_reads_records_real_cell(work):
    """TAIL_READS_RECORDS_REAL_CELL（文件头「按尾巴读的登记」末段）：先在夹具上证 scan 那一处没登记时判得出 records/、research/ 下那两份，
    临时登记前缀 members 之后一处都不判（跑完把表还原）；再在真仓收登记的判法入口与自证登记表里的自证的闭包，有一处判红就红。"""
    fixture = os.path.join(work, "fixture")
    write_tree(fixture, TAIL_READS_RECORDS_FIXTURE)
    scan_entry = [("pkg/scoped.py::scan", None)]
    scan_read = ("pkg/scoped.py", "member_texts", "os.path.join(directory, 'Manifest.toml')")
    wanted = {"records/round/Manifest.toml", "research/round/Manifest.toml"}
    unregistered, unregistered_problems = records_tail_reads(fixture, scan_entry)
    found = {(file, short, expression): files for (file, _line, short, expression), (files, _entries) in unregistered.items()}
    registered = dict(TAIL_READ_SCOPES)
    TAIL_READ_SCOPES[scan_read] = (("members",), "自证夹具")
    try:
        scoped, scoped_problems = records_tail_reads(fixture, scan_entry)
    finally:
        TAIL_READ_SCOPES.clear()
        TAIL_READ_SCOPES.update(registered)
    fixture_held = not unregistered_problems and not scoped_problems and found == {scan_read: wanted} and not scoped
    if not fixture_held:
        return False, (f"夹具：没登记时要恰好判出 {scan_read} 收进 {sorted(wanted)}，实际 {found}（收不成的 {unregistered_problems}）；"
                       f"登记 members 之后要一处都不判，实际 {sorted(scoped)}（收不成的 {scoped_problems}）")
    entries = [(entry, None) for entry in registered_judging_entries(LIBRARY_ROOT)
               if entry.split("::", 1)[0].endswith(TAIL_READS_RECORDS_SCRIPT_SUFFIXES)]
    entries += [(path, ["--selftest"]) for path in selftest_registry_scripts(LIBRARY_ROOT) if path.endswith(TAIL_READS_RECORDS_SCRIPT_SUFFIXES)]
    real, real_problems = records_tail_reads(LIBRARY_ROOT, entries)
    lines = [f"{file}:{line} {short}「{expression}」收进 {len(files)} 份 {sorted(files)[:5]}（{len(entry_set)} 个入口的闭包，头一个 {sorted(entry_set)[0]}）"
             for (file, line, short, expression), (files, entry_set) in sorted(real.items())]
    lines += [f"入口 {entry} 收不成闭包：{problem}" for entry, problem in sorted(real_problems.items())]
    return not lines, (f"真仓 {len(entries)} 个入口里 {len(lines)} 处不对：{lines}（按尾巴整组收的那一处没登记 TAIL_READ_SCOPES：照那一处真走到的地方登记一行"
                       "（文件, 函数的限定名, 表达式）→（前缀, 为什么），读的是调用方交进来的输入的写空元组；收不成闭包的照那一句修）")



def tool_source_cell(work):
    """TOOL_SOURCE_CELL：自证期间临时把夹具里的 build_program 登记进 BUILT_TOOL_SOURCES；改源码目录里的 main.rs 键变，改 target 下的二进制键不变；跑完把表还原。"""
    base = os.path.join(work, "tree")
    write_tree(base, BUILD_OUTPUT_FIXTURE)
    registered = dict(BUILT_TOOL_SOURCES)
    BUILT_TOOL_SOURCES[("pkg/build.py", "build_program")] = "crates/sample-tool"
    try:
        before, before_refused = fixture_key(base, BUILD_OUTPUT_ENTRY)
        edit_file(base, "target/release/tool", "one", "two")
        after_binary, _refused = fixture_key(base, BUILD_OUTPUT_ENTRY)
        edit_file(base, "crates/sample-tool/src/main.rs", "fn main() {}", "fn main() { let _ = 1; }")
        after_source, after_refused = fixture_key(base, BUILD_OUTPUT_ENTRY)
    finally:
        BUILT_TOOL_SOURCES.clear()
        BUILT_TOOL_SOURCES.update(registered)
    if None in (before, after_binary, after_source):
        return False, f"拒算了：改前 {sorted(before_refused)}，改源码后 {sorted(after_refused)}"
    return before == after_binary and before != after_source, (f"改 target 下的二进制：期望键不变，实际{'不变' if before == after_binary else '变了'}；"
                                                              f"改源码目录里的 main.rs：期望键变，实际{'变了' if before != after_source else '不变'}")



def built_tool_launch_cell(work):
    """BUILT_TOOL_LAUNCH_CELL：不登记时 rust() 起 binary 那一处拒算；自证期间临时登记夹具里的编译函数（BUILT_TOOL_SOURCES）与起它的那一处（BUILT_TOOL_LAUNCHES），
    不拒算、改 crates/sample-tool/src/main.rs 里的 helper 键变；跑完把两张表还原。"""
    base = os.path.join(work, "tree")
    write_tree(base, BUILT_TOOL_LAUNCH_FIXTURE)
    unregistered, unregistered_refused = fixture_key(base, BUILT_TOOL_LAUNCH_ENTRY)
    sources, launches = dict(BUILT_TOOL_SOURCES), dict(BUILT_TOOL_LAUNCHES)
    BUILT_TOOL_SOURCES.update(BUILT_TOOL_LAUNCH_FIXTURE_SOURCES)
    BUILT_TOOL_LAUNCHES.update(BUILT_TOOL_LAUNCH_FIXTURE_LAUNCHES)
    try:
        before, before_refused = fixture_key(base, BUILT_TOOL_LAUNCH_ENTRY)
        edit_file(base, "crates/sample-tool/src/main.rs", "    1\n", "    2\n")
        after, after_refused = fixture_key(base, BUILT_TOOL_LAUNCH_ENTRY)
    finally:
        BUILT_TOOL_SOURCES.clear()
        BUILT_TOOL_SOURCES.update(sources)
        BUILT_TOOL_LAUNCHES.clear()
        BUILT_TOOL_LAUNCHES.update(launches)
    ok = unregistered is None and None not in (before, after) and before != after
    return ok, (f"不登记时拒算 {sorted(unregistered_refused)}（要拒算）；登记之后改前拒算 {sorted(before_refused)}、改后拒算 {sorted(after_refused)}（都要不拒算）；"
                f"改 helper 之后键{'变了' if None not in (before, after) and before != after else '没变或算不出'}（要变）")


# 夹具：group.py 的 run_in_group 同 scripts/gate/lib_process_group.py 的 run_in_own_session（拿参数起子进程、本模块里没人调它）；
# callers.py 两处调用交的命令都取环境变量（往上认不出），自证期间只登记 registered_site 那一处
CALLER_SUPPLIED_FIXTURE = {
    "pkg/group.py": "import subprocess\n\n\ndef run_in_group(command, timeout=None):\n    return subprocess.run(command, check=False, timeout=timeout).returncode\n",
    "pkg/callers.py": ('import os\n\nimport group\n\n\n'
                       'def registered_site():\n    return group.run_in_group(["bash", os.environ["SAMPLE_SCRIPT"], "--selftest"], timeout=30)\n\n\n'
                       'def unregistered_site():\n    return group.run_in_group(["bash", os.environ["OTHER_SCRIPT"], "--check"])\n\n\n'
                       'def both_sites():\n    return registered_site() + unregistered_site()\n'),
}
CALLER_SUPPLIED_FIXTURE_ROWS = {("pkg/callers.py", "registered_site", "['bash', os.environ['SAMPLE_SCRIPT'], '--selftest']"): "夹具"}


def with_caller_rows(judge):
    """自证期间临时把 CALLER_SUPPLIED_FIXTURE_ROWS 并进 CALLER_SUPPLIED_COMMANDS 再判，跑完还原。"""
    registered = dict(CALLER_SUPPLIED_COMMANDS)
    CALLER_SUPPLIED_COMMANDS.update(CALLER_SUPPLIED_FIXTURE_ROWS)
    try:
        return judge()
    finally:
        CALLER_SUPPLIED_COMMANDS.clear()
        CALLER_SUPPLIED_COMMANDS.update(registered)


def caller_supplied_registered_cell(work):
    """CALLER_SUPPLIED_REGISTERED_CELL：registered_site 当入口，不登记时拒算（run_in_group 的命令往上认到这一处、认不出）；登记了这一处不拒算。"""
    base = os.path.join(work, "tree")
    write_tree(base, CALLER_SUPPLIED_FIXTURE)
    before, before_refused = fixture_key(base, "pkg/callers.py::registered_site")
    after, after_refused = with_caller_rows(lambda: fixture_key(base, "pkg/callers.py::registered_site"))
    return before is None and after is not None, (f"不登记时拒算 {sorted(before_refused)}（要拒算）；登记了 registered_site 那一处之后拒算 {sorted(after_refused)}"
                                                  "（要不拒算）")


def caller_supplied_unregistered_cell(work):
    """CALLER_SUPPLIED_UNREGISTERED_CELL：只登记了 registered_site 那一处；unregistered_site 当入口、both_sites（两处都调）当入口都照拒算。"""
    base = os.path.join(work, "tree")
    write_tree(base, CALLER_SUPPLIED_FIXTURE)
    alone, alone_refused = with_caller_rows(lambda: fixture_key(base, "pkg/callers.py::unregistered_site"))
    both, both_refused = with_caller_rows(lambda: fixture_key(base, "pkg/callers.py::both_sites"))
    return alone is None and both is None, (f"unregistered_site 当入口拒算 {sorted(alone_refused)}、both_sites 当入口拒算 {sorted(both_refused)}"
                                            "（都要拒算：没登记的那一处不跟着登记了的那一处放行）")



def unit_graph_text_busy_cell(work):
    """造一份可执行的小脚本，另一条线程开着它写、过一会儿关掉；这期间起它：先核直接起真撞 ETXTBSY（撞不上这台判不了、照实报），再核等过去之后起得成。"""
    program = os.path.join(work, "busy-program")
    with open(program, "w", encoding="utf-8") as handle:
        handle.write("#!/bin/sh\necho started\n")
    os.chmod(program, 0o755)
    writer = open(program, "r+b")   # noqa: SIM115  要跨线程开着、到点才关
    try:
        try:
            subprocess.run([program], capture_output=True, stdin=subprocess.DEVNULL)
            return False, "开着写的时候直接起它没有撞 ETXTBSY：这台的内核不拒，这一格在这里造不出要验的条件"
        except OSError as error:
            if error.errno != errno.ETXTBSY:
                return False, f"开着写的时候直接起它抛的不是 ETXTBSY：{error}"
        closer = threading.Timer(3 * UNIT_GRAPH_TEXT_BUSY_PAUSE_SECONDS, writer.close)
        closer.start()
        try:
            completed = run_program_waiting_out_text_busy([program])
        except OSError as error:
            return False, f"写的那一方 {3 * UNIT_GRAPH_TEXT_BUSY_PAUSE_SECONDS} 秒后就关了，等过去之后该起得成，实际抛 {error}"
        finally:
            closer.join()
    finally:
        writer.close()
    return completed.returncode == 0 and completed.stdout == "started\n", f"退 {completed.returncode}、输出 {completed.stdout!r}（要退 0、打 started）"


def with_unit_graph_cache(work, judge):
    """编 unit-graph 那几格：小仓 UNIT_GRAPH_STEP_FIXTURE 写进 work，GATE_CROSS_RUN_TMPDIR 换成 work 下的目录（程序缓存落在它底下），
    judge(小仓, 缓存目录, 这份小仓的程序该放的目录) 跑完把变量还原。各格在自己 fork 出来的进程里跑，改环境只波及这一格。"""
    repository = os.path.join(work, "repository")
    write_tree(repository, UNIT_GRAPH_STEP_FIXTURE)
    saved = os.environ.get("GATE_CROSS_RUN_TMPDIR")
    os.environ["GATE_CROSS_RUN_TMPDIR"] = os.path.join(work, "cross-run")
    try:
        cache = unit_graph_module.unit_graph_product_view_cache_directory()
        source = os.path.join(repository, "crates", "singlefs-unit-graph")
        digest = unit_graph_module.unit_graph_inputs_digest(repository, source, os.path.join(repository, ".claude", "heavy-thread-default"),
                                                            os.path.join(repository, "Cargo.lock"))
        return judge(repository, cache, os.path.join(cache, digest))
    finally:
        if saved is None:
            os.environ.pop("GATE_CROSS_RUN_TMPDIR", None)
        else:
            os.environ["GATE_CROSS_RUN_TMPDIR"] = saved


def unit_graph_failed_step_cell(work):
    """UNIT_GRAPH_FAILED_STEP_CELL：缓存目录的位置上放一份普通文件，built_unit_graph 要抛 ProductViewUnavailable，那一句带「os.makedirs（<缓存目录>）」。
    弄坏开关 unit-graph-failure-step-unnamed 下只报缓存目录与 errno，这一格红。"""
    def judge(repository, cache, _directory):
        os.makedirs(os.path.dirname(cache), exist_ok=True)
        with open(cache, "w", encoding="utf-8") as handle:
            handle.write("不是目录\n")
        try:
            unit_graph_module.built_unit_graph(repository)
        except unit_graph_module.ProductViewUnavailable as error:
            wanted = f"os.makedirs（{cache}）"
            return wanted in str(error), f"报的是「{error}」（要带「{wanted}」）"
        return False, "缓存目录的位置上是普通文件，built_unit_graph 却没抛"
    return with_unit_graph_cache(work, judge)


def unit_graph_present_program_cell(work):
    """UNIT_GRAPH_PRESENT_PROGRAM_CELL：这份小仓的程序目录里先放一份没有执行位的 unit-graph，build_unit_graph 换成只记一笔的：要交那个目录、一次都不编。
    弄坏开关 unit-graph-present-program-rebuilt 下只认执行位，拿锁之后照编，这一格红。"""
    def judge(repository, _cache, directory):
        os.makedirs(directory)
        with open(os.path.join(directory, "unit-graph"), "w", encoding="utf-8") as handle:
            handle.write("#!/bin/sh\necho fake\n")
        os.chmod(os.path.join(directory, "unit-graph"), 0o644)
        builds = []
        original = unit_graph_module.build_unit_graph
        unit_graph_module.build_unit_graph = lambda *arguments: builds.append(arguments)
        try:
            got = unit_graph_module.built_unit_graph(repository)
        finally:
            unit_graph_module.build_unit_graph = original
        return got == directory and not builds, f"交 {got}（要 {directory}），编了 {len(builds)} 次（要 0 次）"
    return with_unit_graph_cache(work, judge)


def unit_graph_shared_directories_cell(work):
    """UNIT_GRAPH_SHARED_DIRECTORIES_CELL：缓存目录与这份小仓的程序目录先建好（别的进程建的那一种），编译那一步换成假的（照 CARGO_TARGET_DIR 写一份可执行的小脚本）：
    built_unit_graph 要交那个目录、程序换上了、有执行位。弄坏开关 unit-graph-directories-not-shared 下建目录不带 exist_ok，撞上已在的缓存目录抛，这一格红。"""
    def fake_run(command, **keywords):
        built = os.path.join(keywords["env"]["CARGO_TARGET_DIR"], "release", "unit-graph")
        os.makedirs(os.path.dirname(built), exist_ok=True)
        with open(built, "w", encoding="utf-8") as handle:
            handle.write("#!/bin/sh\necho fake\n")
        os.chmod(built, 0o755)
        return subprocess.CompletedProcess(command, 0, "", "")

    def judge(repository, _cache, directory):
        os.makedirs(directory)
        original = unit_graph_module.subprocess
        unit_graph_module.subprocess = types.SimpleNamespace(run=fake_run, DEVNULL=subprocess.DEVNULL)
        try:
            got = unit_graph_module.built_unit_graph(repository)
        except unit_graph_module.ProductViewUnavailable as error:
            return False, f"目录已在时抛了：{error}"
        finally:
            unit_graph_module.subprocess = original
        program = os.path.join(directory, "unit-graph")
        return got == directory and os.access(program, os.X_OK), f"交 {got}（要 {directory}），程序 {'可执行' if os.access(program, os.X_OK) else '不在或没有执行位'}"
    return with_unit_graph_cache(work, judge)




def inline_refusal_line_cell(work):
    """INLINE_REFUSAL_LINE_CELL：s/inline_refusal_line.sh 的闭包里 globals-lookup 的拒算恰好两条，各落在夹具里两处 globals().get(…) 那一行、片段是那一行原文。"""
    base = os.path.join(work, "tree")
    write_tree(base, INLINE_REFUSAL_LINE_FIXTURE)
    path = "s/inline_refusal_line.sh"
    closure = closure_of(base, path)
    wanted = sorted((path, number, text.strip()) for number, text in enumerate(INLINE_REFUSAL_LINE_FIXTURE[path].splitlines(), 1) if "globals().get(" in text)
    got = sorted((refusal.file, refusal.line, refusal.snippet) for refusal in closure.refusals if refusal.form == "globals-lookup")
    return got == wanted, f"globals-lookup 的拒算落在 {got}（要 {wanted}：here-doc 正文头一行加正文里的偏移）"


def inline_sourced_body_cell(work):
    """INLINE_SOURCED_BODY_CELL：s/sourced_body.sh 把程序装进 LOOKUP_PROGRAM、它 source 的 s/lib/sourced_body_run.sh 里的函数 python3 -c 起它：
    globals-lookup 的拒算恰好一条，落在入口里 globals().get(…) 那一行、片段是那一行原文；那份内联模块算在入口上（source_file）。"""
    base = os.path.join(work, "tree")
    write_tree(base, INLINE_SOURCED_BODY_FIXTURE)
    path = "s/sourced_body.sh"
    closure = closure_of(base, path)
    wanted = [(path, number, text.strip()) for number, text in enumerate(INLINE_SOURCED_BODY_FIXTURE[path].splitlines(), 1) if "globals().get(" in text]
    got = sorted((refusal.file, refusal.line, refusal.snippet) for refusal in closure.refusals if refusal.form == "globals-lookup")
    return got == wanted, f"globals-lookup 的拒算落在 {got}（要 {wanted}：正文写在入口里，落在入口里那一行）"


def module_object_refusal_lines_cell(work):
    """MODULE_OBJECT_REFUSAL_LINES_CELL：s/located_entry.sh source 的 s/lib/located_library.sh 用 IFS= read -r -d '' 把一段库正文读进 LOCATED_LIBRARY，
    入口把它当参数交给内联 python、exec 进模块对象再调 lookup：globals-lookup 的拒算恰好一条，落在那份库里 globals().get(…) 那一行、片段是那一行原文
    （算键库文件头「exec 进模块对象」拒算落在哪那一段）。"""
    base = os.path.join(work, "tree")
    write_tree(base, MODULE_OBJECT_TEXT_SOURCES_FIXTURE)
    library = "s/lib/located_library.sh"
    closure = closure_of(base, "s/located_entry.sh")
    wanted = [(library, number, text.strip()) for number, text in enumerate(MODULE_OBJECT_TEXT_SOURCES_FIXTURE[library].splitlines(), 1)
              if "globals().get(" in text]
    got = sorted((refusal.file, refusal.line, refusal.snippet) for refusal in closure.refusals if refusal.form == "globals-lookup")
    others = sorted((refusal.form, refusal.file, refusal.line) for refusal in closure.refusals if refusal.form != "globals-lookup")
    return got == wanted and not others, (f"globals-lookup 的拒算落在 {got}（要 {wanted}：库正文写在被 source 的那份库里，落在那份库里那一行）；"
                                          f"别的拒算 {others}（要没有）")


HEREDOC_BLANK_ROW_CELL = "shell：bash <脚本> run，脚本里的 here-doc 带空行：代位置参数走得出来，--selftest 那一支照样剪掉（改 s/child.sh 键不变）"


def stage_cell_functions_real_cell(_work):
    """STAGE_CELL_FUNCTIONS_REAL_CELL：门禁目录顶层每道阶段的 stage_cell 行第二列是字面函数名；弄坏开关 stage-cell-functions-drifted 下多一行拿变量当函数的。"""
    directory = os.path.join(LIBRARY_ROOT, ".claude", "gate.d")
    rows = []
    for name in sorted(os.listdir(directory)):
        if name.endswith(".sh"):
            with open(os.path.join(directory, name), encoding="utf-8", errors="replace") as handle:
                rows += [(name, line.split()) for line in handle if line.lstrip().startswith("stage_cell ")]
    if "stage-cell-functions-drifted" in active_breaks():
        rows.append(("弄坏开关", ["stage_cell", "a-cell", '"$function_variable"']))
    drifted = [(name, words[1:3]) for name, words in rows if len(words) < 3 or not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", words[2])]
    return bool(rows) and not drifted, (f"查了 {len(rows)} 行 stage_cell，第二列不是字面函数名的：{drifted}（写成字面函数名，"
                                        "不然那个函数连不进键；要动态选函数就改 UNKEYED_PROGRAM_WORDS 里 stage_cells_run 那一行的理由）")


def narrowed_view_shared_cell(work):
    """NARROWED_VIEW_SHARED_CELL：s/shared_view.sh 只看 $1，("run", "one") 与 ("run", "two") 剪出同一份正文，("--selftest",) 剪出另一份。"""
    base = os.path.join(work, "tree")
    write_tree(base, SHELL_FOLDS_FIXTURE)
    file = "s/shared_view.sh"
    index = ShellIndex(base, [file])
    first, second = index.narrowed_script_body(file, ("run", "one")), index.narrowed_script_body(file, ("run", "two"))
    other = index.narrowed_script_body(file, ("--selftest",))
    return first is second and first is not other and first is not index.files[file].script_body, (
        f"同一份正文共用一个视图：{first is second}（要 True）；别的正文另一个：{first is not other}（要 True）；剪过的不是原来那一份：{first is not index.files[file].script_body}（要 True）")


def test_service_code_decision_items():
    """D13（验证路线） 已定项 17 三类代码表测试服务代码那一行（TEST_SERVICE_CODE_DECISION）→ (包含那一列里反引号的各项, 那一列提不提 WGSL)。
    以那个行首起头的行不是恰好一行抛 AssertionError（表换了样子，这一格照新样子改）。"""
    path, head = TEST_SERVICE_CODE_DECISION
    with open(os.path.join(LIBRARY_ROOT, path), encoding="utf-8") as handle:
        found = [line for line in handle if line.startswith(head)]
    if len(found) != 1:
        raise AssertionError(f"{path} 里以「{head}」起头的行有 {len(found)} 行（要恰好一行：三类代码表测试服务代码那一行）")
    included = found[0].split("|")[2]
    return re.findall(r"`([^`]+)`", included), "WGSL" in included


def repository_judge_kernels():
    """仓里 GPU 判器的 WGSL 内核：TEST_SERVICE_CODE_KERNEL_PLACES 底下名字以 TEST_SERVICE_CODE_KERNEL_PREFIX 起头的 .wgsl（相对仓根，排好序）。"""
    places = [place for place in TEST_SERVICE_CODE_KERNEL_PLACES if os.path.isdir(os.path.join(LIBRARY_ROOT, place))]
    return [path for path in files_under(LIBRARY_ROOT, (".wgsl",), places) if posixpath.basename(path).startswith(TEST_SERVICE_CODE_KERNEL_PREFIX)]


def test_service_code_table_cell(work):
    """TEST_SERVICE_CODE_TABLE_CELL：登记表与 D13 那一行逐项比。反引号里的一项：以 / 收尾的要有同一行，不带 / 与 . 的是包名、要有 crates/<包名>/ 那一行，
    别的是文件名、要恰好一行登的文件叫这个名字；那一列提了 WGSL 的，仓里 GPU 判器的每一份内核（repository_judge_kernels）都要登了（按文件，或落在登的前缀底下），
    登的 .wgsl 都要是其中一份；登的每一行都要出自上面两样，登的路径在仓里都要在。弄坏开关 test-service-code-table-row-dropped 下表里少头一行，这一格必红。"""
    items, kernels_named = test_service_code_decision_items()
    rows = test_service_code_rows()
    problems, covered = [], set()
    for item in items:
        if item.endswith("/"):
            hit = [item] if item in rows else []
        elif "/" not in item and "." not in item:
            hit = [f"crates/{item}/"] if f"crates/{item}/" in rows else []
        else:
            hit = [row for row in rows if posixpath.basename(row) == item]
        if len(hit) != 1:
            problems.append(f"D13 那一行的 {item} 在表里有 {hit}（要恰好一行）")
        covered.update(hit)
    listed_kernels, kernels = sorted(row for row in rows if row.endswith(".wgsl")), repository_judge_kernels()
    if kernels_named:
        covered.update(listed_kernels)
        unlisted = [kernel for kernel in kernels if kernel not in rows and not any(row.endswith("/") and kernel.startswith(row) for row in rows)]
        stale = [row for row in listed_kernels if row not in kernels]
        if unlisted or stale or not kernels:
            problems.append(f"GPU 判器的内核：仓里找到 {len(kernels)} 份，没登的 {unlisted}，登了却不是仓里内核的 {stale}")
    extra = sorted(set(rows) - covered)
    if extra:
        problems.append(f"登了 D13 那一行没有的 {extra}")
    missing = [row for row in rows if not os.path.exists(os.path.join(LIBRARY_ROOT, row.rstrip("/")))]
    if missing:
        problems.append(f"登的路径仓里不在 {missing}")
    return bool(items) and not problems, (f"D13 那一行 {len(items)} 项、表里 {len(rows)} 行：{'；'.join(problems)}"
                                          "（照 D13 那一行改 scripts/gate/test-service-code-paths.tsv，两边同一次改）")


def test_service_code_real_cell(work):
    """TEST_SERVICE_CODE_REAL_CELL：真仓里该判「是」的三份（crates/singlefs-unit-graph/src 下头一份 .rs、仓里头一份 GPU 判器 .wgsl、登的 .rs 的 <名>/ 下头一份子模块）
    与该判「不是」的两份（TEST_SERVICE_CODE_PRODUCT_SOURCE 下头一份 .rs、登的 .rs 所在目录里头一份没登也不在任何登的 .rs 拆分族里的 .rs）。
    弄坏开关 test-service-code-split-families-unrecognized 下子模块判成「不是」，这一格必红。"""
    rows = test_service_code_rows()
    listed_sources = [row for row in rows if row.endswith(".rs")]
    members, siblings = [], []
    for row in listed_sources:
        family = row[:-len(".rs")]
        if os.path.isdir(os.path.join(LIBRARY_ROOT, family)):
            members += files_under(LIBRARY_ROOT, (".rs",), [family])
        directory = posixpath.dirname(row)
        siblings += [posixpath.join(directory, name) for name in sorted(os.listdir(os.path.join(LIBRARY_ROOT, directory)))
                     if name.endswith(".rs") and posixpath.join(directory, name) not in rows
                     and not any(split_family.in_family(source, posixpath.join(directory, name)) for source in listed_sources)]
    service = (files_under(LIBRARY_ROOT, (".rs",), ["crates/singlefs-unit-graph/src"])[:1] + repository_judge_kernels()[:1] + members[:1])
    other = files_under(LIBRARY_ROOT, (".rs",), [TEST_SERVICE_CODE_PRODUCT_SOURCE])[:1] + siblings[:1]
    if len(service) != 3 or len(other) != 2:
        return False, f"真仓里找不齐要判的几份：该判是的 {service}（要 3 份）、该判不是的 {other}（要 2 份）"
    wrong = [f"{path} 判成不是" for path in service if not is_test_service_code(path)]
    wrong += [f"{path} 判成是" for path in other if is_test_service_code(path)]
    return not wrong, f"判错的 {wrong}（要没有：is_test_service_code 认前缀、登的那份与 .py、.sh、.rs 的拆分族，.wgsl 只认登的那份）"


def with_test_service_code_rows(rows, judge):
    """自证期间临时把 rows 并进 TEST_SERVICE_CODE_ROWS（先读一遍登记表）再判，跑完还原。"""
    registered = dict(test_service_code_rows())
    TEST_SERVICE_CODE_ROWS.update(rows)
    try:
        return judge()
    finally:
        TEST_SERVICE_CODE_ROWS.clear()
        TEST_SERVICE_CODE_ROWS.update(registered)


def test_service_code_unkeyed_cell(work):
    """TEST_SERVICE_CODE_UNKEYED_CELL：夹具 TEST_SERVICE_CODE_FIXTURE 里 judge_on_the_card.rs 临时登成测试服务代码（TEST_SERVICE_CODE_FIXTURE_ROWS），
    入口 entry_point 的闭包里它与 product.rs 各一个单元；照 TEST_SERVICE_CODE_FIXTURE_EDITS 逐个改（累加），每一处改动前后各算一把 closure_key 比。
    弄坏开关 test-service-code-keyed 下测试服务代码的单元照进键，改它键变，这一格必红。"""
    base = os.path.join(work, "tree")
    write_tree(base, TEST_SERVICE_CODE_FIXTURE)

    def judge():
        got, want = {}, {}
        for path, old, new, expected in TEST_SERVICE_CODE_FIXTURE_EDITS:
            before, before_refused = fixture_key(base, TEST_SERVICE_CODE_FIXTURE_ENTRY)
            edit_file(base, path, old, new)
            after, after_refused = fixture_key(base, TEST_SERVICE_CODE_FIXTURE_ENTRY)
            if before is None or after is None:
                return False, f"改 {path} 前后拒算了：{sorted(before_refused | after_refused)}"
            got[path], want[path] = ("changed" if before != after else "same"), expected
        return got == want, f"改了键变没变 {got}（要 {want}：测试服务代码的单元不出 unit 行，产品代码照进）"
    return with_test_service_code_rows(TEST_SERVICE_CODE_FIXTURE_ROWS, judge)


def test_service_code_entry_unkeyed_cell(work):
    """TEST_SERVICE_CODE_ENTRY_UNKEYED_CELL：夹具 TEST_SERVICE_CODE_ENTRY_FIXTURE 里入口所在的 lib.rs 与 judge_on_the_card.rs 临时登成测试服务代码
    （TEST_SERVICE_CODE_ENTRY_FIXTURE_ROWS）。闭包相同的两个入口 entry_point、other_point 的键要不同（entry 行记着是哪一个）；entry_point 照
    TEST_SERVICE_CODE_ENTRY_FIXTURE_EDITS 逐个改（累加），每一处改动前后各算一把 closure_key 比：改入口的正文键不变，改产品代码键变；
    同一份里 #[cfg(test)] mod tests 的测试函数（TEST_SERVICE_CODE_ENTRY_FIXTURE_TEST_ENTRY）当入口，照 TEST_SERVICE_CODE_ENTRY_FIXTURE_TEST_EDIT 改它的正文，键变。
    弄坏开关 test-service-code-entry-keyed 下入口单元的指纹照进 entry 行，改入口正文键变，这一格必红；
    test-service-code-test-entry-unkeyed 下测试函数入口也只记是哪一个，改它的正文键不变，这一格必红。"""
    base = os.path.join(work, "tree")
    write_tree(base, TEST_SERVICE_CODE_ENTRY_FIXTURE)
    entry, other = TEST_SERVICE_CODE_ENTRY_FIXTURE_ENTRIES

    def judge():
        entry_key, other_key = fixture_key(base, entry)[0], fixture_key(base, other)[0]
        if entry_key is None or other_key is None or entry_key == other_key:
            return False, f"两个入口的键 {entry_key}、{other_key}（都要算得出、互不相同：entry 行记着是哪一个入口）"
        got, want = {}, {}
        for path, old, new, expected in TEST_SERVICE_CODE_ENTRY_FIXTURE_EDITS:
            before, before_refused = fixture_key(base, entry)
            edit_file(base, path, old, new)
            after, after_refused = fixture_key(base, entry)
            if before is None or after is None:
                return False, f"改 {path} 前后拒算了：{sorted(before_refused | after_refused)}"
            got[path], want[path] = ("changed" if before != after else "same"), expected
        if got != want:
            return False, f"改了键变没变 {got}（要 {want}：入口落在测试服务代码里，entry 行不记它的指纹；产品代码照进）"
        test_before, test_before_refused = fixture_key(base, TEST_SERVICE_CODE_ENTRY_FIXTURE_TEST_ENTRY)
        edit_file(base, *TEST_SERVICE_CODE_ENTRY_FIXTURE_TEST_EDIT)
        test_after, test_after_refused = fixture_key(base, TEST_SERVICE_CODE_ENTRY_FIXTURE_TEST_ENTRY)
        if test_before is None or test_after is None:
            return False, f"测试函数当入口拒算了：{sorted(test_before_refused | test_after_refused)}"
        tested = "changed" if test_before != test_after else "same"
        return tested == "changed", f"改测试函数自己的正文键 {tested}（要 changed：入口是测试，照记指纹）"
    return with_test_service_code_rows(TEST_SERVICE_CODE_ENTRY_FIXTURE_ROWS, judge)


def module_string_constants(tree):
    """一份模块的语法树 → {模块级只赋过一次的名字: 值}：值是字面量（ast.literal_eval 认得的），或 <这类名字>[<整数>]（取元组、列表里那一项）；
    别的写法不收（library_root_reads 照「折不出」记）。只读语法树、不 import 那份模块：算键的一方认不出按变量名取属性（getattr-variable）。"""
    values, assigned = {}, {}
    for statement in tree.body:
        if not (isinstance(statement, ast.Assign) and len(statement.targets) == 1 and isinstance(statement.targets[0], ast.Name)):
            continue
        name, value = statement.targets[0].id, statement.value
        assigned[name] = assigned.get(name, 0) + 1
        if (isinstance(value, ast.Subscript) and isinstance(value.value, ast.Name) and isinstance(value.slice, ast.Constant)
                and isinstance(value.slice.value, int) and isinstance(values.get(value.value.id), (tuple, list))
                and -len(values[value.value.id]) <= value.slice.value < len(values[value.value.id])):
            values[name] = values[value.value.id][value.slice.value]
            continue
        try:
            values[name] = ast.literal_eval(value)
        except (ValueError, TypeError, SyntaxError, MemoryError, RecursionError):  # 不是字面量：这个名字不收
            values.pop(name, None)
    return {name: value for name, value in values.items() if assigned[name] == 1}


def library_root_reads():
    """算键库各份（scripts/gate/lib_function_units*.py，不含名字以 lib_function_units_selftest 起头的自证模块与夹具）里 LIBRARY_ROOT 的每一处用法 → {键: [文件:行]}。
    os.path.join(LIBRARY_ROOT, 各段) 的各段都是字符串字面量、或那份文件里的模块级字符串常量（module_string_constants 照语法树取值）的，键是拼出的仓内路径；
    别的用法与各段折不出的，键是「<文件>::<所在函数名>」（模块级写 <module>）。"""
    directory = os.path.join(LIBRARY_ROOT, "scripts", "gate")
    reads = {}
    for name in sorted(os.listdir(directory)):
        if not (name.startswith("lib_function_units") and name.endswith(".py")) or name.startswith("lib_function_units_selftest"):
            continue
        relative = posixpath.join("scripts", "gate", name)
        with open(os.path.join(directory, name), encoding="utf-8") as handle:
            tree = ast.parse(handle.read(), relative)
        parents = {child: node for node in ast.walk(tree) for child in ast.iter_child_nodes(node)}
        constants = module_string_constants(tree)
        for node in ast.walk(tree):
            if not (isinstance(node, ast.Name) and node.id == "LIBRARY_ROOT" and isinstance(node.ctx, ast.Load)):
                continue
            joined = parents[node]
            key = None
            if isinstance(joined, ast.Call) and ast.unparse(joined.func) == "os.path.join" and joined.args[0] is node:
                parts = []
                for part in joined.args[1:]:
                    if isinstance(part, ast.Name):
                        parts.append(constants.get(part.id))
                    else:
                        parts.append(part.value if isinstance(part, ast.Constant) else None)
                if parts and all(isinstance(part, str) for part in parts):
                    key = posixpath.join(*parts)
            if key is None:
                function = node
                while function in parents and not isinstance(function, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    function = parents[function]
                key = f"{relative}::{function.name if isinstance(function, (ast.FunctionDef, ast.AsyncFunctionDef)) else '<module>'}"
            reads.setdefault(key, []).append(f"{relative}:{node.lineno}")
    return reads


def runtime_data_files_cell(work):
    """RUNTIME_DATA_FILES_CELL：RUNTIME_DATA_FILES（拷样本仓的一侧只用 ast 读，要写成字符串字面量的元组）要是字面量、各项是不带 .. 的相对路径且不重复；
    library_root_reads() 现扫出的每一处读法要么在 RUNTIME_DATA_FILES 里、要么在 RUNTIME_LIBRARY_ROOT_READS_UNLISTED 里（两边不同时登）；
    登的每一项仓里有文件、也被扫到读；不登的每一行也被扫到读。弄坏开关 runtime-data-files-short 下元组少最后一项，这一格必红。"""
    with open(os.path.join(LIBRARY_ROOT, "scripts", "gate", "lib_function_units_registry.py"), encoding="utf-8") as handle:
        assigned = [node.value for node in ast.parse(handle.read()).body
                    if isinstance(node, ast.Assign) and [target.id for target in node.targets if isinstance(target, ast.Name)] == ["RUNTIME_DATA_FILES"]]
    problems = []
    if len(assigned) != 1 or not isinstance(assigned[0], ast.Tuple) or not all(isinstance(item, ast.Constant) and isinstance(item.value, str) for item in assigned[0].elts):
        problems.append("lib_function_units_registry.py 里 RUNTIME_DATA_FILES 不是恰好一处、值写成字符串字面量的元组")
    elif tuple(item.value for item in assigned[0].elts) != tuple(RUNTIME_DATA_FILES):
        problems.append("RUNTIME_DATA_FILES 载入的值与源码里的字面量不同")
    listed = list(RUNTIME_DATA_FILES)
    if "runtime-data-files-short" in active_breaks():
        listed = listed[:-1]
    reads = library_root_reads()
    unregistered = sorted(set(reads) - set(listed) - set(RUNTIME_LIBRARY_ROOT_READS_UNLISTED))
    for label, found in (("RUNTIME_DATA_FILES 登了两份相同的项", sorted({item for item in listed if listed.count(item) > 1})),
                         ("RUNTIME_DATA_FILES 的项不是不带 .. 的相对路径", [item for item in listed if item.startswith("/") or ".." in item.split("/")]),
                         ("两张表都登了", sorted(set(listed) & set(RUNTIME_LIBRARY_ROOT_READS_UNLISTED))),
                         ("RUNTIME_DATA_FILES 的项仓里没有文件", [item for item in listed if not os.path.isfile(os.path.join(LIBRARY_ROOT, item))]),
                         ("RUNTIME_DATA_FILES 的项没被算键库读到", [item for item in listed if item not in reads]),
                         ("RUNTIME_LIBRARY_ROOT_READS_UNLISTED 的行没被算键库读到", sorted(set(RUNTIME_LIBRARY_ROOT_READS_UNLISTED) - set(reads))),
                         ("算键库读了、两张表都没登的", [f"{key}（{'、'.join(reads[key])}）" for key in unregistered])):
        if found:
            problems.append(f"{label} {found}")
    return bool(reads) and not problems, (f"现扫到 {len(reads)} 种 LIBRARY_ROOT 读法，RUNTIME_DATA_FILES 登了 {len(listed)} 项：{'；'.join(problems) or '一处也没扫到'}"
                                          "（算键必读的数据文件登进 RUNTIME_DATA_FILES，别的写进 RUNTIME_LIBRARY_ROOT_READS_UNLISTED 并写理由）")
