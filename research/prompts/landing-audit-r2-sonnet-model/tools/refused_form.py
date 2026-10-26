import json, glob, os, sys, collections
ROOT = os.environ.get("SESSIONS_ROOT") or os.path.expanduser("~/.claude/projects/-home-fy5090-code-singlefs")
forms = collections.Counter()
examples = {}
n_files = 0
for p in glob.glob(ROOT + "/*/subagents/agent-*.jsonl"):
    try:
        data = open(p, "rb").read()
    except OSError:
        continue
    if b"SubagentHandback" not in data:
        continue
    uses = {}
    results = []
    for line in data.split(b"\n"):
        if b"SubagentHandback" not in line and b"tool_result" not in line:
            continue
        try:
            rec = json.loads(line)
        except Exception:
            continue
        c = rec.get("message", {}).get("content")
        if not isinstance(c, list): continue
        for b in c:
            if not isinstance(b, dict): continue
            if b.get("type") == "tool_use" and b.get("name") == "SubagentHandback":
                uses[b["id"]] = True
            if b.get("type") == "tool_result" and b.get("tool_use_id") in uses:
                cont = b.get("content")
                txt = cont if isinstance(cont, str) else json.dumps(cont, ensure_ascii=False)
                results.append((b.get("is_error"), txt))
    n_files += 1
    for is_err, txt in results:
        blocked = "hook" in txt.lower() and ("block" in txt.lower() or "拒" in txt or "✗" in txt)
        key = (bool(is_err), "success:false" if '"success": false' in txt.replace(' ', ' ') or '"success":false' in txt else ("success:true" if 'success' in txt and 'true' in txt else "other"), "hookblock" if blocked else "plain")
        forms[key] += 1
        examples.setdefault(key, txt[:200])
print("files", n_files)
for k, v in forms.most_common():
    print(v, k, "|", examples[k].replace("\n", " ")[:160])
