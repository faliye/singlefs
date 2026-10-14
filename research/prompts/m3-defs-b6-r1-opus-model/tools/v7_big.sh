#!/usr/bin/env bash
# V7 整份读的时长与内存：在 <WORK>/pv-big 造 200 MiB 短行日志与 300 MiB 单行文件（未跟踪），用 run_pv_cell.py 跑零轮那一版的判法，/usr/bin/time 记挂钟与最大常驻。
# 用法：v7_big.sh <WORK> <零轮那一版仓根>
set -u
work=$1; repo=$2; here=$(cd "$(dirname "$0")" && pwd)
s="$work/pv-big"; rm -rf "$s"; cp -a "$here/../gate-samples/doc-text/pv-tar-mirror-red" "$s"
(cd "$s" && sed -i '/from-peer/d' setup.sh && bash setup.sh)
python3 - "$s" <<'PY'
import os, sys
root = sys.argv[1]
line = b"step=000000 elapsed=12.5s state=ok checker=green rows=31 note=progress-line-padding-padding-xx\n"
with open(os.path.join(root, "research/results/lines-200m.log"), "wb") as f:
    for _ in range(200 * 1024 * 1024 // len(line)):
        f.write(line)
with open(os.path.join(root, "research/results/oneline-300m.log"), "wb") as f:
    f.write(b"x" * (300 * 1024 * 1024))
PY
/usr/bin/time -f 'wall_seconds_rounded_to_one=%e maxrss_kB=%M' python3 "$here/run_pv_cell.py" "$repo" "$s" 2>&1 | sed -E 's/wall_seconds_rounded_to_one=([0-9]+)\.[0-9]+/wall_seconds_rounded_to_one=\1/'
rm -rf "$s"
