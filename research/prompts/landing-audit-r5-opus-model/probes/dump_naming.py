import json, sys
ids = set(sys.argv[2:])
for line in open(sys.argv[1], encoding="utf-8"):
    item = json.loads(line)
    if item["agent"] not in ids:
        continue
    print("=====", item["agent"], item["type"], item["first_time"], item["description"])
    results = {}
    for r in item["records"]:
        for b in r["message"]["content"]:
            if b.get("type") == "tool_result":
                results[b["tool_use_id"]] = b
    for r in item["records"]:
        for b in r["message"]["content"]:
            if b.get("type") != "tool_use":
                continue
            res = results.get(b["id"], {})
            head = (res.get("content") or "")[:120].replace("\n", "⏎")
            if b["name"] in ("Edit", "Write", "MultiEdit") and str(b["input"].get("file_path", "")).endswith(".rs"):
                print("  EDIT", b["input"]["file_path"], "err" if res.get("is_error") else "ok")
            elif b["name"] == "Bash" and ("naming-lint" in b["input"].get("command", "") or "code-source-discipline" in b["input"].get("command", "")):
                print("  BASH cwd=", r.get("cwd"), "err" if res.get("is_error") else "ok", "|", head)
                print("     ", b["input"]["command"][:900].replace("\n", "⏎"))
            elif b["name"] == "SubagentHandback":
                print("  HANDBACK", "err" if res.get("is_error") else "ok", "|", head)
