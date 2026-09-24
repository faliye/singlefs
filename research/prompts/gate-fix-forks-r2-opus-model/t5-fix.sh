#!/usr/bin/env bash
# T5 改法（本腿提的，只在这份模型上量过、被攻过零轮）：gate_added_lines 只在文件头里认「+++ 」、剥 git 给带空格路径补的尾部制表符、
# 路径被 git 加了引号或含制表符就退非 0（调用方退回全量判）、写死 --no-renames、未跟踪的二进制文件不读。
# 补丁打在草稿目录里的副本上，40 / 56 / 61 / 88 四道阶段拷过去，再用 t5-added-lines.sh 的同一批现场跑一遍，另跑补过的共用脚本的 --selftest。
# 用法：bash t5-fix.sh <真仓根> <草稿目录> <模型目录>
set -uo pipefail
REPO="$(cd "$1" && pwd)"; D="$2"; MODEL="$(cd "$3" && pwd)"; R="$D/patched"; rm -rf "$R"
mkdir -p "$R/.claude/gate.d" "$R/research/scripts"
for s in 40-results-cited 56-crates-adversarial-review 61-settled-same-file 88-quoted-result-lines; do cp "$REPO/.claude/gate.d/$s.sh" "$R/.claude/gate.d/"; done
python3 - "$REPO/research/scripts/changed-paths.sh" "$R/research/scripts/changed-paths.sh" <<'PY'
import sys
src, dst = sys.argv[1:3]
text = open(src, encoding='utf-8').read()
old_start = text.index('gate_added_lines() {')
old_end = text.index('\n}\n', old_start) + 3
new = r'''gate_added_lines() {
  local base="$1"; shift
  local quote_option=(-c core.quotepath=false) diff_output untracked_list untracked_path parsed
  [[ "${LIB_CHANGED_PATHS_BREAK:-}" == quotepath ]] && quote_option=()
  diff_output="$(git "${quote_option[@]}" diff --no-renames --unified=0 --no-color --no-ext-diff --src-prefix=a/ --dst-prefix=b/ "$base" -- "$@")" || return 1
  untracked_list="$(git "${quote_option[@]}" ls-files -z --others --exclude-standard -- "$@" | tr '\0' '\n')" || return 1
  # 「+++ 」只在文件头里认（diff --git 与第一个 @@ 之间）；正文里以「++ 」开头的一行新增在 diff 里也长成「+++ 」。
  # git 给带空格的路径在文件头末尾补一个制表符；路径被 git 加了引号（含制表符、引号、反斜杠），这里还原不了，退非 0。
  parsed="$(awk '
    /^diff --git / { header = 1; path = ""; next }
    header && /^\+\+\+ / { path = substr($0, 5); sub(/\t$/, "", path)
                           if (substr(path, 1, 1) == "\"") { quoted = 1; path = "" }
                           else if (path == "/dev/null") path = ""; else sub(/^b\//, "", path); next }
    /^@@ / { header = 0; next }
    !header && /^\+/ && path != "" { print path "\t" substr($0, 2) }
    END { if (quoted) exit 3 }' <<<"$diff_output")" || return 1
  [[ -z "$parsed" ]] || printf '%s\n' "$parsed"
  [[ "${LIB_CHANGED_PATHS_BREAK:-}" == untracked-lines ]] && return 0
  while IFS= read -r untracked_path; do
    [[ -n "$untracked_path" && -f "$untracked_path" ]] || continue
    [[ "$untracked_path" != *$'\t'* ]] || return 1
    grep -Iq . "$untracked_path" || continue   # 二进制（或空）文件不读：一个 NUL 会让下游 grep 把整段新增行当二进制吞掉
    UNTRACKED_PATH="$untracked_path" awk '{ print ENVIRON["UNTRACKED_PATH"] "\t" $0 }' "$untracked_path"
  done <<<"$untracked_list"
}
'''
open(dst, 'w', encoding='utf-8').write(text[:old_start] + new + text[old_end:])
PY
echo "== 补过的共用脚本自证：$(bash "$R/research/scripts/changed-paths.sh" --selftest | head -1)"
for b in quotepath untracked-lines; do echo "   LIB_CHANGED_PATHS_BREAK=$b：退 $(LIB_CHANGED_PATHS_BREAK=$b bash "$R/research/scripts/changed-paths.sh" --selftest >/dev/null 2>&1; echo $?)"; done
STAGES="$R" bash "$MODEL/t5-added-lines.sh" "$REPO" "$D/t5-fix-scenes"
