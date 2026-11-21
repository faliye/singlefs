#!/usr/bin/env bash
# task-server-compiles: yes
# 服务撞上限会重排同一条命令，rerun.sh 拒绝非空的 --work；这一层只多一步：先清空那一个运行目录（目录本身留着，它是交这一条的 --cwd），
# 再交给同一个模型目录里的 rerun.sh。会编声明照模板留着（rerun_steps 不编，服务照声明给的 CARGO_TARGET_DIR 由模型在每一格里 unset）。
# 用法：rerun-from-a-clean-work-directory.sh <仓根> <冻结副本> <运行目录> <模型目录>（四个都写绝对路径）
#   运行目录要是工作根（<家目录>/.local/share/singlefs/work/）下一层的目录、名字以 -run 收尾，别的一律不动、退 2。
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
    echo "  ✗ 运行目录 $work 不在工作根下或名字不以 -run 收尾：不动" >&2
    exit 2
    ;;
esac
below_work_root="${work#*/.local/share/singlefs/work/}"
if [[ "$below_work_root" == */* || ! -d "$work" || -L "$work" ]]; then
  echo "  ✗ 运行目录 $work 不是工作根下一层的目录（或是符号链接）：不动" >&2
  exit 2
fi
find "${work:?}" -mindepth 1 -maxdepth 1 -exec rm -rf -- {} +
bash "$model/tools/rerun.sh" --repo "$repo" --frozen "$frozen" --work "$work" --model "$model"
