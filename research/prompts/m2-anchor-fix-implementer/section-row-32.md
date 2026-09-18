### 第 32 行　步 4：只写被退回的实例那一行、不写中间实例行

文件 `crates/singlefs-core/src/mount.rs`；第 5 段 `-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root`、第 6 段 `rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content` 都没动。

改前原文（第 3 段，表里原样）：
```text
    for row_instance in first_row_instance..instance.0 {
```
改前替换文（第 4 段）：
```text
    for row_instance in first_row_instance..first_row_instance + 1 {
```
改后原文（第 3 段）：
```text
    (first_row_instance..instance_to_acquire.0)
```
改后替换文（第 4 段）：
```text
    (first_row_instance..first_row_instance + 1)
```
命中（\n 当换行，数的是工作区今天的 `crates/singlefs-core/src/mount.rs`）：改前原文 0 次，改后原文 1 次。

不施加替换（基线），`cargo test --offline -p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root` 原样输出行：
```text
test rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.46s
```
施加替换，同一条命令，原样输出行（点名测试的结果行与它 stdout 段）：
```text
test rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.39s
thread 'rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content' (679859) panicked at crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs:160:5:
assertion `left == right` failed: 回退行与中间实例行
  left: [InstanceRow { instance: InstanceGeneration(1), selected_root_txg: CheckpointTxg(3), applied_transaction_high_water: 0, is_rollback: true }]
 right: [InstanceRow { instance: InstanceGeneration(1), selected_root_txg: CheckpointTxg(3), applied_transaction_high_water: 0, is_rollback: true }, InstanceRow { instance: InstanceGeneration(2), selected_root_txg: CheckpointTxg(0), applied_transaction_high_water: 0, is_rollback: false }]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
施加替换，跑整个测试二进制（去掉 `--` 之后的过滤串），红的测试与结果行：
```text
test rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content ... FAILED
test torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount ... FAILED
test rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation ... FAILED
test rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused ... FAILED
test result: FAILED. 3 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.24s
```
