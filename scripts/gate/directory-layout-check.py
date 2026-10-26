#!/usr/bin/env python3
# admission: always 判的是此刻树上的目录形态，几秒跑完；挪了文件、加了目录就要现判
# run-condition: none 只读仓里的文件清单（git ls-files，不在 git 仓里时走目录树），不起别的进程
"""判 scripts/ 与 research/ 的目录形态（.claude/rules/directory-layout.md）。

用法：
    directory-layout-check.py [根]          # 不给就判这份脚本所在的仓
    directory-layout-check.py --selftest    # DIRECTORY_LAYOUT_CHECK_BREAK=<开关> 时必须红在点名的格

判法（文件清单：根是 git 仓时取 git 跟踪的与未跟踪而未忽略的；不是时走目录树，跳过 .git、target、target-*、__pycache__）：
  ① research/ 顶层只有 prompts/、results/ 两个子目录与 README.md，README.md 要在。
  ② scripts/ 顶层只有 README.md 与子目录，README.md 要在；每个子目录顶层有 README.md。
  ③ scripts/ 下的文件最深在 scripts/<子目录>/<装置目录>/<文件>。
  ④ scripts/ 下没有产物与日志类的文件（后缀 .out、.log）。
  research/ 与 scripts/ 都不在时这一格无对象可判。

退出码：0 都对；1 有不对的（逐条列出）；2 用法错；77 research/ 与 scripts/ 都不在。

弄坏开关 DIRECTORY_LAYOUT_CHECK_BREAK=<开关>（只给 --selftest 用，--selftest 之外不认）：
  allow-research-extra  research/ 顶层多出来的不报：research-top 那一格该红
  skip-readme           说明文件缺了不报：readme 那一格该红
  ignore-depth          层级不判：depth 那一格该红
  allow-results         产物与日志类的文件不报：no-results 那一格该红
  allow-scripts-top     scripts/ 顶层多出来的文件不报：scripts-top 那一格该红
"""
import os
import re
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

RESEARCH_SUBDIRECTORIES = ("prompts", "results")
DESCRIPTION_FILE = "README.md"
DEEPEST_COMPONENTS_BELOW_SCRIPTS = 3          # scripts/<子目录>/<装置目录>/<文件>
RESULT_SUFFIXES = (".out", ".log")
SKIPPED_DIRECTORY = re.compile(r"^(\.git|target|target-.*|__pycache__)$")
BREAKS = ("allow-research-extra", "skip-readme", "ignore-depth", "allow-results", "allow-scripts-top")
broken = set()


def listed_files(root):
    """根下的文件（相对根、用 / 分隔）：git 仓里取跟踪的与未跟踪而未忽略的，否则走目录树。"""
    inside = subprocess.run(["git", "-C", root, "rev-parse", "--show-toplevel"], capture_output=True, text=True)
    if inside.returncode == 0 and os.path.realpath(inside.stdout.strip()) == os.path.realpath(root):
        listed = subprocess.run(["git", "-c", "core.quotepath=false", "-C", root, "ls-files", "-co", "--exclude-standard", "-z"],
                                capture_output=True, text=True, check=True).stdout
        return sorted(path for path in listed.split("\0") if path and os.path.lexists(os.path.join(root, path)))
    found = []
    for current, subdirectories, files in os.walk(root):
        subdirectories[:] = sorted(name for name in subdirectories if not SKIPPED_DIRECTORY.match(name))
        for name in sorted(files):
            found.append(os.path.relpath(os.path.join(current, name), root).replace(os.sep, "/"))
    return found


def layout_problems(files):
    """(判了几份, [(格, 一句)])：按文件头判法 ① 到 ④。"""
    problems = []
    research = [path.split("/")[1:] for path in files if path.startswith("research/")]
    scripts = [path.split("/")[1:] for path in files if path.startswith("scripts/")]
    if research:
        extra = sorted({parts[0] for parts in research
                        if not (len(parts) == 1 and parts[0] == DESCRIPTION_FILE) and not (len(parts) > 1 and parts[0] in RESEARCH_SUBDIRECTORIES)})
        if extra and "allow-research-extra" not in broken:
            problems.append(("research-top", f"research/ 顶层多出 {'、'.join(extra)}（只许 prompts/、results/ 与 {DESCRIPTION_FILE}）"))
        if [DESCRIPTION_FILE] not in research and "skip-readme" not in broken:
            problems.append(("readme", f"research/{DESCRIPTION_FILE} 不在"))
    if scripts:
        top_files = sorted(parts[0] for parts in scripts if len(parts) == 1 and parts[0] != DESCRIPTION_FILE)
        if top_files and "allow-scripts-top" not in broken:
            problems.append(("scripts-top", f"scripts/ 顶层有文件 {'、'.join(top_files)}（只许 {DESCRIPTION_FILE} 与子目录）"))
        if "skip-readme" not in broken:
            if [DESCRIPTION_FILE] not in scripts:
                problems.append(("readme", f"scripts/{DESCRIPTION_FILE} 不在"))
            for category in sorted({parts[0] for parts in scripts if len(parts) > 1}):
                if [category, DESCRIPTION_FILE] not in scripts:
                    problems.append(("readme", f"scripts/{category}/{DESCRIPTION_FILE} 不在"))
        if "ignore-depth" not in broken:
            for parts in scripts:
                if len(parts) > DEEPEST_COMPONENTS_BELOW_SCRIPTS:
                    problems.append(("depth", f"scripts/{'/'.join(parts)} 在 scripts/ 下第 {len(parts)} 层（最深 {DEEPEST_COMPONENTS_BELOW_SCRIPTS} 层）"))
        if "allow-results" not in broken:
            for parts in scripts:
                if parts[-1].endswith(RESULT_SUFFIXES):
                    problems.append(("no-results", f"scripts/{'/'.join(parts)} 是产物或日志类的文件"))
    return len(research) + len(scripts), problems


def judge(root):
    files = listed_files(root)
    judged, problems = layout_problems(files)
    if judged == 0:
        print("  ! 本次无对象可判：research/ 与 scripts/ 下一份文件都没有")
        return 77
    if problems:
        for cell, message in problems:
            print(f"  ✗ {cell}：{message}")  # gate-lint:detail
        print(f"  ✗ 判了 research/ 与 scripts/ 下 {judged} 份文件，{len(problems)} 处不合 .claude/rules/directory-layout.md（逐条列在上面）")
        print("     → 怎么办：脚本挪进 scripts/ 下用途对得上的子目录并补那一份 README.md 的表；研究目录只留 prompts/ 与 results/；"
              "产物写进 research/results/，本机运行状态放 ~/.local/share/singlefs/；太深的拆成平级的装置目录")
        return 1
    print(f"  ✓ 判了 research/ 与 scripts/ 下 {judged} 份文件：顶层、说明文件、层级、没有产物与日志，都合 .claude/rules/directory-layout.md")
    return 0


def run_selftest():
    cells = []
    green_files = ["research/README.md", "research/prompts/r1-body.md", "research/results/e1.out",
                   "scripts/README.md", "scripts/kb/README.md", "scripts/kb/tool.py",
                   "scripts/experiment-apparatus/README.md", "scripts/experiment-apparatus/e9_device/main.rs"]
    judged, problems = layout_problems(green_files)
    cells.append(("green", judged == 8 and problems == [], f"绿样本 8 份应当一处都不报，实判 {judged} 份、报 {problems}"))
    cases = [
        ("research-top", green_files + ["research/scripts/tool.sh"], "多出 scripts"),
        ("readme", [path for path in green_files if path != "scripts/kb/README.md"], "scripts/kb/README.md 不在"),
        ("depth", green_files + ["scripts/experiment-apparatus/e9_device/deeper/x.rs"], "第 4 层"),
        ("no-results", green_files + ["scripts/hosts/README.md", "scripts/hosts/run.log"], "run.log"),
        ("scripts-top", green_files + ["scripts/stray.sh"], "stray.sh"),
    ]
    for cell, files, want in cases:
        _, problems = layout_problems(files)
        reported = [message for name, message in problems if name == cell]
        cells.append((cell, any(want in message for message in reported), f"该在 {cell} 格报出含「{want}」的一句，实报 {problems}"))
    with tempfile.TemporaryDirectory() as work:
        for path in green_files + ["scripts/experiment-apparatus/target/release/build.log"]:
            full = os.path.join(work, path)
            os.makedirs(os.path.dirname(full), exist_ok=True)
            open(full, "w", encoding="utf-8").close()
        walked = listed_files(work)
        cells.append(("walk-skips-target", walked == sorted(green_files), f"不在 git 仓里走目录树应当跳过 target/，实列 {walked}"))
    failed = [cell for cell in cells if not cell[1]]
    for name, passed, message in cells:
        if not passed:
            print(f"  ✗ 自证 {name}：{message}")  # gate-lint:detail
    if failed:
        print(f"  ✗ --selftest：{len(cells)} 格里 {len(failed)} 格不对")
        print("     → 怎么办：照上面点名的格查判法；弄坏开关打开时红在点名的格是对的")
        return 1
    print(f"  ✓ --selftest：{len(cells)} 格都对（绿样本不报、research/ 顶层多出、缺说明文件、太深、产物与日志、scripts/ 顶层文件、走目录树跳过编译目录）")
    return 0


def switch_wheel():
    """逐个打开文件头登记的弄坏开关，各自要红在点名的格。"""
    expected = {"allow-research-extra": "research-top", "skip-readme": "readme", "ignore-depth": "depth",
                "allow-results": "no-results", "allow-scripts-top": "scripts-top"}
    failures = 0
    for switch, cell in expected.items():
        completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--selftest-inner"], capture_output=True, text=True,
                                   env=dict(os.environ, DIRECTORY_LAYOUT_CHECK_BREAK=switch))
        red_cells = re.findall(r"✗ 自证 ([a-z-]+)：", completed.stdout)
        if completed.returncode == 1 and cell in red_cells:
            print(f"  · 开关轮：{switch} 红在「{'、'.join(red_cells)}」，含点名的 {cell}")
        else:
            failures += 1
            print(f"  ✗ 开关轮：{switch} 打开后退 {completed.returncode}、红在「{'、'.join(red_cells) or '（没有）'}」，点名的是 {cell}")  # gate-lint:detail
    if failures:
        print(f"  ✗ 开关轮 {len(expected)} 个开关里 {failures} 个没红在点名的格")
        print("     → 怎么办：改那一处判法，或照实改文件头的开关表")
        return 1
    print(f"  ✓ 开关轮：{len(expected)} 个开关逐个打开，各红在点名的格")
    return 0


def main(arguments):
    if arguments[:1] in (["--selftest"], ["--selftest-inner"]):
        if arguments[0] == "--selftest-inner":
            broken.update(item for item in os.environ.get("DIRECTORY_LAYOUT_CHECK_BREAK", "").split(",") if item in BREAKS)
            return run_selftest()
        status = run_selftest()
        return status or switch_wheel()
    if len(arguments) > 1 or any(argument.startswith("-") for argument in arguments):
        print(f"  ✗ 认不得的参数：{' '.join(arguments)}")
        print("     → 怎么办：用法 directory-layout-check.py [根] 或 --selftest")
        return 2
    root = os.path.realpath(arguments[0]) if arguments else REPOSITORY_ROOT
    if not os.path.isdir(root):
        print(f"  ✗ 根不在：{root}")
        print("     → 怎么办：给仓根，或不给参数判这份脚本所在的仓")
        return 2
    return judge(root)


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
