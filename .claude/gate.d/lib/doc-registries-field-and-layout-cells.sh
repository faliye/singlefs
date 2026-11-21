#!/usr/bin/env bash
# 门禁 doc-registries 的格函数库：field-refs、field-projection、field-table-sums、first-txn-hooks、admission-terms、segment-registry、format-const-placeholders、second-txn-hooks、third-txn-hooks、tree-table-reserve、field-widths。
# 由 .claude/gate.d/doc-registries.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释；读的 ROOT 与入口里设好的各变量、bad、ok、howto 在调用时才取。

# ════════════════ 字段与布局组的十三格（字段与布局登记） ════════════════

# ── 格 field-refs：新池新建文件的字段表，每个「指向」都指到一条真实存在的分项
# 判据：kb/layout/01-first-txn.md 每行「指向」列里的「D<n>（简称） 已定项 / 未定项 <k>」（简称可省，也可套一层括号），
# 要在那条决策「### 已定项」/「### 未定项」的索引（第一个 #### 之前）里有那一行。
# 「无分项」不当引用、这一半不判；指向空着的行归下面「指向列空着」那一半判（C66）。表或 decisions/ 不在、一处引用都没有，退 77。
# 另判一半（day-1 预留字段的归属）：.claude/kb/layout/*.md 里字段名以「算法类型」起头、指向写「D9（加密） 已定项 12 第 N 项」的行，
# N 要与这一行所在的表所属的单元类一致。单元类按表的节标题、再加表前那一段（空行隔开的最近一块）判：
#   · 点名了 D9 已定项 12 的表里「自证单元（…）」括号列的某一个（根记录、journal 记录头、系统配置槽；名字现从那张表读，门禁里不抄）
#     ⇒ 自证单元自己身上的算法类型，N 要等于那张表里写着自证单元又写着算法类型的那一行的项号；
#   · 否则点名了指针、extent、节点 ⇒ 数据 / 元数据写的算法类型，N 要等于那张表里写着算法类型而没提自证单元的那一行的项号；
#   · 两样都没点名的行报出来（「!」行）、不判红；同一层里两样都点名，自证单元优先（节标题先判，不够再并上表前那一段）。
# 成功那句多报「核了 K 行预留字段的指向」（K 不含判不出的行）。那张表读不出、而又有行要判时判红，不悄悄当没有。
# 样本：field-refs-reservation-owner-red 里 journal 记录头表的算法类型行写第 4 项（该是第 1 项）判红、点名那一行，指针头表与根记录表各指对不判，
# 第四张表没点名单元类、只报不判；field-refs-reservation-owner-green 三处各指对，报「核了 3 行」。
# 弄坏开关 DOC_REGISTRIES_BREAK=reservation-owner-unchecked：这一半不判。
# 样本：field-refs-red 指向索引里没有的 D322 已定项 7、简称套了括号的 D323 已定项 3、只在论证编号列表里出现的 D322 已定项 5；
# field-registry-green 表格式与列表式两种索引、未定项与「无分项」都放行。
# 「指向」与预留字段的已定项表按编号取哪一份：lib-item-ref-status.py 的 numbered_files（中文的布局表取中文原文，译文的取同一种语言的那一份）。
# 样本：field-refs-translation-green（译文排在前面、中文原文有那几条，判绿）、field-refs-translation-red（只有译文列着那一条，判红）；弄坏开关 KB_NUMBERED_FILE_BREAK=first-match。
cell_field_refs() {
  local KB=.claude/kb
  local TABLE="$KB/layout/01-first-txn.md"
  [[ -f "$TABLE" ]] || { echo "  ! 没有 $TABLE，这一格无对象可判"; exit 77; }
  [[ -d "$KB/decisions" ]] || { echo "  ! 没有 $KB/decisions，这一格无对象可判"; exit 77; }

  local bad=0 checked=0 lineno dnum kind item file section

  # 编号 → 决策文件只走 lib-item-ref-status.py 的 numbered_files：这张表是中文，取中文原文 <号>-<名>.md。
  # 按 ls 排序取第一个命中会取到排在前面的译文 <号>-<名>.en.md，所以不这样取。
  local resolved_files resolved_number resolved_file
  local -A decision_file_by_number=()
  if ! resolved_files="$(python3 - "$ITEM_REF_STATUS_LIBRARY" "$KB/decisions" "$TABLE" 2>&1 <<'PY'
import importlib.util, os, re, sys
spec = importlib.util.spec_from_file_location("item_ref_status", sys.argv[1])
lib = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib)
directory, referrer = sys.argv[2], sys.argv[3]
numbers = sorted({int(hit.group(1)) for hit in (re.match(r"(\d+)-", name) for name in os.listdir(directory)) if hit})
for number in numbers:
    found = lib.numbered_files(directory, number, referrer)
    if found:
        print(number, os.path.basename(found[0]), sep="\t")
PY
  )"; then
    printf '%s\n' "$resolved_files" | sed 's/^/      /'   # gate-lint:detail
    echo "  ✗ 按编号取决策文件的那段 python 没跑成，一处指向都没判"
    echo "     → 怎么办： 按上面的 traceback 修（判法在 $ITEM_REF_STATUS_LIBRARY 的 numbered_files）；没跑成就是没判，不许当成判过了。"
    exit 1
  fi
  while IFS=$'\t' read -r resolved_number resolved_file; do
    [[ -n "$resolved_number" ]] && decision_file_by_number[$resolved_number]="$resolved_file"
  done <<<"$resolved_files"

  while IFS=$'\t' read -r lineno dnum kind item; do
    [[ -z "${dnum:-}" ]] && continue
    checked=$((checked + 1))

    file="${decision_file_by_number[$((10#$dnum))]:-}"
    if [[ -z "$file" ]]; then
      echo "  ✗ $TABLE:$lineno 指向 D$dnum，而 decisions/ 里没有这条决策"   # gate-lint:detail
      bad=$((bad + 1))
      continue
    fi

    # 只看「### 已定项」/「### 未定项」之后到下一个任意级标题之前的索引：`#### 已定项 N` 之下的论证与「## 历史版本」里另有编号列表和表格，那些不是分项
    section="$(awk -v want="$kind" '
      /^### 已定项[[:space:]]*$/ { cur="已定项"; next }
      /^### 未定项[[:space:]]*$/ { cur="未定项"; next }
      /^#+ / { cur="" }
      cur == want { print }
    ' "$KB/decisions/$file")"

    # 分项行认表格行 `| k | …` 与编号列表 `k. **…**` 两种写法
    if ! grep -qE "^(\| *${item} *\||${item}\. )" <<<"$section"; then
      echo "  ✗ $TABLE:$lineno 指向 D$dnum（$file）的「$kind $item」，而那张表里没有这一条"   # gate-lint:detail
      bad=$((bad + 1))
    fi
  done < <(
    grep -n '指向\|已定项\|未定项' "$TABLE" 2>/dev/null \
      | grep -oE '^[0-9]+:.*' \
      | while IFS= read -r line; do
          n="${line%%:*}"
          body="${line#*:}"
          # 一行里可能有多处引用，逐个抠出来：简称可以不写（「D22 已定项 99」），也可以里面再套一层括号
          # （「D14（双轨（大小文件 / 持久临时）） 已定项 9」）；编号左边不许紧挨字母数字
          grep -oE '(^|[^A-Za-z0-9])D[0-9]+(（([^（）]|（[^（）]*）)*）)? *(已定项|未定项) *[0-9]+' <<<"$body" \
            | grep -oE 'D[0-9]+.*' \
            | while IFS= read -r ref; do
                d="$(grep -oE '^D[0-9]+' <<<"$ref" | tr -d 'D')"
                k="$(grep -oE '(已定项|未定项)' <<<"$ref" | head -1)"
                i="$(grep -oE '[0-9]+$' <<<"$ref")"
                printf '%s\t%s\t%s\t%s\n' "$n" "$d" "$k" "$i"
              done
        done
  )

  # 预留字段的归属：python 的输出末行是 COUNT<tab>判了几行<tab>判不出几行<tab>红几行，其余是红行与「!」行
  local owner_report owner_rc owner_checked=0 owner_unknown=0 owner_bad=0 owner_summary_line
  if [[ "$BREAK_SWITCH" == "reservation-owner-unchecked" ]]; then
    owner_report=$'COUNT\t0\t0\t0'
    owner_rc=0
  else
    owner_report="$(python3 - "$KB" "$ITEM_REF_STATUS_LIBRARY" 2>&1 <<'PY'
import glob, importlib.util, os, re, sys

KB = sys.argv[1]
spec = importlib.util.spec_from_file_location("item_ref_status", sys.argv[2])
item_ref_status = importlib.util.module_from_spec(spec); spec.loader.exec_module(item_ref_status)
OWNER_DECISION, OWNER_ITEM = 9, 12     # D9（加密） 已定项 12：day-1 预留四样
POINTER_WORDS = ("指针", "extent", "节点")   # 第 4 项的射程：数据 / 元数据写（指针头表、extent、节点）的表

# ── 先读已定项 12 的表：谁是自证单元、各归第几项。读哪一份按引用方取（numbered_files：中文的布局表读中文原文，
#    译文读同一种语言的那一份）；取 glob 排序的第一个会读到 09-encryption.en.md，那张表一行都读不出
owner_tables = {}


def read_owner_table(path):
    names, owner_self, owner_data = [], None, None
    if path is None:
        return names, owner_self, owner_data
    lines = open(path, encoding="utf-8").read().split("\n## 历史版本")[0].splitlines()
    inside = False
    for l in lines:
        if re.match(rf"#{{3,6}}\s*已定项\s*{OWNER_ITEM}\b", l):
            inside = True
            continue
        if inside and re.match(r"#{2,6}\s", l):
            break
        if inside and l.startswith("|"):
            cells = [c.strip() for c in l.strip().strip("|").split("|")]
            if len(cells) < 2 or not re.fullmatch(r"\d+", cells[0]):
                continue
            field = cells[1]
            if "自证单元" in l and "算法类型" in field:
                owner_self = int(cells[0])
                m = re.search(r"自证单元（([^）]+)）", l)
                if m:
                    names = [n.strip() for n in m.group(1).split("、") if n.strip()]
            elif "算法类型" in field:
                owner_data = int(cells[0])
    return names, owner_self, owner_data


def owner_table_for(referrer):
    found = item_ref_status.numbered_files(f"{KB}/decisions", OWNER_DECISION, referrer)
    key = found[0] if found else None
    if key not in owner_tables:
        owner_tables[key] = read_owner_table(key)
    return owner_tables[key]


REFERENCE = re.compile(rf"D{OWNER_DECISION}(?:（(?:[^（）]|（[^（）]*）)*）)? *已定项 *{OWNER_ITEM} *第 *(\d+) *项")


def names_unit(text):
    # 「journal 记录头」拆成 journal 与 记录 两个词，都在才算点名（表标题写「journal」、表前说明写「记录头字段表」是分开的）
    for name in self_proof_names:
        tokens = name.split()
        if len(tokens) > 1:
            tokens[-1] = tokens[-1].rstrip("头")
        if tokens and all(t in text for t in tokens):
            return name
    return None


def classify(heading, paragraph):
    for text in (heading, heading + "\n" + paragraph):
        unit = names_unit(text)
        if unit:
            return "self", unit
        if any(w in text for w in POINTER_WORDS):
            return "data", next(w for w in POINTER_WORDS if w in text)
    return None, None


bad, unknown, checked = [], [], 0
for path in sorted(glob.glob(f"{KB}/layout/*.md")):
    self_proof_names, owner_self, owner_data = owner_table_for(path)
    body = open(path, encoding="utf-8").read().split("\n## 历史版本")[0].splitlines()
    heading, current, last, in_table, context = "", [], [], False, None
    for number, l in enumerate(body, 1):
        if re.match(r"#{1,6}\s", l):
            heading, current, last, in_table = l.lstrip("# ").strip(), [], [], False
            continue
        if not l.startswith("|"):
            in_table = False
            if not l.strip():
                if current:
                    last, current = current, []
            else:
                current.append(l)
            continue
        if not in_table:
            in_table = True
            context = (heading, "\n".join(current or last))
            current = last = []
        cells = [c.strip() for c in l.strip().strip("|").split("|")]
        if not cells or not re.sub(r"[*`]", "", cells[0]).startswith("算法类型"):
            continue
        for want in REFERENCE.findall("|".join(cells[1:])):
            where = f"{path}:{number}"
            kind, why = classify(*context)
            if kind is None:
                unknown.append(f"{where} 写「D{OWNER_DECISION} 已定项 {OWNER_ITEM} 第 {want} 项」，而它所在的表（节标题「{context[0]}」）"
                               "没点名自证单元、也不是指针 / extent / 节点的表，判不出该是哪一项，没判")
                continue
            checked += 1
            if kind == "self":
                should, tail = owner_self, f"表里点名了自证单元「{why}」，自证单元自己身上的算法类型归第 {owner_self} 项"
            else:
                should, tail = owner_data, f"表是「{why}」这类数据 / 元数据写的表，每个写的算法类型字段归第 {owner_data} 项"
            if should is None:
                bad.append(f"{where} 读不出 D{OWNER_DECISION} 已定项 {OWNER_ITEM} 定案表里{'自证单元' if kind == 'self' else '数据 / 元数据写'}那一行，"
                           "没法判这一行该指第几项")
            elif int(want) != should:
                bad.append(f"{where} 算法类型写「D{OWNER_DECISION} 已定项 {OWNER_ITEM} 第 {want} 项」，该是第 {should} 项（节标题「{context[0]}」）：{tail}")

for b in bad:
    print(f"  ✗ {b}")          # gate-lint:detail
for u in unknown:
    print(f"  ! {u}")
print(f"COUNT\t{checked}\t{len(unknown)}\t{len(bad)}")
sys.exit(1 if bad else 0)
PY
)"
    owner_rc=$?
  fi
  owner_summary_line="$(tail -n 1 <<<"$owner_report")"
  if [[ ( $owner_rc -ne 0 && $owner_rc -ne 1 ) || "$owner_summary_line" != COUNT$'\t'* ]]; then
    printf '%s\n' "$owner_report" | sed 's/^/  /'
    echo "  ✗ 判预留字段归属的那段 python 没跑成（退出码 $owner_rc）——这一半没判"
    echo "     → 怎么办： 按上面的 traceback 修；没跑成就是没判，不许当成判过了。"
    exit 1
  fi
  IFS=$'\t' read -r _ owner_checked owner_unknown owner_bad <<<"$owner_summary_line"
  head -n -1 <<<"$owner_report"   # gate-lint:detail
  bad=$((bad + owner_bad))

  # 指向列空着（「无分项」或什么都没写）的行、根槽那一节（「## 七、」）指向未定项的行：C66（格式级空白没有分项号） 要的收紧。
  # 表头有「指向」一列的每张表都判；「## 历史版本」之后不判。格式级空白要留的，带理由登记进 .claude/gate.d/field-pointer-blanks.tsv
  # （一行「<那一行里的一段原文><TAB># 理由>」，理由至少 8 个字，在指向空着的行里要恰好命中 1 行）；根槽那一节不认豁免。
  # 前一行（中间可隔空行）是 <!-- unit-header-fields: … --> 标记的单元头字段表（unit-header-fields 格比的那几张）在哪一节都照根槽那一节办：不许指未定项、不认豁免。
  # 弄坏开关 DOC_REGISTRIES_BREAK=unit-header-pointer-loose：带标记的表不照根槽那一节办；样本 field-refs-unit-header-red。
  # 输出末行 COUNT<tab>判了几行<tab>豁免几行<tab>红几处。弄坏开关 DOC_REGISTRIES_BREAK=pointer-blanks-unchecked：这一半不判。
  # 样本：field-refs-blank-red（无分项与空着的各一行没豁免、根槽一行指未定项、根槽一行无分项登了豁免也红、一条豁免一行都不命中）；
  #   field-registry-green 那一行「无分项」登了豁免，判绿。
  local blank_report blank_rc blank_summary blank_checked=0 blank_exempted=0 blank_bad=0
  if [[ "$BREAK_SWITCH" == "pointer-blanks-unchecked" ]]; then
    blank_report=$'COUNT\t0\t0\t0'
    blank_rc=0
  else
    blank_report="$(python3 - "$TABLE" .claude/gate.d/field-pointer-blanks.tsv "$BREAK_SWITCH" 2>&1 <<'PY'
import os, re, sys

TABLE, EXEMPT = sys.argv[1:3]
BREAK = sys.argv[3] if len(sys.argv) > 3 else ""
UNIT_HEADER_MARKER = re.compile(r"^<!--\s*unit-header-fields:\s*(\S+)")
split_cells = lambda line: [cell.strip() for cell in re.split(r"(?<!\\)\|", line.strip().strip("|"))]
BLANK = {"", "无分项"}
exempt, bad = {}, []
if os.path.isfile(EXEMPT):
    for number, line in enumerate(open(EXEMPT, encoding="utf-8"), 1):
        if not line.strip() or line.startswith("#"):
            continue
        fragment, _, reason = line.rstrip("\n").partition("\t")
        reason = reason.lstrip("# ").strip()
        if not fragment.strip() or len(reason) < 8:
            bad.append(f"{EXEMPT}:{number} 要写「<那一行里的一段原文><TAB># 理由>」，理由至少 8 个字")
            continue
        exempt[fragment.strip()] = [number, 0]
section, header, checked, exempted, pending_unit, unit_table = "", None, 0, 0, None, None
for number, line in enumerate(open(TABLE, encoding="utf-8").read().split("\n## 历史版本")[0].splitlines(), 1):
    if line.startswith("## "):
        section = line[3:].strip()
    marker = UNIT_HEADER_MARKER.match(line.strip())
    if marker:
        pending_unit = marker.group(1)
    if not line.startswith("|"):
        header = None
        if line.strip() and not marker:
            pending_unit = None
        continue
    cells = split_cells(line)
    if header is None:
        header = cells
        unit_table, pending_unit = (None if BREAK == "unit-header-pointer-loose" else pending_unit), None
        continue
    if set("".join(cells)) <= set("-: ") or "指向" not in header:
        continue
    pointer = cells[header.index("指向")] if len(cells) > header.index("指向") else ""
    checked += 1
    where = f"{TABLE}:{number}「{re.sub(r'[*`]', '', cells[0])[:30]}」"
    root_slot = section.startswith("七、")
    if root_slot and re.search(r"未定项", pointer):
        bad.append(f"{where} 在根槽那一节（{section[:12]}）指向未定项「{pointer[:40]}」：根槽段的字段要指到已定项")
    elif unit_table and re.search(r"未定项", pointer):
        bad.append(f"{where} 在单元头字段表（{unit_table}）指向未定项「{pointer[:40]}」：单元头的字段要指到已定项")
    if pointer.strip() not in BLANK:
        continue
    hits = [fragment for fragment in exempt if fragment in line]
    for fragment in hits:
        exempt[fragment][1] += 1
    if root_slot:
        bad.append(f"{where} 在根槽那一节（{section[:12]}）的指向写「{pointer or '（空）'}」：根槽段的字段不许留格式级空白，豁免表也不认")
    elif unit_table:
        bad.append(f"{where} 在单元头字段表（{unit_table}）的指向写「{pointer or '（空）'}」：单元头的字段不许留格式级空白，豁免表也不认")
    elif hits:
        exempted += 1
    else:
        bad.append(f"{where} 的指向写「{pointer or '（空）'}」：格式级空白要么指到一条分项（没定就开未定项），要么带理由登记进 {EXEMPT}")
for fragment, (number, used) in exempt.items():
    if used != 1:
        bad.append(f"{EXEMPT}:{number}「{fragment}」在指向空着的行里命中 {used} 行（要恰好 1 行：0 行是那一行已经补了指向或改了字，删掉这条豁免；几行是原文太短，写长一点）")
for item in bad:
    print(f"  ✗ {item}")    # gate-lint:detail
print(f"COUNT\t{checked}\t{exempted}\t{len(bad)}")
sys.exit(1 if bad else 0)
PY
)"
    blank_rc=$?
  fi
  blank_summary="$(tail -n 1 <<<"$blank_report")"
  if [[ ( $blank_rc -ne 0 && $blank_rc -ne 1 ) || "$blank_summary" != COUNT$'\t'* ]]; then
    printf '%s\n' "$blank_report" | sed 's/^/  /'
    echo "  ✗ 判指向列空白的那段 python 没跑成（退出码 $blank_rc）——这一半没判"
    echo "     → 怎么办： 按上面的 traceback 修；没跑成就是没判，不许当成判过了。"
    exit 1
  fi
  IFS=$'\t' read -r _ blank_checked blank_exempted blank_bad <<<"$blank_summary"
  head -n -1 <<<"$blank_report"   # gate-lint:detail
  bad=$((bad + blank_bad))

  if ((bad)); then
    echo "  ✗ 新池新建文件的字段表里有 $bad 处指向有误（指向了不存在的分项、指向空着没登记豁免、根槽那一节指未定项，或预留字段的项号与所属单元类对不上）"
    echo "     → 怎么办： 每一行的「指向」列写一条真实存在的分项（编号改过就跟着改；没定的开一条未定项再指它）；"
    echo "                确要留的格式级空白（「无分项」）带理由登记进 .claude/gate.d/field-pointer-blanks.tsv，根槽那一节（## 七、）与带 unit-header-fields 标记的单元头字段表不许留、也不许指未定项。"
    echo "                分项现在叫什么，看 .claude/kb/decisions/ 里那条决策的两张索引表。"
    echo "                预留字段（算法类型）项号对不上的，改那一行的指向，或者这个字段真属于另一类就改表标题；"
    echo "                哪个单元归哪一项，看 .claude/kb/decisions/ 里 D9（加密） 已定项 12 的定案表。"
    exit 1
  fi

  if ((checked == 0)); then
    echo "  ! $TABLE 里一处「D<n>（简称） 已定项 / 未定项 k」引用都没有，这一格无对象可判"
    exit 77
  fi
  echo "  ✓ 新池新建文件的字段表指向都成立（查了 $checked 处引用）"
  echo "  ✓ 核了 $owner_checked 行预留字段的指向（另有 $owner_unknown 行判不出所属单元类，没判）"
  echo "  ✓ 指向列 $blank_checked 行都写了分项（登记豁免的格式级空白 $blank_exempted 行，豁免表 .claude/gate.d/field-pointer-blanks.tsv）"
}

# ── 格 field-projection：决策里定的字段有没有漏投影进新池新建文件的表
# field-refs 判正向（投影表每个字段指得到分项），这一格判反向：分项的字段表新加了一行，投影表没跟着加。
# 射程：
#   1. 只认表头恰好是 `| 字段 | 宽 |` 或 `| 字段 | 宽度 |` 的表，散在段落里的字段定义抓不到。
#   2. 只查 layout/01-first-txn.md 里引过（`D<n>（简称） 已定项 <k>`）的分项：新池新建文件不含快照、加密、目录树，
#      对没投影的分项要求投影会假红；代价是从没进过投影表的分项，这一格一个字也不说。
#   3. 字段名按子串比，只判在不在，不判宽度（宽度归 code-source-discipline 与 C94（登记的格式常量与后来的定案对不上））；history-kept: 欠账 C94 的登记名逐字引用。
#   一张被投影的字段表都没核到，退 77。
# 样本：field-projection-red 里 D362 已定项 7 的字段表三行，投影表只收了 magic，「块头版本」只在别的节的正文里提了一句；
# field-registry-green 里被投影的两行都在引了那条分项的那一节的表里。
cell_field_projection() {
  local FTL=.claude/kb/layout/01-first-txn.md
  [[ -f "$FTL" && -d .claude/kb/decisions ]] || { echo "  ! 找不到 $FTL 或 decisions/，这一格无对象可判"; exit 77; }

  python3 - "$FTL" <<'PY'
import re, sys, glob, os

ftl_path = sys.argv[1]
ftl = open(ftl_path, encoding="utf-8").read().split("\n## 历史版本")[0]

REFERENCE = r"D(\d+)（[^）]*）\s*已定项\s*(\d+)"
refs = {(int(a), int(b)) for a, b in re.findall(REFERENCE, ftl)}

# 字段名要落在引了那条分项的节（按 `## ` 切）的表格行里：整份文件按子串找，会让别的节的正文顶替（「树 ID 水位」在别处提过一句，那一节的表里却没有）
sections = []
for chunk in re.split(r"(?m)^(?=## )", ftl):
    chunk_refs = {(int(a), int(b)) for a, b in re.findall(REFERENCE, chunk)}
    table_rows = "\n".join(line for line in chunk.splitlines() if line.startswith("|"))
    sections.append((chunk_refs, table_rows))

def projected_table_rows(decision_item):
    return "\n".join(rows for chunk_refs, rows in sections if decision_item in chunk_refs)

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
            if name not in projected_table_rows((dnum, item)):
                bad.append(f"{p}:{i}  D{dnum} 已定项 {item} 的字段表有「{name}」（宽 {c[1]}），"
                           f"而 {ftl_path} 里引用了这条分项的那几节的表格行里找不到它")

if bad:
    print(f"  ✗ 分项定了的字段没投影进新池新建文件的表 {len(bad)} 处：")     # gate-lint:summary
    for b in bad:
        print("      " + b)                                              # gate-lint:detail
    print(f"  → 怎么办：把缺的那一行补进 {ftl_path} 对应的那一节（字段、宽度、指向哪条分项、状态），")
    print("    它是「新池新建文件写出哪些字节」的投影表，漏一行等于那几个字节没有任何人在看；")
    print("    若这个字段确实不属于新池新建文件（快照 / 加密 / 目录树），")
    print("    那就把它从这条分项的字段表里挪走，或把该分项从投影表的引用里去掉——两处口径必须一致。")
    sys.exit(1)

if checked_tables == 0:
    print(f"  ! 本次无对象可判：投影表引用了 {len(refs)} 条分项，其中没有一条的正文里有「| 字段 | 宽 |」字段表")
    sys.exit(77)
print(f"  ✓ 被投影的分项，字段表都投影全了（{checked_tables} 张字段表、{checked_rows} 行；"
      f"投影表引用了 {len(refs)} 条分项）")
PY
}

# ── 格 field-table-sums：字段表加出来的数（表后合计、表前说明句、format-const 标记）与字节布局表里结构总宽的登记
# 落的是 C94（登记的格式常量与后来的定案对不上）里能机械判的那几样（history-kept: 欠账 C94 的登记名逐字引用）。射程：
#   1. 只认表头里「字段」列紧跟「宽」或「宽度」列的表（前面可以有「段」这类列）；表后合计在表后 3 行内，写成「合计 N 字节」或「合计 **N** 字节」。
#   2. 宽度格按整数或只含 ×、+、− 的整数算式读；有一行读不出（「未定」「随实现定」「—」），整张表跳过并计入跳过数：加不出确定的和时判红只会是假红。
#   3. 只判总宽与各行之和、标记与分项之和、布局里的总宽有没有登记；各行宽度本身对不对不判。
#   4. 一条「已定项 N」里恰好一个 format-const 标记时，标记值要等于该分项下全部字段表之和；有多个标记时无从对应，跳过并报出。
#   5. 扫 .claude/kb/decisions/*.md 与 .claude/kb/layout/*.md 的正文（「## 历史版本」之前），只读中文原文；登记值也只从中文原文的标记取
#      （译文照抄标记、没跟上的留着旧值，取法 lib-item-ref-status.py 的 originals_only；弄坏开关 DOC_REGISTRIES_BREAK=read-translations 连译文一起读）。
#   6. 标记按 lib-format-const.py 读；以 `<!-- format-const` 开头而读不出的、同一份文件里同一个名字登记了不止一次的，判红。
#   7. 表前说明句只认表头前最多 3 行非空行里以 `**` 起头的标题行。增量式「+M 字节 ⇒ 共 N 字节」判 M = 这张表之和、
#      N = 这一小节（上一个标题起）到这张表为止全部字段表之和；绝对式「合计 / 共 N 字节」「N 字节初值」「（N 字节，」「定长 N」判 N = 这张表之和。
#   8. 字节布局表（.claude/kb/layout/*.md）的结构总宽只认三种写法：① 非表格行的「合计 N 字节」；② 以 ⇒ 起头的行里紧挨求和等式的加粗整数
#      （等式那一侧去掉括注后要有 +，只有 × 的「6 × 55 = **330**」是条数乘宽、不算）；③ 带「状态」列的表里状态以「已定」开头的行、宽度格里加粗的整数。
#      登记 = 同一行（① 还可以是它那张表的说明句）里有值等于 N 的 `<!-- format-const: 名字 = N -->`，或有「`format-const: 名字`」引用、
#      而那个名字在 kb 正文里登记的值等于 N；缺即判红。认不出的写法这一格一个字也不说，成功那句报认出了几处。
#   四样一处都没认出，退 77；表前表后都没写认得出的总宽的表，成功行下面逐个列表头行。
# 样本：field-table-sums-red 放合计写错的决策表与布局表、与表和对不上的标记、多写了键的标记、同一份文件登记两次的名字、
# 登记 78 而正文两笔增量共 13 字节（367-样例）、三句写错的表前说明句（369-样例）、布局里没登记的三种结构总宽、引用值不对与没登记的名字、
# `| 段 | 字段 | 宽 |` 带乘式的表合计写错（02-样例），判红；field-registry-green 放同一个增量样本而登记值改成 91（368-样例）、
# 三种写法各一处登记好的总宽、未定行里的加粗宽度与只有 × 的等式（不算），判绿；
# field-registry-translation-green 在它上面给 368-样例与 01-样例布局表各加英语、日语译文（首行与中文不同，标记照抄、英语那份没跟上留着旧值 78），判绿；
# 弄坏开关 DOC_REGISTRIES_BREAK=read-translations 下译文进来，同名标记登记了几处、旧值顶替登记值，判红。
cell_field_table_sums() {
  [[ -d .claude/kb/decisions ]] || { echo "  ! 找不到 decisions/，这一格无对象可判"; exit 77; }

  BREAK_SWITCH="$BREAK_SWITCH" python3 - "$FORMAT_CONST_LIBRARY" "$ITEM_REF_STATUS_LIBRARY" <<'PY'
import re, glob, importlib.util, os, sys

library_spec = importlib.util.spec_from_file_location("format_const", sys.argv[1])
format_const = importlib.util.module_from_spec(library_spec)
library_spec.loader.exec_module(format_const)
item_ref_status_spec = importlib.util.spec_from_file_location("item_ref_status", sys.argv[2])
item_ref_status = importlib.util.module_from_spec(item_ref_status_spec)
item_ref_status_spec.loader.exec_module(item_ref_status)
# 译文不读（射程第 5 条）：译文照抄了中文的标记，没跟上的译文留着旧值；表头是各自语言的「Field / Width」。弄坏开关 read-translations 连译文一起读
read_translations = os.environ.get("BREAK_SWITCH") == "read-translations"

SUM = re.compile(r"合计\s*\**\s*(\d+)\s*\**\s*字节")
HEAD = re.compile(r"^#{2,6}\s")
ITEM = re.compile(r"^#{3,6}\s*已定项\s*(\d+)")
# 表前说明句（射程第 7 条）：增量式先认，认上了就不再按绝对式判同一句
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


# 登记值：kb 正文里全部读得出来的标记（与 code-source-discipline 同一个取法：不含变更史）
kb_body_paths = [path for path in item_ref_status.originals_only(sorted(glob.glob(".claude/kb/**/*.md", recursive=True)), read_translations)
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


checked, skipped, tables_without_total, bad = 0, [], [], []
caption_checked = 0
mark_checked, mark_skipped = 0, []
marker_problems = []
layout_widths_checked, layout_unregistered = 0, []
decision_paths = item_ref_status.originals_only(sorted(glob.glob(".claude/kb/decisions/*.md")), read_translations)
layout_paths = item_ref_status.originals_only(sorted(glob.glob(".claude/kb/layout/*.md")), read_translations)

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
        # 分项里有标记而没比成的，一律进跳过清单点名：宽度不是数、或这一节没有字段表，都不许静默放过
        if item is not None and item_marks:
            if len(item_marks) != 1:
                mark_skipped.append(f"{os.path.basename(kb_path)} 已定项 {item} 里有 "
                                    f"{len(item_marks)} 个 format-const 标记，对不上哪张表")
            elif not item_ok:
                mark_skipped.append(f"{os.path.basename(kb_path)} 已定项 {item} 的 format-const 标记 {item_marks[0][0]} 没核："
                                    f"这一节有宽度不是数的字段表，加不出确定的和")
            elif not item_sum:
                mark_skipped.append(f"{os.path.basename(kb_path)} 已定项 {item} 的 format-const 标记 {item_marks[0][0]} 没核："
                                    f"这一节没有加得出和的字段表")
            else:
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
                tables_without_total.append(f"{os.path.basename(kb_path)}:{header_line_number}")
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
    print("    （`.claude/rules/evidence-discipline.md`：撤回一个数要当场回扫谁在引它）。")
if marker_problems:
    print(f"  ✗ format-const 标记读不出来或在同一份文件里重复登记 {len(marker_problems)} 处：")  # gate-lint:summary
    for problem in marker_problems:
        print("      " + problem)                                        # gate-lint:detail
    print("  → 怎么办：读不出来的照 <!-- format-const: 名字 = 整数 stale=旧串|旧串 --> 改写，stale= 之外不许有别的键；")
    print("    它只是正文里举的例子、不是登记，就去掉 <!--，写成「`format-const: 名字 = 值`」。")
    print("    重复的只留定这个值的那一处，别处要提它写成不带 <!-- 的文字（例如「`format-const: 名字`」）。")
    print("    标记的文法只有一份，在 .claude/gate.d/lib-format-const.py，code-source-discipline、checker-independence-and-sync 按同一份读。")
if layout_unregistered:
    print(f"  ✗ 字节布局表里写出的结构总宽没有登记 {len(layout_unregistered)} 处：")  # gate-lint:summary
    for entry in layout_unregistered:
        print("      " + entry)                                          # gate-lint:detail
    print("  → 怎么办：在同一行（「合计 N 字节」那句也可以写在它那张表的说明句里）登记这个宽度：")
    print("    这个量还没有登记位，就在定它的那一处加 <!-- format-const: 名字 = N -->；")
    print("    已经在某份决策里登记过，就在这一行写「`format-const: 名字`」引用它，门禁按登记值比。")
    print("    名字与 crates/singlefs-format 里的 const 同名时，code-source-discipline 会拿登记值去核那个 const，要写成整数字面量。")
    print("    这个数其实不是一个结构的总宽，就别用加粗或「合计 N 字节」写它（C94（登记的格式常量与后来的定案对不上））。")
if bad or marker_problems or layout_unregistered:
    sys.exit(1)

# 四样一样都没核到：什么都没比过，不记通过（射程第 8 条：认出 0 处不是「都登记了」）
if checked + caption_checked + mark_checked + layout_widths_checked == 0:
    print(f"  ! 本次无对象可判：扫了 {len(decision_paths)} 份决策、{len(layout_paths)} 份字节布局表，"
          f"表后合计、表前说明句、format-const 标记、结构总宽一处都没认出来")
    sys.exit(77)

message = (f"  ✓ 字段表加出来的数都对得上（扫了 {len(decision_paths)} 份决策、{len(layout_paths)} 份字节布局表；"
           f"表后合计 {checked} 张，表前说明句 {caption_checked} 张，format-const 标记 {mark_checked} 个；"
           f"字节布局表里的结构总宽 {layout_widths_checked} 处都有登记）")
if skipped:
    message += f"，跳过 {len(skipped)} 处（有非数字宽度）"
if tables_without_total:
    message += f"，另有 {len(tables_without_total)} 张表前表后都没写认得出的总宽、这一格没验它们（表头行逐个列在下面）"
print(message)
for entry in skipped + mark_skipped:
    print(f"     ! 跳过：{entry}")
for entry in tables_without_total:
    print(f"     没验：{entry} 那张字段表，表前表后都没写认得出的总宽")
PY
}

# ── 格 first-txn-hooks：新池新建文件的三份文件互相挂钩（字节表 / 里程碑 / 决策索引）
# 三份说的是同一件事：decisions/*.md 的每条未定项判过「改新池新建文件的字节：是 / 否」；layout/01-first-txn.md 列出每个字节，
# 判「是」的未定项在它的「还剩几处空白」表里各占一行；milestone/01-first-txn.md 把写这些字节的活拆成步，每步写「写出的字节」在哪几节、「会碰到的决策点」。
# 判据（每条都报绝对数，扫到 0 也报）：
#   1. 判「是」且仍未定的每一条 `D<n> 未定项 <k>`，字节表与里程碑各至少引一次。
#   2. 字节表「还剩几处空白」表里引到的未定项，必须都在第 1 条那个集合里（定了就要从表里拿走）。
#   3. 字节表每一节（历史版本与空白表除外）标题下紧跟一行 `**里程碑**：… 步 N`，N 是里程碑里存在的 `## 步 N`。
#   4. 里程碑每个 `## 步 N` 有一行 `**写出的字节**：`，要么写「无」，要么点名字节表里存在的节标题（「…」引起来的那段）：
#      要与某一节的标题逐字相同，只许省掉标题末尾那一段全角括注；只是标题的一截不算点名。被字节表某节标成归属的步，不许写「无」。
# 射程：只认「改新池新建文件的字节：**是**（日期…」这一种写法（括号是判据的一部分），写在未定项小节之外的散文抓不到；
#   引用形态只认 `D<n>（简称） 未定项 <k>`，不判引用处说的内容对不对。判「是」的未定项、字节表的节、里程碑的步三样一个都没有，退 77。
# 样本：first-txn-hooks-red 里判「是」的 D401 未定项 1 里程碑没引、空白表引着判「否」的未定项 2、一节没标步号、
# 被标成归属的步 1 写「无」、步 2 只点了标题的一截；field-registry-green 里三面都对得上。
cell_new_pool_file_creation_hooks() {
  local FTL=.claude/kb/layout/01-first-txn.md
  local MS=.claude/kb/milestone/01-first-txn.md
  local DEC=.claude/kb/decisions
  [[ -f "$FTL" && -f "$MS" && -d "$DEC" ]] || { echo "  ! 找不到 $FTL / $MS / $DEC，这一格无对象可判"; exit 77; }

  python3 - "$FTL" "$MS" "$DEC" <<'PY'
import re, sys, glob, os

ftl_path, ms_path, dec_dir = sys.argv[1:4]
ftl = open(ftl_path, encoding="utf-8").read().split("\n## 历史版本")[0]
ms = open(ms_path, encoding="utf-8").read().split("\n## 历史版本")[0]
REF = re.compile(r"D(\d+)（[^）]*）\s*未定项\s*(\d+)")
# 只认带日期括号的规范形态；「……判过「改新池新建文件的字节：是」」这种转述引用不算
YES = re.compile(r"改新池新建文件的字节：\**是\**（")

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
        bad.append(f"D{d} 未定项 {k} 判了「改新池新建文件的字节：是」，而 {ftl_path} 一处都没引它")
    if (d, k) not in in_ms:
        bad.append(f"D{d} 未定项 {k} 判了「改新池新建文件的字节：是」，而 {ms_path} 一处都没引它")

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
    if "layout/01-first-txn.md" not in row or not quoted:
        bad.append(f"{ms_path}:{i + 1} 步 {n} 的「写出的字节」既不是「无」，也没点名 layout/01-first-txn.md 里的节")
        continue
    for q in quoted:
        # 逐字等于某一节的标题，或只省掉了标题末尾的全角括注；标题的一截（「根」之于「七、发布（根记录与根槽）」）不算
        if not any(h == q or (h.startswith(q) and h[len(q):].startswith("（") and h.endswith("）")) for h in headings):
            bad.append(f"{ms_path}:{i + 1} 步 {n} 点名「{q}」，而 {ftl_path} 没有标题是它的节（只许省掉标题末尾的全角括注）")

if bad:
    print(f"  ✗ 新池新建文件的三份文件对不上 {len(bad)} 处：")                    # gate-lint:summary
    for b in bad:
        print("      " + b)                                                     # gate-lint:detail
    print(f"  → 怎么办：判「是」的未定项要同时进 {ftl_path} 的「还剩几处空白」表和 {ms_path} 某一步的「会碰到的决策点」；")
    print("    定了案的就从空白表里拿走；字节表每节标题下写「**里程碑**：… 步 N」，里程碑每步写「**写出的字节**：无」或点名字节表的节标题。")
    sys.exit(1)

if not blocking and not sections and not steps and not gap_refs:
    print(f"  ! 本次无对象可判：{dec_dir} 里没有判「是」的未定项，{ftl_path} 没有节，{ms_path} 没有步，三样一样都没比")
    sys.exit(77)
print(f"  ✓ 新池新建文件的三份文件互相挂钩（判「是」的未定项 {len(blocking)} 条、字节表 {sections} 节、里程碑 {len(steps)} 步）")
PY
}

# ── 格 admission-terms：准入不等式的每一项都有人维护（统计量，或写明的例外）
# 判据在 D5（快照 / 空间记账机制） 已定项 4：式子里出现的每一项都是被维护的统计量，做不到就写明例外是谁。
#   1. 式子的权威正文前面有一行 `<!-- gate:admission-formula -->`，全仓只许一处；取它下面第一个代码块里 `可用 =`、`可用(d) =` 或 `可用（d） =`（半角、全角括号都认）那一行，
#      按 `−` 拆成项（去掉 `Σ设备(` 与括号）。
#   2. 对照表前面有一行 `<!-- gate:admission-terms -->`，全仓只许一处；取它下面第一张表的两列。
#   3. 两边的项逐一相等：式子多一项或对照表多一项，都判红。
#   4. 对照表每一行的右列要么是「第 N 项（名字）」，N 在同一文件那张 `| # | 统计量 |` 表里、名字逐字相等、且不是已撤回的编号；
#      要么以 `**例外**：` 开头并写出理由（不少于 8 个字）。
#   5. `decisions/` 下凡是写着 `可用 = Σ设备` 的地方（索引行、挪走后留的指针），去掉空白与反引号之后必须与权威那一行逐字相等：
#      副本在反引号里的，取到收尾那个反引号；在表格行里、没有反引号的，取到那一格的竖线；都不是的取到行尾（末尾的句号、分号不算）。
#   两个锚不是各恰好一处（含一处都没有）判红、不退 77：锚没了就是式子与对照表脱了钩。decisions/ 不在，退 77。
#   6. 只读 decisions/ 下的中文原文：译文照抄两个锚、式子与对照表，首行与表头是各自的语言，不算第二处（取法 lib-item-ref-status.py 的 originals_only）。
# 射程：只判每一项有归属、归属指得到，不判归属对不对（该不该是统计量、例外的理由成不成立，靠人）；
#   不判统计量表里多出来的项（判定不止准入一处，机器列不全）。
# 样本：admission-terms-red 的式子多了「挂载期承诺量」而对照表没有、副本在权威那一行后面多写一项；field-registry-green 三项各有归属、副本逐字一致；
#   admission-terms-fullwidth-d-green 的式子写成 `可用（d） = …`（全角括号），判绿；
#   field-registry-translation-green 是 field-registry-green 加上准入、树表、字段表那几份决策的英语与日语译文（首行写 `— Settled` / `—— 確定`，照抄两个锚、式子与对照表），
#   只跑 field-table-sums、admission-terms、tree-table-reserve 三格，判绿；弄坏开关 DOC_REGISTRIES_BREAK=read-translations 下锚各三处、判红。
cell_admission_terms() {
  [[ -d .claude/kb/decisions ]] || { echo "  ! 找不到 .claude/kb/decisions/，这一格无对象可判"; exit 77; }

  BREAK_SWITCH="$BREAK_SWITCH" ITEM_REF_STATUS_LIBRARY="$ITEM_REF_STATUS_LIBRARY" python3 - <<'PY'
import glob, importlib.util, os, re, sys

item_ref_status_spec = importlib.util.spec_from_file_location('item_ref_status', os.environ['ITEM_REF_STATUS_LIBRARY'])
item_ref_status = importlib.util.module_from_spec(item_ref_status_spec)
item_ref_status_spec.loader.exec_module(item_ref_status)
# 只读中文原文（第 6 条）：译文照抄两个锚与式子，读进来锚就成了三处。弄坏开关 read-translations 连译文一起读
files = item_ref_status.originals_only(sorted(glob.glob('.claude/kb/decisions/*.md')),
                                       os.environ.get('BREAK_SWITCH') == 'read-translations')
texts = {path: open(path, encoding='utf-8').read() for path in files}

def anchors(marker):
    hits = []
    for path, text in texts.items():
        lines = text.split('\n')
        for number, line in enumerate(lines):
            if line.strip() == marker:
                hits.append((path, number, lines))
    return hits

problems = []

formula_hits = anchors('<!-- gate:admission-formula -->')
terms_hits = anchors('<!-- gate:admission-terms -->')
if len(formula_hits) != 1 or len(terms_hits) != 1:
    print(f'  ✗ 准入式子的锚 {len(formula_hits)} 处、对照表的锚 {len(terms_hits)} 处，各要恰好 1 处')
    print('     → 在准入不等式权威正文的代码块上一行写 `<!-- gate:admission-formula -->`，')
    print('       在逐项对照表上一行写 `<!-- gate:admission-terms -->`；两个锚全仓各一处，别处只留指针。')
    sys.exit(1)

formula_path, formula_line, formula_lines = formula_hits[0]
formula_text = None
inside_block = False
# 弄坏开关 DOC_REGISTRIES_BREAK=formula-fullwidth-d-unmatched：抓式子那一行只认半角 (d)
formula_line_pattern = r'^可用(\(d\))?\s*=' if os.environ.get('BREAK_SWITCH') == 'formula-fullwidth-d-unmatched' else r'^可用(\(d\)|（d）)?\s*='
for line in formula_lines[formula_line + 1:]:
    if line.startswith('```'):
        if inside_block:
            break
        inside_block = True
        continue
    if inside_block and re.match(formula_line_pattern, line.strip()):
        formula_text = line.strip()
        break
if formula_text is None:
    print(f'  ✗ {formula_path}:{formula_line + 1} 的锚下面没有代码块，或代码块里没有 `可用 =` 那一行')
    print('     → 锚要紧挨着式子的代码块，式子写成一行：可用 = Σ设备( … ) − … 。')
    sys.exit(1)

right_hand_side = formula_text.split('=', 1)[1]
right_hand_side = right_hand_side.replace('(d)', '').replace('（d）', '')
right_hand_side = right_hand_side.replace('Σ设备(', ' ').replace('Σ设备（', ' ').replace('(', ' ').replace(')', ' ').replace('（', ' ').replace('）', ' ')
formula_terms = [re.sub(r'\s*÷\s*副本数$', '', term.strip()) for term in right_hand_side.split('−') if term.strip()]

terms_path, terms_line, terms_lines = terms_hits[0]
mapping = {}
table_started = False
for line in terms_lines[terms_line + 1:]:
    stripped = line.strip()
    if not stripped:
        if table_started:
            break
        continue
    if not stripped.startswith('|'):
        if table_started:
            break
        continue
    table_started = True
    cells = [cell.strip() for cell in stripped.strip('|').split('|')]
    if len(cells) < 2 or set(cells[0]) <= set('-: ') or cells[0] == '式子里的项':
        continue
    mapping[cells[0]] = cells[1]

statistics = {}
statistics_text = texts[terms_path].split('\n')
in_statistics_table = False
for line in statistics_text:
    if re.match(r'^\| # \| 统计量 \|', line):
        in_statistics_table = True
        continue
    if in_statistics_table:
        if not line.startswith('|'):
            in_statistics_table = False
            continue
        cells = [cell.strip() for cell in line.strip().strip('|').split('|')]
        if cells and cells[0].isdigit():
            statistics[int(cells[0])] = cells[1].replace('**', '')

if not statistics:
    problems.append(f'{terms_path}: 找不到表头是 `| # | 统计量 |` 的统计量表，对照表的「第 N 项」没处可指')

formula_set, mapping_set = set(formula_terms), set(mapping)
for term in formula_terms:
    if term not in mapping_set:
        problems.append(f'式子里有「{term}」，对照表（{terms_path}）里没有这一行')
for term in mapping:
    if term not in formula_set:
        problems.append(f'对照表（{terms_path}）里有「{term}」，式子（{formula_path}）里没有这一项')

exceptions = []
for term, owner in mapping.items():
    if owner.startswith('**例外**：'):
        reason = owner[len('**例外**：'):].strip()
        if len(reason) < 8:
            problems.append(f'「{term}」写了例外，但理由不到 8 个字：{owner}')
        exceptions.append(term)
        continue
    match = re.match(r'^第 (\d+) 项（(.+)）$', owner)
    if not match:
        problems.append(f'「{term}」的归属写成了「{owner}」，要么「第 N 项（名字）」、要么「**例外**：理由」')
        continue
    number, name = int(match.group(1)), match.group(2)
    if number not in statistics:
        problems.append(f'「{term}」指到第 {number} 项，统计量表里没有这一行')
    elif '已撤回' in statistics[number]:
        problems.append(f'「{term}」指到第 {number} 项，而那一项已撤回')
    elif statistics[number] != name:
        problems.append(f'「{term}」写的是第 {number} 项（{name}），统计量表里第 {number} 项叫「{statistics[number]}」')

canonical = re.sub(r'[\s`]', '', formula_text)
copies = 0
for path, text in texts.items():
    for number, line in enumerate(text.split('\n'), 1):
        position = line.find('可用(d) =')
        if position < 0:
            position = line.find('可用 = Σ设备')
        if position < 0:
            position = line.find('可用 =Σ设备')
        if position < 0:
            continue
        copies += 1
        fragment = line[position:]
        if '`' in line[:position]:
            fragment = fragment.split('`')[0]
        elif line.lstrip().startswith('|'):
            fragment = fragment.split('|')[0]
        fragment = re.sub(r'[\s`]', '', fragment).rstrip('。；;，,')
        # 逐字相等，不是只比开头：副本在权威那一行后面多写一项（「… − 多一项」）也要红
        if fragment != canonical:
            problems.append(f'{path}:{number} 的式子副本与权威那一行（{formula_path}）对不上')

if problems:
    print(f'  ✗ 准入不等式与统计量清单的逐项对照有 {len(problems)} 处不对')
    for problem in problems:
        print(f'     {problem}')  # gate-lint:detail
    print('     → 式子里每一项都要在对照表里有一行：被维护的统计量写「第 N 项（名字）」，做不到的写「**例外**：理由」')
    print('       （2026-09-13 用户定案，D5（快照 / 空间记账机制） 已定项 4）；式子改了，先改对照表，再改各处副本。')
    sys.exit(1)

print(f'  ✓ 准入不等式 {len(formula_terms)} 项逐项有归属：统计量 {len(formula_terms) - len(exceptions)} 项，写明的例外 {len(exceptions)} 项（{"、".join(exceptions)}）；式子副本 {copies} 处与权威一致')
PY
}

# ── 格 segment-registry：段序列登记表与钉住活代码的用例逐字比对（钉不到用例的行与 E142 产物比，产物不在报本次未跑）
# 落的是 C316（提交步骤的登记位有四处且互不相同） 的第①半：layout/01-first-txn.md「八、」登记表的段序列是人抄的。
# 转发给 scripts/crash/check-segment-registry.py，判法不写第二份（在它的文档字符串里）：出处指到用例、或 path 登记了钉住它的 crates 用例的，与用例比；
#   钉不到用例的那几行才与 E142 产物比，产物不在它退 77、这一格跟着退 77（research/ 整个可删，.claude/rules/directory-layout.md）；
#   它退 1（对不上）与 2（结构认不出、文件不在）都记红。
# 它的 --selftest 由 code-tooling 跑，这里不重复跑。
# 脚本按本阶段自己的位置取（$SEGMENT_REGISTRY_SCRIPT），不按 cwd 取，`--root` 指被判的仓：样本会把 cwd 换成样本目录，
#   按 cwd 取到的是样本里的拷贝，仓里的脚本退化了样本照样判对。脚本不在退 1 不退 77：它随仓走，不在就是被删了或挪了。
# 样本：segment-registry-red 只放脚本读的三样合成输入（layout 表的「八、」一节、replay.sh 里 E142 那一行、产物里几行 name=segments；
#   合成的 path 名 a、b… 不在脚本的 PATH_PINNING_TESTS 里，走的是与产物比那一支），
#   十行各埋一种错，另把整条流的状态数、覆盖写固定脚本流的写数与数组与用例里的状态数各错一处，脚本每条逐行比对的分支都有一处会红，逐条点名；
#   field-registry-green 一致，判绿。「声明的 path 集合剩不下恰好一条」走退 2 那一支由 segment-registry-paths-red 判（脚本退 2、这一格退 1）；
#   segment-registry-product-absent 只能与产物比、replay.sh 指的产物不在：脚本退 77，这一格与整道退 77（本次未跑）；
#   与钉住的 crates 用例比那一支与产物行的解析由脚本自己的 --selftest 测（输入是真仓的布局表与用例、脚本旁边的样本，不读 research/）。
cell_segment_registry() {
  if [[ ! -f "$SEGMENT_REGISTRY_SCRIPT" ]]; then
    echo "  ✗ 找不到 $SEGMENT_REGISTRY_SCRIPT"
    echo "     → 怎么办：它随仓走（scripts/ 下），不在就是被删了或挪了：从 git 找回来，挪了就改这个阶段里 SEGMENT_REGISTRY_SCRIPT 那一行的路径。"
    exit 1
  fi
  local segment_registry_status=0
  python3 "$SEGMENT_REGISTRY_SCRIPT" --root "$ROOT" || segment_registry_status=$?
  # 77：只能与产物比的那几处产物不在，脚本已经逐处报了本次未跑；别的非 0 都记红
  if ((segment_registry_status == 0)) || { ((segment_registry_status == 77)) && [[ "$BREAK_SWITCH" != segment-registry-not-run-as-red ]]; }; then
    exit "$segment_registry_status"
  fi
  exit 1
}

# ── 格 format-const-placeholders：格式常量文件里的占位，每个都指得到一条真实存在的分项或欠账
# 占位的写法（crates/singlefs-format/src/*.rs，常量前一行；`/// placeholder:`、`//! placeholder:` 一样认）：
#   // placeholder: D22（单元原子性怎么合成） 已定项 2 —— 为什么还是预想
#   // placeholder: C323（镜像大小全仓没有条款） —— 为什么还是预想
# 判据：编号带简称；D<n> 的分项号在 .claude/kb/decisions/<n>-*.md 的「### 已定项 / ### 未定项」两张表里能找到那一行，
# C<n> 在 .claude/kb/checks-owed.md 里有登记行；简称与登记位一致归 doc-lint，这里只核编号与分项号。
# 没有常量目录、或一个占位都没有，退 77。
# 样本：format-const-placeholders-red 里一个占位指已定项 99、一个把只在已定项表里的第 2 行写成未定项、一个没写编号、
# 一个文档注释写法的指到不存在的 D99；field-registry-green 里一个 D 占位、一个 C 占位都指得到。
# D<n> 取哪一份决策文件：lib-item-ref-status.py 的 numbered_files，代码是中文这一侧，取中文原文。
# 样本：format-const-placeholders-translation-green / -red（中文原文有 / 只有译文有那一条）；弄坏开关 KB_NUMBERED_FILE_BREAK=first-match。
cell_format_const_placeholders() {
  [[ -d crates/singlefs-format/src ]] || { echo "  ! 没有 crates/singlefs-format/src，这一格无对象可判（步 0 之前没有常量文件）"; exit 77; }

  python3 - "$ITEM_REF_STATUS_LIBRARY" <<'PY'
import glob, importlib.util, os, re, sys
spec = importlib.util.spec_from_file_location('item_ref_status', sys.argv[1])
item_ref_status = importlib.util.module_from_spec(spec); spec.loader.exec_module(item_ref_status)
bad = []
placeholders = []
for path in sorted(glob.glob('crates/singlefs-format/src/**/*.rs', recursive=True)):
    for number, line in enumerate(open(path, encoding='utf-8'), 1):
        m = re.match(r'\s*//[/!]?\s*placeholder:\s*(.*)$', line)
        if not m:
            continue
        text = m.group(1).strip()
        placeholders.append((path, number, text))
        d = re.match(r'D(\d+)（[^）]+）\s*(已定项|未定项)\s*(\d+)', text)
        c = re.match(r'C(\d+)（[^）]+）', text)
        if d:
            n, kind, k = d.group(1), d.group(2), d.group(3)
            # 代码是中文这一侧，取中文原文（numbered_files）；取 glob 不排序的第一个会取到哪份译文，看目录项的次序
            files = item_ref_status.numbered_files('.claude/kb/decisions', n, path)
            if not files:
                bad.append((path, number, f'D{n} 没有决策文件')); continue
            body = open(files[0], encoding='utf-8').read()
            # 索引行 `| k | **…` 或正文标题 `#### 已定项 k（`，两种形态任一命中即可。
            # 索引行只在同名那一节（`### 已定项` / `### 未定项`）第一个 #### 之前找：不分节时 `未定项 2` 会被已定项表的第 2 行放行
            section = re.search(rf'^### {kind}[ \t]*$(.*?)(?=^#{{2,4}} |\Z)', body, re.M | re.S)
            index_table = section.group(1) if section else ''
            hit = re.search(rf'^\|\s*{k}\s*\|', index_table, re.M) or re.search(rf'^#{{3,5}}\s*{kind}\s*{k}[（:：]', body, re.M)
            if not hit:
                bad.append((path, number, f'D{n} 里找不到 {kind} {k}'))
        elif c:
            n = c.group(1)
            owed = open('.claude/kb/checks-owed.md', encoding='utf-8').read()
            if not re.search(rf'^\|\s*C{n}\s*\|', owed, re.M):
                bad.append((path, number, f'checks-owed.md 里找不到 C{n} 的登记行'))
        else:
            bad.append((path, number, '占位没写成「D<n>（简称） 已定项/未定项 k」或「C<n>（简称）」'))
if bad:
    print(f'  ✗ 格式常量文件里有 {len(bad)} 个占位指不到分项或欠账：')
    for path, number, why in bad:
        print(f'     {path}:{number}  {why}')  # gate-lint:detail
    print('     → 怎么办：占位那一行写成「// placeholder: D<n>（简称） 已定项 k —— 为什么还是预想」或「// placeholder: C<n>（简称） —— …」，')
    print('       编号要在 .claude/kb/decisions/ 的索引表或 .claude/kb/checks-owed.md 里真的有那一行；分项定了就把占位行删掉。')
    sys.exit(1)
if not placeholders:
    print('  ! 本次无对象可判：crates/singlefs-format/src 下一个 `// placeholder:` 占位都没有，没有编号可核')
    sys.exit(77)
print(f'  ✓ 格式常量文件里的占位都指得到分项或欠账（{len(placeholders)} 个占位：' + '；'.join(t.split(' —— ')[0] for _, _, t in placeholders) + '）')
PY
}

# ── 格 second-txn-hooks：覆盖写、释放、回退与复用的三处互相挂钩
# 里程碑「覆盖写、释放、回退与复用」的字节登记在三处：`layout/02-second-txn.md` 那张表登记第一次出现的盘上形态，
# 每行指里程碑的一步与钉住它的用例；里程碑每步的「**写出的字节**：」点名 layout/01 的节与 layout/02 的行；
# `layout/01-first-txn.md`「八、根槽写路径的段序列登记表」的行标「里程碑「覆盖写、释放、回退与复用」步 N」。
# 新池新建文件那三份归 first-txn-hooks。
# 判据（每条都报绝对数）：
#   0. layout/02 表每一行都要六格（形态 / 第一次出现在哪次发布 / 写成什么 / 决策分项 / 里程碑那一步 / 钉住它的用例），少一格判红，不静默丢掉。
#   1. layout/02 表每一行「里程碑那一步」点名的步 N 在里程碑里存在，且步 N 的「写出的字节」行用「」点名了这一行的形态
#      （前缀即可，但这个前缀只许对得上一行：同时是几行形态的前缀，就指不出点的是哪一行，不算点名，判红）。
#   2. 里程碑里每一行「写出的字节」用「」点名的东西，要么是 layout/01 的节标题（子串即可），要么是 layout/02 表某一行形态的前缀
#      （同样只许对得上一行）；点名 layout/02 某一行的步 N，那一行的「里程碑那一步」里要有步 N。
#   3. layout/02 表「钉住它的用例」里反引号的 `*.rs` 在 `crates/*/tests/` 下存在，反引号的函数名在同一格点名的文件里有 `fn 名字`。
#   4. 步 N 的「写出的字节」点名了 layout/01 的「八、」那一节，当且仅当「八、」那张表有一行写「里程碑「覆盖写、释放、回退与复用」步 N」。
# 射程：只认「」引起来的点名与反引号里的用例名；点名的内容与字节对不对，这里不判（字节由用例与崩溃点重放钉）。三份文件缺一份，退 77。
# 样本：second-txn-hooks-red 里一行指的步没点名它、一步点名了不存在的节、用例文件不在、用例函数改了名、
# 「八、」表标着的步没点名那一节、点名「八、」的步表里没有、一行只有四格、点名写成两行的共同前缀；field-registry-green 三处互相点名。
cell_overwrite_release_rollback_reuse_hooks() {
  local L1=.claude/kb/layout/01-first-txn.md
  local L2=.claude/kb/layout/02-second-txn.md
  local MS=.claude/kb/milestone/02-second-txn.md
  [[ -f "$L1" && -f "$L2" && -f "$MS" ]] || { echo "  ! 找不到 $L1 / $L2 / $MS，这一格无对象可判"; exit 77; }

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
short_rows = [row for row in rows if len(row) < 6]
for row in short_rows:
    bad.append(f"{l2_path}「{row[0][:30]}」这一行只有 {len(row)} 格，登记表每行要 6 格（形态 / 第一次出现在哪次发布 / 写成什么 / 决策分项 / 里程碑那一步 / 钉住它的用例）")
rows = [row for row in rows if len(row) >= 6]

ms_lines = ms.splitlines()
step_starts = [(index, int(match.group(1))) for index, line in enumerate(ms_lines) if (match := re.match(r"^## 步 (\d+)", line))]
step_bytes_line = {}
for position, (start, number) in enumerate(step_starts):
    end = step_starts[position + 1][0] if position + 1 < len(step_starts) else len(ms_lines)
    step_bytes_line[number] = next((line for line in ms_lines[start:end] if line.startswith("**写出的字节**：")), "")

def rows_named_by(quote):
    """「」里的串点名 layout/02 的哪一行：它是那一行形态的前缀，而且只对得上这一行。同时是几行的前缀就指不出是哪一行，不算点名。"""
    matched = [row for row in rows if plain(row[0]).startswith(plain(quote))]
    return matched if len(matched) == 1 else []

def ambiguous_prefix(quote):
    matched = [row for row in rows if plain(row[0]).startswith(plain(quote))]
    return matched if len(matched) > 1 else []

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
        shared_prefix = ambiguous_prefix(quote)
        if shared_prefix:
            bad.append(f"{ms_path}:{index + 1} 点名「{quote[:40]}」，而它是 {l2_path} 里 {len(shared_prefix)} 行形态的共同前缀"
                       f"（{'、'.join(row[0][:20] for row in shared_prefix)}），指不出点的是哪一行，不算点名")
            continue
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
            registry_steps |= {int(x) for x in re.findall(r"里程碑「覆盖写、释放、回退与复用」\s*步 (\d+)", line)}
else:
    bad.append(f"{l1_path} 里没有「## 八、」那一节（段序列登记表）")
steps_naming_registry = {number for number, line in step_bytes_line.items() if re.search(r"「八、[^」]*段序列登记表", line)}
for number in sorted(registry_steps - steps_naming_registry):
    bad.append(f"{l1_path}「八、」有一行标着里程碑「覆盖写、释放、回退与复用」步 {number}，而步 {number} 的「写出的字节」没点名「八、」那一节")
for number in sorted(steps_naming_registry - registry_steps):
    bad.append(f"{ms_path} 步 {number} 的「写出的字节」点名了「八、」那一节，而那张表里没有一行标「里程碑「覆盖写、释放、回退与复用」步 {number}」")

if bad:
    print(f"  ✗ 覆盖写、释放、回退与复用的三处挂钩对不上 {len(bad)} 处：")    # gate-lint:summary
    for item in bad:
        print("      " + item)                                # gate-lint:detail
    print(f"  → 怎么办：{l2_path} 每一行的「里程碑那一步」与那一步的「**写出的字节**：」互相点名（「形态」原样引）；")
    print(f"    点名的节要在 {l1_path} 里存在，用例文件与函数名要在 crates/*/tests/ 里存在；")
    print(f"    某一步写了「八、」段序列那一节，那张表就要有一行标「里程碑「覆盖写、释放、回退与复用」步 N」，反过来也一样。")
    sys.exit(1)

print(f"  ✓ 覆盖写、释放、回退与复用的三处互相挂钩（登记表 {len(rows)} 行、行到步 {row_links} 处、「写出的字节」点名 {quotes_checked} 处、用例 {tests_checked} 个、段序列登记表标到的步 {len(registry_steps)} 个）")
PY
}

# ── 格 third-txn-hooks：删除写出的字节的三处互相挂钩（里程碑「删除与多次 COW」三·6 要的那道，照 second-txn-hooks 的形态）
# 三份文件：.claude/kb/layout/03-third-txn.md（删除新写出的形态登记表）、.claude/kb/milestone/03-third-txn.md（作业点表，一行一个「| 三·N |」）、
# .claude/kb/layout/01-first-txn.md（字段的宽度与落点、「八、」段序列登记表）。
# 判据（每条都报绝对数）：
#   0. layout/03 开头（第一个 ## 之前）点名它是里程碑哪一步的登记表（「三·N 的登记表」），那一步在里程碑里有「| 三·N |」那一行、
#      而且那一行点名 `layout/03-third-txn.md`（字节那一步）；里程碑里一行都没点名这份文件也判红。
#   1. layout/03「一、」那张表（表头首格「形态」、有「里程碑那一步」一列）每行格数等于表头；「里程碑那一步」点名的作业点（X·N）在里程碑里都有那一行，
#      而且含开头点名的字节那一步（行是后来别的作业点加的，把两步都写上；写「第 N 批」这种不是作业点的判红）。
#   2. layout/03 里「[…](01-first-txn.md)「X」」引的 X 是 layout/01 的节标题（子串）或某张表一行的首格；「段序列登记表「X」那一行」的 X 是 layout/01「八、」那张表某一行的首格。
#   3. layout/03「四、」那张表「出处」一列里反引号的 `*.rs` 在仓里（路径相对仓根），紧跟在它后面的反引号函数名在那份文件里有 `fn 名字`。
# 射程：只认上面几种写法的点名；字节对不对由 E172（删除写出的字节的干跑） 与钉住的用例判，这里不判。「## 历史版本」之后不扫。三份文件缺一份，退 77。
# 样本：third-txn-hooks-red 里一行写「第 50 批」（history-kept: 样本里的批次写法是被判的数据）、一行指里程碑没有的 三·9、一行少一格、引 layout/01 一个不存在的节、引段序列登记表不存在的行、
#   出处的函数改了名、出处的文件不在，七处逐条点名；third-txn-hooks-green 三处互相点名、出处一份在 tests/ 一份在 src/。
# 弄坏开关 DOC_REGISTRIES_BREAK=third-txn-hooks（整格不跑）、third-txn-bytes-step-unchecked（判据 1 后半不判：行不点名字节那一步也放过）。
cell_crash_verification_rebuild_hooks() {
  local L1=.claude/kb/layout/01-first-txn.md
  local L3=.claude/kb/layout/03-third-txn.md
  local MS=.claude/kb/milestone/03-third-txn.md
  [[ -f "$L1" && -f "$L3" && -f "$MS" ]] || { echo "  ! 找不到 $L1 / $L3 / $MS，这一格无对象可判"; exit 77; }

  python3 - "$L1" "$L3" "$MS" "$BREAK_SWITCH" <<'PY'
import glob, os, re, sys

l1_path, l3_path, ms_path = sys.argv[1:4]
break_switch = sys.argv[4] if len(sys.argv) > 4 else ""
body = lambda path: open(path, encoding="utf-8").read().split("\n## 历史版本")[0]
l1, l3, ms = body(l1_path), body(l3_path), body(ms_path)
split_cells = lambda line: [cell.strip() for cell in re.split(r"(?<!\\)\|", line.strip().strip("|"))]
plain = lambda text: re.sub(r"[`*]", "", text).strip()
STEP = re.compile(r"[一二三四五六七八九十]+·\d+")
L3_NAME = os.path.basename(l3_path)

bad = []

# 里程碑的作业点行：| 三·6 | 做什么 | … |，作业点号 → 那一行原文
ms_steps = {}
for line in ms.splitlines():
    match = re.match(r"^\|\s*([一二三四五六七八九十]+·\d+)\s*\|", line)
    if match:
        ms_steps[match.group(1)] = line
# 字节那一步：里程碑里点名 layout/03 这份文件的作业点
bytes_steps = sorted(step for step, line in ms_steps.items() if f"layout/{L3_NAME}" in line)
if not bytes_steps:
    bad.append(f"{ms_path} 里没有一行作业点点名 `layout/{L3_NAME}`（登记删除写出的字节那一步）")

# layout/03 开头（第一个 ## 之前）点名的作业点，要是里程碑里点名了这份文件的那一步
head = l3.split("\n## ", 1)[0]
head_steps = [step for step in STEP.findall(head)]
if not head_steps:
    bad.append(f"{l3_path} 开头一段没点名它是里程碑哪一步的登记表（写「三·N 的登记表」）")
for step in head_steps:
    if step not in ms_steps:
        bad.append(f"{l3_path} 开头点名作业点 {step}，而 {ms_path} 里没有「| {step} |」那一行")
    elif step not in bytes_steps:
        bad.append(f"{l3_path} 开头点名作业点 {step}，而 {ms_path} 那一行没点名 `layout/{L3_NAME}`")
own_steps = [step for step in head_steps if step in bytes_steps]

# 0 / 1. 「一、形态」那张表：每行五格，「里程碑那一步」点名的作业点都在里程碑里，且含开头点名的那一步
section = re.search(r"^## 一、.*?(?=^## |\Z)", l3, re.M | re.S)
rows, header = [], None
if section:
    for line in section.group(0).splitlines():
        if not line.startswith("|"):
            continue
        cells = split_cells(line)
        if header is None:
            header = cells
            continue
        if set("".join(cells)) <= set("-: "):
            continue
        rows.append(cells)
if not section or header is None or header[:1] != ["形态"]:
    bad.append(f"{l3_path} 里找不到「## 一、」那一节表头是「| 形态 |」的登记表")
step_column = header.index("里程碑那一步") if header and "里程碑那一步" in header else None
if header and step_column is None:
    bad.append(f"{l3_path}「一、」那张表的表头没有「里程碑那一步」一列")
row_links = 0
for cells in rows:
    name = plain(cells[0])[:30]
    if header and len(cells) != len(header):
        bad.append(f"{l3_path}「{name}」这一行 {len(cells)} 格，表头 {len(header)} 格")
        continue
    if step_column is None:
        continue
    named = STEP.findall(cells[step_column])
    row_links += len(named)
    for step in named:
        if step not in ms_steps:
            bad.append(f"{l3_path}「{name}」指作业点 {step}，而 {ms_path} 里没有「| {step} |」那一行")
    if break_switch != "third-txn-bytes-step-unchecked" and own_steps and not set(own_steps) & set(named):
        bad.append(f"{l3_path}「{name}」的「里程碑那一步」写「{cells[step_column]}」，没点名登记这份表的那一步 {'、'.join(own_steps)}")

# 2. 引 layout/01 的「…」：节标题（子串）或某张表某一行的首格
l1_headings = [re.sub(r"^#{2,4} ", "", line).strip() for line in l1.splitlines() if re.match(r"^#{2,4} ", line)]
l1_first_cells = {plain(split_cells(line)[0]) for line in l1.splitlines() if line.startswith("|")}
registry = re.search(r"^## 八、.*?(?=^## |\Z)", l1, re.M | re.S)
registry_rows = {plain(split_cells(line)[0]) for line in (registry.group(0).splitlines() if registry else []) if line.startswith("|")}
quotes_checked = 0
for number, line in enumerate(l3.splitlines(), 1):
    for match in re.finditer(r"\]\(01-first-txn\.md\)「([^」]+)」", line):
        quotes_checked += 1
        quote = plain(match.group(1))
        if not any(quote in heading for heading in l1_headings) and quote not in l1_first_cells:
            bad.append(f"{l3_path}:{number} 引 layout/01「{quote}」，而 {l1_path} 没有这一节，也没有一行首格是它")
    for match in re.finditer(r"段序列登记表「([^」]+)」那一行", line):
        quotes_checked += 1
        quote = plain(match.group(1))
        if quote not in registry_rows:
            bad.append(f"{l3_path}:{number} 引段序列登记表「{quote}」那一行，而 {l1_path}「八、」那张表里没有首格是它的行")

# 3. 「四、段序列」出处一列点名的源码文件与函数在仓里
tests_checked = 0
segments = re.search(r"^## 四、.*?(?=^## |\Z)", l3, re.M | re.S)
segment_header = None
for line in (segments.group(0).splitlines() if segments else []):
    if not line.startswith("|"):
        continue
    cells = split_cells(line)
    if segment_header is None:
        segment_header = cells
        continue
    if set("".join(cells)) <= set("-: ") or "出处" not in segment_header or len(cells) != len(segment_header):
        continue
    source_cell = cells[segment_header.index("出处")]
    # 「`路径` 的 `函数`」：函数归它前面最近的那个路径
    current = None
    for token in re.findall(r"`([^`]+)`", source_cell):
        if token.endswith(".rs"):
            tests_checked += 1
            current = token if os.path.isfile(token) else None
            if current is None:
                bad.append(f"{l3_path}「{plain(cells[0])[:20]}」出处点名的 {token} 在仓里不存在")
        elif re.fullmatch(r"[a-z_][a-z0-9_]*", token) and current:
            tests_checked += 1
            if not re.search(rf"\bfn {re.escape(token)}\b", open(current, encoding="utf-8").read()):
                bad.append(f"{l3_path}「{plain(cells[0])[:20]}」出处点名的 {token} 在 {current} 里找不到 fn")
if segments is None:
    bad.append(f"{l3_path} 里没有「## 四、」那一节（删除类发布的段序列）")

if bad:
    print(f"  ✗ 删除写出的字节的三处挂钩对不上 {len(bad)} 处：")    # gate-lint:summary
    for item in bad:
        print("      " + item)                                # gate-lint:detail
    print(f"  → 怎么办：{l3_path} 开头写明它是里程碑哪一步（「三·N 的登记表」），那一步在 {ms_path} 里点名 `layout/{L3_NAME}`；")
    print(f"    「一、」表每一行的「里程碑那一步」点名那一步（行是后来别的作业点加的，把两步都写上）；")
    print(f"    引 {l1_path} 的「」要是那份里的节标题或某一行的首格，段序列登记表的行名照「八、」逐字抄；「四、」出处点名的文件与函数要在仓里。")
    sys.exit(1)

print(f"  ✓ 删除写出的字节三处互相挂钩（字节那一步 {'、'.join(own_steps)}；形态 {len(rows)} 行、行到作业点 {row_links} 处、"
      f"引 layout/01 {quotes_checked} 处、段序列出处的文件与函数 {tests_checked} 个）")
PY
}
# ── 格 tree-table-reserve：树表条目预留字节的认购合计（认购不许超过预留，每条要指得到一个编号）
# 补的是 C338（树表条目预留没有余量）要的算和检查：合计靠人算会漂。
# 表的形态（.claude/kb/decisions/08-核心索引结构.md 已定项 11）：
#   <!-- gate:tree-table-reserve total=76 -->
#   | 认购者 | 住哪条条目 | 宽度 | 出处 |
#   |---|---|---|---|
#   | … | 头条目 | 8 | C271（…） |
# 判据三条：① 宽度列全是正整数；② 合计不超过标记里的 total；③ 每行「出处」要带一个编号
#（C<n> 在 .claude/kb/checks-owed.md 有登记行，D<n> 在 .claude/kb/decisions/<n>-*.md 存在；编号的位数不设上限，后面紧跟全角括号里的简称）。
# 带标记的表不是恰好一张（含一张都没有）判红；认购行不足四格（少了宽度或出处）判红，不静默丢掉；标记下面一条认购都没有，退 77。
# 内嵌 python 的退出码要取，而且必须报出 CHECKED 那一行：它崩了（例如读不到 checks-owed.md）时不许走绿。
# 样本：tree-table-reserve-red 里认购合计 88 超过预留 76、一行只有三格；field-registry-green 三条认购合计 32、余 44；
#   tree-table-reserve-wide-number-green 的出处指到四位的欠账号 C1000 与三位的决策号 D105（样本里都有登记行或正文），判绿。
# 标记只认 decisions/ 下的中文原文（取法 lib-item-ref-status.py 的 originals_only）：译文照抄标记与表，首行是各自的语言。
#   样本 field-registry-translation-green（D8 带两份照抄了标记的译文），判绿；弄坏开关 DOC_REGISTRIES_BREAK=read-translations 下「实际 3 张」判红。
cell_tree_table_reserve() {
  local report python_rc
  report="$(BREAK_SWITCH="$BREAK_SWITCH" ITEM_REF_STATUS_LIBRARY="$ITEM_REF_STATUS_LIBRARY" python3 - <<'PY'
import glob, importlib.util, os, re, sys

item_ref_status_spec = importlib.util.spec_from_file_location("item_ref_status", os.environ["ITEM_REF_STATUS_LIBRARY"])
item_ref_status = importlib.util.module_from_spec(item_ref_status_spec)
item_ref_status_spec.loader.exec_module(item_ref_status)
marker = re.compile(r"<!--\s*gate:tree-table-reserve\s+total=(\d+)\s*-->")
# 只读中文原文：译文照抄标记与认购表，读进来带标记的表就成了三张。弄坏开关 read-translations 连译文一起读
files = item_ref_status.originals_only(sorted(glob.glob(".claude/kb/decisions/*.md")),
                                       os.environ.get("BREAK_SWITCH") == "read-translations")
found = []
for path in files:
    lines = open(path, encoding="utf-8").read().split("\n")
    for index, line in enumerate(lines):
        matched = marker.search(line)
        if matched:
            found.append((path, index, int(matched.group(1)), lines))

if len(found) != 1:
    print("BAD", f"带 gate:tree-table-reserve 标记的表应当恰好有一张，实际 {len(found)} 张", sep="\t")
    print("CHECKED", 0, 0, 0, sep="\t")
    sys.exit()

path, index, total, lines = found[0]
rows = []
for line in lines[index + 1:]:
    if not line.startswith("|"):
        if rows:
            break
        continue
    cells = [cell.strip() for cell in re.split(r"(?<!\\)\|", line.strip().strip("|"))]
    if cells[0] in ("认购者",) or cells[0].startswith("---"):
        continue
    if len(cells) < 4:
        print("BAD", f"「{cells[0]}」这一行只有 {len(cells)} 格，认购表每行要四格（认购者 / 住哪条条目 / 宽度 / 出处），它的宽度没进合计", sep="\t")
        continue
    rows.append(cells)

owed = open(".claude/kb/checks-owed.md", encoding="utf-8").read()
subtotal = 0
for cells in rows:
    name, width_text, source = cells[0], cells[2], cells[3]
    if not re.fullmatch(r"\d+", width_text):
        print("BAD", f"「{name}」的宽度列不是正整数：{width_text}", sep="\t")
        continue
    subtotal += int(width_text)
    # 弄坏开关 DOC_REGISTRIES_BREAK=owed-number-width-fixed：编号位数写死（C 三位、D 两位）
    number_pattern = r"(C\d{1,3}|D\d{1,2})（" if os.environ.get("BREAK_SWITCH") == "owed-number-width-fixed" else r"(C\d+|D\d+)（"
    numbers = re.findall(number_pattern, source)
    if not numbers:
        print("BAD", f"「{name}」的出处里没有带简称的编号", sep="\t")
        continue
    for number in numbers:
        if number.startswith("C"):
            if f"| {number} |" not in owed:
                print("BAD", f"「{name}」指的 {number} 在 checks-owed.md 里没有登记行", sep="\t")
        elif not (glob.glob(f".claude/kb/decisions/{int(number[1:])}-*.md")
                  or glob.glob(f".claude/kb/decisions/{int(number[1:]):02d}-*.md")):
            print("BAD", f"「{name}」指的 {number} 在 decisions/ 下没有正文", sep="\t")

if subtotal > total:
    print("BAD", f"认购合计 {subtotal} 超过预留 {total}", sep="\t")
print("CHECKED", len(rows), subtotal, total, sep="\t")
PY
)"
  python_rc=$?
  if ((python_rc != 0)) || ! grep -q '^CHECKED' <<<"$report"; then
    printf '%s\n' "$report" | tail -5 | sed 's/^/      /'   # gate-lint:detail
    echo "  ✗ 认购表没算完：内嵌 python 退出码 $python_rc，$(grep -q '^CHECKED' <<<"$report" && echo '报了' || echo '没报出') CHECKED 那一行"
    echo "    → 怎么办：单独跑一遍这一格（--check tree-table-reserve）看 python 的报错（多半是 .claude/kb/checks-owed.md 或决策文件读不到）；"
    echo "      没算完就是什么都没查，成功句里的数会是空的，而它看着与判过了一模一样。"
    exit 1
  fi

  local -a bad
  local rows subtotal total why
  mapfile -t bad < <(grep '^BAD' <<<"$report")
  read -r _ rows subtotal total < <(grep '^CHECKED' <<<"$report")

  if ((${#bad[@]})); then
    echo "  ✗ 树表条目预留的认购表不合规："
    while IFS=$'\t' read -r _ why; do
      printf '      %s\n' "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${bad[@]}")
    echo "    → 怎么办：认购表在 .claude/kb/decisions/08-核心索引结构.md 已定项 11，标记那一行写着 total=<预留字节数>。"
    echo "      合计超了就先去掉一个认购者（或把那一段的宽度压小），别直接改 total——预留多少由那条已定项定；"
    echo "      出处指不到编号的，补一个还开着的欠账号或一条已定分项，编号要带简称。"
    exit 1
  fi
  if [[ "$rows" == 0 ]]; then
    echo "  ⊘ 本次无对象可判：gate:tree-table-reserve 标记下面一条认购都没有（预留 ${total}）"
    exit 77
  fi
  echo "  ✓ 树表条目预留的认购表对得上（${rows} 条认购，合计 ${subtotal} 字节、预留 ${total}、余 $((total - subtotal))；每条的出处都指得到一个编号）"
}

# ── 格 field-widths：新池新建文件字段表里写了宽度的行，被指的已定分项正文里写出了这个宽度
# 补的是 C66（格式级空白没有分项号）要的第二格：field-refs 只判被指的分项存在，不判那条分项有没有说这句话（D19（块指针的结构与宽度预算） 已定项 1 那一例：
# 指向它、写着它从没定过的宽度，照样放行）。
# 判据：.claude/kb/layout/01-first-txn.md 里表头既有「宽…」一列又有「指向」一列的表，每一行的宽度格（去掉全角括号里的注、粗体与反引号）是一个正整数、
#   「指向」里至少一条「D<n>（简称） 已定项 k」（「/ 已定项 m」接着写的算同一条决策的几项）时，这个数要以独立的数出现在那几条已定项的正文里：
#   正文 = 决策中文原文（lib-item-ref-status.py 的 numbered_files）「### 已定项」索引里那一行（去掉打头的编号格）连同「#### 已定项 k」那一小节（去掉标题行），
#   再挖掉编号写法里的数（已定项 / 未定项 / 硬要求 / 推论 k、第 k 项 / 问 / 条、C<n>、D<n>、E<n>、I-a.b、日期）。
# 不判：宽度写成算式或几段的（「14 × 2」「10 / 8」「随算法」）、指向里没有已定项的（只指未定项、推论、欠账）；成功那句报判了几行、不判几行。
# 判不了：宽度是个常见的小数（1、2、4、8），正文里别处另有这个数时照算写出了。
# 样本：field-widths-red 里「设备 ID」宽 1 指一条只说「带」的已定项、「密文校验和」宽 4 指一条只说「32 位」的已定项，各点名判红；算式宽度与只指未定项的两行不判；
#   field-widths-green 同一张表，被指的已定项写出了 1 与 4。
# 弄坏开关 DOC_REGISTRIES_BREAK=field-widths（整格不跑）、field-widths-numbering-counted（索引行的编号格、小节标题与编号写法里的数照算）。
cell_field_widths() {
  local TABLE=.claude/kb/layout/01-first-txn.md
  [[ -f "$TABLE" ]] || { echo "  ! 没有 $TABLE，这一格无对象可判"; exit 77; }
  [[ -d .claude/kb/decisions ]] || { echo "  ! 没有 .claude/kb/decisions，这一格无对象可判"; exit 77; }
  local report rc summary judged skipped red
  report="$(python3 - "$TABLE" .claude/kb "$ITEM_REF_STATUS_LIBRARY" "$BREAK_SWITCH" 2>&1 <<'PY'
import importlib.util, re, sys

TABLE, KB, LIBRARY = sys.argv[1:4]
BREAK = sys.argv[4] if len(sys.argv) > 4 else ""
spec = importlib.util.spec_from_file_location("item_ref_status", LIBRARY)
item_ref_status = importlib.util.module_from_spec(spec); spec.loader.exec_module(item_ref_status)
split_cells = lambda line: [cell.strip() for cell in re.split(r"(?<!\\)\|", line.strip().strip("|"))]
REFERENCE = re.compile(r"D(\d+)(?:（(?:[^（）]|（[^（）]*）)*）)? *(已定项|未定项) *(\d+)((?: */ *(?:已定项|未定项) *\d+)*)")
# 编号写法（已定项 k、未定项 k、第 k 项、第 k 问、D<n>、C<n>、E<n>、I-a.b）里的数不算「写出了这个宽度」
NUMBERING = re.compile(r"(?:已定项|未定项|硬要求|推论) *\d+|第 *\d+ *[项问条行节]|\b[CDE]\d+|I-\d+(?:\.\d+)?|\d{4}-\d{2}-\d{2}")
item_texts = {}


def item_text(number, kind, item):
    """分项正文：索引里那一行（去掉打头的编号格）连同「#### <kind> <item>」那一小节（去掉标题行）。"""
    key = (number, kind, item)
    if key in item_texts:
        return item_texts[key]
    found = item_ref_status.numbered_files(f"{KB}/decisions", number, TABLE)
    if not found:
        item_texts[key] = None
        return None
    text = open(found[0], encoding="utf-8").read().split("\n## 历史版本")[0]
    pieces = []
    index = re.search(rf"^### {kind}\s*$(.*?)(?=^#)", text, re.M | re.S)
    for line in (index.group(1).splitlines() if index else []):
        row = re.match(rf"^\| *{item} *\|(.*)$", line) or re.match(rf"^{item}\. (.*)$", line)
        if row:
            pieces.append(line if BREAK == "field-widths-numbering-counted" else row.group(1))
    section = re.search(rf"^#### {kind} *{item}(?!\d)[^\n]*\n(.*?)(?=^#{{2,4}} |\Z)", text, re.M | re.S)
    if section:
        pieces.append(section.group(0) if BREAK == "field-widths-numbering-counted" else section.group(1))
    joined = "\n".join(pieces)
    item_texts[key] = joined if BREAK == "field-widths-numbering-counted" else NUMBERING.sub(" ", joined)
    return item_texts[key]


bad, judged, skipped = [], 0, 0
lines = open(TABLE, encoding="utf-8").read().split("\n## 历史版本")[0].splitlines()
header = None
for number, line in enumerate(lines, 1):
    if not line.startswith("|"):
        header = None
        continue
    cells = split_cells(line)
    if header is None:
        header = cells
        continue
    if set("".join(cells)) <= set("-: ") or "指向" not in header or len(cells) != len(header):
        continue
    width_columns = [index for index, name in enumerate(header) if name.startswith("宽")]
    if not width_columns:
        continue
    width_text = cells[width_columns[0]]
    previous = None
    while previous != width_text:
        previous, width_text = width_text, re.sub(r"（[^（）]*）", "", width_text)
    width_text = re.sub(r"[*`]", "", width_text).strip()
    references = []
    for match in REFERENCE.finditer(cells[header.index("指向")]):
        decision = int(match.group(1))
        references.append((decision, match.group(2), match.group(3)))
        for kind, item in re.findall(r"(已定项|未定项) *(\d+)", match.group(4)):
            references.append((decision, kind, item))
    decided = [reference for reference in references if reference[1] == "已定项"]
    if not re.fullmatch(r"[1-9]\d*", width_text) or not decided:
        skipped += 1
        continue
    judged += 1
    texts = [item_text(*reference) for reference in decided]
    joined = "\n".join(text for text in texts if text)
    if not re.search(rf"(?<![\d.]){width_text}(?![\d.])", joined):
        field = re.sub(r"[*`]", "", cells[0])[:30]
        named = "、".join(f"D{d} {k} {i}" for d, k, i in decided)
        bad.append(f"{TABLE}:{number}「{field}」宽 {width_text}，指向的 {named} 正文里没写出 {width_text}")

for item in bad:
    print(f"  ✗ {item}")    # gate-lint:detail
print(f"COUNT\t{judged}\t{skipped}\t{len(bad)}")
sys.exit(1 if bad else 0)
PY
)"
  rc=$?
  summary="$(tail -n 1 <<<"$report")"
  if [[ ( $rc -ne 0 && $rc -ne 1 ) || "$summary" != COUNT$'\t'* ]]; then
    printf '%s\n' "$report" | sed 's/^/  /'
    echo "  ✗ 判字段宽度的那段 python 没跑成（退出码 $rc）——这一格没判"
    echo "     → 怎么办： 按上面的 traceback 修；没跑成就是没判，不许当成判过了。"
    exit 1
  fi
  IFS=$'\t' read -r _ judged skipped red <<<"$summary"
  head -n -1 <<<"$report"   # gate-lint:detail
  if ((red)); then
    echo "  ✗ 新池新建文件字段表里 $red 行写的宽度，被指的已定分项正文里没写出（判了 $judged 行）"
    echo "     → 怎么办： 让被指的已定分项写出这个宽度（定过了就补那一句；从没定过就开一条未定项，把这一行的指向改成它、状态改成未定）；"
    echo "                指错了分项就改「指向」。宽度写成算式或几段的不判，要它判就写成一个数。"
    exit 1
  fi
  if ((judged == 0)); then
    echo "  ! $TABLE 里没有写了单个宽度、又指向已定项的字段行（不判 $skipped 行），这一格无对象可判"
    exit 77
  fi
  echo "  ✓ 新池新建文件字段表写了宽度的行，宽度都写在被指的已定分项里（判了 $judged 行；宽度写成算式、几段或指向里没有已定项的 $skipped 行不判）"
}
