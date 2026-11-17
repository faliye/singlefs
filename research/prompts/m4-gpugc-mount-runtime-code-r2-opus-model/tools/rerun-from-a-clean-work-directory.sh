#!/usr/bin/env bash
# 服务撞内存上限会重排同一条命令；rerun.sh 拒绝非空的草稿目录。这一层只多一步：--work 里有 rerun.sh 上一次留的记号才清掉它，别的目录不碰。
# 用法：rerun-from-a-clean-work-directory.sh <仓根> <冻结副本> <草稿目录> <模型目录>
set -euo pipefail
repository="$1"
frozen="$2"
work="$3"
model="$4"
if [[ -e "$work/.attack-rerun-marker" ]]; then
  rm -rf "$work"
fi
bash "$model/tools/rerun.sh" --repo "$repository" --frozen "$frozen" --work "$work" --model "$model"
