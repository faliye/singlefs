#!/usr/bin/env bash
# code-tooling 的 break-switch-replay 格「强制现判那一条不取回」样本的现场：git init（判定库名字取被判那棵树的 git common-dir；两趟用同一份判定库）。
# 根下没有 .unit-verdict-sample/：第二趟照 .break-switch-replay-second-pass 写的种子抽复用哨兵，强制现判的是 SOLO_BREAK=drop 那一条
set -euo pipefail
git init -q .
