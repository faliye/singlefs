#!/usr/bin/env bash
# 给主 agent 跑门禁格用的小树（子 agent 起不了 --check）。用法：make_trees.sh <冻结副本根（这一轮 b10）> <本模型目录> <新的空目录 T> <仓根>
#   T/opus-trees/…   第一轮攻方模型的 gate-samples/make_trees.sh 照原样对这一轮冻结副本造的七棵（叫停表追加补记、heavy-thread-defaults 三棵、doc-text 一棵、豁免表两棵 git 仓）
#   T/s3-shell       本模型 feeds/s3/sample（S3 变量来源边界的二十九份脚本，EXPECT.tsv 是预期）
#   T/s5/<名>        四棵叫停登记小树（追加补记、术语改名换字、插行再改字、插行再追加补记）
#   T/s6/plain       本模型 feeds/s6/setup.sh 建的 git 样本（假配置，值都是编的）
#   T/s6/anchor      同上，另在 .claude/term-rename-exempt 里加一条对不上的按行豁免，行内锚里写着假主机名
set -euo pipefail
frozen="$(cd "$1" && pwd)" model="$(cd "$2" && pwd)" T="$3" repo="$(cd "$4" && pwd)"
mkdir -p "$T"
bash "$repo/research/prompts/m3-defs-b9-r1-opus-model/gate-samples/make_trees.sh" "$frozen" "$repo/research/prompts/m3-defs-b9-r1-opus-model" "$T/opus-trees"
cp -a "$model/feeds/s3/sample" "$T/s3-shell"
python3 -B "$model/tools/s5_drift_matrix.py" --emit-gate-trees "$T/s5"
mkdir -p "$T/s6/plain"; ( cd "$T/s6/plain" && bash "$model/feeds/s6/setup.sh" )
cp -a "$T/s6/plain" "$T/s6/anchor"
printf '%s\n' "records/2026-10-13-样本调度.md:「不存在的锚 sample-second-host-77 之类」  # 行内锚对不上，sweep-term 会把锚原样打出" >> "$T/s6/anchor/.claude/term-rename-exempt"
echo "建好：$(ls "$T" | tr '\n' ' ')"
