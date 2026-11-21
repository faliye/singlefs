#!/usr/bin/env bash
# task-server-compiles: yes
# 服务撞内存上限会重排同一条命令，rerun.sh 拒绝非空的 --work；这一层只多一步：先清空那一个运行目录（目录本身留着，它是交这一条的 --cwd），
# 再交给同一个模型目录里的 rerun.sh（--work 给这个运行目录）。
# 用法：rerun-from-a-clean-work-directory.sh <仓根> <冻结副本> <运行目录> <模型目录>（四个都写绝对路径）
#   运行目录要在工作根（~/.local/share/singlefs/work/）下、名字以 -run 收尾、已是目录，别的一律不动、退 2。
# 退出码：2 用法错或运行目录不合；别的是 rerun.sh 的退出码。
set -euo pipefail
if (($# != 4)); then
  echo "用法：rerun-from-a-clean-work-directory.sh <仓根> <冻结副本> <运行目录> <模型目录>" >&2
  exit 2
fi
repo="$1"
frozen="$2"
work="$3"
model="$4"
case "$work" in
  /*/.local/share/singlefs/work/*-run) ;;
  *)
    echo "  ✗ 运行目录 $work 不是工作根下、名字以 -run 收尾的绝对路径：不动" >&2
    exit 2
    ;;
esac
if [[ ! -d "$work" || -L "$work" ]]; then
  echo "  ✗ 运行目录 $work 不是目录（先 mkdir -p 它再交，它是这一条的 --cwd）：不动" >&2
  exit 2
fi
shopt -s dotglob nullglob
rm -rf -- "${work:?}"/*
shopt -u dotglob nullglob
bash "$model/tools/rerun.sh" --repo "$repo" --frozen "$frozen" --work "$work" --model "$model"
