#!/usr/bin/env bash
# 门禁阶段共用的「这次改动碰了哪些路径」取法（中文路径不转义、未跟踪可开关、基准的回退次序）。
#
# 这是门禁阶段共用的「这次改动碰了哪些路径、新增了哪些行」取法，阶段 source 它，不各抄一份（门禁 64 号判阶段里没有第二份）：
# 六份拷贝已经分叉过——一半的 git 调用漏了 -c core.quotepath=false（中文路径被转成带引号的八进制串，
# 与仓里的路径逐字对不上），97 号借「与 56 号同一条」时又省掉了未跟踪文件。
#
# 两个函数：
#   gate_diff_base <取法>
#     gate：GATE_BASE 是个提交就用它；否则 @{upstream} 的 merge-base；都没有（或 merge-base 取不到）就 HEAD。
#           一轮可以跨几个提交，问「这一轮碰了哪些」的阶段用它（56、68、69、97 号）。
#           不认上游 gate.sh 导出的 GATE_DIFF_BASE（用户 2026-09-24 定）：带 --staged 那条路在里层把 GATE_BASE 设成上游
#           diff_base，不带 --staged 的整轮用这里的回退次序；全推出去了时前者是 HEAD~1、后者是 HEAD，两条路判的窗口不同（C528）。
#     head：HEAD。问「这一次提交要带哪些」的阶段用它（11 号；为什么不用 GATE_BASE 写在 11 号头部）。
#   gate_changed_paths <基准> <untracked | tracked-only> [<diff-filter>]
#     一行一条、排序去重：基准到工作区的 diff、HEAD 到暂存区的 diff；第二个参数是 untracked 时再加未跟踪文件。
#     给了 diff-filter（例 A）只在两个 diff 上加；未跟踪文件只在 diff-filter 为空或含 A 时加（它们本来就是新增）。
#     git 调用一律带 -c core.quotepath=false。路径里带制表符、换行或引号的，git 照样转义，这一条管不到。
#   gate_added_lines <基准> <路径>…
#     这次改动新增的行，一行一条「路径<TAB>行文」：跟踪的文件取基准到工作区的 diff 里以 + 开头的行，
#     **未跟踪的文件整份都算新增**（`git diff` 看不到它们，新写的 kb 页在 git add 之前整页会被当成「早先写下的」跳过）。
#     不认改名（搬家之后整份算新增，暂存前后判得一样）；未跟踪的二进制文件不读。
#     路径可以是目录。git 失败、或路径被 git 加了引号（带制表符、引号、反斜杠）时退出码非 0、不输出半截结果，
#     调用方按「拿不到改动范围」处理（通常退回全量判）。
#
# 阶段里的用法：
#   LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
#   source "$LIB_CHANGED_PATHS" || { echo "  ✗ …"; echo "     → …"; exit 1; }
#   base="$(gate_diff_base gate)"
#   changed="$(gate_changed_paths "$base" untracked)"
#
# 住在 research/scripts/ 而不是 .claude/gate.d/：共享门禁（gate.sh）把 gate.d 下每个 *.sh 都当阶段跑，
# 阶段调用的共用 shell 脚本照 stage-must-run.sh、change-touches-crates.sh 的惯例放这里；自证由门禁 47 号跑。
# `--selftest` 在临时 git 仓里自证：三种来源（工作区改、暂存新增、未跟踪）各放一个中文路径，逐字核两个函数的输出。
# 判别力：LIB_CHANGED_PATHS_BREAK=quotepath 让取法漏掉 core.quotepath=false、=untracked 让它丢掉未跟踪文件、
# =fallback 让 gate 取法不认 GATE_BASE、=empty-merge-base 让 merge-base 取不到时交出空串、
# =untracked-lines 让新增行丢掉未跟踪文件、=swallow 让路径取法吞掉 git 的失败、
# =header-anywhere 让「+++ 」在正文里也当文件头认、=binary 让未跟踪的二进制文件也整份读，八种自证都必须判红。
#
#   bash research/scripts/changed-paths.sh --selftest      # 自证
#   LIB_CHANGED_PATHS_BREAK=quotepath bash research/scripts/changed-paths.sh --selftest   # 必须判红

gate_diff_base() {
  local mode="$1" merge_base
  # HEAD 还没出生（刚 git init、orphan 分支）：基准取空树，与上游 lib.sh 对空仓的处置同一种；交出 HEAD 会让 git diff 失败、调用方报取不到
  if ! git rev-parse --verify -q HEAD >/dev/null 2>&1; then git hash-object -t tree /dev/null; return 0; fi
  case "$mode" in
    head)
      echo "HEAD"
      ;;
    gate)
      if [[ "${LIB_CHANGED_PATHS_BREAK:-}" != fallback && -n "${GATE_BASE:-}" ]] \
         && git rev-parse --verify -q "${GATE_BASE}^{commit}" >/dev/null 2>&1; then
        echo "$GATE_BASE"
      elif git rev-parse --verify -q '@{upstream}' >/dev/null 2>&1 \
           && { merge_base="$(git merge-base HEAD '@{upstream}' 2>/dev/null)" || [[ "${LIB_CHANGED_PATHS_BREAK:-}" == empty-merge-base ]]; }; then
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
  [[ "${LIB_CHANGED_PATHS_BREAK:-}" == quotepath ]] && quote_option=()
  [[ -n "$diff_filter" ]] && filter_option=("--diff-filter=$diff_filter")
  case "$untracked_mode" in
    untracked|tracked-only) ;;
    *)
      echo "gate_changed_paths：第二个参数只认 untracked 或 tracked-only，收到「$untracked_mode」" >&2
      return 2
      ;;
  esac
  local include_untracked=0
  if [[ "$untracked_mode" == untracked && "${LIB_CHANGED_PATHS_BREAK:-}" != untracked ]] \
     && [[ -z "$diff_filter" || "$diff_filter" == *A* ]]; then
    include_untracked=1
  fi
  # 三次调用逐条判退出码、先落到变量再排序：写成 `{ …; …; …; } | sort -u` 时花括号组只交最后一条的退出码，
  # 基准写错、第一条 git diff 失败，集合就静默少了一截。
  local listed
  listed="$(
    git "${quote_option[@]}" diff --name-only "${filter_option[@]}" "$base" -- || [[ "${LIB_CHANGED_PATHS_BREAK:-}" == swallow ]] || exit 1
    git "${quote_option[@]}" diff --name-only --cached "${filter_option[@]}" -- || exit 1
    if ((include_untracked)); then git "${quote_option[@]}" ls-files --others --exclude-standard -- || exit 1; fi
  )" || return 1
  [[ -z "$listed" ]] || sort -u <<<"$listed"
}

gate_added_lines() {
  local base="$1"; shift
  local quote_option=(-c core.quotepath=false) diff_output untracked_list untracked_path parsed
  [[ "${LIB_CHANGED_PATHS_BREAK:-}" == quotepath ]] && quote_option=()
  # --no-renames 写死：同一次搬家，暂存前（未跟踪的新名）与暂存后（一次改名）判得要一样；
  # 前缀写死：diff.noprefix 之类的配置会让 +++ 行不带 b/，路径就解析错了。
  diff_output="$(git "${quote_option[@]}" diff --no-renames --unified=0 --no-color --no-ext-diff --src-prefix=a/ --dst-prefix=b/ "$base" -- "$@")" || return 1
  untracked_list="$(git "${quote_option[@]}" ls-files -z --others --exclude-standard -- "$@" | tr '\0' '\n')" || return 1
  # 「+++ 」只在文件头里认（diff --git 与第一个 @@ 之间）：正文里以「++ 」开头的一行新增，在 diff 里也长成「+++ 」。
  # git 给带空格的路径在文件头末尾补一个制表符，剥掉；路径被 git 加了引号（带制表符、引号、反斜杠），这里还原不了，退非 0。
  parsed="$(awk -v header_anywhere="$([[ "${LIB_CHANGED_PATHS_BREAK:-}" == header-anywhere ]] && echo 1)" '
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
  [[ "${LIB_CHANGED_PATHS_BREAK:-}" == untracked-lines ]] && return 0
  while IFS= read -r untracked_path; do
    [[ -n "$untracked_path" && -f "$untracked_path" ]] || continue
    [[ "$untracked_path" != *$'\t'* ]] || return 1
    # 二进制（或空）文件不读：一个 NUL 会让下游的 grep 把整段新增行当二进制吞掉（vim 的交换文件就是这种）
    if [[ "${LIB_CHANGED_PATHS_BREAK:-}" != binary ]]; then grep -Iq . "$untracked_path" || continue; fi
    UNTRACKED_PATH="$untracked_path" awk '{ print ENVIRON["UNTRACKED_PATH"] "\t" $0 }' "$untracked_path"
  done <<<"$untracked_list"
}

# ── 直接跑（不是 source）时：只认 --selftest，在临时仓里自证 ─────────────────
if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  set -uo pipefail
  [[ "${1:-}" == "--selftest" ]] || { echo "  ✗ 用法：bash research/scripts/changed-paths.sh --selftest（平时由门禁阶段 source 它）"; echo "     → 怎么办：自证就带 --selftest；要取改动路径，在阶段里 source 它再调 gate_diff_base / gate_changed_paths。"; exit 2; }
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
    check "head 取法是 HEAD" "HEAD" "$(gate_diff_base head)"
    check "GATE_BASE 是个提交就用它" "$first_commit" "$(GATE_BASE="$first_commit" gate_diff_base gate)"
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
  if [[ -n "$failures" || "$check_count" -ne 15 ]]; then
    echo "  ✗ 共用库 research/scripts/changed-paths.sh 的自证没过（查了 $check_count 项，应当 15 项）："   # gate-lint:summary
    printf '%s\n' "$failures" | sed 's/^FAIL /      /'   # gate-lint:detail
    printf '%s\n' "$selftest_output" | grep -v '^PASS \|^FAIL ' | sed 's/^/      /'   # gate-lint:detail
    echo "     → 怎么办：看 gate_changed_paths / gate_added_lines 的 git 调用有没有带 -c core.quotepath=false、未跟踪文件那一支有没有加 ls-files --others，"
    echo "               gate_diff_base 的回退次序是不是 GATE_BASE → @{upstream} 的 merge-base → HEAD；门禁 11、56、68、69、97 号都靠它取改动范围。"
    exit 1
  fi
  echo "  ✓ 共用库 research/scripts/changed-paths.sh 自证通过（$check_count 项：三种来源的中文路径逐字不转义、未跟踪可开关、diff-filter=A、新增行含未跟踪整份且只看给的路径、带空格路径与「++ 」开头的行、不读二进制、路径带制表符退非 0、基准解析不到退非 0、HEAD 没出生取空树、基准回退四档）"
  exit 0
fi
