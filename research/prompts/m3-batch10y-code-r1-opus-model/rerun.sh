#!/bin/bash
# m3-batch10y-code-r1 攻方（opus）原型复跑。只在冻结副本的拷贝上跑（harness 档），经内存包装与 capped.sh。
# 用法：bash rerun.sh setup <空草稿目录> [冻结副本根]   拷冻结副本、打 opus-k4-arms.patch、放两个原型、release 编译
#       bash rerun.sh ledger <草稿目录>                  K1/K2：显存账那几形（不碰显卡；真报告取 out/nvsmi-q-memory-pids.txt）
#       bash rerun.sh k4 <草稿目录>                      K4：b1（C 的账挂上之后读坏）六批、b2（对照：不注账的读坏）四批，并行
#       bash rerun.sh agg <草稿目录>                     python3 k4agg.py 汇总 b1、b2
#       bash rerun.sh shim <草稿目录>                     nvidia-smi 里两张卡读显存那两次 NVML 调用的时刻（LD_PRELOAD，只读、不占卡）
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
REPO=/home/fy5090/code/singlefs
R=$REPO/scripts
case ${1:-} in
  setup)
    W=${2:?草稿目录}; FROZEN=${3:-$HOME/.local/share/singlefs/work/m3-batch10y-code-r1/tree}
    mkdir -p "$W"; [ -z "$(ls -A "$W")" ] || { echo "草稿目录不空：$W" >&2; exit 2; }
    mkdir -p "$W/repo" "$W/out"
    rsync -a --exclude /target --exclude /.git --exclude /scripts/experiment-apparatus/target "$FROZEN/" "$W/repo/"
    (cd "$W/repo" && patch -p1 -s < "$HERE/opus-k4-arms.patch")
    cp "$HERE/opus_10y_ledger_probe.rs" "$HERE/opus_10y_k4_probe.rs" "$W/repo/crates/singlefs-harness/tests/"
    cd "$W/repo" && CARGO_TARGET_DIR="$W/target" nice -n 19 bash "$R/run-with-memory-cap.sh" 8G bash "$R/capped.sh" 10 \
      cargo test --offline --release -p singlefs-harness --test opus_10y_k4_probe --test opus_10y_ledger_probe --no-run > "$W/out/build.log" 2>&1
    echo "setup 完：$W";;
  ledger)
    W=${2:?}; cd "$W/repo"
    CARGO_TARGET_DIR="$W/target" OPUS10Y_REAL_REPORT="$HERE/out/nvsmi-q-memory-pids.txt" nice -n 19 bash "$R/run-with-memory-cap.sh" 4G bash "$R/capped.sh" 10 \
      cargo test --offline --release -p singlefs-harness --test opus_10y_ledger_probe -- --nocapture --test-threads=1 > "$W/out/ledger-probe.log" 2>&1
    grep -o 'OPUS10Y-.*' "$W/out/ledger-probe.log" | sort;;
  k4)
    W=${2:?}; cd "$W/repo"; export CARGO_TARGET_DIR="$W/target"
    one() { local n=$1; shift; { env "$@" nice -n 19 bash "$R/run-with-memory-cap.sh" 4G bash "$R/capped.sh" 1 \
      cargo test --offline --release -p singlefs-harness --test opus_10y_k4_probe -- --nocapture --test-threads=1 > "$W/out/$n.log" 2>&1; echo "$?" > "$W/out/$n.rc"; } & }
    rm -f "$W"/out/b1-*.rc "$W"/out/b2-*.rc
    one b1-1 OPUS_K4_WIDTHS=384 OPUS_K4_ARMS=prod; one b1-2 OPUS_K4_WIDTHS=384 OPUS_K4_ARMS=recorded
    one b1-3 OPUS_K4_WIDTHS=4g OPUS_K4_BEFORES=1 OPUS_K4_ARMS=prod; one b1-4 OPUS_K4_WIDTHS=4g OPUS_K4_BEFORES=1 OPUS_K4_ARMS=recorded
    one b1-5 OPUS_K4_WIDTHS=4g OPUS_K4_BEFORES=3 OPUS_K4_ARMS=prod; one b1-6 OPUS_K4_WIDTHS=4g OPUS_K4_BEFORES=3 OPUS_K4_ARMS=recorded
    wait
    one b2-1 OPUS_K4_NO_ACCOUNT_FAULT=1 OPUS_K4_WIDTHS=384 OPUS_K4_ARMS=prod; one b2-2 OPUS_K4_NO_ACCOUNT_FAULT=1 OPUS_K4_WIDTHS=384 OPUS_K4_ARMS=recorded
    one b2-3 OPUS_K4_NO_ACCOUNT_FAULT=1 OPUS_K4_WIDTHS=4g OPUS_K4_BEFORES=1; one b2-4 OPUS_K4_NO_ACCOUNT_FAULT=1 OPUS_K4_WIDTHS=4g OPUS_K4_BEFORES=3
    wait
    n=$(ls "$W"/out/b1-*.rc "$W"/out/b2-*.rc | wc -l); [ "$n" = 10 ] || { echo "rc 文件 $n 个，不是 10 个：整批作废" >&2; exit 3; }
    for f in b1-1 b1-2 b1-3 b1-4 b1-5 b1-6 b2-1 b2-2 b2-3 b2-4; do echo "$f rc=$(cat "$W/out/$f.rc")"; done;;
  agg)
    W=${2:?}; echo "b1："; python3 "$HERE/k4agg.py" "$W"/out/b1-*.log; echo "b2："; python3 "$HERE/k4agg.py" "$W"/out/b2-*.log;;
  shim)
    W=${2:?}; mkdir -p "$W/shim"; gcc -O1 -shared -fPIC -o "$W/shim/shim.so" "$HERE/nvml-shim/shim.c" -ldl
    nice -n 19 bash "$R/run-with-memory-cap.sh" 1G env LD_PRELOAD="$W/shim/shim.so" nvidia-smi -q -d MEMORY,PIDS > "$W/shim/run.out" 2> "$W/shim/run.err"
    cat "$W/shim/run.err";;
  *) echo "用法见文件头" >&2; exit 2;;
esac
