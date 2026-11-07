# m4-gpugc-shared-bytes-code-r1：改动全文（产品代码的补丁块，按打进主工作区的次序原样抄录）

## 第 86 批（含第 89–91 批续改），补丁目录 ~/.local/share/singlefs/work/m4-gpugc-shared-unit-bytes-r4/patch/

### crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs

```diff
diff --git a/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs b/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs
index d0f0ccc..cf60c2d 100644
--- a/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs
+++ b/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs
@@ -8627,7 +8627,7 @@ mod tests {
 
     /// 共用的存法：写成全是零的块不存，读回照旧是零；各存各的存法照存。
     #[test]
-    fn a_chunk_written_back_to_zeroes_is_not_stored_and_reads_back_zero() {
+    fn chunk_written_back_to_zeroes_is_not_stored_and_reads_back_zero() {
         let recording: SharedRecordingDigest = Arc::new(Mutex::new(RecordingDigest::empty()));
         let mut shared = empty_pool_devices(4 * MEMORY_DEVICE_CHUNK_BYTES, &recording);
         let mut copied = empty_pool_devices_storing_the_chunks(
@@ -8657,7 +8657,7 @@ mod tests {
 
     /// 共用的存法：拷一个池只拷块的引用（拷出来的与原来的指同一份），拷出来的那个池写到一块才复制那一块，原来的读回照旧。
     #[test]
-    fn a_copied_pool_shares_the_chunks_until_one_of_its_devices_writes_a_chunk() {
+    fn copied_pool_shares_the_chunks_until_one_of_its_devices_writes_a_chunk() {
         let recording: SharedRecordingDigest = Arc::new(Mutex::new(RecordingDigest::empty()));
         let mut devices = empty_pool_devices(4 * MEMORY_DEVICE_CHUNK_BYTES, &recording);
         let written = chunk_content(0);
```

### crates/singlefs-core/src/instance_table_sweep.rs

```diff
diff --git a/crates/singlefs-core/src/instance_table_sweep.rs b/crates/singlefs-core/src/instance_table_sweep.rs
index dd83fb1..2fbf51b 100644
--- a/crates/singlefs-core/src/instance_table_sweep.rs
+++ b/crates/singlefs-core/src/instance_table_sweep.rs
@@ -21,6 +21,7 @@
 //! 第一版不做的：现行那一版树表 0 条时不清扫（[`NoRowDeletedThisRound::SweepOnAVersionWithoutFileNotSupportedInTheFirstVersion`]：
 //! 那一版的账住根记录那一项、不在树表里，「跳过根环里可读根的账」那一半读不全；清扫发布在那一版上的形状条款也没写）。
 use std::collections::{BTreeMap, BTreeSet};
+use std::sync::Arc;
 
 use singlefs_format::{
     DATA_UNIT_HEADER_BYTES, INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE,
@@ -652,7 +653,7 @@ pub fn erase_the_heads_and_publish_the_sweep<Device: BlockDevice>(
             &mut pool,
             allocator,
             plan.as_publish_plan(),
-            Some(&current_file_version),
+            Some(Arc::as_ref(&current_file_version)),
         ),
         SweepPublishPlacements::EveryFreeSlotTakenAwayByTheTestOnlySwitch => {
             let mut allocator_without_a_free_slot = allocator.clone();
@@ -661,7 +662,7 @@ pub fn erase_the_heads_and_publish_the_sweep<Device: BlockDevice>(
                 &pool,
                 &mut allocator_without_a_free_slot,
                 &plan.as_publish_plan(),
-                Some(&current_file_version),
+                Some(Arc::as_ref(&current_file_version)),
                 current_file_version.tree_identifiers,
                 ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt,
                 UnmountMarker::NotWrittenByTheUnmountSequence,
```

### crates/singlefs-core/src/mount.rs

```diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 91e43c0..002a033 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -114,6 +114,7 @@ use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES, ROOT_RING_REGIONS, SLOT_BYTES
 use std::borrow::Cow;
 use std::cell::{Cell, RefCell};
 use std::collections::{BTreeMap, BTreeSet};
+use std::sync::Arc;
 
 pub use crate::instance_table::{InstanceRow, InstanceTableRecords};
 
@@ -1099,8 +1100,9 @@ pub struct MountOutput {
     /// 这次挂载重读过哪几样（C554 乙）：读阶段在哪一遍判完、最新那条根的实例表在哪一遍读出来、重建分配器读根环在哪一遍读出有效根。
     pub rereads: RereadsOfThisMount,
     /// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）发成的每一步，按先后：接在写行、暖机与推的抬 F 之后，走到了空时最后一个就是
-    /// `Mounted::current`；所选那一版的 `deleted_inodes` 是空的（常态）时空。
-    pub deletion_queue_continuation: Vec<TransactionOutput>,
+    /// `Mounted::current`（两处指着同一份，不拷一整版；写行那一版与暖机各版同样与 `Mounted::current` 共享）；
+    /// 所选那一版的 `deleted_inodes` 是空的（常态）时空。
+    pub deletion_queue_continuation: Vec<Arc<TransactionOutput>>,
     /// 挂载续做里为空间推的那几串抬 F，按先后（与 `deletion_queue_continuation` 的各步按 txg 交错）：走空里一次删除类发布被拒是空间不够
     /// （不判式子那一判、落得下那一判照判，D28（挂载期承诺量） 已定项 1）时推一串、回收之后接着走（用户 2026-10-03 定案
     /// 「挂载续做被空间拒：同样推抬 F 再试」，与会话那一处同一个做法）。一次都没推的是空的。
@@ -1350,20 +1352,20 @@ impl PreviousVersion {
                     PublishedUnit {
                         slot: root.instance_table.locations[0].slot,
                         identity: TransactionUnit::InstanceTable,
-                        bytes: read_unit_via_locations(
+                        bytes: Arc::from(read_unit_via_locations(
                             devices,
                             &root.instance_table.locations,
                             data_unit_bytes,
-                        )?,
+                        )?),
                     },
                     PublishedUnit {
                         slot: root.tree_table.locations[0].slot,
                         identity: TransactionUnit::TreeTable,
-                        bytes: read_unit_via_locations(
+                        bytes: Arc::from(read_unit_via_locations(
                             devices,
                             &root.tree_table.locations,
                             node_bytes,
-                        )?,
+                        )?),
                     },
                 ];
                 if let Some(tree) =
@@ -1377,11 +1379,11 @@ impl PreviousVersion {
                         units.push(PublishedUnit {
                             slot: pointer.locations[0].slot,
                             identity: role_of_allocation_record_tree_node(*node),
-                            bytes: read_unit_via_locations(
+                            bytes: Arc::from(read_unit_via_locations(
                                 devices,
                                 &pointer.locations,
                                 node_bytes,
-                            )?,
+                            )?),
                         });
                     }
                 }
@@ -4599,12 +4601,13 @@ pub fn unmount<Device: BlockDevice>(
         }
     };
     let rollback_floor = current_file_version.root.checkpoint_txg;
+    // 抬 F 就地推进现行那一版：还有别人指着它（版本列表里那一份）时先拷一份自己的再推，别人指着的那一份不动。
     let raised = raise_the_floor_through(
         RaiseFloorEntry::Unmount,
         parameters,
         devices,
         allocator,
-        current_file_version,
+        Arc::make_mut(current_file_version),
         rollback_floor,
         shadow_ledger,
     )?;
@@ -5916,7 +5919,7 @@ fn publish_empty_after<Device: BlockDevice>(
             pool,
             allocator,
             empty_publish_plan_after_file_version(current_file_version, instance),
-            Some(current_file_version),
+            Some(Arc::as_ref(current_file_version)),
         )
         .map(PoolVersion::of_a_version_of_trees),
         PoolVersion::WithoutFile(current_version_without_file) => {
@@ -6062,7 +6065,7 @@ fn dry_run_of_the_publishes_after_acquisition<Device: BlockDevice>(
                     pool,
                     &mut copy,
                     &plan,
-                    Some(current_file_version),
+                    Some(Arc::as_ref(current_file_version)),
                     current_file_version.tree_identifiers,
                     ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
                     UnmountMarker::NotWrittenByTheUnmountSequence,
@@ -6871,7 +6874,7 @@ fn rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configurati
             !unit.bytes.is_empty()
                 && reader
                     .read(device, unit.slot.to_device_offset(), unit.bytes.len())
-                    .is_some_and(|bytes_on_this_device| bytes_on_this_device == unit.bytes)
+                    .is_some_and(|bytes_on_this_device| bytes_on_this_device[..] == unit.bytes[..])
         });
     if !every_unit_of_the_version_is_on_the_device {
         return Some(RewriteGateNotMet::UnitsOfTheSelectedVersionNotAllOnTheDevice);
@@ -7087,7 +7090,7 @@ fn units_of_the_version_missing_on_a_device_behind_it<Reader: PoolReader + ?Size
         .filter(|unit| {
             !reader
                 .read(device, unit.slot.to_device_offset(), unit.bytes.len())
-                .is_some_and(|bytes_on_this_device| bytes_on_this_device == unit.bytes)
+                .is_some_and(|bytes_on_this_device| bytes_on_this_device[..] == unit.bytes[..])
         })
         .map(|unit| (unit.identity, unit.slot))
         .collect();
@@ -8191,7 +8194,8 @@ fn compaction_intent_continued_at_the_mount<Device: BlockDevice>(
 /// 挂载续做交回的东西（[`continue_the_deletion_queue_until_it_is_empty`]）：发成的每一步与为空间推的那几串抬 F，两样按 txg 交错；
 /// 走到哪儿停（走到了空，或推满仍放不下、队列留着）。
 struct DeletionQueueContinuation {
-    steps_published: Vec<TransactionOutput>,
+    /// 最后一步与挂载交回的现行那一版指着同一份（`transaction::DeletionQueueWalk::steps_published` 那样共享）。
+    steps_published: Vec<Arc<TransactionOutput>>,
     floor_raises: Vec<RaisedFloor>,
     refusals_that_pushed_the_floor_raises: Vec<PublishError>,
     outcome: DeletionQueueContinuationOutcome,
@@ -8204,6 +8208,7 @@ impl DeletionQueueContinuation {
         let mut persisted: Vec<&TransactionOutput> = self
             .steps_published
             .iter()
+            .map(Arc::as_ref)
             .chain(
                 self.floor_raises
                     .iter()
@@ -8273,7 +8278,7 @@ fn continue_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
         return Ok(continuation);
     }
     let switches = allocator.deletion_switches();
-    let mut walked = current_version_of_trees.clone();
+    let mut walked = Arc::clone(current_version_of_trees);
     let mut publishes_pushed_in_this_admission = 0;
     loop {
         let (walk, writes_of_failed_publishes) = {
@@ -8298,13 +8303,13 @@ fn continue_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
         continuation.steps_published.extend(steps_published);
         let refusal = match stopped_by {
             Ok(()) => {
-                *current = PoolVersion::of_a_version_of_trees(walked);
+                *current = PoolVersion::of_a_shared_version_of_trees(walked);
                 return Ok(continuation);
             }
             Err(refusal) => refusal,
         };
         if !refusal_is_short_of_space(&refusal) {
-            *current = PoolVersion::of_a_version_of_trees(walked);
+            *current = PoolVersion::of_a_shared_version_of_trees(walked);
             let writes_of_persisted_publishes: Vec<WritesByStructureKind> =
                 writes_of_the_publishes_before_the_continuation
                     .iter()
@@ -8322,11 +8327,12 @@ fn continue_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
                 writes_of_failed_publishes,
             })));
         }
+        // 抬 F 就地推进走到的那一版：它还与续做交回的最后一步指着同一份时先拷一份自己的再推，交回的那一步不动。
         match push_one_floor_raise_within_the_admission_budget(
             parameters,
             devices,
             allocator,
-            &mut walked,
+            Arc::make_mut(&mut walked),
             shadow_ledger,
             publishes_pushed_in_this_admission,
         ) {
@@ -8338,7 +8344,7 @@ fn continue_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
                     .push(refusal);
             }
             Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(stop)) => {
-                *current = PoolVersion::of_a_version_of_trees(walked);
+                *current = PoolVersion::of_a_shared_version_of_trees(walked);
                 continuation.outcome =
                     DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises {
                         last_refusal: refusal,
@@ -8347,7 +8353,7 @@ fn continue_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
                 return Ok(continuation);
             }
             Err(cause) => {
-                *current = PoolVersion::of_a_version_of_trees(walked);
+                *current = PoolVersion::of_a_shared_version_of_trees(walked);
                 return Err(MountError::FloorRaiseFailedAfterTheMountsPublishes(
                     Box::new(FloorRaiseFailedAfterTheMountsPublishes {
                         cause,
@@ -8694,11 +8700,12 @@ fn acquire_then_publish_until_the_continuation<Device: BlockDevice>(
                     .collect();
             // 写行与暖机那几次的写入口装的节点容量：之后推抬 F、再判的读数按同一档算。
             let node_capacities_of_the_writer = pool.code_two_tree_node_capacities();
+            // 抬 F 就地推进现行那一版：它与写行那一版或最后一次暖机指着同一份，先拷一份自己的再推，交回的那几版不动。
             push_floor_raises_after_the_row_publish(
                 parameters,
                 devices,
                 &mut allocator,
-                current_file_version,
+                Arc::make_mut(current_file_version),
                 start.shadow_ledger,
                 node_capacities_of_the_writer,
                 refusal_before_acquisition,
@@ -14202,3 +14209,222 @@ mod first_txg_past_the_ring_slots_read_bad_at_mount_tests {
         );
     }
 }
+
+#[cfg(test)]
+mod versions_and_unit_bytes_shared_instead_of_copied_tests {
+    //! 判决 `research/prompts/m4-gpugc-publish-growth-r1-main-verification.md` 第二节 Q2.1 甲档与 Q2.2 `Arc<TransactionOutput>` 那一小块：
+    //! 照抄的单元与上一版共用同一份字节，版本列表、挂载交回值与现行那一版共用同一份 `TransactionOutput`。判的是「指着同一份」
+    //! （`Arc::ptr_eq`），内容相同的拷贝判红；每条另带一个对照，证明这一判分得出共用与不共用。
+    use std::sync::Arc;
+
+    use super::empty_publish_plan_after_file_version;
+    use super::pool_in_memory_after_the_first_file::pool_after_the_first_file;
+    use super::{mount_writable, ParametersAndDeviceTableOfTheMount, ShadowLedger};
+    use crate::address::{DataUnitIndexInFile, InodeNumber};
+    use crate::compaction_plan::CompactionOfThisMount;
+    use crate::mounted_session::{MountedSession, UserChange};
+    use crate::transaction::{
+        publish_version, FirstFile, PoolVersion, PoolWriter, PublishedUnit, TransactionUnit,
+        FIRST_INODE_NUMBER,
+    };
+
+    /// 这几条用例里改动的写入时刻：晚于池里第一个文件的写入时刻。
+    const WRITE_TIME_SECONDS_OF_THE_CHANGE: u64 = 1_788_000_100;
+
+    /// 池里第一个文件之后、这个进程里接着它的会话（没经过可写挂载：盘表读盘造，整理状态从新挂载起）。
+    fn session_after_the_first_file() -> (
+        MountedSession,
+        Vec<(
+            crate::address::DeviceIdentity,
+            super::shadow_ledger_exempts_and_isolates_slot_by_slot_tests::SparseDeviceInMemory,
+        )>,
+    ) {
+        let pool = pool_after_the_first_file();
+        let parameters_and_device_table =
+            ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk(&pool.devices)
+                .expect("mkfs 与第一个文件写过的两块盘上择得出系统配置、盘表与它一致");
+        let session = MountedSession {
+            allocator: pool.allocator,
+            current: PoolVersion::of_a_version_of_trees(pool.current),
+            instance: pool.instance,
+            shadow_ledger: ShadowLedger::On,
+            parameters_and_device_table,
+            compaction: CompactionOfThisMount::of_a_new_mount(),
+        };
+        (session, pool.devices)
+    }
+
+    /// 甲档：空发布照抄的每一个单元（第一个文件的数据单元、extent 树根、inode 树、实例表链）与上一版那一份指着同一份字节；
+    /// 对照：这次重写的树表是新装的字节，不与上一版共用。
+    #[test]
+    fn an_empty_publish_shares_the_bytes_of_every_unit_it_carries_with_the_version_before() {
+        let mut pool = pool_after_the_first_file();
+        let previous = pool.current.clone();
+        let next = {
+            let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
+            publish_version(
+                &mut writer,
+                &mut pool.allocator,
+                empty_publish_plan_after_file_version(&previous, pool.instance),
+                Some(&previous),
+            )
+            .expect("4 GiB 两盘上空发布落得下")
+        };
+        let carried: Vec<&PublishedUnit> = next
+            .units
+            .iter()
+            .filter(|unit| !next.rewritten.contains(&unit.identity))
+            .collect();
+        assert!(
+            carried
+                .iter()
+                .any(|unit| unit.identity == TransactionUnit::Data(DataUnitIndexInFile::FIRST)),
+            "前提：空发布照抄第一个文件的数据单元（照抄的那几个里有它）"
+        );
+        for unit in &carried {
+            assert!(
+                Arc::ptr_eq(&unit.bytes, &previous.unit(unit.identity).bytes),
+                "照抄的 {:?} 与上一版那一份指着同一份字节，不是内容相同的拷贝",
+                unit.identity
+            );
+        }
+        assert!(
+            next.rewritten.contains(&TransactionUnit::TreeTable),
+            "前提：空发布重写树表"
+        );
+        assert!(
+            !Arc::ptr_eq(
+                &next.unit(TransactionUnit::TreeTable).bytes,
+                &previous.unit(TransactionUnit::TreeTable).bytes
+            ),
+            "对照：重写的树表是这次装的字节，与上一版不共用（共用这一判分得出两种）"
+        );
+    }
+
+    /// `Arc<TransactionOutput>`：一次覆盖写经会话发成，交回的版本列表最后那一版与会话的现行那一版指着同一份；
+    /// 对照：上一次改动之前的现行那一版与这次的不是同一份。
+    #[test]
+    fn user_change_hands_back_the_version_it_published_as_the_one_the_session_holds_as_current() {
+        let (mut session, mut devices) = session_after_the_first_file();
+        let current_before = Arc::clone(
+            session
+                .current
+                .shared_version_of_trees()
+                .expect("第一个文件之后现行那一版有树"),
+        );
+        let content = [0x5A_u8; 2000];
+        let published = session
+            .publish_user_change(
+                &mut devices,
+                UserChange::Overwrite(FirstFile {
+                    content: &content,
+                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_CHANGE,
+                }),
+            )
+            .unwrap_or_else(|refusal| panic!("4 GiB 两盘上覆盖写经会话发得成：{refusal:?}"));
+        assert_eq!(
+            published.versions_published.len(),
+            1,
+            "前提：队列空、没推抬 F，这次改动只发成一版"
+        );
+        let current_after = session
+            .current
+            .shared_version_of_trees()
+            .expect("覆盖写之后现行那一版有树");
+        assert!(
+            Arc::ptr_eq(&published.versions_published[0], current_after),
+            "交回的那一版与会话的现行那一版指着同一份，不是内容相同的拷贝"
+        );
+        assert!(
+            !Arc::ptr_eq(&current_before, current_after),
+            "对照：改动之前的现行那一版与之后的不是同一份"
+        );
+    }
+
+    /// `Arc<TransactionOutput>`：经会话 unlink 唯一那个文件（意图那一次，再走空到最后一批），走空交回的最后一步与会话的现行那一版
+    /// 指着同一份（走空每发成一步不再拷一整版）；对照：意图那一版与现行那一版不是同一份。
+    #[test]
+    fn an_unlink_through_the_session_hands_back_the_last_step_of_the_walk_as_the_session_current() {
+        let (mut session, mut devices) = session_after_the_first_file();
+        let published = session
+            .publish_user_change(
+                &mut devices,
+                UserChange::Unlink {
+                    inode: InodeNumber(FIRST_INODE_NUMBER),
+                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_CHANGE,
+                },
+            )
+            .unwrap_or_else(|refusal| panic!("unlink 唯一那个文件经会话发得成：{refusal:?}"));
+        assert!(
+            published.versions_published.len() >= 2,
+            "前提：意图那一次之后至少走空一步，实际 {} 版",
+            published.versions_published.len()
+        );
+        let current_after = session
+            .current
+            .shared_version_of_trees()
+            .expect("unlink 之后现行那一版有树（文件集合为空）");
+        assert!(
+            Arc::ptr_eq(
+                published
+                    .versions_published
+                    .last()
+                    .expect("上面判过至少两版"),
+                current_after
+            ),
+            "走空交回的最后一步与会话的现行那一版指着同一份，不是内容相同的拷贝"
+        );
+        assert!(
+            !Arc::ptr_eq(&published.versions_published[0], current_after),
+            "对照：意图那一版与走空之后的现行那一版不是同一份"
+        );
+    }
+
+    /// `Arc<TransactionOutput>`：同一组盘可写重挂，挂载交回值里最后那一次暖机与挂载交回的现行那一版指着同一份；
+    /// 对照：写行那一版与现行那一版不是同一份（暖机至少一次）。
+    #[test]
+    fn writable_remount_hands_back_its_last_warm_up_publish_as_the_current_version() {
+        let mut pool = pool_after_the_first_file();
+        let mounted = mount_writable(&pool.parameters, &mut pool.devices)
+            .expect("第一个文件之后可写重挂做得成");
+        assert!(
+            mounted.output.deletion_queue_continuation.is_empty()
+                && mounted.output.space_admission.floor_raises().is_empty(),
+            "前提：队列空、准入够，挂载不续做、不推抬 F"
+        );
+        let last_warm_up_publish = mounted
+            .output
+            .warm_up_publishes
+            .last()
+            .expect("前提：本实例的根要覆盖两块盘，写行之后至少暖机一次");
+        assert_eq!(
+            last_warm_up_publish.root(),
+            mounted.current.root(),
+            "前提：暖机之后这次挂载没再发布（没续做、没整理、清扫没发），现行那一版就是最后一次暖机"
+        );
+        let current = mounted
+            .current
+            .shared_version_of_trees()
+            .expect("第一个文件之后的一版有树");
+        assert!(
+            Arc::ptr_eq(
+                last_warm_up_publish
+                    .shared_version_of_trees()
+                    .expect("有树的一版上暖机交回有树的一版"),
+                current
+            ),
+            "挂载交回的最后一次暖机与现行那一版指着同一份，不是内容相同的拷贝"
+        );
+        assert!(
+            !Arc::ptr_eq(
+                mounted
+                    .output
+                    .row_publish
+                    .shared_version_of_trees()
+                    .expect("有树的一版上写行交回有树的一版"),
+                current
+            ),
+            "对照：写行那一版与现行那一版不是同一份"
+        );
+    }
+}
```

### crates/singlefs-core/src/mounted_session.rs

```diff
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index 56b5bf7..dc61fb1 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -20,6 +20,7 @@
 
 use std::cell::{Cell, RefCell};
 use std::collections::{BTreeMap, BTreeSet};
+use std::sync::Arc;
 use std::time::Duration;
 
 use singlefs_format::{CLUSTER_SEGMENT_SLOTS, SLOT_BYTES};
@@ -509,7 +510,8 @@ enum PacingOfTheRun<'caller> {
 /// 一次用户改动里推的抬 F 之外发成的每一版，按先后：先走空的删除步（D8（核心索引结构） 已定项 16 第 6 条）、这次改动本身
 /// （unlink 是意图那次）、unlink 之后走空的删除步；与推的抬 F 的空发布按 txg 交错。最后一个是会话的 `current`
 /// （之后没再推抬 F 时）。交给调用方逐次比（模型对拍）：会话的 `current` 只是最后那一版。
-pub type VersionsPublishedInThisChange = Vec<TransactionOutput>;
+/// 每一版按引用计数共享：最后一个与会话的 `current` 指着同一份（[`PoolVersion`] 那一段），放进这里不拷一整版。
+pub type VersionsPublishedInThisChange = Vec<Arc<TransactionOutput>>;
 
 /// 一次用户改动没发成。成员按调用方要做的决定分：换一份盘表再发、换回这个池的盘、换一个版本再改、原样看发布路径的错、报空间不够。
 #[derive(Debug)]
@@ -1783,7 +1785,7 @@ fn rehearse_the_push_then_the_remainder_of_the_change<Device: BlockDevice>(
             .map(|_published| ())
         }
         RemainderOfTheUserChange::WalkTheQueue => {
-            let mut version_to_walk = rehearsed_version.into_version_of_trees().expect(
+            let mut version_to_walk = rehearsed_version.into_shared_version_of_trees().expect(
                 "推环接在有树的一版后面、发的是带文件那一版的空发布（树表 0 条那一版推环入口先拒了），推完还是有树的一版",
             );
             publish_the_queue_until_it_is_empty_raising_the_floor_for_space(
@@ -1911,7 +1913,8 @@ fn wide_protection_at_the_ceiling(
 /// 发成的每一版按先后（最后一版是会话的 `current`），与写掉的那几槽（C 那一圈在这几槽记着的落点都已解开）。
 #[derive(Debug)]
 pub struct RootRingPushed {
-    pub versions_published: Vec<TransactionOutput>,
+    /// 最后一版与会话的 `current` 指着同一份（[`VersionsPublishedInThisChange`] 那样共享）。
+    pub versions_published: Vec<Arc<TransactionOutput>>,
     pub ring_slots_written: Vec<RootRingSlot>,
 }
 
@@ -1969,7 +1972,7 @@ pub struct RootRingPushFailedAfterTheRehearsal {
     pub failed_publish_in_the_push: usize,
     pub publishes_in_the_push: usize,
     pub cause: PublishError,
-    pub versions_published: Vec<TransactionOutput>,
+    pub versions_published: Vec<Arc<TransactionOutput>>,
 }
 
 /// 为空间不够推抬 F 时抬 F 自己报了错：`cause` 是抬 F 的错原样（`mount::push_one_floor_raise_within_the_admission_budget` 的 `Err`），
@@ -2045,7 +2048,7 @@ fn publish_the_queue_until_it_is_empty<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut [(DeviceIdentity, Device)],
     allocator: &mut PoolAllocator,
-    current: &mut TransactionOutput,
+    current: &mut Arc<TransactionOutput>,
     instance: InstanceGeneration,
 ) -> DeletionQueueWalk {
     let switches: DeletionSwitches = allocator.deletion_switches();
@@ -2128,9 +2131,9 @@ fn relocate_the_old_data_units_kept_for_the_snapshots<Device: BlockDevice>(
             relocated,
             floor_raises,
         } => {
-            let relocated = *relocated;
-            versions_published.push(relocated.clone());
-            *current = PoolVersion::of_a_version_of_trees(relocated);
+            let relocated = Arc::new(*relocated);
+            versions_published.push(Arc::clone(&relocated));
+            *current = PoolVersion::of_a_shared_version_of_trees(relocated);
             RelocationOfTheOldDataUnitsKeptForTheSnapshots::Relocated {
                 mapping_keys: planned
                     .into_iter()
@@ -3060,7 +3063,7 @@ impl MountedSession {
         // 队列空（任何一版都是常态）时一个写都不发、不复制；不空时在一份复制上走，走到哪一步（连同推的抬 F）`current` 就是哪一版。
         // 销毁中的快照（判决 S4：删快照照 unlink 的过程做，第七节岔路 4）、截断条目（判决 m4-trunc-r1 K1）与 `deleted_inodes` 一起走空。
         if !current_version_of_trees.queue_of_committed_intents_is_empty() {
-            let mut walked = current_version_of_trees.clone();
+            let mut walked = Arc::clone(current_version_of_trees);
             let walk = publish_the_queue_until_it_is_empty_raising_the_floor_for_space(
                 parameters,
                 devices,
@@ -3071,7 +3074,7 @@ impl MountedSession {
                 RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),
                 published_in_this_change,
             );
-            *current = PoolVersion::of_a_version_of_trees(walked);
+            *current = PoolVersion::of_a_shared_version_of_trees(walked);
             published_in_this_change = walk?;
         }
         let mut publishes_pushed = 0;
@@ -3122,10 +3125,11 @@ impl MountedSession {
             ) {
                 Ok(published) => {
                     // unlink：意图发成之后接着把全部批走完（一次调用 = 走空 + 意图 + 全部批）；别的改动之后队列本来就空，一个写都不发。
-                    let mut published = published;
+                    // 版本列表与会话的 `current` 指着同一份发成的那一版（不拷一整版）。
+                    let mut published = Arc::new(published);
                     published_in_this_change
                         .versions_published
-                        .push(published.clone());
+                        .push(Arc::clone(&published));
                     let walk = publish_the_queue_until_it_is_empty_raising_the_floor_for_space(
                         parameters,
                         devices,
@@ -3136,7 +3140,7 @@ impl MountedSession {
                         RemainderOfTheUserChange::WalkTheQueue,
                         published_in_this_change,
                     );
-                    *current = PoolVersion::of_a_version_of_trees(published);
+                    *current = PoolVersion::of_a_shared_version_of_trees(published);
                     let PublishedInThisChange {
                         floor_raises,
                         refusals_that_pushed_the_floor_raises,
@@ -3331,7 +3335,7 @@ impl MountedSession {
         }
         let start = match &*current {
             PoolVersion::WithFile(version_of_trees)
-            | PoolVersion::WithTreesAndNoFile(version_of_trees) => version_of_trees.clone(),
+            | PoolVersion::WithTreesAndNoFile(version_of_trees) => Arc::clone(version_of_trees),
             PoolVersion::WithoutFile(_) => {
                 return Err(RootRingPushRefused::VersionWithoutFileNotSupportedInTheFirstVersion);
             }
@@ -3362,14 +3366,15 @@ impl MountedSession {
         // 预演：迭代上界是这一串的次数；跨轮携带的是预演出来的现行那一版（下一次的计划接着它）与分配器的拷贝。
         {
             let rehearsal_writer = PoolWriter::new(parameters, devices.as_mut_slice());
+            // 分配器的拷贝是预演要就地改的那一份，照拷；预演起点的那一版只读，与 `start` 指着同一份。
             let mut copy = allocator.clone();
-            let mut rehearsed = start.clone();
+            let mut rehearsed = Arc::clone(&start);
             for publish_index in 0..publishes_in_the_push {
                 let (_, next) = prepare_the_version_publish(
                     &rehearsal_writer,
                     &mut copy,
                     &empty_publish_plan_after_file_version(&rehearsed, instance),
-                    Some(&rehearsed),
+                    Some(Arc::as_ref(&rehearsed)),
                     rehearsed.tree_identifiers,
                     ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
                     UnmountMarker::NotWrittenByTheUnmountSequence,
@@ -3382,7 +3387,7 @@ impl MountedSession {
                         cause,
                     }
                 })?;
-                rehearsed = next;
+                rehearsed = Arc::new(next);
             }
         }
         // 真发：迭代上界同上；跨轮携带的是落盘了的现行那一版与发成的每一版；出口只有发不出去那一次。
@@ -3394,14 +3399,15 @@ impl MountedSession {
                 &mut writer,
                 allocator,
                 empty_publish_plan_after_file_version(&last_published, instance),
-                Some(&last_published),
+                Some(Arc::as_ref(&last_published)),
             ) {
                 Ok(next) => {
-                    versions_published.push(next.clone());
+                    let next = Arc::new(next);
+                    versions_published.push(Arc::clone(&next));
                     last_published = next;
                 }
                 Err(cause) => {
-                    *current = PoolVersion::of_a_version_of_trees(last_published);
+                    *current = PoolVersion::of_a_shared_version_of_trees(last_published);
                     return Err(RootRingPushRefused::PublishFailedAfterTheRehearsal(
                         Box::new(RootRingPushFailedAfterTheRehearsal {
                             failed_publish_in_the_push: publish_index + 1,
@@ -3413,7 +3419,7 @@ impl MountedSession {
                 }
             }
         }
-        *current = PoolVersion::of_a_version_of_trees(last_published);
+        *current = PoolVersion::of_a_shared_version_of_trees(last_published);
         let still_holding = allocator
             .root_ring_occupancy()
             .expect("推环之前读到了 C 那一圈，装着根环表")
@@ -3765,10 +3771,11 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
         PoolVersion::WithoutFile(_) => true,
     };
     if !queue_is_empty {
-        let mut walked = current
-            .version_of_trees()
-            .expect("队列不空的一版是有树的一版")
-            .clone();
+        let mut walked = Arc::clone(
+            current
+                .shared_version_of_trees()
+                .expect("队列不空的一版是有树的一版"),
+        );
         let walk = publish_the_queue_until_it_is_empty(
             parameters,
             devices,
@@ -3776,8 +3783,9 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
             &mut walked,
             instance,
         );
-        *current = PoolVersion::of_a_version_of_trees(walked);
-        versions_published.extend(walk.steps_published);
+        *current = PoolVersion::of_a_shared_version_of_trees(walked);
+        // 整理这一趟的版本列表按值交回（[`CompactionRun::versions_published`]）：走空的各步只有最后一步还与 `current` 共享，拷那一份。
+        versions_published.extend(walk.steps_published.into_iter().map(Arc::unwrap_or_clone));
         if let Err(cause) = walk.stopped_by {
             return finish(
                 compaction,
@@ -5035,7 +5043,8 @@ struct PublishedInThisChange {
 /// `mount::push_one_floor_raise_within_the_admission_budget`），推成交回记上这一串之后的 `published_in_this_change` 与这一串推了几次空发布，
 /// 不再推交回 `NoSpaceAfterRaisingTheFloor`（报不报「被宽护卡住」，报的话记下这次改动剩下的 `remainder`，要出路时照它预演，
 /// [`wide_protection_at_the_ceiling`]），抬 F 自己报错交回 `FloorRaiseFailedWhilePushingForSpace`。
-/// 推的那一串就地推进 `current`（被拒的那一次要接在它后面的那一版）。
+/// 推的那一串就地推进 `current`（被拒的那一次要接在它后面的那一版）：`current` 还有别人指着（版本列表里那一份）时先拷一份再推
+/// （[`Arc::make_mut`]），版本列表里那一份不动。
 #[allow(
     clippy::too_many_arguments,
     reason = "参数各有名字与类型：写入口要的参数与盘表、分配器、现行那一版、影子账那一臂、这次改动还剩什么、预算已用的数、已落盘的、这一次被拒，不成一个概念"
@@ -5044,7 +5053,7 @@ fn push_one_floor_raise_for_a_refusal_for_space<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
     allocator: &mut PoolAllocator,
-    current: &mut TransactionOutput,
+    current: &mut Arc<TransactionOutput>,
     shadow_ledger: ShadowLedger,
     remainder: RemainderOfTheUserChange<'_>,
     publishes_pushed_in_this_admission: usize,
@@ -5067,7 +5076,7 @@ fn push_one_floor_raise_for_a_refusal_for_space<Device: BlockDevice>(
         parameters,
         devices,
         allocator,
-        current,
+        Arc::make_mut(current),
         shadow_ledger,
         publishes_pushed_in_this_admission,
     ) {
@@ -5124,7 +5133,7 @@ fn publish_the_queue_until_it_is_empty_raising_the_floor_for_space<Device: Block
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
     allocator: &mut PoolAllocator,
-    current: &mut TransactionOutput,
+    current: &mut Arc<TransactionOutput>,
     instance: InstanceGeneration,
     shadow_ledger: ShadowLedger,
     remainder: RemainderOfTheUserChange<'_>,
```

### crates/singlefs-core/src/recovery.rs

```diff
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 3f95d5c..6f90442 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -10,6 +10,7 @@
 
 use std::cell::{Cell, OnceCell, RefCell};
 use std::collections::{BTreeMap, BTreeSet};
+use std::sync::Arc;
 
 use singlefs_format::{
     ACCOUNTING_ENTRY_BYTES, CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES,
@@ -4774,7 +4775,7 @@ fn units_of_the_version(
         |identity: TransactionUnit, locations: &[LocationEntry; 2], bytes: Vec<u8>| PublishedUnit {
             slot: locations[0].slot,
             identity,
-            bytes,
+            bytes: Arc::from(bytes),
         };
     let TreesOfTheVersionRead {
         first_file_extents,
```

### crates/singlefs-core/src/transaction.rs

```diff
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 111bcc6..c1bfcbb 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -11,6 +11,7 @@
 
 use std::cell::{Cell, RefCell};
 use std::collections::{BTreeMap, BTreeSet};
+use std::sync::Arc;
 
 use singlefs_format::{
     ACCOUNTING_ENTRY_BYTES, DELETED_INODES_ENTRY_BYTES, EXTENT_KEY_BYTES, INODE_INTERNAL_ENTRY,
@@ -2350,7 +2351,7 @@ pub fn prepare_the_row_publish_on_a_version_without_file<Device: BlockDevice>(
             PublishedUnit {
                 slot: slots[identity],
                 identity: *identity,
-                bytes: page.bytes.clone(),
+                bytes: Arc::from(page.bytes.as_slice()),
             }
         })
         .chain(allocation_built.rewritten_units.iter().cloned())
@@ -2421,17 +2422,21 @@ fn allocate_placement_for_role(
 /// 池里现行的那一版：还没发布过文件版本（树表 0 条，零单元发布或写行那次发布写出的根）、带文件的一版，
 /// 或有树而文件集合为空的一版（D8（核心索引结构） 已定项 16 第 5 条，判定见 [`TransactionOutput::is_a_version_with_trees_and_no_file`]）。
 /// 有树的两种由 [`PoolVersion::of_a_version_of_trees`] 按同一个判定分，不由调用方挑。
+/// 有树的一版按引用计数共享（判决 `research/prompts/m4-gpugc-publish-growth-r1-main-verification.md` 第二节 Q2.2「版本列表与挂载交回值共享同一份」）：
+/// 会话与挂载的现行那一版、交回给调用方的版本列表（[`DeletionQueueWalk::steps_published`]、`mounted_session::VersionsPublishedInThisChange`、
+/// `mount::MountOutput` 里的那几版）指着同一份 [`TransactionOutput`]，不再各拷一整版。一版发出来之后不再改：要改的地方经 [`Arc::make_mut`]
+/// 改一份自己的（还有别人指着时先拷一份，没有时就地改），别人指着的那一份不动。相等照内容比，与共不共享无关。
 #[derive(Clone, Debug, PartialEq, Eq)]
 #[allow(
     clippy::large_enum_variant,
-    reason = "一次挂载只有几个这样的值、不进集合，两个成员差几百字节按值搬无所谓；装箱只多一层解引用"
+    reason = "一次挂载只有几个这样的值、不进集合，树表 0 条那一版按值搬无所谓；装箱只多一层解引用"
 )]
 pub enum PoolVersion {
     WithoutFile(VersionWithoutFilePublishOutput),
-    WithFile(TransactionOutput),
+    WithFile(Arc<TransactionOutput>),
     /// 有树、文件集合为空（D8（核心索引结构） 已定项 16 第 5 条，不借 `WithoutFile`）：接在它后面的只有 `publish_new_inodes`（size 0）、
     /// 写行与空发布（K-U5 X1），会话、挂载按这个成员分流。
-    WithTreesAndNoFile(TransactionOutput),
+    WithTreesAndNoFile(Arc<TransactionOutput>),
 }
 
 impl PoolVersion {
@@ -2439,6 +2444,11 @@ impl PoolVersion {
     /// `WithTreesAndNoFile`，否则 `WithFile`。
     #[must_use]
     pub fn of_a_version_of_trees(version: TransactionOutput) -> PoolVersion {
+        PoolVersion::of_a_shared_version_of_trees(Arc::new(version))
+    }
+    /// 同 [`Self::of_a_version_of_trees`]，收的是已经共享着的一版（版本列表里那一份）：成员里指着同一份，不拷。
+    #[must_use]
+    pub fn of_a_shared_version_of_trees(version: Arc<TransactionOutput>) -> PoolVersion {
         if version.is_a_version_with_trees_and_no_file() {
             PoolVersion::WithTreesAndNoFile(version)
         } else {
@@ -2477,19 +2487,30 @@ impl PoolVersion {
     pub fn file_version(&self) -> Option<&TransactionOutput> {
         match self {
             PoolVersion::WithoutFile(_) | PoolVersion::WithTreesAndNoFile(_) => None,
-            PoolVersion::WithFile(version) => Some(version),
+            PoolVersion::WithFile(version) => Some(Arc::as_ref(version)),
         }
     }
+    /// 同 [`Self::file_version`]，交回自己的一份：没有别人指着就原样交出，还有别人指着（版本列表里那一份）就拷一份。
     #[must_use]
     pub fn into_file_version(self) -> Option<TransactionOutput> {
         match self {
             PoolVersion::WithoutFile(_) | PoolVersion::WithTreesAndNoFile(_) => None,
-            PoolVersion::WithFile(version) => Some(version),
+            PoolVersion::WithFile(version) => Some(Arc::unwrap_or_clone(version)),
         }
     }
     /// 有树的那一版（带文件的、有树而文件集合为空的）；树表 0 条时 `None`。
     #[must_use]
     pub fn version_of_trees(&self) -> Option<&TransactionOutput> {
+        match self {
+            PoolVersion::WithoutFile(_) => None,
+            PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
+                Some(Arc::as_ref(version))
+            }
+        }
+    }
+    /// 同 [`Self::version_of_trees`]，交回共享着的那一份本身（不拷）：放进版本列表、与现行那一版指着同一份时用。
+    #[must_use]
+    pub fn shared_version_of_trees(&self) -> Option<&Arc<TransactionOutput>> {
         match self {
             PoolVersion::WithoutFile(_) => None,
             PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
@@ -2497,8 +2518,15 @@ impl PoolVersion {
             }
         }
     }
+    /// 同 [`Self::into_file_version`] 的有树那两种。
     #[must_use]
     pub fn into_version_of_trees(self) -> Option<TransactionOutput> {
+        self.into_shared_version_of_trees()
+            .map(Arc::unwrap_or_clone)
+    }
+    /// 同 [`Self::shared_version_of_trees`]，交出共享着的那一份本身（不拷）。
+    #[must_use]
+    pub fn into_shared_version_of_trees(self) -> Option<Arc<TransactionOutput>> {
         match self {
             PoolVersion::WithoutFile(_) => None,
             PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
@@ -2530,7 +2558,7 @@ impl PoolVersion {
         match &mut self {
             PoolVersion::WithoutFile(version) => version.writes = writes,
             PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
-                version.writes = writes;
+                Arc::make_mut(version).writes = writes;
             }
         }
         self
@@ -3206,7 +3234,9 @@ impl FileVersionTreeIdentifiers {
 pub struct PublishedUnit {
     pub slot: SlotNumber,
     pub identity: TransactionUnit,
-    pub bytes: Vec<u8>,
+    /// 字节不可变、按引用计数共享：没重写的角色从上一版照抄（`carried_unit`）、版本整份拷贝时只多一个引用，不再复制这份字节
+    /// （判决 `research/prompts/m4-gpugc-publish-growth-r1-main-verification.md` 第二节 Q2.1 甲档）。相等照字节比，与共不共享无关。
+    pub bytes: Arc<[u8]>,
 }
 
 /// 一次发布写出的一条 journal 记录与它的 4096 字节。
@@ -11143,7 +11173,7 @@ fn relocation_checked_against_the_version<Reader: PoolReader + ?Sized>(
             .central_mapping
             .unit_of_role(role)
             .bytes
-            .clone(),
+            .to_vec(),
         RelocatedUnitHolder::KeptForTheSnapshots => {
             let unit_bytes = usize::try_from(planned.source.span * SLOT_BYTES)
                 .expect("一个单元两槽以内，字节数装得进 usize");
@@ -12232,7 +12262,7 @@ pub(crate) fn rehearse_every_step_of_the_deletion_queue_walk<Device: BlockDevice
 pub fn publish_unlink_steps_until_the_queue_is_empty<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
-    current: &mut TransactionOutput,
+    current: &mut Arc<TransactionOutput>,
     batch_size: UnlinkBatchSize,
     instance: InstanceGeneration,
 ) -> Result<usize, PublishError> {
@@ -12241,20 +12271,20 @@ pub fn publish_unlink_steps_until_the_queue_is_empty<Device: BlockDevice>(
     walk.stopped_by.map(|()| walk.steps_published.len())
 }
 
-/// 走空一趟的结局（[`walk_the_deletion_queue_until_it_is_empty`]）：发成的每一步（按先后，最后一个就是 `current`），
+/// 走空一趟的结局（[`walk_the_deletion_queue_until_it_is_empty`]）：发成的每一步（按先后，最后一个就是 `current`，两处指着同一份），
 /// 与这一趟停在哪：走空了是 `Ok`，一步没发成是那一步的错（已经发成的那几步留在盘上）。
 #[derive(Debug)]
 pub struct DeletionQueueWalk {
-    pub steps_published: Vec<TransactionOutput>,
+    pub steps_published: Vec<Arc<TransactionOutput>>,
     pub stopped_by: Result<(), PublishError>,
 }
 
 /// 同 [`publish_unlink_steps_until_the_queue_is_empty`]，连同发成的每一步交回（会话与挂载续做要把它们交给调用方，
-/// 模型对拍逐次比它们）。
+/// 模型对拍逐次比它们）。每发成一步，`current` 与交回的那一步指着同一份（不拷一整版）。
 pub fn walk_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
-    current: &mut TransactionOutput,
+    current: &mut Arc<TransactionOutput>,
     batch_size: UnlinkBatchSize,
     instance: InstanceGeneration,
 ) -> DeletionQueueWalk {
@@ -12286,8 +12316,8 @@ pub fn walk_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
         };
         match next_step {
             Ok(next) => {
-                *current = next;
-                steps_published.push(current.clone());
+                *current = Arc::new(next);
+                steps_published.push(Arc::clone(current));
             }
             Err(refusal) => {
                 return DeletionQueueWalk {
@@ -13575,8 +13605,8 @@ fn carried_unit(
 /// （`carried_file_version_units`）。extent 树的节点不在这里——它们按这些指针由 `build_extent_tree` 装；
 /// 这个对象的 inode 记录也不在这里——它进 inode 树，由 `build_inode_tree_units` 按叶容器装。
 struct FileVersionUnits {
-    /// 文件的数据单元，第 i 项是文件第 i 个单元（与 `data_pointers` 同序）。
-    data_units: Vec<Vec<u8>>,
+    /// 文件的数据单元，第 i 项是文件第 i 个单元（与 `data_pointers` 同序）。照抄的那几个与上一版共用同一份字节（`carried_file_version_units`）。
+    data_units: Vec<Arc<[u8]>>,
     /// 指向每个数据单元的指针，第 i 项是文件第 i 个单元；extent 树下段的叶记录（或内联的那一条）按同一个次序装（key 升序）。
     data_pointers: Vec<DataPointer>,
     /// C319（请求内单元按 key 升序发出没有条款也没有检查）的运行时计数；照抄的一版恒 0。
@@ -13660,7 +13690,7 @@ fn build_file_version_units(
                 .try_into()
                 .expect("24"),
         );
-        data_units.push(data_unit);
+        data_units.push(Arc::from(data_unit));
         data_pointers.push(data_pointer);
     }
     let key_order_mismatches = count_key_order_mismatches(&extent_keys);
@@ -13754,7 +13784,7 @@ fn build_extent_tree(
         units_so_far.push(PublishedUnit {
             slot,
             identity: role,
-            bytes,
+            bytes: Arc::from(bytes),
         });
         pointer
     };
@@ -14127,7 +14157,7 @@ fn build_allocation_record_tree(
                 rewritten_units.push(PublishedUnit {
                     slot,
                     identity: role,
-                    bytes,
+                    bytes: Arc::from(bytes),
                 });
                 pointer
             }
@@ -15042,7 +15072,7 @@ fn publish_admitted<Device: BlockDevice>(
             PublishedUnit {
                 slot: destination.slot,
                 identity: relocation.role_for_the_checks,
-                bytes: relocation.bytes.clone(),
+                bytes: Arc::from(relocation.bytes.as_slice()),
             },
             NamedUnit {
                 locations: destination_locations,
@@ -15235,7 +15265,7 @@ fn publish_admitted<Device: BlockDevice>(
             let position = usize::try_from(transaction.unit_index_in_file.0).expect("单元序号");
             if position < data_pointers.len() {
                 data_pointers[position] = pointer;
-                data_units[position] = data_unit;
+                data_units[position] = Arc::from(data_unit);
             } else {
                 assert_eq!(
                     position,
@@ -15243,7 +15273,7 @@ fn publish_admitted<Device: BlockDevice>(
                     "部分写新加的单元从上一版的单元数起连号接在后面"
                 );
                 data_pointers.push(pointer);
-                data_units.push(data_unit);
+                data_units.push(Arc::from(data_unit));
             }
         }
     }
@@ -16265,7 +16295,7 @@ fn publish_admitted<Device: BlockDevice>(
 
     // 这一版全部角色的单元：重写的是这次装的，照抄的从上一版拷（按 bump 次序：每个数据单元、extent 根、每片 inode 叶容器、
     // inode 根、分配记录树、记账树与中央映射树的每个节点（树内先叶后根）、树表，实例表链的各片在末尾）。
-    let rewritten_unit = |identity: TransactionUnit, bytes: Vec<u8>| PublishedUnit {
+    let rewritten_unit = |identity: TransactionUnit, bytes: Arc<[u8]>| PublishedUnit {
         slot: slot_of(identity),
         identity,
         bytes,
@@ -16308,7 +16338,7 @@ fn publish_admitted<Device: BlockDevice>(
                     identity,
                 )
             }
-            None | Some(_) => rewritten_unit(identity, data_unit.clone()),
+            None | Some(_) => rewritten_unit(identity, Arc::clone(data_unit)),
         });
     }
     // 第一个文件之外各文件的数据单元（按 inode 号、单元序号）：这次写的那一个是这次装的，别的从上一版照抄。
@@ -16322,11 +16352,12 @@ fn publish_admitted<Device: BlockDevice>(
                         .expect("计划里整份重写的那个文件就是这次写的那一个");
                     rewritten_unit(
                         identity,
-                        written_units
-                            .data_units
-                            .get(usize::try_from(position).expect("单元序号"))
-                            .expect("这次写的那个文件的单元数就是计划里的单元数")
-                            .clone(),
+                        Arc::clone(
+                            written_units
+                                .data_units
+                                .get(usize::try_from(position).expect("单元序号"))
+                                .expect("这次写的那个文件的单元数就是计划里的单元数"),
+                        ),
                     )
                 }
                 FileOtherThanTheFirstOrigin::SizeChangedThisPublish { .. }
@@ -16336,14 +16367,16 @@ fn publish_admitted<Device: BlockDevice>(
                 {
                     rewritten_unit(
                         identity,
-                        rewritten_data_unit_of_a_file_other_than_the_first(
-                            &data_units_of_a_file_other_than_the_first_rewritten,
-                            file_plan.inode,
-                            DataUnitIndexInFile(position),
-                        )
-                        .expect("计划里重写的单元就是这次装的那一段里的（同一份计划拼的）")
-                        .bytes
-                        .clone(),
+                        Arc::from(
+                            rewritten_data_unit_of_a_file_other_than_the_first(
+                                &data_units_of_a_file_other_than_the_first_rewritten,
+                                file_plan.inode,
+                                DataUnitIndexInFile(position),
+                            )
+                            .expect("计划里重写的单元就是这次装的那一段里的（同一份计划拼的）")
+                            .bytes
+                            .as_slice(),
+                        ),
                     )
                 }
                 FileOtherThanTheFirstOrigin::CarriedFromThePreviousVersion
@@ -16362,7 +16395,7 @@ fn publish_admitted<Device: BlockDevice>(
         let identity = TransactionUnit::InodeLeafContainer(container.index);
         units.push(match container.origin {
             InodeLeafContainerOrigin::RewrittenThisPublish => {
-                rewritten_unit(identity, container.bytes.clone())
+                rewritten_unit(identity, Arc::from(container.bytes.as_slice()))
             }
             // 照抄上一版那个叶序上的那一份（字节、落点不动），装进这一版时换成这一版的角色。
             InodeLeafContainerOrigin::CarriedFrom(previous_index) => {
@@ -16381,7 +16414,7 @@ fn publish_admitted<Device: BlockDevice>(
         });
     }
     units.push(match &inode_tree_units.rewritten_root {
-        Some(root) => rewritten_unit(TransactionUnit::InodeRoot, root.bytes.clone()),
+        Some(root) => rewritten_unit(TransactionUnit::InodeRoot, Arc::from(root.bytes.as_slice())),
         None => carried_unit(
             units_of_the_previous_version
                 .as_ref()
@@ -16412,7 +16445,9 @@ fn publish_admitted<Device: BlockDevice>(
     units.extend(deadlist_built.units.iter().cloned());
     let deletion_tree_unit =
         |identity: TransactionUnit, node: &DeletionTreeNodeUnit| match &node.rewritten {
-            Some(rewritten_node) => rewritten_unit(identity, rewritten_node.bytes.clone()),
+            Some(rewritten_node) => {
+                rewritten_unit(identity, Arc::from(rewritten_node.bytes.as_slice()))
+            }
             None => carried_unit(
                 units_of_the_previous_version
                     .as_ref()
@@ -16426,7 +16461,9 @@ fn publish_admitted<Device: BlockDevice>(
     // 容器树根之下的节点照抄时按它在上一版里的角色拷（长高那次的旧根上一版是根），装进这一版时换成这一版的角色。
     let container_tree_node_below_the_root_unit =
         |node: &ContainerTreeNodeBelowTheRootUnit| match &node.node.rewritten {
-            Some(rewritten_node) => rewritten_unit(node.identity, rewritten_node.bytes.clone()),
+            Some(rewritten_node) => {
+                rewritten_unit(node.identity, Arc::from(rewritten_node.bytes.as_slice()))
+            }
             None => {
                 let carried = carried_unit(
                     units_of_the_previous_version
@@ -16486,13 +16523,13 @@ fn publish_admitted<Device: BlockDevice>(
     units.extend(mapping_built.units.iter().cloned());
     units.push(rewritten_unit(
         TransactionUnit::TreeTable,
-        tree_table_unit.clone(),
+        Arc::from(tree_table_unit.as_slice()),
     ));
     // 实例表链上的各片按链上的次序（第 0 片在前）：重写的是这次装的，照抄的是上一版带着的那几片。
     match (&plan.instance_table, previous) {
         (InstanceTablePlan::Rewrite(_), _) => {
             for page in &instance_table_rewrite {
-                units.push(rewritten_unit(page.role, page.bytes.clone()));
+                units.push(rewritten_unit(page.role, Arc::from(page.bytes.as_slice())));
             }
         }
         (InstanceTablePlan::Carry(_), Some(previous_version)) => {
@@ -17161,7 +17198,7 @@ fn build_multi_level_tree(
                     PublishedUnit {
                         slot,
                         identity,
-                        bytes,
+                        bytes: Arc::from(bytes),
                     },
                 )
             }
```

## 第 88 批，补丁目录 ~/.local/share/singlefs/work/m4-gpugc-name-collisions-writer/patch/

### crates/singlefs-background-gpu/src/adapter_inventory.rs

```diff
diff --git a/crates/singlefs-background-gpu/src/adapter_inventory.rs b/crates/singlefs-background-gpu/src/adapter_inventory.rs
index 3656539..2ada40a 100644
--- a/crates/singlefs-background-gpu/src/adapter_inventory.rs
+++ b/crates/singlefs-background-gpu/src/adapter_inventory.rs
@@ -332,12 +332,16 @@ pub fn participating_gpus_admitted_by_the_mount_checks(
     Ok(participating)
 }
 
+/// 只在测试里用：本模块与 [`crate::gpus_of_this_host`] 的测试写卡的显存都按这一份。
+#[cfg(test)]
+pub(crate) const GIBIBYTE: u64 = 1 << 30;
+
 #[cfg(test)]
 mod tests {
     use super::*;
 
-    const GIBIBYTE: u64 = 1 << 30;
-    const WAIT_LIMIT_UNDER_TEST: std::time::Duration = std::time::Duration::from_secs(5);
+    const WAIT_LIMIT_IN_THE_SETTINGS_UNDER_TEST: std::time::Duration =
+        std::time::Duration::from_secs(5);
 
     fn adapter(
         name: &str,
@@ -545,7 +549,7 @@ mod tests {
         assert_eq!(
             participating_gpus_admitted_by_the_mount_checks(
                 &BackgroundGpuSelection {
-                    background_gpu_wait_limit: WAIT_LIMIT_UNDER_TEST,
+                    background_gpu_wait_limit: WAIT_LIMIT_IN_THE_SETTINGS_UNDER_TEST,
                     background_gpu_devices: None,
                     background_gpu_memory_limit: None,
                 },
@@ -562,7 +566,7 @@ mod tests {
         assert_eq!(
             participating_gpus_admitted_by_the_mount_checks(
                 &BackgroundGpuSelection {
-                    background_gpu_wait_limit: WAIT_LIMIT_UNDER_TEST,
+                    background_gpu_wait_limit: WAIT_LIMIT_IN_THE_SETTINGS_UNDER_TEST,
                     background_gpu_devices: Some(vec![integrated_address.clone()]),
                     background_gpu_memory_limit: None,
                 },
@@ -582,7 +586,7 @@ mod tests {
         assert_eq!(
             participating_gpus_admitted_by_the_mount_checks(
                 &BackgroundGpuSelection {
-                    background_gpu_wait_limit: WAIT_LIMIT_UNDER_TEST,
+                    background_gpu_wait_limit: WAIT_LIMIT_IN_THE_SETTINGS_UNDER_TEST,
                     background_gpu_devices: None,
                     background_gpu_memory_limit: Some(vec![GpuMemoryInBytes(GIBIBYTE)]),
                 },
@@ -603,7 +607,7 @@ mod tests {
         assert_eq!(
             participating_gpus_admitted_by_the_mount_checks(
                 &BackgroundGpuSelection {
-                    background_gpu_wait_limit: WAIT_LIMIT_UNDER_TEST,
+                    background_gpu_wait_limit: WAIT_LIMIT_IN_THE_SETTINGS_UNDER_TEST,
                     background_gpu_devices: None,
                     background_gpu_memory_limit: Some(vec![
                         GpuMemoryInBytes(GIBIBYTE),
@@ -627,7 +631,7 @@ mod tests {
     fn named_gpus_participate_in_the_named_order_with_their_memory_limits() {
         let participating = participating_gpus_admitted_by_the_mount_checks(
             &BackgroundGpuSelection {
-                background_gpu_wait_limit: WAIT_LIMIT_UNDER_TEST,
+                background_gpu_wait_limit: WAIT_LIMIT_IN_THE_SETTINGS_UNDER_TEST,
                 background_gpu_devices: Some(vec![
                     gpu_device_identifier_of_the_pci_bus_address("0000:41:00.0"),
                     gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0"),
@@ -662,7 +666,7 @@ mod tests {
                 "0000:41:00.0",
             )]),
             background_gpu_memory_limit: Some(vec![GpuMemoryInBytes(2 * GIBIBYTE)]),
-            background_gpu_wait_limit: WAIT_LIMIT_UNDER_TEST,
+            background_gpu_wait_limit: WAIT_LIMIT_IN_THE_SETTINGS_UNDER_TEST,
         };
         assert_eq!(
             BackgroundGpuSelection::of_the_settings(&settings),
@@ -671,7 +675,7 @@ mod tests {
                     "0000:41:00.0"
                 )]),
                 background_gpu_memory_limit: Some(vec![GpuMemoryInBytes(2 * GIBIBYTE)]),
-                background_gpu_wait_limit: WAIT_LIMIT_UNDER_TEST,
+                background_gpu_wait_limit: WAIT_LIMIT_IN_THE_SETTINGS_UNDER_TEST,
             },
             "开卡照挂载那一组的卡、上限与等 GPU 的上限"
         );
@@ -681,7 +685,7 @@ mod tests {
     fn without_named_gpus_every_discrete_gpu_participates_without_a_memory_limit() {
         let participating = participating_gpus_admitted_by_the_mount_checks(
             &BackgroundGpuSelection {
-                background_gpu_wait_limit: WAIT_LIMIT_UNDER_TEST,
+                background_gpu_wait_limit: WAIT_LIMIT_IN_THE_SETTINGS_UNDER_TEST,
                 background_gpu_devices: None,
                 background_gpu_memory_limit: None,
             },
```

### crates/singlefs-background-gpu/src/background_gpu_compute.rs

```diff
diff --git a/crates/singlefs-background-gpu/src/background_gpu_compute.rs b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
index 2db21f2..691c917 100644
--- a/crates/singlefs-background-gpu/src/background_gpu_compute.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
@@ -606,7 +606,7 @@ mod tests {
     use crate::checksum_kernels::UnrecognizedVerdictWord;
     use crate::region_record_kernels::MappingKeyLongerThanTheKernelsHold;
 
-    fn card() -> GpuDeviceIdentifier {
+    fn card_named_in_the_failures() -> GpuDeviceIdentifier {
         GpuDeviceIdentifier::of_opaque_bytes(b"0000:01:00.0".to_vec())
     }
 
@@ -616,7 +616,7 @@ mod tests {
         let wait_limit = Duration::from_secs(10);
         let failed_on_the_card =
             |cause: GpuFailureOnTheDevice| BackgroundGpuComputeFailed::GpuFailed {
-                device: card(),
+                device: card_named_in_the_failures(),
                 cause,
             };
         vec![
@@ -647,7 +647,7 @@ mod tests {
             (
                 GpuFailureCause::WaitForTheGpuExceededTheLimit { wait_limit },
                 BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit {
-                    device: card(),
+                    device: card_named_in_the_failures(),
                     wait_limit,
                     submission_waited_for: SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
                 },
@@ -657,7 +657,7 @@ mod tests {
                     wait_limit,
                 },
                 BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit {
-                    device: card(),
+                    device: card_named_in_the_failures(),
                     wait_limit,
                     submission_waited_for:
                         SubmissionWaitedForPastTheLimit::EarlierTimedOutSubmissionStillRunning,
@@ -700,7 +700,7 @@ mod tests {
         for (cause, expected) in every_gpu_failure_cause_and_its_core_failure() {
             assert_eq!(
                 core_failure_of_the_checksum_batch(&ChecksumBatchFailed::GpuFailed {
-                    device: card(),
+                    device: card_named_in_the_failures(),
                     cause: cause.clone(),
                 }),
                 expected,
@@ -708,7 +708,7 @@ mod tests {
             );
             assert_eq!(
                 core_failure_of_the_region_record_batch(&RegionRecordBatchFailed::GpuFailed {
-                    device: card(),
+                    device: card_named_in_the_failures(),
                     cause: cause.clone(),
                 }),
                 expected,
@@ -716,7 +716,7 @@ mod tests {
             );
             assert_eq!(
                 core_failure_of_the_live_slot_batch(&LiveSlotBatchFailed::GpuFailed {
-                    device: card(),
+                    device: card_named_in_the_failures(),
                     cause: cause.clone(),
                 }),
                 expected,
@@ -729,14 +729,14 @@ mod tests {
     #[test]
     fn inputs_the_gpu_cannot_take_are_reported_with_their_card_and_entry() {
         let smallest_piece_not_held = SmallestPieceNotHeldOnTheGpu {
-            device: card(),
+            device: card_named_in_the_failures(),
             entry_position: 3,
             memory_limit: Some(GpuMemoryInBytes(50)),
             smallest_piece_bytes: 104,
         };
         let reported = BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
             InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice {
-                device: card(),
+                device: card_named_in_the_failures(),
                 entry_position: 3,
                 input_bytes: 104,
             },
@@ -808,14 +808,14 @@ mod tests {
                 last_cause: Box::new(last_cause),
             };
             let expected = BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions {
-                device: card(),
+                device: card_named_in_the_failures(),
                 resubmissions: 3,
                 wait_limit,
                 last_submission_waited_for,
             };
             assert_eq!(
                 core_failure_of_the_checksum_batch(&ChecksumBatchFailed::GpuFailed {
-                    device: card(),
+                    device: card_named_in_the_failures(),
                     cause: unusable.clone(),
                 }),
                 expected,
@@ -823,7 +823,7 @@ mod tests {
             );
             assert_eq!(
                 core_failure_of_the_region_record_batch(&RegionRecordBatchFailed::GpuFailed {
-                    device: card(),
+                    device: card_named_in_the_failures(),
                     cause: unusable.clone(),
                 }),
                 expected,
@@ -831,7 +831,7 @@ mod tests {
             );
             assert_eq!(
                 core_failure_of_the_live_slot_batch(&LiveSlotBatchFailed::GpuFailed {
-                    device: card(),
+                    device: card_named_in_the_failures(),
                     cause: unusable,
                 }),
                 expected,
@@ -848,7 +848,7 @@ mod tests {
     ) {
         let gpus_left_out = vec![
             GpuLeftOutUnderItsMemoryLimit {
-                device: card(),
+                device: card_named_in_the_failures(),
                 memory_limit: GpuMemoryInBytes(50),
                 smallest_piece_bytes: 976,
                 bytes_already_held_on_the_device: 0,
@@ -904,11 +904,11 @@ mod tests {
         assert_eq!(
             core_failure_of_the_batch_not_split_over_the_gpus(
                 &BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch {
-                    device: card(),
+                    device: card_named_in_the_failures(),
                     cause: cause.clone(),
                 }
             ),
-            core_failure_of_the_gpu(&card(), &cause),
+            core_failure_of_the_gpu(&card_named_in_the_failures(), &cause),
             "换设置时量代价出错照那张卡的 GPU 出错报"
         );
     }
@@ -920,7 +920,7 @@ mod tests {
     fn every_participating_gpu_short_of_free_memory_is_reported_with_each_card_short_and_those_left_out_under_their_limit(
     ) {
         let gpus_short_of_free_memory = vec![GpuShortOfFreeMemoryForTheSmallestPiece {
-            device: card(),
+            device: card_named_in_the_failures(),
             bytes_of_the_last_chunk_not_created: 64,
             smallest_piece_bytes: 64,
         }];
@@ -971,7 +971,7 @@ mod tests {
         assert_eq!(
             expected.gpu_devices_named(),
             vec![
-                card(),
+                card_named_in_the_failures(),
                 GpuDeviceIdentifier::of_opaque_bytes(b"0000:02:00.0".to_vec())
             ],
             "因空闲显存不用的在前、因显存上限不用的在后"
```

### crates/singlefs-background-gpu/src/checksum_kernels.rs

```diff
diff --git a/crates/singlefs-background-gpu/src/checksum_kernels.rs b/crates/singlefs-background-gpu/src/checksum_kernels.rs
index 4f87894..9205b69 100644
--- a/crates/singlefs-background-gpu/src/checksum_kernels.rs
+++ b/crates/singlefs-background-gpu/src/checksum_kernels.rs
@@ -575,7 +575,8 @@ mod tests {
     }
 
     #[test]
-    fn the_two_kernels_are_separate_sources_with_their_own_entry_points() {
+    fn the_table_driven_and_bitwise_checksum_kernels_are_separate_sources_with_their_own_entry_points(
+    ) {
         assert!(
             TABLE_DRIVEN_KERNEL_SOURCE.contains(&format!("fn {TABLE_DRIVEN_KERNEL_ENTRY_POINT}("))
         );
```

### crates/singlefs-background-gpu/src/gpus_of_this_host.rs

```diff
diff --git a/crates/singlefs-background-gpu/src/gpus_of_this_host.rs b/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
index 80476cc..899dbf4 100644
--- a/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
+++ b/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
@@ -953,13 +953,11 @@ mod tests {
 
     use crate::adapter_inventory::{
         discrete_gpus_of_the_enumeration, participating_gpus_admitted_by_the_mount_checks,
-        AdapterDeviceKind, BackgroundGpuSelection, EnumeratedAdapterDescription,
+        AdapterDeviceKind, BackgroundGpuSelection, EnumeratedAdapterDescription, GIBIBYTE,
     };
     use crate::background_gpu::COST_PROBE_CHUNK;
     use crate::memory_limit_and_chunks::SubmissionOfTheChunk;
 
-    const GIBIBYTE: u64 = 1 << 30;
-
     /// 本机三张独显，按总线地址排：代价各不相同。
     struct ThreeCardsOnThisHost;
 
```

### crates/singlefs-background-gpu/src/live_slot_kernels.rs

```diff
diff --git a/crates/singlefs-background-gpu/src/live_slot_kernels.rs b/crates/singlefs-background-gpu/src/live_slot_kernels.rs
index 32827f2..428d770 100644
--- a/crates/singlefs-background-gpu/src/live_slot_kernels.rs
+++ b/crates/singlefs-background-gpu/src/live_slot_kernels.rs
@@ -69,7 +69,8 @@ const RELEASED_WORD_OF_A_LIVE_RECORD: u32 = 0;
 /// 两份核各一个结果缓冲与一个回读暂存缓冲。
 const LIVE_SLOT_OUTCOME_AND_READBACK_BUFFERS: u64 = 4;
 
-fn low_and_high_words_of_the_slot(slot: SlotNumber) -> [u32; 2] {
+/// 区域扫描那一段（[`crate::region_record_kernels`]）打包槽号也用这一份：两段交给核的槽号字是同一种拆法。
+pub(crate) fn low_and_high_words_of_the_slot(slot: SlotNumber) -> [u32; 2] {
     [
         u32::try_from(slot.0 & u64::from(u32::MAX)).expect("取了低 32 位"),
         u32::try_from(slot.0 >> 32).expect("盘上槽号是 6 字节，高 32 位装得进 u32"),
@@ -603,7 +604,9 @@ mod tests {
         );
     }
 
-    fn outcome_bytes(outcome_words: [u32; LIVE_SLOT_OUTCOME_WORDS]) -> Vec<u8> {
+    fn live_slot_outcome_bytes_of_the_words(
+        outcome_words: [u32; LIVE_SLOT_OUTCOME_WORDS],
+    ) -> Vec<u8> {
         outcome_words
             .iter()
             .flat_map(|word| word.to_le_bytes())
@@ -612,8 +615,12 @@ mod tests {
 
     #[test]
     fn readback_records_decode_into_sixty_four_bit_live_slot_counts() {
-        let mut readback = outcome_bytes([64, 0, VERDICT_WORD_COUNTED]);
-        readback.extend(outcome_bytes([5, 2, VERDICT_WORD_COUNTED]));
+        let mut readback = live_slot_outcome_bytes_of_the_words([64, 0, VERDICT_WORD_COUNTED]);
+        readback.extend(live_slot_outcome_bytes_of_the_words([
+            5,
+            2,
+            VERDICT_WORD_COUNTED,
+        ]));
         assert_eq!(
             decoded_live_slot_outcomes(&readback),
             Ok(vec![64, (2_u64 << 32) | 5]),
@@ -622,8 +629,8 @@ mod tests {
     }
 
     #[test]
-    fn an_outcome_the_kernels_never_wrote_is_an_unrecognized_verdict_word() {
-        let mut readback = outcome_bytes([1, 0, VERDICT_WORD_COUNTED]);
+    fn live_slot_outcome_the_kernels_never_wrote_is_an_unrecognized_verdict_word() {
+        let mut readback = live_slot_outcome_bytes_of_the_words([1, 0, VERDICT_WORD_COUNTED]);
         readback.extend(live_slot_outcome_buffer_before_the_kernel_writes(1));
         assert_eq!(
             decoded_live_slot_outcomes(&readback),
@@ -739,7 +746,8 @@ mod tests {
     }
 
     #[test]
-    fn the_two_kernels_are_separate_sources_with_their_own_entry_points() {
+    fn the_walking_and_searching_live_slot_kernels_are_separate_sources_with_their_own_entry_points(
+    ) {
         assert!(WALKING_EVERY_RECORD_KERNEL_SOURCE
             .contains(&format!("fn {WALKING_EVERY_RECORD_KERNEL_ENTRY_POINT}(")));
         assert!(SEARCHING_SORTED_RECORDS_KERNEL_SOURCE.contains(&format!(
```

### crates/singlefs-background-gpu/src/region_record_kernels.rs

```diff
diff --git a/crates/singlefs-background-gpu/src/region_record_kernels.rs b/crates/singlefs-background-gpu/src/region_record_kernels.rs
index 62b5a15..b037bc3 100644
--- a/crates/singlefs-background-gpu/src/region_record_kernels.rs
+++ b/crates/singlefs-background-gpu/src/region_record_kernels.rs
@@ -63,6 +63,7 @@ use singlefs_core::unit::{
 };
 
 use crate::checksum_kernels::UnrecognizedVerdictWord;
+use crate::live_slot_kernels::low_and_high_words_of_the_slot;
 use crate::memory_limit_and_chunks::{
     chunks_that_fit_on_the_gpu, input_buffer_bytes, pieces_cut_to_fit_on_the_gpu,
     ChunkBoundsOfAGpu, ChunkFootprint, PieceOfAnEntry, SmallestPieceLargerThanTheRoom,
@@ -231,13 +232,6 @@ impl PackedRegionRecordChunk {
     }
 }
 
-fn low_and_high_words_of_the_slot(slot: SlotNumber) -> [u32; 2] {
-    [
-        u32::try_from(slot.0 & u64::from(u32::MAX)).expect("取了低 32 位"),
-        u32::try_from(slot.0 >> 32).expect("盘上槽号是 6 字节，高 32 位装得进 u32"),
-    ]
-}
-
 /// 把一段的几片按次序打包：每一片的字节首尾相接（起点是前面那些字节数之和，不按字对齐），读不到的占 0 字节；描述带上这一片是哪一种、
 /// 单元在它之后还有几个字节与两份核各自的寄存器（本模块文件头的布局）。
 ///
@@ -1414,7 +1408,7 @@ mod tests {
         })
     }
 
-    fn entry(key: Vec<u8>, locations: [LocationEntry; 2]) -> CentralMappingEntry {
+    fn central_mapping_entry(key: Vec<u8>, locations: [LocationEntry; 2]) -> CentralMappingEntry {
         CentralMappingEntry { key, locations }
     }
 
@@ -1637,7 +1631,7 @@ mod tests {
 
     #[test]
     fn mapping_tables_list_each_entry_with_its_key_length_and_both_locations() {
-        let tables = MappingTablesForTheKernels::of_the_mapping_entries(&[entry(
+        let tables = MappingTablesForTheKernels::of_the_mapping_entries(&[central_mapping_entry(
             vec![1, 2, 3, 4, 5],
             [
                 LocationEntry {
@@ -1670,9 +1664,9 @@ mod tests {
             unit_checksum: 0,
         }; 2];
         let tables = MappingTablesForTheKernels::of_the_mapping_entries(&[
-            entry(vec![1], locations_at(SlotNumber(90))),
-            entry(vec![2], same_location_twice),
-            entry(vec![3], locations_at(SlotNumber(70))),
+            central_mapping_entry(vec![1], locations_at(SlotNumber(90))),
+            central_mapping_entry(vec![2], same_location_twice),
+            central_mapping_entry(vec![3], locations_at(SlotNumber(70))),
         ])
         .expect("短 key 装得下");
         assert_eq!(
@@ -1692,10 +1686,10 @@ mod tests {
     #[test]
     fn the_key_index_is_sorted_by_key_length_and_words_with_ties_in_table_order() {
         let tables = MappingTablesForTheKernels::of_the_mapping_entries(&[
-            entry(vec![9, 9], locations_at(SlotNumber(1))),
-            entry(vec![1, 2, 3], locations_at(SlotNumber(2))),
-            entry(vec![5], locations_at(SlotNumber(3))),
-            entry(vec![9, 9], locations_at(SlotNumber(4))),
+            central_mapping_entry(vec![9, 9], locations_at(SlotNumber(1))),
+            central_mapping_entry(vec![1, 2, 3], locations_at(SlotNumber(2))),
+            central_mapping_entry(vec![5], locations_at(SlotNumber(3))),
+            central_mapping_entry(vec![9, 9], locations_at(SlotNumber(4))),
         ])
         .expect("短 key 装得下");
         assert_eq!(
@@ -1709,8 +1703,8 @@ mod tests {
     fn one_mapping_key_longer_than_the_kernels_hold_refuses_the_tables() {
         assert_eq!(
             MappingTablesForTheKernels::of_the_mapping_entries(&[
-                entry(vec![1; 28], locations_at(SlotNumber(1))),
-                entry(vec![1; 29], locations_at(SlotNumber(2))),
+                central_mapping_entry(vec![1; 28], locations_at(SlotNumber(1))),
+                central_mapping_entry(vec![1; 29], locations_at(SlotNumber(2))),
             ]),
             Err(MappingKeyLongerThanTheKernelsHold {
                 entry_position: 1,
@@ -1733,7 +1727,9 @@ mod tests {
         );
     }
 
-    fn outcome_bytes(outcome_words: [u32; REGION_RECORD_OUTCOME_WORDS]) -> Vec<u8> {
+    fn region_record_outcome_bytes_of_the_words(
+        outcome_words: [u32; REGION_RECORD_OUTCOME_WORDS],
+    ) -> Vec<u8> {
         little_endian_bytes_of_the_words(&outcome_words)
     }
 
@@ -1753,7 +1749,7 @@ mod tests {
         let mut readback = Vec::new();
         let [key_word_0, key_word_1, key_word_2, key_word_3, key_word_4, key_word_5, key_word_6] =
             movable_key_words;
-        readback.extend(outcome_bytes([
+        readback.extend(region_record_outcome_bytes_of_the_words([
             OUTCOME_KIND_MOVABLE,
             u32::from(UNIT_CLASS_DATA),
             0,
@@ -1768,7 +1764,7 @@ mod tests {
             key_word_5,
             key_word_6,
         ]));
-        readback.extend(outcome_bytes([
+        readback.extend(region_record_outcome_bytes_of_the_words([
             OUTCOME_KIND_DEAD,
             0,
             0,
@@ -1783,7 +1779,7 @@ mod tests {
             0,
             0,
         ]));
-        readback.extend(outcome_bytes([
+        readback.extend(region_record_outcome_bytes_of_the_words([
             OUTCOME_KIND_UNIT_CLASS_DISAGREES_WITH_THE_MAPPING_KEY,
             u32::from(UNIT_CLASS_PACKED),
             u32::from(UNIT_CLASS_INDEX_NODE),
@@ -1798,7 +1794,7 @@ mod tests {
             0,
             0,
         ]));
-        readback.extend(outcome_bytes([
+        readback.extend(region_record_outcome_bytes_of_the_words([
             OUTCOME_KIND_HEADER_KEY_DISAGREES_WITH_THE_MAPPING_LOCATION,
             0,
             0,
@@ -1813,7 +1809,7 @@ mod tests {
             0,
             0,
         ]));
-        readback.extend(outcome_bytes([
+        readback.extend(region_record_outcome_bytes_of_the_words([
             OUTCOME_KIND_DEVICE_READ_FAILED,
             0,
             0,
@@ -1828,7 +1824,7 @@ mod tests {
             0,
             0,
         ]));
-        readback.extend(outcome_bytes([
+        readback.extend(region_record_outcome_bytes_of_the_words([
             OUTCOME_KIND_SPAN_IS_NOT_A_UNIT_FOOTPRINT,
             3,
             0,
@@ -1843,7 +1839,7 @@ mod tests {
             0,
             0,
         ]));
-        readback.extend(outcome_bytes([
+        readback.extend(region_record_outcome_bytes_of_the_words([
             OUTCOME_KIND_REFUSED_BY_THE_PARSER,
             UNIT_ERROR_WRONG_UNIT_CLASS,
             u32::from(UNIT_CLASS_DATA) | (2 << 8),
@@ -1858,7 +1854,7 @@ mod tests {
             0,
             0,
         ]));
-        readback.extend(outcome_bytes([
+        readback.extend(region_record_outcome_bytes_of_the_words([
             OUTCOME_KIND_REFUSED_BY_THE_PARSER,
             UNIT_ERROR_STRUCTURE,
             7,
@@ -1910,8 +1906,22 @@ mod tests {
     }
 
     #[test]
-    fn an_outcome_the_kernels_never_wrote_is_an_unrecognized_verdict_word() {
-        let mut readback = outcome_bytes([OUTCOME_KIND_DEAD, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
+    fn region_record_outcome_the_kernels_never_wrote_is_an_unrecognized_verdict_word() {
+        let mut readback = region_record_outcome_bytes_of_the_words([
+            OUTCOME_KIND_DEAD,
+            0,
+            0,
+            0,
+            0,
+            0,
+            0,
+            0,
+            0,
+            0,
+            0,
+            0,
+            0,
+        ]);
         readback.extend(region_record_outcome_buffer_before_the_kernel_writes(1));
         assert_eq!(
             decoded_region_record_outcomes(&readback),
@@ -1955,13 +1965,13 @@ mod tests {
             ),
         ];
         let mapping_entries = vec![
-            entry(data_key, locations_at(slot_in_region(0))),
-            entry(node_key.clone(), locations_at(slot_in_region(2))),
-            entry(packed_key, locations_at(slot_in_region(4))),
+            central_mapping_entry(data_key, locations_at(slot_in_region(0))),
+            central_mapping_entry(node_key.clone(), locations_at(slot_in_region(2))),
+            central_mapping_entry(packed_key, locations_at(slot_in_region(4))),
             // 槽 8 上那个节点的 key 不在映射里，指着槽 8 的是槽 2 那个节点的 key
-            entry(node_key, locations_at(slot_in_region(8))),
+            central_mapping_entry(node_key, locations_at(slot_in_region(8))),
             // 槽 10 上是数据单元，指着它的条目 key 打头是节点的类标签
-            entry(index_node_and_key(61).1, locations_at(slot_in_region(10))),
+            central_mapping_entry(index_node_and_key(61).1, locations_at(slot_in_region(10))),
         ];
         let records: Vec<AllocationRecord> = records_and_bytes
             .iter()
@@ -2056,7 +2066,8 @@ mod tests {
     }
 
     #[test]
-    fn the_two_kernels_are_separate_sources_with_their_own_entry_points() {
+    fn the_walking_and_searching_region_record_kernels_are_separate_sources_with_their_own_entry_points(
+    ) {
         assert!(WALKING_KERNEL_SOURCE.contains(&format!("fn {WALKING_KERNEL_ENTRY_POINT}(")));
         assert!(SEARCHING_KERNEL_SOURCE.contains(&format!("fn {SEARCHING_KERNEL_ENTRY_POINT}(")));
         assert!(
@@ -2101,12 +2112,12 @@ mod tests {
             ),
         ];
         let mapping_entries = vec![
-            entry(data_key, locations_at(slot_in_region(0))),
-            entry(node_key.clone(), locations_at(slot_in_region(2))),
-            entry(packed_key, locations_at(slot_in_region(4))),
-            entry(node_key, locations_at(slot_in_region(8))),
-            entry(index_node_and_key(61).1, locations_at(slot_in_region(10))),
-            entry(key_mapped_elsewhere, locations_at(slot_in_region(40))),
+            central_mapping_entry(data_key, locations_at(slot_in_region(0))),
+            central_mapping_entry(node_key.clone(), locations_at(slot_in_region(2))),
+            central_mapping_entry(packed_key, locations_at(slot_in_region(4))),
+            central_mapping_entry(node_key, locations_at(slot_in_region(8))),
+            central_mapping_entry(index_node_and_key(61).1, locations_at(slot_in_region(10))),
+            central_mapping_entry(key_mapped_elsewhere, locations_at(slot_in_region(40))),
         ];
         let records: Vec<RegionRecordToClassify<'_, usize>> = records_and_bytes
             .iter()
@@ -2329,10 +2340,10 @@ mod tests {
             ),
         ];
         let mapping_entries = vec![
-            entry(data_key, locations_at(slot_in_region(0))),
-            entry(node_key, locations_at(slot_in_region(2))),
-            entry(packed_key, locations_at(slot_in_region(4))),
-            entry(key_mapped_elsewhere, locations_at(slot_in_region(40))),
+            central_mapping_entry(data_key, locations_at(slot_in_region(0))),
+            central_mapping_entry(node_key, locations_at(slot_in_region(2))),
+            central_mapping_entry(packed_key, locations_at(slot_in_region(4))),
+            central_mapping_entry(key_mapped_elsewhere, locations_at(slot_in_region(40))),
         ];
         let records: Vec<RegionRecordToClassify<'_, usize>> = records_and_bytes
             .iter()
@@ -2426,7 +2437,7 @@ mod tests {
         let mut outcome_words = [0_u32; REGION_RECORD_OUTCOME_WORDS];
         outcome_words[0] = OUTCOME_KIND_REGISTER_AFTER_THE_PIECE;
         outcome_words[1] = 0xDEAD_BEEF;
-        let mut readback = outcome_bytes(outcome_words);
+        let mut readback = region_record_outcome_bytes_of_the_words(outcome_words);
         readback.extend(region_record_outcome_buffer_before_the_kernel_writes(1));
         assert_eq!(
             decoded_registers_after_the_pieces(&readback),
@@ -2451,13 +2462,16 @@ mod tests {
         for (entries, expected_location_items) in [
             (Vec::new(), 0_u64),
             (
-                vec![entry(data_key.clone(), locations_at(slot_in_region(0)))],
+                vec![central_mapping_entry(
+                    data_key.clone(),
+                    locations_at(slot_in_region(0)),
+                )],
                 2,
             ),
             (
                 vec![
-                    entry(data_key.clone(), locations_at(slot_in_region(0))),
-                    entry(
+                    central_mapping_entry(data_key.clone(), locations_at(slot_in_region(0))),
+                    central_mapping_entry(
                         data_key,
                         [
                             locations_at(slot_in_region(3))[0],
```

### crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs

```diff
diff --git a/crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs b/crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs
index a894e79..2583aef 100644
--- a/crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs
+++ b/crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs
@@ -147,7 +147,7 @@ mod tests {
     use super::*;
 
     /// 假的条目：键是一个数，结果是一个数（两份核交回的数相同就算相同）。
-    fn entry(
+    fn two_kernel_outcomes_of_the_entry(
         entry_key: u32,
         outcome_by_one_kernel: u32,
         outcome_by_the_independently_written_kernel: u32,
@@ -160,7 +160,7 @@ mod tests {
     }
 
     /// CPU 参照判出的结果固定是 900 加位置，测试认得出是不是它判的。
-    fn cpu_reference_outcome(entry_position: usize) -> u32 {
+    fn outcome_the_cpu_reference_gives_at_the_position(entry_position: usize) -> u32 {
         900 + u32::try_from(entry_position).expect("测试的批不大")
     }
 
@@ -168,7 +168,10 @@ mod tests {
     fn identical_outcomes_are_confirmed() {
         let mut counts = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
         let batch = counts.reconcile_the_two_kernel_outcomes(
-            vec![entry(1, 7, 7), entry(2, 0, 0)],
+            vec![
+                two_kernel_outcomes_of_the_entry(1, 7, 7),
+                two_kernel_outcomes_of_the_entry(2, 0, 0),
+            ],
             |_entry_position| panic!("两份核相同时不交 CPU 判"),
         );
         assert_eq!(
@@ -189,7 +192,10 @@ mod tests {
     fn different_outcomes_are_uncertain_for_one_round_without_asking_the_cpu() {
         let mut counts = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
         let batch = counts.reconcile_the_two_kernel_outcomes(
-            vec![entry(1, 7, 8), entry(2, 5, 5)],
+            vec![
+                two_kernel_outcomes_of_the_entry(1, 7, 8),
+                two_kernel_outcomes_of_the_entry(2, 5, 5),
+            ],
             |_entry_position| panic!("只不确定一轮时不交 CPU 判"),
         );
         assert_eq!(batch.decisions[0], EntryDecision::UncertainThisRound);
@@ -207,15 +213,19 @@ mod tests {
     #[test]
     fn two_consecutive_uncertain_rounds_hand_the_entry_to_the_cpu_reference() {
         let mut counts = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
-        counts.reconcile_the_two_kernel_outcomes(vec![entry(1, 7, 8)], |_entry_position| {
-            panic!("只不确定一轮时不交 CPU 判")
-        });
+        counts.reconcile_the_two_kernel_outcomes(
+            vec![two_kernel_outcomes_of_the_entry(1, 7, 8)],
+            |_entry_position| panic!("只不确定一轮时不交 CPU 判"),
+        );
         let mut cpu_judged_positions = Vec::new();
         let batch = counts.reconcile_the_two_kernel_outcomes(
-            vec![entry(3, 4, 4), entry(1, 9, 2)],
+            vec![
+                two_kernel_outcomes_of_the_entry(3, 4, 4),
+                two_kernel_outcomes_of_the_entry(1, 9, 2),
+            ],
             |entry_position| {
                 cpu_judged_positions.push(entry_position);
-                cpu_reference_outcome(entry_position)
+                outcome_the_cpu_reference_gives_at_the_position(entry_position)
             },
         );
         assert_eq!(
@@ -227,7 +237,9 @@ mod tests {
             batch.decisions,
             vec![
                 EntryDecision::ConfirmedByBothKernels(4),
-                EntryDecision::JudgedByTheCpuReference(cpu_reference_outcome(1)),
+                EntryDecision::JudgedByTheCpuReference(
+                    outcome_the_cpu_reference_gives_at_the_position(1)
+                ),
             ],
             "按 CPU 的判定处理"
         );
@@ -252,8 +264,14 @@ mod tests {
         let never_cpu = |_entry_position: usize| -> u32 {
             panic!("没有连续两轮不确定，不交 CPU 判")
         };
-        counts.reconcile_the_two_kernel_outcomes(vec![entry(1, 7, 8)], never_cpu);
-        let agreeing = counts.reconcile_the_two_kernel_outcomes(vec![entry(1, 7, 7)], never_cpu);
+        counts.reconcile_the_two_kernel_outcomes(
+            vec![two_kernel_outcomes_of_the_entry(1, 7, 8)],
+            never_cpu,
+        );
+        let agreeing = counts.reconcile_the_two_kernel_outcomes(
+            vec![two_kernel_outcomes_of_the_entry(1, 7, 7)],
+            never_cpu,
+        );
         assert_eq!(
             agreeing.decisions,
             vec![EntryDecision::ConfirmedByBothKernels(7)]
@@ -263,8 +281,10 @@ mod tests {
             0,
             "相同了就清零"
         );
-        let disagreeing_again =
-            counts.reconcile_the_two_kernel_outcomes(vec![entry(1, 7, 8)], never_cpu);
+        let disagreeing_again = counts.reconcile_the_two_kernel_outcomes(
+            vec![two_kernel_outcomes_of_the_entry(1, 7, 8)],
+            never_cpu,
+        );
         assert_eq!(
             disagreeing_again.decisions,
             vec![EntryDecision::UncertainThisRound],
@@ -276,8 +296,14 @@ mod tests {
     fn an_entry_absent_from_a_batch_keeps_its_count() {
         let mut counts = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
         let never_cpu = |_entry_position: usize| -> u32 { panic!("这一轮不交 CPU 判") };
-        counts.reconcile_the_two_kernel_outcomes(vec![entry(1, 7, 8)], never_cpu);
-        counts.reconcile_the_two_kernel_outcomes(vec![entry(2, 3, 3)], never_cpu);
+        counts.reconcile_the_two_kernel_outcomes(
+            vec![two_kernel_outcomes_of_the_entry(1, 7, 8)],
+            never_cpu,
+        );
+        counts.reconcile_the_two_kernel_outcomes(
+            vec![two_kernel_outcomes_of_the_entry(2, 3, 3)],
+            never_cpu,
+        );
         assert_eq!(
             counts.consecutive_uncertain_rounds_of(&1),
             1,
@@ -288,15 +314,16 @@ mod tests {
     #[test]
     fn dropping_the_counts_forgets_every_uncertain_round() {
         let mut counts = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
-        counts.reconcile_the_two_kernel_outcomes(vec![entry(1, 7, 8)], |_entry_position| {
-            panic!("只不确定一轮时不交 CPU 判")
-        });
+        counts.reconcile_the_two_kernel_outcomes(
+            vec![two_kernel_outcomes_of_the_entry(1, 7, 8)],
+            |_entry_position| panic!("只不确定一轮时不交 CPU 判"),
+        );
         drop(counts);
         let mut counts_after_a_remount = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
-        let batch = counts_after_a_remount
-            .reconcile_the_two_kernel_outcomes(vec![entry(1, 7, 8)], |_entry_position| {
-                panic!("重挂之后从零数起")
-            });
+        let batch = counts_after_a_remount.reconcile_the_two_kernel_outcomes(
+            vec![two_kernel_outcomes_of_the_entry(1, 7, 8)],
+            |_entry_position| panic!("重挂之后从零数起"),
+        );
         assert_eq!(batch.decisions, vec![EntryDecision::UncertainThisRound]);
     }
 
@@ -305,7 +332,10 @@ mod tests {
     fn the_same_entry_twice_in_one_batch_panics() {
         let mut counts = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
         counts.reconcile_the_two_kernel_outcomes(
-            vec![entry(1, 7, 7), entry(1, 7, 7)],
+            vec![
+                two_kernel_outcomes_of_the_entry(1, 7, 7),
+                two_kernel_outcomes_of_the_entry(1, 7, 7),
+            ],
             |_entry_position| 0,
         );
     }
```

