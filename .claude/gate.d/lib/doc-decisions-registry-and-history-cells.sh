#!/usr/bin/env bash
# 门禁 doc-decisions 的格函数库：freeze-layer-membership、decision-items-sync、blocking-verdict、user-verdict-owed、decision-summary-width、write-frequency、entry-added、shape、history-loss。
# 由 .claude/gate.d/doc-decisions.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释；读的 ROOT 与入口里设好的各变量、bad、ok、howto 在调用时才取。

# ════ 格「冻结层归属登记表」════
# .claude/kb/freeze-layer-membership.md 那张表（按整行的 <!-- freeze-layer-membership:table --> 定位，表头五列逐字），清单从哪来见那份文件「门禁判哪三条」：
#   ① 每层、每个独立冻结组件至少一行；每棵树在 key 编码层与索引节点内部布局层各至少一行（结构列写「树 ID <n>」）；每个单元类码至少一行（结构列写「码 <n>」）。
#      三份清单都现读、不抄：D15 已定项 7 的「第 N 层：」与「- ① **…**」（两层的号按「key 编码」「索引节点内部布局」认）、
#      crates/singlefs-format/src/lib.rs 的 TREE_IDENTIFIER_*（WATERMARK 两个是水位）、D18 已定项 11 登记表里首列是整数、类名不是「无效」「保留」的行。任一份读成 0 项判红。
#   ② 写「退出」的行态别必须是派生态。
#   ③ 依据列点名的「D<n>（简称） 已定项 k」要存在且已定（归属借 lib-item-ref-status.py）；依据以「判不动」开头的不判。
#   层、态别、退出三列只认封闭的几个值，写别的判红（一个错别字会让 ② 静默放行）。② 判不到的「判不动」行、态别判不动而退出已定的行，成功行逐个列出。
# 判不了：某一行的态别对不对（改成自洽的「派生态 + 退出」三条都不红）、判不动的理由、三份清单都没单列的结构（树表单元、实例表单元、容器索引）。
# 样本：freeze-layer-membership-red 漏一棵树的 key 编码行、漏组件 ② 与码 16 两行、一行「权威态 + 退出」、依据指到不存在的分项（一处接在「已定项 5 / 98」后面）、
#   未定项、不存在的决策（简称里套括号），判红；decision-documents-green 里是真表原样的一份（编号与分项号改成连号），判绿。
# D15、D18 与依据点名的决策取哪一份：lib-item-ref-status.py 的 numbered_files 与 heads_for_referrer，登记表是中文，取中文原文；
#   译文的首行读不读得出不归这一格（归格「分项引用状态」），中文原文的首行读不出记进「清单」判红。
# 样本：freeze-layer-membership-translation-green（两条决策各有读不出首行的译文，判绿）、-translation-red（依据点名中文原文没有的分项，判红）；
#   弄坏开关 KB_NUMBERED_FILE_BREAK=first-match。
grid_freeze_layer_membership() (
  LIB="$LIB_ITEM_REF_STATUS"
  REGISTRY=.claude/kb/freeze-layer-membership.md
  TREES=crates/singlefs-format/src/lib.rs
  SELF="$STAGE_RELATIVE_PATH --check freeze-layer-membership"
  MARKER='<!-- freeze-layer-membership:table -->'

  # D15、D18 取哪一份只走 lib-item-ref-status.py 的 numbered_files：登记表是中文，取中文原文。
  # compgen 会把三语三份一并交出来（多行），拼成一条不存在的路径，「#### 已定项 7」「#### 已定项 11」都去译文里找。
  local decision_files
  if ! decision_files="$(python3 - "$LIB" "$REGISTRY" 2>&1 <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("item_ref_status", sys.argv[1])
lib = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib)
for number in (15, 18):
    found = lib.numbered_files(".claude/kb/decisions", number, sys.argv[2])
    print(found[0] if found else "")
PY
  )"; then
    printf '%s\n' "$decision_files" | sed 's/^/      /'   # gate-lint:detail
    echo "  ✗ 按编号取 D15、D18 的那段 python 没跑成，这一格什么也没判"
    echo "    → 怎么办：按上面的 traceback 修（判法在 $LIB 的 numbered_files）；没跑成就是没判，不许当成判过了。"
    exit 1
  fi
  layers_decision="$(sed -n 1p <<<"$decision_files")"
  units_decision="$(sed -n 2p <<<"$decision_files")"
  if [[ -z "$layers_decision" ]]; then
    echo "  ⊘ 本次未跑：.claude/kb/decisions/ 下没有 D15，没有四层图就无对象可判"
    exit 77
  fi

  # 2>&1 要写在这一行：放在 heredoc 结束符后面单独一行是一条空命令，会把 python 的退出码盖成 0。
  report="$(python3 - "$LIB" "$REGISTRY" "$layers_decision" "$units_decision" "$TREES" 2>&1 <<'PY'
import importlib.util, os, re, sys

lib_path, registry_path, layers_path, units_path, trees_path = sys.argv[1:6]
MARKER = "<!-- freeze-layer-membership:table -->"
HEADER = "| 结构 | 落哪一层 | 态别 | 退不退出冻结 | 依据 |"
CIRCLED = "①②③④⑤⑥⑦⑧⑨⑩⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳"
STATES = ("权威态", "派生态", "判不动")
EXITS = ("退出", "不退出", "判不动")

def bad(kind, message):
    print("BAD", kind, message, sep="\t")

def section(path, heading_prefix):
    """从 `#### <heading_prefix>` 起到下一个任意级标题为止的正文；找不到返回 None。"""
    if not os.path.isfile(path):
        return None
    lines = open(path, encoding="utf-8").read().split("\n")
    start = next((index for index, line in enumerate(lines) if line.startswith(heading_prefix)), None)
    if start is None:
        return None
    body = []
    for line in lines[start + 1:]:
        if re.match(r"^#{1,4}\s", line):
            break
        body.append(line)
    return body

def strip_trailing_parenthetical(name):
    """去掉结构名末尾那一组配平的全角括号，只留给人认的那一段。"""
    if not name.endswith("）"):
        return name
    depth = 0
    for index in range(len(name) - 1, -1, -1):
        if name[index] == "）":
            depth += 1
        elif name[index] == "（":
            depth -= 1
            if depth == 0:
                return name[:index].strip() or name
    return name

# ── 清单一：D15 已定项 7 的层与组件 ────────────────────────
layers, components, key_layer, node_layer = [], [], None, None
body = section(layers_path, "#### 已定项 7")
if body is None:
    bad("清单", f"{layers_path} 里没有「#### 已定项 7」这一节，读不出四层图")
else:
    for line in body:
        hit = re.match(r"^第 (\d+) 层：(.*)$", line)
        if hit:
            number = int(hit.group(1))
            layers.append(number)
            if "key 编码" in hit.group(2):
                key_layer = number
            if "索引节点内部布局" in hit.group(2):
                node_layer = number
        hit = re.match(r"^- ([%s]) \*\*" % CIRCLED, line)
        if hit:
            components.append(hit.group(1))
    if not layers:
        bad("清单", f"{layers_path} 已定项 7 的代码块里一行「第 N 层：」都没读到")
    if not components:
        bad("清单", f"{layers_path} 已定项 7 里一行「- ① **…**」的独立冻结组件都没读到")
    if layers and key_layer is None:
        bad("清单", f"{layers_path} 已定项 7 的层图里找不到写着「key 编码」的那一层，一棵树该在哪层登记 key 编码认不出")
    if layers and node_layer is None:
        bad("清单", f"{layers_path} 已定项 7 的层图里找不到写着「索引节点内部布局」的那一层，一棵树该在哪层登记节点内部布局认不出")

# ── 清单二：lib.rs 的树 ID 常量 ────────────────────────
trees = []
if not os.path.isfile(trees_path):
    bad("清单", f"没有 {trees_path}，读不出树 ID 常量")
else:
    for line in open(trees_path, encoding="utf-8"):
        hit = re.match(r"^pub const TREE_IDENTIFIER_([A-Z_]+): u64 = (\d+);", line)
        if hit and not hit.group(1).startswith("WATERMARK"):
            trees.append((hit.group(1), int(hit.group(2))))
    if not trees:
        bad("清单", f"{trees_path} 里一个 TREE_IDENTIFIER_* 常量都没读到（水位常量不算树）")

# ── 清单三：D18 已定项 11 登记表的单元类码 ────────────────────────
unit_classes = {}
body = section(units_path, "#### 已定项 11") if units_path else None
if body is None:
    bad("清单", f"{units_path or '.claude/kb/decisions/18-*.md'} 里没有「#### 已定项 11」这一节，读不出单元类登记表")
else:
    # 表在第一个非 | 行（含空行）处结束：下一张表只隔一个空行时，不这样切会被并进来
    table, seen_table = [], False
    for line in body:
        if line.startswith("|"):
            table.append(line)
            seen_table = True
        elif seen_table:
            break
    if not table or table[0].split("|")[1].strip() != "码":
        bad("清单", f"{units_path} 已定项 11 之下第一张表的首列不是「码」，读不出单元类登记表")
    else:
        for row in table[2:]:
            cells = [cell.strip() for cell in row.strip().strip("|").split("|")]
            if len(cells) < 2 or not cells[0].isdecimal():
                continue
            name = cells[1].replace("*", "").strip()
            if "无效" in name or "保留" in name:
                continue
            unit_classes[int(cells[0])] = name
        if not unit_classes:
            bad("清单", f"{units_path} 已定项 11 登记表里一个登记了名字的单元类都没读到")

# ── 登记表本身 ────────────────────────
rows = []
if not os.path.isfile(registry_path):
    bad("表", f"没有 {registry_path}")
else:
    lines = open(registry_path, encoding="utf-8").read().split("\n")
    # 整行匹配，不用 in：正文里提到这个标记的句子不是标记行。
    marker_at = next((index for index, line in enumerate(lines) if line.strip() == MARKER), None)
    if marker_at is None:
        bad("表", f"{registry_path} 里没有单独成行的 {MARKER}")
    else:
        cursor = marker_at + 1
        while cursor < len(lines) and not lines[cursor].strip():
            cursor += 1
        table = []
        while cursor < len(lines) and lines[cursor].startswith("|"):
            table.append((cursor + 1, lines[cursor]))
            cursor += 1
        if not table:
            bad("表", f"{registry_path} 的标记行之后没有表")
        elif table[0][1].strip() != HEADER:
            bad("表", f"{registry_path}:{table[0][0]} 表头不是 {HEADER}，实际是 {table[0][1].strip()[:60]}")
        else:
            for line_number, row in table[2:]:
                cells = [cell.strip() for cell in re.split(r"(?<!\\)\|", row.strip().strip("|"))]
                if len(cells) != 5:
                    bad("表", f"{registry_path}:{line_number} 不是五格：{row.strip()[:60]}")
                    continue
                rows.append((line_number, cells))

allowed_layers = {f"第 {number} 层" for number in layers} | {f"组件 {mark}" for mark in components} | {"没有条款"}
for line_number, (structure, layer, state, exits, _basis) in rows:
    short = strip_trailing_parenthetical(structure)
    if layers and layer not in allowed_layers:
        bad("表", f"{registry_path}:{line_number}「{short}」的层写成「{layer}」，不在 D15 已定项 7 读出的层与组件里，也不是「没有条款」")
    if state not in STATES:
        bad("表", f"{registry_path}:{line_number}「{short}」的态别写成「{state}」，只认 权威态 / 派生态 / 判不动")
    if exits not in EXITS:
        bad("表", f"{registry_path}:{line_number}「{short}」的退不退出冻结写成「{exits}」，只认 退出 / 不退出 / 判不动")

# ── ① 每层每个结构都有行 ────────────────────────
def rows_with(predicate):
    return [(line_number, cells) for line_number, cells in rows if predicate(cells)]

for number in layers:
    if not rows_with(lambda cells: cells[1] == f"第 {number} 层"):
        bad("漏行", f"第 {number} 层没有一行")
for mark in components:
    if not rows_with(lambda cells: cells[1] == f"组件 {mark}"):
        bad("漏行", f"组件 {mark} 没有一行")
for name, identifier in trees:
    mentions = rows_with(lambda cells: re.search(rf"树 ID {identifier}(?!\d)", cells[0]) is not None)
    if key_layer is not None and not any(cells[1] == f"第 {key_layer} 层" for _, cells in mentions):
        bad("漏行", f"TREE_IDENTIFIER_{name}（树 ID {identifier}）在第 {key_layer} 层（key 编码）没有一行")
    if node_layer is not None and not any(cells[1] == f"第 {node_layer} 层" for _, cells in mentions):
        bad("漏行", f"TREE_IDENTIFIER_{name}（树 ID {identifier}）在第 {node_layer} 层（索引节点内部布局）没有一行")
for code, name in sorted(unit_classes.items()):
    if not rows_with(lambda cells: code in {int(found) for found in re.findall(r"码 (\d+)", cells[0])}):
        bad("漏行", f"码 {code}（{name}）没有一行")

# ── ② 写「退出」的行必须是派生态 ────────────────────────
for line_number, (structure, _layer, state, exits, _basis) in rows:
    if exits == "退出" and state != "派生态":
        bad("退出", f"{registry_path}:{line_number}「{strip_trailing_parenthetical(structure)}」写着退出冻结，态别却是「{state}」")

# ── ③ 依据列点名的分项在且已定 ────────────────────────
spec = importlib.util.spec_from_file_location("item_ref_status", lib_path)
item_ref_status = importlib.util.module_from_spec(spec)
spec.loader.exec_module(item_ref_status)
# 依据列点名的决策号对照中文原文（登记表是中文；heads_for_referrer 只留中文那几份）。load_map() 现读全部首行时，
# 首行写成 `## D15 (Format Freeze Policy) — Settled` 的译文读不出就当场退 1，这一格报不出 COUNT。
# 译文的首行读没读得出归格「分项引用状态」判；中文原文的首行读不出照旧判红，记进「清单」。
heads, unreadable = item_ref_status.decision_heads(strict=False)
heads, unreadable = item_ref_status.heads_for_referrer(heads, unreadable, registry_path)
for unreadable_path, first_line in unreadable:
    bad("清单", f"{unreadable_path} 首行读不出 `## D<n> 简称 —— 状态`（首行是 {first_line[:40]!r}），这一份的分项一条都核不了")
item_map, _names = item_ref_status.load_map(heads)
basis_checked = 0
for line_number, (structure, _layer, _state, _exits, basis) in rows:
    short = strip_trailing_parenthetical(structure)
    # 简称里可以再套一层全角括号（D14（双轨（大小文件 / 持久临时））），一处引用可以接几个项号（「已定项 3 / 4」）：都要认全，否则无声漏判
    references = []
    for hit in re.finditer(r"(D\d+)（(?:[^（）]|（[^（）]*）)*）\s*(已定项|未定项)\s*(\d+)((?:\s*[/、]\s*(?:(?:已定项|未定项)\s*)?\d+)*)", basis):
        decision, written, first, tail = hit.groups()
        references.append((decision, written, first))
        for kind, number in re.findall(r"[/、]\s*(?:(已定项|未定项)\s*)?(\d+)", tail):
            references.append((decision, kind or written, number))
    if not references:
        if not basis.startswith("判不动"):
            bad("依据", f"{registry_path}:{line_number}「{short}」的依据没点名任何「D<n>（简称） 已定项 k」，也不是「判不动」：{basis[:40]}")
        continue
    for decision, written, item in references:
        basis_checked += 1
        item = int(item)
        if decision not in item_map:
            bad("依据", f"{registry_path}:{line_number}「{short}」的依据点名 {decision}，而 .claude/kb/decisions/ 里没有这条决策")
        elif item not in item_map[decision]:
            bad("依据", f"{registry_path}:{line_number}「{short}」的依据点名 {decision} 第 {item} 条分项，而那条决策的索引表里没有第 {item} 条")
        elif item_map[decision][item] != "已":
            bad("依据", f"{registry_path}:{line_number}「{short}」的依据点名 {decision} 第 {item} 条分项，而它在索引表里是未定项——依据只能是已定项")
        elif written != "已定项":
            bad("依据", f"{registry_path}:{line_number}「{short}」的依据把 {decision} 第 {item} 条写成「{written}」，而它是已定项")

# ── 没判的：② 判不到的行，与态别没判出来而冻结归属已定的行 ────────────────────────
exit_undetermined = [strip_trailing_parenthetical(cells[0]) for _, cells in rows if cells[3] == "判不动"]
state_only_undetermined = [f"{strip_trailing_parenthetical(cells[0])}（{cells[3]}）" for _, cells in rows
                           if cells[2] == "判不动" and cells[3] != "判不动"]
for name in exit_undetermined:
    print("SKIP", name, sep="\t")
for name in state_only_undetermined:
    print("STATEONLY", name, sep="\t")
print("COUNT", len(rows), len(layers), len(components), len(trees), len(unit_classes), basis_checked,
      len(exit_undetermined), len(state_only_undetermined), sep="\t")
PY
  )"
  python_rc=$?

  if ((python_rc != 0)) || ! grep -q '^COUNT' <<<"$report"; then
    echo "  ✗ 扫描没跑完：内嵌 python 没报出 COUNT 那一行（退出码 ${python_rc}，多半是它自己崩了）"
    grep -v '^\(BAD\|SKIP\|COUNT\)' <<<"$report" | sed 's/^/      /'   # gate-lint:detail
    echo "    → 怎么办：直接跑 bash ${SELF} 看 python 的报错；⚠️ 别把这一步当通过——"
    echo "      没有 COUNT 就是一行都没判，而汇总行看着与判过了一模一样。"
    exit 1
  fi

  failed=0
  for kind in 清单 表 漏行 退出 依据; do
    mapfile -t hits < <(awk -F'\t' -v kind="$kind" '$1 == "BAD" && $2 == kind { print $3 }' <<<"$report")
    ((${#hits[@]})) || continue
    failed=1
    case "$kind" in
      清单)
        echo "  ✗ 判据要读的清单有 ${#hits[@]} 份读不出来，这一格什么也没在判："   # gate-lint:summary
        printf '      %s\n' "${hits[@]}"   # gate-lint:detail
        echo "    → 怎么办：三份清单是 D15 已定项 7 正文（代码块里「第 N 层：」与列表里「- ① **…**」）、"
        echo "      crates/singlefs-format/src/lib.rs 的 TREE_IDENTIFIER_* 常量、D18 已定项 11 之下首列是「码」的登记表。"
        echo "      哪一份的形态改了，就改这一格去跟上它，别在脚本里抄一份清单——抄了就与条文分叉。"
        ;;
      表)
        echo "  ✗ 登记表 ${REGISTRY} 有 ${#hits[@]} 处不合形态："   # gate-lint:summary
        printf '      %s\n' "${hits[@]}"   # gate-lint:detail
        echo "    → 怎么办：表前单独一行 ${MARKER}，表头逐字"
        echo "      | 结构 | 落哪一层 | 态别 | 退不退出冻结 | 依据 |；层写「第 N 层」「组件 ①」或「没有条款」，"
        echo "      态别写 权威态 / 派生态 / 判不动，退出列写 退出 / 不退出 / 判不动。形态见 .claude/kb/freeze-layer-membership.md 开头几段。"
        ;;
      漏行)
        echo "  ✗ ① 有 ${#hits[@]} 个层或结构在登记表里没有行："   # gate-lint:summary
        printf '      %s\n' "${hits[@]}"   # gate-lint:detail
        echo "    → 怎么办：给每一个补一行（结构 / 落哪一层 / 态别 / 退不退出冻结 / 依据），态别按 D21 已定项 10 的判据推，"
        echo "      推不出就写「判不动」并在表下「判不动的十一行，各卡在哪一句」里写清卡在哪一句；"
        echo "      树的一行结构列要写「树 ID <n>」，单元类的一行要写「码 <n>」，这一格按这两个字样认行。"
        ;;
      退出)
        echo "  ✗ ② 有 ${#hits[@]} 行写着退出冻结而态别不是派生态："   # gate-lint:summary
        printf '      %s\n' "${hits[@]}"   # gate-lint:detail
        echo "    → 怎么办：D21 已定项 9 那张表说权威态「参与格式冻结，是永久契约」、派生态「不参与格式冻结」；"
        echo "      两列只能一起改：要么把态别改回派生态（依据要跟着指到判它派生态的那条分项），要么把退出列改成「不退出」。"
        ;;
      依据)
        echo "  ✗ ③ 有 ${#hits[@]} 处依据指不到一条已定分项："   # gate-lint:summary
        printf '      %s\n' "${hits[@]}"   # gate-lint:detail
        echo "    → 怎么办：依据写成「D<n>（简称） 已定项 k」，那条决策要在 .claude/kb/decisions/ 里、索引表里第 k 条要是已定；"
        echo "      分项还没定就不能拿它当依据——这一行的态别改成「判不动」，依据写「判不动，推导见「判不动的十一行，各卡在哪一句」」。"
        ;;
    esac
  done
  ((failed)) && exit 1

  read -r _ row_count layer_count component_count tree_count unit_count basis_count skipped_count state_only_count < <(grep '^COUNT' <<<"$report")
  mapfile -t skipped < <(awk -F'\t' '$1 == "SKIP" { print $2 }' <<<"$report")
  mapfile -t state_only < <(awk -F'\t' '$1 == "STATEONLY" { print $2 }' <<<"$report")
  echo "  ✓ 冻结层归属登记表判过了（$((row_count)) 行：D15 已定项 7 的 ${layer_count} 层与 ${component_count} 个组件、lib.rs 的 ${tree_count} 棵树、D18 已定项 11 的 ${unit_count} 个单元类都有行；依据点名的 ${basis_count} 处分项都在且已定）"
  if ((skipped_count)); then
    echo "    ② 没判的 ${skipped_count} 行（退不退出冻结写「判不动」）：$(IFS='；'; echo "${skipped[*]}")"
  else
    echo "    ② 每一行都判了，没有退不退出冻结写「判不动」的行"
  fi
  if ((state_only_count)); then
    echo "    另有 ${state_only_count} 行态别写「判不动」而退不退出冻结已定（括号里是退出列）：$(IFS='；'; echo "${state_only[*]}")"
  fi
  exit 0
)

# ════ 格「决策分项清单与正文同步」════
# decisions.md 的分项清单生成块与索引表「状态」「进度」两列是各决策正文的投影，与生成器 .claude/scripts/gen-decision-items.py 的输出逐字比。
# 状态列写分项计数（`已定 6 项 / 未定 0 项`，没有分项写「无分项 · 整条已定」）；三态词的权威在正文首行。
# 进度列（最后一列）写实现计数（`已实现 3 项 / 未实现 1 项`，有无对象的再带 ` / 无对象 2 项`；没有分项写「无对象」），取自各分项索引行末尾的 **进度：…。** 标记。
# 计数本身由格「kb 形状」第 7 段（INDEX_VS_BODY_SOURCE）不经生成器另数：只靠这一格时，生成器数错索引页会跟着错而仍逐字相同。
# --write 时把两块写回被判的仓的 decisions.md（用法在文件头）。
# 样本：decision-items-sync-red 状态列写错、索引漏登、索引多一行、生成块漂了各一处，判红；
#       decision-items-sync-translation-green 的 decisions/ 下放着译文，生成器只读中文正文，判绿（译文当正文读时生成器报首行读不出，这一格红）。
grid_decision_items_sync() (
  GEN="$GENERATOR"
  WRITE="$DECISION_ITEMS_WRITE"
  IDX=.claude/kb/decisions.md
  S='<!-- gen:decision-items:start -->'
  E='<!-- gen:decision-items:end -->'
  [[ -f "$GEN" ]] || { echo "  ✗ 找不到生成器 $GEN"; echo "     → 生成器随门禁住在同一个仓的 .claude/scripts/ 下：它丢了这一格什么都判不了，从 git 里找回它"; exit 1; }
  [[ -f "$IDX" ]] || { echo "  ! 找不到 $IDX，这一格无对象可判"; exit 77; }
  grep -qF "$S" "$IDX" || { echo "  ✗ $IDX 里没有生成块标记 $S"; echo "     → 加回标记，或跑 --write 重建"; exit 1; }

  gen_err="$(mktemp)"
  cells_f="$(mktemp)"
  # 各分支手写的 rm 管不到半路被打断的那一次：退出时一律删
  trap 'rm -f "${gen_err:?}" "${cells_f:?}"' EXIT
  # 生成器的 stderr 里有它自己的出路，不吞
  want="$(python3 "$GEN" "$ROOT" 2>"$gen_err")" || {
    echo "  ✗ 生成器跑不起来"
    sed 's/^/   /' "$gen_err"
    echo "     → 怎么办：按上面这段报错改 $GEN 或它读的那几份 kb；"
    echo "               生成器跑不起来时这一格什么都没查，不是通过。"
    rm -f "$gen_err" "$cells_f"; exit 1; }
  python3 "$GEN" "$ROOT" --status-cells >"$cells_f" 2>"$gen_err" || {
    echo "  ✗ 生成器的 --status-cells 跑不起来"
    sed 's/^/   /' "$gen_err"
    echo "     → 怎么办：同上——状态列没算出来时，这一半什么都没查，不是通过。"
    rm -f "$gen_err" "$cells_f"; exit 1; }
  rm -f "$gen_err"
  got="$(awk -v s="$S" -v e="$E" 'index($0,s){f=1;next} index($0,e){f=0} f' "$IDX")"

  fail=0

  if [[ "$WRITE" == "1" ]]; then
    python3 - "$IDX" "$S" "$E" "$GEN" "$ROOT" <<'PY'
import sys, subprocess
idx, s, e, gen, root = sys.argv[1:6]
body = open(idx, encoding='utf-8').read()
new = subprocess.run(['python3', gen, root], capture_output=True, text=True).stdout.rstrip()
a = body.index(s) + len(s)
b = body.index(e)
open(idx, 'w', encoding='utf-8').write(body[:a] + "\n" + new + "\n" + body[b:])
PY
  fi

  # ── 索引表「状态」列：只改第 2 列，整行重写会抹掉别的会话刚写进结论列的东西 ──
  python3 - "$IDX" "$([[ $WRITE == 1 ]] && echo write || echo check)" "$cells_f" <<'PY'
import re, sys

idx_path, mode, cells_path = sys.argv[1:4]
text = open(idx_path, encoding='utf-8').read()
want = {}
for l in open(cells_path, encoding='utf-8').read().splitlines():
    if l.strip():
        num, status_cell, progress_cell = l.split('\t')
        want[num] = (status_cell, progress_cell)

bad, n, wrote = [], 0, 0
for num, (cell, progress) in want.items():
    pat = re.compile(r'^(\|\s*%s（[^|]*）\s*\|)([^|]*)\|' % re.escape(num), re.M)
    m = pat.search(text)
    if not m:
        bad.append(f"{num} 有正文，而决策索引表里没有它的行")
        continue
    n += 1
    if m.group(2).strip() != cell:
        if mode == 'write':
            text = text[:m.start(2)] + f" {cell} " + text[m.end(2):]
            wrote += 1
        else:
            bad.append(f"{num} 状态列写着「{m.group(2).strip()}」，正文投影是「{cell}」")
    # 进度列是最后一列：只改行尾那一格，前面几格一个字不碰（结论列里可能有转义竖线）。
    line_m = re.search(r'^\|\s*%s（[^|]*）.*$' % re.escape(num), text, re.M)
    cells = [c.strip() for c in re.split(r'(?<!\\)\|', line_m.group(0).strip().strip('|'))]
    if len(cells) < 5:
        bad.append(f"{num} 索引行只有 {len(cells)} 列，缺「进度」列（第 5 列，写成 `| 决策 | 状态 | 结论（简报） | 正文 | 进度 |`）")
        continue
    if cells[4] == progress:
        continue
    tail = re.search(r'\|([^|]*)\|\s*$', line_m.group(0))
    if mode == 'write':
        a, b = line_m.start() + tail.start(1), line_m.start() + tail.end(1)
        text = text[:a] + f" {progress} " + text[b:]
        wrote += 1
    else:
        bad.append(f"{num} 进度列写着「{cells[4]}」，正文投影是「{progress}」")

# 反过来查索引表多出的行（正文删了或改名了），上面的循环够不着
for row in re.findall(r'^\|\s*(D\d+)（', text, re.M):
    if row not in want:
        bad.append(f"{row} 在决策索引表里有行，而 decisions/ 下没有它的正文")

if mode == 'write':
    open(idx_path, 'w', encoding='utf-8').write(text)

if bad:
    print(f"  ✗ 决策索引表的状态列或进度列与正文不同步（{len(bad)} 处）")   # gate-lint:summary
    for b in bad:
        print("     " + b)                                          # gate-lint:detail
    print("     → 怎么办：权威记录是各决策正文的「### 已定项」/「### 未定项」两节。")
    print("       改完正文跑： SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync")
    sys.exit(1)

print(f"  ✓ 决策索引表状态列与进度列与正文同步（{n} 条决策"
      + (f"，写回 {wrote} 行）" if mode == 'write' else "）"))
PY
  [[ $? -ne 0 ]] && fail=1
  rm -f "$cells_f"

  if [[ "$WRITE" == "1" ]]; then
    echo "  ✓ 已重新生成并写回 $IDX"
    exit $fail
  fi

  if [[ "$want" == "$got" ]]; then
    n=$(printf '%s\n' "$want" | grep -c '^  - ' || true)
    decision_lines=$(printf '%s\n' "$want" | grep -c '^- ' || true)
    # 生成器一条决策都没产出、生成块也空：两个空串相等不算比过
    if [[ "$decision_lines" -eq 0 && "$fail" -eq 0 ]]; then
      echo "  ! 本次无对象可判：生成器一条决策都没产出（decisions/ 下没有读得出的决策正文），生成块也是空的"
      exit 77
    fi
    echo "  ✓ 决策分项清单与正文同步（$n 个分项）"
    exit $fail
  fi
  echo "  ✗ 决策分项清单与正文不同步"
  diff <(printf '%s\n' "$got") <(printf '%s\n' "$want") | head -20 | sed 's/^/     /'
  echo "     → 权威记录是各决策正文。改完正文跑： SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync"
  exit 1
)

# ════ 格「未定项判过改不改新池新建文件的字节」════
# 每条未定项（清单取自生成器 gen-decision-items.py，与格「决策分项清单与正文同步」同一个解析）的登记行或列表条目里要有两把尺的判定，缺就红：
#   改新池新建文件的字节：是 / 否 / 无对象（YYYY-MM-DD，依据：…）
#   动不动格式：动 / 不动 / 界不定（YYYY-MM-DD，依据：…）
# 只认这一种写法，散文措辞分不开「判过否」与「随口提了一句阻塞」；日期不许省，判定会因别处的决策变了而过期。
# 第一把尺问：不定它，新池新建文件写出的字节（mkfs 写的也算）能不能由已定条款唯一确定？能为否，不能为是。
# 刻意不叫「阻塞」：那个词同时罩着「不改字节」与「改、但接受先写后改」，挡不挡开工是处置，写在依据里。
# 与格「未定项被别处定了」「定了新东西没回看同文件未定项」的分工：那两格管判定会不会过期，这一格管判定在不在。
# 登记行由这一格自己定位，所以另比条数：定位到的与生成器认的未定项对不上（漏了，或多了：例如表格行缺收尾竖线生成器不收）判红。
# 只判判定在不在，判得对不对（依赖对）还欠着：C50（阻塞标记没人维护）。
# 样本：blocking-verdict-red 两把尺各缺、外加一行表格缺收尾竖线（多定位出），判红。
grid_blocking_verdict() (
  DEC=.claude/kb/decisions
  # 生成器按本脚本所在的仓取（GENERATOR），不按 cwd：样本把 cwd 换成只放着样本 kb 的临时目录，按 cwd 取会找不到生成器而跳过，红样本悄悄变绿。
  # 生成器读显式传给它的项目根（$ROOT，喂样本时是样本根）下的 kb，不从它自己的位置定根。
  GEN="$GENERATOR"
  [[ -d "$DEC" ]] || { echo "  ! 没有 $DEC，这一格无对象可判"; exit 77; }
  # 生成器随仓走，不在是被删了或挪了：退 1 不退 77，与格「决策分项清单与正文同步」一致
  [[ -f "$GEN" ]] || { echo "  ✗ 找不到生成器 $GEN"; echo "     → 生成器随门禁住在同一个仓的 .claude/scripts/ 下：它丢了这一格什么都判不了，从 git 里找回它"; exit 1; }

  python3 - "$DEC" "$GEN" "$ROOT" <<'PY'
import re, sys, glob, subprocess, os

dec, gen, root = sys.argv[1], sys.argv[2], sys.argv[3]

# ── 权威清单：哪些 (决策号, 分项号) 是未定的 ────────────────────
r = subprocess.run(['python3', gen, root], capture_output=True, text=True)
if r.returncode != 0:
    print("  ✗ 生成器跑不起来，拿不到未定项清单")
    print('\n'.join('   ' + l for l in r.stderr.splitlines()))
    print("     → 怎么办：按上面这段报错修生成器或它读的那几份 kb。")
    print("               它跑不起来时这一格一条未定项都没查过，不是通过。")
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

# 两把尺在同一批登记行上各判各的，不拿一把的结论填另一把：一条未定项能前一把判「否」而后一把判「动」
RULERS = [
    ("改新池新建文件的字节",
     re.compile(r'改新池新建文件的字节：\*{0,2}(是|否|无对象)\*{0,2}[（(](\d{4}-\d{2}-\d{2})'),
     "判过改不改新池新建文件的字节"),
    ("动不动格式",
     re.compile(r'动不动格式：\*{0,2}(动|不动|界不定)\*{0,2}[（(](\d{4}-\d{2}-\d{2})'),
     "写了「动不动格式」的判定"),
]

seen, located, bad = set(), set(), {}
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
    # 只取第一个 `####` 之前那一段，与生成器同口径
    cut = re.search(r'^#{4}\s', sec, re.M)
    top = sec[:cut.start()] if cut else sec
    # 每条分项的块从它的登记行到下一条登记行为止（表格形态就是一行）
    starts = [(mo.start(), mo.group(1))
              for mo in re.finditer(r'^(?:\|\s*(\d+)\s*\||(?=\d))(\d+)?\.?\s', top, re.M)]
    marks = [(mo.start(), mo.group(1) or mo.group(2))
             for mo in re.finditer(r'^\|\s*(\d+)\s*\||^(\d+)\.\s', top, re.M)]
    for i, (pos, num) in enumerate(marks):
        end = marks[i + 1][0] if i + 1 < len(marks) else len(top)
        block = top[pos:end]
        located.add((dnum, num))
        if (dnum, num) not in want:
            continue          # 生成器不认它是未定项：解析漂了，下面按「多定位出」报
        seen.add((dnum, num))
        for name, pattern, _ in RULERS:
            if not pattern.search(block):
                first = block.strip().splitlines()[0]
                bad.setdefault(name, []).append((os.path.basename(f), dnum, num, first[:70]))

missed = want - seen
# seen 只收 want 里的，多出来的要拿定位到的去比
extra = located - want
if missed or extra:
    print("  ✗ 登记行定位与生成器对不上，这一格这一轮查的不是全部未定项")
    for d, n in sorted(missed):
        print(f"     定位不到：{d} 未定项 {n}")
    for d, n in sorted(extra):
        print(f"     多定位出：{d} 未定项 {n}（生成器不认它是未定项）")
    print("     → 「### 未定项」一节要有索引表（`| # | 分项 | 状态 |`，每行要有收尾的竖线）或顶格编号列表，每条分项一行")

if bad:
    total = sum(len(v) for v in bad.values())
    print(f"  ✗ {total} 条未定项缺判定（两把尺分开数）：")          # gate-lint:summary
    for name, rows in bad.items():
        print(f"     ── 缺「{name}」：{len(rows)} 条")               # gate-lint:detail
        for fn, d, n, first in rows:
            print(f"        {fn}  {d} 未定项 {n}：{first}")          # gate-lint:detail
    if "改新池新建文件的字节" in bad:
        print("     → 缺「改新池新建文件的字节」怎么办：按「改不改变新池新建文件写出的字节」这把尺判一次，")
        print("               在该分项的登记行里写：改新池新建文件的字节：否（YYYY-MM-DD，依据：…）")
        print("               三个合法取值：是 / 否 / 无对象。**两侧要用同一把尺**——")
        print("               判「不阻塞」用这把尺、判「阻塞」换一把，量出来的集合不是包含关系")
        print("               （decisions-history.md 的 D5（快照 / 空间记账机制） 节 2026-08-29 那一条实测）。")
    if "动不动格式" in bad:
        print("     → 缺「动不动格式」怎么办：按 D15（格式冻结政策） 已定项 1 的尺判一次——它的答案")
        print("               会不会改动任何盘上字节，或改动已有字节的解释口径（宽度不变而含义变了）；")
        print("               在该分项的登记行里写：动不动格式：动（YYYY-MM-DD，依据：…）")
        print("               三个合法取值：动 / 不动 / 界不定（视同动）。⚠️ 别拿上面那把尺的结论来填：")
        print("               那把量「这一版写不写出不同字节」，这把量「将来动不动格式」，")
        print("               两把量的不是一个集合。")
if bad or missed or extra:
    sys.exit(1)

if not seen:
    print("  ! 没有一条未定项，这一格无对象可判")
    sys.exit(77)
print(f"  ✓ {len(seen)} 条未定项两把尺都判过："
      + "、".join(f"{name}" for name, _, _ in RULERS))
PY
)

# ════ 格 user-verdict-owed：待用户复核的决策记了未还的账 ════
# 决策正文里标着「待用户复核 / 等用户复核」的，checks-owed.md 里要有一笔未还的账在同一行点名那条决策（.claude/singlefs-ai-sop/skills/decide/SKILL.md 硬要求第 6 条）。
# 粒度是决策：一份正文标了几处，有一笔未还的账点名这条决策就算有。
# 「未还」看那一行有没有带日期的还清标记（`已还（20…`、`已还一半（…`），不按「已还」两字判：描述里出现这两个字会被误判成还清。已还的账不再被回看，要盯就单立一笔。
# 样本：user-verdict-owed-red 里 D316 只有一笔已还的账、D31 只有一笔点名 D319 的未还账，判红。
grid_user_verdict_owed() (
  KB="$PWD/.claude/kb"
  OWED="$KB/checks-owed.md"
  MARK='待用户复核\|等用户复核'

  [[ -d "$KB/decisions" ]] || { echo "  ! 找不到 $KB/decisions，这一格跳过"; exit 77; }
  [[ -f "$OWED" ]] || { echo "  ✗ 缺 $OWED"
    echo "     → 怎么办： 待复核的条款要有账本盯着。先建 checks-owed.md，再把这一笔记进去。"; exit 1; }

  fail=0; checked=0; marked_lines=0
  while IFS= read -r f; do
    grep -q "$MARK" "$f" || continue
    marked_lines=$((marked_lines + $(grep -c "$MARK" "$f")))
    # 决策号取自文件名：`16-发布语义.md` → D16
    d="D$(basename "$f" | sed 's/^0*//; s/-.*//')"
    checked=$((checked+1))
    # 先落到变量再判，末段不接 `grep -q`：pipefail 下命中会被读成没命中（shell-lint S7）。决策号按整个号认：D1 不命中 D10–D19 的账
    hits=$(grep "$MARK" "$OWED" | grep -E "${d}([^0-9]|\$)" || true)
    if [[ -n "$hits" ]] && grep -qv "已还[^ |]*（20" <<<"$hits"; then continue; fi
    echo "  ✗ $d 的正文标着待用户复核，checks-owed.md 里却没有一笔**未还**的账点名它"
    echo "        正文：$(basename "$f")"
    echo "     → 怎么办： 在 checks-owed.md 立一笔账，点名是 $d 的哪一项、原定案是哪天由谁拍的、"
    echo "                复核要回答什么问题；复核过了再标已还。"
    echo "                只在正文写一句「待用户复核」，没有任何东西保证它会被看到"
    echo "                （decide skill 硬要求第 6 条）。"
    fail=$((fail+1))
  done < <(find "$KB/decisions" -name '*.md' | sort)

  if [[ $fail -gt 0 ]]; then
    echo "  ✗ $fail 份决策正文标着待用户复核、却没有未还的账（共查 $checked 份）"
    echo "     → 怎么办：每一处「待用户复核」都要在 kb/checks-owed.md 里有一条未还的账盯着，"
    echo "               否则那句「待复核」没有任何人会回头看。补上那条账，或者"
    echo "               这一条已经复核完了就把「待用户复核」改成结论。"
    exit 1
  fi
  if [[ $checked -eq 0 ]]; then
    echo "  ! 决策正文里没有一处标着待用户复核，这一格无对象可判"
    exit 77
  fi
  echo "  ✓ 待用户复核的条款都有未还的账盯着（$checked 份决策正文、$marked_lines 行标着待用户复核；按决策判，一份决策有一笔未还的账点名它就算有）"
)

# ════ 格「决策索引结论列的宽度」════
# 决策索引表每一行的「结论」列不超过 decisions.md 里「「结论」列一律 N 字以内」那一句的上限；上限从那一句读、不写死（只许一处权威）。
# 样本：decision-summary-width-red 两行超限（一行结论里带转义竖线），判红。
grid_decision_summary_width() (
  IDX=.claude/kb/decisions.md
  [[ -f "$IDX" ]] || { echo "  ! 找不到 $IDX，这一格跳过"; exit 77; }

  python3 - "$IDX" <<'PY'
import re, sys

path = sys.argv[1]
text = open(path, encoding="utf-8").read()

m = re.search(r"「结论」列一律\s*(\d+)\s*字以内", text)
if not m:
    print("  ✗ decisions.md 的索引表说明里找不到「「结论」列一律 N 字以内」")   # gate-lint:summary
    print("  → 怎么办：上限的权威记录是那一句，这一格从它读。把它写回索引表的说明里，")
    print("    例：⚠️ **「结论」列一律 200 字以内**，写那条决策定了什么。")
    sys.exit(1)
limit = int(m.group(1))

rows, bad = [], []
for line in text.splitlines():
    if not re.match(r"^\|\s*D\d+（", line):
        continue
    # 按没转义的竖线切格：结论里的 `\|` 是格子里的字（与格「冻结层归属登记表」同一种切法）
    cells = [c.strip() for c in re.split(r"(?<!\\)\|", line.strip().strip("|"))]
    if len(cells) < 4:
        continue
    name = cells[0]
    concl = re.sub(r"\s+", " ", cells[2])
    rows.append(name)
    if not concl:
        bad.append(f"{name} 的结论列是空的")
    elif len(concl) > limit:
        bad.append(f"{name} 的结论列 {len(concl)} 字，超出上限 {limit} 字：{concl[:36]}…")

if not rows:
    print("  ✗ decisions.md 里一行 `| D<n>（简称） | 状态 | 结论 | 正文 |` 都没扫到")   # gate-lint:summary
    print("  → 怎么办：索引表的行首形态是 `| D1（数据可移动性 / 反向索引） | …`。")
    print("    改过表格形状就同步改这一格的匹配，别让它扫到 0 行还报绿。")
    sys.exit(1)

if bad:
    print(f"  ✗ 决策索引结论列有 {len(bad)} 行超出上限（限 {limit} 字）：")   # gate-lint:summary
    for b in bad:
        print("      " + b)                                                  # gate-lint:detail
    print("  → 怎么办：结论列只写那条决策**定了什么**，为什么定、还欠什么一律搬进 decisions/ 下的正文。")
    print("    删不动就说明这条决策的主结论本身没收敛——那要么改它的状态，要么把分项拆开写。")
    sys.exit(1)

print(f"  ✓ 决策索引结论列都不超 {limit} 字（共 {len(rows)} 行）")
PY
)

# ════ 格「同一结构的写频率只有一个量级」════
# 决策正文里落在登记结构上的写频率断言，不许与 .claude/gate.d/write-frequency.tsv 登记的量级说反话；.claude/gate.d/write-frequency-exclude.tsv 逐句登记该留的判红句。两张表的写法与量级、量的口径在各自的文件头。
# 扫 .claude/kb/decisions/*.md「## 历史版本」之前、围栏之外的行：去掉 ** 与反引号，按没转义的竖线切格、按「。；！？」切句。
# 断言：频率词后面同一小句里 12 字以内跟写动词（「写放大」不算），或紧跟「一次」；只有频率词、不跟写动词的不判。
# 归属：句里的每个登记叫法都算；句里没有就认同一表格行别的格，还没有就认所属最近一个标题。
# 三种形态：
#   否定（频率词前隔至多三个虚字的否定词，或频率词与写动词之间的「不 / 没 / 无需 / 无须」）：否定别的量级是划界、只计数；否定的时机被登记的罩住，判红。
#   限定（前面紧挨「只 / 仅 / 只有」，可带「在 / 于」）：写的量与登记的量不同就判红，同级不同量也红；「不只 / 不仅 / 不是只」不划界，按否定计数。
#   普通：不同级判红；登记的只是量级里一部分时机的，还要同量。
# 另判红：登记表写坏；权威分项不是已定的「D<n>（简称） 已定项 k」、决策不在或简称与首行不符（归属借 lib-item-ref-status.py）；
#   排除表一行指不存在的文件、那一段不足 8 字或一句判红句都没排到；登记了结构而一处落在它们身上的断言都没扫到。
# 看不见：词表外的频率说法与叫法、跨句指代、否定词与频率词之间隔着虚字表外的字、双重否定、叫法取自表格行或标题时说的是不是这个结构、
#   是不是稳态写频率。成功行逐个列出一处断言都没扫到的结构。登记表不在退 77。
# 样本：write-frequency-red 里不同级、否定同级、隔字与动词前否定三句、「只在 fsync 时写」、主语只在标题里的一句、权威是未定项、简称写错、
#   排除表一行指不存在的文件、一行没排到判红句，判红；write-frequency-green 里同级不同词、限定与登记同量、否定别的量级、不划界的「不仅 / 不是只」、
#   主语在别的格或只在标题、排除表登记的一句，判绿。
# 权威分项点名的决策取哪一份：lib-item-ref-status.py 的 heads_for_referrer，登记表是中文，只对照中文原文；译文的首行归格「分项引用状态」判。
# 样本：write-frequency-translation-green（D1 有两份读不出首行的译文，判绿）；弄坏开关 KB_NUMBERED_FILE_BREAK=first-match。
grid_write_frequency() (
  REGISTRY=.claude/gate.d/write-frequency.tsv
  EXCLUDE=.claude/gate.d/write-frequency-exclude.tsv
  [[ -d .claude/kb/decisions ]] || { echo "  ! 找不到 .claude/kb/decisions，这一格跳过"; exit 77; }
  [[ -f "$REGISTRY" ]] || { echo "  ! 找不到 $REGISTRY（写频率登记表），这一格无对象可判"; exit 77; }

  python3 - "$REGISTRY" "$EXCLUDE" "$LIB_ITEM_REF_STATUS" "$BREAK_SWITCH" <<'PY'
import glob, importlib.util, os, re, sys

registry_path, exclude_path, lib_path, break_switch = sys.argv[1:5]
levels_merged = break_switch == "write-frequency-levels-merged"                        # 弄坏开关：不同级一律判成同级
negation_counted = break_switch == "write-frequency-negation-asserts"                  # 弄坏开关：否定形态也算断言
subject_in_sentence_only = break_switch == "write-frequency-subject-in-sentence-only"  # 弄坏开关：叫法只从句内认
negation_adjacent_only = break_switch == "write-frequency-negation-adjacent-only"      # 弄坏开关：否定词只认紧挨频率词的
restriction_as_plain = break_switch == "write-frequency-restriction-as-plain"          # 弄坏开关：「只在 / 仅」限定形态当普通断言

problems = []   # (类别, 明细)


def bad(kind, detail):
    problems.append((kind, detail))


def phrase_pattern(phrase):
    # 登记里的空格是中英文之间可有可无的空白：「每次 fsync」也认「每次fsync」
    return r"\s*".join(re.escape(piece) for piece in phrase.split(" ") if piece)


# ── 读登记表 ────────────────────────
frequency_level = {}      # 频率词 → 量级
frequency_quantity = {}   # 频率词 → 量（频率词行没写量的，量就是量级本身）
structures = []           # (行号, 结构名, [叫法], 登记频率词, 权威分项)
with open(registry_path, encoding="utf-8") as handle:
    for line_number, raw in enumerate(handle, 1):
        line = raw.rstrip("\n")
        if not line.strip() or line.startswith("#"):
            continue
        cells = line.split("\t")
        reason = cells[-1] if cells[-1].startswith("#") else ""
        if len(reason.lstrip("#").strip()) < 4:
            bad("登记表", f"{registry_path}:{line_number} 末列不是「# 理由」或理由不足 4 个字：{line[:60]}")
            continue
        if cells[0] == "频率词" and len(cells) in (4, 5):
            word, level = cells[1].strip(), cells[2].strip()
            quantity = cells[3].strip() if len(cells) == 5 else level
            if not word or not level or not quantity:
                bad("登记表", f"{registry_path}:{line_number} 频率词行的词、量级或量是空的")
            elif word in frequency_level:
                bad("登记表", f"{registry_path}:{line_number} 频率词「{word}」登记了两次")
            else:
                frequency_level[word] = level
                frequency_quantity[word] = quantity
        elif cells[0] == "结构" and len(cells) == 6:
            name, aliases, registered, authority = (cell.strip() for cell in cells[1:5])
            alias_list = [alias.strip() for alias in aliases.split("|") if alias.strip()]
            if not name or not alias_list or not registered or not authority:
                bad("登记表", f"{registry_path}:{line_number} 结构行有空列：{line[:60]}")
            else:
                structures.append((line_number, name, alias_list, registered, authority))
        else:
            bad("登记表", f"{registry_path}:{line_number} 认不出的行（第一列写「频率词」带 4 或 5 列，或写「结构」带 6 列）：{line[:60]}")

for line_number, name, _aliases, registered, _authority in structures:
    if registered not in frequency_level:
        bad("登记表", f"{registry_path}:{line_number}「{name}」登记的频率「{registered}」不在频率词表里")
structures = [row for row in structures if row[3] in frequency_level]
alias_owner = {}
for line_number, name, alias_list, _registered, _authority in structures:
    for alias in alias_list:
        if alias in alias_owner and alias_owner[alias] != name:
            bad("登记表", f"{registry_path}:{line_number} 叫法「{alias}」同时登记给了「{alias_owner[alias]}」与「{name}」")
        alias_owner[alias] = name

# ── 权威分项在、已定、简称对得上（归属借 lib-item-ref-status.py）────────────────────────
spec = importlib.util.spec_from_file_location("item_ref_status", lib_path)
item_ref_status = importlib.util.module_from_spec(spec)
spec.loader.exec_module(item_ref_status)
# 权威分项点名的决策号对照中文原文（登记表是中文；heads_for_referrer 只留中文那几份）。load_map() 现读全部首行时，
# 读不出的译文首行让这一格当场退 1，后面的判据一条都没走；译文的首行归格「分项引用状态」判，中文原文的首行读不出照旧判红。
heads, unreadable = item_ref_status.decision_heads(strict=False)
heads, unreadable = item_ref_status.heads_for_referrer(heads, unreadable, registry_path)
for unreadable_path, first_line in unreadable:
    bad("权威", f"{unreadable_path} 首行读不出 `## D<n> 简称 —— 状态`（首行是 {first_line[:40]!r}），这一份的分项一条都核不了")
item_map, short_names = item_ref_status.load_map(heads)
AUTHORITY = re.compile(r"^(D\d+)（((?:[^（）]|（[^（）]*）)*)）\s*已定项\s*(\d+)$")
for line_number, name, _aliases, _registered, authority in structures:
    matched = AUTHORITY.match(authority)
    if not matched:
        bad("权威", f"{registry_path}:{line_number}「{name}」的权威分项「{authority}」不是「D<n>（简称） 已定项 k」的写法")
        continue
    decision, short_name, item = matched.group(1), matched.group(2), int(matched.group(3))
    if decision not in item_map:
        bad("权威", f"{registry_path}:{line_number}「{name}」的权威分项点名 {decision}，而 .claude/kb/decisions/ 里没有这条决策")
    elif short_names[decision] != short_name:
        bad("权威", f"{registry_path}:{line_number}「{name}」把 {decision} 的简称写成「{short_name}」，正文首行是「{short_names[decision]}」")
    elif item_map[decision].get(item) != "已":
        state = "没有这一条" if item not in item_map[decision] else "它是未定项"
        bad("权威", f"{registry_path}:{line_number}「{name}」的权威分项是 {decision} 第 {item} 条，而那条决策的分项表里{state}——权威只能是已定项")

# ── 读排除表 ────────────────────────
exclusions = []   # [行号, 决策文件名, 句中逐字的一段, 排到过没有]
if os.path.exists(exclude_path):
    with open(exclude_path, encoding="utf-8") as handle:
        for line_number, raw in enumerate(handle, 1):
            line = raw.rstrip("\n")
            if not line.strip() or line.startswith("#"):
                continue
            cells = line.split("\t")
            if len(cells) != 3 or not cells[2].startswith("#") or len(cells[2].lstrip("#").strip()) < 4:
                bad("排除表", f"{exclude_path}:{line_number} 不是「决策文件名<TAB>句中逐字的一段<TAB># 理由」：{line[:60]}")
                continue
            file_name, snippet = cells[0].strip(), cells[1].strip()
            if not os.path.exists(os.path.join(".claude/kb/decisions", file_name)):
                bad("排除表", f"{exclude_path}:{line_number} 指的 .claude/kb/decisions/{file_name} 不存在")
                continue
            if len(snippet) < 8:
                bad("排除表", f"{exclude_path}:{line_number} 句中那一段不足 8 个字，会排到不相干的句子：{snippet}")
                continue
            exclusions.append([line_number, file_name, snippet, False])

# ── 扫决策正文 ────────────────────────
ordered_words = sorted(frequency_level, key=len, reverse=True)
FREQUENCY = re.compile("|".join(f"(?:{phrase_pattern(word)})" for word in ordered_words)) if ordered_words else None
word_of = {re.sub(r"\s+", "", word): word for word in ordered_words}
# gap 是频率词与写动词之间那一段，留着看里面有没有否定词（「每次发布不写」）
WRITE_AFTER = re.compile(r"^(?P<gap>[^，。；：、,;:（）()「」“”|]{0,12}?)(?:写(?!放大)|更新|轮换|推进|回写|重写|改写|覆写|落盘|涨)|^\s*一次")
# 否定词在频率词前面，中间只许隔三个以内的虚字（「不会在每次发布时写」「不再每次发布推进」）
NEGATED_BEFORE = re.compile(r"(?:不是|并非|而非|没有|无需|无须|不必|不用|(?<!除)非|不|没)\s*[都是在随按会再需要必用应该能可就也于]{0,3}\s*$")
NEGATED_BEFORE_ADJACENT = re.compile(r"(?:不是|并非|而非|没有|无需|不必|不用|不|没|非)\s*[都是在随按]?\s*$")   # 弄坏开关换回的旧判法
NEGATED_BEFORE_VERB = re.compile(r"不(?![仅止只但同变论管])|没|无需|无须")
RESTRICTED_BEFORE = re.compile(r"(?:只有|只|仅)[在于]?\s*$")
RESTRICTION_NEGATED = re.compile(r"(?:不是|并非|而非|不)\s*(?:只有|只|仅)[在于]?\s*$")
alias_patterns = [(re.compile(phrase_pattern(alias)), alias_owner[alias]) for alias in sorted(alias_owner, key=len, reverse=True)]
registered_of = {name: (registered, authority) for _ln, name, _al, registered, authority in structures}
SOURCE_NOTE = {"sentence": "", "row": "（叫法在同一表格行的别的格）", "heading": "（叫法在所属标题）"}


def owners_in(text):
    return {owner for pattern, owner in alias_patterns if pattern.search(text)}


def judge(written, registered, form):
    """form 是 plain / negated / restricted；返回这一处与登记说不说反话。"""
    same_level = levels_merged or frequency_level[written] == frequency_level[registered]
    written_quantity, registered_quantity = frequency_quantity[written], frequency_quantity[registered]
    registered_is_whole_level = registered_quantity == frequency_level[registered]
    if form == "negated":     # 否定的时机被登记的那些时机罩住
        return same_level and (registered_is_whole_level or written_quantity == registered_quantity)
    if form == "restricted":  # 「只在 X 时」：X 与登记的必须是同一个量
        return written_quantity != registered_quantity
    return not same_level or (not registered_is_whole_level and written_quantity != registered_quantity)


files = sorted(glob.glob(".claude/kb/decisions/*.md"))
sentences_scanned = 0
assertion_sentences = 0
attributed_hits = 0
hits_by_source = {"sentence": 0, "row": 0, "heading": 0}
restricted_hits = 0
negations_ignored = 0
hit_structures = set()
red_hits = []   # (文件名, 行号, 结构, 写的词, 登记的词, 权威, 句子, 类别, 叫法从哪来)
for path in files:
    body = open(path, encoding="utf-8").read().split("\n## 历史版本")[0]
    in_fence = False
    heading_owners = set()
    for line_number, line in enumerate(body.split("\n"), 1):
        if line.lstrip().startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        plain = line.replace("**", "").replace("`", "")
        if plain.startswith("#"):
            heading_owners = owners_in(plain)
        cells = re.split(r"(?<!\\)\|", plain)
        cell_owners = [owners_in(cell) for cell in cells] if plain.lstrip().startswith("|") and len(cells) > 2 else []
        for index, cell in enumerate(cells):
            row_owners = set().union(*(owners for other, owners in enumerate(cell_owners) if other != index))
            for sentence in re.split(r"(?<=[。；！？])", cell):
                if not sentence.strip():
                    continue
                sentences_scanned += 1
                if FREQUENCY is None:
                    continue
                owners, source = owners_in(sentence), "sentence"
                if not owners and not subject_in_sentence_only:
                    if row_owners:
                        owners, source = row_owners, "row"
                    elif heading_owners:
                        owners, source = heading_owners, "heading"
                sentence_has_assertion = False
                for found in FREQUENCY.finditer(sentence):
                    after = WRITE_AFTER.search(sentence[found.end():])
                    if not after:
                        continue
                    sentence_has_assertion = True
                    written = word_of[re.sub(r"\s+", "", found.group(0))]
                    before = sentence[:found.start()]
                    if negation_adjacent_only:
                        negated = bool(NEGATED_BEFORE_ADJACENT.search(before))
                    else:
                        negated = bool(NEGATED_BEFORE.search(before) or NEGATED_BEFORE_VERB.search(after.group("gap") or ""))
                    if negation_counted:
                        form = "plain"
                    elif not restriction_as_plain and RESTRICTION_NEGATED.search(before):
                        form = "restriction-negated"   # 「不只在 X 时写」：还在别的时机写，不划界
                    elif negated:
                        form = "negated"
                    elif not restriction_as_plain and RESTRICTED_BEFORE.search(before):
                        form = "restricted"
                    else:
                        form = "plain"
                    for owner in sorted(owners):
                        attributed_hits += 1
                        hits_by_source[source] += 1
                        hit_structures.add(owner)
                        registered, authority = registered_of[owner]
                        if form == "restricted":
                            restricted_hits += 1
                        if form == "restriction-negated" or not judge(written, registered, form):
                            if form in ("negated", "restriction-negated"):
                                negations_ignored += 1
                            continue
                        red_hits.append((os.path.basename(path), line_number, owner, written, registered, authority, sentence.strip(), form, source))
                if sentence_has_assertion:
                    assertion_sentences += 1

for file_name, line_number, owner, written, registered, authority, sentence, form, source in red_hits:
    covering = [exclusion for exclusion in exclusions if exclusion[1] == file_name and exclusion[2] in sentence]
    for exclusion in covering:
        exclusion[3] = True
    if covering:
        continue
    where = f"{file_name}:{line_number}「{owner}」{SOURCE_NOTE[source]}"
    if form == "plain":
        bad("写频率", f"{where}写成「{written}」（{frequency_level[written]}级），登记的是「{registered}」（{frequency_level[registered]}级），权威：{authority}｜{sentence[:80]}")
    elif form == "negated":
        bad("写频率", f"{where}否定了「{written}」，而登记的「{registered}」与它同属{frequency_level[registered]}级，权威：{authority}｜{sentence[:80]}")
    else:
        bad("写频率", f"{where}用「只在 / 仅」把写的时机限定到「{written}」（量：{frequency_quantity[written]}），登记的「{registered}」量是{frequency_quantity[registered]}，"
                      f"不是同一个量，权威：{authority}｜{sentence[:80]}")
for line_number, file_name, snippet, used in exclusions:
    if not used:
        bad("排除表", f"{exclude_path}:{line_number}（{file_name}「{snippet}」）没排到任何一句判红的断言：句子改过了或已经不红，删掉这一行")

if not structures:
    bad("登记表", f"{registry_path} 一个结构都没登记（或登记的频率全不在词表里）")
elif attributed_hits == 0:
    bad("写频率", f"登记了 {len(structures)} 个结构，{len(files)} 份决策正文里一处落在它们身上的频率断言都没扫到——叫法对不上正文了，或者正文搬了地方")

if problems:
    kinds = sorted({kind for kind, _ in problems})
    print(f"  ✗ 写频率有 {len(problems)} 处不对（{'、'.join(kinds)}）：")   # gate-lint:summary
    for kind, detail in problems:
        print(f"      [{kind}] {detail}")                                  # gate-lint:detail
    print("  → 怎么办：[写频率] 按那一行点名的权威分项改这句的频率；权威分项自己改了，先改 .claude/gate.d/write-frequency.tsv 那个结构的登记，")
    print("    再把别处的说法一起改到同一个量级（C245（系统配置槽的写频率，两条条款说反话） 那一族：不许两处各留各的）。")
    print("    这句确实不是在说这个结构多久写一次，把「决策文件名<TAB>句中逐字的一段<TAB># 理由」加进 .claude/gate.d/write-frequency-exclude.tsv。")
    print("    [登记表] [权威] [排除表] 照明细改那张表的那一行；两张表的写法在各自的文件头。改完只重跑这一格：--check write-frequency")
    sys.exit(1)

missed = [name for _ln, name, _al, _r, _a in structures if name not in hit_structures]
levels = sorted(set(frequency_level.values()))
print(f"  ✓ 同一结构的写频率只有一个量级（扫 {len(files)} 份决策正文、{sentences_scanned} 句，{assertion_sentences} 句有频率断言、"
      f"{attributed_hits} 处落在登记的结构上（叫法在句内 {hits_by_source['sentence']}、同一表格行 {hits_by_source['row']}、所属标题 {hits_by_source['heading']}），"
      f"其中「只在 / 仅」限定形态 {restricted_hits} 处；登记 {len(structures)} 个结构、{len(levels)} 个量级、{len(set(frequency_quantity.values()))} 个量、"
      f"{len(frequency_level)} 个频率词；否定形态 {negations_ignored} 处不算断言；排除表 {len(exclusions)} 行）")
if missed:
    print(f"    没扫到断言的结构 {len(missed)} 个（叫法没和频率断言同句、同行或同标题出现，这几个没判）：{'；'.join(missed)}")
PY
)

# ════════════════ 后三格：变更史 ════════════════

# 在仓里时取 gate 基准写进 diff_base；不在仓里 diff_base 留空。source 失败、取不到基准就判红退出。
load_diff_base() {
  local LIB_CHANGED_PATHS
  diff_base=""
  git rev-parse --is-inside-work-tree >/dev/null 2>&1 || return 0
  LIB_CHANGED_PATHS="$REPO_ROOT/scripts/gate/changed-paths.sh"
  # shellcheck source=../../scripts/gate/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用库 $LIB_CHANGED_PATHS"; echo "     → 怎么办：改动范围的取法只有那一份，恢复它，别在阶段里再抄一份。"; exit 1; }
  diff_base="$(gate_diff_base gate)" || { echo "  ✗ 共用库算不出 diff 基准"; echo "     → 怎么办：照上面 gate_diff_base 的报错修（多半是 GATE_BASE 写错了），再跑。"; exit 1; }
}

# ── entry-added：决策正文改了，变更史里要有新增条目 ──
# 改了 decisions.md 或 decisions/ 却没在 decisions-history.md 新增条目，判红（.claude/rules/format-evolution.md「硬约束」：已定决策变更必须记进变更史）。
# 只数有没有新增：顶格 `- ` 变更项，一个都没有按 `### 日期` 标题数；条目写得对不对靠人。
# 基准取 changed-paths.sh 的 gate_diff_base gate：已提交没推出去的决策改动也算。
# 小改动（增删合计不超过 4 行、一行状态都没碰：首行「## D<n> 简称 —— 状态」、「状态：」、已定项 / 未定项标题）不要求变更史，退 77。
# 两条路径放数组：拼成一个字符串再加引号会变成单个 pathspec，什么都匹配不到。
# 与基准只差登记搬迁（.claude/gate.d/path-moves.tsv）的决策文件不算改了：列出与基准不同的决策文件，过一遍 changed-paths.sh 的 gate_drop_moved_only
#   （判法 lib-lag-baseline.py 的 moved_only_paths，与改动范围剔得一样），剩下的才数行、看状态行；一份都不剩退 77。剩下的文件里因搬迁换了路径的行照样计进行数（只按文件剔）。
#   样本 entry-added-moved-only-not-run（只差搬迁，退 77）、entry-added-moved-unregistered-red（没登记，红）、entry-added-moved-and-edited-red（登记了又改了一行，红）；
#   弄坏开关 LAG_BASELINE_BREAK=moved-only-kept 下 entry-added-moved-only-not-run 判红。
cell_entry_added() {
  local KB HIST diff_base diff_rc numstat changed decision_diff status_lines_touched added hist_diff listed kept
  KB=(.claude/kb/decisions.md .claude/kb/decisions)
  HIST=.claude/kb/decisions-history.md

  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "  ! 不在 git 仓库里，这一格无对象可判"
    exit 77
  fi
  # 在仓里时每一道 git 都要成功：失败了判红，不许读成「没改」
  git_failed() {
    bad "git $1 跑不起来（退出码 $2），决策正文改了多少、变更史加了几条都没数出来"
    howto "按上面 git 的报错修好仓库状态再跑（常见是还没有任何提交、HEAD 不存在：先提交一次）；"
    howto "  git 失败时这一格什么都没比，不是通过。"
    exit 1
  }
  load_diff_base
  diff_rc=0
  git diff --quiet "$diff_base" -- "${KB[@]}" || diff_rc=$?
  case "$diff_rc" in
    0) echo "  ! 决策正文与基准 $diff_base 无差异，这一格无对象可判"; exit 77 ;;
    1) ;;
    *) git_failed "diff --quiet $diff_base" "$diff_rc" ;;
  esac
  # 与基准只差登记搬迁的决策文件不算改了（scripts/gate/changed-paths.sh 的 gate_drop_moved_only，与改动范围剔得一样）：剩下的才数行、看状态行
  listed="$(git -c core.quotepath=false diff --name-only "$diff_base" -- "${KB[@]}")" || git_failed "diff --name-only" "$?"
  kept="$(gate_drop_moved_only "$diff_base" <<<"$listed")" || git_failed "diff（只差搬迁那一层，lib-lag-baseline.py moved-only-filter）" "$?"
  if [[ -z "$kept" ]]; then
    echo "  ! 决策正文与基准 $diff_base 只差登记的搬迁（.claude/gate.d/path-moves.tsv），这一格无对象可判"
    exit 77
  fi
  mapfile -t KB <<<"$kept"

  numstat="$(git diff "$diff_base" --numstat -- "${KB[@]}")" || git_failed "diff --numstat" "$?"
  changed=$(awk '{n+=$1+$2} END{print n+0}' <<<"$numstat")
  # 碰了状态行就不算小改动：「—— 半定」改成「—— 已定」只有两行，却最该留痕
  decision_diff="$(git diff "$diff_base" -U0 -- "${KB[@]}")" || git_failed "diff -U0" "$?"
  status_lines_touched=$(grep -cE '^[+-](## D[0-9]+ .*——|#{3,4} *(已定项|未定项)|.*状态：)' <<<"$decision_diff" || true)
  hist_diff="$(git diff "$diff_base" -- "$HIST")" || git_failed "diff（变更史）" "$?"
  added=$(grep -c '^+- ' <<<"$hist_diff" || true)
  if ((added == 0)); then added=$(grep -c '^+### 20[0-9][0-9]-' <<<"$hist_diff" || true); fi

  if [[ "$added" -gt 0 ]]; then
    ok "决策正文改了 $changed 行，变更史新增 $added 条条目"
    exit 0
  fi
  if [[ "$changed" -le 4 && "$status_lines_touched" -eq 0 ]]; then
    echo "  ! 本次未判：决策正文只改了 $changed 行、没碰状态行，按小改动不要求变更史（不记通过）"
    exit 77
  fi

  if [[ "$status_lines_touched" -gt 0 ]]; then
    bad "决策正文改了 $changed 行（碰了 $status_lines_touched 行状态行，不按小改动放行），却没往变更史新增任何条目"
  else
    bad "决策正文改了 $changed 行，却没往变更史新增任何条目"
  fi
  howto "若这次改动推翻或定下了任何结论，在 $HIST 里那条决策的 \`## D<n>（简称）\` 节里加一条变更（.claude/rules/changelog-format.md）："
  howto "  这一天的 \`### $(TZ=Asia/Tokyo date +%F)\` 块已经有就在块里最下面加一个顶格 \`- \` 项，没有就在节标题下面新开一块（新的日期在上）；"
  howto "  项里写结论、依据与改前（改前写一行 git show <提交>:<路径>，不抄原文）；一条改动点名了几条决策，只在标题里点名的第一条决策的节里写全文，别的节写一行「- 见 D<n>（简称） 节同日条目「摘要」」。"
  howto "已定 / 未定项数或一句话现状变了，改 decisions.md 的索引表再跑 SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync 同步索引表的状态列。"
  howto "纯排版改动可以拆成单独一个提交，那时这一格就无对象可判了。"
  exit 1
}

# ── history-loss：变更史整理不许丢 ──
# 整理变更项（.claude/rules/changelog-format.md）不许丢数、编号、产物名、别的日期、引名。两份变更史各拿基准里那一版与现在的比：
# scripts/kb/history-consolidate.py loss-check（判法与出路在它的文件头），任一份退非 0 判红；都与基准无差异或基准里没有退 77。
# 基准那一版经 lib-lag-baseline.py show 取（三种结局只在那一份：退 0 有、退 3 基准里没有这份、别的是 git 取不到）：取不到判红、点名那一份，
#   不当成基准里没有（cat-file -e 失败就跳过会让两份都跳过、报「与基准无差异」退 77，整理丢没丢其实没比）。
# 样本 history-loss-base-unreadable-red：基准那一版变更史的对象从对象库里删掉，判红在「读不出基准 HEAD 里的 .claude/kb/decisions-history.md」。
# 登记了搬迁（.claude/gate.d/path-moves.tsv）时，基准那一版照登记换算，「<提交哈希>:」后面的路径不换（git show <提交>:<路径> 有意留旧路径）：
#   样本 history-loss-moved-commit-ref-green 判绿；history-loss-moved-unregistered-red（没登记）、history-loss-moved-and-edited-red（登记了又删了一个数）判红；
#   弄坏开关 LAG_BASELINE_BREAK=commit-ref-rewritten 下 history-loss-moved-commit-ref-green 判红。
# 登记行先进了基准、搬迁这一次才做的（effective_among 不认它，show 不换）：loss-check 另带 --moves-base <基准>，照「基准的树里旧前缀下有文件、新前缀下没有」的登记行
#   再换一遍基准那一版（history-consolidate.py 文件头「用法」）。术语改名表不换：变更史里的旧叫法是当时的原话。
#   样本 history-loss-moved-registered-in-base-green 判绿；弄坏开关 DOC_DECISIONS_BREAK=history-loss-registered-moves-ignored 下不交 --moves-base，那一份判红。
cell_history_loss() {
  local diff_base checked=0 failed=0 unreadable=0 hist base_copy rc base_exit base_problem moves_base_option=()
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "  ! 不在 git 仓库里，这一格无对象可判"
    exit 77
  fi
  load_diff_base
  [[ ",$BREAK_SWITCH," == *",history-loss-registered-moves-ignored,"* ]] || moves_base_option=(--moves-base "$diff_base")
  for hist in .claude/kb/decisions-history.md .claude/kb/experiments-history.md; do
    [[ -f "$hist" ]] || continue
    if git diff --quiet "$diff_base" -- "$hist" 2>/dev/null; then continue; fi
    base_copy="$(mktemp "${TMPDIR:-/tmp}/history-loss-base.XXXXXX")"
    base_exit=0
    base_problem="$(python3 "$LIB_LAG_BASELINE" show "$diff_base" "$hist" 2>&1 >"$base_copy")" || base_exit=$?
    if ((base_exit == 3)); then rm -f "$base_copy"; continue; fi   # 基准里没有这份变更史：没有可比的
    if ((base_exit != 0)); then
      failed=$((failed + 1)); unreadable=$((unreadable + 1))
      echo "  ✗ $hist：${base_problem:-lib-lag-baseline.py show 退 $base_exit}，整理丢没丢没比"   # gate-lint:summary
      rm -f "$base_copy"
      continue
    fi
    rc=0
    python3 "$REPO_ROOT/scripts/kb/history-consolidate.py" loss-check "$base_copy" "$hist" --repo "$PWD" ${moves_base_option[@]+"${moves_base_option[@]}"} --force > "$base_copy.out" 2>&1 || rc=$?
    checked=$((checked + 1))
    if ((rc != 0)); then
      failed=$((failed + 1))
      echo "  ✗ $hist：整理之后基准里的这几块有数或编号找不到了"   # gate-lint:summary
      grep -E '^  ' "$base_copy.out" | sed 's/^/     /'   # gate-lint:detail
    fi
    rm -f "$base_copy" "$base_copy.out"
  done
  if ((failed)); then
    if ((unreadable)); then
      howto "取不到基准那一版的：照括号里 git 的报错修仓库状态（基准名不在、对象库缺了那一份），再只重跑这一格；取不到时整理丢没丢没比，不是没丢。"
    fi
    if ((failed > unreadable)); then
      howto "红行列出的数或编号逐个补回那一块、改成指针或 git 引用；判法与出路在 scripts/kb/history-consolidate.py 文件头。"
    fi
    exit 1
  fi
  if ((checked == 0)); then
    echo "  ! 两份变更史与基准 $diff_base 无差异，这一格无对象可判"
    exit 77
  fi
  ok "变更史整理没丢：查了 $checked 份，基准里每个日期块的数、编号、产物名、引名在改后都找得到"
}

# ── shape：变更史的形状 ──
# 形态是「条目 → 日期 → 变更清单」：每条决策 / 实验一个 `## D<n>（` / `## E<n>（` 节，节里按 `### YYYY-MM-DD` 分块，块下每条改动一个顶格 `- ` 项（.claude/rules/changelog-format.md）。判五样：
#   a) .claude/kb/ 下任何 .md 里 `### 20YY-MM-DD` 后面直接跟「（其…）」
#   b) 两份变更史里日期块的上一级标题不是 `## D<n>（` / `## E<n>（`；文末「## 历史版本」里的块不算节外，但块里标题或 `- ` 项点名了 `D<n>（` / `E<n>（` 的照样判红
#   c) 同一节里同一个日期的 `### ` 出现两次以上
#   d) 两份变更史里的 `#### ` 子条目、「> 快查·」行、「**现状**：」行
#   e) 节里的 `### 日期` 块一条 `- ` 项都没有
# 只判这一轮新增的（基准同 entry-added）：a、d 看那一行，b 看块里任一行，c 看重复的几行里任一行，e 看 `### ` 行；基准里就有的只计数不判红。
# 新增行按 `git diff -U0` 的 hunk 头取行号（changed-paths.sh 的 gate_added_lines 只给行文），未跟踪的整份算新增；不在 git 仓里整棵树按新增判。围栏里的行不算。
# 登记了搬迁（.claude/gate.d/path-moves.tsv）时，只因搬迁才变了的行不算新增（without_moved_lines 经 lib-lag-baseline.py 的 drop_moved_lines 剔，与 gate_added_lines 剔的同一种）：
#   样本 shape-moved-only-green 判绿，shape-moved-unregistered-red、shape-moved-and-edited-red 判红；弄坏开关 LAG_BASELINE_BREAK=moved-only-kept 下 shape-moved-only-green 判红。
# 判不了：变更项写得对不对、该点名的决策或实验点没点全、日期块是不是倒序。
cell_shape() {
  local diff_base
  load_diff_base
  LIB_LAG_BASELINE="$LIB_LAG_BASELINE" python3 - "$diff_base" <<'PY'
import os, re, subprocess, sys

KB = '.claude/kb'
HISTORY_KIND = {'.claude/kb/decisions-history.md': 'D', '.claude/kb/experiments-history.md': 'E'}
KIND_NAME = {'D': '决策', 'E': '实验'}
FENCE = re.compile(r'^[ \t]*```')
FUSED = re.compile(r'^###[ \t]+20\d\d-\d\d-\d\d[ \t]*（其')
DATE_HEADING = re.compile(r'^###[ \t]+(20\d\d-\d\d-\d\d)')
UPPER_HEADING = re.compile(r'^#{1,2}[ \t]')
BLOCK_END = re.compile(r'^#{1,3}[ \t]')
SUB_HEADING = re.compile(r'^####[ \t]')
OWN_HISTORY = re.compile(r'^## 历史版本[ \t]*$')
HUNK = re.compile(r'^@@ -\S+ \+(\d+)(?:,(\d+))? @@')
BREAK = os.environ.get('DOC_DECISIONS_BREAK', '')
CHECK_COMMAND = 'bash .claude/gate.d/doc-decisions.sh --check shape'
base = sys.argv[1]


class DiffUnreadable(Exception):
    pass


def git(*arguments):
    result = subprocess.run(['git', '-c', 'core.quotepath=false', *arguments],
                            capture_output=True, encoding='utf-8', errors='replace')
    if result.returncode != 0:
        raise DiffUnreadable(f'git {arguments[0]} 退出 {result.returncode}：{result.stderr.strip()[:200]}')
    return result.stdout


def added_line_numbers():
    """（{路径: 新增的行号}, {整份算新增的路径}）；base 为空（不在 git 仓里）返回 None：整棵树都按新增判。"""
    if not base:
        return None
    added, path, in_header = {}, None, False
    diff = git('diff', '--no-renames', '--unified=0', '--no-color', '--no-ext-diff',
               '--src-prefix=a/', '--dst-prefix=b/', base, '--', KB)
    for line in diff.split('\n'):
        if line.startswith('diff --git '):
            in_header, path = True, None
        elif in_header and line.startswith('+++ '):
            target = line[4:].rstrip('\t')
            if target.startswith('"'):
                raise DiffUnreadable(f'git 给路径加了引号（{target}），还原不了是哪一份')
            path = None if target == '/dev/null' else (target[2:] if target.startswith('b/') else target)
        elif line.startswith('@@ '):
            in_header = False
            match = HUNK.match(line)
            if match and path:
                start, count = int(match.group(1)), int(match.group(2) or 1)
                added.setdefault(path, set()).update(range(start, start + count))
    whole = {item for item in git('ls-files', '-z', '--others', '--exclude-standard', '--', KB).split('\0') if item}
    return without_moved_lines(added, whole)


def without_moved_lines(added, whole):
    """只因登记的搬迁才变了的行不算新增（lib-lag-baseline.py 的 drop_moved_lines，与 changed-paths.sh 的 gate_added_lines 剔得一样）。
    搬迁登记表没有内容行时原样交回；整份算新增的（未跟踪）也逐行过一遍，过完按行号记。登记表写坏了照没有登记时交回、打一行提示；git 取不到判红。"""
    import importlib.util
    spec = importlib.util.spec_from_file_location('lag_baseline', os.environ['LIB_LAG_BASELINE'])
    lag_baseline = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(lag_baseline)
    moves, problems = lag_baseline.path_moves()
    if not moves and not problems:
        return added, whole
    entries, unread = [], set()
    for path in sorted(set(added) | whole):
        try:
            with open(path, encoding='utf-8', errors='surrogateescape') as handle:
                lines = handle.read().split('\n')
        except OSError:
            unread.add(path)
            continue
        numbers = range(1, len(lines) + 1) if path in whole else sorted(added[path])
        entries += [(path, lines[number - 1], number) for number in numbers if 0 < number <= len(lines)]
    kept, problem = lag_baseline.drop_moved_lines(base, entries)
    if isinstance(problem, lag_baseline.PathMovesProblem):
        print(f'  ! {problem.sentence()}；新增行没剔只差搬迁的')
    elif problem is not None:
        raise DiffUnreadable(problem.sentence())
    remaining = {}
    for path, _text, number in kept:
        remaining.setdefault(path, set()).add(number)
    for path in unread:
        if path in added:
            remaining[path] = added[path]
    return remaining, {path for path in whole if path in unread}


def is_new(changes, path, numbers):
    if changes is None or BREAK == 'stock-judged':
        return True
    added, whole = changes
    return path in whole or bool(added.get(path, set()) & set(numbers))


def unfenced_lines(path):
    """[(行号, 行)]；围栏代码块里的行（含围栏行本身）不给。"""
    out, fenced = [], False
    with open(path, encoding='utf-8', errors='replace') as handle:
        for number, line in enumerate(handle, 1):
            line = line.rstrip('\n')
            if FENCE.match(line):
                fenced = not fenced
                continue
            if not fenced:
                out.append((number, line))
    return out


def kb_files():
    out = []
    for directory, subdirectories, names in os.walk(KB):
        subdirectories.sort()
        out += [os.path.join(directory, name) for name in sorted(names) if name.endswith('.md')]
    return out


if not os.path.isdir(KB):
    print(f'  ! 没有 {KB}，这一格无对象可判')
    sys.exit(77)
try:
    changes = added_line_numbers()
except DiffUnreadable as error:
    print(f'  ✗ 取不到这一轮新增了哪些行：{error}')  # gate-lint:summary
    print('     → 按上面 git 的报错修好仓库状态再跑；取不到新增行时这一格什么都没比，不是通过。')
    sys.exit(1)

fused_new, no_home_new, duplicate_new, wrapper_new, empty_new = [], [], [], [], []
fused_stock = no_home_stock = duplicate_stock = wrapper_stock = empty_stock = 0
WRAPPER = re.compile(r'^(?:####[ \t]|>[ \t]*快查·|\*\*现状\*\*：)')
ITEM = re.compile(r'^- ')
files = kb_files()
date_headings = history_date_headings = 0
for path in files:
    lines = unfenced_lines(path)
    for number, line in lines:
        if DATE_HEADING.match(line):
            date_headings += 1
        if FUSED.match(line) and BREAK != 'fused-ignored':
            if is_new(changes, path, [number]):
                fused_new.append(f'{path}:{number}  {line}')
            else:
                fused_stock += 1
    kind = HISTORY_KIND.get(path)
    if kind is None:
        continue
    section_heading = re.compile(rf'^## {kind}\d+（')
    mention = re.compile(rf'(?<![\w-]){kind}\d+（')
    parent = None
    by_section = {}
    for index, (number, line) in enumerate(lines):
        if UPPER_HEADING.match(line):
            parent = (number, line)
            continue
        if WRAPPER.match(line) and BREAK != 'wrapper-ignored':
            if is_new(changes, path, [number]):
                wrapper_new.append(f'{path}:{number}  {line[:70]}')
            else:
                wrapper_stock += 1
            continue
        match = DATE_HEADING.match(line)
        if not match:
            continue
        history_date_headings += 1
        block = [(number, line)]
        items = 0
        for later_number, later_line in lines[index + 1:]:
            if BLOCK_END.match(later_line):
                break
            block.append((later_number, later_line))
            if ITEM.match(later_line):
                items += 1
        in_section = bool(parent and section_heading.match(parent[1]))
        if in_section and items == 0 and BREAK != 'empty-ignored':
            if is_new(changes, path, [number]):
                empty_new.append(f'{path}:{number}  {line}')
            else:
                empty_stock += 1
        if in_section:
            by_section.setdefault(parent, {}).setdefault(match.group(1), []).append(number)
            continue
        if parent and OWN_HISTORY.match(parent[1]):
            named = sorted({found.rstrip('（') for _, text in block if text.startswith('#') or ITEM.match(text) for found in mention.findall(text)})
            if not named:
                continue
            reason = f'在文件自己的「## 历史版本」里，标题或变更项点名了 {"、".join(named)}，要写进那一节'
        else:
            upper = parent[1] if parent else '（没有，在文件开头）'
            reason = f'不在任何 `## {kind}<n>（` 节里，上一级标题是「{upper}」'
        if BREAK == 'no-home-ignored':
            continue
        if is_new(changes, path, [item_number for item_number, _ in block]):
            no_home_new.append(f'{path}:{number}  {line}  {reason}')
        else:
            no_home_stock += 1
    for (section_number, section_line), dates in by_section.items():
        for date, numbers in dates.items():
            if len(numbers) < 2 or BREAK == 'duplicate-date-ignored':
                continue
            if is_new(changes, path, numbers):
                duplicate_new.append(f'{path}:{section_number} {section_line} 节里 ### {date} 出现 {len(numbers)} 次'
                                     f'（第 {"、".join(map(str, numbers))} 行）')
            else:
                duplicate_stock += 1

scope = '不在 git 仓里、没有基准：整棵树都按新增判' if changes is None else f'只判基准 {base[:12]} 之后新增的行'
stock = ('没有存量一说' if changes is None else
         f'存量（基准里就有，等搬迁，只计数不判红）：日期与序号粘在一起 {fused_stock} 处、'
         f'条目住在节外 {no_home_stock} 处、同节日期重复 {duplicate_stock} 组、旧形态包装 {wrapper_stock} 行、空日期块 {empty_stock} 个')
failed = False
if fused_new:
    failed = True
    print(f'  ✗ {len(fused_new)} 处新写的标题把日期与「（其N）」粘在同一个 `### ` 里')  # gate-lint:summary
    for item in fused_new:
        print(f'     {item}')  # gate-lint:detail
    print('     → 同一天的改动收进一个 `### YYYY-MM-DD`（这一组里这一天已经有就并进去），块下每条改动一个顶格 `- ` 项，「（其N）」不再写。')
    print('       规则：.claude/rules/changelog-format.md。')
if no_home_new:
    failed = True
    print(f'  ✗ {len(no_home_new)} 个新写的日期块没住进自己的节')  # gate-lint:summary
    for item in no_home_new:
        print(f'     {item}')  # gate-lint:detail
    print('     → decisions-history.md 的变更写进它点名的那条决策的 `## D<n>（简称）` 节，experiments-history.md 的写进 `## E<n>（简称）` 节；')
    print('       点名了几条只在标题里点名的第一条的节里写全文，别的节写一行「- 见 … 节同日条目」。文件自己的「## 历史版本」只写这一份文件自己的改动。规则：.claude/rules/changelog-format.md。')
if duplicate_new:
    failed = True
    print(f'  ✗ {len(duplicate_new)} 组同一节里同一个日期开了几个 `### ` 块')  # gate-lint:summary
    for item in duplicate_new:
        print(f'     {item}')  # gate-lint:detail
    print('     → 同一天的改动并进一个 `### 日期` 块，块下每条改动一个顶格 `- ` 项（先写的在上面）；多出来的 `### 日期` 行删掉。')
if wrapper_new:
    failed = True
    print(f'  ✗ {len(wrapper_new)} 行新写进变更史的旧形态包装（`#### ` 子条目、「> 快查·」行或「**现状**：」行）')  # gate-lint:summary
    for item in wrapper_new:
        print(f'     {item}')  # gate-lint:detail
    print('     → 变更写成顶格 `- ` 项、结论写在项里；快查两行删掉；现状只在 decisions.md / experiments.md 的索引表里写。规则：.claude/rules/changelog-format.md。')
if empty_new:
    failed = True
    print(f'  ✗ {len(empty_new)} 个新开的 `### 日期` 块下一条 `- ` 变更项都没有')  # gate-lint:summary
    for item in empty_new:
        print(f'     {item}')  # gate-lint:detail
    print('     → 在块下写至少一条顶格 `- ` 项说明改了什么，或者删掉这个空块。')
if failed:
    print(f'     {scope}；{stock}')
    print(f'     → 改完只重跑这一格：{CHECK_COMMAND}')
    sys.exit(1)
if date_headings == 0:
    print(f'  ! {KB} 下 {len(files)} 份 .md 里一个 `### 日期` 标题都没有，这一格无对象可判')
    sys.exit(77)
print(f'  ✓ 变更史的形状对：查了 {len(files)} 份 kb 文件、{date_headings} 个 `### 日期` 标题'
      f'（两份变更史里 {history_date_headings} 个），{scope}，新增的没有违例；{stock}')

PY
}
