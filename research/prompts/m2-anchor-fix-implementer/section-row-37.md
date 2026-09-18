### 第 37 行　步 4：checker 的 I-3.1 并集不按实例表排除被抛弃的根

文件 `crates/singlefs-checker/src/walk.rs`；第 5 段 `-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root`、第 6 段 `rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content` 都没动。

改前原文（第 3 段，表里原样）：
```text
        if index != newest_index && !abandoned && !below_floor {
```
改前替换文（第 4 段）：
```text
        if index != newest_index && !below_floor {
```
改后原文（第 3 段）：
```text
            *index == newest_index || (!abandoned && !below_floor)
```
改后替换文（第 4 段）：
```text
            *index == newest_index || !below_floor
```
命中（\n 当换行，数的是工作区今天的 `crates/singlefs-checker/src/walk.rs`）：改前原文 0 次，改后原文 1 次。

不施加替换（基线），`cargo test --offline -p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root` 原样输出行：
```text
test rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.46s
```
施加替换，同一条命令，原样输出行（点名测试的结果行与它 stdout 段）：
```text
test rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.52s
thread 'rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content' (680851) panicked at crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs:275:9:
assertion `left == right` failed: I-3.1 在回退之后的镜像上要成立
  left: Violated("盘 0：记账的已分配 Some(376832)，遍历全部有效根得到 933888")
 right: Holds
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
施加替换，跑整个测试二进制（去掉 `--` 之后的过滤串），红的测试与结果行：
```text
test rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content ... FAILED
test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.23s
```
