#!/usr/bin/env bash
# code-tooling 的 stage-break-switch-replay 格复用三问样本的现场：git init（判定库名字取被判那棵树的 git common-dir；四趟用同一份判定库）
set -euo pipefail
git init -q .
