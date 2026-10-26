import glob, json, collections, sys, os
root = os.path.expanduser("~/.claude/projects/-home-fy5090-code-singlefs")
shapes = collections.Counter(); examples = {}
files = glob.glob(os.path.join(root, "*", "subagents", "agent-*.jsonl"))
per_agent = collections.Counter()
for path in files:
    calls = set()
    with open(path, errors="replace") as h:
        for line in h:
            if "SubagentHandback" not in line and not any(c in line for c in calls):
                continue
            try: r = json.loads(line)
            except ValueError: continue
            m = r.get("message")
            if not isinstance(m, dict) or not isinstance(m.get("content"), list): continue
            for b in m["content"]:
                if not isinstance(b, dict): continue
                if m.get("role") == "assistant" and b.get("type") == "tool_use" and b.get("name") == "SubagentHandback":
                    calls.add(b.get("id"))
                elif m.get("role") == "user" and b.get("type") == "tool_result" and b.get("tool_use_id") in calls:
                    c = b.get("content")
                    if isinstance(c, list): c = "".join(p.get("text") or "" for p in c if isinstance(p, dict))
                    try: parsed = json.loads(c) if isinstance(c, str) else None
                    except ValueError: parsed = None
                    if b.get("is_error") is True:
                        key = "is_error"
                    elif isinstance(parsed, dict):
                        key = f"json success={parsed.get('success')!r} message={str(parsed.get('message'))[:50]!r}"
                    else:
                        key = "plain:" + (c or "")[:60].replace("\n", " ")
                    shapes[key] += 1; examples.setdefault(key, path)
print("files", len(files))
for k, v in shapes.most_common(40): print(v, k, "|", os.path.basename(examples[k]))
