#!/usr/bin/env bash
# 门禁 doc-registries 的格函数库：experiment-refs、decision-refs、declared-counts、governance-refs、invariant-count-elsewhere、citations、invariant-anchors、inline-sums、invariant-prerequisites、field-frequency-sizes、unreachable-claims、counted-claims。
# 由 .claude/gate.d/doc-registries.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释；读的 ROOT 与入口里设好的各变量、bad、ok、howto 在调用时才取。

# ════════════════ 引用与不变量组的七格（引用与不变量） ════════════════

# ── experiment-refs、decision-refs 共用：扫的文件与编号引用的抽法 ─────────────────
# 必须递归扫：决策与实验拆在子目录里，引用大半住在那里，$KB/*.md 单层通配会静默漏掉。
# 扫的文件：kb 下全部 .md（递归）加项目规则。数组装、不靠 $(find …) 的分词，文件名里有空格也不散。
scanned_files_collected=0
collect_scanned_files() {
  ((scanned_files_collected)) && return 0
  scanned_files_collected=1
  kb_md_files=()
  while IFS= read -r -d '' found_file; do kb_md_files+=("$found_file"); done \
    < <(find "$KB" -name '*.md' -print0 2>/dev/null)
  shopt -s nullglob
  rule_md_files=(.claude/rules/*.md)
  shopt -u nullglob
  scanned_md_files=("${kb_md_files[@]}" "${rule_md_files[@]}")
  scanned_note="扫 ${#kb_md_files[@]} 份 kb 文件、${#rule_md_files[@]} 份规则"
}
# 引用抽取用 grep 一次读全部文件；某一份读不了（悬空的符号链接、没有读权限）时 grep 退 2，
# 那一份里的引用就从判据里静默消失，其余照判照绿。所以 grep 的错误不丢进 /dev/null：退 2 判红并列出它的原话。
grep_error_file="$work_directory/grep-errors"
# 用法：extract_numbered_references <字母> <数组名>；抽出全部不同的「字母+1–3 位数字」引用，读文件出错判红
extract_numbered_references() {
  local letter="$1" raw_matches grep_exit_code=0
  local -n extracted_references="$2"
  extracted_references=()
  [[ ${#scanned_md_files[@]} -gt 0 ]] || return 0
  raw_matches="$(grep -ohE "(^|[^A-Za-z0-9/-])${letter}[0-9]{1,3}([^A-Za-z0-9-]|\$)" "${scanned_md_files[@]}" 2>"$grep_error_file")" \
    || grep_exit_code=$?
  if [[ "$grep_exit_code" -ge 2 ]]; then
    bad "抽 ${letter} 号引用时有文件读不了（grep 退 $grep_exit_code），那几份里的引用没进判据："
    sed 's/^/      /' "$grep_error_file"
    howto "按上面 grep 的原话修：悬空的符号链接删掉或指回真文件，没有读权限的补上读权限；" \
          "读不了的文件里的引用一条都没判，不能当成都有定义。"
  fi
  mapfile -t extracted_references < <(printf '%s\n' "$raw_matches" | grep -oE "${letter}[0-9]{1,3}" | sort -u)
}

# 用法：defined_numbers <字母> <目录> <关联数组名>：目录下（递归）每份文件里以「## <字母><1–3 位数字> 」起头的标题，编号放进关联数组。一次 grep 取全部，
# 不对每个被引用的编号各扫一遍整个目录；读不了的文件（或目录不在）当没有定义，与逐个编号 grep -q 2>/dev/null 同一处置。
# 弄坏开关 DOC_REGISTRIES_BREAK=defined-numbers-empty：交空集合，被引用的编号一个都找不到定义。
defined_numbers() {
  local letter="$1" directory="$2" heading_marker defined_number
  local -n defined_set="$3"
  defined_set=()
  [[ "$BREAK_SWITCH" == defined-numbers-empty ]] && return 0
  while read -r heading_marker defined_number _; do
    defined_set[$defined_number]=1
  done < <(grep -rhoE "^## ${letter}[0-9]{1,3} " "$directory" 2>/dev/null)
}

cell_experiment_refs() {
  cell_failed=0
  collect_scanned_files
  local missing=0 e
  local experiment_refs=()
  local -A defined_experiments=()
  # 不能用 \bE[0-9]+\b：它会把 URL 里的 E19253-01 当成实验号
  extract_numbered_references E experiment_refs
  defined_numbers E "$KB/experiments" defined_experiments
  for e in "${experiment_refs[@]}"; do
    [[ -n "${defined_experiments[$e]+set}" ]] || { bad "$e 被引用但 experiments/ 下没有它"
      howto "要么在 experiments/ 下给它建正文（\`## $e <简称>\` 起头），" \
            "要么把引用它的那处改成真实存在的实验号——编号引用悬空，检索到的人会自己补一个。"
      missing=1; }
  done
  if [[ ${#experiment_refs[@]} -eq 0 ]]; then
    bad "一个实验号引用都没扫到（$scanned_note）——这一段没有对象可判"
    howto "确认门禁是在仓库根上跑的（第一个参数是仓库根）、$KB 目录在；" \
          "扫到 0 项不是通过（.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。"
  elif [[ $missing -eq 0 ]]; then
    ok "实验号引用全部有定义：${#experiment_refs[@]} 个不同的实验号（$scanned_note）"
  fi
  return "$cell_failed"
}

# ── decision-refs 的豁免标记：<!-- doc-registries:not-decisions D<n> … -->（与 doc-lint 的 <!-- doc-lint:not-numbers … --> 同形）
# 用处：命令原样输出、产物行里形状像决策号、其实不是的串（例：E165 页一段输出里的域名 D0），改不得原文，就在那一段上方单起一行写标记。
# 判法：标记要独占一行（前后只许空白）；它管的是标记之后的那几行——到下一个空行为止；标记下面紧接着是代码块的，管到那个代码块的收尾围栏
#   （代码块里的空行不算断开）。标记点名的编号，在它管的那几行里（连同标记那一行自己）出现的处数不算引用；一个编号全仓每一处都落在
#   某条标记管的行里，才从「被引用的决策号」里摘掉，管的行外面还有一处就照判。
# 判红：标记一个编号都没点名、点名的不是 D 加 1–3 位数字、点名的编号在 decisions/ 下有定义（等于一行注释把检查对它关掉，与 doc-lint 的 not-numbers 同一条）、
#   点名的编号在它管的那几行里一处都没有（不起作用的标记）。
# 射程：扫的文件与 experiment-refs、decision-refs 同一批（collect_scanned_files）；处数的取法与 extract_numbered_references 的 grep 同一条式子、逐行不重叠地取。
# 样本：decision-refs-not-decisions-red 一份 kb 页里标记管的代码块有 D0、块后空行之外的正文又写了一处 D0（照判），另有点名已定义的 D1、
#   点名的 D7 在管的行里没有、一个编号都没点名的三条标记；decision-refs-not-decisions-green 同一段 D0 只在标记管的代码块里，判绿并报豁免了几处。
apply_not_decisions_markers() {
  local -n kept_references="$1"
  local marker_report kind field rest
  not_decisions_note="not-decisions 标记 0 条"
  [[ "$BREAK_SWITCH" == "not-decisions-ignored" ]] && return 0
  [[ ${#scanned_md_files[@]} -gt 0 ]] || return 0
  printf '%s\0' "${scanned_md_files[@]}" > "$work_directory/decision-refs-scanned-files"
  if ! marker_report="$(python3 - "$work_directory/decision-refs-scanned-files" "$KB/decisions" "$BREAK_SWITCH" 2>&1 <<'PY'
import os, re, sys

files_list, decisions_directory, broken = sys.argv[1], sys.argv[2], sys.argv[3]
# 与 extract_numbered_references 的 grep -E 同一条式子、同样逐行、不重叠地取，数出来的处数与 grep 对得上
REFERENCE = re.compile(r"(?:^|[^A-Za-z0-9/-])(D[0-9]{1,3})(?:[^A-Za-z0-9-]|$)")
MARKER = re.compile(r"^\s*<!--\s*doc-registries:not-decisions(?:\s(.*?))?\s*-->\s*$")
FENCE = re.compile(r"^\s*(?:```|~~~)")
NUMBER = re.compile(r"D[0-9]{1,3}")

defined = set()
for directory, _subdirectories, names in os.walk(decisions_directory):
    for name in names:
        if name.endswith(".md"):
            for line in open(os.path.join(directory, name), encoding="utf-8", errors="replace"):
                heading = re.match(r"## (D[0-9]{1,3}) ", line)
                if heading:
                    defined.add(heading.group(1))

total, exempted = {}, {}
exempted_outside_marker_lines = 0
marker_count = 0
problems = []
paths = [path for path in open(files_list, "rb").read().decode("utf-8", "replace").split("\0") if path]
for path in paths:
    try:
        lines = open(path, encoding="utf-8", errors="replace").read().split("\n")
    except OSError:
        continue  # 读不了的那一份 grep 那一步已经判红
    occurrences = [[match.group(1) for match in REFERENCE.finditer(line)] for line in lines]
    for found in occurrences:
        for token in found:
            total[token] = total.get(token, 0) + 1
    for index, line in enumerate(lines):
        marker = MARKER.match(line)
        if not marker:
            continue
        marker_count += 1
        place = f"{path}:{index + 1}"
        tokens = (marker.group(1) or "").split()
        if not tokens:
            problems.append(f"{place} 的 not-decisions 标记一个编号都没点名")
            continue
        if broken == "not-decisions-whole-file":
            scope = list(range(len(lines)))
        else:
            scope, position, in_code = [index], index + 1, False
            while position < len(lines):
                text = lines[position]
                if not in_code and not text.strip():
                    break
                scope.append(position)
                if FENCE.match(text):
                    if in_code:
                        break
                    in_code = True
                position += 1
        body = [position for position in scope if position != index]
        if not body:
            span = "（标记下面紧挨着的是空行，它一行都不管）"
        elif len(body) == 1:
            span = f"第 {body[0] + 1} 行"
        else:
            span = f"第 {body[0] + 1}–{body[-1] + 1} 行"
        for token in tokens:
            if not NUMBER.fullmatch(token):
                problems.append(f"{place} 的 not-decisions 标记里「{token}」不是决策号的写法（D 加 1–3 位数字）")
                continue
            if token in defined:
                problems.append(f"{token} 在 decisions/ 下有定义，{place} 却用 not-decisions 标记把它摘出去")
                continue
            inside = sum(occurrences[position].count(token) for position in body)
            if inside == 0:
                problems.append(f"{place} 的 not-decisions 标记点名的 {token} 在它管的{span}里一处都没有")
                continue
            exempted[token] = exempted.get(token, 0) + inside + occurrences[index].count(token)
            exempted_outside_marker_lines += inside

for problem in problems:
    print("PROBLEM", problem, sep="\t")
for token in sorted(exempted):
    if exempted[token] >= total.get(token, 0):
        print("EXEMPT", token, sep="\t")
print("COUNT", exempted_outside_marker_lines, marker_count, sep="\t")
PY
)"; then
    bad "not-decisions 标记没判完：内嵌 python 自己出错了，标记一条都没认"
    printf '%s\n' "$marker_report" | tail -5 | sed 's/^/      /'   # gate-lint:detail
    howto "按上面的报错修 .claude/gate.d/lib/doc-registries-reference-and-invariant-cells.sh 里 apply_not_decisions_markers 的 python；没判完不是通过，标记管的编号这一轮照引用判。"
    return 0
  fi
  local -A exempt_numbers=()
  local -a marker_problems=()
  while IFS=$'\t' read -r kind field rest; do
    case "$kind" in
      PROBLEM) marker_problems+=("$field") ;;
      EXEMPT) exempt_numbers[$field]=1 ;;
      COUNT) not_decisions_note="not-decisions 标记 ${rest} 条，豁免 ${field} 处" ;;
    esac
  done <<<"$marker_report"
  if ((${#marker_problems[@]})); then
    for field in "${marker_problems[@]}"; do bad "$field"; done   # gate-lint:detail
    howto "标记写坏的照上面那一句改：点名 D 加 1–3 位数字、只点名它管的那几行里真有的串；有定义的决策号不许摘出去，引用处写成「D<n>（简称）」；" \
          "不起作用的标记删掉，或挪到那段输出正上方（标记与它要管的那几行之间不留空行）。写法与判法在 decision-refs 格上方的注释。"
  fi
  local reference kept=()
  for reference in "${kept_references[@]}"; do
    [[ -n "${exempt_numbers[$reference]+set}" ]] || kept+=("$reference")
  done
  kept_references=(${kept[@]+"${kept[@]}"})
}

cell_decision_refs() {
  cell_failed=0
  collect_scanned_files
  local missing=0 d
  local decision_refs=()
  local -A defined_decisions=()
  extract_numbered_references D decision_refs
  apply_not_decisions_markers decision_refs
  defined_numbers D "$KB/decisions" defined_decisions
  for d in "${decision_refs[@]}"; do
    [[ -n "${defined_decisions[$d]+set}" ]] || { bad "$d 被引用但 decisions/ 下没有它"
      howto "要么在 decisions/ 下给它建正文（\`## $d <简称>\` 起头），" \
            "要么把引用它的那处改成真实存在的决策号。"
      missing=1; }
  done
  if [[ ${#decision_refs[@]} -eq 0 ]]; then
    bad "一个决策号引用都没扫到（$scanned_note）——这一段没有对象可判"
    howto "确认门禁是在仓库根上跑的、$KB 目录在；" \
          "扫到 0 项不是通过（.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。"
  elif [[ $missing -eq 0 ]]; then
    ok "决策号引用全部有定义：${#decision_refs[@]} 个不同的决策号（$scanned_note）；$not_decisions_note"
  fi
  return "$cell_failed"
}

# ── declared-counts、invariant-count-elsewhere、invariant-anchors 共用：不变量表住在哪些文件里 ─────────────
# 表行形如 `| I-1.1 | 简称 | 陈述 | … |`；表里第 2 列是简称（singlefs-ai-sop/rules/kb-discipline.md 第 5 条），陈述在第 3 列。
# 不变量表搬出 .claude/kb/invariants.md 单文件（按类拆成 crates/<包>/invariants/ 这样的多文件）时，
# 登记项目根的 .claude/kb-directories（kb-discipline.md）；这三格不认死 invariants.md 这个文件名，
# 按「候选目录里，哪些 .md 文件有不变量表行」找文件。登记是**替换**不是追加（与 doc-lint.sh、history-ordinal.sh
# 读 kb_directories 的口径一致）：登记了就只认登记的那几个目录，仍要 .claude/kb 的话自己把它也列进登记表；没登记才默认 .claude/kb。
# 登记位的读法与规整只在 scripts/kb/quote-kb.py 的 kb_candidate_roots 一份（stale-candidates.py 也调它）：开头的 ./、结尾的 / 去掉，
# 绝对路径与 .. 走出仓的行拒、逐行打到 stderr（这一道的输出里看得见），没登记或登记的都不在、都被拒就是 .claude/kb。这里调它，不另抄一份读法。
kb_candidate_roots() {  # → 每行一个候选目录，相对项目根，去重（CWD 已经是项目根，见 ROOT 的 cd）
  if ! python3 - "$REPOSITORY/scripts/kb/quote-kb.py" "$PWD" <<'PY'
import importlib.util, sys
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("quote_kb", sys.argv[1])
quote_kb = importlib.util.module_from_spec(spec)
spec.loader.exec_module(quote_kb)
print("\n".join(quote_kb.kb_candidate_roots(sys.argv[2])))
PY
  then
    echo "  ✗ 读不出 kb 目录登记位（$REPOSITORY/scripts/kb/quote-kb.py 的 kb_candidate_roots 起不来），这一格的 kb 候选目录是空的" >&2
    echo "     → 怎么办：照上面 python 的报错改：登记位 .claude/kb-directories 写坏了（不是 UTF-8、读不了）就改它，读法本身坏了修 scripts/kb/quote-kb.py（读法只在那一份），再跑这一道。" >&2
    return 1
  fi
}

# 读登记位的六格（audit-contradictions、declared-counts、invariant-count-elsewhere、invariant-anchors、inline-sums、invariant-prerequisites）开头都调这一份：
# 登记位读不出（kb_candidate_roots 起不来）判红，不当成候选目录是空的——空的会一份文件都扫不到，多半退 77 或报「一个都没有」，与真没有分不开。
# 用法：kb_roots_or_red <格名> <当成空的会被误认成什么> <候选目录写到哪个文件>；读得出返回 0，读不出打红行与出路、返回 1。
# 弄坏开关 DOC_REGISTRIES_BREAK=kb-roots-failure-ignored：不看 kb_candidate_roots 的退出码，照空的往下判。
kb_roots_or_red() {
  if ! kb_candidate_roots > "$3" && [[ "$BREAK_SWITCH" != kb-roots-failure-ignored ]]; then
    bad "$1 没判：kb 目录登记位读不出（原因在上面），候选 kb 目录取不到，$2"
    howto "照上面那一句的出路修 .claude/kb-directories 或 scripts/kb/quote-kb.py，再只重跑这一格。"
    return 1
  fi
}

kb_candidate_md_files() {  # → 每行一个路径：候选目录下全部 .md 文件，按路径排序去重
  local root files=()
  while IFS= read -r root; do
    [[ -d "$root" ]] || continue
    while IFS= read -r -d '' f; do files+=("$f"); done < <(find "$root" -name '*.md' -type f -print0 2>/dev/null)
  done < <(kb_candidate_roots)
  ((${#files[@]})) || return 0
  printf '%s\n' "${files[@]}" | sort -u
}

invariant_table_files() {  # → 每行一个路径：候选目录下含不变量表行的 .md 文件，按路径排序去重
  local files=()
  while IFS= read -r f; do files+=("$f"); done < <(kb_candidate_md_files)
  ((${#files[@]})) || return 0
  grep -lE '^\| I-[0-9]+\.[0-9]+ ' "${files[@]}" 2>/dev/null | sort -u
}

# 设 inv_actual（总行）、inv_retired（退役）、inv_live（在用）、inv_files（数组：含不变量表行的文件，三语都在）、
# inv_counted_files（数组：数条数的那几份，只有中文原文）、inv_translation_files / inv_translation_rows（没数的译文份数与表行数，成功行报出来）。
# 条数只数中文原文：译文（<名>.en.md、<名>.ja.md，常见的是实验页、决策页的译文里抄着几行 `| I-…`）是同一份内容的另一种语言，
# 数进去就把一条算成两三条。取法走 lib-item-ref-status.py 的 same_language_files（与按编号取文件同一份判法，不在这里另认后缀）；
# 弄坏开关 KB_NUMBERED_FILE_BREAK=first-match 下不分语言、译文的表行一并数。inv_files 照旧三语都在：
# invariant-anchors 判这次改动写的每一行（译文里的也判），invariant-count-elsewhere 不扫带表行的文件，都用它。
# 判法库读不到、python 没跑成：退 1，两格判红，不退回不分语言去数。
count_invariant_rows() {
  inv_actual=0; inv_retired=0; inv_files=(); inv_counted_files=(); inv_translation_files=0; inv_translation_rows=0
  local f counted_listing rows
  while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    inv_files+=("$f")
  done < <(invariant_table_files)
  ((${#inv_files[@]})) || { inv_live=0; return 0; }
  if ! counted_listing="$(python3 - "$ITEM_REF_STATUS_LIBRARY" "${inv_files[@]}" <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("item_ref_status", sys.argv[1])
lib = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib)
for path in lib.same_language_files(sys.argv[2:]):
    print(path)
PY
  )"; then
    inv_live=0
    return 1
  fi
  while IFS= read -r f; do
    [[ -n "$f" ]] && inv_counted_files+=("$f")
  done <<<"$counted_listing"
  for f in "${inv_files[@]}"; do
    rows=$(grep -cE '^\| I-[0-9]+\.[0-9]+ ' "$f")
    if grep -qxF -- "$f" <<<"$counted_listing"; then
      inv_actual=$(( inv_actual + rows ))
      inv_retired=$(( inv_retired + $(grep -cE '^\| I-[0-9]+\.[0-9]+ \| [^|]* \| \*\*此编号不再使用' "$f") ))
    else
      inv_translation_files=$(( inv_translation_files + 1 )); inv_translation_rows=$(( inv_translation_rows + rows ))
    fi
  done
  inv_live=$(( inv_actual - inv_retired ))
}

# count_invariant_rows 没跑成时两格同一句红行
report_invariant_count_failure() {
  bad "按语言挑数条数那几份文件的 python 没跑成（判法 $ITEM_REF_STATUS_LIBRARY 的 same_language_files）——不变量条数没数，这一格没判"
  howto "按上面的 traceback 修（判法库在不在、能不能 import）；没数成就是没判，不许退回不分语言去数（译文的表行会被多数一遍）。"
}

# 没数的译文：成功行与红行在括号里报出来（一份都没有时不报）
inv_translation_note() {
  ((inv_translation_files)) && printf '；只数中文原文，译文 %s 份里的 %s 行表行不数' "$inv_translation_files" "$inv_translation_rows"
  return 0
}

# 候选文件只有一份且叫 invariants.md 时，消息里仍称呼它的文件名（历史惯例）；拆成多份时改叫「不变量表」。份数只数中文原文（数条数的那几份）。
inv_table_label() {
  if [[ ${#inv_counted_files[@]} -eq 1 && "$(basename "${inv_counted_files[0]}")" == invariants.md ]]; then
    printf '%s' "invariants.md"
  else
    printf '%s' "不变量表（${#inv_counted_files[@]} 份文件，登记在 .claude/kb-directories）"
  fi
}

cell_declared_counts() {
  cell_failed=0
  # 登记位读不出判红，不当成候选目录是空的（kb_roots_or_red）：不变量条数这一半没判，欠账表条数那一半照数
  if ! kb_roots_or_red declared-counts "不是一个带不变量表行的文件都没扫到" "$work_directory/declared-counts.roots"; then
    :
  elif ! count_invariant_rows; then
    report_invariant_count_failure
  elif [[ ${#inv_files[@]} -gt 0 && ${#inv_counted_files[@]} -eq 0 ]]; then
    bad "候选 kb 目录里带不变量表行的 ${#inv_files[@]} 份文件全是译文，没有一份中文原文：在用条数没有可数的（条数只数中文原文）"
    howto "不变量表的中文原文 <名>.md 搬了家就按 .claude/rules/path-moves.md 改登记（.claude/kb-directories）；译文先有、中文原文缺的，先补中文原文。"
  elif [[ ${#inv_files[@]} -eq 0 ]]; then
    bad "候选 kb 目录里一个带不变量表行的文件都没扫到（表行形如 \`| I-1.1 | 简称 | 陈述 | … |\`）"
    howto "确认门禁是在仓库根上跑的；文件搬了家的话，登记项目根的 .claude/kb-directories（kb-discipline.md），" \
          "或按 .claude/rules/path-moves.md 把这里的路径一起改；写法真改了的话，改这里的 count_invariant_rows（另两格用同一份）。"
  else
    local inv_label marker_hits=() f marker_line_number claim_line inv_claim marker_file
    inv_label="$(inv_table_label)"
    # 当前条数的权威登记位是 <!-- invariant-count --> 下一行那句：历史版本里也有「现共 N 条在用」，那是当时的数、不跟着改，
    # 按文件序取第一处会取到历史里的那一句。标记丢了、不止一处、或下一行读不出那句，都判红，不退回去取第一处。
    # 标记可以住在候选目录里**任一** .md 文件（拆分成多文件时多半落在它们共同的 README.md），不限于带表行的那几份。
    while IFS= read -r f; do
      grep -qF -- '<!-- invariant-count -->' "$f" && marker_hits+=("$f")
    done < <(kb_candidate_md_files)
    if [[ ${#marker_hits[@]} -eq 0 ]]; then
      bad "$inv_label 里没有 <!-- invariant-count --> 标记——当前条数的登记位丢了，读不出正文声称几条（表里在用 $inv_live 条）"
      howto "在「现共 N 条在用」那句的上一行补回 <!-- invariant-count -->；" \
            "不许让检查退回去取文件里第一处「N 条在用」：历史版本里那几句记的是当时的数。"
    elif [[ ${#marker_hits[@]} -gt 1 ]]; then
      bad "$inv_label 里有 ${#marker_hits[@]} 处 <!-- invariant-count --> 标记，登记位该只有一处：${marker_hits[*]}"
      howto "只留一处权威登记位（拆分成多文件时该在它们共同的 README.md 里），其余那句删掉或改成不带标记的历史陈述。"
    else
      marker_file="${marker_hits[0]}"
      marker_line_number=$(grep -n -m1 -F '<!-- invariant-count -->' "$marker_file" | cut -d: -f1)
      claim_line=$(sed -n "$((marker_line_number + 1))p" "$marker_file")
      inv_claim=$(grep -oE '现共 [0-9]+ 条在用' <<<"$claim_line" | grep -oE '[0-9]+' | head -1)
      if [[ -z "$inv_claim" ]]; then
        bad "$marker_file 第 $((marker_line_number + 1)) 行（<!-- invariant-count --> 的下一行）读不出「现共 N 条在用」：${claim_line:0:60}"
        howto "标记的下一行就写那句「现共 $inv_live 条在用（…）」，中间不空行；" \
              "读不出声明就是没核过，不许当成一致。"
      elif [[ "$inv_claim" != "$inv_live" ]]; then
        bad "$inv_label 正文声称在用 $inv_claim 条，实际 $inv_live 条（总行 $inv_actual，退役 $inv_retired$(inv_translation_note)）"
        howto "把正文那句「现共 N 条在用」改成 $inv_live，或者补回漏掉的那几条——" \
              "两个数对不上时，读的人不知道该信哪一个。"
      else
        ok "不变量条数一致：正文声称 $inv_claim 条在用，表里在用 $inv_live 条（总行 $inv_actual，退役 $inv_retired$(inv_translation_note)）"
      fi
    fi
  fi
  # 三段各数各的：欠着的那张表、「### 已还清」那张表、「## 历史版本」里的条目。
  # 分界线是「### 已还清」，不是「## 历史版本」：已还清那张表住在历史版本之前，按历史版本切会把还清的全算进欠账里。
  # 数不出来（文件不在、awk 出错、两张表一行都没数到）判红：两个空白或两个 0 印在成功行里，看着就像数过了。
  if [[ ! -f "$KB/checks-owed.md" ]]; then
    bad "找不到 $KB/checks-owed.md，欠账条数取不到"
    howto "确认门禁是在仓库根上跑的；文件真搬了家的话，按 .claude/rules/path-moves.md 把这里的路径一起改。"
  else
    # 开着与已还清的切法用共用读法 lib-owed.py，不在这里再抄一份 awk
    local chk_counts="" chk_actual chk_done chk_heading
    if chk_counts=$(python3 - "$GATE_DIRECTORY/lib-owed.py" "$KB/checks-owed.md" <<'PY_OWED'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("lib_owed", sys.argv[1])
lib_owed = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib_owed)
table = lib_owed.read_owed_table(sys.argv[2])
print(len(table.open_names), len(table.paid_names), int(table.paid_heading_found))
PY_OWED
    ); then :; else chk_counts=""; fi
    read -r chk_actual chk_done chk_heading <<<"$chk_counts"
    if [[ -z "${chk_actual:-}" || -z "${chk_done:-}" ]]; then
      bad "共用读法 lib-owed.py 没数出 checks-owed.md 的条数（输出「$chk_counts」）"
      howto "单独跑一遍上面那段 python 看它报什么错；数不出来就是没核过，不许当成数过了。"
    elif [[ $((chk_actual + chk_done)) -eq 0 ]]; then
      bad "checks-owed.md 里一行 \`| C<n> \` 都没数到（欠着 0、已还清 0）——欠账表的写法变了，或这份文件是空的"
      howto "欠账行形如 \`| C12 | 简称 | … |\`；写法真改了的话，改 .claude/gate.d/lib-owed.py 的正则，doc-registries、checker-independence-and-sync、code-source-discipline 与这里一起跟上。"
    elif [[ "${chk_heading:-0}" != 1 ]]; then
      bad "checks-owed.md 里认不出「已还清」标题——开着的与已还清的分不开，$chk_actual 条全被算成欠着"
      howto "已还清那张表上面要有一行「### 已还清」；标题改过名的话，改 .claude/gate.d/lib-owed.py 认标题的那条正则。"
    else
      ok "欠检查 $chk_actual 条、已还清 $chk_done 条（checks-owed.md）"
    fi
  fi
  return "$cell_failed"
}

cell_governance_refs() {
  cell_failed=0
  local governance_rc=0 governance_output governance_count_text governance_checked=0 governance_base="" governance_kb_line="" governance_section_line=""
  local -a governance_count_lines=() governance_base_arguments=()
  # 滞后表 dead-path-lag.tsv 与 gate 基准那一版比只缩不涨：基准用共用库 scripts/gate/changed-paths.sh 的 gate 取法；不在 git 仓里不给，lib 那一侧报「没比」
  if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    # shellcheck source=../../scripts/gate/changed-paths.sh
    source "$REPOSITORY/scripts/gate/changed-paths.sh" || { echo "  ✗ 读不到共用库 $REPOSITORY/scripts/gate/changed-paths.sh"; echo "     → 怎么办：gate 基准的取法只有那一份，恢复它，别在阶段里再抄一份。"; return 1; }
    governance_base="$(gate_diff_base gate)"
    governance_base_arguments=(--base "$governance_base")
  fi
  governance_output="$(python3 "$GATE_DIRECTORY/lib-governance-refs.py" ${governance_base_arguments[@]+"${governance_base_arguments[@]}"})" || governance_rc=$?
  [[ -z "$governance_output" ]] || printf '%s\n' "$governance_output"
  if [[ $governance_rc -eq 0 ]]; then
    # 查了多少项只从 lib-governance-refs.py 末行「共查 T 项（…）」取，这里不另数
    mapfile -t governance_count_lines < <(printf '%s\n' "$governance_output" | grep -E '^  共查 [0-9]+ 项（' || true)
    if [[ ${#governance_count_lines[@]} -ne 1 ]]; then
      bad "lib-governance-refs.py 退 0，末行「共查 T 项（…）」却出现了 ${#governance_count_lines[@]} 次（应当恰好一次）——查了多少项不知道，这一段没判"
      howto "单独跑 python3 $GATE_DIRECTORY/lib-governance-refs.py 看末行；打不出计数就是没判，不许当成判过了（改了输出格式的话，这里的正则跟着改）。"
    else
      governance_count_text="${governance_count_lines[0]#  共查 }"
      governance_checked=$((10#${governance_count_text%% 项*}))
      if [[ $governance_checked -eq 0 ]]; then
        bad "lib-governance-refs.py 共查 0 项——治理文档里没有一处门禁号、路径、小节、全名或格名可判，扫到 0 项不是通过"
        howto "确认门禁是在仓库根上跑的、治理文档与 kb 在 CARRIER_PATTERNS 与 KB_PATTERN 射程里；写法真变了的话，改 lib-governance-refs.py 认指向的那几条正则。"
      else
        # 成功句另抄 lib 打的「kb 路径：」那一行（kb 里判了几处、research/ 放行几处、滞后几处、保留几处，相对简写、git 历史命令 -- 之后、说搬迁的行各跳过几处）
        # 与「kb 小节：」那一行（kb 里 `文件「小节」` 判了几处、滞后几处、保留几处，文件找不到几处）
        governance_kb_line="$(printf '%s\n' "$governance_output" | sed -n 's/^  kb 路径：//p' | head -n 1)"
        governance_section_line="$(printf '%s\n' "$governance_output" | sed -n 's/^  kb 小节：//p' | head -n 1)"
        ok "治理文档与 kb 里的门禁号、门禁全名与格名、路径与「小节」都指得到，没有用旧编号称呼门禁（共查 $governance_checked 项${governance_kb_line:+；kb 路径：$governance_kb_line}${governance_section_line:+；kb 小节：$governance_section_line}）"
      fi
    fi
  elif [[ $governance_rc -eq 3 ]]; then
    bad "一份治理文档都没扫到——这一段没有对象可判"
    howto "确认门禁是在仓库根上跑的；治理文档搬了家的话，改 lib-governance-refs.py 的 CARRIER_PATTERNS。"
  elif [[ $governance_rc -ne 1 ]]; then
    bad "lib-governance-refs.py 没跑成（退出码 $governance_rc）——这一段没判"
    howto "单独跑 python3 $GATE_DIRECTORY/lib-governance-refs.py 看它报什么错；没跑成就是没判，不许当成判过了。"
  else
    bad "治理文档与 kb 里有指不到的指向（上面逐条列出）"
    howto "旧编号与全名：门禁一律按门禁目录顶层的文件名去掉 .sh 称呼，旧编号照红行给的今天的名字改，格名用那道门禁的 --list 看；" \
          "门禁号：阶段被删或收归上游的，改成共享 gate.sh 里那一道的名字（「链接指向」这类）或现存的号；" \
          "路径：照红行冒号后面的后继结论改（改成后继；删了没后继的把现状句改写或删去，说那一次的事件句写成 \`<提交>^:<路径>\`），已归档的写裸文件名并指到取法（.claude/agent-common.md「找不到历史实验的数据」那一条）；" \
          "滞后表 .claude/gate.d/dead-path-lag.tsv 只缩不涨，新写的不在的路径当场改、不往里加，红行说「这一行该删」的删掉那一行；" \
          "照规矩该留旧路径的（将来落点、句子说它不在、抄下来的命令或输出、仓外副本里的路径），登进 .claude/gate.d/dead-path-kept.tsv（可以增长，理由至少 8 个字写它属于哪种，同一对不许两张表都登）；" \
          "小节：照红行冒号后面给的那份文件里最像的标题或句子改，那份文件里没有的改写那一句或删去；小节滞后表 .claude/gate.d/dead-section-lag.tsv 只缩不涨，新写的找不到的小节当场改、不往里加；" \
          "照规矩该留的小节（句子本身说那一句已改掉、抄下来的当时的原文）登进 .claude/gate.d/dead-path-kept.tsv，第二列写 <文件>「<小节>」。改 agent 定义与共用约束照走门禁 doc-process-records 的 agent-def-adversarial-review 格那一条。"
  fi
  return "$cell_failed"
}

# ── 格 invariant-count-elsewhere：同一个量被别的文件抄过去之后，declared-counts 只看 invariants.md 自己，就没有东西在对了。
# 查的是：`invariants.md` 之外的 kb 文件，正文里写「N 条在用」或「N 条不变量在用」的地方，N 必须等于实际在用条数。
# 一处这样的声明都没有，退 77。抓不到的：
#   1. 不带「在用」二字的写法（「66 条不变量」）：多半在复述某一轮普查当时的口径，照现值判会误判。要它被管住，就把话写成「N 条在用」。
#   2. `## 历史版本` 之后的内容整段跳过（旧数留在那里是 `kb-discipline.md` 第 8 条要求的）。
#   3. `*-history.md` 与 `decisions-history/` 下整份跳过：那些文件通篇是历史。
cell_invariant_count_elsewhere() {
  cell_failed=0
  # 登记位读不出判红，不当成候选目录是空的：空的会一份带表行的文件都扫不到、退 77（kb_roots_or_red）
  kb_roots_or_red invariant-count-elsewhere "不是一份带表行的文件都没有" "$work_directory/invariant-count-elsewhere.roots" || return 1
  if ! count_invariant_rows; then
    report_invariant_count_failure
    return 1
  fi
  if [[ ${#inv_files[@]} -eq 0 ]]; then
    echo "  ! 候选 kb 目录里一个带不变量表行的文件都没扫到，这一格本次无对象可判"
    return 77
  fi
  if [[ ${#inv_counted_files[@]} -eq 0 ]]; then
    bad "候选 kb 目录里带不变量表行的 ${#inv_files[@]} 份文件全是译文，没有一份中文原文：实际在用条数没有可数的（条数只数中文原文）"
    howto "不变量表的中文原文 <名>.md 搬了家就按 .claude/rules/path-moves.md 改登记（.claude/kb-directories）；译文先有、中文原文缺的，先补中文原文。"
    return 1
  fi
  local inv_label
  inv_label="$(inv_table_label)"
  printf '%s\n' "${inv_files[@]}" > "$work_directory/invariant-count-elsewhere.inv_files"
  local elsewhere_rc=0
  # 实际在用条数只数中文原文（count_invariant_rows）；译文里写的「N 条在用」照扫，与中文原文数出的同一个数比
  python3 - "$inv_live" "$inv_label" "$work_directory/invariant-count-elsewhere.inv_files" "$work_directory/invariant-count-elsewhere.roots" "$(inv_translation_note)" <<'PY' || elsewhere_rc=$?
import re, sys, os, glob

live = int(sys.argv[1])
label = sys.argv[2]
inv_paths = set(os.path.abspath(l.strip()) for l in open(sys.argv[3], encoding="utf-8") if l.strip())
roots = [l.strip() for l in open(sys.argv[4], encoding="utf-8") if l.strip()]
translation_note = sys.argv[5].lstrip("；")

scanned, hits, bad = 0, [], []
for root in roots:
    for p in sorted(glob.glob(os.path.join(root, "**", "*.md"), recursive=True)):
        if os.path.abspath(p) in inv_paths or os.path.basename(p).endswith("-history.md") or "/decisions-history/" in p:
            continue
        scanned += 1
        body = open(p, encoding="utf-8").read().split("\n## 历史版本")[0]
        # 「条」与「在用」之间许夹「不变量」三个字（「现有 26 条不变量在用」）；别的名词不认，免得把别的量读成不变量条数
        for m in re.finditer(r"(\d+)\s*条(?:不变量)?\s*在用", body):
            n = int(m.group(1))
            hits.append((p, n))
            if n != live:
                line = body[:m.start()].count("\n") + 1
                bad.append(f"{p}:{line} 写「{m.group(0)}」，而 {label} 实际在用 {live} 条")

if bad:
    print(f"  ✗ 不变量条数跨文件对不上 {len(bad)} 处：")      # gate-lint:summary
    for b in bad:
        print("      " + b)                                   # gate-lint:detail
    print(f"  → 怎么办：以 {label} 为准，把那句里的数改成 {live}；")
    print("    若那句说的其实是某一轮普查当时的口径（不是现值），就把「在用」二字去掉并写明是哪一轮，")
    print("    否则下一个检索到它的人会拿一个旧数当现状。")
    sys.exit(1)

if not hits:
    print(f"  ! 本次无对象可判：扫了 {scanned} 份 kb 文件（{label} 与变更史之外），没有一处写「N 条在用」的声明")
    sys.exit(77)
print(f"  ✓ 不变量条数跨文件一致：实际在用 {live} 条，扫了 {scanned} 份 kb 文件、命中 {len(hits)} 处声明"
      + (f"（实际条数{translation_note}；译文里写的「N 条在用」照扫）" if translation_note else ""))
PY
  if [[ $elsewhere_rc -ne 0 && $elsewhere_rc -ne 1 && $elsewhere_rc -ne 77 ]]; then
    bad "数「N 条在用」的那段 python 没跑成（退出码 $elsewhere_rc）——这一格没判"
    howto "按上面的 traceback 修；没跑成就是没判，不许当成判过了。"
    return 1
  fi
  return "$elsewhere_rc"
}

# ── 格 citations：还 checks-owed.md C38（外部文献不可复核）的源码那一半。
# 要拦的是证据蒸发（路径变了、版本升了、树被删了），不是 kb 写错；红了怎么处置见 verify-citations.sh 自己打印的下一步。
# 源码树不在也判红，不当跳过：跳过正是让上一批文献无声蒸发的那个行为。
# 真 verify-citations.sh 的判别力不在这里测：它读共享数据根下的两棵源码树（.claude/hooks/lib_work_root.py 的 fs-refs、kernel-tree，FS_REFS、KERNEL_TREE 设了照它们），装不进密封的样本目录；
# 它自己的 --selftest 合成两棵假树测，由 code-tooling 跑。
# 复核最慢（本机内核树整棵 grep），阶段在父进程里、各格开始之前起到后台（文件末尾「提前起 citations」一段），别的格跑的时候它也在跑；
# 格在自己的子 shell 里，等不到父进程起的后台作业（wait 报不是自己的子进程），所以后台作业把退出码写进 citations.rc，格用 tail --pid 等它退出再读那个文件。
CITATIONS_SCRIPT=scripts/kb/verify-citations.sh
citations_pid=""
start_citations() {
  [[ -f "$CITATIONS_SCRIPT" ]] || return 0
  { bash "$CITATIONS_SCRIPT" > "$work_directory/citations.out" 2>&1; echo "$?" > "$work_directory/citations.rc.tmp"; mv "$work_directory/citations.rc.tmp" "$work_directory/citations.rc"; } &
  citations_pid=$!
}
cell_citations() {
  cell_failed=0
  if [[ ! -f "$CITATIONS_SCRIPT" ]]; then
    echo "  ✗ 找不到 $CITATIONS_SCRIPT"
    echo "     → 怎么办：这一格靠它逐条复核引文。文件被挪走就改这里的路径，"
    echo "               还没写就先写它——缺了它，引文一条都没被验过。"
    return 1
  fi
  local citations_rc=0
  if [[ -n "$citations_pid" ]]; then
    tail --pid="$citations_pid" -s 0.2 -f /dev/null
    if [[ -f "$work_directory/citations.rc" ]]; then
      citations_rc="$(<"$work_directory/citations.rc")"
    else
      citations_rc=1
      bad "外部引用复核没写出退出码：$CITATIONS_SCRIPT 起在后台、退出时没留下 citations.rc（被信号杀掉、临时目录没了都会这样）"
      howto "再跑一次这一格；还是没有就单独跑 bash $CITATIONS_SCRIPT 看它自己打印的报错。"
    fi
  else
    bash "$CITATIONS_SCRIPT" > "$work_directory/citations.out" 2>&1 || citations_rc=$?
  fi
  [[ "$BREAK_SWITCH" == citations-exit-ignored ]] && citations_rc=0      # 弄坏：复核退非 0 也不判红
  cat "$work_directory/citations.out"
  if [[ $citations_rc -ne 0 ]]; then
    bad "外部引用复核没过：$CITATIONS_SCRIPT 退 $citations_rc"
    howto "按上面 verify-citations.sh 自己打印的下一步处置；未命中不等于 kb 写错，也可能是源码固定点没了或版本变了。"
  fi
  return "$cell_failed"
}

# ── 格 invariant-anchors：这次改动在不变量表（invariant_table_files 找到的那几份，今天是 `crates/singlefs-checker/invariants/0*.md`）里新增或改写的每一行不变量（`| I-x.y | …`），
# 正文里必须至少点名一条 `D<n>（简称） 已定项 k`，那是这条不变量要判的字段的落点；点不出来判红。正文写着「此编号不再使用」的不判。
# 判不了的不变量比没有这条更糟：checker 会照跑、照报绿（C69（已定不变量没有字段可判））。
# 射程只到这次改动的行：存量里点不出分项的行一次全判会挡住每一次提交；存量那笔账仍欠着（C69（已定不变量没有字段可判）），
# 所以成功那句每轮都把存量的条数现算着报出来。不判点名的那条分项里到底有没有这个字段、宽度对不对。
# 改动范围：GATE_BASE 给了就与它比，否则与 @{upstream} 的 merge-base 比，都没有就与 HEAD 比；工作区、暂存区与未跟踪文件都算
# （基准用共用库 scripts/gate/changed-paths.sh 的 gate 取法）。两份 diff 任一份 git 失败判红，不当成「一行没改」。
# 这次改动没有新写或改写任何不变量行，退 77（存量条数照样报出来）。
cell_invariant_anchors() {
  cell_failed=0
  git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ! $PWD 不是 git 仓，这一格本次无对象可判"; return 77; }
  # 登记位读不出判红，不当成候选目录是空的：空的会一份不变量表都扫不到、退 77（kb_roots_or_red）
  kb_roots_or_red invariant-anchors "不是一个带不变量表行的文件都没扫到" "$work_directory/invariant-anchors.roots" || return 1
  count_invariant_rows
  if [[ ${#inv_files[@]} -eq 0 ]]; then
    echo "  ! 候选 kb 目录里一个带不变量表行的文件都没扫到，这一格本次无对象可判"
    return 77
  fi
  local LIB_CHANGED_PATHS base added added_path f is_new anchors_rc=0
  LIB_CHANGED_PATHS="$(cd "$GATE_DIRECTORY/../.." && pwd)/scripts/gate/changed-paths.sh"
  # shellcheck source=../../scripts/gate/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用库 $LIB_CHANGED_PATHS"; echo "     → 怎么办：改动范围的取法只有那一份，恢复它，别在阶段里再抄一份。"; return 1; }
  base="$(gate_diff_base gate)"
  added="$(gate_changed_paths "$base" untracked A)" || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    return 1
  }
  {
    for f in "${inv_files[@]}"; do
      is_new=0
      while IFS= read -r added_path; do
        [[ "$added_path" == "$f" ]] && { is_new=1; break; }
      done <<<"$added"
      printf '%s\t%s\n' "$f" "$is_new"
    done
  } > "$work_directory/invariant-anchors.files"
  python3 - "$base" "$work_directory/invariant-anchors.files" <<'PY' || anchors_rc=$?
import re, subprocess, sys

base = sys.argv[1]
entries = []
for raw_line in open(sys.argv[2], encoding="utf-8"):
    raw_line = raw_line.rstrip("\n")
    if not raw_line:
        continue
    path, flag = raw_line.rsplit("\t", 1)
    entries.append((path, flag == "1"))
ROW = re.compile(r"^\|\s*(I-[\d.]+)\s*\|")
ITEM = re.compile(r"D\d+（[^）]+）\s*已定项\s*\d+")
RETIRED = "此编号不再使用"

def anchored(line):
    return ITEM.search(line) is not None

# 这次改动新增或改写的行：diff 的 `+` 侧。基准那一版没有这个文件时（新建，含未跟踪），整份都算新写。
# 两份 diff 会重叠（改动已暂存、工作区没再动时两份都有同一行），同一行只算一次，别让报出来的行数翻倍。
# 不变量表拆成几份文件时，逐份各自取「+」侧，再并到同一份 touched 里。
touched = []
for path, file_is_new in entries:
    if file_is_new:
        candidate_lines = open(path, encoding="utf-8").read().split("\n")
    else:
        git_diff = ["git", "-c", "core.quotepath=false", "diff", "-U0"]
        diffs = []
        for arguments in ([base, "--", path], ["--cached", "--", path]):
            shown = subprocess.run([*git_diff, *arguments], capture_output=True, text=True)
            if shown.returncode != 0:
                # git 失败时 + 行是空的：当成「这次没改不变量」就会走到无对象可判，把新写的行整批放过去
                print(f"  ✗ 取不到 {path} 这次改动的 diff（git diff {' '.join(arguments[:-2]) or '基准'} 退 {shown.returncode}：{shown.stderr.strip()[:120]}）")
                print("     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、对象库与索引没坏）再跑；取不到 diff 时这一格什么都没比，不是通过。")
                sys.exit(1)
            diffs.append(shown.stdout)
        diff, staged = diffs
        candidate_lines = [line[1:] for chunk in (diff, staged) for line in chunk.split("\n")
                           if line.startswith("+") and not line.startswith("+++")]
    for body in candidate_lines:
        if ROW.match(body) and RETIRED not in body and body not in touched:
            touched.append(body)

missing = [body for body in touched if not anchored(body)]

# 存量：全部候选文件里点不出分项的行，现算，每轮报出来
total, unanchored_all = 0, 0
for path, _ in entries:
    for line in open(path, encoding="utf-8"):
        if ROW.match(line) and RETIRED not in line:
            total += 1
            if not anchored(line):
                unanchored_all += 1

if missing:
    print(f"  ✗ 这次改动新写或改写的 {len(missing)} 行不变量，没有点名它判的字段住在哪条已定分项（基准 {base}）：")  # gate-lint:summary
    for body in missing:
        identifier = ROW.match(body).group(1)
        print(f"      {identifier}：{body.strip()[:120]}")  # gate-lint:detail
    print("     → 怎么办：在那一行正文里写清这条不变量判的字段住哪，形态是「D<编号>（简称） 已定项 k」。")
    print("               写不出来，说明这条不变量今天判不了任何字节——那正是 C69（已定不变量没有字段可判） 记着的那一类：")
    print("               checker 会老老实实跑它、老老实实报绿。先补字段（在那条决策里定出落点与宽度），或者把这条不变量降级。")
    sys.exit(1)

if not touched:
    print(f"  ⊘ 本次无对象可判：这次改动没有新写或改写不变量行（基准 {base}）；存量 {total} 行里 {unanchored_all} 行还点不出已定分项，那笔账记在 C69（已定不变量没有字段可判）")
    sys.exit(77)
print(f"  ✓ 这次改动新写或改写的 {len(touched)} 行不变量都点名了已定分项（基准 {base}）；"
      f"存量 {total} 行里 {unanchored_all} 行还点不出，那笔账记在 C69（已定不变量没有字段可判）")
PY
  if [[ $anchors_rc -ne 0 && $anchors_rc -ne 1 && $anchors_rc -ne 77 ]]; then
    bad "判不变量字段落点的那段 python 没跑成（退出码 $anchors_rc）——这一格没判"
    howto "按上面的 traceback 修；没跑成就是没判，不许当成判过了。"
    return 1
  fi
  return "$anchors_rc"
}

# ── 格 inline-sums：还 checks-owed.md C305（kb 里的内联算式没人加）。field-table-sums 只看「合计 N 字节」那种表后合计，行内「84 = 42 + 8 + …」一格都盖不到。
# 判据：扫 kb 候选目录（.claude/kb-directories 登记的目录，没登记就是 .claude/kb）下全部 .md 的正文，每一行里的算式
#   左边：一个连续数字的整数（可带粗体 **，可带「字节」「B」），紧挨它左边不许是字母数字、小数点或 + × ÷ / − – - = ^ 这类运算符；
#   右边：= 之后由 + 连接的两项以上，每项是「可选标签（中文、英文单词、反引号里的名字）加一个整数」，整数后可带「字节」「B」；
#   算完与左边不等判红，打印文件:行号、算式、左右两边的数与那一行。
# 不判、只数进成功句的：整条算式里出现 × ÷ * / − – - ^ ( （ 或后面又接 =（连等），报「跳过 K 条含乘除减括号的」；
#   整数后带小数、千分位、KiB 这类别的单位或认不出的写法，另报一个数。
# 不扫：「## 历史版本」之后、代码围栏里的行、*-history.md 与 decisions-history/ 下的文件（与 invariant-count-elsewhere 同一个口径）。一行里写了几处算式各判各的。
# 一条算式都判不到（或一份 kb 文件都扫不到）退 77，不当通过。
# 样本：inline-sums-red 放左右不等的几种写法（纯数字、带标签、带「字节」、粗体左边、登记目录里的类文件），逐条点名判红；
# inline-sums-green 同样的式子改对，另放含 ×、含括号、带 KiB、连等、代码围栏里、历史节里、*-history.md 里的错式，一条都不判，报判了几条、跳过几条。
# 弄坏开关 DOC_REGISTRIES_BREAK=inline-sums（整格不跑）与 inline-sums-unchecked（左右不等也不判）、inline-sums-operators-judged（含乘除括号的也只加能读出的项）、
# inline-sums-fences-scanned、inline-sums-history-scanned、inline-sums-history-files-scanned：写法与对应的样本在文件头弄坏开关一节。
cell_inline_sums() {
  cell_failed=0
  # 登记位读不出判红，不当成候选目录是空的（空的会扫 0 份、退 77；kb_roots_or_red）。候选目录写进的这份文件下面那段 python 接着读
  kb_roots_or_red inline-sums "不是一条算式都没有" "$work_directory/inline-sums.roots" || return 1
  local sums_rc=0
  python3 - "$work_directory/inline-sums.roots" "$BREAK_SWITCH" <<'PY' || sums_rc=$?
import glob, os, re, sys

roots = [l.strip() for l in open(sys.argv[1], encoding="utf-8") if l.strip()]
BREAK = sys.argv[2]
FENCE = re.compile(r"^\s*(```|~~~)")
HISTORY = re.compile(r"^##\s*历史版本")
# 数字紧挨的左边：编号、小数、版本号、变量名里的数不是算式的左边
LEFT = re.compile(r"(?<![A-Za-z0-9_.])(\*\*)?(\d+)(\*\*)?[ ]*(?:字节|B(?![A-Za-z0-9_]))?[ ]*=(?![=>])")
LIST_MARK = re.compile(r"\s*(?:[-*+]|\d+[.)])")
OPERATORS_BEFORE = "+×÷/−–-=^<>~≈≠≤≥"
OPERATORS_AFTER = "×÷*/−–-^(（=<>≈"
OPERATORS_OPENING = "(（−–-"
LABEL_WORD = r"(?:[\u4e00-\u9fffA-Za-z_]+|`[^`\n]*`)"
LABEL = rf"{LABEL_WORD}(?: +{LABEL_WORD})*"
TERM = re.compile(rf" *(?:(?P<label>{LABEL})(?P<gap> *))?\*{{0,2}}(?P<int>\d+)\*{{0,2}}(?:[ ]*(?:字节|B(?![A-Za-z0-9_])))?")
PLUS = re.compile(r" *\+(?!\+)")
CLAUSE_END = "，。；、;|`"

scanned = judged = skipped_operators = skipped_unrecognized = 0
bad = []

def kb_files():
    seen = set()
    for root in roots:
        for path in sorted(glob.glob(os.path.join(root, "**", "*.md"), recursive=True)):
            history_file = os.path.basename(path).endswith("-history.md") or "/decisions-history/" in path
            if path in seen or (history_file and BREAK != "inline-sums-history-files-scanned"):
                continue
            seen.add(path)
            yield path

def current_lines(path):
    in_fence = False
    for number, line in enumerate(open(path, encoding="utf-8", errors="replace").read().split("\n"), 1):
        if FENCE.match(line):
            in_fence = not in_fence if BREAK != "inline-sums-fences-scanned" else False
            continue
        if in_fence:
            continue
        if HISTORY.match(line) and BREAK != "inline-sums-history-scanned":
            return
        yield number, line

def classify(line, start):
    """解析 start（等号之后）起的右边：→ (类, 各项, 右边结束位置)。类：sum、ops、unrecognized、none（不是这个形态）。"""
    terms, index, plus_seen = [], start, False
    while True:
        term = TERM.match(line, index)
        glued = term and term.group("label") and not term.group("gap") and term.group("label")[-1] != "`" and term.group("label")[-1].isascii()
        if not term or glued:
            rest = line[index:].lstrip(" ")
            if not terms:
                return ("ops" if rest[:1] and rest[0] in OPERATORS_OPENING and "+" in chunk_of(rest) else "none"), terms, index
            return "unrecognized", terms, index
        value, end = int(term.group("int")), term.end()
        tail = line[end:]
        follow = tail.lstrip(" ")[:1]
        terms.append(value)
        plus = PLUS.match(line, end)
        if plus:
            index, plus_seen = plus.end(), True
            continue
        has_plus = plus_seen or "+" in chunk_of(tail)
        if follow and follow in OPERATORS_AFTER:
            return ("ops" if has_plus else "none"), terms, end
        if re.match(r"\.\d|,\d{3}(?!\d)|[%^~_]|[A-Za-z]", tail.lstrip(" ")):
            return ("unrecognized" if has_plus else "none"), terms, end
        return ("sum" if len(terms) >= 2 else "none"), terms, end

def chunk_of(text):
    for position, character in enumerate(text):
        if character in CLAUSE_END:
            return text[:position]
    return text

for path in kb_files():
    scanned += 1
    for number, line in current_lines(path):
        for left in LEFT.finditer(line):
            before = line[:left.start()].rstrip(" ")
            if before and not LIST_MARK.fullmatch(before) and before[-1] in OPERATORS_BEFORE:
                continue
            if line[left.start() - 1:left.start()] == "," and left.start() >= 2 and line[left.start() - 2].isdigit():
                continue
            kind, terms, end = classify(line, left.end())
            if kind == "ops" and BREAK == "inline-sums-operators-judged":
                kind = "sum"
            if kind == "ops":
                skipped_operators += 1
            elif kind == "unrecognized":
                skipped_unrecognized += 1
            elif kind == "sum":
                judged += 1
                want, got = int(left.group(2)), sum(terms)
                if want != got and BREAK != "inline-sums-unchecked":
                    bad.append((path, number, line[left.start():end].strip(), want, got, line))

if bad:
    print(f"  ✗ kb 正文里 {len(bad)} 条内联算式左右不等（扫 {scanned} 份、判了 {judged} 条）：")  # gate-lint:summary
    shown = set()
    for path, number, expression, want, got, line in bad:
        print(f"      {path}:{number} 「{expression}」左边是 {want}，右边是 {got}（右边比左边{'多' if got > want else '少'} {abs(got - want)}）")  # gate-lint:detail
        if (path, number) not in shown:
            shown.add((path, number))
            print(f"          原行：{line}")  # gate-lint:detail
    print("  → 怎么办：按右边各项重新加一遍，是合计写错了就改合计，是右边漏了或多了一项就补上或去掉那一项；")
    print("    这一行其实不是在求和（右边的数不是加数）时，改写成不会被读成「整数 = 项 + 项」的句子，不要凑数。")
    sys.exit(1)
if not scanned or not judged:
    print(f"  ! 本次无对象可判：扫了 {scanned} 份 kb 文件，一条「整数 = 项 + 项 + …」的算式都没有")
    sys.exit(77)
print(f"  ✓ kb 正文里的内联算式左右相等：扫 {scanned} 份 kb 文件（「## 历史版本」之后、代码围栏、变更史除外），判了 {judged} 条；"
      f"跳过 {skipped_operators} 条含乘除减括号的、{skipped_unrecognized} 条带小数、其他单位或认不出写法的")
PY
  if [[ $sums_rc -ne 0 && $sums_rc -ne 1 && $sums_rc -ne 77 ]]; then
    bad "判内联算式的那段 python 没跑成（退出码 $sums_rc）——这一格没判"
    howto "按上面的 traceback 修；没跑成就是没判，不许当成判过了。"
    return 1
  fi
  return "$sums_rc"
}

# ── 格 invariant-prerequisites：还 checks-owed.md C446（未实现的不变量没有前置登记，也没有到期）。
# declared-counts 与 invariant-count-elsewhere 只核条数，没有一格问「未实现那几条被什么挡着、挡它的东西还在不在」。
# 判据：不变量表（invariant_table_files 找到的那几份，与 declared-counts 同一份读法）里每一行 `| I-x.y | … |`，最后一格（状态）去掉 ** 之后以「未实现」起头的，
#   状态格里要有「前置：」，它后面（到第一个 。；; 或 ⚠ 为止）至少写一个编号，每个编号都指得到、而且不能全都已经满足：
#     C<n>              checks-owed.md 里有这一行（按共用读法 lib-owed.py）；在「### 已还清」那张表里算已满足
#     D<n> 已定项 k     decisions/ 下 `## D<n> ` 那份文件「### 已定项」索引里有第 k 行；那一行写「进度：已实现」或「进度：无对象」算已满足
#     D<n> 未定项 k     「### 未定项」索引里有第 k 行；还开着就是没满足
#     E<n>              experiments/ 下有 `## E<n> ` 标题；标题「——」后的状态以「已跑」起头算已满足（「部分已跑」不算）
#   编号后面可以带（简称），简称里的字不当编号读。只写 D<n> 不写分项的判红（要写到分项，才判得出满足没有）。
# 判红：没写前置；前置里一个编号都没有（只有散文）；编号指不到；只写决策号；写的编号全都已经满足（可以开工而没人开工）。
# 看不见：里程碑步号（「三·5」这类）不认，写了也不算编号；前置里编号之外的散文不判；状态不以「未实现」起头、而正文里另有「未实现」字样的行不判（「已实现」「部分」照旧不管）。
# 一份不变量表文件都扫不到、或未实现 0 行，退 77。
# 样本：invariant-prerequisites-red 一份不变量类文件放没写前置、只有散文、已还清的 C、不存在的 C、进度已实现的 D 分项、只写决策号、已跑的 E、不存在的未定项各一行，逐行点名判红，
#   另有一行前置还开着的不点名；invariant-prerequisites-green 前置开着的 C、未实现的已定项、开着的未定项、没跑的 E、一满足一没满足的两个编号各一行，判绿并报几行几个编号。
# 弄坏开关 DOC_REGISTRIES_BREAK=invariant-prerequisites（整格不跑）、prerequisites-satisfied-unchecked（前置全都满足了也不判）、
#   prerequisites-existence-unchecked（编号指不到也不判）：写法与对应的样本在文件头弄坏开关一节。
cell_invariant_prerequisites() {
  cell_failed=0
  # 登记位读不出判红，不当成候选目录是空的：空的会一份不变量表都扫不到、退 77（kb_roots_or_red）
  kb_roots_or_red invariant-prerequisites "不是一份不变量表文件都没有" "$work_directory/invariant-prerequisites.roots" || return 1
  invariant_table_files > "$work_directory/invariant-prerequisites.files"
  local prerequisites_rc=0
  python3 - "$work_directory/invariant-prerequisites.files" "$OWED_LIBRARY" "$KB" "$BREAK_SWITCH" <<'PY' || prerequisites_rc=$?
import glob, importlib.util, os, re, sys

invariant_list, owed_library, KB, BREAK = sys.argv[1:5]
spec = importlib.util.spec_from_file_location("lib_owed", owed_library)
lib_owed = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib_owed)

files = [l.strip() for l in open(invariant_list, encoding="utf-8") if l.strip()]
ROW = re.compile(r"^\|\s*(I-\d+\.\d+)\s*\|")
PREREQUISITE = re.compile(r"前置\s*[：:]")
SEGMENT_END = re.compile(r"[。；;]|⚠")
PAREN = r"(?:（(?:[^（）]|（[^（）]*）)*）|\((?:[^()]|\([^()]*\))*\))?"
REFERENCE = re.compile(
    rf"(?<![A-Za-z0-9_])(?:D(?P<item_decision>\d+){PAREN}\s*(?P<kind>已定项|未定项)\s*(?P<item>\d+)(?![0-9])"
    rf"|D(?P<bare_decision>\d+)(?![0-9]){PAREN}"
    rf"|C(?P<owed>\d+)(?![0-9]){PAREN}"
    rf"|E(?P<experiment>\d+)(?![0-9]){PAREN})")
TRANSLATION = (".en.md", ".ja.md")


def current_text(path):
    return open(path, encoding="utf-8", errors="replace").read().split("\n## 历史版本")[0]


def numbered_headings(directory, letter):
    """编号 → 那份文件的路径与标题行（译本不算）。"""
    found = {}
    for path in sorted(glob.glob(os.path.join(KB, directory, "**", "*.md"), recursive=True)):
        if path.endswith(TRANSLATION):
            continue
        for line in current_text(path).splitlines():
            heading = re.match(rf"^##\s+{letter}(\d+)\s(.*)$", line)
            if heading:
                found.setdefault(int(heading.group(1)), (path, heading.group(2)))
    return found


def decision_items(path):
    """「### 已定项」「### 未定项」索引（到下一个标题为止）里的行：项号 → 整行。"""
    items = {"已定项": {}, "未定项": {}}
    current = None
    for line in current_text(path).splitlines():
        section = re.match(r"^###\s*(已定项|未定项)\s*$", line)
        if section:
            current = items[section.group(1)]
            continue
        if re.match(r"^#+\s", line):
            current = None
            continue
        if current is not None:
            row = re.match(r"^\|\s*(\d+)\s*\|", line) or re.match(r"^(\d+)\.\s", line)
            if row:
                current.setdefault(int(row.group(1)), line)
    return items


owed = lib_owed.read_owed_table(os.path.join(KB, "checks-owed.md"))
decisions = numbered_headings("decisions", "D")
experiments = numbered_headings("experiments", "E")
items_cache = {}


def resolve(reference):
    """→ (写法, 指得到没有, 满足没有, 不成立时的原因)。"""
    if reference.group("owed"):
        name = f"C{reference.group('owed')}"
        if not owed.file_found or not owed.paid_heading_found:
            return name, False, False, f"判不了：{KB}/checks-owed.md 读不出开着与已还清两张表"
        if name in owed.paid_names:
            return name, True, True, ""
        if name in owed.open_names:
            return name, True, False, ""
        return name, False, False, "指不到：checks-owed.md 两张表里都没有这一行"
    if reference.group("experiment"):
        number = int(reference.group("experiment"))
        name = f"E{number}"
        if number not in experiments:
            return name, False, False, "指不到：experiments/ 下没有 `## " + name + " ` 标题"
        status = experiments[number][1].split("——", 1)[1].strip() if "——" in experiments[number][1] else ""
        return name, True, status.startswith("已跑"), ""
    if reference.group("bare_decision"):
        name = f"D{reference.group('bare_decision')}"
        return name, False, False, "只写了决策号，要写到分项（D<n> 已定项 k / 未定项 k），才判得出满足没有"
    number, kind, item = int(reference.group("item_decision")), reference.group("kind"), int(reference.group("item"))
    name = f"D{number} {kind} {item}"
    if number not in decisions:
        return name, False, False, f"指不到：decisions/ 下没有 `## D{number} ` 标题"
    path = decisions[number][0]
    if path not in items_cache:
        items_cache[path] = decision_items(path)
    row = items_cache[path][kind].get(item)
    if row is None:
        return name, False, False, f"指不到：{path} 的「### {kind}」索引里没有第 {item} 行"
    if kind == "未定项":
        return name, True, False, ""
    return name, True, bool(re.search(r"进度[：:]\s*\**\s*(已实现|无对象)", row)), ""


unimplemented = references_total = 0
kinds = {"C": 0, "D": 0, "E": 0}
bad = []   # (路径, 行号, 编号, 原因)
for path in files:
    for number, line in enumerate(open(path, encoding="utf-8", errors="replace").read().split("\n"), 1):
        row = ROW.match(line)
        if not row:
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        status = cells[-1].replace("**", "").strip()
        if not status.startswith("未实现"):
            continue
        unimplemented += 1
        invariant = row.group(1)
        marker = PREREQUISITE.search(status)
        if not marker:
            bad.append((path, number, invariant, "状态写「未实现」，没写「前置：」——它被什么挡着没有登记"))
            continue
        segment = status[marker.end():]
        end = SEGMENT_END.search(segment)
        segment = (segment[:end.start()] if end else segment).strip()
        references = list(REFERENCE.finditer(segment))
        if not references:
            bad.append((path, number, invariant, f"前置「{segment[:60]}」里一个编号都没有——散文判不出满足没有"))
            continue
        problems, satisfied = [], []
        for reference in references:
            references_total += 1
            name, exists, done, why = resolve(reference)
            kinds[name[0]] += 1
            if not exists and BREAK != "prerequisites-existence-unchecked":
                problems.append(f"前置 {name} {why}")
            satisfied.append(done)
        if not problems and all(satisfied) and BREAK != "prerequisites-satisfied-unchecked":
            names = "、".join(resolve(r)[0] for r in references)
            problems.append(f"前置 {names} 都已经满足，状态仍写「未实现」——这一条可以开工了")
        for problem in problems:
            bad.append((path, number, invariant, problem))

if bad:
    print(f"  ✗ 不变量表里 {len(bad)} 处未实现的不变量前置不成立（未实现 {unimplemented} 行，扫 {len(files)} 份）：")  # gate-lint:summary
    for path, number, invariant, problem in bad:
        print(f"      {path}:{number} {invariant}：{problem}")  # gate-lint:detail
    print("  → 怎么办：在那一行状态格「未实现」后面写「，前置：<编号>」，编号写 C<n>（欠账）、D<n>（简称） 已定项 k / 未定项 k（决策分项）或 E<n>（实验），")
    print("    挡着它的事还没有编号就先去立一笔欠账；指不到的改成今天存在的编号；前置都已满足的，开工实现它、或写明现在挡着它的新编号。")
    sys.exit(1)
if not files or not unimplemented:
    print(f"  ! 本次无对象可判：扫了 {len(files)} 份不变量表文件，状态写「未实现」的 0 行")
    sys.exit(77)
print(f"  ✓ 未实现的不变量都写了指得到、还没全满足的编号前置：扫 {len(files)} 份不变量表文件，未实现 {unimplemented} 行，"
      f"前置 {references_total} 个编号（C {kinds['C']}、D 分项 {kinds['D']}、E {kinds['E']}）")
PY
  if [[ $prerequisites_rc -ne 0 && $prerequisites_rc -ne 1 && $prerequisites_rc -ne 77 ]]; then
    bad "判未实现不变量前置的那段 python 没跑成（退出码 $prerequisites_rc）——这一格没判"
    howto "按上面的 traceback 修；没跑成就是没判，不许当成判过了。"
    return 1
  fi
  return "$prerequisites_rc"
}

# ── 格 field-frequency-sizes：还 checks-owed.md C72（extent 和单元什么关系没人答）的门禁比对那一半：「每逻辑 extent 一份」与「覆盖 32 KiB」
#   说的是不是同一个粒度，要能从已定条款里读出来。
# 判据：扫 $KB/decisions/ 与 $KB/layout/ 下的 .md 正文（译本 *.en.md / *.ja.md、*-history.md、decisions-history/、代码围栏里、「## 历史版本」之后都不扫；
#   去掉 ** 再认），找「每[个|一个] X [恒|各|只|都|存|带|持|有|写] 一份」：
#   X 是下面粒度表里的写法 ⇒ 要读得出它的大小：
#     ① 它读成的格式常量在 decisions/ 下有 format-const 标记（lib-format-const.py 的文法），而且至少一处住在某条决策的「#### 已定项 k」小节里、
#        那条分项索引行写着「状态：已定」；
#     ② 那个常量在 decisions/ 与 layout/ 里登记的值只有一个；
#     ③ 粒度表点名的那条条款（下表末两列）在那份决策「### 已定项」索引里、写着「状态：已定」，它的索引行加「#### 已定项 k」小节里写出了这个大小
#        （字节数，或整除时的「N KiB」「N MiB」）。
#   X 是字面大小（「每 512 字节一份」「每 32 KiB 一份」）⇒ 换成字节数，要等于粒度表里某个读得出大小的粒度。
#   粒度表（正文写法 → 格式常量，给出它等于这个大小的已定条款）：
#     逻辑 extent、extent → DATA_UNIT_BYTES，D4 已定项 7（一个逻辑 extent 占盘恒为一个单元）
#     数据单元、单元      → DATA_UNIT_BYTES，D4 已定项 5（单元恒 32768 字节、含头）
#     索引节点、节点      → NODE_BYTES，D8 已定项 2（节点大小 16 KiB）
# 豁免：不是这个格式的字段频率（否决掉的候选设计、别家系统的做法）在那一行正上方单起一行写 <!-- doc-registries:not-field-frequency <为什么> -->，
#   它只管下一个非空行；理由去掉空白不到 4 个字、管的那一行里一处「每 X 一份」都没有，都判红。
# 看不见：粒度表外的 X（「每物理位置一份」「每头一份」）不判；「一份」之外的量词（一个、一条）不认；否定句（「不是每个单元一份」）照判 X 的大小；
#   「单元」一律读成数据单元（码 2 索引节点也叫单元，16 KiB，写的人要说节点）；粒度表点名的条款号在这里写死，那条分项改了号要跟着改这张表。
# 一处「每 X 一份」都没扫到退 77。
# 样本：field-frequency-sizes-red 删掉 D4 已定项 1 的 DATA_UNIT_BYTES 登记（C72 的判别力自证）、D8 已定项 2 没写出 16 KiB、字面 128 KiB、
#   不起作用的豁免与没写理由的豁免，逐条点名判红；field-frequency-sizes-registration-red DATA_UNIT_BYTES 登记在未定项小节里、NODE_BYTES 在 layout 里另登记一个值、
#   D4 已定项 7 不是已定，逐条点名判红；field-frequency-sizes-green 各粒度都读得出、字面 32 KiB、带豁免的 512 字节、否定句、代码围栏、历史节与译本里的错写不判，报判了几处。
# 弄坏开关 DOC_REGISTRIES_BREAK=field-frequency-sizes（整格不跑）、field-frequency-mark-anywhere（登记住在哪一节都认）、
#   field-frequency-literal-unchecked（字面大小不判）、field-frequency-markers-ignored（豁免标记不认）：写法与对应的样本在文件头弄坏开关一节。
cell_field_frequency_sizes() {
  cell_failed=0
  local frequency_rc=0
  python3 - "$FORMAT_CONST_LIBRARY" "$KB" "$BREAK_SWITCH" <<'PY' || frequency_rc=$?
import glob, importlib.util, os, re, sys

format_const_library, KB, BREAK = sys.argv[1:4]
spec = importlib.util.spec_from_file_location("lib_format_const", format_const_library)
lib_format_const = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib_format_const)

# 粒度表：正文写法 → (格式常量, 给出大小的决策号, 已定项号)。长写法排在它的短写法前面，正则按这个次序试。
GRANULES = (
    ("逻辑 extent", "DATA_UNIT_BYTES", 4, 7),
    ("extent", "DATA_UNIT_BYTES", 4, 7),
    ("数据单元", "DATA_UNIT_BYTES", 4, 5),
    ("单元", "DATA_UNIT_BYTES", 4, 5),
    ("索引节点", "NODE_BYTES", 8, 2),
    ("节点", "NODE_BYTES", 8, 2),
)
UNIT_BYTES = {"KiB": 1024, "MiB": 1024 * 1024, "字节": 1, "B": 1}
WORDS = "|".join(re.escape(word).replace(r"\ ", r"\s*") for word, *_ in GRANULES)
FREQUENCY = re.compile(
    rf"每(?:个|一个)?\s*(?:(?P<word>{WORDS})|(?P<number>\d+)\s*(?P<unit>KiB|MiB|字节|B(?![A-Za-z])))"
    r"\s*(?:恒|各|只|都|存|带|持|有|写)?\s*一份")
MARKER = re.compile(r"^\s*<!--\s*doc-registries:not-field-frequency\b(?P<reason>.*?)-->\s*$")
FENCE = re.compile(r"^\s*(```|~~~)")
HISTORY = re.compile(r"^##\s*历史版本")
TRANSLATION = (".en.md", ".ja.md")


def scanned_files(directory):
    for path in sorted(glob.glob(os.path.join(KB, directory, "**", "*.md"), recursive=True)):
        if path.endswith(TRANSLATION) or path.endswith("-history.md") or "/decisions-history/" in path:
            continue
        yield path


def current_lines(path):
    """（行号, 行）：代码围栏里与「## 历史版本」之后的不给。"""
    in_fence = False
    for number, line in enumerate(open(path, encoding="utf-8", errors="replace").read().split("\n"), 1):
        if FENCE.match(line):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        if HISTORY.match(line):
            return
        yield number, line


def granule_word(text):
    squeezed = re.sub(r"\s+", " ", text)
    for word, *_ in GRANULES:
        if squeezed == word:
            return word
    return None


decision_files = list(scanned_files("decisions"))
layout_files = list(scanned_files("layout"))

# ── 决策正文：每份的决策号、已定项索引行、「#### 已定项 k」小节正文、format-const 标记落在哪一节
decision_by_number, settled_rows, settled_sections, marks = {}, {}, {}, []
for path in decision_files:
    lines = list(current_lines(path))
    number = None
    for _, line in lines:
        heading = re.match(r"^##\s+D(\d+)\s", line)
        if heading:
            number = int(heading.group(1))
            decision_by_number.setdefault(number, path)
            break
    in_index, section = False, None
    for line_number, line in lines:
        if re.match(r"^###\s*已定项\s*$", line):
            in_index, section = True, None
            continue
        sub = re.match(r"^####\s*(已定项|未定项)\s*(\d+)", line)
        if re.match(r"^#+\s", line):
            in_index = False
            section = (sub.group(1), int(sub.group(2))) if sub else None
            continue
        if in_index and number is not None:
            row = re.match(r"^\|\s*(\d+)\s*\|", line)
            if row:
                settled_rows.setdefault((number, int(row.group(1))), line)
        if section and section[0] == "已定项" and number is not None:
            settled_sections.setdefault((number, section[1]), []).append(line)
        for mark in lib_format_const.parse_marks(line).marks:
            marks.append((mark.name, mark.value, path, line_number, number, section))
for path in layout_files:
    for line_number, line in current_lines(path):
        for mark in lib_format_const.parse_marks(line).marks:
            marks.append((mark.name, mark.value, path, line_number, None, None))


def row_settled(row):
    return bool(row) and bool(re.search(r"状态[：:]\s*\**\s*已定", row))


def size_spellings(value):
    spellings = [rf"(?<![\d.]){value}(?!\d)"]
    for unit, factor in (("KiB", 1024), ("MiB", 1024 * 1024)):
        if value % factor == 0:
            spellings.append(rf"(?<![\d.]){value // factor}\s*{unit}")
    return re.compile("|".join(spellings))


granule_sizes, granule_problems = {}, {}
for word, constant, decision, item in GRANULES:
    problems = []
    registered = [m for m in marks if m[0] == constant]
    in_decisions = [m for m in registered if m[4] is not None]
    settled_marks = [m for m in in_decisions
                     if BREAK == "field-frequency-mark-anywhere"
                     or (m[5] and m[5][0] == "已定项" and row_settled(settled_rows.get((m[4], m[5][1]))))]
    if not in_decisions:
        problems.append(f"decisions/ 下没有 {constant} 的 format-const 登记")
    elif not settled_marks:
        places = "、".join(f"{m[2]}:{m[3]}" for m in in_decisions)
        problems.append(f"{constant} 登记在 {places}，不在任何写着「状态：已定」的已定项小节里")
    values = sorted({m[1] for m in registered})
    if len(values) > 1:
        places = "、".join(f"{m[2]}:{m[3]}（{m[1]}）" for m in registered)
        problems.append(f"{constant} 登记了不止一个值：{places}")
    clause = f"D{decision} 已定项 {item}"
    row = settled_rows.get((decision, item))
    if decision not in decision_by_number:
        problems.append(f"给出它大小的 {clause}：decisions/ 下没有 `## D{decision} ` 标题")
    elif row is None:
        problems.append(f"给出它大小的 {clause}：{decision_by_number[decision]} 的「### 已定项」索引里没有第 {item} 行")
    elif not row_settled(row):
        problems.append(f"给出它大小的 {clause} 没写「状态：已定」")
    elif settled_marks and len(values) == 1:
        # 标记自己写着这个数，不算条款写出了它：先去掉标记再找
        clause_text = lib_format_const.strip_marks("\n".join([row] + settled_sections.get((decision, item), [])))
        if not size_spellings(values[0]).search(clause_text):
            problems.append(f"给出它大小的 {clause} 里没写出 {constant} 登记的 {values[0]}（字节数，或 KiB / MiB 写法）")
    if problems:
        granule_problems[word] = problems
    else:
        granule_sizes[word] = (constant, values[0], clause)

resolved_sizes = sorted({size for _, size, _ in granule_sizes.values()})
hits, exempted, by_kind = 0, 0, {}
bad = []      # (路径, 行号, 原文片段, 原因)
for path in decision_files + layout_files:
    lines = list(current_lines(path))
    governed = {}
    for position, (line_number, line) in enumerate(lines):
        marker = MARKER.match(line)
        if not marker:
            continue
        if BREAK == "field-frequency-markers-ignored":
            continue
        following = next(((n, l) for n, l in lines[position + 1:] if l.strip()), None)
        reason = re.sub(r"\s+", "", marker.group("reason"))
        if len(reason) < 4:
            bad.append((path, line_number, line.strip()[:80], "not-field-frequency 标记没写理由（去掉空白至少 4 个字）"))
        if following is None or not FREQUENCY.search(following[1].replace("**", "")):
            bad.append((path, line_number, line.strip()[:80], "not-field-frequency 标记管的下一个非空行里一处「每 X 一份」都没有，是不起作用的标记"))
        else:
            governed[following[0]] = True
    for line_number, line in lines:
        text = line.replace("**", "")
        for hit in FREQUENCY.finditer(text):
            if MARKER.match(line):
                continue
            hits += 1
            phrase = hit.group(0)
            if governed.get(line_number):
                exempted += 1
                continue
            if hit.group("word"):
                word = granule_word(hit.group("word"))
                by_kind[word] = by_kind.get(word, 0) + 1
                if word in granule_problems:
                    bad.append((path, line_number, phrase, f"「{word}」的大小读不出来：" + "；".join(granule_problems[word])))
                continue
            by_kind["字面大小"] = by_kind.get("字面大小", 0) + 1
            size = int(hit.group("number")) * UNIT_BYTES[hit.group("unit")]
            if size not in resolved_sizes and BREAK != "field-frequency-literal-unchecked":
                known = f"读得出大小的粒度：{'、'.join(str(s) for s in resolved_sizes)} 字节" if resolved_sizes else "没有一个粒度读得出大小"
                bad.append((path, line_number, phrase, f"{size} 字节不是哪条已定条款给出大小的粒度（{known}）"))

scanned = len(decision_files) + len(layout_files)
if bad:
    print(f"  ✗ {len(bad)} 处字段频率的大小指不到已定条款（扫 {scanned} 份、{hits} 处「每 X 一份」）：")  # gate-lint:summary
    for path, line_number, phrase, problem in sorted(bad, key=lambda entry: (entry[0], entry[1])):
        print(f"      {path}:{line_number} 「{phrase}」：{problem}")  # gate-lint:detail
    print("  → 怎么办：粒度的大小读不出来，先让给出大小的那条已定分项写出这个数，并在它的「#### 已定项 k」小节里登记 <!-- format-const: 名字 = 值 -->；")
    print("    字面大小对不上，改成已定的粒度（单元、节点）或把字段频率写对；这句说的不是本格式的字段（否决掉的候选、别家系统），")
    print("    在那一行正上方单起一行写 <!-- doc-registries:not-field-frequency <为什么> -->（判据在 doc-registries.sh 里这一格函数上方的注释）。")
    sys.exit(1)
if not hits:
    print(f"  ! 本次无对象可判：扫了 {scanned} 份决策与布局正文，一处「每 X 一份」都没有")
    sys.exit(77)
kinds = "、".join(f"{word} {count}" for word, count in by_kind.items())
clauses_by_constant = {}
for constant, size, clause in granule_sizes.values():
    entry = clauses_by_constant.setdefault((constant, size), [])
    if clause not in entry:
        entry.append(clause)
sizes = "，".join(f"{constant} = {size}（{'、'.join(clauses)}）" for (constant, size), clauses in clauses_by_constant.items())
print(f"  ✓ 字段频率「每 X 一份」的 X 都指得到已定条款给出的大小：扫 {scanned} 份，{hits} 处（{kinds}；豁免 {exempted} 处）；读得出大小的粒度：{sizes}")
PY
  if [[ $frequency_rc -ne 0 && $frequency_rc -ne 1 && $frequency_rc -ne 77 ]]; then
    bad "判字段频率大小的那段 python 没跑成（退出码 $frequency_rc）——这一格没判"
    howto "按上面的 traceback 修；没跑成就是没判，不许当成判过了。"
    return 1
  fi
  return "$frequency_rc"
}

# ── 格 unreachable-claims：kb 里写「不可达 / 走不到 / 触发不了」的句子，要么钉一条用例，要么写明是推的（拆分提案第四十节第 26 行；只判新增行）
# 为什么：C120 写「掉盘只能只读挂载 ⇒ 不可达」，于是挂载准入与整盘作废一直没实现，实测空盘、旧盘都能挂成可写；「不可达」是一句推论，变成可达时没有东西报警。
# 判据：
#   ① 对象：这次改动在 .claude/kb 下中文原文 .md（不含 *.en.md、*.ja.md 与 en/、ja/ 目录下的译文）里新增的行（scripts/gate/changed-paths.sh 的 gate_added_lines，
#      基准是 gate 取法；未跟踪的 kb 文件整份算新增），行里有 UNREACHABLE_CLAIM_WORDS 之一的。
#   ② 每一行按句切（。！？与表格的 | 是句界，「；」不切），有断言词的每一句要么写了 UNREACHABLE_INFERRED_MARKERS 之一（推的、没量过），
#      要么带一个仓里在的用例名：句里某个标识符是 crates/、scripts/ 下（git 列出的，含未跟踪、不含被忽略的）某份 .rs 里 #[test]（或 #[<路径>::test]）标的 fn 的名字。
#      两样都没有的句子逐句判红；句里有像用例名（小写、带下划线）而找不到定义的标识符，红行另注它。
#   ③ 存量：UNREACHABLE_STOCK_TABLE 一行「<kb 文件路径><制表符><那一行的原文>」，登记这一格立起来时 kb 里已有、不合 ② 的行。新增的行与清单里同一文件的一行逐字相同
#      （行尾空白不算）的放过、成功句里报个数（挪了位置的旧句会以新增行的样子出现）；清单里的行改了一个字就不再放过，照 ② 判。
#      清单每行要两段、同一行不登两次、指的文件在；只缩不涨：基准那一版里有这张表时，此刻的每一行都要在基准那一版里（git 取不到基准那一版判红，基准里还没有这张表不比）。
#      清单里那一行在文件里已经找不到原文的不判红，成功句报个数（改好了的句子把清单那一行删掉）。清单有没有毛病每一趟都判，这次改动一句断言都没新增也判。
#   这次改动 kb 里一句断言都没新增、清单也没毛病：退 77（存量行数照报）。
# 管不到的：用例是不是真构造了这条路径、真断言了被拒（只核名字有定义、标了 #[test]）；宏生成的测试名认不出（照没带用例名红，改写「推的」或点名展开处的名字）；
#   断言词之外的说法（「到不了」「不会发生」「不可能」）不认；新增行在代码围栏里、「## 历史版本」之后的照判（按行取，不知道它落在哪一节）；
#   存量清单里登的是不是真是这一格立起来时就有的（只靠只缩不涨守）；kb 之外（crates/singlefs-checker/invariants/ 这类登记的 kb 目录、records/）不判。
# 样本与弄坏开关见文件头（unreachable-claims-red、unreachable-claims-test-name-green、unreachable-claims-inferred-green、unreachable-claims-stock-grown-red；
# 弄坏开关 DOC_REGISTRIES_BREAK=unreachable-claims、unreachable-claims-names-unresolved、unreachable-claims-names-ignored、unreachable-claims-inferred-ignored、unreachable-claims-stock-growth-unchecked）。
# 退役判据：kb 里不再有这类断言词（存量清单删空、连续四周没有新增句判红）时删掉这一格与清单。
cell_unreachable_claims() {
  cell_failed=0
  git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ! $PWD 不是 git 仓，这一格本次无对象可判"; return 77; }
  if [[ ! -d "$KB" ]]; then
    echo "  ! 没有 $KB，这一格本次无对象可判"
    return 77
  fi
  local LIB_CHANGED_PATHS base claims_rc=0
  LIB_CHANGED_PATHS="$(cd "$GATE_DIRECTORY/../.." && pwd)/scripts/gate/changed-paths.sh"
  # shellcheck source=../../scripts/gate/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用库 $LIB_CHANGED_PATHS"; echo "     → 怎么办：改动范围的取法只有那一份，恢复它，别在阶段里再抄一份。"; return 1; }
  base="$(gate_diff_base gate)"
  if ! gate_added_lines "$base" "$KB" > "$work_directory/unreachable-claims.added"; then
    echo "  ✗ 取不到 $KB 这次改动新增的行（基准 $base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到新增行时这一格什么都没比，不是通过。"
    return 1
  fi
  if ! git -c core.quotepath=false ls-files -co --exclude-standard -z -- 'crates/*.rs' 'scripts/*.rs' > "$work_directory/unreachable-claims.sources"; then
    echo "  ✗ 列不出 crates/、scripts/ 下的 .rs（git ls-files 失败），认不出句子里点名的用例"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（索引没坏）再跑；列不出用例时这一格什么都没比，不是通过。"
    return 1
  fi
  python3 - "$work_directory/unreachable-claims.added" "$work_directory/unreachable-claims.sources" "$UNREACHABLE_STOCK_TABLE" "$base" \
    "$LAG_BASELINE_LIBRARY" "$BREAK_SWITCH" "$UNREACHABLE_CLAIM_WORDS" "$UNREACHABLE_INFERRED_MARKERS" "$KB" <<'PY' || claims_rc=$?
import importlib.util, os, re, sys

added_path, sources_path, stock_path, base, lag_library, breakage, words_text, markers_text, kb = sys.argv[1:10]
spec = importlib.util.spec_from_file_location("lag_baseline", lag_library)
lag_baseline = importlib.util.module_from_spec(spec)
spec.loader.exec_module(lag_baseline)
WORDS = words_text.split()
MARKERS = markers_text.split()
SENTENCE_END = re.compile(r"[。！？!?|]")
IDENTIFIER = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
LOOKS_LIKE_TEST = re.compile(r"[a-z][a-z0-9]*(?:_[a-z0-9]+)+")
TEST_FUNCTION = re.compile(r"#\[(?:[A-Za-z_][A-Za-z0-9_]*::)*test\b[^\]]*\]\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)")
WORDS_SHOWN = " / ".join(WORDS)

def judged_path(path):
    """.claude/kb 下的中文原文 .md：译文（*.en.md、*.ja.md、en/、ja/ 目录下的）不判"""
    if not path.startswith(kb + "/") or not path.endswith(".md") or path.endswith((".en.md", ".ja.md")):
        return False
    directories = path.split("/")[:-1]
    return "en" not in directories and "ja" not in directories

test_names = None
def defined_tests():
    """crates/、scripts/ 下 .rs 里 #[test] 标的 fn 名字（读一次）"""
    global test_names
    if test_names is None:
        test_names = set()
        for raw in open(sources_path, "rb").read().split(b"\0"):
            if not raw:
                continue
            try:
                text = open(raw.decode("utf-8", "replace"), encoding="utf-8", errors="replace").read()
            except OSError:
                continue
            if "test" in text:
                test_names.update(TEST_FUNCTION.findall(text))
    return test_names

def sentence_verdict(sentence):
    """('inferred' | 'named' | 'bare', 像用例名却找不到定义的标识符)"""
    if breakage != "unreachable-claims-inferred-ignored" and any(marker in sentence for marker in MARKERS):
        return "inferred", []
    tokens = IDENTIFIER.findall(sentence)
    look_alike = [token for token in tokens if LOOKS_LIKE_TEST.fullmatch(token)]
    if breakage == "unreachable-claims-names-ignored":
        named = []
    elif breakage == "unreachable-claims-names-unresolved":
        named = look_alike
    else:
        named = [token for token in tokens if token in defined_tests()] if tokens else []
    if named:
        return "named", []
    return "bare", look_alike

# ── 存量清单：形状、重复、文件在不在、只缩不涨 ──
problems, stock, stale = [], {}, 0
growth_note = "没有这张表"
def stock_key(fields):
    return fields[0].strip(), "\t".join(fields[1:]).rstrip()
if os.path.exists(stock_path):
    try:
        stock_text = open(stock_path, encoding="utf-8").read()
    except (OSError, UnicodeDecodeError) as error:
        stock_text = ""
        problems.append(f"{stock_path} 读不了（{type(error).__name__}）")
    file_lines = {}
    for number, fields in lag_baseline.content_rows(stock_text):
        if len(fields) < 2 or not fields[0].strip() or not "\t".join(fields[1:]).strip():
            problems.append(f"{stock_path}:{number} 不是「kb 文件路径<制表符>那一行的原文」两段")
            continue
        key = stock_key(fields)
        if key in stock:
            problems.append(f"{stock_path}:{number} 与第 {stock[key]} 行重复")
            continue
        stock[key] = number
        if key[0] not in file_lines:
            try:
                file_lines[key[0]] = {line.rstrip() for line in open(key[0], encoding="utf-8", errors="replace").read().split("\n")}
            except OSError:
                file_lines[key[0]] = None
        if file_lines[key[0]] is None:
            problems.append(f"{stock_path}:{number} 指的文件 {key[0]} 不在")
        elif key[1] not in file_lines[key[0]]:
            stale += 1
    growth_note = "没和基准比"
    if base and breakage != "unreachable-claims-stock-growth-unchecked":
        baseline, baseline_problem = lag_baseline.baseline_rows(base, stock_path)
        if baseline_problem is not None:
            problems.append(f"{baseline_problem.sentence()}，只缩不涨没比")
        elif baseline is None:
            growth_note = f"基准 {base[:12]} 里还没有这张表，没比只缩不涨"
        else:
            baseline_keys = {stock_key(fields) for _number, fields in baseline if len(fields) >= 2}
            grown = sorted((number, key) for key, number in stock.items() if key not in baseline_keys)
            for number, key in grown:
                problems.append(f"{stock_path}:{number} 是基准 {base[:12]} 之后新加的（{key[0]}：{key[1].strip()[:60]}）：这张表只缩不涨")
            growth_note = f"与基准 {base[:12]} 那一版比，" + (f"多出 {len(grown)} 行" if grown else "没涨")

# ── 这次改动新增的断言句 ──
added, seen = [], set()
for raw in open(added_path, encoding="utf-8", errors="replace").read().split("\n"):
    if "\t" not in raw:
        continue
    path, line = raw.split("\t", 1)
    line = line.rstrip()
    if judged_path(path) and any(word in line for word in WORDS) and (path, line) not in seen:
        seen.add((path, line))
        added.append((path, line))
failing, named_count, inferred_count, stocked = [], 0, 0, 0
for path, line in added:
    verdicts = [(sentence, *sentence_verdict(sentence)) for sentence in SENTENCE_END.split(line) if any(word in sentence for word in WORDS)]
    bare = [(sentence, look_alike) for sentence, verdict, look_alike in verdicts if verdict == "bare"]
    if bare and (path, line) in stock:
        stocked += 1
        continue
    named_count += sum(1 for _sentence, verdict, _look in verdicts if verdict == "named")
    inferred_count += sum(1 for _sentence, verdict, _look in verdicts if verdict == "inferred")
    failing.extend((path, sentence, look_alike) for sentence, look_alike in bare)

stock_summary = f"存量清单 {stock_path} {len(stock)} 行（{growth_note}；原文已找不到、可以删的 {stale} 行）"
if problems:
    print(f"  ✗ 存量清单 {stock_path} 有 {len(problems)} 处不对：")  # gate-lint:summary
    for problem in problems:
        print(f"      {problem}")  # gate-lint:detail
    print("     → 怎么办：这张表只缩不涨：新写的断言句照这一格带用例名或写「推的，没量过」，不往表里加行；一行写成「kb 文件路径<制表符>那一行的原文」，")
    print("               重复的、指的文件已经不在的删掉那一行。")
if failing:
    print(f"  ✗ kb 这次改动新增的 {len(failing)} 句「{WORDS_SHOWN}」断言既没带仓里在的用例名，也没写「{'」「'.join(MARKERS)}」（基准 {base}）：")  # gate-lint:summary
    for path, sentence, look_alike in failing:
        note = f"（{'、'.join(look_alike)} 在 crates/、scripts/ 下找不到 #[test] fn 定义）" if look_alike else ""
        print(f"      {path}：{sentence.strip()[:120]}{note}")  # gate-lint:detail
    print("     → 怎么办：在同一句里写上构造这条路径、断言它被拒的那条用例的名字（crates/ 或 scripts/ 下 #[test] fn <名字>，写函数名本身），")
    print("               没有这样的用例就写「推的，没量过」；要钉住它就先写出那条用例。存量清单不往里加行。")
if problems or failing:
    sys.exit(1)
if not added:
    print(f"  ⊘ 本次无对象可判：kb 这次改动没有新增「{WORDS_SHOWN}」断言句（基准 {base}）；{stock_summary}")
    sys.exit(77)
judged = named_count + inferred_count
print(f"  ✓ kb 这次改动新增的 {len(added)} 行里 {judged} 句「{WORDS_SHOWN}」断言都带了仓里在的用例名或写了推的"
      f"（带用例名 {named_count} 句、写推的 {inferred_count} 句；与存量清单逐字相同、按清单放过的 {stocked} 行；基准 {base}）；{stock_summary}")
PY
  if [[ $claims_rc -ne 0 && $claims_rc -ne 1 && $claims_rc -ne 77 ]]; then
    bad "判「不可达」类断言的那段 python 没跑成（退出码 $claims_rc）——这一格没判"
    howto "按上面的 traceback 修；没跑成就是没判，不许当成判过了。"
    return 1
  fi
  return "$claims_rc"
}

# ── 格 counted-claims：CLAUDE.md 与仓根三份 README 这类文档里写死的能数的数（条数、项数），与登记的命令此刻数出来的相等
# declared-counts、invariant-count-elsewhere 只认 kb 里「N 条在用」这一种写法；CLAUDE.md「checker 判 N 条不变量」「崩溃放量的 N 项」与 README 的
# 「`IMPLEMENTED_INVARIANTS` 有 N 项」这类句子一周内漂过两次，没有东西在对（records/2026-10-30-知识腐烂治理按现状.md「现状」表第 1 行）。
# 判据：登记表 COUNTED_CLAIMS_REGISTRY（被判的根里那一份）一行登一处，四列用制表符分隔：
#   路径      从仓根起的相对路径，一行一份文件；同一个数在几份译文里各写一次的，每份各登一行、各写各语言的正则（不在一格里用分隔符并几份）
#   定位正则  python re 的写法，恰好一个捕获组，抓那个十进制数
#   数的命令  在被判的根里经 `bash -o pipefail -c` 起，标准输入接 /dev/null，限时 COUNTED_CLAIMS_COMMAND_SECONDS 秒；标准输出去掉首尾空白之后要是一个十进制整数；
#             几行写同一条命令的只起一次
#   说明      数的是什么
#   # 起头的行与空行不算登记。
# 逐行判：正则在那份文件里恰好命中一次（0 次、多次都红）；命令退非 0、超时、输出不是十进制整数，判红；两个数不等判红，出路「正文改成 <现数>，或登记表的命令改对」；
# 列数不是四、路径不是仓内相对路径、正则编不过或捕获组不是一个、文件不在、捕获组抓到的不是十进制数，也判红。
# 登记表不在、或一行登记都没有：退 77（本次未跑，不算绿）。成功句报核了几行、几份文件、起了几条命令，下一行逐处列「路径:行号 数」。
# 管不到的：没登记的句子（这一格只看登记表）；命令数得对不对（命令写错了两边一起错，登记时先在真仓上跑一遍核过再登）。
# 样本（都不是 git 仓、没有 setup.sh；数数的命令都是样本根下的假脚本 count-sample-items.sh，经 bash 起）：counted-claims-red：一行数对不上（写 7、现数 9）、一行正则命中两次、一行命中 0 次、一行命令退 3（打的数与正文相同）、
#   一行命令打的不是整数、一行只有三列、一行指的文件不在、一行正则没有捕获组，八处逐行点名判红；另有一行写对的不许点名。
#   counted-claims-green：README.md、README.en.md 与 NOTES.md 三份四行（两行共用一条命令），判绿，报「核了 4 行、3 份文件、起了 3 条命令」。
#   counted-claims-registry-empty：登记表只有注释与空行，退 77；counted-claims-registry-absent：没有登记表，退 77。
# 弄坏开关 DOC_REGISTRIES_BREAK=counted-claims-mismatch-unchecked（不等不判）、counted-claims-match-count-unchecked（命中次数不判，取第一处，一处都没有就跳过那一行）、
#   counted-claims-command-exit-ignored（命令的退出码不看），各在文件头开关表。
cell_counted_claims() {
  if [[ ! -f "$COUNTED_CLAIMS_REGISTRY" ]]; then
    echo "  ! 没有登记表 $COUNTED_CLAIMS_REGISTRY，这一格本次未跑（不算绿）"
    echo "    要让这一格判：照 doc-registries.sh 里这一格函数上方的注释建这张表，一行登一处能数的数。"
    exit 77
  fi
  BREAK_SWITCH="$BREAK_SWITCH" python3 - "$COUNTED_CLAIMS_REGISTRY" "$COUNTED_CLAIMS_COMMAND_SECONDS" <<'PY'
import os, re, signal, subprocess, sys

registry_path, command_seconds = sys.argv[1], int(sys.argv[2])
BREAK = os.environ.get("BREAK_SWITCH", "")
DECIMAL = re.compile(r"[0-9]+")
COLUMN_COUNT = 4
TERMINATE_GRACE_SECONDS = 10

registered = []
with open(registry_path, encoding="utf-8") as registry_file:
    for registry_line_number, raw_line in enumerate(registry_file, 1):
        line = raw_line.rstrip("\n")
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        registered.append((registry_line_number, line.split("\t")))
if not registered:
    print(f"  ! 登记表 {registry_path} 一行登记都没有（只有注释或空行），这一格本次未跑（不算绿）")
    print("    要让这一格判：一行登一处能数的数，四列用制表符分隔：路径、定位正则、数的命令、说明。")
    sys.exit(77)

command_results = {}


def counting_command_result(command):
    """在被判的根里起一条数数的命令，几行写同一条的只起一次；返回（退出码，超时为 None；标准输出；标准错误）。
    命令放进它自己的进程组，超时与跑完都对整组发信号，不留进程（command-safety.md「装置起的样本进程，放进它自己的进程组」）。"""
    if command in command_results:
        return command_results[command]
    process = subprocess.Popen(["bash", "-o", "pipefail", "-c", command], stdin=subprocess.DEVNULL,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True)
    try:
        output, errors = process.communicate(timeout=command_seconds)
        exit_code = process.returncode
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            output, errors = process.communicate(timeout=TERMINATE_GRACE_SECONDS)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            output, errors = process.communicate()
        exit_code = None
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    command_results[command] = (exit_code, output, errors)
    return command_results[command]


problems = []
judged = []
for registry_line_number, cells in registered:
    where = f"{registry_path}:{registry_line_number}"
    if len(cells) != COLUMN_COUNT:
        problems.append(f"{where} 有 {len(cells)} 列，要四列（路径、定位正则、数的命令、说明，用制表符分隔）")
        continue
    path, pattern_text, command, _description = (cell.strip() for cell in cells)
    if not path or os.path.isabs(path) or ".." in path.split("/"):
        problems.append(f"{where} 路径「{path}」不是从仓根起的仓内相对路径")
        continue
    try:
        pattern = re.compile(pattern_text)
    except re.error as error:
        problems.append(f"{where} {path} 的定位正则编不过（{error}）")
        continue
    if pattern.groups != 1:
        problems.append(f"{where} {path} 的定位正则有 {pattern.groups} 个捕获组，要恰好一个（抓那个十进制数）")
        continue
    if not os.path.isfile(path):
        problems.append(f"{where} 指的 {path} 不在")
        continue
    with open(path, encoding="utf-8") as claimed_file:
        text = claimed_file.read()
    matches = list(pattern.finditer(text))
    if BREAK == "counted-claims-match-count-unchecked":
        if not matches:
            continue
        matches = matches[:1]
    match_line_numbers = [text.count("\n", 0, match.start()) + 1 for match in matches]
    if len(matches) != 1:
        lines_named = "、".join(f"第 {number} 行" for number in match_line_numbers)
        way_out = "正则改窄到只罩那一句" if matches else "那一句改了写法就把正则跟着改对，句子删了就删这一行登记"
        problems.append(f"{where} 的定位正则在 {path} 里命中 {len(matches)} 次{'（' + lines_named + '）' if lines_named else ''}，要恰好一次；{way_out}")
        continue
    written = matches[0].group(1)
    claim_location = f"{path}:{match_line_numbers[0]}"
    if not DECIMAL.fullmatch(written or ""):
        problems.append(f"{where} {claim_location} 捕获组抓到的「{written}」不是十进制数；捕获组改成只罩那个数")
        continue
    exit_code, output, errors = counting_command_result(command)
    if exit_code is None:
        problems.append(f"{where} {claim_location} 数的命令跑了 {command_seconds} 秒没结束，停了它：{command}")
        continue
    error_tail = errors.strip().splitlines()[-1][:160] if errors.strip() else "标准错误是空的"
    if exit_code != 0 and BREAK != "counted-claims-command-exit-ignored":
        problems.append(f"{where} {claim_location} 数的命令退 {exit_code}（{error_tail}）：{command}")
        continue
    counted = output.strip()
    if not DECIMAL.fullmatch(counted):
        problems.append(f"{where} {claim_location} 数的命令打的不是一个十进制整数：「{counted[:80]}」：{command}")
        continue
    if int(written) != int(counted) and BREAK != "counted-claims-mismatch-unchecked":
        problems.append(f"{where} {claim_location} 正文写 {int(written)}，现数 {int(counted)}；出路：正文改成 {int(counted)}，或登记表的命令改对")
        continue
    judged.append((claim_location, int(written)))

registered_files = sorted({cells[0].strip() for _, cells in registered if len(cells) == COLUMN_COUNT})
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 登记的能数的数有 {len(problems)} 行没核过（登记 {len(registered)} 行、{len(registered_files)} 份文件，核过 {len(judged)} 行）")  # gate-lint:summary
    print(f"     → 怎么办：数对不上的照红行把正文改成现数，或把 {registry_path} 那一行的命令改对；正则命中不是一次的改窄或改对；")
    print("               命令跑不通的先在仓根手跑一遍修好；表的写法在 doc-registries.sh 里 counted-claims 格函数上方的注释")
    sys.exit(1)
print(f"  ✓ 登记的能数的数都与现数相等（核了 {len(judged)} 行、{len(registered_files)} 份文件、起了 {len(command_results)} 条命令；登记表 {registry_path}）")
print("    " + "；".join(f"{location} {number}" for location, number in judged))
PY
}

# ════ 登记的格函数：弄坏开关点名这一格就不跑、按判过了记，否则调这一格的判法 ════
# 字段与布局组的格（最早那九格与后加的四格）stdin 都接到 /dev/null，与最早那九格并进来之前在后台起时一致。
break_skips() {
  if [[ "$BREAK_SWITCH" == "$1" ]]; then
    echo "  ! 弄坏开关 DOC_REGISTRIES_BREAK=$1：这一格没判"
    exit 0
  fi
}
judge_field_refs()                   { break_skips field-refs;                   cell_field_refs </dev/null; }
judge_field_projection()             { break_skips field-projection;             cell_field_projection </dev/null; }
judge_field_table_sums()             { break_skips field-table-sums;             cell_field_table_sums </dev/null; }
judge_new_pool_file_creation_hooks()              { break_skips first-txn-hooks;              cell_new_pool_file_creation_hooks </dev/null; }
judge_admission_terms()              { break_skips admission-terms;              cell_admission_terms </dev/null; }
judge_segment_registry()             { break_skips segment-registry;             cell_segment_registry </dev/null; }
judge_format_const_placeholders()    { break_skips format-const-placeholders;    cell_format_const_placeholders </dev/null; }
judge_overwrite_release_rollback_reuse_hooks()             { break_skips second-txn-hooks;             cell_overwrite_release_rollback_reuse_hooks </dev/null; }
judge_crash_verification_rebuild_hooks()              { break_skips third-txn-hooks;              cell_crash_verification_rebuild_hooks </dev/null; }
judge_tree_table_reserve()           { break_skips tree-table-reserve;           cell_tree_table_reserve </dev/null; }
judge_field_widths()                 { break_skips field-widths;                 cell_field_widths </dev/null; }
judge_unit_header_fields()           { break_skips unit-header-fields;           cell_unit_header_fields </dev/null; }
judge_mount_transparency_fields()    { break_skips mount-transparency-fields;    cell_mount_transparency_fields </dev/null; }
judge_table_shape()                  { break_skips table-shape;                  cell_table_shape; }
judge_closeout_collects_open()       { break_skips closeout-collects-open;       cell_closeout_collects_open; }
judge_paid_cited_tests()             { break_skips paid-cited-tests;             cell_paid_cited_tests; }
judge_audit_contradictions()         { break_skips audit-contradictions;         cell_audit_contradictions; }
judge_closeout_row27_preconditions() { break_skips closeout-row27-preconditions; cell_closeout_row27_preconditions; }
judge_experiment_refs()              { break_skips experiment-refs;              cell_experiment_refs; }
judge_decision_refs()                { break_skips decision-refs;                cell_decision_refs; }
judge_declared_counts()              { break_skips declared-counts;              cell_declared_counts; }
judge_governance_refs()              { break_skips governance-refs;              cell_governance_refs; }
judge_invariant_count_elsewhere()    { break_skips invariant-count-elsewhere;    cell_invariant_count_elsewhere; }
judge_citations()                    { break_skips citations;                    cell_citations; }
judge_invariant_anchors()            { break_skips invariant-anchors;            cell_invariant_anchors; }
judge_inline_sums()                  { break_skips inline-sums;                  cell_inline_sums; }
judge_invariant_prerequisites()      { break_skips invariant-prerequisites;      cell_invariant_prerequisites; }
judge_field_frequency_sizes()        { break_skips field-frequency-sizes;        cell_field_frequency_sizes; }
judge_unreachable_claims()           { break_skips unreachable-claims;           cell_unreachable_claims; }
judge_counted_claims()               { break_skips counted-claims;               cell_counted_claims; }
