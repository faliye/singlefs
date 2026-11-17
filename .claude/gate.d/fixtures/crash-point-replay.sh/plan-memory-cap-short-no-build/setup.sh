#!/usr/bin/env bash
# 门禁 crash-point-replay 计划那一步样本的现场（样本根的 .gate-shard-phase 当作服务交的 TASK_SERVER_SHARD_PHASE=plan）：样本拷进临时目录之后在那里 git init
# （判定库与实跑的日志目录放 git common-dir），假工具补上执行位；假的构建输出（验证包的构建脚本写的判器键表与键所读的文件的摘要表）：键所读的文件只有
# crates/singlefs-verification/src/lib.rs 一份，摘要按样本根下此刻的内容算，键表附在标记目录 binaries/ 里每个假测试二进制 exec 之后；建一份空的 cargo-calls。
# 上一趟记下的块数（计划那一步定片数时读，门禁文件头「分片」plan）：标记目录里 recorded-blocks.tsv 一行「<用例名><TAB><块数>」，照它在 git common-dir 的日志根下
#   造一个有 whole-case-logs 记号的目录（核账那一趟整份 0/1 跑的），每条用例一份带 CRASH_CASE_BLOCKS 那一行的日志；
#   shard-share-blocks.tsv 同样写法，造一个更新、没有那个记号的目录（一片领的那一份的块数，计划那一步不该认它）；两份都没有就不造（块数没有记录）
set -euo pipefail
git init -q .
tools=.crash-point-replay-sample-tools
chmod +x "$tools/cargo" "$tools/rustc"
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
whole_logs=.git/singlefs-crash-point-replay-logs/20261111T000000Z-1
share_logs=.git/singlefs-crash-point-replay-logs/20261111T010000Z-2
if [[ -f "$tools/recorded-blocks.tsv" ]]; then
  mkdir -p "$whole_logs"
  : >"$whole_logs/whole-case-logs"
  while IFS=$'\t' read -r case_name blocks; do
    [[ -n "$case_name" ]] || continue
    printf 'CRASH_CASE_BLOCKS blocks=%s reused=%s fresh=0 red=0 refused=0 states=%s\n' "$blocks" "$blocks" "$((blocks * 4))" >"$whole_logs/log.$case_name"
  done <"$tools/recorded-blocks.tsv"
fi
if [[ -f "$tools/shard-share-blocks.tsv" ]]; then
  mkdir -p "$share_logs"
  while IFS=$'\t' read -r case_name blocks; do
    [[ -n "$case_name" ]] || continue
    printf 'CRASH_CASE_BLOCKS blocks=%s reused=0 fresh=%s red=0 refused=0 states=%s\n' "$blocks" "$blocks" "$((blocks * 4))" >"$share_logs/log.$case_name"
  done <"$tools/shard-share-blocks.tsv"
fi
