#!/usr/bin/env python3
"""A4 复核：主会话记录文件不在 <会话目录>.jsonl 时（文件缺失），看门狗判法退化成什么，会不会崩。"""
import importlib.util
import os
import tempfile

REPO = "/home/fy5090/code/singlefs"
spec = importlib.util.spec_from_file_location("agent_watch", os.path.join(REPO, "research/scripts/agent-watch.py"))
aw = importlib.util.module_from_spec(spec)
spec.loader.exec_module(aw)

work = tempfile.mkdtemp(prefix="missing-parent-session-")
session = os.path.join(work, "session")
agent_id = "nopeerentfile"
records = [aw.bash_use(20, "t1", "cargo build", "m1"), aw.bash_result(19.9, "t1"),
           aw.record_at(19.5, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")]
aw.write_transcript(session, agent_id, records, {"agentType": "gate-triage", "description": "主会话记录文件缺失"})
# 故意不写 session + ".jsonl"
path = os.path.join(session, "subagents", f"agent-{agent_id}.jsonl")
try:
    transcript = aw.AgentTranscript(agent_id, path)
    print(f"没崩：state = {transcript.state}，is_done() = {transcript.is_done()}")
except Exception as error:
    print(f"崩了：{type(error).__name__}: {error}")
import shutil
shutil.rmtree(work)
