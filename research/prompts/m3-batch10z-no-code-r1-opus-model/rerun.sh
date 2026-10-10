#!/usr/bin/env bash
# m3-batch10z-no-code-r1 攻方复跑。工作目录仓根；WORK 给一个空的草稿目录（不在仓里）。线程上限 8，每件内存 1500M（一批至多 8 件），编译不跑代码。
#   WORK=~/.local/share/singlefs/work/<自己的目录> bash research/prompts/m3-batch10z-no-code-r1-opus-model/rerun.sh
# 做的事：冻结副本 rsync 三份（copy：入库原样加探针；pre14：把第十四块 core 两份文件的改动倒推回去再加同样的探针；k4copy：K4 遮蔽写法），
# release 编探针与 harness 档几个目标，按批跑（件表在 jobs/），汇总，与 runs-summary/ 逐份比。只跑 harness 档与 core 库单测、cargo check，不跑 checker 档测试。
set -euo pipefail
ROOT=$(pwd)
MD=$ROOT/research/prompts/m3-batch10z-no-code-r1-opus-model
FROZEN=${FROZEN:-$HOME/.local/share/singlefs/work/m3-batch10z-no-code-r1/tree}
WORK=${WORK:?给一个草稿目录}
CAP="$ROOT/research/scripts/run-with-memory-cap.sh"
CAPPED="$ROOT/research/scripts/capped.sh"
mkdir -p "$WORK/runs" "$WORK/summary"
for t in copy pre14 k4copy; do
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/$t/"
  mkdir -p "$WORK/$t/.cargo" && cp "$ROOT/.cargo/config.toml" "$WORK/$t/.cargo/config.toml"
done
awk 'NR>=893 && NR<=1402' "$ROOT/research/prompts/_m3-batch10z-no-code-r1-diff.md" > "$WORK/block14-core.diff"
head -1 "$WORK/block14-core.diff" | grep -q '^--- a/crates/singlefs-core/src/allocator.rs$' || { echo "改动全文第 893 行不是第十四块 allocator.rs 那一段的开头"; exit 3; }
( cd "$WORK/pre14" && patch -p1 -R -s < "$WORK/block14-core.diff" )
python3 "$MD/src/patch_probe.py" "$WORK/copy" today
python3 "$MD/src/patch_probe.py" "$WORK/pre14" pre14
python3 "$MD/src/gpu_patch.py" "$WORK/copy" "$MD/src"
cp "$MD/src/opus_no_probe.rs" "$WORK/copy/crates/singlefs-harness/tests/"
cp "$MD/src/opus_no_probe.rs" "$WORK/pre14/crates/singlefs-harness/tests/"
cp "$WORK/k4copy/crates/singlefs-checker-tier/src/gpu_unit_checks.rs" "$WORK/gpu_unit_checks.orig.rs"
K2T="--test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table --test reuse_after_raising_the_floor --test unreadable_abandoned_root_slot --test shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring --test writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing --test writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero --test writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten --test writable_mount_keeps_a_released_slot_named_by_an_unapplied_record_isolated_until_its_root_ring_slot_is_overwritten --test model_answers_unmount_with_the_floor_at_the_current_version"
( cd "$WORK/copy" && CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$CAPPED" 8 cargo test --offline --release -p singlefs-harness --test opus_no_probe $K2T --lib --no-run > "$WORK/build-copy.log" 2>&1 )
( cd "$WORK/copy" && CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$CAPPED" 8 cargo test --offline --release -p singlefs-core --lib --no-run > "$WORK/build-core.log" 2>&1 )
( cd "$WORK/pre14" && CARGO_TARGET_DIR="$WORK/target-pre14" nice -n 19 bash "$CAPPED" 8 cargo build --offline --release -p singlefs-harness --test opus_no_probe > "$WORK/build-pre14.log" 2>&1 )
# 第十四块：随机历史各批（batch.sh 每件各经内存包装起）
for b in s1 s2 s3 s4 m1a m1b u1 u2 n1; do
  nice -n 19 bash "$CAP" 12G bash "$MD/src/batch.sh" "$WORK" "$b" "$MD/jobs/jobs-$b.txt"
done
cd "$WORK"
for b in s1 s2 s3 s4; do python3 "$MD/src/pair.py" "$b" runs/"$b"-*.log > "summary/pair-$b.txt"; done
python3 "$MD/src/compare.py" today pre14 runs/n1-1.log runs/n1-3.log -- runs/n1-2.log runs/n1-4.log > summary/pair-n1.txt
python3 "$MD/src/classify_red.py" runs/s1-1.log runs/s1-2.log runs/s1-3.log runs/s1-4.log > summary/red-classes-s1-today.txt
python3 "$MD/src/classify_red.py" runs/n1-1.log runs/n1-3.log > summary/red-classes-n1-today.txt
python3 "$MD/src/classify_red.py" runs/n1-2.log runs/n1-4.log > summary/red-classes-n1-pre14.txt
for spec in "raise_plain m1a 1 2 3 4" "unmount_plain m1a 5 6 7 8" "held_plain m1b 1 2 3 4" "first_slot_only m1b 5 6 7 8"; do
  read -r arm b i1 i2 i3 i4 <<<"$spec"
  python3 "$MD/src/compare.py" today "$arm" runs/s1-1.log runs/s1-2.log runs/s1-3.log runs/s1-4.log -- runs/"$b-$i1".log runs/"$b-$i2".log runs/"$b-$i3".log runs/"$b-$i4".log | grep -v CMPDIFF
done > summary/arms-m1-vs-today.txt
{
  for i in 1 2; do arm=$([[ $i == 1 ]] && echo today || echo unmount_plain); echo "U1 arm=$arm histories=$(grep 'S3 label' runs/u1-$i.log | grep -vc record=skipped) exposed_right_after_unmount=$(grep -c 'u:exp' runs/u1-$i.log) red=$(grep -c 'hit=true' runs/u1-$i.log)"; done
  for i in 1 2 3 4 5 6; do a=$(sed -n "${i}p" "$MD/jobs/jobs-u2.txt" | grep -o 'OPUS_NO_ARM=[^ ]*' || echo today); echo "U2 job=$i $a $(sed -n "${i}p" "$MD/jobs/jobs-u2.txt" | grep -o 'S3_SLOTS=[0-9]*') histories=$(grep 'S3 label' runs/u2-$i.log | grep -vc record=skipped) exposed_right_after_unmount=$(grep -c 'u:exp' runs/u2-$i.log) red=$(grep -c 'hit=true' runs/u2-$i.log)"; done
} > summary/unmount-branch.txt
# 第十四块 K2：入库用例在几条臂下（臂只看环境变量 OPUS_NO_ARM）
grep -o 'Executable tests/[^ ]* ([^)]*)' build-copy.log | grep -v opus_no_probe | grep -o '(/[^)]*)' | tr -d '()' > test-bins.txt
CB=$(grep -o 'Executable unittests src/lib.rs ([^)]*)' build-core.log | grep -o '(/[^)]*)' | tr -d '()')
{
  while read -r tb; do
    nice -n 19 bash "$CAP" 1500M env OPUS_NO_ARM=unmount_plain bash "$CAPPED" 1 "$tb" --include-ignored > runs/k2u.log 2>&1 && rc=0 || rc=$?
    echo "K2U arm=unmount_plain $(basename "$tb" | sed 's/-[0-9a-f]*$//') rc=$rc $(grep -h '^test result' runs/k2u.log | tail -1)"
  done < test-bins.txt
  nice -n 19 bash "$CAP" 6G env OPUS_NO_ARM=unmount_plain bash "$CAPPED" 8 "$CB" > runs/k2u-core.log 2>&1 || true
  echo "K2U arm=unmount_plain singlefs_core-lib $(grep '^test result' runs/k2u-core.log)"
  TB=$(grep raise_floor_takes test-bins.txt)
  for arm in isoall acczero isoall+acczero none; do
    nice -n 19 bash "$CAP" 1500M env OPUS_NO_ARM=$arm bash "$CAPPED" 1 "$TB" --include-ignored > runs/k2a.log 2>&1 && rc=0 || rc=$?
    echo "K2A arm=$arm rc=$rc $(grep '^test result' runs/k2a.log) failed:$(grep -E '^test .* FAILED$' runs/k2a.log | sed 's/^test //; s/ ... FAILED//' | tr '\n' ' ')"
  done
} > summary/k2-tests-under-arms.txt
# 第十五块：两张卡的随机模型（今天 / 第十五块之前的走法）、并发压测、借之前那一步的七条臂下库单测
L=$(grep -o 'Executable unittests src/lib.rs ([^)]*singlefs_harness[^)]*)' build-copy.log | grep -o '(/[^)]*)' | tr -d '()')
for arm in today listed_only; do
  nice -n 19 bash "$CAP" 2G env OPUS_NO_GPU_ARM=$arm OPUS_NO_GPU_SEEDS=3000 OPUS_NO_GPU_STEPS=300 bash "$CAPPED" 1 "$L" --nocapture --exact gpu_memory_ledger::tests::opus_no_random_two_card_pool_model > runs/gpu2-$arm.log 2>&1
  grep -o 'OPUS_NO_GPU .*' runs/gpu2-$arm.log
done > summary/gpu-two-card-model.txt
nice -n 19 bash "$CAP" 2G env OPUS_NO_THREADS=8 OPUS_NO_ROUNDS=5000 bash "$CAPPED" 8 "$L" --nocapture --exact gpu_memory_ledger::tests::opus_no_two_card_early_lost_mark_racing_the_restart > runs/gpu-stress.log 2>&1
grep -o 'OPUS_NO_STRESS.*' runs/gpu-stress.log > summary/gpu-two-card-stress.txt
for arm in none listed_only skip_last first_only lost_only_listed break_after_lost lost_then_poll poll_all; do
  nice -n 19 bash "$CAP" 1500M env OPUS_NO_GPU_FN_ARM=$arm bash "$CAPPED" 1 "$L" gpu_memory_ledger:: --skip opus_no > runs/k4.log 2>&1 && rc=0 || rc=$?
  echo "K4LEDGER arm=$arm rc=$rc $(grep '^test result' runs/k4.log) failed:$(grep -E '^test .* FAILED$' runs/k4.log | sed 's/^test gpu_memory_ledger::tests:://; s/ ... FAILED//' | tr '\n' ' ')"
done > summary/k4-ledger-mutants.txt
# 第十五块 K4：两种遮蔽写法 cargo check 与照搬的扫源码比
bash "$MD/src/k4_check.sh" "$WORK" "$MD/src" > summary/k4-shadow.txt 2>&1
sed -i "s#$WORK#<草稿目录>#g" summary/k4-shadow.txt
# 对照：秒数（finished in …）与编译用时不比
for s in "$MD"/runs-summary/*.txt; do
  n=$(basename "$s")
  if diff -q <(sed 's/finished in [0-9.]*s//; s/target(s) in [0-9.]*s//' "summary/$n") <(sed 's/finished in [0-9.]*s//; s/target(s) in [0-9.]*s//' "$s") > /dev/null; then echo "同：$n"; else echo "不同：$n"; fi
done
