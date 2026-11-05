"""收尾协议的离线回归：模拟 SSH/systemd/cgroup，不连接主机、不启动服务、不使用 GPU。"""
import copy
import json
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
        for name in ("log_growth", "log_offsets", "cpu_windows", "unit_checked_at", "unit_gone_since"):
            setattr(scheduler, name, {})
        machine = Mock()
        machine.command_exit.return_value = (0, "local", None)
        machine.ended_peaks.return_value = {}
        scheduler.machines = {"local": machine}
        return scheduler

    def attempt(self):
        return dict(id=1, task_id=2, number=1, state="running", host="local",
                    unit="singlefs-task-regression.service", card="192.0.2.2:0:16G",
                    started_at=1, log=str(self.root / "attempt.log"))

    def test_unreadable_scope_retains_card_across_scheduler_recreation(self):
        attempt = self.attempt()
        task = dict(id=2, state="running")
        result = dict(exit_code=0, ended_at=2, scope="/sys/fs/cgroup/regression.scope",
                      unit_leftovers=dict(processes=[], remaining=[], verified=True))
        scheduler = self.scheduler()
        scheduler.store.task.return_value = (task, True)
        with patch.object(server, "cgroup_processes", return_value=None), patch.object(server, "say"):
            scheduler.finish_attempt(task, attempt, result)
        self.assertEqual(attempt["state"], "running")
        self.assertIn("cleanup_pending", attempt)
        scheduler.settle_attempt.assert_not_called()
        # 模拟服务重启后从库恢复，而不是依赖进程内的 blocked 集合。
        restored = self.scheduler()
        persisted = copy.deepcopy(attempt)
        restored.store.select.return_value = [(persisted, True)]
        self.assertIn(server.card_key(attempt["card"]), restored.busy_cards())
        restored.store.task.return_value = (task, True)
        persisted["cleanup_retry_at"] = 0
        with patch.object(server, "cgroup_processes", return_value=[]), patch.object(server, "say"):
            restored.finish_attempt(task, persisted, result)
        self.assertEqual(persisted["state"], "ended")
        self.assertNotIn("cleanup_pending", persisted)
        restored.settle_attempt.assert_called_once()

    def test_surviving_scope_blocks_then_releases(self):
        scheduler = self.scheduler()
        result = dict(scope="/sys/fs/cgroup/regression.scope",
                      unit_leftovers=dict(processes=[], remaining=[], verified=True))
        with patch.object(server, "cgroup_processes", return_value=["123"]), \
                patch.object(server.subprocess, "run"), patch.object(server, "LEFTOVER_STOP_SECONDS", -1), \
                patch.object(server, "say"):
            clean, note = scheduler.stop_leftovers(self.attempt(), result)
        self.assertFalse(clean)
        self.assertIn("保留资源占用", note)
        with patch.object(server, "cgroup_processes", return_value=[]):
            self.assertTrue(scheduler.stop_leftovers(self.attempt(), result)[0])

    def test_scope_read_failure_after_kill_is_not_empty(self):
        with patch.object(server, "cgroup_processes", side_effect=[["123"], None]), \
                patch.object(server.subprocess, "run"):
            clean, _ = self.scheduler().stop_leftovers(
                self.attempt(), dict(scope="/sys/fs/cgroup/regression.scope",
                                     unit_leftovers=dict(verified=True, remaining=[])))
        self.assertFalse(clean)

    def test_unit_snapshot_is_rechecked_without_killing_a_stale_pid(self):
        result = dict(unit_leftovers=dict(processes=["123"], remaining=["123"], verified=True))
        with patch.object(server, "unit_processes", return_value=[]), \
                patch.object(server.subprocess, "run") as run, patch.object(server, "say"):
            clean, _ = self.scheduler().stop_leftovers(self.attempt(), result)
        self.assertTrue(clean)
        run.assert_not_called()

    def test_unit_read_failure_is_not_empty(self):
        with patch.object(server, "unit_processes", return_value=None):
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

    def test_missing_peer_receipt_keeps_card(self):
        attempt = self.attempt()
        attempt["peer_cleanup_file"] = str(self.root / "missing")
        self.assertFalse(self.scheduler().confirm_peer_cleanup(attempt)[0])

    def test_peer_recovery_timeout_keeps_card(self):
        attempt = self.attempt()
        marker = self.root / "peer-cleanup.json"
        marker.write_text(json.dumps(dict(status="pending")))
        attempt["peer_cleanup_file"] = str(marker)
        with patch.object(server.subprocess, "run", side_effect=subprocess.TimeoutExpired("recovery", 60)):
            self.assertFalse(self.scheduler().confirm_peer_cleanup(attempt)[0])

    def run_shell_cleanup(self, stop_status, holder_status=0):
        definitions = shell_functions("scripts/hosts/multi-host-run.sh", [
            "multi_host_record_cleanup", "multi_host_stop_pending_peer",
            "multi_host_remove_peer_run", "multi_host_on_exit"])
        program = definitions + r'''
multi_host_now_microseconds() { echo 123456; }
multi_host_break_is() { return 1; }
multi_host_step() { :; }
multi_host_redact() { cat; }
peer_host_stop_detached() { echo stop >> "$trace"; return "$stop_status"; }
peer_host_run() {
  if [[ "$2" == python3* ]]; then echo holders >> "$trace"; return "$test_holder_status"; fi
  echo remove >> "$trace"; return 0
}
trace="$1/trace"; TASK_SERVER_PEER_CLEANUP_FILE="$1/receipt"
stop_status="$2"; test_holder_status="$3"
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
                                    str(stop_status), str(holder_status)], capture_output=True, text=True)
        return completed, (self.root / "trace").read_text().splitlines(), json.loads((self.root / "receipt").read_text())

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

    def test_missing_parent_without_exit_receipt_is_not_confirmation(self):
        definitions = shell_functions("scripts/hosts/peer-host-lib.sh", [
            "peer_host_signal_one", "peer_host_process_is_alive", "peer_host_process_tree",
            "peer_host_stop_process_tree", "peer_host_stop_detached"])
        (self.root / "pid").write_text("999999999\n")
        program = definitions + r'''
peer_host_run() { ( cd "$1" && bash -c "$2" ); }
peer_host_stop_detached "$1" "$1/pid" "$1/exited"
'''
        for receipt, expected in ((False, 1), (True, 0)):
            if receipt:
                (self.root / "exited").write_text("0\n")
            completed = subprocess.run(["bash", "-c", program, "stop-test", str(self.root)],
                                       capture_output=True, text=True)
            self.assertEqual(completed.returncode, expected, completed.stderr)

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


if __name__ == "__main__":
    unittest.main()
