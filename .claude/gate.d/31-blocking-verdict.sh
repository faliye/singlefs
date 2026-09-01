#!/usr/bin/env bash
# gate-stage: 每个未定项有没有判过改不改第一个事务的字节
#
# 还 checks-owed.md C50（阻塞标记没人维护）的**覆盖性**那一半。
#
# **它拦的不是「判错」，是「没判过」。** 一条未定项挡不挡第一行代码，此前只记在正文
# 某一句 ⚠️ 里，措辞还各不相同（「不阻塞第一行代码」「不阻塞第一行事务层代码」
# 「可延后」），于是**没判过与判过是否**在 kb 里长得一模一样——2026-09-01 现查：
# 23 条未定项里只有 3 条写过明确判定，另外 20 条从来没按那把尺量过，而没有任何东西发现。
#
# **它与 60 / 61 是三把不同的尺子，不是加强版**：
#   60：跨文件 + 看历史——未定项点名了别的决策，而那个决策后来定过东西。
#   61：同文件 + 看本次 diff——本次新增了「已定」小节，而同文件的未定项一个都没碰。
#   31：**不看历史也不看 diff**——每条未定项当下有没有一条判定，缺就红。
# 60 / 61 管的是「判定会不会过期」，31 管的是「判定存不存在」。
# 少了 31，一条**从未判过**的未定项在 60 / 61 眼里是干净的：它没点名别人，本次也没人定案。
#
# **判定的规范形态**（写在该分项的登记行或它那一条列表条目里）：
#
#   改第一个事务的字节：否（2026-09-01，依据：…）
#
# 三个合法取值：**是** / **否** / **无对象**（前置已被推翻，这一项没有对象了）。
#
# **那把尺的可执行形式**（不是新发明，是 decisions-history.md 2026-08-29 其十八 逐字用过的那个）：
# 那一轮的依据写作「攻方逐条给出**第一个事务写不出的是哪些字节**」⇒ 尺子问的不是
# 「两个答案会不会给出不同的字节」，而是——
#
#   **不定它，第一个事务写出的字节能不能由已经定了的条款唯一确定？**
#   （**mkfs 写出的字节算在内**——事务必须落在一个格式化好的盘上，改 mkfs 的布局同样要返工。
#    这一格是 2026-09-01 那轮的本地腿当成尺子的洞攻出来的，主 agent 当轮裁定算在内，
#    它当场改掉一条判定：D12 未定项 3 的一个候选只在 mkfs 写。）
#     能   ⇒ 否
#     不能 ⇒ 是
#
# 这一步不能省，省了两条结构一样的分项会判出相反的结果：
# D18 未定项 7（块头要不要第六个字段）与 D5 未定项 1（维度元组有哪几维）都带着
# 「先按最小的写、以后再加」，但 D18 已定项 3 是一条**正式已定**的五字段集合，
# 第一个事务的块头由它唯一确定 ⇒ 否；而 D5 没有任何已定项钉住维度元组的内容，
# 「先按最小维度」是口头方向不是登记的已定项 ⇒ 第一条记账记录的 key 宽度写不出来 ⇒ 是。
#
# ⚠️ **这个名字是刻意不叫「阻塞」的**，而仓里此前三处判定都写作「不阻塞第一行代码」——
# 那个词被同时用作两个意思：①「不改第一个事务写出的字节」（D21 未定项 1：共享单元数为 0
# ⇒ 两个答案逐字节相同）；②「改，但已定案接受先写后改」（D5 未定项 1、D18 未定项 7）。
# **两者在盘上的后果完全不同**：① 不返工，② 要重写记账树 / 重排块头。
# 一个词罩住两件事，正是 decisions-history.md 2026-08-29 其十八 记下的那次
# 「判据用得不一致」的病因。⇒ **标记只回答那把尺**（改不改字节），
# **挡不挡开工是处置，写在依据里**。
# 日期不许省——判定会因为别处定案而过期，没有日期就没法判它是哪一轮的产物
# （`.claude/singlefs-ai-sop/rules/kb-discipline.md`「每条带出处与状态」）。
#
# ⚠️ **只认这一种写法，不认同义的散文。** 散文形态是这条检查诞生的原因：
# 三条已判过的分项用了三种措辞，机器分不开「判过否」与「随口提了一句阻塞」。
#
# ⚠️ **「哪些是未定项」不自己解析**：调 `.claude/scripts/gen-decision-items.py`，
# 与 21 阶段同一个权威解析器。本阶段自己定位登记行，所以另加一道**条数比对**——
# 两侧对不上就说明定位漏了或多了，判红而不是安静地少查几条。
set -uo pipefail
cd "${1:-$(dirname "$0")/../..}" 2>/dev/null || true
DEC=.claude/kb/decisions
# ⚠️ 生成器按**脚本自身的位置**取，不按 cwd——判别力样本会把 cwd 换成一个只放着
# 样本决策文件的临时目录，那里没有 `.claude/scripts/`。按 cwd 取会「找不到生成器 ⇒ 跳过」，
# 于是红样本安静地绿掉，而这条检查看起来一切正常。生成器自己 glob 的是 cwd 下的 kb，正合样本所需。
GEN="$(cd "$(dirname "$0")/../.." 2>/dev/null && pwd)/.claude/scripts/gen-decision-items.py"
[[ -f "$GEN" ]] || GEN=.claude/scripts/gen-decision-items.py
[[ -d "$DEC" ]] || { echo "  ✓ 没有 $DEC，无对象可判"; exit 0; }
[[ -f "$GEN" ]] || { echo "  ! 找不到 $GEN，本阶段跳过"; exit 0; }

python3 - "$DEC" "$GEN" <<'PY'
import re, sys, glob, subprocess, os

dec, gen = sys.argv[1], sys.argv[2]

# ── 权威清单：哪些 (决策号, 分项号) 是未定的 ────────────────────
r = subprocess.run(['python3', gen], capture_output=True, text=True)
if r.returncode != 0:
    print("  ✗ 生成器跑不起来，拿不到未定项清单")
    print('\n'.join('   ' + l for l in r.stderr.splitlines()))
    sys.exit(1)
want = set()
cur = None
for line in r.stdout.splitlines():
    m = re.match(r'- \*\*(D\d+)（', line)
    if m:
        cur = m.group(1); continue
    m = re.match(r'  - (\d+)\. .* —— (.+)$', line)
    if m and cur and '未定' in m.group(2):
        want.add((cur, m.group(1)))

VERDICT = re.compile(r'改第一个事务的字节：\*{0,2}(是|否|无对象)\*{0,2}[（(](\d{4}-\d{2}-\d{2})')

seen, bad = set(), []
for f in sorted(glob.glob(os.path.join(dec, '*.md'))):
    s = open(f, encoding='utf-8').read()
    mm = re.match(r'## (D\d+) ', s.split('\n', 1)[0])
    if not mm:
        continue
    dnum = mm.group(1)
    body = s.split('\n## 历史版本')[0]
    m = re.search(r'^### 未定项\s*$(.*?)(?=^#{1,3} |\Z)', body, re.M | re.S)
    if not m:
        continue
    sec = m.group(1)
    # 索引表 / 编号列表都只取第一个 `####` 之前那一段，与生成器同一口径
    cut = re.search(r'^#{4}\s', sec, re.M)
    top = sec[:cut.start()] if cut else sec
    # 每条分项的「块」= 它的登记行起，到下一条登记行为止（表格形态就是一行）
    starts = [(mo.start(), mo.group(1))
              for mo in re.finditer(r'^(?:\|\s*(\d+)\s*\||(?=\d))(\d+)?\.?\s', top, re.M)]
    marks = [(mo.start(), mo.group(1) or mo.group(2))
             for mo in re.finditer(r'^\|\s*(\d+)\s*\||^(\d+)\.\s', top, re.M)]
    for i, (pos, num) in enumerate(marks):
        end = marks[i + 1][0] if i + 1 < len(marks) else len(top)
        block = top[pos:end]
        if (dnum, num) not in want:
            continue          # 已定项不在这一节，理论上不会到这里；到了就是解析漂了
        seen.add((dnum, num))
        if not VERDICT.search(block):
            first = block.strip().splitlines()[0]
            bad.append((os.path.basename(f), dnum, num, first[:70]))

missed = want - seen
extra = seen - want
if missed or extra:
    print("  ✗ 登记行定位与生成器对不上，本阶段这一轮查的不是全部未定项")
    for d, n in sorted(missed):
        print(f"     定位不到：{d} 未定项 {n}")
    for d, n in sorted(extra):
        print(f"     多定位出：{d} 未定项 {n}")
    print("     → 「### 未定项」一节要有索引表（`| # | 分项 | 状态 |`）或顶格编号列表，每条分项一行")
    sys.exit(1)

if bad:
    print(f"  ✗ {len(bad)} 条未定项没有判过改不改第一个事务的字节：")
    for fn, d, n, first in bad:
        print(f"     {fn}  {d} 未定项 {n}：{first}")
    print("     → 怎么办：按「改不改变第一个事务写出的字节」这把尺判一次，")
    print("               在该分项的登记行里写上规范形态：")
    print("               改第一个事务的字节：否（YYYY-MM-DD，依据：…）")
    print("               三个合法取值：是 / 否 / 无对象。**两侧要用同一把尺**——")
    print("               判「不阻塞」用这把尺、判「阻塞」换一把，量出来的集合不是包含关系")
    print("               （decisions-history.md 2026-08-29 其十八 实测）。")
    sys.exit(1)

print(f"  ✓ {len(seen)} 条未定项都判过改不改第一个事务的字节")
PY
