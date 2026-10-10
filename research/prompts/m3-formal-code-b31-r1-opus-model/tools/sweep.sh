#!/bin/bash
# 场景乙（build_pool 前缀之后用户动作放开扫）并行跑一批。用法：sweep.sh <草稿目录> <批号> <起始种子> <每件种子数> <件数> <步数>
# 测试二进制取 <草稿目录>/mutants/baseline/build.json 里那一份（先跑 mutant-run.sh baseline）；每件退出码写 <草稿目录>/rc/b<批号>-<件号>.rc。
set -u
D="$1"; batch="$2"; base="$3"; per="$4"; pieces="$5"; steps="$6"
B=$(python3 -c '
import json,sys
for line in open(sys.argv[1]):
    try: m=json.loads(line)
    except Exception: continue
    if m.get("reason")=="compiler-artifact" and m.get("executable") and m["target"]["name"]=="b31_opus_n3_reclaim_predicate_and_ring_table": print(m["executable"])
' "$D/mutants/baseline/build.json")
mkdir -p "$D/rc" "$D/sweep"; rm -f "$D/rc/b$batch"-*.rc
cd "$D/copy/crates/singlefs-harness"
for piece in $(seq 1 "$pieces"); do
  start=$((base + (piece - 1) * per))
  { B31_SEED_BASE=$start B31_SEEDS=$per B31_STEPS=$steps "$B" --exact scenario_b_user_actions_swept_after_the_build_pool_prefix --nocapture > "$D/sweep/$batch-$piece.log" 2>&1; echo "$?" > "$D/rc/b$batch-$piece.rc"; } &
done
wait
count=$(ls "$D/rc/b$batch"-*.rc | wc -l)
echo "batch $batch rcfiles=$count pieces=$pieces"
[[ $count -eq $pieces ]] || exit 1
for piece in $(seq 1 "$pieces"); do echo "piece $piece rc=$(cat "$D/rc/b$batch-$piece.rc")"; grep -h '^name=' "$D/sweep/$batch-$piece.log"; done
