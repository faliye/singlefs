#!/usr/bin/env bash
# 门禁 crash-point-replay 计划那一步放测试二进制时硬链接放不成、改拷并去掉调试段的样本现场：同 shards-run-the-plan-binaries-without-building，另有假 ln（一律退 1）与假 objcopy；
# 样本拷进临时目录之后在那里 git init（判定库与实跑的日志目录放 git common-dir），假工具补上执行位；
# 假的构建输出（验证包的构建脚本写的判器键表与键所读的文件的摘要表）：键所读的文件只有 crates/singlefs-verification/src/lib.rs 一份，摘要按样本根下此刻的内容算，
# 键表附在标记目录 binaries/ 里每个假测试二进制 exec 之后（真二进制里键表也编在里面）；建一份空的 cargo-calls（假 cargo 每起一遍 test 记一行，门禁照它打累计遍数）
set -euo pipefail
git init -q .
tools=.crash-point-replay-sample-tools
chmod +x "$tools/cargo" "$tools/rustc" "$tools/ln" "$tools/objcopy"
build_out="$tools/build/singlefs-verification-0000000000000000/out"
mkdir -p "$build_out"
printf '%s\t%s\n' "$(sha256sum <crates/singlefs-verification/src/lib.rs | cut -d' ' -f1)" crates/singlefs-verification/src/lib.rs >"$build_out/watched_file_digests.tsv"
printf '# singlefs entry group keys\ntoolchain\tsample\ngroup\tsample\tkeyed\t%s\n' "$(sha256sum <"$build_out/watched_file_digests.tsv" | cut -d' ' -f1)" >"$build_out/judging_units_keys.tsv"
for target in plain_stream other_stream; do
  [[ -f "$tools/binaries/$target" ]] || continue
  chmod +x "$tools/binaries/$target"
  cat "$build_out/judging_units_keys.tsv" >>"$tools/binaries/$target"
done
: >"$tools/cargo-calls"
