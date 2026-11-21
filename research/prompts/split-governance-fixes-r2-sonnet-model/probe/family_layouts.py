#!/usr/bin/env python3
"""模型：认拆分族的三种定义（命名与载入的交集 intersection、只按命名 name-only、只按载入 loaded-only）在各种布局下各认出什么。
定义取自 scripts/gate/lib_split_family_loaded.py（按路径载入，只读）：loaded_family(入口, 候选, defects) 的 defects 空集是交集，NAME_ONLY 只按命名，LOADED_ONLY 只按载入。
四类输出，行首标记：
  LAYOUT      造的布局（每个布局一个入口加几份文件，写在 <工作目录>/layouts/ 下）：三种定义各认出的族与「真是这一份拆出来的」那一组（truth）比，多认 over、少认 under
  FUNCTIONS   同一批布局与仓里两份共用模块：三种定义下「函数名从族里取」得到的名字集（agent-write-scope 格的 function_home 那一步）
  REAL        仓里 scripts/、task-server/、.claude/hooks/ 的入口：三种定义认出的族不同的入口，与「只在注释、文档串里被点名」「名字像又在正文里出现、却没被认成载入」两种线索
  SUMMARY     每种定义在造的布局里恰好认对几个
用法：python3 family_layouts.py --repo <仓根> --work <新的工作目录>
退出码：0 跑完；2 用法错或库载不进。读数只看这四种行，不做判。"""
import argparse
import ast
import importlib.util
import os
import shutil
import sys

SCAN_DIRECTORIES = ("scripts/gate", "scripts/mutation", "scripts/hosts", "scripts/crash", "scripts/prune-server", "scripts/agents", "scripts/kb", "task-server", ".claude/hooks")
SHARED_MODULES = (".claude/hooks/lib_shell_words.py", ".claude/hooks/lib_project_code_runs.py")


def load_library(repository):
    path = os.path.join(repository, "scripts", "gate", "lib_split_family_loaded.py")
    specification = importlib.util.spec_from_file_location("lib_split_family_loaded_for_family_layouts", path)
    library = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(library)
    return library


def definition_defects(library):
    return {"intersection": frozenset(), "name-only": frozenset({library.NAME_ONLY}), "loaded-only": frozenset({library.LOADED_ONLY})}


def family_paths(library, entry, defects, pool=None):
    candidates = library.sibling_files(entry) if pool is None else pool
    return library.loaded_family(entry, candidates, defects)[1:]


def function_names(paths):
    names = set()
    for path in paths:
        try:
            with open(path, encoding="utf-8") as handle:
                tree = ast.parse(handle.read())
        except (OSError, SyntaxError, ValueError):
            continue
        names |= {node.name for node in tree.body if isinstance(node, ast.FunctionDef)}
    return sorted(names)


# (布局名, 入口文件名, {文件名: 文本}, 真是这一份拆出来的文件名, 它在判什么)
LAYOUTS = [
    ("dead-name-like-file", "e.py", {"e.py": "import os\n", "e_shadow.py": "def shadow_function():\n    return 1\n"}, set(), "名字像部分文件而没人载入"),
    ("shared-library-imported", "e.py", {"e.py": "import shared_tools\n", "shared_tools.py": "def shared_function():\n    return 1\n"}, set(), "被 import 的共用库，不是它拆出来的"),
    ("docstring-quotes-name-like-file", "e.py", {"e.py": "'''说明里写着 \"e_old_part.py\" 这个名字，没有载入它。'''\nimport os\n", "e_old_part.py": "def old_function():\n    return 1\n"}, set(),
     "只在文档串里被点名的名字像的文件"),
    ("computed-name-import", "e.py", {"e.py": "import importlib\nPART = importlib.import_module('e_' + 'computed')\ndef entry_function():\n    return PART\n",
                                     "e_computed.py": "def computed_function():\n    return 1\n"}, {"e_computed.py"}, "按拼出来的名字载入的真部分文件"),
    ("relative-import-from-dot", "e.py", {"e.py": "from . import e_relative\n", "e_relative.py": "def relative_function():\n    return 1\n"}, {"e_relative.py"}, "from . import 载入的真部分文件"),
    ("relative-import-dotted", "e.py", {"e.py": "from .e_dotted import helper\n", "e_dotted.py": "def helper():\n    return 1\n"}, {"e_dotted.py"}, "from .名字 import 载入的真部分文件"),
    ("plain-imports-and-nesting", "e.py", {"e.py": "import e_one\nfrom e_two import helper\n", "e_one.py": "def one():\n    return 1\n", "e_two.py": "import e_deeper\ndef helper():\n    return 2\n",
                                          "e_deeper.py": "def deeper():\n    return 3\n"}, {"e_one.py", "e_two.py", "e_deeper.py"}, "import 与部分文件再载入的部分文件"),
    ("quoted-part-name-tuple", "e.py", {"e.py": "PART_NAMES = (\"e_a.py\", \"e_b.py\")\n", "e_a.py": "def a():\n    return 1\n", "e_b.py": "def b():\n    return 2\n"}, {"e_a.py", "e_b.py"}, "写死的文件名单 exec"),
    ("part-in-subdirectory", "e.py", {"e.py": "import sys\nsys.path.insert(0, 'parts')\nimport e_sub\n", "parts/e_sub.py": "def sub():\n    return 1\n"}, {"e_sub.py"}, "放在子目录里的真部分文件"),
    ("hyphen-part-of-python-entry", "e.py", {"e.py": "exec(open('e-hyphen.py').read())\n", "e-hyphen.py": "def hyphen():\n    return 1\n"}, {"e-hyphen.py"}, "点号入口按横线命名的真部分文件"),
]


def build_layout(root, name, files):
    directory = os.path.join(root, name)
    if os.path.exists(directory):
        shutil.rmtree(directory)
    for relative, text in files.items():
        path = os.path.join(directory, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
    return directory


def run_layouts(library, root):
    exact = {name: 0 for name in definition_defects(library)}
    original = os.getcwd()
    for name, entry, files, truth, about in LAYOUTS:
        os.chdir(build_layout(root, name, files))
        try:
            for definition, defects in definition_defects(library).items():
                found = {os.path.basename(path) for path in family_paths(library, entry, defects)}
                over, under = sorted(found - truth), sorted(truth - found)
                exact[definition] += not over and not under
                print(f"LAYOUT {name} definition={definition} members={sorted(found)} truth={sorted(truth)} over={over} under={under}  # {about}")
                names = function_names([entry] + [path for path in family_paths(library, entry, defects)])
                print(f"FUNCTIONS layout={name} definition={definition} names={names}")
        finally:
            os.chdir(original)
    return exact


def run_gate_stage_paths(library, root):
    """门禁阶段入口 source 一份 lib/ 下的 <阶段>-<部分>.sh：候选由调用方交（sibling_files 只列入口同目录）；入口与候选按仓内相对路径给、按绝对路径给，各定义认出什么。"""
    directory = build_layout(root, "gate-stage-lib-parts", {".claude/gate.d/stage.sh": 'source "$DIR/lib/stage-cells.sh"\n', ".claude/gate.d/lib/stage-cells.sh": "cells() { :; }\n"})
    original = os.getcwd()
    os.chdir(directory)
    try:
        for style, entry in (("relative", ".claude/gate.d/stage.sh"), ("absolute", os.path.join(directory, ".claude/gate.d/stage.sh"))):
            pool = [os.path.join(os.path.dirname(entry), "lib", "stage-cells.sh")]
            for definition, defects in definition_defects(library).items():
                found = [os.path.basename(path) for path in family_paths(library, entry, defects, pool)]
                print(f"LAYOUT gate-stage-lib-parts-{style} definition={definition} members={found} truth=['stage-cells.sh']")
    finally:
        os.chdir(original)


def code_only_text(path):
    """注释行与文档串那几行换成空行之后的文本：载入的写法只在这些地方出现的，不是真载入。"""
    with open(path, encoding="utf-8", errors="replace") as handle:
        text = handle.read()
    lines, blank = text.split("\n"), set()
    if path.endswith(".py"):
        try:
            tree = ast.parse(text)
        except (SyntaxError, ValueError):
            tree = None
        for node in ast.walk(tree) if tree is not None else ():
            if isinstance(node, (ast.Module, ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)) and node.body:
                first = node.body[0]
                if isinstance(first, ast.Expr) and isinstance(first.value, ast.Constant) and isinstance(first.value.value, str):
                    blank.update(range(first.lineno - 1, first.end_lineno))
    blank.update(index for index, line in enumerate(lines) if line.lstrip().startswith("#"))
    return "\n".join("" if index in blank else line for index, line in enumerate(lines))


def run_real(library, repository):
    original = os.getcwd()
    os.chdir(repository)
    totals = {"entries": 0, "name-only-differs": 0, "loaded-only-differs": 0, "doc-only-mentions": 0, "mentioned-not-loaded": 0}
    try:
        for directory in SCAN_DIRECTORIES:
            if not os.path.isdir(directory):
                continue
            for name in sorted(os.listdir(directory)):
                entry = os.path.join(directory, name)
                if not name.endswith((".py", ".sh")) or not os.path.isfile(entry):
                    continue
                totals["entries"] += 1
                defects = definition_defects(library)
                found = {definition: [os.path.basename(path) for path in family_paths(library, entry, flags)] for definition, flags in defects.items()}
                members = family_paths(library, entry, frozenset())
                name_only_extra = sorted(set(found["name-only"]) - set(found["intersection"]))
                loaded_only_extra = sorted(set(found["loaded-only"]) - set(found["intersection"]))
                totals["name-only-differs"] += bool(name_only_extra)
                totals["loaded-only-differs"] += bool(loaded_only_extra)
                loaders = [entry] + members
                code_names = set().union(*(library.loaded_file_names(code_only_text(path)) for path in loaders))
                for path in members:
                    if os.path.basename(path) not in code_names:
                        totals["doc-only-mentions"] += 1
                        print(f"REAL doc-only-mention entry={entry} member={os.path.basename(path)}")
                texts = ""
                for path in loaders:
                    with open(path, encoding="utf-8", errors="replace") as handle:
                        texts += handle.read()
                for extra in name_only_extra:
                    if extra[:-3] in texts:
                        totals["mentioned-not-loaded"] += 1
                        print(f"REAL name-like-mentioned-but-not-loaded entry={entry} member={extra}")
                if name_only_extra or loaded_only_extra:
                    print(f"REAL differs entry={entry} intersection={len(found['intersection'])} name-only-extra={name_only_extra} loaded-only-extra={loaded_only_extra}")
        for relative in SHARED_MODULES:
            for definition, flags in definition_defects(library).items():
                paths = [relative] + family_paths(library, relative, flags)
                print(f"FUNCTIONS real={relative} definition={definition} files={len(paths)} functions={len(function_names(paths))}")
    finally:
        os.chdir(original)
    print("SUMMARY real " + " ".join(f"{key}={value}" for key, value in totals.items()))


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--repo", required=True)
    parser.add_argument("--work", required=True)
    options = parser.parse_args()
    repository, work = os.path.realpath(options.repo), os.path.realpath(options.work)
    try:
        library = load_library(repository)
    except (OSError, ImportError, SyntaxError) as error:
        print(f"✗ 载不进 {repository}/scripts/gate/lib_split_family_loaded.py：{type(error).__name__}：{error}", file=sys.stderr)
        return 2
    os.makedirs(work, exist_ok=True)
    exact = run_layouts(library, os.path.join(work, "layouts"))
    run_gate_stage_paths(library, os.path.join(work, "layouts"))
    print("SUMMARY layouts " + " ".join(f"{definition}-exact={count}-of-{len(LAYOUTS)}" for definition, count in exact.items()))
    run_real(library, repository)
    print("PROBE_DONE")
    return 0


if __name__ == "__main__":
    sys.exit(main())
