#!/usr/bin/env bash
# 样本：自己钉线程数的几种 shell 写法（命令前缀赋值、-j、--test-threads、:= 默认）
SAMPLE_DEVICE_THREADS=8 cargo run -q --release --bin sample-device
cargo build -j 8
cargo test -- --test-threads 8
: "${CARGO_BUILD_JOBS:=8}"
