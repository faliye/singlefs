#!/usr/bin/env bash
# 门禁 doc-registries 的格函数库：unit-header-fields、mount-transparency-fields、table-shape、closeout-collects-open、paid-cited-tests、audit-contradictions、closeout-row27-preconditions。
# 由 .claude/gate.d/doc-registries.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释；读的 ROOT 与入口里设好的各变量、bad、ok、howto 在调用时才取。

# ── 格 unit-header-fields：单元头字段表与 D18（块里携带什么信息） 逐项比。还 C68（不变量白名单与已定条款打架） 仍欠的 ② 门禁比对工装里能比的那一半，
# 连同 C66（格式级空白没有分项号） 欠的树表单元自己的头字段表（表由写 kb 的一方写，这一格要它在、要它与 D18 对得上）。
# 对象：.claude/kb/layout/01-first-txn.md 里前一行（中间可隔空行）是 <!-- unit-header-fields: <单元名> 码 <n>[ key-width=<k>] --> 的表；「## 历史版本」之后不读。
#   要在的两张：数据单元（码 1）与树表单元（码 2），缺一张、码标错都判红；标记下面紧跟的不是表判红。表头要有「字段」与「宽…」两列，有「段」列的只取写「单元头」的行；
#   宽度格是一个数，或用 +、× 连起来的几个数（全角括号里的注、粗体、反引号与 HTML 注释不算），读不出判红、那张表的偏移不往下比；偏移从 0 起按行累加。
# 判据（D18 取中文原文，lib-item-ref-status.py 的 numbered_files）：
#   ① 五元组（只对码 1）：D18 已定项 3 里的「`(…)`」与「五个字段的宽度：名 宽 + … =」两处的名字要相同；表里「五元组 · <名>」那几行（名字去掉注与结尾的「副本」）
#      与五元组逐项比：少一项、多一项、宽度不同、指向没点名 D18 已定项 3，都判红。
#   ② 偏移与名字：D18 已定项 7 那一节第一张「| 字段 | 宽 | 出处 |」表（共同明文前缀，偏移按行累加）接 D18 已定项 18「| 码 <n> | … |」那一行（「偏移 名 [宽]」用「 / 」隔开，
#      「⇒」之后不读，偏移与宽里的 k 取标记的 key-width，没写宽的取到下一段的偏移，最后一段要写宽）；共同前缀的合计要等于类身份段的起点。每个定下的字段：表里要有一行
#      从它的偏移起，那一行的字段名含定下的名字（或那一行去掉注之后的名字出现在 D18 那一行的原文里，例：共同前缀 flags 那一行写着「偏移 6 = 单元类型标签」），
#      它收尾的偏移上表里要有一行起头（或正是表尾）；表的合计等于定下的头宽；表里从别处起、没有定下的字段盖得住的行判红。一个定下的字段在表里拆成几行（flags 拆成单元类型标签与 flag 位）不判。
#   指到未定项与「无分项」的行归 field-refs 格判（带标记的表照根槽那一节办）。
# 不判：I-6.2（明文头字段白名单） 那条不变量的白名单（在 crates/singlefs-checker/invariants/，管的是加密开启时哪些字段留明文）；码 3 的表（D18 已定项 18 码 3 那一行没写偏移）；字段的值。
# 成功那句报比了几张表、几个字段的偏移与名字、五元组几项。
# 样本：unit-header-fields-green（两张都对得上）；unit-header-fields-five-tuple-removed-red（D18 已定项 3 的五元组与宽度那一句一起删掉锚点偏移，同一份输入由绿转红，
#   C68 ② 的判别力自证）；unit-header-fields-offsets-red（树表单元表少一行出生序号、数据单元表一行五元组的指向没点名已定项 3）；
#   unit-header-fields-missing-red（没有树表单元那张、一个标记下面紧跟的是一段字、一行宽度写「随算法」）。
# 弄坏开关 DOC_REGISTRIES_BREAK=unit-header-fields（整格不跑）、five-tuple-unchecked（① 不判）、header-offsets-unchecked（② 不判）。
cell_unit_header_fields() {
  local TABLE=.claude/kb/layout/01-first-txn.md
  [[ -f "$TABLE" ]] || { echo "  ! 没有 $TABLE，这一格无对象可判"; exit 77; }
  [[ -d .claude/kb/decisions ]] || { echo "  ! 没有 .claude/kb/decisions，这一格无对象可判"; exit 77; }
  local report rc summary tables fields tuple red
  report="$(python3 - "$TABLE" .claude/kb "$ITEM_REF_STATUS_LIBRARY" "$BREAK_SWITCH" 2>&1 <<'PY'
import importlib.util, re, sys

TABLE, KB, LIBRARY = sys.argv[1:4]
BREAK = sys.argv[4] if len(sys.argv) > 4 else ""
spec = importlib.util.spec_from_file_location("item_ref_status", LIBRARY)
item_ref_status = importlib.util.module_from_spec(spec); spec.loader.exec_module(item_ref_status)
DECISION = 18                                     # D18（块里携带什么信息）：五元组在已定项 3，共同前缀在已定项 7，各类的偏移表在已定项 18
REQUIRED = {"数据单元": 1, "树表单元": 2}           # C66 / C68 要的两张：数据单元头（码 1）与树表单元自己的头（码 2）
MARKER = re.compile(r"^<!--\s*unit-header-fields:\s*(\S+)\s+码\s*(\d+)(?:\s+key-width=(\d+))?\s*-->\s*$")
split_cells = lambda line: [cell.strip() for cell in re.split(r"(?<!\\)\|", line.strip().strip("|"))]
bad = []


def plain(text):
    text = re.sub(r"<!--.*?-->", "", text)
    return re.sub(r"[*`]", "", text).strip()


def item_section(text, item):
    found = re.search(rf"^#### 已定项 *{item}(?!\d)[^\n]*\n(.*?)(?=^#{{2,4}} |\Z)", text, re.M | re.S)
    return found.group(1) if found else None


def width_of(text, key_width):
    """宽度格：一个数、几个数相加（「12 + 16 + 1」）或相乘（「2 × 8」），全角括号里的注不算；读不出交 None。"""
    previous = None
    while previous != text:
        previous, text = text, re.sub(r"（[^（）]*）", "", text)
    total = 0
    for term in plain(text).split("+"):
        factors = [factor.strip() for factor in re.split(r"[×*]", term)]
        if not factors or not all(re.fullmatch(r"\d+", factor) for factor in factors):
            return None
        product = 1
        for factor in factors:
            product *= int(factor)
        total += product
    return total


found = item_ref_status.numbered_files(f"{KB}/decisions", DECISION, TABLE)
decision_text = open(found[0], encoding="utf-8").read().split("\n## 历史版本")[0] if found else ""
if not found:
    bad.append(f"{KB}/decisions 里没有 D{DECISION}，单元头字段表没东西可比")

# ── D18 已定项 3：五元组的名字（那一项里的「(…)」）与宽度（「五个字段的宽度：名 宽 + …」）
five_tuple, five_widths = [], {}
section_three = item_section(decision_text, 3) if found else None
if found:
    tuple_match = re.search(r"`\(([^()`]+)\)`", section_three or "")
    widths_match = re.search(r"五个字段的宽度[：:]\s*\**([^=\n]+)=", section_three or "")
    if not tuple_match or not widths_match:
        bad.append(f"{found[0]} 已定项 3 里读不出五元组（定案那一句的「`(…)`」）或它的宽度（「五个字段的宽度：名 宽 + …」）")
    else:
        five_tuple = [name.strip() for name in tuple_match.group(1).split(",") if name.strip()]
        for term in plain(widths_match.group(1)).split("+"):
            pair = re.fullmatch(r"\s*(.+?)\s+(\d+)\s*", term)
            if pair:
                five_widths[pair.group(1)] = int(pair.group(2))
        if sorted(five_widths) != sorted(five_tuple):
            bad.append(f"{found[0]} 已定项 3 的五元组 {five_tuple} 与宽度那一句的名字 {sorted(five_widths)} 对不上")

# ── D18 已定项 7：共同明文前缀（那一节的第一张「| 字段 | 宽 | 出处 |」表），偏移按行累加
prefix = []
section_seven = item_section(decision_text, 7) if found else None
if found:
    header, offset = None, 0
    for line in (section_seven or "").splitlines():
        if not line.startswith("|"):
            if prefix:
                break
            header = None
            continue
        cells = split_cells(line)
        if header is None:
            header = cells
            continue
        if set("".join(cells)) <= set("-: ") or header[:2] != ["字段", "宽"]:
            continue
        width = width_of(cells[1], None)
        if width is None:
            bad.append(f"{found[0]} 已定项 7 共同前缀「{plain(cells[0])}」的宽「{cells[1]}」读不出")
            break
        prefix.append((offset, width, plain(cells[0]), plain(line)))
        offset += width
    if not prefix:
        bad.append(f"{found[0]} 已定项 7 里读不出共同明文前缀那张「| 字段 | 宽 | 出处 |」表")

# ── D18 已定项 18：「| 码 <n> | 偏移 名 [宽] / … ⇒ … |」那一行，k = key 宽
SEGMENT = re.compile(r"^(\d+)(?:\s*\+\s*(\d*)k)?\s+(.+?)(?:\s+(\d*k|\d+))?$")
section_eighteen = item_section(decision_text, 18) if found else None


def class_fields(code, key_width):
    """→ [(偏移, 宽, 名, 原文)]，读不出交 None（原因进 bad）。"""
    row = next((line for line in (section_eighteen or "").splitlines() if re.match(rf"^\| *码 *{code} *\|", line)), None)
    if row is None:
        bad.append(f"{found[0]} 已定项 18 的偏移表里没有「码 {code}」那一行")
        return None
    body = plain(split_cells(row)[1]).split("⇒")[0]
    segments = []
    for piece in [piece.strip() for piece in body.split(" / ") if piece.strip()]:
        hit = SEGMENT.match(piece)
        if not hit:
            bad.append(f"{found[0]} 已定项 18 码 {code} 那一行的「{piece}」读不出偏移与名字")
            return None
        if (hit.group(2) is not None or (hit.group(4) or "").endswith("k")) and key_width is None:
            bad.append(f"码 {code} 的偏移带 k，单元头字段表的标记要写 key-width=<key 宽>")
            return None
        offset = int(hit.group(1)) + (int(hit.group(2) or 1) * key_width if hit.group(2) is not None else 0)
        width = None
        if hit.group(4):
            width = int(hit.group(4)[:-1] or 1) * key_width if hit.group(4).endswith("k") else int(hit.group(4))
        segments.append([offset, width, hit.group(3).strip(), piece])
    for index, segment in enumerate(segments):
        if segment[1] is None:
            if index + 1 == len(segments):
                bad.append(f"{found[0]} 已定项 18 码 {code} 那一行最后一段「{segment[2]}」没写宽，算不出头在哪收尾")
                return None
            segment[1] = segments[index + 1][0] - segment[0]
    return [tuple(segment) for segment in segments]


# ── layout 里带标记的表
lines = open(TABLE, encoding="utf-8").read().split("\n## 历史版本")[0].splitlines()
tables, pending = [], None
for number, line in enumerate(lines, 1):
    marker = MARKER.match(line.strip())
    if marker:
        pending = (marker.group(1), int(marker.group(2)), int(marker.group(3)) if marker.group(3) else None, number)
        continue
    if pending and line.startswith("|"):
        rows, header = [], split_cells(line)
        for row_number in range(number + 1, len(lines) + 1):
            row = lines[row_number - 1]
            if not row.startswith("|"):
                break
            rows.append((row_number, split_cells(row)))
        tables.append((pending, header, rows))
        pending = None
    elif pending and line.strip() and not line.lstrip().startswith("<!--"):
        bad.append(f"{TABLE}:{pending[3]} 的 unit-header-fields 标记下面紧跟的不是表")
        pending = None
seen = {}
for (unit, code, key_width, marker_line), header, rows in tables:
    seen.setdefault(unit, []).append(code)
for unit, code in REQUIRED.items():
    if unit not in seen:
        bad.append(f"{TABLE} 里没有{unit}的头字段表（表前一行写 <!-- unit-header-fields: {unit} 码 {code}[ key-width=<key 宽>] -->）")
    elif code not in seen[unit]:
        bad.append(f"{TABLE} 里{unit}的头字段表标的是码 {'、'.join(map(str, seen[unit]))}，{unit}是码 {code}")

compared_tables = compared_fields = compared_tuple = 0
for (unit, code, key_width, marker_line), header, rows in tables:
    where = f"{TABLE}:{marker_line}（{unit}，码 {code}）"
    if "字段" not in header or not any(name.startswith("宽") for name in header):
        bad.append(f"{where} 那张表的表头没有「字段」与「宽…」两列")
        continue
    name_column = header.index("字段")
    width_column = next(index for index, name in enumerate(header) if name.startswith("宽"))
    segment_column = header.index("段") if "段" in header else None
    pointer_column = header.index("指向") if "指向" in header else None
    entries, offset, readable = [], 0, True
    for row_number, cells in rows:
        if set("".join(cells)) <= set("-: ") or len(cells) != len(header):
            continue
        if segment_column is not None and plain(cells[segment_column]) != "单元头":
            continue
        width = width_of(cells[width_column], key_width)
        if width is None:
            bad.append(f"{TABLE}:{row_number}「{plain(cells[name_column])[:30]}」的宽「{cells[width_column]}」读不出（写一个数，或几个数用 + 、× 连起来），这张表的偏移没往下比")
            readable = False
            break
        entries.append((offset, width, plain(cells[name_column]), row_number, cells[pointer_column] if pointer_column is not None else ""))
        offset += width
    if not readable:
        continue
    compared_tables += 1
    # 五元组（码 1）：表里「五元组 · <名>」那几行与 D18 已定项 3 逐项比，名字、宽度、指向
    if code == 1 and BREAK != "five-tuple-unchecked" and five_tuple:
        listed = {}
        for start, width, name, row_number, pointer in entries:
            hit = re.match(r"五元组\s*·\s*([^（\x28]+)", name)
            if hit:
                listed[re.sub(r"副本$", "", hit.group(1).strip())] = (width, row_number, pointer)
        for name in five_tuple:
            compared_tuple += 1
            if name not in listed:
                bad.append(f"{where} D{DECISION} 已定项 3 的五元组「{name}」，表里没有「五元组 · {name}」那一行")
            elif listed[name][0] != five_widths.get(name):
                bad.append(f"{TABLE}:{listed[name][1]}「五元组 · {name}」宽 {listed[name][0]}，D{DECISION} 已定项 3 定的是 {five_widths.get(name)}")
            elif not re.search(rf"D{DECISION}(?:（(?:[^（）]|（[^（）]*）)*）)? *已定项 *3(?!\d)", listed[name][2]):
                bad.append(f"{TABLE}:{listed[name][1]}「五元组 · {name}」的指向没点名 D{DECISION} 已定项 3")
        for name, (width, row_number, pointer) in listed.items():
            if name not in five_tuple:
                bad.append(f"{TABLE}:{row_number}「五元组 · {name}」不在 D{DECISION} 已定项 3 的五元组 {five_tuple} 里")
    if BREAK == "header-offsets-unchecked" or not prefix:
        continue
    fields = class_fields(code, key_width)
    if fields is None:
        continue
    if prefix[-1][0] + prefix[-1][1] != fields[0][0]:
        bad.append(f"D{DECISION} 已定项 7 的共同前缀合计 {prefix[-1][0] + prefix[-1][1]}，已定项 18 码 {code} 的类身份段从 {fields[0][0]} 起，两处对不上")
        continue
    decided = prefix + fields
    starts = {entry[0]: entry for entry in entries}
    table_end = entries[-1][0] + entries[-1][1] if entries else 0
    for start, width, name, text in decided:
        compared_fields += 1
        row = starts.get(start)
        if row is None:
            bad.append(f"{where} D{DECISION} 定的「{name}」从偏移 {start} 起（宽 {width}），表里没有从 {start} 起的一行（偏移按表里的宽逐行累加）")
            continue
        base = re.split(r"[（\x28：，]", re.sub(r"^五元组\s*·\s*", "", row[2]))[0].strip()
        if name not in row[2] and not (base and base in text):
            bad.append(f"{TABLE}:{row[3]} 从偏移 {start} 起的是「{row[2][:30]}」，D{DECISION} 定的是「{name}」")
        if start + width not in starts and start + width != table_end:
            bad.append(f"{where} D{DECISION} 定的「{name}」占偏移 {start} 到 {start + width - 1}，表里没有一行在 {start + width} 收尾（下一行该从 {start + width} 起）")
    decided_end = decided[-1][0] + decided[-1][1]
    if table_end != decided_end:
        bad.append(f"{where} 表里逐行加起来到 {table_end}，D{DECISION} 已定项 7 加已定项 18 码 {code} 定的头到 {decided_end}")
    for start, width, name, row_number, pointer in entries:
        if not any(field[0] <= start < field[0] + field[1] for field in decided):
            bad.append(f"{TABLE}:{row_number}「{name[:30]}」从偏移 {start} 起，D{DECISION} 定的字段里没有盖住它的")

for item in bad:
    print(f"  ✗ {item}")    # gate-lint:detail
print(f"COUNT\t{compared_tables}\t{compared_fields}\t{compared_tuple}\t{len(bad)}")
sys.exit(1 if bad else 0)
PY
)"
  rc=$?
  summary="$(tail -n 1 <<<"$report")"
  if [[ ( $rc -ne 0 && $rc -ne 1 ) || "$summary" != COUNT$'\t'* ]]; then
    printf '%s\n' "$report" | sed 's/^/  /'
    echo "  ✗ 比单元头字段表的那段 python 没跑成（退出码 $rc）——这一格没判"
    echo "     → 怎么办： 按上面的 traceback 修；没跑成就是没判，不许当成判过了。"
    exit 1
  fi
  IFS=$'\t' read -r _ tables fields tuple red <<<"$summary"
  head -n -1 <<<"$report"   # gate-lint:detail
  if ((red)); then
    echo "  ✗ 单元头字段表与 D18 对不上 $red 处（比了 $tables 张表）"
    echo "     → 怎么办： 表写错了改 $TABLE 那张表：漏的字段补一行，宽度与偏移照 D18 已定项 18 那一类的偏移表，五元组照 D18 已定项 3；"
    echo "                D18 改了定案就照新定案改表；缺表的照红行写的标记在表前一行补上（判据在这一格函数上方的注释）。"
    exit 1
  fi
  echo "  ✓ 单元头字段表与 D18 对得上（比了 $tables 张表、$fields 个字段的偏移与名字、五元组 $tuple 项）"
}

# ── 格 mount-transparency-fields：挂载透明字段表的 kb 一侧对账（C736（挂载透明字段表的 kb 一侧对账门禁格没做））
# harness 一侧（crates/singlefs-verification/src/mount_transparency.rs）有三张按字节区间的字段表 ROOT_RECORD_FIELD_BYTE_RANGES、
# SYSTEM_CONFIGURATION_FIELD_BYTE_RANGES、TREE_TABLE_ENTRY_FIELD_BYTE_RANGES，每段带名字、区间与它引的 D13（验证路线） 已定项 1 逐字段表行名
# （MOUNT_TRANSPARENCY_TABLE_* 字符串常量，值折到下一行的也认）。这一格只读那份源码的文本，不编译。
# 判据：
#   ① 行名：三张表引的 D13 行名与 MOUNT_TRANSPARENCY_TABLE_ROWS_OF_TODAY 里的，每一个都是 D13 中文原文「#### 已定项 1」里表头「| 字段 | 判法 |」那张表某一行的首格；
#      反过来那张表每一行都被引到，或登记在 .claude/gate.d/mount-transparency-d13-rows-without-bytes.tsv（行名<TAB># 理由，理由至少 8 个字；
#      登记了而表里已经没有的判红）。比名字时两边都去掉全角括号里的注、粗体与反引号。
#   ② 名与宽：根记录那张表对 D22 中文原文「#### 已定项 7」的「| 字段 | 宽 |」表（偏移按行累加），系统配置那张表对
#      .claude/kb/layout/01-first-txn.md「**系统配置字段表的偏移**」那一段（「名字 偏移 / … ，合计 N」）；harness 一段的名字按「、」「 / 」拆开，
#      逐个对上 kb 里连着的几行（名字相同），区间正好盖住那几行；kb 里多出来、harness 没盖到的行也判红。改名同宽也红。
# 不判：树表条目那张表的名与宽（kb 里没有能逐行读的树表条目字段表，只核它引的 D13 行名）；每段归哪一类（必须相同、不比……）对不对。
# 样本：mount-transparency-fields-red 里 D13 表删了 harness 引的「卸载记号」一行、多了一行谁都没引的、D22 根记录表把 fsid 改名成同宽的「池号」，逐条点名判红；
#   mount-transparency-fields-green 两边对得上，「POSIX 内容与存在性」登记成不按字节比。
# 弄坏开关 DOC_REGISTRIES_BREAK=mount-transparency-fields（整格不跑）、mount-transparency-names-unchecked（② 只比区间不比名字）。
cell_mount_transparency_fields() {
  local SOURCE=crates/singlefs-verification/src/mount_transparency.rs
  local EXEMPT=.claude/gate.d/mount-transparency-d13-rows-without-bytes.tsv
  [[ -f "$SOURCE" ]] || { echo "  ! 没有 $SOURCE，这一格无对象可判"; exit 77; }
  python3 - .claude/kb "$ITEM_REF_STATUS_LIBRARY" "$SOURCE" "$EXEMPT" "$BREAK_SWITCH" <<'PY'
import importlib.util, os, re, sys

KB, LIBRARY, SOURCE, EXEMPT = sys.argv[1:5]
BREAK = sys.argv[5] if len(sys.argv) > 5 else ""
spec = importlib.util.spec_from_file_location("item_ref_status", LIBRARY)
item_ref_status = importlib.util.module_from_spec(spec); spec.loader.exec_module(item_ref_status)
LAYOUT = f"{KB}/layout/01-first-txn.md"

def decision_file(number):
    found = item_ref_status.numbered_files(f"{KB}/decisions", number, LAYOUT)
    return found[0] if found else None

def clean(text):
    """比名字时去掉全角括号里的注、粗体与反引号、两头空白。"""
    previous = None
    while previous != text:
        previous, text = text, re.sub(r"（[^（）]*）", "", text)
    return re.sub(r"[`*]", "", text).strip()

split_cells = lambda line: [cell.strip() for cell in re.split(r"(?<!\\)\|", line.strip().strip("|"))]
bad, notes = [], []

# ── harness 一侧：三张字段表（名字、字节区间、引的 D13 行名）
if not os.path.isfile(SOURCE):
    print(f"  ! {SOURCE} 不在，这一格无对象可判")
    sys.exit(77)
code = re.sub(r"//[^\n]*", "", open(SOURCE, encoding="utf-8").read())
ROW_NAME_CONSTANT = r"MOUNT_TRANSPARENCY_TABLE_\w+"   # 行名常量的前缀；cargo fmt 把长名字的值折到下一行，= 后面认任意空白
IMMUTABLE_ROW_CONSTANT = "MOUNT_TRANSPARENCY_TABLE_IMMUTABLE_CONFIGURATION"   # immutable_and_the_same 不写行名参数，引的就是这一行
strings = dict(re.findall(rf'const ({ROW_NAME_CONSTANT}): &str =\s*"((?:\\.|[^"\\])*)";', code))
ranges = {name: (int(a), int(b)) for name, a, b in re.findall(r"const (\w+): std::ops::Range<usize> = (\d+)\.\.(\d+);", code)}
CALL = re.compile(r'\b(must_be_the_same|not_compared|determined_relation|immutable_and_the_same)\(\s*"((?:\\.|[^"\\])*)"\s*,\s*(?:(\d+)\.\.(\d+)|(\w+))\s*(?:,([^()]*?))?\)', re.S)
TABLES = {"ROOT_RECORD_FIELD_BYTE_RANGES": "根记录", "SYSTEM_CONFIGURATION_FIELD_BYTE_RANGES": "系统配置",
          "TREE_TABLE_ENTRY_FIELD_BYTE_RANGES": "树表条目"}
harness = {}
for constant, label in TABLES.items():
    match = re.search(rf"pub const {constant}: &\[\w+\] = &\[(.*?)\n\];", code, re.S)
    if not match:
        bad.append(f"{SOURCE} 里读不出 {constant}（{label}字段表）")
        continue
    entries = []
    for call in CALL.finditer(match.group(1)):
        kind, field, start, end, named_range, rest = call.groups()
        span = (int(start), int(end)) if start else ranges.get(named_range)
        if span is None:
            bad.append(f"{SOURCE} {constant} 的「{field}」字节区间写成 {named_range}，读不出它的值")
            continue
        row_constants = [IMMUTABLE_ROW_CONSTANT] if kind == "immutable_and_the_same" else re.findall(rf"\b({ROW_NAME_CONSTANT})\b", rest or "")
        row = strings.get(row_constants[-1]) if row_constants else None
        if row is None:
            bad.append(f"{SOURCE} {constant} 的「{field}」读不出它引的 D13 逐字段表行名")
        entries.append((field, span, row))
    if not entries:
        bad.append(f"{SOURCE} {constant} 一段都读不出")
    harness[label] = entries
today = re.search(r"pub const MOUNT_TRANSPARENCY_TABLE_ROWS_OF_TODAY: \[&str; \d+\] = \[(.*?)\];", code, re.S)
referenced = {row for entries in harness.values() for _, _, row in entries if row}
if today:
    referenced |= {strings[name] for name in re.findall(rf"\b{ROW_NAME_CONSTANT}\b", today.group(1)) if name in strings}

# ── ① D13 已定项 1 挂载透明逐字段表的行名
d13_path = decision_file(13)
d13_rows = []
if d13_path is None:
    bad.append(f"{KB}/decisions/ 下没有 D13 的中文原文")
else:
    text = open(d13_path, encoding="utf-8").read().split("\n## 历史版本")[0]
    section = re.search(r"^#### 已定项 1[：\s].*?(?=^#### |\Z)", text, re.M | re.S)
    inside = False
    for line in (section.group(0).splitlines() if section else []):
        if line.startswith("| 字段 | 判法 |"):
            inside = True
            continue
        if inside and line.startswith("|---"):
            continue
        if inside and line.startswith("|"):
            d13_rows.append(split_cells(line)[0])
            continue
        if inside:
            break
    if not d13_rows:
        bad.append(f"{d13_path}「#### 已定项 1」里读不出表头是「| 字段 | 判法 |」的逐字段表")
exempt = {}
if os.path.isfile(EXEMPT):
    for number, line in enumerate(open(EXEMPT, encoding="utf-8"), 1):
        if not line.strip() or line.startswith("#"):
            continue
        name, _, reason = line.rstrip("\n").partition("\t")
        reason = reason.lstrip("# ").strip()
        if len(reason) < 8:
            bad.append(f"{EXEMPT}:{number}「{name}」的理由不到 8 个字")
        exempt[clean(name)] = number
d13_clean = {clean(row): row for row in d13_rows}
referenced_clean = {clean(row): row for row in referenced}
if d13_rows:
    for name, row in sorted(referenced_clean.items()):
        if name not in d13_clean:
            bad.append(f"harness 字段表引的 D13 行「{row}」在 {d13_path} 的逐字段表里没有（那张表删了这一行或改了名）")
    for name, row in d13_clean.items():
        if name not in referenced_clean and name not in exempt:
            bad.append(f"{d13_path} 逐字段表的「{row}」那一行，harness 字段表一段都没引它，也没登记在 {EXEMPT}")
    for name, number in exempt.items():
        if name not in d13_clean:
            bad.append(f"{EXEMPT}:{number}「{name}」在 {d13_path} 的逐字段表里已经没有，删掉这一行")

# ── ② kb 字段表与 harness 表的（名、宽）
def walk(label, kb_fields, kb_where):
    """kb_fields：[(名, 起始偏移, 宽)] 按盘上次序；harness 每一段按「、」「 / 」拆成几个名字，逐个对上 kb 里连着的几行，区间要正好盖住它们。"""
    entries = harness.get(label) or []
    position, compared = 0, 0
    for field, (start, end), _ in entries:
        parts = [clean(part) for part in re.split(r"、| / ", clean(field)) if clean(part)]
        covered = kb_fields[position:position + len(parts)]
        compared += 1
        if len(covered) < len(parts):
            bad.append(f"harness {label}字段表「{field}」{start}..{end}：{kb_where} 到这里已经没有对得上的行")
            break
        names_differ = [f"「{part}」对 kb「{name}」" for part, (name, _, _) in zip(parts, covered) if part != clean(name)]
        if names_differ and BREAK != "mount-transparency-names-unchecked":
            bad.append(f"harness {label}字段表「{field}」{start}..{end} 与 {kb_where} 名字对不上：{'、'.join(names_differ)}")
        kb_start, kb_end = covered[0][1], covered[-1][1] + covered[-1][2]
        if (start, end) != (kb_start, kb_end):
            bad.append(f"harness {label}字段表「{field}」是 {start}..{end}，{kb_where} 那几行是 {kb_start}..{kb_end}")
        position += len(parts)
    for name, start, width in kb_fields[position:]:
        bad.append(f"{kb_where}「{name}」（{start}，宽 {width}）在 harness {label}字段表里没有对应的一段")
    return compared

compared_root = compared_configuration = 0
d22_path = decision_file(22)
if d22_path is None:
    bad.append(f"{KB}/decisions/ 下没有 D22 的中文原文（根记录字段表在它的已定项 7）")
else:
    text = open(d22_path, encoding="utf-8").read().split("\n## 历史版本")[0]
    section = re.search(r"^#### 已定项 7[：\s].*?(?=^#### |\Z)", text, re.M | re.S)
    fields, offset, header = [], 0, None
    for line in (section.group(0).splitlines() if section else []):
        if not line.startswith("|"):
            if header:
                break
            continue
        cells = split_cells(line)
        if header is None:
            header = cells
            continue
        if set("".join(cells)) <= set("-: "):
            continue
        if header[:2] != ["字段", "宽"]:
            break
        width = re.fullmatch(r"\d+", clean(cells[1]))
        if not width:
            bad.append(f"{d22_path} 已定项 7 根记录字段表「{cells[0]}」的宽「{cells[1]}」不是一个整数")
            continue
        fields.append((clean(cells[0]), offset, int(width.group(0))))
        offset += int(width.group(0))
    if not fields:
        bad.append(f"{d22_path}「#### 已定项 7」里读不出表头是「| 字段 | 宽 |」的根记录字段表")
    else:
        compared_root = walk("根记录", fields, f"{d22_path} 已定项 7 根记录字段表")

if os.path.isfile(LAYOUT):
    line = next((line for line in open(LAYOUT, encoding="utf-8") if line.startswith("**系统配置字段表的偏移**")), None)
    if line is None:
        bad.append(f"{LAYOUT} 里没有「**系统配置字段表的偏移**」那一段")
    else:
        stripped, previous = line, None
        while previous != stripped:
            previous, stripped = stripped, re.sub(r"（[^（）]*）", "", stripped)
        total = re.search(r"^(.*?)，合计 (\d+)", stripped)
        listing = total.group(1).rpartition("：")[2] if total else ""
        starts = []
        for item in re.split(r"\s*/\s*", listing):
            for piece in item.split("、"):
                match = re.fullmatch(r"(.+?) (\d+)", piece.strip())
                if match:
                    starts.append((clean(match.group(1)), int(match.group(2))))
                elif not re.fullmatch(r"\d+", piece.strip()):
                    bad.append(f"{LAYOUT} 系统配置字段表的偏移里「{piece.strip()}」读不出「名字 偏移」")
        if total is None:
            bad.append(f"{LAYOUT} 系统配置字段表的偏移没写「，合计 N」")
        elif starts:
            ends = [start for _, start in starts[1:]] + [int(total.group(2))]
            fields = [(name, start, end - start) for (name, start), end in zip(starts, ends)]
            compared_configuration = walk("系统配置", fields, f"{LAYOUT} 系统配置字段表的偏移")

if bad:
    for item in bad:
        print(f"  ✗ {item}")      # gate-lint:detail
    print(f"  ✗ 挂载透明字段表的 kb 一侧对账有 {len(bad)} 处不对")    # gate-lint:summary
    print("  → 怎么办：D13 逐字段表改了行名，harness 那几个 MOUNT_TRANSPARENCY_TABLE_* 行名常量跟着改（crates/singlefs-verification/src/mount_transparency.rs，交实现员）；")
    print("    kb 字段表改名或改宽，harness 三张 *_FIELD_BYTE_RANGES 那一段的名字与区间跟着改；不按字节比的 D13 行登记进 " + EXEMPT + "（行名<TAB># 理由）。")
    sys.exit(1)
print(f"  ✓ 挂载透明字段表与 kb 对得上：D13 逐字段表 {len(d13_rows)} 行（harness 引 {len(referenced_clean)} 行、不按字节比登记 {len(exempt)} 行），"
      f"根记录 {compared_root} 段、系统配置 {compared_configuration} 段的名字与区间与 kb 相同（树表条目没有能逐行读的 kb 字段表，只核 ①）")
PY
}
# ════════════════ 欠账与收口组的五格（欠账表与里程碑收口表） ════════════════

# ── 格 table-shape：欠账表两张登记表的行形状（已还清那张不许混进六列的欠账行，欠着那张不许留写着已还的行）
# 判据：`.claude/kb/checks-owed.md` 有两张登记表——欠着的那张六列（编号 / 简称 / 要拦什么 / 怎么拦 / 前置 / 出处），
# 「### 已还清」那张四列（编号 / 简称 / 怎么还的 / 还清日期）。一行的格数与它所在那张表的表头对不上，判红；
# 欠着的那张里，一行有哪一格（第 3 列起）开头写着「已还」而后面不是「一半 / 这一半 / 大半」、或开头写着「销账」「作废」
# （前面带 ⚠️ 与日期都算），也判红：还清了就挪走，只还了一部分就把开头写成还欠什么（「已还一半」「决策已定、检查仍欠」「前置已还」）。
# 编号行前面没有表头（被空行从上一张表断开）判红。
# 只看紧挨着 `doc-lint:registry` 标记的表，历史版本里别的表不管；带编号行而没判的表，成功行逐张列出（现算）。
# 「同一个编号在两张表里各登记一次」不在这一格判：两张表都是 kb 的登记位，归上游 doc-lint 的 F（一个编号只许有一处登记位）逐个编号判。
# 样本：checks-owed-red 的欠账表里欠着那张有一行写「已还（日期）」、一行「日期 作废」、一行「日期 销账」，一行被空行断开、前面没有表头，
# 已还清那张混进一行六格的；checks-owed-green 里欠着那张的「已还一半」「已还这一半」「前置已还」、已还清那张里写「销账」的都放行。
cell_table_shape() {
  local F="$ROOT/.claude/kb/checks-owed.md"
  if [[ ! -f "$F" ]]; then
    echo "  ✗ 找不到 $F"
    echo "  → 怎么办：欠账表挪了位置就同步改这个阶段里的路径。"
    exit 1
  fi
  python3 - "$F" <<'PY'
import re, sys
lines = open(sys.argv[1], encoding='utf-8').read().split('\n')
tables, cur, section, marked = [], None, '（开头）', False
headless, in_history = [], False
for i, l in enumerate(lines, 1):
    s = l.strip()
    if s.startswith('<!-- doc-lint:registry'):
        marked = True; cur = None; continue
    if l.startswith('#'):
        section = l.lstrip('#').strip(); cur = None; marked = False
        in_history = in_history or section.startswith('历史版本')
        continue
    if not s.startswith('|'):
        cur = None
        if s:
            marked = False
        continue
    cells = re.sub(r'\\\|', '', s).strip('|').split('|')
    if cur is None:
        # 一张表的第一行就是编号行，说明它前面没有表头：多半是被一个空行从上一张表断开的。
        # Markdown 在空行处结束表格，断开之后的行渲染不成表，也不在 registry 标记管的那张表里。
        if re.match(r'\|\s*C\d+\s*\|', s) and not in_history:
            headless.append(f'第 {i} 行 {s.split("|")[1].strip()}：前面没有表头（上一行是空行或别的正文）')
        cur = {'sec': section, 'width': len(cells), 'head': i, 'rows': [], 'reg': marked}
        tables.append(cur); marked = False; continue
    if re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', s):
        continue
    m = re.match(r'\|\s*(C\d+)\s*\|', s)
    if m:
        cur['rows'].append((i, m.group(1), cells))
reg = [t for t in tables if t['reg'] and t['rows']]
if len(reg) < 2:
    print(f'  ✗ 只找到 {len(reg)} 张带编号行的登记表，欠着的与已还清的两张应当都在')
    print('  → 怎么办：表头或它上面的 doc-lint:registry 标记被改坏了；按文件开头的约定把两张表恢复出来。')
    sys.exit(1)
paid_claim = re.compile(r'(?:20\d\d-\d\d-\d\d\s*)?(已还(?!一半|这一半|大半)|销账|作废)')
bad, claims, checked = [], [], 0
for t in reg:
    for ln, num, cells in t['rows']:
        checked += 1
        if len(cells) != t['width']:
            bad.append(f"第 {ln} 行 {num}：{len(cells)} 格，而它所在那张表（「{t['sec']}」下、表头在第 {t['head']} 行）是 {t['width']} 格")
        if t['sec'] == '已还清':
            continue
        for column_index, cell in enumerate(cells[2:], start=3):
            lead = re.sub(r'^[\s*⚠️]+', '', cell)
            if paid_claim.match(lead):
                claims.append(f"第 {ln} 行 {num}：第 {column_index} 列开头写着「{lead.replace('*', '')[:16]}…」")
if bad or claims or headless:
    print(f'  ✗ 欠账表有 {len(bad)} 行的格数与所在表的表头对不上、{len(claims)} 处还在欠着的表里却写着已还、{len(headless)} 处被空行断开：')  # gate-lint:summary
    for b in bad:
        print('      ' + b)  # gate-lint:detail
    for claim in claims:
        print('      ' + claim)  # gate-lint:detail
    for fragment in headless:
        print('      ' + fragment)  # gate-lint:detail
    print('  → 怎么办：六列的欠账行挪回欠着的那张表，接在它最后一行后面，不接在文件末尾；表中间的空行删掉，一张表从表头到最后一行不许断；')
    print('            真还清了的，按「已还清」那张表的四列改写（编号 / 简称 / 怎么还的 / 还清日期）再挪过去，欠着的那张里删掉；')
    print('            只还了一部分的，把那一格的开头改成写明还欠什么的说法（「已还一半」「决策已定、检查仍欠」「前置已还」）。')
    sys.exit(1)
print(f'  ✓ 欠账表两张登记表的行形状都对，欠着的那张里没有写着已还的行（检查了 {checked} 行，{len(reg)} 张表）')
# 没判的：带 C 编号行、却不紧挨着 registry 标记的表（历史版本里的、别处举例的），逐张列出，清单与被扫集合出自同一趟解析
unjudged_tables = [t for t in tables if not t['reg'] and t['rows']]
print(f"    没判 {len(unjudged_tables)} 张带编号行的表（没有紧挨着的 doc-lint:registry 标记）"
      + ('：' + '；'.join(f"「{t['sec']}」下表头在第 {t['head']} 行、{len(t['rows'])} 行" for t in unjudged_tables) if unjudged_tables else ''))
PY
}

# ── 格 closeout-collects-open：里程碑收口表收全了文件里点名、还开着的欠账号，行号只许是顺序号
# 判据：
#   ① 收口表用一行标记指明：`<!-- milestone:closeout-table -->`，紧挨着写在表上方（中间只许空行），一份里程碑文件至多一处。
#   ② 还开着的欠账号：checks-owed.md 里「### 已还清」标题之前、表格首列是 C<编号> 的行（按 `lib-owed.py` 读）。
#   ③ 带标记的文件，全文（含历史版本节）出现的每个 C<编号>，若还开着，要么在收口表的某一行里出现，
#      要么在表后（表结束到下一个标题之前）有一行显式豁免，形态：`- 不收口 C<编号>（简称）：为什么不收`。
#   任一个开着的编号两处都没有，红。豁免行写坏（没有编号、冒号后面是空的）、同一个编号既豁免又进了表、
#   标记后面不是表、一份文件两处标记，也红。
#   ④ 收口表每一行的首列（表头 `#` 与分隔行除外）要么是纯十进制数字，要么是圆圈号 `①`–`⑳`；
#      带撇号（U+2032、U+2033、ASCII 单引号）或紧跟字母的，红，指名那一行（为什么不许后缀见 path-moves.md「变体起新名字，不用角标」）。
#      全仓的 U+2032、U+2033 另由 doc-text 判；这一条多管的是 ASCII 单引号、字母与其余一切非顺序号的写法，只在收口表首列。
# 没有标记的里程碑文件不判，成功行逐个列出（现算）；一份带标记的文件都没有，退 77。
# 管不到的：表里那一行写的去向对不对、豁免的理由站不站得住（靠人）；编号只按字面 C<数字> 认，
# 写成「C 374」或只写简称的引用看不见；已还清的编号、checks-owed.md 里没有的编号一律不管。
# 样本：checks-owed-red 放一个漏收的开着编号、一个既豁免又进表的编号、一行空理由的豁免、
# 一份标记后面没有表的文件、一行字母后缀与一行 ASCII 单引号后缀的行号，判红；checks-owed-green 放表里一个、豁免一个、已还清一个、
# 一份不带标记的文件，收口表里一个圆圈号、一个一位数与一个两位数的行号，判绿并报对数
# （checks-owed-green 里 closeout-row27-preconditions 那一格的 02-second-txn.md 也带标记，这一格一并判它）。
cell_closeout_collects_open() {
  python3 - "$OWED_LIBRARY" <<'PY'
import glob, importlib.util, os, re, sys

owed_library_spec = importlib.util.spec_from_file_location("owed", sys.argv[1])
owed_library = importlib.util.module_from_spec(owed_library_spec)
owed_library_spec.loader.exec_module(owed_library)

milestone_dir = ".claude/kb/milestone"
owed_path = ".claude/kb/checks-owed.md"
marker = "<!-- milestone:closeout-table -->"
owed_number = re.compile(r"(?<![A-Za-z0-9_])(C[0-9]+)(?![0-9])")
exemption_start = re.compile(r"^- 不收口 ")
exemption_form = re.compile(r"^- 不收口 (?P<number>C[0-9]+)(?:（.*?）)?：(?P<reason>.*\S.*)$")
circled_row_numbers = "①②③④⑤⑥⑦⑧⑨⑩⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳"

milestone_files = [p for p in sorted(glob.glob(os.path.join(milestone_dir, "*.md")))
                   if not p.endswith((".en.md", ".ja.md"))]
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

owed_table = owed_library.read_owed_table(owed_path)
open_owed_names = owed_table.open_names
found_paid_heading = owed_table.paid_heading_found
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
    table_end = index
    table_text = "\n".join(lines[table_start:table_end])
    numbers_in_table = set(owed_number.findall(table_text))

    # ④ 行号：表头 `#` 与分隔行之外，每一行的首列只许是顺序号或圆圈号。
    suffixed_row_numbers = []
    judged_row_count = 0
    for row_index in range(table_start, table_end):
        first_cell = lines[row_index].split("|")[1].strip()
        if not first_cell or first_cell.startswith("-") or first_cell == "#":
            continue
        judged_row_count += 1
        if first_cell.isdecimal() or (len(first_cell) == 1 and first_cell in circled_row_numbers):
            continue
        suffixed_row_numbers.append(f"第 {row_index + 1} 行：行号写成「{first_cell}」")

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
    if suffixed_row_numbers:
        failed = True
        print(f"  ✗ {path} 收口表的行号带了后缀（撇号或字母），行号只许是顺序号或圆圈号：")  # gate-lint:summary
        for entry in suffixed_row_numbers:
            print(f"     {entry}")  # gate-lint:detail
        print("     → 怎么办：改成一个没被占用的顺序号（接着表里最大的那个往下取），不要在旧号上加撇号或字母——后缀会让读的人以为这一行从属于那个基号。")
        print("               ⚠️ 改号要连全仓引用一起改（「第 N 行」「跟第 N 行」这类），照 .claude/rules/path-moves.md「怎么做」那六步办。")
    per_file_counts.append(f"{name}：全文点名 {len(first_line_by_number)} 个 C 编号，其中还开着 {len(open_numbers)} 个：表里 {len([number for number in open_numbers if number in numbers_in_table])} 个、豁免 {len([number for number in open_numbers if number in exempted_numbers and number not in numbers_in_table])} 个；收口表 {judged_row_count} 行的行号都是顺序号或圆圈号")

if failed:
    sys.exit(1)
print(f"  ✓ 里程碑收口表收全了文件里点名、还开着的欠账号（判了 {len(marked_files)} 份；{'；'.join(per_file_counts)}）")
print(f"    没判 {len(unmarked_files)} 份（没有收口表标记）：{'、'.join(unmarked_files) if unmarked_files else '无'}")
PY
}

# ── 格 paid-cited-tests：已还清的欠账行里点名的测试名，仓里还在不在
# 判据：`.claude/kb/checks-owed.md` 的「### 已还清」那张表里，反引号括起来的 snake_case 标识符（至少三段）
# 必须在仓里的 `.rs` / `.py` / `.sh` 的代码里按整词找得到；找不到就红。按整词（加了后缀的不算还在）、只认注释之外
# （`.rs` 的 `//` 之后、`.py` / `.sh` 的 `#` 之后不算：旧名只剩在一句注释里，那个测试其实已经没了）。
# 已还清表一行都没有、或行里一个这样的标识符都没有，退 77。
# 已还清的行里点名的单测常是那条决策唯一的记录位，它被改名或删掉时没有别的东西会说话。
# 欠着的那张表不查：那里点名一个还不存在的测试名是它应有的样子。
# 至少三段：放宽到两段会把 `log_append`、`write_cache` 这类词组也扫进来。
# 找法：点名的全部标识符一次交给 grep（-f - 读名单）：一趟 -o 取每处整词命中属于哪个名字，一趟取命中行的行文，名字归属照 grep 自己的整词判法；
#   某个名字有没有「在注释之外的整词命中」仍逐个名字、逐行判（成不成立与逐个名字各起一趟 grep 的写法逐项相同）。
# 注释里不写被拦的那个标识符的整串字面量，搜索也排掉 `.claude/gate.d/`：否则 grep 扫到这个阶段自己，把该红的那一条判成「还在」。
# 不扫 `research/prompts/`：原样发给模型的材料是冻结证据，里面的旧名字不跟着改（evidence-discipline.md「原样保存的证据不许事后改」）。
# 样本：checks-owed-red 的已还清表里一行引的单测改了名、一行引的旧名只剩在注释里、一行引的名字只剩加了后缀的，三个都要点名；
# 欠着那张里一行引一个不存在的名字，不许点名。checks-owed-green 里已还清表引的单测在、另有不带标识符的行进「没查的」。
cell_paid_cited_tests() {
  local OWED=".claude/kb/checks-owed.md"
  if [[ ! -f "$OWED" ]]; then
    echo "  ! 没有 $OWED，本阶段跳过"
    exit 77
  fi

  python3 - "$OWED" "$BREAK_SWITCH" <<'PY'
import pathlib, re, subprocess, sys

owed = pathlib.Path(sys.argv[1])
lines = owed.read_text(encoding="utf-8").splitlines()

settled_heading = None
for index, line in enumerate(lines):
    if line.strip().startswith("### 已还清"):
        settled_heading = index
        break
if settled_heading is None:
    print("  ✗ 找不到「### 已还清」那一节，判不了已还清的行引的测试名还在不在")
    print("     → 怎么办：确认 .claude/kb/checks-owed.md 里那一节的标题没被改写；改了标题就把这一格的取法一起改。")
    raise SystemExit(1)

identifier = re.compile(r"`([a-z][a-z0-9]*(?:_[a-z0-9]+){2,})`")
row_number = re.compile(r"^\|\s*(C\d+)\s*\|")

rows = []
for line in lines[settled_heading + 1:]:
    stripped = line.strip()
    if stripped.startswith("## ") or (stripped.startswith("### ") and not stripped.startswith("### 已还清")):
        break
    # 只认首格是 `C<数字>` 的登记行：表头与分隔行数进去，成功句报的行数会比真实行数多，而那个数没有东西钉着。
    # 一行掉了编号算表的形状坏了，归 table-shape 判。
    if stripped.startswith("|") and stripped.count("|") >= 4 and row_number.match(stripped):
        rows.append(stripped)

cited = []
skipped = []
for row in rows:
    number = row_number.match(row).group(1)
    names = list(dict.fromkeys(identifier.findall(row)))  # 同一行里引两次算一个
    if names:
        for name in names:
            cited.append((number, name))
    else:
        skipped.append(number)

if not rows:
    print("  ⊘ 本次无对象可判：「### 已还清」那张表里一行首格是 C<编号> 的登记行都没有")
    raise SystemExit(77)
if not cited:
    print(f"  ⊘ 本次无对象可判：已还清的 {len(rows)} 行里没有一个至少三段的 snake_case 标识符："
          + "、".join(skipped))
    raise SystemExit(77)

def code_part(line, suffix):
    """去掉注释之后的那一截：.rs 砍掉 `//` 之后，.py / .sh 砍掉行首或空白之后的 `#` 之后。"""
    if suffix == ".rs":
        return line.split("//", 1)[0]
    return re.split(r"(?:^|\s)#", line, maxsplit=1)[0]

def grep_with_names(names, only_matching):
    """点名的全部标识符一次交给 grep（-f - 从标准输入读名单），按字面、按整词找：only_matching 为真时每处整词命中交 `路径:行号:名字`（名字归哪一行按 grep 自己的整词判法），
    否则交 `路径:行号:行文`。grep 只用来缩小候选，注释里的命中在 names_in_the_repository 里逐行剔掉。"""
    found = subprocess.run(
        ["grep", "-rnwF" + ("o" if only_matching else ""), "--include=*.rs", "--include=*.py", "--include=*.sh",
         "--exclude-dir=prompts", "--exclude-dir=target", "--exclude-dir=.git",
         "--exclude-dir=gate.d", "-f", "-", "--", "."],
        input="".join(f"{name}\n" for name in names), capture_output=True, text=True)
    if found.returncode not in (0, 1):
        print(f"  ✗ grep 找已还清的行点名的 {len(names)} 个标识符时出错（退 {found.returncode}）：{found.stderr.strip()[:120]}")
        print("     → 怎么办：按 grep 的报错修好（读不了的目录、权限），再跑；找的时候出错不等于没找到，也不等于找到了。")
        raise SystemExit(1)
    return found.stdout.split("\n")

def names_in_the_repository(names):
    """names 里在仓里的 .rs / .py / .sh 的代码里按整词找得到的那几个（集合）：grep 整词命中的行里，名字落在注释之外才算。
    弄坏开关 paid-cited-batch-names-dropped：交给 grep 的名单少最后一个名字。"""
    searched = names[:-1] if sys.argv[2] == "paid-cited-batch-names-dropped" else names
    if not searched:
        return set()
    hit_texts = {}
    for hit in filter(None, grep_with_names(searched, only_matching=False)):
        path, _, rest = hit.partition(":")
        number, _, text = rest.partition(":")
        hit_texts[(path, number)] = text
    present = set()
    for hit in filter(None, grep_with_names(searched, only_matching=True)):
        path, _, rest = hit.partition(":")
        number, _, name = rest.partition(":")
        text = hit_texts.get((path, number))
        if text is None or name in present:
            continue
        word = re.compile(r"(?<![A-Za-z0-9_])" + re.escape(name) + r"(?![A-Za-z0-9_])")
        if word.search(code_part(text, pathlib.Path(path).suffix)):
            present.add(name)
    return present

present_names = names_in_the_repository(list(dict.fromkeys(name for _, name in cited)))
missing = [(number, name) for number, name in cited if name not in present_names]

if missing:
    for number, name in missing:
        print(f"  ✗ {number} 引的 `{name}` 在仓里的 .rs / .py / .sh 里一处都找不到")  # gate-lint:detail
    print(f"  ✗ 已还清的行里有 {len(missing)} 个点名的标识符已经不在仓里了")  # gate-lint:summary
    print("     → 怎么办：那条欠账是靠这个测试还清的。先 grep 它今天叫什么（多半是改了名），")
    print("               用 scripts/kb/replace-once.py 把行里的旧名字定点换成现在的名字；")
    print("               它要是被删掉了，那这条欠账并没有还清，把整行挪回欠着的那张表。")
    raise SystemExit(1)

print(f"  ✓ 已还清的行引的测试名都还在（{len(rows)} 行，查了 {len(cited)} 个标识符："
      + "、".join(f"{number} {name}" for number, name in cited) + "）")
print(f"     没查的 {len(skipped)} 行（行里没有至少三段的 snake_case 标识符）："
      + "、".join(skipped))
PY
}

# ── 格 audit-contradictions：最近一次总审核登记的文档级矛盾，每一行都要有去向
# `records/` 里的总审核记录第五节登记的文档级矛盾在 kb 里没有落点，腐化了没有别的检查会红
# （C358（总审核第五节的文档级矛盾没有回扫闸））。
# 判据：第五节表里每一行的处置列，要么以「已改」开头，要么点名一个 checks-owed.md 开着那张表里的欠账号
# （只点名已还清的号不算去向：那笔账还了，这一行的矛盾却还在）。
#   「已改」要用「」引至少一句原文，在这一行点名的 kb 文件今天的正文里找得到（归一与点名的规则在下面 QUOTE 旁的注释）。
#   引文去掉编号（D / E / C / I- 号连同简称括注）与 .md 文件名之后至少剩两个字：只引一个编号或一个字，在点名的文件里总找得到。
#   只判引文找不找得到，不判引文说的是不是那一行的矛盾（靠人）。
# 第五节标题认「## 五」后面跟标点、空白或行尾的写法；最近一次总审核里认不出第五节标题，判红；表里一行矛盾都没有，退 77。
# 挂账表 .claude/audit-rows-pending 已关闭：再出现任何一行非注释内容判红，新的矛盾行一律当场给去向。
# 内嵌 python 的退出码要取，而且必须报出 COUNT 那一行，缺一样判红：不取退出码时，python 崩了成功句照印、只是数变空白。
# 样本：checks-owed-red 放一行「未改」且不给欠账号、一行只点名已还清的号、一行挂进已关闭的挂账表、一行「已改」却不引原文、
# 两行「已改」引的原文在点名的文件里找不到而只在别的文件里有（一行按 D 编号点名、一行按文件名点名）、一行引的原文哪里都没有、
# 一行只引了一个编号「D9」，第五节标题写成「## 五 文档级矛盾」（不带顿号），判红；checks-owed-green 放按 D 编号与按文件名点名、
# 原文都在点名的文件里各一行，一行没点名文件而原文在 kb 别处，一行带「「旧说法」0 命中」而 kb 里没有「旧说法」，
# 欠账表开着的那张里有一行正文提到「### 已还清」这几个字（按子串切会把它后面开着的账算成还清），判绿。
# D / E 编号点名的是哪一份：lib-item-ref-status.py 的 numbered_files，总审核记录是中文，取中文原文，不把三语三份并起来找。
# 样本：audit-contradictions-translation-green / -red（原句在中文原文里 / 只在日语译文里）；弄坏开关 KB_NUMBERED_FILE_BREAK=first-match。
# 一个 kb 文件都没点名的行同样只在中文原文里找（lib-item-ref-status.py 的 originals_only）：样本 audit-contradictions-unnamed-translation-red
#   （没点名文件的一行引的原句只在英语译文里有，译文首行与中文不同），判红；弄坏开关 DOC_REGISTRIES_BREAK=read-translations 连译文一起找，判绿。
cell_audit_contradictions() {
  local PENDING=".claude/audit-rows-pending"
  local latest report invariant_files
  latest="$(ls -1 records/*-总审核.md 2>/dev/null | sort | tail -1)"
  if [[ -z "$latest" ]]; then
    echo "  ⊘ 本次未跑：records/ 下没有总审核记录，今天无对象可判"
    exit 77
  fi
  # 登记位读不出判红，不当成候选目录是空的：空的会一份不变量表都找不到，引了不变量的行全按「kb 别处」去找（kb_roots_or_red）
  kb_roots_or_red audit-contradictions "不是 kb 里一份不变量表都没有" "$work_directory/audit-contradictions.roots" || exit 1

  # 不变量表住在哪几份文件：与 declared-counts 那三格同一份 invariant_table_files（登记的 kb 目录里含不变量表行的 .md）
  invariant_files="$(invariant_table_files)"
  if ! report="$(LATEST="$latest" PENDING="$PENDING" OWED_LIBRARY="$OWED_LIBRARY" ITEM_REF_STATUS_LIBRARY="$ITEM_REF_STATUS_LIBRARY" INVARIANT_TABLE_FILES="$invariant_files" BREAK_SWITCH="$BREAK_SWITCH" python3 - 2>&1 <<'PY'
import glob, importlib.util, os, re

latest = os.environ["LATEST"]
pending_path = os.environ["PENDING"]
owed_library_spec = importlib.util.spec_from_file_location("owed", os.environ["OWED_LIBRARY"])
owed_library = importlib.util.module_from_spec(owed_library_spec)
owed_library_spec.loader.exec_module(owed_library)
item_ref_status_spec = importlib.util.spec_from_file_location("item_ref_status", os.environ["ITEM_REF_STATUS_LIBRARY"])
item_ref_status = importlib.util.module_from_spec(item_ref_status_spec)
item_ref_status_spec.loader.exec_module(item_ref_status)

# 挂账表已关闭：任何一行非注释内容都是新挂的账。
if os.path.exists(pending_path):
    for number, line in enumerate(open(pending_path, encoding="utf-8"), 1):
        stripped = line.strip()
        if stripped and not stripped.startswith("#"):
            print("BAD", f"{pending_path}:{number}",
                  "挂账表 2026-09-23 已清空并关闭，不许再加行；这一行的矛盾要当场给去向", sep="\t")

# 只认开着那张表：「已还清」整行标题之后的号是还清了的，点名它们不算去向。切法用共用读法 lib-owed.py：
# 按子串「### 已还清」切，开着的那张表里有一行正文提到这几个字，它后面开着的账就全被算成还清。
owed_table = owed_library.read_owed_table(".claude/kb/checks-owed.md")
if not owed_table.file_found:
    print("BAD", ".claude/kb/checks-owed.md", "欠账表不在，分不出哪些欠账号还开着", sep="\t")
elif not owed_table.paid_heading_found:
    print("BAD", ".claude/kb/checks-owed.md", "认不出「已还清」那一行整行标题，分不出哪些欠账号还开着（不对着一张认不出的表判）", sep="\t")
open_owed = set(owed_table.open_names) if owed_table.paid_heading_found else set()
repaid_owed = set(owed_table.paid_names)

lines = open(latest, encoding="utf-8").read().split("\n")
in_section = False
section_heading = None
rows = []
for number, line in enumerate(lines, 1):
    if section_heading is None and re.match(r"^##\s*五(?:[、.．:：\s]|$)", line):
        in_section = True
        section_heading = (number, line.strip())
        continue
    if in_section and re.match(r"^##\s", line):
        break
    if in_section and line.startswith("| "):
        rows.append((number, line))

rows = [(number, line) for number, line in rows
        if not line.startswith("|---") and not re.match(r"^\|\s*#\s*\|", line)]
if section_heading is None:
    print("BAD", latest, "认不出第五节的标题（要写成「## 五」后面跟标点、空白或行尾），一行矛盾都没判", sep="\t")

# 「已改」要指得到今天的正文：处置列用「」引的原文，按空白与 ** ` 归一之后，要在今天的 kb 正文
# （「## 历史版本」之前，不含变更史）里找得到，而且要在这一行点名的文件里找得到（D / E 编号、C 号 → checks-owed.md、
# I- 号 → 含这一类不变量表行的那几份文件，类号找不到文件的指它们所在目录的 README.md、直接写出的 .md 文件名）；
# 一个 kb 文件都没点名的行才在 kb 全部正文里找。不变量表的文件由格经 INVARIANT_TABLE_FILES 传进来（invariant_table_files，
# 按 .claude/kb-directories 登记的目录找），连同它们所在目录的 README.md 一起算进 kb 正文。
# 不退到全部正文里找：旧条款的原句常被实验页原样引着，改掉之后照样搜得到，会把已经不在的那句当成还在。
# 引号后面紧跟「0 命中」「不在」「没有了」的，说的是那句话已经删掉，不当成指向正文的引文。
QUOTE = re.compile(r"「([^」]+)」")
NEGATED_AFTER = re.compile(r"^\s*(?:字样|都|也|已经)?\s*(?:0\s*命中|不在|没有了)")

def normalized(text):
    return re.sub(r"\s+", " ", re.sub(r"[*`]", "", text)).strip()

REFERENCE = re.compile(r"(?<![A-Za-z0-9_])(?:[DEC]\d+|I-\d+(?:\.\d+)*)(?:（[^）]*）)?|[\w\-]+\.md")

def anchor_text(quote):
    """引文里去掉编号（连同简称括注）与 .md 文件名、标点与空白之后剩下的字：少于两个就指不出正文里的一句话。"""
    return re.sub(r"[\s\W_]+", "", REFERENCE.sub("", normalized(quote)))

kb_body_by_path = {}
for path in sorted(glob.glob(".claude/kb/**/*.md", recursive=True)):
    if path.endswith("-history.md") or "/decisions-history/" in path:
        continue
    kb_body_by_path[path] = normalized(open(path, encoding="utf-8").read().split("\n## 历史版本")[0])

# 弄坏开关 invariant-single-file：不变量文件不进 kb 正文、I- 号一律指 .claude/kb/invariants.md
single_file_reading = os.environ.get("BREAK_SWITCH") == "invariant-single-file"
invariant_files_by_class, invariant_indexes = {}, []
for path in [] if single_file_reading else [line for line in os.environ.get("INVARIANT_TABLE_FILES", "").split("\n") if line]:
    text = open(path, encoding="utf-8").read()
    kb_body_by_path.setdefault(path, normalized(text.split("\n## 历史版本")[0]))
    for number in set(re.findall(r"^\| I-(\d+)\.\d+ ", text, re.M)):
        invariant_files_by_class.setdefault(int(number), set()).add(path)
    index = os.path.join(os.path.dirname(path), "README.md")
    if os.path.isfile(index) and index not in invariant_indexes:
        invariant_indexes.append(index)
for index in invariant_indexes:
    kb_body_by_path.setdefault(index, normalized(open(index, encoding="utf-8").read().split("\n## 历史版本")[0]))

def shown(path):
    """kb 目录下的写成相对 .claude/kb 的路径，别处的（按类拆开的不变量文件）写相对仓根的路径。"""
    return os.path.relpath(path, ".claude/kb") if path.startswith(".claude/kb/") else path

def kb_paths_named_by(text):
    # 编号指的是总审核记录（中文）那一侧的中文原文（numbered_files）；把三语三份并进来的话，中文正文里没有的原句在日语译文里搜得到也算找到
    named = set()
    for number in re.findall(r"(?<![A-Za-z0-9_])D(\d+)(?!\d)", text):
        named |= set(item_ref_status.numbered_files(".claude/kb/decisions", number, latest))
    for number in re.findall(r"(?<![A-Za-z0-9_])E(\d+)(?!\d)", text):
        named |= set(item_ref_status.numbered_files(".claude/kb/experiments", number, latest))
    if re.search(r"(?<![A-Za-z0-9_])C\d+(?!\d)", text):
        named.add(".claude/kb/checks-owed.md")
    for number in re.findall(r"(?<![A-Za-z0-9_])I-(\d+)", text):
        if single_file_reading:
            named.add(".claude/kb/invariants.md")
        else:
            named |= invariant_files_by_class.get(int(number)) or set(invariant_indexes)
    for file_name in re.findall(r"([\w\-]+\.md)\b", text):
        named |= {path for path in kb_body_by_path if os.path.basename(path) == file_name}
    return {path for path in named if path in kb_body_by_path}

checked = 0
fixed_rows, quote_count, unnamed_rows = 0, 0, 0
for number, line in rows:
    cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
    if len(cells) < 4:
        print("BAD", f"{latest}:{number}", "这一行不足四列，读不出处置列", sep="\t")
        continue
    checked += 1
    disposition = cells[3]
    if disposition.startswith("已改"):
        fixed_rows += 1
        affirmed_quotes = [quote.group(1) for quote in QUOTE.finditer(disposition)
                           if not NEGATED_AFTER.match(disposition[quote.end():])]
        if not affirmed_quotes:
            print("BAD", f"{latest}:{number}",
                  f"第 {cells[0]} 行处置列写「已改」，却没有用「」引一句今天正文里的原文（只写「0 命中」「不在了」的那种不算）",
                  sep="\t")
            continue
        named_paths = kb_paths_named_by(" ".join(cells[1:4]))
        if not named_paths:
            unnamed_rows += 1
        # 没点名文件的行只在中文原文里找（取法 lib-item-ref-status.py 的 originals_only）：总审核是中文，中文正文里删掉的原句
        # 在没跟上的译文里还在，算找到就判绿。译文留在 kb_body_by_path 里只给下面「只在 … 里有」那句提示用。弄坏开关 read-translations 连译文一起找
        searched_paths = named_paths or set(item_ref_status.originals_only(
            kb_body_by_path, os.environ.get("BREAK_SWITCH") == "read-translations"))
        for quote in affirmed_quotes:
            quote_count += 1
            wanted = normalized(quote)
            if len(anchor_text(quote)) < 2:
                print("BAD", f"{latest}:{number}",
                      f"第 {cells[0]} 行处置列写「已改」，引的「{quote[:40]}」只是编号或文件名（去掉之后不足两个字），"
                      "在点名的那份文件里总找得到，指不出今天正文里的一句话", sep="\t")
                continue
            if any(wanted in kb_body_by_path[path] for path in searched_paths):
                continue
            elsewhere = sorted(shown(path) for path, text in kb_body_by_path.items()
                               if path not in searched_paths and wanted in text)
            where_else = f"；只在 {'、'.join(elsewhere[:3])} 里有，那一行没点名它" if elsewhere else ""
            named_shown = "、".join(shown(path) for path in sorted(named_paths)) or "kb 全部正文"
            print("BAD", f"{latest}:{number}",
                  f"第 {cells[0]} 行处置列写「已改」，引的「{quote[:40]}」在 {named_shown} 的今天正文里找不到{where_else}",
                  sep="\t")
        continue
    cited = set(re.findall(r"\bC\d+\b", disposition))
    if cited & open_owed:
        continue
    if cited & repaid_owed:
        print("BAD", f"{latest}:{number}",
              f"第 {cells[0]} 行处置列只点名了已还清的 {'、'.join(sorted(cited & repaid_owed))}——账还了，这一行的矛盾还在",
              sep="\t")
        continue
    if cited:
        print("BAD", f"{latest}:{number}",
              f"第 {cells[0]} 行处置列点名 {'、'.join(sorted(cited))}，而 checks-owed.md 里没有这个号",
              sep="\t")
        continue
    print("BAD", f"{latest}:{number}",
          f"第 {cells[0]} 行处置列写「{disposition[:40]}」，既不是「已改」、也没点名一个还开着的欠账号",
          sep="\t")

print("COUNT", latest, checked, len(rows), fixed_rows, quote_count, unnamed_rows,
      len(open_owed), len(repaid_owed), section_heading[0] if section_heading else 0,
      section_heading[1] if section_heading else "认不出", sep="\t")
PY
)"; then
    echo "  ✗ 扫描没跑完：内嵌 python 自己出错了，一行都没判"
    printf '%s\n' "$report" | tail -5 | sed 's/^/      /'   # gate-lint:detail
    echo "    → 怎么办：按上面的报错修 .claude/gate.d/lib/doc-registries-unit-header-and-debt-cells.sh 里 audit-contradictions 那一格的 python；"
    echo "      这一步没跑完就是什么都没查，不是通过。"
    exit 1
  fi

  if ! grep -q '^COUNT' <<<"$report"; then
    echo "  ✗ 扫描没跑完：内嵌 python 没报出 COUNT 那一行（多半是它自己崩了）"
    echo "    → 怎么办：单独跑一遍这一格（--check audit-contradictions）看 python 的报错；没有 COUNT 就是一行都没判，"
    echo "      而汇总行看着与判过了一模一样。"
    exit 1
  fi

  local -a bad
  local where checked total fixed quotes unnamed open_count repaid_count heading_line heading_text why
  mapfile -t bad < <(grep -E '^BAD' <<<"$report")
  IFS=$'\t' read -r _ where checked total fixed quotes unnamed open_count repaid_count heading_line heading_text < <(grep '^COUNT' <<<"$report")

  if ((${#bad[@]})); then
    echo "  ✗ 最近一次总审核第五节有行没有去向（第五节标题：${heading_text}）："
    while IFS=$'\t' read -r _ where why; do
      printf '      %s：%s\n' "$where" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${bad[@]}")
    echo "    → 怎么办：给这一行一个去向——改了就把处置列写成「已改…」，并用「」引一句今天正文里的原文、"
    echo "      点名它住在哪份文件（D / E 编号、C 号、I- 号或文件名）；引的那句今天找不到，就是那处又被改过，"
    echo "      回去现查这一行的矛盾还在不在：不在了就换成今天的原文，又回来了就当它没改。"
    echo "      还没改就在 .claude/kb/checks-owed.md 开着的那张表里立一笔账、把那个编号写进处置列。"
    echo "      只点名已还清的号不算；挂账表 .claude/audit-rows-pending 已关闭，不许往里加行。"
    exit 1
  fi

  if [[ "$total" == 0 ]]; then
    echo "  ⊘ 本次无对象可判：$where 第五节（第 ${heading_line} 行的标题）下面的表里一行矛盾都没有"
    exit 77
  fi
  echo "  ✓ 最近一次总审核第五节每一行都有去向（$where：第五节 ${total} 行，判了 ${checked} 行；写「已改」的 ${fixed} 行引了 ${quotes} 句原文，都在那一行点名的文件的今天正文里找得到；其中 ${unnamed} 行没点名 kb 文件，按 kb 全部正文找）"
  echo "    欠账表按共用读法 lib-owed.py 切：开着 ${open_count} 笔、已还清 ${repaid_count} 笔；第五节标题在第 ${heading_line} 行"
  exit 0
}

# ── 格 closeout-row27-preconditions：收口表第 27 行那几笔欠账的前置有没有进来（进来了就得补会红的用例）
# .claude/kb/milestone/02-second-txn.md 收口表第 27 行点名了一批「今天不可达的欠账」：走不到那一格，就写不出会红的用例。
# 这一格判的是它们各自的前置动没动：每一笔登记一条逐字探针（文件、原文、今天的命中次数）。
#   全部对上：前置一个都没进来，退 77。任一条对不上：那一笔的前置进来了，或者它压着的那段代码改了形态，红，回去重核那一笔。
#   重构那几处也会红：要做的同样是重核，不是把探针的期望值改成新数（那等于把这一笔的描述悄悄换掉）。
# UNPROBED 表是手写的：第 27 行只写「alloc-basis 第三轮转来的四条」（history-kept: 引的是被判的欠账表里的条目名原文）、没有逐条列，现算不出来。
# 能现算的那一半每轮现算、只报不判：第 27 行第二格按「、」「；」切成几笔，探针与 UNPROBED 两边都没有的逐个列名；
#   「alloc-basis 第三轮转来的 N 条」（history-kept: 引的是被判的欠账表里的条目名原文）拿 N 与这里名字以「alloc-basis 第三轮」起头的笔数比。对不上是这张表手写得不全，要人回去补。
# 样本：checks-owed-red 的 instance_table.rs 里多了一处 `rows.retain(`（C333 那一笔的前置进来了），判红并点名那一笔；
# checks-owed-green 的探针逐条对上今天的值，这一格退 77，并报第 27 行现算的笔数与 alloc-basis 那一笔的条数对照。
cell_closeout_row27_preconditions() {
  local PROBES UNPROBED report row27_report probe_count item_count unprobed_count name why line
  local -a bad
  # 四段：笔名、文件、原文（逐字）、今天的命中次数。同一笔可以有几条探针。
  PROBES="$(cat <<'TSV'
豁免集与隔离集按记录起点做 key	crates/singlefs-core/src/mount.rs	(slot.0..end_slot).map(move |covered_slot| (device.0, covered_slot))	1
扣住位在空发布循环之后无条件放开	crates/singlefs-core/src/mount.rs	    allocator.release_reclaim_holds();	2
alloc-basis 第三轮 ①：挂载内回收	crates/singlefs-core/src/mount.rs	reclaim_released_up_to(	3
C333（删行那次发布被重放）：行回收	crates/singlefs-core/src/instance_table.rs	rows.retain(	0
C333（删行那次发布被重放）：行回收	crates/singlefs-core/src/instance_table.rs	rows.remove(	0
TSV
)"

  # 做不成逐字探针的那几笔，逐个列名——它们同样开着，只是这一格够不着。
  UNPROBED="$(cat <<'TSV'
alloc-basis 第三轮 ②：根槽写失败推进一格重发	这条路径今天不存在，没有一处逐字文本代表「它进来了」
alloc-basis 第三轮 ③：扣住配今天的 checker 放过「扣住位被撤掉」那份坏镜像	扣住位只住内存、盘上没有落点，没有可扫的对象
TSV
)"

  if ! report="$(PROBES="$PROBES" python3 - 2>&1 <<'PY'
import os

probes = []
for line in os.environ["PROBES"].split("\n"):
    if not line.strip():
        continue
    name, path, original, expected = line.split("\t")
    probes.append((name, path, original, int(expected)))

for name, path, original, expected in probes:
    try:
        source = open(path, encoding="utf-8").read()
    except FileNotFoundError:
        print("BAD", name, f"{path} 不在了（探针指着的文件没了）", sep="\t")
        continue
    hits = source.count(original)
    if hits != expected:
        print("BAD", name, f"{path} 里「{original.strip()}」命中 {hits} 次，登记的今天的值是 {expected} 次", sep="\t")
print("CHECKED", len(probes), len({name for name, _, _, _ in probes}), sep="\t")
PY
)"; then
    echo "  ✗ 探针没跑完：内嵌 python 自己出错了，一条都没核"
    printf '%s\n' "$report" | tail -5 | sed 's/^/      /'   # gate-lint:detail
    echo "    → 怎么办：按上面的报错修 .claude/gate.d/lib/doc-registries-unit-header-and-debt-cells.sh 里 closeout-row27-preconditions 那一格的 python；"
    echo "      没跑完不是「本次未跑」，是这一格自己坏了。"
    exit 1
  fi

  if ! grep -q '^CHECKED' <<<"$report"; then
    echo "  ✗ 探针没跑完：内嵌 python 没报出 CHECKED 那一行（多半是它自己崩了）"
    echo "    → 怎么办：单独跑一遍这一格（--check closeout-row27-preconditions）看 python 的报错；没有 CHECKED 就是一条都没核，"
    echo "      而「本次未跑」那句看着与核过了一模一样。"
    exit 1
  fi

  mapfile -t bad < <(grep '^BAD' <<<"$report")
  read -r _ probe_count item_count < <(grep '^CHECKED' <<<"$report")

  if ((${#bad[@]})); then
    echo "  ✗ 收口表第 27 行那几笔欠账的前置动了（或者它压着的那段代码改了形态）："
    while IFS=$'\t' read -r _ name why; do
      printf '      %s：%s\n' "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${bad[@]}")
    echo "    → 怎么办：先回 .claude/kb/milestone/02-second-txn.md 收口表第 27 行重核这一笔今天可不可达。"
    echo "      可达了就当场做成一条会红的用例（改坏一行、看着它红），把这一笔从第 27 行挪出去，探针那一行一起删；"
    echo "      只是重构、这一笔还不可达，就把探针的原文改成今天逐字存在的那一段，命中次数照实写，并在报告里写明重核过。"
    echo "      ⚠️ 别只把期望值改成新数：那等于把这一笔的描述换掉，而外面看不出换过。"
    exit 1
  fi

  unprobed_count="$(grep -c . <<<"$UNPROBED")"
  row27_report="$(PROBES="$PROBES" UNPROBED="$UNPROBED" python3 - 2>&1 <<'PY'
import os, re
path = ".claude/kb/milestone/02-second-txn.md"
names = [line.split("\t")[0] for key in ("PROBES", "UNPROBED") for line in os.environ[key].split("\n") if line.strip()]
if not os.path.isfile(path):
    print(f"读不到 {path}，第 27 行现算对照没做")
    raise SystemExit(0)
row = next((line for line in open(path, encoding="utf-8") if re.match(r"^\|\s*27\s*\|", line)), None)
if row is None:
    print(f"{path} 里找不到收口表第 27 行，现算对照没做")
    raise SystemExit(0)
cell = row.strip().strip("|").split("|")[1].strip()
cell = re.sub(r"^[^：]*：", "", cell, count=1)
items, depth, current = [], 0, ""
for character in cell:   # 只在括号外按「、」「；」切：括注里的「步 3、步 4」不是两笔
    depth += character in "（("
    depth -= character in "）)" and depth > 0
    if character in "、；" and depth == 0:
        items.append(current.strip()); current = ""
    else:
        current += character
items = [item for item in items + [current.strip()] if item]
declared_alloc_basis = None
uncovered = []
for item in items:
    counted = re.match(r"^alloc-basis 第三轮转来的([一二三四五六七八九十0-9]+)条$", item)
    if counted:
        digits = "一二三四五六七八九十"
        text = counted.group(1)
        declared_alloc_basis = int(text) if text.isdigit() else digits.index(text) + 1 if len(text) == 1 else None
        continue
    if not any(item.startswith(name.split("：")[0]) or name.startswith(item) for name in names):
        uncovered.append(item)
listed_alloc_basis = len({name.split("：")[0] for name in names if name.startswith("alloc-basis 第三轮")})
print(f"收口表第 27 行现算 {len(items)} 笔；探针与没做成探针的清单两边都没有的 {len(uncovered)} 笔：{'、'.join(uncovered) or '（没有）'}")
if declared_alloc_basis is not None:
    print(f"「alloc-basis 第三轮转来的」那一笔第 27 行写 {declared_alloc_basis} 条，这里探针与清单合计 {listed_alloc_basis} 条"
          + ("" if declared_alloc_basis == listed_alloc_basis else "，对不上：清单是手写的，回去按那一轮判决补齐"))
PY
)"
  echo "  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（${probe_count} 条逐字探针、覆盖 ${item_count} 笔，逐条对上今天的值）"
  echo "    没做成探针的 ${unprobed_count} 笔："
  while IFS=$'\t' read -r name why; do
    [[ -z "$name" ]] && continue
    printf '      %s：%s\n' "$name" "$why"
  done <<<"$UNPROBED"
  while IFS= read -r line; do
    [[ -n "$line" ]] && printf '    %s\n' "$line"
  done <<<"$row27_report"
  exit 77
}
