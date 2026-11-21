#!/usr/bin/env python3
"""登记行复核（只读，不改任何文件）：锚串命中数、理由里点名的东西在不在、点名的对子对不对。
用法：registration_rows.py <仓根>
每一项一行：name=registration_rows item=<项> got=<读到> expected=<预期>，预期不同的行末加 verdict=DIFFERENT；只读数不比的写 expected=-。
退出码：0 全部与预期相同；1 有不同；2 用法错或要读的文件不在。预期是写这份模型那一刻读代码推出的（推的），与第一轮判决里的改法比对用。"""
import os
import sys

if len(sys.argv) != 2:
    raise SystemExit("用法：registration_rows.py <仓根>")
ROOT = os.path.realpath(sys.argv[1])
DIFFERENT = []


def text_of(relative):
    path = os.path.join(ROOT, relative)
    if not os.path.isfile(path):
        print(f"✗ 要读的文件不在：{relative}", file=sys.stderr)
        raise SystemExit(2)
    with open(path, encoding="utf-8") as handle:
        return handle.read()


def hit_lines(relative, anchor):
    return sum(1 for line in text_of(relative).splitlines() if anchor in line)


def show(item, got, expected="-"):
    flag = ""
    if expected != "-" and str(got) != str(expected):
        flag = " verdict=DIFFERENT"
        DIFFERENT.append(item)
    print(f"name=registration_rows item={item} got={got} expected={expected}{flag}")


def rows_of(relative, separator):
    """(行号, 路径部分, 理由部分) 的表；跳过空行与 # 起头的行。"""
    rows = []
    for number, line in enumerate(text_of(relative).splitlines(), 1):
        if not line.strip() or line.startswith("#"):
            continue
        head, _, reason = line.partition(separator)
        rows.append((number, head, reason))
    return rows


# 一、repository-copy-closure-exempt.tsv：写法 <路径>:<锚串>  # 理由
EXEMPT = ".claude/gate.d/repository-copy-closure-exempt.tsv"
exempt_rows = rows_of(EXEMPT, "  # ")
zero_hit = []
for number, head, _ in exempt_rows:
    path, _, anchor = head.partition(":")
    if not os.path.isfile(os.path.join(ROOT, path)) or hit_lines(path, anchor) == 0:
        zero_hit.append(number)
show("exempt-rows-total", len(exempt_rows))
show("exempt-rows-with-zero-anchor-hits", len(zero_hit), 0)
unit_rows = [row for row in exempt_rows if row[1].startswith(".claude/gate.d/lib/code-tooling-unit-verdict-and-entry-cells.sh:")]
show("exempt-unit-graph-row-count", len(unit_rows), 1)
if unit_rows:
    number, head, reason = unit_rows[0]
    path, _, anchor = head.partition(":")
    show("exempt-unit-graph-row-line", number)
    show("exempt-unit-graph-row-anchor-hit-lines", hit_lines(path, anchor), 1)
    show("exempt-unit-graph-reason-names-new-file", int("scripts/gate/lib_function_units_unit_graph.py" in reason), 1)
    graph = text_of("scripts/gate/lib_function_units_unit_graph.py")
    show("unit-graph-defines-build_unit_graph", int("def build_unit_graph(" in graph), 1)
    show("unit-graph-copies-the-crate-tree", int('shutil.copytree(source, os.path.join(workspace, "crates"' in graph), 1)
    show("unit-graph-copies-the-thread-registry", int('"heavy-thread-default"' in graph), 1)

# 二、memory-bounded-parallelism.tsv：写法 <入口><制表符><理由>。理由里把三格与它们的弄坏开关对在一起，下面按代码里的对子核
MEMORY = ".claude/memory-bounded-parallelism.tsv"
memory_rows = [row for row in rows_of(MEMORY, "\t") if row[1] == ".claude/gate.d/code-tooling.sh"]
show("memory-code-tooling-row-count", len(memory_rows), 1)
reason = memory_rows[0][2] if memory_rows else ""
STAGE_LIBRARY = ".claude/gate.d/lib/code-tooling-stage-break-switch-replay-cells.sh"
SHAPE_LIBRARY = ".claude/gate.d/lib/code-tooling-copy-closure-and-selftest-shape-cells.sh"
BREAK_LIBRARY = ".claude/gate.d/lib/code-tooling-break-switch-replay-cells.sh"
pairing = "break-switch-replay、selftest-cells-alone 两格只在弄坏开关 replay-jobs-by-memory、cells-alone-jobs-by-memory 下用它"
show("code-pairs-two-cells-with-two-switches", hit_lines(STAGE_LIBRARY, pairing), 1)
show("code-defines-stage-replay-jobs-by-memory", int(hit_lines(STAGE_LIBRARY, "stage-replay-jobs-by-memory") >= 1), 1)
show("code-defines-cells-alone-jobs-by-memory-in-selftest-cells-alone-library", int(hit_lines(SHAPE_LIBRARY, "cells-alone-jobs-by-memory") >= 1), 1)
show("code-defines-replay-jobs-by-memory-in-break-switch-replay-library", int(hit_lines(BREAK_LIBRARY, "replay-jobs-by-memory") >= 1), 1)
# 理由里点名的格与开关：按代码里的对子，selftest-cells-alone 与 stage-replay-jobs-by-memory 应当出现，stage-break-switch-replay 配 replay-jobs-by-memory 是错配
show("reason-names-break-switch-replay-cell", int("break-switch-replay" in reason), 1)
show("reason-names-stage-break-switch-replay-cell", int("stage-break-switch-replay" in reason), 1)
show("reason-names-selftest-cells-alone-cell", int("selftest-cells-alone" in reason), 0)
show("reason-names-replay-jobs-by-memory", int("replay-jobs-by-memory" in reason), 1)
show("reason-names-cells-alone-jobs-by-memory", int("cells-alone-jobs-by-memory" in reason), 1)
show("reason-names-stage-replay-jobs-by-memory", int("stage-replay-jobs-by-memory" in reason.replace("cells-alone-jobs-by-memory", "")), 0)
big_selftests = "SELFTEST_RUNNERS_MEMORY_BOUND"
show("reason-names-selftest-runners-memory-bound-and-it-exists", int(big_selftests in reason and hit_lines(".claude/gate.d/lib/code-tooling-selftest-cells.sh", big_selftests) >= 1), 1)

# 三、preflight-exclude：e203 拆出的四份模块各有一行、没有自己的入口；入口自己有没有准入与运行条件
PREFLIGHT = ".claude/preflight-exclude"
exclude_rows = rows_of(PREFLIGHT, "  # ")
prefix = "scripts/experiment-apparatus/e203_admission_scan_cost_counters"
modules = ["applying_edits_and_copying_the_tree", "insertion_edits", "row_table_and_module_template", "selftest"]
for part in modules:
    path = f"{prefix}_{part}.py"
    own = [row for row in exclude_rows if row[1] == path]
    show(f"exclude-own-row-{part}", len(own), 1)
    show(f"module-file-exists-{part}", int(os.path.isfile(os.path.join(ROOT, path))), 1)
    show(f"module-main-guard-count-{part}", text_of(path).count("__main__"), 0)
    show(f"module-row-reason-says-entry-judges-conditions-{part}", int(bool(own) and "准入与运行条件由入口判" in own[0][2]), 1)
entry = f"{prefix}.py"
entry_text = text_of(entry)
show("entry-own-exclude-row", len([row for row in exclude_rows if row[1] == entry]), 0)
header = []
for line in entry_text.splitlines():
    if line.startswith("#") or not line.strip():
        header.append(line)
        continue
    break
show("entry-comment-header-has-admission-line", int(any(line.startswith("# admission") for line in header)), 0)
show("entry-calls-preflight", int("preflight" in entry_text), 0)

# 四、README 第 7 行：给的列法列出多少个目录、其中有 main.rs 的多少个
apparatus = os.path.join(ROOT, "scripts/experiment-apparatus")
directories = sorted(name for name in os.listdir(apparatus) if os.path.isdir(os.path.join(apparatus, name)))
with_main = [name for name in directories if os.path.isfile(os.path.join(apparatus, name, "main.rs"))]
readme_line = text_of("scripts/experiment-apparatus/README.md").splitlines()[6]
show("readme-line-7-names-the-ls-command", int("ls -d scripts/experiment-apparatus/*/" in readme_line), 1)
show("apparatus-directories-from-that-ls", len(directories))
show("apparatus-directories-with-main-rs", len(with_main))
show("apparatus-directories-without-main-rs", len(directories) - len(with_main))
print(f"name=registration_rows end different={len(DIFFERENT)}" + (f" items={','.join(DIFFERENT)}" if DIFFERENT else ""))
raise SystemExit(1 if DIFFERENT else 0)
