#!/usr/bin/env bash
# 复用三问的闭包外改动：样本根加一份 crates/ 之外、不在登记的共用输入里的文件，两组的键都不变
set -euo pipefail
printf '%s\n' '只是样本根里 crates/ 之外的一份文件' > notes.txt
