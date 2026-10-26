"""全部子 agent 会话记录里每次 Write：按文件名（不分大小写）开头是 report / summary / findings / analysis、名字中间带这几个词、其余，分组数被拒与写成。
被拒按工具结果 is_error 且正文含「Subagents should return findings as text」认。用法：write_name_refusals.py <会话目录根>"""
import collections, glob, json, os, re, sys
WORDS = ("report", "summary", "findings", "analysis")
REFUSAL = "Subagents should return findings as text"
table = collections.Counter(); samples = collections.defaultdict(set)
for path in glob.glob(os.path.join(sys.argv[1], "*", "subagents", "agent-*.jsonl")):
    calls = {}
    for line in open(path, errors="replace"):
        if '"Write"' not in line and '"tool_result"' not in line: continue
        try: r = json.loads(line)
        except ValueError: continue
        content = (r.get("message") or {}).get("content")
        if not isinstance(content, list): continue
        for b in content:
            if b.get("type") == "tool_use" and b.get("name") == "Write":
                calls[b["id"]] = os.path.basename(b.get("input", {}).get("file_path", ""))
            elif b.get("type") == "tool_result" and b.get("tool_use_id") in calls:
                name = calls.pop(b["tool_use_id"]); low = name.lower()
                body = b.get("content"); body = body if isinstance(body, str) else json.dumps(body, ensure_ascii=False)
                refused = bool(b.get("is_error")) and REFUSAL in body
                head = next((w for w in WORDS if low.startswith(w)), None)
                inner = next((w for w in WORDS if w in low), None)
                group = f"开头 {head}" if head else (f"中间带 {inner}" if inner else "其余")
                ext = ".md" if low.endswith(".md") else "非 .md"
                table[(group, ext, "被拒" if refused else "写成")] += 1
                if head or inner: samples[(group, ext, "被拒" if refused else "写成")].add(name)
for key in sorted(table): print(key, table[key], sorted(samples.get(key, []))[:6])
