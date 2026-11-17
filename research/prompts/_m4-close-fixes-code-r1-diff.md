# m4-close-fixes-code-r1 改动全文：代码轮 r3 判决之后第一批修补里的产品代码块

来源：`~/.local/share/singlefs/work/m4-close-code-r3-fix-writer/patch-after-crate-merge/crates.patch`，按打进主工作区的原样抄录，测试与变异表的块不在里面。第二批补丁（`~/.local/share/singlefs/work/m4-close-code-r3-fix-writer-2/patch/crates.patch`）只改验证包，不在这里。

```diff
diff --git a/crates/singlefs-core/src/compaction_plan.rs b/crates/singlefs-core/src/compaction_plan.rs
index 82f0f32..0a52ec6 100644
--- a/crates/singlefs-core/src/compaction_plan.rs
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -856,6 +856,10 @@ pub struct CompactionOfThisMount {
     /// 可写挂载读盘建上一版时按位置条目读一个单元只坏一份的次数（D4（校验和位置） 已定项 4「读到只坏一份时计数」）：挂载交回之前记一次
     /// （`mount::mount_writable` 那一串）。与上一样分开记：上一样是读—改—写读回旧数据单元，这一样是挂载读的各类单元。只在内存里，新挂载从 0 起。
     unit_reads_with_one_copy_failing_while_mounting: u64,
+    /// 可写挂载读盘建上一版时读到只坏一份、到挂载交回那一版仍在中央映射里（或留存条目里）的那几个单元的映射 key（一个单元一把，按读到的先后）：
+    /// 会话下一次用户改动发成之后照快照下另发那批同一条路径把它们搬到新槽、两份新写（用户 2026-11-05 定「可写挂载时搬」；
+    /// `mounted_session::MountedSession::publish_user_change`），交给那一批就清空。只在内存里，新挂载是空的。
+    mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting: Vec<Vec<u8>>,
     /// 只供测试的开关：会话另发的那一批搬迁发之前当作被空间准入拒（[`RelocationKeptForTheSnapshotsRefusedForSpace`]）。新挂载是产品值。
     relocation_kept_for_the_snapshots_refused_for_space:
         RelocationKeptForTheSnapshotsRefusedForSpace,
@@ -882,6 +886,7 @@ impl CompactionOfThisMount {
             write_bandwidth_window: None,
             old_data_unit_reads_back_with_one_copy_failing: 0,
             unit_reads_with_one_copy_failing_while_mounting: 0,
+            mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting: Vec::new(),
             relocation_kept_for_the_snapshots_refused_for_space:
                 RelocationKeptForTheSnapshotsRefusedForSpace::NotForced,
             relocation_kept_for_the_snapshots_refused_not_for_space:
@@ -911,6 +916,40 @@ impl CompactionOfThisMount {
             .expect("一次挂载里只坏一份的读次数装得进 u64");
     }
 
+    /// 可写挂载读盘建上一版时读到只坏一份、还没交给另发那一批的那几个单元的映射 key（[`Self::keep_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting`]
+    /// 记的，会话下一次用户改动发成之后取走）。
+    #[must_use]
+    pub fn mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting(
+        &self,
+    ) -> &[Vec<u8>] {
+        &self.mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting
+    }
+
+    /// 记下可写挂载读盘建上一版时读到只坏一份的那几个单元的映射 key（挂载交回之前调一次，接在已记的后面、同一把只记一次）。
+    pub(crate) fn keep_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting(
+        &mut self,
+        mapping_keys: Vec<Vec<u8>>,
+    ) {
+        for mapping_key in mapping_keys {
+            if !self
+                .mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting
+                .contains(&mapping_key)
+            {
+                self.mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting
+                    .push(mapping_key);
+            }
+        }
+    }
+
+    /// 取走记着的那几把 key（交给另发那一批），这一份挂载期状态里清空：另发那一批不管搬没搬成都只试这一次，被拒的结局随那次改动报出。
+    pub(crate) fn take_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting(
+        &mut self,
+    ) -> Vec<Vec<u8>> {
+        std::mem::take(
+            &mut self.mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting,
+        )
+    }
+
     /// 装上只供测试的开关：会话另发的那一批搬迁发之前当作被空间准入拒（[`RelocationKeptForTheSnapshotsRefusedForSpace`]）。产品路径一处都不调它。
     pub fn force_the_relocation_kept_for_the_snapshots_refused_for_space_for_tests(
         &mut self,
```

```diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 590b9f2..f7424ba 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -89,9 +89,10 @@ use crate::system_configuration::SystemImmutableSizes;
 use crate::transaction::{
     central_mapping_locations_in_the_version,
     copies_of_a_released_unit_failing_the_release_checksum_check,
-    next_instance_generation_to_acquire, placement_to_release_after_checking_every_device,
-    prepare_the_publish_without_units, prepare_the_row_publish_on_a_version_without_file,
-    prepare_the_version_publish, publish_instance_table_on_version_without_file, publish_version,
+    mapping_keys_of_the_units_at_these_locations, next_instance_generation_to_acquire,
+    placement_to_release_after_checking_every_device, prepare_the_publish_without_units,
+    prepare_the_row_publish_on_a_version_without_file, prepare_the_version_publish,
+    publish_instance_table_on_version_without_file, publish_version,
     publish_version_appending_a_lineage_record, publish_version_with_unmount_marker,
     publish_without_units, read_the_witness_before_the_expected_acquisition_writes,
     refuse_locations_that_do_not_name_two_pool_devices,
@@ -1192,25 +1193,71 @@ fn map_rebuild_failure(failure: RebuildVersionFailure) -> MountError {
     clippy::enum_variant_names,
     reason = "三个成员与 PoolVersion、RebuiltVersion 同名（D8 已定项 16 第 5 条：第三个成员三处同形），都以 File 结尾是条款的叫法"
 )]
+/// 每个成员另带建它时按位置条目读单元读到只坏一份的那几个单元的位置条目（`units_read_with_one_copy_failing`，一个单元一组；
+/// [`PoolReaderCountingUnitReadsWithOneCopyFailing::locations_of_the_units_read_with_one_copy_failing`]）：可写挂载交回之前按挂载之后那一版
+/// 认成映射 key，交给会话下一次用户改动之后另发的那一批（用户 2026-11-05 定「可写挂载时搬」）。
 enum PreviousVersion {
     /// 树表 0 条：这一版只有根自己指着的实例表与树表两个单元。写行要在这一版那张实例表后面接行，所以连表一起解出来。
     WithoutFile {
         root: RootRecord,
         table: InstanceTableChain,
+        units_read_with_one_copy_failing: Vec<[LocationEntry; 2]>,
     },
     WithFile {
         output: TransactionOutput,
         table: InstanceTableChain,
+        units_read_with_one_copy_failing: Vec<[LocationEntry; 2]>,
     },
     /// 有树、文件集合为空（D8（核心索引结构） 已定项 16 第 5 条，`RebuiltVersion::WithTreesAndNoFile`）：写行与暖机照带文件的一版那样做，
     /// 发出来的一版按 `PoolVersion::of_a_version_of_trees` 归成员。
     WithTreesAndNoFile {
         output: TransactionOutput,
         table: InstanceTableChain,
+        units_read_with_one_copy_failing: Vec<[LocationEntry; 2]>,
     },
 }
 
 impl PreviousVersion {
+    /// 建这一版时读到只坏一份的那几个单元的位置条目（[`Self::record_the_units_read_with_one_copy_failing_while_building_on_it`] 记的）。
+    fn units_read_with_one_copy_failing_while_building_on_it(&self) -> &[[LocationEntry; 2]] {
+        match self {
+            PreviousVersion::WithoutFile {
+                units_read_with_one_copy_failing,
+                ..
+            }
+            | PreviousVersion::WithFile {
+                units_read_with_one_copy_failing,
+                ..
+            }
+            | PreviousVersion::WithTreesAndNoFile {
+                units_read_with_one_copy_failing,
+                ..
+            } => units_read_with_one_copy_failing,
+        }
+    }
+
+    /// 记下建这一版时读到只坏一份的那几个单元的位置条目（建完、落定建的是哪一版之后记一次；退到的那一版建不出、又建所选那一版时，
+    /// 两趟读到的都在里面，认 key 时按挂载之后那一版滤）。
+    fn record_the_units_read_with_one_copy_failing_while_building_on_it(
+        &mut self,
+        locations: Vec<[LocationEntry; 2]>,
+    ) {
+        match self {
+            PreviousVersion::WithoutFile {
+                units_read_with_one_copy_failing,
+                ..
+            }
+            | PreviousVersion::WithFile {
+                units_read_with_one_copy_failing,
+                ..
+            }
+            | PreviousVersion::WithTreesAndNoFile {
+                units_read_with_one_copy_failing,
+                ..
+            } => *units_read_with_one_copy_failing = locations,
+        }
+    }
+
     /// 这一版的实例表链：全部行（写行时要在它后面接上这次的行，取号之前的准入也按它的行数算）与每一片的指针
     /// （写行那次发布整条链重写，这几片逐片释放）。两臂都有表：树表 0 条那一版的表是根记录直接指着的那条链
     /// （mkfs 种下的，或上一次挂载写行重写的）。
@@ -1341,11 +1388,21 @@ fn rebuild_previous_version(
     let table = instance_table_chain_of_root(reader, root)
         .map_err(|failure| instance_table_refusal(&failure))?;
     match rebuilt {
-        RebuiltVersion::WithoutFile => Ok(PreviousVersion::WithoutFile { root: *root, table }),
-        RebuiltVersion::WithFile(output) => Ok(PreviousVersion::WithFile { output, table }),
-        RebuiltVersion::WithTreesAndNoFile(output) => {
-            Ok(PreviousVersion::WithTreesAndNoFile { output, table })
-        }
+        RebuiltVersion::WithoutFile => Ok(PreviousVersion::WithoutFile {
+            root: *root,
+            table,
+            units_read_with_one_copy_failing: Vec::new(),
+        }),
+        RebuiltVersion::WithFile(output) => Ok(PreviousVersion::WithFile {
+            output,
+            table,
+            units_read_with_one_copy_failing: Vec::new(),
+        }),
+        RebuiltVersion::WithTreesAndNoFile(output) => Ok(PreviousVersion::WithTreesAndNoFile {
+            output,
+            table,
+            units_read_with_one_copy_failing: Vec::new(),
+        }),
     }
 }
 
@@ -7655,12 +7712,27 @@ fn establish_instance<Device: BlockDevice>(
         &mut background_compute_of_this_mount,
     );
     // 建上一版读到只坏一份的次数记进这次挂载的状态（D4（校验和位置） 已定项 4「读到只坏一份时计数」），会话从它接着用。
-    // 只计数、不搬：挂载这一串里另发一批搬迁条款没写（要不要、放在哪一步、取号之前的预演罩不罩它、被拒怎么办），交主 agent。
+    // 读到只坏一份的那几个单元要搬（用户 2026-11-05 定「可写挂载时搬，冷恢复只计数」）：挂载这一串里不另发——另发一批就是挂载的第七段写，
+    // 挂载分段、二次崩溃的摆法、取号之前的续做预演都要跟着改（`crates/singlefs-verification/src/writable_mount_phases.rs` 那六段）；
+    // 这里按挂载之后那一版把它们认成映射 key 记进挂载期状态，会话下一次用户改动发成之后照快照下另发那批同一条路径搬
+    // （`mounted_session::MountedSession::publish_user_change`）。挂载这一串里已经重写掉的（树表、记账树这类每次发布都换下的单元）认不出、不记。
     compaction_continuation
         .state_of_this_mount
         .count_unit_reads_with_one_copy_failing_while_mounting(
             start.unit_reads_with_one_copy_failing_while_building_on_the_version,
         );
+    if let Some(version_after_the_mount) = current.version_of_trees() {
+        compaction_continuation
+            .state_of_this_mount
+            .keep_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting(
+                mapping_keys_of_the_units_at_these_locations(
+                    version_after_the_mount,
+                    start
+                        .previous
+                        .units_read_with_one_copy_failing_while_building_on_it(),
+                ),
+            );
+    }
     // 实例表清扫（D18（块里携带什么信息） 已定项 11「回收」一条「谁跑、什么时候跑」，用户 2026-10-03 定）：写行、暖机、抬 F、删除续做之后，
     // 交回之前判一次。清扫自己不让挂载失败（落点被拒这一轮不删、挂载照样交回可写，Q4），走到哪儿记进交回值。
     let instance_table_sweep = with_reads_straight_to_the_disk(&read_route, || {
@@ -9501,6 +9573,12 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
         };
     let unit_reads_with_one_copy_failing_while_building_on_the_version =
         devices_counting_unit_reads_with_one_copy_failing.unit_reads_with_one_copy_failing();
+    // 读到只坏一份的那几个单元的位置条目随建出来的那一版交下去（`PreviousVersion` 带着），挂载交回之前按挂载之后那一版认成映射 key。
+    let mut previous = previous;
+    previous.record_the_units_read_with_one_copy_failing_while_building_on_it(
+        devices_counting_unit_reads_with_one_copy_failing
+            .locations_of_the_units_read_with_one_copy_failing(),
+    );
     // 环里没有记录时从 1 起（D23（journal 的角色与格式） 已定项 14 第 3 条：计数器全池接着走）。
     let next_counter = records
         .values()
```

```diff
diff --git a/crates/singlefs-core/src/mounted_read.rs b/crates/singlefs-core/src/mounted_read.rs
index fffcf7f..dcb355a 100644
--- a/crates/singlefs-core/src/mounted_read.rs
+++ b/crates/singlefs-core/src/mounted_read.rs
@@ -46,12 +46,14 @@ use crate::records::{
 use crate::recovery::{
     choose_root, choose_system_configuration_naming_devices_without_a_mountable_slot,
     fall_back_judged_on_the_three_readers,
+    inode_record_is_not_below_the_separator_key_and_the_container,
     inode_records_of_the_leaf_ascend_after_the_leaves_before,
     inode_tree_root_key_range_is_the_subtree_coverage,
     read_mapped_tree_node_via_hint_then_central_mapping, read_mapped_tree_root,
     read_unit_via_locations, replay_journal, scan_journal,
     tree_table_entries_each_kind_at_most_once, FailureKindForTheFallBack, JournalScanReport,
-    MappedTreeNodeClass, NamedUnitVerification, PoolReader, RecoveryFailure,
+    MappedTreeNodeClass, NamedUnitVerification, PoolReader,
+    PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition, RecoveryFailure,
     TreeTableEntriesRefusal,
 };
 use crate::root_record::RootRecord;
@@ -81,7 +83,8 @@ pub struct ReadPathObservation {
     /// 向块层发了几次读（每条位置条目试一次就算一次，读不回字节也算——请求已经发出去了）。
     pub device_reads_issued: u64,
     /// 读数据单元时只坏一份的次数：同一组位置条目里前一条读不回字节或整单元校验和对不上、后一条对上（D4（校验和位置） 已定项 4
-    /// 「读到只坏一份时计数」；两条位置提示都没对上、转去查映射的那一回不算，映射那一组里只坏一份的照算）。
+    /// 「读到只坏一份时计数」；两条位置提示都没对上、转去查映射的那一回不算，映射那一组里只坏一份的照算；按提示读到只坏一份而提示不是这个单元
+    /// 此刻在映射里的落点（提示过期）不算）。一次读中途报错时，报错之前读到的照样进打开之后的累计（[`OpenFileForRead::read_at`]）。
     /// 只读挂载只计数、不搬：它没有写者，搬迁要走事务层的发布（C946（只坏一份的读没有计数也没有搬迁修复））。
     pub data_unit_reads_with_one_copy_failing: u64,
 }
@@ -506,7 +509,7 @@ pub fn open_pool_for_read(
     let mut inode_records: Vec<InodeRecord> = Vec::new();
     let mut largest_inode_number_of_the_leaves_read = None;
     for entry_bytes in &inode_root.entries {
-        let (_separator_key, identity_in_the_entry, child) = parse_inode_internal_entry_or_refusal(
+        let (separator_key, identity_in_the_entry, child) = parse_inode_internal_entry_or_refusal(
             entry_bytes,
         )
         .map_err(|refusal| match refusal {
@@ -567,12 +570,21 @@ pub fn open_pool_for_read(
         for record_bytes in &container.records {
             let record = InodeRecord::parse(record_bytes)
                 .ok_or(OpenPoolForReadFailure::InodeRecordMalformed)?;
+            // 记录号不小于分隔 key 与容器号（I-9.4 第一句，分隔 key 那一半是 I-9.12 第二句）：与冷走读、可写挂载重建、快照冷读同一个判法，
+            // 只读挂载此前丢了分隔 key、收下了另三个读者都拒的一版（判决 `research/prompts/m4-close-code-r3-main-verification.md` 第三节第 2 条）。
+            inode_record_is_not_below_the_separator_key_and_the_container(
+                record.inode,
+                separator_key,
+                identity_in_the_entry.container,
+            )?;
             records_of_the_container.push(record);
         }
-        // 只读挂载与冷走读、可写挂载重建同一判：全部叶的记录按叶序严格升序（写者的前提，I-9.12 与 I-9.4 第四句），
-        // 同一张两片交叠的镜像读 inode 树的各个读者一个结局，不在这里收下（判决 `research/prompts/m4-close-code-r2-main-verification.md` 第三节第一簇）。
+        // 只读挂载与冷走读、可写挂载重建同一判：全部叶的记录按叶序严格升序（写者的前提，I-9.12 与 I-9.4 第四句），分隔 key 大于前面那几片的
+        // 最大记录号（I-9.12 第三句），同一张坏镜像读 inode 树的各个读者一个结局，不在这里收下（判决 `research/prompts/m4-close-code-r2-main-verification.md`
+        // 第三节第一簇、`research/prompts/m4-close-code-r3-main-verification.md` 第三节第 1 条）。
         largest_inode_number_of_the_leaves_read =
             inode_records_of_the_leaf_ascend_after_the_leaves_before(
+                separator_key,
                 largest_inode_number_of_the_leaves_read,
                 &records_of_the_container,
             )?;
@@ -977,9 +989,42 @@ impl OpenFileForRead<'_> {
                 observation: ReadPathObservation::default(),
             });
         }
+        let mut observation = ReadPathObservation::default();
+        let read = self.read_the_data_units_of_the_range(
+            reader,
+            offset,
+            length_in_bytes,
+            end_exclusive,
+            &mut observation,
+        );
+        // 读到一半报错（后面一个单元两份都坏、映射里没有、自描述对不上……）的那一次，报错之前已经发出的读、已经读到的只坏一份照样并进打开之后的累计：
+        // 与写入口「改动被拒时那几次读也算」同一种口径——读是真发到盘上的、坏的那一份是真读到的，计不计不看这一次调用成没成
+        // （判决 `research/prompts/m4-close-code-r3-main-verification.md` 第三节第 5 条）。这一次读自己的观测随报错丢掉，只留在累计里。
+        self.mount.add_to_observation_since_open(observation);
+        Ok(FileReadOutput {
+            bytes: read?,
+            observation,
+        })
+    }
+
+    /// [`Self::read_at`] 已核过范围、长度不为 0 之后的那一段：逐个单元读、核、拷出 `[offset, end_exclusive)`，这一次读的观测记进 `observation`
+    /// （报错时记到报错之前那一步为止）。
+    ///
+    /// # Errors
+    /// 同 [`Self::read_at`]，范围那一种除外。
+    ///
+    /// # Panics
+    /// 同 [`Self::read_at`]。
+    fn read_the_data_units_of_the_range(
+        &self,
+        reader: &dyn PoolReader,
+        offset: FileOffsetInBytes,
+        length_in_bytes: u64,
+        end_exclusive: u64,
+        observation: &mut ReadPathObservation,
+    ) -> Result<Vec<u8>, FileReadFailure> {
         let payload_capacity = payload_capacity_in_bytes();
         let span = data_unit_span_covering(offset, FileOffsetInBytes(end_exclusive - 1));
-        let mut observation = ReadPathObservation::default();
         let mut bytes =
             Vec::with_capacity(usize::try_from(length_in_bytes).expect("一次读的长度装得进 usize"));
         let mut anchor_offset_of_the_previous_unit: Option<u64> = None;
@@ -1003,7 +1048,7 @@ impl OpenFileForRead<'_> {
                 continue;
             };
             let unit_bytes =
-                self.dereference_data_unit(reader, record, unit_index_in_file, &mut observation)?;
+                self.dereference_data_unit(reader, record, unit_index_in_file, observation)?;
             let header = parse_data_unit(&unit_bytes).map_err(|error| {
                 FileReadFailure::DataUnitMalformed {
                     unit_index_in_file,
@@ -1040,8 +1085,7 @@ impl OpenFileForRead<'_> {
             length_in_bytes,
             "拷出来的字节数要等于请求的长度：各单元的声明长度在上面逐个核过"
         );
-        self.mount.add_to_observation_since_open(observation);
-        Ok(FileReadOutput { bytes, observation })
+        Ok(bytes)
     }
 
     /// 解引用一个数据单元：先按父指针里的位置提示逐条试（整单元 CRC-32C 等于条目里的校验和才算读到），
@@ -1057,8 +1101,32 @@ impl OpenFileForRead<'_> {
         let pointer = record.data_unit_pointer;
         let hinted_slot = pointer.locations[0].slot;
         observation.data_unit_dereferences += 1;
-        let by_hint =
-            read_unit_by_location_entries(reader, &pointer.locations, data_unit_bytes, observation);
+        // 按提示读的那一次另记一份观测：只坏一份按这个单元此刻的权威落点判（中央映射里那两条位置条目，D19（块指针的结构与宽度预算） 已定项 8），
+        // 单元被整理搬走之后提示还指旧槽，旧槽上坏的那一份不归这个单元计、不报（判决 `research/prompts/m4-close-code-r3-main-verification.md`
+        // 第三节第 4 条；报给读者那一半由 `recovery::PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition` 判，同冷走读）。映射那一组读到的只坏一份照计。
+        let mapping_key = mapping_key_for_data(pointer.head, pointer.write_order);
+        let central_mapping_locations_of_key = |mapping_key_looked_up: &[u8]| {
+            Ok(self.mount.central_mapping_lookup(mapping_key_looked_up))
+        };
+        let reader_noting_one_copy_failing_only_at_the_mapping_position =
+            PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition::over(
+                reader,
+                &mapping_key,
+                &central_mapping_locations_of_key,
+            );
+        let mut observation_of_the_read_by_the_hints = ReadPathObservation::default();
+        let by_hint = read_unit_by_location_entries(
+            &reader_noting_one_copy_failing_only_at_the_mapping_position,
+            &pointer.locations,
+            data_unit_bytes,
+            &mut observation_of_the_read_by_the_hints,
+        );
+        if reader_noting_one_copy_failing_only_at_the_mapping_position
+            .dropped_a_one_copy_failing_note_off_the_mapping_position()
+        {
+            observation_of_the_read_by_the_hints.data_unit_reads_with_one_copy_failing = 0;
+        }
+        *observation = observation.plus(observation_of_the_read_by_the_hints);
         let hint_outcome = match by_hint {
             UnitReadOutcome::Read(unit_bytes) => return Ok(unit_bytes),
             UnitReadOutcome::EveryLocationReadButTheChecksumMismatched => {
@@ -1068,7 +1136,6 @@ impl OpenFileForRead<'_> {
         };
         // 提示不算数 ⇒ 走中央映射那条路，多跳一次。
         observation.stale_location_hint_hops += 1;
-        let mapping_key = mapping_key_for_data(pointer.head, pointer.write_order);
         let mapped_locations = self.mount.central_mapping_lookup(&mapping_key).ok_or(
             FileReadFailure::CentralMappingMiss {
                 unit_index_in_file,
```

```diff
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index fc7df5b..eb08b99 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -91,6 +91,7 @@ use crate::transaction::{
     publish_overwrite_of_an_inode, publish_partial_write, publish_sequential_write,
     publish_sequential_write_of_an_inode, publish_size_change, publish_snapshot_destruction_intent,
     publish_unlink_intent, publish_version, relocations_of_old_data_units_kept_for_the_snapshots,
+    relocations_of_the_units_read_with_one_copy_failing_while_mounting,
     walk_the_deletion_queue_until_it_is_empty, CentralMappingOfAVersionReadLeafByLeaf,
     CompactionIntentStep, CompactionStepRefused, DataChangesPublishedTogether,
     DeletionAndLineageOfThePublish, DeletionQueueWalk, FirstFile, MappingLookup,
@@ -256,7 +257,9 @@ pub struct FloorRaisesPushedForTheRelocation {
 
 /// 一次用户改动里各次发布的写入口上读—改—写读回旧数据单元时只坏一份的那几次（`transaction::PoolWriter` 的两份账），按发布的先后并起来：
 /// 次数并进这次挂载的累计（[`MountedSession::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`]），映射 key 交给另发的那一批搬迁。
-/// 被拒的那几次发布读回的也算：读是真发到盘上的。
+/// 被拒的那几次发布读回的也算：读是真发到盘上的。同一个旧单元一次改动里只算一次：被空间拒、推一串抬 F 之后重发同一个改动，
+/// 每一次发布都重读同几个旧单元（同一把映射 key），读到的是同一份坏副本，算一次（判决 `research/prompts/m4-close-code-r3-main-verification.md`
+/// 第三节第 6 条「每次用户改动只计一次」；与可写挂载建上一版那一读「同一份坏副本一次挂载只计一次」同一种口径）。
 struct OldDataUnitReadsBackWithOneCopyFailingInThisChange {
     reads: u64,
     mapping_keys: Vec<Vec<u8>>,
@@ -270,20 +273,47 @@ impl OldDataUnitReadsBackWithOneCopyFailingInThisChange {
         }
     }
 
-    /// 并进一次发布的写入口上那两份账。
+    /// 并进一次发布的写入口上那两份账：这次改动里前面的发布已经读回过的旧单元（映射 key 已在账上）不再加次数、不再记 key。
     ///
     /// # Panics
-    /// 次数加起来越过 u64：每一次都是一次真读盘，一次改动里读不到 2⁶⁴ 次。
+    /// 次数加起来越过 u64：每一次都是一次真读盘，一次改动里读不到 2⁶⁴ 次。写入口上的次数与 key 不是一次一把：两份账在
+    /// `transaction::read_back_the_old_payload_counting_a_failing_copy` 里同一处各记一次。
     fn add_the_reads_of_the_writer<Device: BlockDevice>(
         &mut self,
         writer: &PoolWriter<'_, Device>,
     ) {
+        let mapping_keys_of_the_writer =
+            writer.mapping_keys_of_old_data_units_read_back_with_one_copy_failing();
+        assert_eq!(
+            u64::try_from(mapping_keys_of_the_writer.len())
+                .expect("一次发布读回的单元数装得进 u64"),
+            writer.old_data_unit_reads_back_with_one_copy_failing(),
+            "写入口上只坏一份的读回次数与映射 key 一次一把：两份账同一处各记一次"
+        );
+        let reads_of_units_read_back_before_in_this_change = self
+            .keep_the_mapping_keys_not_read_back_before_in_this_change(mapping_keys_of_the_writer);
         self.reads = self
             .reads
             .checked_add(writer.old_data_unit_reads_back_with_one_copy_failing())
+            .and_then(|reads| reads.checked_sub(reads_of_units_read_back_before_in_this_change))
             .expect("一次改动里只坏一份的读回次数装得进 u64");
-        self.mapping_keys
-            .extend(writer.mapping_keys_of_old_data_units_read_back_with_one_copy_failing());
+    }
+
+    /// 一次发布读回时只坏一份的那几把映射 key 里，这次改动前面的发布没读回过的记上（同一把只记一次）；交回读回过的有几把（这几次不再加次数）。
+    fn keep_the_mapping_keys_not_read_back_before_in_this_change(
+        &mut self,
+        mapping_keys_of_one_publish: Vec<Vec<u8>>,
+    ) -> u64 {
+        let mut reads_of_units_read_back_before_in_this_change: u64 = 0;
+        // 迭代上界是这一次发布读回时只坏一份的单元数；跨轮携带的是已记的 key 与读回过的次数。
+        for mapping_key in mapping_keys_of_one_publish {
+            if self.mapping_keys.contains(&mapping_key) {
+                reads_of_units_read_back_before_in_this_change += 1;
+            } else {
+                self.mapping_keys.push(mapping_key);
+            }
+        }
+        reads_of_units_read_back_before_in_this_change
     }
 }
 
@@ -2038,11 +2068,18 @@ fn publish_the_queue_until_it_is_empty<Device: BlockDevice>(
 /// 那一批发成就把它追加进 `versions_published`、换成 `current`。计划只算一次：每次发之前发布路径照它接在后面的那一版核计划
 /// （计划对不上在任何写之前拒，`RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition` 这一类）。
 ///
+/// `mapping_keys_read_while_mounting` 是可写挂载读盘建上一版时读到只坏一份的那几个单元（`compaction_plan::CompactionOfThisMount` 记下的映射 key，
+/// 用户 2026-11-05 定「可写挂载时搬，冷恢复只计数」）：其中到 `current` 这一版仍在中央映射里或留存条目里的那几个
+/// （`transaction::relocations_of_the_units_read_with_one_copy_failing_while_mounting`）接在上面那几个后面、进同一批，同一条路径、同一套被拒推抬 F；
+/// 结局报在同一个成员里（[`RelocationOfTheOldDataUnitsKeptForTheSnapshots`]）。挂载那一串里不另发：另发就是挂载的第七段写，挂载分段与二次崩溃、
+/// 取号之前的续做预演都要跟着改，下一次用户改动之后这一处已有搬迁路径与它的崩溃点（判决 `research/prompts/m4-close-code-r3-main-verification.md`
+/// 第三节第 7 条「挑走已有路径改动最小的那一处」）。
+///
 /// # Panics
 /// `current` 不是有树的一版：调用方只在一次用户改动发成之后调，发成的都是有树的一版（`publish_user_change` 开头拒了没树的那一形）。
 #[allow(
     clippy::too_many_arguments,
-    reason = "参数各是一样东西：参数、盘表、分配器、现行那一版、实例代号、影子账那一臂、会话这一侧的整理状态、读回只坏一份的 key、这次调用发成的各版"
+    reason = "参数各是一样东西：参数、盘表、分配器、现行那一版、实例代号、影子账那一臂、会话这一侧的整理状态、读回只坏一份的 key、挂载时读到只坏一份的 key、这次调用发成的各版"
 )]
 fn relocate_the_old_data_units_kept_for_the_snapshots<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
@@ -2053,15 +2090,23 @@ fn relocate_the_old_data_units_kept_for_the_snapshots<Device: BlockDevice>(
     shadow_ledger: ShadowLedger,
     compaction: &mut CompactionOfThisMount,
     mapping_keys: &[Vec<u8>],
+    mapping_keys_read_while_mounting: &[Vec<u8>],
     versions_published: &mut VersionsPublishedInThisChange,
 ) -> RelocationOfTheOldDataUnitsKeptForTheSnapshots {
     let version_after_the_change = current
         .version_of_trees()
         .expect("一次用户改动发成之后现行那一版是有树的一版");
-    let planned = relocations_of_old_data_units_kept_for_the_snapshots(
+    let mut planned = relocations_of_old_data_units_kept_for_the_snapshots(
         version_after_the_change,
         mapping_keys,
     );
+    let planned_for_the_units_read_while_mounting =
+        relocations_of_the_units_read_with_one_copy_failing_while_mounting(
+            version_after_the_change,
+            mapping_keys_read_while_mounting,
+            &planned,
+        );
+    planned.extend(planned_for_the_units_read_while_mounting);
     if planned.is_empty() {
         return RelocationOfTheOldDataUnitsKeptForTheSnapshots::NotNeeded;
     }
@@ -2427,6 +2472,17 @@ impl MountedSession {
             .unit_reads_with_one_copy_failing_while_mounting()
     }
 
+    /// 这次可写挂载读盘建上一版时读到只坏一份、还没交给另发那一批的那几个单元的映射 key（挂载交回之前按挂载之后那一版认的，
+    /// `transaction::mapping_keys_of_the_units_at_these_locations`）：下一次用户改动发成之后另发的那一批取走它们、照快照下另发那批同一条路径搬
+    /// （[`Self::publish_user_change`]，用户 2026-11-05 定「可写挂载时搬」），取走之后是空的。
+    #[must_use]
+    pub fn mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting_not_yet_relocated(
+        &self,
+    ) -> &[Vec<u8>] {
+        self.compaction
+            .mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting()
+    }
+
     /// 发布一次用户改动：被拒是空间不够的那两种（[`refusal_is_short_of_space`]）就推一串抬 F 的空发布、再重判，
     /// 直到发成或不再推（D16（发布语义） 已定项 1「准入」那一行）。发成时 `current` 换成新的一版；推了的那几串已落盘，
     /// 没发成时 `current` 停在最后一次落盘的空发布上。写入口只按会话持着的那份参数建（`parameters_and_device_table`），
@@ -2538,6 +2594,10 @@ impl MountedSession {
         // 读回时只坏一份、改动之后仍被活快照钉着的旧单元，照已有的搬迁路径另发一批（用户 2026-11-04 定「另发一批搬迁修好」，
         // 被空间拒「推抬 F 再试一遍」）；没有活快照钉着的那几个在改动那次发布里已经重写、旧落点照释放之前读盘核隔离，不进这一批。
         // 这一批没发成也不让改动算没发成：改动那一版已落盘。
+        // 可写挂载读盘建上一版时读到只坏一份的那几个单元（挂载交回之前记下的映射 key）并进同一批（用户 2026-11-05 定「可写挂载时搬」）：
+        // 挂载之后的头一次用户改动发成时取走，搬没搬成都只试这一次；这次改动已经把它整个重写了的不进计划。
+        let mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting = compaction
+            .take_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting();
         published.relocation_of_the_old_data_units_kept_for_the_snapshots =
             relocate_the_old_data_units_kept_for_the_snapshots(
                 parameters,
@@ -2548,6 +2608,7 @@ impl MountedSession {
                 *shadow_ledger,
                 compaction,
                 &old_data_unit_reads_back.mapping_keys,
+                &mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting,
                 &mut published.versions_published,
             );
         // 改动发成：整理「已经过去的发布数」从这里重新数（判决 G8 `B_gc`）。起跑判据照旧在这里查一次（判决 G8 丁），整理却挪出了用户改动的
@@ -6715,3 +6776,36 @@ mod tests {
         }
     }
 }
+
+#[cfg(test)]
+mod old_data_unit_reads_back_with_one_copy_failing_in_this_change_tests {
+    use super::OldDataUnitReadsBackWithOneCopyFailingInThisChange;
+
+    /// 被空间拒、推一串抬 F 之后重发同一个改动那一形缩小（判决 m4-close-code-r3 第三节第 6 条）：头一次发布读回单元 A、第二次发布又读回 A 与 B，
+    /// 第二次里 A 是这次改动读回过的、交回 1；账上 A、B 各一把，按读到的先后。
+    #[test]
+    fn a_unit_read_back_again_by_a_later_publish_of_the_same_change_is_kept_once() {
+        let mut reads_back = OldDataUnitReadsBackWithOneCopyFailingInThisChange::nothing_read_yet();
+        let unit_a = vec![1u8; 27];
+        let unit_b = vec![2u8; 27];
+        assert_eq!(
+            reads_back
+                .keep_the_mapping_keys_not_read_back_before_in_this_change(vec![unit_a.clone()]),
+            0,
+            "头一次发布读回 A：这次改动里没读回过"
+        );
+        assert_eq!(
+            reads_back.keep_the_mapping_keys_not_read_back_before_in_this_change(vec![
+                unit_a.clone(),
+                unit_b.clone()
+            ]),
+            1,
+            "重发那一次又读回 A：A 读回过、只算一次"
+        );
+        assert_eq!(
+            reads_back.mapping_keys,
+            vec![unit_a, unit_b],
+            "账上 A、B 各一把、按读到的先后"
+        );
+    }
+}
```

```diff
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index abb673c..1f14075 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -8,7 +8,7 @@
 //! （I-8.6（反向链算法），前一条不在盘上不判）、提交标记齐全、在飞上限之内、点名单元逐项验过，
 //! 施加一条记录 = 把所选根的四个字段换成记录新根段里的（D23（journal 的角色与格式） 已定项 15）。
 
-use std::cell::{Cell, OnceCell};
+use std::cell::{Cell, OnceCell, RefCell};
 use std::collections::{BTreeMap, BTreeSet};
 
 use singlefs_format::{
@@ -187,16 +187,22 @@ pub trait PoolReader {
     /// 按一组位置条目读一个单元时只坏一份（D4（校验和位置） 已定项 4「读到只坏一份时计数」）：同一组里前一条读不回字节或整单元校验和
     /// 对不上，后一条对上、交回了它那一份。`locations` 是那一组两条位置条目（哪个单元由它们认）。默认什么都不做；要取数的读者
     /// 套 [`PoolReaderCountingUnitReadsWithOneCopyFailing`]。两条位置提示都没对上、转去查中央映射的那一回不算（提示过期与两份都坏分不开），
-    /// 映射那一组里只坏一份的照算。
+    /// 映射那一组里只坏一份的照算；按提示读到只坏一份而提示不是这个单元此刻在映射里的落点也不报
+    /// （`PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition`）。
     fn note_that_a_unit_was_read_with_one_copy_failing(&self, _locations: &[LocationEntry; 2]) {}
 }
 
 /// 套在一个读者外面数「只坏一份的读」（[`PoolReader::note_that_a_unit_was_read_with_one_copy_failing`]）：读与别的几样报告原样交给里面那一份，
 /// 只坏一份的那一报另加一次计数。计数只在内存里，随这个套子活着：冷恢复读不挂载，只计数、报给读者，不搬
 /// （C946（只坏一份的读没有计数也没有搬迁修复）的取数接口）。
+/// 同一个单元的同一份坏副本（同一组位置条目）套着的这一程里只计一次：可写挂载建上一版时实例表链经两处读两遍
+/// （判决 `research/prompts/m4-close-code-r3-main-verification.md` 第二节 T2 ①），退到的那一版建不出、再建所选那一版时又读一遍同几个单元，
+/// 都是同一份坏副本，一次挂载只算一次。
 pub struct PoolReaderCountingUnitReadsWithOneCopyFailing<'reader, Reader: PoolReader + ?Sized> {
     reader: &'reader Reader,
     unit_reads_with_one_copy_failing: Cell<u64>,
+    /// 计过的那几组位置条目，按第一次报的先后。
+    locations_of_the_units_counted: RefCell<Vec<[LocationEntry; 2]>>,
 }
 
 impl<'reader, Reader: PoolReader + ?Sized>
@@ -207,14 +213,22 @@ impl<'reader, Reader: PoolReader + ?Sized>
         PoolReaderCountingUnitReadsWithOneCopyFailing {
             reader,
             unit_reads_with_one_copy_failing: Cell::new(0),
+            locations_of_the_units_counted: RefCell::new(Vec::new()),
         }
     }
 
-    /// 套上之后读到只坏一份的次数。
+    /// 套上之后读到只坏一份的次数：同一组位置条目只算一次。
     #[must_use]
     pub fn unit_reads_with_one_copy_failing(&self) -> u64 {
         self.unit_reads_with_one_copy_failing.get()
     }
+
+    /// 套上之后读到只坏一份的那几个单元的位置条目（一个单元一组，按第一次报的先后）：可写挂载交回之前按它们认映射 key，
+    /// 交给会话下一次用户改动发成之后另发的那一批搬迁（`mounted_session::MountedSession::publish_user_change`）。
+    #[must_use]
+    pub fn locations_of_the_units_read_with_one_copy_failing(&self) -> Vec<[LocationEntry; 2]> {
+        self.locations_of_the_units_counted.borrow().clone()
+    }
 }
 
 impl<Reader: PoolReader + ?Sized> PoolReader
@@ -284,11 +298,126 @@ impl<Reader: PoolReader + ?Sized> PoolReader
     fn note_that_a_unit_was_read_with_one_copy_failing(&self, locations: &[LocationEntry; 2]) {
         self.reader
             .note_that_a_unit_was_read_with_one_copy_failing(locations);
+        if self
+            .locations_of_the_units_counted
+            .borrow()
+            .contains(locations)
+        {
+            return;
+        }
+        self.locations_of_the_units_counted
+            .borrow_mut()
+            .push(*locations);
         self.unit_reads_with_one_copy_failing
             .set(self.unit_reads_with_one_copy_failing.get() + 1);
     }
 }
 
+/// 套在一个读者外面按位置条目读一个进映射的单元（`mapping_key` 那一个）：只坏一份那一报（[`PoolReader::note_that_a_unit_was_read_with_one_copy_failing`]）
+/// 只在读的那一组位置条目就是这个单元此刻的权威落点——中央映射里它那两条位置条目（D19（块指针的结构与宽度预算） 已定项 8）——时往里转；
+/// 别的读与报告原样交给里面那一份。提示是盘上 extent 叶、父条目里记的，单元被整理搬走之后提示还指旧槽，旧槽上一份被盖掉、另一份还是旧字节，
+/// 按提示读就是「只坏一份」，而映射里的权威落点两份都好：旧槽上坏的那一份不归这个单元计（判决 `research/prompts/m4-close-code-r3-main-verification.md`
+/// 第二节 T2 ②、第三节第 4 条）。映射里查不出（没有这把 key、查映射本身判红）同样不转：这一读已经读成，判红留给走到映射那一步的读者照报。
+/// 经映射回退的那一读读的就是映射里那两条，照转。没转的那一报记下（[`Self::dropped_a_one_copy_failing_note_off_the_mapping_position`]）。
+pub(crate) struct PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition<'reader> {
+    reader: &'reader dyn PoolReader,
+    mapping_key: &'reader [u8],
+    central_mapping_locations_of_key: &'reader CentralMappingLocationsOfKey<'reader>,
+    one_copy_failing_note_dropped: Cell<bool>,
+}
+
+impl<'reader> PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition<'reader> {
+    pub(crate) fn over(
+        reader: &'reader dyn PoolReader,
+        mapping_key: &'reader [u8],
+        central_mapping_locations_of_key: &'reader CentralMappingLocationsOfKey<'reader>,
+    ) -> Self {
+        Self {
+            reader,
+            mapping_key,
+            central_mapping_locations_of_key,
+            one_copy_failing_note_dropped: Cell::new(false),
+        }
+    }
+
+    /// 有一报只坏一份因为读的那一组不是映射里的落点而没往里转。
+    pub(crate) fn dropped_a_one_copy_failing_note_off_the_mapping_position(&self) -> bool {
+        self.one_copy_failing_note_dropped.get()
+    }
+}
+
+impl PoolReader for PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition<'_> {
+    fn device_identities(&self) -> Vec<DeviceIdentity> {
+        self.reader.device_identities()
+    }
+    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
+        self.reader.device_size_in_bytes(device)
+    }
+    fn read(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        length: usize,
+    ) -> Option<Vec<u8>> {
+        self.reader.read(device, offset, length)
+    }
+    fn journal_record_offsets_hint(
+        &self,
+        device: DeviceIdentity,
+        ring_start: DeviceOffsetInBytes,
+        ring_bytes: u64,
+    ) -> Option<Vec<DeviceOffsetInBytes>> {
+        self.reader
+            .journal_record_offsets_hint(device, ring_start, ring_bytes)
+    }
+    fn reread_after_a_bad_read(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        length: usize,
+    ) -> Option<Vec<u8>> {
+        self.reader.reread_after_a_bad_read(device, offset, length)
+    }
+    fn keep_the_self_verified_reread(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+    ) {
+        self.reader
+            .keep_the_self_verified_reread(device, offset, bytes);
+    }
+    fn note_that_the_bytes_read_self_verify(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+    ) {
+        self.reader
+            .note_that_the_bytes_read_self_verify(device, offset, bytes);
+    }
+    fn note_that_the_bytes_read_fail_self_verification(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        length_in_bytes: usize,
+    ) {
+        self.reader.note_that_the_bytes_read_fail_self_verification(
+            device,
+            offset,
+            length_in_bytes,
+        );
+    }
+    fn note_that_a_unit_was_read_with_one_copy_failing(&self, locations: &[LocationEntry; 2]) {
+        match (self.central_mapping_locations_of_key)(self.mapping_key) {
+            Ok(Some(mapped_locations)) if mapped_locations == *locations => self
+                .reader
+                .note_that_a_unit_was_read_with_one_copy_failing(locations),
+            Ok(Some(_) | None) | Err(_) => self.one_copy_failing_note_dropped.set(true),
+        }
+    }
+}
+
 /// 一遍读是第一次读，还是读坏之后当场重读的那一遍（D19（块指针的结构与宽度预算） 已定项 5、D16（发布语义） 已定项 1「根槽这一次读坏」那一行
 /// 与「重读一次」：只重读一次，重读要再读一次盘）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
@@ -882,11 +1011,18 @@ pub(crate) fn read_mapped_tree_node_via_hint_then_central_mapping(
     stale_location_hint_hops: &mut usize,
 ) -> Result<Vec<u8>, RecoveryFailure> {
     let unit_bytes = class.unit_bytes();
+    let mapping_key = mapping_key_for_node(class.unit_class_tag(), *pointer);
+    // 只坏一份按这个单元此刻的权威落点判：按提示读到只坏一份而提示不是映射里那两条（提示过期）不报
+    // （[`PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition`]，判决 m4-close-code-r3 第三节第 4 条）。
+    let reader: &dyn PoolReader = &PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition::over(
+        reader,
+        &mapping_key,
+        central_mapping_locations_of_key,
+    );
     if let Ok(bytes) = read_unit_via_locations(reader, &pointer.locations, unit_bytes) {
         return Ok(bytes);
     }
     *stale_location_hint_hops += 1;
-    let mapping_key = mapping_key_for_node(class.unit_class_tag(), *pointer);
     let Some(mapped_locations) = central_mapping_locations_of_key(&mapping_key)? else {
         return Err(RecoveryFailure::MappingMiss {
             slot: pointer.locations[0].slot,
@@ -935,11 +1071,17 @@ fn read_data_unit_via_hint_then_central_mapping(
     data_unit_stale_location_hint_hops: &mut usize,
 ) -> Result<DataUnitReadThroughTheCentralMapping, RecoveryFailure> {
     let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
+    let mapping_key = mapping_key_for_data(data_pointer.head, data_pointer.write_order);
+    // 只坏一份按这个单元此刻的权威落点判（同 [`read_mapped_tree_node_via_hint_then_central_mapping`]）。
+    let reader: &dyn PoolReader = &PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition::over(
+        reader,
+        &mapping_key,
+        central_mapping_locations_of_key,
+    );
     if let Ok(bytes) = read_unit_via_locations(reader, &data_pointer.locations, data_unit_bytes) {
         return Ok(DataUnitReadThroughTheCentralMapping::Content(bytes));
     }
     *data_unit_stale_location_hint_hops += 1;
-    let mapping_key = mapping_key_for_data(data_pointer.head, data_pointer.write_order);
     let Some(mapped_data_unit_locations) = central_mapping_locations_of_key(&mapping_key)? else {
         return Ok(
             DataUnitReadThroughTheCentralMapping::MissingFromTheMapping {
@@ -3828,9 +3970,11 @@ fn inode_tree_read_and_judged(
                 what: "inode 叶容器没有记录",
             });
         }
-        // 可写挂载拿重建出来的这一版接着写：写者的前提（全部叶的记录按叶序严格升序）在这里判，不收两片交叠的一版。
+        // 可写挂载拿重建出来的这一版接着写：写者的前提（全部叶的记录按叶序严格升序）在这里判，不收两片交叠的一版；
+        // 分隔 key 大于前面那几片的最大记录号（I-9.12 第三句）同一处判。
         largest_inode_number_of_the_leaves_read =
             inode_records_of_the_leaf_ascend_after_the_leaves_before(
+                separator_key,
                 largest_inode_number_of_the_leaves_read,
                 &records,
             )?;
@@ -5997,17 +6141,55 @@ pub const INODE_RECORDS_OF_A_LEAF_DO_NOT_STRICTLY_ASCEND: &str =
 pub const INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM: &str =
     "inode 叶容器的第一条记录号不大于叶序在它前面那几片的最大记录号（两片叶交叠）";
 
+/// 指着一片 inode 叶容器的那条内部条目的分隔 key 不大于叶序在它前面那几片的最大记录号（上一个孩子的最大 key）时
+/// [`RecoveryFailure::InvariantViolated`] 的 `detail`（[`inode_records_of_the_leaf_ascend_after_the_leaves_before`]，I-9.12 第三句）。
+pub const INODE_SEPARATOR_KEY_NOT_ABOVE_THE_LEAVES_BEFORE_IT: &str =
+    "inode 树内部条目的分隔 key 不大于叶序在它前面那几片的最大记录号（上一个孩子的最大 key）";
+
+/// 一片 inode 叶容器里有一条记录号小于指着它的那条内部条目的分隔 key、或小于这片叶的容器号时 [`RecoveryFailure::InvariantViolated`] 的 `detail`
+/// （[`inode_record_is_not_below_the_separator_key_and_the_container`]，I-9.4 第一句与 I-9.12 第二句「分隔 key ≤ 孩子的 key 区间下界」）。
+pub const INODE_RECORD_BELOW_THE_SEPARATOR_KEY_OR_THE_CONTAINER: &str =
+    "inode 记录号小于分隔 key 或容器号";
+
+/// 一片 inode 叶容器里的一条记录号不小于指着这片叶的那条内部条目的分隔 key、也不小于这片叶的容器号（I-9.4 第一句；分隔 key 那一半是
+/// I-9.12 第二句「第 i 条的分隔 key ≤ 第 i 个孩子的 key 区间下界」，逐条记录判就判到了下界）。读 inode 树的四个读者——可写挂载重建、冷走读、
+/// 快照第一个文件的冷读与只读挂载打开池——逐条记录调这一个，同一张镜像一个结局（判决 `research/prompts/m4-close-code-r3-main-verification.md`
+/// 第三节第 2 条：只读挂载此前丢了分隔 key）。
+///
+/// # Errors
+/// 记录号小于分隔 key 或容器号 ⇒ [`RecoveryFailure::InvariantViolated`]（I-9.4，[`INODE_RECORD_BELOW_THE_SEPARATOR_KEY_OR_THE_CONTAINER`]）。
+pub(crate) fn inode_record_is_not_below_the_separator_key_and_the_container(
+    record_inode: u64,
+    separator_key: u64,
+    container: u64,
+) -> Result<(), RecoveryFailure> {
+    if record_inode < separator_key || record_inode < container {
+        return Err(RecoveryFailure::InvariantViolated {
+            invariant: "I-9.4",
+            detail: INODE_RECORD_BELOW_THE_SEPARATOR_KEY_OR_THE_CONTAINER,
+        });
+    }
+    Ok(())
+}
+
 /// 读回来的一片 inode 叶容器（按根的条目次序逐片喂）照写者的前提判：全部叶的记录按叶序连起来按 inode 号严格升序
 /// （`inode_tree::leaf_containers_are_well_formed` 的前一半，[`crate::inode_tree::write_records_into_leaf_containers`] 的 `requires`；
 /// 按叶序折半找 `TransactionOutput::inode_record_found_by_its_leaf` 也靠它）。盘上不满足这一条的一版（两片叶交叠违反 I-9.12，
 /// 一片里不升序违反 I-9.4 第四句）读者不收：收下了写者的前提就不成立，按叶序折半找漏找、维护的 n_live 与 n_记录漂走
 /// （判决 `research/prompts/m4-close-code-r2-main-verification.md` 第二节 T4，第三节取改法甲）。`largest_of_the_leaves_before` 是叶序在它前面那几片的
 /// 最大记录号（头一片交 `None`），交回读完这一片之后的最大记录号；没有记录的叶由调用方另拒，这里交回原来那个。
+/// 再判 I-9.12 第三句「第 i 条的分隔 key > 第 i−1 个孩子的 key 区间上界」：`separator_key_of_the_leaf` 是指着这片叶的那条内部条目的分隔 key，
+/// 要大于前面那几片的最大记录号（判决 `research/prompts/m4-close-code-r3-main-verification.md` 第三节第 1 条，与池级 checker 同一结局）。
+/// 第二句「分隔 key ≤ 这个孩子的最小 key」由调用方逐条记录先判（[`inode_record_is_not_below_the_separator_key_and_the_container`]）；
+/// 「条目按分隔 key 严格递增」由这两句合出来（叶都不空：前一条的分隔 key ≤ 前一片的最小号 ≤ 前一片的最大号 < 这一条的分隔 key），不另判。
+/// 两片交叠的一形（第一条不大于前面的最大号）先报交叠：分隔 key 不大于第一条，交叠时它也不大于前面的最大号，两句都红、报先判的那一句。
 ///
 /// # Errors
 /// 片里相邻两条不严格升序 ⇒ [`RecoveryFailure::InvariantViolated`]（I-9.4，[`INODE_RECORDS_OF_A_LEAF_DO_NOT_STRICTLY_ASCEND`]）；
-/// 第一条不大于前面那几片的最大记录号 ⇒ 同一成员（I-9.12，[`INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM`]）。
+/// 第一条不大于前面那几片的最大记录号 ⇒ 同一成员（I-9.12，[`INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM`]）；
+/// 分隔 key 不大于前面那几片的最大记录号 ⇒ 同一成员（I-9.12，[`INODE_SEPARATOR_KEY_NOT_ABOVE_THE_LEAVES_BEFORE_IT`]）。
 pub(crate) fn inode_records_of_the_leaf_ascend_after_the_leaves_before(
+    separator_key_of_the_leaf: u64,
     largest_of_the_leaves_before: Option<u64>,
     records_of_the_leaf: &[InodeRecord],
 ) -> Result<Option<u64>, RecoveryFailure> {
@@ -6020,6 +6202,14 @@ pub(crate) fn inode_records_of_the_leaf_ascend_after_the_leaves_before(
             detail: INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM,
         });
     }
+    if largest_of_the_leaves_before
+        .is_some_and(|largest_before| largest_before >= separator_key_of_the_leaf)
+    {
+        return Err(RecoveryFailure::InvariantViolated {
+            invariant: "I-9.12",
+            detail: INODE_SEPARATOR_KEY_NOT_ABOVE_THE_LEAVES_BEFORE_IT,
+        });
+    }
     if records_of_the_leaf
         .windows(2)
         .any(|adjacent| adjacent[0].inode >= adjacent[1].inode)
@@ -6805,12 +6995,11 @@ fn inode_leaf_container_judged_against_its_entry(
         let record = InodeRecord::parse(record_bytes).ok_or(RecoveryFailure::UnitMalformed {
             what: "inode 记录",
         })?;
-        if record.inode < separator_key || record.inode < identity.container {
-            return Err(RecoveryFailure::InvariantViolated {
-                invariant: "I-9.4",
-                detail: "inode 记录号小于分隔 key 或容器号",
-            });
-        }
+        inode_record_is_not_below_the_separator_key_and_the_container(
+            record.inode,
+            separator_key,
+            identity.container,
+        )?;
         records.push(record);
     }
     Ok(records)
@@ -7407,9 +7596,10 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
             root,
             expected_filesystem_identifier,
         )?;
-        // 冷走读与可写挂载重建同一判：全部叶的记录按叶序严格升序（写者的前提），不收两片交叠的一版。
+        // 冷走读与可写挂载重建同一判：全部叶的记录按叶序严格升序（写者的前提），不收两片交叠的一版；分隔 key 那一句同判。
         largest_inode_number_of_the_leaves_read =
             inode_records_of_the_leaf_ascend_after_the_leaves_before(
+                separator_key,
                 largest_inode_number_of_the_leaves_read,
                 &records,
             )?;
@@ -7595,6 +7785,7 @@ pub fn read_the_first_file_of_a_snapshot(
         // 快照的冷读与冷走读、可写挂载重建、只读挂载同一判：全部叶的记录按叶序严格升序，同一张两片交叠的镜像四个读者一个结局。
         largest_inode_number_of_the_leaves_of_the_snapshot =
             inode_records_of_the_leaf_ascend_after_the_leaves_before(
+                separator_key,
                 largest_inode_number_of_the_leaves_of_the_snapshot,
                 &records,
             )?;
@@ -8844,9 +9035,12 @@ mod back_chain_judgement_tests {
 #[cfg(test)]
 mod inode_leaf_records_ascending_across_the_leaves_tests {
     use super::{
+        inode_record_is_not_below_the_separator_key_and_the_container,
         inode_records_of_the_leaf_ascend_after_the_leaves_before, RecoveryFailure,
         INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM,
         INODE_RECORDS_OF_A_LEAF_DO_NOT_STRICTLY_ASCEND,
+        INODE_RECORD_BELOW_THE_SEPARATOR_KEY_OR_THE_CONTAINER,
+        INODE_SEPARATOR_KEY_NOT_ABOVE_THE_LEAVES_BEFORE_IT,
     };
     use crate::address::CheckpointTxg;
     use crate::records::{
@@ -8868,14 +9062,67 @@ mod inode_leaf_records_ascending_across_the_leaves_tests {
             .collect()
     }
 
-    /// 逐片喂，交回最后的最大记录号或头一处拒。
+    /// 逐片喂，每片的分隔 key 取它的第一条记录号（诞生时分隔 key 是容器号、也是最小号），交回最后的最大记录号或头一处拒。
     fn judged_leaf_by_leaf(leaves: &[&[u64]]) -> Result<Option<u64>, RecoveryFailure> {
-        leaves.iter().try_fold(None, |largest_before, leaf| {
-            inode_records_of_the_leaf_ascend_after_the_leaves_before(
-                largest_before,
-                &records_numbered(leaf),
-            )
-        })
+        let leaves_with_their_separators: Vec<(u64, &[u64])> = leaves
+            .iter()
+            .map(|leaf| (*leaf.first().expect("每片叶至少一条记录"), *leaf))
+            .collect();
+        judged_leaf_by_leaf_with_their_separators(&leaves_with_their_separators)
+    }
+
+    /// 逐片喂（分隔 key、这片的记录号），交回最后的最大记录号或头一处拒。
+    fn judged_leaf_by_leaf_with_their_separators(
+        leaves: &[(u64, &[u64])],
+    ) -> Result<Option<u64>, RecoveryFailure> {
+        leaves
+            .iter()
+            .try_fold(None, |largest_before, (separator_key, leaf)| {
+                inode_records_of_the_leaf_ascend_after_the_leaves_before(
+                    *separator_key,
+                    largest_before,
+                    &records_numbered(leaf),
+                )
+            })
+    }
+
+    /// 攻方 `separator_200` 那一形缩小：两片叶 [1, 2, 3] 与 [5, 7]，右片的分隔 key 改成 3（不大于左片最大号 3，仍不大于右片最小号 5）：
+    /// 记录照叶序严格升序，拒在 I-9.12 第三句。分隔 key 4 落在 (3, 5] 里照收，交回最大号 7。
+    #[test]
+    fn a_separator_key_not_above_the_largest_record_of_the_leaves_before_is_refused() {
+        assert_eq!(
+            judged_leaf_by_leaf_with_their_separators(&[(1, &[1, 2, 3]), (3, &[5, 7])]),
+            Err(RecoveryFailure::InvariantViolated {
+                invariant: "I-9.12",
+                detail: INODE_SEPARATOR_KEY_NOT_ABOVE_THE_LEAVES_BEFORE_IT,
+            })
+        );
+        assert_eq!(
+            judged_leaf_by_leaf_with_their_separators(&[(1, &[1, 2, 3]), (4, &[5, 7])]),
+            Ok(Some(7))
+        );
+    }
+
+    /// 记录号小于分隔 key（攻方 `separator_235` 那一形：分隔 key 235、头一条 234）或小于容器号：拒在 I-9.4；等于两者照收。
+    #[test]
+    fn a_record_below_the_separator_key_or_the_container_is_refused_on_the_container_number_invariant(
+    ) {
+        let below = Err(RecoveryFailure::InvariantViolated {
+            invariant: "I-9.4",
+            detail: INODE_RECORD_BELOW_THE_SEPARATOR_KEY_OR_THE_CONTAINER,
+        });
+        assert_eq!(
+            inode_record_is_not_below_the_separator_key_and_the_container(234, 235, 234),
+            below
+        );
+        assert_eq!(
+            inode_record_is_not_below_the_separator_key_and_the_container(234, 234, 235),
+            below
+        );
+        assert_eq!(
+            inode_record_is_not_below_the_separator_key_and_the_container(234, 234, 234),
+            Ok(())
+        );
     }
 
     /// 两片叶 [1, 2, 3] 与 [4, 7]：连起来严格升序，交回最大号 7。
@@ -8907,3 +9154,74 @@ mod inode_leaf_records_ascending_across_the_leaves_tests {
         assert_eq!(judged_leaf_by_leaf(&[&[1, 1]]), out_of_order);
     }
 }
+
+#[cfg(test)]
+mod counting_unit_reads_with_one_copy_failing_tests {
+    use super::{PoolReader, PoolReaderCountingUnitReadsWithOneCopyFailing};
+    use crate::address::{DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
+    use crate::pointer::LocationEntry;
+
+    /// 一块盘都没有的读者：这几条用例只往套子里报「只坏一份」，不读盘。
+    struct PoolReaderWithoutDevices;
+
+    impl PoolReader for PoolReaderWithoutDevices {
+        fn device_identities(&self) -> Vec<DeviceIdentity> {
+            Vec::new()
+        }
+        fn device_size_in_bytes(&self, _device: DeviceIdentity) -> Option<u64> {
+            None
+        }
+        fn read(
+            &self,
+            _device: DeviceIdentity,
+            _offset: DeviceOffsetInBytes,
+            _length: usize,
+        ) -> Option<Vec<u8>> {
+            None
+        }
+        fn journal_record_offsets_hint(
+            &self,
+            _device: DeviceIdentity,
+            _ring_start: DeviceOffsetInBytes,
+            _ring_bytes: u64,
+        ) -> Option<Vec<DeviceOffsetInBytes>> {
+            None
+        }
+    }
+
+    /// 两块盘同一个槽上的一个单元的两条位置条目。
+    fn locations_of_a_unit_at(slot: u64) -> [LocationEntry; 2] {
+        [DeviceIdentity(0), DeviceIdentity(1)].map(|device| LocationEntry {
+            device,
+            slot: SlotNumber(slot),
+            unit_checksum: 7,
+        })
+    }
+
+    /// 同一个单元的同一份坏副本报两次（实例表链经两处读两遍那一形，判决 m4-close-code-r3 第二节 T2 ①）只计一次；另一个单元再报一次计第二次；
+    /// 交出的位置条目按第一次报的先后、一个单元一组。
+    #[test]
+    fn the_same_unit_reported_twice_is_counted_once_and_another_unit_once_more() {
+        let counting = PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(
+            &PoolReaderWithoutDevices,
+        );
+        counting.note_that_a_unit_was_read_with_one_copy_failing(&locations_of_a_unit_at(100));
+        counting.note_that_a_unit_was_read_with_one_copy_failing(&locations_of_a_unit_at(100));
+        assert_eq!(
+            counting.unit_reads_with_one_copy_failing(),
+            1,
+            "同一份坏副本读两遍只计一次"
+        );
+        counting.note_that_a_unit_was_read_with_one_copy_failing(&locations_of_a_unit_at(102));
+        assert_eq!(
+            counting.unit_reads_with_one_copy_failing(),
+            2,
+            "另一个单元另计一次"
+        );
+        assert_eq!(
+            counting.locations_of_the_units_read_with_one_copy_failing(),
+            vec![locations_of_a_unit_at(100), locations_of_a_unit_at(102)],
+            "一个单元一组、按第一次报的先后"
+        );
+    }
+}
```

```diff
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 5aa3b0d..d40a162 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -11701,10 +11701,100 @@ pub fn relocations_of_old_data_units_kept_for_the_snapshots(
     planned
 }
 
+/// 一版里此刻住在 `locations` 这几组位置条目上的那几个单元的映射 key：进中央映射的单元按映射里那两条位置条目认，留存的映射条目
+/// （`version.mapping_entries_kept_for_the_snapshots`）按它记的那两条认；一把 key 只出一次，按 `mapped_units` 的次序、留存的排在后面。
+/// 可写挂载读盘建上一版时读到只坏一份的那几个单元（`recovery::PoolReaderCountingUnitReadsWithOneCopyFailing::locations_of_the_units_read_with_one_copy_failing`）
+/// 在挂载交回之前经它认成 key，交给会话下一次用户改动之后另发的那一批（用户 2026-11-05 定「可写挂载时搬」）。
+/// 豁免映射的三类（映射树自己的节点、树表、实例表）不在映射里、认不出；挂载这一串的发布里已经换了落点的单元（树表、记账树这类每次发布都换下的）
+/// 也认不出——它们的旧落点照释放之前读盘核那一路走。
+#[must_use]
+pub fn mapping_keys_of_the_units_at_these_locations(
+    version: &TransactionOutput,
+    locations: &[[LocationEntry; 2]],
+) -> Vec<Vec<u8>> {
+    let mut mapping_keys: Vec<Vec<u8>> = Vec::new();
+    if locations.is_empty() {
+        return mapping_keys;
+    }
+    let mut central_mapping = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);
+    // 迭代上界是这一版进映射的单元数加留存条目数；跨轮携带的只有已认出的那几把 key。
+    for (_, mapping_key) in &version.mapped_units {
+        if mapping_keys.contains(mapping_key) {
+            continue;
+        }
+        match central_mapping.locations_of_key(mapping_key) {
+            MappingLookup::Found(mapped_locations) if locations.contains(&mapped_locations) => {
+                mapping_keys.push(mapping_key.clone());
+            }
+            MappingLookup::Found(_)
+            | MappingLookup::NodeMalformedOrNoEntryWithThisKey
+            | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => {}
+        }
+    }
+    for (kept_key, kept_locations) in &version.mapping_entries_kept_for_the_snapshots {
+        if !mapping_keys.contains(kept_key) && locations.contains(kept_locations) {
+            mapping_keys.push(kept_key.clone());
+        }
+    }
+    mapping_keys
+}
+
+/// 可写挂载读盘建上一版时读到只坏一份的那几个单元（`mapping_keys`，[`mapping_keys_of_the_units_at_these_locations`] 认出来的）里，到 `version`
+/// 这一版仍在中央映射里或留存条目里的那几个，照已有的搬迁路径另发一批时的计划（用户 2026-11-05 定「可写挂载时搬」；与
+/// [`relocations_of_old_data_units_kept_for_the_snapshots`] 同一批发，`planned_before` 是那一份、已在里面的 key 不重复）：源落点取映射里那两条
+/// 位置条目（或留存条目记的那两条）的槽，跨度按 key 的类标签归到的用户可见角色取（[`user_visible_role_standing_for_a_unit_class`]）。
+/// 到 `version` 这一版已经不在的（这次改动或挂载之后的发布把它整个重写了、旧落点照释放之前读盘核那一路走）不进计划。
+#[must_use]
+pub fn relocations_of_the_units_read_with_one_copy_failing_while_mounting(
+    version: &TransactionOutput,
+    mapping_keys: &[Vec<u8>],
+    planned_before: &[PlannedRelocation],
+) -> Vec<PlannedRelocation> {
+    let mut planned: Vec<PlannedRelocation> = Vec::new();
+    let mut central_mapping = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);
+    // 迭代上界是 `mapping_keys` 的个数；跨轮携带的只有已进计划的那几项。
+    for mapping_key in mapping_keys {
+        if planned_before
+            .iter()
+            .chain(&planned)
+            .any(|relocation| relocation.mapping_key == *mapping_key)
+        {
+            continue;
+        }
+        let Some(role) = mapping_key
+            .first()
+            .and_then(|unit_class| user_visible_role_standing_for_a_unit_class(*unit_class))
+        else {
+            continue;
+        };
+        let source_locations = match central_mapping.locations_of_key(mapping_key) {
+            MappingLookup::Found(mapped_locations) => Some(mapped_locations),
+            MappingLookup::NodeMalformedOrNoEntryWithThisKey
+            | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => version
+                .mapping_entries_kept_for_the_snapshots
+                .iter()
+                .find(|(kept_key, _)| kept_key == mapping_key)
+                .map(|(_, kept_locations)| *kept_locations),
+        };
+        let Some(source_locations) = source_locations else {
+            continue;
+        };
+        planned.push(PlannedRelocation {
+            mapping_key: mapping_key.clone(),
+            source: Placement {
+                slot: source_locations[0].slot,
+                span: role.span_slots(),
+            },
+        });
+    }
+    planned
+}
+
 /// 按一组两条位置条目读一个单元时读到的是哪一份（D4（校验和位置） 已定项 4「读到只坏一份时计数」）。封闭集合，`match` 不写通配臂。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum CopiesOfAUnitRead {
-    /// 头一条就对上，或两条位置提示都没对上、映射那一组头一条对上（提示过期与两份都坏分不开，不算只坏一份）。
+    /// 头一条就对上，或两条位置提示都没对上、映射那一组头一条对上（提示过期与两份都坏分不开，不算只坏一份），
+    /// 或按提示读到只坏一份而提示不是这个单元此刻在映射里的落点（提示过期：旧槽上坏的那一份不归这个单元计）。
     WithoutAFailingCopy,
     /// 同一组里前一条读不回字节或整单元 CRC-32C 对不上，后一条对上：只坏一份。
     FromTheOtherCopyAfterOneFailed,
@@ -11771,7 +11861,27 @@ pub fn read_back_the_old_data_unit_telling_whether_a_copy_failed<Reader: PoolRea
         })
     };
     let bytes = match read_by_the_locations(&pointer.locations) {
-        Some(bytes) => bytes,
+        Some(bytes) => {
+            // 按提示读到只坏一份：这个单元此刻的权威落点是上一版中央映射里那两条位置条目（D19（块指针的结构与宽度预算） 已定项 8），
+            // 提示就是它们才算只坏一份；单元被整理搬走之后提示还指旧槽，旧槽上坏的那一份不归这个单元计
+            // （判决 `research/prompts/m4-close-code-r3-main-verification.md` 第三节第 4 条，与冷走读、可写挂载建上一版、挂载态读同一判）。
+            if copies_read.get() == CopiesOfAUnitRead::FromTheOtherCopyAfterOneFailed {
+                let hints_are_the_mapping_position =
+                    match MappedUnitsOfAVersionReadByRole::of_the_version(previous)
+                        .locations_of_role(role)
+                    {
+                        MappingLookup::Found(mapped_locations) => {
+                            mapped_locations == pointer.locations
+                        }
+                        MappingLookup::NodeMalformedOrNoEntryWithThisKey
+                        | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => false,
+                    };
+                if !hints_are_the_mapping_position {
+                    copies_read.set(CopiesOfAUnitRead::WithoutAFailingCopy);
+                }
+            }
+            bytes
+        }
         None => {
             let mapped_locations =
                 match MappedUnitsOfAVersionReadByRole::of_the_version(previous)
```

