#!/usr/bin/env bash
# J2「已有比重的指纹钉住了七组」：拿 git HEAD 的 crates/（没有 26-1r 的新步）在草稿目录里编，把冻结副本里的指纹用例去掉 orphans 一组
# （HEAD 上没有 LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP）之后跑，六个钉值应当原样复现。
# 用法：WORK=<草稿目录> REPO=<仓根> FROZEN=<冻结副本> bash fingerprint_at_head.sh
set -euo pipefail
REPO="${REPO:-/home/fy5090/code/singlefs}"; FROZEN="${FROZEN:-/tmp/claude-1000/m3-formal-code-b26-r1/tree}"; WORK="${WORK:?}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-草稿副本里单跑取宽上限}"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-同一会话下几条腿各跑各的}"
rm -rf "$WORK/work-head"; mkdir -p "$WORK/work-head"
git -C "$REPO" archive HEAD crates Cargo.toml Cargo.lock | tar -x -C "$WORK/work-head"
T=crates/singlefs-harness/tests/random_history_generation_keeps_the_existing_weights_step_for_step.rs
python3 - "$FROZEN/$T" "$WORK/work-head/$T" <<'PY'
import sys
s = open(sys.argv[1]).read()
s = s.replace('        GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP,\n', '')
s = s.replace('            ("orphans", 10_448_260_464_455_344_558),\n', '')
open(sys.argv[2], 'w').write(s)
PY
nice -n 19 bash "$REPO/research/scripts/capped.sh" 7 timeout 235 bash "$REPO/research/scripts/run-with-memory-cap.sh" 12G \
    cargo test --offline --manifest-path "$WORK/work-head/Cargo.toml" -p singlefs-harness \
    --test random_history_generation_keeps_the_existing_weights_step_for_step 2>&1 | grep -E 'test result|Running'
