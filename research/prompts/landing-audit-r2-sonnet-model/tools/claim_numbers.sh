#!/usr/bin/env bash
# 用法：claim_numbers.sh <仓根> <输出目录>
# 复核 K2-8 不采：占号脚本 scripts/kb/claim-experiment.sh 的 used_numbers_in_tree 扫的目录里有没有 crates/singlefs-checker-tier/src/bin 下入库装置的号。
# 输出 claim-vs-bin.txt：bin 下有而占号目录表里没有的号（空即没有），以及 --next 的读数。
set -euo pipefail
root="$1" out="$2"
mkdir -p "$out"
ls "$root/crates/singlefs-checker-tier/src/bin/" | sed -n 's/^e\([0-9][0-9]*\)_.*/\1/p' | sort -u > "$out/bin-numbers.txt"
# shellcheck disable=SC1090
source <(sed -n '/^used_numbers_in_tree()/,/^}/p' "$root/scripts/kb/claim-experiment.sh")
used_numbers_in_tree "$root" 2>/dev/null | sort -u > "$out/claim-numbers.txt" || true
{
  echo "bin 下的号个数 $(wc -l < "$out/bin-numbers.txt")，占号目录表里的号个数 $(wc -l < "$out/claim-numbers.txt")"
  echo "bin 下有、占号目录表里没有的号："
  comm -23 "$out/bin-numbers.txt" "$out/claim-numbers.txt" | tr '\n' ' '
  echo
  echo "（上一行为空即没有）"
  (cd "$root" && bash scripts/kb/claim-experiment.sh --next)
} > "$out/claim-vs-bin.txt"
cat "$out/claim-vs-bin.txt"
