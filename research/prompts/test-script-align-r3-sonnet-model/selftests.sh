#!/usr/bin/env bash
# V3 复现：各件交回里报的自证数。登记为 light 的 --selftest，在本仓路径上、顺序跑（mutate.sh 自证并行会互相干扰）；
# 经 capped.sh 8 起，输出写 <草稿目录>/s-<名>.txt，一行汇总写 <草稿目录>/s-summary.txt。用法：selftests.sh <草稿目录>
S="${1:?用法：selftests.sh <草稿目录>}"; mkdir -p "$S"
cd "$(git rev-parse --show-toplevel)" || exit 2
run() { local name=$1; shift; local t0=$SECONDS; nice -n 19 bash scripts/capped.sh 8 "$@" > "$S/s-$name.txt" 2>&1; echo "$name exit=$? secs=$((SECONDS-t0))" >> "$S/s-summary.txt"; }
: > "$S/s-summary.txt"
run lib python3 .claude/hooks/lib_heavy_tests.py --selftest
run guard bash .claude/hooks/heavy-test-guard.sh --selftest
run hookswap python3 scripts/hook-swap.py --selftest
run caentry bash scripts/crash-amplification.sh --selftest
run caitem bash scripts/crash-amplification-item.sh --selftest
run mutate bash scripts/mutate.sh --selftest
run admission python3 scripts/admission.py --selftest
run stagelist bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh "$PWD/.claude/gate.d" --list-items
run stage54 bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh "$PWD/.claude/gate.d" --item checker-tier-layer0-replay.sh
echo DONE >> "$S/s-summary.txt"
