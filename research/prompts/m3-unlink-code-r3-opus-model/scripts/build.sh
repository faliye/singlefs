#!/usr/bin/env bash
# 用法：build.sh <树目录>   在那棵树上经内存包装编原型（线程上限 10）。
set -u
export CARGO_TARGET_DIR=${ATTACK_DIR:-/tmp/claude-1000/m3-unlink-code-r3-opus}/target
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="原型改了源码或换了树，依赖可能重编，峰值表里那次只编了顶层"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="三方攻方腿与正推腿同会话各跑各的原型"
cd "$1" && nice -n 19 bash ${SINGLEFS_REPO:-/home/fy5090/code/singlefs}/scripts/hosts/run-with-memory-cap.sh 4G bash ${SINGLEFS_REPO:-/home/fy5090/code/singlefs}/scripts/hosts/capped.sh 10 cargo build --release --offline -p attack-r3-probe
