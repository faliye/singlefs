# 第 31 批代码轮第一轮：改动全文（2026-10-05）

三份补丁照实现员交回的原样，按合入次序：31-1（core 断言、build_pool 与 11 处直接建分配器装根环表）、31-3（E170、旧 E171、E156 内联单测、on_device_modes 与两个调用方、overwrite_in_one_instance 用例；底座是主仓 + 31-1）、31-2（E171 新装置照登记修订 6；底座是主仓 + 31-1）。31-2 另交一份 `p6-on-dbef78ed.patch`（同一改动打在 E171 修订 5 那份装置上，给修订 6 冻结树用），不进主仓，不在这里。

## m3-formal-31-1

### crates.patch

````diff
diff --git a/crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs b/crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs
index 6ba68f1..97077ce 100644
--- a/crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs
+++ b/crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs
@@ -81,6 +81,13 @@ fn first_file_publish_after_mkfs() -> (
             span: 1,
         },
     );
+    // mkfs 同一个进程里接着写的会话装上 mkfs 刚写下的根环（没装的分配器记根就断言，C551）。
+    allocator.install_root_ring_occupancy(
+        singlefs_core::make_filesystem::root_ring_occupancy_after_make_filesystem(
+            &publish_parameters,
+            &genesis,
+        ),
+    );
     let (start, end) = {
         let mut writer = PoolWriter::new(&publish_parameters, &mut devices);
         let instance = acquire_instance(&mut writer).expect("取号");
diff --git a/crates/singlefs-checker-tier/tests/crash_segments_per_device_and_torn_in_place_overwrites.rs b/crates/singlefs-checker-tier/tests/crash_segments_per_device_and_torn_in_place_overwrites.rs
index 1824cdd..8f41c4c 100644
--- a/crates/singlefs-checker-tier/tests/crash_segments_per_device_and_torn_in_place_overwrites.rs
+++ b/crates/singlefs-checker-tier/tests/crash_segments_per_device_and_torn_in_place_overwrites.rs
@@ -467,6 +467,13 @@ fn new_pool_file_creation_stream(
             span: 1,
         },
     );
+    // mkfs 同一个进程里接着写的会话装上 mkfs 刚写下的根环（没装的分配器记根就断言，C551）。
+    allocator.install_root_ring_occupancy(
+        singlefs_core::make_filesystem::root_ring_occupancy_after_make_filesystem(
+            &parameters,
+            &genesis,
+        ),
+    );
     let content = file_content();
     {
         let mut pool = PoolWriter::new(&parameters, &mut devices);
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index cc41960..d3fbe6c 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -1293,7 +1293,7 @@ pub struct PoolAllocator {
     commit_generated_placements_that_fell_back_to_the_lowest_free_slot: u64,
     /// 分配器眼里的根环（`RootRingOccupancy`）：挂载时装（`mount::rebuilt_allocator`），mkfs 同一个进程里由建分配器的一方装
     /// （`make_filesystem::root_ring_occupancy_after_make_filesystem`）。`None`：建分配器的一方没装（`new` 与 `rebuild_from_records`
-    /// 都不装），挂载内回收与轮转清隔离位都不做，与只在挂载和抬 F 时回收的样子相同。
+    /// 都不装）。没装的分配器不许记根：`record_root_written_by_this_process` 断言（C551，用户 2026-10-05 定），只做不发布的事（分配、准入读数）。
     root_ring: Option<RootRingOccupancy>,
     /// 落盘中途失败、冻结着等原样重发的那一次发布（`transaction::FrozenPublish`，D23（journal 的角色与格式） 已定项 14
     /// 「这一版的失败处置」）。住在分配器上是因为每一次发布都要交分配器进来：冻结着的时候发布路径在任何读写之前拒绝，
@@ -1907,14 +1907,20 @@ impl PoolAllocator {
     /// **调用时机是调用方的前置条件**：这次发布的落点全部取完、之后不再分配（带单元的发布在装记账行之前调，记账行因此已经按回收之后的数写；
     /// 零单元发布在根落盘之后调）；这次发布失败时调用方把分配器整个换回发布之前（`transaction::publish_version` 与
     /// `transaction::publish_instance_table_on_version_without_file` 都这么做），这张表随之退回，盘上那个槽照旧按旧内容算
-    /// （D16（发布语义） 已定项 1「写失败的槽按旧内容算」）。没装根环时什么都不做。
+    /// （D16（发布语义） 已定项 1「写失败的槽按旧内容算」）。
     ///
     /// # Panics
-    /// 这条根的 txg 不是上一条记到的加一：有一条根没经分配器写出去，这张表说的环已经不是盘上的环，再按它回收会把还被环里的根引用的槽发出去。
+    /// - 分配器没装根环表（`install_root_ring_occupancy`）：不装就一次不回收，转过根环之后已分配统计越积越多、I-3.1（已分配统计对得上）红
+    ///   （C551（`build_pool` 的分配器没有根环表），用户 2026-10-05 定：core 在这里断言，漏装的装置当场 panic，不再静默不回收）。
+    ///   出路是建分配器的一方装上根环表：mkfs 同一个进程里用 `make_filesystem::allocator_after_make_filesystem`，或
+    ///   `install_root_ring_occupancy(make_filesystem::root_ring_occupancy_after_make_filesystem(..))`；挂载建的分配器（`mount::rebuilt_allocator`）装着。
+    /// - 这条根的 txg 不是上一条记到的加一：有一条根没经分配器写出去，这张表说的环已经不是盘上的环，再按它回收会把还被环里的根引用的槽发出去。
     pub fn record_root_written_by_this_process(&mut self, checkpoint_txg: CheckpointTxg) {
-        let Some(occupancy) = self.root_ring.as_mut() else {
-            return;
-        };
+        let occupancy = self.root_ring.as_mut().expect(
+            "分配器没装根环表却记了一条根：写根的分配器都由建它的一方装上根环表（挂载在 mount::rebuilt_allocator 装，\
+             mkfs 同一个进程里用 make_filesystem::allocator_after_make_filesystem，或 install_root_ring_occupancy(\
+             make_filesystem::root_ring_occupancy_after_make_filesystem(..))）；没装就一次不回收，转过根环之后 I-3.1 红（C551）",
+        );
         let expected_txg = CheckpointTxg(occupancy.latest_recorded_txg.0 + 1);
         assert_eq!(
             checkpoint_txg, expected_txg,
@@ -1962,7 +1968,10 @@ impl PoolAllocator {
     /// 回收门槛不越过它——读得出且自证过就照读出来的根算，按实例表有效的照它的 txg（回收门槛取整个环，RGall，多旧都算），被抛弃的当没有；
     /// 仍读不出、读得出而自证不过或读回全零照旧钉着：读回全零不算确认为空，用户 2026-10-05 定乙）。
     /// `reread` 读盘、判全零、自证与实例表（`mount::reread_of_a_root_ring_slot_bad_at_mount`）；
-    /// 这里只定重读哪一槽、按读到的改表。钉着的槽一个都不在最小的位置上时一个都不读。没装根环时只记根（什么都不做）。
+    /// 这里只定重读哪一槽、按读到的改表。钉着的槽一个都不在最小的位置上时一个都不读。
+    ///
+    /// # Panics
+    /// 同 [`Self::record_root_written_by_this_process`]：没装根环表、或这条根的 txg 不是上一条记到的加一。
     pub fn record_root_written_by_this_process_rereading_slots_bad_at_mount(
         &mut self,
         checkpoint_txg: CheckpointTxg,
@@ -2983,4 +2992,18 @@ mod tests {
         let mut stopped = pool_whose_root_ring_holds_txg_0_through_23();
         stopped.record_root_written_by_this_process(CheckpointTxg(25));
     }
+
+    /// 分配器没装根环表（只 `new` 加 mkfs 那两个单元，C551 那一形）而记了一条根：断言失败，不静默跳过回收
+    /// （用户 2026-10-05 定）。
+    #[test]
+    #[should_panic(expected = "分配器没装根环表却记了一条根")]
+    fn recording_a_root_on_an_allocator_without_a_root_ring_occupancy_panics_instead_of_never_reclaiming(
+    ) {
+        let mut without_a_root_ring = pool_after_mkfs();
+        assert!(
+            without_a_root_ring.root_ring_occupancy().is_none(),
+            "前提：new 不装根环表"
+        );
+        without_a_root_ring.record_root_written_by_this_process(CheckpointTxg(1));
+    }
 }
diff --git a/crates/singlefs-harness/tests/accounting_tree_node_full.rs b/crates/singlefs-harness/tests/accounting_tree_node_full.rs
index 9f924ef..8693047 100644
--- a/crates/singlefs-harness/tests/accounting_tree_node_full.rs
+++ b/crates/singlefs-harness/tests/accounting_tree_node_full.rs
@@ -48,11 +48,20 @@ fn many_device_pool(device_count: u32) -> ManyDevicePool {
             )
         })
         .collect();
-    let allocator = PoolAllocator::new(
+    let mut allocator = PoolAllocator::new(
         (0..device_count)
             .map(|device_number| DeviceFreeMap::new(DeviceIdentity(device_number), IMAGE_BYTES))
             .collect(),
     );
+    // 记根要分配器装着根环表（没装就断言，C551）：盘是全新的，根环上一条根都没有，这个进程写的第一条根是第一个文件版本形态那一次（txg 3）。
+    allocator.install_root_ring_occupancy(
+        singlefs_core::allocator::RootRingOccupancy::read_from_the_ring(
+            parameters().geometry.root_ring_slots_per_region,
+            Vec::new(),
+            CheckpointTxg(0),
+            CheckpointTxg(NEW_POOL_FILE_CREATION_TXG - 1),
+        ),
+    );
     ManyDevicePool { devices, allocator }
 }
 
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 56d7882..92be64f 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -5623,6 +5623,27 @@ fn published_nodes_behind_an_intermediate_row_still_count_against_the_tree_ident
                 span: 1,
             },
         );
+        // 记根要分配器装着根环表（没装就断言，C551）：照可写挂载在实例 1 那一版上读到的样子，环里记实例 1 最新那条根
+        // （实例 2 的根槽清零了，挂载读不到它们），F 0，下一条根就是写行那次。环里最旧有效根是实例 1 那一条，写行那次释放的旧链
+        // 释放代比它新，这一次不回收，账与没装根环表时一样。
+        let row_publish_txg =
+            CheckpointTxg(first_file_of_the_second_instance.root.checkpoint_txg.0 + 1);
+        allocator_rebuilt_on_the_first_instance.install_root_ring_occupancy(
+            singlefs_core::allocator::RootRingOccupancy::read_from_the_ring(
+                parameters().geometry.root_ring_slots_per_region,
+                vec![(
+                    singlefs_core::root_ring::target_for_publish(
+                        newest_root_of_the_first_instance.checkpoint_txg,
+                        parameters().geometry.root_ring_slots_per_region,
+                    ),
+                    singlefs_core::allocator::RootRingOccupant::ValidRoot {
+                        checkpoint_txg: newest_root_of_the_first_instance.checkpoint_txg,
+                    },
+                )],
+                CheckpointTxg(0),
+                CheckpointTxg(row_publish_txg.0 - 1),
+            ),
+        );
         let rewrite = InstanceTableRewrite {
             rows: rows_of_the_third_instance,
             replaced_chain: vec![newest_root_of_the_first_instance.instance_table],
@@ -5634,7 +5655,7 @@ fn published_nodes_behind_an_intermediate_row_still_count_against_the_tree_ident
             &mut allocator_rebuilt_on_the_first_instance,
             &newest_root_of_the_first_instance,
             InstanceTableOnlyPublishPlan {
-                txg: CheckpointTxg(first_file_of_the_second_instance.root.checkpoint_txg.0 + 1),
+                txg: row_publish_txg,
                 counter: first_file_of_the_second_instance.record.counter + 1,
                 instance: third_instance,
                 back_chain: 0,
diff --git a/crates/singlefs-harness/tests/common/mod.rs b/crates/singlefs-harness/tests/common/mod.rs
index 5ee88ce..0ee06e9 100644
--- a/crates/singlefs-harness/tests/common/mod.rs
+++ b/crates/singlefs-harness/tests/common/mod.rs
@@ -11,8 +11,8 @@ use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
 use singlefs_core::block_device::{FileBackedBlockDevice, PhysicalBlockSizeInBytes};
 use singlefs_core::journal::{back_chain_of, record_offset, JournalRecord};
 use singlefs_core::make_filesystem::{
-    make_filesystem, MakeFilesystemOutput, MakeFilesystemParameters, INSTANCE_TABLE_SLOT,
-    TREE_TABLE_GENESIS_SLOT,
+    make_filesystem, root_ring_occupancy_after_make_filesystem, MakeFilesystemOutput,
+    MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
 };
 use singlefs_core::recovery::{choose_system_configuration, readable_roots, PoolReader};
 use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlotsPerRegion};
@@ -696,6 +696,11 @@ pub fn build_pool(tag: &str) -> BuiltPool {
             span: 1,
         },
     );
+    // mkfs 同一个进程里接着写的会话：装上 mkfs 刚写下的根环，转过根环之后照可再分配谓词回收（C551；不装的话记根就断言）。
+    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
+        &parameters,
+        &genesis,
+    ));
     let content = file_content();
     let warm_up_operation_count;
     let (warm_up, output) = {
diff --git a/crates/singlefs-harness/tests/common_tree_split/mod.rs b/crates/singlefs-harness/tests/common_tree_split/mod.rs
index 57e7551..668add8 100644
--- a/crates/singlefs-harness/tests/common_tree_split/mod.rs
+++ b/crates/singlefs-harness/tests/common_tree_split/mod.rs
@@ -268,6 +268,13 @@ impl TreeSplitPool {
                 span: 1,
             },
         );
+        // mkfs 同一个进程里接着写的会话装上 mkfs 刚写下的根环（没装的分配器记根就断言，C551）。
+        allocator.install_root_ring_occupancy(
+            singlefs_core::make_filesystem::root_ring_occupancy_after_make_filesystem(
+                &publish_parameters,
+                &genesis,
+            ),
+        );
         let content = file_content();
         let (output, operations_before_the_first_file_version) = {
             let mut writer = PoolWriter::new(&publish_parameters, &mut devices);
diff --git a/crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs b/crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs
index 425919a..f2f16cc 100644
--- a/crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs
+++ b/crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs
@@ -142,8 +142,8 @@ fn content_of(version: u64) -> Vec<u8> {
         .collect()
 }
 
-/// 两块录制着的内存盘上做完 mkfs：交回装置与创世根。
-fn rig_after_make_filesystem() -> (Rig, singlefs_core::root_record::RootRecord) {
+/// 两块录制着的内存盘上做完 mkfs：交回装置与 mkfs 的输出。
+fn rig_after_make_filesystem() -> (Rig, singlefs_core::make_filesystem::MakeFilesystemOutput) {
     let parameters = e142_parameters(512, 512);
     let geometry = FixedGeometry {
         fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
@@ -184,13 +184,13 @@ fn rig_after_make_filesystem() -> (Rig, singlefs_core::root_record::RootRecord)
             failures,
             devices,
         },
-        genesis.root,
+        genesis,
     )
 }
 
 /// mkfs → 取号 1 → 暖机 → 第一个文件（txg 3）。
 fn build_rig() -> (Rig, PoolAllocator, TransactionOutput, InstanceGeneration) {
-    let (mut rig, genesis_root) = rig_after_make_filesystem();
+    let (mut rig, genesis) = rig_after_make_filesystem();
     let mut allocator = PoolAllocator::new(vec![
         DeviceFreeMap::new(DEVICE_ZERO, IMAGE_BYTES),
         DeviceFreeMap::new(DEVICE_ONE, IMAGE_BYTES),
@@ -205,10 +205,17 @@ fn build_rig() -> (Rig, PoolAllocator, TransactionOutput, InstanceGeneration) {
             span: 1,
         },
     );
+    // mkfs 同一个进程里接着写的会话装上 mkfs 刚写下的根环（没装的分配器记根就断言，C551）。
+    allocator.install_root_ring_occupancy(
+        singlefs_core::make_filesystem::root_ring_occupancy_after_make_filesystem(
+            &rig.parameters,
+            &genesis,
+        ),
+    );
     let (first, instance) = {
         let mut writer = PoolWriter::new(&rig.parameters, rig.devices.as_mut_slice());
         let instance = acquire_instance(&mut writer).expect("取号");
-        let warmed = warm_up(&mut writer, &genesis_root, instance).expect("暖机");
+        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
         let first = publish_first_file(
             &mut writer,
             &mut allocator,
@@ -2063,7 +2070,7 @@ fn device_one_behind_by_the_rotation_with_the_data_unit_unreadable_everywhere_st
 #[test]
 fn device_one_stale_on_the_version_without_file_refuses_the_writable_mount_naming_what_the_row_publish_rewrote(
 ) {
-    let (mut rig, _genesis_root) = rig_after_make_filesystem();
+    let (mut rig, _genesis) = rig_after_make_filesystem();
     mount_writable(&rig.parameters, &mut rig.devices).expect("第一次可写挂载");
     let stale_point = rig.stream.operation_count();
     let second_mount = mount_writable(&rig.parameters, &mut rig.devices).expect("第二次可写挂载");
diff --git a/crates/singlefs-harness/tests/multi_record_publishes_of_transaction_zero.rs b/crates/singlefs-harness/tests/multi_record_publishes_of_transaction_zero.rs
index 1bcd009..6ae03cf 100644
--- a/crates/singlefs-harness/tests/multi_record_publishes_of_transaction_zero.rs
+++ b/crates/singlefs-harness/tests/multi_record_publishes_of_transaction_zero.rs
@@ -122,6 +122,13 @@ fn row_publish_whose_named_units_do_not_fit_one_record_spills_over_and_recovery_
             span: 1,
         },
     );
+    // mkfs 同一个进程里接着写的会话装上 mkfs 刚写下的根环（没装的分配器记根就断言，C551）。
+    allocator.install_root_ring_occupancy(
+        singlefs_core::make_filesystem::root_ring_occupancy_after_make_filesystem(
+            &publish_parameters,
+            &genesis,
+        ),
+    );
     let (row_publish, following) = {
         let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
         writer.set_journal_record_named_entry_capacity(
diff --git a/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs b/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs
index 4033ae4..52a91ae 100644
--- a/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs
+++ b/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs
@@ -177,6 +177,13 @@ fn build_pool(tag: &str) -> BuiltPool {
             span: 1,
         },
     );
+    // mkfs 同一个进程里接着写的会话装上 mkfs 刚写下的根环（没装的分配器记根就断言，C551）。
+    allocator.install_root_ring_occupancy(
+        singlefs_core::make_filesystem::root_ring_occupancy_after_make_filesystem(
+            &parameters,
+            &genesis,
+        ),
+    );
     let content = file_content();
     // 暖机两次发布之后录制流有几步：新池新建文件的写从这之后起（量出来，不按新池新建文件写几步倒推）。
     let warm_up_operation_count;
diff --git a/crates/singlefs-harness/tests/root_ring_turn_in_the_make_filesystem_session.rs b/crates/singlefs-harness/tests/root_ring_turn_in_the_make_filesystem_session.rs
new file mode 100644
index 0000000..4bb2650
--- /dev/null
+++ b/crates/singlefs-harness/tests/root_ring_turn_in_the_make_filesystem_session.rs
@@ -0,0 +1,94 @@
+//! mkfs 同一个进程那条会话（`common::build_pool`：取号、暖机、第一个文件，之后一次挂载都不做）里连着覆盖写、转过根环：
+//! 分配器装着 mkfs 刚写下的根环（C551（`build_pool` 的分配器没有根环表）；用户 2026-10-05 定 core 在没装根环表而记根时断言），
+//! 环转过之后按可再分配谓词回收（D16（发布语义） 已定项 1：已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)），池级 checker 一条不红——
+//! I-3.1（已分配统计对得上）是分配器不装根环表、一次不回收时红的那一条（E171 流三与 C551 的连续覆盖写都是这一形）。
+
+mod common;
+
+use common::{build_pool, FIXED_WRITE_TIME_SECONDS};
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+
+/// 覆盖写到这一条根为止：根环 3 × 8 = 24 槽，txg 26 那条根盖掉 txg 2 之后环里最旧有效根是 3，第一个文件那次（txg 3）换下的
+/// mkfs 树表从这里起可再分配；再往后写几条，让回收在好几条根上都发生过。
+const TXG_OF_THE_LAST_OVERWRITE: CheckpointTxg = CheckpointTxg(30);
+
+/// txg 30 那条根盖掉 txg 6 之后环里最旧的有效根（30 − 24 + 1）。
+const OLDEST_VALID_ROOT_AFTER_THE_LAST_OVERWRITE: CheckpointTxg = CheckpointTxg(7);
+
+/// 第 `overwrite_index` 次覆盖写的内容：每次长度与字节都不同。
+fn content_of_overwrite(overwrite_index: usize) -> Vec<u8> {
+    (0..3000 + overwrite_index * 37)
+        .map(|index| u8::try_from((index * 11 + overwrite_index * 5 + 1) % 251).expect("小于 256"))
+        .collect()
+}
+
+/// 不装根环表时（C551 那一形）：第一个文件那次释放的 mkfs 树表（释放代 3）起，被盖出根环的根引用过的落点一直留在 defer 里，
+/// txg 26 起 I-3.1 红；装了之后环每转一格就按谓词回收，到 txg 30 盘上的账与遍历全部有效根得到的对得上。
+#[test]
+fn overwrites_in_the_make_filesystem_session_past_one_turn_of_the_root_ring_reclaim_and_keep_every_invariant(
+) {
+    let mut pool = build_pool("root-ring-turn-in-the-make-filesystem-session");
+    assert_eq!(
+        pool.output.root.checkpoint_txg,
+        CheckpointTxg(3),
+        "前提：build_pool 交回第一个文件那一版（txg 3）"
+    );
+    let mut overwrite_index = 0usize;
+    // 迭代上界：每次覆盖写 txg 加一，从 3 到 30 共 27 次；跨轮携带的是现行那一版。
+    while pool.output.root.checkpoint_txg < TXG_OF_THE_LAST_OVERWRITE {
+        let previous = pool.output.clone();
+        pool.output = common::publish_overwrite_in_process(
+            &mut pool,
+            &previous,
+            &content_of_overwrite(overwrite_index),
+            FIXED_WRITE_TIME_SECONDS + 60,
+            InstanceGeneration(1),
+        )
+        .unwrap_or_else(|error| panic!("第 {} 次覆盖写：{error:?}", overwrite_index + 1));
+        overwrite_index += 1;
+    }
+    assert_eq!(
+        pool.output.root.checkpoint_txg, TXG_OF_THE_LAST_OVERWRITE,
+        "覆盖写到 txg 30，根环转过了一圈"
+    );
+    // 记了 txg 30 那条根之后环里是 txg 7–30（它盖掉 txg 6），最旧有效根是 7、F 是 0：释放代 ≤ 7 的都该回收了，
+    // defer 里只剩释放代比 7 新的。不装根环表时一个都不回收，释放代 3–7 的也还在 defer 里。
+    for device_map in &pool.allocator.devices {
+        let released_after_the_oldest_valid_root: u64 = pool
+            .allocator
+            .records()
+            .iter()
+            .filter(|record| {
+                record.device == device_map.device
+                    && record.is_released
+                    && record.generation > OLDEST_VALID_ROOT_AFTER_THE_LAST_OVERWRITE
+            })
+            .map(|record| u64::from(record.span_slots))
+            .sum();
+        assert_eq!(
+            device_map.deferred_slots(),
+            released_after_the_oldest_valid_root,
+            "盘 {:?}：分配器自己的账上 defer 里只剩释放代比环里最旧有效根（txg 7）新的槽",
+            device_map.device
+        );
+    }
+    let verdicts = check_pool_image(&pool.memory_pool());
+    assert_eq!(
+        verdicts
+            .iter()
+            .find(|(name, _)| *name == "I-3.1")
+            .map(|(_, verdict)| verdict),
+        Some(&InvariantVerdict::Holds),
+        "转过根环之后 I-3.1 判过且成立"
+    );
+    let violated: Vec<_> = verdicts
+        .iter()
+        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
+        .collect();
+    assert!(
+        violated.is_empty(),
+        "转过根环之后池级 checker 一条不红：{violated:?}"
+    );
+}
diff --git a/crates/singlefs-harness/tests/unequal_devices.rs b/crates/singlefs-harness/tests/unequal_devices.rs
index ceae603..30a6bee 100644
--- a/crates/singlefs-harness/tests/unequal_devices.rs
+++ b/crates/singlefs-harness/tests/unequal_devices.rs
@@ -134,6 +134,13 @@ fn build_unequal_pool(tag: &str) -> UnequalPool {
             span: 1,
         },
     );
+    // mkfs 同一个进程里接着写的会话装上 mkfs 刚写下的根环（没装的分配器记根就断言，C551）。
+    allocator.install_root_ring_occupancy(
+        singlefs_core::make_filesystem::root_ring_occupancy_after_make_filesystem(
+            &parameters(),
+            &genesis,
+        ),
+    );
     let content = file_content();
     let first = {
         let publish_parameters = parameters();
diff --git a/crates/singlefs-harness/tests/write_accounting_per_structure.rs b/crates/singlefs-harness/tests/write_accounting_per_structure.rs
index cd7a6ff..9e45793 100644
--- a/crates/singlefs-harness/tests/write_accounting_per_structure.rs
+++ b/crates/singlefs-harness/tests/write_accounting_per_structure.rs
@@ -285,6 +285,13 @@ fn publish_through_overwrite() -> PublishedThroughOverwrite {
             span: 1,
         },
     );
+    // mkfs 同一个进程里接着写的会话装上 mkfs 刚写下的根环（没装的分配器记根就断言，C551）。
+    allocator.install_root_ring_occupancy(
+        singlefs_core::make_filesystem::root_ring_occupancy_after_make_filesystem(
+            &parameters,
+            &genesis,
+        ),
+    );
     let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
     let instance = acquire_instance(&mut writer).expect("取号");
     assert_eq!(instance, InstanceGeneration(1));
````

### mutations-append.tsv

````text
C551：分配器没装根环表记根时静默返回（不断言）	crates/singlefs-core/src/allocator.rs	        let occupancy = self.root_ring.as_mut().expect(	        let Some(occupancy) = self.root_ring.as_mut() else {\n            return;\n        };\n        let _message_of_the_removed_assertion = (	-p singlefs-core --lib -- recording_a_root_on_an_allocator_without_a_root_ring_occupancy	recording_a_root_on_an_allocator_without_a_root_ring_occupancy_panics_instead_of_never_reclaiming
C551：记根时环转了也不回收（mkfs 同进程会话转过根环之后 defer 不放、I-3.1 红）	crates/singlefs-core/src/allocator.rs	            self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);	            {\n                let _ = (reclaim_floor, ReclaimedReuse::Immediately);\n                Vec::new()\n            };	-p singlefs-harness --test root_ring_turn_in_the_make_filesystem_session	overwrites_in_the_make_filesystem_session_past_one_turn_of_the_root_ring_reclaim_and_keep_every_invariant
C551：build_pool 不装根环表	crates/singlefs-harness/tests/common/mod.rs	    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(\n        &parameters,\n        &genesis,\n    ));	    let _ = root_ring_occupancy_after_make_filesystem(&parameters, &genesis);	-p singlefs-harness --test root_ring_turn_in_the_make_filesystem_session	overwrites_in_the_make_filesystem_session_past_one_turn_of_the_root_ring_reclaim_and_keep_every_invariant
````

## m3-formal-31-3

### crates.patch

````diff
diff --git a/crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs b/crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs
index fb6c57a..94b22ed 100644
--- a/crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs
+++ b/crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs
@@ -5024,7 +5024,8 @@ mod tests {
     };
     use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
     use singlefs_core::make_filesystem::{
-        make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
+        make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
+        TREE_TABLE_GENESIS_SLOT,
     };
     use singlefs_core::transaction::{
         acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, TransactionOutput,
@@ -5065,6 +5066,12 @@ mod tests {
                 span: 1,
             },
         );
+        // mkfs 同一个进程里接着写的会话装上 mkfs 刚写下的根环（C551：没装的分配器记根就断言）。用它的单测最多记到 txg 15，
+        // 环转不过一格，一个落点都不回收，钉的数与不装时相同。
+        allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
+            &parameters,
+            &genesis,
+        ));
         let instance = {
             let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
             acquire_instance(&mut pool).expect("取号")
diff --git a/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs b/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs
index cdfbd2b..1a34bcc 100644
--- a/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs
+++ b/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs
@@ -46,7 +46,9 @@ use singlefs_core::journal::{
     back_chain_of, record_offset, JournalRecord, JournalRecordOrdinalWithinPublish,
     JournalRecordPlaceInPublish, JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET,
 };
-use singlefs_core::make_filesystem::{INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
+use singlefs_core::make_filesystem::{
+    root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
+};
 use singlefs_core::mount::{
     mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, RollbackTarget,
     ShadowLedger, UnmountRaisedTheFloor, Unmounted,
@@ -287,6 +289,12 @@ fn drive_through_the_new_pool_file_creation(
             span: 1,
         },
     );
+    // 照 `build_pool` 装上 mkfs 刚写下的根环（C551：没装的分配器记根就断言）。流一、流二在这个分配器上只记到 txg 4，
+    // 环转不过一格，一个落点都不回收，写流与不装时相同。
+    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
+        &parameters,
+        &genesis,
+    ));
     let content = local_first_file_content();
     let (warmed, output, warm_up_operation_count) = {
         let mut pool = PoolWriter::new(&parameters, &mut devices);
diff --git a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs
index 59ae61f..5eb7be0 100644
--- a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs
+++ b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs
@@ -41,7 +41,9 @@ use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
 };
 use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
-use singlefs_core::make_filesystem::{INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
+use singlefs_core::make_filesystem::{
+    root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
+};
 use singlefs_core::mount::{
     mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, RollbackTarget,
     ShadowLedger, UnmountRaisedTheFloor, Unmounted,
@@ -630,6 +632,10 @@ impl LiveStream {
                 span: 1,
             },
         );
+        allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
+            &common::parameters(),
+            &formatted.genesis,
+        ));
         let driver = StreamDriver::new(
             name,
             formatted.stream.clone(),
diff --git a/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs b/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs
index 2e25c2d..51c8766 100644
--- a/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs
+++ b/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs
@@ -18,9 +18,9 @@ use singlefs_checker_tier::device_log::{
     fold_declared_zero_fills, parse_device_log, DeviceEvent, DeviceLog,
 };
 use singlefs_checker_tier::on_device_modes::{
-    allocator_rebuilt_from_the_records_of, publish_the_fourth_version, publish_the_second_version,
-    publish_the_third_version, raise_the_rollback_floor_to_its_ceiling, OnDeviceRunMode,
-    PublishesAfterTheNewPoolFileCreation,
+    allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process,
+    publish_the_fourth_version, publish_the_second_version, publish_the_third_version,
+    raise_the_rollback_floor_to_its_ceiling, OnDeviceRunMode, PublishesAfterTheNewPoolFileCreation,
 };
 use singlefs_core::address::DeviceIdentity;
 use singlefs_core::allocator::PoolAllocator;
@@ -152,7 +152,11 @@ fn rerun_the_program_on_the_host(
         PublishesAfterTheNewPoolFileCreation::SecondVersion
         | PublishesAfterTheNewPoolFileCreation::SecondVersionThenSecondInstance
         | PublishesAfterTheNewPoolFileCreation::SecondVersionThenSecondInstanceThenFourthVersionThenRaiseOfTheRollbackFloor => {
-            let mut allocator = allocator_rebuilt_from_the_records_of(&devices, &run.output);
+            let mut allocator = allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process(
+                parameters,
+                &devices,
+                &run,
+            );
             publish_the_second_version(parameters, &mut devices, &mut allocator, &run.output)
                 .map_err(|failed| format!("发布 B：{:?}", failed.cause))?;
             window_ends.push((ProgramWindow::FileOverwrite, stream.operation_count()));
diff --git a/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs b/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs
index 0fa7a50..30a6b83 100644
--- a/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs
+++ b/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs
@@ -29,9 +29,10 @@
 use std::path::Path;
 
 use singlefs_checker_tier::on_device_modes::{
-    allocator_rebuilt_from_the_records_of, publish_the_fourth_version, publish_the_second_version,
-    publish_the_third_version, raise_the_rollback_floor_to_its_ceiling, FailedPublish,
-    OnDeviceRunMode, PublishesAfterTheNewPoolFileCreation,
+    allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process,
+    publish_the_fourth_version, publish_the_second_version, publish_the_third_version,
+    raise_the_rollback_floor_to_its_ceiling, FailedPublish, OnDeviceRunMode,
+    PublishesAfterTheNewPoolFileCreation,
 };
 use singlefs_core::address::DeviceIdentity;
 use singlefs_core::allocator::PoolAllocator;
@@ -54,7 +55,7 @@ use singlefs_harness::fault_injection::{
 };
 use singlefs_harness::scenario::{
     e142_parameters, run_new_pool_file_creation, NewPoolFileCreationPathFailure,
-    NewPoolFileCreationPathStep, ScenarioPoint,
+    NewPoolFileCreationPathStep, NewPoolFileCreationRun, ScenarioPoint,
 };
 use singlefs_harness::segments::{
     closed_form_state_count, segment_kinds_text, segment_sizes_text, split_into_segments,
@@ -594,7 +595,8 @@ struct FileOverwriteRun {
     window_matches_device: bool,
 }
 
-/// 发布 B：同一个进程里再覆盖写一次，分配器从新池新建文件的分配记录重建（与可写挂载同一条路）；两次快照之差就是这一次发布
+/// 发布 B：同一个进程里再覆盖写一次，分配器从新池新建文件的分配记录重建（与可写挂载同一条路）、装上这个进程的根环
+/// （`allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process`，只在内存里拼、不读盘）；两次快照之差就是这一次发布
 /// 交给设备的调用数，挂钟只计发布本身。
 ///
 /// # Errors
@@ -605,11 +607,14 @@ fn publish_the_second_version_and_describe<Inner: BlockDevice>(
     plan: &SharedFaultPlan,
     stream: &SharedStream,
     geometry: &FixedGeometry,
-    first_version: &TransactionOutput,
+    first_run: &NewPoolFileCreationRun,
 ) -> Result<FileOverwriteRun, FailedRun> {
+    let first_version = &first_run.output;
     let before = device_call_counts(devices, plan);
     let operations_before = stream.operations().len();
-    let mut allocator = allocator_rebuilt_from_the_records_of(devices, first_version);
+    let mut allocator = allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process(
+        parameters, devices, first_run,
+    );
     let started = Instant::now();
     let published = publish_the_second_version(parameters, devices, &mut allocator, first_version);
     let nanoseconds = started.elapsed().as_nanos();
@@ -1601,7 +1606,7 @@ fn main() {
                 &plan,
                 &stream,
                 &geometry,
-                &run.output,
+                &run,
             )
             .unwrap_or_else(|failed| exit_after_a_failed_run(&mut emitter, &failed));
             for line in &second.lines {
@@ -1757,8 +1762,8 @@ mod tests {
         DeviceCallCounts, SegmentClock,
     };
     use singlefs_checker_tier::on_device_modes::{
-        allocator_rebuilt_from_the_records_of, fourth_file_content, publish_the_second_version,
-        third_file_content, OnDeviceRunMode,
+        allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process, fourth_file_content,
+        publish_the_second_version, third_file_content, OnDeviceRunMode,
     };
     use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
     use singlefs_core::block_device::PhysicalBlockSizeInBytes;
@@ -1818,7 +1823,11 @@ mod tests {
         let run =
             run_new_pool_file_creation(&parameters, &mut devices, &stream, |_point, _devices| {})
                 .expect("新池新建文件");
-        let mut allocator = allocator_rebuilt_from_the_records_of(&devices, &run.output);
+        let mut allocator = allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process(
+            &parameters,
+            &devices,
+            &run,
+        );
         let second =
             publish_the_second_version(&parameters, &mut devices, &mut allocator, &run.output)
                 .expect("发布 B");
@@ -2354,7 +2363,7 @@ mod tests {
                 &plan,
                 &stream,
                 &geometry,
-                &run.output,
+                &run,
             ),
             "发布 B",
         );
@@ -2378,7 +2387,11 @@ mod tests {
         let run =
             run_new_pool_file_creation(&parameters, &mut devices, &stream, |_point, _devices| {})
                 .expect("新池新建文件");
-        let mut allocator = allocator_rebuilt_from_the_records_of(&devices, &run.output);
+        let mut allocator = allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process(
+            &parameters,
+            &devices,
+            &run,
+        );
         publish_the_second_version(&parameters, &mut devices, &mut allocator, &run.output)
             .expect("发布 B");
         let mut clock = SegmentClock::start();
@@ -2572,7 +2585,12 @@ mod tests {
                 |_point, _devices| {},
             )
             .expect("新池新建文件");
-            let mut allocator = allocator_rebuilt_from_the_records_of(&devices, &run.output);
+            let mut allocator =
+                allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process(
+                    &parameters,
+                    &devices,
+                    &run,
+                );
             publish_the_second_version(&parameters, &mut devices, &mut allocator, &run.output)
                 .expect("发布 B");
             let mut clock = SegmentClock::start();
@@ -2617,7 +2635,9 @@ mod tests {
         let run =
             run_new_pool_file_creation(parameters, &mut devices, stream, |_point, _devices| {})
                 .expect("新池新建文件");
-        let mut allocator = allocator_rebuilt_from_the_records_of(&devices, &run.output);
+        let mut allocator = allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process(
+            parameters, &devices, &run,
+        );
         publish_the_second_version(parameters, &mut devices, &mut allocator, &run.output)
             .expect("发布 B");
         let switched = match switch_instance_and_publish_third_version(
@@ -3055,7 +3075,7 @@ mod tests {
             &plan,
             &stream,
             &geometry,
-            &run.output,
+            &run,
         );
         assert!(second.is_ok_and(|second| second.window_matches_device));
         let operations_after_the_file_overwrite = stream.operation_count();
diff --git a/crates/singlefs-checker-tier/src/on_device_modes.rs b/crates/singlefs-checker-tier/src/on_device_modes.rs
index 8eabb8a..77a956d 100644
--- a/crates/singlefs-checker-tier/src/on_device_modes.rs
+++ b/crates/singlefs-checker-tier/src/on_device_modes.rs
@@ -5,21 +5,25 @@
 //! 抬 F 抬到多少，两边不各抄一份，宿主重跑的才是同参数同字节的那条写路（里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 30 行：宿主那一侧要接着重跑发布 B、
 //! 可写挂载与发布 C，把程序录制流与设备侧日志逐项比；第 58 行：抬 F 那一串发布同样要在真设备上跑、同样逐项比）。设备侧日志与这里不共享一行代码。
 
-use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
-use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator};
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator, RootRingOccupancy, RootRingOccupant};
 use singlefs_core::block_device::BlockDevice;
-use singlefs_core::make_filesystem::MakeFilesystemParameters;
+use singlefs_core::make_filesystem::{MakeFilesystemParameters, ROLLBACK_FLOOR_AT_MKFS};
 use singlefs_core::mount::{
     raise_rollback_floor, ring_slots_known_to_hold_a_root_by, rollback_floor_ceiling, MountError,
     RaisedFloor, ShadowLedger,
 };
 use singlefs_core::recovery::{choose_system_configuration, instance_table_chain_of_root};
+use singlefs_core::root_ring::RootRingSlot;
 use singlefs_core::transaction::{
     publish_overwrite, FirstFile, PoolWriter, PublishError, TransactionOutput,
 };
 use singlefs_core::write_accounting::WritesByStructureKind;
+use singlefs_format::ROOT_RING_REGIONS;
 
-use singlefs_harness::scenario::{first_file_content, FIXED_WRITE_TIME_SECONDS};
+use singlefs_harness::scenario::{
+    first_file_content, NewPoolFileCreationRun, FIXED_WRITE_TIME_SECONDS,
+};
 
 /// 虚机档的六个模式（命令行最后一个参数）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
@@ -167,19 +171,67 @@ pub struct FailedPublish {
     pub writes_of_failed_publishes: Vec<WritesByStructureKind>,
 }
 
-/// 发布 B 的分配器：从新池新建文件的分配记录重建（与可写挂载同一条路）。
+/// 发布 B 的分配器：从新池新建文件的分配记录重建（与可写挂载同一条路），装上这个进程此刻眼里的根环。
+///
+/// 发布 B 与新池新建文件在同一个进程里，根环表与 `run_new_pool_file_creation` 里那个分配器记到的是同一张：从 mkfs 同进程会话那一张起
+/// （`make_filesystem::root_ring_occupancy_after_make_filesystem`：三个区域的槽 0 各一条第 0 代根，F 取 mkfs 写的，记到第 0 代根），
+/// 再按次序记上这个进程写过的根——暖机那几条（`run.warm_up.roots`）与新池新建文件那一条（`run.output.root`）。
+/// 不装根环表，发布 B 记根时断言（C551）；只装 mkfs 那一张、不补记，记 txg 4 时撞跳号断言。
+/// `run_new_pool_file_creation` 不交回 mkfs 的输出（`MakeFilesystemOutput`），这里调不了 core 那个函数，照它的两样现拼：
+/// 第 0 代根的 txg 取暖机第一条根的减一（暖机接在第 0 代根上、txg 逐个加一），F 取 `ROLLBACK_FLOOR_AT_MKFS`。
+///
+/// # Panics
+/// `run` 里暖机一条根都没有，或第一条根的 txg 是 0（`run_new_pool_file_creation` 做成时暖机两条、txg 1 与 2）；
+/// 补记的根 txg 不逐个加一（core 记根的跳号断言）。
 #[must_use]
-pub fn allocator_rebuilt_from_the_records_of<Device: BlockDevice>(
+pub fn allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process<
+    Device: BlockDevice,
+>(
+    parameters: &MakeFilesystemParameters,
     devices: &[(DeviceIdentity, Device)],
-    version: &TransactionOutput,
+    run: &NewPoolFileCreationRun,
 ) -> PoolAllocator {
-    PoolAllocator::rebuild_from_records(
+    let mut allocator = PoolAllocator::rebuild_from_records(
         devices
             .iter()
             .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
             .collect(),
-        version.allocation_records.clone(),
-    )
+        run.output.allocation_records.clone(),
+    );
+    let genesis_txg = CheckpointTxg(
+        run.warm_up
+            .roots
+            .first()
+            .expect("run_new_pool_file_creation 做成时暖机写了两条根")
+            .checkpoint_txg
+            .0
+            .checked_sub(1)
+            .expect("暖机第一条根接在第 0 代根上，txg 不是 0"),
+    );
+    allocator.install_root_ring_occupancy(RootRingOccupancy::read_from_the_ring(
+        parameters.geometry.root_ring_slots_per_region,
+        (0..ROOT_RING_REGIONS)
+            .map(|region| {
+                (
+                    RootRingSlot { region, slot: 0 },
+                    RootRingOccupant::ValidRoot {
+                        checkpoint_txg: genesis_txg,
+                    },
+                )
+            })
+            .collect(),
+        ROLLBACK_FLOOR_AT_MKFS,
+        genesis_txg,
+    ));
+    for root in run
+        .warm_up
+        .roots
+        .iter()
+        .chain(std::iter::once(&run.output.root))
+    {
+        allocator.record_root_written_by_this_process(root.checkpoint_txg);
+    }
+    allocator
 }
 
 /// 发布 B：接在新池新建文件那一版上覆盖写第二版，同一个实例。
diff --git a/crates/singlefs-harness/tests/overwrite_in_one_instance.rs b/crates/singlefs-harness/tests/overwrite_in_one_instance.rs
index 72c9f98..27dc48f 100644
--- a/crates/singlefs-harness/tests/overwrite_in_one_instance.rs
+++ b/crates/singlefs-harness/tests/overwrite_in_one_instance.rs
@@ -5,7 +5,7 @@
 
 mod common;
 
-use std::collections::BTreeSet;
+use std::collections::{BTreeMap, BTreeSet};
 
 use common::{
     build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS,
@@ -20,7 +20,7 @@ use singlefs_core::admission::{AvailableBytesOnOneDevice, BytesOnOneDevice, Devi
 use singlefs_core::allocation_record_tree::{
     AllocationRecordTreeNode, AllocationRecordTreeNodePosition,
 };
-use singlefs_core::allocator::UnitFootprint;
+use singlefs_core::allocator::{AllocationRecord, UnitFootprint};
 use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
 use singlefs_core::journal::{back_chain_of, record_offset};
 use singlefs_core::mounted_read::mount_read_only;
@@ -34,7 +34,7 @@ use singlefs_core::recovery::{
     RecoveryFailure, RecoveryOutcome,
 };
 use singlefs_core::root_record::RootRecord;
-use singlefs_core::root_ring::{slot_offset, target_for_publish};
+use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
 use singlefs_core::transaction::{
     mapping_locations_for_key, placements_to_release_via_mapping, publish_overwrite, FirstFile,
     MappingLookup, PoolWriter, PublishError, TransactionOutput, TransactionUnit,
@@ -43,7 +43,9 @@ use singlefs_core::transaction::{
 use singlefs_core::unit::{
     build_index_node, data_unit_payload_capacity, index_node_entry_capacity, parse_index_node,
 };
-use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES, UNIT_AREA_START_SLOT};
+use singlefs_format::{
+    JOURNAL_RING_DEFAULT_BYTES, ROOT_RING_REGIONS, SLOT_BYTES, UNIT_AREA_START_SLOT,
+};
 use singlefs_harness::memory_pool::MemoryPool;
 use singlefs_harness::segments::{segment_kinds_text, segment_sizes_text, split_into_segments};
 
@@ -666,27 +668,77 @@ fn repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves
     assert_eq!(content, second_content(), "冷启动读回最后一次覆盖写的内容");
 }
 
-/// 分配记录过 600 条时记账的「已分配」照样等于分配记录的跨度之和（增补 3 第 2 件代码三方第二轮判决第三节第 3 条：攻方变异 m4b——
+/// 这个进程从第 0 代根起每个 txg 写一条根、写到 `newest_txg` 之后，根环里最旧的那条根的 txg：mkfs 在三个区域的槽 0 各写一条第 0 代根
+/// （`make_filesystem` 写的那三条），之后第 n 条根落在 `target_for_publish` 给的槽上、盖掉那槽原来的，环里每槽留最后写到它的那一条。
+/// 一个实例、没有被抛弃的根，环里每一条都有效，所以它就是可再分配谓词里的「环里最旧有效根」（D16（发布语义） 已定项 1）。
+/// 这里从 txg 现算，不读分配器的根环表。
+fn oldest_root_in_the_ring_after_this_process_wrote_every_txg_up_to(
+    newest_txg: CheckpointTxg,
+    genesis_txg: CheckpointTxg,
+) -> CheckpointTxg {
+    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
+    let mut txg_of_each_ring_slot: BTreeMap<RootRingSlot, CheckpointTxg> = (0..ROOT_RING_REGIONS)
+        .map(|region| (RootRingSlot { region, slot: 0 }, genesis_txg))
+        .collect();
+    // 迭代上界：newest_txg − genesis_txg 次（这条用例里不过两百多）；不带提前出口。
+    for written_txg in genesis_txg.0 + 1..=newest_txg.0 {
+        txg_of_each_ring_slot.insert(
+            target_for_publish(CheckpointTxg(written_txg), slots_per_region),
+            CheckpointTxg(written_txg),
+        );
+    }
+    *txg_of_each_ring_slot
+        .values()
+        .min()
+        .expect("环里至少有 mkfs 写的那几槽")
+}
+
+/// 分配记录过 600 条时记账的「已分配」照样等于还没回收的分配记录的跨度之和（增补 3 第 2 件代码三方第二轮判决第三节第 3 条：攻方变异 m4b——
 /// 分配记录过 600 条之后「已分配」少记一槽——整个 workspace 没有一条测试会红）。同一个进程同一个实例里从第一个文件起连着覆盖写，
-/// 直到越过 900 条（分配记录树按位置寻址之后没有 812 条那道墙，D8（核心索引结构） 已定项 14）：没有挂载、没有抬 F，一个落点都不回收，
-/// 已释放的都还在 defer 窗口里、仍算已分配（I-3.1 读法甲），所以每一版每盘的「已分配」= 这一版交回的分配记录里那块盘的跨度之和 × 16 KiB
-/// ——拿分配记录现算，不看分配器的计数。逐版核，过 600 条的版本至少 5 个。每一次覆盖写接着上一版，次序本身就是被测对象，不切片并行。
+/// 直到越过 812 条（一片叶的条目容量；分配记录树按位置寻址之后那道墙拆了，D8（核心索引结构） 已定项 14）。停在 812 而不再往上取：
+/// 回收开始之后复用的落点改写原来那条记录、不再追加，记录涨得慢，200 次覆盖写之后是 882 条（2026-10-05 在装了根环表的 `build_pool` 上跑的）。
+/// 没有挂载、没有抬 F（F 恒 0）；
+/// `build_pool` 的分配器装着 mkfs 的根环（C551），根环转过一格之后照可再分配谓词回收（D16（发布语义） 已定项 1：已释放 ∧ 释放代 ≤
+/// max(F, 环里最旧有效根)）。回收的落点那条记录不删、照旧写着已释放，留到它被重新分配时改写（D3（空间分配） 已定项 7「条目留到该落点
+/// 被重新分配时覆盖」），回收的槽不再算已分配；还在 defer 窗口里的已释放落点仍算已分配（I-3.1 读法甲）。所以每一版每盘的「已分配」=
+/// 这一版交回的分配记录里那块盘上不是已回收的记录的跨度之和 × 16 KiB，「已回收」= 已释放且释放代 ≤ max(这一版的 F, 写到这一版时环里最旧的根)
+/// ——拿分配记录与根的 txg 现算，不看分配器的计数。逐版核，过 600 条的版本至少 5 个，其中带已回收记录的至少 1 个（判法里减掉已回收那一半真走到）。
+/// 每一次覆盖写接着上一版，次序本身就是被测对象，不切片并行。
 #[test]
-fn allocated_statistic_equals_the_span_sum_of_the_allocation_records_past_six_hundred_records() {
+fn allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records(
+) {
     let mut pool = build_pool("allocated-statistic-past-six-hundred-records");
+    let genesis_txg = pool.genesis.root.checkpoint_txg;
     let mut versions_past_six_hundred_records = 0;
+    let mut versions_carrying_reclaimed_records = 0;
     let mut overwrites = 0;
-    // 迭代上界 200 次（每次至少加 16 条）；跨轮携带的是现行那一版。
-    while pool.output.allocation_records.len() <= 900 && overwrites < 200 {
+    // 迭代上界 200 次（回收开始之前每次至少加 16 条，之后有的落点改写原来的记录）；跨轮携带的是现行那一版。
+    while pool.output.allocation_records.len() <= 812 && overwrites < 200 {
         let previous = pool.output.clone();
         let output = try_overwrite(&mut pool, &previous)
             .unwrap_or_else(|error| panic!("第 {} 次覆盖写：{error:?}", overwrites + 1));
         overwrites += 1;
+        let reclaim_floor = output.root.rollback_floor.max(
+            oldest_root_in_the_ring_after_this_process_wrote_every_txg_up_to(
+                output.root.checkpoint_txg,
+                genesis_txg,
+            ),
+        );
+        let is_reclaimed =
+            |record: &&AllocationRecord| record.is_released && record.generation <= reclaim_floor;
+        if output
+            .allocation_records
+            .iter()
+            .any(|record| is_reclaimed(&record))
+        {
+            versions_carrying_reclaimed_records += 1;
+        }
         for device in [DeviceIdentity(0), DeviceIdentity(1)] {
             let span_sum_in_slots: u64 = output
                 .allocation_records
                 .iter()
                 .filter(|record| record.device == device)
+                .filter(|record| !is_reclaimed(record))
                 .map(|record| u64::from(record.span_slots))
                 .sum();
             let allocated_bytes = output
@@ -700,8 +752,9 @@ fn allocated_statistic_equals_the_span_sum_of_the_allocation_records_past_six_hu
             assert_eq!(
                 allocated_bytes,
                 span_sum_in_slots * SLOT_BYTES,
-                "第 {overwrites} 次覆盖写（{} 条分配记录）盘 {}：「已分配」要等于这一版分配记录的跨度之和",
+                "第 {overwrites} 次覆盖写（{} 条分配记录，回收门槛 txg {}）盘 {}：「已分配」要等于这一版不是已回收的分配记录的跨度之和",
                 output.allocation_records.len(),
+                reclaim_floor.0,
                 device.0
             );
         }
@@ -711,7 +764,7 @@ fn allocated_statistic_equals_the_span_sum_of_the_allocation_records_past_six_hu
         pool.output = output;
     }
     assert!(
-        pool.output.allocation_records.len() > 900,
+        pool.output.allocation_records.len() > 812,
         "{overwrites} 次覆盖写之后 {} 条",
         pool.output.allocation_records.len()
     );
@@ -719,6 +772,10 @@ fn allocated_statistic_equals_the_span_sum_of_the_allocation_records_past_six_hu
         versions_past_six_hundred_records >= 5,
         "过 600 条的版本 {versions_past_six_hundred_records} 个"
     );
+    assert!(
+        versions_carrying_reclaimed_records >= 1,
+        "带已回收记录的版本 {versions_carrying_reclaimed_records} 个：根环一次都没转过，判法里减掉已回收那一半没走到"
+    );
 }
 
 /// 可再分配谓词（D16（发布语义） 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」）：F 不抬（恒 0）时一个已释放的落点都回不来——
````

### mutations-append.tsv

````text
C551 后：回收时「已分配」不减（mark_reclaimed 只改空闲与 defer，已分配照旧算着已回收的槽；转过根环之后钉「已分配」等于没回收的记录跨度之和的那条用例判出）	crates/singlefs-core/src/allocator.rs	        self.allocated_slots -= span;\n        self.deferred_slots -= span;\n        self.free_slots += span;	        self.deferred_slots -= span;\n        self.free_slots += span;	-p singlefs-harness --test overwrite_in_one_instance -- allocated_statistic_equals	allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records
C551 E170 装置：流一驱动不装根环表（回到照抄旧 build_pool 的那一形；仓里 core 在第一个文件那次发布记根时断言）	crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs	    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(\n        &parameters,\n        &genesis,\n    ));\n    let content = local_first_file_content();	    let content = local_first_file_content();	-p singlefs-checker-tier --bin e170_o3_spec_executor_export -- the_second_run_table_differs_from_the_first_only_in_the_p8_expectation_and_a_final_p10_row_on_the_p9_base	the_second_run_table_differs_from_the_first_only_in_the_p8_expectation_and_a_final_p10_row_on_the_p9_base
C551 E156 装置：内联单测的新池新建文件搭建不装根环表（仓里 core 在第一个文件那次发布记根时断言）	crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs	        allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(\n            &parameters,\n            &genesis,\n        ));\n        let instance = {	        let instance = {	-p singlefs-checker-tier --bin e156_allocation_basis_counts -- accounting_after_new_pool_file_creation_matches_the_registered_anchor	accounting_after_new_pool_file_creation_matches_the_registered_anchor
C551 on_device_modes：发布 B 的分配器不装根环表（仓里 core 在发布 B 记根时断言）	crates/singlefs-checker-tier/src/on_device_modes.rs	    allocator.install_root_ring_occupancy(RootRingOccupancy::read_from_the_ring(	    drop(RootRingOccupancy::read_from_the_ring(	-p singlefs-checker-tier --bin new_pool_file_creation_on_device -- second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches	second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches
C551 on_device_modes：发布 B 的分配器装了 mkfs 那张根环表、不补记暖机与新池新建文件那几条根（发布 B 记 txg 4 时撞 core 的跳号断言）	crates/singlefs-checker-tier/src/on_device_modes.rs	        allocator.record_root_written_by_this_process(root.checkpoint_txg);	        let _ = root;	-p singlefs-checker-tier --bin new_pool_file_creation_on_device -- second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches	second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches
````

### mutations-replacements.tsv

````text
增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4b 同一处）：分配记录过 600 条之后记账「已分配」少记一槽；直接钉已分配统计的那条用例判出（树分裂换锚点）	crates/singlefs-core/src/transaction.rs	                PerDeviceAccountingRow::AllocatedBytes => device_map.allocated_slots() * SLOT_BYTES,	                PerDeviceAccountingRow::AllocatedBytes => (device_map.allocated_slots() - u64::from(allocator.records().len() > 600)) * SLOT_BYTES,	-p singlefs-harness --test overwrite_in_one_instance -- allocated_statistic_equals	allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records
````

## m3-formal-31-2

### crates.patch

````diff
diff --git a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
index 6bfee63..3dd188c 100644
--- a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
+++ b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
@@ -61,9 +61,12 @@ use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InodeNumber, InstanceGeneration,
 };
 use singlefs_core::admission::SpaceAdmission;
-use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
+use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator, RootRingOccupancy};
 use singlefs_core::journal::JournalRecordPlaceInPublish;
-use singlefs_core::make_filesystem::{INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
+use singlefs_core::make_filesystem::{
+    root_ring_occupancy_after_make_filesystem, MakeFilesystemOutput, INSTANCE_TABLE_SLOT,
+    TREE_TABLE_GENESIS_SLOT,
+};
 use singlefs_core::mount::{
     mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, RollbackTarget,
     ShadowLedger, UnmountRaisedTheFloor, Unmounted,
@@ -1083,6 +1086,30 @@ struct LiveStream {
     split_publish: Option<usize>,
 }
 
+/// S1 启动核对 `<流>_allocator_carries_the_root_ring_of_make_filesystem`（登记修订 6 第 6.2 节第 1 条）：分配器装着的根环表与
+/// `root_ring_occupancy_after_make_filesystem` 给这次 mkfs 算出的那一份 `{:?}` 文本相同。`RootRingOccupancy` 只派生 `Clone, Debug`，
+/// 所以按文本比；两边记的是文本的 sha256（文本长、带空白，结果行里只放摘要），没装记 `None`。
+fn allocator_carries_the_root_ring_of_make_filesystem_check(
+    name: StreamName,
+    allocator: &PoolAllocator,
+    genesis: &MakeFilesystemOutput,
+) -> StopCheck {
+    let digest_of_the_debug_text =
+        |occupancy: &RootRingOccupancy| sha256_hexadecimal(debug_text(occupancy).as_bytes());
+    StopCheck::new(
+        &format!(
+            "{}_allocator_carries_the_root_ring_of_make_filesystem",
+            name.directory_name()
+        ),
+        allocator
+            .root_ring_occupancy()
+            .map(digest_of_the_debug_text),
+        Some(digest_of_the_debug_text(
+            &root_ring_occupancy_after_make_filesystem(&common::parameters(), genesis),
+        )),
+    )
+}
+
 impl LiveStream {
     fn start(name: StreamName, tag: &str, switches: RunSwitches) -> Self {
         let formatted = common::format_pool(tag);
@@ -1100,6 +1127,17 @@ impl LiveStream {
                 span: 1,
             },
         );
+        // 登记修订 6 第 6.2 节第 1 条：mkfs 同一个进程里接着写的会话照 core 部署的那一种，装上 mkfs 刚写下的根环表
+        // （转过根环之后照 D16（发布语义） 已定项 1 的可再分配谓词回收）；空闲图照旧用 `DeviceFreeMap::new`（S1 回比 `build_pool`、S2 比 E170）。
+        allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
+            &common::parameters(),
+            &formatted.genesis,
+        ));
+        let root_ring_check = allocator_carries_the_root_ring_of_make_filesystem_check(
+            name,
+            &allocator,
+            &formatted.genesis,
+        );
         let driver = StreamDriver::new(
             name,
             formatted.stream.clone(),
@@ -1115,7 +1153,7 @@ impl LiveStream {
             output: None,
             mount_output_text: None,
             versions: Vec::new(),
-            checks: Vec::new(),
+            checks: vec![root_ring_check],
             warm_up_operation_count: 0,
             unlink_calls: 0,
             inode_creations: 0,
@@ -3895,7 +3933,8 @@ struct ObservationContext<'stream> {
     reference_filesystem: &'stream ReferenceFileSystemStream,
     /// 写表每一项落在哪一次调用里（[`call_index_of_each_write`]，整条流算一次）。
     call_of_write: Vec<Option<usize>>,
-    /// 阳性对照「判态路 panic 单列」的注入（只有那一遍有，别的都是 None）。
+    /// 判态路 panic 的注入：跑法里只有阳性对照「判态路 panic 单列」那一遍有，别的都是 None；内联用例 U13–U16 另经
+    /// [`ConfirmedLossPiecePanicInjections`] 交给 Q5 的对拍、阳性对照与取样点导出（登记修订 6 第 6.2 节第 3 条）。
     piece_panic_injection: Option<&'stream PiecePanicInjection>,
 }
 
@@ -5011,10 +5050,17 @@ fn export_stream(
         selection,
         started,
         drive_nanoseconds,
+        None,
     )
 }
 
 /// 一条驱动完的流导出：枚举、取样、判、写导出文件、S4 重建核对。`check_against_the_crates` 只在基线那一趟开（S1 的状态数那几条）。
+/// `piece_panic_injection` 交给判态的 [`ObservationContext`]：只有 Q5 主取样点、反向取样点那一次导出可能不空（登记修订 6 第 6.2 节第 3 条，
+/// 跑法里也是空，只有内联用例 U15、U16 交），别的调用处一律交空。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "八个参数各有名字、各有类型，凑不成一个带不变量的概念（code-discipline.md「参数不按个数限」）"
+)]
 fn export_prepared_stream(
     stream: &PreparedStream,
     directory: &Path,
@@ -5023,6 +5069,7 @@ fn export_prepared_stream(
     selection: StateSelection,
     started: Instant,
     drive_nanoseconds: u128,
+    piece_panic_injection: Option<&PiecePanicInjection>,
 ) -> StreamExport {
     std::fs::create_dir_all(directory)
         .unwrap_or_else(|error| panic!("建 {}：{error}", directory.display()));
@@ -5161,7 +5208,7 @@ fn export_prepared_stream(
         torn_image_index_by_write: &torn_image_index_by_write,
         reference_filesystem: &reference_filesystem,
         call_of_write: call_index_of_each_write(stream),
-        piece_panic_injection: None,
+        piece_panic_injection,
     };
     let observe_started = Instant::now();
     let observations = observe_sampled_states(&context, &sampled, switches, consulted_reports);
@@ -7987,18 +8034,38 @@ fn run_confirmed_loss_form(command_line: &ConfirmedLossCommandLine) -> i32 {
             shape,
             drive_nanoseconds,
             pairing,
+            // 跑法里三格一律空、命令行不开放（登记修订 6 第 6.2 节第 3 条；修订 5 第 5.1 节「注入只经装置内部的开关进」）。
+            piece_panic_injections: ConfirmedLossPiecePanicInjections {
+                pairing: None,
+                piece_panic_control: None,
+                sampling_point_export: None,
+            },
         },
         failed_checks,
     )
 }
 
-/// Q5 那一次判态要的两形：主取样点（或反向取样点）那一形与它的造法下标、驱动它的耗时，与对拍那一形（反向取样点没有）。
+/// Q5 那一次判态要的两形：主取样点（或反向取样点）那一形与它的造法下标、驱动它的耗时，与对拍那一形（反向取样点没有）；
+/// 另带判态路 panic 的三格注入（跑法里都空）。
 struct ConfirmedLossShapes {
     sampling_point: &'static str,
     main: PreparedStream,
     shape: ConfirmedLossShape,
     drive_nanoseconds: u128,
     pairing: Option<PreparedStream>,
+    piece_panic_injections: ConfirmedLossPiecePanicInjections,
+}
+
+/// Q5 判决路的注入入口（登记修订 6 第 6.2 节第 3 条；判决 b29 第二节 L1 末行、攻方改法 A1、A2）：
+/// [`judge_the_confirmed_loss_shapes`] 里收 unattributed 那四个调用点前面的三处判态各一格，可空。
+/// 只有内联用例 U13–U16 交非空的，跑法（[`run_confirmed_loss_form`]）一律交三格空。
+struct ConfirmedLossPiecePanicInjections {
+    /// 对拍那一遍：空时与 [`equivalence_pairing_of`] 同义。
+    pairing: Option<PiecePanicInjection>,
+    /// 阳性对照「判态路 panic 单列」那一遍：空时照 [`piece_panic_control_injection`] 造；不空时**替换**内置的那三条规则。
+    piece_panic_control: Option<PiecePanicInjection>,
+    /// 主取样点或反向取样点那一次导出（[`export_prepared_stream`] 交给判态的 [`ObservationContext`]）。
+    sampling_point_export: Option<PiecePanicInjection>,
 }
 
 /// 两形里头一个没有可判的根槽写的（`judged_root_index` 是 `usize::MAX`）：`main` 或 `pairing`（修订 5 第 5.3 节）；都有交 `None`。
@@ -8038,7 +8105,10 @@ fn judge_the_confirmed_loss_shapes(
     }
     if let Some(pairing_stream) = &shapes.pairing {
         // 装置先过「等价类对拍」再跑主取样点（修订 2 第 2.1 节「定之前与定之后」末句）。
-        let pairing = equivalence_pairing_of(pairing_stream);
+        let pairing = equivalence_pairing_with_injected_piece_panics(
+            pairing_stream,
+            shapes.piece_panic_injections.pairing.as_ref(),
+        );
         let in_flight_writes_kept = call_index_of_each_write(pairing_stream)
             .iter()
             .filter(|call| **call == Some(CONFIRMED_LOSS_IN_FLIGHT_CALL))
@@ -8091,7 +8161,11 @@ fn judge_the_confirmed_loss_shapes(
             lines.finish();
             return 3;
         }
-        let control = piece_panic_control_of(pairing_stream, &pairing);
+        let control = piece_panic_control_of(
+            pairing_stream,
+            &pairing,
+            shapes.piece_panic_injections.piece_panic_control.as_ref(),
+        );
         lines.result(&format!(
             "name=q5_piece_panic_control states={} pool_checker={} ignored_recovery={} record_checker={} other_pieces={} consulted_recovery={} consulted_recovery_of_the_plain_pairing={} o1_columns_differing_from_the_plain_pairing={} mismatching_states={} s13={} holds={}",
             control.states,
@@ -8179,6 +8253,7 @@ fn judge_the_confirmed_loss_shapes(
             StateSelection::WholeLastSegmentAndTheFullyPersistedState,
             started,
             drive_nanoseconds,
+            shapes.piece_panic_injections.sampling_point_export.as_ref(),
         );
         let s9 = s9_checks_of_a_stream(&export);
         export.checks.extend(s9);
@@ -8240,6 +8315,7 @@ fn judge_the_confirmed_loss_shapes(
             StateSelection::ListedStatesKeepingTheConsultedReports(judged),
             started,
             drive_nanoseconds,
+            shapes.piece_panic_injections.sampling_point_export.as_ref(),
         );
         let s9 = s9_checks_of_a_stream(&export);
         export.checks.extend(s9);
@@ -9355,7 +9431,9 @@ impl EquivalencePairing {
     }
 }
 
-/// 在一条 Q5 那一形的流上做「等价类对拍」。
+/// 在一条 Q5 那一形的流上做「等价类对拍」。跑法里 Q5 那一处自修订 6 起改调 [`equivalence_pairing_with_injected_piece_panics`]
+/// （注入为空时同义），这个只剩内联用例在用。
+#[cfg(test)]
 fn equivalence_pairing_of(stream: &PreparedStream) -> EquivalencePairing {
     equivalence_pairing_with_injected_piece_panics(stream, None)
 }
@@ -9594,9 +9672,11 @@ impl PiecePanicControl {
 }
 
 /// 在对拍那一条流上照 [`piece_panic_control_injection`] 注入、再做一遍对拍，与不注入那一遍（`plain`）逐态比。
+/// `replacing_the_built_in_rules` 不空时用它替换内置的那三条规则（登记修订 6 第 6.2 节第 3 条，只有内联用例 U14 交）。
 fn piece_panic_control_of(
     stream: &PreparedStream,
     plain: &EquivalencePairing,
+    replacing_the_built_in_rules: Option<&PiecePanicInjection>,
 ) -> PiecePanicControl {
     let tables = layer0_enumeration_tables(
         &stream.base,
@@ -9607,7 +9687,8 @@ fn piece_panic_control_of(
     let facts = equivalence_pairing_shape_facts(stream, &tables);
     let consulted_recovery_panics_of_the_plain_pairing =
         piece_panic_states_of(&plain.observations)[JudgingPiece::ConsultedRecovery.position()];
-    let Some(injection) = piece_panic_control_injection(&facts.in_flight_writes) else {
+    let built_in_injection = piece_panic_control_injection(&facts.in_flight_writes);
+    let Some(injection) = replacing_the_built_in_rules.or(built_in_injection.as_ref()) else {
         return PiecePanicControl {
             states: 0,
             piece_panic_states: [0; JUDGING_PIECE_COUNT],
@@ -9618,7 +9699,7 @@ fn piece_panic_control_of(
             observations: Vec::new(),
         };
     };
-    let injected = equivalence_pairing_with_injected_piece_panics(stream, Some(&injection));
+    let injected = equivalence_pairing_with_injected_piece_panics(stream, Some(injection));
     PiecePanicControl {
         states: injected.states,
         piece_panic_states: piece_panic_states_of(&injected.observations),
@@ -10986,6 +11067,7 @@ mod tests {
             StateSelection::RegisteredBatches { third_batch: false },
             Instant::now(),
             0,
+            None,
         );
         std::fs::remove_dir_all(&directory)
             .unwrap_or_else(|error| panic!("删 {}：{error}", directory.display()));
@@ -11796,6 +11878,11 @@ mod tests {
             pairing: Some(equivalence_pairing_stream_keeping(
                 EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT,
             )),
+            piece_panic_injections: ConfirmedLossPiecePanicInjections {
+                pairing: None,
+                piece_panic_control: None,
+                sampling_point_export: None,
+            },
         }
     }
 
@@ -11964,4 +12051,590 @@ mod tests {
             "一行判决带 s13=false、q5_not_run=unattributed_panic 与 unattributed_panic_states=1：{collected:?}"
         );
     }
+
+    // ─────────── 登记修订 6 第 6.5 节：U10–U12（根环表）与 U13–U16（Q5 判决路四个出口） ───────────
+
+    /// U11 截停的那一次：流三建到第 30 个 inode（调查报告第 4 节 N = 30 同形，最新根 txg 35），环已转过 24 槽。
+    const INODE_CREATIONS_PAST_ONE_TURN_OF_THE_ROOT_RING: usize = 30;
+    /// U11 取样点敏感的那一条：末次发布的最新根 txg 至少这么大（txg 26 起的那条根才盖掉 mkfs 之后第一条还引用旧单元的根，发布不到
+    /// txg 26 时不装根环表也是绿，`.claude/rules/mutation-sampling.md` 第三类）。
+    const LATEST_ROOT_TXG_PAST_ONE_TURN_OF_THE_ROOT_RING: u64 = 30;
+    /// 第 31 批 core 断言（`crates/singlefs-core/src/allocator.rs` 的 `PoolAllocator::record_root_written_by_this_process`，
+    /// `expect` 的消息）里独有的一段原文；U12 只认这一段。
+    const CORE_ASSERTION_TEXT_OF_AN_ALLOCATOR_WITHOUT_A_ROOT_RING: &str =
+        "分配器没装根环表却记了一条根";
+
+    /// 修订 6 之前的造法（旧冻结树装置 `LiveStream::start` 第 1069–1083 行同形）：`PoolAllocator::new` 加 `mark_format_time_units`，
+    /// 不装根环表。只在 U12 里用。
+    fn allocator_without_the_root_ring_as_before_revision_6() -> PoolAllocator {
+        let mut allocator = PoolAllocator::new(vec![
+            DeviceFreeMap::new(DeviceIdentity(0), LOCAL_DEVICE_BYTES),
+            DeviceFreeMap::new(DeviceIdentity(1), LOCAL_DEVICE_BYTES),
+        ]);
+        allocator.mark_format_time_units(
+            Placement {
+                slot: INSTANCE_TABLE_SLOT,
+                span: 2,
+            },
+            Placement {
+                slot: TREE_TABLE_GENESIS_SLOT,
+                span: 1,
+            },
+        );
+        allocator
+    }
+
+    /// U11 用的截停帮手（只在用例里用）：流三照 [`LiveStream::run_the_script`] 与 [`LiveStream::run_the_third_stream_after_the_first_file`]
+    /// 的次序走取号、暖机、第一个文件、删第一个文件（意图、再一步一步走空），再一次建一个 inode、建到第 `inode_creations` 次为止。
+    /// 任一次调用不成功就停在那里交回（用例判每次调用都成功）。迭代上界：走空那一段 `THIRD_STREAM_UNLINK_STEP_LIMIT`，建 inode `inode_creations`。
+    fn third_stream_through_inode_creations(inode_creations: usize) -> LiveStream {
+        let mut live = LiveStream::start(
+            StreamName::FirstFileUnlinkedThenInodesAcrossOneLeaf,
+            "e171r1-unit-third-stream-past-one-turn-of-the-root-ring",
+            RunSwitches::default(),
+        );
+        if !live.step_acquire_and_warm_up() || !live.step_publish_first_file() {
+            return live;
+        }
+        let write_time_seconds = LOCAL_FIXED_WRITE_TIME_SECONDS + 60;
+        let first_inode = InodeNumber(FIRST_INODE_NUMBER);
+        if !live.step_third_stream_publish(
+            "unlink_intent",
+            IdealModel::answer_unlink_intent,
+            &|writer, allocator, previous, instance| {
+                publish_unlink_intent(
+                    writer,
+                    allocator,
+                    previous,
+                    first_inode,
+                    write_time_seconds,
+                    instance,
+                )
+            },
+        ) {
+            return live;
+        }
+        for _ in 0..THIRD_STREAM_UNLINK_STEP_LIMIT {
+            let Some(inode) = live
+                .output
+                .as_ref()
+                .and_then(|output| output.deleted_inodes.first().copied())
+            else {
+                break;
+            };
+            if !live.step_third_stream_publish(
+                "unlink_step",
+                IdealModel::answer_deletion_step,
+                &|writer, allocator, previous, instance| {
+                    publish_unlink_step(
+                        writer,
+                        allocator,
+                        previous,
+                        inode,
+                        UnlinkBatchSize::OneExtentLowerLeaf,
+                        instance,
+                    )
+                },
+            ) {
+                return live;
+            }
+        }
+        for _ in 0..inode_creations {
+            if !live.step_third_stream_publish(
+                "create_inode",
+                IdealModel::answer_create_inode,
+                &|writer, allocator, previous, instance| {
+                    publish_new_inodes(writer, allocator, previous, 1, write_time_seconds, instance)
+                },
+            ) {
+                return live;
+            }
+            live.inode_creations += 1;
+        }
+        live
+    }
+
+    /// U10（登记修订 6 第 6.5 节；V32：装根环表那一句删掉，三条流都由 `Some` 变 `None`、S1 启动核对不过，这里红；与 core 带不带
+    /// 第 31 批断言无关）：三条流各 `LiveStream::start` 一次、不驱动，分配器都装着根环表，`{:?}` 文本与
+    /// `root_ring_occupancy_after_make_filesystem` 给这次 mkfs 现算的那一份相同；第 6.2 节第 1 条那条 S1 在三条流上都过。
+    #[test]
+    fn every_live_stream_starts_with_the_root_ring_of_make_filesystem() {
+        for name in STREAM_NAMES {
+            let label = name.directory_name();
+            let live = LiveStream::start(
+                name,
+                &format!("e171r1-unit-root-ring-at-the-start-{label}"),
+                RunSwitches::default(),
+            );
+            let installed = live
+                .allocator
+                .root_ring_occupancy()
+                .unwrap_or_else(|| panic!("{label}：分配器没装根环表"));
+            assert_eq!(
+                format!("{installed:?}"),
+                format!(
+                    "{:?}",
+                    root_ring_occupancy_after_make_filesystem(
+                        &common::parameters(),
+                        &live.formatted.genesis
+                    )
+                ),
+                "{label}：装的是 mkfs 刚写下的那一份根环表"
+            );
+            let check_name = format!("{label}_allocator_carries_the_root_ring_of_make_filesystem");
+            let root_ring_checks: Vec<&StopCheck> = live
+                .checks
+                .iter()
+                .filter(|check| check.name == check_name)
+                .collect();
+            assert_eq!(
+                root_ring_checks.len(),
+                1,
+                "{label}：start 里记了恰一条 {check_name}"
+            );
+            assert!(
+                root_ring_checks[0].holds(),
+                "{label}：S1 启动核对过：local={} crates={}",
+                root_ring_checks[0].local,
+                root_ring_checks[0].crates
+            );
+        }
+    }
+
+    /// U11（登记修订 6 第 6.5 节；V32 下两种 core 都红：带第 31 批断言的 core 在第一个文件那次发布 panic，「每次调用都成功」红；
+    /// 不带断言的 core 不回收，txg 26 起 I-3.1 红）：流三驱动到第 30 次建 inode 为止，每次调用都成功；取号、暖机之后的每次发布之后，
+    /// 对整条流施加出的镜像（全部写落地）判 I-3.1 都成立；末次发布的最新根 txg 至少 30（根环已转过 24 槽，取样点敏感）。
+    #[test]
+    fn the_third_stream_past_one_turn_of_the_root_ring_holds_i_3_1() {
+        let live =
+            third_stream_through_inode_creations(INODE_CREATIONS_PAST_ONE_TURN_OF_THE_ROOT_RING);
+        let unsuccessful_calls: Vec<(usize, &'static str, &CallOutcome)> = live
+            .driver
+            .calls
+            .iter()
+            .enumerate()
+            .filter(|(_, call)| call.outcome != CallOutcome::Succeeded)
+            .map(|(call_index, call)| (call_index, call.label, &call.outcome))
+            .collect();
+        assert!(
+            unsuccessful_calls.is_empty(),
+            "每次调用都成功：{unsuccessful_calls:?}"
+        );
+        assert_eq!(
+            live.inode_creations, INODE_CREATIONS_PAST_ONE_TURN_OF_THE_ROOT_RING,
+            "建 inode 建到了截停那一次"
+        );
+        let labels_before_the_publishes: Vec<&'static str> = live
+            .driver
+            .calls
+            .iter()
+            .take(2)
+            .map(|call| call.label)
+            .collect();
+        assert_eq!(
+            labels_before_the_publishes,
+            vec!["acquire", "warm_up"],
+            "前两次调用是取号、暖机，不经分配器"
+        );
+        let operations = live.formatted.retained_operations();
+        let mkfs_operation_count = live.formatted.mkfs_operation_count;
+        let mut image = live.formatted.memory_pool_after_mkfs();
+        // 跨轮带的是镜像上已经施加到 mkfs 之后第几条操作；每一轮只补施加这一次调用发出的那一截。
+        let mut operations_applied_after_mkfs = 0usize;
+        let mut publishes_judged = 0usize;
+        for call in live.driver.calls.iter().skip(2) {
+            image.apply(
+                &operations[mkfs_operation_count + operations_applied_after_mkfs
+                    ..mkfs_operation_count + call.return_point],
+            );
+            operations_applied_after_mkfs = call.return_point;
+            let verdicts = check_pool_image(&image);
+            let allocated_statistic_verdict = verdicts
+                .iter()
+                .find(|(invariant, _)| *invariant == "I-3.1")
+                .map(|(_, verdict)| verdict)
+                .expect("池级 checker 每张镜像都判 I-3.1");
+            assert!(
+                matches!(allocated_statistic_verdict, InvariantVerdict::Holds),
+                "第 {publishes_judged} 次发布（{}）之后 I-3.1 成立：{allocated_statistic_verdict:?}",
+                call.label
+            );
+            publishes_judged += 1;
+        }
+        assert_eq!(
+            publishes_judged,
+            live.driver.calls.len() - 2,
+            "取号、暖机之后的每次发布都判过"
+        );
+        let latest_root_txg = live
+            .output
+            .as_ref()
+            .expect("每次调用都成功，末次发布交回了那一版")
+            .root
+            .checkpoint_txg;
+        assert!(
+            latest_root_txg.0 >= LATEST_ROOT_TXG_PAST_ONE_TURN_OF_THE_ROOT_RING,
+            "末次发布的最新根 txg 至少 {LATEST_ROOT_TXG_PAST_ONE_TURN_OF_THE_ROOT_RING}：{latest_root_txg:?}"
+        );
+    }
+
+    /// U12（登记修订 6 第 6.5 节；只在带第 31 批 core 断言的 `crates/` 上成立，修订 6 树的 core 不带、那棵树不编用例；第 31 批给
+    /// core 断言写的变异「删断言」下第 2 次调用成功，这里红）：`LiveStream::start` 之后把分配器换成修订 6 之前的造法（不装根环表），
+    /// 驱动流一三次调用：取号、暖机成功，第一个文件那次发布在 core 记根时 panic，消息带那条断言独有的一段原文。
+    #[test]
+    fn a_live_stream_without_the_root_ring_panics_in_core_at_its_first_root() {
+        let mut live = LiveStream::start(
+            StreamName::NewPoolFileCreation,
+            "e171r1-unit-without-the-root-ring",
+            RunSwitches::default(),
+        );
+        live.allocator = allocator_without_the_root_ring_as_before_revision_6();
+        assert!(
+            live.allocator.root_ring_occupancy().is_none(),
+            "前提：旧造法不装根环表"
+        );
+        live.run_the_script();
+        let calls = &live.driver.calls;
+        let labels: Vec<&'static str> = calls.iter().map(|call| call.label).collect();
+        assert_eq!(
+            labels,
+            vec!["acquire", "warm_up", "publish"],
+            "流一三次调用"
+        );
+        assert_eq!(calls[0].outcome, CallOutcome::Succeeded, "取号成功");
+        assert_eq!(calls[1].outcome, CallOutcome::Succeeded, "暖机成功");
+        let CallOutcome::Panicked(message) = &calls[2].outcome else {
+            panic!("第一个文件那次发布在 core 里 panic：{:?}", calls[2].outcome);
+        };
+        assert!(
+            message.contains(CORE_ASSERTION_TEXT_OF_AN_ALLOCATOR_WITHOUT_A_ROOT_RING),
+            "panic 的是第 31 批那条 core 断言：{message}"
+        );
+    }
+
+    /// U13–U16 用的注入：一条不点名件的规则（`piece: None`），条件是写表下标 `write_index` 那个写落了（「落」= 落地串那一位不是 `0`）。
+    fn an_unattributed_panic_where_the_write_landed(write_index: usize) -> PiecePanicInjection {
+        PiecePanicInjection {
+            rules: vec![PiecePanicInjectionRule {
+                piece: None,
+                conditions: vec![InjectedLandingCondition::Landed(write_index)],
+            }],
+        }
+    }
+
+    /// U15、U16 用的注入：一条不带条件、不点名件的规则（选中全部判的态）。
+    fn an_unattributed_panic_in_every_state() -> PiecePanicInjection {
+        PiecePanicInjection {
+            rules: vec![PiecePanicInjectionRule {
+                piece: None,
+                conditions: Vec::new(),
+            }],
+        }
+    }
+
+    /// 对拍流（k = 2）的 c1（末段里第 3 次调用发出的第一个写）与末段里 c1 落了的态数：用例自己按末段的混合进制逐态数，不读装置的计数。
+    fn first_in_flight_write_and_the_states_where_it_landed(
+        stream: &PreparedStream,
+    ) -> (usize, usize) {
+        let tables = layer0_enumeration_tables(
+            &stream.base,
+            &stream.writes,
+            &stream.segments,
+            &full_expansion,
+        );
+        let first_in_flight_write = *equivalence_pairing_shape_facts(stream, &tables)
+            .in_flight_writes
+            .first()
+            .expect("对拍流末段里有第 3 次调用发出的写");
+        let digits = LastSegmentDigits::of(stream, &tables);
+        let position = digits
+            .write_indexes
+            .iter()
+            .position(|write_index| *write_index == first_in_flight_write)
+            .expect("c1 在末段里");
+        let states_where_it_landed = (digits.first_ordinal..tables.state_count)
+            .filter(|ordinal| digits.digit_of(*ordinal, position) != 0)
+            .count();
+        (first_in_flight_write, states_where_it_landed)
+    }
+
+    /// U13–U16 用的两形：主那一形取 `truncation` 截出的流二前四次调用，对拍那一形照 `pairing` 交，注入照 `piece_panic_injections` 交。
+    fn confirmed_loss_shapes_with_injections(
+        sampling_point: &'static str,
+        truncation: ConfirmedLossTruncation,
+        pairing: Option<PreparedStream>,
+        piece_panic_injections: ConfirmedLossPiecePanicInjections,
+    ) -> ConfirmedLossShapes {
+        let (main, shape) = confirmed_loss_prepared_stream(
+            second_stream_through_the_first_overwrite(RunSwitches::default()),
+            truncation,
+        );
+        ConfirmedLossShapes {
+            sampling_point,
+            main,
+            shape,
+            drive_nanoseconds: 0,
+            pairing,
+            piece_panic_injections,
+        }
+    }
+
+    /// U13–U16 的共同断言：退 3；倒数第二行（末行是 `name=done`）是判决行，带 `s13=false`、`q5_not_run=unattributed_panic` 与
+    /// `unattributed_panic_states=<用例现算的数>`，判决行只有这一行。
+    fn assert_the_q5_stopped_as_unattributed(
+        exit_code: i32,
+        collected: &[String],
+        expected_unattributed_panic_states: usize,
+    ) {
+        assert_eq!(exit_code, 3, "退 3：{collected:?}");
+        assert!(
+            collected
+                .last()
+                .is_some_and(|line| line.starts_with("name=done ")),
+            "末行是 name=done：{collected:?}"
+        );
+        let verdict_lines: Vec<&String> = collected
+            .iter()
+            .filter(|line| line.starts_with("name=verdict "))
+            .collect();
+        assert_eq!(verdict_lines.len(), 1, "判决行恰一行：{collected:?}");
+        assert_eq!(
+            collected
+                .iter()
+                .rposition(|line| line.starts_with("name=verdict ")),
+            Some(collected.len() - 2),
+            "判决行就在 name=done 之前：{collected:?}"
+        );
+        let verdict = verdict_lines[0];
+        assert!(
+            verdict.contains(" s13=false ")
+                && verdict.contains(" q5_not_run=unattributed_panic ")
+                && verdict.ends_with(&format!(
+                    " unattributed_panic_states={expected_unattributed_panic_states}"
+                )),
+            "判决行带 s13=false、q5_not_run=unattributed_panic 与 unattributed_panic_states={expected_unattributed_panic_states}：{verdict}"
+        );
+    }
+
+    /// U13（登记修订 6 第 6.5 节；V33：对拍之后收 unattributed 那一处整块删掉，往下走到阳性对照、k = 2 上内置规则造不出、判决行换成
+    /// `piece_panic_control_failed`，这里红）：主那一形与对拍都是 k = 2 的流，对拍那一遍注入一条不点名件的规则（条件「c1 落」），
+    /// 在对拍那一处就停：退 3、判决行 `q5_not_run=unattributed_panic`、数等于 c1 落了的态数；阳性对照一行都没打。
+    #[test]
+    fn an_unattributed_panic_in_the_pairing_stops_the_q5_at_the_pairing() {
+        let pairing = equivalence_pairing_stream();
+        let (first_in_flight_write, states_where_it_landed) =
+            first_in_flight_write_and_the_states_where_it_landed(&pairing);
+        let shapes = confirmed_loss_shapes_with_injections(
+            "main_before_the_first_barrier_of_call_3",
+            ConfirmedLossTruncation::AfterTheFirstInFlightWrites(
+                IN_FLIGHT_WRITES_KEPT_IN_THE_TESTS,
+            ),
+            Some(pairing),
+            ConfirmedLossPiecePanicInjections {
+                pairing: Some(an_unattributed_panic_where_the_write_landed(
+                    first_in_flight_write,
+                )),
+                piece_panic_control: None,
+                sampling_point_export: None,
+            },
+        );
+        let command_line = ConfirmedLossCommandLine {
+            output_directory: std::env::temp_dir().join(format!(
+                "e171r1-unit-unattributed-in-the-pairing-{}",
+                std::process::id()
+            )),
+            truncate_at_the_return_point: false,
+        };
+        let mut lines = ResultLines {
+            emitted: 0,
+            collected: Some(Vec::new()),
+        };
+        let exit_code = judge_the_confirmed_loss_shapes(&mut lines, &command_line, shapes, 0);
+        let collected = lines.collected.expect("收集模式");
+        assert_the_q5_stopped_as_unattributed(exit_code, &collected, states_where_it_landed);
+        assert!(
+            !collected
+                .iter()
+                .any(|line| line.starts_with("name=q5_piece_panic_control")),
+            "停在对拍那一处，阳性对照一行都没打：{collected:?}"
+        );
+    }
+
+    /// U14（登记修订 6 第 6.5 节；V34：阳性对照之后收 unattributed 那一处整块删掉，走到 F9 那一支、判决行换成
+    /// `piece_panic_control_failed`，这里红）：同 U13 的两形，对拍不注入；阳性对照那一格交同一条规则（替换内置规则），
+    /// 在阳性对照那一处就停：退 3、数等于 c1 落了的态数；阳性对照那一行打了，主取样点那一形一行都没打。
+    #[test]
+    fn an_unattributed_panic_in_the_piece_panic_control_stops_the_q5_at_the_control() {
+        let pairing = equivalence_pairing_stream();
+        let (first_in_flight_write, states_where_it_landed) =
+            first_in_flight_write_and_the_states_where_it_landed(&pairing);
+        let shapes = confirmed_loss_shapes_with_injections(
+            "main_before_the_first_barrier_of_call_3",
+            ConfirmedLossTruncation::AfterTheFirstInFlightWrites(
+                IN_FLIGHT_WRITES_KEPT_IN_THE_TESTS,
+            ),
+            Some(pairing),
+            ConfirmedLossPiecePanicInjections {
+                pairing: None,
+                piece_panic_control: Some(an_unattributed_panic_where_the_write_landed(
+                    first_in_flight_write,
+                )),
+                sampling_point_export: None,
+            },
+        );
+        let command_line = ConfirmedLossCommandLine {
+            output_directory: std::env::temp_dir().join(format!(
+                "e171r1-unit-unattributed-in-the-control-{}",
+                std::process::id()
+            )),
+            truncate_at_the_return_point: false,
+        };
+        let mut lines = ResultLines {
+            emitted: 0,
+            collected: Some(Vec::new()),
+        };
+        let exit_code = judge_the_confirmed_loss_shapes(&mut lines, &command_line, shapes, 0);
+        let collected = lines.collected.expect("收集模式");
+        assert_the_q5_stopped_as_unattributed(exit_code, &collected, states_where_it_landed);
+        assert_eq!(
+            collected
+                .iter()
+                .filter(|line| line.starts_with("name=q5_piece_panic_control "))
+                .count(),
+            1,
+            "阳性对照那一行打了：{collected:?}"
+        );
+        assert!(
+            !collected
+                .iter()
+                .any(|line| line.starts_with("name=q5_shape")),
+            "停在阳性对照那一处，主取样点那一形一行都没打：{collected:?}"
+        );
+    }
+
+    /// U15（登记修订 6 第 6.5 节；V35：反向取样点导出之后收 unattributed 那一处整块删掉，往下打出 `report_the_confirmed_loss` 那几行
+    /// 与不带 `q5_not_run` 的判决行，这里红）：反向取样点那一形（对拍为空），导出那一格交一条不带条件的不点名件规则，导出之后就停：
+    /// 退 3、数等于末段各写取法数之积罩的态加全持久态（用例现算）；`report_the_confirmed_loss` 的计数行一行都没打。
+    #[test]
+    fn an_unattributed_panic_at_the_reverse_sampling_point_stops_before_the_tally() {
+        let shapes = confirmed_loss_shapes_with_injections(
+            "reverse_at_the_return_point_of_call_2",
+            ConfirmedLossTruncation::AtTheConfirmedReturnPoint,
+            None,
+            ConfirmedLossPiecePanicInjections {
+                pairing: None,
+                piece_panic_control: None,
+                sampling_point_export: Some(an_unattributed_panic_in_every_state()),
+            },
+        );
+        let judged_states = {
+            let tables = layer0_enumeration_tables(
+                &shapes.main.base,
+                &shapes.main.writes,
+                &shapes.main.segments,
+                &full_expansion,
+            );
+            let last_segment = shapes.main.segments.last().cloned().unwrap_or_default();
+            let product_of_the_choices: u64 = last_segment
+                .iter()
+                .map(|write_index| {
+                    if tables.is_tearable_by_write[*write_index] {
+                        3u64
+                    } else {
+                        2u64
+                    }
+                })
+                .product();
+            let first_ordinal_of_the_last_segment = tables
+                .state_ranges_by_segment
+                .last()
+                .map_or(0, |state_range| state_range.start);
+            let mut states: BTreeSet<u64> = (first_ordinal_of_the_last_segment
+                ..first_ordinal_of_the_last_segment + product_of_the_choices)
+                .collect();
+            states.insert(tables.state_count - 1);
+            states.len()
+        };
+        let output_directory = std::env::temp_dir().join(format!(
+            "e171r1-unit-unattributed-at-the-reverse-sampling-point-{}",
+            std::process::id()
+        ));
+        let command_line = ConfirmedLossCommandLine {
+            output_directory: output_directory.clone(),
+            truncate_at_the_return_point: true,
+        };
+        let mut lines = ResultLines {
+            emitted: 0,
+            collected: Some(Vec::new()),
+        };
+        let exit_code = judge_the_confirmed_loss_shapes(&mut lines, &command_line, shapes, 0);
+        std::fs::remove_dir_all(&output_directory)
+            .unwrap_or_else(|error| panic!("删 {}：{error}", output_directory.display()));
+        let collected = lines.collected.expect("收集模式");
+        assert_the_q5_stopped_as_unattributed(exit_code, &collected, judged_states);
+        assert!(
+            !collected
+                .iter()
+                .any(|line| line.starts_with("name=q5 sampling_point=")),
+            "停在计数之前，report_the_confirmed_loss 的计数行一行都没打：{collected:?}"
+        );
+    }
+
+    /// U16（登记修订 6 第 6.5 节；V36：主取样点导出之后收 unattributed 那一处整块删掉，往下走到类核对与判决，这里红）：主那一形取
+    /// k = 2 的流、对拍为空（只为走到主取样点导出之后那一处，跑法里不会这么交），导出那一格交同 U15 的规则，导出之后就停：
+    /// 退 3、数等于分法交的判态数（用例从 `classify_the_last_segment` 的 `judged_ordinals` 现数）；类核对那一行没打。
+    #[test]
+    fn an_unattributed_panic_at_the_main_sampling_point_stops_before_the_classes_are_judged() {
+        let shapes = confirmed_loss_shapes_with_injections(
+            "main_before_the_first_barrier_of_call_3",
+            ConfirmedLossTruncation::AfterTheFirstInFlightWrites(
+                IN_FLIGHT_WRITES_KEPT_IN_THE_TESTS,
+            ),
+            None,
+            ConfirmedLossPiecePanicInjections {
+                pairing: None,
+                piece_panic_control: None,
+                sampling_point_export: Some(an_unattributed_panic_in_every_state()),
+            },
+        );
+        let judged_states = {
+            let observation_tables = ObservationTables::of(&shapes.main);
+            let context = observation_tables.context(&shapes.main);
+            let digits = LastSegmentDigits::of(&shapes.main, &observation_tables.tables);
+            let classification = classify_the_last_segment(
+                &context,
+                &digits,
+                CONFIRMED_LOSS_IN_FLIGHT_CALL,
+                CONFIRMED_LOSS_STATE_LIMIT,
+            );
+            assert!(
+                !classification.stopped_at_the_state_limit,
+                "k = 2 的流分类没撞态数上限"
+            );
+            classification.judged_ordinals(&digits).len()
+        };
+        let output_directory = std::env::temp_dir().join(format!(
+            "e171r1-unit-unattributed-at-the-main-sampling-point-{}",
+            std::process::id()
+        ));
+        let command_line = ConfirmedLossCommandLine {
+            output_directory: output_directory.clone(),
+            truncate_at_the_return_point: false,
+        };
+        let mut lines = ResultLines {
+            emitted: 0,
+            collected: Some(Vec::new()),
+        };
+        let exit_code = judge_the_confirmed_loss_shapes(&mut lines, &command_line, shapes, 0);
+        std::fs::remove_dir_all(&output_directory)
+            .unwrap_or_else(|error| panic!("删 {}：{error}", output_directory.display()));
+        let collected = lines.collected.expect("收集模式");
+        assert_the_q5_stopped_as_unattributed(exit_code, &collected, judged_states);
+        assert!(
+            !collected
+                .iter()
+                .any(|line| line.starts_with("name=q5_equivalence sampling_point=")),
+            "停在类核对之前，name=q5_equivalence sampling_point= 一行都没打：{collected:?}"
+        );
+    }
 }
````

### mutations-append.tsv

````text
E171R1 装置 V32 装根环表那一句删掉、退回不装根环表那一形（登记修订 6 第 6.2 节第 5 条；U10 那一侧）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(\n            &common::parameters(),\n            &formatted.genesis,\n        ));\n	        // V32：不装根环表（退回修订 6 之前那一形）\n	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	every_live_stream_starts_with_the_root_ring_of_make_filesystem
E171R1 装置 V32 装根环表那一句删掉、退回不装根环表那一形（登记修订 6 第 6.2 节第 5 条；U11 那一侧）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(\n            &common::parameters(),\n            &formatted.genesis,\n        ));\n	        // V32：不装根环表（退回修订 6 之前那一形）\n	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_third_stream_past_one_turn_of_the_root_ring_holds_i_3_1
E171R1 装置 V33 Q5 对拍之后收 unattributed 那一处整块删掉（登记修订 6 第 6.2 节第 5 条）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        if let Some(exit_code) = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(\n            lines,\n            failed_checks,\n            &pairing.observations,\n        ) {\n            return exit_code;\n        }\n	        // V33：对拍之后不收 unattributed\n	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	an_unattributed_panic_in_the_pairing_stops_the_q5_at_the_pairing
E171R1 装置 V34 Q5 阳性对照之后收 unattributed 那一处整块删掉（登记修订 6 第 6.2 节第 5 条）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        if let Some(exit_code) = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(\n            lines,\n            failed_checks,\n            &control.observations,\n        ) {\n            return exit_code;\n        }\n	        // V34：阳性对照之后不收 unattributed\n	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	an_unattributed_panic_in_the_piece_panic_control_stops_the_q5_at_the_control
E171R1 装置 V35 Q5 反向取样点导出之后收 unattributed 那一处整块删掉（登记修订 6 第 6.2 节第 5 条）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        lines.result(&q5_piece_panics_line("reverse", &export.observations));\n        if let Some(exit_code) = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(\n            lines,\n            failed_checks + totals.failed_checks,\n            &export.observations,\n        ) {\n            return exit_code;\n        }\n	        lines.result(&q5_piece_panics_line("reverse", &export.observations));\n        // V35：反向取样点导出之后不收 unattributed\n	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	an_unattributed_panic_at_the_reverse_sampling_point_stops_before_the_tally
E171R1 装置 V36 Q5 主取样点导出之后收 unattributed 那一处整块删掉（登记修订 6 第 6.2 节第 5 条）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        lines.result(&q5_piece_panics_line("main", &export.observations));\n        if let Some(exit_code) = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(\n            lines,\n            failed_checks + totals.failed_checks,\n            &export.observations,\n        ) {\n            return exit_code;\n        }\n	        lines.result(&q5_piece_panics_line("main", &export.observations));\n        // V36：主取样点导出之后不收 unattributed\n	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	an_unattributed_panic_at_the_main_sampling_point_stops_before_the_classes_are_judged
E171R1 装置 U12 那一侧：第 31 批 core 记根断言删掉、没装根环表的分配器静默返回（登记修订 6 第 6.7 节第 4 条）	crates/singlefs-core/src/allocator.rs	        let occupancy = self.root_ring.as_mut().expect(	        let Some(occupancy) = self.root_ring.as_mut() else {\n            return;\n        };\n        let _message_of_the_removed_assertion = (	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_live_stream_without_the_root_ring_panics_in_core_at_its_first_root
````

