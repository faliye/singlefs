# 收口表第 42 行：`crates/mutations.tsv` 6 行锚点改到今天的写法（实现员报告）

时刻都是 UTC（本机时钟）。开工时 `ps` 看过两次（08:40Z 前、08:43Z）：没有 qemu-system、vm-bench.sh、e152、fio，也没有别的 cargo / gate.sh 在跑，没等锁。

## 结论

- `crates/mutations.tsv` 第 32、37、41、42、69、107 行就地改了第 3 段（原文）和第 4 段（替换文），第 1、2、5、6 段没动，没在末尾追加行，其余 122 行逐字不变（脚本逐行比过改前的副本，改了的只有这 6 行）。
- 6 行新原文在各自第 2 段的文件里都恰好命中 1 次；改前原文在今天的源码里都是 0 次。
- 照 59 号的判法（草稿副本、施加替换、`cargo test --offline <第 5 段>`）：6 行都红在第 6 段点名的测试上，红在测试文件自己的断言上（`crates/singlefs-harness/tests/*.rs` 的行），没有红在 `crates/*/src/` 里的 `assert!` / `expect` 上；这三份源码文件里 `debug_assert` 是 0 处，不用再跑 `--release`。不施加时这 6 条命令都是绿的，4 个测试二进制整个跑也都是绿的（基线红集为空）。
- 按 59 号的预扫对整张表数了一遍：128 行里注释 5 行、变异 123 条，123 条都恰好命中 1 次。
- 6 行守的行为在今天的代码里都还在，没有要停下的行。点名的 4 个测试都还在、名字没改。
- 推翻条件：按 59 号的读法数出这 6 行里任何一行命中不是 1 次；或者崩溃验证员整表复跑时这 6 行里有一行判「没红」或「没跑到」；或者有人指出下面哪一行的替换坏掉的不是变异名说的那个行为（第 41、107 行的取舍见「交主 agent 的问题」）。

## 每一行改了什么（源码为什么变了，今天在哪）

| 行 | 今天源码里的位置 | 原文为什么变了 | 替换坏掉的行为 | 红在哪条断言 |
|---|---|---|---|---|
| 32 | `crates/singlefs-core/src/mount.rs` 第 943 行（`instance_rows_to_write`，第 938–962 行） | 第二波把写行的 `for` 循环从 `establish_instance` 取号之后挪成取号之前的纯函数，区间上界从 `instance.0` 换成 `instance_to_acquire.0` | 区间只剩 `first_row_instance` 一个，中间实例行不写（与改前的替换同形） | `second_transaction_step_four_rollback.rs:160`「回退行与中间实例行」：实际只有实例 1 那行，要的是实例 1、2 两行 |
| 37 | `crates/singlefs-checker/src/walk.rs` 第 1377 行（候选集 `filter`，第 1369–1380 行） | 候选集从 `for` 循环里的 `if` 换成先算 `candidate_indexes` 的 `filter` 闭包；第二波开工前就已经是这样（anchor-origin.out） | 去掉 `!abandoned`：被抛弃时间线的根也进候选集，也进 I-3.1 的并集 | `second_transaction_step_four_rollback.rs:275`「I-3.1 在回退之后的镜像上要成立」：`Violated("盘 0：记账的已分配 Some(376832)，遍历全部有效根得到 933888")` |
| 41 | `crates/singlefs-core/src/allocator.rs` 第 541–543 行（`make_room_for_record_on_device`，第 508–555 行） | 第二波（Z1-d）把「复用时改写那条已回收记录」从 `PoolAllocator::record` 挪进 `make_room_for_record_on_device`，`if self.reclaimed.remove(&key)` 那一句没了 | 那条已回收记录不改写（留着已释放、代 3、跨 1），另追加一条新记录：同盘同槽两条，与改前 `&& false` 的效果相同 | `second_transaction_step_five_reuse.rs:198`「复用时那条记录改写」：`left: (CheckpointTxg(3), true, 1)`，要 `(CheckpointTxg(17), false, 2)` |
| 42 | 同第 37 行 | 同第 37 行 | 去掉 `!below_floor`：F 之下的根也进候选集、照走 | `second_transaction_step_five_reuse.rs:233`「I-2.1 在 E 之后的镜像上要成立」：`Violated("树表单元 在盘 0 槽 50178 的那一份与位置条目里的校验和对不上")`（A 的根在 F = 11 之下，槽 50178 已被 E 复用） |
| 69 | `crates/singlefs-core/src/mount.rs` 第 984–1006 行 | 第二波（Z1-a）给 `refuse_publishes_before_acquisition_that_do_not_pass_admission` 加了参数 `rows_to_write`，调用处多了一行 `rows_written.len(),`（第 990 行） | 与改前相同：`refuse_instance_rows_on_version_without_file` 挪到取号之后；原文与替换文都只多了那一行 | `second_transaction_step_three_formatted_pool.rs:169`「两盘超级块槽逐字节不变、根环没有新根、一个写都没发」 |
| 107 | `crates/singlefs-core/src/mount.rs` 第 986–992 行 | 同第 69 行 | 与改前相同：整次调用换成一行注释；原文多了 `rows_written.len(),` 那一行，替换文没变 | `second_transaction_supplement_two_row_publish_admission.rs:211`：`要在取号之前按写行那次发布的准入拒绝：Err(Publish(AllocationRecordsExceedOneNode { records: 822, capacity: 812 }))` |

点名的测试现查过都还在：`second_transaction_step_four_rollback.rs:153`、`second_transaction_step_five_reuse.rs:131`、`second_transaction_step_three_formatted_pool.rs:141`、`second_transaction_supplement_two_row_publish_admission.rs:163`（`grep -n 'fn <名字>'`），与第二波开工前的副本相比，其中三份测试文件逐字相同，`second_transaction_step_three_formatted_pool.rs` 只改了 `NOT_APPLICABLE_WITHOUT_FILE`（加了 I-5.4），点名的那条测试没动。

## 写过的文件

- `crates/mutations.tsv`：第 32、37、41、42、69、107 行就地改第 3、4 段（7 次 Edit：第 69 行的原文、替换文各一次）。没有追加行，所以没有新增的变异名。
- `/tmp/claude-1000/m2-anchor-fix/report.md`（本报告）。
- 草稿目录 `/tmp/claude-1000/m2-anchor-fix/`：`build_rows.py`（从改前的表算出新行、数命中）、`new-rows.json`、`alternative-41.json`、`run_mutations.py`（照 59 号的判法跑）、`prescan.py` / `prescan.out`（整表预扫）、`render_rows.py`、`mutations.tsv.before`（改前的表，sha256 `05a0b5aa9bbd58237bdbe5d0c6fe7e8131f37d84dcb9e4412e1e6cd497cc72f7`）、`baseline.out`、`mutate.out`、`check.out`、`gate53.out`、`logs/`（每次 cargo test 的完整输出）、`copy/`（仓副本，`rsync -a --exclude target --exclude .git`）、`copy-target/`（副本自己的编译产物）。

`git diff --stat -- crates litmus` 原样（`git diff` 比的是工作区与暂存区；`crates/` 下其余改动在我开工前就已经暂存，不是这一轮写的）：

```text
 crates/mutations.tsv | 12 ++++++------
 1 file changed, 6 insertions(+), 6 deletions(-)
```

改后 `crates/mutations.tsv` 的 sha256：`d1d2e54b79ac134fa2e5ca7e6aadeedccf7cf205909f433d5b78e2015d7964c3`。

## 逐行：改前改后的原文与替换文、命中、变异与基线输出

副本与工作区核过：`diff -rq crates <副本>/crates -x target` 只报 `crates/mutations.tsv` 不同，源码逐字相同，每次施加之后都写回原件、`touch` 过、回读核对过（`run_mutations.py` 末尾那句 `assert`）。编译产物只放 `copy-target/`，没有指到共用目录。基线 08:40:40Z–08:40:50Z，变异 08:41:08Z–08:41:25Z。

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

### 第 41 行的另一种写法（跑过、没采用）

调用方 `PoolAllocator::record` 在「已改写」那一臂也追加（`crates/singlefs-core/src/allocator.rs` 第 678–679 行）：

```text
原文：                RecordAtThePlacementSlot::RewrittenFromReclaimed => {}\n                RecordAtThePlacementSlot::Absent => {
替换：                RecordAtThePlacementSlot::RewrittenFromReclaimed | RecordAtThePlacementSlot::Absent => {
```

原文在 `allocator.rs` 里命中 1 次；施加之后点名测试红在 `second_transaction_step_five_reuse.rs:207`「同盘同槽只有一条记录」（`left: 2`，`right: 1`），整个二进制只红这一条（`test result: FAILED. 10 passed; 1 failed`）。没采用：它先改写、再追加，只坏了变异名「追加而不改写」里「追加」那一半；表里采用的写法两半都坏（不改写、另追加），与改前 `if self.reclaimed.remove(&key) && false {` 的效果相同。

## 整表预扫（照 59 号：六段、`\n` 当换行、数第 2 段那个文件里的命中）

`nice -n 19 python3 /tmp/claude-1000/m2-anchor-fix/prescan.py /home/fy5090/code/singlefs` 末两行原样（逐行 123 条的清单在 `prescan.out`）：

```text
表共 128 行：注释行 5、变异 123 条；命中次数分布 {1: 123}
命中不是 1 次的： 无
```

## check.sh 与登记给我的门禁阶段

`nice -n 19 bash .claude/scripts/check.sh`（08:43:28Z 起，08:44:08Z 完）：四个阶段行是「✓ 格式通过」「✓ clippy 通过」「✓ 构建通过」「✓ 单测通过」，`^test result: ok` 35 行、没有 FAILED。末尾原样：

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests singlefs_harness

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ✓ 单测通过
check.sh exit 0
```

阶段归属表登记给 implementation-writer 的只有 `53-format-const-placeholders.sh`：`nice -n 19 bash .claude/gate.d/53-format-const-placeholders.sh` 退出码 0，末行原样：

```text
  ✓ 格式常量文件里的占位都指得到分项或欠账（2 个占位：D23（journal 的角色与格式） 已定项 19；C323（镜像大小全仓没有条款））
```

## 交主 agent 的问题（没有条款没定的设计判断；这一轮没有要停下的行）

1. 第 107 行删的调用比改前宽了：`refuse_publishes_before_acquisition_that_do_not_pass_admission` 在第二波里多了实例表那一项（Z1-a，`crates/singlefs-core/src/mount.rs` 第 886–897 行）。改前这条变异删掉的是写行与暖机两项准入，现在删掉写行、暖机、实例表三项。点名测试红的原因是写行那次发布取号之后才被拒（`AllocationRecordsExceedOneNode { records: 822, capacity: 812 }`），坏的仍是变异名说的那个行为；实例表那一项另有第 125–128 行的变异守着。要不要把第 107 行收窄到只拿掉写行那一项（例如只改 `std::iter::once(PublishShape::ROW_PUBLISH)` 那一段），由你定；我照「原文改到今天的写法」只加了参数那一行，没收窄。
2. 第 37、42 行坏的范围也比名字宽：今天的 `candidate_indexes` 除了走读循环（I-3.1 的并集、I-7.4、I-4.8），还交给 `judge_release_generation_and_tree_table_birth`（`crates/singlefs-checker/src/walk.rs` 第 1411–1418 行）和后面的判定。这是第二波开工前就有的结构（改前的原文那时已经 0 次命中），不是这一轮带来的。点名测试红在名字说的那一项上（第 37 行红在 I-3.1，第 42 行红在 I-2.1：F 之下的根走到了已被复用的槽）。
3. 第 41 行的取舍见上一节：表里采用的写法照搬改前的效果（不改写、另追加）。它把 `record_at_the_placement_slot` 仍设成 `RewrittenFromReclaimed`（第 544 行没动），这样调用方不会再追加第三条，第 549–553 行那句 `assert!` 也不会先红；要是按「最少字符」挑，就是那种调用方写法。

## 草稿产物为什么没入库

草稿目录 `/tmp/claude-1000/m2-anchor-fix/` 里的 `baseline.out`、`mutate.out`、`logs/`、`prescan.out`、`check.out` 是这一轮跑出来的证据。我的写范围只有 `crates/`、`litmus/` 和 `/tmp/claude-1000/`，写不进 `research/results/` 和 `research/prompts/`，所以没入库。要不要留、放哪由主 agent 定。`copy/` 和 `copy-target/`（445M）是副本与编译产物，不用留。

## 没做什么

- 没跑 59 号整表复跑（派发提示说由崩溃验证员另跑），也没跑 `gate.sh`。第 121–128 行只数过命中，没施加过。
- 没改 `crates/*/src/` 和测试，没追加变异行。
- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异表的整表复跑归 `crash-verifier`；没提交，没做任何 git 写操作。
- 收口表第 42 行「去向」里要改 `implementation-writer` 定义的那一半（整表预扫写进定义、改定义走三方）不归我，没碰。
