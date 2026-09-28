#!/usr/bin/env python3
"""Synthetic local-leg transcripts fed to agent-watch.py's AgentTranscript + agent_alerts (read-only import of the repo copy)."""
import importlib.util, json, os, shutil, sys, tempfile
from datetime import datetime, timezone
from types import SimpleNamespace
WATCH = sys.argv[1] if len(sys.argv) > 1 else "research/scripts/agent-watch.py"
spec = importlib.util.spec_from_file_location("agent_watch", WATCH); aw = importlib.util.module_from_spec(spec); spec.loader.exec_module(aw)
TH = SimpleNamespace(tool_minutes=8, wait_loop_minutes=3, idle_minutes=10, repeat_count=3, context_tokens=700_000,
                     progress_stale_minutes=30, ask_every_minutes=60, interval_seconds=240)
R, U, BU, BR = aw.record_at, aw.bash_use, aw.bash_result, aw.handback_records
CMD = "bash research/scripts/ask-local.sh research/prompts/zz-local-attack.md > research/prompts/zz-local-attack-output-s1.md"
def res(m, tid, text): return R(m, "user", [{"type": "tool_result", "tool_use_id": tid, "content": text}])
def started(tid, task): return f"Command running in background with ID: {task}. Output is being written to: /nonexistent/tasks/{task}.output. You will be notified when it completes."
def moved(tid, task, s): return f"Command did not complete within its {s}s timeout and was moved to the background (ID: {task}). Output is being written to: /nonexistent/tasks/{task}.output. You will be notified when it completes."
def endturn(m, mid): return R(m, "assistant", [{"type": "text", "text": "waiting for the completion notification"}], stop_reason="end_turn", message_id=mid, usage={"input_tokens": 1})
def notif(m, task, status, rc):
    body = f"<task-notification>\n<task-id>{task}</task-id>\n<status>{status}</status>\n<summary>Background command \"x\" {status} (exit code {rc})</summary>\n</task-notification>"
    return R(m, "user", body, origin_kind="task-notification")
def queued(m, task, status, rc):
    body = f"<task-notification>\n<task-id>{task}</task-id>\n<status>{status}</status>\n<summary>x (exit code {rc})</summary>\n</task-notification>"
    return {"timestamp": R(m, "user", "")["timestamp"], "type": "attachment", "attachment": {"type": "queued_command", "prompt": body}}
disp = R(30, "user", "dispatch prompt; draft dir /tmp/claude-1000/zz-local-attack/")
S = {
 "S1 bg start, end turn, waiting 5 min": [disp, U(20, "t1", CMD, "m1"), res(19.99, "t1", started("t1", "bAAA1")), endturn(5, "m2")],
 "S2 fg moved at 240s, end turn, waiting 5 min": [disp, U(20, "t1", CMD, "m1"), res(16, "t1", moved("t1", "bAAA2", 240)), endturn(5, "m2")],
 "S3 bg exit5 failed -> wake -> stat -> >| rerun bg -> end turn": [disp, U(20, "t1", CMD, "m1"), res(19.9, "t1", started("t1", "bAAA3")), endturn(19.8, "m2"),
      notif(8, "bAAA3", "failed", 5), U(7.9, "t2", "stat -c %s research/prompts/zz-local-attack-output-s1.md", "m3"), res(7.8, "t2", "0"),
      U(7.7, "t3", CMD.replace(" > ", " >| "), "m4"), res(7.6, "t3", started("t3", "bAAA4")), endturn(7.5, "m5")],
 "S4 fg pending 9 min (timeout 600000 path)": [disp, U(9, "t1", CMD, "m1")],
 "S5 fg moved, leg relaunches same cmd in bg, first finishes": [disp, U(20, "t1", CMD, "m1"), res(16, "t1", moved("t1", "bAAA5", 240)),
      U(15.9, "t2", CMD, "m2"), res(15.8, "t2", started("t2", "bAAA6")), endturn(15.7, "m3"), notif(6, "bAAA5", "completed", 0),
      U(5.9, "t3", "ls -l research/prompts/zz-local-attack-output-s1.md", "m4"), res(5.8, "t3", "x"), endturn(5.7, "m5")],
 "S6 control: fast exit3 notice queued mid-turn, leg ends turn claiming wait": [disp, U(20, "t1", CMD, "m1"), res(19.99, "t1", started("t1", "bAAA7")),
      queued(19.98, "bAAA7", "failed", 3), endturn(19.9, "m2")],
}
work = tempfile.mkdtemp(dir="/tmp/claude-1000/sync-local-legs-r1-attack")
try:
    now = datetime.now(timezone.utc)
    for i, (name, records) in enumerate(S.items()):
        agent = f"a{i:03d}simleg0000"
        aw.write_transcript(os.path.join(work, "sess"), agent, records, {"agentType": "three-way-local-attack"})
        t = aw.AgentTranscript(agent, os.path.join(work, "sess", "subagents", f"agent-{agent}.jsonl"))
        alerts = [a[0] for a in aw.agent_alerts(t, now, TH)]
        print(f"{name}\tstate={t.state}\tstarted={t.background_started}\tfinished={sorted(t.background_finished)}\talerts={alerts}")
finally:
    shutil.rmtree(work)
