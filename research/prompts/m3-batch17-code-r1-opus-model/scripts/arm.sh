#!/usr/bin/env bash
# 用法：arm.sh <臂名> [补丁…]   冻结副本拷一份到 $ATTACK_DIR/<臂名>、加原型 crate、打补丁（补丁在 $MODEL/patches）。
set -eu
MODEL=${MODEL:?模型目录}
FROZEN=${FROZEN_TREE:-/tmp/claude-1000/m3-batch17-code-r1/tree}
arm=$1; shift
rsync -a --delete --exclude target --exclude .git "$FROZEN/" "$ATTACK_DIR/$arm/"
(cd "$ATTACK_DIR/$arm" && patch -s -p0 < "$MODEL/patches/workspace-member.patch")
for p in "$@"; do (cd "$ATTACK_DIR/$arm" && patch -s -p0 < "$MODEL/patches/$p"); done
mkdir -p "$ATTACK_DIR/$arm/crates/attack-b17-probe/src"
cp "$MODEL/probe/Cargo.toml" "$ATTACK_DIR/$arm/crates/attack-b17-probe/Cargo.toml"
cp "$MODEL/probe/main.rs" "$ATTACK_DIR/$arm/crates/attack-b17-probe/src/main.rs"
