#!/usr/bin/env bash
# 样本脚本：入口，自己不读弄坏开关；source 的 divided-core.sh 读 DIVIDED_BREAK 并在被 source 时守住它
# 弄坏开关：
#   DIVIDED_BREAK=skip  判法整段跳过
#       divided-red（「✗ 自检：该红没红」）
source "$(dirname "$0")/divided-core.sh"
echo "样本：正常"
