#!/usr/bin/env bash
# T5：七处判了共用取法的退出码之后，哪一种正常仓状态让它们红（对照 HEAD 那一版：还没有共用脚本、各自取法）。
# 用法：bash t5-states.sh <快照树> <真仓> <草稿目录>   真仓只用来 git show HEAD 那一版的阶段（只读）。
set -uo pipefail
SNAP="$(cd "$1" && pwd)"; REPO="$(cd "$2" && pwd)"; D="$3"
rm -rf "$D"; mkdir -p "$D/head/.claude/gate.d"
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
unset GATE_BASE GATE_DIFF_BASE GATE_STAGED_FROM GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE
STAGES=(11-batch-scope 56-crates-adversarial-review 68-knowledge-sync 69-evidence-in-repo 75-decision-experiment-links 97-invariant-field-anchors 61-settled-same-file)
for s in "${STAGES[@]}"; do git -C "$REPO" show "HEAD:.claude/gate.d/$s.sh" > "$D/head/.claude/gate.d/$s.sh"; done
git -C "$REPO" show HEAD:.claude/gate.d/knowledge-sync-triggers.tsv > "$D/triggers.tsv"
tree() {  # 各阶段要的最小一套文件
  mkdir -p .claude/kb/decisions .claude/kb/experiments crates/demo/src research/results records .claude/gate.d
  cp "$D/triggers.tsv" .claude/gate.d/knowledge-sync-triggers.tsv   # 11、68 号读它
  printf '## D1 甲 —— 已定\n\n## 历史版本\n' > .claude/kb/decisions/01-甲.md
  printf '# 不变量\n' > .claude/kb/invariants.md
  printf 'pub fn f() {}\n' > crates/demo/src/lib.rs
  [[ -z "${NO_CRATES:-}" ]] || rm -rf crates   # 空仓那两格不放 crates，56 号在 HEAD 版上无对象可判（77），分得出「取不到」是新红
}
judge() {  # $1 = 阶段目录，$2 = 仓；每道打「号:退出码」，取不到改动范围的标 *
  local s out rc line=""
  for s in "${STAGES[@]}"; do
    rc=0; out="$(cd "$2" && nice -n 19 bash "$1/$s.sh" "$2" 2>&1)" || rc=$?
    grep -qE '取不到这次改动碰了哪些路径|取不到 .* 这次新增了哪些行' <<<"$out" && rc="$rc*"
    line+="${s%%-*}:$rc "
  done
  echo "$line"
}
row() { printf '  %-44s 快照 %s\n  %-44s HEAD %s\n' "$1" "$(judge "$SNAP/.claude/gate.d" "$2")" "" "$(judge "$D/head/.claude/gate.d" "$2")"; }
echo "（* = 阶段报「取不到这次改动碰了哪些路径 / 新增了哪些行」退出）"
mkdir -p "$D/empty" && (cd "$D/empty" && git init -q -b master . && NO_CRATES=1 tree)
row '刚 git init、还没有提交，文件都未跟踪' "$D/empty"
mkdir -p "$D/one" && (cd "$D/one" && git init -q -b master . && tree && git add -A && git commit -qm c0 && echo 'pub fn g() {}' >> crates/demo/src/lib.rs)
row '一个提交、没有上游、工作区改了一处' "$D/one"
(cd "$D/one" && git checkout -q --orphan pages && git rm -rqf --cached . && rm -rf crates)
row '同一个仓切到 orphan 分支（HEAD 未出生）' "$D/one"
mkdir -p "$D/origin" && (cd "$D/origin" && git init -q -b master . && tree && git add -A && git commit -qm c0 && echo a >> records/a && git add -A && git commit -qm c1)
git clone -q --depth 1 "file://$D/origin" "$D/shallow" && (cd "$D/shallow" && echo 'pub fn h() {}' >> crates/demo/src/lib.rs)
row '浅克隆（--depth 1，有上游）、工作区改了一处' "$D/shallow"
(cd "$D/origin" && echo b >> records/a && git commit -qam c2 && echo c >> records/a && git commit -qam c3)
(cd "$D/shallow" && git fetch -q --depth 1 origin)
row '浅克隆，上游又前进两步（merge-base 取不到）' "$D/shallow"
mkdir -p "$D/src" && (cd "$D/src" && git init -q -b master . && tree && git add -A && git commit -qm c0 && echo 'pub fn k() {}' >> crates/demo/src/lib.rs && git add -A \
  && git worktree add -q --detach "$D/staged" HEAD && git diff --cached --binary | git -C "$D/staged" apply --index)
printf '  %-44s 快照 %s\n' '照 gate.sh --staged 建的临时 worktree' "$(GATE_BASE="$(git -C "$D/src" rev-parse HEAD)" judge "$SNAP/.claude/gate.d" "$D/staged")"
