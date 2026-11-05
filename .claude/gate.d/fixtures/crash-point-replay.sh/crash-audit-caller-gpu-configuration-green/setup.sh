#!/usr/bin/env bash
# 门禁 crash-point-replay 样本 crash-audit-caller-gpu-configuration-green：同 crash-audit-cells-green 一棵树，另写一份 ENABLE_GPU=1 的多机配置，
# caller-environment 里 SINGLEFS_MULTI_HOST_CONFIG 指到它（当作 task-server 带进来的真配置，绝对路径要这里现写）、不给卡；
# 样本档里准入模块照样本根的找法读配置（样本根没有 multi-host.env，不开 GPU），查库不上卡，判法同 crash-audit-cells-green
set -euo pipefail
git init -q .
chmod +x .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
printf 'ENABLE_GPU=1\n' > .crash-point-replay-sample-tools/caller-multi-host.env
printf 'SINGLEFS_MULTI_HOST_CONFIG=%s\n' "$PWD/.crash-point-replay-sample-tools/caller-multi-host.env" > .crash-point-replay-sample-tools/caller-environment
