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
#   ③ `run_in_background` 起的命令里又自己放后台：外层 shell 当场退出，
#      harness 的完成通知当场发出，真正在跑的东西跑完不再叫醒谁（2026-09-18 一个门禁分诊员这样起门禁与三个缓存计时器，
#      records/2026-09-16-subagent拆分提案.md 第三十三节）。认的形态：`disown`、`coproc`、`setsid -f/--fork`、同一行带单独 `&` 的 `nohup`、
#      `tmux new(-session) … -d`、`screen -dm…`、不带 --wait/--pipe/--pty/--scope 的 `systemd-run`、`start-stop-daemon -b`、
#      子 shell 或命令替换里的 `&`，以及后面没有等它的单独 `&`（不是 `&&`、`>&`、`&>`、`|&`、`<&`）——等它指：之后在命令位置有
#      不带参数的 `wait`，只起了一个 `&` 时带 pid 的 `wait` 也算（`wait -n` 不算），或之后有 `tail --pid` / `while|until … kill -0` 轮询。
#      喂给非 shell 解释器的 heredoc 正文先剥掉再找（Python 的按位与、Rust 的引用不算）。
#      判不到的：前台起的 `… &`（run_in_background 为假，hook 那一刻看不出之后会不会结束本轮去等；由看门狗那一侧的
#      「结束本轮、手里没有在跑的后台任务、也没交回」告警兜）；引号里当数据用的 `&`（sed 替换串、URL 参数）会误记，
#      不能一概跳过引号，`bash -c '… &'` 里的 `&` 是真放后台；`… & (wait)`、`bash -c '… &'; wait` 这种等不到的 `wait` 认不出。
# 主 agent 的看门狗（research/scripts/agent-watch.py watch）每 15 秒读一次检出记录，读到本会话的检出就退出、叫醒主 agent。
# 三种检出都先剥掉喂给非 shell 命令的 heredoc 正文（往报告里写一句「别用 pgrep -f」不是在用它；2026-09-18 撞过，
# `records/2026-09-16-subagent拆分提案.md` 第二十五节）；喂给 bash / sh 的 heredoc 正文照查。其余只看顶层命令文本，
# 引号里当数据写的这些字（`echo "pgrep -f"`）也会被记一条，主 agent 看了判断即可。
#
#   bash-command-detector.sh             # 从 stdin 读 hook 的 JSON，永远退出 0
#   bash-command-detector.sh --selftest  # 走一遍检出与不检出；BASH_COMMAND_DETECTOR_DISABLE_CHECK=1、
#                                        # BASH_COMMAND_DETECTOR_DISABLE_SELF_BACKGROUND=1（只关第三种）或
#                                        # BASH_COMMAND_DETECTOR_KEEP_HEREDOC_BODIES=1（前两种不剥 heredoc 正文）时自检必须判红
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（与 agent-write-scope.sh 同一个坑）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import json, os, re, subprocess, sys, tempfile
from datetime import datetime, timezone

PATTERN_PROCESS_COMMAND = re.compile(r"\bpgrep\s+(?:-\w+\s+)*-\w*f|\bpkill\s+(?:-\w+\s+)*-\w*f|\bkillall\b")
WAIT_LOOP = re.compile(r"\b(until|while)\b[^\n]*?;\s*do\b[\s\S]*?\bsleep\b")
SELF_DETACH = re.compile(
    r"\bsetsid(?:\s+-[-\w]+)*?\s+(?:--fork\b|-[a-eg-z]*f[a-z]*\b)"
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
    executed = command if os.environ.get("BASH_COMMAND_DETECTOR_KEEP_HEREDOC_BODIES") == "1" else strip_data_heredocs(command)
    if PATTERN_PROCESS_COMMAND.search(executed):
        findings.append("按模式匹配的进程命令（pgrep -f / pkill -f / killall）")
    if WAIT_LOOP.search(executed) and not re.search(r"\btimeout\b", executed):
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
        ("写进报告的 heredoc 正文里提到 pgrep -f", "cat >> report.md <<'EOF'\n等进程别用 pgrep -f，它会命中自己\nEOF", 0),
        ("写进笔记的 heredoc 正文里是一段没超时的等待循环", "cat > note.md <<'EOF'\nuntil grep -q done log; do sleep 10; done\nEOF", 0),
        ("喂给 bash 的 heredoc 里真用 pgrep -f", "bash <<'EOF'\npgrep -f e154-binary\nEOF", 1),
        ("heredoc 之后的命令里用 pgrep -f", "cat > note.md <<EOF\n说明\nEOF\npgrep -f e154-binary", 1),
        ("后台起的 nohup … & 加 disown", "nohup nice -n 19 bash gate.sh > gate.log 2>&1 & echo $!; disown", 1, True),
        ("后台起的命令末尾单独一个 &", "bash cache-keepalive.sh > keepalive.log 2>&1 &", 1, True),
        ("后台起的命令只有 2>&1、|& 与 &&", "cargo build 2>&1 | tail -3 && echo ok; make |& tee out", 0, True),
        ("后台起的普通命令", "bash cache-keepalive.sh", 0, True),
        ("后台起的并行加 wait", "bash a.sh > a.log 2>&1 & bash b.sh > b.log 2>&1 & wait", 0, True),
        ("后台起的 & 在 wait 之后", "wait; bash late.sh &", 1, True),
        ("前台命令里的 & 不算（不是 run_in_background）", "sleep 1 & wait", 0),
        ('bg-notify-r1 X01', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & echo "started pid $!"', 1, True),
        ('bg-notify-r1 X02', '(nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 &)', 1, True),
        ('bg-notify-r1 X03', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & echo "gate.sh started as pid $!; I will wait for the notification"', 1, True),
        ('bg-notify-r1 X04', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & bash research/scripts/cache-keepalive.sh & wait $!', 1, True),
        ('bg-notify-r1 X05', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & bash research/scripts/cache-keepalive.sh & wait -n', 1, True),
        ('bg-notify-r1 X06', '(nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 &); wait', 1, True),
        ('bg-notify-r1 X07', 'pid=$(nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & echo $!); echo "gate pid $pid"; wait "$pid"', 1, True),
        ('bg-notify-r1 X08', 'coproc GATE { nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1; }', 1, True),
        ('bg-notify-r1 X09', "tmux new-session -d -s gate 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1'", 1, True),
        ('bg-notify-r1 X10', "screen -dmS gate bash -c 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1'", 1, True),
        ('bg-notify-r1 X11', 'systemd-run --user --collect --unit gate-run nice -n 19 bash .claude/scripts/gate.sh --staged', 1, True),
        ('bg-notify-r1 X12', "nice -n 19 python3 - > /tmp/claude-1000/gate-triage-commit-0918/rerun.log 2>&1 <<'EOF' &\nimport subprocess\np = subprocess.Popen(['bash', 'research/scripts/replay.sh'])\np.wait()\nEOF", 1, True),
        ('bg-notify-r1 X13', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 &', 0, False),
        ('bg-notify-r1 F01', 'nohup nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1', 0, True),
        ('bg-notify-r1 F02（第三种不记；它的轮询循环没有 timeout，记的是第二种）', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & pid=$!; while kill -0 "$pid" 2>/dev/null; do sleep 20; done; tail -3 /tmp/claude-1000/gate-triage-commit-0918/gate-run.log', 1, True),
        ('bg-notify-r1 F03', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & tail --pid=$! -f /tmp/claude-1000/gate-triage-commit-0918/gate-run.log > /dev/null', 0, True),
        ('bg-notify-r1 F04', "nice -n 19 python3 - <<'EOF' > /tmp/claude-1000/gate-triage-commit-0918/tally.txt\nimport json\nrows = [json.loads(line) for line in open('/tmp/claude-1000/gate-triage-commit-0918/events.jsonl')]\nprint(sum(1 for row in rows if row['flags'] & 0x4))\nEOF", 0, True),
        ('bg-notify-r1 F05', "cat > /tmp/claude-1000/impl-probe/copy/crates/singlefs-harness/tests/probe.rs <<'EOF'\nfn digest(bytes: &[u8]) -> u64 { bytes.iter().map(|&b| b as u64).sum() }\n#[test]\nfn probe() { assert_eq!(digest(&[1, 2]), 3); }\nEOF\ncd /tmp/claude-1000/impl-probe/copy && nice -n 19 cargo test --release --test probe 2>&1 | tail -20", 0, True),
        ('bg-notify-r1 F06', "nice -n 19 cargo test --release 2>&1 | sed -E 's/^test (.*) \\.\\.\\. FAILED$/!! &/' > /tmp/claude-1000/impl-probe/test.log", 1, True),
        ('bg-notify-r1 F07', 'setsid -w nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1', 0, True),
        ("裸 setsid 不 fork、外层照样等", "setsid bash long.sh > long.log 2>&1 < /dev/null", 0, True),
        ("setsid -f 放后台", "setsid -f bash long.sh > long.log 2>&1 < /dev/null", 1, True),
        ("setsid 跟着的 tail -f 不是 setsid 的选项", "setsid tail -f log.txt", 0, True),
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
        print("    → 看 findings_for() 与 record()；BASH_COMMAND_DETECTOR_DISABLE_CHECK、_DISABLE_SELF_BACKGROUND 或 _KEEP_HEREDOC_BODIES 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过：按模式找进程、没超时的等待循环、run_in_background 里又自己放后台记进检出记录，普通命令、重定向里的 & 、前台的 & 与写进文件的 heredoc 正文不记，入口一律放行（查了 {len(results)} 种）")
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
