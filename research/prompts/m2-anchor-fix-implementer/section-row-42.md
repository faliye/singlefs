### 第 42 行　步 5：checker 的候选集不按 F 收（F 之下的根照走）

文件 `crates/singlefs-checker/src/walk.rs`；第 5 段 `-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first`、第 6 段 `raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it` 都没动。

改前原文（第 3 段，表里原样）：
```text
        if index != newest_index && !abandoned && !below_floor {
```
改前替换文（第 4 段）：
```text
        if index != newest_index && !abandoned {
```
改后原文（第 3 段）：
```text
            *index == newest_index || (!abandoned && !below_floor)
```
改后替换文（第 4 段）：
```text
            *index == newest_index || !abandoned
```
命中（\n 当换行，数的是工作区今天的 `crates/singlefs-checker/src/walk.rs`）：改前原文 0 次，改后原文 1 次。

不施加替换（基线），`cargo test --offline -p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first` 原样输出行：
```text
test raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.56s
```
施加替换，同一条命令，原样输出行（点名测试的结果行与它 stdout 段）：
```text
test raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.61s
thread 'raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it' (682848) panicked at crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs:233:9:
assertion `left == right` failed: I-2.1 在 E 之后的镜像上要成立
  left: Violated("树表单元 在盘 0 槽 50178 的那一份与位置条目里的校验和对不上")
 right: Holds
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
施加替换，跑整个测试二进制（去掉 `--` 之后的过滤串），红的测试与结果行：
```text
test raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it ... FAILED
test result: FAILED. 10 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.02s
```
