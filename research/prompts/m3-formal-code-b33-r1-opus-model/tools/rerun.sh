#!/usr/bin/env bash
# 第 33 批代码轮攻方（opus）复跑：每一段从冻结副本拷一棵新树（新目录、新编译目录，不复用旧产物），只在拷出来的树上改、编、跑。
# 用法：rerun.sh <工作目录> [段…]；段：x fix p3 p2 c551（不给就全跑）。输出写 <工作目录>/out/。
# 环境：FROZEN（默认 /tmp/claude-1000/m3-formal-code-b33-r1/tree）、CAP（线程上限，默认 10）、MEMORY（默认 8G）。
# harness 档与攻方自己的原型 crate；不跑 checker 档测试、不跑 GPU、不跑变异整表。
set -u
work="$1"; shift
steps=("$@"); [ "${#steps[@]}" -eq 0 ] && steps=(x fix p3 p2 c551)
tools="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
model="$(cd "$tools/.." && pwd)"
repo="$(cd "$model/../../.." && pwd)"
frozen="${FROZEN:-/tmp/claude-1000/m3-formal-code-b33-r1/tree}"
cap="${CAP:-10}"; memory="${MEMORY:-8G}"
out="$work/out"; mkdir -p "$out"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-攻方副本独立编译目录从零编，派发提示给的显式上限 8G}"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-三方攻方腿复跑各跑各的副本}"
wrap() { nice -n 19 bash "$repo/scripts/run-with-memory-cap.sh" "$memory" bash "$repo/scripts/capped.sh" "$cap" "$@"; }
fresh() { rm -rf "$work/$1"; mkdir -p "$work/$1"; rsync -a "$frozen/" "$work/$1/tree/"; ( cd "$frozen" && sha256sum -c ../crates-sha256.txt >/dev/null ) || echo "RERUN 冻结副本 sha256 对不上：$frozen" >&2; }
compiled() { grep -c "Compiling $2" "$1"; }
for step in "${steps[@]}"; do
  case "$step" in
    x)
      fresh x && python3 "$tools/patch_builder.py" "$work/x/tree" "$model"
      ( cd "$work/x/tree" && CARGO_TARGET_DIR="$work/x/target" wrap cargo test --offline -p singlefs-harness --test checker_known_bad_images -- b33x --nocapture --test-threads 1 ) > "$out/x.log" 2>&1
      echo "RERUN x rc=$? compiled_harness=$(compiled "$out/x.log" singlefs-harness)"; grep -E '^B33X (label|mount)' "$out/x.log" ;;
    fix)
      fresh fix && python3 "$tools/patch_fix_rv.py" "$work/fix/tree" && python3 "$tools/patch_builder.py" "$work/fix/tree" "$model"
      ( cd "$work/fix/tree" && CARGO_TARGET_DIR="$work/fix/target" wrap cargo test --offline -p singlefs-harness --test checker_known_bad_images -- --nocapture --test-threads 4 ) > "$out/fix.log" 2>&1
      echo "RERUN fix rc=$? compiled_checker=$(compiled "$out/fix.log" 'singlefs-checker ')"; grep -E '^B33X (label|mount)|^test result' "$out/fix.log" ;;
    p3)
      fresh p3
      ( cd "$work/p3/tree" && wrap bash "$tools/p3_run_batch.sh" "$out/p3" 1 3 baseline row2790 row2791 && wrap bash "$tools/p3_run_batch.sh" "$out/p3" 2 3 floor_plus_one floor_minus_one row105 \
        && wrap bash "$tools/p3_run_batch.sh" "$out/p3" 3 3 row2792 row2788 row2789 && wrap bash "$tools/p3_run_batch.sh" "$out/p3" 4 3 reuse_keeps_old_generation reclaim_device_zero_only ) > "$out/p3.log" 2>&1
      echo "RERUN p3 rc=$?"; grep -E '^P3' "$out/p3.log" ;;
    p2)
      fresh p2 && python3 "$tools/install_proto.py" "$work/p2/tree" "$model"
      ( cd "$work/p2/tree" && CARGO_TARGET_DIR="$work/p2/target" wrap cargo build --offline --release -p opus-b33-proto ) > "$out/p2-build.log" 2>&1
      echo "RERUN p2-build rc=$? compiled_proto=$(compiled "$out/p2-build.log" opus-b33-proto)"
      ( cd "$work/p2/tree" && B33P_RANDOM_HISTORIES=0 wrap "$work/p2/target/release/b33_treeless_crash_states" ) > "$out/p2-scripted.log" 2>&1
      for from_and_count in 0:4 4:2; do from=${from_and_count%%:*}; count=${from_and_count##*:}; ( cd "$work/p2/tree" && B33P_SCRIPTED=0 B33P_RANDOM_FROM=$from B33P_RANDOM_HISTORIES=$count wrap "$work/p2/target/release/b33_treeless_crash_states" ) > "$out/p2-random-$from.log" 2>&1; done
      grep -h -E '^B33P' "$out"/p2-scripted.log "$out"/p2-random-*.log | sed -E 's/outcomes=\[[^]]*\] //' ;;
    c551)
      fresh c551 && python3 "$tools/patch_c551_probe.py" "$work/c551/tree"
      ( cd "$work/c551/tree" && CARGO_TARGET_DIR="$work/c551/target" wrap cargo test --offline -p singlefs-harness --test overwrite_in_one_instance -- allocated_statistic_equals repeated_overwrites_go_past --nocapture --test-threads 1 ) > "$out/c551.log" 2>&1
      echo "RERUN c551 rc=$? compiled_harness=$(compiled "$out/c551.log" singlefs-harness)"; grep 'B33C551 end' "$out/c551.log"; grep 'B33C551 per_version' "$out/c551.log" | sed -E 's/.*not_holding=/not_holding=/' | sort | uniq -c ;;
    *) echo "不认得的段 $step" >&2; exit 2 ;;
  esac
done
