"""收尾协议的离线回归：模拟 SSH/systemd/cgroup，不连接主机、不启动服务、不使用 GPU。
由 task-server/task_server_lib.py --selftest 的 cleanup-protocol-unit-tests 格起（在那一格自己的进程里跑，不起服务）。
假调度器的那几张表照 server.Scheduler.__init__ 里赋成空表、空集合、锁的那几个名字现建（scheduler_tables），不手抄名单：服务再加表这里不用改。
弄坏开关 TASK_SERVER_BREAK=cleanup-tests-tables-hand-listed 下照手抄的名单建（漏 activity_windows 这类后加的表），收尾那几条红。
同样不起服务的另两组：attempt_runner.py 存内存最高那一刻的明细（PeakMemoryStatTests，那份文件头 ③；弄坏开关 TASK_SERVER_BREAK=peak-memory-stat-read-at-exit 下每一轮都覆盖，那一条红），
control clear-queue 判停没停与 stop 同一段回读（ClearQueueReadbackTests，control 的 unit_stopped_readback）。
弄坏开关 TASK_SERVER_BREAK=clear-queue-active-state-only 下 clear-queue 只看 is-active，ClearQueueReadbackTests 的 cgroup 里还有进程与读不到那两条红；
TASK_SERVER_BREAK=clear-queue-running-attempts-ignored 下库里有在跑的尝试也照清，ClearQueueReadbackTests 的在跑尝试那一条红。"""
import ast
import contextlib
import copy
import hashlib
import importlib.machinery
import importlib.util
import inspect
import io
import json
import os
import sqlite3
import textwrap
import threading
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import types
import unittest
from unittest.mock import Mock, patch

PROJECT = Path(__file__).resolve().parents[1]
ROOT = PROJECT.parent
sys.path.insert(0, str(PROJECT))

import attempt_runner  # noqa: E402
import task_server as server  # noqa: E402

def load_control():
    """task-server/control（没有 .py 后缀）照源文件载成模块；载入只定义函数，不起子进程、不读库。"""
    loader = importlib.machinery.SourceFileLoader("task_server_control", str(PROJECT / "control"))
    module = importlib.util.module_from_spec(importlib.util.spec_from_loader(loader.name, loader))
    loader.exec_module(module)
    return module


control = load_control()


def server_modules():
    """服务拆成的各份模块：入口 server 加同目录里已经 import 进来、名字以 task_server_ 起头的各份（库与自证本身的各份不算）。"""
    others = [module for name, module in sorted(sys.modules.items())
              if name.startswith("task_server_") and not name.startswith(("task_server_lib", "task_server_selftest"))
              and str(getattr(module, "__file__", "") or "").endswith(".py") and Path(module.__file__).resolve().parent == PROJECT]
    return [server] + others


@contextlib.contextmanager
def patch_server(name, new=unittest.mock.DEFAULT, **keywords):
    """patch.object(server, 名字, …) 在服务拆成几份之后的写法：一个名字在每份模块里各有一个绑定（定义它的那一份、import 它的几份与入口），
    把每一份里有这个名字的绑定都换成同一个替身（没给 new 就建一个 MagicMock(**keywords)，各份共用它的返回值与调用记录），出来时还原。"""
    replacement = unittest.mock.MagicMock(**keywords) if new is unittest.mock.DEFAULT else new
    with contextlib.ExitStack() as stack:
        for module in server_modules():
            if hasattr(module, name):
                stack.enter_context(patch.object(module, name, replacement))
        yield

HAND_LISTED_TABLES = ("log_growth", "log_offsets", "cpu_windows", "unit_checked_at", "unit_gone_since", "peer_memory", "peer_io_wait", "io_waits")   # 只给弄坏开关


def scheduler_tables():
    """server.Scheduler.__init__ 里赋成空表（{}）、空集合（set()）或锁（threading.Lock() / RLock()）的那几个属性 → {名字: 建一份新的那个函数}。
    照源码的语法树读（inspect.getsource + ast），不跑 __init__（它要配置、库与服务）。"""
    tree = ast.parse(textwrap.dedent(inspect.getsource(server.Scheduler.__init__)))
    factories = {}
    for node in ast.walk(tree):
        if not isinstance(node, ast.Assign) or len(node.targets) != 1:
            continue
        target, value = node.targets[0], node.value
        if not (isinstance(target, ast.Attribute) and isinstance(target.value, ast.Name) and target.value.id == "self"):
            continue
        if isinstance(value, ast.Dict) and not value.keys:
            factories[target.attr] = dict
        elif isinstance(value, ast.Call) and isinstance(value.func, ast.Name) and value.func.id == "set" and not value.args:
            factories[target.attr] = set
        elif (isinstance(value, ast.Call) and isinstance(value.func, ast.Attribute) and isinstance(value.func.value, ast.Name)
              and value.func.value.id == "threading" and value.func.attr in ("Lock", "RLock")):
            factories[target.attr] = getattr(threading, value.func.attr)
    return factories


def shell_functions(path, names):
    source = (ROOT / path).read_text()
    return "\n".join(re.search(r"^" + name + r"\(\) \{.*?^\}", source, re.M | re.S)[0]
                     for name in names)


class CleanupProtocolTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def scheduler(self):
        scheduler = object.__new__(server.Scheduler)
        scheduler.environment = {}
        scheduler.labelled_scope = Mock(return_value="")
        scheduler.peer_runner = str(ROOT / "scripts/hosts/multi-host-run.sh")
        scheduler.resources = Mock()
        scheduler.store = Mock()
        scheduler.alert = Mock()
        scheduler.settle_attempt = Mock()
        if server.lib.broken("cleanup-tests-tables-hand-listed"):   # 弄坏开关：照手抄的旧名单建表
            factories = {name: dict for name in HAND_LISTED_TABLES}
        else:
            factories = scheduler_tables()
        for name, factory in factories.items():
            setattr(scheduler, name, factory())
        machine = Mock()
        machine.command_exit.return_value = (0, "local", None)
        machine.ended_peaks.return_value = {}
        scheduler.machines = {"local": machine}
        return scheduler

    def attempt(self):
        return dict(id=1, task_id=2, number=1, state="running", host="local",
                    unit="singlefs-task-regression.service",
                    started_at=1, log=str(self.root / "attempt.log"))

    def test_unreadable_scope_retained_across_scheduler_recreation(self):
        attempt = self.attempt()
        task = dict(id=2, state="running")
        result = dict(exit_code=0, ended_at=2, scope="/sys/fs/cgroup/regression.scope",
                      unit_leftovers=dict(processes=[], remaining=[], verified=True))
        scheduler = self.scheduler()
        scheduler.store.task.return_value = (task, True)
        with patch_server("cgroup_processes", return_value=None), patch_server("say"):
            scheduler.finish_attempt(task, attempt, result)
        self.assertEqual(attempt["state"], "running")
        self.assertIn("cleanup_pending", attempt)
        scheduler.settle_attempt.assert_not_called()
        # 模拟服务重启后从库恢复，而不是依赖进程内的 blocked 集合。
        restored = self.scheduler()
        persisted = copy.deepcopy(attempt)
        restored.store.select.return_value = [(persisted, True)]
        restored.store.task.return_value = (task, True)
        persisted["cleanup_retry_at"] = 0
        with patch_server("cgroup_processes", return_value=[]), patch_server("say"):
            restored.finish_attempt(task, persisted, result)
        self.assertEqual(persisted["state"], "ended")
        self.assertNotIn("cleanup_pending", persisted)
        restored.settle_attempt.assert_called_once()

    def test_surviving_scope_blocks_then_releases(self):
        scheduler = self.scheduler()
        result = dict(scope="/sys/fs/cgroup/regression.scope",
                      unit_leftovers=dict(processes=[], remaining=[], verified=True))
        with patch_server("cgroup_processes", return_value=["123"]), \
                patch.object(server.subprocess, "run"), patch_server("LEFTOVER_STOP_SECONDS", -1), \
                patch_server("say"):
            clean, note = scheduler.stop_leftovers(self.attempt(), result)
        self.assertFalse(clean)
        self.assertIn("保留资源占用", note)
        with patch_server("cgroup_processes", return_value=[]):
            self.assertTrue(scheduler.stop_leftovers(self.attempt(), result)[0])

    def test_scope_read_failure_after_kill_is_not_empty(self):
        with patch_server("cgroup_processes", side_effect=[["123"], None]), \
                patch.object(server.subprocess, "run"):
            clean, _ = self.scheduler().stop_leftovers(
                self.attempt(), dict(scope="/sys/fs/cgroup/regression.scope",
                                     unit_leftovers=dict(verified=True, remaining=[])))
        self.assertFalse(clean)

    def test_unit_snapshot_is_rechecked_without_killing_a_stale_pid(self):
        result = dict(unit_leftovers=dict(processes=["123"], remaining=["123"], verified=True))
        with patch_server("unit_processes", return_value=[]), \
                patch.object(server.subprocess, "run") as run, patch_server("say"):
            clean, _ = self.scheduler().stop_leftovers(self.attempt(), result)
        self.assertTrue(clean)
        run.assert_not_called()

    def test_unit_read_failure_is_not_empty(self):
        with patch_server("unit_processes", return_value=None):
            self.assertFalse(self.scheduler().stop_leftovers(self.attempt(), None)[0])

    def test_missing_scope_location_is_unknown(self):
        scheduler = self.scheduler()
        scheduler.labelled_scope.return_value = None
        self.assertFalse(scheduler.stop_leftovers(
            self.attempt(), dict(unit_leftovers=dict(verified=True, remaining=[])))[0])

    def test_runner_does_not_report_permission_error_as_empty(self):
        with patch.object(attempt_runner, "cgroup_of", return_value="/regression.service"), \
                patch("builtins.open", side_effect=PermissionError("denied")):
            result = attempt_runner.stop_unit_leftovers("regression.service")
        self.assertFalse(result["verified"])
        self.assertIsNone(result["remaining"])

    def test_missing_procs_in_existing_directory_is_unknown(self):
        self.assertIsNone(server.cgroup_processes(str(self.root)))
        self.assertEqual(server.cgroup_processes(str(self.root / "gone")), [])

    def test_peer_needs_durable_confirmation_not_successful_recovery_exit(self):
        scheduler = self.scheduler()
        attempt = self.attempt()
        marker = self.root / "peer-cleanup.json"
        attempt["peer_cleanup_file"] = str(marker)
        marker.write_text(json.dumps(dict(status="pending")))
        with patch.object(server.subprocess, "run", return_value=types.SimpleNamespace(
                returncode=0, stdout="", stderr="")):
            self.assertFalse(scheduler.confirm_peer_cleanup(attempt)[0])
        marker.write_text(json.dumps(dict(status="confirmed")))
        with patch.object(server.subprocess, "run") as run:
            self.assertTrue(scheduler.confirm_peer_cleanup(attempt)[0])
            run.assert_not_called()

    def test_missing_peer_receipt_keeps_attempt_pending(self):
        attempt = self.attempt()
        attempt["peer_cleanup_file"] = str(self.root / "missing")
        self.assertFalse(self.scheduler().confirm_peer_cleanup(attempt)[0])

    def test_peer_recovery_timeout_keeps_attempt_pending(self):
        attempt = self.attempt()
        marker = self.root / "peer-cleanup.json"
        marker.write_text(json.dumps(dict(status="pending")))
        attempt["peer_cleanup_file"] = str(marker)
        with patch.object(server.subprocess, "run", side_effect=subprocess.TimeoutExpired("recovery", 60)):
            self.assertFalse(self.scheduler().confirm_peer_cleanup(attempt)[0])

    def run_shell_cleanup(self, stop_status, holder_status=0, temporary=""):
        """照 multi-host-run.sh 里收尾那几个函数原样跑一趟（ssh 与停进程换成记到 trace 的桩）：temporary 给了的演第二台那一层记下了这条路径的临时目录
        （读记录文件那一回交它），不给的演记录文件不在。→ (bash 的结果, trace 各行, 清理回执)。"""
        definitions = shell_functions("scripts/hosts/multi-host-run.sh", [
            "multi_host_record_cleanup", "multi_host_stop_pending_peer",
            "multi_host_remove_peer_temporary", "multi_host_remove_peer_run", "multi_host_on_exit"])
        program = definitions + r'''
MULTI_HOST_PEER_TEMPORARY_RECORD_NAME=temporary-directory
MULTI_HOST_PEER_DEFAULT_TEMPORARY_ROOT=/tmp
MULTI_HOST_PEER_TEMPORARY_PREFIX=singlefs-run-with-memory-cap-
multi_host_now_microseconds() { echo 123456; }
multi_host_break_is() { return 1; }
multi_host_step() { :; }
multi_host_redact() { cat; }
multi_host_stop_peer_leftovers() { multi_host_leftovers_stopped=0; }
peer_host_stop_detached() { echo stop >> "$trace"; return "$stop_status"; }
peer_host_run() {
  if [[ "$2" == python3* ]]; then echo holders >> "$trace"; return "$test_holder_status"; fi
  if [[ "$2" == "test -e "* ]]; then
    [[ -n "$test_temporary" ]] || return 3
    echo read-temporary >> "$trace"; printf '%s\n' "$test_temporary"; return 0
  fi
  if [[ "$2" == "rm -rf -- /tmp/"* ]]; then echo remove-temporary >> "$trace"; return 0; fi
  echo remove >> "$trace"; return 0
}
trace="$1/trace"; TASK_SERVER_PEER_CLEANUP_FILE="$1/receipt"
stop_status="$2"; test_holder_status="$3"; test_temporary="$4"
tree_root=/repo; run_label=test; peer_start_directory=/work
peer_scratch=/builds/test; peer_pid_file=/runs/test.pid
peer_exit_file=/runs/test.exit; peer_output_file=/runs/test.log
peer_command_started=1; peer_run_pending=1; multi_host_cleanup_failed=0
multi_host_scratch=""
multi_host_record_cleanup pending starting
trap multi_host_on_exit EXIT
exit 255
'''
        completed = subprocess.run(["bash", "-c", program, "cleanup-test", str(self.root),
                                    str(stop_status), str(holder_status), temporary], capture_output=True, text=True)
        return completed, (self.root / "trace").read_text().splitlines(), json.loads((self.root / "receipt").read_text())

    def test_confirmed_cleanup_removes_only_own_peer_temporary(self):
        """确认收干净之后删第二台那一层没走到 EXIT 留下的临时目录（multi-host-run.sh 的 multi_host_remove_peer_temporary）：记下的路径是默认临时根下直接一层、
        内存包装起的名字的，读出来、删掉，再删这一趟的本地目录；不是那种名字的照留，只删本地目录；两种都不报找不到函数。"""
        cases = (("/tmp/singlefs-run-with-memory-cap-123-abc", ["stop", "holders", "read-temporary", "remove-temporary", "remove"]),
                 ("/home/somebody/keep", ["stop", "holders", "read-temporary", "remove"]))
        for recorded, expected in cases:
            with self.subTest(recorded=recorded):
                (self.root / "trace").unlink(missing_ok=True)
                completed, trace, marker = self.run_shell_cleanup(0, 0, recorded)
                self.assertEqual(trace, expected)
                self.assertEqual(marker["status"], "confirmed")
                self.assertNotIn("command not found", completed.stderr)

    def test_disconnection_attempts_stop_and_preserves_unknown_run(self):
        completed, trace, marker = self.run_shell_cleanup(1)
        self.assertEqual(completed.returncode, 255)
        self.assertEqual(trace, ["stop"])
        self.assertEqual(marker["status"], "pending")
        self.assertFalse(marker["processes_stopped"])

    def test_confirmed_stop_precedes_readback_and_removal(self):
        _, trace, marker = self.run_shell_cleanup(0)
        self.assertEqual(trace, ["stop", "holders", "remove"])
        self.assertEqual(marker["status"], "confirmed")

    def test_unreadable_or_occupied_build_preserves_stopped_receipt(self):
        for status in (1, 3):
            with self.subTest(status=status):
                (self.root / "trace").unlink(missing_ok=True)
                _, trace, marker = self.run_shell_cleanup(0, status)
                self.assertEqual(trace, ["stop", "holders"])
                self.assertEqual(marker["status"], "pending")
                self.assertTrue(marker["processes_stopped"])

    def test_missing_parent_without_exit_receipt_is_checked_by_session(self):
        """起的那个 bash 不在、没有退出回执：照会话号核过一个不剩就判已停、补写退出回执（peer-host-lib.sh 文件头「停下的退出回执」）；
        进程号文件读不出启动时刻的核不了、带 keep（改前的样子）的不核，都不按已停处理；有回执的照收、回执不改。"""
        definitions = shell_functions("scripts/hosts/peer-host-lib.sh", [
            "peer_host_signal_one", "peer_host_process_is_alive", "peer_host_process_tree",
            "peer_host_stop_process_tree", "peer_host_attempt_leftovers", "peer_host_stop_attempt_leftovers",
            "peer_host_stop_detached"])
        (self.root / "pid").write_text("999999999\n")
        program = definitions + r'''
peer_host_run() { ( cd "$1" && bash -c "$2" ); }
peer_host_stop_detached "$1" "$1/pid" "$1/exited" "$3" "$2"
'''
        exited = self.root / "exited"
        cases = (("没有启动时刻、没给标签", None, "", 1, None), ("改前的样子", "1", "keep", 1, None),
                 ("会话里一个不剩", "1", "", 0, "143 stopped already-gone"), ("已有回执", "1", "", 0, "0"))
        for name, start, mode, expected, receipt in cases:
            label = ""
            with self.subTest(case=name):
                if start is None:
                    (self.root / "pid.start").unlink(missing_ok=True)
                else:
                    (self.root / "pid.start").write_text(start + "\n")
                if name == "已有回执":
                    exited.write_text("0\n")
                completed = subprocess.run(["bash", "-c", program, "stop-test", str(self.root), mode, label],
                                           capture_output=True, text=True, timeout=60)
                self.assertEqual(completed.returncode, expected, completed.stdout + completed.stderr)
                self.assertEqual(exited.read_text().strip() if exited.exists() else None, receipt)
        # 起的那一步没连上、进程号文件没写成（任务 16204）：给了尝试标签的只照标签核，一个不剩就补写回执、按没起来收
        exited.unlink(missing_ok=True)
        (self.root / "pid").unlink(missing_ok=True)
        (self.root / "pid.start").unlink(missing_ok=True)
        completed = subprocess.run(["bash", "-c", program, "stop-test", str(self.root), "", "selftest-label-never-started"],
                                   capture_output=True, text=True, timeout=60)
        self.assertEqual(completed.returncode, 0, completed.stdout + completed.stderr)
        self.assertEqual(exited.read_text().strip() if exited.exists() else None, "143 stopped already-gone")

    def test_recovery_after_wrapper_exit_uses_saved_stop_receipt(self):
        self.run_shell_cleanup(0, 1)  # 已停树，但第一次回读不可用；模拟包装已退出。
        (self.root / "trace").unlink()
        definitions = shell_functions("scripts/hosts/multi-host-run.sh", [
            "multi_host_record_cleanup", "multi_host_stop_pending_peer",
            "multi_host_remove_peer_run", "multi_host_recover_cleanup"])
        program = definitions + r'''
multi_host_now_microseconds() { echo 123457; }
multi_host_break_is() { return 1; }
multi_host_step() { :; }
multi_host_redact() { cat; }
peer_host_load_configuration() { PEER_HOST_BUILD_ROOT=/; }
multi_host_stop_peer_leftovers() { multi_host_leftovers_stopped=0; }
peer_host_stop_detached() { echo unexpected-stop >> "$trace"; return 1; }
peer_host_run() {
  if [[ "$2" == python3* ]]; then echo holders >> "$trace"; else echo remove >> "$trace"; fi
  return 0
}
trace="$1/trace"
multi_host_script_directory="$2/scripts/hosts"
multi_host_recover_cleanup "$1/receipt"
'''
        completed = subprocess.run(["bash", "-c", program, "recovery-test", str(self.root), str(ROOT)],
                                   capture_output=True, text=True)
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual((self.root / "trace").read_text().splitlines(), ["holders", "remove"])
        self.assertEqual(json.loads((self.root / "receipt").read_text())["status"], "confirmed")

    def test_internal_recovery_entry_accepts_confirmed_receipt_without_ssh(self):
        marker = self.root / "confirmed.json"
        marker.write_text(json.dumps(dict(status="confirmed")))
        completed = subprocess.run(["bash", str(ROOT / "scripts/hosts/multi-host-run.sh"),
                                    "--recover-cleanup", str(marker)],
                                   capture_output=True, text=True, timeout=20)
        self.assertEqual(completed.returncode, 0, completed.stderr)


class PeakMemoryStatTests(unittest.TestCase):
    """attempt_runner.Sampler 每轮读 scope 的 memory.current，比见过的最高还高才存一份 memory.stat（attempt_runner.py 文件头 ③），不起子进程。"""

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def write_scope(self, current, anon, file_pages):
        (self.root / "memory.current").write_text(f"{current}\n", encoding="utf-8")
        (self.root / "memory.stat").write_text(f"anon {anon}\nfile {file_pages}\nfile_dirty 0\nfile_writeback 0\nshmem 0\nslab 7\n", encoding="utf-8")

    def test_keeps_the_stat_of_the_highest_reading(self):
        specification = {"directory": str(self.root), "log": "", "memory_stat_at_exit": server.lib.broken("peak-memory-stat-read-at-exit")}
        sampler = attempt_runner.Sampler(specification, os.getpid(), None)
        self.write_scope(1000, 900, 100)
        sampler.note_memory_stat(str(self.root))
        self.write_scope(5000, 4900, 100)
        sampler.note_memory_stat(str(self.root))
        self.write_scope(4096, 0, 0)   # 被杀之后：内存已释放
        sampler.note_memory_stat(str(self.root))
        self.assertEqual(sampler.memory_stat_at_peak, {"current": 5000, "anon": 4900, "file": 100, "file_dirty": 0, "file_writeback": 0, "shmem": 0})

    def test_unreadable_scope_keeps_nothing(self):
        sampler = attempt_runner.Sampler({"directory": str(self.root), "log": ""}, os.getpid(), None)
        sampler.note_memory_stat(str(self.root))
        self.assertIsNone(sampler.memory_stat_at_peak)


class ClearQueueReadbackTests(unittest.TestCase):
    """control clear-queue 判停没停与 stop 同一段回读（control 的 unit_stopped_readback）：is-active 与 cgroup.procs 换成给定的回读，库是临时目录里的一份，
    表照服务的 Store.initialize 现建（与服务同一份列，clear-queue 读 attempts 的 state 列）。库里的尝试缺省都已收尾（ended）；在跑尝试那两条另记一次 running：
    有在跑的尝试 clear-queue 拒、库一个字节不动、不拷备份（弄坏开关 clear-queue-running-attempts-ignored 下照清，那一条红）。"""
    QUEUE_ROWS = 2

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.database = self.root / "queue.sqlite"
        store = server.Store(str(self.database), bytes(32), {})
        store.initialize()
        connection = store.connection()
        for table, state in (("tasks", "finished"), ("attempts", "ended")):
            connection.executemany(f"INSERT INTO {table}(task_id, state, session, command_key, record, signature) VALUES (?, ?, '', '', '{{}}', '')",
                                   [(number, state) for number in range(1, self.QUEUE_ROWS + 1)])
        connection.execute("INSERT INTO history(task_id, state, session, command_key, record, signature) VALUES (0, 'pin', '', '', '{}', '')")
        connection.execute("PRAGMA wal_checkpoint(TRUNCATE)")
        connection.close()
        store.local.connection = None

    def mark_attempt_running(self):
        """把一次尝试改记 running（停服务时还在各自单元里跑的那种），→ 改完之后库文件的 sha256。"""
        connection = sqlite3.connect(self.database)
        with connection:
            connection.execute("UPDATE attempts SET state = 'running' WHERE task_id = 1")
        connection.execute("PRAGMA wal_checkpoint(TRUNCATE)")
        connection.close()
        return self.database_digest()

    def database_digest(self):
        return hashlib.sha256(self.database.read_bytes()).hexdigest()

    def clear(self, state, processes):
        """→ (clear_queue 的退出码, 它打的标准错误)。"""
        configuration = types.SimpleNamespace(database=str(self.database))
        error = io.StringIO()
        with patch.object(control, "unit_active_state", return_value=state), patch.object(control, "unit_processes", return_value=processes), \
                patch.object(control.lib, "read_configuration", return_value=configuration), patch.object(control, "say"), \
                contextlib.redirect_stderr(error):
            code = control.clear_queue()
        return code, error.getvalue()

    def rows(self):
        connection = sqlite3.connect(self.database)
        try:
            return {table: connection.execute(f"SELECT COUNT(*) FROM {table}").fetchone()[0] for table in ("tasks", "attempts", "history")}
        finally:
            connection.close()

    def backups(self):
        return sorted(self.root.glob("queue.sqlite.before-clear-*"))

    def assert_refused_untouched(self, code, error):
        self.assertEqual(code, 1, error)
        self.assertEqual(self.rows(), {"tasks": self.QUEUE_ROWS, "attempts": self.QUEUE_ROWS, "history": 1}, "拒了就一行不删")
        self.assertEqual(self.backups(), [], "拒了就不拷备份")

    def test_clear_queue_refuses_while_unit_cgroup_still_has_processes(self):
        code, error = self.clear("inactive", ["4242"])
        self.assert_refused_untouched(code, error)
        self.assertIn("cgroup 里进程 1 个", error)

    def test_clear_queue_refuses_when_unit_cgroup_is_unreadable(self):
        code, error = self.clear("failed", None)
        self.assert_refused_untouched(code, error)
        self.assertIn("cgroup 里进程 读不到 个", error)

    def test_clear_queue_refuses_while_unit_is_deactivating(self):
        code, error = self.clear("deactivating", [])
        self.assert_refused_untouched(code, error)
        self.assertIn("「deactivating」", error)

    def test_clear_queue_refuses_while_attempts_still_running(self):
        before = self.mark_attempt_running()
        code, error = self.clear("inactive", [])
        self.assert_refused_untouched(code, error)
        self.assertEqual(self.database_digest(), before, "拒了库一个字节不动")
        self.assertIn("1 次尝试在跑（任务 1）", error)

    def test_clear_queue_clears_once_unit_is_inactive_and_cgroup_empty(self):
        """尝试都已收尾（setUp 记的 ended）、服务停干净：照清。"""
        code, error = self.clear("inactive", [])
        self.assertEqual(code, 0, error)
        self.assertEqual(self.rows(), {"tasks": 0, "attempts": 0, "history": 1})
        self.assertEqual(len(self.backups()), 1)


if __name__ == "__main__":
    unittest.main()
