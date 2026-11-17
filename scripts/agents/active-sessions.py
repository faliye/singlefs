#!/usr/bin/env python3
# admission: always 读的是此刻盘上的会话登记、此刻 /proc 里的进程与此刻的会话记录，上一次查到的不替这一次作保
# run-condition: none 只读会话登记目录与会话记录目录下的文件，不写任何东西，除了 python3 没有环境要求；登记目录只在 Claude Code 跑着的那一台有，在主 agent 那一台直接起
# gate-similar: agent-watch.py 盯的是本会话派出去的子 agent（交回没有、卡没卡、hook 检出），读子 agent 的会话记录、会叫醒主 agent；这里列本机全部活着的会话、按主会话记录判最近一次自己动作，只读、只打一次，不盯不叫醒，并进去会把一次性的查询塞进常驻的看门狗
# gate-similar: who-touched.py 按路径或符号数每个会话改过、点到过几次，按记录文件的修改时刻筛时间窗；这里不看点到什么，判的是最近一次自己动作的时刻，而且恰恰不能按修改时刻判，两份的输入同源、判据不同
"""查本机此刻活着的会话、各自最近一次自己动作是什么时候、停没停在弹窗上。只读，不写任何文件。

用法：
  active-sessions.py [--within-minutes N] [--sessions-directory <目录>] [--projects-directory <目录>]
  active-sessions.py --selftest

怎么认：
  会话：登记目录（--sessions-directory，其次环境变量 SESSION_REGISTRY_DIRECTORY，其次 lib_session_registry.py 的默认取法）里进程还活着的登记
    （判活与同一个会话号几份登记挑哪一份都照 lib_session_registry.py：session_registry_entries、named_entry_for_session）；进程已退出的登记不列。
  会话记录：<会话记录根>/*/<会话号>.jsonl；会话记录根取 --projects-directory，其次环境变量 ACTIVE_SESSIONS_PROJECTS_DIRECTORY，
    其次 lib_session_directory.py 的 projects_root()（不经 $HOME 拼）。同一个会话号在几个项目目录里都有记录的，取最近一次自己动作最晚的那一份。
  最近一次自己动作：会话记录里最后一条「自己动作」的 timestamp，不取文件修改时刻（别的会话发来消息、排队、钩子插提示都会刷新修改时刻）。自己动作是：
    type 为 assistant 的记录；type 为 user、不带 isMeta 与 isCompactSummary、origin.kind 没有或是 human 的记录（用户敲的字、工具结果）。
    不算的：type 为 queue-operation 的记录（enqueue、dequeue、remove）；origin.kind 是 peer（别的会话发来的消息）、task-notification（后台任务的完成通知）、
    coordinator（续做消息）的 user 记录；isMeta 的 user 记录（钩子与 harness 插的提示）；别的 type（attachment、system、api-request 这类）。
    origin.kind 与 isMeta 两个字段的认法与 agent-watch.py 的 is_new_instruction 同一判据（那里认的是给子 agent 的新指令，这里取反：收到的都不算自己动作），
    queue-operation 与 agent-watch.py 的 is_task_notification_record 同一判据。
  停在弹窗上：最后一条自己动作是 assistant 记录、里面有名为 AskUserQuestion 的 tool_use（它的结果是一条工具结果的 user 记录，来了就不再是最后一条）。

输出：每个活着的会话一行「<会话名> | 会话号 <号> | 最近一次自己动作 <日期>，<N> 分钟前 | 停在弹窗上：是/否 | <N> 分钟内自己动过：是/否」，
  日期按这个进程的本地时区（TZ）取；会话记录找不到、或里面没有自己动作的写明。末行「列了 <N> 个活着的会话，其中 <M> 个在 <N> 分钟内自己动过」。
  --within-minutes 不给是 60。

退出码：0 列出了；1 登记里没有一份进程还活着的登记；2 用法错（--within-minutes 不是正数、参数认不出），或平常跑时设了弄坏开关；
  3 判不了（登记目录读不了或一份能解的登记都没有、会话记录根取不到）。

弄坏开关（只给 --selftest 证红用；平常跑设了 ACTIVE_SESSIONS_BREAK、SESSION_REGISTRY_BREAK 或 SESSION_DIRECTORY_BREAK，参数解析之前就退 2、点名它只在自证里认）。
逐项写开关名、弄坏哪一处、打开后自证哪几格红（失败行「✗ 自检：<格名>：…」，「」里是格名）：
  ACTIVE_SESSIONS_BREAK=by-file-mtime  last_own_action() 交会话记录文件的修改时刻，不读记录
      「只在末尾追加 enqueue 行的会话记录判不活动」「末尾只有别的会话发来的消息判不活动」「末尾只有钩子插的提示判不活动」
      「末尾只有后台任务完成通知判不活动」「最近一条 assistant 在时限内判活动，文件修改时刻很旧也一样」
      「停在弹窗上之后只收到别的会话的消息，照报停在弹窗上」「末行报数」
  ACTIVE_SESSIONS_BREAK=queue-operation-counted  is_own_action() 把 queue-operation 记录算成自己动作
      「只在末尾追加 enqueue 行的会话记录判不活动」「停在弹窗上之后只收到别的会话的消息，照报停在弹窗上」「末行报数」
  ACTIVE_SESSIONS_BREAK=cross-session-message-counted  is_own_action() 把 origin.kind 为 peer 的 user 记录算成自己动作
      「末尾只有别的会话发来的消息判不活动」「停在弹窗上之后只收到别的会话的消息，照报停在弹窗上」「末行报数」
  ACTIVE_SESSIONS_BREAK=hook-injection-counted  is_own_action() 把 isMeta 的 user 记录算成自己动作
      「末尾只有钩子插的提示判不活动」「末行报数」
  ACTIVE_SESSIONS_BREAK=popup-unseen  last_own_action() 一律报不在弹窗上
      「停在弹窗上之后只收到别的会话的消息，照报停在弹窗上」
  ACTIVE_SESSIONS_BREAK=dead-sessions-listed  alive_sessions() 不看登记里的进程活没活，进程已退出的登记也列
      「进程已退出的登记不列」「末行报数」
"""
import argparse
import concurrent.futures
import glob
import hashlib
import json
import os
import sys
import tempfile
import time
from dataclasses import dataclass
from datetime import datetime

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, SCRIPT_DIRECTORY)
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "scripts"))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, "scripts", "hosts"))
import heavy_threads  # noqa: E402  每台默认重活线程数只在这一份里取（登记位 .claude/heavy-thread-default）
import lib_session_directory as session_directories  # noqa: E402  会话记录根的唯一取法（不经 $HOME）
import lib_session_registry as session_registry  # noqa: E402  会话登记与判活的唯一取法
from project_preflight import preflight  # noqa: E402

BREAK_VARIABLE = "ACTIVE_SESSIONS_BREAK"
KNOWN_BREAK_ITEMS = ("by-file-mtime", "queue-operation-counted", "cross-session-message-counted", "hook-injection-counted",
                     "popup-unseen", "dead-sessions-listed")
# 平常跑设了就拒的弄坏开关：自己的，与经 import 进来的两份库的（它们被 import 时照认开关）
GUARDED_BREAK_VARIABLES = (BREAK_VARIABLE, session_registry.BREAK_VARIABLE, session_directories.BREAK_VARIABLE)
PROJECTS_DIRECTORY_VARIABLE = "ACTIVE_SESSIONS_PROJECTS_DIRECTORY"
DEFAULT_WITHIN_MINUTES = 60
POPUP_TOOL_NAME = "AskUserQuestion"
OWN_USER_ORIGINS = (None, "human")   # user 记录的 origin.kind 是这几种之一才算自己动作；peer、task-notification、coordinator 都是收到的
SECONDS_PER_MINUTE = 60
LISTED_EXIT = 0
NOTHING_ALIVE_EXIT = 1
USAGE_EXIT = 2
UNJUDGEABLE_EXIT = 3


def break_items(environment=None):
    environment = os.environ if environment is None else environment
    return [item for item in environment.get(BREAK_VARIABLE, "").split(",") if item]


def broken(item):
    return item in break_items()


def break_switch_refusal(arguments, environment):
    """弄坏开关只在自证里认：设了而命令行不是单独一个 --selftest，交回拒绝说明（两行）；否则交 None。"""
    set_names = [name for name in GUARDED_BREAK_VARIABLES if environment.get(name)]
    if not set_names or arguments == ["--selftest"]:
        return None
    return (f"  ✗ {'、'.join(set_names)} 是弄坏开关，只在自证（单独一个 --selftest）里认；平常跑设了就拒，这一次什么都没做",
            f"     → 怎么办：unset {' '.join(set_names)} 再来；要看开关红在哪一格，跑 {BREAK_VARIABLE}=<项> python3 scripts/agents/active-sessions.py --selftest")


@dataclass(frozen=True)
class LastOwnAction:
    epoch_seconds: float | None   # 最后一条自己动作的时刻；记录里没有自己动作是 None
    is_on_popup: bool             # 最后一条自己动作是 AskUserQuestion 的调用


@dataclass(frozen=True)
class SessionRow:
    name: str
    session_id: str
    transcript_found: bool
    last_action: LastOwnAction


def origin_kind(record):
    origin = record.get("origin")
    return origin.get("kind") if isinstance(origin, dict) else None


def is_own_action(record):
    """这一条记录是不是会话自己的动作（判据在文件头「最近一次自己动作」）。"""
    record_type = record.get("type")
    if record_type == "queue-operation":
        return broken("queue-operation-counted")   # 弄坏开关：排队操作也算
    if record_type == "assistant":
        return True
    if record_type != "user":
        return False
    kind = origin_kind(record)
    if kind not in OWN_USER_ORIGINS:
        return kind == "peer" and broken("cross-session-message-counted")   # 弄坏开关：别的会话发来的消息也算
    if record.get("isMeta") or record.get("isCompactSummary"):
        return broken("hook-injection-counted")   # 弄坏开关：钩子与 harness 插的提示也算
    return True


def calls_popup(record):
    message = record.get("message")
    content = message.get("content") if isinstance(message, dict) else None
    return record.get("type") == "assistant" and isinstance(content, list) and any(
        isinstance(block, dict) and block.get("type") == "tool_use" and block.get("name") == POPUP_TOOL_NAME for block in content)


def timestamp_seconds(text):
    """会话记录里的 ISO 时间戳 → 纪元秒；认不出交 None。"""
    if not isinstance(text, str) or not text:
        return None
    try:
        return datetime.fromisoformat(text.replace("Z", "+00:00")).timestamp()
    except ValueError:
        return None


def last_own_action(transcript_path):
    """读一份会话记录，交最后一条自己动作的时刻与它是不是弹窗调用。解不了的行跳过。"""
    if broken("by-file-mtime"):   # 弄坏开关：拿文件修改时刻当最近活动
        return LastOwnAction(os.path.getmtime(transcript_path), False)
    epoch_seconds, is_on_popup = None, False
    with open(transcript_path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            try:
                record = json.loads(line)
            except ValueError:
                continue
            if not isinstance(record, dict) or not is_own_action(record):
                continue
            seconds = timestamp_seconds(record.get("timestamp"))
            if seconds is None:
                continue
            epoch_seconds = seconds
            is_on_popup = calls_popup(record) and not broken("popup-unseen")   # 弄坏开关：一律不报弹窗
    return LastOwnAction(epoch_seconds, is_on_popup)


def later_action(first, second):
    if first.epoch_seconds is None:
        return second
    if second.epoch_seconds is None:
        return first
    return second if second.epoch_seconds > first.epoch_seconds else first


def alive_sessions(entries):
    """登记里进程还活着的会话，一个会话号一份（named_entry_for_session 挑），按会话号排。"""
    if broken("dead-sessions-listed"):   # 弄坏开关：不看活没活
        listed = entries
    else:
        listed = [entry for entry in entries if entry.is_alive]
    session_ids = sorted({entry.session_id for entry in listed})
    return [session_registry.named_entry_for_session(entries, session_id) for session_id in session_ids]


def session_rows(entries, projects_directory):
    rows = []
    for entry in alive_sessions(entries):
        transcripts = sorted(glob.glob(os.path.join(glob.escape(projects_directory), "*", f"{glob.escape(entry.session_id)}.jsonl")))
        action = LastOwnAction(None, False)
        for transcript in transcripts:
            action = later_action(action, last_own_action(transcript))
        rows.append(SessionRow(entry.name or "（登记里没有名字）", entry.session_id, bool(transcripts), action))
    return rows


def minutes_since(row, now_seconds):
    if row.last_action.epoch_seconds is None:
        return None
    return int((now_seconds - row.last_action.epoch_seconds) // SECONDS_PER_MINUTE)


def acted_within(row, now_seconds, within_minutes):
    return row.last_action.epoch_seconds is not None and now_seconds - row.last_action.epoch_seconds <= within_minutes * SECONDS_PER_MINUTE


def render_row(row, now_seconds, within_minutes):
    if not row.transcript_found:
        action = "会话记录没找到"
    elif row.last_action.epoch_seconds is None:
        action = "会话记录里没有自己动作"
    else:
        action = (f"最近一次自己动作 {datetime.fromtimestamp(row.last_action.epoch_seconds).astimezone().strftime('%Y-%m-%d')}，"
                  f"{minutes_since(row, now_seconds)} 分钟前")
    return (f"{row.name} | 会话号 {row.session_id} | {action} | 停在弹窗上：{'是' if row.last_action.is_on_popup else '否'} | "
            f"{within_minutes:g} 分钟内自己动过：{'是' if acted_within(row, now_seconds, within_minutes) else '否'}")


def summary_line(rows, now_seconds, within_minutes):
    active = sum(1 for row in rows if acted_within(row, now_seconds, within_minutes))
    return f"列了 {len(rows)} 个活着的会话，其中 {active} 个在 {within_minutes:g} 分钟内自己动过"


def main(command_line, environment=None, stream=None, start_ticks_of=session_registry.process_start_ticks, now_seconds=None):
    environment = os.environ if environment is None else environment
    stream = sys.stdout if stream is None else stream
    parser = argparse.ArgumentParser(prog="active-sessions.py", add_help=True)
    parser.add_argument("--within-minutes", type=float, default=DEFAULT_WITHIN_MINUTES)
    parser.add_argument("--sessions-directory")
    parser.add_argument("--projects-directory")
    parser.add_argument("--selftest", action="store_true")
    arguments, unknown = parser.parse_known_args(command_line)
    if arguments.selftest:
        return selftest()
    if unknown or arguments.within_minutes <= 0:
        print(f"  ✗ 用法不对：{' '.join(unknown) if unknown else f'--within-minutes {arguments.within_minutes:g} 不是正数'}", file=stream)
        print("     → 怎么办：写 active-sessions.py [--within-minutes N] [--sessions-directory <目录>] [--projects-directory <目录>]，N 是正数，例如 --within-minutes 60", file=stream)
        return USAGE_EXIT
    sessions_directory = arguments.sessions_directory or session_registry.registry_directory(environment)
    try:
        projects_directory = arguments.projects_directory or environment.get(PROJECTS_DIRECTORY_VARIABLE) or session_directories.projects_root(environment)
    except session_directories.SessionDirectoryError as error:
        print(f"  ✗ 取不到会话记录根：{error}", file=stream)
        print(f"     → 怎么办：{error.howto}；或用 --projects-directory 直接给 <家目录>/.claude/projects", file=stream)
        return UNJUDGEABLE_EXIT
    entries = session_registry.session_registry_entries(sessions_directory, start_ticks_of)
    if entries is None:
        print(f"  ✗ 判不了：会话登记目录 {sessions_directory} 读不了，或里面一份能解的登记都没有", file=stream)
        print("     → 怎么办：ls 那个目录看还在不在；不在这一台（登记目录只在 Claude Code 跑着的那一台有）就到那一台起，或用 --sessions-directory 给对的目录", file=stream)
        return UNJUDGEABLE_EXIT
    rows = session_rows(entries, projects_directory)
    if not rows:
        print(f"  ✗ 会话登记目录 {sessions_directory} 里 {len(entries)} 份登记，没有一份的进程还活着", file=stream)
        print("     → 怎么办：核对 --sessions-directory 是不是 Claude Code 跑着的那一台的登记目录（这个脚本该在那一台直接起）", file=stream)
        return NOTHING_ALIVE_EXIT
    now_seconds = time.time() if now_seconds is None else now_seconds
    for row in rows:
        print(render_row(row, now_seconds, arguments.within_minutes), file=stream)
    print(summary_line(rows, now_seconds, arguments.within_minutes), file=stream)
    return LISTED_EXIT


# ── 自证：假的会话登记与会话记录建在临时目录，进程活没活由假的启动时刻表定，「此刻」固定，结束删掉

SELFTEST_NOW = 1800000000.0
SELFTEST_WITHIN_MINUTES = 60
SELFTEST_STALE_SECONDS = 3 * 3600


def stamp(epoch_seconds):
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(epoch_seconds))  # clock-times:allow 造会话记录里时间戳字段的自检输入


def assistant_text(minutes_ago):
    return {"type": "assistant", "timestamp": stamp(SELFTEST_NOW - minutes_ago * SECONDS_PER_MINUTE),
            "message": {"role": "assistant", "content": [{"type": "text", "text": "做完了这一步"}]}}


def assistant_popup(minutes_ago, call):
    return {"type": "assistant", "timestamp": stamp(SELFTEST_NOW - minutes_ago * SECONDS_PER_MINUTE),
            "message": {"role": "assistant", "content": [{"type": "tool_use", "id": call, "name": POPUP_TOOL_NAME, "input": {"questions": []}}]}}


def tool_result(minutes_ago, call):
    return {"type": "user", "timestamp": stamp(SELFTEST_NOW - minutes_ago * SECONDS_PER_MINUTE),
            "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": call, "content": "选了第二项"}]}}


def user_text(minutes_ago, origin=None):
    record = {"type": "user", "timestamp": stamp(SELFTEST_NOW - minutes_ago * SECONDS_PER_MINUTE),
              "message": {"role": "user", "content": [{"type": "text", "text": "接着做"}]}}
    if origin is not None:
        record["origin"] = {"kind": origin}
    return record


def enqueue(minutes_ago):
    return {"type": "queue-operation", "operation": "enqueue", "timestamp": stamp(SELFTEST_NOW - minutes_ago * SECONDS_PER_MINUTE), "content": "别的会话的消息"}


def peer_message(minutes_ago):
    return {"type": "user", "isMeta": True, "origin": {"kind": "peer"}, "timestamp": stamp(SELFTEST_NOW - minutes_ago * SECONDS_PER_MINUTE),
            "message": {"role": "user", "content": "别的会话发来的消息"}}


def hook_injection(minutes_ago):
    return {"type": "user", "isMeta": True, "timestamp": stamp(SELFTEST_NOW - minutes_ago * SECONDS_PER_MINUTE),
            "message": {"role": "user", "content": "钩子插的提示"}}


def task_notification(minutes_ago):
    return {"type": "user", "origin": {"kind": "task-notification"}, "timestamp": stamp(SELFTEST_NOW - minutes_ago * SECONDS_PER_MINUTE),
            "message": {"role": "user", "content": "后台任务完成了"}}


STALE_MINUTES = SELFTEST_STALE_SECONDS // SECONDS_PER_MINUTE
# 会话号 → (pid, 进程活着吗, 会话记录的各条, 文件修改时刻离此刻几秒；None 是不建会话记录)
SELFTEST_SESSIONS = {
    "session-enqueue-only": (101, True, [assistant_text(STALE_MINUTES), enqueue(2), enqueue(1)], 30),
    "session-peer-only": (102, True, [assistant_text(STALE_MINUTES), peer_message(2), peer_message(1)], 30),
    "session-hook-only": (103, True, [assistant_text(STALE_MINUTES), hook_injection(1)], 30),
    "session-notified-only": (104, True, [assistant_text(STALE_MINUTES), task_notification(1)], 30),
    "session-recent-assistant": (105, True, [assistant_text(10)], SELFTEST_STALE_SECONDS),
    "session-recent-user": (106, True, [assistant_text(STALE_MINUTES), user_text(5)], 5 * SECONDS_PER_MINUTE),
    "session-human-origin": (107, True, [assistant_text(STALE_MINUTES), user_text(2, origin="human")], 2 * SECONDS_PER_MINUTE),
    "session-on-popup": (108, True, [assistant_text(25), assistant_popup(20, "call-popup"), enqueue(3), peer_message(3)], 3 * SECONDS_PER_MINUTE),
    "session-popup-answered": (109, True, [assistant_popup(40, "call-answered"), tool_result(35, "call-answered"), assistant_text(30)], 30 * SECONDS_PER_MINUTE),
    "session-process-exited": (110, False, [assistant_text(1)], SECONDS_PER_MINUTE),
    "session-without-transcript": (111, True, None, None),
}
SELFTEST_ALIVE_LISTED = sum(1 for _, alive, _, _ in SELFTEST_SESSIONS.values() if alive)
SELFTEST_ACTIVE = 5   # recent-assistant、recent-user、human-origin、on-popup、popup-answered


def build_fixture(work):
    """造假的登记目录与会话记录根，→ (登记目录, 会话记录根, 假的启动时刻表)。登记的 procStart 一律 "7"；活着的进程在表里答 "7"，退出的不在表里。"""
    registry, projects = os.path.join(work, "sessions"), os.path.join(work, "projects")
    project = os.path.join(projects, "-fake-repository")
    os.makedirs(registry)
    os.makedirs(project)
    start_ticks = {}
    for session_id, (pid, alive, records, mtime_seconds_ago) in SELFTEST_SESSIONS.items():
        with open(os.path.join(registry, f"{pid}.json"), "w", encoding="utf-8") as handle:
            json.dump({"pid": pid, "sessionId": session_id, "name": f"name-of-{session_id}", "procStart": "7", "startedAt": pid * 1000}, handle)
        if alive:
            start_ticks[pid] = "7"
        if records is None:
            continue
        transcript = os.path.join(project, f"{session_id}.jsonl")
        with open(transcript, "w", encoding="utf-8") as handle:
            handle.write("".join(json.dumps(record, ensure_ascii=False) + "\n" for record in records) + "{坏行\n")
        modified = SELFTEST_NOW - mtime_seconds_ago
        os.utime(transcript, (modified, modified))
    return registry, projects, start_ticks.get


def tree_fingerprint(*directories):
    """每份文件的 (路径, 大小, 修改时刻, sha256) 与每层目录的清单：自证前后比，证明只读。"""
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


class CollectingStream:
    """main() 往里打的行；每格自己一份，线程之间不共用 sys.stdout。"""

    def __init__(self):
        self.lines = []

    def write(self, text):
        self.lines.append(text)

    def flush(self):
        pass

    def text(self):
        return "".join(self.lines)


def selftest_cells(registry, projects, start_ticks_of):
    """→ [(格名, 判这一格的函数)]：函数交 None 是过，交一句是没过的原因。各格只读同一份造好的假目录，互不依赖。"""
    def rows_by_session():
        entries = session_registry.session_registry_entries(registry, start_ticks_of) or []
        return {row.session_id: row for row in session_rows(entries, projects)}

    def expect_activity(session_id, want_active):
        def judge():
            row = rows_by_session().get(session_id)
            if row is None:
                return f"{session_id} 没列出来"
            got = acted_within(row, SELFTEST_NOW, SELFTEST_WITHIN_MINUTES)
            return None if got == want_active else f"{session_id} 应当判{'活动' if want_active else '不活动'}，实际判{'活动' if got else '不活动'}（最近一次自己动作 {minutes_since(row, SELFTEST_NOW)} 分钟前）"
        return judge

    def expect_popup(session_id, want_popup):
        def judge():
            row = rows_by_session().get(session_id)
            if row is None:
                return f"{session_id} 没列出来"
            return None if row.last_action.is_on_popup == want_popup else f"{session_id} 停在弹窗上应当报{'是' if want_popup else '否'}，实际{'是' if row.last_action.is_on_popup else '否'}"
        return judge

    def exited_not_listed():
        rows = rows_by_session()
        return None if "session-process-exited" not in rows else "进程已退出的登记被列出来了"

    def without_transcript_listed():
        row = rows_by_session().get("session-without-transcript")
        if row is None:
            return "会话记录找不到的活会话没列"
        line = render_row(row, SELFTEST_NOW, SELFTEST_WITHIN_MINUTES)
        return None if not row.transcript_found and "会话记录没找到" in line and "自己动过：否" in line else f"那一行是 {line!r}"

    def summary_counted():
        stream = CollectingStream()
        code = main(["--within-minutes", str(SELFTEST_WITHIN_MINUTES), "--sessions-directory", registry, "--projects-directory", projects],
                    {}, stream, start_ticks_of, SELFTEST_NOW)
        lines = stream.text().splitlines()
        want = f"列了 {SELFTEST_ALIVE_LISTED} 个活着的会话，其中 {SELFTEST_ACTIVE} 个在 {SELFTEST_WITHIN_MINUTES} 分钟内自己动过"
        rows = [line for line in lines if "| 会话号 " in line]
        if code != LISTED_EXIT or not lines or lines[-1] != want or len(rows) != SELFTEST_ALIVE_LISTED:
            return f"应当退 {LISTED_EXIT}、{SELFTEST_ALIVE_LISTED} 行加末行「{want}」，实际退 {code}、{len(rows)} 行、末行 {lines[-1] if lines else '（空）'!r}"
        return None

    def directories_from_environment():
        stream = CollectingStream()
        code = main([], {session_registry.REGISTRY_DIRECTORY_ENV: registry, PROJECTS_DIRECTORY_VARIABLE: projects}, stream, start_ticks_of, SELFTEST_NOW)
        return None if code == LISTED_EXIT and "name-of-session-recent-assistant" in stream.text() else f"退 {code}：{stream.text()[-300:]!r}"

    def usage_refused():
        stream = CollectingStream()
        code = main(["--within-minutes", "0", "--sessions-directory", registry, "--projects-directory", projects], {}, stream, start_ticks_of, SELFTEST_NOW)
        unknown_stream = CollectingStream()
        unknown_code = main(["--no-such-option"], {}, unknown_stream, start_ticks_of, SELFTEST_NOW)
        return None if code == USAGE_EXIT and unknown_code == USAGE_EXIT and "怎么办" in stream.text() else f"--within-minutes 0 退 {code}、认不出的参数退 {unknown_code}"

    def unreadable_registry():
        stream = CollectingStream()
        code = main(["--sessions-directory", os.path.join(registry, "no-such-directory"), "--projects-directory", projects], {}, stream, start_ticks_of, SELFTEST_NOW)
        return None if code == UNJUDGEABLE_EXIT and "怎么办" in stream.text() else f"退 {code}：{stream.text()[-200:]!r}"

    def break_guarded():
        refused = break_switch_refusal(["--within-minutes", "5"], {BREAK_VARIABLE: "popup-unseen"})
        library_refused = break_switch_refusal([], {session_registry.BREAK_VARIABLE: "first-entry-for-session"})
        allowed = break_switch_refusal(["--selftest"], {BREAK_VARIABLE: "popup-unseen"}) is None and break_switch_refusal([], {}) is None
        return None if refused is not None and library_refused is not None and allowed else "平常跑设了弄坏开关没拒，或 --selftest、没设开关时拒了"

    return [
        ("只在末尾追加 enqueue 行的会话记录判不活动", expect_activity("session-enqueue-only", False)),
        ("末尾只有别的会话发来的消息判不活动", expect_activity("session-peer-only", False)),
        ("末尾只有钩子插的提示判不活动", expect_activity("session-hook-only", False)),
        ("末尾只有后台任务完成通知判不活动", expect_activity("session-notified-only", False)),
        ("最近一条 assistant 在时限内判活动，文件修改时刻很旧也一样", expect_activity("session-recent-assistant", True)),
        ("最近一条用户敲的字在时限内判活动", expect_activity("session-recent-user", True)),
        ("origin 为 human 的用户消息算自己动作", expect_activity("session-human-origin", True)),
        ("停在弹窗上之后只收到别的会话的消息，照报停在弹窗上", expect_popup("session-on-popup", True)),
        ("弹窗有了结果之后不报停在弹窗上", expect_popup("session-popup-answered", False)),
        ("进程已退出的登记不列", exited_not_listed),
        ("会话记录找不到的活会话照列、写明没找到", without_transcript_listed),
        ("末行报数", summary_counted),
        ("目录经环境变量换成假的", directories_from_environment),
        ("用法错退 2", usage_refused),
        ("登记目录读不了退 3", unreadable_registry),
        ("平常跑设弄坏开关要被拒", break_guarded),
    ]


def selftest():
    unknown = [item for item in break_items() if item not in KNOWN_BREAK_ITEMS]
    if unknown:
        print(f"  ✗ 自检：{BREAK_VARIABLE} 里的 {'、'.join(unknown)} 不认识，只认 {'、'.join(KNOWN_BREAK_ITEMS)}")
        print("     → 怎么办：弄坏开关只写文件头开关表里列的项")
        return 1
    try:
        jobs = heavy_threads.default_heavy_threads()
    except heavy_threads.HeavyThreadDefaultError as error:
        print(f"  ✗ 自检：取不到每台默认重活线程数：{error}")
        print(f"     → 怎么办：{error.howto}")
        return 1
    failures = []
    with tempfile.TemporaryDirectory(prefix="active-sessions-selftest-") as work:
        registry, projects, start_ticks_of = build_fixture(work)
        before = tree_fingerprint(registry, projects)
        cells = selftest_cells(registry, projects, start_ticks_of)
        with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
            verdicts = list(pool.map(lambda cell: cell[1](), cells))
        if len(verdicts) != len(cells):
            failures.append(("收回的格数与派出去的对得上", f"派出去 {len(cells)} 格，收回 {len(verdicts)} 格"))
        failures += [(label, problem) for (label, _), problem in zip(cells, verdicts) if problem is not None]
        if tree_fingerprint(registry, projects) != before:
            failures.append(("全程只读不写", "自证前后假的登记目录与会话记录根的文件清单、大小、修改时刻或内容变了"))
    for label, problem in failures:
        print(f"  ✗ 自检：{label}：{problem}")  # gate-lint:detail
    if failures:
        print(f"  自检没过 {len(failures)} 格（逐格的失败行在上面）")  # gate-lint:summary
        print("     → 怎么办：按各行点名的格查 is_own_action()、last_own_action()、alive_sessions() 与 main()；"
              f"{BREAK_VARIABLE} 设着的话（项与红的格见文件头）这里本来就该红")
        return 1
    print(f"  ✓ active-sessions 自检 {len(cells) + 1} 格都过（并行 {jobs} 路；假的登记 {len(SELFTEST_SESSIONS)} 份、活着的 {SELFTEST_ALIVE_LISTED} 个、"
          f"{SELFTEST_WITHIN_MINUTES} 分钟内自己动过的 {SELFTEST_ACTIVE} 个；只收消息、排队、钩子插提示的会话判不活动，停在弹窗上的报出来，全程只读）")
    return 0


if __name__ == "__main__":
    refusal = break_switch_refusal(sys.argv[1:], os.environ)
    if refusal is not None:
        print(refusal[0], file=sys.stderr)
        print(refusal[1], file=sys.stderr)
        sys.exit(USAGE_EXIT)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
