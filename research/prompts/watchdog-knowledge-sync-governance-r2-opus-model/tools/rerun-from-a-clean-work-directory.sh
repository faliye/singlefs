#!/usr/bin/env bash
# 服务撞内存上限会重排同一条命令，rerun.sh 拒绝非空的 --work；这一层只多一步：先删掉上一趟留下的那一个运行目录，再交给同一个模型目录里的 rerun.sh。
# 用法：rerun-from-a-clean-work-directory.sh <仓根> <冻结副本> <运行目录> <模型目录>
#   运行目录要是工作根（<任意>/.local/share/singlefs/work）底下直接的一层、名字以 -run 收尾、不带 .. 段；别的一律不删、退 2。
#   冻结副本给一棵 scripts/agents/agent-watch.py 与开工快照 judged-sha256.txt 那一行相同的树（探针第一行 PROBE name=watchdog 打出它的 sha256，对一下）。
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
  */..|*/../*|*/./*) echo "  ✗ 运行目录 $work 带 . 或 .. 段：不删" >&2; exit 2 ;;
esac
parent="$(dirname -- "$work")"
case "$parent" in
  */.local/share/singlefs/work) ;;
  *) echo "  ✗ 运行目录 $work 不是工作根底下直接的一层：不删" >&2; exit 2 ;;
esac
case "$(basename -- "$work")" in
  *-run) ;;
  *) echo "  ✗ 运行目录 $work 的名字不以 -run 收尾：不删" >&2; exit 2 ;;
esac
rm -rf -- "${work:?}"
bash "$model/tools/rerun.sh" --repo "$repo" --frozen "$frozen" --work "$work" --model "$model"
