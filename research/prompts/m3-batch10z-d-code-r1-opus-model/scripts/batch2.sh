#!/usr/bin/env bash
# 同 batch.sh；规格行的环境变量串里 OPUS_BIN=<测试二进制名> 点名跑哪一份（不写就用 scripts/binary-name）。
set -u
W=$HOME/.local/share/singlefs/work/m3-batch10z-d-code-r1-opus
S=/home/fy5090/code/singlefs/research/scripts
spec=$1; batch=$2; from=$3; to=$4
cd "$W/repo"
rm -f "$W"/runs/b"$batch"-*.rc
n=0
while IFS=$'\t' read -r name envs; do
  n=$((n+1))
  if (( n < from || n > to )); then continue; fi
  bin=$(printf '%s\n' $envs | sed -n 's/^OPUS_BIN=//p')
  [[ -n "$bin" ]] || bin=$(cat "$W/scripts/binary-name")
  { env $envs nice -n 19 bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 1 "$W/target/release/deps/$bin" --nocapture opus_sweep > "$W/runs/$name.log" 2>&1; echo "$?" > "$W/runs/b$batch-$n.rc"; } &
done < "$spec"
wait
expected=$(( to - from + 1 ))
got=$(ls "$W"/runs/b"$batch"-*.rc 2>/dev/null | wc -l)
echo "batch=$batch expected=$expected rc_files=$got"
for i in $(seq "$from" "$to"); do printf '%s ' "$i:$(cat "$W/runs/b$batch-$i.rc" 2>/dev/null || echo missing)"; done; echo
