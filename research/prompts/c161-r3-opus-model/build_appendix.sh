#!/usr/bin/env bash
# 把报告里用「」引到的 kb 句子整行机械抽出来（sed，不转述），写成附录草稿。
set -euo pipefail
repo="$(cd "$(dirname "$0")/../../.." && pwd)"          # 模型目录在 <仓根>/research/prompts/c161-r3-opus-model
out="${1:-${TMPDIR:-/tmp}/c161-r3-appendix.txt}"
: > "$out"
emit() {
  local tag="$1" file="$2" range="$3"
  printf '**%s `%s:%s`**（sed 机械抽取，整行）\n\n````text\n' "$tag" "$file" "$range" >> "$out"
  sed -n "${range/-/,}p" "$repo/$file" >> "$out"
  printf '````\n\n' >> "$out"
}
emit Q1 .claude/kb/decisions/23-journal的角色与格式.md 691-691
emit Q2 .claude/kb/decisions/05-快照-空间记账机制.md 308-309
emit Q3 .claude/kb/decisions/05-快照-空间记账机制.md 284-287
emit Q4 .claude/kb/decisions/05-快照-空间记账机制.md 296-296
emit Q5 .claude/kb/decisions/08-核心索引结构.md 78-78
emit Q6 .claude/kb/decisions/08-核心索引结构.md 447-447
emit Q7 .claude/kb/decisions/08-核心索引结构.md 460-460
emit Q8 .claude/kb/decisions/11-索引节点要不要留消息缓冲区.md 231-235
emit Q9 .claude/kb/decisions/11-索引节点要不要留消息缓冲区.md 254-256
emit Q10 .claude/kb/decisions/11-索引节点要不要留消息缓冲区.md 137-137
emit Q11 .claude/kb/decisions/16-发布语义.md 21-23
emit Q12 .claude/kb/decisions/16-发布语义.md 161-163
emit Q13 .claude/kb/decisions/23-journal的角色与格式.md 694-694
emit Q14 .claude/kb/decisions/23-journal的角色与格式.md 697-697
emit Q15 .claude/kb/decisions/23-journal的角色与格式.md 698-698
emit Q16 .claude/rules/fs-design.md 105-113
emit Q17 .claude/rules/fs-design.md 130-136
emit Q18 .claude/kb/checks-owed.md 163-163
emit Q19 .claude/kb/checks-owed.md 90-90
emit Q20 .claude/kb/checks-owed.md 62-62
emit Q21 .claude/kb/decisions/08-核心索引结构.md 42-44
emit Q22 .claude/kb/invariants.md 120-120
emit Q23 .claude/kb/decisions/15-格式冻结政策.md 64-64
emit Q24 .claude/kb/checks-owed.md 304-304
emit Q25 .claude/kb/checks-owed.md 306-306
wc -l "$out"
