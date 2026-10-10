#!/usr/bin/env bash
# 攻方腿 m3-formal-code-b25-r1-opus 的复跑。整条脚本经内存包装跑（脚本里不再套包装），在主仓根下：
#   bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/capped.sh 6 \
#     bash research/prompts/m3-formal-code-b25-r1-opus-model/tools/rerun.sh <草稿目录> <段…>
# 段：k1-targeted k1-long k1-bound k1-rollback-mutations k1-container-number k2-device k1-sweep
#   k1-sweep 1296 段历史，本机负载 15–30 时约 15–20 分钟，超过前台时限，单独起；别的段各 1–3 分钟。
# 冻结副本只读：先 rsync 到 <草稿目录>/tree（不带 target），只在那里改、编、跑。
set -euo pipefail
work=$1; shift
repo=$(cd "$(dirname "$0")/../../../.." && pwd)
model="$repo/research/prompts/m3-formal-code-b25-r1-opus-model"
frozen="$repo/research/target/e171-r1-frozen-tree"
tree="$work/tree"; target="$work/target"; out="$work/out"
mkdir -p "$work" "$out"
fresh_tree() {
  rm -rf "$tree"; mkdir -p "$tree"
  rsync -a --exclude target --exclude .git "$frozen/" "$tree/"
  cp "$model/opus_k1_leaf_container_histories.rs" "$tree/crates/singlefs-harness/tests/"
}
build_k1() {
  (cd "$tree" && CARGO_TARGET_DIR="$target" nice -n 19 cargo test --offline --release -p singlefs-harness \
     --test opus_k1_leaf_container_histories --test model_compares_inode_creations_across_two_inode_leaf_containers --no-run) \
     > "$out/build-k1.log" 2>&1
  ls -t "$target"/release/deps/opus_k1_leaf_container_histories-* | grep -v '\.d$' | head -1
}
run_k1() { # <清单> <日志> [环境…]
  local list=$1 log=$2; shift 2
  (cd "$tree" && env K1_HISTORIES="$list" "$@" nice -n 19 "$(build_k1)" --nocapture) > "$log" 2>&1 || true
  grep -a 'K1ROW' "$log" | sed 's/ alloc_compared=[0-9]* roots=[0-9]* ms=[0-9]*//'
}
for stage in "$@"; do
  echo "=== $stage"
  case $stage in
    k1-targeted) fresh_tree; run_k1 "$model/histories-targeted.tsv" "$out/k1-targeted.log" ;;
    k1-long) fresh_tree; run_k1 "$model/histories-long.tsv" "$out/k1-long.log"; run_k1 "$model/histories-big3000.tsv" "$out/k1-big3000.log" ;;
    k1-bound)
      fresh_tree; python3 "$model/tools/patch_model_bound_print.py" "$tree"
      run_k1 "$model/histories-overwrite-only.tsv" "$out/k1-bound.log"; grep -a OPUSBOUND "$out/k1-bound.log" | awk 'NR%2==1 || /txg=127[5-9]/' ;;
    k1-rollback-mutations)
      fresh_tree
      for m in drop-retain drop-extend none; do
        python3 "$model/tools/mutate_rollback.py" "$tree" "$m"
        build_k1 > /dev/null
        (cd "$tree" && env K1_HISTORIES="$model/histories-rollback-mutation.tsv" CARGO_TARGET_DIR="$target" nice -n 19 cargo test --offline --release \
           -p singlefs-harness --test opus_k1_leaf_container_histories --test model_compares_inode_creations_across_two_inode_leaf_containers -- --nocapture) \
           > "$out/k1-mut-$m.log" 2>&1 || true
        echo "-- $m"; grep -a 'K1ROW\|test result' "$out/k1-mut-$m.log" | sed 's/ alloc_compared=[0-9]* roots=[0-9]* ms=[0-9]*//'
      done ;;
    k1-container-number)
      fresh_tree; python3 "$model/tools/mutate_core_container_number.py" "$tree" apply
      echo "-- checker off"; run_k1 "$model/histories-container-number.tsv" "$out/k1-cn-0.log"
      echo "-- checker on"; run_k1 "$model/histories-container-number.tsv" "$out/k1-cn-1.log" K1_CHECKER=1
      echo "-- many_inodes_write_and_read"
      (cd "$tree" && CARGO_TARGET_DIR="$target" nice -n 19 cargo test --offline --release -p singlefs-harness --test many_inodes_write_and_read) \
        > "$out/k1-cn-many.log" 2>&1 || true
      grep -a '^test \|test result' "$out/k1-cn-many.log" ;;
    k2-device)
      fresh_tree
      dev() { (cd "$tree" && CARGO_TARGET_DIR="$target" nice -n 19 cargo build --offline --release -p singlefs-checker-tier \
                --bin e171_three_oracle_collusion_after_the_oracle_fixes) > "$out/build-dev.log" 2>&1; echo "$target/release/e171_three_oracle_collusion_after_the_oracle_fixes"; }
      run_dev() { local name=$1; shift; rm -rf "$out/q5-$name"
        (cd "$tree" && env "$@" nice -n 19 "$(dev)" confirmed-loss-form --output-directory "$out/q5-$name") > "$out/q5-$name.out" 2> "$out/q5-$name.err" && rc=0 || rc=$?
        echo "-- $name rc=$rc"; grep -a 'name=q5_equivalence_pairing\|name=q5_equivalence \|name=q5 \|name=verdict' "$out/q5-$name.out" | cut -c1-400
        echo "OPUSSTATE matches_class=false: $(grep -ac 'OPUSSTATE.*matches_class=false' "$out/q5-$name.out" || true)"; }
      run_dev base X=1
      python3 "$model/tools/patch_device_dump.py" "$tree"
      python3 "$model/tools/patch_checker_fault.py" "$tree"
      python3 "$model/tools/patch_device_no_root.py" "$tree"
      run_dev dump-k2 E171_OPUS_DUMP=1 E171_OPUS_PAIRING_K=2
      run_dev dump-k6 E171_OPUS_DUMP=1 E171_OPUS_PAIRING_K=6
      run_dev fault-k2 OPUS_O2_FAULT_SLOT=50253 E171_OPUS_DUMP=1 E171_OPUS_PAIRING_K=2
      run_dev fault-k6 OPUS_O2_FAULT_SLOT=50253 E171_OPUS_DUMP=1 E171_OPUS_PAIRING_K=6
      run_dev noroot OPUS_NO_ROOT=1
      grep -a OPUSWRITE "$out/q5-dump-k6.out" ;;
    k1-sweep)
      fresh_tree; bin=$(build_k1); rm -rf "$out/sweep"; python3 "$model/tools/gen_sweep.py" "$out/sweep" 216
      for b in $(seq 0 35); do parts=(); for i in 0 1 2 3 4 5; do parts+=("part$((b*6+i))"); done
        K1_BATCH_NO_INNER_CAP=1 bash "$model/tools/run_batch.sh" "$bin" "$out/sweep" "$b" "${parts[@]}" | tr '\n' ' '; echo; done
      echo "rows=$(cat "$out"/sweep/b*.log | grep -ac K1ROW) not_completed=$(cat "$out"/sweep/b*.log | grep -a K1ROW | grep -vc 'ending=Completed' || true)" ;;
    *) echo "认不出的段 $stage"; exit 2 ;;
  esac
done
