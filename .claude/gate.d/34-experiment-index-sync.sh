#!/usr/bin/env bash
# gate-stage: 实验索引行与正文标题说的是不是同一件事
#
# `experiments.md` 的索引表是检索这批实验的入口，而编号的登记位在各正文的
# `## E<n> 简称 —— 状态` 那一行。**两处各写各的，就会一处改对、一处停在旧值**——
# 单独检索到索引那一行的人拿到的是作废结论，而它看起来完全正常。
#
# ⚠️ **这条是实测出来的**（2026-09-06 一轮实验↔决策一致性核查）：
# E69（反向索引取权威态的增量维护代价）的正文标题从 2026-08-31 起就是「结论整体作废」，
# 而索引行状态列写着「已跑（08-31，8 单测）」、结论列**原样登记着**被判作废的那两句
# （「无共享时恰好 0」是计数器从没自增过的仪表伪影，「按盘容量计费」被判为那个实现的性质）。
# E45（事务跨多条记录的环占用）与 E61（反向链 hash 算法的均匀性）的索引行都带了作废标记，
# 只有那一行没带 —— 一处漏改，五天没人看得出来。
#
# 查三样：
#   1. 正文标题带「结论作废 / 结论整体作废」的，索引行里必须也出现「作废」
#   2. 正文标题的状态词是「未跑」或「部分已跑」的，索引状态列必须跟着说
#   3. 索引结论列里的数，正文里要找得到（千位分隔的空格先归一）
#   4. 两边一一对应：正文有而索引无 = 入了正文没进索引，索引有而正文无 = 指到空处
#
# ⚠️ **它抓不到的那一半要说清楚**：一个旧值只要还以历史叙述的形态留在正文里
# （E17（write buffer 合并上限）的「从 31.6M 抬到 31.8M」就是这样），
# 第 3 项照样判它「找得到」。**同一轮核查里 E17 与 E79（根记录的容量）两行的旧值都是人比出来的，
# 这个阶段拦不住它们** —— 拦得住的是第 1 项那一类：正文已经宣告作废而索引只字未提。
#
#   bash .claude/gate.d/34-experiment-index-sync.sh
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
IDX=.claude/kb/experiments.md
EXP=.claude/kb/experiments
[[ -f "$IDX" && -d "$EXP" ]] || { echo "  ! 找不到 $IDX 或 $EXP，本阶段跳过"; exit 0; }

python3 - "$IDX" "$EXP" <<'PY'
import re, sys, glob, os

idx_path, exp_dir = sys.argv[1], sys.argv[2]

def unspace(s):
    for ch in (" ", " ", " "):
        s = s.replace(ch, " ")
    return re.sub(r"(?<=\d)[ 　](?=\d\d\d)", "", s)

bodies = {}
for p in glob.glob(os.path.join(exp_dir, "*.md")):
    n = int(os.path.basename(p).split("-")[0])
    text = open(p, encoding="utf-8").read()
    bodies[n] = (os.path.basename(p), text.splitlines()[0], unspace(text))

rows = []
for line in open(idx_path, encoding="utf-8"):
    if not line.startswith("| E"):
        continue
    cells = [c.strip() for c in line.strip().strip("|").split("|")]
    m = re.match(r"E(\d+)", cells[0])
    if m and len(cells) >= 3:
        rows.append((int(m.group(1)), cells[1], cells[2], line))

def status_seg(title):
    seg = title.split("——", 1)[1] if "——" in title else title
    return seg.split("：", 1)[0]

bad = []
for n in sorted(set(bodies) - {r[0] for r in rows}):
    bad.append(f"E{n} 有正文（{bodies[n][0]}）却不在 experiments.md 的索引表里")
for n, st, concl, raw in rows:
    if n not in bodies:
        bad.append(f"E{n} 索引有行，{exp_dir}/ 下没有正文文件")
        continue
    fname, title, text = bodies[n]
    # 一：正文宣告结论作废，索引一个字不提
    # 只看状态列与结论列：简称里碰巧带「作废」二字（例如一个叫「…作废…」的实验）
    # 不能算索引说了这件事。
    if re.search(r"结论(整体)?作废|结论已作废", title) and "作废" not in (st + concl):
        bad.append(f"E{n} 正文标题写着结论作废，索引行里没有「作废」二字（{fname}）")
    # 二：状态词
    seg = status_seg(title)
    if seg.strip().startswith("未跑") and "未跑" not in st:
        bad.append(f"E{n} 正文状态是「未跑」，索引状态列写「{st}」")
    if "部分已跑" in seg and "部分" not in st:
        bad.append(f"E{n} 正文状态是「部分已跑」，索引状态列写「{st}」")
    # 三：索引结论列里的数，正文里找得到吗
    c = unspace(concl)
    for mm in re.finditer(r"\d[\d.]*", c):
        v = mm.group(0).rstrip(".")
        prev = c[max(0, mm.start() - 3):mm.start()]
        if re.search(r"[EDCIK\-]$", prev) or re.match(r"^20\d\d$", v) or len(v) <= 1:
            continue
        if v not in text:
            bad.append(f"E{n} 索引结论列的「{v}」在 {fname} 正文里找不到")

if bad:
    print(f"  ✗ 实验索引与正文对不上 {len(bad)} 处：")   # gate-lint:summary
    for b in bad:
        print("      " + b)                                # gate-lint:detail
    print("  → 怎么办：以**正文标题**为准改索引行（编号的登记位是正文那行 `## E<n> 简称 —— 状态`，")
    print("    索引表只是导航）；若该改的是正文，就先改正文再回来改索引，并在")
    print("    .claude/kb/experiments-history.md 里按「改前 / 改后 / 依据」记一条。")
    print("    正文有而索引无的那一类：把它登记进 experiments.md 的索引表（编号、状态、一句话结论、正文链接）。")
    sys.exit(1)

print(f"  ✓ 实验索引行与正文标题一致（索引 {len(rows)} 行、正文 {len(bodies)} 份）")
PY
