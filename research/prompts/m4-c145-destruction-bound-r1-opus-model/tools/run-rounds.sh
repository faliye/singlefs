#!/usr/bin/env bash
# 攻方腿在第二台跑的那一段（m4-c145-destruction-bound-r1）：在这棵冻结副本的拷贝里跑基线与三个变体，每一趟四个测试目标，
# 原样输出与退出码写进 out/。从仓树根起：bash <这棵拷贝>/peer-tools/run-rounds.sh <这棵拷贝的路径：相对仓树根，或绝对路径>
set -uo pipefail
REPOSITORY_TREE="$PWD"
case "$1" in /*) COPY="$1" ;; *) COPY="$REPOSITORY_TREE/$1" ;; esac
cd "$COPY" || exit 2
mkdir -p out
: > out/exit-codes.txt
CAPPED="$REPOSITORY_TREE/scripts/hosts/capped.sh"
run_targets() {
  local round="$1" rc
  rc=0; SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 attack_upper_bound_of_an_open_destruction_intent; 范围=1 个测试目标; 为什么=攻方腿：上界那几段历史逐版的纪元与池级 checker、续做与纪元改坏的世界' CARGO_TARGET_DIR=target nice -n 19 bash "$CAPPED" 31 cargo test -p singlefs-harness --test attack_upper_bound_of_an_open_destruction_intent -- --nocapture --test-threads 1 > "out/$round-attack.txt" 2>&1 || rc=$?
  echo "$round attack_upper_bound_of_an_open_destruction_intent $rc" >> out/exit-codes.txt
  rc=0; SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 snapshot_release_and_destroy_through_the_deadlist; 范围=1 个测试目标; 为什么=攻方腿：看删快照的已有用例对这一趟的变体红不红' CARGO_TARGET_DIR=target nice -n 19 bash "$CAPPED" 31 cargo test -p singlefs-harness --test snapshot_release_and_destroy_through_the_deadlist > "out/$round-destroy.txt" 2>&1 || rc=$?
  echo "$round snapshot_release_and_destroy_through_the_deadlist $rc" >> out/exit-codes.txt
  rc=0; SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 checker_snapshot_space_accounting_must_red_worlds; 范围=1 个测试目标; 为什么=攻方腿：看快照记账的必红世界对这一趟的变体红不红' CARGO_TARGET_DIR=target nice -n 19 bash "$CAPPED" 31 cargo test -p singlefs-harness --test checker_snapshot_space_accounting_must_red_worlds > "out/$round-must-red.txt" 2>&1 || rc=$?
  echo "$round checker_snapshot_space_accounting_must_red_worlds $rc" >> out/exit-codes.txt
  rc=0; SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-checker 的测试目标 lib（筛 walk::tests）; 范围=1 个测试目标; 为什么=攻方腿：看池级 checker 走读那一组单测对这一趟的变体红不红' CARGO_TARGET_DIR=target nice -n 19 bash "$CAPPED" 31 cargo test -p singlefs-checker --lib -- walk::tests > "out/$round-checker-lib.txt" 2>&1 || rc=$?
  echo "$round singlefs-checker-lib-walk-tests $rc" >> out/exit-codes.txt
}
run_targets baseline
for mutant in core_hi_published_at_the_batch core_hi_lo_plus_one checker_bound_lo_plus_one; do
  cp peer-tools/pristine/transaction.rs crates/singlefs-core/src/transaction.rs
  cp peer-tools/pristine/walk.rs crates/singlefs-checker/src/walk.rs
  python3 -I peer-tools/mutate.py . "$mutant" >> out/mutants-applied.txt 2>&1 || echo "$mutant mutate-failed" >> out/exit-codes.txt
  run_targets "$mutant"
done
cp peer-tools/pristine/transaction.rs crates/singlefs-core/src/transaction.rs
cp peer-tools/pristine/walk.rs crates/singlefs-checker/src/walk.rs
cat out/exit-codes.txt
rm -rf target
