#!/usr/bin/env python3
"""A4 复核：同一个会话记录里停过别的 agent id，会不会把这个 id 误判成被停。"""
import importlib.util
import json
import os
import tempfile

REPO = "/home/fy5090/code/singlefs"
spec = importlib.util.spec_from_file_location("agent_watch", os.path.join(REPO, "research/scripts/agent-watch.py"))
aw = importlib.util.module_from_spec(spec)
spec.loader.exec_module(aw)

work = tempfile.mkdtemp(prefix="other-id-stopped-")
session = os.path.join(work, "session")
target = "targetagentxyz"
other = "otheragentabc"

records = [aw.bash_use(5, "t1", "cargo build", "m1"), aw.bash_result(4.9, "t1"),
           aw.record_at(4.5, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")]
aw.write_transcript(session, target, records, {"agentType": "gate-triage", "description": "没被停，另一个 id 被停"})

# 主会话记录：只停 other，不停 target
aw.write_session_stops(session, {other: 2})

transcript = aw.AgentTranscript(target, os.path.join(session, "subagents", f"agent-{target}.jsonl"))
stopped_at = aw.task_stop_time(target, session + ".jsonl")
print(f"task_stop_time(target) = {stopped_at}")
print(f"state = {transcript.state}, is_done() = {transcript.is_done()}")
ok = stopped_at is None and transcript.state != "被停" and transcript.is_done() is False
print("PASS" if ok else "FAIL: 别的 id 的 TaskStop 记录污染了这个 id 的判法")
import shutil
shutil.rmtree(work)
