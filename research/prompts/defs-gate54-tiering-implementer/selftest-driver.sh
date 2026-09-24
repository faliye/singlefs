#!/usr/bin/env bash
# 54 号分档的四样自证（外加两样）：临时仓 + 打合成日志的假 cargo。每一步打原样输出与退出码。
set -uo pipefail
base=/tmp/claude-1000/gate54-tiering
repo="$base/selftest-repo"
export PATH="$base/fake-bin:$PATH"
unset SINGLEFS_LAYER0_THREADS SINGLEFS_STAGED_TREE GATE_BASE
stage=".claude/gate.d/54-layer0-replay.sh"
marker="$(git -C "$repo" rev-parse --path-format=absolute --git-common-dir)/singlefs-layer0-full-green"
run_step() { # <标题> <命令…>
  local title="$1"; shift
  echo "════ $title"
  echo "\$ $*"
  "$@"
  echo "[退出码 $?]"
  if [[ -f "$marker" ]]; then echo "[标记在：$(sed -n 's/^input_hash=//p' "$marker" | cut -c1-16)…，$(wc -l < "$marker") 行]"; else echo "[标记不在]"; fi
  echo
}
cd "$repo" || exit 2
run_step "准备：--full（假 cargo 打全绿的合成日志）写标记" bash "$stage" --full
echo "──── 标记全文 $marker"; cat "$marker"; echo
run_step "自证一：标记与输入相等 ⇒ 快档判绿" env SINGLEFS_GATE_FULL=1 bash "$stage"
cp -p crates/singlefs-harness/src/lib.rs "$base/lib.rs.before-byte-change"
printf 'pub fn marker() {}\n' | sed 's/marker/markes/' > crates/singlefs-harness/src/lib.rs
echo "改一个字节：cmp 原文件与改后："; cmp "$base/lib.rs.before-byte-change" crates/singlefs-harness/src/lib.rs; echo
run_step "自证二：改一个输入文件的一个字节 ⇒ 快档判红" env SINGLEFS_GATE_FULL=1 bash "$stage"
cp -p "$base/lib.rs.before-byte-change" crates/singlefs-harness/src/lib.rs
run_step "改回那个字节 ⇒ 快档又判绿（证明自证二红的就是那一个字节）" env SINGLEFS_GATE_FULL=1 bash "$stage"
rm -f -- "${marker:?}"
run_step "自证三：删掉标记 ⇒ 快档判红" env SINGLEFS_GATE_FULL=1 bash "$stage"
run_step "准备：--full 全绿，标记重新写上" bash "$stage" --full
run_step "自证四：--full 喂 LAYER0 下一行不是 CHECKER 的合成日志 ⇒ 判红、不写标记（旧标记开跑时已删）" env FAKE_CARGO_DROP_CHECKER=1 bash "$stage" --full
run_step "自证四之后：快档判红（没有标记）" env SINGLEFS_GATE_FULL=1 bash "$stage"
run_step "准备：--full 全绿，标记重新写上" bash "$stage" --full
run_step "自证四（另一支）：--full 里 cargo 判红 ⇒ 判红、不写标记" env FAKE_CARGO_FAIL=1 bash "$stage" --full
run_step "准备：--full 全绿，标记重新写上" bash "$stage" --full
run_step "外加：--full 跑到第二条流时改了一个输入 ⇒ 判红、不写标记" env FAKE_CARGO_TOUCH_DURING_SECOND="$repo/research/results/e142-sample.out" bash "$stage" --full
cp -p "$base/lib.rs.before-byte-change" crates/singlefs-harness/src/lib.rs
printf 'result\n' > research/results/e142-sample.out
echo "════ 外加：--staged 那条路（照 gate.sh --staged 的建法：worktree add HEAD + 套暂存区的 diff）"
printf 'pub fn staged_change() {}\n' > crates/singlefs-harness/src/new_module.rs
printf 'pub fn marker() {}\npub mod new_module;\n' > crates/singlefs-harness/src/lib.rs
git add crates/singlefs-harness/src/new_module.rs crates/singlefs-harness/src/lib.rs
git status --short
run_step "工作区里跑 --full（新文件已暂存，工作区与暂存区一致）" bash "$stage" --full
staged_worktree="$base/selftest-staged-worktree"
git diff --cached --binary > "$base/selftest-staged.patch"
git worktree add -q --detach "$staged_worktree" HEAD
git -C "$staged_worktree" apply --index "$base/selftest-staged.patch"
run_step "临时 worktree 里跑快档（读 common-dir 里同一份标记）⇒ 判绿" env SINGLEFS_GATE_FULL=1 bash "$staged_worktree/$stage" "$staged_worktree"
printf 'another session\n' > research/results/e999-other-session.out
run_step "工作区多了别的会话没暂存的一个输入文件，再跑 --full" bash "$stage" --full
run_step "临时 worktree 里跑快档 ⇒ 判红，列出那一个文件" env SINGLEFS_GATE_FULL=1 bash "$staged_worktree/$stage" "$staged_worktree"
git worktree remove --force "$staged_worktree"
echo "════ 参数：认不出的参数判红"
bash "$stage" --fast; echo "[退出码 $?]"
