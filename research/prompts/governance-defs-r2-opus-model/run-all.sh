#!/usr/bin/env bash
# 用法：bash run-all.sh <草稿目录>     复跑这条腿的全部模型，输出打到 stdout（报告里的原样输出都出自它）
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; root="$(cd "$here/../../.." && pwd)"; draft="$1"; mkdir -p "$draft"
echo "### 钩子（Bash）"; bash "$here/run-hooks.sh" "$draft"
echo "### 派发闸"; bash "$here/feed-dispatch.sh" "$draft"
echo "### G1 三行命令的退出码（今天的写法）"; bash "$here/g1-exit-status.sh" "$root" "$draft"
echo "### G1 三行命令的退出码（攻方提的写法）"; bash "$here/g1-exit-status-fixed.sh" "$root" "$draft"
echo "### G2 核对命令在几段历史上"; bash "$here/g2-diff-check.sh" "$root" "$draft"
echo "### G2 放开别的会话改哪一个文件"; bash "$here/g2-surface.sh" "$root"; bash "$here/g2-surface-59-copyset.sh" "$root"
echo "### G2 cargo 认不认未跟踪文件"; bash "$here/g2-cargo-discovery.sh" "$root" "$draft"
echo "### G3 同一份输出在 59 号与 mutate.sh 两边判成什么"; python3 "$here/g3-classify.py" "$root" "$here/g3-outputs"
