# 附录二：增补 3 第 1 件（随机历史生成器）代码三方对抗第一轮的代码改动（基准 `00c9d4f`；生成于 2026-09-18 15:24 UTC）

## 一、diff（相对 `00c9d4f`，`git diff 00c9d4f -- crates/singlefs-harness/src/crash.rs crates/singlefs-harness/src/lib.rs crates/mutations.tsv` 原样）

```diff
diff --git a/crates/mutations.tsv b/crates/mutations.tsv
index f563881..6a6389b 100644
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -126,3 +126,5 @@ Z1-a：取号之前不算实例表	crates/singlefs-core/src/mount.rs
 Z1-a：实例表准入漏算链指针	crates/singlefs-core/src/mount.rs	            if rows_in_version + rows_to_write + 1 > records_per_page {	            if rows_in_version + rows_to_write > records_per_page {	-p singlefs-harness --test second_transaction_supplement_two_instance_table_page_full -- writable_mounts_fill	writable_mounts_fill_the_instance_table_page_and_the_next_one_is_refused_before_acquisition
 Z1-a：实例表准入把正好写满一片也拒掉	crates/singlefs-core/src/mount.rs	            if rows_in_version + rows_to_write + 1 > records_per_page {	            if rows_in_version + rows_to_write + 1 >= records_per_page {	-p singlefs-harness --test second_transaction_supplement_two_instance_table_page_full -- mount_after_crashes	mount_after_crashes_right_after_acquisition_may_fill_the_page_but_not_overflow_it
 Z1-a：实例表准入按每次挂载只写一行算	crates/singlefs-core/src/mount.rs	        rows_written.len(),\n        warm_up_publishes_planned.len(),	        1,\n        warm_up_publishes_planned.len(),	-p singlefs-harness --test second_transaction_supplement_two_instance_table_page_full -- mount_after_crashes	mount_after_crashes_right_after_acquisition_may_fill_the_page_but_not_overflow_it
+增补 3 第 1 件：随机历史快档判出「复用时追加记录而不改写」（第 41 行同一处）	crates/singlefs-core/src/allocator.rs	            existing.span_slots = u16::try_from(placement.span).expect("跨度 2 字节");\n            existing.generation = generation;\n            existing.is_released = false;	            records.push(AllocationRecord {\n                device,\n                slot: placement.slot,\n                span_slots: u16::try_from(placement.span).expect("跨度 2 字节"),\n                generation,\n                is_released: false,\n            });	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
+增补 3 第 1 件：随机历史快档判出「复用时新记录罩住的已回收记录不删」（第 121 行同一处）	crates/singlefs-core/src/allocator.rs	            records.retain(|record| !(record.device == device && record.slot == record_slot));	            // 变异：罩住的已回收记录不删	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
diff --git a/crates/singlefs-harness/src/crash.rs b/crates/singlefs-harness/src/crash.rs
index c412988..e4409a6 100644
--- a/crates/singlefs-harness/src/crash.rs
+++ b/crates/singlefs-harness/src/crash.rs
@@ -34,22 +34,29 @@ pub struct SparseDevice {
 impl SparseDevice {
     #[must_use]
     pub fn read(&self, offset: DeviceOffsetInBytes, length: usize) -> Vec<u8> {
+        let mut out = vec![0u8; length];
+        self.read_into(offset, &mut out);
+        out
+    }
+    /// 读进调用方的缓冲：先整段写 0，再只拷区间里写过的扇区（按区间查一次，不逐扇区查）——可写挂载与冷启动要把 768 MiB 的
+    /// journal 环逐条读一遍，几乎全是没写过的扇区（随机历史，增补 3 第 1 件）。
+    pub fn read_into(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) {
         assert!(offset.0.is_multiple_of(SECTOR_BYTES), "读要按扇区对齐");
-        let length_in_bytes = u64::try_from(length).expect("长度");
+        let length_in_bytes = u64::try_from(buffer.len()).expect("长度");
         assert!(
             length_in_bytes.is_multiple_of(SECTOR_BYTES),
             "读长度要是整扇区"
         );
         let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
-        let mut out = vec![0u8; length];
+        buffer.fill(0);
         let first_sector = offset.0 / SECTOR_BYTES;
-        for sector_index in 0..length_in_bytes / SECTOR_BYTES {
-            if let Some(sector) = self.sectors.get(&(first_sector + sector_index)) {
-                let start = usize::try_from(sector_index).expect("扇区下标") * sector_bytes;
-                out[start..start + sector_bytes].copy_from_slice(sector);
-            }
+        for (sector, bytes) in self
+            .sectors
+            .range(first_sector..first_sector + length_in_bytes / SECTOR_BYTES)
+        {
+            let start = usize::try_from(sector - first_sector).expect("扇区下标") * sector_bytes;
+            buffer[start..start + sector_bytes].copy_from_slice(bytes);
         }
-        out
     }
     pub fn write(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
         assert!(offset.0.is_multiple_of(SECTOR_BYTES), "写要按扇区对齐");
@@ -102,7 +109,7 @@ impl singlefs_core::block_device::BlockDevice for SparseBlockDevice {
         offset: DeviceOffsetInBytes,
         buffer: &mut [u8],
     ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
-        buffer.copy_from_slice(&self.image.read(offset, buffer.len()));
+        self.image.read_into(offset, buffer);
         Ok(())
     }
     fn write_at(
diff --git a/crates/singlefs-harness/src/lib.rs b/crates/singlefs-harness/src/lib.rs
index f85018d..46268c4 100644
--- a/crates/singlefs-harness/src/lib.rs
+++ b/crates/singlefs-harness/src/lib.rs
@@ -17,6 +17,7 @@ pub mod crash;
 pub mod device_log;
 pub mod first_transaction_regions;
 pub mod hexadecimal;
+pub mod history;
 pub mod scenario;
 pub mod segments;
 pub mod sha256;
@@ -122,6 +123,11 @@ impl SharedStream {
     pub fn operations(&self) -> Vec<RecordedOperation> {
         self.0.borrow().operations.clone()
     }
+    /// 流里已有几步（不拷整条流）：随机历史拿它判「这一步发没发写」。
+    #[must_use]
+    pub fn operation_count(&self) -> usize {
+        self.0.borrow().operations.len()
+    }
     /// 每一步连同它的内容；没开内容保留的流里 `contents` 全是 None。
     #[must_use]
     pub fn retained_operations(&self) -> Vec<RetainedOperation> {
```

## 二、新文件 `crates/singlefs-harness/src/history.rs`（未进 git，2198 行）：按项抽取（27/27 项；回读逐字节比对；`root_ring_has_turned` 同名两处，按主 agent 2026-09-18 的指示分别取，见下）

### crates/singlefs-harness/src/history.rs:59-64（SeededRandomSource）

```rust
/// 手写的伪随机源：SplitMix64（Steele、Lea、Flood，OOPSLA 2014；`java.util.SplittableRandom` 的输出函数），不加依赖（2026-09-18 用户定）。
/// 同一个种子逐位复现同一串数。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeededRandomSource {
    state: u64,
}
```

### crates/singlefs-harness/src/history.rs:392-418（generate_history）

```rust
/// 按种子生成一段 `operation_count` 步的历史：同一个种子、同一个步数，逐项相同。
#[must_use]
pub fn generate_history(seed: HistorySeed, operation_count: usize) -> GeneratedHistory {
    let mut source = SeededRandomSource::from_seed(seed.0);
    let starting_point = if source.below(2) == 0 {
        HistoryStartingPoint::AfterMakeFilesystem
    } else {
        HistoryStartingPoint::AfterFirstFile
    };
    let mut expected = match starting_point {
        HistoryStartingPoint::AfterMakeFilesystem => ExpectedSession::Closed,
        HistoryStartingPoint::AfterFirstFile => ExpectedSession::OpenWithFile,
    };
    let mut has_file_expected = starting_point == HistoryStartingPoint::AfterFirstFile;
    let mut operations = Vec::with_capacity(operation_count);
    for _ in 0..operation_count {
        let kind = draw_weighted(&mut source, operation_weights(expected));
        operations.push(draw_operation(&mut source, kind));
        expected = expected_session_after(expected, has_file_expected, kind);
        has_file_expected = has_file_expected || expected == ExpectedSession::OpenWithFile;
    }
    GeneratedHistory {
        seed,
        starting_point,
        operations,
    }
}
```

### crates/singlefs-harness/src/history.rs:268-296（operation_weights）

```rust
/// 各类操作在三种估计下的比重（按百分比写，和为 100）。关着的会话只抽挂载与冷启动；树表 0 条的版本上多抽第一个文件，
/// 不然从 mkfs 起的历史多半先推零单元发布、再也接不上第一个文件；带文件的版本上零单元发布只会记「前提不满足」，少抽。
fn operation_weights(expected: ExpectedSession) -> &'static [(HistoryOperationKind, u64)] {
    match expected {
        ExpectedSession::Closed => &[
            (HistoryOperationKind::CloseAndMountWritable, 70),
            (HistoryOperationKind::CloseAndMountRollback, 22),
            (HistoryOperationKind::ColdStartRecover, 8),
        ],
        ExpectedSession::OpenWithoutFile => &[
            (HistoryOperationKind::PublishFirstFile, 60),
            (HistoryOperationKind::PublishWithoutUnits, 20),
            (HistoryOperationKind::CloseAndMountWritable, 6),
            (HistoryOperationKind::CloseAndMountRollback, 4),
            (HistoryOperationKind::ColdStartRecover, 4),
            (HistoryOperationKind::PublishOverwrite, 3),
            (HistoryOperationKind::RaiseRollbackFloor, 3),
        ],
        ExpectedSession::OpenWithFile => &[
            (HistoryOperationKind::PublishOverwrite, 52),
            (HistoryOperationKind::RaiseRollbackFloor, 16),
            (HistoryOperationKind::CloseAndMountWritable, 12),
            (HistoryOperationKind::CloseAndMountRollback, 10),
            (HistoryOperationKind::ColdStartRecover, 4),
            (HistoryOperationKind::PublishFirstFile, 4),
            (HistoryOperationKind::PublishWithoutUnits, 2),
        ],
    }
}
```

### crates/singlefs-harness/src/history.rs:332-365（draw_operation）

```rust
fn draw_operation(source: &mut SeededRandomSource, kind: HistoryOperationKind) -> HistoryOperation {
    match kind {
        HistoryOperationKind::PublishFirstFile => {
            HistoryOperation::PublishFirstFile(draw_content(source))
        }
        HistoryOperationKind::PublishOverwrite => {
            HistoryOperation::PublishOverwrite(draw_content(source))
        }
        HistoryOperationKind::PublishWithoutUnits => HistoryOperation::PublishWithoutUnits,
        HistoryOperationKind::CloseAndMountWritable => HistoryOperation::CloseAndMountWritable,
        HistoryOperationKind::CloseAndMountRollback => {
            // 一半取最近的几条（多半在候选集里），一半在整个根环里均匀取（被抛弃的、F 之下的、暖机那两条树表 0 条的都落得到），
            // 余下的取根环之外。
            let target = match source.below(5) {
                0 | 1 => RollbackTargetChoice::RingRoot {
                    index_from_newest: source.below(4),
                },
                2 | 3 => RollbackTargetChoice::RingRoot {
                    index_from_newest: source.next_word(),
                },
                _ => RollbackTargetChoice::BeyondNewestRoot {
                    txg_beyond_newest: source.below(3),
                },
            };
            HistoryOperation::CloseAndMountRollback(target)
        }
        HistoryOperationKind::RaiseRollbackFloor => {
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: source.next_word(),
            })
        }
        HistoryOperationKind::ColdStartRecover => HistoryOperation::ColdStartRecover,
    }
}
```

### crates/singlefs-harness/src/history.rs:367-390（expected_session_after）

```rust
fn expected_session_after(
    expected: ExpectedSession,
    has_file_expected: bool,
    kind: HistoryOperationKind,
) -> ExpectedSession {
    match kind {
        HistoryOperationKind::CloseAndMountWritable
        | HistoryOperationKind::CloseAndMountRollback => {
            if has_file_expected {
                ExpectedSession::OpenWithFile
            } else {
                ExpectedSession::OpenWithoutFile
            }
        }
        HistoryOperationKind::ColdStartRecover => ExpectedSession::Closed,
        HistoryOperationKind::PublishFirstFile => match expected {
            ExpectedSession::OpenWithoutFile => ExpectedSession::OpenWithFile,
            ExpectedSession::Closed | ExpectedSession::OpenWithFile => expected,
        },
        HistoryOperationKind::PublishOverwrite
        | HistoryOperationKind::PublishWithoutUnits
        | HistoryOperationKind::RaiseRollbackFloor => expected,
    }
}
```

### crates/singlefs-harness/src/history.rs:429-435（HistoryPool）

```rust
/// 一段历史跑到哪了：两块盘、这个进程的会话、挂载的次数。
struct HistoryPool {
    devices: Vec<(DeviceIdentity, HistoryDevice)>,
    session: Option<WritableSession>,
    successful_mounts: usize,
    mount_attempts: usize,
}
```

### crates/singlefs-harness/src/history.rs:1455-1481（apply_operation）

```rust
fn apply_operation(
    pool: &mut HistoryPool,
    operation: &HistoryOperation,
    step_index: usize,
) -> StepOutcome {
    let parameters = history_parameters();
    // 写入时间是参数、不取系统时钟：同一段历史两次跑逐字节相同。
    let write_time_seconds =
        FIXED_WRITE_TIME_SECONDS + u64::try_from(step_index).expect("步号装得进 u64");
    match operation {
        HistoryOperation::PublishFirstFile(content) => {
            apply_publish_first_file(pool, &parameters, content, write_time_seconds)
        }
        HistoryOperation::PublishOverwrite(content) => {
            apply_publish_overwrite(pool, &parameters, content, write_time_seconds)
        }
        HistoryOperation::PublishWithoutUnits => apply_publish_without_units(pool, &parameters),
        HistoryOperation::CloseAndMountWritable => apply_mount_writable(pool, &parameters),
        HistoryOperation::CloseAndMountRollback(choice) => {
            apply_mount_rollback(pool, &parameters, *choice)
        }
        HistoryOperation::RaiseRollbackFloor(choice) => {
            apply_raise_rollback_floor(pool, &parameters, *choice)
        }
        HistoryOperation::ColdStartRecover => apply_cold_start_recover(pool),
    }
}
```

### crates/singlefs-harness/src/history.rs:1210-1248（apply_publish_first_file）

```rust
fn apply_publish_first_file(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    content_choice: &ContentChoice,
    write_time_seconds: u64,
) -> StepOutcome {
    let HistoryPool {
        devices, session, ..
    } = pool;
    let Some(session) = session.as_mut() else {
        return StepOutcome::NotApplicable(MissingPrecondition::NoWritableSession);
    };
    let content = content_choice.bytes();
    let root_to_carry_instance_table_from = *session.current.root();
    let previous_record_bytes = session.current.record_bytes().to_vec();
    let records_before = session.allocator.records().to_vec();
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    match publish_first_file(
        &mut writer,
        &mut session.allocator,
        &root_to_carry_instance_table_from,
        FirstFile {
            content: &content,
            write_time_seconds,
        },
        session.instance,
        &previous_record_bytes,
    ) {
        Ok(output) => {
            let reuse = RecordReuse::between(&records_before, session.allocator.records());
            session.current = PoolVersion::WithFile(output);
            session.publishes_in_this_mount += 1;
            StepOutcome::Applied(AppliedEffect::Published { reuse })
        }
        Err(error) => StepOutcome::Refused {
            member: publish_error_member(&error),
        },
    }
}
```

### crates/singlefs-harness/src/history.rs:1250-1289（apply_publish_overwrite）

```rust
fn apply_publish_overwrite(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    content_choice: &ContentChoice,
    write_time_seconds: u64,
) -> StepOutcome {
    let HistoryPool {
        devices, session, ..
    } = pool;
    let Some(session) = session.as_mut() else {
        return StepOutcome::NotApplicable(MissingPrecondition::NoWritableSession);
    };
    let PoolVersion::WithFile(previous) = &session.current else {
        return StepOutcome::NotApplicable(MissingPrecondition::CurrentVersionWithoutFile);
    };
    let previous = previous.clone();
    let content = content_choice.bytes();
    let records_before = session.allocator.records().to_vec();
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    match publish_overwrite(
        &mut writer,
        &mut session.allocator,
        &previous,
        FirstFile {
            content: &content,
            write_time_seconds,
        },
        session.instance,
    ) {
        Ok(output) => {
            let reuse = RecordReuse::between(&records_before, session.allocator.records());
            session.current = PoolVersion::WithFile(output);
            session.publishes_in_this_mount += 1;
            StepOutcome::Applied(AppliedEffect::Published { reuse })
        }
        Err(error) => StepOutcome::Refused {
            member: publish_error_member(&error),
        },
    }
}
```

### crates/singlefs-harness/src/history.rs:1291-1329（apply_publish_without_units）

```rust
fn apply_publish_without_units(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
) -> StepOutcome {
    let HistoryPool {
        devices, session, ..
    } = pool;
    let Some(session) = session.as_mut() else {
        return StepOutcome::NotApplicable(MissingPrecondition::NoWritableSession);
    };
    // 前提：只在树表 0 条的一版上调（出处见 `MissingPrecondition::CurrentVersionWithFile` 的注释）。
    let PoolVersion::WithoutFile(previous) = &session.current else {
        return StepOutcome::NotApplicable(MissingPrecondition::CurrentVersionWithFile);
    };
    let plan = ZeroUnitPublishPlan {
        txg: CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
        counter: previous.record.counter + 1,
        instance: session.instance,
        back_chain: back_chain_of(&previous.record_bytes),
        rollback_floor: previous.root.rollback_floor,
    };
    let previous_root = previous.root;
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    match publish_without_units(&mut writer, &previous_root, plan) {
        Ok(output) => {
            session.current = PoolVersion::WithoutFile(output);
            session.publishes_in_this_mount += 1;
            StepOutcome::Applied(AppliedEffect::Published {
                reuse: RecordReuse::default(),
            })
        }
        Err(error) => StepOutcome::Refused {
            member: format!(
                "publish_without_units({})",
                block_device_error_member(&error)
            ),
        },
    }
}
```

### crates/singlefs-harness/src/history.rs:1357-1365（apply_mount_writable）

```rust
fn apply_mount_writable(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
) -> StepOutcome {
    pool.session = None;
    pool.mount_attempts += 1;
    let mounted = mount_writable(parameters, &mut pool.devices);
    settle_mount(pool, mounted)
}
```

### crates/singlefs-harness/src/history.rs:1367-1395（apply_mount_rollback）

```rust
fn apply_mount_rollback(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    choice: RollbackTargetChoice,
) -> StepOutcome {
    pool.session = None;
    let (roots, _) = ring_roots_newest_first(&pool.image());
    let Some((newest_txg, newest_instance)) = roots.first().copied() else {
        return StepOutcome::NotApplicable(MissingPrecondition::NoReadableRootInRing);
    };
    let target = match choice {
        RollbackTargetChoice::RingRoot { index_from_newest } => {
            let root_count = u64::try_from(roots.len()).expect("根环至多几十条");
            let (txg, instance) =
                roots[usize::try_from(index_from_newest % root_count).expect("小于条数")];
            RollbackTarget {
                instance: InstanceGeneration(instance),
                checkpoint_txg: CheckpointTxg(txg),
            }
        }
        RollbackTargetChoice::BeyondNewestRoot { txg_beyond_newest } => RollbackTarget {
            instance: InstanceGeneration(newest_instance),
            checkpoint_txg: CheckpointTxg(newest_txg + 1 + txg_beyond_newest % 3),
        },
    };
    pool.mount_attempts += 1;
    let mounted = mount_rollback(parameters, &mut pool.devices, target, ShadowLedger::On);
    settle_mount(pool, mounted)
}
```

### crates/singlefs-harness/src/history.rs:1397-1445（apply_raise_rollback_floor）

```rust
fn apply_raise_rollback_floor(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    choice: FloorTargetChoice,
) -> StepOutcome {
    let HistoryPool {
        devices, session, ..
    } = pool;
    let Some(session) = session.as_mut() else {
        return StepOutcome::NotApplicable(MissingPrecondition::NoWritableSession);
    };
    let WritableSession {
        allocator,
        current,
        publishes_in_this_mount,
        ..
    } = session;
    let PoolVersion::WithFile(current) = current else {
        return StepOutcome::NotApplicable(MissingPrecondition::CurrentVersionWithoutFile);
    };
    let current_floor = current.root.rollback_floor.0;
    let txg_before_raise = current.root.checkpoint_txg.0;
    // 前提：目标只取现行 F 及以上（出处见 `FloorTargetChoice::steps_above_current_floor` 的注释）。
    let choices = txg_before_raise.saturating_sub(current_floor) + 3;
    let new_floor = CheckpointTxg(current_floor + choice.steps_above_current_floor % choices);
    let records_before = allocator.records().to_vec();
    let raised = raise_rollback_floor(
        parameters,
        devices,
        allocator,
        current,
        new_floor,
        ShadowLedger::On,
    );
    // 抬 F 的空发布逐次把现行版本往前推：半路报错时已经推出去的那几次也算这次挂载写出的根。
    *publishes_in_this_mount += usize::try_from(current.root.checkpoint_txg.0 - txg_before_raise)
        .expect("一次抬 F 至多推根环区域数那么多次");
    match raised {
        Ok(raised) => StepOutcome::Applied(AppliedEffect::RaisedFloor {
            new_floor,
            publishes: raised.publishes.len(),
            reclaimed_placements: raised.reclaimed.len(),
            reuse: RecordReuse::between(&records_before, allocator.records()),
        }),
        Err(error) => StepOutcome::Refused {
            member: mount_error_member(&error),
        },
    }
}
```

### crates/singlefs-harness/src/history.rs:1447-1453（apply_cold_start_recover）

```rust
fn apply_cold_start_recover(pool: &mut HistoryPool) -> StepOutcome {
    pool.session = None;
    let report = recover(&pool.devices, JournalPolicy::Consult);
    StepOutcome::Applied(AppliedEffect::Recovered {
        outcome: recovery_outcome_member(&report.outcome),
    })
}
```

### crates/singlefs-harness/src/history.rs:1518-1654（execute_history_observing）

```rust
/// 跑一段历史：起点之后与每一步操作之后都对镜像跑池级 checker，再把那一刻的镜像交给观察者。第一次失败（违例、panic）就停。
/// 盘上的写与屏障录进 `stream`（崩溃注入给开了内容保留的流）。
pub fn execute_history_observing(
    history: &GeneratedHistory,
    stream: &SharedStream,
    observer: &mut dyn FnMut(&StepObservation<'_>),
) -> HistoryRun {
    let mut tally = HistoryTally::default();
    let mut outcomes: Vec<StepOutcome> = Vec::new();
    let mut pool_slot: Option<HistoryPool> = None;
    let position = Cell::new(StepPosition::StartingPoint);
    let mut completed_after_the_root_ring_turned = false;
    let body_result = with_panic_capture(|| -> Option<FailureObservation> {
        let pool = pool_slot.insert(HistoryPool::start(history.starting_point, stream));
        let mut image = pool.image();
        let mut checked_stream_length = stream.operation_count();
        let violations_after_the_starting_point = violations_on(&image, &mut tally);
        if !violations_after_the_starting_point.is_empty() {
            return Some(failure_observation(
                &image,
                StepPosition::StartingPoint,
                None,
                violations_after_the_starting_point,
                None,
            ));
        }
        observer(&StepObservation {
            position: StepPosition::StartingPoint,
            operation: None,
            outcome: None,
            image: &image,
        });
        for (step_index, operation) in history.operations.iter().enumerate() {
            let step_position = StepPosition::Operation(step_index);
            position.set(step_position);
            let outcome = apply_operation(pool, operation, step_index);
            tally.note_outcome(operation, &outcome);
            if let Some(session) = &pool.session {
                tally.most_publishes_in_one_mount = tally
                    .most_publishes_in_one_mount
                    .max(session.publishes_in_this_mount);
                tally.highest_checkpoint_txg = tally
                    .highest_checkpoint_txg
                    .max(session.current.root().checkpoint_txg.0);
            }
            outcomes.push(outcome);
            let stream_length = stream.operation_count();
            if stream_length == checked_stream_length {
                tally.checker_runs_skipped_because_nothing_was_written += 1;
            } else {
                image = pool.image();
                checked_stream_length = stream_length;
                let violations = violations_on(&image, &mut tally);
                if !violations.is_empty() {
                    return Some(failure_observation(
                        &image,
                        step_position,
                        Some(operation.kind()),
                        violations,
                        None,
                    ));
                }
            }
            observer(&StepObservation {
                position: step_position,
                operation: Some(operation),
                outcome: outcomes.last(),
                image: &image,
            });
        }
        let (newest_ring_root_txg, root_ring_slot_count) = newest_ring_root_and_slot_count(&image);
        completed_after_the_root_ring_turned =
            root_ring_has_turned(newest_ring_root_txg, root_ring_slot_count);
        None
    });
    let failure = match body_result {
        Ok(failure) => failure,
        Err(panic) => {
            let step_position = position.get();
            let operation_kind = match step_position {
                StepPosition::StartingPoint => None,
                StepPosition::Operation(step_index) => history
                    .operations
                    .get(step_index)
                    .map(HistoryOperation::kind),
            };
            // panic 之后的盘面照样读一遍根环（读本身也包在捕获里：坏到 checker 的解析也 panic 时只丢掉这两个事实）。
            let (newest_ring_root_txg, root_ring_slot_count) = pool_slot
                .as_ref()
                .and_then(|pool| {
                    with_panic_capture(|| newest_ring_root_and_slot_count(&pool.image())).ok()
                })
                .unwrap_or((None, None));
            Some(FailureObservation {
                position: step_position,
                operation_kind,
                violations: Vec::new(),
                panic: Some(panic),
                newest_ring_root_txg,
                root_ring_slot_count,
            })
        }
    };
    if let Some(pool) = &pool_slot {
        tally.most_successful_mounts_in_one_history = pool.successful_mounts;
        tally.most_mount_attempts_in_one_history = pool.mount_attempts;
    }
    let ending = match failure {
        None => {
            tally.histories_completed = 1;
            if completed_after_the_root_ring_turned {
                tally.histories_that_turned_the_root_ring = 1;
            }
            HistoryEnding::Completed
        }
        Some(observation) => {
            if observation.root_ring_has_turned() {
                tally.histories_that_turned_the_root_ring = 1;
            }
            let ending = classify_failure(observation);
            match &ending {
                HistoryEnding::KnownRed { form, .. } => {
                    tally.histories_ended_known_red.insert(*form, 1);
                }
                HistoryEnding::NewFinding { .. } => tally.histories_ended_new_finding = 1,
                HistoryEnding::Completed => {}
            }
            ending
        }
    };
    tally.histories = 1;
    HistoryRun {
        ending,
        outcomes,
        tally,
    }
}
```

### crates/singlefs-harness/src/history.rs:1483-1497（violations_on）

```rust
/// 对一份镜像跑池级 checker，记每条不变量判绿、不适用各几次，交回判红的那几条。
fn violations_on(image: &MemoryPool, tally: &mut HistoryTally) -> Vec<(&'static str, String)> {
    tally.checker_runs += 1;
    let mut violations = Vec::new();
    for (invariant, verdict) in check_pool_image(image) {
        match verdict {
            InvariantVerdict::Holds => *tally.invariant_holds.entry(invariant).or_insert(0) += 1,
            InvariantVerdict::NotApplicable(_) => {
                *tally.invariant_not_applicable.entry(invariant).or_insert(0) += 1;
            }
            InvariantVerdict::Violated(detail) => violations.push((invariant, detail)),
        }
    }
    violations
}
```

### crates/singlefs-harness/src/history.rs:760-762（KNOWN_RED_FORMS）

```rust
/// 「已知红」清单。修好一条就删一条，删掉之后那条的复现（`tests/second_transaction_supplement_three_random_history.rs` 里钉着）要转绿。
/// 第 0 条的宽度（转圈跨几次挂载也算）2026-09-18 主 agent 定案保留，记在收口表第 ② 行。
pub const KNOWN_RED_FORMS: [KnownRedForm; 2] = [
```

### crates/singlefs-harness/src/history.rs:736-745（only_allocated_statistic_above_walked）

```rust
/// 没有 panic、判红的只有 I-3.1、而且是记账的已分配大于遍历全部有效根得到的（记账多算，不是少算）。
fn only_allocated_statistic_above_walked(observation: &FailureObservation) -> bool {
    observation.panic.is_none()
        && !observation.violations.is_empty()
        && observation.violations.iter().all(|(invariant, detail)| {
            *invariant == "I-3.1"
                && allocated_and_walked_bytes(detail)
                    .is_some_and(|(allocated, walked)| allocated > walked)
        })
}
```

### crates/singlefs-harness/src/history.rs:747-749（ring_turn_leaves_allocated_statistic_above_walked）

```rust
fn ring_turn_leaves_allocated_statistic_above_walked(observation: &FailureObservation) -> bool {
    observation.root_ring_has_turned() && only_allocated_statistic_above_walked(observation)
}
```

### crates/singlefs-harness/src/history.rs:751-758（raise_after_rollback_leaves_allocated_statistic_above_walked）

```rust
/// 只看「抬 F 那一步之后、根环没转圈、只有 I-3.1 红且记账多于遍历」，不看 F 是否落在回退留下的空档里：观察里没有回退历史。
fn raise_after_rollback_leaves_allocated_statistic_above_walked(
    observation: &FailureObservation,
) -> bool {
    observation.operation_kind == Some(HistoryOperationKind::RaiseRollbackFloor)
        && !observation.root_ring_has_turned()
        && only_allocated_statistic_above_walked(observation)
}
```

### crates/singlefs-harness/src/history.rs:775-780（FailureSignature）

```rust
/// 新发现的「同一个」：panic 按位置、违例按判红的不变量集合。收缩只留签名不变的删法。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FailureSignature {
    Panic { location: String },
    CheckerViolations { invariants: Vec<&'static str> },
}
```

### crates/singlefs-harness/src/history.rs:1499-1510（classify_failure）

```rust
fn classify_failure(observation: FailureObservation) -> HistoryEnding {
    match KNOWN_RED_FORMS
        .iter()
        .position(|form| (form.matches)(&observation))
    {
        Some(form) => HistoryEnding::KnownRed { form, observation },
        None => HistoryEnding::NewFinding {
            signature: FailureSignature::of(&observation),
            observation,
        },
    }
}
```

### crates/singlefs-harness/src/history.rs:1694-1759（simpler_variants）

```rust
/// 一步操作的几种更简单的写法，只往简单的方向换（换过去的不会再换回来，收缩因此有界）：内容按 0 字节 < 1 字节 < 别的排，只试排在
/// 现在这个前面的；回退目标与抬 F 目标只试比现在小的选择子（选择子执行时按条数取模：一个很大的选择子与某个小数落到同一条根、同一个 F，
/// 从小到大试，第一个还失败的就是它的小写法）。
fn simpler_variants(operation: HistoryOperation) -> Vec<HistoryOperation> {
    const EMPTY_CONTENT: ContentChoice = ContentChoice {
        length: ContentLength::Empty,
        fill_seed: 0,
    };
    const SHORTEST_NON_EMPTY_CONTENT: ContentChoice = ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 0 },
        fill_seed: 0,
    };
    let simplicity_rank = |content: ContentChoice| {
        if content == EMPTY_CONTENT {
            0
        } else if content == SHORTEST_NON_EMPTY_CONTENT {
            1
        } else {
            2
        }
    };
    let simpler_contents = move |content: ContentChoice| {
        [EMPTY_CONTENT, SHORTEST_NON_EMPTY_CONTENT]
            .into_iter()
            .filter(move |simpler| simplicity_rank(*simpler) < simplicity_rank(content))
    };
    let smaller_selectors = |selector: u64| 0..selector.min(SMALL_SELECTORS_TRIED_WHEN_SHRINKING);
    match operation {
        HistoryOperation::PublishFirstFile(content) => simpler_contents(content)
            .map(HistoryOperation::PublishFirstFile)
            .collect(),
        HistoryOperation::PublishOverwrite(content) => simpler_contents(content)
            .map(HistoryOperation::PublishOverwrite)
            .collect(),
        HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
            index_from_newest,
        }) => smaller_selectors(index_from_newest)
            .map(|smaller| {
                HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
                    index_from_newest: smaller,
                })
            })
            .collect(),
        HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::BeyondNewestRoot {
            txg_beyond_newest,
        }) => smaller_selectors(txg_beyond_newest)
            .map(|smaller| {
                HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::BeyondNewestRoot {
                    txg_beyond_newest: smaller,
                })
            })
            .collect(),
        HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
            steps_above_current_floor,
        }) => smaller_selectors(steps_above_current_floor)
            .map(|smaller| {
                HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                    steps_above_current_floor: smaller,
                })
            })
            .collect(),
        HistoryOperation::PublishWithoutUnits
        | HistoryOperation::CloseAndMountWritable
        | HistoryOperation::ColdStartRecover => Vec::new(),
    }
}
```

### crates/singlefs-harness/src/history.rs:1761-1833（shrink_operations）

```rust
/// 收缩到最短：先按块删（块长从一半起、删不动就减半，到 1 为止），再逐步换成更简单的写法；`still_fails` 为真的删法与换法才留下。
/// 次数有界：每一轮要么少一步，要么块长减半，块长为 1 且一步都删不动就停。
///
/// 删法按窗口并行试：从当前块起连着 `worker_threads` 块各删一块、同时跑，取块号最小的那个还失败的——与一块一块按次序试的结果逐项相同
/// （按次序试时排在它前面的那几块在同一份序列上试、同样不失败），只是多跑了窗口里排在它后面的几个。
pub fn shrink_operations(
    operations: &[HistoryOperation],
    still_fails: &(dyn Fn(&[HistoryOperation]) -> bool + Sync),
    worker_threads: usize,
) -> Vec<HistoryOperation> {
    let window = worker_threads.max(1);
    let mut kept = operations.to_vec();
    let mut chunk_length = (kept.len() / 2).max(1);
    loop {
        let mut removed_this_pass = false;
        let mut chunk_start = 0;
        while chunk_start < kept.len() {
            let window_starts: Vec<usize> = (0..window)
                .map(|offset| chunk_start + offset * chunk_length)
                .take_while(|start| *start < kept.len())
                .collect();
            let candidates: Vec<Vec<HistoryOperation>> = window_starts
                .iter()
                .map(|start| {
                    let end = (start + chunk_length).min(kept.len());
                    kept[..*start].iter().chain(&kept[end..]).copied().collect()
                })
                .collect();
            let verdicts: Vec<bool> = std::thread::scope(|scope| {
                let evaluations: Vec<_> = candidates
                    .iter()
                    .map(|candidate| scope.spawn(move || still_fails(candidate)))
                    .collect();
                evaluations
                    .into_iter()
                    .map(|evaluation| {
                        evaluation.join().expect(
                            "still_fails 自己不 panic：历史里的 panic 在 execute_history 里接住",
                        )
                    })
                    .collect()
            });
            match verdicts.iter().position(|fails| *fails) {
                Some(first_failing) => {
                    kept = candidates[first_failing].clone();
                    chunk_start = window_starts[first_failing];
                    removed_this_pass = true;
                }
                None => {
                    chunk_start =
                        window_starts.last().expect("窗口里至少有当前这一块") + chunk_length;
                }
            }
        }
        if !removed_this_pass {
            if chunk_length == 1 {
                break;
            }
            chunk_length = (chunk_length / 2).max(1);
        }
    }
    for step_index in 0..kept.len() {
        for simpler in simpler_variants(kept[step_index]) {
            let mut candidate = kept.clone();
            candidate[step_index] = simpler;
            if still_fails(&candidate) {
                kept = candidate;
                break;
            }
        }
    }
    kept
}
```

### crates/singlefs-harness/src/history.rs:1835-1870（shrink_failing_history）

```rust
/// 一段失败的历史收缩到最短：起点不变，签名不变的删法与换法才留下。先截掉失败那一步之后的操作。
#[must_use]
pub fn shrink_failing_history(
    history: &GeneratedHistory,
    signature: &FailureSignature,
    worker_threads: usize,
) -> GeneratedHistory {
    let still_fails = |operations: &[HistoryOperation]| {
        let candidate = GeneratedHistory {
            seed: history.seed,
            starting_point: history.starting_point,
            operations: operations.to_vec(),
        };
        matches!(
            execute_history(&candidate).ending,
            HistoryEnding::NewFinding { signature: found, .. } if found == *signature
        )
    };
    let failing_length = match execute_history(history).ending {
        HistoryEnding::NewFinding { observation, .. } => match observation.position {
            StepPosition::StartingPoint => 0,
            StepPosition::Operation(step_index) => step_index + 1,
        },
        HistoryEnding::Completed | HistoryEnding::KnownRed { .. } => history.operations.len(),
    };
    let operations = shrink_operations(
        &history.operations[..failing_length],
        &still_fails,
        worker_threads,
    );
    GeneratedHistory {
        seed: history.seed,
        starting_point: history.starting_point,
        operations,
    }
}
```

### crates/singlefs-harness/src/history.rs:2004-2093（run_history_campaign）

```rust
/// 跑种子 [`first_seed`, `first_seed` + `seed_count`)，每段 `operations_per_history` 步，分给 `worker_threads` 个线程（每段历史各用各的盘、
/// 各用各的录制流，互不相干）；跑完按种子排好再汇总，结论与线程数、调度次序无关。新发现按签名归类，按 `shrinking` 收缩各类的第一个种子。
///
/// # Panics
/// 某个线程在历史之外 panic（历史里的 panic 在 `execute_history` 里接住，走不到这里）。
#[must_use]
pub fn run_history_campaign(
    first_seed: u64,
    seed_count: u64,
    operations_per_history: usize,
    worker_threads: usize,
    shrinking: FindingShrinking,
) -> CampaignReport {
    let next_offset = AtomicU64::new(0);
    let finished: Mutex<Vec<(GeneratedHistory, HistoryRun)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..worker_threads.max(1) {
            scope.spawn(|| loop {
                let offset = next_offset.fetch_add(1, Ordering::Relaxed);
                if offset >= seed_count {
                    break;
                }
                let history =
                    generate_history(HistorySeed(first_seed + offset), operations_per_history);
                let run = execute_history(&history);
                finished
                    .lock()
                    .expect("别的线程拿着这把锁时只做一次 push，不会在锁里 panic")
                    .push((history, run));
            });
        }
    });
    let mut finished = finished
        .into_inner()
        .expect("线程都已结束；锁里只做 push，没有线程在锁里 panic");
    finished.sort_by_key(|(history, _)| history.seed);
    let mut tally = HistoryTally::default();
    let mut known_red_hits = Vec::new();
    let mut findings: BTreeMap<
        FailureSignature,
        (GeneratedHistory, FailureObservation, Vec<HistorySeed>),
    > = BTreeMap::new();
    for (history, run) in finished {
        tally.absorb(&run.tally);
        match run.ending {
            HistoryEnding::Completed => {}
            HistoryEnding::KnownRed { form, observation } => {
                known_red_hits.push((history.seed, form, observation.position));
            }
            HistoryEnding::NewFinding {
                signature,
                observation,
            } => {
                findings
                    .entry(signature)
                    .or_insert_with(|| (history.clone(), observation, Vec::new()))
                    .2
                    .push(history.seed);
            }
        }
    }
    let mut by_first_seed: Vec<_> = findings.into_iter().collect();
    by_first_seed.sort_by_key(|(_, (history, _, _))| history.seed);
    let new_findings = by_first_seed
        .into_iter()
        .map(|(signature, (history, observation, seeds))| {
            let shrunk = match shrinking {
                FindingShrinking::EveryFindingClass => {
                    Some(shrink_to_reproduction(&history, &signature, worker_threads))
                }
                FindingShrinking::ReportSeedsOnly => None,
            };
            NewFindingReport {
                signature,
                first_seed: history.seed,
                observation,
                seeds,
                shrunk,
            }
        })
        .collect();
    CampaignReport {
        first_seed,
        seed_count,
        operations_per_history,
        tally,
        known_red_hits,
        new_findings,
    }
}
```

### crates/singlefs-harness/src/history.rs:824-853（HistoryTally）

```rust
/// 跑过的历史的计数：证明各条路径真的跑到了（`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HistoryTally {
    pub histories: u64,
    pub histories_completed: u64,
    /// 「已知红」清单第几条 → 几段历史以它收尾。
    pub histories_ended_known_red: BTreeMap<usize, u64>,
    pub histories_ended_new_finding: u64,
    pub operations_by_kind: BTreeMap<HistoryOperationKind, OperationTally>,
    /// 发布（第一个文件、覆盖写）的内容长度，按长度种类 → 入口被调了几次。
    pub content_lengths_attempted: BTreeMap<&'static str, u64>,
    pub refusals_by_member: BTreeMap<String, u64>,
    pub not_applicable_by_precondition: BTreeMap<&'static str, u64>,
    pub recovery_outcomes: BTreeMap<String, u64>,
    pub most_publishes_in_one_mount: usize,
    pub most_successful_mounts_in_one_history: usize,
    pub most_mount_attempts_in_one_history: usize,
    pub raises_that_reclaimed: u64,
    pub reclaimed_placements: u64,
    pub records_rewritten_from_released: u64,
    pub records_rewritten_with_changed_span: u64,
    pub released_records_removed: u64,
    pub highest_checkpoint_txg: u64,
    pub histories_that_turned_the_root_ring: u64,
    pub checker_runs: u64,
    /// 这一步一个写都没发（录制流一步没多）：镜像逐字节不变，checker 的结论沿用上一次，不重跑。
    pub checker_runs_skipped_because_nothing_was_written: u64,
    pub invariant_holds: BTreeMap<&'static str, u64>,
    pub invariant_not_applicable: BTreeMap<&'static str, u64>,
}
```
### crates/singlefs-harness/src/history.rs:707-713（impl FailureObservation）

```rust
impl FailureObservation {
    /// 根环转过一圈：盘上最新根的 txg ≥ R × S，第 0 代根的槽已被盖过。
    #[must_use]
    pub fn root_ring_has_turned(&self) -> bool {
        root_ring_has_turned(self.newest_ring_root_txg, self.root_ring_slot_count)
    }
}
```

### crates/singlefs-harness/src/history.rs:1680-1689（root_ring_has_turned，独立函数；主 agent 给的是 1680-1690，第 1690 行核实是空行、不是收尾的花括号，收尾花括号在第 1689 行，这里按核实结果取到 1689）

```rust
/// 根环转过一圈：盘上最新根的 txg ≥ R × S，第 0 代根的槽已被盖过。
fn root_ring_has_turned(
    newest_ring_root_txg: Option<u64>,
    root_ring_slot_count: Option<u64>,
) -> bool {
    matches!(
        (newest_ring_root_txg, root_ring_slot_count),
        (Some(newest), Some(slot_count)) if newest >= slot_count
    )
}
```

## 三、新文件 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`（未进 git）：全部用例按项抽取（`quote-rust-items.py`，6/6 项；回读逐字节比对）

### crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:114-137（random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation）

```rust
/// 快档：种子 [0, 96)、每段 30 步。每一步之后跑池级 checker；入口的 `Err` 算合法结局，panic 与违例算失败，撞到「已知红」清单里的形态
/// 照记、那段到此为止，清单外的一条都不许有（有就按签名归类、报出种子与失败在哪一步；收缩交给「收缩一个种子」那条 `#[ignore]` 用例，
/// debug 下收缩一类要两分多钟）。计数照打，并核各条路径真的跑到了。
#[test]
fn random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation() {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        FAST_TIER_FIRST_SEED,
        FAST_TIER_SEEDS,
        FAST_TIER_OPERATIONS_PER_HISTORY,
        worker_threads_by_default(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史快档 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "「已知红」清单外的失败：\n{rendered}"
    );
    assert_every_path_was_exercised(&report.tally);
}
```

### crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:139-188（turning_the_root_ring_with_overwrites_in_one_mount_ends_in_the_first_known_red_form）

```rust
/// 「已知红」清单第 0 条（增补 2 收口表第 ② 行）的复现：第一个文件之后可写挂载一次（写行 txg 4、暖机 txg 5），同一次挂载里连着覆盖写。
/// 根环 R × S = 24 槽：txg 24、25、26 依次盖掉第 0 代根与两条暖机根，txg 26 那次覆盖写之后再没有一条有效根引用 mkfs 的第 0 版树表单元
/// （1 槽），它在 txg 3 释放、没回收，记账仍算已分配 ⇒ checker 判 I-3.1 红（记账比遍历多 16384 字节）、别的不变量不红，分类成清单第 0 条；
/// 之前每一步之后 checker 全绿。
/// 这一条修好之后本用例要红——那时把清单第 0 条删掉、这里改成「跑完」。
#[test]
fn turning_the_root_ring_with_overwrites_in_one_mount_ends_in_the_first_known_red_form() {
    let overwrite = HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 2999 },
        fill_seed: 1,
    });
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
            .chain(std::iter::repeat_n(overwrite, 25))
            .collect(),
    };
    let run = execute_history(&history);
    let HistoryEnding::KnownRed { form, observation } = &run.ending else {
        panic!("要以已知红收尾：{:?}", run.ending);
    };
    assert_eq!(*form, 0, "{}", KNOWN_RED_FORMS[*form].shape);
    assert_eq!(
        observation.position,
        StepPosition::Operation(21),
        "第 21 步是 txg 26 那次覆盖写（txg 4、5 是写行与暖机，覆盖写从 txg 6 起）"
    );
    assert_eq!(observation.newest_ring_root_txg, Some(26));
    assert_eq!(observation.root_ring_slot_count, Some(24));
    assert_eq!(
        observation
            .violations
            .iter()
            .map(|(invariant, _)| *invariant)
            .collect::<Vec<_>>(),
        vec!["I-3.1"]
    );
    let (allocated, walked) = allocated_and_walked_bytes(&observation.violations[0].1)
        .expect("I-3.1 的违例文字带记账与遍历两个数");
    assert_eq!(
        allocated - walked,
        16384,
        "差的正是 mkfs 那 1 槽第 0 版树表单元"
    );
    assert_eq!(
        run.tally.checker_runs, 23,
        "起点、挂载、前 20 次覆盖写之后各跑一次都是绿的，第 21 次覆盖写之后那一次判红"
    );
}
```

### crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:190-246（raising_the_floor_into_the_gap_left_by_a_rollback_ends_in_the_second_known_red_form）

```rust
/// 「已知红」清单第 1 条（增补 2 收口表第 43 行）的复现，2026-09-18 在快档种子 80 上撞到、收缩出来的那一段（种子号随生成器的比重变，
/// 这一段不随）：可写挂载（实例 2，txg 4、5）、覆盖写两次（6、7）、回退到 (2, 7)（实例 3，txg 8–10）、再回退到 (2, 7)（实例 4，
/// txg 11–13：实例 3 的三条根被抛弃）、覆盖写（14）、可写挂载（实例 5，txg 15、16）、覆盖写三次（17–19）、抬 F 到 8（txg 20–22）。
/// F = 8 那个 txg 上的根被抛弃了，(2, 7) 落到 F 之下出了候选集，而它的实例表与四个固定点单元（6 槽）在 txg 11 才释放、释放代 11 > 8
/// 不回收 ⇒ 记账比遍历多 6 × 16384 字节，I-3.1 红。这一条修好之后本用例要红——那时把清单第 1 条删掉、这里改成「跑完」。
#[test]
fn raising_the_floor_into_the_gap_left_by_a_rollback_ends_in_the_second_known_red_form() {
    let empty = ContentChoice {
        length: ContentLength::Empty,
        fill_seed: 0,
    };
    let history = GeneratedHistory {
        seed: HistorySeed(80),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
                index_from_newest: 0,
            }),
            HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
                index_from_newest: 3,
            }),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 8,
            }),
        ],
    };
    let run = execute_history(&history);
    let HistoryEnding::KnownRed { form, observation } = &run.ending else {
        panic!("要以已知红收尾：{:?}", run.ending);
    };
    assert_eq!(*form, 1, "{}", KNOWN_RED_FORMS[*form].shape);
    assert_eq!(observation.position, StepPosition::Operation(10));
    assert_eq!(
        run.outcomes[10],
        StepOutcome::Applied(AppliedEffect::RaisedFloor {
            new_floor: CheckpointTxg(8),
            publishes: 3,
            reclaimed_placements: 26,
            reuse: RecordReuse::default(),
        })
    );
    let (allocated, walked) = allocated_and_walked_bytes(&observation.violations[0].1)
        .expect("I-3.1 的违例文字带记账与遍历两个数");
    assert_eq!(
        allocated - walked,
        6 * 16384,
        "(2, 7) 的实例表 2 槽与四个固定点单元各 1 槽"
    );
}
```

### crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:248-273（the_same_seed_runs_to_the_same_outcomes_and_the_same_bytes_twice）

```rust
/// 同一个种子跑两次，每一步的结局、收尾、计数逐项相同，每一步之后的整份镜像逐字节相同（随机源手写、写入时间是参数）。
#[test]
fn the_same_seed_runs_to_the_same_outcomes_and_the_same_bytes_twice() {
    let history = generate_history(HistorySeed(7), 24);
    let run_keeping_every_image = || {
        let mut images: Vec<MemoryPool> = Vec::new();
        let run = execute_history_observing(&history, &SharedStream::new(), &mut |observation| {
            images.push(observation.image.clone());
        });
        (run, images)
    };
    let (first, first_images) = run_keeping_every_image();
    let (second, second_images) = run_keeping_every_image();
    assert_eq!(first, second);
    assert!(
        first.tally.checker_runs >= 2,
        "起点之后与至少一步写之后都跑了 checker：{}",
        first.tally.checker_runs
    );
    assert!(
        first_images.len() >= 2,
        "观察者至少见到起点与一步之后的镜像：{}",
        first_images.len()
    );
    assert!(first_images == second_images, "每一步之后的镜像逐字节相同");
}
```

### crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:275-308（random_histories_large_tier_seeds_and_length_from_the_environment）

```rust
/// 大档：种子数、每段步数、第一个种子、线程数从环境变量取。
#[test]
#[ignore = "大档：SINGLEFS_RANDOM_HISTORY_SEEDS 段、每段 SINGLEFS_RANDOM_HISTORY_OPERATIONS 步，从 SINGLEFS_RANDOM_HISTORY_FIRST_SEED 起，SINGLEFS_RANDOM_HISTORY_THREADS 个线程；release 下后台跑"]
fn random_histories_large_tier_seeds_and_length_from_the_environment() {
    let first_seed = number_from_environment("SINGLEFS_RANDOM_HISTORY_FIRST_SEED", 0);
    let seed_count = number_from_environment("SINGLEFS_RANDOM_HISTORY_SEEDS", 1000);
    let operations_per_history = usize::try_from(number_from_environment(
        "SINGLEFS_RANDOM_HISTORY_OPERATIONS",
        60,
    ))
    .expect("步数装得进 usize");
    let worker_threads = usize::try_from(number_from_environment(
        "SINGLEFS_RANDOM_HISTORY_THREADS",
        u64::try_from(std::thread::available_parallelism().map_or(1, usize::from)).expect("线程数"),
    ))
    .expect("线程数装得进 usize");
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        first_seed,
        seed_count,
        operations_per_history,
        worker_threads,
        FindingShrinking::EveryFindingClass,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史大档 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "「已知红」清单外的失败：\n{rendered}"
    );
}
```

### crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:310-352（shrink_one_failing_seed_from_the_environment）

```rust
/// 收缩一个种子：快档只报种子，拿这条把它收到最短复现（`SINGLEFS_RANDOM_HISTORY_SHRINK_SEED`，步数默认同快档）。
#[test]
#[ignore = "收缩一个种子：SINGLEFS_RANDOM_HISTORY_SHRINK_SEED 必给，SINGLEFS_RANDOM_HISTORY_OPERATIONS 默认 30，SINGLEFS_RANDOM_HISTORY_THREADS 默认全部核"]
fn shrink_one_failing_seed_from_the_environment() {
    let seed = HistorySeed(
        std::env::var("SINGLEFS_RANDOM_HISTORY_SHRINK_SEED")
            .expect("要给 SINGLEFS_RANDOM_HISTORY_SHRINK_SEED")
            .parse()
            .expect("SINGLEFS_RANDOM_HISTORY_SHRINK_SEED 是一个非负整数"),
    );
    let operations_per_history = usize::try_from(number_from_environment(
        "SINGLEFS_RANDOM_HISTORY_OPERATIONS",
        u64::try_from(FAST_TIER_OPERATIONS_PER_HISTORY).expect("步数"),
    ))
    .expect("步数装得进 usize");
    let worker_threads = usize::try_from(number_from_environment(
        "SINGLEFS_RANDOM_HISTORY_THREADS",
        u64::try_from(std::thread::available_parallelism().map_or(1, usize::from)).expect("线程数"),
    ))
    .expect("线程数装得进 usize");
    let history = generate_history(seed, operations_per_history);
    let HistoryEnding::NewFinding {
        signature,
        observation,
    } = execute_history(&history).ending
    else {
        panic!(
            "种子 {} 的 {operations_per_history} 步没有撞到新发现",
            seed.0
        );
    };
    let shrunk = shrink_to_reproduction(&history, &signature, worker_threads);
    print_uncaptured(
        &NewFindingReport {
            signature,
            first_seed: seed,
            observation,
            seeds: vec![seed],
            shrunk: Some(shrunk),
        }
        .render(),
    );
}
```
