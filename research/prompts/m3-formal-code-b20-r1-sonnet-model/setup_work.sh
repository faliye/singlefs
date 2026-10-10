#!/usr/bin/env bash
# 建副本：冻结副本的 crates 加仓根的 Cargo 文件。编译目录用 CARGO_TARGET_DIR=/tmp/claude-1000/m3-formal-code-b20-r1-sonnet/target。
set -eu
W=/tmp/claude-1000/m3-formal-code-b20-r1-sonnet/work
R=/home/fy5090/code/singlefs
mkdir -p "$W/.cargo"
cp -r /tmp/claude-1000/m3-formal-code-b20-r1/tree/crates "$W/crates"
cp "$R/Cargo.toml" "$R/Cargo.lock" "$W/"
cp "$R/.cargo/config.toml" "$W/.cargo/"
