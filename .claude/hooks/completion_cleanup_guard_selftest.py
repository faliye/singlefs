# 完工清理闸 completion-cleanup-guard.sh 的 python 部分之一：自证（Scenario、selftest_cells、run_switch、selftest）与 main。
# 入口 completion-cleanup-guard.sh 的引导先静态核每一份（读得到、编得过、顶层绑着入口要的名字），再按 COMPLETION_CLEANUP_PART_NAMES 的次序 import 它们；每一份是普通 python 模块，用到别份的名字写显式 import（不留环），不单独执行；
# 弄坏开关的名单、读开关的代码在这几份里，开关表与各开关红在哪几格写在入口的文件头。
import concurrent.futures
import io
import json
import os
import re
import shlex
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from __main__ import COMPLETION_CLEANUP_PART_NAMES, SELFTEST_MARK_CONTENT, SELFTEST_MARK_VARIABLE, UNJUDGED_SKIP_PREFIX
from completion_cleanup_guard_decisions import (
    ARGUMENTS, BASELINE_KIND, BREAK_TABLE, BREAK_VARIABLE, CLAUDE_PID_VARIABLE, CRASH_AMPLIFICATION_DIRECTORY_NAME,
    CRASH_AMPLIFICATION_HOME_VARIABLE, DECLARATION_FILE, DELIVERABLE_KINDS, DETECTIONS_VARIABLE, GUARD_OFF_ITEM, HANDBACK_TOOL, HAND_TO_MAIN_LINE,
    HAND_TO_MAIN_REASON_MINIMUM, HOOK_DIR, HOOK_PATH, KEEP_SCENE_VALUE, KEEP_SCENE_VARIABLE, KIND_BUILD, KIND_KEPT_SCENE, KIND_LIBRARY,
    KIND_MERGE_SNAPSHOT, MERGE_SNAPSHOT_DIRECTORY_NAME,
    MERGE_SNAPSHOT_NAME, NOT_RUN, REPOSITORY_ROOT, RUN_OWNER_FILE, SCAN_LIMIT_VARIABLE, SCRATCHPAD_ROOT_VARIABLE, SESSION_VARIABLES, SPECIES_SOURCES,
    SPECIES_TOKEN, TASK_SERVER_CLIENT, TASK_SERVER_DATABASE_VARIABLE, UNKNOWN_SECTION_TITLE, broken, declarations, hook_main, module,
    refuse_break_outside_selftest, report_unjudged_pass, unknown_lines,
)
import completion_cleanup_guard_decisions

# ── 自证：每一格造一个答案已知的局面，在本进程里调 decide（包装那几格起钩子本身）
def iso(seconds):
    return datetime.fromtimestamp(seconds, timezone.utc).isoformat().replace("+00:00", "Z")


class Scenario:
    """一格一个：临时目录里的工作根、崩溃放量的家目录、会话临时目录根、项目目录与主会话记录。"""

    def __init__(self, base, name):
        self.root = os.path.join(base, name)
        self.work = os.path.join(self.root, "work")
        self.home = os.path.join(self.root, "crash-home")
        self.runs = os.path.join(self.home, "runs")
        self.scratch = os.path.join(self.root, "session-temporary-root")
        self.project = os.path.join(self.root, "projects", "project-directory")
        self.session = "s-" + re.sub(r"[^A-Za-z0-9_-]", "-", name)
        self.main_transcript = os.path.join(self.project, self.session + ".jsonl")
        self.data = os.path.join(self.root, "data-root")             # 数据根：与工作根分开，迁库遗留那几格往里放
        self.prune_control = os.path.join(self.root, "prune-control")   # prune-server/control 的自证根：状态目录与单元文件都在它底下
        self.service_database_path = os.path.join(self.root, "task-server.sqlite3")   # 不建就是服务没装过：不判任务现场
        self.processes = []
        self.call_count = 0
        for directory in (self.work, self.runs, self.scratch, self.project, self.data):
            os.makedirs(directory, exist_ok=True)
        self.now = time.time()
        self.write(self.main_transcript, [self.user_text(self.now - 100, "开工")])

    def environment(self):
        return {"SINGLEFS_WORK_ROOT": self.work, CRASH_AMPLIFICATION_HOME_VARIABLE: self.home, SCRATCHPAD_ROOT_VARIABLE: self.scratch,
                DETECTIONS_VARIABLE: os.path.join(self.root, "detections.jsonl"), TASK_SERVER_DATABASE_VARIABLE: self.service_database_path,
                module("lib_work_root").DATA_ROOT_VARIABLE: self.data, module("prune_server_control").SELFTEST_ROOT_VARIABLE: self.prune_control}

    def prune_paths(self):
        """prune-server/control 在这一格的自证根底下取的路径（状态目录、单元文件、二进制）。"""
        control = module("prune_server_control")
        saved = os.environ.get(control.SELFTEST_ROOT_VARIABLE)
        os.environ[control.SELFTEST_ROOT_VARIABLE] = self.prune_control
        try:
            return control.Paths()
        finally:
            if saved is None:
                os.environ.pop(control.SELFTEST_ROOT_VARIABLE, None)
            else:
                os.environ[control.SELFTEST_ROOT_VARIABLE] = saved

    def bash(self, command):
        """主会话记录里记一次做完了的 Bash 调用。"""
        self.call_count += 1
        self.write(self.main_transcript, self.bash_call(f"toolu_leftover_{self.call_count}", self.now - 40, self.now - 39, command), append=True)

    @staticmethod
    def filled(path):
        """一个不空的目录（旧文件库的样子：里面一份键文件）。"""
        os.makedirs(path, exist_ok=True)
        with open(os.path.join(path, "keys.json"), "w", encoding="utf-8") as handle:
            handle.write("{}\n")
        return path

    @staticmethod
    def archive(path):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "wb") as handle:
            handle.write(b"x" * 512)
        return path

    @staticmethod
    def write(path, records, append=False):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "a" if append else "w", encoding="utf-8") as handle:
            for record in records:
                handle.write(json.dumps(record, ensure_ascii=False) + "\n")

    @staticmethod
    def user_text(seconds, text):
        return {"type": "user", "timestamp": iso(seconds), "message": {"role": "user", "content": text}}

    @staticmethod
    def bash_call(call_id, started, ended, command, result="完成", background=False):
        tool_input = {"command": command}
        if background:
            tool_input["run_in_background"] = True
        records = [{"type": "assistant", "timestamp": iso(started),
                    "message": {"role": "assistant", "content": [{"type": "tool_use", "id": call_id, "name": "Bash", "input": tool_input}]}}]
        if ended is not None:
            records.append({"type": "user", "timestamp": iso(ended),
                            "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": call_id, "content": result}]}})
        return records

    @staticmethod
    def notification(seconds, task_id, status="completed"):
        return {"type": "user", "timestamp": iso(seconds), "origin": {"kind": "task-notification"},
                "message": {"role": "user", "content": f"<task-notification>\n<task-id>{task_id}</task-id>\n<status>{status}</status>\n</task-notification>"}}

    def owned(self, name, session=None):
        path = os.path.join(self.work, name)
        os.makedirs(path, exist_ok=True)
        with open(os.path.join(path, ".owner"), "w", encoding="utf-8") as handle:
            handle.write(f"session: {session or self.session}\ncreated: 2000-01-01\npurpose: 自证\n")
        return path

    @staticmethod
    def build_directory(path):
        os.makedirs(os.path.join(path, "release", "deps"), exist_ok=True)
        with open(os.path.join(path, "CACHEDIR.TAG"), "w") as handle:
            handle.write("Signature: 8a477f597d28d172789f06886806bc55\n")
        with open(os.path.join(path, "release", "deps", "libx.rlib"), "wb") as handle:
            handle.write(b"x" * 4096)
        return path

    @staticmethod
    def repository_copy(path):
        os.makedirs(os.path.join(path, ".git"), exist_ok=True)
        os.makedirs(os.path.join(path, "crates", "core"), exist_ok=True)
        open(os.path.join(path, "Cargo.toml"), "w").close()
        return path

    @staticmethod
    def library_copy(path):
        os.makedirs(path, exist_ok=True)
        for name in ("CURRENT", "MANIFEST-000004", "000005.sst"):
            open(os.path.join(path, name), "w").close()
        return path

    def service_task(self, task_id, state, session=None, parent_id=None, keep=True, scene=True, collected=False):
        """样本服务库里记一个任务与它的一次尝试（表与列照 task-server/task_server_store.py 的 Store；任务记录的 environment 是提交时带的环境、collected 是结局取没取走、
        尝试记录的 scene_directory 是现场目录，task-server/task_server_lib.py 文件头「现场」）。keep 为真时任务的环境里设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1
        （分片的子任务照父任务，样本里不另设）；collected 为真时结局已被 task wait 取走；scene 为真时现场目录建在盘上、里面放一份文件。
        尝试行的会话列照服务写的形状写空串（task-server/task_server_scheduler_starting.py 起尝试时记录里的 session 是空串，库照记录写这一列）。交现场目录的路径。"""
        connection = sqlite3.connect(self.service_database_path)
        try:
            for table in ("tasks", "attempts"):
                connection.execute(f"CREATE TABLE IF NOT EXISTS {table}(id INTEGER PRIMARY KEY AUTOINCREMENT, task_id INTEGER NOT NULL DEFAULT 0, state TEXT NOT NULL, "
                                   "session TEXT NOT NULL, command_key TEXT NOT NULL, record TEXT NOT NULL, signature TEXT NOT NULL)")
            owner_session = session or self.session
            environment = {KEEP_SCENE_VARIABLE: KEEP_SCENE_VALUE} if keep and parent_id is None else {}
            record = {"id": task_id, "state": state, "session": owner_session, "environment": environment, "collected": collected}
            if parent_id is not None:
                record["parent_id"] = parent_id
            connection.execute("INSERT INTO tasks(id, task_id, state, session, command_key, record, signature) VALUES(?, ?, ?, ?, '', ?, '')",
                               (task_id, parent_id or 0, state, owner_session, json.dumps(record)))
            directory = os.path.join(self.root, "task-server-scenes", f"task-sample-{task_id}-1")   # 不放进这一格的工作根：迁库遗留那一扫不进它
            connection.execute("INSERT INTO attempts(task_id, state, session, command_key, record, signature) VALUES(?, 'finished', '', '', ?, '')",
                               (task_id, json.dumps({"task_id": task_id, "session": "", "scene_directory": directory})))
            connection.commit()
        finally:
            connection.close()
        if scene:
            os.makedirs(directory, exist_ok=True)
            with open(os.path.join(directory, "panic.txt"), "w", encoding="utf-8") as handle:
                handle.write("判红那一趟留下的临时文件\n")
        return directory

    def run_directory(self, pid, suffix="", owner_session=None):
        """运行目录根下一层：名字照入口脚本的「<前缀>-<时刻>-<进程号>[-<后缀>]」；给了 owner_session 的写归属记录。"""
        path = os.path.join(self.runs, f"crash-amplification-{time.strftime('%Y%m%dT%H%M%SZ', time.gmtime())}-{pid}{suffix}")
        os.makedirs(path)
        if owner_session:
            with open(os.path.join(path, RUN_OWNER_FILE), "w", encoding="utf-8") as handle:
                handle.write(f"session: {owner_session}\ncreated: 2000-01-01\npurpose: 自证的运行目录\n")
        return path

    @staticmethod
    def dead_pid():
        """比 pid_max 大的进程号：一定不在。"""
        try:
            return int(open("/proc/sys/kernel/pid_max").read()) + 1
        except (OSError, ValueError):
            return 99999999

    def creator(self, session, launcher_command=None):
        """起运行目录的假入口：环境里带会话号与 CLAUDE_PID（本进程当 Claude 进程）。给了 launcher_command 的照 Bash 工具那样起
        bash -c 'true && eval <命令> && true'，命令用内建的 read 等在管道上（不起子进程，停它只停这一个进程号）。"""
        environment = {key: value for key, value in os.environ.items() if key not in SESSION_VARIABLES}
        environment.update({"CLAUDE_CODE_SESSION_ID": session, CLAUDE_PID_VARIABLE: str(os.getpid())})
        arguments = ["sleep", "120"] if launcher_command is None else ["bash", "-c", f"true && eval {shlex.quote(launcher_command)} && true"]
        process = subprocess.Popen(arguments, stdin=subprocess.PIPE, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=environment)
        self.processes.append(process)
        return process

    def end(self, process):
        process.kill()
        process.wait()
        self.processes.remove(process)

    def subagent(self, agent_id, prompt, records=()):
        path = os.path.join(self.project, self.session, "subagents", f"agent-{agent_id}.jsonl")
        self.write(path, [self.user_text(self.now - 90, prompt), *records])
        return path

    def hold(self, arguments, **options):
        process = subprocess.Popen(arguments, stdin=subprocess.DEVNULL, stderr=subprocess.DEVNULL, **options)
        self.processes.append(process)
        return process

    def background_task(self, task_id, command_arguments):
        """主会话记录里一条还没完成通知的后台任务，它的输出文件由一个活着的进程开着。"""
        output = os.path.join(self.root, "tasks", f"{task_id}.output")
        os.makedirs(os.path.dirname(output), exist_ok=True)
        handle = open(output, "w")
        process = self.hold(command_arguments, stdout=handle)
        handle.close()
        # 等它开着输出文件、exec 换好进程名（bash -c 'exec -a …' 那种，换好之前命令行还是 bash -c …）
        for _ in range(250):
            try:
                holding = os.path.realpath(os.readlink(f"/proc/{process.pid}/fd/1")) == os.path.realpath(output)
                renamed = command_arguments[0] != "bash" or not open(f"/proc/{process.pid}/cmdline", "rb").read().startswith(b"bash")
                if holding and renamed:
                    break
            except OSError:
                pass
            time.sleep(0.02)
        self.write(self.main_transcript, self.bash_call(f"toolu_{task_id}", self.now - 50, self.now - 49, "bash x.sh",
                                                        f"Command running in background with ID: {task_id}. Output is being written to: {output}",
                                                        background=True), append=True)

    def stop(self):
        for process in self.processes:
            process.kill()
            process.wait()
        self.processes = []

    def decide(self, hook_input, extra=None):
        environment = dict(self.environment(), **(extra or {}))
        saved = {key: os.environ.get(key) for key in environment}
        os.environ.update(environment)
        try:
            return completion_cleanup_guard_decisions.decide(hook_input)
        finally:
            for key, value in saved.items():
                if value is None:
                    os.environ.pop(key, None)
                else:
                    os.environ[key] = value

    def stop_input(self, active=False):
        return {"hook_event_name": "Stop", "session_id": self.session, "transcript_path": self.main_transcript, "stop_hook_active": active}

    def subagent_stop_input(self, agent_id, path, active=False):
        return {"hook_event_name": "SubagentStop", "session_id": self.session, "transcript_path": self.main_transcript,
                "agent_id": agent_id, "agent_transcript_path": path, "stop_hook_active": active}

    def handback_input(self, agent_id, message="交回"):
        return {"hook_event_name": "PreToolUse", "tool_name": HANDBACK_TOOL, "session_id": self.session,
                "transcript_path": self.main_transcript, "agent_id": agent_id, "tool_input": {"message": message}}

    @staticmethod
    def handback_call(call_id, started, message, rejected=False):
        """会话记录里交回工具的一次调用与它的结果（rejected 为真是被交回闸拒掉的那一次）。"""
        result = {"type": "tool_result", "tool_use_id": call_id, "content": "完成"}
        if rejected:
            result["is_error"] = True
        return [{"type": "assistant", "timestamp": iso(started),
                 "message": {"role": "assistant", "content": [{"type": "tool_use", "id": call_id, "name": HANDBACK_TOOL, "input": {"message": message}}]}},
                {"type": "user", "timestamp": iso(started + 1), "message": {"role": "user", "content": [result]}}]

    @staticmethod
    def reply(seconds, text):
        """会话记录里 agent 回复的一段文字。"""
        return {"type": "assistant", "timestamp": iso(seconds), "message": {"role": "assistant", "content": [{"type": "text", "text": text}]}}


def selftest_cells():
    base = tempfile.mkdtemp(prefix="completion-cleanup-selftest-")
    failures, checked = [], [0]

    def check(cell, ok, detail):
        checked[0] += 1
        if not ok:
            failures.append(cell)
            print(f"✗ 自证 {cell}：{detail}")
            print(f"→ 怎么办：看 completion-cleanup-guard.sh 里 {cell} 对应的判定；{BREAK_VARIABLE} 设着的话这里本来就该红")

    def blocked_with(outcome, *paths):
        return outcome.get("action") == "block" and all(path in outcome.get("text", "") for path in paths)

    def detection_entries(path):
        entries = []
        if os.path.exists(path):
            with open(path, encoding="utf-8") as handle:
                for line in handle:
                    try:
                        entries.append(json.loads(line))
                    except ValueError:
                        entries.append({"坏行": line})
        return entries

    scenarios = []

    def scenario(name):
        made = Scenario(base, name)
        scenarios.append(made)
        return made

    try:
        # 主 agent：认领目录里留了编译目录
        s = scenario("main-build")
        target = s.build_directory(os.path.join(s.owned("w-build"), "copy", "target"))
        outcome = s.decide(s.stop_input())
        check("main-build-directory-left", blocked_with(outcome, target), f"留了编译目录 {target} 应当拦下并列出，实际 {outcome}")
        # 主 agent：认领目录里留了仓副本
        s = scenario("main-repository")
        copy = s.repository_copy(os.path.join(s.owned("w-copy"), "repository"))
        outcome = s.decide(s.stop_input())
        check("main-repository-copy-left", blocked_with(outcome, copy, f"rm -rf -- {copy}"), f"留了仓副本 {copy} 应当拦下、列出删法，实际 {outcome}")
        # 主 agent：归属记录是本会话的崩溃放量运行目录里留了库拷贝（起它的进程已经不在）
        s = scenario("main-run")
        library = s.library_copy(os.path.join(s.run_directory(s.dead_pid(), "-two-hosts", owner_session=s.session), "library-copy"))
        outcome = s.decide(s.stop_input())
        check("main-run-library-copy-left", blocked_with(outcome, library), f"运行目录里留了库拷贝 {library} 应当拦下，实际 {outcome}")
        # 没设 SINGLEFS_CRASH_AMPLIFICATION_HOME：运行目录根是工作根下的 crash-amplification/runs，那里本会话的运行目录留了库拷贝照拦
        s = scenario("run-root-default")
        s.runs = os.path.join(s.work, CRASH_AMPLIFICATION_DIRECTORY_NAME, "runs")
        library = s.library_copy(os.path.join(s.run_directory(s.dead_pid(), owner_session=s.session), "library-copy"))
        outcome = s.decide(s.stop_input(), {CRASH_AMPLIFICATION_HOME_VARIABLE: ""})
        check("run-root-default-under-work-root", blocked_with(outcome, library),
              f"没设 {CRASH_AMPLIFICATION_HOME_VARIABLE} 时工作根下 crash-amplification/runs 里本会话的运行目录留了库拷贝 {library} 应当拦下，实际 {outcome}")
        # 本会话的进程起的运行目录：起它的进程还在时整个不列（那一趟还在跑），看到一次就记账；进程退了照记账拦
        s = scenario("own-run")
        process = s.creator(s.session)
        run = s.run_directory(process.pid)
        library = s.library_copy(os.path.join(run, "library-copy"))
        target = s.build_directory(os.path.join(run, "target"))
        running = s.decide(s.stop_input())
        check("run-in-progress-not-listed", running.get("action") == "pass" and library not in running.get("text", ""),
              f"起运行目录的进程 {process.pid} 还在（那一趟还在跑），里面的 {library} 不该列，实际 {running}")
        s.end(process)
        outcome = s.decide(s.stop_input())
        check("own-run-blocked-after-creator-exits", blocked_with(outcome, library, target),
              f"本会话的进程起的运行目录（看到它还在跑时记过账），进程退了之后留下的 {library}、{target} 应当拦下，实际 {outcome}")
        # 别的会话的进程起的运行目录，创建时间落在本会话的 Bash 调用时间窗里：不拦（记账认的、归属记录认的各一份）
        s = scenario("foreign-session-run")
        s.write(s.main_transcript, s.bash_call("toolu_run", s.now - 30, s.now + 600, "bash scripts/crash/crash-amplification.sh --item x"), append=True)
        process = s.creator("s-somebody-else")
        seen = s.library_copy(os.path.join(s.run_directory(process.pid), "library-copy"))
        s.decide(s.stop_input())
        s.end(process)
        recorded = s.library_copy(os.path.join(s.run_directory(s.dead_pid(), "-two-hosts", owner_session="s-somebody-else"), "library-copy"))
        outcome = s.decide(s.stop_input())
        check("foreign-session-run-in-window-not-listed",
              outcome.get("action") == "pass" and not any(path in str(outcome) for path in (seen, recorded)),
              f"别的会话的进程起的运行目录（{seen}、{recorded}）时间窗与本会话重叠也不该拦、不该报，实际 {outcome}")
        # 认不出是谁的（进程不在、没记账、没归属记录）、建在本会话 Bash 调用的时间窗里：只在拦下时另起一节报，不进单子
        s = scenario("unknown-run")
        s.write(s.main_transcript, s.bash_call("toolu_run", s.now - 30, s.now + 600, "bash scripts/crash/crash-amplification.sh --item x"), append=True)
        unknown = s.library_copy(os.path.join(s.run_directory(s.dead_pid()), "library-copy"))
        alone = s.decide(s.stop_input())
        target = s.build_directory(os.path.join(s.owned("w-unknown"), "target"))
        outcome = s.decide(s.stop_input())
        text = outcome.get("text", "")
        check("unknown-run-reported-not-blocked",
              alone.get("action") == "pass" and blocked_with(outcome, target) and unknown not in outcome.get("paths", [])
              and UNKNOWN_SECTION_TITLE in text and text.index(UNKNOWN_SECTION_TITLE) < text.index(unknown),
              f"认不出是谁的 {unknown} 不该拦，只在拦下时列进「{UNKNOWN_SECTION_TITLE}」一节，实际单独时 {alone}，有别的要拦时 {outcome}")
        # 运行目录里深处有文件被开着（那一趟还在写日志），库拷贝本身没人开着：整个运行目录不列
        s = scenario("run-held-deep")
        run = s.run_directory(s.dead_pid(), owner_session=s.session)
        library = s.library_copy(os.path.join(run, "library-copy"))
        os.makedirs(os.path.join(run, "logs", "deeper"))
        with open(os.path.join(run, "logs", "deeper", "share.log"), "w") as handle:
            s.hold(["sleep", "120"], stdout=handle)
        outcome = s.decide(s.stop_input())
        check("run-held-deep-not-listed", outcome.get("action") == "pass" and library not in outcome.get("text", ""),
              f"运行目录 {run} 深处的日志有进程开着，里面的 {library} 不该列，实际 {outcome}")
        s.stop()
        # 只经内存映射开着（文件描述符已关）的编译目录不列（run_build_directory.py 的判法看 /proc/<pid>/maps）
        s = scenario("mapped")
        mapped = s.build_directory(os.path.join(s.owned("w-mapped"), "target"))
        mapped_file = os.path.join(mapped, "release", "deps", "libx.rlib")
        ready = os.path.join(s.root, "mapped.ready")
        s.hold([sys.executable, "-c", "import mmap, sys, time\nhandle = open(sys.argv[1], 'rb')\nmapping = mmap.mmap(handle.fileno(), 0, access=mmap.ACCESS_READ)\n"
                "handle.close()\nopen(sys.argv[2], 'w').close()\ntime.sleep(120)\n", mapped_file, ready], stdout=subprocess.DEVNULL)
        for _ in range(250):
            if os.path.exists(ready):
                break
            time.sleep(0.02)
        outcome = s.decide(s.stop_input())
        check("mapped-copy-not-listed", os.path.exists(ready) and outcome.get("action") == "pass",
              f"编译目录 {mapped} 里的文件只经内存映射开着也算在用，不该列，实际 {outcome}")
        s.stop()
        # 子 agent：只认它自己的 Bash 调用起的运行目录；同一会话别的调用（主 agent、兄弟子 agent）起的不算它的
        s = scenario("subagent-run")
        mine_command, sibling_command = ": crash-amplification mine; read -r _", ": crash-amplification sibling; read -r _"
        agent = s.subagent("arun", "派发", s.bash_call("toolu_mine", s.now - 20, s.now + 600, mine_command))
        mine_process, sibling_process = s.creator(s.session, mine_command), s.creator(s.session, sibling_command)
        mine = s.library_copy(os.path.join(s.run_directory(mine_process.pid), "library-copy"))
        sibling = s.library_copy(os.path.join(s.run_directory(sibling_process.pid), "library-copy"))
        s.decide(s.subagent_stop_input("arun", agent))
        s.end(mine_process)
        s.end(sibling_process)
        outcome = s.decide(s.subagent_stop_input("arun", agent))
        check("subagent-own-run-left", blocked_with(outcome, mine), f"子 agent 自己的 Bash 调用起的运行目录里的 {mine} 应当拦下，实际 {outcome}")
        check("subagent-ignores-sibling-runs", sibling not in outcome.get("text", ""), f"同一会话别的调用起的 {sibling} 不该算到这个子 agent 头上，实际 {outcome}")
        # 任务现场（文件头「判哪几处」任务现场那一段）：本会话已有结局、现场目录还在，又占着服务的两个不到期条件之一（结局没取走；设了留现场）的拦下；
        # 设了留现场、结局也没取走的删法是先 task-server/task wait <号>、再 task-server/task clean <号>（clean 不算取走结局；分片的写父任务号，不写 rm）
        s = scenario("kept-scene")
        finished = s.service_task(7, "finished")
        child = s.service_task(8, "finished", parent_id=7)
        removed = s.service_task(9, "finished", scene=False)
        outcome = s.decide(s.stop_input())
        both_way = f"先 {TASK_SERVER_CLIENT} wait 7，再 {TASK_SERVER_CLIENT} clean 7"
        lone_commands = [line.strip() for line in outcome.get("text", "").splitlines() if line.strip() in (f"{TASK_SERVER_CLIENT} wait 7", f"{TASK_SERVER_CLIENT} clean 7")]
        check("main-kept-scene-left", blocked_with(outcome, finished, child, both_way, KIND_KEPT_SCENE) and not lone_commands
              and f"rm -rf -- {finished}" not in outcome.get("text", "") and f"{TASK_SERVER_CLIENT} clean 8" not in outcome.get("text", "")
              and removed not in outcome.get("text", ""),
              f"本会话已有结局、设了 {KEEP_SCENE_VARIABLE}={KEEP_SCENE_VALUE}、结局也没取走的任务 7（连同分片子任务 8）的现场 {finished}、{child} 应当按「{KIND_KEPT_SCENE}」拦下、"
              f"删法写「{both_way}」（取走结局删不掉它，clean 不算取走结局），不单列 wait 7 或 clean 7，盘上已经没了的 {removed} 不列，实际 {outcome}")
        # 设了留现场、结局已取走的：服务不删（scene_due 的另一个不到期条件），拦下，删法只写 task-server/task clean <号>
        s = scenario("kept-collected-scene")
        kept_collected = s.service_task(17, "finished", collected=True)
        outcome = s.decide(s.stop_input())
        check("kept-collected-scene-left", blocked_with(outcome, kept_collected, f"{TASK_SERVER_CLIENT} clean 17", KIND_KEPT_SCENE)
              and f"{TASK_SERVER_CLIENT} wait 17" not in outcome.get("text", "") and f"rm -rf -- {kept_collected}" not in outcome.get("text", ""),
              f"本会话已有结局、设了 {KEEP_SCENE_VARIABLE}={KEEP_SCENE_VALUE}、结局已取走的任务 17 的现场 {kept_collected} 应当按「{KIND_KEPT_SCENE}」拦下、"
              f"删法只写 {TASK_SERVER_CLIENT} clean 17（结局已取走，不写 wait 17），实际 {outcome}")
        s = scenario("unkept-scene")
        unkept = s.service_task(10, "finished", keep=False, collected=True)
        outcome = s.decide(s.stop_input())
        check("unkept-scene-not-listed", outcome.get("action") == "pass" and unkept not in str(outcome),
              f"提交时没设 {KEEP_SCENE_VARIABLE}、结局已经取走的任务的现场 {unkept} 不该列（服务在取走结局之后过一小时自己删），实际 {outcome}")
        # 没设留现场、结局没取走的：服务不删（scene_due 的另一个不到期条件），拦下，删法是 task-server/task wait <号>（取走结局之后服务过一小时自己删）
        s = scenario("uncollected-scene")
        uncollected = s.service_task(16, "finished", keep=False)
        outcome = s.decide(s.stop_input())
        check("uncollected-scene-left", blocked_with(outcome, uncollected, f"{TASK_SERVER_CLIENT} wait 16", KIND_KEPT_SCENE)
              and f"{TASK_SERVER_CLIENT} clean 16" not in outcome.get("text", "") and f"rm -rf -- {uncollected}" not in outcome.get("text", ""),
              f"本会话已有结局、没设 {KEEP_SCENE_VARIABLE}、结局没取走的任务 16 的现场 {uncollected} 应当按「{KIND_KEPT_SCENE}」拦下、"
              f"删法写 {TASK_SERVER_CLIENT} wait 16（取走结局，服务过一小时自己删），实际 {outcome}")
        s = scenario("kept-scene-running")
        running = s.service_task(11, "running")
        queued = s.service_task(12, "queued")
        outcome = s.decide(s.stop_input())
        check("running-kept-scene-not-listed", outcome.get("action") == "pass" and running not in str(outcome) and queued not in str(outcome),
              f"在跑、在排的任务的现场（{running}、{queued}）不该列，实际 {outcome}")
        s = scenario("kept-scene-other-session")
        theirs = s.service_task(13, "finished", session="s-somebody-else")
        outcome = s.decide(s.stop_input())
        check("other-session-kept-scene-ignored", outcome.get("action") == "pass" and theirs not in str(outcome),
              f"别的会话提交的任务的现场 {theirs} 不该列，实际 {outcome}")
        # 子 agent 不交任务、不起 task clean：交回时本会话已有结局、设了留现场或结局没取走的任务的现场一个都不列（归主 agent 收工时判）
        s = scenario("kept-scene-subagent")
        s.subagent("ascene", "派发", s.bash_call("toolu_status", s.now - 20, s.now - 19, "task-server/task status 14", "task 14 finished"))
        session_scene = s.service_task(14, "finished")
        outcome = s.decide(s.handback_input("ascene"))
        check("subagent-kept-scenes-not-listed", outcome.get("action") == "pass" and session_scene not in str(outcome),
              f"子 agent 交回时本会话任务 14 的现场 {session_scene} 不该列（子 agent 不交任务、不起 {TASK_SERVER_CLIENT} clean），实际 {outcome}")
        # 别的会话认领的目录不判
        s = scenario("other-session")
        s.build_directory(os.path.join(s.owned("w-theirs", session="s-somebody-else"), "target"))
        outcome = s.decide(s.stop_input())
        check("other-session-directory-ignored", outcome.get("action") == "pass", f"别的会话认领的目录不该判，实际 {outcome}")
        # 只剩交付物：报告、补丁、声明成备份的副本
        s = scenario("deliverables")
        directory = s.owned("w-deliver")
        open(os.path.join(directory, "result.md"), "w").close()
        os.makedirs(os.path.join(directory, "patch"))
        open(os.path.join(directory, "patch", "crates.patch"), "w").close()
        s.repository_copy(os.path.join(directory, "before-patch"))
        with open(os.path.join(directory, DECLARATION_FILE), "w", encoding="utf-8") as handle:
            handle.write("# 交付物\nbefore-patch 备份 打补丁之前的原样\nbroken-line 留着\n")
        outcome = s.decide(s.stop_input())
        check("deliverables-only-passes", outcome.get("action") == "pass", f"只剩报告、补丁与声明了的备份应当放行，实际 {outcome}")
        # 声明成交付物的编译目录照列
        s = scenario("declared-build")
        directory = s.owned("w-declared")
        kept = s.build_directory(os.path.join(directory, "kept-target"))
        with open(os.path.join(directory, DECLARATION_FILE), "w", encoding="utf-8") as handle:
            handle.write("kept-target 备份 想留着下一趟接着编\n")
        outcome = s.decide(s.stop_input())
        check("build-directory-never-deliverable", blocked_with(outcome, kept), f"声明成备份的编译目录 {kept} 照样要列，实际 {outcome}")
        # 交接副本：主 agent 照列，子 agent 交回时不列
        s = scenario("handover")
        directory = s.owned("w-handover")
        handed = s.repository_copy(os.path.join(directory, "copy"))
        with open(os.path.join(directory, DECLARATION_FILE), "w", encoding="utf-8") as handle:
            handle.write("copy 交接 主 agent 在里面交带 --item 的\n")
        outcome = s.decide(s.stop_input())
        check("main-handover-copy-left", blocked_with(outcome, handed), f"主 agent 结束一轮时交接副本 {handed} 照列，实际 {outcome}")
        agent = s.subagent("ahandover", "派发", s.bash_call("toolu_claim", s.now - 80, s.now - 79,
                                                              "bash scripts/hosts/work-directory-ownership.sh claim w-handover --purpose 自证"))
        outcome = s.decide(s.subagent_stop_input("ahandover", agent))
        check("subagent-handover-passes", outcome.get("action") == "pass", f"子 agent 交回时交接副本不列，实际 {outcome}")
        # 合入后验证留下的快照（文件头「判哪几处」合入后验证快照那一段）：本会话的一层里 repo/ 还在的列（路径、种类、删法）
        s = scenario("merge-snapshot")
        snapshot = s.repository_copy(os.path.join(s.owned(f"{MERGE_SNAPSHOT_DIRECTORY_NAME}/2000-01-01-4242"), MERGE_SNAPSHOT_NAME))
        outcome = s.decide(s.stop_input())
        check("main-merge-snapshot-left", blocked_with(outcome, snapshot, KIND_MERGE_SNAPSHOT, f"rm -rf -- {snapshot}"),
              f"本会话合入后验证留下的快照 {snapshot} 应当拦下、列出种类、路径与删法，实际 {outcome}")
        # 别的会话的一层不列
        s = scenario("merge-snapshot-other-session")
        theirs = s.repository_copy(os.path.join(s.owned(f"{MERGE_SNAPSHOT_DIRECTORY_NAME}/2000-01-01-4343", session="s-somebody-else"), MERGE_SNAPSHOT_NAME))
        outcome = s.decide(s.stop_input())
        check("other-session-merge-snapshot-ignored", outcome.get("action") == "pass" and theirs not in str(outcome),
              f"别的会话的合入后验证快照 {theirs} 不该列，实际 {outcome}")
        # repo/ 已删（日志与归属文件还在）的一层不列
        s = scenario("merge-snapshot-removed")
        layer = s.owned(f"{MERGE_SNAPSHOT_DIRECTORY_NAME}/2000-01-01-4444")
        with open(os.path.join(layer, "test.log"), "w", encoding="utf-8") as handle:
            handle.write("日志留着\n")
        outcome = s.decide(s.stop_input())
        check("merge-snapshot-removed-not-listed", outcome.get("action") == "pass" and layer not in str(outcome),
              f"快照 repo/ 已删、日志与归属文件还在的一层 {layer} 不该列，实际 {outcome}")
        # 没有归属文件、归属文件只写了两行的一层不列（旧目录）
        s = scenario("merge-snapshot-no-owner")
        without = s.repository_copy(os.path.join(s.work, MERGE_SNAPSHOT_DIRECTORY_NAME, "2000-01-01-4545", MERGE_SNAPSHOT_NAME))
        short = s.repository_copy(os.path.join(s.work, MERGE_SNAPSHOT_DIRECTORY_NAME, "2000-01-01-4646", MERGE_SNAPSHOT_NAME))
        with open(os.path.join(os.path.dirname(short), ".owner"), "w", encoding="utf-8") as handle:
            handle.write(f"session: {s.session}\ncreated: 2000-01-01\n")
        outcome = s.decide(s.stop_input())
        check("merge-snapshot-without-owner-ignored", outcome.get("action") == "pass" and not any(path in str(outcome) for path in (without, short)),
              f"没有归属文件的一层 {without} 与归属文件只有两行的一层 {short} 不该列，实际 {outcome}")
        # 一层的 .deliverables 里把 repo 声明成基线：放行
        s = scenario("merge-snapshot-declared")
        layer = s.owned(f"{MERGE_SNAPSHOT_DIRECTORY_NAME}/2000-01-01-4747")
        s.repository_copy(os.path.join(layer, MERGE_SNAPSHOT_NAME))
        with open(os.path.join(layer, DECLARATION_FILE), "w", encoding="utf-8") as handle:
            handle.write(f"{MERGE_SNAPSHOT_NAME} {BASELINE_KIND} 验证段后面还要在它上面验\n")
        outcome = s.decide(s.stop_input())
        check("merge-snapshot-declared-baseline-passes", outcome.get("action") == "pass",
              f"一层的 {DECLARATION_FILE} 里声明成{BASELINE_KIND}的快照应当放行，实际 {outcome}")
        # 一层里有进程开着（那一趟还在跑）：整层不列
        s = scenario("merge-snapshot-held")
        layer = s.owned(f"{MERGE_SNAPSHOT_DIRECTORY_NAME}/2000-01-01-4848")
        snapshot = s.repository_copy(os.path.join(layer, MERGE_SNAPSHOT_NAME))
        with open(os.path.join(layer, "test.log"), "w", encoding="utf-8") as handle:
            s.hold(["sleep", "120"], stdout=handle)
        outcome = s.decide(s.stop_input())
        check("merge-snapshot-held-not-listed", outcome.get("action") == "pass" and snapshot not in str(outcome),
              f"那一层的日志有进程开着（那一趟还在跑），快照 {snapshot} 不该列，实际 {outcome}")
        s.stop()
        # 子 agent 交回时不判主 agent 会话的快照（子 agent 不跑合入后验证、删它不归子 agent）
        s = scenario("merge-snapshot-subagent")
        snapshot = s.repository_copy(os.path.join(s.owned(f"{MERGE_SNAPSHOT_DIRECTORY_NAME}/2000-01-01-4949"), MERGE_SNAPSHOT_NAME))
        s.subagent("amerge", "派发")
        outcome = s.decide(s.handback_input("amerge"))
        check("subagent-merge-snapshots-not-listed", outcome.get("action") == "pass" and snapshot not in str(outcome),
              f"子 agent 交回时本会话的合入后验证快照 {snapshot} 不该列，实际 {outcome}")
        # <工作根>/merge-and-verify/ 在而列不出（chmod 成 300：进得去、列不出）：照常放行，没判全的一句记进 notes
        s = scenario("merge-snapshot-root-unlisted")
        s.repository_copy(os.path.join(s.owned(f"{MERGE_SNAPSHOT_DIRECTORY_NAME}/2000-01-01-5050"), MERGE_SNAPSHOT_NAME))
        snapshot_root = os.path.join(s.work, MERGE_SNAPSHOT_DIRECTORY_NAME)
        os.chmod(snapshot_root, 0o300)
        try:
            outcome = s.decide(s.stop_input())
        finally:
            os.chmod(snapshot_root, 0o700)
        check("merge-snapshot-root-unlisted-pass-reported",
              outcome.get("action") == "pass" and any(f"合入后验证的日志目录根 {snapshot_root} 列不出" in note for note in outcome.get("notes") or []),
              f"{snapshot_root} 列不出应当放行，并在没判全的几处里点名它，实际 {outcome}")
        # 主 agent 声明成基线的段树：放行；子 agent 声明成基线不认（交接才是子 agent 给主 agent 的）
        s = scenario("segment-tree")
        directory = s.owned("w-segment-tree")
        segment_tree = s.repository_copy(os.path.join(directory, "segment-tree"))
        with open(os.path.join(directory, DECLARATION_FILE), "w", encoding="utf-8") as handle:
            handle.write(f"segment-tree {BASELINE_KIND} 写段下一批以它为基线\n")
        outcome = s.decide(s.stop_input())
        check("segment-tree-declared-baseline-passes", outcome.get("action") == "pass",
              f"主 agent 声明成{BASELINE_KIND}的段树 {segment_tree} 应当放行，实际 {outcome}")
        declared_by_subagent, _ = declarations([directory], "subagent")
        check("subagent-baseline-declaration-listed", segment_tree not in declared_by_subagent,
              f"子 agent 角色读同一份 {DECLARATION_FILE} 时{BASELINE_KIND}不算免列，{segment_tree} 应当不在免列表里，实际 {declared_by_subagent}")
        # 交回时 .deliverables 有写法不合的行（种类不在五种里）：没有副本留着也当场拒，说明点名那一行与五种种类；改对之后放行；Stop 不因它拦
        # 单独一个场景：下面「交主 agent 删」那一段的各格共用它自己的 hand 场景，这一段不插进去
        s = scenario("bad-declaration")
        directory = s.owned("w-bad-declaration")
        open(os.path.join(directory, "result.md"), "w").close()
        declaration_path = os.path.join(directory, DECLARATION_FILE)
        with open(declaration_path, "w", encoding="utf-8") as handle:
            handle.write("result.md 总结 这一轮的报告\n")
        s.subagent("abad", "派发", s.bash_call("toolu_claim", s.now - 80, s.now - 79,
                                                 "bash scripts/hosts/work-directory-ownership.sh claim w-bad-declaration --purpose 自证"))
        outcome = s.decide(s.handback_input("abad"))
        check("handback-bad-declaration-blocked",
              outcome.get("action") == "block" and f"{declaration_path}:1" in outcome.get("text", "") and all(kind in outcome.get("text", "") for kind in DELIVERABLE_KINDS),
              f"{DECLARATION_FILE} 里种类不在五种里的行，交回时应当拦下并点名文件、行号与五种种类，实际 {outcome}")
        check("bad-declaration-stop-passes", s.decide(s.stop_input()).get("action") == "pass",
              f"收工（Stop）不因 {DECLARATION_FILE} 里写法不合而拦，实际 {s.decide(s.stop_input())}")
        with open(declaration_path, "w", encoding="utf-8") as handle:
            handle.write("result.md 报告 这一轮的报告\n")
        outcome = s.decide(s.handback_input("abad"))
        check("handback-good-declaration-passes", outcome.get("action") == "pass", f"{DECLARATION_FILE} 改对之后交回应当放行，实际 {outcome}")
        # 子 agent 的草稿目录是工作根下一层目录底下的子目录：上一层是别人写的，那一层里写坏的行不拦它的交回；它自己那一层的坏行照拦
        s = scenario("parent-declaration")
        parent_directory = s.owned("w-parent-declaration")
        child_directory = os.path.join(parent_directory, "child")
        os.makedirs(child_directory)
        parent_declaration_path = os.path.join(parent_directory, DECLARATION_FILE)
        with open(parent_declaration_path, "w", encoding="utf-8") as handle:
            handle.write("snapshot 留底 别的会话在上一层写坏的一行\n")
        s.subagent("aparent", f"# 派发\n草稿目录：{child_directory}\n报告：{child_directory}/result.md\n")
        parent_outcome = s.decide(s.handback_input("aparent"))
        child_declaration_path = os.path.join(child_directory, DECLARATION_FILE)
        with open(child_declaration_path, "w", encoding="utf-8") as handle:
            handle.write("result.md 总结 子 agent 自己写坏的一行\n")
        child_outcome = s.decide(s.handback_input("aparent"))
        check("handback-parent-declaration-passes",
              parent_outcome.get("action") == "pass" and child_outcome.get("action") == "block"
              and f"{child_declaration_path}:1" in child_outcome.get("text", "") and parent_declaration_path not in child_outcome.get("text", ""),
              f"上一层 {parent_declaration_path} 里别人写坏的行不该拦子 agent 的交回（实际 {parent_outcome}），它自己那一层 {child_declaration_path} 的坏行应当拦下、只点名它（实际 {child_outcome}）")
        # 路径带空格、用引号包着：先按引号切再判三段，不判写法不合；声明成备份的带空格副本照样免列
        s = scenario("quoted-declaration")
        quoted_directory = s.owned("w-quoted-declaration")
        open(os.path.join(quoted_directory, "final report.md"), "w").close()
        spaced_copy = s.repository_copy(os.path.join(quoted_directory, "before patch"))
        with open(os.path.join(quoted_directory, DECLARATION_FILE), "w", encoding="utf-8") as handle:
            handle.write('"final report.md" 报告 这一轮的报告，主 agent 按路径存档\n`before patch` 备份 打补丁之前的原样\n')
        s.subagent("aquoted", "派发", s.bash_call("toolu_claim", s.now - 80, s.now - 79,
                                                    "bash scripts/hosts/work-directory-ownership.sh claim w-quoted-declaration --purpose 自证"))
        quoted_outcome = s.decide(s.handback_input("aquoted"))
        check("quoted-declaration-with-space-accepted", quoted_outcome.get("action") == "pass",
              f"带引号、带空格的路径照文件头写法不该判写法不合，声明成备份的 {spaced_copy} 免列，实际 {quoted_outcome}")
        # .deliverables 是符号链接（指过去的文件里种类写错）：不整份跳过，判写法不合并点名它
        s = scenario("symlinked-declaration")
        linked_directory = s.owned("w-symlinked-declaration")
        open(os.path.join(linked_directory, "result.md"), "w").close()
        link_target = os.path.join(s.root, "elsewhere-deliverables")
        with open(link_target, "w", encoding="utf-8") as handle:
            handle.write("result.md 总结 种类写错的一行\n")
        linked_declaration_path = os.path.join(linked_directory, DECLARATION_FILE)
        os.symlink(link_target, linked_declaration_path)
        s.subagent("alinked", "派发", s.bash_call("toolu_claim", s.now - 80, s.now - 79,
                                                   "bash scripts/hosts/work-directory-ownership.sh claim w-symlinked-declaration --purpose 自证"))
        linked_outcome = s.decide(s.handback_input("alinked"))
        check("handback-symlinked-declaration-blocked",
              linked_outcome.get("action") == "block" and linked_declaration_path in linked_outcome.get("text", "") and "符号链接" in linked_outcome.get("text", "")
              and link_target in linked_outcome.get("text", ""),
              f"符号链接的 {linked_declaration_path} 交回时应当拦下并点名它是符号链接与指向的 {link_target}，实际 {linked_outcome}")
        # 嵌套派发：下层子 agent 在它那一层的 .deliverables 声明了交接的副本，上层子 agent 交回时算已声明（那一份里写坏的行不拦上层）；
        # 下层没声明的副本上层照拦；主 agent 结束一轮时交接副本照列（只有子 agent 角色读下层的声明）
        s = scenario("nested-lower-handover")
        upper_directory = s.owned("w-nested-upper")
        lower_directory = os.path.join(upper_directory, "lower")
        os.makedirs(lower_directory)
        nested_copy = s.repository_copy(os.path.join(lower_directory, "copy"))
        lower_declaration_path = os.path.join(lower_directory, DECLARATION_FILE)
        with open(lower_declaration_path, "w", encoding="utf-8") as handle:
            handle.write("copy 交接 主 agent 在里面跑下层腿报告里的命令\nsnapshot 留底 下层自己写坏的一行\n")
        nested_main_outcome = s.decide(s.stop_input())   # 子 agent 的会话记录写下之前判：写下之后主 agent 当它还在跑，放行
        check("nested-lower-handover-main-listed", blocked_with(nested_main_outcome, nested_copy),
              f"主 agent 结束一轮时不读下层的声明，交接的 {nested_copy} 照列，实际 {nested_main_outcome}")
        s.subagent("anestedupper", f"# 派发\n草稿目录：{upper_directory}\n报告：{upper_directory}/result.md\n")
        nested_outcome = s.decide(s.handback_input("anestedupper"))
        check("nested-lower-handover-passes", nested_outcome.get("action") == "pass",
              f"下层 {lower_declaration_path} 声明了交接的 {nested_copy}，上层子 agent 交回时应当算已声明、下层写坏的那一行也不拦上层，实际 {nested_outcome}")
        with open(lower_declaration_path, "w", encoding="utf-8") as handle:
            handle.write("snapshot 留底 下层自己写坏的一行\n")
        nested_undeclared_outcome = s.decide(s.handback_input("anestedupper"))
        check("nested-undeclared-copy-blocks", blocked_with(nested_undeclared_outcome, nested_copy),
              f"下层没声明的 {nested_copy}，上层子 agent 交回时应当照拦，实际 {nested_undeclared_outcome}")
        # 上层被拦时，说明里点名下层的声明为什么没认上：写坏的那一行、整份被删、换成符号链接，各一格
        check("nested-lower-broken-line-named", blocked_with(nested_undeclared_outcome, nested_copy, f"{lower_declaration_path}:1"),
              f"下层 {lower_declaration_path} 第 1 行写坏、没声明 {nested_copy}，上层被拦时说明里应当点名那一行，实际 {nested_undeclared_outcome}")
        os.remove(lower_declaration_path)
        nested_missing_outcome = s.decide(s.handback_input("anestedupper"))
        check("nested-lower-declaration-missing-named", blocked_with(nested_missing_outcome, nested_copy, f"{lower_declaration_path} 不在"),
              f"下层 {lower_declaration_path} 被删，上层被拦时说明里应当点名它不在，实际 {nested_missing_outcome}")
        lower_link_target = os.path.join(s.root, "elsewhere-lower-deliverables")
        with open(lower_link_target, "w", encoding="utf-8") as handle:
            handle.write("copy 交接 主 agent 在里面跑下层腿报告里的命令\n")
        os.symlink(lower_link_target, lower_declaration_path)
        nested_symlink_outcome = s.decide(s.handback_input("anestedupper"))
        check("nested-lower-declaration-symlink-named",
              blocked_with(nested_symlink_outcome, nested_copy, lower_declaration_path, "符号链接", lower_link_target),
              f"下层 {lower_declaration_path} 换成指向 {lower_link_target} 的符号链接，上层被拦时说明里应当点名它是符号链接与指向哪，实际 {nested_symlink_outcome}")
        # 交主 agent 删（文件头「删不掉的」）：子 agent 删不掉的编译目录与仓副本，交回正文里逐个写一行就不拦，没写、写得太短、只写了上层目录的照拦
        s = scenario("hand")
        directory = s.owned("w-hand")
        hand_target = s.build_directory(os.path.join(directory, "target"))
        hand_copy = s.repository_copy(os.path.join(directory, "repository"))
        claim = s.bash_call("toolu_claim", s.now - 80, s.now - 79, "bash scripts/hosts/work-directory-ownership.sh claim w-hand --purpose 自证")
        reason = "rm 被权限拒，连续五次交不回"
        both = f"做完了。\n交主 agent 删 {hand_target}：{reason}\n交主 agent 删 {hand_copy}：{reason}\n"
        s.subagent("ahand", "派发", claim)
        outcome = s.decide(s.handback_input("ahand", both))
        check("subagent-hand-to-main-passes", outcome.get("action") == "pass",
              f"交回正文里逐个写了「交主 agent 删 <全路径>：<原因>」的编译目录 {hand_target} 与仓副本 {hand_copy} 应当放行，实际 {outcome}")
        outcome = s.decide(s.handback_input("ahand", f"做完了。\n交主 agent 删 {hand_target}：{reason}\n"))
        check("subagent-hand-undeclared-blocks", blocked_with(outcome, hand_copy) and hand_target not in outcome.get("text", ""),
              f"只写了编译目录 {hand_target}，没写的仓副本 {hand_copy} 应当照拦、写了的不列，实际 {outcome}")
        short_outcome = s.decide(s.handback_input("ahand", f"交主 agent 删 {hand_target}：删不掉\n交主 agent 删 {hand_copy}：删不掉\n"))
        check("subagent-hand-short-reason-blocks", blocked_with(short_outcome, hand_target, hand_copy),
              f"原因只有 3 个字（至少 {HAND_TO_MAIN_REASON_MINIMUM} 个字）应当照拦、两处都列，实际 {short_outcome}")
        outcome = s.decide(s.handback_input("ahand", f"交主 agent 删 {directory}：{reason}\n"))
        check("subagent-hand-ancestor-blocks", blocked_with(outcome, hand_target, hand_copy),
              f"只写了上层目录 {directory}，它底下的 {hand_target}、{hand_copy} 应当照拦，实际 {outcome}")
        # 出路与例子看一份什么都没写的交回拦下的单子，不借上面原因太短那一份的结果（hand-reason-unchecked 下那一份放行、单子是空的，只有原因太短那一格该红）
        blocked_text = s.decide(s.handback_input("ahand", "做完了。\n")).get("text", "")
        example = re.search(r"例：(交主 agent 删 [^\n]+)", blocked_text)
        parsed = HAND_TO_MAIN_LINE.search(example.group(1)) if example else None
        check("subagent-block-names-hand-way",
              "交主 agent 删 <全路径>：<删不掉的原因，至少 8 个字>" in blocked_text and parsed is not None and parsed.group(1) in (hand_target, hand_copy) and len(parsed.group(2)) >= 8,
              f"拦下的单子应当写出「交主 agent 删 <全路径>：<删不掉的原因，至少 8 个字>」这一条出路、附一行照写就认的例子，实际 {blocked_text!r}")
        # 出路本身要过得了别的闸：那行例子放进交回正文，喂给同挂 PreToolUse[SubagentHandback] 的交回正文闸与上游的交回尾门禁，都要放行
        gate_scripts = (os.path.join(HOOK_DIR, "handback-guard.sh"),
                        os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts", "claude-hooks", "handback-scratch-check.sh"))
        absent_gates = [path for path in gate_scripts if not os.path.isfile(path)]
        if absent_gates:
            NOT_RUN.append(f"subagent-hand-way-passes-handback-gates（{'、'.join(absent_gates)} 不在）")
        else:
            gate_exits = None   # 例子取不出也判红
            if example:
                gate_environment = {key: value for key, value in os.environ.items() if key != BREAK_VARIABLE}
                gate_environment.update(s.environment())
                gate_environment.update({"CLAUDE_PROJECT_DIR": REPOSITORY_ROOT, "HANDBACK_SCRATCH_CHECK_STATE_DIRECTORY": os.path.join(s.root, "scratch-state")})
                gate_payload = dict(s.handback_input("ahand", f"做完了。\n{example.group(1)}\n"), agent_type="tooling-writer")
                gate_exits = tuple(subprocess.run(["bash", script], input=json.dumps(gate_payload), capture_output=True, text=True, env=gate_environment,
                                                  timeout=120).returncode for script in gate_scripts)
            check("subagent-hand-way-passes-handback-gates", gate_exits == (0, 0),
                  f"拦下的单子里那行例子写进交回正文，交回正文闸与上游交回尾门禁都应当放行（退出码 0），实际 {gate_exits}")
        # 子 agent 收工（SubagentStop）：已送达的交回正文、收工前的回复里写了就放行；被交回闸拒掉的那次交回正文不算
        delivered = s.subagent("ahand-delivered", "派发", claim + s.handback_call("toolu_handback", s.now - 30, both))
        outcome = s.decide(s.subagent_stop_input("ahand-delivered", delivered))
        rejected = s.subagent("ahand-rejected", "派发", claim + s.handback_call("toolu_handback", s.now - 30, both, rejected=True))
        rejected_outcome = s.decide(s.subagent_stop_input("ahand-rejected", rejected))
        check("subagent-hand-stop-delivered-passes", outcome.get("action") == "pass" and rejected_outcome.get("action") == "block",
              f"收工时已送达的交回正文里写了「交主 agent 删」应当放行、被拒掉的那次不算（照拦），实际 送达 {outcome}，被拒 {rejected_outcome}")
        replied = s.subagent("ahand-reply", "派发", [*claim, s.reply(s.now - 30, both)])
        outcome = s.decide(s.subagent_stop_input("ahand-reply", replied))
        check("subagent-hand-stop-reply-passes", outcome.get("action") == "pass", f"收工前的回复里写了「交主 agent 删」应当放行，实际 {outcome}")
        # 只认编译目录、工作树与仓副本：库拷贝写了也照拦
        s = scenario("hand-library")
        library = s.library_copy(os.path.join(s.owned("w-hand-library"), "library-copy"))
        s.subagent("alibrary", "派发", s.bash_call("toolu_claim", s.now - 80, s.now - 79, "bash scripts/hosts/work-directory-ownership.sh claim w-hand-library --purpose 自证"))
        outcome = s.decide(s.handback_input("alibrary", f"交主 agent 删 {library}：{reason}\n"))
        check("subagent-hand-library-copy-blocks", blocked_with(outcome, library), f"库拷贝 {library} 写了「交主 agent 删」也应当照拦，实际 {outcome}")
        # 主 agent 结束一轮时照旧列：会话里写了「交主 agent 删」也不认
        s = scenario("hand-main")
        directory = s.owned("w-hand-main")
        main_target = s.build_directory(os.path.join(directory, "target"))
        main_copy = s.repository_copy(os.path.join(directory, "repository"))
        s.write(s.main_transcript, [s.reply(s.now - 30, f"交主 agent 删 {main_target}：{reason}\n交主 agent 删 {main_copy}：{reason}\n")], append=True)
        outcome = s.decide(s.stop_input())
        check("main-hand-declaration-ignored", blocked_with(outcome, main_target, main_copy),
              f"主 agent 结束一轮时，编译目录 {main_target} 与仓副本 {main_copy} 写了「交主 agent 删」也应当照列，实际 {outcome}")
        # 有在跑的后台任务：放行
        s = scenario("background")
        s.build_directory(os.path.join(s.owned("w-busy"), "target"))
        s.background_task("bwork", ["sleep", "120"])
        outcome = s.decide(s.stop_input())
        check("background-running-passes", outcome.get("action") == "pass", f"手里有在跑的后台任务应当放行，实际 {outcome}")
        s.stop()
        # 后台只有循环提醒在等：照判
        s = scenario("reminder")
        target = s.build_directory(os.path.join(s.owned("w-remind"), "target"))
        s.background_task("bremind", ["bash", "-c", "exec -a remind.sh sleep 120"])
        outcome = s.decide(s.stop_input())
        check("reminder-only-still-judged", blocked_with(outcome, target), f"后台只有循环提醒在等不算在跑的活，应当照拦，实际 {outcome}")
        s.stop()
        # 有在跑的子 agent：放行；子 agent 收到完成通知之后：照判
        s = scenario("subagent-running")
        s.build_directory(os.path.join(s.owned("w-sub"), "target"))
        s.subagent("arunning", "派发", s.bash_call("toolu_pending", s.now - 5, None, "sleep 5"))
        outcome = s.decide(s.stop_input())
        check("subagent-running-passes", outcome.get("action") == "pass", f"手里有在跑的子 agent 应当放行，实际 {outcome}")
        s = scenario("subagent-finished")
        target = s.build_directory(os.path.join(s.owned("w-sub"), "target"))
        s.subagent("afinished", "派发")
        s.write(s.main_transcript, [s.notification(time.time() + 5, "afinished")], append=True)
        outcome = s.decide(s.stop_input())
        check("subagent-finished-still-judged", blocked_with(outcome, target), f"子 agent 已收到完成通知、不算在跑，应当照拦，实际 {outcome}")
        # 有进程在用的不列
        s = scenario("held")
        directory = s.owned("w-held")
        held = s.build_directory(os.path.join(directory, "held-target"))
        idle = s.build_directory(os.path.join(directory, "idle-target"))
        s.hold(["sleep", "120"], cwd=held)
        outcome = s.decide(s.stop_input())
        check("held-copy-not-listed", blocked_with(outcome, idle) and held not in outcome.get("text", ""),
              f"有进程在用的 {held} 不该列、没人用的 {idle} 照列，实际 {outcome}")
        s.stop()
        # 换上流程的草稿树 <认领目录>/hook-swap/tree：没有进程在用的照列
        s = scenario("swap-tree-idle")
        idle_tree = s.repository_copy(os.path.join(s.owned("w-swap-idle"), "hook-swap", "tree"))
        outcome = s.decide(s.stop_input())
        check("swap-tree-idle-listed", blocked_with(outcome, idle_tree, f"rm -rf -- {idle_tree}"), f"没有进程在用的换上草稿树 {idle_tree} 应当拦下、列出删法，实际 {outcome}")
        # 换上流程的草稿树：命令行点到草稿目录（hook-swap.py 的 --scratch）、点到树里的文件的，进程的工作目录与文件描述符都不在树里，照样不列
        s = scenario("swap-tree-in-use")
        scratch_named = s.owned("w-swap-scratch-named")
        tree_by_scratch = s.repository_copy(os.path.join(scratch_named, "hook-swap", "tree"))
        tree_by_file = s.repository_copy(os.path.join(s.owned("w-swap-tree-named"), "hook-swap", "tree"))
        sleeper = "import time\ntime.sleep(120)\n"
        s.hold([sys.executable, "-c", sleeper, "hook-swap.py", "--scratch", scratch_named], stdout=subprocess.DEVNULL)
        s.hold([sys.executable, "-c", sleeper, "bash", os.path.join(tree_by_file, ".claude", "hooks", "fed.sh")], stdout=subprocess.DEVNULL)
        outcome = s.decide(s.stop_input())
        check("swap-tree-in-use-not-listed", outcome.get("action") == "pass" and tree_by_scratch not in str(outcome) and tree_by_file not in str(outcome),
              f"命令行点到草稿目录 {scratch_named}、点到 {tree_by_file} 里文件的换上草稿树不该列，实际 {outcome}")
        s.stop()
        # 换上流程的草稿树：决定拦之前那一趟刚删掉树（后台任务那一步换成替身，里面把树删了），不列
        s = scenario("swap-tree-gone")
        gone_tree = s.repository_copy(os.path.join(s.owned("w-swap-gone"), "hook-swap", "tree"))

        def background_work_after_tree_removed(transcript):
            shutil.rmtree(gone_tree)
            return []

        real_background_work = completion_cleanup_guard_decisions.background_work
        completion_cleanup_guard_decisions.background_work = background_work_after_tree_removed
        try:
            outcome = s.decide(s.stop_input())
        finally:
            completion_cleanup_guard_decisions.background_work = real_background_work
        check("swap-tree-gone-not-listed", not os.path.lexists(gone_tree) and outcome.get("action") == "pass",
              f"拦之前换上草稿树 {gone_tree} 已经没了，不该列，实际 {outcome}")
        # 子 agent：只判自己认领的，主 agent 的别的目录不判
        s = scenario("subagent-scope")
        mine = s.build_directory(os.path.join(s.owned("sub-claimed"), "target"))
        sibling = s.build_directory(os.path.join(s.owned("main-own"), "target"))
        agent = s.subagent("ascope", "派发", s.bash_call("toolu_claim", s.now - 80, s.now - 79,
                                                           "bash scripts/hosts/work-directory-ownership.sh claim sub-claimed --purpose '自证'"))
        outcome = s.decide(s.subagent_stop_input("ascope", agent))
        check("subagent-build-directory-left", blocked_with(outcome, mine), f"子 agent 自己认领的目录里的编译目录 {mine} 应当拦下，实际 {outcome}")
        check("subagent-ignores-siblings", sibling not in outcome.get("text", ""), f"主 agent 的别的目录 {sibling} 不该算到子 agent 头上，实际 {outcome}")
        # 子 agent：派发提示点名的草稿目录
        s = scenario("subagent-assigned")
        directory = s.owned("assigned")
        assigned_copy = s.repository_copy(os.path.join(directory, "copy"))
        agent = s.subagent("aassigned", f"# 派发\n草稿目录：{directory}\n报告：{directory}/result.md\n")
        outcome = s.decide(s.subagent_stop_input("aassigned", agent))
        check("subagent-assigned-directory-left", blocked_with(outcome, assigned_copy), f"派发提示点名的草稿目录里的仓副本 {assigned_copy} 应当拦下，实际 {outcome}")
        # 子 agent：「草稿目录：」写相对路径的按仓根解（lib_prompt_scratch_line.py 的 repository_root），不按钩子此刻的当前目录
        # （b12-scratch-lib.md「要主 agent 判的」第二条；开关 prompt-scratch-relative-by-cwd 下这两格红）
        s = scenario("subagent-assigned-relative")
        directory = s.owned("assigned-relative")
        relative_copy = s.repository_copy(os.path.join(directory, "copy"))
        relative_written = os.path.relpath(directory, module("lib_prompt_scratch_line").repository_root())
        deep_elsewhere = os.path.join(s.root, "cwd-elsewhere", *(["deeper"] * 8))
        os.makedirs(deep_elsewhere)
        cwd_named = s.owned("assigned-by-cwd")
        cwd_copy = s.repository_copy(os.path.join(cwd_named, "copy"))
        here = os.getcwd()
        try:
            os.chdir(deep_elsewhere)   # 当前目录在仓外、层数比仓根深：按当前目录解，那几个 .. 落不回工作根
            agent = s.subagent("arelative", f"# 派发\n草稿目录：{relative_written}\n报告：{relative_written}/result.md\n")
            by_root = s.decide(s.subagent_stop_input("arelative", agent))
            os.chdir(s.work)   # 当前目录是工作根：只写目录名的，按当前目录解就落到工作根下那个目录，按仓根解不在工作根下
            agent = s.subagent("acwd", "# 派发\n草稿目录：assigned-by-cwd\n报告：assigned-by-cwd/result.md\n")
            by_cwd = s.decide(s.subagent_stop_input("acwd", agent))
        finally:
            os.chdir(here)
        check("subagent-assigned-relative-by-repository-root", blocked_with(by_root, relative_copy),
              f"「草稿目录：{relative_written}」按仓根解是 {directory}，里面的仓副本 {relative_copy} 应当拦下（当前目录在 {deep_elsewhere}），实际 {by_root}")
        check("subagent-assigned-relative-not-by-cwd", cwd_copy not in by_cwd.get("text", ""),
              f"「草稿目录：assigned-by-cwd」按仓根解不在工作根下，不该按当前目录 {s.work} 解成 {cwd_named}、把 {cwd_copy} 算到这个子 agent 头上，实际 {by_cwd}")
        # 子 agent：提示里转述别的子 agent 的草稿目录（句中「，草稿目录 <别处>」、列表「- 草稿目录 <别处>」）不算它的；那一行的认法调的是 lib_prompt_scratch_line.py
        s = scenario("subagent-assigned-relay")
        mine, previous, listed = s.owned("assigned-mine"), s.owned("assigned-previous"), s.owned("assigned-listed")
        previous_copy = s.repository_copy(os.path.join(previous, "copy"))
        listed_copy = s.repository_copy(os.path.join(listed, "copy"))
        agent = s.subagent("arelay", f"# 派发\n草稿目录：{mine}\n报告：{mine}/result.md\n前一段交回在先，草稿目录 {previous}/ 里有它的副本。\n要读的：\n- 草稿目录 {listed}/ 下的 logs/\n")
        outcome = s.decide(s.subagent_stop_input("arelay", agent))
        check("subagent-assigned-relay-ignored", previous_copy not in outcome.get("text", "") and listed_copy not in outcome.get("text", ""),
              f"转述里别的子 agent 的草稿目录 {previous}、{listed} 不该算到这个子 agent 头上，实际 {outcome}")
        shared_reading = module("lib_prompt_scratch_line")
        reading = shared_reading.scratch_line_values
        shared_reading.scratch_line_values = lambda text: [previous]
        try:
            outcome = s.decide(s.subagent_stop_input("arelay", agent))
        finally:
            shared_reading.scratch_line_values = reading
        check("subagent-assigned-shared-reading", blocked_with(outcome, previous_copy),
              f"草稿目录一行的认法换成替身（只交 {previous}）之后应当拦下 {previous_copy}：调的要是 lib_prompt_scratch_line.py 那一份，实际 {outcome}")
        # 别的时间窗里建的运行目录不算这里的
        s = scenario("foreign-run")
        s.library_copy(os.path.join(s.runs, "crash-amplification-foreign", "library-copy"))
        s.write(s.main_transcript, [s.user_text(s.now - 7300, "早先")] + s.bash_call("toolu_old", s.now - 7200, s.now - 7100, "bash scripts/crash/crash-amplification.sh"), append=False)
        outcome = s.decide(s.stop_input())
        check("foreign-run-directory-ignored", outcome.get("action") == "pass", f"不在自己 Bash 调用时间窗里建的运行目录不该判，实际 {outcome}")
        # 续跑回来同一张单：放行交用户；单子变了：照拦
        s = scenario("release")
        directory = s.owned("w-release")
        target = s.build_directory(os.path.join(directory, "target"))
        first = s.decide(s.stop_input())
        second = s.decide(s.stop_input(active=True))
        check("unchanged-after-block-releases", first.get("action") == "block" and second.get("action") == "release" and target in second.get("text", ""),
              f"拦过一次、续跑回来一个没删应当放行并把单子交用户，实际 {first.get('action')} 之后 {second}")
        s.build_directory(os.path.join(directory, "another-target"))
        third = s.decide(s.stop_input(active=True))
        check("changed-after-block-blocks-again", third.get("action") == "block", f"续跑回来单子变了应当照拦，实际 {third}")
        # 单上写 du 的大小
        s = scenario("size")
        target = s.build_directory(os.path.join(s.owned("w-size"), "target"))
        outcome = s.decide(s.stop_input())
        check("block-lists-size", re.search(rf"  {KIND_BUILD}  {re.escape(target)}  [0-9.,]+[KMGTP]?\n", outcome.get("text", "")) is not None,
              f"单上每一行要写 du -sh 的大小，实际 {outcome}")
        # 主 agent：本会话的临时草稿目录（tasks/ 不看）
        s = scenario("scratchpad")
        pad = os.path.join(s.scratch, "project-directory", s.session)
        pad_copy = s.repository_copy(os.path.join(pad, "copy"))
        task_copy = s.build_directory(os.path.join(pad, "tasks", "target"))
        outcome = s.decide(s.stop_input())
        check("main-scratchpad-copy-left", blocked_with(outcome, pad_copy) and task_copy not in outcome.get("text", ""),
              f"本会话临时草稿目录里的仓副本 {pad_copy} 应当拦下、tasks/ 下的不看，实际 {outcome}")
        # 什么都没留：放行
        s = scenario("clean")
        open(os.path.join(s.owned("w-clean"), "result.md"), "w").close()
        outcome = s.decide(s.stop_input())
        check("nothing-left-passes", outcome.get("action") == "pass", f"只剩报告应当放行，实际 {outcome}")
        # 扫到上限没看完、没有残留：照常放行，没判全的几处与出路打出来、检出记录多一行（结局只记）；上限经 COMPLETION_CLEANUP_SCAN_MAX_DIRECTORIES 调成 3
        s = scenario("truncated-scan")
        deep = s.owned("w-deep")
        for name in ("a", "b", "c", "d", "e", "f"):
            os.makedirs(os.path.join(deep, name, "inner"))
        outcome = s.decide(s.stop_input(), {SCAN_LIMIT_VARIABLE: "3"})
        detections = os.path.join(s.root, "detections-truncated.jsonl")
        printed = io.StringIO()
        report_unjudged_pass(s.stop_input(), outcome.get("notes") or [], printed, detections)
        entries, text = detection_entries(detections), printed.getvalue()
        check("truncated-scan-pass-reported",
              outcome.get("action") == "pass" and f"看了 3 个目录到顶，这几处没看完：{deep}" in text and "出路" in text
              and len(entries) == 1 and entries[0].get("outcome") == "只记" and entries[0].get("session_id") == s.session,
              f"扫到上限（3 个）没看完 {deep}、没有残留应当放行，打出没看完的几处与出路、检出记录多一行，实际 {outcome}，打出 {text!r}，检出 {entries}")
        # 运行目录取不到创建时间（birth_times 换成替身，全交 None）、没有残留：照常放行，「取不到创建时间」那一句打出来、检出记录多一行
        s = scenario("birth-unknown")
        run = s.run_directory(s.dead_pid(), owner_session=s.session)
        real_birth_times = completion_cleanup_guard_decisions.birth_times
        completion_cleanup_guard_decisions.birth_times = lambda paths: {path: None for path in sorted(paths)}
        try:
            outcome = s.decide(s.stop_input())
        finally:
            completion_cleanup_guard_decisions.birth_times = real_birth_times
        detections = os.path.join(s.root, "detections-birth.jsonl")
        printed = io.StringIO()
        report_unjudged_pass(s.stop_input(), outcome.get("notes") or [], printed, detections)
        entries, text = detection_entries(detections), printed.getvalue()
        check("birth-unknown-pass-reported",
              outcome.get("action") == "pass" and "取不到创建时间的运行目录 1 个没判" in text and run in text and len(entries) == 1 and entries[0].get("outcome") == "只记",
              f"运行目录 {run} 取不到创建时间、没有残留应当放行，打出那一句、检出记录多一行，实际 {outcome}，打出 {text!r}，检出 {entries}")
        # 读不了的目录（这里把认领目录里的一层 chmod 成 000）、没有残留：照常放行，「读不了的目录」那一句打出来、检出记录多一行
        s = scenario("unreadable-directory")
        locked = os.path.join(s.owned("w-locked"), "locked")
        os.makedirs(locked)
        os.chmod(locked, 0)
        try:
            outcome = s.decide(s.stop_input())
        finally:
            os.chmod(locked, 0o700)
        detections = os.path.join(s.root, "detections-unreadable.jsonl")
        printed = io.StringIO()
        report_unjudged_pass(s.stop_input(), outcome.get("notes") or [], printed, detections)
        entries, text = detection_entries(detections), printed.getvalue()
        check("unreadable-directory-pass-reported",
              outcome.get("action") == "pass" and "读不了的目录 1 个没看" in text and locked in text and len(entries) == 1 and entries[0].get("outcome") == "只记",
              f"认领目录里的 {locked} 读不了、没有残留应当放行，打出那一句、检出记录多一行，实际 {outcome}，打出 {text!r}，检出 {entries}")
        # 主 agent 的工作根在而列不出（chmod 成 300：进得去、列不出）：照常放行，「工作根列不出」那一句打出来、检出记录多一行
        s = scenario("work-root-unlisted")
        s.owned("w-hidden")
        os.chmod(s.work, 0o300)
        try:
            outcome = s.decide(s.stop_input())
        finally:
            os.chmod(s.work, 0o700)
        detections = os.path.join(s.root, "detections-unlisted.jsonl")
        printed = io.StringIO()
        report_unjudged_pass(s.stop_input(), outcome.get("notes") or [], printed, detections)
        entries, text = detection_entries(detections), printed.getvalue()
        check("work-root-unlisted-pass-reported",
              outcome.get("action") == "pass" and f"工作根 {s.work} 列不出" in text and len(entries) == 1 and entries[0].get("outcome") == "只记",
              f"工作根 {s.work} 列不出应当放行，打出那一句、检出记录多一行，实际 {outcome}，打出 {text!r}，检出 {entries}")
        # 会话记录在而读不了（chmod 成 000）：判不了，照「放行但必记检出」走 skip（入口记检出那一步由 unjudged-skip-recorded 那一格核）
        s = scenario("transcript-unreadable")   # 没有残留：弄坏开关下读不了的会话记录被当成空的，照常放行、不记（有残留时后面读后台任务那一步抛异常，走「自己出错」那一支）
        s.owned("w-unreadable-transcript")
        os.chmod(s.main_transcript, 0)
        try:
            outcome = s.decide(s.stop_input())
        finally:
            os.chmod(s.main_transcript, 0o600)
        check("transcript-unreadable-skip-reported", outcome.get("action") == "skip" and "读不了" in outcome.get("text", ""),
              f"会话记录 {s.main_transcript} 读不了应当判不了（skip、说明写读不了），实际 {outcome}")
        # 迁库遗留（文件头「判哪几处」迁库遗留那一段）：判定库服务的状态目录里服务那份库不列、旁边的整份库拷贝列（写出大小）；
        # 数据根下按种类名认的旧文件库列，命令只点到它的上层目录也算
        s = scenario("leftover-data-root")
        state = s.prune_paths().state_directory
        service_library = s.library_copy(os.path.join(state, "library"))
        old_layout = s.library_copy(os.path.join(state, "library.old-layout-2000-01-01"))
        family = s.filled(os.path.join(s.data, "legacy", "singlefs-unit-verdicts"))
        s.bash(f"mv {service_library} {old_layout}")
        s.bash(f"mkdir -p {s.data}/legacy/ && mv {s.data}/singlefs-unit-verdicts {s.data}/legacy/")
        outcome = s.decide(s.stop_input())
        check("prune-state-library-copy-blocked", blocked_with(outcome, old_layout)
              and re.search(rf"  {KIND_LIBRARY}  {re.escape(old_layout)}  [0-9.,]+[KMGTP]?\n", outcome.get("text", "")) is not None,
              f"判定库服务状态目录里的整份库拷贝 {old_layout} 应当拦下并写出 du 的大小，实际 {outcome}")
        check("service-library-not-listed", service_library not in outcome.get("paths", []) and service_library not in [path for path, _ in outcome.get("unknown", [])],
              f"服务自己那份库 {service_library} 不该列（拦的单与「{UNKNOWN_SECTION_TITLE}」都不该有），实际 {outcome}")
        # 只看拦的单（paths）：拦下时「认不出是谁的」一节也写进说明，只查说明里有没有这条路径，没认作本会话的也照样过
        check("data-root-family-library-blocked", blocked_with(outcome, family) and family in outcome.get("paths", []),
              f"数据根下按种类名认的旧文件库 {family}（命令点到它的上层 {s.data}/legacy）应当拦下，实际 {outcome}")
        # 单元文件 ExecStart 的 --state-directory 指到别处：那里的 library 是服务那份、不列，旁边的拷贝列
        s = scenario("leftover-unit-file")
        unit_state = os.path.join(s.root, "unit-state")
        paths = s.prune_paths()
        os.makedirs(os.path.dirname(paths.unit_file), exist_ok=True)
        with open(paths.unit_file, "w", encoding="utf-8") as handle:
            handle.write(f"[Service]\nExecStart={paths.binary} serve --spawned-by-parent --state-directory {unit_state}\n")
        unit_library = s.library_copy(os.path.join(unit_state, "library"))
        before_migration = s.library_copy(os.path.join(unit_state, "library.before-migration"))
        s.bash(f"cp -a {unit_library} {before_migration}")
        outcome = s.decide(s.stop_input())
        check("unit-file-state-directory-read", blocked_with(outcome, before_migration) and unit_library not in outcome.get("paths", [])
              and unit_library not in [path for path, _ in outcome.get("unknown", [])],
              f"单元文件的 --state-directory 是 {unit_state}：里面的拷贝 {before_migration} 应当拦下、服务那份 {unit_library} 不列，实际 {outcome}")
        # 相对路径照同一条命令里前面 cd 到的目录解
        s = scenario("leftover-relative")
        moved = s.library_copy(os.path.join(s.data, "rocks-old"))
        s.bash(f"cd {s.data} && mv library-copy rocks-old")
        outcome = s.decide(s.stop_input())
        check("relative-after-cd-attributed", blocked_with(outcome, moved), f"cd {s.data} 之后写相对路径点到的 {moved} 应当算本会话的、拦下，实际 {outcome}")
        # 库备份压缩包：名字里带 verdict 的列，别的压缩包不列
        s = scenario("leftover-archive")
        flips = os.path.join(s.data, "family-flips")
        kept_archive = s.archive(os.path.join(flips, "unit-verdicts-old-dirs-2000-01-01.tar.gz"))
        report_archive = s.archive(os.path.join(flips, "report-2000-01-01.tar.gz"))
        s.bash(f"tar -czf {kept_archive} -C {s.data} old-dirs && tar -czf {report_archive} notes")
        outcome = s.decide(s.stop_input())
        check("library-archive-blocked", blocked_with(outcome, kept_archive) and report_archive not in str(outcome),
              f"库备份 {kept_archive} 应当拦下，名字里不带种类名、verdict、library 的 {report_archive} 不列，实际 {outcome}")
        # 工作根下没写归属文件的目录：本会话命令点到过的库拷贝拦，没点到的只进「认不出是谁的」
        s = scenario("leftover-no-owner")
        loose = s.library_copy(os.path.join(s.work, "w-loose", "service-library-before-migration"))
        s.bash(f"cp -a /nowhere/library {loose}")
        outcome = s.decide(s.stop_input())
        check("no-owner-mentioned-copy-blocked", blocked_with(outcome, loose), f"没写归属文件的工作根目录里、本会话命令点到过的库拷贝 {loose} 应当拦下，实际 {outcome}")
        s = scenario("leftover-no-owner-unmentioned")
        stray = s.library_copy(os.path.join(s.work, "w-stray", "library-copy"))
        alone = s.decide(s.stop_input())
        listed = "\n".join(unknown_lines(alone.get("unknown", [])))
        check("no-owner-unmentioned-reported-not-blocked",
              alone.get("action") == "pass" and stray in [path for path, _ in alone.get("unknown", [])]
              and stray in listed and listed.index(UNKNOWN_SECTION_TITLE) < listed.index(stray),
              f"没写归属文件的工作根目录里、没被点到的库拷贝 {stray} 不该拦，只进「{UNKNOWN_SECTION_TITLE}」一节，实际 {alone}")
        # 只点到锚点（数据根本身、字面前缀是数据根的通配、判定库服务的状态目录）不算点到下面的东西；用改动命令点，只读命令本来就不算
        s = scenario("leftover-anchor")
        stale = s.library_copy(os.path.join(s.data, "stale-library"))
        s.bash(f"mkdir -p {s.data} {s.prune_paths().state_directory} && rsync -a {s.data}/* /nowhere/backup/")
        outcome = s.decide(s.stop_input())
        check("anchor-mentions-not-attributed", outcome.get("action") == "pass" and stale not in outcome.get("paths", []),
              f"命令只点到数据根与 {s.data}/* 这种通配，{stale} 不该算本会话的，实际 {outcome}")
        # 只读命令（du、ls、wc、不带 -delete 的 find）那一段里的路径不算点到：只进「认不出是谁的」、不拦；
        # 同一条命令里 mv 那一段点到的、find 带 -delete 点到的照拦
        s = scenario("leftover-read-only")
        looked = s.filled(os.path.join(s.data, "legacy-looked", "singlefs-unit-verdicts"))
        s.bash(f"du -sh {looked} && ls {looked} | wc -l; find {looked} -name keys.json")
        looked_outcome = s.decide(s.stop_input())
        s = scenario("leftover-read-only-then-moved")
        moved = s.filled(os.path.join(s.data, "legacy-moved", "singlefs-unit-verdicts"))
        purged = s.filled(os.path.join(s.data, "legacy-purged", "singlefs-mutate-library"))
        s.bash(f"ls {moved} && mv {s.data}/staging {moved}")
        s.bash(f"find {purged} -name '*.tmp' -delete")
        moved_outcome = s.decide(s.stop_input())
        check("read-only-mentions-not-attributed",
              looked_outcome.get("action") == "pass" and looked in [path for path, _ in looked_outcome.get("unknown", [])]
              and looked not in looked_outcome.get("paths", [])
              and moved_outcome.get("action") == "block" and moved in moved_outcome.get("paths", []) and purged in moved_outcome.get("paths", []),
              f"只用 du、ls、find 点过的 {looked} 应当只进「{UNKNOWN_SECTION_TITLE}」、不拦，实际 {looked_outcome}；"
              f"同一条命令里 mv 点到的 {moved}、find -delete 点到的 {purged} 应当拦下，实际 {moved_outcome}")
        # 种类名现读两处（这里换成样本文件）：样本里的名字认，样本里没有的不认（不是钩子里抄的一份）
        s = scenario("leftover-species")
        samples = os.path.join(s.root, "species-samples")
        os.makedirs(samples)
        sample_sources = {"client": os.path.join(samples, "client.py"), "registry": os.path.join(samples, "registry.tsv")}
        with open(sample_sources["client"], "w", encoding="utf-8") as handle:
            handle.write('"""样本\n\n键空间（样本）：\n  族名：unit-verdicts（种类是 singlefs-sample-verdicts/ 下那一层）\n连哪一处：样本\n"""\n')
        with open(sample_sources["registry"], "w", encoding="utf-8") as handle:
            handle.write("# 样本\nscripts/sample.py\t-\tprune-server（unit-verdicts 族，键空间 unit-verdicts/singlefs-sample-ledger/…）\treuse-three-questions\n")
        sampled = [s.filled(os.path.join(s.data, "old", name)) for name in ("singlefs-sample-verdicts", "singlefs-sample-ledger")]
        unlisted = s.filled(os.path.join(s.data, "old", "singlefs-unit-verdicts"))
        s.bash("mkdir -p " + " ".join(sampled + [unlisted]))
        saved_sources = dict(SPECIES_SOURCES)
        SPECIES_SOURCES.update(sample_sources)
        try:
            outcome = s.decide(s.stop_input())
        finally:
            SPECIES_SOURCES.update(saved_sources)
        check("species-read-from-registries", blocked_with(outcome, *sampled) and unlisted not in outcome.get("text", ""),
              f"种类名照样本两处读：{sampled} 应当拦下，样本里没有的 {unlisted} 不该认，实际 {outcome}")
        # 种类名一处都读不出：照常放行，记一处没判全
        s = scenario("leftover-species-unreadable")
        SPECIES_SOURCES.update({key: os.path.join(s.root, f"missing-{key}") for key in SPECIES_SOURCES})
        try:
            outcome = s.decide(s.stop_input())
        finally:
            SPECIES_SOURCES.update(saved_sources)
        check("species-unreadable-pass-reported", outcome.get("action") == "pass" and any("种类名读不出" in note for note in outcome.get("notes") or []),
              f"种类名的两处都读不了应当放行并记「种类名读不出」一处没判全，实际 {outcome}")
        # 数据根取不到（变量没设、多机配置不在）：照常放行，记一处没判全
        s = scenario("leftover-data-root-unconfigured")
        outcome = s.decide(s.stop_input(), {module("lib_work_root").DATA_ROOT_VARIABLE: "",
                                            module("lib_work_root").MULTI_HOST_CONFIGURATION_VARIABLE: os.path.join(s.root, "missing-multi-host.env")})
        check("data-root-unconfigured-pass-reported", outcome.get("action") == "pass" and any("数据根取不到" in note for note in outcome.get("notes") or []),
              f"数据根取不到应当放行并记「数据根取不到」一处没判全，实际 {outcome}")
        # 在用的迁库遗留不列：旧文件库里的文件、库备份本身各有进程开着
        s = scenario("leftover-held")
        held_library = s.filled(os.path.join(s.data, "legacy-held", "singlefs-unit-verdicts"))
        held_archive = s.archive(os.path.join(s.data, "library-backup-held.tar.gz"))
        with open(os.path.join(held_library, "keys.log"), "w") as handle:
            s.hold(["sleep", "120"], stdout=handle)
        with open(held_archive, "ab") as handle:
            s.hold(["sleep", "120"], stdout=handle)
        s.bash(f"cp -a /nowhere/library {held_library} && cp /nowhere/backup.tar.gz {held_archive}")
        outcome = s.decide(s.stop_input())
        check("held-leftover-not-listed", outcome.get("action") == "pass" and held_library not in str(outcome) and held_archive not in str(outcome),
              f"有进程开着的 {held_library}、{held_archive} 不该列，实际 {outcome}")
        s.stop()
        # 数据根层的旧库靠认领目录里的 .deliverables 声明放不过（路径落到认领目录外的行不算）
        s = scenario("leftover-declared-outside")
        directory = s.owned("w-declare")
        kept_family = s.filled(os.path.join(s.data, "legacy-kept", "singlefs-mutate-library"))
        with open(os.path.join(directory, DECLARATION_FILE), "w", encoding="utf-8") as handle:
            handle.write(f"{os.path.relpath(os.path.join(s.data, 'legacy-kept'), directory)} 备份 想留着这份旧库\n")
        s.bash(f"mv /nowhere/old-library {kept_family}")
        outcome = s.decide(s.stop_input())
        check("data-root-leftover-not-declarable", blocked_with(outcome, kept_family),
              f"认领目录外的 {kept_family} 声明成备份也照拦，实际 {outcome}")
        # 认领目录里声明成备份的库备份照放行
        s = scenario("leftover-declared-archive")
        directory = s.owned("w-archive")
        s.archive(os.path.join(directory, "library-backup-2000-01-01.tar.gz"))
        with open(os.path.join(directory, DECLARATION_FILE), "w", encoding="utf-8") as handle:
            handle.write("library-backup-2000-01-01.tar.gz 备份 打补丁之前的旧库\n")
        outcome = s.decide(s.stop_input())
        check("declared-archive-passes", outcome.get("action") == "pass", f"认领目录里声明成备份的库备份应当放行，实际 {outcome}")
        # 数据根是工作根的上一层（本机的样子）：别的会话认领的目录照认领规则不判，命令点到过也不算
        s = scenario("leftover-other-session")
        theirs = s.library_copy(os.path.join(s.owned("w-theirs-leftover", session="s-somebody-else"), "copy"))
        s.bash(f"cp -a /nowhere/library {theirs}")
        outcome = s.decide(s.stop_input(), {module("lib_work_root").DATA_ROOT_VARIABLE: s.root})
        check("other-session-leftover-ignored", outcome.get("action") == "pass" and theirs not in str(outcome),
              f"别的会话认领的目录里的 {theirs} 不归迁库遗留判，实际 {outcome}")
        # 子 agent 只按它自己会话记录里的命令认：主会话记录点到的不算它的
        s = scenario("leftover-subagent")
        mine = s.filled(os.path.join(s.data, "legacy-mine", "singlefs-unit-verdicts"))
        main_side = s.filled(os.path.join(s.data, "legacy-main", "singlefs-mutate-library"))
        agent = s.subagent("aleftover", "派发", s.bash_call("toolu_leftover_mine", s.now - 20, s.now - 19,
                                                             f"mv {s.data}/singlefs-unit-verdicts {s.data}/legacy-mine/"))
        s.bash(f"mv {s.data}/singlefs-mutate-library {s.data}/legacy-main/")
        outcome = s.decide(s.subagent_stop_input("aleftover", agent))
        check("subagent-leftover-by-own-commands", blocked_with(outcome, mine) and main_side not in outcome.get("paths", []),
              f"子 agent 自己的命令点到的 {mine} 应当拦下，主会话记录点到的 {main_side} 不算它的，实际 {outcome}")
        # 数据根表里登记的在用项（下一层那一个）本身不按名字认成旧文件库
        s = scenario("leftover-registered-entry")
        entry = next(name for name in module("lib_work_root").SHARED_DATA_ENTRIES if re.fullmatch(SPECIES_TOKEN, name))
        live = s.filled(os.path.join(s.data, entry))
        registry_sample = os.path.join(s.root, "registry.tsv")
        with open(registry_sample, "w", encoding="utf-8") as handle:
            handle.write(f"# 样本\nscripts/sample.py\t-\tprune-server（unit-verdicts 族，键空间 unit-verdicts/{entry}/…）\treuse-three-questions\n")
        s.bash(f"cp -a /nowhere/library {live}")
        SPECIES_SOURCES["registry"] = registry_sample
        try:
            outcome = s.decide(s.stop_input())
        finally:
            SPECIES_SOURCES.update(saved_sources)
        check("registered-entry-not-listed", outcome.get("action") == "pass" and live not in str(outcome),
              f"数据根下登记的在用项 {live} 名字与种类名相同也不该列，实际 {outcome}")
        # 包装：起钩子本身（不带弄坏开关）
        clean_environment = {key: value for key, value in os.environ.items() if key != BREAK_VARIABLE}
        s = scenario("wrapper")
        target = s.build_directory(os.path.join(s.owned("w-wrap"), "target"))
        completed = subprocess.run(["bash", HOOK_PATH], input=json.dumps(s.stop_input()), capture_output=True, text=True,
                                   env=dict(clean_environment, **s.environment()))
        try:
            printed = json.loads(completed.stdout)
        except ValueError:
            printed = {}
        check("wrapper-stop-json", completed.returncode == 0 and printed.get("decision") == "block" and target in printed.get("reason", ""),
              f"Stop 上拦下应当退 0、打 decision=block 的 JSON，实际退 {completed.returncode}，输出 {completed.stdout!r}{completed.stderr!r}")
        mine = s.build_directory(os.path.join(s.owned("w-wrap-sub"), "target"))
        s.subagent("awrap", "派发", s.bash_call("toolu_claim", s.now - 80, s.now - 79, "bash scripts/hosts/work-directory-ownership.sh claim w-wrap-sub --purpose x"))
        completed = subprocess.run(["bash", HOOK_PATH], input=json.dumps(s.handback_input("awrap")), capture_output=True, text=True,
                                   env=dict(clean_environment, **s.environment()))
        check("wrapper-handback-exit-2", completed.returncode == 2 and mine in completed.stderr and completed.stdout == "",
              f"交回之前拦下应当退 2、清单写 stderr，实际退 {completed.returncode}，stderr {completed.stderr!r}")
        state = True
        for raw in ("", "不是 JSON", "[]", json.dumps({}), json.dumps({"hook_event_name": "Stop", "session_id": "../x", "transcript_path": "/nowhere.jsonl"})):
            completed = subprocess.run(["bash", HOOK_PATH], input=raw, capture_output=True, text=True, env=dict(clean_environment, **s.environment()))
            state = state and completed.returncode == 0 and completed.stdout == ""
        check("bad-input-passes", state, "读不出的输入、没有会话号、会话号带路径符号都应当放行（退 0、标准输出为空）")
        # 判不了的放行照「放行但必记检出」：读不出的输入、不是 JSON 对象、会话号带路径符号、找不到会话记录，各照常放行、stderr 写「没判」、检出记录各多一行（结局只记）。
        # 在进程里起 hook_main（弄坏开关 unjudged-skip-silent 只在自证里认，起钩子本身时带不进去）；另一格起钩子本身核退出码与 stderr
        import contextlib
        def hook_main_in_process(raw_bytes, environment):
            saved_stdin, saved_environment = sys.stdin, {key: os.environ.get(key) for key in environment}
            os.environ.update(environment)
            sys.stdin = io.TextIOWrapper(io.BytesIO(raw_bytes), encoding="utf-8")
            printed_out, printed_err = io.StringIO(), io.StringIO()
            try:
                with contextlib.redirect_stdout(printed_out), contextlib.redirect_stderr(printed_err):
                    code = hook_main()
            finally:
                sys.stdin = saved_stdin
                for key, value in saved_environment.items():
                    if value is None:
                        os.environ.pop(key, None)
                    else:
                        os.environ[key] = value
            return code, printed_out.getvalue(), printed_err.getvalue()
        s = scenario("unjudged-skip")
        detections = s.environment()[DETECTIONS_VARIABLE]
        unjudged_inputs = ("不是 JSON", "[]", json.dumps({"hook_event_name": "Stop", "session_id": "../x", "transcript_path": "/nowhere.jsonl"}),
                           json.dumps({"hook_event_name": "Stop", "session_id": s.session, "transcript_path": os.path.join(s.project, "missing.jsonl")}))
        outcomes = [hook_main_in_process(raw.encode("utf-8"), s.environment()) for raw in unjudged_inputs]
        entries = detection_entries(detections)
        check("unjudged-skip-recorded",
              all(code == 0 and out == "" and "没判" in err and "放行但必记检出" in err for code, out, err in outcomes) and len(entries) == len(unjudged_inputs)
              and all(entry.get("outcome") == "只记" and all(str(finding).startswith(UNJUDGED_SKIP_PREFIX) for finding in entry.get("findings") or [None])
                      for entry in entries)
              and entries[-1].get("session_id") == s.session,
              f"判不了的放行应当退 0、标准输出为空、stderr 写没判与放行但必记检出、检出记录每次多一行（结局只记），实际 {outcomes}，检出 {entries}")
        # 自己出错（decide 换成抛异常的替身）：照常放行，stderr 写出错与没判，检出记录多一行
        s = scenario("self-error")
        detections = s.environment()[DETECTIONS_VARIABLE]
        real_decide = completion_cleanup_guard_decisions.decide
        def exploding_decide(hook_input):
            raise RuntimeError("自证造的出错")
        completion_cleanup_guard_decisions.decide = exploding_decide
        try:
            code, out, err = hook_main_in_process(json.dumps(s.stop_input()).encode("utf-8"), s.environment())
        finally:
            completion_cleanup_guard_decisions.decide = real_decide
        entries = detection_entries(detections)
        check("self-error-pass-recorded",
              code == 0 and out == "" and "自己出错" in err and len(entries) == 1 and entries[0].get("outcome") == "只记"
              and "自证造的出错" in " ".join(entries[0].get("findings") or []),
              f"自己出错应当照常放行、stderr 写出错、检出记录多一行（结局只记），实际退 {code}，stdout {out!r}，stderr {err[-600:]!r}，检出 {entries}")
        # 起钩子本身（不带弄坏开关）：判不了的照常退 0、标准输出为空，stderr 写没判与检出记进哪里，检出记录多一行
        s = scenario("wrapper-unjudged")
        detections = s.environment()[DETECTIONS_VARIABLE]
        completed = subprocess.run(["bash", HOOK_PATH], input="不是 JSON", capture_output=True, text=True, env=dict(clean_environment, **s.environment()))
        entries = detection_entries(detections)
        check("wrapper-unjudged-skip-recorded",
              completed.returncode == 0 and completed.stdout == "" and "没判" in completed.stderr and "检出记进" in completed.stderr
              and "不是 JSON" in completed.stderr and len(entries) == 1 and entries[0].get("session_id") == "读不到",
              f"读不出的输入应当退 0、标准输出为空、stderr 写没判、输入开头与检出记进哪里、检出记录多一行（会话号读不到），实际退 {completed.returncode}，"
              f"stdout {completed.stdout!r}，stderr {completed.stderr!r}，检出 {entries}")
        # 包装：扫到上限没看完、没有残留时钩子本身退 0、标准输出为空、stderr 打出那一句，检出记录多一行；不设上限（判全了）的放行什么都不打、不记
        s = scenario("wrapper-truncated")
        deep = s.owned("w-wrap-deep")
        for name in ("a", "b", "c", "d", "e", "f"):
            os.makedirs(os.path.join(deep, name, "inner"))
        detections = s.environment()[DETECTIONS_VARIABLE]
        completed = subprocess.run(["bash", HOOK_PATH], input=json.dumps(s.stop_input()), capture_output=True, text=True,
                                   env=dict(clean_environment, **s.environment(), **{SCAN_LIMIT_VARIABLE: "3"}))
        entries = detection_entries(detections)
        check("wrapper-truncated-pass-stderr",
              completed.returncode == 0 and completed.stdout == "" and f"看了 3 个目录到顶，这几处没看完：{deep}" in completed.stderr
              and len(entries) == 1 and entries[0].get("outcome") == "只记",
              f"扫到上限没看完、没有残留时应当退 0、stderr 打出没看完的几处、检出记录多一行，实际退 {completed.returncode}，stdout {completed.stdout!r}，"
              f"stderr {completed.stderr!r}，检出 {entries}")
        plain_environment = {key: value for key, value in clean_environment.items() if key != SCAN_LIMIT_VARIABLE}
        completed = subprocess.run(["bash", HOOK_PATH], input=json.dumps(s.stop_input()), capture_output=True, text=True,
                                   env=dict(plain_environment, **s.environment()))
        check("wrapper-plain-pass-silent",
              completed.returncode == 0 and completed.stdout == "" and completed.stderr == "" and len(detection_entries(detections)) == 1,
              f"判全了、没有残留的放行应当什么都不打、不记检出，实际退 {completed.returncode}，stdout {completed.stdout!r}，stderr {completed.stderr!r}，"
              f"检出 {detection_entries(detections)}")
        # 平常跑设了弄坏开关：退 2、点名变量只在自证里认（guard-off 设着时探的值只写关守卫项一项：同写会被守卫拒，单独一项才照认，这一格红）
        probe = GUARD_OFF_ITEM if broken(GUARD_OFF_ITEM) else "build-directory-unrecognized"
        completed = subprocess.run(["bash", HOOK_PATH, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True,
                                   env=dict(clean_environment, **{BREAK_VARIABLE: probe}))
        check("break-switch-refused-outside-selftest", completed.returncode == 2 and BREAK_VARIABLE in completed.stderr and "自证" in completed.stderr,
              f"平常跑设了 {BREAK_VARIABLE} 应当退 2 并点名它只在自证里认，实际退 {completed.returncode}，stderr {completed.stderr!r}")
        # 关守卫项与别的项同写：起钩子本身平常跑退 2、点名关守卫项只在唯一一项时认；自证进程里调守卫探同写也拒（guard-off-with-others 下照认，进程里那一格红）
        combined = f"{GUARD_OFF_ITEM},build-directory-unrecognized"
        completed = subprocess.run(["bash", HOOK_PATH, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True,
                                   env=dict(clean_environment, **{BREAK_VARIABLE: combined}))
        check("guard-off-combined-refused-outside-selftest",
              completed.returncode == 2 and GUARD_OFF_ITEM in completed.stderr and "唯一一项" in completed.stderr and BREAK_VARIABLE in completed.stderr,
              f"平常跑设了 {BREAK_VARIABLE}={combined} 应当退 2 并点名关守卫项 {GUARD_OFF_ITEM} 只在它是唯一一项时认，实际退 {completed.returncode}，stderr {completed.stderr!r}")
        refused_in_process = io.StringIO()
        try:
            with contextlib.redirect_stderr(refused_in_process):
                refuse_break_outside_selftest(combined, ["--force", "--break-guard-probe-unknown-option"])
            stop_code = None
        except SystemExit as stop:
            stop_code = stop.code
        check("guard-off-combined-refused-in-process", stop_code == 2 and "唯一一项" in refused_in_process.getvalue(),
              f"自证进程里拿 {combined} 与平常跑的参数调守卫应当退 2 并点名关守卫项只在唯一一项时认，实际 {stop_code}，stderr {refused_in_process.getvalue()!r}")
        # 引导载不全任何一份部分文件（读不到、编不过、截空）：拷一份钩子目录（入口加部分文件），删掉、写坏或截成空文件其中一份，先读 hook 输入，主 agent 第一回的 Stop 按缺依赖的处置拒
        # （退 2、stderr 点名那一份），不抛错退 1、不静默放行；续跑回来的 Stop（stop_hook_active 为真）、SubagentStop 与交回的 PreToolUse 放行并记一条检出，免得卡死；
        # 开关 part-unreadable-passes 下引导照原样抛错、进程退 1，这几格红；part-unreadable-always-refuses 下引导不先读输入、一律退 2，放行的两格红
        bootstrap_hooks = os.path.join(base, "bootstrap-copy", ".claude", "hooks")
        os.makedirs(bootstrap_hooks)
        bootstrap_detections = os.path.join(base, "bootstrap-detections.jsonl")
        bootstrap_stop = {"hook_event_name": "Stop", "session_id": "bootstrap-part"}
        def damaged_bootstrap_copy(part_name, damage):
            for copied_name in ("completion-cleanup-guard.sh", "lib_work_root.py") + COMPLETION_CLEANUP_PART_NAMES:
                shutil.copy(os.path.join(HOOK_DIR, copied_name), bootstrap_hooks)
            if damage == "missing":
                os.remove(os.path.join(bootstrap_hooks, part_name))
            else:
                with open(os.path.join(bootstrap_hooks, part_name), "w", encoding="utf-8") as damaged_part:
                    damaged_part.write("" if damage == "emptied" else "def broken(:\n")
        def run_bootstrap_copy(hook_input):
            return subprocess.run(["bash", os.path.join(bootstrap_hooks, "completion-cleanup-guard.sh")], input=json.dumps(hook_input), capture_output=True, text=True,
                                  env=dict(os.environ, **{SELFTEST_MARK_VARIABLE: SELFTEST_MARK_CONTENT, DETECTIONS_VARIABLE: bootstrap_detections}))
        def refused_and_named(completed, part_name):
            return completed.returncode == 2 and f"部分文件 {os.path.join(bootstrap_hooks, part_name)}" in completed.stderr and "按拒判" in completed.stderr
        bootstrap_problems = []
        for part_name in COMPLETION_CLEANUP_PART_NAMES:
            for damage, damage_label in (("missing", "读不到"), ("syntax-error", "编不过")):
                damaged_bootstrap_copy(part_name, damage)
                completed = run_bootstrap_copy(bootstrap_stop)
                if not refused_and_named(completed, part_name):
                    bootstrap_problems.append(f"{damage_label} {part_name}：退 {completed.returncode}，stderr {completed.stderr[-300:]!r}")
        check("bootstrap-part-unreadable-refused", not bootstrap_problems,
              f"读不到、编不过任何一份部分文件，主 agent 第一回的 Stop 都应当按拒判（退 2、stderr 点名那一份），实际 {bootstrap_problems}")
        for cell, part_name in (("bootstrap-first-part-emptied-refused", COMPLETION_CLEANUP_PART_NAMES[0]), ("bootstrap-main-part-emptied-refused", COMPLETION_CLEANUP_PART_NAMES[-1])):
            damaged_bootstrap_copy(part_name, "emptied")
            completed = run_bootstrap_copy(bootstrap_stop)
            check(cell, refused_and_named(completed, part_name),
                  f"{part_name} 截成空文件（编得过、什么都不定义）应当按拒判（退 2、stderr 点名那一份），不静默退 0、不抛错退 1，实际退 {completed.returncode}，stderr {completed.stderr[-300:]!r}")
        damaged_bootstrap_copy(COMPLETION_CLEANUP_PART_NAMES[0], "missing")
        for cell, released_inputs in (
                ("bootstrap-part-unreadable-stop-hook-active-released", [dict(bootstrap_stop, stop_hook_active=True)]),
                ("bootstrap-part-unreadable-subagent-stop-released", [{"hook_event_name": "SubagentStop", "session_id": "bootstrap-part", "agent_id": "agent-x"},
                                                                       {"hook_event_name": "PreToolUse", "tool_name": "SubagentHandback", "session_id": "bootstrap-part", "agent_id": "agent-x"}])):
            released = []
            for released_input in released_inputs:
                before = detection_entries(bootstrap_detections)
                completed = run_bootstrap_copy(released_input)
                entries = detection_entries(bootstrap_detections)
                released.append((completed.returncode, completed.stdout, len(entries) - len(before),
                                 bool(entries) and entries[-1].get("outcome") == "只记" and all(str(finding).startswith(UNJUDGED_SKIP_PREFIX) for finding in entries[-1].get("findings") or [None])))
            check(cell, released == [(0, "", 1, True)] * len(released_inputs),
                  f"部分文件读不到时，{released_inputs} 都应当放行（退 0、标准输出为空）并各记一条检出（结局只记、findings 以「{UNJUDGED_SKIP_PREFIX}」起头），实际（退出码、标准输出、多记几条、结局与起头对不对）{released}")
    finally:
        for made in scenarios:
            made.stop()
        shutil.rmtree(base, ignore_errors=True)
    return failures, checked[0]


def selftest_cells_main():
    failures, checked = selftest_cells()
    if failures:
        print(f"✗ 自证：{len(failures)} 格红（共判了 {checked} 格）")
        print("→ 怎么办：照上面每一格的下一步改，再单独跑 completion-cleanup-guard.sh --selftest")
        return 1
    print(f"✓ 各格都对（判了 {checked} 格{f'；本次未跑 {len(NOT_RUN)} 格：' + '、'.join(NOT_RUN) if NOT_RUN else ''}）")
    return 0


def run_switch(switch):
    clean = {key: value for key, value in os.environ.items() if key != BREAK_VARIABLE}
    completed = subprocess.run(["bash", HOOK_PATH, "--selftest-cells"], capture_output=True, text=True, env=dict(clean, **{BREAK_VARIABLE: switch}), timeout=300)
    return completed.returncode, completed.stdout + completed.stderr


def selftest():
    if os.environ.get(BREAK_VARIABLE):
        failures, checked = selftest_cells()
        if not failures:
            print(f"✗ 自证：打开弄坏开关 {os.environ[BREAK_VARIABLE]} 之后 {checked} 格仍全绿")
            print("→ 怎么办：那个开关弄坏的那一处要真影响它点名的格；两边对不上就改开关或改格")
        return 1
    clean = {key: value for key, value in os.environ.items() if key != BREAK_VARIABLE}
    completed = subprocess.run(["bash", HOOK_PATH, "--selftest-cells"], capture_output=True, text=True, env=clean, timeout=300)
    if completed.returncode != 0:
        print("✗ 自证：不带弄坏开关时各格应当全绿，实际：")
        print(completed.stdout + completed.stderr)
        print("→ 怎么办：先让 --selftest-cells 不带弄坏开关跑绿，再回来跑 --selftest")
        return 1
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(HOOK_DIR)), "scripts", "hosts"))
    import heavy_threads   # 开关轮的并行度取每台默认重活线程数（.claude/heavy-thread-default），不按核数另定
    try:
        workers = heavy_threads.default_heavy_threads()
    except heavy_threads.HeavyThreadDefaultError as error:
        print(f"✗ 自证：取不到每台默认重活线程数，开关轮没起：{error}")
        print(f"→ 怎么办：{error.howto}")
        return 1
    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool:
        results = list(pool.map(lambda pair: (pair, run_switch(pair[0])), BREAK_TABLE))
    failed = 0
    if len(results) != len(BREAK_TABLE):
        print(f"✗ 自证：派出去 {len(BREAK_TABLE)} 个开关、收回 {len(results)} 个")
        print("→ 怎么办：整轮作废，重跑 --selftest")
        return 1
    for (switch, cell), (code, output) in results:
        if code == 0 or f"✗ 自证 {cell}：" not in output:
            failed += 1
            print(f"✗ 自证：打开弄坏开关 {switch} 之后应当红在「{cell}」格，实际退 {code}，输出：{output[-1500:]!r}")
            print(f"→ 怎么办：{switch} 弄坏的那一处要真影响「{cell}」格的判定；两边对不上就改开关或改格")
    if failed:
        print(f"✗ 自证：{len(BREAK_TABLE)} 个弄坏开关里 {failed} 个没红在点名的格")
        print("→ 怎么办：照上面逐个改，再跑 --selftest")
        return 1
    summary = completed.stdout.strip().splitlines()[-1] if completed.stdout.strip() else ""
    print(f"✓ completion-cleanup-guard.sh 自证：{summary.lstrip('✓ ')}；{len(BREAK_TABLE)} 个弄坏开关各红在点名的格")
    return 0


def main():
    refuse_break_outside_selftest()
    if ARGUMENTS == ["--selftest"]:
        return selftest()
    if ARGUMENTS == ["--selftest-cells"]:
        return selftest_cells_main()
    return hook_main()
