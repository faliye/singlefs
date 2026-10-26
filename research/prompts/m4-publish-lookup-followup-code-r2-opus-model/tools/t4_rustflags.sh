#!/usr/bin/env bash
# t4_rustflags.sh：在树根跑（multi-host-run.sh --peer 起）。经 RUSTFLAGS 给 rustc 一个 --cfg feature="lookup-work-counters"、cargo 不开这个特性，
# 两格各用自己的编译目录编 singlefs-core（cargo build -v）：
#   rustflags_with_target：带 --target（cargo 不把 RUSTFLAGS 给宿主的构建脚本，只给目标上的库）；
#   rustflags_without_target：不带 --target（RUSTFLAGS 也给构建脚本）。
# 每格读构建脚本交出的元数据与编库那一行 rustc 有没有那个 cfg，打一行 name=t4。
set -u
triple=$(rustc -vV | sed -n 's/^host: //p')
mkdir -p out
for cell in rustflags_with_target rustflags_without_target; do
  target_arguments=()
  [[ $cell == rustflags_with_target ]] && target_arguments=(--target "$triple")
  RUSTFLAGS='--cfg feature="lookup-work-counters"' CARGO_TARGET_DIR="../r2opus-target-t4-$cell" \
    bash scripts/hosts/capped.sh 31 cargo build --offline -v -p singlefs-core "${target_arguments[@]}" > "out/t4-$cell.txt" 2>&1
  rc=$?
  output=$(find ../r2opus-target-t4-"$cell" -path "*/build/singlefs-core-*/output" -exec cat {} + 2>/dev/null | grep -h 'metadata=enabled_features' | sort -u | tr '\n' ' ')
  if grep -E 'Running `.*rustc --crate-name singlefs_core ' "out/t4-$cell.txt" | grep -v 'build_script_build' | grep -qF 'feature="lookup-work-counters"'; then cfg=on; else cfg=off; fi
  if grep -E 'Running `.*rustc --crate-name build_script_build .*singlefs-core' "out/t4-$cell.txt" | grep -qF 'feature="lookup-work-counters"'; then script_cfg=on; else script_cfg=off; fi
  echo "name=t4 cell=$cell build_rc=$rc library_rustc_cfg_lookup_work_counters=$cfg build_script_rustc_cfg_lookup_work_counters=$script_cfg build_script_output=[${output% }]"
done
