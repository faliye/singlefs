#!/usr/bin/env bash
# 样本根要是 git 工作树：判法的判定库按 git common-dir 取名（样本在临时 prune-server 底下另占一份命名空间）。只起 git。
set -euo pipefail
git init -q .
