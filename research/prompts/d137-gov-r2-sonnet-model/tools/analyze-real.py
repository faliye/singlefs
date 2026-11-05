#!/usr/bin/env python3
"""读 probe.py 在真实命令语料上的输出（run-real.txt）：修补前放行、修补后拒的条数，按拒绝原因分类；随机样本部分单独数（无偏率）。
用法：analyze-real.py <run-real.txt> <real-commands.jsonl> <输出 JSON>"""
import ast, collections, json, random, re, sys
run, corpus, out = sys.argv[1:4]
rows = [json.loads(l) for l in open(corpus, encoding="utf-8")]
base = [r for r in rows if r["type"] != "tooling-writer" and r["date"] <= "2026-10-28" and "hooks/" not in r["cmd"] and "heavy-test" not in r["cmd"] and len(r["cmd"]) <= 20000]
shape = re.compile(r"\bsed\b[^\n]*\$[\(\[]|subprocess\.|os\.system|os\.popen|\bawk\b|\bcd\s+[\"$]|\btrap\s|\bbash\s+(-\w*l\w*|--login)|expand_aliases|/dev/stdin|BASH_ENV|\bgit\s+-c\b|\bperl\s+-|\bxargs\b|find\s[^\n]*-exec|\bmake\b", re.M)
rest = [r for r in base if not shape.search(r["cmd"])]
random.Random(137).shuffle(rest)
restset = {r["cmd"] for r in rest[:5000]}
lines = open(run, encoding="utf-8").read().splitlines()
total = collections.Counter(); reasons = collections.Counter(); examples = {}
for i, line in enumerate(lines):
    m = re.match(r"now:rc=(\d+)/n=(\d+)/([\d.]+)s base:rc=(\d+)/n=(\d+)/([\d.]+)s.*?\| (\w+) (\S+) \| (\S+) \| (.*)$", line)
    if not m:
        continue
    try:
        command = ast.literal_eval(m[10])
    except (ValueError, SyntaxError):
        command = m[10]
    in_rest = command in restset
    total["rows"] += 1
    total["rest rows"] += in_rest
    total["shape rows"] += (not in_rest)
    if m[4] == "0" and m[1] == "2":
        total["newly refused"] += 1
        total["newly refused in random sample"] += in_rest
        why = re.sub(r"^\s+now> ", "", lines[i + 1]) if i + 1 < len(lines) else ""
        found = re.search(r"被拒：(.*?)（", why) or re.search(r"被拒：(.*?)：", why)
        key = re.sub(r"`[^`]+`|\d+", "·", (found.group(1) if found else why)[:70])
        reasons[key] += 1
        examples.setdefault(key, []).append([m[8], command[:400]])
    if m[4] == "2" and m[1] == "0":
        total["newly allowed"] += 1
    if m[4] == "2" and m[1] == "2":
        total["refused both"] += 1
json.dump({"total": total, "reasons": reasons, "examples": {k: v[:8] for k, v in examples.items()}}, open(out, "x", encoding="utf-8"), ensure_ascii=False, indent=1)
print(dict(total))
for key, n in reasons.most_common(40):
    print(n, key)
