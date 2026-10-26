#!/usr/bin/env bash
# 样本脚本：读了弄坏开关，平常跑设了也不拒
# 弄坏开关：
#   SILENT_BREAK=quiet  读到就多打一行
#       silent-red（「✗ 自检：多打的一行」）
if [[ "${SILENT_BREAK:-}" == quiet ]]; then echo "样本：被弄坏了"; fi
echo "样本：照常跑"
