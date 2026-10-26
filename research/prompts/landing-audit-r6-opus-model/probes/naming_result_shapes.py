"""K2 交回闸 ⑤ 读退出码的那一处（tool_calls 的 outcome：不是错误记 0，「Exit code N」开头记 N，别的记 None）在真记录上撞到哪几种结果形状。
扫全部子 agent 真会话记录里命令带 naming-lint.sh 或 code-source-discipline.sh 的 Bash 调用，按 is_error、结果开头的样子与输出是否被落成 persisted-output 分类。只读。
用法：python3 -I naming_result_shapes.py <项目记录目录>"""
import collections, glob, json, os, re, sys
projects = sys.argv[1]
shapes, examples = collections.Counter(), collections.defaultdict(list)
for path in sorted(glob.glob(os.path.join(projects, "*", "subagents", "agent-*.jsonl"))):
    agent = os.path.basename(path)[6:-6]
    commands = {}
    with open(path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if '"tool_use"' not in line and '"tool_result"' not in line:
                continue
            try:
                record = json.loads(line)
            except ValueError:
                continue
            content = (record.get("message") or {}).get("content")
            for block in content if isinstance(content, list) else []:
                if not isinstance(block, dict):
                    continue
                if block.get("type") == "tool_use" and block.get("name") == "Bash":
                    command = str((block.get("input") or {}).get("command") or "")
                    if "naming-lint.sh" in command or "code-source-discipline.sh" in command:
                        commands[block.get("id")] = command
                elif block.get("type") == "tool_result" and block.get("tool_use_id") in commands:
                    text = block.get("content")
                    if isinstance(text, list):
                        text = "".join(str(part.get("text") or "") for part in text if isinstance(part, dict))
                    text = text if isinstance(text, str) else ""
                    error = block.get("is_error") is True
                    persisted = "<persisted-output>" in text
                    head = ("Exit code N 开头" if re.match(r"\s*Exit code \d+", text) else
                            "persisted-output 开头" if text.lstrip().startswith("<persisted-output>") else
                            "被钩子拒" if "hook" in text[:200].lower() else
                            "超时" if "timed out" in text[:300].lower() else "别的开头")
                    key = ("is_error" if error else "不是错误", head, "输出落成 persisted-output" if persisted else "")
                    shapes[key] += 1
                    if len(examples[key]) < 3:
                        examples[key].append(f"{agent}：{text[:70]!r}")
for key, count in sorted(shapes.items(), key=lambda pair: -pair[1]):
    print(f"{'｜'.join(part for part in key if part)}：{count} 条；例：{' ／ '.join(examples[key])}")
