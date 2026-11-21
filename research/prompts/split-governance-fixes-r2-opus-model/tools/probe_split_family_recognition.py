#!/usr/bin/env python3
"""探针：scripts/gate/lib_split_family_loaded.py 的 loaded_family 在绝对路径与几种载入写法下多认、少认。
用法：probe_split_family_recognition.py --frozen <冻结副本> --work <输出目录>
认法取冻结副本里那一份（按文件路径载入），布局建在 <输出目录>/split-family/<格>/ 下；每格一行 PROBE_SPLIT_FAMILY case=<格> …，
真仓的三格（task-server 入口、两份共用模块）读冻结副本里的文件，只读不写。
"""
import argparse, importlib.util, os, sys


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def names(paths):
    return sorted(os.path.basename(path) for path in paths)


def python_case(library, base, case, entry_text, extra_files):
    """一个目录里放 lib_demo.py（入口，正文 entry_text）与 extra_files（{文件名: 正文}），交入口的族里除入口之外的文件名。"""
    directory = os.path.join(base, case)
    write(os.path.join(directory, "lib_demo.py"), entry_text)
    for name, text in extra_files.items():
        write(os.path.join(directory, name), text)
    entry = os.path.join(directory, "lib_demo.py")
    return names(library.loaded_family(entry, library.sibling_files(entry))[1:])


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    frozen = os.path.realpath(arguments.frozen)
    base = os.path.join(os.path.realpath(arguments.work), "split-family")
    sys.dont_write_bytecode = True
    specification = importlib.util.spec_from_file_location("probe_split_family_loaded", os.path.join(frozen, "scripts", "gate", "lib_split_family_loaded.py"))
    library = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(library)

    # 门禁阶段 lib/ 拆法：命名那一半按仓内相对路径比，绝对路径认不认
    stage_root = os.path.join(base, "gate-stage")
    write(os.path.join(stage_root, ".claude", "gate.d", "stagex.sh"), 'STAGE_DIRECTORY="$(cd "$(dirname "$0")" && pwd)"\nsource "$STAGE_DIRECTORY/lib/stagex-cells.sh"\n')
    write(os.path.join(stage_root, ".claude", "gate.d", "lib", "stagex-cells.sh"), "cell_one() { :; }\n")
    relative_entry, relative_part = ".claude/gate.d/stagex.sh", ".claude/gate.d/lib/stagex-cells.sh"
    absolute_entry, absolute_part = os.path.join(stage_root, relative_entry), os.path.join(stage_root, relative_part)
    print(f"PROBE_SPLIT_FAMILY case=gate-stage-naming in_family_relative={library.naming.in_family(relative_entry, relative_part)} "
          f"in_family_absolute={library.naming.in_family(absolute_entry, absolute_part)}", flush=True)
    start = os.getcwd()
    os.chdir(stage_root)
    try:
        relative_family = names(library.loaded_family(relative_entry, [relative_part])[1:])
    finally:
        os.chdir(start)
    print(f"PROBE_SPLIT_FAMILY case=gate-stage-loaded relative_with_lib_candidate={relative_family} "
          f"absolute_with_lib_candidate={names(library.loaded_family(absolute_entry, [absolute_part])[1:])} "
          f"absolute_with_sibling_files={names(library.loaded_family(absolute_entry, library.sibling_files(absolute_entry))[1:])}", flush=True)

    shadow = {"lib_demo_shadow.py": "def commands_at_command_position(text):\n    return [text]\n"}
    part = {"lib_demo_part.py": "def shell_tokens(text):\n    return text.split()\n"}
    cases = [
        ("quoted-name-in-comment", "# 别处留下的 'lib_demo_shadow.py' 是影子，不归这一份\nimport os\n", shadow),
        ("quoted-name-in-message", 'import sys\nMESSAGE = "不要再建 lib_demo 的影子（\'lib_demo_shadow.py\'）"\n', shadow),
        ("import-line-in-docstring", '"""用法说明：\n    import lib_demo_shadow   # 别这样写\n"""\nimport os\n', shadow),
        ("from-import-control", "from lib_demo_part import shell_tokens\n", part),
        ("relative-from-import", "from . import lib_demo_part\n", part),
        ("import-module-by-name", 'import importlib\nlib_demo_part = importlib.import_module("lib_demo_part")\n', part),
        ("from-dotted-import", "from lib_demo_part.inner import shell_tokens\n", part),
    ]
    for case, entry_text, extra in cases:
        print(f"PROBE_SPLIT_FAMILY case={case} family={python_case(library, base, case, entry_text, extra)}", flush=True)

    for case, line in (("shell-source-dirname", 'source "$(dirname "${BASH_SOURCE[0]}")/demo-steps.sh"\n'),
                       ("shell-source-variable-control", 'here="$(cd "$(dirname "$0")" && pwd)"\nsource "$here/demo-steps.sh"\n')):
        directory = os.path.join(base, case)
        write(os.path.join(directory, "demo.sh"), "#!/usr/bin/env bash\n" + line)
        write(os.path.join(directory, "demo-steps.sh"), "step() { :; }\n")
        entry = os.path.join(directory, "demo.sh")
        print(f"PROBE_SPLIT_FAMILY case={case} family={names(library.loaded_family(entry, library.sibling_files(entry))[1:])}", flush=True)

    task_entry = os.path.join(frozen, "task-server", "task_server.py")
    task_family = names(library.loaded_family(task_entry, [path for path in library.sibling_files(task_entry) if path.endswith(".py")])[1:])
    selftests = [name for name in task_family if name.startswith("task_server_selftest")]
    print(f"PROBE_SPLIT_FAMILY case=task-server-entry members={len(task_family)} selftest_members={len(selftests)} family={task_family}", flush=True)
    for module in ("lib_shell_words.py", "lib_project_code_runs.py"):
        entry = os.path.join(frozen, ".claude", "hooks", module)
        print(f"PROBE_SPLIT_FAMILY case=hooks-{module} family={names(library.loaded_family(entry, library.sibling_files(entry))[1:])}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
