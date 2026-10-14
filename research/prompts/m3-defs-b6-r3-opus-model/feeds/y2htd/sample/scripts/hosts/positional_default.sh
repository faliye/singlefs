#!/usr/bin/env bash
# 攻方样本（Y2）：并行度从第一个参数取，没给参数时写死 8（「给了参数照参数」，没给就该取登记值）
jobs=${1:-8}
cargo build -j "$jobs"
