#!/usr/bin/env bash
# 只读：照 crash-verifier.md 第 1 步的字面，在今天这个仓里对四道阶段各跑一遍两条核对命令（git diff --quiet 与 git ls-files --others），只打退出码与点名的文件。
set -uo pipefail
cd "$1"
chk() {
  name=$1; others=$2; shift 2
  git diff --quiet -- "$@"; d=$?
  o=$(git ls-files --others --exclude-standard -- $others | wc -l)
  echo "$name | git diff --quiet 退 $d | 未跟踪 $o 个"
  [ "$d" -ne 0 ] && git diff --name-only -- "$@" | sed 's/^/    不同：/'
}
# 每一道的路径取 stage-inputs.tsv 里它那一行第 2 列，再加它自己的阶段脚本（crash-verifier.md 第 1 步写的）
for stage in checker-tier-qemu-device-streams checker-tier-crates-mutation-replay checker-tier-formal-verification; do
  paths=$(awk -F'\t' -v s="$stage.sh" '$1==s{print $2}' .claude/gate.d/stage-inputs.tsv)
  case $stage in
    checker-tier-formal-verification) others="crates research/results" ;;
    *) others="crates" ;;
  esac
  chk "$stage" "$others" $paths ".claude/gate.d/$stage.sh"
done
chk checker-tier-lkmm "crates litmus .lkmm-static-only" litmus crates .claude/scripts/lkmm.sh .claude/gate.d/checker-tier-lkmm.sh
echo "research/results 的 git status 行首两位统计："
git status --porcelain research/results | awk '{print substr($0,1,2)}' | sort | uniq -c
