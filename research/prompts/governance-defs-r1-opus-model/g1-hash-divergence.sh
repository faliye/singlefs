#!/usr/bin/env bash
# G1 模型：54 号全绿标记的键（输入内容哈希）在三棵树上各算一次，函数原样取自工作区 .claude/gate.d/54-layer0-replay.sh 的 write_layer0_input_manifest。
#   full   ＝主 agent 照 main-agent.md 在 HEAD + 暂存区 worktree 里跑 --full 时写的那一格的键
#   cv     ＝crash-verifier 照改后 crash-verifier.md 第 2 步在主工作区跑 `bash .claude/gate.d/54-layer0-replay.sh`（快档）时找的键
#   staged ＝gate-triage 的 `gate.sh --staged` 在另一个 HEAD + 暂存区 worktree 里跑快档时找的键
# 固定的前缀：一个临时仓（只在草稿目录里），这一轮暂存了一处 crates/ 改动；放开扫的是「暂存之后、crash-verifier 开跑之前，主工作区里 54 号登记路径下还有什么」。
# 用法：bash g1-hash-divergence.sh <草稿目录>；不跑 cargo test，只跑 cargo -V / rustc -V。
set -uo pipefail
REPO=/home/user/singlefs
SCRATCH=${1:?草稿目录}
STAGE="$REPO/.claude/gate.d/54-layer0-replay.sh"
FUNCTION_TEXT="$(sed -n '/^write_layer0_input_manifest() {/,/^}/p' "$STAGE")"
[[ -n "$FUNCTION_TEXT" ]] || { echo "取不到 write_layer0_input_manifest"; exit 1; }
eval "$FUNCTION_TEXT"
layer0_registered_input_paths=(crates/ Cargo.toml Cargo.lock)   # stage-inputs.tsv 里 54 号那一行
key_of() { # <根>
  ROOT="$1"; layer0_stage_script_path="$1/.claude/gate.d/54-layer0-replay.sh"; layer0_stage_file_name=54-layer0-replay.sh
  local manifest; manifest="$(mktemp -p "$SCRATCH")"
  write_layer0_input_manifest "$manifest" || { echo "算不出"; return; }
  echo "${layer0_input_hash:0:16} files=${layer0_input_file_count}"
}
make_repo() { # <目录>：HEAD 里有 crates/a.rs、crates/b.rs、Cargo.toml、Cargo.lock、.gitignore（忽略 target/）、54 号
  local d="$1"; mkdir -p "$d/crates/singlefs-core/src" "$d/.claude/gate.d"
  git -C "$d" init -q; git -C "$d" config user.email m@example; git -C "$d" config user.name model
  printf 'fn a() {}\n' > "$d/crates/singlefs-core/src/a.rs"; printf 'fn b() {}\n' > "$d/crates/singlefs-core/src/b.rs"
  printf '[workspace]\n' > "$d/Cargo.toml"; printf '# lock\n' > "$d/Cargo.lock"; printf 'target/\n' > "$d/.gitignore"
  cp "$STAGE" "$d/.claude/gate.d/54-layer0-replay.sh"
  git -C "$d" add -A; git -C "$d" commit -qm base
  printf 'fn a() { /* this round */ }\n' > "$d/crates/singlefs-core/src/a.rs"; git -C "$d" add crates/singlefs-core/src/a.rs   # 这一轮暂存的改动
}
staged_tree() { # <仓> <基目录>：照 54 号出路 print_staged_worktree_full_commands 的建法
  mkdir -p "$2"; git -C "$1" diff --cached --binary > "$2/staged.patch"
  git -C "$1" worktree add -q --detach "$2/tree" HEAD && { [ ! -s "$2/staged.patch" ] || git -C "$2/tree" apply --index "$2/staged.patch"; }
}
run_case() { # <名字> <在主工作区里做的事（shell 片段，$d 是仓）>
  local name="$1" action="$2" base; base="$(mktemp -d -p "$SCRATCH")"; local d="$base/repo"; make_repo "$d"
  staged_tree "$d" "$base/full"; local full; full="$(key_of "$base/full/tree")"
  ( cd "$d" && eval "$action" )
  local cv; cv="$(key_of "$d")"
  staged_tree "$d" "$base/staged"; local staged; staged="$(key_of "$base/staged/tree")"
  local verdict_cv verdict_staged
  [[ "$cv" == "$full" ]] && verdict_cv="找得到那一格(绿)" || verdict_cv="找不到那一格(红)"
  [[ "$staged" == "$full" ]] && verdict_staged="找得到那一格(绿)" || verdict_staged="找不到那一格(红)"
  printf '%-34s full=%s | cv=%s → %s | staged=%s → %s\n' "$name" "$full" "$cv" "$verdict_cv" "$staged" "$verdict_staged"
  git -C "$d" worktree remove --force "$base/full/tree"; git -C "$d" worktree remove --force "$base/staged/tree"
}
run_case "C0 对照：主工作区与暂存区一致" ':'
run_case "C1 别的会话改了已跟踪文件、未暂存" 'printf "fn b() { /* other session */ }\n" > crates/singlefs-core/src/b.rs'
run_case "C2 别的会话新建未跟踪文件" 'printf "fn c() {}\n" > crates/singlefs-core/src/c.rs'
run_case "C3 自己一处改动漏暂存（Cargo.lock）" 'printf "# lock v2\n" > Cargo.lock'
run_case "C4 删了已跟踪文件、删除未暂存" 'rm crates/singlefs-core/src/b.rs'
run_case "C5 被忽略的文件（crates/target/）" 'mkdir -p crates/target && printf x > crates/target/junk'
run_case "C6 暂存之后又改了已暂存的文件" 'printf "fn a() { /* edited after staging */ }\n" > crates/singlefs-core/src/a.rs'
run_case "C7 主工作区的 54 号有未暂存改动" 'printf "# local edit\n" >> .claude/gate.d/54-layer0-replay.sh'
