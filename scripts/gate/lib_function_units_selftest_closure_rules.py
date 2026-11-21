"""自证的 Python 与 shell 闭包规则格：按角色排除的文件、按路径载入的模块属性、工具链询问、程序入口带参数、shell 入口与表、preflight 装载器、键的工具链与外部程序。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import os
import shutil
import subprocess
import sys
import lib_function_units_script_closure
from lib_function_units_lexer import active_breaks
from lib_function_units_base import FunctionUnitsError, LIBRARY_ROOT, UsageError
from lib_function_units_registry import NOT_JUDGING_FILES
from lib_function_units_python_facts import TOOLCHAIN_UNIT_KIND
from lib_function_units_rust_workspace import discover_rust_crates
from lib_function_units_script_closure import Options, SHARED_SCRIPT_INDICES, closure_of, command_closure, parse_options
from lib_function_units_closure_keys import closure_key, edit_file, row_keyed_closure, write_tree
from lib_function_units_selftest_fixture_trees import (
    SHELL_ENTRY_EXIT_TABLE_FIXTURE, SHELL_ENTRY_FIXTURE, SHELL_TABLE_FIXTURE, SHELL_TABLE_TREE_COPY_FIXTURE, TAIL_VIEW_FIXTURE, fixture_key,
    module_literal, prepare_shared_script_index,
)


def own_program_tree(work, label, files):
    base = os.path.join(work, label)
    write_tree(base, {OWN_PROGRAM_MANIFEST_PATH: OWN_PROGRAM_MANIFEST, **files})
    return base


def own_program_source(restart_function, after=""):
    """main 分两个角色：带参数的那个角色跑 role_body，不带参数的起自己（起自己的写法由 restart_function 给）。"""
    return ("fn role_body() -> u32 {\n    1\n}\n" + restart_function
            + "\nfn main() {\n    if std::env::args().count() > 1 {\n        role_body();\n    } else {\n        restart();\n    }\n}\n" + after)


def own_program_keyed_cell(work):
    """OWN_PROGRAM_KEYED_CELL：四种写法从 main 算得出键，改 role_body 键变；库函数里起自己、从 bin 的 main 连到的算得出键。"""
    problems = []
    for label, restart in OWN_PROGRAM_RESTARTS_KEYED.items():
        base = own_program_tree(work, label, {OWN_PROGRAM_MAIN_FILE: own_program_source(restart)})
        kept, forms = fixture_key(base, OWN_PROGRAM_MAIN_ENTRY)
        edit_file(base, OWN_PROGRAM_MAIN_FILE, "fn role_body() -> u32 {\n    1", "fn role_body() -> u32 {\n    2")
        after = fixture_key(base, OWN_PROGRAM_MAIN_ENTRY)[0]
        if kept is None or after is None or kept == after:
            problems.append(f"{label}：改前键{'有' if kept else '无'}（拒算 {sorted(forms)}），改后键{'有' if after else '无'}，变了：{kept != after}")
    base = own_program_tree(work, "library", {OWN_PROGRAM_LIBRARY_FILE: OWN_PROGRAM_LIBRARY_RESTART, OWN_PROGRAM_MAIN_FILE: "fn main() {\n    frestart::restart_from_library();\n}\n"})
    kept, forms = fixture_key(base, OWN_PROGRAM_MAIN_ENTRY)
    if kept is None:
        problems.append(f"library：拒算 {sorted(forms)}")
    return not problems, f"该算得出键、改角色函数键变的没做到：{problems}"


def own_program_entry_cell(work):
    """OWN_PROGRAM_ENTRY_CELL：同一种起自己的写法，入口不是 bin 的 main 的每一种都拒算 current-exe。"""
    restart = OWN_PROGRAM_RESTARTS_KEYED["command-new"]
    compiled_as_test = Options()
    compiled_as_test.cfg = ["test"]
    test_module = "#[cfg(test)]\nmod tests {\n    #[test]\n    fn it_restarts() {\n        super::restart();\n    }\n    #[test]\n    fn it_calls_main() {\n        super::main();\n    }\n}\n"
    library = OWN_PROGRAM_LIBRARY_RESTART + "#[cfg(test)]\nmod tests {\n    #[test]\n    fn it_restarts() {\n        super::restart_from_library();\n    }\n}\n"
    module_main = "mod inner {\n    pub fn main() {\n        super::restart();\n    }\n}\n"
    cases = (
        ("library-test", {OWN_PROGRAM_LIBRARY_FILE: library}, OWN_PROGRAM_LIBRARY_FILE + "::tests::it_restarts", None),
        ("bin-test", {OWN_PROGRAM_MAIN_FILE: own_program_source(restart, test_module)}, OWN_PROGRAM_MAIN_FILE + "::tests::it_restarts", None),
        ("bin-test-calling-main", {OWN_PROGRAM_MAIN_FILE: own_program_source(restart, test_module)}, OWN_PROGRAM_MAIN_FILE + "::tests::it_calls_main", None),
        ("main-compiled-as-test", {OWN_PROGRAM_MAIN_FILE: own_program_source(restart)}, OWN_PROGRAM_MAIN_ENTRY, compiled_as_test),
        ("helper-entry", {OWN_PROGRAM_MAIN_FILE: own_program_source(restart)}, OWN_PROGRAM_MAIN_FILE + "::restart", None),
        ("module-main", {OWN_PROGRAM_MAIN_FILE: own_program_source(restart, module_main)}, OWN_PROGRAM_MAIN_FILE + "::inner::main", None),
    )
    problems = []
    for label, files, entry, options in cases:
        key, forms = fixture_key(own_program_tree(work, label, files), entry, options)
        if key is not None or "current-exe" not in forms:
            problems.append(f"{label}：键{'有' if key else '无'}，拒算 {sorted(forms)}")
    return not problems, f"该拒算 current-exe 的没拒算：{problems}"


def own_program_refused_cell(restarts):
    """restarts（{标签: 起自己的函数}）里每一种从 bin 的 main 起都拒算 current-exe。"""
    def judge(work):
        problems = []
        for label, restart in restarts.items():
            key, forms = fixture_key(own_program_tree(work, label, {OWN_PROGRAM_MAIN_FILE: own_program_source(restart)}), OWN_PROGRAM_MAIN_ENTRY)
            if key is not None or "current-exe" not in forms:
                problems.append(f"{label}：键{'有' if key else '无'}，拒算 {sorted(forms)}")
        return not problems, f"该拒算 current-exe 的没拒算：{problems}"
    return judge


def own_program_path_cell(work):
    """OWN_PROGRAM_PATH_CELL：入口是 bin 的 main，路径改过或流向别处的每一种都拒算 current-exe。"""
    return own_program_refused_cell(OWN_PROGRAM_RESTARTS_REFUSED)(work)


def extern_block_tree(work, label, library, extra_files=None):
    base = os.path.join(work, label)
    write_tree(base, {"crates/fextern/Cargo.toml": '[package]\nname = "fextern"\n', EXTERN_BLOCK_FILE: library, **(extra_files or {})})
    return base


def extern_listed_cell(work):
    """EXTERN_LISTED_CELL：#[link_name] 指到名单里的两个符号的算得出键、改调用方键变；函数名就是名单里的符号的算得出键。"""
    base = extern_block_tree(work, "link-names", EXTERN_LISTED_LINK_NAMES)
    kept, forms = fixture_key(base, EXTERN_BLOCK_ENTRY)
    edit_file(base, EXTERN_BLOCK_FILE, "fdatasync_raw(3)", "fdatasync_raw(4)")
    after = fixture_key(base, EXTERN_BLOCK_ENTRY)[0]
    own_name, own_name_forms = fixture_key(extern_block_tree(work, "own-name", EXTERN_LISTED_OWN_NAME), EXTERN_BLOCK_ENTRY)
    return kept is not None and after is not None and kept != after and own_name is not None, (
        f"link_name：改前键{'有' if kept else '无'}（拒算 {sorted(forms)}），改调用方后键变：{kept != after}；函数名：键{'有' if own_name else '无'}（拒算 {sorted(own_name_forms)}）")


def extern_refused_cell(work):
    """EXTERN_REFUSED_CELL：EXTERN_BLOCKS_REFUSED 每一种都拒算 extern-block。"""
    problems = []
    for label, library in EXTERN_BLOCKS_REFUSED.items():
        key, forms = fixture_key(extern_block_tree(work, label, library), EXTERN_BLOCK_ENTRY)
        if key is not None or "extern-block" not in forms:
            problems.append(f"{label}：键{'有' if key else '无'}，拒算 {sorted(forms)}")
    return not problems, f"该拒算 extern-block 的没拒算：{problems}"


def extern_refused_cell_for(label, library, extra_files=None):
    """一份 extern 夹具（库文件 library，另有 extra_files）拒算 extern-block 的一格。"""
    def judge(work):
        key, forms = fixture_key(extern_block_tree(work, label, library, extra_files), EXTERN_BLOCK_ENTRY)
        return key is None and "extern-block" in forms, f"该拒算 extern-block：键{'有' if key else '无'}，拒算 {sorted(forms)}"
    return judge


def extern_build_script_link_cell(work):
    """EXTERN_BUILD_SCRIPT_LINK_CELL：名单里的块（EXTERN_LISTED_OWN_NAME）所在的包，构建脚本写了 rustc-link-lib 的拒算 extern-block，写别的指令的算得出键。"""
    linking_key, linking_forms = fixture_key(extern_block_tree(work, "build-script-links", EXTERN_LISTED_OWN_NAME, EXTERN_BUILD_SCRIPT_LINKING), EXTERN_BLOCK_ENTRY)
    quiet_key, quiet_forms = fixture_key(extern_block_tree(work, "build-script-quiet", EXTERN_LISTED_OWN_NAME, EXTERN_BUILD_SCRIPT_QUIET), EXTERN_BLOCK_ENTRY)
    return linking_key is None and "extern-block" in linking_forms and quiet_key is not None, (
        f"写 rustc-link-lib 的：键{'有' if linking_key else '无'}、拒算 {sorted(linking_forms)}（要无、要有 extern-block）；"
        f"不写的：键{'有' if quiet_key else '无'}、拒算 {sorted(quiet_forms)}（要有）")


def workspace_dot_member_cell(work):
    """WORKSPACE_DOT_MEMBER_CELL：[workspace] members = ["."] 且根清单没有 [package]：根收成叫 root 的 crate、入口 root_value 算得出键；有 [package]：只收一个 crate（名字是 [package] 写的）。"""
    problems = []
    base = os.path.join(work, "dot-unnamed")
    write_tree(base, WORKSPACE_DOT_UNNAMED_FIXTURE)
    try:
        key, forms = fixture_key(base, "src/lib.rs::root_value")
        if key is None:
            problems.append(f"没有 [package] 的根：入口拒算 {sorted(forms)}")
    except (FunctionUnitsError, UsageError) as error:
        problems.append(f"没有 [package] 的根：入口找不到（{error}）")
    base = os.path.join(work, "dot-named")
    write_tree(base, WORKSPACE_DOT_NAMED_FIXTURE)
    packages = [crate.package for crate in discover_rust_crates(base)]
    if packages != ["rootpkg"]:
        problems.append(f"有 [package] 的根：收到 {packages}（要 ['rootpkg']）")
    return not problems, "；".join(problems)


def workspace_multiline_members_cell(work):
    """WORKSPACE_MULTILINE_CELL：members 里的成员写成多行基础串：开头的换行去掉、照收（WORKSPACE_MULTILINE_PLAIN_FIXTURE 收到 alpha）；
    串里有反斜杠（行尾反斜杠连行，WORKSPACE_MULTILINE_BACKSLASH_FIXTURE）的 discover_rust_crates 抛 FunctionUnitsError。弄坏开关 rust-workspace-multiline-backslash-read 下这一格必须判错。"""
    problems = []
    base = os.path.join(work, "plain")
    write_tree(base, WORKSPACE_MULTILINE_PLAIN_FIXTURE)
    packages = [crate.package for crate in discover_rust_crates(base)]
    if packages != ["alpha"]:
        problems.append(f"没有反斜杠的多行基础串：收到 {packages}（要 ['alpha']）")
    base = os.path.join(work, "backslash")
    write_tree(base, WORKSPACE_MULTILINE_BACKSLASH_FIXTURE)
    try:
        packages = [crate.package for crate in discover_rust_crates(base)]
        problems.append(f"含反斜杠的多行基础串：没报读不出，收到 {packages}")
    except FunctionUnitsError:
        pass
    return not problems, "；".join(problems)


def workspace_root_with_brackets_cell(work):
    """WORKSPACE_BRACKET_CELL：仓根路径里有 [ ]（目录名 root[with]brackets）时，members = ["crates/*"] 照样展开出 alpha。"""
    base = os.path.join(work, "root[with]brackets")
    write_tree(base, WORKSPACE_MEMBERS_FIXTURE)
    packages = [crate.package for crate in discover_rust_crates(base)]
    return packages == ["alpha"], f"成员展开出 {packages}（要 ['alpha']）"


def workspace_same_named_directories_cell(work):
    """WORKSPACE_SAME_NAME_CELL：crates/common 与 tools/common 各有 tests/smoke.rs，两个测试入口都算得出键（目标号不撞：撞了，后一个目标盖掉前一个，前一个入口找不到目标）。
    键只由闭包里各单元的内容定、不带目标号、路径与名字，所以两个 smoke.rs 各调自己包的库函数 value（夹具里一个返回 1、一个返回 2），闭包的内容不同、键才不同。"""
    base = os.path.join(work, "tree")
    write_tree(base, WORKSPACE_SAME_NAME_FIXTURE)
    keys, problems = {}, []
    for entry in ("crates/common/tests/smoke.rs::smoke", "tools/common/tests/smoke.rs::smoke"):
        try:
            key, forms = fixture_key(base, entry)
        except (FunctionUnitsError, UsageError) as error:
            problems.append(f"{entry}：{error}")
            continue
        keys[entry] = key
        if key is None:
            problems.append(f"{entry}：拒算 {sorted(forms)}")
    if not problems and len(set(keys.values())) != 2:
        problems.append(f"两个入口的键相同：{keys}")
    return not problems, "；".join(problems)


def extern_link_beyond_cell(refused, keyed):
    """名单里的块（EXTERN_LISTED_OWN_NAME，包 fextern）在 refused 的每一棵树（{标签: 另加的文件}）里拒算 extern-block、在 keyed 的每一棵树里算得出键。"""
    def judge(work):
        problems = []
        for label, files in refused.items():
            key, forms = fixture_key(extern_block_tree(work, "refused-" + label, EXTERN_LISTED_OWN_NAME, files), EXTERN_BLOCK_ENTRY)
            if key is not None or "extern-block" not in forms:
                problems.append(f"{label}：键{'有' if key else '无'}、拒算 {sorted(forms)}（要无、要有 extern-block）")
        for label, files in keyed.items():
            key, forms = fixture_key(extern_block_tree(work, "keyed-" + label, EXTERN_LISTED_OWN_NAME, files), EXTERN_BLOCK_ENTRY)
            if key is None:
                problems.append(f"{label}：键无、拒算 {sorted(forms)}（要有）")
        return not problems, "；".join(problems)
    return judge


def no_refusal_cell(fixture, entry):
    def judge(work):
        base = os.path.join(work, "tree")
        write_tree(base, fixture)
        key, forms = fixture_key(base, entry)
        return key is not None, f"不该拒算，实际拒算 {sorted(forms)}"
    return judge


def no_external_program_cell(fixture, entry):
    """不拒算，闭包也不记外部程序（Closure.programs 空着）：版本不进键之后外部程序只记不拒算，判这一种认法要看记下的程序名
    （scripts/crash/lib_verdict_library.py 的 FunctionUnitIndex.key 把它们收成 external 行进判法键），不看拒算。"""
    def judge(work):
        base = os.path.join(work, "tree")
        write_tree(base, fixture)
        closure = closure_of(base, entry)
        key, refusals = closure_key(closure)
        return key is not None and not closure.programs, (f"不该拒算、不该记外部程序，实际拒算 {sorted(refusal.form for refusal in refusals)}、"
                                                          f"记的外部程序 {sorted(closure.programs)}")
    return judge


def no_refusal_no_toolchain_query_cell(fixture, entry):
    """不拒算，也不落到「版本询问」那一条（文件头）：程序是认得出的根外程序或折出的解释器，工具链单元不进键。"""
    def judge(work):
        base = os.path.join(work, "tree")
        write_tree(base, fixture)
        closure = closure_of(base, entry)
        key, refusals = closure_key(closure)
        return key is not None and not closure.toolchain_queries, (f"不该拒算、不该当工具链询问，实际拒算 {sorted(refusal.form for refusal in refusals)}、"
                                                                   f"工具链询问 {sorted(closure.toolchain_queries)}")
    return judge


def launch_argument_cell(work):
    """LAUNCH_ARGUMENT_CELL：check 的闭包不拒算命令参数；字面的 cargo 是登记的外部程序，只记、不拒算（文件头「版本不进键」），不给 externals 也算得出键。"""
    base = os.path.join(work, "tree")
    write_tree(base, LAUNCH_ARGUMENT_FIXTURE)
    key, refusals = closure_key(closure_of(base, "pkg/launch.py::check"))
    return key is not None, f"不该拒算，实际拒算 {sorted(refusal.form for refusal in refusals)}"


def tail_view_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, TAIL_VIEW_FIXTURE)

    def key_now():
        options = Options()
        options.program_arguments = ["--selftest"]
        return fixture_key(base, "pkg/packages.py", options)
    before, before_forms = key_now()
    edit_file(base, "elsewhere/Manifest.toml", "real", "changed")
    after, after_forms = key_now()
    return before is not None and before == after, f"键改前 {before}（拒算 {sorted(before_forms)}）、改后 {after}（拒算 {sorted(after_forms)}）"


def closure_round_limit_cell(work):
    """CLOSURE_ROUND_LIMIT_CELL：check 要收两轮；上限临时压到 1，closure_of 抛 FunctionUnitsError，话里点名入口。"""
    base = os.path.join(work, "tree")
    write_tree(base, CLOSURE_ROUNDS_FIXTURE)
    limit = lib_function_units_script_closure.CLOSURE_ROUND_LIMIT
    lib_function_units_script_closure.CLOSURE_ROUND_LIMIT = 1
    try:
        closure_of(base, "pkg/launch.py::check")
    except FunctionUnitsError as error:
        return "pkg/launch.py::check" in str(error), f"抛了，话里没点名入口：{error}"
    finally:
        lib_function_units_script_closure.CLOSURE_ROUND_LIMIT = limit
    return False, "上限压到 1，闭包照样收完，没有抛"


def shared_script_index_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, {"scripts/one.py": "def entry():\n    return 1\n"})
    prepared, key = prepare_shared_script_index(base)
    try:
        closure = closure_of(base, "scripts/one.py::entry")
        shared = any(getattr(unit, "module", None) is prepared.python.module("scripts/one.py") for unit in closure.order)
        return shared, f"闭包里的单元不是父进程建好的那份索引里的（闭包 {len(closure.order)} 个单元）"
    finally:
        SHARED_SCRIPT_INDICES.pop(key, None)


def shell_table_copy_cell(work):
    """SHELL_TABLE_COPY_CELL：闭包成员，不靠键（复跑表脚本的字节经副本那一处读进键，键变不变分不出函数连没连）。"""
    base = os.path.join(work, "tree")
    write_tree(base, SHELL_TABLE_COPY_FIXTURE)
    closure = closure_of(base, "scripts/gate/copy_launcher.py::judge")
    key, refusals = closure_key(closure)
    names = {unit.name for unit in closure.order if unit.file == SHELL_TABLE_COPY_RUNNER}
    files = {unit.file for unit in closure.order}
    table_only = [name for name in ("driver_row_only", "driver_deferred", "driver_compared", "driver_replaced", "driver_shared") if name in names]
    table_only += sorted(files & {"s/row_only.sh", "s/deferred.sh", "s/compared.sh", "s/replaced.sh", "s/shared.sh"})
    left_behind = [name for name in ("<script-body>", "framework_judge", "framework_helper", "DEFERRED_ROOT") if name not in names]
    return key is not None and not table_only and not left_behind, (f"只被真表点名的进了闭包 {table_only}；副本里留下的没进闭包 {left_behind}；"
                                                                     f"拒算 {sorted(refusal.form for refusal in refusals)}")


def shell_table_real_cell(_work):
    with open(os.path.join(LIBRARY_ROOT, "scripts/experiment-apparatus/replay.sh"), encoding="utf-8") as handle:
        text = handle.read()
    wanted = ("TABLE=$(cat <<'TSV'\n", "ADMISSION_ROW_KEYS=$(cat <<'TSV'\n", 'ONLY+=("$replay_argument")', "want() {",
              'elif [[ "$replay_argument" == "--item" ]]; then replay_expecting_item=1', '[[ "$exp" == E* ]] || continue',
              'ROW_ORDINAL["$exp|$bin|$args"]="${replay_ordinal_of_experiment["$exp"]}"',
              '[[ "${replay_identity%%|*}" == "${replay_item%.*}" && "${ROW_ORDINAL["$replay_identity"]}" == "${replay_item##*.}" ]]')
    missing = [item.strip() for item in wanted if item not in text]
    return not missing, f"replay.sh 里找不到 {missing}：选行的写法变了，看 SHELL_ROW_SELECTED_TABLES 与 rows_of_selected_keys 还对不对"


def excluded_file_cell(path, old, new, want, excluded):
    """EXCLUDED_FILE_*_CELL：pkg/judge.py::judge 的键，排除 excluded 那几份（文件头「按角色排除的文件」），改 path 里的 old 之后键变不变。"""
    def judge(work):
        base = os.path.join(work, "tree")
        write_tree(base, EXCLUDED_FILE_FIXTURE)
        options = Options()
        options.excluded_files = excluded
        before, before_refused = fixture_key(base, "pkg/judge.py::judge", options)
        edit_file(base, path, old, new)
        after, after_refused = fixture_key(base, "pkg/judge.py::judge", options)
        if before is None or after is None:
            return False, f"拒算了：改前 {sorted(before_refused)}，改后 {sorted(after_refused)}"
        got = "changed" if before != after else "same"
        return got == want, f"期望 {want}，实际 {got}"
    return judge


def not_judging_files_cell(_work):
    recorded = tuple(module_literal(LIBRARY_ROOT, "scripts/crash/lib_verdict_library.py", "NOT_JUDGING_FILES"))
    copied = NOT_JUDGING_FILES[:-1] if "not-judging-files-drifted" in active_breaks() else NOT_JUDGING_FILES
    return copied == recorded, f"这份抄的 {copied}，lib_verdict_library.py 那一份 {recorded}：两处一起改"


def judging_closure_cell(work):
    """JUDGING_CLOSURE_CELL：命令行 closure 的收法（parse_options、command_closure）在夹具上带与不带 --judging 各收一遍 pkg/entry.py，
    另带 --judging 收入口就是那份排除文件的；比闭包里有哪几份文件，三趟都不许拒算。"""
    base = os.path.join(work, "tree")
    write_tree(base, JUDGING_CLOSURE_FIXTURE)
    role, direct, only = "scripts/gate/lib_function_units.py", "pkg/direct.py", "pkg/only_via_role.py"

    def collected(entry, arguments):
        options, _positional = parse_options(arguments)
        closure = command_closure(base, entry, options)
        return {unit.file for unit in closure.order}, sorted(refusal.text() for refusal in closure.refusals)

    judging, judging_refused = collected("pkg/entry.py", ["--judging"])
    whole, whole_refused = collected("pkg/entry.py", [])
    role_entry, role_refused = collected(role, ["--judging"])
    problems = []
    if direct not in judging or role in judging or only in judging:
        problems.append(f"带 --judging 收 pkg/entry.py：要有 {direct}、不要 {role} 与 {only}，实际 {sorted(judging)}")
    if not {direct, role, only} <= whole:
        problems.append(f"不带 --judging 收 pkg/entry.py：{direct}、{role}、{only} 都要在，实际 {sorted(whole)}")
    if only not in role_entry:
        problems.append(f"带 --judging 收 {role}（入口就在排除文件里，照 admission.py 不排）：要有 {only}，实际 {sorted(role_entry)}")
    if judging_refused or whole_refused or role_refused:
        problems.append(f"夹具拒算了：{judging_refused + whole_refused + role_refused}")
    return not problems, "；".join(problems) or "带 --judging 排掉只经排除文件起到的那一份与排除文件自己，不带的照收，入口在排除文件里的不排"


def loaded_attributes_cell(work):
    """LOADED_ATTRIBUTES_CELL（文件头「按路径载入的模块只连载入方调到的属性」）：夹具上四个入口各收一遍，看 pkg/library.py 的哪几个单元进了闭包。
    值不出载入方的两种（pkg/loader.py 函数里的局部名字；s/run.sh 内联脚本里放进模块级容器再取）只许有 wanted 与 import 时执行的、不许有 main、
    main-guard 与拒算；交给调用方的（pkg/escaping.py）与以 __main__ 名载入的（pkg/as_main.py）照整个闭包连，要有 main。"""
    base = os.path.join(work, "tree")
    write_tree(base, LOADED_ATTRIBUTES_FIXTURE)
    library = "pkg/library.py"

    def collected(entry):
        closure = closure_of(base, entry)
        return {unit.name for unit in closure.order if unit.file == library}, sorted(refusal.text() for refusal in closure.refusals)

    problems = []
    for entry in ("pkg/loader.py::entry", "s/run.sh"):
        names, refused = collected(entry)
        if not {"wanted", "<module-body>"} <= names or names & {"main", "<main-guard>"} or refused:
            problems.append(f"{entry}（值不出载入方）：要有 wanted 与 <module-body>、不要 main 与 <main-guard>、不拒算，实际 {sorted(names)}，拒算 {refused}")
    for entry in ("pkg/escaping.py::entry", "pkg/as_main.py::entry"):
        names, _refused = collected(entry)
        if not {"wanted", "main"} <= names:
            problems.append(f"{entry}（值交给调用方、以 __main__ 名载入）：照整个闭包连，要有 wanted 与 main，实际 {sorted(names)}")
    return not problems, "；".join(problems) or "值不出载入方的只连调到的 wanted，main 不跟着别处的 x.main() 进来；交出去的与 __main__ 名的照整个闭包连"



def toolchain_query_cell(work):
    """TOOLCHAIN_QUERY_CELL（文件头「版本询问」「版本不进键」）：PATH 最前面放假 rustc、cargo（打 work 里一份版本文件的内容），compiler_version 起
    [os.environ["RUSTC"], "-V"]：闭包记了工具链询问、不拒算、算得出键、没有工具链单元；版本文件换一版，compiler_version 与闭包里没有询问的 judge_file 键都不变。"""
    base = os.path.join(work, "tree")
    write_tree(base, FOLDS_FIXTURE)
    tools, version = os.path.join(work, "fake-toolchain"), os.path.join(work, "fake-toolchain-version")
    os.makedirs(tools)
    for name in ("rustc", "cargo"):
        with open(os.path.join(tools, name), "w", encoding="utf-8") as handle:
            handle.write(FAKE_TOOLCHAIN_SCRIPT.format(name=name, version=version))
        os.chmod(os.path.join(tools, name), 0o755)
    patched = {"PATH": tools + os.pathsep + os.environ.get("PATH", ""), "CARGO_HOME": os.path.join(work, "cargo-home")}
    saved = {name: os.environ.get(name) for name in patched}

    def keys(release):
        with open(version, "w", encoding="utf-8") as handle:
            handle.write(release + "\n")
        queried = closure_of(base, "pkg/folds.py::compiler_version")
        return queried, closure_key(queried), closure_key(closure_of(base, "pkg/folds.py::judge_file"))[0]
    try:
        os.environ.update(patched)
        queried, (before, refusals), plain_before = keys("1.0.0-selftest")
        _queried, (after, _refusals), plain_after = keys("1.1.0-selftest")
    finally:
        for name, value in saved.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
    toolchain_units = [unit for unit in queried.order if unit.kind == TOOLCHAIN_UNIT_KIND]
    return (bool(queried.toolchain_queries) and not toolchain_units and not refusals and None not in (before, after) and before == after
            and plain_before is not None and plain_before == plain_after,
            f"compiler_version：工具链单元 {len(toolchain_units)} 个、拒算 {sorted({refusal.form for refusal in refusals})}、换版本前 {before}、后 {after}；"
            f"judge_file 换版本前 {plain_before}、后 {plain_after}")


def caller_launch_cell(work):
    """CALLER_LAUNCH_CELL（文件头「调用方交进来的命令」）：run_dynamic(code)、run_unknown(interpreter) 当判法入口，-c 的串与解释器是入口的参数，不拒算
    （同一份夹具里经环境变量调它们的那两条照旧拒算，另两格判）。"""
    base = os.path.join(work, "tree")
    write_tree(base, PYTHON_FIXTURE)
    found = {entry: fixture_key(base, entry) for entry in ("pkg/m.py::run_dynamic", "pkg/m.py::run_unknown")}
    refused = [f"{entry} 拒算 {sorted(forms)}" for entry, (key, forms) in found.items() if key is None]
    return not refused, "；".join(refused)



def registry_programs_cell(work):
    """REGISTRY_PROGRAMS_CELL：check_all 读登记表、照每一行调 run_probe（起探针的那一处登在 REGISTRY_LISTED_PROGRAMS 里）：
    键算得出（不拒算），改登记表点名的 probe_beta.sh 之后键不变（探针不连）。开关 python-registry-programs-ignored 下拒算，python-registry-programs-linked 下键变。"""
    base = os.path.join(work, "tree")
    write_tree(base, REGISTRY_FIXTURE)
    before = fixture_key(base, "pkg/probes.py::check_all")
    edit_file(base, "pkg/probe_beta.sh", "echo beta", "echo beta again")
    after = fixture_key(base, "pkg/probes.py::check_all")
    ok = before[0] is not None and after[0] == before[0]
    return ok, f"改前键 {before[0]}（拒算 {before[1]}）；改探针之后键 {after[0]}（拒算 {after[1]}）"



def row_keyed_cell(work):
    """ROW_KEYED_CELL：judge（经 registered_selftest 查 scripts/a.py 那一行）的键照 row_keyed_closure 按入口 scripts/a.py 换过：改 scripts/b.py 那一行、
    改样本档行键不变，改 scripts/a.py 那一行键变；audit（读整张表、不在登记里）改 scripts/b.py 那一行键就变。"""
    base = os.path.join(work, "tree")
    write_tree(base, ROW_KEYED_FIXTURE)

    def keys():
        out = {}
        for entry in ("judge", "audit"):
            closure = closure_of(base, f"pkg/guard.py::{entry}", Options())
            out[entry] = closure_key(row_keyed_closure(closure, base, "scripts/a.py"))[0]
        return out
    before = keys()
    edit_file(base, ".claude/hooks/selftest-registry.tsv", "# 乙", "# 乙改")
    unrelated = keys()
    edit_file(base, ".claude/hooks/selftest-registry.tsv", "samples:tools", "samples:other")
    sample_row = keys()
    edit_file(base, ".claude/hooks/selftest-registry.tsv", "# 甲", "# 甲改")
    own_row = keys()
    ok = (None not in before.values() and unrelated["judge"] == before["judge"] and sample_row["judge"] == before["judge"]
          and own_row["judge"] not in (None, before["judge"]) and unrelated["audit"] not in (None, before["audit"]))
    return ok, f"改前 {before}；改无关的一行之后 {unrelated}；改样本档行之后 {sample_row}；改它查的那一行之后 {own_row}"


# 程序入口带参数（文件头「程序入口带参数」）那一格的夹具：dispatch.py 按 argv 分派（--selftest 一支、分派表一支），launcher.py 带着 alpha 起它
ARGUMENTS_FIXTURE = {
    "tool/dispatch.py": '''import sys


def alpha_work():
    return 1


def beta_work():
    return 2


def run_selftest():
    return alpha_work() + beta_work()


COMMANDS = {"alpha": alpha_work, "beta": beta_work}


def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return run_selftest()
    if not arguments or arguments[0] not in COMMANDS:
        print("用法：dispatch.py alpha|beta|--selftest")
        return 9
    return COMMANDS[arguments[0]]()


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
''',
    "tool/launcher.py": '''import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))


def main():
    return subprocess.run([sys.executable, os.path.join(HERE, "dispatch.py"), "alpha"], check=False).returncode


if __name__ == "__main__":
    sys.exit(main())
''',
}


def program_arguments_cell(work):
    """PROGRAM_ARGUMENTS_CELL：dispatch.py 带 alpha、带 --selftest 与 launcher.py（起 dispatch.py alpha）三条：改 beta_work，alpha 与 launcher 的键不变、
    --selftest 的变；再改 alpha_work，alpha 与 launcher 的键变。"""
    base = os.path.join(work, "tree")
    write_tree(base, ARGUMENTS_FIXTURE)

    def keys():
        out = {}
        for label, entry, arguments in (("alpha", "tool/dispatch.py", ["alpha"]), ("selftest", "tool/dispatch.py", ["--selftest"]),
                                        ("launcher", "tool/launcher.py", ["run"])):
            options = Options()
            options.program_arguments = arguments
            out[label] = fixture_key(base, entry, options)[0]
        return out
    before = keys()
    edit_file(base, "tool/dispatch.py", "    return 2\n", "    return 20\n")
    beta_changed = keys()
    edit_file(base, "tool/dispatch.py", "    return 1\n", "    return 10\n")
    alpha_changed = keys()
    ok = (None not in before.values() and beta_changed["alpha"] == before["alpha"] and beta_changed["launcher"] == before["launcher"]
          and beta_changed["selftest"] not in (None, before["selftest"]) and alpha_changed["alpha"] not in (None, beta_changed["alpha"])
          and alpha_changed["launcher"] not in (None, beta_changed["launcher"]))
    return ok, f"改前 {before}；改 beta_work 之后 {beta_changed}；改 alpha_work 之后 {alpha_changed}"


def entry_arguments_key(base, entry, arguments):
    """入口带 arguments（Options.program_arguments）算的 (键或 None, 拒算形态的集合)。"""
    options = Options()
    options.program_arguments = list(arguments)
    return fixture_key(base, entry, options)


def shell_entry_edits_cell(entry, arguments, edits, fixture=SHELL_ENTRY_FIXTURE):
    """SHELL_ENTRY_ARGUMENTS_CELL、SHELL_ENTRY_UNDECIDED_CELL（SHELL_ENTRY_FIXTURE）、SHELL_TABLE_ITEM_CELL（SHELL_TABLE_ITEM_FIXTURE）：fixture 里 entry 带 arguments，
    edits 逐条 (路径, 旧, 新, 期望 same|changed) 改一处、同上一次的键比。"""
    def judge(work):
        base = os.path.join(work, "tree")
        write_tree(base, fixture)
        previous, previous_refused = entry_arguments_key(base, entry, arguments)
        outcomes = []
        for path, old, new, want in edits:
            edit_file(base, path, old, new)
            after, after_refused = entry_arguments_key(base, entry, arguments)
            if previous is None or after is None:
                return False, f"改 {path} 前后拒算了：{sorted(previous_refused)}、{sorted(after_refused)}"
            outcomes.append((path, want, "changed" if after != previous else "same"))
            previous = after
        return all(want == got for _path, want, got in outcomes), "；".join(f"改 {path} 期望 {want}、实际 {got}" for path, want, got in outcomes)
    return judge


def shell_entry_table_cell(work):
    """SHELL_ENTRY_TABLE_CELL：SHELL_TABLE_FIXTURE 的复跑表脚本当入口，带 E1 与带 --selftest 各算一把键；改 s/two.sh（只由 E2 那一行的驱动起），
    带 E1 的键不变、带 --selftest 的变。"""
    base = os.path.join(work, "tree")
    write_tree(base, SHELL_TABLE_FIXTURE)
    entry = "scripts/experiment-apparatus/replay.sh"

    def keys():
        return {label: entry_arguments_key(base, entry, arguments)[0] for label, arguments in (("E1", ["E1"]), ("selftest", ["--selftest"]))}
    before = keys()
    edit_file(base, "s/two.sh", "echo two", "echo two changed")
    after = keys()
    ok = None not in before.values() and None not in after.values() and after["E1"] == before["E1"] and after["selftest"] != before["selftest"]
    return ok, f"改前 {before}；改 s/two.sh 之后 {after}"


def shell_entry_exit_table_cell(work):
    """SHELL_ENTRY_EXIT_TABLE_CELL：SHELL_ENTRY_EXIT_TABLE_FIXTURE 的复跑表脚本当入口，带 --selftest 与不带参数各算一把键；先改 s/one.sh、再改 s/two.sh
    （各只由表里一行的驱动起），每改一处都同上一次比：带 --selftest 的都不变，不带参数的都变。"""
    base = os.path.join(work, "tree")
    write_tree(base, SHELL_ENTRY_EXIT_TABLE_FIXTURE)
    entry = "scripts/experiment-apparatus/replay.sh"

    def keys():
        return {label: entry_arguments_key(base, entry, arguments)[0] for label, arguments in (("selftest", ["--selftest"]), ("bare", []))}
    previous, outcomes = keys(), []
    for path, old, new in (("s/one.sh", "echo one", "echo one changed"), ("s/two.sh", "echo two", "echo two changed")):
        edit_file(base, path, old, new)
        after = keys()
        if None in previous.values() or None in after.values():
            return False, f"改 {path} 前后有一把键算不出（拒算了）：{previous}、{after}"
        outcomes.append((path, after["selftest"] == previous["selftest"], after["bare"] != previous["bare"]))
        previous = after
    return all(same and changed for _path, same, changed in outcomes), "；".join(
        f"改 {path}：带 --selftest 的键{'不变' if same else '变了'}（要不变）、不带参数的键{'变了' if changed else '不变'}（要变）" for path, same, changed in outcomes)


def shell_table_tree_copy_cell(work):
    """SHELL_TABLE_TREE_COPY_CELL：SHELL_TABLE_TREE_COPY_FIXTURE 的 s/copy_caller.sh 当入口（不带参数），先改 s/two.sh、再改 s/one.sh，每改一处同上一次比：
    改 s/two.sh 键不变、改 s/one.sh 键变。"""
    base = os.path.join(work, "tree")
    write_tree(base, SHELL_TABLE_TREE_COPY_FIXTURE)
    previous, previous_refused = fixture_key(base, "s/copy_caller.sh")
    outcomes = []
    for path, old, new, want in (("s/two.sh", "echo two", "echo two changed", "same"), ("s/one.sh", "echo one", "echo one changed", "changed")):
        edit_file(base, path, old, new)
        after, after_refused = fixture_key(base, "s/copy_caller.sh")
        if previous is None or after is None:
            return False, f"改 {path} 前后拒算了：{sorted(previous_refused)}、{sorted(after_refused)}"
        outcomes.append((path, want, "changed" if after != previous else "same"))
        previous, previous_refused = after, after_refused
    return all(want == got for _path, want, got in outcomes), "；".join(f"改 {path} 期望 {want}、实际 {got}" for path, want, got in outcomes)



def loader_tree(base, canonical):
    """在 base 下拷一棵只有 .claude/ 的小树：两份装载器，canonical 给了就连同规范副本的 preflight.sh / preflight.py。"""
    repository = os.path.normpath(os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", ".."))
    for path in LOADER_FILES:
        os.makedirs(os.path.join(base, os.path.dirname(path)), exist_ok=True)
        shutil.copyfile(os.path.join(repository, path), os.path.join(base, path))
    if canonical is not None:
        target = os.path.join(base, ".claude", "singlefs-ai-sop", "scripts")
        os.makedirs(target, exist_ok=True)
        for name in CANONICAL_FILES:
            shutil.copyfile(os.path.join(canonical, name), os.path.join(target, name))


def loader_resolutions(tree, work):
    """→ (source 装载器之后 PREFLIGHT_PROGRAM 指的文件, import 装载器之后 preflight 模块的文件, 出错说明)，都取真实路径。"""
    environment = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "HOME": work, "LC_ALL": "C", "PYTHONDONTWRITEBYTECODE": "1"}
    shell = subprocess.run(["bash", "--norc", "--noprofile", "-c", 'source "$1/.claude/scripts/preflight.sh" && printf "%s\\n" "$PREFLIGHT_PROGRAM"',
                            "loader", tree], cwd=work, env=environment, capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=60)
    python = subprocess.run([sys.executable, "-B", "-c", "import sys; sys.path.insert(0, sys.argv[1]); import project_preflight; "
                             "print(project_preflight.shared_preflight_module('loader').__file__)", os.path.join(tree, ".claude", "scripts")],
                            cwd=work, env=environment, capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=60)
    got = [os.path.realpath(completed.stdout.strip()) if completed.returncode == 0 and completed.stdout.strip() else None for completed in (shell, python)]
    return got[0], got[1], f"bash 退 {shell.returncode}：{shell.stderr.strip()[:200]}；python 退 {python.returncode}：{python.stderr.strip()[:200]}"


def preflight_loader_cell(work):
    """只拷了 .claude/ 的小树：source / import 两份装载器都解到这棵树里的规范副本（不靠 git、不靠主仓），两份装载器的闭包不拒算、收进副本。"""
    canonical = sys.modules["project_preflight"].shared_preflight_directory()
    if canonical is None:
        return False, "本仓找不到规范副本（.claude/singlefs-ai-sop/scripts/），造不出小树"
    tree = os.path.join(work, "tree")
    loader_tree(tree, canonical)
    want = os.path.realpath(os.path.join(tree, ".claude", "singlefs-ai-sop", "scripts", "preflight.py"))
    shell, python, detail = loader_resolutions(tree, work)
    if shell != want or python != want:
        return False, f"要都解到 {want}，source 解到 {shell}，import 解到 {python}（{detail}）"
    for entry, member in ((".claude/scripts/preflight.sh", ".claude/singlefs-ai-sop/scripts/preflight.sh"),
                          (".claude/scripts/project_preflight.py::shared_preflight_module", ".claude/singlefs-ai-sop/scripts/preflight.py")):
        closure = closure_of(tree, entry)
        files = {unit.file for unit in closure.order}
        if closure.refusals or member not in files:
            return False, f"{entry} 的闭包拒算 {sorted(refusal.text() for refusal in closure.refusals)}，碰到的文件 {sorted(files)}"
    return True, ""


def selftest_git_steps(work, steps):
    """自证造 git 仓：逐条起 git <step>（不读系统与用户的 git 配置，HOME 指 work，提交人写死）→ 头一条退非 0 的那一句，全退 0 交 None。"""
    environment = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "HOME": work, "LC_ALL": "C", "GIT_CONFIG_NOSYSTEM": "1"}
    git = ["git", "-c", "user.name=selftest", "-c", "user.email=selftest@example.invalid", "-c", "commit.gpgsign=false", "-c", "init.defaultBranch=main"]
    for step in steps:
        completed = subprocess.run(git + step, env=environment, capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=60)
        if completed.returncode != 0:
            return f"git {' '.join(step[:3])} 退 {completed.returncode}：{completed.stderr.strip()[:200]}"
    return None


def preflight_worktree_cell(work):
    """git worktree 里没有规范副本（.gitignore 挡着）：source / import 两份装载器都解到主仓那一份。"""
    canonical = sys.modules["project_preflight"].shared_preflight_directory()
    if canonical is None:
        return False, "本仓找不到规范副本（.claude/singlefs-ai-sop/scripts/），造不出主仓"
    main, linked = os.path.join(work, "main"), os.path.join(work, "linked")
    loader_tree(main, canonical)
    with open(os.path.join(main, ".gitignore"), "w", encoding="utf-8") as handle:
        handle.write("/.claude/singlefs-ai-sop/\n")
    problem = selftest_git_steps(work, (["init", "-q", main], ["-C", main, "add", ".gitignore", ".claude/scripts"],
                                        ["-C", main, "commit", "-q", "-m", "loaders"], ["-C", main, "worktree", "add", "-q", "--detach", linked]))
    if problem is not None:
        return False, problem
    if os.path.exists(os.path.join(linked, ".claude", "singlefs-ai-sop")):
        return False, "worktree 里不该有规范副本，样本没造对"
    want = os.path.realpath(os.path.join(main, ".claude", "singlefs-ai-sop", "scripts", "preflight.py"))
    shell, python, detail = loader_resolutions(linked, work)
    return shell == want and python == want, f"要都解到主仓的 {want}，source 解到 {shell}，import 解到 {python}（{detail}）"


def key_toolchain_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, PYTHON_FIXTURE)
    closure = closure_of(base, "pkg/a.py::entry")
    plain, _ = closure_key(closure)
    with_toolchain, _ = closure_key(closure, toolchain=["python 3.12.3"])
    return plain is not None and plain != with_toolchain, "加一行工具链键要变"


def key_external_cell(work):
    """闭包点名了 cargo（Closure.programs 记着）：不给 externals 不拒算、算得出键（文件头「版本不进键」）。"""
    base = os.path.join(work, "tree")
    write_tree(base, SHELL_FIXTURE)
    closure = closure_of(base, "s/tools.sh")
    key, refusals = closure_key(closure)
    return "cargo" in closure.programs and key is not None and not refusals, \
        f"闭包记的外部程序 {sorted(closure.programs)}（要有 cargo）；不给 externals 时键 {key}、拒算 {[refusal.form for refusal in refusals]}"
