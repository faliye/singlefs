#!/usr/bin/env python3
# admission: always 判的是此刻门禁目录里每道阶段的文件头与它们对 --list、--check 的反应，阶段随时在改
# run-condition: command bash
"""门禁阶段的结构（.claude/rules/verification.md「门禁的结构」）：每道阶段都能全跑，也能逐格点名跑。

对 <仓根>/.claude/gate.d/ 顶层每个 *.sh（lib-*.sh 是共用库，不判）逐道判四样：
  table    文件头（#! 之后、第一行代码之前的注释块）有格名表：每格一行 `# gate-cell: <格名> <判什么的一句>`，格名不重复
  preflight  source 了 scripts/preflight.sh 的阶段，preflight 那一行（preflight "${BASH_SOURCE[0]}" "$@"; set -- …）的下一行是守卫
           `if ! declare -F preflight >/dev/null; then`，到它的 fi 为止有一行打 ✗、一行打 →、一行 exit 2（PREFLIGHT_GUARD_OPENING）。
           为什么：门禁开的是 set -uo pipefail、没有 -e，副本里缺 .claude/scripts/preflight.sh 时 source 只报一行，preflight 成了找不到的命令，
           紧跟的 set -- 把参数清空，--check <格> 与 --list 变成全跑（只拷了门禁表的副本里，stage-owners 格起的 --list 就这样一层套一层起 code-tooling.sh，限时一到只杀外层，孙进程成了孤儿）。
           守卫不能写在 source 那一行上或 preflight 那一行之前：上游 preflight-lint.py 只许 preflight 之前是 source、set、单行赋值这几种行。
  list     `bash <阶段> <仓根> --list --force` 退 0，stdout 逐行「格名<制表符>判什么」，与格名表逐格相同、次序相同；
           跑的时候没起重活：PATH 最前面放一排替身（cargo、rustc、rustup、qemu-system-x86_64、qemu-system-aarch64、qemu-img、herd7、fio），
           哪个替身被调了就判红；退出之后它起的进程一个不剩（下面「起法」）
  unknown  `bash <阶段> <仓根> --check gate-structure-check-no-such-cell --force` 退 2，同样不许起重活、不许留下进程
  --force 是交给阶段的 preflight 的：只问结构，不因为「输入没变」被拒。每次起阶段限时 LIST_TIMEOUT_SECONDS 秒。
  文件头没有格名表的阶段不起它（还没改用 lib/stage-cells.sh 的阶段不认 --list，起它就是全跑）。
起法：经 scripts/gate/lib_process_group.py 的 run_in_own_session，每次起阶段放进它自己的会话与进程组；超时、异常、收到 TERM / INT / HUP
  与正常退出之后都对整组先 TERM、过宽限 KILL，再在 /proc 里核会话里没有留下进程（.claude/singlefs-ai-sop/rules/command-safety.md
  「装置起的样本进程，放进它自己的进程组」）。退出之后还留着的、收完还停不掉的，写进那一道的失败信息。
  code-tooling.sh 的 stage-owners 格导入 start_stage（四样返回值不变；留下的进程照样收，那一句接在 stderr 末尾）。
判不了的：按绝对路径起的重活（target/ 下的装置二进制、写死路径的 cargo）替身拦不到；格的函数判得对不对；
  格名表里的「判什么」写得准不准；守卫的出路写得对不对（只认形状：打 ✗、打 →、exit 2）；自己 setsid 出去、离开了会话的进程。

用法：
    gate-structure-check.py [仓根]      不给就取这份脚本往上两级
    gate-structure-check.py --selftest  GATE_STRUCTURE_CHECK_BREAK=<项> 时必须判红（下面那张表）
    gate-structure-check.py --selftest-case <例名> <样本根>   只给 --selftest 用：在新会话里判一道样本阶段（例名见 SELFTEST_GROUP_CASES），
                                                              自证等它退出、数完它的样本记下的进程，再对它那一组 KILL 收尾
自证（selftest()）：三份仓整跑这一份（bad 七道阶段里六道各犯一样、good 一道两格都对、empty 空目录），另有四例在新会话里判一道阶段（SELFTEST_GROUP_CASES）：
  guard-without-preflight  带守卫的阶段、树里没有 .claude/scripts/preflight.sh：--check alpha 退 2、打出守卫的 ✗、一格都没跑
  timeout-group            阶段先放两个后台子进程（一个再起孙进程）、再前台睡过限时（自证把限时缩成 SELFTEST_LIST_TIMEOUT_SECONDS 秒）：
                           报「--list 跑了 2 秒还没退出」，start_stage 返回之后样本记下的进程一个不剩
  left-behind              阶段 --list 照常退 0，却丢下一个后台进程：报「--list 退出之后它起的进程还留着」，返回之后那个进程不在了
  guarded-with-preflight   带守卫的阶段、树里有替身 preflight.sh：结构都对，不报守卫
弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里 cases 的标签，括号里是那几条判错行的开头；
动到 bad 那份仓的几项红时都另有一条「bad：输出里找不到「✗ 6/7 道阶段的结构不对：…」」，每一项都另有汇总行「✗ gate-structure-check 自证 3 份仓、4 例里有判错的」）：
  GATE_STRUCTURE_CHECK_BREAK=table-ignored  文件头没有格名表的阶段悄悄跳过
      bad（「bad：输出里找不到「11-no-table.sh：文件头没有格名表」」）
  GATE_STRUCTURE_CHECK_BREAK=list-ignored  不比 --list 与格名表
      bad（「bad：输出里找不到「12-names.sh：--list 打出的格与格名表不一致：…」」、「bad：输出里找不到「13-title.sh：--list 打出的格与格名表不一致：…」」）
  GATE_STRUCTURE_CHECK_BREAK=heavy-ignored  不看替身有没有被调
      bad（「bad：输出里找不到「14-heavy.sh：--list 起了重活：cargo」」、「bad：输出里找不到「14-heavy.sh：--check 写错的格名起了重活：cargo」」）
  GATE_STRUCTURE_CHECK_BREAK=unknown-ignored  不判 --check 写错格名的退出码
      bad（「bad：输出里找不到「15-lenient.sh：--check 不存在的格退 0，该退 2」」）
  GATE_STRUCTURE_CHECK_BREAK=preflight-guard-ignored  不看 preflight 那一行之后有没有守卫
      bad（「bad：输出里找不到「16-unguarded.sh:6：source preflight 之后没有守卫」」）
  GATE_STRUCTURE_CHECK_BREAK=timeout-kills-leader-only  不开新会话，超时只 SIGKILL 直接起的 bash，不收组、不核会话
      timeout-group（「timeout-group：start_stage 返回之后样本记下的进程还剩」）
      left-behind（「left-behind：判出的问题里找不到「19-left-behind.sh：--list 退出之后它起的进程还留着」」、「left-behind：start_stage 返回之后样本记下的进程还剩」）
退出码：0 每道都对；1 有不对的（逐道列出）；2 用法错；77 门禁目录里一道阶段都没有。
"""
import json
import os
import re
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
sys.path.insert(0, SCRIPT_DIRECTORY)
from preflight import preflight  # noqa: E402
import lib_process_group as process_group  # noqa: E402

BROKEN = os.environ.get("GATE_STRUCTURE_CHECK_BREAK", "")
LIST_TIMEOUT_SECONDS = 60
SELFTEST_LIST_TIMEOUT_SECONDS = 2
# preflight 那一行之后的守卫（文件头 preflight 一样）：source 的是 …/scripts/preflight.sh（项目的 shim 或规范副本）的阶段才判
PREFLIGHT_SOURCE_LINE = re.compile(r'^\s*(?:source|\.)\s+.*/scripts/preflight\.sh"?\s*(?:#.*)?$')
PREFLIGHT_ENTRY_LINE = re.compile(r'^\s*preflight\s+"\$\{BASH_SOURCE\[0\]\}"\s+"\$@"\s*;\s*set\s+--\s')
PREFLIGHT_GUARD_OPENING = "if ! declare -F preflight >/dev/null; then"
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


def start_stage_reporting(root, name, arguments, shim_directory, trace):
    """起一次 <root>/.claude/gate.d/<name> 那道阶段：返回 (退出码或 None（超时）, stdout, stderr, 被调了的替身, 进程收尾的问题或 None)。
    经 lib_process_group.run_in_own_session 起（文件头「起法」）：放进它自己的会话，超时与退出之后收整组、核会话里没有留下进程。
    路径照「阶段目录 + 文件名」拼：函数单元库把这一层的阶段都算进闭包（判的就是它们）。"""
    stage_path = os.path.join(root, ".claude", "gate.d", name)
    environment = {key: value for key, value in os.environ.items() if key not in CLEARED_ENVIRONMENT}
    environment["PATH"] = shim_directory + os.pathsep + environment.get("PATH", "")
    environment["GATE_STRUCTURE_CHECK_SHIM_TRACE"] = trace
    with open(trace, "w", encoding="utf-8"):
        pass
    run = process_group.run_in_own_session(["bash", stage_path, root, *arguments], cwd=root, env=environment,
                                           timeout=LIST_TIMEOUT_SECONDS, leader_only=BROKEN == "timeout-kills-leader-only")
    with open(trace, encoding="utf-8") as handle:
        called = sorted({line.strip() for line in handle if line.strip()})
    if BROKEN == "heavy-ignored":
        called = []
    return (None if run.timed_out else run.returncode), run.stdout, run.stderr, called, run.problem()


def start_stage(root, name, arguments, shim_directory, trace):
    """同 start_stage_reporting，只返回前四样（code-tooling.sh 的 stage-owners 格照这四样用）；进程收尾的问题接在 stderr 末尾。"""
    exit_code, stdout, stderr, called, leftover_problem = start_stage_reporting(root, name, arguments, shim_directory, trace)
    if leftover_problem:
        stderr += f"\ngate-structure-check：{name} {' '.join(arguments)} {leftover_problem}\n"
    return exit_code, stdout, stderr, called


def preflight_guard_problem(name, lines):
    """source 了 …/scripts/preflight.sh 的阶段，preflight 那一行的下一行要是守卫（文件头 preflight 一样）；没 source 的、守卫齐的交 None。"""
    source_index = next((index for index, line in enumerate(lines) if PREFLIGHT_SOURCE_LINE.match(line)), None)
    if source_index is None:
        return None
    entry_index = next((index for index in range(source_index + 1, len(lines)) if PREFLIGHT_ENTRY_LINE.match(lines[index])), None)
    if entry_index is None:
        return (f"{name}:{source_index + 1}：source 了 preflight.sh，后面却没有 preflight \"${{BASH_SOURCE[0]}}\" \"$@\"; set -- … 那一行，"
                "守卫无处可挂（参数也没交给 preflight）")
    following = lines[entry_index + 1].strip() if entry_index + 1 < len(lines) else ""
    if following != PREFLIGHT_GUARD_OPENING:
        return (f"{name}:{entry_index + 1}：source preflight 之后没有守卫（preflight 那一行的下一行不是「{PREFLIGHT_GUARD_OPENING}」）："
                "缺了 .claude/scripts/preflight.sh 时 preflight 成了找不到的命令，紧跟的 set -- 把参数清空，--check 与 --list 变成全跑")
    body = []
    for line in lines[entry_index + 2:]:
        if line.strip() == "fi":
            break
        body.append(line)
    else:
        return f"{name}:{entry_index + 2}：preflight 之后的守卫没有收尾的 fi"
    missing = [what for what, present in (("打 ✗ 的一行", any("✗" in line for line in body)),
                                          ("打 → 出路的一行", any("→" in line for line in body)),
                                          ("exit 2", any(re.match(r"^\s*exit\s+2\s*(?:#.*)?$", line) for line in body))) if not present]
    if missing:
        return f"{name}:{entry_index + 2}：preflight 之后的守卫缺{'、'.join(missing)}（找不到 preflight 时要当场打 ✗ 与出路、退 2）"
    return None


def tail(text, count=4):
    lines = [line for line in text.splitlines() if line.strip()]
    return " | ".join(lines[-count:]) if lines else "（没有输出）"


def judge_stage(root, name, shim_directory, trace):
    """返回 (问题清单, 格数)；问题清单空就是这一道结构对。格名表缺时返回 (None, 0) 表示「跳过」只在弄坏开关下。"""
    stage_path = os.path.join(root, ".claude", "gate.d", name)
    problems = []
    with open(stage_path, encoding="utf-8", errors="replace") as handle:
        guard_problem = preflight_guard_problem(name, handle.read().split("\n"))
    if guard_problem and BROKEN != "preflight-guard-ignored":
        problems.append(guard_problem)
    cells = header_cells(stage_path)
    if not cells:
        if BROKEN == "table-ignored":
            return None, 0
        return problems + [f"{name}：文件头没有格名表（每格一行 # gate-cell: <格名> <判什么的一句>），--list 与 --check 没起它"], 0
    seen = set()
    for number, cell, _title in cells:
        if not CELL_NAME.match(cell):
            problems.append(f"{name}:{number}：格名「{cell}」只许小写字母、数字与连字符")
        if cell in seen:
            problems.append(f"{name}:{number}：格名 {cell} 在格名表里写了两次")
        seen.add(cell)

    exit_code, stdout, stderr, called, leftover_problem = start_stage_reporting(root, name, ["--list", "--force"], shim_directory, trace)
    if called:
        problems.append(f"{name}：--list 起了重活：{'、'.join(called)}")
    if leftover_problem:
        problems.append(f"{name}：--list {leftover_problem}")
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

    exit_code, stdout, stderr, called, leftover_problem = start_stage_reporting(root, name, ["--check", UNKNOWN_CELL, "--force"], shim_directory, trace)
    if called:
        problems.append(f"{name}：--check 写错的格名起了重活：{'、'.join(called)}")
    if leftover_problem:
        problems.append(f"{name}：--check {UNKNOWN_CELL} {leftover_problem}")
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


SELFTEST_PREFLIGHT_PRELUDE = '''source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}'''
SELFTEST_PREFLIGHT_GUARD = PREFLIGHT_GUARD_OPENING + '''
  echo "  ✗ 样本阶段：source preflight.sh 没成，preflight 没有定义，参数已被清空，拒绝往下跑"
  echo "     → 怎么办：样本树里补上 .claude/scripts/preflight.sh"
  exit 2
fi'''
SELFTEST_PREFLIGHT_SHIM = """# 自证用的替身 preflight：摘掉 --force，其余参数原样放回
preflight() {
  shift
  PREFLIGHT_ARGUMENTS=()
  local argument
  for argument in "$@"; do
    [[ "$argument" == --force ]] || PREFLIGHT_ARGUMENTS+=("$argument")
  done
}
"""
SELFTEST_TIMEOUT_STAGE = """#!/usr/bin/env bash
# gate-cell: alpha 判甲
# 自证样本：带什么参数都一样，先放两个后台子进程（一个再起孙进程），再睡过限时；起的进程号逐个记进 selftest-pids
sleep 297 >/dev/null 2>&1 &
printf '%s\\n' "$!" >> selftest-pids
( sleep 298 & printf '%s\\n' "$!" >> selftest-pids; wait "$!" ) &
printf '%s\\n' "$!" >> selftest-pids
sleep 299 &
printf '%s\\n' "$!" >> selftest-pids
wait "$!"
"""
SELFTEST_LEFT_BEHIND_STAGE = """#!/usr/bin/env bash
# gate-cell: alpha 判甲
# 自证样本：--list 照常打格名表、退 0，--check 写错格名照常退 2，却每次都丢下一个后台进程（进程号记进 selftest-pids）
sleep 296 >/dev/null 2>&1 &
printf '%s\\n' "$!" >> selftest-pids
case " $* " in
  *" --list "*) printf 'alpha\\t判甲\\n'; exit 0 ;;
  *" --check alpha "*) exit 0 ;;
  *" --check "*) echo "  ✗ 样本阶段只认 alpha 一个格名"; echo "     → 怎么办：照 --list 写格名"; exit 2 ;;
esac
exit 0
"""
# 在新会话里判一道阶段的四例（文件头「自证」）：例名 →（阶段文件名，样本树里放不放替身 preflight.sh）
SELFTEST_GROUP_CASES = {
    "guard-without-preflight": ("17-guarded.sh", False),
    "timeout-group": ("18-timeout.sh", False),
    "left-behind": ("19-left-behind.sh", False),
    "guarded-with-preflight": ("17-guarded.sh", True),
}
SELFTEST_CASE_TIMEOUT_SECONDS = 60


def write_stage(directory, name, text):
    with open(os.path.join(directory, name), "w", encoding="utf-8") as handle:
        handle.write(text)


def write_preflight_shim(root):
    os.makedirs(os.path.join(root, ".claude", "scripts"), exist_ok=True)
    with open(os.path.join(root, ".claude", "scripts", "preflight.sh"), "w", encoding="utf-8") as handle:
        handle.write(SELFTEST_PREFLIGHT_SHIM)


def guarded_stage_text():
    return SELFTEST_GOOD_STAGE.format(prelude=SELFTEST_PREFLIGHT_PRELUDE + "\n" + SELFTEST_PREFLIGHT_GUARD, beta_title="判乙")


def selftest_case(case, root):
    """--selftest-case：在样本根里判一道阶段，结果按 JSON 打在 stdout 最后一行。timeout-group 把限时缩成 SELFTEST_LIST_TIMEOUT_SECONDS 秒。"""
    global LIST_TIMEOUT_SECONDS
    stage, _with_shim = SELFTEST_GROUP_CASES[case]
    scratch = tempfile.mkdtemp(prefix="gate-structure-check-case-")
    try:
        shim_directory = make_shims(scratch)
        trace = os.path.join(scratch, "shim-trace")
        if case == "guard-without-preflight":
            exit_code, stdout, stderr, _called, leftover_problem = start_stage_reporting(root, stage, ["--check", "alpha", "--force"], shim_directory, trace)
            result = {"exit": exit_code, "output": stdout + stderr, "leftover": leftover_problem}
        else:
            if case == "timeout-group":
                LIST_TIMEOUT_SECONDS = SELFTEST_LIST_TIMEOUT_SECONDS
            problems, _cells = judge_stage(root, stage, shim_directory, trace)
            result = {"problems": problems or []}
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    print(json.dumps(result, ensure_ascii=False))
    return 0


def recorded_processes_alive(root, stage):
    """样本阶段记进 selftest-pids 的进程里此刻还在的（僵尸不算；命令行是 sleep 或那道样本阶段的才算，进程号被别人复用的不算）。只读，不发信号。"""
    path = os.path.join(root, "selftest-pids")
    if not os.path.isfile(path):
        return []
    stage_path = os.path.join(root, ".claude", "gate.d", stage).encode()
    alive = []
    with open(path, encoding="utf-8") as handle:
        recorded = sorted({int(line) for line in handle if line.strip().isdigit()})
    for pid in recorded:
        try:
            with open(f"/proc/{pid}/stat", "rb") as handle:
                state = handle.read().rsplit(b")", 1)[1].split()[0]
            with open(f"/proc/{pid}/cmdline", "rb") as handle:
                command_line = handle.read()
        except (OSError, IndexError):
            continue
        if state not in (b"Z", b"X") and (command_line.startswith(b"sleep\0") or stage_path in command_line):
            shown = command_line.replace(b"\0", b" ").decode("utf-8", errors="replace").strip()
            alive.append(f"{pid} {shown[:80]}")
    return alive


def run_group_case(case, root, environment):
    """在新会话里起 --selftest-case，等它退出、数完样本记下的进程还活着的，再对它那一组 SIGKILL 收尾
    （弄坏开关 timeout-kills-leader-only 下阶段与它丢下的进程都还在这一组里）。返回 (结果或 None, 还活着的, 收尾之后仍在的, 它的输出末几行)。"""
    helper = subprocess.Popen([sys.executable, os.path.realpath(__file__), "--selftest-case", case, root], env=environment,
                              stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                              encoding="utf-8", errors="replace", start_new_session=True)
    try:
        stdout, stderr = helper.communicate(timeout=SELFTEST_CASE_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        os.killpg(helper.pid, signal.SIGKILL)
        stdout, stderr = helper.communicate()
    stage = SELFTEST_GROUP_CASES[case][0]
    alive = recorded_processes_alive(root, stage)
    try:
        os.killpg(helper.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    deadline = time.monotonic() + process_group.SETTLE_SECONDS
    remaining = recorded_processes_alive(root, stage)
    while remaining and time.monotonic() < deadline:
        time.sleep(0.1)
        remaining = recorded_processes_alive(root, stage)
    lines = [line for line in stdout.splitlines() if line.strip()]
    try:
        result = json.loads(lines[-1]) if lines else None
    except ValueError:
        result = None
    return result, alive, remaining, tail(stdout + stderr, 6)


def judge_group_case(case, root, environment):
    """一例的判错清单（空就是判对）。"""
    result, alive, remaining, shown = run_group_case(case, root, environment)
    if result is None:
        return [f"{case}：--selftest-case 没交回结果；输出末几行：{shown}"]
    wrong = []
    stage = SELFTEST_GROUP_CASES[case][0]
    problems = result.get("problems", [])
    if case == "guard-without-preflight":
        if result["exit"] != 2:
            wrong.append(f"{case}：树里没有 preflight.sh 时 --check alpha 退 {result['exit']}，该退 2（守卫没拦住）；输出末几行：{tail(result['output'], 6)}")
        if "✗ 样本阶段：source preflight.sh 没成" not in result["output"]:
            wrong.append(f"{case}：输出里找不到守卫打的「✗ 样本阶段：source preflight.sh 没成」")
        if "── alpha" in result["output"]:
            wrong.append(f"{case}：守卫之后还跑了格（输出里有「── alpha」）")
    elif case == "timeout-group":
        want = f"{stage}：--list 跑了 {SELFTEST_LIST_TIMEOUT_SECONDS} 秒还没退出"
        if not any(want in problem for problem in problems):
            wrong.append(f"{case}：判出的问题里找不到「{want}」；判出的是：{'；'.join(problems) or '（无）'}")
    elif case == "left-behind":
        want = f"{stage}：--list 退出之后它起的进程还留着"
        if not any(want in problem for problem in problems):
            wrong.append(f"{case}：判出的问题里找不到「{want}」；判出的是：{'；'.join(problems) or '（无）'}")
    elif problems:
        wrong.append(f"{case}：带守卫、有替身 preflight.sh 的阶段被判出问题：{'；'.join(problems)}")
    if alive:
        wrong.append(f"{case}：start_stage 返回之后样本记下的进程还剩 {len(alive)} 个（应为 0）：{'；'.join(alive)}")
    if remaining:
        wrong.append(f"{case}：自证对 --selftest-case 那一组 SIGKILL 之后样本记下的进程仍在 {len(remaining)} 个：{'；'.join(remaining)}")
    return wrong


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
        # 16-unguarded.sh：source 了 preflight.sh、preflight 那一行（第 6 行）之后没有守卫；树里有替身 preflight.sh，跑起来照常，只有守卫那一样不对
        write_stage(bad_directory, "16-unguarded.sh", SELFTEST_GOOD_STAGE.format(prelude=SELFTEST_PREFLIGHT_PRELUDE, beta_title="判乙"))
        write_preflight_shim(bad_root)

        cases = [
            ("bad", bad_root, 1, [
                "11-no-table.sh：文件头没有格名表",
                "12-names.sh：--list 打出的格与格名表不一致：--list 是「alpha 判甲；beta 判乙」，格名表是「alpha 判甲；gamma 判乙」",
                "13-title.sh：--list 打出的格与格名表不一致：--list 是「alpha 判甲；beta 判乙（改过）」",
                "14-heavy.sh：--list 起了重活：cargo",
                "14-heavy.sh：--check 写错的格名起了重活：cargo",
                "15-lenient.sh：--check 不存在的格退 0，该退 2",
                "16-unguarded.sh:6：source preflight 之后没有守卫",
                "✗ 6/7 道阶段的结构不对：11-no-table.sh、12-names.sh、13-title.sh、14-heavy.sh、15-lenient.sh、16-unguarded.sh",
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

        stage_texts = {"17-guarded.sh": guarded_stage_text(), "18-timeout.sh": SELFTEST_TIMEOUT_STAGE, "19-left-behind.sh": SELFTEST_LEFT_BEHIND_STAGE}
        for case, (stage, with_shim) in SELFTEST_GROUP_CASES.items():
            root = os.path.join(work, case)
            os.makedirs(os.path.join(root, ".claude", "gate.d", "lib"))
            shutil.copy(library, os.path.join(root, ".claude", "gate.d", "lib", "stage-cells.sh"))
            write_stage(os.path.join(root, ".claude", "gate.d"), stage, stage_texts[stage])
            if with_shim:
                write_preflight_shim(root)
            wrong += judge_group_case(case, root, environment)
    finally:
        shutil.rmtree(work, ignore_errors=True)
    if wrong:
        for entry in wrong:
            print(f"  ✗ {entry}")  # gate-lint:detail
        print(f"  ✗ gate-structure-check 自证 {checked} 份仓、{len(SELFTEST_GROUP_CASES)} 例里有判错的（上面逐条列着）")
        print("  → 怎么办：按判错那一条找 judge_stage、preflight_guard_problem 或 lib_process_group.run_in_own_session 里对应的一样修；弄坏开关打开时判红是对的。")
        return 1
    print(f"  ✓ gate-structure-check 自证 {checked} 份仓都判对（七道合成阶段里六道各犯一样、一道对，另有只放对的一道与空目录），"
          f"{len(SELFTEST_GROUP_CASES)} 例在新会话里判一道阶段都对（缺 preflight.sh 时守卫退 2、超时收整组、退出之后丢下的进程报出并收掉、有守卫有 preflight 照绿）")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    if arguments[:1] == ["--selftest-case"]:
        if len(arguments) != 3 or arguments[1] not in SELFTEST_GROUP_CASES:
            print(f"  ✗ --selftest-case 要两个参数：例名（{'、'.join(SELFTEST_GROUP_CASES)} 之一）与样本根")
            print("  → 怎么办：这一种起法只给 --selftest 用；要判门禁结构写 gate-structure-check.py [仓根]")
            return 2
        return selftest_case(arguments[1], os.path.abspath(arguments[2]))
    root = REPOSITORY_ROOT
    for argument in arguments:
        if argument.startswith("-"):
            print(f"  ✗ 认不出的参数 {argument}")
            print("  → 怎么办：只认 --selftest 与一个仓根：gate-structure-check.py [仓根]")
            return 2
        root = os.path.abspath(argument)
    return run(root)


BREAK_VARIABLES = ("GATE_STRUCTURE_CHECK_BREAK",)


def inside_selftest():
    """往上数这个进程的祖先，有一个的命令行里带 --selftest（自证起的子进程，含别的脚本的自证起它的）就是 True。"""
    pid = os.getppid()
    while pid > 1:
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as handle:
                if b"--selftest" in handle.read().split(b"\0"):
                    return True
            with open(f"/proc/{pid}/stat", "rb") as handle:
                pid = int(handle.read().rsplit(b")", 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            return False
    return False


def settle_break_switches(arguments):
    """弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道。
    放行：命令行以 --selftest 起头；祖先里有自证（inside_selftest）。
    都不是而 BREAK_VARIABLES 里有设了的：点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    if not setting or arguments[:1] == ["--selftest"] or inside_selftest():
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 {setting[0]}=<项> python3 scripts/gate/gate-structure-check.py --selftest", file=sys.stderr)
    return 2


if __name__ == "__main__":
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
