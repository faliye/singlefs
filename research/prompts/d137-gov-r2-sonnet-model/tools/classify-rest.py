#!/usr/bin/env python3
"""真实语料里随机样本（无偏）部分：修补前放行、修补后拒的命令按拒绝原因分类，只打条数与原因，不打命令。
用法：classify-rest.py <run-real.txt> <real-commands.jsonl>"""
import ast, collections, json, random, re, sys
run, corpus = sys.argv[1:3]
rows = [json.loads(l) for l in open(corpus, encoding="utf-8")]
base = [r for r in rows if r["type"] != "tooling-writer" and r["date"] <= "2026-10-28" and "hooks/" not in r["cmd"] and "heavy-test" not in r["cmd"] and len(r["cmd"]) <= 20000]
shape = re.compile(r"\bsed\b[^\n]*\$[\(\[]|subprocess\.|os\.system|os\.popen|\bawk\b|\bcd\s+[\"$]|\btrap\s|\bbash\s+(-\w*l\w*|--login)|expand_aliases|/dev/stdin|BASH_ENV|\bgit\s+-c\b|\bperl\s+-|\bxargs\b|find\s[^\n]*-exec|\bmake\b", re.M)
rest = [r for r in base if not shape.search(r["cmd"])]
random.Random(137).shuffle(rest)
restset = {r["cmd"] for r in rest[:5000]}
lines = open(run, encoding="utf-8").read().splitlines()
reasons = collections.Counter(); rows_rest = 0
for i, line in enumerate(lines):
    m = re.match(r"now:rc=(\d+)/n=(\d+)/([\d.]+)s base:rc=(\d+)/n=(\d+)/([\d.]+)s.*?\| (\w+) (\S+) \| (\S+) \| (.*)$", line)
    if not m:
        continue
    try:
        command = ast.literal_eval(m[10])
    except (ValueError, SyntaxError):
        continue
    if command not in restset:
        continue
    rows_rest += 1
    if m[4] == "0" and m[1] == "2":
        why = re.sub(r"^\s+now> ", "", lines[i + 1])
        found = re.search(r"被拒：(.*?)（", why) or re.search(r"被拒：(.*?)：", why)
        key = re.sub(r"/\S+", "<路径>", (found.group(1) if found else why)[:80])
        reasons[key] += 1
print("随机样本行数", rows_rest, "修补前放行、修补后拒", sum(reasons.values()))
for key, n in reasons.most_common():
    print(n, key)
