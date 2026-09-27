#!/usr/bin/env bash
# 复跑（m2-closeout-code-r2 云端攻方腿）：在快照 refs/sop/m2-closeout-code-r2-snapshot 的副本上加这几份用例跑，不动主工作区。
# 用法：bash rerun.sh <草稿目录> [线程上限，缺省 4]
# 1. Y2（旧快照、盘体对调、一槽自证、旧快照按用户动作扫）  2. Y4（系统配置字段 core / checker 各读各的；mkfs 环长）
# 3. A2a（同段同盘两次系统配置写）  4. Y1/Y7 单故障逐序号扫（两件并行，约 28 分钟）
# 5. Y6 没文件那一族全量（约 2 分钟）、有文件那一族正常卸载全量（约 40 分钟）  6. 原型补丁：打在第二份副本上重跑 Y2 与 5 份回归
set -euo pipefail
S=${1:?草稿目录}; N=${2:-4}
R=/home/fy5090/code/singlefs
M=$R/research/prompts/m2-closeout-code-r2-opus-model
mkdir -p "$S/tree" "$S/fix" "$S/logs"
git -C "$R" archive refs/sop/m2-closeout-code-r2-snapshot crates Cargo.toml Cargo.lock | tar -x -C "$S/tree"
cp "$M"/opus_r2_*.rs "$S/tree/crates/singlefs-harness/tests/"
CAP1="bash $R/research/scripts/capped.sh 1 bash $R/research/scripts/run-with-memory-cap.sh 8G"
CAP2="bash $R/research/scripts/capped.sh 2 bash $R/research/scripts/run-with-memory-cap.sh 8G"
cd "$S/tree"
export CARGO_TARGET_DIR="$S/target"
nice -n 19 $CAP1 cargo test -p singlefs-harness --release --test opus_r2_y2_session_and_entries_with_stale_or_swapped_devices -- --nocapture --test-threads 1 > "$S/logs/y2.log" 2>&1
nice -n 19 $CAP1 cargo test -p singlefs-harness --test opus_r2_y4_system_configuration_fields_core_and_checker_read_differently -- --nocapture --test-threads 1 > "$S/logs/y4.log" 2>&1
nice -n 19 $CAP1 cargo test -p singlefs-harness --release --test opus_r2_y7_same_segment_system_configuration_writes -- --nocapture > "$S/logs/a2a.log" 2>&1
nice -n 19 $CAP1 cargo test -p singlefs-harness --release --test entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration > "$S/logs/z3a-existing.log" 2>&1
nice -n 19 bash "$R/research/scripts/capped.sh" 2 cargo test -p singlefs-harness --release --test opus_r2_y1_y7_single_fault_ordinal_sweep --no-run
bash "$M/batch_sweep.sh" 1 "$S/logs" "$S/tree" "$S/target"
python3 "$M/analyze_sweep.py" "$S/logs/b1-1.log" | grep '^OUT'
python3 "$M/analyze_sweep.py" "$S/logs/b1-2.log" | grep -v '^    '
OPUS_Y6_WINDOW=nofile_remount OPUS_Y6_REMOUNT_ALL=1 nice -n 19 $CAP2 cargo test -p singlefs-harness --release --test opus_r2_y6_journal_ring_turned_torn_records crash_states -- --nocapture > "$S/logs/y6-nofile.log" 2>&1
nice -n 19 $CAP2 cargo test -p singlefs-harness --release --test opus_r2_y6_journal_ring_turned_torn_records probe -- --nocapture > "$S/logs/y6-probe.log" 2>&1
OPUS_Y6_WINDOW=unmount nice -n 19 $CAP2 cargo test -p singlefs-harness --release --test opus_r2_y6_journal_ring_turned_torn_records crash_states -- --nocapture > "$S/logs/y6-unmount.log" 2>&1
# 原型补丁（只在副本上）
git -C "$R" archive refs/sop/m2-closeout-code-r2-snapshot crates Cargo.toml Cargo.lock | tar -x -C "$S/fix"
patch -d "$S/fix" -p1 < "$M/fix_prototype_swapped_or_behind.patch"
cp "$M"/opus_r2_y2_session_and_entries_with_stale_or_swapped_devices.rs "$S/fix/crates/singlefs-harness/tests/"
cd "$S/fix"
export CARGO_TARGET_DIR="$S/target-fix"
nice -n 19 $CAP1 cargo test -p singlefs-harness --release --test opus_r2_y2_session_and_entries_with_stale_or_swapped_devices -- --nocapture --test-threads 1 > "$S/logs/y2-fix.log" 2>&1
for tree in fix tree; do
  cd "$S/$tree"
  if [ "$tree" = fix ]; then export CARGO_TARGET_DIR="$S/target-fix"; out=fix-regression; else export CARGO_TARGET_DIR="$S/target"; out=snapshot-regression; fi
  for t in entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration entries_after_a_writable_mount_refuse_other_parameters_and_device_tables_before_any_write second_transaction_admission_raises_the_floor_before_refusing rollback_floor_written_into_the_system_configuration_first_and_normal_unmount second_transaction_supplement_two_publish_failure_resent_unchanged; do
    echo "== $t"; nice -n 19 $CAP1 cargo test -p singlefs-harness --release --test "$t" 2>&1 | grep -E '^test |test result|panicked' || true
  done > "$S/logs/$out.log"
done
diff <(grep 'FAILED$' "$S/logs/snapshot-regression.log") <(grep 'FAILED$' "$S/logs/fix-regression.log") && echo SAME_FAILED_SET
