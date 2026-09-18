#!/usr/bin/env python3
"""A3/A4 复核：给 agent-watch.py 补一个自检没有覆盖的样本——续做之后又停（第二次 TaskStop）。
不改仓里的 research/scripts/agent-watch.py；只 import 它、直接调用它的函数与既有测试脚手架。
用法：python3 resumed-then-stopped-again-test.py
"""
import importlib.util
import json
import os
import tempfile

REPO = "/home/fy5090/code/singlefs"
spec = importlib.util.spec_from_file_location("agent_watch", os.path.join(REPO, "research/scripts/agent-watch.py"))
aw = importlib.util.module_from_spec(spec)
spec.loader.exec_module(aw)

work = tempfile.mkdtemp(prefix="resumed-then-stopped-again-")
session = os.path.join(work, "session")
agent_id = "resumedstopped"

# 第一段：跑了一下，结束本轮去等后台任务（18 分钟前）
records = [aw.bash_use(20, "t1", "cargo build", "m1"), aw.bash_result(19.9, "t1"),
           aw.record_at(19.5, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")]
aw.write_transcript(session, agent_id, records, {"agentType": "gate-triage", "description": "续做之后又停"})

# 主会话记录：写两次 TaskStop 成功结果——第一次 18 分钟前，第二次 8.8333 分钟前（比后面补的续做记录晚 10 秒）
with open(session + ".jsonl", "w", encoding="utf-8") as handle:
    for minutes_ago in (18, 8.833333333333334):
        use = aw.record_at(minutes_ago, "assistant", [{"type": "tool_use", "id": f"s{minutes_ago}", "name": "TaskStop", "input": {"task_id": agent_id}}],
                            stop_reason="tool_use", message_id=f"ms{minutes_ago}")
        message = f"Successfully stopped task: {agent_id} (样本)"
        result = aw.record_at(minutes_ago - 0.001, "user", [{"tool_use_id": f"s{minutes_ago}", "type": "tool_result",
                                                              "content": json.dumps({"message": message, "task_id": agent_id}, ensure_ascii=False)}])
        result["toolUseResult"] = {"message": message, "task_id": agent_id, "task_type": "local_agent", "command": "样本"}
        handle.write(json.dumps(use, ensure_ascii=False) + "\n")
        handle.write(json.dumps(result, ensure_ascii=False) + "\n")

# 第一次停掉（18 分钟前）之后，子 agent 被续做：又跑了一次，9 分钟前再次结束本轮等后台任务
extra = [aw.record_at(15, "user", "续做：再跑一遍"), aw.bash_use(10, "t2", "cargo test", "m3"), aw.bash_result(9.9, "t2"),
         aw.record_at(9, "assistant", [{"type": "text", "text": "又等后台任务"}], stop_reason="end_turn", message_id="m4")]
path = os.path.join(session, "subagents", f"agent-{agent_id}.jsonl")
with open(path, "a", encoding="utf-8") as handle:
    for record in extra:
        handle.write(json.dumps(record, ensure_ascii=False) + "\n")

transcript = aw.AgentTranscript(agent_id, path)
first_stop = aw.task_stop_time(agent_id, session + ".jsonl")
print(f"task_stop_time 返回（离现在多少秒）: {(__import__('datetime').datetime.now(__import__('datetime').timezone.utc) - first_stop).total_seconds():.1f} 秒前")
print(f"child 最后一条记录时间（离现在多少秒）: {(__import__('datetime').datetime.now(__import__('datetime').timezone.utc) - transcript.last_timestamp).total_seconds():.1f} 秒前")
print(f"state = {transcript.state}")
print(f"is_done() = {transcript.is_done()}")
want_state = "被停"
ok = transcript.state == want_state and transcript.is_done() is True
print("PASS" if ok else "FAIL: 应当是「被停」且 is_done()=True——说明 task_stop_time 没有正确取『最晚一次』TaskStop 记录")
import shutil
shutil.rmtree(work)
