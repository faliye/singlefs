# m4-gpugc-precheck-b-a-code-r2：被判的改动全文（第一轮判决第三节第 1、2、4、5 条，第 128、129 两批补丁）

第 128 批（两处文档、两处调用的真池用例；产品代码只改注释）：`~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-r1-fixes-writer/patch/crates.patch`。

```diff
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index 6c920d1..5c2af5d 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -2117,7 +2117,8 @@ pub fn refusal_is_short_of_space(refusal: &PublishError) -> bool {
     match refusal {
         PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. } => true,
         // 合取 7（全空聚簇段少于 `N₀`）：全空段数按记录口径数（`allocator::DeviceFreeMap::empty_segments`，defer 里已释放的槽本来就不算占用），
-        // 抬 F 把 defer 里的槽回收不改这个数，推了只多发几次空发布；原样交回这个成员，调用方分得出「先整理」与「真没空间」。
+        // 抬 F 把 defer 里的槽回收不改这个数；推串里的空发布会换下固定点单元，可能把全空段数推高，判的是被拒那一刻的数，
+        // 推高之后越不越过 `N₀` 不另估，不为它推串。原样交回这个成员，调用方分得出「先整理」与「真没空间」。
         PublishError::EmptyClusterSegmentsBelowTheFloor(_)
         | PublishError::MultiLevelCodeTwoTreeRefused { .. }
         | PublishError::ReleaseNotInMapping { .. }
@@ -6170,8 +6171,9 @@ impl RefusedPublishUnderTheEmptyClusterSegmentsConjunct {
 /// 抬 F 之前先算的合取 7 那一判（D16（发布语义） 已定项 10 射程「抬 F 之前先算」）：这次调用还一串都没推（`series_pushed_in_this_change` 是 0）、
 /// 被拒是空间不够的那两种（[`refusal_is_short_of_space`]）、被拒的是合取 7 判的前台写入类、准入判着时，读每块盘此刻的全空聚簇段数
 /// （`DeviceFreeMap::empty_segments`，合取 7 自己读的那一个）与整理开关里的 `N₀`，照合取 7 同一个判法
-/// （[`admit_the_empty_cluster_segments_floor_on_every_device`]）判，有一块少于 `N₀` 就交回那几块盘。判的是写之前那一刻的数，不另估这次写之后：
-/// 抬 F 回收 defer 里的槽不改这个数（defer 里的已释放槽本来就不算占用），推了这次写也是被合取 7 拒。只读分配器（内存）。
+/// （[`admit_the_empty_cluster_segments_floor_on_every_device`]）判，有一块少于 `N₀` 就交回那几块盘。判的是写之前那一刻的数：
+/// 抬 F 回收 defer 里的槽不改这个数（defer 里的已释放槽本来就不算占用）；推串里的空发布会换下固定点单元，可能把全空段数推高，
+/// 推高之后越不越过 `N₀` 不另估。只读分配器（内存）。
 fn empty_cluster_segments_below_the_floor_before_any_series(
     allocator: &PoolAllocator,
     refusal: &PublishError,
@@ -8734,7 +8736,7 @@ mod everything_a_floor_raise_could_release_tests {
                 refusal,
             )
         else {
-            panic!("全空聚簇段少于 N₀ 的前台写入推了也被合取 7 拒：不推");
+            panic!("全空聚簇段在写之前已少于 N₀ 的前台写入：先算判出，不推");
         };
         assert_eq!(
             (
diff --git a/crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs b/crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs
index a874f75..06ca999 100644
--- a/crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs
+++ b/crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs
@@ -1,5 +1,6 @@
 //! `accept` 被空间拒时当场推抬 F 再判（D16（发布语义） 已定项 1「准入」那一行）的几格，核心层的攒批会话上判：
-//! 待发里已收下的改动不丢、不重复发（推成、抬 F 写盘报错冻结、排空时抬 F 写盘报错冻结）；推到不再推了还放不下、真满没有可放的报拒不崩，拒绝里带推了几串，盘上状态挂得上、读得出；推抬 F 全放出来也放不下的不推（短在式子那一判、短在段外槽对那一判而三样全放开之后真成的对数也不够、推串之前全空聚簇段已少于 `N₀` 三形），拒绝里带这次调用已落盘的。
+//! 待发里已收下的改动不丢、不重复发（推成、抬 F 写盘报错冻结、排空时抬 F 写盘报错冻结）；推到不再推了还放不下、真满没有可放的报拒不崩，拒绝里带推了几串，盘上状态挂得上、读得出；推抬 F 全放出来也放不下的不推（短在式子那一判、短在段外槽对那一判而三样全放开之后真成的对数也不够、推串之前全空聚簇段已少于 `N₀` 三形；全空聚簇段那一形经 `accept` 与经会话单发各判一次），拒绝里带这次调用已落盘的；
+//! 走空那一步被空间拒时合取 7 免，全空聚簇段少于 `N₀` 也照推。
 //! 真写满的池上的那一族与运行库一层在 `crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs`。
 mod common_admission_pool;
 
@@ -915,30 +916,25 @@ fn an_accept_short_only_behind_isolated_slots_pushes_a_series_and_reports_it_in_
     );
 }
 
-/// 删除队列没走空时交一次放不下的写：走空那几步先发成，这次写才判到全放出来也放不下，拒绝里带着走空发成的那几版，不丢账。
-/// 队列是这样留下的：unlink 意图写盘报错冻结，重发成了之后意图落盘、它的批还没走。
-#[test]
-fn a_write_refused_outright_after_walking_the_queue_keeps_the_versions_of_the_walk() {
-    let PoolWithTheFloorBelowItsCeiling {
-        mut pool,
-        session,
-        fault,
-    } = pool_with_the_floor_below_its_ceiling(
-        UNIT_AREA_SLOTS_SMALLER_THAN_A_BATCH_WITHIN_THE_RECORD_LIMIT,
-    );
-    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
-    let mut session = batching(session, journal_ring_bytes);
-    let inode_to_unlink = InodeNumber(inode_watermark_of(&session));
+/// 删除队列留一个没走完的 unlink：建一个 inode、给它写一个单元，unlink 意图写盘报错冻结，重发成了之后意图落盘、它的批还没走。
+/// 三次改动的写时间从 `write_time_seconds` 起逐次加一秒。
+fn leave_an_unlink_whose_batches_have_not_walked(
+    pool: &mut PoolUnderTest<DeviceFailingAWriteOnRequest>,
+    session: &mut MountedSessionWithPendingChanges,
+    fault: &WriteFaultOfTheTest,
+    write_time_seconds: u64,
+) {
+    let inode_to_unlink = InodeNumber(inode_watermark_of(session));
     let content_of_the_unlinked_file = vec![0x3C_u8; data_unit_payload_capacity()];
     for change in [
         UserChange::NewInodes {
             count: 1,
-            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1705,
+            write_time_seconds,
         },
         UserChange::SequentialWriteOfAnInode {
             inode: inode_to_unlink,
             content: &content_of_the_unlinked_file,
-            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1706,
+            write_time_seconds: write_time_seconds + 1,
         },
     ] {
         session
@@ -950,7 +946,7 @@ fn a_write_refused_outright_after_walking_the_queue_keeps_the_versions_of_the_wa
         &mut pool.devices,
         UserChange::Unlink {
             inode: inode_to_unlink,
-            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1707,
+            write_time_seconds: write_time_seconds + 2,
         },
         reading_zero(),
     );
@@ -971,6 +967,27 @@ fn a_write_refused_outright_after_walking_the_queue_keeps_the_versions_of_the_wa
             .queue_of_committed_intents_is_empty(),
         "阳性对照：意图落盘了，它的批还没走"
     );
+}
+
+/// 删除队列没走空时交一次放不下的写：走空那几步先发成，这次写才判到全放出来也放不下，拒绝里带着走空发成的那几版，不丢账。
+/// 队列照 [`leave_an_unlink_whose_batches_have_not_walked`] 留下。
+#[test]
+fn a_write_refused_outright_after_walking_the_queue_keeps_the_versions_of_the_walk() {
+    let PoolWithTheFloorBelowItsCeiling {
+        mut pool,
+        session,
+        fault,
+    } = pool_with_the_floor_below_its_ceiling(
+        UNIT_AREA_SLOTS_SMALLER_THAN_A_BATCH_WITHIN_THE_RECORD_LIMIT,
+    );
+    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
+    let mut session = batching(session, journal_ring_bytes);
+    leave_an_unlink_whose_batches_have_not_walked(
+        &mut pool,
+        &mut session,
+        &fault,
+        FIRST_WRITE_TIME_SECONDS + 1705,
+    );
     let content =
         vec![0x5A_u8; UNITS_OF_A_BATCH_WITHIN_THE_RECORD_LIMIT * data_unit_payload_capacity()];
     let Err(refused) = session.publish_user_change(
@@ -1166,6 +1183,13 @@ fn nothing_was_pushed_and_the_old_content_reads_back_after_a_crash(
     assert!(pool.image() == *image_before, "盘上逐字节不变");
     assert_eq!(session.pending_changes_accepted(), 0, "待发空");
     drop(session);
+    the_old_content_reads_back_after_a_crash(pool);
+}
+
+/// 会话放下之后进程退出：再可写挂载做得成，读得出这一次之前的内容。
+fn the_old_content_reads_back_after_a_crash(
+    pool: &mut PoolUnderTest<DeviceFailingAWriteOnRequest>,
+) {
     let content_before = pool.content_of_the_current_version.clone();
     pool.crash_and_mount_writable()
         .expect("崩了再可写挂载做得成：这一次没动盘");
@@ -1334,20 +1358,15 @@ fn force_the_empty_cluster_segments_floor(session: &mut MountedSession, floor_in
         });
 }
 
-/// 落得下那一判先拒（段外没挡的槽对比需求少一对，差额压在抬 F 扣住的槽上），槽对那一判的上界够得上需求，而每块盘的全空聚簇段都少于 `N₀`：
-/// 推了也是被合取 7 拒，一串都不推就拒，带上合取 7 那一份（`N₀` 与每块盘的全空段数）、不点名槽对短的盘；盘上逐字节不变、待发空，
-/// 崩了再挂载读得出旧内容。这个池单元区大半空着，全空段多，`N₀` 经只供测试的开关抬到比每块盘的全空段数都多一段。
-/// 判别力：推串之前不判合取 7 时推一串抬 F，推完照旧被拒。
-#[test]
-fn an_accept_while_empty_cluster_segments_are_below_the_floor_is_refused_without_a_series() {
-    let PoolWithTheFloorBelowItsCeiling {
-        mut pool,
-        mut session,
-        fault: _,
-    } = pool_with_the_floor_below_its_ceiling(UNIT_AREA_SLOTS_OF_THE_SMALLER_DEVICES);
+/// 会话摆成落得下那一判会先拒一次 [`UNITS_OF_A_BATCH_WITHIN_THE_RECORD_LIMIT`] 个数据单元的写（段外没挡的槽对比需求少一对，差额压在抬 F 扣住的槽上）、
+/// 槽对那一判的上界够得上需求，再经只供测试的开关把 `N₀` 抬到比每块盘的全空段数都多一段（这个池单元区大半空着，全空段多）。
+/// 交回合取 7 那一判该报的那一份（`N₀` 与每块盘的全空段数）。
+fn hold_slots_and_raise_the_empty_cluster_segments_floor_above_every_device(
+    session: &mut MountedSession,
+) -> EmptyClusterSegmentsBelowTheFloorOnSomeDevices {
     let units_of_the_write =
         u64::try_from(UNITS_OF_A_BATCH_WITHIN_THE_RECORD_LIMIT).expect("单元数装得进 u64");
-    hold_the_highest_free_slots_leaving_slot_pairs(&mut session, units_of_the_write - 1);
+    hold_the_highest_free_slots_leaving_slot_pairs(session, units_of_the_write - 1);
     let cluster_segments = session.allocator.cluster_segments().clone();
     for device_map in &session.allocator.devices {
         assert_eq!(
@@ -1378,24 +1397,19 @@ fn an_accept_while_empty_cluster_segments_are_below_the_floor_is_refused_without
         .max()
         .expect("池里有盘")
         + 1;
-    force_the_empty_cluster_segments_floor(&mut session, floor_in_segments);
-    let floor_before = session.current.root().rollback_floor;
-    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
-    let mut session = batching(session, journal_ring_bytes);
-    let content =
-        vec![0x5A_u8; UNITS_OF_A_BATCH_WITHIN_THE_RECORD_LIMIT * data_unit_payload_capacity()];
-    let image_before = pool.image();
-    let refused = session
-        .accept(
-            &mut pool.devices,
-            UserChange::SequentialWrite(FirstFile {
-                content: &content,
-                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1710,
-            }),
-            reading_zero(),
-        )
-        .expect_err("全空聚簇段少于 N₀：收不下");
-    let no_space = refused_outright_for_this_change_alone(&refused);
+    force_the_empty_cluster_segments_floor(session, floor_in_segments);
+    EmptyClusterSegmentsBelowTheFloorOnSomeDevices {
+        floor_in_segments,
+        short_devices: empty_cluster_segments_on_each_device,
+    }
+}
+
+/// 被拒的这一次是推串之前合取 7 那一判判出的：没有上界小于需求的盘，合取 7 那一份是 `below_the_floor_expected`，
+/// 被拒的那一次发布是落得下那一判的段外槽对那一样拒的；一串都没推、一版都没发。
+fn refused_by_the_empty_cluster_segments_judgement_before_any_series(
+    no_space: &NoSpaceEvenWithEverythingAFloorRaiseCouldRelease,
+    below_the_floor_expected: EmptyClusterSegmentsBelowTheFloorOnSomeDevices,
+) {
     assert!(
         no_space.devices_short.is_empty(),
         "没有上界小于需求的盘：{:?}",
@@ -1403,10 +1417,7 @@ fn an_accept_while_empty_cluster_segments_are_below_the_floor_is_refused_without
     );
     assert_eq!(
         no_space.empty_cluster_segments_below_the_floor,
-        Some(EmptyClusterSegmentsBelowTheFloorOnSomeDevices {
-            floor_in_segments,
-            short_devices: empty_cluster_segments_on_each_device,
-        }),
+        Some(below_the_floor_expected),
         "合取 7 那一份：N₀ 与每块盘的全空段数"
     );
     let PublishError::SpaceAdmissionRefused(admission_refusal) = &no_space.last_refusal else {
@@ -1420,6 +1431,47 @@ fn an_accept_while_empty_cluster_segments_are_below_the_floor_is_refused_without
             }),
         "推串之前被拒的是段外槽对那一判：{admission_refusal:?}"
     );
+    assert!(
+        no_space.floor_raises.is_empty()
+            && no_space.refusals_that_pushed_the_floor_raises.is_empty()
+            && no_space.versions_published.is_empty(),
+        "头一次被拒就判到这里：一串都没推、一版都没发"
+    );
+}
+
+/// 落得下那一判先拒、槽对那一判的上界够得上需求，而每块盘的全空聚簇段在写之前都已少于 `N₀`
+/// （[`hold_slots_and_raise_the_empty_cluster_segments_floor_above_every_device`]）：被拒的是写文件内容、归合取 7 管，
+/// 推串之前那一判判出，一串都不推就拒，带上合取 7 那一份、不点名槽对短的盘；盘上逐字节不变、待发空，崩了再挂载读得出旧内容。
+/// 判别力：推串之前不判合取 7 时推一串抬 F。
+#[test]
+fn an_accept_while_empty_cluster_segments_are_below_the_floor_is_refused_without_a_series() {
+    let PoolWithTheFloorBelowItsCeiling {
+        mut pool,
+        mut session,
+        fault: _,
+    } = pool_with_the_floor_below_its_ceiling(UNIT_AREA_SLOTS_OF_THE_SMALLER_DEVICES);
+    let below_the_floor_expected =
+        hold_slots_and_raise_the_empty_cluster_segments_floor_above_every_device(&mut session);
+    let floor_before = session.current.root().rollback_floor;
+    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
+    let mut session = batching(session, journal_ring_bytes);
+    let content =
+        vec![0x5A_u8; UNITS_OF_A_BATCH_WITHIN_THE_RECORD_LIMIT * data_unit_payload_capacity()];
+    let image_before = pool.image();
+    let refused = session
+        .accept(
+            &mut pool.devices,
+            UserChange::SequentialWrite(FirstFile {
+                content: &content,
+                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1710,
+            }),
+            reading_zero(),
+        )
+        .expect_err("全空聚簇段少于 N₀：收不下");
+    refused_by_the_empty_cluster_segments_judgement_before_any_series(
+        refused_outright_for_this_change_alone(&refused),
+        below_the_floor_expected,
+    );
     nothing_was_pushed_and_the_old_content_reads_back_after_a_crash(
         &mut pool,
         session,
@@ -1428,3 +1480,109 @@ fn an_accept_while_empty_cluster_segments_are_below_the_floor_is_refused_without
         floor_before,
     );
 }
+
+/// 同一形的池与同一次写，不经 `accept`、经会话的 [`MountedSession::publish_user_change`] 单发：被拒的是这次改动自己那一次发布
+/// （删除队列是空的，没有先走空），写文件内容归合取 7 管，推串之前那一判判出，一串都不推就拒，带上合取 7 那一份；
+/// F 没动、盘上逐字节不变，崩了再挂载读得出旧内容。
+/// 判别力：`publish_user_change` 那一处把被拒的改动当成合取 7 免的那一种交下去时照推一串抬 F。
+#[test]
+fn a_publish_of_a_write_while_empty_cluster_segments_are_below_the_floor_is_refused_without_a_series(
+) {
+    let PoolWithTheFloorBelowItsCeiling {
+        mut pool,
+        mut session,
+        fault: _,
+    } = pool_with_the_floor_below_its_ceiling(UNIT_AREA_SLOTS_OF_THE_SMALLER_DEVICES);
+    let below_the_floor_expected =
+        hold_slots_and_raise_the_empty_cluster_segments_floor_above_every_device(&mut session);
+    assert!(
+        session
+            .current
+            .version_of_trees()
+            .expect("有树")
+            .queue_of_committed_intents_is_empty(),
+        "阳性对照：删除队列空，被拒的是这次改动自己那一次发布"
+    );
+    let floor_before = session.current.root().rollback_floor;
+    let content =
+        vec![0x5A_u8; UNITS_OF_A_BATCH_WITHIN_THE_RECORD_LIMIT * data_unit_payload_capacity()];
+    let image_before = pool.image();
+    let Err(refused) = session.publish_user_change(
+        &mut pool.devices,
+        UserChange::SequentialWrite(FirstFile {
+            content: &content,
+            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1711,
+        }),
+    ) else {
+        panic!("全空聚簇段少于 N₀：发不成");
+    };
+    let UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space) = &refused
+    else {
+        panic!("写文件内容归合取 7 管：推串之前判出、不推，实际 {refused:?}");
+    };
+    refused_by_the_empty_cluster_segments_judgement_before_any_series(
+        no_space,
+        below_the_floor_expected,
+    );
+    assert_eq!(
+        session.current.root().rollback_floor,
+        floor_before,
+        "F 没动"
+    );
+    assert!(pool.image() == image_before, "盘上逐字节不变");
+    drop(session);
+    the_old_content_reads_back_after_a_crash(&mut pool);
+}
+
+/// 删除队列没走空、走空那一步被空间拒，而每块盘的全空聚簇段在写之前都已少于 `N₀`：走空的每一步是删除类、合取 7 免，
+/// 推串之前不判合取 7，照推一串抬 F。单元区里空闲的槽都记成已分配，抬 F 那一串自己也放不下，推到不再推了报空间不够，
+/// 报里被拒的是走空那一步、空间不够的那两种之一。
+/// 判别力：走空那一处把被拒的那一步当成合取 7 判的前台写入交下去时，推串之前那一判判出、一串都不推就拒（全放出来也放不下）。
+#[test]
+fn a_walk_step_refused_for_space_while_empty_cluster_segments_are_below_the_floor_is_pushed_for() {
+    let PoolWithTheFloorBelowItsCeiling {
+        mut pool,
+        session,
+        fault,
+    } = pool_with_the_floor_below_its_ceiling(UNIT_AREA_SLOTS_OF_THE_SMALLER_DEVICES);
+    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
+    let mut session = batching(session, journal_ring_bytes);
+    leave_an_unlink_whose_batches_have_not_walked(
+        &mut pool,
+        &mut session,
+        &fault,
+        FIRST_WRITE_TIME_SECONDS + 1712,
+    );
+    mark_every_free_slot_allocated(session.session_for_tests_only());
+    let floor_in_segments = session
+        .session()
+        .allocator
+        .devices
+        .iter()
+        .map(DeviceFreeMap::empty_segments)
+        .max()
+        .expect("池里有盘")
+        + 1;
+    force_the_empty_cluster_segments_floor(session.session_for_tests_only(), floor_in_segments);
+    let Err(refused) = session.publish_user_change(
+        &mut pool.devices,
+        UserChange::NewInodes {
+            count: 1,
+            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1715,
+        },
+        reading_zero(),
+    ) else {
+        panic!("单元区里一个空闲的槽都没有：走空那一步发不成");
+    };
+    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = &*refused.refusal else {
+        panic!(
+            "走空那一步免合取 7：照推，推到不再推了报空间不够，实际 {:?}",
+            refused.refusal
+        );
+    };
+    assert!(
+        refusal_is_short_of_space(&no_space.last_refusal),
+        "被拒的是走空那一步、空间不够：{:?}",
+        no_space.last_refusal
+    );
+}
```

第 129 批（槽对数 newtype、两行不并行的理由）：`~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-slot-pair-type-writer/patch/crates.patch`。

```diff
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index eadb81b..c9ad4f5 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -350,6 +350,17 @@ impl UnitAreaStart {
     }
 }
 
+/// 一块盘上的槽对数：从偶数下标起、不跨聚簇段的两槽算一对（[`DeviceFreeMap::lowest_user_data_slot`] 认的那一种），不是槽数。
+/// 一个数据单元落一对；换算成字节先乘一个数据单元的槽数。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct SlotPairCount(pub u64);
+
+impl std::iter::Sum for SlotPairCount {
+    fn sum<SlotPairCounts: Iterator<Item = Self>>(counts: SlotPairCounts) -> Self {
+        SlotPairCount(counts.map(|count| count.0).sum())
+    }
+}
+
 /// 一块设备的空闲图与它的记账增量。
 #[derive(Clone, Debug)]
 pub struct DeviceFreeMap {
@@ -401,18 +412,18 @@ pub struct DeviceFreeMap {
     /// 每个 64 槽聚簇段里两槽都没挡（三种位都没置）的偶数起点槽对数：用户数据落得下的槽对（[`Self::lowest_user_data_slot`] 认的那一种，
     /// 段内外之分由读的一方减），同上增量维护。单元区起点在段边界上（[`UnitAreaStart`] 的不变量）、段长是偶数 ⇒ 一对不跨段；
     /// 单元区末尾凑不成一对的那一槽不算。
-    unblocked_slot_pairs_per_segment: Vec<u64>,
+    unblocked_slot_pairs_per_segment: Vec<SlotPairCount>,
     /// 上一项全部段之和。
-    unblocked_slot_pairs: u64,
+    unblocked_slot_pairs: SlotPairCount,
     /// 每一槽是不是罩在未释放的分配记录下（`unreleased_per_segment` 数的那几槽，逐槽一位）：分配与复活时置、释放时清，回收不动。
     /// 不罩在未释放记录下的槽只有四种：空闲的、defer 里的、影子账隔离的、抬 F 扣住的——抬 F 能放开的就是后三种。
     is_under_an_unreleased_record: Vec<bool>,
     /// 每个 64 槽聚簇段里两槽都不罩在未释放分配记录下的偶数起点槽对数：defer、影子账隔离、抬 F 扣住三样全放开之后用户数据落得下的槽对
     /// （抬 F 之前先算槽对那一判的上界读它，`mounted_session::most_a_floor_raise_could_make_available`），
     /// 随 `is_under_an_unreleased_record` 每变一槽增量维护。一对不跨段、单元区末尾凑不成一对的那一槽不算，同 `unblocked_slot_pairs_per_segment`。
-    slot_pairs_without_an_unreleased_record_per_segment: Vec<u64>,
+    slot_pairs_without_an_unreleased_record_per_segment: Vec<SlotPairCount>,
     /// 上一项全部段之和。
-    slot_pairs_without_an_unreleased_record: u64,
+    slot_pairs_without_an_unreleased_record: SlotPairCount,
     /// 分配记录树每片叶里三种位都没置的单元区槽数（叶序号 = 槽号 ÷ [`ALLOCATION_RECORD_TREE_LEAF_SLOTS`]，下标 = 叶序号 − 单元区起点所在那片叶的序号；
     /// 单元区首尾那两片叶只数单元区里的那一截）。回落取空槽最多的叶时拿它当每片叶「每块盘都空的槽数」的上界
     /// （[`PoolAllocator::try_allocate_commit_generated_preferring_the_rewrite_set`]），同上增量维护。
@@ -499,9 +510,9 @@ pub fn unit_area_slots_of_device(device_bytes: u64) -> u64 {
 /// （[`DeviceFreeMap::lowest_empty_segment`]），每块盘上都是整段、在单元区里。
 fn pairs_inside_the_cluster_segments(
     unit_area_start: UnitAreaStart,
-    pairs_per_segment: &[u64],
+    pairs_per_segment: &[SlotPairCount],
     cluster_segments: &BTreeSet<SlotNumber>,
-) -> u64 {
+) -> SlotPairCount {
     cluster_segments
         .iter()
         .map(|segment_start| {
@@ -551,10 +562,12 @@ impl DeviceFreeMap {
             .expect("建空闲图的盘末尾不在单元区起点之前：mkfs 判过 UnitAreaBeyondDevice，可写挂载读盘之前判过 every_device_reaches_the_unit_area_start");
         let segments = unit_area_slots.div_ceil(CLUSTER_SEGMENT_SLOTS);
         // 起步整个单元区都空：每段的槽对数是这一段的槽数 ÷ 2（末尾那一段可能不满 64 槽、槽数可能是奇数，多出的一槽不成对）。
-        let unblocked_slot_pairs_per_segment: Vec<u64> = (0..segments)
+        let unblocked_slot_pairs_per_segment: Vec<SlotPairCount> = (0..segments)
             .map(|segment_index| {
                 let first_slot_of_the_segment = segment_index * CLUSTER_SEGMENT_SLOTS;
-                (unit_area_slots - first_slot_of_the_segment).min(CLUSTER_SEGMENT_SLOTS) / 2
+                SlotPairCount(
+                    (unit_area_slots - first_slot_of_the_segment).min(CLUSTER_SEGMENT_SLOTS) / 2,
+                )
             })
             .collect();
         // 起步整个单元区都空：每片叶的数是这片叶与单元区相交那一截的槽数。
@@ -597,14 +610,17 @@ impl DeviceFreeMap {
             ],
             held_per_segment: vec![0; usize::try_from(segments).expect("段数")],
             unblocked_slots: unit_area_slots,
-            unblocked_slot_pairs: unblocked_slot_pairs_per_segment.iter().sum(),
+            unblocked_slot_pairs: unblocked_slot_pairs_per_segment.iter().copied().sum(),
             // 起步一槽都不罩在未释放记录下：每段的数与没挡的槽对数相同。
             is_under_an_unreleased_record: vec![
                 false;
                 usize::try_from(unit_area_slots)
                     .expect("单元区槽数")
             ],
-            slot_pairs_without_an_unreleased_record: unblocked_slot_pairs_per_segment.iter().sum(),
+            slot_pairs_without_an_unreleased_record: unblocked_slot_pairs_per_segment
+                .iter()
+                .copied()
+                .sum(),
             slot_pairs_without_an_unreleased_record_per_segment: unblocked_slot_pairs_per_segment
                 .clone(),
             unblocked_slot_pairs_per_segment,
@@ -676,12 +692,12 @@ impl DeviceFreeMap {
         let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
         match (pair_was_unblocked, self.is_unblocked_pair_at_index(index)) {
             (true, false) => {
-                self.unblocked_slot_pairs_per_segment[segment] -= 1;
-                self.unblocked_slot_pairs -= 1;
+                self.unblocked_slot_pairs_per_segment[segment].0 -= 1;
+                self.unblocked_slot_pairs.0 -= 1;
             }
             (false, true) => {
-                self.unblocked_slot_pairs_per_segment[segment] += 1;
-                self.unblocked_slot_pairs += 1;
+                self.unblocked_slot_pairs_per_segment[segment].0 += 1;
+                self.unblocked_slot_pairs.0 += 1;
             }
             (true, true) | (false, false) => {}
         }
@@ -764,12 +780,13 @@ impl DeviceFreeMap {
         &self,
         cluster_segments: &BTreeSet<SlotNumber>,
     ) -> u64 {
-        self.unblocked_slot_pairs
+        self.unblocked_slot_pairs.0
             - pairs_inside_the_cluster_segments(
                 self.unit_area_start,
                 &self.unblocked_slot_pairs_per_segment,
                 cluster_segments,
             )
+            .0
     }
 
     /// defer、影子账隔离、抬 F 扣住三样全放开之后用户数据落得下的槽对数：两槽都不罩在未释放分配记录下、起点偶数、不在 `cluster_segments` 里任何一段
@@ -783,13 +800,16 @@ impl DeviceFreeMap {
     pub fn slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
         &self,
         cluster_segments: &BTreeSet<SlotNumber>,
-    ) -> u64 {
-        self.slot_pairs_without_an_unreleased_record
-            - pairs_inside_the_cluster_segments(
-                self.unit_area_start,
-                &self.slot_pairs_without_an_unreleased_record_per_segment,
-                cluster_segments,
-            )
+    ) -> SlotPairCount {
+        SlotPairCount(
+            self.slot_pairs_without_an_unreleased_record.0
+                - pairs_inside_the_cluster_segments(
+                    self.unit_area_start,
+                    &self.slot_pairs_without_an_unreleased_record_per_segment,
+                    cluster_segments,
+                )
+                .0,
+        )
     }
 
     #[must_use]
@@ -887,12 +907,12 @@ impl DeviceFreeMap {
             self.is_pair_without_an_unreleased_record_at_index(index),
         ) {
             (true, false) => {
-                self.slot_pairs_without_an_unreleased_record_per_segment[segment] -= 1;
-                self.slot_pairs_without_an_unreleased_record -= 1;
+                self.slot_pairs_without_an_unreleased_record_per_segment[segment].0 -= 1;
+                self.slot_pairs_without_an_unreleased_record.0 -= 1;
             }
             (false, true) => {
-                self.slot_pairs_without_an_unreleased_record_per_segment[segment] += 1;
-                self.slot_pairs_without_an_unreleased_record += 1;
+                self.slot_pairs_without_an_unreleased_record_per_segment[segment].0 += 1;
+                self.slot_pairs_without_an_unreleased_record.0 += 1;
             }
             (true, true) | (false, false) => {}
         }
@@ -3861,7 +3881,7 @@ mod tests {
     fn slot_pairs_once_everything_is_released_by_the_records(
         pool: &PoolAllocator,
         device_map: &DeviceFreeMap,
-    ) -> u64 {
+    ) -> SlotPairCount {
         let first_slot = device_map.unit_area_start().slot().0;
         let past_the_last_slot = first_slot + device_map.unit_area_slots();
         let is_under_an_unreleased_record_by_the_records = |slot: u64| {
@@ -3886,6 +3906,7 @@ mod tests {
                 })
                 .count(),
         )
+        .map(SlotPairCount)
         .expect("槽对数装得进 u64")
     }
 
@@ -3893,7 +3914,7 @@ mod tests {
     fn slot_pairs_after_really_releasing_everything(
         pool: &PoolAllocator,
         device: DeviceIdentity,
-    ) -> u64 {
+    ) -> SlotPairCount {
         let mut released = pool.clone();
         released.reclaim_released_up_to(CheckpointTxg(u64::MAX), ReclaimedReuse::Immediately);
         released.release_reclaim_holds();
@@ -3907,7 +3928,9 @@ mod tests {
         for slot in first_slot..first_slot + device_map.unit_area_slots() {
             device_map.clear_isolation_of_slot(SlotNumber(slot));
         }
-        device_map.unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
+        SlotPairCount(
+            device_map.unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments),
+        )
     }
 
     /// 一串定种子的伪随机分配、开段、释放、回收、抬 F 扣住与放开、影子账隔离与清隔离、管理员回退复活（两块盘，单元区 385 槽：
@@ -4007,6 +4030,7 @@ mod tests {
     /// 抬 F 之前先算槽对那一判的上界读的数（D16（发布语义） 已定项 10 射程「抬 F 之前先算」）是增量维护的：
     /// 一串随机操作的每一步之后，拿按记录逐槽扫出来的同一个量对，两块盘都要对得上。
     /// 判别力：释放时不清那一槽的「罩在未释放记录下」位，defer 里的槽就少算。
+    /// 不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。
     #[test]
     fn slot_pairs_once_everything_is_released_are_maintained_incrementally_and_match_a_scan_of_the_records(
     ) {
@@ -4026,6 +4050,7 @@ mod tests {
 
     /// 上界不许算小（算小了会误拒一个推了本来写得进的写）：一串随机操作的每一步之后，增量维护的数不小于在拷贝上真把 defer、隔离、扣住都放开之后
     /// 段外没挡的槽对数。
+    /// 不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。
     #[test]
     fn slot_pairs_once_everything_is_released_never_fall_below_the_pairs_after_really_releasing_everything(
     ) {
@@ -4039,7 +4064,7 @@ mod tests {
                     slot_pairs_after_really_releasing_everything(pool, device_map.device);
                 assert!(
                     upper_bound >= really_released,
-                    "第 {step_number} 步（{what_the_step_did}）：盘 {:?} 上界 {upper_bound} 小于真放开之后的 {really_released}",
+                    "第 {step_number} 步（{what_the_step_did}）：盘 {:?} 上界 {upper_bound:?} 小于真放开之后的 {really_released:?}",
                     device_map.device
                 );
             }
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index 5c2af5d..f0d1096 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -37,7 +37,7 @@ use crate::admission::{
     SpaceAdmission,
 };
 use crate::allocator::{
-    AllocationRecord, DeviceFreeMap, Placement, PlacementOnDevice, PoolAllocator,
+    AllocationRecord, DeviceFreeMap, Placement, PlacementOnDevice, PoolAllocator, SlotPairCount,
 };
 use crate::background_compute::{
     decisions_of_one_segment, AlarmOfABackgroundTaskStoppedAfterAComputeError,
@@ -6053,7 +6053,7 @@ struct SlotsAFloorRaiseCouldRelease {
     /// 这次挂载开过的聚簇段之外、两槽都不罩在未释放分配记录下的偶数起点槽对：defer、影子账隔离、抬 F 扣住三样全放开之后数据单元落得下的槽对，
     /// 现有没挡的槽对也在里面（`DeviceFreeMap::slot_pairs_without_an_unreleased_record_outside_the_cluster_segments`，分配器按段增量维护）。
     /// defer 里一个数据单元的两槽放开只成一对，另一半罩在未释放记录下的槽放开了也不成对。
-    slot_pairs_once_everything_is_released: u64,
+    slot_pairs_once_everything_is_released: SlotPairCount,
 }
 
 impl SlotsAFloorRaiseCouldRelease {
@@ -6087,7 +6087,7 @@ fn most_a_floor_raise_could_make_available(
             let slots_of_a_data_unit =
                 TransactionUnit::Data(DataUnitIndexInFile::FIRST).span_slots();
             Some(AvailableBytesOnOneDevice(bytes_of_slots(
-                slots.slot_pairs_once_everything_is_released * slots_of_a_data_unit,
+                slots.slot_pairs_once_everything_is_released.0 * slots_of_a_data_unit,
             )))
         }
         JudgementThatFoundTheDeviceShort::SlotsLeftForCommitGeneratedUnits => None,
@@ -8437,7 +8437,7 @@ mod everything_a_floor_raise_could_release_tests {
         SlotsAFloorRaiseCouldRelease {
             deferred: 0,
             isolated: 0,
-            slot_pairs_once_everything_is_released: 0,
+            slot_pairs_once_everything_is_released: SlotPairCount(0),
         }
     }
 
@@ -8465,7 +8465,7 @@ mod everything_a_floor_raise_could_release_tests {
         SlotsAFloorRaiseCouldRelease {
             deferred: 2,
             isolated: 1,
-            slot_pairs_once_everything_is_released: 0,
+            slot_pairs_once_everything_is_released: SlotPairCount(0),
         }
     }
 
@@ -8521,7 +8521,7 @@ mod everything_a_floor_raise_could_release_tests {
                 SlotsAFloorRaiseCouldRelease {
                     deferred: 0,
                     isolated: 0,
-                    slot_pairs_once_everything_is_released: 8,
+                    slot_pairs_once_everything_is_released: SlotPairCount(8),
                 },
             )
             .is_some(),
diff --git a/crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs b/crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs
index 06ca999..e62ad19 100644
--- a/crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs
+++ b/crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs
@@ -1221,9 +1221,11 @@ fn an_accept_short_on_slot_pairs_that_releasing_everything_would_not_complete_is
         .devices
         .iter()
         .map(|device_map| {
-            device_map.slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
-                &cluster_segments,
-            )
+            device_map
+                .slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
+                    &cluster_segments,
+                )
+                .0
         })
         .max()
         .expect("池里有盘")
@@ -1244,9 +1246,8 @@ fn an_accept_short_on_slot_pairs_that_releasing_everything_would_not_complete_is
             "阳性对照：覆盖写过，defer 里有槽"
         );
         let slot_pairs_once_everything_is_released = device_map
-            .slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
-                &cluster_segments,
-            );
+            .slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(&cluster_segments)
+            .0;
         let slot_pairs_counting_one_pair_for_every_slot =
             slot_pairs_counting_one_pair_for_every_slot_a_floor_raise_could_release(
                 device_map,
@@ -1375,9 +1376,12 @@ fn hold_slots_and_raise_the_empty_cluster_segments_floor_above_every_device(
             "阳性对照：段外没挡的槽对比需求少一对，落得下那一判拒"
         );
         assert!(
-            device_map.slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
-                &cluster_segments
-            ) >= units_of_the_write,
+            device_map
+                .slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
+                    &cluster_segments
+                )
+                .0
+                >= units_of_the_write,
             "阳性对照：扣住的槽放得开，槽对那一判的上界够得上需求"
         );
     }
```
