#!/usr/bin/env bash
# Y3：全仓 grep「可写挂载会拒」「单盘全废」「两槽全废」与同义说法，按区域数命中行数与文件数，再列 kb 与 crates 非测试文件里的命中行。
# 用法：y3_greps.sh <仓根>。读的是仓此刻的样子（别的会话在改），所以这份输出只当线索、不进「同一次复跑」的逐字比对。
set -uo pipefail
repo="${1:?用法：y3_greps.sh <仓根>}"
cd "$repo"
patterns=(
  '可写挂载会拒'
  '单盘全废'
  '两槽全废'
  '两槽都自证不过'
  '两槽都废'
  '少于两槽自证'
  '一份自证过的系统配置都没有'
  '一份本池自证过的系统配置都没有'
  '没有自证过的槽'
)
areas=(.claude/kb crates scripts records research/prompts)
for pattern in "${patterns[@]}"; do
  echo "== 命令：grep -rnI --exclude-dir=target --exclude-dir=.git -- '$pattern' <区域>"
  for area in "${areas[@]}"; do
    lines=$(grep -rnI --exclude-dir=target --exclude-dir=.git -- "$pattern" "$area" 2>/dev/null | wc -l || true)
    files=$(grep -rlI --exclude-dir=target --exclude-dir=.git -- "$pattern" "$area" 2>/dev/null | wc -l || true)
    echo "  $area lines=$lines files=$files"
  done
done
echo "== kb 与 crates 里非测试、非变异表文件的命中行（文件:行，前 110 字）"
for pattern in '可写挂载会拒' '单盘全废' '两槽全废' '少于两槽自证'; do
  grep -rnI --exclude-dir=target --exclude-dir=.git --exclude-dir=tests --exclude=mutations.tsv -- "$pattern" .claude/kb crates 2>/dev/null | cut -c1-110 | sed "s/^/  [$pattern] /"
done
