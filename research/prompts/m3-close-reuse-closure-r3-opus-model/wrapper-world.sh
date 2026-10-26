#!/usr/bin/env bash
# 树里那一份真内存包装起一条只建标记文件的命令（这一趟自己的 slice 与状态目录、查机器关着），报包装退几、命令跑没跑、stderr 里第一处 Error。
# 两种世界：
#   start    上限 64M、slice 总上限 1G：放得下、直接起跑（看包装自己 import 的那几份载入时抛不抛）
#   predict  上限 10G、slice 总上限 9G、峰值表里没有这条：predicted_bytes 按路径 exec_module 整份 host-dispatch.py 预测得出就按预测记账、起跑；
#            预测算不出就按上限 10G 记账、比总上限大、当场退 252（第一轮攻方 V2 那一种世界，这里重写一份）
# 用法：wrapper-world.sh <树> <名> <世界> <输出目录>
set -u
root="$1" name="$2" world="$3" out="$4/world-$world-$name"
rm -rf -- "$out"; mkdir -p "$out/state/ledger"
slice="singlefs_opusattack_r3_${world}_$$_${RANDOM}.slice"
case "$world" in
  start) cap=64M total=1G ;;
  predict) cap=10G total=9G ;;
  *) echo "世界写错：$world"; exit 2 ;;
esac
env -u SINGLEFS_HEAVY_TESTS -u CLAUDE_CODE_SESSION_ID -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP \
  RUN_WITH_MEMORY_CAP_SLICE="$slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL="$total" RUN_WITH_MEMORY_CAP_STATE_DIR="$out/state" \
  RUN_WITH_MEMORY_CAP_PEAKS="$out/peaks.tsv" SINGLEFS_CORE_USAGE_ALERTS="$out/alerts.jsonl" RUN_WITH_MEMORY_CAP_HOST_LOOKUP=off \
  RUN_WITH_MEMORY_CAP_KEY="attack-r3-$world-$name" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=5 \
  bash "$root/scripts/hosts/run-with-memory-cap.sh" "$cap" bash -c ': > "$1"' touch-marker "$out/ran" > "$out/stdout" 2> "$out/stderr"
status=$?
echo "name=wrapper_world world=$world tree=$name status=$status ran=$([[ -e $out/ran ]] && echo yes || echo no) first_error=$(grep -m1 -oE '[A-Za-z]+Error: [^）]{0,90}' "$out/stderr")"
systemctl --user stop "$slice" >/dev/null 2>&1; systemctl --user reset-failed "$slice" >/dev/null 2>&1
exit 0
