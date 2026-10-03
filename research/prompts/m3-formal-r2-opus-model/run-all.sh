#!/bin/bash
# 攻方腿复跑入口：light 档合法历史一族 12 个开关组合 + 对抗一族原实现一遍。参数：输出目录。每份只留 E7RESULT 行（E7TIMING 是挂钟，不进比对）。
set -u
out=$1
bin=${E165_ATTACK_BIN:-/tmp/claude-1000/m3-formal-r2-opus/target/release/e165_replay_bounded_enumeration}
mkdir -p "$out"
: > "$out/exits.txt"
for spec in "m0:0:0" "m0row:0:1" "m1:1:0" "m2:2:0" "m3:3:0" "m4:4:0" "m5:5:0" "m6:6:0" "m9:9:0" "m10:10:0" "m5row:5:1" "m6row:6:1"; do
  name=${spec%%:*}; rest=${spec#*:}; mutant=${rest%%:*}; row=${rest#*:}
  SINGLEFS_ATTACK_MUTANT=$mutant ATTACK_ROW_ROOT=$row E165_THREADS=10 nice -n 19 "$bin" --bounds light --family legal > "$out/$name.raw" 2> "$out/$name.err"
  echo "$name mutant=$mutant row_root=$row exit=$?" >> "$out/exits.txt"
  grep '^E7RESULT' "$out/$name.raw" > "$out/$name.out"; rm -f "$out/$name.raw"
done
SINGLEFS_ATTACK_MUTANT=0 E165_THREADS=10 nice -n 19 "$bin" --bounds light --family adversarial > "$out/m0-adv.raw" 2> "$out/m0-adv.err"
echo "m0-adv mutant=0 family=adversarial exit=$?" >> "$out/exits.txt"
grep '^E7RESULT' "$out/m0-adv.raw" > "$out/m0-adv.out"; rm -f "$out/m0-adv.raw"
