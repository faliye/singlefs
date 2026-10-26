"""从真会话记录（<项目记录目录>/*/subagents/agent-*.jsonl）抽出语料：每个子 agent 一行 JSON。
抽：类型（meta.json 的 agentType）、派发提示（第一条 user 记录的文字）、按次序的工具调用（Bash 的命令、Edit/Write/MultiEdit 的 file_path、SubagentHandback 的 message）
与它们的工具结果（is_error 与内容开头 300 字）、记录里的 cwd。只读，不改记录。
用法：python3 -I extract_corpus.py <项目记录目录> <输出.jsonl> <进程数> [跳过的 agent 号…]
      python3 -I extract_corpus.py <项目记录目录> <输出.jsonl> <进程数> --only <agent 号清单文件>（复跑用：只抽清单里那几份）"""
import glob, json, os, sys
from concurrent.futures import ProcessPoolExecutor

KEPT_TOOLS = {"Bash", "Edit", "Write", "MultiEdit", "SubagentHandback", "NotebookEdit"}

def text_of(content):
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        return "\n".join(block.get("text", "") for block in content if isinstance(block, dict) and block.get("type") == "text")
    return ""

def one(path):
    agent = os.path.basename(path)[len("agent-"):-len(".jsonl")]
    meta_path = path[:-len(".jsonl")] + ".meta.json"
    try:
        meta = json.load(open(meta_path, encoding="utf-8"))
    except (OSError, ValueError):
        meta = {}
    prompt, records, first_time, kept_ids = None, [], None, set()
    with open(path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if prompt is not None and '"tool_use"' not in line and '"tool_result"' not in line:
                continue
            try:
                record = json.loads(line)
            except ValueError:
                continue
            message = record.get("message")
            if not isinstance(message, dict):
                continue
            if prompt is None and record.get("type") == "user":
                prompt = text_of(message.get("content"))
                first_time = record.get("timestamp")
                continue
            content = message.get("content")
            if not isinstance(content, list):
                continue
            blocks = []
            for block in content:
                if not isinstance(block, dict):
                    continue
                if record.get("type") == "assistant" and block.get("type") == "tool_use" and block.get("name") in KEPT_TOOLS:
                    arguments = block.get("input") if isinstance(block.get("input"), dict) else {}
                    small = {key: arguments[key] for key in ("command", "file_path", "message") if key in arguments}
                    blocks.append({"type": "tool_use", "id": block.get("id"), "name": block.get("name"), "input": small})
                    kept_ids.add(block.get("id"))
                elif record.get("type") == "user" and block.get("type") == "tool_result" and block.get("tool_use_id") in kept_ids:
                    result = block.get("content")
                    if isinstance(result, list):
                        result = "".join(str(part.get("text") or "") for part in result if isinstance(part, dict))
                    small = {"type": "tool_result", "tool_use_id": block.get("tool_use_id"), "content": str(result or "")[:300]}
                    if block.get("is_error") is True:
                        small["is_error"] = True
                    blocks.append(small)
            if blocks:
                records.append({"type": record.get("type"), "cwd": record.get("cwd"), "message": {"role": message.get("role"), "content": blocks}})
    return {"agent": agent, "session": path.split("/")[-3], "type": meta.get("agentType"), "description": meta.get("description"),
            "first_time": first_time, "prompt": prompt or "", "records": records}

def main():
    root, out, workers, rest = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4:]
    only = set(open(rest[1], encoding="utf-8").read().split()) if rest[:1] == ["--only"] else None
    skipped = set() if only is not None else set(rest)
    paths = sorted(path for path in glob.glob(os.path.join(root, "*", "subagents", "agent-*.jsonl"))
                   if os.path.basename(path)[len("agent-"):-len(".jsonl")] not in skipped
                   and (only is None or os.path.basename(path)[len("agent-"):-len(".jsonl")] in only))
    count = 0
    with ProcessPoolExecutor(max_workers=workers) as pool, open(out, "x", encoding="utf-8") as sink:
        for item in pool.map(one, paths, chunksize=4):
            sink.write(json.dumps(item, ensure_ascii=False) + "\n")
            count += 1
    print(f"抽了 {count} 份（共 {len(paths)} 份记录）", file=sys.stderr)
    if only is not None and len(paths) != len(only):
        print(f"清单里 {len(only)} 份，找到 {len(paths)} 份记录：对不上，这一趟不算数", file=sys.stderr)
        return 1
    return 0 if count == len(paths) else 1

if __name__ == "__main__":
    sys.exit(main())
