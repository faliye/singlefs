#!/usr/bin/env bash
# 攻方腿模型（m3-run-compare-split-r1）：在几棵探针树上算三种结论的代码摘要表（照 build.rs 同一套，judging_closure.rs 经 #[path] 引），
# 看「丙簇录入比对内核只由流水线引到」时内核改了、哪一把结论版本都不动；正对照：改由 crash_judge_gpu.rs 引到时 GPU 判器那一把跟着动。
# 用法：closure_probe_trees.sh <副本根：主工作区 + V…E5 六份补丁> <草稿目录>
# 只读副本；探针树建在草稿目录里；只编一个只用 std 的小程序（不编 checker 档）。
set -euo pipefail
copy_root="$1"; work="$2"
mkdir -p "$work/closure-probe/src"
cp "$(dirname "$0")/closure-probe/Cargo.toml" "$work/closure-probe/Cargo.toml"
sed "s|COPY_ROOT|$copy_root|" "$(dirname "$0")/closure-probe/src/main.rs.template" > "$work/closure-probe/src/main.rs"
(cd "$work/closure-probe" && CARGO_TARGET_DIR="$work/target-closure-probe" nice -n 19 cargo build --release --offline)
for tree in probe-base probe-kernel probe-kernel2 probe-kernel3 probe-kernel4; do
  mkdir -p "$work/$tree"
  rsync -a --exclude target "$copy_root/Cargo.toml" "$copy_root/Cargo.lock" "$work/$tree/"
  rsync -a --exclude target "$copy_root/crates" "$work/$tree/"
done
for tree in probe-kernel probe-kernel2 probe-kernel3 probe-kernel4; do
  source_directory="$work/$tree/crates/singlefs-checker-tier/src"
  printf '%s\n' '//! 探针：丙簇「录入比对」内核的一份占位（只为看判法闭包收不收它）。' \
    'pub const RECORD_COMPARE_SOURCE: &str = include_str!("crash_record_compare.wgsl");' > "$source_directory/crash_record_compare_gpu.rs"
  printf 'fn compare(a: u32, b: u32) -> bool { return a == b; }\n' > "$source_directory/crash_record_compare.wgsl"
  printf '\npub mod crash_record_compare_gpu;\n' >> "$source_directory/lib.rs"
  printf '\n#[allow(unused_imports)]\nuse crate::crash_record_compare_gpu::RECORD_COMPARE_SOURCE as _PROBE_RECORD_COMPARE_SOURCE;\n' >> "$source_directory/crash_amplification.rs"
done
for tree in probe-kernel3 probe-kernel4; do
  printf '\n#[allow(unused_imports)]\nuse crate::crash_record_compare_gpu::RECORD_COMPARE_SOURCE as _PROBE_RECORD_COMPARE_SOURCE;\n' >> "$work/$tree/crates/singlefs-checker-tier/src/crash_judge_gpu.rs"
done
for tree in probe-kernel2 probe-kernel4; do
  printf 'fn compare(a: u32, b: u32) -> bool { return a != b; }\n' > "$work/$tree/crates/singlefs-checker-tier/src/crash_record_compare.wgsl"
done
for tree in probe-base probe-kernel probe-kernel2 probe-kernel3 probe-kernel4; do
  "$work/target-closure-probe/release/closure-probe" "$work/$tree" > "$work/$tree.table"
  printf 'CLOSURE_PROBE tree=%s table_without_gpu_file_list_sha256=%s gpu_judge_digest=%s rows_naming_record_compare=%s\n' "$tree" \
    "$(grep -v '^GPU_JUDGE_CLOSURE_FILE' "$work/$tree.table" | sha256sum | cut -c1-64)" \
    "$(grep -P '^gpu_judge\t' "$work/$tree.table" | cut -f3)" "$(grep -c 'record_compare' "$work/$tree.table" || true)"
done
