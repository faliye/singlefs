#!/usr/bin/env bash
# 给主 agent 跑门禁格用的小树（子 agent 起不了 --check）。树从冻结副本硬链接拷（cp -al），被改的文件先换成新 inode；冻结副本不被碰。
# 用法：make_trees.sh <冻结副本根> <模型目录> <新的空目录 T>
#   T/htd-s3              模型目录 feeds/s3/make_tree.sh 建的 heavy-thread-defaults 样本（八份喂法加登记、排除表与三份取法脚本；不是 git 仓）
#   T/abandoned-misread   叫停表 m3-batch63-code-r1 那一行换成〔〕里写转述、行号写 1594 的登记；出处记录截到第 1594 行（那一刻的样子；不是 git 仓）
set -euo pipefail
frozen="$(cd "$1" && pwd)" model="$(cd "$2" && pwd)" T="$3"
mkdir -p "$T"
bash "$model/feeds/s3/make_tree.sh" "$frozen" "$T/htd-s3"
record="records/2026-10-01-删除与并发事务.md" table="research/prompts/abandoned-rounds.tsv"
cp -al "$frozen" "$T/abandoned-misread"
rm -f "$T/abandoned-misread/$record" "$T/abandoned-misread/$table" "$T/abandoned-misread/multi-host.env"
head -n 1594 "$frozen/$record" > "$T/abandoned-misread/$record"
grep -v '^m3-batch63-code-r1	' "$frozen/$table" > "$T/abandoned-misread/$table"
printf 'm3-batch63-code-r1\t2026-10-11\t用户 2026-10-11 叫停（records/2026-10-01-删除与并发事务.md 第 1594 行〔m3-batch63-code-r1 用户叫停，不再打〕）\t不写判决\n' >> "$T/abandoned-misread/$table"
