#!/usr/bin/env bash
# C161（缓冲两级并存而衔接没定） 第三轮攻方腿：复跑全部模型。
# 用法：bash run_all.sh [并行数] [产物路径]（默认 8 路，产物写到本目录的 c161-r3-model-output.txt）。
# 每个作业的输出先落进一个临时目录；全部跑完、逐个核过「末行 emitted=N 且结果行恰好 N 条」，才拼成产物交出去；
# 任一作业不符就整轮作废，不写产物。
set -euo pipefail
jobs="${1:-8}"
here="$(cd "$(dirname "$0")" && pwd)"
output="${2:-$here/c161-r3-model-output.txt}"
parts="$(mktemp -d "${TMPDIR:-/tmp}/c161-r3-opus-parts.XXXXXX")"
cd "$here"
arms="jia_R4_T jia_R4_T2present jia_R4_foldup extent_tree_T1T2 extent_tree_T2present extent_tree_foldup bing_T6 yi_D_T yi_G_T yi_G_T+T1 yi_D_T2present yi_G_T2present yi_D_foldup"
mutations="M1_jia_split_keep M2_yiD_split_keep M3_yiG_split_keep M4_jia_rebuild_nodrain M5_yiD_rebuild_nodrain M6_yiG_rebuild_nodrain M7_bing_second_writer M8_yiG_counter_reset"
{
  for arm in $arms; do echo "A_bfs0_${arm} c161_r3_tree.py bfs ${arm} 7 0"; done
  for arm in $arms; do echo "A_bfs1_${arm} c161_r3_tree.py bfs ${arm} 6 1"; done
  for arm in $mutations; do echo "A_mut_${arm} c161_r3_tree.py bfs ${arm} 5 1"; done
  for arm in $arms; do echo "A_walk_${arm} c161_r3_tree.py walk ${arm} 40 1 5000"; done
  for index in 0 1 2 3 4 5 6 7; do echo "B_${index} c161_r3_switch.py 8 6 ${index}"; done
  echo "C c161_r3_mono.py 16"
  echo "E c161_r3_cost.py"
} > "$parts/joblist.txt"
PARTS="$parts" xargs -P "$jobs" -L 1 bash -c 'nice -n 19 python3 -B "$@" > "$PARTS/$0.txt"' < "$parts/joblist.txt"
assembled="$parts/assembled.txt"
: > "$assembled"
while read -r name _; do
  file="$parts/${name}.txt"
  last="$(tail -n 1 "$file")"
  count="$(grep -c '^part=' "$file" || true)"
  if [ "$last" != "emitted=${count}" ]; then
    echo "✗ ${name} 的末行是「${last}」，结果行 ${count} 条：这一轮作废，没有写产物" >&2
    echo "→ 看 ${file} 的报错，修好之后整轮重跑 bash run_all.sh" >&2
    exit 1
  fi
  grep '^part=' "$file" >> "$assembled"
done < "$parts/joblist.txt"
echo "emitted=$(grep -c '^part=' "$assembled")" >> "$assembled"
cp "$assembled" "$output"
rm -rf "${parts:?}"
tail -n 1 "$output"
