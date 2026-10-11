#!/usr/bin/env bash
# 正推腿 m3-batch23-code-r1 的复跑：CPU 侧探针（harness 档）、N5 与 N8 的测试和变异、E142 导出的拒绝路径、第 23 批变异行的锚点。
# 用法：rerun.sh <空目录> [anchors|n6|t13|t9|t10|fb|t3b|t1|lb|q17|n5|tests|bin|mut-n5|mut-n8|summary …]
#   不给后面的参数 = 全部按这个次序跑完再 summary。前台一次放不下时分几次给步名，最后单独给 summary。
#   环境：SINGLEFS_REPO（仓路径：取 run-with-memory-cap.sh、capped.sh 与快照 sha256 清单）；
#         FROZEN_TREE（冻结副本，默认 /tmp/claude-1000/m3-batch23-code-r1/tree）；THREADS（默认 7）。
#   编译 6G、跑 4G，都经 scripts/hosts/run-with-memory-cap.sh（显式上限，不写 auto）；用 <目录>/target 当编译目录。
#   第一次调用把冻结副本拷进 <目录>/work、核 13 份 sha256、把 probes/zs_probe.rs 放进 work 的 harness 测试目录。
# 比对：<目录>/summary.txt 与本目录的 expected-summary.txt 逐字相同（每步输出已去掉耗时）。
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO}"
dir="$(cd "$1" && pwd)"; shift
frozen="${FROZEN_TREE:-/tmp/claude-1000/m3-batch23-code-r1/tree}"
threads="${THREADS:-7}"
mkdir -p "$dir/out"
if [ ! -d "$dir/work" ]; then
  mkdir "$dir/work" && cp -r "$frozen/." "$dir/work/" || exit 3
  cp "$here/probes/zs_probe.rs" "$dir/work/crates/singlefs-harness/tests/zs_probe.rs" || exit 3
  (cd "$dir/work" && sha256sum -c "$SINGLEFS_REPO/research/prompts/m3-batch23-code-r1-snapshot/crates-sha256.txt" 2>&1 | grep -c ': OK$' | sed 's/^/SHA256_OK_OF_13 /') | tee "$dir/out/sha256.txt"
fi
export CARGO_TARGET_DIR="$dir/target"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="m3-batch23-code-r1 正推腿在自己的仓副本与编译目录里复跑，不与别的会话共用副本"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="派发提示要求编译 6G 跑 4G 显式写上限，不用 auto"
cap="nice -n 19 bash $SINGLEFS_REPO/scripts/hosts/run-with-memory-cap.sh"
capped="bash $SINGLEFS_REPO/scripts/hosts/capped.sh $threads"
build_prefix="$cap 6G $capped"
run_prefix="$cap 4G $capped"
strip_times() { sed -E 's/; finished in [0-9.]+s//'; }

# probe <步名> <过滤串>：编译探针（6G）再跑（4G），留下 ZS_PROBE 行、测试结果行
probe() {
  local step="$1" filter="$2"
  ( cd "$dir/work" && $build_prefix cargo test --offline --release -p singlefs-harness --test zs_probe --no-run > "$dir/out/$step.build.log" 2>&1; echo "build exit=$?" > "$dir/out/$step.txt"
    $run_prefix cargo test --offline --release -p singlefs-harness --test zs_probe -- "$filter" --nocapture --test-threads=1 > "$dir/out/$step.run.log" 2>&1; echo "run exit=$?" >> "$dir/out/$step.txt"
    grep -oE 'ZS_PROBE .*|^test result.*' "$dir/out/$step.run.log" | strip_times | sort >> "$dir/out/$step.txt" )
}

all_steps=(anchors n6 t13 t9 t10 fb t3b t1 lb q17 n5 tests bin mut-n5 mut-n8)
steps=("$@")
if [ ${#steps[@]} -eq 0 ]; then steps=("${all_steps[@]}" summary); fi
for step in "${steps[@]}"; do
  case "$step" in
    anchors) python3 "$here/anchors.py" "$dir/work" > "$dir/out/anchors.txt" 2>&1; echo "exit=$?" >> "$dir/out/anchors.txt" ;;
    n6) probe n6 zs_n6 ;;
    t13) probe t13 zs_t13 ;;
    t9) probe t9 zs_t9 ;;
    t10) probe t10 zs_t10 ;;
    fb) probe fb zs_first_batch ;;
    t3b) probe t3b zs_t3b ;;
    t1) probe t1 zs_t1_ ;;
    lb) probe lb zs_t11 ;;
    q17) probe q17 zs_queue ;;
    n5) probe n5 zs_n5 ;;
    tests)
      ( cd "$dir/work"
        for spec in "checker_known_bad_images an_abandoned_root_naming_another_object_birth" \
                    "the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times"; do
          set -- $spec
          $run_prefix cargo test --offline --release -p singlefs-harness --test "$1" -- "$2" > "$dir/out/tests.$1.log" 2>&1
          echo "exit=$? $1"
          grep -E '^test .* \.\.\. |^test result' "$dir/out/tests.$1.log" | sed -E 's/^(test [a-z_]{40})[a-z_]*/\1…/' | strip_times
        done ) > "$dir/out/tests.txt" 2>&1 ;;
    bin)
      ( cd "$dir/work" && $build_prefix cargo build --offline --release -p singlefs-checker-tier --bin e142_new_pool_file_creation_write_dump > "$dir/out/bin.build.log" 2>&1; echo "build exit=$?"
        binary="$dir/target/release/e142_new_pool_file_creation_write_dump"
        for bytes in 18446744073709551615 0 1125899906842624 1099511627776; do
          $cap 4G "$binary" --device-bytes "$bytes" > "$dir/out/bin.$bytes.out" 2> "$dir/out/bin.$bytes.err"
          echo "BIN device_bytes=$bytes exit=$? stdout_lines=$(wc -l < "$dir/out/bin.$bytes.out") stderr_first=$(grep -m1 "^e142_new_pool" "$dir/out/bin.$bytes.err" | cut -c1-150)"
        done ) > "$dir/out/bin.txt" 2>&1 ;;
    mut-n5)
      MUTATE_CMD_PREFIX="$run_prefix" python3 "$here/mutate_one.py" "$dir/work" "第23批 X I-9.29" > "$dir/out/mut-n5.txt" 2>&1 ;;
    mut-n8)
      MUTATE_CMD_PREFIX="$run_prefix" python3 "$here/mutate_one.py" "$dir/work" "删除批第 21 批 N3：可写挂载" > "$dir/out/mut-n8.txt" 2>&1
      MUTATE_CMD_PREFIX="$run_prefix" python3 "$here/mutate_one.py" "$dir/work" "删除批第 22 批 N5：可写挂载重建" >> "$dir/out/mut-n8.txt" 2>&1 ;;
    summary)
      {
        for f in sha256 anchors n6 t13 t9 t10 fb t3b t1 lb q17 n5 tests bin mut-n5 mut-n8; do
          if [ -f "$dir/out/$f.txt" ]; then echo "== $f"; cat "$dir/out/$f.txt"; else echo "== $f MISSING"; fi
        done
      } > "$dir/summary.txt"
      if cmp -s "$dir/summary.txt" "$here/expected-summary.txt"; then echo "SUMMARY_MATCHES lines=$(wc -l < "$dir/summary.txt")"; else echo "SUMMARY_DIFFERS"; diff "$dir/summary.txt" "$here/expected-summary.txt" | head -40; fi ;;
    *) echo "不认识的步名 $step"; exit 2 ;;
  esac
  echo "step $step done"
done
