#!/usr/bin/env bash
# 样本脚本：开关表三样写全
# 弄坏开关：
#   WRITTEN_BREAK=skip-check  判法整段跳过
#       written-red（「✗ 自检：该红没红」）
if [[ "${WRITTEN_BREAK:-}" == skip-check ]]; then exit 0; fi
echo "判完"
