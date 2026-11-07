#!/usr/bin/env bash
# 判定库：样本里的假库
printf '%s\n' "REUSE_SELFTEST script=scripts/gate/beta.sh same-inputs=reused unrelated-change=reused unit-change=rejudged-only-affected items=3 affected=1"
