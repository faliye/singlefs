#!/usr/bin/env bash
# m3-batch50-code-r1 攻方腿（Opus）的复跑：在一个空目录里从仓的 crates/ 拷副本、装上探针与变异、编、跑、与 expected/ 比。
# 用法：SINGLEFS_REPO=<仓根> bash rerun.sh <空的工作目录> <阶段>
#   阶段按次序：prepare build edge closed churn-b2f-even churn-b2f-odd churn-b3-even churn-b3-odd m2-build m2-run m3 m4-build m4-run m5-build m5-run compare
#   每个阶段在前台跑得完（churn 四个阶段各 8 个进程并行、每个进程一个种子，本机空闲时约 80 秒、负载 40 时约 160 秒）；all 依次跑全部。
# 每个变异各用自己的编译目录（target-M2 … target-M5）：几份副本共用一个编译目录时 cargo 按相对路径与时间戳判新旧，会拿上一份副本编出的东西当这一份的。
# 内存：每条 cargo 与每个测试二进制都经 $SINGLEFS_REPO/scripts/run-with-memory-cap.sh（cargo 10G，探针 1–8G），线程经 capped.sh 8。
# 盘：全是内存稀疏盘；只有 m5-run 跑的仓里现成用例按它们自己的写法在 TMPDIR 下建稀疏镜像文件，TMPDIR 指到工作目录里。
# compare：expected/ 里每份与 out/ 里同名那份去掉「finished in …」之后排序再比，逐份报一致 / 不一致，有一份不一致退 1。
set -euo pipefail
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO=<仓根>}"
W=$(realpath "$1"); STAGE=$2
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
CAP="$SINGLEFS_REPO/scripts/run-with-memory-cap.sh"
CAPPED="$SINGLEFS_REPO/scripts/capped.sh"
OUT=$W/out; mkdir -p "$OUT"
cargo_in() { local dir=$1 target=$2; shift 2; (cd "$dir" && CARGO_TARGET_DIR=$target nice -n 19 bash "$CAPPED" 8 bash "$CAP" 10G cargo "$@"); }
binary() { ls -t "$1"/release/deps/"$2"-* | grep -v '\.d$' | head -n 1; }
run_bin() { local dir=$1 cap=$2; shift 2; (cd "$dir" && nice -n 19 bash "$CAP" "$cap" "$@"); }
edge_filter() { grep -v 'finished in' | grep -E '^(BASE|ROW|  |LAST_OK)'; }
churn_batch() {
  # 8 个进程并行、每个进程一段（一个种子）：种子 base + offset + 2i，i = 0..7；一行一个种子
  local name=$1 base=$2 offset=$3 extra=$4 bin; bin=$(binary "$W/target" opus_probe_churn)
  rm -f "$W/$name"-*.rc
  for i in 0 1 2 3 4 5 6 7; do
    { env $extra CHURN_SEEDS=$((base + offset + 2 * i))..$((base + offset + 2 * i + 1)) CHURN_STEPS=40 nice -n 19 bash "$CAP" 1G "$bin" --nocapture > "$W/$name-$i.log" 2>&1; echo "$?" > "$W/$name-$i.rc"; } &
  done
  wait
  [ "$(ls "$W/$name"-*.rc | wc -l)" = 8 ] || { echo "$name：退出码文件不是 8 份，整批作废"; exit 1; }
  for i in 0 1 2 3 4 5 6 7; do
    [ "$(cat "$W/$name-$i.rc")" = 0 ] || { echo "$name 第 $i 件退 $(cat "$W/$name-$i.rc")"; exit 1; }
    echo "batch=${name%-*} seed=$((base + offset + 2 * i)) $(grep TOTAL "$W/$name-$i.log")"
  done >> "$OUT/churn-totals.out"
}
case "$STAGE" in
  prepare)
    mkdir -p "$W/copy"
    rsync -a --exclude target --exclude .git "$SINGLEFS_REPO/Cargo.toml" "$SINGLEFS_REPO/Cargo.lock" "$SINGLEFS_REPO/.cargo" "$SINGLEFS_REPO/crates" "$SINGLEFS_REPO/litmus" "$W/copy/"
    cp "$HERE"/probes/*.rs "$W/copy/crates/singlefs-harness/tests/"
    for m in M2 M3 M4 M5; do
      rsync -a "$W/copy/" "$W/copy-$m/"
      python3 -I "$HERE/apply_mutation.py" "$W/copy-$m" "$m"
    done ;;
  build)
    cargo_in "$W/copy" "$W/target" test --release -p singlefs-harness --test opus_probe_unlink_at_the_edge --test opus_probe_churn --test opus_probe_container_closed_early --no-run ;;
  edge)
    PROBE_DU=$(seq -s, 95 120) run_bin "$W/copy/crates/singlefs-harness" 2G "$(binary "$W/target" opus_probe_unlink_at_the_edge)" --nocapture --test-threads=1 | edge_filter > "$OUT/edge-orig.out" ;;
  closed)
    run_bin "$W/copy/crates/singlefs-harness" 1G "$(binary "$W/target" opus_probe_container_closed_early)" --nocapture --test-threads=1 \
      | grep -E 'CASE|containers|violations|VIOLATION' | sed 's/^test [a-z_]* \.\.\. //' > "$OUT/closed-early.out" ;;
  churn-b2f-even) churn_batch b2f-even 200 0 "" ;;
  churn-b2f-odd) churn_batch b2f-odd 200 1 "" ;;
  churn-b3-even) churn_batch b3-even 300 0 "CHURN_CAPPED=1 CHURN_TOMB=815" ;;
  churn-b3-odd) churn_batch b3-odd 300 1 "CHURN_CAPPED=1 CHURN_TOMB=815" ;;
  m2-build) cargo_in "$W/copy-M2" "$W/target-M2" test --release -p singlefs-harness --test opus_probe_unlink_at_the_edge --no-run ;;
  m2-run)
    PROBE_DU=$(seq -s, 160 200) run_bin "$W/copy-M2/crates/singlefs-harness" 2G "$(binary "$W/target-M2" opus_probe_unlink_at_the_edge)" --nocapture --test-threads=1 | edge_filter > "$OUT/edge-mut.out" ;;
  m3)
    cargo_in "$W/copy-M3" "$W/target-M3" test --release -p singlefs-core --lib --no-run
    { run_bin "$W/copy-M3/crates/singlefs-core" 4G "$(binary "$W/target-M3" singlefs_core)" --test-threads=8 || true; } | grep -E '^test result|FAILED$' | grep -v '^failures' > "$OUT/m3-core-lib.out" ;;
  m4-build)
    cargo_in "$W/copy-M4" "$W/target-M4" test --release -p singlefs-core --lib --no-run
    cargo_in "$W/copy-M4" "$W/target-M4" test --release -p singlefs-harness --test inode_leaf_absorb_and_borrow_on_the_unlink_path --no-run ;;
  m4-run)
    run_bin "$W/copy-M4/crates/singlefs-core" 4G "$(binary "$W/target-M4" singlefs_core)" --test-threads=8 | grep -E '^test result' > "$OUT/m4-core-lib.out"
    run_bin "$W/copy-M4/crates/singlefs-harness" 2G "$(binary "$W/target-M4" inode_leaf_absorb_and_borrow_on_the_unlink_path)" --test-threads=4 | grep -E '^test ' > "$OUT/m4-absorb.out" ;;
  m5-build) cargo_in "$W/copy-M5" "$W/target-M5" test --release -p singlefs-harness --test rollback_by_a_forward_publish --test random_histories --no-run ;;
  m5-run)
    mkdir -p "$W/tmp"
    TMPDIR=$W/tmp run_bin "$W/copy-M5/crates/singlefs-harness" 8G "$(binary "$W/target-M5" rollback_by_a_forward_publish)" --include-ignored --test-threads=8 | grep -E '^test result' > "$OUT/m5-rollback-all.out"
    TMPDIR=$W/tmp run_bin "$W/copy-M5/crates/singlefs-harness" 8G "$(binary "$W/target-M5" random_histories)" --include-ignored --exact rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms --test-threads=2 | grep -E '^test |^test result' > "$OUT/m5-random.out" ;;
  compare)
    status=0
    for expected in "$HERE"/expected/*.out; do
      name=$(basename "$expected")
      if diff <(sed 's/finished in [0-9.]*s//' "$expected" | sort) <(sed 's/finished in [0-9.]*s//' "$OUT/$name" | sort) > /dev/null; then
        echo "一致 $name"
      else
        echo "不一致 $name"; status=1
      fi
    done
    exit "$status" ;;
  all)
    for s in prepare build edge closed churn-b2f-even churn-b2f-odd churn-b3-even churn-b3-odd m2-build m2-run m3 m4-build m4-run m5-build m5-run compare; do bash "$0" "$W" "$s"; done ;;
  *) echo "不认的阶段：$STAGE"; exit 2 ;;
esac
