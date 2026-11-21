#!/usr/bin/env bash
# 样本：入口只调同目录的 other-cores.sh；基准那一版里 $(nproc) 那一行就在这一份里
bash "$(dirname "$0")/other-cores.sh"
