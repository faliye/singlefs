# 真实命令语料：tools/extract-commands.py 从会话记录抽出的 Bash 命令（2026-10-10 起、2026-10-28 及以前、不含 tooling-writer、不含提到 hooks/ 与 heavy-test 的），
# 取「形状命中」的全部加一份固定种子的随机样本；主 agent 的命令按主 agent 喂，子 agent 的按 implementation-writer 喂。语料路径从环境变量 REAL_CORPUS 接。
import json, os, random, re
rows = [json.loads(l) for l in open(os.environ["REAL_CORPUS"], encoding="utf-8")]
base = [r for r in rows if r["type"] != "tooling-writer" and r["date"] <= "2026-10-28" and "hooks/" not in r["cmd"] and "heavy-test" not in r["cmd"] and len(r["cmd"]) <= 20000]
shape = re.compile(r"\bsed\b[^\n]*\$[\(\[]|subprocess\.|os\.system|os\.popen|\bawk\b|\bcd\s+[\"$]|\btrap\s|\bbash\s+(-\w*l\w*|--login)|expand_aliases|/dev/stdin|BASH_ENV|\bgit\s+-c\b|\bperl\s+-|\bxargs\b|find\s[^\n]*-exec|\bmake\b", re.M)
hit = [r for r in base if shape.search(r["cmd"])]
rest = [r for r in base if not shape.search(r["cmd"])]
random.Random(137).shuffle(rest)
chosen = hit + rest[:int(os.environ.get("REAL_RANDOM", "5000"))]
CASES = [("real-" + (r["type"] or "main"), "heavy-test-guard.sh", "main" if not r["sub"] else "implementation-writer", r["cmd"]) for r in chosen]
