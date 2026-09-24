#!/usr/bin/env bash
# gate-stage: 冻结层归属登记表：每层每个结构都有行、写「退出」的必是派生态、依据指得到已定分项
#
# 还 checks-owed.md C45（四层图里每个结构的态别没有登记）可机检的那一半。
# 三条判据的权威原文在那一行的「怎么拦」列，清单从哪来写在
# `.claude/kb/freeze-layer-membership.md`「门禁判哪三条」，这一道照它判：
#   ① 表里要有行的对象，三份清单都现读、不在脚本里抄第二份：
#      - 层与独立冻结组件：D15 已定项 7 正文里 `第 N 层：` 的代码块与 `- ①/②/③ **…**` 的列表；
#        一棵树在其中两层各占一个结构——写着「key 编码」的那一层与写着「索引节点内部布局」的那一层，
#        两层的号从代码块里认，不写死；
#      - 树：`crates/singlefs-format/src/lib.rs` 的 `TREE_IDENTIFIER_*` 常量（`WATERMARK` 两个是水位、不是树）；
#      - 单元类：D18 已定项 11 登记表里首列是单个整数、类名不是「无效」「保留」的那几行。
#      每一层、每个组件至少一行；每棵树在 key 编码那层与节点内部布局那层各至少一行（结构列写着「树 ID <n>」）；
#      每个单元类码至少一行（结构列写着「码 <n>」）。漏一个判红。三份清单任一份读成 0 项也判红——扫到 0 项不是通过。
#   ② 「退不退出冻结」写「退出」的行，态别必须是「派生态」，别的都判红。
#   ③ 依据列点名的 `D<n>（简称） 已定项 k` 要在 `.claude/kb/decisions/` 里有那条决策、有第 k 条分项、
#      且索引表里是已定；归属与状态用 22 号那份 `lib-item-ref-status.py`，不另抄一份。
#      依据列写「判不动」开头的行不判 ③。
#   表按 `<!-- freeze-layer-membership:table -->` 那一行定位（整行匹配，不用 in），表头五列逐字；
#   层列只认 D15 已定项 7 读出的那几个值加「没有条款」，态别列只认 权威态 / 派生态 / 判不动，
#   退出列只认 退出 / 不退出 / 判不动——写了别的字判红，不然一个错别字会让 ② 静默放行。
#   退不退出冻结写「判不动」的行 ② 判不到，成功那句逐个列出；态别写「判不动」而退不退出冻结已定的行另列一句，
#   两份清单都从表里现算，不无声跳过。
#
# 门禁管不了的：某一行的态别判得对不对（例：把记账树码 2 节点那行改成「派生态 + 退出」，三条都不红——
# ② 只判「退出 ⇒ 派生态」，这一行两格自洽，错在态别本身）、判不动的理由站不站得住、有没有一个仓里存在而
# 三份清单都没单列的结构（树表单元、实例表单元、容器索引三行不是任何一份清单里的独立一项，删掉哪一行这一道都不红）。
#
# 判别力：fixtures/16-freeze-layer-membership.sh/red 的表漏了一棵树的 key 编码行、漏了组件 ② 与码 16 那两行、
# 一行写「权威态 + 退出」、依据指到不存在的分项（一处接在「已定项 5 / 98」后面）、指到未定项、指到不存在的决策
# （简称里套了一层括号）各有一处，必须判红；green 是真表原样的一份，必须判绿。
#
#   bash .claude/gate.d/16-freeze-layer-membership.sh [项目根]
set -uo pipefail
# LIB 要在 cd 之前算：$0 多半是相对路径，cd 进项目根之后就指不到了（样本在临时目录里跑时当场 FileNotFoundError）。
LIB="$(cd "$(dirname "$0")" && pwd)/lib-item-ref-status.py"
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
REGISTRY=.claude/kb/freeze-layer-membership.md
TREES=crates/singlefs-format/src/lib.rs
SELF=.claude/gate.d/16-freeze-layer-membership.sh
MARKER='<!-- freeze-layer-membership:table -->'

layers_decision="$(compgen -G '.claude/kb/decisions/15-*.md' || true)"
units_decision="$(compgen -G '.claude/kb/decisions/18-*.md' || true)"
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
    # 表在第一个非 | 行就结束，空行也算结束：下一张表只隔一个空行时，不这样切会把它并进来
    # （样本实测：偏移表的「| 42 | 单元类型标签 |」被当成了单元类码 42）。
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
item_map, _names = item_ref_status.load_map()
basis_checked = 0
for line_number, (structure, _layer, _state, _exits, basis) in rows:
    short = strip_trailing_parenthetical(structure)
    # 简称里可以再套一层全角括号（D14（双轨（大小文件 / 持久临时）） 那种），[^）]* 会在里层的 ） 处断开、整处引用漏抓；
    # 一处引用可以接着写几个项号（「已定项 3 / 4」「已定项 3 / 已定项 4」），只取第一个会让后面的无声漏判。
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
      echo "  ✗ 判据要读的清单有 ${#hits[@]} 份读不出来，这一道什么也没在判："   # gate-lint:summary
      printf '      %s\n' "${hits[@]}"   # gate-lint:detail
      echo "    → 怎么办：三份清单是 D15 已定项 7 正文（代码块里「第 N 层：」与列表里「- ① **…**」）、"
      echo "      crates/singlefs-format/src/lib.rs 的 TREE_IDENTIFIER_* 常量、D18 已定项 11 之下首列是「码」的登记表。"
      echo "      哪一份的形态改了，就改这一道去跟上它，别在脚本里抄一份清单——抄了就与条文分叉。"
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
      echo "      推不出就写「判不动」并在表下「判不动的九行，各卡在哪一句」里写清卡在哪一句；"
      echo "      树的一行结构列要写「树 ID <n>」，单元类的一行要写「码 <n>」，这一道按这两个字样认行。"
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
      echo "      分项还没定就不能拿它当依据——这一行的态别改成「判不动」，依据写「判不动，推导见「判不动的九行，各卡在哪一句」」。"
      ;;
  esac
done
((failed)) && exit 1

read -r _ row_count layer_count component_count tree_count unit_count basis_count skipped_count state_only_count < <(grep '^COUNT' <<<"$report")
mapfile -t skipped < <(awk -F'\t' '$1 == "SKIP" { print $2 }' <<<"$report")
mapfile -t state_only < <(awk -F'\t' '$1 == "STATEONLY" { print $2 }' <<<"$report")
echo "  ✓ 冻结层归属登记表判过了（${row_count} 行：D15 已定项 7 的 ${layer_count} 层与 ${component_count} 个组件、lib.rs 的 ${tree_count} 棵树、D18 已定项 11 的 ${unit_count} 个单元类都有行；依据点名的 ${basis_count} 处分项都在且已定）"
if ((skipped_count)); then
  echo "    ② 没判的 ${skipped_count} 行（退不退出冻结写「判不动」）：$(IFS='；'; echo "${skipped[*]}")"
else
  echo "    ② 每一行都判了，没有退不退出冻结写「判不动」的行"
fi
if ((state_only_count)); then
  echo "    另有 ${state_only_count} 行态别写「判不动」而退不退出冻结已定（括号里是退出列）：$(IFS='；'; echo "${state_only[*]}")"
fi
exit 0
