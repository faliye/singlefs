#!/usr/bin/env bash
# 样本：上限变量的默认值与环境前缀写死
MEMORY_CAP="${SAMPLE_MEMORY_CAP:-24G}"
SAMPLE_MEMORY_CAP=40G bash scripts/experiment-apparatus/fixed_wrapper.sh
merge_memory_cap=8G
