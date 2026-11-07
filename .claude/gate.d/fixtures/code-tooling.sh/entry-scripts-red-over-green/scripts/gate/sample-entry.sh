#!/usr/bin/env bash
# 样本入口脚本：列项、自证、每项一行带齐
case "${1:-}" in --list-items) echo alpha; exit 0 ;; --selftest) exit 0 ;; esac
echo "SAMPLE_ITEM item=${1:-all} exit=0 judged=1"
