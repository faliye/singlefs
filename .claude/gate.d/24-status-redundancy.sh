#!/usr/bin/env bash
# gate-stage: 状态别说两遍，也别挂错节
#
# 分项已经按状态分住「### 已定项」与「### 未定项」两节 ⇒ **节本身就是状态**。
# 再在标题或条目里写一遍「—— 已定」，同一个词就说了两遍；
# 而重复的标注会各自漂移——检索取到其中一处时不知道另一处写的是什么
# （`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条：矛盾比空白更糟）。
#
# 判三样：
#   1. 小节标题不许写成「已定项 N —— 已定…」（节名已经说过了）
#   2. 同一个条目不许既有「—— 已定」又有「状态：已定」
#   3. 条目的「状态：」必须与它所在的节一致（已定项节里不许有状态：未定）
#
# ⚠️ 只认**显式的**「状态：X」与破折号形态，不认句中随口提到的「已定」——
#    一条未定项里出现「D18 已定项 3 已定」是正常引用，不是它自己的状态。
#
#   bash .claude/gate.d/24-status-redundancy.sh
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
DEC=.claude/kb/decisions
[[ -d "$DEC" ]] || { echo "  ! 找不到 $DEC，本阶段跳过"; exit 0; }

# 扫的范围：决策正文 + 实验正文 + records/。
# 后两处今天是干净的（2026-08-31 现查：三类毛病各 0 处），扫它们是**防复发**——
# 一条只在出事之后才加的检查，等于承认那一次出事没人拦得住。
python3 - "$DEC" .claude/kb/experiments records <<'PY'
import re, sys, glob, os
bad = []
files = []
for d in sys.argv[1:]:
    if os.path.isdir(d):
        files += sorted(glob.glob(os.path.join(d, '**', '*.md'), recursive=True))

for f in files:
    body = open(f, encoding='utf-8').read().split('\n## 历史版本')[0]
    name = os.path.basename(f)

    # 1. 标题里重复
    for i, line in enumerate(body.split('\n'), 1):
        if re.match(r'^#{3,4} (已定项|未定项) \d+ *—— *\*{0,2}(已定|未定)', line):
            bad.append((name, i, '标题重复', line.strip()[:60]))

    # 2 / 3. 条目层
    for sec_name, want in (('已定项', '已定'), ('未定项', '未定')):
        m = re.search(r'^### %s\s*$(.*?)(?=^#{2,3} |\Z)' % sec_name, body, re.M | re.S)
        if not m:
            continue
        base = body[:m.start(1)].count('\n') + 1
        for off, line in enumerate(m.group(1).split('\n')):
            if not re.match(r'^(?:\d+\.|\|\s*\d+\s*\|)', line):
                continue
            ln = base + off
            dash = re.search(r'——\s*\*{0,2}(已定|未定)', line)
            stat = re.search(r'状态：\s*\*{0,2}(已定|未定)', line)
            if dash and stat and dash.group(1) == stat.group(1):
                bad.append((name, ln, '条目说两遍', line.strip()[:60]))
            if stat and stat.group(1) != want:
                bad.append((name, ln, '挂错节', line.strip()[:60]))

if bad:
    print('  ✗ 状态被说了两遍，或分项挂错了节：')
    for n, ln, why, txt in bad[:20]:
        print(f'     {n}:{ln}  {why} —— {txt}')
    if len(bad) > 20:
        print(f'     …… 另有 {len(bad)-20} 处')
    print('     → 小节标题写成「### 已定项 N（日期）：结论」，别再写「—— 已定」；')
    print('     → 条目二选一：留「状态：已定」这一处机器可读的标注，把「—— 已定」去掉，')
    print('       日期与结论一个字都不要丢；')
    print('     → 状态与所在节不一致的，把条目搬到对的那一节，别就地改状态词。')
    sys.exit(1)

print('  ✓ 状态只说一遍，且分项都在对的节里')
PY
