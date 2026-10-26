#!/usr/bin/env bash
# t4_cells.sh：在仓副本树根里按几种编法各编一次 singlefs-core，读它的构建脚本跑出来的 output 文件里 cargo::metadata 那一行，
# 并看编库的 rustc 命令行里 feature 的 cfg。每格用自己的编译目录。输出一行一格：name=t4 cell=<格> ...
set -u
root=$(pwd)
read_cell() { # <格名> <编译目录> <说明>
  local out
  out=$(find "$2" -path '*/build/singlefs-core-*/output' -type f 2>/dev/null | head -5)
  local line
  line=$(for f in $out; do grep -h 'metadata' "$f"; done | sort -u | tr '\n' ' ')
  local ran
  ran=$(echo "$out" | grep -c .)
  echo "name=t4 cell=$1 core_build_script_outputs=$ran metadata=[${line}] $3"
}
cell() { # <格名> <环境前缀…> -- <cargo 参数…>
  local name=$1; shift
  local target="$root/t4target-$name"; rm -rf "$target"
  local envs=(); while [ "$1" != "--" ]; do envs+=("$1"); shift; done; shift
  env "${envs[@]}" CARGO_TARGET_DIR="$target" cargo "$@" -vv --offline > "$root/t4log-$name.txt" 2>&1
  echo "name=t4 cell=$name cargo_rc=$? lib_cfg=[$(grep 'CARGO_CRATE_NAME=singlefs_core ' "$root/t4log-$name.txt" | head -1 | grep -o -- "--cfg 'feature=\"[a-z-]*\"'" | tr '\n' ' ')] script_cfg=[$(grep 'CARGO_CRATE_NAME=build_script_build' "$root/t4log-$name.txt" | grep 'singlefs-core' | head -1 | grep -o -- "--cfg 'feature=\"[a-z-]*\"'" | tr '\n' ' ')]"
  read_cell "$name" "$target" ""
}
cell plain A=1 -- build -p singlefs-core
cell feature A=1 -- build -p singlefs-core --features lookup-work-counters
cell harness_test A=1 -- test -p singlefs-harness --test publish_lookup_by_identity --no-run
cell rustflags_cfg RUSTFLAGS='--cfg feature="lookup-work-counters"' -- build -p singlefs-core
cell rustflags_cfg_with_target RUSTFLAGS='--cfg feature="lookup-work-counters"' -- build -p singlefs-core --target x86_64-unknown-linux-gnu
# stale：先不带变量编一次，再外带变量编同一个编译目录
target="$root/t4target-stale"; rm -rf "$target"
CARGO_TARGET_DIR="$target" cargo build -p singlefs-core -vv --offline > "$root/t4log-stale1.txt" 2>&1
CARGO_FEATURE_LOOKUP_WORK_COUNTERS=1 CARGO_TARGET_DIR="$target" cargo build -p singlefs-core -vv --offline > "$root/t4log-stale2.txt" 2>&1
echo "name=t4 cell=stale second_build_ran_core_build_script=$(grep -c 'Running.*singlefs-core-.*build-script-build' "$root/t4log-stale2.txt") second_build_recompiled_core=$(grep -c 'Compiling singlefs-core' "$root/t4log-stale2.txt")"
read_cell stale "$target" ""
