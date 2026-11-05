#!/usr/bin/env bash
# 样本：上限写 auto（内存包装不认 auto；上限归 task-server，写 auto 同样是脚本自己定上限）
bash scripts/hosts/run-with-memory-cap.sh auto "$BINARY" run
memory_max="${SAMPLE_MEMORY_MAX:-auto}"
