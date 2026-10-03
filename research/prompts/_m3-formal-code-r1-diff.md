# m3-formal-code-r1 被判的改动（2026-10-02）

底：第 7 批合入之前、第 10 批合入之前各存的原样与三份实现员补丁；E165、E170 两个装置取代码轮 m3-verus-land-code-r3 的冻结副本 `/tmp/claude-1000/m3-verus-land-code-r3/tree/` 当底。

## 实现员补丁 m3-formal-7-1（`/tmp/claude-1000/m3-formal-7-1/patch/crates.patch`）

```diff
diff --git a/crates/singlefs-harness/tests/common/mod.rs b/crates/singlefs-harness/tests/common/mod.rs
index a44a989..4d0baf3 100644
--- a/crates/singlefs-harness/tests/common/mod.rs
+++ b/crates/singlefs-harness/tests/common/mod.rs
@@ -4,7 +4,9 @@
 use std::path::PathBuf;
 use std::sync::atomic::{AtomicU64, Ordering};
 
-use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+};
 use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
 use singlefs_core::block_device::{FileBackedBlockDevice, PhysicalBlockSizeInBytes};
 use singlefs_core::journal::{back_chain_of, record_offset, JournalRecord};
@@ -19,8 +21,13 @@ use singlefs_core::transaction::{
     acquire_instance, publish_first_file, publish_overwrite, publish_sequential_write, warm_up,
     FirstFile, PoolWriter, PublishError, TransactionOutput, WarmUpOutput,
 };
-use singlefs_core::unit::unit_filesystem_identifier;
-use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES};
+use singlefs_core::unit::{
+    unit_filesystem_identifier, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
+};
+use singlefs_format::{
+    DATA_UNIT_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT,
+    NODE_BYTES, SLOT_BYTES,
+};
 use singlefs_harness::memory_pool::MemoryPool;
 use singlefs_harness::segments::FixedGeometry;
 use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};
@@ -484,6 +491,107 @@ fn journal_record_bytes_in_the_default_ring(
     .expect("环里那一格读得到")
 }
 
+/// 块层的一次读：哪块盘、从哪个字节起、多长。读盘次序的用例（只读挂载、可写挂载）逐次记这一项。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct BlockLayerRead {
+    pub device: DeviceIdentity,
+    pub offset: DeviceOffsetInBytes,
+    pub length_in_bytes: usize,
+}
+
+/// 反向链重读从 `device_identities()` 的第一块盘试起，健康镜像上第一份就对得上。
+pub const FIRST_DEVICE_TRIED_BY_THE_BACK_CHAIN_REREAD: DeviceIdentity = DeviceIdentity(0);
+
+/// 一个点名单元的整单元宽：数据单元与码 3 打包记录单元 32768，码 2 索引节点 16384（D23（journal 的角色与格式） 已定项 14 第四条按类读）。
+pub fn named_unit_bytes(unit_class: u8) -> usize {
+    let unit_bytes = match unit_class {
+        UNIT_CLASS_DATA | UNIT_CLASS_PACKED => DATA_UNIT_BYTES,
+        UNIT_CLASS_INDEX_NODE => NODE_BYTES,
+        unknown_class => {
+            panic!("发布路径只点名数据单元、索引节点与打包记录单元，读到类 {unknown_class}")
+        }
+    };
+    usize::try_from(unit_bytes).expect("单元宽装得进 usize")
+}
+
+/// 候选 `record` 的反向链那一判在块层的读：在 0 号盘上重读它的上一条记录（`ring_bytes` 长的环里的落点）。
+pub fn block_read_rereading_the_record_before(
+    record: &JournalRecord,
+    ring_bytes: u64,
+) -> BlockLayerRead {
+    BlockLayerRead {
+        device: FIRST_DEVICE_TRIED_BY_THE_BACK_CHAIN_REREAD,
+        offset: record_offset(record.counter - 1, ring_bytes),
+        length_in_bytes: usize::try_from(JOURNAL_RECORD_BYTES).expect("4096"),
+    }
+}
+
+/// 候选 `record` 的点名验证在块层的读：它点名的每个单元的两份，按点名次序。
+pub fn block_reads_of_the_named_units_of(record: &JournalRecord) -> Vec<BlockLayerRead> {
+    record
+        .named
+        .iter()
+        .flat_map(|named| {
+            named.locations.iter().map(|location| BlockLayerRead {
+                device: location.device,
+                offset: location.slot.to_device_offset(),
+                length_in_bytes: named_unit_bytes(named.unit_class),
+            })
+        })
+        .collect()
+}
+
+pub fn is_inside_the_journal_ring(offset: DeviceOffsetInBytes, ring_bytes: u64) -> bool {
+    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
+    offset.0 >= ring_start && offset.0 < ring_start + ring_bytes
+}
+
+/// 把计数器 `counters` 那几条记录从默认环的落点搬到 `ring_bytes` 长的环的落点（两盘各一份；先全部读出再写，搬过去的会盖掉更早的记录）。
+/// 小环过不了 mkfs 与读者择系统配置，在飞上限那一形的用例在默认环建的池上这样摆记录，再拿环长改小的那份系统配置扫环。
+pub fn image_with_the_records_relocated_onto_a_ring_of(
+    image: &MemoryPool,
+    counters: &[u64],
+    ring_bytes: u64,
+) -> MemoryPool {
+    let mut relocated = image.clone();
+    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+        let records: Vec<(u64, Vec<u8>)> = counters
+            .iter()
+            .map(|counter| {
+                (
+                    *counter,
+                    journal_record_bytes_in_the_default_ring(image, device, *counter),
+                )
+            })
+            .collect();
+        for (counter, bytes) in records {
+            relocated
+                .devices
+                .get_mut(&device)
+                .expect("有这块盘")
+                .write(record_offset(counter, ring_bytes), &bytes);
+        }
+    }
+    relocated
+}
+
+/// 把 [`system_configuration_slots_of_every_device`] 存下的系统配置槽字节写回镜像（盘在盘表里的位置 `index` 就是 `DeviceIdentity(index)`）：
+/// 造「根槽没落、它之后的系统配置轮换也没落」的崩溃状态（D16（发布语义） 已定项 7：轮换在根槽 FUA 之后）。
+pub fn write_back_system_configuration_slots(
+    image: &mut MemoryPool,
+    saved: &[(usize, DeviceOffsetInBytes, Vec<u8>)],
+) {
+    for (index, offset, bytes) in saved {
+        image
+            .devices
+            .get_mut(&DeviceIdentity(
+                u32::try_from(*index).expect("盘在盘表里的位置装得进 u32"),
+            ))
+            .expect("有这块盘")
+            .write(*offset, bytes);
+    }
+}
+
 pub fn format_pool(tag: &str) -> FormattedPool {
     let (paths, devices, stream, genesis, mkfs_operation_count) = formatted_recorded_images(tag);
     FormattedPool {
diff --git a/crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs b/crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs
index 929b2f8..8a6dc3f 100644
--- a/crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs
+++ b/crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs
@@ -6,7 +6,10 @@
 //! 这里在块层逐次记下读，每条用例钉一种形状下扫完环之后的读：判候选的读的绝对次数与次序，之后环里再没有读。
 //! 钉住的只有这几种形状（代码轮第三轮判决 `research/prompts/m3-verus-land-code-r3-main-verification.md` J2、用户定案 U2）：
 //! 锚点读得出、两次单条发布都施加；锚点那条记录撕了；锚点读不出、链首序号不是 1；一次发布两条记录；
-//! 候选在末条的提交标记上被断。可写挂载的读阶段走缓存（`ReadStageCache`）、`recover` 不验点名单元那一臂，读几次盘不在这里钉。
+//! 候选在末条的提交标记上被断；候选的反向链断了；到了在飞上限（3 槽的环，6-1 交回第 2 条的 H08、H09 两形）。
+//! 在飞上限那一格的环过不了只读挂载择系统配置，照只读挂载同一串直接调 `choose_root`、`scan_journal`、`replay_journal`，只记重放那一截的读。
+//! 可写挂载的读阶段走缓存（`ReadStageCache`）那一半在 `writable_mount_reads_each_candidate_in_order_through_the_read_stage_cache.rs`；
+//! `recover` 不验点名单元那一臂读几次盘不在这里钉。
 //!
 //! 单条发布的历史造法同 `journal_replay_applies_no_more_records_than_the_in_flight_limit.rs` 的阳性对照：A（txg 3、jsn 3）的根落了，
 //! B（txg 4、jsn 4）、C（txg 5、jsn 5）两次单条发布的记录与单元都在、根槽都坏了 ⇒ 所选根是 A 的。
@@ -18,10 +21,13 @@ use std::cell::RefCell;
 use std::collections::BTreeMap;
 
 use common::{
-    build_pool, overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
+    block_read_rereading_the_record_before, block_reads_of_the_named_units_of, build_pool,
+    image_with_the_records_relocated_onto_a_ring_of, is_inside_the_journal_ring,
+    overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
     rewrite_journal_record_and_rechain_the_records_after_it,
     sequential_write_in_process_as_instance_one_and_advance_the_pool,
-    tear_the_root_slot_written_for_txg, CONTENT_BYTES_OF_PUBLISH_B, CONTENT_BYTES_OF_PUBLISH_C,
+    tear_the_root_slot_written_for_txg, BlockLayerRead, CONTENT_BYTES_OF_PUBLISH_B,
+    CONTENT_BYTES_OF_PUBLISH_C,
 };
 use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
@@ -30,14 +36,13 @@ use singlefs_core::journal::{
     record_offset, JournalRecord, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish,
 };
 use singlefs_core::mounted_read::{mount_read_only, open_pool_for_read, MountedReadOnly};
-use singlefs_core::recovery::{choose_system_configuration, scan_journal, PoolReader};
-use singlefs_core::root_record::RootRecord;
-use singlefs_core::unit::{
-    data_unit_payload_capacity, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
-};
-use singlefs_format::{
-    DATA_UNIT_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, NODE_BYTES, SLOT_BYTES,
+use singlefs_core::recovery::{
+    choose_root, choose_system_configuration, replay_journal, scan_journal, PoolReader,
 };
+use singlefs_core::root_record::RootRecord;
+use singlefs_core::system_configuration::journal_in_flight_record_limit;
+use singlefs_core::unit::data_unit_payload_capacity;
+use singlefs_format::JOURNAL_RECORD_BYTES;
 use singlefs_harness::memory_pool::MemoryPool;
 
 /// B、C 两版文件的填充字节：两版长度（在 `common`）与内容各不相同。
@@ -45,8 +50,6 @@ const CONTENT_FILL_BYTE_OF_PUBLISH_B: u8 = 0xB0;
 const CONTENT_FILL_BYTE_OF_PUBLISH_C: u8 = 0xC0;
 /// 两条记录那段历史里 B 的填充字节。
 const CONTENT_FILL_BYTE_OF_THE_TWO_RECORD_PUBLISH: u8 = 0xD0;
-/// 反向链重读从 `device_identities()` 的第一块盘试起，健康镜像上第一份就对得上。
-const FIRST_DEVICE_TRIED_BY_THE_BACK_CHAIN_REREAD: DeviceIdentity = DeviceIdentity(0);
 /// 一个候选的上一条记录在环里读得出时，反向链在第一块盘上重读它这么多次。
 const REREADS_OF_THE_PREVIOUS_RECORD_PER_CANDIDATE: usize = 1;
 /// 点名验证把每个点名单元的两份落点各读一次（`NamedUnit::locations` 是两项）。
@@ -62,14 +65,10 @@ const NAMED_UNITS_OF_THE_FIRST_RECORD_OF_THE_TWO_RECORD_PUBLISH: usize = 1;
 const NAMED_UNITS_OF_THE_LAST_RECORD_OF_THE_TWO_RECORD_PUBLISH: usize = 13;
 /// 撕一条 journal 记录时翻的字节：在记录头之后、头部校验和罩着的整条 4096 之内（翻了自证就不过）。
 const BYTE_INSIDE_A_JOURNAL_RECORD_AFTER_ITS_HEADER: u64 = 300;
-
-/// 块层的一次读。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
-struct BlockLayerRead {
-    device: DeviceIdentity,
-    offset: DeviceOffsetInBytes,
-    length_in_bytes: usize,
-}
+/// 改坏一条记录的反向链时异或进去的位：链值变了、jsn 照样接得上（重封之后自证照过）。
+const BIT_FLIPPED_IN_A_BACK_CHAIN: u32 = 1;
+/// 3 个记录槽的环：在飞上限 = 3 ÷ F（3）= 1（同 `journal_replay_applies_no_more_records_than_the_in_flight_limit.rs`）。
+const THREE_RECORD_RING_BYTES: u64 = 3 * JOURNAL_RECORD_BYTES;
 
 /// 挂载那一路对读者做的事，按发生的次序。
 #[derive(Clone, Debug, PartialEq, Eq)]
@@ -134,41 +133,16 @@ impl PoolReader for ReadLoggingPoolReader<'_> {
     }
 }
 
-/// 一个点名单元的整单元宽：数据单元与码 3 打包记录单元 32768，码 2 索引节点 16384（D23（journal 的角色与格式） 已定项 14 第四条按类读）。
-fn named_unit_bytes(unit_class: u8) -> usize {
-    let unit_bytes = match unit_class {
-        UNIT_CLASS_DATA | UNIT_CLASS_PACKED => DATA_UNIT_BYTES,
-        UNIT_CLASS_INDEX_NODE => NODE_BYTES,
-        unknown_class => {
-            panic!("发布路径只点名数据单元、索引节点与打包记录单元，读到类 {unknown_class}")
-        }
-    };
-    usize::try_from(unit_bytes).expect("单元宽装得进 usize")
-}
-
 /// 候选 `record` 的反向链那一判：在 0 号盘上重读它的上一条记录一次。
 fn reread_of_the_record_before(record: &JournalRecord, ring_bytes: u64) -> ReaderEvent {
-    ReaderEvent::Read(BlockLayerRead {
-        device: FIRST_DEVICE_TRIED_BY_THE_BACK_CHAIN_REREAD,
-        offset: record_offset(record.counter - 1, ring_bytes),
-        length_in_bytes: usize::try_from(JOURNAL_RECORD_BYTES).expect("4096"),
-    })
+    ReaderEvent::Read(block_read_rereading_the_record_before(record, ring_bytes))
 }
 
 /// 候选 `record` 的点名验证：它点名的每个单元的两份，按点名次序。
 fn named_unit_reads_of(record: &JournalRecord) -> Vec<ReaderEvent> {
-    record
-        .named
-        .iter()
-        .flat_map(|named| {
-            named.locations.iter().map(|location| {
-                ReaderEvent::Read(BlockLayerRead {
-                    device: location.device,
-                    offset: location.slot.to_device_offset(),
-                    length_in_bytes: named_unit_bytes(named.unit_class),
-                })
-            })
-        })
+    block_reads_of_the_named_units_of(record)
+        .into_iter()
+        .map(ReaderEvent::Read)
         .collect()
 }
 
@@ -179,11 +153,6 @@ fn reads_judging_the_candidate(record: &JournalRecord, ring_bytes: u64) -> Vec<R
     reads
 }
 
-fn is_inside_the_journal_ring(offset: DeviceOffsetInBytes, ring_bytes: u64) -> bool {
-    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
-    offset.0 >= ring_start && offset.0 < ring_start + ring_bytes
-}
-
 /// 一段事件里落在环里的读的次数；要环提示的每一次也算一次（扫完环之后不该再有）。
 fn ring_reads_and_hints_among(events: &[ReaderEvent], ring_bytes: u64) -> usize {
     events
@@ -706,3 +675,152 @@ fn a_last_record_without_its_commit_marker_is_cut_after_rereading_its_previous_r
         "扫环按提示读 jsn 1–5 五个槽、两块盘各一遍"
     );
 }
+
+/// 单条发布的 B、C，C（jsn 5）的反向链改一位、重封（jsn 接得上、链值与 jsn 4 的头算出来的不等，D23 已定项 7、I-8.6）：
+/// jsn 4 照常重读 jsn 3、读它点名的单元，施加；jsn 5 重读 jsn 4 一次就在反向链上断，它点名的单元一个都不读。挂载到 B。
+#[test]
+fn a_candidate_whose_back_chain_is_broken_is_cut_after_rereading_its_previous_record_and_before_reading_its_named_units(
+) {
+    let (mut image, first_version) = single_record_publishes_b_and_c_with_their_roots_torn(
+        "mounted-recovery-reads-back-chain-broken",
+    );
+    rewrite_journal_record_and_rechain_the_records_after_it(&mut image, 5, |record| {
+        record.back_chain ^= BIT_FLIPPED_IN_A_BACK_CHAIN;
+    });
+    let (ring_bytes, records) = journal_records_of(&image);
+    let candidate_b = records
+        .get(&(InstanceGeneration(1), 4))
+        .expect("jsn 4 在环里");
+    let candidate_c = records
+        .get(&(InstanceGeneration(1), 5))
+        .expect("jsn 5 改过反向链、重封，照样自证得过");
+
+    let (mounted, reads) = mount_read_only_logging_every_read(&image, ring_bytes);
+    assert_eq!(
+        (
+            (
+                mounted.chosen_root.instance,
+                mounted.chosen_root.checkpoint_txg
+            ),
+            (
+                mounted.effective_root.instance,
+                mounted.effective_root.checkpoint_txg
+            ),
+            mounted.journal.above_water,
+            mounted.journal.prefix_applied,
+            mounted.journal.verification_passed
+        ),
+        (
+            first_version,
+            (InstanceGeneration(1), CheckpointTxg(4)),
+            2,
+            1,
+            1
+        ),
+        "jsn 4 过了点名验证、施加；jsn 5 断在反向链上，挂载到 B"
+    );
+
+    let mut expected_reads_judging_the_candidates =
+        reads_judging_the_candidate(candidate_b, ring_bytes);
+    expected_reads_judging_the_candidates
+        .push(reread_of_the_record_before(candidate_c, ring_bytes));
+    assert_eq!(
+        expected_reads_judging_the_candidates.len(),
+        REREADS_OF_THE_PREVIOUS_RECORD_PER_CANDIDATE
+            + NAMED_UNITS_OF_A_SINGLE_RECORD_OVERWRITE * READS_PER_NAMED_UNIT
+            + REREADS_OF_THE_PREVIOUS_RECORD_PER_CANDIDATE,
+        "判两个候选的读：jsn 4 一次反向链重读加它点名单元的两份，jsn 5 只有一次反向链重读"
+    );
+    let mut expected_events_after_the_scan = expected_reads_judging_the_candidates;
+    expected_events_after_the_scan.extend(reads_of_opening_the_pool_at(
+        &image,
+        &mounted.effective_root,
+    ));
+    assert_eq!(
+        reads.events_after_the_scan, expected_events_after_the_scan,
+        "扫完环之后：jsn 4 重读 jsn 3、读它点名的单元，jsn 5 重读 jsn 4 一次就断、不读点名单元，然后只剩沿 B 打开挂载态的读"
+    );
+    assert_eq!(
+        reads.scan_reads_inside_the_ring,
+        RING_SLOTS_WRITTEN_UP_TO_JSN_FIVE * DEVICES_THE_JOURNAL_RING_IS_SCANNED_ON,
+        "扫环按提示读 jsn 1–5 五个槽、两块盘各一遍"
+    );
+}
+
+/// 单条发布的 B、C 搬到 3 槽的环上（在飞上限 3 ÷ F = 1，D23 已定项 18）：jsn 4 重读 jsn 3、读它点名的单元，施加；
+/// 取了 1 条就到上限，jsn 5 在任何读之前停下——不重读 jsn 4、不读它点名的单元。重放那一截块层的读就只有 jsn 4 的这几次。
+#[test]
+fn at_the_in_flight_limit_the_next_candidate_is_cut_before_rereading_its_previous_record() {
+    let (image, first_version) = single_record_publishes_b_and_c_with_their_roots_torn(
+        "mounted-recovery-reads-in-flight-limit",
+    );
+    assert_eq!(
+        journal_in_flight_record_limit(THREE_RECORD_RING_BYTES),
+        1,
+        "3 槽 ÷ F（3）= 1"
+    );
+    let relocated = image_with_the_records_relocated_onto_a_ring_of(
+        &image,
+        &[3, 4, 5],
+        THREE_RECORD_RING_BYTES,
+    );
+    let system_configuration =
+        choose_system_configuration(&relocated).expect("系统配置择得出（它记着的是默认环）");
+    let mut three_record_ring_configuration = system_configuration;
+    three_record_ring_configuration
+        .immutable
+        .sizes
+        .journal_ring_bytes = THREE_RECORD_RING_BYTES;
+    let chosen_root = choose_root(&relocated, &system_configuration).expect("择得出根");
+    let records = scan_journal(&relocated, &three_record_ring_configuration);
+    assert_eq!(
+        (
+            (chosen_root.instance, chosen_root.checkpoint_txg),
+            records
+                .keys()
+                .map(|(_, counter)| *counter)
+                .collect::<Vec<_>>()
+        ),
+        (first_version, vec![3, 4, 5]),
+        "所选根是 A 的；3 槽的环里扫到 jsn 3、4、5"
+    );
+    let candidate_b = records
+        .get(&(InstanceGeneration(1), 4))
+        .expect("jsn 4 在环里");
+
+    let logging = ReadLoggingPoolReader {
+        inner: &relocated,
+        events: RefCell::new(Vec::new()),
+    };
+    let (report, rebuilt) = replay_journal(
+        &logging,
+        &chosen_root,
+        THREE_RECORD_RING_BYTES,
+        &records,
+        true,
+    )
+    .expect("所选根那次发布只有 jsn 3 带末条标志：锚点认得出");
+    assert_eq!(
+        (
+            report.above_water,
+            report.prefix_applied,
+            report.verification_passed,
+            (rebuilt.instance, rebuilt.checkpoint_txg)
+        ),
+        (2, 1, 1, (InstanceGeneration(1), CheckpointTxg(4))),
+        "水位之上两条，上限 1：只施加 jsn 4、验过它的点名单元，重建到 B"
+    );
+    let expected_reads_of_the_replay =
+        reads_judging_the_candidate(candidate_b, THREE_RECORD_RING_BYTES);
+    assert_eq!(
+        expected_reads_of_the_replay.len(),
+        REREADS_OF_THE_PREVIOUS_RECORD_PER_CANDIDATE
+            + NAMED_UNITS_OF_A_SINGLE_RECORD_OVERWRITE * READS_PER_NAMED_UNIT,
+        "重放那一截的读：jsn 4 一次反向链重读加它点名单元的两份，jsn 5 一次都没有"
+    );
+    assert_eq!(
+        logging.events.into_inner(),
+        expected_reads_of_the_replay,
+        "重放那一截：jsn 4 在 3 槽环的落点上重读 jsn 3 一次、读它点名的单元，到了上限 jsn 5 不重读 jsn 4、不读点名单元"
+    );
+}
diff --git a/crates/singlefs-harness/tests/journal_scan_keeps_the_first_copy_of_a_duplicated_record_key.rs b/crates/singlefs-harness/tests/journal_scan_keeps_the_first_copy_of_a_duplicated_record_key.rs
new file mode 100644
index 0000000..b96688b
--- /dev/null
+++ b/crates/singlefs-harness/tests/journal_scan_keeps_the_first_copy_of_a_duplicated_record_key.rs
@@ -0,0 +1,142 @@
+//! 全环扫描建记录表时，同一个 (实例代号, 计数器) 在盘上有两份内容不同、各自自证得过的记录：表里留先扫到的那一份
+//! （`recovery::scan_journal` 按 `device_identities()` 的次序逐盘扫，先到先得）。
+//! 取前缀的核要的「键唯一」在真路径上就靠这一判（形式化题面第三轮判决 `research/prompts/m3-formal-r3-main-verification.md`
+//! 第二节 S3、第四节欠账第 10 条）；改成后到覆盖，表里留的就换成后一块盘上的那一份。
+//!
+//! 造法：A（txg 3、jsn 3）的根落了，B（txg 4、jsn 4）一次单条发布只剩记录与单元、根槽坏了。1 号盘上 jsn 4 那一份的提交标记改成 0、重封
+//! （0 号盘那一份原样）：两份都自证得过、内容不同。留 0 号盘那一份 ⇒ B 施加；留 1 号盘那一份 ⇒ B 的末条不带提交标记，整次不施加。
+
+mod common;
+
+use common::{
+    build_pool, overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
+    tear_the_root_slot_written_for_txg, CONTENT_BYTES_OF_PUBLISH_B,
+};
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use singlefs_core::journal::{record_offset, JournalRecord};
+use singlefs_core::mounted_read::mount_read_only;
+use singlefs_core::recovery::{choose_system_configuration, scan_journal, PoolReader};
+use singlefs_core::unit::unit_filesystem_identifier;
+use singlefs_format::JOURNAL_RECORD_BYTES;
+use singlefs_harness::memory_pool::MemoryPool;
+
+/// B 那一版文件的填充字节。
+const CONTENT_FILL_BYTE_OF_PUBLISH_B: u8 = 0xB0;
+/// 扫环先扫的盘（`MemoryPool::device_identities` 按盘号升序）与后扫的盘。
+const DEVICE_SCANNED_FIRST: DeviceIdentity = DeviceIdentity(0);
+const DEVICE_SCANNED_SECOND: DeviceIdentity = DeviceIdentity(1);
+
+/// `device` 上 jsn `counter` 那一格解出来的记录。
+fn record_on_device(
+    image: &MemoryPool,
+    device: DeviceIdentity,
+    ring_bytes: u64,
+    counter: u64,
+) -> Option<JournalRecord> {
+    let bytes = PoolReader::read(
+        image,
+        device,
+        record_offset(counter, ring_bytes),
+        usize::try_from(JOURNAL_RECORD_BYTES).expect("4096"),
+    )
+    .expect("环里那一格读得到");
+    JournalRecord::parse(
+        &bytes,
+        unit_filesystem_identifier(&parameters().filesystem_identifier),
+    )
+}
+
+/// 只改 `device` 上 jsn `counter` 那一份：解出来、改一处、`JournalRecord::to_bytes` 重封载荷与头部校验和、写回；另一块盘那一份不动。
+fn rewrite_the_copy_on_one_device(
+    image: &mut MemoryPool,
+    device: DeviceIdentity,
+    ring_bytes: u64,
+    counter: u64,
+    change: impl Fn(&mut JournalRecord),
+) {
+    let mut record =
+        record_on_device(image, device, ring_bytes, counter).expect("改之前这一份自证过");
+    change(&mut record);
+    image
+        .devices
+        .get_mut(&device)
+        .expect("有这块盘")
+        .write(record_offset(counter, ring_bytes), &record.to_bytes());
+}
+
+/// 同一个键两份内容不同的记录：表里是 0 号盘（先扫）那一份；只读挂载照它施加 B。
+#[test]
+fn of_two_differing_copies_of_one_record_key_the_scan_keeps_the_copy_on_the_device_scanned_first() {
+    let mut pool = build_pool("journal-scan-first-copy-of-a-duplicated-key");
+    let first_version = (pool.output.root.instance, pool.output.root.checkpoint_txg);
+    let second = overwrite_in_process_as_instance_one_and_advance_the_pool(
+        &mut pool,
+        &vec![CONTENT_FILL_BYTE_OF_PUBLISH_B; CONTENT_BYTES_OF_PUBLISH_B],
+    );
+    assert_eq!(
+        (
+            first_version,
+            (second.root.checkpoint_txg, second.record.counter)
+        ),
+        (
+            (InstanceGeneration(1), CheckpointTxg(3)),
+            (CheckpointTxg(4), 4)
+        ),
+        "A 的根在 txg 3、jsn 3；B 一条记录 jsn 4"
+    );
+    let mut image = pool.memory_pool();
+    tear_the_root_slot_written_for_txg(&mut image, 4);
+    let system_configuration = choose_system_configuration(&image).expect("系统配置择得出");
+    let ring_bytes = system_configuration.immutable.sizes.journal_ring_bytes;
+    rewrite_the_copy_on_one_device(&mut image, DEVICE_SCANNED_SECOND, ring_bytes, 4, |record| {
+        record.is_commit = false;
+    });
+    let copy_scanned_first = record_on_device(&image, DEVICE_SCANNED_FIRST, ring_bytes, 4)
+        .expect("0 号盘那一份自证得过");
+    let copy_scanned_second = record_on_device(&image, DEVICE_SCANNED_SECOND, ring_bytes, 4)
+        .expect("1 号盘那一份重封过，自证得过");
+    assert_eq!(
+        (
+            copy_scanned_first.is_commit,
+            copy_scanned_second.is_commit,
+            copy_scanned_first == copy_scanned_second
+        ),
+        (true, false, false),
+        "同一个键 (1, 4) 两份内容不同：0 号盘的带提交标记，1 号盘的不带"
+    );
+    assert_eq!(
+        image.device_identities(),
+        [DEVICE_SCANNED_FIRST, DEVICE_SCANNED_SECOND],
+        "扫环按盘号升序逐盘扫"
+    );
+
+    let records = scan_journal(&image, &system_configuration);
+    assert_eq!(
+        records.get(&(InstanceGeneration(1), 4)),
+        Some(&copy_scanned_first),
+        "表里 (1, 4) 是先扫到的 0 号盘那一份，不被 1 号盘那一份盖掉"
+    );
+
+    let mounted = mount_read_only(&image).expect("只读挂载");
+    assert_eq!(
+        (
+            (
+                mounted.chosen_root.instance,
+                mounted.chosen_root.checkpoint_txg
+            ),
+            (
+                mounted.effective_root.instance,
+                mounted.effective_root.checkpoint_txg
+            ),
+            mounted.journal.above_water,
+            mounted.journal.prefix_applied
+        ),
+        (
+            first_version,
+            (InstanceGeneration(1), CheckpointTxg(4)),
+            1,
+            1
+        ),
+        "所选根是 A 的；照 0 号盘那一份（带提交标记）施加 B，挂载到 B"
+    );
+}
diff --git a/crates/singlefs-harness/tests/rebuilt_root_keeps_the_chosen_roots_fields_except_the_unmount_marker.rs b/crates/singlefs-harness/tests/rebuilt_root_keeps_the_chosen_roots_fields_except_the_unmount_marker.rs
new file mode 100644
index 0000000..a556417
--- /dev/null
+++ b/crates/singlefs-harness/tests/rebuilt_root_keeps_the_chosen_roots_fields_except_the_unmount_marker.rs
@@ -0,0 +1,134 @@
+//! 由 journal 记录重建的根（`recovery::replay_journal` 拼新根那一截）：新根段（D23（journal 的角色与格式） 已定项 15）那几项取被施加的末条记录，
+//! 其余字段等于所选根，卸载记号除外——它取「不是卸载那一串写下的」（D22（单元原子性怎么合成） 已定项 7「其余根写 0」）。
+//! 形式化题面第三轮判决 `research/prompts/m3-formal-r3-main-verification.md` 第二节 S1 后半（用户 2026-10-01 认）、第四节欠账第 1 条。
+//!
+//! 判决里池上原型那一形：正常卸载那一串（`mount::unmount`）的第二次空发布记录落了、根槽没落（这里把那一槽改坏）——所选根是第一次空发布写的、
+//! 带卸载记号；第二次那条记录施加出来的根，卸载记号若照抄所选根，就成了「卸载那一串写下的」，可它没有一条根槽写过。
+
+mod common;
+
+use common::{
+    build_pool, overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
+    tear_the_root_slot_written_for_txg, CONTENT_BYTES_OF_PUBLISH_B,
+};
+use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::mount::{unmount, ShadowLedger, Unmounted};
+use singlefs_core::mounted_read::mount_read_only;
+use singlefs_core::root_record::UnmountMarker;
+use singlefs_core::transaction::PoolVersion;
+
+/// 新池新建文件（txg 3）之后覆盖写三版（txg 4、5、6）的填充字节：现行 txg 6 除以 3 余 0，卸载那一串推两次空发布（txg 7、8）。
+const FILL_BYTES_OF_THE_THREE_OVERWRITES: [u8; 3] = [0xB0, 0xC0, 0xD0];
+
+/// 卸载那一串第二次空发布（txg 8）的根槽没落、记录落了：只读挂载择到第一次（txg 7）那条带卸载记号的根，施加 txg 8 那条记录。
+/// 重建出来的根：新根段那几项是 txg 8 那条记录的；文件系统标识、实例表指针、分配记录树根等于所选根；卸载记号是「不是卸载那一串写下的」。
+#[test]
+fn a_root_rebuilt_from_the_second_unmount_publish_record_keeps_the_chosen_roots_other_fields_and_is_not_marked_by_the_unmount_sequence(
+) {
+    let mut pool = build_pool("rebuilt-root-unmount-marker");
+    for fill_byte in FILL_BYTES_OF_THE_THREE_OVERWRITES {
+        overwrite_in_process_as_instance_one_and_advance_the_pool(
+            &mut pool,
+            &vec![fill_byte; CONTENT_BYTES_OF_PUBLISH_B],
+        );
+    }
+    assert_eq!(
+        pool.output.root.checkpoint_txg,
+        CheckpointTxg(6),
+        "新池新建文件 txg 3，覆盖写三版到 txg 6"
+    );
+    let mut current = PoolVersion::WithFile(pool.output.clone());
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let unmounted = unmount(
+        &parameters(),
+        devices,
+        &mut pool.allocator,
+        &mut current,
+        ShadowLedger::On,
+    )
+    .expect("正常卸载");
+    let Unmounted::FloorRaisedToTheCurrentVersion(raised) = unmounted else {
+        panic!("带文件的一版上卸载报成「树表 0 条、一个字节都不写」");
+    };
+    let [first_unmount_publish, second_unmount_publish] =
+        <[_; 2]>::try_from(raised.publishes).expect("现行 txg 6 除以 3 余 0：卸载那一串推两次");
+    assert_eq!(
+        (
+            (
+                first_unmount_publish.root.checkpoint_txg,
+                first_unmount_publish.root.unmount_marker
+            ),
+            (
+                second_unmount_publish.root.checkpoint_txg,
+                second_unmount_publish.root.unmount_marker
+            ),
+            (
+                second_unmount_publish.record.instance,
+                second_unmount_publish.record.checkpoint_txg
+            )
+        ),
+        (
+            (CheckpointTxg(7), UnmountMarker::WrittenByTheUnmountSequence),
+            (CheckpointTxg(8), UnmountMarker::WrittenByTheUnmountSequence),
+            (InstanceGeneration(1), CheckpointTxg(8))
+        ),
+        "卸载那一串两次空发布 txg 7、8 写的根都带卸载记号；txg 8 那条记录在实例 1"
+    );
+
+    let mut image = pool.memory_pool();
+    tear_the_root_slot_written_for_txg(&mut image, 8);
+    let mounted = mount_read_only(&image).expect("只读挂载");
+    assert_eq!(
+        mounted.chosen_root, first_unmount_publish.root,
+        "txg 8 的根槽没落：所选根是卸载那一串第一次空发布写的那条，带卸载记号"
+    );
+    assert_eq!(
+        (mounted.journal.above_water, mounted.journal.prefix_applied),
+        (1, 1),
+        "水位之上只有 txg 8 那条记录，施加"
+    );
+
+    let chosen = mounted.chosen_root;
+    let rebuilt = mounted.effective_root;
+    let record = &second_unmount_publish.record;
+    assert_eq!(
+        (
+            rebuilt.instance,
+            rebuilt.checkpoint_txg,
+            rebuilt.tree_table,
+            rebuilt.tree_identifier_watermark,
+            rebuilt.rollback_floor,
+            rebuilt.mapping_root
+        ),
+        (
+            record.instance,
+            record.checkpoint_txg,
+            record.new_tree_table,
+            record.new_tree_identifier_watermark,
+            record.new_rollback_floor,
+            record.new_mapping_root
+        ),
+        "新根段那几项（D23 已定项 15）取 txg 8 那条记录的"
+    );
+    assert_eq!(
+        (
+            rebuilt.filesystem_identifier,
+            rebuilt.instance_table,
+            rebuilt.allocation_record_tree_root
+        ),
+        (
+            chosen.filesystem_identifier,
+            chosen.instance_table,
+            chosen.allocation_record_tree_root
+        ),
+        "新根段之外的字段等于所选根"
+    );
+    assert_eq!(
+        (chosen.unmount_marker, rebuilt.unmount_marker),
+        (
+            UnmountMarker::WrittenByTheUnmountSequence,
+            UnmountMarker::NotWrittenByTheUnmountSequence
+        ),
+        "卸载记号除外：所选根带着，由记录重建的根取「不是卸载那一串写下的」——没有一条根槽写过它"
+    );
+}
diff --git a/crates/singlefs-harness/tests/writable_mount_reads_each_candidate_in_order_through_the_read_stage_cache.rs b/crates/singlefs-harness/tests/writable_mount_reads_each_candidate_in_order_through_the_read_stage_cache.rs
new file mode 100644
index 0000000..c9d5b01
--- /dev/null
+++ b/crates/singlefs-harness/tests/writable_mount_reads_each_candidate_in_order_through_the_read_stage_cache.rs
@@ -0,0 +1,633 @@
+//! 可写挂载（`mount::mount_writable`）的读阶段取前缀时，水位之上每个候选在块层读几次盘、按什么次序读（C648）。
+//! 只读挂载那一半在 `mounted_recovery_reads_each_candidate_once_in_order.rs`。
+//!
+//! 可写挂载的读阶段经读缓存（`mount.rs` 的 `ReadStageCache`）读：同一个 (盘, 偏移, 长度) 读出来不全是 0 的，第二次起不落到块层。
+//! 扫环（块设备不交环提示，每块盘整环逐槽读）已经把每条记录读进缓存，反向链重读上一条记录因此一次都不落到块层；
+//! 点名单元在扫环时没读过，每个点名单元的两份各落到块层一次，按候选次序、按点名次序。
+//! 这里在块层逐次记读（写、写零、屏障也记一项，不比），钉住扫完环之后、判据 N-配置 直接读系统配置槽（每块盘两槽，不经缓存）之前的那一截。
+//! 健康镜像上反向链回调多调一次（攻方 S03）、末条提交标记那一判挪到反向链回调之前（攻方 S07）都只多读或少读一次缓存里有的记录，块层看不出；
+//! 缓存不收读回全零的落点，所以另有两条用例把上一条记录在 0 号盘（反向链重读先试的那块）上的一份写成全零：
+//! 那一份每次反向链重读都落到块层一次，S03、S07 在块层看得出。让本该在点名单元之前断的候选去读点名单元，在健康镜像上就看得出。
+//!
+//! 造法：A（txg 3、jsn 3）的根落了；之后的发布只剩记录与单元——根槽改坏，系统配置槽写回 A 那次发布之后的字节
+//! （D16（发布语义） 已定项 7：轮换在根槽 FUA 之后，根槽没落、轮换也没落），判据 N-配置 因此判假，读阶段只走一遍。
+
+mod common;
+
+use std::cell::RefCell;
+use std::rc::Rc;
+
+use common::{
+    block_read_rereading_the_record_before, block_reads_of_the_named_units_of, build_pool,
+    crash_state_devices, is_inside_the_journal_ring,
+    overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
+    rewrite_journal_record_and_rechain_the_records_after_it,
+    sequential_write_in_process_as_instance_one_and_advance_the_pool,
+    system_configuration_slots_of_every_device, tear_the_root_slot_written_for_txg,
+    write_back_system_configuration_slots, BlockLayerRead, CONTENT_BYTES_OF_PUBLISH_B,
+    CONTENT_BYTES_OF_PUBLISH_C, FIRST_DEVICE_TRIED_BY_THE_BACK_CHAIN_REREAD,
+};
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+};
+use singlefs_core::block_device::{
+    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
+};
+use singlefs_core::journal::{record_offset, JournalRecord, JournalRecordPlaceInPublish};
+use singlefs_core::mount::{mount_writable, Mounted};
+use singlefs_core::recovery::{choose_system_configuration, scan_journal};
+use singlefs_core::unit::data_unit_payload_capacity;
+use singlefs_format::{
+    JOURNAL_RECORD_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES,
+};
+use singlefs_harness::memory_pool::MemoryPool;
+use singlefs_harness::SharedStream;
+
+/// B、C 两版文件的填充字节。
+const CONTENT_FILL_BYTE_OF_PUBLISH_B: u8 = 0xB0;
+const CONTENT_FILL_BYTE_OF_PUBLISH_C: u8 = 0xC0;
+/// 两条记录那段历史里 B 的填充字节。
+const CONTENT_FILL_BYTE_OF_THE_TWO_RECORD_PUBLISH: u8 = 0xD0;
+/// 池里两块盘：扫环每块盘各扫一遍整环。
+const DEVICES_THE_JOURNAL_RING_IS_SCANNED_ON: usize = 2;
+/// 点名验证把每个点名单元的两份落点各读一次（`NamedUnit::locations` 是两项）。
+const READS_PER_NAMED_UNIT: usize = 2;
+/// 一次单条覆盖写的记录点名的单元数。
+const NAMED_UNITS_OF_A_SINGLE_RECORD_OVERWRITE: usize = 12;
+/// 顺序写两个数据单元：前一条只点名自己那个数据单元。
+const NAMED_UNITS_OF_THE_FIRST_RECORD_OF_THE_TWO_RECORD_PUBLISH: usize = 1;
+/// 上一条记录在 0 号盘上那一份读回全零时，一个候选的反向链那一判在块层重读它这么多次（缓存不收全零，1 号盘那一份从缓存交回）。
+const BLOCK_LAYER_REREADS_OF_A_ZEROED_PREVIOUS_COPY_PER_CANDIDATE: usize = 1;
+/// 单条发布的 B、C 两个候选。
+const CANDIDATES_OF_THE_SINGLE_RECORD_PUBLISHES: usize = 2;
+
+/// 块层的一件事：读原样记下；写、写零、屏障只记发生过（这几条用例不比它们）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum BlockLayerEvent {
+    Read(BlockLayerRead),
+    WriteOrWriteZeroesOrBarrier { device: DeviceIdentity },
+}
+
+/// 包在一块盘外面，把每次读按发生的次序记进几块盘共用的一份记录里。
+struct ReadLoggingBlockDevice<Inner: BlockDevice> {
+    identity: DeviceIdentity,
+    inner: Inner,
+    events: Rc<RefCell<Vec<BlockLayerEvent>>>,
+}
+
+impl<Inner: BlockDevice> BlockDevice for ReadLoggingBlockDevice<Inner> {
+    fn read_at(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        self.events
+            .borrow_mut()
+            .push(BlockLayerEvent::Read(BlockLayerRead {
+                device: self.identity,
+                offset,
+                length_in_bytes: buffer.len(),
+            }));
+        self.inner.read_at(offset, buffer)
+    }
+
+    fn write_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+        durability: WriteDurability,
+    ) -> Result<(), BlockDeviceError> {
+        self.events
+            .borrow_mut()
+            .push(BlockLayerEvent::WriteOrWriteZeroesOrBarrier {
+                device: self.identity,
+            });
+        self.inner.write_at(offset, bytes, durability)
+    }
+
+    fn write_zeroes_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<(), BlockDeviceError> {
+        self.events
+            .borrow_mut()
+            .push(BlockLayerEvent::WriteOrWriteZeroesOrBarrier {
+                device: self.identity,
+            });
+        self.inner.write_zeroes_at(offset, length)
+    }
+
+    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
+        self.events
+            .borrow_mut()
+            .push(BlockLayerEvent::WriteOrWriteZeroesOrBarrier {
+                device: self.identity,
+            });
+        self.inner.barrier()
+    }
+
+    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
+        self.inner.probe_physical_block_size()
+    }
+
+    fn size_in_bytes(&self) -> u64 {
+        self.inner.size_in_bytes()
+    }
+}
+
+/// 一次可写挂载的块层事件，切在扫环最后一次读之后。
+struct BlockLayerEventsOfOneWritableMount {
+    /// 扫环那一段连着的环内读的次数。
+    scan_reads_inside_the_ring: usize,
+    /// 扫完环之后的事件。
+    events_after_the_scan: Vec<BlockLayerEvent>,
+}
+
+/// 在 `image` 上可写挂载一次，块层每件事按发生的次序记下来；扫环是第一次环内读起、两块盘各整环的那一段。
+fn mount_writable_logging_every_block_layer_event(
+    image: &MemoryPool,
+    ring_bytes: u64,
+) -> (Mounted, BlockLayerEventsOfOneWritableMount) {
+    let events = Rc::new(RefCell::new(Vec::new()));
+    let mut devices: Vec<(DeviceIdentity, ReadLoggingBlockDevice<_>)> =
+        crash_state_devices(image, &[], &[], &SharedStream::retaining_contents())
+            .into_iter()
+            .map(|(identity, inner)| {
+                (
+                    identity,
+                    ReadLoggingBlockDevice {
+                        identity,
+                        inner,
+                        events: Rc::clone(&events),
+                    },
+                )
+            })
+            .collect();
+    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
+    drop(devices);
+    let mut events = Rc::try_unwrap(events)
+        .expect("盘都丢了，只剩这一份")
+        .into_inner();
+    let is_ring_read = |event: &BlockLayerEvent| match event {
+        BlockLayerEvent::Read(read) => is_inside_the_journal_ring(read.offset, ring_bytes),
+        BlockLayerEvent::WriteOrWriteZeroesOrBarrier { .. } => false,
+    };
+    let scan_start = events.iter().position(is_ring_read).expect("读阶段扫过环");
+    let ring_slots_per_device = usize::try_from(ring_bytes / JOURNAL_RECORD_BYTES).expect("槽数");
+    let scan_end = scan_start + ring_slots_per_device * DEVICES_THE_JOURNAL_RING_IS_SCANNED_ON;
+    let scan_reads_inside_the_ring = events[scan_start..scan_end.min(events.len())]
+        .iter()
+        .filter(|event| is_ring_read(event))
+        .count();
+    let events_after_the_scan = events.split_off(scan_end.min(events.len()));
+    (
+        mounted,
+        BlockLayerEventsOfOneWritableMount {
+            scan_reads_inside_the_ring,
+            events_after_the_scan,
+        },
+    )
+}
+
+/// 判据 N-配置 在取前缀之后直接读系统配置槽：按盘号次序，每块盘两槽各一次。
+fn witness_reads_of_the_system_configuration_slots() -> Vec<BlockLayerEvent> {
+    let slot_spacing_in_bytes = u64::from(parameters().geometry.fixed_structure_slot_spacing);
+    [DeviceIdentity(0), DeviceIdentity(1)]
+        .into_iter()
+        .flat_map(|device| {
+            (0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE).map(move |slot| {
+                BlockLayerEvent::Read(BlockLayerRead {
+                    device,
+                    offset: DeviceOffsetInBytes(slot * slot_spacing_in_bytes),
+                    length_in_bytes: usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES)
+                        .expect("4096"),
+                })
+            })
+        })
+        .collect()
+}
+
+fn named_unit_events_of(record: &JournalRecord) -> Vec<BlockLayerEvent> {
+    block_reads_of_the_named_units_of(record)
+        .into_iter()
+        .map(BlockLayerEvent::Read)
+        .collect()
+}
+
+/// 候选 `record` 的反向链那一判在 0 号盘上重读它的上一条记录、落到块层的那一次。
+fn back_chain_reread_event_of(record: &JournalRecord, ring_bytes: u64) -> BlockLayerEvent {
+    BlockLayerEvent::Read(block_read_rereading_the_record_before(record, ring_bytes))
+}
+
+/// 把 jsn `counter` 那条记录在 0 号盘（反向链重读先试的那块）上的一份写成全零，1 号盘那一份不动：
+/// 扫环从 1 号盘收下这条；反向链重读先读 0 号盘那一份，读回全零、读缓存不收，每次都落到块层，再从缓存里的 1 号盘那一份判链。
+fn zero_the_copy_the_back_chain_reread_tries_first(
+    image: &mut MemoryPool,
+    counter: u64,
+    ring_bytes: u64,
+) {
+    image
+        .devices
+        .get_mut(&FIRST_DEVICE_TRIED_BY_THE_BACK_CHAIN_REREAD)
+        .expect("有这块盘")
+        .write(
+            record_offset(counter, ring_bytes),
+            &vec![0; usize::try_from(JOURNAL_RECORD_BYTES).expect("4096")],
+        );
+}
+
+/// 镜像上择出的系统配置里的环长，与 jsn `counters` 那几条扫出来的记录。
+fn ring_bytes_and_records(image: &MemoryPool, counters: &[u64]) -> (u64, Vec<JournalRecord>) {
+    let system_configuration = choose_system_configuration(image).expect("系统配置择得出");
+    let records = scan_journal(image, &system_configuration);
+    (
+        system_configuration.immutable.sizes.journal_ring_bytes,
+        counters
+            .iter()
+            .map(|counter| {
+                records
+                    .get(&(InstanceGeneration(1), *counter))
+                    .expect("这条记录在环里")
+                    .clone()
+            })
+            .collect(),
+    )
+}
+
+/// A 的根落了，B（jsn 4）、C（jsn 5）两次单条发布只剩记录与单元：扫完环之后，jsn 4 的反向链重读 jsn 3、jsn 5 的重读 jsn 4
+/// 都从读缓存交回、不落到块层；块层只有 jsn 4 点名的单元各两份、再 jsn 5 点名的单元各两份，之后是判据 N-配置 读系统配置槽。
+#[test]
+fn writable_mount_reads_each_named_unit_once_per_candidate_in_journal_order_and_the_back_chain_reread_stays_in_the_read_stage_cache(
+) {
+    let mut pool = build_pool("writable-mount-reads-single-record-publishes");
+    let first_version = (pool.output.root.instance, pool.output.root.checkpoint_txg);
+    let system_configuration_after_the_first_version =
+        system_configuration_slots_of_every_device(&pool);
+    let second = overwrite_in_process_as_instance_one_and_advance_the_pool(
+        &mut pool,
+        &vec![CONTENT_FILL_BYTE_OF_PUBLISH_B; CONTENT_BYTES_OF_PUBLISH_B],
+    );
+    let third = overwrite_in_process_as_instance_one_and_advance_the_pool(
+        &mut pool,
+        &vec![CONTENT_FILL_BYTE_OF_PUBLISH_C; CONTENT_BYTES_OF_PUBLISH_C],
+    );
+    assert_eq!(
+        (
+            first_version,
+            (second.root.checkpoint_txg, second.record.counter),
+            (third.root.checkpoint_txg, third.record.counter)
+        ),
+        (
+            (InstanceGeneration(1), CheckpointTxg(3)),
+            (CheckpointTxg(4), 4),
+            (CheckpointTxg(5), 5)
+        ),
+        "A 的根在 txg 3、jsn 3；B、C 各一条记录"
+    );
+    let mut image = pool.memory_pool();
+    for txg in [4u64, 5] {
+        tear_the_root_slot_written_for_txg(&mut image, txg);
+    }
+    write_back_system_configuration_slots(
+        &mut image,
+        &system_configuration_after_the_first_version,
+    );
+    let (ring_bytes, candidates) = ring_bytes_and_records(&image, &[4, 5]);
+
+    let (mounted, events) = mount_writable_logging_every_block_layer_event(&image, ring_bytes);
+    assert_eq!(
+        (
+            (
+                mounted.output.chosen_root.instance,
+                mounted.output.chosen_root.checkpoint_txg
+            ),
+            (
+                mounted.output.effective_root.instance,
+                mounted.output.effective_root.checkpoint_txg
+            ),
+            mounted.output.journal.above_water,
+            mounted.output.journal.prefix_applied,
+            mounted.output.journal.verification_passed
+        ),
+        (
+            first_version,
+            (InstanceGeneration(1), CheckpointTxg(5)),
+            2,
+            2,
+            2
+        ),
+        "所选根是 A 的；水位之上 B、C 两条，都过了点名验证、都施加"
+    );
+
+    let mut expected_events_after_the_scan: Vec<BlockLayerEvent> =
+        candidates.iter().flat_map(named_unit_events_of).collect();
+    assert_eq!(
+        expected_events_after_the_scan.len(),
+        candidates.len() * NAMED_UNITS_OF_A_SINGLE_RECORD_OVERWRITE * READS_PER_NAMED_UNIT,
+        "判两个候选在块层的读：各自点名的 12 个单元两份各一次，反向链重读一次都不落到块层"
+    );
+    expected_events_after_the_scan.extend(witness_reads_of_the_system_configuration_slots());
+    assert_eq!(
+        (
+            events.scan_reads_inside_the_ring,
+            events
+                .events_after_the_scan
+                .get(..expected_events_after_the_scan.len())
+        ),
+        (
+            usize::try_from(ring_bytes / JOURNAL_RECORD_BYTES).expect("槽数")
+                * DEVICES_THE_JOURNAL_RING_IS_SCANNED_ON,
+            Some(expected_events_after_the_scan.as_slice())
+        ),
+        "扫环两块盘各整环逐槽读一遍；之后 jsn 4 点名的单元、jsn 5 点名的单元各读一次（不重读 jsn 3、jsn 4），再是系统配置槽"
+    );
+}
+
+/// 两条记录的 B（jsn 4 不带末条标志、点名 1 个数据单元；jsn 5 带末条标志），jsn 5 的提交标记改成 0（重封）：
+/// jsn 4 点名的单元两份各落到块层一次；jsn 5 的反向链重读从读缓存交回，随即断在「末条不带提交标记」上，它点名的单元一个都不读。
+/// B 整次不施加，挂载在 A 上写行。
+#[test]
+fn writable_mount_cuts_a_last_record_without_its_commit_marker_before_reading_its_named_units() {
+    let mut pool = build_pool("writable-mount-reads-last-record-without-commit");
+    let first_version = (pool.output.root.instance, pool.output.root.checkpoint_txg);
+    let system_configuration_after_the_first_version =
+        system_configuration_slots_of_every_device(&pool);
+    let payload_capacity = data_unit_payload_capacity();
+    let second = sequential_write_in_process_as_instance_one_and_advance_the_pool(
+        &mut pool,
+        &vec![CONTENT_FILL_BYTE_OF_THE_TWO_RECORD_PUBLISH; payload_capacity + payload_capacity / 2],
+    );
+    assert_eq!(
+        (
+            first_version,
+            second.root.checkpoint_txg,
+            second.record.counter
+        ),
+        (
+            (InstanceGeneration(1), CheckpointTxg(3)),
+            CheckpointTxg(4),
+            5
+        ),
+        "A 的根在 txg 3、jsn 3；B 两个数据单元切出 jsn 4、5"
+    );
+    let mut image = pool.memory_pool();
+    tear_the_root_slot_written_for_txg(&mut image, 4);
+    write_back_system_configuration_slots(
+        &mut image,
+        &system_configuration_after_the_first_version,
+    );
+    rewrite_journal_record_and_rechain_the_records_after_it(&mut image, 5, |record| {
+        record.is_commit = false;
+    });
+    let (ring_bytes, candidates) = ring_bytes_and_records(&image, &[4, 5]);
+    let [first_record_of_b, last_record_of_b] =
+        <[_; 2]>::try_from(candidates).expect("jsn 4、5 两条");
+    assert_eq!(
+        (
+            first_record_of_b.named.len(),
+            last_record_of_b.place_in_publish,
+            last_record_of_b.is_commit
+        ),
+        (
+            NAMED_UNITS_OF_THE_FIRST_RECORD_OF_THE_TWO_RECORD_PUBLISH,
+            JournalRecordPlaceInPublish::LastRecordOfThePublish,
+            false
+        ),
+        "jsn 4 点名 1 个单元；jsn 5 带末条标志、不带提交标记"
+    );
+
+    let (mounted, events) = mount_writable_logging_every_block_layer_event(&image, ring_bytes);
+    assert_eq!(
+        (
+            (
+                mounted.output.chosen_root.instance,
+                mounted.output.chosen_root.checkpoint_txg
+            ),
+            (
+                mounted.output.effective_root.instance,
+                mounted.output.effective_root.checkpoint_txg
+            ),
+            mounted.output.journal.above_water,
+            mounted.output.journal.prefix_applied,
+            mounted.output.journal.verification_passed
+        ),
+        (first_version, first_version, 2, 0, 1),
+        "jsn 4 过了点名验证；jsn 5 断在提交标记上，B 整次不施加，挂载在 A"
+    );
+
+    let mut expected_events_after_the_scan = named_unit_events_of(&first_record_of_b);
+    assert_eq!(
+        expected_events_after_the_scan.len(),
+        NAMED_UNITS_OF_THE_FIRST_RECORD_OF_THE_TWO_RECORD_PUBLISH * READS_PER_NAMED_UNIT,
+        "判两条在块层的读：只有 jsn 4 点名单元的两份；两条的反向链重读都在读缓存里，jsn 5 不读点名单元"
+    );
+    expected_events_after_the_scan.extend(witness_reads_of_the_system_configuration_slots());
+    assert_eq!(
+        events
+            .events_after_the_scan
+            .get(..expected_events_after_the_scan.len()),
+        Some(expected_events_after_the_scan.as_slice()),
+        "扫完环之后：jsn 4 点名的单元读一次，jsn 5 一个都不读，再是系统配置槽"
+    );
+}
+
+/// 单条发布的 B（jsn 4）、C（jsn 5），jsn 3、jsn 4 在 0 号盘上那一份写成全零：jsn 4 的反向链重读 jsn 3、jsn 5 的重读 jsn 4，
+/// 各自在 0 号盘上落到块层恰好一次（读回全零、缓存不收），再从缓存里 1 号盘那一份判链、链对得上。
+/// 块层：jsn 3 的零份一次、jsn 4 点名的单元各两份、jsn 4 的零份一次、jsn 5 点名的单元各两份，之后是判据 N-配置 读系统配置槽。
+#[test]
+fn writable_mount_rereads_a_previous_record_whose_first_copy_reads_back_as_zeros_from_the_block_layer_once_per_candidate(
+) {
+    let mut pool = build_pool("writable-mount-reads-zeroed-first-copies");
+    let first_version = (pool.output.root.instance, pool.output.root.checkpoint_txg);
+    let system_configuration_after_the_first_version =
+        system_configuration_slots_of_every_device(&pool);
+    let second = overwrite_in_process_as_instance_one_and_advance_the_pool(
+        &mut pool,
+        &vec![CONTENT_FILL_BYTE_OF_PUBLISH_B; CONTENT_BYTES_OF_PUBLISH_B],
+    );
+    let third = overwrite_in_process_as_instance_one_and_advance_the_pool(
+        &mut pool,
+        &vec![CONTENT_FILL_BYTE_OF_PUBLISH_C; CONTENT_BYTES_OF_PUBLISH_C],
+    );
+    assert_eq!(
+        (
+            first_version,
+            (second.root.checkpoint_txg, second.record.counter),
+            (third.root.checkpoint_txg, third.record.counter)
+        ),
+        (
+            (InstanceGeneration(1), CheckpointTxg(3)),
+            (CheckpointTxg(4), 4),
+            (CheckpointTxg(5), 5)
+        ),
+        "A 的根在 txg 3、jsn 3；B、C 各一条记录"
+    );
+    let mut image = pool.memory_pool();
+    for txg in [4u64, 5] {
+        tear_the_root_slot_written_for_txg(&mut image, txg);
+    }
+    write_back_system_configuration_slots(
+        &mut image,
+        &system_configuration_after_the_first_version,
+    );
+    let ring_bytes = choose_system_configuration(&image)
+        .expect("系统配置择得出")
+        .immutable
+        .sizes
+        .journal_ring_bytes;
+    for counter in [3u64, 4] {
+        zero_the_copy_the_back_chain_reread_tries_first(&mut image, counter, ring_bytes);
+    }
+    let (_, candidates) = ring_bytes_and_records(&image, &[4, 5]);
+    let [candidate_b, candidate_c] = <[_; 2]>::try_from(candidates).expect("jsn 4、5 两条");
+
+    let (mounted, events) = mount_writable_logging_every_block_layer_event(&image, ring_bytes);
+    assert_eq!(
+        (
+            (
+                mounted.output.chosen_root.instance,
+                mounted.output.chosen_root.checkpoint_txg
+            ),
+            (
+                mounted.output.effective_root.instance,
+                mounted.output.effective_root.checkpoint_txg
+            ),
+            mounted.output.journal.above_water,
+            mounted.output.journal.prefix_applied,
+            mounted.output.journal.verification_passed
+        ),
+        (
+            first_version,
+            (InstanceGeneration(1), CheckpointTxg(5)),
+            2,
+            2,
+            2
+        ),
+        "所选根是 A 的；jsn 3、4 的 1 号盘那一份在，反向链对得上，B、C 都过了点名验证、都施加"
+    );
+
+    let mut expected_events_after_the_scan =
+        vec![back_chain_reread_event_of(&candidate_b, ring_bytes)];
+    expected_events_after_the_scan.extend(named_unit_events_of(&candidate_b));
+    expected_events_after_the_scan.push(back_chain_reread_event_of(&candidate_c, ring_bytes));
+    expected_events_after_the_scan.extend(named_unit_events_of(&candidate_c));
+    assert_eq!(
+        expected_events_after_the_scan.len(),
+        CANDIDATES_OF_THE_SINGLE_RECORD_PUBLISHES
+            * (BLOCK_LAYER_REREADS_OF_A_ZEROED_PREVIOUS_COPY_PER_CANDIDATE
+                + NAMED_UNITS_OF_A_SINGLE_RECORD_OVERWRITE * READS_PER_NAMED_UNIT),
+        "判两个候选在块层的读：各自一次 0 号盘零份的反向链重读，加点名的 12 个单元两份各一次"
+    );
+    expected_events_after_the_scan.extend(witness_reads_of_the_system_configuration_slots());
+    assert_eq!(
+        (
+            events.scan_reads_inside_the_ring,
+            events
+                .events_after_the_scan
+                .get(..expected_events_after_the_scan.len())
+        ),
+        (
+            usize::try_from(ring_bytes / JOURNAL_RECORD_BYTES).expect("槽数")
+                * DEVICES_THE_JOURNAL_RING_IS_SCANNED_ON,
+            Some(expected_events_after_the_scan.as_slice())
+        ),
+        "扫完环之后：jsn 4 在 0 号盘重读 jsn 3 一次、读它点名的单元，jsn 5 在 0 号盘重读 jsn 4 一次、读它点名的单元，再是系统配置槽"
+    );
+}
+
+/// 两条记录的 B（jsn 4、5），jsn 5 的提交标记改成 0（重封），jsn 4 在 0 号盘上那一份写成全零：jsn 4 的反向链重读 jsn 3 从缓存交回，
+/// 读它点名的单元；jsn 5 先在 0 号盘上重读 jsn 4、落到块层一次，链对得上，随即断在「末条不带提交标记」上，它点名的单元一个都不读。
+#[test]
+fn writable_mount_rereads_the_zeroed_first_copy_of_the_previous_record_once_before_cutting_a_last_record_without_its_commit_marker(
+) {
+    let mut pool = build_pool("writable-mount-reads-zeroed-copy-before-the-commit-cut");
+    let first_version = (pool.output.root.instance, pool.output.root.checkpoint_txg);
+    let system_configuration_after_the_first_version =
+        system_configuration_slots_of_every_device(&pool);
+    let payload_capacity = data_unit_payload_capacity();
+    let second = sequential_write_in_process_as_instance_one_and_advance_the_pool(
+        &mut pool,
+        &vec![CONTENT_FILL_BYTE_OF_THE_TWO_RECORD_PUBLISH; payload_capacity + payload_capacity / 2],
+    );
+    assert_eq!(
+        (
+            first_version,
+            second.root.checkpoint_txg,
+            second.record.counter
+        ),
+        (
+            (InstanceGeneration(1), CheckpointTxg(3)),
+            CheckpointTxg(4),
+            5
+        ),
+        "A 的根在 txg 3、jsn 3；B 两个数据单元切出 jsn 4、5"
+    );
+    let mut image = pool.memory_pool();
+    tear_the_root_slot_written_for_txg(&mut image, 4);
+    write_back_system_configuration_slots(
+        &mut image,
+        &system_configuration_after_the_first_version,
+    );
+    rewrite_journal_record_and_rechain_the_records_after_it(&mut image, 5, |record| {
+        record.is_commit = false;
+    });
+    let ring_bytes = choose_system_configuration(&image)
+        .expect("系统配置择得出")
+        .immutable
+        .sizes
+        .journal_ring_bytes;
+    zero_the_copy_the_back_chain_reread_tries_first(&mut image, 4, ring_bytes);
+    let (_, candidates) = ring_bytes_and_records(&image, &[4, 5]);
+    let [first_record_of_b, last_record_of_b] =
+        <[_; 2]>::try_from(candidates).expect("jsn 4、5 两条");
+    assert_eq!(
+        (
+            first_record_of_b.named.len(),
+            last_record_of_b.place_in_publish,
+            last_record_of_b.is_commit
+        ),
+        (
+            NAMED_UNITS_OF_THE_FIRST_RECORD_OF_THE_TWO_RECORD_PUBLISH,
+            JournalRecordPlaceInPublish::LastRecordOfThePublish,
+            false
+        ),
+        "jsn 4 点名 1 个单元；jsn 5 带末条标志、不带提交标记"
+    );
+
+    let (mounted, events) = mount_writable_logging_every_block_layer_event(&image, ring_bytes);
+    assert_eq!(
+        (
+            (
+                mounted.output.chosen_root.instance,
+                mounted.output.chosen_root.checkpoint_txg
+            ),
+            (
+                mounted.output.effective_root.instance,
+                mounted.output.effective_root.checkpoint_txg
+            ),
+            mounted.output.journal.above_water,
+            mounted.output.journal.prefix_applied,
+            mounted.output.journal.verification_passed
+        ),
+        (first_version, first_version, 2, 0, 1),
+        "jsn 4 过了点名验证；jsn 5 断在提交标记上，B 整次不施加，挂载在 A"
+    );
+
+    let mut expected_events_after_the_scan = named_unit_events_of(&first_record_of_b);
+    expected_events_after_the_scan.push(back_chain_reread_event_of(&last_record_of_b, ring_bytes));
+    assert_eq!(
+        expected_events_after_the_scan.len(),
+        NAMED_UNITS_OF_THE_FIRST_RECORD_OF_THE_TWO_RECORD_PUBLISH * READS_PER_NAMED_UNIT
+            + BLOCK_LAYER_REREADS_OF_A_ZEROED_PREVIOUS_COPY_PER_CANDIDATE,
+        "判两条在块层的读：jsn 4 点名单元的两份，jsn 5 在 0 号盘重读 jsn 4 零份一次，jsn 5 不读点名单元"
+    );
+    expected_events_after_the_scan.extend(witness_reads_of_the_system_configuration_slots());
+    assert_eq!(
+        events
+            .events_after_the_scan
+            .get(..expected_events_after_the_scan.len()),
+        Some(expected_events_after_the_scan.as_slice()),
+        "扫完环之后：jsn 4 点名的单元读一次，jsn 5 在提交标记那一判之前重读 jsn 4 的零份一次、一个点名单元都不读，再是系统配置槽"
+    );
+}
```

### 追加进 crates/mutations.tsv 的行

```
I-8.3 重放前缀严格连续（形式化题面第二轮第五节第 2 条，R2-4）：接链只比实例代号、不比计数器，所选根之后断号的下一条照样接上	crates/singlefs-replay-prefix/src/lib.rs	            if record.instance.0 != expected_key.0.0 || record.counter.0 != expected_key.1.0 {	            if record.instance.0 != expected_key.0.0 {	-p singlefs-harness --test writable_mount_takes_a_second_instance -- one_missing_record_right_after_the_chosen_root_stops_the_prefix	one_missing_record_right_after_the_chosen_root_stops_the_prefix_even_when_later_records_are_intact
可写挂载读阶段调 replay_journal 的 verify_named_units 从 true 改成 false（形式化题面第三轮第四节第 9 条，R3-9 之一）：崩溃后再挂载施加记录不验点名单元	crates/singlefs-core/src/mount.rs	        &records,\n        true,\n    )?;\n    let witness = newer_publish_witness(	        &records,\n        false,\n    )?;\n    let witness = newer_publish_witness(	-p singlefs-harness --test presumed_clause_checks -- c499_remount_after_a_clean_close	c499_remount_after_a_clean_close_and_after_a_crash_both_scan_the_whole_ring_and_recover_by_the_same_rules
只读挂载调 replay_journal 的 verify_named_units 从 true 改成 false（形式化题面第三轮第四节第 9 条，R3-9 之二）：施加记录不验点名单元	crates/singlefs-core/src/mounted_read.rs	        &records,\n        true,\n    )\n    .map_err(MountReadOnlyFailure::Recovery)?;	        &records,\n        false,\n    )\n    .map_err(MountReadOnlyFailure::Recovery)?;	-p singlefs-harness --test mounted_recovery_reads_each_candidate_once_in_order -- mounting_rereads_the_previous_record_once	mounting_rereads_the_previous_record_once_then_each_named_unit_of_every_candidate_in_journal_order
recover 的 Consult 一臂调 replay_journal 的 verify_named_units 从「policy 是 Consult」改成 false（形式化题面第三轮第四节第 9 条，R3-9 之三）：验的那一臂也不验点名单元	crates/singlefs-core/src/recovery.rs	                policy == JournalPolicy::Consult,	                false,	-p singlefs-harness --test recovery_reads_the_created_file_after_reopen -- the_named_unit_verification_switch_shows_up_in_the_runtime_counter	the_named_unit_verification_switch_shows_up_in_the_runtime_counter
scan_journal 建表同一 (实例代号, 计数器) 先到先得改成后到覆盖（形式化题面第三轮第二节 S3、第四节第 10 条，R3-10）：键唯一那一前提在真路径上换成后扫的那块盘的那一份	crates/singlefs-core/src/recovery.rs	                records\n                    .entry((record.instance, record.counter))\n                    .or_insert(record);	                records.insert((record.instance, record.counter), record);	-p singlefs-harness --test journal_scan_keeps_the_first_copy_of_a_duplicated_record_key	of_two_differing_copies_of_one_record_key_the_scan_keeps_the_copy_on_the_device_scanned_first
由记录重建的根改抄所选根的卸载记号（形式化题面第三轮第二节 S1 后半、第四节第 1 条，R3-1）：卸载那一串第二次空发布的记录施加出的根成了「卸载那一串写下的」	crates/singlefs-core/src/recovery.rs	                unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,	                unmount_marker: root.unmount_marker,	-p singlefs-harness --test rebuilt_root_keeps_the_chosen_roots_fields_except_the_unmount_marker	a_root_rebuilt_from_the_second_unmount_publish_record_keeps_the_chosen_roots_other_fields_and_is_not_marked_by_the_unmount_sequence
可写挂载读阶段缓存不存读到的字节（C648）：扫环读进来的记录不留，反向链重读上一条记录落到块层	crates/singlefs-core/src/mount.rs	        if bytes.iter().any(|byte| *byte != 0) {	        if false && bytes.iter().any(|byte| *byte != 0) {	-p singlefs-harness --test writable_mount_reads_each_candidate_in_order_through_the_read_stage_cache	writable_mount_reads_each_named_unit_once_per_candidate_in_journal_order_and_the_back_chain_reread_stays_in_the_read_stage_cache
取前缀核读盘形状（C648，S07 一类）：末条不带提交标记那一判之前先调一次点名回调，可写挂载时被断的末条也读它点名的单元	crates/singlefs-replay-prefix/src/lib.rs	    if !record.is_commit && record_ends_its_publish(record) {\n        return CandidateVerdict::FailsACheckBeforeTheNamedUnits;\n    }	    if !record.is_commit && record_ends_its_publish(record) {\n        let _named_units_read_before_the_commit_marker_cut = named_units_verified(index);\n        return CandidateVerdict::FailsACheckBeforeTheNamedUnits;\n    }	-p singlefs-harness --test writable_mount_reads_each_candidate_in_order_through_the_read_stage_cache	writable_mount_cuts_a_last_record_without_its_commit_marker_before_reading_its_named_units
取前缀核读盘形状（攻方 H08）：反向链断了之后再调一次点名回调，只读挂载时断在反向链上的候选也读它点名的单元	crates/singlefs-replay-prefix/src/lib.rs	    if back_chain_broken(index) {\n        return CandidateVerdict::FailsACheckBeforeTheNamedUnits;\n    }	    if back_chain_broken(index) {\n        let _named_units_read_anyway = named_units_verified(index);\n        return CandidateVerdict::FailsACheckBeforeTheNamedUnits;\n    }	-p singlefs-harness --test mounted_recovery_reads_each_candidate_once_in_order	a_candidate_whose_back_chain_is_broken_is_cut_after_rereading_its_previous_record_and_before_reading_its_named_units
取前缀核读盘形状（攻方 H09）：到了在飞上限之后对下一个候选先调一次反向链回调再停，3 槽环上 jsn 5 多重读一次 jsn 4	crates/singlefs-replay-prefix/src/lib.rs	        if records_taken_toward_the_in_flight_limit >= in_flight_limit {\n            break ;	        if records_taken_toward_the_in_flight_limit >= in_flight_limit {\n            let _back_chain_read_anyway = back_chain_broken(index);\n            break ;	-p singlefs-harness --test mounted_recovery_reads_each_candidate_once_in_order	at_the_in_flight_limit_the_next_candidate_is_cut_before_rereading_its_previous_record
取前缀核读盘形状（C648 攻方 S03，可写挂载、上一条记录 0 号盘那一份全零那一格）：反向链回调每个候选调两次，可写挂载时读缓存不收的零份每个候选在块层重读两次	crates/singlefs-replay-prefix/src/lib.rs	    if back_chain_broken(index) {	    if back_chain_broken(index) || back_chain_broken(index) {	-p singlefs-harness --test writable_mount_reads_each_candidate_in_order_through_the_read_stage_cache -- writable_mount_rereads_a_previous_record_whose_first_copy	writable_mount_rereads_a_previous_record_whose_first_copy_reads_back_as_zeros_from_the_block_layer_once_per_candidate
取前缀核读盘形状（C648 攻方 S07，可写挂载、上一条记录 0 号盘那一份全零那一格）：末条提交标记那一判挪到反向链回调之前，可写挂载时末条不带提交标记的候选不再在块层重读上一条记录的零份	crates/singlefs-replay-prefix/src/lib.rs	    if back_chain_broken(index) {\n        return CandidateVerdict::FailsACheckBeforeTheNamedUnits;\n    }	    if !record.is_commit && record_ends_its_publish(record) {\n        return CandidateVerdict::FailsACheckBeforeTheNamedUnits;\n    }\n    if back_chain_broken(index) {\n        return CandidateVerdict::FailsACheckBeforeTheNamedUnits;\n    }	-p singlefs-harness --test writable_mount_reads_each_candidate_in_order_through_the_read_stage_cache -- writable_mount_rereads_the_zeroed_first_copy	writable_mount_rereads_the_zeroed_first_copy_of_the_previous_record_once_before_cutting_a_last_record_without_its_commit_marker
```

## 实现员补丁 m3-formal-7-2（`/tmp/claude-1000/m3-formal-7-2/patch/crates.patch`）

```diff
diff --git a/crates/singlefs-replay-prefix/src/lib.rs b/crates/singlefs-replay-prefix/src/lib.rs
--- a/crates/singlefs-replay-prefix/src/lib.rs
+++ b/crates/singlefs-replay-prefix/src/lib.rs
@@ -7,6 +7,8 @@
 //! spec 照 D23（journal 的角色与格式）已定项 4、7、8、14、18、22 与 D13（验证路线）已定项 1 的 M2 那一行写（`replay_follows_the_prefix_rules`）：
 //! 施加哪一截由逐条判定（链首与 jsn 严格连续、反向链、读者规则、提交标记、末条之后同一次发布、点名单元）与在飞上限一起定，
 //! 报告的五个数与两种报错各有一句。
+//! M3（幂等的记录级与不动点两读、单调推论里核自己的两步：落点不早于所选根、规则只容得下一个结果）写成 `replay_core` 的另四条 ensures，
+//! 单调本身写成推论 `corollary_the_landed_version_does_not_go_back`，择根与写侧那两条前提写在它的 requires 里（D13 已定项 1 的 M3 一行）。
 //! 读盘才判得出的两件（反向链、点名单元）由调用方给两个回调，核走到那一步才调。
 //! spec 只说「核据以接受或断链的那一次回调交回了什么」，不管回调调了几次、按什么次序调；回调自己判得对不对也不在 Verus 里。
 //! 块层读几次盘、按什么次序读，只在只读挂载（`mount_read_only`）的这几种形状上由 harness 用例
@@ -660,6 +662,276 @@
     }
 }
 
+// ===== M3 幂等与单调（spec，可信：照判决文字写）=====
+// D13（验证路线）已定项 1 的 M3 一行；题面照第二轮判决 research/prompts/m3-formal-r2-main-verification.md 第二节第 5、6、7 行
+// 与第三轮判决 research/prompts/m3-formal-r3-main-verification.md 第三节（用户 2026-10-01 定 S5 版本序、S6 txg 顶格）。
+// 主语是 replay_core：singlefs-core 里以 verify_named_units = true 调 replay_journal 的三处走到的就是它（第二轮判决第 7 行）。
+// 下面四条都写成 replay_core 的 ensures（证在 M3 的引理那一节），说的是编进产品的这一份交回的结果；单调本身是推论 corollary_the_landed_version_does_not_go_back。
+
+// I-8.12（K10，判决 research/prompts/m3-inv-b-r1-main-verification.md 第二节第 9 行）：同一实例里 journal 记录的 txg 随计数器不减。
+// 核不判它：记录级的第二半与不动点把它写成前提（核拿到的记录表满足它）。
+pub open spec fn journal_txg_does_not_decrease_with_the_counter_within_an_instance<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+) -> bool {
+    forall|earlier: int, later: int|
+        0 <= earlier < records.len() && 0 <= later < records.len()
+            && #[trigger] is_a_later_record_of_the_same_instance(records, earlier, later)
+            ==> records[earlier].checkpoint_txg.0 <= records[later].checkpoint_txg.0
+}
+
+// 「同一个读者」（第二轮判决第 6 行不动点那一读的前提）：读盘回调在同一个下标上只交回一种答案。
+pub open spec fn the_reader_answers_each_position_one_way<Reader: Fn(usize) -> bool>(
+    reader: Reader,
+) -> bool {
+    forall|position: usize|
+        #[trigger] reader.ensures((position,), true) ==> !reader.ensures((position,), false)
+}
+
+// D23 已定项 18「一次恢复重放的前缀最多 N 条」：不动点的前提「所选根之上的记录条数不超在飞上限」。
+pub open spec fn records_above_the_chosen_root_fit_the_in_flight_limit<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    in_flight_limit: usize,
+) -> bool {
+    candidates_before(records, root_instance, root_checkpoint_txg, records.len() as int).len()
+        <= in_flight_limit
+}
+
+// 落到的那一版：一条没施加时是所选根；施加了时是最后施加的那条记录的 (实例代号, checkpoint_txg)
+// （D23 已定项 15：由记录重建的根取那次发布的 txg）。
+pub open spec fn landed_checkpoint_txg<RootSource>(
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    records: Seq<CoreRecord<RootSource>>,
+    applied: Seq<int>,
+) -> PrefixKernelCheckpointTxg {
+    if applied.len() == 0 {
+        root_checkpoint_txg
+    } else {
+        records[applied.last()].checkpoint_txg
+    }
+}
+
+pub open spec fn landed_instance<RootSource>(
+    root_instance: PrefixKernelInstanceGeneration,
+    records: Seq<CoreRecord<RootSource>>,
+    applied: Seq<int>,
+) -> PrefixKernelInstanceGeneration {
+    if applied.len() == 0 {
+        root_instance
+    } else {
+        records[applied.last()].instance
+    }
+}
+
+// S5：版本序取 (txg, 实例代号)，txg 为主、实例代号破平局，与 crates/singlefs-core/src/recovery.rs 择根键
+// (candidate.checkpoint_txg, candidate.instance) 同序。
+pub open spec fn version_is_not_later_than(
+    earlier_checkpoint_txg: PrefixKernelCheckpointTxg,
+    earlier_instance: PrefixKernelInstanceGeneration,
+    later_checkpoint_txg: PrefixKernelCheckpointTxg,
+    later_instance: PrefixKernelInstanceGeneration,
+) -> bool {
+    earlier_checkpoint_txg.0 < later_checkpoint_txg.0 || (earlier_checkpoint_txg.0
+        == later_checkpoint_txg.0 && earlier_instance.0 <= later_instance.0)
+}
+
+// 施加一条记录对根的作用：一次发布的末条把根换成它带的新根段，其余各条不动根（已定项 14 第五条：施加的单位是一次发布）。
+pub open spec fn root_source_after_applying_the_record<RootSource>(
+    root_source: RootSource,
+    record: CoreRecord<RootSource>,
+) -> RootSource {
+    if is_last_record_of_its_publish(record) {
+        record.new_root_source
+    } else {
+        root_source
+    }
+}
+
+pub open spec fn root_source_after_applying_in_order<RootSource>(
+    root_source: RootSource,
+    applied_records: Seq<CoreRecord<RootSource>>,
+) -> RootSource
+    decreases applied_records.len(),
+{
+    if applied_records.len() == 0 {
+        root_source
+    } else {
+        root_source_after_applying_the_record(
+            root_source_after_applying_in_order(root_source, applied_records.drop_last()),
+            applied_records.last(),
+        )
+    }
+}
+
+// M3 幂等·记录级（第二轮判决第二节第 6 行「记录级」，I-8.4（重放幂等）的字面「任一 journal 记录被重放两次与重放一次，产生的盘上状态相同」）：
+// ① 交回的根就是按次序施加那几条的结果，其中任一条紧接着再施加一次，根与施加一次相同；
+// ② 在 I-8.12 之下，施加过的每一条都不在落到的那一版之上：再取一次前缀时它不是候选，不会被第二次施加。
+#[verifier::opaque]
+pub open spec fn replaying_an_applied_record_twice_leaves_the_root_as_once<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    chosen_root_source: RootSource,
+    records: Seq<CoreRecord<RootSource>>,
+    in_flight_limit: usize,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+    replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
+) -> bool {
+    replayed matches Ok(outcome) ==> {
+        let applied = outcome.applied@;
+        let applied_records = records_at(records, applied);
+        &&& outcome.root_source_after_replay == root_source_after_applying_in_order(
+            chosen_root_source,
+            applied_records,
+        )
+        &&& forall|position: int|
+            0 <= position < applied_records.len() ==> #[trigger] root_source_after_applying_in_order(
+                chosen_root_source,
+                applied_records.insert(position + 1, applied_records[position]),
+            ) == outcome.root_source_after_replay
+        &&& journal_txg_does_not_decrease_with_the_counter_within_an_instance(records)
+            ==> forall|position: int|
+            0 <= position < applied.len() ==> !above_the_chosen_root(
+                records[#[trigger] applied[position]],
+                landed_instance(root_instance, records, applied),
+                landed_checkpoint_txg(root_checkpoint_txg, records, applied),
+            )
+    }
+}
+
+// 不动点那一次取前缀的结局（S6 取 Qn）：一条都不施加、交回的根段就是落到的那一版的；或者因落到的那一版 txg 顶格
+// 报「所选根 txg 没有后继」——报错就一条都没施加，算相等。不许报别的错。
+pub open spec fn replaying_again_from_the_landed_root_stays_put<RootSource>(
+    landed_txg: PrefixKernelCheckpointTxg,
+    landed_root_source: RootSource,
+    replayed_again: Result<CoreOutcome<RootSource>, CoreFailure>,
+) -> bool {
+    match replayed_again {
+        Ok(again) => again.applied@.len() == 0 && again.root_source_after_replay
+            == landed_root_source,
+        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => landed_txg.0 == u64::MAX,
+        Err(
+            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+                counters: _,
+            },
+        ) => false,
+    }
+}
+
+// M3 幂等·不动点（第二轮判决第二节第 6 行「不动点」，第三轮 S6）：把落到的那一版当所选根，同一张记录表、同一个读者、同一个开关、
+// 同一个在飞上限再取一次前缀（第二次取前缀的结果是规则容得下的任一个），根不动。
+// 前提照第二轮判决第 6 行辩方那一句：I-8.12；同一个读者；所选根之上的记录条数不超在飞上限。
+// 「同一 (实例代号, 计数器) 只对应一份内容」由 replay_core 的入参严格升序担保；txg 不顶格那一条照 S6 不当前提，写进结局。
+#[verifier::opaque]
+pub open spec fn replaying_from_the_landed_root_is_a_fixed_point<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    chosen_root_source: RootSource,
+    records: Seq<CoreRecord<RootSource>>,
+    in_flight_limit: usize,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+    replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
+) -> bool {
+    replayed matches Ok(outcome) ==> (journal_txg_does_not_decrease_with_the_counter_within_an_instance(
+        records,
+    ) && the_reader_answers_each_position_one_way(back_chain_broken)
+        && the_reader_answers_each_position_one_way(named_units_verified)
+        && records_above_the_chosen_root_fit_the_in_flight_limit(
+        records,
+        root_instance,
+        root_checkpoint_txg,
+        in_flight_limit,
+    ) ==> forall|replayed_again: Result<CoreOutcome<RootSource>, CoreFailure>|
+        #[trigger] replay_follows_the_prefix_rules(
+            landed_instance(root_instance, records, outcome.applied@),
+            landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),
+            outcome.root_source_after_replay,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed_again,
+        ) ==> replaying_again_from_the_landed_root_stays_put(
+            landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),
+            outcome.root_source_after_replay,
+            replayed_again,
+        ))
+}
+
+// M3 单调（推论，第二轮判决第二节第 5 行、第三轮 S5）里核自己的两半，推论本身是下面的 corollary_the_landed_version_does_not_go_back。
+// 一半（攻方 3.2 用的 J1）：落到的那一版不早于所选根，按 S5 的 (txg, 实例代号) 序。
+pub open spec fn replay_lands_no_earlier_than_the_chosen_root<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    chosen_root_source: RootSource,
+    records: Seq<CoreRecord<RootSource>>,
+    in_flight_limit: usize,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+    replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
+) -> bool {
+    replayed matches Ok(outcome) ==> version_is_not_later_than(
+        root_checkpoint_txg,
+        root_instance,
+        landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),
+        landed_instance(root_instance, records, outcome.applied@),
+    )
+}
+
+// 另一半（攻方 3.2 第 4 步「恢复是确定的函数」）：同样的入参、同一个读者，规则只容得下这一个结果。
+#[verifier::opaque]
+pub open spec fn replay_outcome_is_the_only_one_the_rules_allow<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    chosen_root_source: RootSource,
+    records: Seq<CoreRecord<RootSource>>,
+    in_flight_limit: usize,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+    replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
+) -> bool {
+    replayed matches Ok(outcome) ==> (the_reader_answers_each_position_one_way(back_chain_broken)
+        && the_reader_answers_each_position_one_way(named_units_verified) ==> forall|
+        replayed_again: Result<CoreOutcome<RootSource>, CoreFailure>,
+    |
+        #[trigger] replay_follows_the_prefix_rules(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed_again,
+        ) ==> (replayed_again matches Ok(again) && again.admitted@ == outcome.admitted@
+            && again.applied@ == outcome.applied@ && again.root_source_after_replay
+            == outcome.root_source_after_replay))
+}
+
 // ===== 证明用的记法与引理（不可信件：写错了只会证不出来）=====
 pub open spec fn as_ints(indices: Seq<usize>) -> Seq<int> {
     Seq::new(indices.len(), |position: int| indices[position] as int)
@@ -1050,6 +1322,1444 @@
     }
 }
 
+// ===== M3 的引理（不可信件）=====
+// 候选按切片次序严格递增。主循环不调它（上面 lemma_candidates_lie_below_their_end_and_above_the_chosen_root 的注：两两配对的触发会把 rlimit 用尽），只给 M3 用。
+proof fn lemma_candidates_ascend_in_slice_order<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    end: int,
+)
+    requires
+        0 <= end <= records.len(),
+    ensures
+        ({
+            let candidates = candidates_before(records, root_instance, root_checkpoint_txg, end);
+            forall|earlier: int, later: int|
+                0 <= earlier < later < candidates.len() ==> candidates[earlier] < candidates[later]
+        }),
+    decreases end,
+{
+    if end > 0 {
+        lemma_candidates_ascend_in_slice_order(records, root_instance, root_checkpoint_txg, end - 1);
+        lemma_candidates_lie_below_their_end_and_above_the_chosen_root(
+            records,
+            root_instance,
+            root_checkpoint_txg,
+            end - 1,
+        );
+    }
+}
+
+// 在所选根之上的记录一定是候选：交回它在候选里的位置。
+proof fn lemma_a_record_above_the_chosen_root_is_a_candidate<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    end: int,
+    record_index: int,
+) -> (position: int)
+    requires
+        0 <= record_index < end <= records.len(),
+        above_the_chosen_root(records[record_index], root_instance, root_checkpoint_txg),
+    ensures
+        0 <= position < candidates_before(records, root_instance, root_checkpoint_txg, end).len(),
+        candidates_before(records, root_instance, root_checkpoint_txg, end)[position]
+            == record_index,
+    decreases end,
+{
+    if record_index < end - 1 {
+        lemma_a_record_above_the_chosen_root_is_a_candidate(
+            records,
+            root_instance,
+            root_checkpoint_txg,
+            end - 1,
+            record_index,
+        )
+    } else {
+        candidates_before(records, root_instance, root_checkpoint_txg, end).len() - 1
+    }
+}
+
+proof fn lemma_no_record_above_the_chosen_root_means_no_candidate<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    end: int,
+)
+    requires
+        0 <= end <= records.len(),
+        forall|record_index: int|
+            0 <= record_index < end ==> !#[trigger] above_the_chosen_root(
+                records[record_index],
+                root_instance,
+                root_checkpoint_txg,
+            ),
+    ensures
+        candidates_before(records, root_instance, root_checkpoint_txg, end).len() == 0,
+    decreases end,
+{
+    if end > 0 {
+        lemma_no_record_above_the_chosen_root_means_no_candidate(
+            records,
+            root_instance,
+            root_checkpoint_txg,
+            end - 1,
+        );
+        assert(!above_the_chosen_root(records[end - 1], root_instance, root_checkpoint_txg));
+    }
+}
+
+// 落到的那一版之上的候选，就是原来的候选去掉 boundary 及其之前的那几个：boundary 及其之前没有一条在落到的那一版之上，
+// 之后的在落到的那一版之上与在所选根之上是同一回事。
+proof fn lemma_candidates_above_the_landed_root_are_the_tail<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    landed_txg: PrefixKernelCheckpointTxg,
+    boundary: int,
+    end: int,
+)
+    requires
+        0 <= boundary < end <= records.len(),
+        forall|record_index: int|
+            0 <= record_index <= boundary ==> !#[trigger] above_the_chosen_root(
+                records[record_index],
+                root_instance,
+                landed_txg,
+            ),
+        forall|record_index: int|
+            boundary < record_index < records.len() ==> #[trigger] above_the_chosen_root(
+                records[record_index],
+                root_instance,
+                landed_txg,
+            ) == above_the_chosen_root(records[record_index], root_instance, root_checkpoint_txg),
+    ensures
+        ({
+            let below = candidates_before(
+                records,
+                root_instance,
+                root_checkpoint_txg,
+                boundary + 1,
+            ).len() as int;
+            let all = candidates_before(records, root_instance, root_checkpoint_txg, end);
+            &&& below <= all.len()
+            &&& candidates_before(records, root_instance, landed_txg, end) == all.subrange(
+                below,
+                all.len() as int,
+            )
+        }),
+    decreases end,
+{
+    lemma_candidates_before_a_shorter_end_are_a_prefix(
+        records,
+        root_instance,
+        root_checkpoint_txg,
+        boundary + 1,
+        end,
+    );
+    let below = candidates_before(records, root_instance, root_checkpoint_txg, boundary + 1).len() as int;
+    let all = candidates_before(records, root_instance, root_checkpoint_txg, end);
+    if end == boundary + 1 {
+        lemma_no_record_above_the_chosen_root_means_no_candidate(
+            records,
+            root_instance,
+            landed_txg,
+            end,
+        );
+        assert(candidates_before(records, root_instance, landed_txg, end) =~= all.subrange(
+            below,
+            all.len() as int,
+        ));
+    } else {
+        lemma_candidates_above_the_landed_root_are_the_tail(
+            records,
+            root_instance,
+            root_checkpoint_txg,
+            landed_txg,
+            boundary,
+            end - 1,
+        );
+        lemma_candidates_before_a_shorter_end_are_a_prefix(
+            records,
+            root_instance,
+            root_checkpoint_txg,
+            boundary + 1,
+            end - 1,
+        );
+        assert(above_the_chosen_root(records[end - 1], root_instance, landed_txg)
+            == above_the_chosen_root(records[end - 1], root_instance, root_checkpoint_txg));
+        assert(candidates_before(records, root_instance, landed_txg, end) =~= all.subrange(
+            below,
+            all.len() as int,
+        ));
+    }
+}
+
+// 一次发布的末条过了读者规则第四格（之后同一次发布没有读得出的记录），又有 I-8.12 ⇒ 同实例里切片在它之后的记录 txg 都比它大。
+proof fn lemma_a_publish_end_that_passed_the_fourth_reader_rule_closes_its_txg<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    end_index: int,
+)
+    requires
+        records_ascend_by_instance_and_counter(records),
+        journal_txg_does_not_decrease_with_the_counter_within_an_instance(records),
+        0 <= end_index < records.len(),
+        !a_later_record_of_the_same_publish_is_readable(records, end_index),
+    ensures
+        forall|record_index: int|
+            end_index < record_index < records.len() && (#[trigger] records[record_index]).instance
+                == records[end_index].instance ==> records[record_index].checkpoint_txg.0
+                > records[end_index].checkpoint_txg.0,
+{
+    assert forall|record_index: int|
+        end_index < record_index < records.len() && (#[trigger] records[record_index]).instance
+            == records[end_index].instance implies records[record_index].checkpoint_txg.0
+        > records[end_index].checkpoint_txg.0 by {
+        lemma_slice_order_within_an_instance_follows_the_counter(records, end_index, record_index);
+        assert(is_a_later_record_of_the_same_instance(records, end_index, record_index));
+        if records[record_index].checkpoint_txg.0 <= records[end_index].checkpoint_txg.0 {
+            assert forall|between: int|
+                0 <= between < records.len() && #[trigger] is_a_later_record_of_the_same_instance(
+                    records,
+                    end_index,
+                    between,
+                ) && records[between].counter.0 < records[record_index].counter.0
+                    implies records[between].checkpoint_txg.0
+                <= records[end_index].checkpoint_txg.0 by {
+                assert(is_a_later_record_of_the_same_instance(records, between, record_index));
+            }
+            assert(records[record_index].checkpoint_txg == records[end_index].checkpoint_txg);
+        }
+    }
+}
+
+// 落到的那一版的发布里带末条标志的只有落点那一条 ⇒ 以它为所选根时，带末条标志的计数器就是落点那一条的。
+proof fn lemma_the_flagged_counters_of_the_landed_publish<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    root_instance: PrefixKernelInstanceGeneration,
+    landed_txg: PrefixKernelCheckpointTxg,
+    landed_record_index: int,
+    end: int,
+)
+    requires
+        0 <= landed_record_index < records.len(),
+        0 <= end <= records.len(),
+        records[landed_record_index].instance == root_instance,
+        records[landed_record_index].checkpoint_txg == landed_txg,
+        is_last_record_of_its_publish(records[landed_record_index]),
+        forall|record_index: int|
+            0 <= record_index < records.len() && records[record_index].instance == root_instance
+                && records[record_index].checkpoint_txg == landed_txg
+                && #[trigger] is_last_record_of_its_publish(records[record_index]) ==> record_index
+                == landed_record_index,
+    ensures
+        last_record_flag_counters_of_the_chosen_roots_publish(
+            records,
+            root_instance,
+            landed_txg,
+            end,
+        ) == if end > landed_record_index {
+            seq![records[landed_record_index].counter]
+        } else {
+            Seq::<JournalCounter>::empty()
+        },
+    decreases end,
+{
+    if end > 0 {
+        lemma_the_flagged_counters_of_the_landed_publish(
+            records,
+            root_instance,
+            landed_txg,
+            landed_record_index,
+            end - 1,
+        );
+        if end - 1 == landed_record_index {
+            assert(Seq::<JournalCounter>::empty().push(records[landed_record_index].counter)
+                =~= seq![records[landed_record_index].counter]);
+        }
+    }
+}
+
+// 同一个读者不会在同一个候选上既让一串停下、又让另一串接受它（两串在这一条上的非回调判定相同时）。
+#[allow(clippy::too_many_arguments, reason = "停下那一串与接受那一串各三样判定输入，摊成参数才看得出哪一样两边相同")]
+proof fn lemma_the_same_reader_does_not_both_stop_and_admit_a_candidate<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    records: Seq<CoreRecord<RootSource>>,
+    next: int,
+    stopping_previous: Option<CoreRecord<RootSource>>,
+    stopping_anchor: Option<JournalCounter>,
+    stopping_root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    admitting_previous: Option<CoreRecord<RootSource>>,
+    admitting_anchor: Option<JournalCounter>,
+    admitting_root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+    verification_failed: usize,
+)
+    requires
+        the_reader_answers_each_position_one_way(back_chain_broken),
+        the_reader_answers_each_position_one_way(named_units_verified),
+        continues_the_jsn_chain(
+            records[next],
+            stopping_previous,
+            stopping_anchor,
+            root_instance,
+            stopping_root_checkpoint_txg,
+        ) == continues_the_jsn_chain(
+            records[next],
+            admitting_previous,
+            admitting_anchor,
+            root_instance,
+            admitting_root_checkpoint_txg,
+        ),
+        obeys_the_reader_rules_and_the_commit_markers(records[next], stopping_previous)
+            == obeys_the_reader_rules_and_the_commit_markers(records[next], admitting_previous),
+    ensures
+        !(((verification_failed == 0 && fails_a_check_before_the_named_units(
+            records,
+            next,
+            stopping_previous,
+            stopping_anchor,
+            root_instance,
+            stopping_root_checkpoint_txg,
+            back_chain_broken,
+        )) || (verification_failed == 1 && passes_every_check_before_the_named_units(
+            records,
+            next,
+            stopping_previous,
+            stopping_anchor,
+            root_instance,
+            stopping_root_checkpoint_txg,
+            back_chain_broken,
+        ) && named_units_reported_unverified(named_unit_verification, named_units_verified, next)))
+            && passes_every_check_before_the_named_units(
+            records,
+            next,
+            admitting_previous,
+            admitting_anchor,
+            root_instance,
+            admitting_root_checkpoint_txg,
+            back_chain_broken,
+        ) && named_units_reported_verified(named_unit_verification, named_units_verified, next)),
+{
+    reveal(passes_every_check_before_the_named_units);
+    reveal(fails_a_check_before_the_named_units);
+}
+
+// 同样的入参、同一个读者：规则容得下的两个结果里，过了逐条判定的那一串一样长（短的那一串停下的候选，长的那一串不会接受）。
+#[allow(clippy::too_many_arguments, reason = "入参照 replay_follows_the_prefix_rules 原样摊开")]
+proof fn lemma_the_same_inputs_admit_no_longer_chain<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    chosen_root_source: RootSource,
+    records: Seq<CoreRecord<RootSource>>,
+    in_flight_limit: usize,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+    shorter: CoreOutcome<RootSource>,
+    longer: CoreOutcome<RootSource>,
+)
+    requires
+        replay_follows_the_prefix_rules(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            Ok(shorter),
+        ),
+        replay_follows_the_prefix_rules(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            Ok(longer),
+        ),
+        the_reader_answers_each_position_one_way(back_chain_broken),
+        the_reader_answers_each_position_one_way(named_units_verified),
+    ensures
+        shorter.admitted@.len() >= longer.admitted@.len(),
+{
+    let candidates = candidates_before(
+        records,
+        root_instance,
+        root_checkpoint_txg,
+        records.len() as int,
+    );
+    let anchor = anchor_of_the_chain(records, root_instance, root_checkpoint_txg);
+    let short_chain = shorter.admitted@;
+    let long_chain = longer.admitted@;
+    if short_chain.len() < long_chain.len() {
+        let stop_position = short_chain.len() as int;
+        let next = candidates[stop_position];
+        assert(long_chain[stop_position] == next);
+        assert(admitted_record_passed_every_check(
+            records,
+            long_chain,
+            stop_position,
+            anchor,
+            root_instance,
+            root_checkpoint_txg,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+        ));
+        reveal(admitted_record_passed_every_check);
+        if stop_position > 0 {
+            assert(short_chain[stop_position - 1] == candidates[stop_position - 1]);
+            assert(long_chain[stop_position - 1] == candidates[stop_position - 1]);
+        }
+        assert(previous_in_the_chain(records, short_chain, stop_position) == previous_in_the_chain(
+            records,
+            long_chain,
+            stop_position,
+        ));
+        lemma_the_same_reader_does_not_both_stop_and_admit_a_candidate(
+            records,
+            next,
+            previous_in_the_chain(records, short_chain, stop_position),
+            anchor,
+            root_checkpoint_txg,
+            previous_in_the_chain(records, long_chain, stop_position),
+            anchor,
+            root_checkpoint_txg,
+            root_instance,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            shorter.report.verification_failed,
+        );
+    }
+}
+
+// 推论第 4 步「恢复是确定的函数」：同样的入参、同一个读者，规则容得下的两个结果过了逐条判定的、施加的、交回的根段都相同。
+#[allow(clippy::too_many_arguments, reason = "入参照 replay_follows_the_prefix_rules 原样摊开")]
+proof fn lemma_the_rules_allow_one_outcome<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    chosen_root_source: RootSource,
+    records: Seq<CoreRecord<RootSource>>,
+    in_flight_limit: usize,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+    outcome: CoreOutcome<RootSource>,
+    again: CoreOutcome<RootSource>,
+)
+    requires
+        replay_follows_the_prefix_rules(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            Ok(outcome),
+        ),
+        replay_follows_the_prefix_rules(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            Ok(again),
+        ),
+        the_reader_answers_each_position_one_way(back_chain_broken),
+        the_reader_answers_each_position_one_way(named_units_verified),
+    ensures
+        again.admitted@ == outcome.admitted@,
+        again.applied@ == outcome.applied@,
+        again.root_source_after_replay == outcome.root_source_after_replay,
+{
+    lemma_the_same_inputs_admit_no_longer_chain(
+        root_instance,
+        root_checkpoint_txg,
+        chosen_root_source,
+        records,
+        in_flight_limit,
+        named_unit_verification,
+        back_chain_broken,
+        named_units_verified,
+        outcome,
+        again,
+    );
+    lemma_the_same_inputs_admit_no_longer_chain(
+        root_instance,
+        root_checkpoint_txg,
+        chosen_root_source,
+        records,
+        in_flight_limit,
+        named_unit_verification,
+        back_chain_broken,
+        named_units_verified,
+        again,
+        outcome,
+    );
+    let chain = outcome.admitted@;
+    assert(again.admitted@ =~= chain);
+    let first = outcome.applied@;
+    let second = again.applied@;
+    if first.len() < second.len() {
+        assert(second[second.len() - 1] == chain[second.len() - 1]);
+        assert(!is_last_record_of_its_publish(records[chain[second.len() - 1]]));
+    } else if second.len() < first.len() {
+        assert(first[first.len() - 1] == chain[first.len() - 1]);
+        assert(!is_last_record_of_its_publish(records[chain[first.len() - 1]]));
+    }
+    assert forall|position: int| 0 <= position < first.len() implies first[position]
+        == second[position] by {
+        assert(first[position] == chain[position]);
+        assert(second[position] == chain[position]);
+    }
+    assert(first =~= second);
+}
+
+// 落点（施加的最后一条）把它那个 txg 封住：落点及其之前没有记录在它之上，之后的在它之上与在所选根之上是同一回事，
+// 它那次发布里带末条标志的只有它，所选根之上在它之前（含）的候选恰好是施加的那几条。
+#[allow(clippy::too_many_arguments, reason = "入参照 replay_follows_the_prefix_rules 原样摊开")]
+proof fn lemma_the_landed_record_closes_its_txg<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    chosen_root_source: RootSource,
+    records: Seq<CoreRecord<RootSource>>,
+    in_flight_limit: usize,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+    outcome: CoreOutcome<RootSource>,
+)
+    requires
+        records_ascend_by_instance_and_counter(records),
+        journal_txg_does_not_decrease_with_the_counter_within_an_instance(records),
+        replay_follows_the_prefix_rules(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            Ok(outcome),
+        ),
+        outcome.applied@.len() > 0,
+    ensures
+        ({
+            let applied = outcome.applied@;
+            let landed_record_index = applied.last();
+            let landed_txg = records[landed_record_index].checkpoint_txg;
+            let candidates = candidates_before(
+                records,
+                root_instance,
+                root_checkpoint_txg,
+                records.len() as int,
+            );
+            &&& 0 <= landed_record_index < records.len()
+            &&& records[landed_record_index].instance == root_instance
+            &&& landed_txg.0 > root_checkpoint_txg.0
+            &&& is_last_record_of_its_publish(records[landed_record_index])
+            &&& landed_record_index == candidates[applied.len() - 1]
+            &&& applied.len() <= outcome.admitted@.len() <= candidates.len()
+            &&& candidates_before(records, root_instance, landed_txg, records.len() as int)
+                == candidates.subrange(applied.len() as int, candidates.len() as int)
+            &&& last_record_flag_counters_of_the_chosen_roots_publish(
+                records,
+                root_instance,
+                landed_txg,
+                records.len() as int,
+            ) == seq![records[landed_record_index].counter]
+        }),
+{
+    let candidates = candidates_before(
+        records,
+        root_instance,
+        root_checkpoint_txg,
+        records.len() as int,
+    );
+    let anchor = anchor_of_the_chain(records, root_instance, root_checkpoint_txg);
+    let admitted = outcome.admitted@;
+    let applied = outcome.applied@;
+    let applied_count = applied.len() as int;
+    let landed_record_index = applied.last();
+    let landed_record = records[landed_record_index];
+    let landed_txg = landed_record.checkpoint_txg;
+    lemma_candidates_lie_below_their_end_and_above_the_chosen_root(
+        records,
+        root_instance,
+        root_checkpoint_txg,
+        records.len() as int,
+    );
+    lemma_candidates_ascend_in_slice_order(
+        records,
+        root_instance,
+        root_checkpoint_txg,
+        records.len() as int,
+    );
+    assert(applied[applied_count - 1] == admitted[applied_count - 1]);
+    assert(admitted[applied_count - 1] == candidates[applied_count - 1]);
+    assert(above_the_chosen_root(landed_record, root_instance, root_checkpoint_txg));
+    // 过了逐条判定、带末条标志的每一条都过了读者规则第四格
+    assert forall|position_in_chain: int|
+        0 <= position_in_chain < admitted.len() && is_last_record_of_its_publish(
+            records[#[trigger] admitted[position_in_chain]],
+        ) implies !a_later_record_of_the_same_publish_is_readable(
+        records,
+        admitted[position_in_chain],
+    ) by {
+        assert(admitted_record_passed_every_check(
+            records,
+            admitted,
+            position_in_chain,
+            anchor,
+            root_instance,
+            root_checkpoint_txg,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+        ));
+        reveal(admitted_record_passed_every_check);
+        reveal(passes_every_check_before_the_named_units);
+    }
+    lemma_a_publish_end_that_passed_the_fourth_reader_rule_closes_its_txg(
+        records,
+        landed_record_index,
+    );
+    assert forall|record_index: int|
+        0 <= record_index <= landed_record_index implies !#[trigger] above_the_chosen_root(
+        records[record_index],
+        root_instance,
+        landed_txg,
+    ) by {
+        if records[record_index].instance == root_instance && record_index < landed_record_index {
+            lemma_slice_order_within_an_instance_follows_the_counter(
+                records,
+                record_index,
+                landed_record_index,
+            );
+            assert(is_a_later_record_of_the_same_instance(
+                records,
+                record_index,
+                landed_record_index,
+            ));
+        }
+    }
+    assert forall|record_index: int|
+        landed_record_index < record_index < records.len() implies #[trigger] above_the_chosen_root(
+        records[record_index],
+        root_instance,
+        landed_txg,
+    ) == above_the_chosen_root(records[record_index], root_instance, root_checkpoint_txg) by {
+        if records[record_index].instance == root_instance {
+            assert(records[record_index].checkpoint_txg.0 > landed_txg.0);
+        }
+    }
+    // 所选根之上、切片在落点及其之前的候选恰好 applied_count 个
+    lemma_candidates_before_a_shorter_end_are_a_prefix(
+        records,
+        root_instance,
+        root_checkpoint_txg,
+        landed_record_index + 1,
+        records.len() as int,
+    );
+    let through_the_landed_record = candidates_before(
+        records,
+        root_instance,
+        root_checkpoint_txg,
+        landed_record_index + 1,
+    );
+    let below = through_the_landed_record.len() as int;
+    assert(through_the_landed_record[below - 1] == landed_record_index);
+    assert(candidates[below - 1] == landed_record_index);
+    if below < applied_count {
+        assert(candidates[below - 1] < candidates[applied_count - 1]);
+    } else if below > applied_count {
+        assert(candidates[applied_count - 1] < candidates[below - 1]);
+    }
+    lemma_candidates_above_the_landed_root_are_the_tail(
+        records,
+        root_instance,
+        root_checkpoint_txg,
+        landed_txg,
+        landed_record_index,
+        records.len() as int,
+    );
+    // 落到的那一版的发布里带末条标志的只有落点
+    assert forall|record_index: int|
+        0 <= record_index < records.len() && records[record_index].instance == root_instance
+            && records[record_index].checkpoint_txg == landed_txg
+            && #[trigger] is_last_record_of_its_publish(records[record_index]) implies record_index
+        == landed_record_index by {
+        if record_index < landed_record_index {
+            let position = lemma_a_record_above_the_chosen_root_is_a_candidate(
+                records,
+                root_instance,
+                root_checkpoint_txg,
+                records.len() as int,
+                record_index,
+            );
+            if position >= applied_count - 1 {
+                assert(candidates[applied_count - 1] <= candidates[position]);
+            }
+            assert(admitted[position] == record_index);
+            lemma_a_publish_end_that_passed_the_fourth_reader_rule_closes_its_txg(
+                records,
+                record_index,
+            );
+            assert(records[landed_record_index].instance == records[record_index].instance);
+        } else if record_index > landed_record_index {
+            assert(records[record_index].instance == records[landed_record_index].instance);
+        }
+    }
+    lemma_the_flagged_counters_of_the_landed_publish(
+        records,
+        root_instance,
+        landed_txg,
+        landed_record_index,
+        records.len() as int,
+    );
+}
+
+// 不动点那一读（第二轮判决第二节第 6 行；第三轮 S6 取 Qn）：落到的那一版当所选根再取一次前缀，结局是 replaying_again_from_the_landed_root_stays_put。
+// 前提：I-8.12、同一个读者、所选根之上的记录条数不超在飞上限（入参严格升序担保「同一 (实例代号, 计数器) 只对应一份内容」）。
+#[allow(clippy::too_many_arguments, reason = "入参照 replay_follows_the_prefix_rules 原样摊开")]
+proof fn lemma_replaying_from_the_landed_root_stays_put<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    chosen_root_source: RootSource,
+    records: Seq<CoreRecord<RootSource>>,
+    in_flight_limit: usize,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+    outcome: CoreOutcome<RootSource>,
+    replayed_again: Result<CoreOutcome<RootSource>, CoreFailure>,
+)
+    requires
+        records_ascend_by_instance_and_counter(records),
+        replay_follows_the_prefix_rules(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            Ok(outcome),
+        ),
+        // I-8.12
+        journal_txg_does_not_decrease_with_the_counter_within_an_instance(records),
+        // 同一个读者
+        the_reader_answers_each_position_one_way(back_chain_broken),
+        the_reader_answers_each_position_one_way(named_units_verified),
+        // 所选根之上的记录条数不超在飞上限（D23 已定项 18）
+        records_above_the_chosen_root_fit_the_in_flight_limit(
+            records,
+            root_instance,
+            root_checkpoint_txg,
+            in_flight_limit,
+        ),
+        replay_follows_the_prefix_rules(
+            landed_instance(root_instance, records, outcome.applied@),
+            landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),
+            outcome.root_source_after_replay,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed_again,
+        ),
+    ensures
+        replaying_again_from_the_landed_root_stays_put(
+            landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),
+            outcome.root_source_after_replay,
+            replayed_again,
+        ),
+{
+    let applied = outcome.applied@;
+    if applied.len() == 0 {
+        // 一条没施加：落到的就是所选根，第二次的入参与第一次相同
+        match replayed_again {
+            Ok(again) => {
+                lemma_the_rules_allow_one_outcome(
+                    root_instance,
+                    root_checkpoint_txg,
+                    chosen_root_source,
+                    records,
+                    in_flight_limit,
+                    named_unit_verification,
+                    back_chain_broken,
+                    named_units_verified,
+                    outcome,
+                    again,
+                );
+            },
+            Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {},
+            Err(
+                CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+                    counters: _,
+                },
+            ) => {},
+        }
+    } else {
+        lemma_the_landed_record_closes_its_txg(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            outcome,
+        );
+        let candidates = candidates_before(
+            records,
+            root_instance,
+            root_checkpoint_txg,
+            records.len() as int,
+        );
+        let anchor = anchor_of_the_chain(records, root_instance, root_checkpoint_txg);
+        let admitted = outcome.admitted@;
+        let applied_count = applied.len() as int;
+        let landed_record_index = applied.last();
+        let landed_record = records[landed_record_index];
+        let landed_txg = landed_record.checkpoint_txg;
+        let candidates_again = candidates.subrange(applied_count, candidates.len() as int);
+        let anchor_again = anchor_of_the_chain(records, root_instance, landed_txg);
+        assert(anchor_again == Some(landed_record.counter));
+        match replayed_again {
+            Ok(again) => {
+                let chain_again = again.admitted@;
+                let applied_again = again.applied@;
+                if applied_again.len() > 0 {
+                    // 第二次施加的最后一条带末条标志；它要么在第一次攒着没施加的尾巴里（那里一条都不带末条标志），
+                    // 要么第二次接受了第一次停下的那个候选——同一个读者不会这样
+                    let last_again = applied_again.len() - 1;
+                    assert(applied_again[last_again] == chain_again[last_again]);
+                    assert(chain_again[last_again] == candidates_again[last_again]);
+                    if applied_count + last_again < admitted.len() {
+                        assert(admitted[applied_count + last_again] == candidates[applied_count
+                            + last_again]);
+                    } else {
+                        let position_again = admitted.len() - applied_count;
+                        let next = candidates[admitted.len() as int];
+                        assert(chain_again[position_again] == next);
+                        assert(admitted_record_passed_every_check(
+                            records,
+                            chain_again,
+                            position_again,
+                            anchor_again,
+                            root_instance,
+                            landed_txg,
+                            named_unit_verification,
+                            back_chain_broken,
+                            named_units_verified,
+                        ));
+                        reveal(admitted_record_passed_every_check);
+                        if position_again > 0 {
+                            assert(chain_again[position_again - 1] == candidates[admitted.len()
+                                - 1]);
+                            assert(admitted[admitted.len() - 1] == candidates[admitted.len() - 1]);
+                        } else {
+                            assert(admitted[admitted.len() - 1] == applied[applied_count - 1]);
+                        }
+                        lemma_the_same_reader_does_not_both_stop_and_admit_a_candidate(
+                            records,
+                            next,
+                            previous_in_the_chain(records, admitted, admitted.len() as int),
+                            anchor,
+                            root_checkpoint_txg,
+                            previous_in_the_chain(records, chain_again, position_again),
+                            anchor_again,
+                            landed_txg,
+                            root_instance,
+                            named_unit_verification,
+                            back_chain_broken,
+                            named_units_verified,
+                            outcome.report.verification_failed,
+                        );
+                    }
+                }
+            },
+            Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {},
+            Err(
+                CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+                    counters: _,
+                },
+            ) => {},
+        }
+    }
+}
+
+// 记录级那一读的第一半：同一条记录紧接着施加两次，与施加一次的根相同（I-8.4 的字面）。
+proof fn lemma_applying_a_record_twice_in_a_row_is_applying_it_once<RootSource>(
+    root_source: RootSource,
+    applied_records: Seq<CoreRecord<RootSource>>,
+    position: int,
+)
+    requires
+        0 <= position < applied_records.len(),
+    ensures
+        root_source_after_applying_in_order(
+            root_source,
+            applied_records.insert(position + 1, applied_records[position]),
+        ) == root_source_after_applying_in_order(root_source, applied_records),
+    decreases applied_records.len(),
+{
+    let twice = applied_records.insert(position + 1, applied_records[position]);
+    if position == applied_records.len() - 1 {
+        assert(twice.drop_last() =~= applied_records);
+        assert(twice.last() == applied_records.last());
+    } else {
+        lemma_applying_a_record_twice_in_a_row_is_applying_it_once(
+            root_source,
+            applied_records.drop_last(),
+            position,
+        );
+        assert(twice.drop_last() =~= applied_records.drop_last().insert(
+            position + 1,
+            applied_records[position],
+        ));
+        assert(twice.last() == applied_records.last());
+    }
+}
+
+// replay_core 的 M3 ensures 里三条各由下面一个 broadcast 引理从 replay_follows_the_prefix_rules 推出（replay_core 里 broadcast use），
+// 那三个 spec 标 opaque、只在引理里 reveal：开着时它们的定义进了 replay_core 的整个上下文，主循环要 rlimit 10 才过（5 到 9 都报 rlimit 用尽）。
+// 「落点不早于所选根」那一条开着、不 broadcast：Verus 把 replay_follows_the_prefix_rules 展开就得到，它的引理只给推论调。
+#[allow(clippy::too_many_arguments, reason = "入参照 replay_follows_the_prefix_rules 原样摊开")]
+broadcast proof fn lemma_replay_core_result_replays_an_applied_record_twice_as_once<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    chosen_root_source: RootSource,
+    records: Seq<CoreRecord<RootSource>>,
+    in_flight_limit: usize,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+    replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
+)
+    requires
+        records_ascend_by_instance_and_counter(records),
+        replay_follows_the_prefix_rules(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
+    ensures
+        #[trigger] replaying_an_applied_record_twice_leaves_the_root_as_once(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
+{
+    reveal(replaying_an_applied_record_twice_leaves_the_root_as_once);
+    match replayed {
+        Ok(outcome) => {
+            let applied = outcome.applied@;
+            let applied_records = records_at(records, applied);
+            if applied.len() > 0 {
+                assert(applied_records.last() == records[applied.last()]);
+                assert(applied[applied.len() - 1] == outcome.admitted@[applied.len() - 1]);
+            }
+            assert forall|position: int| 0 <= position < applied_records.len() implies #[trigger] root_source_after_applying_in_order(
+                chosen_root_source,
+                applied_records.insert(position + 1, applied_records[position]),
+            ) == outcome.root_source_after_replay by {
+                lemma_applying_a_record_twice_in_a_row_is_applying_it_once(
+                    chosen_root_source,
+                    applied_records,
+                    position,
+                );
+            }
+            if journal_txg_does_not_decrease_with_the_counter_within_an_instance(records) {
+                assert forall|position: int| 0 <= position < applied.len() implies !above_the_chosen_root(
+                    records[#[trigger] applied[position]],
+                    landed_instance(root_instance, records, applied),
+                    landed_checkpoint_txg(root_checkpoint_txg, records, applied),
+                ) by {
+                    if position < applied.len() - 1 {
+                        assert(applied[position] < applied[applied.len() - 1]);
+                        assert(above_the_chosen_root(
+                            records[applied[position]],
+                            root_instance,
+                            root_checkpoint_txg,
+                        ));
+                        assert(above_the_chosen_root(
+                            records[applied[applied.len() - 1]],
+                            root_instance,
+                            root_checkpoint_txg,
+                        ));
+                        lemma_slice_order_within_an_instance_follows_the_counter(
+                            records,
+                            applied[position],
+                            applied.last(),
+                        );
+                        assert(is_a_later_record_of_the_same_instance(
+                            records,
+                            applied[position],
+                            applied.last(),
+                        ));
+                    }
+                }
+            }
+        },
+        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {},
+        Err(
+            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+                counters: _,
+            },
+        ) => {},
+    }
+}
+
+#[allow(clippy::too_many_arguments, reason = "入参照 replay_follows_the_prefix_rules 原样摊开")]
+broadcast proof fn lemma_replay_core_result_is_a_fixed_point<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    chosen_root_source: RootSource,
+    records: Seq<CoreRecord<RootSource>>,
+    in_flight_limit: usize,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+    replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
+)
+    requires
+        records_ascend_by_instance_and_counter(records),
+        replay_follows_the_prefix_rules(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
+    ensures
+        #[trigger] replaying_from_the_landed_root_is_a_fixed_point(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
+{
+    reveal(replaying_from_the_landed_root_is_a_fixed_point);
+    match replayed {
+        Ok(outcome) => {
+            if journal_txg_does_not_decrease_with_the_counter_within_an_instance(records)
+                && the_reader_answers_each_position_one_way(back_chain_broken)
+                && the_reader_answers_each_position_one_way(named_units_verified)
+                && records_above_the_chosen_root_fit_the_in_flight_limit(
+                records,
+                root_instance,
+                root_checkpoint_txg,
+                in_flight_limit,
+            ) {
+                assert forall|replayed_again: Result<CoreOutcome<RootSource>, CoreFailure>|
+                    #[trigger] replay_follows_the_prefix_rules(
+                        landed_instance(root_instance, records, outcome.applied@),
+                        landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),
+                        outcome.root_source_after_replay,
+                        records,
+                        in_flight_limit,
+                        named_unit_verification,
+                        back_chain_broken,
+                        named_units_verified,
+                        replayed_again,
+                    ) implies replaying_again_from_the_landed_root_stays_put(
+                    landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),
+                    outcome.root_source_after_replay,
+                    replayed_again,
+                ) by {
+                    lemma_replaying_from_the_landed_root_stays_put(
+                        root_instance,
+                        root_checkpoint_txg,
+                        chosen_root_source,
+                        records,
+                        in_flight_limit,
+                        named_unit_verification,
+                        back_chain_broken,
+                        named_units_verified,
+                        outcome,
+                        replayed_again,
+                    );
+                }
+            }
+        },
+        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {},
+        Err(
+            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+                counters: _,
+            },
+        ) => {},
+    }
+}
+
+#[allow(clippy::too_many_arguments, reason = "入参照 replay_follows_the_prefix_rules 原样摊开")]
+proof fn lemma_replay_core_result_lands_no_earlier_than_the_chosen_root<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    chosen_root_source: RootSource,
+    records: Seq<CoreRecord<RootSource>>,
+    in_flight_limit: usize,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+    replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
+)
+    requires
+        replay_follows_the_prefix_rules(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
+    ensures
+        replay_lands_no_earlier_than_the_chosen_root(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
+{
+    match replayed {
+        Ok(outcome) => {
+            let applied = outcome.applied@;
+            if applied.len() > 0 {
+                assert(above_the_chosen_root(
+                    records[applied[applied.len() - 1]],
+                    root_instance,
+                    root_checkpoint_txg,
+                ));
+            }
+        },
+        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {},
+        Err(
+            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+                counters: _,
+            },
+        ) => {},
+    }
+}
+
+#[allow(clippy::too_many_arguments, reason = "入参照 replay_follows_the_prefix_rules 原样摊开")]
+broadcast proof fn lemma_replay_core_result_is_the_only_one_the_rules_allow<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    chosen_root_source: RootSource,
+    records: Seq<CoreRecord<RootSource>>,
+    in_flight_limit: usize,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+    replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
+)
+    requires
+        replay_follows_the_prefix_rules(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
+    ensures
+        #[trigger] replay_outcome_is_the_only_one_the_rules_allow(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
+{
+    reveal(replay_outcome_is_the_only_one_the_rules_allow);
+    match replayed {
+        Ok(outcome) => {
+            if the_reader_answers_each_position_one_way(back_chain_broken)
+                && the_reader_answers_each_position_one_way(named_units_verified) {
+                assert forall|replayed_again: Result<CoreOutcome<RootSource>, CoreFailure>|
+                    #[trigger] replay_follows_the_prefix_rules(
+                        root_instance,
+                        root_checkpoint_txg,
+                        chosen_root_source,
+                        records,
+                        in_flight_limit,
+                        named_unit_verification,
+                        back_chain_broken,
+                        named_units_verified,
+                        replayed_again,
+                    ) implies (replayed_again matches Ok(again) && again.admitted@
+                    == outcome.admitted@ && again.applied@ == outcome.applied@
+                    && again.root_source_after_replay == outcome.root_source_after_replay) by {
+                    match replayed_again {
+                        Ok(again) => {
+                            lemma_the_rules_allow_one_outcome(
+                                root_instance,
+                                root_checkpoint_txg,
+                                chosen_root_source,
+                                records,
+                                in_flight_limit,
+                                named_unit_verification,
+                                back_chain_broken,
+                                named_units_verified,
+                                outcome,
+                                again,
+                            );
+                        },
+                        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {},
+                        Err(
+                            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+                                counters: _,
+                            },
+                        ) => {},
+                    }
+                }
+            }
+        },
+        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {},
+        Err(
+            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+                counters: _,
+            },
+        ) => {},
+    }
+}
+
+// M3 单调（推论；第二轮判决第二节第 5 行，第三轮 S5 版本序取 (txg, 实例代号)）：同一次运行里两个崩溃状态 s ⊆ u，
+// u 上恢复落到的那一版不早于 s 上的。核里证的只有两步：落点不早于所选根、恢复是确定的函数（上面两条 ensures 的引理）；
+// 两条前提是择根、写侧的条款，核判不了，照攻方 research/prompts/m3-formal-r2-opus-output.md 3.2 推的写成 requires：
+// 所选根取根环里最新的有效根 ⇒ u 的所选根不早于 s 的（3.2 第 1 步）；s 施加了一次发布时，串行提交与 D16（发布语义）已定项 7 的持久顺序
+// ⇒ u 含那一版的根槽（u 的所选根不早于它），否则 u 与 s 的所选根、记录表、读者都相同（3.2 第 2、4 步）。开并行提交或把域放宽到带故障的状态时重开。
+#[allow(clippy::too_many_arguments, reason = "两次恢复各一套入参，照 replay_follows_the_prefix_rules 原样摊开")]
+pub proof fn corollary_the_landed_version_does_not_go_back<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    in_flight_limit: usize,
+    named_unit_verification: NamedUnitVerification,
+    smaller_root_instance: PrefixKernelInstanceGeneration,
+    smaller_root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    smaller_chosen_root_source: RootSource,
+    smaller_records: Seq<CoreRecord<RootSource>>,
+    smaller_back_chain_broken: BackChain,
+    smaller_named_units_verified: NamedUnits,
+    smaller_outcome: CoreOutcome<RootSource>,
+    larger_root_instance: PrefixKernelInstanceGeneration,
+    larger_root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    larger_chosen_root_source: RootSource,
+    larger_records: Seq<CoreRecord<RootSource>>,
+    larger_back_chain_broken: BackChain,
+    larger_named_units_verified: NamedUnits,
+    larger_replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
+)
+    requires
+        replay_follows_the_prefix_rules(
+            smaller_root_instance,
+            smaller_root_checkpoint_txg,
+            smaller_chosen_root_source,
+            smaller_records,
+            in_flight_limit,
+            named_unit_verification,
+            smaller_back_chain_broken,
+            smaller_named_units_verified,
+            Ok(smaller_outcome),
+        ),
+        replay_follows_the_prefix_rules(
+            larger_root_instance,
+            larger_root_checkpoint_txg,
+            larger_chosen_root_source,
+            larger_records,
+            in_flight_limit,
+            named_unit_verification,
+            larger_back_chain_broken,
+            larger_named_units_verified,
+            larger_replayed,
+        ),
+        // 择根：所选根取根环里最新的有效根（3.2 第 1 步）
+        version_is_not_later_than(
+            smaller_root_checkpoint_txg,
+            smaller_root_instance,
+            larger_root_checkpoint_txg,
+            larger_root_instance,
+        ),
+        // 串行提交与持久顺序（3.2 第 2、4 步）
+        smaller_outcome.applied@.len() > 0 ==> version_is_not_later_than(
+            landed_checkpoint_txg(
+                smaller_root_checkpoint_txg,
+                smaller_records,
+                smaller_outcome.applied@,
+            ),
+            landed_instance(smaller_root_instance, smaller_records, smaller_outcome.applied@),
+            larger_root_checkpoint_txg,
+            larger_root_instance,
+        ) || (larger_root_instance == smaller_root_instance && larger_root_checkpoint_txg
+            == smaller_root_checkpoint_txg && larger_chosen_root_source
+            == smaller_chosen_root_source && larger_records == smaller_records
+            && larger_back_chain_broken == smaller_back_chain_broken && larger_named_units_verified
+            == smaller_named_units_verified),
+        // 同一个读者（只在 u 与 s 相同那一支用到）
+        the_reader_answers_each_position_one_way(smaller_back_chain_broken),
+        the_reader_answers_each_position_one_way(smaller_named_units_verified),
+    ensures
+        larger_replayed matches Ok(larger_outcome) ==> version_is_not_later_than(
+            landed_checkpoint_txg(
+                smaller_root_checkpoint_txg,
+                smaller_records,
+                smaller_outcome.applied@,
+            ),
+            landed_instance(smaller_root_instance, smaller_records, smaller_outcome.applied@),
+            landed_checkpoint_txg(larger_root_checkpoint_txg, larger_records, larger_outcome.applied@),
+            landed_instance(larger_root_instance, larger_records, larger_outcome.applied@),
+        ),
+{
+    lemma_replay_core_result_lands_no_earlier_than_the_chosen_root(
+        smaller_root_instance,
+        smaller_root_checkpoint_txg,
+        smaller_chosen_root_source,
+        smaller_records,
+        in_flight_limit,
+        named_unit_verification,
+        smaller_back_chain_broken,
+        smaller_named_units_verified,
+        Ok(smaller_outcome),
+    );
+    lemma_replay_core_result_lands_no_earlier_than_the_chosen_root(
+        larger_root_instance,
+        larger_root_checkpoint_txg,
+        larger_chosen_root_source,
+        larger_records,
+        in_flight_limit,
+        named_unit_verification,
+        larger_back_chain_broken,
+        larger_named_units_verified,
+        larger_replayed,
+    );
+    match larger_replayed {
+        Ok(larger_outcome) => {
+            if smaller_outcome.applied@.len() > 0 && !version_is_not_later_than(
+                landed_checkpoint_txg(
+                    smaller_root_checkpoint_txg,
+                    smaller_records,
+                    smaller_outcome.applied@,
+                ),
+                landed_instance(smaller_root_instance, smaller_records, smaller_outcome.applied@),
+                larger_root_checkpoint_txg,
+                larger_root_instance,
+            ) {
+                lemma_the_rules_allow_one_outcome(
+                    smaller_root_instance,
+                    smaller_root_checkpoint_txg,
+                    smaller_chosen_root_source,
+                    smaller_records,
+                    in_flight_limit,
+                    named_unit_verification,
+                    smaller_back_chain_broken,
+                    smaller_named_units_verified,
+                    smaller_outcome,
+                    larger_outcome,
+                );
+            }
+        },
+        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {},
+        Err(
+            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+                counters: _,
+            },
+        ) => {},
+    }
+}
+
 // ===== 纯函数核 =====
 // 记录标志位 0（已定项 14 第五条、已定项 17）。
 fn record_ends_its_publish<RootSource>(record: &CoreRecord<RootSource>) -> (is_last_record: bool)
@@ -1529,7 +3239,60 @@
             named_units_verified,
             replayed,
         ),
+        // M3 幂等·记录级（第二轮判决第二节第 6 行「记录级」，I-8.4 的字面）
+        replaying_an_applied_record_twice_leaves_the_root_as_once(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records@,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
+        // M3 幂等·不动点（第二轮判决第二节第 6 行「不动点」；第三轮 S6 取 Qn）
+        replaying_from_the_landed_root_is_a_fixed_point(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records@,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
+        // M3 单调推论的 J1 那一步（第二轮判决第二节第 5 行；第三轮 S5 版本序 (txg, 实例代号)）
+        replay_lands_no_earlier_than_the_chosen_root(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records@,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
+        // M3 单调推论第 4 步「恢复是确定的函数」（第二轮判决第二节第 5 行）
+        replay_outcome_is_the_only_one_the_rules_allow(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records@,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
 {
+    broadcast use {
+        lemma_replay_core_result_replays_an_applied_record_twice_as_once,
+        lemma_replay_core_result_is_a_fixed_point,
+        lemma_replay_core_result_is_the_only_one_the_rules_allow,
+    };
     let mut report = CoreReport {
         above_water: 0,
         prefix_applied: 0,
diff --git a/crates/singlefs-replay-prefix/verus-mutations.tsv b/crates/singlefs-replay-prefix/verus-mutations.tsv
--- a/crates/singlefs-replay-prefix/verus-mutations.tsv
+++ b/crates/singlefs-replay-prefix/verus-mutations.tsv
@@ -16,8 +16,8 @@
 ensures·counter_of_the_last_record_the_root_covers 交回的锚点计数器小于 u64 最大值减一	crates/singlefs-replay-prefix/src/lib.rs	        anchor matches Ok(Some(counter)) ==> counter.0 < u64::MAX,	        anchor matches Ok(Some(counter)) ==> counter.0 < u64::MAX - 1,	proof_errors=1
 ensures·counter_of_the_last_record_the_root_covers 交回的锚点改成不等于 spec 的锚点	crates/singlefs-replay-prefix/src/lib.rs	                Ok(found) => flagged.len() < 2 && found == anchor_of_the_chain(	                Ok(found) => flagged.len() < 2 && found != anchor_of_the_chain(	proof_errors=3
 ensures·replay_core 整个结论取反	crates/singlefs-replay-prefix/src/lib.rs	    ensures\n        replay_follows_the_prefix_rules(	    ensures\n        !replay_follows_the_prefix_rules(	proof_errors=2
-ensures·replay_core 报「所选根 txg 没有后继」那一臂改成恒假	crates/singlefs-replay-prefix/src/lib.rs	        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => flagged.len() < 2\n            && root_checkpoint_txg.0 == u64::MAX,	        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => false,	proof_errors=1
-ensures·replay_core 施加了的每一条都改成要求在所选根之下	crates/singlefs-replay-prefix/src/lib.rs	    &&& forall|position: int|\n        0 <= position < applied.len() ==> above_the_chosen_root(	    &&& forall|position: int|\n        0 <= position < applied.len() ==> !above_the_chosen_root(	proof_errors=1
+ensures·replay_core 报「所选根 txg 没有后继」那一臂改成恒假	crates/singlefs-replay-prefix/src/lib.rs	        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => flagged.len() < 2\n            && root_checkpoint_txg.0 == u64::MAX,	        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => false,	proof_errors=2
+ensures·replay_core 施加了的每一条都改成要求在所选根之下	crates/singlefs-replay-prefix/src/lib.rs	    &&& forall|position: int|\n        0 <= position < applied.len() ==> above_the_chosen_root(	    &&& forall|position: int|\n        0 <= position < applied.len() ==> !above_the_chosen_root(	proof_errors=4
 ensures·冒烟：两条记录的整次发布改成断言不是拼接	crates/singlefs-replay-prefix/src/lib.rs	    ensures\n        is_concatenation_of_whole_publishes(seq![first, second]),	    ensures\n        !is_concatenation_of_whole_publishes(seq![first, second]),	proof_errors=1
 ensures·冒烟：一条不带末条标志的记录改成断言是拼接	crates/singlefs-replay-prefix/src/lib.rs	    ensures\n        !is_concatenation_of_whole_publishes(seq![only]),	    ensures\n        is_concatenation_of_whole_publishes(seq![only]),	proof_errors=1
 ensures·is_above_the_chosen_root 交回的判定取反	crates/singlefs-replay-prefix/src/lib.rs	        is_above == above_the_chosen_root(*record, root_instance, root_checkpoint_txg),	        is_above == !above_the_chosen_root(*record, root_instance, root_checkpoint_txg),	proof_errors=4
@@ -30,7 +30,7 @@
 ensures·judge_the_candidate 断在点名单元时改成点名单元之前就有一判不过	crates/singlefs-replay-prefix/src/lib.rs	                CandidateVerdict::NamedUnitsReportedUnverified => passes_every_check_before_the_named_units(	                CandidateVerdict::NamedUnitsReportedUnverified => !passes_every_check_before_the_named_units(	proof_errors=2
 ensures·引理：候选个数改成严格小于终点	crates/singlefs-replay-prefix/src/lib.rs	            &&& candidates.len() <= end\n	            &&& candidates.len() < end\n	proof_errors=1
 ensures·引理：短终点的候选个数改成严格少于长终点的	crates/singlefs-replay-prefix/src/lib.rs	            &&& fewer.len() <= more.len()	            &&& fewer.len() < more.len()	proof_errors=1
-ensures·引理：同一实例里切片次序改成与计数器次序相反	crates/singlefs-replay-prefix/src/lib.rs	        first < second <==> records[first].counter.0 < records[second].counter.0,	        first < second <==> records[first].counter.0 > records[second].counter.0,	proof_errors=4
+ensures·引理：同一实例里切片次序改成与计数器次序相反	crates/singlefs-replay-prefix/src/lib.rs	        first < second <==> records[first].counter.0 < records[second].counter.0,	        first < second <==> records[first].counter.0 > records[second].counter.0,	proof_errors=7
 ensures·引理：接受下一个候选之后过了逐条判定的多算一条	crates/singlefs-replay-prefix/src/lib.rs	            0 <= position_in_chain < admitted_before.len() + 1 ==> #[trigger] admitted_record_passed_every_check(	            0 <= position_in_chain < admitted_before.len() + 2 ==> #[trigger] admitted_record_passed_every_check(	proof_errors=1
 ensures·引理：接受下一个候选之后攒着的改成不是这一串的尾巴	crates/singlefs-replay-prefix/src/lib.rs	        as_ints(open_before.push(index)) == admitted_before.push(index as int).subrange(	        as_ints(open_before.push(index)) != admitted_before.push(index as int).subrange(	proof_errors=3
 ensures·引理：接受了带末条标志的候选之后攒着的改成不是一次整的发布	crates/singlefs-replay-prefix/src/lib.rs	            &&& is_last_record_of_its_publish(records[index as int]) ==> is_whole_publish(after)	            &&& is_last_record_of_its_publish(records[index as int]) ==> !is_whole_publish(after)	proof_errors=1
@@ -75,33 +75,73 @@
 循环不变式·主循环攒着的改成不是这一串在已施加之后的尾巴	crates/singlefs-replay-prefix/src/lib.rs	            as_ints(records_of_the_open_publish@) == admitted.subrange(	            as_ints(records_of_the_open_publish@) != admitted.subrange(	proof_errors=3
 循环不变式·主循环最大事务号改成不是施加的里最大的	crates/singlefs-replay-prefix/src/lib.rs	            maximum_transaction_is_the_largest_applied(\n                records@,\n                applied,	            !maximum_transaction_is_the_largest_applied(\n                records@,\n                applied,	proof_errors=4
 循环不变式·主循环已施加的下标上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	                0 <= position < applied.len() ==> 0 <= #[trigger] applied[position] < index,	                0 <= position < applied.len() ==> 0 <= #[trigger] applied[position] + 1 < index,	proof_errors=1
-循环不变式·主循环已施加的下标改成严格递减	crates/singlefs-replay-prefix/src/lib.rs	                0 <= earlier < later < applied.len() ==> applied[earlier] < applied[later],	                0 <= earlier < later < applied.len() ==> applied[earlier] > applied[later],	proof_errors=2
+循环不变式·主循环已施加的下标改成严格递减	crates/singlefs-replay-prefix/src/lib.rs	                0 <= earlier < later < applied.len() ==> applied[earlier] < applied[later],	                0 <= earlier < later < applied.len() ==> applied[earlier] > applied[later],	proof_errors=3
 循环不变式·主循环已施加的每一条改成要求在所选根之下	crates/singlefs-replay-prefix/src/lib.rs	                0 <= position < applied.len() ==> above_the_chosen_root(\n                    records@[#[trigger] applied[position]],	                0 <= position < applied.len() ==> !above_the_chosen_root(\n                    records@[#[trigger] applied[position]],	proof_errors=1
 循环不变式·主循环已施加的改成不是整次发布的拼接	crates/singlefs-replay-prefix/src/lib.rs	            is_concatenation_of_whole_publishes(records_at(records@, applied)),	            !is_concatenation_of_whole_publishes(records_at(records@, applied)),	proof_errors=4
 循环不变式·主循环施加条数多算一条	crates/singlefs-replay-prefix/src/lib.rs	            report.prefix_applied == applied.len(),	            report.prefix_applied == applied.len() + 1,	proof_errors=3
 循环不变式·主循环交回的根从哪来两臂对调	crates/singlefs-replay-prefix/src/lib.rs	            root_source_after_replay == if applied.len() == 0 {	            root_source_after_replay == if applied.len() != 0 {	proof_errors=3
 循环不变式·主循环攒着的下标上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	                0 <= position < records_of_the_open_publish@.len()\n                    ==> (#[trigger] records_of_the_open_publish@[position]) < index,	                0 <= position < records_of_the_open_publish@.len()\n                    ==> (#[trigger] records_of_the_open_publish@[position]) + 1 < index,	proof_errors=1
 循环不变式·主循环攒着的下标改成严格递减	crates/singlefs-replay-prefix/src/lib.rs	                    ==> records_of_the_open_publish@[earlier] < records_of_the_open_publish@[later],	                    ==> records_of_the_open_publish@[earlier] > records_of_the_open_publish@[later],	proof_errors=2
-循环不变式·主循环已施加的改成都在攒着的之后	crates/singlefs-replay-prefix/src/lib.rs	                    < records_of_the_open_publish@.len() ==> applied[applied_position]\n                    < records_of_the_open_publish@[open_position],	                    < records_of_the_open_publish@.len() ==> applied[applied_position]\n                    > records_of_the_open_publish@[open_position],	proof_errors=1
+循环不变式·主循环已施加的改成都在攒着的之后	crates/singlefs-replay-prefix/src/lib.rs	                    < records_of_the_open_publish@.len() ==> applied[applied_position]\n                    < records_of_the_open_publish@[open_position],	                    < records_of_the_open_publish@.len() ==> applied[applied_position]\n                    > records_of_the_open_publish@[open_position],	proof_errors=2
 循环不变式·主循环攒着的每一条改成要求在所选根之下	crates/singlefs-replay-prefix/src/lib.rs	                0 <= position < records_of_the_open_publish@.len() ==> above_the_chosen_root(	                0 <= position < records_of_the_open_publish@.len() ==> !above_the_chosen_root(	proof_errors=1
 循环不变式·主循环攒着的改成不是未完的一次发布	crates/singlefs-replay-prefix/src/lib.rs	            is_open_publish(records_at(records@, as_ints(records_of_the_open_publish@))),	            !is_open_publish(records_at(records@, as_ints(records_of_the_open_publish@))),	proof_errors=3
 循环不变式·主循环已施加加攒着的条数改成严格小于下标	crates/singlefs-replay-prefix/src/lib.rs	            applied.len() + records_of_the_open_publish@.len() <= index,	            applied.len() + records_of_the_open_publish@.len() < index,	proof_errors=1
 循环不变式·主循环没接上链首时攒着为空改成接上之后攒着为空（「锚点读不出时链首序号为 1」那一判冗余的证明跟着断）	crates/singlefs-replay-prefix/src/lib.rs	            expected_next is None ==> records_of_the_open_publish@.len() == 0,	            expected_next is Some ==> records_of_the_open_publish@.len() == 0,	proof_errors=1
 循环出口·主循环出口改成下标没到末尾	crates/singlefs-replay-prefix/src/lib.rs	            index <= records.len(),\n            index == records.len() ==> report.verification_failed == 0,	            index < records.len(),\n            index == records.len() ==> report.verification_failed == 0,	proof_errors=1
-循环出口·主循环走完时校验不过条数改成 1	crates/singlefs-replay-prefix/src/lib.rs	            index == records.len() ==> report.verification_failed == 0,	            index == records.len() ==> report.verification_failed == 1,	proof_errors=2
+循环出口·主循环走完时校验不过条数改成 1	crates/singlefs-replay-prefix/src/lib.rs	            index == records.len() ==> report.verification_failed == 0,	            index == records.len() ==> report.verification_failed == 1,	proof_errors=3
 循环出口·主循环停下时那一条改成不在所选根之上	crates/singlefs-replay-prefix/src/lib.rs	            index < records.len() ==> above_the_chosen_root(\n                records@[index as int],	            index < records.len() ==> !above_the_chosen_root(\n                records@[index as int],	proof_errors=3
-循环出口·主循环停下的理由改成落在下一个下标上	crates/singlefs-replay-prefix/src/lib.rs	                admitted,\n                index as int,\n                in_flight_limit,	                admitted,\n                index as int + 1,\n                in_flight_limit,	proof_errors=3
+循环出口·主循环停下的理由改成落在下一个下标上	crates/singlefs-replay-prefix/src/lib.rs	                admitted,\n                index as int,\n                in_flight_limit,	                admitted,\n                index as int + 1,\n                in_flight_limit,	proof_errors=4
 改判定·反向链回调交回「断」也不断链（I-8.6、已定项 8；攻方 V01）	crates/singlefs-replay-prefix/src/lib.rs	    if back_chain_broken(index) {	    if false {	proof_errors=2
 改判定·上一条没带提交标记而这一条换了事务号照接（已定项 7；攻方 V02）	crates/singlefs-replay-prefix/src/lib.rs	        if !previous_record_of_the_open_publish.is_commit\n            && record.transaction.0 != previous_record_of_the_open_publish.transaction.0 {	        if false {	proof_errors=2
 改判定·末条不带提交标记照施加（已定项 14 五条口径第三条；攻方 V03）	crates/singlefs-replay-prefix/src/lib.rs	    if !record.is_commit && record_ends_its_publish(record) {	    if false {	proof_errors=2
 改判定·末条之后同一次发布还有记录照施加（已定项 4 读者规则第四格、C540；攻方 V04）	crates/singlefs-replay-prefix/src/lib.rs	    if record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(	    if false && readable_record_of_the_same_publish_follows(	proof_errors=2
 改判定·点名单元不验（已定项 14 五条口径第四条；攻方 V05）	crates/singlefs-replay-prefix/src/lib.rs	        NamedUnitVerification::Verify => named_units_verified(index),	        NamedUnitVerification::Verify => true,	proof_errors=1
-改判定·在飞上限不看（已定项 18；攻方 V06）	crates/singlefs-replay-prefix/src/lib.rs	        if records_taken_toward_the_in_flight_limit >= in_flight_limit {	        if false {	proof_errors=3
+改判定·在飞上限不看（已定项 18；攻方 V06）	crates/singlefs-replay-prefix/src/lib.rs	        if records_taken_toward_the_in_flight_limit >= in_flight_limit {	        if false {	proof_errors=2
 改判定·第一条水位之上的记录就停、一条都不施加（攻方 V07）	crates/singlefs-replay-prefix/src/lib.rs	        if records_taken_toward_the_in_flight_limit >= in_flight_limit {	        if true {	proof_errors=1
 改判定·锚点读不出时链首不看 txg（已定项 14 注 1；三方第一轮 K4-a；攻方 V08）	crates/singlefs-replay-prefix/src/lib.rs	            if record.checkpoint_txg.0 != chain_start_txg_without_anchor.0\n                || record.ordinal_within_publish.0 != 1 {	            if false\n                || record.ordinal_within_publish.0 != 1 {	proof_errors=1
-改判定·每次都报所选根 txg 没有后继（攻方 V09）	crates/singlefs-replay-prefix/src/lib.rs	    if root_checkpoint_txg.0 == u64::MAX {	    if true {	proof_errors=1
+改判定·每次都报所选根 txg 没有后继（攻方 V09）	crates/singlefs-replay-prefix/src/lib.rs	    if root_checkpoint_txg.0 == u64::MAX {	    if true {	proof_errors=2
 改判定·最大事务号取最小（已定项 14 注 4 的 W 的输入；攻方 V10）	crates/singlefs-replay-prefix/src/lib.rs	        if transaction.0 > maximum.0 {	        if transaction.0 < maximum.0 {	proof_errors=2
 改判定·水位之上条数不数（报告字段；攻方 V11）	crates/singlefs-replay-prefix/src/lib.rs	            report.above_water += 1;	            report.above_water += 0;	proof_errors=1
 改判定·锚点筛选不看 txg（同实例任一带末条标志的都算；已定项 14 注 1；攻方 V12、K19）	crates/singlefs-replay-prefix/src/lib.rs	        if record.instance.0 == root_instance.0 && record.checkpoint_txg.0 == root_checkpoint_txg.0\n            && record_ends_its_publish(record) {	        if record.instance.0 == root_instance.0\n            && record_ends_its_publish(record) {	proof_errors=1
 改判定·删掉 I-8.3 那一判（jsn 严格连续不判；已定项 22；第二轮判决第二节第 2 条）	crates/singlefs-replay-prefix/src/lib.rs	            if record.instance.0 != expected_key.0.0 || record.counter.0 != expected_key.1.0 {	            if false {	proof_errors=2
 改判定·末条之后同一次发布的记录往后找时跳过紧挨着的下一条（已定项 4 读者规则第四格、C540；代码轮第三轮攻方 A02）	crates/singlefs-replay-prefix/src/lib.rs	        if later.instance.0 == record.instance.0 {	        if later.instance.0 == record.instance.0 && later.counter.0 != record.counter.0 + 1 {	proof_errors=2
+# M3 那一节（变异名以「M3·」起头，实现员 7-2 加）：replay_core 的四条 M3 ensures 各取反一行、三个 broadcast 引理各从 broadcast use 里拿掉一行、M3 的 spec 各改坏一处、每条前提（I-8.12、同一个读者、在飞上限、推论的择根与串行提交）各去掉一行、新引理的结论各改坏一行，最后两行「改判定」只改可执行判定（M3 由 M2 推出，这两行 M2 那条 ensures 一起红）。
+M3·ensures·replay_core 记录级幂等那一条取反	crates/singlefs-replay-prefix/src/lib.rs	        // M3 幂等·记录级（第二轮判决第二节第 6 行「记录级」，I-8.4 的字面）\n        replaying_an_applied_record_twice_leaves_the_root_as_once(	        // M3 幂等·记录级（第二轮判决第二节第 6 行「记录级」，I-8.4 的字面）\n        !replaying_an_applied_record_twice_leaves_the_root_as_once(	proof_errors=2
+M3·ensures·replay_core 不动点那一条取反	crates/singlefs-replay-prefix/src/lib.rs	        // M3 幂等·不动点（第二轮判决第二节第 6 行「不动点」；第三轮 S6 取 Qn）\n        replaying_from_the_landed_root_is_a_fixed_point(	        // M3 幂等·不动点（第二轮判决第二节第 6 行「不动点」；第三轮 S6 取 Qn）\n        !replaying_from_the_landed_root_is_a_fixed_point(	proof_errors=2
+M3·ensures·replay_core 落点不早于所选根那一条取反	crates/singlefs-replay-prefix/src/lib.rs	        // M3 单调推论的 J1 那一步（第二轮判决第二节第 5 行；第三轮 S5 版本序 (txg, 实例代号)）\n        replay_lands_no_earlier_than_the_chosen_root(	        // M3 单调推论的 J1 那一步（第二轮判决第二节第 5 行；第三轮 S5 版本序 (txg, 实例代号)）\n        !replay_lands_no_earlier_than_the_chosen_root(	proof_errors=2
+M3·ensures·replay_core 规则只容得下一个结果那一条取反	crates/singlefs-replay-prefix/src/lib.rs	        // M3 单调推论第 4 步「恢复是确定的函数」（第二轮判决第二节第 5 行）\n        replay_outcome_is_the_only_one_the_rules_allow(	        // M3 单调推论第 4 步「恢复是确定的函数」（第二轮判决第二节第 5 行）\n        !replay_outcome_is_the_only_one_the_rules_allow(	proof_errors=2
+M3·broadcast·replay_core 不用记录级那一个 broadcast 引理	crates/singlefs-replay-prefix/src/lib.rs	        lemma_replay_core_result_replays_an_applied_record_twice_as_once,\n        lemma_replay_core_result_is_a_fixed_point,	        lemma_replay_core_result_is_a_fixed_point,	proof_errors=2
+M3·broadcast·replay_core 不用不动点那一个 broadcast 引理	crates/singlefs-replay-prefix/src/lib.rs	        lemma_replay_core_result_is_a_fixed_point,\n        lemma_replay_core_result_is_the_only_one_the_rules_allow,	        lemma_replay_core_result_is_the_only_one_the_rules_allow,	proof_errors=2
+M3·broadcast·replay_core 不用规则只容得下一个结果那一个 broadcast 引理	crates/singlefs-replay-prefix/src/lib.rs	        lemma_replay_core_result_is_the_only_one_the_rules_allow,\n    };	    };	proof_errors=2
+M3·spec·记录级：交回的根改成少施加最后一条的结果	crates/singlefs-replay-prefix/src/lib.rs	        &&& outcome.root_source_after_replay == root_source_after_applying_in_order(\n            chosen_root_source,\n            applied_records,\n        )	        &&& outcome.root_source_after_replay == root_source_after_applying_in_order(\n            chosen_root_source,\n            applied_records.drop_last(),\n        )	proof_errors=1
+M3·spec·记录级：紧接着再施加一次之后的根改成所选根	crates/singlefs-replay-prefix/src/lib.rs	                applied_records.insert(position + 1, applied_records[position]),\n            ) == outcome.root_source_after_replay\n        &&& journal_txg_does_not_decrease_with_the_counter_within_an_instance(records)	                applied_records.insert(position + 1, applied_records[position]),\n            ) == chosen_root_source\n        &&& journal_txg_does_not_decrease_with_the_counter_within_an_instance(records)	proof_errors=1
+M3·前提·记录级第二半去掉 I-8.12	crates/singlefs-replay-prefix/src/lib.rs	        &&& journal_txg_does_not_decrease_with_the_counter_within_an_instance(records)\n            ==> forall|position: int|	        &&& true\n            ==> forall|position: int|	proof_errors=1
+M3·前提·不动点去掉 I-8.12	crates/singlefs-replay-prefix/src/lib.rs	    replayed matches Ok(outcome) ==> (journal_txg_does_not_decrease_with_the_counter_within_an_instance(\n        records,\n    ) && the_reader_answers_each_position_one_way(back_chain_broken)	    replayed matches Ok(outcome) ==> (true && the_reader_answers_each_position_one_way(back_chain_broken)	proof_errors=1
+M3·前提·不动点去掉同一个读者（反向链回调）	crates/singlefs-replay-prefix/src/lib.rs	    ) && the_reader_answers_each_position_one_way(back_chain_broken)\n        && the_reader_answers_each_position_one_way(named_units_verified)\n        && records_above_the_chosen_root_fit_the_in_flight_limit(	    ) && true\n        && the_reader_answers_each_position_one_way(named_units_verified)\n        && records_above_the_chosen_root_fit_the_in_flight_limit(	proof_errors=1
+M3·前提·不动点去掉同一个读者（点名单元回调）	crates/singlefs-replay-prefix/src/lib.rs	        && the_reader_answers_each_position_one_way(named_units_verified)\n        && records_above_the_chosen_root_fit_the_in_flight_limit(	        && true\n        && records_above_the_chosen_root_fit_the_in_flight_limit(	proof_errors=1
+M3·前提·不动点去掉所选根之上的记录不超在飞上限	crates/singlefs-replay-prefix/src/lib.rs	        && records_above_the_chosen_root_fit_the_in_flight_limit(\n        records,\n        root_instance,\n        root_checkpoint_txg,\n        in_flight_limit,\n    ) ==> forall|replayed_again: Result<CoreOutcome<RootSource>, CoreFailure>|	        && true ==> forall|replayed_again: Result<CoreOutcome<RootSource>, CoreFailure>|	proof_errors=1
+M3·spec·不动点 S6：txg 顶格报错不算相等（Qn 改成前提读法 P 漏判的那一侧）	crates/singlefs-replay-prefix/src/lib.rs	        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => landed_txg.0 == u64::MAX,	        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => false,	proof_errors=1
+M3·spec·不动点：第二次改成要施加一条	crates/singlefs-replay-prefix/src/lib.rs	        Ok(again) => again.applied@.len() == 0 && again.root_source_after_replay\n            == landed_root_source,	        Ok(again) => again.applied@.len() == 1 && again.root_source_after_replay\n            == landed_root_source,	proof_errors=1
+M3·spec·S5 版本序：实例代号破平局改成严格小于	crates/singlefs-replay-prefix/src/lib.rs	        == later_checkpoint_txg.0 && earlier_instance.0 <= later_instance.0)	        == later_checkpoint_txg.0 && earlier_instance.0 < later_instance.0)	proof_errors=2
+M3·spec·落点不早于所选根改成所选根不早于落点	crates/singlefs-replay-prefix/src/lib.rs	    replayed matches Ok(outcome) ==> version_is_not_later_than(\n        root_checkpoint_txg,\n        root_instance,\n        landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),\n        landed_instance(root_instance, records, outcome.applied@),\n    )	    replayed matches Ok(outcome) ==> version_is_not_later_than(\n        landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),\n        landed_instance(root_instance, records, outcome.applied@),\n        root_checkpoint_txg,\n        root_instance,\n    )	proof_errors=2
+M3·前提·规则只容得下一个结果去掉同一个读者（反向链回调）	crates/singlefs-replay-prefix/src/lib.rs	    replayed matches Ok(outcome) ==> (the_reader_answers_each_position_one_way(back_chain_broken)\n        && the_reader_answers_each_position_one_way(named_units_verified) ==> forall|	    replayed matches Ok(outcome) ==> (true\n        && the_reader_answers_each_position_one_way(named_units_verified) ==> forall|	proof_errors=1
+M3·前提·推论去掉择根（所选根取最新的有效根）	crates/singlefs-replay-prefix/src/lib.rs	        // 择根：所选根取根环里最新的有效根（3.2 第 1 步）\n        version_is_not_later_than(\n            smaller_root_checkpoint_txg,\n            smaller_root_instance,\n            larger_root_checkpoint_txg,\n            larger_root_instance,\n        ),	        // 择根：所选根取根环里最新的有效根（3.2 第 1 步）\n        true,	proof_errors=1
+M3·前提·推论去掉串行提交与持久顺序	crates/singlefs-replay-prefix/src/lib.rs	        smaller_outcome.applied@.len() > 0 ==> version_is_not_later_than(	        false ==> version_is_not_later_than(	proof_errors=2
+M3·前提·推论去掉同一个读者	crates/singlefs-replay-prefix/src/lib.rs	        // 同一个读者（只在 u 与 s 相同那一支用到）\n        the_reader_answers_each_position_one_way(smaller_back_chain_broken),	        // 同一个读者（只在 u 与 s 相同那一支用到）\n        true,	proof_errors=1
+M3·ensures·推论的结论改成 s 落点晚于 u 的	crates/singlefs-replay-prefix/src/lib.rs	        larger_replayed matches Ok(larger_outcome) ==> version_is_not_later_than(\n            landed_checkpoint_txg(\n                smaller_root_checkpoint_txg,	        larger_replayed matches Ok(larger_outcome) ==> !version_is_not_later_than(\n            landed_checkpoint_txg(\n                smaller_root_checkpoint_txg,	proof_errors=1
+M3·ensures·引理：候选按切片次序改成递减	crates/singlefs-replay-prefix/src/lib.rs	                0 <= earlier < later < candidates.len() ==> candidates[earlier] < candidates[later]	                0 <= earlier < later < candidates.len() ==> candidates[earlier] > candidates[later]	proof_errors=2
+M3·ensures·引理：所选根之上的记录在候选里的位置改成错一位	crates/singlefs-replay-prefix/src/lib.rs	        candidates_before(records, root_instance, root_checkpoint_txg, end)[position]\n            == record_index,	        candidates_before(records, root_instance, root_checkpoint_txg, end)[position]\n            == record_index + 1,	proof_errors=2
+M3·ensures·引理：没有记录在所选根之上时候选改成有一个	crates/singlefs-replay-prefix/src/lib.rs	    ensures\n        candidates_before(records, root_instance, root_checkpoint_txg, end).len() == 0,	    ensures\n        candidates_before(records, root_instance, root_checkpoint_txg, end).len() == 1,	proof_errors=2
+M3·ensures·引理：落点之下的候选个数改成严格少于全部候选	crates/singlefs-replay-prefix/src/lib.rs	            &&& below <= all.len()	            &&& below < all.len()	proof_errors=1
+M3·ensures·引理：过了第四格的末条之后同实例记录的 txg 改成更小	crates/singlefs-replay-prefix/src/lib.rs	                == records[end_index].instance ==> records[record_index].checkpoint_txg.0\n                > records[end_index].checkpoint_txg.0,\n{	                == records[end_index].instance ==> records[record_index].checkpoint_txg.0\n                < records[end_index].checkpoint_txg.0,\n{	proof_errors=2
+M3·ensures·引理：落到的那一版的发布里带末条标志的改成一条都没有	crates/singlefs-replay-prefix/src/lib.rs	        ) == if end > landed_record_index {\n            seq![records[landed_record_index].counter]	        ) == if end > landed_record_index {\n            Seq::<JournalCounter>::empty()	proof_errors=2
+M3·前提·引理：同一个读者不会既让一串停下又让另一串接受，去掉反向链回调那一个读者前提	crates/singlefs-replay-prefix/src/lib.rs	    requires\n        the_reader_answers_each_position_one_way(back_chain_broken),\n        the_reader_answers_each_position_one_way(named_units_verified),\n        continues_the_jsn_chain(	    requires\n        the_reader_answers_each_position_one_way(named_units_verified),\n        continues_the_jsn_chain(	proof_errors=1
+M3·ensures·引理：同样的入参过了逐条判定的那一串改成严格更长	crates/singlefs-replay-prefix/src/lib.rs	        shorter.admitted@.len() >= longer.admitted@.len(),	        shorter.admitted@.len() > longer.admitted@.len(),	proof_errors=1
+M3·ensures·引理：规则容得下的两个结果施加的改成不同	crates/singlefs-replay-prefix/src/lib.rs	        again.applied@ == outcome.applied@,\n        again.root_source_after_replay == outcome.root_source_after_replay,\n{	        again.applied@ != outcome.applied@,\n        again.root_source_after_replay == outcome.root_source_after_replay,\n{	proof_errors=1
+M3·ensures·引理：施加了时落点改成早于所选根（推论调的那一个）	crates/singlefs-replay-prefix/src/lib.rs	        replayed,\n        ),\n    ensures\n        replay_lands_no_earlier_than_the_chosen_root(	        replayed,\n        ),\n    ensures\n        !replay_lands_no_earlier_than_the_chosen_root(	proof_errors=1
+M3·ensures·引理：落点的 txg 改成小于所选根的	crates/singlefs-replay-prefix/src/lib.rs	            &&& landed_txg.0 > root_checkpoint_txg.0	            &&& landed_txg.0 < root_checkpoint_txg.0	proof_errors=1
+M3·ensures·引理：落到的那一版之上的候选改成原候选整张	crates/singlefs-replay-prefix/src/lib.rs	            &&& candidates_before(records, root_instance, landed_txg, records.len() as int)\n                == candidates.subrange(applied.len() as int, candidates.len() as int)	            &&& candidates_before(records, root_instance, landed_txg, records.len() as int)\n                == candidates	proof_errors=3
+M3·ensures·引理：不动点的第二次改成交回第一次的所选根段	crates/singlefs-replay-prefix/src/lib.rs	        replaying_again_from_the_landed_root_stays_put(\n            landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),\n            outcome.root_source_after_replay,\n            replayed_again,\n        ),\n{	        replaying_again_from_the_landed_root_stays_put(\n            landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),\n            chosen_root_source,\n            replayed_again,\n        ),\n{	proof_errors=1
+M3·ensures·引理：同一条记录紧接着施加两次改成与少施加最后一条相同	crates/singlefs-replay-prefix/src/lib.rs	        ) == root_source_after_applying_in_order(root_source, applied_records),	        ) == root_source_after_applying_in_order(root_source, applied_records.drop_last()),	proof_errors=1
+M3·改判定·去掉所选根 txg 顶格那一判（第三轮 S6 那一格的实现侧）	crates/singlefs-replay-prefix/src/lib.rs	    if root_checkpoint_txg.0 == u64::MAX {\n        return Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor);	    if false {\n        return Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor);	proof_errors=1
+M3·改判定·末条没到也施加攒着的（多施加半次发布，不动点那一侧再取一次会接着施加）	crates/singlefs-replay-prefix/src/lib.rs	        if !record_ends_its_publish(record) {\n            index += 1;\n            continue ;\n        }	        if false {\n            index += 1;\n            continue ;\n        }	proof_errors=2
```

## 实现员补丁 m3-formal-10-3（`/tmp/claude-1000/m3-formal-10-3/patch/crates.patch`）

```diff
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -18,10 +18,10 @@
     SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
 };
 use singlefs_replay_prefix::{
-    replay_core, CoreFailure, CoreRecord, JournalCounter, NamedUnitVerification,
-    PrefixKernelCheckpointTxg, PrefixKernelInstanceGeneration,
-    PrefixKernelJournalRecordOrdinalWithinPublish, PrefixKernelJournalRecordPlaceInPublish,
-    TransactionNumber,
+    choose_root_slot, first_copy_of_each_record_key, replay_core, CoreFailure, CoreRecord,
+    JournalCounter, NamedUnitVerification, PrefixKernelCheckpointTxg,
+    PrefixKernelInstanceGeneration, PrefixKernelJournalRecordOrdinalWithinPublish,
+    PrefixKernelJournalRecordPlaceInPublish, RootChoiceKey, TransactionNumber,
 };
 
 use crate::address::{
@@ -1242,27 +1242,48 @@
     }
 }
 
-/// 三个区域全部槽逐个验自证校验和，取 `(checkpoint_txg, 实例代号)` 最大的（D22（单元原子性怎么合成） 已定项 7 择新）。
+/// 三个区域全部槽逐个验自证校验和，取 `(checkpoint_txg, 实例代号)` 最大的（D22（单元原子性怎么合成） 已定项 7 择新；
+/// 已定项 24 先逐个验证全部候选、再在有效者中择新）；最大的不止一个槽时取读根环次序里最前的那个。
+/// 取最大那一截在 `singlefs_replay_prefix::choose_root_slot` 里（Verus 验的是那一份）；这里只读槽、验自证校验和与 fsid，
+/// 把每个槽摊成核的入参（自证过的给择新键，读不出、自证不过的给 None），再照核交回的下标取回那条根。
 #[must_use]
 pub fn choose_root(
     reader: &dyn PoolReader,
     system_configuration: &SystemConfiguration,
 ) -> Option<RootRecord> {
-    let mut best: Option<RootRecord> = None;
-    visit_valid_roots(
-        reader,
-        &system_configuration.immutable.region_devices,
-        &system_configuration.immutable.sizes,
-        &system_configuration.immutable.filesystem_identifier,
-        |candidate| {
-            let candidate_key = (candidate.checkpoint_txg, candidate.instance);
-            if best.is_none_or(|current| candidate_key > (current.checkpoint_txg, current.instance))
-            {
-                best = Some(candidate);
-            }
-        },
-    );
-    best
+    let immutable = &system_configuration.immutable;
+    let self_verified_roots_in_ring_order: Vec<Option<RootRecord>> =
+        every_root_ring_slot(&immutable.sizes)
+            .into_iter()
+            .map(|ring_slot| {
+                match read_root_ring_slot(
+                    reader,
+                    &immutable.region_devices,
+                    &immutable.sizes,
+                    &immutable.filesystem_identifier,
+                    ring_slot,
+                ) {
+                    RootRingSlotReading::SelfVerified(root) => Some(root),
+                    RootRingSlotReading::Bad(
+                        BadRootRingSlotReading::Unreadable
+                        | BadRootRingSlotReading::NotSelfVerified,
+                    ) => None,
+                }
+            })
+            .collect();
+    let choice_keys_in_ring_order: Vec<Option<RootChoiceKey>> = self_verified_roots_in_ring_order
+        .iter()
+        .map(|self_verified_root| {
+            self_verified_root.map(|root| RootChoiceKey {
+                checkpoint_txg: PrefixKernelCheckpointTxg(root.checkpoint_txg.0),
+                instance: PrefixKernelInstanceGeneration(root.instance.0),
+            })
+        })
+        .collect();
+    choose_root_slot(&choice_keys_in_ring_order).map(|chosen_slot| {
+        self_verified_roots_in_ring_order[chosen_slot]
+            .expect("择根核只交回自证过的槽（它的 ensures：所选槽在入参里是 Some），这里是 None 说明核或摊入参错了")
+    })
 }
 
 /// 根环全部自证过的根里最大的 checkpoint_txg：新实例的第一次发布取 max(它, 环里全部自证通过的记录的 checkpoint_txg) + 1
@@ -3007,7 +3028,7 @@
     let ring_start = DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES);
     let ring_bytes = system_configuration.immutable.sizes.journal_ring_bytes;
     let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
-    let mut records = BTreeMap::new();
+    let mut scanned_records_in_read_order: Vec<JournalRecord> = Vec::new();
     for device in reader.device_identities() {
         let ring_bytes_inside_the_device = reader
             .device_size_in_bytes(device)
@@ -3028,12 +3049,23 @@
                 continue;
             };
             if let Some(record) = JournalRecord::parse(&bytes, expected_filesystem_identifier) {
-                records
-                    .entry((record.instance, record.counter))
-                    .or_insert(record);
+                scanned_records_in_read_order.push(record);
             }
         }
     }
+    // 同一 (实例代号, 计数器) 先到先得（按上面的盘次序、环内偏移次序，第一份读到的留下）：留哪一份在
+    // `singlefs_replay_prefix::first_copy_of_each_record_key` 里（Verus 验的是那一份），这里照它交回的位置取。
+    let scanned_record_keys: Vec<(u32, u64)> = scanned_records_in_read_order
+        .iter()
+        .map(|record| (record.instance.0, record.counter))
+        .collect();
+    let first_copies = first_copy_of_each_record_key(&scanned_record_keys);
+    let mut records = BTreeMap::new();
+    for (position, record) in scanned_records_in_read_order.into_iter().enumerate() {
+        if first_copies.get(&(record.instance.0, record.counter)) == Some(&position) {
+            records.insert((record.instance, record.counter), record);
+        }
+    }
     records
 }
 
--- a/crates/singlefs-replay-prefix/src/lib.rs
+++ b/crates/singlefs-replay-prefix/src/lib.rs
@@ -17,9 +17,14 @@
 //! 别的形状（反向链断了、到了在飞上限、可写挂载走读阶段缓存、`recover` 不验点名单元那一臂）读几次盘没有用例钉。
 //! 平时的 stable 编译用 crates.io 的 `vstd` 与 `verus_builtin_macros` 把 `verus!` 擦成普通 Rust；
 //! 验的时候 Verus 用它自带的那一份，两边钉同一个发布（`Cargo.toml` 的 vstd 版本与 Verus 0.2026.09.20.aef82ed）。
+//! 另两个纯函数核也在这里：择根（`choose_root_slot`，`recovery::choose_root` 取 `(checkpoint_txg, 实例代号)` 最大的那一截）与
+//! 扫环建表（`first_copy_of_each_record_key`，`recovery::scan_journal` 同一 (实例代号, 计数器) 先到先得的那一截）；
+//! 择根与取前缀的合判（C650 的 Verus 一半）写成推论 `corollary_recovery_replays_from_the_newest_self_verified_root`。
 //! Verus 的必红世界登记在同目录的 `verus-mutations.tsv`。
 #![forbid(unsafe_code)]
 
+use std::collections::BTreeMap;
+
 use vstd::prelude::*;
 
 verus! {
@@ -83,6 +88,13 @@
     ChosenRootCheckpointTxgHasNoSuccessor,
 }
 
+// 根环一个自证过的槽的择新键（D22（单元原子性怎么合成）已定项 7）：择根核只看这两项，根记录别的字段留在胶水里。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct RootChoiceKey {
+    pub checkpoint_txg: PrefixKernelCheckpointTxg,
+    pub instance: PrefixKernelInstanceGeneration,
+}
+
 pub struct CoreOutcome<RootSource> {
     pub report: CoreReport,
     pub root_source_after_replay: RootSource,
@@ -932,6 +944,209 @@
             == outcome.root_source_after_replay))
 }
 
+// ===== 择根（spec，可信：照条款文字写）=====
+// D22（单元原子性怎么合成）已定项 7：择新按 (checkpoint_txg, 实例代号)，txg 为主、平局时实例代号高者赢；
+// 已定项 24：先逐个验证全部候选、再在有效者中择新。核的入参是根环三个区域全部槽按读根环的次序读出来的样子：
+// 自证过的槽给它的择新键，读不出、自证不过的槽给 None；读盘、自证校验和、fsid 由胶水（recovery.rs 的 choose_root）做，不在 Verus 里。
+// 择新序就是上面 M3 单调用的 version_is_not_later_than（第三轮 S5 定它与择根同序），这里不另写一个序。
+pub open spec fn root_choice_key_of(slot: Option<RootChoiceKey>) -> RootChoiceKey {
+    match slot {
+        Some(choice_key) => choice_key,
+        None => RootChoiceKey {
+            checkpoint_txg: PrefixKernelCheckpointTxg(0),
+            instance: PrefixKernelInstanceGeneration(0),
+        },
+    }
+}
+
+pub open spec fn root_choice_key_is_not_newer_than(
+    older: RootChoiceKey,
+    newer: RootChoiceKey,
+) -> bool {
+    version_is_not_later_than(
+        older.checkpoint_txg,
+        older.instance,
+        newer.checkpoint_txg,
+        newer.instance,
+    )
+}
+
+// 前 end 个槽里择出的那一个：一个自证过的都没有就不择；有就择择新序最大的；最大的不止一个槽（键相同）时取槽序最前的
+// （条款没写键相同取哪一个，这一句照 recovery.rs 原来「严格大于才换」的行为写）。
+pub open spec fn root_chosen_among_the_first_slots(
+    root_slots: Seq<Option<RootChoiceKey>>,
+    end: int,
+    chosen: Option<usize>,
+) -> bool {
+    match chosen {
+        None => forall|slot_index: int|
+            0 <= slot_index < end ==> (#[trigger] root_slots[slot_index]) is None,
+        Some(chosen_slot) => {
+            &&& 0 <= chosen_slot < end
+            &&& root_slots[chosen_slot as int] is Some
+            &&& forall|slot_index: int|
+                0 <= slot_index < end && (#[trigger] root_slots[slot_index]) is Some
+                    ==> root_choice_key_is_not_newer_than(
+                    root_choice_key_of(root_slots[slot_index]),
+                    root_choice_key_of(root_slots[chosen_slot as int]),
+                )
+            &&& forall|slot_index: int|
+                0 <= slot_index < chosen_slot && (#[trigger] root_slots[slot_index]) is Some
+                    ==> !root_choice_key_is_not_newer_than(
+                    root_choice_key_of(root_slots[chosen_slot as int]),
+                    root_choice_key_of(root_slots[slot_index]),
+                )
+        },
+    }
+}
+
+pub open spec fn root_choice_follows_the_newest_rule(
+    root_slots: Seq<Option<RootChoiceKey>>,
+    chosen: Option<usize>,
+) -> bool {
+    root_chosen_among_the_first_slots(root_slots, root_slots.len() as int, chosen)
+}
+
+// ===== 扫环建表（spec，可信）=====
+// recovery.rs 的 scan_journal 按读者的盘次序、每块盘环内偏移次序读到的记录，同一 (实例代号, 计数器) 先到先得：第一份读到的留下
+// （第三轮判决 research/prompts/m3-formal-r3-main-verification.md 第二节 S3、第四节第 10 条；replay_core 入参「严格升序」与 M2 语义层
+// 「同一 (实例代号, 计数器) 只对应一份内容」用的就是这张表）。核只管「每个键留第几份」：入参是读到的记录按读到的次序摊成的键，
+// 交回每个键第一次读到的位置；读盘、解记录由胶水做。
+// 键摊成 (u32, u64) 两个无符号整数，不用 PrefixKernelInstanceGeneration 与 JournalCounter：vstd 给 BTreeMap 的规格只在键的比较守
+// obeys_cmp 时成立，vstd 只为整数与整数的元组证了这一条（vstd::laws_cmp），这两个 newtype derive 出来的 Ord 没有。
+pub open spec fn first_copies_follow_the_first_come_rule(
+    scanned_keys: Seq<(u32, u64)>,
+    end: int,
+    first_copies: Map<(u32, u64), usize>,
+) -> bool {
+    // 表里的键正好是前 end 份读到的键（读到的每个键都在、没读到的一个不在）
+    &&& forall|record_key: (u32, u64)|
+        #[trigger] first_copies.contains_key(record_key) <==> exists|position: int|
+            0 <= position < end && scanned_keys[position] == record_key
+    // 先到先得：每个键记的是它第一次读到的那一份
+    &&& forall|record_key: (u32, u64)|
+        #[trigger] first_copies.contains_key(record_key) ==> {
+            &&& 0 <= first_copies[record_key] < end
+            &&& scanned_keys[first_copies[record_key] as int] == record_key
+            &&& forall|earlier: int|
+                0 <= earlier < first_copies[record_key] ==> #[trigger] scanned_keys[earlier]
+                    != record_key
+        }
+}
+
+// (实例代号, 计数器) 唯一：表里不同的键留的是不同的那一份（同一份读到的记录不会顶两个键）。
+pub open spec fn each_kept_copy_stands_for_one_record_key(
+    first_copies: Map<(u32, u64), usize>,
+) -> bool {
+    forall|one_key: (u32, u64), other_key: (u32, u64)|
+        #[trigger] first_copies.contains_key(one_key) && #[trigger] first_copies.contains_key(
+            other_key,
+        ) && one_key != other_key ==> first_copies[one_key] != first_copies[other_key]
+}
+
+// ===== 择根与取前缀合判（C650 的 Verus 一半；用户 2026-10-02 定 Y6 两边都做）=====
+// 落到的那一版不早于根环里任一自证过的根：择根取的是择新序最大的有效根，取前缀落点又不早于所选根（M3 的 J1）。
+// 改坏择根序（平局取低、实例代号为主、不看实例、取第一个有效者）时，所选根之外会有一条自证过的根比落点新，这一条就不成立。
+pub open spec fn replay_lands_no_earlier_than_any_self_verified_root<RootSource>(
+    root_slots: Seq<Option<RootChoiceKey>>,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
+    records: Seq<CoreRecord<RootSource>>,
+    replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
+) -> bool {
+    replayed matches Ok(outcome) ==> forall|slot_index: int|
+        0 <= slot_index < root_slots.len() && (#[trigger] root_slots[slot_index]) is Some
+            ==> version_is_not_later_than(
+            root_choice_key_of(root_slots[slot_index]).checkpoint_txg,
+            root_choice_key_of(root_slots[slot_index]).instance,
+            landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),
+            landed_instance(root_instance, records, outcome.applied@),
+        )
+}
+
+// 合成的那一条：所选根是有效候选里择新序最大的那条，从它取出的前缀满足 replay_core 的 M2（replay_follows_the_prefix_rules）
+// 与 M3 四条 ensures，落点不早于根环里任一自证过的根。
+pub open spec fn recovery_replays_from_the_newest_self_verified_root<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    root_slots: Seq<Option<RootChoiceKey>>,
+    chosen_slot: usize,
+    chosen_root_source: RootSource,
+    records: Seq<CoreRecord<RootSource>>,
+    in_flight_limit: usize,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+    replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
+) -> bool {
+    let chosen_key = root_choice_key_of(root_slots[chosen_slot as int]);
+    &&& root_choice_follows_the_newest_rule(root_slots, Some(chosen_slot))
+    &&& replay_follows_the_prefix_rules(
+        chosen_key.instance,
+        chosen_key.checkpoint_txg,
+        chosen_root_source,
+        records,
+        in_flight_limit,
+        named_unit_verification,
+        back_chain_broken,
+        named_units_verified,
+        replayed,
+    )
+    &&& replaying_an_applied_record_twice_leaves_the_root_as_once(
+        chosen_key.instance,
+        chosen_key.checkpoint_txg,
+        chosen_root_source,
+        records,
+        in_flight_limit,
+        named_unit_verification,
+        back_chain_broken,
+        named_units_verified,
+        replayed,
+    )
+    &&& replaying_from_the_landed_root_is_a_fixed_point(
+        chosen_key.instance,
+        chosen_key.checkpoint_txg,
+        chosen_root_source,
+        records,
+        in_flight_limit,
+        named_unit_verification,
+        back_chain_broken,
+        named_units_verified,
+        replayed,
+    )
+    &&& replay_lands_no_earlier_than_the_chosen_root(
+        chosen_key.instance,
+        chosen_key.checkpoint_txg,
+        chosen_root_source,
+        records,
+        in_flight_limit,
+        named_unit_verification,
+        back_chain_broken,
+        named_units_verified,
+        replayed,
+    )
+    &&& replay_outcome_is_the_only_one_the_rules_allow(
+        chosen_key.instance,
+        chosen_key.checkpoint_txg,
+        chosen_root_source,
+        records,
+        in_flight_limit,
+        named_unit_verification,
+        back_chain_broken,
+        named_units_verified,
+        replayed,
+    )
+    &&& replay_lands_no_earlier_than_any_self_verified_root(
+        root_slots,
+        chosen_key.instance,
+        chosen_key.checkpoint_txg,
+        records,
+        replayed,
+    )
+}
+
 // ===== 证明用的记法与引理（不可信件：写错了只会证不出来）=====
 pub open spec fn as_ints(indices: Seq<usize>) -> Seq<int> {
     Seq::new(indices.len(), |position: int| indices[position] as int)
@@ -2760,6 +2975,124 @@
     }
 }
 
+// 择根与取前缀合判（C650 的 Verus 一半）：前提就是两个核交回的东西——choose_root_slot 交回 Some(chosen_slot) 时它的 ensures，
+// 与拿那个槽的择新键当所选根调 replay_core 时它的五条 ensures；不加别的前提。胶水真的拿所选槽的根去调重放
+// （recovery.rs 的 recover、mount.rs 与 mounted_read.rs 的读阶段）不在 Verus 里。
+#[allow(clippy::too_many_arguments, reason = "入参照 replay_follows_the_prefix_rules 原样摊开，加根环与所选槽")]
+pub proof fn corollary_recovery_replays_from_the_newest_self_verified_root<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    root_slots: Seq<Option<RootChoiceKey>>,
+    chosen_slot: usize,
+    chosen_root_source: RootSource,
+    records: Seq<CoreRecord<RootSource>>,
+    in_flight_limit: usize,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+    replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
+)
+    requires
+        // choose_root_slot 的 ensures（交回 Some(chosen_slot) 那一支）
+        root_choice_follows_the_newest_rule(root_slots, Some(chosen_slot)),
+        // replay_core 的五条 ensures，所选根取所选槽的择新键
+        replay_follows_the_prefix_rules(
+            root_choice_key_of(root_slots[chosen_slot as int]).instance,
+            root_choice_key_of(root_slots[chosen_slot as int]).checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
+        replaying_an_applied_record_twice_leaves_the_root_as_once(
+            root_choice_key_of(root_slots[chosen_slot as int]).instance,
+            root_choice_key_of(root_slots[chosen_slot as int]).checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
+        replaying_from_the_landed_root_is_a_fixed_point(
+            root_choice_key_of(root_slots[chosen_slot as int]).instance,
+            root_choice_key_of(root_slots[chosen_slot as int]).checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
+        replay_lands_no_earlier_than_the_chosen_root(
+            root_choice_key_of(root_slots[chosen_slot as int]).instance,
+            root_choice_key_of(root_slots[chosen_slot as int]).checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
+        replay_outcome_is_the_only_one_the_rules_allow(
+            root_choice_key_of(root_slots[chosen_slot as int]).instance,
+            root_choice_key_of(root_slots[chosen_slot as int]).checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
+    ensures
+        recovery_replays_from_the_newest_self_verified_root(
+            root_slots,
+            chosen_slot,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
+{
+    let chosen_key = root_choice_key_of(root_slots[chosen_slot as int]);
+    match replayed {
+        Ok(outcome) => {
+            // 每一条自证过的根 ≤ 所选根（择根核）≤ 落点（J1）：择新序可传递。
+            assert forall|slot_index: int|
+                0 <= slot_index < root_slots.len() && (#[trigger] root_slots[slot_index]) is Some
+                    implies version_is_not_later_than(
+                root_choice_key_of(root_slots[slot_index]).checkpoint_txg,
+                root_choice_key_of(root_slots[slot_index]).instance,
+                landed_checkpoint_txg(chosen_key.checkpoint_txg, records, outcome.applied@),
+                landed_instance(chosen_key.instance, records, outcome.applied@),
+            ) by {
+                assert(root_choice_key_is_not_newer_than(
+                    root_choice_key_of(root_slots[slot_index]),
+                    chosen_key,
+                ));
+            }
+        },
+        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {},
+        Err(
+            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+                counters: _,
+            },
+        ) => {},
+    }
+}
+
 // ===== 纯函数核 =====
 // 记录标志位 0（已定项 14 第五条、已定项 17）。
 fn record_ends_its_publish<RootSource>(record: &CoreRecord<RootSource>) -> (is_last_record: bool)
@@ -3638,6 +3971,85 @@
     )
 }
 
+// recovery.rs 的 choose_root 去掉读槽之后的那一截：入参是根环三个区域全部槽按读根环的次序（区域号升序、区域内槽号升序）读出来的样子，
+// 交回所选槽在入参里的下标。判的次序照提交 fcc4d340 的 choose_root：逐槽往后，比已择的严格新才换。
+#[allow(clippy::manual_map, reason = "vstd 没给 Option::map 带闭包的规格，交回值写成 match 才验得了")]
+pub fn choose_root_slot(root_slots: &[Option<RootChoiceKey>]) -> (chosen: Option<usize>)
+    ensures
+        root_choice_follows_the_newest_rule(root_slots@, chosen),
+{
+    let mut chosen: Option<(usize, RootChoiceKey)> = None;
+    let mut slot_index: usize = 0;
+    while slot_index < root_slots.len()
+        invariant
+            slot_index <= root_slots.len(),
+            root_chosen_among_the_first_slots(
+                root_slots@,
+                slot_index as int,
+                match chosen {
+                    Some((chosen_slot, _)) => Some(chosen_slot),
+                    None => None,
+                },
+            ),
+            chosen matches Some((chosen_slot, chosen_key)) ==> chosen_key == root_choice_key_of(
+                root_slots@[chosen_slot as int],
+            ),
+        decreases root_slots.len() - slot_index,
+    {
+        if let Some(candidate_key) = root_slots[slot_index] {
+            let candidate_is_newer = match chosen {
+                None => true,
+                Some((_, chosen_key)) => candidate_key.checkpoint_txg.0 > chosen_key.checkpoint_txg.0
+                    || (candidate_key.checkpoint_txg.0 == chosen_key.checkpoint_txg.0
+                    && candidate_key.instance.0 > chosen_key.instance.0),
+            };
+            if candidate_is_newer {
+                chosen = Some((slot_index, candidate_key));
+            }
+        }
+        slot_index += 1;
+    }
+    match chosen {
+        Some((chosen_slot, _)) => Some(chosen_slot),
+        None => None,
+    }
+}
+
+// recovery.rs 的 scan_journal 去掉读盘、解记录之后建表的那一截：入参是读到的记录按读到的次序摊成的 (实例代号, 计数器)，
+// 交回每个键第一次读到的位置。原文在 BTreeMap 上 entry(key).or_insert(record)；vstd 的 BTreeMap 没有 entry / or_insert 的规格，
+// 这里写成先查后插，与原文同义：键已在就不动，不在才插。
+#[allow(clippy::map_entry, reason = "vstd 的 BTreeMap 没有 entry / or_insert 的规格，先查后插 Verus 才验得了")]
+pub fn first_copy_of_each_record_key(scanned_keys: &[(u32, u64)]) -> (first_copies: BTreeMap<
+    (u32, u64),
+    usize,
+>)
+    ensures
+        first_copies_follow_the_first_come_rule(
+            scanned_keys@,
+            scanned_keys@.len() as int,
+            first_copies@,
+        ),
+        each_kept_copy_stands_for_one_record_key(first_copies@),
+{
+    broadcast use vstd::laws_cmp::group_laws_cmp;
+
+    let mut first_copies: BTreeMap<(u32, u64), usize> = BTreeMap::new();
+    let mut position: usize = 0;
+    while position < scanned_keys.len()
+        invariant
+            position <= scanned_keys.len(),
+            first_copies_follow_the_first_come_rule(scanned_keys@, position as int, first_copies@),
+        decreases scanned_keys.len() - position,
+    {
+        let record_key = scanned_keys[position];
+        if !first_copies.contains_key(&record_key) {
+            first_copies.insert(record_key, position);
+        }
+        position += 1;
+    }
+    first_copies
+}
+
 // ===== spec 自己的冒烟（防 spec 恒真或恒假）=====
 proof fn a_publish_of_two_records_is_whole(first: CoreRecord<u64>, second: CoreRecord<u64>)
     requires
@@ -3673,14 +4085,15 @@
 
 } // verus!
 
-// 平时的 cargo test 跑的两条：Verus 证的是 spec，这两条钉住编进产品的那一份在两个手算得出的小例子上给出的数
+// 平时的 cargo test 跑的几条：Verus 证的是 spec，这几条钉住编进产品的那一份（取前缀、择根、扫环建表三个核）在手算得出的小例子上给出的数
 // （crates/mutations.tsv 里点名它们的变异改坏核，它们红）。
 #[cfg(test)]
 mod tests {
     use super::{
-        replay_core, CoreRecord, JournalCounter, NamedUnitVerification, PrefixKernelCheckpointTxg,
-        PrefixKernelInstanceGeneration, PrefixKernelJournalRecordOrdinalWithinPublish,
-        PrefixKernelJournalRecordPlaceInPublish, TransactionNumber,
+        choose_root_slot, first_copy_of_each_record_key, replay_core, CoreRecord, JournalCounter,
+        NamedUnitVerification, PrefixKernelCheckpointTxg, PrefixKernelInstanceGeneration,
+        PrefixKernelJournalRecordOrdinalWithinPublish, PrefixKernelJournalRecordPlaceInPublish,
+        RootChoiceKey, TransactionNumber,
     };
 
     const ROOT_INSTANCE: PrefixKernelInstanceGeneration = PrefixKernelInstanceGeneration(1);
@@ -3800,4 +4213,69 @@
             "新根段还是所选根的"
         );
     }
+
+    fn root_choice_key(checkpoint_txg: u64, instance: u32) -> Option<RootChoiceKey> {
+        Some(RootChoiceKey {
+            checkpoint_txg: PrefixKernelCheckpointTxg(checkpoint_txg),
+            instance: PrefixKernelInstanceGeneration(instance),
+        })
+    }
+
+    /// 两个槽 checkpoint_txg 相同：实例代号高的那个在后面的槽里也择它（D22 已定项 7 平局实例代号高者赢）。
+    #[test]
+    fn of_two_slots_with_the_same_checkpoint_txg_the_higher_instance_in_the_later_slot_is_chosen() {
+        let root_slots = [root_choice_key(4, 1), None, root_choice_key(4, 2)];
+        assert_eq!(
+            choose_root_slot(&root_slots),
+            Some(2),
+            "txg 平局，实例代号 2 高于 1：择槽 2"
+        );
+    }
+
+    /// txg 大的赢，哪怕另一个槽的实例代号更高（D22 已定项 7 txg 为主、实例代号只破平局）。
+    #[test]
+    fn a_later_checkpoint_txg_is_chosen_over_a_higher_instance_with_an_earlier_checkpoint_txg() {
+        let root_slots = [root_choice_key(2, 2), root_choice_key(3, 1)];
+        assert_eq!(
+            choose_root_slot(&root_slots),
+            Some(1),
+            "txg 3 新于 txg 2，实例代号 1 低于 2 也择槽 1"
+        );
+    }
+
+    /// 两个槽择新键完全相同：取读根环次序里前面那个（与原来严格大于才换同义）。
+    #[test]
+    fn of_two_slots_with_the_same_choice_key_the_earlier_slot_is_chosen() {
+        let root_slots = [None, root_choice_key(5, 3), root_choice_key(5, 3)];
+        assert_eq!(
+            choose_root_slot(&root_slots),
+            Some(1),
+            "键相同：留前面的槽 1"
+        );
+    }
+
+    /// 第一个自证过的槽不是最新的：往后看完全部槽再择（D22 已定项 24 先验全部候选再择新）。
+    #[test]
+    fn a_newer_root_after_the_first_self_verified_slot_is_chosen_over_it() {
+        let root_slots = [None, root_choice_key(2, 1), None, root_choice_key(3, 1)];
+        assert_eq!(
+            choose_root_slot(&root_slots),
+            Some(3),
+            "txg 3 新于 txg 2：择槽 3"
+        );
+    }
+
+    /// 同一 (实例代号, 计数器) 读到两份：留第一份读到的位置，后读到的那一份不顶替它。
+    #[test]
+    fn of_two_scanned_copies_of_one_record_key_the_first_position_is_kept() {
+        let scanned_keys = [(1, 7), (1, 8), (1, 7)];
+        let first_copies = first_copy_of_each_record_key(&scanned_keys);
+        assert_eq!(first_copies.get(&(1, 7)), Some(&0), "(1, 7) 第一份在位置 0");
+        assert_eq!(
+            first_copies.get(&(1, 8)),
+            Some(&1),
+            "(1, 8) 只有位置 1 那一份"
+        );
+        assert_eq!(first_copies.len(), 2, "两个键各留一份");
+    }
 }
--- a/crates/singlefs-replay-prefix/verus-mutations.tsv
+++ b/crates/singlefs-replay-prefix/verus-mutations.tsv
@@ -96,7 +96,7 @@
 改判定·末条不带提交标记照施加（已定项 14 五条口径第三条；攻方 V03）	crates/singlefs-replay-prefix/src/lib.rs	    if !record.is_commit && record_ends_its_publish(record) {	    if false {	proof_errors=2
 改判定·末条之后同一次发布还有记录照施加（已定项 4 读者规则第四格、C540；攻方 V04）	crates/singlefs-replay-prefix/src/lib.rs	    if record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(	    if false && readable_record_of_the_same_publish_follows(	proof_errors=2
 改判定·点名单元不验（已定项 14 五条口径第四条；攻方 V05）	crates/singlefs-replay-prefix/src/lib.rs	        NamedUnitVerification::Verify => named_units_verified(index),	        NamedUnitVerification::Verify => true,	proof_errors=1
-改判定·在飞上限不看（已定项 18；攻方 V06）	crates/singlefs-replay-prefix/src/lib.rs	        if records_taken_toward_the_in_flight_limit >= in_flight_limit {	        if false {	proof_errors=2
+改判定·在飞上限不看（已定项 18；攻方 V06）	crates/singlefs-replay-prefix/src/lib.rs	        if records_taken_toward_the_in_flight_limit >= in_flight_limit {	        if false {	proof_errors=3
 改判定·第一条水位之上的记录就停、一条都不施加（攻方 V07）	crates/singlefs-replay-prefix/src/lib.rs	        if records_taken_toward_the_in_flight_limit >= in_flight_limit {	        if true {	proof_errors=1
 改判定·锚点读不出时链首不看 txg（已定项 14 注 1；三方第一轮 K4-a；攻方 V08）	crates/singlefs-replay-prefix/src/lib.rs	            if record.checkpoint_txg.0 != chain_start_txg_without_anchor.0\n                || record.ordinal_within_publish.0 != 1 {	            if false\n                || record.ordinal_within_publish.0 != 1 {	proof_errors=1
 改判定·每次都报所选根 txg 没有后继（攻方 V09）	crates/singlefs-replay-prefix/src/lib.rs	    if root_checkpoint_txg.0 == u64::MAX {	    if true {	proof_errors=2
@@ -122,7 +122,7 @@
 M3·前提·不动点去掉所选根之上的记录不超在飞上限	crates/singlefs-replay-prefix/src/lib.rs	        && records_above_the_chosen_root_fit_the_in_flight_limit(\n        records,\n        root_instance,\n        root_checkpoint_txg,\n        in_flight_limit,\n    ) ==> forall|replayed_again: Result<CoreOutcome<RootSource>, CoreFailure>|	        && true ==> forall|replayed_again: Result<CoreOutcome<RootSource>, CoreFailure>|	proof_errors=1
 M3·spec·不动点 S6：txg 顶格报错不算相等（Qn 改成前提读法 P 漏判的那一侧）	crates/singlefs-replay-prefix/src/lib.rs	        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => landed_txg.0 == u64::MAX,	        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => false,	proof_errors=1
 M3·spec·不动点：第二次改成要施加一条	crates/singlefs-replay-prefix/src/lib.rs	        Ok(again) => again.applied@.len() == 0 && again.root_source_after_replay\n            == landed_root_source,	        Ok(again) => again.applied@.len() == 1 && again.root_source_after_replay\n            == landed_root_source,	proof_errors=1
-M3·spec·S5 版本序：实例代号破平局改成严格小于	crates/singlefs-replay-prefix/src/lib.rs	        == later_checkpoint_txg.0 && earlier_instance.0 <= later_instance.0)	        == later_checkpoint_txg.0 && earlier_instance.0 < later_instance.0)	proof_errors=2
+M3·spec·S5 版本序：实例代号破平局改成严格小于	crates/singlefs-replay-prefix/src/lib.rs	        == later_checkpoint_txg.0 && earlier_instance.0 <= later_instance.0)	        == later_checkpoint_txg.0 && earlier_instance.0 < later_instance.0)	proof_errors=3
 M3·spec·落点不早于所选根改成所选根不早于落点	crates/singlefs-replay-prefix/src/lib.rs	    replayed matches Ok(outcome) ==> version_is_not_later_than(\n        root_checkpoint_txg,\n        root_instance,\n        landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),\n        landed_instance(root_instance, records, outcome.applied@),\n    )	    replayed matches Ok(outcome) ==> version_is_not_later_than(\n        landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),\n        landed_instance(root_instance, records, outcome.applied@),\n        root_checkpoint_txg,\n        root_instance,\n    )	proof_errors=2
 M3·前提·规则只容得下一个结果去掉同一个读者（反向链回调）	crates/singlefs-replay-prefix/src/lib.rs	    replayed matches Ok(outcome) ==> (the_reader_answers_each_position_one_way(back_chain_broken)\n        && the_reader_answers_each_position_one_way(named_units_verified) ==> forall|	    replayed matches Ok(outcome) ==> (true\n        && the_reader_answers_each_position_one_way(named_units_verified) ==> forall|	proof_errors=1
 M3·前提·推论去掉择根（所选根取最新的有效根）	crates/singlefs-replay-prefix/src/lib.rs	        // 择根：所选根取根环里最新的有效根（3.2 第 1 步）\n        version_is_not_later_than(\n            smaller_root_checkpoint_txg,\n            smaller_root_instance,\n            larger_root_checkpoint_txg,\n            larger_root_instance,\n        ),	        // 择根：所选根取根环里最新的有效根（3.2 第 1 步）\n        true,	proof_errors=1
@@ -145,3 +145,24 @@
 M3·ensures·引理：同一条记录紧接着施加两次改成与少施加最后一条相同	crates/singlefs-replay-prefix/src/lib.rs	        ) == root_source_after_applying_in_order(root_source, applied_records),	        ) == root_source_after_applying_in_order(root_source, applied_records.drop_last()),	proof_errors=1
 M3·改判定·去掉所选根 txg 顶格那一判（第三轮 S6 那一格的实现侧）	crates/singlefs-replay-prefix/src/lib.rs	    if root_checkpoint_txg.0 == u64::MAX {\n        return Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor);	    if false {\n        return Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor);	proof_errors=1
 M3·改判定·末条没到也施加攒着的（多施加半次发布，不动点那一侧再取一次会接着施加）	crates/singlefs-replay-prefix/src/lib.rs	        if !record_ends_its_publish(record) {\n            index += 1;\n            continue ;\n        }	        if false {\n            index += 1;\n            continue ;\n        }	proof_errors=2
+# 择根、扫环与合判那一节（实现员 10-3 加，里程碑三七·4，用户 2026-10-02 定 Y5 择根加扫环、Y6 合判两边都做）：择根核 choose_root_slot 的改判定（原型 m1_tie_last_wins、m2_ignore_instance、m3_first_valid_wins 三条，C650 改坏择根序的平局取低、实例代号为主两条）、ensures、两条循环不变式与 spec 各一行；扫环建表核 first_copy_of_each_record_key 的 R3-10（先到先得改成后到者赢）、只收第一份、ensures、两条 spec 与循环不变式各一行；合判推论 corollary_recovery_replays_from_the_newest_self_verified_root（C650 的 Verus 一半）去掉两样前提各一行、结论改坏一行、择根 spec 改成不要求最大或换成实例代号为主的序各一行。
+择根·改判定·平局取后者赢（实例代号比较的严格大于改成大于等于；原型 m1_tie_last_wins）	crates/singlefs-replay-prefix/src/lib.rs	                    && candidate_key.instance.0 > chosen_key.instance.0),	                    && candidate_key.instance.0 >= chosen_key.instance.0),	proof_errors=1
+择根·改判定·比较不看实例代号（只比 checkpoint_txg；原型 m2_ignore_instance）	crates/singlefs-replay-prefix/src/lib.rs	                Some((_, chosen_key)) => candidate_key.checkpoint_txg.0 > chosen_key.checkpoint_txg.0\n                    || (candidate_key.checkpoint_txg.0 == chosen_key.checkpoint_txg.0\n                    && candidate_key.instance.0 > chosen_key.instance.0),	                Some((_, chosen_key)) => candidate_key.checkpoint_txg.0 > chosen_key.checkpoint_txg.0,	proof_errors=1
+择根·改判定·取第一个有效者（择了一个就不再换；原型 m3_first_valid_wins）	crates/singlefs-replay-prefix/src/lib.rs	            if candidate_is_newer {\n                chosen = Some((slot_index, candidate_key));	            if candidate_is_newer && chosen.is_none() {\n                chosen = Some((slot_index, candidate_key));	proof_errors=1
+C650·改判定·择根 txg 平局时实例代号低者赢（改坏择根序：平局取低）	crates/singlefs-replay-prefix/src/lib.rs	                    && candidate_key.instance.0 > chosen_key.instance.0),	                    && candidate_key.instance.0 < chosen_key.instance.0),	proof_errors=1
+C650·改判定·择根以实例代号为主、txg 破平局（改坏择根序：实例代号为主）	crates/singlefs-replay-prefix/src/lib.rs	                Some((_, chosen_key)) => candidate_key.checkpoint_txg.0 > chosen_key.checkpoint_txg.0\n                    || (candidate_key.checkpoint_txg.0 == chosen_key.checkpoint_txg.0\n                    && candidate_key.instance.0 > chosen_key.instance.0),	                Some((_, chosen_key)) => candidate_key.instance.0 > chosen_key.instance.0\n                    || (candidate_key.instance.0 == chosen_key.instance.0\n                    && candidate_key.checkpoint_txg.0 > chosen_key.checkpoint_txg.0),	proof_errors=1
+择根·ensures·choose_root_slot 的结论取反	crates/singlefs-replay-prefix/src/lib.rs	        root_choice_follows_the_newest_rule(root_slots@, chosen),	        !root_choice_follows_the_newest_rule(root_slots@, chosen),	proof_errors=1
+择根·循环不变式·记着的择新键改成不等于所选槽的键	crates/singlefs-replay-prefix/src/lib.rs	            chosen matches Some((chosen_slot, chosen_key)) ==> chosen_key == root_choice_key_of(	            chosen matches Some((chosen_slot, chosen_key)) ==> chosen_key != root_choice_key_of(	proof_errors=2
+择根·循环不变式·看过的槽改成多算一个	crates/singlefs-replay-prefix/src/lib.rs	                root_slots@,\n                slot_index as int,\n	                root_slots@,\n                slot_index as int + 1,\n	proof_errors=3
+择根·spec·键相同改成取槽序最后的（所选槽之后的自证过的槽都严格更旧）	crates/singlefs-replay-prefix/src/lib.rs	                0 <= slot_index < chosen_slot && (#[trigger] root_slots[slot_index]) is Some	                chosen_slot < slot_index < end && (#[trigger] root_slots[slot_index]) is Some	proof_errors=1
+C650·spec·择根结论只要求所选槽自证过、不要求择新序最大（取第一个有效者也合这一句；合判那一条红）	crates/singlefs-replay-prefix/src/lib.rs	                0 <= slot_index < end && (#[trigger] root_slots[slot_index]) is Some\n                    ==> root_choice_key_is_not_newer_than(	                0 <= slot_index < end && (#[trigger] root_slots[slot_index]) is Some && false\n                    ==> root_choice_key_is_not_newer_than(	proof_errors=2
+C650·spec·择根的择新序改成实例代号为主（与重放的版本序不再同序）	crates/singlefs-replay-prefix/src/lib.rs	    version_is_not_later_than(\n        older.checkpoint_txg,\n        older.instance,\n        newer.checkpoint_txg,\n        newer.instance,\n    )	    older.instance.0 < newer.instance.0 || (older.instance == newer.instance\n        && older.checkpoint_txg.0 <= newer.checkpoint_txg.0)	proof_errors=2
+C650·前提·合判去掉择根那一条（choose_root_slot 的 ensures）	crates/singlefs-replay-prefix/src/lib.rs	        // choose_root_slot 的 ensures（交回 Some(chosen_slot) 那一支）\n        root_choice_follows_the_newest_rule(root_slots, Some(chosen_slot)),\n	        // choose_root_slot 的 ensures（交回 Some(chosen_slot) 那一支）\n	proof_errors=2
+C650·前提·合判去掉 replay_core 的 M2 那一条（replay_follows_the_prefix_rules）	crates/singlefs-replay-prefix/src/lib.rs	        // replay_core 的五条 ensures，所选根取所选槽的择新键\n        replay_follows_the_prefix_rules(	        // replay_core 的五条 ensures，所选根取所选槽的择新键\n        true || replay_follows_the_prefix_rules(	proof_errors=1
+C650·ensures·合判结论改成落点早于每一条自证过的根	crates/singlefs-replay-prefix/src/lib.rs	            ==> version_is_not_later_than(\n            root_choice_key_of(root_slots[slot_index]).checkpoint_txg,	            ==> !version_is_not_later_than(\n            root_choice_key_of(root_slots[slot_index]).checkpoint_txg,	proof_errors=1
+R3-10·改判定·扫环建表先到先得改成后到者赢（先查后插改成直接插）	crates/singlefs-replay-prefix/src/lib.rs	        if !first_copies.contains_key(&record_key) {\n            first_copies.insert(record_key, position);\n        }	        first_copies.insert(record_key, position);	proof_errors=1
+扫环·改判定·只收第一份读到的记录（之后读到的键不进表）	crates/singlefs-replay-prefix/src/lib.rs	        if !first_copies.contains_key(&record_key) {	        if position == 0 {	proof_errors=1
+扫环·ensures·first_copy_of_each_record_key 先到先得那一条取反	crates/singlefs-replay-prefix/src/lib.rs	    ensures\n        first_copies_follow_the_first_come_rule(\n            scanned_keys@,	    ensures\n        !first_copies_follow_the_first_come_rule(\n            scanned_keys@,	proof_errors=1
+扫环·spec·(实例代号, 计数器) 唯一改成不同的键留同一份	crates/singlefs-replay-prefix/src/lib.rs	        ) && one_key != other_key ==> first_copies[one_key] != first_copies[other_key]	        ) && one_key != other_key ==> first_copies[one_key] == first_copies[other_key]	proof_errors=1
+扫环·spec·先到先得改成后到者赢（之前都不是这个键改成之后都不是）	crates/singlefs-replay-prefix/src/lib.rs	                0 <= earlier < first_copies[record_key] ==> #[trigger] scanned_keys[earlier]	                first_copies[record_key] < earlier < end ==> #[trigger] scanned_keys[earlier]	proof_errors=1
+扫环·循环不变式·读过的位置改成多算一个	crates/singlefs-replay-prefix/src/lib.rs	            first_copies_follow_the_first_come_rule(scanned_keys@, position as int, first_copies@),	            first_copies_follow_the_first_come_rule(scanned_keys@, position as int + 1, first_copies@),	proof_errors=3
diff --git a/crates/singlefs-harness/tests/root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot.rs b/crates/singlefs-harness/tests/root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot.rs
new file mode 100644
--- /dev/null
+++ b/crates/singlefs-harness/tests/root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot.rs
@@ -0,0 +1,123 @@
+//! 择根（`recovery::choose_root`，取最大那一截在 `singlefs_replay_prefix::choose_root_slot`）在真池的根环上：
+//! 两条自证过的根 checkpoint_txg 相同、实例代号不同，实例代号低的那条在读根环次序里排前面——
+//! 择新照 D22（单元原子性怎么合成） 已定项 7 txg 为主、平局实例代号高者赢，择排在后面的那条；
+//! 两条都排在 mkfs 那条根之后，已定项 24「先逐个验证全部候选、再在有效者中择新」也要看完全部槽才择得对。
+//! 池是只做过 mkfs 的池，两条根照 mkfs 那条根改实例代号与 txg、重新编码写进根环里空着的槽（择根只读根环，不看别的结构）。
+mod common;
+
+use common::format_pool;
+use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::recovery::{
+    choose_root, choose_system_configuration, every_root_ring_slot, read_root_ring_slot,
+    RootRingSlotReading,
+};
+use singlefs_core::root_record::RootRecord;
+use singlefs_core::root_ring::{slot_offset, RootRingSlot};
+use singlefs_core::system_configuration::SystemConfiguration;
+use singlefs_harness::memory_pool::MemoryPool;
+
+/// 排在前面那条根的实例代号（低）。
+const LOWER_INSTANCE: InstanceGeneration = InstanceGeneration(1);
+/// 排在后面那条根的实例代号（高）。
+const HIGHER_INSTANCE: InstanceGeneration = InstanceGeneration(2);
+
+fn ring_slot_holds_a_self_verified_root(
+    image: &MemoryPool,
+    system_configuration: &SystemConfiguration,
+    ring_slot: RootRingSlot,
+) -> bool {
+    match read_root_ring_slot(
+        image,
+        &system_configuration.immutable.region_devices,
+        &system_configuration.immutable.sizes,
+        &system_configuration.immutable.filesystem_identifier,
+        ring_slot,
+    ) {
+        RootRingSlotReading::SelfVerified(_) => true,
+        RootRingSlotReading::Bad(_) => false,
+    }
+}
+
+fn write_root_into_ring_slot(
+    image: &mut MemoryPool,
+    system_configuration: &SystemConfiguration,
+    ring_slot: RootRingSlot,
+    root: &RootRecord,
+) {
+    let immutable = &system_configuration.immutable;
+    let device = immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
+    let root_slot_bytes = usize::try_from(immutable.sizes.physical_block_size).expect("根槽宽");
+    image.devices.get_mut(&device).expect("池里有这块盘").write(
+        slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing),
+        &root.to_slot(root_slot_bytes),
+    );
+}
+
+#[test]
+fn of_two_self_verified_roots_with_the_same_checkpoint_txg_choose_root_takes_the_higher_instance_although_it_sits_in_a_later_slot(
+) {
+    let mut image = format_pool("root-choice-tied-checkpoint-txg").memory_pool();
+    let system_configuration =
+        choose_system_configuration(&image).expect("mkfs 之后择得出系统配置");
+    let mkfs_root = choose_root(&image, &system_configuration).expect("mkfs 写了一条根");
+    let ring_slots = every_root_ring_slot(&system_configuration.immutable.sizes);
+    let mkfs_root_slot_position = ring_slots
+        .iter()
+        .position(|ring_slot| {
+            ring_slot_holds_a_self_verified_root(&image, &system_configuration, *ring_slot)
+        })
+        .expect("根环里有 mkfs 那条根");
+    let empty_slots_after_the_mkfs_root: Vec<RootRingSlot> = ring_slots
+        [mkfs_root_slot_position + 1..]
+        .iter()
+        .copied()
+        .filter(|ring_slot| {
+            !ring_slot_holds_a_self_verified_root(&image, &system_configuration, *ring_slot)
+        })
+        .collect();
+    let earlier_slot = *empty_slots_after_the_mkfs_root
+        .first()
+        .expect("mkfs 那条根之后有空槽");
+    let later_slot = *empty_slots_after_the_mkfs_root
+        .last()
+        .expect("mkfs 那条根之后有空槽");
+    assert!(
+        earlier_slot < later_slot,
+        "两条根要落在两个不同的槽里，实例代号低的那条在前"
+    );
+    let tied_checkpoint_txg = CheckpointTxg(mkfs_root.checkpoint_txg.0 + 1);
+    // 两条根除实例代号与 txg 之外照抄 mkfs 那条根：择根只看这两项与自证校验和。
+    let mut lower_instance_root = mkfs_root;
+    lower_instance_root.instance = LOWER_INSTANCE;
+    lower_instance_root.checkpoint_txg = tied_checkpoint_txg;
+    let mut higher_instance_root = mkfs_root;
+    higher_instance_root.instance = HIGHER_INSTANCE;
+    higher_instance_root.checkpoint_txg = tied_checkpoint_txg;
+    write_root_into_ring_slot(
+        &mut image,
+        &system_configuration,
+        earlier_slot,
+        &lower_instance_root,
+    );
+    write_root_into_ring_slot(
+        &mut image,
+        &system_configuration,
+        later_slot,
+        &higher_instance_root,
+    );
+
+    let chosen = choose_root(&image, &system_configuration).expect("根环里有自证过的根");
+
+    assert_eq!(
+        chosen.checkpoint_txg, tied_checkpoint_txg,
+        "择的是 txg 最大的那两条之一，不是 mkfs 那条（取第一个有效者会停在 mkfs 那条）"
+    );
+    assert_eq!(
+        chosen.instance, HIGHER_INSTANCE,
+        "txg 平局时实例代号高者赢，哪怕它排在后面的槽里（只比 txg 会停在排前面的实例 1）"
+    );
+    assert_eq!(
+        chosen, higher_instance_root,
+        "择出的就是写进后一个槽的那条根"
+    );
+}
```

### 追加进 crates/mutations.tsv 的行

```
择根核平局取后者赢（里程碑三七·4 择根核，原型 m1_tie_last_wins）：择新键完全相同的两个槽取了后面那个，不是读根环次序里前面那个	crates/singlefs-replay-prefix/src/lib.rs	                    && candidate_key.instance.0 > chosen_key.instance.0),	                    && candidate_key.instance.0 >= chosen_key.instance.0),	-p singlefs-replay-prefix --lib	of_two_slots_with_the_same_choice_key_the_earlier_slot_is_chosen
择根核比较不看实例代号（D22 已定项 7 平局实例代号高者赢；原型 m2_ignore_instance）：txg 平局时停在读根环次序里前面那条实例代号低的根	crates/singlefs-replay-prefix/src/lib.rs	                Some((_, chosen_key)) => candidate_key.checkpoint_txg.0 > chosen_key.checkpoint_txg.0\n                    || (candidate_key.checkpoint_txg.0 == chosen_key.checkpoint_txg.0\n                    && candidate_key.instance.0 > chosen_key.instance.0),	                Some((_, chosen_key)) => candidate_key.checkpoint_txg.0 > chosen_key.checkpoint_txg.0,	-p singlefs-harness --test root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot	of_two_self_verified_roots_with_the_same_checkpoint_txg_choose_root_takes_the_higher_instance_although_it_sits_in_a_later_slot
择根核取第一个有效者（D22 已定项 24 先逐个验证全部候选、再在有效者中择新；原型 m3_first_valid_wins）：择了一个自证过的槽就不再换	crates/singlefs-replay-prefix/src/lib.rs	            if candidate_is_newer {\n                chosen = Some((slot_index, candidate_key));	            if candidate_is_newer && chosen.is_none() {\n                chosen = Some((slot_index, candidate_key));	-p singlefs-replay-prefix --lib	a_newer_root_after_the_first_self_verified_slot_is_chosen_over_it
择根核 txg 平局时实例代号低者赢（C650 改坏择根序：同 checkpoint_txg 平局取低）	crates/singlefs-replay-prefix/src/lib.rs	                    && candidate_key.instance.0 > chosen_key.instance.0),	                    && candidate_key.instance.0 < chosen_key.instance.0),	-p singlefs-replay-prefix --lib	of_two_slots_with_the_same_checkpoint_txg_the_higher_instance_in_the_later_slot_is_chosen
择根核以实例代号为主、txg 只破平局（C650 改坏择根序：实例代号为主）	crates/singlefs-replay-prefix/src/lib.rs	                Some((_, chosen_key)) => candidate_key.checkpoint_txg.0 > chosen_key.checkpoint_txg.0\n                    || (candidate_key.checkpoint_txg.0 == chosen_key.checkpoint_txg.0\n                    && candidate_key.instance.0 > chosen_key.instance.0),	                Some((_, chosen_key)) => candidate_key.instance.0 > chosen_key.instance.0\n                    || (candidate_key.instance.0 == chosen_key.instance.0\n                    && candidate_key.checkpoint_txg.0 > chosen_key.checkpoint_txg.0),	-p singlefs-replay-prefix --lib	a_later_checkpoint_txg_is_chosen_over_a_higher_instance_with_an_earlier_checkpoint_txg
择根胶水摊择新键时不带实例代号（核看到的实例代号都是 0，txg 平局时停在读根环次序里前面那条）	crates/singlefs-core/src/recovery.rs	                instance: PrefixKernelInstanceGeneration(root.instance.0),	                instance: PrefixKernelInstanceGeneration(0),	-p singlefs-harness --test root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot	of_two_self_verified_roots_with_the_same_checkpoint_txg_choose_root_takes_the_higher_instance_although_it_sits_in_a_later_slot
扫环胶水不照核交回的位置取、读到的每一份都插（同一 (实例代号, 计数器) 后读到的那份覆盖先读到的，R3-10 的胶水一侧）	crates/singlefs-core/src/recovery.rs	        if first_copies.get(&(record.instance.0, record.counter)) == Some(&position) {	        if first_copies.contains_key(&(record.instance.0, record.counter)) {	-p singlefs-harness --test journal_scan_keeps_the_first_copy_of_a_duplicated_record_key	of_two_differing_copies_of_one_record_key_the_scan_keeps_the_copy_on_the_device_scanned_first
扫环建表核只收第一份读到的记录（之后读到的键都不进表）	crates/singlefs-replay-prefix/src/lib.rs	        if !first_copies.contains_key(&record_key) {	        if position == 0 {	-p singlefs-replay-prefix --lib	of_two_scanned_copies_of_one_record_key_the_first_position_is_kept
```

## E165 装置（底 → 现状）

```diff
--- /tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	2026-10-01 06:34:33.854827694 +0000
+++ crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	2026-10-02 11:00:57.163920725 +0000
@@ -8,6 +8,8 @@
 //!
 //! 域（登记 5.2）：合法历史一族 L、L6、L9、U、X、E1、E2、R、C、C6；对抗一族 A、A6、AC。界由 `--bounds segment1|segment2|light` 给，
 //! `--only <域>`（可写几个，逗号分隔）、`--max-n <N>`、`--family legal|adversarial|all` 只跑一部分。
+//! 多盘域 M、M6、MC（登记 `research/prompts/e165-r2-prereg.md` 5.3，第三段）只在 `--profile r2 --family multi_disk` 时跑：每条记录在 0、1 号盘
+//! 同一落点各一份、各自落或没落，「完整在盘上」取任一份，直接建表先 0 号盘再 1 号盘；`--family all` 仍只含合法历史与对抗两族。
 //! 线程数取环境变量 `E165_THREADS`，没设取 `SINGLEFS_THREAD_CAP`，都没设取 `available_parallelism`；计数与线程数无关。
 //! 产物是 `E7RESULT` 行，收尾 `name=done emitted=N`；耗时另打 `E7TIMING` 行，不进逐字比对。
 //! `light` 档的界：L 到 9、U 到 5、A 到 3，其余同第一段（2026-10-01 在 10 线程上量到每态挂钟 0.24–1.2 微秒，每个域取到 3 秒之内）。
@@ -25,7 +27,8 @@
 //!
 //! 钉住的红例逐行重放：`--pinned-cases <表>` 读 `.claude/gate.d/e165-pinned-red-examples.tsv` 那种表（格式在它的文件头），
 //! 按数据行的次序每行判一个状态、打一行 `name=state`（与 `--replay-state` 打的那一行是同一段代码）；按 ②③④ 找到状态之后
-//! 拿它重算 ①（`sha256(<域>|<N>|<state= 的值，截到 ;first= 之前>)`），与表里不同、或有行找不到状态、或行本身不对，退 1。
+//! 拿它重算 ①（`sha256(<域>|<N>|<state= 的值，截到 ;first= 之前>)`），与表里不同、或有行找不到状态、或行本身不对
+//! （不是二十一段、⑨ 起抄来的红例账那一行与 ②③④⑦⑧ 对不上），退 1。
 //! 逐态行与 ⑦ 比不比得上由读这张表的一方（`research/scripts/e165-pin-red-examples.py --replay`）判，装置不判。
 
 use std::cell::Cell;
@@ -49,10 +52,16 @@
 };
 use singlefs_core::pointer::{BirthSequence, LocationEntry, NodePointer, PointerHead};
 use singlefs_core::recovery::{
-    replay_journal, PoolReader, RecoveryFailure, CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR,
+    replay_journal, scan_journal, PoolReader, RecoveryFailure,
+    CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR,
 };
 use singlefs_core::root_record::{RootRecord, UnmountMarker};
-use singlefs_core::system_configuration::journal_in_flight_record_limit;
+use singlefs_core::root_ring::RootRingSlotsPerRegion;
+use singlefs_core::system_configuration::{
+    journal_in_flight_record_limit, SystemConfiguration, SystemImmutableConfiguration,
+    SystemImmutableSizes, SystemMutableConfiguration, SystemRuntimeConfiguration,
+    SystemRuntimeQuantities,
+};
 use singlefs_harness::sha256::sha256_hexadecimal;
 
 // ── 本地常量：值抄自 kb，不从 `crates/` 引；[`cross_check_local_constants_against_the_implementation`] 逐个回比（登记第三节、停机条款 H2）。
@@ -150,6 +159,244 @@
     }
 }
 
+// ── 档（登记 r2 5.1）：r1 档的生成器、判据与输出行同 r1 时的装置；r2 档投影之外的根字段随版本变，另判 J6Qn、J7
+
+/// r2 档卸载记号的奇偶（登记 r2 第一节、第八节；`--marker-parity even|odd`，不给取 even）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum MarkerParity {
+    /// 偶数版 `WrittenByTheUnmountSequence`、奇数版 `NotWrittenByTheUnmountSequence`（R_0 是 `Written…`）。
+    EvenVersionsWrittenByTheUnmountSequence,
+    /// 反过来：奇数版 `Written…`、偶数版 `NotWritten…`（R_0 是 `NotWritten…`）。
+    OddVersionsWrittenByTheUnmountSequence,
+}
+
+impl MarkerParity {
+    const fn label(self) -> &'static str {
+        match self {
+            Self::EvenVersionsWrittenByTheUnmountSequence => "even",
+            Self::OddVersionsWrittenByTheUnmountSequence => "odd",
+        }
+    }
+}
+
+/// 时间线上每一版根记录投影（六个字段）之外的四个字段怎么取。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum FieldsBeyondTheProjectionGenerator {
+    /// r1 档：全历史相同（fsid 常量、卸载记号 `NotWritten…`、实例表槽 69 000、分配记录树根空）。
+    SameForEveryVersion,
+    /// r2 档（登记 r2 第一节）：第 j 版实例表指针取槽 69 000 + j、分配记录树根取槽 68 000 + j、fsid 末字节取 j，卸载记号按奇偶。
+    VaryWithTheVersion(MarkerParity),
+}
+
+/// 判哪几条判据：r1 档只判原登记与 r1 登记的；r2 档另判 J6Qn 两行与 J7 五行（登记 r2 5.4）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum CriteriaSet {
+    RegisteredBeforeR2,
+    WithJ6QnAndJ7,
+}
+
+/// 判据一侧拿哪张记录表去判（登记 r2 5.4 T-表、T-行；`--table direct|scan|both`，不给取 `direct`，只在 r2 档有）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum RecordTableSource {
+    /// 装置自己建的表：生成每条记录时解一次 `parse`，按盘次序逐槽收，`(实例, 计数器)` 先到先得（登记 5.1）。
+    Direct,
+    /// 把状态摆上内存盘，调 `scan_journal` 从盘上字节建表（登记 r2 第三节「由 scan_journal 建表」）。
+    Scanned,
+    /// 两张表各判一遍逐态比；汇总行取直接建表那一遍，另打 `name=table_comparison` 行。
+    Both,
+}
+
+impl RecordTableSource {
+    const fn label(self) -> &'static str {
+        match self {
+            Self::Direct => "direct",
+            Self::Scanned => "scan",
+            Self::Both => "both",
+        }
+    }
+}
+
+/// 内存盘的 `device_identities()` 报哪块盘在前（登记 r2 第八节；`--disk-order normal|reversed`，不给取 `normal`，只在 r2 档有）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum ReportedDiskOrder {
+    FirstDeviceBeforeSecond,
+    SecondDeviceBeforeFirst,
+}
+
+impl ReportedDiskOrder {
+    const fn label(self) -> &'static str {
+        match self {
+            Self::FirstDeviceBeforeSecond => "normal",
+            Self::SecondDeviceBeforeFirst => "reversed",
+        }
+    }
+
+    const fn device_numbers(self) -> [u32; 2] {
+        match self {
+            Self::FirstDeviceBeforeSecond => [0, 1],
+            Self::SecondDeviceBeforeFirst => [1, 0],
+        }
+    }
+}
+
+/// 一次跑的档：生成器与判据两样，另有 r2 档第四段加的两个旋钮（记录表怎么建、盘次序）。`r1` 与 `r2` 是命令行给得出的两种；
+/// 「r1 档生成器 + r2 判据」只给 J7 的阳性对照用（登记 r2 5.5「同一个对照在 r1 档生成器上 = 0」）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct RunProfile {
+    generator: FieldsBeyondTheProjectionGenerator,
+    criteria: CriteriaSet,
+    table_source: RecordTableSource,
+    reported_disk_order: ReportedDiskOrder,
+}
+
+/// 不给 `--profile` 时取的档（登记 r2 5.1）。
+const DEFAULT_PROFILE_LABEL: &str = "r1";
+/// r1 档时间线根的实例表单元指针住的槽；r2 档第 j 版取它加 j。
+const INSTANCE_TABLE_SLOT_OF_THE_FIRST_VERSION: u64 = 69_000;
+/// r2 档第 j 版分配记录树根住的槽：它加 j。
+const ALLOCATION_RECORD_TREE_ROOT_SLOT_OF_THE_FIRST_VERSION: u64 = 68_000;
+/// 16 字节 fsid 里 r2 档随版本变的那一个字节：末字节（低 8 字节不动，登记 r2 第一节）。
+const FILESYSTEM_IDENTIFIER_BYTE_THAT_VARIES_WITH_THE_VERSION: usize = 15;
+
+impl RunProfile {
+    const R1: Self = Self {
+        generator: FieldsBeyondTheProjectionGenerator::SameForEveryVersion,
+        criteria: CriteriaSet::RegisteredBeforeR2,
+        table_source: RecordTableSource::Direct,
+        reported_disk_order: ReportedDiskOrder::FirstDeviceBeforeSecond,
+    };
+
+    const fn r2(marker_parity: MarkerParity) -> Self {
+        Self {
+            generator: FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
+            criteria: CriteriaSet::WithJ6QnAndJ7,
+            table_source: RecordTableSource::Direct,
+            reported_disk_order: ReportedDiskOrder::FirstDeviceBeforeSecond,
+        }
+    }
+
+    /// 只给 J7 的阳性对照用：生成器照 r1 档，判据照 r2 档。
+    const R1_GENERATOR_WITH_R2_CRITERIA: Self = Self {
+        generator: FieldsBeyondTheProjectionGenerator::SameForEveryVersion,
+        criteria: CriteriaSet::WithJ6QnAndJ7,
+        table_source: RecordTableSource::Direct,
+        reported_disk_order: ReportedDiskOrder::FirstDeviceBeforeSecond,
+    };
+
+    const fn with_table_source(self, table_source: RecordTableSource) -> Self {
+        Self {
+            generator: self.generator,
+            criteria: self.criteria,
+            table_source,
+            reported_disk_order: self.reported_disk_order,
+        }
+    }
+
+    const fn with_reported_disk_order(self, reported_disk_order: ReportedDiskOrder) -> Self {
+        Self {
+            generator: self.generator,
+            criteria: self.criteria,
+            table_source: self.table_source,
+            reported_disk_order,
+        }
+    }
+
+    const fn judges_j6qn_and_j7(self) -> bool {
+        match self.criteria {
+            CriteriaSet::RegisteredBeforeR2 => false,
+            CriteriaSet::WithJ6QnAndJ7 => true,
+        }
+    }
+
+    /// 两种建表法各判一遍逐态比（`--table both`）。
+    const fn compares_the_two_record_tables(self) -> bool {
+        matches!(self.table_source, RecordTableSource::Both)
+    }
+
+    /// 时间线第 `version` 版 R_j 投影之外的四个字段。
+    fn fields_beyond_the_projection_of_version(self, version: usize) -> FieldsBeyondTheProjection {
+        match self.generator {
+            FieldsBeyondTheProjectionGenerator::SameForEveryVersion => FieldsBeyondTheProjection {
+                filesystem_identifier: ROOT_FILESYSTEM_IDENTIFIER,
+                unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
+                instance_table: node_pointer_at_slot(INSTANCE_TABLE_SLOT_OF_THE_FIRST_VERSION, 1),
+                allocation_record_tree_root: NodePointer::empty_root(),
+            },
+            FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity) => {
+                let version_in_the_varying_fields = version;
+                let version_number =
+                    u64::try_from(version_in_the_varying_fields).expect("版本数不超过 12");
+                let mut filesystem_identifier = ROOT_FILESYSTEM_IDENTIFIER;
+                filesystem_identifier[FILESYSTEM_IDENTIFIER_BYTE_THAT_VARIES_WITH_THE_VERSION] =
+                    u8::try_from(version_number).expect("版本数不超过 12，装得进一个字节");
+                let version_is_even = version_number % 2 == 0;
+                let written_by_the_unmount_sequence = match marker_parity {
+                    MarkerParity::EvenVersionsWrittenByTheUnmountSequence => version_is_even,
+                    MarkerParity::OddVersionsWrittenByTheUnmountSequence => !version_is_even,
+                };
+                FieldsBeyondTheProjection {
+                    filesystem_identifier,
+                    unmount_marker: if written_by_the_unmount_sequence {
+                        UnmountMarker::WrittenByTheUnmountSequence
+                    } else {
+                        UnmountMarker::NotWrittenByTheUnmountSequence
+                    },
+                    instance_table: node_pointer_at_slot(
+                        INSTANCE_TABLE_SLOT_OF_THE_FIRST_VERSION + version_number,
+                        1,
+                    ),
+                    allocation_record_tree_root: node_pointer_at_slot(
+                        ALLOCATION_RECORD_TREE_ROOT_SLOT_OF_THE_FIRST_VERSION + version_number,
+                        1,
+                    ),
+                }
+            }
+        }
+    }
+}
+
+/// 根记录里投影之外的四个字段（登记 r2 第一节「投影之外的根字段」）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct FieldsBeyondTheProjection {
+    filesystem_identifier: [u8; 16],
+    unmount_marker: UnmountMarker,
+    instance_table: NodePointer,
+    allocation_record_tree_root: NodePointer,
+}
+
+impl FieldsBeyondTheProjection {
+    fn of_root(root: &RootRecord) -> Self {
+        Self {
+            filesystem_identifier: root.filesystem_identifier,
+            unmount_marker: root.unmount_marker,
+            instance_table: root.instance_table,
+            allocation_record_tree_root: root.allocation_record_tree_root,
+        }
+    }
+
+    /// 见证里的写法：卸载记号、实例表与分配记录树根住的槽（空根是槽 0）、fsid 末字节。
+    fn describe(self) -> String {
+        format!(
+            "marker_{}:instance_table_slot{}:allocation_record_tree_root_slot{}:fsid_last_byte{}",
+            match self.unmount_marker {
+                UnmountMarker::WrittenByTheUnmountSequence => "written",
+                UnmountMarker::NotWrittenByTheUnmountSequence => "not_written",
+            },
+            self.instance_table.locations[0].slot.0,
+            self.allocation_record_tree_root.locations[0].slot.0,
+            self.filesystem_identifier[FILESYSTEM_IDENTIFIER_BYTE_THAT_VARIES_WITH_THE_VERSION]
+        )
+    }
+}
+
+/// 系统配置 fsid 低 8 字节的本地取法：16 字节 fsid 的前 8 字节按小端读（登记 r2 第三节回比表；D22 已定项 13「盘上定长字段一律小端」）。
+/// [`check_the_fields_beyond_the_projection_of_the_r2_generator`] 拿它与 `singlefs_core::unit::unit_filesystem_identifier` 回比。
+fn local_filesystem_identifier_low_eight_bytes(filesystem_identifier: &[u8; 16]) -> u64 {
+    let mut low_eight_bytes = [0u8; 8];
+    low_eight_bytes.copy_from_slice(&filesystem_identifier[..8]);
+    u64::from_le_bytes(low_eight_bytes)
+}
+
 // ── 内存盘（读者）
 
 /// 内存盘某个位置上放着什么：读得出的字节，或一份读不出的（`read` 返回 `None`）。
@@ -160,14 +407,20 @@
 }
 
 /// `replay_journal` 的读者：两块盘；环里没落盘的记录槽读出来是 4096 个 0（登记 5.1），别处没写过的读不出。
+/// `device_identities()` 报哪块盘在前由 `reported_disk_order` 定（登记 r2 第八节「盘次序」）。
 struct MemoryDisk<'bytes> {
     ring: RingGeometry,
+    reported_disk_order: ReportedDiskOrder,
     contents: Vec<(u32, u64, StoredBytes<'bytes>)>,
 }
 
 impl PoolReader for MemoryDisk<'_> {
     fn device_identities(&self) -> Vec<DeviceIdentity> {
-        vec![DeviceIdentity(0), DeviceIdentity(1)]
+        self.reported_disk_order
+            .device_numbers()
+            .into_iter()
+            .map(DeviceIdentity)
+            .collect()
     }
 
     fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
@@ -293,21 +546,32 @@
         }
     }
 
-    /// 这六个字段放进一条根记录；不在新根段里的字段（实例表指针、分配记录树根、卸载记号、fsid）全历史相同，不进判据。
-    fn to_root(self) -> RootRecord {
+    /// 这六个字段放进一条根记录，投影之外的四个字段取 `beyond`。
+    fn to_root_with(self, beyond: FieldsBeyondTheProjection) -> RootRecord {
         RootRecord {
-            filesystem_identifier: ROOT_FILESYSTEM_IDENTIFIER,
-            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
+            filesystem_identifier: beyond.filesystem_identifier,
+            unmount_marker: beyond.unmount_marker,
             instance: InstanceGeneration(self.instance),
             checkpoint_txg: CheckpointTxg(self.checkpoint_txg),
             tree_table: self.segment.tree_table,
             tree_identifier_watermark: self.segment.tree_identifier_watermark,
             rollback_floor: CheckpointTxg(self.segment.rollback_floor),
-            instance_table: node_pointer_at_slot(69_000, 1),
+            instance_table: beyond.instance_table,
             mapping_root: self.segment.mapping_root,
-            allocation_record_tree_root: NodePointer::empty_root(),
+            allocation_record_tree_root: beyond.allocation_record_tree_root,
         }
     }
+
+    /// 由记录重建的根（登记 r2 第一节，S1 的形态）：投影取这六个字段，实例表指针、分配记录树根、fsid 抄 `chosen_root`，
+    /// 卸载记号「不是卸载那一串写下的」。r1 档生成器上它与 r1 时的 `to_root` 逐字段相同（所选根的这四个字段就是那几个常量）。
+    fn to_root_rebuilt_over(self, chosen_root: &RootRecord) -> RootRecord {
+        self.to_root_with(FieldsBeyondTheProjection {
+            filesystem_identifier: chosen_root.filesystem_identifier,
+            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
+            instance_table: chosen_root.instance_table,
+            allocation_record_tree_root: chosen_root.allocation_record_tree_root,
+        })
+    }
 }
 
 // ── 生成一条记录槽
@@ -466,6 +730,7 @@
     chosen_instance_publishes: Vec<PublishShape>,
     next_instance_publishes: Vec<PublishShape>,
     names_units: bool,
+    run_profile: RunProfile,
 }
 
 /// `total` 拆成有序的正整数之和的全部拆法，每一份不超过 `largest_part`（没给就不限）。
@@ -532,6 +797,7 @@
     chosen_instance_publish_count: usize,
     units: Vec<Option<GeneratedUnit>>,
     shape_description: String,
+    run_profile: RunProfile,
 }
 
 fn describe_publish_sequence(publishes: &[PublishShape]) -> String {
@@ -649,6 +915,7 @@
         chosen_instance_publish_count: plan.chosen_instance_publishes.len(),
         units,
         shape_description,
+        run_profile: plan.run_profile,
     }
 }
 
@@ -657,6 +924,14 @@
         self.publishes[version].fields
     }
 
+    /// 时间线第 `version` 版的根 R_j：六个字段照这次发布，投影之外的四个字段照档（登记 r2 第一节）。
+    fn timeline_root(&self, version: usize) -> RootRecord {
+        self.version_fields(version).to_root_with(
+            self.run_profile
+                .fields_beyond_the_projection_of_version(version),
+        )
+    }
+
     fn record_indexes_of_publish(&self, publish_index: usize) -> Range<usize> {
         let publish = &self.publishes[publish_index];
         publish.first_record_index..publish.first_record_index + publish.record_count
@@ -705,9 +980,21 @@
     OnDisk(UnitState),
 }
 
+/// 每条记录在 1 号盘上有没有一份（登记 r2 5.3 多盘域；D22（单元原子性怎么合成） 已定项 8 第 2 条「journal 环在两块盘上互为镜像」）。
+#[derive(Clone, Copy)]
+enum JournalMirroring<'state> {
+    /// 单盘：生成的记录只写 0 号盘（原登记 5.1；R 域的残留另在 1 号盘）。
+    FirstDeviceOnly,
+    /// 两盘镜像：第 r 条记录在 1 号盘同一落点那一份落没落；0 号盘那一份看 `presence`。
+    BothDevices {
+        second_device_copy_is_on_disk: &'state [bool],
+    },
+}
+
 struct LegalState<'state> {
     history: &'state History,
     presence: &'state [RecordPresence],
+    mirroring: JournalMirroring<'state>,
     residue: Option<&'state Residue>,
     chosen_version: usize,
     verify_named_units: bool,
@@ -724,77 +1011,141 @@
     named_units_all_verified: bool,
 }
 
-/// 把一个状态摆上内存盘，并照登记 5.1 建 `records` 表：按盘号升序、逐槽取 `parse` 的结果，读得出的按 (实例, 计数器) 先到先得。
-fn record_table_and_disk_of<'state>(
+/// 直接建表读盘的次序（登记 5.1「按盘号升序」；r2 5.3「先 0 号盘的全部落了的记录（按记录次序），再 1 号盘的」）。
+const DEVICE_ORDER_OF_THE_DIRECT_TABLE: [u32; 2] = [0, 1];
+
+/// 扫盘建表喂给 `scan_journal` 的系统配置里，`scan_journal` 不读的那些字段取的值（它只读 fsid 与 journal 环长，`recovery.rs` 里
+/// `scan_journal` 的函数体）：值随便取、只求造得出一份结构体；不借 mkfs 与写路径（登记 r2 第三节「盘上字节由谁造」、停机条款 SH9）。
+const SCAN_CONFIGURATION_BLOCK_BYTES_THE_SCAN_DOES_NOT_READ: u32 = 4096;
+const SCAN_CONFIGURATION_DEVICE_COUNT: u32 = 2;
+const SCAN_CONFIGURATION_ROOT_RING_BASE_SLOT_THE_SCAN_DOES_NOT_READ: u64 = 0;
+const SCAN_CONFIGURATION_SLOT_GENERATION_THE_SCAN_DOES_NOT_READ: u64 = 1;
+const SCAN_CONFIGURATION_JOURNAL_TAIL_THE_SCAN_DOES_NOT_READ: u64 = 0;
+const SCAN_CONFIGURATION_ROLLBACK_FLOOR_THE_SCAN_DOES_NOT_READ: u64 = 0;
+
+/// 一条记录在一块盘上的一份：槽（字节、`parse` 的结果、头算出的链值）、它点名的单元两份摆在哪（跟着 0 号盘那一份摆）、
+/// 点名单元是不是两份都好（S 的 (k) 要它）。
+struct RecordCopyOnADevice<'state> {
+    slot: &'state GeneratedSlot,
+    named_unit_copies: Option<(u64, StoredBytes<'state>, StoredBytes<'state>)>,
+    named_units_all_verified: bool,
+}
+
+/// 一块盘上落了的记录，按记录次序：0 号盘是 `presence` 里在盘上的那几条；1 号盘是镜像那几份（多盘域），之后是 R 域的残留。
+fn record_copies_on_the_device<'state>(
     state: &LegalState<'state>,
-) -> (RecordTable, MemoryDisk<'state>, Vec<RecordTruthOnDisk>) {
-    let mut records = RecordTable::new();
-    let mut truths: Vec<RecordTruthOnDisk> = Vec::new();
-    let mut contents: Vec<(u32, u64, StoredBytes<'state>)> = Vec::new();
-    for (record_index, slot) in state.history.slots.iter().enumerate() {
-        let RecordPresence::OnDisk(unit_state) = state.presence[record_index] else {
-            continue;
-        };
-        contents.push((
-            slot.device,
-            slot.offset_in_bytes,
-            StoredBytes::Readable(&slot.bytes),
-        ));
-        if let Some(unit) = state.history.units[record_index].as_ref() {
-            let (first_device_copy, second_device_copy) = match unit_state {
-                UnitState::BothCopiesGood => (
-                    StoredBytes::Readable(&unit.good_bytes),
-                    StoredBytes::Readable(&unit.good_bytes),
-                ),
-                UnitState::FirstDeviceCopyUnreadable => (
-                    StoredBytes::Unreadable,
-                    StoredBytes::Readable(&unit.good_bytes),
-                ),
-                UnitState::SecondDeviceCopyUnreadable => (
-                    StoredBytes::Readable(&unit.good_bytes),
-                    StoredBytes::Unreadable,
-                ),
-                UnitState::FirstDeviceCopyChecksumWrong => (
-                    StoredBytes::Readable(&unit.bytes_with_wrong_checksum),
-                    StoredBytes::Readable(&unit.good_bytes),
-                ),
-                UnitState::SecondDeviceCopyChecksumWrong => (
-                    StoredBytes::Readable(&unit.good_bytes),
-                    StoredBytes::Readable(&unit.bytes_with_wrong_checksum),
-                ),
+    device: u32,
+) -> Vec<RecordCopyOnADevice<'state>> {
+    let mut copies = Vec::new();
+    if device == 0 {
+        for (record_index, slot) in state.history.slots.iter().enumerate() {
+            let RecordPresence::OnDisk(unit_state) = state.presence[record_index] else {
+                continue;
             };
-            contents.push((0, unit.offset_in_bytes, first_device_copy));
-            contents.push((1, unit.offset_in_bytes, second_device_copy));
-        }
-        if let Some(parsed) = slot.parsed.as_ref() {
-            records
-                .entry((parsed.instance, parsed.counter))
-                .or_insert_with(|| parsed.clone());
-            truths.push(RecordTruthOnDisk {
-                instance: parsed.instance.0,
-                counter: parsed.counter,
-                chain_value_of_header: slot.chain_value_of_header,
+            let named_unit_copies = state.history.units[record_index].as_ref().map(|unit| {
+                let (first_device_copy, second_device_copy) = match unit_state {
+                    UnitState::BothCopiesGood => (
+                        StoredBytes::Readable(&unit.good_bytes),
+                        StoredBytes::Readable(&unit.good_bytes),
+                    ),
+                    UnitState::FirstDeviceCopyUnreadable => (
+                        StoredBytes::Unreadable,
+                        StoredBytes::Readable(&unit.good_bytes),
+                    ),
+                    UnitState::SecondDeviceCopyUnreadable => (
+                        StoredBytes::Readable(&unit.good_bytes),
+                        StoredBytes::Unreadable,
+                    ),
+                    UnitState::FirstDeviceCopyChecksumWrong => (
+                        StoredBytes::Readable(&unit.bytes_with_wrong_checksum),
+                        StoredBytes::Readable(&unit.good_bytes),
+                    ),
+                    UnitState::SecondDeviceCopyChecksumWrong => (
+                        StoredBytes::Readable(&unit.good_bytes),
+                        StoredBytes::Readable(&unit.bytes_with_wrong_checksum),
+                    ),
+                };
+                (unit.offset_in_bytes, first_device_copy, second_device_copy)
+            });
+            copies.push(RecordCopyOnADevice {
+                slot,
+                named_unit_copies,
                 named_units_all_verified: unit_state == UnitState::BothCopiesGood,
             });
         }
+        return copies;
+    }
+    if let JournalMirroring::BothDevices {
+        second_device_copy_is_on_disk,
+    } = state.mirroring
+    {
+        for (record_index, slot) in state.history.slots.iter().enumerate() {
+            if !second_device_copy_is_on_disk[record_index] {
+                continue;
+            }
+            assert!(
+                state.history.units[record_index].is_none(),
+                "多盘域不点名单元（登记 r2 5.3）：1 号盘那一份没有自己的点名单元摆法"
+            );
+            copies.push(RecordCopyOnADevice {
+                slot,
+                named_unit_copies: None,
+                named_units_all_verified: true,
+            });
+        }
     }
     if let Some(residue) = state.residue {
-        contents.push((
-            residue.slot.device,
-            residue.slot.offset_in_bytes,
-            StoredBytes::Readable(&residue.slot.bytes),
-        ));
-        if let Some(parsed) = residue.slot.parsed.as_ref() {
-            if let btree_map::Entry::Vacant(vacant_place) =
-                records.entry((parsed.instance, parsed.counter))
+        copies.push(RecordCopyOnADevice {
+            slot: &residue.slot,
+            named_unit_copies: None,
+            named_units_all_verified: true,
+        });
+    }
+    copies
+}
+
+/// 把一个状态摆上内存盘，并照登记 5.1 建 `records` 表：按 [`DEVICE_ORDER_OF_THE_DIRECT_TABLE`] 的盘次序、逐槽取 `parse` 的结果，
+/// 读得出的按 (实例, 计数器) 先到先得；`truths` 记先到的那一份。
+fn record_table_and_disk_of<'state>(
+    state: &LegalState<'state>,
+) -> (RecordTable, MemoryDisk<'state>, Vec<RecordTruthOnDisk>) {
+    record_table_and_disk_of_in_device_order(state, DEVICE_ORDER_OF_THE_DIRECT_TABLE)
+}
+
+/// 同 [`record_table_and_disk_of`]，直接建表读盘的次序由 `device_order` 给：登记 r2 第八节盘次序那一点上，直接建表分两栏各比一遍，
+/// 「盘号升序」栏给 [`DEVICE_ORDER_OF_THE_DIRECT_TABLE`]、「照 `device_identities()` 次序」栏给内存盘报的次序。
+fn record_table_and_disk_of_in_device_order<'state>(
+    state: &LegalState<'state>,
+    device_order: [u32; 2],
+) -> (RecordTable, MemoryDisk<'state>, Vec<RecordTruthOnDisk>) {
+    let mut records = RecordTable::new();
+    let mut truths: Vec<RecordTruthOnDisk> = Vec::new();
+    let mut contents: Vec<(u32, u64, StoredBytes<'state>)> = Vec::new();
+    for device in device_order {
+        for copy in record_copies_on_the_device(state, device) {
+            contents.push((
+                device,
+                copy.slot.offset_in_bytes,
+                StoredBytes::Readable(&copy.slot.bytes),
+            ));
+            if let Some((unit_offset_in_bytes, first_device_copy, second_device_copy)) =
+                copy.named_unit_copies
             {
-                vacant_place.insert(parsed.clone());
-                truths.push(RecordTruthOnDisk {
-                    instance: parsed.instance.0,
-                    counter: parsed.counter,
-                    chain_value_of_header: residue.slot.chain_value_of_header,
-                    named_units_all_verified: true,
-                });
+                contents.push((0, unit_offset_in_bytes, first_device_copy));
+                contents.push((1, unit_offset_in_bytes, second_device_copy));
+            }
+            if let Some(parsed) = copy.slot.parsed.as_ref() {
+                if let btree_map::Entry::Vacant(vacant_place) =
+                    records.entry((parsed.instance, parsed.counter))
+                {
+                    vacant_place.insert(parsed.clone());
+                    truths.push(RecordTruthOnDisk {
+                        instance: parsed.instance.0,
+                        counter: parsed.counter,
+                        chain_value_of_header: copy.slot.chain_value_of_header,
+                        named_units_all_verified: copy.named_units_all_verified,
+                    });
+                }
             }
         }
     }
@@ -802,20 +1153,241 @@
         records,
         MemoryDisk {
             ring: state.history.ring,
+            reported_disk_order: state.history.run_profile.reported_disk_order,
             contents,
         },
         truths,
     )
 }
 
-/// 一条记录「完整在盘上」：在 0 号盘上读得出，且（U 域）它点名的单元两份都读得出、校验和都对（登记 5.3）。
+// ── 扫盘建表（登记 r2 第三节「由 scan_journal 建表」、5.4 的 T-表与 T-行）
+
+/// 喂给 `scan_journal` 的系统配置：fsid 取时间线根的那个 16 字节常量（低 8 字节就是记录头里那 8 字节），journal 环长取这个环几何的字节数，
+/// 其余字段 `scan_journal` 不读（[`SCAN_CONFIGURATION_BLOCK_BYTES_THE_SCAN_DOES_NOT_READ`] 一组常量）。
+fn system_configuration_for_the_scan(ring: RingGeometry) -> SystemConfiguration {
+    SystemConfiguration {
+        immutable: SystemImmutableConfiguration {
+            filesystem_identifier: ROOT_FILESYSTEM_IDENTIFIER,
+            this_device: DeviceIdentity(0),
+            device_count: SCAN_CONFIGURATION_DEVICE_COUNT,
+            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
+            sizes: SystemImmutableSizes {
+                physical_block_size: SCAN_CONFIGURATION_BLOCK_BYTES_THE_SCAN_DOES_NOT_READ,
+                minimum_input_output_bytes: SCAN_CONFIGURATION_BLOCK_BYTES_THE_SCAN_DOES_NOT_READ,
+                fixed_structure_slot_spacing: SCAN_CONFIGURATION_BLOCK_BYTES_THE_SCAN_DOES_NOT_READ,
+                journal_ring_bytes: ring.ring_bytes(),
+                root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
+            },
+            journal_ring_start_slot: SlotNumber(LOCAL_JOURNAL_RING_START_SLOT),
+            root_ring_base_slot: SlotNumber(
+                SCAN_CONFIGURATION_ROOT_RING_BASE_SLOT_THE_SCAN_DOES_NOT_READ,
+            ),
+        },
+        mutable: SystemMutableConfiguration,
+        runtime: SystemRuntimeConfiguration,
+        quantities: SystemRuntimeQuantities {
+            slot_generation: SCAN_CONFIGURATION_SLOT_GENERATION_THE_SCAN_DOES_NOT_READ,
+            journal_tail: SCAN_CONFIGURATION_JOURNAL_TAIL_THE_SCAN_DOES_NOT_READ,
+            journal_instance: InstanceGeneration(CHOSEN_INSTANCE),
+            rollback_floor: CheckpointTxg(SCAN_CONFIGURATION_ROLLBACK_FLOOR_THE_SCAN_DOES_NOT_READ),
+        },
+    }
+}
+
+/// 扫盘建表：对摆好的内存盘调 `scan_journal`。`implementation` 是 T-表那条对照时，建完把键最大的那条记录删掉（假结局：扫盘建的表少一条）。
+fn scanned_record_table_of(disk: &MemoryDisk<'_>, implementation: Implementation) -> RecordTable {
+    let mut table = scan_journal(disk, &system_configuration_for_the_scan(disk.ring));
+    if implementation == Implementation::Control(ControlKind::ScanTableLosesItsLargestKeyRecord) {
+        table.pop_last();
+    }
+    table
+}
+
+/// T-表：同一个状态上两张表逐键逐值比，有一处不同即不同（登记 r2 5.4）。
+fn the_two_record_tables_differ(direct_table: &RecordTable, scanned_table: &RecordTable) -> bool {
+    direct_table != scanned_table
+}
+
+/// 两张表差在哪：只在直接建的表里的键、只在扫盘建的表里的键、两边都有而记录不同的键（键写成 `实例:计数器`）。
+fn describe_how_the_two_record_tables_differ(
+    direct_table: &RecordTable,
+    scanned_table: &RecordTable,
+) -> String {
+    let key_text = |key: &(InstanceGeneration, u64)| format!("{}:{}", key.0 .0, key.1);
+    let keys_of = |keys: Vec<String>| {
+        if keys.is_empty() {
+            "none".to_string()
+        } else {
+            keys.join("+")
+        }
+    };
+    let only_in_the_direct_table = keys_of(
+        direct_table
+            .keys()
+            .filter(|key| !scanned_table.contains_key(*key))
+            .map(key_text)
+            .collect(),
+    );
+    let only_in_the_scanned_table = keys_of(
+        scanned_table
+            .keys()
+            .filter(|key| !direct_table.contains_key(*key))
+            .map(key_text)
+            .collect(),
+    );
+    let with_different_records = keys_of(
+        direct_table
+            .iter()
+            .filter(|(key, record)| {
+                scanned_table
+                    .get(*key)
+                    .is_some_and(|scanned_record| scanned_record != *record)
+            })
+            .map(|(key, _record)| key_text(key))
+            .collect(),
+    );
+    format!(
+        "direct_table_entries={};scanned_table_entries={};only_in_the_direct_table={only_in_the_direct_table};only_in_the_scanned_table={only_in_the_scanned_table};same_key_different_record={with_different_records}",
+        direct_table.len(),
+        scanned_table.len()
+    )
+}
+
+/// 一个状态在一种建表法下判完的小结（登记 r2 5.4 T-行要比的两样）：第一次调用的结局，这个状态让哪几个计数器加了一。
+struct JudgedRowSummary {
+    first_outcome: ReplayOutcome,
+    counter_labels: String,
+}
+
+/// T-行：两遍的实现结局（`describe_outcome` 那一串）与这个状态加了一的计数器标签集合，有一项不同即不同（登记 r2 5.4）。
+fn the_two_judged_rows_differ(direct: &JudgedRowSummary, scanned: &JudgedRowSummary) -> bool {
+    describe_outcome(&direct.first_outcome) != describe_outcome(&scanned.first_outcome)
+        || direct.counter_labels != scanned.counter_labels
+}
+
+/// 两张表各判一遍之后逐态比，比出来不同的记进 `tally` 的 `tables_differ_counter`、`rows_differ_counter`，各记第一个见证（状态的写法 + 差在哪）。
+fn add_the_comparison_of_two_judged_tables(
+    tally: &mut Tally,
+    (tables_differ_counter, rows_differ_counter): (Counter, Counter),
+    direct: (&RecordTable, &JudgedRowSummary),
+    scanned: (&RecordTable, &JudgedRowSummary),
+    describe_state: &dyn Fn() -> String,
+) {
+    let (direct_table, direct_summary) = direct;
+    let (scanned_table, scanned_summary) = scanned;
+    if the_two_record_tables_differ(direct_table, scanned_table) {
+        tally.add_violation(tables_differ_counter, &|| {
+            format!(
+                "{};{}",
+                describe_state(),
+                describe_how_the_two_record_tables_differ(direct_table, scanned_table)
+            )
+        });
+    }
+    if the_two_judged_rows_differ(direct_summary, scanned_summary) {
+        tally.add_violation(rows_differ_counter, &|| {
+            format!(
+                "{};direct_outcome={};scanned_outcome={};direct_counters={};scanned_counters={}",
+                describe_state(),
+                describe_outcome(&direct_summary.first_outcome),
+                describe_outcome(&scanned_summary.first_outcome),
+                direct_summary.counter_labels,
+                scanned_summary.counter_labels
+            )
+        });
+    }
+}
+
+/// 扫盘那一遍用哪种实现、算不算指定状态：T-行那条对照把扫盘那一遍的结局换成 panic（表不动），别的情形与直接建表那一遍相同。
+fn implementation_and_designation_of_the_scan_pass(
+    implementation: Implementation,
+    is_the_designated_state: bool,
+) -> (Implementation, bool) {
+    match implementation {
+        Implementation::Control(ControlKind::ScanPassOutcomeBecomesAPanic) => (
+            Implementation::Control(ControlKind::PanicsOnTheDesignatedState),
+            true,
+        ),
+        Implementation::TodayReplayJournal | Implementation::Control(_) => {
+            (implementation, is_the_designated_state)
+        }
+    }
+}
+
+/// 一个状态判完之后「这个状态让哪几个计数器加了一」：对着判之前的计数。
+fn counter_labels_a_state_added_to(tally: &Tally, counts_before: &[u64]) -> String {
+    labels_of_the_counters_a_state_added(counts_before, &tally.counts)
+}
+
+/// 每条记录在每块盘上都落了（登记 r2 第七节甲类「全部落盘那一态」）：0 号盘上都在，多盘域 1 号盘上的镜像也都在。
+fn every_record_is_landed_on_every_device(state: &LegalState<'_>) -> bool {
+    state
+        .presence
+        .iter()
+        .all(|record_presence| *record_presence != RecordPresence::NotOnDisk)
+        && (0..state.presence.len()).all(|record_index| match state.mirroring {
+            JournalMirroring::FirstDeviceOnly => true,
+            JournalMirroring::BothDevices {
+                second_device_copy_is_on_disk,
+            } => second_device_copy_is_on_disk[record_index],
+        })
+}
+
+/// 登记 r2 第七节甲类：每条记录在每块盘上都落了的那一态，扫盘表的条数等于这一态生成的记录条数（R 域的残留与 0 号盘那一份同键，后到不进表）；
+/// 不符走 SF9（停下，两边都查）。只在今天的实现上断言：T-表那条对照故意删一条。
+fn assert_the_scan_table_of_a_fully_landed_state(
+    state: &LegalState<'_>,
+    scanned_table: &RecordTable,
+    implementation: Implementation,
+    tally: &mut Tally,
+) {
+    if implementation != Implementation::TodayReplayJournal
+        || !every_record_is_landed_on_every_device(state)
+    {
+        return;
+    }
+    tally.add(Counter::FullyLandedScanTableSizeChecked);
+    assert_eq!(
+        scanned_table.len(),
+        state.history.slots.len(),
+        "停机条款 SF9：每条记录在每块盘上都落了的那一态，扫盘表的条数不等于生成的记录条数（形状 {}）",
+        state.history.shape_description
+    );
+}
+
+/// 一个状态判完的结果：返回给 C、MC 域状态对要用的施加区间，加上第一次调用的结局与它落到的最远版本（逐态行的写法要）。
+struct LegalJudgement {
+    interval: Option<AppliedInterval>,
+    first_outcome: ReplayOutcome,
+    farthest_version: usize,
+}
+
+/// 一条记录「完整在盘上」：在 0 号盘上读得出，且（U 域）它点名的单元两份都读得出、校验和都对（登记 5.3）；
+/// 多盘域任一份在即在（登记 r2 5.3，D23（journal 的角色与格式） 已定项 14「两份镜像何时算「记录在」：任一份自证校验和过即在」）。
 fn record_is_completely_on_disk(state: &LegalState<'_>, record_index: usize) -> bool {
-    match state.presence[record_index] {
+    let first_device_copy_is_complete = match state.presence[record_index] {
         RecordPresence::NotOnDisk => false,
         RecordPresence::OnDisk(unit_state) => unit_state == UnitState::BothCopiesGood,
+    };
+    first_device_copy_is_complete || second_device_copy_is_on_disk(state, record_index)
+}
+
+/// 1 号盘上那一份落没落；单盘域恒为假。
+fn second_device_copy_is_on_disk(state: &LegalState<'_>, record_index: usize) -> bool {
+    match state.mirroring {
+        JournalMirroring::FirstDeviceOnly => false,
+        JournalMirroring::BothDevices {
+            second_device_copy_is_on_disk,
+        } => second_device_copy_is_on_disk[record_index],
     }
 }
 
+/// 这条记录有没有一份落了盘（不看点名单元）。
+fn record_has_a_copy_on_disk(state: &LegalState<'_>, record_index: usize) -> bool {
+    state.presence[record_index] != RecordPresence::NotOnDisk
+        || second_device_copy_is_on_disk(state, record_index)
+}
+
 fn publish_is_completely_on_disk(state: &LegalState<'_>, publish_index: usize) -> bool {
     state
         .history
@@ -827,7 +1399,7 @@
     state
         .history
         .record_indexes_of_publish(publish_index)
-        .any(|record_index| state.presence[record_index] != RecordPresence::NotOnDisk)
+        .any(|record_index| record_has_a_copy_on_disk(state, record_index))
 }
 
 /// j\*（登记 5.3 的 J4）：最大的 j ≥ c，使 (c, j] 里每次发布都完整在盘上、且这些发布的记录条数之和不超过在飞上限；
@@ -1252,7 +1824,7 @@
             let root = if fields == VersionFields::of_root(chosen_root) {
                 *chosen_root
             } else {
-                fields.to_root()
+                fields.to_root_rebuilt_over(chosen_root)
             };
             ReplayOutcome::Returned(ReturnedReplay {
                 root,
@@ -1338,6 +1910,25 @@
         LimitViolations => "A_limit",
         SpecificationExpectsFailure => "specification_expects_failure",
         StatesHoldingARecordRejectedByParse => "states_holding_a_record_rejected_by_parse",
+        J6QnPremiseHoldsJudged => "j6qn_premise_holds_judged",
+        J6QnPremiseHoldsViolations => "J6Qn_premise_holds",
+        J6QnPremiseFailsJudged => "j6qn_premise_fails_judged",
+        J6QnPremiseFailsViolations => "J6Qn_premise_fails",
+        FirstReturnedRootTxgIsTheLargest => "first_returned_root_txg_is_the_largest",
+        J7Judged => "j7_judged",
+        J7aInstanceTableViolations => "J7a_instance_table",
+        J7aAllocationRecordTreeRootViolations => "J7a_allocation_record_tree_root",
+        J7aFilesystemIdentifierViolations => "J7a_filesystem_identifier",
+        J7bJudged => "j7b_judged",
+        J7bViolations => "J7b",
+        J7cJudged => "j7c_judged",
+        J7cViolations => "J7c",
+        TablesDiffer => "tables_differ",
+        RowsDiffer => "rows_differ",
+        TablesDifferInReportedOrder => "tables_differ_in_reported_order",
+        RowsDifferInReportedOrder => "rows_differ_in_reported_order",
+        DirectTableHoldsARecord => "direct_table_holds_a_record",
+        FullyLandedScanTableSizeChecked => "fully_landed_scan_table_size_checked",
     }
 }
 
@@ -1367,12 +1958,54 @@
     }
 }
 
+labelled_enumeration! {
+    /// 多盘 M、M6 一个状态两块盘落盘集合的关系（登记 r2 5.3「落后类别」；SQ17 把落后按方向拆两行）。
+    enum LaggingClass {
+        Equal => "equal",
+        FirstDeviceBehind => "first_device_behind",
+        SecondDeviceBehind => "second_device_behind",
+        Crossed => "crossed",
+    }
+}
+
+/// 落后类别各自数的量（登记 r2 SQ9、SQ17：状态数与 J1、J2、J4、J6、J6Qn、J7 的违例数，各行照原样分开）。
+const LAGGING_CLASS_COLUMNS: [Counter; 13] = [
+    Counter::States,
+    Counter::J1Violations,
+    Counter::J2Violations,
+    Counter::J4Violations,
+    Counter::J6PremiseHoldsViolations,
+    Counter::J6PremiseFailsViolations,
+    Counter::J6QnPremiseHoldsViolations,
+    Counter::J6QnPremiseFailsViolations,
+    Counter::J7aInstanceTableViolations,
+    Counter::J7aAllocationRecordTreeRootViolations,
+    Counter::J7aFilesystemIdentifierViolations,
+    Counter::J7bViolations,
+    Counter::J7cViolations,
+];
+
+/// 两块盘的落盘集合（按记录下标置位）：相等；0 号盘的真包含于 1 号盘的是 0 号盘落后；反过来是 1 号盘落后；其余互不包含。
+fn lagging_class_of(first_device_mask: u32, second_device_mask: u32) -> LaggingClass {
+    if first_device_mask == second_device_mask {
+        LaggingClass::Equal
+    } else if first_device_mask & second_device_mask == first_device_mask {
+        LaggingClass::FirstDeviceBehind
+    } else if first_device_mask & second_device_mask == second_device_mask {
+        LaggingClass::SecondDeviceBehind
+    } else {
+        LaggingClass::Crossed
+    }
+}
+
 #[derive(Clone, Debug)]
 struct Tally {
     counts: Vec<u64>,
     witnesses: Vec<Option<String>>,
     pair_counts: Vec<u64>,
     above_water_maximum: usize,
+    /// 下标 = 落后类别的下标 × [`LAGGING_CLASS_COLUMNS`] 的列数 + 列下标；单盘域恒为 0。
+    lagging_class_counts: Vec<u64>,
 }
 
 impl Tally {
@@ -1382,9 +2015,27 @@
             witnesses: vec![None; Counter::ALL.len()],
             pair_counts: vec![0; PairSet::ALL.len() * PairColumn::ALL.len()],
             above_water_maximum: 0,
+            lagging_class_counts: vec![0; LaggingClass::ALL.len() * LAGGING_CLASS_COLUMNS.len()],
         }
     }
 
+    /// 一个多盘状态判完：把它让 [`LAGGING_CLASS_COLUMNS`] 各加的数记进它的落后类别（`counts_before` 是判这个状态之前的计数）。
+    fn add_to_the_lagging_class(&mut self, class: LaggingClass, counts_before: &[u64]) {
+        for (column_index, counter) in LAGGING_CLASS_COLUMNS.iter().enumerate() {
+            let added = self.counts[counter.index()] - counts_before[counter.index()];
+            self.lagging_class_counts
+                [class.index() * LAGGING_CLASS_COLUMNS.len() + column_index] += added;
+        }
+    }
+
+    fn lagging_class_count(&self, class: LaggingClass, counter: Counter) -> u64 {
+        let column_index = LAGGING_CLASS_COLUMNS
+            .iter()
+            .position(|column| *column == counter)
+            .expect("只问 LAGGING_CLASS_COLUMNS 里列了的量");
+        self.lagging_class_counts[class.index() * LAGGING_CLASS_COLUMNS.len() + column_index]
+    }
+
     fn add(&mut self, counter: Counter) {
         self.counts[counter.index()] += 1;
     }
@@ -1422,6 +2073,13 @@
         for (count, later_count) in self.pair_counts.iter_mut().zip(&later.pair_counts) {
             *count += later_count;
         }
+        for (count, later_count) in self
+            .lagging_class_counts
+            .iter_mut()
+            .zip(&later.lagging_class_counts)
+        {
+            *count += later_count;
+        }
         for (witness, later_witness) in self.witnesses.iter_mut().zip(&later.witnesses) {
             if witness.is_none() {
                 witness.clone_from(later_witness);
@@ -1619,20 +2277,27 @@
 }
 
 /// 探针模式的主流程：不跑自检与阳性对照，单线程把这个 (域, N) 照工作单元的次序走一遍。
-fn run_the_state_probe(domain: Domain, size: usize, probe: StateProbe) {
+fn run_the_state_probe(domain: Domain, size: usize, probe: StateProbe, run_profile: RunProfile) {
     install_the_panic_hook_that_stays_quiet_inside_the_replay_call();
     println!(
-        "E7RESULT name=config experiment=E165 mode={} domain={} ring_slots={} n={size} self_checks=not_run positive_controls=not_run threads=1",
+        "E7RESULT name=config experiment=E165 mode={} domain={} ring_slots={} n={size} self_checks=not_run positive_controls=not_run threads=1{}",
         match probe {
             StateProbe::ReplayOneState { .. } => "replay_state",
             StateProbe::ListEveryState => "list_states",
             StateProbe::CaptureOneStateRow { .. } => "capture_state",
         },
         domain.label(),
-        domain.ring().ring_slots
+        domain.ring().ring_slots,
+        profile_fields_of_the_run_rows(run_profile)
     );
     set_the_state_probe(domain, size, probe);
-    let tally = tally_of_a_domain_size(domain, size, Implementation::TodayReplayJournal, 1);
+    let tally = tally_of_a_domain_size(
+        domain,
+        size,
+        Implementation::TodayReplayJournal,
+        run_profile,
+        1,
+    );
     let walked = NEXT_STATE_ORDINAL.load(Ordering::Relaxed);
     let captured_state_row = CAPTURED_STATE_ROW
         .lock()
@@ -1677,8 +2342,87 @@
 
 // ── 钉住的红例逐行重放（`.claude/gate.d/e165-pinned-red-examples.tsv` 文件头的约定）：只按 ②③④ 找状态、重算 ①，不改任何判据
 
-/// 钉住的表一行有几段（① 输入指纹、② 域、③ N、④ 序号、⑤ 出处、⑥ 变异、⑦ 原实现的逐态行、⑧ 变体的逐态行）。
-const PINNED_TABLE_COLUMN_COUNT: usize = 8;
+/// 钉住的表自己的段数（① 输入指纹、② 域、③ N、④ 序号、⑤ 出处、⑥ 变异、⑦ 原实现的逐态行、⑧ 变体的逐态行）。
+const PINNED_TABLE_OWN_COLUMN_COUNT: usize = 8;
+
+/// 红例账（`research/results/e165-*-red-examples.tsv`）一行的段数，列定义在账的表头。
+const RED_EXAMPLE_LEDGER_COLUMN_COUNT: usize = 13;
+
+/// 钉住的表一行有几段：自己的八段后面接红例账出红的那一行原样（⑨–㉑），⑨ 起各段用制表符接起来与账那一行逐字节相同。
+const PINNED_TABLE_COLUMN_COUNT: usize =
+    PINNED_TABLE_OWN_COLUMN_COUNT + RED_EXAMPLE_LEDGER_COLUMN_COUNT;
+
+/// 钉住的表的段号写法，下标是段号减一。
+const PINNED_TABLE_COLUMN_MARKS: [&str; PINNED_TABLE_COLUMN_COUNT] = [
+    "①", "②", "③", "④", "⑤", "⑥", "⑦", "⑧", "⑨", "⑩", "⑪", "⑫", "⑬", "⑭", "⑮", "⑯", "⑰", "⑱", "⑲",
+    "⑳", "㉑",
+];
+
+/// 抄来的账行里的一段与钉住的表自己的一段必须逐字相同：段号都从 1 数，账的段号按账自己数（钉表里是它加八）。
+struct LedgerColumnThatMustEqualAPinnedColumn {
+    ledger_column_number: usize,
+    pinned_column_number: usize,
+}
+
+/// 按红例账表头的列定义：第 3、4、5 段是域、N、枚举次序里的序号，对 ②③④；第 12、13 段是原实现与变体的逐态行，对 ⑦⑧。
+const LEDGER_COLUMNS_THAT_MUST_EQUAL_A_PINNED_COLUMN: [LedgerColumnThatMustEqualAPinnedColumn; 5] = [
+    LedgerColumnThatMustEqualAPinnedColumn {
+        ledger_column_number: 3,
+        pinned_column_number: 2,
+    },
+    LedgerColumnThatMustEqualAPinnedColumn {
+        ledger_column_number: 4,
+        pinned_column_number: 3,
+    },
+    LedgerColumnThatMustEqualAPinnedColumn {
+        ledger_column_number: 5,
+        pinned_column_number: 4,
+    },
+    LedgerColumnThatMustEqualAPinnedColumn {
+        ledger_column_number: 12,
+        pinned_column_number: 7,
+    },
+    LedgerColumnThatMustEqualAPinnedColumn {
+        ledger_column_number: 13,
+        pinned_column_number: 8,
+    },
+];
+
+/// 段数已经是 [`PINNED_TABLE_COLUMN_COUNT`] 的一行：抄来的账行（⑨ 起）逐对对着钉表自己的段核，不同的每一对都报出来、点名是哪一段。
+fn check_that_the_copied_ledger_row_agrees_with_the_pinned_columns(
+    columns: &[&str],
+) -> Result<(), String> {
+    assert_eq!(
+        columns.len(),
+        PINNED_TABLE_COLUMN_COUNT,
+        "调用方 parse_a_pinned_red_example 先核过段数"
+    );
+    let disagreements: Vec<String> = LEDGER_COLUMNS_THAT_MUST_EQUAL_A_PINNED_COLUMN
+        .iter()
+        .filter_map(|pair| {
+            let ledger_column_index_in_the_table =
+                PINNED_TABLE_OWN_COLUMN_COUNT + pair.ledger_column_number - 1;
+            let pinned_column_index = pair.pinned_column_number - 1;
+            let ledger_value = columns[ledger_column_index_in_the_table];
+            let pinned_value = columns[pinned_column_index];
+            (ledger_value != pinned_value).then(|| {
+                format!(
+                    "{}（红例账第 {} 段）与 {} 不同：{} 是「{ledger_value}」，{} 是「{pinned_value}」",
+                    PINNED_TABLE_COLUMN_MARKS[ledger_column_index_in_the_table],
+                    pair.ledger_column_number,
+                    PINNED_TABLE_COLUMN_MARKS[pinned_column_index],
+                    PINNED_TABLE_COLUMN_MARKS[ledger_column_index_in_the_table],
+                    PINNED_TABLE_COLUMN_MARKS[pinned_column_index],
+                )
+            })
+        })
+        .collect();
+    if disagreements.is_empty() {
+        Ok(())
+    } else {
+        Err(disagreements.join("；"))
+    }
+}
 
 /// 钉住的表里的一行数据：① 与 ②③ 留原文（重算 ① 用原文），②③④ 另解出值来找状态。
 #[derive(Clone, Debug, PartialEq, Eq)]
@@ -1692,18 +2436,25 @@
     ordinal: u64,
 }
 
-/// 表里一行数据 → [`PinnedRedExample`]：段数不是八、域不认识、N 不在这个域的范围、序号不是非负整数，都拒。
+/// 表里一行数据 → [`PinnedRedExample`]：段数不是二十一（八段的旧行另报「没整行抄红例账」）、抄来的账行与 ②③④⑦⑧ 对不上、
+/// 域不认识、N 不在这个域的范围、序号不是非负整数，都拒。
 fn parse_a_pinned_red_example(
     line_number_in_the_table: usize,
     line: &str,
 ) -> Result<PinnedRedExample, String> {
     let columns: Vec<&str> = line.split('\t').collect();
+    if columns.len() == PINNED_TABLE_OWN_COLUMN_COUNT {
+        return Err(format!(
+            "这一行没整行抄红例账，用 research/scripts/e165-pin-red-examples.py 补（这一行是 {PINNED_TABLE_OWN_COLUMN_COUNT} 段的旧格式，要 {PINNED_TABLE_COLUMN_COUNT} 段）"
+        ));
+    }
     if columns.len() != PINNED_TABLE_COLUMN_COUNT {
         return Err(format!(
-            "要 {PINNED_TABLE_COLUMN_COUNT} 段（制表符分隔），这一行是 {} 段",
+            "要 {PINNED_TABLE_COLUMN_COUNT} 段（制表符分隔：自己的 {PINNED_TABLE_OWN_COLUMN_COUNT} 段后面接红例账一行的 {RED_EXAMPLE_LEDGER_COLUMN_COUNT} 段），这一行是 {} 段",
             columns.len()
         ));
     }
+    check_that_the_copied_ledger_row_agrees_with_the_pinned_columns(&columns)?;
     let (domain, size) = domain_and_size_of(columns[1], columns[2])?;
     let ordinal = columns[3]
         .parse::<u64>()
@@ -1774,7 +2525,10 @@
 }
 
 /// 在今天的实现上重放一行：设记下模式的探针，单线程把这个 (域, N) 走一遍，交回记下的那一行与走过的状态数。
-fn replay_a_pinned_red_example(example: &PinnedRedExample) -> (Option<String>, u64) {
+fn replay_a_pinned_red_example(
+    example: &PinnedRedExample,
+    run_profile: RunProfile,
+) -> (Option<String>, u64) {
     set_the_state_probe(
         example.domain,
         example.size,
@@ -1786,6 +2540,7 @@
         example.domain,
         example.size,
         Implementation::TodayReplayJournal,
+        run_profile,
         1,
     );
     let states_walked = NEXT_STATE_ORDINAL.load(Ordering::Relaxed);
@@ -1796,8 +2551,40 @@
     (captured_state_row, states_walked)
 }
 
+/// 钉住的表全文解出来的样子：数据行（非空、不以 `#` 起头）有几行，解得出的各行（按表里的次序），解不出的各行的问题（带行号）。
+#[derive(Debug)]
+struct PinnedTableParse {
+    data_line_count: usize,
+    examples: Vec<PinnedRedExample>,
+    problems_of_the_rows_that_do_not_parse: Vec<String>,
+}
+
+/// 钉住的表全文 → [`PinnedTableParse`]：纯函数，不碰探针、不重放。
+fn parse_the_pinned_table(table_text: &str) -> PinnedTableParse {
+    let mut parse = PinnedTableParse {
+        data_line_count: 0,
+        examples: Vec::new(),
+        problems_of_the_rows_that_do_not_parse: Vec::new(),
+    };
+    let data_lines = table_text
+        .lines()
+        .enumerate()
+        .map(|(line_index, line)| (line_index + 1, line))
+        .filter(|(_line_number, line)| !line.trim().is_empty() && !line.starts_with('#'));
+    for (line_number, line) in data_lines {
+        parse.data_line_count += 1;
+        match parse_a_pinned_red_example(line_number, line) {
+            Ok(example) => parse.examples.push(example),
+            Err(problem) => parse
+                .problems_of_the_rows_that_do_not_parse
+                .push(format!("第 {line_number} 行：{problem}")),
+        }
+    }
+    parse
+}
+
 /// `--pinned-cases <表>` 的主流程：按数据行的次序每行打一行 `name=state`；有行不对、找不到状态或指纹漂了，全部报完退 1。
-fn replay_every_pinned_red_example(table_path: &str) {
+fn replay_every_pinned_red_example(table_path: &str, run_profile: RunProfile) {
     let table_text = match fs::read_to_string(table_path) {
         Ok(table_text) => table_text,
         Err(error) => {
@@ -1807,30 +2594,23 @@
         }
     };
     install_the_panic_hook_that_stays_quiet_inside_the_replay_call();
-    let data_lines: Vec<(usize, &str)> = table_text
-        .lines()
-        .enumerate()
-        .map(|(line_index, line)| (line_index + 1, line))
-        .filter(|(_line_number, line)| !line.trim().is_empty() && !line.starts_with('#'))
-        .collect();
+    let PinnedTableParse {
+        data_line_count,
+        examples,
+        problems_of_the_rows_that_do_not_parse,
+    } = parse_the_pinned_table(&table_text);
     println!(
-        "E7RESULT name=config experiment=E165 mode=pinned_cases rows={} self_checks=not_run positive_controls=not_run threads=1",
-        data_lines.len()
+        "E7RESULT name=config experiment=E165 mode=pinned_cases rows={data_line_count} self_checks=not_run positive_controls=not_run threads=1"
     );
     let mut problems: Vec<String> = Vec::new();
-    if data_lines.is_empty() {
+    if data_line_count == 0 {
         problems.push(format!("{table_path} 一行数据都没有"));
     }
+    problems.extend(problems_of_the_rows_that_do_not_parse);
     let mut state_rows_printed: u64 = 0;
-    for (line_number, line) in data_lines {
-        let example = match parse_a_pinned_red_example(line_number, line) {
-            Ok(example) => example,
-            Err(problem) => {
-                problems.push(format!("第 {line_number} 行：{problem}"));
-                continue;
-            }
-        };
-        let (captured_state_row, states_walked) = replay_a_pinned_red_example(&example);
+    for example in examples {
+        let (captured_state_row, states_walked) =
+            replay_a_pinned_red_example(&example, run_profile);
         let place = format!(
             "第 {} 行（{}:{}:{}）",
             example.line_number_in_the_table,
@@ -1873,7 +2653,7 @@
             eprintln!("  ✗ {problem}");
         }
         eprintln!(
-            "  → 怎么办：指纹漂了先查装置的枚举次序与状态编码变没变；行不对的照表头的八段格式找回原行（表只由 research/scripts/e165-pin-red-examples.py 追加）"
+            "  → 怎么办：指纹漂了先查装置的枚举次序与状态编码变没变；行不对的照表头的二十一段格式（自己的八段后面接红例账出红的那一行原样）找回原行，八段的旧行用 research/scripts/e165-pin-red-examples.py 补整行（表只由它写）"
         );
         std::process::exit(1);
     }
@@ -1945,6 +2725,67 @@
     ReportsNoFailureOnTheLargestTxg,
     /// IA2：关掉点名验证时 `verification_passed` 报成 1。
     ReportsOneVerificationWithoutVerifying,
+    /// J6Qn「(i)(ii) 之外都算违例」：第一次照真结局，第二次换成「报 `RootPublishCarriesMoreThanOneLastRecordFlag…`」。
+    SecondCallReportsMoreThanOneLastRecordFlag,
+    /// J6Qn「(ii) 要核 txg 顶格」：第一次照真结局，第二次换成「报 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`」。
+    SecondCallReportsTheChosenRootTxgHasNoSuccessor,
+    /// J6 与 J6Qn 分得开：第一次取理想恢复（落到 j\*），第二次照真调用。
+    IdealFirstCallThenTodaysSecondCall,
+    /// J7a：理想恢复，施加过时把投影之外的一个字段换成落到的那一版 R_j 的。
+    IdealWithAFieldOfTheLandedVersion(FieldOfTheRebuiltRoot),
+    /// J7b：理想恢复，施加过时卸载记号抄所选根的。
+    IdealCopyingTheChosenUnmountMarkerWhenApplied,
+    /// J7c：理想恢复，一条没施加时卸载记号写成 `NotWritten…`。
+    IdealWritingNotWrittenWhenNothingIsApplied,
+    /// 多盘「任一份在即在」（登记 r2 5.5）：理想恢复，但 j\* 只数 0 号盘那一份。
+    IdealCountingOnlyTheFirstDeviceCopies,
+    /// T-表（登记 r2 5.5）：扫盘建表之后删掉表里键最大的那条记录，直接建的表不动。
+    ScanTableLosesItsLargestKeyRecord,
+    /// T-行（登记 r2 5.5）：扫盘建表那一遍的实现结局换成 `Panicked`，表不动。
+    ScanPassOutcomeBecomesAPanic,
+}
+
+/// J7a 判的三个字段（登记 r2 第一节）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum FieldOfTheRebuiltRoot {
+    InstanceTable,
+    AllocationRecordTreeRoot,
+    FilesystemIdentifier,
+}
+
+impl FieldOfTheRebuiltRoot {
+    const fn label(self) -> &'static str {
+        match self {
+            Self::InstanceTable => "instance_table",
+            Self::AllocationRecordTreeRoot => "allocation_record_tree_root",
+            Self::FilesystemIdentifier => "filesystem_identifier",
+        }
+    }
+
+    const ALL: [Self; 3] = [
+        Self::InstanceTable,
+        Self::AllocationRecordTreeRoot,
+        Self::FilesystemIdentifier,
+    ];
+
+    const fn violations_counter(self) -> Counter {
+        match self {
+            Self::InstanceTable => Counter::J7aInstanceTableViolations,
+            Self::AllocationRecordTreeRoot => Counter::J7aAllocationRecordTreeRootViolations,
+            Self::FilesystemIdentifier => Counter::J7aFilesystemIdentifierViolations,
+        }
+    }
+
+    /// 把 `root` 的这个字段换成 `source` 的。
+    fn copy_from(self, root: &mut RootRecord, source: &RootRecord) {
+        match self {
+            Self::InstanceTable => root.instance_table = source.instance_table,
+            Self::AllocationRecordTreeRoot => {
+                root.allocation_record_tree_root = source.allocation_record_tree_root;
+            }
+            Self::FilesystemIdentifier => root.filesystem_identifier = source.filesystem_identifier,
+        }
+    }
 }
 
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
@@ -2005,7 +2846,10 @@
         root: if version == state.chosen_version {
             *chosen_root
         } else {
-            state.history.version_fields(version).to_root()
+            state
+                .history
+                .version_fields(version)
+                .to_root_rebuilt_over(chosen_root)
         },
         prefix_applied,
         verification_passed: if state.verify_named_units {
@@ -2016,14 +2860,18 @@
     }
 }
 
-/// 一条根，六个字段都不等于时间线上任何一版。
-fn off_timeline_root(checkpoint_txg: u64, distinguishing_number: u64) -> RootRecord {
+/// 一条根，六个字段都不等于时间线上任何一版；投影之外照「由记录重建」抄 `rebuilt_over`。
+fn off_timeline_root(
+    checkpoint_txg: u64,
+    distinguishing_number: u64,
+    rebuilt_over: &RootRecord,
+) -> RootRecord {
     VersionFields {
         instance: CHOSEN_INSTANCE,
         checkpoint_txg,
         segment: NewRootSegment::off_timeline(distinguishing_number, checkpoint_txg),
     }
-    .to_root()
+    .to_root_rebuilt_over(rebuilt_over)
 }
 
 /// 合法历史一族里第一次调用的结局：今天的实现，或阳性对照的假结局。
@@ -2053,7 +2901,7 @@
         ControlKind::OffTimelineRootWheneverApplied => {
             let mut returned = ideal();
             if truth.farthest_version > state.chosen_version {
-                returned.root = off_timeline_root(returned.root.checkpoint_txg.0, 1);
+                returned.root = off_timeline_root(returned.root.checkpoint_txg.0, 1, chosen_root);
             }
             ReplayOutcome::Returned(returned)
         }
@@ -2083,10 +2931,8 @@
             state.chosen_version,
         )),
         ControlKind::ReturnsTheChosenRootWhenEveryRecordIsOnDisk => {
-            let every_record_is_on_disk = state
-                .presence
-                .iter()
-                .all(|presence| *presence != RecordPresence::NotOnDisk);
+            let every_record_is_on_disk = (0..state.presence.len())
+                .all(|record_index| record_has_a_copy_on_disk(state, record_index));
             ReplayOutcome::Returned(returned_at_version(
                 state,
                 chosen_root,
@@ -2113,10 +2959,54 @@
             }
             outcome @ (ReplayOutcome::Panicked | ReplayOutcome::Failed(_)) => outcome,
         },
+        ControlKind::IdealFirstCallThenTodaysSecondCall => ReplayOutcome::Returned(ideal()),
+        ControlKind::IdealWithAFieldOfTheLandedVersion(field) => {
+            let mut returned = ideal();
+            if truth.farthest_version > state.chosen_version {
+                field.copy_from(
+                    &mut returned.root,
+                    &state.history.timeline_root(truth.farthest_version),
+                );
+            }
+            ReplayOutcome::Returned(returned)
+        }
+        ControlKind::IdealCopyingTheChosenUnmountMarkerWhenApplied => {
+            let mut returned = ideal();
+            if truth.farthest_version > state.chosen_version {
+                returned.root.unmount_marker = chosen_root.unmount_marker;
+            }
+            ReplayOutcome::Returned(returned)
+        }
+        ControlKind::IdealWritingNotWrittenWhenNothingIsApplied => {
+            let mut returned = ideal();
+            if truth.farthest_version == state.chosen_version {
+                returned.root.unmount_marker = UnmountMarker::NotWrittenByTheUnmountSequence;
+            }
+            ReplayOutcome::Returned(returned)
+        }
+        ControlKind::IdealCountingOnlyTheFirstDeviceCopies => {
+            let first_device_view = LegalState {
+                history: state.history,
+                presence: state.presence,
+                mirroring: JournalMirroring::FirstDeviceOnly,
+                residue: state.residue,
+                chosen_version: state.chosen_version,
+                verify_named_units: state.verify_named_units,
+            };
+            ReplayOutcome::Returned(returned_at_version(
+                state,
+                chosen_root,
+                farthest_version_of(&first_device_view, Some(truth.in_flight_limit)),
+            ))
+        }
         ControlKind::SecondCallMovesOn
         | ControlKind::SpecificationWithReading(_)
         | ControlKind::RootFromARecordWithoutTheLastRecordFlag
-        | ControlKind::ReportsOneRecordOverTheLimit => today(),
+        | ControlKind::ReportsOneRecordOverTheLimit
+        | ControlKind::SecondCallReportsMoreThanOneLastRecordFlag
+        | ControlKind::SecondCallReportsTheChosenRootTxgHasNoSuccessor
+        | ControlKind::ScanTableLosesItsLargestKeyRecord
+        | ControlKind::ScanPassOutcomeBecomesAPanic => today(),
     }
 }
 
@@ -2132,12 +3022,19 @@
     match implementation {
         Implementation::Control(ControlKind::SecondCallMovesOn) => {
             ReplayOutcome::Returned(ReturnedReplay {
-                root: next_timeline_version_root
-                    .unwrap_or_else(|| off_timeline_root(first_root.checkpoint_txg.0, 2)),
+                root: next_timeline_version_root.unwrap_or_else(|| {
+                    off_timeline_root(first_root.checkpoint_txg.0, 2, first_root)
+                }),
                 prefix_applied: 1,
                 verification_passed: 0,
             })
         }
+        Implementation::Control(ControlKind::SecondCallReportsMoreThanOneLastRecordFlag) => {
+            ReplayOutcome::Failed(FailureKind::RootPublishCarriesMoreThanOneLastRecordFlag)
+        }
+        Implementation::Control(ControlKind::SecondCallReportsTheChosenRootTxgHasNoSuccessor) => {
+            ReplayOutcome::Failed(FailureKind::ChosenRootTxgHasNoSuccessor)
+        }
         Implementation::TodayReplayJournal | Implementation::Control(_) => {
             call_replay_journal(disk, first_root, records, verify_named_units)
         }
@@ -2151,6 +3048,7 @@
 )]
 fn judge_the_fixed_point(
     implementation: Implementation,
+    run_profile: RunProfile,
     first: &ReturnedReplay,
     next_timeline_version_root: Option<RootRecord>,
     premise_holds: bool,
@@ -2174,6 +3072,19 @@
     } else {
         Counter::J6PremiseFailsJudged
     });
+    if run_profile.judges_j6qn_and_j7() {
+        if first.root.checkpoint_txg.0 == u64::MAX {
+            tally.add(Counter::FirstReturnedRootTxgIsTheLargest);
+        }
+        judge_the_fixed_point_under_qn(
+            &first.root,
+            &second,
+            fixed_point_holds,
+            premise_holds,
+            describe_witness,
+            tally,
+        );
+    }
     if fixed_point_holds {
         return;
     }
@@ -2207,6 +3118,112 @@
     });
 }
 
+/// J6Qn (ii)（登记 r2 第一节）：第二次报 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`，且判据自己看到 ρ_1 的 txg 是 u64 最大值
+/// （「因所选根 txg 顶格」由判据核，不只认报错的字样；不取 Qw）。
+fn second_call_failed_because_the_first_root_txg_is_the_largest(
+    second: &ReplayOutcome,
+    first_root: &RootRecord,
+) -> bool {
+    let first_root_txg_is_the_largest = first_root.checkpoint_txg.0 == u64::MAX;
+    let second_call_reports_no_successor = matches!(
+        second,
+        ReplayOutcome::Failed(FailureKind::ChosenRootTxgHasNoSuccessor)
+    );
+    second_call_reports_no_successor && first_root_txg_is_the_largest
+}
+
+/// J6Qn（登记 r2 第一节，判决 S6 取 Qn）：第二次 (i) 返回根且与 ρ_1 全字段相等，或 (ii) 因 ρ_1 的 txg 顶格报
+/// `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`，记相等；别的报错与 panic 一律违例。前提真 / 假两行照原 J6 分。
+fn judge_the_fixed_point_under_qn(
+    first_root: &RootRecord,
+    second: &ReplayOutcome,
+    fixed_point_holds: bool,
+    premise_holds: bool,
+    describe_witness: &dyn Fn() -> String,
+    tally: &mut Tally,
+) {
+    tally.add(if premise_holds {
+        Counter::J6QnPremiseHoldsJudged
+    } else {
+        Counter::J6QnPremiseFailsJudged
+    });
+    let counts_as_equal_under_qn = fixed_point_holds
+        || second_call_failed_because_the_first_root_txg_is_the_largest(second, first_root);
+    if counts_as_equal_under_qn {
+        return;
+    }
+    let describe_with_the_second_call = || {
+        format!(
+            "{};first_root_txg={};second={}",
+            describe_witness(),
+            first_root.checkpoint_txg.0,
+            describe_outcome(second)
+        )
+    };
+    tally.add_violation(
+        if premise_holds {
+            Counter::J6QnPremiseHoldsViolations
+        } else {
+            Counter::J6QnPremiseFailsViolations
+        },
+        &describe_with_the_second_call,
+    );
+}
+
+/// J7（登记 r2 第一节，判决 S1 后半）：凡第一次调用返回根 ρ 的态都判。J7a：实例表指针、分配记录树根、fsid 各等于所选根的；
+/// 投影不等于所选根的（由记录重建过）J7b：卸载记号是 `NotWritten…`；投影等于所选根的（一条没施加）J7c：卸载记号等于所选根的。
+/// 「重建过没有」由判据比投影定，不读报告的 `prefix_applied`。
+fn judge_the_fields_beyond_the_projection(
+    chosen_root: &RootRecord,
+    returned_root: &RootRecord,
+    describe_witness: &dyn Fn() -> String,
+    tally: &mut Tally,
+) {
+    let projection_equals_the_chosen_one =
+        VersionFields::of_root(returned_root) == VersionFields::of_root(chosen_root);
+    let describe_with_both_roots = || {
+        format!(
+            "{};chosen_root={};returned_root={};projection_equals_the_chosen_one={projection_equals_the_chosen_one}",
+            describe_witness(),
+            FieldsBeyondTheProjection::of_root(chosen_root).describe(),
+            FieldsBeyondTheProjection::of_root(returned_root).describe()
+        )
+    };
+    tally.add(Counter::J7Judged);
+    if returned_root.instance_table != chosen_root.instance_table {
+        tally.add_violation(
+            Counter::J7aInstanceTableViolations,
+            &describe_with_both_roots,
+        );
+    }
+    if returned_root.allocation_record_tree_root != chosen_root.allocation_record_tree_root {
+        tally.add_violation(
+            Counter::J7aAllocationRecordTreeRootViolations,
+            &describe_with_both_roots,
+        );
+    }
+    if returned_root.filesystem_identifier != chosen_root.filesystem_identifier {
+        tally.add_violation(
+            Counter::J7aFilesystemIdentifierViolations,
+            &describe_with_both_roots,
+        );
+    }
+    let expected_unmount_marker = if projection_equals_the_chosen_one {
+        chosen_root.unmount_marker
+    } else {
+        UnmountMarker::NotWrittenByTheUnmountSequence
+    };
+    let (judged_counter, violations_counter) = if projection_equals_the_chosen_one {
+        (Counter::J7cJudged, Counter::J7cViolations)
+    } else {
+        (Counter::J7bJudged, Counter::J7bViolations)
+    };
+    tally.add(judged_counter);
+    if returned_root.unmount_marker != expected_unmount_marker {
+        tally.add_violation(violations_counter, &describe_with_both_roots);
+    }
+}
+
 /// 这个域的状态进不进「S 自己的对照」（登记 5.4：L、L 小环、U 开着验证、X、E1）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum SpecificationSelfCheck {
@@ -2214,7 +3231,50 @@
     NotJudged,
 }
 
-/// 判合法历史一族的一个状态：J0–J4、J6、IA1、IA2、域非空见证、在飞上限的分布、S 自己的对照。
+/// 一个合法历史状态的写法（逐态行的 `state=` 与每条违例的见证）：形状、每条记录在盘上与否、1 号盘那几份、残留、所选版本、j\*、第一次调用的结局。
+fn describe_a_legal_state_with_its_first_outcome(
+    state: &LegalState<'_>,
+    farthest_version: usize,
+    first: &ReplayOutcome,
+) -> String {
+    let presence: Vec<String> = state
+        .presence
+        .iter()
+        .map(|record_presence| match record_presence {
+            RecordPresence::NotOnDisk => "-".to_string(),
+            RecordPresence::OnDisk(unit_state) => unit_state.label().to_string(),
+        })
+        .collect();
+    let second_device_presence = match state.mirroring {
+        JournalMirroring::FirstDeviceOnly => String::new(),
+        JournalMirroring::BothDevices {
+            second_device_copy_is_on_disk,
+        } => format!(
+            ";on_second_device={}",
+            second_device_copy_is_on_disk
+                .iter()
+                .map(|is_on_disk| if *is_on_disk { "good" } else { "-" })
+                .collect::<Vec<&str>>()
+                .join(",")
+        ),
+    };
+    format!(
+        "shape={};on_disk={}{second_device_presence};residue_at={};chosen={};farthest={};verify={};first={}",
+        state.history.shape_description,
+        presence.join(","),
+        state.residue.map_or_else(
+            || "none".to_string(),
+            |residue| residue.record_index.to_string()
+        ),
+        state.chosen_version,
+        farthest_version,
+        state.verify_named_units,
+        describe_outcome(first)
+    )
+}
+
+/// 判合法历史一族的一个状态：J0–J4、J6、IA1、IA2、域非空见证、在飞上限的分布、S 自己的对照。记录表怎么建由档的 `table_source` 定
+/// （登记 r2 5.4）：`Both` 时两张表各判一遍、逐态比，汇总的计数取直接建表那一遍。
 /// 返回这一次重放施加的发布区间（C 域的状态对要它）；panic 或落点不在时间线上时是 `None`。
 fn judge_a_legal_state(
     state: &LegalState<'_>,
@@ -2227,43 +3287,178 @@
     if matches!(probe_decision, ProbeDecision::SkipThisState) {
         return None;
     }
-    let (records, disk, truths) = record_table_and_disk_of(state);
-    let truth = legal_truth_of(state, &records);
+    let run_profile = state.history.run_profile;
+    let (direct_table, disk, truths) = record_table_and_disk_of(state);
+    let judge_on = |table: &RecordTable,
+                    implementation_of_the_pass: Implementation,
+                    designated_in_the_pass: bool,
+                    tally_of_the_pass: &mut Tally| {
+        judge_a_legal_state_on_a_table(
+            state,
+            implementation_of_the_pass,
+            specification_self_check,
+            designated_in_the_pass,
+            table,
+            &disk,
+            &truths,
+            tally_of_the_pass,
+        )
+    };
+    let judgement = match run_profile.table_source {
+        RecordTableSource::Direct => judge_on(
+            &direct_table,
+            implementation,
+            is_the_designated_state,
+            tally,
+        ),
+        RecordTableSource::Scanned => {
+            let scanned_table = scanned_record_table_of(&disk, implementation);
+            assert_the_scan_table_of_a_fully_landed_state(
+                state,
+                &scanned_table,
+                implementation,
+                tally,
+            );
+            judge_on(
+                &scanned_table,
+                implementation,
+                is_the_designated_state,
+                tally,
+            )
+        }
+        RecordTableSource::Both => {
+            let scanned_table = scanned_record_table_of(&disk, implementation);
+            let counts_before_the_state = tally.counts.clone();
+            let (scan_pass_implementation, scan_pass_is_designated) =
+                implementation_and_designation_of_the_scan_pass(
+                    implementation,
+                    is_the_designated_state,
+                );
+            let mut scanned_tally = Tally::new();
+            let scanned_judgement = judge_on(
+                &scanned_table,
+                scan_pass_implementation,
+                scan_pass_is_designated,
+                &mut scanned_tally,
+            );
+            let scanned_summary = JudgedRowSummary {
+                first_outcome: scanned_judgement.first_outcome,
+                counter_labels: counter_labels_a_state_added_to(
+                    &scanned_tally,
+                    &Tally::new().counts,
+                ),
+            };
+            let direct_judgement = judge_on(
+                &direct_table,
+                implementation,
+                is_the_designated_state,
+                tally,
+            );
+            let direct_summary = JudgedRowSummary {
+                first_outcome: direct_judgement.first_outcome,
+                counter_labels: counter_labels_a_state_added_to(tally, &counts_before_the_state),
+            };
+            let describe_state = || {
+                describe_a_legal_state_with_its_first_outcome(
+                    state,
+                    direct_judgement.farthest_version,
+                    &direct_judgement.first_outcome,
+                )
+            };
+            add_the_comparison_of_two_judged_tables(
+                tally,
+                (Counter::TablesDiffer, Counter::RowsDiffer),
+                (&direct_table, &direct_summary),
+                (&scanned_table, &scanned_summary),
+                &describe_state,
+            );
+            if !direct_table.is_empty() {
+                tally.add(Counter::DirectTableHoldsARecord);
+            }
+            assert_the_scan_table_of_a_fully_landed_state(
+                state,
+                &scanned_table,
+                implementation,
+                tally,
+            );
+            if run_profile.reported_disk_order == ReportedDiskOrder::SecondDeviceBeforeFirst {
+                let (reported_order_table, _reported_order_disk, reported_order_truths) =
+                    record_table_and_disk_of_in_device_order(
+                        state,
+                        run_profile.reported_disk_order.device_numbers(),
+                    );
+                let mut reported_order_tally = Tally::new();
+                let reported_order_judgement = judge_a_legal_state_on_a_table(
+                    state,
+                    implementation,
+                    specification_self_check,
+                    is_the_designated_state,
+                    &reported_order_table,
+                    &disk,
+                    &reported_order_truths,
+                    &mut reported_order_tally,
+                );
+                let reported_order_summary = JudgedRowSummary {
+                    first_outcome: reported_order_judgement.first_outcome,
+                    counter_labels: counter_labels_a_state_added_to(
+                        &reported_order_tally,
+                        &Tally::new().counts,
+                    ),
+                };
+                add_the_comparison_of_two_judged_tables(
+                    tally,
+                    (
+                        Counter::TablesDifferInReportedOrder,
+                        Counter::RowsDifferInReportedOrder,
+                    ),
+                    (&reported_order_table, &reported_order_summary),
+                    (&scanned_table, &scanned_summary),
+                    &describe_state,
+                );
+            }
+            direct_judgement
+        }
+    };
+    finish_probing_a_state(probe_decision, tally, &|| {
+        describe_a_legal_state_with_its_first_outcome(
+            state,
+            judgement.farthest_version,
+            &judgement.first_outcome,
+        )
+    });
+    judgement.interval
+}
+
+/// [`judge_a_legal_state`] 在一张给定的记录表上判一个状态：不管探针、不管两种建表法怎么比，只数这一遍判出来的量。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "每个参数都是这一判要用的一样东西，凑不成一个带不变量的概念"
+)]
+fn judge_a_legal_state_on_a_table(
+    state: &LegalState<'_>,
+    implementation: Implementation,
+    specification_self_check: SpecificationSelfCheck,
+    is_the_designated_state: bool,
+    records: &RecordTable,
+    disk: &MemoryDisk<'_>,
+    truths: &[RecordTruthOnDisk],
+    tally: &mut Tally,
+) -> LegalJudgement {
+    let truth = legal_truth_of(state, records);
     let history = state.history;
     let chosen_version = state.chosen_version;
-    let chosen_root = history.version_fields(chosen_version).to_root();
+    let chosen_root = history.timeline_root(chosen_version);
     let first = first_outcome_on_a_legal_state(
         implementation,
         state,
         &truth,
         &chosen_root,
-        &records,
-        &disk,
+        records,
+        disk,
         is_the_designated_state,
     );
-    let describe_witness = || {
-        let presence: Vec<String> = state
-            .presence
-            .iter()
-            .map(|record_presence| match record_presence {
-                RecordPresence::NotOnDisk => "-".to_string(),
-                RecordPresence::OnDisk(unit_state) => unit_state.label().to_string(),
-            })
-            .collect();
-        format!(
-            "shape={};on_disk={};residue_at={};chosen={};farthest={};verify={};first={}",
-            history.shape_description,
-            presence.join(","),
-            state.residue.map_or_else(
-                || "none".to_string(),
-                |residue| residue.record_index.to_string()
-            ),
-            chosen_version,
-            truth.farthest_version,
-            state.verify_named_units,
-            describe_outcome(&first)
-        )
-    };
+    let describe_witness =
+        || describe_a_legal_state_with_its_first_outcome(state, truth.farthest_version, &first);
 
     tally.add(Counter::States);
     if chosen_version > 0 {
@@ -2288,10 +3483,10 @@
     if specification_self_check == SpecificationSelfCheck::Judged {
         let view = SpecificationView {
             chosen_root: &chosen_root,
-            records: &records,
+            records,
             in_flight_limit: truth.in_flight_limit,
             verify_named_units: state.verify_named_units,
-            truths: &truths,
+            truths,
         };
         let expected_by_the_farthest_version =
             SpecificationExpectation::ReturnsRoot(history.version_fields(truth.farthest_version));
@@ -2344,18 +3539,31 @@
             let next_timeline_version_root = landing
                 .map(|version| version + 1)
                 .filter(|next_version| *next_version < history.chosen_instance_publish_count)
-                .map(|next_version| history.version_fields(next_version).to_root());
+                .map(|next_version| {
+                    history
+                        .version_fields(next_version)
+                        .to_root_rebuilt_over(&returned.root)
+                });
             judge_the_fixed_point(
                 implementation,
+                history.run_profile,
                 returned,
                 next_timeline_version_root,
                 truth.above_water_record_count <= truth.in_flight_limit,
-                &records,
-                &disk,
+                records,
+                disk,
                 state.verify_named_units,
                 &describe_witness,
                 tally,
             );
+            if history.run_profile.judges_j6qn_and_j7() {
+                judge_the_fields_beyond_the_projection(
+                    &chosen_root,
+                    &returned.root,
+                    &describe_witness,
+                    tally,
+                );
+            }
             landing
         }
     };
@@ -2396,8 +3604,7 @@
         tally.add(Counter::AppliedAtLeastTwo);
     }
 
-    finish_probing_a_state(probe_decision, tally, &describe_witness);
-    landing
+    let interval = landing
         .filter(|version| *version >= chosen_version)
         .map(|version| AppliedInterval {
             low: if implementation
@@ -2408,7 +3615,12 @@
                 chosen_version
             },
             high: version,
-        })
+        });
+    LegalJudgement {
+        interval,
+        first_outcome: first,
+        farthest_version: truth.farthest_version,
+    }
 }
 
 // ── 域（登记 5.2）
@@ -2430,6 +3642,9 @@
         TxgOneBelowTheLargest => "E1",
         TxgAtTheLargest => "E2",
         Residue => "R",
+        MirroredBigRing => "M",
+        MirroredSmallRingSix => "M6",
+        MirroredMonotoneBigRing => "MC",
     }
 }
 
@@ -2437,6 +3652,8 @@
 enum Family {
     LegalHistory,
     Adversarial,
+    /// 登记 r2 5.3 的多盘域 M、M6、MC：只在 `--profile r2 --family multi_disk` 时跑。
+    MultiDisk,
 }
 
 impl Domain {
@@ -2451,10 +3668,13 @@
             | Self::CrossInstance
             | Self::TxgOneBelowTheLargest
             | Self::TxgAtTheLargest
-            | Self::Residue => RingGeometry::BIG,
+            | Self::Residue
+            | Self::MirroredBigRing
+            | Self::MirroredMonotoneBigRing => RingGeometry::BIG,
             Self::LegalSmallRingSix
             | Self::MonotoneSmallRingSix
-            | Self::AdversarialSmallRingSix => RingGeometry::SMALL_SIX,
+            | Self::AdversarialSmallRingSix
+            | Self::MirroredSmallRingSix => RingGeometry::SMALL_SIX,
             Self::LegalSmallRingNine => RingGeometry::SMALL_NINE,
         }
     }
@@ -2475,6 +3695,9 @@
             Self::AdversarialBigRing
             | Self::AdversarialSmallRingSix
             | Self::AdversarialBackChain => Family::Adversarial,
+            Self::MirroredBigRing | Self::MirroredSmallRingSix | Self::MirroredMonotoneBigRing => {
+                Family::MultiDisk
+            }
         }
     }
 
@@ -2493,7 +3716,10 @@
             | Self::CrossInstance
             | Self::TxgOneBelowTheLargest
             | Self::TxgAtTheLargest
-            | Self::Residue => self.label(),
+            | Self::Residue
+            | Self::MirroredBigRing
+            | Self::MirroredSmallRingSix
+            | Self::MirroredMonotoneBigRing => self.label(),
         }
     }
 
@@ -2512,7 +3738,10 @@
             | Self::AdversarialSmallRingSix
             | Self::AdversarialBackChain
             | Self::TxgOneBelowTheLargest
-            | Self::TxgAtTheLargest => 1,
+            | Self::TxgAtTheLargest
+            | Self::MirroredBigRing
+            | Self::MirroredSmallRingSix
+            | Self::MirroredMonotoneBigRing => 1,
         }
     }
 
@@ -2532,7 +3761,10 @@
             | Self::AdversarialSmallRingSix
             | Self::AdversarialBackChain
             | Self::TxgAtTheLargest
-            | Self::Residue => SpecificationSelfCheck::NotJudged,
+            | Self::Residue
+            | Self::MirroredBigRing
+            | Self::MirroredSmallRingSix
+            | Self::MirroredMonotoneBigRing => SpecificationSelfCheck::NotJudged,
         }
     }
 }
@@ -2557,6 +3789,10 @@
         Domain::TxgOneBelowTheLargest => &[2, 12, 64, 320],
         Domain::TxgAtTheLargest => &[2, 8, 32],
         Domain::Residue => &[4, 64, 688, 6_208],
+        // 登记 r2 第七节乙类：M 是 4 × 12^(N − 1)；M6 每次发布至多 2 条；MC 是 Nc = 1..4。
+        Domain::MirroredBigRing => &[4, 48, 576, 6_912, 82_944, 995_328],
+        Domain::MirroredSmallRingSix => &[4, 48, 320, 2_816, 21_504, 176_128],
+        Domain::MirroredMonotoneBigRing => &[8, 128, 2_048, 32_768],
     }
 }
 
@@ -2631,7 +3867,119 @@
         | Domain::CrossInstance
         | Domain::TxgOneBelowTheLargest
         | Domain::TxgAtTheLargest
-        | Domain::Residue => &[],
+        | Domain::Residue
+        | Domain::MirroredBigRing
+        | Domain::MirroredSmallRingSix
+        | Domain::MirroredMonotoneBigRing => &[],
+    }
+}
+
+/// M、M6 每个 N 的锚点（登记 r2 第七节乙类，`anchors_r2.py`、`anchors_r2b.py`、`anchors_r2c.py`）：三类（相等, 落后, 互不包含）、
+/// 落后的一个方向（0 号盘落后与 1 号盘落后两行相等）、见证（j\* ≥ 1, j\* ≥ 2；记录任一份在即在）。
+struct PinnedMirroredCounts {
+    equal_states: u64,
+    lagging_states: u64,
+    crossed_states: u64,
+    lagging_states_in_one_direction: u64,
+    farthest_at_least_one: u64,
+    farthest_at_least_two: u64,
+}
+
+const fn pinned_mirrored_counts(domain: Domain) -> &'static [PinnedMirroredCounts] {
+    const fn row(
+        classes: (u64, u64, u64),
+        lagging_states_in_one_direction: u64,
+        witnesses: (u64, u64),
+    ) -> PinnedMirroredCounts {
+        PinnedMirroredCounts {
+            equal_states: classes.0,
+            lagging_states: classes.1,
+            crossed_states: classes.2,
+            lagging_states_in_one_direction,
+            farthest_at_least_one: witnesses.0,
+            farthest_at_least_two: witnesses.1,
+        }
+    }
+    const MIRRORED_BIG_RING: [PinnedMirroredCounts; 6] = [
+        row((2, 2, 0), 1, (0, 0)),
+        row((12, 30, 6), 15, (12, 0)),
+        row((72, 342, 162), 171, (216, 36)),
+        row((432, 3_510, 2_970), 1_755, (3_024, 864)),
+        row((2_592, 34_182, 46_170), 17_091, (38_880, 14_256)),
+        row((15_552, 323_190, 656_586), 161_595, (482_112, 202_176)),
+    ];
+    const MIRRORED_SMALL_RING_SIX: [PinnedMirroredCounts; 6] = [
+        row((2, 2, 0), 1, (0, 0)),
+        row((12, 30, 6), 15, (12, 0)),
+        row((40, 190, 90), 95, (216, 36)),
+        row((176, 1_430, 1_210), 715, (1_824, 432)),
+        row((672, 8_862, 11_970), 4_431, (14_208, 2_880)),
+        row((2_752, 57_190, 116_186), 28_595, (115_200, 25_344)),
+    ];
+    match domain {
+        Domain::MirroredBigRing => &MIRRORED_BIG_RING,
+        Domain::MirroredSmallRingSix => &MIRRORED_SMALL_RING_SIX,
+        Domain::LegalBigRing
+        | Domain::LegalSmallRingSix
+        | Domain::LegalSmallRingNine
+        | Domain::NamedUnitsVerified
+        | Domain::NamedUnitsNotVerified
+        | Domain::MonotoneBigRing
+        | Domain::MonotoneSmallRingSix
+        | Domain::AdversarialBigRing
+        | Domain::AdversarialSmallRingSix
+        | Domain::AdversarialBackChain
+        | Domain::CrossInstance
+        | Domain::TxgOneBelowTheLargest
+        | Domain::TxgAtTheLargest
+        | Domain::Residue
+        | Domain::MirroredMonotoneBigRing => &[],
+    }
+}
+
+/// MC 每个 Nc 的锚点（登记 r2 第七节乙类）：宽对数；见证（所选根不是 R_0 的态数, j\* − c ≥ 1, j\* − c ≥ 2）。
+struct PinnedMirroredMonotoneCounts {
+    wide_pairs: u64,
+    chosen_root_is_a_later_version: u64,
+    farthest_at_least_one: u64,
+    farthest_at_least_two: u64,
+}
+
+const fn pinned_mirrored_monotone_counts(
+    domain: Domain,
+) -> &'static [PinnedMirroredMonotoneCounts] {
+    const fn row(wide_pairs: u64, witnesses: (u64, u64, u64)) -> PinnedMirroredMonotoneCounts {
+        PinnedMirroredMonotoneCounts {
+            wide_pairs,
+            chosen_root_is_a_later_version: witnesses.0,
+            farthest_at_least_one: witnesses.1,
+            farthest_at_least_two: witnesses.2,
+        }
+    }
+    const MIRRORED_MONOTONE_BIG_RING: [PinnedMirroredMonotoneCounts; 4] = [
+        row(27, (4, 3, 0)),
+        row(1_215, (80, 42, 9)),
+        row(54_675, (1_472, 636, 180)),
+        row(2_460_375, (25_856, 9_960, 2_988)),
+    ];
+    match domain {
+        Domain::MirroredMonotoneBigRing => &MIRRORED_MONOTONE_BIG_RING,
+        Domain::LegalBigRing
+        | Domain::LegalSmallRingSix
+        | Domain::LegalSmallRingNine
+        | Domain::NamedUnitsVerified
+        | Domain::NamedUnitsNotVerified
+        | Domain::MonotoneBigRing
+        | Domain::MonotoneSmallRingSix
+        | Domain::AdversarialBigRing
+        | Domain::AdversarialSmallRingSix
+        | Domain::AdversarialBackChain
+        | Domain::CrossInstance
+        | Domain::TxgOneBelowTheLargest
+        | Domain::TxgAtTheLargest
+        | Domain::Residue
+        | Domain::MirroredBigRing
+        | Domain::MirroredSmallRingSix => &[],
     }
 }
 
@@ -2658,8 +4006,10 @@
     first_txg: u64,
     chosen_instance_publishes: Vec<PublishShape>,
     next_instance_publishes: Vec<PublishShape>,
+    run_profile: RunProfile,
 ) -> HistoryPlan {
     HistoryPlan {
+        run_profile,
         ring: domain.ring(),
         first_txg,
         chosen_instance_publishes,
@@ -2677,32 +4027,39 @@
             | Domain::CrossInstance
             | Domain::TxgOneBelowTheLargest
             | Domain::TxgAtTheLargest
-            | Domain::Residue => false,
+            | Domain::Residue
+            | Domain::MirroredBigRing
+            | Domain::MirroredSmallRingSix
+            | Domain::MirroredMonotoneBigRing => false,
         },
     }
 }
 
 /// 一个合法历史域在记录总数 `size` 上的全部工作单元（登记 5.2 每个域的定义）。
-fn legal_work_units(domain: Domain, size: usize) -> Vec<LegalWorkUnit> {
+fn legal_work_units(domain: Domain, size: usize, run_profile: RunProfile) -> Vec<LegalWorkUnit> {
     let plain = |first_txg: u64, sequences: Vec<Vec<PublishShape>>| -> Vec<LegalWorkUnit> {
         sequences
             .into_iter()
             .map(|sequence| LegalWorkUnit {
-                plan: history_plan(domain, first_txg, sequence, Vec::new()),
+                plan: history_plan(domain, first_txg, sequence, Vec::new(), run_profile),
                 residue_record_index: None,
             })
             .collect()
     };
     let in_flight_limit = domain.ring().local_in_flight_limit();
     match domain {
-        Domain::LegalBigRing | Domain::NamedUnitsVerified | Domain::NamedUnitsNotVerified => {
-            plain(ORDINARY_FIRST_TXG, publish_sequences_of(size, None))
+        // M：同 L，每条记录在两块盘上各一份（登记 r2 5.3）；落盘子集在 [`tally_of_a_mirrored_history`] 里两盘各取。
+        Domain::LegalBigRing
+        | Domain::NamedUnitsVerified
+        | Domain::NamedUnitsNotVerified
+        | Domain::MirroredBigRing => plain(ORDINARY_FIRST_TXG, publish_sequences_of(size, None)),
+        // 小环：每次发布不超过上限条（合法写侧一次发布最多切上限条）；M6 同 L6。
+        Domain::LegalSmallRingSix | Domain::LegalSmallRingNine | Domain::MirroredSmallRingSix => {
+            plain(
+                ORDINARY_FIRST_TXG,
+                publish_sequences_of(size, Some(in_flight_limit)),
+            )
         }
-        // 小环：每次发布不超过上限条（合法写侧一次发布最多切上限条）。
-        Domain::LegalSmallRingSix | Domain::LegalSmallRingNine => plain(
-            ORDINARY_FIRST_TXG,
-            publish_sequences_of(size, Some(in_flight_limit)),
-        ),
         // X：实例 g 的一段合法历史之后接实例 g + 1 的一段合法历史，两段各至少 1 条。
         Domain::CrossInstance => {
             let mut units = Vec::new();
@@ -2716,6 +4073,7 @@
                                 ORDINARY_FIRST_TXG,
                                 chosen_sequence.clone(),
                                 next_sequence,
+                                run_profile,
                             ),
                             residue_record_index: None,
                         });
@@ -2754,6 +4112,7 @@
                             ORDINARY_FIRST_TXG,
                             sequence.clone(),
                             Vec::new(),
+                            run_profile,
                         ),
                         residue_record_index: Some(residue_record_index),
                     });
@@ -2761,8 +4120,10 @@
             }
             units
         }
-        // C：P_0 一条记录，之后 Nc 条记录；小环每次发布至多上限条。
-        Domain::MonotoneBigRing | Domain::MonotoneSmallRingSix => {
+        // C：P_0 一条记录，之后 Nc 条记录；小环每次发布至多上限条。MC 同 C 大环（登记 r2 5.3）。
+        Domain::MonotoneBigRing
+        | Domain::MonotoneSmallRingSix
+        | Domain::MirroredMonotoneBigRing => {
             let largest_publish =
                 (domain == Domain::MonotoneSmallRingSix).then_some(in_flight_limit);
             publish_sequences_of(size, largest_publish)
@@ -2774,7 +4135,13 @@
                     }];
                     publishes.extend(sequence);
                     LegalWorkUnit {
-                        plan: history_plan(domain, ORDINARY_FIRST_TXG, publishes, Vec::new()),
+                        plan: history_plan(
+                            domain,
+                            ORDINARY_FIRST_TXG,
+                            publishes,
+                            Vec::new(),
+                            run_profile,
+                        ),
                         residue_record_index: None,
                     }
                 })
@@ -2809,6 +4176,7 @@
         let state = LegalState {
             history: &history,
             presence: presence_of_the_state,
+            mirroring: JournalMirroring::FirstDeviceOnly,
             residue: residue.as_ref(),
             chosen_version: 0,
             verify_named_units,
@@ -2909,6 +4277,7 @@
         let state = LegalState {
             history: &history,
             presence: &presence,
+            mirroring: JournalMirroring::FirstDeviceOnly,
             residue: None,
             chosen_version,
             verify_named_units: true,
@@ -3048,6 +4417,176 @@
     tally
 }
 
+// ── 多盘域（登记 r2 5.3）：每条记录在 0、1 号盘同一落点各一份，各自落或没落；没有点名单元
+
+/// M、M6 一段历史：每条记录四态（0 号盘那一份落没落 × 1 号盘那一份落没落），所选根 R_0；每个状态另按两盘落盘集合的关系记进落后类别。
+fn tally_of_a_mirrored_history(
+    unit: &LegalWorkUnit,
+    unit_index: usize,
+    implementation: Implementation,
+) -> Tally {
+    let history = generate_history(&unit.plan);
+    let record_count = history.slots.len();
+    let mut tally = Tally::new();
+    let mut presence = vec![RecordPresence::NotOnDisk; record_count];
+    let mut second_device_copy_is_on_disk = vec![false; record_count];
+    // 状态掩码：低 N 位是各条记录 0 号盘那一份，高 N 位是 1 号盘那一份。
+    for state_mask in every_subset_mask(2 * record_count) {
+        let first_device_mask = state_mask & ((1u32 << record_count) - 1);
+        let second_device_mask = state_mask >> record_count;
+        for (record_index, (record_presence, second_device_copy)) in presence
+            .iter_mut()
+            .zip(second_device_copy_is_on_disk.iter_mut())
+            .enumerate()
+        {
+            *record_presence = if bit_is_set(first_device_mask, record_index) {
+                RecordPresence::OnDisk(UnitState::BothCopiesGood)
+            } else {
+                RecordPresence::NotOnDisk
+            };
+            *second_device_copy = bit_is_set(second_device_mask, record_index);
+        }
+        let state = LegalState {
+            history: &history,
+            presence: &presence,
+            mirroring: JournalMirroring::BothDevices {
+                second_device_copy_is_on_disk: &second_device_copy_is_on_disk,
+            },
+            residue: None,
+            chosen_version: 0,
+            verify_named_units: true,
+        };
+        let counts_before = tally.counts.clone();
+        judge_a_legal_state(
+            &state,
+            implementation,
+            SpecificationSelfCheck::NotJudged,
+            unit_index == 0 && state_mask == 0 && record_count == 1,
+            &mut tally,
+        );
+        // 探针跳过的状态不判、不进落后类别。
+        if tally.get(Counter::States) > counts_before[Counter::States.index()] {
+            tally.add_to_the_lagging_class(
+                lagging_class_of(first_device_mask, second_device_mask),
+                &counts_before,
+            );
+        }
+    }
+    tally
+}
+
+/// MC 一段历史（登记 r2 5.3）：P_0 一条两盘都在；其后 Nc 条记录各四态；P_1… 每次发布一个根槽写落或没落；所选根 = 落了的根里
+/// 最新那条（R_0 恒在）。先逐状态判，再在「宽」集合上数 J5a、J5b（甲合法、乙合法与同段那几个集合这一次不建）。
+fn tally_of_a_mirrored_monotone_history(
+    unit: &LegalWorkUnit,
+    unit_index: usize,
+    implementation: Implementation,
+) -> Tally {
+    let history = generate_history(&unit.plan);
+    let later_record_count = history.slots.len() - 1;
+    let root_write_count = history.chosen_instance_publish_count - 1;
+    let state_bit_count = 2 * later_record_count + root_write_count;
+    let mut tally = Tally::new();
+    let mut today_intervals: Vec<Option<AppliedInterval>> = Vec::new();
+    let mut ideal_intervals: Vec<AppliedInterval> = Vec::new();
+    let mut presence = vec![RecordPresence::OnDisk(UnitState::BothCopiesGood); history.slots.len()];
+    let mut second_device_copy_is_on_disk = vec![true; history.slots.len()];
+    // 状态掩码：低 Nc 位是 P_0 之后各条记录 0 号盘那一份，其上 Nc 位是 1 号盘那一份，最高 m 位是各次发布的根槽写落没落。
+    for state_mask in every_subset_mask(state_bit_count) {
+        for (later_record, (record_presence, second_device_copy)) in presence
+            .iter_mut()
+            .zip(second_device_copy_is_on_disk.iter_mut())
+            .skip(1)
+            .enumerate()
+        {
+            *record_presence = if bit_is_set(state_mask, later_record) {
+                RecordPresence::OnDisk(UnitState::BothCopiesGood)
+            } else {
+                RecordPresence::NotOnDisk
+            };
+            *second_device_copy = bit_is_set(state_mask, later_record_count + later_record);
+        }
+        let chosen_version = (1..=root_write_count)
+            .rev()
+            .find(|version| bit_is_set(state_mask, 2 * later_record_count + version - 1))
+            .unwrap_or(0);
+        let state = LegalState {
+            history: &history,
+            presence: &presence,
+            mirroring: JournalMirroring::BothDevices {
+                second_device_copy_is_on_disk: &second_device_copy_is_on_disk,
+            },
+            residue: None,
+            chosen_version,
+            verify_named_units: true,
+        };
+        let today_interval = judge_a_legal_state(
+            &state,
+            implementation,
+            SpecificationSelfCheck::NotJudged,
+            unit_index == 0 && state_mask == 0 && later_record_count == 1,
+            &mut tally,
+        );
+        if today_interval.is_none() {
+            tally.add(Counter::PairStatesWithoutAnInterval);
+        }
+        today_intervals.push(today_interval);
+        ideal_intervals.push(AppliedInterval {
+            low: chosen_version,
+            high: farthest_version_of(&state, Some(history.ring.local_in_flight_limit())),
+        });
+    }
+    add_the_wide_pairs(
+        &mut tally,
+        state_bit_count,
+        &today_intervals,
+        &ideal_intervals,
+    );
+    tally
+}
+
+/// 「宽」集合（登记 r2 5.3：两个状态的落盘集合与根槽集合都是包含关系，含相等）上数今天的实现与理想恢复各自的 J5a、J5b：
+/// 对每个 u 枚举它的子掩码 s，迭代次数合计 3^(状态位数)。
+fn add_the_wide_pairs(
+    tally: &mut Tally,
+    state_bit_count: usize,
+    today_intervals: &[Option<AppliedInterval>],
+    ideal_intervals: &[AppliedInterval],
+) {
+    for outer_mask in every_subset_mask(state_bit_count) {
+        let outer_index = usize::try_from(outer_mask).expect("状态掩码不超过 12 位");
+        let mut inner_mask = outer_mask;
+        loop {
+            let inner_index = usize::try_from(inner_mask).expect("状态掩码不超过 12 位");
+            if inner_mask & outer_mask == inner_mask {
+                tally.add_pair(PairSet::Wide, PairColumn::Pairs);
+                if let Some((today_inner, today_outer)) =
+                    today_intervals[inner_index].zip(today_intervals[outer_index])
+                {
+                    if !applied_set_is_contained(today_inner, today_outer) {
+                        tally.add_pair(PairSet::Wide, PairColumn::TodayAppliedSetViolations);
+                    }
+                    if landed_version_is_later(today_inner, today_outer) {
+                        tally.add_pair(PairSet::Wide, PairColumn::TodayLandedVersionViolations);
+                    }
+                }
+                let (ideal_inner, ideal_outer) =
+                    (ideal_intervals[inner_index], ideal_intervals[outer_index]);
+                if !applied_set_is_contained(ideal_inner, ideal_outer) {
+                    tally.add_pair(PairSet::Wide, PairColumn::IdealAppliedSetViolations);
+                }
+                if landed_version_is_later(ideal_inner, ideal_outer) {
+                    tally.add_pair(PairSet::Wide, PairColumn::IdealLandedVersionViolations);
+                }
+            }
+            if inner_mask == 0 {
+                break;
+            }
+            inner_mask = (inner_mask - 1) & outer_mask;
+        }
+    }
+}
+
 // ── 对抗一族（登记 5.2 的 A、A 小环、A 链）
 
 /// 对抗域里一个槽上那条记录的自由字段（登记 5.2：txg ∈ {t, t + 1, t + 2} × 序号 ∈ {0, 1, 2} × 末条标志 × 提交标记 × 事务号 ∈ {1, 2}）。
@@ -3111,13 +4650,14 @@
     choices
 }
 
-fn adversarial_chosen_root() -> RootRecord {
+/// 对抗一族的所选根：时间线 R_0 那一份（登记 r2 第一节：投影之外的四个字段照档取第 0 版的）。
+fn adversarial_chosen_root(run_profile: RunProfile) -> RootRecord {
     VersionFields {
         instance: CHOSEN_INSTANCE,
         checkpoint_txg: ORDINARY_FIRST_TXG,
         segment: NewRootSegment::of_timeline_publish(0, ORDINARY_FIRST_TXG),
     }
-    .to_root()
+    .to_root_with(run_profile.fields_beyond_the_projection_of_version(0))
 }
 
 /// 对抗域枚举到一半的状态：锚点槽与其后各槽上放着什么、由它们建出来的表。
@@ -3128,6 +4668,7 @@
     choices: &'run [AdversarialSlotChoice],
     chosen_root: RootRecord,
     implementation: Implementation,
+    run_profile: RunProfile,
     /// 下标 0 是锚点槽，之后是第 1..n 个槽；`None` 是不在。
     slots: Vec<Option<GeneratedSlot>>,
     choice_trail: Vec<AdversarialSlotChoice>,
@@ -3285,15 +4826,39 @@
         self.slots.pop();
     }
 
-    /// 判对抗域的一个状态：A0、AS 三条、A-互 三个数、A-结构、A-限、J6。
+    /// 一个对抗状态的写法（逐态行的 `state=` 与每条违例的见证）：锚点与各槽、第一次调用的结局、S 三种读法的期望。
+    fn describe_the_state_with_its_first_outcome(
+        &self,
+        first: &ReplayOutcome,
+        expectations: &[SpecificationExpectation; 3],
+    ) -> String {
+        format!(
+            "{};first={};expected={}",
+            self.describe_the_state(),
+            describe_outcome(first),
+            expectations
+                .iter()
+                .map(|expectation| match expectation {
+                    SpecificationExpectation::FailsWithoutApplying => "fails".to_string(),
+                    SpecificationExpectation::ReturnsRoot(fields) => format!(
+                        "txg{}:watermark{}",
+                        fields.checkpoint_txg, fields.segment.tree_identifier_watermark
+                    ),
+                })
+                .collect::<Vec<String>>()
+                .join("/")
+        )
+    }
+
+    /// 判对抗域的一个状态：A0、AS 三条、A-互 三个数、A-结构、A-限、J6。记录表怎么建由档的 `table_source` 定（登记 r2 5.4）：
+    /// `Both` 时两张表各判一遍、逐态比，汇总的计数取直接建表那一遍。对抗域的槽都在 0 号盘，盘次序不改直接建的表，不另判「照盘报的次序」那一栏。
     fn judge_the_state(&self, tally: &mut Tally) {
         let probe_decision = begin_probing_a_state(tally);
         if matches!(probe_decision, ProbeDecision::SkipThisState) {
             return;
         }
         let ring = self.domain.ring();
-        let in_flight_limit = ring.local_in_flight_limit();
-        let mut records = RecordTable::new();
+        let mut direct_table = RecordTable::new();
         let mut truths: Vec<RecordTruthOnDisk> = Vec::new();
         let mut contents: Vec<(u32, u64, StoredBytes<'_>)> = Vec::new();
         let mut holds_a_record_rejected_by_parse = false;
@@ -3305,7 +4870,7 @@
             ));
             match slot.parsed.as_ref() {
                 Some(parsed) => {
-                    records.insert((parsed.instance, parsed.counter), parsed.clone());
+                    direct_table.insert((parsed.instance, parsed.counter), parsed.clone());
                     truths.push(RecordTruthOnDisk {
                         instance: parsed.instance.0,
                         counter: parsed.counter,
@@ -3316,22 +4881,133 @@
                 None => holds_a_record_rejected_by_parse = true,
             }
         }
-        let disk = MemoryDisk { ring, contents };
+        let disk = MemoryDisk {
+            ring,
+            reported_disk_order: self.run_profile.reported_disk_order,
+            contents,
+        };
+        let judge_on = |table: &RecordTable,
+                        implementation_of_the_pass: Implementation,
+                        designated_in_the_pass: bool,
+                        tally_of_the_pass: &mut Tally| {
+            self.judge_the_state_on_a_table(
+                table,
+                &disk,
+                &truths,
+                holds_a_record_rejected_by_parse,
+                implementation_of_the_pass,
+                designated_in_the_pass,
+                tally_of_the_pass,
+            )
+        };
+        let judgement = match self.run_profile.table_source {
+            RecordTableSource::Direct => judge_on(
+                &direct_table,
+                self.implementation,
+                self.is_the_designated_state,
+                tally,
+            ),
+            RecordTableSource::Scanned => {
+                let scanned_table = scanned_record_table_of(&disk, self.implementation);
+                judge_on(
+                    &scanned_table,
+                    self.implementation,
+                    self.is_the_designated_state,
+                    tally,
+                )
+            }
+            RecordTableSource::Both => {
+                let scanned_table = scanned_record_table_of(&disk, self.implementation);
+                let counts_before_the_state = tally.counts.clone();
+                let (scan_pass_implementation, scan_pass_is_designated) =
+                    implementation_and_designation_of_the_scan_pass(
+                        self.implementation,
+                        self.is_the_designated_state,
+                    );
+                let mut scanned_tally = Tally::new();
+                let scanned_judgement = judge_on(
+                    &scanned_table,
+                    scan_pass_implementation,
+                    scan_pass_is_designated,
+                    &mut scanned_tally,
+                );
+                let scanned_summary = JudgedRowSummary {
+                    first_outcome: scanned_judgement.first_outcome,
+                    counter_labels: counter_labels_a_state_added_to(
+                        &scanned_tally,
+                        &Tally::new().counts,
+                    ),
+                };
+                let direct_judgement = judge_on(
+                    &direct_table,
+                    self.implementation,
+                    self.is_the_designated_state,
+                    tally,
+                );
+                let direct_summary = JudgedRowSummary {
+                    first_outcome: direct_judgement.first_outcome,
+                    counter_labels: counter_labels_a_state_added_to(
+                        tally,
+                        &counts_before_the_state,
+                    ),
+                };
+                add_the_comparison_of_two_judged_tables(
+                    tally,
+                    (Counter::TablesDiffer, Counter::RowsDiffer),
+                    (&direct_table, &direct_summary),
+                    (&scanned_table, &scanned_summary),
+                    &|| {
+                        self.describe_the_state_with_its_first_outcome(
+                            &direct_judgement.first_outcome,
+                            &direct_judgement.expectations,
+                        )
+                    },
+                );
+                if !direct_table.is_empty() {
+                    tally.add(Counter::DirectTableHoldsARecord);
+                }
+                direct_judgement
+            }
+        };
+        finish_probing_a_state(probe_decision, tally, &|| {
+            self.describe_the_state_with_its_first_outcome(
+                &judgement.first_outcome,
+                &judgement.expectations,
+            )
+        });
+    }
+
+    /// [`Self::judge_the_state`] 在一张给定的记录表上判一个状态：不管探针、不管两种建表法怎么比，只数这一遍判出来的量。
+    #[allow(
+        clippy::too_many_arguments,
+        reason = "每个参数都是这一判要用的一样东西，凑不成一个带不变量的概念"
+    )]
+    fn judge_the_state_on_a_table(
+        &self,
+        records: &RecordTable,
+        disk: &MemoryDisk<'_>,
+        truths: &[RecordTruthOnDisk],
+        holds_a_record_rejected_by_parse: bool,
+        implementation: Implementation,
+        is_the_designated_state: bool,
+        tally: &mut Tally,
+    ) -> AdversarialJudgement {
+        let in_flight_limit = self.domain.ring().local_in_flight_limit();
         let view = SpecificationView {
             chosen_root: &self.chosen_root,
-            records: &records,
+            records,
             in_flight_limit,
             verify_named_units: true,
-            truths: &truths,
+            truths,
         };
         let expectations =
             FollowReading::REGISTERED.map(|reading| specification_expectation(&view, reading));
-        let today = || call_replay_journal(&disk, &self.chosen_root, &records, true);
-        let first = match self.implementation {
+        let today = || call_replay_journal(disk, &self.chosen_root, records, true);
+        let first = match implementation {
             Implementation::TodayReplayJournal => today(),
             Implementation::Control(control) => match control {
                 ControlKind::PanicsOnTheDesignatedState => {
-                    if self.is_the_designated_state {
+                    if is_the_designated_state {
                         ReplayOutcome::Panicked
                     } else {
                         today()
@@ -3350,7 +5026,8 @@
                     });
                     match record_without_the_flag {
                         Some(record) => ReplayOutcome::Returned(ReturnedReplay {
-                            root: VersionFields::of_record(record).to_root(),
+                            root: VersionFields::of_record(record)
+                                .to_root_rebuilt_over(&self.chosen_root),
                             prefix_applied: 1,
                             verification_passed: 1,
                         }),
@@ -3378,27 +5055,20 @@
                 | ControlKind::IdealWithTheCumulativeAppliedSet
                 | ControlKind::SecondCallMovesOn
                 | ControlKind::ReportsNoFailureOnTheLargestTxg
-                | ControlKind::ReportsOneVerificationWithoutVerifying => today(),
+                | ControlKind::ReportsOneVerificationWithoutVerifying
+                | ControlKind::SecondCallReportsMoreThanOneLastRecordFlag
+                | ControlKind::SecondCallReportsTheChosenRootTxgHasNoSuccessor
+                | ControlKind::IdealFirstCallThenTodaysSecondCall
+                | ControlKind::IdealWithAFieldOfTheLandedVersion(_)
+                | ControlKind::IdealCopyingTheChosenUnmountMarkerWhenApplied
+                | ControlKind::IdealWritingNotWrittenWhenNothingIsApplied
+                | ControlKind::IdealCountingOnlyTheFirstDeviceCopies
+                | ControlKind::ScanTableLosesItsLargestKeyRecord
+                | ControlKind::ScanPassOutcomeBecomesAPanic => today(),
             },
         };
-        let describe_witness = || {
-            format!(
-                "{};first={};expected={}",
-                self.describe_the_state(),
-                describe_outcome(&first),
-                expectations
-                    .iter()
-                    .map(|expectation| match expectation {
-                        SpecificationExpectation::FailsWithoutApplying => "fails".to_string(),
-                        SpecificationExpectation::ReturnsRoot(fields) => format!(
-                            "txg{}:watermark{}",
-                            fields.checkpoint_txg, fields.segment.tree_identifier_watermark
-                        ),
-                    })
-                    .collect::<Vec<String>>()
-                    .join("/")
-            )
-        };
+        let describe_witness =
+            || self.describe_the_state_with_its_first_outcome(&first, &expectations);
 
         tally.add(Counter::States);
         if holds_a_record_rejected_by_parse {
@@ -3478,22 +5148,40 @@
                     tally.add_violation(Counter::LimitViolations, &describe_witness);
                 }
                 judge_the_fixed_point(
-                    self.implementation,
+                    implementation,
+                    self.run_profile,
                     returned,
                     None,
                     above_water_record_count <= in_flight_limit,
-                    &records,
-                    &disk,
+                    records,
+                    disk,
                     true,
                     &describe_witness,
                     tally,
                 );
+                if self.run_profile.judges_j6qn_and_j7() {
+                    judge_the_fields_beyond_the_projection(
+                        &self.chosen_root,
+                        &returned.root,
+                        &describe_witness,
+                        tally,
+                    );
+                }
             }
         }
-        finish_probing_a_state(probe_decision, tally, &describe_witness);
+        AdversarialJudgement {
+            first_outcome: first,
+            expectations,
+        }
     }
 }
 
+/// 一个对抗状态判完的结果：第一次调用的结局与 S 三种读法的期望（逐态行的写法要）。
+struct AdversarialJudgement {
+    first_outcome: ReplayOutcome,
+    expectations: [SpecificationExpectation; 3],
+}
+
 /// 对抗域的一个工作单元：锚点在不在、第一个槽取哪一种，其余的槽在单元里取遍。
 #[derive(Clone, Copy, Debug)]
 struct AdversarialWorkUnit {
@@ -3507,6 +5195,7 @@
     unit: &AdversarialWorkUnit,
     unit_index: usize,
     implementation: Implementation,
+    run_profile: RunProfile,
 ) -> Tally {
     let alphabet = adversarial_record_alphabet();
     let choices = adversarial_slot_choices(domain, alphabet.len());
@@ -3515,8 +5204,9 @@
         slot_count,
         alphabet: &alphabet,
         choices: &choices,
-        chosen_root: adversarial_chosen_root(),
+        chosen_root: adversarial_chosen_root(run_profile),
         implementation,
+        run_profile,
         slots: Vec::new(),
         choice_trail: Vec::new(),
         is_the_designated_state: unit_index == 0 && slot_count == 1,
@@ -3535,11 +5225,12 @@
     domain: Domain,
     size: usize,
     implementation: Implementation,
+    run_profile: RunProfile,
     thread_count: usize,
 ) -> Tally {
     match domain.family() {
         Family::LegalHistory => {
-            let units = legal_work_units(domain, size);
+            let units = legal_work_units(domain, size, run_profile);
             match domain {
                 Domain::MonotoneBigRing | Domain::MonotoneSmallRingSix => {
                     tally_of_work_units(&units, thread_count, &|unit_index, unit| {
@@ -3562,6 +5253,23 @@
                         tally_of_a_legal_history(domain, unit, unit_index, implementation)
                     })
                 }
+                Domain::MirroredBigRing
+                | Domain::MirroredSmallRingSix
+                | Domain::MirroredMonotoneBigRing => {
+                    unreachable!("多盘域走 Family::MultiDisk 那一支")
+                }
+            }
+        }
+        Family::MultiDisk => {
+            let units = legal_work_units(domain, size, run_profile);
+            if domain == Domain::MirroredMonotoneBigRing {
+                tally_of_work_units(&units, thread_count, &|unit_index, unit| {
+                    tally_of_a_mirrored_monotone_history(unit, unit_index, implementation)
+                })
+            } else {
+                tally_of_work_units(&units, thread_count, &|unit_index, unit| {
+                    tally_of_a_mirrored_history(unit, unit_index, implementation)
+                })
             }
         }
         Family::Adversarial => {
@@ -3576,7 +5284,14 @@
                 }
             }
             tally_of_work_units(&units, thread_count, &|unit_index, unit| {
-                tally_of_an_adversarial_work_unit(domain, size, unit, unit_index, implementation)
+                tally_of_an_adversarial_work_unit(
+                    domain,
+                    size,
+                    unit,
+                    unit_index,
+                    implementation,
+                    run_profile,
+                )
             })
         }
     }
@@ -3616,6 +5331,9 @@
                 Domain::TxgOneBelowTheLargest => 4,
                 Domain::TxgAtTheLargest => 3,
                 Domain::Residue => 5,
+                // 登记 r2 5.3：M、M6 到 N = 6，MC 到 Nc = 4。
+                Domain::MirroredBigRing | Domain::MirroredSmallRingSix => 6,
+                Domain::MirroredMonotoneBigRing => 4,
             }),
             Self::SegmentTwo => match domain {
                 Domain::LegalBigRing => Some(8),
@@ -3630,22 +5348,30 @@
                 Domain::TxgOneBelowTheLargest => Some(4),
                 Domain::TxgAtTheLargest => Some(3),
                 Domain::Residue => Some(5),
+                // 第二段的变体不跑多盘域（登记 r2 5.6：多盘是第三段的事）。
+                Domain::MirroredBigRing
+                | Domain::MirroredSmallRingSix
+                | Domain::MirroredMonotoneBigRing => None,
             },
-            Self::Light => Some(match domain {
-                Domain::LegalBigRing => 9,
-                Domain::LegalSmallRingSix => 6,
-                Domain::LegalSmallRingNine => 9,
-                Domain::NamedUnitsVerified | Domain::NamedUnitsNotVerified => 5,
-                Domain::MonotoneBigRing => 6,
-                Domain::MonotoneSmallRingSix => 5,
+            Self::Light => match domain {
+                Domain::LegalBigRing => Some(9),
+                Domain::LegalSmallRingSix => Some(6),
+                Domain::LegalSmallRingNine => Some(9),
+                Domain::NamedUnitsVerified | Domain::NamedUnitsNotVerified => Some(5),
+                Domain::MonotoneBigRing => Some(6),
+                Domain::MonotoneSmallRingSix => Some(5),
                 Domain::AdversarialBigRing
                 | Domain::AdversarialSmallRingSix
-                | Domain::AdversarialBackChain => 3,
-                Domain::CrossInstance => 7,
-                Domain::TxgOneBelowTheLargest => 4,
-                Domain::TxgAtTheLargest => 3,
-                Domain::Residue => 5,
-            }),
+                | Domain::AdversarialBackChain => Some(3),
+                Domain::CrossInstance => Some(7),
+                Domain::TxgOneBelowTheLargest => Some(4),
+                Domain::TxgAtTheLargest => Some(3),
+                Domain::Residue => Some(5),
+                // `light` 是 r1 档的轻量档（用户 2026-10-01 定），不跑多盘域。
+                Domain::MirroredBigRing
+                | Domain::MirroredSmallRingSix
+                | Domain::MirroredMonotoneBigRing => None,
+            },
         }
     }
 }
@@ -3700,6 +5426,58 @@
             domain.label()
         );
     }
+    if let Some(pinned) = pinned_mirrored_counts(domain).get(size_index) {
+        let class_states = |class: LaggingClass| tally.lagging_class_count(class, Counter::States);
+        assert_eq!(
+            [
+                class_states(LaggingClass::Equal),
+                class_states(LaggingClass::FirstDeviceBehind)
+                    + class_states(LaggingClass::SecondDeviceBehind),
+                class_states(LaggingClass::Crossed),
+                class_states(LaggingClass::FirstDeviceBehind),
+                class_states(LaggingClass::SecondDeviceBehind),
+                tally.get(Counter::FarthestAtLeastOne),
+                tally.get(Counter::FarthestAtLeastTwo),
+            ],
+            [
+                pinned.equal_states,
+                pinned.lagging_states,
+                pinned.crossed_states,
+                pinned.lagging_states_in_one_direction,
+                pinned.lagging_states_in_one_direction,
+                pinned.farthest_at_least_one,
+                pinned.farthest_at_least_two,
+            ],
+            "作废条款 SV7：{} 域 N = {size} 的落后类别或见证数与登记 r2 第七节乙类不等（次序：相等、落后、互不包含、0 号盘落后、1 号盘落后、j* ≥ 1、j* ≥ 2）",
+            domain.label()
+        );
+        assert_eq!(
+            LaggingClass::ALL
+                .iter()
+                .map(|class| class_states(*class))
+                .sum::<u64>(),
+            tally.get(Counter::States),
+            "作废条款 SV7：{} 域 N = {size} 四类状态数之和不等于状态数（登记 r2 5.5「落后类别」那一行）",
+            domain.label()
+        );
+    }
+    if let Some(pinned) = pinned_mirrored_monotone_counts(domain).get(size_index) {
+        assert_eq!(
+            [
+                tally.pair(PairSet::Wide, PairColumn::Pairs),
+                tally.get(Counter::ChosenRootIsALaterVersion),
+                tally.get(Counter::FarthestAtLeastOne),
+                tally.get(Counter::FarthestAtLeastTwo),
+            ],
+            [
+                pinned.wide_pairs,
+                pinned.chosen_root_is_a_later_version,
+                pinned.farthest_at_least_one,
+                pinned.farthest_at_least_two,
+            ],
+            "作废条款 SV7：MC 域 Nc = {size} 的宽对数或见证数与登记 r2 第七节乙类不等（次序：宽对数、所选根不是 R_0、j* − c ≥ 1、j* − c ≥ 2）"
+        );
+    }
 }
 
 // ── 阳性对照（登记 5.4）：判据表的每一行一条假结局，必须红
@@ -3725,6 +5503,18 @@
         AMutual => "A_mutual",
         AStructure => "A_structure",
         ALimit => "A_limit",
+        J6QnFixedPoint => "J6Qn",
+        J7aInstanceTable => "J7a_instance_table",
+        J7aAllocationRecordTreeRoot => "J7a_allocation_record_tree_root",
+        J7aFilesystemIdentifier => "J7a_filesystem_identifier",
+        J7bUnmountMarkerOfARebuiltRoot => "J7b",
+        J7cUnmountMarkerWhenNothingIsApplied => "J7c",
+        MultiDiskAnyCopyIsOnDisk => "multi_disk_any_copy",
+        MultiDiskReturnedRoot => "J1_multi_disk",
+        MultiDiskFarthestVersion => "J4_multi_disk",
+        MultiDiskAppliedSetAndLandedVersion => "J5_multi_disk",
+        TableComparisonSeesADifferentTable => "T_table",
+        TableComparisonSeesADifferentRow => "T_row",
     }
 }
 
@@ -3741,7 +5531,15 @@
             | Self::J5aAndJ5bAreSeparable
             | Self::J6FixedPoint
             | Self::Ia1LargestTxgFails
-            | Self::Ia2NoVerificationCounted => Family::LegalHistory,
+            | Self::Ia2NoVerificationCounted
+            | Self::J6QnFixedPoint
+            | Self::J7aInstanceTable
+            | Self::J7aAllocationRecordTreeRoot
+            | Self::J7aFilesystemIdentifier
+            | Self::J7bUnmountMarkerOfARebuiltRoot
+            | Self::J7cUnmountMarkerWhenNothingIsApplied
+            | Self::TableComparisonSeesADifferentTable
+            | Self::TableComparisonSeesADifferentRow => Family::LegalHistory,
             Self::A0Defined
             | Self::AsWholeTable
             | Self::AsGapContinues
@@ -3749,6 +5547,55 @@
             | Self::AMutual
             | Self::AStructure
             | Self::ALimit => Family::Adversarial,
+            Self::MultiDiskAnyCopyIsOnDisk
+            | Self::MultiDiskReturnedRoot
+            | Self::MultiDiskFarthestVersion
+            | Self::MultiDiskAppliedSetAndLandedVersion => Family::MultiDisk,
+        }
+    }
+
+    /// T-表与 T-行判的是每个域上两种建表法的逐态比，对照只拿 L 域跑一遍；每个带 `--table both` 的跑法都带这两条对照，不管选的是哪一族。
+    const fn is_judged_in_every_family(self) -> bool {
+        matches!(
+            self,
+            Self::TableComparisonSeesADifferentTable | Self::TableComparisonSeesADifferentRow
+        )
+    }
+
+    /// 这一档判不判这条判据：J6Qn 与 J7 只在 r2 档判（登记 r2 5.4），T-表与 T-行只在 `--table both` 时判，别的两档都判。
+    const fn is_judged_under(self, run_profile: RunProfile) -> bool {
+        match self {
+            Self::TableComparisonSeesADifferentTable | Self::TableComparisonSeesADifferentRow => {
+                run_profile.compares_the_two_record_tables()
+            }
+            Self::J6QnFixedPoint
+            | Self::J7aInstanceTable
+            | Self::J7aAllocationRecordTreeRoot
+            | Self::J7aFilesystemIdentifier
+            | Self::J7bUnmountMarkerOfARebuiltRoot
+            | Self::J7cUnmountMarkerWhenNothingIsApplied
+            | Self::MultiDiskAnyCopyIsOnDisk
+            | Self::MultiDiskReturnedRoot
+            | Self::MultiDiskFarthestVersion
+            | Self::MultiDiskAppliedSetAndLandedVersion => run_profile.judges_j6qn_and_j7(),
+            Self::J0Defined
+            | Self::J1ReturnedRoot
+            | Self::J2RootPlusDisk
+            | Self::J3WithinTheInFlightLimit
+            | Self::J4FarthestVersion
+            | Self::J5aAppliedSet
+            | Self::J5bLandedVersion
+            | Self::J5aAndJ5bAreSeparable
+            | Self::J6FixedPoint
+            | Self::Ia1LargestTxgFails
+            | Self::Ia2NoVerificationCounted
+            | Self::A0Defined
+            | Self::AsWholeTable
+            | Self::AsGapContinues
+            | Self::AsGapStops
+            | Self::AMutual
+            | Self::AStructure
+            | Self::ALimit => true,
         }
     }
 }
@@ -3770,20 +5617,60 @@
 const CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN: usize = 2;
 /// 对照界里 L 域 j\* ≥ 1 的态数：登记第七节见证数 N = 1..5 的和，0 + 2 + 16 + 104 + 640。
 const FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND: u64 = 762;
+/// 对照界里 L 域的态数：登记 r2 第七节乙类「L 大环对照界 N ≤ 5：态数 3 110」（2 + 12 + 72 + 432 + 2 592）。
+const STATES_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN: u64 = 3_110;
+/// 对照界里 L 域 j\* = 0 的态数：登记 r2 第七节乙类「j\* = 0 2 348」。
+const FARTHEST_IS_THE_CHOSEN_VERSION_STATES_WITHIN_THE_CONTROL_BOUND: u64 = 2_348;
+/// E1 全界（N ≤ 4）上 j\* = 1 的态数：登记 r2 第七节乙类「E1 上 j\* = 1 的态数 0、2、12、56，合 70」。
+const E1_STATES_WHOSE_FARTHEST_VERSION_TXG_IS_THE_LARGEST: u64 = 70;
+/// 多盘对照的界（登记 r2 5.5 末段）：M 到 N = 3，M6 到 N = 4，MC 到 Nc = 2。
+const CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_BIG_RING_DOMAIN: usize = 3;
+const CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_SMALL_RING_SIX_DOMAIN: usize = 4;
+const CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_MONOTONE_DOMAIN: usize = 2;
+/// 登记 r2 第七节「多盘「任一份在即在」」：只数 0 号盘的 j\* 与任一份的 j\* 不等的态数，M 域 N ≤ 3 是 0 + 4 + 96，
+/// M6 域 N ≤ 4 是 0 + 4 + 96 + 896（`anchors_r2c.py` 的 `jstar_disk0_only_differs`）。
+const MIRRORED_BIG_RING_STATES_WHERE_THE_FIRST_DEVICE_ALONE_GIVES_ANOTHER_FARTHEST_VERSION: u64 =
+    100;
+const MIRRORED_SMALL_RING_SIX_STATES_WHERE_THE_FIRST_DEVICE_ALONE_GIVES_ANOTHER_FARTHEST_VERSION:
+    u64 = 996;
+/// 原登记 J1、J4 两条对照在多盘域上的绝对值（登记 r2 第七节）：j\* − c ≥ 1 的态数，M 域 N ≤ 3 是 0 + 12 + 216，
+/// M6 域 N ≤ 4 是 0 + 12 + 216 + 1 824，MC 域 Nc ≤ 2 是 3 + 42。
+const MIRRORED_BIG_RING_FARTHEST_AT_LEAST_ONE_WITHIN_THE_CONTROL_BOUND: u64 = 228;
+const MIRRORED_SMALL_RING_SIX_FARTHEST_AT_LEAST_ONE_WITHIN_THE_CONTROL_BOUND: u64 = 2_052;
+const MIRRORED_MONOTONE_FARTHEST_AT_LEAST_ONE_WITHIN_THE_CONTROL_BOUND: u64 = 45;
+
+/// 一条假结局跑在哪个档上：T-表、T-行两条对照拿 `--table both`，别的对照一律拿直接建表、盘次序照常（对照证的是判据分得出差别，
+/// 与这一次跑选了哪张表、盘报什么次序无关；这样 `--table both` 的跑法里别的对照读数与默认跑法逐字相同）。
+fn run_profile_of_a_control(control: ControlKind, run_profile: RunProfile) -> RunProfile {
+    let without_the_record_table_knobs = run_profile
+        .with_table_source(RecordTableSource::Direct)
+        .with_reported_disk_order(ReportedDiskOrder::FirstDeviceBeforeSecond);
+    if matches!(
+        control,
+        ControlKind::ScanTableLosesItsLargestKeyRecord | ControlKind::ScanPassOutcomeBecomesAPanic
+    ) {
+        without_the_record_table_knobs.with_table_source(RecordTableSource::Both)
+    } else {
+        without_the_record_table_knobs
+    }
+}
 
 /// 一条假结局在一个域上、从最小的 N 扫到 `largest_size` 的合计。
 fn control_tally(
     domain: Domain,
     largest_size: usize,
     control: ControlKind,
+    run_profile: RunProfile,
     thread_count: usize,
 ) -> Tally {
+    let run_profile = run_profile_of_a_control(control, run_profile);
     let mut total = Tally::new();
     for size in domain.smallest_size()..=largest_size {
         total.merge(&tally_of_a_domain_size(
             domain,
             size,
             Implementation::Control(control),
+            run_profile,
             thread_count,
         ));
     }
@@ -3791,7 +5678,10 @@
 }
 
 /// AS 三条的对照要 A-互 三个数都不为 0；对照界 n ≤ 2 里三个数全为 0 时照登记 5.4 放宽 n，一档一档放到第一段的界为止。
-fn smallest_adversarial_size_where_the_three_readings_part(thread_count: usize) -> usize {
+fn smallest_adversarial_size_where_the_three_readings_part(
+    run_profile: RunProfile,
+    thread_count: usize,
+) -> usize {
     let largest_registered_size = BoundsTier::SegmentOne
         .largest_size(Domain::AdversarialBigRing)
         .expect("第一段跑 A 域");
@@ -3800,6 +5690,7 @@
             Domain::AdversarialBigRing,
             largest_size,
             ControlKind::SpecificationWithReading(FollowReading::WholeTable),
+            run_profile,
             thread_count,
         );
         if tally.get(Counter::MutualWholeTableAndGapContinues) > 0
@@ -3847,13 +5738,16 @@
     other_reading: FollowReading,
     violation_counter: Counter,
     mutual_counter: Counter,
+    run_profile: RunProfile,
     thread_count: usize,
 ) -> Vec<ControlReading> {
-    let largest_size = smallest_adversarial_size_where_the_three_readings_part(thread_count);
+    let largest_size =
+        smallest_adversarial_size_where_the_three_readings_part(run_profile, thread_count);
     let tally = control_tally(
         Domain::AdversarialBigRing,
         largest_size,
         ControlKind::SpecificationWithReading(other_reading),
+        run_profile,
         thread_count,
     );
     vec![
@@ -3879,12 +5773,17 @@
 }
 
 /// 判据表一行的阳性对照（登记 5.4 那张表的一行）。
-fn control_readings_of(criterion: Criterion, thread_count: usize) -> Vec<ControlReading> {
+fn control_readings_of(
+    criterion: Criterion,
+    run_profile: RunProfile,
+    thread_count: usize,
+) -> Vec<ControlReading> {
     let big_ring_legal = |control: ControlKind| {
         control_tally(
             Domain::LegalBigRing,
             CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
             control,
+            run_profile,
             thread_count,
         )
     };
@@ -3893,6 +5792,7 @@
             Domain::AdversarialBigRing,
             largest_size,
             control,
+            run_profile,
             thread_count,
         )
     };
@@ -3935,6 +5835,7 @@
                 Domain::NamedUnitsVerified,
                 CONTROL_LARGEST_SIZE_OF_THE_NAMED_UNITS_DOMAIN,
                 ControlKind::LandsOnePublishTooFarOverAnIncompletePublish,
+                run_profile,
                 thread_count,
             );
             vec![
@@ -3966,6 +5867,7 @@
                 Domain::LegalSmallRingSix,
                 small_ring_largest_size,
                 ControlKind::IgnoresTheInFlightLimit,
+                run_profile,
                 thread_count,
             );
             let big_ring_tally = big_ring_legal(ControlKind::IgnoresTheInFlightLimit);
@@ -4017,6 +5919,7 @@
                 Domain::MonotoneBigRing,
                 CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN,
                 ControlKind::ReturnsTheChosenRootWhenEveryRecordIsOnDisk,
+                run_profile,
                 thread_count,
             );
             let column = if criterion == Criterion::J5aAppliedSet {
@@ -4049,6 +5952,7 @@
                     domain,
                     largest_size,
                     ControlKind::IdealWithTheCumulativeAppliedSet,
+                    run_profile,
                     thread_count,
                 );
                 for set in PairSet::ALL {
@@ -4083,6 +5987,7 @@
                 Domain::TxgAtTheLargest,
                 largest_size,
                 ControlKind::ReportsNoFailureOnTheLargestTxg,
+                run_profile,
                 thread_count,
             );
             vec![reading_that_must_be_positive(
@@ -4096,6 +6001,7 @@
                 Domain::NamedUnitsNotVerified,
                 CONTROL_LARGEST_SIZE_OF_THE_NAMED_UNITS_DOMAIN,
                 ControlKind::ReportsOneVerificationWithoutVerifying,
+                run_profile,
                 thread_count,
             );
             vec![reading_that_must_be_positive(
@@ -4125,6 +6031,7 @@
             FollowReading::ConsecutiveGapContinues,
             Counter::AsWholeTableViolations,
             Counter::MutualWholeTableAndGapContinues,
+            run_profile,
             thread_count,
         ),
         Criterion::AsGapContinues => specification_control_readings(
@@ -4132,6 +6039,7 @@
             FollowReading::ConsecutiveGapStops,
             Counter::AsGapContinuesViolations,
             Counter::MutualGapContinuesAndGapStops,
+            run_profile,
             thread_count,
         ),
         Criterion::AsGapStops => specification_control_readings(
@@ -4139,11 +6047,12 @@
             FollowReading::WholeTable,
             Counter::AsGapStopsViolations,
             Counter::MutualWholeTableAndGapStops,
+            run_profile,
             thread_count,
         ),
         Criterion::AMutual => {
             let largest_size =
-                smallest_adversarial_size_where_the_three_readings_part(thread_count);
+                smallest_adversarial_size_where_the_three_readings_part(run_profile, thread_count);
             let tally = adversarial(
                 largest_size,
                 ControlKind::SpecificationWithReading(FollowReading::NeverJudges),
@@ -4183,17 +6092,558 @@
                 tally.get(Counter::States),
             )]
         }
+        Criterion::J6QnFixedPoint => j6qn_control_readings(run_profile, thread_count),
+        Criterion::J7aInstanceTable => j7a_control_readings(
+            criterion,
+            FieldOfTheRebuiltRoot::InstanceTable,
+            run_profile,
+            thread_count,
+        ),
+        Criterion::J7aAllocationRecordTreeRoot => j7a_control_readings(
+            criterion,
+            FieldOfTheRebuiltRoot::AllocationRecordTreeRoot,
+            run_profile,
+            thread_count,
+        ),
+        Criterion::J7aFilesystemIdentifier => j7a_control_readings(
+            criterion,
+            FieldOfTheRebuiltRoot::FilesystemIdentifier,
+            run_profile,
+            thread_count,
+        ),
+        Criterion::J7bUnmountMarkerOfARebuiltRoot => unmount_marker_control_readings(
+            criterion,
+            ControlKind::IdealCopyingTheChosenUnmountMarkerWhenApplied,
+            Counter::J7bViolations,
+            FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND,
+            run_profile,
+            thread_count,
+        ),
+        Criterion::J7cUnmountMarkerWhenNothingIsApplied => unmount_marker_control_readings(
+            criterion,
+            ControlKind::IdealWritingNotWrittenWhenNothingIsApplied,
+            Counter::J7cViolations,
+            FARTHEST_IS_THE_CHOSEN_VERSION_STATES_WITHIN_THE_CONTROL_BOUND,
+            run_profile,
+            thread_count,
+        ),
+        Criterion::MultiDiskAnyCopyIsOnDisk => any_copy_control_readings(run_profile, thread_count),
+        Criterion::MultiDiskReturnedRoot => {
+            mirrored_returned_root_control_readings(run_profile, thread_count)
+        }
+        Criterion::MultiDiskFarthestVersion => {
+            mirrored_farthest_version_control_readings(run_profile, thread_count)
+        }
+        Criterion::MultiDiskAppliedSetAndLandedVersion => {
+            mirrored_monotone_control_readings(run_profile, thread_count)
+        }
+        Criterion::TableComparisonSeesADifferentTable => {
+            table_comparison_control_readings_on_tables(criterion, run_profile, thread_count)
+        }
+        Criterion::TableComparisonSeesADifferentRow => {
+            table_comparison_control_readings_on_rows(criterion, run_profile, thread_count)
+        }
+    }
+}
+
+/// L 域 N ≤ 5 上直接建的表非空的态数（至少一条记录落了盘的态数）：登记 r2 第七节乙类「L 大环对照界 N ≤ 5：……至少一条记录落盘的态数 2 989」
+/// （每段历史的 2^N 个落盘子集里去掉空集：1 + 9 + 63 + 405 + 2 511）。
+const STATES_WITH_A_RECORD_ON_DISK_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN: u64 =
+    2_989;
+
+/// T-表的对照（登记 r2 5.5、第七节）：扫盘建表之后删掉键最大的那条记录、直接建的表不动。L 域 N ≤ 5 上 `tables_differ` 等于直接建的表非空的态数
+/// （装置另数一遍，钉 2 989），`rows_differ` 大于 0。
+fn table_comparison_control_readings_on_tables(
+    criterion: Criterion,
+    run_profile: RunProfile,
+    thread_count: usize,
+) -> Vec<ControlReading> {
+    let tally = control_tally(
+        Domain::LegalBigRing,
+        CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
+        ControlKind::ScanTableLosesItsLargestKeyRecord,
+        run_profile,
+        thread_count,
+    );
+    let place = |counter: Counter| {
+        control_place(
+            Domain::LegalBigRing,
+            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
+            "scan_table_loses_its_largest_key_record",
+            counter,
+        )
+    };
+    vec![
+        reading_that_must_equal(
+            criterion,
+            place(Counter::TablesDiffer),
+            tally.get(Counter::TablesDiffer),
+            tally.get(Counter::DirectTableHoldsARecord),
+        ),
+        reading_that_must_equal(
+            criterion,
+            place(Counter::DirectTableHoldsARecord),
+            tally.get(Counter::DirectTableHoldsARecord),
+            STATES_WITH_A_RECORD_ON_DISK_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN,
+        ),
+        reading_that_must_be_positive(
+            criterion,
+            place(Counter::RowsDiffer),
+            tally.get(Counter::RowsDiffer),
+        ),
+    ]
+}
+
+/// T-行的对照（登记 r2 5.5、第七节）：扫盘建表那一遍的实现结局换成 `Panicked`、表不动。L 域 N ≤ 5 上 `rows_differ` 等于状态数（3 110）、
+/// `tables_differ` 是 0（两格分得开）。
+fn table_comparison_control_readings_on_rows(
+    criterion: Criterion,
+    run_profile: RunProfile,
+    thread_count: usize,
+) -> Vec<ControlReading> {
+    let tally = control_tally(
+        Domain::LegalBigRing,
+        CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
+        ControlKind::ScanPassOutcomeBecomesAPanic,
+        run_profile,
+        thread_count,
+    );
+    let place = |counter: Counter| {
+        control_place(
+            Domain::LegalBigRing,
+            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
+            "scan_pass_outcome_becomes_a_panic",
+            counter,
+        )
+    };
+    vec![
+        reading_that_must_equal(
+            criterion,
+            place(Counter::RowsDiffer),
+            tally.get(Counter::RowsDiffer),
+            STATES_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN,
+        ),
+        reading_that_must_equal(
+            criterion,
+            place(Counter::States),
+            tally.get(Counter::States),
+            STATES_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN,
+        ),
+        reading_that_must_equal(
+            criterion,
+            place(Counter::TablesDiffer),
+            tally.get(Counter::TablesDiffer),
+            0,
+        ),
+    ]
+}
+
+/// 多盘三个域与各自对照的界、j\* − c ≥ 1 的态数（登记 r2 第七节）。
+const MIRRORED_DOMAINS_WITH_THE_CONTROL_BOUND: [(Domain, usize, u64); 3] = [
+    (
+        Domain::MirroredBigRing,
+        CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_BIG_RING_DOMAIN,
+        MIRRORED_BIG_RING_FARTHEST_AT_LEAST_ONE_WITHIN_THE_CONTROL_BOUND,
+    ),
+    (
+        Domain::MirroredSmallRingSix,
+        CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_SMALL_RING_SIX_DOMAIN,
+        MIRRORED_SMALL_RING_SIX_FARTHEST_AT_LEAST_ONE_WITHIN_THE_CONTROL_BOUND,
+    ),
+    (
+        Domain::MirroredMonotoneBigRing,
+        CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_MONOTONE_DOMAIN,
+        MIRRORED_MONOTONE_FARTHEST_AT_LEAST_ONE_WITHIN_THE_CONTROL_BOUND,
+    ),
+];
+
+fn control_place(
+    domain: Domain,
+    largest_size: usize,
+    implementation_label: &str,
+    counter: Counter,
+) -> String {
+    format!(
+        "domain={},max_n={largest_size},implementation={implementation_label},counter={}",
+        domain.label(),
+        counter.label()
+    )
+}
+
+/// 多盘「任一份在即在」（登记 r2 5.5、第七节）：理想恢复但 j\* 只数 0 号盘那一份。M 域 N ≤ 3 的 J4 = 100、M6 域 N ≤ 4 = 996，
+/// L 域 N ≤ 5（单盘，0 号盘那一份就是全部）= 0。
+fn any_copy_control_readings(run_profile: RunProfile, thread_count: usize) -> Vec<ControlReading> {
+    let control = ControlKind::IdealCountingOnlyTheFirstDeviceCopies;
+    [
+        (
+            Domain::MirroredBigRing,
+            CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_BIG_RING_DOMAIN,
+            MIRRORED_BIG_RING_STATES_WHERE_THE_FIRST_DEVICE_ALONE_GIVES_ANOTHER_FARTHEST_VERSION,
+        ),
+        (
+            Domain::MirroredSmallRingSix,
+            CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_SMALL_RING_SIX_DOMAIN,
+            MIRRORED_SMALL_RING_SIX_STATES_WHERE_THE_FIRST_DEVICE_ALONE_GIVES_ANOTHER_FARTHEST_VERSION,
+        ),
+        (
+            Domain::LegalBigRing,
+            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
+            0,
+        ),
+    ]
+    .into_iter()
+    .map(|(domain, largest_size, expected)| {
+        let tally = control_tally(domain, largest_size, control, run_profile, thread_count);
+        reading_that_must_equal(
+            Criterion::MultiDiskAnyCopyIsOnDisk,
+            control_place(domain, largest_size, "ideal_counting_only_the_first_device_copies", Counter::J4Violations),
+            tally.get(Counter::J4Violations),
+            expected,
+        )
+    })
+    .collect()
+}
+
+/// 原登记 J1 那条对照（施加过就返回时间线之外的根）在多盘三个域上：J1 违例数与 j\* − c ≥ 1 的态数都等于第七节的值。
+fn mirrored_returned_root_control_readings(
+    run_profile: RunProfile,
+    thread_count: usize,
+) -> Vec<ControlReading> {
+    let mut readings = Vec::new();
+    for (domain, largest_size, farthest_at_least_one) in MIRRORED_DOMAINS_WITH_THE_CONTROL_BOUND {
+        let tally = control_tally(
+            domain,
+            largest_size,
+            ControlKind::OffTimelineRootWheneverApplied,
+            run_profile,
+            thread_count,
+        );
+        for counter in [Counter::J1Violations, Counter::FarthestAtLeastOne] {
+            readings.push(reading_that_must_equal(
+                Criterion::MultiDiskReturnedRoot,
+                control_place(
+                    domain,
+                    largest_size,
+                    "off_timeline_root_whenever_applied",
+                    counter,
+                ),
+                tally.get(counter),
+                farthest_at_least_one,
+            ));
+        }
+    }
+    readings
+}
+
+/// 原登记 J4 那条对照（永远返回所选根）在多盘三个域上：J4 违例数等于第七节的值，J1、J2、J3 是 0。
+fn mirrored_farthest_version_control_readings(
+    run_profile: RunProfile,
+    thread_count: usize,
+) -> Vec<ControlReading> {
+    let mut readings = Vec::new();
+    for (domain, largest_size, farthest_at_least_one) in MIRRORED_DOMAINS_WITH_THE_CONTROL_BOUND {
+        let tally = control_tally(
+            domain,
+            largest_size,
+            ControlKind::AlwaysReturnsTheChosenRoot,
+            run_profile,
+            thread_count,
+        );
+        for (counter, expected) in [
+            (Counter::J4Violations, farthest_at_least_one),
+            (Counter::J1Violations, 0),
+            (Counter::J2Violations, 0),
+            (Counter::J3Violations, 0),
+        ] {
+            readings.push(reading_that_must_equal(
+                Criterion::MultiDiskFarthestVersion,
+                control_place(
+                    domain,
+                    largest_size,
+                    "always_returns_the_chosen_root",
+                    counter,
+                ),
+                tally.get(counter),
+                expected,
+            ));
+        }
+    }
+    readings
+}
+
+/// MC 宽对上 J5a、J5b 的对照（登记 r2 第十二节修订 3 补的；原登记 5.4 J5 那条假结局搬到多盘）：记录每条都有一份在盘上时返回所选根，
+/// 其余照理想恢复；MC 域 Nc ≤ 2 宽对上今天那两列都大于 0。
+fn mirrored_monotone_control_readings(
+    run_profile: RunProfile,
+    thread_count: usize,
+) -> Vec<ControlReading> {
+    let tally = control_tally(
+        Domain::MirroredMonotoneBigRing,
+        CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_MONOTONE_DOMAIN,
+        ControlKind::ReturnsTheChosenRootWhenEveryRecordIsOnDisk,
+        run_profile,
+        thread_count,
+    );
+    [PairColumn::TodayAppliedSetViolations, PairColumn::TodayLandedVersionViolations]
+        .into_iter()
+        .map(|column| {
+            reading_that_must_be_positive(
+                Criterion::MultiDiskAppliedSetAndLandedVersion,
+                format!(
+                    "domain=MC,max_n={CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_MONOTONE_DOMAIN},implementation=returns_the_chosen_root_when_every_record_is_on_disk,set=wide,column={}",
+                    column.label()
+                ),
+                tally.pair(PairSet::Wide, column),
+            )
+        })
+        .collect()
+}
+
+/// J6Qn 的三条对照（登记 r2 5.5；第七节把它们钉成等式：L 域 N ≤ 5 两条各 3 110，E1 全界原 J6 70、J6Qn 0）。
+fn j6qn_control_readings(run_profile: RunProfile, thread_count: usize) -> Vec<ControlReading> {
+    let criterion = Criterion::J6QnFixedPoint;
+    let sum_of_both_premises = |tally: &Tally, premise_holds: Counter, premise_fails: Counter| {
+        tally.get(premise_holds) + tally.get(premise_fails)
+    };
+    let mut readings = Vec::new();
+    for (control, implementation_label) in [
+        (
+            ControlKind::SecondCallReportsMoreThanOneLastRecordFlag,
+            "second_call_reports_more_than_one_last_record_flag",
+        ),
+        (
+            ControlKind::SecondCallReportsTheChosenRootTxgHasNoSuccessor,
+            "second_call_reports_chosen_root_txg_has_no_successor",
+        ),
+    ] {
+        let tally = control_tally(
+            Domain::LegalBigRing,
+            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
+            control,
+            run_profile,
+            thread_count,
+        );
+        let place = format!(
+            "domain=L,max_n={CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN},implementation={implementation_label}"
+        );
+        readings.push(reading_that_must_equal(
+            criterion,
+            format!("{place},counter=J6Qn_premise_holds_plus_J6Qn_premise_fails"),
+            sum_of_both_premises(
+                &tally,
+                Counter::J6QnPremiseHoldsViolations,
+                Counter::J6QnPremiseFailsViolations,
+            ),
+            STATES_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN,
+        ));
+        readings.push(reading_that_must_equal(
+            criterion,
+            format!("{place},counter=j6qn_premise_holds_judged_plus_j6qn_premise_fails_judged"),
+            sum_of_both_premises(
+                &tally,
+                Counter::J6QnPremiseHoldsJudged,
+                Counter::J6QnPremiseFailsJudged,
+            ),
+            STATES_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN,
+        ));
+        if control == ControlKind::SecondCallReportsTheChosenRootTxgHasNoSuccessor {
+            readings.push(reading_that_must_equal(
+                criterion,
+                format!("{place},counter=J6_premise_holds_plus_J6_premise_fails"),
+                sum_of_both_premises(
+                    &tally,
+                    Counter::J6PremiseHoldsViolations,
+                    Counter::J6PremiseFailsViolations,
+                ),
+                STATES_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN,
+            ));
+        }
+    }
+    let e1_largest_size = BoundsTier::SegmentOne
+        .largest_size(Domain::TxgOneBelowTheLargest)
+        .expect("第一段跑 E1");
+    let tally = control_tally(
+        Domain::TxgOneBelowTheLargest,
+        e1_largest_size,
+        ControlKind::IdealFirstCallThenTodaysSecondCall,
+        run_profile,
+        thread_count,
+    );
+    let place = format!(
+        "domain=E1,max_n={e1_largest_size},implementation=ideal_first_call_then_todays_second_call"
+    );
+    readings.push(reading_that_must_equal(
+        criterion,
+        format!("{place},counter=J6_premise_holds_plus_J6_premise_fails"),
+        sum_of_both_premises(
+            &tally,
+            Counter::J6PremiseHoldsViolations,
+            Counter::J6PremiseFailsViolations,
+        ),
+        E1_STATES_WHOSE_FARTHEST_VERSION_TXG_IS_THE_LARGEST,
+    ));
+    readings.push(reading_that_must_equal(
+        criterion,
+        format!("{place},counter=J6Qn_premise_holds_plus_J6Qn_premise_fails"),
+        sum_of_both_premises(
+            &tally,
+            Counter::J6QnPremiseHoldsViolations,
+            Counter::J6QnPremiseFailsViolations,
+        ),
+        0,
+    ));
+    readings
+}
+
+/// J7a 一格的对照（登记 r2 5.5、第七节）：理想恢复、施加过时把这个字段换成落到的那一版的。L 域 N ≤ 5 上这一格 = 762、另两格 = 0；
+/// 同一个对照在 r1 档生成器上这一格 = 0（判别力自证：看得见是生成器换了的结果）。
+fn j7a_control_readings(
+    criterion: Criterion,
+    field: FieldOfTheRebuiltRoot,
+    run_profile: RunProfile,
+    thread_count: usize,
+) -> Vec<ControlReading> {
+    let control = ControlKind::IdealWithAFieldOfTheLandedVersion(field);
+    let place = format!(
+        "domain=L,max_n={CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN},implementation=ideal_with_the_landed_versions_{}",
+        field.label()
+    );
+    let tally = control_tally(
+        Domain::LegalBigRing,
+        CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
+        control,
+        run_profile,
+        thread_count,
+    );
+    let mut readings = Vec::new();
+    for judged_field in FieldOfTheRebuiltRoot::ALL {
+        let counter = judged_field.violations_counter();
+        readings.push(reading_that_must_equal(
+            criterion,
+            format!("{place},counter={}", counter.label()),
+            tally.get(counter),
+            if judged_field == field {
+                FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND
+            } else {
+                0
+            },
+        ));
+    }
+    let r1_generator_tally = control_tally(
+        Domain::LegalBigRing,
+        CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
+        control,
+        RunProfile::R1_GENERATOR_WITH_R2_CRITERIA,
+        thread_count,
+    );
+    readings.push(reading_that_must_equal(
+        criterion,
+        format!(
+            "{place},generator=r1,counter={}",
+            field.violations_counter().label()
+        ),
+        r1_generator_tally.get(field.violations_counter()),
+        0,
+    ));
+    readings
+}
+
+/// J7b、J7c 的对照（登记 r2 5.5、第七节、第八节奇偶的判别力自证）。偶数版带记号（R_0 是 `Written…`）：L 域 N ≤ 5 上
+/// 等于 `count_when_the_first_version_is_written`，r1 档生成器上 0。奇数版带记号（R_0 是 `NotWritten…`）：L 域 0（假结局与期望同值），
+/// C 域 Nc ≤ 3 大于 0（所选根是第几版取决于哪些根槽落了，两种记号的所选根都有）。
+fn unmount_marker_control_readings(
+    criterion: Criterion,
+    control: ControlKind,
+    counter: Counter,
+    count_when_the_first_version_is_written: u64,
+    run_profile: RunProfile,
+    thread_count: usize,
+) -> Vec<ControlReading> {
+    let big_ring_place = format!(
+        "domain=L,max_n={CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN},implementation={},counter={}",
+        criterion.label(),
+        counter.label()
+    );
+    let big_ring_tally = |profile: RunProfile| {
+        control_tally(
+            Domain::LegalBigRing,
+            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
+            control,
+            profile,
+            thread_count,
+        )
+        .get(counter)
+    };
+    let r1_generator_reading = reading_that_must_equal(
+        criterion,
+        format!("{big_ring_place},generator=r1"),
+        big_ring_tally(RunProfile::R1_GENERATOR_WITH_R2_CRITERIA),
+        0,
+    );
+    match run_profile.generator {
+        FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(
+            MarkerParity::EvenVersionsWrittenByTheUnmountSequence,
+        ) => vec![
+            reading_that_must_equal(
+                criterion,
+                format!("{big_ring_place},marker_parity=even"),
+                big_ring_tally(run_profile),
+                count_when_the_first_version_is_written,
+            ),
+            r1_generator_reading,
+        ],
+        FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(
+            MarkerParity::OddVersionsWrittenByTheUnmountSequence,
+        ) => {
+            let monotone_tally = control_tally(
+                Domain::MonotoneBigRing,
+                CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN,
+                control,
+                run_profile,
+                thread_count,
+            );
+            vec![
+                reading_that_must_equal(
+                    criterion,
+                    format!("{big_ring_place},marker_parity=odd"),
+                    big_ring_tally(run_profile),
+                    0,
+                ),
+                reading_that_must_be_positive(
+                    criterion,
+                    format!(
+                        "domain=C,max_n={CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN},implementation={},counter={},marker_parity=odd",
+                        criterion.label(),
+                        counter.label()
+                    ),
+                    monotone_tally.get(counter),
+                ),
+                r1_generator_reading,
+            ]
+        }
+        FieldsBeyondTheProjectionGenerator::SameForEveryVersion => {
+            unreachable!(
+                "J7 的对照只在 r2 档跑（Criterion::is_judged_under），r2 档的生成器随版本变"
+            )
+        }
     }
 }
 
-/// 阳性对照那段循环：遍历判据表的每一行（只跑选中的那一族）。
-fn positive_control_readings(families: &[Family], thread_count: usize) -> Vec<ControlReading> {
+/// 阳性对照那段循环：遍历判据表的每一行（只跑选中的那一族、这一档判的判据）。
+fn positive_control_readings(
+    families: &[Family],
+    run_profile: RunProfile,
+    thread_count: usize,
+) -> Vec<ControlReading> {
     let mut readings = Vec::new();
     for criterion in Criterion::ALL {
-        if !families.contains(&criterion.family()) {
+        if !families.contains(&criterion.family()) && !criterion.is_judged_in_every_family() {
+            continue;
+        }
+        if !criterion.is_judged_under(run_profile) {
             continue;
         }
-        readings.extend(control_readings_of(*criterion, thread_count));
+        readings.extend(control_readings_of(*criterion, run_profile, thread_count));
     }
     readings
 }
@@ -4228,12 +6678,56 @@
     tally.witness(counter).unwrap_or("none").to_string()
 }
 
+/// 落后类别的五行（登记 r2 SQ9、SQ17）：相等、落后（两个方向之和）、互不包含、0 号盘落后、1 号盘落后；每行列出
+/// [`LAGGING_CLASS_COLUMNS`] 各量。`row_name` 是 `lagging_class`（每个 (域, N)）或 `verdict`（域的合计）。
+fn lagging_class_rows(row_name: &str, place: &str, tally: &Tally) -> Vec<String> {
+    let groups: [(&str, &[LaggingClass]); 5] = [
+        (LaggingClass::Equal.label(), &[LaggingClass::Equal]),
+        (
+            "lagging",
+            &[
+                LaggingClass::FirstDeviceBehind,
+                LaggingClass::SecondDeviceBehind,
+            ],
+        ),
+        (LaggingClass::Crossed.label(), &[LaggingClass::Crossed]),
+        (
+            LaggingClass::FirstDeviceBehind.label(),
+            &[LaggingClass::FirstDeviceBehind],
+        ),
+        (
+            LaggingClass::SecondDeviceBehind.label(),
+            &[LaggingClass::SecondDeviceBehind],
+        ),
+    ];
+    groups
+        .iter()
+        .map(|(group_label, classes)| {
+            let columns: Vec<String> = LAGGING_CLASS_COLUMNS
+                .iter()
+                .map(|counter| {
+                    let count: u64 = classes
+                        .iter()
+                        .map(|class| tally.lagging_class_count(*class, *counter))
+                        .sum();
+                    format!("{}={count}", counter.label())
+                })
+                .collect();
+            format!(
+                "name={row_name} {place} lagging_class={group_label} {}",
+                columns.join(" ")
+            )
+        })
+        .collect()
+}
+
 /// 一个 (域, N) 的全部行：状态数与见证、每条判据一行违例数、S 自己的对照、C 域的状态对、对抗域的 A-互。
 fn emit_the_rows_of_a_domain_size(
     emitter: &mut Emitter,
     domain: Domain,
     size: usize,
     tally: &Tally,
+    run_profile: RunProfile,
 ) {
     let place = format!(
         "domain={} ring_slots={} n={size}",
@@ -4266,7 +6760,7 @@
     };
     let states = tally.get(Counter::States);
     match domain.family() {
-        Family::LegalHistory => {
+        Family::LegalHistory | Family::MultiDisk => {
             violations_row("J0", Counter::Panicked, states);
             violations_row("J1", Counter::J1Violations, states);
             violations_row("J2", Counter::J2Violations, states);
@@ -4308,6 +6802,27 @@
         Counter::J6PremiseFailsViolations,
         tally.get(Counter::J6PremiseFailsJudged),
     );
+    if run_profile.judges_j6qn_and_j7() {
+        violations_row(
+            "J6Qn_premise_holds",
+            Counter::J6QnPremiseHoldsViolations,
+            tally.get(Counter::J6QnPremiseHoldsJudged),
+        );
+        violations_row(
+            "J6Qn_premise_fails",
+            Counter::J6QnPremiseFailsViolations,
+            tally.get(Counter::J6QnPremiseFailsJudged),
+        );
+        for field in FieldOfTheRebuiltRoot::ALL {
+            violations_row(
+                field.violations_counter().label(),
+                field.violations_counter(),
+                tally.get(Counter::J7Judged),
+            );
+        }
+        violations_row("J7b", Counter::J7bViolations, tally.get(Counter::J7bJudged));
+        violations_row("J7c", Counter::J7cViolations, tally.get(Counter::J7cJudged));
+    }
     if tally.get(Counter::Ia1Judged) > 0 {
         violations_row("IA1", Counter::Ia1Violations, tally.get(Counter::Ia1Judged));
     }
@@ -4320,6 +6835,12 @@
         tally.get(Counter::J6ThirdCallEqualsSecond),
         tally.get(Counter::J6ThirdCallDiffersOrIsMissing),
     ));
+    if run_profile.judges_j6qn_and_j7() {
+        emitter.result(&format!(
+            "name=first_returned_root_txg {place} first_returned_root_txg_is_the_largest={}",
+            tally.get(Counter::FirstReturnedRootTxgIsTheLargest),
+        ));
+    }
     if domain.specification_self_check() == SpecificationSelfCheck::Judged {
         for counter in [
             Counter::SpecificationWholeTableMismatch,
@@ -4353,6 +6874,48 @@
             ));
         }
     }
+    if !pinned_mirrored_counts(domain).is_empty() {
+        for row in lagging_class_rows("lagging_class", &place, tally) {
+            emitter.result(&row);
+        }
+    }
+    if !pinned_mirrored_monotone_counts(domain).is_empty() {
+        emitter.result(&format!(
+            "name=pair_states {place} pair_states_without_an_interval={}",
+            tally.get(Counter::PairStatesWithoutAnInterval),
+        ));
+        emitter.result(&format!(
+            "name=pairs {place} set={} pairs={} today_j5a={} today_j5b={} ideal_j5a={} ideal_j5b={}",
+            PairSet::Wide.label(),
+            tally.pair(PairSet::Wide, PairColumn::Pairs),
+            tally.pair(PairSet::Wide, PairColumn::TodayAppliedSetViolations),
+            tally.pair(PairSet::Wide, PairColumn::TodayLandedVersionViolations),
+            tally.pair(PairSet::Wide, PairColumn::IdealAppliedSetViolations),
+            tally.pair(PairSet::Wide, PairColumn::IdealLandedVersionViolations),
+        ));
+    }
+    if run_profile.compares_the_two_record_tables() {
+        let reported_order_fields = match run_profile.reported_disk_order {
+            ReportedDiskOrder::FirstDeviceBeforeSecond => String::new(),
+            ReportedDiskOrder::SecondDeviceBeforeFirst => format!(
+                " tables_differ_in_reported_order={} rows_differ_in_reported_order={} witness_in_reported_order={} rows_witness_in_reported_order={}",
+                tally.get(Counter::TablesDifferInReportedOrder),
+                tally.get(Counter::RowsDifferInReportedOrder),
+                witness_field(tally, Counter::TablesDifferInReportedOrder),
+                witness_field(tally, Counter::RowsDifferInReportedOrder),
+            ),
+        };
+        emitter.result(&format!(
+            "name=table_comparison {place} states={} tables_differ={} rows_differ={} direct_table_holds_a_record={} fully_landed_scan_table_size_checked={} witness={} rows_witness={}{reported_order_fields}",
+            tally.get(Counter::States),
+            tally.get(Counter::TablesDiffer),
+            tally.get(Counter::RowsDiffer),
+            tally.get(Counter::DirectTableHoldsARecord),
+            tally.get(Counter::FullyLandedScanTableSizeChecked),
+            witness_field(tally, Counter::TablesDiffer),
+            witness_field(tally, Counter::RowsDiffer),
+        ));
+    }
     if domain.family() == Family::Adversarial {
         emitter.result(&format!(
             "name=specification {place} expects_failure={}",
@@ -4380,6 +6943,7 @@
     domain: Domain,
     largest_size: usize,
     total: &Tally,
+    run_profile: RunProfile,
 ) {
     let place = format!(
         "domain={} ring_slots={} smallest_n={} largest_n={largest_size}",
@@ -4406,7 +6970,15 @@
         ));
     }
     match domain.family() {
-        Family::LegalHistory => {
+        Family::LegalHistory | Family::MultiDisk => {
+            // 登记 r2 第七节甲类、SF9：M、M6、MC 里今天的实现第一次调用报错的态数是 0；不符停下，两边都查。
+            if domain.family() == Family::MultiDisk && total.get(Counter::Failed) > 0 {
+                findings.halt_conditions.push(format!(
+                    "SF9 {} 域里今天的实现第一次调用报错的态数 {}（登记 r2 第七节甲类锚点是 0）",
+                    domain.label(),
+                    total.get(Counter::Failed)
+                ));
+            }
             // 甲类锚点（登记第七节）：L、L 小环、U 开着验证、X、E1 里原实现报错的态数是 0；不符走 F11，停下两边都查。
             let failed_states_are_pinned_to_zero =
                 domain.specification_self_check() == SpecificationSelfCheck::Judged;
@@ -4447,6 +7019,22 @@
                     ));
                 }
             }
+            if !pinned_mirrored_counts(domain).is_empty() {
+                for row in lagging_class_rows("verdict", &place, total) {
+                    emitter.result(&row);
+                }
+            }
+            if !pinned_mirrored_monotone_counts(domain).is_empty() {
+                emitter.result(&format!(
+                    "name=verdict {place} set={} pairs={} today_j5a_violations={} today_j5b_violations={} ideal_j5a_violations={} ideal_j5b_violations={}",
+                    PairSet::Wide.label(),
+                    total.pair(PairSet::Wide, PairColumn::Pairs),
+                    total.pair(PairSet::Wide, PairColumn::TodayAppliedSetViolations),
+                    total.pair(PairSet::Wide, PairColumn::TodayLandedVersionViolations),
+                    total.pair(PairSet::Wide, PairColumn::IdealAppliedSetViolations),
+                    total.pair(PairSet::Wide, PairColumn::IdealLandedVersionViolations),
+                ));
+            }
         }
         Family::Adversarial => {
             emitter.result(&format!(
@@ -4470,6 +7058,47 @@
             ));
         }
     }
+    if run_profile.judges_j6qn_and_j7() {
+        let first_returned_root_txg_is_the_largest =
+            total.get(Counter::FirstReturnedRootTxgIsTheLargest);
+        // 登记 r2 作废条款 SV4：E1 上 ρ_1 的 txg 是 u64 最大值的态数为 0，J6Qn 在 E1 上的 0 不算「Qn 管住了顶格」。
+        if domain == Domain::TxgOneBelowTheLargest && first_returned_root_txg_is_the_largest == 0 {
+            findings.void_conditions.push(
+                "SV4 E1 域上第一次返回的根 txg 是 u64 最大值的态数为 0，J6Qn 在 E1 上的 0 不算数"
+                    .to_string(),
+            );
+        }
+        emitter.result(&format!(
+            "name=verdict {place} criteria=r2 j6qn_premise_holds_violations={} j6qn_premise_holds_judged={} j6qn_premise_fails_violations={} j6qn_premise_fails_judged={} j7_judged={} j7a_instance_table_violations={} j7a_allocation_record_tree_root_violations={} j7a_filesystem_identifier_violations={} j7b_violations={} j7b_judged={} j7c_violations={} j7c_judged={} first_returned_root_txg_is_the_largest={first_returned_root_txg_is_the_largest}",
+            total.get(Counter::J6QnPremiseHoldsViolations),
+            total.get(Counter::J6QnPremiseHoldsJudged),
+            total.get(Counter::J6QnPremiseFailsViolations),
+            total.get(Counter::J6QnPremiseFailsJudged),
+            total.get(Counter::J7Judged),
+            total.get(Counter::J7aInstanceTableViolations),
+            total.get(Counter::J7aAllocationRecordTreeRootViolations),
+            total.get(Counter::J7aFilesystemIdentifierViolations),
+            total.get(Counter::J7bViolations),
+            total.get(Counter::J7bJudged),
+            total.get(Counter::J7cViolations),
+            total.get(Counter::J7cJudged),
+        ));
+    }
+    if run_profile.compares_the_two_record_tables() {
+        let reported_order_fields = match run_profile.reported_disk_order {
+            ReportedDiskOrder::FirstDeviceBeforeSecond => String::new(),
+            ReportedDiskOrder::SecondDeviceBeforeFirst => format!(
+                " tables_differ_in_reported_order={} rows_differ_in_reported_order={}",
+                total.get(Counter::TablesDifferInReportedOrder),
+                total.get(Counter::RowsDifferInReportedOrder),
+            ),
+        };
+        emitter.result(&format!(
+            "name=verdict {place} criteria=r2_record_tables tables_differ={} rows_differ={}{reported_order_fields}",
+            total.get(Counter::TablesDiffer),
+            total.get(Counter::RowsDiffer),
+        ));
+    }
 }
 
 // ── 常量回比（登记第三节、停机条款 H2）与装置自检
@@ -4582,7 +7211,7 @@
 
 /// S 在一张手写的表上、三种读法各自的期望（所选根取对抗域那一条，上限取大环的，表里的前一条一律当不在盘上）。
 fn expectations_on_a_hand_made_table(records: &RecordTable) -> [SpecificationExpectation; 3] {
-    let chosen_root = adversarial_chosen_root();
+    let chosen_root = adversarial_chosen_root(RunProfile::R1);
     let view = SpecificationView {
         chosen_root: &chosen_root,
         records,
@@ -4595,7 +7224,7 @@
 
 fn check_the_back_chain_of_every_generated_record() {
     for ring_domain in [Domain::LegalBigRing, Domain::LegalSmallRingSix] {
-        for unit in legal_work_units(ring_domain, 4) {
+        for unit in legal_work_units(ring_domain, 4, RunProfile::R1) {
             let history = generate_history(&unit.plan);
             for (record_index, slot) in history.slots.iter().enumerate() {
                 let expected_back_chain = match record_index.checked_sub(1) {
@@ -4614,7 +7243,7 @@
 }
 
 fn check_the_new_root_segment_of_every_generated_record() {
-    for unit in legal_work_units(Domain::LegalBigRing, 4) {
+    for unit in legal_work_units(Domain::LegalBigRing, 4, RunProfile::R1) {
         let history = generate_history(&unit.plan);
         for publish_index in 0..history.publishes.len() {
             for record_index in history.record_indexes_of_publish(publish_index) {
@@ -4640,10 +7269,19 @@
         "三位的子集掩码从空集起共 8 个"
     );
     for domain in Domain::ALL {
+        // 多盘域只在 r2 档有，它们的最小几档在 [`check_the_multi_disk_state_counts_of_the_smallest_sizes`] 里照 r2 档核。
+        if domain.family() == Family::MultiDisk {
+            continue;
+        }
         let largest_checked_size = (domain.smallest_size() + 1).min(2);
         for size in domain.smallest_size()..=largest_checked_size {
-            let tally =
-                tally_of_a_domain_size(*domain, size, Implementation::TodayReplayJournal, 1);
+            let tally = tally_of_a_domain_size(
+                *domain,
+                size,
+                Implementation::TodayReplayJournal,
+                RunProfile::R1,
+                1,
+            );
             assert_the_pinned_absolute_values(*domain, size, &tally);
         }
     }
@@ -4666,8 +7304,9 @@
         [true, false, false],
         "带标志一条、txg 越过一条、同 txg 一条：全表看 / 连着看·缺号不停 / 连着看·缺号即停"
     );
-    let nothing_applied =
-        SpecificationExpectation::ReturnsRoot(VersionFields::of_root(&adversarial_chosen_root()));
+    let nothing_applied = SpecificationExpectation::ReturnsRoot(VersionFields::of_root(
+        &adversarial_chosen_root(RunProfile::R1),
+    ));
     let lands_on_counter_nine = SpecificationExpectation::ReturnsRoot(VersionFields::of_record(
         &hand_made_record(9, 102, 1, true),
     ));
@@ -4717,6 +7356,7 @@
         ORDINARY_FIRST_TXG,
         vec![one_record_publish(), one_record_publish()],
         Vec::new(),
+        RunProfile::R1,
     );
     let history = generate_history(&plan);
     let presence = [
@@ -4726,6 +7366,7 @@
     let state = LegalState {
         history: &history,
         presence: &presence,
+        mirroring: JournalMirroring::FirstDeviceOnly,
         residue: None,
         chosen_version: 0,
         verify_named_units: true,
@@ -4757,7 +7398,7 @@
 }
 
 fn check_the_in_flight_limit_control_on_both_rings() {
-    let readings = control_readings_of(Criterion::J3WithinTheInFlightLimit, 1);
+    let readings = control_readings_of(Criterion::J3WithinTheInFlightLimit, RunProfile::R1, 1);
     assert_eq!(readings.len(), 2, "J3 的对照两个读数：小环、大环");
     assert!(
         readings[0].value > 0 && readings[0].is_red_as_required,
@@ -4780,12 +7421,14 @@
         ORDINARY_FIRST_TXG,
         vec![shape(1), shape(2), shape(1)],
         Vec::new(),
+        RunProfile::R1,
     );
     let history = generate_history(&plan);
     let presence = [RecordPresence::OnDisk(UnitState::BothCopiesGood); 4];
     let state = LegalState {
         history: &history,
         presence: &presence,
+        mirroring: JournalMirroring::FirstDeviceOnly,
         residue: None,
         chosen_version: 0,
         verify_named_units: true,
@@ -4802,7 +7445,13 @@
 }
 
 fn check_the_fixed_point_control() {
-    let tally = control_tally(Domain::LegalBigRing, 3, ControlKind::SecondCallMovesOn, 1);
+    let tally = control_tally(
+        Domain::LegalBigRing,
+        3,
+        ControlKind::SecondCallMovesOn,
+        RunProfile::R1,
+        1,
+    );
     assert!(
         tally.get(Counter::J6PremiseHoldsViolations) > 0,
         "第二次调用换成「再多落一版」的假结局，J6 违例数大于 0"
@@ -4814,6 +7463,7 @@
         Domain::MonotoneBigRing,
         1,
         Implementation::TodayReplayJournal,
+        RunProfile::R1,
         1,
     );
     assert_eq!(
@@ -4832,6 +7482,7 @@
         Domain::AdversarialBigRing,
         1,
         ControlKind::RootFromARecordWithoutTheLastRecordFlag,
+        RunProfile::R1,
         1,
     );
     assert!(
@@ -4842,7 +7493,7 @@
 
 fn check_a_residue_does_not_complete_a_publish() {
     // R 域 N = 2：P_1 那一条 0 号盘没落，1 号盘上有残留 ⇒ 它进了表，但 P_1 不算完整在盘上，j* 是 0。
-    let units = legal_work_units(Domain::Residue, 2);
+    let units = legal_work_units(Domain::Residue, 2, RunProfile::R1);
     assert_eq!(units.len(), 1, "R 域 N = 2 只有一段历史、残留只有一个位置");
     let history = generate_history(&units[0].plan);
     let residue = generate_residue(&history, 1);
@@ -4853,6 +7504,7 @@
     let state = LegalState {
         history: &history,
         presence: &presence,
+        mirroring: JournalMirroring::FirstDeviceOnly,
         residue: Some(&residue),
         chosen_version: 0,
         verify_named_units: true,
@@ -4940,6 +7592,630 @@
     ("structure_control", check_the_structure_control),
 ];
 
+/// r2 档生成器的四个字段（登记 r2 第一节、第八节）：两种奇偶各取第 0–3 版逐字段比；fsid 低 8 字节不随版本变、本地取法与
+/// `unit_filesystem_identifier` 回比（登记 r2 第三节）；由记录重建的根取 S1 的形态；r1 档生成器上「重建」与时间线根逐字段相同。
+fn check_the_fields_beyond_the_projection_of_the_r2_generator() {
+    for version in 0..4usize {
+        assert_eq!(
+            RunProfile::R1.fields_beyond_the_projection_of_version(version),
+            FieldsBeyondTheProjection {
+                filesystem_identifier: ROOT_FILESYSTEM_IDENTIFIER,
+                unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
+                instance_table: node_pointer_at_slot(69_000, 1),
+                allocation_record_tree_root: NodePointer::empty_root(),
+            },
+            "r1 档：第 {version} 版投影之外的四个字段与 r1 时的装置相同"
+        );
+    }
+    for (marker_parity, versions_written_by_the_unmount_sequence) in [
+        (
+            MarkerParity::EvenVersionsWrittenByTheUnmountSequence,
+            [0usize, 2],
+        ),
+        (MarkerParity::OddVersionsWrittenByTheUnmountSequence, [1, 3]),
+    ] {
+        for version in 0..4usize {
+            let fields =
+                RunProfile::r2(marker_parity).fields_beyond_the_projection_of_version(version);
+            let version_number = u64::try_from(version).expect("版本数很小");
+            let mut expected_filesystem_identifier = ROOT_FILESYSTEM_IDENTIFIER;
+            expected_filesystem_identifier[15] = u8::try_from(version).expect("版本数很小");
+            assert_eq!(
+                (
+                    fields.instance_table,
+                    fields.allocation_record_tree_root,
+                    fields.filesystem_identifier
+                ),
+                (
+                    node_pointer_at_slot(69_000 + version_number, 1),
+                    node_pointer_at_slot(68_000 + version_number, 1),
+                    expected_filesystem_identifier
+                ),
+                "r2 档（{}）第 {version} 版：实例表槽 69 000 + j、分配记录树根槽 68 000 + j、fsid 末字节 j",
+                marker_parity.label()
+            );
+            assert_eq!(
+                fields.unmount_marker,
+                if versions_written_by_the_unmount_sequence.contains(&version) {
+                    UnmountMarker::WrittenByTheUnmountSequence
+                } else {
+                    UnmountMarker::NotWrittenByTheUnmountSequence
+                },
+                "r2 档（{}）第 {version} 版的卸载记号",
+                marker_parity.label()
+            );
+            assert_eq!(
+                local_filesystem_identifier_low_eight_bytes(&fields.filesystem_identifier),
+                FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES,
+                "fsid 低 8 字节不随版本变，等于记录头那 8 字节"
+            );
+            assert_eq!(
+                local_filesystem_identifier_low_eight_bytes(&fields.filesystem_identifier),
+                singlefs_core::unit::unit_filesystem_identifier(&fields.filesystem_identifier),
+                "登记 r2 第三节：fsid 低 8 字节的本地取法与 unit_filesystem_identifier 不等"
+            );
+        }
+    }
+    let one_record_publish = || PublishShape {
+        record_count: 1,
+        commit_marker_bits: 0,
+    };
+    let history_under = |run_profile: RunProfile| {
+        generate_history(&history_plan(
+            Domain::LegalBigRing,
+            ORDINARY_FIRST_TXG,
+            vec![one_record_publish(), one_record_publish()],
+            Vec::new(),
+            run_profile,
+        ))
+    };
+    let r2_profile = RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
+    let r2_history = history_under(r2_profile);
+    let first_root = r2_history.timeline_root(0);
+    let second_root = r2_history.timeline_root(1);
+    assert_eq!(
+        VersionFields::of_root(&second_root),
+        r2_history.version_fields(1)
+    );
+    assert_eq!(
+        FieldsBeyondTheProjection::of_root(&second_root),
+        r2_profile.fields_beyond_the_projection_of_version(1),
+        "时间线根 R_1 的四个字段照 r2 档第 1 版"
+    );
+    assert_eq!(
+        adversarial_chosen_root(r2_profile),
+        first_root,
+        "对抗一族的所选根就是时间线 R_0"
+    );
+    let rebuilt = r2_history
+        .version_fields(1)
+        .to_root_rebuilt_over(&first_root);
+    assert_eq!(
+        (
+            VersionFields::of_root(&rebuilt),
+            rebuilt.instance_table,
+            rebuilt.allocation_record_tree_root,
+            rebuilt.filesystem_identifier,
+            rebuilt.unmount_marker
+        ),
+        (
+            r2_history.version_fields(1),
+            first_root.instance_table,
+            first_root.allocation_record_tree_root,
+            first_root.filesystem_identifier,
+            UnmountMarker::NotWrittenByTheUnmountSequence
+        ),
+        "由记录重建的根：投影取记录的，实例表指针、分配记录树根、fsid 抄所选根，卸载记号 NotWritten"
+    );
+    let r1_history = history_under(RunProfile::R1);
+    assert_eq!(
+        r1_history
+            .version_fields(1)
+            .to_root_rebuilt_over(&r1_history.timeline_root(0)),
+        r1_history.timeline_root(1),
+        "r1 档生成器上由记录重建的根与时间线根逐字段相同（r1 档输出不变的前提）"
+    );
+}
+
+/// `--profile` 与 `--marker-parity`（登记 r2 5.1、第八节）：不给 `--profile` 时 `replay.sh` 三份 r1 产物的那三条命令仍是 r1 档、
+/// 参数原样留下；两个参数写在哪都行；`--marker-parity` 只在 r2 档有。
+fn check_the_profile_options() {
+    let split = |words: &[&str]| {
+        split_off_the_profile_options(
+            &words
+                .iter()
+                .map(|word| (*word).to_string())
+                .collect::<Vec<String>>(),
+        )
+    };
+    let owned = |words: &[&str]| {
+        words
+            .iter()
+            .map(|word| (*word).to_string())
+            .collect::<Vec<String>>()
+    };
+    for r1_product_command in [
+        ["--bounds", "segment1", "--family", "legal"],
+        ["--bounds", "light", "--family", "legal"],
+        ["--bounds", "segment1", "--family", "adversarial"],
+    ] {
+        assert_eq!(
+            split(&r1_product_command),
+            Ok((RunProfile::R1, owned(&r1_product_command))),
+            "不给 --profile 时是 r1 档、参数原样留下：{r1_product_command:?}"
+        );
+    }
+    assert_eq!(
+        split(&["--profile", "r2", "--bounds", "segment1"]),
+        Ok((
+            RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
+            owned(&["--bounds", "segment1"])
+        ))
+    );
+    assert_eq!(
+        split(&[
+            "--bounds",
+            "segment1",
+            "--profile",
+            "r2",
+            "--marker-parity",
+            "odd"
+        ]),
+        Ok((
+            RunProfile::r2(MarkerParity::OddVersionsWrittenByTheUnmountSequence),
+            owned(&["--bounds", "segment1"])
+        ))
+    );
+    assert_eq!(
+        split(&["--profile", "r2", "--list-states", "L:2"]),
+        Ok((
+            RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
+            owned(&["--list-states", "L:2"])
+        )),
+        "逐态判定行同样认 --profile"
+    );
+    assert!(split(&["--marker-parity", "odd"]).is_err(), "r1 档不认奇偶");
+    assert!(split(&["--profile", "r3"]).is_err());
+    assert!(split(&["--profile"]).is_err());
+    assert!(split(&["--profile", "r2", "--profile", "r1"]).is_err());
+}
+
+/// J6Qn 三条对照的读数（登记 r2 第七节「5.5 的对照钉成等式」）：L 域 N ≤ 5 两种第二次报错各 3 110（违例与 judged），报
+/// `CHOSEN_ROOT_…` 那一条原 J6 同值；E1 全界第一次取理想恢复时原 J6 70、J6Qn 0。
+#[cfg(test)]
+fn check_the_j6qn_control_readings() {
+    let readings = control_readings_of(
+        Criterion::J6QnFixedPoint,
+        RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
+        1,
+    );
+    assert_eq!(
+        readings
+            .iter()
+            .map(|reading| reading.value)
+            .collect::<Vec<u64>>(),
+        vec![3_110, 3_110, 3_110, 3_110, 3_110, 70, 0],
+        "J6Qn 三条对照的读数：{readings:?}"
+    );
+    assert!(
+        readings.iter().all(|reading| reading.is_red_as_required),
+        "J6Qn 的对照都照登记红：{readings:?}"
+    );
+}
+
+/// J7 五行对照的读数（登记 r2 第七节）：r2 档 J7a 各自那一格 762、另两格 0，J7b 762，J7c 2 348；r1 档生成器上都是 0。
+#[cfg(test)]
+fn check_the_j7_control_readings() {
+    let run_profile = RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
+    for (criterion, expected_values) in [
+        (Criterion::J7aInstanceTable, vec![762, 0, 0, 0]),
+        (Criterion::J7aAllocationRecordTreeRoot, vec![0, 762, 0, 0]),
+        (Criterion::J7aFilesystemIdentifier, vec![0, 0, 762, 0]),
+        (Criterion::J7bUnmountMarkerOfARebuiltRoot, vec![762, 0]),
+        (
+            Criterion::J7cUnmountMarkerWhenNothingIsApplied,
+            vec![2_348, 0],
+        ),
+    ] {
+        let readings = control_readings_of(criterion, run_profile, 1);
+        assert_eq!(
+            readings
+                .iter()
+                .map(|reading| reading.value)
+                .collect::<Vec<u64>>(),
+            expected_values,
+            "{} 的对照读数：{readings:?}",
+            criterion.label()
+        );
+        assert!(
+            readings.iter().all(|reading| reading.is_red_as_required),
+            "{} 的对照都照登记红：{readings:?}",
+            criterion.label()
+        );
+    }
+}
+
+/// 第八节奇偶的判别力自证：`--marker-parity odd` 解出来的档上，J7b、J7c 两条对照在 L 域 N ≤ 5 都是 0（R_0 是 `NotWritten…`，
+/// 假结局与期望同值），在 C 域 Nc ≤ 3 都大于 0。
+#[cfg(test)]
+fn check_the_odd_marker_parity_turns_the_unmount_marker_controls() {
+    let arguments: Vec<String> = [
+        "--profile",
+        "r2",
+        "--marker-parity",
+        "odd",
+        "--bounds",
+        "segment1",
+    ]
+    .iter()
+    .map(|word| (*word).to_string())
+    .collect();
+    let (run_profile, _remaining_arguments) =
+        split_off_the_profile_options(&arguments).expect("这几个参数都认得");
+    for (control, counter) in [
+        (
+            ControlKind::IdealCopyingTheChosenUnmountMarkerWhenApplied,
+            Counter::J7bViolations,
+        ),
+        (
+            ControlKind::IdealWritingNotWrittenWhenNothingIsApplied,
+            Counter::J7cViolations,
+        ),
+    ] {
+        let big_ring = control_tally(
+            Domain::LegalBigRing,
+            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
+            control,
+            run_profile,
+            1,
+        );
+        let monotone = control_tally(
+            Domain::MonotoneBigRing,
+            CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN,
+            control,
+            run_profile,
+            1,
+        );
+        assert_eq!(
+            big_ring.get(counter),
+            0,
+            "反奇偶：L 域 {} 对照是 0",
+            counter.label()
+        );
+        assert!(
+            monotone.get(counter) > 0,
+            "反奇偶：C 域 {} 对照大于 0",
+            counter.label()
+        );
+    }
+}
+
+/// r2 档另跑的启动自检（r1 档不跑、不进产物，r1 档的输出因此与 r1 时的装置逐字节相同）。每一项都有一条同名的单测。
+///
+/// 启动自检只放不读实现结局的：第二段把装置交给被变异的实现跑，实现一变，读它结局的断言在启动时红、一个域都没跑
+/// （segment2b：登记行 406、414、448、518、1003、1004 五个变体在 [`check_the_j6qn_control_readings`] 上 panic）。
+/// 逐条判（登记 r2 修订 4）：
+/// - `fields_beyond_the_projection_of_the_r2_generator`：只读生成器、`VersionFields` 与 `unit_filesystem_identifier`，不调 `replay_journal`，留下。
+/// - `profile_options`：纯参数解析，留下。
+/// - `j6qn_control_readings`、`j7_control_readings`、`odd_marker_parity_turns_the_unmount_marker_controls`：三条都经 [`control_readings_of`] /
+///   [`control_tally`] 跑假结局，假结局里「第一次照真结局」「第二次照真调用」都调 `replay_journal`（J6Qn 的读数直接取决于它返回什么，
+///   J7 与奇偶两条的读数由理想恢复定、但同样调它）；挪成单测（`tests` 模块里同名的三条，断言一个字没动，三个检查函数只在 `cfg(test)` 下编）。产物里它们的读数
+///   仍由 [`positive_control_readings`] 逐行打成 `name=control`，不红记 V2。
+const R2_SELF_CHECKS: &[(&str, fn())] = &[
+    (
+        "fields_beyond_the_projection_of_the_r2_generator",
+        check_the_fields_beyond_the_projection_of_the_r2_generator,
+    ),
+    ("profile_options", check_the_profile_options),
+];
+
+/// 多盘三个域最小的两档（登记 r2 第七节乙类）：状态数、M 与 M6 的落后类别与见证、MC 的宽对与见证，照 r2 档逐格比钉的值。
+fn check_the_multi_disk_state_counts_of_the_smallest_sizes() {
+    for domain in [
+        Domain::MirroredBigRing,
+        Domain::MirroredSmallRingSix,
+        Domain::MirroredMonotoneBigRing,
+    ] {
+        for size in 1..=2 {
+            let tally = tally_of_a_domain_size(
+                domain,
+                size,
+                Implementation::TodayReplayJournal,
+                RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
+                1,
+            );
+            assert_the_pinned_absolute_values(domain, size, &tally);
+        }
+    }
+}
+
+/// 落后的方向（登记 r2 SQ17、9.5「落后方向两行对调」）：0 号盘什么都没落、1 号盘落了第一条 ⇒ 0 号盘落后；反过来 1 号盘落后。
+fn check_the_lagging_direction_on_hand_made_masks() {
+    assert_eq!(
+        lagging_class_of(0b0, 0b1),
+        LaggingClass::FirstDeviceBehind,
+        "0 号盘 {{}}、1 号盘 {{r_1}}：0 号盘的落盘集合真包含于 1 号盘的，是 0 号盘落后"
+    );
+    assert_eq!(
+        lagging_class_of(0b1, 0b0),
+        LaggingClass::SecondDeviceBehind,
+        "0 号盘 {{r_1}}、1 号盘 {{}}：1 号盘落后"
+    );
+    assert_eq!(lagging_class_of(0b11, 0b11), LaggingClass::Equal);
+    assert_eq!(lagging_class_of(0b01, 0b10), LaggingClass::Crossed);
+}
+
+/// 多盘四条判据的对照读数（登记 r2 5.5、第七节）：只数 0 号盘的 j\* 在 M、M6、L 上 100、996、0；原登记 J1、J4 两条对照在 M、M6、MC 上
+/// 228、2 052、45（J4 那条的 J1、J2、J3 是 0）；MC 宽对上 J5 那条对照两列都大于 0。
+fn check_the_multi_disk_control_readings() {
+    let run_profile = RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
+    for (criterion, expected_values) in [
+        (Criterion::MultiDiskAnyCopyIsOnDisk, vec![100, 996, 0]),
+        (
+            Criterion::MultiDiskReturnedRoot,
+            vec![228, 228, 2_052, 2_052, 45, 45],
+        ),
+        (
+            Criterion::MultiDiskFarthestVersion,
+            vec![228, 0, 0, 0, 2_052, 0, 0, 0, 45, 0, 0, 0],
+        ),
+    ] {
+        let readings = control_readings_of(criterion, run_profile, 1);
+        assert_eq!(
+            readings
+                .iter()
+                .map(|reading| reading.value)
+                .collect::<Vec<u64>>(),
+            expected_values,
+            "{} 的对照读数：{readings:?}",
+            criterion.label()
+        );
+    }
+    let readings = control_readings_of(
+        Criterion::MultiDiskAppliedSetAndLandedVersion,
+        run_profile,
+        1,
+    );
+    assert_eq!(readings.len(), 2, "J5 那条对照两个读数：J5a、J5b");
+    assert!(
+        readings
+            .iter()
+            .all(|reading| reading.value > 0 && reading.is_red_as_required),
+        "MC 宽对上 J5 那条对照两列都大于 0：{readings:?}"
+    );
+}
+
+/// 直接建表的盘次序（登记 5.1、r2 5.3、9.5「多盘直接建表先读 1 号盘」）：R 域 N = 2，P_1 那条在 0 号盘落了时表里取 0 号盘那一份、
+/// 残留不进表；0 号盘那一份没落时残留进表。M 域两条记录、第二条只有 1 号盘那一份：它进表，P_1 完整在盘上。
+fn check_the_direct_table_reads_the_first_device_before_the_second() {
+    let units = legal_work_units(Domain::Residue, 2, RunProfile::R1);
+    let history = generate_history(&units[0].plan);
+    let residue = generate_residue(&history, 1);
+    let second_record = history.slots[1]
+        .parsed
+        .clone()
+        .expect("合法历史的每条记录都解得出");
+    let residue_record = residue.slot.parsed.clone().expect("残留解得出");
+    let key = (second_record.instance, second_record.counter);
+    for (second_record_presence, expected_record) in [
+        (
+            RecordPresence::OnDisk(UnitState::BothCopiesGood),
+            &second_record,
+        ),
+        (RecordPresence::NotOnDisk, &residue_record),
+    ] {
+        let presence = [
+            RecordPresence::OnDisk(UnitState::BothCopiesGood),
+            second_record_presence,
+        ];
+        let state = LegalState {
+            history: &history,
+            presence: &presence,
+            mirroring: JournalMirroring::FirstDeviceOnly,
+            residue: Some(&residue),
+            chosen_version: 0,
+            verify_named_units: true,
+        };
+        let (records, _disk, _truths) = record_table_and_disk_of(&state);
+        assert_eq!(
+            records.get(&key),
+            Some(expected_record),
+            "残留只在 0 号盘那一份没落时进表（P_1 那条 0 号盘：{second_record_presence:?}）"
+        );
+    }
+    let one_record_publish = || PublishShape {
+        record_count: 1,
+        commit_marker_bits: 0,
+    };
+    let mirrored_history = generate_history(&history_plan(
+        Domain::MirroredBigRing,
+        ORDINARY_FIRST_TXG,
+        vec![one_record_publish(), one_record_publish()],
+        Vec::new(),
+        RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
+    ));
+    let presence = [
+        RecordPresence::OnDisk(UnitState::BothCopiesGood),
+        RecordPresence::NotOnDisk,
+    ];
+    let second_device_copy_is_on_disk = [false, true];
+    let state = LegalState {
+        history: &mirrored_history,
+        presence: &presence,
+        mirroring: JournalMirroring::BothDevices {
+            second_device_copy_is_on_disk: &second_device_copy_is_on_disk,
+        },
+        residue: None,
+        chosen_version: 0,
+        verify_named_units: true,
+    };
+    let (records, _disk, _truths) = record_table_and_disk_of(&state);
+    assert_eq!(records.len(), 2, "只有 1 号盘那一份的记录也进表");
+    assert_eq!(
+        farthest_version_of(&state, Some(mirrored_history.ring.local_in_flight_limit())),
+        1,
+        "多盘：记录任一份在即在，P_1 完整在盘上，j* = 1"
+    );
+}
+
+/// 多盘一族另跑的自检（只在 `--family multi_disk` 时跑、进那一份产物）。每一项都有一条同名的单测。
+const MULTI_DISK_SELF_CHECKS: &[(&str, fn())] = &[
+    (
+        "multi_disk_state_counts_of_the_smallest_sizes",
+        check_the_multi_disk_state_counts_of_the_smallest_sizes,
+    ),
+    (
+        "lagging_direction_on_hand_made_masks",
+        check_the_lagging_direction_on_hand_made_masks,
+    ),
+    (
+        "multi_disk_control_readings",
+        check_the_multi_disk_control_readings,
+    ),
+    (
+        "direct_table_reads_the_first_device_before_the_second",
+        check_the_direct_table_reads_the_first_device_before_the_second,
+    ),
+];
+
+/// 扫盘建表喂的系统配置与生成器对得上（登记 r2 第三节回比表的 fsid 一行、第七节甲类）：fsid 低 8 字节的本地取法等于记录头里那 8 字节、
+/// 等于 `unit_filesystem_identifier`，journal 环长等于这个环几何的字节数。不调 `scan_journal`（它是被测对象），可以放进启动自检。
+fn check_the_scan_system_configuration_matches_the_generator() {
+    for ring in [
+        RingGeometry::BIG,
+        RingGeometry::SMALL_SIX,
+        RingGeometry::SMALL_NINE,
+    ] {
+        let configuration = system_configuration_for_the_scan(ring);
+        let filesystem_identifier = configuration.immutable.filesystem_identifier;
+        assert_eq!(
+            local_filesystem_identifier_low_eight_bytes(&filesystem_identifier),
+            FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES,
+            "扫盘建表的系统配置 fsid 低 8 字节要等于记录头里那 8 字节（环 {} 槽）",
+            ring.ring_slots
+        );
+        assert_eq!(
+            local_filesystem_identifier_low_eight_bytes(&filesystem_identifier),
+            singlefs_core::unit::unit_filesystem_identifier(&filesystem_identifier),
+            "登记 r2 第三节：fsid 低 8 字节的本地取法与 unit_filesystem_identifier 不等（环 {} 槽）",
+            ring.ring_slots
+        );
+        assert_eq!(
+            configuration.immutable.sizes.journal_ring_bytes,
+            ring.ring_bytes(),
+            "扫盘建表的系统配置里 journal 环长要等于这个环几何的字节数（环 {} 槽）",
+            ring.ring_slots
+        );
+    }
+}
+
+/// `--table` 与 `--disk-order`（登记 r2 5.4、第八节）：不给取 `direct` 与 `normal`；写在哪都行；r1 档不认。
+fn check_the_table_and_disk_order_options() {
+    let owned = |words: &[&str]| {
+        words
+            .iter()
+            .map(|word| (*word).to_string())
+            .collect::<Vec<String>>()
+    };
+    let split = |words: &[&str]| split_off_the_profile_options(&owned(words));
+    let even_r2 = RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
+    assert_eq!(
+        split(&["--profile", "r2", "--bounds", "segment1"]),
+        Ok((even_r2, owned(&["--bounds", "segment1"]))),
+        "不给 --table 与 --disk-order 时取 direct 与 normal"
+    );
+    assert_eq!(
+        split(&[
+            "--bounds",
+            "segment1",
+            "--table",
+            "both",
+            "--profile",
+            "r2",
+            "--disk-order",
+            "reversed"
+        ]),
+        Ok((
+            even_r2
+                .with_table_source(RecordTableSource::Both)
+                .with_reported_disk_order(ReportedDiskOrder::SecondDeviceBeforeFirst),
+            owned(&["--bounds", "segment1"])
+        ))
+    );
+    assert_eq!(
+        split(&["--profile", "r2", "--table", "scan"]),
+        Ok((
+            even_r2.with_table_source(RecordTableSource::Scanned),
+            Vec::new()
+        ))
+    );
+    assert_eq!(
+        split(&[
+            "--profile",
+            "r2",
+            "--table",
+            "direct",
+            "--disk-order",
+            "normal"
+        ]),
+        Ok((even_r2, Vec::new()))
+    );
+    assert!(split(&["--table", "both"]).is_err(), "r1 档不认 --table");
+    assert!(
+        split(&["--disk-order", "reversed"]).is_err(),
+        "r1 档不认 --disk-order"
+    );
+    assert!(split(&["--profile", "r2", "--table", "scanned"]).is_err());
+    assert!(split(&["--profile", "r2", "--disk-order", "backwards"]).is_err());
+    assert!(split(&["--profile", "r2", "--table"]).is_err());
+    assert!(
+        split(&["--profile", "r2", "--table", "scan", "--table", "both"]).is_err(),
+        "同一个参数给了不止一次"
+    );
+}
+
+/// 盘次序（登记 r2 第八节「判别力自证」前一半）：`--disk-order reversed` 的内存盘 `device_identities()` 报 1 号盘在前，默认的报 0 号盘在前。
+fn check_a_reversed_memory_disk_reports_the_second_device_first() {
+    let device_numbers_reported_by = |reported_disk_order: ReportedDiskOrder| -> Vec<u32> {
+        MemoryDisk {
+            ring: RingGeometry::BIG,
+            reported_disk_order,
+            contents: Vec::new(),
+        }
+        .device_identities()
+        .iter()
+        .map(|device| device.0)
+        .collect()
+    };
+    assert_eq!(
+        device_numbers_reported_by(ReportedDiskOrder::SecondDeviceBeforeFirst),
+        vec![1, 0],
+        "--disk-order reversed：device_identities() 要报 1 号盘在前"
+    );
+    assert_eq!(
+        device_numbers_reported_by(ReportedDiskOrder::FirstDeviceBeforeSecond),
+        vec![0, 1],
+        "默认的盘次序：0 号盘在前"
+    );
+}
+
+/// 记录表怎么建、盘次序有一个不是默认值时另跑的启动自检（只在 r2 档能设、进那一份产物）。每一项都有一条同名的单测。
+/// 都不调 `scan_journal` 与 `replay_journal`：它们是被测对象，读它们结局的断言在单测里。
+const RECORD_TABLE_SELF_CHECKS: &[(&str, fn())] = &[
+    (
+        "scan_system_configuration_matches_the_generator",
+        check_the_scan_system_configuration_matches_the_generator,
+    ),
+    (
+        "table_and_disk_order_options",
+        check_the_table_and_disk_order_options,
+    ),
+    (
+        "reversed_memory_disk_reports_the_second_device_first",
+        check_a_reversed_memory_disk_reports_the_second_device_first,
+    ),
+];
+
 // ── 命令行与主流程
 
 struct RunSelection {
@@ -4950,7 +8226,7 @@
     family_label: &'static str,
 }
 
-const USAGE: &str = "用法：e165_replay_bounded_enumeration --bounds segment1|segment2|light [--family legal|adversarial|all] [--only <域>[,<域>…]] [--max-n <N>]；域名：L L6 L9 U C C6 A A6 AC X E1 E2 R。单状态重放：--replay-state <域>:<N>:<序号>；逐态判定行：--list-states <域>:<N>（这两种的域名用产物里 domain= 的写法，U 域是 U_verify_on、U_verify_off）；钉住的红例逐行重放：--pinned-cases <表>（格式见 .claude/gate.d/e165-pinned-red-examples.tsv 文件头）";
+const USAGE: &str = "用法：e165_replay_bounded_enumeration --bounds segment1|segment2|light [--family legal|adversarial|all|multi_disk] [--only <域>[,<域>…]] [--max-n <N>] [--profile r1|r2] [--marker-parity even|odd] [--table direct|scan|both] [--disk-order normal|reversed]（档不给取 r1；奇偶、建表法、盘次序只在 r2 档有，不给分别取 even、direct、normal；这几个参数写在哪都行，单状态重放、逐态判定行、钉住的红例逐行重放同样认）；域名：L L6 L9 U C C6 A A6 AC X E1 E2 R M M6 MC（多盘三个域只在 --family multi_disk 时跑、要 --profile r2；all 只含合法历史与对抗两族）。单状态重放：--replay-state <域>:<N>:<序号>；逐态判定行：--list-states <域>:<N>（这两种的域名用产物里 domain= 的写法，U 域是 U_verify_on、U_verify_off）；钉住的红例逐行重放：--pinned-cases <表>（格式见 .claude/gate.d/e165-pinned-red-examples.tsv 文件头）";
 
 fn parse_the_command_line(arguments: &[String]) -> Result<RunSelection, String> {
     let mut bounds: Option<BoundsTier> = None;
@@ -4983,6 +8259,10 @@
                     families = vec![Family::Adversarial];
                     family_label = "adversarial";
                 }
+                "multi_disk" => {
+                    families = vec![Family::MultiDisk];
+                    family_label = "multi_disk";
+                }
                 "all" => {}
                 unknown_family => return Err(format!("不认识的族：{unknown_family}")),
             },
@@ -5018,6 +8298,130 @@
     })
 }
 
+/// 从参数里取出 `--profile` 与 `--marker-parity`（写在哪都行），交回档与其余参数（登记 r2 5.1、第八节）。
+/// 档不给取 [`DEFAULT_PROFILE_LABEL`]；`--marker-parity` 只在 r2 档有。
+fn split_off_the_profile_options(
+    arguments: &[String],
+) -> Result<(RunProfile, Vec<String>), String> {
+    let mut profile_label: Option<String> = None;
+    let mut marker_parity_label: Option<String> = None;
+    let mut table_source_label: Option<String> = None;
+    let mut disk_order_label: Option<String> = None;
+    let mut remaining_arguments = Vec::new();
+    let mut position = 0;
+    while position < arguments.len() {
+        let argument = &arguments[position];
+        let destination = match argument.as_str() {
+            "--profile" => Some(&mut profile_label),
+            "--marker-parity" => Some(&mut marker_parity_label),
+            "--table" => Some(&mut table_source_label),
+            "--disk-order" => Some(&mut disk_order_label),
+            _other_argument => None,
+        };
+        let Some(destination) = destination else {
+            remaining_arguments.push(argument.clone());
+            position += 1;
+            continue;
+        };
+        let value = arguments
+            .get(position + 1)
+            .ok_or_else(|| format!("{argument} 后面缺一个值"))?;
+        if destination.replace(value.clone()).is_some() {
+            return Err(format!("{argument} 给了不止一次"));
+        }
+        position += 2;
+    }
+    let marker_parity = match marker_parity_label.as_deref() {
+        None | Some("even") => MarkerParity::EvenVersionsWrittenByTheUnmountSequence,
+        Some("odd") => MarkerParity::OddVersionsWrittenByTheUnmountSequence,
+        Some(unknown_parity) => {
+            return Err(format!(
+                "不认识的卸载记号奇偶：{unknown_parity}（只认 even、odd）"
+            ))
+        }
+    };
+    let table_source = match table_source_label.as_deref() {
+        None | Some("direct") => RecordTableSource::Direct,
+        Some("scan") => RecordTableSource::Scanned,
+        Some("both") => RecordTableSource::Both,
+        Some(unknown_table_source) => {
+            return Err(format!(
+                "不认识的建表法：{unknown_table_source}（只认 direct、scan、both）"
+            ))
+        }
+    };
+    let reported_disk_order = match disk_order_label.as_deref() {
+        None | Some("normal") => ReportedDiskOrder::FirstDeviceBeforeSecond,
+        Some("reversed") => ReportedDiskOrder::SecondDeviceBeforeFirst,
+        Some(unknown_disk_order) => {
+            return Err(format!(
+                "不认识的盘次序：{unknown_disk_order}（只认 normal、reversed）"
+            ))
+        }
+    };
+    match profile_label.as_deref().unwrap_or(DEFAULT_PROFILE_LABEL) {
+        "r1" if marker_parity_label.is_some() => {
+            Err("--marker-parity 只在 --profile r2 时有".to_string())
+        }
+        "r1" if table_source_label.is_some() || disk_order_label.is_some() => {
+            Err("--table 与 --disk-order 只在 --profile r2 时有".to_string())
+        }
+        "r1" => Ok((RunProfile::R1, remaining_arguments)),
+        "r2" => Ok((
+            RunProfile::r2(marker_parity)
+                .with_table_source(table_source)
+                .with_reported_disk_order(reported_disk_order),
+            remaining_arguments,
+        )),
+        unknown_profile => Err(format!("不认识的档：{unknown_profile}（只认 r1、r2）")),
+    }
+}
+
+/// 配置行与收尾判决行里档的那几段：r1 档什么都不加（同 r1 时的装置逐字节）；r2 档的建表法与盘次序只在不是默认值时才写
+/// （默认的 r2 档输出因此不因第四段加了这两个旋钮而多一个字）。
+fn profile_fields_of_the_run_rows(run_profile: RunProfile) -> String {
+    let table_source_field = match run_profile.table_source {
+        RecordTableSource::Direct => String::new(),
+        RecordTableSource::Scanned | RecordTableSource::Both => {
+            format!(" table={}", run_profile.table_source.label())
+        }
+    };
+    let disk_order_field = match run_profile.reported_disk_order {
+        ReportedDiskOrder::FirstDeviceBeforeSecond => String::new(),
+        ReportedDiskOrder::SecondDeviceBeforeFirst => {
+            format!(" disk_order={}", run_profile.reported_disk_order.label())
+        }
+    };
+    format!(
+        "{}{table_source_field}{disk_order_field}",
+        generator_and_criteria_fields_of_the_run_rows(run_profile)
+    )
+}
+
+/// [`profile_fields_of_the_run_rows`] 里生成器与判据那两样的几段。
+fn generator_and_criteria_fields_of_the_run_rows(run_profile: RunProfile) -> String {
+    match (run_profile.generator, run_profile.criteria) {
+        (
+            FieldsBeyondTheProjectionGenerator::SameForEveryVersion,
+            CriteriaSet::RegisteredBeforeR2,
+        ) => String::new(),
+        (
+            FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
+            CriteriaSet::WithJ6QnAndJ7,
+        ) => format!(" profile=r2 marker_parity={}", marker_parity.label()),
+        (FieldsBeyondTheProjectionGenerator::SameForEveryVersion, CriteriaSet::WithJ6QnAndJ7) => {
+            " profile=r1_generator_with_r2_criteria".to_string()
+        }
+        (
+            FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
+            CriteriaSet::RegisteredBeforeR2,
+        ) => format!(
+            " profile=r2_generator_with_r1_criteria marker_parity={}",
+            marker_parity.label()
+        ),
+    }
+}
+
 /// 线程数：`E165_THREADS`，没设取 `SINGLEFS_THREAD_CAP`，都没设取 `available_parallelism`。
 fn thread_count_from_the_environment() -> usize {
     for variable in [
@@ -5036,16 +8440,31 @@
 }
 
 fn main() {
-    let arguments: Vec<String> = env::args().skip(1).collect();
+    let arguments_as_given: Vec<String> = env::args().skip(1).collect();
+    let (run_profile, arguments) = match split_off_the_profile_options(&arguments_as_given) {
+        Ok(split) => split,
+        Err(problem) => {
+            eprintln!("  ✗ {problem}");
+            eprintln!("  → 怎么办：{USAGE}");
+            std::process::exit(2);
+        }
+    };
     if let [option, table_path] = arguments.as_slice() {
         if option == "--pinned-cases" {
-            replay_every_pinned_red_example(table_path);
+            replay_every_pinned_red_example(table_path, run_profile);
             return;
         }
     }
     match parse_the_state_probe(&arguments) {
+        Ok(Some((domain, _size, _probe)))
+            if domain.family() == Family::MultiDisk && !run_profile.judges_j6qn_and_j7() =>
+        {
+            eprintln!("  ✗ 多盘域 {} 只在 --profile r2 时有", domain.label());
+            eprintln!("  → 怎么办：{USAGE}");
+            std::process::exit(2);
+        }
         Ok(Some((domain, size, probe))) => {
-            run_the_state_probe(domain, size, probe);
+            run_the_state_probe(domain, size, probe, run_profile);
             return;
         }
         Ok(None) => {}
@@ -5063,13 +8482,20 @@
             std::process::exit(2);
         }
     };
+    if selection.families.contains(&Family::MultiDisk) && !run_profile.judges_j6qn_and_j7() {
+        eprintln!(
+            "  ✗ --family multi_disk 只在 --profile r2 时有（登记 r2 5.1：多盘域只在 r2 档）"
+        );
+        eprintln!("  → 怎么办：{USAGE}");
+        std::process::exit(2);
+    }
     let thread_count = thread_count_from_the_environment();
     install_the_panic_hook_that_stays_quiet_inside_the_replay_call();
     let run_started = Instant::now();
     let mut emitter = Emitter { emitted: 0 };
     let mut findings = RunFindings::default();
     emitter.result(&format!(
-        "name=config experiment=E165 bounds={} family={} only={} max_n={} chosen_instance={CHOSEN_INSTANCE} first_txg={ORDINARY_FIRST_TXG} big_ring_slots={BIG_RING_SLOTS} big_ring_first_counter={BIG_RING_FIRST_HISTORY_COUNTER} small_ring_slots={SMALL_RING_SIX_SLOTS},{SMALL_RING_NINE_SLOTS}",
+        "name=config experiment=E165 bounds={} family={} only={} max_n={} chosen_instance={CHOSEN_INSTANCE} first_txg={ORDINARY_FIRST_TXG} big_ring_slots={BIG_RING_SLOTS} big_ring_first_counter={BIG_RING_FIRST_HISTORY_COUNTER} small_ring_slots={SMALL_RING_SIX_SLOTS},{SMALL_RING_NINE_SLOTS}{}",
         selection.bounds.label(),
         selection.family_label,
         selection
@@ -5079,10 +8505,37 @@
         selection
             .largest_size_cap
             .map_or_else(|| "none".to_string(), |cap| cap.to_string()),
+        profile_fields_of_the_run_rows(run_profile),
     ));
 
-    // ① 常量回比断言与装置自检（断言不过当场 panic，进程以非 0 退出，一行结果都不算）。
-    for (check_name, check) in SELF_CHECKS {
+    // ① 常量回比断言与装置自检（断言不过当场 panic，进程以非 0 退出，一行结果都不算）；r2 档另跑 r2 那几项。
+    let self_checks_of_the_profile: &[(&str, fn())] = if run_profile.judges_j6qn_and_j7() {
+        R2_SELF_CHECKS
+    } else {
+        &[]
+    };
+    // 多盘一族另跑多盘那几项（只在 --family multi_disk 时）。
+    let self_checks_of_the_multi_disk_family: &[(&str, fn())] =
+        if selection.families.contains(&Family::MultiDisk) {
+            MULTI_DISK_SELF_CHECKS
+        } else {
+            &[]
+        };
+    // 记录表怎么建、盘次序有一个不是默认值时另跑这几项（只在 r2 档能设）。
+    let self_checks_of_the_record_table_knobs: &[(&str, fn())] = if run_profile.table_source
+        != RecordTableSource::Direct
+        || run_profile.reported_disk_order != ReportedDiskOrder::FirstDeviceBeforeSecond
+    {
+        RECORD_TABLE_SELF_CHECKS
+    } else {
+        &[]
+    };
+    for (check_name, check) in SELF_CHECKS
+        .iter()
+        .chain(self_checks_of_the_profile)
+        .chain(self_checks_of_the_multi_disk_family)
+        .chain(self_checks_of_the_record_table_knobs)
+    {
         check();
         emitter.result(&format!("name=self_check check={check_name} passed=true"));
     }
@@ -5101,7 +8554,8 @@
 
     // ② 阳性对照：遍历判据表的每一行；有一行没红，这一段作废（V2）。
     let controls_started = Instant::now();
-    let control_readings = positive_control_readings(&selection.families, thread_count);
+    let control_readings =
+        positive_control_readings(&selection.families, run_profile, thread_count);
     for reading in &control_readings {
         emitter.result(&format!(
             "name=control criterion={} measured={} value={} must_see={} red_as_required={}",
@@ -5157,10 +8611,32 @@
                 *domain,
                 size,
                 Implementation::TodayReplayJournal,
+                run_profile,
                 thread_count,
             );
             assert_the_pinned_absolute_values(*domain, size, &tally);
-            emit_the_rows_of_a_domain_size(&mut emitter, *domain, size, &tally);
+            // 登记 r2 第七节甲类、SF9：每条记录在每块盘上都落了的那一态扫盘表的条数断言要真跑到过（一个都没跑到，这条断言是空的）。
+            if run_profile.table_source != RecordTableSource::Direct
+                && domain.family() != Family::Adversarial
+                && tally.get(Counter::FullyLandedScanTableSizeChecked) == 0
+            {
+                findings.halt_conditions.push(format!(
+                    "SF9 {} 域 N = {size} 没有一个「每条记录在每块盘上都落了」的态被断言过扫盘表的条数",
+                    domain.label()
+                ));
+            }
+            // 登记 r2 作废条款 SV6：两种建表法的表相同、判出来的行却不同（同一张表同一套判据不该有两种结局）。
+            if run_profile.compares_the_two_record_tables()
+                && tally.get(Counter::TablesDiffer) == 0
+                && tally.get(Counter::RowsDiffer) > 0
+            {
+                findings.void_conditions.push(format!(
+                    "SV6 {} 域 N = {size} 两种建表法的表逐态相同、判出来的行却有 {} 个态不同",
+                    domain.label(),
+                    tally.get(Counter::RowsDiffer)
+                ));
+            }
+            emit_the_rows_of_a_domain_size(&mut emitter, *domain, size, &tally, run_profile);
             println!(
                 "E7TIMING name=elapsed scope=domain domain={} n={size} states={} elapsed_seconds={:.3} threads={thread_count}",
                 domain.label(),
@@ -5176,6 +8652,7 @@
             *domain,
             largest_size,
             &domain_total,
+            run_profile,
         );
     }
 
@@ -5192,10 +8669,13 @@
         ));
     }
     emitter.result(&format!(
-        "name=verdict scope=run bounds={} family={} states={total_states} self_checks_passed={} control_readings={} controls_not_red={} void_conditions={} halt_conditions={}",
+        "name=verdict scope=run bounds={} family={} states={total_states} self_checks_passed={} control_readings={} controls_not_red={} void_conditions={} halt_conditions={}{}",
         selection.bounds.label(),
         selection.family_label,
-        SELF_CHECKS.len(),
+        SELF_CHECKS.len()
+            + self_checks_of_the_profile.len()
+            + self_checks_of_the_multi_disk_family.len()
+            + self_checks_of_the_record_table_knobs.len(),
         control_readings.len(),
         control_readings
             .iter()
@@ -5203,6 +8683,7 @@
             .count(),
         findings.void_conditions.len(),
         findings.halt_conditions.len(),
+        profile_fields_of_the_run_rows(run_profile),
     ));
     println!(
         "E7TIMING name=elapsed scope=run states={total_states} elapsed_seconds={:.3} threads={thread_count}",
@@ -5285,6 +8766,217 @@
         check_parse_rejects_ordinal_zero_whose_checksums_hold();
     }
 
+    #[test]
+    fn the_r2_generator_varies_the_four_fields_beyond_the_projection_with_the_version() {
+        check_the_fields_beyond_the_projection_of_the_r2_generator();
+    }
+
+    #[test]
+    fn without_profile_the_three_r1_product_commands_stay_on_the_r1_profile_and_the_marker_parity_needs_r2(
+    ) {
+        check_the_profile_options();
+    }
+
+    #[test]
+    fn the_j6qn_controls_read_3110_on_the_big_ring_and_70_against_0_on_e1() {
+        check_the_j6qn_control_readings();
+    }
+
+    #[test]
+    fn the_j7_controls_read_762_and_2348_on_the_r2_generator_and_0_on_the_r1_generator() {
+        check_the_j7_control_readings();
+    }
+
+    #[test]
+    fn the_odd_marker_parity_turns_the_unmount_marker_controls_to_0_on_l_and_above_0_on_c() {
+        check_the_odd_marker_parity_turns_the_unmount_marker_controls();
+    }
+
+    #[test]
+    fn the_multi_disk_state_counts_lagging_classes_and_witnesses_of_the_smallest_sizes_equal_the_pinned_values(
+    ) {
+        check_the_multi_disk_state_counts_of_the_smallest_sizes();
+    }
+
+    #[test]
+    fn a_state_whose_first_device_holds_nothing_and_whose_second_holds_the_first_record_lags_on_the_first_device(
+    ) {
+        check_the_lagging_direction_on_hand_made_masks();
+    }
+
+    #[test]
+    fn the_multi_disk_controls_read_100_996_0_on_the_first_device_alone_and_228_2052_45_on_j1_and_j4(
+    ) {
+        check_the_multi_disk_control_readings();
+    }
+
+    #[test]
+    fn a_residue_enters_the_direct_table_only_when_the_first_device_copy_is_not_on_disk() {
+        check_the_direct_table_reads_the_first_device_before_the_second();
+    }
+
+    #[test]
+    fn the_scan_configuration_carries_the_generators_filesystem_identifier_and_ring_length() {
+        check_the_scan_system_configuration_matches_the_generator();
+    }
+
+    #[test]
+    fn the_table_and_disk_order_options_default_to_direct_and_normal_and_the_r1_profile_refuses_them(
+    ) {
+        check_the_table_and_disk_order_options();
+    }
+
+    #[test]
+    fn a_reversed_memory_disk_reports_the_second_device_before_the_first() {
+        check_a_reversed_memory_disk_reports_the_second_device_first();
+    }
+
+    /// 一张表键相同、其中一条记录的新根段不同：必须报「两张表不同」（登记 r2 9.5「T-表只比键集合、不比值」）。
+    #[test]
+    fn two_record_tables_with_the_same_keys_and_a_different_new_root_segment_are_reported_as_different(
+    ) {
+        let record = hand_made_record(7, 101, 1, true);
+        let mut other_record = record.clone();
+        other_record.new_tree_identifier_watermark += 1;
+        let direct_table = table_of(vec![record]);
+        let scanned_table = table_of(vec![other_record]);
+        assert!(
+            direct_table.keys().eq(scanned_table.keys()),
+            "两张表的键相同"
+        );
+        assert!(the_two_record_tables_differ(&direct_table, &scanned_table));
+        assert!(!the_two_record_tables_differ(
+            &direct_table,
+            &direct_table.clone()
+        ));
+    }
+
+    /// 两行的计数器标签集合相同、结局一边「报错」一边「返回根」：必须报「两行不同」（登记 r2 9.5「T-行只比计数器标签、不比实现结局」）。
+    #[test]
+    fn two_judged_rows_with_the_same_counter_labels_and_a_failure_against_a_returned_root_are_reported_as_different(
+    ) {
+        let returned = ReplayOutcome::Returned(ReturnedReplay {
+            root: adversarial_chosen_root(RunProfile::R1),
+            prefix_applied: 0,
+            verification_passed: 0,
+        });
+        let failed = ReplayOutcome::Failed(FailureKind::Other);
+        let summary = |first_outcome: ReplayOutcome, counter_labels: &str| JudgedRowSummary {
+            first_outcome,
+            counter_labels: counter_labels.to_string(),
+        };
+        assert!(the_two_judged_rows_differ(
+            &summary(returned, "states,J4"),
+            &summary(failed, "states,J4")
+        ));
+        assert!(the_two_judged_rows_differ(
+            &summary(returned, "states,J4"),
+            &summary(returned, "states")
+        ));
+        assert!(!the_two_judged_rows_differ(
+            &summary(returned, "states,J4"),
+            &summary(returned, "states,J4")
+        ));
+    }
+
+    /// 两种建表法在最小的几档上逐态相同（登记 r2 第七节甲类「全部落盘那一态的扫盘表条数等于生成的条数」由装置在每个这样的态上断言，
+    /// 断言没跑到的域这里也报红）：合法历史一族、多盘一族、对抗一族各取最小的两三档。
+    #[test]
+    fn the_two_table_sources_agree_on_the_smallest_sizes_and_the_scan_table_of_every_fully_landed_state_holds_every_record(
+    ) {
+        let run_profile = RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence)
+            .with_table_source(RecordTableSource::Both);
+        for (domain, sizes) in [
+            (Domain::LegalBigRing, 1..=3),
+            (Domain::LegalSmallRingSix, 1..=3),
+            (Domain::NamedUnitsVerified, 1..=2),
+            (Domain::MonotoneBigRing, 1..=2),
+            (Domain::CrossInstance, 2..=3),
+            (Domain::TxgOneBelowTheLargest, 1..=3),
+            (Domain::Residue, 2..=3),
+            (Domain::MirroredBigRing, 1..=2),
+            (Domain::MirroredSmallRingSix, 1..=2),
+            (Domain::MirroredMonotoneBigRing, 1..=2),
+            (Domain::AdversarialBigRing, 1..=1),
+        ] {
+            for size in sizes {
+                let tally = tally_of_a_domain_size(
+                    domain,
+                    size,
+                    Implementation::TodayReplayJournal,
+                    run_profile,
+                    1,
+                );
+                assert_eq!(
+                    (
+                        tally.get(Counter::TablesDiffer),
+                        tally.get(Counter::RowsDiffer)
+                    ),
+                    (0, 0),
+                    "{} 域 N = {size}：两种建表法逐态相同",
+                    domain.label()
+                );
+                if domain.family() != Family::Adversarial {
+                    assert!(
+                        tally.get(Counter::FullyLandedScanTableSizeChecked) > 0,
+                        "{} 域 N = {size}：全部落盘那一态的扫盘表条数断言要真跑到",
+                        domain.label()
+                    );
+                    assert!(tally.get(Counter::DirectTableHoldsARecord) > 0);
+                }
+            }
+        }
+    }
+
+    /// 盘次序反过来（登记 r2 第八节「判别力自证」后一半）：R 域 N = 2，残留在 1 号盘，扫盘先读到残留；「盘号升序」那一栏的直接表在
+    /// 0 号盘那一份落了的态上与扫盘表不同，必须大于 0。「照盘报的次序」那一栏是推 s7 的预期（0），不是判别力自证，不在这里断言。
+    #[test]
+    fn with_the_disk_order_reversed_the_scan_takes_the_residue_first_so_the_device_number_order_direct_table_differs(
+    ) {
+        let run_profile = RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence)
+            .with_table_source(RecordTableSource::Both)
+            .with_reported_disk_order(ReportedDiskOrder::SecondDeviceBeforeFirst);
+        let tally = tally_of_a_domain_size(
+            Domain::Residue,
+            2,
+            Implementation::TodayReplayJournal,
+            run_profile,
+            1,
+        );
+        assert!(
+            tally.get(Counter::TablesDiffer) > 0,
+            "R 域 N = 2：盘号升序一栏的直接表在残留所在那条记录 0 号盘那份落了的态上要与扫盘表不同"
+        );
+    }
+
+    /// T-表与 T-行两条对照的读数（登记 r2 5.5、第七节）：T-表 `tables_differ` = `direct_table_holds_a_record` = 2 989、`rows_differ` > 0；
+    /// T-行 `rows_differ` = 状态数 = 3 110、`states` = 3 110、`tables_differ` = 0。
+    #[test]
+    fn the_table_comparison_controls_read_2989_on_the_tables_and_3110_on_the_rows() {
+        let run_profile = RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence)
+            .with_table_source(RecordTableSource::Both);
+        for (criterion, expected_values) in [
+            (
+                Criterion::TableComparisonSeesADifferentTable,
+                [Some(2_989), Some(2_989), None],
+            ),
+            (
+                Criterion::TableComparisonSeesADifferentRow,
+                [Some(3_110), Some(3_110), Some(0)],
+            ),
+        ] {
+            let readings = control_readings_of(criterion, run_profile, 1);
+            assert_eq!(readings.len(), expected_values.len(), "{readings:?}");
+            for (reading, expected) in readings.iter().zip(expected_values) {
+                assert!(reading.is_red_as_required, "对照没红：{reading:?}");
+                match expected {
+                    Some(value) => assert_eq!(reading.value, value, "{reading:?}"),
+                    None => assert!(reading.value > 0, "{reading:?}"),
+                }
+            }
+        }
+    }
+
     /// 逐态判定行里「这个状态让哪几个量加了一」：照 `Counter::ALL` 的次序，加了不止一的带次数，一个都没加写 none。
     #[test]
     fn the_counters_a_state_added_are_listed_in_order_with_their_multiplicity() {
@@ -5333,12 +9025,21 @@
         assert!(parse_the_state_probe(&arguments("--replay-state", "X:1:0")).is_err());
     }
 
-    /// 判据表每一行都有一条对照且都红（登记 5.4、作废条款 V2）。对抗一族那几行要扫到 n = 3，用上全部核。
+    /// 判据表每一行都有一条对照且都红（登记 5.4、作废条款 V2；r2 档加的 J6Qn、J7 五行同样，登记 r2 5.5）。r1 档只判 r1 时的那几行、
+    /// 也都红；r2 档判全表。对抗一族那几行要扫到 n = 3，用上全部核。
     #[test]
     fn every_criterion_has_a_positive_control_and_every_control_is_red() {
         let thread_count = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
-        let readings =
-            positive_control_readings(&[Family::LegalHistory, Family::Adversarial], thread_count);
+        let families = [Family::LegalHistory, Family::Adversarial];
+        let r1_readings = positive_control_readings(&families, RunProfile::R1, thread_count);
+        for reading in &r1_readings {
+            assert!(reading.is_red_as_required, "r1 档对照没红：{reading:?}");
+        }
+        let readings = positive_control_readings(
+            &[Family::LegalHistory, Family::Adversarial, Family::MultiDisk],
+            RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
+            thread_count,
+        );
         for criterion in Criterion::ALL.iter() {
             assert!(
                 readings
@@ -5376,12 +9077,12 @@
         );
     }
 
-    /// 钉住的表一行：八段、②③④ 解得出才收；段数不对、域不认识、N 越界、序号不是数都拒。
+    /// 钉住的表一行：二十一段、抄来的账行与 ②③④⑦⑧ 相同、②③④ 解得出才收；段数不对、域不认识、N 越界、序号不是数都拒。
     #[test]
-    fn a_pinned_red_example_row_needs_eight_columns_a_known_domain_a_size_in_range_and_an_ordinal()
-    {
+    fn a_pinned_red_example_row_needs_twenty_one_columns_a_known_domain_a_size_in_range_and_an_ordinal(
+    ) {
         let row = |domain: &str, size: &str, ordinal: &str| {
-            format!("abc\t{domain}\t{size}\t{ordinal}\tledger.tsv:2\tcrates/mutations.tsv#name\tE7RESULT original\tE7RESULT variant")
+            format!("abc\t{domain}\t{size}\t{ordinal}\tledger.tsv:2\tcrates/mutations.tsv#name\tE7RESULT original\tE7RESULT variant\ttsv-1\t1\t{domain}\t{size}\t{ordinal}\tcounters\t+J1\ttrue\ttrue\ttrue\tcommand\tE7RESULT original\tE7RESULT variant")
         };
         assert_eq!(
             parse_a_pinned_red_example(20, &row("U_verify_off", "3", "36")),
@@ -5395,16 +9096,186 @@
                 ordinal: 36,
             })
         );
-        let seven_columns = row("L", "2", "3")
+        let twenty_columns = row("L", "2", "3")
             .rsplit_once('\t')
-            .map(|(first_seven_columns, _eighth_column)| first_seven_columns.to_string())
-            .expect("造的行有八段");
-        assert!(parse_a_pinned_red_example(20, &seven_columns).is_err());
+            .map(|(first_twenty_columns, _twenty_first_column)| first_twenty_columns.to_string())
+            .expect("造的行有二十一段");
+        assert!(parse_a_pinned_red_example(20, &twenty_columns).is_err());
         assert!(parse_a_pinned_red_example(20, &row("U", "2", "3")).is_err());
         assert!(parse_a_pinned_red_example(20, &row("X", "1", "3")).is_err());
         assert!(parse_a_pinned_red_example(20, &row("L", "2", "three")).is_err());
     }
 
+    /// 今天钉住的表里 X:2:2 那一行自己的八段（原样抄来）。
+    const OWN_COLUMNS_OF_THE_PINNED_CROSS_INSTANCE_ROW: [&str; 8] = [
+        "371a71190dce62e090cbf0a361b11069fb68cc63ab2a44e88398bb2f0570f2f0",
+        "X",
+        "2",
+        "2",
+        "e165-replay-bounded-enumeration-2026-10-01-segment2-red-examples.tsv:2",
+        "crates/mutations.tsv#步 3：前缀跨实例边界（把别的实例的记录也接上）",
+        "E7RESULT name=state domain=X ring_slots=64 n=2 ordinal=2 counters=states,j6_premise_holds_judged state=shape=1c0/next_instance:1c0;on_disk=-,good;residue_at=none;chosen=0;farthest=0;verify=true;first=returned:txg100:watermark1000:applied0",
+        "E7RESULT name=state domain=X ring_slots=64 n=2 ordinal=2 counters=states,J1,J2,J3,J4,j6_premise_holds_judged,off_timeline_landings state=shape=1c0/next_instance:1c0;on_disk=-,good;residue_at=none;chosen=0;farthest=0;verify=true;first=returned:txg101:watermark1001:applied1",
+    ];
+
+    /// 那一行 ⑤ 指的红例账 `research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-red-examples.tsv` 第 2 行的十三段（原样抄来）。
+    const LEDGER_COLUMNS_OF_THE_PINNED_CROSS_INSTANCE_ROW: [&str; 13] = [
+        "tsv-23",
+        "23+355",
+        "X",
+        "2",
+        "2",
+        "counters",
+        "+J1,+J2,+J3,+J4,+off_timeline_landings",
+        "true",
+        "true",
+        "true",
+        "target/release/e165_replay_bounded_enumeration --replay-state X:2:2",
+        "E7RESULT name=state domain=X ring_slots=64 n=2 ordinal=2 counters=states,j6_premise_holds_judged state=shape=1c0/next_instance:1c0;on_disk=-,good;residue_at=none;chosen=0;farthest=0;verify=true;first=returned:txg100:watermark1000:applied0",
+        "E7RESULT name=state domain=X ring_slots=64 n=2 ordinal=2 counters=states,J1,J2,J3,J4,j6_premise_holds_judged,off_timeline_landings state=shape=1c0/next_instance:1c0;on_disk=-,good;residue_at=none;chosen=0;farthest=0;verify=true;first=returned:txg101:watermark1001:applied1",
+    ];
+
+    /// 一张表：两行表头注释，第 3 行是 X:2:2 那一行自己的八段接上给的账行。
+    fn pinned_table_of_the_cross_instance_row_with_ledger_columns(
+        ledger_columns: &[&str],
+    ) -> String {
+        let mut row_columns: Vec<&str> = OWN_COLUMNS_OF_THE_PINNED_CROSS_INSTANCE_ROW.to_vec();
+        row_columns.extend_from_slice(ledger_columns);
+        format!(
+            "# 钉住的红例\n# 二十一段，制表符分隔\n{}\n",
+            row_columns.join("\t")
+        )
+    }
+
+    /// X:2:2 那一行的账行把第 `ledger_column_number` 段（从 1 数）换成 `replacement`，整张表解出来的问题。
+    fn problems_when_one_ledger_column_of_the_cross_instance_row_is_replaced(
+        ledger_column_number: usize,
+        replacement: &str,
+    ) -> Vec<String> {
+        let mut ledger_columns = LEDGER_COLUMNS_OF_THE_PINNED_CROSS_INSTANCE_ROW;
+        ledger_columns[ledger_column_number - 1] = replacement;
+        let parse = parse_the_pinned_table(
+            &pinned_table_of_the_cross_instance_row_with_ledger_columns(&ledger_columns),
+        );
+        assert_eq!(parse.data_line_count, 1, "表里只有一行数据");
+        assert!(
+            parse.examples.is_empty(),
+            "账行第 {ledger_column_number} 段与钉表自己的段不同，这一行不该收：{parse:?}"
+        );
+        parse.problems_of_the_rows_that_do_not_parse
+    }
+
+    /// 二十一段、⑨ 起是红例账那一行原样的表：每行都收，②③④ 与 ① 照表里的解出来。
+    #[test]
+    fn a_pinned_table_whose_rows_carry_the_whole_ledger_row_parses_every_row() {
+        let parse =
+            parse_the_pinned_table(&pinned_table_of_the_cross_instance_row_with_ledger_columns(
+                &LEDGER_COLUMNS_OF_THE_PINNED_CROSS_INSTANCE_ROW,
+            ));
+        assert_eq!(parse.data_line_count, 1, "两行注释不算数据行");
+        assert_eq!(
+            parse.problems_of_the_rows_that_do_not_parse,
+            Vec::<String>::new(),
+            "合法的二十一段行不该有问题"
+        );
+        assert_eq!(
+            parse.examples,
+            vec![PinnedRedExample {
+                line_number_in_the_table: 3,
+                input_fingerprint:
+                    "371a71190dce62e090cbf0a361b11069fb68cc63ab2a44e88398bb2f0570f2f0".to_string(),
+                domain_text: "X".to_string(),
+                size_text: "2".to_string(),
+                domain: Domain::CrossInstance,
+                size: 2,
+                ordinal: 2,
+            }]
+        );
+    }
+
+    /// 只有八段的旧行：报「没整行抄红例账」并给补的脚本，这一行不收。
+    #[test]
+    fn a_pinned_table_row_with_only_the_old_eight_columns_is_rejected_for_not_carrying_the_ledger_row(
+    ) {
+        let parse = parse_the_pinned_table(
+            &pinned_table_of_the_cross_instance_row_with_ledger_columns(&[]),
+        );
+        assert_eq!(parse.data_line_count, 1, "表里只有一行数据");
+        assert!(parse.examples.is_empty(), "八段的旧行不该收：{parse:?}");
+        assert_eq!(parse.problems_of_the_rows_that_do_not_parse.len(), 1);
+        assert!(
+            parse.problems_of_the_rows_that_do_not_parse[0].starts_with(
+                "第 3 行：这一行没整行抄红例账，用 research/scripts/e165-pin-red-examples.py 补"
+            ),
+            "要点名行号、说没整行抄、给补的脚本：{:?}",
+            parse.problems_of_the_rows_that_do_not_parse
+        );
+    }
+
+    /// 账行第 12 段（原实现的逐态行）与 ⑦ 不同：拒，点名第 3 行的 ⑳ 与 ⑦。
+    #[test]
+    fn a_pinned_table_row_whose_ledger_original_state_row_differs_from_column_seven_is_rejected() {
+        let problems = problems_when_one_ledger_column_of_the_cross_instance_row_is_replaced(
+            12,
+            "E7RESULT name=state domain=X ring_slots=64 n=2 ordinal=2 counters=states state=shape=1c0;first=returned:txg100:watermark1000:applied0",
+        );
+        assert_eq!(problems.len(), 1, "{problems:?}");
+        assert!(
+            problems[0].starts_with("第 3 行：⑳（红例账第 12 段）与 ⑦ 不同"),
+            "{problems:?}"
+        );
+    }
+
+    /// 账行第 13 段（变体的逐态行）与 ⑧ 不同：拒，点名第 3 行的 ㉑ 与 ⑧。
+    #[test]
+    fn a_pinned_table_row_whose_ledger_variant_state_row_differs_from_column_eight_is_rejected() {
+        let problems = problems_when_one_ledger_column_of_the_cross_instance_row_is_replaced(
+            13,
+            "E7RESULT name=state domain=X ring_slots=64 n=2 ordinal=2 counters=states state=shape=1c0;first=returned:txg101:watermark1001:applied1",
+        );
+        assert_eq!(problems.len(), 1, "{problems:?}");
+        assert!(
+            problems[0].starts_with("第 3 行：㉑（红例账第 13 段）与 ⑧ 不同"),
+            "{problems:?}"
+        );
+    }
+
+    /// 账行第 3 段（域）与 ② 不同：拒，点名第 3 行的 ⑪ 与 ②。
+    #[test]
+    fn a_pinned_table_row_whose_ledger_domain_differs_from_column_two_is_rejected() {
+        let problems =
+            problems_when_one_ledger_column_of_the_cross_instance_row_is_replaced(3, "L");
+        assert_eq!(problems.len(), 1, "{problems:?}");
+        assert!(
+            problems[0].starts_with("第 3 行：⑪（红例账第 3 段）与 ② 不同"),
+            "{problems:?}"
+        );
+    }
+
+    /// 账行第 4 段（N）与 ③ 不同：拒，点名第 3 行的 ⑫ 与 ③。
+    #[test]
+    fn a_pinned_table_row_whose_ledger_size_differs_from_column_three_is_rejected() {
+        let problems =
+            problems_when_one_ledger_column_of_the_cross_instance_row_is_replaced(4, "3");
+        assert_eq!(problems.len(), 1, "{problems:?}");
+        assert!(
+            problems[0].starts_with("第 3 行：⑫（红例账第 4 段）与 ③ 不同"),
+            "{problems:?}"
+        );
+    }
+
+    /// 账行第 5 段（序号）与 ④ 不同：拒，点名第 3 行的 ⑬ 与 ④。
+    #[test]
+    fn a_pinned_table_row_whose_ledger_ordinal_differs_from_column_four_is_rejected() {
+        let problems =
+            problems_when_one_ledger_column_of_the_cross_instance_row_is_replaced(5, "7");
+        assert_eq!(problems.len(), 1, "{problems:?}");
+        assert!(
+            problems[0].starts_with("第 3 行：⑬（红例账第 5 段）与 ④ 不同"),
+            "{problems:?}"
+        );
+    }
+
     /// 重放出来的逐态行对着表里那一行判：指纹相同、指纹漂了、序号不在这个 (域, N) 里，三种各判各的。
     #[test]
     fn a_pinned_red_example_whose_recomputed_fingerprint_differs_is_reported_as_drifted() {
```

## E170 导出装置（新文件，全文在 crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs，3742 行，sha256 8c526293…）
