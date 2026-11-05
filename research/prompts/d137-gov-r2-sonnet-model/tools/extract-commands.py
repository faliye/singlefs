#!/usr/bin/env python3
"""从会话记录里抽 Bash 工具的真实命令（主会话与子 agent 的都抽），去重，写 JSON 行：{"cmd":…, "sub":是否子 agent, "n":出现次数}。
用法：extract-commands.py <会话记录目录> <输出文件> [--since 日期]
只读会话记录；输出里可能有私有主机信息，输出放草稿目录，不进仓。"""
import json, os, sys, collections
root, out = sys.argv[1], sys.argv[2]
since = sys.argv[sys.argv.index("--since") + 1] if "--since" in sys.argv else ""
seen = collections.OrderedDict()
files = 0
for dirpath, _, names in os.walk(root):
    for name in names:
        if not name.endswith(".jsonl"):
            continue
        path = os.path.join(dirpath, name)
        sub = "/subagents/" in path
        agent_type = ""
        if sub:
            try:
                agent_type = json.load(open(path[:-6] + ".meta.json", encoding="utf-8")).get("agentType", "")
            except (OSError, ValueError):
                agent_type = "?"
        files += 1
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                for line in handle:
                    if '"name":"Bash"' not in line and '"name": "Bash"' not in line:
                        continue
                    try:
                        obj = json.loads(line)
                    except ValueError:
                        continue
                    if since and str(obj.get("timestamp", ""))[:10] < since:
                        continue
                    message = obj.get("message")
                    content = message.get("content") if isinstance(message, dict) else None
                    if not isinstance(content, list):
                        continue
                    for block in content:
                        if isinstance(block, dict) and block.get("type") == "tool_use" and block.get("name") == "Bash":
                            command = (block.get("input") or {}).get("command")
                            if isinstance(command, str) and command.strip():
                                key = (command, sub, agent_type, str(obj.get("timestamp", ""))[:10])
                                seen[key] = seen.get(key, 0) + 1
        except OSError:
            pass
with open(out, "x", encoding="utf-8") as handle:
    for (command, sub, agent_type, date), n in seen.items():
        handle.write(json.dumps({"cmd": command, "sub": sub, "type": agent_type, "date": date, "n": n}, ensure_ascii=False) + "\n")
print(f"files={files} distinct={len(seen)}")
