"""实现员、工具员、执行员会话里两条命名检查命令（naming-lint.sh、code-source-discipline.sh --check）真跑出来的整条退出码，和每个子 agent 最后一次 naming-lint 的退出码。
用法：python3 -I naming_stats.py <抽出来的目录>"""
import collections, json, re, sys
rows = [r for r in json.load(open(sys.argv[1] + "/subagent_prompts.json")) if r["type"] in ("implementation-writer", "tooling-writer", "experiment-runner")]
codes, with_timeout, last_lint = collections.Counter(), 0, collections.Counter()
for r in rows:
    uses, results, sequence = {}, {}, []
    for line in open(r["path"], encoding="utf-8", errors="replace"):
        if "naming-lint.sh" not in line and "code-source-discipline.sh --check" not in line and "tool_result" not in line:
            continue
        try:
            content = json.loads(line).get("message", {}).get("content")
        except Exception:
            continue
        for part in content if isinstance(content, list) else []:
            if not isinstance(part, dict):
                continue
            if part.get("type") == "tool_use" and part.get("name") == "Bash":
                command = part.get("input", {}).get("command", "")
                if ("naming-lint.sh" in command or "code-source-discipline.sh --check" in command) and part["id"] not in uses:   # 同一次调用在记录里会重复出现，按 id 去重
                    uses[part["id"]] = command
                    sequence.append(part["id"])
            elif part.get("type") == "tool_result":
                text = part.get("content")
                if isinstance(text, list):
                    text = "\n".join(x.get("text", "") for x in text if isinstance(x, dict))
                results[part.get("tool_use_id")] = (text or "")
    lint_codes = []
    for identifier in sequence:
        if identifier not in results:
            continue
        match = re.match(r"\s*Exit code (\d+)", results[identifier])
        code = int(match.group(1)) if match else 0
        codes[code] += 1
        with_timeout += "timeout" in uses[identifier]
        if "naming-lint.sh" in uses[identifier]:
            lint_codes.append(code)
    if lint_codes:
        last_lint[lint_codes[-1]] += 1
print(f"name=naming_command_exit_codes 两条命令的 Bash 调用={sum(codes.values())} 整条退出码分布={dict(sorted(codes.items()))} 其中命令里带 timeout={with_timeout}")
print(f"name=last_naming_lint_exit 调用过 naming-lint.sh 的子agent={sum(last_lint.values())} 最后一次整条退出码分布={dict(sorted(last_lint.items()))}")
