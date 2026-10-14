#!/usr/bin/env bash
# m3-defs-b9-r3 攻方腿：给主 agent 跑门禁 doc-process-records 的 abandoned-rounds 格造小树（子 agent 起不了 --check）。
# 树：给的冻结副本 cp -al 一份，删掉根下 multi-host.env 那一条硬链接；登记表断开硬链接后追加一行；加一份当天的调度记录、一轮的正文与快照。
#   T/abandoned-paren-after-migrate   出处那一行已照 date-once 的出路（doc-consolidate.py migrate）删掉「（2026-10-14）」之后的样子
#   T/abandoned-paren-before-migrate  同一行删之前的样子（对照）
# 用法：make_trees.sh <冻结副本根> <新的空目录 T>
set -euo pipefail
frozen="$(cd "$1" && pwd)" T="$2"
mkdir -p "$T"
record="records/2026-10-14-样本调度.md" table="research/prompts/abandoned-rounds.tsv" round=hx-paren-r1
make_tree() {   # 树名 记录第 6 行
  local tree="$T/$1"
  cp -al "$frozen" "$tree"
  rm -f "$tree/multi-host.env"
  cp --remove-destination "$frozen/$table" "$tree/$table"
  printf '%s\t2026-10-14\t用户叫停（出处 %s 第 6 行〔%s〕），两条腿当场停掉\t不写判决\n' "$round" "$record" 'hx-paren-r1 用户叫停（2026-10-14）' >> "$tree/$table"
  printf '%s\n' '# 样本调度' '' '## 课题' '' "- 开轮 $round：冻结副本在工作根下，快照 research/prompts/$round-snapshot/。" "$2" '- 别的活照常。' > "$tree/$record"
  : > "$tree/research/prompts/_$round-body.md"
  mkdir -p "$tree/research/prompts/$round-snapshot"
  printf '%064d  %s\n' 0 crates/singlefs-core/src/lib.rs > "$tree/research/prompts/$round-snapshot/frozen-copy-sha256.txt"
}
make_tree abandoned-paren-before-migrate '- hx-paren-r1 用户叫停（2026-10-14）：「这一轮先停掉，不写判决」，两条腿当场停掉。'
make_tree abandoned-paren-after-migrate '- hx-paren-r1 用户叫停：「这一轮先停掉，不写判决」，两条腿当场停掉。'
