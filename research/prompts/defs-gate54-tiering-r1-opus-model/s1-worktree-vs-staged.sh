#!/usr/bin/env bash
# S1　工作区 ≠ 暂存区：主 agent 照 main-agent.md 第 43 行在「暂存之前」于主工作区跑 --full；别的会话 B 在 crates/ 下有
# 没暂存的改动与未跟踪文件。量两件事：①这份标记在 gate.sh --staged 下对不对得上；②出路句那条路走不走得通。
# 放开扫的由人决定的几步：B 的东西是（未跟踪 / 已跟踪没暂存 / 两样都有）；出路 worktree 在（暂存之前 / 暂存之后）建。
source "$(dirname "$0")/lib.sh"
base=/tmp/claude-1000/defs54-attack/s1
for foreign in untracked modified both; do
  for outlet_timing in after-staging before-staging; do
    repo="$base/repo-$foreign-$outlet_timing"; build_repo "$repo" post; cd "$repo" || exit 2
    echo "════ S1 [B 的东西：$foreign｜出路 worktree：$outlet_timing]"
    # A 这一批：新模块 + lib.rs 一行
    printf 'pub fn batch_a() {}\n' > crates/singlefs-harness/src/batch_a.rs; printf 'pub mod crash;\npub mod batch_a;\n' > crates/singlefs-harness/src/lib.rs
    # B 的半成品（不属于这一批，照 session-wrapup.md 第 4 节不许卷进提交）
    [[ "$foreign" != modified ]] && printf 'pub fn wip() {}\n' > crates/singlefs-core/src/b_wip.rs
    [[ "$foreign" != untracked ]] && printf 'pub fn core() { /* B 改到一半 */ }\n' > crates/singlefs-core/src/lib.rs
    git status --porcelain
    run "① 收尾：主工作区里 --full（main-agent.md 第 43 行，暂存之前）" bash "$STAGE" --full | tail -2; show_marker "$repo"
    if [[ "$outlet_timing" == before-staging ]]; then
      # 暂存之前建「只含这一批」的 worktree：HEAD + 暂存区此刻是空的，只能得到 HEAD
      build_batch_worktree "$repo" "$base/outlet-$foreign-$outlet_timing"
      run "② 出路（暂存之前建 worktree）：--full <它的根>" bash "$base/outlet-$foreign-$outlet_timing/$STAGE" --full "$base/outlet-$foreign-$outlet_timing" | tail -2; show_marker "$repo"
      gitq add crates/singlefs-harness/src/batch_a.rs crates/singlefs-harness/src/lib.rs
    else
      gitq add crates/singlefs-harness/src/batch_a.rs crates/singlefs-harness/src/lib.rs
      echo "──── ③ 暂存这一批之后、出路之前：gate.sh --staged"; gate_staged_54 "$repo" 2>&1 | grep -E '✗|✓|只在|内容不同|退出码'
      build_batch_worktree "$repo" "$base/outlet-$foreign-$outlet_timing"
      run "② 出路（暂存之后建 worktree）：--full <它的根>" bash "$base/outlet-$foreign-$outlet_timing/$STAGE" --full "$base/outlet-$foreign-$outlet_timing" | tail -2; show_marker "$repo"
    fi
    echo "──── ④ gate.sh --staged（HEAD + 暂存区）"; gate_staged_54 "$repo" 2>&1 | grep -E '✗|✓|只在|内容不同|退出码'
    gitq -C "$repo" worktree remove --force "$base/outlet-$foreign-$outlet_timing"
    echo
  done
done
