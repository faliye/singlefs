"""导入仓里的 agent-cost.py（只读），跑 outcome 的同一个 collect_outcome，另把每个子 agent 的写入行数、落地行数、点到与否落盘，供门槛换值复算。"""
import datetime, importlib.util, json, sys, os
repo = sys.argv[1]
sys.path.insert(0, os.path.join(repo, "scripts", "agents"))
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("agent_cost", os.path.join(repo, "scripts/agents/agent-cost.py"))
cost = importlib.util.module_from_spec(spec); sys.modules["agent_cost"] = cost; spec.loader.exec_module(cost)
since_day, until_day, workers, out_prefix = sys.argv[2], sys.argv[3], int(sys.argv[4]), sys.argv[5]
UTC = datetime.timezone.utc
since = datetime.datetime.fromisoformat(since_day).replace(tzinfo=UTC)
until = datetime.datetime.fromisoformat(until_day).replace(tzinfo=UTC) + datetime.timedelta(days=1)
original = cost.outcome_bucket
def recording(agent, used_main, used_any, now):
    agent["used_main"], agent["used_any"] = used_main, used_any
    return original(agent, used_main, used_any, now)
cost.outcome_bucket = recording
now = datetime.datetime.now(UTC)
agents = cost.collect_outcome(cost.DEFAULT_ROOT, since, cost.REPOSITORY_ROOT, now, workers, until)
open(out_prefix + "-summary.txt", "w").write("\n".join(cost.outcome_lines(agents, since, False, until_day)) + "\n")
with open(out_prefix + "-agents.jsonl", "w") as handle:
    for a in agents:
        handle.write(json.dumps({k: a[k] for k in ("agent", "type", "description", "handed_back", "cut", "bucket", "cost", "rework", "written", "landed", "used_main", "used_any")} | {"start": a["start"].isoformat(), "end": a["end"].isoformat() if a["end"] else None}, ensure_ascii=False) + "\n")
print("agents", len(agents), "now", now.isoformat())
