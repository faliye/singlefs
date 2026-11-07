#!/usr/bin/env bash
# 探针纯逻辑的桩自检（不碰仓里的代码、不起 cargo）：用 rustc 把 stub_verification.rs 编成假的 singlefs_verification 库，
# 再把 ../../probe 下的分类器与逐态对照两份 .rs.in 加 selfcheck_tests.rs.in 拼成一个测试程序编出来。
# 用法：bash build.sh <输出目录>；编完在输出目录里直接执行 ./selfcheck_bin（两个测试：记录解析与宽读法对按类取宽度、比较器在加性判定上不报、在跨组判定上报红且分片加起来对得上）。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="${1:?输出目录}"
mkdir -p "$out"
{
  printf '#![allow(dead_code, unused)]\nuse singlefs_verification::crash::PublishedVersion;\nuse singlefs_verification::memory_pool::{MemoryPool, RetainedWrite};\n'
  cat "$here/../../probe/named_slots_classifier.rs.in" "$here/../../probe/exhaustive.rs.in" "$here/selfcheck_tests.rs.in"
} > "$out/selfcheck.rs"
rustc --edition 2021 --crate-type lib --crate-name singlefs_verification "$here/stub_verification.rs" -o "$out/libsv.rlib"
rustc --edition 2021 --test "$out/selfcheck.rs" --extern "singlefs_verification=$out/libsv.rlib" -o "$out/selfcheck_bin"
echo "编好：$out/selfcheck_bin"
