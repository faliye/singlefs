"""K2 ⑤：真会话里在主工作区起的两条命名检查（只起它一条的 Bash）各跑多久：tool_use 与 tool_result 两条记录的时间戳之差。
用法：python3 -I durations.py <项目记录目录>（只读；rerun.sh 不跑它，单跑）"""
import glob, json, sys
from datetime import datetime
projects = sys.argv[1]
rows = []
for path in glob.glob(f"{projects}/*/subagents/agent-*.jsonl"):
    uses = {}
    try:
        handle = open(path, encoding="utf-8", errors="replace")
    except OSError:
        continue
    for line in handle:
        if ("naming-lint" not in line and "code-source-discipline" not in line) and '"tool_result"' not in line:
            continue
        try:
            r = json.loads(line)
        except ValueError:
            continue
        c = (r.get("message") or {}).get("content")
        if not isinstance(c, list):
            continue
        for b in c:
            if not isinstance(b, dict):
                continue
            if b.get("type") == "tool_use" and b.get("name") == "Bash":
                cmd = (b.get("input") or {}).get("command") or ""
                kind = [k for k in ("code-source-discipline.sh --check", "naming-lint.sh") if k in cmd]
                if len(kind) == 1 and ("/home/fy5090/code/singlefs" in cmd) and "grep" not in cmd.split(kind[0])[0][-40:]:
                    uses[b["id"]] = (kind[0], r.get("timestamp"))
            elif b.get("type") == "tool_result" and b.get("tool_use_id") in uses:
                kind, start = uses.pop(b["tool_use_id"])
                try:
                    seconds = (datetime.fromisoformat(r["timestamp"].replace("Z", "+00:00")) - datetime.fromisoformat(start.replace("Z", "+00:00"))).total_seconds()
                except Exception:
                    continue
                rows.append((kind, seconds, r["timestamp"][:10]))
import statistics
for kind in ("code-source-discipline.sh --check", "naming-lint.sh"):
    for since in ("2026-10-15",):
        values = sorted(s for k, s, d in rows if k == kind and d >= since)
        if values:
            print(f"{kind}（主工作区为根、只起它一条、{since} 起）：{len(values)} 次，中位 {statistics.median(values):.0f} 秒，最长 {values[-1]:.0f} 秒，超过 120 秒的 {sum(v > 120 for v in values)} 次")
