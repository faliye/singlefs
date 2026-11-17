#!/usr/bin/env python3
# admission: always 读的是此刻盘上的会话登记与会话记录，上一次查到的不替这一次作保
# run-condition: none 只读 ~/.claude/sessions 与 ~/.claude/projects 下的文件，不写任何东西，除了 python3 没有环境要求；登记目录只在 Claude Code 跑着的那一台有，在主 agent 那一台直接起，不经统一任务服务派到别的机器
"""查谁改了一个文件、谁点到过它：按会话名逐行报每个会话对这个路径做了几次 Edit / Write、几条 Bash 命令里点到它、几处只在文本里提到它。

用法：
  who-touched.py <路径或符号>… [--since-minutes N] [--symbol | --path] [--root 仓根] [--jobs N]
                  [--sessions-directory 目录] [--project-directory 目录]
  who-touched.py --selftest

怎么认：
  参数里带 / 的当路径，不带的当符号（函数名、编号、一段字面串）；--path、--symbol 对全部参数强制成一种。
  路径：仓内的绝对路径先换成相对仓根的写法（--root 或环境变量 WHO_TOUCHED_REPOSITORY，没给取这份脚本所在的仓）；前后不连着路径字符才算点到
  （找 scripts/x.py 时 myscripts/x.py、scripts/x.pyc、scripts/x.py.bak 都不算）。符号：前后不连着字母数字下划线才算。
  会话记录是 <项目目录>/<会话号>.jsonl，子 agent 的记录是 <项目目录>/<会话号>/subagents/*.jsonl；子 agent 的记录一律归到它的父会话，不另起一行。
  项目目录取法：--project-directory，其次环境变量 WHO_TOUCHED_PROJECT_DIRECTORY，其次 lib_session_directory.py 照仓根算出的会话目录。
  会话名取法：登记目录里 sessionId 相同的每一份登记的名字，全列（同一个会话号续开过会有几个名字，用 " / " 连起来），按启动先后排；
  登记目录取 --sessions-directory，其次环境变量 SESSION_REGISTRY_DIRECTORY，其次 ~/.claude/sessions；登记里没有的会话号写「（登记里没有）」。
  --since-minutes N：只看修改时刻在最近 N 分钟内的记录文件（主记录与子 agent 记录各按自己的修改时刻筛）。
  --jobs N：同时读几份记录文件，不给取 min(8, 每台默认重活线程数)。

三栏（路径参数）：
  Edit/Write   工具调用（Edit、Write、MultiEdit、NotebookEdit）的 file_path 与路径逐字相同的次数；括号里「其中子 agent」是子 agent 记录里的那几次，
               「出错」是后面跟着 is_error 的 tool_result 的那几次（被钩子拒了、没改成）
  Bash         Bash 工具的 command 里点到路径的命令条数
  只提到       文本块（用户与助手的文字）和其余工具调用的输入（派发提示、SendMessage、别的文件的 Edit 内容……）里点到路径的块数；
               点到不算改：只有这一栏有数的会话没改过它
  最近一次改   最近一次 Edit/Write 的日期（取记录时间戳的日期部分）
符号参数只有「只提到」一栏：文本块与所有工具调用的输入（含 Edit 的新旧串与 Bash 命令）里点到的块数。
同一个文本块或同一次工具调用点到几处，只记一次。tool_result（读出来的文件内容、命令输出）和 thinking 不数。

输出：先一行读了几份记录，再每个参数一节，每个会话一行「名字 | 会话号 … | 三栏」，改得多的在前；没有会话点到它就写一句没有。

退出码：0 至少一个参数查到了会话；1 一个都没查到；2 用法错（没给参数、--since-minutes 不是正数），或平常跑时设了弄坏开关；3 项目目录读不了。

弄坏开关（只给 --selftest 证红用；平常跑设了 WHO_TOUCHED_BREAK 参数解析之前就退 2、点名它只在自证里认）。逐项写开关名、弄坏哪一处、打开后自证哪几格红（失败行「✗ 自检：<格名>：…」，括号里是格名）：
  WHO_TOUCHED_BREAK=skip-subagents  session_files() 不收 <会话号>/subagents/ 下的记录
      「子 agent 的 Edit 归到父会话」「出错的 Edit 标出」「时间窗只看窗内改过的记录文件」「不带时间窗时旧记录都在」
  WHO_TOUCHED_BREAK=mention-counted-as-edit  scan_block() 把文本块里的提到也记进 Edit/Write 栏
      「Edit 与 Write 都计入」「只提到不算改」
  WHO_TOUCHED_BREAK=first-name-only  names_by_session() 同一个会话号只留第一个名字
      「会话号两份名字都列」
  WHO_TOUCHED_BREAK=ignore-window  session_files() 不按修改时刻筛
      「时间窗只看窗内改过的记录文件」
"""
import argparse
import concurrent.futures
import contextlib
import hashlib
import io
import itertools
import json
import os
import re
import sys
import tempfile
import time
from dataclasses import dataclass

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, SCRIPT_DIRECTORY)
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "scripts"))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, "scripts", "hosts"))
import heavy_threads  # noqa: E402  每台默认重活线程数只在这一份里取（登记位 .claude/heavy-thread-default）
import lib_session_directory as session_directories  # noqa: E402  默认会话目录的唯一取法
import lib_session_registry as session_registry  # noqa: E402  会话登记的唯一取法
from project_preflight import preflight  # noqa: E402

BREAK_VARIABLE = "WHO_TOUCHED_BREAK"
KNOWN_BREAK_ITEMS = ("skip-subagents", "mention-counted-as-edit", "first-name-only", "ignore-window")
PROJECT_DIRECTORY_VARIABLE = "WHO_TOUCHED_PROJECT_DIRECTORY"
REPOSITORY_VARIABLE = "WHO_TOUCHED_REPOSITORY"
EDIT_TOOLS = ("Edit", "Write", "MultiEdit", "NotebookEdit")
RECORD_TYPES = ("assistant", "user")
UNREGISTERED_NAME = "（登记里没有）"
DEFAULT_JOB_LIMIT = 8
FOUND_EXIT = 0
NOTHING_FOUND_EXIT = 1
USAGE_EXIT = 2
UNREADABLE_EXIT = 3
# 路径字符只认 ASCII：中文紧挨着路径写（「改了scripts/x.py的」）照样算点到
PATH_BEFORE = r"(?<![A-Za-z0-9_.-])"
PATH_AFTER = r"(?![A-Za-z0-9_-]|\.[A-Za-z0-9_])"
SYMBOL_BEFORE = r"(?<![A-Za-z0-9_])"
SYMBOL_AFTER = r"(?![A-Za-z0-9_])"


def break_items(environment=None):
    environment = os.environ if environment is None else environment
    return [item for item in environment.get(BREAK_VARIABLE, "").split(",") if item]


def broken(item):
    return item in break_items()


def break_switch_refusal(arguments, environment):
    """弄坏开关只在自证里认：设了而命令行不是单独一个 --selftest，交回拒绝说明（两行）；否则交 None。"""
    if not environment.get(BREAK_VARIABLE) or arguments == ["--selftest"]:
        return None
    return (f"  ✗ {BREAK_VARIABLE} 是弄坏开关，只在自证（单独一个 --selftest）里认；平常跑设了就拒，这一次什么都没做",
            f"     → 怎么办：unset {BREAK_VARIABLE} 再来；要看开关红在哪一格，跑 {BREAK_VARIABLE}=<项> python3 scripts/agents/who-touched.py --selftest")


@dataclass(frozen=True)
class Target:
    text: str          # 找的串（路径是相对仓根的写法）
    is_path: bool
    exact_paths: tuple  # Edit / Write 的 file_path 逐字比的写法
    pattern: re.Pattern
    needles: tuple     # 整行快筛用的字节串：原文与 JSON 转义后的写法


def needles_for(text):
    escaped = json.dumps(text, ensure_ascii=True)[1:-1]
    return tuple(dict.fromkeys((text.encode("utf-8"), escaped.encode("utf-8"))))


def make_target(argument, root, kind=None):
    is_path = kind == "path" or (kind is None and "/" in argument)
    if not is_path:
        return Target(argument, False, (), re.compile(SYMBOL_BEFORE + re.escape(argument) + SYMBOL_AFTER), needles_for(argument))
    text = argument[2:] if argument.startswith("./") else argument
    prefix = root.rstrip("/") + "/"
    if text.startswith(prefix):
        text = text[len(prefix):]
    exact_paths = (text,) if text.startswith("/") else (os.path.join(root, text), text)
    return Target(text, True, exact_paths, re.compile(PATH_BEFORE + re.escape(text) + PATH_AFTER), needles_for(text))


def names_by_session(sessions_directory):
    """→ ({会话号: [名字…]}, 登记读得了吗)。同一个会话号的每一份登记的名字全列，按启动先后。"""
    entries = session_registry.session_registry_entries(sessions_directory)
    if not entries:
        return {}, False
    names = {}
    for entry in sorted(entries, key=lambda item: (item.started_seconds is None, item.started_seconds or 0)):
        listed = names.setdefault(entry.session_id, [])
        if entry.name and entry.name not in listed:
            listed.append(entry.name)
    if broken("first-name-only"):   # 弄坏开关：只留第一个名字
        names = {session_id: listed[:1] for session_id, listed in names.items()}
    return names, True


def session_files(project_directory, cutoff):
    """→ {会话号: [(记录文件路径, 是不是子 agent 记录)…]}。cutoff 不是 None 时只收修改时刻不早于它的文件。"""
    files = {}

    def add(session_id, path, is_subagent):
        try:
            if cutoff is not None and not broken("ignore-window") and os.path.getmtime(path) < cutoff:   # 弄坏开关 ignore-window：不按修改时刻筛
                return
        except OSError:
            return
        files.setdefault(session_id, []).append((path, is_subagent))

    for name in sorted(os.listdir(project_directory)):
        full = os.path.join(project_directory, name)
        if name.endswith(".jsonl") and os.path.isfile(full):
            add(name[:-len(".jsonl")], full, False)
        elif os.path.isdir(full) and not broken("skip-subagents"):   # 弄坏开关 skip-subagents：不收子 agent 的记录
            subagents = os.path.join(full, "subagents")
            if os.path.isdir(subagents):
                for sub_name in sorted(os.listdir(subagents)):
                    if sub_name.endswith(".jsonl"):
                        add(name, os.path.join(subagents, sub_name), True)
    return files


def empty_counts():
    return {"edits": 0, "failed": 0, "bash": 0, "mentions": 0, "last": ""}


def string_leaves(value):
    if isinstance(value, str):
        yield value
    elif isinstance(value, dict):
        for item in value.values():
            yield from string_leaves(item)
    elif isinstance(value, list):
        for item in value:
            yield from string_leaves(item)


def scan_block(block, moment, targets, counts, pending):
    kind = block.get("type")
    if kind == "tool_result":
        call = block.get("tool_use_id")
        key = call.encode("utf-8") if isinstance(call, str) else None
        if key in pending:
            for index in pending.pop(key):
                if block.get("is_error") is True:
                    counts[index]["failed"] += 1
        return
    if kind == "text":
        text = block.get("text")
        for index, target in enumerate(targets):
            if isinstance(text, str) and target.pattern.search(text):
                counts[index]["mentions"] += 1
                if target.is_path and broken("mention-counted-as-edit"):   # 弄坏开关：文本里的提到也记成改
                    counts[index]["edits"] += 1
        return
    if kind != "tool_use":
        return
    name = block.get("name")
    tool_input = block.get("input") if isinstance(block.get("input"), dict) else {}
    call = block.get("id")
    leaves = list(string_leaves(tool_input))
    for index, target in enumerate(targets):
        counted = False
        if target.is_path and name in EDIT_TOOLS:
            if tool_input.get("file_path") in target.exact_paths or tool_input.get("notebook_path") in target.exact_paths:
                counts[index]["edits"] += 1
                counts[index]["last"] = max(counts[index]["last"], moment)
                if isinstance(call, str):
                    pending.setdefault(call.encode("utf-8"), []).append(index)
                counted = True
        elif target.is_path and name == "Bash":
            command = tool_input.get("command")
            if isinstance(command, str) and target.pattern.search(command):
                counts[index]["bash"] += 1
                counted = True
        if not counted and any(target.pattern.search(leaf) for leaf in leaves):
            counts[index]["mentions"] += 1


def scan_file(path, targets):
    """读一份记录文件，→ 每个目标一份计数。整行先按字节串快筛，筛过的才解析。"""
    counts = [empty_counts() for _ in targets]
    needles = [needle for target in targets for needle in target.needles]
    pending = {}   # 还没等到结果的 Edit / Write 调用号 → 目标下标
    with open(path, "rb") as handle:
        for raw in handle:
            if not (any(needle in raw for needle in needles) or any(call in raw for call in pending)):
                continue
            try:
                record = json.loads(raw)
            except ValueError:
                continue
            if not isinstance(record, dict) or record.get("type") not in RECORD_TYPES or not isinstance(record.get("message"), dict):
                continue
            content = record["message"].get("content")
            blocks = [{"type": "text", "text": content}] if isinstance(content, str) else content
            if not isinstance(blocks, list):
                continue
            moment = record.get("timestamp") if isinstance(record.get("timestamp"), str) else ""
            for block in blocks:
                if isinstance(block, dict):
                    scan_block(block, moment, targets, counts, pending)
    return counts


def collect(project_directory, targets, since_minutes, jobs):
    """→ {"sessions": {会话号: {"targets": [每个目标的合计…]}}, "file_count": 读了几份, "subagent_file_count": 其中子 agent 的几份}。"""
    cutoff = None if since_minutes is None else time.time() - since_minutes * 60
    listed = [(session_id, path, is_subagent) for session_id, items in sorted(session_files(project_directory, cutoff).items())
              for path, is_subagent in items]
    paths = [path for _, path, _ in listed]
    if jobs > 1 and len(paths) > 1:
        with concurrent.futures.ProcessPoolExecutor(max_workers=min(jobs, len(paths))) as pool:
            scanned = list(pool.map(scan_file, paths, itertools.repeat(targets)))
    else:
        scanned = [scan_file(path, targets) for path in paths]
    sessions = {}
    for (session_id, _, is_subagent), per_target in zip(listed, scanned):
        totals = sessions.setdefault(session_id, {"targets": [dict(empty_counts(), subagent_edits=0) for _ in targets]})["targets"]
        for total, counts in zip(totals, per_target):
            for field in ("edits", "failed", "bash", "mentions"):
                total[field] += counts[field]
            if is_subagent:
                total["subagent_edits"] += counts["edits"]
            total["last"] = max(total["last"], counts["last"])
    return {"sessions": sessions, "file_count": len(listed), "subagent_file_count": sum(1 for _, _, is_subagent in listed if is_subagent)}


def row_text(label, session_id, target, total):
    if not target.is_path:
        return f"  {label} | 会话号 {session_id} | 只提到 {total['mentions']}"
    extras = []
    if total["subagent_edits"]:
        extras.append(f"子 agent {total['subagent_edits']}")
    if total["failed"]:
        extras.append(f"出错 {total['failed']}")
    detail = "（其中" + "，".join(extras) + "）" if extras else ""
    parts = [f"Edit/Write {total['edits']}{detail}", f"Bash {total['bash']}", f"只提到 {total['mentions']}"]
    if total["edits"] and total["last"]:
        parts.append(f"最近一次改 {total['last'][:10]}")
    return f"  {label} | 会话号 {session_id} | " + " | ".join(parts)


def render(result, names, targets):
    """→ ([输出行…], 每个目标有没有会话点到它的列表)。"""
    lines, found = [], []
    for index, target in enumerate(targets):
        lines.append(f"== {target.text}（{'路径' if target.is_path else '符号'}）==")
        rows = []
        for session_id, entry in result["sessions"].items():
            total = entry["targets"][index]
            if total["edits"] or total["bash"] or total["mentions"]:
                rows.append((session_id, total))
        rows.sort(key=lambda row: (-row[1]["edits"], -row[1]["bash"], -row[1]["mentions"], row[0]))
        for session_id, total in rows:
            lines.append(row_text(" / ".join(names.get(session_id) or []) or UNREGISTERED_NAME, session_id, target, total))
        if not rows:
            lines.append("  没有会话改过、点到或提到它" if target.is_path else "  没有会话提到它")
        found.append(bool(rows))
    return lines, found


def main(argv, environment=None):
    environment = os.environ if environment is None else environment
    parser = argparse.ArgumentParser(prog="who-touched.py", add_help=True)
    parser.add_argument("targets", nargs="*")
    parser.add_argument("--since-minutes", type=float)
    parser.add_argument("--sessions-directory")
    parser.add_argument("--project-directory")
    parser.add_argument("--root")
    parser.add_argument("--jobs", type=int)
    kinds = parser.add_mutually_exclusive_group()
    kinds.add_argument("--symbol", dest="kind", action="store_const", const="symbol")
    kinds.add_argument("--path", dest="kind", action="store_const", const="path")
    parser.add_argument("--selftest", action="store_true")
    arguments = parser.parse_args(argv)
    if arguments.selftest:
        return selftest()
    if not arguments.targets:
        print("  ✗ 用法：who-touched.py <路径或符号>… [--since-minutes N]")
        print("     → 怎么办：至少给一个路径（带 /）或符号；用法全文看文件头")
        return USAGE_EXIT
    if arguments.since_minutes is not None and arguments.since_minutes <= 0:
        print("  ✗ --since-minutes 要是正数")
        print("     → 怎么办：写几分钟，例如 --since-minutes 120；要看全部就不带这个参数")
        return USAGE_EXIT
    root = arguments.root or environment.get(REPOSITORY_VARIABLE) or REPOSITORY_ROOT
    try:
        project_directory = arguments.project_directory or environment.get(PROJECT_DIRECTORY_VARIABLE) \
            or session_directories.session_directory(REPOSITORY_ROOT, repository=arguments.root or environment.get(REPOSITORY_VARIABLE), environment=environment)
    except session_directories.SessionDirectoryError as error:
        print(f"  ✗ 取不到会话目录：{error}")
        print("     → 怎么办：用 --project-directory 直接给会话记录所在的目录")
        return UNREADABLE_EXIT
    if not os.path.isdir(project_directory):
        print(f"  ✗ 会话记录目录不存在：{project_directory}")
        print("     → 怎么办：核对仓根（--root）或直接用 --project-directory 给目录")
        return UNREADABLE_EXIT
    sessions_directory = arguments.sessions_directory or session_registry.registry_directory(environment)
    targets = [make_target(argument, root, arguments.kind) for argument in arguments.targets]
    if arguments.jobs:
        jobs = arguments.jobs
    else:
        try:
            jobs = min(DEFAULT_JOB_LIMIT, heavy_threads.default_heavy_threads(environment))
        except heavy_threads.HeavyThreadDefaultError as error:
            print(f"  ✗ 取不到每台默认重活线程数：{error}")
            print("     → 怎么办：用 --jobs N 直接给同时读几份记录文件，或照上面那一句修登记位 .claude/heavy-thread-default")
            return UNREADABLE_EXIT
    result = collect(project_directory, targets, arguments.since_minutes, jobs)
    names, registry_readable = names_by_session(sessions_directory)
    window = "全部" if arguments.since_minutes is None else f"修改时刻在最近 {arguments.since_minutes:g} 分钟内的"
    print(f"读了 {result['file_count']} 份会话记录（其中子 agent 记录 {result['subagent_file_count']} 份），范围：{window}")
    if not registry_readable:
        print(f"  ! 会话登记目录读不到或里面没有能解的登记：{sessions_directory}；名字都写成{UNREGISTERED_NAME}")
    lines, found = render(result, names, targets)
    print("\n".join(lines))
    return FOUND_EXIT if any(found) else NOTHING_FOUND_EXIT


def stamp(epoch):
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(epoch))  # clock-times:allow 造会话记录里时间戳字段的自检输入


def day_of(epoch):
    return time.strftime("%Y-%m-%d", time.gmtime(epoch))


def record_use(call, name, epoch, **tool_input):
    return {"type": "assistant", "timestamp": stamp(epoch),
            "message": {"role": "assistant", "content": [{"type": "tool_use", "id": call, "name": name, "input": tool_input}]}}


def record_text(text, epoch):
    return {"type": "assistant", "timestamp": stamp(epoch), "message": {"role": "assistant", "content": [{"type": "text", "text": text}]}}


def record_result(call, is_error, epoch):
    return {"type": "user", "timestamp": stamp(epoch),
            "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": call, "is_error": is_error, "content": "x"}]}}


def write_records(path, records, trailing_garbage=""):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        for record in records:
            handle.write(json.dumps(record, ensure_ascii=False) + "\n")
        handle.write(trailing_garbage)


def write_registration(directory, pid, session_id, name, started_milliseconds):
    os.makedirs(directory, exist_ok=True)
    with open(os.path.join(directory, f"{pid}.json"), "w", encoding="utf-8") as handle:
        json.dump({"pid": pid, "sessionId": session_id, "name": name, "startedAt": started_milliseconds}, handle)


def tree_fingerprint(*directories):
    """每份文件的 (路径, 大小, 修改时刻, sha256)，加上每层目录的清单：自证前后比，证明只读。"""
    fingerprint = []
    for directory in directories:
        for here, subdirectories, names in os.walk(directory):
            subdirectories.sort()
            fingerprint.append((here, tuple(subdirectories), tuple(sorted(names))))
            for name in sorted(names):
                path = os.path.join(here, name)
                with open(path, "rb") as handle:
                    fingerprint.append((path, os.path.getsize(path), os.stat(path).st_mtime_ns, hashlib.sha256(handle.read()).hexdigest()))
    return fingerprint


def build_fixture(work):
    """造假的会话登记与会话记录，→ (登记目录, 项目目录, 仓根)。目标路径是 scripts/x.py，符号是 frobnicate。"""
    registry, project, root = os.path.join(work, "sessions"), os.path.join(work, "project"), "/fake/repo"
    base, day = 1700000000, 86400
    target = f"{root}/scripts/x.py"
    # 登记：session-alpha 有两份（两个名字），session-charlie 没有登记
    write_registration(registry, 11, "session-alpha", "name-alpha-old", 1000)
    write_registration(registry, 12, "session-alpha", "name-alpha-new", 2000)
    write_registration(registry, 13, "session-bravo", "name-bravo", 3000)
    write_registration(registry, 14, "session-delta", "name-delta", 4000)
    write_registration(registry, 15, "session-echo", "name-echo", 5000)
    # session-alpha 主记录：Edit 一次、Write 一次、Bash 点到一次、字面相近的 Bash 一条、文本提到一次、改别的文件时写了符号
    write_records(os.path.join(project, "session-alpha.jsonl"), [
        record_use("call-edit-1", "Edit", base, file_path=target, old_string="1", new_string="2"),
        record_result("call-edit-1", False, base),
        record_use("call-write-1", "Write", base + day, file_path=target, content="y"),
        record_use("call-bash-1", "Bash", base + day, command="python3 scripts/x.py --selftest"),
        record_use("call-bash-2", "Bash", base + day, command="cat myscripts/x.py scripts/x.pyc scripts/x.py.bak"),
        record_text("我改了 scripts/x.py", base + day),
        record_text("frobnicates 不是那个符号", base + day),
        record_use("call-edit-other", "Edit", base + day, file_path=f"{root}/scripts/y.py", old_string="p", new_string="frobnicate()"),
    ])
    # session-alpha 的子 agent 记录：Edit 两次，前一次出错；日期比主记录晚
    write_records(os.path.join(project, "session-alpha", "subagents", "agent-one.jsonl"), [
        record_use("call-sub-1", "Edit", base + 2 * day, file_path=target, old_string="2", new_string="3"),
        record_result("call-sub-1", True, base + 2 * day),
        record_use("call-sub-2", "Edit", base + 3 * day, file_path=target, old_string="2", new_string="4"),
        record_result("call-sub-2", False, base + 3 * day),
    ])
    # session-bravo：只在文本里提到路径与符号（紧挨着中文），改的是别的文件，结尾一行是坏行
    write_records(os.path.join(project, "session-bravo.jsonl"), [
        record_text("看了scripts/x.py的结果，frobnicate 也要查", base),
        record_use("call-edit-z", "Edit", base, file_path=f"{root}/scripts/z.py", old_string="p", new_string="q"),
    ], trailing_garbage="{坏行 scripts/x.py\n")
    # session-charlie：Bash 里用绝对路径点到
    write_records(os.path.join(project, "session-charlie.jsonl"), [
        record_use("call-bash-3", "Bash", base, command=f"sed -n 1,5p {target}"),
    ])
    # session-delta：主记录很早（三小时前）、子 agent 记录刚写
    write_records(os.path.join(project, "session-delta.jsonl"), [record_use("call-old-1", "Edit", base, file_path=target, old_string="a", new_string="b")])
    write_records(os.path.join(project, "session-delta", "subagents", "agent-two.jsonl"), [record_use("call-new-1", "Edit", base + day, file_path=target, old_string="b", new_string="c")])
    # session-echo：只有三小时前的主记录
    write_records(os.path.join(project, "session-echo.jsonl"), [record_use("call-old-2", "Write", base, file_path=target, content="z")])
    old = time.time() - 3 * 3600
    for old_file in ("session-delta.jsonl", "session-echo.jsonl"):
        os.utime(os.path.join(project, old_file), (old, old))
    return registry, project, root


def selftest():
    unknown = [item for item in break_items() if item not in KNOWN_BREAK_ITEMS]
    if unknown:
        print(f"  ✗ 自检：{BREAK_VARIABLE} 里的 {'、'.join(unknown)} 不认识，只认 {'、'.join(KNOWN_BREAK_ITEMS)}")
        print("     → 怎么办：弄坏开关只写文件头开关表里列的项")
        return 1
    failures = []

    def expect(label, condition, detail=""):
        if not condition:
            failures.append(label)
            print(f"  ✗ 自检：{label}：{detail}" if detail else f"  ✗ 自检：{label}")  # gate-lint:detail

    with tempfile.TemporaryDirectory(prefix="who-touched-selftest-") as work:
        registry, project, root = build_fixture(work)
        fake_environment = {PROJECT_DIRECTORY_VARIABLE: project, session_registry.REGISTRY_DIRECTORY_ENV: registry, REPOSITORY_VARIABLE: root}
        before = tree_fingerprint(registry, project)
        names, readable = names_by_session(registry)
        path_target = make_target("scripts/x.py", root)
        symbol_target = make_target("frobnicate", root)
        everything = collect(project, [path_target], None, 1)
        path_lines, path_found = render(everything, names, [path_target])

        def row_of(lines, session_id):
            return next((line for line in lines if f"会话号 {session_id} |" in line), "")

        def totals_of(result, session_id):
            return result["sessions"].get(session_id, {"targets": [dict(empty_counts(), subagent_edits=0)]})["targets"][0]

        alpha, bravo, charlie = (totals_of(everything, name) for name in ("session-alpha", "session-bravo", "session-charlie"))
        alpha_row = row_of(path_lines, "session-alpha")
        expect("登记读得了", readable)
        # 主记录里 Edit 一次加 Write 一次，减去子 agent 的两次
        expect("Edit 与 Write 都计入", alpha["edits"] - alpha["subagent_edits"] == 2, f"主记录里本该 2 次，实际 {alpha['edits'] - alpha['subagent_edits']}（合计 {alpha['edits']}）")
        expect("子 agent 的 Edit 归到父会话",
               alpha["subagent_edits"] == 2 and "子 agent 2" in alpha_row and "最近一次改 " + day_of(1700000000 + 3 * 86400) in alpha_row,
               f"session-alpha 一行：{alpha_row!r}")
        expect("出错的 Edit 标出", alpha["failed"] == 1 and "出错 1" in alpha_row, f"session-alpha 一行：{alpha_row!r}")
        expect("子 agent 记录不另起一行", sum(1 for line in path_lines if "会话号 " in line) == 5, f"本该 5 行（alpha、bravo、charlie、delta、echo）：{path_lines!r}")
        expect("只提到不算改", bravo["edits"] == 0 and bravo["bash"] == 0 and bravo["mentions"] == 1
               and "Edit/Write 0 | Bash 0 | 只提到 1" in row_of(path_lines, "session-bravo"), f"session-bravo：{bravo!r}")
        expect("会话号两份名字都列", "name-alpha-old" in alpha_row and "name-alpha-new" in alpha_row, f"session-alpha 一行：{alpha_row!r}")
        expect("Bash 栏只数命令里点到的", charlie["bash"] == 1 and charlie["edits"] == 0 and "Bash 1" in row_of(path_lines, "session-charlie"), f"session-charlie：{charlie!r}")
        expect("字面相近的路径不计", alpha["bash"] == 1 and alpha["mentions"] == 1, f"session-alpha 的 Bash 与提到本该各 1 次：{alpha!r}")
        expect("登记里没有的会话号照列", UNREGISTERED_NAME in row_of(path_lines, "session-charlie"), f"session-charlie 一行：{row_of(path_lines, 'session-charlie')!r}")
        listed_sessions = [line.split("会话号 ")[1].split(" |")[0] for line in path_lines if "会话号 " in line]
        expect("改得多的排在前面", listed_sessions[:1] == ["session-alpha"], f"{path_lines!r}")

        symbol_result = collect(project, [symbol_target], None, 1)
        symbol_lines, symbol_found = render(symbol_result, names, [symbol_target])
        symbol_rows = [line for line in symbol_lines if "会话号 " in line]
        expect("符号只报提到次数", len(symbol_rows) == 2 and all("只提到 1" in line and "Edit/Write" not in line and "Bash" not in line for line in symbol_rows)
               and "会话号 session-alpha |" in "".join(symbol_rows) and "会话号 session-bravo |" in "".join(symbol_rows), f"{symbol_lines!r}")

        windowed = collect(project, [path_target], 60, 1)
        delta = totals_of(windowed, "session-delta")
        expect("时间窗只看窗内改过的记录文件", "session-echo" not in windowed["sessions"] and "session-delta" in windowed["sessions"]
               and delta["edits"] == 1 and delta["subagent_edits"] == 1, f"窗内本该没有 session-echo、session-delta 只剩子 agent 的 1 次：{windowed['sessions']!r}")
        expect("不带时间窗时旧记录都在", totals_of(everything, "session-delta")["edits"] == 2 and "session-echo" in everything["sessions"],
               f"session-delta 本该 2 次：{totals_of(everything, 'session-delta')!r}")

        expect("多进程与单进程结果相同", collect(project, [path_target], None, 2)["sessions"] == everything["sessions"], "jobs=2 与 jobs=1 的合计不同")
        absolute = make_target(f"{root}/scripts/x.py", root)
        expect("仓内绝对路径换成相对仓根再找", absolute.text == "scripts/x.py" and collect(project, [absolute], None, 1)["sessions"] == everything["sessions"],
               f"换出来的是 {absolute.text!r}")

        captured = io.StringIO()
        with contextlib.redirect_stdout(captured):
            code = main(["scripts/x.py", "--jobs", "1"], fake_environment)
        expect("目录经环境变量换成假的", code == FOUND_EXIT and "name-bravo" in captured.getvalue() and "会话号 session-alpha |" in captured.getvalue(), f"退出码 {code}：{captured.getvalue()!r}")
        captured = io.StringIO()
        with contextlib.redirect_stdout(captured):
            code = main(["no/such/path.py", "--jobs", "1"], fake_environment)
        expect("一个会话都没点到时退 1", code == NOTHING_FOUND_EXIT and "没有会话改过、点到或提到它" in captured.getvalue(), f"退出码 {code}：{captured.getvalue()!r}")
        with contextlib.redirect_stdout(io.StringIO()):
            code = main([], fake_environment)
        expect("没给参数退 2", code == USAGE_EXIT, f"退出码 {code}")

        expect("平常跑设弄坏开关要被拒", break_switch_refusal(["scripts/x.py"], {BREAK_VARIABLE: "skip-subagents"}) is not None
               and break_switch_refusal(["--selftest"], {BREAK_VARIABLE: "skip-subagents"}) is None
               and break_switch_refusal(["scripts/x.py"], {}) is None)
        expect("全程只读不写", tree_fingerprint(registry, project) == before, "自证前后两个假目录的文件清单、大小、修改时刻或内容变了")
    if failures:
        print(f"  自检没过 {len(failures)} 格（逐格的失败行在上面）：{'、'.join(failures)}")
        print("     → 怎么办：按各行点名的格查 scan_block()、session_files()、names_by_session() 与 render()；"
              f"{BREAK_VARIABLE} 设着的话（项与红的格见文件头）这里本来就该红")
        return 1
    print("  ✓ 自检：Edit 与 Write 都计入、子 agent 的 Edit 归到父会话（出错的标出、不另起一行）、只提到不算改、会话号两份名字都列、"
          "Bash 栏只数命令里点到的、字面相近的路径不计、登记里没有的会话号照列、符号只报提到次数、时间窗只看窗内改过的记录文件、"
          "多进程与单进程结果相同、仓内绝对路径换成相对仓根再找、目录经环境变量换成假的、没点到退 1、平常跑设弄坏开关被拒、全程只读不写")
    return 0


if __name__ == "__main__":
    refusal = break_switch_refusal(sys.argv[1:], os.environ)
    if refusal is not None:
        print(refusal[0], file=sys.stderr)
        print(refusal[1], file=sys.stderr)
        sys.exit(USAGE_EXIT)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
