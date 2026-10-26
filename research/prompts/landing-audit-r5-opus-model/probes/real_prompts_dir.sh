#!/usr/bin/env bash
# K2 ⑳：派发闸自证读的 14 份真提示，目录不在、在、改了一个字节三种情况下自证退几、报什么。只在 <树> 的拷贝里放、改，跑完删掉拷进去的那份目录。
# 用法：bash real_prompts_dir.sh <树（有 .claude/hooks 等，rerun.sh 建的 tree-cur）> <仓根（只读：拷那 14 份真提示）>
set -uo pipefail
tree="$1"; repo="$2"; prompts=research/prompts/landing-audit-r3-opus-model/prompts
run() {   # run <情形>
  local out rc
  out="$(cd "$tree" && bash .claude/hooks/runner-dispatch-guard.sh --selftest 2>&1)"; rc=$?
  echo "$1：自证退 $rc；$(grep -o '自检通过（查了 [0-9]* 种）' <<< "$out" | head -1)；本次未跑行 $(grep -c '本次未跑：⑳' <<< "$out")；✗ 行 $(grep -c '✗ 自检' <<< "$out")"
  grep '✗ 自检' <<< "$out" | cut -c1-140
}
rm -rf "${tree:?}/research"
run "目录不在"
mkdir -p "$tree/$prompts" && cp -a "$repo/$prompts/." "$tree/$prompts/"
run "目录在、14 份原样"
printf ' ' >> "$tree/$prompts/a9717cf07fb6e7c21.md"
run "目录在、a9717cf07fb6e7c21.md 末尾多一个空格"
rm -rf "${tree:?}/research"
