#!/usr/bin/env bash
# 复跑 m3-batch31-code-r1 攻方（Opus）模型。
# 环境变量：SINGLEFS_REPO 冻结副本根（有 Cargo.toml、Cargo.lock、crates/）；SINGLEFS_TOOLS 仓根（包装脚本
#   research/scripts/run-with-memory-cap.sh 与 research/scripts/capped.sh 从这里取，冻结副本里没有 research/）；THREADS 线程数（默认 10）。
# 用法：rerun.sh <工作目录> [臂…]，臂默认 base mut-fl0 fix-key（见 make_arm.py）；点名的臂在工作目录里不许已经跑过，
#   可以分几次各跑一条臂（一次前台跑不完时），summary.txt 每次按 base、mut-fl0、fix-key 的次序把已跑的臂重新拼一遍。同一时刻只起一条 cargo；编译 6G、跑 4G，经内存包装。
# 每条臂：拷一份 crates 到 <工作目录>/<臂>/，make_arm.py 改，编译目录 <工作目录>/target-<臂>；产出
#   <臂>.probe  排好序的 PROBE / PROBE-INFO 行；<臂>.model  expect_model.py 比那几行的输出（排好序：并行的测试打行次序不定）；
#   <臂>.kbi    base 与 fix-key 两条臂另跑整个 checker_known_bad_images 目标，test 结果行排序、去掉耗时。
# 汇总 <工作目录>/summary.txt（各臂的 .probe、.model、.kbi 依次拼起来）；compare.sh 拿它比 baseline/summary.txt。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
work="${1:?要一个空工作目录}"
shift
arms=("$@")
[ "${#arms[@]}" -gt 0 ] || arms=(base mut-fl0 fix-key)
: "${SINGLEFS_REPO:?SINGLEFS_REPO 指冻结副本根}"
: "${SINGLEFS_TOOLS:?SINGLEFS_TOOLS 指仓根（取包装脚本）}"
threads="${THREADS:-10}"
mkdir -p "$work"
for arm in "${arms[@]}"; do
  [ ! -e "$work/$arm" ] || { echo "这条臂已经跑过：$work/$arm（换一个空工作目录，或只点没跑过的臂）" >&2; exit 2; }
done
cap="$SINGLEFS_TOOLS/research/scripts/run-with-memory-cap.sh"
capped="$SINGLEFS_TOOLS/research/scripts/capped.sh"
for arm in "${arms[@]}"; do
  tree="$work/$arm"
  mkdir -p "$tree"
  cp "$SINGLEFS_REPO/Cargo.toml" "$SINGLEFS_REPO/Cargo.lock" "$tree/"
  cp -r "$SINGLEFS_REPO/crates" "$tree/crates"
  [ -d "$SINGLEFS_REPO/litmus" ] && cp -r "$SINGLEFS_REPO/litmus" "$tree/litmus"
  python3 -B "$here/make_arm.py" "$tree" "$arm"
  export CARGO_TARGET_DIR="$work/target-$arm"
  ( cd "$tree" && nice -n 19 bash "$cap" 6G bash "$capped" "$threads" \
      cargo test -p singlefs-harness --test checker_known_bad_images --no-run ) > "$work/$arm.build.log" 2>&1
  ( cd "$tree" && nice -n 19 bash "$cap" 4G bash "$capped" "$threads" \
      cargo test -p singlefs-harness --test checker_known_bad_images -- opus_b31 --nocapture --test-threads "$threads" ) \
      > "$work/$arm.run.log" 2>&1
  grep -E '^PROBE' "$work/$arm.run.log" | LC_ALL=C sort > "$work/$arm.probe"
  python3 -B "$here/expect_model.py" "$work/$arm.run.log" | LC_ALL=C sort > "$work/$arm.model"
  if [ "$arm" = base ] || [ "$arm" = fix-key ]; then
    set +e
    ( cd "$tree" && nice -n 19 bash "$cap" 4G bash "$capped" "$threads" \
        cargo test -p singlefs-harness --test checker_known_bad_images -- --test-threads "$threads" ) > "$work/$arm.kbi.log" 2>&1
    echo "exit=$?" > "$work/$arm.kbi"
    set -e
    grep -E '^test .* \.\.\. ' "$work/$arm.kbi.log" | LC_ALL=C sort >> "$work/$arm.kbi"
    grep -E '^test result:' "$work/$arm.kbi.log" | sed -E 's/; finished in [0-9.]+s//' >> "$work/$arm.kbi"
  fi
  unset CARGO_TARGET_DIR
done
: > "$work/summary.txt"
for arm in base mut-fl0 fix-key; do
  for part in probe model kbi; do
    [ -f "$work/$arm.$part" ] || continue
    echo "== $arm.$part" >> "$work/summary.txt"
    cat "$work/$arm.$part" >> "$work/summary.txt"
  done
done
echo "RERUN done: $work/summary.txt（$(wc -l < "$work/summary.txt") 行）"
