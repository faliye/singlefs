#!/usr/bin/env bash
# C439 正推腿模型库：在临时 git 仓里模拟「整轮门禁两条路（不带 / 带 --staged）算出的窗口」与各候选的基准。
# 只 source 冻结副本的 scripts/gate/changed-paths.sh 与上游 lib.sh 的 diff_base，不起任何门禁阶段、不跑 gate.sh。
# 必须先设：FROZEN（冻结副本）、WORK（草稿目录，仓建在 $WORK/repos 下）。
# 「判」的模拟：窗口里出现 tag 含 bad 的 E7RESULT 行 ⇒ 这一趟红（等于 quoted-result-lines 格报出错引文）。
# 模拟的前移规则逐条照冻结副本：
#   不带 --staged 整轮绿 ⇒ update-ref refs/sop/gate-ok 到开跑时的 HEAD（gate.sh:774-785，GATE_BASE 为空才写）
#   带 --staged 整轮绿且暂存树没变 ⇒ refs/sop/staged-green 前移到包着暂存树的无父提交（gate-staged.sh:36-48）
set -uo pipefail
: "${FROZEN:?}" "${WORK:?}"
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=model GIT_AUTHOR_EMAIL=model@example.invalid GIT_COMMITTER_NAME=model GIT_COMMITTER_EMAIL=model@example.invalid
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GATE_BASE
# shellcheck disable=SC1091
source "$FROZEN/scripts/gate/changed-paths.sh"
# 冻结副本只带 10 份文件，没有 lib.sh 第 53 行要 source 的 preflight.sh（diff_base 不用它）：把冻结副本的 lib.sh 原样拷到草稿目录，旁边放一个空的 preflight.sh，source 才不在 set -e 下当场退出
UPSTREAM_STUB="$WORK/upstream-lib-stub"; mkdir -p "$UPSTREAM_STUB"
cp -f "$FROZEN/.claude/singlefs-ai-sop/scripts/lib.sh" "$UPSTREAM_STUB/lib.sh"; [[ -f "$UPSTREAM_STUB/preflight.sh" ]] || : > "$UPSTREAM_STUB/preflight.sh"
UPSTREAM_LIB="$UPSTREAM_STUB/lib.sh"
SCOPE=(.claude/kb ':(glob)research/**/*.md')     # doc-experiments.sh:139 取新增行的路径集
EMPTY_TREE="$(git hash-object -t tree /dev/null)"
REPOS="$WORK/repos"; mkdir -p "$REPOS"
STEP=0

# ── 造仓与改文件 ──────────────────────────────────────────
new_repo() {   # new_repo <名字>：HEAD=c0，页 a.md 里有一行好的引文
  NAME="$1"; REPO="$REPOS/$1"; rm -rf "$REPO"; mkdir -p "$REPO"; cd "$REPO" || exit 2
  git init -q -b master . ; git config core.fsync none; git config gc.auto 0; mkdir -p .claude/kb
  printf '# 页 a\nE7RESULT ok_0=1\n' > .claude/kb/a.md
  printf '# 页 b\n' > .claude/kb/b.md
  git add -A && git commit -qm c0
  : > "$(git rev-parse --git-dir)/ledger.tsv"; STEP=0
}
add_line() {   # add_line <页 a|b> <tag>：往页里追加一行 E7RESULT <tag>=1（tag 含 bad 的是错引文）
  printf 'E7RESULT %s=1\n' "$2" >> ".claude/kb/$1.md"
}
stage_all() { git add -A; }
commit_staged() { git commit -qm "${1:-c}"; }
commit_all() { git add -A; git commit -qm "${1:-c}"; }    # 绕过门禁的提交
set_upstream() {   # set_upstream <rev>：模拟「推出去了」：origin/master 指到 <rev>，master 的上游设成它
  git config remote.origin.url /nonexistent; git config remote.origin.fetch '+refs/heads/*:refs/remotes/origin/*'
  git config branch.master.remote origin; git config branch.master.merge refs/heads/master
  git update-ref refs/remotes/origin/master "$(git rev-parse "$1")"
}

# ── 基准：今天的两条路 ──────────────────────────────────────
upstream_diff_base() { ( source "$UPSTREAM_LIB" >/dev/null 2>&1; diff_base "$PWD" ); }   # lib.sh:144 起
base_today_N() { ( unset GATE_BASE; gate_diff_base gate ); }                              # changed-paths.sh:67 起，不带 --staged
base_today_S() { upstream_diff_base; }                                                    # gate.sh:102、115：里层 GATE_BASE
short() { git rev-parse --short "$1" 2>/dev/null || echo "$1"; }
label_of() { local r="$1"; if [[ "$r" == "$EMPTY_TREE" ]]; then echo "空树"; elif [[ "$r" == HEAD ]]; then echo "HEAD=$(git log -1 --format=%s HEAD)"; else git log -1 --format=%s "$r"; fi; }   # 基准用提交说明标，哈希每趟不同

# ── 候选的基准（path: N 不带 --staged；S 带 --staged；在源仓里算）──────────
cand_base() {   # cand_base <候选> <N|S>；MODEL_BREAK=bing-as-today 把丙退回今天的取法（给 h5 证红用）
  local cand="$1" path="$2" g r
  if [[ "${MODEL_BREAK:-}" == bing-as-today && ( "$cand" == bing_s || "$cand" == bing_sn ) ]]; then cand=today; fi
  case "$cand" in
    today) if [[ "$path" == N ]]; then base_today_N; else base_today_S; fi ;;
    jia)   echo HEAD ;;                                      # 甲：带不带 --staged 都取 HEAD
    yi)    r="$(git rev-parse -q --verify refs/sop/round-start 2>/dev/null)" || r=""
           if [[ -n "$r" ]] && git merge-base --is-ancestor "$r" HEAD 2>/dev/null; then echo "$r"
           elif [[ "$path" == N ]]; then base_today_N; else base_today_S; fi ;;   # 乙：开轮记下的起点，没记退回今天
    bing_s)  if [[ "$path" == S ]] && g="$(git rev-parse -q --verify 'refs/sop/staged-green^{commit}' 2>/dev/null)"; then echo "$g"; elif [[ "$path" == N ]]; then base_today_N; else base_today_S; fi ;;   # 丙，只改带 --staged 一条路
    bing_sn) if g="$(git rev-parse -q --verify 'refs/sop/staged-green^{commit}' 2>/dev/null)"; then echo "$g"; elif [[ "$path" == N ]]; then base_today_N; else base_today_S; fi ;;   # 丙，两条路都取
    ding)  r=""; local c
           while read -r c; do if git notes --ref=sop-green list "$c" >/dev/null 2>&1; then r="$c"; break; fi; done < <(git rev-list --first-parent HEAD)
           if [[ -n "$r" ]]; then echo "$r"; elif [[ "$path" == N ]]; then base_today_N; else base_today_S; fi ;;   # 丁：最近的带标记祖先（HEAD 本身带标记时窗口只有未提交的）
    geng)  if [[ "$path" == N ]]; then   # 庚：gate_diff_base gate 的回退次序补一档 gate-ok（GATE_BASE → 上游 → gate-ok → HEAD），带 --staged 那条路不动
             r="$(base_today_N)"
             if [[ "$r" == HEAD ]] && g="$(git rev-parse -q --verify refs/sop/gate-ok 2>/dev/null)" && [[ "$g" != "$(git rev-parse HEAD)" ]] && git merge-base --is-ancestor "$g" HEAD 2>/dev/null; then echo "$g"; else echo "$r"; fi
           else base_today_S; fi ;;
    wu)    echo "$EMPTY_TREE" ;;                             # 戊：不按窗口判（全量），另扣存量豁免表
    *) echo "未知候选 $cand" >&2; return 2 ;;
  esac
}
CANDS=(today jia yi bing_s bing_sn ding wu geng)
cand_name() { case "$1" in today) echo 今天;; jia) echo 甲;; yi) echo 乙;; bing_s) echo 丙只改带暂存;; bing_sn) echo 丙两条路;; ding) echo 丁;; wu) echo 戊;; geng) echo 庚;; esac; }

# ── 跑一趟（模拟）──────────────────────────────────────────
ledger() { printf '%s\t%s\t%s\t%s\t%s\n' "$STEP" "$1" "$2" "$3" "$4" >> "$(git rev-parse --git-dir)/ledger.tsv"; }
tags_in_window() {   # 读 stdin 的「路径<TAB>行文」，打出 E7RESULT 行的 tag（等号之前）
  awk -F'\t' '{ t=$2; sub(/^[ \t]+/,"",t); if (t ~ /^E7RESULT /) { sub(/^E7RESULT /,"",t); sub(/=.*/,"",t); print t } }' | sort -u
}
EXEMPT_FILE=""   # 戊的存量豁免表（一行一个 tag）
ding_mark() {   # 丁：这一趟开头，若 HEAD 的树就是 staged-green 的树，给 HEAD 挂标记
  local g; g="$(git rev-parse -q --verify 'refs/sop/staged-green^{tree}' 2>/dev/null)" || return 0
  [[ "$(git rev-parse HEAD^{tree})" == "$g" ]] && git notes --ref=sop-green add -f -m green HEAD 2>/dev/null
  return 0
}
run_gate() {   # run_gate <N|S> <候选> <标签>：登记窗口、判红绿、绿则按冻结副本的规则前移。返回 0 绿 1 红
  local path="$1" cand="$2" label="$3" base tags verdict start_head tree_before wt
  STEP=$((STEP+1)); start_head="$(git rev-parse HEAD)"
  [[ "$cand" == ding ]] && ding_mark
  base="$(cand_base "$cand" "$path")" || return 2
  if [[ "$path" == S ]]; then
    tree_before="$(git write-tree)"
    wt="$WORK/wt-$NAME"; rm -rf "$wt"; git worktree prune
    git worktree add -q --detach "$wt" HEAD || return 2
    git diff --cached --binary > "$WORK/staged-$NAME.patch"
    [[ -s "$WORK/staged-$NAME.patch" ]] && ( cd "$wt" && git apply --index "$WORK/staged-$NAME.patch" )
    tags="$( cd "$wt" && export GATE_BASE="$base" && { if [[ "$GATE_BASE" == "$EMPTY_TREE" ]]; then b2="$GATE_BASE"; else b2="$(gate_diff_base gate)"; fi; } && gate_added_lines "$b2" "${SCOPE[@]}" | tags_in_window )"
    git worktree remove --force "$wt"
  else
    tags="$( unset GATE_BASE; gate_added_lines "$base" "${SCOPE[@]}" | tags_in_window )"
  fi
  verdict=green
  local t
  for t in $tags; do
    if [[ "$t" == *bad* ]]; then
      if [[ -n "$EXEMPT_FILE" && -f "$EXEMPT_FILE" ]] && grep -qxF "$t" "$EXEMPT_FILE"; then continue; fi
      verdict=red
    fi
  done
  ledger "$path" "$cand" "$(label_of "$base")" "$(echo $tags | tr ' ' ',')|$verdict|$label"
  if [[ "$verdict" == green ]]; then
    if [[ "$path" == N ]]; then
      git update-ref refs/sop/gate-ok "$start_head"                                     # gate.sh:781
    else
      if [[ "$(git write-tree)" == "$tree_before" ]]; then                              # gate-staged.sh:38-39
        git update-ref refs/sop/staged-green "$(git commit-tree "$tree_before" -m "绿树@$(git log -1 --format=%s HEAD)")"   # gate-staged.sh:45-46
      fi
    fi
  fi
  [[ "$verdict" == green ]]
}
show_runs() {   # 打出这个仓的账本：步 / 路 / 候选 / 基准 / 窗口里的 tag | 红绿 | 说明
  while IFS=$'\t' read -r step path cand base rest; do printf '  步%s %s %s 基准=%s 窗口={%s}\n' "$step" "$path" "$(cand_name "$cand")" "$base" "${rest//|/ }"; done < "$(git rev-parse --git-dir)/ledger.tsv"
}
ever_in_window() {   # ever_in_window <tag>：账本里是否有一趟的窗口含它（含红的那趟）：打 yes <首趟步号> 或 no
  awk -F'\t' -v t="$1" '{ split($5,a,"|"); n=split(a[1],w,","); for(i=1;i<=n;i++) if (w[i]==t) { print "yes 步" $1 " " $2; found=1; exit } } END{ if(!found) print "no" }' "$(git rev-parse --git-dir)/ledger.tsv"
}
