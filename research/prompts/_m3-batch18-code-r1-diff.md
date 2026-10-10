# 删除批第 18 批合入的改动原样（2026-10-05）

## 实现员 x：crates.patch（/tmp/claude-1000/m3-batch18/x/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs b/crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs
new file mode 100644
index 0000000..42b5696
--- /dev/null
+++ b/crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs
@@ -0,0 +1,274 @@
+//! 里程碑三·6（`.claude/kb/milestone/03-third-txn.md` 第三节「三·6」）干跑的 `crates/` 侧只读导出：删除写出的字节。
+//!
+//! 在两块内存盘上依次跑 `singlefs_harness::scenario::run_deletion_history` 的两段历史（H1、H2：E142 那套参数、
+//! 三个数据单元的第一个文件、只供测试的批大小 1），把 unlink 之后的窗口（意图那次发布之前录制流的长度起，到这段历史结束为止）
+//! 里**每一次写**按（步号、设备、字节偏移、字节长度、种类、sha256、整段十六进制）原样打出来，屏障各打一行，
+//! 每一段（一次发布，或不属于任何一次发布的几次写）前后各打一行边界：发布的边界带 txg、哪一种发布、这次重写的角色。
+//! **不带任何位置寻址的布局知识**：不按偏移归类；段的边界与种类来自场景函数（直接调的发布按调用前后录制流的长度，
+//! 卸载与挂载里的几次按 core 交回的每次发布写账切开），角色名来自 core 交回的 `TransactionOutput::rewritten`。
+//! 步号从窗口起点数，窗口起点在录制流里的下标打在 `name=before_window_summary` 的 `operations=`。
+//!
+//! 只驱动、只观测：不改 `singlefs-core`、`singlefs-checker`、`singlefs-format`。
+//!
+//!   deletion_write_dump
+//!
+//! 不收参数、不读环境、不碰真设备、不写任何文件。退出码：0 = 正常；1 = 命令行给了参数。
+
+use singlefs_core::address::DeviceIdentity;
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_format::TEST_IMAGE_DEFAULT_BYTES;
+use singlefs_harness::hexadecimal::hexadecimal_text;
+use singlefs_harness::memory_pool::SparseBlockDevice;
+use singlefs_harness::scenario::{
+    deletion_history_batch_size, e142_parameters, run_deletion_history,
+    three_data_unit_file_content, DeletionHistory, DeletionHistorySegment,
+    E142_FILESYSTEM_IDENTIFIER, FIXED_WRITE_TIME_SECONDS,
+    SECONDS_FROM_THE_FIRST_FILE_TO_THE_SEQUENTIAL_WRITE_IN_THE_DELETION_HISTORIES,
+};
+use singlefs_harness::sha256::sha256_hexadecimal;
+use singlefs_harness::{
+    RecordedOperationKind, RecordingBlockDevice, RetainedOperation, SharedStream,
+};
+
+/// E142 那套几何：两块同构盘，`physical_block_size` = io_min = 512（与 `e142_new_pool_file_creation_write_dump` 同一组值）。
+const DEVICE_COUNT: u32 = 2;
+const PHYSICAL_BLOCK_SIZE_IN_BYTES: u32 = 512;
+const MINIMUM_INPUT_OUTPUT_BYTES: u32 = 512;
+
+struct Emitter {
+    emitted: u64,
+}
+
+impl Emitter {
+    fn emit(&mut self, body: &str) {
+        self.emitted += 1;
+        println!("E7RESULT {body}");
+    }
+    fn finish(&mut self) {
+        self.emitted += 1;
+        println!("E7RESULT name=done emitted={}", self.emitted);
+    }
+}
+
+fn kind_name(kind: RecordedOperationKind) -> &'static str {
+    match kind {
+        RecordedOperationKind::Write => "write",
+        RecordedOperationKind::WriteForceUnitAccess => "write_fua",
+        RecordedOperationKind::WriteZeroes => "write_zeroes",
+        RecordedOperationKind::Barrier => "barrier",
+    }
+}
+
+/// 屏障没有字节内容；写清零的内容按长度铺 0（整段全 0 由偏移与长度全定）。
+fn operation_bytes(retained: &RetainedOperation) -> Option<Vec<u8>> {
+    match retained.operation.kind {
+        RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
+            retained.contents.clone()
+        }
+        RecordedOperationKind::WriteZeroes => {
+            Some(vec![
+                0u8;
+                usize::try_from(retained.operation.length)
+                    .expect("窗口里的写清零长度装得进 usize")
+            ])
+        }
+        RecordedOperationKind::Barrier => None,
+    }
+}
+
+/// 一段前后那两行边界的正文（不带 `edge=` 与 `step=`）。
+fn segment_boundary_fields(segment: &DeletionHistorySegment) -> (String, String) {
+    match segment {
+        DeletionHistorySegment::Publish { kind, version, .. } => (
+            "publish_boundary".to_string(),
+            format!(
+                "publish={} txg={} rewritten_units={} rewritten_roles={} data_units_after={} deleted_inodes_after={} inode_record_after={}",
+                kind.name(),
+                version.root.checkpoint_txg.0,
+                version.rewritten.len(),
+                version
+                    .rewritten
+                    .iter()
+                    .map(|role| role.tag())
+                    .collect::<Vec<_>>()
+                    .join(","),
+                version.data_pointers.len(),
+                version.deleted_inodes.len(),
+                version.inode_record.is_some(),
+            ),
+        ),
+        DeletionHistorySegment::WritesOutsideAnyPublish { kind, .. } => (
+            "writes_outside_any_publish_boundary".to_string(),
+            format!("segment={}", kind.name()),
+        ),
+    }
+}
+
+/// 一段历史：跑、打窗口里的每一步与每一段的边界。
+fn dump_one_history(history: DeletionHistory, emitter: &mut Emitter) {
+    let parameters = e142_parameters(PHYSICAL_BLOCK_SIZE_IN_BYTES, MINIMUM_INPUT_OUTPUT_BYTES);
+    let stream = SharedStream::retaining_contents();
+    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0
+        ..DEVICE_COUNT)
+        .map(|device_number| {
+            let identity = DeviceIdentity(device_number);
+            (
+                identity,
+                RecordingBlockDevice::with_shared_stream(
+                    identity,
+                    SparseBlockDevice::new(
+                        TEST_IMAGE_DEFAULT_BYTES,
+                        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_SIZE_IN_BYTES),
+                    ),
+                    stream.clone(),
+                ),
+            )
+        })
+        .collect();
+    let run = run_deletion_history(history, &parameters, &mut devices, &stream)
+        .unwrap_or_else(|failure| panic!("{} 在内存盘上没走完：{failure:?}", history.name()));
+    let history_name = history.name();
+    let content_bytes = three_data_unit_file_content().len();
+    emitter.emit(&format!(
+        "name=impl_config history={history_name} devices={DEVICE_COUNT} image_bytes={TEST_IMAGE_DEFAULT_BYTES} physical_block_size={PHYSICAL_BLOCK_SIZE_IN_BYTES} minimum_input_output_bytes={MINIMUM_INPUT_OUTPUT_BYTES} region_device_0={} region_device_1={} region_device_2={} slots_per_region={} journal_ring_bytes={} first_file_bytes_before_the_sequential_write={} file_bytes={content_bytes} batch_branch={} batch_data_units={} write_time_seconds={FIXED_WRITE_TIME_SECONDS} sequential_write_time_seconds={} filesystem_identifier={} mkfs_operations={}",
+        parameters.region_devices[0].0,
+        parameters.region_devices[1].0,
+        parameters.region_devices[2].0,
+        parameters.geometry.root_ring_slots_per_region.count(),
+        parameters.geometry.journal_ring_bytes,
+        singlefs_harness::scenario::FIRST_FILE_BYTES,
+        deletion_history_batch_size().name(),
+        deletion_history_batch_size().data_units_per_batch(),
+        FIXED_WRITE_TIME_SECONDS
+            + SECONDS_FROM_THE_FIRST_FILE_TO_THE_SEQUENTIAL_WRITE_IN_THE_DELETION_HISTORIES,
+        hexadecimal_text(&E142_FILESYSTEM_IDENTIFIER),
+        run.mkfs_operation_count,
+    ));
+
+    let all_operations = stream.retained_operations();
+    let window_start = run.operations_before_the_unlink;
+    let before_window = &all_operations[..window_start];
+    emitter.emit(&format!(
+        "name=before_window_summary history={history_name} operations={} writes={}",
+        before_window.len(),
+        before_window
+            .iter()
+            .filter(|retained| retained.operation.kind != RecordedOperationKind::Barrier)
+            .count(),
+    ));
+
+    let mut write_count = 0u64;
+    let mut barrier_count = 0u64;
+    let mut publish_count = 0u64;
+    let mut next_step_expected = window_start;
+    for segment in &run.segments {
+        let operations = segment.operations();
+        assert_eq!(
+            operations.start, next_step_expected,
+            "场景函数交回的各段首尾相接，从窗口起点铺起"
+        );
+        next_step_expected = operations.end;
+        let (line_name, fields) = segment_boundary_fields(segment);
+        if matches!(segment, DeletionHistorySegment::Publish { .. }) {
+            publish_count += 1;
+        }
+        emitter.emit(&format!(
+            "name={line_name} history={history_name} edge=before step={} {fields}",
+            operations.start - window_start
+        ));
+        for absolute_step in operations.clone() {
+            let retained = &all_operations[absolute_step];
+            let operation = &retained.operation;
+            let step = absolute_step - window_start;
+            if operation.kind == RecordedOperationKind::Barrier {
+                barrier_count += 1;
+                emitter.emit(&format!(
+                    "name=window_barrier history={history_name} step={step} device={}",
+                    operation.device.0
+                ));
+                continue;
+            }
+            write_count += 1;
+            let bytes = operation_bytes(retained)
+                .expect("write / write_fua / write_zeroes 都能重建出完整字节");
+            emitter.emit(&format!(
+                "name=device_region_bytes history={history_name} step={step} device={} offset={} length={} kind={} sha256={} hexadecimal={}",
+                operation.device.0,
+                operation.offset.0,
+                operation.length,
+                kind_name(operation.kind),
+                sha256_hexadecimal(&bytes),
+                hexadecimal_text(&bytes),
+            ));
+        }
+        emitter.emit(&format!(
+            "name={line_name} history={history_name} edge=after step={} {fields}",
+            operations.end - window_start
+        ));
+    }
+    assert_eq!(
+        next_step_expected,
+        all_operations.len(),
+        "场景函数交回的各段铺到录制流的尾上"
+    );
+    emitter.emit(&format!(
+        "name=device_region_bytes_summary history={history_name} window_operations={} writes={write_count} barriers={barrier_count} segments={} publishes={publish_count}",
+        all_operations.len() - window_start,
+        run.segments.len(),
+    ));
+}
+
+fn main() {
+    let arguments: Vec<String> = std::env::args().skip(1).collect();
+    if !arguments.is_empty() {
+        eprintln!("deletion_write_dump 不收参数，收到 {arguments:?}");
+        std::process::exit(1);
+    }
+    let mut emitter = Emitter { emitted: 0 };
+    for history in DeletionHistory::EVERY_HISTORY {
+        dump_one_history(history, &mut emitter);
+    }
+    emitter.finish();
+}
+
+#[cfg(test)]
+mod tests {
+    use super::{kind_name, operation_bytes};
+    use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
+    use singlefs_harness::{RecordedOperation, RecordedOperationKind, RetainedOperation};
+
+    #[test]
+    fn kind_name_covers_every_recorded_operation_kind_without_a_wildcard_arm() {
+        assert_eq!(kind_name(RecordedOperationKind::Write), "write");
+        assert_eq!(
+            kind_name(RecordedOperationKind::WriteForceUnitAccess),
+            "write_fua"
+        );
+        assert_eq!(
+            kind_name(RecordedOperationKind::WriteZeroes),
+            "write_zeroes"
+        );
+        assert_eq!(kind_name(RecordedOperationKind::Barrier), "barrier");
+    }
+
+    #[test]
+    fn operation_bytes_rebuilds_an_all_zero_buffer_for_write_zeroes_and_none_for_a_barrier() {
+        let operation = |kind, length| RecordedOperation {
+            device: DeviceIdentity(0),
+            kind,
+            offset: DeviceOffsetInBytes(0),
+            length,
+            content_hash: 0,
+        };
+        let write_zeroes = RetainedOperation {
+            operation: operation(RecordedOperationKind::WriteZeroes, 4),
+            contents: None,
+        };
+        let barrier = RetainedOperation {
+            operation: operation(RecordedOperationKind::Barrier, 0),
+            contents: None,
+        };
+        assert_eq!(operation_bytes(&write_zeroes), Some(vec![0, 0, 0, 0]));
+        assert_eq!(operation_bytes(&barrier), None);
+    }
+}
diff --git a/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs b/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs
index caf847b..b6c7f38 100644
--- a/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs
+++ b/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs
@@ -11,8 +11,12 @@
 //! 每个常量另加一条断言与 `crate::scenario` 的公开常量、`singlefs_format::TEST_IMAGE_DEFAULT_BYTES` 回比。
 //!
 //!   e142_new_pool_file_creation_write_dump
+//!   e142_new_pool_file_creation_write_dump --device-bytes <每盘字节数>
 //!
-//! 不收参数、不读环境、不碰真设备、不写任何文件。退出码：0 = 正常；1 = 命令行给了参数。
+//! 不带参数时两块盘各 `TEST_IMAGE_DEFAULT_BYTES`（4 GiB），输出与这个 bin 只收零个参数时逐字节相同；`--device-bytes` 换每盘字节数
+//! （E142 第 22 次第二段的 P8G：两块 8 GiB 盘，`research/prompts/e142-r22-prereg.md` 读法 R67），别的参数照旧，
+//! `make_filesystem` 收不收由它自己判（环不超过盘的四分之一）。`name=impl_config` 的 `image_bytes=` 打的是这一次的每盘字节数。
+//! 不读环境、不碰真设备、不写任何文件。退出码：0 = 正常；1 = 命令行参数不是上面两种之一。
 
 use singlefs_core::address::DeviceIdentity;
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
@@ -82,12 +86,32 @@ fn operation_bytes(retained: &RetainedOperation) -> Option<Vec<u8>> {
     }
 }
 
+/// 换每盘字节数的那个选项。
+const DEVICE_BYTES_OPTION: &str = "--device-bytes";
+
+/// 命令行给的每盘字节数：零个参数是 `TEST_IMAGE_DEFAULT_BYTES`，`--device-bytes <十进制字节数>` 是给的那个数；
+/// 别的写法（多给、少给、数解不开）交回给人看的一句。盘够不够大不在这里判，交给 `make_filesystem`。
+fn device_bytes_from_the_command_line(arguments: &[String]) -> Result<u64, String> {
+    match arguments {
+        [] => Ok(TEST_IMAGE_DEFAULT_BYTES),
+        [option, value] if option == DEVICE_BYTES_OPTION => value.parse::<u64>().map_err(|error| {
+            format!("{DEVICE_BYTES_OPTION} 后面要一个十进制字节数，收到 {value:?}：{error}")
+        }),
+        other => Err(format!(
+            "只收零个参数或 {DEVICE_BYTES_OPTION} <每盘字节数>，收到 {other:?}"
+        )),
+    }
+}
+
 fn main() {
     let arguments: Vec<String> = std::env::args().skip(1).collect();
-    if !arguments.is_empty() {
-        eprintln!("e142_new_pool_file_creation_write_dump 不收参数，收到 {arguments:?}");
-        std::process::exit(1);
-    }
+    let device_bytes = match device_bytes_from_the_command_line(&arguments) {
+        Ok(device_bytes) => device_bytes,
+        Err(refusal) => {
+            eprintln!("e142_new_pool_file_creation_write_dump {refusal}");
+            std::process::exit(1);
+        }
+    };
 
     // 绝对值断言（门禁 80 号）：4 GiB = 4 × 1024 × 1024 × 1024，独立算出的字面量，不从代码读回来。
     // 两边在编译期就定了，clippy 建议挪进 const 块（否则判「assertions_on_constants」）。
@@ -117,7 +141,7 @@ fn main() {
                 RecordingBlockDevice::with_shared_stream(
                     identity,
                     SparseBlockDevice::new(
-                        TEST_IMAGE_DEFAULT_BYTES,
+                        device_bytes,
                         PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_SIZE_IN_BYTES),
                     ),
                     stream.clone(),
@@ -151,7 +175,7 @@ fn main() {
 
     let mut emitter = Emitter { emitted: 0 };
     emitter.emit(&format!(
-        "name=impl_config devices={DEVICE_COUNT} image_bytes={TEST_IMAGE_DEFAULT_BYTES} physical_block_size={PHYSICAL_BLOCK_SIZE_IN_BYTES} minimum_input_output_bytes={MINIMUM_INPUT_OUTPUT_BYTES} region_device_0={} region_device_1={} region_device_2={} slots_per_region={} journal_ring_bytes={} file_bytes={FIRST_FILE_BYTES} write_time_seconds={FIXED_WRITE_TIME_SECONDS} filesystem_identifier={} checkpoint_txg={NEW_POOL_FILE_CREATION_TXG} mkfs_operations={}",
+        "name=impl_config devices={DEVICE_COUNT} image_bytes={device_bytes} physical_block_size={PHYSICAL_BLOCK_SIZE_IN_BYTES} minimum_input_output_bytes={MINIMUM_INPUT_OUTPUT_BYTES} region_device_0={} region_device_1={} region_device_2={} slots_per_region={} journal_ring_bytes={} file_bytes={FIRST_FILE_BYTES} write_time_seconds={FIXED_WRITE_TIME_SECONDS} filesystem_identifier={} checkpoint_txg={NEW_POOL_FILE_CREATION_TXG} mkfs_operations={}",
         parameters.region_devices[0].0,
         parameters.region_devices[1].0,
         parameters.region_devices[2].0,
@@ -226,10 +250,47 @@ fn main() {
 
 #[cfg(test)]
 mod tests {
-    use super::{kind_name, operation_bytes};
+    use super::{device_bytes_from_the_command_line, kind_name, operation_bytes};
     use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
     use singlefs_harness::{RecordedOperation, RecordedOperationKind, RetainedOperation};
 
+    fn arguments(words: &[&str]) -> Vec<String> {
+        words.iter().map(|word| (*word).to_string()).collect()
+    }
+
+    #[test]
+    fn no_argument_keeps_the_four_gibibyte_devices_of_the_first_registration() {
+        assert_eq!(
+            device_bytes_from_the_command_line(&arguments(&[])),
+            Ok(4_294_967_296),
+            "不带参数是跑前登记第一节那两块 4 GiB 盘"
+        );
+    }
+
+    #[test]
+    fn the_device_bytes_option_takes_the_eight_gibibyte_devices_of_the_second_geometry() {
+        assert_eq!(
+            device_bytes_from_the_command_line(&arguments(&["--device-bytes", "8589934592"])),
+            Ok(8_589_934_592),
+            "P8G 每盘 8 589 934 592 字节（第 22 次登记读法 R67）"
+        );
+    }
+
+    #[test]
+    fn an_unparsable_value_an_unknown_option_and_a_missing_value_are_each_refused() {
+        for refused in [
+            arguments(&["--device-bytes", "8GiB"]),
+            arguments(&["--image-bytes", "8589934592"]),
+            arguments(&["--device-bytes"]),
+            arguments(&["--device-bytes", "8589934592", "extra"]),
+        ] {
+            assert!(
+                device_bytes_from_the_command_line(&refused).is_err(),
+                "{refused:?} 要拒"
+            );
+        }
+    }
+
     #[test]
     fn kind_name_covers_every_recorded_operation_kind_without_a_wildcard_arm() {
         assert_eq!(kind_name(RecordedOperationKind::Write), "write");
diff --git a/crates/singlefs-harness/src/scenario.rs b/crates/singlefs-harness/src/scenario.rs
index c8fae25..bb3ac26 100644
--- a/crates/singlefs-harness/src/scenario.rs
+++ b/crates/singlefs-harness/src/scenario.rs
@@ -1,21 +1,35 @@
 //! 新池新建文件的整条路（mkfs → 取号 → 暖机 → 新池新建文件），参数照 E142（新池新建文件的干跑） 装置取：
 //! 虚机里的真设备、宿主上的内存盘都跑这一份，同参数同字节（mkfs 与发布都不取系统时钟与随机数）。
 
-use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
+use std::ops::Range;
+
+use singlefs_core::address::{DeviceIdentity, InodeNumber, InstanceGeneration};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::allocator::PoolAllocator;
 use singlefs_core::block_device::BlockDevice;
+use singlefs_core::instance_table_sweep::InstanceTableSweepAtTheEndOfTheMount;
 use singlefs_core::make_filesystem::{
     allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
 };
+use singlefs_core::mount::{
+    mount_writable_with_test_only_switches_and_deletion_switches, unmount,
+    DeletionQueueContinuationOutcome, ShadowLedger, Unmounted,
+};
 use singlefs_core::root_ring::RootRingSlotsPerRegion;
 use singlefs_core::system_configuration::SystemImmutableSizes;
 use singlefs_core::transaction::{
-    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, TransactionOutput,
-    WarmUpOutput,
+    acquire_instance, publish_first_file, publish_sequential_write, publish_unlink_intent,
+    publish_unlink_step, warm_up, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
+    WarmUpOutput, FIRST_INODE_NUMBER,
+};
+use singlefs_core::unit::data_unit_payload_capacity;
+use singlefs_core::unlink::{
+    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
 };
 use singlefs_core::write_accounting::WritesByStructureKind;
 use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
 
-use crate::SharedStream;
+use crate::{RecordedOperation, RecordedOperationKind, RecordedPublishEntry, SharedStream};
 
 /// E142 装置里的固定 fsid（`FIXED_FSID`）。
 pub const E142_FILESYSTEM_IDENTIFIER: [u8; 16] = [
@@ -134,8 +148,22 @@ pub fn run_new_pool_file_creation<
     parameters: &MakeFilesystemParameters,
     devices: &mut [(DeviceIdentity, Device)],
     stream: &SharedStream,
-    mut at_point: AtPoint,
+    at_point: AtPoint,
 ) -> Result<NewPoolFileCreationRun, NewPoolFileCreationPathFailure> {
+    run_new_pool_file_creation_keeping_the_allocator(parameters, devices, stream, at_point)
+        .map(|(run, _allocator)| run)
+}
+
+/// 同 [`run_new_pool_file_creation`]，连同新池新建文件之后的分配器一起交回：同一个进程接着发布（删除那两段历史）要接着用它。
+fn run_new_pool_file_creation_keeping_the_allocator<
+    Device: BlockDevice,
+    AtPoint: FnMut(ScenarioPoint, &mut [(DeviceIdentity, Device)]),
+>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut [(DeviceIdentity, Device)],
+    stream: &SharedStream,
+    mut at_point: AtPoint,
+) -> Result<(NewPoolFileCreationRun, PoolAllocator), NewPoolFileCreationPathFailure> {
     let genesis = make_filesystem(parameters, devices).map_err(|error| {
         NewPoolFileCreationPathFailure::before_any_publish(
             NewPoolFileCreationPathStep::MakeFilesystem,
@@ -185,9 +213,618 @@ pub fn run_new_pool_file_creation<
         writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),
         writes_of_persisted_publishes: Vec::new(),
     })?;
-    Ok(NewPoolFileCreationRun {
-        mkfs_operation_count,
-        warm_up: warm_up_output,
-        output,
+    Ok((
+        NewPoolFileCreationRun {
+            mkfs_operation_count,
+            warm_up: warm_up_output,
+            output,
+        },
+        allocator,
+    ))
+}
+
+/// 新池新建文件那个实例：删除那两段历史里在这个进程里发的每一次（顺序写、意图、H1 的各批）都在它里面发。
+const INSTANCE_OF_THE_NEW_POOL_FILE_CREATION: InstanceGeneration = InstanceGeneration(1);
+
+/// 删除那两段历史删的是第一个文件。
+const FILE_DELETED_IN_THE_DELETION_HISTORIES: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);
+
+/// 删除那两段历史里顺序写比新池新建文件晚几秒（与 harness 档 unlink 那几条用例的 `OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE` 同一个数）。
+pub const SECONDS_FROM_THE_FIRST_FILE_TO_THE_SEQUENTIAL_WRITE_IN_THE_DELETION_HISTORIES: u64 = 60;
+
+/// 删除那两段历史里第一个文件顺序写成的内容：两个满净荷加半个净荷，三个数据单元（与 harness 档 unlink 那几条用例的
+/// `pool_with_a_three_unit_file` 同一份字节）。
+#[must_use]
+pub fn three_data_unit_file_content() -> Vec<u8> {
+    let payload_capacity = data_unit_payload_capacity();
+    (0..2 * payload_capacity + payload_capacity / 2)
+        .map(|index| u8::try_from((index * 13 + 5) % 251).expect("对 251 取余小于 256"))
+        .collect()
+}
+
+/// 删除那两段历史的批大小：只供测试的批大小旋钮压到每批一个数据单元（`.claude/rules/fs-design.md` 五条硬要求第 2 条），
+/// 三个数据单元的文件走「留 2、留 1、最后一批」三批。
+#[must_use]
+pub fn deletion_history_batch_size() -> UnlinkBatchSize {
+    UnlinkBatchSize::CappedForTests(
+        UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=产品批大小里"),
+    )
+}
+
+/// H2 那次可写挂载装的删除开关：批大小同 [`deletion_history_batch_size`]，三个容量照产品路径。
+#[must_use]
+pub fn deletion_history_switches() -> DeletionSwitches {
+    DeletionSwitches {
+        batch_size: deletion_history_batch_size(),
+        tombstone_container_capacity: DeletionSwitches::PRODUCT.tombstone_container_capacity,
+        lineage_container_capacity: DeletionSwitches::PRODUCT.lineage_container_capacity,
+        container_tree_node_entries: DeletionSwitches::PRODUCT.container_tree_node_entries,
+    }
+}
+
+/// 三·6 干跑导出的两段删除历史（调查报告第 4.3 节表里的 H1、H2）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum DeletionHistory {
+    /// H1：意图与每一批都在新池新建文件那个进程里发，直到队列空。
+    IntentAndEveryBatchInProcess,
+    /// H2：意图在这个进程里发，正常卸载，同一组盘重开可写挂载，挂载续做在写行与暖机之后把队列走到空。
+    IntentThenUnmountThenMountContinuation,
+}
+
+impl DeletionHistory {
+    pub const EVERY_HISTORY: [DeletionHistory; 2] = [
+        DeletionHistory::IntentAndEveryBatchInProcess,
+        DeletionHistory::IntentThenUnmountThenMountContinuation,
+    ];
+
+    /// 结果行里的名字，与调查报告那张表的行名相同。
+    #[must_use]
+    pub const fn name(self) -> &'static str {
+        match self {
+            DeletionHistory::IntentAndEveryBatchInProcess => "h1",
+            DeletionHistory::IntentThenUnmountThenMountContinuation => "h2",
+        }
+    }
+}
+
+/// 删除历史里的一次发布是哪一种。删除类是前两种与最后一种，其余三种是这段历史里别的发布。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum DeletionHistoryPublishKind {
+    /// unlink 的意图（`publish_unlink_intent`）。
+    UnlinkIntent,
+    /// 这个进程里发的一批（`publish_unlink_step`），最后一批也是它。
+    UnlinkBatchInProcess,
+    /// 正常卸载那一串带卸载记号的空发布。
+    UnmountFloorRaise,
+    /// 可写挂载写行那次发布。
+    MountInstanceRow,
+    /// 可写挂载写行之后的暖机空发布。
+    MountWarmUp,
+    /// 挂载续做发的一批，最后一批也是它。
+    UnlinkBatchOfTheMountContinuation,
+}
+
+impl DeletionHistoryPublishKind {
+    /// 结果行里的名字。
+    #[must_use]
+    pub const fn name(self) -> &'static str {
+        match self {
+            DeletionHistoryPublishKind::UnlinkIntent => "unlink_intent",
+            DeletionHistoryPublishKind::UnlinkBatchInProcess => "unlink_batch",
+            DeletionHistoryPublishKind::UnmountFloorRaise => "unmount_floor_raise",
+            DeletionHistoryPublishKind::MountInstanceRow => "mount_instance_row",
+            DeletionHistoryPublishKind::MountWarmUp => "mount_warm_up",
+            DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation => {
+                "mount_continuation_unlink_batch"
+            }
+        }
+    }
+}
+
+/// 删除历史里不属于任何一次发布的写。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum DeletionHistoryWritesOutsideAnyPublish {
+    /// 正常卸载在它第一次发布之前先把新 F 写进每块盘系统配置那一步。
+    UnmountSystemConfigurationBeforeTheFirstFloorRaise,
+    /// 可写挂载在写行那次发布之前发的写（core 的发布写账不记它们，按「挂载那一截的写数减去各次发布的写账」算出来）。
+    MountWritesBeforeItsFirstPublish,
+}
+
+impl DeletionHistoryWritesOutsideAnyPublish {
+    /// 结果行里的名字。
+    #[must_use]
+    pub const fn name(self) -> &'static str {
+        match self {
+            DeletionHistoryWritesOutsideAnyPublish::UnmountSystemConfigurationBeforeTheFirstFloorRaise => {
+                "unmount_system_configuration_before_the_first_floor_raise"
+            }
+            DeletionHistoryWritesOutsideAnyPublish::MountWritesBeforeItsFirstPublish => {
+                "mount_writes_before_its_first_publish"
+            }
+        }
+    }
+}
+
+/// 删除历史里 unlink 之后的一段：`operations` 是录制流下标的半开区间（与 [`SharedStream::operations`] 同一套下标）。
+#[derive(Clone, Debug)]
+pub enum DeletionHistorySegment {
+    Publish {
+        kind: DeletionHistoryPublishKind,
+        version: Box<TransactionOutput>,
+        operations: Range<usize>,
+    },
+    WritesOutsideAnyPublish {
+        kind: DeletionHistoryWritesOutsideAnyPublish,
+        operations: Range<usize>,
+    },
+}
+
+impl DeletionHistorySegment {
+    #[must_use]
+    pub fn operations(&self) -> Range<usize> {
+        match self {
+            DeletionHistorySegment::Publish { operations, .. }
+            | DeletionHistorySegment::WritesOutsideAnyPublish { operations, .. } => {
+                operations.clone()
+            }
+        }
+    }
+}
+
+/// 一段删除历史跑完交回的东西。
+pub struct DeletionHistoryRun {
+    pub mkfs_operation_count: usize,
+    /// 意图那次发布之前录制流的长度：导出的窗口从这里起。
+    pub operations_before_the_unlink: usize,
+    /// unlink 之后的每一段，按录制流的先后，首尾相接铺满 [`operations_before_the_unlink`](Self::operations_before_the_unlink) 到流尾。
+    pub segments: Vec<DeletionHistorySegment>,
+    /// 这段历史最后的现行那一版：H1 是这个进程最后一批，H2 是挂载交回的现行那一版。
+    pub final_version: PoolVersion,
+}
+
+/// 删除历史里的哪一步。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum DeletionHistoryStep {
+    NewPoolFileCreationPath(NewPoolFileCreationPathStep),
+    SequentialWrite,
+    UnlinkIntent,
+    UnlinkBatchInProcess,
+    Unmount,
+    WritableMount,
+    /// 卸载与挂载那两截按 core 的写账切开那一步。
+    AttributingTheWritesToPublishes,
+}
+
+/// 删除历史没走完：停在哪一步、为什么。
+#[derive(Debug)]
+pub struct DeletionHistoryFailure {
+    pub failed_step: DeletionHistoryStep,
+    pub cause: String,
+}
+
+impl DeletionHistoryFailure {
+    fn at(failed_step: DeletionHistoryStep, cause: String) -> Self {
+        Self { failed_step, cause }
+    }
+}
+
+/// 一次调用里先后发出的一段，切开之前：哪一种、core 的写账记了几次写调用。
+struct PartOfACall {
+    part: PartOfACallKind,
+    write_calls: u64,
+}
+
+enum PartOfACallKind {
+    Publish {
+        kind: DeletionHistoryPublishKind,
+        version: Box<TransactionOutput>,
+    },
+    OutsideAnyPublish(DeletionHistoryWritesOutsideAnyPublish),
+}
+
+impl PartOfACall {
+    fn publish(kind: DeletionHistoryPublishKind, version: TransactionOutput) -> Self {
+        Self {
+            write_calls: version.writes.total().write_calls,
+            part: PartOfACallKind::Publish {
+                kind,
+                version: Box::new(version),
+            },
+        }
+    }
+}
+
+/// 一次调用在录制流里发出的那一截（`operations_of_the_call`），按先后几段各自的写数（`write_calls_of_each_part`，每个都至少 1）切开：
+/// 第 k 段是第 k 个写数那么多次写，加上紧跟其后、下一段第一次写之前的屏障；这一截开头、第一次写之前的屏障归第一段。
+/// 交回各段在这一截里的半开区间，首尾相接铺满整截。只看操作是不是屏障，不看偏移、长度与内容。
+///
+/// # Errors
+/// 有一段写数是 0，或这一截的写数与各段之和对不上（多了或少了）。
+pub fn spans_of_consecutive_parts_by_their_write_calls(
+    operations_of_the_call: &[RecordedOperation],
+    write_calls_of_each_part: &[u64],
+) -> Result<Vec<Range<usize>>, String> {
+    if write_calls_of_each_part.contains(&0) {
+        return Err(format!(
+            "每段至少一次写才切得开：{write_calls_of_each_part:?}"
+        ));
+    }
+    let writes_in_the_call = operations_of_the_call
+        .iter()
+        .filter(|operation| operation.kind != RecordedOperationKind::Barrier)
+        .count();
+    let writes_of_the_parts: u64 = write_calls_of_each_part.iter().sum();
+    if u64::try_from(writes_in_the_call).expect("录制流的长度装得进 u64") != writes_of_the_parts
+    {
+        return Err(format!(
+            "这一截有 {writes_in_the_call} 次写，各段写账加起来 {writes_of_the_parts}：{write_calls_of_each_part:?}"
+        ));
+    }
+    let mut spans = Vec::with_capacity(write_calls_of_each_part.len());
+    let mut part_start = 0;
+    let mut part_index = 0;
+    let mut writes_in_this_part = 0;
+    // 跨轮携带的状态：`part_index` 是正在装的那一段，`writes_in_this_part` 是它已装的写数，不超过它的写账
+    // （上面核过总数相等，装满一段才遇到下一次写时换段，`part_index` 不越界）。迭代次数的上界是这一截的长度，没有提前出口。
+    for (index, operation) in operations_of_the_call.iter().enumerate() {
+        if operation.kind == RecordedOperationKind::Barrier {
+            continue;
+        }
+        if writes_in_this_part == write_calls_of_each_part[part_index] {
+            spans.push(part_start..index);
+            part_index += 1;
+            part_start = index;
+            writes_in_this_part = 0;
+        }
+        writes_in_this_part += 1;
+        assert!(
+            writes_in_this_part <= write_calls_of_each_part[part_index],
+            "装满一段才换段：第 {part_index} 段装了 {writes_in_this_part} 次写"
+        );
+    }
+    spans.push(part_start..operations_of_the_call.len());
+    assert_eq!(
+        spans.len(),
+        write_calls_of_each_part.len(),
+        "总数相等时每段都装满了、最后一段收在这一截的尾上"
+    );
+    Ok(spans)
+}
+
+/// 把一次调用在录制流里发出的那一截按 `parts` 的写账切成几段；写数是 0 的「不属于任何一次发布的写」不成段。
+fn segments_of_one_call(
+    stream: &SharedStream,
+    operations_of_the_call: Range<usize>,
+    parts: Vec<PartOfACall>,
+) -> Result<Vec<DeletionHistorySegment>, DeletionHistoryFailure> {
+    let parts: Vec<PartOfACall> = parts
+        .into_iter()
+        .filter(|part| match part.part {
+            PartOfACallKind::Publish { .. } => true,
+            PartOfACallKind::OutsideAnyPublish(_) => part.write_calls > 0,
+        })
+        .collect();
+    let write_calls: Vec<u64> = parts.iter().map(|part| part.write_calls).collect();
+    let recorded_operations = stream.operations();
+    let spans = spans_of_consecutive_parts_by_their_write_calls(
+        &recorded_operations[operations_of_the_call.clone()],
+        &write_calls,
+    )
+    .map_err(|cause| {
+        DeletionHistoryFailure::at(DeletionHistoryStep::AttributingTheWritesToPublishes, cause)
+    })?;
+    Ok(parts
+        .into_iter()
+        .zip(spans)
+        .map(|(part, span)| {
+            let operations =
+                operations_of_the_call.start + span.start..operations_of_the_call.start + span.end;
+            match part.part {
+                PartOfACallKind::Publish { kind, version } => DeletionHistorySegment::Publish {
+                    kind,
+                    version,
+                    operations,
+                },
+                PartOfACallKind::OutsideAnyPublish(kind) => {
+                    DeletionHistorySegment::WritesOutsideAnyPublish { kind, operations }
+                }
+            }
+        })
+        .collect())
+}
+
+/// 三·6 干跑那两段删除历史（D8（核心索引结构） 已定项 16）：E142 那套参数上 mkfs → 取号 → 暖机 → 新池新建文件
+/// （与 [`run_new_pool_file_creation`] 同一条路）→ 同一个实例里把第一个文件顺序写成三个数据单元（[`three_data_unit_file_content`]）
+/// → unlink，批大小 [`deletion_history_batch_size`]。H1 意图与每一批都在这个进程里发，直到队列空；H2 意图在这个进程里发，
+/// 正常卸载（[`unmount`]，录制流把那一截记成卸载入口），丢掉这个进程的分配器，同一组盘重开可写挂载（[`deletion_history_switches`]）：
+/// 挂载续做在写行与暖机之后把队列走到空。
+///
+/// 交回 unlink 之前录制流的长度与它之后的每一段：直接调的发布各一段；卸载与挂载里的几次发布按 core 交回的每次发布写账
+/// （`TransactionOutput::writes`，一次写调用 = 交给一块盘的一次写）在流里切开（[`spans_of_consecutive_parts_by_their_write_calls`]）。
+///
+/// # Errors
+/// 哪一步没做成、为什么。挂载推了抬 F、做了清扫发布、续做没走到空、某一版没有树，或写账与流里那一截的写数对不上，
+/// 都报在 [`DeletionHistoryStep::AttributingTheWritesToPublishes`]：这几样这段历史不该出现，出现了就切不准。
+pub fn run_deletion_history<Device: BlockDevice>(
+    history: DeletionHistory,
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    stream: &SharedStream,
+) -> Result<DeletionHistoryRun, DeletionHistoryFailure> {
+    let (new_pool_file_creation, mut allocator) = run_new_pool_file_creation_keeping_the_allocator(
+        parameters,
+        devices.as_mut_slice(),
+        stream,
+        |_point, _devices| {},
+    )
+    .map_err(|failure| {
+        DeletionHistoryFailure::at(
+            DeletionHistoryStep::NewPoolFileCreationPath(failure.failed_step),
+            failure.cause,
+        )
+    })?;
+    let content = three_data_unit_file_content();
+    let three_units = publish_sequential_write(
+        &mut PoolWriter::new(parameters, devices.as_mut_slice()),
+        &mut allocator,
+        &new_pool_file_creation.output,
+        FirstFile {
+            content: &content,
+            write_time_seconds: FIXED_WRITE_TIME_SECONDS
+                + SECONDS_FROM_THE_FIRST_FILE_TO_THE_SEQUENTIAL_WRITE_IN_THE_DELETION_HISTORIES,
+        },
+        INSTANCE_OF_THE_NEW_POOL_FILE_CREATION,
+    )
+    .map_err(|error| {
+        DeletionHistoryFailure::at(DeletionHistoryStep::SequentialWrite, format!("{error:?}"))
+    })?;
+    let operations_before_the_unlink = stream.operation_count();
+    // 意图带第一个文件那条记录现有的写入时间：与 harness 档 unlink 那几条用例同一个取法。
+    let write_time_seconds = three_units
+        .inode_record
+        .expect("顺序写之后第一个文件那条记录在树里")
+        .write_time_seconds;
+    let intent = publish_unlink_intent(
+        &mut PoolWriter::new(parameters, devices.as_mut_slice()),
+        &mut allocator,
+        &three_units,
+        FILE_DELETED_IN_THE_DELETION_HISTORIES,
+        write_time_seconds,
+        INSTANCE_OF_THE_NEW_POOL_FILE_CREATION,
+    )
+    .map_err(|error| {
+        DeletionHistoryFailure::at(DeletionHistoryStep::UnlinkIntent, format!("{error:?}"))
+    })?;
+    let intent_segment = DeletionHistorySegment::Publish {
+        kind: DeletionHistoryPublishKind::UnlinkIntent,
+        version: Box::new(intent.clone()),
+        operations: operations_before_the_unlink..stream.operation_count(),
+    };
+    let (later_segments, final_version) = match history {
+        DeletionHistory::IntentAndEveryBatchInProcess => {
+            every_batch_in_process(parameters, devices, stream, &mut allocator, intent)?
+        }
+        DeletionHistory::IntentThenUnmountThenMountContinuation => {
+            unmount_then_mount_continuation(parameters, devices, stream, allocator, intent)?
+        }
+    };
+    Ok(DeletionHistoryRun {
+        mkfs_operation_count: new_pool_file_creation.mkfs_operation_count,
+        operations_before_the_unlink,
+        segments: std::iter::once(intent_segment)
+            .chain(later_segments)
+            .collect(),
+        final_version,
+    })
+}
+
+/// H1 意图之后：这个进程里一批一批发，直到队列空。
+fn every_batch_in_process<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    stream: &SharedStream,
+    allocator: &mut PoolAllocator,
+    intent: TransactionOutput,
+) -> Result<(Vec<DeletionHistorySegment>, PoolVersion), DeletionHistoryFailure> {
+    // 迭代次数的上界：每批至少截掉一个数据单元，意图那一版有几个数据单元就至多几批；跨轮携带的是现行那一版，
+    // 出口是队列空。上界用完队列还不空就是批没截短，报错、不再发。
+    let batch_limit = intent.data_pointers.len();
+    let mut segments = Vec::with_capacity(batch_limit);
+    let mut current = intent;
+    for _ in 0..batch_limit {
+        if current.deleted_inodes.is_empty() {
+            break;
+        }
+        let start = stream.operation_count();
+        current = publish_unlink_step(
+            &mut PoolWriter::new(parameters, devices.as_mut_slice()),
+            allocator,
+            &current,
+            FILE_DELETED_IN_THE_DELETION_HISTORIES,
+            deletion_history_batch_size(),
+            INSTANCE_OF_THE_NEW_POOL_FILE_CREATION,
+        )
+        .map_err(|error| {
+            DeletionHistoryFailure::at(
+                DeletionHistoryStep::UnlinkBatchInProcess,
+                format!("{error:?}"),
+            )
+        })?;
+        segments.push(DeletionHistorySegment::Publish {
+            kind: DeletionHistoryPublishKind::UnlinkBatchInProcess,
+            version: Box::new(current.clone()),
+            operations: start..stream.operation_count(),
+        });
+    }
+    if !current.deleted_inodes.is_empty() {
+        return Err(DeletionHistoryFailure::at(
+            DeletionHistoryStep::UnlinkBatchInProcess,
+            format!(
+                "发了 {batch_limit} 批队列还不空：{:?}",
+                current.deleted_inodes
+            ),
+        ));
+    }
+    Ok((segments, PoolVersion::of_a_version_of_trees(current)))
+}
+
+/// 一版要有树才切得开：写行与暖机发在有树的一版上，交回的就是有树的一版。
+fn version_of_trees_of_a_mount_publish(
+    version: &PoolVersion,
+) -> Result<TransactionOutput, DeletionHistoryFailure> {
+    version.version_of_trees().cloned().ok_or_else(|| {
+        DeletionHistoryFailure::at(
+            DeletionHistoryStep::AttributingTheWritesToPublishes,
+            format!(
+                "挂载里 txg {:?} 那次发布交回的一版没有树",
+                version.root().checkpoint_txg
+            ),
+        )
+    })
+}
+
+/// H2 意图之后：正常卸载，进程退出，同一组盘重开可写挂载，挂载续做走到空。
+fn unmount_then_mount_continuation<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    stream: &SharedStream,
+    mut allocator: PoolAllocator,
+    intent: TransactionOutput,
+) -> Result<(Vec<DeletionHistorySegment>, PoolVersion), DeletionHistoryFailure> {
+    let mut current = PoolVersion::WithFile(intent);
+    let unmount_start = stream.operation_count();
+    let unmounted = stream
+        .record_entry(RecordedPublishEntry::Unmount, || {
+            unmount(
+                parameters,
+                devices,
+                &mut allocator,
+                &mut current,
+                ShadowLedger::On,
+            )
+        })
+        .map_err(|error| {
+            DeletionHistoryFailure::at(DeletionHistoryStep::Unmount, format!("{error:?}"))
+        })?;
+    let unmount_operations = unmount_start..stream.operation_count();
+    let raised = match unmounted {
+        Unmounted::FloorRaisedToTheCurrentVersion(raised) => raised,
+        Unmounted::NothingWrittenOnAVersionWithoutFile {
+            current: version_the_unmount_saw,
+        } => {
+            return Err(DeletionHistoryFailure::at(
+                DeletionHistoryStep::Unmount,
+                format!("意图那一版带文件，卸载却一个字节都没写：{version_the_unmount_saw:?}"),
+            ));
+        }
+    };
+    // 进程退出：这个进程的分配器随它丢掉；盘留着，重开就是同一组盘接着用。
+    drop(allocator);
+    let mut unmount_parts = vec![PartOfACall {
+        write_calls: raised
+            .system_configuration_writes_before_the_first_publish
+            .total()
+            .write_calls,
+        part: PartOfACallKind::OutsideAnyPublish(
+            DeletionHistoryWritesOutsideAnyPublish::UnmountSystemConfigurationBeforeTheFirstFloorRaise,
+        ),
+    }];
+    unmount_parts.extend(raised.publishes.into_iter().map(|publish| {
+        PartOfACall::publish(DeletionHistoryPublishKind::UnmountFloorRaise, publish)
+    }));
+    let unmount_segments = segments_of_one_call(stream, unmount_operations, unmount_parts)?;
+
+    let mount_start = stream.operation_count();
+    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
+        parameters,
+        devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        deletion_history_switches(),
+    )
+    .map_err(|error| {
+        DeletionHistoryFailure::at(DeletionHistoryStep::WritableMount, format!("{error:?}"))
+    })?;
+    let mount_operations = mount_start..stream.operation_count();
+    let output = &mounted.output;
+    let publishes_this_history_does_not_attribute = [
+        (
+            !output.space_admission.floor_raises().is_empty(),
+            "挂载在写行之后推了抬 F",
+        ),
+        (
+            !output.floor_raises_pushed_by_the_continuation.is_empty(),
+            "挂载续做推了抬 F",
+        ),
+        (
+            matches!(
+                output.instance_table_sweep,
+                InstanceTableSweepAtTheEndOfTheMount::Swept(_)
+            ),
+            "挂载末尾判了实例表清扫",
+        ),
+        (
+            !matches!(
+                output.deletion_queue_continuation_outcome,
+                DeletionQueueContinuationOutcome::WalkedToEmpty
+            ),
+            "挂载续做没走到空",
+        ),
+    ];
+    if let Some((_, what)) = publishes_this_history_does_not_attribute
+        .iter()
+        .find(|(happened, _)| *happened)
+    {
+        return Err(DeletionHistoryFailure::at(
+            DeletionHistoryStep::AttributingTheWritesToPublishes,
+            format!("{what}：这段历史只切写行、暖机与续做各批"),
+        ));
+    }
+    let mut publishes = vec![PartOfACall::publish(
+        DeletionHistoryPublishKind::MountInstanceRow,
+        version_of_trees_of_a_mount_publish(&output.row_publish)?,
+    )];
+    for warm_up_publish in &output.warm_up_publishes {
+        publishes.push(PartOfACall::publish(
+            DeletionHistoryPublishKind::MountWarmUp,
+            version_of_trees_of_a_mount_publish(warm_up_publish)?,
+        ));
+    }
+    publishes.extend(output.deletion_queue_continuation.iter().map(|step| {
+        PartOfACall::publish(
+            DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation,
+            step.clone(),
+        )
+    }));
+    let writes_of_the_publishes: u64 = publishes.iter().map(|part| part.write_calls).sum();
+    let writes_in_the_mount = u64::try_from(
+        stream.operations()[mount_operations.clone()]
+            .iter()
+            .filter(|operation| operation.kind != RecordedOperationKind::Barrier)
+            .count(),
+    )
+    .expect("录制流的长度装得进 u64");
+    let writes_before_the_first_publish = writes_in_the_mount
+        .checked_sub(writes_of_the_publishes)
+        .ok_or_else(|| {
+            DeletionHistoryFailure::at(
+                DeletionHistoryStep::AttributingTheWritesToPublishes,
+                format!(
+                    "挂载那一截 {writes_in_the_mount} 次写，少于各次发布写账之和 {writes_of_the_publishes}"
+                ),
+            )
+        })?;
+    let mount_parts = std::iter::once(PartOfACall {
+        write_calls: writes_before_the_first_publish,
+        part: PartOfACallKind::OutsideAnyPublish(
+            DeletionHistoryWritesOutsideAnyPublish::MountWritesBeforeItsFirstPublish,
+        ),
     })
+    .chain(publishes)
+    .collect();
+    let mount_segments = segments_of_one_call(stream, mount_operations, mount_parts)?;
+    Ok((
+        unmount_segments.into_iter().chain(mount_segments).collect(),
+        mounted.current,
+    ))
 }
diff --git a/crates/singlefs-harness/tests/deletion_scenario_writes.rs b/crates/singlefs-harness/tests/deletion_scenario_writes.rs
new file mode 100644
index 0000000..6234537
--- /dev/null
+++ b/crates/singlefs-harness/tests/deletion_scenario_writes.rs
@@ -0,0 +1,297 @@
+//! 三·6 干跑那两段删除历史（`scenario::run_deletion_history`，D8（核心索引结构） 已定项 16）在录制流里怎么分段：
+//! 每次删除类发布重写几个单元（每盘）、按什么次序发，以及卸载与挂载那两截按 core 的发布写账切开之后每段对不对得上。
+//! 进程里那三批的单元数照 `unlink_single_file_publishes.rs` 现钉的角色集合：意图 13、3 → 2 那一批 12、最后一批 10；2 → 1 那一批 11。
+//! 挂载续做那三批在卸载、写行与暖机之后发，分配记录树根之下多改一对叶（`t5@0.0.62`、`t5@0.1.62`），后两批各多 2 个；
+//! 分配记录树根之下那几片之外的角色数两段历史逐批相同（9、8、7、6）。
+
+use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::transaction::{PoolVersion, TransactionOutput, TransactionUnit};
+use singlefs_core::write_accounting::WrittenStructureKind;
+use singlefs_format::TEST_IMAGE_DEFAULT_BYTES;
+use singlefs_harness::memory_pool::SparseBlockDevice;
+use singlefs_harness::scenario::{
+    e142_parameters, run_deletion_history, spans_of_consecutive_parts_by_their_write_calls,
+    DeletionHistory, DeletionHistoryPublishKind, DeletionHistoryRun, DeletionHistorySegment,
+};
+use singlefs_harness::{
+    RecordedOperation, RecordedOperationKind, RecordingBlockDevice, SharedStream,
+};
+
+/// 两块 4 GiB 内存盘（E142 那套几何）上跑一段删除历史，连同它的录制流交回。
+fn run_on_two_memory_devices(
+    history: DeletionHistory,
+) -> (DeletionHistoryRun, Vec<RecordedOperation>) {
+    let parameters = e142_parameters(512, 512);
+    let stream = SharedStream::retaining_contents();
+    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = [0, 1]
+        .map(|device_number| {
+            let identity = DeviceIdentity(device_number);
+            (
+                identity,
+                RecordingBlockDevice::with_shared_stream(
+                    identity,
+                    SparseBlockDevice::new(TEST_IMAGE_DEFAULT_BYTES, PhysicalBlockSizeInBytes(512)),
+                    stream.clone(),
+                ),
+            )
+        })
+        .into();
+    let run = run_deletion_history(history, &parameters, &mut devices, &stream)
+        .unwrap_or_else(|failure| panic!("{}：{failure:?}", history.name()));
+    (run, stream.operations())
+}
+
+fn publishes(run: &DeletionHistoryRun) -> Vec<(DeletionHistoryPublishKind, &TransactionOutput)> {
+    run.segments
+        .iter()
+        .filter_map(|segment| match segment {
+            DeletionHistorySegment::Publish { kind, version, .. } => {
+                Some((*kind, version.as_ref()))
+            }
+            DeletionHistorySegment::WritesOutsideAnyPublish { .. } => None,
+        })
+        .collect()
+}
+
+fn is_a_deletion_publish(kind: DeletionHistoryPublishKind) -> bool {
+    match kind {
+        DeletionHistoryPublishKind::UnlinkIntent
+        | DeletionHistoryPublishKind::UnlinkBatchInProcess
+        | DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation => true,
+        DeletionHistoryPublishKind::UnmountFloorRaise
+        | DeletionHistoryPublishKind::MountInstanceRow
+        | DeletionHistoryPublishKind::MountWarmUp => false,
+    }
+}
+
+/// 重写的角色里分配记录树根之下那几片之外的个数：那几片按槽号按位置寻址，改哪几片随这段历史之前用过哪些槽变。
+fn rewritten_units_outside_the_allocation_record_tree_below_its_root(
+    version: &TransactionOutput,
+) -> usize {
+    version
+        .rewritten
+        .iter()
+        .filter(|role| !matches!(role, TransactionUnit::AllocationTreeNodeBelowTheRoot(_)))
+        .count()
+}
+
+/// 每次删除类发布：哪一种、重写的单元数（每盘）、其中分配记录树根之下那几片之外的个数、这一版之后剩几个数据单元。
+fn deletion_publishes_with_units_per_device_and_data_units_after(
+    run: &DeletionHistoryRun,
+) -> Vec<(DeletionHistoryPublishKind, usize, usize, usize)> {
+    publishes(run)
+        .into_iter()
+        .filter(|(kind, _)| is_a_deletion_publish(*kind))
+        .map(|(kind, version)| {
+            (
+                kind,
+                version.rewritten.len(),
+                rewritten_units_outside_the_allocation_record_tree_below_its_root(version),
+                version.data_pointers.len(),
+            )
+        })
+        .collect()
+}
+
+/// 一次发布的写账里单元写的次数（journal 记录、根槽、系统配置槽之外的那几种），两块盘各一次。
+fn unit_write_calls(version: &TransactionOutput) -> u64 {
+    WrittenStructureKind::IN_REPORT_ORDER
+        .into_iter()
+        .filter(|kind| {
+            !matches!(
+                kind,
+                WrittenStructureKind::JournalRecord
+                    | WrittenStructureKind::RootSlot
+                    | WrittenStructureKind::SystemConfigurationSlot
+            )
+        })
+        .map(|kind| version.writes.of(kind).write_calls)
+        .sum()
+}
+
+/// 每段对不对得上：各段首尾相接铺满窗口；每次发布那一段的写数等于它自己的写账、恰有一次根槽 FUA、单元写 = 重写角色数 × 2 盘。
+fn segment_mismatches(run: &DeletionHistoryRun, operations: &[RecordedOperation]) -> Vec<String> {
+    let mut mismatches = Vec::new();
+    let mut expected_start = run.operations_before_the_unlink;
+    for segment in &run.segments {
+        let span = segment.operations();
+        if span.start != expected_start {
+            mismatches.push(format!("{span:?} 不接在 {expected_start} 上"));
+        }
+        expected_start = span.end;
+        let DeletionHistorySegment::Publish { kind, version, .. } = segment else {
+            continue;
+        };
+        let writes = operations[span.clone()]
+            .iter()
+            .filter(|operation| operation.kind != RecordedOperationKind::Barrier)
+            .count();
+        let force_unit_access_writes = operations[span.clone()]
+            .iter()
+            .filter(|operation| operation.kind == RecordedOperationKind::WriteForceUnitAccess)
+            .count();
+        let observed = (
+            u64::try_from(writes).expect("段长"),
+            force_unit_access_writes,
+            unit_write_calls(version),
+        );
+        let expected = (
+            version.writes.total().write_calls,
+            1,
+            2 * u64::try_from(version.rewritten.len()).expect("角色数"),
+        );
+        if observed != expected {
+            mismatches.push(format!(
+                "{kind:?} txg {:?} {span:?}：（写数、FUA 数、单元写）= {observed:?}，应为 {expected:?}",
+                version.root.checkpoint_txg
+            ));
+        }
+    }
+    if expected_start != operations.len() {
+        mismatches.push(format!(
+            "各段收在 {expected_start}，流尾在 {}",
+            operations.len()
+        ));
+    }
+    mismatches
+}
+
+fn write_at(device: u32) -> RecordedOperation {
+    RecordedOperation {
+        device: DeviceIdentity(device),
+        kind: RecordedOperationKind::Write,
+        offset: DeviceOffsetInBytes(0),
+        length: 512,
+        content_hash: 0,
+    }
+}
+
+fn barrier_on(device: u32) -> RecordedOperation {
+    RecordedOperation {
+        device: DeviceIdentity(device),
+        kind: RecordedOperationKind::Barrier,
+        offset: DeviceOffsetInBytes(0),
+        length: 0,
+        content_hash: 0,
+    }
+}
+
+#[test]
+fn spans_of_consecutive_parts_give_each_part_the_barriers_after_its_last_write_and_the_first_part_the_leading_barriers(
+) {
+    let operations = [
+        barrier_on(0),
+        write_at(0),
+        write_at(1),
+        barrier_on(0),
+        barrier_on(1),
+        write_at(0),
+        barrier_on(0),
+        write_at(1),
+    ];
+    assert_eq!(
+        spans_of_consecutive_parts_by_their_write_calls(&operations, &[2, 1, 1]),
+        Ok(vec![0..5, 5..7, 7..8]),
+        "开头那道屏障归第一段；每段收在下一段第一次写之前"
+    );
+}
+
+#[test]
+fn spans_of_consecutive_parts_refuse_a_part_without_writes_and_accounts_that_do_not_add_up() {
+    let operations = [write_at(0), barrier_on(0), write_at(1)];
+    for write_calls in [&[2, 0][..], &[1, 0, 1][..], &[1][..], &[1, 2][..]] {
+        assert!(
+            spans_of_consecutive_parts_by_their_write_calls(&operations, write_calls).is_err(),
+            "写账 {write_calls:?} 切不开两次写，要拒"
+        );
+    }
+}
+
+#[test]
+fn the_in_process_history_publishes_the_intent_then_three_batches_of_thirteen_twelve_eleven_and_ten_units_per_device(
+) {
+    let (run, operations) =
+        run_on_two_memory_devices(DeletionHistory::IntentAndEveryBatchInProcess);
+    assert_eq!(
+        deletion_publishes_with_units_per_device_and_data_units_after(&run),
+        [
+            (DeletionHistoryPublishKind::UnlinkIntent, 13, 9, 3),
+            (DeletionHistoryPublishKind::UnlinkBatchInProcess, 12, 8, 2),
+            (DeletionHistoryPublishKind::UnlinkBatchInProcess, 11, 7, 1),
+            (DeletionHistoryPublishKind::UnlinkBatchInProcess, 10, 6, 0),
+        ],
+        "意图 13、3 → 2 那一批 12、2 → 1 那一批 11、最后一批 10（每盘），批大小 1 走三批"
+    );
+    let txgs: Vec<u64> = publishes(&run)
+        .iter()
+        .map(|(_, version)| version.root.checkpoint_txg.0)
+        .collect();
+    assert_eq!(
+        txgs,
+        (txgs[0]..txgs[0] + 4).collect::<Vec<_>>(),
+        "四次发布按 txg 连着、没有别的发布插进来"
+    );
+    assert!(
+        matches!(&run.final_version, PoolVersion::WithTreesAndNoFile(version) if version.deleted_inodes.is_empty()),
+        "最后一批之后是删空的一版、队列空"
+    );
+    assert_eq!(segment_mismatches(&run, &operations), Vec::<String>::new());
+}
+
+#[test]
+fn the_mount_continuation_history_publishes_its_three_batches_after_the_unmount_the_row_and_the_warm_up(
+) {
+    let (run, operations) =
+        run_on_two_memory_devices(DeletionHistory::IntentThenUnmountThenMountContinuation);
+    assert_eq!(
+        deletion_publishes_with_units_per_device_and_data_units_after(&run),
+        [
+            (DeletionHistoryPublishKind::UnlinkIntent, 13, 9, 3),
+            (DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation, 12, 8, 2),
+            (DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation, 13, 7, 1),
+            (DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation, 12, 6, 0),
+        ],
+        "意图在进程里发，三批都由挂载续做发：分配记录树根之下那几片之外与进程里那三批相同，后两批多改一对分配记录树的叶"
+    );
+    let kinds_and_txgs: Vec<(DeletionHistoryPublishKind, u64)> = publishes(&run)
+        .iter()
+        .map(|(kind, version)| (*kind, version.root.checkpoint_txg.0))
+        .collect();
+    let intent_txg = kinds_and_txgs[0].1;
+    assert_eq!(
+        kinds_and_txgs,
+        [
+            (DeletionHistoryPublishKind::UnlinkIntent, intent_txg),
+            (
+                DeletionHistoryPublishKind::UnmountFloorRaise,
+                intent_txg + 1
+            ),
+            (
+                DeletionHistoryPublishKind::UnmountFloorRaise,
+                intent_txg + 2
+            ),
+            (DeletionHistoryPublishKind::MountInstanceRow, intent_txg + 3),
+            (DeletionHistoryPublishKind::MountWarmUp, intent_txg + 4),
+            (DeletionHistoryPublishKind::MountWarmUp, intent_txg + 5),
+            (
+                DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation,
+                intent_txg + 6
+            ),
+            (
+                DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation,
+                intent_txg + 7
+            ),
+            (
+                DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation,
+                intent_txg + 8
+            ),
+        ],
+        "意图 → 卸载两次抬 F → 写行 → 暖机两次 → 续做三批，按 txg 连着"
+    );
+    assert!(
+        matches!(&run.final_version, PoolVersion::WithTreesAndNoFile(version) if version.deleted_inodes.is_empty()),
+        "挂载交回删空的一版、队列空"
+    );
+    assert_eq!(segment_mismatches(&run, &operations), Vec::<String>::new());
+}
````

### x：mutations-append.tsv

````
三·6 删除历史：只供测试的批大小从 1 改成 2（H1 少走一批）	crates/singlefs-harness/src/scenario.rs	        UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=产品批大小里"),	        UnlinkBatchDataUnitsCappedForTests::new(2).expect("1 在 1..=产品批大小里"),	-p singlefs-harness --test deletion_scenario_writes	the_in_process_history_publishes_the_intent_then_three_batches_of_thirteen_twelve_eleven_and_ten_units_per_device
三·6 删除历史：H2 的可写挂载不装批大小 1 的删除开关、续做走产品批大小	crates/singlefs-harness/src/scenario.rs	        deletion_history_switches(),	        DeletionSwitches::PRODUCT,	-p singlefs-harness --test deletion_scenario_writes	the_mount_continuation_history_publishes_its_three_batches_after_the_unmount_the_row_and_the_warm_up
三·6 删除历史：按写账切段时每段少收末尾一步（段与段之间漏一步）	crates/singlefs-harness/src/scenario.rs	            spans.push(part_start..index);	            spans.push(part_start..index - 1);	-p singlefs-harness --test deletion_scenario_writes	spans_of_consecutive_parts_give_each_part_the_barriers_after_its_last_write_and_the_first_part_the_leading_barriers
三·6 删除历史：按写账切段不拒写数是 0 的段	crates/singlefs-harness/src/scenario.rs	    if write_calls_of_each_part.contains(&0) {	    if write_calls_of_each_part.is_empty() {	-p singlefs-harness --test deletion_scenario_writes	spans_of_consecutive_parts_refuse_a_part_without_writes_and_accounts_that_do_not_add_up
E142 第 22 次第二段：导出不带参数时每盘字节数不再是 TEST_IMAGE_DEFAULT_BYTES	crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs	        [] => Ok(TEST_IMAGE_DEFAULT_BYTES),	        [] => Ok(8_589_934_592),	-p singlefs-checker-tier --bin e142_new_pool_file_creation_write_dump	no_argument_keeps_the_four_gibibyte_devices_of_the_first_registration
E142 第 22 次第二段：盘大小参数按 u32 解（8 GiB 解不开）	crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs	        [option, value] if option == DEVICE_BYTES_OPTION => value.parse::<u64>().map_err(|error| {	        [option, value] if option == DEVICE_BYTES_OPTION => value.parse::<u32>().map(u64::from).map_err(|error| {	-p singlefs-checker-tier --bin e142_new_pool_file_creation_write_dump	the_device_bytes_option_takes_the_eight_gibibyte_devices_of_the_second_geometry
E142 第 22 次第二段：盘大小参数不看选项名、什么选项都收	crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs	        [option, value] if option == DEVICE_BYTES_OPTION => value.parse::<u64>().map_err(|error| {	        [_option, value] => value.parse::<u64>().map_err(|error| {	-p singlefs-checker-tier --bin e142_new_pool_file_creation_write_dump	an_unparsable_value_an_unknown_option_and_a_missing_value_are_each_refused
三·6 删除导出：写清零的种类名打成 write	crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs	        RecordedOperationKind::WriteZeroes => "write_zeroes",	        RecordedOperationKind::WriteZeroes => "write",	-p singlefs-checker-tier --bin deletion_write_dump	kind_name_covers_every_recorded_operation_kind_without_a_wildcard_arm
三·6 删除导出：屏障也重建出（空的）字节	crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs	        RecordedOperationKind::Barrier => None,	        RecordedOperationKind::Barrier => Some(Vec::new()),	-p singlefs-checker-tier --bin deletion_write_dump	operation_bytes_rebuilds_an_all_zero_buffer_for_write_zeroes_and_none_for_a_barrier
````

## 实现员 y：crates.patch（/tmp/claude-1000/m3-batch18/y/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs b/crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs
--- a/crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs
+++ b/crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs
@@ -6,6 +6,8 @@
 //! 登记：`research/prompts/e161-preregistration.md`（5.5「第一段」：D1-small、D1-stride、D1-quick，停机条款 S1–S6、
 //! 锚点 A1–A7、5.3 全部阳性对照、第八节几何敏感性里 D1-quick 与线程数 1 那两格）。第二、三段（D2、D3 全量）是同一个装置换取样域，
 //! 本文件给了模式名但这一段不跑。
+//! 第 2 次重跑登记：`research/prompts/e161-r2-prereg.md`（C577 那道屏障之后的段形：第一条流 11 段、第二条流 78 段取段 11–26，
+//! 取样域照它 5.5 的读法甲重定，装置的改动照它 5.6 的 I-1 到 I-12）。
 //!
 //! 只读、只驱动、只观测：不改 `crates/singlefs-core`、`crates/singlefs-checker`、harness 已有的文件。
 //! 第一条流经 `#[path]` 引 `tests/common/mod.rs` 的 `build_pool`；第二条流的脚本照抄
@@ -83,7 +85,7 @@
 const STRIDE_STEP: u64 = 2_311;
 /// 对照样本里大段各取序号最小的这么多个（5.5）。
 const CONTROL_SAMPLE_PER_LARGE_SEGMENT: usize = 4_096;
-/// PC3：第二条流段 14 里序号最小的 1000 个状态、每个计时区各加 2 ms 忙等（5.3）。
+/// PC3：第二条流段 16（第 1 次登记的段 14，同一批写）里序号最小的 1000 个状态、每个计时区各加 2 ms 忙等（5.3；重跑登记 I-6）。
 const TIMING_CONTROL_STATES: usize = 1_000;
 const TIMING_CONTROL_BUSY_WAIT: Duration = Duration::from_millis(2);
 /// PC3 的判据：加忙等的那个区多出 ≥ 1.8 s，其余每个区变化 < 1 s。
@@ -95,11 +97,11 @@
 const SMALLEST_SIXTEEN_GIGABYTE_CARD_MEBIBYTES: u128 = 16_303;
 const FIVE_CARDS_MEBIBYTES: u128 = 97_843;
 const CARD_COUNT: u128 = 5;
-/// 外推点：问题单的 10¹⁰，另报第二条流整条流的全量状态数（第四节 4.2）。
+/// 外推点：问题单的 10¹⁰，另报第二条流整条流的全量状态数（第四节 4.2；重跑登记 A7：C577 之后 78 段）。
 const EXTRAPOLATED_STATES: u128 = 10_000_000_000;
-const SECOND_STREAM_FULL_STATES: u128 = 14_960_689_284;
-/// A7：第二条流整条流最大一段 3² · 2²⁸ − 1。
-const SECOND_STREAM_LARGEST_SEGMENT_STATES: u128 = 2_415_919_103;
+const SECOND_STREAM_FULL_STATES: u128 = 1_678_377_204;
+/// A7：第二条流整条流最大一段 2²⁸ − 1（重跑登记 A7：28 个单元写的段，C577 之前是 3² · 2²⁸ − 1 的 30 写段）。
+const SECOND_STREAM_LARGEST_SEGMENT_STATES: u128 = 268_435_455;
 /// 容量的 95% 到 100% 之间记「临界」（第六节 G5 表下）。
 const CRITICAL_BAND_PERCENT: u128 = 95;
 /// G5 的字节口径（第六节）：读集每次调用 21 字节，集合每个元素 20 字节，每状态三个键各 16 字节，每个不同键一条 20 字节。
@@ -289,7 +291,7 @@
 enum StreamName {
     /// 层 0 第一条流：mkfs → 取号 → 暖机两次 → 新池新建文件。
     First,
-    /// 层 0 第二条流：固定脚本到 E 再正常卸载，取段 9–22。
+    /// 层 0 第二条流：固定脚本到 E 再正常卸载，取段 11–26。
     Second,
 }
 
@@ -564,23 +566,24 @@
     }
 }
 
-/// S2 的锚点：第一条流段长 `[2,2,1,2,2,1,26,2,1,2]`、41 次写（`crash_enumeration_new_pool_file_creation_stream.rs:464-471`）；
-/// 第二条流 64 段的段长表、477 次写、19 条根（`crash_enumeration_fixed_script_stream.rs:431-483`）。
-const FIRST_STREAM_SEGMENT_LENGTHS: [usize; 10] = [2, 2, 1, 2, 2, 1, 26, 2, 1, 2];
+/// S2 的锚点（重跑登记 A8，C577 那道屏障之后的段形）：第一条流 11 段的段长、41 次写
+/// （`crash_enumeration_new_pool_file_creation_stream.rs:466`、`:470-472`）；
+/// 第二条流 78 段的段长表、481 次写、19 条根（`crash_enumeration_fixed_script_stream.rs:439-457`、`:489`，到 E 再正常卸载那一支）。
+const FIRST_STREAM_SEGMENT_LENGTHS: [usize; 11] = [2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2];
 const FIRST_STREAM_WRITES: usize = 41;
 /// 第一条流的根槽写：暖机两次各一条、新池新建文件一条（段表里三个 1 写段，收严：登记 S2 没列这一格）。
 const FIRST_STREAM_ROOTS: usize = 3;
-const SECOND_STREAM_SEGMENT_LENGTHS: [usize; 64] = [
-    2, 2, 1, 2, 2, 1, 26, 2, 1, 26, 2, 1, 2, 2, 18, 2, 1, 18, 2, 1, 18, 2, 1, 26, 2, 1, 22, 2, 1,
-    30, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 2, 2, 16, 2, 1, 18, 2, 1, 30, 2, 1, 2, 2, 16,
-    2, 1, 18, 2, 1, 2,
+const SECOND_STREAM_SEGMENT_LENGTHS: [usize; 78] = [
+    2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2, 16, 2, 1, 2, 24, 2,
+    1, 2, 24, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2,
+    1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2,
 ];
-const SECOND_STREAM_WRITES: usize = 477;
+const SECOND_STREAM_WRITES: usize = 481;
 const SECOND_STREAM_ROOTS: usize = 19;
-/// 第二条流取段 9–22（第一节「读法写死」）；段 0–8 与第一条流段 0–8 逐字节相同（S2）。
-const SECOND_STREAM_FIRST_SAMPLED_SEGMENT: usize = 9;
-const SECOND_STREAM_LAST_SAMPLED_SEGMENT: usize = 22;
-const SHARED_PREFIX_SEGMENTS: usize = 9;
+/// 第二条流取段 11–26（重跑登记 1.3「第二条流的取样段」）；段 0–10 与第一条流段 0–10 逐字节相同（S2），不枚举。
+const SECOND_STREAM_FIRST_SAMPLED_SEGMENT: usize = 11;
+const SECOND_STREAM_LAST_SAMPLED_SEGMENT: usize = 26;
+const SHARED_PREFIX_SEGMENTS: usize = 11;
 
 /// S2 的一行：每条判据各自一个布尔。
 struct StreamCheck {
@@ -605,7 +608,7 @@
     }
 }
 
-/// S2：两条流的基镜像逐字节相同；第二条流段 0–8 的写与第一条流段 0–8 逐字节相同。
+/// S2：两条流的基镜像逐字节相同；第二条流段 0–10 的写与第一条流段 0–10 逐字节相同。
 fn shared_prefix_is_identical(first: &PreparedStream, second: &PreparedStream) -> (bool, bool) {
     let bases_identical = first.base == second.base;
     let prefix_of = |stream: &PreparedStream| -> Vec<RetainedWrite> {
@@ -2733,7 +2736,7 @@
 
 // ───────────────────────────── 取样域（5.5） ─────────────────────────────
 
-/// 这条流的取样段（第一条流全部；第二条流段 9–22）。
+/// 这条流的取样段（第一条流全部；第二条流段 11–26）。
 fn is_sampled_segment(stream: StreamName, segment_index: usize) -> bool {
     match stream {
         StreamName::First => true,
@@ -2743,7 +2746,7 @@
     }
 }
 
-/// 全部持久那一个在不在域里：第一条流在（D2 = 150994980 含它），第二条流不在（段 23 起不取）。
+/// 全部持久那一个在不在域里：第一条流在（第一条流全量 16777260 含它），第二条流不在（段 27 起不取）。
 fn includes_every_write_persisted(stream: StreamName) -> bool {
     match stream {
         StreamName::First => true,
@@ -2763,12 +2766,13 @@
         .collect()
 }
 
-/// 26 写段（D1-stride 取步长的那两段）。
-const STRIDED_SEGMENT_WRITES: usize = 26;
-/// 对照样本里「大段」的写数下限（18 写段与 26 写段，5.5）。
+/// 大段的写数：24 写段（D1-stride 取步长、可行性档取段头的那两段：第一条流段 7、第二条流段 11）。
+/// 重跑登记读法甲：第 1 次的 26 写段（3² · 2²⁴ − 1 个状态）在 C577 之后拆成轮换 2 写与单元 24 写，阈值照改成 24。
+const STRIDED_SEGMENT_WRITES: usize = 24;
+/// 对照样本里「大段」的写数下限（18 写段与 24 写段，5.5）。
 const CONTROL_SAMPLE_LARGE_SEGMENT_WRITES: usize = 18;
 
-/// D1-small：取样段里写数少于 26 的每一段的全部状态（第一条流再加全部持久那一个）；按全量计划的序号。
+/// D1-small：取样段里写数少于 24 的每一段的全部状态（第一条流再加全部持久那一个）；按全量计划的序号。
 fn small_domain_ordinals(stream: &PreparedStream, plan: &LocalPlan<'_>) -> Vec<u64> {
     let mut ordinals = Vec::new();
     for (segment_index, segment) in stream.segments.iter().enumerate() {
@@ -2784,7 +2788,7 @@
     ordinals
 }
 
-/// D1-stride：26 写段里段内序号 ≡ 0 (mod 2311) 的状态。
+/// D1-stride：24 写段里段内序号 ≡ 0 (mod 2311) 的状态。
 fn stride_domain_ordinals(stream: &PreparedStream, plan: &LocalPlan<'_>) -> Vec<u64> {
     let mut ordinals = Vec::new();
     for (segment_index, segment) in stream.segments.iter().enumerate() {
@@ -2808,7 +2812,7 @@
     (0..last).collect()
 }
 
-/// 对照样本（5.5）：D1 里每一段，18 写段与 26 写段取序号最小的 4096 个，其余段取全部；第一条流的全部持久那一个也在。
+/// 对照样本（5.5）：D1 里每一段，18 写及以上的段取序号最小的 4096 个，其余段取全部；第一条流的全部持久那一个也在。
 fn control_ordinals(
     stream: &PreparedStream,
     plan: &LocalPlan<'_>,
@@ -2851,7 +2855,7 @@
 }
 
 /// 逐个数：只展开 `segment_index` 那一段的计划里，每个序号的持久集合真生成出来，数不同的有几个。取样段里一律逐个数（段内这一段的写、
-/// 它们的撕裂镜像与重放至多 32 个位置时用位图，否则用集合）；取样段之外（第二条流段 23 起，一段至多 3² · 2²⁸ − 1 个）按计划的组合数报。
+/// 它们的撕裂镜像与重放至多 32 个位置时用位图，否则用集合）；取样段之外（第二条流段 0–10 与段 27 起，一段至多 2²⁸ − 1 个）按计划的组合数报。
 /// 交回（数出来的个数，是不是逐个数的）。
 fn count_distinct_persisted_sets(
     stream: &PreparedStream,
@@ -3915,34 +3919,26 @@
 
 // ───────────────────────────── 锚点 A2–A7 的值（第七节 7.2，登记第十三节 anchors.py / anchors2.py 的原样输出） ─────────────────────────────
 
-const ANCHOR_FIRST_STREAM_SEGMENT_STATES: [u64; 10] = [8, 3, 1, 8, 3, 1, 150_994_943, 3, 1, 8];
-const ANCHOR_FIRST_STREAM_STATES: u64 = 150_994_980;
-/// txg1、txg2、txg3、最后一条根之后（再加全部持久那一个）。
-const ANCHOR_FIRST_STREAM_BY_PUBLISH: [u64; 4] = [12, 12, 150_994_947, 8];
-const ANCHOR_SECOND_STREAM_SEGMENT_STATES: [u64; 14] = [
-    150_994_943,
-    3,
-    1,
-    8,
-    8,
-    262_143,
-    3,
-    1,
-    589_823,
-    3,
-    1,
-    589_823,
-    3,
-    1,
+// 重跑登记（`research/prompts/e161-r2-prereg.md`）第七节 A2、A3、A4 与第十三节 `anchors_r2.py` 的原样输出（C577 之后的段形）。
+const ANCHOR_FIRST_STREAM_SEGMENT_STATES: [u64; 11] = [8, 3, 1, 8, 3, 1, 8, 16_777_215, 3, 1, 8];
+const ANCHOR_FIRST_STREAM_STATES: u64 = 16_777_260;
+/// txg1、txg2、txg3、最后一条根之后。
+const ANCHOR_FIRST_STREAM_BY_PUBLISH: [u64; 4] = [12, 12, 16_777_227, 8];
+/// 第二条流取样段 11–26 逐段。
+const ANCHOR_SECOND_STREAM_SEGMENT_STATES: [u64; 16] = [
+    16_777_215, 3, 1, 8, 8, 262_143, 3, 1, 8, 65_535, 3, 1, 8, 65_535, 3, 1,
 ];
-const ANCHOR_SECOND_STREAM_STATES: u64 = 152_436_764;
-/// txg4、txg5、txg6、txg7。
-const ANCHOR_SECOND_STREAM_BY_PUBLISH: [u64; 4] = [150_994_947, 262_163, 589_827, 589_827];
-const ANCHOR_FIRST_STREAM_SMALL: u64 = 37;
-const ANCHOR_SECOND_STREAM_SMALL: u64 = 1_441_821;
-const ANCHOR_STRIDE_PER_SEGMENT: u64 = 65_338;
-const ANCHOR_FIRST_STREAM_QUICK: u64 = 54;
-const ANCHOR_SECOND_STREAM_QUICK: u64 = 84;
+const ANCHOR_SECOND_STREAM_STATES: u64 = 17_170_476;
+/// txg4（不含不取的段 10）、txg5、txg6、txg7。
+const ANCHOR_SECOND_STREAM_BY_PUBLISH: [u64; 4] = [16_777_219, 262_163, 65_547, 65_547];
+const ANCHOR_FIRST_STREAM_SMALL: u64 = 45;
+const ANCHOR_SECOND_STREAM_SMALL: u64 = 393_261;
+/// 每个 24 写段 ⌈(2²⁴ − 1) / 2311⌉；两条流取样段里各只有一个 24 写段。
+const ANCHOR_STRIDE_PER_SEGMENT: u64 = 7_260;
+const ANCHOR_FIRST_STREAM_QUICK: u64 = 46;
+const ANCHOR_SECOND_STREAM_QUICK: u64 = 52;
+/// 段头：每条流的大段（24 写段）各取段内序号 0–4095（重跑登记 I-4 新加的一格）。
+const ANCHOR_SEGMENT_HEADS_PER_STREAM: u64 = 4_096;
 /// A5：取三态的写数（第一条流 8 次系统配置槽写，第二条流 46 次）。
 const ANCHOR_FIRST_STREAM_IN_PLACE_OVERWRITES: usize = 8;
 const ANCHOR_SECOND_STREAM_IN_PLACE_OVERWRITES: usize = 46;
@@ -3977,12 +3973,47 @@
     }
 }
 
-/// 一条流的锚点 A1、A2 / A3、A5 与 A4 的域大小；交回全部都对上没有。
+/// 整条流每段按 A1 的公式算的状态数（只用公式，不枚举；重跑登记 I-5）：交回（全量 = 1 + Σ 各段，最大一段）。
+fn whole_stream_states_by_the_formula(
+    stream: &PreparedStream,
+    table: &TornWriteTable,
+) -> (u128, u128) {
+    let per_segment: Vec<u128> = stream
+        .segments
+        .iter()
+        .map(|segment| {
+            let in_place_overwrites = segment
+                .iter()
+                .filter(|write_index| table.is_in_place_overwrite[**write_index])
+                .count();
+            u128::from(formula_segment_state_count(
+                u64::try_from(segment.len()).expect("写数"),
+                u64::try_from(in_place_overwrites).expect("个数"),
+            ))
+        })
+        .collect();
+    let largest = per_segment
+        .iter()
+        .copied()
+        .max()
+        .expect("录制流至少有一段（S2 核过段长表）");
+    (1 + per_segment.iter().sum::<u128>(), largest)
+}
+
+/// 一条流的锚点核下来的两样。
+struct StreamAnchorOutcome {
+    /// A1、A2 / A3、A4、A5 与（第二条流）A7 全部对上。
+    every_anchor_holds: bool,
+    /// 可行性档要评的三个域（D1-small、D1-quick、段头）的大小都等于 A4（重跑登记 I-7 的域守卫只看这一样）。
+    evaluated_domain_sizes_hold: bool,
+}
+
+/// 一条流的锚点 A1、A2 / A3、A5 与 A4 的域大小（第二条流另核 A7 的整条流全量与最大一段）。
 fn report_stream_anchors(
     output: &mut ResultLines,
     stream: &PreparedStream,
     table: &TornWriteTable,
-) -> bool {
+) -> StreamAnchorOutcome {
     let checks = segment_count_checks(stream, table);
     let mut all_three_agree = true;
     for check in &checks {
@@ -4033,6 +4064,7 @@
     let small = u64::try_from(small_domain_ordinals(stream, &full).len()).expect("个数");
     let stride = u64::try_from(stride_domain_ordinals(stream, &full).len()).expect("个数");
     let quick_states = u64::try_from(every_ordinal(stream, &quick).len()).expect("个数");
+    let heads = u64::try_from(strided_segment_heads(stream, &full).len()).expect("个数");
     let (segments_ok, states_ok, publish_ok, small_ok, quick_ok, in_place_ok) = match stream.name {
         StreamName::First => (
             sampled_counts == ANCHOR_FIRST_STREAM_SEGMENT_STATES,
@@ -4057,10 +4089,28 @@
         ),
     };
     let stride_ok = stride == ANCHOR_STRIDE_PER_SEGMENT;
+    let heads_ok = heads == ANCHOR_SEGMENT_HEADS_PER_STREAM;
     let mask_width_formula_holds =
         table.mask_width() == stream.writes.len() + in_place + table.replay_count;
+    let (whole_stream_fields, whole_stream_ok) = match stream.name {
+        StreamName::First => (String::new(), true),
+        StreamName::Second => {
+            let (whole_stream_states, whole_stream_largest) =
+                whole_stream_states_by_the_formula(stream, table);
+            let states_match = whole_stream_states == SECOND_STREAM_FULL_STATES;
+            let largest_match = whole_stream_largest == SECOND_STREAM_LARGEST_SEGMENT_STATES;
+            (
+                format!(
+                    " whole_stream_states={whole_stream_states} whole_stream_largest_segment_states={whole_stream_largest} whole_stream_states_match={} whole_stream_largest_segment_match={}",
+                    bool_text(states_match),
+                    bool_text(largest_match),
+                ),
+                states_match && largest_match,
+            )
+        }
+    };
     output.line(format!(
-        "name=anchor_stream stream={} sampled_segment_states={sampled_counts:?} domain_states={domain_states} by_publish={by_publish:?} d1_small={small} d1_stride={stride} d1_quick={quick_states} recorded_writes={} in_place_overwrites={in_place} replays={} w={} segments_match={} domain_states_match={} by_publish_match={} d1_small_match={} d1_stride_match={} d1_quick_match={} in_place_overwrites_match={} w_formula_holds={}",
+        "name=anchor_stream stream={} sampled_segment_states={sampled_counts:?} domain_states={domain_states} by_publish={by_publish:?} d1_small={small} d1_stride={stride} d1_quick={quick_states} d1_heads={heads} recorded_writes={} in_place_overwrites={in_place} replays={} w={} segments_match={} domain_states_match={} by_publish_match={} d1_small_match={} d1_stride_match={} d1_quick_match={} d1_heads_match={} in_place_overwrites_match={} w_formula_holds={}{whole_stream_fields}",
         stream.name.label(),
         stream.writes.len(),
         table.replay_count,
@@ -4071,18 +4121,24 @@
         bool_text(small_ok),
         bool_text(stride_ok),
         bool_text(quick_ok),
+        bool_text(heads_ok),
         bool_text(in_place_ok),
         bool_text(mask_width_formula_holds),
     ));
-    all_three_agree
-        && segments_ok
-        && states_ok
-        && publish_ok
-        && small_ok
-        && stride_ok
-        && quick_ok
-        && in_place_ok
-        && mask_width_formula_holds
+    StreamAnchorOutcome {
+        every_anchor_holds: all_three_agree
+            && segments_ok
+            && states_ok
+            && publish_ok
+            && small_ok
+            && stride_ok
+            && quick_ok
+            && heads_ok
+            && in_place_ok
+            && mask_width_formula_holds
+            && whole_stream_ok,
+        evaluated_domain_sizes_hold: small_ok && quick_ok && heads_ok,
+    }
 }
 
 /// A6、A7：容量与最大一段的算术。
@@ -4090,7 +4146,7 @@
     LARGEST_CARD_MEBIBYTES * MEBIBYTE == ANCHOR_LARGEST_CARD_BYTES
         && SMALLEST_SIXTEEN_GIGABYTE_CARD_MEBIBYTES * MEBIBYTE == ANCHOR_SIXTEEN_GIGABYTE_CARD_BYTES
         && FIVE_CARDS_MEBIBYTES * MEBIBYTE == ANCHOR_FIVE_CARDS_BYTES
-        && 3u128.pow(2) * 2u128.pow(28) == SECOND_STREAM_LARGEST_SEGMENT_STATES + 1
+        && 2u128.pow(28) == SECOND_STREAM_LARGEST_SEGMENT_STATES + 1
 }
 
 // ───────────────────────────── 模式 stop-clauses：S1–S6 与 A1–A7（跑产物之前） ─────────────────────────────
@@ -4265,7 +4321,7 @@
         ));
     }
     output.line(format!(
-        "name=stop_clause_shared_prefix bases_identical={} first_nine_segments_identical={} clause_S2_holds={}",
+        "name=stop_clause_shared_prefix bases_identical={} shared_prefix_segments={SHARED_PREFIX_SEGMENTS} shared_prefix_segments_identical={} clause_S2_holds={}",
         bool_text(bases_identical),
         bool_text(prefix_identical),
         bool_text(streams_clause_holds)
@@ -4281,7 +4337,7 @@
     let mut walk_subset_clause_holds = true;
     for stream in [&first, &second] {
         let table = TornWriteTable::of(stream);
-        anchors_hold &= report_stream_anchors(output, stream, &table);
+        anchors_hold &= report_stream_anchors(output, stream, &table).every_anchor_holds;
         let context = EvaluationContext::new(stream, &table);
         let full = LocalPlan::new(
             stream,
@@ -4855,7 +4911,41 @@
     holds
 }
 
-/// PC3：第二条流段 14 序号最小的 1000 个状态，先照常跑一遍，再六遍各在一个计时区里加 2 ms 忙等。交回全部六个区都过没过。
+/// PC3 那一段的样子：（写数，全量计划里的状态数，序号最小的 1000 个，这 1000 个是不是都落在那一段）。
+struct TimingControlSegmentShape {
+    writes: usize,
+    states: u64,
+    ordinals: Vec<u64>,
+    every_ordinal_in_the_segment: bool,
+}
+
+impl TimingControlSegmentShape {
+    fn of(stream: &PreparedStream, full: &LocalPlan<'_>) -> Self {
+        let segment = TIMING_CONTROL_SEGMENT;
+        let first = full.first_ordinal_by_segment[segment];
+        let ordinals: Vec<u64> =
+            (first..first + u64::try_from(TIMING_CONTROL_STATES).expect("1000")).collect();
+        let every_ordinal_in_the_segment = ordinals
+            .iter()
+            .all(|ordinal| full.segment_of(*ordinal) == segment);
+        Self {
+            writes: stream.segments[segment].len(),
+            states: full.count_by_segment[segment],
+            ordinals,
+            every_ordinal_in_the_segment,
+        }
+    }
+
+    /// 重跑登记 I-6：那一段写数是 18、状态数不少于 1000、取的 1000 个序号都落在那一段。
+    fn holds(&self) -> bool {
+        self.writes == TIMING_CONTROL_SEGMENT_WRITES
+            && self.states >= u64::try_from(TIMING_CONTROL_STATES).expect("1000")
+            && self.every_ordinal_in_the_segment
+    }
+}
+
+/// PC3：第二条流段 16 序号最小的 1000 个状态，先照常跑一遍，再六遍各在一个计时区里加 2 ms 忙等。交回全部六个区都过没过。
+/// 那一段不是写行那 18 个单元写、或不够 1000 个状态时，六个区一律判不过、一个状态都不评（`pc3_segment_holds=false`）。
 fn report_timing_control(
     output: &mut ResultLines,
     stream: &PreparedStream,
@@ -4868,10 +4958,19 @@
         &table,
         domain_expansions(stream, LocalExpansion::EveryCombination),
     );
-    let segment = TIMING_CONTROL_SEGMENT;
-    let first = full.first_ordinal_by_segment[segment];
-    let ordinals: Vec<u64> =
-        (first..first + u64::try_from(TIMING_CONTROL_STATES).expect("1000")).collect();
+    let shape = TimingControlSegmentShape::of(stream, &full);
+    if !shape.holds() {
+        for busy_zone in TIMING_ZONES {
+            output.line(format!(
+                "name=positive_control_timing busy_zone={} states=0 segment={TIMING_CONTROL_SEGMENT} segment_writes={} segment_states={} pc3_segment_holds=false pc3_zone_holds=false",
+                busy_zone.label(),
+                shape.writes,
+                shape.states,
+            ));
+        }
+        return false;
+    }
+    let ordinals = shape.ordinals;
     let zone_totals = |busy_zone: Option<TimingZone>| -> [u128; 6] {
         let totals = evaluate_stream_domain(&StreamDomainRun {
             label: format!("pc3-{}", busy_zone.map_or("none", TimingZone::label)),
@@ -4914,7 +5013,7 @@
             .map(|(zone, change)| format!("{}_delta_elapsed_ns={change}", zone.label()))
             .collect();
         output.line(format!(
-            "name=positive_control_timing busy_zone={} states={} {} busy_zone_added_at_least_1800ms={} other_zones_changed_under_1000ms={} pc3_zone_holds={}",
+            "name=positive_control_timing busy_zone={} states={} {} busy_zone_added_at_least_1800ms={} other_zones_changed_under_1000ms={} pc3_segment_holds=true pc3_zone_holds={}",
             busy_zone.label(),
             ordinals.len(),
             change_fields.join(" "),
@@ -4926,8 +5025,10 @@
     every_zone_holds
 }
 
-/// PC3 取的段（5.3：第二条流段 14）。
-const TIMING_CONTROL_SEGMENT: usize = 14;
+/// PC3 取的段（5.3：第 1 次第二条流段 14「写行发布 18 个单元写」那一段；C577 之后同一批写是段 16，重跑登记 I-6、A9）。
+const TIMING_CONTROL_SEGMENT: usize = 16;
+/// PC3 那一段的写数：写行发布的 18 个单元写（重跑登记 A9）。
+const TIMING_CONTROL_SEGMENT_WRITES: usize = 18;
 
 // ───────────────────────────── 几何敏感性与判别力自证（8.2） ─────────────────────────────
 
@@ -5236,10 +5337,10 @@
 
 // ───────────────────────────── 模式 feasibility：可行性档（用户 2026-09-27 定，登记 12.2） ─────────────────────────────
 
-/// 可行性档里 26 写段取段内序号最小的这么多个状态（用户 2026-09-27 定，登记 12.2）。
+/// 可行性档里大段（24 写段）取段内序号最小的这么多个状态（用户 2026-09-27 定，登记 12.2；重跑登记 1.3「段头」）。
 const FEASIBILITY_HEAD_STATES: u64 = 4_096;
 
-/// 取样段里 26 写段的段头：段内序号 0 到 4095（按全量计划的序号）。
+/// 取样段里大段（24 写段）的段头：段内序号 0 到 4095（按全量计划的序号）。
 fn strided_segment_heads(stream: &PreparedStream, plan: &LocalPlan<'_>) -> Vec<u64> {
     let mut ordinals = Vec::new();
     for (segment_index, segment) in stream.segments.iter().enumerate() {
@@ -5253,7 +5354,7 @@
     ordinals
 }
 
-/// S4 在 26 写段段头上的形态：crates 按段枚举要把整段 1.5 亿个跑完，这里改用 crates 的单状态入口
+/// S4 在大段段头上的形态：crates 按段枚举要把整段 1677 万个跑完，这里改用 crates 的单状态入口
 /// `evaluate_state_for_versions`，在装置给的同一批持久集合上逐个评，计数与装置照跑的逐项比。交回相不相同。
 fn report_per_state_crates_comparison(
     output: &mut ResultLines,
@@ -5312,8 +5413,9 @@
     holds
 }
 
-/// 可行性档（用户 2026-09-27 定）：第一条流 D1-small、两条流的 D1-quick、两个 26 写段各取段头 4096 个状态；
+/// 可行性档（用户 2026-09-27 定）：第一条流 D1-small、两条流的 D1-quick、两个大段（24 写段）各取段头 4096 个状态；
 /// 停机条款、阳性对照与 crates 的对拍都只在这个小域上做。G 行一律不判（只跑了可行性档）。
+/// 要评的三个域的大小先与 A4 比，对不上一个状态都不评、退 3（重跑登记 I-7 的域守卫）。
 #[allow(
     clippy::too_many_lines,
     reason = "可行性档的全部格按登记 12.2 的次序排下来：S1、S2、锚点、与 crates 对拍、两遍对照、各格结果行、阳性对照、判定行"
@@ -5321,7 +5423,7 @@
 fn run_feasibility(output: &mut ResultLines) -> i32 {
     let threads = threads_from_environment();
     output.line(format!(
-        "name=config experiment=E161 mode=feasibility threads={threads} registration=research/prompts/e161-preregistration.md domain=first_small_37+quick_54_84+strided_segment_heads_4096"
+        "name=config experiment=E161 mode=feasibility threads={threads} registration=research/prompts/e161-preregistration.md rerun_registration=research/prompts/e161-r2-prereg.md domain=first_small_45+quick_46_52+largest_segment_heads_4096"
     ));
     let constants = local_constants_against_the_crates();
     let constants_clause_holds = constants
@@ -5356,30 +5458,48 @@
         ));
     }
     output.line(format!(
-        "name=stop_clause_shared_prefix bases_identical={} first_nine_segments_identical={} clause_S2_holds={}",
+        "name=stop_clause_shared_prefix bases_identical={} shared_prefix_segments={SHARED_PREFIX_SEGMENTS} shared_prefix_segments_identical={} clause_S2_holds={}",
         bool_text(bases_identical),
         bool_text(prefix_identical),
         bool_text(streams_clause_holds)
     ));
+    let first_table = TornWriteTable::of(&first);
+    let second_table = TornWriteTable::of(&second);
+    let streams_and_tables = [(&first, &first_table), (&second, &second_table)];
     let mut anchors_hold = capacity_anchors_hold();
+    let mut evaluated_domain_sizes_hold = true;
+    for (stream, table) in streams_and_tables {
+        let anchors = report_stream_anchors(output, stream, table);
+        anchors_hold &= anchors.every_anchor_holds;
+        evaluated_domain_sizes_hold &= anchors.evaluated_domain_sizes_hold;
+    }
+    // 域守卫（重跑登记 I-7）：要评的域与 A4 对不上时一个状态都不评。第 1 次的段形一变，「写数少于 26」把 1677 万个状态的段整段收进
+    // D1-small，装置照样往下评、撞内存上限才停；先核域大小，对不上就打判决行、退 3（停机条款 S9）。
+    if !evaluated_domain_sizes_hold {
+        output.line(format!(
+            "name=verdict scope=feasibility clause_S1_holds={} clause_S2_holds={} anchors_a1_to_a7_hold={} evaluated_domain_sizes_hold=false states_evaluated=false",
+            bool_text(constants_clause_holds),
+            bool_text(streams_clause_holds),
+            bool_text(anchors_hold),
+        ));
+        return 3;
+    }
     let mut enumeration_clause_holds = true;
     let mut tally_clause_holds = true;
     let mut determinism_clause_holds = true;
     let mut walk_subset_clause_holds = true;
     let mut positive_controls: Vec<(String, bool)> = Vec::new();
-    for stream in [&first, &second] {
+    for (stream, table) in streams_and_tables {
         let name = stream.name.label();
-        let table = TornWriteTable::of(stream);
-        anchors_hold &= report_stream_anchors(output, stream, &table);
-        let context = EvaluationContext::new(stream, &table);
+        let context = EvaluationContext::new(stream, table);
         let full = LocalPlan::new(
             stream,
-            &table,
+            table,
             domain_expansions(stream, LocalExpansion::EveryCombination),
         );
         let quick = LocalPlan::new(
             stream,
-            &table,
+            table,
             domain_expansions(
                 stream,
                 LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll,
@@ -5534,7 +5654,10 @@
         }
     }
     let timing_control_holds = report_timing_control(output, &second, threads);
-    positive_controls.push(("second_segment_14_pc3".to_string(), timing_control_holds));
+    positive_controls.push((
+        format!("second_segment_{TIMING_CONTROL_SEGMENT}_pc3"),
+        timing_control_holds,
+    ));
     let control_fields: Vec<String> = positive_controls
         .iter()
         .map(|(name, holds)| format!("{name}={}", bool_text(*holds)))
@@ -5569,13 +5692,15 @@
 mod tests {
     use super::*;
 
-    /// 第一条流的 D1-small（37 个状态）跑一遍 5.1 的四件事；各测试共用这一份搭建的写法。
+    /// 第一条流的 D1-small（45 个状态）跑一遍 5.1 的四件事；各测试共用这一份搭建的写法。
+    /// 评之前先核 D1-small 的个数（重跑登记 I-7 的域守卫）：段形或阈值一变，D1-small 会把 1677 万个状态的段整段收进来，
+    /// 不先核就一路评到撞内存上限。
     fn evaluate_the_first_stream_small_domain(
         threads: usize,
         busy_zone: Option<TimingZone>,
     ) -> (PreparedStream, StreamDomainTotals, Vec<u64>) {
         let stream = prepare_first_stream();
-        let totals = {
+        let (totals, small) = {
             let table = TornWriteTable::of(&stream);
             let context = EvaluationContext::new(&stream, &table);
             let plan = LocalPlan::new(
@@ -5584,8 +5709,13 @@
                 domain_expansions(&stream, LocalExpansion::EveryCombination),
             );
             let small = small_domain_ordinals(&stream, &plan);
+            assert_eq!(
+                small.len(),
+                45,
+                "域守卫：第一条流 D1-small 是 45 个状态（重跑登记 A4），对不上就不评"
+            );
             let control: BTreeSet<u64> = small.iter().copied().collect();
-            evaluate_stream_domain(&StreamDomainRun {
+            let totals = evaluate_stream_domain(&StreamDomainRun {
                 label: "test-first-small".to_string(),
                 context: &context,
                 plan: &plan,
@@ -5594,16 +5724,8 @@
                 busy_zone,
                 threads,
                 depth: EvaluationDepth::FourSteps,
-            })
-        };
-        let small = {
-            let table = TornWriteTable::of(&stream);
-            let plan = LocalPlan::new(
-                &stream,
-                &table,
-                domain_expansions(&stream, LocalExpansion::EveryCombination),
-            );
-            small_domain_ordinals(&stream, &plan)
+            });
+            (totals, small)
         };
         (stream, totals, small)
     }
@@ -5627,14 +5749,7 @@
         let stream = prepare_first_stream();
         let table = TornWriteTable::of(&stream);
         let checks = segment_count_checks_up_to(&stream, &table, 1 << 20);
-        let counted: Vec<u64> = checks
-            .iter()
-            .map(|check| check.counted_by_the_plan)
-            .collect();
-        assert_eq!(
-            counted, ANCHOR_FIRST_STREAM_SEGMENT_STATES,
-            "A2：装置计划逐段数"
-        );
+        // 先 A1 两半、后 A2（重跑登记 I-8）：装置计划改坏时头一个断的是「装置对 crates」，crates 那一列因此被比到。
         for check in &checks {
             assert_eq!(
                 check.counted_by_the_plan, check.counted_by_the_crates,
@@ -5647,6 +5762,14 @@
                 check.segment_index
             );
         }
+        let counted: Vec<u64> = checks
+            .iter()
+            .map(|check| check.counted_by_the_plan)
+            .collect();
+        assert_eq!(
+            counted, ANCHOR_FIRST_STREAM_SEGMENT_STATES,
+            "A2：装置计划逐段数"
+        );
         assert_eq!(
             table.mask_width(),
             49,
@@ -5657,6 +5780,11 @@
             150_994_943,
             "3² · 2²⁴ − 1"
         );
+        assert_eq!(
+            formula_segment_state_count(24, 0),
+            16_777_215,
+            "2²⁴ − 1：C577 之后第一条流段 7 的 24 个单元写"
+        );
     }
 
     #[test]
@@ -5678,25 +5806,118 @@
         );
         assert_eq!(
             small_domain_ordinals(&stream, &full).len(),
-            37,
+            45,
             "A4：D1-small 第一条流"
         );
         assert_eq!(
             stride_domain_ordinals(&stream, &full).len(),
-            65_338,
-            "A4：D1-stride 每个 26 写段"
+            7_260,
+            "A4：D1-stride 第一条流唯一的 24 写段（段 7）"
         );
         assert_eq!(
             every_ordinal(&stream, &quick).len(),
-            54,
+            46,
             "A4：D1-quick 第一条流"
         );
+        assert_eq!(
+            strided_segment_heads(&stream, &full).len(),
+            4_096,
+            "A4：段头，第一条流段 7 序号最小的 4096 个"
+        );
+    }
+
+    /// 重跑登记 I-10：只搭第二条流、建计划，一个状态都不评。
+    #[test]
+    fn second_stream_shape_and_sampled_domains_have_the_registered_sizes() {
+        let first = prepare_first_stream();
+        let stream = prepare_second_stream();
+        assert_eq!(stream.segments.len(), 78, "A8：第二条流 78 段");
+        assert_eq!(stream.writes.len(), 481, "A8：第二条流 481 次写");
+        assert_eq!(stream.root_write_count, 19, "A8：第二条流 19 条根");
+        let shape = check_stream_shape(&stream);
+        assert!(shape.segment_lengths_match, "S2：段长表与本地常量逐段相同");
+        assert!(shape.writes_match, "S2：写数与本地常量相同");
+        assert!(shape.roots_match, "S2：根数与本地常量相同");
+        let (bases_identical, prefix_identical) = shared_prefix_is_identical(&first, &stream);
+        assert!(bases_identical, "S2：两条流的基镜像逐字节相同");
+        assert!(
+            prefix_identical,
+            "S2：第二条流前 11 段的写与第一条流 11 段逐字节相同"
+        );
+        let table = TornWriteTable::of(&stream);
+        let full = LocalPlan::new(
+            &stream,
+            &table,
+            domain_expansions(&stream, LocalExpansion::EveryCombination),
+        );
+        let quick = LocalPlan::new(
+            &stream,
+            &table,
+            domain_expansions(
+                &stream,
+                LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll,
+            ),
+        );
+        let sampled_counts: Vec<u64> = full
+            .count_by_segment
+            .iter()
+            .enumerate()
+            .filter(|(segment_index, _count)| is_sampled_segment(stream.name, *segment_index))
+            .map(|(_segment_index, count)| *count)
+            .collect();
+        assert_eq!(
+            sampled_counts, ANCHOR_SECOND_STREAM_SEGMENT_STATES,
+            "A3：取样段 11–26 逐段状态数"
+        );
+        assert_eq!(
+            small_domain_ordinals(&stream, &full).len(),
+            393_261,
+            "A4：D1-small 第二条流"
+        );
+        assert_eq!(
+            stride_domain_ordinals(&stream, &full).len(),
+            7_260,
+            "A4：D1-stride 第二条流唯一的 24 写段（段 11）"
+        );
+        assert_eq!(
+            every_ordinal(&stream, &quick).len(),
+            52,
+            "A4：D1-quick 第二条流"
+        );
+        assert_eq!(
+            strided_segment_heads(&stream, &full).len(),
+            4_096,
+            "A4：段头，第二条流段 11 序号最小的 4096 个"
+        );
+        let timing_control = TimingControlSegmentShape::of(&stream, &full);
+        assert_eq!(
+            timing_control.writes, 18,
+            "A9：PC3 那一段是写行发布的 18 个单元写"
+        );
+        assert_eq!(
+            timing_control.states, 262_143,
+            "A9：PC3 那一段 2¹⁸ − 1 个状态"
+        );
+        assert!(
+            timing_control.holds(),
+            "I-6：PC3 取的 1000 个序号都落在那一段"
+        );
+        let (whole_stream_states, whole_stream_largest) =
+            whole_stream_states_by_the_formula(&stream, &table);
+        assert_eq!(
+            whole_stream_states, 1_678_377_204,
+            "A7：第二条流整条流全量（公式，1 + Σ 各段）"
+        );
+        assert_eq!(
+            whole_stream_largest, 268_435_455,
+            "A7：第二条流整条流最大一段 2²⁸ − 1"
+        );
     }
 
     #[test]
     fn recorded_recovery_answers_exactly_like_the_baseline_on_the_first_stream() {
         let (_stream, totals, _small) = evaluate_the_first_stream_small_domain(2, None);
-        assert_eq!(totals.states(), 37, "D1-small 第一条流 37 个状态");
+        assert_eq!(totals.states(), 45, "D1-small 第一条流 45 个状态");
         assert_eq!(
             totals.consult_report_differs, 0,
             "S5：记录跑看 journal 那一遍与照跑逐项相同"
@@ -5741,7 +5962,7 @@
         assert_eq!(
             summary.arm_inconsistent[Arm::RecoveryConsult.slot()],
             0,
-            "按读集复用在这 37 个状态上没有不一致"
+            "按读集复用在这 45 个状态上没有不一致"
         );
     }
 
@@ -5783,8 +6004,8 @@
         };
         let added = zone_total(&busy, 0).saturating_sub(zone_total(&quiet, 0));
         assert!(
-            added >= 37 * TIMING_CONTROL_BUSY_WAIT.as_nanos() * 9 / 10,
-            "PC3 的缩小版：看 journal 那一区多出 37 × 2 ms 的九成以上，实际 {added} ns"
+            added >= 45 * TIMING_CONTROL_BUSY_WAIT.as_nanos() * 9 / 10,
+            "PC3 的缩小版：看 journal 那一区多出 45 × 2 ms 的九成以上，实际 {added} ns"
         );
     }
 
````

### y：mutations-append.tsv

````
E161 crash_state_dedup 大段写数改回 26（M13，A4 与域守卫该红）	crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs	const STRIDED_SEGMENT_WRITES: usize = 24;	const STRIDED_SEGMENT_WRITES: usize = 26;	-p singlefs-checker-tier --bin e161_crash_state_dedup_and_time_split	stride_and_quick_domains_have_the_registered_sizes
E161 crash_state_dedup PC3 的段改回 14（M14，I-10 该红）	crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs	const TIMING_CONTROL_SEGMENT: usize = 16;	const TIMING_CONTROL_SEGMENT: usize = 14;	-p singlefs-checker-tier --bin e161_crash_state_dedup_and_time_split	second_stream_shape_and_sampled_domains_have_the_registered_sizes
E161 crash_state_dedup 第二条流第一个取样段 11 改成 10（M15，I-10 该红）	crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs	const SECOND_STREAM_FIRST_SAMPLED_SEGMENT: usize = 11;	const SECOND_STREAM_FIRST_SAMPLED_SEGMENT: usize = 10;	-p singlefs-checker-tier --bin e161_crash_state_dedup_and_time_split	second_stream_shape_and_sampled_domains_have_the_registered_sizes
E161 crash_state_dedup 装置公式里的 3 换成 2（M16，A1 crates 对公式该红）	crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs	    3u64.pow(u32::try_from(in_place_overwrites).expect("m"))	    2u64.pow(u32::try_from(in_place_overwrites).expect("m"))	-p singlefs-checker-tier --bin e161_crash_state_dedup_and_time_split	first_stream_plan_counts_match_the_formula_the_crates_and_the_anchor
````

## 实现员 p1：crates.patch（/tmp/claude-1000/m3-batch18/p1/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
index 728ac6e..01e9b2f 100644
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -81,7 +81,11 @@ pub const SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY: &str =
 ///
 /// 第 69 条（下标 68）是 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同，用户 2026-10-04 定新立，里程碑三第 9 批），同样只往末尾追加；
 /// GPU 判器内核还没判它（`crash_judge_gpu` 那一侧要补，归 GPU 那一簇）。
-pub const IMPLEMENTED_INVARIANTS: [&str; 69] = [
+///
+/// 第 70–74 条（下标 69–73）是里程碑三·3 删除线那五条（判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节，
+/// 用户 2026-10-05 定 I-4.4、I-4.6 取乙）：I-9.5、I-3.4、I-4.7、I-4.6、I-4.4，同样只往末尾追加；GPU 判器内核还没判它们
+/// （`crash_judge_gpu` 那一侧要补，归 GPU 那一簇）。
+pub const IMPLEMENTED_INVARIANTS: [&str; 74] = [
     "I-1.1",
     "I-1.2",
     "I-1.3",
@@ -151,6 +155,11 @@ pub const IMPLEMENTED_INVARIANTS: [&str; 69] = [
     HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
     FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
     TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
+    DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+    PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
+    INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
+    DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+    INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
 ];
 
 /// I-1.12：走读跟随的一条指针，偏移 34 的出生树等于被指单元头里的出生树——码 1、码 3 取头偏移 43；码 2 取哪个字段随 C289 定，
@@ -209,6 +218,23 @@ pub const FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK:
 /// （条目偏移 0）的条目，诞生 txg 相同。与 I-9.14 的分工：I-9.14 只比同一条时间线上的根，这一条不分时间线（树 ID 被重发时
 /// 新线与老线上同一个 ID 的树诞生 txg 不同，C342（树 ID 水位在根读不出时退回去重发））。
 pub const TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT: &str = "I-9.28";
+/// I-9.5（删除队列里的号仍在树里），两个方向：① `deleted_inodes` 里的每个号在同一版的 inode 树里有记录且 nlink = 0；
+/// ② inode 树里 nlink = 0 的每条记录，号都在同一版的 `deleted_inodes` 里（队列 = inode 树里 nlink = 0 的号集合，
+/// D8（核心索引结构） 已定项 16 第 3 条；C708）。判法（判哪几版、经头 ID 配对、读不全报射程里判不了）是下面四条的共用判法。
+pub const DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES: &str = "I-9.5";
+/// I-3.4（可用空间扣待删占用） 只落「记账标签 4 恒 0」那一句：判的每一版里走全了的记账树都有 (4, 0, 0xFFFF_FFFF) 那一行、值为 0，缺行判红
+/// （判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 F5 三格）。
+pub const PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO: &str = "I-3.4";
+/// I-4.7（删除中 inode 有账），对象取丙：inode 树里 nlink = 0 的记录的号，与 extent 树里有数据单元、而 inode 树里没有 nlink ≥ 1 记录的号；
+/// 这些号要么在 `deleted_inodes` 里，要么一个数据单元都不剩（同一判决第二节 F1）。
+pub const INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT: &str = "I-4.7";
+/// I-4.6（中间态其余不变量成立） 取乙：每条 nlink = 0 的记录，extent 树里它的数据单元数与 size 对得上——size 为 0 时 0 或 1 个，
+/// size > 0 时 ⌈size ÷ 数据单元净荷容量⌉（用户 2026-10-05 定，岔路单 `research/prompts/m3-delete-checker-r1-forks.md`）。
+pub const DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE: &str = "I-4.6";
+/// I-4.4（意图只描述目标态）：第一版报不适用（用户 2026-10-05 定，岔路单同上），理由见 [`INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION`]。
+pub const INTENT_DESCRIBES_ONLY_THE_TARGET_STATE: &str = "I-4.4";
+/// I-4.4 在第一版报不适用的理由，照 `.claude/kb/invariants.md` I-4.4 那一行的状态列。
+pub const INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION: &str = "第一版不适用：意图载体（nlink = 0 的记录、`deleted_inodes` 的 key、墓碑）没有可装增量的字段；logged_ops 有字段表时实现";
 
 /// 判定累加器：每条不变量记评估了几次、第一处违例、整条不适用的理由，以及射程里有一部分判不了的理由。
 #[derive(Default)]
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 7889e48..ca434f5 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -38,16 +38,20 @@ use crate::image::{
     verified_system_configuration_slots, ImageReader, InvariantVerdict, Judgements,
     PointerLocation, PointerView, PoolGeometry,
     ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
+    DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+    DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
     DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
     FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
     HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
+    INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
     INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
     INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
+    INTENT_DESCRIBES_ONLY_THE_TARGET_STATE, INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION,
     JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
     LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH, MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
-    NAMED_ENTRY_MATCHES_THE_UNIT_HEADER, POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
-    PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES, REFERENCE_INTERVALS_ARE_EXACT,
-    SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
+    NAMED_ENTRY_MATCHES_THE_UNIT_HEADER, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
+    POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER, PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
+    REFERENCE_INTERVALS_ARE_EXACT, SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
     TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
     TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
     UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
@@ -154,8 +158,21 @@ const STATISTIC_FREE_BYTES: u16 = 2;
 const STATISTIC_DEFER_QUEUE_BYTES: u16 = 5;
 /// inode 号水位（D5（快照 / 空间记账机制） 已定项 4 第 12 项的行号）。
 const STATISTIC_INODE_WATERMARK: u16 = 12;
+/// 待删除但意图未完成占用（D5（快照 / 空间记账机制） 已定项 4 第 4 项的行号，不带设备维）：恒 0 的保留位，I-3.4（可用空间扣待删占用） 判它。
+const STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES: u16 = 4;
+/// 记账行里值那 8 字节的偏移：key 22（统计量 2 + 树 8 + 设备 4 + 代 8）之后（D8（核心索引结构） 已定项 11 的记账条目 34）。
+const ACCOUNTING_ROW_VALUE_OFFSET: usize = 22;
+/// 记账行里设备段的偏移（统计量 2 + 树 8 之后）。
+const ACCOUNTING_ROW_DEVICE_OFFSET: usize = 2 + 8;
 /// 不带设备维的统计量把设备段写成这个保留值（D5（快照 / 空间记账机制） 已定项 10）。
 const STATISTIC_NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;
+/// inode 记录里 nlink 的偏移（D8（核心索引结构） 已定项 6 字段表「24 mode / uid / gid / nlink，各 4」，4 字节；
+/// `.claude/kb/layout/03-third-txn.md`「inode 记录的 nlink 第一次为 0」那一行）。
+const INODE_RECORD_NLINK_OFFSET: usize = 36;
+/// 墓碑记录里对象 ID 的偏移（`.claude/kb/layout/03-third-txn.md`「二、墓碑记录」：偏移 0、8 字节，被删的 inode 号）。
+const TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET: usize = 0;
+/// 码 3 单元头里写序的实例代号（D18（块里携带什么信息） 已定项 11 的类身份段表：写序 93 起，实例代号 4 字节在前）。
+const PACKED_UNIT_WRITE_ORDER_INSTANCE_OFFSET: usize = 93;
 /// inode 记录 140 里 `size` 与 `blocks` 两个字段的偏移（D8（核心索引结构） 已定项 6 的偏移表），按字段表另写一份、不用实现的解析。
 const INODE_RECORD_SIZE_OFFSET: usize = 40;
 /// inode 记录里对象出生代（偏移 8）与改动计数（偏移 88）的位置（D8（核心索引结构） 已定项 6 的字段表）：I-9.26 比这两个与容器诞生代号。
@@ -4071,6 +4088,33 @@ fn extent_tree_without_judging(
     root_pointer: &PointerView,
     cache: &mut IndexNodeCache,
 ) -> Option<(Vec<PointerView>, Vec<PointerView>)> {
+    extent_tree_with_the_inode_of_each_data_pointer_without_judging(reader, root_pointer, cache)
+        .map(|extent_tree| {
+            (
+                extent_tree.node_pointers,
+                extent_tree
+                    .data_pointers_of_inodes
+                    .into_iter()
+                    .map(|(_, data_pointer)| data_pointer)
+                    .collect(),
+            )
+        })
+}
+
+/// [`extent_tree_with_the_inode_of_each_data_pointer_without_judging`] 读出的一棵 extent 树：每个节点的指针（根在内），
+/// 与每个数据指针连同它属于哪个 inode 号。
+struct ExtentTreeReadWithoutJudging {
+    node_pointers: Vec<PointerView>,
+    data_pointers_of_inodes: Vec<(u64, PointerView)>,
+}
+
+/// [`extent_tree_without_judging`] 那一遍，每个数据指针带上它属于哪个 inode 号（上段叶条目的 inode 号；下段叶记录属于引它那条上段条目的号）：
+/// 删除态扫描按号数数据单元（I-4.7（删除中 inode 有账）、I-4.6（中间态其余不变量成立））。
+fn extent_tree_with_the_inode_of_each_data_pointer_without_judging(
+    reader: &dyn ImageReader,
+    root_pointer: &PointerView,
+    cache: &mut IndexNodeCache,
+) -> Option<ExtentTreeReadWithoutJudging> {
     /// 待读的一个节点：上段 (层级, 序号)，或某个 inode 下段 (层级, 序号)；根那一个层级取它自己头里的。
     enum Pending {
         Upper(PointerView, Option<(u8, u64)>),
@@ -4118,23 +4162,23 @@ fn extent_tree_without_judging(
                             ));
                         }
                         ExtentUpperLeafEntryView::InlineDataUnit {
+                            inode,
                             pointer: inline_data_pointer,
-                            ..
                         } => {
-                            data_pointers.push(parse_data_pointer(&inline_data_pointer));
+                            data_pointers.push((inode, parse_data_pointer(&inline_data_pointer)));
                         }
                         ExtentUpperLeafEntryView::Malformed => return None,
                     }
                 }
             }
-            (Some(_), 0) => {
+            (Some(inode), 0) => {
                 if node.entry_width < extent_leaf_record_bytes() {
                     return None;
                 }
                 data_pointers.extend(
                     node.entries
                         .iter()
-                        .map(|record| parse_data_pointer(&record[24..112])),
+                        .map(|record| (inode, parse_data_pointer(&record[24..112]))),
                 );
             }
             (None, _) => {
@@ -4165,7 +4209,10 @@ fn extent_tree_without_judging(
             }
         }
     }
-    Some((node_pointers, data_pointers))
+    Some(ExtentTreeReadWithoutJudging {
+        node_pointers,
+        data_pointers_of_inodes: data_pointers,
+    })
 }
 
 /// 一棵树的根节点下面还引用了什么：extent 树与分配记录树整棵按位置读（每个节点、数据指针、分配记录）、
@@ -7815,6 +7862,737 @@ fn slots_referenced_per_device(
     slots_per_device
 }
 
+/// 删除态扫描要判的一版（I-9.5（删除队列里的号仍在树里） 那一行的共用判法：回退候选集里每条根，加由记录施加出来的那几版）：
+/// 称呼（判红与判不了时写进说明）与它的树表指针（根环里的根取根记录偏移 36 起的 86 字节，由记录施加出来的那一版取记录新根段里的那一条）。
+struct VersionToScanTheDeletionStateOf<'record> {
+    name: String,
+    tree_table_pointer: &'record [u8],
+}
+
+/// 删除态扫描在一版上读出的一条 inode 记录里要的两段（D8（核心索引结构） 已定项 6 字段表：nlink 偏移 36、size 偏移 40）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct InodeRecordOfTheDeletionScan {
+    nlink: u32,
+    size_in_bytes: u64,
+}
+
+/// 删除态扫描在一版上读不全、配不上的是哪一样：那一版在射程里判不了，报 `part_of_the_range_not_judged`、不判红
+/// （判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 F4-b、F4-c）。读不出、解不开、形状不对由主走读判
+/// （I-2.1、I-1.1、I-1.7、I-1.10、I-9.1、I-9.27 等），这一遍只如实记「判不了」，不进 `walk_failures`。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum DeletionScanGap {
+    TreeTableUnreadable,
+    TreeTableEntryShorterThanTheFieldTable,
+    DeletedInodesHeadIdentifierNamesNoSingleInodeTree,
+    DeletedInodesRootUnreadable,
+    InodeTreeUnreadable,
+    ExtentTreeOfTheInodeTreeNotSingle,
+    ExtentTreeUnreadable,
+    AccountingTreeUnreadable,
+    TombstoneTreeUnreadable,
+}
+
+impl DeletionScanGap {
+    fn reason(self) -> &'static str {
+        match self {
+            DeletionScanGap::TreeTableUnreadable => {
+                "走读到的某一版树表读不出（两份都对不上整单元校验和或解不开）：那一版的删除态射程里判不了"
+            }
+            DeletionScanGap::TreeTableEntryShorterThanTheFieldTable => {
+                "走读到的某一版树表条目宽不到字段表的 200 字节：那一版的删除态射程里判不了"
+            }
+            DeletionScanGap::DeletedInodesHeadIdentifierNamesNoSingleInodeTree => {
+                "走读到的某一版 deleted_inodes 条目偏移 116 的头 ID 配不上同一张树表里恰好一条 inode 树条目：那一版判不了"
+            }
+            DeletionScanGap::DeletedInodesRootUnreadable => {
+                "走读到的某一版 deleted_inodes 根读不出，或不是层级 0、key 宽与条目宽 8 的码 2 根：那一版的队列判不了"
+            }
+            DeletionScanGap::InodeTreeUnreadable => {
+                "走读到的某一版 inode 树读不全（节点或叶容器读不出、层级或条目宽不对、类型段不认得）：那一版的记录判不了"
+            }
+            DeletionScanGap::ExtentTreeOfTheInodeTreeNotSingle => {
+                "走读到的某一版树表里头 ID 指着这棵 inode 树的 extent 树条目不是恰好一条：那一版的数据单元数判不了"
+            }
+            DeletionScanGap::ExtentTreeUnreadable => {
+                "走读到的某一版 extent 树读不全：那一版每个号的数据单元数判不了"
+            }
+            DeletionScanGap::AccountingTreeUnreadable => {
+                "走读到的某一版记账树读不全（节点读不出、层级或条目宽不对）：那一版的记账行判不了"
+            }
+            DeletionScanGap::TombstoneTreeUnreadable => {
+                "所选根的树表或墓碑树读不全：水位大于墓碑对象 ID 那一半判不了"
+            }
+        }
+    }
+}
+
+/// 删除态扫描读 I-9.5、I-4.7、I-4.6 三条要的那一组：一版里任一样判不了，三条在那一版都判不了。
+const DELETION_QUEUE_SCAN_INVARIANTS: [&str; 3] = [
+    DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+    INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
+    DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+];
+
+/// 删除态扫描读过的码 3 单元：解开的容器与头里写序的实例代号（I-9.6 墓碑那一半按已发布谓词滤容器要它）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct PackedUnitOfTheDeletionScan {
+    view: crate::PackedUnitView,
+    write_order_instance: u32,
+}
+
+/// 删除态扫描里读过的码 3 单元：(设备身份, 起点槽, 位置条目里的整单元校验和) → 解开的单元（解不开记 None），同 [`IndexNodeCache`]：
+/// 候选集里几版常共用同一片 inode 叶容器，按位置条目记一份，省掉重复的整单元 CRC。
+type PackedUnitCache = BTreeMap<(u32, u64, u32), Option<PackedUnitOfTheDeletionScan>>;
+
+fn read_packed_unit_without_judging(
+    reader: &dyn ImageReader,
+    pointer: &PointerView,
+    cache: &mut PackedUnitCache,
+) -> Option<PackedUnitOfTheDeletionScan> {
+    if pointer.all_zero {
+        return None;
+    }
+    let key = (
+        pointer.locations[0].device,
+        pointer.locations[0].slot,
+        pointer.locations[0].checksum,
+    );
+    if let Some(cached) = cache.get(&key) {
+        return cached.clone();
+    }
+    let parsed = read_unit_without_judging(reader, pointer, data_unit_bytes()).and_then(|unit| {
+        packed_unit_view(&unit)
+            .ok()
+            .map(|view| PackedUnitOfTheDeletionScan {
+                view,
+                write_order_instance: read_u32(&unit, PACKED_UNIT_WRITE_ORDER_INSTANCE_OFFSET),
+            })
+    });
+    cache.insert(key, parsed.clone());
+    parsed
+}
+
+/// 一版的 `deleted_inodes` 号集，只读不判（D8（核心索引结构） 已定项 16：码 2、层级 0 的一个根，key = inode 号 8 字节、无 value；
+/// 队列空时树表条目的根指针置零）。
+fn deleted_inodes_queue_without_judging(
+    reader: &dyn ImageReader,
+    root_pointer: &PointerView,
+    cache: &mut IndexNodeCache,
+) -> Result<BTreeSet<u64>, DeletionScanGap> {
+    if root_pointer.all_zero {
+        return Ok(BTreeSet::new());
+    }
+    let root = read_index_node_without_judging(reader, root_pointer, cache)
+        .ok_or(DeletionScanGap::DeletedInodesRootUnreadable)?;
+    if root.level != 0
+        || root.key_width != DELETED_INODES_ENTRY_BYTES
+        || root.entry_width < DELETED_INODES_ENTRY_BYTES
+    {
+        return Err(DeletionScanGap::DeletedInodesRootUnreadable);
+    }
+    Ok(root
+        .entries
+        .iter()
+        .map(|entry| read_u64(entry, 0))
+        .collect())
+}
+
+/// 一版 inode 树的全部记录（号 → nlink 与 size），只读不判：根层级 0 是零条目根（一条记录都没有，D8（核心索引结构） 已定项 6、16 第 5 条），
+/// 层级 ≥ 1 的往下走（[`collect_inode_records_below`]）。
+fn inode_records_without_judging(
+    reader: &dyn ImageReader,
+    root_pointer: &PointerView,
+    index_cache: &mut IndexNodeCache,
+    packed_cache: &mut PackedUnitCache,
+) -> Result<BTreeMap<u64, InodeRecordOfTheDeletionScan>, DeletionScanGap> {
+    let mut records = BTreeMap::new();
+    if root_pointer.all_zero {
+        return Ok(records);
+    }
+    let root = read_index_node_without_judging(reader, root_pointer, index_cache)
+        .ok_or(DeletionScanGap::InodeTreeUnreadable)?;
+    if root.level == 0 {
+        return if root.entries.is_empty() {
+            Ok(records)
+        } else {
+            Err(DeletionScanGap::InodeTreeUnreadable)
+        };
+    }
+    collect_inode_records_below(reader, &root, index_cache, packed_cache, &mut records)?;
+    Ok(records)
+}
+
+/// inode 树一个内部节点下面的记录逐条收进 `records`：类型段 2 的叶容器（码 3 打包记录类型 2，记录宽 140）读它的每条记录，
+/// 类型段 0 的内部节点（层级 = 父层级 − 1，且 ≥ 1）往下走。读不出、形状不对的交回 [`DeletionScanGap::InodeTreeUnreadable`]。
+/// 同一个号出现两条时后读到的那条盖掉先读到的（号的唯一性由主走读的 I-9.4、I-1.1 判）。递归的上界：每下一层层级减一。
+fn collect_inode_records_below(
+    reader: &dyn ImageReader,
+    node: &crate::IndexNodeView,
+    index_cache: &mut IndexNodeCache,
+    packed_cache: &mut PackedUnitCache,
+    records: &mut BTreeMap<u64, InodeRecordOfTheDeletionScan>,
+) -> Result<(), DeletionScanGap> {
+    if node.entry_width < inode_internal_entry_bytes() {
+        return Err(DeletionScanGap::InodeTreeUnreadable);
+    }
+    let inode_record_bytes =
+        registered_record_width(PACKED_TYPE_INODE).expect("类型 2 登记了记录宽 140");
+    for entry in &node.entries {
+        let child_pointer = parse_node_pointer(&entry[34..120]);
+        match InodeInternalEntryType::of(read_u16(entry, 16)) {
+            InodeInternalEntryType::LeafContainer => {
+                let container =
+                    read_packed_unit_without_judging(reader, &child_pointer, packed_cache)
+                        .ok_or(DeletionScanGap::InodeTreeUnreadable)?;
+                if container.view.record_type != PACKED_TYPE_INODE
+                    || container.view.record_width < inode_record_bytes
+                {
+                    return Err(DeletionScanGap::InodeTreeUnreadable);
+                }
+                for record in &container.view.records {
+                    records.insert(
+                        read_u64(record, 0),
+                        InodeRecordOfTheDeletionScan {
+                            nlink: read_u32(record, INODE_RECORD_NLINK_OFFSET),
+                            size_in_bytes: read_u64(record, INODE_RECORD_SIZE_OFFSET),
+                        },
+                    );
+                }
+            }
+            InodeInternalEntryType::InternalNode => {
+                if child_pointer.all_zero {
+                    return Err(DeletionScanGap::InodeTreeUnreadable);
+                }
+                let child = read_index_node_without_judging(reader, &child_pointer, index_cache)
+                    .ok_or(DeletionScanGap::InodeTreeUnreadable)?;
+                if node.level.checked_sub(1) != Some(child.level) || child.level == 0 {
+                    return Err(DeletionScanGap::InodeTreeUnreadable);
+                }
+                collect_inode_records_below(reader, &child, index_cache, packed_cache, records)?;
+            }
+            InodeInternalEntryType::Unregistered(_) => {
+                return Err(DeletionScanGap::InodeTreeUnreadable);
+            }
+        }
+    }
+    Ok(())
+}
+
+/// 一版 extent 树里每个 inode 号的数据单元数，只读不判：上段叶条目内联的那一个数一个，下段叶的每条 extent 叶记录数一个；
+/// 全零的数据指针不指单元，不数。根指针全零的树一个单元都没有。
+fn data_unit_count_of_each_inode_without_judging(
+    reader: &dyn ImageReader,
+    root_pointer: &PointerView,
+    cache: &mut IndexNodeCache,
+) -> Result<BTreeMap<u64, u64>, DeletionScanGap> {
+    let mut data_units_of_each_inode: BTreeMap<u64, u64> = BTreeMap::new();
+    if root_pointer.all_zero {
+        return Ok(data_units_of_each_inode);
+    }
+    let extent_tree = extent_tree_with_the_inode_of_each_data_pointer_without_judging(
+        reader,
+        root_pointer,
+        cache,
+    )
+    .ok_or(DeletionScanGap::ExtentTreeUnreadable)?;
+    for (inode, _) in extent_tree
+        .data_pointers_of_inodes
+        .iter()
+        .filter(|(_, data_pointer)| !data_pointer.all_zero)
+    {
+        *data_units_of_each_inode.entry(*inode).or_insert(0) += 1;
+    }
+    Ok(data_units_of_each_inode)
+}
+
+/// 一版记账树的全部行，只读不判（D8（核心索引结构） 已定项 11：多层码 2 树，叶条目 34、内部条目 = key 22 + 子指针 86）：
+/// 走全了才交回，有一个节点读不出、key 宽或条目宽不对、孩子层级不是父层级减一都交回 [`DeletionScanGap::AccountingTreeUnreadable`]。
+/// 递归的上界：每下一层层级减一。
+fn collect_accounting_rows_without_judging(
+    reader: &dyn ImageReader,
+    node_pointer: &PointerView,
+    cache: &mut IndexNodeCache,
+    rows: &mut Vec<Vec<u8>>,
+) -> Result<(), DeletionScanGap> {
+    let node = read_index_node_without_judging(reader, node_pointer, cache)
+        .ok_or(DeletionScanGap::AccountingTreeUnreadable)?;
+    let key_width = KEY_SCHEMA_ACCOUNTING.width();
+    if node.key_width != key_width {
+        return Err(DeletionScanGap::AccountingTreeUnreadable);
+    }
+    if node.level == 0 {
+        if node.entry_width < accounting_entry_bytes() {
+            return Err(DeletionScanGap::AccountingTreeUnreadable);
+        }
+        rows.extend(node.entries.iter().cloned());
+        return Ok(());
+    }
+    if node.entry_width < internal_entry_bytes(KEY_SCHEMA_ACCOUNTING) {
+        return Err(DeletionScanGap::AccountingTreeUnreadable);
+    }
+    for entry in &node.entries {
+        let child_pointer = parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]);
+        if child_pointer.all_zero {
+            return Err(DeletionScanGap::AccountingTreeUnreadable);
+        }
+        let child_level = read_index_node_without_judging(reader, &child_pointer, cache)
+            .ok_or(DeletionScanGap::AccountingTreeUnreadable)?
+            .level;
+        if node.level.checked_sub(1) != Some(child_level) {
+            return Err(DeletionScanGap::AccountingTreeUnreadable);
+        }
+        collect_accounting_rows_without_judging(reader, &child_pointer, cache, rows)?;
+    }
+    Ok(())
+}
+
+/// 删除态扫描（判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 F4、第七节骨架）：要判的每一版（回退候选集里每条根，
+/// 加由记录施加出来的那几版；树表单元相同的几版内容相同，只扫一次）各自重读一遍树表与几棵树，只读、不进 `walk_failures`，
+/// 判 I-3.4（[`judge_pending_deletion_occupancy_of_a_version`]）与 I-9.5、I-4.7、I-4.6（[`judge_deletion_queue_of_a_version`]）。
+/// 主走读按 `visited_units` 只走一遍共享子树，这几条要的是「这一版自己的」队列、记录与单元数，所以另走一遍（与 [`references_of_root`] 同一个缘故）。
+/// 树表指针全零、树表 0 条的那一版没有这几棵树，不判。
+fn judge_deletion_state_of_the_versions(
+    reader: &dyn ImageReader,
+    versions: &[VersionToScanTheDeletionStateOf<'_>],
+    index_cache: &mut IndexNodeCache,
+    judgements: &mut Judgements,
+) {
+    let mut packed_cache = PackedUnitCache::new();
+    let mut tree_tables_scanned: BTreeSet<(u32, u64, u32)> = BTreeSet::new();
+    for version in versions {
+        let tree_table_pointer = parse_node_pointer(version.tree_table_pointer);
+        if tree_table_pointer.all_zero {
+            continue;
+        }
+        let first_location = tree_table_pointer.locations[0];
+        if !tree_tables_scanned.insert((
+            first_location.device,
+            first_location.slot,
+            first_location.checksum,
+        )) {
+            continue;
+        }
+        let gap = match read_index_node_without_judging(reader, &tree_table_pointer, index_cache) {
+            None => Some(DeletionScanGap::TreeTableUnreadable),
+            Some(tree_table) if tree_table.entries.is_empty() => None,
+            Some(tree_table)
+                if tree_table
+                    .entries
+                    .iter()
+                    .any(|entry| entry.len() < tree_table_entry_bytes()) =>
+            {
+                Some(DeletionScanGap::TreeTableEntryShorterThanTheFieldTable)
+            }
+            Some(tree_table) => {
+                judge_pending_deletion_occupancy_of_a_version(
+                    reader,
+                    &version.name,
+                    &tree_table.entries,
+                    index_cache,
+                    judgements,
+                );
+                judge_deletion_queue_of_a_version(
+                    reader,
+                    &version.name,
+                    &tree_table.entries,
+                    index_cache,
+                    &mut packed_cache,
+                    judgements,
+                );
+                None
+            }
+        };
+        if let Some(gap) = gap {
+            for invariant in DELETION_QUEUE_SCAN_INVARIANTS
+                .into_iter()
+                .chain([PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO])
+            {
+                judgements.part_of_the_range_not_judged(invariant, gap.reason());
+            }
+        }
+    }
+    // 一次都没判到时的理由（判到过的照判定报，`Judgements::not_applicable` 不改已经评估过的）。
+    judgements.not_applicable(
+        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+        "走读到的每一版树表里都没有 deleted_inodes 条目（树表指针全零或 0 条）：没有队列可比",
+    );
+    judgements.not_applicable(
+        PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
+        "走读到的每一版树表里都没有记账树（树表指针全零或 0 条）",
+    );
+    judgements.not_applicable(
+        INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
+        "走读到的每一版里都没有处于删除中的 inode（nlink = 0 的记录、有数据单元而没有 nlink ≥ 1 记录的号）",
+    );
+    judgements.not_applicable(
+        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+        "走读到的每一版 inode 树里都没有 nlink = 0 的记录",
+    );
+}
+
+/// I-3.4（可用空间扣待删占用） 只落「记账标签 4 恒 0」（判决第二节 F5 三格）：这一版树表里每一棵记账树（种类 4）走全了，
+/// 要有 (4, 树, 0xFFFF_FFFF, 代) 那一行（缺行判红），每一行标签 4 的值都是 0；读不全的报射程里判不了。根指针全零的不判（没有记账树）。
+fn judge_pending_deletion_occupancy_of_a_version(
+    reader: &dyn ImageReader,
+    version_name: &str,
+    tree_table_entries: &[Vec<u8>],
+    index_cache: &mut IndexNodeCache,
+    judgements: &mut Judgements,
+) {
+    let invariant = PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO;
+    for accounting_entry in tree_table_entries
+        .iter()
+        .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_ACCOUNTING)
+    {
+        let tree = read_u64(accounting_entry, 0);
+        let root_pointer = parse_node_pointer(&accounting_entry[14..100]);
+        if root_pointer.all_zero {
+            continue;
+        }
+        let mut rows = Vec::new();
+        if let Err(gap) =
+            collect_accounting_rows_without_judging(reader, &root_pointer, index_cache, &mut rows)
+        {
+            judgements.part_of_the_range_not_judged(invariant, gap.reason());
+            continue;
+        }
+        let pending_deletion_rows: Vec<&Vec<u8>> = rows
+            .iter()
+            .filter(|row| read_u16(row, 0) == STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES)
+            .collect();
+        let has_the_pending_deletion_row = pending_deletion_rows.iter().any(|row| {
+            read_u32(row, ACCOUNTING_ROW_DEVICE_OFFSET) == STATISTIC_NO_DEVICE_DIMENSION
+        });
+        judgements.judge(invariant, has_the_pending_deletion_row, || {
+            format!(
+                "{version_name}的记账树（树 {tree}）走全了，没有待删除但意图未完成占用（标签 {STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES}、设备段 {STATISTIC_NO_DEVICE_DIMENSION:#x}）那一行"
+            )
+        });
+        for row in pending_deletion_rows {
+            let value = read_u64(row, ACCOUNTING_ROW_VALUE_OFFSET);
+            judgements.judge(invariant, value == 0, || {
+                format!(
+                    "{version_name}的记账树（树 {tree}）里待删除但意图未完成占用（标签 {STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES}、设备段 {:#x}）那一行的值是 {value}，不是 0",
+                    read_u32(row, ACCOUNTING_ROW_DEVICE_OFFSET)
+                )
+            });
+        }
+    }
+}
+
+/// I-9.5（两个方向）、I-4.7（对象取丙）、I-4.6（取乙）在一版上判：这一版树表里每一条 `deleted_inodes` 条目经偏移 116 的头 ID 配同一张表里
+/// 树 ID 等于它的那一条 inode 树条目（判决第二节 F4-b；配不上、配到不止一条记判不了），extent 树同样按头 ID 配这棵 inode 树。
+/// I-9.5 每一版每个方向各判一次（队列与 nlink = 0 的号集合两两比，干净的一版也真比过）；I-4.7 每个处于删除中的号判一次、
+/// I-4.6 每条 nlink = 0 的记录判一次（一个都没有时没有对象）。
+fn judge_deletion_queue_of_a_version(
+    reader: &dyn ImageReader,
+    version_name: &str,
+    tree_table_entries: &[Vec<u8>],
+    index_cache: &mut IndexNodeCache,
+    packed_cache: &mut PackedUnitCache,
+    judgements: &mut Judgements,
+) {
+    let kind_of = |entry: &&Vec<u8>| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET);
+    let head_identifier_of =
+        |entry: &&Vec<u8>| read_u64(entry, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
+    for deleted_inodes_entry in tree_table_entries
+        .iter()
+        .filter(|entry| kind_of(entry) == TREE_KIND_DELETED_INODES)
+    {
+        let inode_tree = head_identifier_of(&deleted_inodes_entry);
+        let inode_tree_entries: Vec<&Vec<u8>> = tree_table_entries
+            .iter()
+            .filter(|entry| kind_of(entry) == TREE_KIND_INODE && read_u64(entry, 0) == inode_tree)
+            .collect();
+        let read_back = match inode_tree_entries.as_slice() {
+            [inode_tree_entry] => deleted_inodes_queue_without_judging(
+                reader,
+                &parse_node_pointer(&deleted_inodes_entry[14..100]),
+                index_cache,
+            )
+            .and_then(|queue| {
+                inode_records_without_judging(
+                    reader,
+                    &parse_node_pointer(&inode_tree_entry[14..100]),
+                    index_cache,
+                    packed_cache,
+                )
+                .map(|records| (queue, records))
+            }),
+            [] | [_, _, ..] => {
+                Err(DeletionScanGap::DeletedInodesHeadIdentifierNamesNoSingleInodeTree)
+            }
+        };
+        let (queue, records) = match read_back {
+            Ok(queue_and_records) => queue_and_records,
+            Err(gap) => {
+                for invariant in DELETION_QUEUE_SCAN_INVARIANTS {
+                    judgements.part_of_the_range_not_judged(invariant, gap.reason());
+                }
+                continue;
+            }
+        };
+        judge_the_deletion_queue_against_the_inode_records(
+            version_name,
+            inode_tree,
+            &queue,
+            &records,
+            judgements,
+        );
+        let extent_tree_entries: Vec<&Vec<u8>> = tree_table_entries
+            .iter()
+            .filter(|entry| {
+                kind_of(entry) == TREE_KIND_EXTENT && head_identifier_of(entry) == inode_tree
+            })
+            .collect();
+        let data_units_of_each_inode = match extent_tree_entries.as_slice() {
+            [extent_tree_entry] => data_unit_count_of_each_inode_without_judging(
+                reader,
+                &parse_node_pointer(&extent_tree_entry[14..100]),
+                index_cache,
+            ),
+            [] | [_, _, ..] => Err(DeletionScanGap::ExtentTreeOfTheInodeTreeNotSingle),
+        };
+        match data_units_of_each_inode {
+            Ok(data_units_of_each_inode) => {
+                judge_inodes_being_deleted_against_the_queue_and_their_data_units(
+                    version_name,
+                    &queue,
+                    &records,
+                    &data_units_of_each_inode,
+                    judgements,
+                );
+                judge_data_unit_counts_of_the_inodes_being_deleted_against_their_size(
+                    version_name,
+                    &records,
+                    &data_units_of_each_inode,
+                    judgements,
+                );
+            }
+            Err(gap) => {
+                for invariant in [
+                    INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
+                    DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+                ] {
+                    judgements.part_of_the_range_not_judged(invariant, gap.reason());
+                }
+            }
+        }
+    }
+}
+
+/// I-9.5 两个方向，一版各判一次：① 队列里的每个号在这棵 inode 树里有记录且 nlink = 0；② nlink = 0 的每条记录，号在队列里。
+fn judge_the_deletion_queue_against_the_inode_records(
+    version_name: &str,
+    inode_tree: u64,
+    queue: &BTreeSet<u64>,
+    records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>,
+    judgements: &mut Judgements,
+) {
+    let invariant = DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES;
+    let first_queued_inode_without_a_record_of_nlink_zero = queue
+        .iter()
+        .find(|inode| records.get(inode).is_none_or(|record| record.nlink != 0));
+    judgements.judge(
+        invariant,
+        first_queued_inode_without_a_record_of_nlink_zero.is_none(),
+        || {
+            let inode = first_queued_inode_without_a_record_of_nlink_zero
+                .expect("判红时队列里有一个号没有 nlink = 0 的记录");
+            match records.get(inode) {
+                None => format!(
+                    "{version_name}：deleted_inodes 里的号 {inode} 在同一版的 inode 树（树 {inode_tree}）里没有记录"
+                ),
+                Some(record) => format!(
+                    "{version_name}：deleted_inodes 里的号 {inode} 在同一版的 inode 树（树 {inode_tree}）里的记录 nlink = {}，不是 0",
+                    record.nlink
+                ),
+            }
+        },
+    );
+    let first_record_of_nlink_zero_outside_the_queue = records
+        .iter()
+        .find(|(inode, record)| record.nlink == 0 && !queue.contains(inode))
+        .map(|(inode, _)| *inode);
+    judgements.judge(
+        invariant,
+        first_record_of_nlink_zero_outside_the_queue.is_none(),
+        || {
+            let inode = first_record_of_nlink_zero_outside_the_queue
+                .expect("判红时有一条 nlink = 0 的记录不在队列里");
+            format!(
+                "{version_name}：inode 树（树 {inode_tree}）里 inode {inode} 的记录 nlink = 0，号不在同一版的 deleted_inodes 里"
+            )
+        },
+    );
+}
+
+/// I-4.7，对象取丙（判决第二节 F1）：nlink = 0 的记录的号，加上 extent 树里有数据单元、而 inode 树里没有 nlink ≥ 1 记录的号；
+/// 每个这样的号要么在队列里，要么一个数据单元都不剩。
+fn judge_inodes_being_deleted_against_the_queue_and_their_data_units(
+    version_name: &str,
+    queue: &BTreeSet<u64>,
+    records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>,
+    data_units_of_each_inode: &BTreeMap<u64, u64>,
+    judgements: &mut Judgements,
+) {
+    let mut inodes_being_deleted: BTreeSet<u64> = records
+        .iter()
+        .filter(|(_, record)| record.nlink == 0)
+        .map(|(inode, _)| *inode)
+        .collect();
+    inodes_being_deleted.extend(
+        data_units_of_each_inode
+            .iter()
+            .filter(|(inode, data_units)| {
+                **data_units > 0 && !records.get(inode).is_some_and(|record| record.nlink >= 1)
+            })
+            .map(|(inode, _)| *inode),
+    );
+    for inode in inodes_being_deleted {
+        let data_units_left = data_units_of_each_inode.get(&inode).copied().unwrap_or(0);
+        judgements.judge(
+            INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
+            queue.contains(&inode) || data_units_left == 0,
+            || {
+                let why_being_deleted = match records.get(&inode) {
+                    Some(_) => "记录 nlink = 0",
+                    None => "inode 树里没有它的记录",
+                };
+                format!(
+                    "{version_name}：inode {inode} 处于删除中（{why_being_deleted}），extent 树里还有 {data_units_left} 个数据单元，号也不在 deleted_inodes 里"
+                )
+            },
+        );
+    }
+}
+
+/// I-4.6 取乙：每条 nlink = 0 的记录，数据单元数与 size 对得上——size 为 0 时 0 或 1 个，size > 0 时 ⌈size ÷ 数据单元净荷容量⌉
+/// （净荷容量 checker 自己算一份，[`data_unit_payload_capacity_in_bytes`]；实现那一侧同一式在 `crates/singlefs-core/src/write_request_split.rs`）。
+fn judge_data_unit_counts_of_the_inodes_being_deleted_against_their_size(
+    version_name: &str,
+    records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>,
+    data_units_of_each_inode: &BTreeMap<u64, u64>,
+    judgements: &mut Judgements,
+) {
+    let payload_capacity_in_bytes = data_unit_payload_capacity_in_bytes();
+    for (inode, record) in records.iter().filter(|(_, record)| record.nlink == 0) {
+        let data_units = data_units_of_each_inode.get(inode).copied().unwrap_or(0);
+        let size_in_bytes = record.size_in_bytes;
+        let data_units_the_size_asks_for = size_in_bytes.div_ceil(payload_capacity_in_bytes);
+        let matches_the_size = if size_in_bytes == 0 {
+            data_units <= 1
+        } else {
+            data_units == data_units_the_size_asks_for
+        };
+        judgements.judge(
+            DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+            matches_the_size,
+            || {
+                format!(
+                    "{version_name}：inode {inode} 的记录 nlink = 0、size {size_in_bytes}，按净荷容量 {payload_capacity_in_bytes} 要 {} 个数据单元，extent 树里有 {data_units} 个",
+                    if size_in_bytes == 0 {
+                        "0 或 1".to_string()
+                    } else {
+                        data_units_the_size_asks_for.to_string()
+                    }
+                )
+            },
+        );
+    }
+}
+
+/// I-9.6 墓碑那一半：所选根（最新的那条根）的树表里墓碑树（种类 10）整棵读、不判，交回已发布的（按所选根指着的实例表判，
+/// [`PublishedPredicate`]）墓碑容器里全部记录的对象 ID 最大值，一条都没有时 `Ok(None)`。被崩溃恢复抛弃的时间线上写的墓碑
+/// 不在所选根的墓碑树里（`.claude/kb/invariants.md` I-9.6 那一行）。墓碑树根之下：类型段 1 的条目指码 3 墓碑容器，
+/// 类型段 0 的指下一层码 2 节点（层级 = 父层级 − 1，且 ≥ 1）；读不全交回 [`DeletionScanGap::TombstoneTreeUnreadable`]。
+fn largest_published_tombstone_object_identifier(
+    reader: &dyn ImageReader,
+    tree_table_pointer_of_the_chosen_root: &[u8],
+    published: &PublishedPredicate<'_>,
+    index_cache: &mut IndexNodeCache,
+) -> Result<Option<u64>, DeletionScanGap> {
+    let tree_table_pointer = parse_node_pointer(tree_table_pointer_of_the_chosen_root);
+    if tree_table_pointer.all_zero {
+        return Ok(None);
+    }
+    let tree_table = read_index_node_without_judging(reader, &tree_table_pointer, index_cache)
+        .ok_or(DeletionScanGap::TombstoneTreeUnreadable)?;
+    if tree_table
+        .entries
+        .iter()
+        .any(|entry| entry.len() < tree_table_entry_bytes())
+    {
+        return Err(DeletionScanGap::TombstoneTreeUnreadable);
+    }
+    let mut packed_cache = PackedUnitCache::new();
+    let mut largest_object_identifier: Option<u64> = None;
+    let mut pending_nodes: Vec<PointerView> = tree_table
+        .entries
+        .iter()
+        .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_TOMBSTONE)
+        .map(|entry| parse_node_pointer(&entry[14..100]))
+        .filter(|root_pointer| !root_pointer.all_zero)
+        .collect();
+    let tombstone_record_bytes =
+        registered_record_width(PACKED_TYPE_TOMBSTONE).expect("类型 1 登记了记录宽 40");
+    // 迭代上界：每个码 2 节点只从它的父条目入队一次，类型段 0 的孩子层级比父低一、层级 0 不入队，所以入队的节点数有限。
+    while let Some(node_pointer) = pending_nodes.pop() {
+        let node = read_index_node_without_judging(reader, &node_pointer, index_cache)
+            .ok_or(DeletionScanGap::TombstoneTreeUnreadable)?;
+        if node.level == 0 || node.entry_width < inode_internal_entry_bytes() {
+            return Err(DeletionScanGap::TombstoneTreeUnreadable);
+        }
+        for entry in &node.entries {
+            let child_pointer = parse_node_pointer(&entry[34..120]);
+            match read_u16(entry, 16) {
+                0 => {
+                    let child_level =
+                        read_index_node_without_judging(reader, &child_pointer, index_cache)
+                            .ok_or(DeletionScanGap::TombstoneTreeUnreadable)?
+                            .level;
+                    if node.level.checked_sub(1) != Some(child_level) || child_level == 0 {
+                        return Err(DeletionScanGap::TombstoneTreeUnreadable);
+                    }
+                    pending_nodes.push(child_pointer);
+                }
+                PACKED_TYPE_TOMBSTONE => {
+                    let container =
+                        read_packed_unit_without_judging(reader, &child_pointer, &mut packed_cache)
+                            .ok_or(DeletionScanGap::TombstoneTreeUnreadable)?;
+                    if container.view.record_type != PACKED_TYPE_TOMBSTONE
+                        || container.view.record_width < tombstone_record_bytes
+                    {
+                        return Err(DeletionScanGap::TombstoneTreeUnreadable);
+                    }
+                    let is_published = published.holds_for(
+                        WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit {
+                            birth_txg: container.view.birth_txg,
+                            instance: container.write_order_instance,
+                        },
+                    );
+                    if !is_published {
+                        continue;
+                    }
+                    for record in &container.view.records {
+                        let object_identifier =
+                            read_u64(record, TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET);
+                        largest_object_identifier = Some(
+                            largest_object_identifier.map_or(object_identifier, |largest| {
+                                largest.max(object_identifier)
+                            }),
+                        );
+                    }
+                }
+                _type_segment_outside_the_tombstone_tree => {
+                    return Err(DeletionScanGap::TombstoneTreeUnreadable);
+                }
+            }
+        }
+    }
+    Ok(largest_object_identifier)
+}
+
 /// 池级 checker 的入口：每条第一版不变量都报，没评估到的报「不适用」并带理由。
 #[must_use]
 pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {
@@ -8425,10 +9203,43 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
         &mut index_node_cache,
         &mut judgements,
     );
+    // 删除态扫描（I-9.5、I-3.4、I-4.7、I-4.6；判哪几版、配对与读不全的报法照 I-9.5 那一行的共用判法，判决
+    // `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 F4）：回退候选集里每条根与由记录施加出来的那几版，各自重读一遍（只读）。
+    let versions_to_scan_the_deletion_state_of: Vec<VersionToScanTheDeletionStateOf<'_>> =
+        candidate_indexes
+            .iter()
+            .map(|index| {
+                let root = &roots[*index].2;
+                VersionToScanTheDeletionStateOf {
+                    name: format!("根（实例 {}、txg {}）", root.instance, root.checkpoint_txg),
+                    tree_table_pointer: &root.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],
+                }
+            })
+            .chain(versions_applied_only_by_records.iter().map(|version| {
+                VersionToScanTheDeletionStateOf {
+                    name: format!(
+                        "由记录施加出来的那一版（实例 {}、txg {}）",
+                        version.instance, version.checkpoint_txg
+                    ),
+                    tree_table_pointer: version.tree_table_and_mapping_root_pointers().0,
+                }
+            }))
+            .collect();
+    judge_deletion_state_of_the_versions(
+        reader,
+        &versions_to_scan_the_deletion_state_of,
+        &mut index_node_cache,
+        &mut judgements,
+    );
+    // I-4.4（意图只描述目标态）：第一版的意图载体没有可装增量的字段，报不适用并写理由（用户 2026-10-05 定）。
+    judgements.not_applicable(
+        INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
+        INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION,
+    );
     // I-9.6（水位大于两处最大号）：记账里那条「inode 号水位」要大于遍历侧算出的 inode 树内最大 key。
     // 两条独立路径——水位是发布路径在记账树里写下的一个数，最大 key 是 checker 逐片叶容器逐条记录数出来的。
-    // 另一半（> 全部已发布的墓碑记录的对象 ID）今天没有对象：墓碑是打包记录类型 1，这一版一片都不写
-    // （没有删除，C118（`deleted_inodes` 树的形态无落点） 未定形态）。
+    // 另一半（> 全部已发布的墓碑记录的对象 ID）接在这一判后面：墓碑是打包记录类型 1，意图那次发布往墓碑树开放容器追加一条
+    // （D8（核心索引结构） 已定项 16 第 1 条；`deleted_inodes` 的形态已定，C118 还清）。
     let inode_number_watermark = accounting
         .get(&(STATISTIC_INODE_WATERMARK, STATISTIC_NO_DEVICE_DIMENSION))
         .copied();
@@ -8451,6 +9262,25 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
             judgements.not_applicable("I-9.6", "最新的根下面走不到 inode 树里的任何一条记录")
         }
     }
+    // I-9.6 墓碑那一半：水位 > 所选根的墓碑树里全部已发布的（按实例表判）墓碑记录的对象 ID；一条墓碑都没有时没有对象。
+    if let Some(watermark) = inode_number_watermark {
+        match largest_published_tombstone_object_identifier(
+            reader,
+            &roots[newest_index].2.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],
+            &published,
+            &mut index_node_cache,
+        ) {
+            Ok(Some(largest_tombstone_object)) => {
+                judgements.judge("I-9.6", watermark > largest_tombstone_object, || {
+                    format!(
+                        "记账里的 inode 号水位 {watermark} 不大于所选根（txg {newest_txg}）的墓碑树里已发布的墓碑记录的最大对象 ID {largest_tombstone_object}"
+                    )
+                });
+            }
+            Ok(None) => {}
+            Err(gap) => judgements.part_of_the_range_not_judged("I-9.6", gap.reason()),
+        }
+    }
     if !walk.inode_tree_walked {
         judgements.not_applicable(
             "I-9.12",
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index c0b1a0c..463ce91 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -3,6 +3,8 @@
 //! 被改的单元重封校验和，再把新的整单元校验和沿引用链一路补到根记录（父节点里的位置条目、中央映射条目、根槽的自证校验和）。
 
 mod common;
+mod common_deletion_in_progress;
+mod common_pointer_head_corruption;
 
 use common::{
     abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
@@ -18,6 +20,7 @@ use singlefs_checker::image::{
     HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
     INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
     INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
+    INTENT_DESCRIBES_ONLY_THE_TARGET_STATE, INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION,
     JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
     LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH, NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
     POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER, PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
@@ -1834,6 +1837,10 @@ fn expected_verdict_with_one_record_per_transaction(invariant: &str) -> Invarian
         "I-7.9" => InvariantVerdict::NotApplicable(NO_ROOT_RAISED_THE_FLOOR),
         // 最新根之上没有记录：重放之后那一版就是最新根。
         "I-4.1" => InvariantVerdict::NotApplicable(NOTHING_APPLIED_ABOVE_THE_CHOSEN_ROOT),
+        // 用它的镜像上都没有删除：没有处于删除中的号、没有 nlink = 0 的记录；I-4.4 第一版报不适用。
+        "I-4.7" => InvariantVerdict::NotApplicable(NO_INODE_BEING_DELETED_IN_ANY_WALKED_VERSION),
+        "I-4.6" => InvariantVerdict::NotApplicable(NO_RECORD_OF_NLINK_ZERO_IN_ANY_WALKED_VERSION),
+        "I-4.4" => InvariantVerdict::NotApplicable(INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION),
         _ => InvariantVerdict::Holds,
     }
 }
@@ -2912,6 +2919,14 @@ fn devices_whose_own_device_numbers_are_swapped_redden_only_the_own_device_numbe
     );
 }
 
+/// 清单里不要求坏镜像的几条：I-4.4（意图只描述目标态） 第一版报不适用（用户 2026-10-05 定，`.claude/kb/milestone/03-third-txn.md` 三·3 那一行
+/// 「I-4.4 不要求坏镜像与变异行」）。
+const INVARIANTS_WITHOUT_A_BAD_IMAGE: [&str; 1] = [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE];
+/// 干净镜像上 I-4.7（删除中 inode 有账） 与 I-4.6（中间态其余不变量成立） 没有对象时报的理由（`walk` 的删除态扫描）。
+const NO_INODE_BEING_DELETED_IN_ANY_WALKED_VERSION: &str = "走读到的每一版里都没有处于删除中的 inode（nlink = 0 的记录、有数据单元而没有 nlink ≥ 1 记录的号）";
+const NO_RECORD_OF_NLINK_ZERO_IN_ANY_WALKED_VERSION: &str =
+    "走读到的每一版 inode 树里都没有 nlink = 0 的记录";
+
 #[test]
 fn the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target() {
     let clean = build_pool("known-bad").memory_pool();
@@ -2948,6 +2963,18 @@ fn the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target()
             // 这一份上最新根（txg 3）之上没有记录：重放之后那一版就是它。阳性对照与坏镜像在「发布 B 的记录落了、根槽没落」那一份崩溃镜像上
             // （`a_replayed_version_whose_tree_table_does_not_read_back_reddens_only_the_replayed_version_invariant`）。
             "I-4.1" => InvariantVerdict::NotApplicable(NOTHING_APPLIED_ABOVE_THE_CHOSEN_ROOT),
+            // 这一份上没有删除：inode 1 的记录 nlink 1、队列空，没有处于删除中的号。阳性对照与坏镜像在删除中间那几版上
+            // （`pool_checker_judges_the_deletion_state_of_every_walked_version.rs`，坏镜像登记在下面那一串 `.chain` 里）。
+            "I-4.7" => {
+                InvariantVerdict::NotApplicable(NO_INODE_BEING_DELETED_IN_ANY_WALKED_VERSION)
+            }
+            "I-4.6" => {
+                InvariantVerdict::NotApplicable(NO_RECORD_OF_NLINK_ZERO_IN_ANY_WALKED_VERSION)
+            }
+            // 第一版报不适用并写理由（用户 2026-10-05 定），不要求坏镜像（`INVARIANTS_WITHOUT_A_BAD_IMAGE`）。
+            "I-4.4" => {
+                InvariantVerdict::NotApplicable(INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION)
+            }
             _ => InvariantVerdict::Holds,
         };
         assert_eq!(found, expected, "干净镜像上 {invariant} 的判定");
@@ -2984,15 +3011,23 @@ fn the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target()
         .chain(known_bad_images_of_the_ring_geometry_and_the_root_slot_alignment().iter())
         .chain(known_bad_images_of_the_tail_slot_witnesses().iter())
         .chain(known_bad_images_of_the_replayed_version().iter())
+        .chain(common_deletion_in_progress::known_bad_images_after_the_intent_of_a_three_unit_file().iter())
+        .chain(
+            common_deletion_in_progress::known_bad_images_after_the_first_batch_of_a_three_unit_file()
+                .iter(),
+        )
+        .chain(common_deletion_in_progress::known_bad_images_after_one_new_inode().iter())
+        .chain(common_deletion_in_progress::known_bad_images_of_the_pending_deletion_occupancy().iter())
         .map(|(invariant, _)| *invariant)
         .collect();
     let listed: std::collections::BTreeSet<&str> = singlefs_checker::image::IMPLEMENTED_INVARIANTS
         .iter()
         .copied()
+        .filter(|invariant| !INVARIANTS_WITHOUT_A_BAD_IMAGE.contains(invariant))
         .collect();
     assert_eq!(
         targets, listed,
-        "第一版每条不变量都至少有一份坏镜像，也没有清单之外的"
+        "第一版每条不变量（`INVARIANTS_WITHOUT_A_BAD_IMAGE` 那几条除外）都至少有一份坏镜像，也没有清单之外的"
     );
     for (invariant, mutation) in cases {
         let mut image = clean.clone();
@@ -3417,7 +3452,8 @@ fn reused_unit_referenced_only_by_the_newest_root_makes_the_reuse_invariant_red(
 /// （字段表宽 34），别的自描述字段跟着调成自洽的一份（见 `narrow_the_entry_width_to_the_key_width`）。
 ///
 /// 跟着变的只有四条**不适用**：走读在判红的那一步停下、不按字段表解那 15 行记账条目，于是
-/// I-3.1（已分配统计对得上）、I-3.11（已分配减 defer 等于最新根走读）、I-5.2（空闲统计对得上）与 I-9.6（水位大于两处最大号） 都没有了判的对象。
+/// I-3.1（已分配统计对得上）、I-3.11（已分配减 defer 等于最新根走读）、I-5.2（空闲统计对得上）与 I-9.6（水位大于两处最大号） 都没有了判的对象
+/// （另有 I-3.13 与 I-3.4（可用空间扣待删占用） 两条读的也是记账行：删除态扫描里记账树条目宽不到 34 那一版报射程里判不了）。
 /// 「停下不解」正是 I-1.10（码 2 条目宽等于字段表宽） 要的判定顺序（**先于**按字段表解条目，与
 /// I-1.7（打包容器合法与判定顺序） 同一纪律），所以这四条转成不适用是这份镜像的结构决定的，照实钉下来。
 #[test]
@@ -3450,8 +3486,9 @@ fn an_entry_width_that_is_not_the_field_table_width_reddens_only_the_entry_width
         .collect();
     assert_eq!(
         turned_not_applicable,
-        ["I-3.1", "I-3.11", "I-5.2", "I-9.6", "I-3.13"],
-        "跟着转成不适用的该只有这五条（走读在 I-1.10 那一步停下，记账条目不再解；I-3.13 判的就是记账行的代段）：{verdicts:?}"
+        ["I-3.1", "I-3.11", "I-5.2", "I-9.6", "I-3.13", "I-3.4"],
+        "跟着转成不适用的该只有这六条（走读在 I-1.10 那一步停下，记账条目不再解；I-3.13 判的就是记账行的代段；\
+         I-3.4 的删除态扫描读记账树也按条目宽 ≥ 34 才解，读不全那一版报射程里判不了）：{verdicts:?}"
     );
 }
 
diff --git a/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs b/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs
new file mode 100644
index 0000000..bfab0bf
--- /dev/null
+++ b/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs
@@ -0,0 +1,569 @@
+//! 删除中间态的搭建与池级 checker 删除线（I-9.5、I-3.4、I-4.7、I-4.6、I-9.6 墓碑那一半）的坏镜像：
+//! 新池新建文件之后把第一个文件顺序写成三个数据单元、按只供测试的批大小 1 发意图与几批（D8（核心索引结构） 已定项 16），
+//! 或建几个空 inode；坏镜像在发布路径写出的一版上找到要改的单元（从最新的根读树表往下找），改一处、重封，
+//! 新的整单元校验和沿引用链补到根槽（`common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum`）。
+//! 用这一份的测试文件同时声明 `mod common;` 与 `mod common_pointer_head_corruption;`（照 `common_tree_split` 的做法，两份测试文件各 `mod` 一次）。
+#![allow(dead_code, reason = "两份测试文件各自只用到其中一部分")]
+
+use singlefs_checker::image::{
+    InvariantVerdict, DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+    DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+    INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
+    PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
+};
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{InodeNumber, InstanceGeneration, SlotNumber};
+use singlefs_core::bytes::ByteReader;
+use singlefs_core::checksum::crc32_castagnoli;
+use singlefs_core::pointer::NodePointer;
+use singlefs_core::records::{
+    TREE_KIND_ACCOUNTING, TREE_KIND_DELETED_INODES, TREE_KIND_INODE, TREE_KIND_TOMBSTONE,
+};
+use singlefs_core::transaction::{
+    publish_new_inodes, publish_unlink_intent, publish_unlink_step, PoolWriter, TransactionOutput,
+    FIRST_INODE_NUMBER,
+};
+use singlefs_core::unit::{data_unit_payload_capacity, parse_index_node, seal_header_checksum};
+use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
+use singlefs_harness::memory_pool::MemoryPool;
+
+use crate::common::{
+    build_pool, parameters, sequential_write_in_process_as_instance_one_and_advance_the_pool,
+    BuiltPool, FIXED_WRITE_TIME_SECONDS,
+};
+use crate::common_pointer_head_corruption::{
+    data_unit_bytes, node_bytes, replace_the_unit_and_propagate_its_checksum, the_slot_of,
+    tree_table_entry_of_kind, unit_bytes_at,
+};
+
+/// 删除与建 inode 都在新池新建文件那个实例里发。
+pub const INSTANCE_OF_THE_PUBLISHES: InstanceGeneration = InstanceGeneration(1);
+/// 第一个文件的 inode 号。
+pub const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);
+/// 建空 inode 那次的写入时间。
+pub const WRITE_TIME_SECONDS_OF_THE_NEW_INODES: u64 = 1_788_000_100;
+
+/// 坏镜像：在一份干净镜像上改一处。
+pub type Mutation = Box<dyn Fn(&mut MemoryPool)>;
+
+/// 码 3 单元头里记录数（偏移 69）、声明长度（偏移 8）与记录区起点（含 29 字节预留位的头 136，D18（块里携带什么信息） 已定项 16）。
+const PACKED_UNIT_RECORD_COUNT_OFFSET: usize = 69;
+const UNIT_DECLARED_LENGTH_OFFSET: usize = 8;
+const PACKED_UNIT_RECORDS_START: usize = 136;
+/// inode 记录 140 字节：inode 号 0、nlink 36（4 字节）、size 40、blocks 48（D8（核心索引结构） 已定项 6 字段表）。
+const INODE_RECORD_BYTES: usize = 140;
+const INODE_RECORD_NLINK_OFFSET: usize = 36;
+const INODE_RECORD_SIZE_OFFSET: usize = 40;
+const INODE_RECORD_BLOCKS_OFFSET: usize = 48;
+/// `blocks` = ⌈size ÷ 512⌉（I-9.15）。
+const INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES: u64 = 512;
+/// inode 树与墓碑树的码 2 内部条目：分隔 key 8 + 身份引用 26 + 子指针 86（子指针从 34 起）。
+const CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET: usize = 34;
+/// 码 2 节点头（key 宽 k）：最大 key 在 52 + k、条目数在 82 + 2k、条目区从 115 + 2k 起（D18（块里携带什么信息） 已定项 18）。
+const INDEX_NODE_SMALLEST_KEY_OFFSET: usize = 52;
+fn index_node_entry_count_offset(key_width: usize) -> usize {
+    82 + 2 * key_width
+}
+fn index_node_entries_start(key_width: usize) -> usize {
+    115 + 2 * key_width
+}
+/// `deleted_inodes` 根：key 宽 8、条目宽 8（D8（核心索引结构） 已定项 16）。
+const DELETED_INODES_KEY_BYTES: usize = 8;
+/// 记账树叶：key 宽 22、条目 34（统计量 2、树 8、设备 4、代 8、值 8、seq 4）。
+const ACCOUNTING_KEY_BYTES: usize = 22;
+const ACCOUNTING_ROW_BYTES: usize = 34;
+const ACCOUNTING_ROW_DEVICE_OFFSET: usize = 10;
+const ACCOUNTING_ROW_VALUE_OFFSET: usize = 22;
+/// 待删除但意图未完成占用（D5（快照 / 空间记账机制） 已定项 4 第 4 项），不带设备维的设备段。
+pub const STATISTIC_PENDING_DELETION_OCCUPANCY: u16 = 4;
+const STATISTIC_NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;
+
+pub fn batches_of_one_data_unit() -> UnlinkBatchSize {
+    UnlinkBatchSize::CappedForTests(
+        UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
+    )
+}
+
+/// 三个数据单元的内容：最后一个单元装半个净荷（与 `unlink_single_file_publishes.rs` 那一份同形）。
+pub fn content_of_three_data_units() -> Vec<u8> {
+    let payload_capacity = data_unit_payload_capacity();
+    (0..2 * payload_capacity + payload_capacity / 2)
+        .map(|index| u8::try_from((index * 13 + 5) % 251).expect("小于 256"))
+        .collect()
+}
+
+/// 新池新建文件之后把第一个文件顺序写成三个数据单元。
+pub fn pool_with_a_three_unit_file(tag: &str) -> BuiltPool {
+    let mut pool = build_pool(tag);
+    let grown = sequential_write_in_process_as_instance_one_and_advance_the_pool(
+        &mut pool,
+        &content_of_three_data_units(),
+    );
+    assert_eq!(grown.data_pointers.len(), 3, "第一个文件三个数据单元");
+    pool
+}
+
+/// unlink 第一个文件的意图那次发布：inode 1 的记录 nlink 写成 0、size 不变，`deleted_inodes` 加 1，墓碑树开放容器追加一条。
+pub fn publish_the_intent(pool: &mut BuiltPool) -> TransactionOutput {
+    let publish_parameters = parameters();
+    let previous = pool.output.clone();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+    let write_time_seconds = previous
+        .inode_record
+        .map_or(FIXED_WRITE_TIME_SECONDS, |record| record.write_time_seconds);
+    let output = publish_unlink_intent(
+        &mut writer,
+        &mut pool.allocator,
+        &previous,
+        FIRST_FILE,
+        write_time_seconds,
+        INSTANCE_OF_THE_PUBLISHES,
+    )
+    .expect("意图");
+    pool.output = output.clone();
+    output
+}
+
+/// 按批大小 1 删下一批（不是最后一批的：放掉尾部一个单元、size 改成剩下单元数 × 净荷容量；最后一批删记录与队列里的号）。
+pub fn publish_the_next_step_in_batches_of_one(pool: &mut BuiltPool) -> TransactionOutput {
+    let publish_parameters = parameters();
+    let previous = pool.output.clone();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+    let output = publish_unlink_step(
+        &mut writer,
+        &mut pool.allocator,
+        &previous,
+        FIRST_FILE,
+        batches_of_one_data_unit(),
+        INSTANCE_OF_THE_PUBLISHES,
+    )
+    .expect("删除的下一步");
+    pool.output = output.clone();
+    output
+}
+
+/// 三个单元的文件发完意图：inode 1 nlink 0、size 2.5 个净荷、三个单元都在，队列 {1}，墓碑树一条墓碑。
+pub fn image_after_the_intent_of_a_three_unit_file(tag: &str) -> MemoryPool {
+    let mut pool = pool_with_a_three_unit_file(tag);
+    publish_the_intent(&mut pool);
+    pool.memory_pool()
+}
+
+/// 三个单元的文件发完意图与第一批：inode 1 nlink 0、size 2 个净荷、剩两个单元，队列 {1}。
+pub fn image_after_the_first_batch_of_a_three_unit_file(tag: &str) -> MemoryPool {
+    let mut pool = pool_with_a_three_unit_file(tag);
+    publish_the_intent(&mut pool);
+    publish_the_next_step_in_batches_of_one(&mut pool);
+    pool.memory_pool()
+}
+
+/// 新池新建文件之后建 `count` 个空 inode（号从 2 起，size 0、没有数据单元，与第一个文件同在最左那片叶里）。
+pub fn pool_with_new_inodes(tag: &str, count: u64) -> BuiltPool {
+    let mut pool = build_pool(tag);
+    let publish_parameters = parameters();
+    let previous = pool.output.clone();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+    pool.output = publish_new_inodes(
+        &mut writer,
+        &mut pool.allocator,
+        &previous,
+        count,
+        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
+        INSTANCE_OF_THE_PUBLISHES,
+    )
+    .expect("建空 inode");
+    pool
+}
+
+/// 判违例的不变量，按 checker 报的次序。
+pub fn violated_invariants(image: &MemoryPool) -> Vec<&'static str> {
+    check_pool_image(image)
+        .into_iter()
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(_) => Some(invariant),
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
+        })
+        .collect()
+}
+
+/// 一条不变量的判定。
+pub fn verdict_of(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
+    check_pool_image(image)
+        .into_iter()
+        .find(|(identifier, _)| *identifier == invariant)
+        .expect("清单里有这一条")
+        .1
+}
+
+/// 改过的单元重封：载荷 CRC 与头校验和（码 3 罩 [107, 32768)、住 89，头 [0, 107)；码 2 罩 [86 + 2k, 16384)、住 76 + 2k，头 [0, 86 + 2k)）。
+fn reseal_unit(bytes: &mut [u8]) {
+    let (header_end, payload_checksum_offset) = match bytes[6] {
+        3 => (107, 89),
+        2 => {
+            let key_width = usize::from(bytes[51]);
+            (86 + 2 * key_width, 76 + 2 * key_width)
+        }
+        other_class => panic!("这几份坏镜像只改码 2 与码 3 单元，读到类 {other_class}"),
+    };
+    let payload_checksum = crc32_castagnoli(&bytes[header_end..]);
+    bytes[payload_checksum_offset..payload_checksum_offset + 4]
+        .copy_from_slice(&payload_checksum.to_le_bytes());
+    seal_header_checksum(bytes, header_end);
+}
+
+/// `slot` 那个单元（两盘同槽，`length_in_bytes` 宽）改一处、重封，新的整单元校验和沿引用链补到根槽。
+pub fn change_the_unit_and_propagate_its_checksum(
+    image: &mut MemoryPool,
+    slot: SlotNumber,
+    length_in_bytes: usize,
+    change: impl FnOnce(&mut Vec<u8>),
+) {
+    let mut bytes = unit_bytes_at(image, slot, length_in_bytes);
+    change(&mut bytes);
+    reseal_unit(&mut bytes);
+    replace_the_unit_and_propagate_its_checksum(image, slot, &bytes);
+}
+
+fn get_u16(bytes: &[u8], offset: usize) -> u16 {
+    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("2 字节"))
+}
+fn get_u32(bytes: &[u8], offset: usize) -> u32 {
+    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4 字节"))
+}
+fn get_u64(bytes: &[u8], offset: usize) -> u64 {
+    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
+}
+fn set_u16(bytes: &mut [u8], offset: usize, value: u16) {
+    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
+}
+fn set_u32(bytes: &mut [u8], offset: usize, value: u32) {
+    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
+}
+fn set_u64(bytes: &mut [u8], offset: usize, value: u64) {
+    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
+}
+
+/// 一个码 2 内部条目（inode 树、墓碑树：120 字节）里子指针指的那个槽。
+fn child_slot_of_a_container_tree_entry(entry: &[u8]) -> SlotNumber {
+    let child = NodePointer::read_from(&mut ByteReader::at(
+        entry,
+        CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET,
+    ));
+    assert_eq!(
+        child.locations[0].slot, child.locations[1].slot,
+        "第一版两盘同槽"
+    );
+    child.locations[0].slot
+}
+
+/// `leaf_slot` 那片 inode 叶容器里 `inode` 那条记录的下标。
+fn index_of_the_inode_record_in_the_leaf_at(
+    image: &MemoryPool,
+    leaf_slot: SlotNumber,
+    inode: u64,
+) -> Option<usize> {
+    let leaf = unit_bytes_at(image, leaf_slot, data_unit_bytes());
+    let record_count = usize::from(get_u16(&leaf, PACKED_UNIT_RECORD_COUNT_OFFSET));
+    (0..record_count).find(|index| {
+        get_u64(
+            &leaf,
+            PACKED_UNIT_RECORDS_START + index * INODE_RECORD_BYTES,
+        ) == inode
+    })
+}
+
+/// 最新的根那一版 inode 树里装着 `inode` 那条记录的叶容器的槽（根是层级 1、孩子都是叶容器的那一档）与记录在容器里的下标。
+fn inode_leaf_holding(image: &MemoryPool, inode: u64) -> (SlotNumber, usize) {
+    let inode_root = tree_table_entry_of_kind(image, TREE_KIND_INODE).root;
+    let root = parse_index_node(&unit_bytes_at(
+        image,
+        the_slot_of(&inode_root),
+        node_bytes(),
+    ))
+    .expect("inode 树根解得开");
+    assert_eq!(root.level, 1, "这几份镜像上 inode 树根直接管叶容器");
+    root.entries
+        .iter()
+        .map(|entry| child_slot_of_a_container_tree_entry(entry))
+        .find_map(|leaf_slot| {
+            index_of_the_inode_record_in_the_leaf_at(image, leaf_slot, inode)
+                .map(|index| (leaf_slot, index))
+        })
+        .unwrap_or_else(|| panic!("最新的根那一版 inode 树里有 inode {inode} 的记录"))
+}
+
+/// 最新的根那一版里 `inode` 那条记录改一处（`change` 拿到的是那 140 字节），重封叶、补校验和。
+pub fn change_the_inode_record(image: &mut MemoryPool, inode: u64, change: impl FnOnce(&mut [u8])) {
+    let (leaf_slot, _) = inode_leaf_holding(image, inode);
+    change_the_inode_record_in_the_leaf_at(image, leaf_slot, inode, change);
+}
+
+/// `leaf_slot` 那片 inode 叶容器（哪一版的都行：只被一版引用的那片改了只动那一版）里 `inode` 那条记录改一处，重封叶、补校验和。
+pub fn change_the_inode_record_in_the_leaf_at(
+    image: &mut MemoryPool,
+    leaf_slot: SlotNumber,
+    inode: u64,
+    change: impl FnOnce(&mut [u8]),
+) {
+    let index = index_of_the_inode_record_in_the_leaf_at(image, leaf_slot, inode)
+        .unwrap_or_else(|| panic!("槽 {} 那片叶里有 inode {inode} 的记录", leaf_slot.0));
+    change_the_unit_and_propagate_its_checksum(image, leaf_slot, data_unit_bytes(), |leaf| {
+        let start = PACKED_UNIT_RECORDS_START + index * INODE_RECORD_BYTES;
+        change(&mut leaf[start..start + INODE_RECORD_BYTES]);
+    });
+}
+
+/// I-9.5 反方向（C708 那一形，也是判决里说的僵尸记录：nlink 0、单元全放、不在队列）：`inode` 的记录 nlink 1 → 0，队列不动。
+pub fn set_the_nlink_of_an_inode_record_to_zero(image: &mut MemoryPool, inode: u64) {
+    change_the_inode_record(image, inode, |record| {
+        assert_eq!(
+            get_u32(record, INODE_RECORD_NLINK_OFFSET),
+            1,
+            "改之前 inode {inode} 的 nlink 是 1"
+        );
+        set_u32(record, INODE_RECORD_NLINK_OFFSET, 0);
+    });
+}
+
+/// I-9.5 正方向「队列里的号没有记录」那一形（判决第二节最后一行：只红 I-9.5）：意图之后的 `deleted_inodes` 根 {1} 改成 {1, 7}
+/// （条目数 1 → 2、声明长度 8 → 16、头里最大 key 1 → 7），7 号在 inode 树里没有记录、extent 树里也没有单元。
+pub fn put_an_inode_number_without_a_record_into_the_deletion_queue(image: &mut MemoryPool) {
+    const INODE_NUMBER_WITHOUT_A_RECORD: u64 = 7;
+    let deleted_inodes_root = tree_table_entry_of_kind(image, TREE_KIND_DELETED_INODES).root;
+    change_the_unit_and_propagate_its_checksum(
+        image,
+        the_slot_of(&deleted_inodes_root),
+        node_bytes(),
+        |root| {
+            let count_offset = index_node_entry_count_offset(DELETED_INODES_KEY_BYTES);
+            let entries_start = index_node_entries_start(DELETED_INODES_KEY_BYTES);
+            assert_eq!(get_u16(root, count_offset), 1, "意图之后队列里一个号");
+            assert_eq!(
+                get_u64(root, entries_start),
+                FIRST_INODE_NUMBER,
+                "队列里是 1"
+            );
+            set_u16(root, count_offset, 2);
+            set_u16(root, UNIT_DECLARED_LENGTH_OFFSET, 2 * 8);
+            set_u64(
+                root,
+                INDEX_NODE_SMALLEST_KEY_OFFSET + DELETED_INODES_KEY_BYTES,
+                INODE_NUMBER_WITHOUT_A_RECORD,
+            );
+            set_u64(root, entries_start + 8, INODE_NUMBER_WITHOUT_A_RECORD);
+        },
+    );
+}
+
+/// I-4.7 只红自己的那一形（判决第二节 F1：孤儿单元，今天的实现走不到，在 checker 侧直接改字节造）：第一个文件（inode 1，有一个内联数据单元）
+/// 与空 inode 2 同在最左那片叶里，把 1 号那条记录从叶里拿掉（2 号挪到第 0 条、记录数 2 → 1、声明长度 280 → 140），extent 树不动：
+/// 1 号在 extent 树里还有数据单元、inode 树里没有它的记录、号也不在队列里。
+pub fn drop_the_record_of_the_first_file_and_keep_its_data_units(image: &mut MemoryPool) {
+    let (leaf_slot, index) = inode_leaf_holding(image, FIRST_INODE_NUMBER);
+    assert_eq!(index, 0, "1 号是叶里第 0 条");
+    change_the_unit_and_propagate_its_checksum(image, leaf_slot, data_unit_bytes(), |leaf| {
+        assert_eq!(
+            get_u16(leaf, PACKED_UNIT_RECORD_COUNT_OFFSET),
+            2,
+            "叶里两条：1 号与空 inode 2"
+        );
+        let second = PACKED_UNIT_RECORDS_START + INODE_RECORD_BYTES;
+        let second_record = leaf[second..second + INODE_RECORD_BYTES].to_vec();
+        leaf[PACKED_UNIT_RECORDS_START..second].copy_from_slice(&second_record);
+        leaf[second..second + INODE_RECORD_BYTES].fill(0);
+        set_u16(leaf, PACKED_UNIT_RECORD_COUNT_OFFSET, 1);
+        set_u16(
+            leaf,
+            UNIT_DECLARED_LENGTH_OFFSET,
+            u16::try_from(INODE_RECORD_BYTES).expect("140"),
+        );
+    });
+}
+
+/// I-4.6 只红自己的那一形：删除中间那一版 inode 1 的 size 多加一个净荷容量（blocks 跟着按 ⌈size ÷ 512⌉ 改，I-9.15 照旧成立），
+/// extent 树里的单元数不动。
+pub fn raise_the_size_of_the_first_file_by_one_data_unit_payload(image: &mut MemoryPool) {
+    let (leaf_slot, _) = inode_leaf_holding(image, FIRST_INODE_NUMBER);
+    raise_the_size_of_the_first_file_in_the_leaf_at(image, leaf_slot);
+}
+
+/// [`raise_the_size_of_the_first_file_by_one_data_unit_payload`] 改的是 `leaf_slot` 那片叶（删除中间某一版的叶）。
+pub fn raise_the_size_of_the_first_file_in_the_leaf_at(
+    image: &mut MemoryPool,
+    leaf_slot: SlotNumber,
+) {
+    change_the_inode_record_in_the_leaf_at(image, leaf_slot, FIRST_INODE_NUMBER, |record| {
+        assert_eq!(
+            get_u32(record, INODE_RECORD_NLINK_OFFSET),
+            0,
+            "1 号处于删除中"
+        );
+        let payload_capacity = u64::try_from(data_unit_payload_capacity()).expect("净荷容量");
+        let size = get_u64(record, INODE_RECORD_SIZE_OFFSET) + payload_capacity;
+        set_u64(record, INODE_RECORD_SIZE_OFFSET, size);
+        set_u64(
+            record,
+            INODE_RECORD_BLOCKS_OFFSET,
+            size.div_ceil(INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES),
+        );
+    });
+}
+
+/// 记账树（`slot` 那个根兼叶，层级 0）里待删除但意图未完成占用那一行改一处：`Some(value)` 改值，`None` 把这一行从叶里拿掉
+/// （后面的行往前挪、条目数与声明长度各减一行；这一行不在首末，头里的 key 区间不变）。
+pub fn change_the_pending_deletion_occupancy_row_of_the_accounting_tree_at(
+    image: &mut MemoryPool,
+    slot: SlotNumber,
+    value: Option<u64>,
+) {
+    change_the_unit_and_propagate_its_checksum(image, slot, node_bytes(), |node| {
+        assert_eq!(node[50], 0, "这几份镜像上记账树只有根兼叶");
+        let count_offset = index_node_entry_count_offset(ACCOUNTING_KEY_BYTES);
+        let entries_start = index_node_entries_start(ACCOUNTING_KEY_BYTES);
+        let row_count = usize::from(get_u16(node, count_offset));
+        let row_index = (0..row_count)
+            .find(|index| {
+                let row = entries_start + index * ACCOUNTING_ROW_BYTES;
+                get_u16(node, row) == STATISTIC_PENDING_DELETION_OCCUPANCY
+                    && get_u32(node, row + ACCOUNTING_ROW_DEVICE_OFFSET)
+                        == STATISTIC_NO_DEVICE_DIMENSION
+            })
+            .expect("记账树里有待删除但意图未完成占用那一行");
+        let row = entries_start + row_index * ACCOUNTING_ROW_BYTES;
+        match value {
+            Some(value) => set_u64(node, row + ACCOUNTING_ROW_VALUE_OFFSET, value),
+            None => {
+                assert!(
+                    row_index > 0 && row_index + 1 < row_count,
+                    "这一行不在首末：拿掉它头里的 key 区间不变"
+                );
+                let end = entries_start + row_count * ACCOUNTING_ROW_BYTES;
+                node.copy_within(row + ACCOUNTING_ROW_BYTES..end, row);
+                node[end - ACCOUNTING_ROW_BYTES..end].fill(0);
+                set_u16(
+                    node,
+                    count_offset,
+                    u16::try_from(row_count - 1).expect("行数"),
+                );
+                set_u16(
+                    node,
+                    UNIT_DECLARED_LENGTH_OFFSET,
+                    u16::try_from((row_count - 1) * ACCOUNTING_ROW_BYTES).expect("声明长度"),
+                );
+            }
+        }
+    });
+}
+
+/// 最新的根那一版的记账树根的槽。
+pub fn accounting_tree_slot_of_the_newest_root(image: &MemoryPool) -> SlotNumber {
+    the_slot_of(&tree_table_entry_of_kind(image, TREE_KIND_ACCOUNTING).root)
+}
+
+/// I-3.4：最新的根那一版的记账树里待删占用那一行 0 → 1。
+pub fn set_the_pending_deletion_occupancy_row_to_one(image: &mut MemoryPool) {
+    let slot = accounting_tree_slot_of_the_newest_root(image);
+    change_the_pending_deletion_occupancy_row_of_the_accounting_tree_at(image, slot, Some(1));
+}
+
+/// I-3.4：最新的根那一版的记账树里拿掉待删占用那一行。
+pub fn drop_the_pending_deletion_occupancy_row(image: &mut MemoryPool) {
+    let slot = accounting_tree_slot_of_the_newest_root(image);
+    change_the_pending_deletion_occupancy_row_of_the_accounting_tree_at(image, slot, None);
+}
+
+/// I-9.6 墓碑那一半：意图之后墓碑树第一片容器里那条墓碑的对象 ID 1 → 水位（记账里的 inode 号水位 2）：水位不再大于全部墓碑的对象 ID，
+/// inode 树里最大的 key 仍是 1。
+pub fn raise_the_tombstone_object_identifier_to_the_inode_number_watermark(image: &mut MemoryPool) {
+    const INODE_NUMBER_WATERMARK_AFTER_THE_FIRST_FILE: u64 = 2;
+    let tombstone_root = tree_table_entry_of_kind(image, TREE_KIND_TOMBSTONE).root;
+    let first_entry = parse_index_node(&unit_bytes_at(
+        image,
+        the_slot_of(&tombstone_root),
+        node_bytes(),
+    ))
+    .expect("墓碑树根解得开")
+    .entries
+    .first()
+    .expect("墓碑树根至少一条条目")
+    .clone();
+    let container_slot = child_slot_of_a_container_tree_entry(&first_entry);
+    change_the_unit_and_propagate_its_checksum(
+        image,
+        container_slot,
+        data_unit_bytes(),
+        |container| {
+            assert_eq!(
+                get_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET),
+                1,
+                "意图之后墓碑容器里一条墓碑"
+            );
+            assert_eq!(
+                get_u64(container, PACKED_UNIT_RECORDS_START),
+                FIRST_INODE_NUMBER,
+                "那条墓碑的对象 ID 是 1"
+            );
+            set_u64(
+                container,
+                PACKED_UNIT_RECORDS_START,
+                INODE_NUMBER_WATERMARK_AFTER_THE_FIRST_FILE,
+            );
+        },
+    );
+}
+
+/// 意图之后那一版上的坏镜像（`image_after_the_intent_of_a_three_unit_file`）。
+pub fn known_bad_images_after_the_intent_of_a_three_unit_file() -> Vec<(&'static str, Mutation)> {
+    vec![
+        (
+            DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+            Box::new(put_an_inode_number_without_a_record_into_the_deletion_queue),
+        ),
+        (
+            "I-9.6",
+            Box::new(raise_the_tombstone_object_identifier_to_the_inode_number_watermark),
+        ),
+    ]
+}
+
+/// 意图与第一批之后那一版上的坏镜像（`image_after_the_first_batch_of_a_three_unit_file`）。
+pub fn known_bad_images_after_the_first_batch_of_a_three_unit_file() -> Vec<(&'static str, Mutation)>
+{
+    vec![(
+        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+        Box::new(raise_the_size_of_the_first_file_by_one_data_unit_payload),
+    )]
+}
+
+/// 第一个文件之后建了一个空 inode（2 号）那一版上的坏镜像（`pool_with_new_inodes(…, 1)`）。
+pub fn known_bad_images_after_one_new_inode() -> Vec<(&'static str, Mutation)> {
+    vec![
+        (
+            DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+            Box::new(|image: &mut MemoryPool| set_the_nlink_of_an_inode_record_to_zero(image, 2)),
+        ),
+        (
+            INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
+            Box::new(drop_the_record_of_the_first_file_and_keep_its_data_units),
+        ),
+    ]
+}
+
+/// 写完新池新建文件那一版（`build_pool`）上的坏镜像：记账标签 4 非 0、标签 4 缺行。
+pub fn known_bad_images_of_the_pending_deletion_occupancy() -> Vec<(&'static str, Mutation)> {
+    vec![
+        (
+            PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
+            Box::new(set_the_pending_deletion_occupancy_row_to_one),
+        ),
+        (
+            PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
+            Box::new(drop_the_pending_deletion_occupancy_row),
+        ),
+    ]
+}
diff --git a/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs b/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs
new file mode 100644
index 0000000..190be19
--- /dev/null
+++ b/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs
@@ -0,0 +1,377 @@
+//! 池级 checker 的删除线（里程碑三·3）：I-9.5（删除队列里的号仍在树里，两个方向）、I-3.4（只落「记账标签 4 恒 0」）、
+//! I-4.7（对象取丙）、I-4.6（取乙：nlink = 0 的记录单元数对 size）、I-4.4（第一版不适用）与 I-9.6 墓碑那一半。
+//! 读法照判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节与岔路单 `research/prompts/m3-delete-checker-r1-forks.md`：
+//! 判回退候选集里每一版（加由记录施加出来的几版），不只判最新根；读不全的那一版报射程里判不了、不判红。
+//! 坏镜像各一，每份只红自己那一条；搭建与坏镜像的改法在 `common_deletion_in_progress`（`checker_known_bad_images.rs` 那一串 `.chain` 也登记它们）。
+
+mod common;
+mod common_deletion_in_progress;
+mod common_pointer_head_corruption;
+
+use common::build_pool;
+use common_deletion_in_progress::{
+    change_the_pending_deletion_occupancy_row_of_the_accounting_tree_at,
+    drop_the_pending_deletion_occupancy_row,
+    drop_the_record_of_the_first_file_and_keep_its_data_units,
+    image_after_the_first_batch_of_a_three_unit_file, image_after_the_intent_of_a_three_unit_file,
+    pool_with_a_three_unit_file, pool_with_new_inodes, publish_the_intent,
+    publish_the_next_step_in_batches_of_one,
+    put_an_inode_number_without_a_record_into_the_deletion_queue,
+    raise_the_size_of_the_first_file_by_one_data_unit_payload,
+    raise_the_size_of_the_first_file_in_the_leaf_at,
+    raise_the_tombstone_object_identifier_to_the_inode_number_watermark,
+    set_the_nlink_of_an_inode_record_to_zero, set_the_pending_deletion_occupancy_row_to_one,
+    verdict_of, violated_invariants,
+};
+use common_pointer_head_corruption::{data_unit_bytes, unit_bytes_at};
+use singlefs_checker::image::{
+    InvariantVerdict, DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+    DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+    INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT, INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
+    INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
+};
+use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
+use singlefs_core::transaction::TransactionUnit;
+use singlefs_harness::memory_pool::MemoryPool;
+
+/// 第一处违例的说明；这一条没判违例就当场失败。
+fn first_violation_of(image: &MemoryPool, invariant: &str) -> String {
+    match verdict_of(image, invariant) {
+        InvariantVerdict::Violated(detail) => detail,
+        other_verdict @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
+            panic!("{invariant} 应当判违例，实际 {other_verdict:?}")
+        }
+    }
+}
+
+/// 干净的那一版上一条违例都没有：坏镜像「只红自己」从这里比起。
+fn assert_the_base_image_is_clean(image: &MemoryPool, base: &str) {
+    assert_eq!(
+        violated_invariants(image),
+        Vec::<&str>::new(),
+        "{base}那一版没有违例"
+    );
+}
+
+/// I-9.5 正方向，只红自己的那一形（判决第二节最后一行）：意图之后队列 {1} 改成 {1, 7}，7 号在 inode 树里没有记录、extent 树里也没有单元。
+/// I-9.18（删除队列里的号有谱系上的墓碑） 这一版 checker 还没实现（不在 `IMPLEMENTED_INVARIANTS` 里），这一形上它不判。
+#[test]
+fn a_queue_number_without_an_inode_record_reddens_only_the_deletion_queue_invariant() {
+    let clean =
+        image_after_the_intent_of_a_three_unit_file("deletion-checker-queue-without-record");
+    assert_the_base_image_is_clean(&clean, "意图之后");
+    let mut image = clean.clone();
+    put_an_inode_number_without_a_record_into_the_deletion_queue(&mut image);
+    assert_eq!(
+        violated_invariants(&image),
+        vec![DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES],
+        "只红 I-9.5"
+    );
+    let detail = first_violation_of(
+        &image,
+        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+    );
+    assert!(
+        detail.contains("deleted_inodes 里的号 7") && detail.contains("没有记录"),
+        "违例说明点名队列里那个没有记录的号：{detail}"
+    );
+}
+
+/// I-9.5 反方向（C708 那一形，也是判决里说的僵尸记录：nlink 0、单元全放、不在队列）：第一个文件之后建一个空 inode（2 号，没有数据单元），
+/// 它的记录 nlink 1 → 0，队列不动。I-4.7 对它成立（一个数据单元都不剩），I-4.6 也成立（size 0、0 个单元）。
+#[test]
+fn an_inode_record_with_nlink_zero_outside_the_queue_reddens_only_the_deletion_queue_invariant() {
+    let clean = pool_with_new_inodes("deletion-checker-zombie-record", 1).memory_pool();
+    assert_the_base_image_is_clean(&clean, "建了一个空 inode 之后");
+    let mut image = clean.clone();
+    set_the_nlink_of_an_inode_record_to_zero(&mut image, 2);
+    assert_eq!(
+        violated_invariants(&image),
+        vec![DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES],
+        "只红 I-9.5"
+    );
+    let detail = first_violation_of(
+        &image,
+        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+    );
+    assert!(
+        detail.contains("inode 2 的记录 nlink = 0") && detail.contains("不在"),
+        "违例说明点名不在队列里的那条记录：{detail}"
+    );
+}
+
+/// 形式化接入那条线攻方的 `being-deleted` 那一形（`research/prompts/m3-formal-code-b21-r1-opus-model/out/k2a.txt` 第 5 行：
+/// 一次建 inode 的发布写出一条新记录 (3, 9, nlink 0, size 0)、号不在队列里，那时池级 checker 一条都不红；C708 的验收）：
+/// 一次发布建两个空 inode（2、3 号），把这次新写出的 3 号记录 nlink 改成 0。池级 checker 在这一形上红 I-9.5。
+#[test]
+fn a_new_inode_published_with_nlink_zero_as_in_the_formal_line_being_deleted_variant_reddens_the_deletion_queue_invariant(
+) {
+    let clean = pool_with_new_inodes("deletion-checker-being-deleted-variant", 2).memory_pool();
+    assert_the_base_image_is_clean(&clean, "一次发布建了两个空 inode 之后");
+    let mut image = clean.clone();
+    set_the_nlink_of_an_inode_record_to_zero(&mut image, 3);
+    assert_eq!(
+        violated_invariants(&image),
+        vec![DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES],
+        "k2a 那一形红 I-9.5，别的不红"
+    );
+    let detail = first_violation_of(
+        &image,
+        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+    );
+    assert!(
+        detail.contains("inode 3 的记录 nlink = 0"),
+        "违例说明点名新写出的那条记录：{detail}"
+    );
+}
+
+/// I-4.7 只红自己的那一形（判决第二节 F1：孤儿单元，今天的实现走不到，在 checker 侧直接改字节造）：第一个文件（1 号，一个内联数据单元）
+/// 的记录从叶里拿掉、extent 树不动，号也不在队列里。I-9.5 两个方向都成立（队列空、2 号 nlink 1），I-4.6 没有对象。
+#[test]
+fn data_units_of_an_inode_without_a_record_outside_the_queue_redden_only_the_inode_being_deleted_invariant(
+) {
+    let clean = pool_with_new_inodes("deletion-checker-orphan-data-units", 1).memory_pool();
+    assert_the_base_image_is_clean(&clean, "建了一个空 inode 之后");
+    let mut image = clean.clone();
+    drop_the_record_of_the_first_file_and_keep_its_data_units(&mut image);
+    assert_eq!(
+        violated_invariants(&image),
+        vec![INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT],
+        "只红 I-4.7"
+    );
+    let detail = first_violation_of(
+        &image,
+        INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
+    );
+    assert!(
+        detail.contains("inode 1 处于删除中（inode 树里没有它的记录）")
+            && detail.contains("还有 1 个数据单元"),
+        "违例说明点名那个号、为什么算删除中与剩几个单元：{detail}"
+    );
+}
+
+/// I-4.6 只红自己的那一形：意图与第一批之后（inode 1 nlink 0、size 2 个净荷、剩两个单元）把 size 多加一个净荷容量、blocks 跟着改，
+/// 单元数不动。I-9.5、I-4.7 照旧成立（队列 {1}、nlink 0）。
+#[test]
+fn an_inode_being_deleted_whose_size_asks_for_more_data_units_than_it_has_reddens_only_the_intermediate_state_invariant(
+) {
+    let clean =
+        image_after_the_first_batch_of_a_three_unit_file("deletion-checker-size-ahead-of-units");
+    assert_the_base_image_is_clean(&clean, "意图与第一批之后");
+    let mut image = clean.clone();
+    raise_the_size_of_the_first_file_by_one_data_unit_payload(&mut image);
+    assert_eq!(
+        violated_invariants(&image),
+        vec![DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE],
+        "只红 I-4.6"
+    );
+    let detail = first_violation_of(
+        &image,
+        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+    );
+    assert!(
+        detail.contains("要 3 个数据单元") && detail.contains("extent 树里有 2 个"),
+        "违例说明写出 size 要几个、实有几个：{detail}"
+    );
+}
+
+/// I-3.4：写完新池新建文件那一版记账树里待删占用那一行 0 → 1。别的条不读标签 4 的值（I-3.13 只比代段）。
+#[test]
+fn a_pending_deletion_occupancy_row_that_is_not_zero_reddens_only_the_pending_deletion_invariant() {
+    let clean = build_pool("deletion-checker-pending-row-not-zero").memory_pool();
+    assert_the_base_image_is_clean(&clean, "写完新池新建文件");
+    let mut image = clean.clone();
+    set_the_pending_deletion_occupancy_row_to_one(&mut image);
+    assert_eq!(
+        violated_invariants(&image),
+        vec![PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO],
+        "只红 I-3.4"
+    );
+    let detail = first_violation_of(&image, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO);
+    assert!(
+        detail.contains("的值是 1"),
+        "违例说明写出那一行的值：{detail}"
+    );
+}
+
+/// I-3.4 缺行判红（判决第二节 F5-b）：写完新池新建文件那一版记账树里拿掉待删占用那一行，别的行一个字节都不动。
+#[test]
+fn an_accounting_tree_without_the_pending_deletion_occupancy_row_reddens_only_the_pending_deletion_invariant(
+) {
+    let clean = build_pool("deletion-checker-pending-row-missing").memory_pool();
+    assert_the_base_image_is_clean(&clean, "写完新池新建文件");
+    let mut image = clean.clone();
+    drop_the_pending_deletion_occupancy_row(&mut image);
+    assert_eq!(
+        violated_invariants(&image),
+        vec![PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO],
+        "只红 I-3.4"
+    );
+    let detail = first_violation_of(&image, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO);
+    assert!(
+        detail.contains("没有待删除但意图未完成占用"),
+        "违例说明写出缺的那一行：{detail}"
+    );
+}
+
+/// I-9.6 墓碑那一半（`.claude/kb/invariants.md` I-9.6 那一行：水位 > 所选根的墓碑树里全部已发布的墓碑记录的对象 ID）：
+/// 意图之后那条墓碑的对象 ID 1 → 2（记账里的 inode 号水位），inode 树里最大的 key 仍是 1——只判「水位 > 树内最大 key」那一半时它全绿。
+#[test]
+fn a_tombstone_whose_object_identifier_reaches_the_inode_number_watermark_reddens_only_the_watermark_invariant(
+) {
+    let clean =
+        image_after_the_intent_of_a_three_unit_file("deletion-checker-tombstone-at-watermark");
+    assert_the_base_image_is_clean(&clean, "意图之后");
+    let mut image = clean.clone();
+    raise_the_tombstone_object_identifier_to_the_inode_number_watermark(&mut image);
+    assert_eq!(violated_invariants(&image), vec!["I-9.6"], "只红 I-9.6");
+    let detail = first_violation_of(&image, "I-9.6");
+    assert!(
+        detail.contains("墓碑记录的最大对象 ID 2"),
+        "违例说明写出墓碑那一半：{detail}"
+    );
+}
+
+/// 判决第二节 F4-a / F5-c：判回退候选集里每一版，不只判最新根。三个单元的文件删到「意图、留 2、留 1」，
+/// 只把中间那一版（留 2 那一批）记账树里待删占用那一行改成 1；最新的根（留 1）那一版是对的——只判最新根会漏这一形（攻方量过）。
+#[test]
+fn an_intermediate_deletion_batch_with_a_non_zero_pending_deletion_row_reddens_while_the_newest_root_is_clean(
+) {
+    let mut pool = pool_with_a_three_unit_file("deletion-checker-intermediate-pending-row");
+    publish_the_intent(&mut pool);
+    let intermediate = publish_the_next_step_in_batches_of_one(&mut pool);
+    let newest = publish_the_next_step_in_batches_of_one(&mut pool);
+    let intermediate_accounting_slot = intermediate.unit(TransactionUnit::AccountingTree).slot;
+    assert_ne!(
+        intermediate_accounting_slot,
+        newest.unit(TransactionUnit::AccountingTree).slot,
+        "每一版重写记账树：中间那一版的记账树只有它自己引用"
+    );
+    let clean = pool.memory_pool();
+    assert_the_base_image_is_clean(&clean, "删到留 1");
+    let mut image = clean.clone();
+    change_the_pending_deletion_occupancy_row_of_the_accounting_tree_at(
+        &mut image,
+        intermediate_accounting_slot,
+        Some(1),
+    );
+    assert_eq!(
+        violated_invariants(&image),
+        vec![PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO],
+        "中间那一版写坏、最新根写对：只红 I-3.4"
+    );
+    let detail = first_violation_of(&image, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO);
+    assert!(
+        detail.contains(&format!("txg {}）", intermediate.root.checkpoint_txg.0)),
+        "违例说明点名中间那一版：{detail}"
+    );
+}
+
+/// 同上一形，判的是 I-4.6：只把中间那一版（留 2 那一批）的 inode 叶里 1 号的 size 多加一个净荷容量，最新的根（留 1）那一版是对的。
+#[test]
+fn an_intermediate_deletion_batch_whose_size_asks_for_more_data_units_reddens_while_the_newest_root_is_clean(
+) {
+    let mut pool = pool_with_a_three_unit_file("deletion-checker-intermediate-size");
+    publish_the_intent(&mut pool);
+    let intermediate = publish_the_next_step_in_batches_of_one(&mut pool);
+    let newest = publish_the_next_step_in_batches_of_one(&mut pool);
+    let leftmost_leaf =
+        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST);
+    let intermediate_leaf_slot = intermediate.unit(leftmost_leaf).slot;
+    assert_ne!(
+        intermediate_leaf_slot,
+        newest.unit(leftmost_leaf).slot,
+        "每一批重写 inode 叶：中间那一版的叶只有它自己引用"
+    );
+    let clean = pool.memory_pool();
+    assert_the_base_image_is_clean(&clean, "删到留 1");
+    let mut image = clean.clone();
+    raise_the_size_of_the_first_file_in_the_leaf_at(&mut image, intermediate_leaf_slot);
+    assert_eq!(
+        violated_invariants(&image),
+        vec![DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE],
+        "中间那一版写坏、最新根写对：只红 I-4.6"
+    );
+    let detail = first_violation_of(
+        &image,
+        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+    );
+    assert!(
+        detail.contains(&format!("txg {}）", intermediate.root.checkpoint_txg.0)),
+        "违例说明点名中间那一版：{detail}"
+    );
+}
+
+/// 判决第二节 F4-c：读不全的那一版报射程里判不了（`part_of_the_range_not_judged`，没有违例时整条报不适用），不判红。
+/// 中间那一版（留 2）的 inode 叶两份都盖坏、不补校验和：那一版的记录读不出，I-9.5、I-4.7、I-4.6 在那一版判不了；
+/// 别的版本照判、都成立，整条报不适用并带「inode 树读不全」的理由。主走读那几条（I-2.1、I-7.4、I-4.8）照常因为这一片红。
+#[test]
+fn an_intermediate_version_whose_inode_leaf_does_not_read_back_is_not_judged_rather_than_red() {
+    let mut pool = pool_with_a_three_unit_file("deletion-checker-intermediate-unreadable-leaf");
+    publish_the_intent(&mut pool);
+    let intermediate = publish_the_next_step_in_batches_of_one(&mut pool);
+    publish_the_next_step_in_batches_of_one(&mut pool);
+    let leftmost_leaf =
+        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST);
+    let intermediate_leaf_slot = intermediate.unit(leftmost_leaf).slot;
+    let mut image = pool.memory_pool();
+    let mut overwritten = unit_bytes_at(&image, intermediate_leaf_slot, data_unit_bytes());
+    overwritten[1000] ^= 0xFF;
+    for device in image.devices.values_mut() {
+        device.write(intermediate_leaf_slot.to_device_offset(), &overwritten);
+    }
+    for invariant in [
+        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+        INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
+        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+    ] {
+        let found = verdict_of(&image, invariant);
+        assert!(
+            matches!(&found, InvariantVerdict::NotApplicable(reason) if reason.contains("inode 树读不全")),
+            "{invariant} 在读不全的那一版上判不了、不判红，实际 {found:?}"
+        );
+    }
+}
+
+/// 阳性对照：三个单元的文件按批大小 1 删完（意图、留 2、留 1、最后一批），每一步的镜像上一条违例都没有；
+/// I-9.5、I-3.4、I-4.7、I-4.6 真被评估过且成立（最后一批之后，回退候选集里意图与各批那几版还在，删除中的号仍有对象），
+/// I-4.4 报不适用、理由照 `.claude/kb/invariants.md` I-4.4 那一行。
+#[test]
+fn every_published_step_of_unlinking_a_three_unit_file_holds_the_deletion_invariants() {
+    let mut pool = pool_with_a_three_unit_file("deletion-checker-every-step-holds");
+    publish_the_intent(&mut pool);
+    let mut images_of_the_steps = vec![("意图", pool.memory_pool())];
+    publish_the_next_step_in_batches_of_one(&mut pool);
+    images_of_the_steps.push(("留 2", pool.memory_pool()));
+    publish_the_next_step_in_batches_of_one(&mut pool);
+    images_of_the_steps.push(("留 1", pool.memory_pool()));
+    publish_the_next_step_in_batches_of_one(&mut pool);
+    images_of_the_steps.push(("最后一批", pool.memory_pool()));
+    for (step, image) in &images_of_the_steps {
+        assert_eq!(
+            violated_invariants(image),
+            Vec::<&str>::new(),
+            "{step}之后没有违例"
+        );
+        for invariant in [
+            DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+            PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
+            INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
+            DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+            "I-9.6",
+        ] {
+            assert_eq!(
+                verdict_of(image, invariant),
+                InvariantVerdict::Holds,
+                "{step}之后 {invariant} 真被评估过且成立"
+            );
+        }
+        assert_eq!(
+            verdict_of(image, INTENT_DESCRIBES_ONLY_THE_TARGET_STATE),
+            InvariantVerdict::NotApplicable(INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION),
+            "{step}之后 I-4.4 报第一版不适用"
+        );
+    }
+}
diff --git a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
index 12f0e4f..5272dad 100644
--- a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
+++ b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
@@ -555,8 +555,8 @@ fn region_device(txg: u64) -> DeviceIdentity {
     parameters().region_devices[usize::try_from(target.region).expect("区域号")]
 }
 
-/// 最新的根下面还没有记账树、inode 树与分配记录树时（mkfs 之后、零单元发布之后）checker 报不适用的那 31 条；
-/// 其余 38 条要真被评估过且成立（条数按今天的 `IMPLEMENTED_INVARIANTS` 69 条数）。映射 key 与单元头相符（`MAPPING_KEY_MATCHES_THE_UNIT_HEADER`）报不适用是因为中央映射树的根指针全零，
+/// 最新的根下面还没有记账树、inode 树与分配记录树时（mkfs 之后、零单元发布之后）checker 报不适用的那 36 条；
+/// 其余 38 条要真被评估过且成立（条数按今天的 `IMPLEMENTED_INVARIANTS` 74 条数）。映射 key 与单元头相符（`MAPPING_KEY_MATCHES_THE_UNIT_HEADER`）报不适用是因为中央映射树的根指针全零，
 /// 一条映射条目都没有。I-3.11（已分配减 defer 等于最新根走读） 与 I-3.1（已分配统计对得上） 同一个理由：最新根下面没有记账树。I-9.14（树表条目的诞生 txg 跨根不变） 在这里报不适用是因为 mkfs 种的第 0 版树表是空的：
 /// 一棵树的条目都没有，跨根比不出来（有了文件版本之后也只有一条根有条目，见 `NOT_APPLICABLE_WITH_ONE_FILE_VERSION`）；
 /// I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 同一个理由，比的是根环里全部读得出的根（不按实例表判有效）：根环里每条根的树表都是
@@ -575,7 +575,10 @@ fn region_device(txg: u64) -> DeviceIdentity {
 /// 出现规则一个单元都判不到（mkfs 之后那一格的理由不同，见 `NOT_APPLICABLE_RIGHT_AFTER_MKFS`）。
 /// I-9.27（树表条目的头 ID 取值） 与 I-5.5（从 mkfs 水位建树那一版的 t1 落点） 三张没有文件的表都有：每条根的树表都是 0 条，
 /// 头 ID 一格都没有，也没有哪一版是从 mkfs 水位第一次建树的那一版（第一个文件版本之后两条都真被评估过且成立）。
-const NOT_APPLICABLE_WITHOUT_FILE: [&str; 31] = [
+/// 删除线那五条（里程碑三·3）五张没有文件的表都有：I-9.5（删除队列里的号仍在树里） 与 I-3.4（可用空间扣待删占用） 要树表里的
+/// `deleted_inodes`、inode 树与记账树，每一版树表都是 0 条（第一个文件版本之后两条都真被评估过且成立）；I-4.7（删除中 inode 有账） 与
+/// I-4.6（中间态其余不变量成立） 没有处于删除中的号；I-4.4（意图只描述目标态） 第一版恒报不适用（下面每一张表都有它）。
+const NOT_APPLICABLE_WITHOUT_FILE: [&str; 36] = [
     "I-9.27",
     "I-5.5",
     "I-1.10",
@@ -607,6 +610,11 @@ const NOT_APPLICABLE_WITHOUT_FILE: [&str; 31] = [
     "I-4.1",
     "I-3.5",
     "I-9.28",
+    "I-9.5",
+    "I-3.4",
+    "I-4.7",
+    "I-4.6",
+    "I-4.4",
 ];
 
 /// mkfs 刚写完那一份比 `NOT_APPLICABLE_WITHOUT_FILE` 多报四条不适用：journal 环整段是 0，一条自证过的记录都没有 ⇒ I-8.6（反向链算法）、
@@ -614,7 +622,8 @@ const NOT_APPLICABLE_WITHOUT_FILE: [&str; 31] = [
 /// 可写挂载之后环里有记录（写行 / 暖机的空发布），它们就真被评估过了——所以这四条只在 mkfs 之后那一格。
 /// I-3.5（引用区间的精确性） 两张表都有，这一格的理由是回退候选集只剩 mkfs 的第 0 代根一版（环里没有记录、根环里只有它），
 /// 凑不出相邻的一对。
-const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 35] = [
+/// 删除线那五条的理由同 `NOT_APPLICABLE_WITHOUT_FILE`。
+const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 40] = [
     "I-9.27",
     "I-5.5",
     "I-1.10",
@@ -650,6 +659,11 @@ const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 35] = [
     "I-4.1",
     "I-3.5",
     "I-9.28",
+    "I-9.5",
+    "I-3.4",
+    "I-4.7",
+    "I-4.6",
+    "I-4.4",
 ];
 
 /// 写完第一个文件版本之后仍报不适用的那六条（I-4.1 的理由见 `NOT_APPLICABLE_WITHOUT_FILE`）：
@@ -662,13 +676,16 @@ const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 35] = [
 /// I-8.8（前缀里的事务不被切开）——第一版一事务一条、每条都带提交标记，③ ④ 没有对象。
 /// 三条比得出来的镜像都在 `checker_known_bad_images.rs`（再覆盖写一次：树表两版、事务号 1 与 2；
 /// B 之后再接一事务两条记录的阳性对照与坏镜像）。
-const NOT_APPLICABLE_WITH_ONE_FILE_VERSION: [&str; 6] =
-    ["I-7.9", "I-8.7", "I-8.8", "I-9.14", "I-4.1", "I-9.28"];
+/// I-4.7（删除中 inode 有账） 与 I-4.6（中间态其余不变量成立）——没有删除，没有处于删除中的号、没有 nlink = 0 的记录
+/// （阳性对照与坏镜像在 `pool_checker_judges_the_deletion_state_of_every_walked_version.rs`）；I-4.4（意图只描述目标态） 第一版恒报不适用。
+const NOT_APPLICABLE_WITH_ONE_FILE_VERSION: [&str; 9] = [
+    "I-7.9", "I-8.7", "I-8.8", "I-9.14", "I-4.1", "I-9.28", "I-4.7", "I-4.6", "I-4.4",
+];
 
 /// 只有实例 1 的池上第一个文件版本之后：比 `NOT_APPLICABLE_WITH_ONE_FILE_VERSION` 多 I-3.12（实例表行 T 低于引用它的根的 txg）——
 /// 实例表一行都没有（写过行的那一份上它真被评估过且成立）。
-const NOT_APPLICABLE_WITH_ONE_FILE_VERSION_OF_THE_ONLY_INSTANCE: [&str; 7] = [
-    "I-7.9", "I-8.7", "I-8.8", "I-9.14", "I-3.12", "I-4.1", "I-9.28",
+const NOT_APPLICABLE_WITH_ONE_FILE_VERSION_OF_THE_ONLY_INSTANCE: [&str; 10] = [
+    "I-7.9", "I-8.7", "I-8.8", "I-9.14", "I-3.12", "I-4.1", "I-9.28", "I-4.7", "I-4.6", "I-4.4",
 ];
 
 /// 树表 0 条的一版上写行那次发布之后：比 `NOT_APPLICABLE_WITHOUT_FILE` 少七条——I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）、
@@ -683,7 +700,8 @@ const NOT_APPLICABLE_WITH_ONE_FILE_VERSION_OF_THE_ONLY_INSTANCE: [&str; 7] = [
 /// 判点名项与单元头逐字段相同——两条都真被评估过且成立。
 /// I-3.5（引用区间的精确性） 也不在这张表里：写行那次发布（实例 2、txg 3）把实例表 COW 到新落点、写了自己那棵分配记录树，
 /// 这几个单元前一版（实例 1 的暖机根，txg 2）不引用、诞生代号 3 大于 2，出现规则真判到了它们且成立。
-const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 24] = [
+/// 删除线那五条的理由同 `NOT_APPLICABLE_WITHOUT_FILE`。
+const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 29] = [
     "I-9.27",
     "I-5.5",
     "I-3.1",
@@ -708,6 +726,11 @@ const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 24] = [
     "I-9.26",
     "I-4.1",
     "I-9.28",
+    "I-9.5",
+    "I-3.4",
+    "I-4.7",
+    "I-4.6",
+    "I-4.4",
 ];
 
 /// 这一步的镜像上 checker 的每一条判决逐条核：`not_applicable` 里的报不适用，其余每一条都真被评估过且成立（一条违例都没有）。
@@ -1092,10 +1115,10 @@ fn plant_a_root_without_its_allocation_record_tree(
 /// 验收：取号 1、不写行；写行那次发布 (1, 1)、jsn 1、事务号 0、反向链 0、零单元、落盘 1；暖机一次 (1, 2)、jsn 2、反向链接 jsn 1、落盘 0；
 /// 分配器只有 mkfs 的两个单元（每盘 3 槽）；第一个文件版本 (1, 3)、jsn 3、事务号 1，数据单元落 50180、换下 mkfs 那片树表；
 /// mkfs 之后的录制流与新池新建文件那条逐字节相同、写出的那一版也相同；冷启动读回文件；checker 一条违例都没有（条数按今天的
-/// `IMPLEMENTED_INVARIANTS` 69 条数）——mkfs 之后 34 条成立（环是空的，I-8.6、I-8.9、I-8.10、I-8.12 也报不适用）、挂载之后 38 条成立，
-/// 没有记账树 / inode 树 / 分配记录树 / 映射条目的那几条报不适用，第一个文件版本之后 62 条成立、I-7.9、I-9.14、I-9.28、I-8.7、I-8.8、
-/// I-3.12 与 I-4.1 仍报不适用（树表条目只出现在这一条根的树表里；同一实例只有一条非 0 事务号的记录；没有哪个事务号落了两条记录、
-/// 每条都带提交标记；只有实例 1，实例表一行都没有；最新根之上没有记录）。
+/// `IMPLEMENTED_INVARIANTS` 74 条数）——mkfs 之后 34 条成立（环是空的，I-8.6、I-8.9、I-8.10、I-8.12 也报不适用）、挂载之后 38 条成立，
+/// 没有记账树 / inode 树 / 分配记录树 / 映射条目的那几条报不适用，第一个文件版本之后 64 条成立、I-7.9、I-9.14、I-9.28、I-8.7、I-8.8、
+/// I-3.12、I-4.1、I-4.7、I-4.6 与 I-4.4 仍报不适用（树表条目只出现在这一条根的树表里；同一实例只有一条非 0 事务号的记录；没有哪个事务号落了两条记录、
+/// 每条都带提交标记；只有实例 1，实例表一行都没有；最新根之上没有记录；没有删除；I-4.4 第一版恒报不适用）。
 #[test]
 fn writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold(
 ) {
````

### p1：mutations-append.tsv

````
三·3 I-9.5 正方向恒真：队列里的号不核记录与 nlink（队列里没有记录的号放过）	crates/singlefs-checker/src/walk.rs	first_queued_inode_without_a_record_of_nlink_zero.is_none(),	true,	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	a_queue_number_without_an_inode_record_reddens_only_the_deletion_queue_invariant
三·3 I-9.5 反方向恒真：nlink = 0 的记录不核队列（僵尸记录放过）	crates/singlefs-checker/src/walk.rs	first_record_of_nlink_zero_outside_the_queue.is_none(),	true,	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_inode_record_with_nlink_zero_outside_the_queue_reddens_only_the_deletion_queue_invariant
三·3 I-9.5 反方向恒真：形式化线 being-deleted 那一形放过	crates/singlefs-checker/src/walk.rs	first_record_of_nlink_zero_outside_the_queue.is_none(),	true,	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	a_new_inode_published_with_nlink_zero_as_in_the_formal_line_being_deleted_variant_reddens_the_deletion_queue_invariant
三·3 I-4.7 对象漏了有数据单元而没有 nlink ≥ 1 记录的号（孤儿单元放过）	crates/singlefs-checker/src/walk.rs	**data_units > 0 && !records.get(inode).is_some_and(|record| record.nlink >= 1)	false	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	data_units_of_an_inode_without_a_record_outside_the_queue_redden_only_the_inode_being_deleted_invariant
三·3 I-4.6 单元数与 size 不比	crates/singlefs-checker/src/walk.rs	data_units == data_units_the_size_asks_for	true	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_inode_being_deleted_whose_size_asks_for_more_data_units_than_it_has_reddens_only_the_intermediate_state_invariant
三·3 I-4.6 按 size 除净荷容量向下取整比（意图那一版半个单元的尾巴误红）	crates/singlefs-checker/src/walk.rs	size_in_bytes.div_ceil(payload_capacity_in_bytes)	size_in_bytes / payload_capacity_in_bytes	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	every_published_step_of_unlinking_a_three_unit_file_holds_the_deletion_invariants
三·3 I-3.4 待删占用那一行的值不比 0	crates/singlefs-checker/src/walk.rs	judgements.judge(invariant, value == 0, || {	judgements.judge(invariant, true, || {	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	a_pending_deletion_occupancy_row_that_is_not_zero_reddens_only_the_pending_deletion_invariant
三·3 I-3.4 记账树缺待删占用那一行不判红	crates/singlefs-checker/src/walk.rs	judgements.judge(invariant, has_the_pending_deletion_row, || {	judgements.judge(invariant, true, || {	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_accounting_tree_without_the_pending_deletion_occupancy_row_reddens_only_the_pending_deletion_invariant
三·3 I-9.6 墓碑那一半不比水位	crates/singlefs-checker/src/walk.rs	watermark > largest_tombstone_object	true	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	a_tombstone_whose_object_identifier_reaches_the_inode_number_watermark_reddens_only_the_watermark_invariant
三·3 删除态扫描只扫最新根（中间那一版的记账行漏判）	crates/singlefs-checker/src/walk.rs	        candidate_indexes\n            .iter()\n            .map(|index| {\n                let root = &roots[*index].2;\n                VersionToScanTheDeletionStateOf {	        [newest_index]\n            .iter()\n            .map(|index| {\n                let root = &roots[*index].2;\n                VersionToScanTheDeletionStateOf {	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_intermediate_deletion_batch_with_a_non_zero_pending_deletion_row_reddens_while_the_newest_root_is_clean
三·3 删除态扫描只扫最新根（中间那一版的 size 漏判）	crates/singlefs-checker/src/walk.rs	        candidate_indexes\n            .iter()\n            .map(|index| {\n                let root = &roots[*index].2;\n                VersionToScanTheDeletionStateOf {	        [newest_index]\n            .iter()\n            .map(|index| {\n                let root = &roots[*index].2;\n                VersionToScanTheDeletionStateOf {	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_intermediate_deletion_batch_whose_size_asks_for_more_data_units_reddens_while_the_newest_root_is_clean
三·3 删除态扫描把读不出的 inode 叶当成空叶（读不全的那一版判红）	crates/singlefs-checker/src/walk.rs	                let container =\n                    read_packed_unit_without_judging(reader, &child_pointer, packed_cache)\n                        .ok_or(DeletionScanGap::InodeTreeUnreadable)?;	                let Some(container) =\n                    read_packed_unit_without_judging(reader, &child_pointer, packed_cache)\n                else {\n                    continue;\n                };	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_intermediate_version_whose_inode_leaf_does_not_read_back_is_not_judged_rather_than_red
三·3 I-4.4 不报第一版不适用的理由	crates/singlefs-checker/src/walk.rs	    judgements.not_applicable(\n        INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,\n        INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION,\n    );	    // 变异：I-4.4 不报理由	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	every_published_step_of_unlinking_a_three_unit_file_holds_the_deletion_invariants
三·3 删除态扫描把 nlink 读成 gid（偏移 32）	crates/singlefs-checker/src/walk.rs	const INODE_RECORD_NLINK_OFFSET: usize = 36;	const INODE_RECORD_NLINK_OFFSET: usize = 32;	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_inode_record_with_nlink_zero_outside_the_queue_reddens_only_the_deletion_queue_invariant
````

## 实现员 p1fix：crates.patch（/tmp/claude-1000/m3-batch18/p1fix/patch/crates.patch）

````diff
diff --git a/crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs b/crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs
index a815912..87cc3c1 100644
--- a/crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs
+++ b/crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs
@@ -116,7 +116,7 @@ struct Remount {
 
 fn remount(tag: &str, account: Account, unreadability: Unreadability) -> Remount {
     let (mut pool, abandoned) = pool_after_a_recovery_abandoned_c(tag);
-    let unit = abandoned.unit(account.unit());
+    let account_unit = abandoned.unit(account.unit());
     let failing = match unreadability {
         Unreadability::Readable => FailingReadsOfARange::Never,
         Unreadability::EveryRead | Unreadability::EveryReadUntilTheRereadHook => {
@@ -128,8 +128,8 @@ fn remount(tag: &str, account: Account, unreadability: Unreadability) -> Remount
         .into_iter()
         .map(|device| UnreadableRange {
             device,
-            offset_in_bytes: unit.slot.0 * SLOT_BYTES,
-            length_in_bytes: u64::try_from(unit.bytes.len()).expect("长度"),
+            offset_in_bytes: account_unit.slot.0 * SLOT_BYTES,
+            length_in_bytes: u64::try_from(account_unit.bytes.len()).expect("长度"),
             failing_reads: failing,
         })
         .collect();
@@ -156,7 +156,7 @@ fn remount(tag: &str, account: Account, unreadability: Unreadability) -> Remount
     let mut remount = Remount {
         pool,
         abandoned: abandoned.clone(),
-        account_slot: unit.slot.0,
+        account_slot: account_unit.slot.0,
         result: Ok(()),
         isolated: Vec::new(),
         abandoned_roots_unreadable: 0,
diff --git a/crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs b/crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs
index 35b24f5..cfab66f 100644
--- a/crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs
+++ b/crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs
@@ -166,8 +166,10 @@ fn assert_the_rebuild_reproduces(unit_bytes: &[u8], header: &IndexNodeHeader) {
 
 /// 第一个文件版本的 extent 根兼叶里，把那条内联数据指针抄一份给下一个 inode 号：同一个根里两个 inode 的 extent 指向同一个数据单元。
 /// 改之前的 checker 在第二次遇到这个单元时直接跳过（整个走读共用一份「走过的单元」），五元组、出生身份、I-5.1 全不判，全绿。
+/// 抄出来的下一个 inode 号在 inode 树里没有记录，extent 树里却有数据单元、号也不在 `deleted_inodes` 里：照 I-4.7 的对象（取丙）
+/// 它处于删除中而没有账，I-4.7 跟着红；判红的恰好是 I-5.1 与 I-4.7 两条。
 #[test]
-fn two_inodes_whose_extents_name_one_data_unit_in_one_root_redden_only_the_disjoint_ranges_invariant(
+fn two_inodes_whose_extents_name_one_data_unit_in_one_root_redden_the_disjoint_ranges_and_the_deletion_accounting_invariants(
 ) {
     let pool = product_capacity_pool();
     let clean = pool.memory_pool();
@@ -220,13 +222,29 @@ fn two_inodes_whose_extents_name_one_data_unit_in_one_root_redden_only_the_disjo
     let red = violations(&image);
     assert_eq!(
         violated_names(&red),
-        ["I-5.1"],
-        "判红的该只有 I-5.1：{red:?}"
+        ["I-5.1", "I-4.7"],
+        "判红的该恰好是 I-5.1 与 I-4.7：{red:?}"
     );
+    let disjoint_ranges_detail = &red
+        .iter()
+        .find(|(name, _)| *name == "I-5.1")
+        .expect("上一条断言钉了 I-5.1 判红")
+        .1;
     assert!(
-        red[0].1.contains("被同一版引用了两次"),
-        "说明要点出同一版里的第二次引用：{}",
-        red[0].1
+        disjoint_ranges_detail.contains("被同一版引用了两次"),
+        "说明要点出同一版里的第二次引用：{disjoint_ranges_detail}"
+    );
+    let inode_without_a_record = inode + 1;
+    let deletion_accounting_detail = &red
+        .iter()
+        .find(|(name, _)| *name == "I-4.7")
+        .expect("上一条断言钉了 I-4.7 判红")
+        .1;
+    assert!(
+        deletion_accounting_detail.contains(&format!(
+            "inode {inode_without_a_record} 处于删除中（inode 树里没有它的记录）"
+        )),
+        "I-4.7 的说明要点出没有记录的 inode {inode_without_a_record}：{deletion_accounting_detail}"
     );
 }
 
````

### p1fix：mutations-append.tsv

````
第 18 批 P1 修：I-4.7 一判恒成立（同一版两个 inode 指同一数据单元的坏镜像里，没有记录的 inode 2 不再判删除中没账）	crates/singlefs-checker/src/walk.rs	            queue.contains(&inode) || data_units_left == 0,	            true || queue.contains(&inode) || data_units_left == 0,	-p singlefs-harness --test checker_cross_links_malformed_nodes_and_mapping_entries -- two_inodes_whose_extents_name_one_data_unit_in_one_root_redden_the_disjoint_ranges_and_the_deletion_accounting_invariants	two_inodes_whose_extents_name_one_data_unit_in_one_root_redden_the_disjoint_ranges_and_the_deletion_accounting_invariants
````

### p1fix：mutations-replacements.tsv

````
实审 B1 第 5 条：checker 同一版里第二次引用同一个落点判成立（两个 inode 的 extent 指同一个数据单元照绿）	crates/singlefs-checker/src/walk.rs	                self.judgements.judge("I-5.1", false, || {	                self.judgements.judge("I-5.1", true, || {	-p singlefs-harness --test checker_cross_links_malformed_nodes_and_mapping_entries -- two_inodes_whose_extents_name_one_data_unit_in_one_root_redden_the_disjoint_ranges_and_the_deletion_accounting_invariants	two_inodes_whose_extents_name_one_data_unit_in_one_root_redden_the_disjoint_ranges_and_the_deletion_accounting_invariants
````

## 实现员 p2：crates.patch（/tmp/claude-1000/m3-batch18/p2/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
index 80f6dab..33b5b85 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
@@ -148,6 +148,17 @@ const INVARIANT_I_5_5: u32 = 67u;
 // 第 69 条：I-9.28（全部可读根里同一树 ID 的诞生 txg 相同），扫描内核挨着 I-9.14 判
 const INVARIANT_I_9_28: u32 = 68u;
 
+// 第 70–74 条：里程碑三·3 删除线那五条（只往末尾追加），扫描内核的删除态扫描判前四条；I-4.4 第一版报不适用，内核不置它那一位
+const INVARIANT_I_9_5: u32 = 69u;
+
+const INVARIANT_I_3_4: u32 = 70u;
+
+const INVARIANT_I_4_7: u32 = 71u;
+
+const INVARIANT_I_4_6: u32 = 72u;
+
+const INVARIANT_I_4_4: u32 = 73u;
+
 // 墓碑树、谱系树叶容器的打包记录类型（walk.rs 的 PACKED_TYPE_TOMBSTONE / PACKED_TYPE_LINEAGE；登记的记录宽墓碑 40、谱系 16）
 const PACKED_TYPE_TOMBSTONE: u32 = 1u;
 
@@ -746,8 +757,23 @@ const C_APPEARANCE_MAX_WALKED_TREE_TABLES: u32 = 64u;
 
 const C_APPEARANCE_WALKED_TREE_TABLES: u32 = 287728u;
 
+// ───────────────────────── 走读内核那几张表之后：扫描内核的删除态扫描（walk.rs judge_deletion_state_of_the_versions）一版一版重填的两张表 ─────────────────────────
+// ① 这一版 inode 树的记录（walk.rs inode_records_without_judging）：号 lo/hi、nlink、size lo/hi；同一个号再读到时后读到的盖掉先读到的
+const D_INODE_RECORD_WORDS: u32 = 5u;
+
+const D_MAX_INODE_RECORDS: u32 = 1024u;
+
+const D_INODE_RECORDS: u32 = 290544u;
+
+// ② 这一版 extent 树里每个号的数据单元数（walk.rs data_unit_count_of_each_inode_without_judging，全零数据指针不数）：号 lo/hi、单元数
+const D_DATA_UNIT_COUNT_WORDS: u32 = 3u;
+
+const D_MAX_DATA_UNIT_INODES: u32 = 1024u;
+
+const D_DATA_UNIT_COUNTS: u32 = 295664u;
+
 // 每个状态的草稿区到这里为止：crash_judge_gpu.rs 的 SCRATCH_WORDS 与它相等（那一份的单测核）
-const SCRATCH_END: u32 = 290544u;
+const SCRATCH_END: u32 = 298736u;
 
 fn handoff_region_device(region: u32) -> u32 {
     if (region == 0u) {
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
index 6788988..df4d8ef 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -1,6 +1,7 @@
 // GPU 判器的池级 checker 扫描内核（接在 crash_judge_common.wgsl 与 crash_judge_checker_common.wgsl 之后）：走读内核判完、交接段有效时，
 // 判第三批——扫描方向与记账那九条（I-1.8、I-3.9、I-9.14、I-5.4、I-3.10、I-7.9、I-3.1、I-5.2、I-3.11），再加 I-3.12（每条根各自那张实例表）、
-// I-5.5 与 I-9.28（根表里每一条根的树表），违例位并进违例位那几个字，走到第几步那一字写成 5、判不了时下一字写原因。只读不判的读法照 walk.rs 的 *_without_judging 各写一份。
+// I-5.5 与 I-9.28（根表里每一条根的树表），再加里程碑三·3 删除线的删除态扫描（I-9.5、I-3.4、I-4.7、I-4.6，回退候选集每一版）与 I-9.6 墓碑那一半
+// （所选根的墓碑树），违例位并进违例位那几个字，走到第几步那一字写成 5、判不了时下一字写原因。只读不判的读法照 walk.rs 的 *_without_judging 各写一份。
 
 
 var<private> r_complete: bool;
@@ -71,6 +72,34 @@ fn r_note_data_pointer(p: Ptr) {
     r_note_data_unit_slot(p.device1, p.slot1);
 }
 
+// 删除态扫描那一遍开着时，r_extent_tree 读到的每条数据指针按它属于的 inode 号数一个（walk.rs data_unit_count_of_each_inode_without_judging：
+// 上段叶内联的那一个属于条目 key 里的号，下段叶的每条记录属于引它那条上段条目的号；全零的数据指针不指单元，不数），记进 D_DATA_UNIT_COUNTS
+var<private> r_counting_data_units_of_each_inode: bool;
+
+var<private> d_data_unit_inode_count: u32;
+
+fn r_count_data_unit_of_inode(inode: vec2<u32>, p: Ptr) {
+    if (!r_counting_data_units_of_each_inode || p.all_zero) {
+        return;
+    }
+    for (var i = 0u; i < d_data_unit_inode_count; i = i + 1u) {
+        let row = D_DATA_UNIT_COUNTS + i * D_DATA_UNIT_COUNT_WORDS;
+        if (sw(row) == inode.x && sw(row + 1u) == inode.y) {
+            sws(row + 2u, sw(row + 2u) + 1u);
+            return;
+        }
+    }
+    if (d_data_unit_inode_count >= D_MAX_DATA_UNIT_INODES) {
+        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (48u << 8u);
+        return;
+    }
+    let row = D_DATA_UNIT_COUNTS + d_data_unit_inode_count * D_DATA_UNIT_COUNT_WORDS;
+    sws(row, inode.x);
+    sws(row + 1u, inode.y);
+    sws(row + 2u, 1u);
+    d_data_unit_inode_count = d_data_unit_inode_count + 1u;
+}
+
 // read_index_node_without_judging：第一份校验和对得上的那一份，按 index_node_view 解
 fn r_read_index_node(p: Ptr) -> Node {
     let s = read_unit_via_locations(p, NODE_BYTES);
@@ -278,7 +307,9 @@ fn r_extent_tree(base: u32, offset: u32) -> bool {
                         sws(child + 8u, entry_inode.y);
                         pending = pending + 1u;
                     } else if (tag == 2u) {
-                        r_note_data_pointer(parse_data_pointer(n.base, o + 25u));
+                        let inline_data_pointer = parse_data_pointer(n.base, o + 25u);
+                        r_note_data_pointer(inline_data_pointer);
+                        r_count_data_unit_of_inode(extent_key_inode(n.base, o), inline_data_pointer);
                     } else {
                         return false;
                     }
@@ -288,7 +319,9 @@ fn r_extent_tree(base: u32, offset: u32) -> bool {
                     return false;
                 }
                 for (var e = 0u; e < n.entry_count; e = e + 1u) {
-                    r_note_data_pointer(parse_data_pointer(n.base, entry_offset(n, e) + 24u));
+                    let lower_data_pointer = parse_data_pointer(n.base, entry_offset(n, e) + 24u);
+                    r_note_data_pointer(lower_data_pointer);
+                    r_count_data_unit_of_inode(inode, lower_data_pointer);
                 }
             }
             continue;
@@ -2016,6 +2049,474 @@ fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
     }
 }
 
+// ───────────────────────── 删除态扫描（walk.rs judge_deletion_state_of_the_versions）与 I-9.6 墓碑那一半 ─────────────────────────
+// I-9.5、I-3.4、I-4.7、I-4.6 在回退候选集里每条根与由记录施加出来的那几版上各重读一遍树表与几棵树（只读）。CPU 记成射程里判不了的那几格
+// （树表读不出、条目宽不到 200、deleted_inodes 的头 ID 配不上恰好一条 inode 树、队列根读不出或形状不对、inode 树 / extent 树 / 记账树读不全、
+// 头 ID 指着这棵 inode 树的 extent 树不是恰好一条）在这里一格都不置位；CPU 按树表单元去重只扫一次，违例位是并集，这里不去重。
+// I-4.4 第一版报不适用（check_pool_image 恒报），内核不置它那一位。
+
+// inode 记录里 nlink（4 字节）与 size（8 字节）的偏移、inode 记录宽（walk.rs INODE_RECORD_NLINK_OFFSET / INODE_RECORD_SIZE_OFFSET，类型 2 登记 140）
+const DELETION_INODE_RECORD_NLINK_OFFSET: u32 = 36u;
+const DELETION_INODE_RECORD_SIZE_OFFSET: u32 = 40u;
+const DELETION_PACKED_TYPE_INODE: u32 = 2u;
+const DELETION_INODE_RECORD_BYTES: u32 = 140u;
+// inode 树与墓碑树码 2 节点条目里的类型段（偏移 16，2 字节）：0 指下一层码 2 节点，2 指 inode 叶容器（墓碑树里 1 指墓碑容器）
+const DELETION_CONTAINER_TREE_ENTRY_TYPE_OFFSET: u32 = 16u;
+const DELETION_CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE: u32 = 0u;
+// 墓碑记录宽（类型 1 登记 40）与对象 ID 的偏移（walk.rs TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET）
+const DELETION_TOMBSTONE_RECORD_BYTES: u32 = 40u;
+const DELETION_TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET: u32 = 0u;
+// 记账树：key 22（统计量 2、树 8、设备 4、代 8），叶条目 34，内部条目 key + 子指针 86；行里设备段在 10、值在 22
+const DELETION_ACCOUNTING_KEY_BYTES: u32 = 22u;
+const DELETION_ACCOUNTING_ENTRY_BYTES: u32 = 34u;
+const DELETION_ACCOUNTING_INTERNAL_ENTRY_BYTES: u32 = 108u;
+const DELETION_ACCOUNTING_ROW_DEVICE_OFFSET: u32 = 10u;
+const DELETION_ACCOUNTING_ROW_VALUE_OFFSET: u32 = 22u;
+// 记账统计量的行号：4 待删除但意图未完成占用（walk.rs STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES）、12 inode 号水位（STATISTIC_INODE_WATERMARK）
+const DELETION_STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES: u32 = 4u;
+const DELETION_STATISTIC_INODE_WATERMARK: u32 = 12u;
+// 树表条目宽（字段表 200）、条目里种类与根指针的偏移
+const DELETION_TREE_TABLE_ENTRY_BYTES: u32 = 200u;
+const DELETION_TREE_TABLE_ENTRY_KIND_OFFSET: u32 = 10u;
+const DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET: u32 = 14u;
+
+var<private> d_inode_record_count: u32;
+
+// 这一版的一条 inode 记录记进 D_INODE_RECORDS：同一个号已有一行就盖掉（walk.rs 是按号插入的 BTreeMap，后读到的盖掉先读到的）；表满交回 false
+fn d_note_inode_record(inode: vec2<u32>, nlink: u32, size_in_bytes: vec2<u32>) -> bool {
+    for (var i = 0u; i < d_inode_record_count; i = i + 1u) {
+        let row = D_INODE_RECORDS + i * D_INODE_RECORD_WORDS;
+        if (sw(row) == inode.x && sw(row + 1u) == inode.y) {
+            sws(row + 2u, nlink);
+            sws(row + 3u, size_in_bytes.x);
+            sws(row + 4u, size_in_bytes.y);
+            return true;
+        }
+    }
+    if (d_inode_record_count >= D_MAX_INODE_RECORDS) {
+        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (49u << 8u);
+        return false;
+    }
+    let row = D_INODE_RECORDS + d_inode_record_count * D_INODE_RECORD_WORDS;
+    sws(row, inode.x);
+    sws(row + 1u, inode.y);
+    sws(row + 2u, nlink);
+    sws(row + 3u, size_in_bytes.x);
+    sws(row + 4u, size_in_bytes.y);
+    d_inode_record_count = d_inode_record_count + 1u;
+    return true;
+}
+
+// 这一版 inode 记录表里号 inode 的那一行（没有交回 NONE）
+fn d_inode_record_row_of(inode: vec2<u32>) -> u32 {
+    for (var i = 0u; i < d_inode_record_count; i = i + 1u) {
+        let row = D_INODE_RECORDS + i * D_INODE_RECORD_WORDS;
+        if (sw(row) == inode.x && sw(row + 1u) == inode.y) {
+            return row;
+        }
+    }
+    return NONE;
+}
+
+// 这一版 extent 树里号 inode 的数据单元数（没有的号是 0）
+fn d_data_units_of_inode(inode: vec2<u32>) -> u32 {
+    for (var i = 0u; i < d_data_unit_inode_count; i = i + 1u) {
+        let row = D_DATA_UNIT_COUNTS + i * D_DATA_UNIT_COUNT_WORDS;
+        if (sw(row) == inode.x && sw(row + 1u) == inode.y) {
+            return sw(row + 2u);
+        }
+    }
+    return 0u;
+}
+
+// 码 2 节点 node 的条目逐个压进 C_R_PENDING（倒着压、弹出来是条目序：walk.rs collect_inode_records_below 按条目序往下走，同号后读到的盖掉先读到的，
+// 次序要一样）：每行是条目所在的字节基、条目偏移、节点层级。压满交回 false（判不了，交 CPU）
+fn d_push_container_tree_entries_in_reverse(node: Node, pending: ptr<function, u32>, site: u32) -> bool {
+    for (var k = 0u; k < node.entry_count; k = k + 1u) {
+        if (*pending >= C_R_MAX_PENDING) {
+            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (site << 8u);
+            return false;
+        }
+        let row = C_R_PENDING + *pending * C_R_PENDING_WORDS;
+        sws(row, node.base);
+        sws(row + 1u, entry_offset(node, node.entry_count - 1u - k));
+        sws(row + 2u, node.level);
+        *pending = *pending + 1u;
+    }
+    return true;
+}
+
+// walk.rs inode_records_without_judging：一版 inode 树的全部记录读进 D_INODE_RECORDS。根指针全零是零条记录；根层级 0 只许是零条目根；
+// 层级 ≥ 1 的往下走：类型段 2 的叶容器（码 3、记录类型 2、记录宽 ≥ 140）读每条记录，类型段 0 的内部节点（子指针非零、层级 = 父层级 − 1 且 ≥ 1、
+// 条目宽 ≥ 120）往下走，别的类型段、读不出、形状不对都交回 false（判不了）
+fn d_read_inode_records(root_pointer: Ptr) -> bool {
+    d_inode_record_count = 0u;
+    if (root_pointer.all_zero) {
+        return true;
+    }
+    let root = r_read_index_node(root_pointer);
+    if (!root.ok) {
+        return false;
+    }
+    if (root.level == 0u) {
+        return root.entry_count == 0u;
+    }
+    if (root.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
+        return false;
+    }
+    var pending = 0u;
+    if (!d_push_container_tree_entries_in_reverse(root, &pending, 50u)) {
+        return false;
+    }
+    while (pending > 0u) {
+        pending = pending - 1u;
+        let row = C_R_PENDING + pending * C_R_PENDING_WORDS;
+        let base = sw(row);
+        let o = sw(row + 1u);
+        let parent_level = sw(row + 2u);
+        let child_pointer = parse_node_pointer(base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
+        let entry_type = rd16(base, o + DELETION_CONTAINER_TREE_ENTRY_TYPE_OFFSET);
+        if (entry_type == DELETION_PACKED_TYPE_INODE) {
+            if (child_pointer.all_zero) {
+                return false;
+            }
+            let container = parse_packed_unit(read_unit_via_locations(child_pointer, DATA_UNIT_BYTES));
+            if (!container.ok || container.record_type != DELETION_PACKED_TYPE_INODE
+                || container.record_width < DELETION_INODE_RECORD_BYTES) {
+                return false;
+            }
+            for (var r = 0u; r < container.record_count; r = r + 1u) {
+                let record = packed_record_offset(container, r);
+                if (!d_note_inode_record(rd64(container.base, record), rd32(container.base, record + DELETION_INODE_RECORD_NLINK_OFFSET),
+                    rd64(container.base, record + DELETION_INODE_RECORD_SIZE_OFFSET))) {
+                    return false;
+                }
+            }
+        } else if (entry_type == DELETION_CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE) {
+            if (child_pointer.all_zero) {
+                return false;
+            }
+            let child = r_read_index_node(child_pointer);
+            if (!child.ok || child.level + 1u != parent_level || child.level == 0u
+                || child.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
+                return false;
+            }
+            if (!d_push_container_tree_entries_in_reverse(child, &pending, 51u)) {
+                return false;
+            }
+        } else {
+            return false;
+        }
+    }
+    return true;
+}
+
+// 队列根 queue（层级 0、key 宽 8 的码 2 根，条目前 8 字节是号）里有没有号 inode；queue_present 为假是根指针全零的空队列
+fn d_queue_contains(queue: Node, queue_present: bool, inode: vec2<u32>) -> bool {
+    if (!queue_present) {
+        return false;
+    }
+    for (var e = 0u; e < queue.entry_count; e = e + 1u) {
+        if (eq64(rd64(queue.base, entry_offset(queue, e)), inode)) {
+            return true;
+        }
+    }
+    return false;
+}
+
+// walk.rs judge_pending_deletion_occupancy_of_a_version 的一棵记账树（根指针在 (base, offset)，非零）：整棵读（key 宽 22；叶条目宽 ≥ 34；
+// 内部条目宽 ≥ 108、子指针非零、孩子层级 = 父层级 − 1），走全了才判——要有标签 4、设备段 0xFFFF_FFFF 的那一行（缺行判红），
+// 每一行标签 4 的值都是 0；读不全的不判
+fn d_judge_pending_deletion_occupancy_of_an_accounting_tree(base: u32, offset: u32) {
+    var has_the_pending_deletion_row = false;
+    var some_pending_deletion_value_is_not_zero = false;
+    var pending = 1u;
+    sws(C_R_PENDING, base);
+    sws(C_R_PENDING + 1u, offset);
+    sws(C_R_PENDING + 2u, NONE);   // 根：层级取它自己头里的
+    while (pending > 0u) {
+        pending = pending - 1u;
+        let row = C_R_PENDING + pending * C_R_PENDING_WORDS;
+        let p = parse_node_pointer(sw(row), sw(row + 1u));
+        let expected_level = sw(row + 2u);
+        let n = r_read_index_node(p);
+        if (!n.ok || (expected_level != NONE && n.level != expected_level) || n.key_width != DELETION_ACCOUNTING_KEY_BYTES) {
+            return;
+        }
+        if (n.level == 0u) {
+            if (n.entry_width < DELETION_ACCOUNTING_ENTRY_BYTES) {
+                return;
+            }
+            for (var e = 0u; e < n.entry_count; e = e + 1u) {
+                let o = entry_offset(n, e);
+                if (rd16(n.base, o) != DELETION_STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES) {
+                    continue;
+                }
+                if (rd32(n.base, o + DELETION_ACCOUNTING_ROW_DEVICE_OFFSET) == NONE) {
+                    has_the_pending_deletion_row = true;
+                }
+                if (!is_zero64(rd64(n.base, o + DELETION_ACCOUNTING_ROW_VALUE_OFFSET))) {
+                    some_pending_deletion_value_is_not_zero = true;
+                }
+            }
+            continue;
+        }
+        if (n.entry_width < DELETION_ACCOUNTING_INTERNAL_ENTRY_BYTES) {
+            return;
+        }
+        for (var e = 0u; e < n.entry_count; e = e + 1u) {
+            let o = entry_offset(n, e) + DELETION_ACCOUNTING_KEY_BYTES;
+            if (parse_node_pointer(n.base, o).all_zero) {
+                return;
+            }
+            if (pending >= C_R_MAX_PENDING) {
+                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (52u << 8u);
+                return;
+            }
+            let child = C_R_PENDING + pending * C_R_PENDING_WORDS;
+            sws(child, n.base);
+            sws(child + 1u, o);
+            sws(child + 2u, n.level - 1u);
+            pending = pending + 1u;
+        }
+    }
+    if (!has_the_pending_deletion_row || some_pending_deletion_value_is_not_zero) {
+        violate(INVARIANT_I_3_4);
+    }
+}
+
+// walk.rs judge_deletion_queue_of_a_version 里一条 deleted_inodes 条目（树表 tt 的条目偏移 deleted_inodes_entry）：经偏移 116 的头 ID 配同一张表里
+// 树 ID 等于它的恰好一条 inode 树条目，读队列与记录判 I-9.5 两个方向；再配头 ID 等于这棵 inode 树的恰好一条 extent 树条目，数每个号的数据单元判
+// I-4.7（对象取丙）、I-4.6（取乙）
+fn d_judge_deletion_queue_of_an_entry(tt: Node, deleted_inodes_entry: u32) {
+    let inode_tree = rd64(tt.base, deleted_inodes_entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
+    var inode_tree_entry_count = 0u;
+    var inode_tree_entry = 0u;
+    var extent_tree_entry_count = 0u;
+    var extent_tree_entry = 0u;
+    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
+        let entry = entry_offset(tt, entry_index);
+        let kind = rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET);
+        if (kind == TREE_KIND_INODE && eq64(rd64(tt.base, entry), inode_tree)) {
+            inode_tree_entry_count = inode_tree_entry_count + 1u;
+            inode_tree_entry = entry;
+        }
+        if (kind == TREE_KIND_EXTENT && eq64(rd64(tt.base, entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET), inode_tree)) {
+            extent_tree_entry_count = extent_tree_entry_count + 1u;
+            extent_tree_entry = entry;
+        }
+    }
+    if (inode_tree_entry_count != 1u) {
+        return;
+    }
+    // 队列（walk.rs deleted_inodes_queue_without_judging）：根指针全零是空队列；否则要读得出、层级 0、key 宽 8、条目宽 ≥ 8
+    let queue_pointer = parse_node_pointer(tt.base, deleted_inodes_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET);
+    let queue_present = !queue_pointer.all_zero;
+    var queue = bad_node();
+    if (queue_present) {
+        queue = r_read_index_node(queue_pointer);
+        if (!queue.ok || queue.level != 0u || queue.key_width != DELETED_INODES_ENTRY_BYTES || queue.entry_width < DELETED_INODES_ENTRY_BYTES) {
+            return;
+        }
+    }
+    if (!d_read_inode_records(parse_node_pointer(tt.base, inode_tree_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET))) {
+        return;
+    }
+    // I-9.5 ①：队列里的每个号在这棵 inode 树里有记录且 nlink = 0
+    if (queue_present) {
+        for (var e = 0u; e < queue.entry_count; e = e + 1u) {
+            let row = d_inode_record_row_of(rd64(queue.base, entry_offset(queue, e)));
+            if (row == NONE || sw(row + 2u) != 0u) {
+                violate(INVARIANT_I_9_5);
+            }
+        }
+    }
+    // I-9.5 ②：nlink = 0 的每条记录，号在队列里
+    for (var i = 0u; i < d_inode_record_count; i = i + 1u) {
+        let row = D_INODE_RECORDS + i * D_INODE_RECORD_WORDS;
+        if (sw(row + 2u) == 0u && !d_queue_contains(queue, queue_present, vec2<u32>(sw(row), sw(row + 1u)))) {
+            violate(INVARIANT_I_9_5);
+        }
+    }
+    if (extent_tree_entry_count != 1u) {
+        return;
+    }
+    // 每个号的数据单元数（walk.rs data_unit_count_of_each_inode_without_judging）：根指针全零的树一个单元都没有
+    d_data_unit_inode_count = 0u;
+    let extent_root_offset = extent_tree_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
+    if (!parse_node_pointer(tt.base, extent_root_offset).all_zero) {
+        r_noting = false;
+        r_collecting_data_unit_slots = false;
+        r_counting_data_units_of_each_inode = true;
+        let read_the_whole_extent_tree = r_extent_tree(tt.base, extent_root_offset);
+        r_counting_data_units_of_each_inode = false;
+        if (!read_the_whole_extent_tree || undecidable != 0u) {
+            return;
+        }
+    }
+    // I-4.7：处于删除中的号（nlink = 0 的记录的号，与有数据单元而没有 nlink ≥ 1 记录的号）要么在队列里，要么一个数据单元都不剩——
+    // 单元数为 0 的号自己就成立，所以只看有单元的号
+    for (var i = 0u; i < d_data_unit_inode_count; i = i + 1u) {
+        let count_row = D_DATA_UNIT_COUNTS + i * D_DATA_UNIT_COUNT_WORDS;
+        let inode = vec2<u32>(sw(count_row), sw(count_row + 1u));
+        let record_row = d_inode_record_row_of(inode);
+        let has_a_record_of_nlink_one_or_more = record_row != NONE && sw(record_row + 2u) >= 1u;
+        if (sw(count_row + 2u) > 0u && !has_a_record_of_nlink_one_or_more && !d_queue_contains(queue, queue_present, inode)) {
+            violate(INVARIANT_I_4_7);
+        }
+    }
+    // I-4.6：每条 nlink = 0 的记录，size 为 0 时数据单元 0 或 1 个，size > 0 时 ⌈size ÷ 净荷容量⌉ 个
+    for (var i = 0u; i < d_inode_record_count; i = i + 1u) {
+        let row = D_INODE_RECORDS + i * D_INODE_RECORD_WORDS;
+        if (sw(row + 2u) != 0u) {
+            continue;
+        }
+        let data_units = d_data_units_of_inode(vec2<u32>(sw(row), sw(row + 1u)));
+        let size_in_bytes = vec2<u32>(sw(row + 3u), sw(row + 4u));
+        var matches_the_size = data_units <= 1u;
+        if (!is_zero64(size_in_bytes)) {
+            let division = divmod64(size_in_bytes, u64_of(DATA_UNIT_PAYLOAD_CAPACITY));
+            var data_units_the_size_asks_for = division.quotient;
+            if (!is_zero64(division.remainder)) {
+                data_units_the_size_asks_for = add64(data_units_the_size_asks_for, u64_of(1u));
+            }
+            matches_the_size = eq64(u64_of(data_units), data_units_the_size_asks_for);
+        }
+        if (!matches_the_size) {
+            violate(INVARIANT_I_4_6);
+        }
+    }
+}
+
+// walk.rs judge_deletion_state_of_the_versions 的一版：树表指针在 (base, offset)。指针全零、树表 0 条的不判；树表读不出、条目宽不到 200 的不判
+fn d_judge_deletion_state_of_a_version(base: u32, tree_table_pointer_offset: u32) {
+    let tree_table_pointer = parse_node_pointer(base, tree_table_pointer_offset);
+    if (tree_table_pointer.all_zero) {
+        return;
+    }
+    let tt = r_read_index_node(tree_table_pointer);
+    if (!tt.ok || tt.entry_count == 0u || tt.entry_width < DELETION_TREE_TABLE_ENTRY_BYTES) {
+        return;
+    }
+    // I-3.4（walk.rs judge_pending_deletion_occupancy_of_a_version）：每一棵根指针非零的记账树
+    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
+        let entry = entry_offset(tt, entry_index);
+        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) != TREE_KIND_ACCOUNTING
+            || parse_node_pointer(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET).all_zero) {
+            continue;
+        }
+        d_judge_pending_deletion_occupancy_of_an_accounting_tree(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET);
+    }
+    // I-9.5、I-4.7、I-4.6（walk.rs judge_deletion_queue_of_a_version）：每一条 deleted_inodes 条目
+    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
+        let entry = entry_offset(tt, entry_index);
+        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_DELETED_INODES) {
+            d_judge_deletion_queue_of_an_entry(tt, entry);
+        }
+    }
+}
+
+// 删除态扫描：回退候选集里每条根（树表指针在根记录偏移 36）与由记录施加出来的那几版（记录新根段打头的树表指针），同 I-5.5 那一份清单
+fn judge_deletion_state_of_the_versions() {
+    for (var candidate = 0u; candidate < c_candidate_count; candidate = candidate + 1u) {
+        let root = checker_root_of(sw(C_CANDIDATES + candidate));
+        d_judge_deletion_state_of_a_version(root.base, 36u);
+    }
+    for (var version = 0u; version < c_version_count; version = version + 1u) {
+        let record = checker_record_at(sw(R_DATA + version));
+        d_judge_deletion_state_of_a_version(record.base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET);
+    }
+}
+
+// I-9.6 墓碑那一半（walk.rs largest_published_tombstone_object_identifier 与 check_pool_image 里接它的那一处）：记账里有 inode 号水位那一行时，
+// 所选根（最新的根）的树表里种类 10 的墓碑树整棵读——类型段 0 的条目指下一层码 2 节点（层级 = 父层级 − 1 且 ≥ 1），类型段 1 的指墓碑容器
+// （码 3、记录类型 1、记录宽 ≥ 40），按已发布谓词滤容器（码 3：写序实例代号与诞生代号），取已发布墓碑记录对象 ID 的最大值，要小于水位。
+// 一条墓碑都没有不判；树表或墓碑树读不全不判
+fn judge_inode_number_watermark_above_every_published_tombstone() {
+    var watermark = vec2<u32>(0u, 0u);
+    if (!c_accounting_row(DELETION_STATISTIC_INODE_WATERMARK, NONE, &watermark)) {
+        return;
+    }
+    let newest_root = checker_root_of(c_newest_index);
+    let tree_table_pointer = parse_node_pointer(newest_root.base, 36u);
+    if (tree_table_pointer.all_zero) {
+        return;
+    }
+    let tt = r_read_index_node(tree_table_pointer);
+    if (!tt.ok || (tt.entry_count > 0u && tt.entry_width < DELETION_TREE_TABLE_ENTRY_BYTES)) {
+        return;
+    }
+    var pending = 0u;
+    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
+        let entry = entry_offset(tt, entry_index);
+        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) != TREE_KIND_TOMBSTONE
+            || parse_node_pointer(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET).all_zero) {
+            continue;
+        }
+        if (pending >= C_R_MAX_PENDING) {
+            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (53u << 8u);
+            return;
+        }
+        let row = C_R_PENDING + pending * C_R_PENDING_WORDS;
+        sws(row, tt.base);
+        sws(row + 1u, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET);
+        pending = pending + 1u;
+    }
+    var has_a_published_tombstone = false;
+    var largest_object_identifier = vec2<u32>(0u, 0u);
+    while (pending > 0u) {
+        pending = pending - 1u;
+        let row = C_R_PENDING + pending * C_R_PENDING_WORDS;
+        let n = r_read_index_node(parse_node_pointer(sw(row), sw(row + 1u)));
+        if (!n.ok || n.level == 0u || n.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
+            return;
+        }
+        for (var e = 0u; e < n.entry_count; e = e + 1u) {
+            let o = entry_offset(n, e);
+            let child_pointer = parse_node_pointer(n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
+            let entry_type = rd16(n.base, o + DELETION_CONTAINER_TREE_ENTRY_TYPE_OFFSET);
+            if (entry_type == DELETION_CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE) {
+                let child = r_read_index_node(child_pointer);
+                if (!child.ok || child.level + 1u != n.level || child.level == 0u) {
+                    return;
+                }
+                if (pending >= C_R_MAX_PENDING) {
+                    undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (54u << 8u);
+                    return;
+                }
+                let child_row = C_R_PENDING + pending * C_R_PENDING_WORDS;
+                sws(child_row, n.base);
+                sws(child_row + 1u, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
+                pending = pending + 1u;
+            } else if (entry_type == PACKED_TYPE_TOMBSTONE) {
+                if (child_pointer.all_zero) {
+                    return;
+                }
+                let container = parse_packed_unit(read_unit_via_locations(child_pointer, DATA_UNIT_BYTES));
+                if (!container.ok || container.record_type != PACKED_TYPE_TOMBSTONE
+                    || container.record_width < DELETION_TOMBSTONE_RECORD_BYTES) {
+                    return;
+                }
+                if (!w_published(container.birth_txg, container.instance, false, vec2<u32>(0u, 0u))) {
+                    continue;
+                }
+                for (var r = 0u; r < container.record_count; r = r + 1u) {
+                    let object_identifier = rd64(container.base, packed_record_offset(container, r) + DELETION_TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET);
+                    if (!has_a_published_tombstone || lt64(largest_object_identifier, object_identifier)) {
+                        largest_object_identifier = object_identifier;
+                    }
+                    has_a_published_tombstone = true;
+                }
+            } else {
+                return;
+            }
+        }
+    }
+    if (has_a_published_tombstone && !lt64(largest_object_identifier, watermark)) {
+        violate(INVARIANT_I_9_6);
+    }
+}
+
 fn scan_stage(fsid_low: vec2<u32>) {
     judge_merged_version_total_order(fsid_low);
     judge_release_generation_and_tree_table_birth();
@@ -2023,6 +2524,8 @@ fn scan_stage(fsid_low: vec2<u32>) {
     judge_allocation_records_disjoint();
     judge_allocation_generations_against_unit_births();
     judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark();
+    judge_deletion_state_of_the_versions();
+    judge_inode_number_watermark_above_every_published_tombstone();
     judge_rollback_floor_raises_against_their_ceilings();
     judge_accounting(fsid_low);
     judge_instance_table_rows_against_the_roots_referencing_them();
diff --git a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
index f168b44..78a60e2 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
@@ -879,14 +879,14 @@ mod tests {
     #[test]
     fn states_in_one_dispatch_follow_the_scratch_memory_up_to_the_lane_limit() {
         let bytes_per_state = u64::try_from(SCRATCH_WORDS * 4).expect("装得进 u64");
-        assert_eq!(states_per_scratch_lane(BINDING_LIMIT), 1847);
+        assert_eq!(states_per_scratch_lane(BINDING_LIMIT), 1797);
         assert_eq!(states_per_scratch_lane(1), 1, "再小的上限也带一个");
-        // 12 GiB 的草稿区：12 × 1024 × 1024 × 1024 ÷ 1 162 176 = 11 086 个（向下取整）
+        // 12 GiB 的草稿区：12 × 1024 × 1024 × 1024 ÷ 1 194 944 = 10 782 个（向下取整）
         assert_eq!(
             states_per_dispatch(BINDING_LIMIT, 12 * 1024),
             usize::try_from(12 * 1024 * 1024 * 1024 / bytes_per_state).expect("装得进 usize")
         );
-        assert_eq!(states_per_dispatch(BINDING_LIMIT, 12 * 1024), 11_086);
+        assert_eq!(states_per_dispatch(BINDING_LIMIT, 12 * 1024), 10_782);
         assert_eq!(
             states_per_dispatch(BINDING_LIMIT, 0),
             1,
@@ -894,7 +894,7 @@ mod tests {
         );
         assert_eq!(
             states_per_dispatch(BINDING_LIMIT, 1024 * 1024),
-            1847 * MAXIMUM_SCRATCH_LANES,
+            1797 * MAXIMUM_SCRATCH_LANES,
             "显存再多也只到条数上限"
         );
     }
diff --git a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
index 3012b48..d00933a 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
@@ -3,7 +3,7 @@
 //! 七类 oracle、记录核对器两条判据，判法与 CPU 判器（`crate::crash::judge_crash_image`）逐条相同；GPU 与 CPU 逐状态比是它的判别力自证。
 //! 内核分五份源码：`crash_judge_common.wgsl`（各内核共用的解析与查表）、`crash_judge_recovery.wgsl`（恢复、oracle、记录核对器）、
 //! `crash_judge_checker_common.wgsl`（池级 checker 两个内核共用的读法与交接段）、`crash_judge_checker.wgsl`（走读内核：第一、二批不变量，
-//! 连同里程碑三二·1 那一批里的十五条、I-9.27 与 I-3.5）、`crash_judge_checker_scan.wgsl`（扫描内核：第三批、I-3.12、I-5.5 与 I-9.28）；判不了的状态（表越过内核容量）置最高位、交 CPU 判。
+//! 连同里程碑三二·1 那一批里的十五条、I-9.27 与 I-3.5）、`crash_judge_checker_scan.wgsl`（扫描内核：第三批、I-3.12、I-5.5、I-9.28，与里程碑三·3 删除线的删除态扫描、I-9.6 墓碑那一半）；判不了的状态（表越过内核容量）置最高位、交 CPU 判。
 //!
 //! 一个状态的结论 [`VERDICT_WORDS`] 个 u32：第 0 字是红位（位 0..6 看 journal 那一遍 oracle 的七类、位 8..14 不看那一遍、
 //! 位 16 根在而记录都不在、位 17 恢复自称的状态少单元、位 31 判不了）；第 1 字判不了的原因（判得了时低 16 位是看 journal 那一遍
@@ -58,17 +58,20 @@ const _: () = assert!(
     "违例位读成一个 u128"
 );
 
-/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 69 条——三批 49 条，加里程碑三二·1 那一批 16 条
+/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 74 条——三批 49 条，加里程碑三二·1 那一批 16 条
 /// （判决 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行），下标 65 的 I-3.5（引用区间的精确性，
 /// 走读内核在回退候选集的版本都走完之后逐对判），再加 C673（三 oracle 合谋测试的补 oracle 改法没做）⑤ ⑥ 那两条
 /// （下标 66 是 I-9.27，走读内核挨着 I-9.16 按表判；下标 67 是 I-5.5，扫描内核每一版重读树表与 extent 树判），
 /// 再加下标 68 的 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同，扫描内核挨着 I-9.14 读根表里每一条根的树表判；
-/// 部分没判与不适用一样不置违例位，结论里只有违例位），对拍逐位比、不屏蔽。
+/// 部分没判与不适用一样不置违例位，结论里只有违例位），再加里程碑三·3 删除线那五条：下标 69 的 I-9.5、70 的 I-3.4、71 的 I-4.7、
+/// 72 的 I-4.6 由扫描内核的删除态扫描在回退候选集的每一版上判（判哪几版、经头 ID 配对、读不全不置位，照 `singlefs_checker::walk` 的
+/// `judge_deletion_state_of_the_versions`），73 的 I-4.4 第一版池级 checker 恒报不适用、内核不置位；I-9.6 墓碑那一半（水位 > 所选根墓碑树里
+/// 已发布墓碑的最大对象 ID）也在扫描内核里判、并进下标 41 那一位。对拍逐位比、不屏蔽。
 /// 用户 2026-10-04 定「改回要求全覆盖」：没有「内核还没判」那一张表。
 pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[
     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
     26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
-    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68,
+    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
 ];
 
 // 覆盖表就是池级 checker 的每一条，按下标升序：第 k 项是 k。新立一条不变量而内核没跟上时这里编译不过，不悄悄让 GPU 那一位恒 0。
@@ -112,8 +115,8 @@ pub fn verdict_layout_wgsl_constants() -> String {
     )
 }
 /// 每个状态的草稿区几个 u32（checker 共用那一半里 `SCRATCH_END`：共用的 `SCRATCH_WORDS_REQUIRED`、走读与扫描的集合、两个内核的交接段、
-/// 交接段之后走读内核自己用的表：两张，加 I-3.5 的出现规则读的四张）。
-pub const SCRATCH_WORDS: usize = 290_544;
+/// 交接段之后走读内核自己用的表：两张，加 I-3.5 的出现规则读的四张；再之后是扫描内核删除态扫描一版一版重填的两张：inode 记录与每个号的数据单元数）。
+pub const SCRATCH_WORDS: usize = 298_736;
 /// 内核的工作组大小（三个内核都是 `@workgroup_size(64)`）：派 n 个状态要 ⌈n ÷ 64⌉ 个工作组。
 pub const WORKGROUP_SIZE: usize = 64;
 /// 内核的四个绑定（`crash_judge_common.wgsl` 开头）：派活参数、打包的表、结论、草稿区。
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 026f9ca..154ac87 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -8,6 +8,9 @@
 //! 恢复施加一条记录取新根段的那几段、施加出来的那一版从记录走实例表与分配记录树、I-8.10 后三段，各有一条同样形态的流；
 //! I-9.28 比到候选集之外的根与树表读不出的根、I-7.8 扫描方向的认领规则、I-7.9 两支只取锚在镜像最新根上的一圈窗口里的根，也各有流；
 //! 树表 11 条与种类 9–11（谱系树的容器记录宽、谱系树根的树 ID、谱系树与 logged_ops 两条的发号次序）各有一条，树表条数不超过扫描内核的上限取卡之前就判。
+//! 里程碑三·3 删除线（I-9.5、I-3.4、I-4.7、I-4.6、I-9.6 墓碑那一半）：删除中间态（unlink 意图与第一批）一条流钉四条判过而成立，
+//! 每条各一条只改坏它管的那个字段的流钉 CPU 红、GPU 逐位跟上；走过 nlink = 0 那几版的流只比池级 checker 那一半（恢复那一半今天对不上，见
+//! `compare_the_checker_on_a_deletion_flow`）。
 //! 要本机一张空闲显存够的 NVIDIA 独显：本机一张 NVIDIA 独显适配器都没有时用卡的用例打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、
 //! 返回成功；有卡而用不上（一张都没起来，或 PCI 上有卡而 Vulkan 枚举不出）判红（`gpu_unit_checks::gpu_test_start`）。要 `verdict-store,gpu` 特性；
 //! 展开上限 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`（默认 8）、显存额度 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（没设不定额度）、
@@ -17,11 +20,19 @@
 #[path = "../../singlefs-harness/tests/common/mod.rs"]
 mod common;
 mod common_crash_points;
+#[path = "../../singlefs-harness/tests/common_deletion_in_progress/mod.rs"]
+mod common_deletion_in_progress;
+#[path = "../../singlefs-harness/tests/common_pointer_head_corruption/mod.rs"]
+mod common_pointer_head_corruption;
 
 use std::time::Instant;
 
-use common::{build_pool, file_content, geometry, parameters};
+use common::{build_pool, file_content, geometry, parameters, BuiltPool};
 use common_crash_points::{mount_writable_step, overwrite_step, record_pool_build, roll_back_step};
+use common_deletion_in_progress::{
+    content_of_three_data_units, pool_with_a_three_unit_file, pool_with_new_inodes,
+    publish_the_intent, publish_the_next_step_in_batches_of_one, FIRST_FILE,
+};
 use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
 use singlefs_checker_tier::crash::{
     judge_layer0_state_range, layer0_plan_state_count, Layer0OracleViolationKind,
@@ -1847,6 +1858,8 @@ struct Comparison {
     /// 看 journal 那一遍落到的根或两遍的结局类别与 CPU 不同的状态数（算进不一致）。
     recovery_mismatches: u64,
     mismatches: Vec<String>,
+    /// 只看池级 checker 的不一致：违例位与 CPU 不同、或判不了的状态（前 12 个带说明，之后记空串，同 `mismatches`）。
+    checker_mismatches: Vec<String>,
 }
 
 fn compare_on_every_state(card: &GpuCard, flow: &RecordedFlow, limit: usize) -> Comparison {
@@ -1968,6 +1981,7 @@ fn compare_on_every_state(card: &GpuCard, flow: &RecordedFlow, limit: usize) ->
         cpu_checker_bits_of_each_state: Vec::with_capacity(cpu.len()),
         recovery_mismatches: 0,
         mismatches: Vec::new(),
+        checker_mismatches: Vec::new(),
     };
     for (
         ordinal,
@@ -2005,6 +2019,18 @@ fn compare_on_every_state(card: &GpuCard, flow: &RecordedFlow, limit: usize) ->
         if gpu_recovery != *cpu_recovery {
             comparison.recovery_mismatches += 1;
         }
+        if undecidable || gpu_checker_bits != *cpu_checker_bits {
+            if comparison.checker_mismatches.len() < 12 {
+                comparison.checker_mismatches.push(format!(
+                    "状态 {ordinal}：CPU checker={:?}；GPU checker={:?} 判不了的原因 {:#x}",
+                    invariant_names(*cpu_checker_bits),
+                    invariant_names(gpu_checker_bits),
+                    gpu_words[POOL_CHECKER_UNDECIDABLE_WORD]
+                ));
+            } else {
+                comparison.checker_mismatches.push(String::new());
+            }
+        }
         if undecidable
             || gpu_bits != *cpu_bits
             || gpu_checker_bits != *cpu_checker_bits
@@ -2031,7 +2057,7 @@ fn compare_on_every_state(card: &GpuCard, flow: &RecordedFlow, limit: usize) ->
         eprintln!("CRASH_JUDGE_GPU_MISMATCH flow={} {shown}", flow.name);
     }
     eprintln!(
-        "CRASH_JUDGE_GPU flow={} states={} cpu_red={} gpu_red={} cpu_checker_red={} gpu_checker_red={} recovery_mismatches={} mismatches={} gpu_elapsed={:?} cpu_elapsed={:?}",
+        "CRASH_JUDGE_GPU flow={} states={} cpu_red={} gpu_red={} cpu_checker_red={} gpu_checker_red={} recovery_mismatches={} mismatches={} checker_mismatches={} gpu_elapsed={:?} cpu_elapsed={:?}",
         flow.name,
         comparison.states,
         comparison.cpu_red,
@@ -2040,6 +2066,7 @@ fn compare_on_every_state(card: &GpuCard, flow: &RecordedFlow, limit: usize) ->
         comparison.gpu_checker_red,
         comparison.recovery_mismatches,
         comparison.mismatches.len(),
+        comparison.checker_mismatches.len(),
         gpu_elapsed,
         cpu_elapsed
     );
@@ -2339,7 +2366,7 @@ fn verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several(
 /// 在扫描内核读回违例位之后种第 128 位（下标 127）：
 /// 两份种过的内核判出的每个状态，违例位等于没种的那一份判出的再并上种的位（第 65 位每个状态都有，第 128 位只在扫描内核判完的状态上有），
 /// 其余的字逐字相同。违例位的高 64 位在走读内核写出、扫描内核读回又写出、Rust 读出这三处任一处被截掉，这条红。
-/// 池级 checker 今天 69 条（下标 0..=68，内核每一条都判），种的是第 70 位（下标 69）。
+/// 池级 checker 今天 74 条（下标 0..=73，内核每一条都判；I-4.4 第一版报不适用、那一位恒 0），种的是第 75 位（下标 74）。
 #[test]
 fn a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels() {
     const FIRST_BIT_OF_THE_HIGH_HALF: usize = 64;
@@ -4022,3 +4049,482 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_and_logged_operations_entr
         mismatch_listing(&comparison)
     );
 }
+
+// ───────────── 里程碑三·3 删除线：I-9.5、I-3.4、I-4.7、I-4.6 与 I-9.6 墓碑那一半（GPU 扫描内核的删除态扫描与 CPU 池级 checker 逐状态逐位对拍）─────────────
+
+/// 码 2 / 码 3 单元头里的声明长度（偏移 8，2 字节）。
+const UNIT_DECLARED_LENGTH_OFFSET: usize = 8;
+/// 码 3 单元头里的记录数（偏移 69，2 字节）与记录区起点（含 29 字节预留位的头 136）。
+const PACKED_UNIT_RECORD_COUNT_OFFSET: usize = 69;
+const PACKED_UNIT_RECORDS_START: usize = 136;
+/// inode 记录 140 字节：号 0、nlink 36（4 字节）、size 40、blocks 48（D8（核心索引结构） 已定项 6 字段表）；blocks = ⌈size ÷ 512⌉（I-9.15）。
+const INODE_RECORD_BYTES: usize = 140;
+const INODE_RECORD_NLINK_OFFSET: usize = 36;
+const INODE_RECORD_SIZE_OFFSET: usize = 40;
+const INODE_RECORD_BLOCKS_OFFSET: usize = 48;
+const INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES: u64 = 512;
+/// 打包记录类型：1 墓碑、2 inode。
+const PACKED_RECORD_TYPE_TOMBSTONE: u16 = 1;
+const PACKED_RECORD_TYPE_INODE: u16 = 2;
+/// 码 2 节点头（key 宽 k）：最小 key 在 52、最大 key 在 52 + k、条目数在 82 + 2k、条目区从 115 + 2k 起（D18（块里携带什么信息） 已定项 18）。
+const INDEX_NODE_SMALLEST_KEY_OFFSET: usize = 52;
+/// `deleted_inodes` 根：key 宽 8、条目宽 8（D8（核心索引结构） 已定项 16）。
+const DELETED_INODES_KEY_BYTES: usize = 8;
+/// 记账树叶：key 宽 22、条目 34（统计量 2、树 8、设备 4、代 8、值 8、seq 4），设备段在 10、值在 22。
+const ACCOUNTING_KEY_BYTES: usize = 22;
+const ACCOUNTING_ROW_BYTES: usize = 34;
+const ACCOUNTING_ROW_DEVICE_OFFSET: usize = 10;
+const ACCOUNTING_ROW_VALUE_OFFSET: usize = 22;
+/// 待删除但意图未完成占用（D5（快照 / 空间记账机制） 已定项 4 第 4 项）与不带设备维的设备段。
+const STATISTIC_PENDING_DELETION_OCCUPANCY: u16 = 4;
+const STATISTIC_NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;
+/// I-9.5 正方向那一形塞进队列的号：inode 树里没有它的记录、extent 树里也没有它的单元。
+const INODE_NUMBER_QUEUED_WITHOUT_A_RECORD: u64 = 7;
+/// I-9.6 墓碑那一半那一形把墓碑的对象 ID 改成的值：远大于这条流的 inode 号水位（新池新建文件之后是 2）。
+const TOMBSTONE_OBJECT_IDENTIFIER_PAST_THE_WATERMARK: u64 = 1 << 40;
+
+fn index_node_entry_count_offset(key_width: usize) -> usize {
+    82 + 2 * key_width
+}
+fn index_node_entries_start(key_width: usize) -> usize {
+    115 + 2 * key_width
+}
+fn read_u16_at(bytes: &[u8], offset: usize) -> u16 {
+    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("2"))
+}
+fn read_u32_at(bytes: &[u8], offset: usize) -> u32 {
+    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4"))
+}
+fn read_u64_at(bytes: &[u8], offset: usize) -> u64 {
+    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8"))
+}
+
+/// 删除线那几条流共用的收尾：`pool` 从 mkfs 之后录下的写切段，被判的是最后一次根槽写。
+fn recorded_flow_of_the_pool(
+    pool: &BuiltPool,
+    name: &'static str,
+    versions: Vec<PublishedVersion>,
+) -> RecordedFlow {
+    let operations = pool.retained_operations();
+    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
+        &operations[pool.mkfs_operation_count..],
+        &geometry(),
+    );
+    let judged_root_index = writes
+        .iter()
+        .rposition(|write| write.kind == StepKind::RootRecordFua)
+        .expect("发布有根槽写");
+    RecordedFlow {
+        name,
+        base: pool.memory_pool_after_mkfs(),
+        writes,
+        segments,
+        judged_root_index,
+        versions,
+    }
+}
+
+/// 删除中间态的流（`common_deletion_in_progress` 的搭建）：新池新建文件（txg 3）、第一个文件顺序写成三个数据单元（txg 4）、unlink 意图（txg 5：
+/// inode 1 的记录 nlink 写成 0、号进 `deleted_inodes`、墓碑树开放容器追加一条）、按批大小 1 删第一批（txg 6：剩两个单元、size 两个净荷）。
+/// 恢复从意图那一版起答「没有文件」，所以带文件的版本只有 txg 3、4。
+fn record_new_pool_through_the_first_deletion_batch(tag: &str, name: &'static str) -> RecordedFlow {
+    let mut pool = pool_with_a_three_unit_file(tag);
+    publish_the_intent(&mut pool);
+    publish_the_next_step_in_batches_of_one(&mut pool);
+    recorded_flow_of_the_pool(
+        &pool,
+        name,
+        vec![
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(3),
+                content: file_content(),
+            },
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(4),
+                content: content_of_three_data_units(),
+            },
+        ],
+    )
+}
+
+/// 删除中间态的流上改坏最后一个 `select` 命中的单元（两份副本）的一处、重封到根。
+fn record_deletion_with_a_resealed_field(
+    tag: &str,
+    name: &'static str,
+    select: &dyn Fn(&[u8]) -> bool,
+    mutate: &dyn Fn(&mut [u8]),
+) -> RecordedFlow {
+    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
+    let targets = last_unit_copies(&flow, name, select, false);
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, mutate);
+    flow
+}
+
+fn is_a_deleted_inodes_root(bytes: &[u8]) -> bool {
+    unit_class(bytes) == 2
+        && usize::from(index_node_key_width(bytes)) == DELETED_INODES_KEY_BYTES
+        && usize::from(index_node_entry_width(bytes)) == DELETED_INODES_KEY_BYTES
+}
+fn is_an_accounting_leaf(bytes: &[u8]) -> bool {
+    unit_class(bytes) == 2
+        && usize::from(index_node_key_width(bytes)) == ACCOUNTING_KEY_BYTES
+        && usize::from(index_node_entry_width(bytes)) == ACCOUNTING_ROW_BYTES
+}
+fn is_an_inode_leaf_container(bytes: &[u8]) -> bool {
+    unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_INODE
+}
+fn is_a_tombstone_container(bytes: &[u8]) -> bool {
+    unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_TOMBSTONE
+}
+
+/// inode 叶容器里号 `inode` 那条记录的起点。
+fn offset_of_the_inode_record(bytes: &[u8], inode: u64) -> usize {
+    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
+    (0..record_count)
+        .map(|index| PACKED_UNIT_RECORDS_START + index * INODE_RECORD_BYTES)
+        .find(|offset| read_u64_at(bytes, *offset) == inode)
+        .unwrap_or_else(|| panic!("这片 inode 叶容器里有 inode {inode} 的记录"))
+}
+
+/// I-9.5 正方向那一形：`deleted_inodes` 根 {1} 改成 {1, 7}（条目数加一、声明长度加 8、头里最大 key 改成 7），7 号在 inode 树里没有记录。
+fn put_an_inode_number_without_a_record_into_the_deletion_queue_root(bytes: &mut [u8]) {
+    let count_offset = index_node_entry_count_offset(DELETED_INODES_KEY_BYTES);
+    let entry_count = usize::from(read_u16_at(bytes, count_offset));
+    let declared_length = read_u16_at(bytes, UNIT_DECLARED_LENGTH_OFFSET);
+    assert_eq!(
+        usize::from(declared_length),
+        entry_count * DELETED_INODES_KEY_BYTES,
+        "队列根的声明长度是条目数 × 8"
+    );
+    let new_entry =
+        index_node_entries_start(DELETED_INODES_KEY_BYTES) + entry_count * DELETED_INODES_KEY_BYTES;
+    bytes[new_entry..new_entry + DELETED_INODES_KEY_BYTES]
+        .copy_from_slice(&INODE_NUMBER_QUEUED_WITHOUT_A_RECORD.to_le_bytes());
+    bytes[count_offset..count_offset + 2].copy_from_slice(
+        &u16::try_from(entry_count + 1)
+            .expect("装得进 u16")
+            .to_le_bytes(),
+    );
+    bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
+        &(declared_length + u16::try_from(DELETED_INODES_KEY_BYTES).expect("8")).to_le_bytes(),
+    );
+    let largest_key = INDEX_NODE_SMALLEST_KEY_OFFSET + DELETED_INODES_KEY_BYTES;
+    bytes[largest_key..largest_key + DELETED_INODES_KEY_BYTES]
+        .copy_from_slice(&INODE_NUMBER_QUEUED_WITHOUT_A_RECORD.to_le_bytes());
+}
+
+/// I-3.4 那一形：记账叶里待删除但意图未完成占用（标签 4、设备段 0xFFFF_FFFF）那一行的值 0 → 1。
+fn set_the_pending_deletion_occupancy_row_to_one(bytes: &mut [u8]) {
+    let entry_count = usize::from(read_u16_at(
+        bytes,
+        index_node_entry_count_offset(ACCOUNTING_KEY_BYTES),
+    ));
+    let row = (0..entry_count)
+        .map(|index| index_node_entries_start(ACCOUNTING_KEY_BYTES) + index * ACCOUNTING_ROW_BYTES)
+        .find(|row| {
+            read_u16_at(bytes, *row) == STATISTIC_PENDING_DELETION_OCCUPANCY
+                && read_u32_at(bytes, row + ACCOUNTING_ROW_DEVICE_OFFSET)
+                    == STATISTIC_NO_DEVICE_DIMENSION
+        })
+        .expect("记账叶里有待删除但意图未完成占用那一行");
+    assert_eq!(
+        read_u64_at(bytes, row + ACCOUNTING_ROW_VALUE_OFFSET),
+        0,
+        "那一行恒 0"
+    );
+    bytes[row + ACCOUNTING_ROW_VALUE_OFFSET..row + ACCOUNTING_ROW_VALUE_OFFSET + 8]
+        .copy_from_slice(&1u64.to_le_bytes());
+}
+
+/// I-4.6 那一形：inode 叶容器里第一个文件（删除中、nlink = 0）的 size 多加一个净荷容量，blocks 跟着按 ⌈size ÷ 512⌉ 改（I-9.15 照旧成立），
+/// extent 树里的单元数不动。
+fn raise_the_size_of_the_first_file_by_one_data_unit_payload(bytes: &mut [u8]) {
+    let record = offset_of_the_inode_record(bytes, FIRST_FILE.0);
+    let payload_capacity =
+        u64::try_from(singlefs_core::unit::data_unit_payload_capacity()).expect("装得进 u64");
+    let size = read_u64_at(bytes, record + INODE_RECORD_SIZE_OFFSET) + payload_capacity;
+    bytes[record + INODE_RECORD_SIZE_OFFSET..record + INODE_RECORD_SIZE_OFFSET + 8]
+        .copy_from_slice(&size.to_le_bytes());
+    bytes[record + INODE_RECORD_BLOCKS_OFFSET..record + INODE_RECORD_BLOCKS_OFFSET + 8]
+        .copy_from_slice(
+            &size
+                .div_ceil(INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES)
+                .to_le_bytes(),
+        );
+}
+
+/// I-9.6 墓碑那一半那一形：墓碑容器第一条墓碑的对象 ID 改成远大于 inode 号水位的值。
+fn raise_the_first_tombstone_object_identifier_past_the_watermark(bytes: &mut [u8]) {
+    assert!(
+        read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET) >= 1,
+        "墓碑容器里至少一条墓碑"
+    );
+    bytes[PACKED_UNIT_RECORDS_START..PACKED_UNIT_RECORDS_START + 8]
+        .copy_from_slice(&TOMBSTONE_OBJECT_IDENTIFIER_PAST_THE_WATERMARK.to_le_bytes());
+}
+
+/// I-3.4 缺行那一形：记账叶里待删除但意图未完成占用那一行整行拿掉（后面的行往前挪、条目数减一、声明长度减 34；这一行不在首末，头里的 key 区间不变）。
+fn drop_the_pending_deletion_occupancy_row(bytes: &mut [u8]) {
+    let count_offset = index_node_entry_count_offset(ACCOUNTING_KEY_BYTES);
+    let entry_count = usize::from(read_u16_at(bytes, count_offset));
+    let entries_start = index_node_entries_start(ACCOUNTING_KEY_BYTES);
+    let row_index = (0..entry_count)
+        .find(|index| {
+            let row = entries_start + index * ACCOUNTING_ROW_BYTES;
+            read_u16_at(bytes, row) == STATISTIC_PENDING_DELETION_OCCUPANCY
+                && read_u32_at(bytes, row + ACCOUNTING_ROW_DEVICE_OFFSET)
+                    == STATISTIC_NO_DEVICE_DIMENSION
+        })
+        .expect("记账叶里有待删除但意图未完成占用那一行");
+    assert!(
+        row_index > 0 && row_index + 1 < entry_count,
+        "那一行不在首末：拿掉它头里的 key 区间不变"
+    );
+    let row = entries_start + row_index * ACCOUNTING_ROW_BYTES;
+    let rows_end = entries_start + entry_count * ACCOUNTING_ROW_BYTES;
+    bytes.copy_within(row + ACCOUNTING_ROW_BYTES..rows_end, row);
+    bytes[rows_end - ACCOUNTING_ROW_BYTES..rows_end].fill(0);
+    bytes[count_offset..count_offset + 2].copy_from_slice(
+        &u16::try_from(entry_count - 1)
+            .expect("装得进 u16")
+            .to_le_bytes(),
+    );
+    let declared_length = read_u16_at(bytes, UNIT_DECLARED_LENGTH_OFFSET);
+    bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
+        &(declared_length - u16::try_from(ACCOUNTING_ROW_BYTES).expect("34")).to_le_bytes(),
+    );
+}
+
+/// 新池新建文件之后建一个空 inode（txg 4，号 2，size 0、没有数据单元，与第一个文件同在最左那片叶里）的流：
+/// 那片叶（最后一个 inode 叶容器的两份副本）照 `mutate` 改一处、重封到根。
+fn record_new_pool_with_a_new_inode_and_its_inode_leaf_resealed(
+    tag: &str,
+    name: &'static str,
+    mutate: &dyn Fn(&mut [u8]),
+) -> RecordedFlow {
+    let pool = pool_with_new_inodes(tag, 1);
+    let mut flow = recorded_flow_of_the_pool(
+        &pool,
+        name,
+        vec![
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(3),
+                content: file_content(),
+            },
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(4),
+                content: file_content(),
+            },
+        ],
+    );
+    let targets = last_unit_copies(&flow, name, &is_an_inode_leaf_container, false);
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, mutate);
+    flow
+}
+
+/// I-4.7 那一形（判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 F1 的孤儿单元，在 checker 侧直接改字节造）：
+/// 空 inode 那片叶里 1 号的记录拿掉（后面的记录往前挪、记录数减一、声明长度减 140），extent 树不动：1 号在 extent 树里还有数据单元、
+/// inode 树里没有它的记录、号也不在队列里。
+fn drop_the_record_of_the_first_file(bytes: &mut [u8]) {
+    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
+    assert!(record_count >= 2, "第一个文件与空 inode 同在这片叶里");
+    let dropped = offset_of_the_inode_record(bytes, FIRST_FILE.0);
+    let records_end = PACKED_UNIT_RECORDS_START + record_count * INODE_RECORD_BYTES;
+    bytes.copy_within(dropped + INODE_RECORD_BYTES..records_end, dropped);
+    bytes[records_end - INODE_RECORD_BYTES..records_end].fill(0);
+    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2].copy_from_slice(
+        &u16::try_from(record_count - 1)
+            .expect("装得进 u16")
+            .to_le_bytes(),
+    );
+    let declared_length = read_u16_at(bytes, UNIT_DECLARED_LENGTH_OFFSET);
+    bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
+        &(declared_length - u16::try_from(INODE_RECORD_BYTES).expect("140")).to_le_bytes(),
+    );
+}
+
+/// I-9.5 反方向那一形（C708、判决里说的僵尸记录）：空 inode 2 的记录 nlink 1 → 0，号不进队列（它没有数据单元，I-4.7、I-4.6 照旧成立）。
+fn set_the_nlink_of_the_new_inode_to_zero(bytes: &mut [u8]) {
+    let record = offset_of_the_inode_record(bytes, FIRST_FILE.0 + 1);
+    assert_eq!(
+        read_u32_at(bytes, record + INODE_RECORD_NLINK_OFFSET),
+        1,
+        "空 inode 的 nlink 是 1"
+    );
+    bytes[record + INODE_RECORD_NLINK_OFFSET..record + INODE_RECORD_NLINK_OFFSET + 4]
+        .copy_from_slice(&0u32.to_le_bytes());
+}
+
+/// 删除线那几条流的对拍：池级 checker 的违例位逐状态逐位相同、一个状态都不许判不了；CPU 要在这条流上红 `must_red`（`None` 不要求）。
+/// 恢复那一半这里不比：CPU 恢复从意图那一版起把 nlink = 0 的第一个文件答成没有文件（`singlefs_core::recovery`，D8（核心索引结构） 已定项 16
+/// 第 4 条），GPU 恢复内核（`crash_judge_common.wgsl` 找第一个文件那一段）不看 nlink、照读文件，两边在删除中间态上的恢复结局对不上
+/// （第 18 批 P2 交回报告里写明、由主 agent 定怎么修），这几条流只钉池级 checker 那一半；恢复的不一致数照样打在 `CRASH_JUDGE_GPU` 那一行。
+fn compare_the_checker_on_a_deletion_flow(
+    test_name: &str,
+    flow: &RecordedFlow,
+    must_red: Option<&'static str>,
+) -> Option<Comparison> {
+    let card = one_usable_card_or_report_not_run(test_name)?;
+    let comparison = compare_on_every_state(&card, flow, expand_up_to());
+    if let Some(invariant) = must_red {
+        assert!(
+            comparison.cpu_checker_red_invariants.contains(invariant),
+            "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
+            flow.name,
+            comparison.cpu_checker_red_invariants
+        );
+    }
+    assert!(
+        comparison.checker_mismatches.is_empty(),
+        "流 {}：{} 个状态池级 checker 的违例位 GPU 与 CPU 不一致或判不了（前几个）：\n{}",
+        flow.name,
+        comparison.checker_mismatches.len(),
+        comparison
+            .checker_mismatches
+            .iter()
+            .filter(|line| !line.is_empty())
+            .cloned()
+            .collect::<Vec<_>>()
+            .join("\n")
+    );
+    Some(comparison)
+}
+
+/// 删除中间态（意图之后、第一批之后）的每个状态上 GPU 与 CPU 池级 checker 逐位相同；删除线四条在这条流上 CPU 判过而成立、一个状态都不红
+/// （I-9.5 队列 {1} 与 nlink = 0 的号集合相同、I-3.4 标签 4 那一行在且为 0、I-4.7 号 1 在队列里、I-4.6 单元数等于 ⌈size ÷ 净荷容量⌉）。
+#[test]
+fn the_gpu_checker_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress() {
+    let flow = record_new_pool_through_the_first_deletion_batch(
+        "crash-judge-gpu-deletion-in-progress",
+        "deletion-through-the-first-batch",
+    );
+    let Some(comparison) = compare_the_checker_on_a_deletion_flow(
+        "the_gpu_checker_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress",
+        &flow,
+        None,
+    ) else {
+        return;
+    };
+    for invariant in ["I-9.5", "I-3.4", "I-4.7", "I-4.6"] {
+        assert!(
+            comparison.cpu_checker_held_invariants.contains(invariant),
+            "删除中间态的流上 CPU 池级 checker 判过 {invariant} 而成立（成立过的：{:?}）",
+            comparison.cpu_checker_held_invariants
+        );
+        assert!(
+            !comparison.cpu_checker_red_invariants.contains(invariant),
+            "删除中间态的流上 CPU 池级 checker 不红 {invariant}（红过的：{:?}）",
+            comparison.cpu_checker_red_invariants
+        );
+    }
+}
+
+#[test]
+fn the_gpu_checker_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record() {
+    let flow = record_deletion_with_a_resealed_field(
+        "crash-judge-gpu-i95",
+        "deletion-queue-names-an-inode-without-a-record",
+        &is_a_deleted_inodes_root,
+        &put_an_inode_number_without_a_record_into_the_deletion_queue_root,
+    );
+    compare_the_checker_on_a_deletion_flow(
+        "the_gpu_checker_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record",
+        &flow,
+        Some("I-9.5"),
+    );
+}
+
+#[test]
+fn the_gpu_checker_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero() {
+    let flow = record_deletion_with_a_resealed_field(
+        "crash-judge-gpu-i34",
+        "deletion-pending-occupancy-row-one",
+        &is_an_accounting_leaf,
+        &set_the_pending_deletion_occupancy_row_to_one,
+    );
+    compare_the_checker_on_a_deletion_flow(
+        "the_gpu_checker_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero",
+        &flow,
+        Some("I-3.4"),
+    );
+}
+
+#[test]
+fn the_gpu_checker_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_missing() {
+    let flow = record_deletion_with_a_resealed_field(
+        "crash-judge-gpu-i34-missing-row",
+        "deletion-pending-occupancy-row-missing",
+        &is_an_accounting_leaf,
+        &drop_the_pending_deletion_occupancy_row,
+    );
+    compare_the_checker_on_a_deletion_flow(
+        "the_gpu_checker_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_missing",
+        &flow,
+        Some("I-3.4"),
+    );
+}
+
+/// 这条流上没有 nlink = 0 的记录（第一个文件的记录整条拿掉了），恢复两边都答「没有文件」，所以连恢复一起逐状态比。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_without_a_record_keeps_its_data_unit_outside_the_queue(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_without_a_record_keeps_its_data_unit_outside_the_queue",
+        &record_new_pool_with_a_new_inode_and_its_inode_leaf_resealed(
+            "crash-judge-gpu-i47",
+            "new-pool-new-inode-first-file-record-dropped",
+            &drop_the_record_of_the_first_file,
+        ),
+        "I-4.7",
+    );
+}
+
+/// 这条流上 nlink = 0 的是空 inode 2，第一个文件照常可读，恢复两边都读出文件，所以连恢复一起逐状态比。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_record_of_nlink_zero_is_not_in_the_deletion_queue() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_record_of_nlink_zero_is_not_in_the_deletion_queue",
+        &record_new_pool_with_a_new_inode_and_its_inode_leaf_resealed(
+            "crash-judge-gpu-i95-zombie",
+            "new-pool-new-inode-nlink-zero-outside-the-queue",
+            &set_the_nlink_of_the_new_inode_to_zero,
+        ),
+        "I-9.5",
+    );
+}
+
+#[test]
+fn the_gpu_checker_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units(
+) {
+    let flow = record_deletion_with_a_resealed_field(
+        "crash-judge-gpu-i46",
+        "deletion-size-past-the-data-units",
+        &is_an_inode_leaf_container,
+        &raise_the_size_of_the_first_file_by_one_data_unit_payload,
+    );
+    compare_the_checker_on_a_deletion_flow(
+        "the_gpu_checker_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units",
+        &flow,
+        Some("I-4.6"),
+    );
+}
+
+#[test]
+fn the_gpu_checker_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark(
+) {
+    let flow = record_deletion_with_a_resealed_field(
+        "crash-judge-gpu-i96-tombstone",
+        "deletion-tombstone-past-the-watermark",
+        &is_a_tombstone_container,
+        &raise_the_first_tombstone_object_identifier_past_the_watermark,
+    );
+    compare_the_checker_on_a_deletion_flow(
+        "the_gpu_checker_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark",
+        &flow,
+        Some("I-9.6"),
+    );
+}
````

### p2：mutations-append.tsv

````
三·3 GPU 判器：I-9.5 正方向不判（队列里没有记录、或记录 nlink ≠ 0 的号放过）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	            if (row == NONE || sw(row + 2u) != 0u) {	            if (false && (row == NONE || sw(row + 2u) != 0u)) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_checker_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record	the_gpu_checker_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record
三·3 GPU 判器：I-9.5 反方向不判（nlink = 0 而号不在队列里的僵尸记录放过）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (sw(row + 2u) == 0u && !d_queue_contains(queue, queue_present, vec2<u32>(sw(row), sw(row + 1u)))) {	        if (false && sw(row + 2u) == 0u && !d_queue_contains(queue, queue_present, vec2<u32>(sw(row), sw(row + 1u)))) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_record_of_nlink_zero_is_not_in_the_deletion_queue	the_gpu_judge_agrees_with_the_cpu_when_a_record_of_nlink_zero_is_not_in_the_deletion_queue
三·3 GPU 判器：I-3.4 只判缺行、不判标签 4 那一行的值	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    if (!has_the_pending_deletion_row || some_pending_deletion_value_is_not_zero) {	    if (!has_the_pending_deletion_row) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_checker_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero	the_gpu_checker_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero
三·3 GPU 判器：I-3.4 只判值、不判缺行	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    if (!has_the_pending_deletion_row || some_pending_deletion_value_is_not_zero) {	    if (some_pending_deletion_value_is_not_zero) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_checker_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_missing	the_gpu_checker_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_missing
三·3 GPU 判器：I-4.7 不判（有数据单元、没有 nlink ≥ 1 记录、不在队列的号放过）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (sw(count_row + 2u) > 0u && !has_a_record_of_nlink_one_or_more && !d_queue_contains(queue, queue_present, inode)) {	        if (false && sw(count_row + 2u) > 0u && !has_a_record_of_nlink_one_or_more && !d_queue_contains(queue, queue_present, inode)) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_inode_without_a_record_keeps_its_data_unit_outside_the_queue	the_gpu_judge_agrees_with_the_cpu_when_an_inode_without_a_record_keeps_its_data_unit_outside_the_queue
三·3 GPU 判器：I-4.6 不判（nlink = 0 的记录单元数与 size 对不上放过）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (!matches_the_size) {	        if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_checker_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units	the_gpu_checker_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units
三·3 GPU 判器：I-9.6 墓碑那一半不判（已发布墓碑的对象 ID 不小于水位放过）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    if (has_a_published_tombstone && !lt64(largest_object_identifier, watermark)) {	    if (false && has_a_published_tombstone && !lt64(largest_object_identifier, watermark)) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_checker_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark	the_gpu_checker_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark
三·3 GPU 判器：extent 树下段叶的数据单元不按号数（删除中 inode 的单元数读成 0）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	                    r_count_data_unit_of_inode(inode, lower_data_pointer);	                    let lower_data_pointer_not_counted = lower_data_pointer;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_checker_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress	the_gpu_checker_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress
三·3 GPU 判器：extent 树上段叶内联的数据单元不按号数（孤儿单元数成 0）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	                        r_count_data_unit_of_inode(extent_key_inode(n.base, o), inline_data_pointer);	                        let inline_data_pointer_not_counted = inline_data_pointer;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_inode_without_a_record_keeps_its_data_unit_outside_the_queue	the_gpu_judge_agrees_with_the_cpu_when_an_inode_without_a_record_keeps_its_data_unit_outside_the_queue
三·3 GPU 判器：扫描内核没接删除态扫描	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    judge_deletion_state_of_the_versions();	    let deletion_state_not_judged = 0u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_checker_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero	the_gpu_checker_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero
三·3 GPU 判器：I-9.5 的下标错位成 I-3.4 那一位（70）	crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl	const INVARIANT_I_9_5: u32 = 69u;	const INVARIANT_I_9_5: u32 = 70u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_checker_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record	the_gpu_checker_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record
````

### p2：mutations-replacements.tsv

````
第 5 批 GPU 判器：派活给每个状态的草稿区停在交接段末端、没跟上内核新加的两张表	crates/singlefs-checker-tier/src/crash_judge_gpu.rs	pub const SCRATCH_WORDS: usize = 298_736;	pub const SCRATCH_WORDS: usize = 247_792;	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_gpu::tests::scratch_words_given_to_each_state_reach_the_scratch_end_of_the_kernels	scratch_words_given_to_each_state_reach_the_scratch_end_of_the_kernels
````

