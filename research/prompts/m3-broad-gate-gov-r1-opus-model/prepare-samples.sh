#!/usr/bin/env bash
# 攻方腿模型（m3-broad-gate-gov-r1，G5）：把门禁 checker-tier-broad-mutation.sh 的 12 份样本拷进 <草稿目录>/run/ 并跑各自的 setup.sh（照 stage-selftest.sh 的做法），
# 再拷一份 <草稿目录>/run-fixed/，把 6 份 library-version 样本的库目录指纹改成今天驱动 version_key 的公式（fix-green.py）；
# 最后写出逐份样本起阶段的命令文件（字面路径，重型闸「门禁样本」② 认得：cd 进样本根、根参数写同一个目录）：
#   gen-base-cmds.txt、gen-break-cmds.txt（8 个弄坏开关）、gen-swallow-cmds.txt（STAGE_CELLS_BREAK=red-swallowed），run-fixed 的三份前缀 gen-fixed-。
# 跑：bash <草稿目录>/gen-*.txt；判：python3 judge.py <tree>/.claude/gate.d/fixtures/checker-tier-broad-mutation.sh <草稿目录>/out <前缀>
# 用法：bash prepare-samples.sh <冻结副本的拷贝> <草稿目录>
set -euo pipefail
tree="$1"; scratch="$2"; model="$(cd "$(dirname "$0")" && pwd)"
fixtures="$tree/.claude/gate.d/fixtures/checker-tier-broad-mutation.sh"
stage="$tree/.claude/gate.d/checker-tier-broad-mutation.sh"
rm -rf "$scratch/run" "$scratch/run-fixed"; mkdir -p "$scratch/run" "$scratch/out"
for sample in "$fixtures"/*/; do
  name="$(basename "$sample")"
  cp -a "$sample" "$scratch/run/$name"
  if [[ -f "$scratch/run/$name/setup.sh" ]]; then (cd "$scratch/run/$name" && env -u GATE_BASE -u GATE_STAGED_FROM -u GATE_DIFF_BASE bash setup.sh >/dev/null 2>&1); fi
done
cp -a "$scratch/run" "$scratch/run-fixed"
for root in "$scratch"/run-fixed/library-version-is-staged-tree-*/; do python3 "$model/fix-green.py" "$tree" "${root%/}"; done
emit() {   # $1 run 目录名  $2 输出前缀  $3 环境前缀（空、或 VAR=值）
  local name root out
  for root in "$scratch/$1"/*/; do
    name="$(basename "$root")"; root="${root%/}"; out="$scratch/out/$2-$name.txt"
    printf '( cd %s && %sbash %s %s --force > %s 2>&1; echo "exit=$?" >> %s )\n' "$root" "${3:+$3 }" "$stage" "$root" "$out" "$out"
  done
}
for kind in run run-fixed; do
  prefix=""; [[ "$kind" == run-fixed ]] && prefix="fixed-"
  emit "$kind" "${prefix}base" "" > "$scratch/gen-${prefix}base-cmds.txt"
  : > "$scratch/gen-${prefix}break-cmds.txt"
  for item in work-tree-read files-unchecked toolchain-unchecked export-range-unchecked summary-red-passes counts-unchecked gate-line-optional version-unchecked; do
    emit "$kind" "${prefix}brk-$item" "CHECKER_TIER_BROAD_MUTATION_BREAK=$item" >> "$scratch/gen-${prefix}break-cmds.txt"
  done
  emit "$kind" "${prefix}brk-red-swallowed" "STAGE_CELLS_BREAK=red-swallowed" > "$scratch/gen-${prefix}swallow-cmds.txt"
done
wc -l "$scratch"/gen-*cmds.txt
