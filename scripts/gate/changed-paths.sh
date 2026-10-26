#!/usr/bin/env bash
# 门禁阶段共用的「这次改动碰了哪些路径」取法（中文路径不转义、未跟踪可开关、基准的回退次序）。
#
# 这是门禁阶段共用的「这次改动碰了哪些路径、新增了哪些行」取法，阶段 source 它，不各抄一份（门禁 code-tooling 的 change-range-single-source 格判阶段里没有第二份）：
# 六份拷贝已经分叉过——一半的 git 调用漏了 -c core.quotepath=false（中文路径被转成带引号的八进制串，
# 与仓里的路径逐字对不上），今天门禁 doc-registries 的 invariant-anchors 那一格（当时单独是一道）借「与 doc-process-records 同一条」时又省掉了未跟踪文件。
#
# 函数：
#   gate_diff_base <取法>
#     gate：GATE_BASE 是个提交就用它；否则 @{upstream} 的 merge-base；都没有（或 merge-base 取不到）就 HEAD。
#           一轮可以跨几个提交，问「这一轮碰了哪些」的阶段用它（doc-process-records 的 verdict-names-local-samples、crates-adversarial-review、code-round-snapshot、knowledge-sync、agent-def-adversarial-review 五格，doc-experiments 用改动范围的六格，doc-registries 的 invariant-anchors 那一格）。
#           不认上游 gate.sh 导出的 GATE_DIFF_BASE（用户 2026-09-24 定）：带 --staged 那条路在里层把 GATE_BASE 设成上游
#           diff_base，不带 --staged 的整轮用这里的回退次序；全推出去了时前者是 HEAD~1、后者是 HEAD，两条路判的窗口不同（C528）。
#           搬迁登记的生效窗口跟着这个基准走（.claude/gate.d/lib-lag-baseline.py 文件头「搬迁登记」）：有上游而没推、--staged 推完之后基准取 HEAD~1 时，搬迁已提交的登记行仍生效。
#     head：HEAD。问「这一次提交要带哪些」的阶段用它（doc-process-records 的 batch-scope 格；为什么不用 GATE_BASE 写在 doc-process-records 文件头 batch-scope 那一段）。
#   gate_changed_paths <基准> <untracked | tracked-only> [<diff-filter>]
#     一行一条、排序去重：基准到工作区的 diff、HEAD 到暂存区的 diff；第二个参数是 untracked 时再加未跟踪文件。
#     给了 diff-filter（例 A）只在两个 diff 上加；未跟踪文件只在 diff-filter 为空或含 A 时加（它们本来就是新增）。
#     git 调用一律带 -c core.quotepath=false。路径里带制表符、换行或引号的，git 照样转义，这一条管不到。
#     与基准相比内容只差此刻生效的登记搬迁（.claude/gate.d/path-moves.tsv，被判的根里那一份；一行只在不在 gate 基准那一版的登记表里、gate 基准那一版里旧前缀下有文件、
#     新前缀下没有文件时生效；只差仓根起的前缀，别的目录下同名的一段不算）的路径剔掉，不算这一次碰了的：经 gate_drop_moved_only。
#   gate_drop_moved_only <基准>
#     标准输入一行一条路径（相对仓根），交回其中不只差登记搬迁的那几条（次序不变）。判法在 .claude/gate.d/lib-lag-baseline.py 的 moved_only_paths
#     （文件头「搬迁登记」：原地改的按原路径到基准里取，改了路径的只在新路径恰好等于旧路径照一行生效的登记换前缀（单个文件登记的文件名不变）时按旧路径取，
#     整份搬进别的按路径管的类的不剔；换算之后与此刻逐字节相同、模式相同就剔；暂存区那一份也要只差搬迁；
#     「<提交哈希>:」后面的路径不换）。登记表没有内容行时不起 python、原样交回，与没有这一层时一字不差；git 取不到退非 0、不交半截。
#     自己拿 git diff 比内容的格（门禁 doc-decisions 的 entry-added）把它列出的路径过一遍这个函数，与改动范围剔得一样。
#   gate_added_lines <基准> <路径>…
#     这次改动新增的行，一行一条「路径<TAB>行文」：跟踪的文件取基准到工作区的 diff 里以 + 开头的行，
#     **未跟踪的文件整份都算新增**（`git diff` 看不到它们，新写的 kb 页在 git add 之前整页会被当成「早先写下的」跳过）。
#     不认改名（搬家之后整份算新增，暂存前后判得一样）；未跟踪的二进制文件不读。
#     登记了搬迁的，剔掉只因此刻生效的登记搬迁才变了的行（lib-lag-baseline.py 的 drop_moved_lines：与基准那一版按 diff 的位置配对，同一块里被删的第 k 行
#     照登记换算之后等于新增的第 k 行才剔，删加条数不等的块不剔；整份恰好换前缀搬了家、没搬进别的按路径管的类的，另剔 diff 里没动的行；别的一行不剔）。
#     路径可以是目录。git 失败、或路径被 git 加了引号（带制表符、引号、反斜杠）时退出码非 0、不输出半截结果，
#     调用方按「拿不到改动范围」处理（通常退回全量判）。
#
# 阶段里的用法：
#   LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/scripts/gate/changed-paths.sh"
#   source "$LIB_CHANGED_PATHS" || { echo "  ✗ …"; echo "     → …"; exit 1; }
#   base="$(gate_diff_base gate)"
#   changed="$(gate_changed_paths "$base" untracked)"
#
# 住在 scripts/ 而不是 .claude/gate.d/：共享门禁（gate.sh）把 gate.d 下每个 *.sh 都当阶段跑，
# 阶段调用的共用 shell 脚本放这里；自证由门禁 code-tooling 跑。
# `--selftest` 在临时 git 仓里自证：三种来源（工作区改、暂存新增、未跟踪）各放一个中文路径，逐字核两个函数的输出。
# 弄坏开关 LIB_CHANGED_PATHS_BREAK=<项>（只给证红用，一次给一个；打开时自证退 1）。只在 --selftest 那一支里认：门禁阶段 source 这一份时，
# 环境里留着这个变量也不认（source 时把内部的 _lib_changed_paths_break 清空，只有自证那一支照 LIB_CHANGED_PATHS_BREAK 设它；C839）。
# 逐项写开关名、弄坏哪一处、打开后自证哪几项红：一行一项，「」里是那一格失败行的开头「✗ <标签>：」（标签是自证里 check 的标签）；失败行逐条列在汇总句
# 「✗ 共用库 scripts/gate/changed-paths.sh 的自证没过」下面，整行是「✗ <标签>：期望「…」，实测「…」」，括号里抄那一行的实测（制表符写成 <制表符>）：
#   LIB_CHANGED_PATHS_BREAK=quotepath  gate_changed_paths 与 gate_added_lines 的 git 调用漏掉 -c core.quotepath=false
#       「✗ 未跟踪也算、中文路径不转义：」（实测『"\345\267\262\350\267\237\350\270\252.md" | …』：中文路径成了带引号的八进制串）
#       「✗ tracked-only 不带未跟踪：」（实测『"\345\267\262\350\267\237\350\270\252.md" | …』）
#       「✗ diff-filter=A 只要新增、未跟踪算新增：」（实测『"\346\232\202\345\255\230\346\226\260\345\242\236.md" | …』）
#       「✗ 新增行：改过的行、暂存新增的整份、未跟踪的整份，中文路径不转义：」（实测『』）
#       「✗ 新增行只看给的路径、删掉的旧行不算：」（实测『』）
#       「✗ 新增行：路径带空格、正文里以「++ 」开头的一行照样归到那份文件：」（实测『』）
#       「✗ HEAD 还没出生：基准是空树、未跟踪文件照常交出：」（实测『<空树> "\346\226\260.md"』）
#       「✗ 搬迁：登记表没有内容行时只差搬迁的文件照旧算改动：」（实测『0』）
#       「✗ 搬迁：只差登记搬迁的文件不算改动：」（实测『"\346\226\207\346\241\243.md" | …』：带引号的路径认不出、一条都没剔）
#       「✗ 搬迁：新增行剔掉只差搬迁的行：」（实测『』）
#       「✗ 搬迁：换算只认仓根起的前缀，别的目录下同名的一段改指另一份实现照算改动：」
#       「✗ 搬迁：搬迁已提交、有上游没推，基准取 merge-base，这一行仍生效：」
#       「✗ 搬迁：推上去之后基准取 HEAD，登记行不再作用（留旧路径的记录改回新路径照算改动）：」
#   LIB_CHANGED_PATHS_BREAK=untracked  gate_changed_paths 带 untracked 时也不加未跟踪文件
#       「✗ 未跟踪也算、中文路径不转义：」（实测『已跟踪.md | 暂存新增.md | 第二个提交.md』）
#       「✗ diff-filter=A 只要新增、未跟踪算新增：」（实测『暂存新增.md』）
#       「✗ HEAD 还没出生：基准是空树、未跟踪文件照常交出：」（实测『<空树> 』）
#       「✗ 搬迁：只差登记搬迁的文件不算改动：」（实测『混合.md』：未跟踪的登记表本身没交出）
#       「✗ 搬迁：换算只认仓根起的前缀，别的目录下同名的一段改指另一份实现照算改动：」
#   LIB_CHANGED_PATHS_BREAK=fallback  gate_diff_base gate 不认 GATE_BASE，直接取上游的 merge-base
#       「✗ GATE_BASE 是个提交就用它：」（实测是上游那个提交的哈希）
#   LIB_CHANGED_PATHS_BREAK=empty-merge-base  merge-base 取不到时交出空串，不回退到 HEAD
#       「✗ 上游与 HEAD 没有共同祖先就回退到 HEAD：」（实测『』）
#   LIB_CHANGED_PATHS_BREAK=untracked-lines  gate_added_lines 不读未跟踪文件
#       「✗ 新增行：改过的行、暂存新增的整份、未跟踪的整份，中文路径不转义：」（实测『已跟踪.md<制表符>第二版 | 暂存新增.md<制表符>新』）
#       「✗ 新增行：路径带制表符时退非 0：」（实测『0』：带制表符的那份未跟踪文件根本没读）
#   LIB_CHANGED_PATHS_BREAK=swallow  gate_changed_paths 第一条 git diff 失败时吞掉退出码
#       「✗ 基准解析不到：两个取法都退非 0，不交半截结果：」（实测『0 1 空』）
#   LIB_CHANGED_PATHS_BREAK=header-anywhere  「+++ 」在正文里也当文件头认
#       「✗ 新增行：路径带空格、正文里以「++ 」开头的一行照样归到那份文件：」（实测『以两个加号开头的新增行<制表符>紧跟的一行』）
#   LIB_CHANGED_PATHS_BREAK=binary  未跟踪的二进制文件也整份读
#       「✗ 新增行：未跟踪的二进制文件不读：」（实测『1』）
#   LIB_CHANGED_PATHS_BREAK=moved-kept  gate_drop_moved_only 与 gate_added_lines 不剔只差登记搬迁的，原样交回
#       「✗ 搬迁：只差登记搬迁的文件不算改动：」（实测『.claude/gate.d/path-moves.tsv | 文档.md | 新目录/脚本.sh | 旧目录/脚本.sh | 混合.md』）
#       「✗ 搬迁：新增行剔掉只差搬迁的行：」（实测『文档.md<制表符>见 新目录/脚本.sh | 新目录/脚本.sh<制表符>见 新目录/脚本.sh | …』）
#       「✗ 搬迁：换算只认仓根起的前缀，别的目录下同名的一段改指另一份实现照算改动：」
#       「✗ 搬迁：搬迁已提交、有上游没推，基准取 merge-base，这一行仍生效：」
#   共用库那一侧的同一处由 LAG_BASELINE_BREAK=moved-only-kept 弄坏（写法在 .claude/gate.d/lib-lag-baseline.py 文件头），这两项同样红；
#   moved-kept 下搬迁那几项另有两项红：「✗ 搬迁：换算只认仓根起的前缀…」「✗ 搬迁：搬迁已提交、有上游没推…」；
#   LAG_BASELINE_BREAK=slash-prefix-rewritten  共用库退回前缀前面接 / 也换
#       「✗ 搬迁：换算只认仓根起的前缀，别的目录下同名的一段改指另一份实现照算改动：」（实测『.claude/gate.d/path-moves.tsv』：调用.sh 被剔）
#       「✗ 搬迁：搬迁已提交、有上游没推，基准取 merge-base，这一行仍生效：」（实测『.claude/gate.d/path-moves.tsv』）
#   LAG_BASELINE_BREAK=window-ignored  共用库把每一行都当此刻生效（去掉生效窗口）
#       「✗ 搬迁：推上去之后基准取 HEAD，登记行不再作用（留旧路径的记录改回新路径照算改动）：」（实测『』：记录.md 被剔）
#   门禁阶段的样本只认那一个（这一份的开关 source 时不认，C839）。
#
#   bash scripts/gate/changed-paths.sh --selftest      # 自证
#   LIB_CHANGED_PATHS_BREAK=quotepath bash scripts/gate/changed-paths.sh --selftest   # 必须判红

# 弄坏开关的内部读法：source 时清空，平常跑的门禁不认环境里的 LIB_CHANGED_PATHS_BREAK（C839）；只有下面 --selftest 那一支照它设
_lib_changed_paths_break=""
# 只差搬迁那一层的判法在同一个仓的 .claude/gate.d/lib-lag-baseline.py（从这一份的位置往上两层找；搬迁登记表读被判的根里那一份）
_lib_changed_paths_lag_baseline="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." 2>/dev/null && pwd)/.claude/gate.d/lib-lag-baseline.py"

# 被判的根（当前目录）里的搬迁登记表有没有内容行：没有时改动范围与新增行原样交出，不起 python
_lib_changed_paths_moves_registered() {
  local table=.claude/gate.d/path-moves.tsv
  [[ -f "$table" ]] && grep -qv -e '^[[:space:]]*#' -e '^[[:space:]]*$' "$table"
}

gate_drop_moved_only() {
  local base="$1" listed
  listed="$(cat)"
  [[ -n "$listed" ]] || return 0
  if [[ "${_lib_changed_paths_break:-}" == moved-kept ]] || ! _lib_changed_paths_moves_registered; then
    printf '%s\n' "$listed"
    return 0
  fi
  python3 "$_lib_changed_paths_lag_baseline" moved-only-filter "$base" <<<"$listed"
}

gate_diff_base() {
  local mode="$1" merge_base
  # HEAD 还没出生（刚 git init、orphan 分支）：基准取空树，与上游 lib.sh 对空仓的处置同一种；交出 HEAD 会让 git diff 失败、调用方报取不到
  if ! git rev-parse --verify -q HEAD >/dev/null 2>&1; then git hash-object -t tree /dev/null; return 0; fi
  case "$mode" in
    head)
      echo "HEAD"
      ;;
    gate)
      if [[ "${_lib_changed_paths_break:-}" != fallback && -n "${GATE_BASE:-}" ]] \
         && git rev-parse --verify -q "${GATE_BASE}^{commit}" >/dev/null 2>&1; then
        echo "$GATE_BASE"
      elif git rev-parse --verify -q '@{upstream}' >/dev/null 2>&1 \
           && { merge_base="$(git merge-base HEAD '@{upstream}' 2>/dev/null)" || [[ "${_lib_changed_paths_break:-}" == empty-merge-base ]]; }; then
        echo "$merge_base"
      else
        echo "HEAD"
      fi
      ;;
    *)
      echo "gate_diff_base：取法只认 gate 或 head，收到「$mode」" >&2
      return 2
      ;;
  esac
}

gate_changed_paths() {
  local base="$1" untracked_mode="$2" diff_filter="${3:-}"
  local quote_option=(-c core.quotepath=false) filter_option=()
  [[ "${_lib_changed_paths_break:-}" == quotepath ]] && quote_option=()
  [[ -n "$diff_filter" ]] && filter_option=("--diff-filter=$diff_filter")
  case "$untracked_mode" in
    untracked|tracked-only) ;;
    *)
      echo "gate_changed_paths：第二个参数只认 untracked 或 tracked-only，收到「$untracked_mode」" >&2
      return 2
      ;;
  esac
  local include_untracked=0
  if [[ "$untracked_mode" == untracked && "${_lib_changed_paths_break:-}" != untracked ]] \
     && [[ -z "$diff_filter" || "$diff_filter" == *A* ]]; then
    include_untracked=1
  fi
  # 三次调用逐条判退出码、先落到变量再排序：写成 `{ …; …; …; } | sort -u` 时花括号组只交最后一条的退出码，
  # 基准写错、第一条 git diff 失败，集合就静默少了一截。
  local listed
  listed="$(
    git "${quote_option[@]}" diff --name-only "${filter_option[@]}" "$base" -- || [[ "${_lib_changed_paths_break:-}" == swallow ]] || exit 1
    git "${quote_option[@]}" diff --name-only --cached "${filter_option[@]}" -- || exit 1
    if ((include_untracked)); then git "${quote_option[@]}" ls-files --others --exclude-standard -- || exit 1; fi
  )" || return 1
  [[ -n "$listed" ]] || return 0
  listed="$(sort -u <<<"$listed")"
  gate_drop_moved_only "$base" <<<"$listed"
}

gate_added_lines() {
  local base="$1" statuses
  if [[ "${_lib_changed_paths_break:-}" == moved-kept ]] || ! _lib_changed_paths_moves_registered; then
    _lib_changed_paths_added_lines_raw "$@"
    return
  fi
  # 经管道剔只因搬迁才变了的行（新增行可以上百兆，不先落进变量）；两段的退出码都判，任一段非 0 就退 1，调用方照「取不到」处理
  _lib_changed_paths_added_lines_raw "$@" | python3 "$_lib_changed_paths_lag_baseline" moved-lines-filter "$base"
  statuses=("${PIPESTATUS[@]}")
  ((statuses[0] == 0 && statuses[1] == 0)) || return 1
}

_lib_changed_paths_added_lines_raw() {
  local base="$1"; shift
  local quote_option=(-c core.quotepath=false) diff_output untracked_list untracked_path parsed
  [[ "${_lib_changed_paths_break:-}" == quotepath ]] && quote_option=()
  # --no-renames 写死：同一次搬家，暂存前（未跟踪的新名）与暂存后（一次改名）判得要一样；
  # 前缀写死：diff.noprefix 之类的配置会让 +++ 行不带 b/，路径就解析错了。
  diff_output="$(git "${quote_option[@]}" diff --no-renames --unified=0 --no-color --no-ext-diff --src-prefix=a/ --dst-prefix=b/ "$base" -- "$@")" || return 1
  untracked_list="$(git "${quote_option[@]}" ls-files -z --others --exclude-standard -- "$@" | tr '\0' '\n')" || return 1
  # 「+++ 」只在文件头里认（diff --git 与第一个 @@ 之间）：正文里以「++ 」开头的一行新增，在 diff 里也长成「+++ 」。
  # git 给带空格的路径在文件头末尾补一个制表符，剥掉；路径被 git 加了引号（带制表符、引号、反斜杠），这里还原不了，退非 0。
  parsed="$(awk -v header_anywhere="$([[ "${_lib_changed_paths_break:-}" == header-anywhere ]] && echo 1)" '
    /^diff --git / { header = 1; path = ""; next }
    (header || header_anywhere) && /^\+\+\+ / {
      path = substr($0, 5); sub(/\t$/, "", path)
      if (substr(path, 1, 1) == "\"") { quoted = 1; path = "" }
      else if (path == "/dev/null") path = ""; else sub(/^b\//, "", path)
      next }
    /^@@ / { header = 0; next }
    !header && /^\+/ && path != "" { print path "\t" substr($0, 2) }
    END { if (quoted) exit 3 }' <<<"$diff_output")" || return 1
  [[ -z "$parsed" ]] || printf '%s\n' "$parsed"
  [[ "${_lib_changed_paths_break:-}" == untracked-lines ]] && return 0
  while IFS= read -r untracked_path; do
    [[ -n "$untracked_path" && -f "$untracked_path" ]] || continue
    [[ "$untracked_path" != *$'\t'* ]] || return 1
    # 二进制（或空）文件不读：一个 NUL 会让下游的 grep 把整段新增行当二进制吞掉（vim 的交换文件就是这种）
    if [[ "${_lib_changed_paths_break:-}" != binary ]]; then grep -Iq . "$untracked_path" || continue; fi
    UNTRACKED_PATH="$untracked_path" awk '{ print ENVIRON["UNTRACKED_PATH"] "\t" $0 }' "$untracked_path"
  done <<<"$untracked_list"
}

# ── 直接跑（不是 source）时：只认 --selftest，在临时仓里自证 ─────────────────
if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  set -uo pipefail
  [[ "${1:-}" == "--selftest" ]] || { echo "  ✗ 用法：bash scripts/gate/changed-paths.sh --selftest（平时由门禁阶段 source 它）"; echo "     → 怎么办：自证就带 --selftest；要取改动路径，在阶段里 source 它再调 gate_diff_base / gate_changed_paths。"; exit 2; }
  _lib_changed_paths_break="${LIB_CHANGED_PATHS_BREAK:-}"
  library_path="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
  selftest_directory="$(mktemp -d)"
  trap 'rm -rf "${selftest_directory:?}"' EXIT
  selftest_output="$(
    cd "$selftest_directory" || exit 2
    # 与使用者的 git 配置隔开：全局配了 core.quotepath=false 的机器上，漏掉那个选项的取法也照样对，自证就分不出来。
    export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
    export GIT_AUTHOR_NAME=selftest GIT_AUTHOR_EMAIL=selftest@example.invalid
    export GIT_COMMITTER_NAME=selftest GIT_COMMITTER_EMAIL=selftest@example.invalid
    unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GATE_BASE
    git init -q -b master . || exit 2
    printf '第一版\n' > 已跟踪.md
    git add -A && git commit -qm 基准 || exit 2
    first_commit="$(git rev-parse HEAD)"
    printf 'x\n' > 第二个提交.md
    git add -A && git commit -qm 第二个提交 || exit 2
    second_commit="$(git rev-parse HEAD)"
    # 上游指第二个提交：merge-base 与 GATE_BASE 取的第一个提交不同，回退那一档错了才分得出来。
    git branch 上游 "$second_commit" && git branch -q --set-upstream-to=上游 master || exit 2
    printf '第二版\n' > 已跟踪.md
    printf '新\n' > 暂存新增.md && git add 暂存新增.md
    printf '未跟踪\n' > 未跟踪.md
    check() {
      local label="$1" expected="$2" actual="$3"
      if [[ "$actual" == "$expected" ]]; then
        echo "PASS $label"
      else
        echo "FAIL $label：期望「${expected//$'\n'/ | }」，实测「${actual//$'\n'/ | }」"
      fi
    }
    check "未跟踪也算、中文路径不转义" "$(printf '%s\n' 已跟踪.md 暂存新增.md 未跟踪.md 第二个提交.md | sort -u)" \
          "$(gate_changed_paths "$first_commit" untracked)"
    check "tracked-only 不带未跟踪" "$(printf '%s\n' 已跟踪.md 暂存新增.md | sort -u)" \
          "$(gate_changed_paths HEAD tracked-only)"
    check "diff-filter=A 只要新增、未跟踪算新增" "$(printf '%s\n' 暂存新增.md 未跟踪.md | sort -u)" \
          "$(gate_changed_paths HEAD untracked A)"
    check "新增行：改过的行、暂存新增的整份、未跟踪的整份，中文路径不转义" \
          "$(printf '%s\t%s\n' 已跟踪.md 第二版 暂存新增.md 新 未跟踪.md 未跟踪 | sort)" "$(gate_added_lines HEAD . | sort)"
    check "新增行只看给的路径、删掉的旧行不算" "$(printf '%s\t%s\n' 已跟踪.md 第二版 第二个提交.md x | sort)" \
          "$(gate_added_lines "$first_commit" 第二个提交.md 已跟踪.md | sort)"
    mkdir -p 带空格目录 && printf '旧\n' > '带空格目录/a b.md' && git add -A && git commit -qm 带空格的路径 || exit 2
    printf '旧\n++ 以两个加号开头的新增行\n紧跟的一行\n' > '带空格目录/a b.md'
    check "新增行：路径带空格、正文里以「++ 」开头的一行照样归到那份文件" \
          "$(printf '%s\t%s\n' '带空格目录/a b.md' '++ 以两个加号开头的新增行' '带空格目录/a b.md' '紧跟的一行')" \
          "$(gate_added_lines HEAD 带空格目录)"
    printf 'x\0y\n' > 带空格目录/交换文件.swp
    check "新增行：未跟踪的二进制文件不读" "0" "$(gate_added_lines HEAD 带空格目录 | grep -c '交换文件' )"
    printf 'z\n' > "$(printf '带空格目录/制表\t符.md')"
    check "新增行：路径带制表符时退非 0" "1" "$(gate_added_lines HEAD 带空格目录 >/dev/null 2>&1; echo "$(( $? != 0 ))")"
    rm -rf 带空格目录
    check "基准解析不到：两个取法都退非 0，不交半截结果" "1 1 空" \
          "$(gate_changed_paths 没有这个提交 untracked >/dev/null 2>&1; a=$?; gate_added_lines 没有这个提交 . >/dev/null 2>&1; b=$?; \
             out="$(gate_changed_paths 没有这个提交 untracked 2>/dev/null)"; echo "$((a != 0)) $((b != 0)) ${out:-空}")"
    # HEAD 还没出生的仓（刚 git init）：基准取空树，取路径照常交出未跟踪文件，不报取不到
    unborn_result="$(mkdir 未出生 && cd 未出生 && git init -q -b master . && printf 'x\n' > 新.md \
      && base="$(gate_diff_base gate)" && paths="$(gate_changed_paths "$base" untracked)" && echo "$base $paths")"
    check "HEAD 还没出生：基准是空树、未跟踪文件照常交出" "$(git hash-object -t tree /dev/null) 新.md" "$unborn_result"
    rm -rf 未出生
    # 搬迁登记：与基准只差登记搬迁的文件与行不算改动（判法在 .claude/gate.d/lib-lag-baseline.py 的 moved_only_paths / drop_moved_lines）。
    # 旧目录 git mv 到新目录、改了里面的路径；文档.md 原地换了路径、留着一行提交引用；混合.md 换了路径又改了一行。
    (
      mkdir 搬迁仓 && cd 搬迁仓 && git init -q -b master . && mkdir -p 旧目录 records .claude/gate.d || exit 2
      printf '见 旧目录/脚本.sh\n' > 旧目录/脚本.sh
      printf '见 旧目录/脚本.sh\n改前 31e47115:旧目录/脚本.sh\n' > 文档.md
      printf '见 旧目录/脚本.sh\n第一版\n' > 混合.md
      printf '# 搬迁\n' > records/2026-01-01-搬迁.md
      git add -A && git commit -qm 基准 && git mv 旧目录 新目录 || exit 2
      printf '见 新目录/脚本.sh\n' > 新目录/脚本.sh
      printf '见 新目录/脚本.sh\n改前 31e47115:旧目录/脚本.sh\n' > 文档.md
      printf '见 新目录/脚本.sh\n第二版\n' > 混合.md
      printf '# 只有注释\n' > .claude/gate.d/path-moves.tsv
      check "搬迁：登记表没有内容行时只差搬迁的文件照旧算改动" "2" "$(gate_changed_paths HEAD untracked | grep -cxF -e 新目录/脚本.sh -e 文档.md)"
      printf '旧目录\t新目录\t2026-01-01\trecords/2026-01-01-搬迁.md\n' > .claude/gate.d/path-moves.tsv
      check "搬迁：只差登记搬迁的文件不算改动" "$(printf '%s\n' .claude/gate.d/path-moves.tsv 混合.md | sort -u)" "$(gate_changed_paths HEAD untracked)"
      check "搬迁：新增行剔掉只差搬迁的行" "$(printf '%s\t%s\n' 混合.md 第二版)" "$(gate_added_lines HEAD 混合.md 新目录 文档.md)"
    )
    rm -rf 搬迁仓
    # 搬迁登记只认仓根起的前缀，生效窗口跟着 gate 基准走：别处/旧目录 是别的目录下同名的一段，不是仓根起的 旧目录/，调用方改指别处/新目录（另一份实现）照算改动；
    # 有上游、搬迁已提交没推时基准取上游的 merge-base，那一行仍生效；推上去之后基准取 HEAD，这一行不再作用（留旧路径的记录改回新路径照算改动）。
    (
      mkdir 搬迁上游仓 && cd 搬迁上游仓 && git init -q -b master . && mkdir -p 旧目录 别处/旧目录 别处/新目录 records .claude/gate.d || exit 2
      printf '见 旧目录/脚本.sh\n' > 旧目录/脚本.sh
      printf '别处的老实现\n' > 别处/旧目录/脚本.sh
      printf '别处的另一份实现\n' > 别处/新目录/脚本.sh
      printf 'bash 别处/旧目录/脚本.sh\n' > 调用.sh
      printf '见 旧目录/脚本.sh\n' > 文档.md
      printf 'git log --format=%%s -- 旧目录/脚本.sh\n' > 记录.md
      printf '# 搬迁\n' > records/2026-01-01-搬迁.md
      git add -A && git commit -qm 基准 && git branch 上游 && git branch -q --set-upstream-to=上游 master && git mv 旧目录 新目录 || exit 2
      printf '见 新目录/脚本.sh\n' > 新目录/脚本.sh
      printf 'bash 别处/新目录/脚本.sh\n' > 调用.sh
      printf '见 新目录/脚本.sh\n' > 文档.md
      printf '旧目录\t新目录\t2026-01-01\trecords/2026-01-01-搬迁.md\n' > .claude/gate.d/path-moves.tsv
      check "搬迁：换算只认仓根起的前缀，别的目录下同名的一段改指另一份实现照算改动" "$(printf '%s\n' .claude/gate.d/path-moves.tsv 调用.sh | sort -u)" "$(gate_changed_paths "$(gate_diff_base gate)" untracked)"
      git add -A && git commit -qm 搬迁 || exit 2
      check "搬迁：搬迁已提交、有上游没推，基准取 merge-base，这一行仍生效" "$(printf '%s\n' .claude/gate.d/path-moves.tsv 调用.sh | sort -u)" "$(gate_changed_paths "$(gate_diff_base gate)" untracked)"
      git branch -qf 上游 HEAD && printf 'git log --format=%%s -- 新目录/脚本.sh\n' > 记录.md
      check "搬迁：推上去之后基准取 HEAD，登记行不再作用（留旧路径的记录改回新路径照算改动）" "记录.md" "$(gate_changed_paths "$(gate_diff_base gate)" untracked)"
    )
    rm -rf 搬迁上游仓
    check "head 取法是 HEAD" "HEAD" "$(gate_diff_base head)"
    check "GATE_BASE 是个提交就用它" "$first_commit" "$(GATE_BASE="$first_commit" gate_diff_base gate)"
    # C839：门禁阶段 source 这一份时环境里留着弄坏开关也不认（这一格在任何开关下都该绿）
    check "source 这一份时环境里的 LIB_CHANGED_PATHS_BREAK=fallback 不认" "$first_commit" \
          "$(LIB_CHANGED_PATHS_BREAK=fallback GATE_BASE="$first_commit" bash -c 'source "$1" && gate_diff_base gate' sourced "$library_path")"
    check "GATE_BASE 不是提交就回退到上游的 merge-base" "$second_commit" "$(GATE_BASE=没有这个提交 gate_diff_base gate)"
    # 上游是一条没有共同祖先的分支：merge-base 取不到，要回退到 HEAD，不许拿空串当基准（空基准让 git diff 报错、改动范围静默变空）。
    orphan_commit="$(git commit-tree "$(git mktree </dev/null)" -m 孤儿)" && git branch 孤儿 "$orphan_commit" \
      && git branch -q --set-upstream-to=孤儿 master || exit 2
    check "上游与 HEAD 没有共同祖先就回退到 HEAD" "HEAD" "$(gate_diff_base gate)"
    git branch -q --unset-upstream
    check "没有上游就回退到 HEAD" "HEAD" "$(gate_diff_base gate)"
  )"
  check_count="$(grep -c '^PASS \|^FAIL ' <<<"$selftest_output")"
  failures="$(grep '^FAIL ' <<<"$selftest_output")"
  if [[ -n "$failures" || "$check_count" -ne 22 ]]; then
    echo "  ✗ 共用库 scripts/gate/changed-paths.sh 的自证没过（查了 $check_count 项，应当 22 项）："   # gate-lint:summary
    printf '%s\n' "$failures" | sed 's/^FAIL /  ✗ /'   # gate-lint:detail
    printf '%s\n' "$selftest_output" | grep -v '^PASS \|^FAIL ' | sed 's/^/      /'   # gate-lint:detail
    echo "     → 怎么办：看 gate_changed_paths / gate_added_lines 的 git 调用有没有带 -c core.quotepath=false、未跟踪文件那一支有没有加 ls-files --others，"
    echo "               gate_diff_base 的回退次序是不是 GATE_BASE → @{upstream} 的 merge-base → HEAD；门禁 doc-process-records、doc-experiments 与 doc-registries 的 invariant-anchors 那一格都靠它取改动范围。"
    exit 1
  fi
  echo "  ✓ 共用库 scripts/gate/changed-paths.sh 自证通过（$check_count 项：三种来源的中文路径逐字不转义、未跟踪可开关、diff-filter=A、新增行含未跟踪整份且只看给的路径、带空格路径与「++ 」开头的行、不读二进制、路径带制表符退非 0、基准解析不到退非 0、HEAD 没出生取空树、基准回退四档、source 时不认环境里的弄坏开关、与基准只差登记搬迁的文件与行不算改动、登记表没有内容行时照旧算、搬迁登记只认仓根起的前缀、生效窗口跟着 gate 基准走）"
  exit 0
fi
