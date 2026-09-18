### 第 107 行　增补 2 第 20a 行：写行那次发布的准入不在取号之前算（取号之后发布才被拒，实例代号已经烧掉）

文件 `crates/singlefs-core/src/mount.rs`；第 5 段 `-p singlefs-harness --test second_transaction_supplement_two_row_publish_admission -- a_writable_mount_that_cannot_publish`、第 6 段 `a_writable_mount_that_cannot_publish_the_rows_is_refused_before_the_instance_is_acquired` 都没动。

改前原文（第 3 段，表里原样）：
```text
    refuse_publishes_before_acquisition_that_do_not_pass_admission(\n        &allocator,\n        &start,\n        instance_to_acquire,\n        warm_up_publishes_planned.len(),\n    )?;\n
```
改前替换文（第 4 段）：
```text
    // 变异：写行那次发布的准入不在取号之前算\n
```
改后原文（第 3 段）：
```text
    refuse_publishes_before_acquisition_that_do_not_pass_admission(\n        &allocator,\n        &start,\n        instance_to_acquire,\n        rows_written.len(),\n        warm_up_publishes_planned.len(),\n    )?;\n
```
改后替换文（第 4 段）：
```text
    // 变异：写行那次发布的准入不在取号之前算\n
```
命中（\n 当换行，数的是工作区今天的 `crates/singlefs-core/src/mount.rs`）：改前原文 0 次，改后原文 1 次。

不施加替换（基线），`cargo test --offline -p singlefs-harness --test second_transaction_supplement_two_row_publish_admission -- a_writable_mount_that_cannot_publish` 原样输出行：
```text
test a_writable_mount_that_cannot_publish_the_rows_is_refused_before_the_instance_is_acquired ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.67s
```
施加替换，同一条命令，原样输出行（点名测试的结果行与它 stdout 段）：
```text
test a_writable_mount_that_cannot_publish_the_rows_is_refused_before_the_instance_is_acquired ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.64s
thread 'a_writable_mount_that_cannot_publish_the_rows_is_refused_before_the_instance_is_acquired' (684731) panicked at crates/singlefs-harness/tests/second_transaction_supplement_two_row_publish_admission.rs:211:18:
要在取号之前按写行那次发布的准入拒绝：Err(Publish(AllocationRecordsExceedOneNode { records: 822, capacity: 812 }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
施加替换，跑整个测试二进制（去掉 `--` 之后的过滤串），红的测试与结果行：
```text
test a_writable_mount_that_cannot_publish_the_rows_is_refused_before_the_instance_is_acquired ... FAILED
test a_writable_mount_whose_first_warm_up_publish_does_not_fit_is_refused_before_the_instance_is_acquired ... FAILED
test a_writable_mount_whose_second_warm_up_publish_does_not_fit_is_refused_before_the_instance_is_acquired ... FAILED
test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.78s
```
