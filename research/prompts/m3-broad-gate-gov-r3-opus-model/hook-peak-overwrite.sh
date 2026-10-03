#!/usr/bin/env bash
# 攻方现场 J2-a 撞顶那一边：同一线程数下基线那一步又跑了一遍（上一行基线不绿之后重跑、或峰值表丢了），
# 包装的 finish 照 research/scripts/run-with-memory-cap.sh 的写法把这个键的峰值改写成这一次量的（按 MiB 向上取整，这里 1025 MiB）。
set -euo pipefail
v=e5ca1a7e000000000000000000000000000000000000000000000000000000f1
f=.broad-sample/home/versions/$v/baseline-peaks.8.tsv
printf '# 样本的基线峰值（包装的峰值表写法）\n1074790400\t2026-10-04T00:00:00Z\t24G\tbroad-mutation baseline one-test\t8\n' > "$f.new"; mv "$f.new" "$f"
