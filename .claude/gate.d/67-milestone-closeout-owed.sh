#!/usr/bin/env bash
# gate-stage: 里程碑收口表收全了文件里点名、还开着的欠账号
#
# 实测（2026-09-17 核出）：.claude/kb/milestone/02-second-txn.md「增补 2」的收口表立表时 13 行，
# 同一文件别处点名、.claude/kb/checks-owed.md 里还开着的 C 编号（C374、C329、C330、C331、C287 等）表里一个字没提，
# 当天派四个只读核查才补进第 5–34 行；此前没有任何检查报警（那张表第 34 行）。
#
# 判据：
#   ① 收口表用一行标记指明：`<!-- milestone:closeout-table -->`，紧挨着写在表上方（中间只许空行），一份里程碑文件至多一处。
#   ② 还开着的欠账号：checks-owed.md 里「### 已还清」标题之前、表格首列是 C<编号> 的行。
#   ③ 带标记的文件，全文（含历史版本节）出现的每个 C<编号>，若还开着，要么在收口表的某一行里出现，
#      要么在表后（表结束到下一个标题之前）有一行显式豁免，形态：`- 不收口 C<编号>（简称）：为什么不收`。
#   任一个开着的编号两处都没有 ⇒ 红。豁免行写坏（没有编号、冒号后面是空的）、同一个编号既豁免又进了表、
#   标记后面不是表、一份文件两处标记 ⇒ 也红。
#
# 不判的：没有标记的里程碑文件，成功行逐个列出（现算）；一份带标记的文件都没有 ⇒ 退 77，不记通过。
# ⚠️ 管不到的：表里那一行写的去向对不对、豁免的理由站不站得住（靠人）；编号只按字面 C<数字> 认，
# 写成「C 374」或只写简称的引用它看不见；已还清的编号、checks-owed.md 里没有的编号一律不管。
# 判别力：fixtures/67-milestone-closeout-owed.sh/red 放一个漏收的开着编号、一个既豁免又进表的编号、一行空理由的豁免、
# 一份标记后面没有表的文件，必须判红；green 放表里一个、豁免一个、已还清一个与一份不带标记的文件，必须判绿并报对数。
#
#   bash .claude/gate.d/67-milestone-closeout-owed.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
python3 - <<'PY'
import glob, os, re, sys

milestone_dir = ".claude/kb/milestone"
owed_path = ".claude/kb/checks-owed.md"
marker = "<!-- milestone:closeout-table -->"
owed_number = re.compile(r"(?<![A-Za-z0-9_])(C[0-9]+)(?![0-9])")
exemption_start = re.compile(r"^- 不收口 ")
exemption_form = re.compile(r"^- 不收口 (?P<number>C[0-9]+)(?:（.*?）)?：(?P<reason>.*\S.*)$")

milestone_files = sorted(glob.glob(os.path.join(milestone_dir, "*.md")))
if not milestone_files:
    print(f"  ! {milestone_dir} 下没有里程碑文件，本阶段无对象可判")
    sys.exit(77)

marked_files = []
unmarked_files = []
for path in milestone_files:
    lines = open(path, encoding="utf-8").read().split("\n")
    if any(line.strip() == marker for line in lines):
        marked_files.append((path, lines))
    else:
        unmarked_files.append(os.path.basename(path))

if not marked_files:
    print(f"  ! {milestone_dir} 下 {len(milestone_files)} 份里程碑文件没有一份带收口表标记 {marker}，本阶段无对象可判；没判：{'、'.join(unmarked_files)}")
    print(f"    要让本阶段判一份里程碑：在它的收口表上方单独写一行 {marker}，表后逐行写豁免「- 不收口 C<编号>（简称）：为什么不收」。")
    sys.exit(77)

if not os.path.isfile(owed_path):
    print(f"  ✗ 没有 {owed_path}，分不出哪些欠账号还开着")
    print(f"     → 怎么办：确认欠账表还在 {owed_path}；搬过家就照 .claude/rules/path-moves.md 改，并改本阶段的路径。")
    sys.exit(1)

open_owed_names = {}
found_paid_heading = False
for line in open(owed_path, encoding="utf-8"):
    if re.match(r"^#+\s*已还清\s*$", line):
        found_paid_heading = True
        break
    match = re.match(r"^\|\s*(C[0-9]+)\s*\|\s*([^|]*?)\s*\|", line)
    if match:
        open_owed_names[match.group(1)] = match.group(2)
if not found_paid_heading or not open_owed_names:
    print(f"  ✗ {owed_path} 里认不出还开着的欠账：找到「已还清」标题 {found_paid_heading}，标题之前首列是 C<编号> 的表格行 {len(open_owed_names)} 行")
    print(f"     → 怎么办：本阶段按「### 已还清 之前、表格首列是 C<编号>」认开着的账；欠账表改了形状，就同步改本阶段的解析，别让它对着一张认不出的表判绿。")
    sys.exit(1)

failed = False
per_file_counts = []
for path, lines in marked_files:
    name = os.path.basename(path)
    marker_line_indexes = [index for index, line in enumerate(lines) if line.strip() == marker]
    if len(marker_line_indexes) > 1:
        failed = True
        print(f"  ✗ {path} 有 {len(marker_line_indexes)} 处收口表标记（第 {', '.join(str(index + 1) for index in marker_line_indexes)} 行）")
        print(f"     → 怎么办：一份里程碑只留一张收口表、一处标记；另一张表要收的项并进收口表。")
        continue
    index = marker_line_indexes[0] + 1
    while index < len(lines) and not lines[index].strip():
        index += 1
    if index >= len(lines) or not lines[index].lstrip().startswith("|"):
        failed = True
        print(f"  ✗ {path} 第 {marker_line_indexes[0] + 1} 行的收口表标记后面第一处非空内容不是表格")
        print(f"     → 怎么办：把 {marker} 挪到收口表表头的正上方（中间只许空行）；还没有表就先删掉标记，本阶段对这份文件报未判。")
        continue
    table_start = index
    while index < len(lines) and lines[index].lstrip().startswith("|"):
        index += 1
    table_text = "\n".join(lines[table_start:index])
    numbers_in_table = set(owed_number.findall(table_text))

    exempted_numbers = {}
    malformed_exemptions = []
    while index < len(lines) and not lines[index].startswith("#"):
        line = lines[index]
        if exemption_start.match(line):
            match = exemption_form.match(line)
            if match:
                exempted_numbers[match.group("number")] = index + 1
            else:
                malformed_exemptions.append(f"第 {index + 1} 行：{line}")
        index += 1

    first_line_by_number = {}
    for line_number, line in enumerate(lines, 1):
        for number in owed_number.findall(line):
            first_line_by_number.setdefault(number, line_number)
    open_numbers = sorted((number for number in first_line_by_number if number in open_owed_names), key=lambda number: int(number[1:]))
    missing_numbers = [number for number in open_numbers if number not in numbers_in_table and number not in exempted_numbers]
    contradicted_numbers = sorted((number for number in exempted_numbers if number in numbers_in_table), key=lambda number: int(number[1:]))

    if malformed_exemptions:
        failed = True
        print(f"  ✗ {path} 收口表后面这些豁免行写坏了（没有编号，或冒号后面没写为什么不收）：")  # gate-lint:summary
        for entry in malformed_exemptions:
            print(f"     {entry}")  # gate-lint:detail
        print("     → 怎么办：一行豁免一个编号，形态「- 不收口 C<编号>（简称）：为什么不收」，冒号后面写它归哪个里程碑、为什么不在这里还。")
    if contradicted_numbers:
        failed = True
        print(f"  ✗ {path} 这些编号豁免了，又写进了收口表——两处说反话：")  # gate-lint:summary
        for number in contradicted_numbers:
            print(f"     {number}（{open_owed_names.get(number, '已还清或欠账表里没有')}）：豁免在第 {exempted_numbers[number]} 行")  # gate-lint:detail
        print("     → 怎么办：这个里程碑要还它，就删掉豁免行；不还，就把它从收口表里拿掉，豁免行写清归哪。")
    if missing_numbers:
        failed = True
        print(f"  ✗ {path} 点名了这些还开着的欠账号，收口表里没有、表后也没有豁免：")  # gate-lint:summary
        for number in missing_numbers:
            print(f"     {number}（{open_owed_names[number]}）：第 {first_line_by_number[number]} 行起出现")  # gate-lint:detail
        print("     → 怎么办：在收口表里给它一行（或并进已有的一行，写清性质与去向），")
        print("               或在表后加一行「- 不收口 C<编号>（简称）：为什么不收」；它其实已经还了，就先把 checks-owed.md 那一行挪进「### 已还清」。")
    per_file_counts.append(f"{name}：全文点名 {len(first_line_by_number)} 个 C 编号，其中还开着 {len(open_numbers)} 个：表里 {len([number for number in open_numbers if number in numbers_in_table])} 个、豁免 {len([number for number in open_numbers if number in exempted_numbers and number not in numbers_in_table])} 个")

if failed:
    sys.exit(1)
print(f"  ✓ 里程碑收口表收全了文件里点名、还开着的欠账号（判了 {len(marked_files)} 份；{'；'.join(per_file_counts)}）")
print(f"    没判 {len(unmarked_files)} 份（没有收口表标记）：{'、'.join(unmarked_files) if unmarked_files else '无'}")
PY
