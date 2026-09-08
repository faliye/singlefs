#!/usr/bin/env bash
# gate-stage: 决策里定的字段有没有漏投影进第一个事务的表
#
# `32-first-txn-fields.sh` 查的是**正向**：[first-txn-layout.md] 表里每个字段都指得到一条真实分项。
# **反向没人查**：一条分项的字段表里新加了一行，而投影表没跟着加。
#
# ⚠️ **这条是实测出来的**（2026-09-07）：D8（核心索引结构）已定项 8 ② 于 2026-09-06 把
# **树 ID 水位 8 字节**加进根记录（D22 已定项 7 的字段表当天就加了这一行，合计 186 → 194），
# 而 [first-txn-layout.md] 第七节「发布（根记录与根槽）」那张表一个字没动 —— 漏了整整一行，
# 正向检查全程判绿，因为它只问「表里已有的字段指不指得到分项」。
#
# 射程（`.claude/singlefs-ai-sop/rules/show-me-test.md`「没实现的要明说」）：
#   1. 只认表头恰好是 `| 字段 | 宽 |` 或 `| 字段 | 宽度 |` 的表。散在段落里的字段定义抓不到。
#   2. **只检查已经被投影过的分项**——[first-txn-layout.md] 里出现过 `D<n>（简称） 已定项 <k>` 的那些。
#      理由：不是每条分项都属于第一个事务（口径见 [first-txn-layout.md] 开头：不含快照、加密、目录树），
#      对没被投影的分项要求投影会假红。**代价是：一条从头到尾就没进过投影表的分项，本阶段一个字也不说。**
#   3. 字段名按子串比对，只判「在不在」，不判宽度值对不对（宽度归 27-format-constants 与 C94）。
#
#   bash .claude/gate.d/38-field-table-projection.sh
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
FTL=.claude/kb/first-txn-layout.md
[[ -f "$FTL" && -d .claude/kb/decisions ]] || { echo "  ! 找不到 $FTL 或 decisions/，本阶段跳过"; exit 0; }

python3 - "$FTL" <<'PY'
import re, sys, glob, os

ftl_path = sys.argv[1]
ftl = open(ftl_path, encoding="utf-8").read().split("\n## 历史版本")[0]

# 投影表引用过哪些分项：D<n>（任意简称） 已定项 <k>
refs = {(int(a), int(b)) for a, b in re.findall(r"D(\d+)（[^）]*）\s*已定项\s*(\d+)", ftl)}

checked_tables, checked_rows, bad = 0, 0, []
for p in sorted(glob.glob(".claude/kb/decisions/*.md")):
    dnum = int(os.path.basename(p).split("-")[0])
    lines = open(p, encoding="utf-8").read().split("\n## 历史版本")[0].splitlines()
    item = None      # 最近一个「已定项 N」标题的编号
    in_tbl = False
    for i, l in enumerate(lines, 1):
        m = re.match(r"#{3,6}\s*已定项\s*(\d+)", l)
        if m:
            item = int(m.group(1))
        if not l.startswith("|"):
            in_tbl = False
            continue
        c = [x.strip() for x in l.strip().strip("|").split("|")]
        if len(c) >= 2 and c[0] == "字段" and c[1] in ("宽", "宽度"):
            in_tbl = item is not None and (dnum, item) in refs
            if in_tbl:
                checked_tables += 1
            continue
        if in_tbl and len(c) >= 2 and re.fullmatch(r"\**\d+\**", c[1]):
            name = re.sub(r"[*`]", "", c[0]).strip()
            if not name:
                continue
            checked_rows += 1
            if name not in ftl:
                bad.append(f"{p}:{i}  D{dnum} 已定项 {item} 的字段表有「{name}」（宽 {c[1]}），"
                           f"而 {ftl_path} 里找不到它")

if bad:
    print(f"  ✗ 分项定了的字段没投影进第一个事务的表 {len(bad)} 处：")     # gate-lint:summary
    for b in bad:
        print("      " + b)                                              # gate-lint:detail
    print(f"  → 怎么办：把缺的那一行补进 {ftl_path} 对应的那一节（字段、宽度、指向哪条分项、状态），")
    print("    它是「第一个事务写出哪些字节」的投影表，漏一行等于那几个字节没有任何人在看；")
    print("    若这个字段确实不属于第一个事务（快照 / 加密 / 目录树），")
    print("    那就把它从这条分项的字段表里挪走，或把该分项从投影表的引用里去掉——两处口径必须一致。")
    sys.exit(1)

print(f"  ✓ 被投影的分项，字段表都投影全了（{checked_tables} 张字段表、{checked_rows} 行；"
      f"投影表引用了 {len(refs)} 条分项）")
PY
