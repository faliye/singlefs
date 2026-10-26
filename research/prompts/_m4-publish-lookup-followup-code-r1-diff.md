# m4-publish-lookup-followup-code-r1：七·6 后续批与 singlefs-core 构建脚本的补丁原文（crates.patch、mutations-append.tsv 原样）（2026-10-22）

## 七·6 后续批 crates.patch

````diff
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 3bba044..414be47 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -3268,11 +3268,27 @@ pub mod lookup_work {
     pub enum LookupSite {
         /// 快照判定那一圈：换下的用户可见角色各取一次映射 key，进 deadlist 的再取映射条目（`deadlist_changes_of_this_publish`）。
         SnapshotJudgement,
-        /// 经映射释放的角色取映射条目：释放判定路径、核位置项指池里两块不同的盘、释放之前读盘核校验和，三处各取一遍。
-        MappingEntriesOfReleasedUnits,
+        /// 经映射释放的角色取映射条目，释放判定路径那一遍（`placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots`）。
+        /// 释放的三处各记各的（七·6 代码轮第三轮判决第三节第 2 条）：一处改回线性，另两处补不上它的数。
+        MappingEntriesOfTheReleaseJudgement,
+        /// 经映射释放的角色取映射条目，核两条位置项各指池里一块不同的盘那一遍（`refuse_mapping_entries_that_do_not_name_two_pool_devices`）。
+        MappingEntriesCheckedForTwoPoolDevices,
+        /// 经映射释放的角色取映射条目，释放之前读盘核校验和那一遍（`copies_failing_the_release_checksum_check`）。
+        MappingEntriesCheckedAgainstTheReleaseChecksums,
+        /// 删快照的批放掉的 deadlist 条目按映射 key 取它留存的映射条目（`deadlist_changes_of_this_publish`）。
+        KeptMappingEntriesReleasedByTheSnapshotDestruction,
+        /// 换下的第一个文件之外那一族角色在上一版里有没有落点：按 inode 号找上一版的那个文件（`has_a_placement_in_the_previous_version`）。
+        FilesOfTheReplacedRolesInThePreviousVersion,
+        /// 发布照抄 extent 树的节点：按角色取上一版那个节点的单元（`build_extent_tree` 的 `carried_node`）。
+        CarriedExtentTreeNodes,
+        /// 发布照抄第一个文件之外一个文件的下段节点：按 inode 号找上一版的那个文件、取那个节点的指针（`build_extent_tree`）。
+        FilesOfTheCarriedExtentLowerNodes,
+        /// 发布照抄记账树、deadlist 树、中央映射树的节点：按角色取上一版那个节点的单元（`build_multi_level_tree`）。
+        CarriedNodesOfTheMultiLevelTrees,
         /// 整理读一版取全部进映射单元的映射条目（`mounted_session::central_mapping_as_the_compaction_reads_it`）。
         CompactionReadsTheCentralMapping,
-        /// 整理发批核计划里每一项的源落点：按映射 key 找角色、取映射条目（`relocation_checked_against_the_version`）。
+        /// 整理发批核计划里每一项的源落点：按映射 key 找角色、取映射条目，找不到的按 key 找留存的映射条目，
+        /// 角色的单元按角色取字节（`relocation_checked_against_the_version`）。
         RelocationSources,
     }
 
@@ -4080,6 +4096,18 @@ impl TransactionOutput {
         FirstEntryOfEachIdentity::of_the_table(&self.mapped_units, |(_, key)| key.as_slice())
     }
 
+    /// `mapping_entries_kept_for_the_snapshots` 按映射 key 的索引（删快照的批取放掉的条目留存的映射条目、整理发批找计划里留存的那一项，
+    /// 七·6 代码轮第三轮判决第三节第 1 条）：同一把 key 取最先那一项，与在表上按 key `iter().find` 同一项。
+    #[must_use]
+    pub fn mapping_entries_kept_for_the_snapshots_by_key(
+        &self,
+    ) -> FirstEntryOfEachIdentity<'_, &[u8], (Vec<u8>, [LocationEntry; 2])> {
+        FirstEntryOfEachIdentity::of_the_table(
+            &self.mapping_entries_kept_for_the_snapshots,
+            |(key, _)| key.as_slice(),
+        )
+    }
+
     /// `files_other_than_the_first` 按 inode 号的索引（[`TransactionOutput::file_other_than_the_first`] 的批量形态，七·6）。
     #[must_use]
     pub fn files_other_than_the_first_by_inode(
@@ -4299,10 +4327,12 @@ pub fn placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots
     roles: &[TransactionUnit],
     kept_for_the_snapshots: &[TransactionUnit],
 ) -> Result<Vec<Placement>, PublishError> {
-    let _inside_the_releases = lookup_work::entering(LookupSite::MappingEntriesOfReleasedUnits);
+    let _inside_the_releases =
+        lookup_work::entering(LookupSite::MappingEntriesOfTheReleaseJudgement);
     let mut placements = Vec::new();
     // 换下的角色逐个取上一版的映射条目：key 与落点都走建一次的索引（七·6），不逐个在 `mapped_units` 上找、逐 key 重解叶。
     let mut previous_read_by_role = MappedUnitsOfAVersionReadByRole::of_the_version(previous);
+    let mut previous_files = FilesOtherThanTheFirstOfAVersionReadByInode::of_the_version(previous);
     for identity in roles_replaced_via_mapping(previous, roles, file_content)
         .into_iter()
         .filter(|identity| !kept_for_the_snapshots.contains(identity))
@@ -4333,7 +4363,8 @@ pub fn placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots
             | TransactionUnit::RetirementLeafContainer(_)
             | TransactionUnit::RetirementRoot
             | TransactionUnit::RetirementNodeBelowTheRoot(_) => {
-                if !has_a_placement_in_the_previous_version(previous, identity) {
+                if !has_a_placement_in_the_previous_version(previous, &mut previous_files, identity)
+                {
                     continue;
                 }
                 match previous_read_by_role.locations_of_role(identity) {
@@ -4461,8 +4492,10 @@ fn roles_replaced_via_mapping(
 /// 这个角色在上一版里有没有落点。这次发布新建的叶容器（末尾分裂出来的右半）在上一版里没有落点 ⇒ 没有东西要释放、要核；
 /// 数据单元同一条：文件变长时新多出来的那几个单元在上一版里没有落点。
 /// 判据取上一版的容器数、单元数，不取「映射里查不到」：查不到要报 `ReleaseNotInMapping`，那是另一件事。
+/// 第一个文件之外那一族按 inode 号找文件走 `previous_files`（它就是 `previous` 的那张表的索引，七·6）。
 fn has_a_placement_in_the_previous_version(
     previous: &TransactionOutput,
+    previous_files: &mut FilesOtherThanTheFirstOfAVersionReadByInode<'_>,
     identity: TransactionUnit,
 ) -> bool {
     match identity {
@@ -4473,18 +4506,20 @@ fn has_a_placement_in_the_previous_version(
             index.0 < u64::try_from(previous.data_pointers.len()).expect("单元数")
         }
         // 第一个文件之外的文件：上一版里那个文件有没有这个单元、这个下段位置（这次第一次写它、或写长了，新多出来的在上一版里没有落点）。
-        TransactionUnit::DataOfAFileOtherThanTheFirst { inode, unit } => previous
+        TransactionUnit::DataOfAFileOtherThanTheFirst { inode, unit } => previous_files
             .file_other_than_the_first(inode)
             .is_some_and(|file| unit.0 < u64::try_from(file.data_pointers.len()).expect("单元数")),
-        TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { inode, position } => previous
-            .file_other_than_the_first(inode)
-            .is_some_and(|file| {
-                file.extent_lower_nodes
-                    .iter()
-                    .any(|(position_in_the_previous_version, _)| {
-                        *position_in_the_previous_version == position
-                    })
-            }),
+        TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { inode, position } => {
+            previous_files
+                .file_other_than_the_first(inode)
+                .is_some_and(|file| {
+                    file.extent_lower_nodes
+                        .iter()
+                        .any(|(position_in_the_previous_version, _)| {
+                            *position_in_the_previous_version == position
+                        })
+                })
+        }
         // 按位置寻址的两棵树的节点：上一版有没有那个位置（这次新长出来的节点在上一版里没有落点）。
         TransactionUnit::ExtentLowerNode(position) => {
             previous.extent_tree.lower_pointer_of(position).is_some()
@@ -4565,6 +4600,39 @@ fn has_a_placement_in_the_previous_version(
     }
 }
 
+/// 一版的 `files_other_than_the_first` 按 inode 号找文件的批量形态（七·6 代码轮第三轮判决第三节第 1 条）：每个 inode 号的答案与
+/// [`TransactionOutput::file_other_than_the_first`]（在表上 `iter().find`，取最先那一项）相同，但走 [`TransactionOutput::files_other_than_the_first_by_inode`]
+/// 那个索引。释放的三处与快照判定那一圈对换下的每个角色问一次它在上一版里有没有落点，逐个在表上找是角色数乘文件数。
+/// 索引在第一次查时才建：一个第一个文件之外的角色都不问的不建。查记在 [`LookupSite::FilesOfTheReplacedRolesInThePreviousVersion`] 那一处。
+pub(crate) struct FilesOtherThanTheFirstOfAVersionReadByInode<'version> {
+    version: &'version TransactionOutput,
+    index_built_at_the_first_lookup:
+        Option<FirstEntryOfEachIdentity<'version, InodeNumber, FileOfAnInodeOtherThanTheFirst>>,
+}
+
+impl<'version> FilesOtherThanTheFirstOfAVersionReadByInode<'version> {
+    /// 还什么都没建。
+    pub(crate) fn of_the_version(version: &'version TransactionOutput) -> Self {
+        Self {
+            version,
+            index_built_at_the_first_lookup: None,
+        }
+    }
+
+    /// 同 [`TransactionOutput::file_other_than_the_first`]`(inode)`。
+    pub(crate) fn file_other_than_the_first(
+        &mut self,
+        inode: InodeNumber,
+    ) -> Option<&'version FileOfAnInodeOtherThanTheFirst> {
+        let _inside_the_file_lookups =
+            lookup_work::entering(LookupSite::FilesOfTheReplacedRolesInThePreviousVersion);
+        let version = self.version;
+        self.index_built_at_the_first_lookup
+            .get_or_insert_with(|| version.files_other_than_the_first_by_inode())
+            .first_entry_of(&inode)
+    }
+}
+
 /// 一版里进映射的角色按角色取那条映射条目的批量形态（七·6，用户 2026-10-21 定并进）：每个角色的答案与「在 `mapped_units` 上按角色
 /// `iter().find` 取最先那一把 key、再 [`central_mapping_locations_in_the_version`]」逐个相同，但 key 走 `mapped_units` 按角色的索引、
 /// 落点走批量读者（每片叶只解一次）。释放判定路径、核位置项、释放前读盘核校验和、快照判定那一圈进 deadlist 的那一支各对一串角色逐个查，
@@ -4706,6 +4774,15 @@ impl<'version> CentralMappingOfAVersionReadLeafByLeaf<'version> {
         }
     }
 
+    /// 同 [`TransactionOutput::unit`]：这一版里角色是 `identity` 的那个单元，按角色从这份读者建好的单元索引取（整理发批取计划里单元的字节，
+    /// 七·6 代码轮第三轮判决第三节第 1 条），不另建索引、不在 `units` 上线性找。
+    ///
+    /// # Panics
+    /// 同 [`TransactionOutput::unit`]。
+    pub(crate) fn unit_of_role(&self, identity: TransactionUnit) -> &'version PublishedUnit {
+        self.units_by_role.unit(identity)
+    }
+
     /// 同 [`central_mapping_locations_in_the_version`]`(version, key)`。
     ///
     /// # Panics
@@ -4737,6 +4814,7 @@ fn mapped_roles_released_by_this_publish(
     kept_for_the_snapshots: &[TransactionUnit],
     file_content: FileContentOfThePublish,
 ) -> Vec<TransactionUnit> {
+    let mut previous_files = FilesOtherThanTheFirstOfAVersionReadByInode::of_the_version(previous);
     roles_replaced_via_mapping(previous, roles, file_content)
         .into_iter()
         .filter(|identity| !kept_for_the_snapshots.contains(identity))
@@ -4772,7 +4850,9 @@ fn mapped_roles_released_by_this_publish(
             | TransactionUnit::InstanceTable
             | TransactionUnit::InstanceTablePageAfterTheFirst(_) => false,
         })
-        .filter(|identity| has_a_placement_in_the_previous_version(previous, *identity))
+        .filter(|identity| {
+            has_a_placement_in_the_previous_version(previous, &mut previous_files, *identity)
+        })
         .collect()
 }
 
@@ -4813,7 +4893,8 @@ pub fn refuse_mapping_entries_that_do_not_name_two_pool_devices(
     pool_devices: &[DeviceIdentity],
     file_content: FileContentOfThePublish,
 ) -> Result<(), PublishError> {
-    let _inside_the_releases = lookup_work::entering(LookupSite::MappingEntriesOfReleasedUnits);
+    let _inside_the_releases =
+        lookup_work::entering(LookupSite::MappingEntriesCheckedForTwoPoolDevices);
     // 同释放判定路径：key 与落点都走建一次的索引（七·6）。
     let mut previous_read_by_role = MappedUnitsOfAVersionReadByRole::of_the_version(previous);
     for identity in
@@ -4888,7 +4969,8 @@ pub fn copies_failing_the_release_checksum_check<Reader: PoolReader + ?Sized>(
         &pool_devices,
         file_content,
     )?;
-    let _inside_the_releases = lookup_work::entering(LookupSite::MappingEntriesOfReleasedUnits);
+    let _inside_the_releases =
+        lookup_work::entering(LookupSite::MappingEntriesCheckedAgainstTheReleaseChecksums);
     let mut quarantined = Vec::new();
     // 同释放判定路径：key 与落点都走建一次的索引（七·6）。
     let mut previous_read_by_role = MappedUnitsOfAVersionReadByRole::of_the_version(previous);
@@ -6034,11 +6116,19 @@ fn deadlist_changes_of_this_publish(
         return Ok(DeadlistChangesOfThisPublish::default());
     };
     let _inside_the_snapshot_judgement = lookup_work::entering(LookupSite::SnapshotJudgement);
+    let mut previous_files =
+        FilesOtherThanTheFirstOfAVersionReadByInode::of_the_version(previous_version);
     let user_visible_roles_replaced: Vec<TransactionUnit> =
         roles_replaced_via_mapping(previous_version, candidate_roles, file_content)
             .into_iter()
             .filter(|identity| asks_the_snapshot_lifetime_judgement(*identity))
-            .filter(|identity| has_a_placement_in_the_previous_version(previous_version, *identity))
+            .filter(|identity| {
+                has_a_placement_in_the_previous_version(
+                    previous_version,
+                    &mut previous_files,
+                    *identity,
+                )
+            })
             .collect();
     let previous_snapshot_txg_of_the_live_head =
         previous_version.previous_snapshot_txg_of_the_live_head();
@@ -6109,11 +6199,15 @@ fn deadlist_changes_of_this_publish(
         | SnapshotStepOfThePublish::DestructionIntent { .. } => Vec::new(),
     };
     let mut entries_released_by_the_destruction = Vec::with_capacity(entries_released.len());
+    // 放掉的每条条目按映射 key 取它留存的映射条目：走建一次的索引（七·6 代码轮第三轮判决第三节第 1 条），同一把 key 取最先那一项，
+    // 与在 `mapping_entries_kept_for_the_snapshots` 上 `iter().find` 同一项；一条都不放的不建。
+    let mut kept_mapping_entries_by_key = None;
     for entry in entries_released {
-        let locations = previous_version
-            .mapping_entries_kept_for_the_snapshots
-            .iter()
-            .find(|(key, _)| *key == entry.mapping_key)
+        let _inside_the_destruction =
+            lookup_work::entering(LookupSite::KeptMappingEntriesReleasedByTheSnapshotDestruction);
+        let locations = kept_mapping_entries_by_key
+            .get_or_insert_with(|| previous_version.mapping_entries_kept_for_the_snapshots_by_key())
+            .first_entry_of(&entry.mapping_key.as_slice())
             .map(|(_, locations)| *locations)
             .ok_or_else(|| PublishError::DeadlistEntryNotReleasable {
                 mapping_key: entry.mapping_key.clone(),
@@ -9694,7 +9788,6 @@ pub(crate) fn publish_compaction_step_in_the_session<Device: BlockDevice>(
         for planned_relocation in planned {
             relocations.push(relocation_checked_against_the_version(
                 &*pool.devices,
-                previous,
                 previous_read_by_key.get_or_insert_with(|| {
                     MappedUnitsOfAVersionReadByKey::of_the_version(previous)
                 }),
@@ -9864,30 +9957,51 @@ fn compaction_step_check_on_the_version_after_this_publish(
 
 /// 整理发批核计划时按映射 key 查上一版的两样（[`relocation_checked_against_the_version`]）：`mapped_units` 按 key 的索引
 /// （同一把 key 取最先那一项，同在表上按 key `iter().find`），与中央映射的批量读者（答案同逐 key 查 [`central_mapping_locations_in_the_version`]）。
+/// 计划里的单元是这一版的角色时，它的字节按角色从批量读者手里那份单元索引取（不另建一份）；不是这一版进映射的单元时，按 key 在留存的映射条目里找，
+/// 那个索引在第一次要找时才建（七·6 代码轮第三轮判决第三节第 1 条，同一把 key 取最先那一项，同在表上 `iter().find`）。
 struct MappedUnitsOfAVersionReadByKey<'version> {
+    version: &'version TransactionOutput,
     mapped_units_by_key:
         FirstEntryOfEachIdentity<'version, &'version [u8], (TransactionUnit, Vec<u8>)>,
     central_mapping: CentralMappingOfAVersionReadLeafByLeaf<'version>,
+    kept_mapping_entries_by_key_built_at_the_first_lookup:
+        Option<KeptMappingEntriesOfAVersionByKey<'version>>,
 }
 
+/// 一版留存的映射条目按映射 key 的索引（[`TransactionOutput::mapping_entries_kept_for_the_snapshots_by_key`]）。
+type KeptMappingEntriesOfAVersionByKey<'version> =
+    FirstEntryOfEachIdentity<'version, &'version [u8], (Vec<u8>, [LocationEntry; 2])>;
+
 impl<'version> MappedUnitsOfAVersionReadByKey<'version> {
     fn of_the_version(version: &'version TransactionOutput) -> Self {
         Self {
+            version,
             mapped_units_by_key: version.mapped_units_by_key(),
             central_mapping: CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version),
+            kept_mapping_entries_by_key_built_at_the_first_lookup: None,
         }
     }
+
+    /// 这一版留存的映射条目里 key 是 `mapping_key` 的最先那一项（同在 `mapping_entries_kept_for_the_snapshots` 上按 key `iter().find`）。
+    fn kept_mapping_entry_of_key(
+        &mut self,
+        mapping_key: &[u8],
+    ) -> Option<&'version (Vec<u8>, [LocationEntry; 2])> {
+        let version = self.version;
+        self.kept_mapping_entries_by_key_built_at_the_first_lookup
+            .get_or_insert_with(|| version.mapping_entries_kept_for_the_snapshots_by_key())
+            .first_entry_of(&mapping_key)
+    }
 }
 
 /// 计划里的一项对着上一版核过、带上搬它要的东西（[`publish_relocation_batch`]）：它是这一版进映射的单元（取它的角色与内存里的字节），
 /// 或是留存的映射条目（字节从盘上按映射位置读、核位置项里的校验和，两份取先核得上的那一份）；源落点要就是映射位置、跨度合它的类。
-/// 按 key 找角色与取映射条目走 `previous_read_by_key`（它就是 `previous` 的索引与批量读者）。
+/// 按 key 找角色、取映射条目、找留存的映射条目、取角色的字节都走 `previous_read_by_key`（它就是上一版的索引与批量读者）。
 ///
 /// # Errors
 /// [`RelocationBatchRefused`] 的四种计划错。
 fn relocation_checked_against_the_version<Reader: PoolReader + ?Sized>(
     reader: &Reader,
-    previous: &TransactionOutput,
     previous_read_by_key: &mut MappedUnitsOfAVersionReadByKey<'_>,
     planned: &PlannedRelocation,
 ) -> Result<RelocationOfThePublish, RelocationBatchRefused> {
@@ -9911,10 +10025,8 @@ fn relocation_checked_against_the_version<Reader: PoolReader + ?Sized>(
                 locations,
             ))
         }
-        None => previous
-            .mapping_entries_kept_for_the_snapshots
-            .iter()
-            .find(|(key, _)| *key == mapping_key)
+        None => previous_read_by_key
+            .kept_mapping_entry_of_key(&mapping_key)
             .and_then(|(_, locations)| {
                 user_visible_role_standing_for_a_unit_class(
                     mapping_key.first().copied().unwrap_or(0),
@@ -9983,7 +10095,11 @@ fn relocation_checked_against_the_version<Reader: PoolReader + ?Sized>(
         );
     };
     let bytes = match holder {
-        RelocatedUnitHolder::RoleOfThisVersion(role) => previous.unit(role).bytes.clone(),
+        RelocatedUnitHolder::RoleOfThisVersion(role) => previous_read_by_key
+            .central_mapping
+            .unit_of_role(role)
+            .bytes
+            .clone(),
         RelocatedUnitHolder::KeptForTheSnapshots => {
             let unit_bytes = usize::try_from(planned.source.span * SLOT_BYTES)
                 .expect("一个单元两槽以内，字节数装得进 usize");
@@ -11798,12 +11914,18 @@ struct BuiltExtentTree {
 /// （角色清单按同一份计划排、上一版的字节是这个进程装的或重建时核过的、两份清单由 `publish_admitted` 按同一份计划拼）。
 #[allow(
     clippy::too_many_arguments,
-    reason = "参数各是一样东西：计划、第一个文件之外各文件的计划、上一版、第一个文件的数据指针、它尾部有没有洞、别的文件的数据指针、树号、身份、落点、出生序号"
+    reason = "参数各是一样东西：计划、第一个文件之外各文件的计划、上一版、上一版按身份的两份索引、第一个文件的数据指针、它尾部有没有洞、别的文件的数据指针、树号、身份、落点、出生序号"
 )]
 fn build_extent_tree(
     plan: &ExtentTreePlan,
     files_other_than_the_first: &FilesOtherThanTheFirstPlan,
     previous: Option<&TransactionOutput>,
+    units_of_the_previous_version: Option<
+        &FirstEntryOfEachIdentity<'_, TransactionUnit, PublishedUnit>,
+    >,
+    files_of_the_previous_version: Option<
+        &FirstEntryOfEachIdentity<'_, InodeNumber, FileOfAnInodeOtherThanTheFirst>,
+    >,
     data_pointers: &[DataPointer],
     tail_of_the_first_file: TailOfAFile,
     data_pointers_of_the_files_other_than_the_first: &[(InodeNumber, Vec<DataPointer>)],
@@ -11850,8 +11972,10 @@ fn build_extent_tree(
     let carried_node = |role_in_the_previous_version: TransactionUnit,
                         role: TransactionUnit,
                         units_so_far: &mut Vec<PublishedUnit>| {
-        let previous_version = previous.expect("照抄的 extent 树节点只出现在接着上一版的发布里");
-        let carried = previous_version.unit(role_in_the_previous_version);
+        let _inside_the_carried_nodes = lookup_work::entering(LookupSite::CarriedExtentTreeNodes);
+        let carried = units_of_the_previous_version
+            .expect("照抄的 extent 树节点只出现在接着上一版的发布里")
+            .unit(role_in_the_previous_version);
         units_so_far.push(PublishedUnit {
             slot: carried.slot,
             identity: role,
@@ -11980,10 +12104,10 @@ fn build_extent_tree(
                 }
                 ExtentTreeNodeOrigin::CarriedFromThePreviousVersion => {
                     carried_node(role, role, &mut units);
-                    previous
-                        .and_then(|previous_version| {
-                            previous_version.file_other_than_the_first(*inode)
-                        })
+                    let _inside_the_carried_files =
+                        lookup_work::entering(LookupSite::FilesOfTheCarriedExtentLowerNodes);
+                    files_of_the_previous_version
+                        .and_then(|previous_files| previous_files.first_entry_of(inode))
                         .and_then(|previous_file| {
                             previous_file
                                 .extent_lower_nodes
@@ -13167,6 +13291,8 @@ fn publish_admitted<Device: BlockDevice>(
     // （整理的一批之后上一版的拷贝里它们已换成映射位置）。照抄的文件约有上一版那么多个，按 inode 号查走上一版的索引（七·6）。
     let files_of_the_previous_version =
         previous.map(TransactionOutput::files_other_than_the_first_by_inode);
+    // 上一版 `units` 按角色的索引一次发布建一次（七·6）：照抄 extent 树与多层码 2 树的节点、照抄全部单元那一圈都按角色查它。
+    let units_of_the_previous_version = previous.map(TransactionOutput::units_by_role);
     let data_pointers_of_the_files_other_than_the_first: Vec<(InodeNumber, Vec<DataPointer>)> =
         resolved
             .files_other_than_the_first
@@ -13206,6 +13332,8 @@ fn publish_admitted<Device: BlockDevice>(
         &resolved.extent_tree,
         &resolved.files_other_than_the_first,
         previous,
+        units_of_the_previous_version.as_ref(),
+        files_of_the_previous_version.as_ref(),
         &data_pointers,
         record_of_the_first_file_after_this_publish.map_or(
             TailOfAFile::NoHoleAfterTheLastDataUnit,
@@ -13402,6 +13530,7 @@ fn publish_admitted<Device: BlockDevice>(
         trees.accounting,
         &resolved.accounting_tree,
         previous,
+        units_of_the_previous_version.as_ref(),
         &accounting_entries
             .iter()
             .map(|entry| {
@@ -13423,6 +13552,7 @@ fn publish_admitted<Device: BlockDevice>(
         trees.deadlist,
         &resolved.deadlist_tree,
         previous,
+        units_of_the_previous_version.as_ref(),
         &resolved
             .deadlist
             .entries_after_this_publish
@@ -13716,6 +13846,7 @@ fn publish_admitted<Device: BlockDevice>(
         trees.central_mapping,
         &resolved.central_mapping_tree,
         previous,
+        units_of_the_previous_version.as_ref(),
         &mapping_entries_by_key,
         &multi_level_tree_context,
         &slots,
@@ -14046,7 +14177,6 @@ fn publish_admitted<Device: BlockDevice>(
         identity,
         bytes,
     };
-    let units_of_the_previous_version = previous.map(TransactionOutput::units_by_role);
     let mut units: Vec<PublishedUnit> = Vec::new();
     for (position, data_unit) in data_units.iter().enumerate() {
         let identity = TransactionUnit::Data(DataUnitIndexInFile(
@@ -14745,13 +14875,16 @@ impl BuiltMultiLevelTree {
 /// 同一个进程里接着的上一版是发布路径自己装的，key 取自 `BTreeMap`。
 #[allow(
     clippy::too_many_arguments,
-    reason = "装一棵树要的八样：哪一棵、它的号、计划、上一版、条目、身份字段、落点、发号器，各自独立"
+    reason = "装一棵树要的九样：哪一棵、它的号、计划、上一版、上一版按角色的单元索引、条目、身份字段、落点、发号器，各自独立"
 )]
 fn build_multi_level_tree(
     tree: MultiLevelCodeTwoTree,
     tree_identifier: TreeIdentifier,
     plan: &CodeTwoTreePlan,
     previous: Option<&TransactionOutput>,
+    units_of_the_previous_version: Option<
+        &FirstEntryOfEachIdentity<'_, TransactionUnit, PublishedUnit>,
+    >,
     leaf_entry_bytes_by_key: &BTreeMap<CodeTwoTreeKey, Vec<u8>>,
     context: &MultiLevelTreeBuildContext<'_>,
     slots: &BTreeMap<TransactionUnit, SlotNumber>,
@@ -14782,7 +14915,12 @@ fn build_multi_level_tree(
             CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => {
                 let previous_version = previous.expect("照抄的节点只出现在接着上一版的发布里");
                 let previous_tree = previous_version.multi_level_tree(tree);
-                let carried = previous_version
+                let _inside_the_carried_nodes =
+                    lookup_work::entering(LookupSite::CarriedNodesOfTheMultiLevelTrees);
+                let carried = units_of_the_previous_version
+                    .expect(
+                        "照抄的节点只出现在接着上一版的发布里：上一版的单元索引与上一版一起交进来",
+                    )
                     .unit(tree.role_of_node(*previous_position, &previous_tree.shape));
                 (
                     previous_tree.pointer_of(*previous_position),
@@ -15073,6 +15211,8 @@ mod tests {
                         previous_roles_without_a_successor: Vec::new(),
                     },
                     None,
+                    None,
+                    None,
                     &units.data_pointers,
                     TailOfAFile::NoHoleAfterTheLastDataUnit,
                     &[],
diff --git a/crates/singlefs-harness/tests/publish_lookup_by_identity.rs b/crates/singlefs-harness/tests/publish_lookup_by_identity.rs
index 8d9ea0c..fe205d5 100644
--- a/crates/singlefs-harness/tests/publish_lookup_by_identity.rs
+++ b/crates/singlefs-harness/tests/publish_lookup_by_identity.rs
@@ -23,14 +23,16 @@ use std::collections::BTreeSet;
 
 use singlefs_checker::image::InvariantVerdict;
 use singlefs_checker::walk::check_pool_image;
-use singlefs_core::address::{DataUnitIndexInFile, DeviceIdentity, InodeNumber, SlotNumber};
+use singlefs_core::address::{
+    DataUnitIndexInFile, DeviceIdentity, InodeNumber, SlotNumber, TreeIdentifier,
+};
 use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
 use singlefs_core::code_two_tree::{
     leaf_position_routed_to, CodeTwoKeyFieldWidths, CodeTwoTreeKey, CodeTwoTreeNodePosition,
 };
 use singlefs_core::compaction_plan::{
-    CompactionBatchSize, CompactionBatchUnits, CompactionSwitches,
+    CompactionBatchSize, CompactionBatchUnits, CompactionSwitches, PlannedRelocation,
 };
 use singlefs_core::make_filesystem::{
     make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
@@ -43,18 +45,24 @@ use singlefs_core::mounted_session::{
     central_mapping_as_the_compaction_reads_it, CompactionRun, CompactionRunOutcome,
     MountedSession, UserChange,
 };
-use singlefs_core::records::parse_mapping_entry;
+use singlefs_core::records::{parse_mapping_entry, TREE_KIND_SNAPSHOT_INODE_ROOT};
 use singlefs_core::region_scan::CentralMappingEntry;
 use singlefs_core::transaction::lookup_work::{
     lookup_work_while, LookupSite, LookupWork, LookupWorkOfEachSite,
 };
 use singlefs_core::transaction::{
-    acquire_instance, central_mapping_locations_in_the_version, publish_first_file, warm_up,
-    CentralMappingOfAVersionReadLeafByLeaf, FirstEntryOfEachIdentity, FirstFile,
-    MappedUnitsOfAVersionReadByRole, MappingLookup, PoolVersion, PoolWriter, TransactionOutput,
-    TransactionUnit,
+    acquire_instance, asks_the_snapshot_lifetime_judgement,
+    central_mapping_locations_in_the_version, copies_failing_the_release_checksum_check,
+    placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots, publish_first_file,
+    publish_relocation_batch, publish_sequential_write_of_an_inode,
+    refuse_mapping_entries_that_do_not_name_two_pool_devices, warm_up,
+    CentralMappingOfAVersionReadLeafByLeaf, FileContentOfThePublish, FirstEntryOfEachIdentity,
+    FirstFile, MappedUnitsOfAVersionReadByRole, MappingLookup, PoolVersion, PoolWriter,
+    RelocatedUnitHolder, TransactionOutput, TransactionUnit,
+};
+use singlefs_core::unit::{
+    build_index_node, data_unit_payload_capacity, parse_index_node, UNIT_CLASS_DATA,
 };
-use singlefs_core::unit::{build_index_node, data_unit_payload_capacity, parse_index_node};
 use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
 
 type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;
@@ -347,12 +355,16 @@ fn build_a_pool_of_files(
             },
             on_every_version,
         );
-        // 整份重写换下这个文件的每个数据单元：快照判定那一圈逐个取映射 key，释放的三处逐个取映射条目，都经索引。
+        // 整份重写换下这个文件的每个数据单元：快照判定那一圈逐个取映射 key，释放的三处各自逐个取映射条目（三处分开记、各判各的：
+        // 一处改回线性，另两处补不上它的数），每个换下的角色按 inode 号找上一版的文件，都经索引。
         let data_units_replaced =
             u64::try_from(shape.data_units_per_file).expect("一个文件的数据单元数装得进 u64");
         for site in [
             LookupSite::SnapshotJudgement,
-            LookupSite::MappingEntriesOfReleasedUnits,
+            LookupSite::MappingEntriesOfTheReleaseJudgement,
+            LookupSite::MappingEntriesCheckedForTwoPoolDevices,
+            LookupSite::MappingEntriesCheckedAgainstTheReleaseChecksums,
+            LookupSite::FilesOfTheReplacedRolesInThePreviousVersion,
         ] {
             let work = lookup_work.of_the_site(site);
             assert!(
@@ -1239,3 +1251,683 @@ fn compaction_batches_decode_each_routed_mapping_leaf_once_and_examine_index_ent
         "数据单元四倍，整理每一趟建索引的次数涨了 {growth_in_percent_of_the_index_builds}%，门槛 {HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE}%（每查一把 key 现建一个索引就按 key 数涨；小池 {smaller:?} 共 {smaller_calls} 次，大池 {larger:?} 共 {larger_calls} 次）"
     );
 }
+
+/// 第一个文件之外 `files` 个文件，各整份写 `data_units_per_file` 个数据单元（经会话一个个写，不抬 F）：交回盘、会话与文件的号。
+fn pool_with_files_written_once(files: u64, data_units_per_file: usize) -> PoolBuilt {
+    let (mut devices, mut session) = formatted_and_mounted(CompactionSwitches::PRODUCT);
+    let first_inode = version_of(&session).inode_number_watermark();
+    let write_time_of_the_new_inodes = common::FIXED_WRITE_TIME_SECONDS + 600;
+    session
+        .publish_user_change(
+            &mut devices,
+            UserChange::NewInodes {
+                count: files,
+                write_time_seconds: write_time_of_the_new_inodes,
+            },
+        )
+        .unwrap_or_else(|refusal| panic!("建 {files} 个 inode：{refusal:?}"));
+    let inodes: Vec<InodeNumber> = (first_inode..first_inode + files)
+        .map(InodeNumber)
+        .collect();
+    // 每写一个文件写入时间往后推一秒。
+    for (write_time_seconds, inode) in (write_time_of_the_new_inodes + 1..).zip(&inodes) {
+        let content = content_of_data_units(data_units_per_file, inode.0);
+        session
+            .publish_user_change(
+                &mut devices,
+                UserChange::SequentialWriteOfAnInode {
+                    inode: *inode,
+                    content: &content,
+                    write_time_seconds,
+                },
+            )
+            .unwrap_or_else(|refusal| panic!("写 inode {}：{refusal:?}", inode.0));
+    }
+    PoolBuilt {
+        devices,
+        session,
+        inodes,
+    }
+}
+
+/// 不经会话、直接在会话现行那一版上整份重写 `inode`（一次发布，`transaction::publish_sequential_write_of_an_inode`），会话接到这一版上：
+/// 交回上一版、这一版与这一次发布在产品里做的按身份查（每次查至多一次折半查找，这里就判）。一次调用恰好一次发布，下面的用例按这一版逐处算该查几次。
+fn whole_rewrite_published_once(
+    pool: &mut PoolBuilt,
+    inode: InodeNumber,
+    seed: u64,
+    data_units: usize,
+    write_time_seconds: u64,
+) -> (TransactionOutput, TransactionOutput, LookupWorkOfEachSite) {
+    let parameters = common::parameters();
+    let previous = version_of(&pool.session).clone();
+    let content = content_of_data_units(data_units, seed);
+    let session = &mut pool.session;
+    let devices = &mut pool.devices;
+    let (published, lookup_work) = lookup_work_while(|| {
+        let mut writer = PoolWriter::new(&parameters, devices);
+        publish_sequential_write_of_an_inode(
+            &mut writer,
+            &mut session.allocator,
+            &previous,
+            inode,
+            &content,
+            write_time_seconds,
+            session.instance,
+        )
+    });
+    let published =
+        published.unwrap_or_else(|refusal| panic!("整份重写 inode {}：{refusal:?}", inode.0));
+    assert_every_lookup_examines_at_most_a_binary_search(
+        &lookup_work,
+        &format!("整份重写 inode {}", inode.0),
+    );
+    session.current = PoolVersion::of_a_version_of_trees(published.clone());
+    (previous, published, lookup_work)
+}
+
+/// `keys` 按 `version` 中央映射树的形状走到的叶有几片（去重）：批量读者查这几把 key 该解的就是这几片。
+fn distinct_mapping_leaves_routed_to_by<'key>(
+    version: &TransactionOutput,
+    keys: impl Iterator<Item = &'key [u8]>,
+) -> u64 {
+    let leaves: BTreeSet<CodeTwoTreeNodePosition> = keys
+        .filter_map(|key| {
+            leaf_position_routed_to(
+                &version.central_mapping_tree.shape,
+                &CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING),
+            )
+        })
+        .collect();
+    u64::try_from(leaves.len()).expect("叶的片数")
+}
+
+/// 这一版照抄（没重写）的单元里角色合 `is_of_the_kind` 的有几个。
+fn carried_units_of_the_kind(
+    published: &TransactionOutput,
+    is_of_the_kind: impl Fn(TransactionUnit) -> bool,
+) -> u64 {
+    let carried = published
+        .units
+        .iter()
+        .filter(|unit| {
+            is_of_the_kind(unit.identity) && !published.rewritten.contains(&unit.identity)
+        })
+        .count();
+    u64::try_from(carried).expect("单元数")
+}
+
+/// extent 树的节点（`build_extent_tree` 照抄的那几种）。
+fn is_an_extent_tree_node(identity: TransactionUnit) -> bool {
+    matches!(
+        identity,
+        TransactionUnit::ExtentLowerNode(_)
+            | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
+            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
+            | TransactionUnit::ExtentRoot
+    )
+}
+
+/// 第一个文件之外一个文件的下段节点。
+fn is_a_lower_node_of_a_file_other_than_the_first(identity: TransactionUnit) -> bool {
+    matches!(
+        identity,
+        TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
+    )
+}
+
+/// 记账树、deadlist 树、中央映射树的节点（`build_multi_level_tree` 照抄的那几种）。
+fn is_a_node_of_a_multi_level_tree(identity: TransactionUnit) -> bool {
+    matches!(
+        identity,
+        TransactionUnit::AccountingTree
+            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
+            | TransactionUnit::DeadlistTree
+            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
+            | TransactionUnit::MappingTree
+            | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
+    )
+}
+
+/// 一处做的查找（建索引次数、解叶次数、查的次数）恰好是那一处走一遍该做的 `one_pass` 的正整数倍：发布算到定点时同一处可以走几遍
+/// （每一遍对同一个上一版、同一串角色做一样多的事），遍数从建索引次数（建索引不为 0 时）或查的次数现取。交回遍数。
+fn assert_the_work_is_whole_passes_of(
+    work: LookupWork,
+    one_pass: (u64, u64, u64),
+    what: &str,
+) -> u64 {
+    let (builds_of_one_pass, decodes_of_one_pass, lookups_of_one_pass) = one_pass;
+    assert!(
+        lookups_of_one_pass >= 1,
+        "{what}：走一遍至少查一次，这一格才判得出（走一遍 {one_pass:?}）"
+    );
+    // 走一遍要建索引的，遍数按建索引次数算；走一遍不建索引的，按查的次数算（上面判过走一遍至少查一次）。
+    let passes = work
+        .index_builds
+        .checked_div(builds_of_one_pass)
+        .unwrap_or(work.index_lookups / lookups_of_one_pass);
+    assert!(
+        passes >= 1
+            && (
+                work.index_builds,
+                work.mapping_leaf_decodes,
+                work.index_lookups
+            ) == (
+                passes * builds_of_one_pass,
+                passes * decodes_of_one_pass,
+                passes * lookups_of_one_pass
+            ),
+        "{what}：（建索引次数、解叶次数、查的次数）是走一遍 {one_pass:?} 的整数倍（{passes} 遍）：{work:?}"
+    );
+    passes
+}
+
+/// 第一个文件之外 48 个八单元的文件（中央映射树长到不止一片叶）上整份重写其中一个（七·6 代码轮第三轮判决第三节第 1、2 条）。
+/// 重写之前，拿那个文件的数据单元与下段节点各调一次释放的三处（`transaction` 的三个公开入口，只读）：每一处各自建两份索引（按角色的 key 索引、
+/// 批量读者的单元索引）、那几把 key 走到的叶各解一次、每个角色查一次 key、每片叶查一次单元；释放判定路径那一处每个角色按 inode 号经索引找一次上一版的文件。
+/// 再整份重写它（一次发布）：快照判定那一圈（池里没有快照）每建一份索引至少查换下的数据单元那么多次；照抄的 extent 树节点、第一个文件之外文件的
+/// 下段节点、多层码 2 树的节点各按角色（按 inode 号）经上一版建一次的索引查一次（发布算到定点时同一处可以走几遍，判整数遍）。
+/// 改回线性的那一处查的次数掉到 0，每查一个角色现建索引的那一处建的次数按角色数涨，落点改回逐 key 查的那一处解叶次数按 key 数涨。
+/// 判别力：`crates/mutations.tsv` 里以「七·6 第三轮」起头、点名这一条的几行（攻方 RB、RD、RE、RF，正推 M3、M4，释放与照抄那几处改回线性）。
+#[test]
+fn a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site(
+) {
+    let mut pool = pool_with_files_written_once(48, 8);
+    let rewritten_inode = pool.inodes[7];
+    let before_the_rewrite = version_of(&pool.session).clone();
+    assert!(
+        before_the_rewrite.central_mapping_tree.shape.nodes().len() > 1,
+        "中央映射树不止一个节点：照抄多层码 2 树的节点走得到"
+    );
+    let file = before_the_rewrite
+        .file_other_than_the_first(rewritten_inode)
+        .expect("写过的文件在这一版里");
+    let roles: Vec<TransactionUnit> = file
+        .data_roles()
+        .into_iter()
+        .chain(file.extent_lower_node_roles())
+        .collect();
+    let replaced_roles = u64::try_from(roles.len()).expect("角色数");
+    let routed_leaves = distinct_mapping_leaves_routed_to_by(
+        &before_the_rewrite,
+        before_the_rewrite
+            .mapped_units
+            .iter()
+            .filter(|(role, _)| roles.contains(role))
+            .map(|(_, key)| key.as_slice()),
+    );
+    assert!(
+        replaced_roles >= 9,
+        "八个数据单元连同下段节点：{replaced_roles}"
+    );
+    let one_pass_of_a_release_site = (2, routed_leaves, replaced_roles + routed_leaves);
+    let (_, release_judgement) = lookup_work_while(|| {
+        placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots(
+            &before_the_rewrite,
+            &pool.session.allocator,
+            FileContentOfThePublish::NotRewrittenWhole,
+            &roles,
+            &[],
+        )
+        .expect("这个文件的单元在上一版里都分配着、映射里查得到")
+    });
+    let pool_devices: Vec<DeviceIdentity> =
+        pool.devices.iter().map(|(identity, _)| *identity).collect();
+    let (_, two_pool_devices_check) = lookup_work_while(|| {
+        refuse_mapping_entries_that_do_not_name_two_pool_devices(
+            &before_the_rewrite,
+            &roles,
+            &[],
+            &pool_devices,
+            FileContentOfThePublish::NotRewrittenWhole,
+        )
+        .expect("每条映射条目的两条位置项各指池里一块不同的盘")
+    });
+    let (_, release_checksum_check) = lookup_work_while(|| {
+        copies_failing_the_release_checksum_check(
+            &before_the_rewrite,
+            &roles,
+            &[],
+            pool.devices.as_slice(),
+            FileContentOfThePublish::NotRewrittenWhole,
+        )
+        .expect("位置项都指池里的盘")
+    });
+    println!(
+        "publish_lookup_by_identity release_sites replaced_roles={replaced_roles} routed_leaves={routed_leaves} release_judgement={release_judgement:?} two_pool_devices_check={two_pool_devices_check:?} release_checksum_check={release_checksum_check:?}"
+    );
+    for (site, lookup_work) in [
+        (
+            LookupSite::MappingEntriesOfTheReleaseJudgement,
+            &release_judgement,
+        ),
+        (
+            LookupSite::MappingEntriesCheckedForTwoPoolDevices,
+            &two_pool_devices_check,
+        ),
+        (
+            LookupSite::MappingEntriesCheckedAgainstTheReleaseChecksums,
+            &release_checksum_check,
+        ),
+    ] {
+        let work = lookup_work.of_the_site(site);
+        assert_eq!(
+            (work.index_builds, work.mapping_leaf_decodes, work.index_lookups),
+            one_pass_of_a_release_site,
+            "{site:?}：建两份索引（按角色的 key 索引、批量读者的单元索引）、{replaced_roles} 个角色的 key 走到的 {routed_leaves} 片叶各解一次、\
+             每个角色查一次 key、每片叶查一次单元（建索引次数、解叶次数、查的次数）：{work:?}"
+        );
+    }
+    let files_of_the_replaced_roles =
+        release_judgement.of_the_site(LookupSite::FilesOfTheReplacedRolesInThePreviousVersion);
+    assert_eq!(
+        (
+            files_of_the_replaced_roles.index_builds,
+            files_of_the_replaced_roles.index_lookups
+        ),
+        (1, replaced_roles),
+        "释放判定路径：{replaced_roles} 个角色在上一版里有没有落点，各按 inode 号经一份建一次的索引找一次文件：{files_of_the_replaced_roles:?}"
+    );
+
+    let (_, published, lookup_work) = whole_rewrite_published_once(
+        &mut pool,
+        rewritten_inode,
+        rewritten_inode.0 + 1_000_003,
+        8,
+        common::FIXED_WRITE_TIME_SECONDS + 5_000,
+    );
+    println!("publish_lookup_by_identity whole_rewrite lookup_work={lookup_work:?}");
+    let snapshot_judgement = lookup_work.of_the_site(LookupSite::SnapshotJudgement);
+    assert!(
+        snapshot_judgement.index_builds >= 1
+            && snapshot_judgement.index_lookups >= 8 * snapshot_judgement.index_builds,
+        "快照判定那一圈（池里没有快照）：每走一遍建一份按角色的索引，换下的用户可见角色（至少八个数据单元）各经它查一次映射 key：{snapshot_judgement:?}"
+    );
+    for (site, carried, which) in [
+        (
+            LookupSite::CarriedExtentTreeNodes,
+            carried_units_of_the_kind(&published, is_an_extent_tree_node),
+            "照抄的 extent 树节点",
+        ),
+        (
+            LookupSite::FilesOfTheCarriedExtentLowerNodes,
+            carried_units_of_the_kind(&published, is_a_lower_node_of_a_file_other_than_the_first),
+            "照抄的第一个文件之外文件的下段节点",
+        ),
+        (
+            LookupSite::CarriedNodesOfTheMultiLevelTrees,
+            carried_units_of_the_kind(&published, is_a_node_of_a_multi_level_tree),
+            "照抄的记账树、deadlist 树、中央映射树节点",
+        ),
+    ] {
+        assert_the_work_is_whole_passes_of(
+            lookup_work.of_the_site(site),
+            (0, 0, carried),
+            &format!(
+                "{site:?}：这一版{which}有 {carried} 个，走一遍每个经上一版建一次的索引查一次（这一处不另建索引、不解叶）"
+            ),
+        );
+    }
+}
+
+/// 池里有一个快照的样子：快照之后整份重写过的文件的旧单元进了 deadlist、映射条目留存。
+struct PoolWithASnapshot {
+    pool: PoolBuilt,
+    snapshot: TreeIdentifier,
+}
+
+/// 第一个文件之外 `files` 个四单元的文件，建一个快照（经会话）。
+fn pool_with_a_snapshot_of_files(files: u64) -> PoolWithASnapshot {
+    let mut pool = pool_with_files_written_once(files, 4);
+    let created = pool
+        .session
+        .publish_user_change(&mut pool.devices, UserChange::CreateSnapshot)
+        .unwrap_or_else(|refusal| panic!("建快照：{refusal:?}"));
+    drop(created);
+    let snapshot = version_of(&pool.session)
+        .tree_table_entries
+        .iter()
+        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
+        .expect("建了一个快照")
+        .tree;
+    PoolWithASnapshot { pool, snapshot }
+}
+
+/// 一个快照下整份重写一个文件（一次发布），再删这个快照（经会话）：
+/// 重写那一次，快照判定那一圈每走一遍换下的用户可见角色各查一次映射 key，进 deadlist 的各经批量读者查一次映射条目（建三份索引：按角色的 key 索引两份、
+/// 批量读者的单元索引一份；进 deadlist 的 key 走到的叶各解一次）；删快照那几批，放掉的每条条目按 key 经索引取一次它留存的映射条目。
+/// 判别力：`crates/mutations.tsv` 里正推 M2（进 deadlist 那一支退回线性找 key、逐 key 查）与「七·6 第三轮 删快照：」起头的那一行。
+#[test]
+fn a_whole_rewrite_under_a_snapshot_and_its_destruction_look_up_kept_mapping_entries_through_indexes_built_once(
+) {
+    let PoolWithASnapshot { mut pool, snapshot } = pool_with_a_snapshot_of_files(8);
+    let rewritten_inode = pool.inodes[2];
+    let (previous, published, lookup_work) = whole_rewrite_published_once(
+        &mut pool,
+        rewritten_inode,
+        rewritten_inode.0 + 3_000_017,
+        4,
+        common::FIXED_WRITE_TIME_SECONDS + 7_000,
+    );
+    let killed_keys: Vec<&[u8]> = published
+        .deadlist_entries
+        .iter()
+        .filter(|entry| {
+            !previous
+                .deadlist_entries
+                .iter()
+                .any(|before| before.mapping_key == entry.mapping_key)
+        })
+        .map(|entry| entry.mapping_key.as_slice())
+        .collect();
+    let killed = u64::try_from(killed_keys.len()).expect("条数");
+    let routed_leaves =
+        distinct_mapping_leaves_routed_to_by(&previous, killed_keys.iter().copied());
+    let replaced_user_visible_roles = u64::try_from(
+        previous
+            .mapped_units
+            .iter()
+            .filter(|(role, _)| {
+                published.rewritten.contains(role) && asks_the_snapshot_lifetime_judgement(*role)
+            })
+            .count(),
+    )
+    .expect("角色数");
+    let snapshot_judgement = lookup_work.of_the_site(LookupSite::SnapshotJudgement);
+    println!(
+        "publish_lookup_by_identity rewrite_under_a_snapshot killed={killed} routed_leaves={routed_leaves} replaced_user_visible_roles={replaced_user_visible_roles} snapshot_judgement={snapshot_judgement:?}"
+    );
+    assert!(
+        killed >= 4,
+        "快照下整份重写：旧的四个数据单元都进了 deadlist（{killed} 条）"
+    );
+    assert_the_work_is_whole_passes_of(
+        snapshot_judgement,
+        (
+            3,
+            routed_leaves,
+            replaced_user_visible_roles + killed + routed_leaves,
+        ),
+        &format!(
+            "快照判定那一圈：走一遍建三份索引、进 deadlist 的 {killed} 把 key 走到的 {routed_leaves} 片叶各解一次；换下的 {replaced_user_visible_roles} 个用户可见角色各查一次 key，\
+             进 deadlist 的各再查一次 key、每片叶查一次单元"
+        ),
+    );
+
+    let kept_before = version_of(&pool.session)
+        .mapping_entries_kept_for_the_snapshots
+        .len();
+    let (destroyed, destruction_work) = lookup_work_while(|| {
+        pool.session
+            .publish_user_change(&mut pool.devices, UserChange::DestroySnapshot { snapshot })
+    });
+    destroyed.unwrap_or_else(|refusal| panic!("删快照：{refusal:?}"));
+    assert_every_lookup_examines_at_most_a_binary_search(&destruction_work, "删快照");
+    let kept_after = version_of(&pool.session)
+        .mapping_entries_kept_for_the_snapshots
+        .len();
+    let released = u64::try_from(kept_before - kept_after).expect("条数");
+    let kept_entries_released = destruction_work
+        .of_the_site(LookupSite::KeptMappingEntriesReleasedByTheSnapshotDestruction);
+    assert!(
+        released >= 4
+            && kept_entries_released.index_lookups >= released
+            && kept_entries_released.index_builds >= 1,
+        "删快照放掉 {released} 条留存的映射条目（之前 {kept_before} 条、之后 {kept_after} 条）：发布算到定点时每走一遍，放掉的每条按 key 经一份建一次的索引取一次：{kept_entries_released:?}"
+    );
+}
+
+/// `version` 里第一个文件之外 `inode` 那个文件前 `data_units` 个数据单元的搬迁计划项：映射 key、映射位置上的源落点。
+fn planned_relocations_of_the_data_units(
+    version: &TransactionOutput,
+    inode: InodeNumber,
+    data_units: u64,
+) -> Vec<PlannedRelocation> {
+    (0..data_units)
+        .map(|unit| {
+            let role = TransactionUnit::DataOfAFileOtherThanTheFirst {
+                inode,
+                unit: DataUnitIndexInFile(unit),
+            };
+            let (_, mapping_key) = version
+                .mapped_units
+                .iter()
+                .find(|(mapped, _)| *mapped == role)
+                .expect("写过的数据单元进映射");
+            let MappingLookup::Found(locations) =
+                central_mapping_locations_in_the_version(version, mapping_key)
+            else {
+                panic!("{role:?} 在这一版的映射里查得到");
+            };
+            PlannedRelocation {
+                mapping_key: mapping_key.clone(),
+                source: Placement {
+                    slot: locations[0].slot,
+                    span: role.span_slots(),
+                },
+            }
+        })
+        .collect()
+}
+
+/// `version` 里留存的、数据单元那一类的映射条目，从第 `skipped` 条起取 `count` 条的搬迁计划项。
+fn planned_relocations_of_kept_data_units(
+    version: &TransactionOutput,
+    skipped: usize,
+    count: usize,
+) -> Vec<PlannedRelocation> {
+    let planned: Vec<PlannedRelocation> = version
+        .mapping_entries_kept_for_the_snapshots
+        .iter()
+        .filter(|(key, _)| key.first() == Some(&UNIT_CLASS_DATA))
+        .skip(skipped)
+        .take(count)
+        .map(|(key, locations)| PlannedRelocation {
+            mapping_key: key.clone(),
+            source: Placement {
+                slot: locations[0].slot,
+                span: 2,
+            },
+        })
+        .collect();
+    assert_eq!(planned.len(), count, "留存的数据单元映射条目够 {count} 条");
+    planned
+}
+
+/// 发一批整理（`transaction::publish_relocation_batch`，搬 `planned` 那几项），会话接到这一版上：判核源落点那一处的计数——
+/// 建索引：按 key 的索引、批量读者的单元索引各一份，计划里有留存的映射条目时再建一份留存条目按 key 的索引，不随批大小涨；
+/// 解叶：进映射那几项的 key 走到的叶各解一次；查：每一项按 key 查一次，留存的那几项再按 key 在留存条目里查一次，进映射的那几项按角色取一次字节，
+/// 每片解的叶按角色取一次单元，合起来是项数的两倍加解叶次数。
+fn relocate_a_batch_and_check_the_relocation_sources(
+    pool: &mut PoolBuilt,
+    planned: &[PlannedRelocation],
+    kept_items: u64,
+) {
+    let parameters = common::parameters();
+    let previous = version_of(&pool.session).clone();
+    let items = u64::try_from(planned.len()).expect("项数");
+    let role_keys: Vec<&[u8]> = planned
+        .iter()
+        .map(|item| item.mapping_key.as_slice())
+        .filter(|key| {
+            previous
+                .mapped_units
+                .iter()
+                .any(|(_, mapped)| mapped.as_slice() == *key)
+        })
+        .collect();
+    let routed_leaves = distinct_mapping_leaves_routed_to_by(&previous, role_keys.iter().copied());
+    let session = &mut pool.session;
+    let devices = &mut pool.devices;
+    let (relocated, lookup_work) = lookup_work_while(|| {
+        let mut writer = PoolWriter::new(&parameters, devices);
+        publish_relocation_batch(
+            &mut writer,
+            &mut session.allocator,
+            &previous,
+            planned,
+            session.instance,
+        )
+    });
+    let relocated = relocated.unwrap_or_else(|refusal| panic!("搬 {items} 项的一批：{refusal:?}"));
+    assert_every_lookup_examines_at_most_a_binary_search(
+        &lookup_work,
+        &format!("搬 {items} 项的一批"),
+    );
+    let relocated_kept = relocated
+        .units_relocated_by_this_publish
+        .iter()
+        .filter(|unit| unit.holder == RelocatedUnitHolder::KeptForTheSnapshots)
+        .count();
+    assert_eq!(
+        (
+            relocated.units_relocated_by_this_publish.len(),
+            u64::try_from(relocated_kept).expect("项数")
+        ),
+        (planned.len(), kept_items),
+        "计划里每一项都搬了，留存的那几项按留存的搬"
+    );
+    session.current = PoolVersion::of_a_version_of_trees(relocated);
+    let relocation_sources = lookup_work.of_the_site(LookupSite::RelocationSources);
+    println!(
+        "publish_lookup_by_identity relocation_batch items={items} kept_items={kept_items} routed_leaves={routed_leaves} relocation_sources={relocation_sources:?}"
+    );
+    let index_builds_of_the_batch = if kept_items == 0 { 2 } else { 3 };
+    assert_eq!(
+        (
+            relocation_sources.index_builds,
+            relocation_sources.mapping_leaf_decodes,
+            relocation_sources.index_lookups
+        ),
+        (
+            index_builds_of_the_batch,
+            routed_leaves,
+            2 * items + routed_leaves
+        ),
+        "搬 {items} 项（其中留存的 {kept_items} 项）的一批：建 {index_builds_of_the_batch} 份索引、进映射那几项的 key 走到的 {routed_leaves} 片叶各解一次、\
+         查项数两倍加解叶次数那么多次（建索引次数、解叶次数、查的次数）：{relocation_sources:?}"
+    );
+}
+
+/// 快照下整份重写过两个文件的池上，整理发两批：两项（一项进映射、一项留存）与八项（四项进映射、四项留存）。
+/// 两批建的索引都是那三份，不随批大小涨（每项现建索引的那一处按项数涨）；查的次数是项数的两倍加解叶次数（找留存条目或取字节改回线性的少了那几次）；
+/// 解叶恰好是进映射那几项的 key 走到的叶（落点改回逐 key 查的按项数解）。
+/// 判别力：`crates/mutations.tsv` 里攻方 RA、RC，正推 M7，与「七·6 第三轮 核源落点：」起头的两行。
+#[test]
+fn relocation_batches_of_two_and_of_eight_units_build_their_indexes_once_and_look_up_each_planned_unit_twice(
+) {
+    let PoolWithASnapshot { mut pool, .. } = pool_with_a_snapshot_of_files(8);
+    for (position, rewritten_inode) in [pool.inodes[1], pool.inodes[2]].into_iter().enumerate() {
+        whole_rewrite_published_once(
+            &mut pool,
+            rewritten_inode,
+            rewritten_inode.0 + 5_000_011,
+            4,
+            common::FIXED_WRITE_TIME_SECONDS + 9_000 + u64::try_from(position).expect("序号"),
+        );
+    }
+    let mut planned_batch_of_two =
+        planned_relocations_of_the_data_units(version_of(&pool.session), pool.inodes[3], 1);
+    planned_batch_of_two.extend(planned_relocations_of_kept_data_units(
+        version_of(&pool.session),
+        0,
+        1,
+    ));
+    relocate_a_batch_and_check_the_relocation_sources(&mut pool, &planned_batch_of_two, 1);
+    let mut planned_batch_of_eight =
+        planned_relocations_of_the_data_units(version_of(&pool.session), pool.inodes[4], 4);
+    planned_batch_of_eight.extend(planned_relocations_of_kept_data_units(
+        version_of(&pool.session),
+        1,
+        4,
+    ));
+    relocate_a_batch_and_check_the_relocation_sources(&mut pool, &planned_batch_of_eight, 4);
+}
+
+/// 4096 项、64 个身份各出现 64 次（次序按与表长互素的步长打乱）的长表上，索引对每个身份取的是表里它最先那一项（比地址，同 `iter().find`），
+/// 按身份升序交出的每一项也是。短表上不稳定排序碰巧也保序，长表上同一身份的几项会被它换了次序。
+/// 判别力：`crates/mutations.tsv` 里正推 M1（稳定排序换成不稳定排序）。
+#[test]
+fn first_entry_of_each_identity_keeps_the_first_entry_of_identities_repeated_across_a_long_shuffled_table(
+) {
+    const ENTRIES: u64 = 4096;
+    const DISTINCT_IDENTITIES: u64 = 64;
+    let table: Vec<(u64, u64)> = (0..ENTRIES)
+        .map(|position| {
+            (
+                position * STRIDE_THAT_SHUFFLES_THE_SYNTHETIC_TABLE % ENTRIES % DISTINCT_IDENTITIES,
+                position,
+            )
+        })
+        .collect();
+    let index = FirstEntryOfEachIdentity::of_the_table(&table, |(identity, _)| *identity);
+    for identity in (0..DISTINCT_IDENTITIES).chain([DISTINCT_IDENTITIES]) {
+        let found_linearly = table.iter().find(|(candidate, _)| *candidate == identity);
+        assert!(
+            is_the_same_entry(index.first_entry_of(&identity), found_linearly),
+            "身份 {identity}：索引查到的 {:?} 不是表里它最先那一项 {found_linearly:?}",
+            index.first_entry_of(&identity)
+        );
+    }
+    assert_the_index_lists_the_first_entry_of_every_identity_once(
+        &index,
+        &table,
+        |(identity, _)| *identity,
+        "4096 项、64 个身份的长表",
+    );
+}
+
+/// 计数器自己会红（七·6 代码轮第三轮判决第三节第 4 条）：在 512 个身份的合成表上建一次索引记一次建、从前往后与从后往前各走一遍各记 512 项看过、
+/// 每个身份连同一个表里没有的各查一次记 513 次查、每次看过的项数在 1 与折半查找的层数之间。
+/// 判别力：`crates/mutations.tsv` 里攻方 RG（删掉从前往后走那一句计数）与「七·6 第三轮 计数：」起头的那一行。
+#[test]
+fn lookup_work_counts_one_entry_per_step_of_a_walk_from_either_end_and_one_lookup_per_search() {
+    let table = synthetic_table_of_identities(IDENTITIES_OF_THE_SMALLER_SYNTHETIC_TABLE);
+    let (index, building) =
+        lookup_work_while(|| FirstEntryOfEachIdentity::of_the_table(&table, |identity| *identity));
+    assert_eq!(
+        building.of_every_site(),
+        LookupWork {
+            index_builds: 1,
+            ..LookupWork::default()
+        },
+        "建一次索引记一次建、别的都不记"
+    );
+    let (walked_forward, forward) = lookup_work_while(|| index.first_entries().count());
+    let (walked_backward, backward) = lookup_work_while(|| index.first_entries().rev().count());
+    let entries = u64::try_from(table.len()).expect("项数");
+    for (walked, work, direction) in [
+        (walked_forward, forward, "从前往后"),
+        (walked_backward, backward, "从后往前"),
+    ] {
+        assert_eq!(
+            (u64::try_from(walked).expect("项数"), work.of_every_site()),
+            (
+                entries,
+                LookupWork {
+                    index_entries_examined: entries,
+                    ..LookupWork::default()
+                }
+            ),
+            "{direction}走一遍：走过 {entries} 项、记 {entries} 项看过，不记查与建"
+        );
+    }
+    let (_, searching) = lookup_work_while(|| {
+        // 迭代上界是表长加一。
+        for identity in table.iter().copied().chain([u64::MAX]) {
+            let _ = index.first_entry_of(&identity);
+        }
+    });
+    let searched = searching.of_every_site();
+    assert!(
+        searched.index_lookups == entries + 1
+            && searched.index_builds == 0
+            && searched.index_entries_examined > entries
+            && searched.index_entries_examined
+                <= (entries + 1) * binary_digits_of(entries)
+            && searched.largest_index_looked_up == entries,
+        "每个身份连同一个表里没有的各查一次：记 {} 次查，每次看过的项在 1 与 {} 之间，最大的索引 {entries} 项：{searched:?}",
+        entries + 1,
+        binary_digits_of(entries)
+    );
+}
````

## 七·6 后续批 mutations-append.tsv

````text
七·6 第三轮 核源落点：找留存的映射条目退回在表上线性找	crates/singlefs-core/src/transaction.rs	        let version = self.version;\n        self.kept_mapping_entries_by_key_built_at_the_first_lookup\n            .get_or_insert_with(|| version.mapping_entries_kept_for_the_snapshots_by_key())\n            .first_entry_of(&mapping_key)	        self.version\n            .mapping_entries_kept_for_the_snapshots\n            .iter()\n            .find(|(key, _)| key.as_slice() == mapping_key)	-p singlefs-harness --test publish_lookup_by_identity -- --exact relocation_batches_of_two_and_of_eight_units_build_their_indexes_once_and_look_up_each_planned_unit_twice	relocation_batches_of_two_and_of_eight_units_build_their_indexes_once_and_look_up_each_planned_unit_twice
七·6 第三轮 核源落点：取计划里单元的字节退回在 units 上线性找	crates/singlefs-core/src/transaction.rs	        RelocatedUnitHolder::RoleOfThisVersion(role) => previous_read_by_key\n            .central_mapping\n            .unit_of_role(role)\n            .bytes\n            .clone(),	        RelocatedUnitHolder::RoleOfThisVersion(role) => previous_read_by_key\n            .version\n            .unit(role)\n            .bytes\n            .clone(),	-p singlefs-harness --test publish_lookup_by_identity -- --exact relocation_batches_of_two_and_of_eight_units_build_their_indexes_once_and_look_up_each_planned_unit_twice	relocation_batches_of_two_and_of_eight_units_build_their_indexes_once_and_look_up_each_planned_unit_twice
七·6 第三轮 删快照：取放掉条目留存的映射条目退回在表上线性找	crates/singlefs-core/src/transaction.rs	        let locations = kept_mapping_entries_by_key\n            .get_or_insert_with(|| previous_version.mapping_entries_kept_for_the_snapshots_by_key())\n            .first_entry_of(&entry.mapping_key.as_slice())\n            .map(|(_, locations)| *locations)	        let _built_and_not_looked_up = kept_mapping_entries_by_key\n            .get_or_insert_with(|| previous_version.mapping_entries_kept_for_the_snapshots_by_key());\n        let locations = previous_version\n            .mapping_entries_kept_for_the_snapshots\n            .iter()\n            .find(|(key, _)| *key == entry.mapping_key)\n            .map(|(_, locations)| *locations)	-p singlefs-harness --test publish_lookup_by_identity -- --exact a_whole_rewrite_under_a_snapshot_and_its_destruction_look_up_kept_mapping_entries_through_indexes_built_once	a_whole_rewrite_under_a_snapshot_and_its_destruction_look_up_kept_mapping_entries_through_indexes_built_once
七·6 第三轮 释放：换下的角色按 inode 号找上一版的文件退回线性找	crates/singlefs-core/src/transaction.rs	        let version = self.version;\n        self.index_built_at_the_first_lookup\n            .get_or_insert_with(|| version.files_other_than_the_first_by_inode())\n            .first_entry_of(&inode)	        self.version.file_other_than_the_first(inode)	-p singlefs-harness --test publish_lookup_by_identity -- --exact a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site	a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site
七·6 第三轮 照抄：extent 树节点按角色取上一版的单元退回线性找	crates/singlefs-core/src/transaction.rs	        let carried = units_of_the_previous_version\n            .expect("照抄的 extent 树节点只出现在接着上一版的发布里")\n            .unit(role_in_the_previous_version);	        let _ = units_of_the_previous_version;\n        let carried = previous\n            .expect("照抄的 extent 树节点只出现在接着上一版的发布里")\n            .unit(role_in_the_previous_version);	-p singlefs-harness --test publish_lookup_by_identity -- --exact a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site	a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site
七·6 第三轮 照抄：第一个文件之外文件的下段节点按 inode 号找文件退回线性找	crates/singlefs-core/src/transaction.rs	                    files_of_the_previous_version\n                        .and_then(|previous_files| previous_files.first_entry_of(inode))	                    previous\n                        .and_then(|previous_version| previous_version.file_other_than_the_first(*inode))	-p singlefs-harness --test publish_lookup_by_identity -- --exact a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site	a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site
七·6 第三轮 照抄：多层码 2 树节点按角色取上一版的单元退回线性找	crates/singlefs-core/src/transaction.rs	                let carried = units_of_the_previous_version\n                    .expect(\n                        "照抄的节点只出现在接着上一版的发布里：上一版的单元索引与上一版一起交进来",\n                    )\n                    .unit(	                let _ = units_of_the_previous_version;\n                let carried = previous_version\n                    .unit(	-p singlefs-harness --test publish_lookup_by_identity -- --exact a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site	a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site
七·6 第三轮 核源落点：落点退回逐 key 查映射、按 key 的索引照用（攻方 RA）	crates/singlefs-core/src/transaction.rs	            let locations = match previous_read_by_key\n                .central_mapping\n                .locations_of_key(&mapping_key)\n            {	            let locations = match central_mapping_locations_in_the_version(previous_read_by_key.version, &mapping_key) {	-p singlefs-harness --test publish_lookup_by_identity -- --exact relocation_batches_of_two_and_of_eight_units_build_their_indexes_once_and_look_up_each_planned_unit_twice	relocation_batches_of_two_and_of_eight_units_build_their_indexes_once_and_look_up_each_planned_unit_twice
七·6 第三轮 释放：按角色读者的落点退回逐 key 查映射、按角色的索引照用（攻方 RB）	crates/singlefs-core/src/transaction.rs	        central_mapping.locations_of_key(key)\n    }\n}	        let _ = central_mapping;\n        central_mapping_locations_in_the_version(version, key)\n    }\n}	-p singlefs-harness --test publish_lookup_by_identity -- --exact a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site	a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site
七·6 第三轮 核源落点：计划里每一项现建按 key 的索引与批量读者（攻方 RC）	crates/singlefs-core/src/transaction.rs	                previous_read_by_key.get_or_insert_with(|| {\n                    MappedUnitsOfAVersionReadByKey::of_the_version(previous)\n                }),	                previous_read_by_key.insert(\n                    MappedUnitsOfAVersionReadByKey::of_the_version(previous)\n                ),	-p singlefs-harness --test publish_lookup_by_identity -- --exact relocation_batches_of_two_and_of_eight_units_build_their_indexes_once_and_look_up_each_planned_unit_twice	relocation_batches_of_two_and_of_eight_units_build_their_indexes_once_and_look_up_each_planned_unit_twice
七·6 第三轮 释放：按角色读者每查一个角色现建两份索引（攻方 RD）	crates/singlefs-core/src/transaction.rs	            .get_or_insert_with(|| IndexesOfAVersionReadByRole {\n                mapped_units_by_role: version.mapped_units_by_role(),	            .insert(IndexesOfAVersionReadByRole {\n                mapped_units_by_role: version.mapped_units_by_role(),	-p singlefs-harness --test publish_lookup_by_identity -- --exact a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site	a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site
七·6 第三轮 快照判定：每个换下的角色现建按角色的索引（攻方 RE）	crates/singlefs-core/src/transaction.rs	    let mapped_units_of_the_previous_version = previous_version.mapped_units_by_role();\n    for identity in &user_visible_roles_replaced {\n        let mapping_key = mapped_units_of_the_previous_version\n            .mapping_key_of(*identity)	    for identity in &user_visible_roles_replaced {\n        let mapping_key = previous_version\n            .mapped_units_by_role()\n            .mapping_key_of(*identity)	-p singlefs-harness --test publish_lookup_by_identity -- --exact a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site	a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site
七·6 第三轮 释放：核两条位置项那一处退回线性找 key、逐 key 查映射（攻方 RF）	crates/singlefs-core/src/transaction.rs	        let locations = mapping_locations_of_a_released_unit(&mut previous_read_by_role, identity);\n        refuse_locations_that_do_not_name_two_pool_devices	        let _ = &mut previous_read_by_role;\n        let (_, key) = previous\n            .mapped_units\n            .iter()\n            .find(|(mapped, _)| *mapped == identity)\n            .expect("进映射的单元每个一把 key");\n        let MappingLookup::Found(locations) = central_mapping_locations_in_the_version(previous, key) else {\n            panic!("上一版的映射里查不到")\n        };\n        refuse_locations_that_do_not_name_two_pool_devices	-p singlefs-harness --test publish_lookup_by_identity -- --exact a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site	a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site
七·6 第三轮 快照判定：进 deadlist 那一支退回线性找 key、逐 key 查映射（正推 M2）	crates/singlefs-core/src/transaction.rs	                let locations = match previous_version_read_by_role.locations_of_role(*identity) {	                let _ = &mut previous_version_read_by_role;\n                let locations = match {\n                    let (_, linear_key) = previous_version.mapped_units.iter().find(|(mapped, _)| *mapped == *identity).expect("进映射的单元每个一把 key");\n                    central_mapping_locations_in_the_version(previous_version, linear_key)\n                } {	-p singlefs-harness --test publish_lookup_by_identity -- --exact a_whole_rewrite_under_a_snapshot_and_its_destruction_look_up_kept_mapping_entries_through_indexes_built_once	a_whole_rewrite_under_a_snapshot_and_its_destruction_look_up_kept_mapping_entries_through_indexes_built_once
七·6 第三轮 释放：释放判定路径那一处退回线性找 key、逐 key 查映射（正推 M3）	crates/singlefs-core/src/transaction.rs	                match previous_read_by_role.locations_of_role(identity) {	                let _ = &mut previous_read_by_role;\n                match {\n                    let (_, linear_key) = previous.mapped_units.iter().find(|(mapped, _)| *mapped == identity).expect("进映射的单元每个一把 key");\n                    central_mapping_locations_in_the_version(previous, linear_key)\n                } {	-p singlefs-harness --test publish_lookup_by_identity -- --exact a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site	a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site
七·6 第三轮 释放：读盘核校验和那一处退回线性找 key、逐 key 查映射（正推 M4）	crates/singlefs-core/src/transaction.rs	        let locations = mapping_locations_of_a_released_unit(&mut previous_read_by_role, identity);\n        quarantined.extend(	        let _ = &mut previous_read_by_role;\n        let locations = {\n            let (_, linear_key) = previous.mapped_units.iter().find(|(mapped, _)| *mapped == identity).expect("进映射的单元每个一把 key");\n            match central_mapping_locations_in_the_version(previous, linear_key) {\n                MappingLookup::Found(found) => found,\n                MappingLookup::NodeMalformedOrNoEntryWithThisKey | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => panic!("上一版的映射里查不到"),\n            }\n        };\n        quarantined.extend(	-p singlefs-harness --test publish_lookup_by_identity -- --exact a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site	a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site
七·6 第三轮 核源落点：每一项另建一份按 key 的索引与批量读者、外层那份照建（正推 M7）	crates/singlefs-core/src/transaction.rs	                previous_read_by_key.get_or_insert_with(|| {\n                    MappedUnitsOfAVersionReadByKey::of_the_version(previous)\n                }),	                {\n                    let _ = previous_read_by_key.get_or_insert_with(|| MappedUnitsOfAVersionReadByKey::of_the_version(previous));\n                    &mut MappedUnitsOfAVersionReadByKey::of_the_version(previous)\n                },	-p singlefs-harness --test publish_lookup_by_identity -- --exact relocation_batches_of_two_and_of_eight_units_build_their_indexes_once_and_look_up_each_planned_unit_twice	relocation_batches_of_two_and_of_eight_units_build_their_indexes_once_and_look_up_each_planned_unit_twice
七·6 第三轮 按身份查：稳定排序换成不稳定排序（正推 M1）	crates/singlefs-core/src/transaction.rs	            pairs.sort_by(|(left, _), (right, _)| left.cmp(right));	            pairs.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));	-p singlefs-harness --test publish_lookup_by_identity -- --exact first_entry_of_each_identity_keeps_the_first_entry_of_identities_repeated_across_a_long_shuffled_table	first_entry_of_each_identity_keeps_the_first_entry_of_identities_repeated_across_a_long_shuffled_table
七·6 第三轮 计数：从前往后走索引不记看过几项（攻方 RG）	crates/singlefs-core/src/transaction.rs	            let (identity, value) = self.remaining.next()?;\n            count(|work| work.index_entries_examined += 1);\n	            let (identity, value) = self.remaining.next()?;\n	-p singlefs-harness --test publish_lookup_by_identity -- --exact lookup_work_counts_one_entry_per_step_of_a_walk_from_either_end_and_one_lookup_per_search	lookup_work_counts_one_entry_per_step_of_a_walk_from_either_end_and_one_lookup_per_search
````

## 判法单元键认并上来的特性 crates.patch（只抄 crates/singlefs-core/ 下的两份）

````diff
diff --git a/crates/singlefs-core/Cargo.toml b/crates/singlefs-core/Cargo.toml
index 9b159de..62d1561 100644
--- a/crates/singlefs-core/Cargo.toml
+++ b/crates/singlefs-core/Cargo.toml
@@ -4,6 +4,10 @@ version = "0.1.0"
 edition = "2021"
 publish = false
 description = "核心层：地址 newtype 与块设备抽象（步 0），事务层与写路径随后各步接上"
+# 构建脚本（build.rs）把这一次编译里开了哪些特性交给直接依赖本包的构建脚本（DEP_SINGLEFS_CORE_ENABLED_FEATURES）：
+# checker 档的构建脚本拿它算判法单元键，认得出别的包在同一次构建里并上来的特性（harness 档的 dev 依赖打开的 lookup-work-counters）。
+# 不链接任何本地库，links 只用来交这一份元数据。
+links = "singlefs-core"
 
 [dependencies]
 # 文件后端写零打洞要调 fallocate(2)（`src/block_device.rs`）；与 Cargo.lock 里已有的 libc 同一份，不新增包。
diff --git a/crates/singlefs-core/build.rs b/crates/singlefs-core/build.rs
new file mode 100644
index 0000000..fccb009
--- /dev/null
+++ b/crates/singlefs-core/build.rs
@@ -0,0 +1,19 @@
+//! 把这一次编译里 cargo 给本包打开的特性交给直接依赖本包的构建脚本：`links` 的元数据，依赖方读 `DEP_SINGLEFS_CORE_ENABLED_FEATURES`
+//! （`singlefs_unit_graph::build_features::features_enabled_on_the_dependencies`，键名是那里的 `ENABLED_FEATURES_METADATA_KEY`）。
+//! cargo 给的 `CARGO_FEATURE_*` 是并过之后的：别的包在同一次构建里替本包打开的特性（harness 档的 dev 依赖打开的 `lookup-work-counters`）
+//! 也在里面，依赖方自己的 `CARGO_FEATURE_*` 看不到它们。checker 档的构建脚本拿这一份算判法单元键。
+//!
+//! 只做交出：不读别的输入、不生成代码，值是变量名里 `CARGO_FEATURE_` 之后那一段（大写、`-` 换成 `_`），排好序、逗号隔开。
+
+fn main() {
+    let mut enabled_feature_environment_names: Vec<String> = std::env::vars_os()
+        .filter_map(|(name, _value)| name.into_string().ok())
+        .filter_map(|name| name.strip_prefix("CARGO_FEATURE_").map(str::to_string))
+        .collect();
+    enabled_feature_environment_names.sort();
+    println!(
+        "cargo::metadata=enabled_features={}",
+        enabled_feature_environment_names.join(",")
+    );
+    println!("cargo::rerun-if-changed=build.rs");
+}
````
