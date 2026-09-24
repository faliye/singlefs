# 附录二：增补 3 第 4 件（故障注入）代码轮第一轮改动（`git diff -- crates/` 原样，加两个新文件全文；生成于 2026-09-21 15:10 UTC）

基准：HEAD ad787cc（2026-09-21 13:05:52 +0000），工作区未提交改动。

## 一、diff（crates/ 下相对 HEAD ad787cc 的工作区改动，`git diff -- crates/`）

```diff
diff --git a/crates/mutations.tsv b/crates/mutations.tsv
index 4c031ed..4dc1c8a 100644
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -103,7 +103,7 @@ Ignore 那一遍恢复不过 oracle	crates/singlefs-harness/src/crash.rs
 增补 2 第 28 行：记账树装不下不判（80 块盘走到装节点的断言 panic）	crates/singlefs-core/src/transaction.rs	    if accounting_entries_of_this_publish > accounting_node_capacity {	    if false && accounting_entries_of_this_publish > accounting_node_capacity {	-p singlefs-harness --test second_transaction_supplement_two_accounting_node_full -- eightieth_device	eightieth_device_overflows_the_accounting_node_and_the_publish_is_refused_before_anything_is_written
 增补 2 第 28 行：记账树正好装满 477 行也判装不下（79 块盘发布被拒）	crates/singlefs-core/src/transaction.rs	    if accounting_entries_of_this_publish > accounting_node_capacity {	    if accounting_entries_of_this_publish >= accounting_node_capacity {	-p singlefs-harness --test second_transaction_supplement_two_accounting_node_full -- seventy_nine_devices	seventy_nine_devices_fill_the_accounting_node_exactly_and_still_publish
 增补 2 第 30 行：真设备二进制的挂载窗口从重开那一刻算起（取号的两次系统配置槽写进了设备一层的数、不在按种类的账里）	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	        pool_writes_between(&counts_after_acquisition, &counts_after_mount),	        pool_writes_between(&vec![DeviceCallCounts { write_calls: 0, written_bytes: 0, force_unit_access_writes: 0, barrier_calls: 0 }; counts_after_mount.len()], &counts_after_mount),	-p singlefs-harness --bin first_transaction_on_device -- second_instance_mode	second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches
-增补 2 第 30 行：真设备二进制每道屏障都覆盖「第一道屏障」那份计数（挂载窗口从最后一道屏障算起）	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	        if self.counts_when_first_barrier_arrived.is_none() {	        if true {	-p singlefs-harness --bin first_transaction_on_device -- second_instance_mode	second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches
+增补 2 第 30 行：每道屏障都覆盖「第一道屏障」那份计数（挂载窗口从最后一道屏障算起；2026-09-21 随通用故障注入包装搬到 fault_injection.rs）	crates/singlefs-harness/src/fault_injection.rs	        state\n            .counts_when_the_first_barrier_arrived\n            .entry(device)\n            .or_insert(counts);	        state\n            .counts_when_the_first_barrier_arrived\n            .insert(device, counts);	-p singlefs-harness --bin first_transaction_on_device -- second_instance_mode	second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches
 增补 2 第 20a 行：写行那次发布的准入不在取号之前算（取号之后发布才被拒，实例代号已经烧掉）	crates/singlefs-core/src/mount.rs	    refuse_publishes_before_acquisition_that_do_not_pass_admission(\n        &allocator,\n        &start,\n        instance_to_acquire,\n        rows_written.len(),\n        warm_up_publishes_planned.len(),\n    )?;\n	    // 变异：写行那次发布的准入不在取号之前算\n	-p singlefs-harness --test second_transaction_supplement_two_row_publish_admission -- writable_mount_that_cannot_publish	writable_mount_that_cannot_publish_the_rows_is_refused_before_the_instance_is_acquired
 增补 2 第 20b 行：中途失败的发布不把这次已记的写交出去（与设备一层记的合计对不上）	crates/singlefs-core/src/transaction.rs	    if let Err(cause) = persist(pool) {\n        pool.count_failed_publish(&writes_before_this_publish);\n        return Err(PublishError::BlockDevice(cause));\n    }	    if let Err(cause) = persist(pool) {\n        return Err(PublishError::BlockDevice(cause));\n    }	-p singlefs-harness --test second_transaction_supplement_one_write_accounting -- publish_that_fails_midway	publish_that_fails_midway_hands_out_what_it_already_wrote_so_the_retry_still_adds_up
 增补 2 第 20c 行：用户数据只排除当前那一个开放段（回落把开放段置空之后，那一段对用户数据开放）	crates/singlefs-core/src/allocator.rs	        let cluster_segments = &self.cluster_segments;	        let cluster_segments = &self.open_segment.into_iter().collect::<BTreeSet<SlotNumber>>();	-p singlefs-core --lib -- allocator::tests::user_data_does_not_land	user_data_does_not_land_in_a_cluster_segment_that_the_fallback_closed
@@ -194,3 +194,11 @@ Z1-a：实例表准入按每次挂载只写一行算	crates/singlefs-core/src/mo
 系统配置四类：分档记账把系统运行量记进系统运行配置那个计数器（四档的和仍是 481，分法错了）	crates/singlefs-core/src/system_configuration.rs	            SlotFieldMutability::RuntimeQuantity => &mut self.runtime_quantity_bytes,	            SlotFieldMutability::RuntimeQuantity => &mut self.runtime_configuration_bytes,	-p singlefs-core --lib -- to_slot_writes_exactly_the_budgeted_bytes	to_slot_writes_exactly_the_budgeted_bytes_into_each_mutability_class
 系统配置四类：整理三水位不再写内置默认的 0（字节数不变、盘上 24 字节变了）	crates/singlefs-core/src/system_configuration.rs	        [COMPACTION_WATERMARK_BUILT_IN_DEFAULT; COMPACTION_WATERMARK_COUNT]	        [COMPACTION_WATERMARK_BUILT_IN_DEFAULT + 1; COMPACTION_WATERMARK_COUNT]	-p singlefs-harness --test system_configuration_mutability_classes -- splitting_the_system_configuration	splitting_the_system_configuration_into_four_mutability_classes_changes_no_byte_on_disk
 系统配置四类：节点大小那一段标成系统不可变配置（可改性分错档，改它的代价从重建索引说成重建文件系统）	crates/singlefs-core/src/system_configuration.rs	        slot.write(SlotFieldMutability::MutableConfiguration, |writer| {	        slot.write(SlotFieldMutability::ImmutableConfiguration, |writer| {	-p singlefs-harness --test system_configuration_mutability_classes -- each_mutability_class_writes	each_mutability_class_writes_its_own_field_table_budget_389_4_36_52
+增补 3 第 4 件：通用故障注入包装「第 n 次命中才注入」改成永不命中（注入装置自证：注不进去，判「返回错误」的用例就没有被测对象）	crates/singlefs-harness/src/fault_injection.rs	            FaultOccurrence::TheNthMatchingCall(ordinal) => matching_call_ordinal == ordinal,	            FaultOccurrence::TheNthMatchingCall(_) => false,	-p singlefs-harness --lib -- fault_injection::tests::the_nth_write_across_the_pool	the_nth_write_across_the_pool_fails_once_and_writes_nothing
+增补 3 第 4 件：注入的写报错那一次照样把字节落到盘上（报了错又写了，盘面不是「一个字节都没落」）	crates/singlefs-harness/src/fault_injection.rs	            Some(InjectedFault::WriteFails) => return Err(injected_block_device_error("写")),	            Some(InjectedFault::WriteFails) => {}	-p singlefs-harness --lib -- fault_injection::tests::the_nth_write_across_the_pool	the_nth_write_across_the_pool_fails_once_and_writes_nothing
+增补 3 第 4 件：注入的「读回改坏的字节」其实一位都没翻（坏字节注不进去）	crates/singlefs-harness/src/fault_injection.rs	                    buffer[index] ^= mask;	                    buffer[index] ^= 0;	-p singlefs-harness --lib -- fault_injection::tests::a_corrupted_read	a_corrupted_read_flips_one_bit_in_the_buffer_and_leaves_the_device_alone
+增补 3 第 4 件：被吞掉的那一次写其实落了盘（设备说谎那一形注不进去）	crates/singlefs-harness/src/fault_injection.rs	            Some(InjectedFault::WriteIsSwallowed) => return Ok(()),	            Some(InjectedFault::WriteIsSwallowed) => {}	-p singlefs-harness --lib -- fault_injection::tests::a_swallowed_write	a_swallowed_write_reports_success_and_changes_nothing_on_the_device
+增补 3 第 4 件：每块盘第一道屏障那一刻的计数根本不记（真设备那一路按它切挂载窗口，不记就切不出来）	crates/singlefs-harness/src/fault_injection.rs	        state\n            .counts_when_the_first_barrier_arrived\n            .entry(device)\n            .or_insert(counts);	        let _ = (&mut state, device, counts);	-p singlefs-harness --lib -- fault_injection::tests::the_counts_when_the_first_barrier_arrived	the_counts_when_the_first_barrier_arrived_are_frozen_at_that_moment
+增补 3 第 4 件（被测的性质）：发布落盘阶段的块设备错改成 panic，不再把错返回上来	crates/singlefs-core/src/transaction.rs	    if let Err(cause) = persist(pool) {\n        pool.count_failed_publish(&writes_before_this_publish);\n        return Err(PublishError::BlockDevice(cause));\n    }	    persist(pool).expect("落盘那几步不报错");	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- fault_injection_fast_tier	fault_injection_fast_tier_returns_errors_instead_of_panicking
+增补 3 第 4 件（增补 2 收口表第 40 行 / C381 的判别力）：发布失败时分配器不再退回（C381 的候选改法之一），下一次发布就不会盖掉那条根指着的单元	crates/singlefs-core/src/transaction.rs	    if outcome.is_err() {\n        *allocator = allocator_before_this_publish;\n    }	    if false {\n        *allocator = allocator_before_this_publish;\n    }	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- a_publish_that_fails	a_publish_that_fails_on_the_system_configuration_slot_leaves_a_root_whose_units_the_next_publish_overwrites
+增补 3 第 4 件（增补 2 收口表第 40 行 / C381 的前提）：发布的持久顺序把系统配置槽轮换挪到根槽 FUA 写之前，最后一步失败时根就不会已经落盘	crates/singlefs-core/src/transaction.rs	        writer.perform(CommitStep::WriteRootRecordForceUnitAccess {\n            checkpoint_txg: txg,\n            root_slot: &root_slot,\n        })?;\n        writer.perform(CommitStep::RotateSystemConfigurationSlots {\n            journal_tail: plan.counter,\n            journal_instance: instance,\n        })	        writer.perform(CommitStep::RotateSystemConfigurationSlots {\n            journal_tail: plan.counter,\n            journal_instance: instance,\n        })?;\n        writer.perform(CommitStep::WriteRootRecordForceUnitAccess {\n            checkpoint_txg: txg,\n            root_slot: &root_slot,\n        })	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- a_publish_that_fails	a_publish_that_fails_on_the_system_configuration_slot_leaves_a_root_whose_units_the_next_publish_overwrites
diff --git a/crates/singlefs-harness/src/bin/first_transaction_on_device.rs b/crates/singlefs-harness/src/bin/first_transaction_on_device.rs
index f2b6299..85c8e9e 100644
--- a/crates/singlefs-harness/src/bin/first_transaction_on_device.rs
+++ b/crates/singlefs-harness/src/bin/first_transaction_on_device.rs
@@ -19,12 +19,11 @@
 
 use std::path::Path;
 
-use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
+use singlefs_core::address::DeviceIdentity;
 use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator};
 use singlefs_core::block_device::{
-    probe_queue_number_under, probe_queue_text_under, BlockDevice, BlockDeviceError,
-    DirectInputOutputBlockDevice, PageCachePolicy, PhysicalBlockSizeInBytes,
-    PhysicalBlockSizeSource, WriteDurability, SYSFS_CLASS_BLOCK,
+    probe_queue_number_under, probe_queue_text_under, BlockDevice, DirectInputOutputBlockDevice,
+    PageCachePolicy, PhysicalBlockSizeSource, SYSFS_CLASS_BLOCK,
 };
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
 use singlefs_core::mount::mount_writable;
@@ -33,6 +32,10 @@ use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolVersion, Pool
 use singlefs_core::write_accounting::{
     WriteCallsAndBytes, WritesByStructureKind, WrittenStructureKind,
 };
+use singlefs_harness::fault_injection::{
+    FaultCounting, FaultDeviceCounts, FaultDeviceSelector, FaultInjectingBlockDevice,
+    FaultOccurrence, FaultPlacement, FaultSchedule, InjectedFault, SharedFaultPlan,
+};
 use singlefs_harness::scenario::{
     e142_parameters, first_file_content, run_first_transaction, ScenarioPoint,
     FIXED_WRITE_TIME_SECONDS,
@@ -77,12 +80,13 @@ struct DeviceCallCounts {
 }
 
 impl DeviceCallCounts {
-    fn of<Inner: BlockDevice>(device: &FaultInjectingDevice<Inner>) -> Self {
+    /// 从共用的注入计划里取这块盘此刻的计数（`FaultDeviceCounts` 的六项里，结果行只报这四项）。
+    fn of(counts: FaultDeviceCounts) -> Self {
         Self {
-            write_calls: device.write_calls,
-            written_bytes: device.written_bytes,
-            force_unit_access_writes: device.force_unit_access_writes,
-            barrier_calls: device.barrier_calls,
+            write_calls: counts.writes,
+            written_bytes: counts.written_bytes,
+            force_unit_access_writes: counts.force_unit_access_writes,
+            barrier_calls: counts.barriers_forwarded,
         }
     }
 
@@ -168,74 +172,15 @@ fn publish_writes_against_device(
     )
 }
 
-/// 包在真设备外面、录制器里面：数程序真正交给设备的调用，按需吞掉一道屏障。
-struct FaultInjectingDevice<Inner: BlockDevice> {
-    inner: Inner,
-    skip_next_barrier: bool,
-    skipped_barriers: u64,
-    barrier_calls: u64,
-    write_calls: u64,
-    written_bytes: u64,
-    force_unit_access_writes: u64,
-    /// 这块盘收到第一道屏障那一刻的计数（屏障本身不算进去）：冷重开之后的可写挂载里第一道屏障是取号那一道，
-    /// 挂载那一段窗口从这里算起（`second-instance` 模式；可写挂载在取号与写行之间没有给调用方的口子）。
-    counts_when_first_barrier_arrived: Option<DeviceCallCounts>,
-}
-
-impl<Inner: BlockDevice> FaultInjectingDevice<Inner> {
-    /// 只计数、不吞屏障。
-    fn counting(inner: Inner) -> Self {
-        Self {
-            inner,
-            skip_next_barrier: false,
-            skipped_barriers: 0,
-            barrier_calls: 0,
-            write_calls: 0,
-            written_bytes: 0,
-            force_unit_access_writes: 0,
-            counts_when_first_barrier_arrived: None,
-        }
-    }
-}
-
-impl<Inner: BlockDevice> BlockDevice for FaultInjectingDevice<Inner> {
-    fn read_at(
-        &self,
-        offset: DeviceOffsetInBytes,
-        buffer: &mut [u8],
-    ) -> Result<(), BlockDeviceError> {
-        self.inner.read_at(offset, buffer)
-    }
-    fn write_at(
-        &mut self,
-        offset: DeviceOffsetInBytes,
-        bytes: &[u8],
-        durability: WriteDurability,
-    ) -> Result<(), BlockDeviceError> {
-        self.write_calls += 1;
-        self.written_bytes += u64::try_from(bytes.len()).expect("长度");
-        if durability == WriteDurability::ForceUnitAccess {
-            self.force_unit_access_writes += 1;
-        }
-        self.inner.write_at(offset, bytes, durability)
-    }
-    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
-        if self.counts_when_first_barrier_arrived.is_none() {
-            self.counts_when_first_barrier_arrived = Some(DeviceCallCounts::of(self));
-        }
-        if self.skip_next_barrier {
-            self.skip_next_barrier = false;
-            self.skipped_barriers += 1;
-            return Ok(());
-        }
-        self.barrier_calls += 1;
-        self.inner.barrier()
-    }
-    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
-        self.inner.probe_physical_block_size()
-    }
-    fn size_in_bytes(&self) -> u64 {
-        self.inner.size_in_bytes()
+/// 只吞掉这块盘接下来的第一道屏障（虚机档的「漏一道屏障」）：通用的故障注入包装（增补 3 第 4 件，
+/// `singlefs_harness::fault_injection`；这里原先手写的 `FaultInjectingDevice` 2026-09-21 并进了它）。
+fn swallow_the_next_barrier_on(device: DeviceIdentity) -> FaultSchedule {
+    FaultSchedule {
+        fault: InjectedFault::BarrierIsSwallowed,
+        device: FaultDeviceSelector::OnlyDevice(device),
+        placement: FaultPlacement::AnyOffset,
+        counting: FaultCounting::PerDevice,
+        occurrence: FaultOccurrence::TheNthMatchingCall(1),
     }
 }
 
@@ -305,16 +250,17 @@ fn open(path: &str, policy: PageCachePolicy) -> DirectInputOutputBlockDevice {
     })
 }
 
-/// 录制器里面那一层计数的盘。
-type CountedDevice<Inner> = RecordingBlockDevice<FaultInjectingDevice<Inner>>;
+/// 录制器外面那一层数调用、按需注入的盘。
+type CountedDevice<Inner> = FaultInjectingBlockDevice<RecordingBlockDevice<Inner>>;
 
-/// 每块盘此刻的计数快照。
+/// 每块盘此刻的计数快照，按 `devices` 的次序。
 fn device_call_counts<Inner: BlockDevice>(
     devices: &[(DeviceIdentity, CountedDevice<Inner>)],
+    plan: &SharedFaultPlan,
 ) -> Vec<DeviceCallCounts> {
     devices
         .iter()
-        .map(|(_, device)| DeviceCallCounts::of(device.inner()))
+        .map(|(identity, _)| DeviceCallCounts::of(plan.counts_of_device(*identity)))
         .collect()
 }
 
@@ -361,17 +307,19 @@ where
 {
     let closed_devices: Vec<(DeviceIdentity, Inner)> = devices_after_second_transaction
         .into_iter()
-        .map(|(identity, device)| (identity, device.into_inner_and_operations().0.inner))
+        .map(|(identity, device)| (identity, device.into_inner().into_inner_and_operations().0))
         .collect();
+    // 重开是新一轮：计数从零重数，所以这一段自己开一个注入计划（不接着上一段那个）。
+    let plan = SharedFaultPlan::unarmed(*geometry);
     let mut devices: Vec<(DeviceIdentity, CountedDevice<Inner>)> = reopen(closed_devices)
         .into_iter()
         .map(|(identity, inner)| {
             (
                 identity,
-                RecordingBlockDevice::with_shared_stream(
+                FaultInjectingBlockDevice::new(
                     identity,
-                    FaultInjectingDevice::counting(inner),
-                    stream.clone(),
+                    RecordingBlockDevice::with_shared_stream(identity, inner, stream.clone()),
+                    plan.clone(),
                 ),
             )
         })
@@ -383,13 +331,12 @@ where
     let mut mounted = mount_writable(parameters, &mut devices)
         .map_err(|failure| format!("可写挂载：{failure:?}"))?;
     let mount_nanoseconds = mount_started.elapsed().as_nanos();
-    let counts_after_mount = device_call_counts(&devices);
+    let counts_after_mount = device_call_counts(&devices, &plan);
     let counts_after_acquisition: Vec<DeviceCallCounts> = devices
         .iter()
-        .map(|(identity, device)| {
-            device
-                .inner()
-                .counts_when_first_barrier_arrived
+        .map(|(identity, _)| {
+            plan.counts_when_the_first_barrier_arrived_at(*identity)
+                .map(DeviceCallCounts::of)
                 .ok_or_else(|| {
                     format!(
                         "盘 {} 在可写挂载里一道屏障都没收到：取号那道屏障没发",
@@ -469,7 +416,7 @@ where
     }
     .map_err(|failure| format!("发布 C：{failure:?}"))?;
     let third_nanoseconds = third_started.elapsed().as_nanos();
-    let counts_after_third = device_call_counts(&devices);
+    let counts_after_third = device_call_counts(&devices, &plan);
     let third_operations = stream.operations();
     let third_segments =
         split_into_segments(&third_operations[operations_before_third..], geometry);
@@ -566,21 +513,31 @@ fn main() {
     .unwrap_or_else(|| "NA".to_string());
     let parameters = e142_parameters(physical_block_size, minimum_input_output_bytes);
     let stream = SharedStream::new();
-    let mut devices: Vec<(
-        DeviceIdentity,
-        RecordingBlockDevice<FaultInjectingDevice<DirectInputOutputBlockDevice>>,
-    )> = device_paths
-        .iter()
-        .enumerate()
-        .map(|(index, path)| {
-            let identity = DeviceIdentity(u32::try_from(index).expect("设备号"));
-            let device = FaultInjectingDevice::counting(open(path, policy));
-            (
-                identity,
-                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
-            )
-        })
-        .collect();
+    let geometry_for_faults = FixedGeometry {
+        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
+        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
+    };
+    let plan = SharedFaultPlan::unarmed(geometry_for_faults);
+    let mut devices: Vec<(DeviceIdentity, CountedDevice<DirectInputOutputBlockDevice>)> =
+        device_paths
+            .iter()
+            .enumerate()
+            .map(|(index, path)| {
+                let identity = DeviceIdentity(u32::try_from(index).expect("设备号"));
+                (
+                    identity,
+                    FaultInjectingBlockDevice::new(
+                        identity,
+                        RecordingBlockDevice::with_shared_stream(
+                            identity,
+                            open(path, policy),
+                            stream.clone(),
+                        ),
+                        plan.clone(),
+                    ),
+                )
+            })
+            .collect();
     let device_bytes = devices[0].1.size_in_bytes();
     emitter.emit(&format!(
         "name=geometry mode={} physical_block_size={physical_block_size} minimum_io={minimum_input_output_bytes} spacing={} write_cache={} device_bytes={device_bytes}",
@@ -597,14 +554,14 @@ fn main() {
     let run = run_first_transaction(&parameters, &mut devices, &stream, |point, devices| {
         let counts: Vec<DeviceCallCounts> = devices
             .iter()
-            .map(|(_, device)| DeviceCallCounts::of(device.inner()))
+            .map(|(identity, _)| DeviceCallCounts::of(plan.counts_of_device(*identity)))
             .collect();
         match point {
             ScenarioPoint::AfterInstanceAcquisition => counts_after_acquisition = Some(counts),
             ScenarioPoint::BeforeFirstTransaction => {
                 counts_before_first_transaction = Some(counts);
                 if mode == RunMode::SkipFirstTransactionBarrier {
-                    devices[0].1.inner_mut().skip_next_barrier = true;
+                    plan.arm(swallow_the_next_barrier_on(devices[0].0));
                 }
             }
         }
@@ -619,7 +576,7 @@ fn main() {
         .collect();
     let counts_after_first_transaction: Vec<DeviceCallCounts> = devices
         .iter()
-        .map(|(_, device)| DeviceCallCounts::of(device.inner()))
+        .map(|(identity, _)| DeviceCallCounts::of(plan.counts_of_device(*identity)))
         .collect();
     let counts_after_acquisition =
         counts_after_acquisition.expect("run_first_transaction 取号之后叫过一次");
@@ -655,17 +612,17 @@ fn main() {
             segment_kinds_text(&segments)
         ));
     }
-    for (index, (identity, device)) in devices.iter().enumerate() {
-        let counted = device.inner();
+    for (index, (identity, _)) in devices.iter().enumerate() {
+        let counted = plan.counts_of_device(*identity);
         emitter.emit(&format!(
             "name=device_calls device={} path={} writes={} written_bytes={} force_unit_access_writes={} barriers={} skipped_barriers={} {}",
             identity.0,
             device_paths[index],
-            counted.write_calls,
+            counted.writes,
             counted.written_bytes,
             counted.force_unit_access_writes,
-            counted.barrier_calls,
-            counted.skipped_barriers,
+            counted.barriers_forwarded,
+            counted.barriers_swallowed,
             describe_counters(counters_before[index], counters_after[index])
         ));
     }
@@ -711,7 +668,7 @@ fn main() {
     let expected_content = if publishes_second_version {
         let before: Vec<DeviceCallCounts> = devices
             .iter()
-            .map(|(_, device)| DeviceCallCounts::of(device.inner()))
+            .map(|(identity, _)| DeviceCallCounts::of(plan.counts_of_device(*identity)))
             .collect();
         let operations_before = stream.operations().len();
         let device_maps: Vec<DeviceFreeMap> = devices
@@ -744,7 +701,7 @@ fn main() {
             split_into_segments(&after_operations[operations_before..], &geometry);
         let after: Vec<DeviceCallCounts> = devices
             .iter()
-            .map(|(_, device)| DeviceCallCounts::of(device.inner()))
+            .map(|(identity, _)| DeviceCallCounts::of(plan.counts_of_device(*identity)))
             .collect();
         let per_device: Vec<String> = after
             .iter()
@@ -865,7 +822,7 @@ mod tests {
     use super::{
         describe_publish_writes, pool_writes_between, publish_writes_against_device,
         second_file_content, switch_instance_and_publish_third_version, third_file_content,
-        CountedDevice, DeviceCallCounts, FaultInjectingDevice,
+        CountedDevice, DeviceCallCounts,
     };
     use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
     use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator};
@@ -876,6 +833,7 @@ mod tests {
         WriteCallsAndBytes, WritesByStructureKind, WrittenStructureKind,
     };
     use singlefs_harness::crash::SparseBlockDevice;
+    use singlefs_harness::fault_injection::{FaultInjectingBlockDevice, SharedFaultPlan};
     use singlefs_harness::scenario::{
         e142_parameters, run_first_transaction, FIXED_WRITE_TIME_SECONDS,
     };
@@ -899,18 +857,27 @@ mod tests {
     ) {
         let parameters = e142_parameters(512, 512);
         let stream = SharedStream::new();
+        let geometry_for_faults = FixedGeometry {
+            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
+            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
+        };
+        let plan = SharedFaultPlan::unarmed(geometry_for_faults);
         let mut devices: Vec<(DeviceIdentity, CountedDevice<SparseBlockDevice>)> = (0..2u32)
             .map(|device_number| {
                 let identity = DeviceIdentity(device_number);
                 (
                     identity,
-                    RecordingBlockDevice::with_shared_stream(
+                    FaultInjectingBlockDevice::new(
                         identity,
-                        FaultInjectingDevice::counting(SparseBlockDevice::new(
-                            SPARSE_DEVICE_BYTES,
-                            PhysicalBlockSizeInBytes(512),
-                        )),
-                        stream.clone(),
+                        RecordingBlockDevice::with_shared_stream(
+                            identity,
+                            SparseBlockDevice::new(
+                                SPARSE_DEVICE_BYTES,
+                                PhysicalBlockSizeInBytes(512),
+                            ),
+                            stream.clone(),
+                        ),
+                        plan.clone(),
                     ),
                 )
             })
@@ -1036,7 +1003,7 @@ mod tests {
         let cold: Vec<(DeviceIdentity, SparseBlockDevice)> = switched
             .devices
             .into_iter()
-            .map(|(identity, device)| (identity, device.into_inner_and_operations().0.inner))
+            .map(|(identity, device)| (identity, device.into_inner().into_inner_and_operations().0))
             .collect();
         let report = recover(&cold, JournalPolicy::Consult);
         match report.outcome {
diff --git a/crates/singlefs-harness/src/history.rs b/crates/singlefs-harness/src/history.rs
index 04cc164..01155f8 100644
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -8,8 +8,10 @@
 //! 删掉前面几步之后，后面的操作照样有意义，收缩靠的就是这一条。
 //!
 //! 第 3 至 5 件在这里生成的历史上做：录制流由调用方给（`execute_history_observing`；崩溃注入给开了内容保留的流，截断点从流里取），
-//! 每一步之后的镜像经观察者交出去（坏盘输入拿它当合法镜像）；故障注入要在录制器与内存盘之间再包一层，届时把 `HistoryDevice`
-//! 换成泛型——今天只有一种盘，按编码纪律「只有一个实现的 trait 不抽」不先抽。
+//! 每一步之后的镜像经观察者交出去（坏盘输入拿它当合法镜像）；故障注入（第 4 件）在**录制器外面**包了一层
+//! `crate::fault_injection::FaultInjectingBlockDevice`，注入计划由调用方给（`execute_history_with_faults`），
+//! 不注入时它只数调用、每一次都原样交给录制器——报错的写、被吞掉的写与被吞掉的屏障都不进录制流，
+//! 录制流因此恒等于真正落到盘上的那一串。
 
 use std::cell::{Cell, RefCell};
 use std::collections::{BTreeMap, BTreeSet};
@@ -48,6 +50,7 @@ use singlefs_core::unit::data_unit_payload_capacity;
 use singlefs_format::{DATA_UNIT_BYTES, SLOT_BYTES, UNIT_AREA_START_SLOT};
 
 use crate::crash::{MemoryPool, RecordCheck, SparseBlockDevice};
+use crate::fault_injection::{FaultInjectingBlockDevice, SharedFaultPlan};
 use crate::model::{
     IdealModel, ModelAnswer, ModelCheckpointTxg, ModelDeviceIdentity, ModelDisagreement,
     ModelJudgementCounts, ModelPoolGeometry, ModelRefusalReason, ModelRootKey, ObservedEffect,
@@ -60,6 +63,7 @@ use crate::model_comparison::{
     reported_ceiling_of_mount_error,
 };
 use crate::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
+use crate::segments::FixedGeometry;
 use crate::{RecordingBlockDevice, SharedStream};
 
 /// 历史里两块内存盘多宽（取样点的参数；两块恒等大：模型的几何只有一个盘大小，`ModelPoolGeometry::device_size_in_bytes`）。
@@ -105,6 +109,16 @@ impl HistoryDeviceWidth {
         parameters
     }
 
+    /// 按落点分写的种类要的那份几何（`crate::segments::FixedGeometry`）：故障注入拿它报「注入落在哪一类结构上」。
+    #[must_use]
+    pub fn fixed_geometry(self) -> FixedGeometry {
+        let parameters = self.parameters();
+        FixedGeometry {
+            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
+            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
+        }
+    }
+
     /// 报告里的名字。
     #[must_use]
     pub fn name(self) -> &'static str {
@@ -117,8 +131,10 @@ impl HistoryDeviceWidth {
     }
 }
 
-/// 历史里的一块盘：内存盘外面包录制器，写与屏障进调用方给的那条流。
-pub type HistoryDevice = RecordingBlockDevice<SparseBlockDevice>;
+/// 历史里的一块盘：内存盘外面包录制器（写与屏障进调用方给的那条流），录制器外面再包故障注入
+/// （`crate::fault_injection::FaultInjectingBlockDevice`；不注入时只数调用）。
+/// 注入层在最外面：报错的写、被吞掉的写与被吞掉的屏障都不进录制流。
+pub type HistoryDevice = FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>;
 
 /// 一段历史的种子。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
@@ -742,7 +758,7 @@ fn image_of(
     MemoryPool {
         devices: devices
             .iter()
-            .map(|(identity, device)| (*identity, device.inner().image.clone()))
+            .map(|(identity, device)| (*identity, device.inner().inner().image.clone()))
             .collect(),
         device_size_in_bytes: device_width.device_bytes(),
     }
@@ -791,6 +807,7 @@ impl HistoryPool {
         starting_point: HistoryStartingPoint,
         device_width: HistoryDeviceWidth,
         stream: &SharedStream,
+        fault_plan: &SharedFaultPlan,
     ) -> (Self, Option<ModelVerdict>) {
         let parameters = device_width.parameters();
         let device_bytes = device_width.device_bytes();
@@ -800,10 +817,14 @@ impl HistoryPool {
                 .map(|identity| {
                     (
                         identity,
-                        RecordingBlockDevice::with_shared_stream(
+                        FaultInjectingBlockDevice::new(
                             identity,
-                            SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
-                            stream.clone(),
+                            RecordingBlockDevice::with_shared_stream(
+                                identity,
+                                SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
+                                stream.clone(),
+                            ),
+                            fault_plan.clone(),
                         ),
                     )
                 })
@@ -1794,6 +1815,11 @@ pub struct HistoryRun {
     /// mkfs 不是事务，它写到一半的盘面上没有池，恢复报不出根是对的，不该拿事务的 oracle 去判
     /// （层 0 那一路同样从 `mkfs_operation_count` 之后起枚举，见 `tests/first_transaction_step_seven_layer0.rs`）。
     pub operations_written_by_make_filesystem: usize,
+    /// 这段历史停下时模型根环里的每一条根（`IdealModel::committed_versions`）：故障注入（增补 3 第 4 件）拿它当
+    /// 「模型认下来的每一版」判重开走到的那一版——失败那一步的根，模型认了就在里面，没认（判定对不上、模型没往前走）就不在。
+    /// 观察者那一路看不到这一份：判出失败的那一步不调观察者，而失败常常出在模型已经认下那一版之后（checker 在那一步判红）。
+    /// 内容拷成 `Vec<u8>` 不留 `Rc`：`HistoryRun` 要跨线程送（`run_history_campaign` 的那张表），`Rc` 不是 `Send`。
+    pub committed_versions_at_the_end: Vec<(ModelRootKey, Option<Vec<u8>>)>,
 }
 
 /// 每一步之后交给观察者的东西：第几步、这一步是什么、结局、此刻的整份镜像、跟到这一步的理想模型。
@@ -2792,6 +2818,24 @@ pub fn execute_history_with(
     execution: HistoryExecution,
     stream: &SharedStream,
     observer: &mut dyn FnMut(&StepObservation<'_>),
+) -> HistoryRun {
+    execute_history_with_faults(
+        history,
+        execution,
+        stream,
+        &SharedFaultPlan::unarmed(execution.device_width.fixed_geometry()),
+        observer,
+    )
+}
+
+/// 同 [`execute_history_with`]，再按调用方给的注入计划在读 / 写 / 刷盘上注入故障（增补 3 第 4 件）。
+/// 计划不开时与 [`execute_history_with`] 逐字节相同：注入层只数调用，每一次都原样交给录制器。
+pub fn execute_history_with_faults(
+    history: &GeneratedHistory,
+    execution: HistoryExecution,
+    stream: &SharedStream,
+    fault_plan: &SharedFaultPlan,
+    observer: &mut dyn FnMut(&StepObservation<'_>),
 ) -> HistoryRun {
     let per_step_checker = execution.per_step_checker;
     let mut tally = HistoryTally::default();
@@ -2800,8 +2844,12 @@ pub fn execute_history_with(
     let position = Cell::new(StepPosition::StartingPoint);
     let mut completed_after_the_root_ring_turned = false;
     let body_result = with_panic_capture(|| -> Option<FailureObservation> {
-        let (started_pool, starting_verdict) =
-            HistoryPool::start(history.starting_point, execution.device_width, stream);
+        let (started_pool, starting_verdict) = HistoryPool::start(
+            history.starting_point,
+            execution.device_width,
+            stream,
+            fault_plan,
+        );
         let pool = pool_slot.insert(started_pool);
         let mut image = pool.image();
         let mut checked_stream_length = stream.operation_count();
@@ -2989,6 +3037,13 @@ pub fn execute_history_with(
         operations_written_by_make_filesystem: pool_slot
             .as_ref()
             .map_or(0, |pool| pool.operations_written_by_make_filesystem),
+        committed_versions_at_the_end: pool_slot.as_ref().map_or_else(Vec::new, |pool| {
+            pool.model
+                .committed_versions()
+                .into_iter()
+                .map(|(key, content)| (key, content.map(|content| content.to_vec())))
+                .collect()
+        }),
     }
 }
 
diff --git a/crates/singlefs-harness/src/lib.rs b/crates/singlefs-harness/src/lib.rs
index 2f892f4..103bad5 100644
--- a/crates/singlefs-harness/src/lib.rs
+++ b/crates/singlefs-harness/src/lib.rs
@@ -16,6 +16,7 @@ use singlefs_core::block_device::{
 pub mod crash;
 pub mod crash_injection;
 pub mod device_log;
+pub mod fault_injection;
 pub mod first_transaction_regions;
 pub mod hexadecimal;
 pub mod history;
diff --git a/crates/singlefs-harness/tests/instance_acquisition.rs b/crates/singlefs-harness/tests/instance_acquisition.rs
index 1582498..8eee525 100644
--- a/crates/singlefs-harness/tests/instance_acquisition.rs
+++ b/crates/singlefs-harness/tests/instance_acquisition.rs
@@ -3,68 +3,47 @@
 
 mod common;
 
-use std::io;
-
-use common::{build_pool, parameters, BuiltPool, Recorded};
-use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
-use singlefs_core::block_device::{
-    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
-};
+use common::{build_pool, geometry, parameters, BuiltPool, Recorded};
+use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
 use singlefs_core::recovery::{choose_system_configuration, verified_system_configuration_slots};
 use singlefs_core::transaction::{acquire_instance, AcquisitionRollback, CommitStep, PoolWriter};
+use singlefs_harness::fault_injection::{
+    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
+    FaultSchedule, InjectedFault, SharedFaultPlan,
+};
 
 /// 系统配置两槽住在偏移 0 与 4096，都在这个界之下。
 const SYSTEM_CONFIGURATION_SLOTS_END_OFFSET: u64 = 8192;
 
-/// 包在录制盘外面：按开关让屏障或系统配置槽写报错，并数真正交给设备的屏障。
-struct FaultInjectingDevice {
-    inner: Recorded,
-    fail_barriers: bool,
-    fail_system_configuration_writes: bool,
-    barrier_calls: u64,
+/// 包在录制盘外面的通用故障注入（增补 3 第 4 件，`singlefs_harness::fault_injection`；这里原先手写的 `FaultInjectingDevice`
+/// 2026-09-21 并进了它）：按开关让屏障或系统配置槽写报错，并数真正交给设备的屏障（`FaultDeviceCounts::barriers_forwarded`）。
+type FaultInjectingDevice = FaultInjectingBlockDevice<Recorded>;
+
+/// 两块盘共用的注入计划连同它们。
+struct WrappedDevices {
+    plan: SharedFaultPlan,
+    devices: Vec<(DeviceIdentity, FaultInjectingDevice)>,
 }
 
-fn injected(what: &str) -> BlockDeviceError {
-    BlockDeviceError::InputOutput(io::Error::other(format!("注入的{what}错")))
+/// 每一道屏障都报错。
+fn fail_every_barrier() -> FaultSchedule {
+    FaultSchedule::every_call_across_the_pool(InjectedFault::BarrierFails)
 }
 
-impl BlockDevice for FaultInjectingDevice {
-    fn read_at(
-        &self,
-        offset: DeviceOffsetInBytes,
-        buffer: &mut [u8],
-    ) -> Result<(), BlockDeviceError> {
-        self.inner.read_at(offset, buffer)
-    }
-    fn write_at(
-        &mut self,
-        offset: DeviceOffsetInBytes,
-        bytes: &[u8],
-        durability: WriteDurability,
-    ) -> Result<(), BlockDeviceError> {
-        if self.fail_system_configuration_writes && offset.0 < SYSTEM_CONFIGURATION_SLOTS_END_OFFSET
-        {
-            return Err(injected("系统配置槽写"));
-        }
-        self.inner.write_at(offset, bytes, durability)
-    }
-    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
-        if self.fail_barriers {
-            return Err(injected("屏障"));
-        }
-        self.barrier_calls += 1;
-        self.inner.barrier()
-    }
-    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
-        self.inner.probe_physical_block_size()
-    }
-    fn size_in_bytes(&self) -> u64 {
-        self.inner.size_in_bytes()
+/// 这块盘上每一次系统配置槽写都报错（两槽住在偏移 0 起的 `SYSTEM_CONFIGURATION_SLOTS_END_OFFSET` 之内）。
+fn fail_every_system_configuration_write_on(device: DeviceIdentity) -> FaultSchedule {
+    FaultSchedule {
+        fault: InjectedFault::WriteFails,
+        device: FaultDeviceSelector::OnlyDevice(device),
+        placement: FaultPlacement::OffsetBelow(SYSTEM_CONFIGURATION_SLOTS_END_OFFSET),
+        counting: FaultCounting::AcrossThePool,
+        occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
     }
 }
 
-fn wrap(built: &mut BuiltPool) -> Vec<(DeviceIdentity, FaultInjectingDevice)> {
-    built
+fn wrap(built: &mut BuiltPool) -> WrappedDevices {
+    let plan = SharedFaultPlan::unarmed(geometry());
+    let devices = built
         .devices
         .take()
         .expect("第一个事务写完，盘还开着")
@@ -72,15 +51,11 @@ fn wrap(built: &mut BuiltPool) -> Vec<(DeviceIdentity, FaultInjectingDevice)> {
         .map(|(identity, inner)| {
             (
                 identity,
-                FaultInjectingDevice {
-                    inner,
-                    fail_barriers: false,
-                    fail_system_configuration_writes: false,
-                    barrier_calls: 0,
-                },
+                FaultInjectingBlockDevice::new(identity, inner, plan.clone()),
             )
         })
-        .collect()
+        .collect();
+    WrappedDevices { plan, devices }
 }
 
 /// 一块盘两槽里自证过的系统配置：(世代号, 实例代号)，按世代号排好。
@@ -121,7 +96,10 @@ const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
 #[test]
 fn second_acquisition_writes_generation_six_on_both_disks_and_the_next_acquisition_gets_three() {
     let mut built = build_pool("acquire-second");
-    let mut devices = wrap(&mut built);
+    let WrappedDevices {
+        plan: _,
+        mut devices,
+    } = wrap(&mut built);
     assert_eq!(
         acquire(&mut devices).expect("第二次取号"),
         InstanceGeneration(2)
@@ -151,10 +129,8 @@ fn second_acquisition_writes_generation_six_on_both_disks_and_the_next_acquisiti
 fn failed_barrier_after_acquisition_rolls_both_disks_back_and_the_skipped_number_is_never_handed_out_again(
 ) {
     let mut built = build_pool("acquire-barrier-error");
-    let mut devices = wrap(&mut built);
-    for (_, device) in &mut devices {
-        device.fail_barriers = true;
-    }
+    let WrappedDevices { plan, mut devices } = wrap(&mut built);
+    plan.arm(fail_every_barrier());
     let failure =
         acquire(&mut devices).expect_err("取号之后那道屏障报错，取号必须失败、不交出新号");
     assert!(
@@ -168,9 +144,7 @@ fn failed_barrier_after_acquisition_rolls_both_disks_back_and_the_skipped_number
             "{disk:?}：取号写世代 6 带新号 2，回卷写世代 7 带取号之前全部自证过的槽中最大的号 1"
         );
     }
-    for (_, device) in &mut devices {
-        device.fail_barriers = false;
-    }
+    plan.disarm();
     assert_eq!(
         acquire(&mut devices).expect("屏障好了再取号"),
         InstanceGeneration(3),
@@ -182,13 +156,8 @@ fn failed_barrier_after_acquisition_rolls_both_disks_back_and_the_skipped_number
 fn failed_system_configuration_write_on_the_second_disk_rolls_the_first_disk_back_and_leaves_the_second_untouched(
 ) {
     let mut built = build_pool("acquire-write-error");
-    let mut devices = wrap(&mut built);
-    devices
-        .iter_mut()
-        .find(|(identity, _)| *identity == DeviceIdentity(1))
-        .expect("盘 1")
-        .1
-        .fail_system_configuration_writes = true;
+    let WrappedDevices { plan, mut devices } = wrap(&mut built);
+    plan.arm(fail_every_system_configuration_write_on(DeviceIdentity(1)));
     let failure = acquire(&mut devices).expect_err("盘 1 的取号写报错，取号必须失败");
     assert!(
         matches!(failure.rollback, AcquisitionRollback::RolledBack),
@@ -208,16 +177,17 @@ fn failed_system_configuration_write_on_the_second_disk_rolls_the_first_disk_bac
 #[test]
 fn barrier_right_after_the_acquisition_barrier_is_not_sent_to_the_devices() {
     let mut built = build_pool("acquire-barrier-skip");
-    let mut devices = wrap(&mut built);
+    let WrappedDevices { plan, mut devices } = wrap(&mut built);
     {
         let parameters = parameters();
         let mut pool = PoolWriter::new(&parameters, &mut devices);
         acquire_instance(&mut pool).expect("取号");
         pool.perform(CommitStep::Barrier).expect("紧跟着的一道屏障");
     }
-    for (identity, device) in &devices {
+    for (identity, _) in &devices {
         assert_eq!(
-            device.barrier_calls, 1,
+            plan.counts_of_device(*identity).barriers_forwarded,
+            1,
             "{identity:?}：取号之后那道屏障发一次；紧跟着的那道前面没有写，不再发——首次挂载路径上暖机开场那道就这样并掉，设备收到的 FLUSH 数不变"
         );
     }
diff --git a/crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs b/crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs
index 9abc30d..e77398b 100644
--- a/crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs
+++ b/crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs
@@ -16,9 +16,8 @@ use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
 };
 use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
-use singlefs_core::block_device::{
-    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
-};
+use singlefs_core::block_device::BlockDevice;
+
 use singlefs_core::journal::back_chain_of;
 use singlefs_core::make_filesystem::{INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
 use singlefs_core::mount::{mount_writable, MountError, RollbackTarget};
@@ -29,50 +28,25 @@ use singlefs_core::transaction::{
     PoolWriter, PublishError, ZeroUnitPublishPlan,
 };
 use singlefs_harness::crash::writes_and_segments;
+use singlefs_harness::fault_injection::{
+    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
+    FaultSchedule, InjectedFault, SharedFaultPlan,
+};
 use singlefs_harness::SharedStream;
-use std::cell::Cell;
 
-/// 读系统配置槽 0（偏移 0）时按「这块盘第几次读槽 0」注入一次瞬时读错的盘，别的读写原样交给内层（m2-emptypool-nonempty-r1 云端攻方腿
-/// 模型 `opus_attack_emptypool.rs` 第 294 行起的做法）。
-struct TransientSystemConfigurationReadErrorDevice<Inner: BlockDevice> {
-    inner: Inner,
-    slot_zero_reads: Cell<u32>,
-    failing_slot_zero_read_ordinal: u32,
-}
-
-impl<Inner: BlockDevice> BlockDevice for TransientSystemConfigurationReadErrorDevice<Inner> {
-    fn read_at(
-        &self,
-        offset: DeviceOffsetInBytes,
-        buffer: &mut [u8],
-    ) -> Result<(), BlockDeviceError> {
-        if offset.0 == 0 {
-            let ordinal = self.slot_zero_reads.get() + 1;
-            self.slot_zero_reads.set(ordinal);
-            if ordinal == self.failing_slot_zero_read_ordinal {
-                return Err(BlockDeviceError::InputOutput(std::io::Error::other(
-                    "注入的瞬时读错",
-                )));
-            }
-        }
-        self.inner.read_at(offset, buffer)
-    }
-    fn write_at(
-        &mut self,
-        offset: DeviceOffsetInBytes,
-        bytes: &[u8],
-        durability: WriteDurability,
-    ) -> Result<(), BlockDeviceError> {
-        self.inner.write_at(offset, bytes, durability)
-    }
-    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
-        self.inner.barrier()
-    }
-    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
-        self.inner.probe_physical_block_size()
-    }
-    fn size_in_bytes(&self) -> u64 {
-        self.inner.size_in_bytes()
+/// 每块盘第 `ordinal` 次读系统配置槽 0（偏移 0）时注入一次读错，别的读写原样交给内层
+/// （m2-emptypool-nonempty-r1 云端攻方腿模型 `opus_attack_emptypool.rs` 第 294 行起的做法）。
+/// 用的是通用的故障注入包装（增补 3 第 4 件，`singlefs_harness::fault_injection`；这里原先手写的
+/// `TransientSystemConfigurationReadErrorDevice` 2026-09-21 并进了它）：逐盘数，所以两块盘各在自己第 `ordinal` 次上报错一次。
+fn fail_the_nth_read_of_system_configuration_slot_zero_on_each_device(
+    ordinal: u64,
+) -> FaultSchedule {
+    FaultSchedule {
+        fault: InjectedFault::ReadFails,
+        device: FaultDeviceSelector::EveryDevice,
+        placement: FaultPlacement::OffsetExactly(DeviceOffsetInBytes(0)),
+        counting: FaultCounting::PerDevice,
+        occurrence: FaultOccurrence::TheNthMatchingCall(ordinal),
     }
 }
 
@@ -94,20 +68,17 @@ fn transient_system_configuration_read_errors_between_the_refusal_and_the_acquis
         );
     }
     let before = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
-    let mut devices: Vec<(
-        DeviceIdentity,
-        TransientSystemConfigurationReadErrorDevice<_>,
-    )> = formatted
+    let plan = SharedFaultPlan::armed(
+        geometry(),
+        fail_the_nth_read_of_system_configuration_slot_zero_on_each_device(2),
+    );
+    let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<_>)> = formatted
         .reopen_recorded()
         .into_iter()
         .map(|(identity, recorded)| {
             (
                 identity,
-                TransientSystemConfigurationReadErrorDevice {
-                    inner: recorded,
-                    slot_zero_reads: Cell::new(0),
-                    failing_slot_zero_read_ordinal: 2,
-                },
+                FaultInjectingBlockDevice::new(identity, recorded, plan.clone()),
             )
         })
         .collect();
@@ -115,7 +86,7 @@ fn transient_system_configuration_read_errors_between_the_refusal_and_the_acquis
     formatted.devices = Some(
         devices
             .into_iter()
-            .map(|(identity, device)| (identity, device.inner))
+            .map(|(identity, device)| (identity, device.into_inner()))
             .collect(),
     );
     assert!(
diff --git a/crates/singlefs-harness/tests/second_transaction_supplement_one_write_accounting.rs b/crates/singlefs-harness/tests/second_transaction_supplement_one_write_accounting.rs
index 720c346..f2fee08 100644
--- a/crates/singlefs-harness/tests/second_transaction_supplement_one_write_accounting.rs
+++ b/crates/singlefs-harness/tests/second_transaction_supplement_one_write_accounting.rs
@@ -4,14 +4,9 @@
 //! 单元两盘各一份、journal 记录 4096 两盘各一份、根槽 512 一次 FUA、系统配置槽 4096 两盘各一次），种类归错了合计不变、这一条红。
 //! 两块内存盘（与宿主重跑虚机那条路同一种设备），不落文件。
 
-use std::cell::Cell;
-use std::rc::Rc;
-
-use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
+use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
 use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
-use singlefs_core::block_device::{
-    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
-};
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
 use singlefs_core::make_filesystem::{
     make_filesystem, MakeFilesystemOutput, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
 };
@@ -24,62 +19,37 @@ use singlefs_core::write_accounting::{
     WriteCallsAndBytes, WritesByStructureKind, WrittenStructureKind,
 };
 use singlefs_harness::crash::SparseBlockDevice;
+use singlefs_harness::fault_injection::{
+    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
+    FaultSchedule, InjectedFault, SharedFaultPlan,
+};
 use singlefs_harness::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
+use singlefs_harness::segments::FixedGeometry;
 use singlefs_harness::{
     RecordedOperation, RecordedOperationKind, RecordingBlockDevice, SharedStream,
 };
 
 const IMAGE_BYTES: u64 = 4 << 30;
 
-/// 还许成功几次写：`None` = 不注入（一直放行），`Some(n)` = 再放行 n 次写，之后每次写都报错。
-/// 装的是 `Rc<Cell<..>>`，测试拿着同一个句柄，写入口正拿着设备时也开得了、关得掉。
-type RemainingSuccessfulWrites = Rc<Cell<Option<u64>>>;
+/// 内存盘外面包录制器，录制器外面包通用的故障注入（增补 3 第 4 件，`singlefs_harness::fault_injection`；
+/// 这里原先手写的 `WriteFailingDevice` 2026-09-21 并进了它）。注入层在录制器**外面**：报错的那次写既不进录制流、
+/// 也不进按种类的账（录制器只记它报成功的写），两边口径仍相同。
+type MemoryDevice = FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>;
 
-/// 包在内存盘外面、录制器里面：按开关让写报错。放在录制器**里面**，失败的那次写就既不进录制流、也不进按种类的账
-/// （录制器只记内层报成功的写，`RecordingBlockDevice::write_at`），两边口径仍相同。
-struct WriteFailingDevice<Inner: BlockDevice> {
-    inner: Inner,
-    remaining_successful_writes: RemainingSuccessfulWrites,
-}
-
-impl<Inner: BlockDevice> BlockDevice for WriteFailingDevice<Inner> {
-    fn read_at(
-        &self,
-        offset: DeviceOffsetInBytes,
-        buffer: &mut [u8],
-    ) -> Result<(), BlockDeviceError> {
-        self.inner.read_at(offset, buffer)
-    }
-    fn write_at(
-        &mut self,
-        offset: DeviceOffsetInBytes,
-        bytes: &[u8],
-        durability: WriteDurability,
-    ) -> Result<(), BlockDeviceError> {
-        match self.remaining_successful_writes.get() {
-            None => {}
-            Some(0) => {
-                return Err(BlockDeviceError::InputOutput(std::io::Error::other(
-                    "注入的设备写错",
-                )));
-            }
-            Some(remaining) => self.remaining_successful_writes.set(Some(remaining - 1)),
-        }
-        self.inner.write_at(offset, bytes, durability)
-    }
-    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
-        self.inner.barrier()
-    }
-    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
-        self.inner.probe_physical_block_size()
-    }
-    fn size_in_bytes(&self) -> u64 {
-        self.inner.size_in_bytes()
+/// 让盘 `device` 再放行 `successful_writes` 次写、之后每次写都报错（原先那个 `Rc<Cell<Option<u64>>>` 开关的同义写法）。
+fn fail_every_write_on_one_device_after(
+    device: DeviceIdentity,
+    successful_writes: u64,
+) -> FaultSchedule {
+    FaultSchedule {
+        fault: InjectedFault::WriteFails,
+        device: FaultDeviceSelector::OnlyDevice(device),
+        placement: FaultPlacement::AnyOffset,
+        counting: FaultCounting::PerDevice,
+        occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(successful_writes + 1),
     }
 }
 
-type MemoryDevice = RecordingBlockDevice<WriteFailingDevice<SparseBlockDevice>>;
-
 fn calls_and_bytes(write_calls: u64, written_bytes: u64) -> WriteCallsAndBytes {
     WriteCallsAndBytes {
         write_calls,
@@ -229,8 +199,8 @@ fn second_file_content() -> Vec<u8> {
 /// mkfs → 取号 → 暖机 → 第一个事务 → 发布 B，同一个写入口；记下每一步开始时录制流有几条。
 struct PublishedThroughOverwrite {
     devices: Vec<(DeviceIdentity, MemoryDevice)>,
-    /// 与 `devices` 同序：每块盘的注入开关。
-    write_faults: Vec<RemainingSuccessfulWrites>,
+    /// 几块盘共用的故障注入计划：开关一条计划就注入，收起来就一直放行。
+    fault_plan: SharedFaultPlan,
     stream: SharedStream,
     allocator: PoolAllocator,
     warm_up: WarmUpOutput,
@@ -245,22 +215,23 @@ struct PublishedThroughOverwrite {
 fn publish_through_overwrite() -> PublishedThroughOverwrite {
     let parameters = e142_parameters(512, 512);
     let stream = SharedStream::new();
-    let write_faults: Vec<RemainingSuccessfulWrites> =
-        (0..2).map(|_| Rc::new(Cell::new(None))).collect();
+    let fault_plan = SharedFaultPlan::unarmed(FixedGeometry {
+        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
+        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
+    });
     let mut devices: Vec<(DeviceIdentity, MemoryDevice)> = (0..2u32)
         .map(|device_number| {
             let identity = DeviceIdentity(device_number);
             (
                 identity,
-                RecordingBlockDevice::with_shared_stream(
+                FaultInjectingBlockDevice::new(
                     identity,
-                    WriteFailingDevice {
-                        inner: SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
-                        remaining_successful_writes: write_faults
-                            [usize::try_from(device_number).expect("设备号")]
-                        .clone(),
-                    },
-                    stream.clone(),
+                    RecordingBlockDevice::with_shared_stream(
+                        identity,
+                        SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
+                        stream.clone(),
+                    ),
+                    fault_plan.clone(),
                 ),
             )
         })
@@ -314,7 +285,7 @@ fn publish_through_overwrite() -> PublishedThroughOverwrite {
     drop(writer);
     PublishedThroughOverwrite {
         devices,
-        write_faults,
+        fault_plan,
         stream,
         allocator,
         warm_up: warmed,
@@ -526,7 +497,9 @@ fn publish_that_fails_midway_hands_out_what_it_already_wrote_so_the_retry_still_
     let window_start = published.stream.operations().len();
     let content = third_file_content();
     // 盘 1 再放行 5 次写：发布 C 的第六个单元写到盘 1 时报错（单元按 bump 次序写，每个单元两盘各一次）。
-    published.write_faults[1].set(Some(5));
+    published
+        .fault_plan
+        .arm(fail_every_write_on_one_device_after(DeviceIdentity(1), 5));
     let (failed_publishes, retry) = {
         let mut writer = PoolWriter::new(&parameters, published.devices.as_mut_slice());
         let failure = publish_overwrite(
@@ -547,7 +520,7 @@ fn publish_that_fails_midway_hands_out_what_it_already_wrote_so_the_retry_still_
             ),
             "中途失败的是设备写：{failure:?}"
         );
-        published.write_faults[1].set(None);
+        published.fault_plan.disarm();
         let retry = publish_overwrite(
             &mut writer,
             &mut published.allocator,
```

## 二、新文件 `crates/singlefs-harness/src/fault_injection.rs` 全文（2053 行，工作区未跟踪）

```rust
//! 故障注入（里程碑「第二个事务」增补 3 第 4 件）：一个通用的设备包装，按种子让任意一次写、读、刷盘返回 `BlockDeviceError`，
//! 或让一次读返回改坏的字节。被测的性质有两条：注入之后 `singlefs-core` 那一侧**返回错误而不是 panic**，
//! 出错之后**重开要恢复到模型允许的版本**。
//!
//! 包装的位置在录制器**外面**（调用方与录制器之间）：报错的写、被吞掉的写与被吞掉的屏障都不进录制流，
//! 录制流因此恒等于真正落到盘上的那一串——注入之后的镜像可以从录制流重建（`crate::crash::MemoryPool::apply`），
//! 不必把执行器里的两块盘再交出来一份。
//!
//! 一段历史上怎么注入：先跑一遍不注入的（测量跑），记下每一步跑完时读 / 写 / 刷盘各调了多少次、模型到那一步提交过哪些版本；
//! 再按种子在起点之后的调用里挑几个注入点，每个注入点重跑一遍同一段历史（同一个种子逐位复现同一串数，重跑到注入那一刻为止逐字节相同）。
//! 重跑之后判三样：
//! 1. 整段历史里一个 panic 都没有（`singlefs-core` 的断言被盘上内容或设备错走到了，就是缺口，增补 3 第 6 件那一类）；
//! 2. 注入那一步的结局是「入口返回 Err」或「冷启动读回报错」这两种形态之一，不是别的；
//! 3. 重开（拿录制流重建镜像、跑 `recovery::recover`）走到的那一版在模型允许的集合里，读回的内容与模型记的逐字节相同
//!    （`crate::model::crash_recovery_disagreement`，与崩溃注入同一条判据、同一份实现）。
//!
//! 允许的集合 = 注入那一步之前模型提交过的每一版 ∪ 不注入时那一步会提交的那几版。后一半是刻意放行的：
//! 「发布在最后一步失败、根已 FUA 落盘，重开之后那一版看得见」正是增补 2 收口表第 40 行（C381）在争的那一格，
//! 三方三轮判完、改法未定，这里不替它定，只把落在那一格的次数记成一笔（`FaultInjectionTally::reopened_into_the_version_the_failed_step_was_writing`）。
//! 第 40 行那一格的判别力由写死的那条用例钉（`tests/second_transaction_supplement_three_fault_injection.rs`）。

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Instant;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, recover, JournalPolicy, RecoveryOutcome,
};

use crate::crash::{newest_persisted_root, writes_and_segments, MemoryPool, RecordCheck};
use crate::history::{
    classify_failure, execute_history_with_faults, generate_history_with_weights,
    newest_ring_root_and_slot_count, raised_floor_lands_only_on_abandoned_roots, AppliedEffect,
    ColdStartReadBack, FailureObservation, FailureSignature, GeneratedHistory, GenerationWeights,
    HistoryEnding, HistoryExecution, HistoryOperation, HistoryOperationKind, HistoryRun,
    HistorySeed, PerStepChecker, SeededRandomSource, StepOutcome, StepPosition,
};
use crate::model::{crash_recovery_disagreement, ModelRootKey, ObservedReadBack};
use crate::model_comparison::{model_root_key, observed_read_back_after_a_crash};
use crate::segments::{FixedGeometry, StepKind};
use crate::{RecordedOperation, RecordedOperationKind, SharedStream};

/// 故障注入的工作线程数从这个环境变量取（十进制正整数）；没设就取 [`std::thread::available_parallelism`]。
pub const FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_FAULT_INJECTION_WORKER_THREADS";

/// 每个工作线程摊到的片数：各段历史的长短差得远，片切得比线程多，先跑完的线程接着领下一片。
const SLICES_PER_WORKER_THREAD: usize = 4;

/// 调用块设备的三种动作里，注入点认得的那三种（`probe_physical_block_size` 与 `size_in_bytes` 不碰盘，不注入）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FaultCallKind {
    Read,
    Write,
    Barrier,
}

impl FaultCallKind {
    pub const ALL: [FaultCallKind; 3] = [
        FaultCallKind::Read,
        FaultCallKind::Write,
        FaultCallKind::Barrier,
    ];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            FaultCallKind::Read => "read",
            FaultCallKind::Write => "write",
            FaultCallKind::Barrier => "barrier",
        }
    }
}

/// 一次读回来的字节里翻掉哪一位：第几个字节、字节里第几位。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FlippedBit {
    /// 这次读的缓冲区里第几个字节（比缓冲区长时按缓冲区长度取模，读多短都翻得到一位）。
    pub byte_index: u64,
    /// 那个字节里第几位（0..8）。
    pub bit_index: u32,
}

impl FlippedBit {
    /// 缓冲区里真正被翻的那个字节的下标与掩码。
    #[must_use]
    fn within(self, buffer_length: usize) -> Option<(usize, u8)> {
        if buffer_length == 0 {
            return None;
        }
        let length = u64::try_from(buffer_length).expect("读的长度装得进 u64");
        let index = usize::try_from(self.byte_index % length).expect("取模之后装得回 usize");
        Some((index, 1u8 << (self.bit_index % 8)))
    }
}

/// 注入什么。每个成员自带它作用在哪一种调用上，写不出「让一次屏障返回改坏的字节」这种非法组合
/// （`code-discipline.md`「类型：让非法状态写不出来」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InjectedFault {
    /// 这一次写报块设备错，一个字节都不落盘。
    WriteFails,
    /// 这一次写报成功、其实一个字节都没落盘（写进了设备缓存又掉电的那一形）。
    WriteIsSwallowed,
    /// 这一次读报块设备错，缓冲区不动。
    ReadFails,
    /// 这一次读报成功，读回的字节里翻掉一位。
    ReadReturnsCorruptedBytes { flipped_bit: FlippedBit },
    /// 这一次刷盘报块设备错。
    BarrierFails,
    /// 这一次刷盘报成功、其实没发给设备（漏一道屏障；虚机档拿它量「少一道屏障多出哪些崩溃状态」）。
    BarrierIsSwallowed,
}

impl InjectedFault {
    #[must_use]
    pub const fn call_kind(self) -> FaultCallKind {
        match self {
            InjectedFault::WriteFails | InjectedFault::WriteIsSwallowed => FaultCallKind::Write,
            InjectedFault::ReadFails | InjectedFault::ReadReturnsCorruptedBytes { .. } => {
                FaultCallKind::Read
            }
            InjectedFault::BarrierFails | InjectedFault::BarrierIsSwallowed => {
                FaultCallKind::Barrier
            }
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            InjectedFault::WriteFails => "write_fails",
            InjectedFault::WriteIsSwallowed => "write_is_swallowed",
            InjectedFault::ReadFails => "read_fails",
            InjectedFault::ReadReturnsCorruptedBytes { .. } => "read_returns_corrupted_bytes",
            InjectedFault::BarrierFails => "barrier_fails",
            InjectedFault::BarrierIsSwallowed => "barrier_is_swallowed",
        }
    }
}

/// 注入落在哪块盘上。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultDeviceSelector {
    EveryDevice,
    OnlyDevice(DeviceIdentity),
}

/// 注入只挑落点落在这儿的调用（屏障没有落点，恒算命中）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultPlacement {
    AnyOffset,
    /// 偏移小于这个数（系统配置两槽住在偏移 0 起的那一段）。
    OffsetBelow(u64),
    OffsetExactly(DeviceOffsetInBytes),
}

impl FaultPlacement {
    #[must_use]
    fn matches(self, offset: DeviceOffsetInBytes) -> bool {
        match self {
            FaultPlacement::AnyOffset => true,
            FaultPlacement::OffsetBelow(end) => offset.0 < end,
            FaultPlacement::OffsetExactly(wanted) => offset == wanted,
        }
    }
}

/// 命中的调用按什么数：整个池数一份，还是每块盘各数一份。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultCounting {
    AcrossThePool,
    PerDevice,
}

/// 注入在第几次命中的调用上（序号从 1 起）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultOccurrence {
    /// 只在第 n 次命中的调用上注入一次，之后放行（随机注入那一路用这一种：一段历史一个注入点）。
    TheNthMatchingCall(u64),
    /// 第 n 次命中起，每一次都注入（手写包装里「之后每次写都报错」「屏障一直报错」那一种）。
    EveryMatchingCallFromTheNthOnward(u64),
}

impl FaultOccurrence {
    /// 每一次命中都注入。
    pub const EVERY_MATCHING_CALL: FaultOccurrence =
        FaultOccurrence::EveryMatchingCallFromTheNthOnward(1);

    #[must_use]
    fn fires_at(self, matching_call_ordinal: u64) -> bool {
        match self {
            FaultOccurrence::TheNthMatchingCall(ordinal) => matching_call_ordinal == ordinal,
            FaultOccurrence::EveryMatchingCallFromTheNthOnward(first) => {
                matching_call_ordinal >= first
            }
        }
    }
}

/// 一次注入怎么摆：注入什么、挑哪块盘、挑哪个落点、按什么数、在第几次上。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaultSchedule {
    pub fault: InjectedFault,
    pub device: FaultDeviceSelector,
    pub placement: FaultPlacement,
    pub counting: FaultCounting,
    pub occurrence: FaultOccurrence,
}

impl FaultSchedule {
    /// 整个池上第 `ordinal` 次这一种调用（随机注入那一路摆的形态）。
    #[must_use]
    pub fn the_nth_call_across_the_pool(fault: InjectedFault, ordinal: u64) -> Self {
        Self {
            fault,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::AnyOffset,
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::TheNthMatchingCall(ordinal),
        }
    }

    /// 每一次这一种调用都注入（手写包装里的开关那一种）。
    #[must_use]
    pub fn every_call_across_the_pool(fault: InjectedFault) -> Self {
        Self {
            fault,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::AnyOffset,
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
        }
    }

    #[must_use]
    fn matches(&self, device: DeviceIdentity, offset: DeviceOffsetInBytes) -> bool {
        let device_matches = match self.device {
            FaultDeviceSelector::EveryDevice => true,
            FaultDeviceSelector::OnlyDevice(wanted) => device == wanted,
        };
        device_matches && self.placement.matches(offset)
    }

    /// 报告里的名字。
    #[must_use]
    pub fn render(&self) -> String {
        let device = match self.device {
            FaultDeviceSelector::EveryDevice => "每块盘".to_string(),
            FaultDeviceSelector::OnlyDevice(identity) => format!("只盘 {}", identity.0),
        };
        let placement = match self.placement {
            FaultPlacement::AnyOffset => "任意落点".to_string(),
            FaultPlacement::OffsetBelow(end) => format!("偏移小于 {end}"),
            FaultPlacement::OffsetExactly(offset) => format!("偏移恰好 {}", offset.0),
        };
        let counting = match self.counting {
            FaultCounting::AcrossThePool => "整池数",
            FaultCounting::PerDevice => "逐盘数",
        };
        let occurrence = match self.occurrence {
            FaultOccurrence::TheNthMatchingCall(ordinal) => format!("第 {ordinal} 次"),
            FaultOccurrence::EveryMatchingCallFromTheNthOnward(first) => {
                format!("第 {first} 次起每一次")
            }
        };
        format!(
            "{}（{device}、{placement}、{counting}、{occurrence}）",
            self.fault.name()
        )
    }
}

/// 一块盘上真正发生过的调用数：报成功交给里面那块盘的那些。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FaultDeviceCounts {
    pub reads: u64,
    pub writes: u64,
    pub written_bytes: u64,
    pub force_unit_access_writes: u64,
    /// 真交给里面那块盘的屏障（被吞掉的那些不算）。
    pub barriers_forwarded: u64,
    pub barriers_swallowed: u64,
}

impl FaultDeviceCounts {
    /// 两次快照之差（一段窗口里发生了多少）。
    #[must_use]
    pub fn since(&self, earlier: &FaultDeviceCounts) -> FaultDeviceCounts {
        FaultDeviceCounts {
            reads: self.reads - earlier.reads,
            writes: self.writes - earlier.writes,
            written_bytes: self.written_bytes - earlier.written_bytes,
            force_unit_access_writes: self.force_unit_access_writes
                - earlier.force_unit_access_writes,
            barriers_forwarded: self.barriers_forwarded - earlier.barriers_forwarded,
            barriers_swallowed: self.barriers_swallowed - earlier.barriers_swallowed,
        }
    }
}

/// 整个池上调用方发出的调用数（注入与否都数，注入那一次也算发出过）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FaultCallCounts {
    pub reads: u64,
    pub writes: u64,
    pub barriers: u64,
}

impl FaultCallCounts {
    #[must_use]
    pub fn of(&self, kind: FaultCallKind) -> u64 {
        match kind {
            FaultCallKind::Read => self.reads,
            FaultCallKind::Write => self.writes,
            FaultCallKind::Barrier => self.barriers,
        }
    }
}

/// 注入真的发生过的那一次：注入了什么、落在哪块盘的哪个落点上、那次调用是整池第几次。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FiredFault {
    pub fault: InjectedFault,
    pub device: DeviceIdentity,
    pub offset: DeviceOffsetInBytes,
    pub length: u64,
    /// 写落在哪一类结构上（按落点分，`crate::segments::FixedGeometry::classify`）；读与屏障没有这一项。
    pub written_structure: Option<StepKind>,
    /// 这次调用是整池第几次这一种调用（1 起）。
    pub call_ordinal_across_the_pool: u64,
}

impl FiredFault {
    /// 报告里的名字：注入了什么、落在哪一类结构上。
    #[must_use]
    pub fn render(&self) -> String {
        let structure = match self.written_structure {
            Some(kind) => kind.name(),
            None => "-",
        };
        format!(
            "{}@{structure}（盘 {}、偏移 {}、整池第 {} 次{}）",
            self.fault.name(),
            self.device.0,
            self.offset.0,
            self.call_ordinal_across_the_pool,
            self.fault.call_kind().name()
        )
    }
}

struct FaultPlanState {
    geometry: FixedGeometry,
    schedule: Option<FaultSchedule>,
    calls: FaultCallCounts,
    matching_calls_across_the_pool: u64,
    matching_calls_per_device: BTreeMap<DeviceIdentity, u64>,
    per_device_counts: BTreeMap<DeviceIdentity, FaultDeviceCounts>,
    /// 每块盘收到第一道屏障那一刻这块盘的计数（屏障本身不算进去）：真设备那一路按它切「挂载」那一段窗口。
    counts_when_the_first_barrier_arrived: BTreeMap<DeviceIdentity, FaultDeviceCounts>,
    fired: Vec<FiredFault>,
}

/// 一个池里几块设备共用的注入计划：数调用、按计划注入、记下注入真的发生在哪一次。
/// 装的是 `Rc<RefCell<..>>`，写入口正拿着设备时调用方也开得了、关得掉（手写包装里的开关就是这么用的）。
#[derive(Clone)]
pub struct SharedFaultPlan(Rc<RefCell<FaultPlanState>>);

impl SharedFaultPlan {
    /// 不注入，只数调用。
    #[must_use]
    pub fn unarmed(geometry: FixedGeometry) -> Self {
        Self(Rc::new(RefCell::new(FaultPlanState {
            geometry,
            schedule: None,
            calls: FaultCallCounts::default(),
            matching_calls_across_the_pool: 0,
            matching_calls_per_device: BTreeMap::new(),
            per_device_counts: BTreeMap::new(),
            counts_when_the_first_barrier_arrived: BTreeMap::new(),
            fired: Vec::new(),
        })))
    }

    /// 开着一条计划起步。
    #[must_use]
    pub fn armed(geometry: FixedGeometry, schedule: FaultSchedule) -> Self {
        let plan = Self::unarmed(geometry);
        plan.arm(schedule);
        plan
    }

    /// 换上一条计划：命中计数从零重数（换计划就是换注入点，旧计划数到哪与新的无关）。
    pub fn arm(&self, schedule: FaultSchedule) {
        let mut state = self.0.borrow_mut();
        state.schedule = Some(schedule);
        state.matching_calls_across_the_pool = 0;
        state.matching_calls_per_device.clear();
    }

    /// 收起计划：之后每次调用都原样交给里面那块盘。
    pub fn disarm(&self) {
        self.0.borrow_mut().schedule = None;
    }

    #[must_use]
    pub fn calls(&self) -> FaultCallCounts {
        self.0.borrow().calls
    }

    #[must_use]
    pub fn counts_of_device(&self, device: DeviceIdentity) -> FaultDeviceCounts {
        self.0
            .borrow()
            .per_device_counts
            .get(&device)
            .copied()
            .unwrap_or_default()
    }

    /// 这块盘收到第一道屏障那一刻的计数；一道屏障都没收到过时 None。
    #[must_use]
    pub fn counts_when_the_first_barrier_arrived_at(
        &self,
        device: DeviceIdentity,
    ) -> Option<FaultDeviceCounts> {
        self.0
            .borrow()
            .counts_when_the_first_barrier_arrived
            .get(&device)
            .copied()
    }

    /// 注入真的发生过的那几次，按发生次序。
    #[must_use]
    pub fn fired(&self) -> Vec<FiredFault> {
        self.0.borrow().fired.clone()
    }

    #[must_use]
    pub fn fired_count(&self) -> usize {
        self.0.borrow().fired.len()
    }

    /// 这一次调用要不要注入；要注入就把它记进 `fired`。命中计数在这里推进，所以每次调用只许问一次。
    fn decide(
        &self,
        kind: FaultCallKind,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Option<InjectedFault> {
        let mut state = self.0.borrow_mut();
        match kind {
            FaultCallKind::Read => state.calls.reads += 1,
            FaultCallKind::Write => state.calls.writes += 1,
            FaultCallKind::Barrier => state.calls.barriers += 1,
        }
        let schedule = state.schedule?;
        if schedule.fault.call_kind() != kind || !schedule.matches(device, offset) {
            return None;
        }
        state.matching_calls_across_the_pool += 1;
        let per_device = state.matching_calls_per_device.entry(device).or_insert(0);
        *per_device += 1;
        let ordinal = match schedule.counting {
            FaultCounting::AcrossThePool => state.matching_calls_across_the_pool,
            FaultCounting::PerDevice => *per_device,
        };
        if !schedule.occurrence.fires_at(ordinal) {
            return None;
        }
        let written_structure = match kind {
            FaultCallKind::Write => Some(state.geometry.classify(&RecordedOperation {
                device,
                kind: RecordedOperationKind::Write,
                offset,
                length,
                content_hash: 0,
            })),
            FaultCallKind::Read | FaultCallKind::Barrier => None,
        };
        let call_ordinal_across_the_pool = state.calls.of(kind);
        state.fired.push(FiredFault {
            fault: schedule.fault,
            device,
            offset,
            length,
            written_structure,
            call_ordinal_across_the_pool,
        });
        Some(schedule.fault)
    }

    fn note_forwarded_read(&self, device: DeviceIdentity) {
        self.0
            .borrow_mut()
            .per_device_counts
            .entry(device)
            .or_default()
            .reads += 1;
    }

    fn note_forwarded_write(
        &self,
        device: DeviceIdentity,
        bytes: u64,
        durability: WriteDurability,
    ) {
        let mut state = self.0.borrow_mut();
        let counts = state.per_device_counts.entry(device).or_default();
        counts.writes += 1;
        counts.written_bytes += bytes;
        match durability {
            WriteDurability::Plain => {}
            WriteDurability::ForceUnitAccess => counts.force_unit_access_writes += 1,
        }
    }

    /// 屏障进来了：先给这块盘拍第一道屏障的快照（屏障本身不算进去），再记下它是发下去了还是被吞了。
    fn note_barrier_arrived(&self, device: DeviceIdentity) {
        let mut state = self.0.borrow_mut();
        let counts = *state.per_device_counts.entry(device).or_default();
        state
            .counts_when_the_first_barrier_arrived
            .entry(device)
            .or_insert(counts);
    }

    fn note_forwarded_barrier(&self, device: DeviceIdentity) {
        self.0
            .borrow_mut()
            .per_device_counts
            .entry(device)
            .or_default()
            .barriers_forwarded += 1;
    }

    fn note_swallowed_barrier(&self, device: DeviceIdentity) {
        self.0
            .borrow_mut()
            .per_device_counts
            .entry(device)
            .or_default()
            .barriers_swallowed += 1;
    }
}

/// 注入的那一个错：块设备的 I/O 错，消息里写明是注入的。
#[must_use]
pub fn injected_block_device_error(what: &str) -> BlockDeviceError {
    BlockDeviceError::InputOutput(std::io::Error::other(format!("注入的{what}错")))
}

/// 通用的故障注入设备包装：包在任意一块设备外面，按共用的计划让某一次读 / 写 / 刷盘报错、吞掉、或读回改坏的字节。
/// 几块盘共用同一个 [`SharedFaultPlan`]，序号因此可以整池数（「这个池第 7 次写」只有一次）。
pub struct FaultInjectingBlockDevice<Inner: BlockDevice> {
    inner: Inner,
    device: DeviceIdentity,
    plan: SharedFaultPlan,
}

impl<Inner: BlockDevice> FaultInjectingBlockDevice<Inner> {
    pub fn new(device: DeviceIdentity, inner: Inner, plan: SharedFaultPlan) -> Self {
        Self {
            inner,
            device,
            plan,
        }
    }

    #[must_use]
    pub fn inner(&self) -> &Inner {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }

    #[must_use]
    pub fn into_inner(self) -> Inner {
        self.inner
    }

    #[must_use]
    pub fn plan(&self) -> &SharedFaultPlan {
        &self.plan
    }
}

impl<Inner: BlockDevice> BlockDevice for FaultInjectingBlockDevice<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("读的长度装得进 u64");
        match self
            .plan
            .decide(FaultCallKind::Read, self.device, offset, length)
        {
            Some(InjectedFault::ReadFails) => return Err(injected_block_device_error("读")),
            Some(InjectedFault::ReadReturnsCorruptedBytes { flipped_bit }) => {
                self.inner.read_at(offset, buffer)?;
                self.plan.note_forwarded_read(self.device);
                if let Some((index, mask)) = flipped_bit.within(buffer.len()) {
                    buffer[index] ^= mask;
                }
                return Ok(());
            }
            // 读的计划只有这两条；写与屏障的计划在 `decide` 里按调用种类挡掉了，走不到这里。
            Some(
                InjectedFault::WriteFails
                | InjectedFault::WriteIsSwallowed
                | InjectedFault::BarrierFails
                | InjectedFault::BarrierIsSwallowed,
            )
            | None => {}
        }
        self.inner.read_at(offset, buffer)?;
        self.plan.note_forwarded_read(self.device);
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(bytes.len()).expect("写的长度装得进 u64");
        match self
            .plan
            .decide(FaultCallKind::Write, self.device, offset, length)
        {
            Some(InjectedFault::WriteFails) => return Err(injected_block_device_error("写")),
            // 吞掉：报成功，一个字节都不落盘，也不记进这块盘的计数（真的没写）。
            Some(InjectedFault::WriteIsSwallowed) => return Ok(()),
            Some(
                InjectedFault::ReadFails
                | InjectedFault::ReadReturnsCorruptedBytes { .. }
                | InjectedFault::BarrierFails
                | InjectedFault::BarrierIsSwallowed,
            )
            | None => {}
        }
        self.inner.write_at(offset, bytes, durability)?;
        self.plan
            .note_forwarded_write(self.device, length, durability);
        Ok(())
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.plan.note_barrier_arrived(self.device);
        match self.plan.decide(
            FaultCallKind::Barrier,
            self.device,
            DeviceOffsetInBytes(0),
            0,
        ) {
            Some(InjectedFault::BarrierFails) => return Err(injected_block_device_error("屏障")),
            Some(InjectedFault::BarrierIsSwallowed) => {
                self.plan.note_swallowed_barrier(self.device);
                return Ok(());
            }
            Some(
                InjectedFault::WriteFails
                | InjectedFault::WriteIsSwallowed
                | InjectedFault::ReadFails
                | InjectedFault::ReadReturnsCorruptedBytes { .. },
            )
            | None => {}
        }
        self.inner.barrier()?;
        self.plan.note_forwarded_barrier(self.device);
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 一段历史上注入哪一次调用：注入什么，加上那次调用是整池第几次这一种调用。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DrawnFault {
    pub fault: InjectedFault,
    /// 整池第几次这一种调用（1 起）。
    pub call_ordinal: u64,
    /// 这次调用落在历史的第几步里（起点之后，从 0 数）。
    pub step_index: usize,
    pub operation_kind: HistoryOperationKind,
}

impl DrawnFault {
    #[must_use]
    pub fn render(&self) -> String {
        format!(
            "{}：整池第 {} 次{}，落在第 {} 步（{:?}）",
            self.fault.name(),
            self.call_ordinal,
            self.fault.call_kind().name(),
            self.step_index,
            self.operation_kind
        )
    }
}

/// 一个注入点：哪一类操作上的哪一种调用（写再按落点分成哪一类结构）。验收要的「每个注入点的命中次数」按它数。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct InjectionPoint {
    pub operation: HistoryOperationKind,
    pub call: FaultCallKind,
    /// 写落在哪一类结构上；读与屏障没有这一项。
    pub written_structure: Option<StepKind>,
}

/// 写落在哪一类结构上，四种（`StepKind::Barrier` 不是写）。
const WRITTEN_STRUCTURES: [StepKind; 4] = [
    StepKind::UnitWrite,
    StepKind::JournalRecord,
    StepKind::RootRecordFua,
    StepKind::SystemConfigurationSlot,
];

impl InjectionPoint {
    /// 全部注入点：七类操作 × （四类写落点 + 读 + 屏障）。里面有几格在今天的实现上摆不出来
    /// （冷启动只读不写、零单元发布不写单元），报告里按「没命中」列名，不当失败。
    #[must_use]
    pub fn all() -> Vec<InjectionPoint> {
        let mut points = Vec::new();
        for operation in HistoryOperationKind::ALL {
            for structure in WRITTEN_STRUCTURES {
                points.push(InjectionPoint {
                    operation,
                    call: FaultCallKind::Write,
                    written_structure: Some(structure),
                });
            }
            for call in [FaultCallKind::Read, FaultCallKind::Barrier] {
                points.push(InjectionPoint {
                    operation,
                    call,
                    written_structure: None,
                });
            }
        }
        points
    }

    #[must_use]
    pub fn render(&self) -> String {
        match self.written_structure {
            Some(structure) => format!("{:?}/{}", self.operation, structure.name()),
            None => format!("{:?}/{}", self.operation, self.call.name()),
        }
    }
}

/// 注入之后重开走到的那一版落在哪一格。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReopenedVersion {
    /// 走到的是模型认下来的某一版（注入那一段历史跑完时模型根环里的那几条，`HistoryRun::committed_versions_at_the_end`）。
    CommittedByTheModel,
    /// 模型没认这一版，但它正是不注入时失败那一步会写出的那一版（增补 2 收口表第 40 行 / C381 在争的那一格；这里只记不判）。
    TheVersionTheFaultedStepWasWriting,
    /// 两样都不是：`crash_recovery_disagreement` 判出对不上。
    OutsideEverythingTheModelAllows,
}

/// 注入之后这一次怎么收尾。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FaultOutcome {
    /// 注入那一步返回了 `Err`（或冷启动读回报错），历史到此为止：要的就是这一条。
    SurfacedAsAnError,
    /// 注入了，那一步照样做成了（读错被容错、吞掉的写没被用到），整段历史跑完。
    ToleratedAndTheHistoryFinished,
    /// 注入了，历史却停在别的东西上（不是注入那一步返回的错）：这一格逐条分类，有真缺口就在这里。
    StoppedOnSomethingOtherThanTheInjectedError,
    /// 注入点没走到（这一段历史在注入那一次调用之前就停了）。
    NeverFired,
}

impl FaultOutcome {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            FaultOutcome::SurfacedAsAnError => "返回了错误",
            FaultOutcome::ToleratedAndTheHistoryFinished => "被容错、历史跑完",
            FaultOutcome::StoppedOnSomethingOtherThanTheInjectedError => "历史停在别的东西上",
            FaultOutcome::NeverFired => "没走到",
        }
    }
}

/// 一次注入上的一条失败。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaultFinding {
    pub signature: FailureSignature,
    pub seed: HistorySeed,
    pub drawn: DrawnFault,
    pub observation: FailureObservation,
    /// 重开走到了哪一版（判红的那一次才带）。
    pub reopen: String,
}

impl FaultFinding {
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = format!(
            "种子 {} {}：{:?}\n",
            self.seed.0,
            self.drawn.render(),
            self.signature
        );
        let _ = writeln!(text, "  重开：{}", self.reopen);
        if let Some(panic) = &self.observation.panic {
            let _ = writeln!(text, "  panic：{} —— {}", panic.location, panic.message);
        }
        for (invariant, detail) in &self.observation.violations {
            let _ = writeln!(text, "  {invariant}：{detail}");
        }
        if let Some(disagreement) = &self.observation.model_disagreement {
            let _ = writeln!(
                text,
                "  模型对不上（{}）：模型 {}；实现 {}",
                disagreement.aspect.name(),
                disagreement.model_answer,
                disagreement.implementation_answer
            );
        }
        text
    }
}

/// 「已知红」收尾的一次注入。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownRedAtAFault {
    pub seed: HistorySeed,
    pub form: usize,
    pub drawn: DrawnFault,
}

/// 跑过的注入的计数：绝对数，证明各条路径真的跑到了（`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FaultInjectionTally {
    pub histories: u64,
    /// 测量跑（不注入）就以「已知红」收尾的段数：注入点只摆在测量跑跑完的那几步里。
    pub histories_whose_measurement_run_stopped_early: u64,
    pub histories_without_any_injection_point: u64,
    pub faults: u64,
    pub faults_by_kind: BTreeMap<&'static str, u64>,
    pub faults_by_outcome: BTreeMap<&'static str, u64>,
    pub faults_by_injection_point: BTreeMap<String, u64>,
    /// 注入之后整段历史里 `singlefs-core` panic 的次数：这是被测的性质，一次都不许有。
    pub faults_that_panicked: u64,
    /// 注入那一步返回的错误成员（`StepOutcome::Refused` 的成员名）。
    pub refusal_members: BTreeMap<String, u64>,
    pub reopens: u64,
    pub reopens_reading_a_file: u64,
    pub reopens_without_a_file: u64,
    pub reopens_that_failed: u64,
    pub reopened_into_a_version_the_model_committed: u64,
    /// 重开走到了失败那一步正在写、模型没提交的那一版（收口表第 40 行那一格；只记不判）。
    pub reopened_into_the_version_the_faulted_step_was_writing: u64,
    pub reopens_outside_everything_the_model_allows: u64,
    pub checker_runs_on_the_image_after_the_fault: u64,
    pub invariant_holds: BTreeMap<&'static str, u64>,
    pub invariant_not_applicable: BTreeMap<&'static str, u64>,
    pub faults_ending_known_red: BTreeMap<usize, u64>,
    pub faults_ending_new_finding: u64,
}

impl FaultInjectionTally {
    /// 把另一份计数并进来（按片的次序相加，结果与线程数无关）。
    pub fn absorb(&mut self, other: &FaultInjectionTally) {
        self.histories += other.histories;
        self.histories_whose_measurement_run_stopped_early +=
            other.histories_whose_measurement_run_stopped_early;
        self.histories_without_any_injection_point += other.histories_without_any_injection_point;
        self.faults += other.faults;
        for (key, count) in &other.faults_by_kind {
            *self.faults_by_kind.entry(key).or_insert(0) += count;
        }
        for (key, count) in &other.faults_by_outcome {
            *self.faults_by_outcome.entry(key).or_insert(0) += count;
        }
        for (key, count) in &other.faults_by_injection_point {
            *self
                .faults_by_injection_point
                .entry(key.clone())
                .or_insert(0) += count;
        }
        self.faults_that_panicked += other.faults_that_panicked;
        for (key, count) in &other.refusal_members {
            *self.refusal_members.entry(key.clone()).or_insert(0) += count;
        }
        self.reopens += other.reopens;
        self.reopens_reading_a_file += other.reopens_reading_a_file;
        self.reopens_without_a_file += other.reopens_without_a_file;
        self.reopens_that_failed += other.reopens_that_failed;
        self.reopened_into_a_version_the_model_committed +=
            other.reopened_into_a_version_the_model_committed;
        self.reopened_into_the_version_the_faulted_step_was_writing +=
            other.reopened_into_the_version_the_faulted_step_was_writing;
        self.reopens_outside_everything_the_model_allows +=
            other.reopens_outside_everything_the_model_allows;
        self.checker_runs_on_the_image_after_the_fault +=
            other.checker_runs_on_the_image_after_the_fault;
        for (key, count) in &other.invariant_holds {
            *self.invariant_holds.entry(key).or_insert(0) += count;
        }
        for (key, count) in &other.invariant_not_applicable {
            *self.invariant_not_applicable.entry(key).or_insert(0) += count;
        }
        for (form, count) in &other.faults_ending_known_red {
            *self.faults_ending_known_red.entry(*form).or_insert(0) += count;
        }
        self.faults_ending_new_finding += other.faults_ending_new_finding;
    }

    /// 一个注入点都没命中的那几格（验收要的「没命中的逐个列名」）。
    #[must_use]
    pub fn injection_points_never_hit(&self) -> Vec<String> {
        InjectionPoint::all()
            .into_iter()
            .map(|point| point.render())
            .filter(|name| !self.faults_by_injection_point.contains_key(name))
            .collect()
    }

    /// 这一类操作上注入过几次（验收要的「每个发布步骤与挂载步骤上各注入过至少一次」按它判）。
    #[must_use]
    pub fn faults_on_operation(&self, operation: HistoryOperationKind) -> u64 {
        let prefix = format!("{operation:?}/");
        self.faults_by_injection_point
            .iter()
            .filter(|(name, _)| name.starts_with(&prefix))
            .map(|(_, count)| count)
            .sum()
    }

    /// 给人看的一整块：绝对数。
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "历史 {} 段：测量跑就提前停的 {} 段、摆不出注入点的 {} 段；注入 {} 次",
            self.histories,
            self.histories_whose_measurement_run_stopped_early,
            self.histories_without_any_injection_point,
            self.faults
        );
        for (kind, count) in &self.faults_by_kind {
            let _ = writeln!(text, "  注入 {kind}：{count} 次");
        }
        for (outcome, count) in &self.faults_by_outcome {
            let _ = writeln!(text, "  收尾「{outcome}」：{count} 次");
        }
        let _ = writeln!(
            text,
            "注入之后 core panic 的次数：{}（被测的性质：返回错误而不是 panic）",
            self.faults_that_panicked
        );
        for operation in HistoryOperationKind::ALL {
            let _ = writeln!(
                text,
                "  {operation:?} 上注入 {} 次",
                self.faults_on_operation(operation)
            );
        }
        for (point, count) in &self.faults_by_injection_point {
            let _ = writeln!(text, "    注入点 {point}：{count} 次");
        }
        let never_hit = self.injection_points_never_hit();
        let _ = writeln!(
            text,
            "  没命中的注入点 {} 个：{}",
            never_hit.len(),
            never_hit.join("、")
        );
        for (member, count) in &self.refusal_members {
            let _ = writeln!(text, "  返回的错误成员 {member}：{count} 次");
        }
        let _ = writeln!(
            text,
            "重开 {} 次：读回文件 {}、没有文件 {}、走读失败 {}",
            self.reopens,
            self.reopens_reading_a_file,
            self.reopens_without_a_file,
            self.reopens_that_failed
        );
        let _ = writeln!(
            text,
            "  走到模型认下的某一版 {} 次、走到失败那一步正在写的那一版 {} 次（收口表第 40 行那一格，只记不判）、模型都不允许的 {} 次",
            self.reopened_into_a_version_the_model_committed,
            self.reopened_into_the_version_the_faulted_step_was_writing,
            self.reopens_outside_everything_the_model_allows
        );
        let _ = writeln!(
            text,
            "注入之后的镜像上跑池级 checker {} 次：判绿 {} 条次、不适用 {} 条次",
            self.checker_runs_on_the_image_after_the_fault,
            self.invariant_holds.values().sum::<u64>(),
            self.invariant_not_applicable.values().sum::<u64>()
        );
        let _ = writeln!(
            text,
            "以「已知红」收尾 {:?}、新发现 {}",
            self.faults_ending_known_red, self.faults_ending_new_finding
        );
        text
    }
}

/// 测量跑里每一步跑完时记下的东西：这一步是什么、到这一步为止读 / 写 / 刷盘各调了多少次、模型到这一步提交过哪些版本。
struct StepMark {
    position: StepPosition,
    operation_kind: Option<HistoryOperationKind>,
    calls: FaultCallCounts,
    committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
}

/// 一段历史注入故障之后交回的东西。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryFaultInjection {
    pub seed: HistorySeed,
    /// 摆出来的注入点，按调用序号从小到大。
    pub drawn_faults: Vec<DrawnFault>,
    pub tally: FaultInjectionTally,
    pub known_red_hits: Vec<KnownRedAtAFault>,
    /// 同一段历史里同一个签名只留第一个。
    pub new_findings: Vec<FaultFinding>,
}

/// 这个镜像上，最新那条根带的回退下界 F 落不落在回退留下的空档里（与活盘面、崩溃状态两路同一个谓词、同一份实现）。
fn raised_floor_of_the_newest_root_lands_only_on_abandoned_roots(
    image: &MemoryPool,
) -> Option<bool> {
    let system_configuration = choose_system_configuration(image).ok()?;
    let newest_root = choose_root(image, &system_configuration)?;
    raised_floor_lands_only_on_abandoned_roots(image, newest_root.rollback_floor)
}

/// 对一份镜像跑池级 checker，记每条不变量判绿、不适用各几次，交回判红的那几条（与 `history.rs` 的 `violations_on` 同一条口径，
/// 计数进的是故障注入自己的那一份）。
fn checker_violations_on(
    image: &MemoryPool,
    tally: &mut FaultInjectionTally,
) -> Vec<(&'static str, String)> {
    tally.checker_runs_on_the_image_after_the_fault += 1;
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

/// 随机注入抽的就是里程碑「增补 3」第 4 件逐字写的那几种：「让任意一次写、读、刷盘返回 `BlockDeviceError`，
/// 或让一次读返回改坏的字节」。四种等概率。
///
/// [`InjectedFault::WriteIsSwallowed`] 与 [`InjectedFault::BarrierIsSwallowed`] 不抽：它们是**设备说谎**
/// （报了成功却什么都没做），不是「返回错误」。一次被吞掉的写在盘上无声地丢一份内容，任何文件系统都发现不了
/// （写入口拿到的是 Ok，校验和是它自己算的，读回来才看得见）——拿它判「实现要返回错误而不是 panic」是在判一件实现做不到的事。
/// 包装里留着这两种：漏一道屏障是真设备那一路量「少一道屏障多出哪些崩溃状态」的手段（`bin/first_transaction_on_device.rs`），
/// 丢一次写要配崩溃注入一起用（增补 3 第 3 件的枚举域）。实测：在快档上抽 `WriteIsSwallowed`，
/// 24 段里有 2 段的池级 checker 判出 I-2.1 / I-4.8 / I-7.4 红（根指着一份从没落盘的单元），与这一条判断相符。
fn draw_injected_fault(random: &mut SeededRandomSource) -> InjectedFault {
    let variety = usize::try_from(random.below(4)).expect("小于 4");
    match variety {
        0 => InjectedFault::WriteFails,
        1 => InjectedFault::ReadFails,
        2 => InjectedFault::ReadReturnsCorruptedBytes {
            flipped_bit: FlippedBit {
                byte_index: random.next_word(),
                bit_index: u32::try_from(random.below(8)).expect("小于 8"),
            },
        },
        3 => InjectedFault::BarrierFails,
        // `below(4)` 的值域就是 0..4，第五种写不出来。
        other => unreachable!("按 below(4) 抽出来的只有 0..4，抽到了 {other}"),
    }
}

/// 按种子在测量跑记下的调用里摆注入点：先抽一种注入，再在「发过这一种调用」的那几步里均匀抽一步，
/// 最后在那一步发出的那几次调用里均匀抽一次。先抽步再抽调用（不是在整条调用流里均匀抽）是为了摊开到各类操作上——
/// 一次发布发几十次写、一次冷启动只发几十次读，整条流上均匀抽会把注入点全堆在写得最多的那几步里。
/// 同一个 (种类, 序号) 只摆一次。
fn draw_faults(
    seed: HistorySeed,
    marks: &[StepMark],
    faults_per_history: usize,
) -> Vec<DrawnFault> {
    let mut random = SeededRandomSource::from_seed(seed.0 ^ 0x46_41_55_4c_54_00_00_01);
    let mut drawn: Vec<DrawnFault> = Vec::new();
    let mut taken: BTreeSet<(FaultCallKind, u64)> = BTreeSet::new();
    for _ in 0..faults_per_history {
        let fault = draw_injected_fault(&mut random);
        let kind = fault.call_kind();
        // 起点那一段不摆：起点（mkfs、取号、暖机、第一个文件）在执行器里是 `expect`，注入在那里只会撞出执行器自己的 panic，
        // 不是 `singlefs-core` 的缺口（增补 3 第 6 件要在前提之外各调一次，那是另一件）。
        let candidate_steps: Vec<usize> = (1..marks.len())
            .filter(|index| marks[*index].calls.of(kind) > marks[index - 1].calls.of(kind))
            .collect();
        if candidate_steps.is_empty() {
            continue;
        }
        let picked = candidate_steps[usize::try_from(
            random.below(u64::try_from(candidate_steps.len()).expect("步数")),
        )
        .expect("下标")];
        let first = marks[picked - 1].calls.of(kind) + 1;
        let last = marks[picked].calls.of(kind);
        let call_ordinal = first + random.below(last - first + 1);
        if !taken.insert((kind, call_ordinal)) {
            continue;
        }
        let StepPosition::Operation(step_index) = marks[picked].position else {
            continue;
        };
        let Some(operation_kind) = marks[picked].operation_kind else {
            continue;
        };
        drawn.push(DrawnFault {
            fault,
            call_ordinal,
            step_index,
            operation_kind,
        });
    }
    drawn.sort();
    drawn
}

/// 跑一段历史，按种子在它起点之后的读 / 写 / 刷盘里摆 `faults_per_history` 个注入点，每个注入点重跑一遍这段历史，
/// 判「返回错误而不是 panic」与「重开恢复到模型允许的版本」。
#[must_use]
pub fn inject_faults_into_history(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    faults_per_history: usize,
) -> HistoryFaultInjection {
    let geometry = execution.device_width.fixed_geometry();
    let mut tally = FaultInjectionTally {
        histories: 1,
        ..FaultInjectionTally::default()
    };

    // 一、测量跑：不注入，不跑池级 checker（那一档由随机历史与崩溃注入罩着），只要每一步的调用数与模型目录。
    let measurement_execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: execution.device_width,
    };
    let measurement_plan = SharedFaultPlan::unarmed(geometry);
    let measurement_stream = SharedStream::new();
    let mut marks: Vec<StepMark> = Vec::new();
    let mut committed_so_far: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = BTreeMap::new();
    let measurement_run = execute_history_with_faults(
        history,
        measurement_execution,
        &measurement_stream,
        &measurement_plan,
        &mut |observation| {
            for (key, file) in observation.model.committed_versions() {
                committed_so_far.insert(key, file);
            }
            marks.push(StepMark {
                position: observation.position,
                operation_kind: observation.operation.map(HistoryOperation::kind),
                calls: measurement_plan.calls(),
                committed_versions: committed_so_far.clone(),
            });
        },
    );
    match measurement_run.ending {
        HistoryEnding::Completed => {}
        HistoryEnding::KnownRed { .. } | HistoryEnding::NewFinding { .. } => {
            tally.histories_whose_measurement_run_stopped_early = 1;
        }
    }

    let drawn_faults = draw_faults(history.seed, &marks, faults_per_history);
    if drawn_faults.is_empty() {
        tally.histories_without_any_injection_point = 1;
    }
    let mut known_red_hits = Vec::new();
    let mut new_findings: Vec<FaultFinding> = Vec::new();
    {
        let mut accumulator = FaultInjectionAccumulator {
            tally: &mut tally,
            known_red_hits: &mut known_red_hits,
            new_findings: &mut new_findings,
        };
        for drawn in &drawn_faults {
            inject_one_fault(history, execution, &marks, drawn, &mut accumulator);
        }
    }
    HistoryFaultInjection {
        seed: history.seed,
        drawn_faults,
        tally,
        known_red_hits,
        new_findings,
    }
}

/// 一批注入攒出来的东西：计数、「已知红」命中、新发现。
struct FaultInjectionAccumulator<'run> {
    tally: &'run mut FaultInjectionTally,
    known_red_hits: &'run mut Vec<KnownRedAtAFault>,
    new_findings: &'run mut Vec<FaultFinding>,
}

/// 注入那一步的结局是不是「入口返回了错误」：没 panic、checker 没判红，而且那一步要么返回了 `Err`，
/// 要么是冷启动、读回报了错。注入的块设备错在模型里没有对应的拒绝理由（`ObservedRefusalReason::Unexplained`），
/// 模型因此必然报「该成却拒了」——那一格不算失败，它就是这一件要的结果。
fn the_fault_surfaced_as_an_error(
    run: &HistoryRun,
    observation: &FailureObservation,
    step_index: usize,
) -> bool {
    if observation.position != StepPosition::Operation(step_index)
        || observation.panic.is_some()
        || !observation.violations.is_empty()
    {
        return false;
    }
    match run.outcomes.last() {
        Some(StepOutcome::Refused { .. }) => true,
        Some(StepOutcome::Applied(AppliedEffect::Recovered { read_back, .. })) => match read_back {
            ColdStartReadBack::Failed => true,
            ColdStartReadBack::NoFile | ColdStartReadBack::FileRead => false,
        },
        Some(StepOutcome::Applied(
            AppliedEffect::Published { .. }
            | AppliedEffect::Mounted { .. }
            | AppliedEffect::RaisedFloor { .. },
        ))
        | Some(StepOutcome::NotApplicable(_))
        | None => false,
    }
}

/// 注入一次：armed 的计划重跑这段历史，判 panic、判那一步的结局、再重开判恢复到的那一版。
fn inject_one_fault(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    marks: &[StepMark],
    drawn: &DrawnFault,
    accumulator: &mut FaultInjectionAccumulator<'_>,
) {
    let geometry = execution.device_width.fixed_geometry();
    let plan = SharedFaultPlan::armed(
        geometry,
        FaultSchedule::the_nth_call_across_the_pool(drawn.fault, drawn.call_ordinal),
    );
    let stream = SharedStream::retaining_contents();
    let mut committed_by_the_armed_run: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = BTreeMap::new();
    let run = execute_history_with_faults(history, execution, &stream, &plan, &mut |observation| {
        for (key, file) in observation.model.committed_versions() {
            committed_by_the_armed_run.insert(key, file);
        }
    });
    let fired = plan.fired();
    accumulator.tally.faults += 1;
    *accumulator
        .tally
        .faults_by_kind
        .entry(drawn.fault.name())
        .or_insert(0) += 1;
    if let Some(first) = fired.first() {
        let point = InjectionPoint {
            operation: drawn.operation_kind,
            call: drawn.fault.call_kind(),
            written_structure: first.written_structure,
        };
        *accumulator
            .tally
            .faults_by_injection_point
            .entry(point.render())
            .or_insert(0) += 1;
    }

    let ending_observation = match &run.ending {
        HistoryEnding::Completed => None,
        HistoryEnding::KnownRed { observation, .. }
        | HistoryEnding::NewFinding { observation, .. } => Some(observation),
    };
    let surfaced = ending_observation.is_some_and(|observation| {
        the_fault_surfaced_as_an_error(&run, observation, drawn.step_index)
    });
    if surfaced {
        if let Some(StepOutcome::Refused { member }) = run.outcomes.last() {
            *accumulator
                .tally
                .refusal_members
                .entry(member.clone())
                .or_insert(0) += 1;
        }
    }
    let outcome = if fired.is_empty() {
        FaultOutcome::NeverFired
    } else if surfaced {
        FaultOutcome::SurfacedAsAnError
    } else {
        match &run.ending {
            HistoryEnding::Completed => FaultOutcome::ToleratedAndTheHistoryFinished,
            HistoryEnding::KnownRed { .. } | HistoryEnding::NewFinding { .. } => {
                FaultOutcome::StoppedOnSomethingOtherThanTheInjectedError
            }
        }
    };
    *accumulator
        .tally
        .faults_by_outcome
        .entry(outcome.name())
        .or_insert(0) += 1;

    // 二、重开：拿录制流（只记真正落到盘上的那些写）重建镜像，跑恢复，问模型走到的这一版允不允许。
    let operations = stream.retained_operations();
    let mut image = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    image.apply(&operations);
    let (writes, _segments) = writes_and_segments(&operations, &geometry);
    let all_persisted = vec![true; writes.len()];
    let newest_persisted = newest_persisted_root(&writes, &all_persisted)
        .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
    let recovery = recover(&image, JournalPolicy::Consult);
    accumulator.tally.reopens += 1;
    match &recovery.outcome {
        RecoveryOutcome::FileRead { .. } => accumulator.tally.reopens_reading_a_file += 1,
        RecoveryOutcome::NoFile { .. } => accumulator.tally.reopens_without_a_file += 1,
        RecoveryOutcome::Failed { .. } => accumulator.tally.reopens_that_failed += 1,
    }
    let read_back = observed_read_back_after_a_crash(&recovery);
    // 模型认下来的每一版：这段历史（开着注入跑的那一遍）停下时模型根环里的那几条。观察者攒的那一份漏掉失败那一步——
    // 判出失败的那一步不调观察者，而模型常常已经认下那一版了，两份并起来才完整。
    let mut committed_by_the_model = committed_by_the_armed_run.clone();
    for (key, file) in &run.committed_versions_at_the_end {
        committed_by_the_model.insert(
            *key,
            file.as_ref().map(|content| Rc::from(content.as_slice())),
        );
    }
    // 再放行一样：不注入时失败那一步会写出的那几版（测量跑记的那一份）。
    let mut allowed = committed_by_the_model.clone();
    if let Some(index) = marks
        .iter()
        .position(|mark| mark.position == StepPosition::Operation(drawn.step_index))
    {
        for (key, file) in &marks[index].committed_versions {
            allowed.insert(*key, file.clone());
        }
    }
    let disagreement = crash_recovery_disagreement(&allowed, newest_persisted, &read_back);
    let reopened =
        if crash_recovery_disagreement(&committed_by_the_model, newest_persisted, &read_back)
            .is_none()
        {
            accumulator
                .tally
                .reopened_into_a_version_the_model_committed += 1;
            ReopenedVersion::CommittedByTheModel
        } else if disagreement.is_none() {
            accumulator
                .tally
                .reopened_into_the_version_the_faulted_step_was_writing += 1;
            ReopenedVersion::TheVersionTheFaultedStepWasWriting
        } else {
            accumulator
                .tally
                .reopens_outside_everything_the_model_allows += 1;
            ReopenedVersion::OutsideEverythingTheModelAllows
        };
    let violations_after_the_fault = checker_violations_on(&image, accumulator.tally);
    let reopen_text = format!("{reopened:?}（{}）", describe_read_back(&read_back));

    // 三、判：注入那一步返回错误的那一格不算失败；别的失败（panic、checker 判红、重开走到模型不允许的版本）逐条分类。
    let mut failures: Vec<FailureObservation> = Vec::new();
    if !surfaced {
        if let Some(observation) = ending_observation {
            failures.push(observation.clone());
        }
    }
    if !violations_after_the_fault.is_empty() || disagreement.is_some() {
        let (newest_ring_root_txg, root_ring_slot_count) = newest_ring_root_and_slot_count(&image);
        failures.push(FailureObservation {
            position: StepPosition::Operation(drawn.step_index),
            operation_kind: Some(drawn.operation_kind),
            violations: violations_after_the_fault,
            panic: None,
            newest_ring_root_txg,
            root_ring_slot_count,
            harness_judgement: None,
            model_disagreement: disagreement,
            raised_floor_lands_only_on_abandoned_roots:
                raised_floor_of_the_newest_root_lands_only_on_abandoned_roots(&image),
            record_check: RecordCheck::default(),
        });
    }
    for observation in failures {
        if observation.panic.is_some() {
            accumulator.tally.faults_that_panicked += 1;
        }
        match classify_failure(observation) {
            HistoryEnding::Completed => {}
            HistoryEnding::KnownRed { form, .. } => {
                *accumulator
                    .tally
                    .faults_ending_known_red
                    .entry(form)
                    .or_insert(0) += 1;
                accumulator.known_red_hits.push(KnownRedAtAFault {
                    seed: history.seed,
                    form,
                    drawn: *drawn,
                });
            }
            HistoryEnding::NewFinding {
                signature,
                observation,
            } => {
                accumulator.tally.faults_ending_new_finding += 1;
                if !accumulator
                    .new_findings
                    .iter()
                    .any(|finding| finding.signature == signature)
                {
                    accumulator.new_findings.push(FaultFinding {
                        signature,
                        seed: history.seed,
                        drawn: *drawn,
                        observation,
                        reopen: reopen_text.clone(),
                    });
                }
            }
        }
    }
}

/// 重开读回了什么，给人看的一行。
fn describe_read_back(read_back: &ObservedReadBack) -> String {
    match read_back {
        ObservedReadBack::NoFile { root } => {
            format!(
                "走到实例 {} 第 {} 代根、没有文件",
                root.instance.0, root.checkpoint_txg.0
            )
        }
        ObservedReadBack::FileRead { root, content } => format!(
            "走到实例 {} 第 {} 代根、读回 {} 字节",
            root.instance.0,
            root.checkpoint_txg.0,
            content.len()
        ),
        ObservedReadBack::Failed { what } => format!("走读失败：{what}"),
    }
}

/// 工作线程数是从哪来的：与实际起的线程数一起打进进度行，「机器多于 1 核却只用了 1 个线程」看得出来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultInjectionWorkerThreads {
    /// 环境变量 [`FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE`] 显式给的。
    FromTheEnvironmentVariable(usize),
    /// 没设环境变量，取 `available_parallelism`。
    FromAvailableParallelism(usize),
    /// 没设环境变量，`available_parallelism` 也报不出来：只用 1 个线程，照实报出来。
    AvailableParallelismUnknown,
    /// 调用方在代码里直接给的（用例拿不同线程数对拍）。
    GivenByCaller(usize),
}

impl FaultInjectionWorkerThreads {
    /// 线程数取环境变量，没设就取 `available_parallelism`。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数（含 0、空串、非 UTF-8）：配错了就停，不悄悄退回单线程。
    #[must_use]
    pub fn from_the_environment() -> Self {
        Self::from_the_environment_value(
            std::env::var(FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE),
            std::thread::available_parallelism,
        )
    }

    /// [`Self::from_the_environment`] 的判定本身：环境变量读到什么、`available_parallelism` 报什么都由调用方给
    /// （用例不改进程的环境变量）。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数。
    #[must_use]
    fn from_the_environment_value(
        variable: Result<String, std::env::VarError>,
        available_parallelism: impl Fn() -> std::io::Result<std::num::NonZeroUsize>,
    ) -> Self {
        match variable {
            Ok(text) => {
                let count: usize = text.trim().parse().unwrap_or_else(|error| {
                    panic!(
                        "{FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE}={text} 不是十进制正整数：{error}"
                    )
                });
                assert!(
                    count > 0,
                    "{FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE}={text}：至少要 1 个线程"
                );
                Self::FromTheEnvironmentVariable(count)
            }
            Err(std::env::VarError::NotPresent) => match available_parallelism() {
                Ok(count) => Self::FromAvailableParallelism(count.get()),
                Err(_) => Self::AvailableParallelismUnknown,
            },
            Err(std::env::VarError::NotUnicode(raw)) => {
                panic!("{FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE} 不是 UTF-8：{raw:?}")
            }
        }
    }

    #[must_use]
    pub fn count(self) -> usize {
        match self {
            Self::FromTheEnvironmentVariable(count)
            | Self::FromAvailableParallelism(count)
            | Self::GivenByCaller(count) => count,
            Self::AvailableParallelismUnknown => 1,
        }
    }

    #[must_use]
    pub fn source_name(self) -> &'static str {
        match self {
            Self::FromTheEnvironmentVariable(_) => "environment_variable",
            Self::FromAvailableParallelism(_) => "available_parallelism",
            Self::AvailableParallelismUnknown => "available_parallelism_unknown",
            Self::GivenByCaller(_) => "given_by_caller",
        }
    }
}

/// 一次故障注入跑什么：种子区间、每段几步、每段注入几次、比重、历史怎么跑、线程数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaultInjectionCampaign {
    pub first_seed: u64,
    pub seed_count: u64,
    pub operations_per_history: usize,
    pub faults_per_history: usize,
    pub weights: GenerationWeights,
    pub execution: HistoryExecution,
    pub worker_threads: FaultInjectionWorkerThreads,
}

/// 一批种子跑下来的故障注入报告。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaultInjectionReport {
    pub first_seed: u64,
    pub seed_count: u64,
    pub operations_per_history: usize,
    pub faults_per_history: usize,
    pub weights: GenerationWeights,
    pub execution: HistoryExecution,
    pub tally: FaultInjectionTally,
    pub known_red_hits: Vec<KnownRedAtAFault>,
    pub new_findings: Vec<FaultFinding>,
}

impl FaultInjectionReport {
    /// 给人看的一整块：跑了什么、计数、「已知红」命中、新发现。
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = format!(
            "故障注入：种子 [{}, {})（种子基是这个测试周期写死的那一个）、每段 {} 步、每段注入 {} 次、比重 {}、{}\n",
            self.first_seed,
            self.first_seed + self.seed_count,
            self.operations_per_history,
            self.faults_per_history,
            self.weights.name,
            self.execution.name()
        );
        text.push_str(&self.tally.render());
        let mut known_red_by_form: BTreeMap<usize, Vec<&KnownRedAtAFault>> = BTreeMap::new();
        for hit in &self.known_red_hits {
            known_red_by_form.entry(hit.form).or_default().push(hit);
        }
        for (form, hits) in &known_red_by_form {
            let first = hits.first().expect("这一条至少有一次命中");
            let _ = writeln!(
                text,
                "「已知红」第 {form} 条：{} 次，第一次在种子 {}（{}）",
                hits.len(),
                first.seed.0,
                first.drawn.render()
            );
        }
        for finding in &self.new_findings {
            text.push_str(&finding.render());
        }
        text
    }
}

/// 跑种子 [`first_seed`, `first_seed` + `seed_count`)，分给 `worker_threads.count()` 个工作线程：种子区间切成首尾相接的片，
/// 线程按片号从小到大领片，调用线程收到一片就打一行 `FAULT_INJECTION_PROGRESS`（片号、种子区间、已跑完的片数与段数），
/// 再按片号从小到大并计数。计数按片的次序相加、新发现按种子从小到大留第一个，所以
/// [`FaultInjectionReport::render`] 与线程数、调度次序无关（进度行是按到达次序打的，只报跑到哪了，不带判定）。
///
/// # Panics
/// 有一片领了却没交回。
#[must_use]
pub fn run_fault_injection_campaign(campaign: &FaultInjectionCampaign) -> FaultInjectionReport {
    let FaultInjectionCampaign {
        first_seed,
        seed_count,
        operations_per_history,
        faults_per_history,
        weights,
        execution,
        worker_threads,
    } = *campaign;
    let slices = seed_slices(seed_count, worker_threads.count());
    let spawned_worker_threads = worker_threads.count().min(slices.len().max(1));
    let started = Instant::now();
    println!(
        "FAULT_INJECTION_START seeds=[{first_seed},{}) slices={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={}",
        first_seed + seed_count,
        slices.len(),
        worker_threads.count(),
        worker_threads.source_name()
    );
    let next_slice_index = AtomicUsize::new(0);
    let (finished_slices, merged_slice_count) = std::thread::scope(|scope| {
        let (sender, receiver) = mpsc::channel::<(usize, Vec<HistoryFaultInjection>)>();
        for _ in 0..spawned_worker_threads {
            let sender = sender.clone();
            let slices = &slices;
            let next_slice_index = &next_slice_index;
            scope.spawn(move || loop {
                let slice_index = next_slice_index.fetch_add(1, Ordering::Relaxed);
                let Some(slice) = slices.get(slice_index) else {
                    break;
                };
                let injections: Vec<HistoryFaultInjection> = slice
                    .clone()
                    .map(|offset| {
                        let history = generate_history_with_weights(
                            HistorySeed(first_seed.wrapping_add(offset)),
                            operations_per_history,
                            &weights,
                        );
                        inject_faults_into_history(&history, execution, faults_per_history)
                    })
                    .collect();
                if sender.send((slice_index, injections)).is_err() {
                    break;
                }
            });
        }
        drop(sender);
        let mut waiting_for_earlier_slices: BTreeMap<usize, Vec<HistoryFaultInjection>> =
            BTreeMap::new();
        let mut in_order: Vec<HistoryFaultInjection> = Vec::new();
        let mut next_slice_to_merge = 0usize;
        let mut finished_histories = 0u64;
        for (finished_slice_count, (slice_index, injections)) in receiver.iter().enumerate() {
            let slice = &slices[slice_index];
            finished_histories += slice.end - slice.start;
            println!(
                "FAULT_INJECTION_PROGRESS slice={}/{} seeds=[{},{}) finished_slices={}/{} finished_histories={finished_histories}/{seed_count} elapsed_seconds={:.1}",
                slice_index + 1,
                slices.len(),
                first_seed.wrapping_add(slice.start),
                first_seed.wrapping_add(slice.end),
                finished_slice_count + 1,
                slices.len(),
                started.elapsed().as_secs_f64()
            );
            waiting_for_earlier_slices.insert(slice_index, injections);
            while let Some(ready) = waiting_for_earlier_slices.remove(&next_slice_to_merge) {
                in_order.extend(ready);
                next_slice_to_merge += 1;
            }
        }
        (in_order, next_slice_to_merge)
    });
    assert_eq!(
        merged_slice_count,
        slices.len(),
        "每一片都按次序并进来了：少了说明有工作线程领了片却没交回"
    );
    println!(
        "FAULT_INJECTION_FINISHED seeds=[{first_seed},{}) slices={} worker_threads={spawned_worker_threads} elapsed_seconds={:.1}",
        first_seed + seed_count,
        slices.len(),
        started.elapsed().as_secs_f64()
    );
    let mut tally = FaultInjectionTally::default();
    let mut known_red_hits = Vec::new();
    let mut new_findings: Vec<FaultFinding> = Vec::new();
    let mut signatures_seen: BTreeSet<FailureSignature> = BTreeSet::new();
    for injection in &finished_slices {
        tally.absorb(&injection.tally);
        known_red_hits.extend(injection.known_red_hits.iter().cloned());
        for finding in &injection.new_findings {
            if signatures_seen.insert(finding.signature.clone()) {
                new_findings.push(finding.clone());
            }
        }
    }
    FaultInjectionReport {
        first_seed,
        seed_count,
        operations_per_history,
        faults_per_history,
        weights,
        execution,
        tally,
        known_red_hits,
        new_findings,
    }
}

/// 把 [0, `seed_count`) 切成首尾相接的种子区间：片数取 min(种子数, 4 × 线程数)，各片长度相差至多 1。
fn seed_slices(seed_count: u64, worker_threads: usize) -> Vec<std::ops::Range<u64>> {
    if seed_count == 0 {
        return Vec::new();
    }
    let wanted = u64::try_from(worker_threads.max(1) * SLICES_PER_WORKER_THREAD).expect("片数");
    let slice_count = wanted.min(seed_count);
    (0..slice_count)
        .map(|slice_index| {
            let start = seed_count * slice_index / slice_count;
            let end = seed_count * (slice_index + 1) / slice_count;
            start..end
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crash::SparseBlockDevice;

    /// 用例里的几何：与 E142 装置相同（物理块 512、io_min 512 ⇒ 固定结构槽距 4096），journal 环取默认的 768 MiB。
    const TEST_GEOMETRY: FixedGeometry = FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
    };

    const DEVICE_BYTES: u64 = 4 << 30;
    /// 单元区里的一个落点（第一个单元槽，`UNIT_AREA_START_SLOT * SLOT_BYTES`）：分类成「写单元」。
    const UNIT_AREA_OFFSET: DeviceOffsetInBytes =
        DeviceOffsetInBytes(singlefs_format::UNIT_AREA_START_SLOT * singlefs_format::SLOT_BYTES);

    fn two_devices(
        plan: &SharedFaultPlan,
    ) -> Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> {
        (0..2u32)
            .map(|device_number| {
                let identity = DeviceIdentity(device_number);
                (
                    identity,
                    FaultInjectingBlockDevice::new(
                        identity,
                        SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
                        plan.clone(),
                    ),
                )
            })
            .collect()
    }

    fn write(
        device: &mut FaultInjectingBlockDevice<SparseBlockDevice>,
        offset: u64,
        byte: u8,
    ) -> Result<(), BlockDeviceError> {
        device.write_at(
            DeviceOffsetInBytes(offset),
            &[byte; 512],
            WriteDurability::Plain,
        )
    }

    fn read(device: &FaultInjectingBlockDevice<SparseBlockDevice>, offset: u64) -> [u8; 512] {
        let mut buffer = [0u8; 512];
        device
            .read_at(DeviceOffsetInBytes(offset), &mut buffer)
            .expect("这次读没注入");
        buffer
    }

    /// 计划不开时只数调用：写进去的字节原样读得回来，每块盘的计数与发出的调用一一对上。
    #[test]
    fn an_unarmed_plan_forwards_every_call_and_counts_it() {
        let plan = SharedFaultPlan::unarmed(TEST_GEOMETRY);
        let mut devices = two_devices(&plan);
        write(&mut devices[0].1, 0, 7).expect("写");
        write(&mut devices[1].1, 512, 9).expect("写");
        devices[0].1.barrier().expect("屏障");
        assert_eq!(read(&devices[0].1, 0), [7u8; 512]);
        assert_eq!(
            plan.calls(),
            FaultCallCounts {
                reads: 1,
                writes: 2,
                barriers: 1
            }
        );
        assert_eq!(
            plan.counts_of_device(DeviceIdentity(0)),
            FaultDeviceCounts {
                reads: 1,
                writes: 1,
                written_bytes: 512,
                force_unit_access_writes: 0,
                barriers_forwarded: 1,
                barriers_swallowed: 0,
            }
        );
        assert!(plan.fired().is_empty(), "没开计划就不该注入");
    }

    /// 「整池第 n 次写报错」：前面的照做、第 n 次报错、第 n + 1 次又照做（一次性）；报错那次一个字节都没落盘。
    #[test]
    fn the_nth_write_across_the_pool_fails_once_and_writes_nothing() {
        let plan = SharedFaultPlan::armed(
            TEST_GEOMETRY,
            FaultSchedule::the_nth_call_across_the_pool(InjectedFault::WriteFails, 2),
        );
        let mut devices = two_devices(&plan);
        write(&mut devices[0].1, 0, 1).expect("第 1 次写照做");
        let error = write(&mut devices[1].1, 0, 2).expect_err("第 2 次写报错");
        assert!(matches!(error, BlockDeviceError::InputOutput(_)));
        write(&mut devices[1].1, 512, 3).expect("第 3 次写照做");
        assert_eq!(
            read(&devices[1].1, 0),
            [0u8; 512],
            "报错那次一个字节都没落盘"
        );
        assert_eq!(read(&devices[1].1, 512), [3u8; 512]);
        let fired = plan.fired();
        assert_eq!(fired.len(), 1, "一次性：只注入一次");
        assert_eq!(fired[0].device, DeviceIdentity(1));
        assert_eq!(fired[0].call_ordinal_across_the_pool, 2);
        assert_eq!(
            fired[0].written_structure,
            Some(StepKind::SystemConfigurationSlot),
            "偏移 0 落在系统配置两槽里"
        );
    }

    /// 「第 n 次起每一次都注入」加「只这块盘」加「偏移小于某个数」：三样选择合起来只挑得中该挑的那几次。
    #[test]
    fn every_call_from_the_second_onward_on_one_device_below_an_offset() {
        let plan = SharedFaultPlan::armed(
            TEST_GEOMETRY,
            FaultSchedule {
                fault: InjectedFault::WriteFails,
                device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(1)),
                placement: FaultPlacement::OffsetBelow(8192),
                counting: FaultCounting::PerDevice,
                occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(2),
            },
        );
        let mut devices = two_devices(&plan);
        write(&mut devices[0].1, 0, 1).expect("盘 0 不在选中的盘里");
        write(&mut devices[1].1, UNIT_AREA_OFFSET.0, 2).expect("落点不在 8192 之下");
        write(&mut devices[1].1, 0, 3).expect("盘 1 第 1 次命中：还没到第 2 次");
        write(&mut devices[1].1, 4096, 4).expect_err("盘 1 第 2 次命中：报错");
        write(&mut devices[1].1, 0, 5).expect_err("第 2 次起每一次都报错");
        assert_eq!(plan.fired().len(), 2);
    }

    /// 读回改坏的字节：报成功、缓冲区里那一位被翻掉，盘上那一份一个字节都没变。
    #[test]
    fn a_corrupted_read_flips_one_bit_in_the_buffer_and_leaves_the_device_alone() {
        let plan = SharedFaultPlan::unarmed(TEST_GEOMETRY);
        let mut devices = two_devices(&plan);
        write(&mut devices[0].1, UNIT_AREA_OFFSET.0, 0b0000_0001).expect("写");
        plan.arm(FaultSchedule::the_nth_call_across_the_pool(
            InjectedFault::ReadReturnsCorruptedBytes {
                flipped_bit: FlippedBit {
                    byte_index: 3,
                    bit_index: 2,
                },
            },
            1,
        ));
        let corrupted = read(&devices[0].1, UNIT_AREA_OFFSET.0);
        assert_eq!(corrupted[3], 0b0000_0101, "第 3 个字节的第 2 位被翻掉");
        assert_eq!(corrupted[2], 0b0000_0001, "别的字节不动");
        plan.disarm();
        assert_eq!(
            read(&devices[0].1, UNIT_AREA_OFFSET.0),
            [0b0000_0001u8; 512],
            "盘上那一份没被改坏"
        );
    }

    /// 吞掉一次写：报成功，盘上一个字节都没变（设备说谎那一形，随机注入不抽它）。
    #[test]
    fn a_swallowed_write_reports_success_and_changes_nothing_on_the_device() {
        let plan = SharedFaultPlan::armed(
            TEST_GEOMETRY,
            FaultSchedule::the_nth_call_across_the_pool(InjectedFault::WriteIsSwallowed, 1),
        );
        let mut devices = two_devices(&plan);
        write(&mut devices[0].1, UNIT_AREA_OFFSET.0, 6).expect("报成功");
        assert_eq!(
            read(&devices[0].1, UNIT_AREA_OFFSET.0),
            [0u8; 512],
            "其实一个字节都没落盘"
        );
        assert_eq!(
            plan.counts_of_device(DeviceIdentity(0)).writes,
            0,
            "吞掉的写不算这块盘真发生过的写"
        );
    }

    /// 吞掉一道屏障：报成功，里面那块盘没收到；数进 `barriers_swallowed`。
    #[test]
    fn a_swallowed_barrier_reports_success_and_is_counted_apart() {
        let plan = SharedFaultPlan::armed(
            TEST_GEOMETRY,
            FaultSchedule::the_nth_call_across_the_pool(InjectedFault::BarrierIsSwallowed, 1),
        );
        let mut devices = two_devices(&plan);
        devices[0].1.barrier().expect("报成功");
        devices[0].1.barrier().expect("第二道照发");
        let counts = plan.counts_of_device(DeviceIdentity(0));
        assert_eq!(counts.barriers_swallowed, 1);
        assert_eq!(counts.barriers_forwarded, 1);
    }

    /// 每块盘收到第一道屏障那一刻的计数：屏障本身不算进去，之后的调用也不算。
    #[test]
    fn the_counts_when_the_first_barrier_arrived_are_frozen_at_that_moment() {
        let plan = SharedFaultPlan::unarmed(TEST_GEOMETRY);
        let mut devices = two_devices(&plan);
        assert_eq!(
            plan.counts_when_the_first_barrier_arrived_at(DeviceIdentity(0)),
            None,
            "一道屏障都没收到过"
        );
        write(&mut devices[0].1, UNIT_AREA_OFFSET.0, 1).expect("写");
        devices[0].1.barrier().expect("第一道屏障");
        write(&mut devices[0].1, UNIT_AREA_OFFSET.0 + 512, 2).expect("写");
        devices[0].1.barrier().expect("第二道屏障");
        let frozen = plan
            .counts_when_the_first_barrier_arrived_at(DeviceIdentity(0))
            .expect("收到过屏障");
        assert_eq!(frozen.writes, 1);
        assert_eq!(frozen.barriers_forwarded, 0, "屏障本身不算进去");
        assert_eq!(plan.counts_of_device(DeviceIdentity(0)).writes, 2);
    }

    /// 屏障报错：交回块设备错，里面那块盘没收到。
    #[test]
    fn a_failing_barrier_returns_a_block_device_error() {
        let plan = SharedFaultPlan::armed(
            TEST_GEOMETRY,
            FaultSchedule::every_call_across_the_pool(InjectedFault::BarrierFails),
        );
        let mut devices = two_devices(&plan);
        let error = devices[0].1.barrier().expect_err("屏障报错");
        assert!(matches!(error, BlockDeviceError::InputOutput(_)));
        assert_eq!(
            plan.counts_of_device(DeviceIdentity(0)).barriers_forwarded,
            0
        );
    }

    /// 读报错：交回块设备错，缓冲区不动。
    #[test]
    fn a_failing_read_returns_a_block_device_error_and_leaves_the_buffer_alone() {
        let plan = SharedFaultPlan::armed(
            TEST_GEOMETRY,
            FaultSchedule::the_nth_call_across_the_pool(InjectedFault::ReadFails, 1),
        );
        let devices = two_devices(&plan);
        let mut buffer = [0xabu8; 512];
        let error = devices[0]
            .1
            .read_at(UNIT_AREA_OFFSET, &mut buffer)
            .expect_err("读报错");
        assert!(matches!(error, BlockDeviceError::InputOutput(_)));
        assert_eq!(buffer, [0xabu8; 512], "报错的读不动缓冲区");
    }

    /// 线程数从环境变量取，没设取 `available_parallelism`；设了不是正整数就停，不悄悄退回单线程。
    #[test]
    fn worker_thread_count_comes_from_the_environment_variable_or_available_parallelism() {
        let four = || Ok(std::num::NonZeroUsize::new(4).expect("4"));
        assert_eq!(
            FaultInjectionWorkerThreads::from_the_environment_value(Ok("3".to_string()), four),
            FaultInjectionWorkerThreads::FromTheEnvironmentVariable(3)
        );
        assert_eq!(
            FaultInjectionWorkerThreads::from_the_environment_value(
                Err(std::env::VarError::NotPresent),
                four
            ),
            FaultInjectionWorkerThreads::FromAvailableParallelism(4)
        );
        assert_eq!(
            FaultInjectionWorkerThreads::from_the_environment_value(
                Err(std::env::VarError::NotPresent),
                || Err(std::io::Error::other("报不出来"))
            ),
            FaultInjectionWorkerThreads::AvailableParallelismUnknown
        );
        assert_eq!(
            FaultInjectionWorkerThreads::AvailableParallelismUnknown.count(),
            1
        );
    }

    /// 种子区间切片：片数取 min(种子数, 4 × 线程数)，首尾相接、不重不漏。
    #[test]
    fn seed_slices_cover_every_seed_exactly_once() {
        for (seed_count, threads) in [(0u64, 4usize), (1, 4), (7, 2), (100, 8)] {
            let slices = seed_slices(seed_count, threads);
            let covered: Vec<u64> = slices.iter().flat_map(std::clone::Clone::clone).collect();
            assert_eq!(covered, (0..seed_count).collect::<Vec<u64>>());
        }
    }
}
```

## 三、新测试文件 `crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs` 全文（482 行，工作区未跟踪）

```rust
//! 里程碑「第二个事务」增补 3 第 4 件：故障注入。通用的设备包装、注入点怎么摆、注入之后怎么判在
//! `singlefs_harness::fault_injection`；这里是快档（普通 `cargo test`）、多线程对拍、写死的那几条用例与大档（`#[ignore]`，规模从环境变量取）。
//!
//! 被测的两条性质：注入之后 `singlefs-core` **返回错误而不是 panic**；出错之后**重开恢复到模型允许的版本**。
//! 种子基是这个测试周期写死的那一个（`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`，随机历史与崩溃注入那两个二进制用的是同一个）。

use std::io::Write as _;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::recovery::{
    choose_system_configuration, readable_roots, recover, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
    PublishError,
};
use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::crash_injection::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
use singlefs_harness::fault_injection::{
    inject_faults_into_history, run_fault_injection_campaign, FaultCounting, FaultDeviceSelector,
    FaultInjectingBlockDevice, FaultInjectionCampaign, FaultInjectionReport,
    FaultInjectionWorkerThreads, FaultOccurrence, FaultOutcome, FaultPlacement, FaultSchedule,
    FlippedBit, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::history::{
    generate_history_with_weights, GenerationWeights, HistoryDeviceWidth, HistoryExecution,
    HistoryOperationKind, HistorySeed, PerStepChecker,
};
use singlefs_harness::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 写死的那几条用例里两块内存盘各多大（与各步用例、随机历史的 4 GiB 档相同）。
const IMAGE_BYTES: u64 = 4 << 30;

/// 快档的规模：段数、每段步数、每段注入几次。每次注入都要把这段历史整个重跑一遍，所以规模按「别显著变慢」定。
const FAST_TIER_SEEDS: u64 = 24;
const FAST_TIER_OPERATIONS_PER_HISTORY: usize = 20;
const FAST_TIER_FAULTS_PER_HISTORY: usize = 4;

/// 注入之后这段历史怎么跑：每一步之后跑池级 checker、已知红第 0 条那一形只记不停，两块 4 GiB 的盘。
/// 跑 checker 是这一件的一半——注入之后盘面坏没坏，只有它看得见。
const CHECKED_PAST_THE_RING_TURN: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::RunContinuingPastTheRingTurnForm,
    device_width: HistoryDeviceWidth::FourGibibytes,
};

/// 报告直接写进进程的标准输出，不经 libtest 的捕获：通过时计数照样出现在 `check.sh` 的输出里
/// （`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
fn print_uncaptured(text: &str) {
    let mut standard_output = std::io::stdout();
    let _ = standard_output.write_all(text.as_bytes());
    let _ = standard_output.flush();
}

fn number_from_environment(name: &str, default: u64) -> u64 {
    std::env::var(name).map_or(default, |text| {
        text.parse()
            .unwrap_or_else(|error| panic!("{name}={text} 不是一个非负整数：{error}"))
    })
}

/// 判红时先把种子基与规模说清楚：同一个构建加同一个种子基重放得出来。
fn how_to_replay(report: &FaultInjectionReport) -> String {
    format!(
        "重放：SINGLEFS_FAULT_INJECTION_FIRST_SEED={} SINGLEFS_FAULT_INJECTION_SEEDS={} SINGLEFS_FAULT_INJECTION_OPERATIONS={} SINGLEFS_FAULT_INJECTION_FAULTS={} 跑大档那条 #[ignore] 用例",
        report.first_seed, report.seed_count, report.operations_per_history, report.faults_per_history
    )
}

fn fast_tier_campaign(worker_threads: FaultInjectionWorkerThreads) -> FaultInjectionCampaign {
    FaultInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: FAST_TIER_SEEDS,
        operations_per_history: FAST_TIER_OPERATIONS_PER_HISTORY,
        faults_per_history: FAST_TIER_FAULTS_PER_HISTORY,
        weights: GenerationWeights::BROAD,
        execution: CHECKED_PAST_THE_RING_TURN,
        worker_threads,
    }
}

/// 快档：这个测试周期的种子基起 24 段、每段 20 步、每段摆 4 个注入点。每个注入点重跑一遍这段历史，
/// 判「返回错误而不是 panic」与「重开恢复到模型允许的版本」；「已知红」清单里的形态照记不停，清单外的一条都不许有。
#[test]
fn fault_injection_fast_tier_returns_errors_instead_of_panicking() {
    let started = std::time::Instant::now();
    let report = run_fault_injection_campaign(&fast_tier_campaign(
        FaultInjectionWorkerThreads::from_the_environment(),
    ));
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 故障注入快档 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert_eq!(
        report.tally.faults_that_panicked,
        0,
        "注入之后 core panic 了：这是被测的性质（返回错误而不是 panic）；{}\n{rendered}",
        how_to_replay(&report)
    );
    assert!(
        report.new_findings.is_empty(),
        "注入之后「已知红」清单外的失败；{}\n{rendered}",
        how_to_replay(&report)
    );
    assert_every_fault_injection_path_was_exercised(&report);
}

/// 各条路径真的跑到了：六种注入都摆过、每一类会写盘的操作上都注入过、重开都跑过、四类写落点都注入过。
fn assert_every_fault_injection_path_was_exercised(report: &FaultInjectionReport) {
    let tally = &report.tally;
    let rendered = report.render();
    assert!(
        tally.faults >= FAST_TIER_SEEDS,
        "摆出来的注入点太少（{}）：每段至少该摆出一个\n{rendered}",
        tally.faults
    );
    // 随机注入抽的就是里程碑逐字写的那几种：写、读、刷盘返回块设备错，加上读回改坏的字节
    // （设备说谎那两种不抽，理由写在 `fault_injection::draw_injected_fault` 的文档注释里）。
    for fault in [
        InjectedFault::WriteFails,
        InjectedFault::ReadFails,
        InjectedFault::ReadReturnsCorruptedBytes {
            flipped_bit: FlippedBit {
                byte_index: 0,
                bit_index: 0,
            },
        },
        InjectedFault::BarrierFails,
    ] {
        assert!(
            tally.faults_by_kind.contains_key(fault.name()),
            "一次都没注入过 {}：这一种注入在快档里没跑到\n{rendered}",
            fault.name()
        );
    }
    assert!(
        tally.reopens_reading_a_file > 0 && tally.reopens_without_a_file > 0,
        "重开的两种结局没都见过：读回文件 {}、没有文件 {}\n{rendered}",
        tally.reopens_reading_a_file,
        tally.reopens_without_a_file
    );
    assert!(
        tally
            .faults_by_outcome
            .contains_key(FaultOutcome::SurfacedAsAnError.name()),
        "一次都没有「注入那一步返回了错误」：这一件要的就是这一格\n{rendered}"
    );
    // 会写盘的四类操作：发布（覆盖写）、零单元发布、可写挂载、回退挂载、抬 F。冷启动只读不写，不要求写落点上注入过。
    for operation in [
        HistoryOperationKind::PublishOverwrite,
        HistoryOperationKind::CloseAndMountWritable,
        HistoryOperationKind::CloseAndMountRollback,
        HistoryOperationKind::RaiseRollbackFloor,
    ] {
        assert!(
            tally.faults_on_operation(operation) > 0,
            "{operation:?} 上一次都没注入过（验收要「每个发布步骤与挂载步骤上各注入过至少一次」）\n{rendered}"
        );
    }
    assert!(
        tally.reopens >= tally.faults,
        "每次注入之后都要重开一次：重开 {} 次、注入 {} 次\n{rendered}",
        tally.reopens,
        tally.faults
    );
    assert_eq!(
        tally.reopens_outside_everything_the_model_allows,
        0,
        "重开走到了模型都不允许的版本：{}\n{rendered}",
        how_to_replay(report)
    );
    assert!(
        tally.checker_runs_on_the_image_after_the_fault >= tally.faults,
        "每次注入之后都要对镜像跑一次池级 checker\n{rendered}"
    );
}

/// 大档：规模从环境变量取，后台跑（`cargo test --release -p singlefs-harness --test
/// second_transaction_supplement_three_fault_injection -- --ignored --nocapture`）。
#[test]
#[ignore = "大档：规模从环境变量取，按里程碑「增补 3」后台跑"]
fn fault_injection_large_tier_from_the_environment() {
    let first_seed = number_from_environment(
        "SINGLEFS_FAULT_INJECTION_FIRST_SEED",
        SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
    );
    let seed_count = number_from_environment("SINGLEFS_FAULT_INJECTION_SEEDS", 512);
    let operations_per_history = usize::try_from(number_from_environment(
        "SINGLEFS_FAULT_INJECTION_OPERATIONS",
        30,
    ))
    .expect("步数");
    let faults_per_history = usize::try_from(number_from_environment(
        "SINGLEFS_FAULT_INJECTION_FAULTS",
        6,
    ))
    .expect("注入次数");
    let device_width = match std::env::var("SINGLEFS_FAULT_INJECTION_DEVICE_WIDTH").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("4gib") => HistoryDeviceWidth::FourGibibytes,
        Ok("small") => HistoryDeviceWidth::UnitAreaOf384Slots,
        Ok(other) => panic!("SINGLEFS_FAULT_INJECTION_DEVICE_WIDTH={other} 只认 4gib 与 small"),
        Err(std::env::VarError::NotUnicode(raw)) => {
            panic!("SINGLEFS_FAULT_INJECTION_DEVICE_WIDTH 不是 UTF-8：{raw:?}")
        }
    };
    let started = std::time::Instant::now();
    let report = run_fault_injection_campaign(&FaultInjectionCampaign {
        first_seed,
        seed_count,
        operations_per_history,
        faults_per_history,
        weights: GenerationWeights::BROAD,
        execution: HistoryExecution {
            per_step_checker: PerStepChecker::RunContinuingPastTheRingTurnForm,
            device_width,
        },
        worker_threads: FaultInjectionWorkerThreads::from_the_environment(),
    });
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 故障注入大档 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert_eq!(
        report.tally.faults_that_panicked,
        0,
        "注入之后 core panic 了；{}\n{rendered}",
        how_to_replay(&report)
    );
    assert!(
        report.new_findings.is_empty(),
        "注入之后「已知红」清单外的失败；{}\n{rendered}",
        how_to_replay(&report)
    );
}

/// 一段写死的历史上把注入点摆满：同一段历史注入 12 次，逐次判「返回错误而不是 panic」。
#[test]
fn one_fixed_history_injects_twelve_faults_and_none_of_them_panics() {
    let history = generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        24,
        &GenerationWeights::BROAD,
    );
    let injection = inject_faults_into_history(&history, CHECKED_PAST_THE_RING_TURN, 12);
    let rendered = injection.tally.render();
    print_uncaptured(&format!("── 故障注入：一段写死的历史 ──\n{rendered}"));
    assert!(
        injection.drawn_faults.len() >= 8,
        "12 次抽里摆出来的注入点太少（{}）",
        injection.drawn_faults.len()
    );
    assert_eq!(
        injection.tally.faults_that_panicked, 0,
        "注入之后 core panic 了\n{rendered}"
    );
    assert!(
        injection.new_findings.is_empty(),
        "「已知红」清单外的失败\n{rendered}"
    );
}

/// 线程数换了，报告逐字相同：计数按片的次序相加、新发现按种子从小到大留第一个
/// （`implementation-workflow.md`「测试与崩溃检测优先多线程」的「合并要确定」）。
#[test]
fn the_report_is_the_same_text_with_one_worker_thread_and_with_four() {
    let campaign = |threads| FaultInjectionCampaign {
        seed_count: 6,
        operations_per_history: 12,
        faults_per_history: 2,
        ..fast_tier_campaign(FaultInjectionWorkerThreads::GivenByCaller(threads))
    };
    let one = run_fault_injection_campaign(&campaign(1));
    let four = run_fault_injection_campaign(&campaign(4));
    assert_eq!(
        one.render(),
        four.render(),
        "线程数变了报告就变了：并到一起的次序不确定"
    );
}

/// 增补 2 收口表第 40 行（C381（根已落盘之后发布失败，分配器仍退回））的判别力：发布在最后一步（系统配置槽）失败时根已 FUA 落盘、
/// 分配器照样退回，同一个写入口拿同一个上一版再发一次，就把那条根指着的单元原地盖掉——那一次再断电（这里用注入的根槽写错代替断电），
/// 盘上留下的就是「最新的那条根指着一份内容已经换掉的单元」。
///
/// 这条用例钉的是**今天这个缺陷的现形**：池级 checker 判红、冷启动走不到那条根。
/// C381 一旦按哪条候选改掉（发布失败之后那条根不再看得见，或者分配器不再退回），这条用例会转绿在别处、这里的断言会红——
/// 那时候连同「已知红」的账一起改，不许把断言改松（`show-me-test.md`「改代码还是改断言，先想清楚是哪一种」）。
#[test]
fn a_publish_that_fails_on_the_system_configuration_slot_leaves_a_root_whose_units_the_next_publish_overwrites(
) {
    let parameters = e142_parameters(512, 512);
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
    };
    let stream = SharedStream::retaining_contents();
    let plan = SharedFaultPlan::unarmed(geometry);
    let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<_>)> = (0..2u32)
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                FaultInjectingBlockDevice::new(
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                    plan.clone(),
                ),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号");
    let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    let first = publish_first_file(
        &mut writer,
        &mut allocator,
        &genesis.root,
        FirstFile {
            content: &first_file_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        },
        instance,
        &warmed.last_record_bytes,
    )
    .expect("第一个事务");
    assert_eq!(first.root.checkpoint_txg, CheckpointTxg(3));

    // 一、发布 B：系统配置槽那两次写全报错。根槽 FUA 写在它们之前，已经落盘。
    plan.arm(FaultSchedule {
        fault: InjectedFault::WriteFails,
        device: FaultDeviceSelector::EveryDevice,
        placement: FaultPlacement::OffsetBelow(
            SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
                * u64::from(parameters.geometry.fixed_structure_slot_spacing),
        ),
        counting: FaultCounting::AcrossThePool,
        occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
    });
    let refused = publish_overwrite(
        &mut writer,
        &mut allocator,
        &first,
        FirstFile {
            content: &second_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect_err("系统配置槽写报错，这次发布必须失败");
    assert!(
        matches!(refused, PublishError::BlockDevice(_)),
        "报的应当是块设备错：{refused:?}"
    );
    let fired = plan.fired();
    assert_eq!(fired.len(), 1, "系统配置槽第一次写就报错、发布到此为止");
    assert_eq!(
        fired[0].written_structure,
        Some(StepKind::SystemConfigurationSlot),
        "注入的那一次要落在系统配置槽上"
    );
    let image_after_the_failed_publish = image_from(&stream);
    let roots_after: Vec<u64> = readable_root_txgs(&image_after_the_failed_publish);
    assert!(
        roots_after.contains(&4),
        "C381 的前提：这次发布报了失败，txg 4 那条根却已经 FUA 落盘、在环里读得出来；读到的是 {roots_after:?}"
    );

    // 二、同一个写入口、同一个上一版再发一次（分配器已经退回，落点与发布 B 完全相同），在根槽 FUA 写上断电。
    plan.arm(FaultSchedule::the_nth_call_across_the_pool(
        InjectedFault::WriteFails,
        19,
    ));
    let refused_again = publish_overwrite(
        &mut writer,
        &mut allocator,
        &first,
        FirstFile {
            content: &third_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
        },
        instance,
    )
    .expect_err("根槽 FUA 写报错，这次发布也必须失败");
    assert!(
        matches!(refused_again, PublishError::BlockDevice(_)),
        "报的应当是块设备错：{refused_again:?}"
    );
    let fired_again = plan.fired();
    assert_eq!(
        fired_again.last().map(|last| last.written_structure),
        Some(Some(StepKind::RootRecordFua)),
        "一次发布的第 19 次写是根槽 FUA 写（字节表零 t1..t8 十六次单元写 + 两次 journal 记录 + 根槽）：{fired_again:?}"
    );
    drop(writer);

    // 三、现形：环里最新那条根还是发布 B 的 txg 4，它指着的单元已经被发布 C 原地盖掉。
    let image = image_from(&stream);
    let violations: Vec<&'static str> = check_pool_image(&image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(_) => Some(invariant),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect();
    assert!(
        violations.contains(&"I-7.4") && violations.contains(&"I-7.2"),
        "收口表第 40 行那一格的现形：最新根引用的单元已被复用（I-7.4）、最新根走不完（I-7.2）。判红的是 {violations:?}"
    );
    let recovery = recover(&image, JournalPolicy::Consult);
    assert!(
        matches!(recovery.outcome, RecoveryOutcome::Failed { .. }),
        "重开走不到任何一条根（收口表第 40 行写的 `Recovery(UnitUnreadable)`）：{:?}",
        recovery.outcome
    );
}

/// 发布 B 的内容（4100 字节，与虚机二进制 `second-transaction` 模式相同）。
fn second_content() -> Vec<u8> {
    (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 发布 C 的内容：与发布 B 不同，单元字节才会真的变。
fn third_content() -> Vec<u8> {
    (0..4100usize)
        .map(|index| u8::try_from((index * 11 + 5) % 251).expect("小于 256"))
        .collect()
}

/// 录制流里真正落到盘上的那些写重建出来的镜像。
fn image_from(stream: &SharedStream) -> MemoryPool {
    let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    image.apply(&stream.retained_operations());
    image
}

/// 环里自证过的根的 txg，从小到大。
fn readable_root_txgs(image: &MemoryPool) -> Vec<u64> {
    let system_configuration = choose_system_configuration(image).expect("择系统配置");
    let mut txgs: Vec<u64> = readable_roots(
        image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .map(|root| root.checkpoint_txg.0)
    .collect();
    txgs.sort_unstable();
    txgs
}
```
