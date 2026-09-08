#!/usr/bin/env bash
# gate-stage: 不变量条数在 invariants.md 之外也要对
#
# `10-kb-rot.sh` 已经查 `invariants.md` 自己那句「现共 N 条在用」与表里实际条数对不对，
# **但它只看那一份文件**。同一个量被别的文件抄过去之后，就没有任何东西在对了。
#
# ⚠️ **这条是实测出来的**（2026-09-07）：`decisions/13-验证路线.md` 的「冲突 1」
# 写着「26 条不变量若既是 checker 源码又是形式验证的 spec，就成了单点」，
# 而 `invariants.md` 当时已是 63 条在用 —— 那个 26 是 2026 年某轮的旧数，
# 从写下起就没人回来改过，`10-kb-rot.sh` 全程判绿，因为它压根不看这份文件。
# 坑在**跨文件的口径层**，不在某一段代码里
# （`.claude/singlefs-ai-sop/rules/show-me-test.md`「先问这个坑在哪一层」）。
#
# 查的是：`invariants.md` 之外的 kb 文件，正文里写「N 条在用」的地方，N 必须等于实际在用条数。
#
# ⚠️ **抓不到的那一半要说清楚**：
#   1. 不带「在用」二字的写法（「66 条不变量」）抓不到 —— 那种句子多半在复述某一轮普查
#      当时的口径，是历史事实，照现值判会误判。要它被管住，就把话写成「N 条在用」。
#   2. `## 历史版本` 之后的内容整段跳过（旧数留在那里是 `kb-discipline.md` 第 8 条要求的）。
#   3. `*-history.md` 整份跳过 —— 那些文件通篇是历史。
#
#   bash .claude/gate.d/36-invariant-count-cross-file.sh
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
KB=.claude/kb
[[ -f "$KB/invariants.md" ]] || { echo "  ! 找不到 $KB/invariants.md，本阶段跳过"; exit 0; }

python3 - "$KB" <<'PY'
import re, sys, os, glob

kb = sys.argv[1]
inv = os.path.abspath(os.path.join(kb, "invariants.md"))

text = open(inv, encoding="utf-8").read()
rows = [l for l in text.splitlines() if re.match(r"^\| I-\d+\.\d+ ", l)]
retired = [l for l in rows if re.match(r"^\| I-\d+\.\d+ \| [^|]* \| \*\*此编号不再使用", l)]
live = len(rows) - len(retired)

scanned, hits, bad = 0, [], []
for p in sorted(glob.glob(os.path.join(kb, "**", "*.md"), recursive=True)):
    if os.path.abspath(p) == inv or os.path.basename(p).endswith("-history.md"):
        continue
    scanned += 1
    body = open(p, encoding="utf-8").read().split("\n## 历史版本")[0]
    for m in re.finditer(r"(\d+)\s*条在用", body):
        n = int(m.group(1))
        hits.append((p, n))
        if n != live:
            line = body[:m.start()].count("\n") + 1
            bad.append(f"{p}:{line} 写「{n} 条在用」，而 invariants.md 实际在用 {live} 条")

if bad:
    print(f"  ✗ 不变量条数跨文件对不上 {len(bad)} 处：")      # gate-lint:summary
    for b in bad:
        print("      " + b)                                   # gate-lint:detail
    print(f"  → 怎么办：以 .claude/kb/invariants.md 为准，把那句里的数改成 {live}；")
    print("    若那句说的其实是某一轮普查当时的口径（不是现值），就把「在用」二字去掉并写明是哪一轮，")
    print("    否则下一个检索到它的人会拿一个旧数当现状。")
    sys.exit(1)

print(f"  ✓ 不变量条数跨文件一致：实际在用 {live} 条，扫了 {scanned} 份 kb 文件、命中 {len(hits)} 处声明")
PY
