#!/usr/bin/env bash
# run-layer0.sh <线程数或 unset> <测试二进制> <用例名> <日志路径>
# 在副本里跑一条层 0 全量用例：记开跑 / 跑完的 UTC 时刻与 uptime、挂钟秒数、退出码。
set -uo pipefail
threads="$1"; test_binary="$2"; test_name="$3"; log="$4"
cd /tmp/claude-1000/m2-layer0-parallel/copy || exit 2
{
  echo "RUN_START $(date -u +%Y-%m-%dT%H:%M:%SZ) threads=$threads test=$test_binary::$test_name"
  echo "RUN_START_UPTIME $(uptime)"
} >"$log"
start_nanoseconds="$(date +%s%N)"
if [[ "$threads" == unset ]]; then
  env -u SINGLEFS_LAYER0_THREADS nice -n 19 cargo test --offline --release -p singlefs-harness --test "$test_binary" -- --include-ignored --exact "$test_name" --nocapture >>"$log" 2>&1
else
  SINGLEFS_LAYER0_THREADS="$threads" nice -n 19 cargo test --offline --release -p singlefs-harness --test "$test_binary" -- --include-ignored --exact "$test_name" --nocapture >>"$log" 2>&1
fi
exit_code=$?
end_nanoseconds="$(date +%s%N)"
{
  echo "RUN_EXIT $exit_code"
  echo "RUN_WALL_SECONDS $(awk -v start="$start_nanoseconds" -v end="$end_nanoseconds" 'BEGIN { printf "%.2f", (end - start) / 1e9 }')"
  echo "RUN_END $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "RUN_END_UPTIME $(uptime)"
} >>"$log"
exit "$exit_code"
