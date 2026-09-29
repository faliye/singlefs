#!/usr/bin/env bash
# PreToolUse hook（SubagentHandback）：子 agent 交回之前拦四样——自己起的后台任务还在跑（全部子 agent），交回正文太长、三方腿与核查员报告里的引文对不上、
# 书记员与执行员的绿行不报数（这三样只判项目子 agent）。
# hook-events: PreToolUse:SubagentHandback
# gate-similar: handback-scratch-check.sh 同挂 PreToolUse[SubagentHandback]，但它判的是临时目录里自己建的编译目录与仓副本删没删（上游副本，本仓不许就地改），判据与这里四样没有一条相同（④ 判的是自己起的后台任务还在不在跑，不是临时目录），并进去要改上游
# gate-similar: continuation-guard.sh 也按 agent 号去主会话目录下找子 agent 的会话记录，但它挂 SendMessage、判收件人交回过没有；这里只取子 agent 第一条记录的时刻当「开工时刻」，找法两行，不值得抽共用
# admission: always Claude Code 在子 agent 每次调交回工具之前调它，判的是这一次交回的正文与它点名的报告
# run-condition: command python3
#
# 判法。任一条成立就拒（退出 2，stderr 写原因与出路）：
#   ④ 全部子 agent（输入里有 agent_id、不是 main；general-purpose 也判，主 agent 不判）：它自己起的后台任务（它会话记录
#      <主会话记录去掉 .jsonl>/subagents/agent-<agent_id>.jsonl 里 run_in_background 的工具结果、或前台跑满超时被挪进后台的，带输出文件路径的那些）
#      此刻还有进程开着输出文件，就是还在跑：拒，按任务列出最上层的进程号、已跑多久、命令与整棵子树从最底层往上的停止次序，
#      出路是结束本轮等完成通知再交回，或按进程号逐个停掉再交回。认法与看门狗「交回之后后台还在跑」同一套，从 research/scripts/agent-watch.py 导入
#      （background_start_in_result、own_background_tasks_still_running），不另抄一份。命令里自己用 `&`、`nohup` 放出去又改了输出去向的进程认不出（与看门狗同一处射程）。
#      找不到它的会话记录、导入不了 agent-watch.py 就不判这一条，stderr 写一句。
# 下面三条只判项目子 agent（`.claude/agents/<agent_type>.md` 在的）；主 agent、general-purpose 这类不判：
#   ① 交回正文超过 HANDBACK_CHARACTER_LIMIT 个字符（按 python 的 len 数，默认 4000；原 2000 时 8 天里拒了 67 次，每次都在最大的上下文上重写整份交回）：全文写进报告文件，交回只写结论、报告路径与 sha256；
#   ② three-way-forward / three-way-attack / three-way-defense / three-way-verifier：交回里点名的 research/prompts/…-output.md 逐份交给
#      research/scripts/cite-check.py（--root 项目根，--unchanged-since 取这个子 agent 会话记录第一条的时刻），有对不上的拒；交回里一份报告都没点名也拒；
#      找不到它的会话记录（认不出开工时刻）就不判这一条，stderr 写一句；
#   ③ kb-scribe / experiment-runner：交回正文与它点名的报告文件（research/prompts/ 或 /tmp/claude-<uid>/ 下的 .md）里，以 ✓ 起头的行没有数字、
#      或写「查了 0 项」这类零项的，拒，逐行列出——绿行要连它报的查了多少项一起贴，0 项或没有数的按没判写。
# 这一道自己出错（读不了报告、cite-check 跑不起来、JSON 解析不了）不拒：放行，错误写进 stderr。
#
#   handback-guard.sh             # 从 stdin 读 hook 的 JSON
#   handback-guard.sh --selftest  # 在临时目录里走一遍放行与拒绝；HANDBACK_GUARD_BREAK=no-limit / no-cite / cite-any-agent / no-green-count /
#                                 # zero-count-ok / judge-general-purpose / no-background 各自必须让它红
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$HOOK_DIR/../singlefs-ai-sop/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import json, os, re, shutil, subprocess, sys, tempfile
from datetime import datetime, timezone

HANDBACK_CHARACTER_LIMIT = int(os.environ.get("HANDBACK_CHARACTER_LIMIT", "4000"))
CITATION_CHECKED_AGENTS = {"three-way-forward", "three-way-attack", "three-way-defense", "three-way-verifier"}
GREEN_COUNT_CHECKED_AGENTS = {"kb-scribe", "experiment-runner"}
LEG_REPORT_MENTION = re.compile(r"research/prompts/[^\s`'\"，。；：、（）()「」]+-output\.md")
REPORT_MENTION = re.compile(r"(?:research/prompts/|/tmp/claude-\d+/)[^\s`'\"，。；：、（）()「」]+\.md")
GREEN_LINE = re.compile(r"^\s*(?:[-*>|]\s*)*✓")
# 编号不是数：阶段号、实验 / 决策 / 欠账 / 不变量号、阶段文件名、时刻
NOT_A_COUNT = re.compile(r"\d+\s*号|\b\d{2}-[\w.-]+|\b[EDC]\d+|\bI-[\d.]+|\d{1,2}:\d{2}")
ZERO_COUNT = re.compile(r"(?:查了|检查了?|扫了|判了|核了|共|跑了)\s*0\s*(?:项|个|条|份|处|种|行)")
BROKEN = os.environ.get("HANDBACK_GUARD_BREAK", "")


def is_project_agent(agent_type, project_root):
    if BROKEN == "judge-general-purpose" and agent_type:
        return True
    return bool(agent_type) and os.path.isfile(os.path.join(project_root, ".claude", "agents", f"{agent_type}.md"))


def started_at(hook_input):
    """这个子 agent 会话记录第一条的时刻（epoch 秒）；找不到返回 None。"""
    transcript_path = hook_input.get("transcript_path") or ""
    agent_id = hook_input.get("agent_id") or ""
    if not transcript_path.endswith(".jsonl") or not agent_id:
        return None
    own = os.path.join(transcript_path[: -len(".jsonl")], "subagents", f"agent-{agent_id}.jsonl")
    try:
        with open(own, encoding="utf-8", errors="replace") as handle:
            for line in handle:
                stamp = json.loads(line).get("timestamp")
                if stamp:
                    return datetime.fromisoformat(stamp.replace("Z", "+00:00")).timestamp()
    except (OSError, ValueError):
        return None
    return None


def resolved(path, project_root):
    return path if os.path.isabs(path) else os.path.join(project_root, path)


def citation_problems(message, hook_input, project_root, cite_check):
    reports = sorted(set(LEG_REPORT_MENTION.findall(message)))
    if not reports:
        return ["交回里没点名报告文件（research/prompts/<轮>-…-output.md）"]
    since = started_at(hook_input)
    if since is None:
        print("! handback-guard：找不到这个子 agent 的会话记录，认不出开工时刻，引文这一条没判（放行）", file=sys.stderr)
        return []
    command = [sys.executable, cite_check, *[resolved(path, project_root) for path in reports], "--root", project_root, "--unchanged-since", str(since)]
    try:
        result = subprocess.run(command, capture_output=True, text=True, timeout=110)
    except (OSError, subprocess.TimeoutExpired) as error:
        print(f"! handback-guard：cite-check 跑不起来，引文这一条没判（放行）：{error}", file=sys.stderr)
        return []
    if result.returncode == 1:
        return [line.strip() for line in result.stdout.splitlines() if line.strip().startswith("✗")]
    if result.returncode != 0:
        print(f"! handback-guard：cite-check 退出码 {result.returncode}，引文这一条没判（放行）：{result.stdout[-300:]}{result.stderr[-300:]}", file=sys.stderr)
    return []


def green_count_problems(message, project_root):
    texts = [("交回正文", message)]
    for path in sorted(set(REPORT_MENTION.findall(message))):
        absolute = resolved(path, project_root)
        if os.path.isfile(absolute):
            try:
                texts.append((path, open(absolute, encoding="utf-8", errors="replace").read()))
            except OSError as error:
                print(f"! handback-guard：读不了 {absolute}（{error}），这一份的绿行没判", file=sys.stderr)
    problems = []
    for name, text in texts:
        for number, line in enumerate(text.splitlines(), 1):
            if not GREEN_LINE.match(line):
                continue
            if BROKEN != "no-green-count" and not re.search(r"\d", NOT_A_COUNT.sub("", line)):
                problems.append(f"{name}:{number} 绿行没有数：{line.strip()[:120]}")
            elif BROKEN != "zero-count-ok" and ZERO_COUNT.search(line):
                problems.append(f"{name}:{number} 绿行报的是 0 项：{line.strip()[:120]}")
    return problems


def background_problems(hook_input, agent_watch_path):
    """④ 这个子 agent 自己起的后台任务此刻还在跑的：交 own_background_tasks_still_running 的结果；主 agent、判不了的交空表。"""
    agent_id = hook_input.get("agent_id") or ""
    transcript_path = hook_input.get("transcript_path") or ""
    if BROKEN == "no-background" or agent_id in ("", "main"):
        return []
    own = os.path.join(transcript_path[: -len(".jsonl")], "subagents", f"agent-{agent_id}.jsonl") if transcript_path.endswith(".jsonl") else ""
    if not os.path.isfile(own):
        print(f"! handback-guard：找不到这个子 agent 自己的会话记录（{own or '输入里没有主会话记录'}），后台任务这一条没判（放行）", file=sys.stderr)
        return []
    try:
        import importlib.util
        spec = importlib.util.spec_from_file_location("agent_watch", agent_watch_path)
        agent_watch = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(agent_watch)
        return agent_watch.own_background_tasks_still_running(own)
    except Exception as error:   # 文件不在、导入失败、读不了会话记录：这一条没判，放行
        print(f"! handback-guard：后台任务这一条没判（放行）：{type(error).__name__}：{error}", file=sys.stderr)
        return []


def background_refusal(running):
    lines = []
    for task_id, output, tops in running:
        for pid, seconds, command, stop_order in tops:
            lines.append(f"  任务 {task_id}（输出 {output}）：进程 {pid} 已跑 {int(seconds // 60)} 分 {int(seconds % 60)} 秒：{command[:160]}；"
                         f"停的次序（从最底层往上）：{' '.join(map(str, stop_order[:24]))}{' …' if len(stop_order) > 24 else ''}")
    return (f"✗ 交回时你自己起的后台任务还在跑（{len(running)} 个任务）：\n" + "\n".join(lines[:20]) +
            "\n→ 怎么办：要它的结果就结束本轮、不再调工具，等它的完成通知到了、把结果写进报告再交回；不要了就按上面的次序逐个跑 "
            "`python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>`，停完 `ps -o pid,ppid,args --ppid <进程号>` 再列一遍确认一个不剩，"
            "在报告「没做什么」里写明停了哪条、为什么，再交回（.claude/agent-common.md「不做」一节停自己起的一条链那一条）。")


def decide(hook_input, project_root, cite_check):
    """返回 (0, None) 放行；(2, 说明) 拒绝。"""
    running = background_problems(hook_input, os.path.join(os.path.dirname(cite_check), "agent-watch.py"))
    if running:
        return 2, background_refusal(running)
    agent_type = hook_input.get("agent_type")
    if not is_project_agent(agent_type, project_root):
        return 0, None
    message = str((hook_input.get("tool_input") or {}).get("message") or "")
    if BROKEN != "no-limit" and len(message) > HANDBACK_CHARACTER_LIMIT:
        return 2, (f"✗ {agent_type} 的交回正文 {len(message)} 字，超过 {HANDBACK_CHARACTER_LIMIT} 字。\n"
                   "→ 怎么办：全文写进报告文件（派发提示给的报告路径；没给就写进草稿目录），交回只写结论、报告路径与 sha256sum（.claude/agent-common.md「报告」一节）。")
    if (agent_type in CITATION_CHECKED_AGENTS or BROKEN == "cite-any-agent") and BROKEN != "no-cite":
        problems = citation_problems(message, hook_input, project_root, cite_check)
        if problems:
            return 2, ("✗ 报告里的引文对不上（research/scripts/cite-check.py）：\n" + "\n".join(f"  {line}" for line in problems[:20]) +
                       "\n→ 怎么办：按每处的提示改报告里的行号或引文——行号去被引文件里现查（grep -nF），不从背景材料里数；改完再跑一次 "
                       "python3 research/scripts/cite-check.py <报告> 看它转绿，再交回。")
    if agent_type in GREEN_COUNT_CHECKED_AGENTS:
        problems = green_count_problems(message, project_root)
        if problems:
            return 2, ("✗ 绿行没报查了多少项，或报的是 0 项：\n" + "\n".join(f"  {line}" for line in problems[:20]) +
                       "\n→ 怎么办：贴绿行时连它报的「查了多少项」一起贴；那一道报 0 项或根本没报数的，按没判写（「本次未判：…」），不写成 ✓（.claude/agent-common.md「门禁」一节）。")
    return 0, None


def selftest(hook_dir):
    work = tempfile.mkdtemp(prefix="handback-guard-selftest-")
    sleeper = None   # ④ 的样本：开着一个后台任务输出文件在睡的进程，自检自己起、自己停
    try:
        cite_check = os.path.join(os.path.dirname(os.path.dirname(hook_dir)), "research", "scripts", "cite-check.py")
        for agent in ("three-way-attack", "three-way-verifier", "kb-scribe", "experiment-runner", "sweep"):
            os.makedirs(os.path.join(work, ".claude", "agents"), exist_ok=True)
            open(os.path.join(work, ".claude", "agents", f"{agent}.md"), "w").write(f"# {agent}\n")
        os.makedirs(os.path.join(work, "kb"))
        open(os.path.join(work, "kb", "rule.md"), "w", encoding="utf-8").write("# 规则\n第一条：先查再写。\n")
        os.makedirs(os.path.join(work, "research", "prompts"))
        open(os.path.join(work, "research", "prompts", "r1-opus-output.md"), "w", encoding="utf-8").write("「第一条：先查再写」（`kb/rule.md:2`）\n")
        open(os.path.join(work, "research", "prompts", "r2-opus-output.md"), "w", encoding="utf-8").write("「第一条：先查再写」（`kb/rule.md:1`）\n")
        session = os.path.join(work, "session")
        os.makedirs(os.path.join(session, "subagents"))
        # 开工时刻取样本文件都建好之后：被引文件不算「开工之后改过」
        started = datetime.fromtimestamp(os.path.getmtime(os.path.join(work, "research", "prompts", "r2-opus-output.md")) + 5, timezone.utc)
        open(os.path.join(session, "subagents", "agent-a1.jsonl"), "w").write(json.dumps({"timestamp": started.isoformat()}) + "\n")
        # ④ a2 起的后台任务 blive 的输出文件有一个在睡的进程开着（交回时还在跑）；a3 起的 bdone 输出文件在、没有进程开着（跑完了）
        live_output, done_output = os.path.join(work, "tasks", "blive.output"), os.path.join(work, "tasks", "bdone.output")
        os.makedirs(os.path.dirname(live_output))
        open(done_output, "w").close()

        def background_started_record(task_id, output):
            return json.dumps({"type": "user", "timestamp": started.isoformat(), "message": {"role": "user", "content": [
                {"type": "tool_result", "tool_use_id": f"t-{task_id}",
                 "content": f"Command running in background with ID: {task_id}. Output is being written to: {output}. You will be notified when it completes."}]}})
        for agent_id, task_id, output in (("a2", "blive", live_output), ("a3", "bdone", done_output)):
            open(os.path.join(session, "subagents", f"agent-{agent_id}.jsonl"), "w").write(
                json.dumps({"timestamp": started.isoformat()}) + "\n" + background_started_record(task_id, output) + "\n")
        with open(live_output, "w") as output_handle:
            sleeper = subprocess.Popen(["sleep", "120"], stdout=output_handle, stderr=subprocess.STDOUT)
        scribe_report = os.path.join(work, "research", "prompts", "scribe-report.md")
        open(scribe_report, "w", encoding="utf-8").write("✓ 文档铁律检查通过（检查 486，跳过 0）\n✓ 通过\n")

        def case(label, agent_type, message, want, agent_id="a1"):
            hook_input = {"tool_name": "SubagentHandback", "tool_input": {"message": message}, "agent_type": agent_type,
                          "agent_id": agent_id, "transcript_path": session + ".jsonl"}
            return (label, want, decide(hook_input, work, cite_check)[0])
        cases = [
            case("超过上限的项目子 agent", "sweep", "字" * (HANDBACK_CHARACTER_LIMIT + 1), 2),
            case("正好在上限的项目子 agent", "sweep", "字" * HANDBACK_CHARACTER_LIMIT, 0),
            case("不是项目定义的 agent 不判", "general-purpose", "字" * (HANDBACK_CHARACTER_LIMIT + 1), 0),
            case("主 agent 不判", None, "字" * (HANDBACK_CHARACTER_LIMIT + 1), 0),
            case("攻方腿引文对上", "three-way-attack", "报告 research/prompts/r1-opus-output.md sha256 x", 0),
            case("攻方腿引文行号错", "three-way-attack", "报告 research/prompts/r2-opus-output.md sha256 x", 2),
            case("核查员没点名报告", "three-way-verifier", "核完了", 2),
            case("回扫员不核引文", "sweep", "报告 research/prompts/r2-opus-output.md", 0),
            case("书记员绿行带数", "kb-scribe", "✓ doc-decisions 检查通过（查了 12 项）", 0),
            case("书记员绿行没数", "kb-scribe", "✓ doc-registries 转绿", 2),
            case("书记员绿行 0 项", "kb-scribe", "✓ doc-registries 通过（查了 0 项）", 2),
            case("执行员报告文件里的绿行没数", "experiment-runner", f"报告 {os.path.relpath(scribe_report, work)}", 2),
            case("回扫员的绿行不判", "sweep", "✓ 通过", 0),
            case("交回时自己起的后台任务还在跑（项目子 agent）", "sweep", "报告 /tmp/x", 2, "a2"),
            case("交回时自己起的后台任务还在跑（通用 agent 也判）", "general-purpose", "交回", 2, "a2"),
            case("起过的后台任务都跑完了", "sweep", "交回", 0, "a3"),
            case("主 agent 不判后台任务", None, "交回", 0, None),
        ]
        live_refusal = decide({"tool_name": "SubagentHandback", "tool_input": {"message": "交回"}, "agent_type": "sweep", "agent_id": "a2",
                               "transcript_path": session + ".jsonl"}, work, cite_check)[1] or ""
        cases.append(("后台还在跑的拒绝说明列出进程号、命令与两条出路（等完成通知、按进程号停）", True,
                      all(part in live_refusal for part in (f"进程 {sleeper.pid}", "sleep 120", "blive", "完成通知", "proc.py stop"))))
        sleeper.kill()
        sleeper.wait()
        cases.append(case("那个后台任务结束之后再交回放行", "sweep", "交回", 0, "a2"))
        stdin_input = json.dumps({"tool_name": "SubagentHandback", "tool_input": {"message": "字" * (HANDBACK_CHARACTER_LIMIT + 1)}, "agent_type": "sweep"})
        entry = subprocess.run(["bash", os.path.join(hook_dir, "handback-guard.sh")], input=stdin_input, capture_output=True, text=True,
                               env={**os.environ, "CLAUDE_PROJECT_DIR": work}, timeout=60)
        cases.append(("走真实 stdin 入口：超长交回", 2, entry.returncode))
        failures = [item for item in cases if item[1] != item[2]]
        for label, want, got in failures:
            print(f"  ✗ 自检：{label} 应当返回 {want}，实际 {got}")  # gate-lint:detail
        if failures:
            print("    → 看 decide()、citation_problems()、green_count_problems() 的判法；HANDBACK_GUARD_BREAK 设着的话这里本来就该红")
            return 1
        print(f"  ✓ handback-guard 自检通过（查了 {len(cases)} 种）：子 agent 交回时自己起的后台任务还在跑拒（通用 agent 也判、主 agent 不判、说明里列出进程号与两条出路），"
              "跑完了或停掉之后放行；项目子 agent 超长交回拒、正好在上限放行、非项目 agent 与主 agent 不判；"
              "三方腿与核查员引文对不上或没点名报告拒、对上放行；书记员与执行员绿行没数或报 0 项拒（交回正文与点名的报告文件都查）；走真实 stdin 入口拒得出")
        return 0
    finally:
        if sleeper is not None and sleeper.poll() is None:
            sleeper.kill()
            sleeper.wait()
        shutil.rmtree(work, ignore_errors=True)


def main():
    hook_dir = sys.argv[1]
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    try:
        hook_input = json.load(sys.stdin)
    except ValueError as error:
        print(f"! handback-guard 读不懂 hook 的 JSON，这一次放行：{error}", file=sys.stderr)
        return 0
    project_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_dir))
    cite_check = os.path.join(os.path.dirname(os.path.dirname(hook_dir)), "research", "scripts", "cite-check.py")
    code, message = decide(hook_input, project_root, cite_check)
    if message:
        print(message, file=sys.stderr)
    return code

sys.exit(main())
PY
