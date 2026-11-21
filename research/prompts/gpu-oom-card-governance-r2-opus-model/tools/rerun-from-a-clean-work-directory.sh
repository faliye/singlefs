#!/usr/bin/env bash
# task-server-compiles: yes
# 服务撞内存上限会重排同一条命令，rerun.sh 拒绝非空的 --work；这一层只多一步：先清空运行目录里的东西（目录本身留着，它是交这一条的 --cwd），
# 再交给同一个模型目录里的 rerun.sh（rerun.sh 的 rerun_steps 里起 cargo，服务照这一层文件头的会编声明给 CARGO_TARGET_DIR）。
# 用法：rerun-from-a-clean-work-directory.sh <仓根> <冻结副本> <运行目录> <模型目录>
#   运行目录要是工作根（~/.local/share/singlefs/work/）下第一层、名字以 -run 收尾、已存在的目录，别的一律不动、退 2。
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
  */.local/share/singlefs/work/*/*)
    echo "  ✗ 运行目录 $work 不在工作根的第一层：不动" >&2
    exit 2
    ;;
  */.local/share/singlefs/work/*-run) ;;
  *)
    echo "  ✗ 运行目录 $work 不在工作根下或名字不以 -run 收尾：不动" >&2
    exit 2
    ;;
esac
if [[ ! -d "$work" ]]; then
  echo "  ✗ 运行目录 $work 不在：先 mkdir -p 它（它是交这一条的 --cwd）" >&2
  exit 2
fi
find "${work:?}" -mindepth 1 -maxdepth 1 -exec rm -rf -- {} +
bash "$model/tools/rerun.sh" --repo "$repo" --frozen "$frozen" --work "$work" --model "$model"
