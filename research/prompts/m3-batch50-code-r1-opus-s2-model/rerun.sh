#!/usr/bin/env bash
# m3-batch50-code-r1 攻方 s2 的复跑：按阶段跑，每个阶段一条前台命令跑得完（4 分钟内，k7 每条变异一个阶段）。
# 用法：SINGLEFS_REPO=<仓根> bash rerun.sh <工作目录（空目录或已跑过前几个阶段的同一个）> <阶段>
#   阶段依次：prepare build k5 k3 random random-knobs random-churn random-abandoned control scan
#             k7-m3-left-first k7-m3-floor-116 k7-m3-floor-118 k7-r3-carried-from-new-position
#             k7-checker-i934-no-lineage-filter k7-checker-i932-no-birth-before-death k7-checker-i932-no-death-at-most-version
#             direction crash compare
#   SINGLEFS_S2_TREE：被判的冻结副本（默认 ~/.local/share/singlefs/work/m3-batch50-code-r1/tree），只读、拷进工作目录再改。
# 每条编译与跑都经仓里的 run-with-memory-cap.sh（上限 10G，单个随机历史进程 2G）与 capped.sh（线程 8；并行的那几批每件 1 线程、8 件）。
# 每份变异副本各用自己的编译目录（工作目录下 target-mut-<名>）。产物写进工作目录的 outputs/，compare 阶段排序后与本目录 outputs/ 比。
set -u
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO（仓根）}"
WORK=${1:?工作目录}; STAGE=${2:?阶段}
MODEL=$(cd "$(dirname "$0")" && pwd)
TREE=${SINGLEFS_S2_TREE:-$HOME/.local/share/singlefs/work/m3-batch50-code-r1/tree}
CAP="$SINGLEFS_REPO/scripts/hosts/run-with-memory-cap.sh"
CAPPED="$SINGLEFS_REPO/scripts/hosts/capped.sh"
mkdir -p "$WORK/outputs"
OUT=$WORK/outputs
PROBES="opus_s2_probe_edge_unlink.rs opus_s2_probe_first_file_tree_identifier_watermark.rs opus_s2_probe_random_leaves.rs opus_s2_probe_random_leaves_abandoned.rs opus_s2_probe_direction.rs"

copy_tree() { # <目标>
  rsync -a --exclude target "$TREE/" "$1/"
  for p in $PROBES; do cp "$MODEL/$p" "$1/crates/singlefs-harness/tests/"; done
}
ctest() { # <仓副本> <编译目录> <上限> <线程> cargo test 的参数…
  local repo=$1 target=$2 cap=$3 threads=$4; shift 4
  (cd "$repo" && CARGO_TARGET_DIR=$target nice -n 19 bash "$CAP" "$cap" bash "$CAPPED" "$threads" cargo test --release --offline "$@")
}
# 一批随机历史：8 件并行，每件 5 段（或给的段数），各写自己的日志与退出码文件；最后单独一个 wait 等齐、数退出码文件。
batch() { # <名> <测试目标> <测试函数名或空> <起始种子> <每件段数> <步数> 环境…
  local name=$1 target=$2 filter=$3 from=$4 per=$5 steps=$6; shift 6
  mkdir -p "$WORK/$name"; rm -f "$WORK/$name"/$name-*.rc
  for k in 0 1 2 3 4 5 6 7; do
    { env "$@" S2_SEED_FROM=$((from + k * per)) S2_SEEDS=$per S2_STEPS=$steps \
        bash -c 'cd "$0" && CARGO_TARGET_DIR="$1" nice -n 19 bash "$2" 2G bash "$3" 1 cargo test --release --offline -p singlefs-harness --test "$4" -- --nocapture $5' \
        "$WORK/repo" "$WORK/target" "$CAP" "$CAPPED" "$target" "$filter" > "$WORK/$name/$name-$k.log" 2>&1; echo "$?" > "$WORK/$name/$name-$k.rc"; } &
  done
  wait
  local n; n=$(ls "$WORK/$name"/$name-*.rc | wc -l)
  [ "$n" = 8 ] || { echo "$name: 退出码文件 $n 个，不是 8 个，整批作废"; return 1; }
  for k in 0 1 2 3 4 5 6 7; do echo "$name-$k rc=$(cat "$WORK/$name/$name-$k.rc")"; done
}
case $STAGE in
  prepare)
    copy_tree "$WORK/repo"
    copy_tree "$WORK/repo-noret"; python3 "$MODEL/apply.py" "$MODEL/mutations.py" noret "$WORK/repo-noret"
    rm -rf "$WORK/crash-proto"; mkdir -p "$WORK/crash-proto/src" "$WORK/crash-proto/.cargo"
    sed "s#REPO#$WORK/repo#g" "$MODEL/crash-proto/Cargo.toml" > "$WORK/crash-proto/Cargo.toml"
    sed "s#REPO#$WORK/repo#g" "$MODEL/crash-proto/src/main.rs" > "$WORK/crash-proto/src/main.rs"
    cp "$WORK/repo/Cargo.lock" "$WORK/crash-proto/"; cp "$WORK/repo/.cargo/config.toml" "$WORK/crash-proto/.cargo/" ;;
  build)
    (cd "$WORK/repo" && CARGO_TARGET_DIR=$WORK/target nice -n 19 bash "$CAP" 10G bash "$CAPPED" 8 cargo build --release --offline -p singlefs-harness \
      --test opus_s2_probe_edge_unlink --test opus_s2_probe_first_file_tree_identifier_watermark --test opus_s2_probe_random_leaves \
      --test opus_s2_probe_random_leaves_abandoned --test opus_s2_probe_direction --test checker_known_bad_images)
    (cd "$WORK/repo-noret" && CARGO_TARGET_DIR=$WORK/target-noret nice -n 19 bash "$CAP" 10G bash "$CAPPED" 8 cargo build --release --offline -p singlefs-harness --test opus_s2_probe_edge_unlink) ;;
  k5)
    for n in 233 232; do
      S2_ROUNDS=0 S2_NEW=$n S2_DU_FROM=112 S2_DU_TO=116 ctest "$WORK/repo" "$WORK/target" 10G 8 -p singlefs-harness --test opus_s2_probe_edge_unlink -- --nocapture | grep '^name=' > "$OUT/k5-edge-main-new$n.txt"
      S2_ROUNDS=0 S2_NEW=$n S2_DU_FROM=206 S2_DU_TO=216 ctest "$WORK/repo-noret" "$WORK/target-noret" 10G 8 -p singlefs-harness --test opus_s2_probe_edge_unlink -- --nocapture | grep '^name=' > "$OUT/k5-edge-noret-new$n.txt"
    done
    ctest "$WORK/repo" "$WORK/target" 10G 8 -p singlefs-harness --test checker_known_bad_images | grep '^test result' > "$OUT/main-badimages.txt" ;;
  k3)
    ctest "$WORK/repo" "$WORK/target" 10G 8 -p singlefs-harness --test opus_s2_probe_first_file_tree_identifier_watermark -- --nocapture | grep '^name=' > "$OUT/first-file-watermark.txt" ;;
  random)
    batch b-random opus_s2_probe_random_leaves "" 100 5 40 S2_TRACE=1
    cat "$WORK"/b-random/b-random-*.log | grep -oE 'name=hist .*' | sed 's/^name=hist knobs=false /name=hist /' > "$OUT/random-leaves-hist.txt"
    cat "$WORK"/b-random/b-random-*.log | grep -E '^name=red' > "$OUT/random-leaves-red.txt" || true
    cat "$WORK"/b-random/b-random-*.log | grep -oE 'name=trace .*' | awk '{print $4}' | sort | uniq -c > "$OUT/random-leaves-op-counts.txt" ;;
  random-knobs)
    batch b-knobs opus_s2_probe_random_leaves "" 300 5 40 S2_KNOBS=1
    cat "$WORK"/b-knobs/b-knobs-*.log | grep -oE 'name=hist .*' > "$OUT/random-leaves-knobs-hist.txt"
    cat "$WORK"/b-knobs/b-knobs-*.log | grep -oE 'name=retirement-tree .*' > "$OUT/random-leaves-knobs-retirement-trees.txt" ;;
  random-churn)
    batch b-churn opus_s2_probe_random_leaves "" 400 3 25 S2_KNOBS=1 S2_CHURN=1
    cat "$WORK"/b-churn/b-churn-*.log | grep -oE 'name=hist .*' > "$OUT/random-leaves-knobs-churn-hist.txt"
    cat "$WORK"/b-churn/b-churn-*.log | grep -oE 'name=retirement-tree .*' > "$OUT/random-leaves-knobs-churn-retirement-trees.txt" ;;
  random-abandoned)
    batch b-abandoned opus_s2_probe_random_leaves_abandoned random_histories_over_several_inode_leaves_with_abandoned_roots 200 5 40 S2_DUMMY=1
    cat "$WORK"/b-abandoned/b-abandoned-*.log | grep -oE 'name=hist .*' > "$OUT/random-leaves-abandoned-hist.txt"
    cat "$WORK"/b-abandoned/b-abandoned-*.log | grep -E '^name=red' > "$OUT/random-leaves-abandoned-red.txt" || true ;;
  control)
    S2_NO_DELETE=1 S2_SEED_FROM=200 S2_SEEDS=12 S2_STEPS=40 ctest "$WORK/repo" "$WORK/target" 2G 1 -p singlefs-harness --test opus_s2_probe_random_leaves_abandoned -- --nocapture random_histories_over_several_inode_leaves_with_abandoned_roots | grep -oE 'name=(hist|red) .*' > "$OUT/abandoned-no-delete-control.txt" ;;
  scan)
    : > "$OUT/abandoned-suffix-scan.txt"
    for c in "1 233 233" "232 1 1" "232 2 300" "233 233 1" "465 233 300" "699 1 233" "240 5 3"; do
      set -- $c
      S2_K1=$1 S2_K2=$2 S2_K3=$3 ctest "$WORK/repo" "$WORK/target" 2G 1 -p singlefs-harness --test opus_s2_probe_random_leaves_abandoned -- --nocapture abandoned_creates_then_every_two_step_suffix | grep -oE 'name=scan.*' >> "$OUT/abandoned-suffix-scan.txt"
    done ;;
  k7-*)
    name=${STAGE#k7-}; repo=$WORK/repo-mut-$name; target=$WORK/target-mut-$name
    copy_tree "$repo"; python3 "$MODEL/apply.py" "$MODEL/mutations.py" "$name" "$repo"
    res="mutation=$name"
    ctest "$repo" "$target" 10G 8 -p singlefs-core --lib inode_tree > "$WORK/k7-$name-core.log" 2>&1; res="$res core-inode_tree rc=$?"
    for t in inode_leaf_absorb_and_borrow_on_the_unlink_path unlink_idempotence_criteria_and_rollback_to_the_emptied_version pool_checker_judges_the_deletion_state_of_every_walked_version checker_known_bad_images; do
      ctest "$repo" "$target" 10G 8 -p singlefs-harness --test $t > "$WORK/k7-$name-$t.log" 2>&1; res="$res $t rc=$?"
    done
    S2_SEED_FROM=100 S2_SEEDS=6 S2_STEPS=40 ctest "$repo" "$target" 10G 8 -p singlefs-harness --test opus_s2_probe_random_leaves -- --nocapture > "$WORK/k7-$name-probe.log" 2>&1
    res="$res probe rc=$? red=$(grep -c '^name=red' "$WORK/k7-$name-probe.log")"
    echo "$res" >> "$OUT/k7-mutations.txt"; echo "$res" ;;
  direction)
    ctest "$WORK/repo" "$WORK/target" 10G 8 -p singlefs-harness --test opus_s2_probe_direction -- --nocapture | grep '^name=' > "$OUT/k7-direction-main.txt"
    ctest "$WORK/repo-mut-m3-left-first" "$WORK/target-mut-m3-left-first" 10G 8 -p singlefs-harness --test opus_s2_probe_direction -- --nocapture | grep '^name=' > "$OUT/k7-direction-left-first.txt" ;;
  crash)
    cd "$WORK/crash-proto" || exit 2
    : > "$OUT/crash-plans-over-budget.txt"
    for s in absorb borrow lender-absorb reunlink-after-rollback; do
      S2_PLAN_ONLY=1 CARGO_TARGET_DIR=$WORK/target-proto nice -n 19 bash "$CAP" 4G bash "$CAPPED" 8 cargo run --release --offline -q -- $s | grep '^name=plan' >> "$OUT/crash-plans-over-budget.txt"
    done
    CARGO_TARGET_DIR=$WORK/target-proto nice -n 19 bash "$CAP" 8G bash "$CAPPED" 8 cargo run --release --offline -q -- rollback-after-absorb | grep '^name=' > "$OUT/crash-rollback-after-absorb.txt" ;;
  compare)
    bad=0
    for f in "$MODEL"/outputs/*.txt; do
      b=$(basename "$f"); [ -f "$OUT/$b" ] || { echo "缺 $b"; bad=1; continue; }
      if diff <(sed -E 's/ (elapsed_seconds|finished in)[= ][^ ]*//g' "$f" | sort) <(sed -E 's/ (elapsed_seconds|finished in)[= ][^ ]*//g' "$OUT/$b" | sort) > /dev/null; then echo "同 $b"; else echo "不同 $b"; bad=1; fi
    done
    exit $bad ;;
  *) echo "不认的阶段 $STAGE"; exit 2 ;;
esac
