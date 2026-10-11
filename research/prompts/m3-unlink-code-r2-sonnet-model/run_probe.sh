#!/usr/bin/env bash
# 用法：run_probe.sh <探针号> <相对文件> <测试二进制> [测试名过滤…]
# 在草稿副本里按 probes/<号>.orig → .repl 改一处，跑指定 harness 档测试二进制，打结果行，再还原。
# 环境：S=草稿目录；经 run-with-memory-cap.sh 与 capped.sh 6 跑；只读冻结副本。
set -u
S=/tmp/claude-1000/m3-unlink-r2-forward
FROZEN=/tmp/claude-1000/m3-unlink-code-r2/tree
HERE=$(cd "$(dirname "$0")" && pwd)
probe=$1; rel=$2; bin=$3; shift 3
python3 "$HERE/probe.py" apply "$S/tree" "$rel" "$HERE/probes/$probe.orig" "$HERE/probes/$probe.repl" || exit 3
cd "$S/tree"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="正推腿 m3-unlink-r2-forward 与别的派出去的 agent 各跑各的，只跑 harness 档单个测试二进制"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS=150
CARGO_TARGET_DIR=$S/target nice -n 19 bash /home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh "${CAP:-4G}" \
  bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh 6 \
  cargo test --offline -p singlefs-harness --test "$bin" -- "$@" > "$S/probe-$probe-$bin.log" 2>&1
rc=$?
python3 "$HERE/probe.py" restore "$S/tree" "$FROZEN" "$rel"
echo "probe=$probe bin=$bin rc=$rc"
grep -E '^test result|^test .* FAILED|panicked' "$S/probe-$probe-$bin.log" | head -8
