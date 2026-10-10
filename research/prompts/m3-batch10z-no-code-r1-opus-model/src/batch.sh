#!/usr/bin/env bash
# m3-batch10z-no-code-r1 攻方：一批并行跑探针。用法：bash batch.sh <草稿目录> <批名> <件表>
# 件表一行一件：「today|<环境变量…>」或「pre14|<环境变量…>」，# 开头与空行不算。每件单线程、各经内存包装 1500M；
# 退出码写进 <草稿目录>/runs/<批名>-<件号>.rc，最后数一遍件数对不对。
set -u
W=$1; b=$2; jobs=$3
ROOT=${ROOT:-/home/fy5090/code/singlefs}
CAP=$ROOT/research/scripts/run-with-memory-cap.sh
CAPPED=$ROOT/research/scripts/capped.sh
BT=$(ls -t "$W"/target/release/deps/opus_no_probe-* | grep -v '\.d$' | head -1)
BP=$(ls -t "$W"/target-pre14/release/deps/opus_no_probe-* | grep -v '\.d$' | head -1)
mkdir -p "$W/runs"
rm -f "$W"/runs/"$b"-*.rc
n=0
while IFS='|' read -r arm envs; do
  [[ -z "$arm" || "$arm" == \#* ]] && continue
  n=$((n+1)); bin=$BT; [[ $arm == pre14 ]] && bin=$BP
  # shellcheck disable=SC2086
  { nice -n 19 bash "$CAP" 1500M env $envs S3_LABEL="$b-$n" ARM_TREE="$arm" bash "$CAPPED" 1 "$bin" --nocapture s3_sweep > "$W/runs/$b-$n.log" 2>&1; echo "$?" > "$W/runs/$b-$n.rc"; } &
done < "$jobs"
wait
got=$(ls "$W"/runs/"$b"-*.rc 2>/dev/null | wc -l)
if [[ "$got" -ne "$n" ]]; then echo "批 $b 件数对不上：派 $n 收 $got，整批作废"; exit 4; fi
for i in $(seq 1 "$n"); do echo "$b-$i rc=$(cat "$W/runs/$b-$i.rc") $(grep -h S3_DONE "$W/runs/$b-$i.log")"; done
