#!/usr/bin/env bash
# systemd-run 的两个写法真这样起命令（S1 / S3 的语义）：-p Environment= 设给 service 单元里那条命令的环境变量；--expand-environment 的值另起一个词、getopt 照认。
# 起的是 printenv / printf，--wait / --scope 等它结束，不留单元。
set -uo pipefail
printf 'SEM\tS1\t%s\trc=%s\n' "$(timeout 60 systemd-run --user --quiet --wait --pipe -p Environment=PROBE_R3_VARIABLE=seen-by-the-unit printenv PROBE_R3_VARIABLE)" "$?"
printf 'SEM\tS3\t%s\trc=%s\n' "$(timeout 60 systemd-run --user --quiet --scope --expand-environment no printf '%s' 'expand-environment-two-word-form-ran')" "$?"
