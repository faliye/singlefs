#!/usr/bin/env python3
# admission: always 直接执行只跑自证，判的是此刻造出来的几份会话记录，上一次的结论不替这一次作保
# run-condition: none 被 import 的函数库；直接执行只在临时目录里造会话记录，除了 python3 没有环境要求
"""子 agent 会话记录的共用认法：被 import 的函数库（只用标准库，import 本身不读文件、不起子进程）。
「交回」与「起点」各一个定义：续做闸（.claude/hooks/continuation-guard.sh）、看门狗（scripts/agents/agent-watch.py）、
结局账与用量账（scripts/agents/agent-cost.py）都从这里取，各自不另写一份。

    sys.path.insert(0, <仓根>/scripts/agents); import subagent_transcript as transcripts

交回（送达）：assistant 调过 SubagentHandback（content 块 type 为 tool_use、name 为 SubagentHandback），后面有 tool_use_id 与它相同的 tool_result，
  而且不是错误：is_error 不为真，内容也不是 {"success": false, …}（「Nothing was sent」那种没送出）。被交回闸拒的那一次（is_error 为真）不算，
  调了还没有 tool_result 的不算。不按字符串搜：每份会话记录开头的工具清单与派发提示里都写着这个工具名。
  - handback_result_is_error(block)        一个 tool_result 块算不算没送出
  - HandbackTracker                        按会话记录的次序逐条喂：wants(line) 按原文快筛（带工具名的、带已知调用号的才要解析），
                                           feed(record) 认调用与结果，返回这一条里认出的送达（没有是 None）；delivered 是最近一次送达的那一次
                                           （call_id、message、called_at、delivered_at），一次都没送达是 None
  - delivered_handback(path)、handback_delivered(path)   整份读一份会话记录：最近一次送达的那一次 / 有没有送达过
起点：会话记录里第一条带 timestamp（ISO 8601，可带 Z）的记录的时刻；不按文件修改时刻。
  - record_moment(record)、first_record_moment(path)
  - started_between(path, since, until=None)   since ≤ 起点 < until（until 不给不设上界）；文件修改时刻早于 since 的先按不在窗口里筛掉
    （起点不会晚于最后一次写的时刻，这一步只省读文件，不改结论）
三处调用方的自证都拿同两份样本（sample_refused_then_delivered、sample_refused_only：先被交回闸拒、后来送达 / 被拒之后再没送达）判，
三处判得一样：前一份是交回，后一份不是。

  python3 scripts/agents/subagent_transcript.py --selftest

直接执行而不是 --selftest 时设了 SUBAGENT_TRANSCRIPT_BREAK：参数解析之前就拒、退 2，点名它只在自证里认（refuse_break_outside_selftest()）；
被 import 时照认（三处调用方的自证要用它）。

弄坏开关（只给自证证红用；被 import 时三处调用方的自证同样红）。逐项写开关名、弄坏哪一处、打开后自证哪几格红（失败行「✗ subagent_transcript 自证：<格名>：…」）：
  SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered  handback_result_is_error() 一律当送达：被交回闸拒的、内容是 {"success": false} 的 tool_result 也算交回
      「✗ subagent_transcript 自证：被拒之后再没送达不算交回」「✗ subagent_transcript 自证：success:false 不算交回」
      「✗ subagent_transcript 自证：被拒之后再没送达的样本」（另：续做闸、看门狗、结局账三处自证里用这份样本的那一格同样红）
  SUBAGENT_TRANSCRIPT_BREAK=start-by-mtime  first_record_moment() 交文件修改时刻，不读第一条记录
      「✗ subagent_transcript 自证：起点按首条记录」「✗ subagent_transcript 自证：窗口按起点筛」「✗ subagent_transcript 自证：一条带时间戳的记录都没有」
      （另：agent-cost.py 自证里交回与起点那一格同样红）
  SUBAGENT_TRANSCRIPT_BREAK=honor-break-outside-selftest  refuse_break_outside_selftest() 不拒：直接执行而不是 --selftest 时设了开关也照常往下走（改前的样子）
      「✗ subagent_transcript 自证：平常跑设开关要被拒」
"""
import datetime
import json
import os
import shutil
import subprocess
import sys
import tempfile

HANDBACK_TOOL = "SubagentHandback"
BREAK_VARIABLE = "SUBAGENT_TRANSCRIPT_BREAK"
KNOWN_BREAK_MODES = ("refused-counts-delivered", "start-by-mtime", "honor-break-outside-selftest")
RECORD_TIMEZONE = datetime.timezone.utc   # clock-times:allow 会话记录的时间戳是协调世界时，按它比较，不是给人看的钟点


def break_mode():
    """弄坏开关的值；认不出的值抛 ValueError（拼错的开关不能被当成「没开」）。"""
    mode = os.environ.get(BREAK_VARIABLE, "")
    if mode and mode not in KNOWN_BREAK_MODES:
        raise ValueError(f"{BREAK_VARIABLE}={mode} 认不出（只认 {'、'.join(KNOWN_BREAK_MODES)}）")
    return mode


def refuse_break_outside_selftest(arguments):
    """直接执行时、参数解析之前调：不是 --selftest 而设了弄坏开关就打拒绝、交 2；没设或是 --selftest 交 None。"""
    mode = os.environ.get(BREAK_VARIABLE, "")
    if not mode or arguments == ["--selftest"] or mode == "honor-break-outside-selftest":
        return None
    print(f"  ✗ {BREAK_VARIABLE}={mode} 只在自证里认（只给 --selftest 证红用），这一趟不是 --selftest，不往下走", file=sys.stderr)
    print(f"     → 怎么办：去掉 {BREAK_VARIABLE} 再跑；要证红就跑 {BREAK_VARIABLE}=<项> python3 scripts/agents/subagent_transcript.py --selftest", file=sys.stderr)
    return 2


def handback_result_is_error(block):
    """交回调用的 tool_result 算不算没送出：is_error 为真，或内容是 {"success": false, …}（例如「Nothing was sent」）。"""
    if break_mode() == "refused-counts-delivered":
        return False
    if block.get("is_error") is True:
        return True
    content = block.get("content")
    if isinstance(content, list):
        content = "".join(part.get("text") or "" for part in content if isinstance(part, dict))
    try:
        parsed = json.loads(content) if isinstance(content, str) else None
    except ValueError:
        parsed = None
    return isinstance(parsed, dict) and parsed.get("success") is False


def record_moment(record):
    """一条记录的 timestamp（带时区的 datetime）；没有或认不出是 None。"""
    text = record.get("timestamp") if isinstance(record, dict) else None
    if not isinstance(text, str) or not text:
        return None
    try:
        moment = datetime.datetime.fromisoformat(text.replace("Z", "+00:00"))
    except ValueError:
        return None
    return moment if moment.tzinfo else moment.replace(tzinfo=RECORD_TIMEZONE)


def handback_message_of(tool_input):
    """交回调用的正文：message 是字符串就交它，否则把整个输入写成 JSON。"""
    message = tool_input.get("message") if isinstance(tool_input, dict) else None
    return message if isinstance(message, str) else json.dumps(tool_input, ensure_ascii=False)


class HandbackTracker:
    """按会话记录的次序逐条喂，认送达的交回（判法在文件头）。"""

    def __init__(self):
        self.calls = {}         # 交回调用的 id → (正文, 调用的时刻)
        self.delivered = None   # 最近一次送达：{"call_id", "message", "called_at", "delivered_at"}

    def wants(self, line):
        """这一行原文要不要解析：带工具名的（可能是调用），或带着已知调用号的（可能是它的 tool_result）。"""
        return HANDBACK_TOOL in line or any(call_id in line for call_id in self.calls)

    def feed(self, record):
        """喂一条已解析的记录；这一条里认出送达就交那一次（同时记进 delivered），没有交 None。"""
        message = record.get("message") if isinstance(record, dict) else None
        if not isinstance(message, dict) or not isinstance(message.get("content"), list):
            return None
        moment = record_moment(record)
        found = None
        for block in message["content"]:
            if not isinstance(block, dict):
                continue
            if (message.get("role") == "assistant" and block.get("type") == "tool_use"
                    and block.get("name") == HANDBACK_TOOL and block.get("id")):
                self.calls[block["id"]] = (handback_message_of(block.get("input") or {}), moment)
            elif (message.get("role") == "user" and block.get("type") == "tool_result"
                    and block.get("tool_use_id") in self.calls and not handback_result_is_error(block)):
                text, called_at = self.calls[block["tool_use_id"]]
                found = {"call_id": block["tool_use_id"], "message": text, "called_at": called_at, "delivered_at": moment or called_at}
                self.delivered = found
        return found


def delivered_handback(path):
    """整份读一份子 agent 会话记录：最近一次送达的交回（HandbackTracker.delivered 的样子），没送达过是 None。只解析快筛留下的行。"""
    tracker = HandbackTracker()
    with open(path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if not tracker.wants(line):
                continue
            try:
                record = json.loads(line)
            except ValueError:
                continue
            tracker.feed(record)
    return tracker.delivered


def handback_delivered(path):
    """子 agent 的会话记录里有没有一次送达的交回。"""
    return delivered_handback(path) is not None


def modified_moment(path):
    return datetime.datetime.fromtimestamp(os.path.getmtime(path), tz=RECORD_TIMEZONE)


def first_record_moment(path):
    """起点：第一条带 timestamp 的记录的时刻；一条都没有是 None。"""
    if break_mode() == "start-by-mtime":
        return modified_moment(path)
    with open(path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if '"timestamp"' not in line:
                continue
            try:
                moment = record_moment(json.loads(line))
            except ValueError:
                continue
            if moment is not None:
                return moment
    return None


def started_between(path, since, until=None):
    """起点落在 [since, until) 里（until 为 None 不设上界）；起点认不出的不算在里面。"""
    if modified_moment(path) < since and break_mode() != "start-by-mtime":
        return False   # 起点不晚于最后一次写的时刻：修改时刻早于 since 的起点一定更早，不读文件
    start = first_record_moment(path)
    return start is not None and start >= since and (until is None or start < until)


# ── 三处调用方自证共用的两份样本：先被交回闸拒、后来送达 / 被拒之后再没送达 ──
SAMPLE_BASE = datetime.datetime(2026, 10, 5, 1, 0, tzinfo=RECORD_TIMEZONE)


def sample_stamp(base, minutes):
    return (base + datetime.timedelta(minutes=minutes)).isoformat()


def sample_records(handbacks, base=SAMPLE_BASE, prompt="派发提示：干完用 SubagentHandback 交回"):
    """一份子 agent 会话记录：开头的工具清单与派发提示（都带着工具名）、一次 Bash，再按 handbacks 逐次交回。
    handbacks：[(调用号, 正文, 结果)]，结果是 "refused"（交回闸拒，is_error）、"not-sent"（success:false）、"delivered"（success:true）、
    "plain"（纯文字、不是错误）或 None（还没结果）。"""
    usage = {"input_tokens": 10, "output_tokens": 1, "cache_read_input_tokens": 0, "cache_creation_input_tokens": 0}
    records = [
        {"type": "user", "timestamp": sample_stamp(base, 0), "message": {"role": "user", "content": prompt}},
        {"type": "attachment", "timestamp": sample_stamp(base, 0), "attachment": {"type": "prompt_snapshot", "tools": [{"name": HANDBACK_TOOL, "description": "Deliver your final report"}]}},
        {"type": "assistant", "timestamp": sample_stamp(base, 1), "message": {"id": "sample-bash", "role": "assistant", "content": [
            {"type": "tool_use", "id": "toolu_sample_bash", "name": "Bash", "input": {"command": "ls"}}], "usage": usage}},
        {"type": "user", "timestamp": sample_stamp(base, 2), "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "toolu_sample_bash", "content": "a\nb"}]}},
    ]
    minute = 3
    for call_id, text, result in handbacks:
        records.append({"type": "assistant", "timestamp": sample_stamp(base, minute), "message": {"id": f"sample-{call_id}", "role": "assistant", "content": [
            {"type": "tool_use", "id": call_id, "name": HANDBACK_TOOL, "input": {"message": text}}], "usage": usage}})
        block = {"type": "tool_result", "tool_use_id": call_id}
        if result == "refused":
            block.update(is_error=True, content="PreToolUse:SubagentHandback hook error: [bash .claude/hooks/handback-guard.sh]: ✗ 交回正文超过 4000 字。")
        elif result == "not-sent":
            block.update(content=[{"type": "text", "text": json.dumps({"success": False, "message": "Nothing was sent"})}])
        elif result == "delivered":
            block.update(content=[{"type": "text", "text": json.dumps({"success": True, "message": "Report delivered to your caller."})}])
        elif result == "plain":
            block.update(content="Report delivered.")
        if result is not None:
            records.append({"type": "user", "timestamp": sample_stamp(base, minute + 1), "message": {"role": "user", "content": [block]}})
        minute += 2
    return records


def sample_refused_then_delivered(base=SAMPLE_BASE):
    """先被交回闸拒（正文太长）、改短后送达：算交回，交回正文是送达的那一次。"""
    return sample_records([("toolu_sample_refused", "长的报告", "refused"), ("toolu_sample_delivered", "短的报告", "delivered")], base)


def sample_refused_only(base=SAMPLE_BASE):
    """被交回闸拒之后再没送达（之后被停或撞限额）：不算交回。"""
    return sample_records([("toolu_sample_refused_only", "长的报告", "refused")], base)


def write_records(path, records):
    with open(path, "w", encoding="utf-8") as handle:
        handle.write("\n".join(json.dumps(record, ensure_ascii=False) for record in records) + "\n")


def run_selftest():
    try:
        break_mode()
    except ValueError as error:
        print(f"  ✗ subagent_transcript 自证：{error}")
        print("    → 怎么办：SUBAGENT_TRANSCRIPT_BREAK 只写文件头那张表里的值，或不设")
        return 1
    work = tempfile.mkdtemp(prefix="subagent-transcript-selftest-")
    failures, checked = [], 0
    try:
        def transcript(name, records):
            path = os.path.join(work, f"{name}.jsonl")
            write_records(path, records)
            return path
        cells = [
            ("先被拒、后来送达算交回", sample_refused_then_delivered(), "短的报告"),
            ("被拒之后再没送达不算交回", sample_refused_only(), None),
            ("success:false 不算交回", sample_records([("toolu_not_sent", "报告", "not-sent")]), None),
            ("调了还没有结果不算交回", sample_records([("toolu_pending", "报告", None)]), None),
            ("只在工具清单与派发提示里带着名字不算交回", sample_records([]), None),
            ("结果是纯文字、不是错误也算交回（续做闸原来的认法）", sample_records([("toolu_plain", "纯文字结果", "plain")]), "纯文字结果"),
            ("送达两次取最近一次", sample_records([("toolu_first", "第一次", "delivered"), ("toolu_second", "第二次", "delivered")]), "第二次"),
        ]
        for index, (label, records, want) in enumerate(cells):
            path = transcript(f"cell-{index}", records)
            checked += 1
            delivered = delivered_handback(path)
            got = delivered["message"] if delivered else None
            if got != want or handback_delivered(path) != (want is not None):
                failures.append(f"{label}：应当 {want!r}，实际 {got!r}")
        # 被拒之后再没送达的样本：逐条喂时 feed 一次都不该交出送达（看门狗按 feed 的返回值记交回）
        checked += 1
        tracker = HandbackTracker()
        fed = [tracker.feed(record) for record in sample_refused_only()]
        if any(fed) or tracker.delivered is not None:
            failures.append(f"被拒之后再没送达的样本：feed 逐条应当一次都不交出送达，实际 {[item for item in fed if item]}")
        # 快筛：结果那一行不带工具名，只带调用号，也要留下
        checked += 1
        tracker = HandbackTracker()
        result_line = json.dumps(sample_refused_then_delivered()[-1], ensure_ascii=False)
        call_line = json.dumps(sample_refused_then_delivered()[-2], ensure_ascii=False)
        wanted_before = tracker.wants(result_line)
        tracker.feed(json.loads(call_line))
        if wanted_before or not tracker.wants(call_line) or not tracker.wants(result_line):
            failures.append("快筛：调用那一行要留；结果那一行在认出调用之前不留、认出之后要留")
        # 起点：第一条记录不带 timestamp、第二条带；文件修改时刻改到窗口之后，起点照首条记录
        path = transcript("start", [{"type": "summary", "summary": "没有时间戳"}] + sample_refused_only())
        late = (SAMPLE_BASE + datetime.timedelta(days=4)).timestamp()
        os.utime(path, (late, late))
        checked += 1
        start = first_record_moment(path)
        if start != SAMPLE_BASE:
            failures.append(f"起点按首条记录：应当 {SAMPLE_BASE.date()} 首条记录那一刻，实际 {start}")
        checked += 1
        day = lambda offset: SAMPLE_BASE.replace(hour=0, minute=0) + datetime.timedelta(days=offset)
        window = [(day(-4), day(1), True), (day(-4), day(0), False), (day(1), None, False), (SAMPLE_BASE, None, True)]
        wrong = [(str(since.date()), str(until.date()) if until else None, want) for since, until, want in window if started_between(path, since, until) != want]
        if wrong:
            failures.append(f"窗口按起点筛：(since, until, 应当) 判错的 {wrong}")
        checked += 1
        empty = transcript("no-timestamp", [{"type": "summary", "summary": "没有时间戳"}])
        if first_record_moment(empty) is not None or started_between(empty, day(-3650)):
            failures.append("一条带时间戳的记录都没有：起点应当是 None、不算在窗口里")
        # 平常跑设开关要被拒：起自己一次（不是 --selftest），设上开关要退 2、点名它只在自证里认；不设就不拒（照常报认不出的用法）
        checked += 1
        own_mode = os.environ.get(BREAK_VARIABLE, "")
        probe_mode = own_mode if own_mode == "honor-break-outside-selftest" else "refused-counts-delivered"
        probes = {}
        for label, value in (("设上", probe_mode), ("不设", None)):
            environment = {name: text for name, text in os.environ.items() if name != BREAK_VARIABLE}
            if value is not None:
                environment[BREAK_VARIABLE] = value
            done = subprocess.run([sys.executable, "-B", os.path.realpath(__file__), "--break-guard-probe-unknown-option"], env=environment,
                                  stdin=subprocess.DEVNULL, capture_output=True, text=True, errors="replace", timeout=60)
            named = [row for row in (done.stdout + done.stderr).splitlines() if BREAK_VARIABLE in row and "自证" in row]
            probes[label] = (done.returncode, named)
        if probes["设上"][0] != 2 or not probes["设上"][1] or probes["不设"][1]:
            failures.append(f"平常跑设开关要被拒：设上 {BREAK_VARIABLE}={probe_mode} 应当退 2 并点名它只在自证里认、不设不点名，"
                            f"实际设上退 {probes['设上'][0]}、点名 {len(probes['设上'][1])} 行，不设点名 {len(probes['不设'][1])} 行")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ subagent_transcript 自证：{failure}")   # gate-lint:detail
    if failures:
        print("    → 怎么办：看 handback_result_is_error()、HandbackTracker.feed()、first_record_moment() 与 refuse_break_outside_selftest()；SUBAGENT_TRANSCRIPT_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ subagent_transcript 自证通过（查了 {checked} 项）：送达按调用之后对应的不是错误的 tool_result 认（被拒、没送出、还没结果、只在工具清单里带着名字的都不算，"
          "先被拒后送达的算、取最近一次），快筛不漏结果那一行；起点按首条带时间戳的记录、不按修改时刻，窗口按 [since, until) 筛；直接执行不是 --selftest 而设了开关的被拒、退 2")
    return 0


def main(arguments):
    refused = refuse_break_outside_selftest(arguments)
    if refused is not None:
        return refused
    if arguments == ["--selftest"]:
        return run_selftest()
    print(f"  ✗ 认不出的用法：{' '.join(arguments)}", file=sys.stderr)
    print("     → 怎么办：import 它再调 handback_delivered()、HandbackTracker、first_record_moment()、started_between()；直接执行只认 --selftest", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.dont_write_bytecode = True
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..", ".claude", "scripts"))
    from project_preflight import preflight
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
