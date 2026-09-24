#!/usr/bin/env bash
# 改法各修哪一格：拿打了改法的一份 54 号（model/fixes/54-<改法>.sh）当分档之后那一臂，只重跑打中的格与两格对照。
#   g1   判这一格的 54 号自己（脚本的 sha256）进输入哈希
#   g2   工具链版本（cargo -V）进输入哈希
#   g4   「跑的过程中输入变了」的红不删开跑那一格
#   g124 三样一起
# 全部是攻方提的，只在这个模型上量过、被攻过零轮。
set -uo pipefail
MODEL="$(cd "$(dirname "$0")" && pwd)"
BASE="${RUNS_BASE:-/tmp/claude-1000/defs54-r2-attack/runs}"
for fix in ${FIXES:-g1 g2 g4 g124}; do
  export POST_54_OVERRIDE="$MODEL/fixes/54-$fix.sh" RUNS="$BASE/fix-$fix"
  rm -rf "$RUNS"; mkdir -p "$RUNS"
  {
    echo "######## 改法 $fix（$(sha256sum < "$POST_54_OVERRIDE" | cut -c1-16)…）"
    CASES="post-T1-SINGLE_THREADED-54 post-T2-SINGLE_THREADED-54 post-T3-SINGLE_THREADED-54 post-T1-multi-54" bash "$MODEL/h-a-late-stage-54.sh"
    CASES="post-54-during post-54-kill9 post-toolchain-during post-toolchain-kill9 post-54-finish" bash "$MODEL/h-b-revert-stale-cell.sh"
    CASES="post-forced" bash "$MODEL/h-b2-forced-recheck.sh"
    CASES="post-wip-Zlast post-wip-overlapZlast post-edit-Zlast post-edit-overlapZlast" bash "$MODEL/h-c-foreign-run-deletes.sh"
  } > "$MODEL/outputs/fix-arms-$fix.out" 2>&1
  echo "改法 $fix 跑完：$MODEL/outputs/fix-arms-$fix.out"
done
