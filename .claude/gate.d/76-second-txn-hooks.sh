#!/usr/bin/env bash
# gate-stage: 第二个事务的三处互相挂钩（layout/02 登记表 / 里程碑各步「写出的字节」/ layout/01 段序列登记表）
#
# 里程碑「第二个事务」的字节登记在三处：`layout/02-second-txn.md` 那张表登记第一次出现的盘上形态，
# 每行指里程碑的一步与钉住它的用例；里程碑每步的「**写出的字节**：」点名 layout/01 的节与 layout/02 的行；
# `layout/01-first-txn.md`「八、根槽写路径的段序列登记表」的行标「里程碑「第二个事务」步 N」。
# 42 号只管第一个事务那三份，这三处没有任何阶段同时看两份（里程碑「第二个事务」增补 2 收口表第 31 行）。
#
# 判据（每条都报绝对数）：
#   1. layout/02 表每一行「里程碑那一步」点名的步 N 在里程碑里存在，且步 N 的「写出的字节」行用「」点名了这一行的形态（前缀即可）。
#   2. 里程碑里每一行「写出的字节」用「」点名的东西，要么是 layout/01 的节标题（子串即可），要么是 layout/02 表某一行形态的前缀；
#      点名 layout/02 某一行的步 N，那一行的「里程碑那一步」里要有步 N。
#   3. layout/02 表「钉住它的用例」里反引号的 `*.rs` 在 `crates/*/tests/` 下存在，反引号的函数名在同一格点名的文件里有 `fn 名字`。
#   4. 步 N 的「写出的字节」点名了 layout/01 的「八、」那一节，当且仅当「八、」那张表有一行写「里程碑「第二个事务」步 N」。
#
# 射程：只认「」引起来的点名与反引号里的用例名；点名的内容与字节对不对，这里不判（字节由用例与层 0 钉）。
#
#   bash .claude/gate.d/76-second-txn-hooks.sh
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
L1=.claude/kb/layout/01-first-txn.md
L2=.claude/kb/layout/02-second-txn.md
MS=.claude/kb/milestone/02-second-txn.md
[[ -f "$L1" && -f "$L2" && -f "$MS" ]] || { echo "  ! 找不到 $L1 / $L2 / $MS，本阶段无对象可判"; exit 77; }

python3 - "$L1" "$L2" "$MS" <<'PY'
import glob, re, sys

l1_path, l2_path, ms_path = sys.argv[1:4]
body = lambda path: open(path, encoding="utf-8").read().split("\n## 历史版本")[0]
l1, l2, ms = body(l1_path), body(l2_path), body(ms_path)
headings = [re.sub(r"^#{2,3} ", "", line).strip() for line in l1.splitlines() if re.match(r"^#{2,3} ", line)]
split_cells = lambda line: [cell.strip() for cell in re.split(r"(?<!\\)\|", line.strip().strip("|"))]
plain = lambda text: text.replace("`", "")

rows, in_table = [], False
for line in l2.splitlines():
    if line.startswith("| 形态 |"):
        in_table = True
        continue
    if in_table and line.startswith("|---"):
        continue
    if in_table and line.startswith("|"):
        rows.append(split_cells(line))
        continue
    in_table = False

bad = []
if not rows:
    bad.append(f"{l2_path} 里找不到表头是「| 形态 |」的登记表，或表里一行都没有")
rows = [row for row in rows if len(row) >= 6] if rows else rows

ms_lines = ms.splitlines()
step_starts = [(index, int(match.group(1))) for index, line in enumerate(ms_lines) if (match := re.match(r"^## 步 (\d+)", line))]
step_bytes_line = {}
for position, (start, number) in enumerate(step_starts):
    end = step_starts[position + 1][0] if position + 1 < len(step_starts) else len(ms_lines)
    step_bytes_line[number] = next((line for line in ms_lines[start:end] if line.startswith("**写出的字节**：")), "")

def rows_named_by(quote):
    return [row for row in rows if plain(row[0]).startswith(plain(quote))]

# 1. 登记表一行 → 里程碑那一步
row_links = 0
for row in rows:
    for number in [int(x) for x in re.findall(r"步 (\d+)", row[4])]:
        row_links += 1
        if number not in step_bytes_line:
            bad.append(f"{l2_path}「{row[0][:30]}」指里程碑步 {number}，而 {ms_path} 里没有「## 步 {number}」")
            continue
        quotes = re.findall(r"「([^」]+)」", step_bytes_line[number])
        if not any(row in rows_named_by(quote) for quote in quotes):
            bad.append(f"{l2_path}「{row[0][:30]}」指里程碑步 {number}，而步 {number} 的「写出的字节」没点名这一行")

# 2. 里程碑每行「写出的字节」的点名都落得到
quotes_checked = 0
for index, line in enumerate(ms_lines):
    if not line.startswith("**写出的字节**："):
        continue
    step_number = next((number for start, number in reversed(step_starts) if start < index), None)
    for quote in re.findall(r"「([^」]+)」", line):
        quotes_checked += 1
        named_rows = rows_named_by(quote)
        if not named_rows and not any(quote in heading for heading in headings):
            bad.append(f"{ms_path}:{index + 1} 点名「{quote[:40]}」，而 {l1_path} 没有这一节、{l2_path} 表里也没有这一行")
            continue
        for row in named_rows:
            if step_number is not None and str(step_number) not in re.findall(r"步 (\d+)", row[4]):
                bad.append(f"{ms_path}:{index + 1} 步 {step_number} 点名 {l2_path}「{row[0][:30]}」，而那一行的「里程碑那一步」里没有步 {step_number}")

# 3. 钉住它的用例在仓里
tests_checked = 0
for row in rows:
    tokens = re.findall(r"`([^`]+)`", row[5])
    files = [token for token in tokens if token.endswith(".rs")]
    functions = [token for token in tokens if re.fullmatch(r"[a-z_][a-z0-9_]*", token)]
    paths = []
    for file_name in files:
        tests_checked += 1
        found = glob.glob(f"crates/*/tests/{file_name}")
        if not found:
            bad.append(f"{l2_path}「{row[0][:30]}」点名的用例文件 {file_name} 在 crates/*/tests/ 下不存在")
        paths += found
    sources = [open(path, encoding="utf-8").read() for path in paths]
    for function in functions:
        tests_checked += 1
        if not any(re.search(rf"\bfn {re.escape(function)}\b", source) for source in sources):
            bad.append(f"{l2_path}「{row[0][:30]}」点名的用例 {function} 在同一格点名的文件里找不到 fn")

# 4. 段序列登记表的行与里程碑各步对得上
registry = re.search(r"^## 八、.*?(?=^## |\Z)", l1, re.M | re.S)
registry_steps = set()
if registry:
    for line in registry.group(0).splitlines():
        if line.startswith("|"):
            registry_steps |= {int(x) for x in re.findall(r"里程碑「第二个事务」\s*步 (\d+)", line)}
else:
    bad.append(f"{l1_path} 里没有「## 八、」那一节（段序列登记表）")
steps_naming_registry = {number for number, line in step_bytes_line.items() if re.search(r"「八、[^」]*段序列登记表", line)}
for number in sorted(registry_steps - steps_naming_registry):
    bad.append(f"{l1_path}「八、」有一行标着里程碑「第二个事务」步 {number}，而步 {number} 的「写出的字节」没点名「八、」那一节")
for number in sorted(steps_naming_registry - registry_steps):
    bad.append(f"{ms_path} 步 {number} 的「写出的字节」点名了「八、」那一节，而那张表里没有一行标「里程碑「第二个事务」步 {number}」")

if bad:
    print(f"  ✗ 第二个事务的三处挂钩对不上 {len(bad)} 处：")    # gate-lint:summary
    for item in bad:
        print("      " + item)                                # gate-lint:detail
    print(f"  → 怎么办：{l2_path} 每一行的「里程碑那一步」与那一步的「**写出的字节**：」互相点名（「形态」原样引）；")
    print(f"    点名的节要在 {l1_path} 里存在，用例文件与函数名要在 crates/*/tests/ 里存在；")
    print(f"    某一步写了「八、」段序列那一节，那张表就要有一行标「里程碑「第二个事务」步 N」，反过来也一样。")
    sys.exit(1)

print(f"  ✓ 第二个事务的三处互相挂钩（登记表 {len(rows)} 行、行到步 {row_links} 处、「写出的字节」点名 {quotes_checked} 处、用例 {tests_checked} 个、段序列登记表标到的步 {len(registry_steps)} 个）")
PY
