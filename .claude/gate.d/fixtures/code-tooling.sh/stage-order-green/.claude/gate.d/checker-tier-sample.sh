#!/usr/bin/env bash
# gate-stage: 样本：跑测试的阶段（复跑实验），排在环境检查前面
ROOT="${1:-.}"
cd "$ROOT" || exit 2
bash research/scripts/replay.sh
