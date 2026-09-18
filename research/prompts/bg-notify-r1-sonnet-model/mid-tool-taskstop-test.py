#!/usr/bin/env python3
"""A4 复核：停在工具调用中途（没有 [Request interrupted 标记，只有 pending 的 tool_use），
TaskStop 那条判法是否也会把它判成「被停」——核 agent-watch.py 文件头注释「认两处」是不是真的互斥。"""
import importlib.util
import os
import tempfile

REPO = "/home/fy5090/code/singlefs"
spec = importlib.util.spec_from_file_location("agent_watch", os.path.join(REPO, "research/scripts/agent-watch.py"))
aw = importlib.util.module_from_spec(spec)
spec.loader.exec_module(aw)

work = tempfile.mkdtemp(prefix="mid-tool-taskstop-")
session = os.path.join(work, "session")
agent_id = "midtoolstopped"

# 只有一个 Bash tool_use，没有对应的 tool_result（模拟工具还没跑完就被 TaskStop 停掉），也没有 [Request interrupted 文本
records = [aw.bash_use(2, "t1", "cargo test --release", "m1")]
aw.write_transcript(session, agent_id, records, {"agentType": "experiment-runner", "description": "停在工具中途，没有打断标记"})
aw.write_session_stops(session, {agent_id: 1.9})

path = os.path.join(session, "subagents", f"agent-{agent_id}.jsonl")
transcript = aw.AgentTranscript(agent_id, path)
print(f"state = {transcript.state}, is_done() = {transcript.is_done()}, pending_tool = {transcript.pending_tool}")
import shutil
shutil.rmtree(work)
