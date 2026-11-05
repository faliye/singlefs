#!/usr/bin/env python3
# admission: always 读的是此刻盘上的会话登记与此刻 /proc 里的进程，上一次读到的不替这一次作保
# run-condition: none 被 import 的函数库，只用标准库，只读本机的会话登记目录与 /proc；直接执行当场查一次或跑自证
"""读 Claude Code 的会话登记（~/.claude/sessions/<pid>.json）的唯一取法：被 import 的函数库，也可直接执行当场查一次。

    sys.path.insert(0, <仓根>/scripts/agents); import lib_session_registry as session_registry

每个 Claude Code 进程在登记目录里写一份 <pid>.json，带 sessionId（会话号，统一任务服务记的「会话」就是它）、name（ListAgents 列的会话名，
或用户起的标题）、pid、procStart（/proc/<pid>/stat 第 22 列）、startedAt（毫秒）、messagingSocketPath。同一个会话号续开过会留几份，各是一个 pid。
活着 = 登记里的 pid 还在、而且 /proc/<pid>/stat 第 22 列与登记的 procStart 一致（pid 被别的进程占了不算）；登记没写 procStart 的只看 pid 在不在。
名字会随会话续开而变（同一个会话号先后叫过几个名字），死掉的登记 Claude Code 不删：每次都当场读、当场判活，查到的名字不写进仓、不存。

  - registry_directory(environment=None)          登记目录：环境变量 SESSION_REGISTRY_DIRECTORY 设了取它，没设取 ~/.claude/sessions
  - session_registry_entries(directory)           [SessionEntry…]；目录读不了、或一份能解的登记都没有，交 None（判不了，不当成全都不在）
  - process_start_ticks(pid)                       /proc/<pid>/stat 第 22 列（字符串）；进程不在或读不出是 None
  - named_entry_for_session(entries, session_id)   这个会话号的登记里挑一份：活着的优先，几份都活着取启动最晚的；一份都没有交 None
  - entries_for_name(entries, name)               名字或标题逐字相同的登记，活着的在前，同是活着或同是不在的启动晚的在前

命令行（当场读登记目录，不存结果）：
  python3 scripts/agents/lib_session_registry.py by-session <会话号>   打这个会话号此刻的名字
  python3 scripts/agents/lib_session_registry.py by-name <名字或标题>   打这个名字此刻对应的会话号
  python3 scripts/agents/lib_session_registry.py --selftest
  输出一行一份：SESSION session=<会话号> name=<名字> alive=<yes|no> pid=<pid> socket=<路径或 none>；by-session 只打挑中的那一份，by-name 打全部、活着的在前。
  任务号要换成会话号：task-server/task status <号> 第一行的「会话 <会话号>」。

调它的：scripts/agents/agent-watch.py（live_sessions）。
登记目录只在 Claude Code 跑着的那一台有；读它的是会话工具，在主 agent 那一台直接起，不经统一任务服务派到别的机器。

弄坏开关 SESSION_REGISTRY_BREAK=<项>（只给 --selftest 证红用；直接执行而不是单独一个 --selftest 时设了它，参数解析之前就拒、退 2、点名它只在自证里认；被 import 时照认，调用方的自证要用它）：
  SESSION_REGISTRY_BREAK=alive-without-start-check  判活着只看 pid 在不在，不比 procStart
      「✗ 自检：pid 被别的进程占了的登记应当判不在」
  SESSION_REGISTRY_BREAK=first-entry-for-session  named_entry_for_session 取第一份，不挑活着的
      「✗ 自检：同一个会话号有一份不在、一份活着时应当取活着的那一份」
  SESSION_REGISTRY_BREAK=name-ignores-liveness  entries_for_name 不把活着的排在前面
      「✗ 自检：按名字查时活着的登记应当排在前面」

退出码（直接执行）：0 查到了（自证：全过）；1 登记里没有（自证：有格判错）；
  2 用法错（子命令或参数不对、开关里有不认识的项，或不是单独一个 --selftest 时设了开关）；3 判不了（登记目录读不了，或里面一份能解的登记都没有）。
"""
import json
import os
import sys
import tempfile
from dataclasses import dataclass

sys.dont_write_bytecode = True

REGISTRY_DIRECTORY_ENV = "SESSION_REGISTRY_DIRECTORY"
BREAK_VARIABLE = "SESSION_REGISTRY_BREAK"
KNOWN_BREAK_ITEMS = ("alive-without-start-check", "first-entry-for-session", "name-ignores-liveness")
PROC_STAT_START_FIELD_AFTER_COMM = 19   # /proc/<pid>/stat 里右括号之后的字段从第 3 列起数，第 22 列是下标 19
NOT_FOUND_EXIT = 1
USAGE_EXIT = 2
UNJUDGEABLE_EXIT = 3


@dataclass(frozen=True)
class SessionEntry:
    session_id: str
    name: str | None
    pid: int
    is_alive: bool
    started_seconds: float | None
    messaging_socket_path: str | None


def break_items():
    return [item for item in os.environ.get(BREAK_VARIABLE, "").split(",") if item]


def registry_directory(environment=None):
    environment = os.environ if environment is None else environment
    return environment.get(REGISTRY_DIRECTORY_ENV) or os.path.expanduser("~/.claude/sessions")


def process_start_ticks(pid):
    """命令名里可以有空格与括号，从最后一个右括号往后切。"""
    try:
        with open(f"/proc/{pid}/stat", encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    except OSError:
        return None
    fields = text[text.rfind(")") + 2:].split()
    return fields[PROC_STAT_START_FIELD_AFTER_COMM] if len(fields) > PROC_STAT_START_FIELD_AFTER_COMM else None


def entry_is_alive(pid, registered_start, start_ticks_of=process_start_ticks):
    started = start_ticks_of(pid)
    if started is None:
        return False
    if "alive-without-start-check" in break_items():   # 弄坏开关：只看 pid 在不在
        return True
    return registered_start is None or str(registered_start) == started


def session_registry_entries(directory, start_ticks_of=process_start_ticks):
    try:
        names = sorted(name for name in os.listdir(directory) if name.endswith(".json"))
    except OSError:
        return None
    entries = []
    for name in names:
        try:
            with open(os.path.join(directory, name), encoding="utf-8") as handle:
                raw = json.load(handle)
            pid, session_id = int(raw["pid"]), str(raw["sessionId"])
        except (OSError, ValueError, KeyError, TypeError):
            continue
        started_at = raw.get("startedAt")
        entries.append(SessionEntry(
            session_id=session_id,
            name=raw.get("name") if isinstance(raw.get("name"), str) else None,
            pid=pid,
            is_alive=entry_is_alive(pid, raw.get("procStart"), start_ticks_of),
            started_seconds=started_at / 1000 if isinstance(started_at, (int, float)) else None,
            messaging_socket_path=raw.get("messagingSocketPath") if isinstance(raw.get("messagingSocketPath"), str) else None,
        ))
    return entries or None


def started_or_earliest(entry):
    return entry.started_seconds if entry.started_seconds is not None else float("-inf")


def named_entry_for_session(entries, session_id):
    candidates = [entry for entry in entries if entry.session_id == session_id]
    if not candidates:
        return None
    if "first-entry-for-session" in break_items():   # 弄坏开关：不挑活着的
        return candidates[0]
    alive = [entry for entry in candidates if entry.is_alive]
    return max(alive or candidates, key=started_or_earliest)


def entries_for_name(entries, name):
    matched = [entry for entry in entries if entry.name == name]
    if "name-ignores-liveness" in break_items():   # 弄坏开关：不把活着的排在前面
        return matched
    return sorted(matched, key=lambda entry: (not entry.is_alive, -started_or_earliest(entry)))


def entry_line(entry):
    return (f"SESSION session={entry.session_id} name={entry.name if entry.name is not None else 'none'} "
            f"alive={'yes' if entry.is_alive else 'no'} pid={entry.pid} socket={entry.messaging_socket_path or 'none'}")


def selftest():
    failures = []

    def expect(label, condition):
        if not condition:
            failures.append(label)
            print(f"  ✗ 自检：{label}")  # gate-lint:detail

    with tempfile.TemporaryDirectory(prefix="session-registry-selftest-") as directory:
        def write(file_name, content):
            with open(os.path.join(directory, file_name), "w", encoding="utf-8") as handle:
                handle.write(content if isinstance(content, str) else json.dumps(content))

        # 假的 /proc：pid 100 活着、启动时刻 "555"；pid 200 活着、启动时刻 "999"（登记写的是 "111"，pid 被别的进程占了）；pid 300 不在
        fake_ticks = {100: "555", 200: "999"}.get
        write("100.json", {"pid": 100, "sessionId": "session-alpha", "name": "name-of-live-alpha", "procStart": "555", "startedAt": 2000,
                           "messagingSocketPath": "/run/alpha.sock"})
        write("200.json", {"pid": 200, "sessionId": "session-beta", "name": "name-of-beta", "procStart": "111", "startedAt": 1000})
        # 不在的那一份文件名排在活着的那一份前面：取第一份（不挑活着的）就取错
        write("050.json", {"pid": 300, "sessionId": "session-alpha", "name": "name-of-dead-alpha", "procStart": "1", "startedAt": 500})
        write("broken.json", "{not json")
        entries = session_registry_entries(directory, fake_ticks)
        by_pid = {entry.pid: entry for entry in entries or []}
        expect("坏掉的登记跳过、三份能解的都读到", sorted(by_pid) == [100, 200, 300] and len(entries or []) == 3)
        expect("pid 与 procStart 都对得上的登记应当判活着", by_pid.get(100) is not None and by_pid[100].is_alive)
        expect("pid 被别的进程占了的登记应当判不在", by_pid.get(200) is not None and not by_pid[200].is_alive)
        expect("pid 不在的登记应当判不在", by_pid.get(300) is not None and not by_pid[300].is_alive)
        chosen = named_entry_for_session(entries or [], "session-alpha")
        expect("同一个会话号有一份不在、一份活着时应当取活着的那一份", chosen is not None and chosen.name == "name-of-live-alpha")
        expect("登记里没有的会话号交 None", named_entry_for_session(entries or [], "session-gamma") is None)
        expect("读不了的登记目录交 None", session_registry_entries(os.path.join(directory, "no-such-directory"), fake_ticks) is None)
        expect("环境变量指定登记目录", registry_directory({REGISTRY_DIRECTORY_ENV: directory}) == directory)
        line = entry_line(by_pid[100]) if 100 in by_pid else ""
        expect("输出行带会话号、名字、活着没有、pid 与 socket",
               line == "SESSION session=session-alpha name=name-of-live-alpha alive=yes pid=100 socket=/run/alpha.sock")
        # 同一个标题两份：不在的那一份文件名在前、启动更晚，活着的那一份照样排前面
        write("400.json", {"pid": 300, "sessionId": "session-epsilon", "name": "shared-title", "procStart": "1", "startedAt": 9000})
        write("500.json", {"pid": 100, "sessionId": "session-delta", "name": "shared-title", "procStart": "555", "startedAt": 100})
        named = entries_for_name(session_registry_entries(directory, fake_ticks) or [], "shared-title")
        expect("按名字查时活着的登记应当排在前面", [entry.session_id for entry in named] == ["session-delta", "session-epsilon"])
        expect("按名字查不到交空表", entries_for_name(entries or [], "no-such-name") == [])

    cells = 11
    if failures:
        print(f"  ✗ lib_session_registry 自证 {cells} 格里 {len(failures)} 格判错")
        print("     → 怎么办：照判错那几格的标签找对应的函数修；判据写在文件头")
        return 1
    print(f"  ✓ lib_session_registry 自证 {cells} 格都过（假的登记目录与假的 /proc 启动时刻，不读本机的会话）")
    return 0


def query_registry(command, value):
    directory = registry_directory()
    entries = session_registry_entries(directory)
    if entries is None:
        print(f"  ✗ 判不了：会话登记目录 {directory} 读不了，或里面一份能解的登记都没有（Claude Code 改了登记的格式也是这样）")
        print("     → 怎么办：ls 那个目录看还在不在、开一份看 pid、sessionId、name 这几个字段还在不在；字段变了改这份库的 session_registry_entries")
        return UNJUDGEABLE_EXIT
    matched = [named_entry_for_session(entries, value)] if command == "by-session" else entries_for_name(entries, value)
    matched = [entry for entry in matched if entry is not None]
    if not matched:
        what = "会话号" if command == "by-session" else "名字"
        print(f"  ✗ 会话登记里没有{what}是 {value} 的（查了 {len(entries)} 份登记）")
        print("     → 怎么办：会话号从 task-server/task status <号> 第一行取；名字从 ListAgents 取，标题要逐字相同；会话退出后登记被删了的查不到")
        return NOT_FOUND_EXIT
    for entry in matched:
        print(entry_line(entry))
    return 0


def main(arguments):
    if break_items() and arguments != ["--selftest"]:
        print(f"  ✗ {BREAK_VARIABLE}={os.environ.get(BREAK_VARIABLE)} 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒，这一次什么都没做")
        print(f"     → 怎么办：unset {BREAK_VARIABLE} 再来；要看开关红在哪一格，跑 {BREAK_VARIABLE}=<项> python3 scripts/agents/lib_session_registry.py --selftest")
        return USAGE_EXIT
    unknown = [item for item in break_items() if item not in KNOWN_BREAK_ITEMS]
    if unknown:
        print(f"  ✗ {BREAK_VARIABLE} 里的 {'、'.join(unknown)} 不认识，只认 {'、'.join(KNOWN_BREAK_ITEMS)}")
        print(f"     → 怎么办：照文件头的开关表改 {BREAK_VARIABLE} 再跑")
        return USAGE_EXIT
    if arguments == ["--selftest"]:
        return selftest()
    if len(arguments) == 2 and arguments[0] in ("by-session", "by-name") and arguments[1]:
        return query_registry(arguments[0], arguments[1])
    print(f"  ✗ 认不出的参数：{' '.join(arguments) or '（空）'}")
    print("     → 怎么办：照文件头「命令行」一段写：by-session <会话号>、by-name <名字或标题>，或单独一个 --selftest")
    return USAGE_EXIT


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
