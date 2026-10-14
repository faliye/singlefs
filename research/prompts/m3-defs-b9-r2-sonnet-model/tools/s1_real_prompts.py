#!/usr/bin/env python3
"""S1：真派发提示语料。语料 = ~/.local/share/singlefs/work/*/prompt.md（每个草稿目录里主 agent 存的派发提示，首行 `# 派发：<类型>` 是存档标题，不在真提示里，去掉）。
对每份用 b9、b10、b10 加弄坏开关 code-block-library-missing（当作 markdown-it 载不进）三种配置经 --check-prompt 判，只比较两项：线程上限（⑮）与预计挂钟与长活声明（⑭）拒没拒。
排除 --skip 指定的目录名。只打汇总与不一致的目录名，不打提示正文。用法：s1_real_prompts.py <b9 树> <b10 树> <工作目录> [--skip 名 ...]"""
import glob, os, re, subprocess, sys, collections
t9, t10, work = (os.path.abspath(p) for p in sys.argv[1:4])
skip = set(sys.argv[5:]) if len(sys.argv) > 4 and sys.argv[4] == "--skip" else set()
os.makedirs(work, exist_ok=True)
base = os.path.expanduser("~/.local/share/singlefs/work")
files = sorted(p for p in glob.glob(os.path.join(base, "*", "prompt.md")) if os.path.basename(os.path.dirname(p)) not in skip)
def run(tree, path, kind, extra=None):
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", CLAUDE_PROJECT_DIR=tree, **(extra or {}))
    r = subprocess.run(["bash", os.path.join(tree, ".claude/hooks/runner-dispatch-guard.sh"), "--check-prompt", path, "--type", kind, "--root", tree],
                       capture_output=True, text=True, env=env, timeout=120)
    out = r.stdout + r.stderr
    m = re.search(r"会拒 \d+ 处（([^）]*)）", out)
    names = set(m.group(1).split("、")) if m else set()
    return frozenset(n for n in names if n in ("线程上限", "预计挂钟与长活声明")), "载不进" in out
count = collections.Counter(); diffs = []
for path in files:
    text = open(path, encoding="utf-8", errors="replace").read()
    head, _, body = text.partition("\n")
    m = re.match(r"# 派发：(\S+)", head)
    if not m:
        count["首行不是存档标题"] += 1; continue
    p = os.path.join(work, "p.txt"); open(p, "w", encoding="utf-8").write(body)
    a, _ = run(t9, p, m.group(1)); b, _ = run(t10, p, m.group(1)); c, noted = run(t10, p, m.group(1), {"RUNNER_DISPATCH_GUARD_BREAK": "code-block-library-missing"})
    count["份数"] += 1
    count["b9 与 b10 的两项不同"] += a != b
    count["b10 有库 与 无库 的两项不同"] += b != c
    count["b10 无库时 stderr 写了载不进"] += noted
    count["fence 里含『线程上限』的份数"] += bool(re.search(r"```[\s\S]*?线程上限[\s\S]*?```", body))
    if a != b or b != c:
        diffs.append((os.path.basename(os.path.dirname(path)), sorted(a), sorted(b), sorted(c)))
for k, v in count.items(): print(f"{k}\t{v}")
for d in diffs: print("不一致\t", d)
