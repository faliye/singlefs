# 第 16 批代码轮被判的改动全文（2026-10-04）

基：合入前备份（会话草稿 batch16-premerge/，即合入那一刻的主仓）；今天：主工作区（第 16 批 16-2、16-3、16-4 三份补丁）。

## `crates/singlefs-harness/src/model.rs`

````diff
--- crates/singlefs-harness/src/model.rs	2026-10-03 13:02:45.166906997 +0000
+++ crates/singlefs-harness/src/model.rs	2026-10-03 15:04:59.288562953 +0000
@@ -756,6 +756,7 @@
     MountWritable,
     RollbackWhileMounted,
     RaiseRollbackFloor,
+    Unmount,
     ColdStartRecover,
     PublishUnlinkIntent,
     PublishDeletionStep,
@@ -2399,8 +2400,31 @@
                 ))
             }
         }
+        let roots = self.empty_publishes_carrying_the_floor_to_every_device(session, new_floor);
+        let mut answer = self.answer_for_publishes(
+            ModelOperationKind::RaiseRollbackFloor,
+            required_refusals,
+            permitted_refusals,
+            roots,
+        );
+        answer.rollback_floor_ceiling = ceiling;
+        // 实现在任何写之前把这一串整串预演一遍（`mount::raise_rollback_floor`），哪一次撞墙都在第一次写之前拒、一次都不发。
+        answer.walls_are_judged_before_the_first_write = true;
+        Ok(answer)
+    }
+
+    /// 抬 F 那一串（D16（发布语义） 已定项 1「抬 F 那一串」：准入与卸载共用）接在会话现行那一版后面写出的根：带 `new_floor` 的空发布
+    /// （有树的一版上重写四个固定点单元），推到每块盘上都有一条为止（「生效」那一行），至多 R 次。
+    /// 先写系统配置那一步不属于任何一次发布（已定项 7），模型不建系统配置，这里只有根。
+    fn empty_publishes_carrying_the_floor_to_every_device(
+        &self,
+        session: &ModelSession,
+        new_floor: ModelCheckpointTxg,
+    ) -> Vec<ModelRoot> {
+        let current = &session.current;
         let mut roots: Vec<ModelRoot> = Vec::new();
         let mut covered: BTreeSet<ModelDeviceIdentity> = BTreeSet::new();
+        // 迭代上界是根环区域数 R；跨轮带的是已写出的根与已罩到的盘；没有提前出口。
         while self
             .geometry
             .devices
@@ -2427,15 +2451,46 @@
                 Rc::clone(&previous.instance_table_rows),
             ));
         }
+        roots
+    }
+
+    /// 正常卸载（`mount::unmount`，D16（发布语义） 已定项 1「正常卸载」）：推一串空发布把 F 抬到现行那一版的 txg，走抬 F 那一串
+    /// （推到每块盘上都有一条带新 F 的根，第一版几何上 2 或 3 次），不另判上限（「抬 F 的上限」那一行「卸载抬 F」）；
+    /// 做完之后这个进程不再有可写会话。
+    ///
+    /// 模型不比的两样：这一串的根在 flags 里带卸载记号（D22（单元原子性怎么合成） 已定项 7）——[`ObservedRoot`] 不带 flags；
+    /// 先写系统配置那一步——模型不建系统配置。容量墙同抬 F：实现在任何写之前把这一串整串预演一遍，模型答允许拒绝的区间。
+    ///
+    /// # Errors
+    /// 模型里没有可写会话：会话状态与实现对不上。现行那一版树表 0 条：条款写「推一串空发布」，实现在这一版上一个字节都不写
+    /// （`mount::Unmounted::NothingWrittenOnAVersionWithoutFile`，主 agent 2026-09-26 定、没进 D16），两种答法条款没判，模型不答。
+    pub fn answer_unmount(&self) -> Result<ModelAnswer, ModelDisagreement> {
+        let session = self.open_session()?;
+        let current = &session.current;
+        if !current.contents.has_trees() {
+            return Err(ModelDisagreement::new(
+                ModelDisagreementAspect::NotModeled,
+                format!(
+                    "模型里现行版本 {:?} 树表 0 条：卸载推不推空发布条款没判，模型不答",
+                    current.key
+                ),
+                "执行器调了卸载".to_string(),
+            ));
+        }
+        let floor_raised_to_the_current_txg = current.key.checkpoint_txg;
+        let roots = self.empty_publishes_carrying_the_floor_to_every_device(
+            session,
+            floor_raised_to_the_current_txg,
+        );
         let mut answer = self.answer_for_publishes(
-            ModelOperationKind::RaiseRollbackFloor,
-            required_refusals,
-            permitted_refusals,
+            ModelOperationKind::Unmount,
+            BTreeSet::new(),
+            BTreeSet::new(),
             roots,
         );
-        answer.rollback_floor_ceiling = ceiling;
-        // 实现在任何写之前把这一串整串预演一遍（`mount::raise_rollback_floor`），哪一次撞墙都在第一次写之前拒、一次都不发。
+        // 实现在任何写之前把这一串整串预演一遍（`mount::unmount` 走的就是抬 F 那一串），哪一次撞墙都在第一次写之前拒、一次都不发。
         answer.walls_are_judged_before_the_first_write = true;
+        answer.session_after_success = ModelSessionAfterSuccess::Closed;
         Ok(answer)
     }
 
````

## `crates/singlefs-harness/src/model_comparison.rs`

````diff
--- crates/singlefs-harness/src/model_comparison.rs	2026-10-03 13:02:45.166982637 +0000
+++ crates/singlefs-harness/src/model_comparison.rs	2026-10-03 15:04:59.289240998 +0000
@@ -17,7 +17,7 @@
 };
 use singlefs_core::mount::{
     InstanceRow, MountError, Mounted, RollbackCandidateExclusion, RollbackError,
-    StillUnreadableAfterOneReread,
+    StillUnreadableAfterOneReread, Unmounted,
 };
 use singlefs_core::pointer::LocationEntry;
 use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
@@ -640,6 +640,24 @@
     }
 }
 
+/// 一次正常卸载（`mount::unmount`）做成了什么：推了那一串的，是那几次带新 F 的空发布写出的根（每一条照带文件的一版比，F 读根上带的）；
+/// 现行那一版树表 0 条、一个字节都没写的，是零条根。卸载不判上限，不交上限。
+#[must_use]
+pub fn observed_unmount(unmounted: &Unmounted) -> ObservedEffect {
+    let roots = match unmounted {
+        Unmounted::FloorRaisedToTheCurrentVersion(raised) => raised
+            .publishes
+            .iter()
+            .map(observed_root_of_file_version)
+            .collect(),
+        Unmounted::NothingWrittenOnAVersionWithoutFile { .. } => Vec::new(),
+    };
+    ObservedEffect::Publishes {
+        roots,
+        reported_ceiling: None,
+    }
+}
+
 fn explained(reason: ModelRefusalReason) -> ObservedRefusalReason {
     ObservedRefusalReason::Explained(reason)
 }
````

## `crates/singlefs-harness/src/mount_transparency.rs`

````diff
--- crates/singlefs-harness/src/mount_transparency.rs	2026-10-03 12:09:16.714828116 +0000
+++ crates/singlefs-harness/src/mount_transparency.rs	2026-10-03 15:05:03.378625646 +0000
@@ -6,9 +6,10 @@
 //! 或挂载加一次发布——之后再只读恢复一次（R2），逐字段比（[`judge_mount_transparency`]）；挂载途中各相位的二次崩溃镜像上，
 //! 只读恢复读出的与 R1 比（用户 2026-10-02 定 F6「加二次」）；抬 F 的写有持久的那几态 F 不判（[`FloorRaisesInTheMountWrites`]；
 //! 抬 F 的界在挂载写表里对不上位置的那一次挂载，二次崩溃整段 F 不判、按 [`FieldNotJudged`] 计数）。
-//! 可写挂载被拒的状态不判透明，判拒绝本身合不合法（[`judge_writable_mount_refusal`]）。
+//! 可写挂载被拒的状态不判透明，判拒绝本身合不合法（[`judge_writable_mount_refusal`]；崩溃注入第二截再加 C670 那一条：
+//! 没注入读故障的崩溃态上判据 N-配置 复算为真，拒得不合法，[`judge_writable_mount_refusal_of_a_crash_state`]）。
 //!
-//! 读盘的只有 [`read_recovery_fields`] 与 [`judge_writable_mount_refusal`] 里在崩溃镜像上复算拒的判据那一截；[`judge_mount_transparency`]
+//! 读盘的只有 [`read_recovery_fields`] 与那两个判拒绝的函数里在崩溃镜像上复算拒的判据那一截；[`judge_mount_transparency`]
 //! 只吃读好的字段与挂载交回值。checker 档的崩溃注入调它，harness 档的轻用例也调它（harness 档不依赖 checker 档）。
 //! r5 判决标「推的」的几样（树 ID 水位、两个根指针、txg、扫描报告）照写进判法，每一样红的时候在交回值里单独标出它是推的还是量过的
 //! （[`MountTransparencyField::basis`]）。
@@ -28,7 +29,7 @@
     UserVisibleTreeRootPointers,
 };
 use singlefs_core::root_record::{RootRecord, UnmountMarker};
-use singlefs_core::system_configuration::SystemImmutableConfiguration;
+use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableConfiguration};
 use singlefs_core::transaction::TransactionOutput;
 use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
 
@@ -1344,13 +1345,39 @@
             .map(|slot| slot.quantities.journal_tail)
             .fold(witnessed_journal_counter, u64::max);
     }
+    witnessed_counter_holds_against_the_selected_version(
+        crash_image,
+        &system_configuration,
+        witnessed_journal_counter,
+        selected_version,
+    )
+    .is_some()
+}
+
+/// 判据 N-配置 在 c_见证 与所选那一版比出来为真的那一支（D23（journal 的角色与格式） 已定项 14「可写挂载读到的更新状态读不出」那一段的后半）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub enum WitnessedCounterBranchThatHolds {
+    /// 所选那一版那次发布带末条标志的记录读得出，c_见证 大于它的计数器。
+    WitnessedCounterAboveTheLastRecordOfTheSelectedVersion,
+    /// 所选那一版的末条读不出，计数器等于 c_见证 的记录读得出、它的 (实例代号, checkpoint_txg) 大于所选那一版。
+    RecordAtTheWitnessedCounterNewerThanTheSelectedVersion,
+    /// 两条都读不出，按真。
+    NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable,
+}
+
+/// c_见证 已经取好之后的那一半：0 ⇒ 假；所选那一版那次发布带末条标志的记录读得出 ⇒ c_见证 > 它的计数器；读不出而计数器等于 c_见证 的记录读得出 ⇒
+/// 它的 (实例代号, checkpoint_txg) > 所选那一版；两条都读不出按真。为真交回是哪一支，为假（或没有所选那一版、判据无从复算）交回 `None`。
+fn witnessed_counter_holds_against_the_selected_version(
+    crash_image: &dyn PoolReader,
+    system_configuration: &SystemConfiguration,
+    witnessed_journal_counter: u64,
+    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
+) -> Option<WitnessedCounterBranchThatHolds> {
     if witnessed_journal_counter == 0 {
-        return false;
+        return None;
     }
-    let Some(selected_version) = selected_version else {
-        return false;
-    };
-    let records = scan_journal(crash_image, &system_configuration);
+    let selected_version = selected_version?;
+    let records = scan_journal(crash_image, system_configuration);
     let last_record_counter_of_the_selected_version = records
         .values()
         .filter(|record| {
@@ -1360,7 +1387,9 @@
         .map(|record| record.counter)
         .max();
     if let Some(last_record_counter) = last_record_counter_of_the_selected_version {
-        return witnessed_journal_counter > last_record_counter;
+        return (witnessed_journal_counter > last_record_counter).then_some(
+            WitnessedCounterBranchThatHolds::WitnessedCounterAboveTheLastRecordOfTheSelectedVersion,
+        );
     }
     let record_at_the_witnessed_counter = records
         .values()
@@ -1368,11 +1397,55 @@
         .map(|record| (record.instance, record.checkpoint_txg))
         .max();
     match record_at_the_witnessed_counter {
-        Some(record) => record > selected_version,
-        None => true,
+        Some(record) => (record > selected_version).then_some(
+            WitnessedCounterBranchThatHolds::RecordAtTheWitnessedCounterNewerThanTheSelectedVersion,
+        ),
+        None => Some(
+            WitnessedCounterBranchThatHolds::NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable,
+        ),
     }
 }
 
+/// C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）：判据 N-配置 在没注入读故障的崩溃镜像上复算，为真交回是哪一支。
+/// 没有读故障时设备不报读错，见证读判得出（R2，D22（单元原子性怎么合成） 已定项 8 射程：判不出 ⟺ 有槽设备报读错且没有一块盘两槽都自证过；
+/// 撕裂的系统配置轮换不算判不出）：c_见证 取全部自证过、fsid 与本池相同的槽 journal tail 的最大值，自证不过的槽（撕裂的、没写过的）跳过。
+/// 撕裂系统配置那一支因此不落在这里（C664（挂载拒绝合法复算把两种拒因报成一个）、C331（择根倒挂压过已确认的写））：
+/// [`newer_publish_witnessed_on_the_crash_image`] 照甲把它判成按真，那一支的拒照 U6 判。一份自证过的系统配置都择不到时 c_见证 为 0，交回 `None`。
+fn witnessed_counter_holds_on_a_crash_image_without_read_faults(
+    crash_image: &dyn PoolReader,
+    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
+) -> Option<WitnessedCounterBranchThatHolds> {
+    let Ok(system_configuration) = choose_system_configuration(crash_image) else {
+        return None;
+    };
+    let slot_spacing = u64::from(
+        system_configuration
+            .immutable
+            .sizes
+            .fixed_structure_slot_spacing,
+    );
+    let witnessed_journal_counter = crash_image
+        .device_identities()
+        .into_iter()
+        .flat_map(|device| {
+            verified_system_configuration_slots(
+                crash_image,
+                device,
+                slot_spacing,
+                &system_configuration.immutable.filesystem_identifier,
+            )
+        })
+        .map(|slot| slot.quantities.journal_tail)
+        .max()
+        .unwrap_or(0);
+    witnessed_counter_holds_against_the_selected_version(
+        crash_image,
+        &system_configuration,
+        witnessed_journal_counter,
+        selected_version,
+    )
+}
+
 /// 「重建分配器判抛弃用的最新那条根的实例表读不出」在崩溃镜像上复算：根环里择到的最新那条根指着的实例表沿链读不出、解不开（择不到根也算）。
 fn instance_table_of_the_newest_root_unreadable_on_the_crash_image(
     crash_image: &dyn PoolReader,
@@ -1398,6 +1471,14 @@
     OutsideTheAllowedSet,
 }
 
+/// 崩溃态注没注入过读故障（D13（验证路线） 已定项 1：挂载透明的射程只管不带故障的状态；C670（无读故障的崩溃态上可写挂载见证判据为真没有判官））。
+/// 没注入：崩溃镜像读得出的就是盘上的字节，造出这份盘面的历史里也没有一步在读故障下发过写。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum ReadFaultsInTheCrashState {
+    NoneInjected,
+    Injected,
+}
+
 /// 单子上的理由，合法还差哪一条。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
 pub enum RefusalConditionNotMet {
@@ -1405,6 +1486,9 @@
     CriterionFalseWhenRecomputedOnTheCrashImage,
     DiskChangedByTheRefusedMount,
     ReadOnlyReadBackOutsideTheAllowedSet,
+    /// C670：崩溃态没注入读故障，判据 N-配置 在崩溃镜像上（撕裂的系统配置槽跳过）复算仍为真——正确实现的可写挂载在这份盘面上永远拒，
+    /// 一次普通断电之后池只剩只读挂载；这样的崩溃态不许出现，拒是对的、造出这份盘面的那一段不对。带的是判据为真的那一支。
+    WitnessCriterionTrueOnACrashImageWithoutReadFaults(WitnessedCounterBranchThatHolds),
 }
 
 /// 可写挂载被拒，拒得合不合法（r5 判决第二节 U6：拒绝合法 ⇔ 理由在 D23 已定项 14 那张单子上、那一条判据在崩溃镜像上复算为真、
@@ -1420,7 +1504,8 @@
     ReasonNotOnTheD23List,
 }
 
-/// 判一次拒可写合不合法。拒的判据在崩溃镜像（`crash_image`）上自己复算（不经 O1 判官，也不经实现判拒的那一段）；
+/// 判一次拒可写合不合法，只判 r5 判决 U6 那四条，不判 C670 那一条（崩溃态知道注没注入读故障的调用方用
+/// [`judge_writable_mount_refusal_of_a_crash_state`]）。拒的判据在崩溃镜像（`crash_image`）上自己复算（不经 O1 判官，也不经实现判拒的那一段）；
 /// `selected_version` 是崩溃镜像上只读恢复落到的那一版（R1 恢复报告里的 `effective_root`）。
 #[must_use]
 pub fn judge_writable_mount_refusal(
@@ -1433,6 +1518,77 @@
     let Some(reason) = refusal_reason_on_the_d23_list(refusal) else {
         return WritableMountRefusalVerdict::ReasonNotOnTheD23List;
     };
+    verdict_of_a_refusal_on_the_d23_list(
+        reason,
+        conditions_not_met_by_a_refusal_on_the_d23_list(
+            crash_image,
+            selected_version,
+            reason,
+            disk,
+            read_only_read_back,
+        ),
+    )
+}
+
+/// 判崩溃注入第二截的一次拒可写合不合法：U6 那四条（[`judge_writable_mount_refusal`]），再加 C670 那一条——崩溃态没注入读故障
+/// （`read_faults`）、理由是判据 N-配置，而判据在崩溃镜像上复算、撕裂的系统配置槽跳过之后仍为真（[`WitnessedCounterBranchThatHolds`]），
+/// 拒得不合法（[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]）。注入过读故障的崩溃态不判这一条
+/// （D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。
+#[must_use]
+pub fn judge_writable_mount_refusal_of_a_crash_state(
+    crash_image: &dyn PoolReader,
+    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
+    refusal: &MountError,
+    disk: DiskAfterTheRefusal,
+    read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
+    read_faults: ReadFaultsInTheCrashState,
+) -> WritableMountRefusalVerdict {
+    let Some(reason) = refusal_reason_on_the_d23_list(refusal) else {
+        return WritableMountRefusalVerdict::ReasonNotOnTheD23List;
+    };
+    let mut conditions_not_met = conditions_not_met_by_a_refusal_on_the_d23_list(
+        crash_image,
+        selected_version,
+        reason,
+        disk,
+        read_only_read_back,
+    );
+    match (reason, read_faults) {
+        (
+            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
+            ReadFaultsInTheCrashState::NoneInjected,
+        ) => {
+            if let Some(branch) = witnessed_counter_holds_on_a_crash_image_without_read_faults(
+                crash_image,
+                selected_version,
+            ) {
+                conditions_not_met.push(
+                    RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(
+                        branch,
+                    ),
+                );
+            }
+        }
+        (
+            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
+            ReadFaultsInTheCrashState::Injected,
+        )
+        | (
+            RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable,
+            ReadFaultsInTheCrashState::NoneInjected | ReadFaultsInTheCrashState::Injected,
+        ) => {}
+    }
+    verdict_of_a_refusal_on_the_d23_list(reason, conditions_not_met)
+}
+
+/// 理由在单子上（U6 第一条）之后，U6 另三条里没满足的：判据在崩溃镜像上复算为真、盘上逐字节不变、只读读出在允许集合里。
+fn conditions_not_met_by_a_refusal_on_the_d23_list(
+    crash_image: &dyn PoolReader,
+    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
+    reason: RefusalReasonOnTheD23List,
+    disk: DiskAfterTheRefusal,
+    read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
+) -> Vec<RefusalConditionNotMet> {
     let criterion_holds_on_the_crash_image = match reason {
         RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration => {
             newer_publish_witnessed_on_the_crash_image(crash_image, selected_version)
@@ -1458,6 +1614,13 @@
             conditions_not_met.push(RefusalConditionNotMet::ReadOnlyReadBackOutsideTheAllowedSet);
         }
     }
+    conditions_not_met
+}
+
+fn verdict_of_a_refusal_on_the_d23_list(
+    reason: RefusalReasonOnTheD23List,
+    conditions_not_met: Vec<RefusalConditionNotMet>,
+) -> WritableMountRefusalVerdict {
     if conditions_not_met.is_empty() {
         WritableMountRefusalVerdict::Legal(reason)
     } else {
````

## `crates/singlefs-checker-tier/src/crash_injection.rs`

````diff
--- crates/singlefs-checker-tier/src/crash_injection.rs	2026-10-03 12:09:16.714739675 +0000
+++ crates/singlefs-checker-tier/src/crash_injection.rs	2026-10-03 15:05:03.378625646 +0000
@@ -74,10 +74,11 @@
     refusal_reason_of_publish_error,
 };
 use singlefs_harness::mount_transparency::{
-    judge_mount_transparency, judge_writable_mount_refusal, read_recovery_fields, ComparedVersion,
-    DiskAfterTheRefusal, FieldOutcome, FloorRaisedByTheMount, FloorRaisesInTheMountWrites,
-    MountTransparency, MountTransparencyAction, ReadOnlyReadBackOfTheCrashImage,
-    RecoveryFieldsRead, UserVisibleTreeRootPointersOfTheContinuation, WritableMountRefusalVerdict,
+    judge_mount_transparency, judge_writable_mount_refusal_of_a_crash_state, read_recovery_fields,
+    ComparedVersion, DiskAfterTheRefusal, FieldOutcome, FloorRaisedByTheMount,
+    FloorRaisesInTheMountWrites, MountTransparency, MountTransparencyAction,
+    ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage, RecoveryFieldsRead,
+    UserVisibleTreeRootPointersOfTheContinuation, WritableMountRefusalVerdict,
 };
 use singlefs_harness::numeric_environment_variable::{
     number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
@@ -538,6 +539,8 @@
     /// 崩溃后镜像上的可写挂载被拒、理由在 D23（journal 的角色与格式） 已定项 14 那张单子上的：拒得合法的、不合法的（判红）。
     pub writable_mount_refusals_on_the_d23_list_legal: u64,
     pub writable_mount_refusals_on_the_d23_list_illegal: u64,
+    /// 崩溃窗口末尾之前历史里有一步在注入的读故障下发过东西的崩溃状态：第二截判拒可写时不判 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）那一条。
+    pub crash_points_after_a_step_reading_under_injected_faults: u64,
     /// 第一截与二次崩溃那一截上 RefFS 与 O1 判官两票合起来（G1 用户定案：任一红即红），按哪一边红分开计；
     /// RefFS 弃权的按理由另计（两截并在一起）。
     pub reference_filesystem_and_model_verdicts_on_crash_images:
@@ -608,6 +611,7 @@
             mount_transparency_fields_not_judged,
             writable_mount_refusals_on_the_d23_list_legal,
             writable_mount_refusals_on_the_d23_list_illegal,
+            crash_points_after_a_step_reading_under_injected_faults,
             reference_filesystem_and_model_verdicts_on_crash_images,
             reference_filesystem_and_model_verdicts_on_second_crashes,
             reference_filesystem_abstentions,
@@ -735,6 +739,8 @@
             writable_mount_refusals_on_the_d23_list_legal;
         self.writable_mount_refusals_on_the_d23_list_illegal +=
             writable_mount_refusals_on_the_d23_list_illegal;
+        self.crash_points_after_a_step_reading_under_injected_faults +=
+            crash_points_after_a_step_reading_under_injected_faults;
         for (verdict, count) in reference_filesystem_and_model_verdicts_on_crash_images {
             *self
                 .reference_filesystem_and_model_verdicts_on_crash_images
@@ -903,9 +909,10 @@
         );
         let _ = writeln!(
             text,
-            "可写挂载被拒、理由在 D23 已定项 14 那张单子上的：拒得合法 {}、不合法 {}",
+            "可写挂载被拒、理由在 D23 已定项 14 那张单子上的：拒得合法 {}、不合法 {}；崩溃窗口之前有一步带读故障、不判 C670 那一条的崩溃状态 {}",
             self.writable_mount_refusals_on_the_d23_list_legal,
-            self.writable_mount_refusals_on_the_d23_list_illegal
+            self.writable_mount_refusals_on_the_d23_list_illegal,
+            self.crash_points_after_a_step_reading_under_injected_faults
         );
         let _ = writeln!(
             text,
@@ -971,6 +978,52 @@
             .find(|(_, operations_so_far)| *operations_so_far > stream_index)
             .map(|(position, _)| *position)
     }
+
+    /// 崩溃窗口末尾那个写（录制流第 `last_stream_index_of_the_crash_window` 步）之前（含），有没有一步在读故障下往录制流里发过东西
+    /// （C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）；D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。
+    /// 崩溃注入跑历史不带注入计划（`execute_history_with`），读故障只由 [`operation_reads_under_injected_faults`] 认的那几类步造；
+    /// 那样的一步一个都没发（取号之前就拒了：C554 乙之后「崩溃恢复抛弃根」的常态）时盘面与没走这一步的历史一样，不算注入过。
+    fn read_faults_in_a_crash_state(
+        &self,
+        step_kinds: &[HistoryOperationKind],
+        last_stream_index_of_the_crash_window: usize,
+    ) -> ReadFaultsInTheCrashState {
+        let mut operations_before_the_step = 0;
+        for (position, operations_after_the_step) in &self.after_each_step {
+            let step_reads_under_injected_faults = match position {
+                StepPosition::StartingPoint => false,
+                StepPosition::Operation(step_index) => step_kinds
+                    .get(*step_index)
+                    .copied()
+                    .is_some_and(operation_reads_under_injected_faults),
+            };
+            let step_sent_something_by_the_crash = operations_before_the_step
+                < *operations_after_the_step
+                && operations_before_the_step <= last_stream_index_of_the_crash_window;
+            if step_reads_under_injected_faults && step_sent_something_by_the_crash {
+                return ReadFaultsInTheCrashState::Injected;
+            }
+            operations_before_the_step = *operations_after_the_step;
+        }
+        ReadFaultsInTheCrashState::NoneInjected
+    }
+}
+
+/// 这一类历史步在注入的读故障下读盘：崩溃恢复抛弃根（那次可写挂载期间最新那条根的根槽与它点名的单元读回全 0，
+/// `singlefs_harness::history` 的 `DeviceReadingZerosOverHiddenRanges`）。别的步读的就是盘上的字节。
+fn operation_reads_under_injected_faults(kind: HistoryOperationKind) -> bool {
+    match kind {
+        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot => true,
+        HistoryOperationKind::PublishFirstFile
+        | HistoryOperationKind::PublishOverwrite
+        | HistoryOperationKind::PublishWithoutUnits
+        | HistoryOperationKind::CloseAndMountWritable
+        | HistoryOperationKind::RollBackWhileMounted
+        | HistoryOperationKind::RaiseRollbackFloor
+        | HistoryOperationKind::ColdStartRecover
+        | HistoryOperationKind::Unlink
+        | HistoryOperationKind::CreateInode => false,
+    }
 }
 
 /// 跑一段历史，按 `draw` 在它的录制流上摆崩溃状态，逐个重建镜像、跑恢复、池级 checker 与记录核对器、问模型；
@@ -1206,6 +1259,16 @@
         // 第二截与第三截（代码审阅第 2 条）：只读恢复只读盘，取号、写行、暖机在崩溃状态上从没跑过；在同一份崩溃后镜像上
         // 真的起一次可写挂载、再发一次布、跑 checker 与记录核对器，挂载途中再崩一次。
         let crash_image_materialized = materialized_crash_image(&image);
+        let read_faults_in_the_crash_state = marks.read_faults_in_a_crash_state(
+            &step_kinds,
+            stream_indexes[first_write_of_the_segment + writes_in_the_segment - 1],
+        );
+        match read_faults_in_the_crash_state {
+            ReadFaultsInTheCrashState::NoneInjected => {}
+            ReadFaultsInTheCrashState::Injected => {
+                tally.crash_points_after_a_step_reading_under_injected_faults += 1;
+            }
+        }
         let content_read_back_after_the_crash = match &read_back {
             ObservedReadBack::FileRead { content, .. } => Some(Some(Rc::from(content.as_slice()))),
             ObservedReadBack::NoFile { .. } => Some(None),
@@ -1245,6 +1308,7 @@
                 } else {
                     ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet
                 },
+                read_faults: read_faults_in_the_crash_state,
             },
             history.seed,
             crash_point,
@@ -2006,13 +2070,15 @@
     crash_image_materialized: &'crash MemoryPool,
     version_landed_on_after_the_crash: Option<(InstanceGeneration, CheckpointTxg)>,
     read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
+    /// 这个崩溃态注没注入过读故障（[`StreamMarks::read_faults_in_a_crash_state`]）：没注入的，判据 N-配置 复算为真即拒得不合法（C670）。
+    read_faults: ReadFaultsInTheCrashState,
 }
 
 /// 第二截：挂载或那次发布被拒（单元区墙只计数，模型在单元区墙上只答允许拒绝的区间、而这里没有崩溃点上的模型状态可判区间），
 /// 做没做成都在之后的池上跑池级 checker；`record_check` 是同一个池上记录核对器的结论
 /// （[`HistoryThenWritableMountRecords::record_check_after_the_writable_mount`]）。挂载被拒而理由在 D23（journal 的角色与格式） 已定项 14
-/// 那张单子上的，判拒绝本身（`singlefs_harness::mount_transparency::judge_writable_mount_refusal`：判据在崩溃镜像上复算为真、盘上逐字节不变、
-/// 只读读出在允许集合里），合法的只计数，不合法的报红。别的理由被拒按「模型说该成、实现拒了」报：
+/// 那张单子上的，判拒绝本身（`singlefs_harness::mount_transparency::judge_writable_mount_refusal_of_a_crash_state`：判据在崩溃镜像上复算为真、
+/// 盘上逐字节不变、只读读出在允许集合里；崩溃态没注入读故障而判据 N-配置 复算为真的另判不合法，C670），合法的只计数，不合法的报红。别的理由被拒按「模型说该成、实现拒了」报：
 /// 模型对可写挂载与覆盖写、第一个文件从不要求拒，容量墙之外也不许拒（`IdealModel::answer_mount_writable`）。
 #[allow(
     clippy::too_many_arguments,
@@ -2041,12 +2107,13 @@
                 } else {
                     DiskAfterTheRefusal::Changed
                 };
-            match judge_writable_mount_refusal(
+            match judge_writable_mount_refusal_of_a_crash_state(
                 refusal_on_the_crash_image.crash_image_materialized,
                 refusal_on_the_crash_image.version_landed_on_after_the_crash,
                 refusal_error,
                 disk,
                 refusal_on_the_crash_image.read_only_read_back,
+                refusal_on_the_crash_image.read_faults,
             ) {
                 WritableMountRefusalVerdict::Legal(_) => {
                     tally.writable_mount_refusals_on_the_d23_list_legal += 1;
@@ -4708,4 +4775,141 @@
                 .collect::<Vec<StepPosition>>()
         );
     }
+
+    /// 带读故障的步与别的步交错时登记的那几段流：起点发到第 10 步，关掉重挂发到 20，崩溃恢复抛弃根（带读故障）发到 25，零单元发布发到 40。
+    fn marks_with_an_abandoning_recovery_sending_stream_steps_15_through_24() -> StreamMarks {
+        StreamMarks {
+            after_make_filesystem: 4,
+            after_the_last_finished_step: 40,
+            after_each_step: vec![
+                (StepPosition::StartingPoint, 10),
+                (StepPosition::Operation(0), 20),
+                (StepPosition::Operation(1), 25),
+                (StepPosition::Operation(2), 40),
+            ],
+        }
+    }
+
+    /// C670：崩溃窗口末尾那个写之前（含）历史里有一步「崩溃恢复抛弃根」往录制流里发过东西，这个崩溃态才算注入过读故障；
+    /// 那一步落在窗口末尾之后的不算。
+    #[test]
+    fn a_crash_state_counts_as_read_faulted_from_the_first_stream_step_of_an_abandoning_recovery_on(
+    ) {
+        let step_kinds = [
+            HistoryOperationKind::CloseAndMountWritable,
+            HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
+            HistoryOperationKind::PublishWithoutUnits,
+        ];
+        let marks = marks_with_an_abandoning_recovery_sending_stream_steps_15_through_24();
+        assert_eq!(
+            marks.read_faults_in_a_crash_state(&step_kinds, 19),
+            ReadFaultsInTheCrashState::NoneInjected,
+            "窗口末尾落在带读故障那一步发的第一步之前：没注入"
+        );
+        assert_eq!(
+            marks.read_faults_in_a_crash_state(&step_kinds, 20),
+            ReadFaultsInTheCrashState::Injected,
+            "窗口末尾就是带读故障那一步发的第一步：注入过"
+        );
+        assert_eq!(
+            marks.read_faults_in_a_crash_state(&step_kinds, 35),
+            ReadFaultsInTheCrashState::Injected,
+            "窗口末尾落在带读故障那一步之后的零单元发布里：注入过"
+        );
+    }
+
+    /// C670：「崩溃恢复抛弃根」那一步一个都没往录制流里发（取号之前就拒了），盘面与没走这一步的历史一样：之后的崩溃态都不算注入过读故障。
+    #[test]
+    fn an_abandoning_recovery_that_sent_nothing_leaves_every_crash_state_without_read_faults() {
+        let step_kinds = [
+            HistoryOperationKind::CloseAndMountWritable,
+            HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
+            HistoryOperationKind::PublishWithoutUnits,
+        ];
+        let marks = StreamMarks {
+            after_make_filesystem: 4,
+            after_the_last_finished_step: 40,
+            after_each_step: vec![
+                (StepPosition::StartingPoint, 10),
+                (StepPosition::Operation(0), 20),
+                (StepPosition::Operation(1), 20),
+                (StepPosition::Operation(2), 40),
+            ],
+        };
+        for last_stream_index_of_the_crash_window in [12, 19, 20, 35] {
+            assert_eq!(
+                marks.read_faults_in_a_crash_state(&step_kinds, last_stream_index_of_the_crash_window),
+                ReadFaultsInTheCrashState::NoneInjected,
+                "带读故障那一步一个都没发，窗口末尾在第 {last_stream_index_of_the_crash_window} 步：没注入"
+            );
+        }
+    }
+
+    /// C670 判别力（E171（三oracle合谋测试） 第 242、269 行）：起点（取号、暖机、第一个文件）加一次关掉重挂的历史，短段整段枚举；
+    /// 历史里没有带读故障的步，每个崩溃态都判 C670 那一条，第二截一个拒得不合法的都没有、没有新发现。
+    /// 系统配置轮换挪到根槽 FUA 之前时（`crates/mutations.tsv` 里「发布的持久顺序把系统配置槽轮换挪到根槽 FUA 写之前」那条改法），暖机那一段摆得出
+    /// 「轮换落了、根没落」：记录不跨实例边界，所选那一版停在上一个实例那一版，见证值比它新，正确实现在这份盘面上永远拒可写——这一条红；
+    /// 取号写的见证值取成槽世代号时（「见证值取成槽世代号」那条），重挂取号写落了、暖机的根没落的那几态同样红。
+    // crash-case-check:not-a-crash-case 起点加一步的合成历史、短段整段枚举，不调全量枚举，崩溃态数以百计（推的，没量过）
+    #[test]
+    fn no_crash_state_without_read_faults_after_a_remount_is_refused_writable_for_a_witness_newer_than_the_selected_version(
+    ) {
+        use singlefs_harness::history::{HistoryDeviceWidth, PerStepChecker};
+        let history = GeneratedHistory {
+            seed: HistorySeed(0),
+            starting_point: HistoryStartingPoint::AfterFirstFile,
+            operations: vec![HistoryOperation::CloseAndMountWritable],
+        };
+        let injection = inject_crashes_into_history(
+            &history,
+            HistoryExecution {
+                per_step_checker: PerStepChecker::Skipped,
+                device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
+                space_admission: SpaceAdmission::JudgedByTheFormula,
+            },
+            CrashPointDraw::EveryProperSubsetOfShortSegments {
+                segment_writes: 4,
+                sampled_in_longer_segments: 2,
+            },
+            None,
+        );
+        let tally = &injection.tally;
+        assert!(tally.crash_points > 0, "一个崩溃态都没摆出来：{tally:?}");
+        assert_eq!(
+            tally.writable_mounts_after_the_crash, tally.crash_points,
+            "每个崩溃态都起了一次可写挂载：{tally:?}"
+        );
+        assert_eq!(
+            tally.crash_points_after_a_step_reading_under_injected_faults, 0,
+            "这段历史没有带读故障的步，每个崩溃态都判 C670 那一条：{tally:?}"
+        );
+        assert!(
+            injection
+                .crash_points
+                .iter()
+                .any(|crash_point| crash_point.step == StepPosition::StartingPoint)
+                && injection.crash_points.iter().any(|crash_point| {
+                    crash_point.operation_kind == Some(HistoryOperationKind::CloseAndMountWritable)
+                }),
+            "起点与关掉重挂那一步都要摆到崩溃态：{:?}",
+            injection
+                .crash_points
+                .iter()
+                .map(|crash_point| (crash_point.step, crash_point.operation_kind))
+                .collect::<Vec<_>>()
+        );
+        let new_findings: Vec<String> = injection
+            .new_findings
+            .iter()
+            .map(CrashPointFinding::render)
+            .collect();
+        assert_eq!(
+            tally.writable_mount_refusals_on_the_d23_list_illegal, 0,
+            "没注入读故障的崩溃态上可写挂载以见证判据为由拒、拒得不合法：{new_findings:?}"
+        );
+        assert!(
+            new_findings.is_empty(),
+            "没注入读故障的崩溃态上有新发现：{new_findings:?}"
+        );
+    }
 }
````

## `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`

````diff
--- crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	2026-10-02 23:57:36.669782210 +0000
+++ crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	2026-10-03 15:05:08.103117810 +0000
@@ -3151,6 +3151,12 @@
                 if (!lt64(highest_seen, watermark)) {
                     violate(INVARIANT_I_7_8);
                 }
+                // I-7.8 第二格：最新那条根自己的水位 > 盘上出现过的最大树 ID，与 CPU 的池级 checker 同判
+                // （`crates/singlefs-checker/src/walk.rs` 的 `check_pool_image` 末尾取 max 那一格之后那一格，C674）：
+                // 只把最新那条根的水位压低时取 max 那一格被更早的根盖住，而恢复按所选根的水位往下发号
+                if (!lt64(highest_seen, newest_root.watermark)) {
+                    violate(INVARIANT_I_7_8);
+                }
                 if (undecidable != 0u) {
                     return WALK_REACHED_THE_ROOT_RING;
                 }
````

## `crates/mutations.tsv`

````diff
--- crates/mutations.tsv	2026-10-03 13:33:07.452703829 +0000
+++ crates/mutations.tsv	2026-10-03 15:05:08.103659122 +0000
@@ -2212,3 +2212,18 @@
 代码轮 U13 映射树分裂：模型照旧按一个节点算中央映射树的占槽（根之下的节点不进占槽上界）	crates/singlefs-harness/src/model.rs	            slots_written += (mapping_nodes - 1) * ModelUnitRole::MappingTree.span_in_slots()	            slots_written += 0 * ModelUnitRole::MappingTree.span_in_slots()	-p singlefs-harness --lib -- model::tests::a_sequential_write_of_three_lower_leaves_counts_the_central_mapping_tree_nodes	a_sequential_write_of_three_lower_leaves_counts_the_central_mapping_tree_nodes_below_the_root_in_the_occupied_slots
 代码轮 U13 映射树分裂：中央映射树节点数的上界不随条目数涨（叶切出来的右半不算，树恒一个节点）	crates/singlefs-harness/src/model.rs	        + 2 * (entries / smallest_half_of_a_split(central_mapping_tree_leaf_entry_capacity()));	        + 0 * (entries / smallest_half_of_a_split(central_mapping_tree_leaf_entry_capacity()));	-p singlefs-harness --lib -- model::tests::central_mapping_tree_nodes_grow_by_two_per_147_entries	central_mapping_tree_nodes_grow_by_two_per_147_entries_and_keep_the_previous_bound
 LR5 丁-defer 选根一律不并：F 之下、按最新根的表有效的根引用的槽也不并进 I-3.1 的实际遍历；小世界穷举里原式与「被抛弃根引用的也并」那一式在换下槽的那一版读得出时就不同	crates/singlefs-checker/src/walk.rs	        RootStandingInTheNewestRootsTable::Valid => checkpoint_txg < effective_rollback_floor,	        RootStandingInTheNewestRootsTable::Valid => checkpoint_txg < effective_rollback_floor && false,	-p singlefs-checker --lib	deferred_selection_merging_the_abandoned_roots_too_selects_the_same_slots_on_every_small_world_but_the_double_fault_form
+C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：模型答卸载那一串的根带现行那一版自己的 F、不是它的 txg	crates/singlefs-harness/src/model.rs	        let floor_raised_to_the_current_txg = current.key.checkpoint_txg;	        let floor_raised_to_the_current_txg = current.rollback_floor;	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- unmount_on_the_first_file_at_txg_three	unmount_on_the_first_file_at_txg_three_pushes_two_empty_publishes_carrying_floor_three_and_the_model_agrees
+C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：实现卸载把 F 抬到现行那一版带的 F（原地不动）而不是它的 txg（E171 第 42 行的形态，由模型对拍判）	crates/singlefs-core/src/mount.rs	    let rollback_floor = current_file_version.root.checkpoint_txg;	    let rollback_floor = current_file_version.root.rollback_floor;	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version	unmount_on_the_first_file_at_txg_three_pushes_two_empty_publishes_carrying_floor_three_and_the_model_agrees
+C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：模型在树表 0 条的一版上也答卸载（条款没判的那一格不再停下）	crates/singlefs-harness/src/model.rs	        if !current.contents.has_trees() {\n            return Err(ModelDisagreement::new(\n                ModelDisagreementAspect::NotModeled,\n                format!(\n                    "模型里现行版本 {:?} 树表 0 条：卸载推不推空发布条款没判，模型不答",	        if false {\n            return Err(ModelDisagreement::new(\n                ModelDisagreementAspect::NotModeled,\n                format!(\n                    "模型里现行版本 {:?} 树表 0 条：卸载推不推空发布条款没判，模型不答",	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- the_model_does_not_answer_unmount_on_a_version_without_trees	the_model_does_not_answer_unmount_on_a_version_without_trees
+C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：对拍不比根带的 F（卸载那一串带错 F 不报）	crates/singlefs-harness/src/model.rs	            if expected.rollback_floor != observed.rollback_floor {	            if false {	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- an_unmount_whose_roots_carry_the_floor	an_unmount_whose_roots_carry_the_floor_of_the_current_version_instead_of_its_txg_is_reported_at_the_rollback_floor
+C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：模型答卸载之后会话照旧开着	crates/singlefs-harness/src/model.rs	        answer.session_after_success = ModelSessionAfterSuccess::Closed;	        answer.session_after_success = ModelSessionAfterSuccess::StaysOpen;	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- unmount_on_the_first_file_at_txg_three	unmount_on_the_first_file_at_txg_three_pushes_two_empty_publishes_carrying_floor_three_and_the_model_agrees
+C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：胶水只交卸载那一串的第一条根	crates/singlefs-harness/src/model_comparison.rs	            .map(observed_root_of_file_version)\n            .collect(),\n        Unmounted::NothingWrittenOnAVersionWithoutFile { .. } => Vec::new(),	            .map(observed_root_of_file_version)\n            .take(1)\n            .collect(),\n        Unmounted::NothingWrittenOnAVersionWithoutFile { .. } => Vec::new(),	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- unmount_on_the_first_file_at_txg_three	unmount_on_the_first_file_at_txg_three_pushes_two_empty_publishes_carrying_floor_three_and_the_model_agrees
+C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：模型的抬 F 那一串至多推 2 次（现行 txg 除以 3 余 1 时条款要 3 次）	crates/singlefs-harness/src/model.rs	            && u64::try_from(roots.len()).expect("次数") < ROOT_RING_REGIONS	            && u64::try_from(roots.len()).expect("次数") < 2	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- unmount_on_an_overwrite_at_txg_four	unmount_on_an_overwrite_at_txg_four_pushes_three_empty_publishes_carrying_floor_four_and_the_model_agrees
+C670（16-3）：没注入读故障的崩溃态上判据 N-配置 复算为真不报（崩溃注入第二截判官这一判丢了）	crates/singlefs-harness/src/mount_transparency.rs	                conditions_not_met.push(\n                    RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(\n                        branch,\n                    ),\n                );	                let _ = RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(\n                    branch,\n                );	-p singlefs-harness --test writable_mount_refusal_on_a_crash_state_without_read_faults	a_witness_refusal_on_a_crash_image_without_read_faults_where_only_a_rotation_landed_is_illegal
+C670（16-3）：撕裂系统配置那一支没分开（按甲把自证不过的槽也当见证判据为真，撕裂的拒一起判不合法）	crates/singlefs-harness/src/mount_transparency.rs	            if let Some(branch) = witnessed_counter_holds_on_a_crash_image_without_read_faults(\n                crash_image,\n                selected_version,\n            ) {	            if let Some(branch) = newer_publish_witnessed_on_the_crash_image(\n                crash_image,\n                selected_version,\n            )\n            .then_some(WitnessedCounterBranchThatHolds::NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable) {	-p singlefs-harness --test writable_mount_refusal_on_a_crash_state_without_read_faults	a_refusal_on_a_crash_image_without_read_faults_where_a_device_has_both_system_configuration_slots_torn_is_legal
+C670（16-3）：注入过读故障的崩溃态也判见证判据（越出挂载透明只管不带故障的状态那条射程）	crates/singlefs-harness/src/mount_transparency.rs	            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,\n            ReadFaultsInTheCrashState::NoneInjected,\n        ) => {	            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,\n            ReadFaultsInTheCrashState::NoneInjected | ReadFaultsInTheCrashState::Injected,\n        ) => {	-p singlefs-harness --test writable_mount_refusal_on_a_crash_state_without_read_faults	the_same_witness_refusal_in_a_crash_state_with_read_faults_injected_is_legal
+C670（16-3）：崩溃态一律判成没注入读故障（带读故障那一步之后的崩溃态也判见证判据）	crates/singlefs-checker-tier/src/crash_injection.rs	            if step_reads_under_injected_faults && step_sent_something_by_the_crash {	            if false && step_reads_under_injected_faults && step_sent_something_by_the_crash {	-p singlefs-checker-tier --lib -- crash_injection::tests::a_crash_state_counts_as_read_faulted_from_the_first_stream_step_of_an_abandoning_recovery_on	a_crash_state_counts_as_read_faulted_from_the_first_stream_step_of_an_abandoning_recovery_on
+C670（16-3）：带读故障那一步一个都没发也算注入过（取号之前就拒的抛弃根之后整段历史不判见证判据）	crates/singlefs-checker-tier/src/crash_injection.rs	            let step_sent_something_by_the_crash = operations_before_the_step\n                < *operations_after_the_step\n                && operations_before_the_step <= last_stream_index_of_the_crash_window;	            let step_sent_something_by_the_crash =\n                operations_before_the_step <= last_stream_index_of_the_crash_window;	-p singlefs-checker-tier --lib -- crash_injection::tests::an_abandoning_recovery_that_sent_nothing_leaves_every_crash_state_without_read_faults	an_abandoning_recovery_that_sent_nothing_leaves_every_crash_state_without_read_faults
+C670（16-3）判别力（E171 第 242 行）：发布的持久顺序把系统配置槽轮换挪到根槽 FUA 写之前，崩溃注入第二截判拒得不合法	crates/singlefs-core/src/transaction.rs	    writer.perform_commit_step(CommitStep::WriteRootRecordForceUnitAccess {\n        checkpoint_txg,\n        root_slot,\n    })?;\n    writer.perform_commit_step(CommitStep::RotateSystemConfigurationSlots {\n        journal_tail,\n        journal_instance,\n    })	    writer.perform_commit_step(CommitStep::RotateSystemConfigurationSlots {\n        journal_tail,\n        journal_instance,\n    })?;\n    writer.perform_commit_step(CommitStep::WriteRootRecordForceUnitAccess {\n        checkpoint_txg,\n        root_slot,\n    })	-p singlefs-checker-tier --lib -- crash_injection::tests::no_crash_state_without_read_faults_after_a_remount_is_refused_writable_for_a_witness_newer_than_the_selected_version	no_crash_state_without_read_faults_after_a_remount_is_refused_writable_for_a_witness_newer_than_the_selected_version
+C670（16-3）判别力（E171 第 269 行）：取号写的见证值取成槽世代号，崩溃注入第二截判拒得不合法	crates/singlefs-core/src/transaction.rs	        .map(|system_configuration| system_configuration.quantities.journal_tail)	        .map(|system_configuration| system_configuration.quantities.slot_generation)	-p singlefs-checker-tier --lib -- crash_injection::tests::no_crash_state_without_read_faults_after_a_remount_is_refused_writable_for_a_witness_newer_than_the_selected_version	no_crash_state_without_read_faults_after_a_remount_is_refused_writable_for_a_witness_newer_than_the_selected_version
+C674：GPU 判器不判最新那条根自己的水位（I-7.8 只剩根环取 max 那一格）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	                if (!lt64(highest_seen, newest_root.watermark)) {	                if (!lt64(highest_seen, newest_root.watermark) && false) {	-p singlefs-checker-tier --features gpu --test gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker	gpu_judge_reddens_a_newest_root_whose_own_tree_identifier_watermark_is_low_like_the_cpu_checker
````

## `crates/singlefs-harness/tests/model_answers_unmount_with_the_floor_at_the_current_version.rs`（新文件）

````diff
--- /dev/null	2026-09-29 10:41:02.505274808 +0000
+++ crates/singlefs-harness/tests/model_answers_unmount_with_the_floor_at_the_current_version.rs	2026-10-03 15:04:59.289240998 +0000
@@ -0,0 +1,336 @@
+//! 理想模型给正常卸载的答（`IdealModel::answer_unmount`）与对拍（`model_comparison::observed_unmount`）：卸载那一串带新 F 的空发布，
+//! 根上带的 F 是卸载开始时现行那一版的 txg（D16（发布语义） 已定项 1「正常卸载」与「抬 F 的上限」那一行「卸载抬 F」：抬到现行那一版的 txg，
+//! 不另判上限；射程：卸载之后 F = 现行那一版的 txg）；第一版几何（R = 3、区域归属 0 / 1 / 0）下现行 txg 除以 3 余 1 时推 3 次，
+//! 余 0 或余 2 时推 2 次。E171（三oracle合谋测试） 第 42 行（`crates/mutations.tsv` 里「卸载把 F 抬到现行那一版带的 F」那一行的形态）
+//! 此前三条 oracle 都放行：模型没有卸载的答，对拍停在卸载之前。
+//!
+//! 池：两块 4 GiB 的稀疏盘。mkfs → 第一次可写挂载（取号 1，零单元发布 txg 1 与暖机 txg 2，拿模型判过）→ 同一次挂载里第一个文件版本
+//! （txg 3，拿模型判过）→ 各条用例自己的后几步。每一步都由实现做、交回的东西换成观测拿模型判；判的是实现交出的东西，不另读盘。
+
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
+use singlefs_core::mount::{
+    mount_writable, unmount, Mounted, ShadowLedger, UnmountRaisedTheFloor, Unmounted,
+};
+use singlefs_core::transaction::{
+    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
+};
+use singlefs_harness::history::HistoryDeviceWidth;
+use singlefs_harness::memory_pool::SparseBlockDevice;
+use singlefs_harness::model::{
+    IdealModel, ModelAnswer, ModelCheckpointTxg, ModelDeviceIdentity, ModelDisagreement,
+    ModelDisagreementAspect, ModelJudgementCounts, ModelPoolGeometry, ObservedEffect,
+    ObservedOutcome,
+};
+use singlefs_harness::model_comparison::{
+    observed_mount, observed_root_of_file_version, observed_unmount,
+};
+
+const DEVICE_WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::FourGibibytes;
+
+/// 文件内容：一个数据单元装得下。
+const FILE_CONTENT_BYTES: usize = 3000;
+const FIRST_FILE_FILL_BYTE: u8 = 0x5A;
+const OVERWRITE_FILL_BYTE: u8 = 0xB0;
+const FIXED_WRITE_TIME_SECONDS: u64 = 1_700_000_000;
+
+/// 一次可写挂载之后的池与走到这一刻的模型（每一步都判过、对得上）。
+struct MountedPoolJudgedByTheModel {
+    parameters: MakeFilesystemParameters,
+    devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
+    mounted: Mounted,
+    model: IdealModel,
+}
+
+/// 拿模型判一步；对不上就带着哪一格 panic（证红时看的就是这一格）。
+fn judged_by_the_model(
+    model: &mut IdealModel,
+    answer: &ModelAnswer,
+    observed: &ObservedOutcome,
+) -> ModelJudgementCounts {
+    model
+        .judge_and_advance(answer, observed)
+        .unwrap_or_else(|disagreement: ModelDisagreement| {
+            panic!(
+                "模型与实现对不上：{}（{:?}）：模型 {}；实现 {}",
+                disagreement.aspect.name(),
+                disagreement.aspect,
+                disagreement.model_answer,
+                disagreement.implementation_answer
+            )
+        })
+}
+
+/// mkfs → 第一次可写挂载（零单元发布 txg 1、暖机 txg 2），挂载拿模型判过；现行那一版树表 0 条。
+fn first_mount_on_a_fresh_pool() -> MountedPoolJudgedByTheModel {
+    let parameters = DEVICE_WIDTH.parameters();
+    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
+        [DeviceIdentity(0), DeviceIdentity(1)]
+            .into_iter()
+            .map(|identity| {
+                (
+                    identity,
+                    SparseBlockDevice::new(
+                        DEVICE_WIDTH.device_bytes(),
+                        PhysicalBlockSizeInBytes(512),
+                    ),
+                )
+            })
+            .collect();
+    make_filesystem(&parameters, &mut devices).expect("两块 4 GiB 的稀疏盘上 mkfs");
+    let mut model = IdealModel::after_make_filesystem(ModelPoolGeometry {
+        devices: vec![ModelDeviceIdentity(0), ModelDeviceIdentity(1)],
+        device_size_in_bytes: DEVICE_WIDTH.device_bytes(),
+    });
+    let answer = model.answer_mount_writable();
+    let mounted = mount_writable(&parameters, &mut devices).expect("mkfs 之后第一次可写挂载");
+    judged_by_the_model(
+        &mut model,
+        &answer,
+        &ObservedOutcome::Succeeded(observed_mount(&mounted)),
+    );
+    MountedPoolJudgedByTheModel {
+        parameters,
+        devices,
+        mounted,
+        model,
+    }
+}
+
+/// 在 [`first_mount_on_a_fresh_pool`] 那一版（树表 0 条）上发第一个文件版本（txg 3），拿模型判过；交回它。
+fn publish_the_first_file_judged(pool: &mut MountedPoolJudgedByTheModel) -> TransactionOutput {
+    let (version_to_build_on, record_bytes) = match &pool.mounted.current {
+        PoolVersion::WithoutFile(version_without_file) => (
+            version_without_file.root,
+            version_without_file.record_bytes.clone(),
+        ),
+        PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => {
+            panic!("刚 mkfs 的池第一次挂载之后现行那一版树表 0 条")
+        }
+    };
+    let content = vec![FIRST_FILE_FILL_BYTE; FILE_CONTENT_BYTES];
+    let answer = pool
+        .model
+        .answer_publish_first_file(&content)
+        .expect("模型里第一次挂载的会话开着");
+    let first_file = {
+        let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
+        publish_first_file(
+            &mut writer,
+            &mut pool.mounted.allocator,
+            &version_to_build_on,
+            FirstFile {
+                content: &content,
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
+            },
+            pool.mounted.output.instance,
+            &record_bytes,
+        )
+        .expect("第一个文件版本")
+    };
+    judged_by_the_model(
+        &mut pool.model,
+        &answer,
+        &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
+            roots: vec![observed_root_of_file_version(&first_file)],
+            reported_ceiling: None,
+        }),
+    );
+    first_file
+}
+
+/// 在 `previous` 后面覆盖写一版（txg + 1），拿模型判过；交回它。
+fn publish_an_overwrite_judged(
+    pool: &mut MountedPoolJudgedByTheModel,
+    previous: &TransactionOutput,
+) -> TransactionOutput {
+    let content = vec![OVERWRITE_FILL_BYTE; FILE_CONTENT_BYTES];
+    let answer = pool
+        .model
+        .answer_publish_overwrite(&content)
+        .expect("模型里会话开着、现行那一版带文件");
+    let overwrite = {
+        let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
+        publish_overwrite(
+            &mut writer,
+            &mut pool.mounted.allocator,
+            previous,
+            FirstFile {
+                content: &content,
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
+            },
+            pool.mounted.output.instance,
+        )
+        .expect("覆盖写")
+    };
+    judged_by_the_model(
+        &mut pool.model,
+        &answer,
+        &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
+            roots: vec![observed_root_of_file_version(&overwrite)],
+            reported_ceiling: None,
+        }),
+    );
+    overwrite
+}
+
+/// 在现行那一版 `current` 上正常卸载（影子账开着，同随机历史的执行器）；推了那一串的交回它。
+fn unmount_raising_the_floor(
+    pool: &mut MountedPoolJudgedByTheModel,
+    current: TransactionOutput,
+) -> UnmountRaisedTheFloor {
+    let mut current = PoolVersion::WithFile(current);
+    let unmounted = unmount(
+        &pool.parameters,
+        &mut pool.devices,
+        &mut pool.mounted.allocator,
+        &mut current,
+        ShadowLedger::On,
+    )
+    .expect("正常卸载");
+    let Unmounted::FloorRaisedToTheCurrentVersion(raised) = unmounted else {
+        panic!("带文件的一版上卸载报成「树表 0 条、一个字节都不写」");
+    };
+    raised
+}
+
+/// 卸载那一串写出的根的 (txg, 根上带的 F)，按次序。
+fn txgs_and_floors_of(raised: &UnmountRaisedTheFloor) -> Vec<(u64, u64)> {
+    raised
+        .publishes
+        .iter()
+        .map(|publish| (publish.root.checkpoint_txg.0, publish.root.rollback_floor.0))
+        .collect()
+}
+
+/// 第一个文件版本 txg 3（除以 3 余 0）上卸载：推 2 次空发布（txg 4、5），两条根都带 F = 3；模型答的就是这两条，对拍比过两条根、
+/// 对得上；卸载之后模型里这个进程没有可写会话，F 生效值是 3。
+#[test]
+fn unmount_on_the_first_file_at_txg_three_pushes_two_empty_publishes_carrying_floor_three_and_the_model_agrees(
+) {
+    let mut pool = first_mount_on_a_fresh_pool();
+    let first_file = publish_the_first_file_judged(&mut pool);
+    assert_eq!(
+        (
+            first_file.root.checkpoint_txg,
+            first_file.root.rollback_floor
+        ),
+        (CheckpointTxg(3), CheckpointTxg(0)),
+        "第一次挂载写 txg 1、2，第一个文件版本 txg 3，F 还是 0"
+    );
+    let answer = pool
+        .model
+        .answer_unmount()
+        .expect("模型里会话开着、现行那一版有树");
+    let raised = unmount_raising_the_floor(&mut pool, first_file);
+    let txgs_and_floors_handed_in = txgs_and_floors_of(&raised);
+    // 先拿模型判（对拍那一格先红），再钉实现交回的数。
+    let counts = judged_by_the_model(
+        &mut pool.model,
+        &answer,
+        &ObservedOutcome::Succeeded(observed_unmount(
+            &Unmounted::FloorRaisedToTheCurrentVersion(raised),
+        )),
+    );
+    assert_eq!(
+        (counts.successes_matched, counts.roots_compared),
+        (1, 2),
+        "卸载那一步做成、比过两条根：{counts:?}"
+    );
+    assert_eq!(
+        txgs_and_floors_handed_in,
+        [(4, 3), (5, 3)],
+        "现行 txg 3 除以 3 余 0：卸载推 2 次，每条根带 F = 现行那一版的 txg 3"
+    );
+    assert!(
+        !pool.model.has_writable_session(),
+        "卸载之后这个进程没有可写会话"
+    );
+    assert_eq!(
+        pool.model.effective_rollback_floor(),
+        ModelCheckpointTxg(3),
+        "卸载之后 F 生效值 = 卸载开始时现行那一版的 txg"
+    );
+}
+
+/// 第一个文件版本之后覆盖写一版（txg 4，除以 3 余 1）再卸载：推 3 次空发布（txg 5、6、7；txg 5、6 落盘 0，txg 7 才落到盘 1），
+/// 三条根都带 F = 4；模型答的也是 3 条，对拍对得上。
+#[test]
+fn unmount_on_an_overwrite_at_txg_four_pushes_three_empty_publishes_carrying_floor_four_and_the_model_agrees(
+) {
+    let mut pool = first_mount_on_a_fresh_pool();
+    let first_file = publish_the_first_file_judged(&mut pool);
+    let overwrite = publish_an_overwrite_judged(&mut pool, &first_file);
+    assert_eq!(
+        overwrite.root.checkpoint_txg,
+        CheckpointTxg(4),
+        "覆盖写接在 txg 3 后面"
+    );
+    let answer = pool
+        .model
+        .answer_unmount()
+        .expect("模型里会话开着、现行那一版有树");
+    let raised = unmount_raising_the_floor(&mut pool, overwrite);
+    let txgs_and_floors_handed_in = txgs_and_floors_of(&raised);
+    // 先拿模型判（对拍那一格先红），再钉实现交回的数。
+    let counts = judged_by_the_model(
+        &mut pool.model,
+        &answer,
+        &ObservedOutcome::Succeeded(observed_unmount(
+            &Unmounted::FloorRaisedToTheCurrentVersion(raised),
+        )),
+    );
+    assert_eq!(
+        (counts.successes_matched, counts.roots_compared),
+        (1, 3),
+        "卸载那一步做成、比过三条根：{counts:?}"
+    );
+    assert_eq!(
+        txgs_and_floors_handed_in,
+        [(5, 4), (6, 4), (7, 4)],
+        "现行 txg 4 除以 3 余 1：卸载推 3 次，每条根带 F = 4"
+    );
+}
+
+/// 卸载那一串的根带的是现行那一版自己带的 F（0），不是它的 txg（3）——E171 第 42 行那种实现交回的样子（在交回的根上改出来）：
+/// 对拍报「根带的 F」那一格。
+#[test]
+fn an_unmount_whose_roots_carry_the_floor_of_the_current_version_instead_of_its_txg_is_reported_at_the_rollback_floor(
+) {
+    let mut pool = first_mount_on_a_fresh_pool();
+    let first_file = publish_the_first_file_judged(&mut pool);
+    let floor_of_the_current_version = first_file.root.rollback_floor;
+    let answer = pool
+        .model
+        .answer_unmount()
+        .expect("模型里会话开着、现行那一版有树");
+    let mut raised = unmount_raising_the_floor(&mut pool, first_file);
+    for publish in &mut raised.publishes {
+        publish.root.rollback_floor = floor_of_the_current_version;
+    }
+    let observed = ObservedOutcome::Succeeded(observed_unmount(
+        &Unmounted::FloorRaisedToTheCurrentVersion(raised),
+    ));
+    assert_eq!(
+        pool.model
+            .judge_and_advance(&answer, &observed)
+            .map(|_| ())
+            .map_err(|disagreement| disagreement.aspect),
+        Err(ModelDisagreementAspect::RollbackFloor)
+    );
+}
+
+/// 现行那一版树表 0 条（第一次挂载之后还没写过文件）：条款写「推一串空发布」，实现一个字节都不写（主 agent 2026-09-26 定、没进 D16），
+/// 两种答法条款没判，模型不答，报「模型答不了」。
+#[test]
+fn the_model_does_not_answer_unmount_on_a_version_without_trees() {
+    let pool = first_mount_on_a_fresh_pool();
+    assert_eq!(
+        pool.model
+            .answer_unmount()
+            .map(|_| ())
+            .map_err(|disagreement| disagreement.aspect),
+        Err(ModelDisagreementAspect::NotModeled)
+    );
+}
````

## `crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs`（新文件）

````diff
--- /dev/null	2026-09-29 10:41:02.505274808 +0000
+++ crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs	2026-10-03 15:05:03.378625646 +0000
@@ -0,0 +1,313 @@
+//! C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）：崩溃注入第二截判拒可写合不合法时，崩溃态没注入读故障、拒的理由是
+//! D23（journal 的角色与格式） 已定项 14 的判据 N-配置，而判据在崩溃镜像上复算（撕裂的系统配置槽跳过）仍为真 ⇒ 拒得不合法
+//! （`singlefs_harness::mount_transparency::judge_writable_mount_refusal_of_a_crash_state`）。注入过读故障的崩溃态不判这一条
+//! （D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）；撕裂系统配置那一支照 U6 判合法（C664、C331）。
+//! 小池是单元区 384 槽的两块内存盘，第一个文件之后的盘面；崩溃镜像是内存盘面本身，读不报错。
+//! 见证判据为真的那份盘面取法照 E171（三oracle合谋测试） 第 242 行（系统配置轮换先于根槽 FUA 落盘）：一次覆盖写那次发布只落了系统配置轮换，
+//! 它的单元、记录与根都没落。
+
+use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::mount::{
+    mount_writable_with_space_admission, MountError, StillUnreadableAfterOneReread,
+    WitnessedCounterComparison,
+};
+use singlefs_core::recovery::PoolReader;
+use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
+use singlefs_harness::history::{
+    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
+    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker,
+};
+use singlefs_harness::memory_pool::{
+    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
+};
+use singlefs_harness::mount_transparency::{
+    judge_writable_mount_refusal_of_a_crash_state, read_recovery_fields, ComparedVersion,
+    DiskAfterTheRefusal, ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage,
+    RefusalConditionNotMet, RefusalReasonOnTheD23List, WitnessedCounterBranchThatHolds,
+    WritableMountRefusalVerdict,
+};
+use singlefs_harness::segments::{FixedGeometry, StepKind};
+use singlefs_harness::{RetainedOperation, SharedStream};
+
+const SMALL: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;
+/// 撕裂一个系统配置槽时改的那个字节在槽里的位置：槽中间，落在整槽校验和罩着的范围里。
+const TORN_BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT: usize = 2048;
+
+/// 从第一个文件起、再跑 `operations` 的录制流（内容保留）。
+fn recorded_stream_after_the_first_file_then(
+    operations: Vec<HistoryOperation>,
+) -> Vec<RetainedOperation> {
+    let stream = SharedStream::retaining_contents();
+    let history = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations,
+    };
+    let run = execute_history_with(
+        &history,
+        HistoryExecution {
+            per_step_checker: PerStepChecker::Skipped,
+            device_width: SMALL,
+            space_admission: SpaceAdmission::JudgedByTheFormula,
+        },
+        &stream,
+        &mut |_observation| {},
+    );
+    assert_eq!(
+        run.outcomes.len(),
+        history.operations.len(),
+        "前提：历史每一步都跑完：{:?}",
+        run.ending
+    );
+    stream.retained_operations()
+}
+
+fn pool_of(operations: &[RetainedOperation]) -> MemoryPool {
+    let mut pool = MemoryPool::with_devices(
+        &[DeviceIdentity(0), DeviceIdentity(1)],
+        SMALL.device_bytes(),
+    );
+    pool.apply(operations);
+    pool
+}
+
+/// 第一个文件之后的盘面上，只落了一次覆盖写那次发布的系统配置轮换（每块盘一槽，journal tail 写的是这次发布末条记录的计数器），
+/// 它的单元、记录与根都没落：系统配置见证过一次比盘上任何一版都新的发布。崩溃镜像读不报错。
+fn crash_image_where_only_the_rotation_of_an_overwrite_landed() -> (MemoryPool, MemoryPool) {
+    let first_file = recorded_stream_after_the_first_file_then(Vec::new());
+    let overwrite =
+        recorded_stream_after_the_first_file_then(vec![HistoryOperation::PublishOverwrite(
+            ContentChoice {
+                length: ContentLength::InsideOneDataUnit { selector: 0 },
+                fill_seed: 1,
+            },
+        )]);
+    assert_eq!(
+        overwrite[..first_file.len()],
+        first_file[..],
+        "前提：同一个起点跑出同一段录制流"
+    );
+    let parameters = SMALL.parameters();
+    let geometry = FixedGeometry {
+        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
+        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
+        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
+    };
+    let (writes_of_the_overwrite, _segments, _stream_indexes) =
+        writes_and_segments_with_stream_indexes(&overwrite[first_file.len()..], &geometry);
+    let rotation: Vec<RetainedWrite> = writes_of_the_overwrite
+        .iter()
+        .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
+        .cloned()
+        .collect();
+    assert_eq!(
+        rotation.len(),
+        2,
+        "前提：覆盖写那次发布每块盘轮换一个系统配置槽"
+    );
+    assert!(
+        writes_of_the_overwrite
+            .iter()
+            .any(|write| write.kind == StepKind::RootRecordFua)
+            && writes_of_the_overwrite
+                .iter()
+                .any(|write| write.kind == StepKind::JournalRecord),
+        "前提：覆盖写那次发布写了记录与根（这份盘面上都不落）"
+    );
+    let healthy = pool_of(&first_file);
+    let mut crash_image = healthy.clone();
+    crash_image.apply_writes(&rotation);
+    (healthy, crash_image)
+}
+
+/// 第一个文件之后的盘面上，盘 0 两个系统配置槽各改一个字节（读得出、自证不过，设备不报错）：撕裂的系统配置，盘 0 一份自证过的都没有。
+fn crash_image_where_both_system_configuration_slots_of_device_zero_are_torn(
+) -> (MemoryPool, MemoryPool) {
+    let healthy = pool_of(&recorded_stream_after_the_first_file_then(Vec::new()));
+    let slot_spacing = u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing);
+    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
+    let mut crash_image = healthy.clone();
+    for slot in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
+        let offset = DeviceOffsetInBytes(slot * slot_spacing);
+        let mut bytes = healthy
+            .read(DeviceIdentity(0), offset, slot_bytes)
+            .expect("内存盘上系统配置槽读得出");
+        bytes[TORN_BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT] ^= 0xFF;
+        crash_image
+            .devices
+            .get_mut(&DeviceIdentity(0))
+            .expect("有盘 0")
+            .write(offset, &bytes);
+    }
+    (healthy, crash_image)
+}
+
+/// 在 `crash_image` 的副本上起可写挂载（读不报错）：拒了交回错误与拒之后的盘面，做成了 panic。
+fn refused_writable_mount_on(crash_image: &MemoryPool) -> (MountError, MemoryPool) {
+    let parameters = SMALL.parameters();
+    let physical_block_size = PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size);
+    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = crash_image
+        .devices
+        .iter()
+        .map(|(identity, sectors)| {
+            let mut device =
+                SparseBlockDevice::new(crash_image.device_size_in_bytes, physical_block_size);
+            device.image = sectors.clone();
+            (*identity, device)
+        })
+        .collect();
+    let error = match mount_writable_with_space_admission(
+        &parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+    ) {
+        Err(error) => error,
+        Ok(mounted) => panic!(
+            "前提：这份盘面上可写挂载要拒，却挂上了实例 {:?}",
+            mounted.output.instance
+        ),
+    };
+    let after = MemoryPool {
+        devices: devices
+            .into_iter()
+            .map(|(identity, device)| (identity, device.image))
+            .collect(),
+        device_size_in_bytes: crash_image.device_size_in_bytes,
+    };
+    (error, after)
+}
+
+/// 拒因是判据 N-配置 为真、重读仍为真时，重读那一遍拿 c_见证 跟什么比的（别的拒因交 `None`）。
+fn witnessed_counter_comparison_on_the_reread(
+    error: &MountError,
+) -> Option<WitnessedCounterComparison> {
+    let MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) = error else {
+        return None;
+    };
+    let StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
+        reread,
+        ..
+    } = still_unreadable.as_ref()
+    else {
+        return None;
+    };
+    Some(reread.witness.comparison)
+}
+
+fn disk_after(crash_image: &MemoryPool, after: &MemoryPool) -> DiskAfterTheRefusal {
+    if crash_image == after {
+        DiskAfterTheRefusal::ByteForByteUnchanged
+    } else {
+        DiskAfterTheRefusal::Changed
+    }
+}
+
+/// 崩溃镜像上只读恢复读出的与第一个文件那一版逐字节相同：在允许集合里（那次覆盖写没确认过，第一个文件那一版是允许的）。
+fn read_only_read_back_against_the_first_file(
+    healthy: &MemoryPool,
+    crash_image: &MemoryPool,
+) -> ReadOnlyReadBackOfTheCrashImage {
+    let content_of = |pool: &MemoryPool| {
+        read_recovery_fields(pool, ComparedVersion::TheRecoveredVersion)
+            .compared_version
+            .map(|version| version.content)
+    };
+    if content_of(crash_image) == content_of(healthy) {
+        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet
+    } else {
+        ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet
+    }
+}
+
+/// 只落了覆盖写那次轮换的盘面，没注入读故障：所选那一版是第一个文件那一版，c_见证 大于它末条记录的计数器，正确实现在这份盘面上永远拒可写——
+/// 拒的理由在单子上、判据复算为真、盘没变、读出在允许集合里，U6 四条都满足，C670 那一条判不合法。
+#[test]
+fn a_witness_refusal_on_a_crash_image_without_read_faults_where_only_a_rotation_landed_is_illegal()
+{
+    let (healthy, crash_image) = crash_image_where_only_the_rotation_of_an_overwrite_landed();
+    let (error, after) = refused_writable_mount_on(&crash_image);
+    assert!(
+        matches!(
+            witnessed_counter_comparison_on_the_reread(&error),
+            Some(WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord { .. })
+        ),
+        "前提：判据 N-配置 拿 c_见证 与所选那一版的末条比为真、重读仍真，拒：{error:?}"
+    );
+    let read_only_read_back = read_only_read_back_against_the_first_file(&healthy, &crash_image);
+    assert_eq!(
+        read_only_read_back,
+        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+        "前提：只读恢复读出第一个文件那一版"
+    );
+    let fields = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
+    assert_eq!(
+        judge_writable_mount_refusal_of_a_crash_state(
+            &crash_image,
+            fields.report.effective_root,
+            &error,
+            disk_after(&crash_image, &after),
+            read_only_read_back,
+            ReadFaultsInTheCrashState::NoneInjected,
+        ),
+        WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
+            conditions_not_met: vec![
+                RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(
+                    WitnessedCounterBranchThatHolds::WitnessedCounterAboveTheLastRecordOfTheSelectedVersion
+                ),
+            ],
+        }
+    );
+}
+
+/// 同一份只落了轮换的盘面，调用方说这个崩溃态注入过读故障：C670 那一条不判，只判 U6 四条，合法。
+#[test]
+fn the_same_witness_refusal_in_a_crash_state_with_read_faults_injected_is_legal() {
+    let (healthy, crash_image) = crash_image_where_only_the_rotation_of_an_overwrite_landed();
+    let (error, after) = refused_writable_mount_on(&crash_image);
+    let fields = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
+    assert_eq!(
+        judge_writable_mount_refusal_of_a_crash_state(
+            &crash_image,
+            fields.report.effective_root,
+            &error,
+            disk_after(&crash_image, &after),
+            read_only_read_back_against_the_first_file(&healthy, &crash_image),
+            ReadFaultsInTheCrashState::Injected,
+        ),
+        WritableMountRefusalVerdict::Legal(
+            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
+        )
+    );
+}
+
+/// 撕裂系统配置那一支：盘 0 两槽都撕裂、没注入读故障，实现在取号之前以「这块盘一份自证过的系统配置都没有」拒（报成判据 N-配置 的拒因，C664）；
+/// 跳过撕裂的槽复算判据为假（c_见证 等于第一个文件那一版末条的计数器），C670 那一条不报，U6 照甲把撕裂判成按真：合法。
+#[test]
+fn a_refusal_on_a_crash_image_without_read_faults_where_a_device_has_both_system_configuration_slots_torn_is_legal(
+) {
+    let (healthy, crash_image) =
+        crash_image_where_both_system_configuration_slots_of_device_zero_are_torn();
+    let (error, after) = refused_writable_mount_on(&crash_image);
+    assert_eq!(
+        witnessed_counter_comparison_on_the_reread(&error),
+        Some(WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot),
+        "前提：盘 0 一份自证过的系统配置都没有，照判据 N-配置 的拒因拒：{error:?}"
+    );
+    let fields = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
+    assert_eq!(
+        judge_writable_mount_refusal_of_a_crash_state(
+            &crash_image,
+            fields.report.effective_root,
+            &error,
+            disk_after(&crash_image, &after),
+            read_only_read_back_against_the_first_file(&healthy, &crash_image),
+            ReadFaultsInTheCrashState::NoneInjected,
+        ),
+        WritableMountRefusalVerdict::Legal(
+            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
+        )
+    );
+}
````

## `crates/singlefs-checker-tier/tests/gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs`（新文件）

````diff
--- /dev/null	2026-09-29 10:41:02.505274808 +0000
+++ crates/singlefs-checker-tier/tests/gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs	2026-10-03 15:05:08.103117810 +0000
@@ -0,0 +1,343 @@
+//! checker 档模块：crash、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
+//! GPU 判器里池级 checker 那一份的 I-7.8（根记录树 ID 水位不低于全池最大树 ID）判两格，与 CPU 的池级 checker 同判（C674）：
+//! 根环全部根的水位取 max 那一格，与最新那条根自己的水位那一格（`crates/singlefs-checker/src/walk.rs` 的 `check_pool_image` 末尾）。
+//! 造法：新池新建文件（发布 A，发出十二棵树）再覆盖写一次（发布 B）那条流，把 B 的根（最新那条）的水位压到 mkfs 时的水位、重封，
+//! B 那次的 journal 记录的新根段跟着压（同一份新根段，I-8.10（记录新根段与同代根相同） 不跟着红）。A 的根水位与 B 原样相同、
+//! 照旧盖得住 A 发的树 ⇒ 根环取 max 不变、取 max 那一格不红，只有最新那条根自己那一格红（前提在取卡之前用 CPU 判；
+//! 与 harness 档 `checker_known_bad_images.rs` 里 I-7.8 最新那条根那一格的坏镜像同一个坏法，那一份钉 CPU 判器、这一份钉 GPU 判器）。
+//! B 的根落了的状态上 CPU 判 I-7.8 红，GPU 与 CPU 逐状态的违例位要相同；GPU 只判取 max 那一格时这些状态上少一位 I-7.8。
+//! 要本机一张空闲显存够的 NVIDIA 独显：一张独显适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
+//! （`gpu_unit_checks::gpu_test_start`）。要 `gpu` 特性。
+#![cfg(feature = "gpu")]
+
+#[path = "../../singlefs-harness/tests/common/mod.rs"]
+mod common;
+
+use std::collections::BTreeMap;
+
+use common::{
+    build_pool, file_content, geometry, overwrite_in_process_as_instance_one_and_advance_the_pool,
+    parameters, CONTENT_BYTES_OF_PUBLISH_B,
+};
+use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
+use singlefs_checker::walk::check_pool_image;
+use singlefs_checker_tier::crash::{
+    judge_layer0_state_range, layer0_plan_state_count, Layer0SegmentExpansion, Layer0StateJudgement,
+};
+use singlefs_checker_tier::crash_judge_dispatch::GpuCrashJudge;
+use singlefs_checker_tier::crash_judge_gpu::{
+    pool_checker_violated_bits, verdict_bits, POOL_CHECKER_UNDECIDABLE_WORD, VERDICT_WORDS,
+};
+use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
+use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::first_usable_card_or_report_not_run;
+use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::journal::JournalRecord;
+use singlefs_core::root_record::RootRecord;
+use singlefs_core::unit::unit_filesystem_identifier;
+use singlefs_format::{JOURNAL_RECORD_BYTES, TREE_IDENTIFIER_WATERMARK_AT_MKFS};
+use singlefs_harness::memory_pool::{
+    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
+    WrittenContents,
+};
+use singlefs_harness::segments::StepKind;
+
+/// 段长不超过这个数的段逐个真子集展开（与 GPU 对拍那一份的默认展开上限同一档）。
+const EXPAND_UP_TO: usize = 8;
+/// B 的根压到的水位：mkfs 时的水位，不高于 A 发出的树 ID。
+const LOWERED_TREE_IDENTIFIER_WATERMARK: u64 = TREE_IDENTIFIER_WATERMARK_AT_MKFS;
+/// 池级 checker 判最新那条根自己的水位那一格的违例说明开头（`walk.rs` 的 `check_pool_image` 末尾）；取 max 那一格的说明以「根环水位最大」开头。
+/// 同一条不变量只留第一格违例的说明，取 max 那一格先判：说明以这一句开头 ⟺ 取 max 那一格没红、最新那条根那一格红了。
+const NEWEST_ROOT_WATERMARK_VIOLATION_PREFIX: &str = "最新那条根";
+
+/// 一条流：mkfs 之后的基镜像、写表与段、判的那条根、模型认的版本。
+struct RecordedFlow {
+    base: MemoryPool,
+    writes: Vec<RetainedWrite>,
+    segments: Vec<Vec<usize>>,
+    judged_root_index: usize,
+    versions: Vec<PublishedVersion>,
+}
+
+/// B 的根（流里最后那条根槽写）的水位怎么放。
+enum NewestRootWatermark {
+    AsPublished,
+    LoweredToTheMkfsWatermark,
+}
+
+fn root_record_of_the_write(write: &RetainedWrite) -> RootRecord {
+    let bytes = write.bytes().expect("根槽写带字节");
+    let root = RootRecord::parse_slot(bytes, &parameters().filesystem_identifier)
+        .expect("流里的根槽写自证得过");
+    assert_eq!(
+        root.to_slot(bytes.len()),
+        bytes,
+        "根记录解出再写回逐字节相同：改水位之后重写的槽只差水位与校验和"
+    );
+    root
+}
+
+/// 覆盖写 B 那一版的内容：与 A 的不同。
+fn content_of_publish_b() -> Vec<u8> {
+    (0..CONTENT_BYTES_OF_PUBLISH_B)
+        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
+        .collect()
+}
+
+/// 新池新建文件（A）再覆盖写一次（B）那条流，从 mkfs 之后起；`newest_root_watermark` 是压低时，B 的根的水位换成
+/// [`LOWERED_TREE_IDENTIFIER_WATERMARK`]、重封，流里 B 那次（同 (实例, txg)）的每条 journal 记录的新根段水位跟着换、重封。
+fn overwrite_flow(tag: &str, newest_root_watermark: NewestRootWatermark) -> RecordedFlow {
+    let mut pool = build_pool(tag);
+    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content_of_publish_b());
+    let operations = pool.retained_operations();
+    let (mut writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
+        &operations[pool.mkfs_operation_count..],
+        &geometry(),
+    );
+    let judged_root_index = writes
+        .iter()
+        .rposition(|write| write.kind == StepKind::RootRecordFua)
+        .expect("发布有一条根槽写");
+    let publish_a_root_index = writes[..judged_root_index]
+        .iter()
+        .rposition(|write| write.kind == StepKind::RootRecordFua)
+        .expect("B 之前有 A 的根槽写");
+    let publish_root = root_record_of_the_write(&writes[judged_root_index]);
+    let publish_a_root = root_record_of_the_write(&writes[publish_a_root_index]);
+    match newest_root_watermark {
+        NewestRootWatermark::AsPublished => {}
+        NewestRootWatermark::LoweredToTheMkfsWatermark => {
+            assert_eq!(
+                publish_a_root.tree_identifier_watermark, publish_root.tree_identifier_watermark,
+                "A 的根的水位与 B 的相同：只压 B 的根，根环里全部根取 max 不变"
+            );
+            assert!(
+                LOWERED_TREE_IDENTIFIER_WATERMARK < publish_root.tree_identifier_watermark,
+                "压到的水位 {LOWERED_TREE_IDENTIFIER_WATERMARK} 低于 B 的根原来的 {}",
+                publish_root.tree_identifier_watermark
+            );
+            let slot_bytes = writes[judged_root_index]
+                .bytes()
+                .expect("根槽写带字节")
+                .len();
+            let mut lowered_root = publish_root;
+            lowered_root.tree_identifier_watermark = LOWERED_TREE_IDENTIFIER_WATERMARK;
+            writes[judged_root_index].contents =
+                WrittenContents::Bytes(lowered_root.to_slot(slot_bytes));
+            let journal_filesystem_identifier =
+                unit_filesystem_identifier(&parameters().filesystem_identifier);
+            // 每块盘上 B 那次记录里最大的计数器，与流里同实例的记录里最大的计数器：后者更大就有记录要按改后的记录头重算反向链。
+            let mut highest_rewritten_counter_on_each_device = BTreeMap::new();
+            let mut highest_counter_of_the_instance_on_each_device = BTreeMap::new();
+            for write in writes
+                .iter_mut()
+                .filter(|write| write.kind == StepKind::JournalRecord)
+            {
+                let bytes = write.bytes().expect("记录写带字节");
+                assert_eq!(
+                    u64::try_from(bytes.len()).expect("记录写的长度装得进 u64"),
+                    JOURNAL_RECORD_BYTES,
+                    "一次记录写恰好一条记录"
+                );
+                let mut record = JournalRecord::parse(bytes, journal_filesystem_identifier)
+                    .expect("流里的记录写自证得过");
+                assert_eq!(
+                    record.to_bytes(),
+                    bytes,
+                    "记录解出再写回逐字节相同：改水位之后重写的记录只差新根段水位与校验和"
+                );
+                if record.instance != publish_root.instance {
+                    continue;
+                }
+                let highest_of_the_instance = highest_counter_of_the_instance_on_each_device
+                    .entry(write.device)
+                    .or_insert(record.counter);
+                *highest_of_the_instance = (*highest_of_the_instance).max(record.counter);
+                if record.checkpoint_txg != publish_root.checkpoint_txg {
+                    continue;
+                }
+                assert_eq!(
+                    record.new_tree_identifier_watermark, publish_root.tree_identifier_watermark,
+                    "B 那次记录的新根段水位与 B 的根相同"
+                );
+                record.new_tree_identifier_watermark = LOWERED_TREE_IDENTIFIER_WATERMARK;
+                write.contents = WrittenContents::Bytes(record.to_bytes());
+                let highest_rewritten = highest_rewritten_counter_on_each_device
+                    .entry(write.device)
+                    .or_insert(record.counter);
+                *highest_rewritten = (*highest_rewritten).max(record.counter);
+            }
+            assert_eq!(
+                highest_rewritten_counter_on_each_device.len(),
+                2,
+                "两块盘上各有 B 那次的记录"
+            );
+            assert_eq!(
+                highest_rewritten_counter_on_each_device,
+                highest_counter_of_the_instance_on_each_device,
+                "B 那次的记录是各盘上本实例最后的记录：后面没有记录的反向链要跟着重算"
+            );
+        }
+    }
+    RecordedFlow {
+        base: pool.memory_pool_after_mkfs(),
+        writes,
+        segments,
+        judged_root_index,
+        versions: vec![
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(3),
+                content: file_content(),
+            },
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(4),
+                content: content_of_publish_b(),
+            },
+        ],
+    }
+}
+
+/// 全部写都落了的那个镜像上，CPU 池级 checker 判红的不变量与各自第一格违例的说明。
+fn violations_with_every_write_landed(flow: &RecordedFlow) -> BTreeMap<&'static str, String> {
+    let mut image = flow.base.clone();
+    image.apply_writes(&flow.writes);
+    check_pool_image(&image)
+        .into_iter()
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
+            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+        })
+        .collect()
+}
+
+/// CPU 池级 checker 的违例位：位 i ↔ `IMPLEMENTED_INVARIANTS[i]`（与 GPU 判器结论里那一段同一个编号）。
+fn pool_checker_bits_on_the_cpu(judgement: &Layer0StateJudgement) -> u128 {
+    judgement
+        .pool_checker
+        .iter()
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(_) => Some(
+                IMPLEMENTED_INVARIANTS
+                    .iter()
+                    .position(|candidate| candidate == invariant)
+                    .expect("池级 checker 报的不变量在清单里"),
+            ),
+            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+        })
+        .fold(0u128, |bits, index| bits | (1u128 << index))
+}
+
+fn invariant_names(bits: u128) -> Vec<&'static str> {
+    IMPLEMENTED_INVARIANTS
+        .iter()
+        .enumerate()
+        .filter(|(index, _)| bits & (1u128 << index) != 0)
+        .map(|(_, name)| *name)
+        .collect()
+}
+
+#[test]
+fn gpu_judge_reddens_a_newest_root_whose_own_tree_identifier_watermark_is_low_like_the_cpu_checker()
+{
+    // 前提（取卡之前、CPU 上判）：原样的流全部落盘时 I-7.8 不红；压低之后只红 I-7.8，而且红的是最新那条根自己那一格、
+    // 取 max 那一格没红——这条流只在这一格上分得出 GPU 判没判这一格。
+    let as_published = violations_with_every_write_landed(&overwrite_flow(
+        "gpu-judge-newest-root-watermark-as-published",
+        NewestRootWatermark::AsPublished,
+    ));
+    assert!(
+        !as_published.contains_key("I-7.8"),
+        "原样的流全部落盘时 I-7.8 不红，实际 {as_published:?}"
+    );
+    let flow = overwrite_flow(
+        "gpu-judge-newest-root-watermark-lowered",
+        NewestRootWatermark::LoweredToTheMkfsWatermark,
+    );
+    let lowered = violations_with_every_write_landed(&flow);
+    assert_eq!(
+        lowered.keys().copied().collect::<Vec<_>>(),
+        vec!["I-7.8"],
+        "压低最新那条根的水位之后只红 I-7.8，实际 {lowered:?}"
+    );
+    assert!(
+        lowered["I-7.8"].starts_with(NEWEST_ROOT_WATERMARK_VIOLATION_PREFIX),
+        "红的是最新那条根自己那一格、取 max 那一格没红，实际说明「{}」",
+        lowered["I-7.8"]
+    );
+    let Some(card) = first_usable_card_or_report_not_run(
+        "gpu_judge_reddens_a_newest_root_whose_own_tree_identifier_watermark_is_low_like_the_cpu_checker",
+    ) else {
+        return;
+    };
+    let expansion = |_segment_index: usize, segment: &[usize]| {
+        if segment.len() <= EXPAND_UP_TO {
+            Layer0SegmentExpansion::EveryProperSubset
+        } else {
+            Layer0SegmentExpansion::NotExpanded
+        }
+    };
+    let tables = build_judge_tables(&JudgeFlow {
+        base: &flow.base,
+        writes: &flow.writes,
+        segments: &flow.segments,
+        judged_root_index: flow.judged_root_index,
+        versions: &flow.versions,
+        expansion: &expansion,
+    })
+    .expect("建得出表");
+    let state_count = layer0_plan_state_count(&flow.base, &flow.writes, &flow.segments, &expansion);
+    assert_eq!(tables.state_count, state_count);
+    let gpu: Vec<[u32; VERDICT_WORDS]> =
+        GpuCrashJudge::new(&card, &tables).judge_states(0..state_count);
+    let mut cpu: Vec<u128> = Vec::new();
+    let judged_on_the_cpu = judge_layer0_state_range(
+        &flow.base,
+        &flow.writes,
+        &flow.segments,
+        flow.judged_root_index,
+        &flow.versions,
+        &expansion,
+        0..state_count,
+        &mut |_ordinal, _image, judgement| cpu.push(pool_checker_bits_on_the_cpu(judgement)),
+    );
+    assert_eq!(judged_on_the_cpu, state_count);
+    assert_eq!(cpu.len(), gpu.len(), "两边判的状态数相同");
+    let tree_identifier_watermark_bit = 1u128
+        << IMPLEMENTED_INVARIANTS
+            .iter()
+            .position(|invariant| *invariant == "I-7.8")
+            .expect("I-7.8 在清单里");
+    let states_red_on_the_tree_identifier_watermark_on_the_cpu = cpu
+        .iter()
+        .filter(|bits| *bits & tree_identifier_watermark_bit != 0)
+        .count();
+    assert!(
+        states_red_on_the_tree_identifier_watermark_on_the_cpu > 0,
+        "展开到的状态里要有 B 的根落了、CPU 判 I-7.8 红的，这条流才分得出 GPU 判没判最新那条根那一格"
+    );
+    let mismatches: Vec<String> = cpu
+        .iter()
+        .zip(&gpu)
+        .enumerate()
+        .filter_map(|(ordinal, (cpu_bits, gpu_words))| {
+            let undecidable = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
+                || gpu_words[POOL_CHECKER_UNDECIDABLE_WORD] != 0;
+            let gpu_bits = pool_checker_violated_bits(gpu_words);
+            (undecidable || gpu_bits != *cpu_bits).then(|| {
+                format!(
+                    "状态 {ordinal}：CPU {:?}；GPU {:?}（判不了：{undecidable}）",
+                    invariant_names(*cpu_bits),
+                    invariant_names(gpu_bits)
+                )
+            })
+        })
+        .collect();
+    assert!(
+        mismatches.is_empty(),
+        "{} 个状态 GPU 与 CPU 的池级 checker 违例位不同（CPU 判 I-7.8 红的状态 {states_red_on_the_tree_identifier_watermark_on_the_cpu} 个；\
+         GPU 少判最新那条根自己的水位那一格时这些状态上少一位 I-7.8）：\n{}",
+        mismatches.len(),
+        mismatches.iter().take(8).cloned().collect::<Vec<_>>().join("\n")
+    );
+}
````
