#!/usr/bin/env bash
# defs-m2-closeout-r3 云端攻方 K2 / G2 × 40 号：执行员照第 4 步把重跑的输出写成新文件名（「带今天的日期或 rN」），
# 页不改写时 40 号第一道（产物有没有被实验索引或实验页点名）对四种新文件名各判什么、给什么出路。
# 在临时目录里搭副本：40 号阶段、实验索引与实验页原样拷，research/results/ 下每个 *.out 只按同名建空文件（第一道只看文件名），
# 另拷 research/scripts/changed-paths.sh。副本不是 git 仓，第三道会把归档了的点名全报成找不到——那一道不看，只取第一道那一段。
# 用法：bash g2-results-cited-demo.sh [临时目录]
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
scratch="$(mktemp -d "${1:-${TMPDIR:-/tmp}}/g2-40-demo.XXXXXX")"
copy="$scratch/copy"
mkdir -p "$copy/.claude/gate.d" "$copy/.claude/kb" "$copy/research/results" "$copy/research/scripts"
cp "$root/.claude/gate.d/40-results-cited.sh" "$copy/.claude/gate.d/"
cp "$root/.claude/kb/experiments.md" "$copy/.claude/kb/"
cp -r "$root/.claude/kb/experiments" "$copy/.claude/kb/"
cp "$root/research/scripts/changed-paths.sh" "$copy/research/scripts/"
while IFS= read -r product; do : > "$copy/research/results/$(basename "$product")"; done < <(find "$root/research/results" -maxdepth 1 -name '*.out' -type f)
# 第一道的判定行、它点名的新文件那一行与出路里「删掉它」那一句；基线里别的会话留下的未点名产物只数条数
first_check() { awk '/^  [✗✓] (有实验产物|research\/results 下的实验产物)/{print; p=1; next} p&&/^     research\/results\//{n++; if ($0 ~ /e14-discrimination/) print} p&&/废弃产物就删掉它/{print; p=0} END{print "     （第一道列出的未点名产物共 " n+0 " 份）"}' ; }
echo "== 基线（不加新文件）"
( cd "$copy" && bash .claude/gate.d/40-results-cited.sh "$copy" 2>&1 ) | first_check
for name in e14-discrimination-2026-09-26.out e14-discrimination-r2.out e14-discrimination.r2.out e14-discrimination.r12.out; do
  : > "$copy/research/results/$name"
  echo "== 加 $name"
  ( cd "$copy" && bash .claude/gate.d/40-results-cited.sh "$copy" 2>&1 ) | first_check
  rm -f "$copy/research/results/$name"
done
rm -rf -- "${scratch:?}"
