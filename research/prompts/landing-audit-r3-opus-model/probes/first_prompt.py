"""从子 agent 会话记录取首条 user 消息的正文（派发提示原样）。用法：first_prompt.py <agent-*.jsonl> <输出文件>"""
import json, sys
for line in open(sys.argv[1], errors="replace"):
    try: record = json.loads(line)
    except ValueError: continue
    if record.get("type") == "user":
        content = record["message"]["content"]
        text = content if isinstance(content, str) else "".join(block.get("text", "") for block in content if block.get("type") == "text")
        open(sys.argv[2], "w").write(text); break
