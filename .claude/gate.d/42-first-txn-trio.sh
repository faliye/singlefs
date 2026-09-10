#!/usr/bin/env bash
# gate-stage: 第一个事务的三份文件互相挂钩（字节表 / 里程碑 / 决策索引）
#
# 三份文件说的是同一件事的三面：
#   decisions/*.md 里每条未定项判过「改第一个事务的字节：是 / 否」（31 阶段管「判过没有」）；
#   first-txn-layout.md 把第一个事务的每个字节列成表，判「是」的未定项在它的「还剩几处空白」表里各占一行；
#   milestone-first-txn.md 把写这些字节的活拆成步，每步写「写出的字节」在字节表哪几节、「会碰到的决策点」有哪些。
#
# **它拦的是三面对不上**（2026-09-10 实测三处）：字节表的空白表里躺着一条 2026-09-06 已定案的分项
# （D8 已定项 7）、判「是」的 D2 未定项 15 与 D19 未定项 8 在字节表与里程碑里一处都没有、
# 里程碑说「六个根槽」而字节表按已定项 8 只有三份。三处门禁全绿——没有任何阶段同时看两份文件。
#
# 判据（每条都报绝对数，扫到 0 也报出来）：
#   1. 判「是」且仍未定的每一条 `D<n> 未定项 <k>`，字节表与里程碑各至少引一次。
#   2. 字节表「还剩几处空白」表里引到的未定项，必须都在第 1 条那个集合里（定了就要从表里拿走）。
#   3. 字节表每一节（历史版本与空白表除外）标题下紧跟一行 `**里程碑**：… 步 N`，N 是里程碑里存在的 `## 步 N`。
#   4. 里程碑每个 `## 步 N` 有一行 `**写出的字节**：`，要么写「无」，要么点名字节表里存在的节标题（「…」引起来的那段）；
#      被字节表某节标成归属的步，不许写「无」。
#
# 射程（`.claude/singlefs-ai-sop/rules/show-me-test.md`「没实现的要明说」）：
#   只认「改第一个事务的字节：**是**（日期…」这一种写法（31 阶段定的规范形态，括号是判据的一部分）；
#   判定写在未定项小节之外的散文抓不到，转述别的分项判过「是」的引号句不算。
#   引用形态只认 `D<n>（简称） 未定项 <k>`，与 32 阶段同一形态。不判引用处说的内容对不对。
#
#   bash .claude/gate.d/42-first-txn-trio.sh
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
FTL=.claude/kb/first-txn-layout.md
MS=.claude/kb/milestone-first-txn.md
DEC=.claude/kb/decisions
[[ -f "$FTL" && -f "$MS" && -d "$DEC" ]] || { echo "  ! 找不到 $FTL / $MS / $DEC，本阶段跳过"; exit 0; }

python3 - "$FTL" "$MS" "$DEC" <<'PY'
import re, sys, glob, os

ftl_path, ms_path, dec_dir = sys.argv[1:4]
ftl = open(ftl_path, encoding="utf-8").read().split("\n## 历史版本")[0]
ms = open(ms_path, encoding="utf-8").read().split("\n## 历史版本")[0]
REF = re.compile(r"D(\d+)（[^）]*）\s*未定项\s*(\d+)")
# 只认带日期括号的规范形态；「……判过「改第一个事务的字节：是」」这种转述引用不算（D26 未定项 5 实测误判）
YES = re.compile(r"改第一个事务的字节：\**是\**（")

# 1. 判「是」且仍未定的分项集合
blocking = set()
for p in sorted(glob.glob(os.path.join(dec_dir, "*.md"))):
    m = re.match(r"(\d+)-", os.path.basename(p))
    if not m:
        continue
    dnum = int(m.group(1))
    body = open(p, encoding="utf-8").read().split("\n## 历史版本")[0]
    sec = re.search(r"^### 未定项.*?(?=^### |\Z)", body, re.M | re.S)
    if not sec:
        continue
    entries = re.split(r"^(?=\| *\d+ *\||\s*\d+\. )", sec.group(0), flags=re.M)
    for e in entries:
        m2 = re.match(r"\| *(\d+) *\||\s*(\d+)\. ", e)
        if not m2:
            continue
        k = int(m2.group(1) or m2.group(2))
        if YES.search(e):
            blocking.add((dnum, k))

in_ftl = {(int(a), int(b)) for a, b in REF.findall(ftl)}
in_ms = {(int(a), int(b)) for a, b in REF.findall(ms)}
bad = []
for d, k in sorted(blocking):
    if (d, k) not in in_ftl:
        bad.append(f"D{d} 未定项 {k} 判了「改第一个事务的字节：是」，而 {ftl_path} 一处都没引它")
    if (d, k) not in in_ms:
        bad.append(f"D{d} 未定项 {k} 判了「改第一个事务的字节：是」，而 {ms_path} 一处都没引它")

# 2. 空白表只许装第 1 条那个集合里的分项
gap = re.search(r"^## 还剩几处空白.*?(?=^## |\Z)", ftl, re.M | re.S)
gap_refs = set()
if gap:
    for l in gap.group(0).splitlines():
        if l.startswith("|"):
            gap_refs |= {(int(a), int(b)) for a, b in REF.findall(l)}
for d, k in sorted(gap_refs - blocking):
    bad.append(f"{ftl_path} 的「还剩几处空白」表引着 D{d} 未定项 {k}，而它今天不在「判是且未定」的集合里（定了，或判的不是「是」）")

# 3. 字节表每节标里程碑步号
steps = {int(x) for x in re.findall(r"^## 步 (\d+)", ms, re.M)}
ftl_lines = ftl.splitlines()
sections, tagged_steps = 0, set()
for i, l in enumerate(ftl_lines):
    if not re.match(r"^#{2,3} ", l) or l.startswith("## 还剩几处空白"):
        continue
    sections += 1
    nxt = next((x for x in ftl_lines[i + 1:] if x.strip()), "")
    m = re.match(r"\*\*里程碑\*\*：.*步 (\d+)", nxt)
    if not m:
        bad.append(f"{ftl_path}:{i + 1} 「{l.strip()}」标题下面没有一行「**里程碑**：… 步 N」")
        continue
    ns = {int(x) for x in re.findall(r"步 (\d+)", nxt)}
    for n in ns:
        if n not in steps:
            bad.append(f"{ftl_path}:{i + 2} 标着里程碑步 {n}，而 {ms_path} 里没有「## 步 {n}」")
    tagged_steps |= ns

# 4. 里程碑每步一行「写出的字节」
headings = [re.sub(r"^#{2,3} ", "", l).strip() for l in ftl_lines if re.match(r"^#{2,3} ", l)]
ms_lines = ms.splitlines()
step_lines = [(i, int(m.group(1))) for i, l in enumerate(ms_lines) if (m := re.match(r"^## 步 (\d+)", l))]
for idx, (i, n) in enumerate(step_lines):
    end = step_lines[idx + 1][0] if idx + 1 < len(step_lines) else len(ms_lines)
    block = ms_lines[i:end]
    row = next((x for x in block if x.startswith("**写出的字节**：")), None)
    if row is None:
        bad.append(f"{ms_path}:{i + 1} 「## 步 {n}」里没有「**写出的字节**：」这一行")
        continue
    if re.match(r"\*\*写出的字节\*\*：无", row):
        if n in tagged_steps:
            bad.append(f"{ms_path}:{i + 1} 步 {n} 写「写出的字节：无」，而 {ftl_path} 有节标着归它")
        continue
    quoted = re.findall(r"「([^」]+)」", row)
    if "first-txn-layout.md" not in row or not quoted:
        bad.append(f"{ms_path}:{i + 1} 步 {n} 的「写出的字节」既不是「无」，也没点名 first-txn-layout.md 里的节")
        continue
    for q in quoted:
        if not any(q in h for h in headings):
            bad.append(f"{ms_path}:{i + 1} 步 {n} 点名「{q}」，而 {ftl_path} 没有这一节")

if bad:
    print(f"  ✗ 第一个事务的三份文件对不上 {len(bad)} 处：")                    # gate-lint:summary
    for b in bad:
        print("      " + b)                                                     # gate-lint:detail
    print(f"  → 怎么办：判「是」的未定项要同时进 {ftl_path} 的「还剩几处空白」表和 {ms_path} 某一步的「会碰到的决策点」；")
    print("    定了案的就从空白表里拿走；字节表每节标题下写「**里程碑**：… 步 N」，里程碑每步写「**写出的字节**：无」或点名字节表的节标题。")
    sys.exit(1)

print(f"  ✓ 第一个事务的三份文件互相挂钩（判「是」的未定项 {len(blocking)} 条、字节表 {sections} 节、里程碑 {len(steps)} 步）")
PY
