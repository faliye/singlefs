#!/usr/bin/env python3
# admission: always 判的是此刻在跑的进程与没交回的子 agent，几秒读完；搬之前每次都要现判，上一次的结论不替这一次作保
# run-condition: none 只读 /proc 与 Claude Code 的会话记录，除了 python3 之外没有环境要求，不起子进程
"""整目录搬迁之前，核还有没有在跑的子 agent 与进程引着旧路径（欠账 C859，.claude/kb/milestone/03-third-txn.md「待优化」第 27 行）。

为什么：2026-10-11 脚本目录在两条线都有子 agent 在跑时整体搬家，在跑的子 agent 用的全路径中途失效，只能逐个发消息改；
用户 2026-10-18 定「挪出没有问题 但是要解决问题」：搬之前列出在跑的子 agent，有就判红。

用法：
    move-directory-in-use-check.py <旧目录> [--repository <仓根>] [--projects-directory <目录>] [--active-hours <小时>]
    move-directory-in-use-check.py --selftest      # MOVE_DIRECTORY_IN_USE_BREAK=<项> 时点名的格必须红
  <旧目录> 写仓根相对路径（例 .claude/gate.d）或绝对路径；仓根不给取这份脚本所在的仓。

判法（两半，各自逐个列出，有一处就红）：
  ① 进程：本用户能读到的每个进程（/proc/<pid>，跳过这个检查自己），命令行里有旧目录的绝对写法、或当前目录在仓根里而命令行里有它的仓根相对写法，
     当前目录、跑的程序（exe）、开着的文件（fd）落在旧目录底下，都算引着；写进程号、怎么引的与命令。
  ② 子 agent：会话记录目录（不给取 ~/.claude/projects/<仓根路径把 / 换成 ->，各会话的 <会话号>/subagents/agent-<号>.jsonl）里
     --active-hours（默认 24）以内改过、正文里出现旧目录的仓根相对写法或绝对写法、还没交回也没被停（判法是 scripts/agents/agent-watch.py 的
     AgentTranscript.is_done 与 stale_session_reason：交回按交回工具成功返回、被停认打断与 TaskStop、重启前旧进程留下的判已失效；这里按文件路径导入，不另写）的，
     写 agent 号、类型、描述与第一处提到旧目录的那一行原文。
  仓根相对写法统一成「<旧目录>/」（带结尾的 /），免得 scripts/gate 命中 scripts/gate-x。
退出码：0 没有引着它的；1 有（逐个列出）；2 用法错（旧目录不在仓里也不是绝对路径）。

弄坏开关（只给 --selftest 证红用；一次设一个）：MOVE_DIRECTORY_IN_USE_BREAK=
  processes-ignored    不看进程                                  自证「进程的当前目录与命令行引着旧目录」红
  agents-ignored       不看子 agent                              自证「没交回的子 agent 提到旧目录」红
  handed-back-counted  交回了的子 agent 也算                     自证「交回了的不算」红
  prefix-loose         仓根相对写法不带结尾的 /                  自证「名字只是前缀相同的不算」红
"""
import argparse
import importlib.util
import os
import shutil
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

BROKEN = os.environ.get("MOVE_DIRECTORY_IN_USE_BREAK", "")
KNOWN_BREAKS = ("processes-ignored", "agents-ignored", "handed-back-counted", "prefix-loose")
SNIPPET_LENGTH = 160


def load_agent_watch():
    """子 agent 交没交回、记录是不是已失效：看门狗那一份（文件头「判法」②），按文件路径导入。"""
    specification = importlib.util.spec_from_file_location("agent_watch_for_move_check", os.path.join(REPOSITORY_ROOT, "scripts", "agents", "agent-watch.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


AGENT_WATCH = load_agent_watch()


def forms_of(old, repository):
    """→ (绝对写法, 仓根相对写法「<旧目录>/」)；旧目录既不在仓里也不是绝对路径交 None。"""
    absolute = os.path.normpath(old if os.path.isabs(old) else os.path.join(repository, old))
    if not os.path.isabs(old) and not os.path.exists(absolute):
        return None
    relative = os.path.relpath(absolute, repository)
    if relative.startswith(".."):
        relative = None
    elif BROKEN != "prefix-loose":
        relative = relative.rstrip("/") + "/"
    return absolute, relative


def mentions(text, absolute, relative):
    """text 里第一处提到旧目录的下标；没有交 None。绝对写法要后面是 / 或结尾或非路径字符，免得 /x/gate 命中 /x/gate-y。"""
    positions = []
    start = 0
    while True:
        found = text.find(absolute, start)
        if found < 0:
            break
        following = text[found + len(absolute):found + len(absolute) + 1]
        if following in ("", "/") or not (following.isalnum() or following in "-_.") or BROKEN == "prefix-loose":
            positions.append(found)
            break
        start = found + 1
    if relative:
        found = text.find(relative)
        if found >= 0:
            positions.append(found)
    return min(positions) if positions else None


def processes_using(absolute, relative, repository):
    """→ [(进程号, 怎么引的, 命令)]（文件头「判法」①）。"""
    if BROKEN == "processes-ignored":
        return []
    found = []
    real = os.path.realpath(absolute)
    for name in sorted((entry for entry in os.listdir("/proc") if entry.isdigit()), key=int):
        pid = int(name)
        if pid == os.getpid():
            continue
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as handle:
                arguments = [part.decode("utf-8", "replace") for part in handle.read().split(b"\0") if part]
        except OSError:
            continue
        reasons = []
        command = " ".join(arguments)
        try:
            directory = os.readlink(f"/proc/{pid}/cwd")
        except OSError:
            directory = ""
        if mentions(command, absolute, None) is not None or (real != absolute and mentions(command, real, None) is not None):
            reasons.append("命令行里有它的绝对路径")
        elif relative and (directory == repository or directory.startswith(repository + "/")) and mentions(command, absolute, relative) is not None:
            reasons.append(f"当前目录在仓里、命令行里有「{relative}」")
        if directory == real or directory.startswith(real + "/"):
            reasons.append("当前目录在它底下")
        links = [f"/proc/{pid}/exe"]
        try:
            links += [f"/proc/{pid}/fd/{descriptor}" for descriptor in os.listdir(f"/proc/{pid}/fd")]
        except OSError:
            pass
        for link in links:
            try:
                target = os.readlink(link)
            except OSError:
                continue
            if target == real or target.startswith(real + "/"):
                reasons.append(f"{'跑的程序' if link.endswith('/exe') else '开着的文件'}在它底下（{target}）")
                break
        if reasons:
            found.append((pid, "；".join(reasons), command[:200]))
    return found


def default_projects_directory(repository):
    return os.path.join(os.path.expanduser("~"), ".claude", "projects", os.path.realpath(repository).replace("/", "-"))


def agents_using(absolute, relative, projects_directory, active_hours):
    """→ [(agent 号, 类型, 描述, 第一处提到旧目录的那一行)]（文件头「判法」②）。"""
    if BROKEN == "agents-ignored" or not os.path.isdir(projects_directory):
        return []
    found = []
    now = time.time()
    live = AGENT_WATCH.live_sessions()
    for session in sorted(os.scandir(projects_directory), key=lambda entry: entry.name):
        subagents = os.path.join(session.path, "subagents")
        if not session.is_dir() or not os.path.isdir(subagents):
            continue
        for entry in sorted(os.scandir(subagents), key=lambda item: item.name):
            if not (entry.name.startswith("agent-") and entry.name.endswith(".jsonl")) or now - entry.stat().st_mtime > active_hours * 3600:
                continue
            text = open(entry.path, encoding="utf-8", errors="replace").read()
            position = mentions(text, absolute, relative)
            if position is None:
                continue
            agent_id = entry.name[len("agent-"):-len(".jsonl")]
            transcript = AGENT_WATCH.AgentTranscript(agent_id, entry.path)
            if BROKEN != "handed-back-counted":
                if transcript.is_done():
                    continue
                if AGENT_WATCH.stale_session_reason(entry.path, transcript.last_timestamp, session.path, live):
                    continue
            line_start = text.rfind("\n", 0, position) + 1
            snippet = text[max(line_start, position - 24):position + 120].replace("\\n", " ")
            found.append((agent_id, transcript.agent_type, transcript.description, " ".join(snippet.split())[:SNIPPET_LENGTH]))
    return found


def run(arguments, say=print):
    if BROKEN and BROKEN not in KNOWN_BREAKS:
        say(f"  ✗ MOVE_DIRECTORY_IN_USE_BREAK 的「{BROKEN}」不认识（只认 {'、'.join(KNOWN_BREAKS)}）")
        say(f"  → 怎么办：弄坏开关只给自证用，平时不设")
        return 2
    repository = os.path.realpath(arguments.repository or REPOSITORY_ROOT)
    forms = forms_of(arguments.old, repository)
    if forms is None:
        say(f"  ✗ 认不出旧目录「{arguments.old}」：它不在仓 {repository} 里，也不是绝对路径")
        say(f"  → 怎么办：写仓根相对路径（例 .claude/gate.d）或绝对路径")
        return 2
    absolute, relative = forms
    processes = processes_using(absolute, relative, repository)
    agents = agents_using(absolute, relative, arguments.projects_directory or default_projects_directory(repository), arguments.active_hours)
    for pid, how, command in processes:
        say(f"  ✗ 进程 {pid}：{how}：{command}")  # gate-lint:detail
    for agent_id, kind, description, snippet in agents:
        say(f"  ✗ 子 agent {agent_id} {kind}「{description}」还没交回，会话记录里提到它：…{snippet}…")  # gate-lint:detail
    if processes or agents:
        say(f"  ✗ 旧目录 {absolute}（仓根相对写法「{relative or '—'}」）还有 {len(processes)} 个进程、{len(agents)} 个没交回的子 agent 引着，现在搬会让它们中途失效")  # gate-lint:summary
        say("  → 怎么办：等这些进程退出、子 agent 交回再搬；等不了的，给在跑的子 agent 发消息改路径，搬完在旧路径留一个指到新路径的符号链接，"
            "到它们都交回再删（.claude/rules/path-moves.md）；搬之前再跑一次这条到绿")
        return 1
    say(f"  ✓ 旧目录 {absolute} 没有进程、也没有 {arguments.active_hours:g} 小时内动过而没交回的子 agent 引着（查了 /proc 与会话记录；仓根相对写法「{relative or '—'}」）")
    return 0


def parse(argv):
    parser = argparse.ArgumentParser(description="整目录搬迁之前核还有没有在跑的子 agent 与进程引着旧路径")
    parser.add_argument("old")
    parser.add_argument("--repository")
    parser.add_argument("--projects-directory")
    parser.add_argument("--active-hours", type=float, default=24)
    return parser.parse_args(argv)


def selftest():
    """临时仓里放旧目录，起两个引着它的进程、一个不相干的；造会话记录：没交回而提到它的、交回了的、提到名字只是前缀相同的目录的。"""
    work = tempfile.mkdtemp(prefix="move-directory-in-use-selftest-")
    started, failures = [], []
    os.environ[AGENT_WATCH.SESSION_REGISTRY_ENV] = os.path.join(work, "no-session-registry")   # 判不了会话在不在，一律不当失效
    try:
        repository = os.path.join(work, "repository")
        old = os.path.join(repository, "scripts", "gate")
        for directory in (old, os.path.join(repository, "scripts", "gate-x")):
            os.makedirs(directory)
        projects = os.path.join(work, "projects")
        session = os.path.join(projects, "session-a")

        def transcript(agent_id, text, handed_back):
            records = [AGENT_WATCH.record_at(5, "user", f"派发提示：{text}"), AGENT_WATCH.bash_use(4, "t1", "ls", "m1"), AGENT_WATCH.bash_result(3.9, "t1")]
            if handed_back:
                records += AGENT_WATCH.handback_records(3, "h1", "m2")
            else:
                records.append(AGENT_WATCH.bash_use(1, "t2", f"cat {text}", "m3"))
            AGENT_WATCH.write_transcript(session, agent_id, records, {"agentType": "tooling-writer", "description": f"样本 {agent_id}"})

        transcript("runningold", "改 scripts/gate/apply-writer-patch.py", False)
        transcript("handedold", "改 scripts/gate/apply-writer-patch.py", True)
        transcript("prefixonly", "改 scripts/gate-x/other.py", False)
        in_cwd = subprocess.Popen(["sleep", "60"], cwd=old)
        in_arguments = subprocess.Popen(["bash", "-c", "sleep 60; true", os.path.join(old, "x.sh")], cwd=work)   # 带 ; true：bash 不把自己 exec 成 sleep，命令行里留着那条路径
        unrelated = subprocess.Popen(["sleep", "61"], cwd=work)
        started += [in_cwd, in_arguments, unrelated]
        time.sleep(0.3)
        said = []
        code = run(parse(["scripts/gate", "--repository", repository, "--projects-directory", projects]), say=said.append)
        text = "\n".join(said)
        if f"进程 {in_cwd.pid}：" not in text or f"进程 {in_arguments.pid}：" not in text or f"进程 {unrelated.pid}：" in text:
            failures.append(("进程的当前目录与命令行引着旧目录", text))
        if "子 agent runningold " not in text:
            failures.append(("没交回的子 agent 提到旧目录", text))
        if "子 agent handedold " in text:
            failures.append(("交回了的不算", text))
        if "子 agent prefixonly " in text:
            failures.append(("名字只是前缀相同的不算", text))
        if code != 1:
            failures.append(("有引着的就退 1", f"退 {code}：{text}"))
        for process in started:
            process.kill()
            process.wait()
        started = []
        said = []
        code = run(parse([os.path.join(repository, "scripts", "gate-x"), "--repository", repository, "--projects-directory", os.path.join(work, "no-projects")]),
                   say=said.append)
        if code != 0 or "✓ 旧目录" not in "\n".join(said):
            failures.append(("没人引着就退 0", f"退 {code}：{said}"))
    finally:
        for process in started:
            process.kill()
            process.wait()
        shutil.rmtree(work, ignore_errors=True)
    cells = ("进程的当前目录与命令行引着旧目录", "没交回的子 agent 提到旧目录", "交回了的不算", "名字只是前缀相同的不算", "有引着的就退 1", "没人引着就退 0")
    for cell, failure in failures:
        print(f"  ✗ 自证 {cell}：{failure}")  # gate-lint:detail
    if failures:
        print(f"  ✗ 自证 {len(cells)} 格里 {len(failures)} 格判错")  # gate-lint:summary
        print("  → 怎么办：看 processes_using / agents_using / mentions 的判法；MOVE_DIRECTORY_IN_USE_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ move-directory-in-use-check 自证通过（{len(cells)} 格）：当前目录在旧目录里与命令行带它的进程、没交回而提到它的子 agent 都报，"
          "不相干的进程、交回了的、名字只是前缀相同的不报；有就退 1，没有退 0")
    return 0


if __name__ == "__main__":
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    sys.exit(run(parse(sys.argv[1:])))
