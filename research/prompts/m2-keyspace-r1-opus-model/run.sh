#!/usr/bin/env bash
# m2-keyspace-r1 云端攻方原型的驱动（副本上跑，不是入库装置）。用法：bash run.sh <副本根> <输出目录>
# 先把 keyspace_opus_model.rs 拷进 <副本根>/crates/singlefs-harness/tests/。线程上限 CAP（默认 4），经 research/scripts/capped.sh 落到每条命令上。
# 这一轮的原始输出是分几次、用不同线程数跑的（报告「复跑」一节列了）；计数与线程数无关。
set -uo pipefail
repo=${1:?副本根}
out=${2:?输出目录}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/tmp/claude-1000/m2-keyspace-opus/target}
cap=${CAP:-4}
capped=$(cd "$(dirname "$0")/../../.." && pwd)/research/scripts/capped.sh
cd "$repo" || exit 2
nice -n 19 bash "$capped" "$cap" cargo test --offline --release -p singlefs-harness --test keyspace_opus_model --no-run || exit 3
bin=$(nice -n 19 bash "$capped" "$cap" cargo test --offline --release -p singlefs-harness --test keyspace_opus_model --no-run --message-format=json 2>/dev/null \
  | python3 -c 'import sys,json
for l in sys.stdin:
    m=json.loads(l)
    if m.get("reason")=="compiler-artifact" and m.get("executable") and "keyspace_opus_model" in m["executable"]: print(m["executable"])' | tail -1)
run() {
  local tag=$1; local test=$2; shift 2
  { date -u +%FT%TZ; env "$@" nice -n 19 bash "$capped" "$cap" "$bin" "$test" --nocapture 2>&1 | grep -E 'SHAPE|SELF|KNOWN|FIXED|COST|SCAN|prefix|example|panicked|test result'; echo "exit=${PIPESTATUS[0]}"; date -u +%FT%TZ; } > "$out/$tag.out"
}
run selftest selftest
run cost cost_model KS_COST=abcd KS_THREADS=$cap
run scan scan_candidates KS_SCAN=s1,s2 KS_MAX_GROUPS=15 KS_THREADS=$cap
run scan-s4 scan_candidates KS_SCAN=s4 KS_MAX_GROUPS=15 KS_THREADS=$cap
