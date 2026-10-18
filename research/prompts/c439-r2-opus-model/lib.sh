#!/usr/bin/env bash
# C439 第二轮攻方腿的造仓公用函数。用法：source lib.sh <冻结副本根> <工具副本根> <草稿根>
# 冻结副本只读；临时仓都建在草稿根下，跑完由调用方删。
set -uo pipefail
FROZEN="$1"; TOOLS="$2"; WORK="$3"
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=model GIT_AUTHOR_EMAIL=model@example.invalid GIT_COMMITTER_NAME=model GIT_COMMITTER_EMAIL=model@example.invalid
export GIT_AUTHOR_DATE='@1760000000 +0900' GIT_COMMITTER_DATE='@1760000000 +0900'   # clock-times:allow 造历史要固定提交时刻，提交号逐次一样
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GATE_BASE LIB_CHANGED_PATHS_BREAK EXPERIMENT_PAGES_AND_PRODUCTS_BREAK SINGLEFS_GATE_FULL
# quoted-result-lines 格的判法：冻结 doc-experiments.sh 第 1625–1796 行（python 的 heredoc 正文）原样抽出来跑。
# 每个进程各抽一份（并行扫描时共用一份会被别的进程重写到一半，空文件跑出退 0，K3 第一次扫就是这样扫出了假的漏判）
CELL_PY="$(mktemp "$WORK/cell-quoted-result-lines.XXXXXX.py")"; trap 'rm -f "$CELL_PY"' EXIT
awk 'NR>=1625 && NR<=1796' "$FROZEN/.claude/gate.d/doc-experiments.sh" > "$CELL_PY"
[[ "$(wc -l < "$CELL_PY")" == 172 && "$(head -1 "$CELL_PY")" == "import glob, os, subprocess, sys" ]] || { echo "抽出来的格判法不对" >&2; exit 2; }
new_repo() {   # new_repo <目录>：空仓，带 replay 登记表、一份产物（对的行）与 .gitignore
  rm -rf "$1"; mkdir -p "$1"; cd "$1" || exit 2
  git init -q -b master .
  mkdir -p .claude/kb/experiments research/results scripts/experiments
  printf '/.claude/singlefs-ai-sop/\n' > .gitignore
  printf 'E900|e900|--x|e900-x.out|exact\n' > scripts/experiments/replay.sh
  printf 'E7RESULT name=alpha value=1\nE7RESULT name=beta value=1\n' > research/results/e900-x.out
  printf '# E900\n\nE7RESULT name=alpha value=1\n' > .claude/kb/experiments/900-x.md
  git add -A && git commit -qm root
}
green_tree_commit() { git commit-tree "$1" -m "gate --staged 整轮全绿"; }
sg_tree() { git rev-parse -q --verify 'refs/sop/staged-green^{tree}' 2>/dev/null || git hash-object -t tree /dev/null; }
sg_commit_or_empty() {   # 庚二的基准：staged-green 那个提交；没有就空树（第一轮 c439-model.sh 第 75 行的取法），空树包成一个提交
  git rev-parse -q --verify 'refs/sop/staged-green^{commit}' 2>/dev/null || git commit-tree "$(git hash-object -t tree /dev/null)" -m empty
}
run_cell() {   # run_cell <被判的根> <基准提交|空串=全量>：照冻结 doc-experiments.sh 第 128–139、1617–1622 行取窗口再跑格；打 rc= 与输出
  local root="$1" base="$2" added rc
  ( cd "$root" || exit 2
    source "$FROZEN/scripts/gate/changed-paths.sh"
    added="$(mktemp)"
    if [[ -n "$base" ]]; then
      export GATE_BASE="$base"
      shared_base="$(gate_diff_base gate)"
      if gate_added_lines "$shared_base" .claude/kb ':(glob)research/**/*.md' > "$added"; then SCOPE_BASE="$shared_base"; else SCOPE_BASE=""; fi
    else
      SCOPE_BASE=""
    fi
    export SCOPE_BASE ADDED_LINES="$added"
    echo "    window_base=${SCOPE_BASE:-全量} added_E7RESULT_lines=$(grep -c $'\tE7RESULT' "$added" 2>/dev/null || true)"
    python3 "$CELL_PY" 2>&1 | sed 's/^/    /'
    rc=${PIPESTATUS[0]}; rm -f "$added"; echo "    cell_rc=$rc"; exit "$rc" )
}
judge_tree() {   # judge_tree <树> <基准提交|空串>：庚二落法「里层判的就是外壳那棵树」——从这棵树建临时 worktree 再跑格
  local tree="$1" base="$2" wt commit rc
  commit="$(git commit-tree "$tree" -m judged)"; wt="$(mktemp -d "$WORK/wt.XXXXXX")"; rmdir "$wt"
  git worktree add -q --detach "$wt" "$commit"
  run_cell "$wt" "$base"; rc=$?
  git worktree remove --force "$wt"; return "$rc"
}
geng2_gate() {   # geng2_gate [环境赋值…]：庚二一趟 gate-staged（落法 A）：write-tree、按 staged-green 判、退 0 且没人给 GATE_BASE、没带 --force 就前移
  local tree base rc
  tree="$(git write-tree)"; base="$(sg_commit_or_empty)"
  echo "  gate: judged_tree=${tree:0:12} base_tree=$(git rev-parse "$base^{tree}" | cut -c1-12)"
  env "$@" bash -c "$(declare -f judge_tree run_cell); FROZEN='$FROZEN' WORK='$WORK' CELL_PY='$CELL_PY' judge_tree '$tree' '$base'"; rc=$?
  if [[ $rc -eq 0 || $rc -eq 77 ]]; then git update-ref refs/sop/staged-green "$(green_tree_commit "$tree")"; echo "  staged-green -> ${tree:0:12}（格 rc=$rc，整轮按退 0 记）"; else echo "  staged-green 不动（格 rc=$rc）"; fi
}
