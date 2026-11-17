#!/usr/bin/env bash
# code-tooling 的 break-switch-replay 格「打断之后第二趟只补没落盘的」样本的现场：git init（判定库名字取被判那棵树的 git common-dir；三趟用同一份判定库）
set -euo pipefail
git init -q .
