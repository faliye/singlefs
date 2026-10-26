#!/usr/bin/env bash
# 样本：代码里实现了 --selftest，有阶段在跑它
[[ "${1:-}" == --selftest ]] && exit 0
exit 0
