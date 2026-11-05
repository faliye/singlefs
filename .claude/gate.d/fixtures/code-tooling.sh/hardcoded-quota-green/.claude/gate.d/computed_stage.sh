#!/usr/bin/env bash
# 样本：门禁阶段里读多机配置写的各台 slice 总上限（读来报数，不当一条命令的上限）
memory_max="$(grep -m1 '^SLICE_MEMORY_CAP_BY_HOST=' "$SINGLEFS_MULTI_HOST_CONFIG" | cut -d= -f2-)"
