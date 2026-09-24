#!/usr/bin/env bash
# gate-stage: 字段表加出来的数（表后合计、表前说明句、format-const 标记）与字节布局表里结构总宽的登记
#
# C94（登记的格式常量与后来的定案对不上）逐字要的两条：
# 「每个 `format-const` 标记的值，与同一份正文里点名同一个量的全部已定增量之和相等，不等即判红」；
# 「[layout/01-first-txn.md] 表里每个已定宽度都有登记标记，缺即判红」。
# 本阶段落的是它们**能机械判的那几格**：字段表紧跟着的「合计 N 字节」、表前说明句里的总宽与增量、
# 一个分项里唯一的那个标记，以及字节布局表里写成认得出的形态的结构总宽（射程第 7、8 条）。
#
# ⚠️ **这条是实测出来的**（2026-09-07）：D22（单元原子性怎么合成）已定项 7 的根记录字段表
# 2026-09-06 加了「树 ID 水位 8」，正文的合计跟着改成 194，
# **而同文件索引表那一行还写着「合计 186」** —— 一份文件里两个合计，差一整个字段。
# 顺着它查下去，[verification-build.md] 三处、C78、D19 的下游数全停在更早的 127。
#
# 射程（`.claude/singlefs-ai-sop/rules/show-me-test.md`「没实现的要明说」）：
#   1. 只认表头里「字段」一列紧跟着「宽」或「宽度」一列的表（前面可以有「段」这类列：
#      字节布局表的系统配置字段表就是 `| 段 | 字段 | 宽 |`），表后合计要在表后 3 行内、写成「合计 N 字节」或「合计 **N** 字节」。
#   2. 宽度格按整数或只含 ×、+、− 的整数算式读（「8 × 3」「12 + 16 + 1」）；表里只要有一行读不出来
#      （「未定」「随实现定」「—」），整张表跳过并计入跳过数：加不出确定的和时判红只会是假红。
#   3. 只判总宽与各行之和、标记与分项之和、布局里的总宽有没有登记；各行宽度本身对不对不判。
#   4. 第二段检查：一条「已定项 N」里**恰好一个** `format-const` 标记时，标记值要等于该分项下
#      全部字段表之和（`DATA_UNIT_HEADER_BYTES = 105` 对着 D18 已定项 7 那两张表 42 + 63）。一个分项里
#      有多个标记时无从对应，跳过并报出——**跳过的那些本阶段一个字也没验**。
#   5. 扫两处的正文（「## 历史版本」之前）：`.claude/kb/decisions/*.md` 与 `.claude/kb/layout/*.md`。
#      字节布局表里的字段表与决策正文里的一样会写「合计 N 字节」，只扫决策时它们一张都没人核。
#   6. 标记按 `lib-format-const.py` 读（27、92 号用的是同一份）：以 `<!-- format-const` 开头而按文法读不出来的、
#      同一份文件里同一个名字登记了不止一次的，判红——前者会让「一个分项里恰好一个标记」数错，
#      后者两个值里哪个算数本阶段说不清。
#   7. 表前说明句：表头之前最多 3 行非空行里、以 `**` 起头的标题行（D18（块里携带什么信息） 已定项 7 那两张表的
#      「**类身份段——数据单元（+63 字节 ⇒ 共 105 字节初值）**：」）。认两种写法：
#      增量式「+M 字节 ⇒ 共 N 字节」判 M = 这张表之和、N = 这一小节（上一个标题起）到这张表为止全部字段表之和；
#      绝对式「合计 / 共 N 字节」「N 字节初值」「（N 字节，」「定长 N」判 N = 这张表之和。增量式认上了就不再按绝对式判。
#      不是 `**` 起头的普通段落不认：「记录定长 140」说的多半不是紧跟着的那张表。
#   8. 字节布局表（`.claude/kb/layout/*.md`）里的结构总宽要有登记。「结构总宽」只认三种写法：
#      ① 非表格行的「合计 N 字节」；② 以 ⇒ 起头的行里、紧挨着一个求和等式的加粗整数（「**88** = 头部 50 + …」
#      「24 + 88 = **112**」；等式那一侧去掉括注后要有 +，只有 × 的「6 × 55 = **330**」是条数乘宽、不算）；
#      ③ 带「状态」列的表里、状态以「已定」开头的行、宽度格里加粗的整数。
#      登记 = 同一行（①还可以是它那张表的说明句）里有值等于 N 的 `<!-- format-const: 名字 = N -->`，
#      或有「`format-const: 名字`」引用、而那个名字在 kb 正文里登记的值等于 N。缺即判红。
#      ⚠️ 认不出的写法（不加粗的总宽、「107 + 29」这种算式格、各字段自己的宽度）本阶段**一个字也不说**；
#      成功那句报出认出了几处，认出 0 处不是「都登记了」。
#
# 判别力：fixtures/39-field-table-sum.sh/red 放一张合计写错的决策字段表、一个与表和对不上的标记、
# 一张合计写错的布局字段表、一条多写了键的标记与一个同一份文件里登记两次的名字；C94（登记的格式常量与后来的定案对不上） 要的「登记 78 而正文写着
# 两笔增量共 13 字节」（367-样例）；三句写错的表前说明句（369-样例：绝对式、增量、累计各一句）；
# 以及布局里没登记的三种结构总宽、引用了值不对的名字与没登记的名字、`| 段 | 字段 | 宽 |` 带乘式的表合计写错（02-样例），
# 必须判红。green 放同一个增量样本而登记值改成 91（368-样例）、三种写法各一处登记好的总宽、
# 未定行里的加粗宽度与只有 × 的等式（不算），必须判绿。
#
#   bash .claude/gate.d/39-field-table-sum.sh
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
LIB="$(cd "$(dirname "$0")" && pwd)/lib-format-const.py"
cd "$ROOT" 2>/dev/null || exit 2
[[ -d .claude/kb/decisions ]] || { echo "  ! 找不到 decisions/，本阶段跳过"; exit 77; }

python3 - "$LIB" <<'PY'
import re, glob, importlib.util, os, sys

library_spec = importlib.util.spec_from_file_location("format_const", sys.argv[1])
format_const = importlib.util.module_from_spec(library_spec)
library_spec.loader.exec_module(format_const)

SUM = re.compile(r"合计\s*\**\s*(\d+)\s*\**\s*字节")
HEAD = re.compile(r"^#{2,6}\s")
ITEM = re.compile(r"^#{3,6}\s*已定项\s*(\d+)")
# 表前说明句的写法（射程第 7 条）。增量式先认，认上了就不再按绝对式判同一句。
CAPTION_INCREMENT = re.compile(r"\+\s*(\d+)\s*字节\s*⇒\s*共\s*\**\s*(\d+)\s*\**\s*字节")
CAPTION_ABSOLUTE_FORMS = (
    re.compile(r"(?:合计|共)\s*\**\s*(\d+)\s*\**\s*字节"),
    re.compile(r"(\d+)\s*字节初值"),
    re.compile(r"（\s*(\d+)\s*字节[，；）]"),
    re.compile(r"定长\s*(\d+)"),
)
WIDTH_EXPRESSION = re.compile(r"\d+(?:\s*[×+−-]\s*\d+)*")
# 射程第 8 条：字节布局表里认得出的结构总宽
BOLD_WIDTH = re.compile(r"\*\*([^*\d]{0,4})(\d+)\*\*")
BOLD_PURE_WIDTH = re.compile(r"\*\*(\d+)\*\*")
CLAUSE_BOUNDARY = re.compile(r"[；;，,。：:⇒]")
PARENTHETICAL = re.compile(r"（[^（）]*）|\([^()]*\)")
REFERENCE = re.compile(r"format-const:\s*`?([A-Z][A-Z0-9_]*)")


def table_cells(line):
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


def is_separator_row(cells):
    return set("".join(cells)) <= set("-: ")


def width_value(cell):
    """宽度格按整数或只含 ×、+、− 的整数算式读；读不出来返回 None。"""
    text = re.sub(r"[*`]", "", cell).strip()
    if not WIDTH_EXPRESSION.fullmatch(text):
        return None
    expression_value = 0
    for sign, term in re.findall(r"([+−-]?)\s*(\d+(?:\s*×\s*\d+)*)", text):
        product = 1
        for factor in re.split(r"\s*×\s*", term):
            product *= int(factor)
        expression_value += -product if sign in ("−", "-") else product
    return expression_value


def field_table_width_column(cells):
    """表头里「字段」紧跟着「宽 / 宽度」时，返回宽度列的下标。"""
    for index in range(len(cells) - 1):
        if cells[index] == "字段" and cells[index + 1] in ("宽", "宽度"):
            return index + 1
    return None


def caption_lines(lines, header_index):
    """表头之前的说明句：往上最多 3 行非空行，碰到表格行或标题就停。"""
    collected = []
    index = header_index - 1
    while index >= 0 and len(collected) < 3:
        line = lines[index]
        if line.startswith("|") or HEAD.match(line):
            break
        if line.strip():
            collected.append(index)
        index -= 1
    return sorted(collected)


def without_parentheticals(text):
    """去掉括注（可以嵌套）：「头部 50 + 位置条目 14 × 2（`format-const: LOC_ENTRY`）+ 写序 10」只剩算式。"""
    previous = None
    while previous != text:
        previous, text = text, PARENTHETICAL.sub("", text)
    return text


def strip_bold_and_code(text):
    return re.sub(r"[*`]", "", text)


def note_bold_width_in_settled_row(header, cells, line_index, layout_widths):
    """带「状态」列的表里，状态以「已定」开头的行，宽度格里加粗的整数算一处结构总宽。"""
    if is_separator_row(cells) or "状态" not in header:
        return
    width_names = [name for name in ("宽", "宽度") if name in header]
    if not width_names:
        return
    status_index, width_index = header.index("状态"), header.index(width_names[0])
    status = strip_bold_and_code(cells[status_index]) if status_index < len(cells) else ""
    if not status.startswith("已定") or width_index >= len(cells):
        return
    for bold in BOLD_PURE_WIDTH.finditer(cells[width_index]):
        layout_widths.append((line_index + 1, int(bold.group(1)), [line_index], "已定行的加粗宽度"))


# 登记值：kb 正文里全部读得出来的标记（与 27 号同一个取法：不含变更史）
kb_body_paths = [path for path in sorted(glob.glob(".claude/kb/**/*.md", recursive=True))
                 if not path.endswith("-history.md") and "/decisions-history/" not in path]
registered_value_by_name = {}
for path in kb_body_paths:
    body_text = open(path, encoding="utf-8").read().split("\n## 历史版本")[0]
    for mark in format_const.parse_marks(body_text).marks:
        registered_value_by_name.setdefault(mark.name, mark.value)


def registrations_in(scope_text):
    """一段文字里的登记：标记（带值）与 `format-const: 名字` 引用（值取它在 kb 里登记的那个）。"""
    found = [(mark.name, mark.value) for mark in format_const.parse_marks(scope_text).marks]
    for name in REFERENCE.findall(format_const.strip_marks(scope_text)):
        found.append((name, registered_value_by_name.get(name)))
    return found


checked, skipped, tables_without_total, bad = 0, [], 0, []
caption_checked = 0
mark_checked, mark_skipped = 0, []
marker_problems = []
layout_widths_checked, layout_unregistered = 0, []
decision_paths = sorted(glob.glob(".claude/kb/decisions/*.md"))
layout_paths = sorted(glob.glob(".claude/kb/layout/*.md"))

for kb_path in decision_paths + layout_paths:
    body = open(kb_path, encoding="utf-8").read().split("\n## 历史版本")[0]
    lines = body.split("\n")
    is_layout = kb_path in layout_paths
    parsed_marks = format_const.parse_marks(body)
    marks_by_line = {}
    for mark in parsed_marks.marks:
        marks_by_line.setdefault(mark.line_number, []).append((mark.name, mark.value))
    for unparsable in parsed_marks.unparsable:
        marker_problems.append(f"{kb_path}:{unparsable.line_number}  标记按文法读不出来：「{unparsable.excerpt}」")
    for duplicate in parsed_marks.duplicates:
        marker_problems.append(f"{kb_path}  {duplicate.name} 在这一份里登记了 {len(duplicate.line_numbers)} 次"
                               f"（第 {'、'.join(str(line_number) for line_number in duplicate.line_numbers)} 行）")
    # 分项区间：从「已定项 N」标题到下一个任意标题
    item, item_sum, item_marks, item_ok, item_head = None, 0, [], True, 0
    # 小节区间：从任意标题到下一个任意标题，给「+M 字节 ⇒ 共 N 字节」的累计用
    section_sum, section_ok = 0, True
    # 字节布局表里最近一张字段表：「合计 N 字节」那句的登记可以写在它的说明句里
    last_table_end, last_table_caption = -10, []
    layout_widths = []   # (行号, 值, 登记范围的行下标, 怎么认出来的)

    def close_item():
        global item, item_sum, item_marks, item_ok, item_head, mark_checked
        if item is not None and item_marks and item_sum:
            if len(item_marks) != 1:
                mark_skipped.append(f"{os.path.basename(kb_path)} 已定项 {item} 里有 "
                                    f"{len(item_marks)} 个 format-const 标记，对不上哪张表")
            elif item_ok:
                name, value_registered = item_marks[0]
                mark_checked += 1
                if value_registered != item_sum:
                    bad.append(f"{kb_path}:{item_head}  已定项 {item} 登记 {name} = {value_registered}，"
                               f"而这一节的字段表加起来是 {item_sum}")
        item, item_sum, item_marks, item_ok, item_head = None, 0, [], True, 0

    header_of_current_table = None
    line_index = 0
    while line_index < len(lines):
        line = lines[line_index]
        item_match = ITEM.match(line)
        if item_match:
            close_item()
            item, item_head = int(item_match.group(1)), line_index + 1
        elif HEAD.match(line) and item is not None:
            close_item()
        if HEAD.match(line):
            section_sum, section_ok = 0, True
        if item is not None:
            item_marks.extend(marks_by_line.get(line_index + 1, []))
        if not line.startswith("|"):
            header_of_current_table = None
            if is_layout:
                for sum_match in SUM.finditer(line):
                    scope = [line_index] + (last_table_caption if line_index - last_table_end <= 3 else [])
                    layout_widths.append((line_index + 1, int(sum_match.group(1)), scope, "合计"))
                if line.lstrip().startswith("⇒"):
                    for bold in BOLD_WIDTH.finditer(line):
                        after = CLAUSE_BOUNDARY.split(without_parentheticals(line[bold.end():]), 1)[0]
                        before = CLAUSE_BOUNDARY.split(without_parentheticals(line[:bold.start()]))[-1]
                        defines_by_sum = (after.lstrip().startswith("=") and "+" in after) or \
                                         (before.rstrip().endswith("=") and "+" in before)
                        if defines_by_sum:
                            layout_widths.append((line_index + 1, int(bold.group(2)), [line_index], "⇒ 等式"))
            line_index += 1
            continue
        cells = table_cells(line)
        if header_of_current_table is None:
            header_of_current_table = cells
        elif is_layout:
            note_bold_width_in_settled_row(header_of_current_table, cells, line_index, layout_widths)
        width_index = field_table_width_column(cells)
        if width_index is None:
            line_index += 1
            continue
        header_line_number = line_index + 1
        caption = caption_lines(lines, line_index)
        line_index += 2
        table_sum, unreadable_width, summed_rows = 0, None, 0
        while line_index < len(lines) and lines[line_index].startswith("|"):
            row_cells = table_cells(lines[line_index])
            if is_layout:
                note_bold_width_in_settled_row(cells, row_cells, line_index, layout_widths)
            cell = row_cells[width_index] if width_index < len(row_cells) else ""
            value = width_value(cell)
            if value is not None:
                table_sum += value; summed_rows += 1
            elif cell and not is_separator_row([cell]):
                unreadable_width = unreadable_width or (f"{os.path.basename(kb_path)}:{line_index+1} 「{strip_bold_and_code(row_cells[width_index - 1])[:40]}」"
                                  f"宽度是「{strip_bold_and_code(cell)[:40]}」")
            line_index += 1
        header_of_current_table = None
        last_table_end, last_table_caption = line_index - 1, caption
        if item is not None:
            if unreadable_width:
                item_ok = False
            else:
                item_sum += table_sum
        if unreadable_width:
            section_ok = False
        else:
            section_sum += table_sum
        # 表前说明句只认加粗起头的标题行（「**类身份段——数据单元（+63 字节 ⇒ 共 105 字节）**：」）：
        # 普通段落里的「记录定长 140」说的多半不是紧跟着的这张表
        caption_text = format_const.strip_marks(" ".join(lines[index] for index in caption
                                                         if lines[index].lstrip().startswith("**")))
        caption_judged = False
        increment_match = CAPTION_INCREMENT.search(caption_text)
        absolute_match = None
        if not increment_match:
            for form in CAPTION_ABSOLUTE_FORMS:
                absolute_match = form.search(caption_text)
                if absolute_match:
                    break
        if (increment_match or absolute_match) and unreadable_width:
            skipped.append(unreadable_width)
            caption_judged = True
        elif increment_match:
            caption_judged = True
            caption_checked += 1
            increment, cumulative = int(increment_match.group(1)), int(increment_match.group(2))
            if increment != table_sum:
                bad.append(f"{kb_path}:{header_line_number}  表前写「+{increment} 字节」，而这张字段表 {summed_rows} 行加起来是 {table_sum}")
            if not section_ok:
                skipped.append(f"{os.path.basename(kb_path)}:{header_line_number} 表前写「共 {cumulative} 字节」，"
                               f"而同一节前面有宽度不是数的字段表，累计加不出来")
            elif cumulative != section_sum:
                bad.append(f"{kb_path}:{header_line_number}  表前写「共 {cumulative} 字节」，而这一节到这张表为止的字段表加起来是 {section_sum}")
        elif absolute_match:
            caption_judged = True
            caption_checked += 1
            if int(absolute_match.group(1)) != table_sum:
                bad.append(f"{kb_path}:{header_line_number}  表前写「{absolute_match.group(0)}」，而这张字段表 {summed_rows} 行加起来是 {table_sum}")
        sum_match_after, sum_line_number = None, 0
        for following_index in range(line_index, min(line_index + 3, len(lines))):
            sum_match_after = SUM.search(lines[following_index])
            if sum_match_after:
                sum_line_number = following_index + 1
                break
        if not sum_match_after:
            if not caption_judged:
                tables_without_total += 1
            continue
        if unreadable_width:
            if unreadable_width not in skipped:
                skipped.append(unreadable_width)
            continue
        checked += 1
        if int(sum_match_after.group(1)) != table_sum:
            bad.append(f"{kb_path}:{sum_line_number}  写「合计 {sum_match_after.group(1)} 字节」，而它上面那张字段表 {summed_rows} 行加起来是 {table_sum}"
                       f"（表头在第 {header_line_number} 行）")
    close_item()

    for line_number, value, scope, form in layout_widths:
        layout_widths_checked += 1
        found = registrations_in("\n".join(lines[index] for index in scope))
        if any(registered == value for _name, registered in found):
            continue
        shown = "、".join(f"{name} = {registered if registered is not None else '（kb 里没登记）'}"
                         for name, registered in found) or "一个都没有"
        layout_unregistered.append(f"{kb_path}:{line_number}  {form}写出宽度 {value}，这一行的登记：{shown}")

if bad:
    print(f"  ✗ 字段表加起来的数与写下来的对不上 {len(bad)} 处：")        # gate-lint:summary
    for entry in bad:
        print("      " + entry)                                              # gate-lint:detail
    print("  → 怎么办：加错了就改那个数；若是刚加了一行字段，那么**同一轮要一起改的还有**——")
    print("    该决策索引表那一行里的合计、这一节的 format-const 标记、表前说明句里的「+M 字节 ⇒ 共 N 字节」、")
    print("    [layout/01-first-txn.md] 对应那一节，以及全仓引过这个数的地方")
    print("    （`.claude/singlefs-ai-sop/rules/evidence-discipline.md`：撤回一个数要当场回扫谁在引它）。")
if marker_problems:
    print(f"  ✗ format-const 标记读不出来或在同一份文件里重复登记 {len(marker_problems)} 处：")  # gate-lint:summary
    for problem in marker_problems:
        print("      " + problem)                                        # gate-lint:detail
    print("  → 怎么办：读不出来的照 <!-- format-const: 名字 = 整数 stale=旧串|旧串 --> 改写，stale= 之外不许有别的键；")
    print("    它只是正文里举的例子、不是登记，就去掉 <!--，写成「`format-const: 名字 = 值`」。")
    print("    重复的只留定这个值的那一处，别处要提它写成不带 <!-- 的文字（例如「`format-const: 名字`」）。")
    print("    标记的文法只有一份，在 .claude/gate.d/lib-format-const.py，27、92 号按同一份读。")
if layout_unregistered:
    print(f"  ✗ 字节布局表里写出的结构总宽没有登记 {len(layout_unregistered)} 处：")  # gate-lint:summary
    for entry in layout_unregistered:
        print("      " + entry)                                          # gate-lint:detail
    print("  → 怎么办：在同一行（「合计 N 字节」那句也可以写在它那张表的说明句里）登记这个宽度：")
    print("    这个量还没有登记位，就在定它的那一处加 <!-- format-const: 名字 = N -->；")
    print("    已经在某份决策里登记过，就在这一行写「`format-const: 名字`」引用它，门禁按登记值比。")
    print("    名字与 crates/singlefs-format 里的 const 同名时，27 号会拿登记值去核那个 const，要写成整数字面量。")
    print("    这个数其实不是一个结构的总宽，就别用加粗或「合计 N 字节」写它（C94（登记的格式常量与后来的定案对不上））。")
if bad or marker_problems or layout_unregistered:
    sys.exit(1)

message = (f"  ✓ 字段表加出来的数都对得上（扫了 {len(decision_paths)} 份决策、{len(layout_paths)} 份字节布局表；"
           f"表后合计 {checked} 张，表前说明句 {caption_checked} 张，format-const 标记 {mark_checked} 个；"
           f"字节布局表里的结构总宽 {layout_widths_checked} 处都有登记）")
if skipped:
    message += f"，跳过 {len(skipped)} 处（有非数字宽度）"
if tables_without_total:
    message += f"，另有 {tables_without_total} 张表前表后都没写认得出的总宽、本阶段没验它们"
print(message)
for entry in skipped + mark_skipped:
    print(f"     ! 跳过：{entry}")
PY
