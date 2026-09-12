#!/usr/bin/env bash
# gate-stage: 字段表加出来的数（表后合计 + format-const 标记）
#
# C94（登记的格式常量与后来的定案对不上）逐字要的第一条：
# 「每个 `format-const` 标记的值，与同一份正文里点名同一个量的全部已定增量之和相等，不等即判红」。
# 本阶段落的是它**能机械判的那一格**：字段表紧跟着的那句「合计 N 字节」。
#
# ⚠️ **这条是实测出来的**（2026-09-07）：D22（单元原子性怎么合成）已定项 7 的根记录字段表
# 2026-09-06 加了「树 ID 水位 8」，正文的合计跟着改成 194，
# **而同文件索引表那一行还写着「合计 186」** —— 一份文件里两个合计，差一整个字段。
# 顺着它查下去，[verification-build.md] 三处、C78、D19 的下游数全停在更早的 127。
#
# 射程（`.claude/singlefs-ai-sop/rules/show-me-test.md`「没实现的要明说」）：
#   1. 只认表头恰好是 `| 字段 | 宽 |` / `| 字段 | 宽度 |` 的表，且合计要在表后 3 行内、
#      写成「合计 N」或「合计 **N**」。别的措辞（「共 N 字节」「+N 字节 ⇒ 共 M」）**一概不判**——
#      D18（块里携带什么信息）已定项 7 那两张表就是这种形态，本阶段对它们一个字也不说。
#   2. 表里只要有一行宽度不是纯数字（「未定」「随实现定」），整张表跳过并计入跳过数：
#      加不出确定的和时判红只会是假红。
#   3. 只判「合计 = 各行之和」，不判各行宽度对不对（那归 27-format-constants 与 C94 的另一半）。
#   4. 第二段检查：一条「已定项 N」里**恰好一个** `format-const` 标记时，标记值要等于该分项下
#      全部字段表之和（`DATA_UNIT_HEADER_BYTES = 105` 对着 D18 已定项 7 那两张表 42 + 63）。一个分项里
#      有多个标记时无从对应，跳过并报出——**跳过的那些本阶段一个字也没验**。
#
#   bash .claude/gate.d/39-field-table-sum.sh
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
[[ -d .claude/kb/decisions ]] || { echo "  ! 找不到 decisions/，本阶段跳过"; exit 0; }

python3 - <<'PY'
import re, glob, os, sys

SUM = re.compile(r"合计\s*\**\s*(\d+)\s*\**\s*字节")
MARK = re.compile(r"<!--\s*format-const:\s*(\w+)\s*=\s*(-?\d+)")
HEAD = re.compile(r"^#{2,6}\s")
ITEM = re.compile(r"^#{3,6}\s*已定项\s*(\d+)")

checked, skipped, nosum, bad = 0, [], 0, []
mark_checked, mark_skipped = 0, []

for p in sorted(glob.glob(".claude/kb/decisions/*.md")):
    lines = open(p, encoding="utf-8").read().split("\n## 历史版本")[0].splitlines()
    # 分项区间：从「已定项 N」标题到下一个任意标题
    item, item_sum, item_marks, item_ok, item_head = None, 0, [], True, 0

    def close_item():
        global item, item_sum, item_marks, item_ok, item_head, mark_checked
        if item is not None and item_marks and item_sum:
            if len(item_marks) != 1:
                mark_skipped.append(f"{os.path.basename(p)} 已定项 {item} 里有 "
                                    f"{len(item_marks)} 个 format-const 标记，对不上哪张表")
            elif item_ok:
                name, val = item_marks[0]
                mark_checked += 1
                if val != item_sum:
                    bad.append(f"{p}:{item_head}  已定项 {item} 登记 {name} = {val}，"
                               f"而这一节的字段表加起来是 {item_sum}")
        item, item_sum, item_marks, item_ok, item_head = None, 0, [], True, 0

    i = 0
    while i < len(lines):
        l = lines[i]
        m = ITEM.match(l)
        if m:
            close_item()
            item, item_head = int(m.group(1)), i + 1
        elif HEAD.match(l) and item is not None:
            close_item()
        if item is not None:
            for nm, v in MARK.findall(l):
                item_marks.append((nm, int(v)))
        c = [x.strip() for x in l.strip().strip("|").split("|")] if l.startswith("|") else []
        if not (len(c) >= 2 and c[0] == "字段" and c[1] in ("宽", "宽度")):
            i += 1
            continue
        head = i + 1
        i += 2
        total, bogus, rows = 0, None, 0
        while i < len(lines) and lines[i].startswith("|"):
            cc = [x.strip() for x in lines[i].strip().strip("|").split("|")]
            if len(cc) >= 2:
                w = re.sub(r"[*`]", "", cc[1]).strip()
                if re.fullmatch(r"\d+", w):
                    total += int(w); rows += 1
                elif w and not set(w) <= set("-: "):
                    bogus = bogus or f"{os.path.basename(p)}:{i+1} 「{cc[0]}」宽度是「{w}」"
            i += 1
        if item is not None:
            if bogus:
                item_ok = False
            else:
                item_sum += total
        m2, ln = None, 0
        for j in range(i, min(i + 3, len(lines))):
            m2 = SUM.search(lines[j])
            if m2:
                ln = j + 1
                break
        if not m2:
            nosum += 1
            continue
        if bogus:
            skipped.append(bogus)
            continue
        checked += 1
        if int(m2.group(1)) != total:
            bad.append(f"{p}:{ln}  写「合计 {m2.group(1)} 字节」，而它上面那张字段表 {rows} 行加起来是 {total}"
                       f"（表头在第 {head} 行）")
    close_item()

if bad:
    print(f"  ✗ 字段表加起来的数与写下来的对不上 {len(bad)} 处：")        # gate-lint:summary
    for b in bad:
        print("      " + b)                                              # gate-lint:detail
    print("  → 怎么办：加错了就改那个数；若是刚加了一行字段，那么**同一轮要一起改的还有**——")
    print("    该决策索引表那一行里的合计、这一节的 format-const 标记、")
    print("    [first-txn-layout.md] 对应那一节，以及全仓引过这个数的地方")
    print("    （`.claude/singlefs-ai-sop/rules/evidence-discipline.md`：撤回一个数要当场回扫谁在引它）。")
    sys.exit(1)

msg = (f"  ✓ 字段表加出来的数都对得上（表后合计 {checked} 张，"
       f"format-const 标记 {mark_checked} 个）")
if skipped:
    msg += f"，跳过 {len(skipped)} 张（有非数字宽度）"
if nosum:
    msg += f"，另有 {nosum} 张表后 3 行内没写「合计 N 字节」、本阶段没验它们"
print(msg)
for x in skipped + mark_skipped:
    print(f"     ! 跳过：{x}")

PY
