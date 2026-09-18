### 第 41 行　步 5：复用时追加记录而不改写（同盘同槽两条）

文件 `crates/singlefs-core/src/allocator.rs`；第 5 段 `-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first`、第 6 段 `raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it` 都没动。

改前原文（第 3 段，表里原样）：
```text
            if self.reclaimed.remove(&key) {
```
改前替换文（第 4 段）：
```text
            if self.reclaimed.remove(&key) && false {
```
改后原文（第 3 段）：
```text
            existing.span_slots = u16::try_from(placement.span).expect("跨度 2 字节");\n            existing.generation = generation;\n            existing.is_released = false;
```
改后替换文（第 4 段）：
```text
            records.push(AllocationRecord {\n                device,\n                slot: placement.slot,\n                span_slots: u16::try_from(placement.span).expect("跨度 2 字节"),\n                generation,\n                is_released: false,\n            });
```
命中（\n 当换行，数的是工作区今天的 `crates/singlefs-core/src/allocator.rs`）：改前原文 0 次，改后原文 1 次。

不施加替换（基线），`cargo test --offline -p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first` 原样输出行：
```text
test raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.57s
```
施加替换，同一条命令，原样输出行（点名测试的结果行与它 stdout 段）：
```text
test raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.43s
thread 'raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it' (681844) panicked at crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs:198:5:
assertion `left == right` failed: 复用时那条记录改写
  left: (CheckpointTxg(3), true, 1)
 right: (CheckpointTxg(17), false, 2)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
施加替换，跑整个测试二进制（去掉 `--` 之后的过滤串），红的测试与结果行：
```text
test raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it ... FAILED
test result: FAILED. 10 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.16s
```
