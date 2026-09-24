#!/usr/bin/env bash
# PostToolUse hook（Write、Edit、Bash）：书记官（项目 subagent kb-scribe）每次写入成功之后，核这次写入的更新点与相关记录跟没跟上。
#
# 写了哪类文件、要跟上哪些相关记录、用哪几道门禁阶段核，登记在同目录的 kb-scribe-followups.tsv（唯一登记位）。
# 钩子按写到的路径挑出那几道阶段在仓库根上跑，红的那几道把 ✗ 与 → 两类行交回给书记官（JSON 的 additionalContext），
# 同时往检出记录（默认 /tmp/claude-1000/agent-hook-detections.jsonl，环境变量 AGENT_HOOK_DETECTIONS 可改）追加一行，
# 主 agent 的看门狗读得到。它只检出、不拦：写入已经发生，退出码恒为 0（main-agent.md「hook 只负责检出」）。
#
# 写入怎么认：Write / Edit 直接取 tool_input.file_path。Bash 要两条都成立：命令本身像一次写 kb（调了 WRITER_COMMAND
# 里那几个写 kb 的脚本，或 >、tee、sed -i、mv、cp、rm 作用在 .claude/kb/ 上；只读的 grep、cat 不算），
# 再认它碰了哪些 kb 文件：命令里点名了 .claude/kb/ 下的路径就只认这几份（几个会话共写一个仓时，按修改时间收会把
# 别人同一段时间里改的 kb 文件也算到书记官头上）；一个都没点名（21 号 --write 这类生成器）才退回按修改时间收
# 上一次核过之后变新的文件（按 agent_id 记在状态目录里；第一次取这个 agent 会话记录的开工时刻）。
# 输入里没有 agent_type、或 agent_type 不是 kb-scribe：什么都不做。
#
#   kb-scribe-followup.sh             # 从 stdin 读 hook 的 JSON
#   kb-scribe-followup.sh --selftest  # 在临时仓里走一遍；KB_SCRIBE_FOLLOWUP_BREAK=skip-stages 时自检必须判红
#   KB_SCRIBE_FOLLOWUP_AGENT=<别的 agent 名>  只供实测：把「认哪个 agent 为书记官」换掉，不动 kb 就能验证反馈真的送到了子 agent
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import json, os, re, subprocess, sys, tempfile, time
from datetime import datetime, timezone

HOOK_DIR = sys.argv[1]
TABLE = os.path.join(HOOK_DIR, "kb-scribe-followups.tsv")
GATE_DIR = os.path.join(os.path.dirname(HOOK_DIR), "gate.d")
STATE_DIR = os.environ.get("KB_SCRIBE_FOLLOWUP_STATE") or "/tmp/claude-1000/kb-scribe-followup"
DEFAULT_DETECTIONS = "/tmp/claude-1000/agent-hook-detections.jsonl"
SCRIBE = os.environ.get("KB_SCRIBE_FOLLOWUP_AGENT") or "kb-scribe"  # 只供实测「钩子真的送到了子 agent」时换成别的 agent 名
FALLBACK_FIRST_WINDOW_SECONDS = 60
# 脚本名要在调用的位置上（跟在 python3 后面，或在命令开头、;、&&、| 之后）：find -iname 'replace-once.py' 这类只读命令不算
WRITER_COMMAND = re.compile(
    r"(?:\bpython3?\s+(?:-\S+\s+)*|(?:^|[;&|(]\s*))[^\s;&|]*(replace-once|replace-batch|insert-row|relabel-item|sweep-term)\.py\b(?![^;&|]*--selftest)"
    r"|21-decision-items-sync\.sh\s+--write|49-history-brief\.sh\s+--write|lib-history-brief\.py\s+write"
    r"|(>>?|\btee\b|\bsed\s+-i|\bmv\b|\bcp\b|\brm\b)[^|;&]*\.claude/kb")
STAGE_TIMEOUT_SECONDS = 30
LINES_PER_RED_STAGE = 12


def load_table(path):
    """返回 ([(正则, [阶段], 说明)], [配置错])。"""
    rows, problems = [], []
    try:
        lines = open(path, encoding="utf-8").read().splitlines()
    except OSError as error:
        return [], [f"读不到登记表 {path}：{error}"]
    for number, line in enumerate(lines, 1):
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) < 2:
            problems.append(f"登记表第 {number} 行不足两列（路径正则、阶段）")
            continue
        try:
            pattern = re.compile(fields[0])
        except re.error as error:
            problems.append(f"登记表第 {number} 行的路径正则写坏了：{error}")
            continue
        stages = fields[1].split()
        missing = [stage for stage in stages if not os.path.isfile(os.path.join(GATE_DIR, stage))]
        if missing:
            problems.append(f"登记表第 {number} 行登记的阶段不存在：{' '.join(missing)}")
        rows.append((pattern, [stage for stage in stages if stage not in missing], fields[2].lstrip("# ") if len(fields) > 2 else ""))
    return rows, problems


KB_PATH_IN_COMMAND = re.compile(r'\.claude/kb/[^\s\'"`;|&<>]+')


def touched_paths(hook_input, root):
    """这次写入碰到的 .claude/kb/ 下的路径（从仓库根起）。"""
    tool = hook_input.get("tool_name")
    if tool in ("Write", "Edit"):
        file_path = (hook_input.get("tool_input") or {}).get("file_path") or ""
        absolute = os.path.abspath(os.path.join(root, file_path))
        relative = os.path.relpath(absolute, root)
        return [relative] if relative.startswith(".claude/kb/") else []
    command = (hook_input.get("tool_input") or {}).get("command") or ""
    if tool != "Bash" or not WRITER_COMMAND.search(command):
        return []
    # 命令里点名了 kb 路径（定点替换、插行这类）就只认这几份：按修改时间收会把别的会话同时改的 kb 文件也算到书记官头上
    named = sorted({match.group(0).rstrip('.,;:)\'"`\\') for match in KB_PATH_IN_COMMAND.finditer(command)})
    named = [path for path in named if os.path.exists(os.path.join(root, path))]
    if named:
        return named
    # 一个路径都没点名（21 号 --write 这类生成器）才退回按修改时间收
    state_file = os.path.join(STATE_DIR, f"{hook_input.get('agent_id') or 'unknown'}.last")
    try:
        since = float(open(state_file).read().strip())
    except (OSError, ValueError):
        since = agent_started_at(hook_input.get("transcript_path"))
    found = []
    for directory, _, files in os.walk(os.path.join(root, ".claude", "kb")):
        for name in files:
            absolute = os.path.join(directory, name)
            try:
                if os.path.getmtime(absolute) > since:
                    found.append(os.path.relpath(absolute, root))
            except OSError:
                continue
    return sorted(found)


def agent_started_at(transcript_path):
    """这个 agent 开工的时刻：会话记录第一行的 timestamp；取不到就往前退一小段。"""
    try:
        with open(transcript_path, encoding="utf-8") as handle:
            for line in handle:
                stamp = json.loads(line).get("timestamp")
                if stamp:
                    return datetime.fromisoformat(stamp.replace("Z", "+00:00")).timestamp()
    except (OSError, TypeError, ValueError):
        pass
    return time.time() - FALLBACK_FIRST_WINDOW_SECONDS


def remember_check(hook_input, started):
    try:
        os.makedirs(STATE_DIR, exist_ok=True)
        with open(os.path.join(STATE_DIR, f"{hook_input.get('agent_id') or 'unknown'}.last"), "w") as handle:
            handle.write(str(started))
    except OSError:
        pass


def run_stage(stage, root):
    """返回 (退出码, 要交回的行)。"""
    if os.environ.get("KB_SCRIBE_FOLLOWUP_BREAK") == "skip-stages":
        return 0, []
    try:
        completed = subprocess.run(["bash", os.path.join(GATE_DIR, stage), root], cwd=root,
                                   capture_output=True, text=True, timeout=STAGE_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        return 124, [f"没在 {STAGE_TIMEOUT_SECONDS} 秒内跑完，这一道这次没核成——收尾时单独跑一次"]
    output = (completed.stdout + completed.stderr).splitlines()
    kept = [line.rstrip() for line in output if re.match(r"\s*(✗|→|⇒)", line)]
    return completed.returncode, (kept or [line.rstrip() for line in output[-3:]])[:LINES_PER_RED_STAGE]


def record_detection(hook_input, paths, reds):
    entry = {
        "time": datetime.now(timezone.utc).isoformat(),
        "session_id": hook_input.get("session_id"),
        "agent_id": hook_input.get("agent_id"),
        "agent_type": hook_input.get("agent_type"),
        "transcript_path": hook_input.get("transcript_path"),
        "command": f"{hook_input.get('tool_name')} {' '.join(paths)}"[:500],
        "findings": [f"书记官写入之后相关记录没跟上：{stage}" for stage, _, _ in reds],
    }
    path = os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS
    try:
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError:
        pass


def decide(hook_input):
    """返回要交回书记官的文字；None 表示这一次不关它的事。"""
    if hook_input.get("agent_type") != SCRIBE:
        return None
    root = os.environ.get("CLAUDE_PROJECT_DIR") or hook_input.get("cwd") or os.getcwd()
    started = time.time()
    paths = touched_paths(hook_input, root)
    if not paths:
        return None  # 不是写 kb 的调用：不挪「上一次核过的时刻」，否则之前那次写入会被当成已经核过
    remember_check(hook_input, started)
    rows, problems = load_table(TABLE)
    chosen, reasons = [], []
    for pattern, stages, why in rows:
        if any(pattern.search(path) for path in paths):
            reasons.append(why)
            chosen += [stage for stage in stages if stage not in chosen]
    reds = []
    for stage in chosen:
        code, lines = run_stage(stage, root)
        if code not in (0, 77):
            reds.append((stage, code, lines))
    head = f"书记官写入之后核相关记录（{' '.join(paths[:6])}{' 等' if len(paths) > 6 else ''}）："
    if problems:
        head += "\n  ⚠️ 登记表 .claude/hooks/kb-scribe-followups.tsv 有配置错：" + "；".join(problems)
    if not reds:
        return head + f"跑了 {len(chosen)} 道（{' '.join(s.split('-')[0] for s in chosen)}），相关记录都跟上了。" if chosen else None
    record_detection(hook_input, paths, reds)
    body = [head + f"跑了 {len(chosen)} 道，{len(reds)} 道没过——相关记录还没跟上，按下面每一道的出路补："]
    for stage, code, lines in reds:
        body.append(f"  【{stage}，退出码 {code}】")
        body += [f"    {line.strip()}" for line in lines]
    body.append("  判断归属：红点名的文件若不在你这一批规格里，多半是别的会话在飞的改动——照报给主 agent，不修；"
                "在你这一批里的，补完再写下一处。这个钩子只检出、不拦写入，慢的阶段（10、32、70 号与 doc-lint）不在这里跑，收尾时照定义全跑。")
    return "\n".join(body)


def emit(text):
    if text:
        print(json.dumps({"hookSpecificOutput": {"hookEventName": "PostToolUse", "additionalContext": text}}, ensure_ascii=False))


def selftest():
    script = os.path.join(HOOK_DIR, "kb-scribe-followup.sh")
    failures = []
    checked = 0
    with tempfile.TemporaryDirectory() as work:
        env_git = dict(os.environ, GIT_AUTHOR_NAME="t", GIT_AUTHOR_EMAIL="t@t", GIT_COMMITTER_NAME="t", GIT_COMMITTER_EMAIL="t@t")
        decisions = os.path.join(work, ".claude", "kb", "decisions")
        os.makedirs(decisions)
        os.makedirs(os.path.join(work, ".claude", "kb", "decisions-history"))
        decision = os.path.join(decisions, "01-样本.md")
        open(decision, "w", encoding="utf-8").write("## D1 样本 —— 已定\n\n### 已定项\n\n| # | 分项 | 结论 |\n|---|---|---|\n| 1 | **甲** | 取甲 **状态：已定。** |\n\n## 历史版本\n\n无。\n")
        subprocess.run(["git", "init", "-q", "-b", "master", work], check=True, env=env_git)
        subprocess.run(["git", "-C", work, "add", "-A"], check=True, env=env_git)
        subprocess.run(["git", "-C", work, "commit", "-qm", "base"], check=True, env=env_git)
        # 决策正文改了五行、变更史一条没加：门禁 30 号该红
        with open(decision, "a", encoding="utf-8") as handle:
            handle.write("\n补一段。\n第二行。\n第三行。\n第四行。\n第五行。\n")
        state = os.path.join(work, "state")
        detections = os.path.join(work, "detections.jsonl")

        def run(hook_input):
            nonlocal checked
            checked += 1
            environment = dict(os.environ, CLAUDE_PROJECT_DIR=work, KB_SCRIBE_FOLLOWUP_STATE=state, AGENT_HOOK_DETECTIONS=detections)
            completed = subprocess.run(["bash", script], input=json.dumps(hook_input), capture_output=True, text=True, env=environment)
            return completed.returncode, completed.stdout.strip()

        edit = {"tool_name": "Edit", "agent_type": SCRIBE, "agent_id": "a1", "tool_input": {"file_path": decision}}
        code, out = run(edit)
        context = json.loads(out)["hookSpecificOutput"]["additionalContext"] if out else ""
        if code != 0 or "30-decision-history.sh" not in context or "没跟上" not in context:
            failures.append(f"① 书记官改了决策正文、变更史没跟：应交回 30 号的红，实际退出码 {code}、交回「{context[:120]}」")
        if not os.path.isfile(detections) or "30-decision-history.sh" not in open(detections, encoding="utf-8").read():
            failures.append("① 检出记录里没有这一次")
        code, out = run(dict(edit, agent_type=None))
        if out:
            failures.append(f"② 主 agent 的写入不该触发，实际交回「{out[:80]}」")
        code, out = run(dict(edit, tool_input={"file_path": os.path.join(work, "notes.md")}))
        if out:
            failures.append(f"③ 写 kb 以外的文件不该触发，实际交回「{out[:80]}」")
        bash_call = {"tool_name": "Bash", "agent_type": SCRIBE, "agent_id": "a1", "tool_input": {"command": "grep -n x .claude/kb/decisions/01-样本.md"}}
        time.sleep(0.05)
        with open(decision, "a", encoding="utf-8") as handle:
            handle.write("再补一行。\n")
        code, out = run(bash_call)
        if out:
            failures.append(f"④ 只读的 Bash 不该触发（哪怕 kb 刚被别人改过），实际交回「{out[:80]}」")
        code, out = run(dict(bash_call, tool_input={"command": "python3 research/scripts/replace-once.py .claude/kb/decisions/01-样本.md 旧 新"}))
        if "30-decision-history.sh" not in out:
            failures.append(f"⑤ Bash 改了 kb（mtime 变新）应当触发，实际交回「{out[:120]}」")
        # ⑥ 别的会话同时改了另一份 kb 文件：命令点名了哪份就只认哪份，不按修改时间把别人的也算进来
        other = os.path.join(work, ".claude", "kb", "invariants.md")
        time.sleep(0.05)
        open(other, "w", encoding="utf-8").write("# 不变量\n\n别的会话刚改的。\n")
        named_call = dict(bash_call, agent_id="a6", tool_input={"command": "python3 research/scripts/insert-row.py .claude/kb/decisions/01-样本.md --after x --line y"})
        checked += 1
        attributed = touched_paths(named_call, work)
        if attributed != [".claude/kb/decisions/01-样本.md"]:
            failures.append(f"⑥ 命令只点名了决策文件，却认了「{attributed}」（别的会话改的 invariants.md 不该算进来）")
        # ⑦⑧ 命令里出现写 kb 的脚本名、但不是在调用它写 kb：别的会话刚改过 kb，也不该按修改时间把那些文件算进来
        time.sleep(0.05)
        open(other, "a", encoding="utf-8").write("又改了一行。\n")
        for mark, command in (("⑦", "find research/scripts -iname 'replace-batch.py' -o -iname 'replace-once.py'"),
                              ("⑧", "python3 research/scripts/replace-once.py --selftest")):
            checked += 1
            attributed = touched_paths(dict(bash_call, agent_id="a7", tool_input={"command": command}), work)
            if attributed:
                failures.append(f"{mark} 「{command}」不写 kb，却认了「{attributed}」")
    if checked != 8:
        failures.append(f"自检该跑 8 格，实际跑了 {checked} 格")
    if failures:
        print("  ✗ kb-scribe-followup.sh 自检没过：")
        for failure in failures:
            print(f"      {failure}")  # gate-lint:detail
        print("  → 怎么办：照上面每一条改 decide() / touched_paths()；登记表在 .claude/hooks/kb-scribe-followups.tsv。")
        return 1
    print(f"  ✓ kb-scribe-followup.sh 自检通过（{checked} 格：书记官改决策正文交回 30 号的红并记检出、主 agent 不触发、写 kb 以外不触发、只读 Bash 在 kb 变了之后也不触发、写 kb 的 Bash 触发、命令点名了哪份 kb 就只认哪份、只找脚本名的 find 与脚本自检不算写 kb）")
    return 0


if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
    sys.exit(selftest())
try:
    hook_input = json.load(sys.stdin)
except (json.JSONDecodeError, ValueError):
    sys.exit(0)
emit(decide(hook_input))
sys.exit(0)
PY
