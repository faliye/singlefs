#!/usr/bin/env bash
# t4_links_metadata.sh <仓根> <草稿目录>：在改后副本（after）里看 singlefs-core 构建脚本交出的 enabled_features 元数据与 cargo 给 rustc 的特性对不对得上。
# 每一格用自己的新编译目录（target-t4-<格>），只编 singlefs-core（cargo build -v），读构建脚本的 output 与 rustc 那一行有没有 --cfg 'feature="lookup-work-counters"'：
#   plain：什么都不设；feature：--features lookup-work-counters（对照：cargo 真开）；
#   environment：环境里带 CARGO_FEATURE_LOOKUP_WORK_COUNTERS=1、不开特性；config_env：--config 里 [env] 设同一个变量、不开特性；
#   stale：environment 那个编译目录里不带那个变量再编一次（构建脚本有没有重跑、元数据变没变）。
# 再在 environment 与 plain 两种设法下各 cargo check 一次 checker 档（--no-default-features，库本身编不过不要紧，构建脚本照跑），
# 读它写进 OUT_DIR 的判器键表：build-feature 行、各组键行两边有几组不同。
set -u
repo=$1; work=$2
copy="$work/after"
cargo_in() {   # cargo_in <格> <日志名> <cargo 参数…>（环境前缀由调用方给）
  local cell=$1 log=$2; shift 2
  ( cd "$copy" && CARGO_TARGET_DIR="$work/target-t4-$cell" nice -n 19 bash "$repo/scripts/hosts/capped.sh" 31 \
      bash "$repo/scripts/hosts/run-with-memory-cap.sh" 16G cargo "$@" ) > "$work/t4-$log.txt" 2>&1
}
report() {     # report <格> <日志名>
  local cell=$1 log=$2 output cfg rerun
  output=$(cat "$work"/target-t4-"$cell"/debug/build/singlefs-core-*/output 2>/dev/null | grep -h 'metadata=enabled_features' | sort -u | tr '\n' ' ')
  if grep -E 'Running `.*rustc --crate-name singlefs_core ' "$work/t4-$log.txt" | grep -qF 'feature="lookup-work-counters"'; then cfg=on
  elif grep -qE 'Running `.*rustc --crate-name singlefs_core ' "$work/t4-$log.txt"; then cfg=off
  else cfg=not-recompiled; fi
  if grep -qE 'Running `.*singlefs-core-[0-9a-f]+/build-script-build`' "$work/t4-$log.txt"; then rerun=yes; else rerun=no; fi
  echo "name=t4 cell=$log build_script_ran=$rerun rustc_cfg_lookup_work_counters=$cfg build_script_output=[${output% }]"
}
cargo_in plain plain build --offline -v -p singlefs-core; report plain plain
cargo_in feature feature build --offline -v -p singlefs-core --features lookup-work-counters; report feature feature
CARGO_FEATURE_LOOKUP_WORK_COUNTERS=1 cargo_in environment environment build --offline -v -p singlefs-core; report environment environment
cargo_in config_env config_env build --offline -v -p singlefs-core --config 'env.CARGO_FEATURE_LOOKUP_WORK_COUNTERS="1"'; report config_env config_env
cargo_in environment stale build --offline -v -p singlefs-core; report environment stale
CARGO_FEATURE_LOOKUP_WORK_COUNTERS=1 cargo_in environment checker_tier check --offline -v -p singlefs-checker-tier --no-default-features
cargo_in plain checker_tier_plain check --offline -v -p singlefs-checker-tier --no-default-features
for cell in plain environment; do
  keys=$(ls "$work"/target-t4-$cell/debug/build/singlefs-checker-tier-*/out/judging_units_keys.tsv 2>/dev/null | head -1)
  grep '^group' "$keys" | sort > "$work/t4-keys-$cell.txt"
  echo "name=t4 cell=checker_tier_keys_$cell table=$([[ -n "$keys" ]] && echo found || echo missing) groups=$(wc -l < "$work/t4-keys-$cell.txt") build_feature_lines=[$(grep '^build-feature' "$keys" 2>/dev/null | tr '\t\n' '  ' | sed 's/ *$//')]"
done
echo "name=t4 cell=checker_tier_keys_compare groups_with_a_different_line=$(comm -3 "$work/t4-keys-plain.txt" "$work/t4-keys-environment.txt" | grep -c '^group' ) of $(wc -l < "$work/t4-keys-plain.txt")"
