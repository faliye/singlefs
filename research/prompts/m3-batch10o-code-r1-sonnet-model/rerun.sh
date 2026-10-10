#!/bin/bash
# m3-batch10o-code-r1 正推腿（sonnet）T2「回退不拒」臂的复跑。
# 原型来源：main.rs 取自 m3-batch10i-code-r1-opus-model/main.rs（攻方原型，s1grid 子命令），改动见 probe-main-vs-10i-opus-main.diff：
#   回退那一步加了「无故障盘面上的 F_生效 真值」与 OPUS_ROLLBACK_BELOW（挑 F_生效 之下的有效根当目标）。
# 核心补丁 t2-arm-core.patch：只在环境变量 SONNET_T2_ARM=norefuse 时生效（回退判候选那一遍，S1 槽重读仍坏拿记着的根顶上）；不设就是冻结副本今天的码。
# 用法：
#   bash rerun.sh setup <草稿目录> [冻结副本根]   拷冻结副本 crates、打补丁、放原型、release 编译（经内存包装与 capped.sh 15）
#   bash rerun.sh run <草稿目录> <臂 today|norefuse> <步数> <模式 p0|p1|p2|b0|b1>   日志写 <草稿目录>/out/<臂>-s<步数>-<模式>.log
#   python3 agg.py <日志…>                          按（起点、槽类、挂载后回退结局、期望、顶上次数、checker red）汇总
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
REPO=/home/fy5090/code/singlefs
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="复跑正推腿原型，与别的会话并行"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="原型各子命令与线程数不同，峰值表里那一次是另一种"
case ${1:-} in
  setup)
    W=${2:?草稿目录}; FROZEN=${3:-$HOME/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2}
    mkdir -p "$W/tree/crates/t2-arm-probe/src" "$W/out"
    cp -a "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$FROZEN/crates" "$W/tree/"
    mkdir -p "$W/tree/research"; cp "$FROZEN/research/Cargo.toml" "$W/tree/research/"
    (cd "$W/tree" && patch -p1 -s < "$HERE/t2-arm-core.patch")
    sed -i 's#    "crates/singlefs-reference-filesystem",#    "crates/singlefs-reference-filesystem",\n    "crates/t2-arm-probe",#' "$W/tree/Cargo.toml"
    cp "$HERE/Cargo.toml" "$W/tree/crates/t2-arm-probe/Cargo.toml"; cp "$HERE/main.rs" "$W/tree/crates/t2-arm-probe/src/main.rs"
    cd "$W/tree" && nice -n 19 bash "$REPO/research/scripts/run-with-memory-cap.sh" 6G bash "$REPO/research/scripts/capped.sh" 15 cargo build --offline --release -p t2-arm-probe > "$W/out/build.log" 2>&1
    echo "setup 完：$W";;
  run)
    W=${2:?草稿目录}; arm=${3:?臂}; steps=${4:?步数}; mode=${5:?模式}
    envs="OPUS_ROLLBACK_AT_END=1 OPUS_STEPS=$steps"
    [ "$arm" = norefuse ] && envs="$envs SONNET_T2_ARM=norefuse"
    case $mode in
      p0) envs="$envs OPUS_ROLLBACK_PICK=0";; p1) envs="$envs OPUS_ROLLBACK_PICK=1";; p2) envs="$envs OPUS_ROLLBACK_PICK=2";;
      b0) envs="$envs OPUS_ROLLBACK_BELOW=1 OPUS_ROLLBACK_PICK=0";; b1) envs="$envs OPUS_ROLLBACK_BELOW=1 OPUS_ROLLBACK_PICK=1";;
    esac
    env $envs OPUS_WORKERS=14 nice -n 19 bash "$REPO/research/scripts/run-with-memory-cap.sh" 3G bash "$REPO/research/scripts/capped.sh" 15 \
      "$W/tree/target/release/t2-arm-probe" s1grid mat-384-30-8,ab-384-3-5 > "$W/out/$arm-s$steps-$mode.log" 2>&1
    echo "rc=$? $arm-s$steps-$mode";;
  *) echo "用法见文件头" >&2; exit 2;;
esac
