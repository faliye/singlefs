#!/usr/bin/env bash
# probe_g 单独复跑（登记驱动那一份在卡上空闲不足一份驱动开销时还登记得上）。经 run-with-memory-cap.sh 跑。
set -uo pipefail
S="$HOME/.local/share/singlefs/work/m3-batch10t-code-r1-sonnet"
export TMPDIR="$S/tmp" CARGO_TARGET_DIR="$S/target"
cd "$S/repo" || exit 2
nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/capped.sh 8 cargo test --offline -p singlefs-harness --test k_forward_ledger_probe -- probe_g --nocapture 2>&1 | grep -E 'PROBE|test result|FAILED|panicked at' | cut -c1-300
