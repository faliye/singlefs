#!/usr/bin/env bash
# 在 work/ 上把 harness 档的目标逐个跑一遍（名字带 layer0 的不跑；singlefs-checker-tier 整包是 checker 档，一样都不跑）。
# 用法：run-all.sh <输出目录> [目标名…]；不给目标名就跑全部。每个目标的退出码写进 <输出目录>/exit-codes.tsv。
set -uo pipefail
out=$1; shift
here=/tmp/claude-1000/impl-r2-fixes-b
main=<仓根>
mkdir -p "$out"
cd "$here/work"
run() {
  local name=$1; shift
  nice -n 19 bash "$main/research/scripts/run-with-memory-cap.sh" 8G bash "$main/research/scripts/capped.sh" 4 cargo test --offline "$@" > "$out/$name.log" 2>&1
  local code=$?
  printf '%s\t%s\n' "$name" "$code" >> "$out/exit-codes.tsv"
}
if [ $# -gt 0 ]; then
  for target in "$@"; do
    case "$target" in
      lib-*) run "$target" -p "${target#lib-}" --lib ;;
      *) run "$target" -p singlefs-harness --test "$target" ;;
    esac
  done
else
  run lib-singlefs-format -p singlefs-format --lib
  run lib-singlefs-core -p singlefs-core --lib
  run lib-singlefs-harness -p singlefs-harness --lib
  for file in crates/singlefs-harness/tests/*.rs; do
    target=$(basename "$file" .rs)
    case "$target" in *layer0*) continue ;; esac
    run "$target" -p singlefs-harness --test "$target"
  done
fi
echo "ALL-DONE $(date -u +%FT%TZ)" >> "$out/exit-codes.tsv"
