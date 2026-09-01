#!/usr/bin/env bash
# gate-stage: 说某条决策未定，而它已经定了
#
# **判据**：kb 正文里凡是紧贴着「D<n>（简称）」写下「未定」的句子，
# 拿它与那条决策**状态行**上的实际状态比对；实际不是「待定」就判红。
#
# ⚠️ **它与「未定项有没有被别处定了」（`60-stale-open-items.sh`）不是一条。**
# 那个阶段只扫**未定项小节**，且靠「谁比谁新」这个时间判据；
# 本阶段扫**全部正文**，靠「说的状态与写着的状态对不对得上」这个文本判据。
# 实测三处它抓得到而 60 抓不到：一处写在**已定项的论证里**（D22 轴② 的前置①
# 说「D25，取值未定」，而 D25 已定），一处写在**不变量清单的开篇**
# （说「D4/D6/D8/D9/D10 未定」，而其中四条都定了），一处写在**实验正文的局限里**。
# 三处都不在未定项小节，60 一个字都看不见。
#
# ⚠️ **为什么必须机检**：`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条
# 「矛盾比空白更糟」——检索不会把两条都端出来，它挑一条而且不告诉你挑了哪条。
# 一句陈旧的「X 还没定」会让读的人以为那一格还开着，从而**不去读那条定案**。
#
# ⚠️ **判据是收紧过的，收紧的理由是实测**：第一版允许「D<n> …… 未定」之间隔任意
# 40 字，在真实语料上报 32 处而只有 4 处是真的——分项索引表的「状态：未定」、
# 自动生成的分项清单、「从未定过」、以及**被引号括起来当历史陈述引用的**
# 「且 D6 未定」全被误伤。假红压倒真红的检查等于没有检查
# （`.claude/singlefs-ai-sop/rules/show-me-test.md`），所以只认**紧贴**的写法，
# 并跳过「」引文。代价是漏掉隔着名词短语的那种（例：「D2 的条带宽度可变粒度仍然未定」），
# 那一类留给人看——**宁可漏，不可假红**。
#
# 判别力：样本 red 里「D101 未定」而 D101 标着已定，必须红；
# green 里同一句话而 D101 标着待定，必须绿。
set -uo pipefail
ROOT="${1:-.}"
KB="$ROOT/.claude/kb"
[[ -d "$KB/decisions" ]] || { echo "  ✓ 没有 $KB/decisions，无对象可判"; exit 0; }

python3 - "$KB" <<'PY'
import re, sys, pathlib

kb = pathlib.Path(sys.argv[1])

# 1) 每条决策的实际状态：取正文标题行 `## D<n> 简称 —— 状态`
# ⚠️ **破折号前的空格是可选的**：实测 D14 写作「…持久临时）—— 半定」，
# 第一版的 `\s+——` 把整条 D14 静默漏掉了，而它恰恰是矛盾最密的一条。
# ⚠️ **解析不了的标题一律判红，不许当成跳过**
# （`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）——
# 一条没被扫到的决策与一条干净的决策，在门禁输出里长得一模一样。
status, name, unparsed = {}, {}, []
title_re = re.compile(r'^##\s+(D\d+)\s*(.+?)\s*——\s*(.+?)\s*$')
for f in sorted((kb / "decisions").glob("*.md")):
    lines = f.read_text(encoding="utf-8").splitlines()
    hit = next((m for m in (title_re.match(l) for l in lines) if m), None)
    if hit:
        status[hit.group(1)], name[hit.group(1)] = hit.group(3), hit.group(2)
    elif any(re.match(r'^##\s+D\d+', l) for l in lines):
        unparsed.append(f.name)
if unparsed:
    for fn in unparsed:
        print(f"  ✗ {fn} 的决策标题解析不了，本阶段没有扫过它")
    print("     → 标题要写成 `## D<n> 简称 —— 状态`。解析不了就等于没扫，不许当成通过。")
    sys.exit(1)
if not status:
    print("  ✓ 没有决策正文，无对象可判"); sys.exit(0)

# 2) 紧贴写法：D<n>（简称） 之后只允许空白 / 逗号 / 括号 / 「取值」「状态」
#    再接可选的「仍/仍然/尚/都/均」，然后就是「未定」，且不许是「未定项」。
ref_re = re.compile(r'D\d+(?:（[^）]*）)?')
tight_re = re.compile(r'^[\s，,、（(]*(?:取值|状态)?[\s：:]*(?:仍然|仍|尚|都|均)?未定(?!项)')

bad, in_hist = [], False
for f in sorted(kb.rglob("*.md")):
    rel = f.relative_to(kb.parent.parent)
    in_hist = False
    for i, line in enumerate(f.read_text(encoding="utf-8").splitlines(), 1):
        if re.match(r'^#+\s*历史版本', line):
            in_hist = True          # 文末历史节按定义写的是旧状态，不判
        if in_hist:
            continue
        for m in ref_re.finditer(line):
            d = re.match(r'D\d+', m.group(0)).group(0)
            if d not in status:
                continue
            tm = tight_re.match(line[m.end():])
            if not tm:
                continue
            # 「」引文里的是被当作历史陈述引用的原话，不判
            head = line[:m.end()]
            if head.count('「') > head.count('」'):
                continue
            if status[d] == '待定':
                continue
            bad.append((str(rel), i, d, status[d], line.strip()[:110]))
            break

if bad:
    for rel, ln, d, st, text in bad:
        print(f"  ✗ {rel}:{ln} 说 {d} 未定，而 {d}（{name[d]}）的状态行是「{st}」")
        print(f"     原文：{text}")
    print("     → 怎么办：读那条决策现在定了什么，把这句改写成它的现状；")
    print("               若指的是它下面某个还开着的分项，写成「D<n>（简称） 未定项 k」。")
    sys.exit(1)
print(f"  ✓ 没有把已定的决策说成未定（扫 {len(status)} 条决策）")
PY
