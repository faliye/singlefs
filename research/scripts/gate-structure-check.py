#!/usr/bin/env python3
# admission: always 判的是此刻门禁目录里每道阶段的文件头与它们对 --list、--check 的反应，阶段随时在改
# run-condition: command bash
"""门禁阶段的结构（.claude/rules/verification.md「门禁的结构」）：每道阶段都能全跑，也能逐格点名跑。

对 <仓根>/.claude/gate.d/ 顶层每个 *.sh（lib-*.sh 是共用库，不判）逐道判三样：
  table    文件头（#! 之后、第一行代码之前的注释块）有格名表：每格一行 `# gate-cell: <格名> <判什么的一句>`，格名不重复
  list     `bash <阶段> <仓根> --list --force` 退 0，stdout 逐行「格名<制表符>判什么」，与格名表逐格相同、次序相同；
           跑的时候没起重活：PATH 最前面放一排替身（cargo、rustc、rustup、qemu-system-x86_64、qemu-system-aarch64、qemu-img、herd7、fio），
           哪个替身被调了就判红
  unknown  `bash <阶段> <仓根> --check gate-structure-check-no-such-cell --force` 退 2，同样不许起重活
  --force 是交给阶段的 preflight 的：只问结构，不因为「输入没变」被拒。每次起阶段限时 LIST_TIMEOUT_SECONDS 秒。
  文件头没有格名表的阶段不起它（还没改用 lib/stage-cells.sh 的阶段不认 --list，起它就是全跑）。
判不了的：按绝对路径起的重活（target/ 下的装置二进制、写死路径的 cargo）替身拦不到；格的函数判得对不对；
  格名表里的「判什么」写得准不准。

用法：
    gate-structure-check.py [仓根]      不给就取这份脚本往上两级
    gate-structure-check.py --selftest  GATE_STRUCTURE_CHECK_BREAK=<项> 时必须判红：
        table-ignored    文件头没有格名表的阶段悄悄跳过          11-no-table.sh 那一条找不到
        list-ignored     不比 --list 与格名表                     12-names.sh、13-title.sh 那两条找不到
        heavy-ignored    不看替身有没有被调                       14-heavy.sh 那一条找不到
        unknown-ignored  不判 --check 写错格名的退出码            15-lenient.sh 那一条找不到
退出码：0 每道都对；1 有不对的（逐道列出）；2 用法错；77 门禁目录里一道阶段都没有。
"""
import os
import re
import shutil
import stat
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

BROKEN = os.environ.get("GATE_STRUCTURE_CHECK_BREAK", "")
LIST_TIMEOUT_SECONDS = 60
UNKNOWN_CELL = "gate-structure-check-no-such-cell"
HEAVY_TOOLS = ("cargo", "rustc", "rustup", "qemu-system-x86_64", "qemu-system-aarch64", "qemu-img", "herd7", "fio")
CELL_LINE = re.compile(r"^#\s*gate-cell:\s*(\S+)\s+(\S.*?)\s*$")
CELL_NAME = re.compile(r"^[a-z0-9][a-z0-9-]*$")
CLEARED_ENVIRONMENT = ("GATE_NOT_RUN_FILE", "GATE_BASE", "GATE_STAGED_FROM", "GATE_DIFF_BASE", "GATE_IN_STAGE")
SHIM = """#!/bin/sh
printf '%s\\n' "${0##*/}" >> "$GATE_STRUCTURE_CHECK_SHIM_TRACE"
echo "gate-structure-check：替身 ${0##*/} 被调了，按起了重活记" >&2
exit 97
"""


def header_cells(path):
    """文件头注释块里的格名表：[(行号, 格名, 判什么)]。"""
    cells = []
    with open(path, encoding="utf-8", errors="replace") as handle:
        for number, line in enumerate(handle, 1):
            line = line.rstrip("\n")
            if number == 1 and line.startswith("#!"):
                continue
            if line.strip() and not line.lstrip().startswith("#"):
                break
            match = CELL_LINE.match(line.strip())
            if match:
                cells.append((number, match.group(1), match.group(2)))
    return cells


def stage_files(root):
    directory = os.path.join(root, ".claude", "gate.d")
    if not os.path.isdir(directory):
        return []
    return sorted(name for name in os.listdir(directory)
                  if name.endswith(".sh") and not name.startswith("lib-")
                  and os.path.isfile(os.path.join(directory, name)))


def make_shims(scratch):
    shim_directory = os.path.join(scratch, "shims")
    os.makedirs(shim_directory)
    for tool in HEAVY_TOOLS:
        path = os.path.join(shim_directory, tool)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(SHIM)
        os.chmod(path, os.stat(path).st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    return shim_directory


def start_stage(stage_path, root, arguments, shim_directory, trace):
    """起一次阶段：返回 (退出码或 None（超时）, stdout, stderr, 被调了的替身)。"""
    environment = {key: value for key, value in os.environ.items() if key not in CLEARED_ENVIRONMENT}
    environment["PATH"] = shim_directory + os.pathsep + environment.get("PATH", "")
    environment["GATE_STRUCTURE_CHECK_SHIM_TRACE"] = trace
    with open(trace, "w", encoding="utf-8"):
        pass
    try:
        result = subprocess.run(["bash", stage_path, root, *arguments], cwd=root, env=environment,
                                capture_output=True, encoding="utf-8", errors="replace",
                                timeout=LIST_TIMEOUT_SECONDS)
        exit_code, stdout, stderr = result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired as expired:
        exit_code, stdout, stderr = None, str(expired.stdout or ""), str(expired.stderr or "")
    with open(trace, encoding="utf-8") as handle:
        called = sorted({line.strip() for line in handle if line.strip()})
    if BROKEN == "heavy-ignored":
        called = []
    return exit_code, stdout, stderr, called


def tail(text, count=4):
    lines = [line for line in text.splitlines() if line.strip()]
    return " | ".join(lines[-count:]) if lines else "（没有输出）"


def judge_stage(root, name, shim_directory, trace):
    """返回 (问题清单, 格数)；问题清单空就是这一道结构对。格名表缺时返回 (None, 0) 表示「跳过」只在弄坏开关下。"""
    stage_path = os.path.join(root, ".claude", "gate.d", name)
    problems = []
    cells = header_cells(stage_path)
    if not cells:
        if BROKEN == "table-ignored":
            return None, 0
        return [f"{name}：文件头没有格名表（每格一行 # gate-cell: <格名> <判什么的一句>），--list 与 --check 没起它"], 0
    seen = set()
    for number, cell, _title in cells:
        if not CELL_NAME.match(cell):
            problems.append(f"{name}:{number}：格名「{cell}」只许小写字母、数字与连字符")
        if cell in seen:
            problems.append(f"{name}:{number}：格名 {cell} 在格名表里写了两次")
        seen.add(cell)

    exit_code, stdout, stderr, called = start_stage(stage_path, root, ["--list", "--force"], shim_directory, trace)
    if called:
        problems.append(f"{name}：--list 起了重活：{'、'.join(called)}")
    if exit_code is None:
        problems.append(f"{name}：--list 跑了 {LIST_TIMEOUT_SECONDS} 秒还没退出（--list 只该打格名表）")
    elif exit_code != 0:
        problems.append(f"{name}：--list 退 {exit_code}，该退 0；输出末几行：{tail(stdout + stderr)}")
    elif BROKEN != "list-ignored":
        listed = [line.split("\t", 1) for line in stdout.splitlines() if line.strip()]
        listed = [(parts[0], parts[1] if len(parts) > 1 else "（没有制表符）") for parts in listed]
        expected = [(cell, title) for _number, cell, title in cells]
        if listed != expected:
            listed_text = "；".join(f"{cell} {title}" for cell, title in listed) or "（空）"
            expected_text = "；".join(f"{cell} {title}" for cell, title in expected)
            problems.append(f"{name}：--list 打出的格与格名表不一致：--list 是「{listed_text}」，格名表是「{expected_text}」")

    exit_code, stdout, stderr, called = start_stage(stage_path, root, ["--check", UNKNOWN_CELL, "--force"], shim_directory, trace)
    if called:
        problems.append(f"{name}：--check 写错的格名起了重活：{'、'.join(called)}")
    if exit_code is None:
        problems.append(f"{name}：--check {UNKNOWN_CELL} 跑了 {LIST_TIMEOUT_SECONDS} 秒还没退出（格名写错该当场退 2）")
    elif exit_code != 2 and BROKEN != "unknown-ignored":
        problems.append(f"{name}：--check 不存在的格退 {exit_code}，该退 2；输出末几行：{tail(stdout + stderr)}")
    return problems, len(cells)


def run(root):
    names = stage_files(root)
    if not names:
        print(f"  ! {root}/.claude/gate.d/ 下一道阶段都没有，本次无对象可判")
        return 77
    scratch = tempfile.mkdtemp(prefix="gate-structure-check-")
    try:
        shim_directory = make_shims(scratch)
        trace = os.path.join(scratch, "shim-trace")
        wrong_stages, problems, cell_count, judged = [], [], 0, 0
        for name in names:
            stage_problems, cells = judge_stage(root, name, shim_directory, trace)
            if stage_problems is None:
                continue
            judged += 1
            cell_count += cells
            if stage_problems:
                wrong_stages.append(name)
                problems += stage_problems
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    if problems:
        for problem in problems:
            print(f"     {problem}")  # gate-lint:detail
        print(f"  ✗ {len(wrong_stages)}/{judged} 道阶段的结构不对：{'、'.join(wrong_stages)}")
        print("  → 怎么办：改用 .claude/gate.d/lib/stage-cells.sh（写法在它的文件头，样板是 doc-decisions.sh）："
              "文件头逐格写 # gate-cell: <格名> <判什么>，每格一个函数 stage_cell 登记；--list 之前不做任何重活。"
              "规则：.claude/rules/verification.md「门禁的结构」。")
        return 1
    print(f"  ✓ 门禁阶段的结构都对：查了 {judged} 道阶段、共 {cell_count} 格"
          f"（格名表在、--list 与表一致且没起重活、--check 写错格名退 2）")
    return 0


SELFTEST_GOOD_STAGE = """#!/usr/bin/env bash
# gate-cell: alpha 判甲
# gate-cell: beta 判乙
set -uo pipefail
{prelude}
source "$(dirname "${{BASH_SOURCE[0]}}")/lib/stage-cells.sh"
cell_alpha() {{ echo "  ✓ 甲 1 项"; }}
cell_beta() {{ echo "  ✓ 乙 1 项"; }}
stage_cell alpha cell_alpha "判甲" "改甲"
stage_cell beta cell_beta "{beta_title}" "改乙"
stage_cells_parse "$@"; set -- ${{STAGE_CELLS_REST[@]+"${{STAGE_CELLS_REST[@]}}"}}
ROOT="${{1:-.}}"; cd "$ROOT" || exit 2
stage_cells_run "$ROOT"
"""
SELFTEST_LENIENT_STAGE = """#!/usr/bin/env bash
# gate-cell: alpha 判甲
case " $* " in *" --list "*) printf 'alpha\\t判甲\\n'; exit 0 ;; esac
echo "  ✓ 什么参数都照跑，1 格"
exit 0
"""


def write_stage(directory, name, text):
    with open(os.path.join(directory, name), "w", encoding="utf-8") as handle:
        handle.write(text)


def selftest():
    library = os.path.join(REPOSITORY_ROOT, ".claude", "gate.d", "lib", "stage-cells.sh")
    work = tempfile.mkdtemp(prefix="gate-structure-check-selftest-")
    wrong = []
    checked = 0
    try:
        good_root, bad_root, empty_root = (os.path.join(work, part) for part in ("good", "bad", "empty"))
        for root in (good_root, bad_root, empty_root):
            os.makedirs(os.path.join(root, ".claude", "gate.d", "lib"))
        for root in (good_root, bad_root):
            shutil.copy(library, os.path.join(root, ".claude", "gate.d", "lib", "stage-cells.sh"))
        good = SELFTEST_GOOD_STAGE.format(prelude="", beta_title="判乙")
        write_stage(os.path.join(good_root, ".claude", "gate.d"), "10-good.sh", good)
        bad_directory = os.path.join(bad_root, ".claude", "gate.d")
        write_stage(bad_directory, "10-good.sh", good)
        write_stage(bad_directory, "11-no-table.sh", good.replace("# gate-cell: alpha 判甲\n# gate-cell: beta 判乙\n", ""))
        write_stage(bad_directory, "12-names.sh", good.replace("# gate-cell: beta 判乙", "# gate-cell: gamma 判乙"))
        write_stage(bad_directory, "13-title.sh", SELFTEST_GOOD_STAGE.format(prelude="", beta_title="判乙（改过）"))
        write_stage(bad_directory, "14-heavy.sh", SELFTEST_GOOD_STAGE.format(prelude="cargo --version >/dev/null 2>&1", beta_title="判乙"))
        write_stage(bad_directory, "15-lenient.sh", SELFTEST_LENIENT_STAGE)

        cases = [
            ("bad", bad_root, 1, [
                "11-no-table.sh：文件头没有格名表",
                "12-names.sh：--list 打出的格与格名表不一致：--list 是「alpha 判甲；beta 判乙」，格名表是「alpha 判甲；gamma 判乙」",
                "13-title.sh：--list 打出的格与格名表不一致：--list 是「alpha 判甲；beta 判乙（改过）」",
                "14-heavy.sh：--list 起了重活：cargo",
                "14-heavy.sh：--check 写错的格名起了重活：cargo",
                "15-lenient.sh：--check 不存在的格退 0，该退 2",
                "✗ 5/6 道阶段的结构不对：11-no-table.sh、12-names.sh、13-title.sh、14-heavy.sh、15-lenient.sh",
            ]),
            ("good", good_root, 0, ["✓ 门禁阶段的结构都对：查了 1 道阶段、共 2 格"]),
            ("empty", empty_root, 77, ["一道阶段都没有，本次无对象可判"]),
        ]
        environment = {key: value for key, value in os.environ.items() if key not in CLEARED_ENVIRONMENT}
        for label, root, want_exit, wants in cases:
            checked += 1
            result = subprocess.run([sys.executable, os.path.realpath(__file__), root], env=environment,
                                    capture_output=True, encoding="utf-8", errors="replace")
            output = result.stdout + result.stderr
            if result.returncode != want_exit:
                wrong.append(f"{label}：期望退出 {want_exit}，实测 {result.returncode}；输出末几行：{tail(output, 8)}")
            for want in wants:
                if want not in output:
                    wrong.append(f"{label}：输出里找不到「{want}」")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    if wrong:
        for entry in wrong:
            print(f"     {entry}")  # gate-lint:detail
        print(f"  ✗ gate-structure-check 自证 {checked} 份仓里有判错的（上面逐条列着）")
        print("  → 怎么办：按判错那一条找 judge_stage 里对应的一样修；弄坏开关打开时判红是对的。")
        return 1
    print(f"  ✓ gate-structure-check 自证 {checked} 份仓都判对（六道合成阶段里五道各犯一样、一道对，另有只放对的一道与空目录）")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    root = REPOSITORY_ROOT
    for argument in arguments:
        if argument.startswith("-"):
            print(f"  ✗ 认不出的参数 {argument}")
            print("  → 怎么办：只认 --selftest 与一个仓根：gate-structure-check.py [仓根]")
            return 2
        root = os.path.abspath(argument)
    return run(root)


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
