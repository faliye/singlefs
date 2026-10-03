#!/usr/bin/env bash
# 攻方现场 J2-a：在 gate-verdict-escalated-green 的库目录上，照驱动 ensure_baseline 换了 --threads 之后会做的那样，
# 往这一版 baseline.tsv 追加 one-build 的一行新绿记录（热编译的秒数 1.2）。登记表、日志、--summary 一个字不动。
set -euo pipefail
v=e5ca1a7e000000000000000000000000000000000000000000000000000000f1
printf 'one-build\tok\t1.2\t2026-10-04\n' >> .broad-sample/home/versions/$v/baseline.tsv
