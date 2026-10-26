"""载入一份 agent-cost.py（只读），跑它的 collect_outcome，把每个子 agent 一行落盘（含点到与否、三方腿点名的 -output.md 与认到的判决）。
用法：outcome_dump.py <仓根> <agent-cost.py 路径> <窗口切法 tokyo|utc> <since> <until> <进程数> <输出前缀>"""
import datetime, importlib.util, json, sys, os
repo, script, cut, since_day, until_day, workers, out_prefix = sys.argv[1:8]
sys.dont_write_bytecode = True
for extra in ("scripts/agents", "scripts/hosts", ".claude/scripts"):
    sys.path.insert(0, os.path.join(repo, extra))
spec = importlib.util.spec_from_file_location("agent_cost_probe", script)
cost = importlib.util.module_from_spec(spec); sys.modules["agent_cost_probe"] = cost; spec.loader.exec_module(cost)
UTC = datetime.timezone.utc
zone = UTC if cut == "utc" else __import__("zoneinfo").ZoneInfo("Asia/Tokyo")
since = datetime.datetime.fromisoformat(since_day).replace(tzinfo=zone)
until = datetime.datetime.fromisoformat(until_day).replace(tzinfo=zone) + datetime.timedelta(days=1)
original = cost.outcome_bucket
def recording(agent, used_main, used_any, now):
    agent["used_main"], agent["used_any"] = used_main, used_any
    return original(agent, used_main, used_any, now)
cost.outcome_bucket = recording
root = os.path.join(os.path.expanduser("~/.claude/projects"), os.path.abspath(repo).replace("/", "-"))
now = datetime.datetime.now(UTC)
agents = cost.collect_outcome(root, since, repo, now, int(workers), until)
mention = getattr(cost, "LEG_OUTPUT_MENTION", None)
with open(out_prefix + "-summary.txt", "w") as handle:
    handle.write("\n".join(cost.outcome_lines(agents, since, False, until_day)) + "\n")
with open(out_prefix + "-agents.jsonl", "w") as handle:
    for a in agents:
        row = {k: a.get(k) for k in ("agent", "type", "description", "handed_back", "cut", "bucket", "cost", "rework", "written", "landed", "used_main", "used_any")}
        row["start"] = a["start"].isoformat(); row["end"] = a["end"].isoformat() if a["end"] else None
        row["leg_verdicts"] = a.get("leg_verdicts")
        row["named_outputs"] = sorted(set(mention.findall(a.get("handback_text") or ""))) if (mention and a["type"].startswith("three-way")) else None
        handle.write(json.dumps(row, ensure_ascii=False) + "\n")
print("agents", len(agents), "cut", cut, "since", since.isoformat(), "until", until.isoformat())
