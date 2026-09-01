#!/usr/bin/env bash
# gate-stage: 已定分项的正文里说自己还没定
#
# **判据**：在一条决策的「已定项」小节（含各 `#### 已定项 N` 论证）里，
# 凡出现「X 仍未定 / X 还没定 / X 尚未定」这样的断言，就查 X 是谁：
#   a. X 是**自指**（取值 / 本项 / 该项 / 本分项）⇒ 这条分项标着已定却说自己没定，判红；
#   b. X 落在**同一个决策里另一条已定分项的标题**上 ⇒ 说一个已经定了的东西没定，判红。
#
# ⚠️ **这一类只发生在同一个文件内部，而门禁此前对它整个是盲的。**
# `20-kb-shape` 比的是索引行与标题行的状态，`22-item-ref-status` 比的是引用处
# 写的状态与索引表，两个都只看**状态标记**，不看正文说了什么。
# 实测三处：D16 已定项 5 定了 `T_time` = 5 s，同一项正文里留着
# 「取值仍未定」；D23 已定项 10 说「hash 算法仍未定」而已定项 11 定了 CRC32C；
# D23 已定项 8 自己的标题写着「宽度 32 位」而它的论证里留着「宽度仍未定」。
# 三处都是**定案之后没清理推导过程**留下的，而检索会把陈旧的那一条单独端出来
# （`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 8 条）。
#
# ⚠️ **不判「本笔仍然未定」这一类。** 一条已定分项里就某个**子问题**显式记
# 「这一笔还没定」是 kb-discipline 第 3 条要求的「显式记录不知道」，不是矛盾；
# 真要收拾它，该做的是把那个子问题升成独立分项，而不是把这句话删掉。
# 判据因此只认自指与兄弟分项两种，**宁可漏，不可假红**。
#
# 判别力：样本 red 里已定项 1 的正文写「取值仍未定」，必须红；
# green 里同一句挪进未定项小节，必须绿。
set -uo pipefail
ROOT="${1:-.}"
DEC="$ROOT/.claude/kb/decisions"
[[ -d "$DEC" ]] || { echo "  ✓ 没有 $DEC，无对象可判"; exit 0; }

python3 - "$DEC" <<'PY'
import re, sys, pathlib

dec = pathlib.Path(sys.argv[1])
SELF = ("取值", "本项", "该项", "本分项", "本条")
OPEN = re.compile(r'(仍然未定|仍未定|尚未定(?!案)|还没定|均未定)')
# X：断言之前那一小段名词短语。停在标点 / markdown 记号 / 空白上。
PHRASE = re.compile(r'([^\s。，、；：（）()「」|*`＊>#⚠️⇒—]{2,12})$')

bad = []
for f in sorted(dec.glob("*.md")):
    lines = f.read_text(encoding="utf-8").splitlines()

    # 已定项小节 = `### 已定项` 索引表 + 各 `#### 已定项 N` 论证
    inside, region, settled_lines = False, [], []
    for i, line in enumerate(lines, 1):
        if re.match(r'^###\s+已定项\s*$', line) or re.match(r'^####\s+已定项', line):
            inside = True
            settled_lines.append(line)
            continue
        if re.match(r'^###\s+未定项\s*$', line) or re.match(r'^####\s+未定项', line) or re.match(r'^##\s', line):
            inside = False
        if inside:
            region.append((i, line))
            settled_lines.append(line)

    if not region:
        continue
    settled_text = "\n".join(settled_lines)

    for i, line in region:
        for m in OPEN.finditer(line):
            head = line[:m.start()].rstrip()
            pm = PHRASE.search(head)
            if not pm:
                continue
            x = pm.group(1).strip('*` ')
            if len(x) < 2:
                continue
            why = None
            if x in SELF:
                why = "自指——这条分项标着已定，正文却说它自己没定"
            else:
                # 同一文件里另一条已定分项的标题含 X？（排除本行自己）
                for other in settled_text.splitlines():
                    if other.strip() == line.strip():
                        continue
                    if re.match(r'^\s*(\|\s*)?\d+[.\s|]', other) and x in other and '已定' in other:
                        why = f"「{x}」在同一条决策的另一条**已定**分项里已经定了"
                        break
            if why:
                bad.append((f.name, i, x, why, line.strip()[:110]))
            break

if bad:
    for fn, ln, x, why, text in bad:
        print(f"  ✗ {fn}:{ln} 已定项正文里断言「{x}」未定：{why}")
        print(f"     原文：{text}")
    print("     → 怎么办：这是定案之后没清理的推导过程。把这句改写成定案后的现状；")
    print("               若那个子问题真的还开着，把它升成一条独立的未定项，别留在已定项正文里。")
    sys.exit(1)
print(f"  ✓ 已定项的正文没有把已经定了的东西说成未定（扫 {len(list(dec.glob('*.md')))} 条决策）")
PY
