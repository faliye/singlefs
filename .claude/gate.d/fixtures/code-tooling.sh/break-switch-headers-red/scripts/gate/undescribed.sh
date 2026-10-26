#!/usr/bin/env bash
# 样本脚本：开关那一行没写弄坏哪一处
#   UNDESCRIBED_BREAK=skip
#       undescribed-red（「✗ 自检：该红没红」）
if [[ "${UNDESCRIBED_BREAK:-}" == skip ]]; then exit 0; fi
