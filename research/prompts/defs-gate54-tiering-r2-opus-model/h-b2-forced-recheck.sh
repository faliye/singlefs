#!/usr/bin/env bash
# H-B2（V2，工具链）：不退回、不并发。提交 A 的全量在工具链 1 下绿、写格；之后工具链升到 2（rustup update 一类，不在仓里）；
# 干净工作树上照 change-touches-crates.sh 第 27 行给的办法「SINGLEFS_GATE_FULL=1 … 在干净工作树上重新验一遍 HEAD」起门禁。
# 放开扫的用户动作：重新验的时候带不带 --staged（带：照 gate.sh --staged 的建法；不带：在主工作区直接跑 54 号）。
source "$(dirname "$0")/lib.sh"
outpath_file "$MODEL/inputs/54-post.sh" "$RUNS/outpath.sh"
for arm in post pre; do
  want "$arm-forced" || continue
  r="$RUNS/h-b2-$arm"
  echo "════ H-B2 [$arm] 工具链 1 下提交 A → 工具链升到 2 → SINGLEFS_GATE_FULL=1 重新验 HEAD"
  echo 1 > "$FAKE_TOOLCHAIN_FILE"
  make_repo "$r" "$arm" new ""
  set_crash "$r" c0 TOOLCHAIN_SENSITIVE; git -C "$r" add -A
  if [[ "$arm" == post ]]; then (cd "$r" && bash "$RUNS/outpath.sh" > "$r.a-full.log" 2>&1); fi
  echo "  ── 提交 A 之前的门禁（工具链 1）"; gate_staged_54 "$r"
  git -C "$r" commit -q -m A
  echo 2 > "$FAKE_TOOLCHAIN_FILE"
  echo "  ── 工具链升到 2；工作树干净：$(git -C "$r" status --porcelain | wc -l) 行 status"
  cells "$r"
  echo "  ── SINGLEFS_GATE_FULL=1，带 --staged 的建法"; SINGLEFS_GATE_FULL=1 gate_staged_54 "$r"
  echo "  ── SINGLEFS_GATE_FULL=1，主工作区直接跑 54 号"
  (cd "$r" && SINGLEFS_GATE_FULL=1 bash "$STAGE" > "$r.direct.out" 2>&1); echo "    [退出码 $?]"; grep -E '^  (✓|✗|!) ' "$r.direct.out" | trim 150
  echo "  ── 真值（工具链 2 下这份 HEAD 用它自己的 54 号跑全量）"; truth_full "$r"
  echo
done
