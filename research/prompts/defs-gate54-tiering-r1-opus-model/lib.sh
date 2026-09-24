#!/usr/bin/env bash
# 攻方腿的共用装置：造合成仓、仿 gate.sh --staged 只跑 54 号那一段。只读主仓（cp 与 git show），不写主仓、不碰主仓的 git common-dir。
set -uo pipefail
MAIN_REPO=/home/fy5090/code/singlefs
MODEL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export PATH="$MODEL_DIR/fake-bin:$PATH"
unset SINGLEFS_LAYER0_THREADS SINGLEFS_STAGED_TREE GATE_BASE SINGLEFS_GATE_FULL FAKE_CARGO_SLEEP_FULL FAKE_CARGO_HOOK_BEFORE_SECOND FAKE_CARGO_HOOK_AFTER_SECOND
STAGE=.claude/gate.d/54-layer0-replay.sh
gitq() { git -c user.email=attack@example.com -c user.name=attack "$@"; }

# build_repo <目录> <post|pre>：post＝工作区今天的 54 号与 stage-inputs.tsv（开工快照里的那一版），pre＝HEAD 的（分档之前）。
build_repo() {
  local repo="$1" arm="$2"
  rm -rf -- "${repo:?}"; mkdir -p "$repo/.claude/gate.d" "$repo/research/scripts" "$repo/research/results" "$repo/.claude/kb/layout" \
    "$repo/crates/singlefs-harness/src" "$repo/crates/singlefs-harness/tests" "$repo/crates/singlefs-core/src"
  if [[ "$arm" == post ]]; then
    cp "$MAIN_REPO/$STAGE" "$repo/$STAGE"; cp "$MAIN_REPO/.claude/gate.d/stage-inputs.tsv" "$repo/.claude/gate.d/stage-inputs.tsv"
  else
    git -C "$MAIN_REPO" show "HEAD:$STAGE" > "$repo/$STAGE"; git -C "$MAIN_REPO" show HEAD:.claude/gate.d/stage-inputs.tsv > "$repo/.claude/gate.d/stage-inputs.tsv"
  fi
  chmod +x "$repo/$STAGE"
  cp "$MAIN_REPO/research/scripts/stage-must-run.sh" "$MAIN_REPO/research/scripts/change-touches-crates.sh" "$repo/research/scripts/"
  printf '[workspace]\nmembers = ["crates/*"]\n' > "$repo/Cargo.toml"
  printf '# lock v1\n[[package]]\nname = "dep"\nversion = "1.0.0"\n' > "$repo/Cargo.lock"
  printf 'pub mod crash;\n' > "$repo/crates/singlefs-harness/src/lib.rs"
  printf 'pub fn enumerate() {}\n' > "$repo/crates/singlefs-harness/src/crash.rs"
  printf '#[test] fn quick() {}\n' > "$repo/crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs"
  printf '#[test] fn quick_b() {}\n' > "$repo/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs"
  printf 'pub fn core() {}\n' > "$repo/crates/singlefs-core/src/lib.rs"
  printf '# 变异表\n' > "$repo/crates/mutations.tsv"
  printf 'layout\n' > "$repo/.claude/kb/layout/01-first-txn.md"; printf 'result\n' > "$repo/research/results/e142-sample.out"
  gitq -C "$repo" init -q; gitq -C "$repo" add -A; gitq -C "$repo" commit -q -m base
}

marker_of() { echo "$(git -C "$1" rev-parse --path-format=absolute --git-common-dir)/singlefs-layer0-full-green"; }
show_marker() { # <仓>
  local m; m="$(marker_of "$1")"
  if [[ -f "$m" ]]; then echo "[标记：input_hash=$(sed -n 's/^input_hash=//p' "$m" | cut -c1-16)… judged_root=$(sed -n 's/^judged_root=//p' "$m")；$(grep -E '^LAYER0 ' "$m" | grep -o 'exhaustive=[a-z]*')]"
  else echo "[标记不在]"; fi
}
# run <标题> <命令…>：打命令、原样输出、退出码
run() { local t="$1"; shift; echo "──── $t"; echo "\$ $*"; "$@"; local rc=$?; echo "[退出码 $rc]"; return 0; }

# gate_staged_54 <仓> [额外环境…]：照共享 gate.sh --staged 的建法（.claude/singlefs-ai-sop/scripts/gate.sh 第 54–97 行）
# 建临时 worktree（worktree add --detach HEAD + apply --index 暂存区 diff），在里面按 gate.sh 的调法 `bash <阶段> <根>` 只跑 54 号。
gate_staged_54() {
  local repo="$1"; shift
  local base tree rc
  base="$(mktemp -d /tmp/claude-1000/defs54-attack/staged.XXXX)"; tree="$base/tree"
  git -C "$repo" diff --cached --binary > "$base/staged.patch"
  gitq -C "$repo" worktree add -q --detach "$tree" HEAD
  [[ -s "$base/staged.patch" ]] && git -C "$tree" apply --index "$base/staged.patch"
  echo "\$ (临时 worktree = HEAD + 暂存区) env $* GATE_BASE=HEAD GATE_IN_STAGE=1 bash $STAGE <临时 worktree>"
  ( cd "$tree" && env "$@" GATE_BASE=HEAD GATE_IN_STAGE=1 bash "$tree/$STAGE" "$tree" ); rc=$?
  echo "[gate --staged 里 54 号的退出码 $rc]"
  gitq -C "$repo" worktree remove --force "$tree"; rm -rf -- "${base:?}"
  return 0
}

# build_batch_worktree <仓> <目录>：出路句「只含这一批的 worktree」的一种建法（与 gate.sh --staged 同建法，留着不删）。
build_batch_worktree() {
  local repo="$1" tree="$2"
  git -C "$repo" diff --cached --binary > "$tree.patch"
  gitq -C "$repo" worktree add -q --detach "$tree" HEAD
  [[ -s "$tree.patch" ]] && git -C "$tree" apply --index "$tree.patch"
  return 0
}
