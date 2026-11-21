"""自证：夹具树的写法与改法、比键的格函数（compare_cell、refused_cell）、构建档不进键的格、真仓拷贝（real_tree）与判法入口并行核对用的函数。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import ast
import glob
import importlib.util
import os
import re
import shutil
from lib_function_units_lexer import BREAK_VARIABLE, active_breaks
from lib_function_units_base import FunctionUnitsError, LIBRARY_ROOT, files_under
from lib_function_units_rust_workspace import RustWorkspace
from lib_function_units_script_closure import SHARED_SCRIPT_INDICES, ScriptClosure, closure_of, default_script_directories
from lib_function_units_closure_keys import closure_key, dump_lines, edit_file, write_tree


# shell 一侧的几种折法（文件头「起代码的写法逐种认」shell 那几条）的夹具：$PWD、${名%词} / ${名##词}、cd 到临时树里的目录、cd 到折不出的目录之后的相对路径、
# 「<根里的目录>/$1」、函数里 python3 "$@"
SHELL_FOLDS_FIXTURE = {
    "top_child.sh": "#!/usr/bin/env bash\necho top\n",
    "s/child.sh": "#!/usr/bin/env bash\necho child\n",
    "s/tools/one.py": 'def main():\n    return 1\n\n\nif __name__ == "__main__":\n    main()\n',
    "s/tools/two.py": 'def main():\n    return 2\n\n\nif __name__ == "__main__":\n    main()\n',
    "s/pwd_script.sh": '#!/usr/bin/env bash\ncd "$(dirname "$0")/.." || exit 2\nrun_top() {\n  bash "$PWD/top_child.sh"\n}\nrun_top\n',
    "s/pattern_removal.sh": ('#!/usr/bin/env bash\nSELF="s/pattern_removal.sh"\nCHILD_NAME="x/child.sh"\n'
                             'CHILD="${SELF%/pattern_removal.sh}/${CHILD_NAME##*/}"\nbash "$CHILD"\n'),
    "s/directory_copy.sh": '#!/usr/bin/env bash\nrun_tool() {\n  (cd "$1/s/tools" && python3 one.py)\n}\nrun_tool "$1"\n',
    "s/relative_copy.sh": '#!/usr/bin/env bash\nrun_in_copy() {\n  (cd "$1" && bash s/child.sh)\n}\nrun_in_copy "$1"\n',
    "s/directory_wildcard.sh": '#!/usr/bin/env bash\nrun_named() {\n  python3 "s/tools/$1"\n}\nrun_named one.py\n',
    "s/positional_wrapper.sh": '#!/usr/bin/env bash\nrun_python() {\n  PYTHONDONTWRITEBYTECODE=1 python3 "$@"\n}\nrun_python s/tools/one.py --flag\n',
    "s/printed_word_wrapper.sh": ('#!/usr/bin/env bash\ncd "$(dirname "$0")/.." || exit 1\nrun_python() {\n  PYTHONDONTWRITEBYTECODE=1 python3 "$@"\n}\n'
                                  'where_is() { printf \'%s\\n\' "$PWD/s/$1"; }\nrun_python "$(where_is tools)/one.py" --flag\n'),
    "s/quoted_assignment_env.sh": '#!/usr/bin/env bash\nthreads=4\nenv "E_THREADS=$threads" nice -n 19 s/child.sh --flag\n',
    "s/quoted_assignment_array.sh": ('#!/usr/bin/env bash\ncount_of() { echo 4; }\nrun() {\n  local environment=()\n  environment+=("E_THREADS=$(count_of)")\n'
                                     '  env ${environment[@]+"${environment[@]}"} \\\n    s/child.sh --flag\n}\nrun\n'),
    "s/relative_cd.sh": '#!/usr/bin/env bash\n(cd s/tools && python3 one.py)\n',
    "s/quoted_brace.sh": ("#!/usr/bin/env bash\ncount_rows() {\n  rows=\"$(awk '\n    /x/ {\n      split($2, field, \"=\")\n    }\n"
                          "    END { print NR }\n' \"$1\")\"\n  bash s/child.sh \"$rows\"\n}\ncount_rows input.tsv\n"),
    "s/tail_group.sh": ('#!/usr/bin/env bash\nROOT="$(pwd)"\nSAMPLE_DIRECTORY=".sample"\nrun_driver() {\n  bash "$ROOT/$SAMPLE_DIRECTORY/driver.sh"\n}\n'
                        'run_driver\n'),
    "s/tail_outside.sh": '#!/usr/bin/env bash\nROOT="$(pwd)"\nbash "$ROOT/.nowhere/run.sh"\n',
    "fixtures/one/.sample/driver.sh": "#!/usr/bin/env bash\necho driver\n",
    "s/dispatch.sh": ('#!/usr/bin/env bash\nif [[ "${1:-}" == --selftest ]]; then\n  bash s/child.sh\n  exit $?\nfi\ncase "$1" in\n  run)\n'
                      '    python3 s/tools/one.py\n    ;;\n  *)\n    python3 s/tools/two.py\n    ;;\nesac\n'),
    "s/dispatch_caller.sh": "#!/usr/bin/env bash\nbash s/dispatch.sh run\n",
    "s/open_tail_caller.sh": '#!/usr/bin/env bash\nbash s/dispatch.sh run "$@"\n',
    "s/open_tail_first_word.sh": '#!/usr/bin/env bash\nbash s/dispatch.sh "$@"\n',
    "s/open_tail_launcher.py": 'import subprocess\nimport sys\n\n\ndef judge():\n    return subprocess.run(["bash", "s/dispatch.sh", "run", *sys.argv[1:]], check=False)\n',
    "s/dispatch_preflight.sh": ('#!/usr/bin/env bash\npreflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}\n'
                                'if [[ "${1:-}" == --selftest ]]; then\n  bash s/child.sh\n  exit $?\nfi\npython3 s/tools/one.py\n'),
    "s/dispatch_preflight_caller.sh": "#!/usr/bin/env bash\nbash s/dispatch_preflight.sh run\n",
    "s/dispatch_exit.sh": '#!/usr/bin/env bash\nif [[ "${1:-}" == --only ]]; then\n  bash s/child.sh\n  exit $?\nfi\npython3 s/tools/two.py\n',
    "s/dispatch_exit_caller.sh": "#!/usr/bin/env bash\nbash s/dispatch_exit.sh --only\n",
    "s/launcher.py": ('import subprocess\n\n\ndef run_script(arguments):\n    return subprocess.run(["bash", "s/dispatch.sh", *arguments], check=False)\n\n\n'
                      'def judge():\n    return run_script(["run"])\n'),
    "s/nested_word.sh": ("#!/usr/bin/env bash\nonly_selftest() {\n  device() {\n    bash s/child.sh\n  }\n  device\n}\nnamed_as_a_word() {\n  device=\"$1\"\n  echo \"$device\"\n}\n"
                         "named_as_a_word disk\n"),
    "s/nested_called_inside.sh": "#!/usr/bin/env bash\nouter() {\n  helper() {\n    bash s/child.sh\n  }\n  helper\n}\nouter\n",
    "s/nested_called_outside.sh": "#!/usr/bin/env bash\nouter() {\n  helper() {\n    bash s/child.sh\n  }\n}\nouter\nhelper\n",
    "s/commands/dispatch.py": ('import sys\n\n\ndef alpha():\n    return 301\n\n\ndef beta():\n    return 302\n\n\nCOMMANDS = {"alpha": alpha}\n\n\n'
                              'def main(arguments):\n    if arguments[:1] == ["--selftest"]:\n        return beta()\n    return COMMANDS[arguments[0]]()\n\n\n'
                              'if __name__ == "__main__":\n    sys.exit(main(sys.argv[1:]))\n'),
    "s/python_words.sh": "#!/usr/bin/env bash\npython3 s/commands/dispatch.py alpha \"$1\"\n",
    "s/python_words_open.sh": "#!/usr/bin/env bash\npython3 s/commands/dispatch.py \"$@\"\n",
    "s/module_body_command.py": ('import subprocess\n\nfor name in ("first", "second"):\n    prefix = ["bash", "s/child.sh"]\n    for arguments in (["one"], ["two"]):\n'
                                 '        subprocess.run([*prefix, *arguments], check=False)\n'),
    "s/exported_path.sh": ('#!/usr/bin/env bash\nEXPORTED_SELF_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"\nexport EXPORTED_SELF_PATH\n'
                           'exec python3 - "$@" <<\'PYTHON\'\nimport os\nimport subprocess\nimport sys\n\nSCRIPT_DIRECTORY = os.path.dirname(os.environ["EXPORTED_SELF_PATH"])\n'
                           'sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "tools"))\nsubprocess.run(["bash", os.path.join(SCRIPT_DIRECTORY, "child.sh")], check=False)\nPYTHON\n'),
    # 拆成 shell 入口加同目录模块：入口 export 自己的路径再起内联 python，拆出的 exported_part_driver.py 读那个变量取目录、在模块级把它插进 sys.path、
    # 起同目录的 child.sh。插 sys.path 那一句只有照入口折出变量才认得（折不出拒算 sys-path-unfolded）；起 child.sh 那一句折不出时照脚本名那一截整组连，判不出继承
    "s/exported_part.sh": ('#!/usr/bin/env bash\nEXPORTED_PART_SELF_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"\n'
                           'export EXPORTED_PART_SELF_PATH\nexec python3 - "$@" <<\'PYTHON\'\nimport os\nimport sys\n\n'
                           'sys.path.insert(0, os.path.dirname(os.environ["EXPORTED_PART_SELF_PATH"]))\nfrom exported_part_driver import launch_child  # noqa: E402\n'
                           'launch_child()\nPYTHON\n'),
    "s/exported_part_driver.py": ('import os\nimport subprocess\nimport sys\n\nSELF_PATH = os.environ["EXPORTED_PART_SELF_PATH"]\nSCRIPT_DIRECTORY = os.path.dirname(SELF_PATH)\n'
                                  'sys.path.insert(0, SCRIPT_DIRECTORY)\n\n\n'
                                  'def launch_child():\n    return subprocess.run(["bash", os.path.join(SCRIPT_DIRECTORY, "child.sh")], check=False)\n'),
    "s/guard_path/run.py": ('import os\nimport sys\n\n\ndef main():\n    return 0\n\n\nif __name__ == "__main__":\n'
                            '    HELPER_DIRECTORY = os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "guard_helpers")\n    if os.path.isdir(HELPER_DIRECTORY):\n'
                            '        sys.path.insert(0, HELPER_DIRECTORY)\n        from guard_helper import guard_value\n        guard_value()\n    sys.exit(main())\n'),
    "s/guard_helpers/guard_helper.py": "def guard_value():\n    return 601\n",
    "s/heredoc_blank_row.sh": ('#!/usr/bin/env bash\ncat <<\'TEXT\'\nbefore the blank row\n\nafter the blank row\nTEXT\nif [[ "${1:-}" == --selftest ]]; then\n  bash s/child.sh\n  exit $?\nfi\n'
                               'python3 s/tools/one.py\n'),
    "s/heredoc_blank_row_caller.sh": "#!/usr/bin/env bash\nbash s/heredoc_blank_row.sh run\n",
    "s/shared_view.sh": '#!/usr/bin/env bash\nif [[ "${1:-}" == --selftest ]]; then\n  bash s/child.sh\n  exit $?\nfi\npython3 s/tools/one.py\n',
}
SHELL_ARGUMENTS_PREFLIGHT_CELL = ("shell：bash s/dispatch_preflight.sh run：preflight 放回参数那一句（set -- ${PREFLIGHT_ARGUMENTS[@]+…}）不算动了位置参数，"
                                  "--selftest 那一支照样剪掉（改 s/child.sh 键不变）")
SHELL_PYTHON_ARGUMENTS_SKIPPED_CELL = "shell：python3 <脚本> alpha \"$1\"：--selftest 那一支走不到（改它调的函数键不变）"
SHELL_PYTHON_ARGUMENTS_OPEN_CELL = "shell：python3 <脚本> \"$@\"：词不知道，--selftest 那一支照收（改它调的函数键变）"
# 按第一列选行的表（SHELL_ROW_SELECTED_TABLES）：夹具用登记的那个路径，caller 带实验号起它
SHELL_TABLE_FIXTURE = {
    "scripts/experiment-apparatus/replay.sh": ("#!/usr/bin/env bash\ndriver_one() {\n  bash s/one.sh\n}\ndriver_two() {\n  bash s/two.sh\n}\n"
                                               "TABLE=$(cat <<'TSV'\nE1|@driver_one||one.out|exact\nE2|@driver_two||two.out|exact\nTSV\n)\n"
                                               "ADMISSION_ROW_KEYS=$(cat <<'TSV'\nE2/extra\tE2|@driver_two|\treplay\nTSV\n)\n"
                                               "bad_rows=$(printf '%s\\n' \"$TABLE\" | grep -c slash)\n"
                                               "printf '%s\\n' \"$bad_rows\" \"$ADMISSION_ROW_KEYS\"\n"),
    "s/one.sh": "#!/usr/bin/env bash\necho one\n",
    "s/two.sh": "#!/usr/bin/env bash\necho two\n",
    "s/caller.sh": "#!/usr/bin/env bash\nbash scripts/experiment-apparatus/replay.sh E1\n",
    "s/caller_mixed.sh": "#!/usr/bin/env bash\nbash scripts/experiment-apparatus/replay.sh E1\nbash scripts/experiment-apparatus/replay.sh \"$1\"\n",
}
# 只带选项、走到的那一支以 exit 收尾、表在 fi 之后才赋值的复跑表脚本（replay.sh --selftest 那一种；ShellIndex.launch_defines_tables）：
# 自证那一支只按实验号 E1 起自己，E2 那一行的驱动只由整张表连得到
SHELL_TABLE_SELFTEST_FIXTURE = {
    "scripts/experiment-apparatus/replay.sh": ("#!/usr/bin/env bash\ndriver_one() {\n  bash s/one.sh\n}\ndriver_two() {\n  bash s/two.sh\n}\n"
                                               "selftest_body() {\n  bash scripts/experiment-apparatus/replay.sh E1\n}\n"
                                               "if [[ \"${1:-}\" == \"--selftest\" ]]; then\n  selftest_body\n  exit $?\nfi\n"
                                               "TABLE=$(cat <<'TSV'\nE1|@driver_one||one.out|exact\nE2|@driver_two||two.out|exact\nTSV\n)\n"
                                               "printf '%s\\n' \"$TABLE\"\n"),
    "s/one.sh": "#!/usr/bin/env bash\necho one\n",
    "s/two.sh": "#!/usr/bin/env bash\necho two\n",
}
SHELL_TABLE_SELFTEST_ENTRY_CELL = ("shell 入口是 <复跑表脚本> 只带 --selftest、走到的那一支以 exit 收尾、表在 fi 之后才赋值：这次起法不算整张表，"
                                   "读表只照自证里 bash <复跑表脚本> E1 那一次连（改 s/one.sh 键变、改 s/two.sh 键不变）")
# 入口自己只带 --selftest、那一支以 exit 收尾、自证那一支不再起自己（ShellIndex.narrowed_script_body 照这组词剪入口）：表的赋值与读表的变量都在 fi 之后
SHELL_ENTRY_EXIT_TABLE_FIXTURE = {
    "scripts/experiment-apparatus/replay.sh": ("#!/usr/bin/env bash\ndriver_one() {\n  bash s/one.sh\n}\ndriver_two() {\n  bash s/two.sh\n}\n"
                                               "if [[ \"${1:-}\" == \"--selftest\" ]]; then\n  echo selftest\n  exit $?\nfi\n"
                                               "TABLE=$(cat <<'TSV'\nE1|@driver_one||one.out|exact\nE2|@driver_two||two.out|exact\nTSV\n)\n"
                                               "bad_rows=$(printf '%s\\n' \"$TABLE\" | grep -c slash)\n"
                                               "printf '%s\\n' \"$bad_rows\"\n"),
    "s/one.sh": "#!/usr/bin/env bash\necho one\n",
    "s/two.sh": "#!/usr/bin/env bash\necho two\n",
}
SHELL_ENTRY_EXIT_TABLE_CELL = ('shell 入口是 <复跑表脚本>、--selftest 那一支以 exit 收尾、表与读表的变量在 fi 之后：'
                               '带 --selftest 时表里的驱动函数都不连（改 s/one.sh、s/two.sh 键不变），不带参数时都连（改它们键变）')
# 仓副本里起复跑表脚本（cd 到折不出的目录之后的相对路径，照根里同一相对路径那一份认）：起法的字面参数照记（ShellIndex.record_launch_words）
SHELL_TABLE_TREE_COPY_FIXTURE = {
    **{path: SHELL_TABLE_FIXTURE[path] for path in ("scripts/experiment-apparatus/replay.sh", "s/one.sh", "s/two.sh")},
    "s/copy_caller.sh": '#!/usr/bin/env bash\nrun_in_copy() {\n  (cd "$1" && bash scripts/experiment-apparatus/replay.sh E1)\n}\nrun_in_copy "$1"\n',
}
# shell 入口带参数（文件头「程序入口带参数」）：入口是 script-body、闭包从 closure_of 带 Options.program_arguments 起
SHELL_ENTRY_FIXTURE = {
    **{path: SHELL_FOLDS_FIXTURE[path] for path in ("s/dispatch.sh", "s/child.sh", "s/tools/one.py", "s/tools/two.py")},
    "s/dispatch_undecided.sh": ('#!/usr/bin/env bash\nif [[ "${1:-}" == "$SELFTEST_WORD" ]]; then\n  bash s/child.sh\n  exit $?\nfi\n'
                                'python3 s/tools/one.py\n'),
}
SHELL_ENTRY_ARGUMENTS_CELL = ("shell 入口带参数：s/dispatch.sh 带 run，--selftest 那一支与 *) 那一支走不到（改 s/child.sh、s/tools/two.py 键不变），"
                              "run 那一支走到（改 s/tools/one.py 键变）")
SHELL_ENTRY_TABLE_CELL = ("shell 入口是 <复跑表脚本> 带参数：带 E1 时别的实验的驱动函数走不到（改 s/two.sh 键不变），"
                          "只带 --selftest 照整张表连（改 s/two.sh 键变）")
SHELL_ARGUMENTS_SKIPPED_CELL = "shell：bash s/dispatch.sh run：if [[ \"${1:-}\" == --selftest ]] 那一支走不到，它起的 s/child.sh 不连（改它键不变）"
OPEN_TAIL_SKIPPED_CELL = "shell：bash s/dispatch.sh run \"$@\"：头一个词字面，--selftest 那一支走不到（改 s/child.sh 键不变）"
OPEN_TAIL_FIRST_WORD_CELL = "shell：bash s/dispatch.sh \"$@\"：头一个词就折不出，--selftest 那一支照收（改 s/child.sh 键变）"
OPEN_TAIL_PYTHON_CELL = "python：[\"bash\", <脚本>, \"run\", *<折不出的列表>]：头一个词字面，--selftest 那一支走不到（改 s/child.sh 键不变）"
TAIL_VIEW_CELL = "python：程序入口带 --selftest：main 里走不到的那一处调用不算调用处，读 <参数>/<字面尾巴> 的那一组不收（改它键不变）"
TAIL_VIEW_FIXTURE = {
    "elsewhere/Manifest.toml": "real\n",
    "pkg/packages.py": '''import os
import sys
import tempfile


def package_name(tree, relative):
    directory = os.path.dirname(os.path.join(tree, relative))
    with open(os.path.join(directory, "Manifest.toml"), encoding="utf-8") as handle:
        return handle.read()


def check(tree):
    return package_name(tree, "src/lib.rs")


def selftest():
    work = tempfile.mkdtemp()
    return check(os.path.join(work, "tree"))


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    return check(arguments[1])


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
''',
}


def fixture_key(root, entry, options=None):
    """→ (键或 None, 拒算形态的集合)。"""
    closure = closure_of(root, entry, options)
    key, refusals = closure_key(closure)
    return key, {refusal.form for refusal in refusals}


def compare_cell(fixture, entry, path, old, new, want):
    def judge(work):
        base = os.path.join(work, "tree")
        write_tree(base, fixture)
        before, before_refused = fixture_key(base, entry)
        edit_file(base, path, old, new)
        after, after_refused = fixture_key(base, entry)
        if before is None or after is None:
            return False, f"拒算了：改前 {sorted(before_refused)}，改后 {sorted(after_refused)}"
        got = "changed" if before != after else "same"
        return got == want, f"期望 {want}，实际 {got}"
    return judge


UNIT_GRAPH_SOURCE_DIRECTORY = os.path.join("crates", "singlefs-unit-graph", "src")


def unit_graph_source_files():
    """crates/singlefs-unit-graph/src 底下每一份 .rs（连同子目录里的：拆过的模块在 <模块>/ 底下），按相对路径排。
    弄坏开关 unit-graph-sources-top-level-only 下只看 src 这一层。"""
    source_directory = os.path.join(LIBRARY_ROOT, UNIT_GRAPH_SOURCE_DIRECTORY)
    if "unit-graph-sources-top-level-only" in active_breaks():
        return [os.path.join(source_directory, name) for name in sorted(os.listdir(source_directory)) if name.endswith(".rs")]
    return [os.path.join(source_directory, relative) for relative in files_under(source_directory, (".rs",))]


def rust_string_list_constant(constant_name):
    """crates/singlefs-unit-graph/src 下一个 `const <名>: [&str; N] = [ "…", … ];` 的 (N, 各项按写的次序)；整个目录里恰好一份才算找到，不然抛 AssertionError。"""
    source_directory = os.path.join(LIBRARY_ROOT, UNIT_GRAPH_SOURCE_DIRECTORY)
    pattern = re.compile(r"\bconst\s+" + re.escape(constant_name) + r"\s*:\s*\[&str;\s*(\d+)\]\s*=\s*\[(.*?)\];", re.DOTALL)
    found = []
    for path in unit_graph_source_files():
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
        found += [(int(match.group(1)), re.findall(r'"([^"]*)"', match.group(2))) for match in pattern.finditer(text)]
    if len(found) != 1:
        raise AssertionError(f"{source_directory} 下 const {constant_name}: [&str; N] 找到 {len(found)} 份（要恰好 1 份）")
    return found[0]


def rust_string_list_cell(constant_name, python_list):
    """读 Rust 源码里那个常量，与 python_list()（这一边用的那份名单）逐项比：项数、每一项、次序都要相同。"""
    def judge(work):
        declared, items = rust_string_list_constant(constant_name)
        got = list(python_list())
        return declared == len(items) and items == got, f"Rust 源码里 {constant_name} 写 {declared} 项、读到 {items}；这一边 {got}"
    return judge


def rust_string_constant(constant_name):
    """crates/singlefs-unit-graph/src 下一个 `const <名>: &str = "…";` 的值；整个目录里恰好一份才算找到，不然抛 AssertionError。"""
    source_directory = os.path.join(LIBRARY_ROOT, UNIT_GRAPH_SOURCE_DIRECTORY)
    pattern = re.compile(r'\bconst\s+' + re.escape(constant_name) + r'\s*:\s*&str\s*=\s*"([^"]*)"\s*;')
    found = []
    for path in unit_graph_source_files():
        with open(path, encoding="utf-8") as handle:
            found += pattern.findall(handle.read())
    if len(found) != 1:
        raise AssertionError(f"{source_directory} 下 const {constant_name}: &str 找到 {len(found)} 份（要恰好 1 份）")
    return found[0]


PROFILE_UNKEYED_RUST_FORMS = (re.compile(r"\bconst\s+PROFILE_SETTING_KEYS\b"), re.compile(r'"profile\.incremental: '),
                              re.compile(r'"profile\.codegen-units-of-non-path-dependencies: '))
PROFILE_UNKEYED_PYTHON_FORMS = (re.compile(r"^\s*PROFILE_SETTING_KEYS\s*=", re.M), re.compile(r"^\s*def\s+effective_profile_lines", re.M),
                                re.compile(r"""["']profile[.:]"""))
PROFILE_UNKEYED_SAMPLE = ("弄坏开关 profile-unkeyed-scan-sample 扫的样本", 'pub const PROFILE_SETTING_KEYS: [&str; 1] = ["opt-level"];\n')


def profile_unkeyed_sources():
    """PROFILE_UNKEYED_CELL 扫的源码 → [(相对路径, 原文, 认它的几个写法)]：crates/singlefs-unit-graph/src 下每一份 .rs（unit_graph_source_files），
    这份库切单元、算键的几份 .py（自证与夹具那几份不算：它们的夹具文字里本来就有这些词）。弄坏开关 profile-unkeyed-scan-sample 下多一段 PROFILE_UNKEYED_SAMPLE。"""
    sources = []
    for path in unit_graph_source_files():
        with open(path, encoding="utf-8", errors="replace") as handle:
            sources.append((os.path.relpath(path, LIBRARY_ROOT), handle.read(), PROFILE_UNKEYED_RUST_FORMS))
    gate = os.path.join(LIBRARY_ROOT, "scripts", "gate")
    for name in sorted(os.listdir(gate)):
        if (name == "lib_function_units.py" or name.startswith("lib_function_units_")) and name.endswith(".py") and "_selftest" not in name:
            with open(os.path.join(gate, name), encoding="utf-8", errors="replace") as handle:
                sources.append((os.path.join("scripts", "gate", name), handle.read(), PROFILE_UNKEYED_PYTHON_FORMS))
    if "profile-unkeyed-scan-sample" in active_breaks():
        sources.append((PROFILE_UNKEYED_SAMPLE[0], PROFILE_UNKEYED_SAMPLE[1], PROFILE_UNKEYED_RUST_FORMS))
    return sources


def profile_unkeyed_cell(work):
    """PROFILE_UNKEYED_CELL：构建档与线程数不进键（文件头「构建档与线程数不进键」）：crates/singlefs-unit-graph/src 与这份库的源码里都没有出编译档行的写法
    （Rust 的 const PROFILE_SETTING_KEYS、"profile.incremental: "、"profile.codegen-units-of-non-path-dependencies: " 这几个行头；这一边的 PROFILE_SETTING_KEYS =、
    def effective_profile_lines、以 profile. / profile: 起头的串），扫的源码一份都不缺。"""
    sources = profile_unkeyed_sources()
    found = [f"{path}:{text.count(chr(10), 0, match.start()) + 1}「{match.group(0)}」" for path, text, forms in sources for form in forms
             for match in form.finditer(text)]
    rust_files = sum(1 for path, _text, forms in sources if forms is PROFILE_UNKEYED_RUST_FORMS)
    return bool(rust_files) and not found, (f"扫了 {len(sources)} 份（Rust {rust_files} 份）；出编译档行的写法 {found[:8]}"
                                            "（照算键库文件头「构建档与线程数不进键」删掉出这些行的写法，Rust 与这一边同一次改）")


def with_breaks_added(judge, *switches):
    """这一格在当时设着的弄坏开关之上再打开 switches 判（只看一条边、别的边先关掉的格用）：BREAK_CELLS 轮打开点名的开关时，两样一起开着。"""
    def judge_with_breaks(work):
        previous = os.environ.get(BREAK_VARIABLE)
        os.environ[BREAK_VARIABLE] = ",".join(sorted(active_breaks() | set(switches)))
        try:
            return judge(work)
        finally:
            if previous is None:
                os.environ.pop(BREAK_VARIABLE, None)
            else:
                os.environ[BREAK_VARIABLE] = previous
    return judge_with_breaks


def refused_cell(fixture, entry, form, edit=None):
    def judge(work):
        base = os.path.join(work, "tree")
        write_tree(base, fixture)
        if edit is not None:
            edit_file(base, *edit)
        key, forms = fixture_key(base, entry)
        return key is None and form in forms, f"要拒算 {form}，实际键 {'有' if key else '无'}、拒算 {sorted(forms)}"
    return judge


def rust_doctest_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, RUST_FIXTURE)
    rust_file = RustWorkspace(base).file("crates/fextra/src/lib.rs")
    doctests = [unit.name for unit in rust_file.units if unit.kind == "doctest"]
    if len(doctests) != 1:
        return False, f"要恰好一个文档测试单元，实际 {doctests}"
    entry = f"crates/fextra/src/lib.rs::{doctests[0]}"
    before, _ = fixture_key(base, entry)
    edit_file(base, "crates/fextra/src/lib.rs", "documented(), 4);", "documented(), 5);")
    after, _ = fixture_key(base, entry)
    return before is not None and after is not None and before != after, f"文档测试入口 {entry} 的键改前 {before}、改后 {after}"


def rust_dump_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, RUST_FIXTURE)
    unit_lines, refusal_lines, counts = dump_lines(base, "rust")
    shape = re.compile(r"^[a-z_]+\t[^\t]+\t[^\t]+:\d+-\d+\t[0-9a-f]{64}$")
    bad = [line for line in unit_lines if not shape.match(line)]
    wanted = ("item_macro\tfextra::listing!@5\tcrates/fextra/src/lib.rs:5-10", "fn\tfextra::bounded\tcrates/fextra/src/lib.rs:",
              "fn\tfextra::proven\tcrates/fextra/src/lib.rs:", "impl\tfcore::<impl std::ops::Add for Money>\tcrates/fcore/src/lib.rs:15-15",
              "fn\tfcore::<impl std::ops::Add for Money>::add\t")
    missing = [prefix for prefix in wanted if not any(line.startswith(prefix) for line in unit_lines)]
    return not bad and not missing and not refusal_lines and counts["rust"] > 30, f"形态不对 {bad[:2]}，缺 {missing}，拒算 {refusal_lines[:3]}"


def alignment_dump(work):
    base = os.path.join(work, "tree")
    write_tree(base, ALIGNMENT_FIXTURE)
    unit_lines = dump_lines(base, "rust")[0] + dump_lines(base, "wgsl")[0]
    return base, unit_lines


def line_with(lines, prefix):
    return any(line.startswith(prefix) for line in lines)


def alignment_kind_names_cell(work):
    """dump 的种类名照 crates/singlefs-unit-graph 的拼法：macro_rules、item_macro、mod、mod_head、wgsl_compute、wgsl_var。"""
    _base, lines = alignment_dump(work)
    wanted = ("macro_rules\tfalign::declare_named\t", "item_macro\tfalign::declare_named!@26\t", "mod\tfalign::helpers\t", "mod_head\tfalign::nested\t",
              "wgsl_compute\tfalign::kernel.wgsl::main\t", "wgsl_var\tfalign::kernel.wgsl::output\t")
    missing = [prefix for prefix in wanted if not line_with(lines, prefix)]
    return not missing, f"dump 里缺 {missing}"


# ── 判法入口的登记（文件头「判法入口的登记」）──

JUDGING_ENTRY_STAGE_DIRECTORY = ".claude/gate.d"
JUDGING_ENTRY_TOKEN = re.compile(r'"judge:([^"$\s]+?|\.claude/gate\.d/\$STAGE_NAME)::(\w+)"')


def module_literal(root, path, name):
    """root 里 path 那份 python 的模块级赋值 name 的字面值（module_literal_location 交的值）。"""
    return module_literal_location(root, path, name)[0]


def module_literal_location(root, path, name, depth=0):
    """root 里 path 那份 python 的模块级赋值 name 的 (字面值, 赋值所在的那份文件相对根)：只认字面量；这一份里没有赋值、而是从同目录的一份模块
    from <模块> import … name 再导出的（拆成入口加同目录模块），到那一份里接着找。
    找不到、不是字面量抛 FunctionUnitsError（弄坏开关 judging-entry-literals-split-unfollowed 下不到再导出的那一份里找）。"""
    with open(os.path.join(root, path), encoding="utf-8") as handle:
        tree = ast.parse(handle.read(), filename=path)
    if depth < 4 and "judging-entry-literals-split-unfollowed" not in active_breaks() and not any(
            isinstance(statement, ast.Assign) and any(isinstance(target, ast.Name) and target.id == name for target in statement.targets)
            for statement in tree.body):
        for statement in tree.body:
            if isinstance(statement, ast.ImportFrom) and not statement.level and statement.module and "." not in statement.module \
                    and any(alias.name == name and alias.asname in (None, name) for alias in statement.names):
                sibling = os.path.join(os.path.dirname(path), statement.module + ".py").replace(os.sep, "/")
                if os.path.isfile(os.path.join(root, sibling)):
                    return module_literal_location(root, sibling, name, depth + 1)
    return module_assigned_literal(tree, path, name), path


def module_assigned_literal(tree, path, name):
    """解好的模块 tree 里模块级头一处赋值 name 的字面值（module_literal_location 的那一份文件自己里的赋值；只认字面量）。"""
    for statement in tree.body:
        if isinstance(statement, ast.Assign) and any(isinstance(target, ast.Name) and target.id == name for target in statement.targets):
            try:
                return ast.literal_eval(statement.value)
            except (ValueError, TypeError, SyntaxError, RecursionError) as error:
                raise FunctionUnitsError(f"{path} 里模块级的 {name} 不是字面量（判法入口的登记只认字面量）：{error}") from error
    raise FunctionUnitsError(f"{path} 里没有模块级的 {name}（判法入口的登记挪了：照文件头「判法入口的登记」改 registered_judging_entries）")


def registered_judging_entries(root):
    """存判定的各路判法入口，从各份脚本的登记现取（不手抄）：admission.py 的 REPLAY_SCRIPT 与 REPLAY_JUDGING_FUNCTIONS、e165 的 JUDGING_ENTRY、
    E165 第二段判法模块的 JUDGING_ENTRIES、verus-proof-check.py 的 JUDGING_ENTRIES，再加门禁阶段（.claude/gate.d/*.sh）里写死的 "judge:<文件>::<函数>" 令牌
    （$STAGE_NAME 换成那一道阶段自己）。去重、保持登记的次序。变异的两路（广谱、crates）键里没有判法的代码，没有判法入口。"""
    root = os.path.abspath(root)
    entries = []
    replay_script = module_literal(root, "scripts/gate/admission.py", "REPLAY_SCRIPT")
    entries += [f"{replay_script}::{function}" for function in module_literal(root, "scripts/gate/admission.py", "REPLAY_JUDGING_FUNCTIONS")]
    entries.append(f"scripts/experiment-apparatus/e165-pin-red-examples.py::{module_literal(root, 'scripts/experiment-apparatus/e165-pin-red-examples.py', 'JUDGING_ENTRY')}")
    entries += [f"scripts/experiment-apparatus/e165_replay_mutation_kill_bounds_variant_judgement.py::{function}" for function in module_literal(root, "scripts/experiment-apparatus/e165_replay_mutation_kill_bounds_variant_judgement.py", "JUDGING_ENTRIES")]
    # 形式化判法的 judging_units 按 __file__ 取入口文件：入口写 JUDGING_ENTRIES 赋值所在的那一份（拆出的模块里判结局的函数与它同文件）
    formal_entries, formal_file = module_literal_location(root, "scripts/gate/verus-proof-check.py", "JUDGING_ENTRIES")
    entries += [f"{formal_file}::{function}" for function in formal_entries.values()]
    for stage in sorted(glob.glob(os.path.join(root, JUDGING_ENTRY_STAGE_DIRECTORY, "*.sh"))):
        with open(stage, encoding="utf-8") as handle:
            text = handle.read()
        relative = os.path.relpath(stage, root).replace(os.sep, "/")
        for path, function in JUDGING_ENTRY_TOKEN.findall(text):
            entries.append(f"{relative if '$STAGE_NAME' in path else path}::{function}")
    return list(dict.fromkeys(entries))


def real_tree(work, tree=None):
    """本仓的 scripts/ 与 .claude/（不带 fixtures、kb、__pycache__）拷进临时目录（tree 没给是 <work>/real；给了的拷进它底下，它可以已经在）：
    改真文件的几格在拷贝上改，不碰本仓。"""
    tree = tree or os.path.join(work, "real")
    for top in ("scripts", ".claude"):
        shutil.copytree(os.path.join(LIBRARY_ROOT, top), os.path.join(tree, top), ignore=REAL_TREE_IGNORED, symlinks=True)
    return tree


def judging_entry_refusal_workers(environment, registration_file=None):
    """judging_entry_refusals 开几个进程：这份库所在仓的 scripts/hosts/heavy_threads.py 的 default_heavy_threads（SINGLEFS_THREAD_CAP 设了照它，
    没设取登记的每台默认重活线程数 .claude/heavy-thread-default；取法只在那一处，这里不另解析；按路径载入、不进 sys.modules），
    取不到照它抛 HeavyThreadDefaultError，不退回按核数算。registration_file 只给自证换登记文件。
    弄坏开关 judging-entry-workers-head-word：取登记文件的头一个词，读不出就不用登记的数（退回按核数算；这里退回 1 个进程，
    不调取核数的函数，门禁 code-tooling 的 heavy-thread-defaults 格不许）。那份文件头一个词是 #，总是读不出。"""
    if "judging-entry-workers-head-word" in active_breaks():
        try:
            with open(registration_file or os.path.join(LIBRARY_ROOT, ".claude", "heavy-thread-default"), encoding="utf-8") as handle:
                return int(handle.read().split()[0])
        except (OSError, ValueError, IndexError):
            return 1
    return int(heavy_threads_module().default_heavy_threads(environment, registration_file=registration_file))


def heavy_threads_module():
    """这份库所在仓的 scripts/hosts/heavy_threads.py（每台默认重活线程数的取法只在那一处，这里不另解析），按路径载入、不进 sys.modules。
    判法入口拒算（judging_entry_refusal_workers）与自证（selftest）的并行度都从它取。"""
    specification = importlib.util.spec_from_file_location("heavy_threads_for_function_units",
                                                           os.path.join(LIBRARY_ROOT, "scripts", "hosts", "heavy_threads.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def repository_copy_module():
    """这份库所在仓的 scripts/hosts/lib_repository_copy.py（拷仓副本的闭包与拷法只在那一处，自证拷仓根条目也经它），按路径载入、不进 sys.modules。"""
    specification = importlib.util.spec_from_file_location("lib_repository_copy_for_function_units",
                                                           os.path.join(LIBRARY_ROOT, "scripts", "hosts", "lib_repository_copy.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def prepare_shared_script_index(root):
    """父进程把 root 下默认几个目录里的 python 模块与 shell 脚本先解析好，登进 SHARED_SCRIPT_INDICES（closure_of 取用）→ (那份 ScriptClosure, 登记的键)。"""
    under = default_script_directories(root)
    prepared = ScriptClosure(root, under)
    for path in files_under(root, (".py",), under):
        prepared.python.module(path)
    prepared.shell_index()
    key = (os.path.abspath(root), tuple(under))
    SHARED_SCRIPT_INDICES[key] = prepared
    return prepared, key
