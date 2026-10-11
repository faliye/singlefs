#!/usr/bin/env bash
# 用法：build.sh <臂名 ga|imm|held>   把原型放进 $ATTACK_DIR/<臂名>（冻结副本的拷贝，补丁已打）、经内存包装编 release（线程上限 ATTACK_THREADS，默认 7）。
set -eu
REPO=${SINGLEFS_REPO:-/home/fy5090/code/singlefs}
MODEL=$REPO/research/prompts/m3-batch14-code-r1-opus-model
arm=$1
mkdir -p "$ATTACK_DIR/$arm/crates/attack-b14-probe/src"
cp "$MODEL/probe/Cargo.toml" "$ATTACK_DIR/$arm/crates/attack-b14-probe/Cargo.toml"
cp "$MODEL/probe/main.rs" "$ATTACK_DIR/$arm/crates/attack-b14-probe/src/main.rs"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS=${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-180}
export CARGO_TARGET_DIR=$ATTACK_DIR/target-$arm
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="原型 crate 与依赖在冻结副本的拷贝上整编"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="三方攻方腿与别的会话各跑各的编译"
cd "$ATTACK_DIR/$arm" && nice -n 19 bash "$REPO/scripts/hosts/run-with-memory-cap.sh" 8G bash "$REPO/scripts/hosts/capped.sh" "${ATTACK_THREADS:-7}" cargo build --release --offline -p attack-b14-probe
