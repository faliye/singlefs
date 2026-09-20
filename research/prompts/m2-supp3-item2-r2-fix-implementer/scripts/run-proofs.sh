#!/usr/bin/env bash
# 变异证明：一份仓副本（自己的 target，debug），先跑不改动的基线，再逐条施加 proof-mutants/ 里的变异、跑它点名的每个测试二进制的整个、还原（拷回 + 摸 mtime）。
D=/tmp/claude-1000/m2-supp3-item2-r2-fix-implementer
C=$D/copies/proof
export CARGO_TARGET_DIR=$C/target
cd $C || exit 2
L=$D/logs/proof
mkdir -p $L
progress=$D/logs/proof-progress.txt
args_of() {
  case $1 in
    random-history) echo "-p singlefs-harness --test second_transaction_supplement_three_random_history" ;;
    step-four) echo "-p singlefs-harness --test second_transaction_step_four_rollback" ;;
    step-one) echo "-p singlefs-harness --test second_transaction_step_one_overwrite" ;;
    unequal-devices) echo "-p singlefs-harness --test second_transaction_supplement_two_unequal_devices" ;;
    lib) echo "-p singlefs-harness --lib" ;;
  esac
}
run_binary() { # $1 标签 $2 二进制
  local log=$L/$1--$2.log
  local started=$(date +%s)
  nice -n 19 cargo test $(args_of $2) > $log 2>&1
  local code=$?
  echo "$(date -u +%H:%M:%S) $1 $2 exit $code（$(( $(date +%s) - started )) 秒）；红：$(grep -E '^test .* \.\.\. FAILED$' $log | sed -E 's/^test (\S+) .*/\1/' | tr '\n' ' ')" >> $progress
}
echo "$(date -u +%H:%M:%S) 开跑" >> $progress
for binary in ${BASE_BINARIES:-random-history step-four step-one unequal-devices lib}; do
  run_binary base $binary
done
for m in ${MUTANTS:-$(ls $D/proof-mutants)}; do
  M=$D/proof-mutants/$m
  python3 $D/scripts/mutate.py apply $C "$(cat $M/file)" $M/old $M/new >> $progress 2>&1 || { echo "施加失败 $m" >> $progress; continue; }
  for binary in $(cat $M/binary); do
    run_binary $m $binary
  done
  python3 $D/scripts/mutate.py restore $C "$(cat $M/file)" >> $progress 2>&1
done
echo "$(date -u +%H:%M:%S) 全部跑完" >> $progress
