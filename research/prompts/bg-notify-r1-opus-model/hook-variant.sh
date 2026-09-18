#!/usr/bin/env bash
# PreToolUse hook（Bash）：检出可能出问题的命令，记下来交给主 agent 判断；一律放行，不拦、不停任何命令与脚本。
#
# 为什么：2026-09-17 一个实验执行员用 `pgrep -f` 找复跑进程，接着在一个等不到的 `until` 循环里空转，
# 主 agent 查进度才发现（records/2026-09-17-已分配口径三方与两个实验.md 第六节）。用户定：hook 不能终止任务和脚本，
# 只负责检出问题，交给主 agent 去判断；子 agent 与脚本结不结束由主 agent 定。
#
# 检出三种，写进检出记录（默认 /tmp/claude-1000/agent-hook-detections.jsonl，环境变量 AGENT_HOOK_DETECTIONS 可改），每条一行 JSON：
#   ① 按模式匹配的进程命令（`pgrep -f`、`pkill -f`、`killall`）：模式串会命中自己所在的 shell（command-safety.md）；
#   ② 没有 `timeout` 的 `until` / `while` 等待循环（里面有 `sleep`）：等的条件不成立时会一直等；
#   ③ `run_in_background` 起的命令里又自己放后台（`nohup`、`setsid`、`disown`、后面没有 `wait` 的单独 `&`）：外层 shell 当场退出，
#      harness 的完成通知当场发出，真正在跑的东西跑完不再叫醒谁（2026-09-18 一个门禁分诊员这样起门禁与三个缓存计时器，
#      records/2026-09-16-subagent拆分提案.md 第三十三节）。
# 主 agent 的看门狗（research/scripts/agent-watch.py watch）每 15 秒读一次检出记录，读到本会话的检出就退出、叫醒主 agent。
# 只看顶层命令文本；命令里只是带着这些字（heredoc 里的测试数据）也会被记一条，主 agent 看了判断即可。
#
#   bash-command-detector.sh             # 从 stdin 读 hook 的 JSON，永远退出 0
#   bash-command-detector.sh --selftest  # 走一遍检出与不检出；BASH_COMMAND_DETECTOR_DISABLE_CHECK=1 或
#                                        # BASH_COMMAND_DETECTOR_DISABLE_SELF_BACKGROUND=1（只关第三种）时自检必须判红
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（与 agent-write-scope.sh 同一个坑）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import json, os, re, subprocess, sys, tempfile
from datetime import datetime, timezone

PATTERN_PROCESS_COMMAND = re.compile(r"\bpgrep\s+(?:-\w+\s+)*-\w*f|\bpkill\s+(?:-\w+\s+)*-\w*f|\bkillall\b")
WAIT_LOOP = re.compile(r"\b(until|while)\b[^\n]*?;\s*do\b[\s\S]*?\bsleep\b")
SELF_DETACH = re.compile(
    r"\bsetsid\b(?![^\n]*\s(?:-w|--wait)\b)"
    r"|\bdisown\b|\bcoproc\b"
    r"|\bnohup\b[^\n]*(?<![&>|<])&(?![&>])"
    r"|\btmux\b[^\n]*\bnew(?:-session)?\b[^\n]*\s-d\b"
    r"|\bscreen\b[^\n]*\s-(?:[dD]m\w*|d\s+-m)"
    r"|\bsystemd-run\b(?![^\n]*--(?:wait|pipe|pty|scope)\b)"
    r"|\bstart-stop-daemon\b[^\n]*\s(?:-b|--background)\b")
LONE_AMPERSAND = re.compile(r"(?<![&>|<])&(?![&>])")
WAIT_COMMAND = re.compile(r"(?:^|[;&|\n({]|\b(?:then|do|else)\b)\s*wait\b(?![\w.(-])([^;&|\n)}]*)")
SUBSHELL_AMPERSAND = re.compile(r"(?<![&>|<])&\s*\)|\$\([^()]*(?<![&>|<])&(?![&>])[^()]*\)")
POLL_AFTER = re.compile(r"\btail\b[^\n]*--pid\b|\b(?:until|while)\b[^\n]*\bkill\s+-0\b")
SHELL_HEAD = re.compile(r"\b(?:bash|sh|dash|zsh|ksh)\b")
HEREDOC = re.compile(r"<<-?\s*(['\"]?)(\w+)\1([^\n]*)\n(.*?)\n\s*\2[ \t]*(?=\n|$)", re.S)

def strip_data_heredocs(command):
    def replace(match):
        head = command[command.rfind("\n", 0, match.start()) + 1:match.start()]
        if SHELL_HEAD.search(head):
            return match.group(0)
        return f"<<{match.group(1)}{match.group(2)}{match.group(1)}{match.group(3)}\n{match.group(2)}"
    return HEREDOC.sub(replace, command)

def puts_itself_in_background(command):
    command = strip_data_heredocs(command)
    if SELF_DETACH.search(command) or SUBSHELL_AMPERSAND.search(command):
        return True
    ampersands = list(LONE_AMPERSAND.finditer(command))
    for match in ampersands:
        rest = command[match.end():]
        if POLL_AFTER.search(rest):
            continue
        waits = [found.group(1).strip() for found in WAIT_COMMAND.finditer(rest)]
        if any(arguments == "" for arguments in waits):
            continue
        if len(ampersands) == 1 and any(arguments and not arguments.startswith("-n") for arguments in waits):
            continue
        return True
    return False
DEFAULT_DETECTIONS = "/tmp/claude-1000/agent-hook-detections.jsonl"

def findings_for(command, run_in_background=False):
    if os.environ.get("BASH_COMMAND_DETECTOR_DISABLE_CHECK") == "1":
        return []
    findings = []
    if PATTERN_PROCESS_COMMAND.search(command):
        findings.append("按模式匹配的进程命令（pgrep -f / pkill -f / killall）")
    if WAIT_LOOP.search(command) and not re.search(r"\btimeout\b", command):
        findings.append("没有 timeout 的等待循环")
    if run_in_background and puts_itself_in_background(command) and os.environ.get("BASH_COMMAND_DETECTOR_DISABLE_SELF_BACKGROUND") != "1":
        findings.append("run_in_background 里又自己放后台（nohup / setsid / disown / 后面没有 wait 的 &）：完成通知当场发出，跑完的那个不会叫醒你")
    return findings

def record(hook_input, detections_path):
    """检出就追加一行；返回写了几行。永远不拦命令。"""
    tool_input = hook_input.get("tool_input") or {}
    command = tool_input.get("command") or ""
    findings = findings_for(command, bool(tool_input.get("run_in_background")))
    if not findings:
        return 0
    entry = {
        "time": datetime.now(timezone.utc).isoformat(),
        "session_id": hook_input.get("session_id"),
        "agent_id": hook_input.get("agent_id"),
        "agent_type": hook_input.get("agent_type") or "主 agent",
        "transcript_path": hook_input.get("transcript_path"),
        "command": command[:500],
        "findings": findings,
    }
    os.makedirs(os.path.dirname(detections_path), exist_ok=True)
    with open(detections_path, "a", encoding="utf-8") as handle:
        handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    return 1

def selftest(hook_dir):
    work = tempfile.mkdtemp(prefix="bash-command-detector-")
    detections = os.path.join(work, "detections.jsonl")
    cases = [
        ("普通命令", "cargo test --release", 0),
        ("带 timeout 的等待循环", "timeout 600 bash -c 'until grep -q done log; do sleep 10; done'", 0),
        ("while read 读文件", 'while read -r line; do echo "$line"; done < list.txt', 0),
        ("grep 里出现 pgrep 这个词", "grep -n 'pgrep' notes.md", 0),
        ("没有 timeout 的 until 等待", 'until grep -q "^exit=" log; do sleep 10; done', 1),
        ("pgrep -f", "pgrep -f e154-binary", 1),
        ("pkill -f", "pkill -f cargo", 1),
        ("killall", "killall cargo", 1),
        ("后台起的 nohup … & 加 disown", "nohup nice -n 19 bash gate.sh > gate.log 2>&1 & echo $!; disown", 1, True),
        ("后台起的命令末尾单独一个 &", "bash cache-keepalive.sh > keepalive.log 2>&1 &", 1, True),
        ("后台起的命令只有 2>&1、|& 与 &&", "cargo build 2>&1 | tail -3 && echo ok; make |& tee out", 0, True),
        ("后台起的普通命令", "bash cache-keepalive.sh", 0, True),
        ("后台起的并行加 wait", "bash a.sh > a.log 2>&1 & bash b.sh > b.log 2>&1 & wait", 0, True),
        ("后台起的 & 在 wait 之后", "wait; bash late.sh &", 1, True),
        ("前台命令里的 & 不算（不是 run_in_background）", "sleep 1 & wait", 0),
    ]
    results = []
    for label, command, want, *background in cases:
        tool_input = {"command": command, "run_in_background": bool(background and background[0])}
        written = record({"tool_name": "Bash", "session_id": "s", "tool_input": tool_input}, detections)
        results.append((label, want, written))
    # 走真实入口：从标准输入喂 JSON，退出码必须是 0（不拦），检出要落进文件
    script = os.path.join(hook_dir, "bash-command-detector.sh")
    before = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    completed = subprocess.run(["bash", script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {"command": "pgrep -f x"}}),
                               capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    after = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    results.append(("stdin:检出也放行（退出码 0）", 0, completed.returncode))
    results.append(("stdin:检出落进文件", 1, after - before))
    subprocess.run(["rm", "-rf", work])
    failures = [item for item in results if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 findings_for() 与 record()；BASH_COMMAND_DETECTOR_DISABLE_CHECK 或 _DISABLE_SELF_BACKGROUND 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过：按模式找进程、没超时的等待循环、run_in_background 里又自己放后台记进检出记录，普通命令、重定向里的 & 与前台的 & 不记，入口一律放行（查了 {len(results)} 种）")
    return 0

def main():
    hook_dir = sys.argv[1]
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    try:
        hook_input = json.load(sys.stdin)
        record(hook_input, os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS)
    except Exception:
        pass
    return 0

sys.exit(main())
PY
