### 第 69 行　步 3：树表 0 条的一版上要写行时，拒绝挪到取号之后（超级块已写进新号才返回）

文件 `crates/singlefs-core/src/mount.rs`；第 5 段 `-p singlefs-harness --test second_transaction_step_three_formatted_pool -- writable_mount_after_a_crash_right_after_acquiring`、第 6 段 `writable_mount_after_a_crash_right_after_acquiring_an_instance_is_refused_before_acquiring_again` 都没动。

改前原文（第 3 段，表里原样）：
```text
    refuse_instance_rows_on_version_without_file(&start, instance_to_acquire)?;\n    refuse_formatted_pool_mount_not_shaped_like_the_first_transaction(parameters, &start)?;\n    refuse_publishes_before_acquisition_that_do_not_pass_admission(\n        &allocator,\n        &start,\n        instance_to_acquire,\n        warm_up_publishes_planned.len(),\n    )?;\n    let instance = acquire_expected_instance(&mut pool, instance_to_acquire).map_err(\n        |failure| match failure {\n            ExpectedInstanceAcquisitionFailed::InstanceGenerationChangedBeforeWrite {\n                expected,\n                recomputed,\n            } => MountError::InstanceGenerationChangedBeforeAcquisition {\n                expected,\n                recomputed,\n            },\n            ExpectedInstanceAcquisitionFailed::Acquisition(acquisition) => {\n                MountError::Acquisition(acquisition)\n            }\n        },\n    )?;\n
```
改前替换文（第 4 段）：
```text
    refuse_formatted_pool_mount_not_shaped_like_the_first_transaction(parameters, &start)?;\n    refuse_publishes_before_acquisition_that_do_not_pass_admission(\n        &allocator,\n        &start,\n        instance_to_acquire,\n        warm_up_publishes_planned.len(),\n    )?;\n    let instance = acquire_expected_instance(&mut pool, instance_to_acquire).map_err(\n        |failure| match failure {\n            ExpectedInstanceAcquisitionFailed::InstanceGenerationChangedBeforeWrite {\n                expected,\n                recomputed,\n            } => MountError::InstanceGenerationChangedBeforeAcquisition {\n                expected,\n                recomputed,\n            },\n            ExpectedInstanceAcquisitionFailed::Acquisition(acquisition) => {\n                MountError::Acquisition(acquisition)\n            }\n        },\n    )?;\n    refuse_instance_rows_on_version_without_file(&start, instance_to_acquire)?;\n
```
改后原文（第 3 段）：
```text
    refuse_instance_rows_on_version_without_file(&start, instance_to_acquire)?;\n    refuse_formatted_pool_mount_not_shaped_like_the_first_transaction(parameters, &start)?;\n    refuse_publishes_before_acquisition_that_do_not_pass_admission(\n        &allocator,\n        &start,\n        instance_to_acquire,\n        rows_written.len(),\n        warm_up_publishes_planned.len(),\n    )?;\n    let instance = acquire_expected_instance(&mut pool, instance_to_acquire).map_err(\n        |failure| match failure {\n            ExpectedInstanceAcquisitionFailed::InstanceGenerationChangedBeforeWrite {\n                expected,\n                recomputed,\n            } => MountError::InstanceGenerationChangedBeforeAcquisition {\n                expected,\n                recomputed,\n            },\n            ExpectedInstanceAcquisitionFailed::Acquisition(acquisition) => {\n                MountError::Acquisition(acquisition)\n            }\n        },\n    )?;\n
```
改后替换文（第 4 段）：
```text
    refuse_formatted_pool_mount_not_shaped_like_the_first_transaction(parameters, &start)?;\n    refuse_publishes_before_acquisition_that_do_not_pass_admission(\n        &allocator,\n        &start,\n        instance_to_acquire,\n        rows_written.len(),\n        warm_up_publishes_planned.len(),\n    )?;\n    let instance = acquire_expected_instance(&mut pool, instance_to_acquire).map_err(\n        |failure| match failure {\n            ExpectedInstanceAcquisitionFailed::InstanceGenerationChangedBeforeWrite {\n                expected,\n                recomputed,\n            } => MountError::InstanceGenerationChangedBeforeAcquisition {\n                expected,\n                recomputed,\n            },\n            ExpectedInstanceAcquisitionFailed::Acquisition(acquisition) => {\n                MountError::Acquisition(acquisition)\n            }\n        },\n    )?;\n    refuse_instance_rows_on_version_without_file(&start, instance_to_acquire)?;\n
```
命中（\n 当换行，数的是工作区今天的 `crates/singlefs-core/src/mount.rs`）：改前原文 0 次，改后原文 1 次。

不施加替换（基线），`cargo test --offline -p singlefs-harness --test second_transaction_step_three_formatted_pool -- writable_mount_after_a_crash_right_after_acquiring` 原样输出行：
```text
test writable_mount_after_a_crash_right_after_acquiring_an_instance_is_refused_before_acquiring_again ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.19s
```
施加替换，同一条命令，原样输出行（点名测试的结果行与它 stdout 段）：
```text
test writable_mount_after_a_crash_right_after_acquiring_an_instance_is_refused_before_acquiring_again ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.19s
thread 'writable_mount_after_a_crash_right_after_acquiring_an_instance_is_refused_before_acquiring_again' (683861) panicked at crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs:169:5:
assertion `left == right` failed: 两盘超级块槽逐字节不变、根环没有新根、一个写都没发
  left: DiskSnapshot { superblock_slots: [[83, 70, 83, 66, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 …（这一行原长 53598 字符，报告里截到 300）
 right: DiskSnapshot { superblock_slots: [[83, 70, 83, 66, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 …（这一行原长 53597 字符，报告里截到 300）
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
施加替换，跑整个测试二进制（去掉 `--` 之后的过滤串），红的测试与结果行：
```text
test writable_mount_after_a_crash_right_after_acquiring_an_instance_is_refused_before_acquiring_again ... FAILED
test writable_mount_after_a_crash_between_warm_up_and_the_first_file_is_refused_before_acquiring_an_instance ... FAILED
test result: FAILED. 4 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
```
