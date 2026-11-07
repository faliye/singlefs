#!/usr/bin/env python3
# admission: always 判的是此刻在跑的进程与没交回的子 agent，几秒读完；搬之前每次都要现判，上一次的结论不替这一次作保
# run-condition: none 只读 /proc 与 Claude Code 的会话记录，除了 python3 之外没有环境要求，不起子进程
"""整目录搬迁之前，核还有没有在跑的子 agent 与进程引着旧路径（欠账 C859，.claude/kb/milestone/03-third-txn.md「待优化」第 27 行）。

为什么：脚本目录在有子 agent 在跑时整体搬家，在跑的子 agent 用的全路径会中途失效，只能逐个发消息改；
所以搬之前列出在跑的子 agent，有就判红。

用法：
    move-directory-in-use-check.py <旧目录> [--repository <仓根>] [--projects-directory <目录>] [--active-hours <小时>]
    move-directory-in-use-check.py <旧目录> --moved-to <新目录> [同上几个选项]   # 搬完那一半：留没留链、该不该删链
    move-directory-in-use-check.py --selftest      # MOVE_DIRECTORY_IN_USE_BREAK=<项> 时点名的格必须红
  <旧目录> 写仓根相对路径（例 .claude/gate.d）或绝对路径；仓根不给取这份脚本所在的仓。

判法（两半，各自逐个列出，有一处就红）：
  ① 进程：本用户能读到的每个进程（/proc/<pid>，跳过这个检查自己与它往上的每一层父进程：起它的那条命令行里本来就写着旧目录），命令行里有旧目录的绝对写法、或当前目录在仓根里而命令行里有它的仓根相对写法，
     当前目录、跑的程序（exe）、开着的文件（fd）落在旧目录底下，都算引着；写进程号、怎么引的与命令。
  ② 子 agent：会话记录目录（--projects-directory 不给取 <家目录>/.claude/projects/<仓根路径把 / 换成 ->：仓根路径取 --repository，没给取统一任务服务给任务的来源仓根
     TASK_SERVER_SOURCE_REPOSITORY，再没有取这份脚本所在的仓；家目录从来源仓根的路径取（两台一致，不用 $HOME），没有来源仓根取本机家目录；取法只在
     scripts/agents/lib_session_directory.py 里。各会话的 <会话号>/subagents/agent-<号>.jsonl）里
     --active-hours（默认 24）以内改过、正文里出现旧目录的仓根相对写法或绝对写法、还没交回也没被停（判法是 scripts/agents/agent-watch.py 的
     AgentTranscript.is_done 与 stale_session_reason：交回按交回工具成功返回、被停认打断与 TaskStop、重启前旧进程留下的判已失效；这里按文件路径导入，不另写）的，
     写 agent 号、类型、描述与第一处提到旧目录的那一行原文。
  仓根相对写法统一成「<旧目录>/」（带结尾的 /），免得 scripts/gate 命中 scripts/gate-x。
  ③ 搬完（带 --moved-to <新目录>）：进程只看命令行（当前目录、exe 与 fd 跟着 inode 走，搬了不失效），子 agent 照 ②；再看旧路径是什么：
     还有引着的而旧路径不在（没留链）判红；还有引着的、旧路径是指到新目录的符号链接，退 3（链留着，等它们交回再跑这条）；
     没人引着了、链还在判红（删链）；没人引着、旧路径也不在，退 0；旧路径还是一份真目录或文件、或是指到别处的链，判红。
退出码：0 没有引着它的（搬完那一半：没人引着、链也删了）；1 有（逐个列出），或搬完那一半判红；2 用法错（旧目录不在仓里也不是绝对路径、--moved-to 的新目录不在）；
  3 搬完那一半：还有引着的、链留着，到它们都交回再跑一次这条。

弄坏开关（只给 --selftest 证红用；一次设一个；--selftest 本身与它起的子进程认，平常跑设了退 2、点名只在自证里认）。
下一行起写打开后自证红的格（失败行 `自证 <格名>：` 起头）：
  MOVE_DIRECTORY_IN_USE_BREAK=processes-ignored  不看进程
      「自证 进程的当前目录与命令行引着旧目录」「自证 搬完还有人引着、链留着：退 3，只认命令行」
  MOVE_DIRECTORY_IN_USE_BREAK=agents-ignored  不看子 agent
      「自证 没交回的子 agent 提到旧目录」「自证 搬完还有人引着而没留链」
  MOVE_DIRECTORY_IN_USE_BREAK=handed-back-counted  交回了的子 agent 也算
      「自证 交回了的不算」
  MOVE_DIRECTORY_IN_USE_BREAK=prefix-loose  仓根相对写法不带结尾的 /
      「自证 名字只是前缀相同的不算」
  MOVE_DIRECTORY_IN_USE_BREAK=link-not-required  搬完还有引着的、旧路径不在也不判红
      「自证 搬完还有人引着而没留链」
  MOVE_DIRECTORY_IN_USE_BREAK=leftover-link-ignored  搬完没人引着了、链还在也不判红
      「自证 都交回之后链还在」
  MOVE_DIRECTORY_IN_USE_BREAK=ancestors-counted  起这个检查的父进程也算
      「自证 起这个检查的那条命令行写着旧路径不算」
scripts/agents/lib_session_directory.py 的开关（判法在那一份，开关表也在它的文件头；这里同样只在 --selftest 本身与它起的子进程认），打开后这里红的格：
  SESSION_DIRECTORY_BREAK=home-from-environment  设了 TASK_SERVER_SOURCE_REPOSITORY 也取 HOME
      「自证 没给 --projects-directory 时取来源仓根的会话目录」
"""
import argparse
import contextlib
import importlib.util
import os
import shutil
import subprocess
import signal
import sys
import tempfile
import time

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, "scripts", "agents"))
import lib_session_directory as session_directories  # noqa: E402  默认会话记录目录的唯一取法（来源仓根 TASK_SERVER_SOURCE_REPOSITORY，不经 $HOME；agent-cost.py、agent-handover.py、agent-watch.py、watch.sh 同取它）

BROKEN = os.environ.get("MOVE_DIRECTORY_IN_USE_BREAK", "")
KNOWN_BREAKS = ("processes-ignored", "agents-ignored", "handed-back-counted", "prefix-loose", "link-not-required", "leftover-link-ignored", "ancestors-counted")


def own_lineage():
    """这个检查自己与它往上的每一层父进程（读 /proc/<pid>/stat 的父进程号）：起它的命令行里写着旧目录，不算引着。"""
    lineage, pid = set(), os.getpid()
    while pid > 1 and pid not in lineage:
        lineage.add(pid)
        if BROKEN == "ancestors-counted":
            break
        try:
            with open(f"/proc/{pid}/stat", encoding="utf-8", errors="replace") as handle:
                pid = int(handle.read().rsplit(")", 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            break
    return lineage
SNIPPET_LENGTH = 160


def load_agent_watch():
    """子 agent 交没交回、记录是不是已失效：看门狗那一份（文件头「判法」②），按文件路径导入。"""
    specification = importlib.util.spec_from_file_location("agent_watch_for_move_check", os.path.join(REPOSITORY_ROOT, "scripts", "agents", "agent-watch.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


AGENT_WATCH = load_agent_watch()


def forms_of(old, repository, must_exist=True):
    """→ (绝对写法, 仓根相对写法「<旧目录>/」)；旧目录既不在仓里也不是绝对路径交 None。搬完那一半（must_exist=False）旧路径可以已经不在。"""
    absolute = os.path.normpath(old if os.path.isabs(old) else os.path.join(repository, old))
    if not os.path.isabs(old) and must_exist and not os.path.exists(absolute):
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


def processes_using(absolute, relative, repository, command_line_only=False):
    """→ [(进程号, 怎么引的, 命令)]（文件头「判法」①；command_line_only 是「判法」③：只看命令行，旧路径这时可能是指到新目录的链，不按它解析）。"""
    if BROKEN == "processes-ignored":
        return []
    found = []
    real = absolute if command_line_only else os.path.realpath(absolute)
    lineage = own_lineage()
    for name in sorted((entry for entry in os.listdir("/proc") if entry.isdigit()), key=int):
        pid = int(name)
        if pid in lineage:
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
        if command_line_only:
            if reasons:
                found.append((pid, "；".join(reasons), command[:200]))
            continue
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


def default_projects_directory(explicit_repository, environment=None):
    """--projects-directory 不给时的会话记录目录（文件头「判法」②）：--repository 给了按那个仓拼会话目录名，没给取统一任务服务给的来源仓根 TASK_SERVER_SOURCE_REPOSITORY，
    再没有取这份脚本所在的仓；家目录从来源仓根的路径取（两台一致，不用 $HOME），没有来源仓根取本机家目录。取法只在 scripts/agents/lib_session_directory.py 里；
    取不到抛 session_directories.SessionDirectoryError（ValueError 的子类）。"""
    return session_directories.session_directory(REPOSITORY_ROOT, os.path.realpath(explicit_repository) if explicit_repository else None, environment)


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


def old_path_state(absolute, new):
    """旧路径此刻是什么（文件头「判法」③）：link 指到新目录的符号链接、absent 不在、other-link 指到别处的链、kept 还是一份真目录或文件。"""
    if os.path.islink(absolute):
        return "link" if os.path.realpath(absolute) == new else "other-link"
    return "kept" if os.path.lexists(absolute) else "absent"


def after_move(arguments, repository, absolute, relative, say):
    """搬完那一半（文件头「判法」③）：还有引着的要留链，都交回了要删链。"""
    new = os.path.realpath(arguments.moved_to if os.path.isabs(arguments.moved_to) else os.path.join(repository, arguments.moved_to))
    if not os.path.isdir(new):
        say(f"  ✗ --moved-to 的新目录「{arguments.moved_to}」不在（按 {new} 找）")
        say("  → 怎么办：写搬到的那个目录（仓根相对路径或绝对路径），搬完再跑这条")
        return 2
    processes = processes_using(absolute, relative, repository, command_line_only=True)
    agents = agents_using(absolute, relative, arguments.projects_directory, arguments.active_hours)
    for pid, how, command in processes:
        say(f"  · 进程 {pid}：{how}：{command}")
    for agent_id, kind, description, snippet in agents:
        say(f"  · 子 agent {agent_id} {kind}「{description}」还没交回，会话记录里提到旧路径：…{snippet}…")
    state = old_path_state(absolute, new)
    users = len(processes) + len(agents)
    link_text = os.path.relpath(new, os.path.dirname(absolute))
    if state == "kept":
        say(f"  ✗ 旧路径 {absolute} 还是一份真目录或文件，不是搬完的样子")
        say(f"  → 怎么办：还没搬的去掉 --moved-to 跑搬之前那一半；搬了一半的先 git mv 搬完（.claude/rules/path-moves.md「怎么做」），再跑这条")
        return 1
    if state == "other-link":
        say(f"  ✗ 旧路径 {absolute} 是符号链接，指到 {os.path.realpath(absolute)}，不是新目录 {new}")
        say(f"  → 怎么办：rm {absolute}（只删这条链）之后 ln -s {link_text} {absolute}，再跑这条")
        return 1
    if users and state == "absent" and BROKEN != "link-not-required":
        say(f"  ✗ 搬完了，旧路径 {absolute} 不在，而还有 {len(processes)} 个进程、{len(agents)} 个没交回的子 agent 引着它（上面逐个列了），它们会中途失效")
        say(f"  → 怎么办：ln -s {link_text} {absolute}（指到新目录的一次性符号链接，不 git add），到它们都交回再跑这条（.claude/rules/path-moves.md「怎么做」）")
        return 1
    if users and state == "link":
        # gate-lint:nocount 这一句不是成功句，退 3：数在上面逐个列出的进程与子 agent 里
        say(f"  ! 链 {absolute} → {new} 要留着：还有 {len(processes)} 个进程、{len(agents)} 个没交回的子 agent 引着旧路径；到它们都交回再跑一次这条（那时要删链）")
        return 3
    if not users and state == "link" and BROKEN != "leftover-link-ignored":
        say(f"  ✗ 旧路径上的链 {absolute} → {new} 还在，而已经没有进程、也没有 {arguments.active_hours:g} 小时内动过而没交回的子 agent 引着旧路径")
        say(f"  → 怎么办：rm {absolute}（只删这条链，不带 -r），再跑这条到绿")
        return 1
    say(f"  ✓ 搬迁收尾：旧路径 {absolute}（{'不在' if state == 'absent' else '链还在'}），查了 /proc 与会话记录，引着它的进程 {len(processes)} 个、没交回的子 agent {len(agents)} 个")
    return 0


def run(arguments, say=print):
    if BROKEN and BROKEN not in KNOWN_BREAKS:
        say(f"  ✗ MOVE_DIRECTORY_IN_USE_BREAK 的「{BROKEN}」不认识（只认 {'、'.join(KNOWN_BREAKS)}）")
        say(f"  → 怎么办：弄坏开关只给自证用，平时不设")
        return 2
    repository = os.path.realpath(arguments.repository or REPOSITORY_ROOT)
    forms = forms_of(arguments.old, repository, must_exist=arguments.moved_to is None)
    if forms is None:
        say(f"  ✗ 认不出旧目录「{arguments.old}」：它不在仓 {repository} 里，也不是绝对路径")
        say(f"  → 怎么办：写仓根相对路径（例 .claude/gate.d）或绝对路径")
        return 2
    absolute, relative = forms
    try:
        arguments.projects_directory = arguments.projects_directory or default_projects_directory(arguments.repository)
    except ValueError as error:   # SessionDirectoryError 是它的子类；弄坏开关的值认不出也是 ValueError
        say(f"  ✗ 取不到默认的会话记录目录：{error}")
        say(f"  → 怎么办：{getattr(error, 'howto', '去掉不认识的弄坏开关再跑')}；或给 --projects-directory")
        return 2
    if arguments.moved_to is not None:
        return after_move(arguments, repository, absolute, relative, say)
    processes = processes_using(absolute, relative, repository)
    agents = agents_using(absolute, relative, arguments.projects_directory, arguments.active_hours)
    for pid, how, command in processes:
        say(f"  ✗ 进程 {pid}：{how}：{command}")  # gate-lint:detail
    for agent_id, kind, description, snippet in agents:
        say(f"  ✗ 子 agent {agent_id} {kind}「{description}」还没交回，会话记录里提到它：…{snippet}…")  # gate-lint:detail
    if processes or agents:
        say(f"  ✗ 旧目录 {absolute}（仓根相对写法「{relative or '—'}」）还有 {len(processes)} 个进程、{len(agents)} 个没交回的子 agent 引着，现在搬会让它们中途失效")  # gate-lint:summary
        say("  → 怎么办：等这些进程退出、子 agent 交回再搬；等不了的，给在跑的子 agent 发消息改路径，搬完在旧路径留一个指到新路径的符号链接，"
            "到它们都交回再删，留没留、该不该删由同一条命令带 --moved-to <新目录> 判（.claude/rules/path-moves.md「怎么做」）；搬之前再跑一次这条到绿")
        return 1
    say(f"  ✓ 旧目录 {absolute} 没有进程、也没有 {arguments.active_hours:g} 小时内动过而没交回的子 agent 引着（查了 /proc 与会话记录；仓根相对写法「{relative or '—'}」）")
    return 0


def parse(argv):
    parser = argparse.ArgumentParser(description="整目录搬迁之前核还有没有在跑的子 agent 与进程引着旧路径")
    parser.add_argument("old")
    parser.add_argument("--repository")
    parser.add_argument("--projects-directory")
    parser.add_argument("--active-hours", type=float, default=24)
    parser.add_argument("--moved-to")
    return parser.parse_args(argv)


def selftest():
    """临时仓里放旧目录，起两个引着它的进程、一个不相干的；造会话记录：没交回而提到它的、交回了的、提到名字只是前缀相同的目录的。"""
    work = tempfile.mkdtemp(prefix="move-directory-in-use-selftest-")
    failures = []
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
        in_cwd = subprocess.Popen(["sleep", "60"], cwd=old, start_new_session=True)
        in_arguments = subprocess.Popen(["bash", "-c", "sleep 60; true", os.path.join(old, "x.sh")], cwd=work, start_new_session=True)   # 带 ; true：bash 不把自己 exec 成 sleep，命令行里留着那条路径
        unrelated = subprocess.Popen(["sleep", "61"], cwd=work, start_new_session=True)
        # 三条样本进程各在自己的进程组里（start_new_session=True）：判完逐个点名停整组、收尸（command-safety.md「装置起的样本进程，放进它自己的进程组」）
        try:
            time.sleep(0.3)
            said = []
            code = run(parse(["scripts/gate", "--repository", repository, "--projects-directory", projects]), say=said.append)
        finally:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(in_cwd.pid, signal.SIGKILL)
            with contextlib.suppress(ProcessLookupError):
                os.killpg(in_arguments.pid, signal.SIGKILL)
            with contextlib.suppress(ProcessLookupError):
                os.killpg(unrelated.pid, signal.SIGKILL)
            in_cwd.wait()
            in_arguments.wait()
            unrelated.wait()
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
        said = []
        code = run(parse([os.path.join(repository, "scripts", "gate-x"), "--repository", repository, "--projects-directory", os.path.join(work, "no-projects")]),
                   say=said.append)
        if code != 0 or "✓ 旧目录" not in "\n".join(said):
            failures.append(("没人引着就退 0", f"退 {code}：{said}"))

        # 搬完那一半（文件头「判法」③）：旧路径 scripts/hooks-old 已搬到 scripts/hooks-new；会话记录里有一个没交回、提到旧路径的子 agent
        moved_old = os.path.join(repository, "scripts", "hooks-old")
        moved_new = os.path.join(repository, "scripts", "hooks-new")
        os.makedirs(moved_new)
        moved_projects = os.path.join(work, "projects-moved")
        session = os.path.join(moved_projects, "session-b")
        transcript("runningmoved", "改 scripts/hooks-old/write-guard.sh", False)
        moved = ["scripts/hooks-old", "--moved-to", "scripts/hooks-new", "--repository", repository]

        said = []
        code = run(parse(moved + ["--projects-directory", moved_projects]), say=said.append)
        text = "\n".join(said)
        if code != 1 or "不在，而还有" not in text or "子 agent runningmoved " not in text:
            failures.append(("搬完还有人引着而没留链", f"退 {code}：{text}"))

        os.symlink("hooks-new", moved_old)
        in_new = subprocess.Popen(["sleep", "62"], cwd=moved_new, start_new_session=True)
        names_old = subprocess.Popen(["bash", "-c", "sleep 62; true", os.path.join(moved_old, "y.sh")], cwd=work, start_new_session=True)
        try:
            time.sleep(0.3)
            said = []
            code = run(parse(moved + ["--projects-directory", moved_projects]), say=said.append)
        finally:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(in_new.pid, signal.SIGKILL)
            with contextlib.suppress(ProcessLookupError):
                os.killpg(names_old.pid, signal.SIGKILL)
            in_new.wait()
            names_old.wait()
        text = "\n".join(said)
        if code != 3 or "要留着" not in text or f"进程 {names_old.pid}：" not in text or f"进程 {in_new.pid}：" in text:
            failures.append(("搬完还有人引着、链留着：退 3，只认命令行", f"退 {code}：{text}"))

        said = []
        code = run(parse(moved + ["--projects-directory", os.path.join(work, "no-projects")]), say=said.append)
        text = "\n".join(said)
        if code != 1 or "还在，而已经没有" not in text:
            failures.append(("都交回之后链还在", f"退 {code}：{text}"))

        os.remove(moved_old)
        said = []
        code = run(parse(moved + ["--projects-directory", os.path.join(work, "no-projects")]), say=said.append)
        if code != 0 or "✓ 搬迁收尾" not in "\n".join(said):
            failures.append(("都交回、链也删了就退 0", f"退 {code}：{said}"))

        said = []
        code = run(parse(["scripts/gate", "--moved-to", "scripts/hooks-new", "--repository", repository, "--projects-directory", os.path.join(work, "no-projects")]),
                   say=said.append)
        if code != 1 or "还是一份真目录或文件" not in "\n".join(said):
            failures.append(("搬完那一半旧路径还是真目录", f"退 {code}：{said}"))

        # 起这个检查的那条命令行（父进程 bash）写着旧路径的绝对写法：不算引着，没人引着、旧路径不在就退 0
        nested = subprocess.run(["bash", "-c", f"{sys.executable} {os.path.realpath(__file__)} {moved_old} --moved-to {moved_new} --repository {repository} "
                                 f"--projects-directory {os.path.join(work, 'no-projects')}; exit $?"], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        if nested.returncode != 0 or "✓ 搬迁收尾" not in nested.stdout:
            failures.append(("起这个检查的那条命令行写着旧路径不算", f"退 {nested.returncode}：{nested.stdout}"))

        # 没给 --projects-directory：默认会话记录目录取来源仓根 TASK_SERVER_SOURCE_REPOSITORY 的，HOME 指到别处、这份脚本所在的仓在别处也取对（--repository 给了按它拼目录名、家目录仍从来源仓根取）；
        # 来源仓会话目录里放一个没交回、提到旧目录的子 agent，整条 run() 不给 --projects-directory 也要列出它（弄坏开关 SESSION_DIRECTORY_BREAK=home-from-environment 下这一格红）
        tree = session_directories.build_selftest_tree(os.path.join(work, "default-directory-tree"))
        got_default = default_projects_directory(None, tree["environment"])
        got_explicit = default_projects_directory(repository, tree["environment"])
        want_explicit = os.path.join(tree["projects_root"], os.path.realpath(repository).replace("/", "-"))
        problems = []
        if got_default != tree["session_directory"] or got_explicit != want_explicit:
            problems.append(f"没给 --repository 应当 {tree['session_directory']}，实际 {got_default}；给了 --repository 应当 {want_explicit}，实际 {got_explicit}")
        session = os.path.join(tree["session_directory"], "selftest-session")
        transcript("runningviatree", f"改 {old}/apply-writer-patch.py", False)
        patched = {"HOME": tree["other_home"], session_directories.SOURCE_REPOSITORY_VARIABLE: tree["source_repository"]}
        saved = {name: os.environ.get(name) for name in patched}
        os.environ.update(patched)
        said = []
        try:
            code = run(parse([old]), say=said.append)
        finally:
            for name, value in saved.items():
                if value is None:
                    os.environ.pop(name, None)
                else:
                    os.environ[name] = value
        if code != 1 or "子 agent runningviatree " not in "\n".join(said):
            problems.append(f"run() 不给 --projects-directory 应当在来源仓的会话目录里找到没交回的 runningviatree 并退 1，实际退 {code}：{said}")
        if problems:
            failures.append(("没给 --projects-directory 时取来源仓根的会话目录", "；".join(problems)))
    finally:
        shutil.rmtree(work, ignore_errors=True)
    cells = ("进程的当前目录与命令行引着旧目录", "没交回的子 agent 提到旧目录", "交回了的不算", "名字只是前缀相同的不算", "有引着的就退 1", "没人引着就退 0",
             "搬完还有人引着而没留链", "搬完还有人引着、链留着：退 3，只认命令行", "都交回之后链还在", "都交回、链也删了就退 0", "搬完那一半旧路径还是真目录",
             "起这个检查的那条命令行写着旧路径不算", "没给 --projects-directory 时取来源仓根的会话目录")
    for cell, failure in failures:
        print(f"  ✗ 自证 {cell}：{failure}")  # gate-lint:detail
    if failures:
        print(f"  ✗ 自证 {len(cells)} 格里 {len(failures)} 格判错")  # gate-lint:summary
        print("  → 怎么办：看 processes_using / agents_using / mentions / after_move 的判法；MOVE_DIRECTORY_IN_USE_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ move-directory-in-use-check 自证通过（{len(cells)} 格）：当前目录在旧目录里与命令行带它的进程、没交回而提到它的子 agent 都报，"
          "不相干的进程、交回了的、名字只是前缀相同的不报；有就退 1，没有退 0；搬完还有人引着而没留链、都交回之后链还在判红，链留着等交回退 3")
    return 0


BREAK_VARIABLES = ("MOVE_DIRECTORY_IN_USE_BREAK", "SESSION_DIRECTORY_BREAK")   # 后一个经 lib_session_directory.py 进来


def inside_selftest():
    """往上数这个进程的祖先，有一个的命令行里带 --selftest（自证起的子进程，含别的脚本的自证起它的）就是 True。"""
    pid = os.getppid()
    while pid > 1:
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as handle:
                if b"--selftest" in handle.read().split(b"\0"):
                    return True
            with open(f"/proc/{pid}/stat", "rb") as handle:
                pid = int(handle.read().rsplit(b")", 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            return False
    return False


def settle_break_switches(arguments):
    """弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道。
    放行：命令行以 --selftest 起头；祖先里有自证（inside_selftest）。
    都不是而 BREAK_VARIABLES 里有设了的：点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    if not setting or arguments[:1] == ["--selftest"] or inside_selftest():
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 {setting[0]}=<项> python3 scripts/kb/move-directory-in-use-check.py --selftest", file=sys.stderr)
    return 2


if __name__ == "__main__":
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    sys.exit(run(parse(sys.argv[1:])))
