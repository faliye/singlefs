#!/usr/bin/env bash
# V2：登记的判法入口里，哪些的闭包经内存包装、哪些拒算。用法：entries_scan.sh <冻结副本> <入口清单 tools/entries.txt>
# 一行一个入口（<文件>::<限定名>）；每个入口在副本根上收一次闭包（默认扫描范围），报：退出码、闭包里内存包装的单元数、host-resources.py 的单元数、拒算形态计数。
# 入口清单来自：scripts/mutation/broad-mutation.py 的 JUDGING_ENTRIES 与 GPU_JUDGING_ENTRIES、scripts/mutation/crates-mutation-rows.py 的 JUDGING_ENTRIES、
# scripts/experiment-apparatus/replay.sh 的几个判法函数（scripts/gate/admission.py 的 REPLAY_JUDGING_FUNCTIONS 取三个）、e165-pin-red-examples.py 的 replay、verus-proof-check.py 的三个入口。
# out/entries-scan-current.out 是同一段循环（逐字相同的命令）在冻结副本上跑出的原样输出；这份脚本本身这一轮没再整份重跑。
set -u
repo="$1"; list="$2"
cd "$repo" || exit 2
while read -r entry; do
  out="$(timeout 300 nice -n 19 python3 scripts/gate/lib_function_units.py closure . "$entry" 2>&1)"; rc=$?
  wrapper="$(printf '%s\n' "$out" | awk -F'\t' '$3 ~ /^scripts\/hosts\/run-with-memory-cap.sh/' | wc -l)"
  resources="$(printf '%s\n' "$out" | awk -F'\t' '$3 ~ /^scripts\/hosts\/host-resources.py/' | wc -l)"
  refused="$(printf '%s\n' "$out" | awk -F'\t' '$1=="refused"{print $2}' | sort | uniq -c | tr '\n' ';')"
  echo "rc=$rc wrapper_units=$wrapper host_resources_units=$resources refused=[$refused] $entry"
done < "$list"
