#!/usr/bin/env bash
# 要主 agent 跑（子 agent 起驱动拷贝的 --selftest 被重型闸按「提交时才跑的检查」拒）：
#   SINGLEFS_HEAVY_TESTS=user-request nice -n 19 bash driver-selftest-mutants.sh <冻结副本树根> <草稿目录>
# 在冻结副本的硬链接拷贝里逐个换上 h2-mutants.py 造的驱动写错版，各跑一次驱动 --selftest，打末尾那一行（「--selftest：N 格里 M 格判错」或「N 格都对」）。
# 判读：一个写错版这里「都对」、h2-sweep.txt 里 26 份样本也「判错 0 个」，就是谁都不盯它。
set -uo pipefail
frozen="$1"; work="$2"; here="$(cd "$(dirname "$0")" && pwd)"
base="$work/selftest-trees/base"; rm -rf "${work:?}/selftest-trees"; mkdir -p "$work/selftest-trees/h2"
rsync -a --exclude target --exclude .git "$frozen/" "$base/"
python3 "$here/h2-mutants.py" "$base" "$work/selftest-trees/h2" > "$work/selftest-trees/mutants.list"
for tree in "$base" $(cut -f1 "$work/selftest-trees/mutants.list" | sed "s|^|$work/selftest-trees/h2/|"); do
  name="$(basename "$tree")"
  last="$(cd "$tree" && python3 research/scripts/broad-mutation.py --selftest 2>&1 | grep -E -- '--selftest：' | tail -1)"
  printf '%s\t%s\n' "$name" "${last:-（没打出汇总行）}"
done
