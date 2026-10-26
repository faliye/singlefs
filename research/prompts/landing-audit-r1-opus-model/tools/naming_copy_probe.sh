#!/usr/bin/env bash
# .claude/agent-common.md「命名检查交回前跑」那两条命令在三种根上各跑一次，只留汇总行：
#   最小副本（照同一份共用约束「写」一节：crates/、Cargo.toml、Cargo.lock、.cargo、.claude/heavy-thread-default，rsync 不带 .git）、
#   最小副本加上 .claude/ 下的命名词表与排除表、仓根。命令本身照抄，范围声明照抄（为什么那一段换成这次的用途）。
# 用法：naming_copy_probe.sh <仓根> <草稿目录>（草稿目录里建 copy-root 与 copy-root-wordlists 两份副本，跑完删掉）
set -uo pipefail
repo="$1"; work="$2"
cd "$repo"
mkdir -p "$work/copy-root/.claude"
rsync -a --exclude target crates Cargo.toml Cargo.lock .cargo "$work/copy-root/"
cp .claude/heavy-thread-default "$work/copy-root/.claude/"
cp -a "$work/copy-root" "$work/copy-root-wordlists"
cp .claude/naming-* .claude/abbreviations .claude/data-name-lint-exclude "$work/copy-root-wordlists/.claude/"
for root in "$work/copy-root" "$work/copy-root-wordlists" "$repo"; do
  echo "== 根：${root#$work/}"
  SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 vague-names、test-file-names、ordinal-names、data-names 四格; 范围=4 格; 为什么=攻方腿核命名自查在副本根与仓根上判不判得到' \
    nice -n 19 bash .claude/gate.d/code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names "$root" > "$work/naming-cells.txt" 2>&1
  echo "code-source-discipline 四格 rc=$?"
  grep -E '^  (✓|✗|⊘|!) ' "$work/naming-cells.txt" | grep -vE '^  ✗ (crates|scripts|\.claude)/' | cut -c1-160
  nice -n 19 bash .claude/singlefs-ai-sop/scripts/naming-lint.sh "$root" > "$work/naming-lint.txt" 2>&1
  echo "naming-lint rc=$?"
  grep -E '^  ✗ [0-9]+ 处名字不合命名纪律' "$work/naming-lint.txt"
done
rm -rf "${work:?}/copy-root" "${work:?}/copy-root-wordlists" "$work/naming-cells.txt" "$work/naming-lint.txt"
