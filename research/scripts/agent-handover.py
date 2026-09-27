#!/usr/bin/env python3
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本（与它点名的会话记录），除了 python3 之外没有环境要求
"""从一个子 agent 的会话记录里机械地抽交接摘要，交给接手它的新 agent。

为什么：一个子 agent 撞了限额、被停或报错，就该换一个新的接着做（用户 2026-09-19 定；上下文大小本身不是换的理由，见
.claude/rules/three-way-inference.md「撞了限额、被停或报错的腿」那一段）；而老的那个做到哪、写了哪些文件、试过什么、最后在想什么，都只在它的上下文里。它被停掉、撞限额、来不及交回时，
没有任何东西把这些交出来，新的一个只能从头摸（2026-09-19 E155 第二次跑的执行员上下文到 95 万被停，第二段没交回报告；
用户当场指出「终止的 agent 的进度和生成的内容要拿来复用，不可能完全重开」）。这个脚本不靠老 agent 配合，读会话记录就抽得出来。

只抽它做过的动作与说过的话，不判哪一件做成了：新 agent 开工先核现场（编不编得过、产物是不是这一轮的），再接着做。

用法：
    agent-handover.py --agent <agent id> --out <交接摘要.md>   # 在 ~/.claude/projects/*/*/subagents/ 里按 id 找会话记录
    agent-handover.py --transcript <会话记录.jsonl> --out <交接摘要.md>
    agent-handover.py --selftest                              # 造一份假会话记录走一遍，临时目录用完即删（判红、抛异常也删）；AGENT_HANDOVER_BREAK=<项> 时必须判红

退出码：0 写成；2 参数或找不到会话记录；1 自检不过。
"""
import argparse
import glob
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

BROKEN = os.environ.get("AGENT_HANDOVER_BREAK", "")
SELFTEST_DIRECTORY_PREFIX = "agent-handover-selftest-"
# 自检拉起自己的子进程时给的角色：red 走判红那条路，crash 在写完文件之后抛异常（scratch_cleanup_problems 用）
SELFTEST_ROLE = os.environ.get("AGENT_HANDOVER_SELFTEST_ROLE", "")
SELFTEST_RED_LABEL = "子进程按 AGENT_HANDOVER_SELFTEST_ROLE=red 判红"
SELFTEST_CRASH_MARKER = "agent-handover 自检按 AGENT_HANDOVER_SELFTEST_ROLE=crash 故意抛的异常"
WRITE_TOOLS = ("Write", "Edit", "NotebookEdit")
WRITING_COMMAND = re.compile(r"(^|[\s;|&])(>>?|tee|cp|mv|rsync|sed\s+-i|cargo\s+(test|run|build)|bash\s+research/scripts/replay\.sh|python3\s+research/scripts/[\w./-]+\.py)")
LAST_TEXT_BLOCKS = 12
LAST_WRITING_COMMANDS = 25
OUTPUT_TAIL_LINES = 6


def find_transcript(agent_id):
    candidates = glob.glob(os.path.expanduser(f"~/.claude/projects/*/*/subagents/agent-{agent_id}.jsonl"))
    return max(candidates, key=os.path.getmtime) if candidates else None


def text_of(content):
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        return "\n".join(block.get("text", "") if isinstance(block, dict) else str(block) for block in content)
    return json.dumps(content, ensure_ascii=False)


def tail_lines(text, count):
    lines = (text or "").rstrip("\n").split("\n")
    return "\n".join(lines[-count:])


def file_state(path):
    if not path or not os.path.isfile(path):
        return "现在不在"
    with open(path, "rb") as handle:
        return f"现在在，sha256 {hashlib.sha256(handle.read()).hexdigest()[:16]}…"


def read_transcript(path):
    dispatch, continuations, texts, handbacks = None, [], [], []
    written = {}          # 路径 → [次数, 最后时间, 最后一次的工具]
    read_counts = {}
    commands = []         # [时间, 命令, 输出]
    by_tool_id = {}
    background_started, background_finished = {}, set()
    model_calls, last_context = 0, 0
    seen_message_ids = set()
    for line in open(path, encoding="utf-8", errors="replace"):
        try:
            record = json.loads(line)
        except ValueError:
            continue
        timestamp = record.get("timestamp", "")
        message = record.get("message")
        if not isinstance(message, dict):
            continue
        role, content = message.get("role"), message.get("content")
        if role == "user":
            origin = (record.get("origin") or {}).get("kind")
            if isinstance(content, str) or (isinstance(content, list) and all(block.get("type") == "text" for block in content if isinstance(block, dict))):
                body = text_of(content)
                if body.startswith("<system-reminder>"):
                    continue
                if dispatch is None and origin is None:
                    dispatch = (timestamp, body)
                elif origin == "coordinator" and BROKEN != "continuations":
                    continuations.append((timestamp, body))
                elif origin == "task-notification":
                    background_finished.update(re.findall(r"<task-id>([^<]+)</task-id>", body))
            if isinstance(content, list):
                for block in content:
                    if not isinstance(block, dict) or block.get("type") != "tool_result":
                        continue
                    result = text_of(block.get("content"))
                    entry = by_tool_id.get(block.get("tool_use_id"))
                    if entry is not None:
                        entry[2] = result
                    started = re.search(r"running in background with ID: (\S+?)\.", result)
                    if started:
                        output = re.search(r"Output is being written to: (\S+?\.output)", result)
                        background_started[started.group(1)] = output.group(1) if output else ""
        elif role == "assistant":
            usage = message.get("usage") or {}
            if usage and message.get("id") not in seen_message_ids:
                seen_message_ids.add(message.get("id"))
                model_calls += 1
                last_context = (usage.get("cache_read_input_tokens") or 0) + (usage.get("cache_creation_input_tokens") or 0) + (usage.get("input_tokens") or 0)
            if not isinstance(content, list):
                continue
            for block in content:
                if block.get("type") == "text" and block.get("text", "").strip():
                    texts.append((timestamp, block["text"].strip()))
                elif block.get("type") == "tool_use":
                    name, tool_input = block.get("name"), block.get("input") or {}
                    if name in WRITE_TOOLS and BROKEN != "written":
                        target = tool_input.get("file_path") or tool_input.get("notebook_path") or ""
                        entry = written.setdefault(target, [0, "", ""])
                        entry[0] += 1
                        entry[1] = timestamp
                        entry[2] = name
                    elif name == "Read":
                        target = tool_input.get("file_path") or ""
                        read_counts[target] = read_counts.get(target, 0) + 1
                    elif name == "Bash":
                        command = tool_input.get("command") or ""
                        entry = [timestamp, command, None]
                        by_tool_id[block.get("id")] = entry
                        if WRITING_COMMAND.search(command):
                            commands.append(entry)
                    elif name == "SubagentHandback":
                        handbacks.append((timestamp, text_of(tool_input.get("message") or tool_input)))
    return {
        "dispatch": dispatch, "continuations": continuations, "texts": texts, "handbacks": handbacks,
        "written": written, "read_counts": read_counts, "commands": commands,
        "background_started": background_started, "background_finished": background_finished,
        "model_calls": model_calls, "last_context": last_context,
    }


def fence(text):
    longest = max((len(run) for run in re.findall(r"`+", text or "")), default=0)
    marker = "`" * max(3, longest + 1)
    return f"{marker}\n{text}\n{marker}"


def render(path, meta, facts):
    lines = [f"# 交接摘要：子 agent {os.path.basename(path)[len('agent-'):-len('.jsonl')]}（{meta.get('agentType', '?')}「{meta.get('description', '?')}」）", ""]
    lines.append(f"会话记录 `{path}`；模型调用 {facts['model_calls']} 次，最后一次上下文 {facts['last_context'] / 10000:.1f} 万。")
    lines.append("这份摘要是机械抽的：只列它做过的动作与说过的话，不判哪一件做成了。接手的 agent 开工先核现场（编不编得过、测试绿不绿、产物是不是这一轮的），再接着做。")
    lines.append("")
    lines.append("## 一、派发提示（原样）")
    lines.append("")
    lines.append(fence(facts["dispatch"][1]) if facts["dispatch"] else "（会话记录里没找到）")
    lines.append("")
    lines.append(f"## 二、续做消息（原样，{len(facts['continuations'])} 条）")
    for timestamp, body in facts["continuations"]:
        lines += ["", f"{timestamp}：", fence(body)]
    lines += ["", f"## 三、写过、改过的文件（Write / Edit，{len(facts['written'])} 个）", "", "| 文件 | 次数 | 最后一次 | 现在 |", "|---|---|---|---|"]
    for target, (count, timestamp, tool) in sorted(facts["written"].items(), key=lambda item: item[1][1]):
        lines.append(f"| `{target}` | {count}（{tool}） | {timestamp} | {file_state(target)} |")
    lines += ["", f"## 四、会写东西或跑东西的 Bash 命令（最后 {LAST_WRITING_COMMANDS} 条，共 {len(facts['commands'])} 条；输出只留末 {OUTPUT_TAIL_LINES} 行）"]
    for timestamp, command, output in facts["commands"][-LAST_WRITING_COMMANDS:]:
        lines += ["", f"{timestamp}：", fence(command[:600]), "输出末尾：" if output is not None else "输出：（没有返回，多半是停在这一条上）"]
        if output is not None:
            lines.append(fence(tail_lines(output, OUTPUT_TAIL_LINES)))
    pending = [task for task in facts["background_started"] if task not in facts["background_finished"]]
    lines += ["", f"## 五、后台任务（起过 {len(facts['background_started'])} 个，没收到完成通知的 {len(pending)} 个）", ""]
    for task in pending:
        lines.append(f"- `{task}`：输出文件 `{facts['background_started'][task] or '?'}`")
    lines += ["", "## 六、读得最多的文件（接手时别整份再读一遍，按段读）", "", "| 文件 | 读了几次 |", "|---|---|"]
    for target, count in sorted(facts["read_counts"].items(), key=lambda item: -item[1])[:12]:
        lines.append(f"| `{target}` | {count} |")
    lines += ["", f"## 七、它最后说的话（最后 {LAST_TEXT_BLOCKS} 段文字，原样）"]
    for timestamp, text in facts["texts"][-LAST_TEXT_BLOCKS:]:
        lines += ["", f"{timestamp}：", fence(text)]
    lines += ["", f"## 八、交回报告（{len(facts['handbacks'])} 份，原样）"]
    for timestamp, text in facts["handbacks"]:
        lines += ["", f"{timestamp}：", fence(text)]
    return "\n".join(lines) + "\n"


def write_handover(transcript, out_path):
    meta_path = transcript[:-len(".jsonl")] + ".meta.json"
    meta = json.load(open(meta_path)) if os.path.isfile(meta_path) else {}
    facts = read_transcript(transcript)
    with open(out_path, "w", encoding="utf-8") as handle:
        handle.write(render(transcript, meta, facts))
    return facts


def selftest():
    """造一份假会话记录走一遍。临时目录用完即删，判红、抛异常也删：门禁每轮给阶段一个私有 TMPDIR，跑完里面还剩东西就判红。
    弄坏开关 AGENT_HANDOVER_BREAK=keepscratch 走回 mkdtemp 不删的旧毛病，「临时目录」那三格必须判红。"""
    if BROKEN == "keepscratch":
        work = tempfile.mkdtemp(prefix=SELFTEST_DIRECTORY_PREFIX)
        checks = extraction_checks_in(work)
    else:
        with tempfile.TemporaryDirectory(prefix=SELFTEST_DIRECTORY_PREFIX) as work:
            checks = extraction_checks_in(work)
    problems = [f"交接摘要里没有「{label}」" for label, ok in checks if not ok]
    # 子进程只走它那一条路，不再往下拉子进程
    cleanup_problems = [] if SELFTEST_ROLE else scratch_cleanup_problems(work)
    for problem in problems + cleanup_problems:
        print(f"  ✗ 自检：{problem}")  # gate-lint:detail
    if problems or cleanup_problems:
        print("    → 看 read_transcript() 与 render()；临时目录那几格看 selftest() 里的 TemporaryDirectory；AGENT_HANDOVER_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ agent-handover 自检通过：派发提示、续做消息、写过的文件与现状、跑过的命令与输出、最后说的话、上下文都抽得出；"
          f"自检的临时目录在通过、判红、抛异常三条路上都删掉了（查了 {len(checks) + SCRATCH_CLEANUP_CELLS} 项）")
    return 0


SCRATCH_CLEANUP_CELLS = 3   # scratch_cleanup_problems 判的格数：通过的这一次、判红的子进程、抛异常的子进程


def scratch_cleanup_problems(own_directory):
    """自检的临时目录用完即删，三格：通过的这一次（own_directory 已经不在）、判红那条路、抛异常那条路。
    后两条路拉起自己的子进程去走，TMPDIR 指进一个探查目录；子进程退出之后那里必须是空的。"""
    problems = []
    if os.path.exists(own_directory):
        problems.append(f"通过的这一次，临时目录用完没删：{own_directory}")
    with tempfile.TemporaryDirectory(prefix="agent-handover-cleanup-probe-") as probe:
        for role, expected_text in (("red", SELFTEST_RED_LABEL), ("crash", SELFTEST_CRASH_MARKER)):
            child_temporary_directory = os.path.join(probe, role)
            os.mkdir(child_temporary_directory)
            child = subprocess.run([sys.executable, os.path.abspath(__file__), "--selftest"],
                                   env=dict(os.environ, AGENT_HANDOVER_SELFTEST_ROLE=role, TMPDIR=child_temporary_directory),
                                   capture_output=True, text=True)
            child_output = child.stdout + child.stderr
            left_behind = sorted(os.listdir(child_temporary_directory))
            if child.returncode == 0 or expected_text not in child_output:
                problems.append(f"{role} 那条路没走到（子进程退出码 {child.returncode}，输出里没有「{expected_text}」）：{child_output[-400:]}")
            elif left_behind:
                problems.append(f"{role} 那条路上临时目录没删：{child_temporary_directory} 里留下 {'、'.join(left_behind)}")
    return problems


def extraction_checks_in(work):
    """在 work 里造假会话记录、抽交接摘要，回 [(这一格叫什么, 过没过)]。"""
    transcript = os.path.join(work, "agent-feedbeef12345678.jsonl")
    written_file = os.path.join(work, "device.rs")
    open(written_file, "w").write("fn main() {}\n")
    records = [
        {"timestamp": "T1", "message": {"role": "user", "content": "重跑登记：e999-r2-prereg.md\n这一段回答的岔路：第 7 行"}},
        {"timestamp": "T2", "message": {"role": "assistant", "id": "m1", "usage": {"input_tokens": 5, "cache_read_input_tokens": 700000},
                                        "content": [{"type": "tool_use", "id": "t1", "name": "Edit", "input": {"file_path": written_file}}]}},
        {"timestamp": "T3", "message": {"role": "assistant", "id": "m2", "usage": {"input_tokens": 5, "cache_read_input_tokens": 700100},
                                        "content": [{"type": "tool_use", "id": "t2", "name": "Bash", "input": {"command": "cargo test --release > log.txt 2>&1"}}]}},
        {"timestamp": "T4", "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "t2", "content": "test result: ok. 47 passed"}]}},
        {"timestamp": "T5", "origin": {"kind": "coordinator"}, "message": {"role": "user", "content": "续做：接着做第二段"}},
        {"timestamp": "T6", "message": {"role": "assistant", "id": "m3", "usage": {"input_tokens": 5, "cache_read_input_tokens": 950000},
                                        "content": [{"type": "text", "text": "接着改 replay.sh，指到 stage2 产物"}]}},
    ]
    with open(transcript, "w") as handle:
        handle.write("\n".join(json.dumps(record, ensure_ascii=False) for record in records) + "\n")
    out = os.path.join(work, "handover.md")
    facts = write_handover(transcript, out)
    text = open(out).read()
    checks = [
        ("派发提示原样", "这一段回答的岔路：第 7 行" in text),
        ("续做消息", "续做：接着做第二段" in text and len(facts["continuations"]) == 1),
        ("写过的文件与现在的 sha256", written_file in text and "现在在，sha256" in text),
        ("跑过的命令与输出末尾", "cargo test --release" in text and "47 passed" in text),
        ("最后说的话", "接着改 replay.sh" in text),
        ("最后一次上下文", "95.0 万" in text),
    ]
    if SELFTEST_ROLE == "crash":
        raise RuntimeError(SELFTEST_CRASH_MARKER)
    if SELFTEST_ROLE == "red":
        checks.append((SELFTEST_RED_LABEL, False))
    return checks


def main():
    if len(sys.argv) >= 2 and sys.argv[1] == "--selftest":
        return selftest()
    parser = argparse.ArgumentParser(description="从子 agent 的会话记录里抽交接摘要")
    parser.add_argument("--agent", help="子 agent id")
    parser.add_argument("--transcript", help="会话记录路径（给了就不按 id 找）")
    parser.add_argument("--out", required=True, help="交接摘要写到哪")
    arguments = parser.parse_args()
    transcript = arguments.transcript or (find_transcript(arguments.agent) if arguments.agent else None)
    if not transcript or not os.path.isfile(transcript):
        print(f"  ✗ 找不到会话记录（--agent {arguments.agent} / --transcript {arguments.transcript}）")
        print("    → 怎么办：给 --transcript 指到 ~/.claude/projects/<项目>/<会话>/subagents/agent-<id>.jsonl，或核对 agent id")
        return 2
    facts = write_handover(transcript, arguments.out)
    print(f"  ✓ 交接摘要写到 {arguments.out}：续做消息 {len(facts['continuations'])} 条、写过的文件 {len(facts['written'])} 个、"
          f"会写东西的命令 {len(facts['commands'])} 条、没收到完成通知的后台任务 "
          f"{len([task for task in facts['background_started'] if task not in facts['background_finished']])} 个")
    return 0


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main())
