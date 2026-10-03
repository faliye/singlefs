#!/usr/bin/env bash
# 用法：run-batch.sh <批号> <件表文件> <输出目录> <二进制>
# 件表一行一件：PROBE_MODE PROBE_SLOTS PROBE_OVERWRITES PROBE_UNITS PROBE_BATCHES [PROBE_ARM]
# 每件经内存包装（1G、线程 1）后台跑，退出码写 <输出目录>/b<批号>-<件号>.rc，输出写 b<批号>-<件号>.log；单独一个 wait 等齐。
set -u
batch="$1"; items="$2"; out="$3"; bin="$4"
repo=/home/fy5090/code/singlefs
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON='攻方腿与正推腿各自独立在各自副本上编译与跑原型'
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='峰值表里那一条是撞了 44M 上限被杀的一次，不是真峰值'
rm -f "$out"/b"$batch"-*.rc
index=0
while read -r mode slots overwrites units batches arm; do
  [ -z "$mode" ] && continue
  index=$((index + 1))
  {
    case "${arm:-today}" in
      jia) export SINGLEFS_ATTACK_ARM_JIA=1 ;;
      jia-b) export SINGLEFS_ATTACK_ARM_JIA=1 SINGLEFS_ATTACK_ARM_JIA_B=1 ;;
      jia-c) export SINGLEFS_ATTACK_ARM_JIA=1 SINGLEFS_ATTACK_ARM_JIA_B=1 SINGLEFS_ATTACK_ARM_JIA_C=1 ;;
    esac
    PROBE_MODE="$mode" PROBE_SLOTS="$slots" PROBE_OVERWRITES="$overwrites" PROBE_UNITS="$units" PROBE_BATCHES="$batches" PROBE_ARM="${arm:-today}" \
      nice -n 19 bash "$repo/research/scripts/run-with-memory-cap.sh" "${PROBE_CAP:-1G}" bash "$repo/research/scripts/capped.sh" 1 "$bin" --nocapture > "$out/b$batch-$index.log" 2>&1
    echo "$?" > "$out/b$batch-$index.rc"
  } &
done < "$items"
wait
count=$(ls "$out"/b"$batch"-*.rc 2>/dev/null | wc -l)
echo "batch $batch dispatched=$index rc-files=$count"
for i in $(seq 1 "$index"); do echo "b$batch-$i rc=$(cat "$out/b$batch-$i.rc" 2>/dev/null || echo missing)"; done
