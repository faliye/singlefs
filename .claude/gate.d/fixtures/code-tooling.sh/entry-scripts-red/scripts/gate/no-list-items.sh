#!/usr/bin/env bash
# 样本入口脚本：有自证，没有列项那个参数
if [[ "${1:-}" == --selftest ]]; then exit 0; fi
echo "SAMPLE_ITEM item=${1:-all} exit=0"
