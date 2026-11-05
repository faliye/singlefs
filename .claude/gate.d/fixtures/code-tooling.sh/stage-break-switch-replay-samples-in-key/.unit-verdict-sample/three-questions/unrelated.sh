#!/usr/bin/env bash
# 复用三问的闭包外改动：beta-stage.sh 末尾加一行注释（shell 只改注释不作废），三条的键都不变
set -euo pipefail
printf '# 闭包外的改动，只加一行注释\n' >> .claude/gate.d/beta-stage.sh
