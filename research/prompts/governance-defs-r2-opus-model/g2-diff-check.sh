#!/usr/bin/env bash
# 用法：bash g2-diff-check.sh <仓根> <草稿目录>
# 临时仓里造几段「暂存了这一批、工作区另有别的会话的改动」的历史，逐段跑 crash-verifier.md 第 1 步那条原样的
# `git diff --quiet -- <六条路径>`（从定义里现抽），再按每道阶段自己读的路径算「工作区与暂存区在这一道的输入上到底同不同」：
#   55、59：.claude/gate.d/stage-inputs.tsv 里那一行（从仓里现读），外加阶段脚本自己；
#   57：57-lkmm.sh 与 .claude/scripts/lkmm.sh 读的 litmus/、crates/（抓 .rs 里的名字与锚点）、lkmm.sh、57 号自己。
set -uo pipefail
root="$1"; draft="$2"
definition="$root/.claude/agents/crash-verifier.md"
check_command="$(grep -o 'git diff --quiet -- [^`]*' "$definition" | head -1)"
echo "定义里的核对命令：$check_command"
row_paths() { awk -F'\t' -v s="$1" '$1 == s { print $2 }' "$root/.claude/gate.d/stage-inputs.tsv"; }
inputs_55="$(row_paths 55-qemu-first-transaction.sh) .claude/gate.d/55-qemu-first-transaction.sh"
inputs_59="$(row_paths 59-crates-mutation-replay.sh) .claude/gate.d/59-crates-mutation-replay.sh"
inputs_57="litmus/ crates/ .claude/scripts/lkmm.sh .claude/gate.d/57-lkmm.sh"
new_repo() {
  local repo; repo="$(mktemp -d "$draft/g2-repo-XXXXXX")"
  git -C "$repo" init -q; git -C "$repo" config user.email m@x; git -C "$repo" config user.name m
  mkdir -p "$repo/crates/singlefs-harness/tests" "$repo/litmus" "$repo/research/scripts" "$repo/research/results" "$repo/.claude/scripts" "$repo/.claude/gate.d"
  echo 'fn publish() {}' > "$repo/crates/singlefs-harness/src.rs"; echo '[workspace]' > "$repo/Cargo.toml"; echo 'lock' > "$repo/Cargo.lock"
  echo 'C x' > "$repo/litmus/a.litmus"; echo 'E142|x|y|e142.out' > "$repo/research/scripts/replay.sh"; echo cap > "$repo/research/scripts/run-with-memory-cap.sh"
  echo out > "$repo/research/results/e142.out"; echo lkmm-v1 > "$repo/.claude/scripts/lkmm.sh"
  for s in 55-qemu-first-transaction 57-lkmm 59-crates-mutation-replay; do echo "$s v1" > "$repo/.claude/gate.d/$s.sh"; done
  git -C "$repo" add -A; git -C "$repo" commit -qm base; echo "$repo"
}
judge() {
  local repo="$1" label="$2" rc stage inputs differs
  ( cd "$repo" && eval "$check_command" ); rc=$?
  printf '%s\n  定义那条命令退 %s（%s）\n' "$label" "$rc" "$([[ $rc == 0 ]] && echo 三道都照跑 || echo 三道都停下报告)"
  for stage in 55 57 59; do
    inputs_var="inputs_$stage"; inputs="${!inputs_var}"
    # shellcheck disable=SC2086
    if ( cd "$repo" && git diff --quiet -- $inputs ); then differs=同; else differs=不同; fi
    printf '  %s 号自己读的路径上工作区与暂存区：%s\n' "$stage" "$differs"
  done
  printf '  未跟踪（定义要求原样列进报告、不停）：%s\n' "$(cd "$repo" && git ls-files --others --exclude-standard -- crates Cargo.toml Cargo.lock litmus research/scripts research/results | tr '\n' ' ')"
}
repo="$(new_repo)"; echo 'fn publish() { /* 这一批 */ }' > "$repo/crates/singlefs-harness/src.rs"; git -C "$repo" add crates
echo 'E999|别的会话的执行员第 5 步登记的复跑行' >> "$repo/research/scripts/replay.sh"
judge "$repo" "S1 这一批暂存了 crates/；别的会话的执行员往 research/scripts/replay.sh 追加了一行、没暂存"
repo="$(new_repo)"; echo lkmm-v2-这一批 > "$repo/.claude/scripts/lkmm.sh"; git -C "$repo" add .claude/scripts/lkmm.sh
echo lkmm-v3-工作区里别的会话还在改 > "$repo/.claude/scripts/lkmm.sh"
judge "$repo" "S2 这一批暂存了 .claude/scripts/lkmm.sh；工作区里那一份又被改了、没暂存"
repo="$(new_repo)"; echo '59 v2 这一批' > "$repo/.claude/gate.d/59-crates-mutation-replay.sh"; git -C "$repo" add .claude/gate.d
echo '59 v3 工作区' > "$repo/.claude/gate.d/59-crates-mutation-replay.sh"
judge "$repo" "S3 这一批暂存了 59 号脚本；工作区那一份不同"
repo="$(new_repo)"; echo 'fn publish() { /* 这一批 */ }' > "$repo/crates/singlefs-harness/src.rs"; git -C "$repo" add crates
echo '#[test] fn wip() { panic!() }' > "$repo/crates/singlefs-harness/tests/wip_from_other_session.rs"
judge "$repo" "S4 这一批暂存了 crates/；别的会话在 crates/singlefs-harness/tests/ 下新建了一个没跟踪的测试文件"
