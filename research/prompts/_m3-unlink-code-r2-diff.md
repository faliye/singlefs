# 第 11 批合入的改动全文（m3-unlink-code-r2，2026-10-03）

冻结副本 `/tmp/claude-1000/m3-unlink-code-r2/tree/`；行号以冻结副本为准。三节：实现员 A 的 `crates.patch`（`/tmp/claude-1000/m3-batch11/a/patch/`，与交回时 on-main 那份逐字相同）、实现员 B 的 `crates.patch`（`/tmp/claude-1000/m3-batch11/b/patch/`，同上）、`crates/mutations.tsv` 从合入前到现在的差（含改钉值那一份：1064、2127 改原文，120、156、746、2128 删）。

## 一、实现员 A

~~~~~~diff
diff --git a/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs b/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
index 01cba3a..1c18240 100644
--- a/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
+++ b/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
@@ -19100,9 +19100,9 @@ mod fourth_run_segment_two {
             });
             let floor_raises: &[singlefs_core::mount::RaisedFloor] = match &change {
                 Ok(published) => &published.floor_raises,
+                Err(UserChangeRefused::NoFileVersionToChange { floor_raises, .. }) => floor_raises,
                 Err(
                     UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
-                    | UserChangeRefused::NoFileVersionToChange
                     | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
                     | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
                     | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 55f3bd0..39da40a 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -98,7 +98,8 @@ pub enum MountError {
     /// 判不了候选集，在任何写之前拒绝。
     InstanceTableMalformed,
     Acquisition(AcquisitionFailed),
-    /// 取号之后那一串发布（写行一次、暖机至多 R 次）里有一次没做成：错，连同这次挂载的写入口交得出的写账（[`PublishSequenceFailed`]）。
+    /// 取号之后那一串发布（写行一次、暖机至多 R 次）里有一次没做成，或挂载续做里的一步没发成、不是空间不够：错，连同这次挂载交得出的写账
+    /// （[`PublishSequenceFailed`]：续做那一格的已落盘各次是写行、暖机、推的抬 F 与续做里已落盘的每一次，按先后）。
     /// 装箱（与 `RaiseFloorSequencePublishFailed` 同）：写账带上抬 F 先写系统配置那一步之后，不装箱 `MountError` 就大过 clippy
     /// `result_large_err` 的 128 字节。
     Publish(Box<PublishSequenceFailed>),
@@ -895,7 +896,7 @@ pub struct MountOutput {
     pub parameters_and_device_table: ParametersAndDeviceTableOfTheMount,
     /// 这次挂载重读过哪几样（C554 乙）：读阶段在哪一遍判完、最新那条根的实例表在哪一遍读出来。
     pub rereads: RereadsOfThisMount,
-    /// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）发成的每一步，按先后：接在写行、暖机与推的抬 F 之后，最后一个就是
+    /// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）发成的每一步，按先后：接在写行、暖机与推的抬 F 之后，走到了空时最后一个就是
     /// `Mounted::current`；所选那一版的 `deleted_inodes` 是空的（常态）时空。
     pub deletion_queue_continuation: Vec<TransactionOutput>,
     /// 挂载续做里为空间推的那几串抬 F，按先后（与 `deletion_queue_continuation` 的各步按 txg 交错）：走空里一次删除类发布被拒是空间不够
@@ -904,6 +905,23 @@ pub struct MountOutput {
     pub floor_raises_pushed_by_the_continuation: Vec<RaisedFloor>,
     /// 每一串是因为哪一次被拒才推的（与 `floor_raises_pushed_by_the_continuation` 一一对应）：`SpaceAdmissionRefused` 或 `PlacementRefused`。
     pub refusals_that_pushed_the_continuation_floor_raises: Vec<PublishError>,
+    /// 挂载续做停在哪儿（[`DeletionQueueContinuationOutcome`]）：走到了空，或推满仍放不下、队列留着而挂载照样做成可写。
+    pub deletion_queue_continuation_outcome: DeletionQueueContinuationOutcome,
+}
+
+/// 挂载续做停在哪儿。成员按调用方要做的决定分：照常写；队列还留着——之后会话里每一次用户改动先走空、走不动报空间不够
+/// （`mounted_session::UserChangeRefused::NoSpaceAfterRaisingTheFloor`），正常卸载照常可走。
+#[derive(Debug)]
+pub enum DeletionQueueContinuationOutcome {
+    /// 所选那一版的 `deleted_inodes` 是空的（常态），或续做把它走到了空。
+    WalkedToEmpty,
+    /// 续做里一次删除类发布被拒是空间不够、推满仍放不下（用户 2026-10-03 定「照 D16 做成可写」，D16（发布语义） 已定项 1「准入」那一行
+    /// 「挂载处推满仍不够时挂载照样做成可写」）：挂载照样交回可写，`Mounted::current` 是续做最后落盘的那一版（删除那一步或推的空发布），
+    /// 它的 `deleted_inodes` 不空。`last_refusal` 是最后那一次被拒（`SpaceAdmissionRefused` 或 `PlacementRefused`），`stop` 是不再推的原因。
+    StillShortAfterTheFloorRaises {
+        last_refusal: PublishError,
+        stop: FloorRaiseStop,
+    },
 }
 
 /// 一次可写挂载的空间准入怎么判的（D28（挂载期承诺量） 已定项 1 的式子逐设备合取；D16（发布语义） 已定项 1「准入」那一行：
@@ -4250,10 +4268,25 @@ fn establish_instance<Device: BlockDevice>(
         }
     };
 
+    // 续做之前这次挂载已落盘的发布（写行那次、暖机每次、写行之后推的抬 F 每一次空发布）各自的写，按先后：续做里的一步没发成、
+    // 不是空间不够时随错交出（`PublishSequenceFailed::writes_of_persisted_publishes` 的契约，代码轮 U9）。
+    let writes_of_the_publishes_before_the_continuation: Vec<WritesByStructureKind> =
+        std::iter::once(&row_publish)
+            .chain(&warm_up_publishes)
+            .map(|version| version.writes().clone())
+            .chain(
+                space_admission
+                    .floor_raises()
+                    .iter()
+                    .flat_map(|raised| raised.publishes.iter())
+                    .map(|publish| publish.writes.clone()),
+            )
+            .collect();
     let DeletionQueueContinuation {
         steps_published: deletion_queue_continuation,
         floor_raises: floor_raises_pushed_by_the_continuation,
         refusals_that_pushed_the_floor_raises: refusals_that_pushed_the_continuation_floor_raises,
+        outcome: deletion_queue_continuation_outcome,
     } = continue_the_deletion_queue_until_it_is_empty(
         parameters,
         devices,
@@ -4261,6 +4294,7 @@ fn establish_instance<Device: BlockDevice>(
         &mut current,
         instance,
         start.shadow_ledger,
+        writes_of_the_publishes_before_the_continuation,
     )?;
 
     Ok(Mounted {
@@ -4287,36 +4321,64 @@ fn establish_instance<Device: BlockDevice>(
             deletion_queue_continuation,
             floor_raises_pushed_by_the_continuation,
             refusals_that_pushed_the_continuation_floor_raises,
+            deletion_queue_continuation_outcome,
         },
         allocator,
         current,
     })
 }
 
-/// 挂载续做交回的东西（[`continue_the_deletion_queue_until_it_is_empty`]）：发成的每一步与为空间推的那几串抬 F，两样按 txg 交错。
+/// 挂载续做交回的东西（[`continue_the_deletion_queue_until_it_is_empty`]）：发成的每一步与为空间推的那几串抬 F，两样按 txg 交错；
+/// 走到哪儿停（走到了空，或推满仍放不下、队列留着）。
 struct DeletionQueueContinuation {
     steps_published: Vec<TransactionOutput>,
     floor_raises: Vec<RaisedFloor>,
     refusals_that_pushed_the_floor_raises: Vec<PublishError>,
+    outcome: DeletionQueueContinuationOutcome,
+}
+
+impl DeletionQueueContinuation {
+    /// 续做里已落盘的每一次发布（发成的每一步、推的每一串的每一次空发布）各自的写，按 txg 的先后。
+    /// 推的那几串先写系统配置那一步（`RaisedFloor::system_configuration_writes_before_the_first_publish`）不是发布，不在这里。
+    fn writes_of_the_persisted_publishes_in_txg_order(&self) -> Vec<WritesByStructureKind> {
+        let mut persisted: Vec<&TransactionOutput> = self
+            .steps_published
+            .iter()
+            .chain(
+                self.floor_raises
+                    .iter()
+                    .flat_map(|raised| raised.publishes.iter()),
+            )
+            .collect();
+        persisted.sort_by_key(|publish| publish.root.checkpoint_txg);
+        persisted
+            .into_iter()
+            .map(|publish| publish.writes.clone())
+            .collect()
+    }
 }
 
 /// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）：写行与暖机（连同之后为空间推的抬 F）之后、交回之前，把现行那一版的 `deleted_inodes`
 /// 走到空；与会话每次用户改动先走空共用同一个走空函数（`transaction::walk_the_deletion_queue_until_it_is_empty`），不进取号之前的预演。
 /// 批大小与墓碑容器容量取分配器上装的开关（`mount_writable_with_test_only_switches_and_deletion_switches`）。队列空（常态）时一个写都不发；
-/// 走完 `current` 按 `PoolVersion::of_a_version_of_trees` 重新归成员。交回发成的每一步（按先后）与为空间推的那几串抬 F。
+/// 走完 `current` 按 `PoolVersion::of_a_version_of_trees` 重新归成员。交回发成的每一步（按先后）、为空间推的那几串抬 F 与停在哪儿。
 ///
 /// 走空里的删除类发布（不判式子那一判、落得下那一判照判，D28（挂载期承诺量） 已定项 1）被拒是空间不够的那两种
 /// （[`refusal_is_short_of_space`]）时，推一串抬 F 的空发布（[`push_one_floor_raise_within_the_admission_budget`]）、回收之后接着走
 /// （用户 2026-10-03 定案「挂载续做被空间拒：同样推抬 F 再试」，与会话那一处 `mounted_session` 的走空同一个做法）：每一次被拒的删除类发布
-/// 各算一次准入（这一趟走空有一步落了盘，下一次被拒的就是后面那一次发布，这次准入已推的数从 0 数起）。
-/// 迭代上界：每一轮要么走空交回，要么交回错，要么推成一串（推的数加进这次准入，同一次发布的准入在预算 B − 1 次空发布之内停），
+/// 各算一次准入（用户 2026-10-03 定案 Q-V 预算「甲 每次被拒各算一次」：这一趟走空有一步落了盘，下一次被拒的就是后面那一次发布，
+/// 这次准入已推的数从 0 数起）。推满仍放不下（一次准入的发布数做满、F 已在上限、抬 F 那一串自己也取不到落点或被空间准入拒）时
+/// 挂载照样做成可写、队列留着（用户 2026-10-03 定「照 D16 做成可写」，D16（发布语义） 已定项 1「准入」那一行「挂载处推满仍不够时挂载照样做成可写」）：
+/// 交回 [`DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises`]，`current` 是最后落盘的那一版；之后会话里每一次用户改动先走空、
+/// 走不动报空间不够，正常卸载照常可走。
+/// 迭代上界：每一轮要么走空交回，要么推满仍放不下交回，要么交回错，要么推成一串（推的数加进这次准入，同一次发布的准入在预算 B − 1 次空发布之内停），
 /// 要么这一趟有一步落了盘（队首那个文件少了单元或删掉了，队列与单元数都有限）。跨轮携带的是走到的那一版、已推的数与已落盘的。
 ///
 /// # Errors
-/// 走空里的一步没发成、不是空间不够：错连同这一趟走空的写入口的账（`MountError::Publish`，[`PublishSequenceFailed`]）。
-/// 是空间不够、推满仍不够（一次准入的发布数做满、F 已在上限、抬 F 那一串自己也取不到落点或被空间准入拒）：照旧交回最后那一次被拒
-/// （`MountError::Publish`），推了的那几串不在错里。抬 F 自己报了不是空间不够的错：那个错原样（[`push_one_floor_raise_within_the_admission_budget`] 的 `Err`）。
-/// 三种都是已经发成的那几步与推成的那几串留在盘上，下一次挂载接着走。
+/// 走空里的一步没发成、不是空间不够：错连同这次挂载已落盘各次发布的写（`writes_of_the_publishes_before_the_continuation` 接上续做里已落盘的
+/// 每一步与推的每一次空发布，按 txg 的先后）与这一趟走空的写入口的失败账（`MountError::Publish`，[`PublishSequenceFailed`] 的契约）。
+/// 抬 F 自己报了不是空间不够的错：那个错原样（[`push_one_floor_raise_within_the_admission_budget`] 的 `Err`）。
+/// 两种都是已经发成的那几步与推成的那几串留在盘上。
 fn continue_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
@@ -4324,11 +4386,13 @@ fn continue_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
     current: &mut PoolVersion,
     instance: InstanceGeneration,
     shadow_ledger: ShadowLedger,
+    writes_of_the_publishes_before_the_continuation: Vec<WritesByStructureKind>,
 ) -> Result<DeletionQueueContinuation, MountError> {
     let mut continuation = DeletionQueueContinuation {
         steps_published: Vec::new(),
         floor_raises: Vec::new(),
         refusals_that_pushed_the_floor_raises: Vec::new(),
+        outcome: DeletionQueueContinuationOutcome::WalkedToEmpty,
     };
     let current_version_of_trees = match &*current {
         PoolVersion::WithoutFile(_) => return Ok(continuation),
@@ -4368,17 +4432,20 @@ fn continue_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
             }
             Err(refusal) => refusal,
         };
-        let walk_failed = |cause: PublishError| {
-            MountError::Publish(Box::new(PublishSequenceFailed {
-                cause,
-                writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,
-                writes_of_persisted_publishes: Vec::new(),
-                writes_of_failed_publishes: writes_of_failed_publishes.clone(),
-            }))
-        };
         if !refusal_is_short_of_space(&refusal) {
             *current = PoolVersion::of_a_version_of_trees(walked);
-            return Err(walk_failed(refusal));
+            let writes_of_persisted_publishes: Vec<WritesByStructureKind> =
+                writes_of_the_publishes_before_the_continuation
+                    .iter()
+                    .cloned()
+                    .chain(continuation.writes_of_the_persisted_publishes_in_txg_order())
+                    .collect();
+            return Err(MountError::Publish(Box::new(PublishSequenceFailed {
+                cause: refusal,
+                writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,
+                writes_of_persisted_publishes,
+                writes_of_failed_publishes,
+            })));
         }
         match push_one_floor_raise_within_the_admission_budget(
             parameters,
@@ -4395,9 +4462,14 @@ fn continue_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
                     .refusals_that_pushed_the_floor_raises
                     .push(refusal);
             }
-            Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(_stop)) => {
+            Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(stop)) => {
                 *current = PoolVersion::of_a_version_of_trees(walked);
-                return Err(walk_failed(refusal));
+                continuation.outcome =
+                    DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises {
+                        last_refusal: refusal,
+                        stop,
+                    };
+                return Ok(continuation);
             }
             Err(cause) => {
                 *current = PoolVersion::of_a_version_of_trees(walked);
@@ -4528,6 +4600,8 @@ fn push_floor_raises_after_the_row_publish<Device: BlockDevice>(
 /// （块设备错这一类不是「推满仍不够」；那时实例已取、行已写，写行、暖机与之前推成的那几串已落盘，它们的写账随错交出，实审 A1b Q5）；
 /// 盘上读来的实例代号或 txg 已到顶、下一个号装不下（`SequenceNumberPastTheTopOfItsRange`，取号之前）；
 /// 抬 F 那一串自己也取不到落点、被空间准入拒的，是推满仍不够，挂载照样做成（`MountSpaceAdmission::StillShortAfterTheFloorRaises`）。
+/// 挂载续做里推满仍放不下同样不报错：挂载照样做成、队列留着（`MountOutput::deletion_queue_continuation_outcome`）；续做里的一步没发成、
+/// 不是空间不够才报 `Publish`（[`continue_the_deletion_queue_until_it_is_empty`]）。
 pub fn mount_writable<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index 97a2adb..05d14fa 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -93,7 +93,15 @@ pub enum UserChangeRefused {
     /// 不经这一层），在任何读写之前返回；或有树而文件集合为空（`PoolVersion::WithTreesAndNoFile`，D8（核心索引结构） 已定项 16 第 5 条：
     /// 删完之后只许建 inode（size 0）、写行与空发布）而这次改动不是建 inode——先走空队列（那一版的 `deleted_inodes` 根指针置零，
     /// 走空一个写都不发；走空把带文件的一版走成文件集合为空的，那几步已落盘），再拒这次改动。
-    NoFileVersionToChange,
+    /// 走空发成的那几版与走空里为空间推的那几串抬 F 已落盘、带在这里（代码轮 U10：与别的被拒成员一样交回被拒之前发成的，
+    /// 调用方逐版比得到）；树表 0 条那一格与队列本来就空的那一格没有走空，三样都是空的。
+    NoFileVersionToChange {
+        floor_raises: Vec<RaisedFloor>,
+        /// 每一串是因为哪一次被拒才推的（与 `floor_raises` 一一对应）。
+        refusals_that_pushed_the_floor_raises: Vec<PublishError>,
+        /// 被拒之前发成的那几版（[`VersionsPublishedInThisChange`]）：走空的每一步。
+        versions_published: VersionsPublishedInThisChange,
+    },
     /// 盘表的身份与这次挂载核过的那一份相同，`devices` 里的这几块（按交进来的次序）却不「可见」：两个系统配置槽里一份本池自证过的
     /// （整槽校验和过、fsid 与本池相同）都没有——换上去的空盘、别的池的盘、两槽都读不出（D18（块里携带什么信息） 已定项 11
     /// 「可写挂载的顺序」；与可写挂载取号之前的逐盘核第一支、挂着之后收盘表的入口同一判，`mount::devices_without_a_self_verified_system_configuration`）。
@@ -322,7 +330,13 @@ impl MountedSession {
         let current_version_of_trees = match &*current {
             PoolVersion::WithFile(current_version_of_trees)
             | PoolVersion::WithTreesAndNoFile(current_version_of_trees) => current_version_of_trees,
-            PoolVersion::WithoutFile(_) => return Err(UserChangeRefused::NoFileVersionToChange),
+            PoolVersion::WithoutFile(_) => {
+                return Err(no_file_version_to_change(PublishedInThisChange {
+                    floor_raises: Vec::new(),
+                    refusals_that_pushed_the_floor_raises: Vec::new(),
+                    versions_published: Vec::new(),
+                }))
+            }
         };
         let devices_not_visible =
             devices_without_a_self_verified_system_configuration(devices.as_slice(), parameters);
@@ -379,7 +393,7 @@ impl MountedSession {
         }
         let mut publishes_pushed = 0;
         // 按成员分流（类型上放什么）：有树而文件集合为空的一版只接建 inode。走空把带文件的一版走成这一形时，走空的那几步与推的抬 F
-        // 已落盘、不在这个拒绝里（`NoFileVersionToChange` 不带它们）。
+        // 已落盘，带在这个拒绝里（`NoFileVersionToChange` 的三样，代码轮 U10）。
         let version_to_change = match current {
             PoolVersion::WithFile(file_version) => file_version,
             PoolVersion::WithTreesAndNoFile(emptied_version) => match change {
@@ -387,10 +401,12 @@ impl MountedSession {
                 UserChange::Overwrite(_)
                 | UserChange::SequentialWrite(_)
                 | UserChange::Unlink { .. } => {
-                    return Err(UserChangeRefused::NoFileVersionToChange);
+                    return Err(no_file_version_to_change(published_in_this_change));
                 }
             },
-            PoolVersion::WithoutFile(_) => return Err(UserChangeRefused::NoFileVersionToChange),
+            PoolVersion::WithoutFile(_) => {
+                return Err(no_file_version_to_change(published_in_this_change))
+            }
         };
         loop {
             let refusal = match publish_the_change_once(
@@ -447,6 +463,20 @@ impl MountedSession {
     }
 }
 
+/// 现行那一版没有可改的文件：拒成 [`UserChangeRefused::NoFileVersionToChange`]，带上这次改动里已经落盘的（走空的那几步与推的抬 F）。
+fn no_file_version_to_change(published_in_this_change: PublishedInThisChange) -> UserChangeRefused {
+    let PublishedInThisChange {
+        floor_raises,
+        refusals_that_pushed_the_floor_raises,
+        versions_published,
+    } = published_in_this_change;
+    UserChangeRefused::NoFileVersionToChange {
+        floor_raises,
+        refusals_that_pushed_the_floor_raises,
+        versions_published,
+    }
+}
+
 /// 这次用户改动里已经落盘的：推了的几串抬 F（按先后）与推它们的那几次被拒（一一对应），推的抬 F 之外发成的每一版（按先后）。
 struct PublishedInThisChange {
     floor_raises: Vec<RaisedFloor>,
diff --git a/crates/singlefs-harness/src/history.rs b/crates/singlefs-harness/src/history.rs
index caabf41..6fab81c 100644
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -57,8 +57,8 @@ use crate::fault_injection::{FaultInjectingBlockDevice, SharedFaultPlan};
 use crate::memory_pool::{MemoryPool, RecordCheck, SparseBlockDevice};
 use crate::model::{
     IdealModel, ModelAnswer, ModelCheckpointTxg, ModelDeviceIdentity, ModelDisagreement,
-    ModelJudgementCounts, ModelPoolGeometry, ModelRingPosition, ModelRootKey, ObservedEffect,
-    ObservedOutcome, ObservedRefusalReason,
+    ModelJudgementCounts, ModelPoolGeometry, ModelRefusalReason, ModelRingPosition, ModelRootKey,
+    ObservedEffect, ObservedOutcome, ObservedRefusalReason,
 };
 use crate::model_comparison::{
     model_root_key, observed_mount_carrying_to_zero_unit_publishes,
@@ -1330,15 +1330,11 @@ pub enum MissingPrecondition {
     /// 崩溃恢复抛弃根：最新那次发布的记录一个单元都没点名（树表 0 条上的零单元发布）——点名验证恒过，恢复照样施加它、
     /// 走到的就是它，抛弃不了。
     NewestPublishNamesNoUnit,
-    /// 现行版本的 `deleted_inodes` 不空（unlink 的一批被拒之后、回退到删除中的一版之后）：覆盖写与 unlink 经会话先走空，
-    /// 走成有树而文件集合为空的一版、再拒成 `UserChangeRefused::NoFileVersionToChange`，那个成员不带走空的那几步与推的抬 F
-    /// （`crates/singlefs-core/src/mounted_session.rs` 里 `NoFileVersionToChange` 的文档），模型比不了已落盘的那几版——执行器不调。
-    /// 走空在挂载续做那一处照比（`MountOutput::deletion_queue_continuation`）。
-    CurrentVersionHasAFileBeingDeleted,
     /// 现行版本有树、第一个文件已删完（在删完的一版上建过 inode 之后，实现归带文件的一版）：覆盖写与 unlink 只写、只删第一个文件
     /// （`transaction::live_record_of_the_first_file` 拒成 `WriteToAnInodeThatIsNotInTheTree`），模型只罩 inode 1 一个文件、没有这条理由——执行器不调。
     CurrentVersionWithoutTheFirstFile,
-    /// 现行版本里第一个文件还在（活着或删除中）：建 inode 只接在第一个文件已删完的一版上（`HistoryOperation::CreateInode` 的文档）。
+    /// 现行版本里第一个文件还活着：建 inode 只接在第一个文件已删完的一版上（`HistoryOperation::CreateInode` 的文档）；
+    /// 删除中的那一版照调，会话先走空、把它走成删完的一版（代码轮 U11）。
     CurrentVersionStillHasTheFirstFile,
 }
 
@@ -1355,7 +1351,6 @@ impl MissingPrecondition {
             }
             MissingPrecondition::NoReadableRootBeforeTheNewest => "最新那条根之前没有可读根",
             MissingPrecondition::NewestPublishNamesNoUnit => "最新那次发布一个单元都没点名",
-            MissingPrecondition::CurrentVersionHasAFileBeingDeleted => "现行版本有删除中的文件",
             MissingPrecondition::CurrentVersionWithoutTheFirstFile => "现行版本里第一个文件已删完",
             MissingPrecondition::CurrentVersionStillHasTheFirstFile => "现行版本里第一个文件还在",
         }
@@ -1840,19 +1835,6 @@ pub fn allocation_statistic_mechanism(detail: &str) -> Option<AllocationStatisti
     })
 }
 
-/// 这次失败里 I-3.1 那几条（逐盘各一条）带的机理标识：取第一条读得出来的。
-/// 一条 I-3.1 都没有、或那几条文字里都没有机理标识，都是 None——机理读不出来时「已知红」一条都不接。
-#[must_use]
-fn allocation_statistic_mechanism_of(
-    observation: &FailureObservation,
-) -> Option<AllocationStatisticMechanism> {
-    observation
-        .violations
-        .iter()
-        .filter(|(invariant, _)| *invariant == "I-3.1")
-        .find_map(|(_, detail)| allocation_statistic_mechanism(detail))
-}
-
 /// 「已知红」清单的一条：形态与它在增补 2 收口表里的那一行。
 pub struct KnownRedForm {
     /// 增补 2 收口表（`.claude/kb/milestone/02-second-txn.md`「增补 2」）里的哪一行。
@@ -1862,46 +1844,11 @@ pub struct KnownRedForm {
     pub matches: fn(&FailureObservation) -> bool,
 }
 
-/// 没有 panic、执行器没判出失败、模型没对不上、记录核对器也没判出、判红的只有 I-3.1、而且是记账的已分配大于遍历全部有效根得到的
-/// （记账多算，不是少算）。
-fn only_allocated_statistic_above_walked(observation: &FailureObservation) -> bool {
-    observation.panic.is_none()
-        && observation.harness_judgement.is_none()
-        && observation.model_disagreement.is_none()
-        && observation.record_check == RecordCheck::default()
-        && !observation.violations.is_empty()
-        && observation.violations.iter().all(|(invariant, detail)| {
-            *invariant == "I-3.1"
-                && allocated_and_walked_bytes(detail)
-                    .is_some_and(|(allocated, walked)| allocated > walked)
-        })
-}
-
-/// F 落在回退留下的空档里（F 那个 txg 上的根全属于被抛弃的实例）、只有 I-3.1 红且记账多于遍历，
-/// 而且 checker 报的机理是「回退下界把环里读得到的根挡在了遍历之外」
-/// （代码三方第一轮判决第二节第 1 条收窄：此前不看空档，不经回退的抬 F 之后记账多算也被接走）。
-///
-/// 两条不再要求，都因为机理标识把它们替代掉了（用户 2026-09-20 定案第 6 条）：
-/// 「这一步是抬 F」——活盘面那一路上 `raised_floor_lands_only_on_abandoned_roots` 只有抬 F 那一步算得出来（别的步恒 None），
-/// 卡着操作种类只会把崩溃状态上同一机理的盘面判成新发现；
-/// 「根环没转圈」——环转没转圈与这条机理正交：F 之下的根本来就不走，它们掉出环一个字节都不差。
-fn raise_after_rollback_leaves_allocated_statistic_above_walked(
-    observation: &FailureObservation,
-) -> bool {
-    observation.raised_floor_lands_only_on_abandoned_roots == Some(true)
-        && only_allocated_statistic_above_walked(observation)
-        && allocation_statistic_mechanism_of(observation)
-            .is_some_and(|mechanism| mechanism.the_floor_dropped_readable_roots())
-}
-
 /// 「已知红」清单。修好一条就删一条，删掉之后那条的复现（`tests/random_histories.rs` 里钉着）要转绿。
-pub const KNOWN_RED_FORMS: [KnownRedForm; 1] = [
-    KnownRedForm {
-        closeout_table_row: "增补 2 收口表第 43 行",
-        shape: "回退之后把 F 抬进回退目标根与新实例第一次发布之间（抬之前的镜像上，F 那个 txg 上的根全属于被抛弃的实例）：抬 F 那一步之后 I-3.1（已分配统计对得上） 红、记账的已分配大于遍历全部有效根得到的、别的不变量都不红、没有 panic、根环没转圈",
-        matches: raise_after_rollback_leaves_allocated_statistic_above_walked,
-    },
-];
+/// 增补 2 收口表第 43 行（F 抬进被抛弃实例留下的空档、抬 F 之后只有 I-3.1 红、记账多于遍历）取丁-defer 修好
+/// （用户 2026-09-28 定），它的最小复现（`tests/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs`）
+/// 与随机历史那边原先的复现在 2026-10-03 的副本上都跑绿，那一条从清单里删了（代码轮收口表第 43 行那一项）；清单此刻是空的。
+pub const KNOWN_RED_FORMS: [KnownRedForm; 0] = [];
 
 /// 新发现的「同一个」：panic 按位置、违例按判红的不变量集合。收缩只留签名不变的删法。
 #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
@@ -2976,14 +2923,12 @@ fn apply_publish_overwrite(
     let Some(current_file_version) = open_session.current.file_version() else {
         return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
     };
-    if !current_file_version.deleted_inodes.is_empty() {
-        return AppliedStep::not_applicable(
-            MissingPrecondition::CurrentVersionHasAFileBeingDeleted,
-        );
-    }
     if current_file_version.inode_record.is_none() {
         return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutTheFirstFile);
     }
+    // 现行那一版有删除中的文件（unlink 的一批被拒之后、回退到删除中的一版之后）：照调，会话先走空、把它走成有树而文件集合为空的一版、
+    // 再拒成 `UserChangeRefused::NoFileVersionToChange`，那个成员带着走空的那几步与推的抬 F（代码轮 U10），逐版交模型比（代码轮 U11）。
+    let queue_is_walked_first = !current_file_version.deleted_inodes.is_empty();
     let content = content_choice.bytes();
     let records_before = open_session.allocator.records().to_vec();
     let stream_length_before = stream.operation_count();
@@ -3022,6 +2967,17 @@ fn apply_publish_overwrite(
         publishes_in_this_mount,
         carried_to_zero_unit_publishes,
     });
+    if queue_is_walked_first {
+        return settle_user_change_comparing_every_version_published(
+            session_after_the_change,
+            model,
+            &|ideal_model: &IdealModel| ideal_model.answer_publish_overwrite(&content),
+            &records_before,
+            &change_result,
+            stream,
+            stream_length_before,
+        );
+    }
     settle_user_change(
         session_after_the_change,
         model,
@@ -3128,12 +3084,14 @@ fn settle_user_change(
         Ok(published) => &published.floor_raises,
         Err(
             UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
-            | UserChangeRefused::NoFileVersionToChange
             | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
             | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
             | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. },
         ) => &[],
-        Err(UserChangeRefused::Publish { floor_raises, .. }) => floor_raises,
+        Err(
+            UserChangeRefused::Publish { floor_raises, .. }
+            | UserChangeRefused::NoFileVersionToChange { floor_raises, .. },
+        ) => floor_raises,
         Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused)) => &refused.floor_raises,
         Err(UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(refused)) => {
             &refused.floor_raises
@@ -3240,9 +3198,12 @@ fn member_and_reason_of_a_refused_user_change(
             "UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount".to_string(),
             ObservedRefusalReason::Unexplained,
         ),
-        UserChangeRefused::NoFileVersionToChange => (
+        // 有树而文件集合为空的一版上不是建 inode 的改动（会话先走空、走成这一形之后拒）：D8（核心索引结构） 已定项 16 第 5 条那一条理由。
+        UserChangeRefused::NoFileVersionToChange { .. } => (
             "UserChangeRefused::NoFileVersionToChange".to_string(),
-            ObservedRefusalReason::Unexplained,
+            ObservedRefusalReason::Explained(
+                ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile,
+            ),
         ),
         // 执行器交的是整池那两块盘、没换过盘（Z3-A 乙那一判）：走不到，走到了就按说不出理由的拒绝判。
         UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. } => (
@@ -3289,9 +3250,13 @@ fn floor_raises_and_versions_published_of_a_user_change(
 ) {
     match change_result {
         Ok(published) => (&published.floor_raises, &published.versions_published),
+        Err(UserChangeRefused::NoFileVersionToChange {
+            floor_raises,
+            versions_published,
+            ..
+        }) => (floor_raises, versions_published),
         Err(
             UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
-            | UserChangeRefused::NoFileVersionToChange
             | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
             | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
             | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. },
@@ -3348,7 +3313,8 @@ fn floor_raises_and_versions_in_txg_order<'change>(
 }
 
 /// unlink 第一个文件（`HistoryOperation::Unlink`）：经挂着的会话发（意图那次 + 走空，D8（核心索引结构） 已定项 16）。
-/// 前提：现行那一版带文件、`deleted_inodes` 空（[`MissingPrecondition::CurrentVersionHasAFileBeingDeleted`]）。
+/// 前提：现行那一版带文件、第一个文件那条记录在。删除中的那一版照调：会话先走空、把它走成有树而文件集合为空的一版、再拒成
+/// `UserChangeRefused::NoFileVersionToChange`，走空的那几步逐版交模型比（代码轮 U11）。
 fn apply_unlink(pool: &mut HistoryPool) -> AppliedStep {
     let HistoryPool {
         devices,
@@ -3363,11 +3329,6 @@ fn apply_unlink(pool: &mut HistoryPool) -> AppliedStep {
     let Some(current_file_version) = open_session.current.file_version() else {
         return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
     };
-    if !current_file_version.deleted_inodes.is_empty() {
-        return AppliedStep::not_applicable(
-            MissingPrecondition::CurrentVersionHasAFileBeingDeleted,
-        );
-    }
     if current_file_version.inode_record.is_none() {
         return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutTheFirstFile);
     }
@@ -3410,7 +3371,7 @@ fn apply_unlink(pool: &mut HistoryPool) -> AppliedStep {
     settle_user_change_comparing_every_version_published(
         session_after_the_change,
         model,
-        IdealModel::answer_unlink_intent,
+        &IdealModel::answer_unlink_intent,
         &records_before,
         &change_result,
         stream,
@@ -3426,7 +3387,7 @@ fn apply_unlink(pool: &mut HistoryPool) -> AppliedStep {
 fn settle_user_change_comparing_every_version_published(
     session: &mut WritableSession,
     model: &mut IdealModel,
-    answer_of_the_change: fn(&IdealModel) -> Result<ModelAnswer, ModelDisagreement>,
+    answer_of_the_change: &dyn Fn(&IdealModel) -> Result<ModelAnswer, ModelDisagreement>,
     records_before: &[AllocationRecord],
     change_result: &Result<UserChangePublished, UserChangeRefused>,
     stream: &SharedStream,
@@ -3540,7 +3501,8 @@ fn settle_user_change_comparing_every_version_published(
 }
 
 /// 建一个 size 0 的 inode（`HistoryOperation::CreateInode`）：经挂着的会话发（`UserChange::NewInodes`，count 1）。
-/// 前提：第一个文件已删完——现行那一版有树而文件集合为空，或是在它上面建过 inode 的一版（带文件、没有第一个文件那条记录、队列空）。
+/// 前提：第一个文件不再活着——现行那一版有树而文件集合为空，或是在它上面建过 inode 的一版（带文件、没有第一个文件那条记录、队列空），
+/// 或第一个文件删除中的那一版（会话先走空，代码轮 U11）。
 fn apply_create_inode(pool: &mut HistoryPool, write_time_seconds: u64) -> AppliedStep {
     let HistoryPool {
         devices,
@@ -3557,7 +3519,8 @@ fn apply_create_inode(pool: &mut HistoryPool, write_time_seconds: u64) -> Applie
             return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
         }
         PoolVersion::WithFile(version) => {
-            if version.inode_record.is_some() || !version.deleted_inodes.is_empty() {
+            // 删除中的那一版（队列不空）照调：会话先走空、把它走成删完的一版再建（代码轮 U11）。
+            if version.inode_record.is_some() && version.deleted_inodes.is_empty() {
                 return AppliedStep::not_applicable(
                     MissingPrecondition::CurrentVersionStillHasTheFirstFile,
                 );
@@ -3603,7 +3566,7 @@ fn apply_create_inode(pool: &mut HistoryPool, write_time_seconds: u64) -> Applie
     settle_user_change_comparing_every_version_published(
         session_after_the_change,
         model,
-        IdealModel::answer_create_inode,
+        &IdealModel::answer_create_inode,
         &records_before,
         &change_result,
         stream,
@@ -3828,6 +3791,8 @@ fn settle_mount(
             }
             // 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）：接在推的抬 F 之后，每一步照模型的「走空的一步」比；
             // 走空里为空间推的那几串抬 F（用户 2026-10-03 定案「挂载续做被空间拒：同样推抬 F 再试」）按 txg 插在各步之间，照模型的抬 F 逐串比。
+            // 推满仍放不下（`DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises`，用户 2026-10-03 定「照 D16 做成可写」）时挂载照样做成、
+            // 队列留着：模型的现行那一版停在删除中，那一次被拒交给这次挂载之后会话里下一次用户改动的走空重判（会话先走空，`settle_user_change_comparing_every_version_published`）。
             for published in floor_raises_and_versions_in_txg_order(
                 &mounted.output.floor_raises_pushed_by_the_continuation,
                 &mounted.output.deletion_queue_continuation,
@@ -6403,93 +6368,46 @@ mod tests {
         assert!(
             matches!(
                 classify_failure(without_the_judgement),
-                HistoryEnding::KnownRed { form: 0, .. }
+                HistoryEnding::NewFinding {
+                    signature: FailureSignature::CheckerViolations { .. },
+                    ..
+                }
             ),
-            "去掉执行器判的那一条，同一个观察是清单那一形（收口表第 43 行）"
+            "去掉执行器判的那一条，签名是 checker 判红的那几条；收口表第 43 行那一形已从清单里删了，照新发现报"
         );
     }
 
-    /// 「已知红」按机理认，不按签名认（代码三方 m2-supp3-item3-code-r1 判决 K6 的假阴那一半）：
-    /// 同一个签名（只有 I-3.1 红、记账多于遍历）下，机理对得上才接进清单那一形（收口表第 43 行）。
-    /// 根环转过一圈把 F 之上的根挤出了环那一形（收口表第 ② 行）已经修好、从清单里删了：同一个观察现在是新发现。
+    /// 清单删空之后（收口表第 ② 行、第 43 行两形都修好、都删了），只有 I-3.1 红、记账多于遍历的观察不论 checker 报的机理是哪一种、
+    /// F 落没落在被抛弃实例留下的空档里，一律照新发现报。
     #[test]
-    fn known_red_forms_are_matched_by_mechanism_not_by_signature() {
-        let with_detail = |detail: String| FailureObservation {
-            position: StepPosition::Operation(21),
-            operation_kind: Some(HistoryOperationKind::PublishOverwrite),
-            violations: vec![("I-3.1", detail)],
-            panic: None,
-            newest_ring_root_txg: Some(26),
-            root_ring_slot_count: Some(24),
-            harness_judgement: None,
-            model_disagreement: None,
-            raised_floor_lands_only_on_abandoned_roots: None,
-            record_check: RecordCheck::default(),
-        };
-        assert!(
-            matches!(
-                classify_failure(with_detail(allocation_statistic_detail(26, 3, 0, 0))),
-                HistoryEnding::NewFinding { .. }
-            ),
-            "环转过一圈把 F 之上的根挤出了环、记账多算：收口表第 ② 行修好了，清单里没有这一形，照新发现报"
-        );
-        assert!(
-            matches!(
-                classify_failure(with_detail(
-                    "盘 0：记账的已分配 Some(3817472)，遍历全部有效根得到 3801088".to_string()
-                )),
-                HistoryEnding::NewFinding { .. }
-            ),
-            "说明文字里没有机理标识：认不出机理就不接，照新发现报"
-        );
+    fn with_the_known_red_list_empty_an_allocated_statistic_above_walked_is_a_new_finding_whatever_its_mechanism(
+    ) {
+        assert!(KNOWN_RED_FORMS.is_empty(), "清单是空的");
         let raised_into_the_gap = raised_into_the_gap_observation();
-        // 环转没转圈与这条机理正交：环转过了、而 F 之上一条根都没掉出环（最老的自证过的根就在 F 之下），照样是这一形。
-        let mut the_ring_also_turned = with_detail(allocation_statistic_detail(27, 4, 8, 4));
+        let mut the_ring_also_turned = raised_into_the_gap.clone();
+        the_ring_also_turned.violations = vec![("I-3.1", allocation_statistic_detail(27, 4, 8, 4))];
         the_ring_also_turned.newest_ring_root_txg = Some(27);
-        the_ring_also_turned.raised_floor_lands_only_on_abandoned_roots = Some(true);
-        assert!(
-            matches!(
-                classify_failure(the_ring_also_turned),
-                HistoryEnding::KnownRed { form: 0, .. }
-            ),
-            "环也转过一圈、但缺口出在 F 上：还是清单那一形，不是新发现"
-        );
-        assert!(
-            matches!(
-                classify_failure(raised_into_the_gap.clone()),
-                HistoryEnding::KnownRed { form: 0, .. }
-            ),
-            "F 落在空档里、环没转圈、机理对得上：接进清单那一形"
-        );
-        let mut nothing_dropped_by_the_floor = raised_into_the_gap.clone();
-        nothing_dropped_by_the_floor.violations =
-            vec![("I-3.1", allocation_statistic_detail(9, 0, 5, 0))];
-        assert!(
-            matches!(
-                classify_failure(nothing_dropped_by_the_floor),
-                HistoryEnding::NewFinding { .. }
-            ),
-            "F 一条读得到的根都没挡掉：缺口不出在这条机理上，照新发现报"
-        );
-        let mut floor_not_in_a_gap = raised_into_the_gap.clone();
-        floor_not_in_a_gap.raised_floor_lands_only_on_abandoned_roots = Some(false);
-        assert!(
-            matches!(
-                classify_failure(floor_not_in_a_gap),
-                HistoryEnding::NewFinding { .. }
-            ),
-            "F 那个 txg 上的根不全属于被抛弃的实例：不经回退的抬 F 之后记账多算，照新发现报"
-        );
-        // 这一形不再卡「这一步是抬 F」：崩溃状态摆在抬 F 之后的别的步上，同一机理照样接得进清单那一形。
-        let mut later_step = raised_into_the_gap;
+        let mut later_step = raised_into_the_gap.clone();
         later_step.operation_kind = Some(HistoryOperationKind::PublishOverwrite);
-        assert!(
-            matches!(
-                classify_failure(later_step),
-                HistoryEnding::KnownRed { form: 0, .. }
+        for (observation, what) in [
+            (
+                raised_into_the_gap,
+                "F 落在空档里、环没转圈、机理是 F 挡掉了读得到的根",
             ),
-            "抬 F 之后的别的步上同一机理照样是清单那一形"
-        );
+            (the_ring_also_turned, "环也转过一圈、缺口出在 F 上"),
+            (later_step, "抬 F 之后的别的步上同一机理"),
+        ] {
+            assert!(
+                matches!(
+                    classify_failure(observation),
+                    HistoryEnding::NewFinding {
+                        signature: FailureSignature::CheckerViolations { .. },
+                        ..
+                    }
+                ),
+                "{what}：照新发现报"
+            );
+        }
     }
 
     /// 记录核对器判红的不接进「已知红」：签名是 `RecordCheck`，照新发现报。
@@ -6514,9 +6432,12 @@ mod tests {
         assert!(
             matches!(
                 classify_failure(observation),
-                HistoryEnding::KnownRed { form: 0, .. }
+                HistoryEnding::NewFinding {
+                    signature: FailureSignature::CheckerViolations { .. },
+                    ..
+                }
             ),
-            "去掉记录核对器那一条，同一个观察是清单那一形（收口表第 43 行）"
+            "去掉记录核对器那一条，签名是 checker 判红的那几条（收口表第 43 行那一形已从清单里删了）"
         );
     }
 }
diff --git a/crates/singlefs-harness/src/model.rs b/crates/singlefs-harness/src/model.rs
index cc89c84..b722c06 100644
--- a/crates/singlefs-harness/src/model.rs
+++ b/crates/singlefs-harness/src/model.rs
@@ -468,6 +468,9 @@ pub enum ModelRefusalReason {
     /// 「乙 重读后再判」，`research/prompts/c554-fix-forks.md` 第 1 行；重读次数取自 D16（发布语义） 已定项 1「根槽这一次读坏」那一行的
     /// 「重读一次」，挂进 D23（journal 的角色与格式） 已定项 14）：取号之前拒，盘上逐字节不变。
     NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread,
+    /// 现行那一版有树而文件集合为空（D8（核心索引结构） 已定项 16 第 5 条：删完之后只许建 inode（size 0）、写行与空发布），
+    /// 这次改动不是建 inode：覆盖写、unlink 先走空队列（走空把删除中的那一版走成这一形），再拒（`UserChangeRefused::NoFileVersionToChange`）。
+    OnlyNewInodesOnAVersionWithTreesAndNoFile,
 }
 
 impl ModelRefusalReason {
@@ -493,6 +496,9 @@ impl ModelRefusalReason {
             ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread => {
                 "系统配置见证过比所选那一版新的发布、重读一次仍读不出（C554 乙）"
             }
+            ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile => {
+                "有树而文件集合为空的一版只许建 inode"
+            }
         }
     }
 
@@ -509,9 +515,8 @@ impl ModelRefusalReason {
             | ModelRefusalReason::RollbackTargetWithoutFile
             | ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem
             | ModelRefusalReason::FloorAboveCeiling
-            | ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread => {
-                false
-            }
+            | ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread
+            | ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile => false,
         }
     }
 }
@@ -1336,16 +1341,22 @@ impl IdealModel {
         ))
     }
 
-    /// 覆盖写（`transaction::publish_overwrite`）：接在现行的带文件的一版后面。
+    /// 覆盖写（`transaction::publish_overwrite`）：接在现行的带文件的一版后面。现行那一版有树而文件集合为空时（执行器在删除中的那一版上调，
+    /// 会话先走空、把它走成这一形）必须拒（[`ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile`]）。
     ///
     /// # Errors
-    /// 模型里没有可写会话，或现行版本没有文件（执行器只在带文件时调）：会话状态与实现对不上。
+    /// 模型里没有可写会话，或现行版本没有文件、也不是有树而文件集合为空（执行器只在带文件或删除中时调）：会话状态与实现对不上。
     pub fn answer_publish_overwrite(
         &self,
         content: &[u8],
     ) -> Result<ModelAnswer, ModelDisagreement> {
         let session = self.open_session()?;
         let current = &session.current;
+        if let Some(refused) = self.answer_refusing_a_change_on_a_version_with_trees_and_no_file(
+            ModelOperationKind::PublishOverwrite,
+        ) {
+            return Ok(refused);
+        }
         if current.contents.live_file().is_none() {
             return Err(ModelDisagreement::new(
                 ModelDisagreementAspect::SessionState,
@@ -1420,6 +1431,11 @@ impl IdealModel {
     pub fn answer_unlink_intent(&self) -> Result<ModelAnswer, ModelDisagreement> {
         let session = self.open_session()?;
         let current = &session.current;
+        if let Some(refused) = self.answer_refusing_a_change_on_a_version_with_trees_and_no_file(
+            ModelOperationKind::PublishUnlinkIntent,
+        ) {
+            return Ok(refused);
+        }
         if current.contents.live_file().is_none() {
             return Err(ModelDisagreement::new(
                 ModelDisagreementAspect::SessionState,
@@ -1526,6 +1542,27 @@ impl IdealModel {
         ))
     }
 
+    /// 现行那一版有树而文件集合为空时，这次不是建 inode 的改动（覆盖写、unlink）必须拒（D8（核心索引结构） 已定项 16 第 5 条，
+    /// [`ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile`]），一条根都不写；别的版本交 `None`，照各自的答法。
+    fn answer_refusing_a_change_on_a_version_with_trees_and_no_file(
+        &self,
+        operation: ModelOperationKind,
+    ) -> Option<ModelAnswer> {
+        let session = self.session.as_ref()?;
+        match session.current.contents {
+            ModelVersionContents::TreesAndNoFile { .. } => Some(self.answer_for_publishes(
+                operation,
+                BTreeSet::from([ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile]),
+                BTreeSet::new(),
+                Vec::new(),
+            )),
+            ModelVersionContents::WithoutTrees
+            | ModelVersionContents::LiveFile(_)
+            | ModelVersionContents::FileBeingDeleted { .. }
+            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => None,
+        }
+    }
+
     /// 模型的现行版本有没有删除中的文件（`deleted_inodes` 不空）：执行器拿它分一次会话交回的那几版里哪一版是走空的一步。
     #[must_use]
     pub fn current_version_has_a_file_being_deleted(&self) -> bool {
@@ -2021,9 +2058,8 @@ impl IdealModel {
             | ModelRefusalReason::RollbackTargetWithoutFile
             | ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem
             | ModelRefusalReason::FloorAboveCeiling
-            | ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread => {
-                false
-            }
+            | ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread
+            | ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile => false,
         }
     }
 
diff --git a/crates/singlefs-harness/src/model_comparison.rs b/crates/singlefs-harness/src/model_comparison.rs
index 7eada56..dd1002c 100644
--- a/crates/singlefs-harness/src/model_comparison.rs
+++ b/crates/singlefs-harness/src/model_comparison.rs
@@ -672,8 +672,8 @@ pub fn refusal_reason_of_publish_error(error: &PublishError) -> ObservedRefusalR
         // 模型里没有它们的理由。
         | PublishError::JournalRecordsOfThePublishExceedTheLimit { .. }
         | PublishError::NextCheckpointTxgPastTheTopOfItsRange { .. }
-        // 写删除中的 inode、写删完的 inode、unlink 不是活记录的号、deleted_inodes 装不下：执行器只在实现报带文件、队列空时
-        // 调覆盖写与 unlink（队列不空时那两种改动先走空、再拒成不带走空那几步的 `NoFileVersionToChange`，执行器不调），
+        // 写删除中的 inode、写删完的 inode、unlink 不是活记录的号、deleted_inodes 装不下：执行器只在实现报带文件时调覆盖写与 unlink，
+        // 队列不空时那两种改动先走空、再拒成带着走空那几步的 `NoFileVersionToChange`（代码轮 U10、U11），不报这几种；
         // 队列至多 inode 1 一个号，造不出这几格。
         | PublishError::WriteToAnInodeBeingDeleted { .. }
         | PublishError::WriteToAnInodeThatIsNotInTheTree { .. }
diff --git a/crates/singlefs-harness/tests/a_floor_raise_refused_for_space_counts_as_short_of_space.rs b/crates/singlefs-harness/tests/a_floor_raise_refused_for_space_counts_as_short_of_space.rs
index 795cb26..82735f9 100644
--- a/crates/singlefs-harness/tests/a_floor_raise_refused_for_space_counts_as_short_of_space.rs
+++ b/crates/singlefs-harness/tests/a_floor_raise_refused_for_space_counts_as_short_of_space.rs
@@ -160,3 +160,28 @@ fn a_mount_whose_floor_raise_after_the_row_publish_finds_no_slot_is_still_made_a
         "抬 F 那一串两次空发布在预演里第 1 次就取不到落点：{raise_refusal:?}"
     );
 }
+
+/// `deleted_inodes` 根装不下（队列多于一个码 2 根装得下的 2031 个号，D8（核心索引结构） 已定项 16 第 6 条「多于一个根装得下那一支拒成类型化错误、
+/// 不是空间不够」）那一种拒绝归「不是空间不够」那一组：会话与挂载续做都不为它推抬 F，原样交回（代码轮 U3）。
+/// 拒绝取自构造根条目的那个函数对 2032 个号真交回的错，不手拼。
+/// 判别力：把它归进空间不够那两种（推抬 F 再试）时这一判红。
+#[test]
+fn a_deleted_inodes_queue_longer_than_one_root_holds_is_not_short_of_space_and_pushes_no_floor_raise(
+) {
+    use singlefs_core::address::InodeNumber;
+    use singlefs_core::deleted_inodes::{
+        deleted_inodes_one_root_node_holds, deleted_inodes_root_entries,
+    };
+    let one_too_many: Vec<InodeNumber> =
+        (1..=u64::try_from(deleted_inodes_one_root_node_holds()).expect("一个根装得下的号数") + 1)
+            .map(InodeNumber)
+            .collect();
+    let refusal = deleted_inodes_root_entries(&one_too_many)
+        .expect_err("比一个根装得下的多一个号：拒成类型化错误");
+    assert!(
+        !singlefs_core::mounted_session::refusal_is_short_of_space(
+            &PublishError::DeletedInodesQueueRefused(refusal)
+        ),
+        "deleted_inodes 装不下不是空间不够：不推抬 F（{refusal:?}）"
+    );
+}
diff --git a/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs b/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs
index 03dac17..c80d48b 100644
--- a/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs
+++ b/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs
@@ -12,15 +12,15 @@
 mod common_admission;
 
 use common_admission::{
-    checker_violations_on, space_admission_of_a_crash_remount_on_a_copy,
+    checker_violations_on, plain_devices_on, space_admission_of_a_crash_remount_on_a_copy,
     start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
-    start_plain, DeviceOverASparseImage, PoolUnderTest, OVERWRITE_BYTES,
+    start_plain, DeviceOverASparseImage, PoolUnderTest, FIRST_WRITE_TIME_SECONDS, OVERWRITE_BYTES,
 };
 use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InodeNumber, InstanceGeneration, SlotNumber,
 };
 use singlefs_core::admission::SpaceAdmission;
-use singlefs_core::allocator::UnitAreaStart;
+use singlefs_core::allocator::{PlacementRefusal, UnitAreaStart};
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
 use singlefs_core::container_tree::ContainerTreeNodeEntries;
 use singlefs_core::lineage_tree::LineageContainerCapacity;
@@ -29,11 +29,11 @@ use singlefs_core::mount::{
     mount_writable_with_space_admission,
     mount_writable_with_test_only_switches_and_deletion_switches, raise_rollback_floor,
     raise_rollback_floor_to_the_admission_ceiling, ring_slots_known_to_hold_a_root_by,
-    rollback_floor_ceiling, unmount, FloorRaiseStop, MountError, MountSpaceAdmission, Mounted,
-    RaiseToTheAdmissionCeiling, ShadowLedger, Unmounted,
+    rollback_floor_ceiling, unmount, DeletionQueueContinuationOutcome, FloorRaiseStop, MountError,
+    MountSpaceAdmission, Mounted, RaiseToTheAdmissionCeiling, ShadowLedger, Unmounted,
 };
 use singlefs_core::mounted_read::{mount_read_only, OpenFileFailure};
-use singlefs_core::mounted_session::{UserChange, UserChangeRefused};
+use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
 use singlefs_core::recovery::{
     choose_system_configuration, instance_table_chain_of_root, BadRootRingSlotReading,
 };
@@ -41,8 +41,8 @@ use singlefs_core::root_record::RootRecord;
 use singlefs_core::root_ring::{slot_offset, RootRingSlot};
 use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
 use singlefs_core::transaction::{
-    publish_unlink_intent, publish_unlink_step, PoolVersion, PoolWriter, PublishError,
-    FIRST_INODE_NUMBER,
+    publish_unlink_intent, publish_unlink_step, FirstFile, PoolVersion, PoolWriter, PublishError,
+    TransactionUnit, FIRST_INODE_NUMBER,
 };
 use singlefs_core::unit::data_unit_payload_capacity;
 use singlefs_core::unlink::{
@@ -59,6 +59,8 @@ use singlefs_harness::history::{
     StepOutcome, StepPosition,
 };
 use singlefs_harness::memory_pool::{SparseBlockDevice, SparseDevice};
+use singlefs_harness::model::{ModelRefusalReason, ObservedRefusalReason};
+use singlefs_harness::model_comparison::refusal_reason_of_mount_error;
 use singlefs_harness::{RecordingBlockDevice, SharedStream};
 
 /// 会话里 unlink 之前覆盖写几次：换下的单元进 defer，等推的抬 F 回收（意图那次被拒之后靠它们腾出槽）。
@@ -427,25 +429,86 @@ fn a_mount_whose_deletion_continuation_finds_no_unblocked_slot_raises_the_floor_
     );
 }
 
-/// 推满仍不够（同一个定案的另一半：照旧交回错、已发成的留在盘上）：同一份窄盘造法，发完意图与第一批才崩，重开、准入判着、删除批大小 1。
-/// 续做里一批被落得下那一判拒，为它推的那一串推不出空间（一串都没推成，F 没动）：挂载交回 `MountError::Publish`、`cause` 是那一次
-/// 落得下那一判的拒绝；挂载写下的 13 次发布（写行、暖机与续做发成的几步）留在盘上：只读挂载走的是挂载最后落盘的那一版，
-/// 第一个文件仍在删除中（打开它答「没有文件」）。推不动的原因（预算、F 已在上限、抬 F 那一串自己落不下）照旧不在错里。
-/// 判别力：推满仍不够时把错换成别的成员、或把已落盘的几版退掉，那几项红。
+/// 推满仍放不下（用户 2026-10-03 定「照 D16 做成可写」：挂载照样做成可写、队列留着）：同一份窄盘造法，发完意图与第一批才崩，重开、准入判着、
+/// 删除批大小 1。续做里一批被落得下那一判拒，为它推的那一串推不出空间（一串都没推成，F 没动）：挂载照样交回可写，
+/// 续做停在 `StillShortAfterTheFloorRaises`、最后那一次被拒是落得下那一判，现行那一版的队列还留着第一个文件；
+/// 挂着时一次用户改动先走空、走不动报空间不够（一版都没发成）；正常卸载照常走抬 F 到现行那一版那一串，这块盘上它在预演里就取不到落点、
+/// 在任何写之前拒。挂载写下的 13 次发布（写行、暖机与续做发成的几步）留在盘上：只读挂载走的是挂载最后落盘的那一版，
+/// 第一个文件仍在删除中（打开它答「没有文件」）。
+/// 判别力：推满仍放不下照旧交回 `MountError::Publish`（这一条定案之前的做法）时挂载那一步就红。
 #[test]
-fn a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_hands_back_the_refusal_and_leaves_its_publishes_on_disk(
+fn a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_with_the_queue_left_and_its_publishes_on_disk(
 ) {
     let (mut pool, root_at_the_crash) = pool_crashed_between_deletion_batches_on_narrow_devices(1);
-    let failed = match mount_after_the_crash_with_batches_of_one(&mut pool) {
-        Err(MountError::Publish(failed)) => failed,
-        Err(other) => panic!("推满仍不够交回 MountError::Publish：{other:?}"),
-        Ok(_) => panic!("推满仍不够交回 MountError::Publish，挂载却做成了"),
+    let mounted = mount_after_the_crash_with_batches_of_one(&mut pool)
+        .unwrap_or_else(|error| panic!("推满仍放不下：挂载照样做成可写：{error:?}"));
+    let DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { last_refusal, .. } =
+        &mounted.output.deletion_queue_continuation_outcome
+    else {
+        panic!(
+            "续做推满仍放不下：{:?}",
+            mounted.output.deletion_queue_continuation_outcome
+        );
     };
     assert_eq!(
-        kind_of_the_refusal_that_pushed(&failed.cause),
+        kind_of_the_refusal_that_pushed(last_refusal),
         "SpaceAdmissionRefused",
-        "交回的是续做里那一批的落得下那一判：{:?}",
-        failed.cause
+        "最后那一次被拒是续做里那一批的落得下那一判：{last_refusal:?}"
+    );
+    assert_eq!(
+        mounted
+            .current
+            .version_of_trees()
+            .expect("续做走到的那一版有树")
+            .deleted_inodes,
+        vec![InodeNumber(FIRST_INODE_NUMBER)],
+        "队列留着第一个文件"
+    );
+    let image_after_the_mount = pool.image();
+    let (_, mut session) = MountedSession::of_the_mount(mounted);
+    let content = vec![9_u8; 100];
+    let change = session.publish_user_change(
+        &mut pool.devices,
+        UserChange::Overwrite(FirstFile {
+            content: &content,
+            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1,
+        }),
+    );
+    let Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) = &change else {
+        panic!("这次挂载的会话里一次用户改动先走空、走不动报空间不够：{change:?}");
+    };
+    assert_eq!(
+        (
+            kind_of_the_refusal_that_pushed(&no_space.last_refusal),
+            no_space.versions_published.len(),
+            no_space.floor_raises.len(),
+            matches!(
+                no_space.stop,
+                FloorRaiseStop::FloorRaiseRefusedForSpace { .. }
+            ),
+        ),
+        ("SpaceAdmissionRefused", 0, 0, true),
+        "走空的那一批照样被落得下那一判拒、为它推的那一串自己也落不下：一版都没发成，报空间不够"
+    );
+    // 正常卸载照常走它那一条路（抬 F 到现行那一版的一串空发布，D16（发布语义） 已定项 1「正常卸载」）：这块窄盘的槽被续做吃光了，
+    // 那一串在任何写之前的预演里就取不到落点（攻方 E8b 量到的那一形；派发的验收写「卸载成」，这块盘上做不到，交主 agent）。
+    let unmounted = unmount(
+        &pool.parameters,
+        &mut pool.devices,
+        &mut session.allocator,
+        &mut session.current,
+        session.shadow_ledger,
+    );
+    assert!(
+        matches!(
+            unmounted,
+            Err(MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. })
+        ),
+        "卸载那一串在预演里就取不到落点"
+    );
+    assert!(
+        pool.image() == image_after_the_mount,
+        "那一次用户改动与卸载都在任何写之前被拒：两块盘逐字节停在挂载之后"
     );
     let image = pool.image();
     let read_only = mount_read_only(&image).expect("只读挂载");
@@ -468,6 +531,80 @@ fn a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_hands
     );
 }
 
+/// 推满仍放不下、挂载照样做成可写之后进程退出，同一份窄盘再可写挂载（代码轮 U5，攻方 `m3-unlink-code-r1` A5 / E8c 那一形）：
+/// 上一次挂载把最后的空槽吃掉了，这一次取号之前在分配器的拷贝上预演写行那一串就取不到树表的落点，拒成
+/// `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`（要取的号 3、写行那次、暖机计划一次、树表、每块盘上都没有），
+/// 胶水把它映射成模型的单元区墙；一个写都不发：录制流 0 步、两块盘逐字节不变。
+/// 判别力：预演取不到落点也照样取号（变异行「取号之前的预演取不到落点也照样取号」）时挂载写了盘才被拒，录制流不是 0 步；
+/// 落点被拒的原因报成别的成员、或胶水把它映射成模型没有的理由，那几项红。
+#[test]
+fn after_a_mount_whose_continuation_stays_short_the_next_mount_is_refused_by_the_placement_before_acquisition_writing_nothing(
+) {
+    let (mut pool, _) = pool_crashed_between_deletion_batches_on_narrow_devices(1);
+    let first = mount_after_the_crash_with_batches_of_one(&mut pool)
+        .unwrap_or_else(|error| panic!("推满仍放不下：挂载照样做成可写：{error:?}"));
+    assert!(
+        matches!(
+            first.output.deletion_queue_continuation_outcome,
+            DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { .. }
+        ),
+        "第一次挂载续做推满仍放不下：{:?}",
+        first.output.deletion_queue_continuation_outcome
+    );
+    drop(first);
+    let image_before = pool.image();
+    let stream = SharedStream::new();
+    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
+        plain_devices_on(&image_before)
+            .into_iter()
+            .map(|(identity, device)| {
+                (
+                    identity,
+                    RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
+                )
+            })
+            .collect();
+    let refusal = mount_writable_with_test_only_switches_and_deletion_switches(
+        &pool.parameters,
+        &mut recorded,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        deletion_switches_with_batches_of_one(),
+    )
+    .expect_err("取号之前预演取不到落点：拒");
+    assert!(
+        matches!(
+            refusal,
+            MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
+                instance_to_acquire: InstanceGeneration(3),
+                publish_index: 0,
+                warm_up_publishes_planned: 1,
+                unit: TransactionUnit::TreeTable,
+                refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
+            }
+        ),
+        "取号之前在拷贝上预演写行那一串，树表在每块盘上都没有落点：{refusal:?}"
+    );
+    assert_eq!(
+        refusal_reason_of_mount_error(&refusal),
+        ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall),
+        "胶水把它映射成模型的单元区墙"
+    );
+    assert_eq!(
+        stream.operation_count(),
+        0,
+        "录制流 0 步：一个写、一道屏障都没发"
+    );
+    let image_after = singlefs_harness::memory_pool::MemoryPool {
+        devices: recorded
+            .iter()
+            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
+            .collect(),
+        device_size_in_bytes: image_before.device_size_in_bytes,
+    };
+    assert!(image_after == image_before, "两块盘逐字节不变");
+}
+
 /// 被拒的那一种（只分空间不够的两种；会话只因这两种推抬 F，`mounted_session::refusal_is_short_of_space`）。
 fn kind_of_the_refusal_that_pushed(refusal: &PublishError) -> &'static str {
     if matches!(refusal, PublishError::SpaceAdmissionRefused(_)) {
diff --git a/crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs b/crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs
index ebcd9ac..a815912 100644
--- a/crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs
+++ b/crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs
@@ -519,3 +519,300 @@ fn c393_checker_on_a_persistently_damaged_allocation_record_tree_without_reuse()
 fn c393_checker_on_a_persistently_damaged_tree_table_without_reuse() {
     checker_on_a_persistently_damaged_account_without_reuse(Account::TreeTable);
 }
+
+/// 抬 F 之前这次挂载里覆盖写几次：上限取「第 4 新的不同状态」（D16（发布语义） 已定项 1「抬 F 的上限」），覆盖写 5 次之后它是这次挂载里
+/// 第 2 次覆盖写那一版，越过了被抛弃根的前一版（3 次时上限还停在第一个文件 B 那一版的 txg 4，副本上实测）。
+const OVERWRITES_BEFORE_THE_RAISE: usize = 5;
+
+/// 这一版的池里根环上按 `readable_roots` 的次序列出的、按 (实例, txg) 认的那几条根里，`wanted` 各排第几（抬 F 的影子账按这个次序逐条算被抛弃根）。
+fn positions_in_the_ring_order(
+    pool: &mut BuiltPool,
+    wanted: &[(InstanceGeneration, CheckpointTxg)],
+) -> Vec<usize> {
+    let devices = pool.reopen_recorded();
+    let system_configuration =
+        singlefs_core::recovery::choose_system_configuration(&devices).expect("择系统配置");
+    let roots = singlefs_core::recovery::readable_roots(
+        &devices,
+        &system_configuration.immutable.region_devices,
+        &system_configuration.immutable.sizes,
+        &system_configuration.immutable.filesystem_identifier,
+    );
+    pool.devices = Some(devices);
+    wanted
+        .iter()
+        .map(|key| {
+            roots
+                .iter()
+                .position(|root| (root.instance, root.checkpoint_txg) == *key)
+                .expect("那条根在根环里读得出")
+        })
+        .collect()
+}
+
+/// 把一条根的树表与中央映射树根节点两份都抹零（盘上真坏，重读也读不出）：它的账与从映射现算都不成。
+fn zero_the_tree_table_and_the_mapping_root_of(
+    pool: &mut BuiltPool,
+    root: &singlefs_core::root_record::RootRecord,
+) {
+    use singlefs_core::block_device::{BlockDevice, WriteDurability};
+    let mut devices = pool.reopen_recorded();
+    for location in root
+        .tree_table
+        .locations
+        .iter()
+        .chain(root.mapping_root.locations.iter())
+    {
+        let index = devices
+            .iter()
+            .position(|(identity, _)| *identity == location.device)
+            .expect("池里有这块盘");
+        devices[index]
+            .1
+            .write_at(
+                location.slot.to_device_offset(),
+                &vec![0u8; usize::try_from(SLOT_BYTES).expect("槽宽")],
+                WriteDurability::Plain,
+            )
+            .expect("抹零");
+    }
+    pool.devices = Some(devices);
+}
+
+/// 抬 F 那一串在影子账上被拒（代码轮 LR3，攻方 `m3-leftover-code-r1` Z2 ③：此前没有用例走到这一支）：
+/// 崩溃恢复先后抛弃 C (2, 8) 与 E（实例 3 覆盖写 D 之后的那一版），再可写挂载一次、覆盖写 [`OVERWRITES_BEFORE_THE_RAISE`] 次，然后把两条被抛弃根里影子账排在后面的那一条的
+/// 树表与中央映射树根两份都抹零。抬 F 到上限：新 F 越过两条被抛弃根的前一版，排在前面那条被抛弃根引用的槽（前一版那张实例表这类）
+/// 此刻只被它引用，影子账先给它补上隔离；算到后面那条，账与从映射现算都不成、重读一次仍不成，`isolate_slots_referenced_only_by_abandoned_roots`
+/// 交 `Err`——抬 F 拒成 `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`、点名那条根，一个写都不发，
+/// 分配器整个换回抬 F 之前那一份（补上的隔离撤掉）。
+/// 判别力：影子账交 `Err` 时不换回分配器，前一条补上的隔离留在分配器上，「与抬 F 之前逐项相同」那一项红。
+#[test]
+fn a_floor_raise_whose_shadow_ledger_meets_an_abandoned_root_unreadable_by_account_and_mapping_is_refused_and_restores_the_allocator(
+) {
+    let (mut pool, abandoned_c) =
+        pool_after_a_recovery_abandoned_c("lr3-raise-refused-on-the-shadow-ledger");
+    let before_d = pool.output.clone();
+    pool.output = publish_overwrite_in_process(
+        &mut pool,
+        &before_d,
+        &content_of(3000, 21),
+        FIXED_WRITE_TIME_SECONDS + 120,
+        InstanceGeneration(3),
+    )
+    .expect("D");
+    let before_e = pool.output.clone();
+    let abandoned_e = publish_overwrite_in_process(
+        &mut pool,
+        &before_e,
+        &content_of(2200, 23),
+        FIXED_WRITE_TIME_SECONDS + 120,
+        InstanceGeneration(3),
+    )
+    .expect("E");
+    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &abandoned_e);
+    let mut devices_of_the_mount = pool.reopen_recorded();
+    let mounted = singlefs_core::mount::mount_writable(&parameters(), &mut devices_of_the_mount)
+        .expect("两条被抛弃根都读得出：照常可写挂载");
+    let instance = mounted.output.instance;
+    pool.devices = Some(devices_of_the_mount);
+    pool.allocator = mounted.allocator;
+    pool.output = mounted.current.into_file_version().expect("带文件");
+    for overwrite_index in 0..OVERWRITES_BEFORE_THE_RAISE {
+        let previous = pool.output.clone();
+        pool.output = publish_overwrite_in_process(
+            &mut pool,
+            &previous,
+            &content_of(1800 + overwrite_index, overwrite_index + 31),
+            FIXED_WRITE_TIME_SECONDS + 600,
+            instance,
+        )
+        .expect("这次挂载里覆盖写");
+    }
+    let keys = [
+        (abandoned_c.root.instance, abandoned_c.root.checkpoint_txg),
+        (abandoned_e.root.instance, abandoned_e.root.checkpoint_txg),
+    ];
+    let positions = positions_in_the_ring_order(&mut pool, &keys);
+    let later = if positions[0] > positions[1] {
+        &abandoned_c
+    } else {
+        &abandoned_e
+    };
+    zero_the_tree_table_and_the_mapping_root_of(&mut pool, &later.root);
+    let allocator_before_the_raise = format!("{:?}", pool.allocator);
+    let image_before_the_raise = pool.memory_pool();
+    let mut devices_of_the_raise = pool.reopen_recorded();
+    let refusal = singlefs_core::mount::raise_rollback_floor_to_the_admission_ceiling(
+        &parameters(),
+        &mut devices_of_the_raise,
+        &mut pool.allocator,
+        &mut pool.output,
+        ShadowLedger::On,
+    )
+    .expect_err("影子账算到账与映射都读不出的被抛弃根：拒这一次抬");
+    pool.devices = Some(devices_of_the_raise);
+    assert!(
+        matches!(
+            &refusal,
+            singlefs_core::mount::MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
+                if matches!(
+                    still_unreadable.as_ref(),
+                    singlefs_core::mount::StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { abandoned_root }
+                        if (abandoned_root.instance, abandoned_root.checkpoint_txg)
+                            == (later.root.instance, later.root.checkpoint_txg)
+                )
+        ),
+        "拒因点名影子账排在后面的那条被抛弃根：{refusal:?}"
+    );
+    assert_eq!(
+        format!("{:?}", pool.allocator),
+        allocator_before_the_raise,
+        "分配器与抬 F 之前逐项相同：前一条被抛弃根补上的隔离撤掉了"
+    );
+    assert!(
+        pool.memory_pool() == image_before_the_raise,
+        "影子账在任何写之前拒：两块盘逐字节不变"
+    );
+}
+
+/// 被抛弃根的实例表有两片时，账读不出、从中央映射现算的那一臂把两片都算进它引用的落点（代码轮 LR2，攻方 `m3-leftover-code-r1` Z4 ①
+/// 「现算只认实例表第 0 片」放过：用例的池都只有一片实例表）。
+/// 造法：A、B（实例 1）之后连取 370 次号再可写挂载（实例 372），写行把 1 到 371 的 371 行写进两片实例表；覆盖写 C、崩溃恢复抛弃 C
+/// （实例 373 写行，换下两片）；C 的树表两份抹零（账读不出）再可写挂载（实例 374）、覆盖写 [`OVERWRITES_BEFORE_THE_RAISE`] 次、抬 F 到上限：新 F 越过 C 的前一版，
+/// C 那两片实例表此刻只被 C 引用、在现行那一版的账里已释放，抬 F 的影子账从映射现算、两片都隔离，回收之后照样不空闲。
+/// 判别力：现算只认实例表第 0 片时第 1 片不隔离、回收之后空闲，那一项红。
+#[test]
+fn an_abandoned_root_with_a_two_page_instance_table_and_an_unreadable_account_isolates_both_pages_from_the_mapping(
+) {
+    let mut pool = build_pool("lr2-two-page-instance-table-recomputed-from-the-mapping");
+    let first = pool.output.clone();
+    pool.output = publish_overwrite_in_process(
+        &mut pool,
+        &first,
+        &content_of(4100, 3),
+        FIXED_WRITE_TIME_SECONDS + 60,
+        InstanceGeneration(1),
+    )
+    .expect("B");
+    {
+        let publish_parameters = parameters();
+        let devices = pool.devices.as_mut().expect("镜像还开着");
+        let mut writer = singlefs_core::transaction::PoolWriter::new(
+            &publish_parameters,
+            devices.as_mut_slice(),
+        );
+        // 迭代上界 370：取号只写系统配置两槽，下一次可写挂载为每个烧掉的号写一行。
+        for _ in 0..370 {
+            singlefs_core::transaction::acquire_instance(&mut writer).expect("取号");
+        }
+    }
+    let mut devices_after_the_acquisitions = pool.reopen_recorded();
+    let mounted_after_the_acquisitions =
+        singlefs_core::mount::mount_writable(&parameters(), &mut devices_after_the_acquisitions)
+            .expect("连取 370 次号之后可写挂载");
+    let instance_of_c = mounted_after_the_acquisitions.output.instance;
+    pool.devices = Some(devices_after_the_acquisitions);
+    pool.allocator = mounted_after_the_acquisitions.allocator;
+    pool.output = mounted_after_the_acquisitions
+        .current
+        .into_file_version()
+        .expect("带文件");
+    let before_c = pool.output.clone();
+    let abandoned_c = publish_overwrite_in_process(
+        &mut pool,
+        &before_c,
+        &content_of(2500, 11),
+        FIXED_WRITE_TIME_SECONDS + 60,
+        instance_of_c,
+    )
+    .expect("C");
+    let pages_of_c: Vec<singlefs_core::pointer::NodePointer> = {
+        let devices_for_the_page_pointers = pool.reopen_recorded();
+        let pages = singlefs_core::recovery::instance_table_page_pointers_as_far_as_readable(
+            &devices_for_the_page_pointers,
+            &abandoned_c.root,
+        );
+        pool.devices = Some(devices_for_the_page_pointers);
+        pages
+    };
+    assert_eq!(
+        pages_of_c.len(),
+        2,
+        "C 的实例表两片（371 行，一片装 369 行）"
+    );
+    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &abandoned_c);
+    {
+        use singlefs_core::block_device::{BlockDevice, WriteDurability};
+        let mut devices_to_zero_the_tree_table = pool.reopen_recorded();
+        let tree_table_of_c = abandoned_c.unit(TransactionUnit::TreeTable);
+        for (_, device) in &mut devices_to_zero_the_tree_table {
+            device
+                .write_at(
+                    DeviceOffsetInBytes(tree_table_of_c.slot.0 * SLOT_BYTES),
+                    &vec![0u8; tree_table_of_c.bytes.len()],
+                    WriteDurability::Plain,
+                )
+                .expect("C 的树表两份抹零");
+        }
+        pool.devices = Some(devices_to_zero_the_tree_table);
+    }
+    let mut devices_of_the_remount = pool.reopen_recorded();
+    let remounted =
+        singlefs_core::mount::mount_writable(&parameters(), &mut devices_of_the_remount)
+            .expect("C 的账读不出、从映射现算得成：照常可写挂载");
+    assert_eq!(
+        remounted.output.abandoned_roots_unreadable, 1,
+        "C 的账读不出，计数往上报"
+    );
+    let instance = remounted.output.instance;
+    pool.devices = Some(devices_of_the_remount);
+    pool.allocator = remounted.allocator;
+    pool.output = remounted.current.into_file_version().expect("带文件");
+    for overwrite_index in 0..OVERWRITES_BEFORE_THE_RAISE {
+        let previous = pool.output.clone();
+        pool.output = publish_overwrite_in_process(
+            &mut pool,
+            &previous,
+            &content_of(1800 + overwrite_index, overwrite_index + 31),
+            FIXED_WRITE_TIME_SECONDS + 600,
+            instance,
+        )
+        .expect("这次挂载里覆盖写");
+    }
+    let mut devices_of_the_raise = pool.reopen_recorded();
+    let raised = singlefs_core::mount::raise_rollback_floor_to_the_admission_ceiling(
+        &parameters(),
+        &mut devices_of_the_raise,
+        &mut pool.allocator,
+        &mut pool.output,
+        ShadowLedger::On,
+    )
+    .expect("抬 F 到上限");
+    pool.devices = Some(devices_of_the_raise);
+    assert!(
+        matches!(
+            &raised,
+            singlefs_core::mount::RaiseToTheAdmissionCeiling::Raised(raised)
+                if raised.publishes.first().is_some_and(|publish| publish.root.rollback_floor >= abandoned_c.root.checkpoint_txg)
+        ),
+        "新 F 越过 C 的前一版：{raised:?}"
+    );
+    let free_page_slots: Vec<(u32, u64)> = pages_of_c
+        .iter()
+        .flat_map(|page| page.locations)
+        .filter(|location| {
+            pool.allocator
+                .devices
+                .iter()
+                .find(|device_map| device_map.device == location.device)
+                .expect("池里有这块盘")
+                .is_free(location.slot)
+        })
+        .map(|location| (location.device.0, location.slot.0))
+        .collect();
+    assert_eq!(
+        free_page_slots,
+        Vec::<(u32, u64)>::new(),
+        "C 那两片实例表的落点抬 F 回收之后一槽都不空闲（从映射现算两片都隔离）"
+    );
+}
diff --git a/crates/singlefs-harness/tests/common_admission_pool/mod.rs b/crates/singlefs-harness/tests/common_admission_pool/mod.rs
index 19c90f8..123acfc 100644
--- a/crates/singlefs-harness/tests/common_admission_pool/mod.rs
+++ b/crates/singlefs-harness/tests/common_admission_pool/mod.rs
@@ -300,9 +300,11 @@ impl<Device: DeviceOverASparseImage> PoolUnderTest<Device> {
             Err(UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(failed)) => {
                 self.note_floor_raises(&failed.floor_raises);
             }
+            Err(UserChangeRefused::NoFileVersionToChange { floor_raises, .. }) => {
+                self.note_floor_raises(floor_raises);
+            }
             Err(
                 UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
-                | UserChangeRefused::NoFileVersionToChange
                 | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
                 | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
                 | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
diff --git a/crates/singlefs-harness/tests/unlink_in_histories_judged_by_the_model.rs b/crates/singlefs-harness/tests/unlink_in_histories_judged_by_the_model.rs
index 4af662b..2921c12 100644
--- a/crates/singlefs-harness/tests/unlink_in_histories_judged_by_the_model.rs
+++ b/crates/singlefs-harness/tests/unlink_in_histories_judged_by_the_model.rs
@@ -326,3 +326,107 @@ fn the_fast_tier_weights_draw_an_inode_creation_after_an_unlink_and_the_first_hi
         run.outcomes
     );
 }
+
+/// 回退到意图那一版（队列带回第一个文件）之后的一步（攻方 `m3-unlink-code-r1` A3 那一形）：unlink 第一个文件（意图 + 最后一批）、
+/// 删完之后建一个 inode（实现把这一版归带文件，执行器才调回退）、挂着时回退到从新到旧第 3 条根（意图那一版），再走 `then`。执行器不再跳过删除中的版本（代码轮 U11）：会话先走空、交回走空发成的那一版，
+/// 模型照「走空的一步」比，再比这次改动本身。交回整段的结局与第 4 步的结局。
+fn history_after_rolling_back_to_the_intent_then(
+    then: HistoryOperation,
+) -> (
+    HistoryEnding,
+    StepOutcome,
+    singlefs_harness::model::ModelJudgementCounts,
+) {
+    let history = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations: vec![
+            HistoryOperation::Unlink,
+            HistoryOperation::CreateInode,
+            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
+                index_from_newest: 2,
+            }),
+            then,
+        ],
+    };
+    let run = execute_history_with(
+        &history,
+        HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+        &SharedStream::new(),
+        &mut |_| {},
+    );
+    assert!(
+        matches!(
+            run.outcomes[2],
+            StepOutcome::Applied(AppliedEffect::RolledBack { .. })
+        ),
+        "回退到意图那一版做成：{:?}",
+        run.outcomes
+    );
+    (
+        run.ending,
+        run.outcomes[3].clone(),
+        run.tally.model_counts.clone(),
+    )
+}
+
+/// 回退到意图那一版之后覆盖写：会话先走空（最后一批落盘）、再拒成 `NoFileVersionToChange`，模型比过走空那一版（做成一次）与这次该拒（一次），
+/// 每一步之后池级 checker 绿、整段跑完。判别力：执行器在删除中的版本上跳过覆盖写时第 4 步是「前提不满足」，这一项红。
+#[test]
+fn an_overwrite_after_rolling_back_to_the_intent_is_judged_by_the_model_through_the_walked_step() {
+    let (ending, outcome, counts) = history_after_rolling_back_to_the_intent_then(
+        HistoryOperation::PublishOverwrite(ContentChoice {
+            length: ContentLength::InsideOneDataUnit { selector: 2999 },
+            fill_seed: 5,
+        }),
+    );
+    assert_eq!(ending, HistoryEnding::Completed);
+    assert_eq!(
+        outcome,
+        StepOutcome::Refused {
+            member: "UserChangeRefused::NoFileVersionToChange".to_string()
+        },
+        "走空之后拒成没有可改的文件"
+    );
+    assert!(
+        counts.required_refusals_matched >= 1,
+        "模型判过这一次该拒：{counts:?}"
+    );
+}
+
+/// 回退到意图那一版之后再 unlink：同上，会话先走空、再拒成 `NoFileVersionToChange`，模型逐版比、整段跑完。
+/// 判别力：执行器在删除中的版本上跳过 unlink 时第 4 步是「前提不满足」，这一项红。
+#[test]
+fn an_unlink_after_rolling_back_to_the_intent_is_judged_by_the_model_through_the_walked_step() {
+    let (ending, outcome, counts) =
+        history_after_rolling_back_to_the_intent_then(HistoryOperation::Unlink);
+    assert_eq!(ending, HistoryEnding::Completed);
+    assert_eq!(
+        outcome,
+        StepOutcome::Refused {
+            member: "UserChangeRefused::NoFileVersionToChange".to_string()
+        },
+        "走空之后拒成没有可改的文件"
+    );
+    assert!(
+        counts.required_refusals_matched >= 1,
+        "模型判过这一次该拒：{counts:?}"
+    );
+}
+
+/// 回退到意图那一版之后建 inode：会话先走空、再建，做成；模型比过走空那一版与建的那一版，整段跑完。
+/// 判别力：执行器在删除中的版本上跳过建 inode 时第 4 步是「前提不满足」，这一项红。
+#[test]
+fn creating_an_inode_after_rolling_back_to_the_intent_walks_the_queue_first_and_is_judged_by_the_model(
+) {
+    let (ending, outcome, _counts) =
+        history_after_rolling_back_to_the_intent_then(HistoryOperation::CreateInode);
+    assert_eq!(ending, HistoryEnding::Completed);
+    assert!(
+        matches!(
+            outcome,
+            StepOutcome::Applied(AppliedEffect::Published { .. })
+        ),
+        "走空之后建 inode 做成：{outcome:?}"
+    );
+}
diff --git a/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs b/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
index 795973b..cdb9078 100644
--- a/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
+++ b/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
@@ -309,8 +309,17 @@ fn after_the_unlink_the_session_holds_the_emptied_version_and_refuses_an_overwri
         )
         .expect_err("删空的一版上只放建 inode");
     assert!(
-        matches!(refusal, UserChangeRefused::NoFileVersionToChange),
-        "拒成没有可改的文件：{refusal:?}"
+        matches!(
+            &refusal,
+            UserChangeRefused::NoFileVersionToChange {
+                floor_raises,
+                refusals_that_pushed_the_floor_raises,
+                versions_published,
+            } if floor_raises.is_empty()
+                && refusals_that_pushed_the_floor_raises.is_empty()
+                && versions_published.is_empty()
+        ),
+        "拒成没有可改的文件，队列本来就空、没有走空：发成的几版与推的抬 F 都是空的：{refusal:?}"
     );
     assert_eq!(
         disk_snapshot(&pool.memory_pool(), &pool.stream),
@@ -483,3 +492,246 @@ fn rolling_back_to_the_intent_takes_its_deleted_inodes_root_and_the_next_mount_c
         "续做之后池级 checker 全绿"
     );
 }
+
+/// 一块内存盘外面再包一层：整池数写调用（两块盘共用一个计数），数到 `failing_write` 那一次报块设备错、一个字节都不落盘，别的原样交给内层。
+/// 造「续做里的一步在落盘途中失败」那一形（不是空间不够的错，代码轮 U9）。
+struct DeviceFailingOneCountedWrite<Inner> {
+    inner: Inner,
+    writes_so_far: std::rc::Rc<std::cell::Cell<u64>>,
+    failing_write: u64,
+}
+
+impl<Inner: singlefs_core::block_device::BlockDevice> singlefs_core::block_device::BlockDevice
+    for DeviceFailingOneCountedWrite<Inner>
+{
+    fn read_at(
+        &self,
+        offset: singlefs_core::address::DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
+        self.inner.read_at(offset, buffer)
+    }
+    fn write_at(
+        &mut self,
+        offset: singlefs_core::address::DeviceOffsetInBytes,
+        bytes: &[u8],
+        durability: singlefs_core::block_device::WriteDurability,
+    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
+        let this_write = self.writes_so_far.get() + 1;
+        self.writes_so_far.set(this_write);
+        if this_write == self.failing_write {
+            return Err(singlefs_core::block_device::BlockDeviceError::InputOutput(
+                std::io::Error::other("用例注入的写错"),
+            ));
+        }
+        self.inner.write_at(offset, bytes, durability)
+    }
+    fn write_zeroes_at(
+        &mut self,
+        offset: singlefs_core::address::DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
+        self.inner.write_zeroes_at(offset, length)
+    }
+    fn barrier(&mut self) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
+        self.inner.barrier()
+    }
+    fn probe_physical_block_size(&self) -> singlefs_core::block_device::PhysicalBlockSizeInBytes {
+        self.inner.probe_physical_block_size()
+    }
+    fn size_in_bytes(&self) -> u64 {
+        self.inner.size_in_bytes()
+    }
+}
+
+/// 续做里的一步没发成、不是空间不够（块设备错）：挂载交回 `MountError::Publish`，错里的已落盘写账（`writes_of_persisted_publishes`）
+/// 一次发布一份、按先后，份数等于这次挂载落在盘上的发布数（写行、暖机、续做第一步），每一份都有写（代码轮 U9，
+/// `PublishSequenceFailed::writes_of_persisted_publishes` 的契约）。
+/// 造法：三个单元的文件发完意图就崩（续做三步），同一份镜像先在一份拷贝上挂一次数清写调用，再在另一份上让续做第二步的第一次写报错。
+/// 判别力：续做那一处的错照旧交空的已落盘写账时，份数是 0、盘上落了 1 + 暖机 + 1 次，那一项红。
+#[test]
+fn a_mount_whose_continuation_step_fails_on_a_device_error_hands_back_the_writes_of_every_publish_it_landed(
+) {
+    let mut pool = pool_with_a_three_unit_file("continuation-device-error-hands-back-the-writes");
+    let intent = publish_the_intent_in_process(&mut pool);
+    let image = pool.memory_pool();
+    let counting_stream = SharedStream::new();
+    let mut counting_devices = crash_state_devices(&image, &[], &[], &counting_stream);
+    let counted = mount_writable_with_test_only_switches_and_deletion_switches(
+        &parameters(),
+        &mut counting_devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        deletion_switches_with_batches_of_one(),
+    )
+    .expect("拷贝上的那一次挂载续做走完");
+    let continuation_writes: Vec<u64> = counted
+        .output
+        .deletion_queue_continuation
+        .iter()
+        .map(|step| step.writes.total().write_calls)
+        .collect();
+    assert_eq!(continuation_writes.len(), 3, "续做三步");
+    let writes_of_the_whole_mount = u64::try_from(
+        counting_stream
+            .operations()
+            .iter()
+            .filter(|operation| operation.kind != singlefs_harness::RecordedOperationKind::Barrier)
+            .count(),
+    )
+    .expect("写数装得进 u64");
+    let writes_before_the_second_step =
+        writes_of_the_whole_mount - continuation_writes[1] - continuation_writes[2];
+    let warm_up_publishes = counted.output.warm_up_publishes.len();
+    drop(counted);
+    let writes_so_far = std::rc::Rc::new(std::cell::Cell::new(0));
+    let mut failing_devices: Vec<(
+        singlefs_core::address::DeviceIdentity,
+        DeviceFailingOneCountedWrite<_>,
+    )> = crash_state_devices(&image, &[], &[], &SharedStream::new())
+        .into_iter()
+        .map(|(identity, device)| {
+            (
+                identity,
+                DeviceFailingOneCountedWrite {
+                    inner: device,
+                    writes_so_far: writes_so_far.clone(),
+                    failing_write: writes_before_the_second_step + 1,
+                },
+            )
+        })
+        .collect();
+    let failed = match mount_writable_with_test_only_switches_and_deletion_switches(
+        &parameters(),
+        &mut failing_devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        deletion_switches_with_batches_of_one(),
+    ) {
+        Err(singlefs_core::mount::MountError::Publish(failed)) => failed,
+        Err(other) => panic!("续做那一步的块设备错交回 MountError::Publish：{other:?}"),
+        Ok(_) => panic!("续做第二步的写报错，挂载却做成了"),
+    };
+    assert!(
+        matches!(
+            failed.cause,
+            singlefs_core::transaction::PublishError::BlockDevice(_)
+        ),
+        "错是那一次写的块设备错：{:?}",
+        failed.cause
+    );
+    let image_after = MemoryPool {
+        devices: failing_devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.inner.wrapped_device().image.clone()))
+            .collect(),
+        device_size_in_bytes: common::IMAGE_BYTES,
+    };
+    let publishes_landed = mount_read_only(&image_after)
+        .expect("只读挂载")
+        .effective_root
+        .checkpoint_txg
+        .0
+        - intent.root.checkpoint_txg.0;
+    assert_eq!(
+        (
+            failed.writes_of_persisted_publishes.len(),
+            u64::try_from(failed.writes_of_persisted_publishes.len()).expect("份数"),
+        ),
+        (1 + warm_up_publishes + 1, publishes_landed,),
+        "错里的已落盘写账一次发布一份：写行、暖机、续做第一步，份数等于盘上落了几次发布"
+    );
+    assert!(
+        failed
+            .writes_of_persisted_publishes
+            .iter()
+            .all(|writes| writes.total().write_calls > 0),
+        "每一份都有写：{:?}",
+        failed.writes_of_persisted_publishes
+    );
+}
+
+/// 回退到意图那一版（队列带回第一个文件）之后经会话覆盖写（攻方 `m3-unlink-code-r1` A2 那一形）：会话先走空、落盘三步（留 2、留 1、最后一批），
+/// 现行那一版成了有树而文件集合为空的一版，再拒成 `NoFileVersionToChange`——拒绝里带着走空发成的三版（按 txg 接在回退那一版之后），
+/// 一串抬 F 都没推；盘上从回退那一版起正好落了三次（代码轮 U10）。
+/// 判别力：这个成员照旧不带发成的几版时交回的是空表，那一项红。
+#[test]
+fn an_overwrite_after_rolling_back_to_the_intent_walks_the_queue_and_its_refusal_carries_every_version_it_published(
+) {
+    let mut pool = pool_with_a_three_unit_file("overwrite-after-rolling-back-to-the-intent");
+    let (mounted, mut devices) = remount_with_batches_of_one(&mut pool);
+    let (_, mut session) = MountedSession::of_the_mount(mounted);
+    let unlinked = session
+        .publish_user_change(&mut devices, UserChange::Unlink { inode: FIRST_FILE })
+        .expect("会话 unlink");
+    let intent_root = unlinked.versions_published[0].root;
+    let mut current = session
+        .current
+        .version_of_trees()
+        .expect("删完之后有树")
+        .clone();
+    roll_back_by_a_forward_publish(
+        &parameters(),
+        &mut devices,
+        &mut session.allocator,
+        &mut current,
+        RollbackTarget {
+            instance: intent_root.instance,
+            checkpoint_txg: intent_root.checkpoint_txg,
+        },
+    )
+    .expect("挂着时回退到意图那一版");
+    let rollback_txg = current.root.checkpoint_txg.0;
+    assert_eq!(current.deleted_inodes, vec![FIRST_FILE], "回退带回队列");
+    session.current = PoolVersion::of_a_version_of_trees(current);
+    let content = vec![7_u8; 100];
+    let refusal = session
+        .publish_user_change(
+            &mut devices,
+            UserChange::Overwrite(FirstFile {
+                content: &content,
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
+            }),
+        )
+        .expect_err("走空之后没有可改的文件");
+    let UserChangeRefused::NoFileVersionToChange {
+        floor_raises,
+        refusals_that_pushed_the_floor_raises,
+        versions_published,
+    } = &refusal
+    else {
+        panic!("拒成没有可改的文件：{refusal:?}");
+    };
+    assert_eq!(
+        (
+            floor_raises.len(),
+            refusals_that_pushed_the_floor_raises.len(),
+            versions_published
+                .iter()
+                .map(|version| (
+                    version.root.checkpoint_txg.0 - rollback_txg,
+                    version.data_pointers.len(),
+                    version.deleted_inodes.len(),
+                ))
+                .collect::<Vec<_>>(),
+        ),
+        (0, 0, vec![(1, 2, 1), (2, 1, 1), (3, 0, 0)]),
+        "拒绝里带着走空发成的三版（留 2、留 1、最后一批），一串抬 F 都没推"
+    );
+    assert_eq!(
+        member_name(&session.current),
+        "WithTreesAndNoFile",
+        "会话的现行那一版是走空之后的那一版"
+    );
+    drop(devices);
+    assert_eq!(
+        mount_read_only(&pool.memory_pool())
+            .expect("只读挂载")
+            .effective_root
+            .checkpoint_txg
+            .0
+            - rollback_txg,
+        3,
+        "盘上从回退那一版起正好落了三次发布，与拒绝里带的版数相等"
+    );
+}
diff --git a/crates/singlefs-harness/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs b/crates/singlefs-harness/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs
new file mode 100644
index 0000000..e12e7d9
--- /dev/null
+++ b/crates/singlefs-harness/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs
@@ -0,0 +1,151 @@
+//! 真发布路径上、按产品批大小（`UnlinkBatchSize::OneExtentLowerLeaf`，一批 144 个数据单元）删一个罩着两片 extent 下段叶的文件
+//! （D8（核心索引结构） 已定项 16 第 2 条「从尾部分批」；代码轮 U2：入库的删除用例此前只删 3 个单元的文件、批大小 1，只改尾叶那一支没有真发布路径上的用例）。
+//! 287 个数据单元：第一片叶罩单元 0–143、第二片罩 144–286；第一批从尾部删掉 144 个（143–286），留下 143 个——第一片叶的末个单元（143）
+//! 正是被删掉的第一个单元，这片叶不能整片照抄、要重写（`unlink::plan_the_extent_tree_after_a_deletion_batch` 的 `is_entirely_kept`）；
+//! 第二批删完。逐版判：从盘上重建出来的那一版与发布的那一版（inode 记录、队列、数据指针、extent 树、进映射的单元）逐项相同，
+//! 池级 checker 无红；删完之后冷走读答「没有文件」。草稿出处：正推腿探针 `research/prompts/m3-unlink-code-r1-sonnet-model/sonnet_probe_multi_leaf_unlink.rs`。
+
+mod common;
+
+use common::{
+    build_pool, parameters, sequential_write_in_process_as_instance_one_and_advance_the_pool,
+};
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{InodeNumber, InstanceGeneration};
+use singlefs_core::recovery::{
+    rebuild_version, recover, JournalPolicy, RebuiltVersion, RecoveryOutcome,
+};
+use singlefs_core::transaction::{
+    publish_unlink_intent, publish_unlink_step, PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
+};
+use singlefs_core::unit::data_unit_payload_capacity;
+use singlefs_core::unlink::UnlinkBatchSize;
+use singlefs_harness::memory_pool::MemoryPool;
+
+const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);
+
+/// 新池新建文件那个实例：意图与各批都在这个进程里发。
+const INSTANCE_OF_THE_PROCESS: InstanceGeneration = InstanceGeneration(1);
+
+/// 文件的数据单元数：两片下段叶（一片罩 144 个），第一批之后留下的个数（143）比第一片叶罩的少一个。
+const DATA_UNITS_OF_THE_FILE: usize = 287;
+
+fn violations(image: &MemoryPool) -> Vec<String> {
+    check_pool_image(image)
+        .into_iter()
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
+            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+        })
+        .collect()
+}
+
+/// 这一版从盘上重建出来的那几样与发布的那一版逐项相同。
+fn assert_rebuilt_from_the_disk_equals_the_published(
+    image: &MemoryPool,
+    version: &TransactionOutput,
+) {
+    let rebuilt = rebuild_version(image, &version.root, Some(version.record.clone()))
+        .unwrap_or_else(|failure| {
+            panic!(
+                "txg {:?} 从盘上重建：{failure:?}",
+                version.root.checkpoint_txg
+            )
+        });
+    let (RebuiltVersion::WithFile(rebuilt) | RebuiltVersion::WithTreesAndNoFile(rebuilt)) = rebuilt
+    else {
+        panic!("txg {:?} 有树", version.root.checkpoint_txg);
+    };
+    assert_eq!(
+        (
+            &rebuilt.inode_record,
+            &rebuilt.deleted_inodes,
+            &rebuilt.data_pointers,
+            &rebuilt.extent_tree,
+            &rebuilt.mapped_units,
+        ),
+        (
+            &version.inode_record,
+            &version.deleted_inodes,
+            &version.data_pointers,
+            &version.extent_tree,
+            &version.mapped_units,
+        ),
+        "txg {:?}：从盘上重建出来的与发布的逐项相同",
+        version.root.checkpoint_txg
+    );
+}
+
+/// 判别力：`is_entirely_kept` 把「末个单元等于留下的个数」也算整片留下（`<` 改 `<=`）时，第一批之后那一版的第一片叶照抄、
+/// 还指着被删掉的单元 143，重建或池级 checker 那一项红。
+#[test]
+fn a_file_over_two_extent_lower_leaves_unlinked_in_product_batches_rewrites_the_leaf_whose_last_unit_is_cut(
+) {
+    let payload_capacity = data_unit_payload_capacity();
+    let content: Vec<u8> = (0..(DATA_UNITS_OF_THE_FILE - 1) * payload_capacity
+        + payload_capacity / 2)
+        .map(|index| u8::try_from((index * 31 + 7) % 251).expect("小于 256"))
+        .collect();
+    let mut pool = build_pool("unlink-two-extent-lower-leaves-in-product-batches");
+    let grown =
+        sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
+    assert_eq!(grown.data_pointers.len(), DATA_UNITS_OF_THE_FILE);
+    let publish_parameters = parameters();
+    let mut published: Vec<TransactionOutput> = Vec::new();
+    {
+        let previous = pool.output.clone();
+        let devices = pool.devices.as_mut().expect("镜像还开着");
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        pool.output = publish_unlink_intent(
+            &mut writer,
+            &mut pool.allocator,
+            &previous,
+            FIRST_FILE,
+            INSTANCE_OF_THE_PROCESS,
+        )
+        .expect("意图");
+    }
+    published.push(pool.output.clone());
+    assert_rebuilt_from_the_disk_equals_the_published(&pool.memory_pool(), &pool.output);
+    // 迭代上界：每一步至少删一个单元，文件 287 个单元。
+    while !pool.output.deleted_inodes.is_empty() {
+        assert!(published.len() <= DATA_UNITS_OF_THE_FILE, "删除步数有界");
+        let previous = pool.output.clone();
+        let devices = pool.devices.as_mut().expect("镜像还开着");
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        pool.output = publish_unlink_step(
+            &mut writer,
+            &mut pool.allocator,
+            &previous,
+            FIRST_FILE,
+            UnlinkBatchSize::OneExtentLowerLeaf,
+            INSTANCE_OF_THE_PROCESS,
+        )
+        .expect("删除的下一步");
+        published.push(pool.output.clone());
+        let image = pool.memory_pool();
+        assert_rebuilt_from_the_disk_equals_the_published(&image, &pool.output);
+        assert_eq!(
+            violations(&image),
+            Vec::<String>::new(),
+            "txg {:?} 之后池级 checker 无红",
+            pool.output.root.checkpoint_txg
+        );
+    }
+    assert_eq!(
+        published
+            .iter()
+            .map(|version| version.data_pointers.len())
+            .collect::<Vec<_>>(),
+        vec![DATA_UNITS_OF_THE_FILE, 143, 0],
+        "意图那一版单元都在；第一批从尾部删掉 144 个、留 143 个；第二批删完"
+    );
+    assert!(
+        matches!(
+            recover(&pool.memory_pool(), JournalPolicy::Consult).outcome,
+            RecoveryOutcome::NoFile { .. }
+        ),
+        "删完之后冷走读答「没有文件」"
+    );
+}
diff --git a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
index 19d4133..e70a664 100644
--- a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
+++ b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
@@ -55,6 +55,7 @@ use singlefs_harness::fault_injection::{
     FaultSchedule, InjectedFault, SharedFaultPlan,
 };
 use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool};
+use singlefs_harness::segments::StepKind;
 use singlefs_harness::SharedStream;
 
 /// 每块盘第 `ordinal` 次读系统配置槽 0（偏移 0）时注入一次读错，别的读写原样交给内层
@@ -218,22 +219,65 @@ fn writable_mount_after_a_crash_right_after_acquiring_an_instance_writes_a_row_f
 
 /// 环里留着一条孤记录时，只做过 mkfs 的池照常可写挂载（2026-09-23 用户定案收窄 R4：`NEW_POOL_FILE_CREATION_TXG` 只管 mkfs
 /// 同一个进程里那条流，不管任何池的第一个文件版本，所以「新实例的第一次发布不是 txg 1、jsn 1」不再是拒绝的理由）：
-/// mkfs 之后第一次可写挂载崩在取号两写与 txg 1 的记录两写都持久、txg 1 的根槽没持久；两块盘系统配置槽 0（取号写进号 1 的那一槽）
-/// 退回 mkfs 写的那一份（造出来的盘面：取号写之后才有记录，真崩溃留不下这一形）——择系统配置只剩 mkfs 的那一份、根环只有第 0 代根，要取的号 1、要写的行为空；环里那条 txg 1 的记录让新实例从
+/// mkfs 之后第一次可写挂载取号那两次写（两块盘系统配置槽 0，写进号 1 的那一槽）被说谎的设备吞掉（`InjectedFault::WriteIsSwallowed`：报成功、
+/// 一个字节都没落盘），之后崩在 txg 1 的记录两写持久、txg 1 的根槽没持久（真崩溃留不下这一形：取号写与记录写之间有屏障，记录持久而取号写没持久
+/// 只有说谎的设备做得出；代码轮 LR6 把用例自己改写系统配置槽的造法换成这一种）——两块盘槽 0 仍是 mkfs 写的那一份，择系统配置只剩 mkfs 的那一份、
+/// 根环只有第 0 代根，要取的号 1、要写的行为空；环里那条 txg 1 的记录让新实例从
 /// txg 2、jsn 2 起 ⇒ 零单元发布 txg 2（落盘 0），暖机推到 txg 4（落盘 1）才覆盖两块盘，之后第一个文件版本接着写 txg 5。
 #[test]
 fn formatted_pool_mount_starting_after_a_leftover_record_publishes_from_the_next_txg() {
     let mut formatted = format_pool("step-three-formatted-leftover-record");
-    let mut first_mount_devices = formatted.reopen_recorded();
-    mount_writable(&parameters(), &mut first_mount_devices).expect("第一次可写挂载");
-    formatted.devices = Some(first_mount_devices);
+    // 取号那两次写（每块盘头一次写系统配置槽 0）被设备吞掉：注入层包在录制器外面，吞掉的写不进录制流，录制流里的就是落到设备上的。
+    // 不吞成撕裂：C331（择根倒挂压过已确认的写） 取甲之后见证读缺一槽就拒可写（用户 2026-09-28 定），撕裂造的走不到孤记录这一支；
+    // 吞掉之后槽 0 仍是 mkfs 写的那一份（mkfs 两槽写同一份字节），两槽都自证得过。
+    let swallowing_the_acquisition = SharedFaultPlan::armed(
+        geometry(),
+        FaultSchedule {
+            fault: InjectedFault::WriteIsSwallowed,
+            device: FaultDeviceSelector::EveryDevice,
+            placement: FaultPlacement::OffsetExactly(DeviceOffsetInBytes(0)),
+            counting: FaultCounting::PerDevice,
+            occurrence: FaultOccurrence::TheNthMatchingCall(1),
+        },
+    );
+    let mut first_mount_devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<_>)> = formatted
+        .reopen_recorded()
+        .into_iter()
+        .map(|(identity, device)| {
+            (
+                identity,
+                FaultInjectingBlockDevice::new(
+                    identity,
+                    device,
+                    swallowing_the_acquisition.clone(),
+                ),
+            )
+        })
+        .collect();
+    // 说谎的设备之后这次挂载自己怎么收尾不判：只要它在 txg 1 的根槽之前那几次写（录制流里）。
+    let _first_mount = mount_writable(&parameters(), &mut first_mount_devices);
+    assert_eq!(
+        swallowing_the_acquisition.fired().len(),
+        2,
+        "两块盘各吞了一次写系统配置槽 0（取号那两次）"
+    );
+    drop(first_mount_devices);
     let operations = formatted.retained_operations();
-    let (writes, segments) =
+    let (writes, _segments) =
         writes_and_segments(&operations[formatted.mkfs_operation_count..], &geometry());
-    let mut persisted = vec![false; writes.len()];
-    for write_index in segments[0].iter().chain(segments[1].iter()) {
-        persisted[*write_index] = true;
-    }
+    let first_root_slot_write = writes
+        .iter()
+        .position(|write| write.kind == StepKind::RootRecordFua)
+        .expect("第一次可写挂载写了 txg 1 的根槽");
+    let persisted: Vec<bool> = (0..writes.len())
+        .map(|write_index| write_index < first_root_slot_write)
+        .collect();
+    assert!(
+        writes[..first_root_slot_write]
+            .iter()
+            .any(|write| write.kind == StepKind::JournalRecord),
+        "txg 1 的根槽之前落了它的记录"
+    );
     let crash_stream = SharedStream::retaining_contents();
     let mut devices = crash_state_devices(
         &formatted.memory_pool_after_mkfs(),
@@ -241,22 +285,6 @@ fn formatted_pool_mount_starting_after_a_leftover_record_publishes_from_the_next
         &persisted,
         &crash_stream,
     );
-    // 系统配置槽 0 退回 mkfs 写的那一份（mkfs 两槽写同一份字节，拿槽 1 的原样写回），两槽都自证得过：取号写进号 1 的那一槽看不见了。
-    // 不写坏一个字节：C331（择根倒挂压过已确认的写） 取甲之后见证读缺一槽就拒可写（用户 2026-09-28 定），那样造的是撕裂轮换，走不到孤记录这一支。
-    let slot_spacing_in_bytes = u64::from(geometry().fixed_structure_slot_spacing);
-    let system_configuration_slot_bytes =
-        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
-    for (_, device) in &mut devices {
-        let mut mkfs_slot = vec![0u8; system_configuration_slot_bytes];
-        device
-            .wrapped_device()
-            .read_at(DeviceOffsetInBytes(slot_spacing_in_bytes), &mut mkfs_slot)
-            .expect("读系统配置槽 1（mkfs 写的那一份）");
-        device
-            .wrapped_device_mut()
-            .image
-            .write(DeviceOffsetInBytes(0), &mkfs_slot);
-    }
     let image_before = memory_pool_of_sparse_devices(&devices);
     let before = disk_snapshot(&image_before, &crash_stream);
     assert_eq!(
~~~~~~

## 二、实现员 B

~~~~~~diff
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index 4037b0c..8a19e71 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -40,14 +40,17 @@ use singlefs_core::address::{
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
-use singlefs_core::mount::{mount_writable_with_space_admission, MountError};
+use singlefs_core::mount::{
+    mount_writable_with_space_admission, MountError, MountOutput, RaisedFloor,
+};
 use singlefs_core::recovery::{
     choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryOutcome,
     RecoveryReport,
 };
 use singlefs_core::transaction::{
-    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
+    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
 };
+use singlefs_core::write_accounting::{WritesByStructureKind, WrittenStructureKind};
 
 use crate::crash::check_records_against;
 use singlefs_harness::history::{
@@ -201,8 +204,28 @@ pub enum CrashPointDraw {
         segment_writes: usize,
         sampled_in_longer_segments: usize,
     },
+    /// 就摆这一个崩溃状态（靶向用例用，代码三方 m2-supp3-item3-code-r2 判决第四节第 1 题，用户 2026-09-28 选「靶向用例 + 变异行」）：
+    /// 从第 `segment_index` 段的第一个写起数 `writes_from_the_segment_on` 个写，其中下标（从这一段的第一个写数起）在
+    /// `withheld_among_them` 里的扣下、别的持久；更早的段整段持久，这几个写之后的一个都没持久。
+    ///
+    /// 这几个写可以越过这一段的末尾，越过的那几道屏障当设备没守：记录核对器两条判据防的正是屏障没守住才有的状态
+    /// （记录与根之间那道没守：根在而记录不在；单元与根之间的没守：恢复自称的那一版之前的发布缺单元），
+    /// 今天的持久顺序下它们在段内真子集里一个都摆不出来，判据接没接上只有这样摆得出来看（K3-④⑨）。
+    ///
+    /// 只给写死的一段历史经 [`inject_crashes_into_history`] 用，不进战役：写死的段号只对写死的那一段历史成立，重放行也写不出它
+    /// （[`run_crash_injection_campaign`] 拿到它就 panic）。段号不是候选段、这几个写越过最后一个跑完的操作（或写表末尾）、
+    /// `writes_from_the_segment_on` 是 0、扣下的下标越界或一个都没有时 panic——写死的输入写错了，不悄悄摆别的状态。
+    ExactlyOneCrashState {
+        segment_index: usize,
+        writes_from_the_segment_on: usize,
+        withheld_among_them: &'static [usize],
+    },
 }
 
+/// 靶向用例那一支（[`CrashPointDraw::ExactlyOneCrashState`]）拿去跑战役或写重放行时的 panic 消息。
+const A_TARGETED_CRASH_STATE_IS_NOT_A_CAMPAIGN_DRAW: &str =
+    "靶向用例那一个崩溃状态不进战役：写死的段号只对写死的那一段历史成立，重放行写不出它（只经 inject_crashes_into_history 摆）";
+
 /// 失败时那份崩溃镜像留不留。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum FailureImageRetention {
@@ -214,10 +237,11 @@ pub enum FailureImageRetention {
 
 /// 一个崩溃状态：截在录制流的第几段、那一段里哪几个写持久了。
 /// 更早的段整段持久、更晚的段一个都没持久；这一段里持久的是一个真子集（全持久等于「没崩在这一段」，那个状态由下一段的空子集摆）。
+/// 靶向用例（[`CrashPointDraw::ExactlyOneCrashState`]）摆的那一个例外：持久集合从这一段的第一个写起，可以越过这一段的末尾。
 #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
 pub struct CrashPoint {
     pub segment_index: usize,
-    /// 这一段里第 k 个写持久了没有（长度就是这一段的写数）。
+    /// 这一段里第 k 个写持久了没有（长度就是这一段的写数；靶向用例摆的那一个，长度是它从这一段的第一个写起数的写数）。
     pub persisted_within_the_segment: Vec<bool>,
     /// 这一段里第一个没持久的写落在历史的哪一步里。
     pub step: StepPosition,
@@ -956,7 +980,8 @@ impl StreamMarks {
 /// `image_directory` 给了就把判红的那几个崩溃状态（哪一截判红就落哪一截的镜像）落成镜像文件放进去（留现场）。
 ///
 /// # Panics
-/// 录制流没开内容保留（重建镜像要字节）——这里自己建流，走不到。
+/// 录制流没开内容保留（重建镜像要字节）——这里自己建流，走不到；`draw` 是 [`CrashPointDraw::ExactlyOneCrashState`] 而写死的段号、
+/// 写数或扣下的下标在这段历史的录制流上不成立（见那一支）。
 #[must_use]
 pub fn inject_crashes_into_history(
     history: &GeneratedHistory,
@@ -1045,8 +1070,7 @@ pub fn inject_crashes_into_history(
         new_findings: Vec::new(),
     };
     for crash_point in &crash_points {
-        let segment = &segments[crash_point.segment_index];
-        let first_write_of_the_segment = segment[0];
+        let first_write_of_the_segment = segments[crash_point.segment_index][0];
         let writes_in_the_segment = crash_point.persisted_within_the_segment.len();
         // 段号从小到大，基线只往前叠：更早的段整段持久，按写表整段施加，不为每个崩溃状态从头重建。
         base.apply_writes(&writes[writes_applied_to_the_base..first_write_of_the_segment]);
@@ -1066,7 +1090,7 @@ pub fn inject_crashes_into_history(
         };
 
         tally.crash_points += 1;
-        for kind in withheld_write_kinds(&writes, segment, crash_point) {
+        for kind in withheld_write_kinds(&writes, first_write_of_the_segment, crash_point) {
             *tally
                 .crash_points_withholding_write_kind
                 .entry(kind.name())
@@ -1111,9 +1135,14 @@ pub fn inject_crashes_into_history(
         let disagreement =
             crash_recovery_disagreement(&committed_versions, newest_persisted, &read_back);
         // RefFS 那一票：崩溃时刻按切开当前段的那道屏障由哪一步发出归属（r6 用户定案 H1-收严，只给 RefFS 用；
-        // O1 判官与 `crash_point.step` 照旧按段里第一个被扣下的写）。
+        // O1 判官与 `crash_point.step` 照旧按段里第一个被扣下的写）。当前段取持久窗口最后一个写所在那一段：段内的崩溃状态就是
+        // `segment_index` 那一段；靶向用例越过段尾的那一个，崩在窗口末尾那一段里（窗口里更晚的写都可能已落）。
+        let segment_the_crash_closes = segment_containing_the_write(
+            &segments,
+            first_write_of_the_segment + writes_in_the_segment - 1,
+        );
         let reference_filesystem_judgement = reference_filesystem_history.judge_a_crash(
-            reference_filesystem_history.crash_moment_in_segment(crash_point.segment_index),
+            reference_filesystem_history.crash_moment_in_segment(segment_the_crash_closes),
             &read_back,
         );
         tally.note_the_reference_filesystem_vote(
@@ -1552,10 +1581,10 @@ pub struct WritableMountOnTheCrashImage {
     /// 挂载交回值里推没推抬 F、最后抬到的 F（挂载透明要它）；挂载被拒时 None。
     pub floor_raised_by_the_mount: Option<FloorRaisedByTheMount>,
     /// 挂载续做之前这次挂载发了几次（写行、暖机、推的抬 F 的每一次空发布）：挂载那一段录制流里第几个根槽写之后起是续做；
-    /// 挂载被拒时 None。
+    /// 挂载交回 `MountError::Publish` 时按错里交回的已落盘发布写账划（划不出续做、别的错都是 None：写行之后都算暖机）。
     pub publishes_before_the_deletion_queue_continuation: Option<usize>,
     /// 挂载推的抬 F 从挂载那一段写表（`mount_operations` 按同一份几何切出来的写表）的哪个写起（二次崩溃上 F 判不判按它分；
-    /// 界按挂载交回的抬 F 那几串在写表末尾的位置取，取不出的是 [`FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites`]）；挂载被拒时 None。
+    /// 挂载准入推的与续做里推的都认，界按它们在写表里的位置取，取不出的是 [`FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites`]）；挂载被拒时 None。
     pub floor_raises_in_the_mount_writes: Option<FloorRaisesInTheMountWrites>,
 }
 
@@ -1602,18 +1631,30 @@ pub fn mount_and_publish_once_on_the_crash_image(
         .as_ref()
         .ok()
         .map(|mounted| FloorRaisedByTheMount::of_the_mount_output(&mounted.output));
-    let publishes_before_the_deletion_queue_continuation = mounted.as_ref().ok().map(|mounted| {
-        1 + mounted.output.warm_up_publishes.len()
-            + mounted
-                .output
-                .space_admission
-                .floor_raises()
-                .iter()
-                .map(|raised| raised.publishes.len())
-                .sum::<usize>()
-    });
-    // 录制流此刻只有挂载那一段：按建池那一份参数的几何切出写表，与调用方切二次崩溃的写表同一张。抬 F 的界按位置取（抬 F 那几串是这张表
-    // 末尾的写），取不出时不 panic、交 NotLocatedInTheMountWrites，这一次挂载的二次崩溃整段 F 不判、按理由计进 `mount_transparency_fields_not_judged`。
+    let publishes_before_the_deletion_queue_continuation = match &mounted {
+        Ok(mounted) => Some(
+            1 + mounted.output.warm_up_publishes.len()
+                + mounted
+                    .output
+                    .space_admission
+                    .floor_raises()
+                    .iter()
+                    .map(|raised| raised.publishes.len())
+                    .sum::<usize>(),
+        ),
+        // 挂载在续做里交回错（U12，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：续做从第几次发布起按错里交回的
+        // 已落盘发布写账划（[`publishes_before_the_continuation_of_a_mount_that_handed_back_an_error`]），不并进暖机段。
+        Err(MountError::Publish(failed)) => {
+            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
+                &failed.writes_of_persisted_publishes,
+                &failed.writes_of_failed_publishes,
+            )
+        }
+        Err(_) => None,
+    };
+    // 录制流此刻只有挂载那一段：按建池那一份参数的几何切出写表，与调用方切二次崩溃的写表同一张。抬 F 的界按位置取（挂载准入推的与续做里推的
+    // 那几串都认，[`floor_raises_in_the_mount_writes_counting_the_continuation`]），取不出时不 panic、交 NotLocatedInTheMountWrites，
+    // 这一次挂载的二次崩溃整段 F 不判、按理由计进 `mount_transparency_fields_not_judged`。
     let floor_raises_in_the_mount_writes = mounted.as_ref().ok().map(|mounted| {
         let geometry = FixedGeometry {
             fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
@@ -1622,7 +1663,7 @@ pub fn mount_and_publish_once_on_the_crash_image(
         };
         let (mount_writes, _mount_segments, _mount_stream_indexes) =
             writes_and_segments_with_stream_indexes(&stream.retained_operations(), &geometry);
-        FloorRaisesInTheMountWrites::of_the_mount_output(&mounted.output, &mount_writes)
+        floor_raises_in_the_mount_writes_counting_the_continuation(&mounted.output, &mount_writes)
     });
     let outcome = match mounted {
         Err(error) => WritableMountOutcome::MountRefused {
@@ -1688,6 +1729,249 @@ pub fn mount_and_publish_once_on_the_crash_image(
     }
 }
 
+/// 挂载里一次发布交给设备的写数与它写的根身份 (实例代号, checkpoint_txg)。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct PublishInTheMountWrites {
+    write_calls: u64,
+    root: (InstanceGeneration, CheckpointTxg),
+}
+
+impl PublishInTheMountWrites {
+    fn of_the_publish(publish: &TransactionOutput) -> Self {
+        PublishInTheMountWrites {
+            write_calls: publish.writes.total().write_calls,
+            root: (publish.root.instance, publish.root.checkpoint_txg),
+        }
+    }
+}
+
+/// 挂载推的一串抬 F 的写账：先把新 F 写进每块盘系统配置的那几个写（`RaisedFloor::system_configuration_writes_before_the_first_publish`），
+/// 再是各次带新 F 的空发布（按先后）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct FloorRaiseInTheMountWrites {
+    system_configuration_write_calls: u64,
+    publishes: Vec<PublishInTheMountWrites>,
+}
+
+impl FloorRaiseInTheMountWrites {
+    fn of_the_raise(raise: &RaisedFloor) -> Self {
+        FloorRaiseInTheMountWrites {
+            system_configuration_write_calls: raise
+                .system_configuration_writes_before_the_first_publish
+                .total()
+                .write_calls,
+            publishes: raise
+                .publishes
+                .iter()
+                .map(PublishInTheMountWrites::of_the_publish)
+                .collect(),
+        }
+    }
+
+    fn write_calls(&self) -> u64 {
+        self.system_configuration_write_calls
+            + self
+                .publishes
+                .iter()
+                .map(|publish| publish.write_calls)
+                .sum::<u64>()
+    }
+
+    /// # Panics
+    /// 这一串一次空发布都没有：抬 F 那一串推空发布直到每块盘上都有带新 F 的根，至少一次（`mount::RaisedFloor::publishes`）。
+    fn checkpoint_txg_of_the_first_publish(&self) -> CheckpointTxg {
+        self.publishes
+            .first()
+            .expect("抬 F 那一串至少推一次带新 F 的空发布（RaisedFloor::publishes 的契约）")
+            .root
+            .1
+    }
+}
+
+/// 一次可写挂载交回的、取抬 F 那条界要用的三样：挂载准入在写行与暖机之后推的那几串（`MountOutput::space_admission`）、
+/// 续做发成的每一步（`MountOutput::deletion_queue_continuation`）与续做里为空间推的那几串（`MountOutput::floor_raises_pushed_by_the_continuation`，
+/// 与续做的各步按 txg 交错）。挂载写表的末尾依次是：准入推的那几串、续做的各步与续做推的那几串。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct FloorRaisesOfTheMount {
+    pushed_by_the_admission: Vec<FloorRaiseInTheMountWrites>,
+    continuation_steps: Vec<PublishInTheMountWrites>,
+    pushed_by_the_continuation: Vec<FloorRaiseInTheMountWrites>,
+}
+
+impl FloorRaisesOfTheMount {
+    fn of_the_mount_output(output: &MountOutput) -> Self {
+        FloorRaisesOfTheMount {
+            pushed_by_the_admission: output
+                .space_admission
+                .floor_raises()
+                .iter()
+                .map(FloorRaiseInTheMountWrites::of_the_raise)
+                .collect(),
+            continuation_steps: output
+                .deletion_queue_continuation
+                .iter()
+                .map(PublishInTheMountWrites::of_the_publish)
+                .collect(),
+            pushed_by_the_continuation: output
+                .floor_raises_pushed_by_the_continuation
+                .iter()
+                .map(FloorRaiseInTheMountWrites::of_the_raise)
+                .collect(),
+        }
+    }
+
+    /// 从第一串抬 F 的第一个写起、到挂载写表末尾一共几个写，连同这一截里每一次抬 F 发布的根身份；一串都没推时 None。
+    /// 第一串是准入推的（推了的话）：它之后是准入推的别的串、续做的每一步与续做推的每一串。准入一串都没推、续做里推了的：
+    /// 第一串是续做推的第一串，它之后是续做推的每一串与 txg 大于它第一次空发布的那几步续做（之前的那几步在界之前）。
+    fn writes_from_the_first_floor_raise_on(
+        &self,
+    ) -> Option<(u64, Vec<(InstanceGeneration, CheckpointTxg)>)> {
+        let writes_of = |raises: &[FloorRaiseInTheMountWrites]| {
+            raises
+                .iter()
+                .map(FloorRaiseInTheMountWrites::write_calls)
+                .sum::<u64>()
+        };
+        let roots_of = |raises: &[FloorRaiseInTheMountWrites]| {
+            raises
+                .iter()
+                .flat_map(|raise| raise.publishes.iter().map(|publish| publish.root))
+                .collect::<Vec<_>>()
+        };
+        let continuation_steps_after = |checkpoint_txg: Option<CheckpointTxg>| {
+            self.continuation_steps
+                .iter()
+                .filter(|step| checkpoint_txg.is_none_or(|boundary| step.root.1 > boundary))
+                .map(|step| step.write_calls)
+                .sum::<u64>()
+        };
+        if !self.pushed_by_the_admission.is_empty() {
+            let mut roots = roots_of(&self.pushed_by_the_admission);
+            roots.extend(roots_of(&self.pushed_by_the_continuation));
+            return Some((
+                writes_of(&self.pushed_by_the_admission)
+                    + continuation_steps_after(None)
+                    + writes_of(&self.pushed_by_the_continuation),
+                roots,
+            ));
+        }
+        let first_pushed_by_the_continuation = self.pushed_by_the_continuation.first()?;
+        Some((
+            writes_of(&self.pushed_by_the_continuation)
+                + continuation_steps_after(Some(
+                    first_pushed_by_the_continuation.checkpoint_txg_of_the_first_publish(),
+                )),
+            roots_of(&self.pushed_by_the_continuation),
+        ))
+    }
+}
+
+/// 挂载推的抬 F 从挂载写表的哪个写起（[`FloorRaisesInTheMountWrites`]；`singlefs_harness::mount_transparency` 那一份
+/// `FloorRaisesInTheMountWrites::of_the_mount_output` 只认挂载准入推的、按「抬 F 那几串在写表末尾」取界，挂载有续做时取不出、
+/// 续做里推的看不见）：挂载准入推的与续做里推的都认（[`FloorRaisesOfTheMount::writes_from_the_first_floor_raise_on`]），
+/// 界 = 写表长度减去从第一串抬 F 起的写数。取出的界再核两样：界上那个写是系统配置槽写（每一串先把新 F 写进系统配置），
+/// 每一次抬 F 发布的根槽写（按实例与 txg 认）都在界之后；对不上的交 [`FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites`]，不 panic。
+fn floor_raises_in_the_mount_writes_counting_the_continuation(
+    output: &MountOutput,
+    mount_writes: &[RetainedWrite],
+) -> FloorRaisesInTheMountWrites {
+    floor_raises_located_in_the_mount_writes(
+        &FloorRaisesOfTheMount::of_the_mount_output(output),
+        mount_writes,
+    )
+}
+
+fn floor_raises_located_in_the_mount_writes(
+    floor_raises: &FloorRaisesOfTheMount,
+    mount_writes: &[RetainedWrite],
+) -> FloorRaisesInTheMountWrites {
+    let Some((writes_from_the_first_floor_raise_on, floor_raise_roots)) =
+        floor_raises.writes_from_the_first_floor_raise_on()
+    else {
+        return FloorRaisesInTheMountWrites::NotPushed;
+    };
+    let Some(first_write) = usize::try_from(writes_from_the_first_floor_raise_on)
+        .ok()
+        .and_then(|writes| mount_writes.len().checked_sub(writes))
+    else {
+        return FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites;
+    };
+    let the_boundary_is_a_system_configuration_slot_write = mount_writes
+        .get(first_write)
+        .is_some_and(|write| write.kind == StepKind::SystemConfigurationSlot);
+    let roots_written_from_the_boundary: Vec<(InstanceGeneration, CheckpointTxg)> = mount_writes
+        [first_write..]
+        .iter()
+        .filter(|write| write.kind == StepKind::RootRecordFua)
+        .filter_map(|write| write.bytes().map(root_identity_written_by))
+        .collect();
+    let every_floor_raise_root_is_written_from_the_boundary = floor_raise_roots
+        .iter()
+        .all(|root| roots_written_from_the_boundary.contains(root));
+    if the_boundary_is_a_system_configuration_slot_write
+        && every_floor_raise_root_is_written_from_the_boundary
+    {
+        FloorRaisesInTheMountWrites::FirstWriteAt(first_write)
+    } else {
+        FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites
+    }
+}
+
+/// 一次发布写没写只有续做的删除步才写的那几种结构：文件的 extent 树、inode 树与 `deleted_inodes` 树（数据单元也算，挂载里没有别的发布写它）。
+/// 写行、暖机与抬 F 的空发布只重写分配记录树、记账树、中央映射树、树表、实例表与固定结构（2026-10-03 在副本上量的：崩在意图之后重挂，
+/// 写行与两次暖机写的是这几种，续做那一步写了 extent 树与 inode 树根）。
+fn writes_a_structure_only_the_continuation_changes(writes: &WritesByStructureKind) -> bool {
+    WrittenStructureKind::IN_REPORT_ORDER.iter().any(|kind| {
+        let only_the_continuation_writes_it = match kind {
+            WrittenStructureKind::DataUnit
+            | WrittenStructureKind::ExtentTreeNode
+            | WrittenStructureKind::InodeTreeLeafContainer
+            | WrittenStructureKind::InodeTreeRoot
+            | WrittenStructureKind::DeletedInodesTreeNode => true,
+            WrittenStructureKind::AllocationRecordTreeNode
+            | WrittenStructureKind::AccountingTreeNode
+            | WrittenStructureKind::TombstoneTreeNode
+            | WrittenStructureKind::LineageTreeNode
+            | WrittenStructureKind::CentralMappingTreeNode
+            | WrittenStructureKind::TreeTableUnit
+            | WrittenStructureKind::InstanceTableUnit
+            | WrittenStructureKind::JournalRecord
+            | WrittenStructureKind::RootSlot
+            | WrittenStructureKind::SystemConfigurationSlot => false,
+        };
+        only_the_continuation_writes_it && writes.of(*kind).write_calls > 0
+    })
+}
+
+/// 挂载在写行之后交回 `MountError::Publish` 时续做之前发了几次（U12，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：
+/// `writes_of_persisted_publishes` 是已落盘的各次发布的写账（按先后，写行那次在最前，`mount::PublishSequenceFailed`），
+/// `writes_of_failed_publishes` 是落盘阶段失败的那一次的。写行之后第一个写了只有续做才写的结构
+/// （[`writes_a_structure_only_the_continuation_changes`]）的那一次起是续做；已落盘的都没写、失败的那一次写了，续做从失败那一次起
+/// （已落盘的那几次之后）；都没写（错出在写行、暖机或准入推的抬 F 里，或续做的第一步落盘之前就被拒、一个写都没发）是 None，写行之后都算暖机。
+/// 续做开头先被空间拒、先推的那一串抬 F 归暖机：错里分不出它是准入推的还是续做推的。
+fn publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
+    writes_of_persisted_publishes: &[WritesByStructureKind],
+    writes_of_failed_publishes: &[WritesByStructureKind],
+) -> Option<usize> {
+    let first_persisted_continuation_step = writes_of_persisted_publishes
+        .iter()
+        .enumerate()
+        .skip(1)
+        .find(|(_, writes)| writes_a_structure_only_the_continuation_changes(writes))
+        .map(|(publish_index, _)| publish_index);
+    match first_persisted_continuation_step {
+        Some(publish_index) => Some(publish_index),
+        None if !writes_of_persisted_publishes.is_empty()
+            && writes_of_failed_publishes
+                .iter()
+                .any(writes_a_structure_only_the_continuation_changes) =>
+        {
+            Some(writes_of_persisted_publishes.len())
+        }
+        None => None,
+    }
+}
+
 /// 第二、三截交给记录核对器的记录流：整条历史录制流接上那份崩溃后镜像上可写挂载那一段录制流、再接上挂载之后那一次发布的录制流，
 /// 在历史与挂载的接缝处断开（[`RecordStreamContinuity::ResumedAfterACrash`]：历史那一段只留第一次崩溃之后那条时间线上的发布）。
 /// 历史那一段的持久集合是第一次崩溃那个状态的，与第一截交的同一张。只交挂载那一段时，历史那几次发布在挂载之后的池与二次崩溃镜像上
@@ -2586,10 +2870,11 @@ fn judge_the_remount_after_a_second_crash(
     );
 }
 
-/// 这个崩溃状态在段内扣下的写各是什么种类（同一种只记一次）。
+/// 这个崩溃状态在段内扣下的写各是什么种类（同一种只记一次）。段是写表里连续的一截
+/// （`writes_and_segments_with_stream_indexes` 按次序切），第 k 个写的下标是段首加 k——靶向用例摆的持久集合越过段尾时也按这个数。
 fn withheld_write_kinds(
     writes: &[RetainedWrite],
-    segment: &[usize],
+    first_write_of_the_segment: usize,
     crash_point: &CrashPoint,
 ) -> Vec<StepKind> {
     let mut kinds: BTreeSet<StepKind> = BTreeSet::new();
@@ -2597,12 +2882,23 @@ fn withheld_write_kinds(
         crash_point.persisted_within_the_segment.iter().enumerate()
     {
         if !is_persisted {
-            kinds.insert(writes[segment[within_the_segment]].kind);
+            kinds.insert(writes[first_write_of_the_segment + within_the_segment].kind);
         }
     }
     kinds.into_iter().collect()
 }
 
+/// 写表下标 `write` 落在第几段。
+///
+/// # Panics
+/// 这个写不在任何一段里：段与写表不是同一条流切的（每个写都恰好落在一个段里，`split_into_segments`）。
+fn segment_containing_the_write(segments: &[Vec<usize>], write: usize) -> usize {
+    segments
+        .iter()
+        .position(|segment| segment.contains(&write))
+        .expect("每个写都恰好落在一个段里：段与写表是同一条流按同一条规则切的")
+}
+
 /// 按 `draw` 摆崩溃状态：候选段是「整段的写都落在 mkfs 之后、最后一个跑完的操作之前」的那几段，段内取真子集。
 ///
 /// 起点那一段（取号、暖机、第一个文件）在内——2026-09-20 才放开；mkfs 那几段不在内，与层 0 同一条界：
@@ -2610,6 +2906,7 @@ fn withheld_write_kinds(
 /// （层 0 也从 `mkfs_operation_count` 之后起枚举）。失败那一步的写也不在内（模型还没判过它写出的根）。
 ///
 /// 抽的那一路同一段同一个子集只摆一次；摆完按段号排好——基线只往前叠。
+/// 靶向用例那一支（[`CrashPointDraw::ExactlyOneCrashState`]）只摆写死的那一个，持久集合可以越过段尾；写死的输入不对就 panic（见那一支）。
 fn draw_crash_points(
     seed: HistorySeed,
     segments: &[Vec<usize>],
@@ -2626,15 +2923,16 @@ fn draw_crash_points(
             })
         })
         .collect();
-    if candidate_segments.is_empty() {
-        return Vec::new();
-    }
     let mut source = SeededRandomSource::from_seed(seed.0 ^ CRASH_POINT_SEED_SALT);
     let mut chosen: BTreeSet<(usize, Vec<bool>)> = BTreeSet::new();
     match draw {
         CrashPointDraw::Sampled {
             crash_points_per_history,
         } => {
+            // 一个候选段都没有时一个都不摆（下面按候选段数取随机下标，取不出）。
+            if candidate_segments.is_empty() {
+                return Vec::new();
+            }
             for _ in 0..crash_points_per_history {
                 let picked = usize::try_from(
                     source.below(u64::try_from(candidate_segments.len()).expect("候选段数")),
@@ -2663,6 +2961,42 @@ fn draw_crash_points(
                 }
             }
         }
+        CrashPointDraw::ExactlyOneCrashState {
+            segment_index,
+            writes_from_the_segment_on,
+            withheld_among_them,
+        } => {
+            assert!(
+                candidate_segments.contains(&segment_index),
+                "靶向用例写死的第 {segment_index} 段不是候选段（整段落在 mkfs 之后、最后一个跑完的操作之前的那几段）：{candidate_segments:?}"
+            );
+            let first_write_of_the_segment = segments[segment_index][0];
+            let writes_of_the_state =
+                first_write_of_the_segment..first_write_of_the_segment + writes_from_the_segment_on;
+            assert!(
+                writes_from_the_segment_on > 0 && writes_of_the_state.end <= stream_indexes.len(),
+                "靶向用例从第 {segment_index} 段起数 {writes_from_the_segment_on} 个写：要至少一个、不越过写表末尾（{} 个写）",
+                stream_indexes.len()
+            );
+            // 段首已经在候选段里（落在 mkfs 之后）；这几个写的末尾不越过最后一个跑完的操作。
+            assert!(
+                stream_indexes[writes_of_the_state.end - 1] < marks.after_the_last_finished_step,
+                "靶向用例从第 {segment_index} 段起数的 {writes_from_the_segment_on} 个写越过了最后一个跑完的操作"
+            );
+            assert!(
+                !withheld_among_them.is_empty()
+                    && withheld_among_them
+                        .iter()
+                        .all(|withheld| *withheld < writes_from_the_segment_on),
+                "靶向用例扣下的写 {withheld_among_them:?} 要至少一个、都落在从这一段起数的 {writes_from_the_segment_on} 个写里"
+            );
+            chosen.insert((
+                segment_index,
+                (0..writes_from_the_segment_on)
+                    .map(|within_the_state| !withheld_among_them.contains(&within_the_state))
+                    .collect(),
+            ));
+        }
     }
     chosen
         .into_iter()
@@ -2671,8 +3005,9 @@ fn draw_crash_points(
                 .iter()
                 .position(|persisted| !persisted)
                 .expect("段内持久的是真子集，至少有一个写被扣下");
+            // 段是写表里连续的一截：段内第 k 个写是段首加 k（靶向用例的持久集合越过段尾时也这样数）。
             let step = marks
-                .step_containing(stream_indexes[segments[segment_index][first_withheld]])
+                .step_containing(stream_indexes[segments[segment_index][0] + first_withheld])
                 .expect("崩溃状态摆在最后一个跑完的操作之前，登记里一定找得到它那一步");
             let operation_kind = match step {
                 StepPosition::StartingPoint => None,
@@ -3028,6 +3363,9 @@ fn draw_environment_values(draw: CrashPointDraw) -> (String, String) {
             sampled_in_longer_segments.to_string(),
             segment_writes.to_string(),
         ),
+        CrashPointDraw::ExactlyOneCrashState { .. } => {
+            panic!("{A_TARGETED_CRASH_STATE_IS_NOT_A_CAMPAIGN_DRAW}")
+        }
     }
 }
 
@@ -3221,7 +3559,8 @@ pub struct CrashInjectionCampaign {
 /// [`CrashInjectionReport::render`] 与线程数、调度次序无关（进度行是按到达次序打的，只报跑到哪了，不带判定）。
 ///
 /// # Panics
-/// 某个工作线程 panic（历史里的 panic 在执行器里接住，走到这里的是崩溃状态上恢复或 checker 的断言）；有一片领了却没交回。
+/// 某个工作线程 panic（历史里的 panic 在执行器里接住，走到这里的是崩溃状态上恢复或 checker 的断言）；有一片领了却没交回；
+/// `draw` 是靶向用例那一支 [`CrashPointDraw::ExactlyOneCrashState`]（只经 [`inject_crashes_into_history`] 摆，不进战役）。
 #[must_use]
 pub fn run_crash_injection_campaign(campaign: &CrashInjectionCampaign) -> CrashInjectionReport {
     let CrashInjectionCampaign {
@@ -3235,6 +3574,10 @@ pub fn run_crash_injection_campaign(campaign: &CrashInjectionCampaign) -> CrashI
         image_retention,
         assertions,
     } = *campaign;
+    assert!(
+        !matches!(draw, CrashPointDraw::ExactlyOneCrashState { .. }),
+        "{A_TARGETED_CRASH_STATE_IS_NOT_A_CAMPAIGN_DRAW}"
+    );
     let slices = seed_slices(seed_count, worker_threads.count());
     let spawned_worker_threads = worker_threads.count().min(slices.len().max(1));
     let started = Instant::now();
@@ -3381,6 +3724,7 @@ pub fn seed_slices(seed_count: u64, worker_threads: usize) -> Vec<std::ops::Rang
 mod tests {
     use super::*;
     use singlefs_harness::history::{generate_history, HistoryStartingPoint};
+    use singlefs_harness::mount_transparency::FloorRaisesAtTheSecondCrash;
 
     const FOUR_CRASH_POINTS: CrashPointDraw = CrashPointDraw::Sampled {
         crash_points_per_history: 4,
@@ -3505,6 +3849,242 @@ mod tests {
         );
     }
 
+    /// 造写表用：一个 512 字节的写；根槽写的字节里在偏移 24、28 写上根身份（`root_identity_written_by` 的读法）。
+    fn synthetic_write(kind: StepKind, root: Option<(u32, u64)>) -> RetainedWrite {
+        let mut bytes = vec![0u8; 512];
+        if let Some((instance, checkpoint_txg)) = root {
+            bytes[24..28].copy_from_slice(&instance.to_le_bytes());
+            bytes[28..36].copy_from_slice(&checkpoint_txg.to_le_bytes());
+        }
+        RetainedWrite {
+            device: DeviceIdentity(0),
+            kind,
+            is_force_unit_access: kind == StepKind::RootRecordFua,
+            offset: DeviceOffsetInBytes(0),
+            contents: singlefs_harness::memory_pool::WrittenContents::Bytes(bytes),
+        }
+    }
+
+    fn publish_of(write_calls: u64, instance: u32, checkpoint_txg: u64) -> PublishInTheMountWrites {
+        PublishInTheMountWrites {
+            write_calls,
+            root: (InstanceGeneration(instance), CheckpointTxg(checkpoint_txg)),
+        }
+    }
+
+    /// 挂载续做里为空间推的抬 F（D8（核心索引结构） 已定项 16 第 6 条，用户 2026-10-03 定「挂载续做被空间拒：同样推抬 F 再试」）：
+    /// 那一串夹在续做的两步之间、不在挂载写表末尾，界按它在写表里的位置取。造的写表：取号；写行（实例 2、txg 10）；暖机一次（txg 11）；
+    /// 续做第一步（txg 12）；续做推的一串抬 F（先把新 F 写进系统配置，再一次空发布 txg 13）；续做第二步（txg 14）。
+    /// 判别力：只认挂载准入推的那几串（`singlefs_harness::mount_transparency` 那一份的做法）时这一形交「没推」，
+    /// 二次崩溃崩在那一串里时 F 照「没推」判。
+    #[test]
+    fn a_floor_raise_pushed_inside_the_continuation_is_located_between_the_continuation_steps() {
+        let writes = vec![
+            synthetic_write(StepKind::SystemConfigurationSlot, None),
+            synthetic_write(StepKind::UnitWrite, None),
+            synthetic_write(StepKind::JournalRecord, None),
+            synthetic_write(StepKind::RootRecordFua, Some((2, 10))),
+            synthetic_write(StepKind::SystemConfigurationSlot, None),
+            synthetic_write(StepKind::UnitWrite, None),
+            synthetic_write(StepKind::RootRecordFua, Some((2, 11))),
+            synthetic_write(StepKind::SystemConfigurationSlot, None),
+            synthetic_write(StepKind::UnitWrite, None),
+            synthetic_write(StepKind::JournalRecord, None),
+            synthetic_write(StepKind::RootRecordFua, Some((2, 12))),
+            synthetic_write(StepKind::SystemConfigurationSlot, None),
+            synthetic_write(StepKind::SystemConfigurationSlot, None),
+            synthetic_write(StepKind::UnitWrite, None),
+            synthetic_write(StepKind::RootRecordFua, Some((2, 13))),
+            synthetic_write(StepKind::SystemConfigurationSlot, None),
+            synthetic_write(StepKind::UnitWrite, None),
+            synthetic_write(StepKind::JournalRecord, None),
+            synthetic_write(StepKind::RootRecordFua, Some((2, 14))),
+            synthetic_write(StepKind::SystemConfigurationSlot, None),
+        ];
+        let floor_raises = FloorRaisesOfTheMount {
+            pushed_by_the_admission: Vec::new(),
+            continuation_steps: vec![publish_of(4, 2, 12), publish_of(4, 2, 14)],
+            pushed_by_the_continuation: vec![FloorRaiseInTheMountWrites {
+                system_configuration_write_calls: 1,
+                publishes: vec![publish_of(3, 2, 13)],
+            }],
+        };
+        let located = floor_raises_located_in_the_mount_writes(&floor_raises, &writes);
+        assert_eq!(
+            located,
+            FloorRaisesInTheMountWrites::FirstWriteAt(12),
+            "续做推的那一串从它把新 F 写进系统配置的那个写（第 12 个）起，后面还有续做第二步的 4 个写"
+        );
+        let mut persisted_through_the_first_continuation_step = vec![false; writes.len()];
+        persisted_through_the_first_continuation_step[..12].fill(true);
+        assert_eq!(
+            located.at_a_second_crash(&persisted_through_the_first_continuation_step),
+            FloorRaisesAtTheSecondCrash::NoneLandedYet,
+            "崩在续做第一步之后、抬 F 那一串之前：F 一步都还没落"
+        );
+        let mut persisted_into_the_floor_raise = persisted_through_the_first_continuation_step;
+        persisted_into_the_floor_raise[12] = true;
+        assert_eq!(
+            located.at_a_second_crash(&persisted_into_the_floor_raise),
+            FloorRaisesAtTheSecondCrash::SomeMayHaveLanded,
+            "崩在续做推的那一串里：新 F 可能落了一部分，F 不照「没推」判"
+        );
+    }
+
+    /// 挂载准入推了抬 F、之后又有续做：准入推的那几串不在写表末尾（后面接着续做的各步），界要把续做的写也算进去。
+    /// 造的写表：取号；写行（txg 10）；暖机（txg 11）；准入推的一串抬 F（系统配置写、空发布 txg 12）；续做一步（txg 13）。
+    /// 判别力：按「抬 F 那几串在写表末尾」取界时界落在续做那一步的单元写上，交「界取不出」，这一次挂载的二次崩溃整段 F 不判。
+    #[test]
+    fn floor_raises_pushed_by_the_mount_admission_are_located_before_the_continuation_that_follows_them(
+    ) {
+        let writes = vec![
+            synthetic_write(StepKind::SystemConfigurationSlot, None),
+            synthetic_write(StepKind::UnitWrite, None),
+            synthetic_write(StepKind::JournalRecord, None),
+            synthetic_write(StepKind::RootRecordFua, Some((2, 10))),
+            synthetic_write(StepKind::SystemConfigurationSlot, None),
+            synthetic_write(StepKind::UnitWrite, None),
+            synthetic_write(StepKind::RootRecordFua, Some((2, 11))),
+            synthetic_write(StepKind::SystemConfigurationSlot, None),
+            synthetic_write(StepKind::SystemConfigurationSlot, None),
+            synthetic_write(StepKind::UnitWrite, None),
+            synthetic_write(StepKind::RootRecordFua, Some((2, 12))),
+            synthetic_write(StepKind::SystemConfigurationSlot, None),
+            synthetic_write(StepKind::UnitWrite, None),
+            synthetic_write(StepKind::JournalRecord, None),
+            synthetic_write(StepKind::RootRecordFua, Some((2, 13))),
+            synthetic_write(StepKind::SystemConfigurationSlot, None),
+        ];
+        let floor_raises = FloorRaisesOfTheMount {
+            pushed_by_the_admission: vec![FloorRaiseInTheMountWrites {
+                system_configuration_write_calls: 1,
+                publishes: vec![publish_of(3, 2, 12)],
+            }],
+            continuation_steps: vec![publish_of(4, 2, 13)],
+            pushed_by_the_continuation: Vec::new(),
+        };
+        assert_eq!(
+            floor_raises_located_in_the_mount_writes(&floor_raises, &writes),
+            FloorRaisesInTheMountWrites::FirstWriteAt(8),
+            "准入推的那一串从第 8 个写（新 F 写进系统配置）起，后面是它的空发布与续做那一步"
+        );
+        let nothing_pushed = FloorRaisesOfTheMount {
+            pushed_by_the_admission: Vec::new(),
+            continuation_steps: vec![publish_of(4, 2, 13)],
+            pushed_by_the_continuation: Vec::new(),
+        };
+        assert_eq!(
+            floor_raises_located_in_the_mount_writes(&nothing_pushed, &writes),
+            FloorRaisesInTheMountWrites::NotPushed,
+            "准入与续做都没推：没推"
+        );
+    }
+
+    fn writes_of_kinds(kinds: &[WrittenStructureKind]) -> WritesByStructureKind {
+        let mut writes = WritesByStructureKind::NOTHING_WRITTEN;
+        for kind in kinds {
+            writes.count_write_call(*kind, &[0u8; 512]);
+        }
+        writes
+    }
+
+    /// U12（`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：挂载在续做里交回 `MountError::Publish` 时，
+    /// 按错里交回的已落盘发布写账划出续做段。写账照 2026-10-03 在副本上量到的种类造：写行（实例表、树表、分配记录树……）、
+    /// 暖机（分配记录树、记账树……）、续做那一步（extent 树、inode 树根……）。
+    /// 判别力：错的那一支交 None 时续做的写全归暖机段（攻方 A4）；不认续做的种类时同样交 None。
+    #[test]
+    fn a_mount_that_handed_back_an_error_inside_the_continuation_marks_the_continuation_by_the_persisted_publishes(
+    ) {
+        let row_publish = writes_of_kinds(&[
+            WrittenStructureKind::AllocationRecordTreeNode,
+            WrittenStructureKind::TreeTableUnit,
+            WrittenStructureKind::InstanceTableUnit,
+            WrittenStructureKind::JournalRecord,
+            WrittenStructureKind::RootSlot,
+            WrittenStructureKind::SystemConfigurationSlot,
+        ]);
+        let warm_up = writes_of_kinds(&[
+            WrittenStructureKind::AllocationRecordTreeNode,
+            WrittenStructureKind::AccountingTreeNode,
+            WrittenStructureKind::TreeTableUnit,
+            WrittenStructureKind::JournalRecord,
+            WrittenStructureKind::RootSlot,
+            WrittenStructureKind::SystemConfigurationSlot,
+        ]);
+        let continuation_step = writes_of_kinds(&[
+            WrittenStructureKind::ExtentTreeNode,
+            WrittenStructureKind::InodeTreeRoot,
+            WrittenStructureKind::AllocationRecordTreeNode,
+            WrittenStructureKind::JournalRecord,
+            WrittenStructureKind::RootSlot,
+            WrittenStructureKind::SystemConfigurationSlot,
+        ]);
+        assert_eq!(
+            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
+                &[
+                    row_publish.clone(),
+                    warm_up.clone(),
+                    warm_up.clone(),
+                    continuation_step.clone(),
+                    continuation_step.clone()
+                ],
+                &[],
+            ),
+            Some(3),
+            "写行、两次暖机之后的第一步续做起是续做"
+        );
+        assert_eq!(
+            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
+                &[row_publish.clone(), warm_up.clone()],
+                std::slice::from_ref(&continuation_step),
+            ),
+            Some(2),
+            "续做第一步落盘阶段失败：续做从失败那一次起"
+        );
+        assert_eq!(
+            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
+                std::slice::from_ref(&row_publish),
+                std::slice::from_ref(&warm_up),
+            ),
+            None,
+            "错出在暖机里：没有续做，写行之后都算暖机"
+        );
+        assert_eq!(
+            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
+                &[],
+                std::slice::from_ref(&row_publish),
+            ),
+            None,
+            "写行那次就失败：没有续做"
+        );
+        let phases_of_a_mount_that_failed_in_the_second_continuation_step =
+            writable_mount_phase_of_each_write(
+                &[
+                    synthetic_write(StepKind::SystemConfigurationSlot, None),
+                    synthetic_write(StepKind::UnitWrite, None),
+                    synthetic_write(StepKind::RootRecordFua, Some((2, 10))),
+                    synthetic_write(StepKind::UnitWrite, None),
+                    synthetic_write(StepKind::RootRecordFua, Some((2, 11))),
+                    synthetic_write(StepKind::UnitWrite, None),
+                    synthetic_write(StepKind::RootRecordFua, Some((2, 12))),
+                    synthetic_write(StepKind::UnitWrite, None),
+                ],
+                publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
+                    &[row_publish, warm_up, continuation_step.clone()],
+                    std::slice::from_ref(&continuation_step),
+                ),
+            );
+        assert_eq!(
+            phases_of_a_mount_that_failed_in_the_second_continuation_step[5..],
+            [
+                WritableMountPhase::Continuation,
+                WritableMountPhase::Continuation,
+                WritableMountPhase::Continuation
+            ],
+            "续做那两步（落盘的一步与失败那一步写出的单元）的写落在续做段，不并进暖机段"
+        );
+    }
+
     /// I-8.4 的 R-换 在真的二次崩溃上不误红：从第一个文件起的历史（起点就是攻方 `research/prompts/m3-inv-a-r1-opus-output.md`
     /// 一·2 的「新池新建文件做完的镜像」），起点那一段也摆崩溃状态；每个二次崩溃镜像上都再挂了一次，取号、写行、暖机三段
     /// 各有判过的，读出的内容都与没崩过的那一次挂载之后读出的相同（攻方量到实例代号、实例表行、扇区都不同的那种也算相同）。
diff --git a/crates/singlefs-checker-tier/src/device_log.rs b/crates/singlefs-checker-tier/src/device_log.rs
index abe99a2..6522d45 100644
--- a/crates/singlefs-checker-tier/src/device_log.rs
+++ b/crates/singlefs-checker-tier/src/device_log.rs
@@ -838,4 +838,46 @@ mod tests {
             "最后一块里先到的那一条伸出段尾：一条都不折"
         );
     }
+
+    /// 声明了一段长度 0 的清零（LR7，`research/prompts/m3-leftover-code-r1-main-verification.md` 第三节）：盘上没有条目可折，
+    /// 铺满的判定交回「铺不满」，整份日志原样留着。判别力：去掉长度 0 的守卫时铺满的判定交回「用掉 0 条」，
+    /// `fold_declared_zero_fills` 原地不动、一直往结果里推（正推腿 `k4/mut_noguard` 在 1G 的内存包装里被杀）；
+    /// 先判铺满那一步，这一条红在第一句断言上，不跑到不终止的那一步。
+    #[test]
+    fn a_declared_zero_fill_of_length_zero_tiles_no_entry_and_leaves_the_log_unchanged() {
+        let fill_start = 4 * ZERO_FILL_CHUNK_BYTES;
+        let declared = vec![(DeviceOffsetInBytes(fill_start), 0)];
+        let observed = vec![zero_write(fill_start, ZERO_FILL_CHUNK_BYTES)];
+        assert_eq!(
+            entries_tiling_one_zero_fill(&observed, 0, DeviceOffsetInBytes(fill_start), 0),
+            None,
+            "长度 0 的清零在盘上没有条目可折：铺不满，不交回「用掉 0 条」"
+        );
+        assert_eq!(
+            fold_declared_zero_fills(&observed, &declared),
+            observed,
+            "声明长度 0 的那一段一条都不折"
+        );
+    }
+
+    /// 段长 2.5 块：前两块按序到，最后一条接在已铺满的末端上、却伸出段尾 1/4 块（LR7：接续写越过段尾）。越过段尾，一条都不折。
+    /// 判别力：接在末端的那一条不查段尾时它被并进来，已铺满的末端越过段尾，函数末尾「已铺满的末端停在段尾」那一句断言 panic。
+    #[test]
+    fn zero_fill_whose_piece_continuing_at_the_covered_end_runs_past_the_end_does_not_fold() {
+        let quarter_chunk = ZERO_FILL_CHUNK_BYTES / 4;
+        let fill_start = 4 * ZERO_FILL_CHUNK_BYTES;
+        let fill_length = 2 * ZERO_FILL_CHUNK_BYTES + 2 * quarter_chunk;
+        let declared = vec![(DeviceOffsetInBytes(fill_start), fill_length)];
+        let last_chunk_start = fill_start + 2 * ZERO_FILL_CHUNK_BYTES;
+        let observed = vec![
+            zero_write(fill_start, ZERO_FILL_CHUNK_BYTES),
+            zero_write(fill_start + ZERO_FILL_CHUNK_BYTES, ZERO_FILL_CHUNK_BYTES),
+            zero_write(last_chunk_start, 3 * quarter_chunk),
+        ];
+        assert_eq!(
+            fold_declared_zero_fills(&observed, &declared),
+            observed,
+            "最后一条接在末端、伸出段尾：一条都不折"
+        );
+    }
 }
diff --git a/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs b/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs
index c0e3710..a893596 100644
--- a/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs
+++ b/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs
@@ -17,6 +17,7 @@ use singlefs_checker_tier::crash_injection::{
     run_crash_injection_campaign, CrashInjectionAssertions, CrashInjectionCampaign,
     CrashInjectionReport, CrashInjectionTally, CrashInjectionWorkerThreads, CrashPoint,
     CrashPointDraw, CrashPointFinding, CrashStateStage, FailureImageRetention,
+    HistoryCrashInjection,
 };
 use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
 use singlefs_core::admission::SpaceAdmission;
@@ -233,6 +234,12 @@ fn a_batch_that_judged_no_crash_state_is_red_under_the_new_findings_only_asserti
 fn assert_the_small_fast_tier_ran_each_stage_on_every_crash_state(report: &CrashInjectionReport) {
     let replay = how_to_replay(report);
     let tally = &report.tally;
+    // 每段历史都摆出过崩溃状态（总数够放得过「几段一个都没有、别的段多摆」，判决 `m2-supp3-item3-code-r2-main-verification.md`
+    // 第四节第 4 题，用户定「全补」；为什么每段都摆得出见快档那一组）。
+    assert_eq!(
+        tally.histories_without_any_crash_point, 0,
+        "有历史段一个崩溃状态都没摆出来；{replay}"
+    );
     assert!(
         tally.crash_points >= SMALL_FAST_TIER_SEEDS,
         "平均每段连一个崩溃状态都没摆出来：{}；{replay}",
@@ -297,6 +304,14 @@ fn fast_tier_campaign(worker_threads: CrashInjectionWorkerThreads) -> CrashInjec
 fn assert_every_crash_injection_path_was_exercised(report: &CrashInjectionReport) {
     let tally = &report.tally;
     let replay = how_to_replay(report);
+    // 总数够不说明每段都摆到了：`crash_points >= FAST_TIER_SEEDS` 放得过「几段一个都没有、别的段多摆」（判决
+    // `m2-supp3-item3-code-r2-main-verification.md` 第四节第 4 题，用户定「全补」）。从第一个文件起的历史，起点那一段（取号、暖机、
+    // 第一个文件）就有写、都是候选段；从 mkfs 起的历史要等第一次做成的可写挂载，关着会话时只抽可写挂载与冷启动两种，
+    // 一次可写挂载都没做成就收尾的（新发现或已知红收尾在那之前）这一条照判红。推的，没在快档上跑过。
+    assert_eq!(
+        tally.histories_without_any_crash_point, 0,
+        "有历史段一个崩溃状态都没摆出来；{replay}"
+    );
     assert!(
         tally.crash_points >= FAST_TIER_SEEDS,
         "平均每段连一个崩溃状态都没摆出来：{}；{replay}",
@@ -584,6 +599,95 @@ fn crash_states_of_the_history_that_raised_the_floor_into_a_rollback_gap_are_cle
     }
 }
 
+/// 各类操作的写上摆到几个崩溃状态（判决 `m2-supp3-item3-code-r2-main-verification.md` 第四节第 4 题，用户定「全补」：七类操作各钉一条；
+/// 这之后又加了崩溃恢复抛弃根，共八类）。覆盖写与可写挂载由快档钉、回退与抬 F 由上一条写死历史钉，剩下四类钉在这一段写死的历史上：
+/// mkfs → 可写挂载 → 零单元发布（树表 0 条的一版上才调得成）→ 冷启动 → 可写挂载 → 第一个文件 → 崩溃恢复抛弃根（C554 乙拒）→ 可写挂载。
+/// 写数不超过 2 的段整段枚举（2^n − 1 个真子集），更长的段各抽 1 个，所以每一类摆到几个是定数，不靠抽样运气：
+/// 零单元发布的段是 2、1、2 ⇒ 3 + 1 + 3 = 7；第一个文件的段是 24、2、1、2 ⇒ 1 + 3 + 1 + 3 = 8；
+/// 冷启动只读盘、一个写都不发，崩溃恢复抛弃根在取号之前被拒、也不发写 ⇒ 两类都是 0（崩溃状态按第一个被扣下的写落在哪一步归类，
+/// 不发写的那一步归不到）。段序列是 2026-10-03 在 harness 侧按同一段历史、同一个执行（`execute_history_with` 加
+/// `writes_and_segments_with_stream_indexes`）现量的，这一条在 checker 档里没在交回之前跑过。
+/// 比已有的两段写死历史多罩的：树表 0 条的一版上的零单元发布、冷启动之后与崩溃恢复被拒之后的可写挂载那几段的崩溃状态。
+#[test]
+fn crash_states_land_in_zero_unit_publishes_and_first_files_but_never_in_cold_starts_or_refused_crash_recoveries(
+) {
+    let history = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
+        operations: vec![
+            HistoryOperation::CloseAndMountWritable,
+            HistoryOperation::PublishWithoutUnits,
+            HistoryOperation::ColdStartRecover,
+            HistoryOperation::CloseAndMountWritable,
+            HistoryOperation::PublishFirstFile(ContentChoice {
+                length: ContentLength::InsideOneDataUnit { selector: 3 },
+                fill_seed: 1,
+            }),
+            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,
+            HistoryOperation::CloseAndMountWritable,
+        ],
+    };
+    let crash_recovery_outcomes = outcomes_of_the_crash_recovery_steps(&history);
+    assert_eq!(
+        crash_recovery_outcomes,
+        vec![StepOutcome::Refused {
+            member: CRASH_RECOVERY_REFUSED_BY_THE_WITNESS.to_string()
+        }],
+        "崩溃恢复抛弃根那一步被 C554 乙拒：它归到 0 个崩溃状态说的是「被拒的那一步不发写」"
+    );
+    let injection = inject_crashes_into_history(
+        &history,
+        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+        CrashPointDraw::EveryProperSubsetOfShortSegments {
+            segment_writes: 2,
+            sampled_in_longer_segments: 1,
+        },
+        None,
+    );
+    print_uncaptured(&format!(
+        "── 崩溃注入：零单元发布、第一个文件、冷启动与崩溃恢复抛弃根各摆几个崩溃状态 ──\n{}",
+        injection.tally.render()
+    ));
+    let crash_points_in = |kind: HistoryOperationKind| {
+        injection
+            .tally
+            .crash_points_by_operation_kind
+            .get(&kind)
+            .copied()
+            .unwrap_or(0)
+    };
+    assert_eq!(
+        crash_points_in(HistoryOperationKind::ColdStartRecover),
+        0,
+        "冷启动一个写都不发，崩溃状态归不到它：{:?}",
+        injection.tally.crash_points_by_operation_kind
+    );
+    assert_eq!(
+        crash_points_in(HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot),
+        0,
+        "崩溃恢复抛弃根在取号之前被拒、一个写都不发，崩溃状态归不到它：{:?}",
+        injection.tally.crash_points_by_operation_kind
+    );
+    assert_eq!(
+        crash_points_in(HistoryOperationKind::PublishWithoutUnits),
+        7,
+        "零单元发布的段 2、1、2：3 + 1 + 3：{:?}",
+        injection.tally.crash_points_by_operation_kind
+    );
+    assert_eq!(
+        crash_points_in(HistoryOperationKind::PublishFirstFile),
+        8,
+        "第一个文件的段 24、2、1、2：1 + 3 + 1 + 3：{:?}",
+        injection.tally.crash_points_by_operation_kind
+    );
+    // 计数先钉、判定后钉：计数只看段序列与归步，少一道屏障那类变异先红在上面的计数上（`crates/mutations.tsv`）。
+    assert!(
+        injection.new_findings.is_empty(),
+        "一条清单外的失败都不许有：{:#?}",
+        injection.new_findings
+    );
+}
+
 /// 一段历史上写出的每次发布：它写出的单元写（写表下标）与它的根身份 (实例代号, checkpoint_txg)。
 /// 归法与记录核对器的相同：一次根槽写之前、上一次根槽写之后的单元写都归这一次。
 fn unit_writes_of_each_publish(writes: &[RetainedWrite]) -> Vec<((u32, u64), Vec<usize>)> {
@@ -1317,3 +1421,283 @@ fn replay_lines_of_the_rollback_sampling_run_the_same_campaign_on_the_large_tier
         &rollback_sampling_campaign(CrashInjectionWorkerThreads::GivenByCaller(1)),
     );
 }
+
+/// 记录核对器两条判据的靶向用例（代码三方 m2-supp3-item3-code-r2 判决第四节第 1 题，用户 2026-09-28 选「靶向用例 + 变异行」；
+/// 攻方报告第四节改法甲、丙）用的那段历史：起点发完第一个文件 (1, 3)，再覆盖写两次 (1, 4)、(1, 5)。
+/// 两块 4 GiB 的盘上这段历史的录制流切成 24 段（2026-10-03 在 harness 侧现量的）：第 12 段起是 (1, 3) 的单元写
+/// （段内第 0、1 个是它的数据单元在两块盘上的两份），第 21 段是 (1, 5) 的两份 journal 记录，第 22 段是 (1, 5) 的根槽写；
+/// 从第 12 段的第一个写数到 (1, 4) 的根槽写共 56 个写。两条用例都先核这几处，段形变了先在那里红，不悄悄判别的状态。
+fn history_of_a_first_file_and_two_overwrites() -> GeneratedHistory {
+    GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations: vec![overwrite_choosing(5), overwrite_choosing(7)],
+    }
+}
+
+/// 一次覆盖写：内容落在一个数据单元里，长度与字节都随 `selector` 变（每一次都换下上一版的数据单元）。
+fn overwrite_choosing(selector: u64) -> HistoryOperation {
+    HistoryOperation::PublishOverwrite(ContentChoice {
+        length: ContentLength::InsideOneDataUnit { selector },
+        fill_seed: selector,
+    })
+}
+
+/// 这段历史在崩溃注入那一种执行下的写表与切段（与 `inject_crashes_into_history` 自己切的同一份：同一条执行、同一份几何）。
+fn writes_and_segments_of(history: &GeneratedHistory) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
+    let execution = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES;
+    let stream = SharedStream::retaining_contents();
+    let run = execute_history_with(history, execution, &stream, &mut |_| {});
+    assert_eq!(run.ending, HistoryEnding::Completed, "这段历史跑完");
+    writes_and_segments(
+        &stream.retained_operations(),
+        &execution.device_width.fixed_geometry(),
+    )
+}
+
+/// 崩溃镜像那一截（只读恢复、模型、池级 checker、记录核对器）判红记下的那一条新发现；没有时 panic，带上全部新发现。
+fn finding_on_the_crash_image(injection: &HistoryCrashInjection) -> &CrashPointFinding {
+    injection
+        .new_findings
+        .iter()
+        .find(|finding| finding.stage == CrashStateStage::CrashImage)
+        .unwrap_or_else(|| panic!("崩溃镜像那一截没有判红：{:#?}", injection.new_findings))
+}
+
+/// 记录核对器第一条判据（根在而那次发布的记录一条都不在）接进崩溃注入之后真的判得出东西（K3-④）：
+/// 第 21 段起数 3 个写（(1, 5) 的两份 journal 记录与它的根槽写），两份记录扣下、根槽写持久——越过了记录与根之间那道屏障，
+/// 屏障守住时摆不出来。恢复落到 (1, 5)、读回它的内容，模型认、池级 checker 不红（2026-10-03 在 harness 侧照同一个持久集合现量的），
+/// 这个崩溃状态只有记录核对器判得出。把崩溃镜像那一截通过条件里的 `record_check == RecordCheck::default()` 丢掉、
+/// 计数照记（攻方变异 A，`crates/mutations.tsv`），这一截就没有新发现，这一条红。
+#[test]
+fn a_crash_state_with_the_root_but_neither_of_its_records_is_a_finding_of_the_record_checker_alone()
+{
+    let history = history_of_a_first_file_and_two_overwrites();
+    let (writes, segments) = writes_and_segments_of(&history);
+    let first_write = segments[21][0];
+    assert_eq!(
+        writes[first_write..first_write + 3]
+            .iter()
+            .map(|write| write.kind)
+            .collect::<Vec<_>>(),
+        vec![
+            StepKind::JournalRecord,
+            StepKind::JournalRecord,
+            StepKind::RootRecordFua
+        ],
+        "第 21 段起的 3 个写是一次发布的两份 journal 记录与它的根槽写"
+    );
+    assert_eq!(
+        root_identity_of_a_root_slot_write(&writes[first_write + 2]),
+        Some((InstanceGeneration(1), CheckpointTxg(5))),
+        "那个根槽写是第二次覆盖写 (1, 5) 的"
+    );
+
+    let injection = inject_crashes_into_history(
+        &history,
+        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+        CrashPointDraw::ExactlyOneCrashState {
+            segment_index: 21,
+            writes_from_the_segment_on: 3,
+            withheld_among_them: &[0, 1],
+        },
+        None,
+    );
+    print_uncaptured(&format!(
+        "── 崩溃注入：记录核对器第一条判据的靶向崩溃状态 ──\n{}",
+        injection.tally.render()
+    ));
+    assert_eq!(injection.crash_points.len(), 1, "只摆写死的那一个崩溃状态");
+    assert_eq!(
+        injection.tally.record_root_without_record, 1,
+        "记录核对器在这个崩溃状态上判出「根在而它那次发布的记录一条都不在」"
+    );
+    let finding = finding_on_the_crash_image(&injection);
+    assert_eq!(
+        finding.signature,
+        FailureSignature::RecordCheck {
+            aspects: vec!["root_without_record"]
+        },
+        "崩溃镜像那一截的新发现记在记录核对器头上：{finding:#?}"
+    );
+    assert!(
+        finding.observation.violations.is_empty()
+            && finding.observation.model_disagreement.is_none(),
+        "池级 checker、RefFS 与模型都不红，这一条新发现只来自记录核对器：{finding:#?}"
+    );
+}
+
+/// 记录核对器第二条判据（恢复自称的那一版不早于某次发布、而那次发布写出的某个单元两份都不在）接进崩溃注入之后真的判得出东西（K3-⑨）：
+/// 第 12 段起数 56 个写（(1, 3) 的单元写起、到 (1, 4) 的根槽写为止），扣下 (1, 3) 的数据单元两份、别的都持久——
+/// 越过了 (1, 3) 自己单元与记录之间、记录与根之间两道屏障。恢复落到 (1, 4)（它的根槽写持久了），读回它的内容，模型认。
+/// (1, 3) 的数据单元已被 (1, 4) 换下、还没被复用，盘上那两处一直是空的，记录核对器判「恢复自称新态而单元缺席」。
+/// 照常点名验证的恢复下，只扣单元不越过根的状态落回上一版，这一条判据判不到（K3-⑨ 调查），所以这里让根也持久。
+/// 池级 checker 也红（根环里还留着 (1, 3) 的根，从它出发的遍历读不到那个数据单元），签名照「记录核对器先于 checker」取记录核对器；
+/// 只有记录核对器红的那一个状态是下一条用例。
+/// 把崩溃镜像那一截交给记录核对器的「恢复落到的那一版」换成 None（`crates/mutations.tsv`），第二条判据的前置不成立，这一条红。
+#[test]
+fn a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_finding_of_the_record_checker(
+) {
+    let history = history_of_a_first_file_and_two_overwrites();
+    let (writes, segments) = writes_and_segments_of(&history);
+    let first_write = segments[12][0];
+    let withheld_copies = &writes[first_write..first_write + 2];
+    assert!(
+        withheld_copies
+            .iter()
+            .all(|write| write.kind == StepKind::UnitWrite
+                && write.length_in_bytes() == DATA_UNIT_BYTES
+                && write.offset == withheld_copies[0].offset)
+            && withheld_copies[0].device != withheld_copies[1].device,
+        "第 12 段的头两个写是同一个数据单元在两块盘上的两份：{withheld_copies:?}"
+    );
+    assert_eq!(
+        writes[first_write + 2..first_write + 56]
+            .iter()
+            .filter_map(root_identity_of_a_root_slot_write)
+            .collect::<Vec<_>>(),
+        vec![
+            (InstanceGeneration(1), CheckpointTxg(3)),
+            (InstanceGeneration(1), CheckpointTxg(4))
+        ],
+        "从第 12 段起数的 56 个写里有 (1, 3) 与 (1, 4) 的根槽写，最后一个是 (1, 4) 的"
+    );
+    assert_eq!(
+        root_identity_of_a_root_slot_write(&writes[first_write + 55]),
+        Some((InstanceGeneration(1), CheckpointTxg(4))),
+        "从第 12 段起数的第 56 个写是 (1, 4) 的根槽写"
+    );
+
+    let injection = inject_crashes_into_history(
+        &history,
+        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+        CrashPointDraw::ExactlyOneCrashState {
+            segment_index: 12,
+            writes_from_the_segment_on: 56,
+            withheld_among_them: &[0, 1],
+        },
+        None,
+    );
+    print_uncaptured(&format!(
+        "── 崩溃注入：记录核对器第二条判据的靶向崩溃状态 ──\n{}",
+        injection.tally.render()
+    ));
+    assert_eq!(injection.crash_points.len(), 1, "只摆写死的那一个崩溃状态");
+    assert_eq!(
+        injection.tally.record_claimed_state_missing_unit, 1,
+        "记录核对器在这个崩溃状态上判出「恢复自称新态而单元缺席」"
+    );
+    let finding = finding_on_the_crash_image(&injection);
+    assert_eq!(
+        finding.signature,
+        FailureSignature::RecordCheck {
+            aspects: vec!["claimed_state_missing_unit"]
+        },
+        "崩溃镜像那一截的新发现记在记录核对器第二条判据头上：{finding:#?}"
+    );
+    assert!(
+        finding.observation.model_disagreement.is_none(),
+        "恢复落到 (1, 4) 并读回它的内容，模型认：{finding:#?}"
+    );
+}
+
+/// 第一个文件之后接着覆盖写的次数：根环三个区域各 8 槽，第 24 次覆盖写 (1, 27) 的根槽写盖掉 (1, 3) 的那一槽。
+const OVERWRITES_THAT_TURN_THE_ROOT_RING_PAST_THE_FIRST_FILE: u64 = 24;
+
+/// 第一个文件 (1, 3) 之后覆盖写 24 次：根环转过一圈，(1, 3) 的根槽已被 (1, 27) 盖掉。
+fn history_of_a_first_file_and_overwrites_that_turn_the_root_ring() -> GeneratedHistory {
+    GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations: (0..OVERWRITES_THAT_TURN_THE_ROOT_RING_PAST_THE_FIRST_FILE)
+            .map(|overwrite_index| overwrite_choosing(5 + 2 * overwrite_index))
+            .collect(),
+    }
+}
+
+/// 记录核对器第二条判据只有它自己判得出的那一个状态（第二批报告 `research/prompts/m2-rest-batch2/report.md`「停下交主 agent 的设计问题」
+/// 第 2 条说找不到：缺单元的发布只要它的根还在环里，从那条根出发的 checker 遍历就会红）。根环转过一圈之后 (1, 3) 的根不在环里了：
+/// 第 12 段起到流尾的每一个写都持久，只扣下第 12 段段内第 4、5 个写——(1, 3) 那一片 inode 树叶容器在两块盘上的两份（它已被 (1, 4)
+/// 换下，之后没有一个写落在那两处）。恢复落到 (1, 27)、读回它的内容，模型认，池级 checker 不红（2026-10-03 在 harness 侧照同一个持久集合现量：
+/// 覆盖写 16 次时 (1, 3) 的根还在环里，checker 红 I-2.1、I-4.8、I-7.4；24 次时不红）；记录核对器判 (1, 3) 那个单元两份都缺席。
+/// RefFS 的崩溃时刻取持久窗口最后一个写所在那一段（流尾那一次覆盖写），读回最后一版的内容合它。
+/// 把崩溃镜像那一截通过条件里的 `record_check == RecordCheck::default()` 丢掉（攻方变异 A，`crates/mutations.tsv`），这一截就没有新发现，这一条红。
+#[test]
+fn a_crash_state_missing_a_unit_of_a_publish_whose_root_left_the_ring_is_a_finding_of_the_record_checker_alone(
+) {
+    let history = history_of_a_first_file_and_overwrites_that_turn_the_root_ring();
+    let (writes, segments) = writes_and_segments_of(&history);
+    let first_write = segments[12][0];
+    let withheld_copies = [&writes[first_write + 4], &writes[first_write + 5]];
+    assert!(
+        withheld_copies
+            .iter()
+            .all(|write| write.kind == StepKind::UnitWrite
+                && write.offset == withheld_copies[0].offset)
+            && withheld_copies[0].device != withheld_copies[1].device,
+        "第 12 段段内第 4、5 个写是同一个单元在两块盘上的两份：{withheld_copies:?}"
+    );
+    let first_file_root = writes
+        .iter()
+        .position(|write| {
+            root_identity_of_a_root_slot_write(write)
+                == Some((InstanceGeneration(1), CheckpointTxg(3)))
+        })
+        .expect("流里有第一个文件 (1, 3) 的根槽写");
+    assert!(
+        first_write + 5 < first_file_root,
+        "扣下的两份是 (1, 3) 那次发布的单元写（在它的根槽写之前）"
+    );
+    assert!(
+        writes[first_file_root + 1..].iter().any(|write| {
+            write.kind == StepKind::RootRecordFua
+                && write.device == writes[first_file_root].device
+                && write.offset == writes[first_file_root].offset
+        }),
+        "(1, 3) 的根槽之后被别的根盖掉了：根环转过一圈"
+    );
+    assert!(
+        writes[first_write + 6..].iter().all(|later| withheld_copies
+            .iter()
+            .all(|withheld| later.device != withheld.device || later.offset != withheld.offset)),
+        "扣下的那两处之后没有一个写落上去：缺席没有更晚的写来解释"
+    );
+    let writes_from_the_segment_to_the_end = writes.len() - first_write;
+    assert_eq!(
+        writes_from_the_segment_to_the_end, 793,
+        "从第 12 段的第一个写数到流尾共 793 个写"
+    );
+
+    let injection = inject_crashes_into_history(
+        &history,
+        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+        CrashPointDraw::ExactlyOneCrashState {
+            segment_index: 12,
+            writes_from_the_segment_on: 793,
+            withheld_among_them: &[4, 5],
+        },
+        None,
+    );
+    print_uncaptured(&format!(
+        "── 崩溃注入：只有记录核对器判得出的第二条判据靶向崩溃状态 ──\n{}",
+        injection.tally.render()
+    ));
+    assert_eq!(injection.crash_points.len(), 1, "只摆写死的那一个崩溃状态");
+    assert_eq!(
+        injection.tally.record_claimed_state_missing_unit, 1,
+        "记录核对器在这个崩溃状态上判出「恢复自称新态而单元缺席」"
+    );
+    let finding = finding_on_the_crash_image(&injection);
+    assert_eq!(
+        finding.signature,
+        FailureSignature::RecordCheck {
+            aspects: vec!["claimed_state_missing_unit"]
+        },
+        "崩溃镜像那一截的新发现记在记录核对器第二条判据头上：{finding:#?}"
+    );
+    assert!(
+        finding.observation.violations.is_empty()
+            && finding.observation.model_disagreement.is_none(),
+        "池级 checker、RefFS 与模型都不红，这一条新发现只来自记录核对器：{finding:#?}"
+    );
+}
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index c3e6497..0aa0c1c 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -5633,11 +5633,13 @@ pub fn publish_overwrite<Device: BlockDevice>(
 /// 只要一个数据单元的内容，写出的字节与同内容的 `publish_overwrite` 一个不差。上一版比这一版多出来的数据单元
 /// 在同一次发布里释放（文件变短的那一格，`placements_to_release_via_mapping`）。
 ///
+/// 切出的单元多于一片下段叶装得下的 144 个时不拒：这个文件的 extent 树下段按单元号按位置寻址长成两层及以上（叶罩 144 个单元、
+/// 内部扇出 147，D8（核心索引结构） 已定项 14），这一版下段的节点在同一次发布里重写、上一版的在同一次发布里释放；只有一个单元时
+/// 不建下段，那个数据指针内联在上段叶条目里。
+///
 /// # Errors
-/// 切出的单元多于一片 extent 叶装得下的记录数 ⇒ `ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided`
-/// （extent 内部条目的格式没有条款），在分配器、块设备都还没被碰过的时候返回，盘上逐字节不变。
-/// 最后一个事务要点名的项多于一条记录装得下的，末条再跨记录（D23（journal 的角色与格式） 已定项 17），不拒。其余与 `publish_overwrite` 相同
-/// （不报 `ContentExceedsDataUnit`：多个数据单元正是这条路径要写的）。
+/// 与 `publish_overwrite` 相同（不报 `ContentExceedsDataUnit`：多个数据单元正是这条路径要写的）。这条路径不按单元数拒：extent 树
+/// 没有装不下这一格；最后一个事务要点名的项多于一条记录装得下的，末条再跨记录（D23（journal 的角色与格式） 已定项 17），也不拒。
 pub fn publish_sequential_write<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
diff --git b/crates/singlefs-harness/tests/named_unit_verification_stands_in_for_the_barrier_before_the_record.rs b/crates/singlefs-harness/tests/named_unit_verification_stands_in_for_the_barrier_before_the_record.rs
new file mode 100644
index 0000000..8b72ece
--- /dev/null
+++ b/crates/singlefs-harness/tests/named_unit_verification_stands_in_for_the_barrier_before_the_record.rs
@@ -0,0 +1,400 @@
+//! 发布的「单元写 → 屏障 → journal 记录」那道屏障（`singlefs_core::transaction` 的 `persist_publish_writes` 里第一道）防住的状态——
+//! 记录已持久、它点名的某个单元两份都没落——恢复端还有第二道防线：施加记录之前逐份读点名单元核校验和（`replay_journal` 的点名验证）。
+//! 两道至少留一道（K3-⑨，用户 2026-09-28 选；事实见 `research/prompts/k3-9-mechanism/report.md`「四次运行」：删掉那道屏障，
+//! 照常点名验证时三条判据 0、验证失败 903 次；再关掉点名验证，记录核对器与模型各红 752 次）。
+//! 这里不改 core，用录制器录下一段真实历史，在持久集合里直接摆出这个状态，钉住第二道防线承重：
+//! 照常恢复落回上一版、验证失败有数、理想模型不分歧；关掉点名验证（只供测试的开关）时理想模型报分歧。
+//!
+//! 点名验证替不了这道屏障的那一处也钉在这里（C592，`research/prompts/k3-9-barrier-r1-main-verification.md` 第四节、第六节第 2 题）：
+//! 复用回收槽的历史上，池级 checker I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）射程④ 的同源前提靠这道屏障撑着。
+
+use std::collections::BTreeMap;
+use std::rc::Rc;
+
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::make_filesystem::make_filesystem;
+use singlefs_core::mount::{
+    mount_writable_with_space_admission, raise_rollback_floor, ShadowLedger,
+};
+use singlefs_core::recovery::{recover, JournalPolicy, RecoveryReport};
+use singlefs_core::transaction::{
+    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
+};
+use singlefs_harness::history::{
+    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
+    HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
+    PerStepChecker,
+};
+use singlefs_harness::memory_pool::{
+    newest_persisted_root, publishes_in, writes_and_segments, CrashImage, MemoryPool,
+    RecordStreamContinuity, RetainedWrite, SparseBlockDevice,
+};
+use singlefs_harness::model::{crash_recovery_disagreement, ModelDisagreement, ModelRootKey};
+use singlefs_harness::model_comparison::{model_root_key, observed_read_back_after_a_crash};
+use singlefs_harness::segments::StepKind;
+use singlefs_harness::{RecordingBlockDevice, SharedStream};
+
+/// 不跑每一步的池级 checker（这里只看崩溃之后那一个状态），两块 4 GiB 的盘。
+const EXECUTION: HistoryExecution = HistoryExecution {
+    per_step_checker: PerStepChecker::Skipped,
+    device_width: HistoryDeviceWidth::FourGibibytes,
+    space_admission: SpaceAdmission::JudgedByTheFormula,
+};
+
+/// 起点发完第一个文件 (1, 3)，再覆盖写一次 (1, 4)：被摆的是 (1, 4) 那次发布。
+const THE_PUBLISH_BEFORE: (InstanceGeneration, CheckpointTxg) =
+    (InstanceGeneration(1), CheckpointTxg(3));
+const THE_PUBLISH_WHOSE_UNIT_NEVER_LANDED: (InstanceGeneration, CheckpointTxg) =
+    (InstanceGeneration(1), CheckpointTxg(4));
+
+/// 录下来的这段历史与摆好的那一个崩溃状态：写表、持久集合（与写表逐条对应）、模型提交过的每一版。
+struct RecordPersistedWithoutOneOfItsUnits {
+    writes: Vec<RetainedWrite>,
+    persisted: Vec<bool>,
+    committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
+}
+
+/// 跑这段历史、录下整条流，摆出：(1, 4) 的根槽写之前的写都持久，只有它第一个单元写那个偏移上的每一份（两块盘各一份）扣下；
+/// 它的根槽写与之后的都没持久。于是 (1, 4) 的 journal 记录两份都在盘上，而它点名的一个单元两份都不在——
+/// 这正是记录之前那道屏障守住时摆不出来的状态（屏障之前的单元写没落，屏障之后的记录就不会落）。
+fn record_persisted_without_one_of_its_units() -> RecordPersistedWithoutOneOfItsUnits {
+    let history = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations: vec![HistoryOperation::PublishOverwrite(ContentChoice {
+            length: ContentLength::InsideOneDataUnit { selector: 5 },
+            fill_seed: 5,
+        })],
+    };
+    let stream = SharedStream::retaining_contents();
+    let mut committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = BTreeMap::new();
+    let run = execute_history_with(&history, EXECUTION, &stream, &mut |observation| {
+        for (key, file) in observation.model.committed_versions() {
+            committed_versions.insert(key, file);
+        }
+    });
+    assert_eq!(run.ending, HistoryEnding::Completed, "这段历史跑完");
+    let (writes, _segments) = writes_and_segments(
+        &stream.retained_operations(),
+        &EXECUTION.device_width.fixed_geometry(),
+    );
+    let every_write = vec![true; writes.len()];
+    let publishes = publishes_in(&writes, &every_write, RecordStreamContinuity::OneRecording);
+    let publish = publishes
+        .iter()
+        .find(|publish| {
+            (
+                InstanceGeneration(publish.instance),
+                CheckpointTxg(publish.checkpoint_txg),
+            ) == THE_PUBLISH_WHOSE_UNIT_NEVER_LANDED
+        })
+        .unwrap_or_else(|| {
+            panic!(
+                "流里有 {THE_PUBLISH_WHOSE_UNIT_NEVER_LANDED:?} 那次发布：{:?}",
+                publishes
+                    .iter()
+                    .map(|publish| (publish.instance, publish.checkpoint_txg))
+                    .collect::<Vec<_>>()
+            )
+        });
+    assert!(
+        !publish.units.is_empty() && !publish.records.is_empty(),
+        "覆盖写那次发布写了单元，也写了 journal 记录"
+    );
+    let withheld_offset = writes[publish.units[0]].offset;
+    let mut persisted = vec![false; writes.len()];
+    persisted[..publish.root].fill(true);
+    let mut withheld_copies = 0usize;
+    for unit in &publish.units {
+        if writes[*unit].offset == withheld_offset {
+            persisted[*unit] = false;
+            withheld_copies += 1;
+        }
+    }
+    assert_eq!(withheld_copies, 2, "扣下的是同一个单元在两块盘上的两份");
+    assert!(
+        publish.records.iter().all(|record| persisted[*record]),
+        "那次发布的 journal 记录每一份都持久"
+    );
+    RecordPersistedWithoutOneOfItsUnits {
+        writes,
+        persisted,
+        committed_versions,
+    }
+}
+
+/// 在这个崩溃状态上按 `policy` 恢复，交回恢复报告与理想模型的判定（盘上已持久的最新根槽取持久集合里的）。
+fn recover_and_ask_the_model(
+    state: &RecordPersistedWithoutOneOfItsUnits,
+    policy: JournalPolicy,
+) -> (RecoveryReport, Option<ModelDisagreement>) {
+    let empty_devices = MemoryPool::with_devices(
+        &[DeviceIdentity(0), DeviceIdentity(1)],
+        EXECUTION.device_width.device_bytes(),
+    );
+    let image = CrashImage {
+        base: &empty_devices,
+        writes: &state.writes,
+        persisted: state.persisted.clone(),
+    };
+    let report = recover(&image, policy);
+    let newest_persisted = newest_persisted_root(&state.writes, &state.persisted)
+        .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
+    assert_eq!(
+        newest_persisted,
+        Some(model_root_key(THE_PUBLISH_BEFORE.0, THE_PUBLISH_BEFORE.1)),
+        "盘上已持久的最新根槽是上一版：被摆的那次发布的根槽写没持久"
+    );
+    let disagreement = crash_recovery_disagreement(
+        &state.committed_versions,
+        newest_persisted,
+        &observed_read_back_after_a_crash(&report),
+    );
+    (report, disagreement)
+}
+
+/// 照常恢复（点名验证开着）：那条记录点名的单元读不出，验证失败、这次发布整体不施加，恢复落回上一版，理想模型认。
+/// 把点名验证短路（恢复交给前缀核的开关恒为「不验」，`crates/mutations.tsv`）就施加了那条记录、走读在缺的单元上失败，这一条红。
+#[test]
+fn with_named_unit_verification_a_record_whose_unit_never_landed_falls_back_to_the_previous_version(
+) {
+    let state = record_persisted_without_one_of_its_units();
+    let (report, disagreement) = recover_and_ask_the_model(&state, JournalPolicy::Consult);
+    assert!(
+        report.journal.verification_failed > 0,
+        "点名验证读到了缺的那个单元、判了失败：{:?}",
+        report.journal
+    );
+    assert_eq!(
+        report.journal.prefix_applied, 0,
+        "验证失败的那次发布整体不施加：{:?}",
+        report.journal
+    );
+    assert_eq!(
+        report.effective_root,
+        Some(THE_PUBLISH_BEFORE),
+        "恢复落回上一版：{:?}",
+        report.outcome
+    );
+    assert_eq!(
+        disagreement, None,
+        "理想模型认上一版（盘上已持久的最新根槽就是它）"
+    );
+}
+
+/// 关掉点名验证（只供测试的开关）：同一个状态上记录被施加、恢复自称落到那次发布，走读在缺的单元上失败，理想模型报分歧——
+/// 两道防线都拿掉时这个状态就是一次错的恢复，上一条的「落回上一版」确实是点名验证挡下来的。
+#[test]
+fn without_named_unit_verification_the_same_state_is_a_model_disagreement() {
+    let state = record_persisted_without_one_of_its_units();
+    let (report, disagreement) =
+        recover_and_ask_the_model(&state, JournalPolicy::ConsultWithoutNamedVerification);
+    assert_eq!(
+        (report.journal.prefix_applied > 0, report.effective_root),
+        (true, Some(THE_PUBLISH_WHOSE_UNIT_NEVER_LANDED)),
+        "不验就施加了那条记录、恢复自称落到那次发布：{:?}",
+        report.journal
+    );
+    assert!(
+        disagreement.is_some(),
+        "理想模型报分歧（恢复走读失败或读错）：{:?}",
+        report.outcome
+    );
+}
+
+/// 复用回收槽的那段历史（攻方 `research/prompts/k3-9-barrier-r1-opus-model/` 的 floorreuse4 场景）：两块单元区 384 槽的小盘，
+/// mkfs、挂载、第一个文件、覆盖写 4 次、抬 F 到上限（现行 txg − 3），抬 F 之后换下的槽可回收；再覆盖写一次（被判的那一次），
+/// 它的单元落在盘上已有旧单元的槽里。交回被判那一次之前的池（之前的写全持久）、被判那一次的写表与切段。
+struct FloorRaisedThenAPublishReusingReclaimedSlots {
+    base: MemoryPool,
+    writes: Vec<RetainedWrite>,
+    segments: Vec<Vec<usize>>,
+}
+
+/// 第 `round` 次发布的内容：长度不同、字节不同，每次覆盖写都换下上一版的数据单元。
+fn content_of_round(round: usize, length: usize) -> Vec<u8> {
+    (0..length)
+        .map(|index| u8::try_from((index * 13 + round * 7 + 3) % 251).expect("小于 251"))
+        .collect()
+}
+
+/// 每次发布的写入时间：照 E142 那个固定时刻起，每次往后一秒。
+const FIRST_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
+
+fn floor_raised_then_a_publish_reusing_reclaimed_slots(
+) -> FloorRaisedThenAPublishReusingReclaimedSlots {
+    let width = HistoryDeviceWidth::UnitAreaOf384Slots;
+    let parameters = width.parameters();
+    let stream = SharedStream::retaining_contents();
+    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
+        [DeviceIdentity(0), DeviceIdentity(1)]
+            .into_iter()
+            .map(|identity| {
+                (
+                    identity,
+                    RecordingBlockDevice::with_shared_stream(
+                        identity,
+                        SparseBlockDevice::new(
+                            width.device_bytes(),
+                            PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
+                        ),
+                        stream.clone(),
+                    ),
+                )
+            })
+            .collect();
+    make_filesystem(&parameters, &mut devices).expect("mkfs");
+    let mut mounted = mount_writable_with_space_admission(
+        &parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+    )
+    .expect("mkfs 之后的可写挂载");
+    let PoolVersion::WithoutFile(version_without_file) = &mounted.current else {
+        panic!("mkfs 之后挂上的那一版还没有文件");
+    };
+    let root_without_file = version_without_file.root;
+    let record_bytes_without_file = version_without_file.record_bytes.clone();
+    let instance = mounted.output.instance;
+    let first_content = content_of_round(1, 3000);
+    let mut current = {
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        publish_first_file(
+            &mut writer,
+            &mut mounted.allocator,
+            &root_without_file,
+            FirstFile {
+                content: &first_content,
+                write_time_seconds: FIRST_WRITE_TIME_SECONDS,
+            },
+            instance,
+            &record_bytes_without_file,
+        )
+        .expect("第一个文件")
+    };
+    for round in 0..4u64 {
+        let round_index = usize::try_from(round).expect("小于 4");
+        let content = content_of_round(round_index + 10, 3000 + round_index);
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        current = publish_overwrite(
+            &mut writer,
+            &mut mounted.allocator,
+            &current,
+            FirstFile {
+                content: &content,
+                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1 + round,
+            },
+            instance,
+        )
+        .expect("覆盖写");
+    }
+    // 抬 F 的上限是现行 txg − 3（攻方先试时 `RollbackFloorAboveCeiling` 报的 ceiling）。
+    let floor_at_the_ceiling = CheckpointTxg(current.root.checkpoint_txg.0 - 3);
+    raise_rollback_floor(
+        &parameters,
+        &mut devices,
+        &mut mounted.allocator,
+        &mut current,
+        floor_at_the_ceiling,
+        ShadowLedger::On,
+    )
+    .expect("抬 F 到上限");
+    let operations_before_the_judged_publish = stream.operation_count();
+    let judged_content = content_of_round(2, 4000);
+    {
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        publish_overwrite(
+            &mut writer,
+            &mut mounted.allocator,
+            &current,
+            FirstFile {
+                content: &judged_content,
+                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 5,
+            },
+            instance,
+        )
+        .expect("被判的那一次覆盖写");
+    }
+    let operations = stream.retained_operations();
+    let mut base = MemoryPool::with_devices(
+        &[DeviceIdentity(0), DeviceIdentity(1)],
+        width.device_bytes(),
+    );
+    base.apply(&operations[..operations_before_the_judged_publish]);
+    let (writes, segments) = writes_and_segments(
+        &operations[operations_before_the_judged_publish..],
+        &width.fixed_geometry(),
+    );
+    FloorRaisedThenAPublishReusingReclaimedSlots {
+        base,
+        writes,
+        segments,
+    }
+}
+
+/// C592（单独拿掉单元写与记录之间那道屏障，今天没有判据会红）的那一格：复用回收槽的历史上，被判那一次发布的每一段各扣下一个写
+/// （更早的段整段持久、这一段别的写持久、更晚的段一个都没持久），池级 checker 一条不红。这道屏障拿掉之后单元写与记录并成一段，
+/// 「复用槽上那个数据单元的一份没落、记录落了」那个状态进了枚举域，下一次挂载会先施加的那一版的分配记录与它罩住的单元不同源，
+/// I-3.10 在结局正确的状态上红（攻方 floorreuse4：删屏障 271 个状态红 48，带屏障 73 个状态 0；`crates/mutations.tsv` 删屏障那一行点名这一条）。
+// crash-case-check:not-a-crash-case 只摆被判那一次发布的单扣状态，几十个，debug 下单跑几秒
+#[test]
+fn with_the_barrier_before_the_record_a_publish_reusing_reclaimed_slots_keeps_every_single_withheld_state_free_of_checker_violations(
+) {
+    let judged = floor_raised_then_a_publish_reusing_reclaimed_slots();
+    let reused_unit_writes = judged
+        .writes
+        .iter()
+        .filter(|write| {
+            write.kind == StepKind::UnitWrite
+                && !judged.base.devices[&write.device]
+                    .written_sectors_in(write.offset, write.length_in_bytes())
+                    .is_empty()
+        })
+        .count();
+    assert!(
+        reused_unit_writes >= 1,
+        "前提：被判那一次至少有一个单元写落在盘上已写过的槽里（复用回收的槽）"
+    );
+    let mut states_judged = 0usize;
+    let mut violations_found: Vec<String> = Vec::new();
+    // 迭代上界是被判那一次发布的写数；每一轮从同一份基线起摆一个状态，不跨轮带状态。
+    for (segment_index, segment) in judged.segments.iter().enumerate() {
+        for withheld in segment {
+            let mut persisted = vec![false; judged.writes.len()];
+            for earlier_segment in &judged.segments[..segment_index] {
+                for write in earlier_segment {
+                    persisted[*write] = true;
+                }
+            }
+            for write in segment {
+                persisted[*write] = write != withheld;
+            }
+            let image = CrashImage {
+                base: &judged.base,
+                writes: &judged.writes,
+                persisted,
+            };
+            states_judged += 1;
+            for (invariant, verdict) in check_pool_image(&image) {
+                if let InvariantVerdict::Violated(detail) = verdict {
+                    violations_found.push(format!(
+                        "第 {segment_index} 段扣下第 {withheld} 个写：{invariant} {detail}"
+                    ));
+                }
+            }
+        }
+    }
+    assert_eq!(
+        states_judged,
+        judged.writes.len(),
+        "每个写各扣下一次，状态数等于写数"
+    );
+    assert!(
+        violations_found.is_empty(),
+        "带着单元写与记录之间那道屏障，单扣状态上池级 checker 红了：{violations_found:#?}"
+    );
+}
diff --git a/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs b/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs
index 5915bc2..bd132e5 100644
--- a/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs
+++ b/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs
@@ -12,7 +12,7 @@ mod common_corrupted_allocation_record;
 
 use common::{
     abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, disk_snapshot,
-    file_content, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
+    file_content, geometry, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
 };
 use common_corrupted_allocation_record::move_the_first_allocation_record_past_the_end_of_the_unit_area_and_reseal_the_chain;
 use singlefs_checker::image::InvariantVerdict;
@@ -54,7 +54,9 @@ use singlefs_harness::fault_injection::{
     injected_block_device_error, NamedRootRingSlots, PoolReaderWithUnreadableRootRingSlots,
     RootRingSlotTarget,
 };
-use singlefs_harness::memory_pool::MemoryPool;
+use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool};
+use singlefs_harness::segments::{segment_kinds_text, segment_sizes_text, split_into_segments};
+use singlefs_harness::{RecordedOperation, RecordedOperationKind};
 use std::collections::BTreeSet;
 
 const SECOND_FILE_BYTES: usize = 4100;
@@ -456,6 +458,68 @@ fn rolling_back_to_the_first_version_while_mounted_publishes_one_root_carrying_i
     );
 }
 
+/// 回退那次发布 D「每段写几个」（`layout/01-first-txn.md` 八节「实例切换 / 管理员回退」那一行，原来没有用例钉）：
+/// 从录制流上截回退那次发布（回退调用之前流里有几步，到调用返回），按层 0 同一条切段规则切（`split_into_segments`，屏障按设备记）。
+/// D 孤立看是 [24 个单元写] 屏障 [两份记录] 屏障 [根 FUA] [两份系统配置槽] 屏障：重写 12 个单元、两块盘各一份，共 29 次写。
+/// 整条流（mkfs 之后）到 D 为止 36 段，与 checker 档固定脚本那条流 `RollbackToFirstVersion` 那一臂钉的段数相同：
+/// C 的两份系统配置槽写过了发布末尾那道屏障、自成一段，D 的单元写不与它并段。
+/// 不跑 checker 与冷启动，debug 下单跑几秒，是轻用例。
+#[test]
+fn rolling_back_to_the_first_version_writes_its_units_in_one_segment_before_the_records_the_root_and_the_system_configuration(
+) {
+    let ThroughTheThirdPublish { mut pool, .. } =
+        build_through_third_publish("rollback-publish-segments");
+    let operations_before_the_rollback = pool.stream.operation_count();
+    roll_back(&mut pool, first_root()).expect("回退到 A");
+    let rollback_version = pool.output.clone();
+    let operations_of_the_rollback: Vec<RecordedOperation> =
+        pool.stream.operations()[operations_before_the_rollback..].to_vec();
+    let segments_of_the_rollback = split_into_segments(&operations_of_the_rollback, &geometry());
+    let writes_of_the_rollback = operations_of_the_rollback
+        .iter()
+        .filter(|operation| match operation.kind {
+            RecordedOperationKind::Write
+            | RecordedOperationKind::WriteForceUnitAccess
+            | RecordedOperationKind::WriteZeroes => true,
+            RecordedOperationKind::Barrier => false,
+        })
+        .count();
+    let retained = pool.retained_operations();
+    let (writes_after_mkfs, segments_after_mkfs) =
+        writes_and_segments(&retained[pool.mkfs_operation_count..], &geometry());
+    let segment_lengths_after_mkfs: Vec<usize> = segments_after_mkfs.iter().map(Vec::len).collect();
+    assert_eq!(
+        segment_sizes_text(&segments_of_the_rollback),
+        "24+2+1+2",
+        "D 孤立看的段序列：单元写一段、记录一段、根一段、系统配置一段"
+    );
+    assert_eq!(
+        segment_kinds_text(&segments_of_the_rollback),
+        "[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]",
+        "D 的单元写只有单元写一种，不取号、不写系统配置之外的原地写"
+    );
+    assert_eq!(writes_of_the_rollback, 29, "D 共 29 次写：24 + 2 + 1 + 2");
+    assert_eq!(
+        rollback_version.rewritten.len(),
+        12,
+        "D 重写 12 个单元，每块盘各一份：{:?}",
+        rollback_version.rewritten
+    );
+    assert_eq!(
+        segment_lengths_after_mkfs,
+        vec![
+            2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2, 16, 2, 1,
+            2, 24, 2, 1, 2, 24, 2, 1, 2
+        ],
+        "整条流到 D 为止的段序列：C 的系统配置槽写自成一段、D 的单元写自成一段"
+    );
+    assert_eq!(
+        writes_after_mkfs.len(),
+        166 + 29,
+        "到 C 166 次写、D 29 次写"
+    );
+}
+
 /// 验收「释放与复活」：D 之后接着覆盖写 E，E 换下的是 D 引用的 A 的那几个单元——它们在 D 的账里是仍分配的（复活过），
 /// 发布路径照常释放；checker 全绿。只释放不复活时 D 的账里 A 的单元是已释放的：I-3.11 在 D 上红，E 被释放判定路径拒
 /// （`ReleaseTargetAlreadyReleased`，第一轮判决第二节 F1 里 H3 那一格）——`crates/mutations.tsv` 里拿掉复活那一步的那一行证它。
~~~~~~

## 三、变异表

~~~~~~diff
--- crates/mutations.tsv.before-batch11
+++ crates/mutations.tsv
@@ -117,7 +117,6 @@
 增补 2 第 41 行 层 0 并行：并片时 checker 每条不变量的第一处违例取后面那一片的	crates/singlefs-checker-tier/src/crash.rs	            self.checker_first_violation\n                .entry(invariant)\n                .or_insert(detail);	            self.checker_first_violation.insert(invariant, detail);	-p singlefs-checker-tier --lib -- absorbing_a_following_slice	absorbing_a_following_slice_adds_counts_and_keeps_the_earlier_first_violation
 增补 2 第 41 行 层 0 并行：不读 SINGLEFS_LAYER0_THREADS	crates/singlefs-checker-tier/src/crash.rs	                LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE,\n                environment_value,\n            ) {	                LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE,\n                environment_value.and(None::<&str>),\n            ) {	-p singlefs-checker-tier --lib -- worker_threads_come_from_the_environment_variable	worker_threads_come_from_the_environment_variable_before_available_parallelism
 增补 2 第 41 行 层 0 并行：SINGLEFS_LAYER0_THREADS=0 悄悄退回 1 个线程	crates/singlefs-checker-tier/src/crash.rs	                LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE,\n                environment_value,\n            ) {	                LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE,\n                environment_value.map(|text| if text.trim() == "0" { "1" } else { text }),\n            ) {	-p singlefs-checker-tier --lib -- zero_worker_threads_in_the_environment_variable	zero_worker_threads_in_the_environment_variable_stops_instead_of_falling_back
-增补 3 第 1 件（代码三方第一轮第 1 条）：已知红第 1 条不看 F 是否落在回退留下的空档里	crates/singlefs-harness/src/history.rs	    observation.raised_floor_lands_only_on_abandoned_roots == Some(true)\n        && only_allocated_statistic_above_walked(observation)	    only_allocated_statistic_above_walked(observation)	-p singlefs-harness --test random_histories -- an_allocated_statistic_over_count	an_allocated_statistic_over_count_after_raising_the_floor_without_a_rollback_is_a_new_finding
 增补 3 第 1 件（代码三方第一轮第 3 条）：随机历史快档判出「复用改写已回收记录时不改分配代」（N2）	crates/singlefs-core/src/allocator.rs	            existing.generation = generation;\n            existing.is_released = false;	            existing.is_released = false;	-p singlefs-harness --test random_histories -- --include-ignored random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
 增补 3 第 1 件（代码三方第一轮第 4 条）：随机历史快档判出「checker 判绿的镜像上冷启动读回报错」（B6：补齐字节从最后一个载荷字节算起）	crates/singlefs-core/src/unit.rs	    if bytes[payload_end..].iter().any(|byte| *byte != 0) {	    if bytes[payload_end - 1..].iter().any(|byte| *byte != 0) {	-p singlefs-harness --test random_histories -- --include-ignored random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
 增补 2 第 41 行 层 0 并行（代码三方第一轮 Y5m2）：并片时「不看 journal 那一遍的第一处」取后面那一片的	crates/singlefs-checker-tier/src/crash.rs	        if self.first_ignored_violation.is_none() {\n            self.first_ignored_violation = first_ignored_violation;\n        }	        if first_ignored_violation.is_some() {\n            self.first_ignored_violation = first_ignored_violation;\n        }	-p singlefs-checker-tier --lib -- absorbing_a_following_slice_keeps_the_earlier_first_ignored_violation	absorbing_a_following_slice_keeps_the_earlier_first_ignored_violation_when_both_slices_have_one
@@ -144,7 +143,7 @@
 增补 3 第 2 件（代码三方第一轮判决第三节第 2 条，C368 仍欠的那一半）：发布层把分配器的落点拒绝一律报成「每块盘上都没有」	crates/singlefs-core/src/transaction.rs	        unit: identity,\n        refusal,\n    })\n}	        unit: identity,\n        refusal: {\n            let _ = refusal;\n            PlacementRefusal::NoFreeSlotOnAnyDevice\n        },\n    })\n}	-p singlefs-harness --test unequal_devices -- filling_the_smaller_device	filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking
 增补 3 第 2 件（代码三方第一轮判决第三节第 2 条的改法）：胶水把小盘写满、各盘落点不一致也映射成单元区墙	crates/singlefs-harness/src/model_comparison.rs	        } => ObservedRefusalReason::Unexplained,	        } => explained(ModelRefusalReason::UnitAreaWall),	-p singlefs-harness --lib -- model_comparison::tests::only_a_placement_refused	only_a_placement_refused_on_every_device_passes_as_the_unit_area_wall
 增补 3 第 2 件（代码三方第二轮判决第三节第 1 条的改法）：胶水把回退候选排除的三条揉成一条（「不在环里」也报成「低于 F」）	crates/singlefs-harness/src/model_comparison.rs	        RollbackCandidateExclusion::NotInRing => ModelRefusalReason::RollbackTargetNotInRing,	        RollbackCandidateExclusion::NotInRing => ModelRefusalReason::RollbackTargetBelowEffectiveFloor,	-p singlefs-harness --lib -- model_comparison::tests::each_rollback_candidate_exclusion	each_rollback_candidate_exclusion_maps_to_its_own_reason
-增补 3 第 2 件（代码三方第二轮判决第三节第 2 条，攻方变异 m2h）：分配器用户数据那一处把「每块盘上都没有」报成「小盘写满」；小盘段判出	crates/singlefs-core/src/allocator.rs	            DeviceAgreement::Agreed(slot) => slot,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);	            DeviceAgreement::Agreed(slot) => slot,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices: Vec::new() });	-p singlefs-harness --test random_histories -- --include-ignored unit_area_wall_sampling	unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped_runs_to_the_end_without_reaching_the_placement_wall
+增补 3 第 2 件（代码三方第二轮判决第三节第 2 条，攻方变异 m2h）：分配器用户数据那一处把「每块盘上都没有」报成「小盘写满」；小盘段判出	crates/singlefs-core/src/allocator.rs	            DeviceAgreement::Agreed(slot) => slot,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);	            DeviceAgreement::Agreed(slot) => slot,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices: Vec::new() });	-p singlefs-harness --test admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments -- with_the_space_admission_switched_off_the_session_raises_the_floor_after_a_placement_refusal	with_the_space_admission_switched_off_the_session_raises_the_floor_after_a_placement_refusal_and_publishes_the_write
 增补 3 第 2 件（代码三方第二轮判决第三节第 2 条，攻方变异 m2i）：分配器用户数据那一处把「小盘写满」报成「每块盘上都没有」；等大的小盘走不到，不等盘那条用例判出	crates/singlefs-core/src/allocator.rs	            DeviceAgreement::SomeDevicesWithoutAnswer(full_devices) => {\n                return Err(\n                    PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices },\n                );\n            }\n            DeviceAgreement::AnswersDiffer(slot_per_device) => {	            DeviceAgreement::SomeDevicesWithoutAnswer(_full_devices) => {\n                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);\n            }\n            DeviceAgreement::AnswersDiffer(slot_per_device) => {	-p singlefs-harness --test unequal_devices -- filling_the_smaller_device	filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking
 增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4a）：根环转过之后回收门槛取「环里最旧有效根 + 1」；逼近分配记录墙那一段照跑 checker 判出	crates/singlefs-core/src/mount.rs	    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))	    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(if oldest.0 > 0 { CheckpointTxg(oldest.0 + 1) } else { oldest }))	-p singlefs-harness --test random_histories -- --include-ignored allocation_record_sampling	allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall
 增补 3 第 3 件：抽 0 个崩溃点时照样抽一个（抽崩溃点会改变这段历史怎么跑，判别力那一半失效）	crates/singlefs-checker-tier/src/crash_injection.rs	            for _ in 0..crash_points_per_history {	            for _ in 0..crash_points_per_history.max(1) {	-p singlefs-checker-tier --test crash_injection_campaign -- drawing_crash_points	drawing_crash_points_does_not_change_the_generated_history
@@ -153,7 +152,6 @@
 增补 3 第 3 件（用户 2026-09-20 定案第 2 条）：段内只截前缀，不摆任意真子集（「后发的写先持久」整类零覆盖）	crates/singlefs-checker-tier/src/crash_injection.rs	    if segment_length <= WRITES_A_SUBSET_MASK_HOLDS {\n        let mask = source.below((1u64 << segment_length) - 1);\n        return bits_of(mask, segment_length);\n    }\n    let mut persisted: Vec<bool> = (0..segment_length).map(|_| source.below(2) == 1).collect();\n    if persisted.iter().all(|is_persisted| *is_persisted) {\n        let withheld = usize::try_from(source.below(u64::try_from(segment_length).expect("段长")))\n            .expect("下标装得进 usize");\n        persisted[withheld] = false;\n    }\n    persisted	    let persisted_prefix = source.below(u64::try_from(segment_length).expect("段长"));\n    (0..segment_length)\n        .map(|write| u64::try_from(write).expect("下标") < persisted_prefix)\n        .collect()	-p singlefs-checker-tier --lib -- crash_injection::tests::crash_points_withhold_every_kind_of_write	crash_points_withhold_every_kind_of_write_and_leave_holes_inside_a_segment
 增补 3 第 3 件（用户 2026-09-20 定案第 3 条）：崩溃状态上不跑记录核对器	crates/singlefs-checker-tier/src/crash_injection.rs	        let record_check = check_records_against(\n            &image,\n            &image,\n            &writes,\n            &persisted,\n            RecordStreamContinuity::OneRecording,\n            report.effective_root,\n        );\n        tally.record_checks += 1;	        let record_check = RecordCheck::default();	-p singlefs-checker-tier --lib -- crash_injection::tests::crash_points_are_reproducible	crash_points_are_reproducible_proper_subsets_sorted_by_segment
 增补 3 第 3 件（用户 2026-09-20 定案第 4 条）：崩溃状态又只摆在起点跑完之后（起点那一段的写永远抽不到）	crates/singlefs-checker-tier/src/crash_injection.rs	                (marks.after_make_filesystem..marks.after_the_last_finished_step)\n                    .contains(&stream_indexes[*write])	                (marks\n                    .after_each_step\n                    .first()\n                    .map_or(0, |(_, after)| *after)\n                    ..marks.after_the_last_finished_step)\n                    .contains(&stream_indexes[*write])	-p singlefs-checker-tier --lib -- crash_injection::tests::crash_points_fall_inside_the_starting_point	crash_points_fall_inside_the_starting_point_too
-增补 3 第 3 件（用户 2026-09-20 定案第 6 条）：已知红第 1 条不读机理（不看 F 有没有真把环里读得到的根挡掉）	crates/singlefs-harness/src/history.rs	        && only_allocated_statistic_above_walked(observation)\n        && allocation_statistic_mechanism_of(observation)\n            .is_some_and(|mechanism| mechanism.the_floor_dropped_readable_roots())	        && only_allocated_statistic_above_walked(observation)	-p singlefs-harness --lib -- history::tests::known_red_forms_are_matched_by_mechanism	known_red_forms_are_matched_by_mechanism_not_by_signature
 增补 3 第 3 件（用户 2026-09-20 定案第 7 条，同日定的范围）：这个测试周期的种子基换成别的数（这一周期量过的判红整批作废，两个二进制也不再同基）	crates/singlefs-harness/src/history.rs	pub const SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE: u64 = 7_463_871_032_432_355_113;	pub const SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE: u64 = 12345;	-p singlefs-checker-tier --test crash_injection_campaign -- the_test_cycle_seed_base	the_test_cycle_seed_base_is_the_number_drawn_for_this_cycle
 增补 3 第 3 件（用户 2026-09-20 定案第 9 条）：镜像文件不排他创建，撞上同名接着用旧镜像	crates/singlefs-core/src/block_device.rs	            .create_new(true)\n            .open(path)	            .create(true)\n            .truncate(false)\n            .open(path)	-p singlefs-core --lib -- block_device::tests::creating_an_image_that_already_exists	creating_an_image_that_already_exists_is_refused_and_leaves_the_old_bytes_alone
 增补 3 第 3 件（用户 2026-09-20 定案第 3 条的配套口径）：记录核对器第二条判据不认「被流里更晚的写盖过」，合法复用被判成单元缺席	crates/singlefs-checker-tier/src/crash.rs	        let explained_by_a_persisted_legal_later_write =\n            (copy + 1..writes.len()).any(|later_index| {	        let explained_by_a_persisted_legal_later_write =\n            false && (copy + 1..writes.len()).any(|later_index| {	-p singlefs-checker-tier --test crash_enumeration_fixed_script_stream -- stale_tail_with_a_reused_named_unit	stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
@@ -475,10 +473,10 @@
 E158 root_choice_repair 候选 (b) 分配记录树指称「从内容树反推占用集合」：allocation_record_tree_reachable_placements_via_central_mapping 交回之前清空集合（该带着中央映射树读出的落点，改成永远交回空集）	crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs	        for location in locations {\n            placements.insert((location.device.0, location.slot.0));\n        }\n    }\n    Ok(placements)\n}	        for location in locations {\n            placements.insert((location.device.0, location.slot.0));\n        }\n    }\n    placements.clear();\n    Ok(placements)\n}	-p singlefs-checker-tier --bin e158_root_choice_repair	allocation_record_tree_reachable_placements_via_central_mapping_reflects_written_content_and_errors_on_an_empty_pointer
 增补 2 收口表第 58 行（`raise_rollback_floor` 同形）：抬 F 那一串失败时 core 不交已落盘那几次空发布的账；真设备二进制那一侧的判法（设备一层逐项相等）判出	crates/singlefs-core/src/mount.rs	                        publishes\n                            .iter()\n                            .map(|persisted: &TransactionOutput| persisted.writes.clone())\n                            .collect(),	                        Vec::new(),	-p singlefs-checker-tier --bin new_pool_file_creation_on_device -- failed_raise	failed_raise_of_the_rollback_floor_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count
 增补 2 收口表第 58 行（`raise_rollback_floor` 同形）：抬 F 那一串失败时 core 不交写入口的失败账（与可写挂载共用那一处）	crates/singlefs-core/src/mount.rs	        writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),	        writes_of_failed_publishes: Vec::new(),	-p singlefs-checker-tier --bin new_pool_file_creation_on_device -- failed_raise	failed_raise_of_the_rollback_floor_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count
-增补 2 收口表第 39 行那一族：取号之前的预演取不到落点也照样取号（240 槽小盘上可写挂载取号之后才被落点拒绝、盘上已经写了）	crates/singlefs-core/src/mount.rs	        Err(DryRunRefusal {\n            publish_index,\n            cause: PublishError::PlacementRefused { unit, refusal },\n        }) => {\n            return Err(\n                MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {\n                    instance_to_acquire,\n                    publish_index,\n                    warm_up_publishes_planned: warm_up_publishes_planned.len(),\n                    unit,\n                    refusal,\n                },\n            )\n        }	        Err(DryRunRefusal {\n            publish_index: _,\n            cause: PublishError::PlacementRefused { .. },\n        }) => None,	-p singlefs-harness --test random_histories -- eighteen_empty_overwrites_on_240_slot_devices	eighteen_empty_overwrites_on_240_slot_devices_with_the_admission_skipped_leave_room_for_a_writable_mount_with_one_warm_up
+增补 2 收口表第 39 行那一族：取号之前的预演取不到落点也照样取号（240 槽小盘上可写挂载取号之后才被落点拒绝、盘上已经写了）	crates/singlefs-core/src/mount.rs	        Err(DryRunRefusal {\n            publish_index,\n            cause: PublishError::PlacementRefused { unit, refusal },\n        }) => {\n            return Err(\n                MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {\n                    instance_to_acquire,\n                    publish_index,\n                    warm_up_publishes_planned: warm_up_publishes_planned.len(),\n                    unit,\n                    refusal,\n                },\n            )\n        }	        Err(DryRunRefusal {\n            publish_index: _,\n            cause: PublishError::PlacementRefused { .. },\n        }) => None,	-p singlefs-harness --test admission_raises_the_floor_before_refusing -- after_a_mount_whose_continuation_stays_short_the_next_mount	after_a_mount_whose_continuation_stays_short_the_next_mount_is_refused_by_the_placement_before_acquisition_writing_nothing
 增补 2 收口表第 39 行那一族：发布取完落点之后不记根（不转环、不回收；取号之前的预演与真发走同一段，两边一起不回收）	crates/singlefs-core/src/transaction.rs	    allocator.record_root_written_by_this_process(txg);\n    let slot_of = |identity: TransactionUnit| slots[&identity];	    let slot_of = |identity: TransactionUnit| slots[&identity];	-p singlefs-harness --test random_histories -- --include-ignored rolling_back_while_mounted_to_the_oldest_ring_root	rolling_back_while_mounted_to_the_oldest_ring_root_on_a_small_device_and_overwriting_again_all_succeed
 增补 2 收口表第 39 行那一族：发布不释放换下的落点（取号之前的预演与真发走同一段；回退到环里最旧的根时写行当场回收的几片哪一边都看不见）	crates/singlefs-core/src/transaction.rs	        allocator.release_leaving_the_record_allocated_on(\n            *placement,\n            txg,\n            &devices_whose_copy_failed_the_checksum,\n        );	        let _ = (placement, &devices_whose_copy_failed_the_checksum);	-p singlefs-harness --test random_histories -- --include-ignored rolling_back_while_mounted_to_the_oldest_ring_root	rolling_back_while_mounted_to_the_oldest_ring_root_on_a_small_device_and_overwriting_again_all_succeed
-增补 2 收口表第 39 行那一族：胶水把「取号之前在拷贝上取不到落点」映射成模型没有的理由（单元区墙那一格对不上）	crates/singlefs-harness/src/model_comparison.rs	        MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {\n            refusal, ..\n        } => refusal_reason_of_placement_refusal(refusal),	        MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. } => {\n            ObservedRefusalReason::Unexplained\n        }	-p singlefs-harness --test random_histories -- eighteen_empty_overwrites_on_240_slot_devices	eighteen_empty_overwrites_on_240_slot_devices_with_the_admission_skipped_leave_room_for_a_writable_mount_with_one_warm_up
+增补 2 收口表第 39 行那一族：胶水把「取号之前在拷贝上取不到落点」映射成模型没有的理由（单元区墙那一格对不上）	crates/singlefs-harness/src/model_comparison.rs	        MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {\n            refusal, ..\n        } => refusal_reason_of_placement_refusal(refusal),	        MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. } => {\n            ObservedRefusalReason::Unexplained\n        }	-p singlefs-harness --test admission_raises_the_floor_before_refusing -- after_a_mount_whose_continuation_stays_short_the_next_mount	after_a_mount_whose_continuation_stays_short_the_next_mount_is_refused_by_the_placement_before_acquisition_writing_nothing
 收口表第 26 行 I-7.9：判法拿掉（抬 F 的根带的 F 高于上限也判成立）	crates/singlefs-checker/src/walk.rs	        judgements.judge("I-7.9", raised_floor <= ceiling.lowest_possible, || {	        judgements.judge("I-7.9", true, || {	-p singlefs-harness --test checker_known_bad_images -- raising_the_floor_above_its_ceiling	raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant
 收口表第 26 行 I-7.9：上限算法改宽（每块盘上最新的有效根与第 4 新的非空有效根取大，不取小）	crates/singlefs-checker/src/walk.rs	    newest_valid_root_txg_on_every_device.min(fourth_newest_non_empty_or_oldest_valid)	    newest_valid_root_txg_on_every_device.max(fourth_newest_non_empty_or_oldest_valid)	-p singlefs-harness --test checker_known_bad_images -- raising_the_floor_above_its_ceiling	raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant
 收口表第 26 行 I-7.9：上限算法改宽（第 4 新的非空有效根改成第 3 新的）	crates/singlefs-checker/src/walk.rs	    let fourth_newest_non_empty_or_oldest_valid = newest_first\n        .get(3)	    let fourth_newest_non_empty_or_oldest_valid = newest_first\n        .get(2)	-p singlefs-harness --test checker_known_bad_images -- raising_the_floor_above_its_ceiling	raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant
@@ -743,7 +741,6 @@
 实四乙（实三交回 Q3：影子账里树表 0 条的被抛弃根那一支；崩溃恢复抛弃写过行的树表 0 条根、之后第一个文件与正常卸载把 F 抬过它引用的槽）：影子账把树表 0 条的被抛弃根当成账读不出，F 抬过之后它的树表与分配记录树根那两槽不隔离、可以再发出去	crates/singlefs-core/src/mount.rs	            Some(placements)\n        }\n        Ok(false) => Some(	            let _ = placements;\n            None\n        }\n        Ok(false) => Some(	-p singlefs-harness --test writable_mount_of_a_formatted_pool -- crash_recovery_abandoning_the_unwitnessed_row_publish	crash_recovery_abandoning_the_unwitnessed_row_publish_of_the_version_without_file_keeps_its_tree_table_and_allocation_record_tree_root_isolated_once_the_floor_passes_them
 实四乙（同上一支）：树表 0 条的被抛弃根只认实例表与树表、漏掉它那棵分配记录树的节点，F 抬过之后分配记录树根那一槽不隔离	crates/singlefs-core/src/mount.rs	                .chain([(&root.tree_table, TransactionUnit::TreeTable)])\n                .chain(\n                    allocation_record_tree_nodes\n                        .iter()\n                        .map(|pointer| (pointer, TransactionUnit::AllocationTree)),\n                )	                .chain([(&root.tree_table, TransactionUnit::TreeTable)])	-p singlefs-harness --test writable_mount_of_a_formatted_pool -- crash_recovery_abandoning_the_unwitnessed_row_publish	crash_recovery_abandoning_the_unwitnessed_row_publish_of_the_version_without_file_keeps_its_tree_table_and_allocation_record_tree_root_isolated_once_the_floor_passes_them
 实四乙（实三交回 Q4：收口表第 43 行那一形在向前回退下由崩溃恢复造、走得到）：checker 在 I-3.1 的说明文字里少写回退下界那一项机理标识，判读的一方认不出「F 挡掉了环里读得到的根」，那一形归成新发现	crates/singlefs-checker/src/walk.rs	、回退下界 F {effective_rollback_floor}、低于 F 的根槽 {root_slots_dropped_below_floor} 个"	、低于 F 的根槽 {root_slots_dropped_below_floor} 个"	-p singlefs-harness --test checker_known_bad_images -- the_allocated_statistic_violation_carries_every_mechanism_field_the_known_red_forms_read	the_allocated_statistic_violation_carries_every_mechanism_field_the_known_red_forms_read
-实四乙（同上一支）：已知红第 1 条把「F 那个 txg 上的根全属于被抛弃的实例」判反，崩溃恢复抛弃之后抬进空档的那一形归成新发现	crates/singlefs-harness/src/history.rs	    observation.raised_floor_lands_only_on_abandoned_roots == Some(true)	    observation.raised_floor_lands_only_on_abandoned_roots == Some(false)	-p singlefs-harness --lib -- known_red_forms_are_matched_by_mechanism_not_by_signature	known_red_forms_are_matched_by_mechanism_not_by_signature
 实四乙（C542 普通可写挂载那一支：所选根是最新根时这一串盖不掉它，回收不了这一串自己换下的，预演与真发不分叉）：挂载内每写一条根按这条根自己的 txg 当环里最旧的有效根回收，这一串自己换下的当场回收	crates/singlefs-core/src/allocator.rs	            crate::mount::reclaim_floor(occupancy.effective_floor, occupancy.oldest_valid_root());	            crate::mount::reclaim_floor(occupancy.effective_floor, Some(checkpoint_txg));	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored writable_mount_on_the_newest_root	writable_mount_on_the_newest_root_cannot_reclaim_what_its_own_chain_released_so_the_rehearsal_without_the_release_read_cannot_diverge
 实四乙（实三交回 Q2 查原因：恢复之前的镜像上 checker 只把已被某次恢复施加的记录并进遍历，I-3.1 多出来的正好是被写坏根槽的那条根独占的单元）：checker 把没被任何恢复施加过的记录也并进遍历	crates/singlefs-checker/src/walk.rs	            && instance_table_rows.iter().any(|row| {\n                row.instance == record.instance\n                    && record.checkpoint_txg <= row.published_checkpoint_txg\n            });	            && true;	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored plain_overwrite_whose_root_slot	plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced
 实五 P3pl（D16 已定项 1「准入」那一行：准入放行而落点取不到也推）：会话只在准入拒时推抬 F，落点被拒原样交回	crates/singlefs-core/src/mounted_session.rs	        PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. } => true,	        PublishError::SpaceAdmissionRefused(_) => true,\n        PublishError::PlacementRefused { .. } => false,	-p singlefs-harness --test admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments -- with_the_space_admission_switched_off_the_session_raises_the	with_the_space_admission_switched_off_the_session_raises_the_floor_after_a_placement_refusal_and_publishes_the_write
@@ -1061,7 +1058,7 @@
 实审 A2c 第 36 条：回退那次发布的 txg 直接加一（溢出 panic）	crates/singlefs-core/src/mount.rs	        .0\n        .checked_add(1)\n        .map(CheckpointTxg)\n        .ok_or(RollbackError::NextCheckpointTxgPastTheTopOfItsRange {	        .0\n        .checked_add(0)\n        .map(|_| CheckpointTxg(current.root.checkpoint_txg.0 + 1))\n        .ok_or(RollbackError::NextCheckpointTxgPastTheTopOfItsRange {	-p singlefs-harness --test core_review_unit_area_start_and_publish_limits	rolling_back_after_a_version_at_the_top_txg_is_refused_before_any_write
 实审 A2c 回退入口：调用方的参数与盘表核不过时不报成自己的成员（报成实例表坏）	crates/singlefs-core/src/mount.rs	            CallerInputsCheckRefused::Disagreeing(disagreeing) => {\n                RollbackError::CallerInputsDisagreeWithTheDisk(disagreeing)\n            }	            CallerInputsCheckRefused::Disagreeing(_disagreeing) => {\n                RollbackError::CurrentInstanceTableMalformed\n            }	-p singlefs-harness --test core_review_unit_area_start_and_publish_limits	rolling_back_with_a_device_identity_handed_in_twice_is_refused_before_any_write
 实审 A2c 回退入口：调用方参数与盘上不一致只看盘表那一张	crates/singlefs-core/src/mount.rs	    if disagreeing_fields.is_empty() && disagreeing_device_table.is_empty() {	    if disagreeing_device_table.is_empty() {	-p singlefs-harness --test core_review_unit_area_start_and_publish_limits	rolling_back_with_other_parameters_is_refused_before_any_write_and_with_the_pool_parameters_is_made
-实审 A2c Q5：抬 F 报错时交回的写行与暖机写账是空的	crates/singlefs-core/src/mount.rs	            .map(|version| version.writes().clone())	            .map(|_version| WritesByStructureKind::NOTHING_WRITTEN)	-p singlefs-harness --test core_review_unit_area_start_and_publish_limits	a_block_device_error_in_the_first_floor_raise_after_the_row_publish_hands_back_the_writes_of_the_mounts_publishes
+实审 A2c Q5：抬 F 报错时交回的写行与暖机写账是空的	crates/singlefs-core/src/mount.rs	                    .map(|version| version.writes().clone())\n                    .collect();	                    .map(|_version| WritesByStructureKind::NOTHING_WRITTEN)\n                    .collect();	-p singlefs-harness --test core_review_unit_area_start_and_publish_limits	a_block_device_error_in_the_first_floor_raise_after_the_row_publish_hands_back_the_writes_of_the_mounts_publishes
 实审 A2c Q5：抬 F 报错时不交回之前推成的那几串	crates/singlefs-core/src/mount.rs	                        writes_of_persisted_publishes,\n                        floor_raises,\n                    }),	                        writes_of_persisted_publishes,\n                        floor_raises: Vec::new(),\n                    }),	-p singlefs-harness --test core_review_unit_area_start_and_publish_limits	a_block_device_error_in_the_second_floor_raise_after_the_row_publish_hands_back_the_first_raise_too
 实审 A2c Q5：对拍不认装在里面的抬 F 的错（理由报成说不清）	crates/singlefs-harness/src/model_comparison.rs	        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {\n            refusal_reason_of_mount_error(&failed.cause)\n        }	        MountError::FloorRaiseFailedAfterTheMountsPublishes(_failed) => {\n            ObservedRefusalReason::Unexplained\n        }	-p singlefs-harness --lib -- model_comparison::tests::a_floor_raise_failure_after_the_mounts_publishes_is_judged_by_the_floor_raise_error_inside	model_comparison::tests::a_floor_raise_failure_after_the_mounts_publishes_is_judged_by_the_floor_raise_error_inside
 实审 A2c Q5：对拍不认装在里面的抬 F 报的上限	crates/singlefs-harness/src/model_comparison.rs	        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {\n            reported_ceiling_of_mount_error(&failed.cause)\n        }	        MountError::FloorRaiseFailedAfterTheMountsPublishes(_failed) => None,	-p singlefs-harness --lib -- model_comparison::tests::a_floor_raise_failure_after_the_mounts_publishes_is_judged_by_the_floor_raise_error_inside	model_comparison::tests::a_floor_raise_failure_after_the_mounts_publishes_is_judged_by_the_floor_raise_error_inside
@@ -1324,7 +1321,7 @@
 实审 A3b Q6：抬 F 读根环时根槽重读仍读不出改回「当没有根」	crates/singlefs-core/src/recovery.rs	            RootRingSlotReading::Bad(reread) => {\n                return Err(RootRingSlotKnownToHoldARootStillBadAfterOneReread {\n                    ring_slot,\n                    first_reading,\n                    reread,\n                });\n            }	            RootRingSlotReading::Bad(_reread) => {}	-p singlefs-harness --test unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over	raising_the_floor_while_a_root_ring_slot_known_to_hold_a_root_stays_unreadable_is_refused_before_any_write
 实审 A3b Q6：读根环时读不出的根槽不重读	crates/singlefs-core/src/recovery.rs	            RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => {\n                read_the_slot(ring_slot)\n            }	            RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => {\n                RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable)\n            }	-p singlefs-harness --test unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over	a_system_configuration_write_rereads_an_unreadable_root_ring_slot_once_before_taking_the_effective_floor
 实审 A3b 第 15 条：崩溃镜像的单元扫描候选仍从编译期常量 50176 起	crates/singlefs-harness/src/memory_pool.rs	            .filter(|slot| *slot >= lowest_candidate_slot)	            .filter(|slot| *slot >= lowest_candidate_slot.max(50_176))	-p singlefs-harness --test unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over	a_128_mebibyte_journal_ring_puts_the_unit_area_right_after_the_ring_through_mount_and_checker
-实审 A3b 第 15 条：模型的单元区起点改回编译期常量 50176（小盘单元区墙）	crates/singlefs-harness/src/model.rs	            unit_area_start_slot: model_unit_area_start_slot(journal_ring_bytes),	            unit_area_start_slot: 50_176,	-p singlefs-harness --test random_histories -- --include-ignored	unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped_runs_to_the_end_without_reaching_the_placement_wall
+实审 A3b 第 15 条：模型的单元区起点改回编译期常量 50176（小盘单元区墙）	crates/singlefs-harness/src/model.rs	            unit_area_start_slot: model_unit_area_start_slot(journal_ring_bytes),	            unit_area_start_slot: 50_176,	-p singlefs-harness --test random_histories -- --include-ignored unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped	unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped_runs_to_the_end_without_reaching_the_placement_wall
 A3b 善后一：可写挂载重建分配器时生效 F 又先自己读一遍最新根的实例表（那一片读次数回到 (5, 3)）	crates/singlefs-core/src/mount.rs	    let current_records = allocator.records().to_vec();\n    let (newest_table, instance_table_of_the_newest_root) =	    let current_records = allocator.records().to_vec();\n    let _ = effective_rollback_floor(\n        devices,\n        &system_configuration.immutable.region_devices,\n        &system_configuration.immutable.sizes,\n        &system_configuration.immutable.filesystem_identifier,\n    );\n    let (newest_table, instance_table_of_the_newest_root) =	-p singlefs-harness --test a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable	a_newest_instance_table_unreadable_when_the_shadow_ledger_reads_it_is_reread_once_then_refuses_writable_instead_of_turning_the_isolation_off
 A3b 善后一：mkfs 改回默认环的起点（1 GiB 的环把实例表写进环里）	crates/singlefs-core/src/make_filesystem.rs	    let unit_area_start = UnitAreaStart::following_the_journal_ring(ring_bytes)\n        .map_err(MakeFilesystemError::UnitAreaStartOffTheClusterSegmentBoundaryUnsupported)?;	    let unit_area_start = UnitAreaStart::of_the_default_journal_ring();	-p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode	a_journal_ring_reaching_past_784_mebibytes_is_made_with_the_instance_table_at_the_slot_after_the_ring_instead_of_inside_it
 A3b 善后一：mkfs 改回默认环的起点（默认环加一条记录不再按段边界拒）	crates/singlefs-core/src/make_filesystem.rs	    let unit_area_start = UnitAreaStart::following_the_journal_ring(ring_bytes)\n        .map_err(MakeFilesystemError::UnitAreaStartOffTheClusterSegmentBoundaryUnsupported)?;	    let unit_area_start = UnitAreaStart::of_the_default_journal_ring();	-p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode	a_journal_ring_one_record_longer_than_the_default_ends_off_the_cluster_segment_boundary_and_is_refused_and_the_default_ring_is_not
@@ -2019,7 +2016,7 @@
 RefFS 翻译（13-4）：故障注入的刷盘被吞档退回照判	crates/singlefs-harness/src/reference_filesystem_translation.rs	        InjectedFault::BarrierIsSwallowed => true,	        InjectedFault::BarrierIsSwallowed => false,	-p singlefs-harness --test reference_filesystem_translation_of_histories	the_reopen_after_a_swallowed_barrier_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish
 挂载透明（13-4）：抬 F 落没落从挂载第一个写起算（暖机与抬 F 退回一起算）	crates/singlefs-harness/src/mount_transparency.rs	                    .get(first_write..)	                    .get(..)	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_second_crash_after_the_warm_up_judges_the_floor_unchanged_and_one_after_the_first_floor_raise_write_leaves_it_unjudged
 挂载透明（13-4）：抬 F 的界取到挂载第一个系统配置槽写（不认抬到的 F）	crates/singlefs-harness/src/mount_transparency.rs	            FloorRaisesInTheMountWrites::FirstWriteAt(first_write)\n        } else {	            FloorRaisesInTheMountWrites::FirstWriteAt(0)\n        } else {	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_second_crash_after_the_warm_up_judges_the_floor_unchanged_and_one_after_the_first_floor_raise_write_leaves_it_unjudged
-崩溃注入（13-4）：崩溃后镜像上的挂载不交回抬 F 从哪个写起	crates/singlefs-checker-tier/src/crash_injection.rs	        FloorRaisesInTheMountWrites::of_the_mount_output(&mounted.output, &mount_writes)	        FloorRaisesInTheMountWrites::NotPushed	-p singlefs-checker-tier --lib	the_writable_mount_on_a_crash_image_names_the_first_floor_raise_write_after_the_warm_up
+崩溃注入（13-4）：崩溃后镜像上的挂载不交回抬 F 从哪个写起	crates/singlefs-checker-tier/src/crash_injection.rs	        floor_raises_in_the_mount_writes_counting_the_continuation(&mounted.output, &mount_writes)	        FloorRaisesInTheMountWrites::NotPushed	-p singlefs-checker-tier --lib	the_writable_mount_on_a_crash_image_names_the_first_floor_raise_write_after_the_warm_up
 E171 装置 取号逐盘核「少于两槽」写成「不多于两槽」、不带故障的盘上每次取号都拒（代码轮第三轮判决 K4-c 攻方造法；O1-对拍那一栏该红，装置不记红时这一行没红）	crates/singlefs-core/src/transaction.rs	< SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE	<= SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE	-p singlefs-checker-tier --bin e171_three_oracle_collusion	an_acquire_and_a_warm_up_on_a_fresh_pool_leave_the_comparison_column_without_a_red
 E171 装置 取号或暖机被拒不记成 O1-对拍的红（代码轮第三轮判决 K4-c 甲；O1-对拍留在 not_modelled）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs	        self.calls[call_index].model = ModelVerdictOfCall::Disagrees {\n            aspect:\n                singlefs_harness::model::ModelDisagreementAspect::RefusedWhenModelRequiresSuccess	        let _discarded = ModelVerdictOfCall::Disagrees {\n            aspect:\n                singlefs_harness::model::ModelDisagreementAspect::RefusedWhenModelRequiresSuccess	-p singlefs-checker-tier --bin e171_three_oracle_collusion	a_refused_acquire_on_a_stream_without_faults_is_recorded_as_a_comparison_red_and_stops_the_model
 E171 装置 关掉 O1 时被拒的取号照记红（Q3-核关 O1 那一列该写 off、不该有红）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs	        if !self.model_is_enabled() {\n            return;\n        }\n        let call_index = self.calls.len() - 1;\n        self.calls[call_index].model = ModelVerdictOfCall::Disagrees {\n            aspect:\n	        let call_index = self.calls.len() - 1;\n        self.calls[call_index].model = ModelVerdictOfCall::Disagrees {\n            aspect:\n	-p singlefs-checker-tier --bin e171_three_oracle_collusion	with_the_model_switched_off_a_refused_acquire_is_not_a_comparison_red
@@ -2124,8 +2121,7 @@
 三·2 Q-J：续做从「续做之前发了几次」那个根槽写之后再晚一次才算起（续做第一步的写归暖机）	crates/singlefs-checker-tier/src/crash_injection.rs	            if publishes_landed >= publishes_before_the_continuation =>	            if publishes_landed > publishes_before_the_continuation =>	-p singlefs-checker-tier --lib -- crash_injection::tests	the_writes_after_the_publishes_before_the_continuation_belong_to_the_continuation_phase_and_get_a_second_crash_point
 三·2 后续 1：执行器比挂载的分配记录时写行与暖机照旧按「带文件」挑（删完的一版上那几次被跳过）	crates/singlefs-harness/src/history.rs	        .filter_map(PoolVersion::version_of_trees)	        .filter_map(PoolVersion::file_version)	-p singlefs-harness --test random_histories -- a_mount_on_the_emptied_version_in_the_unit_area_wall_sampling	a_mount_on_the_emptied_version_in_the_unit_area_wall_sampling_compares_its_row_and_warm_up_publishes
 三·2 后续 1：删完的一版上发布的分配记录把本次的代写成前一版（调查员的反向注入：比较链补进那几版之后照样判红）	crates/singlefs-harness/src/history.rs	            allocation_generation_judgement(\n                &previous_records,\n                &output.allocation_records,	            allocation_generation_judgement(\n                &previous_records,\n                &output\n                    .allocation_records\n                    .iter()\n                    .map(|record| {\n                        let mut record = record.clone();\n                        if output.is_a_version_with_trees_and_no_file() && record.generation == publish_txg {\n                            record.generation = CheckpointTxg(publish_txg.0 - 1);\n                        }\n                        record\n                    })\n                    .collect::<Vec<_>>(),	-p singlefs-harness --test random_histories -- a_mount_on_the_emptied_version_in_the_unit_area_wall_sampling	a_mount_on_the_emptied_version_in_the_unit_area_wall_sampling_compares_its_row_and_warm_up_publishes
-三·2 后续 2：挂载续做里删除类发布被空间拒不推抬 F（原样交回 MountError::Publish）	crates/singlefs-core/src/mount.rs	        if !refusal_is_short_of_space(&refusal) {\n            *current = PoolVersion::of_a_version_of_trees(walked);\n            return Err(walk_failed(refusal));	        if true || !refusal_is_short_of_space(&refusal) {\n            *current = PoolVersion::of_a_version_of_trees(walked);\n            return Err(walk_failed(refusal));	-p singlefs-harness --test admission_raises_the_floor_before_refusing -- a_mount_whose_deletion_continuation	a_mount_whose_deletion_continuation_finds_no_unblocked_slot_raises_the_floor_and_continues_to_the_emptied_version
-三·2 后续 2：挂载续做推满仍不够时当作走完、挂载交回做成（队列留着）	crates/singlefs-core/src/mount.rs	            Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(_stop)) => {\n                *current = PoolVersion::of_a_version_of_trees(walked);\n                return Err(walk_failed(refusal));	            Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(_stop)) => {\n                *current = PoolVersion::of_a_version_of_trees(walked);\n                return Ok(continuation);	-p singlefs-harness --test admission_raises_the_floor_before_refusing -- a_mount_whose_deletion_continuation	a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_hands_back_the_refusal_and_leaves_its_publishes_on_disk
+三·2 后续 2：挂载续做里删除类发布被空间拒不推抬 F（原样交回 MountError::Publish）	crates/singlefs-core/src/mount.rs	        if !refusal_is_short_of_space(&refusal) {\n            *current = PoolVersion::of_a_version_of_trees(walked);\n            let writes_of_persisted_publishes: Vec<WritesByStructureKind> =	        if true || !refusal_is_short_of_space(&refusal) {\n            *current = PoolVersion::of_a_version_of_trees(walked);\n            let writes_of_persisted_publishes: Vec<WritesByStructureKind> =	-p singlefs-harness --test admission_raises_the_floor_before_refusing -- a_mount_whose_deletion_continuation	a_mount_whose_deletion_continuation_finds_no_unblocked_slot_raises_the_floor_and_continues_to_the_emptied_version
 三·2 Q-V 推满仍不够：会话交回空间不够时把已落盘的意图那一版丢掉不交	crates/singlefs-core/src/mounted_session.rs	                stop,\n                versions_published,\n            })),	                stop,\n                versions_published: Vec::new(),\n            })),	-p singlefs-harness --test admission_raises_the_floor_before_refusing -- an_unlink_whose_last_batch_and_the_floor_raise	an_unlink_whose_last_batch_and_the_floor_raise_pushed_for_it_both_find_no_unblocked_slot_hands_back_no_space_with_the_intent_published
 随机历史建 inode：快档那组比重「删过之后」那张表里建 inode 那一格是 0（unlink 之后生成不到建 inode）	crates/singlefs-harness/src/history.rs	            (HistoryOperationKind::CreateInode, 40),\n            (HistoryOperationKind::RaiseRollbackFloor, 15),	            (HistoryOperationKind::CreateInode, 0),\n            (HistoryOperationKind::RaiseRollbackFloor, 15),	-p singlefs-harness --test unlink_in_histories_judged_by_the_model -- the_fast_tier_weights_draw_an_inode_creation	the_fast_tier_weights_draw_an_inode_creation_after_an_unlink_and_the_first_history_with_one_runs_to_the_end
 随机历史建 inode：模型的建 inode 那次不重写 inode 叶	crates/singlefs-harness/src/model.rs	            ModelPublishKind::NewInode => &[\n                ModelUnitRole::InodeLeaf,\n                ModelUnitRole::InodeRoot,	            ModelPublishKind::NewInode => &[\n                ModelUnitRole::InodeRoot,	-p singlefs-harness --test unlink_in_histories_judged_by_the_model -- creating_inodes_after_the_first_file_was_deleted	creating_inodes_after_the_first_file_was_deleted_runs_through_a_mount_with_the_model_agreeing
@@ -2134,3 +2130,34 @@
 Q-T 改钉：抬 F 的上限取第三新的非空状态（少算一档）；种子 4000000045 第 27 步	crates/singlefs-core/src/mount.rs	        .get(3)\n        .copied()\n        .or(oldest_valid_root)?;	        .get(2)\n        .copied()\n        .or(oldest_valid_root)?;	-p singlefs-harness --test random_histories -- seed_4000000045_on_240_slot_devices	seed_4000000045_on_240_slot_devices_with_the_admission_skipped_raises_the_floor_to_19_reclaiming_135_placements
 Q-T 改钉：抬 F 的上限取第三新的非空状态（少算一档）；种子 4000000001 第 30 步	crates/singlefs-core/src/mount.rs	        .get(3)\n        .copied()\n        .or(oldest_valid_root)?;	        .get(2)\n        .copied()\n        .or(oldest_valid_root)?;	-p singlefs-harness --test random_histories -- --include-ignored seed_4000000001_on_240_slot_devices	seed_4000000001_on_240_slot_devices_with_the_admission_skipped_raises_the_floor_to_7_in_two_publishes
 Q-T 改钉：单元区墙取样点那组比重里 unlink 那一格是 0（准入关着那一段生成的历史跟着变）	crates/singlefs-harness/src/history.rs	            (HistoryOperationKind::CloseAndMountWritable, 3),\n            (HistoryOperationKind::Unlink, 3),	            (HistoryOperationKind::CloseAndMountWritable, 3),\n            (HistoryOperationKind::Unlink, 0),	-p singlefs-harness --test random_histories -- --include-ignored unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped	unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped_runs_to_the_end_without_reaching_the_placement_wall
+三·2 删除批代码轮 U14（用户 2026-10-03 定「照 D16 做成可写」）：挂载续做推满仍放不下照旧交回 MountError::Publish（定案之前的做法），不做成可写	crates/singlefs-core/src/mount.rs	                continuation.outcome =\n                    DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises {\n                        last_refusal: refusal,\n                        stop,\n                    };\n                return Ok(continuation);	                let _ = stop;\n                return Err(MountError::Publish(Box::new(PublishSequenceFailed {\n                    cause: refusal,\n                    writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,\n                    writes_of_persisted_publishes: Vec::new(),\n                    writes_of_failed_publishes: Vec::new(),\n                })));	-p singlefs-harness --test admission_raises_the_floor_before_refusing -- a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable	a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_with_the_queue_left_and_its_publishes_on_disk
+三·2 删除批代码轮 U9：挂载续做里的一步没发成、不是空间不够时错里的已落盘写账照旧交空表（mount.rs PublishSequenceFailed 契约）	crates/singlefs-core/src/mount.rs	                writes_of_the_publishes_before_the_continuation\n                    .iter()\n                    .cloned()\n                    .chain(continuation.writes_of_the_persisted_publishes_in_txg_order())\n                    .collect();	                Vec::new();	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version -- a_mount_whose_continuation_step_fails_on_a_device_error	a_mount_whose_continuation_step_fails_on_a_device_error_hands_back_the_writes_of_every_publish_it_landed
+三·2 删除批代码轮 U10：会话先走空之后拒成 NoFileVersionToChange 时不带走空发成的那几版	crates/singlefs-core/src/mounted_session.rs	    UserChangeRefused::NoFileVersionToChange {\n        floor_raises,\n        refusals_that_pushed_the_floor_raises,\n        versions_published,\n    }\n}	    let _ = versions_published;\n    UserChangeRefused::NoFileVersionToChange {\n        floor_raises,\n        refusals_that_pushed_the_floor_raises,\n        versions_published: Vec::new(),\n    }\n}	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version -- an_overwrite_after_rolling_back_to_the_intent	an_overwrite_after_rolling_back_to_the_intent_walks_the_queue_and_its_refusal_carries_every_version_it_published
+三·2 删除批代码轮 U11：随机历史执行器在删除中的版本上照旧跳过覆盖写（模型比不到走空那几版）	crates/singlefs-harness/src/history.rs	    let queue_is_walked_first = !current_file_version.deleted_inodes.is_empty();	    if !current_file_version.deleted_inodes.is_empty() {\n        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionStillHasTheFirstFile);\n    }\n    let queue_is_walked_first = false;	-p singlefs-harness --test unlink_in_histories_judged_by_the_model -- an_overwrite_after_rolling_back_to_the_intent	an_overwrite_after_rolling_back_to_the_intent_is_judged_by_the_model_through_the_walked_step
+三·2 删除批代码轮 U11：随机历史执行器在删除中的版本上照旧跳过 unlink	crates/singlefs-harness/src/history.rs	        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutTheFirstFile);\n    }\n    let records_before = open_session.allocator.records().to_vec();	        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutTheFirstFile);\n    }\n    if !current_file_version.deleted_inodes.is_empty() {\n        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionStillHasTheFirstFile);\n    }\n    let records_before = open_session.allocator.records().to_vec();	-p singlefs-harness --test unlink_in_histories_judged_by_the_model -- an_unlink_after_rolling_back_to_the_intent	an_unlink_after_rolling_back_to_the_intent_is_judged_by_the_model_through_the_walked_step
+三·2 删除批代码轮 U11：随机历史执行器在删除中的版本上照旧跳过建 inode	crates/singlefs-harness/src/history.rs	            if version.inode_record.is_some() && version.deleted_inodes.is_empty() {	            if version.inode_record.is_some() || !version.deleted_inodes.is_empty() {	-p singlefs-harness --test unlink_in_histories_judged_by_the_model -- creating_an_inode_after_rolling_back_to_the_intent	creating_an_inode_after_rolling_back_to_the_intent_walks_the_queue_first_and_is_judged_by_the_model
+三·2 删除批代码轮 U3：deleted_inodes 装不下（多于一个根装得下的号）的拒绝归进空间不够那两种（会为它推抬 F）	crates/singlefs-core/src/mounted_session.rs	        PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. } => true,	        PublishError::SpaceAdmissionRefused(_)\n        | PublishError::PlacementRefused { .. }\n        | PublishError::DeletedInodesQueueRefused(_) => true,	-p singlefs-harness --test a_floor_raise_refused_for_space_counts_as_short_of_space -- a_deleted_inodes_queue_longer_than_one_root_holds	a_deleted_inodes_queue_longer_than_one_root_holds_is_not_short_of_space_and_pushes_no_floor_raise
+三·2 删除批代码轮 U6：冷走读里 nlink = 0 的记录照样当活文件读（D8 已定项 16 第 4 条从意图起读者答「没有文件」）	crates/singlefs-core/src/recovery.rs	    if inode_record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {	    if false && inode_record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {	-p singlefs-harness --test unlink_single_file_publishes -- from_the_intent_on_the_read_only_mount_and_the_cold_walk_answer_no_file	from_the_intent_on_the_read_only_mount_and_the_cold_walk_answer_no_file
+三·2 删除批代码轮 U2：删除批只改尾叶那一判把「末个单元等于留下的个数」那片叶也算整片留下（< 改 <=）	crates/singlefs-core/src/unlink.rs	        last_data_unit < data_units_kept	        last_data_unit <= data_units_kept	-p singlefs-harness --test unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches -- a_file_over_two_extent_lower_leaves	a_file_over_two_extent_lower_leaves_unlinked_in_product_batches_rewrites_the_leaf_whose_last_unit_is_cut
+遗留改动代码轮 LR4（C393 (b)-从映射现算最后一支）：被抛弃根的账与从映射现算重读一次仍都不成时不拒可写，当它什么都不引用	crates/singlefs-core/src/mount.rs	                                Some(placements) => placements,\n                                None => return Err(\n                                    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(\n                                        StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {\n                                            abandoned_root: RollbackTarget {\n                                                instance: root.instance,\n                                                checkpoint_txg: root.checkpoint_txg,\n                                            },\n                                        },\n                                    )),\n                                ),	                                Some(placements) => placements,\n                                None => Vec::new(),	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area	an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area_refuses_the_writable_mount_instead_of_panicking
+遗留改动代码轮 LR3（攻方变异 raise-no-restore）：抬 F 那一串的影子账交 Err 时不把分配器换回抬 F 之前那一份	crates/singlefs-core/src/mount.rs	            Ok(computed) => computed.abandoned_roots_unreadable,\n            Err(error) => {\n                *allocator = allocator_before_the_raise;\n                return Err(error);\n            }	            Ok(computed) => computed.abandoned_roots_unreadable,\n            Err(error) => {\n                return Err(error);\n            }	-p singlefs-harness --test an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping -- a_floor_raise_whose_shadow_ledger_meets	a_floor_raise_whose_shadow_ledger_meets_an_abandoned_root_unreadable_by_account_and_mapping_is_refused_and_restores_the_allocator
+遗留改动代码轮 LR2（攻方变异 map-first-instance-page-only）：被抛弃根的账读不出、从映射现算时只认实例表第 0 片	crates/singlefs-core/src/mount.rs	    for page in instance_table_page_pointers_as_far_as_readable(devices, root) {\n        located.push((page.locations, TransactionUnit::InstanceTable.span_slots()));	    for page in instance_table_page_pointers_as_far_as_readable(devices, root)\n        .into_iter()\n        .take(1)\n    {\n        located.push((page.locations, TransactionUnit::InstanceTable.span_slots()));	-p singlefs-harness --test an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping -- an_abandoned_root_with_a_two_page_instance_table	an_abandoned_root_with_a_two_page_instance_table_and_an_unreadable_account_isolates_both_pages_from_the_mapping
+三·2 挂载续做（m3-batch11 第 1 条）：抬 F 的界不认续做里推的那几串（准入没推、续做里推了的那一形交「没推」，二次崩溃崩在那一串里时 F 照没推判）	crates/singlefs-checker-tier/src/crash_injection.rs	        let first_pushed_by_the_continuation = self.pushed_by_the_continuation.first()?;	        let first_pushed_by_the_continuation = self.pushed_by_the_continuation.first().filter(|_| false)?;	-p singlefs-checker-tier --lib -- crash_injection::tests::a_floor_raise_pushed_inside_the_continuation_is_located_between_the_continuation_steps	a_floor_raise_pushed_inside_the_continuation_is_located_between_the_continuation_steps
+三·2 挂载续做（m3-batch11 第 1 条）：准入推了抬 F 之后又有续做时界不把续做的写算进去（界落在续做那一步的单元写上、取不出）	crates/singlefs-checker-tier/src/crash_injection.rs	                writes_of(&self.pushed_by_the_admission)\n                    + continuation_steps_after(None)\n	                writes_of(&self.pushed_by_the_admission)\n	-p singlefs-checker-tier --lib -- crash_injection::tests::floor_raises_pushed_by_the_mount_admission_are_located_before_the_continuation_that_follows_them	floor_raises_pushed_by_the_mount_admission_are_located_before_the_continuation_that_follows_them
+三·2 U12：挂载在续做里交回错时认不出续做的那几次发布（续做的写并进暖机段）	crates/singlefs-checker-tier/src/crash_injection.rs	        .find(|(_, writes)| writes_a_structure_only_the_continuation_changes(writes))	        .find(|(_, writes)| false && writes_a_structure_only_the_continuation_changes(writes))	-p singlefs-checker-tier --lib -- crash_injection::tests::a_mount_that_handed_back_an_error_inside_the_continuation_marks_the_continuation_by_the_persisted_publishes	a_mount_that_handed_back_an_error_inside_the_continuation_marks_the_continuation_by_the_persisted_publishes
+LR7：设备侧日志折清零段不守长度 0（声明长度 0 时铺满的判定交回「用掉 0 条」，折叠原地不动）	crates/singlefs-checker-tier/src/device_log.rs	    if declared_length == 0 {	    if false && declared_length == 0 {	-p singlefs-checker-tier --lib -- device_log::	device_log::tests::a_declared_zero_fill_of_length_zero_tiles_no_entry_and_leaves_the_log_unchanged
+LR7：设备侧日志折清零段时接在末端的那一条不查段尾（越过段尾的接续写并进来，「已铺满的末端停在段尾」那一句断言 panic）	crates/singlefs-checker-tier/src/device_log.rs	let continues_at_cursor = piece.start == covered_up_to && piece.end <= fill_end;	let continues_at_cursor = piece.start == covered_up_to;	-p singlefs-checker-tier --lib -- device_log::	device_log::tests::zero_fill_whose_piece_continuing_at_the_covered_end_runs_past_the_end_does_not_fold
+LR7：设备侧日志折清零段时恰好接在末端的那一条不并（「接得上的都并进来了」那一句断言 panic）	crates/singlefs-checker-tier/src/device_log.rs	.position(|arrived| arrived.start <= covered_up_to)	.position(|arrived| arrived.start < covered_up_to)	-p singlefs-checker-tier --lib -- device_log::	device_log::tests::zero_fill_whose_two_parts_of_one_chunk_arrive_reversed_folds_into_one_event
+回退那次发布 D 的段序列：发布的单元写与记录之间少一道屏障（D 的 24 个单元写与 2 条记录并成一段，孤立看 24+2+1+2 变 26+1+2）	crates/singlefs-core/src/transaction.rs	    }\n    writer.perform_commit_step(CommitStep::Barrier)?;\n    for record in &writes.records {	    }\n    for record in &writes.records {	-p singlefs-harness --test rollback_by_a_forward_publish -- rolling_back_to_the_first_version_writes_its_units_in_one_segment_before_the_records_the_root_and_the_system_configuration	rolling_back_to_the_first_version_writes_its_units_in_one_segment_before_the_records_the_root_and_the_system_configuration
+增补 3 第 3 件第 4 题：摆崩溃状态的起点错放到流尾（每段历史一个崩溃状态都摆不出，快档判 histories_without_any_crash_point == 0）	crates/singlefs-checker-tier/src/crash_injection.rs	    marks.after_make_filesystem = run.operations_written_by_make_filesystem;	    marks.after_make_filesystem = marks.after_the_last_finished_step;	-p singlefs-checker-tier --test crash_injection_campaign -- --include-ignored crash_injection_fast_tier_recovers_only_into_versions_the_model_committed	crash_injection_fast_tier_recovers_only_into_versions_the_model_committed
+增补 3 第 3 件第 4 题：摆崩溃状态的起点错放到流尾（每段历史一个崩溃状态都摆不出，小快档判 histories_without_any_crash_point == 0）	crates/singlefs-checker-tier/src/crash_injection.rs	    marks.after_make_filesystem = run.operations_written_by_make_filesystem;	    marks.after_make_filesystem = marks.after_the_last_finished_step;	-p singlefs-checker-tier --test crash_injection_campaign -- crash_injection_small_fast_tier_recovers_only_into_versions_the_model_committed	crash_injection_small_fast_tier_recovers_only_into_versions_the_model_committed
+增补 3 第 3 件第 4 题：崩溃状态记到下一步的类名下（零单元发布那一步的崩溃状态记到冷启动头上）	crates/singlefs-checker-tier/src/crash_injection.rs	                StepPosition::Operation(step_index) => step_kinds.get(step_index).copied(),	                StepPosition::Operation(step_index) => step_kinds.get(step_index + 1).copied(),	-p singlefs-checker-tier --test crash_injection_campaign -- crash_states_land_in_zero_unit_publishes_and_first_files_but_never_in_cold_starts_or_refused_crash_recoveries	crash_states_land_in_zero_unit_publishes_and_first_files_but_never_in_cold_starts_or_refused_crash_recoveries
+增补 3 第 3 件第 4 题：可写挂载那一步记成崩溃恢复抛弃根（被拒的崩溃恢复不发写那一格判不出）	crates/singlefs-harness/src/history.rs	            HistoryOperation::CloseAndMountWritable => HistoryOperationKind::CloseAndMountWritable,	            HistoryOperation::CloseAndMountWritable => HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,	-p singlefs-checker-tier --test crash_injection_campaign -- crash_states_land_in_zero_unit_publishes_and_first_files_but_never_in_cold_starts_or_refused_crash_recoveries	crash_states_land_in_zero_unit_publishes_and_first_files_but_never_in_cold_starts_or_refused_crash_recoveries
+增补 3 第 3 件第 4 题：发布的记录与根槽之间少一道屏障（零单元发布的段 2、1、2 并成一段，摆到的崩溃状态 7 变少）	crates/singlefs-core/src/transaction.rs	    writer.perform_commit_step(CommitStep::Barrier)?;\n    persist_the_root_then_rotate_the_system_configuration(\n        writer,\n        writes.checkpoint_txg,\n	    persist_the_root_then_rotate_the_system_configuration(\n        writer,\n        writes.checkpoint_txg,\n	-p singlefs-checker-tier --test crash_injection_campaign -- crash_states_land_in_zero_unit_publishes_and_first_files_but_never_in_cold_starts_or_refused_crash_recoveries	crash_states_land_in_zero_unit_publishes_and_first_files_but_never_in_cold_starts_or_refused_crash_recoveries
+增补 3 第 3 件第 4 题：发布的单元写与记录之间少一道屏障（第一个文件的段 24、2 并成一段，摆到的崩溃状态 8 变 5；零单元发布没有单元写，不变）	crates/singlefs-core/src/transaction.rs	    }\n    writer.perform_commit_step(CommitStep::Barrier)?;\n    for record in &writes.records {	    }\n    for record in &writes.records {	-p singlefs-checker-tier --test crash_injection_campaign -- crash_states_land_in_zero_unit_publishes_and_first_files_but_never_in_cold_starts_or_refused_crash_recoveries	crash_states_land_in_zero_unit_publishes_and_first_files_but_never_in_cold_starts_or_refused_crash_recoveries
+增补 3 第 3 件第 1 题：崩溃镜像那一截保留记录核对器的调用与计数、只把判据从通过条件里丢掉（攻方变异 A；根在而记录一条都不在的靶向崩溃状态没有新发现）	crates/singlefs-checker-tier/src/crash_injection.rs	        if !(violations.is_empty()\n            && disagreement.is_none()\n            && record_check == RecordCheck::default())\n        {	        if !(violations.is_empty() && disagreement.is_none()) {	-p singlefs-checker-tier --test crash_injection_campaign -- a_crash_state_with_the_root_but_neither_of_its_records_is_a_finding_of_the_record_checker_alone	a_crash_state_with_the_root_but_neither_of_its_records_is_a_finding_of_the_record_checker_alone
+增补 3 第 3 件第 1 题：崩溃镜像那一截交给记录核对器的恢复落到的那一版换成 None（第二条判据的前置恒不成立，缺单元的靶向崩溃状态判不出）	crates/singlefs-checker-tier/src/crash_injection.rs	            RecordStreamContinuity::OneRecording,\n            report.effective_root,\n        );	            RecordStreamContinuity::OneRecording,\n            None,\n        );	-p singlefs-checker-tier --test crash_injection_campaign -- a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_finding_of_the_record_checker	a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_finding_of_the_record_checker
+增补 3 第 3 件第 1 题：攻方变异 A 对第二条判据（根环转过一圈之后缺单元、只有记录核对器判得出的靶向崩溃状态没有新发现）	crates/singlefs-checker-tier/src/crash_injection.rs	        if !(violations.is_empty()\n            && disagreement.is_none()\n            && record_check == RecordCheck::default())\n        {	        if !(violations.is_empty() && disagreement.is_none()) {	-p singlefs-checker-tier --test crash_injection_campaign -- a_crash_state_missing_a_unit_of_a_publish_whose_root_left_the_ring_is_a_finding_of_the_record_checker_alone	a_crash_state_missing_a_unit_of_a_publish_whose_root_left_the_ring_is_a_finding_of_the_record_checker_alone
+K3-⑨ 两道防线至少留一道：恢复交给前缀核的点名验证开关恒为不验（记录已持久而它点名的单元两份都没落时照样施加、走读失败）	crates/singlefs-core/src/recovery.rs	    let named_unit_verification = if verify_named_units {	    let named_unit_verification = if false && verify_named_units {	-p singlefs-harness --test named_unit_verification_stands_in_for_the_barrier_before_the_record -- with_named_unit_verification_a_record_whose_unit_never_landed_falls_back_to_the_previous_version	with_named_unit_verification_a_record_whose_unit_never_landed_falls_back_to_the_previous_version
+K3-⑨ 两道防线至少留一道：只供测试的开关关掉点名验证那一臂照样验（ConsultWithoutNamedVerification 走成 Consult，记录已持久而单元没落的状态上模型不再分歧）	crates/singlefs-core/src/recovery.rs	                policy == JournalPolicy::Consult,\n	                policy != JournalPolicy::Ignore,\n	-p singlefs-harness --test named_unit_verification_stands_in_for_the_barrier_before_the_record -- without_named_unit_verification_the_same_state_is_a_model_disagreement	without_named_unit_verification_the_same_state_is_a_model_disagreement
+C592：发布的单元写与记录之间少一道屏障（复用回收槽的那一次发布上，扣下复用槽上数据单元一份、记录已落的状态 I-3.10 红）	crates/singlefs-core/src/transaction.rs	    }\n    writer.perform_commit_step(CommitStep::Barrier)?;\n    for record in &writes.records {	    }\n    for record in &writes.records {	-p singlefs-harness --test named_unit_verification_stands_in_for_the_barrier_before_the_record -- with_the_barrier_before_the_record_a_publish_reusing_reclaimed_slots_keeps_every_single_withheld_state_free_of_checker_violations	with_the_barrier_before_the_record_a_publish_reusing_reclaimed_slots_keeps_every_single_withheld_state_free_of_checker_violations
~~~~~~
