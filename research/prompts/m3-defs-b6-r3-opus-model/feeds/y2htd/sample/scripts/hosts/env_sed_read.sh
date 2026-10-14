#!/usr/bin/env bash
# 攻方样本（Y2）：从 env 的输出里取自己的线程变量（printenv 的近邻），没设时回退到写死的 8
threads=$(env | sed -n 's/^SAMPLE_ENVSED_THREADS=//p')
threads=${threads:-8}
cargo build -j "$threads"
