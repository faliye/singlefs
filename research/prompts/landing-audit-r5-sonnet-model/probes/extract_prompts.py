"""把 Claude Code 会话目录里每个子 agent 的首条用户消息（= 派发提示）和主会话里每一次 Agent/Task 派发的 prompt 抽成两份 json。
用法：python3 -I extract_prompts.py <会话目录> <输出目录> <截止秒数>
  会话目录  ~/.claude/projects/-home-fy5090-code-singlefs
  截止秒数  纪元秒（整数）：首条消息 / 派发的记录时刻不早于它的丢掉——会话目录一直在长，不设截止复跑的数会变；用秒数是因为文档里不写钟点
输出：<输出目录>/subagent_prompts.json、<输出目录>/main_prompts.json；打一行 name=extract 条数"""
import datetime, glob, json, os, sys
projects, out, cutoff = sys.argv[1], sys.argv[2], int(sys.argv[3])


def before_cutoff(stamp):
    return bool(stamp) and datetime.datetime.fromisoformat(stamp.replace('Z', '+00:00')).timestamp() < cutoff
os.makedirs(out, exist_ok=True)
subagents = []
for path in sorted(glob.glob(projects + "/*/subagents/agent-*.jsonl")):
    meta = {}
    try:
        meta = json.load(open(path[:-6] + ".meta.json"))
    except Exception:
        pass
    first, stamp = None, ""
    with open(path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            try:
                record = json.loads(line)
            except Exception:
                continue
            if record.get("type") == "user":
                content = record.get("message", {}).get("content")
                if isinstance(content, list):
                    content = "\n".join(part.get("text", "") for part in content if isinstance(part, dict))
                first, stamp = content, record.get("timestamp", "")
                break
    if first is None or not before_cutoff(stamp):
        continue
    subagents.append({"session": path.split("/")[-3], "agent": os.path.basename(path)[6:-6], "type": meta.get("agentType", ""), "ts": stamp, "prompt": first, "path": path})
dispatched = {}
for path in glob.glob(projects + "/*.jsonl"):
    with open(path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if '"tool_use"' not in line:
                continue
            try:
                record = json.loads(line)
            except Exception:
                continue
            content = record.get("message", {}).get("content")
            if not isinstance(content, list):
                continue
            for part in content:
                if isinstance(part, dict) and part.get("type") == "tool_use" and part.get("name") in ("Agent", "Task"):
                    stamp = record.get("timestamp", "")
                    # 续跑的会话文件会把老记录连同老 id 重抄一遍：同一次派发取最早的那条记录，别因文件读的次序不同而变
                    earlier = dispatched.get(part.get("id"))
                    if before_cutoff(stamp) and (earlier is None or stamp < earlier["ts"]):
                        dispatched[part.get("id")] = {"id": part.get("id"), "ts": stamp, "type": part.get("input", {}).get("subagent_type"), "prompt": part.get("input", {}).get("prompt", "")}
json.dump(subagents, open(out + "/subagent_prompts.json", "w"), ensure_ascii=False)
json.dump(list(dispatched.values()), open(out + "/main_prompts.json", "w"), ensure_ascii=False)
print(f"name=extract 子agent首条消息={len(subagents)} 主会话派发={len(dispatched)} 截止秒数={cutoff}")
