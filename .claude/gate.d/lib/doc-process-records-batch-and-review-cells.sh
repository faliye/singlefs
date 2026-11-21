#!/usr/bin/env bash
# 门禁 doc-process-records 的格函数库：batch-scope、verdict-names-local-samples、crates-adversarial-review、code-round-snapshot、implementation-premise。
# 由 .claude/gate.d/doc-process-records.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释；读的 ROOT 与入口里设好的各变量、bad、ok、howto 在调用时才取。

# ── batch-scope ──────────────────────────────────────────────────────────────
check_batch_scope() {
  ((inside_git)) || { not_git_repository; return; }
  load_changed_since_head || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 HEAD）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    return 1
  }
  # 改动清单经进程替换当文件传：当成一个命令行参数传时，单个参数超过 128 KiB 就起不来（Linux 的 MAX_ARG_STRLEN）。
  python3 - <(printf '%s\n' "$changed_since_head") "$TRIGGER_TABLE" <<'PY'
import os, re, sys

with open(sys.argv[1], encoding="utf-8", errors="replace") as handle:
    changed_files = sorted({path for path in handle.read().split("\n") if path})

TRIGGER_TABLE = sys.argv[2]
SCOPE_FILE = ".claude/batch-scope"

if not os.path.isfile(TRIGGER_TABLE):
    print(f"  ✗ 读不到触发文件清单 {TRIGGER_TABLE}")
    print("     → 怎么办：那份表是触发文件的唯一登记位，knowledge-sync 格与这一格都读它；恢复它，别在脚本里再存一份清单。")
    sys.exit(1)
patterns = []
for raw in open(TRIGGER_TABLE, encoding="utf-8").read().split("\n"):
    body = raw.split("\t")[0].strip()
    if body and not body.startswith("#"):
        patterns.append(re.compile(body))
if not patterns:
    print(f"  ✗ 触发文件清单 {TRIGGER_TABLE} 里一条正则都没有")
    print("     → 怎么办：一行一条「<python 正则><制表符>#<这一类是什么>」，空表等于这道检查整个关掉。")
    sys.exit(1)

if not changed_files:
    print("  ! 相对 HEAD 一个改动都没有，这一格无对象可判")
    sys.exit(77)

triggers = [path for path in changed_files if any(pattern.search(path) for pattern in patterns)]

registered = {}
registered_lines = []   # (行号, 路径, 整行原文)：同一个路径登记了几行就有几条，悬空的要一行不漏地列、删
no_reason, bad_path = [], []
if os.path.isfile(SCOPE_FILE):
    for number, raw in enumerate(open(SCOPE_FILE, encoding="utf-8").read().split("\n"), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        path, _, reason = line.partition("#")
        path = path.strip()
        if not path:
            continue
        registered[path] = number
        registered_lines.append((number, path, raw))
        if not reason.strip():
            no_reason.append(f"{SCOPE_FILE}:{number}：{path} 后面没写为什么这一批要碰它")
        if path.startswith(("/", "./", "../")) or "/../" in path:
            bad_path.append(f"{SCOPE_FILE}:{number}：{path} 不是从仓库根起写的")

missing = [path for path in triggers if path not in registered]
stale = [path for path in registered if path not in triggers]

failed = False
if missing:
    failed = True
    print(f"  ✗ {len(missing)} 个触发文件没登记进这一批的范围（{SCOPE_FILE} 现在登记了 {len(registered)} 个）：")
    for path in sorted(missing):
        print(f"      {path}")                                  # gate-lint:detail
    print(f"     → 怎么办：每多碰一个触发文件就多一轮阶段同步（knowledge-sync 格按触发文件算范围），所以先问一句"
          "「它挡不挡着这一批的课题」——")
    print(f"               不挡：从暂存区撤出去（git restore --staged <路径>，工作区的改动留着），"
          f"把这件事记进 .claude/kb/checks-owed.md 往后延；")
    print(f"               挡着：在 {SCOPE_FILE} 里补一行「<路径><制表符>#<为什么>」，并认下它带来的那一轮阶段同步。")
def shell_quote(text):
    """bash 能直接粘贴的引用：平常的字符原样，含空白、引号、反斜杠或控制字符的写成 $'…'。"""
    if re.fullmatch(r"[A-Za-z0-9_./:=@%+,\-]+", text):
        return text
    if "'" not in text and text.isprintable():
        return "'" + text + "'"
    escapes = {"\\": "\\\\", "'": "\\'", "\t": "\\t", "\n": "\\n", "\r": "\\r"}
    return "$'" + "".join(escapes.get(ch, ch if ch.isprintable() else "\\x%02x" % ord(ch)) for ch in text) + "'"


def prune_command(lines):
    """删掉这几行（按整行内容匹配）并回读确认它们不在了的一条命令，在仓库根跑。
    弄坏开关 BATCH_SCOPE_BREAK=prune-command-misses-last：命令里漏掉最后一行，照它跑完这一格仍红。"""
    patterns = [raw for _, _, raw in lines]
    if os.environ.get("BATCH_SCOPE_BREAK", "") == "prune-command-misses-last" and len(patterns) > 1:
        patterns = patterns[:-1]
    selected = " ".join("-e " + shell_quote(text) for text in patterns)
    pruned_file = f"{SCOPE_FILE}.pruned"
    return (f"grep -vxF {selected} {SCOPE_FILE} > {pruned_file}; [ $? -le 1 ] && mv {pruned_file} {SCOPE_FILE} "
            f"&& ! grep -qxF {selected} {SCOPE_FILE} && echo '已删 {len(lines)} 行，回读确认它们已不在 {SCOPE_FILE} 里' "
            f"|| {{ rm -f {pruned_file}; echo '没删成，请手工核对 {SCOPE_FILE}'; }}")


if stale:
    failed = True
    stale_set = set(stale)
    stale_lines = [entry for entry in registered_lines if entry[1] in stale_set]
    print(f"  ✗ {len(stale)} 个登记的路径不是这一批的触发文件（登记项不起作用，会让下一批白放行）：")
    for number, path, _ in stale_lines:
        print(f"      {SCOPE_FILE}:{number}：{path}")  # gate-lint:detail
    print(f"     → 怎么办：上一批提交完没清表就会这样，把这几行删掉；"
          f"路径写错了就照 git status 的写法改（从仓库根起，中文路径不要带引号）。")
    print(f"               上面每行的格式是「文件:行号：路径」：冒号后面那一段才是路径，前面的 {SCOPE_FILE}:行号 是登记行在哪，不是路径的一部分。")
    print(f"               要整行删掉上面列的这 {len(stale_lines)} 行：在仓库根粘贴下面这一条（按整行内容匹配，别的行不动，删完回读确认它们已不在）：")
    print(f"               {prune_command(stale_lines)}")
if no_reason:
    failed = True
    print(f"  ✗ {len(no_reason)} 行登记没写理由：")
    for entry in no_reason:
        print(f"      {entry}")                                  # gate-lint:detail
    print(f"     → 怎么办：路径后面写 #，# 后面写为什么这一批非碰它不可，理由不许省"
          f"（规矩同 .claude/abbreviations 与 .claude/naming-lint-exclude）。")
if bad_path:
    failed = True
    print(f"  ✗ {len(bad_path)} 行登记的路径不是从仓库根起写的：")
    for entry in bad_path:
        print(f"      {entry}")                                  # gate-lint:detail
    print("     → 怎么办：照 git status 的写法写（例 .claude/gate.d/doc-process-records.sh），别写 ./ 开头或相对别处的路径。")
if failed:
    sys.exit(1)

print(f"  ✓ 这一批的 {len(triggers)} 个触发文件都登记进了 {SCOPE_FILE}，登记的 {len(registered)} 个都在这一批里"
      f"（相对 HEAD 共 {len(changed_files)} 个改动路径，另外 {len(changed_files) - len(triggers)} 个不是触发文件、不用登记；"
      f"触发文件的判据是 {TRIGGER_TABLE} 的 {len(patterns)} 条正则）")
PY
}

# ── verdict-names-local-samples ─────────────────────────────────────────────
check_verdict_names_local_samples() {
  ((inside_git)) || { not_git_repository; return; }
  load_added_since_gate || {
    echo "  ✗ 取不到这次改动新增了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到新增的判决文件时这一格什么都没比，不是通过。"
    return 1
  }
  local -a new_verdicts=()
  mapfile -t new_verdicts < <(grep -E '^research/prompts/[^/]+-main-verification\.md$' <<<"$added_since_gate" || true)
  if ((${#new_verdicts[@]} == 0)); then
    echo "  ! 这次改动没有新写的判决 research/prompts/<轮>-main-verification.md，这一格无对象可判（基准 $gate_base）"
    return 77
  fi
  python3 - "$gate_base" "${new_verdicts[@]}" <<'PY'
import os
import sys

base = sys.argv[1]
verdict_paths = [path for path in sys.argv[2:] if path]
prompts_directory = "research/prompts"
suffix = "-main-verification.md"
breakage = os.environ.get("VERDICT_LOCAL_SAMPLES_BREAK", "")

top_level_names = sorted(os.listdir(prompts_directory)) if os.path.isdir(prompts_directory) else []


def samples_of(round_name):
    """那一轮本地腿的样本文件名（research/prompts/ 顶层的普通文件）。"""
    prefix = round_name + "-local-"
    found = []
    for name in top_level_names:
        if not (name.startswith(prefix) and name.endswith(".md")):
            continue
        is_void = "-output-void" in name
        if not ("-output-s" in name or is_void):
            continue
        if is_void and breakage == "skip-void":
            continue
        if not os.path.isfile(os.path.join(prompts_directory, name)):
            continue
        found.append(name)
    return found


named_total = 0
with_samples = 0
without_samples = []
skipped_empty = []
missing_by_verdict = []
for verdict_path in verdict_paths:
    if not os.path.isfile(verdict_path):
        continue
    round_name = os.path.basename(verdict_path)[: -len(suffix)]
    samples = samples_of(round_name)
    if not samples:
        without_samples.append(verdict_path)
        continue
    with open(verdict_path, encoding="utf-8", errors="replace") as handle:
        verdict_text = handle.read()
    judged = 0
    missing = []
    for name in samples:
        sample_path = os.path.join(prompts_directory, name)
        if os.path.getsize(sample_path) == 0 and breakage != "require-empty":
            skipped_empty.append(sample_path)
            continue
        judged += 1
        if name in verdict_text:
            named_total += 1
        else:
            missing.append(name)
    if judged:
        with_samples += 1
    else:
        without_samples.append(verdict_path)
    if missing:
        missing_by_verdict.append((verdict_path, missing))

verdict_count = len(verdict_paths)
if missing_by_verdict:
    missing_count = sum(len(missing) for _path, missing in missing_by_verdict)
    print(f"  ✗ 新写的判决里 {len(missing_by_verdict)} 份没按路径点名那一轮本地腿的全部样本（共漏 {missing_count} 份）：")  # gate-lint:summary
    for verdict_path, missing in missing_by_verdict:
        print(f"      {verdict_path} 漏 {len(missing)} 份：")  # gate-lint:detail
        for name in missing:
            print(f"        {name}")  # gate-lint:detail
    print("     → 怎么办：逐份读这一轮本地腿交回的样本（干净的、通读判带损坏的、损坏闸留下的 -output-void<n>.md），在这份判决里按路径列出、")
    print("               各标干净或参考，写它说了什么、去向如何（.claude/rules/three-way-inference.md「判决由主 agent 做，不由投票做」一节")
    print("               「本地腿交回的每一份样本都读，判决里逐份按路径列出」那一条）；不改样本、不删样本。")
    sys.exit(1)

print(f"  ✓ 新写的判决 {verdict_count} 份：{with_samples} 份逐份点名了那一轮本地腿的 {named_total} 份样本，"
      f"{len(without_samples)} 份那一轮没有本地腿样本；0 字节样本跳过 {len(skipped_empty)} 份（基准 {base}）")
if without_samples:
    print("    没有本地腿样本的：" + "、".join(without_samples))
if skipped_empty:
    print(f"    0 字节样本跳过 {len(skipped_empty)} 份：" + "、".join(skipped_empty))
PY
}

# ── crates-adversarial-review ───────────────────────────────────────────────
# user_exemption_reason <豁免表> <路径>：豁免表一行三段（路径、豁免时那一版的 sha256、为什么），打印这一路径、这一份内容那一行的理由，没有就不打印。
# 按内容哈希认：那份文件再改一个字就对不上。删掉的文件没有内容可算哈希，第二段写 deleted，只在文件确实不在时对得上。
# 豁免表不再加行（只缩不涨，check_review_exempt_frozen 判）；表里已有的行照旧生效。两格共用：.claude/agent-def-review-exempt 与 .claude/crates-review-exempt。
user_exemption_reason() {
  local table="$1" path="$2" now
  [[ -f "$table" ]] || return 0
  if [[ -f "$path" ]]; then now="$(sha256sum -- "$path" | cut -d' ' -f1)"; else now="deleted"; fi
  if [[ "${CRATES_ADVERSARIAL_REVIEW_BREAK:-}" == exempt-hash-ignored ]]; then
    awk -F'\t' -v path="$path" '$1 !~ /^#/ && NF >= 3 && $1 == path { print $3; exit }' "$table"; return 0
  fi
  awk -F'\t' -v path="$path" -v hash="$now" \
    '$1 !~ /^#/ && NF >= 3 && $1 == path && $2 == hash { print $3; exit }' "$table"
}

# new_verdict_paths：这次改动新写的判决（一行一个），要先调过 load_added_since_gate。crates-adversarial-review、agent-def-adversarial-review 与 code-round-snapshot 三格共用这一份取法。
new_verdict_paths() {
  grep -E '^research/prompts/.*-main-verification\.md$' <<<"$added_since_gate" || true
}

# ── 判决里「这一轮被判过的文件」一节：crates-adversarial-review 与 agent-def-adversarial-review 两格只在这一节里找路径 ──
# 判据在文件头「点名一节」。下面这组函数都在当前 shell 里调（缓存是全局数组，经 $( ) 子 shell 传不回来）。
declare -A review_scope_cache=()
declare -A review_pending_rows=()
review_pending_loaded=0
review_scope_result=""

# load_review_pending：读挂账表第一列（判决路径）进 review_pending_rows，一趟只读一次。表自己合不合规是 check_review_pending_table 的事。
load_review_pending() {
  ((review_pending_loaded)) && return 0
  review_pending_loaded=1
  [[ -f "$REVIEW_PENDING_TABLE" ]] || return 0
  local path reason
  while IFS=$'\t' read -r path reason _ || [[ -n "$path" ]]; do
    path="${path//[$' \r']/}"
    [[ -z "$path" || "$path" == '#'* ]] && continue
    review_pending_rows[$path]="$reason"
  done < "$REVIEW_PENDING_TABLE"
  return 0
}

# review_scope_text <判决>：把「在这份判决里按路径点名」认的那段文字放进 review_scope_result。
# 默认是那一节的正文；挂账表里登记的判决、以及弄坏开关打开时，认整份正文（改前的整份子串匹配）。
review_scope_text() {
  local verdict="$1"
  if [[ -n "${review_scope_cache[$verdict]+set}" ]]; then review_scope_result="${review_scope_cache[$verdict]}"; return 0; fi
  load_review_pending
  if [[ "$review_break" == whole-verdict-substring || -n "${review_pending_rows[$verdict]+set}" ]]; then
    review_scope_result="$(cat -- "$verdict")"
  else
    review_scope_result="$(review_section_text "$verdict")"
    # 节里写了否定词的行（「没攻它」「没判」）不算点名；弄坏开关 negated-lines-counted 退回改前：节里出现就算
    [[ "$review_break" == negated-lines-counted ]] || review_scope_result="$(review_drop_negated <<<"$review_scope_result")"
  fi
  review_scope_cache[$verdict]="$review_scope_result"
}

# review_lists_path <判决> <路径>：这份判决点名了这个路径吗（字面子串，范围见 review_scope_text）
review_lists_path() {
  [[ "$review_break" == lists-path-never ]] && return 1      # 弄坏：没有一份判决点名任何路径
  review_scope_text "$1"
  # 路径里没有换行，整段文字里的子串判与逐行字面匹配同义；不为每一对（判决, 路径）起一个 grep
  [[ "$review_scope_result" == *"$2"* ]]
}

# review_mention_note <路径> <判决（一行一个）>：没被点名的路径，若某份新判决在正文别处提到过它，打印一句提示（给红格的出路用），没有就不打印
review_mention_note() {
  local path="$1" verdicts="$2" verdict section
  local -a mentioned=() negated=()
  while IFS= read -r verdict; do
    [[ -z "$verdict" || ! -f "$verdict" ]] && continue
    section="$(review_section_text "$verdict")"
    if grep -qF -- "$path" <<<"$section"; then negated+=("${verdict##*/}")
    elif grep -qF -- "$path" "$verdict"; then mentioned+=("${verdict##*/}"); fi
  done <<<"$verdicts"
  if ((${#negated[@]})); then
    printf '（%s 在「%s」一节里只在写了否定词（%s 之一）的行上提到它，不算点名）' "${negated[*]}" "$REVIEWED_SECTION_HEADING" "${REVIEWED_SECTION_NEGATIONS// /、}"
    return 0
  fi
  ((${#mentioned[@]})) || return 0
  printf '（%s 只在正文别处提到了它，没列在「%s」一节里）' "${mentioned[*]}" "$REVIEWED_SECTION_HEADING"
}

# ── 判决之后又改过没有：拿点名它的那一轮开工快照清单里的 sha256 比此刻的内容（文件头「点名一节」判据 ⑥） ──
review_snapshot_state=""
review_snapshot_detail=""
review_snapshot_matched=0
review_outcome=""
review_outcome_note=""

# review_snapshot_compare <判决> <路径>：结果放进 review_snapshot_state——
#   match（那一轮快照目录 research/prompts/<轮>-snapshot/ 里某份 *sha256*.txt 清单记的这份文件的哈希等于此刻被判的树里的内容；--staged 时那棵树就是暂存进来的那一份）、
#   mismatch（清单里有它的行、没有一行对得上；review_snapshot_detail 写哪份清单记的什么、此刻是什么）、
#   deleted（文件此刻不在，没有内容可比）、none（没有快照目录、或清单里没有它的行）。
# 清单行是 sha256sum 的格式：64 位十六进制、两个空格（二进制模式是一个空格接 *）、路径（开头的 ./ 不碍）。
review_snapshot_compare() {
  local verdict="$1" path="$2" round directory manifest now hash recorded=""
  review_snapshot_state=none; review_snapshot_detail=""
  [[ -f "$path" ]] || { review_snapshot_state=deleted; return 0; }
  round="${verdict##*/}"; round="${round%-main-verification.md}"
  directory="research/prompts/${round}-snapshot"
  [[ -d "$directory" ]] || return 0
  now="$(sha256sum -- "$path" | cut -d' ' -f1)"
  for manifest in "$directory"/*sha256*.txt; do
    [[ -f "$manifest" ]] || continue
    while IFS= read -r hash; do
      if [[ "$hash" == "$now" ]]; then review_snapshot_state=match; review_snapshot_detail="$manifest"; return 0; fi
      recorded+="${recorded:+、}$manifest 记的 ${hash:0:12}…"
    done < <(awk -v path="$path" '
      { sub(/\r$/, "") }
      length($1) == 64 && $1 ~ /^[0-9a-f]+$/ && (substr($0, 65, 2) == "  " || substr($0, 65, 2) == " *") {
        listed = substr($0, 67); sub(/^\.\//, "", listed)
        if (listed == path) print $1
      }' "$manifest")
  done
  if [[ -n "$recorded" ]]; then review_snapshot_state=mismatch; review_snapshot_detail="$recorded，此刻是 ${now:0:12}…"; fi
  return 0
}

# review_judge_named <路径> <判决（一行一个）>：这份文件在这次新写的判决里的去向，放进 review_outcome（两格共用；豁免表由调用方接着查）：
#   named（某份判决在那一节里点名了它，而且那一轮快照清单记的哈希与此刻的内容对得上，或文件已删）、
#   unhashed（点名了，可点名它的判决那一轮都没有快照、或清单里没有它的行，没比）、
#   stale（点名了，快照清单里有它的行而都对不上：判决之后又改过；review_outcome_note 写哪份判决、哪份清单记的什么、此刻是什么）、
#   missing（没有一份判决在那一节里点名它）。
# 弄坏开关 REVIEW_SECTION_BREAK=snapshot-hash-unchecked 退回改前：点名了就算 named，不比哈希。
review_judge_named() {
  local path="$1" verdicts="$2" verdict unhashed=0 notes=""
  review_outcome=missing; review_outcome_note=""
  while IFS= read -r verdict; do
    [[ -z "$verdict" || ! -f "$verdict" ]] && continue
    review_lists_path "$verdict" "$path" || continue
    if [[ "$review_break" == snapshot-hash-unchecked ]]; then review_outcome=named; return 0; fi
    review_snapshot_compare "$verdict" "$path"
    case "$review_snapshot_state" in
      match) review_outcome=named; review_snapshot_matched=$((review_snapshot_matched + 1)); return 0 ;;
      deleted) review_outcome=named; return 0 ;;
      mismatch) notes+="${notes:+；}${verdict##*/} 那一轮的 $review_snapshot_detail" ;;
      *) unhashed=1 ;;
    esac
  done <<<"$verdicts"
  if [[ -n "$notes" ]]; then review_outcome=stale; review_outcome_note="$notes"
  elif ((unhashed)); then review_outcome=unhashed; fi
  return 0
}

# review_report_stale <这一格的对象叫什么> <豁免表> <条目…>：判决之后又改过的那几份，打红行与出路（两格共用）
review_report_stale() {
  local what="$1" table="$2"; shift 2
  echo "  ✗ ${what}里 $# 个在判决之后又改过（点名它的新判决那一轮开工快照 research/prompts/<轮>-snapshot/ 的 sha256 清单记的哈希，与此刻被判的树里——--staged 时是暂存进来的那一份——的内容对不上）："   # gate-lint:summary
  printf '      %s\n' "$@"   # gate-lint:detail
  echo "     → 怎么办：判决之后又改过：再开一轮（派腿之前重记快照，新判决在「$REVIEWED_SECTION_HEADING」一节里点名它）；"
  echo "               豁免表 $table 不再加行，表里已有的行（路径与内容哈希都对得上）照旧生效。"
}

# review_report_snapshot_coverage <没比的路径说明> <没比的路径…>：成功行之后报快照比对了几份、没比的逐个列出（两格共用；弄坏开关 snapshot-hash-unchecked 下不报）
review_report_snapshot_coverage() {
  local why="$1"; shift
  [[ "$review_break" == snapshot-hash-unchecked ]] && return 0
  echo "    判决之后改没改过（拿那一轮开工快照清单里的 sha256 比此刻的内容）：${review_snapshot_matched} 个对得上，$# 个没比（$why）"
  (($#)) && printf '      没比：%s\n' "$@"
  return 0
}

# report_review_pending_in_use <判决（一行一个）>：这一趟新写的判决里，有哪几份是按挂账表认整份正文的，逐个报出，没有就不报
report_review_pending_in_use() {
  local verdict listed=0
  load_review_pending
  while IFS= read -r verdict; do
    [[ -n "$verdict" && -n "${review_pending_rows[$verdict]+set}" ]] || continue
    if ((listed == 0)); then echo "    按挂账表 $REVIEW_PENDING_TABLE 认整份正文里提到的新写判决（没有「$REVIEWED_SECTION_HEADING」一节；只缩不涨，补了那一节的删掉这一行）："; fi
    listed=$((listed + 1))
    printf '      %s：%s\n' "$verdict" "${review_pending_rows[$verdict]}"
  done <<<"$1"
  return 0
}

# check_review_pending_table：挂账表自己合不合规，有问题打印并返回 1。两格各在开头调一次，没有对象可判的那一趟也判，表烂了不靠有改动才发现。
# 表不存在不算红（少一张表只会让判据更严）。判据：每行「判决路径<制表符>理由」两段非空、理由 8 个字以上、路径是 research/prompts/ 顶层的 *-main-verification.md 且文件在、
# 同一路径不登两次；只缩不涨：基准（gate 取法的基准）里有这张表时，表里不许出现基准里没有的路径。
check_review_pending_table() {
  [[ -f "$REVIEW_PENDING_TABLE" ]] || return 0
  python3 - "$REVIEW_PENDING_TABLE" "$gate_base" "$inside_git" "$REVIEWED_SECTION_HEADING" "$review_break" "$LAG_BASELINE_LIBRARY" <<'PY'
import importlib.util, re, sys

table, base, inside_git, heading, breakage = sys.argv[1], sys.argv[2], sys.argv[3] == "1", sys.argv[4], sys.argv[5]
lag_baseline_spec = importlib.util.spec_from_file_location("lag_baseline", sys.argv[6])
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)
VERDICT_PATH = re.compile(r"research/prompts/[^/]+-main-verification\.md")
MINIMUM_REASON = 8

problems, rows = [], {}
with open(table, encoding="utf-8") as handle:
    for number, fields in lag_baseline.content_rows(handle.read()):
        path = fields[0].strip()
        reason = fields[1].strip() if len(fields) > 1 else ""
        if breakage == "pending-rows-unchecked":
            rows[path] = number
            continue
        if len(fields) < 2 or not path or len(reason) < MINIMUM_REASON:
            problems.append(f"第 {number} 行不是「判决路径<制表符>理由」两段，或理由不到 {MINIMUM_REASON} 个字：{fields[0].strip()}")
            continue
        if not VERDICT_PATH.fullmatch(path):
            problems.append(f"第 {number} 行的路径不是 research/prompts/ 顶层的 *-main-verification.md：{path}")
            continue
        if path in rows:
            problems.append(f"第 {number} 行重复登记了 {path}（第 {rows[path]} 行已经有）")
            continue
        rows[path] = number
        try:
            open(path, "rb").close()
        except OSError:
            problems.append(f"第 {number} 行指向的判决不存在：{path}")

baseline = None
if inside_git and base and breakage != "pending-growth-ignored":
    # 「基准里还没有这张表」与「git 取不到基准那一版」分开（lib-lag-baseline.py 的 baseline_rows）：后者判红，不当成没有基线静默不比
    baseline_rows, baseline_problem = lag_baseline.baseline_rows(base, table)
    if baseline_problem is not None:
        problems.append(f"{baseline_problem.sentence()}，只缩不涨没比")
    elif baseline_rows is not None:
        baseline = {fields[0].strip() for _number, fields in baseline_rows}
if baseline is not None:
    for path, number in rows.items():
        if path not in baseline:
            problems.append(f"第 {number} 行 {path} 是基准 {base} 之后新加的：挂账表只缩不涨")

if problems:
    print(f"  ✗ 点名一节的挂账表 {table} 有 {len(problems)} 处不合规：")   # gate-lint:summary
    for problem in problems:
        print(f"      {problem}")   # gate-lint:detail
    print(f"     → 怎么办：表里一行一份判决「research/prompts/<轮>-main-verification.md<制表符>理由（8 个字以上）」，指到的判决要在仓里；")
    print(f"               只缩不涨：新写的判决一律在「{heading}」一节里按路径列出文件，不往表里加行；补了那一节或判决已不再是这一轮新写的，把那一行删掉。")
    sys.exit(1)
PY
}

# load_transition_rows <过渡挂账表>：读第一列（路径）与第二列（理由）进 transition_rows，一格只读一张；弄坏开关 transition-ignored 下不读（登记的路径照旧要点名）。
# 表自己合不合规是 check_review_transition_table 的事。
declare -A transition_rows=()
load_transition_rows() {
  transition_rows=()
  [[ -f "$1" && "$review_break" != transition-ignored ]] || return 0
  local path reason
  while IFS=$'\t' read -r path reason _ || [[ -n "$path" ]]; do
    path="${path//[$' \r']/}"
    [[ -z "$path" || "$path" == '#'* ]] && continue
    transition_rows[$path]="${reason%$'\r'}"
  done < "$1"
  return 0
}

# check_review_exempt_frozen <豁免表>：豁免表只缩不涨（文件头「豁免表与过渡挂账表」①）：基准（gate 取法的基准）里有这张表时，表里不许出现基准里没有的行（按「路径、sha256」两列认），
# 基准里还没有这张表时不比，表不存在不算红。两格各在开头调一次，没有对象可判的那一趟也判。弄坏开关 exempt-growth-ignored 不比。
check_review_exempt_frozen() {
  [[ -f "$1" ]] || return 0
  python3 - "$1" "$gate_base" "$inside_git" "$review_break" "$LAG_BASELINE_LIBRARY" <<'PY'
import importlib.util, sys

table, base, inside_git, breakage = sys.argv[1], sys.argv[2], sys.argv[3] == "1", sys.argv[4]
lag_baseline_spec = importlib.util.spec_from_file_location("lag_baseline", sys.argv[5])
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)


def row_key(fields):
    return (fields[0].strip(), fields[1].strip() if len(fields) > 1 else "")


problems = []
if inside_git and base and breakage != "exempt-growth-ignored":
    baseline_rows, baseline_problem = lag_baseline.baseline_rows(base, table)
    if baseline_problem is not None:
        problems.append(f"{baseline_problem.sentence()}，只缩不涨没比")
    elif baseline_rows is not None:
        baseline = {row_key(fields) for _number, fields in baseline_rows}
        with open(table, encoding="utf-8") as handle:
            for number, fields in lag_baseline.content_rows(handle.read()):
                path, digest = row_key(fields)
                if (path, digest) not in baseline:
                    problems.append(f"第 {number} 行 {path}（sha256 {digest[:12]}…）是基准 {base} 之后新加的")
if problems:
    print(f"  ✗ 豁免表 {table} 只缩不涨，有 {len(problems)} 处不合规：")   # gate-lint:summary
    for problem in problems:
        print(f"      {problem}")   # gate-lint:detail
    print("     → 怎么办：三方不再豁免，豁免表不再加行：把新加的行删掉，改走一轮三方（.claude/rules/three-way-inference.md），")
    print("               判决在「## 这一轮被判过的文件」一节里按路径点名这份文件；表里已有的行照旧生效，补完一轮三方就删一行。")
    sys.exit(1)
PY
}

# check_review_transition_table <过渡挂账表> <这张表认的路径叫什么> <路径形态…>：过渡挂账表自己合不合规（文件头「豁免表与过渡挂账表」②）。
# 路径形态一项一个参数，写成「认路径的正则<制表符>排除的正则」（排除的可以空）。每行「路径<制表符>理由」两段非空、理由 8 个字以上、路径对得上某一项形态、同一路径不登两次；
# 只缩不涨：基准里有这张表时，表里不许出现基准里没有的路径。表不存在不算红；两格各在开头调一次，没有对象可判的那一趟也判。
# 弄坏开关 transition-rows-unchecked 不核行，transition-growth-ignored 不和基准比。
check_review_transition_table() {
  [[ -f "$1" ]] || return 0
  local table="$1" what="$2"
  shift 2
  python3 - "$table" "$what" "$gate_base" "$inside_git" "$review_break" "$LAG_BASELINE_LIBRARY" "$@" <<'PY'
import importlib.util, re, sys

table, what, base, inside_git, breakage = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4] == "1", sys.argv[5]
lag_baseline_spec = importlib.util.spec_from_file_location("lag_baseline", sys.argv[6])
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)
FORMS = [tuple((form.split("\t") + [""])[:2]) for form in sys.argv[7:]]
MINIMUM_REASON = 8


def allowed(path):
    return any(re.search(include, path) and not (exclude and re.search(exclude, path)) for include, exclude in FORMS)


problems, rows = [], {}
with open(table, encoding="utf-8") as handle:
    for number, fields in lag_baseline.content_rows(handle.read()):
        path = fields[0].strip()
        reason = fields[1].strip() if len(fields) > 1 else ""
        if breakage == "transition-rows-unchecked":
            rows[path] = number
        elif len(fields) < 2 or not path or len(reason) < MINIMUM_REASON:
            problems.append(f"第 {number} 行不是「路径<制表符>理由」两段，或理由不到 {MINIMUM_REASON} 个字：{path}")
        elif not allowed(path):
            problems.append(f"第 {number} 行的路径不是{what}：{path}")
        elif path in rows:
            problems.append(f"第 {number} 行重复登记了 {path}（第 {rows[path]} 行已经有）")
        else:
            rows[path] = number

baseline = None
if inside_git and base and breakage != "transition-growth-ignored":
    baseline_rows, baseline_problem = lag_baseline.baseline_rows(base, table)
    if baseline_problem is not None:
        problems.append(f"{baseline_problem.sentence()}，只缩不涨没比")
    elif baseline_rows is not None:
        baseline = {fields[0].strip() for _number, fields in baseline_rows}
if baseline is not None:
    for path, number in rows.items():
        if path not in baseline:
            problems.append(f"第 {number} 行 {path} 是基准 {base} 之后新加的：挂账表只缩不涨")

if problems:
    print(f"  ✗ 过渡挂账表 {table} 有 {len(problems)} 处不合规：")   # gate-lint:summary
    for problem in problems:
        print(f"      {problem}")   # gate-lint:detail
    print(f"     → 怎么办：表里一行一条「路径<制表符>理由（{MINIMUM_REASON} 个字以上）」，路径要是{what}；")
    print("               只缩不涨：新改的文件一律在判决「## 这一轮被判过的文件」一节里按路径点名，不往表里加行；判决点名了它、或那一份不再有改动，就把那一行删掉。")
    sys.exit(1)
PY
}

# governance_objects：改动范围里被管的路径，一行「<类名><制表符><路径>」，一个路径只算头一个对上的类（类与次序见 GOVERNANCE_CLASS_NAMES）。
# 弄坏开关（AGENT_DEF_ADVERSARIAL_REVIEW_BREAK）definitions-only 只认头一类；要先调过 load_changed_since_gate。
governance_objects() {
  local index matched path last=$((${#GOVERNANCE_CLASS_NAMES[@]} - 1))
  local -A seen=()
  [[ "${AGENT_DEF_ADVERSARIAL_REVIEW_BREAK:-}" == definitions-only ]] && last=0
  for ((index = 0; index <= last; index++)); do
    matched="$(grep -E "${GOVERNANCE_CLASS_PATTERNS[index]}" <<<"$changed_since_gate" || true)"
    if [[ -n "$matched" && -n "${GOVERNANCE_CLASS_EXCLUDES[index]}" ]]; then
      matched="$(grep -vE "${GOVERNANCE_CLASS_EXCLUDES[index]}" <<<"$matched" || true)"
    fi
    while IFS= read -r path; do
      [[ -z "$path" || -n "${seen[$path]+set}" ]] && continue
      seen[$path]=1
      printf '%s\t%s\n' "${GOVERNANCE_CLASS_NAMES[index]}" "$path"
    done <<<"$matched"
  done
}

check_crates_adversarial_review() {
  ((inside_git)) || { not_git_repository; return; }
  load_changed_since_gate || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    return 1
  }
  check_review_pending_table || return 1
  check_review_exempt_frozen "$CRATES_EXEMPT_TABLE" || return 1
  check_review_transition_table "$CRATES_TRANSITION_TABLE" "非产品角色的包（$CRATE_ROLE_TABLE）的 src、各包 tests 里的 .rs 或 crates/mutations.tsv" "$CRATES_TRANSITION_PATHS"$'\t' || return 1
  load_transition_rows "$CRATES_TRANSITION_TABLE"
  local sources verdicts source verdict_count why
  local -a missing=() exempted=() stale=() unhashed=() transitioned=()
  local count=0
  local exempt_table="$CRATES_EXEMPT_TABLE"
  sources="$(grep -E "$CRATES_REVIEW_OBJECTS" <<<"$changed_since_gate" || true)"
  # 弄坏开关 test-code-skipped：测试代码（CRATES_TRANSITION_PATHS 那一类）不算对象
  if [[ "${CRATES_ADVERSARIAL_REVIEW_BREAK:-}" == test-code-skipped ]]; then
    sources="$(grep -vE "$CRATES_TRANSITION_PATHS" <<<"$sources" || true)"
  fi
  # 只改了注释的 .rs 不走三方：去掉注释与空白之后代码词元与基准逐个相同（判法文件头写了哪些注释照代码算）；不是 .rs 的（crates/mutations.tsv）判法照代码算
  local comment_only="" comment_only_count=0
  local -a source_list=()
  if [[ -n "$sources" && "${CRATES_ADVERSARIAL_REVIEW_BREAK:-}" != comments-count-as-code ]]; then
    if [[ ! -f "$COMMENT_ONLY_JUDGE" ]]; then
      echo "  ✗ 找不到判「只改了注释」的判法 $COMMENT_ONLY_JUDGE"
      echo "     → 怎么办：照 CLAUDE.md「规范从哪来」那一行同步规范副本（判法在上游 scripts/comment-only-change.py），不在这道门禁里另写一份。"
      return 1
    fi
    mapfile -t source_list <<<"$sources"
    if ! comment_only="$(python3 "$COMMENT_ONLY_JUDGE" "$ROOT" "$gate_base" "${source_list[@]}")"; then
      echo "  ✗ 判不出哪些 crates 代码文件只改了注释（$COMMENT_ONLY_JUDGE 判不了，原话在上面；基准 $gate_base）"
      echo "     → 怎么办：核基准在不在、改动清单里的路径对不对，修好再跑；判不了的不当只改注释。"
      return 1
    fi
    if [[ -n "$comment_only" ]]; then
      sources="$(grep -vxF -f <(printf '%s\n' "$comment_only") <<<"$sources" || true)"
      comment_only_count="$(grep -c . <<<"$comment_only" || true)"
    fi
  fi
  if [[ -z "$sources" ]]; then
    local comment_only_note=""
    ((comment_only_count)) && comment_only_note="；只改了注释的 ${comment_only_count} 个不走三方"
    echo "  ! 这次改动没碰 crates/ 下要判的代码（各包 src 与 tests 里的 .rs、crates/mutations.tsv${comment_only_note}），这一格无对象可判（基准 $gate_base）"
    report_comment_only_sources "$comment_only" "$comment_only_count"
    return 77
  fi
  load_added_since_gate || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    return 1
  }
  verdicts="$(new_verdict_paths)"
  while IFS= read -r source; do
    [[ -z "$source" ]] && continue
    count=$((count + 1))
    # 登记在过渡挂账表里的路径不要求点名；别的点名了、快照对得上（或没快照可比）的有去向，又改过的与没点名的先查豁免表（按此刻的内容哈希）
    if [[ -n "${transition_rows[$source]+set}" ]]; then transitioned+=("$source：${transition_rows[$source]}"); continue; fi
    review_judge_named "$source" "$verdicts"
    case "$review_outcome" in
      named) continue ;;
      unhashed) unhashed+=("$source"); continue ;;
    esac
    why="$(user_exemption_reason "$exempt_table" "$source")"
    if [[ -n "$why" ]]; then exempted+=("$source：$why"); continue; fi
    if [[ "$review_outcome" == stale ]]; then stale+=("$source（$review_outcome_note）"); continue; fi
    missing+=("$source$(review_mention_note "$source" "$verdicts")")
  done <<<"$sources"
  if ((${#stale[@]})); then
    review_report_stale "crates 改动" "$exempt_table" "${stale[@]}"
  fi
  if ((${#missing[@]})); then
    echo "  ✗ crates 改动里 ${#missing[@]} 个文件没有三方判决点名（这次改动新写的 research/prompts/*-main-verification.md 里一份都没在「$REVIEWED_SECTION_HEADING」一节里按路径列出它；正文别处提到不算，节里写了没攻、没判这类否定词的行也不算；被改过的旧判决不算）："   # gate-lint:summary
    printf '      %s\n' "${missing[@]}"   # gate-lint:detail
    echo "     → 怎么办：写代码之后、提交之前走一轮三方（.claude/rules/three-way-inference.md）：材料带上 diff，正推 / 反推 / 找反例三条腿各攻一遍，"
    echo "               主 agent 的判决写进 research/prompts/<题>-r<n>-main-verification.md，在判决里写一节，标题一行逐字是「$REVIEWED_SECTION_HEADING」（二级标题），"
    echo "               节里一行一个，逐个按路径列出这一轮判过的 crates 文件（产品代码、测试代码与 crates/mutations.tsv 都算；节到下一个一、二级标题为止；打中的写回代码再改一轮）："
    echo "                 $REVIEWED_SECTION_HEADING"
    echo "                 - crates/<包>/src/<文件>.rs：<这一轮判了它的什么>"
    echo "               点名写进新的判决，不往旧判决里补；豁免表 $exempt_table 不再加行（表里已有的行照旧生效）；"
    echo "               存量测试代码没补完三方的登记在挂账表 $CRATES_TRANSITION_TABLE（只缩不涨，新改的文件不往里加）。"
    report_comment_only_sources "$comment_only" "$comment_only_count"
    return 1
  fi
  if ((${#stale[@]})); then
    report_comment_only_sources "$comment_only" "$comment_only_count"
    return 1
  fi
  verdict_count="$(grep -c . <<<"$verdicts" || true)"
  if ((${#exempted[@]} || ${#transitioned[@]})); then
    local extras=""
    ((${#exempted[@]})) && extras+="，另有 ${#exempted[@]} 个按 $exempt_table 豁免"
    ((${#transitioned[@]})) && extras+="，另有 ${#transitioned[@]} 个按 $CRATES_TRANSITION_TABLE 挂账、不要求点名"
    echo "  ✓ crates 改动 ${count} 个文件都有去向（新写的判决文件 ${verdict_count} 份${extras}，基准 $gate_base）"
    if ((${#exempted[@]})); then
      echo "    豁免的 ${#exempted[@]} 个（按内容哈希认，那份文件再改一个字就不再豁免）："
      printf '      %s\n' "${exempted[@]}"
    fi
    if ((${#transitioned[@]})); then
      echo "    挂账的 ${#transitioned[@]} 个（登记的路径这一格不要求点名；只缩不涨，补完一轮三方删一行）："
      printf '      %s\n' "${transitioned[@]}"
    fi
  else
    echo "  ✓ crates 改动 ${count} 个文件都有三方判决点名（新写的判决文件 ${verdict_count} 份，基准 $gate_base）"
  fi
  review_report_snapshot_coverage "那一轮没有开工快照、或快照清单里没有它的行；快照立没立住归 code-round-snapshot 格判" ${unhashed[@]+"${unhashed[@]}"}
  report_review_pending_in_use "$verdicts"
  report_comment_only_sources "$comment_only" "$comment_only_count"
}

# report_comment_only_sources <只改注释的路径（一行一个）> <个数>：逐个报出只改了注释、不走三方的 crates 代码，0 个不报
report_comment_only_sources() {
  local listed="$1" listed_count="$2"
  ((listed_count)) || return 0
  echo "    只改了注释、不走三方的 crates 代码 ${listed_count} 个（判法 $COMMENT_ONLY_JUDGE）："
  printf '%s\n' "$listed" | sed 's/^/      /'
}

# ── code-round-snapshot ─────────────────────────────────────────────────────
# 对象与判据在文件头「code-round-snapshot」那一节。判决的取法是 new_verdict_paths，与 crates-adversarial-review 同一份。
check_code_round_snapshot() {
  ((inside_git)) || { not_git_repository; return; }
  load_added_since_gate || {
    echo "  ✗ 取不到这次改动新增了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到新增的判决文件时这一格什么都没比，不是通过。"
    return 1
  }
  local -a new_verdicts=()
  mapfile -t new_verdicts < <(new_verdict_paths)
  if ((${#new_verdicts[@]} == 0)); then
    echo "  ! 这次改动没有新写的判决 research/prompts/<轮>-main-verification.md，这一格无对象可判（基准 $gate_base）"
    return 77
  fi
  python3 - "$gate_base" "$TEST_CODE_SOURCES" "${CODE_ROUND_SNAPSHOT_BREAK:-}" "${new_verdicts[@]}" <<'PY'
import glob
import os
import re
import sys

base, test_code_pattern, breakage = sys.argv[1:4]
verdict_paths = [path for path in sys.argv[4:] if path]
suffix = "-main-verification.md"
# 判决正文里按路径点名的产品代码文件：crates/<包>/src/<…>.rs 的字面路径（ASCII 字符，遇到中文标点、反引号、冒号、井号就断）；crates/*/src/*.rs 这种通配写法认不出，不算点名
named_source = re.compile(r"crates/[A-Za-z0-9_.-]+/src/[A-Za-z0-9_./-]+\.rs(?![A-Za-z0-9_])")
test_code = re.compile(test_code_pattern)
hash_line = re.compile(r"([0-9a-f]{64})  (\S.*)")

judged = []          # (判决路径, 点名的产品代码文件)
not_judged = []      # (判决路径, 为什么不判)
for verdict_path in verdict_paths:
    if not os.path.isfile(verdict_path):
        continue
    with open(verdict_path, encoding="utf-8", errors="replace") as handle:
        text = handle.read()
    named = sorted(set(named_source.findall(text)))
    # 树里没有那个包目录的路径（定义轮里举例用的 crates/demo/src/… 这类）不是真文件，不算点名
    real = [path for path in named if os.path.isdir(path.split("/src/", 1)[0])]
    product = [path for path in real if not test_code.search(path)]
    if product:
        judged.append((verdict_path, product, text))
    elif real:
        not_judged.append((verdict_path, f"只点名了角色不是产品的包的代码 {len(real)} 份"))
    elif named:
        not_judged.append((verdict_path, f"点名的 {len(named)} 处路径所在的包不在树里（举例用的路径）"))
    else:
        not_judged.append((verdict_path, "没按路径点名 crates/*/src/*.rs"))

if not judged:
    print(f"  ! 新写的判决 {len(verdict_paths)} 份都没按路径点名产品代码的 crates/*/src/*.rs，这一格无对象可判（基准 {base}）")
    for verdict_path, why in not_judged:
        print(f"      {verdict_path}：{why}")
    sys.exit(77)

problems_by_verdict = []
covered_total = 0
absent_notes = []
for verdict_path, product, text in judged:
    round_name = os.path.basename(verdict_path)[: -len(suffix)]
    directory = f"research/prompts/{round_name}-snapshot"
    # 清单：快照目录下任一 *sha256.txt（按包记的 crates-sha256.txt，或冻结副本那一套的 frozen-copy-sha256.txt），点名的文件在哪一份里有行都算；kb-sha256.txt 记的是 kb 文件，不算 crates 清单
    manifests = sorted(path for path in glob.glob(f"{directory}/*sha256.txt") if os.path.basename(path) != "kb-sha256.txt") if os.path.isdir(directory) else []
    problems = []
    if directory + "/" not in text:
        problems.append(f"正文没写开工快照目录 {directory}/")
    if not os.path.isdir(directory):
        problems.append(f"目录 {directory}/ 不在树里（这次改动新加的、已跟踪的都算在树里）")
    elif not manifests:
        problems.append(f"目录里没有 {directory}/crates-sha256.txt（也没有别的 *sha256.txt 清单）")
    elif breakage != "snapshot-coverage-skipped":
        well_formed, malformed = set(), {}
        for manifest in manifests:
            with open(manifest, encoding="utf-8", errors="replace") as handle:
                for number, raw in enumerate(handle.read().split("\n"), 1):
                    line = raw.rstrip("\r")
                    if not line.strip():
                        continue
                    found = hash_line.fullmatch(line)
                    if found:
                        well_formed.add(found.group(2))
                        continue
                    pieces = line.split(None, 1)
                    if len(pieces) == 2:
                        malformed.setdefault(pieces[1].lstrip("*").strip(), (manifest, number))
        shown = "、".join(manifests)
        for path in product:
            if not os.path.exists(path):
                absent_notes.append(f"{verdict_path} 点名的 {path}")
                continue
            if path in well_formed:
                covered_total += 1
            elif path in malformed:
                bad_manifest, bad_number = malformed[path]
                problems.append(f"{bad_manifest} 第 {bad_number} 行点名了 {path}，但哈希不是 64 位十六进制、或哈希与路径之间不是两个空格")
            else:
                problems.append(f"{shown} 没有 {path} 的一行")
    if problems:
        problems_by_verdict.append((verdict_path, problems))

if problems_by_verdict:
    problem_count = sum(len(problems) for _path, problems in problems_by_verdict)
    print(f"  ✗ 新写的判决里 {len(problems_by_verdict)} 份点名了产品代码 crates 文件，开工快照没立住（共 {problem_count} 处）：")  # gate-lint:summary
    for verdict_path, problems in problems_by_verdict:
        print(f"      {verdict_path}：")  # gate-lint:detail
        for problem in problems:
            print(f"        {problem}")  # gate-lint:detail
    print("     → 怎么办：派腿之前 sha256sum <判决点名的 crates 文件…> > research/prompts/<轮>-snapshot/crates-sha256.txt（kb 文件另记 kb-sha256.txt），")
    print("               判决里写一行「开工快照」指到那个目录；腿已经派过的，现在补记并在判决里写明是事后补的、哈希对应哪一版")
    print("               （.claude/rules/implementation-workflow.md「代码轮派腿之前记一份开工快照」一节）。")
    sys.exit(1)

named_total = sum(len(product) for _path, product, _text in judged)
print(f"  ✓ 新写的判决 {len(verdict_paths)} 份：{len(judged)} 份点名了产品代码 crates 文件（共点名 {named_total} 处），"
      f"开工快照目录与清单（crates-sha256.txt 或 frozen-copy-sha256.txt）都在、点名的文件逐份有哈希行（{covered_total} 处）；{len(not_judged)} 份没点名产品代码文件不判（基准 {base}）")
if not_judged:
    print("    不判的：")
    for verdict_path, why in not_judged:
        print(f"      {verdict_path}：{why}")
if absent_notes:
    print(f"    点名的文件树里已经不存在、不要求快照有行的 {len(absent_notes)} 处：")
    for note in absent_notes:
        print(f"      {note}")
print("    哈希与派腿那一刻的文件内容对不对得上，这一格判不了（靠主 agent 写判决前 sha256sum -c 核）；"
      "与此刻的内容对不对得上由 crates-adversarial-review 格比（对不上判红：判决之后又改过）")
PY
}

# ── implementation-premise ──────────────────────────────────────────────────
check_implementation_premise() {
  local prompts_directory="research/prompts"
  local body heading date date_label round listed
  local -a bodies=("$prompts_directory"/_*-body.md)
  local checked=0 side_checked=0 failed=0
  local -a missing=() before_cutoff=() undated=() missing_snapshot=()
  for body in "${bodies[@]}"; do
    heading="$(head -n 1 "$body")"
    date="$(grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' <<<"$heading" | head -n 1)"
    # 标题里没有日期：证不出它早于 PREMISE_CUTOFF，照要查的算，不静默跳过
    if [[ -z "$date" ]]; then
      undated+=("$body")
      date_label="标题没有日期，按要查的算"
    elif [[ "$date" < "$PREMISE_CUTOFF" ]]; then
      before_cutoff+=("$body（$date）")
      continue
    else
      date_label="$date"
    fi
    checked=$((checked+1))
    grep -qF 'crates/' "$body" || missing+=("$body（$date_label）")
    if [[ -n "$date" && ! "$date" < "$SIDE_AND_SNAPSHOT_CUTOFF" ]]; then
      side_checked=$((side_checked + 1))
      round="$(basename "$body")"; round="${round#_}"; round="${round%-body.md}"
      if [[ -f "$prompts_directory/_$round-background.md" ]]; then
        if [[ ! -d "$prompts_directory/$round-snapshot" ]]; then
          missing_snapshot+=("$round：没有 research/prompts/$round-snapshot/")
        else
          while IFS= read -r listed; do
            [[ "$listed" == .claude/kb/* ]] || continue
            grep -qsF "$listed" "$prompts_directory/$round-snapshot"/* || missing_snapshot+=("$round：$listed 不在快照清单里")
          done < <(sed -n 's/^### 小节清单：`\([^`]*\)`.*/\1/p' "$prompts_directory/_$round-checklist.md" 2>/dev/null)
        fi
      fi
    fi
  done
  if ((checked == 0)); then
    echo "  ! 标题日期在 $PREMISE_CUTOFF 及以后的三方论证正文 0 份，这一格无对象可判"
    return 77
  fi
  if ((${#missing[@]})); then
    echo "  ✗ 这些三方论证正文没有「实现今天的样子」——全文一处 crates/ 都没有："   # gate-lint:summary
    printf '     %s\n' "${missing[@]}"
    echo "     → 怎么办：读 crates/ 里与这一问相关的代码路径，在正文前提表里加一行「实现今天的样子」，写文件名与看到的事实并标成观测；"
    echo "               实现里没有相关代码时，写 grep 命令与零命中（.claude/rules/implementation-first.md 第 2、3 条）。"
    failed=1
  fi
  if ((${#missing_snapshot[@]})); then
    echo "  ✗ 这些轮次的开工快照缺了（有背景材料就要有 <轮>-snapshot/，清单点名的 kb 文件都要在快照清单里）："   # gate-lint:summary
    printf '     %s\n' "${missing_snapshot[@]}"   # gate-lint:detail
    echo "     → 怎么办：照 .claude/agents/three-way-materials.md 第 5b 步补 research/prompts/<轮>-snapshot/kb-sha256.txt（sha256sum 原样输出，路径从仓根起）；腿已经开跑的，快照里写明是补写的与补写的日期"
    failed=1
  fi
  ((failed)) && return 1
  echo "  ✓ 三方论证正文都写了实现今天的样子（检查了 $checked 份：标题日期 ≥ $PREMISE_CUTOFF 的 $((checked - ${#undated[@]})) 份、标题没有日期的 ${#undated[@]} 份）"
  if ((${#undated[@]})); then
    echo "    标题没有日期、照要查的算的 ${#undated[@]} 份：${undated[*]}"
  fi
  echo "    标题日期 ≥ $SIDE_AND_SNAPSHOT_CUTOFF、另核了开工快照的 $side_checked 份"
  if ((${#before_cutoff[@]})); then
    echo "    没查 ${#before_cutoff[@]} 份（标题日期早于 $PREMISE_CUTOFF，规范落地之前写的）：${before_cutoff[*]}"
  else
    echo "    没查 0 份"
  fi
}
