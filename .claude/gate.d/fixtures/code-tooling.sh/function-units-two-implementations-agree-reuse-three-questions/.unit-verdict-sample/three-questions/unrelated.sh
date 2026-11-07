#!/usr/bin/env bash
# 复用三问的闭包外改动：样本根加一份 crate 目录之外的文件（两条的键都不收它），两条的键都不变
set -euo pipefail
printf '%s\n' '只是样本根里 crate 目录之外的一份文件' > notes.txt
