"""给定几个子 agent 号：打首条提示里报告标签所在的整行，和它对名字带 report 的文件的每次 Write 的工具结果（被拒 / 写成）。
用法：write_refusals.py <会话目录根> <agent 号>..."""
import glob, json, os, re, sys
root = sys.argv[1]
for agent in sys.argv[2:]:
    path = glob.glob(os.path.join(root, "*", "subagents", f"agent-{agent}.jsonl"))[0]
    calls, prompt = {}, None
    print(f"== {agent} {json.load(open(path.replace('.jsonl', '.meta.json'))).get('agentType')}")
    for line in open(path, errors="replace"):
        try: r = json.loads(line)
        except ValueError: continue
        content = (r.get("message") or {}).get("content")
        if prompt is None and r.get("type") == "user":
            prompt = content if isinstance(content, str) else "".join(b.get("text", "") for b in content if b.get("type") == "text")
            for l in prompt.splitlines():
                if "报告" in l and re.search(r"report[^/\s`]*\.\w+", l, re.I):
                    print("   提示行：", l.strip()[:220])
            continue
        if not isinstance(content, list): continue
        for b in content:
            if b.get("type") == "tool_use" and b.get("name") == "Write" and "report" in os.path.basename(b.get("input", {}).get("file_path", "")).lower():
                calls[b["id"]] = b["input"]["file_path"]
            if b.get("type") == "tool_result" and b.get("tool_use_id") in calls:
                body = b.get("content"); body = body if isinstance(body, str) else json.dumps(body, ensure_ascii=False)
                print(f"   Write {calls[b['tool_use_id']]} → {'被拒' if b.get('is_error') else '写成'}：{body[:110]}")
