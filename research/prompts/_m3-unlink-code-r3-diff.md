# 删除批第 12 批合入的代码：代码轮第三轮 diff（2026-10-03）

四份补丁原样（实现员 X、Y、Z 与合入后的改钉值），加变异表合入前后的差。补丁的底是各自派发那一刻的主仓 crates/；冻结副本 /tmp/claude-1000/m3-unlink-code-r3/tree/ 是四份都打上之后的样子。

## 1、实现员 X（丙1、V1–V5、Q-F、U8、续做结局守护）

`/tmp/claude-1000/m3-batch12/x/patch/crates.patch`

````diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 283636c..1847724 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -80,7 +80,7 @@ use crate::transaction::{
     TransactionOutput, TransactionUnit, VersionWithoutFilePublishOutput, ZeroUnitPublishPlan,
 };
 use crate::unlink::DeletionSwitches;
-use crate::write_accounting::WritesByStructureKind;
+use crate::write_accounting::{WriteCallsAndBytes, WritesByStructureKind, WrittenStructureKind};
 use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES, ROOT_RING_REGIONS, SLOT_BYTES};
 use std::borrow::Cow;
 use std::collections::{BTreeMap, BTreeSet};
@@ -291,6 +291,8 @@ pub enum MountError {
     /// 写入口、分配器与现行那一版随错丢掉，这次挂载已落盘的写账都在 [`FloorRaiseFailedAfterTheMountsPublishes`] 里
     /// （增补 2 收口表第 58 行：与 `Publish` 同一个形态，不交出来设备一层数到的写与程序交得出的账对不上）。
     /// 抬 F 那一串自己也取不到落点、被空间准入拒的不走这里（推满仍不够，挂载照样做成，`MountSpaceAdmission::StillShortAfterTheFloorRaises`）。
+    /// 挂载续做里为空间推的抬 F 自己报了不是空间不够的错，同样交这个成员（代码轮 V1）：那时写行、暖机、准入推的、续做发成的各步与续做推成的那几串
+    /// 都已落盘，写账口径见 [`FloorRaiseFailedAfterTheMountsPublishes`]。
     /// 装箱：不装箱 `MountError` 就大过 clippy `result_large_err` 的 128 字节。
     FloorRaiseFailedAfterTheMountsPublishes(Box<FloorRaiseFailedAfterTheMountsPublishes>),
     /// C554 乙（用户 2026-09-27 定：重读后再判，R = 1；岔路单 `research/prompts/c554-fix-forks.md` 第 1 行）：
@@ -502,16 +504,22 @@ pub enum SequenceNumberAtTheTopOfItsRange {
     CheckpointTxg(CheckpointTxg),
 }
 
-/// 可写挂载写行与暖机之后推抬 F、抬 F 自己报错时（`MountError::FloorRaiseFailedAfterTheMountsPublishes`，实审 A1b Q5）交得出的东西。
+/// 可写挂载推的抬 F 自己报错时（`MountError::FloorRaiseFailedAfterTheMountsPublishes`）交得出的东西。两处交它：写行与暖机之后为准入推的
+/// 那一串（实审 A1b Q5），与挂载续做里为空间推的那一串（代码轮 V1，`research/prompts/m3-unlink-code-r2-main-verification.md` 第三节）。
+/// 这次挂载交给设备的写（取号那两次系统配置槽写除外）= `writes_of_persisted_publishes` 各份 + `floor_raises` 各串先写系统配置那一步
+/// + 写行之后那一处另加 `floor_raises` 各串的各次空发布（续做那一处它们已在 `writes_of_persisted_publishes` 里，不另加）+ `cause` 里那一串自己的写账。
 #[derive(Debug)]
 pub struct FloorRaiseFailedAfterTheMountsPublishes {
     /// 抬 F 自己报的错，原样：它那一串自己的写账（先写系统配置那一步、已落盘的几次空发布、失败那一次已记的写）在它里面
     /// （`RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot`、`RaiseFloorSequencePublishFailed`）。
     pub cause: MountError,
-    /// 这次挂载在推抬 F 之前已经落盘的发布各自的写，按先后：写行那次在最前，之后暖机每次一份
-    /// （与 `PublishSequenceFailed::writes_of_persisted_publishes` 同一个口径；取号那两次系统配置槽写不是发布、不在这里）。
+    /// 这次挂载在推报错那一串之前已经落盘的发布各自的写，按先后（与 `PublishSequenceFailed::writes_of_persisted_publishes` 同一个口径；
+    /// 取号那两次系统配置槽写不是发布、不在这里）。写行之后那一处：写行那次在最前，之后暖机每次一份，推成的那几串的空发布不在这里
+    /// （在 `floor_raises` 里）。续做那一处：按 txg 先后接着写行、暖机、准入推的那几串的各次空发布、续做发成的各步与续做里推成的那几串的各次空发布——
+    /// 与续做那一格交 `Publish` 时的 `writes_of_persisted_publishes` 同一张表，崩溃注入照同一个划法从里面划出续做段。
     pub writes_of_persisted_publishes: Vec<WritesByStructureKind>,
-    /// 报错那一串之前已经推成的那几串抬 F，按先后（每一串先写系统配置那一步与各次空发布的写都在里面，都已落盘）。
+    /// 报错那一串之前已经推成的那几串抬 F，按先后（每一串先写系统配置那一步与各次空发布的写都在里面，都已落盘）。续做那一处是准入推的那几串
+    /// 接着续做里推成的那几串，它们的各次空发布同时在 `writes_of_persisted_publishes` 里。
     pub floor_raises: Vec<RaisedFloor>,
 }
 
@@ -729,8 +737,11 @@ pub enum SelectedVersionLackingOnDevice {
 #[derive(Debug)]
 pub struct PublishSequenceFailed {
     pub cause: PublishError,
-    /// 这一串第一次发布之前、不属于任何一次发布、这个写入口已经发出去的写：抬 F 那一串先把新 F 写进每块盘系统配置那一步
-    /// （每块盘一次系统配置槽写，D16（发布语义） 已定项 1「抬 F 那一串」）；可写挂载那一串是空账（取号那两次写不在这里，见上）。
+    /// 这一串里不属于任何一次发布、已经发出去的写：抬 F 那一串是第一次发布之前先把新 F 写进每块盘系统配置那一步
+    /// （每块盘一次系统配置槽写，D16（发布语义） 已定项 1「抬 F 那一串」）；可写挂载那一串在写行、暖机那几次失败时是空账（取号那两次写不在这里，见上），
+    /// 在挂载续做里失败时是这次挂载推成的每一串抬 F（准入推的与续做推的）先写系统配置那一步逐种相加——那几串的空发布在
+    /// `writes_of_persisted_publishes` 里，先写系统配置那一步不是发布，只能放在这里（代码轮 V2，`research/prompts/m3-unlink-code-r2-main-verification.md`
+    /// 第三节）。于是这一串交给设备的写 = 这一格 + 已落盘各份 + 失败账。
     pub writes_before_the_first_publish: WritesByStructureKind,
     /// 失败之前这一串里已经落盘的发布各自的写，按先后（可写挂载写行那次在最前）；第一次就失败时是空的。
     pub writes_of_persisted_publishes: Vec<WritesByStructureKind>,
@@ -917,7 +928,8 @@ pub struct MountOutput {
 }
 
 /// 挂载续做停在哪儿。成员按调用方要做的决定分：照常写；队列还留着——之后会话里每一次用户改动先走空、走不动报空间不够
-/// （`mounted_session::UserChangeRefused::NoSpaceAfterRaisingTheFloor`），正常卸载照常可走。
+/// （`mounted_session::UserChangeRefused::NoSpaceAfterRaisingTheFloor`），正常卸载照常做成：靠的是卸载那一串的预演算上新 F 回收出来的槽
+/// （[`unmount`]，用户 2026-10-03 定丙 1），之后再挂可写、续做接着走。
 #[derive(Debug)]
 pub enum DeletionQueueContinuationOutcome {
     /// 所选那一版的 `deleted_inodes` 是空的（常态），或续做把它走到了空。
@@ -946,7 +958,7 @@ pub enum MountSpaceAdmission {
     },
     /// 推满仍不够（C565（挂载处推满仍不够怎么收尾没定），实现员提的收尾、交主 agent 定）：挂载照样做成——实例已取、行已写、
     /// 暖机与推的那几串都已落盘；`last_refusal` 是最后一次判的结局，`stop` 是不再推的原因。之后的发布照发布路径的准入判，
-    /// 不够就报空间不够；正常卸载（抬 F 到现行那一版、不判上限）照常可走。
+    /// 不够就报空间不够；正常卸载（抬 F 到现行那一版、不判上限）照常做成，它的预演算上新 F 回收出来的槽（[`unmount`]，用户 2026-10-03 定丙 1）。
     StillShortAfterTheFloorRaises {
         refusal_before_acquisition: AdmissionRefusedOnSomeDevices,
         floor_raises: Vec<RaisedFloor>,
@@ -2226,6 +2238,18 @@ impl RaiseFloorEntry {
             RaiseFloorEntry::Unmount => UnmountMarker::WrittenByTheUnmountSequence,
         }
     }
+
+    /// 这一串回收出来的槽什么时候能发出去（用户 2026-10-03 定「丙 1 进下一批」，丙 1 的定义在
+    /// `research/prompts/m3-unmount-after-short-continuation-r1-opus-output.md`「丙1」那一节）：
+    /// 准入那一格扣到生效（每块盘上都有带新 F 的根）；卸载那一格不扣，整串预演与真发都能用——先写系统配置、过屏障那一步在第一条带新 F 的根之前，
+    /// 它之后恢复读得出新 F（D16（发布语义） 已定项 1「生效」那一行把系统配置里读得出的 F 算进 F_生效），回收的槽被这一串的单元写掉时，
+    /// 引用它们的那几条根都已低于 F_生效。挂载续做推满仍放不下之后卸载靠这一样做成（槽被续做吃光，扣着的话这一串在预演里就取不到落点）。
+    const fn reuse_of_the_reclaimed_slots(self) -> ReclaimedReuse {
+        match self {
+            RaiseFloorEntry::Admission => ReclaimedReuse::HeldUntilFloorTakesEffect,
+            RaiseFloorEntry::Unmount => ReclaimedReuse::Immediately,
+        }
+    }
 }
 
 /// 抬 F 那一串做完之后两个入口共有的东西；准入那一格另带它判过的上限。
@@ -2289,8 +2313,9 @@ pub fn raise_rollback_floor<Device: BlockDevice>(
 }
 
 /// 正常卸载（D16（发布语义） 已定项 1「正常卸载」，用户 2026-09-25 定 B1）：推一串空发布把 F 抬到现行那一版的 txg、不另判上限，
-/// 走的是准入抬 F 那一串（影子账按新 F 重算、回收扣到生效、整串预演、先把新 F 写进每块盘的系统配置过一道屏障、推到每块盘上都有一条带新 F 的根），
-/// 这一串写的根在 flags 里带卸载记号（D22（单元原子性怎么合成） 已定项 7）。第一版几何（R = 3、区域归属 0 / 1 / 0）下推 2 或 3 次：
+/// 走的是准入抬 F 那一串（影子账按新 F 重算、回收、整串预演、先把新 F 写进每块盘的系统配置过一道屏障、推到每块盘上都有一条带新 F 的根），
+/// 这一串写的根在 flags 里带卸载记号（D22（单元原子性怎么合成） 已定项 7）。与准入那一格差在回收出来的槽不扣到生效、预演与真发都能用
+/// （用户 2026-10-03 定丙 1）：挂载续做推满仍放不下、槽被吃光的盘上卸载照样做成，靠的就是这一串的预演算上新 F 回收出来的槽。第一版几何（R = 3、区域归属 0 / 1 / 0）下推 2 或 3 次：
 /// 现行 txg 除以 3 余 1 时 3 次，余 0 或余 2 时 2 次。不写干净关闭标记，下一次挂载照旧走恢复。做完之后这个写入口不再发布。
 /// 现行那一版树表 0 条时一个字节都不写，卸载照常做成（[`Unmounted::NothingWrittenOnAVersionWithoutFile`]）。
 ///
@@ -2671,7 +2696,8 @@ fn raise_the_floor_through<Device: BlockDevice>(
     // 回收在写第一条带新 F 的根之前：一条根带的 F 与它的记账行要说同一件事——checker 的 I-3.1（已分配统计对得上） 按那条根自己的 F
     // 取候选集的并集，释放代 ≤ F 的落点不在并集里，记账的「已分配」也不能再算它们。但回收的槽在生效（每块盘都有带新 F 的根）之前
     // 不许发出去：带新 F 的空发布自己就在分配固定点，开放段满了会开到刚回收空的那一段（alloc-basis 第二轮云端攻方腿打中），所以先扣住、
-    // 落满每块盘之后再放开。记账按写那条根的那一刻的 F 算还是按它持久之后的 F_生效 算，口径交 alloc-basis 那一轮（预想）。
+    // 落满每块盘之后再放开——这是准入那一格；卸载那一格不扣（用户 2026-10-03 定丙 1，`RaiseFloorEntry::reuse_of_the_reclaimed_slots`）。
+    // 记账按写那条根的那一刻的 F 算还是按它持久之后的 F_生效 算，口径交 alloc-basis 那一轮（预想）。
     // 回收门槛与影子账读的这一遍根环，照 D16（发布语义） 已定项 1「根槽这一次读坏」那一行分两类（与算上限那一遍同一个分法）：
     // 这个进程知道住着根的槽读坏就重读那一槽一次，仍坏就拒这一次抬（C554 乙报告 Q6：按「没有根」往下走，那一槽里的被抛弃根引用的槽
     // 不隔离、也不计数）；挂载那一刻就读坏、之后没写过的槽当没有根。在动分配器（影子账重算、回收）之前返回，分配器与盘都不动。
@@ -2724,9 +2750,10 @@ fn raise_the_floor_through<Device: BlockDevice>(
     } else {
         0
     };
+    // 卸载那一格回收的槽不扣（`RaiseFloorEntry::reuse_of_the_reclaimed_slots`，丙 1）：下面的整串预演就在分配器的拷贝上算上它们。
     let reclaimed = allocator.reclaim_released_up_to(
         reclaim_floor(new_floor, oldest_valid_root_in_the_window),
-        ReclaimedReuse::HeldUntilFloorTakesEffect,
+        entry.reuse_of_the_reclaimed_slots(),
     );
     let publishes_in_the_sequence = planned_txgs.len();
     let mut pool = PoolWriter::new(&parameters_of_the_pool, devices.as_mut_slice());
@@ -4445,6 +4472,7 @@ fn establish_instance<Device: BlockDevice>(
         instance,
         start.shadow_ledger,
         writes_of_the_publishes_before_the_continuation,
+        space_admission.floor_raises(),
     )?;
 
     Ok(Mounted {
@@ -4520,15 +4548,21 @@ impl DeletionQueueContinuation {
 /// 这次准入已推的数从 0 数起）。推满仍放不下（一次准入的发布数做满、F 已在上限、抬 F 那一串自己也取不到落点或被空间准入拒）时
 /// 挂载照样做成可写、队列留着（用户 2026-10-03 定「照 D16 做成可写」，D16（发布语义） 已定项 1「准入」那一行「挂载处推满仍不够时挂载照样做成可写」）：
 /// 交回 [`DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises`]，`current` 是最后落盘的那一版；之后会话里每一次用户改动先走空、
-/// 走不动报空间不够，正常卸载照常可走。
+/// 走不动报空间不够；正常卸载照常做成，靠的是卸载那一串的预演算上新 F 回收出来的槽（[`unmount`]，用户 2026-10-03 定丙 1）。
 /// 迭代上界：每一轮要么走空交回，要么推满仍放不下交回，要么交回错，要么推成一串（推的数加进这次准入，同一次发布的准入在预算 B − 1 次空发布之内停），
 /// 要么这一趟有一步落了盘（队首那个文件少了单元或删掉了，队列与单元数都有限）。跨轮携带的是走到的那一版、已推的数与已落盘的。
 ///
 /// # Errors
 /// 走空里的一步没发成、不是空间不够：错连同这次挂载已落盘各次发布的写（`writes_of_the_publishes_before_the_continuation` 接上续做里已落盘的
-/// 每一步与推的每一次空发布，按 txg 的先后）与这一趟走空的写入口的失败账（`MountError::Publish`，[`PublishSequenceFailed`] 的契约）。
-/// 抬 F 自己报了不是空间不够的错：那个错原样（[`push_one_floor_raise_within_the_admission_budget`] 的 `Err`）。
+/// 每一步与推的每一次空发布，按 txg 的先后）、这次挂载推成的每一串抬 F（`floor_raises_pushed_by_the_admission` 接上续做里推成的）
+/// 先写系统配置那一步（代码轮 V2）与这一趟走空的写入口的失败账（`MountError::Publish`，[`PublishSequenceFailed`] 的契约）。
+/// 抬 F 自己报了不是空间不够的错：那个错装进 `MountError::FloorRaiseFailedAfterTheMountsPublishes`（代码轮 V1），
+/// 连同同一张已落盘发布的写与这次挂载推成的每一串抬 F（准入推的接上续做推的）。
 /// 两种都是已经发成的那几步与推成的那几串留在盘上。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "挂载续做要的：参数、盘、分配器、现行那一版、实例、影子账那一臂、续做之前已落盘的写、准入推成的那几串，各自独立"
+)]
 fn continue_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
@@ -4537,6 +4571,7 @@ fn continue_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
     instance: InstanceGeneration,
     shadow_ledger: ShadowLedger,
     writes_of_the_publishes_before_the_continuation: Vec<WritesByStructureKind>,
+    floor_raises_pushed_by_the_admission: &[RaisedFloor],
 ) -> Result<DeletionQueueContinuation, MountError> {
     let mut continuation = DeletionQueueContinuation {
         steps_published: Vec::new(),
@@ -4592,7 +4627,11 @@ fn continue_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
                     .collect();
             return Err(MountError::Publish(Box::new(PublishSequenceFailed {
                 cause: refusal,
-                writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,
+                writes_before_the_first_publish: system_configuration_writes_of_the_floor_raises(
+                    floor_raises_pushed_by_the_admission
+                        .iter()
+                        .chain(&continuation.floor_raises),
+                ),
                 writes_of_persisted_publishes,
                 writes_of_failed_publishes,
             })));
@@ -4623,12 +4662,65 @@ fn continue_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
             }
             Err(cause) => {
                 *current = PoolVersion::of_a_version_of_trees(walked);
-                return Err(cause);
+                return Err(MountError::FloorRaiseFailedAfterTheMountsPublishes(
+                    Box::new(FloorRaiseFailedAfterTheMountsPublishes {
+                        cause,
+                        writes_of_persisted_publishes: [
+                            writes_of_the_publishes_before_the_continuation,
+                            continuation.writes_of_the_persisted_publishes_in_txg_order(),
+                        ]
+                        .concat(),
+                        floor_raises: floor_raises_pushed_by_the_admission
+                            .iter()
+                            .cloned()
+                            .chain(continuation.floor_raises)
+                            .collect(),
+                    }),
+                ));
             }
         }
     }
 }
 
+/// 几串抬 F 各自先写系统配置那一步的写（`RaisedFloor::system_configuration_writes_before_the_first_publish`）逐种相加：挂载续做那一格交
+/// `Publish` 时装进 `PublishSequenceFailed::writes_before_the_first_publish`（代码轮 V2）。`WritesByStructureKind` 每一种只记写调用数与字节数之和、
+/// 没有加法，这里按种类把两样各自加起来、重放成同样多次写调用（这一种的字节全记在第一次里，其余几次 0 字节）：交出去的每一种的调用数与字节数
+/// 与逐串相加相同；合计另核一次，`WrittenStructureKind::IN_REPORT_ORDER` 漏一种就对不上。
+fn system_configuration_writes_of_the_floor_raises<'raised>(
+    floor_raises: impl IntoIterator<Item = &'raised RaisedFloor>,
+) -> WritesByStructureKind {
+    let ledgers: Vec<&WritesByStructureKind> = floor_raises
+        .into_iter()
+        .map(|raised| &raised.system_configuration_writes_before_the_first_publish)
+        .collect();
+    let mut added_up = WritesByStructureKind::NOTHING_WRITTEN;
+    for kind in WrittenStructureKind::IN_REPORT_ORDER {
+        let of_this_kind = ledgers
+            .iter()
+            .fold(WriteCallsAndBytes::NONE, |sum, ledger| {
+                sum.plus(ledger.of(kind))
+            });
+        // 迭代上界是这一种加起来的写调用数；跨轮只多记这一种的一次调用。
+        for call_index in 0..of_this_kind.write_calls {
+            let bytes_of_this_call = if call_index == 0 {
+                usize::try_from(of_this_kind.written_bytes)
+                    .expect("几串抬 F 先写系统配置那一步的字节数之和装得进 usize")
+            } else {
+                0
+            };
+            added_up.count_write_call(kind, &vec![0_u8; bytes_of_this_call]);
+        }
+    }
+    assert_eq!(
+        added_up.total(),
+        ledgers
+            .iter()
+            .fold(WriteCallsAndBytes::NONE, |sum, ledger| sum.plus(ledger.total())),
+        "逐种相加的合计与逐串合计之和相同：IN_REPORT_ORDER 罩着每一种，每一种有字节就至少有一次写调用"
+    );
+    added_up
+}
+
 /// 可写挂载取号之前那一判的结局（`establish_instance` 用，交回之前换成 [`MountSpaceAdmission`]）。
 enum WritableMountAdmissionBeforeAcquisition {
     Admitted,
@@ -4655,7 +4747,7 @@ fn no_demand_on_any_device_of(allocator: &PoolAllocator) -> Vec<DemandOnDevice>
 /// （[`push_one_floor_raise_within_the_admission_budget`]，一次准入最多 [`PUBLISHES_PER_ADMISSION_AT_MOST`] 次发布，写行那次算一次）、
 /// 再判，直到够了或不再推。推满仍不够时怎么收尾条款没定（C565（挂载处推满仍不够怎么收尾没定））：这里取实现员提的那一种——
 /// 挂载照样交回做成（[`MountSpaceAdmission::StillShortAfterTheFloorRaises`]），实例已取、行已写，之后的发布照发布路径的准入判，
-/// 正常卸载照常可走（它抬 F 到现行那一版、不判上限）；交主 agent 定。
+/// 正常卸载照常做成（它抬 F 到现行那一版、不判上限，预演算上新 F 回收出来的槽，[`unmount`]，用户 2026-10-03 定丙 1）；交主 agent 定。
 ///
 /// 循环：每一轮先判，够了就交回；不够就推一串，推成了（至少一次空发布、`publishes_pushed` 增加）进下一轮，推满了就交回
 /// （抬 F 那一串自己也取不到落点、被空间准入拒，同样是推满了：[`FloorRaiseStop::FloorRaiseRefusedForSpace`]，实审 A1b Q1），
@@ -4751,7 +4843,8 @@ fn push_floor_raises_after_the_row_publish<Device: BlockDevice>(
 /// 盘上读来的实例代号或 txg 已到顶、下一个号装不下（`SequenceNumberPastTheTopOfItsRange`，取号之前）；
 /// 抬 F 那一串自己也取不到落点、被空间准入拒的，是推满仍不够，挂载照样做成（`MountSpaceAdmission::StillShortAfterTheFloorRaises`）。
 /// 挂载续做里推满仍放不下同样不报错：挂载照样做成、队列留着（`MountOutput::deletion_queue_continuation_outcome`）；续做里的一步没发成、
-/// 不是空间不够才报 `Publish`（[`continue_the_deletion_queue_until_it_is_empty`]）。
+/// 不是空间不够才报 `Publish`，续做里为空间推的抬 F 自己报了不是空间不够的错报 `FloorRaiseFailedAfterTheMountsPublishes`
+/// （[`continue_the_deletion_queue_until_it_is_empty`]）。
 pub fn mount_writable<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 3d99aa1..194e9bd 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -4149,20 +4149,52 @@ fn mapped_units_of_a_container_tree<Record>(
 /// （[`crate::admission::live_objects_of_the_inode_tree`]）。`TransactionOutput` 恒有树表条目（`deleted_inodes` 那一条在内），
 /// 所以走不到「没有 `deleted_inodes` 条目取 0」那一支——那一支是树表 0 条的一版，它不走这条发布路径、没有记账树。
 /// 记账行写它，发布路径判式子读它（「写入类发布按发布之后的标签判」）：一个函数算，两处读同一个数。
+/// H 照条款字面从墓碑树根节点的码 2 头现读（[`tombstone_tree_internal_levels_read_from_the_root_node_header`]，代码轮 U8）。
 #[must_use]
 pub fn committed_reservation_for_tombstones_after_this_publish(
     resolved: &ResolvedPublish,
+    previous: Option<&TransactionOutput>,
     replicas: ReplicaCount,
 ) -> BytesSummedOverAllReplicas {
     committed_reservation_for_tombstones(
         TombstonePrepayBasis::VersionWithADeletedInodesTreeTableEntry {
-            tombstone_tree_internal_levels: resolved.tombstone_tree.internal_levels(),
+            tombstone_tree_internal_levels:
+                tombstone_tree_internal_levels_read_from_the_root_node_header(resolved, previous),
             live_objects: live_objects_of_the_inode_tree(&resolved.inode_tree.containers),
         },
         replicas,
     )
 }
 
+/// 这次发布之后墓碑树的内部层数 H（D3（空间分配） 已定项 2「H 从墓碑树根节点码 2 头现读」，代码轮 U8，
+/// `research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：根节点头的层级减一（零根时 0）。
+/// 这次重写根（意图那次往开放容器追加墓碑）：新根的头由同一份形状编出（`build_index_node` 的层级取根在形状里的层级），按构造就是
+/// `resolved.tombstone_tree.internal_levels()`，不另读；这次照抄墓碑树（写入类发布、各批）：读上一版那份 `TombstoneRoot` 单元字节的码 2 头，
+/// 照 [`TransactionOutput::height_read_from_the_root_node_header`] 的先例，不读内存里那份形状。
+///
+/// # Panics
+/// 上一版有墓碑容器而它的单元里没有 `TombstoneRoot`，或那份字节解不开：单元是这个进程装出来的，或从盘上重建时读出、核过的
+/// （`recovery::rebuild_version` 把删除那两棵树的每个节点都放进 `units`）。
+#[must_use]
+pub fn tombstone_tree_internal_levels_read_from_the_root_node_header(
+    resolved: &ResolvedPublish,
+    previous: Option<&TransactionOutput>,
+) -> u64 {
+    if resolved.tombstone_tree.is_the_root_rewritten() {
+        return resolved.tombstone_tree.internal_levels();
+    }
+    match previous {
+        Some(previous_version) if !previous_version.tombstone_containers.is_empty() => {
+            let root_header =
+                parse_index_node(&previous_version.unit(TransactionUnit::TombstoneRoot).bytes)
+                    .expect("墓碑树根单元是这个进程装的，或重建时读出、核过的");
+            u64::from(root_header.level).saturating_sub(1)
+        }
+        // 上一版墓碑树零根（没有容器就没有根），这次又没重写它：这次之后仍是零根。
+        Some(_) | None => 0,
+    }
+}
+
 /// 一次发布重写哪些角色，只由三件事定：这次写不写文件内容（数据单元 + extent 树根）、重写几片 inode 叶容器、
 /// 实例表是重写还是照抄。计划的身份字段（新实例代号、表的字节、txg）都不进来——可写挂载要在取号之前算写行那次发布的准入，
 /// 而那时新实例代号还没取（增补 2 第 20a 行）。
@@ -5799,7 +5831,10 @@ pub fn publish_new_inodes<Device: BlockDevice>(
 
 /// unlink 的意图那次发布（D8（核心索引结构） 已定项 16 第 1 条）：`inode` 那条记录写成 nlink = 0、size 不变，`deleted_inodes` 加 key = `inode`，
 /// 墓碑树开放容器追加一条整对象墓碑（死亡 txg = 这次发布的 checkpoint_txg，D18（块里携带什么信息） 已定项 24 第 2 条）；不立 logged_ops。
-/// 数据单元与 extent 树照抄。一个事务（不写用户数据，不切），改动计数与写入时间不改（条款只写了 nlink 与 size）。
+/// 数据单元与 extent 树照抄。一个事务（不写用户数据，不切）。改动计数设成这次发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88
+/// 「最后一次改动所在发布的 checkpoint_txg」：用户 2026-10-03 定 Q-F「算改动，改实现」，意图那次与每一批都算一次改动）；
+/// 写入时间不改：记录里没有 ctime 字段，`write_time_seconds` 由调用方给，删除步没有确定的来源，改它会让同一份崩溃镜像各自续做出不同的盘
+/// （D8（核心索引结构） 已定项 16 第 7 条 ①b）。
 ///
 /// # Errors
 /// `inode` 不是树里一条活着的记录（没有、或已经 nlink = 0）⇒ `UnlinkOfAnInodeThatIsNotALiveRecord`；其余同 `publish_version`。
@@ -5826,6 +5861,8 @@ pub fn publish_unlink_intent<Device: BlockDevice>(
         .copied()
         .ok_or(PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { inode: inode.0 })?;
     let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
+    // 改动计数取成局部量、不写 `change_count: txg.0,`：`crates/mutations.tsv` 按那串字面锚在 `publish_overwrite` 上（同 `publish_new_inodes`）。
+    let change_count_of_the_intent = txg.0;
     let deletion_and_lineage = DeletionAndLineageOfThePublish {
         step: DeletionStepOfThePublish::Intent {
             record_with_nlink_zero: InodeRecord {
@@ -5833,7 +5870,7 @@ pub fn publish_unlink_intent<Device: BlockDevice>(
                 object_birth: record.object_birth,
                 nlink: NLINK_OF_AN_INODE_BEING_DELETED,
                 size: record.size,
-                change_count: record.change_count,
+                change_count: change_count_of_the_intent,
                 write_time_seconds: record.write_time_seconds,
             },
             tombstone: TombstoneRecord::of_the_whole_object(inode, record.object_birth, txg),
@@ -5876,6 +5913,8 @@ pub fn publish_unlink_step<Device: BlockDevice>(
         .filter(|_| previous.deleted_inodes.contains(&inode))
         .ok_or(PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { inode: inode.0 })?;
     let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
+    // 改动计数取成局部量，理由同 [`publish_unlink_intent`] 那一处。
+    let change_count_of_the_batch = txg.0;
     // 第一版只有第一个文件写数据：别的 inode 一个数据单元都没有，意图之后直接是最后一批。
     let data_units_remaining = if inode.0 == FIRST_INODE_NUMBER {
         u64::try_from(previous.data_pointers.len()).expect("单元数")
@@ -5892,7 +5931,8 @@ pub fn publish_unlink_step<Device: BlockDevice>(
                 object_birth: record.object_birth,
                 nlink: record.nlink,
                 size: size_after_the_batch_in_bytes,
-                change_count: record.change_count,
+                // 每一批算一次改动（用户 2026-10-03 定 Q-F，同意图那次）；写入时间照旧，理由同 [`publish_unlink_intent`]。
+                change_count: change_count_of_the_batch,
                 write_time_seconds: record.write_time_seconds,
             },
             data_units_kept,
@@ -6346,6 +6386,7 @@ pub fn prepare_the_version_publish<Device: BlockDevice>(
                 previous,
                 committed_reservation_for_tombstones_after_this_publish(
                     &settled.resolved,
+                    previous,
                     ReplicaCount::of_every_device_in_the_pool(allocator),
                 ),
                 pool.parameters.geometry.root_ring_slots_per_region,
@@ -7878,6 +7919,7 @@ fn publish_admitted<Device: BlockDevice>(
             PoolWideAccountingRow::CommittedReservationBytes => {
                 committed_reservation_for_tombstones_after_this_publish(
                     resolved,
+                    previous,
                     ReplicaCount::of_every_device_in_the_pool(allocator),
                 )
                 .0
diff --git a/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs b/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs
index c80d48b..c2af4bc 100644
--- a/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs
+++ b/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs
@@ -35,7 +35,8 @@ use singlefs_core::mount::{
 use singlefs_core::mounted_read::{mount_read_only, OpenFileFailure};
 use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
 use singlefs_core::recovery::{
-    choose_system_configuration, instance_table_chain_of_root, BadRootRingSlotReading,
+    choose_system_configuration, instance_table_chain_of_root, recover, BadRootRingSlotReading,
+    JournalPolicy,
 };
 use singlefs_core::root_record::RootRecord;
 use singlefs_core::root_ring::{slot_offset, RootRingSlot};
@@ -48,6 +49,9 @@ use singlefs_core::unit::data_unit_payload_capacity;
 use singlefs_core::unlink::{
     DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
 };
+use singlefs_core::write_accounting::{
+    WriteCallsAndBytes, WritesByStructureKind, WrittenStructureKind,
+};
 use singlefs_harness::fault_injection::{
     FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
     FaultSchedule, InjectedFault, SharedFaultPlan,
@@ -58,10 +62,10 @@ use singlefs_harness::history::{
     HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed,
     StepOutcome, StepPosition,
 };
-use singlefs_harness::memory_pool::{SparseBlockDevice, SparseDevice};
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
 use singlefs_harness::model::{ModelRefusalReason, ObservedRefusalReason};
 use singlefs_harness::model_comparison::refusal_reason_of_mount_error;
-use singlefs_harness::{RecordingBlockDevice, SharedStream};
+use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};
 
 /// 会话里 unlink 之前覆盖写几次：换下的单元进 defer，等推的抬 F 回收（意图那次被拒之后靠它们腾出槽）。
 const OVERWRITES_BEFORE_THE_UNLINK: usize = 10;
@@ -291,21 +295,41 @@ fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
 /// （批大小 1，不经会话、不走空），然后进程退出（会话丢掉）。交回池与退出那一刻现行那一版的根。
 fn pool_crashed_between_deletion_batches_on_narrow_devices(
     batches: usize,
+) -> (PoolUnderTest<SparseBlockDevice>, RootRecord) {
+    pool_crashed_between_deletion_batches(NarrowPoolCrashedBetweenDeletionBatches {
+        unit_area_slots: UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES_FOR_THE_CONTINUATION,
+        overwrites_before_the_unlink: OVERWRITES_BEFORE_THE_CONTINUATION,
+        data_units_of_the_file: DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION,
+        batches_published_after_the_intent: batches,
+    })
+}
+
+/// 不补环窗口、准入关着造的窄盘上崩在两批删除之间的那一刻（草稿探针的网格就按这四样排）：单元区几槽、unlink 之前先覆盖写几次、
+/// 第一个文件顺序写长到几个数据单元、意图之后又发了几批（批大小 1）。
+#[derive(Clone, Copy, Debug)]
+struct NarrowPoolCrashedBetweenDeletionBatches {
+    unit_area_slots: u64,
+    overwrites_before_the_unlink: usize,
+    data_units_of_the_file: usize,
+    batches_published_after_the_intent: usize,
+}
+
+/// 照 `shape` 造盘：准入关着的窄盘上第一个文件之后覆盖写、顺序写长大，在这个进程里直接发 unlink 的意图那次与之后那几批
+/// （批大小 1，不经会话、不走空），然后进程退出（会话丢掉）。交回池与退出那一刻现行那一版的根。
+fn pool_crashed_between_deletion_batches(
+    shape: NarrowPoolCrashedBetweenDeletionBatches,
 ) -> (PoolUnderTest<SparseBlockDevice>, RootRecord) {
     let mut pool =
         start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
             HistoryDeviceWidth::UnitAreaOf384Slots,
-            UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES_FOR_THE_CONTINUATION,
+            shape.unit_area_slots,
         );
-    for overwrite_index in 1..=OVERWRITES_BEFORE_THE_CONTINUATION {
+    for overwrite_index in 1..=shape.overwrites_before_the_unlink {
         pool.overwrite(OVERWRITE_BYTES)
             .unwrap_or_else(|refusal| panic!("第 {overwrite_index} 次覆盖写做成：{refusal:?}"));
     }
-    pool.sequential_write(
-        (DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION - 1) * data_unit_payload_capacity()
-            + 100,
-    )
-    .unwrap_or_else(|refusal| panic!("顺序写把文件长到几个单元：{refusal:?}"));
+    pool.sequential_write((shape.data_units_of_the_file - 1) * data_unit_payload_capacity() + 100)
+        .unwrap_or_else(|refusal| panic!("顺序写把文件长到几个单元：{refusal:?}"));
     let PoolUnderTest {
         session,
         devices,
@@ -327,7 +351,7 @@ fn pool_crashed_between_deletion_batches_on_narrow_devices(
         session.instance,
     )
     .expect("意图那次");
-    for batch_index in 0..batches {
+    for batch_index in 0..shape.batches_published_after_the_intent {
         current = publish_unlink_step(
             &mut writer,
             &mut session.allocator,
@@ -432,12 +456,14 @@ fn a_mount_whose_deletion_continuation_finds_no_unblocked_slot_raises_the_floor_
 /// 推满仍放不下（用户 2026-10-03 定「照 D16 做成可写」：挂载照样做成可写、队列留着）：同一份窄盘造法，发完意图与第一批才崩，重开、准入判着、
 /// 删除批大小 1。续做里一批被落得下那一判拒，为它推的那一串推不出空间（一串都没推成，F 没动）：挂载照样交回可写，
 /// 续做停在 `StillShortAfterTheFloorRaises`、最后那一次被拒是落得下那一判，现行那一版的队列还留着第一个文件；
-/// 挂着时一次用户改动先走空、走不动报空间不够（一版都没发成）；正常卸载照常走抬 F 到现行那一版那一串，这块盘上它在预演里就取不到落点、
-/// 在任何写之前拒。挂载写下的 13 次发布（写行、暖机与续做发成的几步）留在盘上：只读挂载走的是挂载最后落盘的那一版，
-/// 第一个文件仍在删除中（打开它答「没有文件」）。
-/// 判别力：推满仍放不下照旧交回 `MountError::Publish`（这一条定案之前的做法）时挂载那一步就红。
+/// 挂着时一次用户改动先走空、走不动报空间不够（一版都没发成、盘不动）。挂载写下的 13 次发布（写行、暖机与续做发成的几步）留在盘上：
+/// 只读挂载走的是挂载最后落盘的那一版，第一个文件仍在删除中（打开它答「没有文件」）。
+/// 正常卸载照常做成（用户 2026-10-03 定丙 1，`m3-unmount-after-short-continuation-r1`）：槽被续做吃光了，卸载那一串的预演算上新 F 回收出来的槽，
+/// 抬 F 到现行那一版、两次空发布；之后池级 checker 全绿，再挂可写、续做接着走到空（两步）。
+/// 判别力：推满仍放不下照旧交回 `MountError::Publish`（这一条定案之前的做法）、或结局报成 `WalkedToEmpty`，挂载那一步就红；
+/// 卸载回收的槽照旧扣到生效（`ReclaimedReuse::HeldUntilFloorTakesEffect`）时卸载在预演里取不到落点，卸载那一步红。
 #[test]
-fn a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_with_the_queue_left_and_its_publishes_on_disk(
+fn a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue(
 ) {
     let (mut pool, root_at_the_crash) = pool_crashed_between_deletion_batches_on_narrow_devices(1);
     let mounted = mount_after_the_crash_with_batches_of_one(&mut pool)
@@ -490,28 +516,11 @@ fn a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_ma
         ("SpaceAdmissionRefused", 0, 0, true),
         "走空的那一批照样被落得下那一判拒、为它推的那一串自己也落不下：一版都没发成，报空间不够"
     );
-    // 正常卸载照常走它那一条路（抬 F 到现行那一版的一串空发布，D16（发布语义） 已定项 1「正常卸载」）：这块窄盘的槽被续做吃光了，
-    // 那一串在任何写之前的预演里就取不到落点（攻方 E8b 量到的那一形；派发的验收写「卸载成」，这块盘上做不到，交主 agent）。
-    let unmounted = unmount(
-        &pool.parameters,
-        &mut pool.devices,
-        &mut session.allocator,
-        &mut session.current,
-        session.shadow_ledger,
-    );
-    assert!(
-        matches!(
-            unmounted,
-            Err(MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. })
-        ),
-        "卸载那一串在预演里就取不到落点"
-    );
     assert!(
         pool.image() == image_after_the_mount,
-        "那一次用户改动与卸载都在任何写之前被拒：两块盘逐字节停在挂载之后"
+        "那一次用户改动在任何写之前被拒：两块盘逐字节停在挂载之后"
     );
-    let image = pool.image();
-    let read_only = mount_read_only(&image).expect("只读挂载");
+    let read_only = mount_read_only(&image_after_the_mount).expect("只读挂载");
     assert_eq!(
         (
             read_only.effective_root.checkpoint_txg.0 - root_at_the_crash.checkpoint_txg.0,
@@ -524,11 +533,72 @@ fn a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_ma
         matches!(
             read_only
                 .mounted
-                .open_file(&image, InodeNumber(FIRST_INODE_NUMBER)),
+                .open_file(&image_after_the_mount, InodeNumber(FIRST_INODE_NUMBER)),
             Err(OpenFileFailure::NoSuchInode { .. })
         ),
         "第一个文件仍在删除中：打开它答「没有文件」"
     );
+    // 正常卸载照常做成（D16（发布语义） 已定项 1「正常卸载」，丙 1）：这块窄盘的槽被续做吃光了，卸载那一串回收出来的槽不扣到生效，
+    // 预演与真发都落在它们上面。
+    let current_txg = session.current.root().checkpoint_txg;
+    let unmounted = unmount(
+        &pool.parameters,
+        &mut pool.devices,
+        &mut session.allocator,
+        &mut session.current,
+        session.shadow_ledger,
+    );
+    let Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) = &unmounted else {
+        panic!(
+            "卸载那一串的预演算上新 F 回收出来的槽，做成：{:?}",
+            unmounted.as_ref().err()
+        );
+    };
+    assert_eq!(
+        (raised.rollback_floor, raised.publishes.len()),
+        (current_txg, 2),
+        "F 抬到现行那一版的 txg，两次带卸载记号的空发布"
+    );
+    drop(session);
+    let image_after_the_unmount = pool.image();
+    assert_eq!(
+        checker_violations_on(&image_after_the_unmount),
+        Vec::<String>::new(),
+        "卸载之后池级 checker 全绿"
+    );
+    let remounted = mount_after_the_crash_with_batches_of_one(&mut pool)
+        .unwrap_or_else(|error| panic!("卸载之后再挂可写：{error:?}"));
+    assert!(
+        matches!(
+            remounted.output.deletion_queue_continuation_outcome,
+            DeletionQueueContinuationOutcome::WalkedToEmpty
+        ),
+        "再挂之后续做接着走：{:?}",
+        remounted.output.deletion_queue_continuation_outcome
+    );
+    assert_eq!(
+        (
+            remounted.output.deletion_queue_continuation.len(),
+            member_name_of(&remounted.current)
+        ),
+        (2, "WithTreesAndNoFile"),
+        "再挂之后续做走了两步、走到有树而文件集合为空的一版"
+    );
+    drop(remounted);
+    assert_eq!(
+        checker_violations_on(&pool.image()),
+        Vec::<String>::new(),
+        "再挂续做之后池级 checker 全绿"
+    );
+}
+
+/// 现行那一版是 `PoolVersion` 的哪一个成员。
+fn member_name_of(version: &PoolVersion) -> &'static str {
+    match version {
+        PoolVersion::WithoutFile(_) => "WithoutFile",
+        PoolVersion::WithFile(_) => "WithFile",
+        PoolVersion::WithTreesAndNoFile(_) => "WithTreesAndNoFile",
+    }
 }
 
 /// 推满仍放不下、挂载照样做成可写之后进程退出，同一份窄盘再可写挂载（代码轮 U5，攻方 `m3-unlink-code-r1` A5 / E8c 那一形）：
@@ -605,6 +675,860 @@ fn after_a_mount_whose_continuation_stays_short_the_next_mount_is_refused_by_the
     assert!(image_after == image_before, "两块盘逐字节不变");
 }
 
+/// 续做走到空、F 一次都没抬、可删的单元都还在 defer 里的盘面上卸载（攻方 `m3-unmount-after-short-continuation-r1` 表 A 那 9 格的形：
+/// 卸载被拒不绑队列）：单元区 256 槽、不先覆盖写、14 个单元，崩在意图与两批之后。重开、准入判着、删除批大小 1：取号之前那一判放行、
+/// 没推抬 F，续做 12 步走到空。挂载之后每块盘只剩 9 个没挡的空槽，卸载那一串三次空发布要 18 个——扣着回收出来的槽就在预演里取不到落点；
+/// 丙 1 之下预演算上新 F 回收出来的槽，卸载做成、F 抬到现行那一版，池级 checker 全绿，再挂可写做成。
+/// 判别力：卸载回收的槽照旧扣到生效时卸载在预演里被拒，卸载那一步红。
+#[test]
+fn a_mount_whose_continuation_walks_to_empty_without_raising_the_floor_is_unmounted_on_the_slots_its_unmount_sequence_reclaims(
+) {
+    let (mut pool, root_at_the_crash) =
+        pool_crashed_between_deletion_batches(NarrowPoolCrashedBetweenDeletionBatches {
+            unit_area_slots: UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES_FOR_THE_CONTINUATION,
+            overwrites_before_the_unlink: 0,
+            data_units_of_the_file: DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION,
+            batches_published_after_the_intent: 2,
+        });
+    let mounted = mount_after_the_crash_with_batches_of_one(&mut pool)
+        .unwrap_or_else(|error| panic!("续做走到空：{error:?}"));
+    assert!(
+        matches!(
+            mounted.output.deletion_queue_continuation_outcome,
+            DeletionQueueContinuationOutcome::WalkedToEmpty
+        ),
+        "续做走到空：{:?}",
+        mounted.output.deletion_queue_continuation_outcome
+    );
+    assert_eq!(
+        (
+            mounted.output.space_admission.floor_raises().len(),
+            mounted.output.floor_raises_pushed_by_the_continuation.len(),
+            mounted.output.deletion_queue_continuation.len(),
+            mounted.current.root().rollback_floor,
+        ),
+        (0, 0, 12, root_at_the_crash.rollback_floor),
+        "准入与续做都没推抬 F、F 没动，续做 12 步走到空"
+    );
+    let free_slots_before_the_unmount: Vec<u64> = mounted
+        .allocator
+        .devices
+        .iter()
+        .map(|device_map| device_map.free_slots())
+        .collect();
+    let (_, mut session) = MountedSession::of_the_mount(mounted);
+    let current_txg = session.current.root().checkpoint_txg;
+    let unmounted = unmount(
+        &pool.parameters,
+        &mut pool.devices,
+        &mut session.allocator,
+        &mut session.current,
+        session.shadow_ledger,
+    );
+    let Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) = &unmounted else {
+        panic!(
+            "卸载那一串的预演算上新 F 回收出来的槽，做成：{:?}",
+            unmounted.as_ref().err()
+        );
+    };
+    let slots_taken_by_the_unmount_sequence: Vec<SlotNumber> = raised
+        .publishes
+        .iter()
+        .flat_map(|publish| {
+            publish
+                .units
+                .iter()
+                .filter(|unit| publish.rewritten.contains(&unit.identity))
+                .map(|unit| unit.slot)
+        })
+        .collect();
+    let reclaimed_slots: Vec<SlotNumber> = raised
+        .reclaimed
+        .iter()
+        .map(|placement| placement.slot)
+        .collect();
+    assert_eq!(
+        (
+            free_slots_before_the_unmount,
+            slots_taken_by_the_unmount_sequence.len(),
+            raised.rollback_floor,
+        ),
+        (vec![9, 9], 18, current_txg),
+        "卸载之前每块盘 9 个空槽，那一串要 18 个；F 抬到现行那一版"
+    );
+    assert!(
+        slots_taken_by_the_unmount_sequence
+            .iter()
+            .any(|slot| reclaimed_slots.contains(slot)),
+        "那一串有单元落在它自己回收出来的槽上：{slots_taken_by_the_unmount_sequence:?}"
+    );
+    drop(session);
+    assert_eq!(
+        checker_violations_on(&pool.image()),
+        Vec::<String>::new(),
+        "卸载之后池级 checker 全绿"
+    );
+    mount_after_the_crash_with_batches_of_one(&mut pool)
+        .unwrap_or_else(|error| panic!("卸载之后再挂可写：{error:?}"));
+}
+
+/// 一块稀疏盘外面先包录制器（录下真落到盘上的每一步），再包可注错的那一层。
+type FaultInjectingRecordedDevice =
+    FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>;
+
+/// 一份镜像交给两块新的 [`FaultInjectingRecordedDevice`]（`schedule` 为 `None` 时只数调用、不注错），连同它们共用的那条录制流。
+fn fault_injecting_recorded_devices_on(
+    image: &MemoryPool,
+    schedule: Option<FaultSchedule>,
+) -> (
+    Vec<(DeviceIdentity, FaultInjectingRecordedDevice)>,
+    SharedStream,
+) {
+    let geometry = HistoryDeviceWidth::UnitAreaOf384Slots.fixed_geometry();
+    let plan = match schedule {
+        Some(schedule) => SharedFaultPlan::armed(geometry, schedule),
+        None => SharedFaultPlan::unarmed(geometry),
+    };
+    let stream = SharedStream::new();
+    let devices = plain_devices_on(image)
+        .into_iter()
+        .map(|(identity, device)| {
+            (
+                identity,
+                FaultInjectingBlockDevice::new(
+                    identity,
+                    RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
+                    plan.clone(),
+                ),
+            )
+        })
+        .collect();
+    (devices, stream)
+}
+
+/// 这几块录制着、可注错的盘此刻的镜像。
+fn image_of_the_fault_injecting_recorded_devices(
+    devices: &[(DeviceIdentity, FaultInjectingRecordedDevice)],
+    device_size_in_bytes: u64,
+) -> MemoryPool {
+    MemoryPool {
+        devices: devices
+            .iter()
+            .map(|(identity, device)| {
+                (
+                    *identity,
+                    device.wrapped_device().wrapped_device().image.clone(),
+                )
+            })
+            .collect(),
+        device_size_in_bytes,
+    }
+}
+
+/// 整池第 `ordinal` 次写报块设备错（从 1 数）；`ThisWriteAndEveryLaterOne` 时之后每一次写都报错（掉电的那一形：盘停在这次写之前）。
+#[derive(Clone, Copy, Debug)]
+enum FailingWrites {
+    OnlyThisWrite,
+    ThisWriteAndEveryLaterOne,
+}
+
+fn write_failing_from(ordinal: u64, failing_writes: FailingWrites) -> FaultSchedule {
+    FaultSchedule {
+        fault: InjectedFault::WriteFails,
+        device: FaultDeviceSelector::EveryDevice,
+        placement: FaultPlacement::AnyOffset,
+        counting: FaultCounting::AcrossThePool,
+        occurrence: match failing_writes {
+            FailingWrites::OnlyThisWrite => FaultOccurrence::TheNthMatchingCall(ordinal),
+            FailingWrites::ThisWriteAndEveryLaterOne => {
+                FaultOccurrence::EveryMatchingCallFromTheNthOnward(ordinal)
+            }
+        },
+    }
+}
+
+/// 录制流里真落到盘上的写（屏障不算）与其中的根槽写（根槽是唯一走 FUA 的写，`transaction::CommitStep::WriteRootRecordForceUnitAccess`）。
+fn landed_writes_and_root_slot_writes(stream: &SharedStream) -> (u64, u64) {
+    let operations = stream.operations();
+    let count_of = |kinds: &[RecordedOperationKind]| {
+        u64::try_from(
+            operations
+                .iter()
+                .filter(|operation| kinds.contains(&operation.kind))
+                .count(),
+        )
+        .expect("步数装得进 u64")
+    };
+    (
+        count_of(&[
+            RecordedOperationKind::Write,
+            RecordedOperationKind::WriteForceUnitAccess,
+            RecordedOperationKind::WriteZeroes,
+        ]),
+        count_of(&[RecordedOperationKind::WriteForceUnitAccess]),
+    )
+}
+
+/// 几份写账的写调用数之和。
+fn write_calls_of<'ledger>(
+    ledgers: impl IntoIterator<Item = &'ledger WritesByStructureKind>,
+) -> u64 {
+    ledgers
+        .into_iter()
+        .fold(WriteCallsAndBytes::NONE, |sum, ledger| {
+            sum.plus(ledger.total())
+        })
+        .write_calls
+}
+
+/// 取号那两次系统配置槽写（每块盘一次）：不是发布，挂载交回的错与输出里的写账都不含它们（`PublishSequenceFailed` 的契约）。
+const ACQUISITION_WRITES: u64 = 2;
+
+/// 一次不注错的可写挂载里，txg 低于 `txg` 的那几次发布（写行、暖机、准入推的与续做推的空发布、续做的各步）与第一次发布低于它的
+/// 那几串抬 F 先写系统配置那一步，一共交给设备几个写（连取号那两次）；交回 (写数, 发布数)。注错点就按它摆在 txg 那一次发布的第一个写上。
+fn writes_and_publishes_of_the_mount_before(
+    output: &singlefs_core::mount::MountOutput,
+    txg: CheckpointTxg,
+) -> (u64, usize) {
+    let publishes: Vec<(CheckpointTxg, &WritesByStructureKind)> =
+        std::iter::once(&output.row_publish)
+            .chain(&output.warm_up_publishes)
+            .map(|version| (version.root().checkpoint_txg, version.writes()))
+            .chain(
+                output
+                    .space_admission
+                    .floor_raises()
+                    .iter()
+                    .chain(&output.floor_raises_pushed_by_the_continuation)
+                    .flat_map(|raised| raised.publishes.iter())
+                    .chain(&output.deletion_queue_continuation)
+                    .map(|publish| (publish.root.checkpoint_txg, &publish.writes)),
+            )
+            .filter(|(publish_txg, _)| *publish_txg < txg)
+            .collect();
+    let system_configuration_writes_of_the_raises = write_calls_of(
+        output
+            .space_admission
+            .floor_raises()
+            .iter()
+            .chain(&output.floor_raises_pushed_by_the_continuation)
+            .filter(|raised| {
+                raised
+                    .publishes
+                    .first()
+                    .is_some_and(|first| first.root.checkpoint_txg < txg)
+            })
+            .map(|raised| &raised.system_configuration_writes_before_the_first_publish),
+    );
+    (
+        ACQUISITION_WRITES
+            + write_calls_of(publishes.iter().map(|(_, writes)| *writes))
+            + system_configuration_writes_of_the_raises,
+        publishes.len(),
+    )
+}
+
+/// 挂载续做推过抬 F 的三种起点（代码轮 V2、V3，攻方 `m3-unlink-code-r2` G2、G3 那几格）：同一份崩溃镜像先在一份拷贝上不注错地挂一次，
+/// 数清各次发布的写，再在另一份上让 txg 为 `failing_publish` 的那一次续做发布的第一个写报块设备错。交回不注错那一次的挂载输出、
+/// 注错那一次的错与设备一层真落下的写与根槽写。
+fn mount_failing_on_the_first_write_of_a_continuation_step(
+    shape: NarrowPoolCrashedBetweenDeletionBatches,
+    choose_the_failing_step: impl Fn(&singlefs_core::mount::MountOutput) -> CheckpointTxg,
+) -> (
+    singlefs_core::mount::MountOutput,
+    CheckpointTxg,
+    MountError,
+    (u64, u64),
+) {
+    let (pool, _) = pool_crashed_between_deletion_batches(shape);
+    let image = pool.image();
+    let mut counting_devices = plain_devices_on(&image);
+    let counted = mount_writable_with_test_only_switches_and_deletion_switches(
+        &pool.parameters,
+        &mut counting_devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        deletion_switches_with_batches_of_one(),
+    )
+    .unwrap_or_else(|error| panic!("拷贝上不注错的那一次挂载做成：{error:?}"));
+    let failing_step = choose_the_failing_step(&counted.output);
+    let (writes_before_the_failing_step, _) =
+        writes_and_publishes_of_the_mount_before(&counted.output, failing_step);
+    let (mut failing_devices, stream) = fault_injecting_recorded_devices_on(
+        &image,
+        Some(write_failing_from(
+            writes_before_the_failing_step + 1,
+            FailingWrites::OnlyThisWrite,
+        )),
+    );
+    let refusal = mount_writable_with_test_only_switches_and_deletion_switches(
+        &pool.parameters,
+        &mut failing_devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        deletion_switches_with_batches_of_one(),
+    )
+    .expect_err("续做那一步的第一个写报块设备错");
+    (
+        counted.output,
+        failing_step,
+        refusal,
+        landed_writes_and_root_slot_writes(&stream),
+    )
+}
+
+/// 续做那一格交回的 `Publish` 错：已落盘份数 = 设备一层落下的根槽写数，写账合计（这一串里不属于任何发布的写 + 已落盘各份 + 失败账）
+/// = 落下的写 − 取号两次（代码轮 V2 / V3 的判法）。
+fn assert_the_continuation_publish_error_accounts_for_every_landed_write(
+    refusal: &MountError,
+    landed_writes: u64,
+    landed_root_slot_writes: u64,
+    publishes_before_the_failing_step: usize,
+) {
+    let MountError::Publish(failed) = refusal else {
+        panic!("续做那一步的块设备错交回 MountError::Publish：{refusal:?}");
+    };
+    assert!(
+        matches!(failed.cause, PublishError::BlockDevice(_)),
+        "错是那一次写的块设备错：{:?}",
+        failed.cause
+    );
+    assert_eq!(
+        (
+            failed.writes_of_persisted_publishes.len(),
+            u64::try_from(failed.writes_of_persisted_publishes.len()).expect("份数"),
+        ),
+        (publishes_before_the_failing_step, landed_root_slot_writes),
+        "已落盘的发布一次一份，份数等于设备一层落下的根槽写数（推的抬 F 的空发布也在里面）"
+    );
+    assert_eq!(
+        write_calls_of(
+            std::iter::once(&failed.writes_before_the_first_publish)
+                .chain(&failed.writes_of_persisted_publishes)
+                .chain(&failed.writes_of_failed_publishes)
+        ),
+        landed_writes - ACQUISITION_WRITES,
+        "写账合计等于落下的写减去取号那两次（推的每一串先写系统配置那一步在 writes_before_the_first_publish 里）"
+    );
+}
+
+/// 准入推过抬 F 的起点（代码轮 V3 第一形）：单元区 256 槽、覆盖写 2 次、14 个单元、崩在第 7 批之后；取号之前不够、写行与暖机之后推了一串
+/// （三次空发布），续做 7 步、没再推。续做第二步的第一个写报块设备错：错里 6 份（写行、暖机、推的三次、续做第一步）= 落下的 6 个根槽写，
+/// 写账合计 = 落下的写 − 取号两次（推的那一串先写系统配置那 2 个写在内）。
+/// 判别力：续做之前的写账去掉准入推的那几次空发布（攻方 MU1）时份数少 3；先写系统配置那一步不进错（改之前）时合计少 2。
+#[test]
+fn a_continuation_step_failing_after_the_admission_raised_the_floor_hands_back_every_landed_write()
+{
+    let (output, failing_step, refusal, (landed_writes, landed_root_slot_writes)) =
+        mount_failing_on_the_first_write_of_a_continuation_step(
+            NarrowPoolCrashedBetweenDeletionBatches {
+                unit_area_slots: UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES_FOR_THE_CONTINUATION,
+                overwrites_before_the_unlink: OVERWRITES_BEFORE_THE_CONTINUATION,
+                data_units_of_the_file: DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION,
+                batches_published_after_the_intent: 7,
+            },
+            |output| output.deletion_queue_continuation[1].root.checkpoint_txg,
+        );
+    assert_eq!(
+        (
+            output
+                .space_admission
+                .floor_raises()
+                .iter()
+                .map(|raised| raised.publishes.len())
+                .collect::<Vec<_>>(),
+            output.floor_raises_pushed_by_the_continuation.len(),
+            output.deletion_queue_continuation.len(),
+        ),
+        (vec![3], 0, 7),
+        "起点：准入推了一串三次空发布，续做 7 步、没推"
+    );
+    let (_, publishes_before_the_failing_step) =
+        writes_and_publishes_of_the_mount_before(&output, failing_step);
+    assert_eq!(
+        publishes_before_the_failing_step, 6,
+        "写行、暖机、推的三次、续做第一步"
+    );
+    assert_the_continuation_publish_error_accounts_for_every_landed_write(
+        &refusal,
+        landed_writes,
+        landed_root_slot_writes,
+        publishes_before_the_failing_step,
+    );
+}
+
+/// 续做里推过抬 F 的起点（代码轮 V3 第二形）：同一种盘崩在第 0 批之后（只发了意图）；续做 14 步，第 11 步之后被落得下那一判拒、推一串两次空发布、
+/// 回收之后接着走。推的那一串之后的第一步的第一个写报块设备错：错里 16 份（写行、暖机两次、续做 11 步、推的两次）= 落下的 16 个根槽写，
+/// 写账合计 = 落下的写 − 取号两次。
+/// 判别力：续做的写账去掉续做推的那几次空发布（攻方 MU2）时份数少 2；先写系统配置那一步不进错时合计少 2。
+#[test]
+fn a_continuation_step_failing_after_the_continuation_raised_the_floor_hands_back_every_landed_write(
+) {
+    let (output, failing_step, refusal, (landed_writes, landed_root_slot_writes)) =
+        mount_failing_on_the_first_write_of_a_continuation_step(
+            NarrowPoolCrashedBetweenDeletionBatches {
+                unit_area_slots: UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES_FOR_THE_CONTINUATION,
+                overwrites_before_the_unlink: OVERWRITES_BEFORE_THE_CONTINUATION,
+                data_units_of_the_file: DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION,
+                batches_published_after_the_intent: 0,
+            },
+            first_continuation_step_after_the_continuation_raised_the_floor,
+        );
+    assert_eq!(
+        (
+            output.space_admission.floor_raises().len(),
+            output
+                .floor_raises_pushed_by_the_continuation
+                .iter()
+                .map(|raised| raised.publishes.len())
+                .collect::<Vec<_>>(),
+            output.deletion_queue_continuation.len(),
+        ),
+        (0, vec![2], 14),
+        "起点：准入没推，续做 14 步、中间推了一串两次空发布"
+    );
+    let (_, publishes_before_the_failing_step) =
+        writes_and_publishes_of_the_mount_before(&output, failing_step);
+    assert_eq!(
+        publishes_before_the_failing_step, 16,
+        "写行、暖机两次、续做 11 步、推的两次"
+    );
+    assert_the_continuation_publish_error_accounts_for_every_landed_write(
+        &refusal,
+        landed_writes,
+        landed_root_slot_writes,
+        publishes_before_the_failing_step,
+    );
+}
+
+/// 准入与续做都推过抬 F 的起点（代码轮 V2 第三形）：单元区 192 槽、不先覆盖写、14 个单元、崩在第 1 批之后；准入推了一串两次空发布，
+/// 续做 13 步、第 7 步之后推了一串两次空发布。续做推的那一串之后的第一步的第一个写报块设备错：错里 13 份 = 落下的 13 个根槽写，
+/// 写账合计 = 落下的写 − 取号两次（两串先写系统配置那 4 个写在内）。
+/// 判别力：先写系统配置那一步不进错时合计少 4。
+#[test]
+fn a_continuation_step_failing_after_both_the_admission_and_the_continuation_raised_the_floor_hands_back_every_landed_write(
+) {
+    let (output, failing_step, refusal, (landed_writes, landed_root_slot_writes)) =
+        mount_failing_on_the_first_write_of_a_continuation_step(
+            NarrowPoolCrashedBetweenDeletionBatches {
+                unit_area_slots: 192,
+                overwrites_before_the_unlink: 0,
+                data_units_of_the_file: DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION,
+                batches_published_after_the_intent: 1,
+            },
+            first_continuation_step_after_the_continuation_raised_the_floor,
+        );
+    assert_eq!(
+        (
+            output
+                .space_admission
+                .floor_raises()
+                .iter()
+                .map(|raised| raised.publishes.len())
+                .collect::<Vec<_>>(),
+            output
+                .floor_raises_pushed_by_the_continuation
+                .iter()
+                .map(|raised| raised.publishes.len())
+                .collect::<Vec<_>>(),
+            output.deletion_queue_continuation.len(),
+        ),
+        (vec![2], vec![2], 13),
+        "起点：准入推了一串、续做里又推了一串，各两次空发布，续做 13 步"
+    );
+    let (_, publishes_before_the_failing_step) =
+        writes_and_publishes_of_the_mount_before(&output, failing_step);
+    assert_eq!(
+        publishes_before_the_failing_step, 13,
+        "写行、暖机、准入推的两次、续做 7 步、续做推的两次"
+    );
+    assert_the_continuation_publish_error_accounts_for_every_landed_write(
+        &refusal,
+        landed_writes,
+        landed_root_slot_writes,
+        publishes_before_the_failing_step,
+    );
+}
+
+/// 续做推的第一串抬 F 之后的第一步（它的 txg 比那一串最后一次空发布大的第一步）。
+fn first_continuation_step_after_the_continuation_raised_the_floor(
+    output: &singlefs_core::mount::MountOutput,
+) -> CheckpointTxg {
+    let last_publish_of_the_raise = output.floor_raises_pushed_by_the_continuation[0]
+        .publishes
+        .last()
+        .expect("推成的一串至少一次空发布")
+        .root
+        .checkpoint_txg;
+    output
+        .deletion_queue_continuation
+        .iter()
+        .map(|step| step.root.checkpoint_txg)
+        .find(|step_txg| *step_txg > last_publish_of_the_raise)
+        .expect("推的那一串之后续做还有一步")
+}
+
+/// 续做里为空间推的抬 F 自己报了不是空间不够的错（代码轮 V1，攻方 `m3-unlink-code-r2` H2 / G1）：192 槽那一种盘（准入推过一串、续做第 7 步之后
+/// 又推一串）上，续做推的那一串第二次空发布的第一个写报块设备错。挂载不再原样交回抬 F 的错，交 `FloorRaiseFailedAfterTheMountsPublishes`：
+/// `cause` 是那一串的 `RaiseFloorSequencePublishFailed`（第一次空发布已落盘），`writes_of_persisted_publishes` 是之前按 txg 落盘的 11 份
+/// （写行、暖机、准入推的两次、续做 7 步），`floor_raises` 是准入推成的那一串；写账合计（各份 + 推成的那几串先写系统配置那一步 + `cause` 里那一串
+/// 自己的账）= 落下的写 − 取号两次，已落盘份数（这里的 11 份 + `cause` 里的 1 份）= 落下的根槽写数。
+/// 判别力：抬 F 的错照旧原样交回（改之前）时交的是 `RaiseFloorSequencePublishFailed`，第一项就红。
+#[test]
+fn a_floor_raise_failing_on_a_device_error_in_the_continuation_hands_back_the_writes_of_the_mounts_publishes(
+) {
+    let shape = NarrowPoolCrashedBetweenDeletionBatches {
+        unit_area_slots: 192,
+        overwrites_before_the_unlink: 0,
+        data_units_of_the_file: DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION,
+        batches_published_after_the_intent: 1,
+    };
+    let (output, failing_publish, refusal, (landed_writes, landed_root_slot_writes)) =
+        mount_failing_on_the_first_write_of_a_continuation_step(shape, |output| {
+            output.floor_raises_pushed_by_the_continuation[0].publishes[1]
+                .root
+                .checkpoint_txg
+        });
+    let MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) = &refusal else {
+        panic!("续做推的抬 F 报块设备错：交 FloorRaiseFailedAfterTheMountsPublishes：{refusal:?}");
+    };
+    let MountError::RaiseFloorSequencePublishFailed(failed_raise) = &failed.cause else {
+        panic!("cause 是续做推的那一串的错原样：{:?}", failed.cause);
+    };
+    let (_, publishes_before_the_failing_publish) =
+        writes_and_publishes_of_the_mount_before(&output, failing_publish);
+    assert_eq!(
+        (
+            failed.writes_of_persisted_publishes.len(),
+            failed_raise.writes_of_persisted_publishes.len(),
+            failed
+                .floor_raises
+                .iter()
+                .map(|raised| raised.publishes.len())
+                .collect::<Vec<_>>(),
+        ),
+        (11, 1, vec![2]),
+        "之前落盘的 11 份、那一串自己落了 1 次、之前推成的是准入那一串"
+    );
+    assert_eq!(
+        (
+            u64::try_from(
+                failed.writes_of_persisted_publishes.len()
+                    + failed_raise.writes_of_persisted_publishes.len()
+            )
+            .expect("份数"),
+            failed.writes_of_persisted_publishes.len()
+                + failed_raise.writes_of_persisted_publishes.len(),
+        ),
+        (
+            landed_root_slot_writes,
+            publishes_before_the_failing_publish
+        ),
+        "已落盘份数等于落下的根槽写数"
+    );
+    assert_eq!(
+        write_calls_of(
+            failed
+                .writes_of_persisted_publishes
+                .iter()
+                .chain(
+                    failed
+                        .floor_raises
+                        .iter()
+                        .map(|raised| &raised.system_configuration_writes_before_the_first_publish)
+                )
+                .chain(std::iter::once(
+                    &failed_raise.writes_before_the_first_publish
+                ))
+                .chain(&failed_raise.writes_of_persisted_publishes)
+                .chain(&failed_raise.writes_of_failed_publishes)
+        ),
+        landed_writes - ACQUISITION_WRITES,
+        "写账合计等于落下的写减去取号那两次"
+    );
+}
+
+/// 卸载那一串（丙 1，回收出来的槽不扣）在先写系统配置之后、第一条带新 F 的根之前掉电（攻方 `m3-unmount-after-short-continuation-r1` 丙 1 那一节
+/// 自列没判的第一件）：续做推满仍放不下的那块窄盘（[`a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue`]
+/// 的造法），先在一份拷贝上不注错地卸载一次，数清那一串第一次空发布写几个单元、其中几个落在这一串自己回收出来的槽上；
+/// 再在另一份上从「先写系统配置那 2 个写 + 第一次空发布的单元写」之后的第一个写起每一个写都报错——单元已写进回收出来的槽，journal 记录与根都没落。
+/// 掉电之后：池级 checker 全绿；只读挂载走的是卸载之前现行那一版（根上带旧 F），而系统配置里已是新 F；再挂可写做成、续做接着走到空，池级 checker 全绿。
+/// 判别力：卸载回收的槽照旧扣到生效时拷贝上那一次卸载就在预演里被拒，第一步红。
+#[test]
+fn an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version(
+) {
+    let (mut pool, _) = pool_crashed_between_deletion_batches_on_narrow_devices(1);
+    let mounted = mount_after_the_crash_with_batches_of_one(&mut pool)
+        .unwrap_or_else(|error| panic!("推满仍放不下：挂载照样做成可写：{error:?}"));
+    let image_after_the_mount = pool.image();
+    let (_, session) = MountedSession::of_the_mount(mounted);
+    let current_root = *session.current.root();
+    let (unit_writes_of_the_first_publish, publishes_of_the_sequence) = {
+        let mut allocator = session.allocator.clone();
+        let mut current = session.current.clone();
+        let (mut devices, _) = fault_injecting_recorded_devices_on(&image_after_the_mount, None);
+        let unmounted = unmount(
+            &pool.parameters,
+            &mut devices,
+            &mut allocator,
+            &mut current,
+            session.shadow_ledger,
+        );
+        let Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) = &unmounted else {
+            panic!("拷贝上卸载做成：{:?}", unmounted.as_ref().err());
+        };
+        let first = &raised.publishes[0];
+        let first_publish_slots_reclaimed_by_the_sequence = first
+            .units
+            .iter()
+            .filter(|unit| first.rewritten.contains(&unit.identity))
+            .filter(|unit| {
+                raised
+                    .reclaimed
+                    .iter()
+                    .any(|placement| placement.slot == unit.slot)
+            })
+            .count();
+        assert!(
+            first_publish_slots_reclaimed_by_the_sequence > 0,
+            "第一次空发布有单元落在这一串自己回收出来的槽上（这一刀要打的就是它们先于带新 F 的根写下去）"
+        );
+        let structure_writes = |kind: WrittenStructureKind| first.writes.of(kind).write_calls;
+        (
+            first.writes.total().write_calls
+                - structure_writes(WrittenStructureKind::JournalRecord)
+                - structure_writes(WrittenStructureKind::RootSlot)
+                - structure_writes(WrittenStructureKind::SystemConfigurationSlot),
+            raised.publishes.len(),
+        )
+    };
+    assert_eq!(publishes_of_the_sequence, 2, "卸载那一串两次空发布");
+    let system_configuration_writes_of_the_sequence = 2;
+    let (mut devices, stream) = fault_injecting_recorded_devices_on(
+        &image_after_the_mount,
+        Some(write_failing_from(
+            system_configuration_writes_of_the_sequence + unit_writes_of_the_first_publish + 1,
+            FailingWrites::ThisWriteAndEveryLaterOne,
+        )),
+    );
+    let mut allocator = session.allocator.clone();
+    let mut current = session.current.clone();
+    let unmounted = unmount(
+        &pool.parameters,
+        &mut devices,
+        &mut allocator,
+        &mut current,
+        session.shadow_ledger,
+    );
+    assert!(
+        matches!(
+            unmounted,
+            Err(MountError::RaiseFloorSequencePublishFailed(_))
+        ),
+        "掉电之后那一串第一次空发布写 journal 记录时报错：{:?}",
+        unmounted.as_ref().err()
+    );
+    assert_eq!(
+        landed_writes_and_root_slot_writes(&stream),
+        (
+            system_configuration_writes_of_the_sequence + unit_writes_of_the_first_publish,
+            0
+        ),
+        "落下的只有先写系统配置那两个写与第一次空发布的单元写，一个根都没落"
+    );
+    let crashed = image_of_the_fault_injecting_recorded_devices(
+        &devices,
+        image_after_the_mount.device_size_in_bytes,
+    );
+    assert_eq!(
+        checker_violations_on(&crashed),
+        Vec::<String>::new(),
+        "掉电那一刻的盘面上池级 checker 全绿"
+    );
+    let read_only = mount_read_only(&crashed).expect("只读挂载");
+    assert_eq!(
+        (
+            read_only.effective_root.checkpoint_txg,
+            read_only.effective_root.rollback_floor,
+            choose_system_configuration(&crashed)
+                .expect("系统配置读得出")
+                .quantities
+                .rollback_floor,
+        ),
+        (
+            current_root.checkpoint_txg,
+            current_root.rollback_floor,
+            current_root.checkpoint_txg,
+        ),
+        "只读挂载走卸载之前现行那一版（根上带旧 F），系统配置里已是新 F（现行那一版的 txg）"
+    );
+    let mut remount_devices = plain_devices_on(&crashed);
+    let remounted = mount_writable_with_test_only_switches_and_deletion_switches(
+        &pool.parameters,
+        &mut remount_devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        deletion_switches_with_batches_of_one(),
+    )
+    .unwrap_or_else(|error| panic!("掉电之后再挂可写：{error:?}"));
+    assert!(
+        matches!(
+            remounted.output.deletion_queue_continuation_outcome,
+            DeletionQueueContinuationOutcome::WalkedToEmpty
+        ) && member_name_of(&remounted.current) == "WithTreesAndNoFile",
+        "再挂之后续做接着走到空：{:?}",
+        remounted.output.deletion_queue_continuation_outcome
+    );
+    drop(remounted);
+    let image_after_the_remount = MemoryPool {
+        devices: remount_devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.image.clone()))
+            .collect(),
+        device_size_in_bytes: crashed.device_size_in_bytes,
+    };
+    assert_eq!(
+        checker_violations_on(&image_after_the_remount),
+        Vec::<String>::new(),
+        "再挂续做之后池级 checker 全绿"
+    );
+}
+
+/// 同一刀（卸载那一串先写系统配置之后、第一次空发布的单元写完、journal 记录与根都没落）之后一块盘丢掉、只剩一块（攻方丙 1 那一节自列没判的第二件）：
+/// 丢掉的是住着现行那一版的根的盘 0（现行那一版 txg 21，21 mod 3 = 0，第一版区域归属 0 / 1 / 0），剩下的盘 1 上最新的根是 txg 19，
+/// 早于它系统配置里的新 F（21）。择根与重放：恢复择到盘 1 上最新的那条根、journal 没往后施加，只读挂载走 txg 19 那一版、根上带旧 F；
+/// 可写挂载在读任何盘之前就被 w 的下限拒。池级 checker 在这块剩下的盘上不全绿，与「只写了系统配置、单元一个都没写」那一刀的判定逐条相同：
+/// 第一次空发布写进回收出来的槽的那几个单元，没让 checker 多判出一条（红的那几条不是丙 1 带来的，见报告）。
+/// 判别力：卸载回收的槽照旧扣到生效时拷贝上那一次卸载就在预演里被拒（这一刀摆不出来），第一步红。
+#[test]
+fn after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_surviving_device_recovers_its_own_newest_root_below_the_floor(
+) {
+    let (mut pool, _) = pool_crashed_between_deletion_batches_on_narrow_devices(1);
+    let mounted = mount_after_the_crash_with_batches_of_one(&mut pool)
+        .unwrap_or_else(|error| panic!("推满仍放不下：挂载照样做成可写：{error:?}"));
+    let image_after_the_mount = pool.image();
+    let (_, session) = MountedSession::of_the_mount(mounted);
+    let current_root = *session.current.root();
+    let unit_writes_of_the_first_publish = {
+        let mut allocator = session.allocator.clone();
+        let mut current = session.current.clone();
+        let (mut devices, _) = fault_injecting_recorded_devices_on(&image_after_the_mount, None);
+        let unmounted = unmount(
+            &pool.parameters,
+            &mut devices,
+            &mut allocator,
+            &mut current,
+            session.shadow_ledger,
+        );
+        let Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) = &unmounted else {
+            panic!("拷贝上卸载做成：{:?}", unmounted.as_ref().err());
+        };
+        let first = &raised.publishes[0];
+        let structure_writes = |kind: WrittenStructureKind| first.writes.of(kind).write_calls;
+        first.writes.total().write_calls
+            - structure_writes(WrittenStructureKind::JournalRecord)
+            - structure_writes(WrittenStructureKind::RootSlot)
+            - structure_writes(WrittenStructureKind::SystemConfigurationSlot)
+    };
+    let system_configuration_writes_of_the_sequence = 2;
+    let surviving_device_after_losing_power_at = |first_failing_write: u64| {
+        let (mut devices, _) = fault_injecting_recorded_devices_on(
+            &image_after_the_mount,
+            Some(write_failing_from(
+                first_failing_write,
+                FailingWrites::ThisWriteAndEveryLaterOne,
+            )),
+        );
+        let mut allocator = session.allocator.clone();
+        let mut current = session.current.clone();
+        let unmounted = unmount(
+            &pool.parameters,
+            &mut devices,
+            &mut allocator,
+            &mut current,
+            session.shadow_ledger,
+        );
+        assert!(
+            matches!(
+                unmounted,
+                Err(MountError::RaiseFloorSequencePublishFailed(_))
+            ),
+            "掉电之后那一串报错：{:?}",
+            unmounted.as_ref().err()
+        );
+        let crashed = image_of_the_fault_injecting_recorded_devices(
+            &devices,
+            image_after_the_mount.device_size_in_bytes,
+        );
+        MemoryPool {
+            devices: crashed
+                .devices
+                .iter()
+                .filter(|(identity, _)| **identity != DeviceIdentity(0))
+                .map(|(identity, device)| (*identity, device.clone()))
+                .collect(),
+            device_size_in_bytes: crashed.device_size_in_bytes,
+        }
+    };
+    let survivor = surviving_device_after_losing_power_at(
+        system_configuration_writes_of_the_sequence + unit_writes_of_the_first_publish + 1,
+    );
+    let survivor_with_only_the_system_configuration_written =
+        surviving_device_after_losing_power_at(system_configuration_writes_of_the_sequence + 1);
+    assert_eq!(
+        (
+            current_root.checkpoint_txg,
+            choose_system_configuration(&survivor)
+                .expect("剩下那块盘的系统配置读得出")
+                .quantities
+                .rollback_floor,
+            recover(&survivor, JournalPolicy::Consult).effective_root,
+        ),
+        (
+            CheckpointTxg(21),
+            CheckpointTxg(21),
+            Some((current_root.instance, CheckpointTxg(19)))
+        ),
+        "剩下的盘 1 系统配置里是新 F 21，恢复择到它上面最新的根 txg 19、journal 没往后施加"
+    );
+    let read_only = mount_read_only(&survivor).expect("剩下那块盘只读挂载");
+    assert_eq!(
+        (
+            read_only.effective_root.checkpoint_txg,
+            read_only.effective_root.rollback_floor,
+        ),
+        (CheckpointTxg(19), current_root.rollback_floor),
+        "只读挂载走 txg 19 那一版，根上带旧 F，低于系统配置里的新 F"
+    );
+    let mut survivor_devices = plain_devices_on(&survivor);
+    assert!(
+        matches!(
+            mount_after_the_crash_on(&pool.parameters, &mut survivor_devices),
+            Err(MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. })
+        ),
+        "只剩一块盘：可写挂载被 w 的下限拒"
+    );
+    assert_eq!(
+        checker_violations_on(&survivor),
+        checker_violations_on(&survivor_with_only_the_system_configuration_written),
+        "第一次空发布写进回收出来的槽的那几个单元没让池级 checker 多判出一条"
+    );
+}
+
+/// 同 [`mount_after_the_crash_with_batches_of_one`]，盘由调用方给。
+fn mount_after_the_crash_on(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, SparseBlockDevice)>,
+) -> Result<Mounted, MountError> {
+    mount_writable_with_test_only_switches_and_deletion_switches(
+        parameters,
+        devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        deletion_switches_with_batches_of_one(),
+    )
+}
+
 /// 被拒的那一种（只分空间不够的两种；会话只因这两种推抬 F，`mounted_session::refusal_is_short_of_space`）。
 fn kind_of_the_refusal_that_pushed(refusal: &PublishError) -> &'static str {
     if matches!(refusal, PublishError::SpaceAdmissionRefused(_)) {
diff --git a/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs b/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
index cdb9078..604838f 100644
--- a/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
+++ b/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
@@ -22,17 +22,20 @@ use singlefs_core::mount::{
 };
 use singlefs_core::mounted_read::{mount_read_only, OpenFileFailure};
 use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
-use singlefs_core::records::TREE_KIND_DELETED_INODES;
+use singlefs_core::records::{
+    InodeRecord, STATISTIC_COMMITTED_RESERVATION_BYTES, TREE_KIND_DELETED_INODES,
+};
 use singlefs_core::recovery::{rebuild_version, RebuiltVersion};
 use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
 use singlefs_core::transaction::{
-    publish_unlink_intent, publish_unlink_step, FirstFile, PoolVersion, PoolWriter,
-    TransactionOutput, FIRST_INODE_NUMBER,
+    publish_new_inodes, publish_overwrite, publish_unlink_intent, publish_unlink_step, FirstFile,
+    PoolVersion, PoolWriter, TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
 };
-use singlefs_core::unit::data_unit_payload_capacity;
+use singlefs_core::unit::{build_index_node, data_unit_payload_capacity, parse_index_node};
 use singlefs_core::unlink::{
     DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
 };
+use singlefs_format::SLOT_BYTES;
 use singlefs_harness::memory_pool::MemoryPool;
 use singlefs_harness::SharedStream;
 
@@ -404,6 +407,175 @@ fn a_writable_mount_after_the_intent_continues_the_queue_to_empty_after_its_warm
     );
 }
 
+/// 第一个文件那条 inode 记录在这一版里的样子。
+fn record_of_the_first_file(version: &TransactionOutput) -> InodeRecord {
+    version
+        .inode_leaf_containers
+        .iter()
+        .flat_map(|container| container.contents.records.iter())
+        .find(|record| record.inode == FIRST_FILE.0)
+        .copied()
+        .expect("意图与中间各批之后第一个文件的记录还在（nlink = 0）")
+}
+
+/// 意图那次与中间一批都算一次改动（用户 2026-10-03 定 Q-F「算改动，改实现」；D8（核心索引结构） 已定项 6 偏移 88 改动计数 =
+/// 最后一次改动所在发布的 checkpoint_txg）：三个单元的文件发意图、再发一批（留 2），两次之后第一个文件那条记录的改动计数各等于那一次发布的 txg；
+/// 写入时间不动（记录里没有 ctime，删除步没有确定的来源）。两次之后池级 checker 全绿（I-9.26（inode 改动计数落在出生代与容器诞生代号之间））。
+/// 判别力：意图那次或那一批照旧带上一版的改动计数时那一项红。
+#[test]
+fn the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_own_publish() {
+    let mut pool = pool_with_a_three_unit_file("change-count-of-the-deletion-steps");
+    let before_the_unlink = record_of_the_first_file(&pool.output);
+    let intent = publish_the_intent_in_process(&mut pool);
+    let middle_batch = publish_the_next_step_in_process(&mut pool);
+    let after_the_intent = record_of_the_first_file(&intent);
+    let after_the_middle_batch = record_of_the_first_file(&middle_batch);
+    assert!(
+        before_the_unlink.change_count < intent.root.checkpoint_txg.0,
+        "改动计数本来低于意图那次的 txg：{} 对 {}",
+        before_the_unlink.change_count,
+        intent.root.checkpoint_txg.0
+    );
+    assert_eq!(
+        (
+            after_the_intent.change_count,
+            after_the_middle_batch.change_count,
+        ),
+        (
+            intent.root.checkpoint_txg.0,
+            middle_batch.root.checkpoint_txg.0,
+        ),
+        "意图那次与中间那一批各把改动计数设成自己那次发布的 txg"
+    );
+    assert_eq!(
+        (
+            after_the_intent.write_time_seconds,
+            after_the_middle_batch.write_time_seconds,
+        ),
+        (
+            before_the_unlink.write_time_seconds,
+            before_the_unlink.write_time_seconds,
+        ),
+        "写入时间不动"
+    );
+    assert_eq!(
+        violations(&pool.memory_pool()),
+        Vec::<String>::new(),
+        "两次之后池级 checker 全绿"
+    );
+}
+
+/// 两块盘的池：标签 6 是全部副本之和，每份一个副本。
+const REPLICAS_OF_THE_TWO_DEVICE_POOL: u64 = 2;
+
+/// 记账标签 6（墓碑预付）里的 H 从墓碑树根节点的码 2 头现读（代码轮 U8，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节；
+/// D3（空间分配） 已定项 2 字面）：建一个空 inode 再 unlink 它（意图那次开出墓碑树的第一片容器、根层级 1，H = 0），在内存里那一版上把墓碑树根单元
+/// 头里的层级改成 3（形状不动），直接在它上面发一次覆盖写（写入类发布，墓碑树照抄）：标签 6 比在原样那一版上发的多 2 × 16384 × 副本数——
+/// 按头里的层级算，不按内存里那份形状。
+/// 判别力：H 照旧读内存里的形状时两次一样多，那一项红。
+#[test]
+fn the_tombstone_prepay_of_an_overwrite_reads_the_tombstone_tree_height_from_the_root_node_header()
+{
+    let mut pool = build_pool("tombstone-prepay-reads-the-root-node-header");
+    let second_inode = InodeNumber(FIRST_INODE_NUMBER + 1);
+    let with_a_tombstone = {
+        let publish_parameters = parameters();
+        let previous = pool.output.clone();
+        let devices = pool.devices.as_mut().expect("镜像还开着");
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        let with_a_second_inode = publish_new_inodes(
+            &mut writer,
+            &mut pool.allocator,
+            &previous,
+            1,
+            FIXED_WRITE_TIME_SECONDS,
+            INSTANCE_OF_THE_PROCESS,
+        )
+        .expect("建一个空 inode");
+        let intent = publish_unlink_intent(
+            &mut writer,
+            &mut pool.allocator,
+            &with_a_second_inode,
+            second_inode,
+            INSTANCE_OF_THE_PROCESS,
+        )
+        .expect("unlink 那个空 inode 的意图");
+        publish_unlink_step(
+            &mut writer,
+            &mut pool.allocator,
+            &intent,
+            second_inode,
+            batches_of_one_data_unit(),
+            INSTANCE_OF_THE_PROCESS,
+        )
+        .expect("它没有数据单元：意图之后直接是最后一批")
+    };
+    pool.output = with_a_tombstone.clone();
+    let root_header =
+        parse_index_node(&with_a_tombstone.unit(TransactionUnit::TombstoneRoot).bytes)
+            .expect("墓碑树根解得开");
+    assert_eq!(
+        (
+            root_header.level,
+            with_a_tombstone.tombstone_containers.len()
+        ),
+        (1, 1),
+        "一片容器、根层级 1：H = 0"
+    );
+    let mut filesystem_identifier = [0_u8; 16];
+    filesystem_identifier[..8].copy_from_slice(&root_header.filesystem_identifier.to_le_bytes());
+    let root_two_levels_higher = build_index_node(
+        root_header.tree,
+        root_header.level + 2,
+        root_header.key_width,
+        &root_header.smallest_key,
+        &root_header.largest_key,
+        root_header.birth_txg,
+        &filesystem_identifier,
+        root_header.instance,
+        root_header.birth_sequence,
+        u16::try_from(root_header.entry_width).expect("条目宽两字节"),
+        &root_header.entries,
+    );
+    let mut with_the_root_header_two_levels_higher = with_a_tombstone.clone();
+    with_the_root_header_two_levels_higher
+        .units
+        .iter_mut()
+        .find(|unit| unit.identity == TransactionUnit::TombstoneRoot)
+        .expect("这一版的单元里有墓碑树根")
+        .bytes = root_two_levels_higher;
+    let image = pool.memory_pool();
+    let tombstone_prepay_after_an_overwrite_on = |previous: &TransactionOutput| {
+        let mut allocator = pool.allocator.clone();
+        let mut devices = crash_state_devices(&image, &[], &[], &SharedStream::new());
+        let publish_parameters = parameters();
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        let content = vec![7_u8; 100];
+        publish_overwrite(
+            &mut writer,
+            &mut allocator,
+            previous,
+            FirstFile {
+                content: &content,
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
+            },
+            INSTANCE_OF_THE_PROCESS,
+        )
+        .expect("覆盖写做成")
+        .accounting_entries
+        .iter()
+        .find(|entry| entry.statistic == STATISTIC_COMMITTED_RESERVATION_BYTES)
+        .expect("记账里有标签 6 那一行")
+        .value
+    };
+    assert_eq!(
+        tombstone_prepay_after_an_overwrite_on(&with_the_root_header_two_levels_higher)
+            - tombstone_prepay_after_an_overwrite_on(&with_a_tombstone),
+        2 * SLOT_BYTES * REPLICAS_OF_THE_TWO_DEVICE_POOL,
+        "标签 6 按墓碑树根节点头里的层级算：头里高两层，H 多 2"
+    );
+}
+
 #[test]
 fn two_copies_of_the_image_between_two_batches_continue_to_byte_identical_disks() {
     let mut pool = pool_with_a_three_unit_file("two-copies-continue-the-same-batches");
@@ -545,8 +717,9 @@ impl<Inner: singlefs_core::block_device::BlockDevice> singlefs_core::block_devic
 }
 
 /// 续做里的一步没发成、不是空间不够（块设备错）：挂载交回 `MountError::Publish`，错里的已落盘写账（`writes_of_persisted_publishes`）
-/// 一次发布一份、按先后，份数等于这次挂载落在盘上的发布数（写行、暖机、续做第一步），每一份都有写（代码轮 U9，
-/// `PublishSequenceFailed::writes_of_persisted_publishes` 的契约）。
+/// 一次发布一份、按先后，每一份都有写（代码轮 U9，`PublishSequenceFailed::writes_of_persisted_publishes` 的契约）。份数等于这次挂载落在盘上的
+/// 发布数（写行、暖机、续做第一步）只在这条用例挑的注错点上成立：注错点放在续做第二步的第一个写上。注错点落在一次发布的记录写之后
+/// （根槽写、轮换的系统配置写）时，只读恢复靠那条记录多走一次，份数比只读挂载看到的少 1（攻方 `m3-unlink-code-r2` H4，代码轮 V4），这条用例不判那几点。
 /// 造法：三个单元的文件发完意图就崩（续做三步），同一份镜像先在一份拷贝上挂一次数清写调用，再在另一份上让续做第二步的第一次写报错。
 /// 判别力：续做那一处的错照旧交空的已落盘写账时，份数是 0、盘上落了 1 + 暖机 + 1 次，那一项红。
 #[test]
````

## 2、实现员 Y（挂载透明认续做、删完那一版上 publish_new_inodes、U4、V1 划段、V6）

`/tmp/claude-1000/m3-batch12/y/patch/crates.patch`

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index 8a19e71..ceebabd 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -40,15 +40,13 @@ use singlefs_core::address::{
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
-use singlefs_core::mount::{
-    mount_writable_with_space_admission, MountError, MountOutput, RaisedFloor,
-};
+use singlefs_core::mount::{mount_writable_with_space_admission, MountError};
 use singlefs_core::recovery::{
     choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryOutcome,
     RecoveryReport,
 };
 use singlefs_core::transaction::{
-    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
+    publish_first_file, publish_new_inodes, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
 };
 use singlefs_core::write_accounting::{WritesByStructureKind, WrittenStructureKind};
 
@@ -79,7 +77,7 @@ use singlefs_harness::mount_transparency::{
     judge_mount_transparency, judge_writable_mount_refusal, read_recovery_fields, ComparedVersion,
     DiskAfterTheRefusal, FieldOutcome, FloorRaisedByTheMount, FloorRaisesInTheMountWrites,
     MountTransparency, MountTransparencyAction, ReadOnlyReadBackOfTheCrashImage,
-    RecoveryFieldsRead, WritableMountRefusalVerdict,
+    RecoveryFieldsRead, UserVisibleTreeRootPointersOfTheContinuation, WritableMountRefusalVerdict,
 };
 use singlefs_harness::numeric_environment_variable::{
     number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
@@ -1292,6 +1290,9 @@ pub fn inject_crashes_into_history(
                 fields_read_after_the_first_crash: &fields_read_after_the_crash,
                 floor_raises_in_the_uninterrupted_mount_writes: mount_run
                     .floor_raises_in_the_mount_writes,
+                continuation_of_the_uninterrupted_mount: mount_run
+                    .user_visible_tree_root_pointers_of_the_continuation
+                    .as_ref(),
                 publishes_before_the_deletion_queue_continuation: mount_run
                     .publishes_before_the_deletion_queue_continuation,
                 parameters: &parameters,
@@ -1581,11 +1582,15 @@ pub struct WritableMountOnTheCrashImage {
     /// 挂载交回值里推没推抬 F、最后抬到的 F（挂载透明要它）；挂载被拒时 None。
     pub floor_raised_by_the_mount: Option<FloorRaisedByTheMount>,
     /// 挂载续做之前这次挂载发了几次（写行、暖机、推的抬 F 的每一次空发布）：挂载那一段录制流里第几个根槽写之后起是续做；
-    /// 挂载交回 `MountError::Publish` 时按错里交回的已落盘发布写账划（划不出续做、别的错都是 None：写行之后都算暖机）。
+    /// 挂载交回 `MountError::Publish` 或 `MountError::FloorRaiseFailedAfterTheMountsPublishes` 时按错里交回的已落盘发布写账划
+    /// （[`publishes_before_the_continuation_of_a_mount_that_failed`]；划不出续做、别的错都是 None：写行之后都算暖机）。
     pub publishes_before_the_deletion_queue_continuation: Option<usize>,
     /// 挂载推的抬 F 从挂载那一段写表（`mount_operations` 按同一份几何切出来的写表）的哪个写起（二次崩溃上 F 判不判按它分；
     /// 挂载准入推的与续做里推的都认，界按它们在写表里的位置取，取不出的是 [`FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites`]）；挂载被拒时 None。
     pub floor_raises_in_the_mount_writes: Option<FloorRaisesInTheMountWrites>,
+    /// 挂载续做每一步那一版的两棵用户可见树的根指针（挂载透明那一样按它比，用户 2026-10-03 定「续做算挂载的一部分」）；挂载被拒时 None。
+    pub user_visible_tree_root_pointers_of_the_continuation:
+        Option<UserVisibleTreeRootPointersOfTheContinuation>,
 }
 
 /// 挂载之后那一次发布写的内容：随种子与崩溃状态变，装得进一个数据单元（模型只罩一个数据单元的文件）。
@@ -1598,7 +1603,8 @@ fn content_of_the_publish_after_the_crash(seed: HistorySeed, crash_point: &Crash
 }
 
 /// 在 `crash_image_materialized` 的副本上起可写挂载（建池那一份参数、那一段历史的空间准入开关），做成了就接着发一次布：
-/// 挂载交回的现行版本带文件时覆盖写，树表 0 条时发第一个文件（与随机历史里这两个入口同一种调法）。
+/// 挂载交回的现行版本带文件时覆盖写，树表 0 条时发第一个文件（与随机历史里这两个入口同一种调法），有树而文件集合为空（删完那一版）时
+/// 建一个 size 0 的 inode（D8（核心索引结构） 已定项 16 第 5 条）。
 #[must_use]
 pub fn mount_and_publish_once_on_the_crash_image(
     crash_image_materialized: &MemoryPool,
@@ -1642,18 +1648,14 @@ pub fn mount_and_publish_once_on_the_crash_image(
                     .map(|raised| raised.publishes.len())
                     .sum::<usize>(),
         ),
-        // 挂载在续做里交回错（U12，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：续做从第几次发布起按错里交回的
-        // 已落盘发布写账划（[`publishes_before_the_continuation_of_a_mount_that_handed_back_an_error`]），不并进暖机段。
-        Err(MountError::Publish(failed)) => {
-            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
-                &failed.writes_of_persisted_publishes,
-                &failed.writes_of_failed_publishes,
-            )
-        }
-        Err(_) => None,
+        Err(error) => publishes_before_the_continuation_of_a_mount_that_failed(error),
     };
+    let user_visible_tree_root_pointers_of_the_continuation =
+        mounted.as_ref().ok().map(|mounted| {
+            UserVisibleTreeRootPointersOfTheContinuation::of_the_mount_output(&mounted.output)
+        });
     // 录制流此刻只有挂载那一段：按建池那一份参数的几何切出写表，与调用方切二次崩溃的写表同一张。抬 F 的界按位置取（挂载准入推的与续做里推的
-    // 那几串都认，[`floor_raises_in_the_mount_writes_counting_the_continuation`]），取不出时不 panic、交 NotLocatedInTheMountWrites，
+    // 那几串都认，`FloorRaisesInTheMountWrites::of_the_mount_output`），取不出时不 panic、交 NotLocatedInTheMountWrites，
     // 这一次挂载的二次崩溃整段 F 不判、按理由计进 `mount_transparency_fields_not_judged`。
     let floor_raises_in_the_mount_writes = mounted.as_ref().ok().map(|mounted| {
         let geometry = FixedGeometry {
@@ -1663,7 +1665,7 @@ pub fn mount_and_publish_once_on_the_crash_image(
         };
         let (mount_writes, _mount_segments, _mount_stream_indexes) =
             writes_and_segments_with_stream_indexes(&stream.retained_operations(), &geometry);
-        floor_raises_in_the_mount_writes_counting_the_continuation(&mounted.output, &mount_writes)
+        FloorRaisesInTheMountWrites::of_the_mount_output(&mounted.output, &mount_writes)
     });
     let outcome = match mounted {
         Err(error) => WritableMountOutcome::MountRefused {
@@ -1680,16 +1682,25 @@ pub fn mount_and_publish_once_on_the_crash_image(
             let mut allocator = mounted.allocator;
             let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
             let published = match &mounted.current {
-                PoolVersion::WithFile(current) | PoolVersion::WithTreesAndNoFile(current) => {
-                    publish_overwrite(
-                        &mut writer,
-                        &mut allocator,
-                        current,
-                        file,
-                        mounted.output.instance,
-                    )
-                    .map(|_| ())
-                }
+                PoolVersion::WithFile(current) => publish_overwrite(
+                    &mut writer,
+                    &mut allocator,
+                    current,
+                    file,
+                    mounted.output.instance,
+                )
+                .map(|_| ()),
+                // 删完那一版（D8（核心索引结构） 已定项 16 第 5 条：之后只许建 inode、写行与空发布）：建一个 size 0 的 inode
+                // （随机历史的「建 inode」经会话建的也是一个）；覆盖写在这一版上被拒成写一个不在树里的 inode。
+                PoolVersion::WithTreesAndNoFile(current) => publish_new_inodes(
+                    &mut writer,
+                    &mut allocator,
+                    current,
+                    1,
+                    FIXED_WRITE_TIME_SECONDS,
+                    mounted.output.instance,
+                )
+                .map(|_| ()),
                 PoolVersion::WithoutFile(current) => publish_first_file(
                     &mut writer,
                     &mut allocator,
@@ -1726,200 +1737,19 @@ pub fn mount_and_publish_once_on_the_crash_image(
         floor_raised_by_the_mount,
         publishes_before_the_deletion_queue_continuation,
         floor_raises_in_the_mount_writes,
-    }
-}
-
-/// 挂载里一次发布交给设备的写数与它写的根身份 (实例代号, checkpoint_txg)。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
-struct PublishInTheMountWrites {
-    write_calls: u64,
-    root: (InstanceGeneration, CheckpointTxg),
-}
-
-impl PublishInTheMountWrites {
-    fn of_the_publish(publish: &TransactionOutput) -> Self {
-        PublishInTheMountWrites {
-            write_calls: publish.writes.total().write_calls,
-            root: (publish.root.instance, publish.root.checkpoint_txg),
-        }
-    }
-}
-
-/// 挂载推的一串抬 F 的写账：先把新 F 写进每块盘系统配置的那几个写（`RaisedFloor::system_configuration_writes_before_the_first_publish`），
-/// 再是各次带新 F 的空发布（按先后）。
-#[derive(Clone, Debug, PartialEq, Eq)]
-struct FloorRaiseInTheMountWrites {
-    system_configuration_write_calls: u64,
-    publishes: Vec<PublishInTheMountWrites>,
-}
-
-impl FloorRaiseInTheMountWrites {
-    fn of_the_raise(raise: &RaisedFloor) -> Self {
-        FloorRaiseInTheMountWrites {
-            system_configuration_write_calls: raise
-                .system_configuration_writes_before_the_first_publish
-                .total()
-                .write_calls,
-            publishes: raise
-                .publishes
-                .iter()
-                .map(PublishInTheMountWrites::of_the_publish)
-                .collect(),
-        }
-    }
-
-    fn write_calls(&self) -> u64 {
-        self.system_configuration_write_calls
-            + self
-                .publishes
-                .iter()
-                .map(|publish| publish.write_calls)
-                .sum::<u64>()
-    }
-
-    /// # Panics
-    /// 这一串一次空发布都没有：抬 F 那一串推空发布直到每块盘上都有带新 F 的根，至少一次（`mount::RaisedFloor::publishes`）。
-    fn checkpoint_txg_of_the_first_publish(&self) -> CheckpointTxg {
-        self.publishes
-            .first()
-            .expect("抬 F 那一串至少推一次带新 F 的空发布（RaisedFloor::publishes 的契约）")
-            .root
-            .1
-    }
-}
-
-/// 一次可写挂载交回的、取抬 F 那条界要用的三样：挂载准入在写行与暖机之后推的那几串（`MountOutput::space_admission`）、
-/// 续做发成的每一步（`MountOutput::deletion_queue_continuation`）与续做里为空间推的那几串（`MountOutput::floor_raises_pushed_by_the_continuation`，
-/// 与续做的各步按 txg 交错）。挂载写表的末尾依次是：准入推的那几串、续做的各步与续做推的那几串。
-#[derive(Clone, Debug, PartialEq, Eq)]
-struct FloorRaisesOfTheMount {
-    pushed_by_the_admission: Vec<FloorRaiseInTheMountWrites>,
-    continuation_steps: Vec<PublishInTheMountWrites>,
-    pushed_by_the_continuation: Vec<FloorRaiseInTheMountWrites>,
-}
-
-impl FloorRaisesOfTheMount {
-    fn of_the_mount_output(output: &MountOutput) -> Self {
-        FloorRaisesOfTheMount {
-            pushed_by_the_admission: output
-                .space_admission
-                .floor_raises()
-                .iter()
-                .map(FloorRaiseInTheMountWrites::of_the_raise)
-                .collect(),
-            continuation_steps: output
-                .deletion_queue_continuation
-                .iter()
-                .map(PublishInTheMountWrites::of_the_publish)
-                .collect(),
-            pushed_by_the_continuation: output
-                .floor_raises_pushed_by_the_continuation
-                .iter()
-                .map(FloorRaiseInTheMountWrites::of_the_raise)
-                .collect(),
-        }
-    }
-
-    /// 从第一串抬 F 的第一个写起、到挂载写表末尾一共几个写，连同这一截里每一次抬 F 发布的根身份；一串都没推时 None。
-    /// 第一串是准入推的（推了的话）：它之后是准入推的别的串、续做的每一步与续做推的每一串。准入一串都没推、续做里推了的：
-    /// 第一串是续做推的第一串，它之后是续做推的每一串与 txg 大于它第一次空发布的那几步续做（之前的那几步在界之前）。
-    fn writes_from_the_first_floor_raise_on(
-        &self,
-    ) -> Option<(u64, Vec<(InstanceGeneration, CheckpointTxg)>)> {
-        let writes_of = |raises: &[FloorRaiseInTheMountWrites]| {
-            raises
-                .iter()
-                .map(FloorRaiseInTheMountWrites::write_calls)
-                .sum::<u64>()
-        };
-        let roots_of = |raises: &[FloorRaiseInTheMountWrites]| {
-            raises
-                .iter()
-                .flat_map(|raise| raise.publishes.iter().map(|publish| publish.root))
-                .collect::<Vec<_>>()
-        };
-        let continuation_steps_after = |checkpoint_txg: Option<CheckpointTxg>| {
-            self.continuation_steps
-                .iter()
-                .filter(|step| checkpoint_txg.is_none_or(|boundary| step.root.1 > boundary))
-                .map(|step| step.write_calls)
-                .sum::<u64>()
-        };
-        if !self.pushed_by_the_admission.is_empty() {
-            let mut roots = roots_of(&self.pushed_by_the_admission);
-            roots.extend(roots_of(&self.pushed_by_the_continuation));
-            return Some((
-                writes_of(&self.pushed_by_the_admission)
-                    + continuation_steps_after(None)
-                    + writes_of(&self.pushed_by_the_continuation),
-                roots,
-            ));
-        }
-        let first_pushed_by_the_continuation = self.pushed_by_the_continuation.first()?;
-        Some((
-            writes_of(&self.pushed_by_the_continuation)
-                + continuation_steps_after(Some(
-                    first_pushed_by_the_continuation.checkpoint_txg_of_the_first_publish(),
-                )),
-            roots_of(&self.pushed_by_the_continuation),
-        ))
-    }
-}
-
-/// 挂载推的抬 F 从挂载写表的哪个写起（[`FloorRaisesInTheMountWrites`]；`singlefs_harness::mount_transparency` 那一份
-/// `FloorRaisesInTheMountWrites::of_the_mount_output` 只认挂载准入推的、按「抬 F 那几串在写表末尾」取界，挂载有续做时取不出、
-/// 续做里推的看不见）：挂载准入推的与续做里推的都认（[`FloorRaisesOfTheMount::writes_from_the_first_floor_raise_on`]），
-/// 界 = 写表长度减去从第一串抬 F 起的写数。取出的界再核两样：界上那个写是系统配置槽写（每一串先把新 F 写进系统配置），
-/// 每一次抬 F 发布的根槽写（按实例与 txg 认）都在界之后；对不上的交 [`FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites`]，不 panic。
-fn floor_raises_in_the_mount_writes_counting_the_continuation(
-    output: &MountOutput,
-    mount_writes: &[RetainedWrite],
-) -> FloorRaisesInTheMountWrites {
-    floor_raises_located_in_the_mount_writes(
-        &FloorRaisesOfTheMount::of_the_mount_output(output),
-        mount_writes,
-    )
-}
-
-fn floor_raises_located_in_the_mount_writes(
-    floor_raises: &FloorRaisesOfTheMount,
-    mount_writes: &[RetainedWrite],
-) -> FloorRaisesInTheMountWrites {
-    let Some((writes_from_the_first_floor_raise_on, floor_raise_roots)) =
-        floor_raises.writes_from_the_first_floor_raise_on()
-    else {
-        return FloorRaisesInTheMountWrites::NotPushed;
-    };
-    let Some(first_write) = usize::try_from(writes_from_the_first_floor_raise_on)
-        .ok()
-        .and_then(|writes| mount_writes.len().checked_sub(writes))
-    else {
-        return FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites;
-    };
-    let the_boundary_is_a_system_configuration_slot_write = mount_writes
-        .get(first_write)
-        .is_some_and(|write| write.kind == StepKind::SystemConfigurationSlot);
-    let roots_written_from_the_boundary: Vec<(InstanceGeneration, CheckpointTxg)> = mount_writes
-        [first_write..]
-        .iter()
-        .filter(|write| write.kind == StepKind::RootRecordFua)
-        .filter_map(|write| write.bytes().map(root_identity_written_by))
-        .collect();
-    let every_floor_raise_root_is_written_from_the_boundary = floor_raise_roots
-        .iter()
-        .all(|root| roots_written_from_the_boundary.contains(root));
-    if the_boundary_is_a_system_configuration_slot_write
-        && every_floor_raise_root_is_written_from_the_boundary
-    {
-        FloorRaisesInTheMountWrites::FirstWriteAt(first_write)
-    } else {
-        FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites
+        user_visible_tree_root_pointers_of_the_continuation,
     }
 }
 
 /// 一次发布写没写只有续做的删除步才写的那几种结构：文件的 extent 树、inode 树与 `deleted_inodes` 树（数据单元也算，挂载里没有别的发布写它）。
 /// 写行、暖机与抬 F 的空发布只重写分配记录树、记账树、中央映射树、树表、实例表与固定结构（2026-10-03 在副本上量的：崩在意图之后重挂，
 /// 写行与两次暖机写的是这几种，续做那一步写了 extent 树与 inode 树根）。
+/// 五种里 `DataUnit` 与 `DeletedInodesTreeNode` 在那次探针上空转（续做那一步没写它们）、仍留着（代码轮第二轮判决 V6）：这一判认的是
+/// 「写行、暖机与抬 F 的空发布都不写的结构」，不是「今天的续做那一步写了的结构」，两种都属于前者——那几种发布不碰文件数据，也不碰
+/// `deleted_inodes`（队列只由删除类发布改），所以认它们不会把写行或暖机认成续做；去掉它们，划段就随续做那一步今天的写法变：
+/// 第一版只有一个文件、走空那一步删 key 时 `deleted_inodes` 根指针置零（D8（核心索引结构） 已定项 16 第 5 条）而不写它的节点，
+/// 队列里有不止一条 key 时删一条要重写那一片（推的，没量）；删除批只释放数据单元、不写（已定项 16 第 2 条），以后某一步续做写了数据单元，
+/// 它也只可能是续做。
 fn writes_a_structure_only_the_continuation_changes(writes: &WritesByStructureKind) -> bool {
     WrittenStructureKind::IN_REPORT_ORDER.iter().any(|kind| {
         let only_the_continuation_writes_it = match kind {
@@ -1943,6 +1773,54 @@ fn writes_a_structure_only_the_continuation_changes(writes: &WritesByStructureKi
     })
 }
 
+/// 挂载交回错时续做从第几次发布起（U12，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节；V1，
+/// `research/prompts/m3-unlink-code-r2-main-verification.md` 第三节）：带着这次挂载已落盘发布写账的两个成员按同一个划法划
+/// （[`publishes_before_the_continuation_of_a_mount_that_handed_back_an_error`]），不并进暖机段——`Publish`（写行、暖机或续做里的一步没发成）
+/// 连同失败那一次的写账；`FloorRaiseFailedAfterTheMountsPublishes`（准入或续做里推的抬 F 自己报了不是空间不够的错）按它的
+/// `writes_of_persisted_publishes`，抬 F 那一串自己的写账在 `cause` 里，都是空发布、不写只有续做才写的结构，不交进来。
+/// 别的错在写行之前（取号之前的判、取号自己）就交回，没有续做：None。
+fn publishes_before_the_continuation_of_a_mount_that_failed(error: &MountError) -> Option<usize> {
+    match error {
+        MountError::Publish(failed) => {
+            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
+                &failed.writes_of_persisted_publishes,
+                &failed.writes_of_failed_publishes,
+            )
+        }
+        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {
+            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
+                &failed.writes_of_persisted_publishes,
+                &[],
+            )
+        }
+        MountError::Recovery(_)
+        | MountError::FileVersionWithoutAnyJournalRecord
+        | MountError::InstanceTableMalformed
+        | MountError::Acquisition(_)
+        | MountError::RaiseFloorSequencePublishFailed(_)
+        | MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. }
+        | MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_)
+        | MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided { .. }
+        | MountError::RollbackFloorAboveCeiling { .. }
+        | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
+        | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
+        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
+        | MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { .. }
+        | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
+        | MountError::SpaceAdmissionRefusedBeforeAcquisition { .. }
+        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
+        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
+        | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. }
+        | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
+        | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. }
+        | MountError::DeviceIdentitiesHandedInMoreThanOnce { .. }
+        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
+        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
+        | MountError::SequenceNumberPastTheTopOfItsRange(_)
+        | MountError::NewerStateStillUnreadableAfterOneReread(_) => None,
+    }
+}
+
 /// 挂载在写行之后交回 `MountError::Publish` 时续做之前发了几次（U12，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：
 /// `writes_of_persisted_publishes` 是已落盘的各次发布的写账（按先后，写行那次在最前，`mount::PublishSequenceFailed`），
 /// `writes_of_failed_publishes` 是落盘阶段失败的那一次的。写行之后第一个写了只有续做才写的结构
@@ -2302,7 +2180,7 @@ fn violations_of_the_mount_transparency(
 /// 判红的落成这一截（[`CrashStateStage::MountTransparencyAfterTheCrash`]）的新发现，镜像留的是动作之后的池。
 #[allow(
     clippy::too_many_arguments,
-    reason = "每个参数各是一样东西：计数与发现、哪个动作、两次恢复与动作之后的池、动作的交回值、这一个崩溃状态落在哪"
+    reason = "每个参数各是一样东西：计数与发现、哪个动作、两次恢复与动作之后的池、动作与挂载续做那几步的交回值、这一个崩溃状态落在哪"
 )]
 fn judge_mount_transparency_of_one_comparison(
     tally: &mut CrashInjectionTally,
@@ -2312,11 +2190,13 @@ fn judge_mount_transparency_of_one_comparison(
     pool_after_the_action: &MemoryPool,
     after_the_action: &RecoveryFieldsRead,
     action: MountTransparencyAction,
+    continuation: &UserVisibleTreeRootPointersOfTheContinuation,
     seed: HistorySeed,
     crash_point: &CrashPoint,
     image_directory: Option<&Path>,
 ) {
-    let transparency = judge_mount_transparency(before_the_mount, after_the_action, action);
+    let transparency =
+        judge_mount_transparency(before_the_mount, after_the_action, action, continuation);
     match &transparency {
         MountTransparency::Judged(_) => {
             *tally
@@ -2361,6 +2241,16 @@ fn judge_mount_transparency_of_one_comparison(
     );
 }
 
+/// 挂载做成了（`floor_raised_by_the_mount` 是 Some）时一起交回的续做那几步的两根指针。
+fn continuation_of_a_mount_that_was_made(
+    mount_run: &WritableMountOnTheCrashImage,
+) -> &UserVisibleTreeRootPointersOfTheContinuation {
+    mount_run
+        .user_visible_tree_root_pointers_of_the_continuation
+        .as_ref()
+        .expect("挂载做成时抬 F 与续做那几步的根指针一起交回（mount_and_publish_once_on_the_crash_image 里同一个 Ok 取的两样）")
+}
+
 /// 第一次崩溃镜像上的挂载透明（r5 判决第二节 U5）：只挂载（v2a，`after_the_uninterrupted_mount` 是那个池与它上面的只读恢复）、
 /// 挂载加一次发布（比发布之前那一版）各判一次。挂载被拒的不判（判的是拒绝本身），发布被拒的不判「挂载加一次发布」那一个。
 #[allow(
@@ -2389,6 +2279,7 @@ fn judge_mount_transparency_on_the_crash_image(
             pool,
             after_the_mount,
             MountTransparencyAction::MountAlone(floor),
+            continuation_of_a_mount_that_was_made(mount_run),
             seed,
             crash_point,
             image_directory,
@@ -2414,6 +2305,7 @@ fn judge_mount_transparency_on_the_crash_image(
                 &mount_run.pool_after,
                 &after_the_publish,
                 MountTransparencyAction::MountAndOnePublish(floor),
+                continuation_of_a_mount_that_was_made(mount_run),
                 seed,
                 crash_point,
                 image_directory,
@@ -2443,6 +2335,7 @@ fn mount_transparency_violations_of_a_second_crash(
     fields_read_after_the_first_crash: &RecoveryFieldsRead,
     fields_read_after_the_second_crash: &RecoveryFieldsRead,
     floor_raises_in_the_uninterrupted_mount_writes: Option<FloorRaisesInTheMountWrites>,
+    continuation_of_the_uninterrupted_mount: Option<&UserVisibleTreeRootPointersOfTheContinuation>,
     persisted_in_the_mount: &[bool],
     phase: WritableMountPhase,
 ) -> Vec<(&'static str, String)> {
@@ -2454,11 +2347,15 @@ fn mount_transparency_violations_of_a_second_crash(
             .or_insert(0) += 1;
         return Vec::new();
     };
+    let continuation = continuation_of_the_uninterrupted_mount.expect(
+        "没崩过的那一次挂载做成时抬 F 的界与续做那几步的根指针一起交回（mount_and_publish_once_on_the_crash_image 里同一个 Ok 取的两样）",
+    );
     let floor_raises = floor_raises_in_the_mount_writes.at_a_second_crash(persisted_in_the_mount);
     let transparency = judge_mount_transparency(
         fields_read_after_the_first_crash,
         fields_read_after_the_second_crash,
         MountTransparencyAction::SecondCrashInsideTheMount(floor_raises),
+        continuation,
     );
     match &transparency {
         MountTransparency::Judged(_) => {
@@ -2606,6 +2503,9 @@ struct SecondCrashInputs<'history> {
     fields_read_after_the_first_crash: &'history RecoveryFieldsRead,
     /// 没崩过的那一次挂载推的抬 F 从挂载那一段写表的哪个写起（没推是 [`FloorRaisesInTheMountWrites::NotPushed`]）；挂载被拒时 None。
     floor_raises_in_the_uninterrupted_mount_writes: Option<FloorRaisesInTheMountWrites>,
+    /// 没崩过的那一次挂载续做每一步那一版的两根指针（二次崩溃落到挂载写的那一版时挂载透明按它比）；挂载被拒时 None。
+    continuation_of_the_uninterrupted_mount:
+        Option<&'history UserVisibleTreeRootPointersOfTheContinuation>,
     /// 二次崩溃之后再挂用的建池参数与空间准入开关（与没崩过的那一次挂载同一份）。
     parameters: &'history MakeFilesystemParameters,
     space_admission: SpaceAdmission,
@@ -2637,6 +2537,7 @@ fn judge_second_crashes_inside_the_writable_mount(
         publishes_before_the_deletion_queue_continuation,
         fields_read_after_the_first_crash,
         floor_raises_in_the_uninterrupted_mount_writes,
+        continuation_of_the_uninterrupted_mount,
         parameters,
         space_admission,
         reference_filesystem_at_the_first_crash,
@@ -2742,6 +2643,7 @@ fn judge_second_crashes_inside_the_writable_mount(
             fields_read_after_the_first_crash,
             &fields_read_after_the_second_crash,
             floor_raises_in_the_uninterrupted_mount_writes,
+            continuation_of_the_uninterrupted_mount,
             &persisted_in_the_mount,
             second.phase,
         ));
@@ -3723,8 +3625,8 @@ pub fn seed_slices(seed_count: u64, worker_threads: usize) -> Vec<std::ops::Rang
 #[cfg(test)]
 mod tests {
     use super::*;
+    use singlefs_core::mount::FloorRaiseFailedAfterTheMountsPublishes;
     use singlefs_harness::history::{generate_history, HistoryStartingPoint};
-    use singlefs_harness::mount_transparency::FloorRaisesAtTheSecondCrash;
 
     const FOUR_CRASH_POINTS: CrashPointDraw = CrashPointDraw::Sampled {
         crash_points_per_history: 4,
@@ -3865,121 +3767,6 @@ mod tests {
         }
     }
 
-    fn publish_of(write_calls: u64, instance: u32, checkpoint_txg: u64) -> PublishInTheMountWrites {
-        PublishInTheMountWrites {
-            write_calls,
-            root: (InstanceGeneration(instance), CheckpointTxg(checkpoint_txg)),
-        }
-    }
-
-    /// 挂载续做里为空间推的抬 F（D8（核心索引结构） 已定项 16 第 6 条，用户 2026-10-03 定「挂载续做被空间拒：同样推抬 F 再试」）：
-    /// 那一串夹在续做的两步之间、不在挂载写表末尾，界按它在写表里的位置取。造的写表：取号；写行（实例 2、txg 10）；暖机一次（txg 11）；
-    /// 续做第一步（txg 12）；续做推的一串抬 F（先把新 F 写进系统配置，再一次空发布 txg 13）；续做第二步（txg 14）。
-    /// 判别力：只认挂载准入推的那几串（`singlefs_harness::mount_transparency` 那一份的做法）时这一形交「没推」，
-    /// 二次崩溃崩在那一串里时 F 照「没推」判。
-    #[test]
-    fn a_floor_raise_pushed_inside_the_continuation_is_located_between_the_continuation_steps() {
-        let writes = vec![
-            synthetic_write(StepKind::SystemConfigurationSlot, None),
-            synthetic_write(StepKind::UnitWrite, None),
-            synthetic_write(StepKind::JournalRecord, None),
-            synthetic_write(StepKind::RootRecordFua, Some((2, 10))),
-            synthetic_write(StepKind::SystemConfigurationSlot, None),
-            synthetic_write(StepKind::UnitWrite, None),
-            synthetic_write(StepKind::RootRecordFua, Some((2, 11))),
-            synthetic_write(StepKind::SystemConfigurationSlot, None),
-            synthetic_write(StepKind::UnitWrite, None),
-            synthetic_write(StepKind::JournalRecord, None),
-            synthetic_write(StepKind::RootRecordFua, Some((2, 12))),
-            synthetic_write(StepKind::SystemConfigurationSlot, None),
-            synthetic_write(StepKind::SystemConfigurationSlot, None),
-            synthetic_write(StepKind::UnitWrite, None),
-            synthetic_write(StepKind::RootRecordFua, Some((2, 13))),
-            synthetic_write(StepKind::SystemConfigurationSlot, None),
-            synthetic_write(StepKind::UnitWrite, None),
-            synthetic_write(StepKind::JournalRecord, None),
-            synthetic_write(StepKind::RootRecordFua, Some((2, 14))),
-            synthetic_write(StepKind::SystemConfigurationSlot, None),
-        ];
-        let floor_raises = FloorRaisesOfTheMount {
-            pushed_by_the_admission: Vec::new(),
-            continuation_steps: vec![publish_of(4, 2, 12), publish_of(4, 2, 14)],
-            pushed_by_the_continuation: vec![FloorRaiseInTheMountWrites {
-                system_configuration_write_calls: 1,
-                publishes: vec![publish_of(3, 2, 13)],
-            }],
-        };
-        let located = floor_raises_located_in_the_mount_writes(&floor_raises, &writes);
-        assert_eq!(
-            located,
-            FloorRaisesInTheMountWrites::FirstWriteAt(12),
-            "续做推的那一串从它把新 F 写进系统配置的那个写（第 12 个）起，后面还有续做第二步的 4 个写"
-        );
-        let mut persisted_through_the_first_continuation_step = vec![false; writes.len()];
-        persisted_through_the_first_continuation_step[..12].fill(true);
-        assert_eq!(
-            located.at_a_second_crash(&persisted_through_the_first_continuation_step),
-            FloorRaisesAtTheSecondCrash::NoneLandedYet,
-            "崩在续做第一步之后、抬 F 那一串之前：F 一步都还没落"
-        );
-        let mut persisted_into_the_floor_raise = persisted_through_the_first_continuation_step;
-        persisted_into_the_floor_raise[12] = true;
-        assert_eq!(
-            located.at_a_second_crash(&persisted_into_the_floor_raise),
-            FloorRaisesAtTheSecondCrash::SomeMayHaveLanded,
-            "崩在续做推的那一串里：新 F 可能落了一部分，F 不照「没推」判"
-        );
-    }
-
-    /// 挂载准入推了抬 F、之后又有续做：准入推的那几串不在写表末尾（后面接着续做的各步），界要把续做的写也算进去。
-    /// 造的写表：取号；写行（txg 10）；暖机（txg 11）；准入推的一串抬 F（系统配置写、空发布 txg 12）；续做一步（txg 13）。
-    /// 判别力：按「抬 F 那几串在写表末尾」取界时界落在续做那一步的单元写上，交「界取不出」，这一次挂载的二次崩溃整段 F 不判。
-    #[test]
-    fn floor_raises_pushed_by_the_mount_admission_are_located_before_the_continuation_that_follows_them(
-    ) {
-        let writes = vec![
-            synthetic_write(StepKind::SystemConfigurationSlot, None),
-            synthetic_write(StepKind::UnitWrite, None),
-            synthetic_write(StepKind::JournalRecord, None),
-            synthetic_write(StepKind::RootRecordFua, Some((2, 10))),
-            synthetic_write(StepKind::SystemConfigurationSlot, None),
-            synthetic_write(StepKind::UnitWrite, None),
-            synthetic_write(StepKind::RootRecordFua, Some((2, 11))),
-            synthetic_write(StepKind::SystemConfigurationSlot, None),
-            synthetic_write(StepKind::SystemConfigurationSlot, None),
-            synthetic_write(StepKind::UnitWrite, None),
-            synthetic_write(StepKind::RootRecordFua, Some((2, 12))),
-            synthetic_write(StepKind::SystemConfigurationSlot, None),
-            synthetic_write(StepKind::UnitWrite, None),
-            synthetic_write(StepKind::JournalRecord, None),
-            synthetic_write(StepKind::RootRecordFua, Some((2, 13))),
-            synthetic_write(StepKind::SystemConfigurationSlot, None),
-        ];
-        let floor_raises = FloorRaisesOfTheMount {
-            pushed_by_the_admission: vec![FloorRaiseInTheMountWrites {
-                system_configuration_write_calls: 1,
-                publishes: vec![publish_of(3, 2, 12)],
-            }],
-            continuation_steps: vec![publish_of(4, 2, 13)],
-            pushed_by_the_continuation: Vec::new(),
-        };
-        assert_eq!(
-            floor_raises_located_in_the_mount_writes(&floor_raises, &writes),
-            FloorRaisesInTheMountWrites::FirstWriteAt(8),
-            "准入推的那一串从第 8 个写（新 F 写进系统配置）起，后面是它的空发布与续做那一步"
-        );
-        let nothing_pushed = FloorRaisesOfTheMount {
-            pushed_by_the_admission: Vec::new(),
-            continuation_steps: vec![publish_of(4, 2, 13)],
-            pushed_by_the_continuation: Vec::new(),
-        };
-        assert_eq!(
-            floor_raises_located_in_the_mount_writes(&nothing_pushed, &writes),
-            FloorRaisesInTheMountWrites::NotPushed,
-            "准入与续做都没推：没推"
-        );
-    }
-
     fn writes_of_kinds(kinds: &[WrittenStructureKind]) -> WritesByStructureKind {
         let mut writes = WritesByStructureKind::NOTHING_WRITTEN;
         for kind in kinds {
@@ -4085,31 +3872,172 @@ mod tests {
         );
     }
 
-    /// I-8.4 的 R-换 在真的二次崩溃上不误红：从第一个文件起的历史（起点就是攻方 `research/prompts/m3-inv-a-r1-opus-output.md`
-    /// 一·2 的「新池新建文件做完的镜像」），起点那一段也摆崩溃状态；每个二次崩溃镜像上都再挂了一次，取号、写行、暖机三段
-    /// 各有判过的，读出的内容都与没崩过的那一次挂载之后读出的相同（攻方量到实例代号、实例表行、扇区都不同的那种也算相同）。
+    /// V1（`research/prompts/m3-unlink-code-r2-main-verification.md` 第三节）：挂载交回 `MountError::FloorRaiseFailedAfterTheMountsPublishes`
+    /// （续做里推的抬 F 自己报了不是空间不够的错）时，按它的 `writes_of_persisted_publishes` 划续做段，与 `MountError::Publish` 那一臂同一个划法：
+    /// 写账照 U12 那一条的种类造（写行、暖机、续做那一步），续做从第 2 次发布起；只有写行与暖机的（准入推的抬 F 报错，今天就交这个成员）没有续做。
+    /// 判别力：这个成员归「别的错」交 None 时续做的写全归暖机段。
     #[test]
-    fn the_remount_after_each_second_crash_reads_the_same_content_as_the_uninterrupted_mount_in_every_mount_phase(
+    fn a_mount_whose_floor_raise_failed_after_a_continuation_step_marks_the_continuation_by_the_persisted_publishes(
     ) {
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
+        let floor_raise_failed_after =
+            |writes_of_persisted_publishes: Vec<WritesByStructureKind>| {
+                MountError::FloorRaiseFailedAfterTheMountsPublishes(Box::new(
+                    FloorRaiseFailedAfterTheMountsPublishes {
+                        cause: MountError::InstanceTableMalformed,
+                        writes_of_persisted_publishes,
+                        floor_raises: Vec::new(),
+                    },
+                ))
+            };
+        assert_eq!(
+            publishes_before_the_continuation_of_a_mount_that_failed(&floor_raise_failed_after(
+                vec![row_publish.clone(), warm_up.clone(), continuation_step]
+            )),
+            Some(2),
+            "写行、一次暖机之后的那一步续做起是续做"
+        );
+        assert_eq!(
+            publishes_before_the_continuation_of_a_mount_that_failed(&floor_raise_failed_after(
+                vec![row_publish, warm_up]
+            )),
+            None,
+            "准入推的抬 F 报错（只落了写行与暖机）：没有续做，写行之后都算暖机"
+        );
+    }
+
+    /// D8（核心索引结构） 已定项 16 第 5 条：删完那一版上只许建 inode、写行与空发布。第一个文件之后经挂着的会话 unlink 它跑完（意图那次 + 最后一批），
+    /// 整份盘面当崩溃镜像：可写挂载交回的现行那一版是有树而文件集合为空的一版，之后那一次发布建一个 size 0 的 inode、做成。
+    /// 判别力：那一版上照旧调覆盖写时被拒成 `WriteToAnInodeThatIsNotInTheTree`、结局是「发布拒了」。
+    #[test]
+    fn the_publish_after_a_writable_mount_on_the_emptied_version_creates_an_inode_and_is_published()
+    {
+        use singlefs_harness::history::{HistoryDeviceWidth, PerStepChecker};
+        let width = HistoryDeviceWidth::UnitAreaOf384Slots;
+        let execution = HistoryExecution {
+            per_step_checker: PerStepChecker::Skipped,
+            device_width: width,
+            space_admission: SpaceAdmission::JudgedByTheFormula,
+        };
         let history = GeneratedHistory {
-            seed: HistorySeed(3),
+            seed: HistorySeed(0),
             starting_point: HistoryStartingPoint::AfterFirstFile,
-            operations: generate_history(HistorySeed(3), 10).operations,
+            operations: vec![HistoryOperation::Unlink],
         };
-        let injection = inject_crashes_into_history(
-            &history,
-            HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
-            CrashPointDraw::EveryProperSubsetOfShortSegments {
-                segment_writes: 2,
-                sampled_in_longer_segments: 2,
-            },
-            None,
+        let stream = SharedStream::retaining_contents();
+        let run = execute_history_with(&history, execution, &stream, &mut |_observation| {});
+        assert_eq!(
+            run.outcomes.len(),
+            history.operations.len(),
+            "unlink 那一步要跑完：{:?}",
+            run.ending
         );
-        let tally = &injection.tally;
+        let mut crash_image = MemoryPool::with_devices(
+            &[DeviceIdentity(0), DeviceIdentity(1)],
+            width.device_bytes(),
+        );
+        crash_image.apply(&stream.retained_operations());
         assert!(
-            injection
-                .crash_points
+            matches!(
+                recover(&crash_image, JournalPolicy::Consult).outcome,
+                RecoveryOutcome::NoFile { .. }
+            ),
+            "前提：unlink 跑完之后的盘面上只读恢复读出「没有文件」"
+        );
+        let crash_point = CrashPoint {
+            segment_index: 0,
+            persisted_within_the_segment: Vec::new(),
+            step: StepPosition::StartingPoint,
+            operation_kind: None,
+        };
+        let mount_run = mount_and_publish_once_on_the_crash_image(
+            &crash_image,
+            &width.parameters(),
+            execution,
+            HistorySeed(0),
+            &crash_point,
+        );
+        assert!(
+            matches!(mount_run.outcome, WritableMountOutcome::Published),
+            "删完那一版上挂载之后那一次发布建 inode、做成：{:?}",
+            mount_run.outcome
+        );
+    }
+
+    /// 挂载每一段都摆得到二次崩溃的两段历史（U4，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：从第一个文件起的
+    /// 随机一段（起点就是攻方 `research/prompts/m3-inv-a-r1-opus-output.md` 一·2 的「新池新建文件做完的镜像」），与第一个文件之后经挂着的会话
+    /// unlink 它的一段（意图那次 + 最后一批：崩溃状态有落在意图那一版上的，那一版上的可写挂载续做、写出续做那一段，
+    /// 之后那一次发布落在删完那一版上）。两段同一种摆法（短段整段枚举），计数并在一起。
+    fn crash_injections_into_histories_reaching_every_mount_phase(
+    ) -> (CrashInjectionTally, Vec<HistoryCrashInjection>) {
+        let histories = [
+            GeneratedHistory {
+                seed: HistorySeed(3),
+                starting_point: HistoryStartingPoint::AfterFirstFile,
+                operations: generate_history(HistorySeed(3), 10).operations,
+            },
+            GeneratedHistory {
+                seed: HistorySeed(3),
+                starting_point: HistoryStartingPoint::AfterFirstFile,
+                operations: vec![HistoryOperation::Unlink],
+            },
+        ];
+        let injections: Vec<HistoryCrashInjection> = histories
+            .iter()
+            .map(|history| {
+                inject_crashes_into_history(
+                    history,
+                    HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+                    CrashPointDraw::EveryProperSubsetOfShortSegments {
+                        segment_writes: 2,
+                        sampled_in_longer_segments: 2,
+                    },
+                    None,
+                )
+            })
+            .collect();
+        let mut tally = CrashInjectionTally::default();
+        for injection in &injections {
+            tally.absorb(&injection.tally);
+        }
+        (tally, injections)
+    }
+
+    /// I-8.4 的 R-换 在真的二次崩溃上不误红：两段历史（[`crash_injections_into_histories_reaching_every_mount_phase`]），
+    /// 起点那一段也摆崩溃状态；每个二次崩溃镜像上都再挂了一次，取号、写行、暖机、续做四段各有判过的，
+    /// 读出的内容都与没崩过的那一次挂载之后读出的相同（攻方量到实例代号、实例表行、扇区都不同的那种也算相同）。
+    #[test]
+    fn the_remount_after_each_second_crash_reads_the_same_content_as_the_uninterrupted_mount_in_every_mount_phase(
+    ) {
+        let (tally, injections) = crash_injections_into_histories_reaching_every_mount_phase();
+        let tally = &tally;
+        assert!(
+            injections
                 .iter()
+                .flat_map(|injection| injection.crash_points.iter())
                 .any(|crash_point| crash_point.step == StepPosition::StartingPoint),
             "起点那一段（新池新建文件做完）里要有崩溃状态"
         );
@@ -4118,14 +4046,6 @@ mod tests {
             "每个二次崩溃镜像上都再挂过一次：{tally:?}"
         );
         for phase in WritableMountPhase::ALL {
-            match phase {
-                WritableMountPhase::Acquisition
-                | WritableMountPhase::RowPublish
-                | WritableMountPhase::WarmUp => {}
-                // 续做那一段只在所选那一版有删除中的文件时才有写：这段历史里没有走到它，分段与摆点由
-                // `the_writes_after_the_publishes_before_the_continuation_belong_to_the_continuation_phase_and_get_a_second_crash_point` 钉。
-                WritableMountPhase::Continuation => continue,
-            }
             assert!(
                 tally
                     .remount_idempotence_judged_by_phase
@@ -4143,9 +4063,9 @@ mod tests {
             tally.remounts_after_a_second_crash_reading_different_content,
             0,
             "二次崩溃之后再挂读出的内容与没崩过的那一次不同（I-8.4 的 R-换 判红）：{:?}",
-            injection
-                .new_findings
+            injections
                 .iter()
+                .flat_map(|injection| injection.new_findings.iter())
                 .filter(|finding| matches!(
                     finding.stage,
                     CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(_)
@@ -4162,26 +4082,14 @@ mod tests {
         );
     }
 
-    /// 挂载透明在崩溃注入里真的判了（r5 判决第二节 U5、U6）：与上一条同一段历史、同一种摆法，每个挂载做成的崩溃状态都判了只挂载那一个，
-    /// 挂载与之后那次发布都做成的都判了挂载加一次发布那一个，二次崩溃取号、写行、暖机三段各有判过的；一样都没红。
+    /// 挂载透明在崩溃注入里真的判了（r5 判决第二节 U5、U6）：与上一条同样两段历史、同一种摆法，每个挂载做成的崩溃状态都判了只挂载那一个，
+    /// 挂载与之后那次发布都做成的都判了挂载加一次发布那一个（删完那一版上那次发布建 inode），二次崩溃取号、写行、暖机、续做四段各有判过的；
+    /// 一样都没红（挂载续做过时用户可见树根指针按续做那几步比、F 认续做推的抬 F：用户 2026-10-03 定「续做算挂载的一部分」）。
     #[test]
     fn mount_transparency_is_judged_after_every_writable_mount_and_inside_every_mount_phase_without_a_red_field(
     ) {
-        let history = GeneratedHistory {
-            seed: HistorySeed(3),
-            starting_point: HistoryStartingPoint::AfterFirstFile,
-            operations: generate_history(HistorySeed(3), 10).operations,
-        };
-        let injection = inject_crashes_into_history(
-            &history,
-            HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
-            CrashPointDraw::EveryProperSubsetOfShortSegments {
-                segment_writes: 2,
-                sampled_in_longer_segments: 2,
-            },
-            None,
-        );
-        let tally = &injection.tally;
+        let (tally, injections) = crash_injections_into_histories_reaching_every_mount_phase();
+        let tally = &tally;
         assert_eq!(
             tally.recoveries_failed, 0,
             "前提：崩溃镜像上的只读恢复都落到了一版：{tally:?}"
@@ -4211,13 +4119,6 @@ mod tests {
             tally.mount_transparency_not_judged
         );
         for phase in WritableMountPhase::ALL {
-            match phase {
-                WritableMountPhase::Acquisition
-                | WritableMountPhase::RowPublish
-                | WritableMountPhase::WarmUp => {}
-                // 同上一条用例：这段历史里没有续做那一段的写。
-                WritableMountPhase::Continuation => continue,
-            }
             assert!(
                 tally
                     .mount_transparency_judged_on_second_crashes_by_phase
@@ -4234,9 +4135,9 @@ mod tests {
             tally.mount_transparency_violations_by_field.is_empty(),
             "挂载透明判红的字段 {:?}：{}",
             tally.mount_transparency_violations_by_field,
-            injection
-                .new_findings
+            injections
                 .iter()
+                .flat_map(|injection| injection.new_findings.iter())
                 .map(CrashPointFinding::render)
                 .collect::<String>()
         );
diff --git a/crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs b/crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs
index 4955486..7aeb5c8 100644
--- a/crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs
+++ b/crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs
@@ -1,16 +1,17 @@
 //! checker 档模块：crash_injection
 //! 代码审阅第 2 条（崩溃注入层补可写挂载）：崩溃注入在每个崩溃状态上，只读恢复与判定照旧之后，
 //! 在同一份崩溃后镜像上真的起一次可写挂载（取号、写行、暖机）、再发一次布、跑池级 checker；挂载途中再崩一次（二次崩溃），
-//! 取号、写行、暖机三段各摆一个，每个二次崩溃状态上只读恢复、问模型、池级 checker、记录核对器。
+//! 取号、写行、暖机、续做各段各摆一个，每个二次崩溃状态上只读恢复、问模型、池级 checker、记录核对器。
 //! 实现在 `singlefs_checker_tier::crash_injection`；随机崩溃注入的快档与大档（`crash_injection_campaign.rs`）
-//! 走的是同一个入口，这里用两段短历史钉「每一截都跑到了、三段都摆到了、一条新发现都没有」。
+//! 走的是同一个入口，这里用两段短历史与一段 unlink 的历史钉「每一截都跑到了、四段都摆到了、一条新发现都没有」。
 
 use singlefs_checker_tier::crash_injection::{
     inject_crashes_into_history, CrashInjectionTally, CrashPointDraw, WritableMountPhase,
 };
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_harness::history::{
-    generate_history, HistoryDeviceWidth, HistoryExecution, HistorySeed, PerStepChecker,
+    generate_history, GeneratedHistory, HistoryDeviceWidth, HistoryExecution, HistoryOperation,
+    HistorySeed, HistoryStartingPoint, PerStepChecker,
 };
 
 /// 历史本身不跑每一步的 checker（活盘面由随机历史那一路罩着），两块 4 GiB 的盘：单元区墙在这里够不着，挂载与之后那一次发布都该成。
@@ -20,8 +21,10 @@ const UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
     space_admission: SpaceAdmission::JudgedByTheFormula,
 };
 
-/// 两段短历史、每段摆两个崩溃状态：每个崩溃状态之后都起了可写挂载、发了一次布、跑了 checker，
-/// 挂载途中的二次崩溃取号、写行、暖机三段都摆到了，二次崩溃状态上恢复、问模型、checker、记录核对器各跑了一次；
+/// 两段短历史、每段摆两个崩溃状态，再加第一个文件之后经挂着的会话 unlink 它的一段（意图那次 + 最后一批，短段整段枚举：
+/// 崩溃状态有落在意图那一版上的，那一版上的可写挂载续做，U4，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：
+/// 每个崩溃状态之后都起了可写挂载、发了一次布（删完那一版上建 inode）、跑了 checker，
+/// 挂载途中的二次崩溃取号、写行、暖机、续做四段都摆到了，二次崩溃状态上恢复、问模型、checker、记录核对器各跑了一次；
 /// 两截上一条新发现都没有，挂载与那次发布一次都没被拒。
 #[test]
 fn every_crash_state_is_followed_by_a_writable_mount_one_publish_the_checker_and_second_crashes_in_each_phase(
@@ -40,11 +43,26 @@ fn every_crash_state_is_followed_by_a_writable_mount_one_publish_the_checker_and
         tally.absorb(&injection.tally);
         new_findings.extend(injection.new_findings);
     }
+    let unlinking_the_first_file = inject_crashes_into_history(
+        &GeneratedHistory {
+            seed: HistorySeed(3),
+            starting_point: HistoryStartingPoint::AfterFirstFile,
+            operations: vec![HistoryOperation::Unlink],
+        },
+        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+        CrashPointDraw::EveryProperSubsetOfShortSegments {
+            segment_writes: 2,
+            sampled_in_longer_segments: 2,
+        },
+        None,
+    );
+    tally.absorb(&unlinking_the_first_file.tally);
+    new_findings.extend(unlinking_the_first_file.new_findings);
     let rendered = tally.render();
     println!("{rendered}");
     assert!(
         tally.crash_points >= 2,
-        "两段历史里摆出了崩溃状态：{rendered}"
+        "三段历史里摆出了崩溃状态：{rendered}"
     );
     assert_eq!(
         tally.writable_mounts_after_the_crash, tally.crash_points,
@@ -63,14 +81,6 @@ fn every_crash_state_is_followed_by_a_writable_mount_one_publish_the_checker_and
         "每次挂载与发布之后都跑一次 checker：{rendered}"
     );
     for phase in WritableMountPhase::ALL {
-        match phase {
-            WritableMountPhase::Acquisition
-            | WritableMountPhase::RowPublish
-            | WritableMountPhase::WarmUp => {}
-            // 续做那一段只在所选那一版有删除中的文件时才有写：这两段短历史里没有 unlink，那一段的分段与摆点由
-            // `crash_injection` 的库单测钉（Q-J，三·2 第 8 条）。
-            WritableMountPhase::Continuation => continue,
-        }
         assert!(
             tally
                 .second_crash_points_by_phase
diff --git a/crates/singlefs-harness/src/mount_transparency.rs b/crates/singlefs-harness/src/mount_transparency.rs
index 7c1a6fc..f0c128c 100644
--- a/crates/singlefs-harness/src/mount_transparency.rs
+++ b/crates/singlefs-harness/src/mount_transparency.rs
@@ -1,7 +1,8 @@
 //! 挂载透明（独立的第四条，不是 M3 的第三读：`research/prompts/m3-formal-r4-main-verification.md` 第四节第 3 条；
 //! 逐字段表与二次崩溃、拒绝合法照 `research/prompts/m3-formal-r5-main-verification.md` 第二节 U5、U6 两行与第四节第 5、6 条，零轮）的判法。
 //!
-//! 射程：不带故障、可写挂载做成的崩溃状态。崩溃镜像上只读恢复一次（R1），在它上面做一个动作——只挂载（取号、写行、暖机与挂载自己推的抬 F），
+//! 射程：不带故障、可写挂载做成的崩溃状态。崩溃镜像上只读恢复一次（R1），在它上面做一个动作——只挂载（取号、写行、暖机与挂载自己推的抬 F，
+//! 连同挂载续做与续做里推的抬 F：用户 2026-10-03 定「续做算挂载的一部分」），
 //! 或挂载加一次发布——之后再只读恢复一次（R2），逐字段比（[`judge_mount_transparency`]）；挂载途中各相位的二次崩溃镜像上，
 //! 只读恢复读出的与 R1 比（用户 2026-10-02 定 F6「加二次」）；抬 F 的写有持久的那几态 F 不判（[`FloorRaisesInTheMountWrites`]；
 //! 抬 F 的界在挂载写表里对不上位置的那一次挂载，二次崩溃整段 F 不判、按 [`FieldNotJudged`] 计数）。
@@ -17,7 +18,8 @@ use std::collections::BTreeMap;
 use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
 use singlefs_core::instance_table::InstanceRow;
 use singlefs_core::journal::JournalRecordPlaceInPublish;
-use singlefs_core::mount::{MountError, MountOutput, StillUnreadableAfterOneReread};
+use singlefs_core::mount::{MountError, MountOutput, RaisedFloor, StillUnreadableAfterOneReread};
+use singlefs_core::records::{TreeTableEntry, TREE_KIND_EXTENT, TREE_KIND_INODE};
 use singlefs_core::recovery::{
     choose_root, choose_system_configuration, effective_rollback_floor, highest_root_instance,
     highest_root_txg, instance_table_of_root, readable_roots, recover, replay_journal,
@@ -27,6 +29,7 @@ use singlefs_core::recovery::{
 };
 use singlefs_core::root_record::{RootRecord, UnmountMarker};
 use singlefs_core::system_configuration::SystemImmutableConfiguration;
+use singlefs_core::transaction::TransactionOutput;
 use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
 
 use crate::memory_pool::{root_identity_written_by, RetainedWrite};
@@ -226,7 +229,8 @@ fn version_fields(
     }
 }
 
-/// 挂载交回值里挂载透明要的那一样：挂载在写行与暖机之后推没推抬 F，推了的话最后一串抬到的 F（`MountOutput::space_admission`）。
+/// 挂载交回值里挂载透明要的那一样：挂载推没推抬 F，推了的话最后一串抬到的 F。挂载准入在写行与暖机之后推的（`MountOutput::space_admission`）
+/// 与挂载续做里为空间推的（`MountOutput::floor_raises_pushed_by_the_continuation`）都算：用户 2026-10-03 定「续做算挂载的一部分」。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum FloorRaisedByTheMount {
     NotRaised,
@@ -234,11 +238,19 @@ pub enum FloorRaisedByTheMount {
 }
 
 impl FloorRaisedByTheMount {
+    /// 最后一串是续做里推的（推了的话）：续做接在准入推的那几串之后（`MountOutput::deletion_queue_continuation` 的契约）。
+    ///
     /// # Panics
     /// 推过的一串里一次空发布都没有：抬 F 那一串推空发布直到每块盘上都有带新 F 的根，至少一次（`mount::RaisedFloor::publishes`）。
     #[must_use]
     pub fn of_the_mount_output(output: &MountOutput) -> Self {
-        match output.space_admission.floor_raises().last() {
+        match output
+            .space_admission
+            .floor_raises()
+            .iter()
+            .chain(output.floor_raises_pushed_by_the_continuation.iter())
+            .last()
+        {
             None => FloorRaisedByTheMount::NotRaised,
             Some(last_raise) => FloorRaisedByTheMount::RaisedTo(
                 last_raise
@@ -276,35 +288,188 @@ pub enum FloorRaisesInTheMountWrites {
     NotLocatedInTheMountWrites,
 }
 
+/// 挂载里一次发布交给设备的写数与它写的根身份 (实例代号, checkpoint_txg)。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct PublishInTheMountWrites {
+    pub write_calls: u64,
+    pub root: (InstanceGeneration, CheckpointTxg),
+}
+
+impl PublishInTheMountWrites {
+    #[must_use]
+    pub fn of_the_publish(publish: &TransactionOutput) -> Self {
+        PublishInTheMountWrites {
+            write_calls: publish.writes.total().write_calls,
+            root: (publish.root.instance, publish.root.checkpoint_txg),
+        }
+    }
+}
+
+/// 挂载推的一串抬 F 的写账：先把新 F 写进每块盘系统配置的那几个写（`RaisedFloor::system_configuration_writes_before_the_first_publish`），
+/// 再是各次带新 F 的空发布（按先后，至少一次：`mount::RaisedFloor::publishes` 的契约，构造时核）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct FloorRaiseInTheMountWrites {
+    system_configuration_write_calls: u64,
+    publishes: Vec<PublishInTheMountWrites>,
+}
+
+impl FloorRaiseInTheMountWrites {
+    /// # Panics
+    /// `publishes` 是空的：抬 F 那一串推空发布直到每块盘上都有带新 F 的根，至少一次（`mount::RaisedFloor::publishes` 的契约）。
+    #[must_use]
+    pub fn new(
+        system_configuration_write_calls: u64,
+        publishes: Vec<PublishInTheMountWrites>,
+    ) -> Self {
+        assert!(
+            !publishes.is_empty(),
+            "抬 F 那一串至少推一次带新 F 的空发布（RaisedFloor::publishes 的契约）"
+        );
+        FloorRaiseInTheMountWrites {
+            system_configuration_write_calls,
+            publishes,
+        }
+    }
+
+    #[must_use]
+    pub fn of_the_raise(raise: &RaisedFloor) -> Self {
+        FloorRaiseInTheMountWrites::new(
+            raise
+                .system_configuration_writes_before_the_first_publish
+                .total()
+                .write_calls,
+            raise
+                .publishes
+                .iter()
+                .map(PublishInTheMountWrites::of_the_publish)
+                .collect(),
+        )
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
+    fn checkpoint_txg_of_the_first_publish(&self) -> CheckpointTxg {
+        self.publishes
+            .first()
+            .expect("构造时核过：抬 F 那一串至少一次空发布")
+            .root
+            .1
+    }
+}
+
+/// 一次可写挂载交回的、取抬 F 那条界要用的三样：挂载准入在写行与暖机之后推的那几串（`MountOutput::space_admission`）、
+/// 续做发成的每一步（`MountOutput::deletion_queue_continuation`）与续做里为空间推的那几串（`MountOutput::floor_raises_pushed_by_the_continuation`，
+/// 与续做的各步按 txg 交错）。挂载写表的末尾依次是：准入推的那几串、续做的各步与续做推的那几串。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct FloorRaisesOfTheMount {
+    pub pushed_by_the_admission: Vec<FloorRaiseInTheMountWrites>,
+    pub continuation_steps: Vec<PublishInTheMountWrites>,
+    pub pushed_by_the_continuation: Vec<FloorRaiseInTheMountWrites>,
+}
+
+impl FloorRaisesOfTheMount {
+    #[must_use]
+    pub fn of_the_mount_output(output: &MountOutput) -> Self {
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
 impl FloorRaisesInTheMountWrites {
-    /// 按位置取界（代码轮第三轮判决 `research/prompts/m3-formal-code-r3-main-verification.md` 第三节第 3 条）：挂载推的抬 F 那几串接在暖机之后，
-    /// 是这次挂载最后交给设备的写（`MountOutput::space_admission`），每一串是先把新 F 写进每块盘系统配置的那几个写
-    /// （`RaisedFloor::system_configuration_writes_before_the_first_publish`）再加各次空发布自己的写（`TransactionOutput::writes`）；
-    /// 界 = 写表长度减去这几串的写数。不按写进系统配置的 F 认：系统配置里写的是抬到的 F 与盘上已生效的 F 的大者
-    /// （`singlefs_core::transaction` 写系统配置槽那一处），盘上已生效的 F 更高时不等于根里的 F，挂载里更早的系统配置写也可能恰带同一个 F。
-    /// 取出的界再核两样：界上那个写是系统配置槽写，每一次抬 F 发布的根槽写（按实例与 txg 认）都在界之后；对不上的交
-    /// [`FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites`]，不 panic。
+    /// 按位置取界（代码轮第三轮判决 `research/prompts/m3-formal-code-r3-main-verification.md` 第三节第 3 条）：挂载准入推的与续做里推的
+    /// 抬 F 那几串都认（[`FloorRaisesOfTheMount`]），界 = 写表长度减去从第一串抬 F 起到写表末尾的写数（每一串是先把新 F 写进每块盘系统配置的
+    /// 那几个写再加各次空发布自己的写，续做的各步按 txg 落在第一串之后的那几步也在这一截里）。崩溃注入与 harness 档的用例取界都经这一份。
     #[must_use]
     pub fn of_the_mount_output(output: &MountOutput, mount_writes: &[RetainedWrite]) -> Self {
-        let floor_raises = output.space_admission.floor_raises();
-        if floor_raises.is_empty() {
+        FloorRaisesInTheMountWrites::of_the_floor_raises_of_the_mount(
+            &FloorRaisesOfTheMount::of_the_mount_output(output),
+            mount_writes,
+        )
+    }
+
+    /// 不按写进系统配置的 F 认：系统配置里写的是抬到的 F 与盘上已生效的 F 的大者（`singlefs_core::transaction` 写系统配置槽那一处），
+    /// 盘上已生效的 F 更高时不等于根里的 F，挂载里更早的系统配置写也可能恰带同一个 F。
+    /// 取出的界再核两样：界上那个写是系统配置槽写（每一串先把新 F 写进系统配置），每一次抬 F 发布的根槽写（按实例与 txg 认）都在界之后；
+    /// 对不上的交 [`FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites`]，不 panic。
+    #[must_use]
+    pub fn of_the_floor_raises_of_the_mount(
+        floor_raises: &FloorRaisesOfTheMount,
+        mount_writes: &[RetainedWrite],
+    ) -> Self {
+        let Some((writes_from_the_first_floor_raise_on, floor_raise_roots)) =
+            floor_raises.writes_from_the_first_floor_raise_on()
+        else {
             return FloorRaisesInTheMountWrites::NotPushed;
-        }
-        let writes_of_the_floor_raises: u64 = floor_raises
-            .iter()
-            .map(|raise| {
-                raise
-                    .system_configuration_writes_before_the_first_publish
-                    .total()
-                    .write_calls
-                    + raise
-                        .publishes
-                        .iter()
-                        .map(|publish| publish.writes.total().write_calls)
-                        .sum::<u64>()
-            })
-            .sum();
-        let Some(first_write) = usize::try_from(writes_of_the_floor_raises)
+        };
+        let Some(first_write) = usize::try_from(writes_from_the_first_floor_raise_on)
             .ok()
             .and_then(|writes| mount_writes.len().checked_sub(writes))
         else {
@@ -319,13 +484,9 @@ impl FloorRaisesInTheMountWrites {
                 .filter(|write| write.kind == StepKind::RootRecordFua)
                 .filter_map(|write| write.bytes().map(root_identity_written_by))
                 .collect();
-        let every_floor_raise_root_is_written_from_the_boundary = floor_raises
+        let every_floor_raise_root_is_written_from_the_boundary = floor_raise_roots
             .iter()
-            .flat_map(|raise| raise.publishes.iter())
-            .all(|publish| {
-                roots_written_from_the_boundary
-                    .contains(&(publish.root.instance, publish.root.checkpoint_txg))
-            });
+            .all(|root| roots_written_from_the_boundary.contains(root));
         if the_boundary_is_a_system_configuration_slot_write
             && every_floor_raise_root_is_written_from_the_boundary
         {
@@ -361,6 +522,73 @@ impl FloorRaisesInTheMountWrites {
     }
 }
 
+/// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）每一步那一版树表里两棵用户可见树的根指针（用户 2026-10-03 定「续做算挂载的一部分」）：
+/// 挂载写的版本里，续做之前的那几版（写行、暖机、准入推的抬 F）照抄 R1 那一版的两棵树；续做的每一步改了它们，之后挂载写的版本
+/// （续做推的抬 F 的空发布）照抄最近一步的。挂载没续做时一步都没有，挂载写的每一版都比 R1 那一版。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct UserVisibleTreeRootPointersOfTheContinuation {
+    /// (那一步的实例代号与 checkpoint_txg, 两根指针)，按 txg 升序（构造时排）。
+    steps: Vec<(
+        (InstanceGeneration, CheckpointTxg),
+        UserVisibleTreeRootPointers,
+    )>,
+}
+
+impl UserVisibleTreeRootPointersOfTheContinuation {
+    /// 挂载没续做（`MountOutput::deletion_queue_continuation` 空）。
+    pub const NOT_CONTINUED: Self = Self { steps: Vec::new() };
+
+    /// 两根指针取续做每一步交回的树表条目（`TransactionOutput::tree_table_entries`）里那 86 字节，与只读恢复读盘上树表的取法
+    /// （`recovery::user_visible_tree_root_pointers`）同一段字节。
+    #[must_use]
+    pub fn of_the_mount_output(output: &MountOutput) -> Self {
+        let mut steps: Vec<(
+            (InstanceGeneration, CheckpointTxg),
+            UserVisibleTreeRootPointers,
+        )> = output
+            .deletion_queue_continuation
+            .iter()
+            .map(|step| {
+                (
+                    (step.root.instance, step.root.checkpoint_txg),
+                    user_visible_tree_root_pointers_of_a_published_version(step),
+                )
+            })
+            .collect();
+        steps.sort_by_key(|(version, _)| (version.1, version.0));
+        UserVisibleTreeRootPointersOfTheContinuation { steps }
+    }
+
+    /// 挂载写的 `version` 那一版该带的两根指针：同一个实例里 txg 不大于它的最近一步续做那一版的；它在续做之前（或挂载没续做）时 None，
+    /// 调用方比 R1 那一版。
+    fn of_the_last_step_at_or_before(
+        &self,
+        version: (InstanceGeneration, CheckpointTxg),
+    ) -> Option<&UserVisibleTreeRootPointers> {
+        self.steps
+            .iter()
+            .filter(|(step, _)| step.0 == version.0 && step.1 <= version.1)
+            .map(|(_, pointers)| pointers)
+            .next_back()
+    }
+}
+
+/// 一次发布交回的树表条目里 inode 树、extent 树两条的根指针盘上字节；树表里没有那棵树的条目时 None。
+fn user_visible_tree_root_pointers_of_a_published_version(
+    version: &TransactionOutput,
+) -> UserVisibleTreeRootPointers {
+    let mut pointers = UserVisibleTreeRootPointers::ABSENT;
+    for entry in &version.tree_table_entries {
+        let root_pointer = Some(TreeTableEntry::root_pointer_bytes(&entry.to_bytes()).to_vec());
+        if entry.kind == TREE_KIND_INODE {
+            pointers.inode_tree = root_pointer;
+        } else if entry.kind == TREE_KIND_EXTENT {
+            pointers.extent_tree = root_pointer;
+        }
+    }
+    pointers
+}
+
 /// 做的是哪个动作。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum MountTransparencyAction {
@@ -731,8 +959,9 @@ fn rows_sorted_by_instance(rows: &[InstanceRow]) -> Vec<InstanceRow> {
 /// 动作做成之后的那一版：POSIX 内容与存在性相同；不可变配置（除本盘设备号）相同；卸载记号恒为「不是卸载写的」、R1 的记号不进比较；
 /// 挂载没推抬 F 时 R2 的 F 生效值与 R2 根里的 F 都 = R1 的 F 生效值，推了时 R2 的 F 生效值 = 挂载交回的最后一串抬到的 F；
 /// 实例代号 = max(R1 镜像上各盘两槽自证系统配置的实例代号, 根环全部可读根的实例代号) + 1；实例表行照 D18 已定项 11「恢复写的行」；
-/// 树 ID 水位相同（挂载不发第一个文件）；两棵用户可见树的根指针相同；txg ≥ max(根环全部根的 txg, 环里全部自证记录的 checkpoint_txg) + 1
-/// （只判下界：kb 允许跳号）；扫描报告施加 0 条、水位之上 0 条。
+/// 树 ID 水位相同（挂载不发第一个文件）；两棵用户可见树的根指针相同——挂载续做过时（用户 2026-10-03 定「续做算挂载的一部分」）
+/// 比挂载交回的续做那几步里 txg 不大于 R2 那一版的最近一步的（`continuation`，[`UserVisibleTreeRootPointersOfTheContinuation`]）；
+/// txg ≥ max(根环全部根的 txg, 环里全部自证记录的 checkpoint_txg) + 1（只判下界：kb 允许跳号）；扫描报告施加 0 条、水位之上 0 条。
 /// 二次崩溃之后落回 v1 那一版自己（实例与 txg 都是 R1 那一版的）：各样与 R1 那一版相同（推的）；落到这次挂载写的某一版：照上一段，
 /// 扫描报告不判，崩在抬 F 开始之后的 F 不判。
 ///
@@ -743,6 +972,7 @@ pub fn judge_mount_transparency(
     before_the_mount: &RecoveryFieldsRead,
     after_the_action: &RecoveryFieldsRead,
     action: MountTransparencyAction,
+    continuation: &UserVisibleTreeRootPointersOfTheContinuation,
 ) -> MountTransparency {
     assert_eq!(
         before_the_mount.compared,
@@ -835,6 +1065,7 @@ pub fn judge_mount_transparency(
                 relation,
                 new_instance,
                 action,
+                continuation,
             ),
         })
         .collect();
@@ -843,7 +1074,7 @@ pub fn judge_mount_transparency(
 
 #[allow(
     clippy::too_many_arguments,
-    reason = "每个参数各是一样东西：判哪一样、两次恢复与各自比的那一版、两版的关系、R1 推出的新实例、动作"
+    reason = "每个参数各是一样东西：判哪一样、两次恢复与各自比的那一版、两版的关系、R1 推出的新实例、动作、挂载续做那几步的两根指针"
 )]
 fn judge_one_field(
     field: MountTransparencyField,
@@ -854,6 +1085,7 @@ fn judge_one_field(
     relation: Relation,
     new_instance: Option<InstanceGeneration>,
     action: MountTransparencyAction,
+    continuation: &UserVisibleTreeRootPointersOfTheContinuation,
 ) -> FieldOutcome {
     match field {
         MountTransparencyField::PosixContent => held_or_violated(
@@ -964,13 +1196,24 @@ fn judge_one_field(
             first_recovery.root.tree_identifier_watermark.to_string(),
             compared.root.tree_identifier_watermark.to_string(),
         ),
-        MountTransparencyField::UserVisibleTreeRootPointers => held_or_violated(
-            first_recovery.user_visible_tree_root_pointers.is_some()
-                && compared.user_visible_tree_root_pointers
-                    == first_recovery.user_visible_tree_root_pointers,
-            format!("{:?}", first_recovery.user_visible_tree_root_pointers),
-            format!("{:?}", compared.user_visible_tree_root_pointers),
-        ),
+        MountTransparencyField::UserVisibleTreeRootPointers => {
+            let expected = match relation {
+                Relation::WrittenByTheMount(_) => continuation
+                    .of_the_last_step_at_or_before((
+                        compared.root.instance,
+                        compared.root.checkpoint_txg,
+                    ))
+                    .or(first_recovery.user_visible_tree_root_pointers.as_ref()),
+                Relation::TheVersionOfTheFirstRecovery(_) => {
+                    first_recovery.user_visible_tree_root_pointers.as_ref()
+                }
+            };
+            held_or_violated(
+                expected.is_some() && compared.user_visible_tree_root_pointers.as_ref() == expected,
+                format!("{expected:?}"),
+                format!("{:?}", compared.user_visible_tree_root_pointers),
+            )
+        }
         MountTransparencyField::CheckpointTxg => match relation {
             Relation::WrittenByTheMount(_) => {
                 let lower_bound = before_the_mount
diff --git a/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs b/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
index af854f0..5e2e8e4 100644
--- a/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
+++ b/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
@@ -1,19 +1,39 @@
 //! 挂载透明的逐字段判（`singlefs_harness::mount_transparency`，r5 判决第二节 U5、U6 两行）在几个小池上：每一样一条阳性（合法挂载判绿）、
 //! 一条阴性（把 R2 的那一样改坏判红，别的仍绿）。小池是单元区 384 槽（抬 F 那一条取 256 槽）的两块内存盘，历史由随机历史的执行器照抄
 //! 一段写死的操作跑出来；「崩溃镜像」取那段历史停下时的整份盘面（或一次发布扣下根槽写之后的盘面），它就是一个不带故障的崩溃状态。
+//! 挂载续做过的两形（用户 2026-10-03 定「续做算挂载的一部分」）：崩在 unlink 意图之后重挂；窄盘上续做里被空间拒、推抬 F 再走
+//! （盘面照 `admission_raises_the_floor_before_refusing.rs` 那一条的造法，搭建经 `tests/common_admission`）。
 
-use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+mod common_admission;
+
+use common_admission::{
+    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
+    PoolUnderTest, OVERWRITE_BYTES,
+};
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InodeNumber, InstanceGeneration,
+};
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::container_tree::ContainerTreeNodeEntries;
+use singlefs_core::lineage_tree::LineageContainerCapacity;
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
 use singlefs_core::mount::{
-    mount_writable_with_space_admission, unmount, MountOutput, Mounted, Unmounted,
+    mount_writable_with_space_admission,
+    mount_writable_with_test_only_switches_and_deletion_switches, unmount, MountOutput, Mounted,
+    ShadowLedger, Unmounted,
 };
 use singlefs_core::mounted_session::MountedSession;
 use singlefs_core::root_record::UnmountMarker;
 use singlefs_core::system_configuration::SystemConfiguration;
+use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
 use singlefs_core::transaction::{
-    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
+    publish_first_file, publish_overwrite, publish_unlink_intent, FirstFile, PoolVersion,
+    PoolWriter, FIRST_INODE_NUMBER,
+};
+use singlefs_core::unit::data_unit_payload_capacity;
+use singlefs_core::unlink::{
+    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
 };
 use singlefs_harness::history::{
     execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
@@ -25,9 +45,10 @@ use singlefs_harness::memory_pool::{
 };
 use singlefs_harness::mount_transparency::{
     judge_mount_transparency, read_recovery_fields, ComparedVersion, FieldNotJudged, FieldOutcome,
-    FloorRaisedByTheMount, FloorRaisesAtTheSecondCrash, FloorRaisesInTheMountWrites,
-    MountTransparency, MountTransparencyAction, MountTransparencyField, PosixContentRead,
-    RecoveryFieldsRead, VersionFieldsRead,
+    FloorRaiseInTheMountWrites, FloorRaisedByTheMount, FloorRaisesAtTheSecondCrash,
+    FloorRaisesInTheMountWrites, FloorRaisesOfTheMount, MountTransparency, MountTransparencyAction,
+    MountTransparencyField, PosixContentRead, PublishInTheMountWrites, RecoveryFieldsRead,
+    UserVisibleTreeRootPointersOfTheContinuation, VersionFieldsRead,
 };
 use singlefs_harness::segments::StepKind;
 use singlefs_harness::{RecordingBlockDevice, SharedStream};
@@ -157,33 +178,59 @@ fn publish_once_after_the_mount(
     .expect("挂载之后那一次发布该成");
 }
 
-/// 一次判下来要的三样：R1、R2 与动作。
+/// 一次判下来要的四样：R1、R2、动作与挂载续做那几步的两根指针。
 struct JudgedScenario {
     before_the_mount: RecoveryFieldsRead,
     after_the_action: RecoveryFieldsRead,
     action: MountTransparencyAction,
+    continuation: UserVisibleTreeRootPointersOfTheContinuation,
 }
 
 impl JudgedScenario {
     fn judge(&self) -> MountTransparency {
-        judge_mount_transparency(&self.before_the_mount, &self.after_the_action, self.action)
+        judge_mount_transparency(
+            &self.before_the_mount,
+            &self.after_the_action,
+            self.action,
+            &self.continuation,
+        )
     }
 }
 
-/// 在 `crash_image` 上只挂载（v2a）。
-fn mount_alone_on(width: HistoryDeviceWidth, crash_image: &MemoryPool) -> JudgedScenario {
-    let parameters = width.parameters();
+/// 在 `crash_image` 上用 `mount_writable` 只挂载（v2a），连同挂载交回值一起交回。
+fn mount_alone_with(
+    parameters: &MakeFilesystemParameters,
+    crash_image: &MemoryPool,
+    mount_writable: impl FnOnce(
+        &MakeFilesystemParameters,
+        &mut Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>,
+    ) -> Mounted,
+) -> (JudgedScenario, MountOutput) {
     let stream = SharedStream::new();
-    let mut devices = devices_of(crash_image, &parameters, &stream);
-    let mounted = mount(&parameters, &mut devices);
+    let mut devices = devices_of(crash_image, parameters, &stream);
+    let mounted = mount_writable(parameters, &mut devices);
     let floor = FloorRaisedByTheMount::of_the_mount_output(&mounted.output);
-    drop(mounted);
+    let continuation =
+        UserVisibleTreeRootPointersOfTheContinuation::of_the_mount_output(&mounted.output);
+    let output = mounted.output;
     let after = pool_of(devices, crash_image.device_size_in_bytes);
-    JudgedScenario {
-        before_the_mount: read_recovery_fields(crash_image, ComparedVersion::TheRecoveredVersion),
-        after_the_action: read_recovery_fields(&after, ComparedVersion::TheRecoveredVersion),
-        action: MountTransparencyAction::MountAlone(floor),
-    }
+    (
+        JudgedScenario {
+            before_the_mount: read_recovery_fields(
+                crash_image,
+                ComparedVersion::TheRecoveredVersion,
+            ),
+            after_the_action: read_recovery_fields(&after, ComparedVersion::TheRecoveredVersion),
+            action: MountTransparencyAction::MountAlone(floor),
+            continuation,
+        },
+        output,
+    )
+}
+
+/// 在 `crash_image` 上只挂载（v2a）。
+fn mount_alone_on(width: HistoryDeviceWidth, crash_image: &MemoryPool) -> JudgedScenario {
+    mount_alone_with(&width.parameters(), crash_image, mount).0
 }
 
 /// 在 `crash_image` 上挂载加一次发布，R2 比发布之前那一版。
@@ -193,6 +240,8 @@ fn mount_and_one_publish_on(width: HistoryDeviceWidth, crash_image: &MemoryPool)
     let mut devices = devices_of(crash_image, &parameters, &stream);
     let mounted = mount(&parameters, &mut devices);
     let floor = FloorRaisedByTheMount::of_the_mount_output(&mounted.output);
+    let continuation =
+        UserVisibleTreeRootPointersOfTheContinuation::of_the_mount_output(&mounted.output);
     publish_once_after_the_mount(&parameters, &mut devices, mounted);
     let after = pool_of(devices, crash_image.device_size_in_bytes);
     JudgedScenario {
@@ -202,6 +251,7 @@ fn mount_and_one_publish_on(width: HistoryDeviceWidth, crash_image: &MemoryPool)
             ComparedVersion::TheVersionBeforeTheRecoveredOne,
         ),
         action: MountTransparencyAction::MountAndOnePublish(floor),
+        continuation,
     }
 }
 
@@ -296,7 +346,10 @@ fn second_crash_after_the_acquisition(crash_image: &MemoryPool) -> JudgedScenari
     let parameters = SMALL.parameters();
     let stream = SharedStream::retaining_contents();
     let mut devices = devices_of(crash_image, &parameters, &stream);
-    drop(mount(&parameters, &mut devices));
+    let mounted = mount(&parameters, &mut devices);
+    let continuation =
+        UserVisibleTreeRootPointersOfTheContinuation::of_the_mount_output(&mounted.output);
+    drop(mounted);
     let operations = stream.retained_operations();
     let (writes, _segments, stream_indexes) =
         writes_and_segments_with_stream_indexes(&operations, &SMALL.fixed_geometry());
@@ -315,6 +368,7 @@ fn second_crash_after_the_acquisition(crash_image: &MemoryPool) -> JudgedScenari
         action: MountTransparencyAction::SecondCrashInsideTheMount(
             FloorRaisesAtTheSecondCrash::NoneLandedYet,
         ),
+        continuation,
     }
 }
 
@@ -341,6 +395,8 @@ fn second_crashes_on_both_sides_of_the_first_floor_raise_write(
     let (writes, _segments, stream_indexes) =
         writes_and_segments_with_stream_indexes(&operations, &width.fixed_geometry());
     let floor_raises = FloorRaisesInTheMountWrites::of_the_mount_output(&mounted.output, &writes);
+    let continuation =
+        UserVisibleTreeRootPointersOfTheContinuation::of_the_mount_output(&mounted.output);
     let FloorRaisesInTheMountWrites::FirstWriteAt(first_floor_raise_write) = floor_raises else {
         panic!("前提：这次挂载在写行与暖机之后推了抬 F：{floor_raises:?}");
     };
@@ -360,6 +416,7 @@ fn second_crashes_on_both_sides_of_the_first_floor_raise_write(
             action: MountTransparencyAction::SecondCrashInsideTheMount(
                 floor_raises.at_a_second_crash(&persisted_in_the_mount),
             ),
+            continuation: continuation.clone(),
         }
     };
     SecondCrashesOnBothSidesOfTheFirstFloorRaiseWrite {
@@ -948,3 +1005,372 @@ fn a_mount_whose_floor_raises_are_not_in_the_handed_writes_leaves_the_floor_of_e
         "合法挂载途中再崩一次，一样都不许红：{verdict:#?}"
     );
 }
+
+/// 第一个文件之后经挂着的会话 unlink 它（意图那次 + 最后一批），崩在意图那次的根槽写与紧跟着的系统配置槽写之后、最后一批的第一个写之前：
+/// 只读恢复落到意图那一版（第一个文件在删除中，读出「没有文件」），可写挂载在写行与暖机之后续做那一批（D8（核心索引结构） 已定项 16 第 6 条）。
+fn pool_crashed_right_after_the_unlink_intent() -> MemoryPool {
+    let stream = SharedStream::retaining_contents();
+    let history = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations: vec![HistoryOperation::Unlink],
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
+        "unlink 那一步要跑完：{:?}",
+        run.ending
+    );
+    let operations = stream.retained_operations();
+    let (writes, _segments, stream_indexes) =
+        writes_and_segments_with_stream_indexes(&operations, &SMALL.fixed_geometry());
+    let root_writes: Vec<usize> = writes
+        .iter()
+        .enumerate()
+        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
+        .map(|(write_index, _)| write_index)
+        .collect();
+    let [.., intent_root_write, _last_batch_root_write] = root_writes.as_slice() else {
+        panic!("前提：录制流里至少有意图与最后一批两次根槽写：{root_writes:?}");
+    };
+    let first_write_of_the_last_batch = (intent_root_write + 1..writes.len())
+        .find(|write_index| writes[*write_index].kind != StepKind::SystemConfigurationSlot)
+        .expect("意图那次之后还有最后一批的写");
+    let mut crash_image = MemoryPool::with_devices(&DEVICES, SMALL.device_bytes());
+    crash_image.apply(&operations[..stream_indexes[first_write_of_the_last_batch]]);
+    crash_image
+}
+
+/// 删除批大小 1 个数据单元（只供测试的开关，挂载的开关链装上去，续做照它）。
+fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
+    DeletionSwitches {
+        batch_size: UnlinkBatchSize::CappedForTests(
+            UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
+        ),
+        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
+        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
+        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
+    }
+}
+
+/// 窄盘上 unlink 一个 14 个数据单元的文件、发完意图就崩（单元区 256 槽、不为环窗口补足，第一个文件之后覆盖写 2 次再顺序写长到 14 个单元），
+/// 与 `admission_raises_the_floor_before_refusing.rs` 里「挂载续做里一批被落得下那一判拒、推一串抬 F 之后走完」那一条同一种造法（那边第十位实现员的
+/// 草稿探针定的 256 槽、2 次、14 个单元）。交回崩溃那一刻的盘面与建池那一份参数。
+fn pool_crashed_after_the_unlink_intent_of_fourteen_data_units_on_narrow_devices(
+) -> (MemoryPool, MakeFilesystemParameters) {
+    let mut pool =
+        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
+            HistoryDeviceWidth::UnitAreaOf384Slots,
+            256,
+        );
+    for overwrite_index in 1..=2 {
+        pool.overwrite(OVERWRITE_BYTES)
+            .unwrap_or_else(|refusal| panic!("第 {overwrite_index} 次覆盖写做成：{refusal:?}"));
+    }
+    pool.sequential_write(13 * data_unit_payload_capacity() + 100)
+        .unwrap_or_else(|refusal| panic!("顺序写把文件长到 14 个单元：{refusal:?}"));
+    let PoolUnderTest {
+        session,
+        devices,
+        parameters,
+        ..
+    } = &mut pool;
+    let session = session.as_mut().expect("这一刻有可写会话");
+    let current = session
+        .current
+        .version_of_trees()
+        .expect("第一个文件之后有树")
+        .clone();
+    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
+    publish_unlink_intent(
+        &mut writer,
+        &mut session.allocator,
+        &current,
+        InodeNumber(FIRST_INODE_NUMBER),
+        session.instance,
+    )
+    .expect("意图那次");
+    pool.session = None;
+    (pool.image(), pool.parameters.clone())
+}
+
+/// 用户 2026-10-03 定「续做算挂载的一部分」：崩在 unlink 意图之后重挂，挂载在写行与暖机之后续做最后一批、把两棵用户可见树的根改了；
+/// 只挂载之后的只读恢复落到续做那一版，两根指针比挂载交回的续做最后一步那一版的，不比 R1 那一版的，别的九样照常判绿。
+/// 判别力：不认续做那几步、照旧比 R1 那一版的时，用户可见树根指针那一样判红（第 11 批实现员 B 的探针量到的那一形）。
+#[test]
+fn after_a_mount_that_continued_the_deletion_the_two_tree_root_pointers_are_those_of_its_last_continuation_step(
+) {
+    let crash_image = pool_crashed_right_after_the_unlink_intent();
+    let (scenario, output) = mount_alone_with(&SMALL.parameters(), &crash_image, mount);
+    assert!(
+        !output.deletion_queue_continuation.is_empty(),
+        "前提：崩在意图之后重挂，挂载续做了"
+    );
+    assert_eq!(
+        scenario
+            .before_the_mount
+            .compared_version
+            .as_ref()
+            .expect("R1 落到意图那一版")
+            .content,
+        PosixContentRead::NoFile,
+        "前提：R1 落到意图那一版，第一个文件在删除中、读出「没有文件」"
+    );
+    assert_ne!(
+        scenario
+            .after_the_action
+            .compared_version
+            .as_ref()
+            .expect("R2 落到一版")
+            .user_visible_tree_root_pointers,
+        scenario
+            .before_the_mount
+            .compared_version
+            .as_ref()
+            .expect("R1 落到一版")
+            .user_visible_tree_root_pointers,
+        "前提：续做改了两棵树的根指针"
+    );
+    assert_holds(
+        &scenario.judge(),
+        MountTransparencyField::UserVisibleTreeRootPointers,
+    );
+}
+
+/// 用户 2026-10-03 定「续做算挂载的一部分」：窄盘上崩在 unlink 意图之后重挂（删除批大小 1），准入在取号之前放行、一串都没推，
+/// 续做里一批被空间拒、推一串抬 F 之后走完；只挂载之后 F 生效值是续做推的那一串抬到的 F，别的九样照常判绿。
+/// 判别力：抬到的 F 只认挂载准入推的那几串时这一形交「没推」，F 照「生效值不变」判红。
+#[test]
+fn a_mount_whose_continuation_raised_the_floor_leaves_the_effective_floor_at_the_last_one_the_continuation_raised_to(
+) {
+    let (crash_image, parameters) =
+        pool_crashed_after_the_unlink_intent_of_fourteen_data_units_on_narrow_devices();
+    let (scenario, output) = mount_alone_with(&parameters, &crash_image, |parameters, devices| {
+        mount_writable_with_test_only_switches_and_deletion_switches(
+            parameters,
+            devices,
+            SpaceAdmission::JudgedByTheFormula,
+            ShadowLedger::On,
+            deletion_switches_with_batches_of_one(),
+        )
+        .unwrap_or_else(|error| panic!("续做推过抬 F 之后走完：{error:?}"))
+    });
+    assert_eq!(
+        (
+            output.space_admission.floor_raises().len(),
+            output.floor_raises_pushed_by_the_continuation.len()
+        ),
+        (0, 1),
+        "前提：准入一串都没推，续做里推了一串"
+    );
+    assert_holds(&scenario.judge(), MountTransparencyField::RollbackFloor);
+}
+
+/// 造写表用：一个 512 字节的写；根槽写的字节里在偏移 24、28 写上根身份（`root_identity_written_by` 的读法）。
+fn synthetic_write(kind: StepKind, root: Option<(u32, u64)>) -> RetainedWrite {
+    let mut bytes = vec![0u8; 512];
+    if let Some((instance, checkpoint_txg)) = root {
+        bytes[24..28].copy_from_slice(&instance.to_le_bytes());
+        bytes[28..36].copy_from_slice(&checkpoint_txg.to_le_bytes());
+    }
+    RetainedWrite {
+        device: DeviceIdentity(0),
+        kind,
+        is_force_unit_access: kind == StepKind::RootRecordFua,
+        offset: DeviceOffsetInBytes(0),
+        contents: WrittenContents::Bytes(bytes),
+    }
+}
+
+fn publish_of(write_calls: u64, instance: u32, checkpoint_txg: u64) -> PublishInTheMountWrites {
+    PublishInTheMountWrites {
+        write_calls,
+        root: (InstanceGeneration(instance), CheckpointTxg(checkpoint_txg)),
+    }
+}
+
+/// 挂载续做里为空间推的抬 F（D8（核心索引结构） 已定项 16 第 6 条，用户 2026-10-03 定「挂载续做被空间拒：同样推抬 F 再试」）：
+/// 那一串夹在续做的两步之间、不在挂载写表末尾，界按它在写表里的位置取。造的写表：取号；写行（实例 2、txg 10）；暖机一次（txg 11）；
+/// 续做第一步（txg 12）；续做推的一串抬 F（先把新 F 写进系统配置，再一次空发布 txg 13）；续做第二步（txg 14）。
+/// 判别力：只认挂载准入推的那几串（这一份取界在第 12 批之前的做法）时这一形交「没推」，
+/// 二次崩溃崩在那一串里时 F 照「没推」判。
+#[test]
+fn a_floor_raise_pushed_inside_the_continuation_is_located_between_the_continuation_steps() {
+    let writes = vec![
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::UnitWrite, None),
+        synthetic_write(StepKind::JournalRecord, None),
+        synthetic_write(StepKind::RootRecordFua, Some((2, 10))),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::UnitWrite, None),
+        synthetic_write(StepKind::RootRecordFua, Some((2, 11))),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::UnitWrite, None),
+        synthetic_write(StepKind::JournalRecord, None),
+        synthetic_write(StepKind::RootRecordFua, Some((2, 12))),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::UnitWrite, None),
+        synthetic_write(StepKind::RootRecordFua, Some((2, 13))),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::UnitWrite, None),
+        synthetic_write(StepKind::JournalRecord, None),
+        synthetic_write(StepKind::RootRecordFua, Some((2, 14))),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+    ];
+    let floor_raises = FloorRaisesOfTheMount {
+        pushed_by_the_admission: Vec::new(),
+        continuation_steps: vec![publish_of(4, 2, 12), publish_of(4, 2, 14)],
+        pushed_by_the_continuation: vec![FloorRaiseInTheMountWrites::new(
+            1,
+            vec![publish_of(3, 2, 13)],
+        )],
+    };
+    let located =
+        FloorRaisesInTheMountWrites::of_the_floor_raises_of_the_mount(&floor_raises, &writes);
+    assert_eq!(
+        located,
+        FloorRaisesInTheMountWrites::FirstWriteAt(12),
+        "续做推的那一串从它把新 F 写进系统配置的那个写（第 12 个）起，后面还有续做第二步的 4 个写"
+    );
+    let mut persisted_through_the_first_continuation_step = vec![false; writes.len()];
+    persisted_through_the_first_continuation_step[..12].fill(true);
+    assert_eq!(
+        located.at_a_second_crash(&persisted_through_the_first_continuation_step),
+        FloorRaisesAtTheSecondCrash::NoneLandedYet,
+        "崩在续做第一步之后、抬 F 那一串之前：F 一步都还没落"
+    );
+    let mut persisted_into_the_floor_raise = persisted_through_the_first_continuation_step;
+    persisted_into_the_floor_raise[12] = true;
+    assert_eq!(
+        located.at_a_second_crash(&persisted_into_the_floor_raise),
+        FloorRaisesAtTheSecondCrash::SomeMayHaveLanded,
+        "崩在续做推的那一串里：新 F 可能落了一部分，F 不照「没推」判"
+    );
+}
+
+/// 挂载准入推了抬 F、之后又有续做：准入推的那几串不在写表末尾（后面接着续做的各步），界要把续做的写也算进去。
+/// 造的写表：取号；写行（txg 10）；暖机（txg 11）；准入推的一串抬 F（系统配置写、空发布 txg 12）；续做一步（txg 13）。
+/// 判别力：按「抬 F 那几串在写表末尾」取界时界落在续做那一步的单元写上，交「界取不出」，这一次挂载的二次崩溃整段 F 不判。
+#[test]
+fn floor_raises_pushed_by_the_mount_admission_are_located_before_the_continuation_that_follows_them(
+) {
+    let writes = vec![
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::UnitWrite, None),
+        synthetic_write(StepKind::JournalRecord, None),
+        synthetic_write(StepKind::RootRecordFua, Some((2, 10))),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::UnitWrite, None),
+        synthetic_write(StepKind::RootRecordFua, Some((2, 11))),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::UnitWrite, None),
+        synthetic_write(StepKind::RootRecordFua, Some((2, 12))),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::UnitWrite, None),
+        synthetic_write(StepKind::JournalRecord, None),
+        synthetic_write(StepKind::RootRecordFua, Some((2, 13))),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+    ];
+    let floor_raises = FloorRaisesOfTheMount {
+        pushed_by_the_admission: vec![FloorRaiseInTheMountWrites::new(
+            1,
+            vec![publish_of(3, 2, 12)],
+        )],
+        continuation_steps: vec![publish_of(4, 2, 13)],
+        pushed_by_the_continuation: Vec::new(),
+    };
+    assert_eq!(
+        FloorRaisesInTheMountWrites::of_the_floor_raises_of_the_mount(&floor_raises, &writes),
+        FloorRaisesInTheMountWrites::FirstWriteAt(8),
+        "准入推的那一串从第 8 个写（新 F 写进系统配置）起，后面是它的空发布与续做那一步"
+    );
+    let nothing_pushed = FloorRaisesOfTheMount {
+        pushed_by_the_admission: Vec::new(),
+        continuation_steps: vec![publish_of(4, 2, 13)],
+        pushed_by_the_continuation: Vec::new(),
+    };
+    assert_eq!(
+        FloorRaisesInTheMountWrites::of_the_floor_raises_of_the_mount(&nothing_pushed, &writes),
+        FloorRaisesInTheMountWrites::NotPushed,
+        "准入与续做都没推：没推"
+    );
+}
+
+/// V6（`research/prompts/m3-unlink-code-r2-main-verification.md` 第三节）：挂载准入推了一串抬 F、之后的续做里又为空间推了一串：
+/// 界从准入推的那一串起，后面的续做各步与续做推的那一串都在界之后。造的写表：取号；写行（txg 10）；暖机（txg 11）；准入推的一串
+/// （系统配置写、空发布 txg 12）；续做第一步（txg 13）；续做推的一串（系统配置写、空发布 txg 14）；续做第二步（txg 15）。
+/// 判别力：准入推过时不把续做推的那几串的写算进去，界落在续做第一步的单元写上、取不出。
+#[test]
+fn floor_raises_pushed_by_both_the_admission_and_the_continuation_are_located_from_the_first_admission_raise(
+) {
+    let writes = vec![
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::UnitWrite, None),
+        synthetic_write(StepKind::JournalRecord, None),
+        synthetic_write(StepKind::RootRecordFua, Some((2, 10))),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::UnitWrite, None),
+        synthetic_write(StepKind::RootRecordFua, Some((2, 11))),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::UnitWrite, None),
+        synthetic_write(StepKind::RootRecordFua, Some((2, 12))),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::UnitWrite, None),
+        synthetic_write(StepKind::JournalRecord, None),
+        synthetic_write(StepKind::RootRecordFua, Some((2, 13))),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::UnitWrite, None),
+        synthetic_write(StepKind::RootRecordFua, Some((2, 14))),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+        synthetic_write(StepKind::UnitWrite, None),
+        synthetic_write(StepKind::JournalRecord, None),
+        synthetic_write(StepKind::RootRecordFua, Some((2, 15))),
+        synthetic_write(StepKind::SystemConfigurationSlot, None),
+    ];
+    let floor_raises = FloorRaisesOfTheMount {
+        pushed_by_the_admission: vec![FloorRaiseInTheMountWrites::new(
+            1,
+            vec![publish_of(3, 2, 12)],
+        )],
+        continuation_steps: vec![publish_of(4, 2, 13), publish_of(4, 2, 15)],
+        pushed_by_the_continuation: vec![FloorRaiseInTheMountWrites::new(
+            1,
+            vec![publish_of(3, 2, 14)],
+        )],
+    };
+    let located =
+        FloorRaisesInTheMountWrites::of_the_floor_raises_of_the_mount(&floor_raises, &writes);
+    assert_eq!(
+        located,
+        FloorRaisesInTheMountWrites::FirstWriteAt(8),
+        "准入推的那一串从第 8 个写（新 F 写进系统配置）起，后面是它的空发布、续做两步与续做推的那一串"
+    );
+    let mut persisted_through_the_warm_up = vec![false; writes.len()];
+    persisted_through_the_warm_up[..8].fill(true);
+    assert_eq!(
+        located.at_a_second_crash(&persisted_through_the_warm_up),
+        FloorRaisesAtTheSecondCrash::NoneLandedYet,
+        "崩在暖机之后、准入推的那一串之前：F 一步都还没落"
+    );
+    let mut persisted_into_the_continuation_raise = vec![false; writes.len()];
+    persisted_into_the_continuation_raise[..17].fill(true);
+    assert_eq!(
+        located.at_a_second_crash(&persisted_into_the_continuation_raise),
+        FloorRaisesAtTheSecondCrash::SomeMayHaveLanded,
+        "崩在续做推的那一串里：F 不照「没推」判"
+    );
+}
````

## 3、实现员 Z（U13、重钉取样点、A3b-15、清单为空那一形、LR1）

`/tmp/claude-1000/m3-batch12/z/patch/crates.patch`

````diff
diff --git a/crates/singlefs-harness/src/history.rs b/crates/singlefs-harness/src/history.rs
index 6fab81c..ac1cbfe 100644
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -51,7 +51,7 @@ use singlefs_core::transaction::{
     PoolWriter, PublishError, TransactionOutput, ZeroUnitPublishPlan,
 };
 use singlefs_core::unit::data_unit_payload_capacity;
-use singlefs_format::{DATA_UNIT_BYTES, SLOT_BYTES};
+use singlefs_format::{DATA_UNIT_BYTES, EXTENT_TREE_LOWER_LEAF_DATA_UNITS, SLOT_BYTES};
 
 use crate::fault_injection::{FaultInjectingBlockDevice, SharedFaultPlan};
 use crate::memory_pool::{MemoryPool, RecordCheck, SparseBlockDevice};
@@ -307,6 +307,23 @@ pub enum ContentLength {
     OneByteAboveDataUnitPayloadCapacity,
     /// 装不下：一个数据单元的整宽（含头与预留）。
     ExactlyDataUnitBytes,
+    /// 跨 `data_units`（≥ 2）个数据单元的内容：前面的单元装满净荷容量，最后一个单元装 [1, 载荷容量] 里按选择子取的字节。
+    /// 这一种走顺序写入口（[`ContentLength::whole_file_write_entry`]，`UserChange::SequentialWrite`）：覆盖写的契约是一个数据单元，
+    /// 多于一片下段叶（144 个单元）的文件只有顺序写长得出来（代码轮 U13）。只有偏向删除多片下段叶那一组比重抽它
+    /// （[`OverwriteContentDraw::AlsoSpanningSeveralDataUnits`]），别的几组比重的抽法与生成的历史不变。
+    SpanningDataUnitsThroughTheSequentialWrite {
+        data_units: u64,
+        selector: u64,
+    },
+}
+
+/// 整份重写文件内容走哪个入口（`mounted_session::UserChange` 的哪一个成员）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum WholeFileWriteEntry {
+    /// `UserChange::Overwrite`：契约是一个数据单元，装不下拒。
+    Overwrite,
+    /// `UserChange::SequentialWrite`：按切分纪律切成几个一单元事务，不按单元数拒。
+    SequentialWrite,
 }
 
 impl ContentLength {
@@ -323,6 +340,32 @@ impl ContentLength {
             ContentLength::ExactlyDataUnitPayloadCapacity => capacity,
             ContentLength::OneByteAboveDataUnitPayloadCapacity => capacity + 1,
             ContentLength::ExactlyDataUnitBytes => usize::try_from(DATA_UNIT_BYTES).expect("32768"),
+            ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
+                data_units,
+                selector,
+            } => {
+                let capacity_in_bytes = u64::try_from(capacity).expect("载荷容量三万多字节");
+                let bytes_of_the_full_units = (data_units - 1) * capacity_in_bytes;
+                let bytes_of_the_last_unit = 1 + selector % capacity_in_bytes;
+                usize::try_from(bytes_of_the_full_units + bytes_of_the_last_unit)
+                    .expect("几百个单元的内容装得进 usize")
+            }
+        }
+    }
+
+    /// 这一种内容走哪个入口：跨几个数据单元的走顺序写，别的（含装不下一个单元的那两种：它们要钉的正是覆盖写的拒）走覆盖写。
+    #[must_use]
+    pub fn whole_file_write_entry(self) -> WholeFileWriteEntry {
+        match self {
+            ContentLength::Empty
+            | ContentLength::InsideOneDataUnit { .. }
+            | ContentLength::OneByteBelowDataUnitPayloadCapacity
+            | ContentLength::ExactlyDataUnitPayloadCapacity
+            | ContentLength::OneByteAboveDataUnitPayloadCapacity
+            | ContentLength::ExactlyDataUnitBytes => WholeFileWriteEntry::Overwrite,
+            ContentLength::SpanningDataUnitsThroughTheSequentialWrite { .. } => {
+                WholeFileWriteEntry::SequentialWrite
+            }
         }
     }
 
@@ -336,6 +379,15 @@ impl ContentLength {
             ContentLength::ExactlyDataUnitPayloadCapacity => "载荷容量",
             ContentLength::OneByteAboveDataUnitPayloadCapacity => "载荷容量 + 1（装不下）",
             ContentLength::ExactlyDataUnitBytes => "数据单元整宽 32768（装不下）",
+            ContentLength::SpanningDataUnitsThroughTheSequentialWrite { data_units, .. } => {
+                if data_units <= EXTENT_TREE_LOWER_LEAF_DATA_UNITS {
+                    "跨一片下段叶以内的多个单元（顺序写）"
+                } else if data_units <= 2 * EXTENT_TREE_LOWER_LEAF_DATA_UNITS {
+                    "跨两片下段叶（顺序写）"
+                } else {
+                    "跨三片及以上下段叶（顺序写）"
+                }
+            }
         }
     }
 }
@@ -381,6 +433,21 @@ pub enum RollbackTargetDraw {
     RecentUniformOrBeyond,
     /// 先抽一次：一半取候选集的下沿（`RingRootAtTheNewestFloor`），另一半照第一版的抽法。
     FloorRootHalfTheTime,
+    /// 先抽一次：一半取根环里第四新的那条根（`RingRoot { index_from_newest: 3 }`），另一半照第一版的抽法。unlink 一个两片下段叶的文件
+    /// 一次写出三条根（意图那次、一个删除批、最后一批）；删完的那一版没有文件、回退不接（执行器记「前提不满足」），建一个 inode 之后才接，
+    /// 这时第四新的根就是意图那一版（三片的是第一个删除批那一版），回退到它把号带回队列（D8（核心索引结构） 已定项 16 依据
+    /// 「回退到意图那一版会把号带回队列」）、之后的用户改动先走空多步（代码轮 U13）。
+    FourthNewestRootHalfTheTime,
+}
+
+/// 覆盖写（`HistoryOperation::PublishOverwrite`）的内容按什么抽。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum OverwriteContentDraw {
+    /// 第一版的六种长度（`draw_content`）：都在一个数据单元以内或刚越过它，全走覆盖写入口。
+    WithinOneDataUnitOrJustBeyond,
+    /// 另抽跨几个数据单元、走顺序写入口的内容（[`ContentLength::SpanningDataUnitsThroughTheSequentialWrite`]），
+    /// 多半多于一片下段叶（`draw_content_spanning_several_data_units`）。
+    AlsoSpanningSeveralDataUnits,
 }
 
 /// 抬 F 的目标怎么取。
@@ -400,7 +467,9 @@ pub struct FloorTargetChoice {
 pub enum HistoryOperation {
     /// `transaction::publish_first_file`，接在现行版本后面（上一条记录取现行版本的；照抄实例表指针的那条根也取现行版本的）。
     PublishFirstFile(ContentChoice),
-    /// `transaction::publish_overwrite`，接在现行的带文件的版本后面。
+    /// 整份重写文件内容，经挂着的会话发，接在现行的带文件的版本后面：内容跨几个数据单元的
+    /// （[`ContentLength::SpanningDataUnitsThroughTheSequentialWrite`]）走顺序写（`UserChange::SequentialWrite`，代码轮 U13），
+    /// 别的走覆盖写（`UserChange::Overwrite`，`transaction::publish_overwrite`）。
     PublishOverwrite(ContentChoice),
     /// `transaction::publish_without_units`，接在现行的树表 0 条的版本后面。
     PublishWithoutUnits,
@@ -420,8 +489,8 @@ pub enum HistoryOperation {
     /// `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 另把见证它的系统配置槽清掉）这一步不造。
     CrashRecoveryAbandoningTheNewestRoot,
     /// 经挂着的会话 unlink 第一个文件（`mounted_session::UserChange::Unlink`，D8（核心索引结构） 已定项 16）：一次调用 = 意图那次 +
-    /// 从尾部全部分批（模型的文件一个数据单元，就是最后一批），中间为空间推的抬 F 照会话交回的比。只在现行那一版带文件、
-    /// `deleted_inodes` 空时调。
+    /// 从尾部全部分批（文件多于一片下段叶的 144 个单元时先有不是最后一批的删除批，再最后一批），中间为空间推的抬 F 照会话交回的比。
+    /// 现行那一版有第一个文件那条记录时调（删除中的那一版也调：会话先走空、再拒）。
     Unlink,
     /// 经挂着的会话建一个 size 0 的 inode（`mounted_session::UserChange::NewInodes`，count 1；D8（核心索引结构） 已定项 16 第 5 条：
     /// 删完之后只许建 inode、写行与空发布）。只在第一个文件已删完的一版上调（有树而文件集合为空的一版、或在它上面建过 inode 的一版）：
@@ -512,6 +581,8 @@ pub enum NamedGenerationWeights {
     TowardTheAllocationRecordWall,
     /// [`GenerationWeights::TOWARD_THE_UNIT_AREA_WALL`]。
     TowardTheUnitAreaWall,
+    /// [`GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES`]。
+    DeletingFilesOfSeveralLowerLeaves,
 }
 }
 
@@ -533,6 +604,9 @@ impl NamedGenerationWeights {
             NamedGenerationWeights::TowardTheUnitAreaWall => {
                 GenerationWeights::TOWARD_THE_UNIT_AREA_WALL
             }
+            NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves => {
+                GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES
+            }
         }
     }
 
@@ -545,6 +619,7 @@ impl NamedGenerationWeights {
             NamedGenerationWeights::RollbackAfterRaisingTheFloor => "rollback",
             NamedGenerationWeights::TowardTheAllocationRecordWall => "wall",
             NamedGenerationWeights::TowardTheUnitAreaWall => "unit-area-wall",
+            NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves => "several-leaves",
         }
     }
 }
@@ -568,6 +643,8 @@ pub struct GenerationWeights {
     pub with_session_open_after_the_file_was_deleted:
         Option<&'static [(HistoryOperationKind, u64)]>,
     pub rollback_targets: RollbackTargetDraw,
+    /// 覆盖写的内容按什么抽：只有偏向删除多片下段叶那一组抽跨几个单元的，别的几组照第一版，生成的历史不变。
+    pub overwrite_contents: OverwriteContentDraw,
 }
 
 impl GenerationWeights {
@@ -633,6 +710,7 @@ impl GenerationWeights {
             (HistoryOperationKind::PublishWithoutUnits, 4),
         ]),
         rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
+        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
     };
 
     /// 第 121 行那一类（复用时新记录罩住别的已回收记录）的专门取样点：多从第一个文件起、多覆盖写、多抬 F、多可写挂载，
@@ -681,6 +759,7 @@ impl GenerationWeights {
             (HistoryOperationKind::PublishWithoutUnits, 2),
         ]),
         rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
+        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
     };
 
     /// 理想模型那一格（B2：回退到 txg = F_生效 的根被拒）的专门取样点：多从第一个文件起、多覆盖写（攒非空根，抬 F 的上限才抬得动）、
@@ -717,6 +796,7 @@ impl GenerationWeights {
         ],
         with_session_open_after_the_file_was_deleted: None,
         rollback_targets: RollbackTargetDraw::FloorRootHalfTheTime,
+        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
     };
 
     /// 原分配记录墙那一格（增补 3 第 2 件代码三方第一轮判决第三节第 1 条）的取样点：一律从第一个文件起，带文件的版本上多半覆盖写，
@@ -741,6 +821,7 @@ impl GenerationWeights {
         ],
         with_session_open_after_the_file_was_deleted: None,
         rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
+        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
     };
 
     /// 单元区墙那一格（增补 3 第 2 件代码三方第二轮判决第三节第 2 条）的取样点，配 `HistoryDeviceWidth::UnitAreaOf256Slots` 跑：
@@ -770,6 +851,53 @@ impl GenerationWeights {
         ],
         with_session_open_after_the_file_was_deleted: None,
         rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
+        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
+    };
+
+    /// 删除多于一片下段叶的文件那一格（代码轮 U13：模型罩多个数据单元的文件、随机历史里出现 ≥ 2 片下段叶的删除批；
+    /// 代码轮第二轮判决第二节 N3「走空多步」）的取样点，配两块 4 GiB 的盘跑：多从第一个文件起；带文件的版本上覆盖写的内容另抽跨几个单元、
+    /// 走顺序写入口的（[`OverwriteContentDraw::AlsoSpanningSeveralDataUnits`]，多半两片下段叶），夹着很多 unlink（一次 unlink
+    /// = 意图那次 + 从尾部每批 144 个单元 + 最后一批）；unlink 之后只抽建 inode、回退、覆盖写与 unlink（不抽挂载、抬 F、冷启动：
+    /// 它们写的根会把意图那一版在根环里的名次往后推），回退一半取第四新的根（建过一个 inode 之后落在意图那一版或第一个删除批那一版上，
+    /// 号回到队列），再建 inode、覆盖写或 unlink（会话先把队列走空，多片的文件走空多步）。快档那组比重不动：它的种子钉着的取样点与
+    /// checker 档、实验装置的复现都按它生成。
+    pub const DELETING_FILES_OF_SEVERAL_LOWER_LEAVES: GenerationWeights = GenerationWeights {
+        name: "偏向删除多于一片下段叶的文件（U13 那一格的取样点）",
+        named: NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves,
+        starting_points: &[
+            (HistoryStartingPoint::AfterMakeFilesystem, 1),
+            (HistoryStartingPoint::AfterFirstFile, 9),
+        ],
+        with_session_closed: &[
+            (HistoryOperationKind::CloseAndMountWritable, 95),
+            (HistoryOperationKind::ColdStartRecover, 5),
+        ],
+        with_session_open_without_file: &[
+            (HistoryOperationKind::PublishFirstFile, 80),
+            (HistoryOperationKind::CloseAndMountWritable, 10),
+            (HistoryOperationKind::PublishWithoutUnits, 5),
+            (HistoryOperationKind::ColdStartRecover, 5),
+        ],
+        with_session_open_with_file: &[
+            (HistoryOperationKind::PublishOverwrite, 45),
+            (HistoryOperationKind::Unlink, 25),
+            (HistoryOperationKind::RaiseRollbackFloor, 8),
+            (HistoryOperationKind::CloseAndMountWritable, 8),
+            (HistoryOperationKind::RollBackWhileMounted, 8),
+            (HistoryOperationKind::ColdStartRecover, 3),
+            (
+                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
+                3,
+            ),
+        ],
+        with_session_open_after_the_file_was_deleted: Some(&[
+            (HistoryOperationKind::CreateInode, 40),
+            (HistoryOperationKind::RollBackWhileMounted, 40),
+            (HistoryOperationKind::PublishOverwrite, 10),
+            (HistoryOperationKind::Unlink, 10),
+        ]),
+        rollback_targets: RollbackTargetDraw::FourthNewestRootHalfTheTime,
+        overwrite_contents: OverwriteContentDraw::AlsoSpanningSeveralDataUnits,
     };
 
     /// 复现入口认得的每一组比重：按 `environment_value` 找回（[`HistorySampling::from_environment`]）。
@@ -837,29 +965,83 @@ fn draw_content(source: &mut SeededRandomSource) -> ContentChoice {
     }
 }
 
+/// 跨几个数据单元的内容（[`OverwriteContentDraw::AlsoSpanningSeveralDataUnits`]）：与 `draw_content` 抽的随机数一样多（选择子、
+/// 长度种类、填充种子各一个），五分之一在一个单元以内（走覆盖写），其余走顺序写——一片下段叶以内的几个单元（2–16 个）、两片
+/// （145–152 个单元，删除要一个删除批加最后一批）、三片（289–292 个单元，两个删除批加最后一批）。单元数取得窄是为了每一步的池级 checker 读得快。
+fn draw_content_spanning_several_data_units(source: &mut SeededRandomSource) -> ContentChoice {
+    let selector = source.next_word();
+    let length = draw_weighted(
+        source,
+        &[
+            (ContentLength::InsideOneDataUnit { selector }, 20),
+            (
+                ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
+                    data_units: 2 + selector % 15,
+                    selector,
+                },
+                15,
+            ),
+            (
+                ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
+                    data_units: EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 1 + selector % 8,
+                    selector,
+                },
+                55,
+            ),
+            (
+                ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
+                    data_units: 2 * EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 1 + selector % 4,
+                    selector,
+                },
+                10,
+            ),
+        ],
+    );
+    ContentChoice {
+        length,
+        fill_seed: source.next_word(),
+    }
+}
+
 fn draw_operation(
     source: &mut SeededRandomSource,
     kind: HistoryOperationKind,
     rollback_targets: RollbackTargetDraw,
+    overwrite_contents: OverwriteContentDraw,
 ) -> HistoryOperation {
     match kind {
         HistoryOperationKind::PublishFirstFile => {
             HistoryOperation::PublishFirstFile(draw_content(source))
         }
         HistoryOperationKind::PublishOverwrite => {
-            HistoryOperation::PublishOverwrite(draw_content(source))
+            HistoryOperation::PublishOverwrite(match overwrite_contents {
+                OverwriteContentDraw::WithinOneDataUnitOrJustBeyond => draw_content(source),
+                OverwriteContentDraw::AlsoSpanningSeveralDataUnits => {
+                    draw_content_spanning_several_data_units(source)
+                }
+            })
         }
         HistoryOperationKind::PublishWithoutUnits => HistoryOperation::PublishWithoutUnits,
         HistoryOperationKind::CloseAndMountWritable => HistoryOperation::CloseAndMountWritable,
         HistoryOperationKind::RollBackWhileMounted => {
-            let takes_the_floor_root = match rollback_targets {
-                RollbackTargetDraw::RecentUniformOrBeyond => false,
-                RollbackTargetDraw::FloorRootHalfTheTime => source.below(2) == 0,
-            };
-            if takes_the_floor_root {
-                return HistoryOperation::RollBackWhileMounted(
-                    RollbackTargetChoice::RingRootAtTheNewestFloor,
-                );
+            match rollback_targets {
+                RollbackTargetDraw::RecentUniformOrBeyond => {}
+                RollbackTargetDraw::FloorRootHalfTheTime => {
+                    if source.below(2) == 0 {
+                        return HistoryOperation::RollBackWhileMounted(
+                            RollbackTargetChoice::RingRootAtTheNewestFloor,
+                        );
+                    }
+                }
+                RollbackTargetDraw::FourthNewestRootHalfTheTime => {
+                    if source.below(2) == 0 {
+                        return HistoryOperation::RollBackWhileMounted(
+                            RollbackTargetChoice::RingRoot {
+                                index_from_newest: 3,
+                            },
+                        );
+                    }
+                }
             }
             // 一半取最近的几条（多半在候选集里），一半在整个根环里均匀取（被抛弃的、F 之下的、暖机那两条树表 0 条的都落得到），
             // 余下的取根环之外。
@@ -1002,7 +1184,12 @@ pub fn generate_history_with_weights(
     let mut operations = Vec::with_capacity(operation_count);
     for _ in 0..operation_count {
         let kind = draw_weighted(&mut source, weights.for_session(expected));
-        operations.push(draw_operation(&mut source, kind, weights.rollback_targets));
+        operations.push(draw_operation(
+            &mut source,
+            kind,
+            weights.rollback_targets,
+            weights.overwrite_contents,
+        ));
         expected = expected_session_after(
             expected,
             first_file,
@@ -1991,6 +2178,12 @@ pub struct HistoryTally {
     /// （D8（核心索引结构） 已定项 14），这个数越过 812 说明历史走过了原先那道墙。
     pub most_allocation_records_in_one_version: u64,
     pub histories_that_turned_the_root_ring: u64,
+    /// 模型比过至少一个不是最后一批的删除批（≥ 2 片下段叶的文件的删除批，代码轮 U13）的段数。
+    pub histories_with_a_deletion_batch_keeping_data_units: u64,
+    /// 会话里一次用户改动之前先走空、走了至少两步、模型逐步比过的段数（代码轮第二轮判决第二节 N3「走空多步」）。
+    pub histories_with_a_walk_of_several_steps_before_a_user_change: u64,
+    /// 会话里一次用户改动之前先走空、走空被空间拒推了抬 F、模型逐串逐步比过的段数（N3「走空被空间拒」）。
+    pub histories_with_a_walk_before_a_user_change_refused_for_space: u64,
     pub checker_runs: u64,
     /// 这一步一个写都没发（录制流一步没多）：镜像逐字节不变，checker 的结论沿用上一次，不重跑。
     pub checker_runs_skipped_because_nothing_was_written: u64,
@@ -2068,6 +2261,12 @@ impl HistoryTally {
             .most_allocation_records_in_one_version
             .max(other.most_allocation_records_in_one_version);
         self.histories_that_turned_the_root_ring += other.histories_that_turned_the_root_ring;
+        self.histories_with_a_deletion_batch_keeping_data_units +=
+            other.histories_with_a_deletion_batch_keeping_data_units;
+        self.histories_with_a_walk_of_several_steps_before_a_user_change +=
+            other.histories_with_a_walk_of_several_steps_before_a_user_change;
+        self.histories_with_a_walk_before_a_user_change_refused_for_space +=
+            other.histories_with_a_walk_before_a_user_change_refused_for_space;
         self.checker_runs += other.checker_runs;
         self.checker_runs_skipped_because_nothing_was_written +=
             other.checker_runs_skipped_because_nothing_was_written;
@@ -2289,6 +2488,18 @@ impl HistoryTally {
             self.user_changes_published_after_the_session_raised_the_floor,
             self.floor_raises_pushed_by_the_session_before_a_published_change
         );
+        let _ = writeln!(
+            text,
+            "  删除：模型比过不是最后一批的删除批（≥ 2 片下段叶）{} 次、在 {} 段里；会话改动之前先走空走了至少两步 {} 次、在 {} 段里；走空被空间拒推了抬 F {} 次、在 {} 段里",
+            self.model_counts.deletion_batches_keeping_data_units_matched,
+            self.histories_with_a_deletion_batch_keeping_data_units,
+            self.model_counts
+                .walks_of_several_steps_before_a_user_change_matched,
+            self.histories_with_a_walk_of_several_steps_before_a_user_change,
+            self.model_counts
+                .walks_before_a_user_change_refused_for_space_matched,
+            self.histories_with_a_walk_before_a_user_change_refused_for_space
+        );
         let _ = writeln!(
             text,
             "  checker 跑了 {} 次；一个写都没发、沿用上一次结论的 {} 步",
@@ -2951,13 +3162,25 @@ fn apply_publish_overwrite(
         shadow_ledger,
         parameters_and_device_table,
     };
+    let file = FirstFile {
+        content: &content,
+        write_time_seconds,
+    };
+    // 内容跨几个数据单元的走顺序写入口（代码轮 U13），别的走覆盖写入口；模型那一侧各按各的契约答。
+    let entry = content_choice.length.whole_file_write_entry();
     let change_result = mounted_session.publish_user_change(
         devices,
-        UserChange::Overwrite(FirstFile {
-            content: &content,
-            write_time_seconds,
-        }),
+        match entry {
+            WholeFileWriteEntry::Overwrite => UserChange::Overwrite(file),
+            WholeFileWriteEntry::SequentialWrite => UserChange::SequentialWrite(file),
+        },
     );
+    let answer_of_the_change = |ideal_model: &IdealModel| match entry {
+        WholeFileWriteEntry::Overwrite => ideal_model.answer_publish_overwrite(&content),
+        WholeFileWriteEntry::SequentialWrite => {
+            ideal_model.answer_publish_sequential_write(&content)
+        }
+    };
     let session_after_the_change = session_slot.insert(WritableSession {
         allocator: mounted_session.allocator,
         current: mounted_session.current,
@@ -2971,7 +3194,7 @@ fn apply_publish_overwrite(
         return settle_user_change_comparing_every_version_published(
             session_after_the_change,
             model,
-            &|ideal_model: &IdealModel| ideal_model.answer_publish_overwrite(&content),
+            &answer_of_the_change,
             &records_before,
             &change_result,
             stream,
@@ -2981,7 +3204,7 @@ fn apply_publish_overwrite(
     settle_user_change(
         session_after_the_change,
         model,
-        &content,
+        &answer_of_the_change,
         &records_before,
         change_result,
         stream,
@@ -3074,7 +3297,7 @@ fn writes_of_the_floor_raises(floor_raises: &[singlefs_core::mount::RaisedFloor]
 fn settle_user_change(
     session: &mut WritableSession,
     model: &mut IdealModel,
-    content: &[u8],
+    answer_of_the_change: &dyn Fn(&IdealModel) -> Result<ModelAnswer, ModelDisagreement>,
     records_before: &[AllocationRecord],
     change_result: Result<UserChangePublished, UserChangeRefused>,
     stream: &SharedStream,
@@ -3120,7 +3343,7 @@ fn settle_user_change(
     }
     let refusal = match &change_result {
         Ok(_) => {
-            let answer = model.answer_publish_overwrite(content);
+            let answer = answer_of_the_change(model);
             let output = session.current.file_version().expect(
                 "会话发成一次覆盖写之后现行那一版带文件：`publish_user_change` 只接在带文件的一版后面、把新的一版放进 `current`",
             );
@@ -3159,7 +3382,7 @@ fn settle_user_change(
     };
     let (member, reason) = member_and_reason_of_a_refused_user_change(refusal);
     if verdict.disagreement.is_none() {
-        let answer = model.answer_publish_overwrite(content);
+        let answer = answer_of_the_change(model);
         let wrote_anything_besides_the_floor_raises = if floor_raises.is_empty() {
             stream.operation_count() != stream_length_before
         } else {
@@ -3400,10 +3623,19 @@ fn settle_user_change_comparing_every_version_published(
     let mut verdict = ModelVerdict::default();
     let mut harness_judgement = None;
     let mut records_of_the_previous_publish: &[AllocationRecord] = records_before;
+    // 这次改动自己（意图那次、建 inode 那次、覆盖写）发成之前，会话先走空的那几步与走空时为空间推的抬 F（代码轮第二轮判决第二节 N3）。
+    let mut the_change_itself_was_published = false;
+    let mut steps_walked_before_the_change = 0_u64;
+    let mut floor_raises_while_walking_before_the_change = 0_u64;
     // 迭代上界是这次落盘的样数；跨轮带的是上一样落盘之后的分配记录与合起来的判定；模型对不上之后不再问模型（分配代照判）。
     for published in &published_in_txg_order {
         let outputs: Vec<&singlefs_core::transaction::TransactionOutput> = match published {
             FloorRaiseOrVersion::FloorRaise(raised) => {
+                if !the_change_itself_was_published
+                    && model.current_version_has_a_file_being_deleted()
+                {
+                    floor_raises_while_walking_before_the_change += 1;
+                }
                 if verdict.disagreement.is_none() {
                     let raise_verdict = judge_one_floor_raise_pushed_by_the_session(model, raised);
                     verdict.counts.add(&raise_verdict.counts);
@@ -3412,8 +3644,15 @@ fn settle_user_change_comparing_every_version_published(
                 raised.publishes.iter().collect()
             }
             FloorRaiseOrVersion::Version(version) => {
+                let walking = model.current_version_has_a_file_being_deleted();
+                if walking && !the_change_itself_was_published {
+                    steps_walked_before_the_change += 1;
+                }
+                if !walking {
+                    the_change_itself_was_published = true;
+                }
                 if verdict.disagreement.is_none() {
-                    let answer = if model.current_version_has_a_file_being_deleted() {
+                    let answer = if walking {
                         model.answer_deletion_step()
                     } else {
                         answer_of_the_change(model)
@@ -3446,6 +3685,18 @@ fn settle_user_change_comparing_every_version_published(
             session.publishes_in_this_mount += 1;
         }
     }
+    if verdict.disagreement.is_none() {
+        if steps_walked_before_the_change >= 2 {
+            verdict
+                .counts
+                .walks_of_several_steps_before_a_user_change_matched += 1;
+        }
+        if floor_raises_while_walking_before_the_change > 0 {
+            verdict
+                .counts
+                .walks_before_a_user_change_refused_for_space_matched += 1;
+        }
+    }
     let reuse = RecordReuse::between(records_before, session.allocator.records());
     let refusal = match change_result {
         Ok(_) => {
@@ -5190,6 +5441,24 @@ pub fn execute_history_with_faults(
         }
     };
     tally.histories = 1;
+    tally.histories_with_a_deletion_batch_keeping_data_units = u64::from(
+        tally
+            .model_counts
+            .deletion_batches_keeping_data_units_matched
+            > 0,
+    );
+    tally.histories_with_a_walk_of_several_steps_before_a_user_change = u64::from(
+        tally
+            .model_counts
+            .walks_of_several_steps_before_a_user_change_matched
+            > 0,
+    );
+    tally.histories_with_a_walk_before_a_user_change_refused_for_space = u64::from(
+        tally
+            .model_counts
+            .walks_before_a_user_change_refused_for_space_matched
+            > 0,
+    );
     HistoryRun {
         ending,
         outcomes,
diff --git a/crates/singlefs-harness/src/model.rs b/crates/singlefs-harness/src/model.rs
index b722c06..309a316 100644
--- a/crates/singlefs-harness/src/model.rs
+++ b/crates/singlefs-harness/src/model.rs
@@ -22,10 +22,11 @@ use std::rc::Rc;
 use singlefs_format::{
     ACCOUNTING_ENTRY_BYTES, ACCOUNTING_KEY_BYTES, ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
     ALLOCATION_RECORD_TREE_LEAF_SLOTS, CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES,
-    DATA_UNIT_PAYLOAD_OFFSET, INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE,
-    INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT, INSTANCE_TABLE_PAGE_RECORDS, JOURNAL_RING_DEFAULT_BYTES,
-    JOURNAL_RING_START_SLOT, NODE_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES, ROOT_RING_REGIONS,
-    ROOT_RING_REGION_DEVICES, ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM, SLOT_BYTES,
+    DATA_UNIT_PAYLOAD_OFFSET, EXTENT_TREE_INTERNAL_FANOUT, EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
+    INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE, INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT,
+    INSTANCE_TABLE_PAGE_RECORDS, JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, NODE_BYTES,
+    NONCE_MAC_ALGORITHM_RESERVED_BYTES, ROOT_RING_REGIONS, ROOT_RING_REGION_DEVICES,
+    ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM, SLOT_BYTES,
 };
 
 /// 模型里的 checkpoint 号（D16（发布语义） 已定项 6：每次发布 + 1）。与实现的 `CheckpointTxg` 各自声明（D13（验证路线） 已定项 5）。
@@ -108,7 +109,10 @@ enum ModelPublishKind {
     /// 第一个文件版本：四个文件角色，加四个固定点单元（D16（发布语义） 已定项 9：记账行每发布重写，连带记账树节点、分配记录、
     /// 映射条目与树表单元）。与覆盖写重写的角色相同，分开只为一件事——它走 `previous: None`，这一版的内存态从头建。
     FirstFileVersion,
-    /// 覆盖写：与第一个文件版本重写的角色相同，接在上一版的内存态后面。
+    /// 整份重写文件内容：覆盖写（一个数据单元以内）与顺序写（跨几个数据单元，`transaction::publish_sequential_write`）。
+    /// 与第一个文件版本重写的角色相同，接在上一版的内存态后面；内容的每个数据单元都在这次写（同一个分配代），
+    /// 多于一个单元时 extent 树下段的节点（叶罩 144 个单元、内部扇出 147，D8（核心索引结构） 已定项 14）也在这次写，
+    /// 一个单元一个事务（D16（发布语义） 已定项 5 的切分纪律）。
     OverwriteFileVersion,
     /// 写行：实例表单元（D18（块里携带什么信息） 已定项 11：每次可写挂载都写行）加四个固定点单元（D16（发布语义） 已定项 9）。
     RowsOnFileVersion,
@@ -122,7 +126,12 @@ enum ModelPublishKind {
     /// unlink 的意图那次（D8（核心索引结构） 已定项 16 第 1 条）：inode 叶与根（nlink 写 0）、`deleted_inodes` 根（加 key）、
     /// 墓碑树开放容器与根（追加一条墓碑），加四个固定点单元；数据单元与 extent 树照抄。
     UnlinkIntent,
-    /// 最后一批（D8（核心索引结构） 已定项 16 第 2 条）：模型的文件只有一个数据单元，意图之后的下一步就是它——释放剩下的单元、
+    /// 不是最后一批的删除批（D8（核心索引结构） 已定项 16 第 2 条：剩下的单元多于产品批大小 144 时从尾部释放 144 个）：
+    /// 留下的数据单元照抄（分配代不变）；重写被截掉那一片下段叶与它的祖先（下段节点不对应角色，占槽按下段层数取上界）、
+    /// extent 上段根、inode 叶与根（记录的 size 改成剩下单元数 × 净荷容量，nlink 仍 0），加四个固定点单元；
+    /// `deleted_inodes` 那条 key 与墓碑树不动。这一版仍是删除中的那一版。
+    DeletionBatchKeepingDataUnits,
+    /// 最后一批（D8（核心索引结构） 已定项 16 第 2 条）：剩下的单元不多于产品批大小 144 时意图之后（或上一批之后）的下一步——释放剩下的单元、
     /// 删 inode 记录（树里只有 inode 1，叶容器一片都不剩）、删 `deleted_inodes` 那条 key（删空取根指针置零、不写它的节点），
     /// 同一次发布；重写 extent 零条目根与 inode 零条目根，加四个固定点单元。
     FinalDeletionBatch,
@@ -155,24 +164,28 @@ impl ModelPublishKind {
             | ModelPublishKind::EmptyOnFileVersion
             | ModelPublishKind::ZeroUnit
             | ModelPublishKind::UnlinkIntent
+            | ModelPublishKind::DeletionBatchKeepingDataUnits
             | ModelPublishKind::FinalDeletionBatch
             | ModelPublishKind::NewInode => false,
         }
     }
 
-    /// 这次发布的记录承载一个事务（事务号取本实例的下一个）：写文件内容的两种（D16（发布语义） 已定项 5）与删除类发布
-    /// （删除那几步各是一个事务，接着本实例的计数）；空发布、写行、暖机、抬 F、回退的记录事务号 0（D23（journal 的角色与格式） 已定项 19 ①）。
-    fn carries_a_transaction(self) -> bool {
+    /// 这次发布的记录承载几个事务（事务号接着本实例的计数往下取）：写文件内容的两种一个数据单元一个事务（D16（发布语义） 已定项 5
+    /// 的切分纪律，`data_units_of_the_new_content` 是这次写的内容切成几个单元）；删除类发布与建 inode 各一个事务；
+    /// 空发布、写行、暖机、抬 F、回退的记录事务号 0（D23（journal 的角色与格式） 已定项 19 ①）。
+    fn transactions_carried(self, data_units_of_the_new_content: u64) -> u64 {
         match self {
-            ModelPublishKind::FirstFileVersion
-            | ModelPublishKind::OverwriteFileVersion
-            | ModelPublishKind::UnlinkIntent
+            ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion => {
+                data_units_of_the_new_content
+            }
+            ModelPublishKind::UnlinkIntent
+            | ModelPublishKind::DeletionBatchKeepingDataUnits
             | ModelPublishKind::FinalDeletionBatch
-            | ModelPublishKind::NewInode => true,
+            | ModelPublishKind::NewInode => 1,
             ModelPublishKind::RowsOnFileVersion
             | ModelPublishKind::EmptyOnFileVersion
             | ModelPublishKind::RowsOnVersionWithoutFile
-            | ModelPublishKind::ZeroUnit => false,
+            | ModelPublishKind::ZeroUnit => 0,
         }
     }
 
@@ -191,6 +204,7 @@ impl ModelPublishKind {
             | ModelPublishKind::RowsOnVersionWithoutFile
             | ModelPublishKind::ZeroUnit
             | ModelPublishKind::UnlinkIntent
+            | ModelPublishKind::DeletionBatchKeepingDataUnits
             | ModelPublishKind::NewInode => &[],
         }
     }
@@ -239,6 +253,15 @@ impl ModelPublishKind {
                 ModelUnitRole::MappingTree,
                 ModelUnitRole::TreeTable,
             ],
+            ModelPublishKind::DeletionBatchKeepingDataUnits => &[
+                ModelUnitRole::ExtentRoot,
+                ModelUnitRole::InodeLeaf,
+                ModelUnitRole::InodeRoot,
+                ModelUnitRole::AllocationTree,
+                ModelUnitRole::AccountingTree,
+                ModelUnitRole::MappingTree,
+                ModelUnitRole::TreeTable,
+            ],
             ModelPublishKind::FinalDeletionBatch => &[
                 ModelUnitRole::ExtentRoot,
                 ModelUnitRole::InodeRoot,
@@ -275,8 +298,13 @@ pub enum ModelVersionContents {
     /// inode 1 是活记录：读回它的内容。
     LiveFile(ModelFileVersion),
     /// unlink 的意图那次之后、最后一批之前：inode 1 的记录 nlink = 0、号在 `deleted_inodes` 里，读回「没有文件」，它的单元还在。
-    /// `trees_written_by`：写出这一版 inode 树、extent 树根指针的那次发布（意图那次；回退到这一版的根照抄它）。
-    FileBeingDeleted { trees_written_by: ModelRootKey },
+    /// `trees_written_by`：写出这一版 inode 树、extent 树根指针的那次发布（意图那次或上一个不是最后一批的删除批；回退到这一版的根照抄它）。
+    /// `data_units_remaining`：这一版里这个文件还剩几个数据单元（意图那次 = 文件的单元数，每个不是最后一批的删除批少 144 个）：
+    /// 下一步是不是最后一批由它定（D8（核心索引结构） 已定项 16 第 2 条）。
+    FileBeingDeleted {
+        trees_written_by: ModelRootKey,
+        data_units_remaining: u64,
+    },
     /// 最后一批之后：有树而文件集合为空（`deleted_inodes` 空、inode 树与 extent 树零条目根），读回「没有文件」。
     TreesAndNoFile { trees_written_by: ModelRootKey },
     /// 删完第一个文件之后建过 inode：inode 树里只有 size 0 的记录、没有 inode 1，没有数据单元、队列空，读回「没有文件」；
@@ -316,7 +344,9 @@ impl ModelVersionContents {
         match self {
             ModelVersionContents::WithoutTrees => None,
             ModelVersionContents::LiveFile(file) => Some(file.written_by),
-            ModelVersionContents::FileBeingDeleted { trees_written_by }
+            ModelVersionContents::FileBeingDeleted {
+                trees_written_by, ..
+            }
             | ModelVersionContents::TreesAndNoFile { trees_written_by }
             | ModelVersionContents::InodesWithoutTheFirstFile { trees_written_by } => {
                 Some(*trees_written_by)
@@ -360,8 +390,8 @@ pub struct ModelRoot {
     pub role_written_at: BTreeMap<ModelUnitRole, ModelCheckpointTxg>,
     /// 每盘已占槽数的上界：沿来路每次发布写出的单元的槽数之和，释放与回收一概不扣（分配记录树每次重写的节点数取按位置寻址现算的上界）。
     pub occupied_slots_upper_bound_per_device: u64,
-    /// 写出这条根的那次发布的记录里最大的事务号：只有写文件内容的发布（第一个文件、覆盖写）承载用户事务，内容装在一个数据单元里
-    /// （模型拒装不下的，`ContentExceedsDataUnitPayload`），一事务一单元（D16（发布语义） 已定项 5）⇒ 恰好一个事务、取实例内计数的下一个号；
+    /// 写出这条根的那次发布的记录里最大的事务号：写文件内容的发布（第一个文件、覆盖写、顺序写）一事务一单元（D16（发布语义） 已定项 5）
+    /// ⇒ 内容切成几个数据单元就承载几个事务、接着实例内计数往下取，这里记最后那一个号；删除类发布与建 inode 各一个事务；
     /// 空发布、写行、暖机、抬 F、回退的记录事务号 0（D23（journal 的角色与格式） 已定项 19 ①），这里记 0。
     /// 恢复施加这次发布的记录时，W 按它取（D23（journal 的角色与格式） 已定项 14 第 4 条；空发布的 0 不进 max）。
     pub highest_transaction_number_of_its_publish: u64,
@@ -437,6 +467,60 @@ pub fn data_unit_payload_capacity_in_bytes() -> u64 {
     DATA_UNIT_BYTES - DATA_UNIT_PAYLOAD_OFFSET
 }
 
+/// 一份内容切成几个数据单元（D16（发布语义） 已定项 5 的切分纪律：一事务一单元，按净荷容量切）：长度除净荷容量向上取整，
+/// 长度 0 也是一个单元（声明长度 0 的数据单元）。模型自己的一份式子，不调实现的切分（D13（验证路线） 已定项 5）。
+#[must_use]
+pub fn data_units_of_content(content_length_in_bytes: u64) -> u64 {
+    content_length_in_bytes
+        .div_ceil(data_unit_payload_capacity_in_bytes())
+        .max(1)
+}
+
+/// 一个文件有 `data_units` 个数据单元时 extent 树下段有几个节点（D8（核心索引结构） 已定项 14：只有一个单元时不建下段、
+/// 那个指针内联在上段叶条目里；下段按单元号按位置寻址，叶罩 144 个单元，内部节点扇出 147，往上数到只剩一个节点为止）。
+#[must_use]
+pub fn extent_lower_segment_nodes(data_units: u64) -> u64 {
+    if data_units <= 1 {
+        return 0;
+    }
+    let mut nodes_on_this_level = data_units.div_ceil(EXTENT_TREE_LOWER_LEAF_DATA_UNITS);
+    let mut nodes = nodes_on_this_level;
+    // 迭代上界：每一层的节点数除扇出 147 向上取整，严格变少直到 1。
+    while nodes_on_this_level > 1 {
+        nodes_on_this_level = nodes_on_this_level.div_ceil(EXTENT_TREE_INTERNAL_FANOUT);
+        nodes += nodes_on_this_level;
+    }
+    nodes
+}
+
+/// 一个文件有 `data_units` 个数据单元时 extent 树下段有几层（同 [`extent_lower_segment_nodes`] 的数法；没有下段是 0 层）：
+/// 不是最后一批的删除批只重写被截掉那一片下段叶与它的祖先，每层至多一个（K-R3-1），占槽的上界按截之前的层数取。
+#[must_use]
+pub fn extent_lower_segment_levels(data_units: u64) -> u64 {
+    if data_units <= 1 {
+        return 0;
+    }
+    let mut nodes_on_this_level = data_units.div_ceil(EXTENT_TREE_LOWER_LEAF_DATA_UNITS);
+    let mut levels = 1;
+    // 迭代上界同 `extent_lower_segment_nodes`。
+    while nodes_on_this_level > 1 {
+        nodes_on_this_level = nodes_on_this_level.div_ceil(EXTENT_TREE_INTERNAL_FANOUT);
+        levels += 1;
+    }
+    levels
+}
+
+/// 整份重写文件内容的入口按哪个契约收内容：覆盖写一个数据单元以内（装不下拒，D4（校验和位置） 已定项 5），顺序写不按单元数拒。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum WholeFileWriteContract {
+    OneDataUnit,
+    AnyNumberOfDataUnits,
+}
+
+/// 删除那几步每批释放几个数据单元：产品取一片下段叶罩的 144 个（D8（核心索引结构） 已定项 16 第 2 条，K 取
+/// `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`）。模型只罩产品批大小：随机历史的挂载不装只供测试压小批大小的开关。
+const MODEL_DELETION_BATCH_DATA_UNITS: u64 = EXTENT_TREE_LOWER_LEAF_DATA_UNITS;
+
 /// 模型对拒绝理由的分法：按条款说的「为什么不能做」分，不按实现的错误成员分（胶水把成员映射过来）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
 pub enum ModelRefusalReason {
@@ -538,6 +622,7 @@ pub enum ModelReadBack {
 pub enum ModelOperationKind {
     PublishFirstFile,
     PublishOverwrite,
+    PublishSequentialWrite,
     PublishWithoutUnits,
     MountWritable,
     RollbackWhileMounted,
@@ -605,7 +690,7 @@ pub struct ObservedRoot {
 pub enum ObservedFile {
     /// 树表 0 条：这一版没有文件。
     NoFile,
-    /// 从这一版的数据单元解出来的内容（整份，不取摘要：模型只用 std 与格式常量，没有与胶水共用的哈希；一版至多一个数据单元）。
+    /// 从这一版的数据单元按单元号接起来解出的内容（整份，不取摘要：模型只用 std 与格式常量，没有与胶水共用的哈希）。
     Decoded(Vec<u8>),
     /// 这一版带文件，而胶水从实现交回的单元里解不出它的内容（数据单元不在、解不开）。
     Undecodable { what: String },
@@ -824,6 +909,14 @@ pub struct ModelJudgementCounts {
     pub rollbacks_accepted_at_the_effective_floor: u64,
     /// 单元区墙拒（实现报每块盘上都没有合政策的落点）、在允许拒绝的区间里而放行的次数（小盘那一段的落点拒绝真的走到了、判过）。
     pub unit_area_wall_refusals_in_the_interval: u64,
+    /// 比过、对得上的「不是最后一批的删除批」（文件多于一片下段叶的 144 个单元时才有，D8（核心索引结构） 已定项 16 第 2 条；代码轮 U13）。
+    pub deletion_batches_keeping_data_units_matched: u64,
+    /// 会话里一次用户改动之前先走空、走了至少两步，每一步都比过、对得上的次数（执行器数，D8（核心索引结构） 已定项 16 第 6 条；
+    /// 代码轮第二轮判决第二节 N3「走空多步」那一格）。
+    pub walks_of_several_steps_before_a_user_change_matched: u64,
+    /// 会话里一次用户改动之前先走空时，走空的删除类发布被空间拒、推了抬 F 再试，推的那几串与走空的每一步都比过、对得上的次数
+    /// （执行器数；代码轮第二轮判决第二节 N3「走空被空间拒」那一格）。
+    pub walks_before_a_user_change_refused_for_space_matched: u64,
 }
 
 impl ModelJudgementCounts {
@@ -843,6 +936,9 @@ impl ModelJudgementCounts {
             ceilings_compared,
             rollbacks_accepted_at_the_effective_floor,
             unit_area_wall_refusals_in_the_interval,
+            deletion_batches_keeping_data_units_matched,
+            walks_of_several_steps_before_a_user_change_matched,
+            walks_before_a_user_change_refused_for_space_matched,
         } = other;
         self.required_refusals_matched += required_refusals_matched;
         self.permitted_refusals_taken += permitted_refusals_taken;
@@ -857,6 +953,12 @@ impl ModelJudgementCounts {
         self.ceilings_compared += ceilings_compared;
         self.rollbacks_accepted_at_the_effective_floor += rollbacks_accepted_at_the_effective_floor;
         self.unit_area_wall_refusals_in_the_interval += unit_area_wall_refusals_in_the_interval;
+        self.deletion_batches_keeping_data_units_matched +=
+            deletion_batches_keeping_data_units_matched;
+        self.walks_of_several_steps_before_a_user_change_matched +=
+            walks_of_several_steps_before_a_user_change_matched;
+        self.walks_before_a_user_change_refused_for_space_matched +=
+            walks_before_a_user_change_refused_for_space_matched;
     }
 }
 
@@ -1194,17 +1296,62 @@ impl IdealModel {
         for role in kind.removed_roles() {
             role_written_at.remove(role);
         }
+        let data_units_of_the_new_content = new_file_content.map_or(0, |content| {
+            data_units_of_content(u64::try_from(content.len()).expect("内容长度"))
+        });
+        let data_units_remaining_before_this_publish = match previous.contents {
+            ModelVersionContents::FileBeingDeleted {
+                data_units_remaining,
+                ..
+            } => Some(data_units_remaining),
+            ModelVersionContents::WithoutTrees
+            | ModelVersionContents::LiveFile(_)
+            | ModelVersionContents::TreesAndNoFile { .. }
+            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => None,
+        };
         let mut slots_written = 0;
         let mut rewrites_the_allocation_record_tree = false;
         let rewritten = kind.rewritten_roles();
         for role in rewritten {
             role_written_at.insert(*role, checkpoint_txg);
-            if *role == ModelUnitRole::AllocationTree {
-                rewrites_the_allocation_record_tree = true;
-            } else {
-                slots_written += role.span_in_slots();
+            match role {
+                ModelUnitRole::AllocationTree => rewrites_the_allocation_record_tree = true,
+                // 内容的每个数据单元都在这次写（模型把它们记成同一个角色、同一个分配代）。
+                ModelUnitRole::Data => {
+                    slots_written += role.span_in_slots() * data_units_of_the_new_content;
+                }
+                ModelUnitRole::ExtentRoot
+                | ModelUnitRole::InodeLeaf
+                | ModelUnitRole::InodeRoot
+                | ModelUnitRole::AccountingTree
+                | ModelUnitRole::MappingTree
+                | ModelUnitRole::TreeTable
+                | ModelUnitRole::InstanceTable
+                | ModelUnitRole::DeletedInodesRoot
+                | ModelUnitRole::TombstoneLeaf
+                | ModelUnitRole::TombstoneRoot => slots_written += role.span_in_slots(),
             }
         }
+        // extent 树下段的节点不对应角色（同分配记录树根之下的节点：分配代与位置归池级 checker 判），占槽按条款现算：
+        // 写内容的两种重写整个下段；不是最后一批的删除批只重写被截掉那一片下段叶与它的祖先，每层至多一个。
+        let extent_lower_segment_nodes_written = match kind {
+            ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion => {
+                extent_lower_segment_nodes(data_units_of_the_new_content)
+            }
+            ModelPublishKind::DeletionBatchKeepingDataUnits => {
+                extent_lower_segment_levels(data_units_remaining_before_this_publish.expect(
+                    "不是最后一批的删除批只接在删除中的那一版后面（`answer_deletion_step` 判过）",
+                ))
+            }
+            ModelPublishKind::RowsOnFileVersion
+            | ModelPublishKind::EmptyOnFileVersion
+            | ModelPublishKind::RowsOnVersionWithoutFile
+            | ModelPublishKind::ZeroUnit
+            | ModelPublishKind::UnlinkIntent
+            | ModelPublishKind::FinalDeletionBatch
+            | ModelPublishKind::NewInode => 0,
+        };
+        slots_written += extent_lower_segment_nodes_written * (NODE_BYTES / SLOT_BYTES);
         // 实例表链多于一片时第 1 片起每一片也是一个重写的单元：`rewritten_roles` 里实例表只记一个角色，多出来的几片在这里补上。
         let instance_table_pages_after_the_first = if kind.rewrites_the_instance_table() {
             instance_table_pages_for_rows(instance_table_rows.len()) - 1
@@ -1228,7 +1375,19 @@ impl IdealModel {
             }),
             (ModelPublishKind::UnlinkIntent, None) => ModelVersionContents::FileBeingDeleted {
                 trees_written_by: key,
+                data_units_remaining: previous.contents.live_file().map_or_else(
+                    || panic!("意图那次只接在带活文件的一版后面（`answer_unlink_intent` 判过）：{:?}", previous.contents),
+                    |file| data_units_of_content(u64::try_from(file.content.len()).expect("内容长度")),
+                ),
             },
+            (ModelPublishKind::DeletionBatchKeepingDataUnits, None) => {
+                ModelVersionContents::FileBeingDeleted {
+                    trees_written_by: key,
+                    data_units_remaining: data_units_remaining_before_this_publish
+                        .expect("不是最后一批的删除批只接在删除中的那一版后面（`answer_deletion_step` 判过）")
+                        - MODEL_DELETION_BATCH_DATA_UNITS,
+                }
+            }
             (ModelPublishKind::FinalDeletionBatch, None) => ModelVersionContents::TreesAndNoFile {
                 trees_written_by: key,
             },
@@ -1249,6 +1408,7 @@ impl IdealModel {
                 | ModelPublishKind::RowsOnVersionWithoutFile
                 | ModelPublishKind::ZeroUnit
                 | ModelPublishKind::UnlinkIntent
+                | ModelPublishKind::DeletionBatchKeepingDataUnits
                 | ModelPublishKind::FinalDeletionBatch
                 | ModelPublishKind::NewInode,
                 Some(_),
@@ -1259,14 +1419,19 @@ impl IdealModel {
         } else {
             0
         };
-        let highest_transaction_number_of_its_publish = if kind.carries_a_transaction() {
-            highest_transaction_number_before_this_publish + 1
+        let transactions_of_this_publish = kind.transactions_carried(data_units_of_the_new_content);
+        let highest_transaction_number_of_its_publish = if transactions_of_this_publish > 0 {
+            highest_transaction_number_before_this_publish + transactions_of_this_publish
         } else {
             0
         };
+        // 一个事务一条记录（C310（事务切分纪律与记录数口径打架） 用户定案「一条记录装一个事务连同它的元数据」），不承载事务的发布也写一条；
+        // jsn 一条记录加一（D23（journal 的角色与格式） 已定项 7），根带的是这次发布末条记录的 jsn。`journal_counter` 是这次第一条记录的。
+        // 末条再跨记录（D23 已定项 17）只在最后一个事务要点名的项多于一条记录装得下的 64 项时才有，模型罩的发布点名的项远不到。
+        let records_of_this_publish = transactions_of_this_publish.max(1);
         ModelRoot {
             key,
-            journal_counter,
+            journal_counter: ModelJournalCounter(journal_counter.0 + records_of_this_publish - 1),
             rollback_floor,
             contents,
             instance_table_rows,
@@ -1349,12 +1514,43 @@ impl IdealModel {
     pub fn answer_publish_overwrite(
         &self,
         content: &[u8],
+    ) -> Result<ModelAnswer, ModelDisagreement> {
+        self.answer_rewriting_the_whole_file(
+            content,
+            ModelOperationKind::PublishOverwrite,
+            WholeFileWriteContract::OneDataUnit,
+        )
+    }
+
+    /// 顺序写（`transaction::publish_sequential_write`，经会话的 `UserChange::SequentialWrite`）：同覆盖写接在现行的带文件的一版后面、
+    /// 整份重写文件内容，不按单元数拒——内容切成几个数据单元就写几个（一个单元一个事务），多于一个时连同 extent 树下段一起写
+    /// （D8（核心索引结构） 已定项 14）。现行那一版有树而文件集合为空时同覆盖写必须拒。
+    ///
+    /// # Errors
+    /// 同 [`IdealModel::answer_publish_overwrite`]。
+    pub fn answer_publish_sequential_write(
+        &self,
+        content: &[u8],
+    ) -> Result<ModelAnswer, ModelDisagreement> {
+        self.answer_rewriting_the_whole_file(
+            content,
+            ModelOperationKind::PublishSequentialWrite,
+            WholeFileWriteContract::AnyNumberOfDataUnits,
+        )
+    }
+
+    /// 覆盖写与顺序写共用的答法：两者只差入口的契约（一个数据单元以内，还是不按单元数拒）。
+    fn answer_rewriting_the_whole_file(
+        &self,
+        content: &[u8],
+        operation: ModelOperationKind,
+        contract: WholeFileWriteContract,
     ) -> Result<ModelAnswer, ModelDisagreement> {
         let session = self.open_session()?;
         let current = &session.current;
-        if let Some(refused) = self.answer_refusing_a_change_on_a_version_with_trees_and_no_file(
-            ModelOperationKind::PublishOverwrite,
-        ) {
+        if let Some(refused) =
+            self.answer_refusing_a_change_on_a_version_with_trees_and_no_file(operation)
+        {
             return Ok(refused);
         }
         if current.contents.live_file().is_none() {
@@ -1364,13 +1560,19 @@ impl IdealModel {
                     "模型里现行版本 {:?} 没有读得出的文件：{:?}",
                     current.key, current.contents
                 ),
-                "实现的现行版本带文件、队列空（执行器调了覆盖写）".to_string(),
+                "实现的现行版本带文件、队列空（执行器调了覆盖写或顺序写）".to_string(),
             ));
         }
         let mut required_refusals = BTreeSet::new();
-        if u64::try_from(content.len()).expect("内容长度") > data_unit_payload_capacity_in_bytes()
-        {
-            required_refusals.insert(ModelRefusalReason::ContentExceedsDataUnitPayload);
+        match contract {
+            WholeFileWriteContract::OneDataUnit => {
+                if u64::try_from(content.len()).expect("内容长度")
+                    > data_unit_payload_capacity_in_bytes()
+                {
+                    required_refusals.insert(ModelRefusalReason::ContentExceedsDataUnitPayload);
+                }
+            }
+            WholeFileWriteContract::AnyNumberOfDataUnits => {}
         }
         let root = self.next_root(
             current,
@@ -1382,12 +1584,7 @@ impl IdealModel {
             Some(content),
             Rc::clone(&current.instance_table_rows),
         );
-        Ok(self.answer_for_publishes(
-            ModelOperationKind::PublishOverwrite,
-            required_refusals,
-            BTreeSet::new(),
-            vec![root],
-        ))
+        Ok(self.answer_for_publishes(operation, required_refusals, BTreeSet::new(), vec![root]))
     }
 
     /// 零单元发布（`transaction::publish_without_units`）：接在现行的树表 0 条的一版后面。
@@ -1423,7 +1620,7 @@ impl IdealModel {
     }
 
     /// unlink 第一个文件的意图那次（`mounted_session::UserChange::Unlink`，D8（核心索引结构） 已定项 16 第 1 条）：接在现行那一版后面，
-    /// inode 1 的记录 nlink 写 0、`deleted_inodes` 加 key、墓碑树追加一条；这一版读回「没有文件」。会话接着走空（最后一批由
+    /// inode 1 的记录 nlink 写 0、`deleted_inodes` 加 key、墓碑树追加一条；这一版读回「没有文件」。会话接着走空（每一批由
     /// [`IdealModel::answer_deletion_step`] 答）；两次之间会话为空间推的抬 F 由执行器按 txg 次序插进来比。逐次发布判墙。
     ///
     /// # Errors
@@ -1465,14 +1662,19 @@ impl IdealModel {
     }
 
     /// 走空 `deleted_inodes` 的一步（D8（核心索引结构） 已定项 16 第 2、6 条：会话里 unlink 之后、每次用户改动之前，挂载续做）：
-    /// 模型的文件只有一个数据单元，删除中的那一版的下一步就是最后一批，之后这一版有树而文件集合为空。
+    /// 删除中的那一版这个文件还剩的数据单元多于产品批大小 144 就是一个不是最后一批的删除批（从尾部释放 144 个，之后仍是删除中的那一版），
+    /// 不多于就是最后一批，之后这一版有树而文件集合为空。下一步只由现行那一版定，挂载续做与一路做下来同一个答法。
     ///
     /// # Errors
     /// 模型里没有可写会话，或现行版本没有删除中的文件（执行器只在实现交回了一步删除时问）：会话状态与实现对不上。
     pub fn answer_deletion_step(&self) -> Result<ModelAnswer, ModelDisagreement> {
         let session = self.open_session()?;
         let current = &session.current;
-        let ModelVersionContents::FileBeingDeleted { .. } = current.contents else {
+        let ModelVersionContents::FileBeingDeleted {
+            data_units_remaining,
+            ..
+        } = current.contents
+        else {
             return Err(ModelDisagreement::new(
                 ModelDisagreementAspect::SessionState,
                 format!(
@@ -1482,12 +1684,17 @@ impl IdealModel {
                 "实现交回了走空 deleted_inodes 的一步".to_string(),
             ));
         };
+        let step = if data_units_remaining > MODEL_DELETION_BATCH_DATA_UNITS {
+            ModelPublishKind::DeletionBatchKeepingDataUnits
+        } else {
+            ModelPublishKind::FinalDeletionBatch
+        };
         let root = self.next_root(
             current,
             session.instance,
             ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
             ModelJournalCounter(self.highest_journal_counter.0 + 1),
-            ModelPublishKind::FinalDeletionBatch,
+            step,
             current.rollback_floor,
             None,
             Rc::clone(&current.instance_table_rows),
@@ -2167,6 +2374,13 @@ impl IdealModel {
                 }
                 self.judge_effect(answer, effect, &mut counts)?;
                 counts.successes_matched += 1;
+                if answer.operation == ModelOperationKind::PublishDeletionStep
+                    && answer.expected_roots.iter().any(|root| {
+                        matches!(root.contents, ModelVersionContents::FileBeingDeleted { .. })
+                    })
+                {
+                    counts.deletion_batches_keeping_data_units_matched += 1;
+                }
                 self.advance_by_success(answer);
                 Ok(counts)
             }
@@ -3435,4 +3649,83 @@ mod tests {
             ModelDisagreementAspect::SucceededWhenModelRequiresRefusal
         );
     }
+
+    /// 单元区墙的区间按单元区起点随环长走算（C475（非默认环长下单元区起点取编译期常量），实审 A3b 第 15 条）：6 MiB 的环，单元区从槽 1408 起；
+    /// 盘只比默认环长那一档的单元区起点（编译期常量那个 50176）多 1000 槽。第一个文件之后一次覆盖写的上界乘 64 落在两道墙之间：
+    /// 按随环长走的起点，单元区有四万九千多槽，离墙还远，实现报单元区墙就是对不上；按编译期常量取起点，单元区只剩 1000 槽，同一个拒绝落在区间里放行。
+    /// 随机历史的单元区墙取样点（两块单元区 256 槽的小盘、准入关掉的那一段）一次单元区墙的拒绝都没有，两个起点在它上面判得一样（取样点不敏感）。
+    #[test]
+    fn a_unit_area_wall_refusal_far_below_the_wall_that_follows_the_ring_is_judged_a_disagreement()
+    {
+        let journal_ring_bytes = 6 << 20;
+        let slots_past_the_unit_area_start_of_the_default_ring = 1000;
+        let device_slots = model_unit_area_start_slot(JOURNAL_RING_DEFAULT_BYTES)
+            + slots_past_the_unit_area_start_of_the_default_ring;
+        let mut model = IdealModel::after_make_filesystem_on_a_journal_ring_of(
+            ModelPoolGeometry {
+                devices: vec![ModelDeviceIdentity(0), ModelDeviceIdentity(1)],
+                device_size_in_bytes: device_slots * SLOT_BYTES,
+            },
+            journal_ring_bytes,
+        );
+        assert_eq!(
+            model.unit_area_slots_per_device(),
+            device_slots - 1408,
+            "6 MiB 的环末尾的下一个槽 1024 + 384 = 1408 起"
+        );
+        model.acquire_and_warm_up_in_the_make_filesystem_process();
+        let first = model
+            .answer_publish_first_file(&[3])
+            .expect("会话开着、现行 txg 2");
+        succeed(&mut model, &first);
+        let overwrite = model
+            .answer_publish_overwrite(&[4])
+            .expect("会话开着、现行版本带文件");
+        let planned_slots = overwrite.planned_publish_upper_bounds[0].occupied_slots_per_device;
+        assert!(
+            planned_slots * CLUSTER_SEGMENT_SLOTS
+                > slots_past_the_unit_area_start_of_the_default_ring
+                && planned_slots * CLUSTER_SEGMENT_SLOTS <= device_slots - 1408,
+            "取样点：上界 {planned_slots} 槽乘 64 落在两道墙之间"
+        );
+        let disagreement = model
+            .judge_and_advance(
+                &overwrite,
+                &ObservedOutcome::Refused {
+                    member: "PublishError::PlacementRefused(NoPlacementOnAnyDevice)".to_string(),
+                    reason: ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall),
+                    publishes_completed: 0,
+                    wrote_anything: false,
+                    reported_ceiling: None,
+                    reported_root_ring_slot_still_bad_after_one_reread: None,
+                },
+            )
+            .expect_err("离墙还远：单元区墙的拒绝不在允许的区间里");
+        assert_eq!(
+            disagreement.aspect,
+            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess
+        );
+    }
+
+    /// extent 树下段的节点数与层数（D8（核心索引结构） 已定项 14：一个单元内联、不建下段；叶罩 144 个单元、内部扇出 147）：
+    /// 模型按它给顺序写与删除批的占槽上界。
+    #[test]
+    fn the_lower_segment_of_a_file_has_a_leaf_per_144_data_units_and_an_internal_node_per_147_children(
+    ) {
+        assert_eq!(
+            [1, 2, 144, 145, 288, 289].map(|data_units| (
+                extent_lower_segment_nodes(data_units),
+                extent_lower_segment_levels(data_units)
+            )),
+            [(0, 0), (1, 1), (1, 1), (2 + 1, 2), (2 + 1, 2), (3 + 1, 2)]
+        );
+        assert_eq!(
+            (
+                extent_lower_segment_nodes(144 * 147 + 1),
+                extent_lower_segment_levels(144 * 147 + 1)
+            ),
+            (148 + 2 + 1, 3),
+            "148 片叶要两个第一层节点，再一个根"
+        );
+    }
 }
diff --git a/crates/singlefs-harness/src/model_comparison.rs b/crates/singlefs-harness/src/model_comparison.rs
index dd1002c..e847dfa 100644
--- a/crates/singlefs-harness/src/model_comparison.rs
+++ b/crates/singlefs-harness/src/model_comparison.rs
@@ -62,22 +62,15 @@ pub fn model_root_key(instance: InstanceGeneration, checkpoint_txg: CheckpointTx
 #[must_use]
 pub fn model_unit_role(unit: TransactionUnit) -> Option<ModelUnitRole> {
     let role = match unit {
-        // 模型罩的发布（第一个文件版本 / 覆盖写、写行、暖机空发布）写的文件恒只有一个数据单元：多单元只会从
-        // `publish_sequential_write` 那条路径出来，而随机历史与层 0 的固定脚本一次都不调它。
-        TransactionUnit::Data(index) => {
-            assert_eq!(
-                index,
-                DataUnitIndexInFile::FIRST,
-                "模型今天只罩一个数据单元的文件"
-            );
-            ModelUnitRole::Data
-        }
-        // 模型罩的文件恒一个数据单元、只有 inode 1：extent 树上段只有根兼叶，那个单元内嵌在它的条目里（D8（核心索引结构） 已定项 14），
-        // 没有下段、上段也没有根之下的节点。
+        // 文件的每个数据单元都是这一个角色：写内容的发布（第一个文件版本、覆盖写、顺序写）整份重写、单元同一个分配代，
+        // 不是最后一批的删除批留下的单元照抄（代码轮 U13：随机历史经会话的顺序写长出多于一片下段叶的文件）。
+        TransactionUnit::Data(_) => ModelUnitRole::Data,
+        // 模型只罩 inode 1：extent 树上段只有根兼叶（D8（核心索引结构） 已定项 14）；一个单元的文件那个指针内嵌在它的条目里，
+        // 多个单元的文件条目指着下段根。
         TransactionUnit::ExtentRoot => ModelUnitRole::ExtentRoot,
-        TransactionUnit::ExtentLowerNode(position) => {
-            panic!("模型今天只罩一个数据单元的文件（内嵌、没有下段）：{position:?}")
-        }
+        // 下段的节点不对应角色（同分配记录树根之下的节点）：一次删除批只重写被截掉那一片下段叶与它的祖先、别的照抄，
+        // 一版里下段各节点的分配代不止一个，模型不记是哪几片；它们的分配代、位置与记录归池级 checker 判，占槽模型按层数现算上界。
+        TransactionUnit::ExtentLowerNode(_) => return None,
         TransactionUnit::ExtentUpperNodeBelowTheRoot(position) => {
             panic!("模型今天只罩 inode 1 一个文件（上段只有根兼叶）：{position:?}")
         }
@@ -163,7 +156,8 @@ pub fn observed_root_of_file_version(output: &TransactionOutput) -> ObservedRoot
     }
 }
 
-/// 这一版的文件内容：文件第 0 个数据单元（模型只罩一个数据单元的文件）的载荷，按单元头里的声明长度取。
+/// 这一版的文件内容：文件第 0、1、2…… 个数据单元的载荷（各按单元头里的声明长度取）按单元号接起来；单元号要从 0 起连着、
+/// 每个号恰好一个单元，不然解不出。
 /// inode 1 不是活记录（删除中的记录 nlink = 0、删完树里没有它，D8（核心索引结构） 已定项 16 第 4 条）时这一版没有读得出的文件：
 /// 意图那次之后数据单元还在，按单元取会读出被删的内容。
 #[must_use]
@@ -175,23 +169,45 @@ pub fn observed_file_of_file_version(output: &TransactionOutput) -> ObservedFile
         }
         Some(_) => {}
     }
-    let Some(data_unit) = output
+    let mut data_units: Vec<(u64, &[u8])> = output
         .units
         .iter()
-        .find(|unit| unit.identity == TransactionUnit::Data(DataUnitIndexInFile::FIRST))
-    else {
+        .filter_map(|unit| {
+            if let TransactionUnit::Data(DataUnitIndexInFile(index)) = unit.identity {
+                Some((index, unit.bytes.as_slice()))
+            } else {
+                None
+            }
+        })
+        .collect();
+    data_units.sort_by_key(|(index, _)| *index);
+    if data_units.is_empty() {
         return ObservedFile::Undecodable {
             what: "这一版的 units 里没有文件第 0 个数据单元".to_string(),
         };
-    };
-    match parse_data_unit(&data_unit.bytes)
-        .and_then(|header| data_unit_payload(&data_unit.bytes, header.declared_length))
-    {
-        Ok(payload) => ObservedFile::Decoded(payload.to_vec()),
-        Err(error) => ObservedFile::Undecodable {
-            what: format!("文件第 0 个数据单元解不开：{error:?}"),
-        },
     }
+    let mut content = Vec::new();
+    // 迭代上界：这一版的数据单元数；跨轮只往内容末尾追加；提前出口两个：单元号断了或重了、一个单元解不开。
+    for (expected_index, (index, bytes)) in (0_u64..).zip(&data_units) {
+        if *index != expected_index {
+            return ObservedFile::Undecodable {
+                what: format!(
+                    "这一版的数据单元号不是从 0 起连着的：第 {expected_index} 个位置上是 {index}"
+                ),
+            };
+        }
+        match parse_data_unit(bytes)
+            .and_then(|header| data_unit_payload(bytes, header.declared_length))
+        {
+            Ok(payload) => content.extend_from_slice(payload),
+            Err(error) => {
+                return ObservedFile::Undecodable {
+                    what: format!("文件第 {index} 个数据单元解不开：{error:?}"),
+                }
+            }
+        }
+    }
+    ObservedFile::Decoded(content)
 }
 
 /// 实例表链上一片的角色是第几片。
diff --git a/crates/singlefs-harness/tests/fault_injection_fast_tier.rs b/crates/singlefs-harness/tests/fault_injection_fast_tier.rs
index e11889d..61025f4 100644
--- a/crates/singlefs-harness/tests/fault_injection_fast_tier.rs
+++ b/crates/singlefs-harness/tests/fault_injection_fast_tier.rs
@@ -480,40 +480,51 @@ fn replay_line_of_a_batch_unlike_the_large_tier_defaults_runs_the_same_campaign_
 }
 
 /// 说谎的设备许可留下的盘面不一致里 `["I-3.1"]` 那一组的取样点（`fault_injection::INCONSISTENCIES_A_LYING_DEVICE_MAY_LEAVE`；
-/// `crates/mutations.tsv` 钉着把那一组去掉的变异）。快档那 24 段里说谎的设备留下的不一致全落在另一组（`["I-2.1", "I-4.8", "I-7.4"]`），
-/// 那一组去掉了快档照样绿；大档（这个测试周期的种子基起 512 段、每段 30 步、注入 6 次）里 `["I-3.1"]` 那一组落在 14 个注入点上
-/// （2026-09-26 在管理员回退改成挂着时的向前发布之后的代码上扫的：比重表里关着时回退并进可写挂载，同一个种子生成的历史变了，
-/// 原先取的种子 7463871032432355306 那一段里不再有这一组）。
-/// 这里只跑其中一段：种子 7463871032432355210，同一组规模下抽出的 6 个注入点里有一次被吞掉的写（`write_is_swallowed`，
-/// 整池第 620 次写调用、`step_index` 25 那一步覆盖写 33 次写里的第 31 次，即这次发布的根槽写）落在覆盖写上，之后池级 checker 只判红 I-3.1。
-/// 它要被白名单豁免、记进说谎那一格，不算新发现（这一格为什么只剩 I-3.1，没有逐字节追过）。
+/// `crates/mutations.tsv` 钉着把被吞的写插回去那一步拿掉的变异）。快档那 24 段里说谎的设备留下的不一致全落在另一组
+/// （`["I-2.1", "I-4.8", "I-7.4"]`），这一组要单独钉。
+///
+/// 取样点写死成一段历史与一个注入点，不按种子从比重表里抽：此前钉的是种子 7463871032432355210 在 `GenerationWeights::BROAD` 上抽出的
+/// 第 25 步覆盖写那次根槽写，删除批把比重表里可写挂载那一格切出 unlink 之后同一个种子抽出的历史变了、那次被吞的写不在了，用例跟着红
+/// （调度记录 `records/2026-10-01-删除与并发事务.md` 第 11 批的慢档红调查问 1；比重表一改，种子钉的取样点就会再丢一次）。
+/// 这一段：第一个文件（txg 3）之后覆盖写五次（txg 4–8），txg 7 那次（第 3 步）的根槽 FUA 写被吞；每一步之后照跑池级 checker，
+/// txg 8 那次覆盖写之后 checker 只判红 I-3.1（被吞的根槽留下的，被吞的写插回去就消失）。它要按注入点认成说谎的设备留下的、
+/// 记进说谎那一格，不算新发现——带着注入跑的那一遍里每一步之后的池级 checker 判红那一处、注入之后的镜像上再跑的那一遍，各记一次。
 #[test]
-#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
 fn swallowed_write_after_which_the_checker_flags_only_the_allocated_statistic_invariant_is_excused_as_what_the_lying_device_may_leave(
 ) {
-    let report = run_fault_injection_campaign(&FaultInjectionCampaign {
-        first_seed: 7_463_871_032_432_355_210,
-        seed_count: 1,
-        operations_per_history: 30,
-        faults_per_history: 6,
-        weights: GenerationWeights::BROAD,
-        execution: CHECKED_AFTER_EVERY_STEP,
-        worker_threads: FaultInjectionWorkerThreads::from_the_environment(),
-    });
-    let rendered = report.render();
+    const STEP_OF_THE_SWALLOWED_ROOT_SLOT_WRITE: usize = 3;
+    let history = GeneratedHistory {
+        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations: (1..=5).map(overwrite_with_fill_seed).collect(),
+    };
+    let injection = inject_one_fault_into_the_segment(
+        &history,
+        CHECKED_AFTER_EVERY_STEP,
+        InjectedFault::WriteIsSwallowed,
+        FaultedSegment::Operation {
+            step_index: STEP_OF_THE_SWALLOWED_ROOT_SLOT_WRITE,
+            operation_kind: HistoryOperationKind::PublishOverwrite,
+        },
+        ordinal_of_the_root_slot_write_within_the_step(
+            &history,
+            STEP_OF_THE_SWALLOWED_ROOT_SLOT_WRITE,
+        ),
+    );
     assert!(
-        report.new_findings.is_empty(),
-        "说谎的设备留下的 I-3.1 要被白名单豁免，不算新发现；{}\n{rendered}",
-        how_to_replay(&report)
+        injection.new_findings.is_empty(),
+        "说谎的设备留下的 I-3.1 要被白名单豁免，不算新发现：{:?}",
+        injection.new_findings
     );
     assert_eq!(
-        report
+        injection
             .tally
             .lying_device_signatures
             .get("CheckerViolations { invariants: [\"I-3.1\"] }"),
         Some(&2),
         "这一段里被吞掉的那一次写之后只判红 I-3.1，记进说谎那一格——带着注入跑的那一遍里每一步之后的池级 checker 判红那一处、\
-         注入之后的镜像上再跑的那一遍，各记一次：\n{rendered}"
+         注入之后的镜像上再跑的那一遍，各记一次：{:?}",
+        injection.tally.lying_device_signatures
     );
 }
 
diff --git a/crates/singlefs-harness/tests/random_histories.rs b/crates/singlefs-harness/tests/random_histories.rs
index d71c2e2..4ca6bbf 100644
--- a/crates/singlefs-harness/tests/random_histories.rs
+++ b/crates/singlefs-harness/tests/random_histories.rs
@@ -1950,3 +1950,166 @@ fn a_random_history_seed_count_of_zero_panics_naming_the_variable_and_the_value(
 fn a_random_history_worker_thread_count_of_zero_panics_naming_the_variable_and_the_value() {
     let _threads = worker_threads_from_the_value(Some("0"));
 }
+
+/// 删除多于一片下段叶的文件那一格（代码轮 U13）的取样点：种子区间与每段步数，写死。
+const DELETION_SAMPLING_FIRST_SEED: u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
+const DELETION_SAMPLING_SEEDS: u64 = 48;
+
+const DELETION_SAMPLING: HistorySampling = HistorySampling {
+    operations_per_history: 30,
+    weights: GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES,
+    execution: HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+};
+
+/// 删除多于一片下段叶的文件那一格（代码轮第一轮判决 U13：模型罩多个数据单元的文件、随机历史里出现 ≥ 2 片下段叶的删除批；
+/// 第二轮判决第二节 N3「走空多步」）：比重取 `GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES`，种子与步数见常量。
+/// 覆盖写的内容另抽跨几个单元的、经会话的顺序写发（多半两片下段叶），unlink 之后建 inode、多回退到第四新的根（意图那一版或第一个删除批那一版）、
+/// 再建 inode、覆盖写或 unlink：会话先把队列走空。先判没有新发现，再核这一格真的跑到了：两片下段叶的文件发成过、模型比过不是最后一批的删除批的段数
+/// 与会话改动之前先走空走了至少两步的段数都不为 0（「走空被空间拒」在 4 GiB 的盘上走不到，只打数不判）。
+#[test]
+#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
+fn random_histories_deleting_files_of_several_lower_leaves_compare_every_batch_and_the_walks_before_a_change(
+) {
+    let started = std::time::Instant::now();
+    let report = run_history_campaign(
+        DELETION_SAMPLING_FIRST_SEED,
+        DELETION_SAMPLING_SEEDS,
+        DELETION_SAMPLING.operations_per_history,
+        &DELETION_SAMPLING.weights,
+        DELETION_SAMPLING.execution,
+        worker_threads_by_default(),
+        FindingShrinking::ReportSeedsOnly,
+    );
+    let rendered = report.render();
+    print_uncaptured(&format!(
+        "── 随机历史：偏向删除多于一片下段叶的文件的取样点 ──\n{rendered}用时 {:.1} 秒\n",
+        started.elapsed().as_secs_f64()
+    ));
+    assert!(
+        report.new_findings.is_empty(),
+        "「已知红」清单外的失败：\n{rendered}"
+    );
+    let two_lower_leaves = ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
+        data_units: 147,
+        selector: 0,
+    };
+    assert!(
+        report
+            .tally
+            .content_lengths_published
+            .get(two_lower_leaves.name())
+            .copied()
+            .unwrap_or(0)
+            >= 1,
+        "两片下段叶的文件一次都没经顺序写发成：\n{rendered}"
+    );
+    assert!(
+        report
+            .tally
+            .histories_with_a_deletion_batch_keeping_data_units
+            >= 1,
+        "没有一段历史里模型比过不是最后一批的删除批（≥ 2 片下段叶）：\n{rendered}"
+    );
+    assert!(
+        report
+            .tally
+            .histories_with_a_walk_of_several_steps_before_a_user_change
+            >= 1,
+        "没有一段历史里会话改动之前先走空走了至少两步：\n{rendered}"
+    );
+}
+
+/// 两片下段叶的文件（150 个数据单元，`UserChange::SequentialWrite` 写的）经会话 unlink：意图那次、一个不是最后一批的删除批
+/// （释放尾部 144 个，留 6 个）、最后一批，三版逐版交模型比（代码轮 U13）。判别力：模型把文件当一个数据单元（意图之后的下一步就是最后一批）时，
+/// 第一个删除批交回的这一版还带着数据单元、inode 叶与 `deleted_inodes` 根，模型那一版已经没有这几个角色，判「单元的分配代」对不上。
+#[test]
+fn deleting_a_file_of_two_lower_leaves_publishes_a_batch_keeping_data_units_that_the_model_compares(
+) {
+    let history = GeneratedHistory {
+        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations: vec![
+            HistoryOperation::PublishOverwrite(ContentChoice {
+                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
+                    data_units: 150,
+                    selector: 7,
+                },
+                fill_seed: 11,
+            }),
+            HistoryOperation::Unlink,
+        ],
+    };
+    let run = execute_history(&history);
+    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
+    assert!(
+        run.outcomes
+            .iter()
+            .all(|outcome| matches!(outcome, StepOutcome::Applied(_))),
+        "顺序写与 unlink 都做成：{:?}",
+        run.outcomes
+    );
+    let counts = &run.tally.model_counts;
+    assert_eq!(
+        (
+            counts.deletion_batches_keeping_data_units_matched,
+            run.tally.histories_with_a_deletion_batch_keeping_data_units,
+            counts.file_contents_compared,
+        ),
+        (1, 1, 2),
+        "一个不是最后一批的删除批比过；带文件的两版（起点的第一个文件、150 个单元的那一版）各比过一次整份内容"
+    );
+}
+
+/// 两片下段叶的文件 unlink 之后建一个 inode（删完的那一版没有文件、回退不接，建过 inode 的才接），再回退到 unlink 意图那一版
+/// （第四新的根：建 inode、最后一批、删除批、意图），号回到队列；随后再建一个 inode，会话先走空——一个删除批、最后一批两步——再建 inode，
+/// 三版逐版交模型比（第二轮判决第二节 N3「走空多步」）。判别力：模型把删除中的那一版的下一步一律当最后一批时，走空的第一步交回的这一版
+/// 还带着数据单元，判「单元的分配代」对不上。
+#[test]
+fn deleting_a_file_of_two_lower_leaves_then_rolling_back_to_its_intent_walks_two_steps_before_the_next_change(
+) {
+    let history = GeneratedHistory {
+        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations: vec![
+            HistoryOperation::PublishOverwrite(ContentChoice {
+                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
+                    data_units: 150,
+                    selector: 7,
+                },
+                fill_seed: 11,
+            }),
+            HistoryOperation::Unlink,
+            HistoryOperation::CreateInode,
+            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
+                index_from_newest: 3,
+            }),
+            HistoryOperation::CreateInode,
+        ],
+    };
+    let run = execute_history(&history);
+    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
+    assert!(
+        matches!(
+            run.outcomes[3],
+            StepOutcome::Applied(AppliedEffect::RolledBack { .. })
+        ),
+        "回退到意图那一版做成：{:?}",
+        run.outcomes[3]
+    );
+    assert!(
+        matches!(run.outcomes[4], StepOutcome::Applied(_)),
+        "走空之后建 inode 做成：{:?}",
+        run.outcomes[4]
+    );
+    let counts = &run.tally.model_counts;
+    assert_eq!(
+        (
+            counts.walks_of_several_steps_before_a_user_change_matched,
+            run.tally
+                .histories_with_a_walk_of_several_steps_before_a_user_change,
+            counts.deletion_batches_keeping_data_units_matched,
+        ),
+        (1, 1, 2),
+        "建 inode 之前走空两步比过一次；不是最后一批的删除批 unlink 里一个、走空里一个"
+    );
+}
diff --git a/crates/singlefs-harness/tests/a_writable_mount_with_system_configuration_slot_zero_unreadable.rs b/crates/singlefs-harness/tests/a_writable_mount_with_system_configuration_slot_zero_unreadable.rs
new file mode 100644
index 0000000..d7fc26f
--- /dev/null
+++ b/crates/singlefs-harness/tests/a_writable_mount_with_system_configuration_slot_zero_unreadable.rs
@@ -0,0 +1,453 @@
+//! 系统配置见证读取 R2（C331（择根倒挂压过已确认的写），D22（单元原子性怎么合成） 已定项 8 射程，用户 2026-10-03 定「改取 R2」）在
+//! 槽 0（偶数世代）那一侧的可写挂载与取号（遗留改动代码轮判决 `research/prompts/m3-leftover-code-r1-main-verification.md` 第三节 LR1）。
+//!
+//! 新池新建文件之后每块盘两槽是世代 4（槽 0）、世代 5（槽 1）；再可写挂载一次，系统配置轮换两次（世代 7 落在槽 1、世代 8 落在槽 0），
+//! 最新的世代 8，8 mod 2 = 0，在槽 0。
+//! `system_configuration_witness_is_undecidable_only_when_a_slot_reports_a_read_error_and_no_device_verified_both.rs` 里写坏的都是世代 5
+//! 那一槽（槽 1）：见证读与取号只看每块盘的槽 1 的错实现在那几条上全过（攻方变异 slot1-only-both）。这里写坏的是最新的世代 8 那一槽（槽 0）：
+//! - 两块盘槽 0 都报读错：有槽报读错、没有一块盘两槽都自证过 ⇒ 判不出，可写挂载重读一次仍判不出、取号之前拒，取号那一刻也拒；
+//! - 盘 0 槽 0 读回全零（撕裂的轮换）、盘 1 槽 0 报读错：报读错的是第二块盘，同样判不出、拒可写；
+//! - 读阶段两槽都读得出、到取号之前的逐盘核那一遍盘 1 两槽才读不出（两次读之间的瞬时读错）：逐盘核交 `Err(盘 1)`，照见证读的拒因报。
+mod common;
+
+use std::cell::RefCell;
+use std::rc::Rc;
+
+use common::{
+    build_pool, disk_snapshot, parameters, with_unreadable_ranges, without_unreadable_ranges,
+    BuiltPool, FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange,
+    UnreadableRangeReadBack,
+};
+use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
+use singlefs_core::block_device::{
+    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
+};
+use singlefs_core::mount::{
+    mount_writable, MountError, Mounted, StillUnreadableAfterOneReread, WitnessedCounterComparison,
+};
+use singlefs_core::mounted_read::mount_read_only;
+use singlefs_core::recovery::PoolReader;
+use singlefs_core::system_configuration::SystemConfiguration;
+use singlefs_core::transaction::{acquire_instance, InstanceAcquisitionFailed, PoolWriter};
+use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
+
+/// 新池新建文件之后可写挂载一次、系统配置轮换两次之后最新的世代号（新池新建文件那次是 5）：落在槽 8 mod 2 = 0。
+const NEWEST_GENERATION_AFTER_THE_WRITABLE_MOUNT: u64 = 8;
+
+const DEVICE_ZERO: DeviceIdentity = DeviceIdentity(0);
+const DEVICE_ONE: DeviceIdentity = DeviceIdentity(1);
+
+fn slot_spacing_in_bytes() -> u64 {
+    u64::from(parameters().geometry.fixed_structure_slot_spacing)
+}
+
+/// 最新的世代 8 那一槽在盘上的偏移：槽 0。
+fn offset_of_the_newest_slot() -> u64 {
+    (NEWEST_GENERATION_AFTER_THE_WRITABLE_MOUNT % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
+        * slot_spacing_in_bytes()
+}
+
+/// `device` 那一槽（偏移 `offset_in_bytes`）读出来自证得过的系统配置的世代号；读不出或自证不过交 `None`。
+fn self_verified_generation_at(
+    pool: &BuiltPool,
+    device: DeviceIdentity,
+    offset_in_bytes: u64,
+) -> Option<u64> {
+    PoolReader::read(
+        &pool.memory_pool(),
+        device,
+        DeviceOffsetInBytes(offset_in_bytes),
+        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096"),
+    )
+    .and_then(|bytes| SystemConfiguration::parse_slot(&bytes).ok())
+    .map(|system_configuration| system_configuration.quantities.slot_generation)
+}
+
+/// 新池新建文件之后照常可写挂载一次（取到实例 2，系统配置轮换两次：世代 7 进每块盘的槽 1、世代 8 进槽 0），会话随即丢掉（进程退出）。
+/// 交回之前核这一形：每块盘槽 0 是世代 8、槽 1 是世代 7——最新的那一槽是槽 0。
+fn pool_whose_newest_system_configuration_slot_is_slot_zero(tag: &str) -> BuiltPool {
+    let mut pool = build_pool(tag);
+    let mut devices = pool.reopen_recorded();
+    let mounted =
+        mount_writable(&parameters(), &mut devices).expect("新池新建文件之后照常可写挂载");
+    assert_eq!(mounted.output.instance, InstanceGeneration(2), "取到实例 2");
+    drop(mounted);
+    pool.devices = Some(devices);
+    for device in [DEVICE_ZERO, DEVICE_ONE] {
+        assert_eq!(
+            (
+                self_verified_generation_at(&pool, device, 0),
+                self_verified_generation_at(&pool, device, slot_spacing_in_bytes()),
+            ),
+            (
+                Some(NEWEST_GENERATION_AFTER_THE_WRITABLE_MOUNT),
+                Some(NEWEST_GENERATION_AFTER_THE_WRITABLE_MOUNT - 1)
+            ),
+            "盘 {device:?}：槽 0 是最新的世代 8、槽 1 是世代 7"
+        );
+    }
+    pool
+}
+
+/// `device` 最新的世代 8 那一槽（槽 0），每次读都报读错。
+fn slot_zero_reporting_a_read_error_on(device: DeviceIdentity) -> UnreadableRange {
+    UnreadableRange {
+        device,
+        offset_in_bytes: offset_of_the_newest_slot(),
+        length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
+        failing_reads: FailingReadsOfARange::Every,
+    }
+}
+
+/// 重开、包上 `ranges` 的读故障可写挂载一次，交回结局；盘拆掉读故障交回池。
+fn writable_mount_with_unreadable_ranges(
+    pool: &mut BuiltPool,
+    ranges: Vec<UnreadableRange>,
+) -> Result<Mounted, MountError> {
+    let shared = SharedUnreadableRanges::new(ranges, UnreadableRangeReadBack::DeviceError);
+    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &shared);
+    let outcome = mount_writable(&parameters(), &mut devices);
+    pool.devices = Some(without_unreadable_ranges(devices));
+    outcome
+}
+
+/// 拒因：判据 N-配置 为真、重读仍为真，两遍读数都判 R2 的判不出。
+fn refused_as_undecidable_on_both_reads(outcome: &Result<Mounted, MountError>) -> bool {
+    let Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)) = outcome else {
+        return false;
+    };
+    let StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
+        first_read,
+        reread,
+    } = still_unreadable.as_ref()
+    else {
+        return false;
+    };
+    [first_read, reread].iter().all(|reading| {
+        reading.witness.comparison
+            == WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots
+    })
+}
+
+fn describe(outcome: &Result<Mounted, MountError>) -> String {
+    match outcome {
+        Ok(mounted) => format!("挂载做成：取到实例 {:?}", mounted.output.instance),
+        Err(error) => format!("{error:?}"),
+    }
+}
+
+/// 场景：最新的系统配置在槽 0（世代 8），两块盘的槽 0 都报读错。
+/// 预期：有槽报读错、没有一块盘两槽都自证过 ⇒ 判不出，重读一次仍判不出，取号之前拒可写；系统配置槽、根环、录制流步数都不变；只读挂载照常。
+/// 判别力：见证读与取号只看每块盘的槽 1 的错实现看不见槽 0 的读错、两槽算一槽，判得出、照常挂上取到实例 3。
+#[test]
+fn a_writable_mount_is_refused_when_slot_zero_of_every_device_reports_a_read_error() {
+    let mut pool = pool_whose_newest_system_configuration_slot_is_slot_zero(
+        "lr1-slot-zero-read-error-every-device",
+    );
+    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
+    let outcome = writable_mount_with_unreadable_ranges(
+        &mut pool,
+        vec![
+            slot_zero_reporting_a_read_error_on(DEVICE_ZERO),
+            slot_zero_reporting_a_read_error_on(DEVICE_ONE),
+        ],
+    );
+    assert!(
+        refused_as_undecidable_on_both_reads(&outcome),
+        "两块盘最新那一槽（槽 0）都报读错：判不出，拒可写；得到 {}",
+        describe(&outcome)
+    );
+    assert_eq!(
+        disk_snapshot(&pool.memory_pool(), &pool.stream),
+        before,
+        "拒在取号之前：系统配置槽、根环、录制流步数都不变"
+    );
+    mount_read_only(&pool.memory_pool()).expect("只读挂载照常");
+}
+
+/// 场景：同上一形的盘，不经可写挂载直接取号，两块盘的槽 0 都报读错。
+/// 预期：取号那一刻同一判（判不出），在第一个取号写之前拒，报盘表次序第一块报读错的盘（盘 0）；盘上不变。
+/// 判别力：取号只看槽 1 时取号照做、写进实例 3。
+#[test]
+fn an_acquisition_is_refused_before_any_write_when_slot_zero_of_every_device_reports_a_read_error()
+{
+    let mut pool =
+        pool_whose_newest_system_configuration_slot_is_slot_zero("lr1-slot-zero-acquisition");
+    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
+    let shared = SharedUnreadableRanges::new(
+        vec![
+            slot_zero_reporting_a_read_error_on(DEVICE_ZERO),
+            slot_zero_reporting_a_read_error_on(DEVICE_ONE),
+        ],
+        UnreadableRangeReadBack::DeviceError,
+    );
+    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &shared);
+    let pool_parameters = parameters();
+    let acquisition = {
+        let mut writer = PoolWriter::new(&pool_parameters, devices.as_mut_slice());
+        acquire_instance(&mut writer)
+    };
+    pool.devices = Some(without_unreadable_ranges(devices));
+    assert!(
+        matches!(
+            acquisition,
+            Err(InstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
+                first_device_reporting_a_read_error: DEVICE_ZERO,
+            })
+        ),
+        "判不出：取号在任何写之前拒，报盘 0；得到 {acquisition:?}"
+    );
+    assert_eq!(
+        disk_snapshot(&pool.memory_pool(), &pool.stream),
+        before,
+        "拒在第一个取号写之前：盘上不变"
+    );
+}
+
+/// 场景：最新那一槽（槽 0）在盘 0 读回全零（撕裂的轮换：读得出、自证不过，设备没报错），在盘 1 报读错。
+/// 预期：报读错的是第二块盘，有槽报读错、没有一块盘两槽都自证过 ⇒ 判不出，拒可写、盘上不变。
+/// 判别力：判不出的前一半只看盘表次序第一块盘有没有报读错时，盘 0 没报，判得出、照常挂上。
+#[test]
+fn a_writable_mount_is_refused_when_only_the_second_device_reports_a_read_error_on_slot_zero_and_the_first_reads_back_zeros(
+) {
+    let mut pool =
+        pool_whose_newest_system_configuration_slot_is_slot_zero("lr1-slot-zero-second-device");
+    {
+        let devices = pool.devices.as_mut().expect("挂载之后盘还开着");
+        let (_, zeroed_device) = devices
+            .iter_mut()
+            .find(|(identity, _)| *identity == DEVICE_ZERO)
+            .expect("池里有盘 0");
+        zeroed_device
+            .write_at(
+                DeviceOffsetInBytes(offset_of_the_newest_slot()),
+                &vec![0u8; usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")],
+                WriteDurability::Plain,
+            )
+            .expect("盘 0 槽 0 清零");
+    }
+    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
+    let outcome = writable_mount_with_unreadable_ranges(
+        &mut pool,
+        vec![slot_zero_reporting_a_read_error_on(DEVICE_ONE)],
+    );
+    assert!(
+        refused_as_undecidable_on_both_reads(&outcome),
+        "盘 0 槽 0 读回全零、盘 1 槽 0 报读错：判不出，拒可写；得到 {}",
+        describe(&outcome)
+    );
+    assert_eq!(
+        disk_snapshot(&pool.memory_pool(), &pool.stream),
+        before,
+        "拒在取号之前：盘上不变"
+    );
+}
+
+/// 盘 1 两个系统配置槽各被读了几次（每一次碰到那一槽的读算一次），第一道屏障那一刻各几次；从第几次读起读坏、读坏过之后碰到别处的第一次读撤掉读故障。
+/// 几块盘共用一份。
+struct SlotReadsOfDeviceOne {
+    reads_of_each_slot: [u64; 2],
+    first_failing_read_of_each_slot: Option<[u64; 2]>,
+    failing_reads_lifted: bool,
+    failed_reads: u64,
+    reads_of_each_slot_at_the_first_barrier: Option<[u64; 2]>,
+}
+
+#[derive(Clone)]
+struct SharedSlotReadsOfDeviceOne(Rc<RefCell<SlotReadsOfDeviceOne>>);
+
+impl SharedSlotReadsOfDeviceOne {
+    fn new(first_failing_read_of_each_slot: Option<[u64; 2]>) -> Self {
+        Self(Rc::new(RefCell::new(SlotReadsOfDeviceOne {
+            reads_of_each_slot: [0; 2],
+            first_failing_read_of_each_slot,
+            failing_reads_lifted: false,
+            failed_reads: 0,
+            reads_of_each_slot_at_the_first_barrier: None,
+        })))
+    }
+}
+
+/// 读按 [`SharedSlotReadsOfDeviceOne`] 数盘 1 两槽、读坏、撤掉；写、写零原样交给里面那块盘；屏障那一刻记下两槽的读数。
+struct DeviceCountingSlotReadsOfDeviceOne<Inner: BlockDevice> {
+    identity: DeviceIdentity,
+    inner: Inner,
+    shared: SharedSlotReadsOfDeviceOne,
+}
+
+impl<Inner: BlockDevice> BlockDevice for DeviceCountingSlotReadsOfDeviceOne<Inner> {
+    fn read_at(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        let length_in_bytes = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
+        let slot_read = if self.identity == DEVICE_ONE {
+            [0, slot_spacing_in_bytes()]
+                .into_iter()
+                .position(|slot_offset| {
+                    offset.0 < slot_offset + SYSTEM_CONFIGURATION_SLOT_BYTES
+                        && slot_offset < offset.0 + length_in_bytes
+                })
+        } else {
+            None
+        };
+        let mut state = self.shared.0.borrow_mut();
+        match slot_read {
+            Some(slot_index) => {
+                state.reads_of_each_slot[slot_index] += 1;
+                let reads_of_this_slot = state.reads_of_each_slot[slot_index];
+                let fails_this_time = !state.failing_reads_lifted
+                    && state
+                        .first_failing_read_of_each_slot
+                        .is_some_and(|first_failing| {
+                            reads_of_this_slot >= first_failing[slot_index]
+                        });
+                if fails_this_time {
+                    state.failed_reads += 1;
+                    return Err(BlockDeviceError::InputOutput(std::io::Error::other(
+                        "用例点名的这一槽这一次读坏",
+                    )));
+                }
+            }
+            None => {
+                if state.failed_reads > 0 {
+                    state.failing_reads_lifted = true;
+                }
+            }
+        }
+        drop(state);
+        self.inner.read_at(offset, buffer)
+    }
+
+    fn write_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+        durability: WriteDurability,
+    ) -> Result<(), BlockDeviceError> {
+        self.inner.write_at(offset, bytes, durability)
+    }
+
+    fn write_zeroes_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<(), BlockDeviceError> {
+        self.inner.write_zeroes_at(offset, length)
+    }
+
+    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
+        let mut state = self.shared.0.borrow_mut();
+        if state.reads_of_each_slot_at_the_first_barrier.is_none() {
+            state.reads_of_each_slot_at_the_first_barrier = Some(state.reads_of_each_slot);
+        }
+        drop(state);
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
+fn counting_slot_reads_of_device_one<Inner: BlockDevice>(
+    devices: Vec<(DeviceIdentity, Inner)>,
+    shared: &SharedSlotReadsOfDeviceOne,
+) -> Vec<(DeviceIdentity, DeviceCountingSlotReadsOfDeviceOne<Inner>)> {
+    devices
+        .into_iter()
+        .map(|(identity, inner)| {
+            (
+                identity,
+                DeviceCountingSlotReadsOfDeviceOne {
+                    identity,
+                    inner,
+                    shared: shared.clone(),
+                },
+            )
+        })
+        .collect()
+}
+
+/// 取号写之前那道屏障之前、取号自己读盘 1 每一槽的次数：重算要取的号一次（`transaction::next_instance_generation_to_acquire`）、
+/// 取号写之前读上一个号一次、见证值那一核一次（`transaction::self_verified_system_configurations_of_every_device`）。
+/// 逐盘核那一遍是它们之前最后一次读这一槽。
+const SLOT_READS_OF_THE_ACQUISITION_BEFORE_ITS_BARRIER: u64 = 3;
+
+/// 场景：新池新建文件之后重开、可写挂载；读阶段与之前几遍读盘 1 两槽都读得出，到取号之前的逐盘核那一遍（取号写之前那道屏障那一刻
+/// 两槽各被读的次数减去取号自己的三次）两槽读坏，之后碰到别处的第一次读起读故障撤掉（两次读之间的瞬时读错）。
+/// 次数取另一份逐字节相同的池上照常挂载时数到的。
+/// 预期：逐盘核交 `Err(盘 1)`，照见证读的拒因报 `NewerStateStillUnreadableAfterOneReread`（后一遍判「有一块盘一份自证过的系统配置都没有」）；
+/// 只读坏了逐盘核那两次；系统配置槽、根环、录制流步数都不变。
+/// 判别力：逐盘核读不出就跳过这块盘时，取号那几遍读得出，挂载做成。
+#[test]
+fn a_writable_mount_is_refused_when_the_slots_of_the_second_device_turn_unreadable_only_for_the_check_of_every_device(
+) {
+    let mut probe_pool = build_pool("lr1-check-of-every-device-probe");
+    let probe = SharedSlotReadsOfDeviceOne::new(None);
+    let mut probe_devices = counting_slot_reads_of_device_one(probe_pool.reopen_recorded(), &probe);
+    let probe_mount = mount_writable(&parameters(), &mut probe_devices);
+    assert!(
+        probe_mount.is_ok(),
+        "一个读都不坏时这份池照常可写挂载：{}",
+        describe(&probe_mount)
+    );
+    drop(probe_mount);
+    drop(probe_devices);
+    let reads_at_the_first_barrier = probe
+        .0
+        .borrow()
+        .reads_of_each_slot_at_the_first_barrier
+        .expect("可写挂载取号之前发过屏障");
+    assert!(
+        reads_at_the_first_barrier
+            .iter()
+            .all(|reads| *reads > SLOT_READS_OF_THE_ACQUISITION_BEFORE_ITS_BARRIER + 1),
+        "取号写之前盘 1 两槽各被读过：择系统配置、见证读、逐盘核与取号那三次：{reads_at_the_first_barrier:?}"
+    );
+    let read_of_the_check_of_every_device = reads_at_the_first_barrier
+        .map(|reads| reads - SLOT_READS_OF_THE_ACQUISITION_BEFORE_ITS_BARRIER);
+    let mut pool = build_pool("lr1-check-of-every-device");
+    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
+    let shared = SharedSlotReadsOfDeviceOne::new(Some(read_of_the_check_of_every_device));
+    let mut devices = counting_slot_reads_of_device_one(pool.reopen_recorded(), &shared);
+    let outcome = mount_writable(&parameters(), &mut devices);
+    pool.devices = Some(
+        devices
+            .into_iter()
+            .map(|(identity, device)| (identity, device.inner))
+            .collect(),
+    );
+    let refused_by_the_check_of_every_device = matches!(
+        &outcome,
+        Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable))
+            if matches!(
+                still_unreadable.as_ref(),
+                StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
+                    reread,
+                    ..
+                } if reread.witness.comparison
+                    == WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot
+            )
+    );
+    assert!(
+        refused_by_the_check_of_every_device,
+        "逐盘核那一遍盘 1 两槽读不出：拒可写，后一遍判「有一块盘一份自证过的系统配置都没有」；得到 {}",
+        describe(&outcome)
+    );
+    assert_eq!(
+        shared.0.borrow().failed_reads,
+        2,
+        "只读坏了逐盘核读盘 1 两槽那两次（第 {read_of_the_check_of_every_device:?} 次）"
+    );
+    assert_eq!(
+        disk_snapshot(&pool.memory_pool(), &pool.stream),
+        before,
+        "拒在取号之前：盘上不变"
+    );
+}
````

## 4、改钉值（两行锚点、U13 映射树分裂）

`/tmp/claude-1000/m3-batch12/pins/patch/crates.patch`

````diff
diff --git a/crates/singlefs-harness/src/model.rs b/crates/singlefs-harness/src/model.rs
index 309a316..da01f95 100644
--- a/crates/singlefs-harness/src/model.rs
+++ b/crates/singlefs-harness/src/model.rs
@@ -24,9 +24,10 @@ use singlefs_format::{
     ALLOCATION_RECORD_TREE_LEAF_SLOTS, CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES,
     DATA_UNIT_PAYLOAD_OFFSET, EXTENT_TREE_INTERNAL_FANOUT, EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
     INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE, INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT,
-    INSTANCE_TABLE_PAGE_RECORDS, JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, NODE_BYTES,
-    NONCE_MAC_ALGORITHM_RESERVED_BYTES, ROOT_RING_REGIONS, ROOT_RING_REGION_DEVICES,
-    ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM, SLOT_BYTES,
+    INSTANCE_TABLE_PAGE_RECORDS, JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT,
+    LINEAGE_CONTAINER_RECORDS, MAPPING_ENTRY_BYTES, MAPPING_KEY_BYTES, NODE_BYTES,
+    NODE_POINTER_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES, ROOT_RING_REGIONS,
+    ROOT_RING_REGION_DEVICES, ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM, SLOT_BYTES,
 };
 
 /// 模型里的 checkpoint 号（D16（发布语义） 已定项 6：每次发布 + 1）。与实现的 `CheckpointTxg` 各自声明（D13（验证路线） 已定项 5）。
@@ -390,6 +391,9 @@ pub struct ModelRoot {
     pub role_written_at: BTreeMap<ModelUnitRole, ModelCheckpointTxg>,
     /// 每盘已占槽数的上界：沿来路每次发布写出的单元的槽数之和，释放与回收一概不扣（分配记录树每次重写的节点数取按位置寻址现算的上界）。
     pub occupied_slots_upper_bound_per_device: u64,
+    /// 这一版中央映射树节点数的上界（[`central_mapping_tree_nodes_upper_bound`] 沿来路带下来，每盘一个节点占一个槽）；
+    /// 树表 0 条的一版没有这棵树，记 0。
+    pub central_mapping_tree_nodes_upper_bound: u64,
     /// 写出这条根的那次发布的记录里最大的事务号：写文件内容的发布（第一个文件、覆盖写、顺序写）一事务一单元（D16（发布语义） 已定项 5）
     /// ⇒ 内容切成几个数据单元就承载几个事务、接着实例内计数往下取，这里记最后那一个号；删除类发布与建 inode 各一个事务；
     /// 空发布、写行、暖机、抬 F、回退的记录事务号 0（D23（journal 的角色与格式） 已定项 19 ①），这里记 0。
@@ -510,6 +514,131 @@ pub fn extent_lower_segment_levels(data_units: u64) -> u64 {
     levels
 }
 
+/// 中央映射树一片叶装几条映射条目：(节点 − 头) ÷ 映射条目宽（D8（核心索引结构） 已定项 11；条目 55，D19（块指针的结构与宽度预算） 已定项 6）。
+fn central_mapping_tree_leaf_entry_capacity() -> u64 {
+    index_node_entry_capacity(MAPPING_KEY_BYTES, MAPPING_ENTRY_BYTES)
+}
+
+/// 中央映射树一个内部节点装几个孩子：内部条目 = 本树 key + 子指针 86（D8（核心索引结构） 已定项 11）。
+fn central_mapping_tree_internal_child_capacity() -> u64 {
+    index_node_entry_capacity(MAPPING_KEY_BYTES, MAPPING_KEY_BYTES + NODE_POINTER_BYTES)
+}
+
+/// 一个装不下的节点从中间切开之后，切出来的右半至少几条：装不下时是容量 + 1 条，从中间切两半各至少 ⌊(容量 + 1) ÷ 2⌋ = ⌈容量 ÷ 2⌉
+/// （D8（核心索引结构） 已定项 11 ②）。
+fn smallest_half_of_a_split(capacity: u64) -> u64 {
+    capacity.div_ceil(2)
+}
+
+/// 中央映射树一次发布之后至多几个节点（D8（核心索引结构） 已定项 11 ②③ 与「一次发布里先删后插」）。
+///
+/// 节点数不是这一版条目数的函数：叶装不下时从中间切，收缩只摘空节点、不合并，删得只剩几条的叶照样留着，形状随来路变
+/// （同一组条目，来路上长过的树可以比一次建起来的多几个节点）。模型不记 key、不照抄实现的分裂（D13（验证路线） 已定项 5），
+/// 沿来路带一个上界：
+/// - 先删：只摘空节点、根只剩一个孩子时降高，节点数不增；
+/// - 后插：设叶切了 s 次、内部节点切了 t 次。切出来的右半至少 ⌊(容量 + 1) ÷ 2⌋（叶 147 条、内部 72 个孩子，
+///   [`smallest_half_of_a_split`]），之后这一段只插不删，到这次结束仍至少这么多、彼此不相交：147 s ≤ 这一版的条目数 E，
+///   72 t ≤ 这次之后的节点数 N − 1（根之外每个节点恰是一个内部节点的孩子）；根切开再长一个新根，至多 s + t 次。
+///
+/// ⇒ N ≤ N_前 + 2 (s + t) ≤ N_前 + 2 ⌊E ÷ 147⌋ + 2 ⌊(N − 1) ÷ 72⌋。右边随 N 涨得比 N 慢，满足它的 N 有个最大值：
+/// 从 2 (N_前 + 2 ⌊E ÷ 147⌋)（那里已不满足）往下迭代，每一步都仍是 N 的上界，停在不动点（同分配记录树那一份的取法）。
+///
+/// `nodes_before`：上一版这棵树节点数的上界；这一版从空树建起时取 1（实现的规划从一片空的根兼叶起插）。
+///
+/// # Panics
+/// `nodes_before` 为 0：一棵有条目的树至少一个节点，调用方给的是上一版的上界或 1。
+#[must_use]
+pub fn central_mapping_tree_nodes_upper_bound(nodes_before: u64, entries: u64) -> u64 {
+    assert!(
+        nodes_before >= 1,
+        "上一版的树至少一个节点（从空树建起也从一片空的根兼叶起插）"
+    );
+    let nodes_without_internal_splits = nodes_before
+        + 2 * (entries / smallest_half_of_a_split(central_mapping_tree_leaf_entry_capacity()));
+    let internal_split_bound = |nodes: u64| {
+        nodes_without_internal_splits
+            + 2 * ((nodes - 1)
+                / smallest_half_of_a_split(central_mapping_tree_internal_child_capacity()))
+    };
+    let mut bound = 2 * nodes_without_internal_splits;
+    // 迭代上界：bound 每一轮严格变小，不小于 1。跨轮带的是「这次之后的节点数不超过 bound」。
+    loop {
+        let next = internal_split_bound(bound);
+        if next >= bound {
+            return bound;
+        }
+        bound = next;
+    }
+}
+
+/// 一版的中央映射树里除分配记录树的节点之外装几条条目的上界（D19（块指针的结构与宽度预算） 已定项 8：进映射的是数据单元、extent 树每个节点、
+/// inode 叶容器与根、分配记录树每个节点、记账树每个节点、`deleted_inodes` 根、墓碑树与谱系树的容器与索引节点；
+/// 映射树自己的节点、树表、实例表豁免）。分配记录树的节点数由调用方按它的上界另加。
+///
+/// 谱系树模型不记（管理员回退往它上面追加一条，D18（块里携带什么信息） 已定项 24 第 5 条）：一条时间线上每次发布至多是一次回退，
+/// 回退至多 txg 次，一片容器装 2039 条，加根；没回退过的一版没有这棵树，多算的两条只让上界松一点。
+///
+/// # Panics
+/// 谱系容器的上界多于一片：模型罩的历史一条时间线上发布不到 2039 次，谱系树根之下不长层。
+fn central_mapping_entries_other_than_the_allocation_record_tree(
+    contents: &ModelVersionContents,
+    role_written_at: &BTreeMap<ModelUnitRole, ModelCheckpointTxg>,
+    checkpoint_txg: ModelCheckpointTxg,
+) -> u64 {
+    let data_units = match contents {
+        ModelVersionContents::LiveFile(file) => {
+            data_units_of_content(u64::try_from(file.content.len()).expect("内容长度"))
+        }
+        ModelVersionContents::FileBeingDeleted {
+            data_units_remaining,
+            ..
+        } => *data_units_remaining,
+        ModelVersionContents::WithoutTrees
+        | ModelVersionContents::TreesAndNoFile { .. }
+        | ModelVersionContents::InodesWithoutTheFirstFile { .. } => 0,
+    };
+    let entries_of_one_node_roles: u64 = role_written_at
+        .keys()
+        .map(|role| match role {
+            // 模型罩的一版里这几个角色各一个单元（上段只有根兼叶、一片 inode 叶容器、记账树一个节点、墓碑树一片容器加根）。
+            ModelUnitRole::ExtentRoot
+            | ModelUnitRole::InodeLeaf
+            | ModelUnitRole::InodeRoot
+            | ModelUnitRole::AccountingTree
+            | ModelUnitRole::DeletedInodesRoot
+            | ModelUnitRole::TombstoneLeaf
+            | ModelUnitRole::TombstoneRoot => 1,
+            // 数据单元按这一版的单元数另数，分配记录树的节点由调用方按上界另加。
+            ModelUnitRole::Data | ModelUnitRole::AllocationTree => 0,
+            // 豁免映射（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
+            ModelUnitRole::MappingTree
+            | ModelUnitRole::TreeTable
+            | ModelUnitRole::InstanceTable => 0,
+        })
+        .sum();
+    let lineage_containers = checkpoint_txg.0.div_ceil(LINEAGE_CONTAINER_RECORDS);
+    assert!(
+        lineage_containers <= 1,
+        "模型罩的历史一条时间线上发布不到 2039 次（谱系树至多一片容器）：txg {}",
+        checkpoint_txg.0
+    );
+    let lineage_root = u64::from(lineage_containers > 0);
+    data_units
+        + extent_lower_segment_nodes(data_units)
+        + entries_of_one_node_roles
+        + lineage_containers
+        + lineage_root
+}
+
+/// 一次发布写出的那一版的用户可见树（inode 树、extent 树、`deleted_inodes`）从哪来。
+#[derive(Clone, Copy, Debug)]
+enum UserVisibleTreesOfTheNewVersion<'target> {
+    /// 这次发布按它的种类写出或照抄上一版的。
+    AsThisPublishLeavesThem,
+    /// 管理员回退那次：取回退目标那一版的（D23（journal 的角色与格式） 已定项 14）。
+    TakenFromTheRollbackTarget(&'target ModelRoot),
+}
+
 /// 整份重写文件内容的入口按哪个契约收内容：覆盖写一个数据单元以内（装不下拒，D4（校验和位置） 已定项 5），顺序写不按单元数拒。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum WholeFileWriteContract {
@@ -1020,6 +1149,7 @@ impl IdealModel {
             ]),
             occupied_slots_upper_bound_per_device: ModelUnitRole::InstanceTable.span_in_slots()
                 + ModelUnitRole::TreeTable.span_in_slots(),
+            central_mapping_tree_nodes_upper_bound: 0,
             highest_transaction_number_of_its_publish: 0,
             highest_transaction_number_in_its_instance: 0,
         };
@@ -1260,6 +1390,46 @@ impl IdealModel {
         }
     }
 
+    /// 这次发布之后中央映射树节点数的上界，与分配记录树这次至多重写几个节点
+    /// （[`IdealModel::allocation_record_tree_nodes_rewritten_upper_bound`]）。
+    ///
+    /// 两个量互相牵连：映射树装着分配记录树每个节点的一条条目（D19（块指针的结构与宽度预算） 已定项 8），分配记录树可能装着记录的节点数
+    /// 又随映射树多写的节点占的槽涨。两个都对另一个单调不减：从「分配记录树罩满单元区」那一头起交替往下迭代，
+    /// 每一步都仍是两者的上界，停在映射树节点数不再变小的那一步。
+    ///
+    /// `occupied_slots_before_the_two_trees`：沿来路写过的槽，加这次除映射树根之下的节点与分配记录树之外写的槽（映射树的根已算在里面）。
+    fn central_mapping_and_allocation_record_tree_nodes_upper_bounds(
+        &self,
+        central_mapping_tree_nodes_before: u64,
+        central_mapping_entries_other_than_the_allocation_record_tree: u64,
+        occupied_slots_before_the_two_trees: u64,
+    ) -> (u64, u64) {
+        let mapping_nodes_given = |allocation_record_tree_nodes: u64| {
+            central_mapping_tree_nodes_upper_bound(
+                central_mapping_tree_nodes_before,
+                central_mapping_entries_other_than_the_allocation_record_tree
+                    + allocation_record_tree_nodes,
+            )
+        };
+        let allocation_record_tree_nodes_given = |mapping_nodes: u64| {
+            self.allocation_record_tree_nodes_rewritten_upper_bound(
+                occupied_slots_before_the_two_trees
+                    + (mapping_nodes - 1) * ModelUnitRole::MappingTree.span_in_slots(),
+            )
+        };
+        let mut mapping_nodes =
+            mapping_nodes_given(self.allocation_record_tree_nodes_rewritten_upper_bound(u64::MAX));
+        // 迭代上界：映射树节点数每一轮严格变小、不小于 1。跨轮带的是「两者的真值都不超过这一轮的 (映射树, 分配记录树)」。
+        loop {
+            let allocation_record_tree_nodes = allocation_record_tree_nodes_given(mapping_nodes);
+            let next = mapping_nodes_given(allocation_record_tree_nodes);
+            if next >= mapping_nodes {
+                return (mapping_nodes, allocation_record_tree_nodes);
+            }
+            mapping_nodes = next;
+        }
+    }
+
     /// 一次写记账行的发布要几行、一个节点装几行（D5（快照 / 空间记账机制） 已定项 8）。
     fn accounting_rows_exceed_one_node(&self) -> bool {
         let device_count = u64::try_from(self.geometry.devices.len()).expect("盘数");
@@ -1287,6 +1457,37 @@ impl IdealModel {
         rollback_floor: ModelCheckpointTxg,
         new_file_content: Option<&[u8]>,
         instance_table_rows: Rc<Vec<ModelInstanceRow>>,
+    ) -> ModelRoot {
+        self.next_root_with_the_user_visible_trees(
+            previous,
+            instance,
+            checkpoint_txg,
+            journal_counter,
+            kind,
+            rollback_floor,
+            new_file_content,
+            instance_table_rows,
+            UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem,
+        )
+    }
+
+    /// 同 [`IdealModel::next_root`]，另说这一版的用户可见树从哪来：管理员回退那次发布取回退目标那一版的。
+    /// 中央映射树装的是这一版的条目，占槽上界要按换过之后的这一版算，所以换在算占槽之前做。
+    #[allow(
+        clippy::too_many_arguments,
+        reason = "一条根的每个身份字段各是一个参数，收成结构体只会多一层没人验的名字"
+    )]
+    fn next_root_with_the_user_visible_trees(
+        &self,
+        previous: &ModelRoot,
+        instance: ModelInstanceGeneration,
+        checkpoint_txg: ModelCheckpointTxg,
+        journal_counter: ModelJournalCounter,
+        kind: ModelPublishKind,
+        rollback_floor: ModelCheckpointTxg,
+        new_file_content: Option<&[u8]>,
+        instance_table_rows: Rc<Vec<ModelInstanceRow>>,
+        user_visible_trees: UserVisibleTreesOfTheNewVersion<'_>,
     ) -> ModelRoot {
         let key = ModelRootKey {
             checkpoint_txg,
@@ -1309,63 +1510,11 @@ impl IdealModel {
             | ModelVersionContents::TreesAndNoFile { .. }
             | ModelVersionContents::InodesWithoutTheFirstFile { .. } => None,
         };
-        let mut slots_written = 0;
-        let mut rewrites_the_allocation_record_tree = false;
         let rewritten = kind.rewritten_roles();
         for role in rewritten {
             role_written_at.insert(*role, checkpoint_txg);
-            match role {
-                ModelUnitRole::AllocationTree => rewrites_the_allocation_record_tree = true,
-                // 内容的每个数据单元都在这次写（模型把它们记成同一个角色、同一个分配代）。
-                ModelUnitRole::Data => {
-                    slots_written += role.span_in_slots() * data_units_of_the_new_content;
-                }
-                ModelUnitRole::ExtentRoot
-                | ModelUnitRole::InodeLeaf
-                | ModelUnitRole::InodeRoot
-                | ModelUnitRole::AccountingTree
-                | ModelUnitRole::MappingTree
-                | ModelUnitRole::TreeTable
-                | ModelUnitRole::InstanceTable
-                | ModelUnitRole::DeletedInodesRoot
-                | ModelUnitRole::TombstoneLeaf
-                | ModelUnitRole::TombstoneRoot => slots_written += role.span_in_slots(),
-            }
-        }
-        // extent 树下段的节点不对应角色（同分配记录树根之下的节点：分配代与位置归池级 checker 判），占槽按条款现算：
-        // 写内容的两种重写整个下段；不是最后一批的删除批只重写被截掉那一片下段叶与它的祖先，每层至多一个。
-        let extent_lower_segment_nodes_written = match kind {
-            ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion => {
-                extent_lower_segment_nodes(data_units_of_the_new_content)
-            }
-            ModelPublishKind::DeletionBatchKeepingDataUnits => {
-                extent_lower_segment_levels(data_units_remaining_before_this_publish.expect(
-                    "不是最后一批的删除批只接在删除中的那一版后面（`answer_deletion_step` 判过）",
-                ))
-            }
-            ModelPublishKind::RowsOnFileVersion
-            | ModelPublishKind::EmptyOnFileVersion
-            | ModelPublishKind::RowsOnVersionWithoutFile
-            | ModelPublishKind::ZeroUnit
-            | ModelPublishKind::UnlinkIntent
-            | ModelPublishKind::FinalDeletionBatch
-            | ModelPublishKind::NewInode => 0,
-        };
-        slots_written += extent_lower_segment_nodes_written * (NODE_BYTES / SLOT_BYTES);
-        // 实例表链多于一片时第 1 片起每一片也是一个重写的单元：`rewritten_roles` 里实例表只记一个角色，多出来的几片在这里补上。
-        let instance_table_pages_after_the_first = if kind.rewrites_the_instance_table() {
-            instance_table_pages_for_rows(instance_table_rows.len()) - 1
-        } else {
-            0
-        };
-        slots_written +=
-            instance_table_pages_after_the_first * ModelUnitRole::InstanceTable.span_in_slots();
-        if rewrites_the_allocation_record_tree {
-            slots_written += self.allocation_record_tree_nodes_rewritten_upper_bound(
-                previous.occupied_slots_upper_bound_per_device + slots_written,
-            ) * ModelUnitRole::AllocationTree.span_in_slots();
         }
-        let contents = match (kind, new_file_content) {
+        let mut contents = match (kind, new_file_content) {
             (
                 ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion,
                 Some(content),
@@ -1414,6 +1563,110 @@ impl IdealModel {
                 Some(_),
             ) => panic!("只有写文件内容的两种发布带内容，别的都不带：{kind:?}"),
         };
+        match user_visible_trees {
+            UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem => {}
+            UserVisibleTreesOfTheNewVersion::TakenFromTheRollbackTarget(target_root) => {
+                contents.clone_from(&target_root.contents);
+                // 新根的 inode 树、extent 树与 `deleted_inodes` 取回退目标那一版的（D23（journal 的角色与格式） 已定项 14）：
+                // 目标那一版有的角色取它的分配代，没有的（删完的一版没有数据单元、叶容器与 `deleted_inodes` 根）这一版也没有；
+                // 墓碑树随现行那一版。
+                for role in [
+                    ModelUnitRole::Data,
+                    ModelUnitRole::ExtentRoot,
+                    ModelUnitRole::InodeLeaf,
+                    ModelUnitRole::InodeRoot,
+                    ModelUnitRole::DeletedInodesRoot,
+                ] {
+                    match target_root.role_written_at.get(&role) {
+                        Some(written_at) => {
+                            role_written_at.insert(role, *written_at);
+                        }
+                        None => {
+                            role_written_at.remove(&role);
+                        }
+                    }
+                }
+            }
+        }
+        let mut slots_written = 0;
+        let mut rewrites_the_allocation_record_tree = false;
+        for role in rewritten {
+            match role {
+                ModelUnitRole::AllocationTree => rewrites_the_allocation_record_tree = true,
+                // 内容的每个数据单元都在这次写（模型把它们记成同一个角色、同一个分配代）。
+                ModelUnitRole::Data => {
+                    slots_written += role.span_in_slots() * data_units_of_the_new_content;
+                }
+                // 中央映射树在这里只记它的根：根之下的节点数按上界另算（下面）。
+                ModelUnitRole::ExtentRoot
+                | ModelUnitRole::InodeLeaf
+                | ModelUnitRole::InodeRoot
+                | ModelUnitRole::AccountingTree
+                | ModelUnitRole::MappingTree
+                | ModelUnitRole::TreeTable
+                | ModelUnitRole::InstanceTable
+                | ModelUnitRole::DeletedInodesRoot
+                | ModelUnitRole::TombstoneLeaf
+                | ModelUnitRole::TombstoneRoot => slots_written += role.span_in_slots(),
+            }
+        }
+        // extent 树下段的节点不对应角色（同分配记录树根之下的节点：分配代与位置归池级 checker 判），占槽按条款现算：
+        // 写内容的两种重写整个下段；不是最后一批的删除批只重写被截掉那一片下段叶与它的祖先，每层至多一个。
+        let extent_lower_segment_nodes_written = match kind {
+            ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion => {
+                extent_lower_segment_nodes(data_units_of_the_new_content)
+            }
+            ModelPublishKind::DeletionBatchKeepingDataUnits => {
+                extent_lower_segment_levels(data_units_remaining_before_this_publish.expect(
+                    "不是最后一批的删除批只接在删除中的那一版后面（`answer_deletion_step` 判过）",
+                ))
+            }
+            ModelPublishKind::RowsOnFileVersion
+            | ModelPublishKind::EmptyOnFileVersion
+            | ModelPublishKind::RowsOnVersionWithoutFile
+            | ModelPublishKind::ZeroUnit
+            | ModelPublishKind::UnlinkIntent
+            | ModelPublishKind::FinalDeletionBatch
+            | ModelPublishKind::NewInode => 0,
+        };
+        slots_written += extent_lower_segment_nodes_written * (NODE_BYTES / SLOT_BYTES);
+        // 实例表链多于一片时第 1 片起每一片也是一个重写的单元：`rewritten_roles` 里实例表只记一个角色，多出来的几片在这里补上。
+        let instance_table_pages_after_the_first = if kind.rewrites_the_instance_table() {
+            instance_table_pages_for_rows(instance_table_rows.len()) - 1
+        } else {
+            0
+        };
+        slots_written +=
+            instance_table_pages_after_the_first * ModelUnitRole::InstanceTable.span_in_slots();
+        // 有树的一版每次发布都重写中央映射树（根恒在 `rewritten_roles` 里）与分配记录树；一次至多重写这一版的全部节点。
+        // 树表 0 条的一版没有映射树，分配记录树照写行那次重写不重写算。
+        let central_mapping_tree_nodes_upper_bound = if contents.has_trees() {
+            assert!(
+                rewritten.contains(&ModelUnitRole::MappingTree)
+                    && rewrites_the_allocation_record_tree,
+                "有树的一版每次发布都重写映射树与分配记录树：{kind:?}"
+            );
+            let (mapping_nodes, allocation_record_tree_nodes) = self
+                .central_mapping_and_allocation_record_tree_nodes_upper_bounds(
+                    previous.central_mapping_tree_nodes_upper_bound.max(1),
+                    central_mapping_entries_other_than_the_allocation_record_tree(
+                        &contents,
+                        &role_written_at,
+                        checkpoint_txg,
+                    ),
+                    previous.occupied_slots_upper_bound_per_device + slots_written,
+                );
+            slots_written += (mapping_nodes - 1) * ModelUnitRole::MappingTree.span_in_slots()
+                + allocation_record_tree_nodes * ModelUnitRole::AllocationTree.span_in_slots();
+            mapping_nodes
+        } else {
+            if rewrites_the_allocation_record_tree {
+                slots_written += self.allocation_record_tree_nodes_rewritten_upper_bound(
+                    previous.occupied_slots_upper_bound_per_device + slots_written,
+                ) * ModelUnitRole::AllocationTree.span_in_slots();
+            }
+            0
+        };
         let highest_transaction_number_before_this_publish = if previous.key.instance == instance {
             previous.highest_transaction_number_in_its_instance
         } else {
@@ -1438,6 +1691,7 @@ impl IdealModel {
             role_written_at,
             occupied_slots_upper_bound_per_device: previous.occupied_slots_upper_bound_per_device
                 + slots_written,
+            central_mapping_tree_nodes_upper_bound,
             highest_transaction_number_of_its_publish,
             highest_transaction_number_in_its_instance:
                 highest_transaction_number_before_this_publish
@@ -1967,7 +2221,7 @@ impl IdealModel {
         let roots = match (target_root, required_refusals.is_empty()) {
             (Some(target_root), true) => {
                 let rollback_txg = ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1);
-                let mut rollback_root = self.next_root(
+                let rollback_root = self.next_root_with_the_user_visible_trees(
                     current,
                     session.instance,
                     rollback_txg,
@@ -1976,27 +2230,8 @@ impl IdealModel {
                     current.rollback_floor,
                     None,
                     Rc::clone(&current.instance_table_rows),
+                    UserVisibleTreesOfTheNewVersion::TakenFromTheRollbackTarget(target_root),
                 );
-                rollback_root.contents.clone_from(&target_root.contents);
-                // 新根的 inode 树、extent 树与 `deleted_inodes` 取回退目标那一版的（D23（journal 的角色与格式） 已定项 14）：
-                // 目标那一版有的角色取它的分配代，没有的（删完的一版没有数据单元、叶容器与 `deleted_inodes` 根）这一版也没有；
-                // 墓碑树随现行那一版。
-                for role in [
-                    ModelUnitRole::Data,
-                    ModelUnitRole::ExtentRoot,
-                    ModelUnitRole::InodeLeaf,
-                    ModelUnitRole::InodeRoot,
-                    ModelUnitRole::DeletedInodesRoot,
-                ] {
-                    match target_root.role_written_at.get(&role) {
-                        Some(written_at) => {
-                            rollback_root.role_written_at.insert(role, *written_at);
-                        }
-                        None => {
-                            rollback_root.role_written_at.remove(&role);
-                        }
-                    }
-                }
                 vec![rollback_root]
             }
             (Some(_) | None, _) => Vec::new(),
@@ -3728,4 +3963,82 @@ mod tests {
             "148 片叶要两个第一层节点，再一个根"
         );
     }
+
+    /// 中央映射树节点数的上界（D8（核心索引结构） 已定项 11：叶装 294 条、内部节点装 143 个孩子，装不下从中间切，切出来的右半至少 147 条 / 72 个孩子；
+    /// 收缩只摘空节点）：条目不到 147 条时一次都切不了，上界照上一版的；每多 147 条至多多切一片叶、多长一个新根；上一版长过的节点不往回收。
+    #[test]
+    fn central_mapping_tree_nodes_grow_by_two_per_147_entries_and_keep_the_previous_bound() {
+        assert_eq!(
+            (
+                central_mapping_tree_leaf_entry_capacity(),
+                central_mapping_tree_internal_child_capacity(),
+                smallest_half_of_a_split(central_mapping_tree_leaf_entry_capacity()),
+                smallest_half_of_a_split(central_mapping_tree_internal_child_capacity()),
+            ),
+            (294, 143, 147, 72)
+        );
+        assert_eq!(
+            [(1, 146), (1, 147), (1, 294), (5, 10), (1, 147 * 72)]
+                .map(|(nodes_before, entries)| central_mapping_tree_nodes_upper_bound(
+                    nodes_before,
+                    entries
+                )),
+            [1, 1 + 2, 1 + 2 * 2, 5, 1 + 2 * 72 + 2 * 2],
+            "147 × 72 条：叶至多切 72 次，内部节点再切的次数 t 满足 72 t ≤ N − 1，停在 N = 149（t = 2）"
+        );
+    }
+
+    /// 顺序写一个跨三片下段叶的文件（292 个单元）：映射条目 302 条加分配记录树的节点，多于一片叶装的 294 条，映射树从中间切开。
+    /// 4 GiB 两块盘、第一个文件之后（每盘写过 21 槽，映射树一个节点）：这次映射树之外写 595 槽（数据 2 × 292、下段 3 片叶加 1 个内部节点、
+    /// extent 根 1、inode 叶 2、inode 根 1、记账树、映射树的根、树表各 1）；分配记录树至多 123 个节点，条目至多 302 + 123 = 425，
+    /// 叶至多切 2 次 ⇒ 映射树至多 1 + 2 × 2 = 5 个节点，根之下 4 个各占一槽：这次占槽上界 595 + 4 + 123 = 722。
+    /// 条目里 302 = 数据 292 + 下段 4 + extent 根、inode 叶、inode 根、记账树各 1 + 谱系树按 txg 4 至多一片容器加根 2。
+    #[test]
+    fn a_sequential_write_of_three_lower_leaves_counts_the_central_mapping_tree_nodes_below_the_root_in_the_occupied_slots(
+    ) {
+        let mut model = two_device_model();
+        model.acquire_and_warm_up_in_the_make_filesystem_process();
+        let first = model
+            .answer_publish_first_file(&[3])
+            .expect("会话开着、现行 txg 2");
+        succeed(&mut model, &first);
+        let slots_before = model.newest_root().occupied_slots_upper_bound_per_device;
+        assert_eq!(
+            (
+                slots_before,
+                model.newest_root().central_mapping_tree_nodes_upper_bound
+            ),
+            (3 + 9 + 9, 1),
+            "第一个文件之后：映射条目十来条，映射树一个节点"
+        );
+        let data_units = 2 * EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 4;
+        let capacity = data_unit_payload_capacity_in_bytes();
+        let content =
+            vec![5_u8; usize::try_from((data_units - 1) * capacity + 1).expect("九兆多字节")];
+        assert_eq!(
+            data_units_of_content(u64::try_from(content.len()).expect("内容长度")),
+            292
+        );
+        let sequential_write = model
+            .answer_publish_sequential_write(&content)
+            .expect("会话开着、现行版本带文件");
+        assert!(
+            sequential_write.required_refusals.is_empty(),
+            "{:?}",
+            sequential_write.required_refusals
+        );
+        let written = &sequential_write.expected_roots[0];
+        let slots_outside_the_two_trees = 2 * 292 + (3 + 1) + 1 + 2 + 1 + 1 + 1 + 1;
+        assert_eq!(
+            (
+                written.central_mapping_tree_nodes_upper_bound,
+                model.allocation_record_tree_nodes_rewritten_upper_bound(
+                    slots_before + slots_outside_the_two_trees + 4
+                ),
+                written.occupied_slots_upper_bound_per_device - slots_before,
+            ),
+            (5, 123, slots_outside_the_two_trees + 4 + 123),
+            "映射树至多 5 个节点（根之下 4 个），分配记录树至多 123 个节点：这次占槽上界 722"
+        );
+    }
 }
diff --git a/crates/singlefs-harness/src/model_comparison.rs b/crates/singlefs-harness/src/model_comparison.rs
index e847dfa..e4b6db4 100644
--- a/crates/singlefs-harness/src/model_comparison.rs
+++ b/crates/singlefs-harness/src/model_comparison.rs
@@ -56,7 +56,7 @@ pub fn model_root_key(instance: InstanceGeneration, checkpoint_txg: CheckpointTx
     }
 }
 
-/// 实现的一个单元是模型的哪个角色。分配记录树根之下的节点不对应角色（`None`）：D8（核心索引结构） 已定项 14 按绝对槽号按位置寻址，
+/// 实现的一个单元是模型的哪个角色。不对应角色（`None`）的几种各臂写着为什么；分配记录树根之下的节点是这样：D8（核心索引结构） 已定项 14 按绝对槽号按位置寻址，
 /// 一次发布重写根之下哪几片由这次动了哪些槽定，模型不记落点、不记是哪几片（占槽只取上界）；那几片的分配代、位置与记录归池级 checker 判
 /// （I-1.1 位置、I-3.10 每个节点都有记录）。
 #[must_use]
@@ -88,15 +88,17 @@ pub fn model_unit_role(unit: TransactionUnit) -> Option<ModelUnitRole> {
         TransactionUnit::InodeRoot => ModelUnitRole::InodeRoot,
         TransactionUnit::AllocationTree => ModelUnitRole::AllocationTree,
         TransactionUnit::AllocationTreeNodeBelowTheRoot(_) => return None,
-        // 模型罩的池两块盘、15 行记账，映射条目恒 6 条：两棵树恒只有一个节点（根兼叶），根之下的节点只在压小容量的
+        // 模型罩的池两块盘、15 行记账：记账树恒只有一个节点（根兼叶），根之下的节点只在压小容量的
         // 只供测试的开关下、或记账行装不下一个节点的池（80 块盘起）上出现，随机历史与层 0 的固定脚本都不装那个开关。
         TransactionUnit::AccountingTreeNodeBelowTheRoot(position) => {
             panic!("模型今天只罩记账树一个节点的池：{position:?}")
         }
         TransactionUnit::AccountingTree => ModelUnitRole::AccountingTree,
-        TransactionUnit::MappingTreeNodeBelowTheRoot(position) => {
-            panic!("模型今天只罩中央映射树一个节点的池：{position:?}")
-        }
+        // 中央映射树根之下的节点不对应角色（同分配记录树根之下的节点）：每个数据单元一条映射条目，数据单元加别的进映射的单元
+        // 多于一片叶装的 294 条时（随机历史里是跨三片下段叶的那一档），映射树从中间切开、长出根之下的节点（D8（核心索引结构） 已定项 11）；
+        // 哪几片这次重写由这次删插了哪些 key 定，模型不记 key、不记是哪几片。它们的分配代、位置与记录归池级 checker 判，
+        // 占槽模型沿来路带节点数的上界（`model::central_mapping_tree_nodes_upper_bound`）。
+        TransactionUnit::MappingTreeNodeBelowTheRoot(_) => return None,
         TransactionUnit::MappingTree => ModelUnitRole::MappingTree,
         TransactionUnit::TreeTable => ModelUnitRole::TreeTable,
         // 删除那两棵树（里程碑三·2 第 8 条）：`deleted_inodes` 至多一条 key（只有 inode 1），根兼叶一个节点；墓碑一次 unlink 一条、
diff --git a/crates/singlefs-harness/tests/random_histories.rs b/crates/singlefs-harness/tests/random_histories.rs
index 4ca6bbf..a464475 100644
--- a/crates/singlefs-harness/tests/random_histories.rs
+++ b/crates/singlefs-harness/tests/random_histories.rs
@@ -2113,3 +2113,44 @@ fn deleting_a_file_of_two_lower_leaves_then_rolling_back_to_its_intent_walks_two
         "建 inode 之前走空两步比过一次；不是最后一批的删除批 unlink 里一个、走空里一个"
     );
 }
+
+/// 跨三片下段叶的文件（292 个单元，`UserChange::SequentialWrite` 写的）：映射条目多于一片叶装的 294 条，实现的中央映射树从中间切开、
+/// 长出根之下的节点（D8（核心索引结构） 已定项 11）；再经会话 unlink：意图那次、两个不是最后一批的删除批（292 → 148 → 4）、最后一批，
+/// 逐版交模型比（U13 慢档取样段在种子 …115 等六段上撞到的那一形）。判别力：胶水把映射树根之下的节点当成模型罩不到的池而 panic 时，
+/// 顺序写那一步就以 panic 收尾。
+#[test]
+fn a_file_of_three_lower_leaves_grows_the_central_mapping_tree_below_its_root_and_the_model_compares_every_version(
+) {
+    let history = GeneratedHistory {
+        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations: vec![
+            HistoryOperation::PublishOverwrite(ContentChoice {
+                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
+                    data_units: 292,
+                    selector: 7,
+                },
+                fill_seed: 11,
+            }),
+            HistoryOperation::Unlink,
+        ],
+    };
+    let run = execute_history(&history);
+    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
+    assert!(
+        run.outcomes
+            .iter()
+            .all(|outcome| matches!(outcome, StepOutcome::Applied(_))),
+        "顺序写与 unlink 都做成：{:?}",
+        run.outcomes
+    );
+    let counts = &run.tally.model_counts;
+    assert_eq!(
+        (
+            counts.deletion_batches_keeping_data_units_matched,
+            counts.file_contents_compared,
+        ),
+        (2, 2),
+        "两个不是最后一批的删除批比过；带文件的两版（起点的第一个文件、292 个单元的那一版）各比过一次整份内容"
+    );
+}
````

## 5、变异表 crates/mutations.tsv 合入前后的差

````diff
@@ -43,7 +43,7 @@
 步 4：被抛弃根的树表读不出时不计数	crates/singlefs-core/src/mount.rs	                None => {\n                    unreadable += 1;\n                    match placements_referenced_by_root_computed_from_the_mapping(	                None => {\n                    match placements_referenced_by_root_computed_from_the_mapping(	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored torn_tree_table_of_an_abandoned_root	torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount
 步 5：抬 F 之后不按新候选集重算影子账	crates/singlefs-core/src/mount.rs	    let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::On {\n        match isolate_slots_referenced_only_by_abandoned_roots(	    let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::Off {\n        match isolate_slots_referenced_only_by_abandoned_roots(	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
 步 5：影子账的豁免不看候选根的 txg 是否 ≥ F	crates/singlefs-core/src/mount.rs	        !is_abandoned(root)\n            && root.checkpoint_txg >= floor\n            && is_inside_the_window(root.checkpoint_txg)	        !is_abandoned(root) && is_inside_the_window(root.checkpoint_txg)	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
-步 5：抬 F 回收的槽不扣住、生效之前就能发出去	crates/singlefs-core/src/mount.rs	        ReclaimedReuse::HeldUntilFloorTakesEffect,	        ReclaimedReuse::Immediately,	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored slots_reclaimed_by_raising	slots_reclaimed_by_raising_the_floor_are_not_handed_out_before_the_floor_takes_effect
+步 5：抬 F 回收的槽不扣住、生效之前就能发出去	crates/singlefs-core/src/mount.rs	            RaiseFloorEntry::Admission => ReclaimedReuse::HeldUntilFloorTakesEffect,	            RaiseFloorEntry::Admission => ReclaimedReuse::Immediately,	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored slots_reclaimed_by_raising	slots_reclaimed_by_raising_the_floor_are_not_handed_out_before_the_floor_takes_effect
 步 5：抬 F 生效之后不放开扣住的槽	crates/singlefs-core/src/mount.rs	    allocator.release_reclaim_holds();\n    Ok(FloorRaisedThroughTheSequence {	    Ok(FloorRaisedThroughTheSequence {	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
 步 5：抬 F 重算影子账时读不出账的被抛弃根不计数	crates/singlefs-core/src/mount.rs	        reclaimed,\n        abandoned_roots_unreadable,\n    })	        reclaimed,\n        abandoned_roots_unreadable: 0,\n    })	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_counts_abandoned_roots	raising_the_floor_counts_abandoned_roots_whose_ledger_is_unreadable
 步 5：提交内生块的 bump 游标不绕开开段之后才隔离或扣住的槽	crates/singlefs-core/src/allocator.rs	        self.allocated[index] || self.isolated[index] || self.held_until_floor_takes_effect[index]	        self.allocated[index]	-p singlefs-core commit_generated_bump_skips	commit_generated_bump_skips_a_slot_isolated_after_the_segment_was_opened
@@ -799,7 +799,7 @@
 实八 (b) 模型：实现拒之前一个写都没发也不带读坏的根环槽	crates/singlefs-harness/src/model.rs	                            .filter(|_| refused_before_any_write);	                            .filter(|_| false);	-p singlefs-harness --test fault_injection_fast_tier -- floor_raise_in_the_same_process	floor_raise_in_the_same_process_after_a_swallowed_root_slot_write_is_refused_for_that_slot_and_recognised_as_left_by_the_lying_device
 实八 (b) 胶水：实现报的读坏的根环槽换不成模型的槽	crates/singlefs-harness/src/model_comparison.rs	        } => Some(model_ring_position(*ring_slot)),	        } => None,	-p singlefs-harness --test fault_injection_fast_tier -- floor_raise_in_the_same_process	floor_raise_in_the_same_process_after_a_swallowed_root_slot_write_is_refused_for_that_slot_and_recognised_as_left_by_the_lying_device
 实八 (b) 按注入点认：被吞的根槽写落在哪个槽一个都不收	crates/singlefs-harness/src/fault_injection.rs	        let root_ring_slots_of_the_swallowed_root_slot_writes = swallowed_root_slot_writes\n            .iter()	        let root_ring_slots_of_the_swallowed_root_slot_writes = swallowed_root_slot_writes\n            .iter()\n            .take(0)	-p singlefs-harness --test fault_injection_fast_tier -- floor_raise_in_the_same_process	floor_raise_in_the_same_process_after_a_swallowed_root_slot_write_is_refused_for_that_slot_and_recognised_as_left_by_the_lying_device
-实八 (d) 模型：写文件内容的发布记事务号 0（重算的行 W 恒为 0，与实现照 D23 已定项 14 第 4 条写的 W 对不上）	crates/singlefs-harness/src/model.rs	            highest_transaction_number_before_this_publish + 1\n        } else {	            0\n        } else {	-p singlefs-harness --test fault_injection_fast_tier -- the_rows_written_after	the_rows_written_after_a_swallowed_root_slot_write_are_recognised_by_the_rows_the_model_recomputes_with_the_record_applied
+实八 (d) 模型：写文件内容的发布记事务号 0（重算的行 W 恒为 0，与实现照 D23 已定项 14 第 4 条写的 W 对不上）	crates/singlefs-harness/src/model.rs	            highest_transaction_number_before_this_publish + transactions_of_this_publish\n        } else {	            0\n        } else {	-p singlefs-harness --test fault_injection_fast_tier -- the_rows_written_after	the_rows_written_after_a_swallowed_root_slot_write_are_recognised_by_the_rows_the_model_recomputes_with_the_record_applied
 实八 (d) 模型：由记录重建被吞根槽的根时 W 不取施加的记录的事务号	crates/singlefs-harness/src/model.rs	            applied_transaction_high_water = applied_transaction_high_water\n                .max(rebuilt.highest_transaction_number_of_its_publish);	            applied_transaction_high_water = applied_transaction_high_water.max(0);	-p singlefs-harness --test fault_injection_fast_tier -- the_rows_written_after	the_rows_written_after_a_swallowed_root_slot_write_are_recognised_by_the_rows_the_model_recomputes_with_the_record_applied
 实八 (d) 按注入点认：实现写的行与重算的行相同反而不认	crates/singlefs-harness/src/fault_injection.rs	                (Some(written), Some(recomputed)) if written == recomputed	                (Some(written), Some(recomputed)) if written != recomputed	-p singlefs-harness --test fault_injection_fast_tier -- the_rows_written_after	the_rows_written_after_a_swallowed_root_slot_write_are_recognised_by_the_rows_the_model_recomputes_with_the_record_applied
 实八 (d) 模型：「写的实例表行」对不上时不带实现写的行	crates/singlefs-harness/src/model.rs	                    disagreement.implementation_rows_written = Some(rows_written.clone());	                    disagreement.implementation_rows_written = None;	-p singlefs-harness --test fault_injection_fast_tier -- the_rows_written_after	the_rows_written_after_a_swallowed_root_slot_write_are_recognised_by_the_rows_the_model_recomputes_with_the_record_applied
@@ -887,7 +887,7 @@
 实审 B3b 第 12 条（分配代对拍两个方向）：模型这一版有、实现没交回的角色不报	crates/singlefs-harness/src/model.rs	            .find(|role| !handed_in.contains(role))	            .find(|_role| false)	-p singlefs-harness --lib -- model::tests::a_role_of_the_version_that_the_implementation_did_not_hand_in	a_role_of_the_version_that_the_implementation_did_not_hand_in_is_reported
 实审 B3b 第 12 条（每一版比内容）：每一版的文件内容只比长度	crates/singlefs-harness/src/model.rs	                if file.content.as_ref() == content.as_slice() {	                if file.content.len() == content.len() {	-p singlefs-harness --lib -- model::tests::each_version_compares_its_file_content	each_version_compares_its_file_content
 实审 B3b 第 12 条（每一版比实例表）：实现交回的行不多于模型的就放过（丢行不报）	crates/singlefs-harness/src/model.rs	                if expected.instance_table_rows.as_slice() == rows.as_slice() {	                if expected.instance_table_rows.len() >= rows.len() {	-p singlefs-harness --lib -- model::tests::each_version_compares_its_instance_table	each_version_compares_its_instance_table
-实审 B3b 第 12 条（胶水）：从数据单元解出的内容倒过来交给模型（每一版比内容那一格在真历史上跑不跑得到）	crates/singlefs-harness/src/model_comparison.rs	        Ok(payload) => ObservedFile::Decoded(payload.to_vec()),	        Ok(payload) => ObservedFile::Decoded(payload.iter().rev().copied().collect()),	-p singlefs-harness --lib -- --include-ignored model_comparison::tests::every_published_version_is_compared	every_published_version_is_compared_by_content_instance_table_and_every_role_both_ways
+实审 B3b 第 12 条（胶水）：从数据单元解出的内容倒过来交给模型（每一版比内容那一格在真历史上跑不跑得到）	crates/singlefs-harness/src/model_comparison.rs	    ObservedFile::Decoded(content)\n}	    ObservedFile::Decoded(content.into_iter().rev().collect())\n}	-p singlefs-harness --lib -- --include-ignored model_comparison::tests::every_published_version_is_compared	every_published_version_is_compared_by_content_instance_table_and_every_role_both_ways
 实审 B3b 第 12 条（胶水）：units 里没有实例表时不按根记录里那条实例表指针交回它的分配记录（模型那一侧照样问这个角色）	crates/singlefs-harness/src/model_comparison.rs	    if !records_of_roles\n        .iter()\n        .any(|(role, _)| *role == ModelUnitRole::InstanceTable)\n    {	    if false {	-p singlefs-harness --lib -- --include-ignored model_comparison::tests::every_published_version_is_compared	every_published_version_is_compared_by_content_instance_table_and_every_role_both_ways
 实审 B3b 第 12 条（胶水）：解实例表时每一片丢掉第一行	crates/singlefs-harness/src/model_comparison.rs	        rows.extend(page.rows.iter().map(model_instance_row));	        rows.extend(page.rows.iter().skip(1).map(model_instance_row));	-p singlefs-harness --lib -- --include-ignored model_comparison::tests::every_published_version_is_compared	every_published_version_is_compared_by_content_instance_table_and_every_role_both_ways
 实审 B3b 第 2 条（崩溃注入补可写挂载）：崩溃后的可写挂载起在空盘上，不是那份崩溃后镜像	crates/singlefs-checker-tier/src/crash_injection.rs	                device.image = sectors.clone();	                device.image = crate::crash::SparseDevice::default();	-p singlefs-checker-tier --test crash_injection_writable_mount_after_the_crash	every_crash_state_is_followed_by_a_writable_mount_one_publish_the_checker_and_second_crashes_in_each_phase
@@ -1320,7 +1320,7 @@
 实审 A3b Q6：抬 F 读根环时根槽重读仍读不出改回「当没有根」	crates/singlefs-core/src/recovery.rs	            RootRingSlotReading::Bad(reread) => {\n                return Err(RootRingSlotKnownToHoldARootStillBadAfterOneReread {\n                    ring_slot,\n                    first_reading,\n                    reread,\n                });\n            }	            RootRingSlotReading::Bad(_reread) => {}	-p singlefs-harness --test unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over	raising_the_floor_while_a_root_ring_slot_known_to_hold_a_root_stays_unreadable_is_refused_before_any_write
 实审 A3b Q6：读根环时读不出的根槽不重读	crates/singlefs-core/src/recovery.rs	            RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => {\n                read_the_slot(ring_slot)\n            }	            RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => {\n                RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable)\n            }	-p singlefs-harness --test unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over	a_system_configuration_write_rereads_an_unreadable_root_ring_slot_once_before_taking_the_effective_floor
 实审 A3b 第 15 条：崩溃镜像的单元扫描候选仍从编译期常量 50176 起	crates/singlefs-harness/src/memory_pool.rs	            .filter(|slot| *slot >= lowest_candidate_slot)	            .filter(|slot| *slot >= lowest_candidate_slot.max(50_176))	-p singlefs-harness --test unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over	a_128_mebibyte_journal_ring_puts_the_unit_area_right_after_the_ring_through_mount_and_checker
-实审 A3b 第 15 条：模型的单元区起点改回编译期常量 50176（小盘单元区墙）	crates/singlefs-harness/src/model.rs	            unit_area_start_slot: model_unit_area_start_slot(journal_ring_bytes),	            unit_area_start_slot: 50_176,	-p singlefs-harness --test random_histories -- --include-ignored unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped	unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped_runs_to_the_end_without_reaching_the_placement_wall
+实审 A3b 第 15 条：模型的单元区起点改回编译期常量 50176（小盘单元区墙）	crates/singlefs-harness/src/model.rs	            unit_area_start_slot: model_unit_area_start_slot(journal_ring_bytes),	            unit_area_start_slot: 50_176,	-p singlefs-harness --lib -- model::tests::	a_unit_area_wall_refusal_far_below_the_wall_that_follows_the_ring_is_judged_a_disagreement
 A3b 善后一：可写挂载重建分配器时生效 F 又先自己读一遍最新根的实例表（那一片读次数回到 (5, 3)）	crates/singlefs-core/src/mount.rs	    let current_records = allocator.records().to_vec();\n    let (newest_table, instance_table_of_the_newest_root) =	    let current_records = allocator.records().to_vec();\n    let _ = effective_rollback_floor(\n        devices,\n        &system_configuration.immutable.region_devices,\n        &system_configuration.immutable.sizes,\n        &system_configuration.immutable.filesystem_identifier,\n    );\n    let (newest_table, instance_table_of_the_newest_root) =	-p singlefs-harness --test a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable	a_newest_instance_table_unreadable_when_the_shadow_ledger_reads_it_is_reread_once_then_refuses_writable_instead_of_turning_the_isolation_off
 A3b 善后一：mkfs 改回默认环的起点（1 GiB 的环把实例表写进环里）	crates/singlefs-core/src/make_filesystem.rs	    let unit_area_start = UnitAreaStart::following_the_journal_ring(ring_bytes)\n        .map_err(MakeFilesystemError::UnitAreaStartOffTheClusterSegmentBoundaryUnsupported)?;	    let unit_area_start = UnitAreaStart::of_the_default_journal_ring();	-p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode	a_journal_ring_reaching_past_784_mebibytes_is_made_with_the_instance_table_at_the_slot_after_the_ring_instead_of_inside_it
 A3b 善后一：mkfs 改回默认环的起点（默认环加一条记录不再按段边界拒）	crates/singlefs-core/src/make_filesystem.rs	    let unit_area_start = UnitAreaStart::following_the_journal_ring(ring_bytes)\n        .map_err(MakeFilesystemError::UnitAreaStartOffTheClusterSegmentBoundaryUnsupported)?;	    let unit_area_start = UnitAreaStart::of_the_default_journal_ring();	-p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode	a_journal_ring_one_record_longer_than_the_default_ends_off_the_cluster_segment_boundary_and_is_refused_and_the_default_ring_is_not
@@ -1865,15 +1865,15 @@
 挂载透明（11-4）：新实例只要比根环里的大（漏了取号的 max 与 + 1）	crates/singlefs-harness/src/mount_transparency.rs	Some(compared.root.instance) == new_instance,	Some(compared.root.instance) > before_the_mount.highest_root_instance_in_the_ring,	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	an_instance_generation_one_too_high_after_the_mount_is_red
 挂载透明（11-4）：恢复写的行里所选根那个实例的 W 写 0	crates/singlefs-harness/src/mount_transparency.rs	                applied_transaction_high_water: maximum_applied_transaction,	                applied_transaction_high_water: 0,	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	instance_table_rows_follow_the_recovery_rule_when_the_crash_image_is_rebuilt_from_a_record
 挂载透明（11-4）：树 ID 水位不比	crates/singlefs-harness/src/mount_transparency.rs	            compared.root.tree_identifier_watermark\n                == first_recovery.root.tree_identifier_watermark,	            true,	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_tree_identifier_watermark_that_moved_with_the_mount_is_red
-挂载透明（11-4）：两棵树的根指针只看 R1 读不读得出、不比	crates/singlefs-harness/src/mount_transparency.rs	            first_recovery.user_visible_tree_root_pointers.is_some()\n                && compared.user_visible_tree_root_pointers\n                    == first_recovery.user_visible_tree_root_pointers,	            first_recovery.user_visible_tree_root_pointers.is_some(),	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	an_inode_tree_root_pointer_that_moved_with_the_mount_is_red
+挂载透明（11-4）：两棵树的根指针只看 R1 读不读得出、不比	crates/singlefs-harness/src/mount_transparency.rs	                expected.is_some() && compared.user_visible_tree_root_pointers.as_ref() == expected,	                expected.is_some(),	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	an_inode_tree_root_pointer_that_moved_with_the_mount_is_red
 挂载透明（11-4）：txg 下界只取根环、不取环里自证记录的 checkpoint_txg	crates/singlefs-harness/src/mount_transparency.rs	                    .max(before_the_mount.highest_record_checkpoint_txg_in_the_ring)\n	                    .max(None)\n	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_checkpoint_txg_that_does_not_clear_the_newest_record_of_the_crash_image_is_red
 挂载透明（11-4）：扫描报告不看施加条数	crates/singlefs-harness/src/mount_transparency.rs	journal.prefix_applied == 0 && journal.above_water == 0,	journal.above_water == 0,	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_recovery_after_the_mount_that_applies_a_record_is_red
 挂载透明（11-4）：挂载加一次发布取「发布之前那一版」时连恢复落到的那一版也算（比成发布之后那一版）	crates/singlefs-harness/src/mount_transparency.rs	                < (recovered_root.checkpoint_txg, recovered_root.instance)	                <= (recovered_root.checkpoint_txg, recovered_root.instance)	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	posix_content_of_the_version_before_the_publish_after_the_mount_is_the_content_recovered_from_the_crash_image
 挂载透明（11-4）：二次崩溃落回 v1 那一版也照挂载写的那一版判	crates/singlefs-harness/src/mount_transparency.rs	            if (compared.root.instance, compared.root.checkpoint_txg)\n                == (	            if false && (compared.root.instance, compared.root.checkpoint_txg)\n                == (	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_second_crash_right_after_the_acquisition_recovers_the_first_version_field_by_field
-拒绝合法（11-4）：有一槽自证不过时 N-配置 复算成假（C331 取甲「按真」那一句丢了）	crates/singlefs-harness/src/mount_transparency.rs	            < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE\n        {\n            return true;\n        }	            < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE\n        {\n            return false;\n        }	-p singlefs-harness --test writable_mount_refusal_judged_against_the_d23_list	a_refusal_after_a_torn_system_configuration_slot_is_legal
+拒绝合法（11-4）：有一槽自证不过时 N-配置 复算成假（C331 取甲「按真」那一句丢了）	crates/singlefs-harness/src/mount_transparency.rs	            < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE\n        {\n            return true;\n        }	            < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE\n        {\n            return false;\n        }	-p singlefs-harness --test writable_mount_refusal_judged_against_the_d23_list	a_refusal_when_the_newest_system_configuration_slot_of_every_device_reports_a_read_error_is_legal
 拒绝合法（11-4）：判据在崩溃镜像上复算为假也放行	crates/singlefs-harness/src/mount_transparency.rs	    if !criterion_holds_on_the_crash_image {	    if false && !criterion_holds_on_the_crash_image {	-p singlefs-harness --test writable_mount_refusal_judged_against_the_d23_list	a_witness_refusal_on_a_healthy_pool_is_a_refusal_for_the_wrong_reason
 拒绝合法（11-4）：最新那条根的实例表读不出在镜像上恒判真	crates/singlefs-harness/src/mount_transparency.rs	.is_none_or(|newest| instance_table_of_root(crash_image, &newest).is_none())	.is_none_or(|_newest| true)	-p singlefs-harness --test writable_mount_refusal_judged_against_the_d23_list	an_unreadable_instance_table_refusal_on_a_healthy_pool_is_a_refusal_for_the_wrong_reason
-拒绝合法（11-4）：拒之后盘变了不报	crates/singlefs-harness/src/mount_transparency.rs	            conditions_not_met.push(RefusalConditionNotMet::DiskChangedByTheRefusedMount);	            let _ = RefusalConditionNotMet::DiskChangedByTheRefusedMount;	-p singlefs-harness --test writable_mount_refusal_judged_against_the_d23_list	a_torn_slot_refusal_that_changed_the_disk_is_illegal
+拒绝合法（11-4）：拒之后盘变了不报	crates/singlefs-harness/src/mount_transparency.rs	            conditions_not_met.push(RefusalConditionNotMet::DiskChangedByTheRefusedMount);	            let _ = RefusalConditionNotMet::DiskChangedByTheRefusedMount;	-p singlefs-harness --test writable_mount_refusal_judged_against_the_d23_list	a_refusal_on_read_errors_of_every_newest_system_configuration_slot_that_changed_the_disk_is_illegal
 拒绝合法（11-4）：单子外的理由也当单子上的 N-配置 判	crates/singlefs-harness/src/mount_transparency.rs	        | MountError::FloorRaiseFailedAfterTheMountsPublishes(_) => None,	        | MountError::FloorRaiseFailedAfterTheMountsPublishes(_) => {\n            Some(RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration)\n        }	-p singlefs-harness --test writable_mount_refusal_judged_against_the_d23_list	a_refusal_whose_reason_is_not_on_the_d23_list_is_left_to_the_campaign
 O1 判官判据 ③ 只比 txg、不比实例（11-4）	crates/singlefs-harness/src/model.rs	        if chosen < persisted {	        if chosen.checkpoint_txg < persisted.checkpoint_txg {	-p singlefs-harness --test crash_recovery_judge_refuses_a_root_older_than_the_newest_persisted_one	landing_on_the_lower_instance_of_the_same_txg_as_the_newest_persisted_root_is_red
 O1 判官去掉判据 ③（11-4）	crates/singlefs-harness/src/model.rs	        if chosen < persisted {	        if false && chosen < persisted {	-p singlefs-harness --test crash_recovery_judge_refuses_a_root_older_than_the_newest_persisted_one	landing_on_an_older_txg_than_the_newest_persisted_root_is_red
@@ -2010,7 +2010,7 @@
 RefFS 翻译（13-4）：故障注入的刷盘被吞档退回照判	crates/singlefs-harness/src/reference_filesystem_translation.rs	        InjectedFault::BarrierIsSwallowed => true,	        InjectedFault::BarrierIsSwallowed => false,	-p singlefs-harness --test reference_filesystem_translation_of_histories	the_reopen_after_a_swallowed_barrier_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish
 挂载透明（13-4）：抬 F 落没落从挂载第一个写起算（暖机与抬 F 退回一起算）	crates/singlefs-harness/src/mount_transparency.rs	                    .get(first_write..)	                    .get(..)	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_second_crash_after_the_warm_up_judges_the_floor_unchanged_and_one_after_the_first_floor_raise_write_leaves_it_unjudged
 挂载透明（13-4）：抬 F 的界取到挂载第一个系统配置槽写（不认抬到的 F）	crates/singlefs-harness/src/mount_transparency.rs	            FloorRaisesInTheMountWrites::FirstWriteAt(first_write)\n        } else {	            FloorRaisesInTheMountWrites::FirstWriteAt(0)\n        } else {	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_second_crash_after_the_warm_up_judges_the_floor_unchanged_and_one_after_the_first_floor_raise_write_leaves_it_unjudged
-崩溃注入（13-4）：崩溃后镜像上的挂载不交回抬 F 从哪个写起	crates/singlefs-checker-tier/src/crash_injection.rs	        floor_raises_in_the_mount_writes_counting_the_continuation(&mounted.output, &mount_writes)	        FloorRaisesInTheMountWrites::NotPushed	-p singlefs-checker-tier --lib	the_writable_mount_on_a_crash_image_names_the_first_floor_raise_write_after_the_warm_up
+崩溃注入（13-4）：崩溃后镜像上的挂载不交回抬 F 从哪个写起	crates/singlefs-checker-tier/src/crash_injection.rs	        FloorRaisesInTheMountWrites::of_the_mount_output(&mounted.output, &mount_writes)	        FloorRaisesInTheMountWrites::NotPushed	-p singlefs-checker-tier --lib	the_writable_mount_on_a_crash_image_names_the_first_floor_raise_write_after_the_warm_up
 E171 装置 取号或暖机被拒不记成 O1-对拍的红（代码轮第三轮判决 K4-c 甲；O1-对拍留在 not_modelled）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs	        self.calls[call_index].model = ModelVerdictOfCall::Disagrees {\n            aspect:\n                singlefs_harness::model::ModelDisagreementAspect::RefusedWhenModelRequiresSuccess	        let _discarded = ModelVerdictOfCall::Disagrees {\n            aspect:\n                singlefs_harness::model::ModelDisagreementAspect::RefusedWhenModelRequiresSuccess	-p singlefs-checker-tier --bin e171_three_oracle_collusion	a_refused_acquire_on_a_stream_without_faults_is_recorded_as_a_comparison_red_and_stops_the_model
 E171 装置 关掉 O1 时被拒的取号照记红（Q3-核关 O1 那一列该写 off、不该有红）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs	        if !self.model_is_enabled() {\n            return;\n        }\n        let call_index = self.calls.len() - 1;\n        self.calls[call_index].model = ModelVerdictOfCall::Disagrees {\n            aspect:\n	        let call_index = self.calls.len() - 1;\n        self.calls[call_index].model = ModelVerdictOfCall::Disagrees {\n            aspect:\n	-p singlefs-checker-tier --bin e171_three_oracle_collusion	with_the_model_switched_off_a_refused_acquire_is_not_a_comparison_red
 E171 装置 取号 panic（实现自崩）也记成 O1-对拍的红（代码轮第三轮判决 K4-c 甲；自崩走自崩那一类）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs	        let Some(failure) = failure else {\n            return;\n        };\n        if !self.model_is_enabled() {	        let failure = failure.expect("panic 也记");\n        if !self.model_is_enabled() {	-p singlefs-checker-tier --bin e171_three_oracle_collusion	a_panic_in_the_acquire_is_an_implementation_crash_and_not_a_comparison_red
@@ -2123,7 +2123,7 @@
 Q-T 改钉：抬 F 的上限取第三新的非空状态（少算一档）；种子 4000000045 第 27 步	crates/singlefs-core/src/mount.rs	        .get(3)\n        .copied()\n        .or(oldest_valid_root)?;	        .get(2)\n        .copied()\n        .or(oldest_valid_root)?;	-p singlefs-harness --test random_histories -- seed_4000000045_on_240_slot_devices	seed_4000000045_on_240_slot_devices_with_the_admission_skipped_raises_the_floor_to_19_reclaiming_135_placements
 Q-T 改钉：抬 F 的上限取第三新的非空状态（少算一档）；种子 4000000001 第 30 步	crates/singlefs-core/src/mount.rs	        .get(3)\n        .copied()\n        .or(oldest_valid_root)?;	        .get(2)\n        .copied()\n        .or(oldest_valid_root)?;	-p singlefs-harness --test random_histories -- --include-ignored seed_4000000001_on_240_slot_devices	seed_4000000001_on_240_slot_devices_with_the_admission_skipped_raises_the_floor_to_7_in_two_publishes
 Q-T 改钉：单元区墙取样点那组比重里 unlink 那一格是 0（准入关着那一段生成的历史跟着变）	crates/singlefs-harness/src/history.rs	            (HistoryOperationKind::CloseAndMountWritable, 3),\n            (HistoryOperationKind::Unlink, 3),	            (HistoryOperationKind::CloseAndMountWritable, 3),\n            (HistoryOperationKind::Unlink, 0),	-p singlefs-harness --test random_histories -- --include-ignored unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped	unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped_runs_to_the_end_without_reaching_the_placement_wall
-三·2 删除批代码轮 U14（用户 2026-10-03 定「照 D16 做成可写」）：挂载续做推满仍放不下照旧交回 MountError::Publish（定案之前的做法），不做成可写	crates/singlefs-core/src/mount.rs	                continuation.outcome =\n                    DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises {\n                        last_refusal: refusal,\n                        stop,\n                    };\n                return Ok(continuation);	                let _ = stop;\n                return Err(MountError::Publish(Box::new(PublishSequenceFailed {\n                    cause: refusal,\n                    writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,\n                    writes_of_persisted_publishes: Vec::new(),\n                    writes_of_failed_publishes: Vec::new(),\n                })));	-p singlefs-harness --test admission_raises_the_floor_before_refusing -- a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable	a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_with_the_queue_left_and_its_publishes_on_disk
+三·2 删除批代码轮 U14（用户 2026-10-03 定「照 D16 做成可写」）：挂载续做推满仍放不下照旧交回 MountError::Publish（定案之前的做法），不做成可写	crates/singlefs-core/src/mount.rs	                continuation.outcome =\n                    DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises {\n                        last_refusal: refusal,\n                        stop,\n                    };\n                return Ok(continuation);	                let _ = stop;\n                return Err(MountError::Publish(Box::new(PublishSequenceFailed {\n                    cause: refusal,\n                    writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,\n                    writes_of_persisted_publishes: Vec::new(),\n                    writes_of_failed_publishes: Vec::new(),\n                })));	-p singlefs-harness --test admission_raises_the_floor_before_refusing -- a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable	a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue
 三·2 删除批代码轮 U9：挂载续做里的一步没发成、不是空间不够时错里的已落盘写账照旧交空表（mount.rs PublishSequenceFailed 契约）	crates/singlefs-core/src/mount.rs	                writes_of_the_publishes_before_the_continuation\n                    .iter()\n                    .cloned()\n                    .chain(continuation.writes_of_the_persisted_publishes_in_txg_order())\n                    .collect();	                Vec::new();	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version -- a_mount_whose_continuation_step_fails_on_a_device_error	a_mount_whose_continuation_step_fails_on_a_device_error_hands_back_the_writes_of_every_publish_it_landed
 三·2 删除批代码轮 U10：会话先走空之后拒成 NoFileVersionToChange 时不带走空发成的那几版	crates/singlefs-core/src/mounted_session.rs	    UserChangeRefused::NoFileVersionToChange {\n        floor_raises,\n        refusals_that_pushed_the_floor_raises,\n        versions_published,\n    }\n}	    let _ = versions_published;\n    UserChangeRefused::NoFileVersionToChange {\n        floor_raises,\n        refusals_that_pushed_the_floor_raises,\n        versions_published: Vec::new(),\n    }\n}	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version -- an_overwrite_after_rolling_back_to_the_intent	an_overwrite_after_rolling_back_to_the_intent_walks_the_queue_and_its_refusal_carries_every_version_it_published
 三·2 删除批代码轮 U11：随机历史执行器在删除中的版本上照旧跳过覆盖写（模型比不到走空那几版）	crates/singlefs-harness/src/history.rs	    let queue_is_walked_first = !current_file_version.deleted_inodes.is_empty();	    if !current_file_version.deleted_inodes.is_empty() {\n        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionStillHasTheFirstFile);\n    }\n    let queue_is_walked_first = false;	-p singlefs-harness --test unlink_in_histories_judged_by_the_model -- an_overwrite_after_rolling_back_to_the_intent	an_overwrite_after_rolling_back_to_the_intent_is_judged_by_the_model_through_the_walked_step
@@ -2135,8 +2135,8 @@
 遗留改动代码轮 LR4（C393 (b)-从映射现算最后一支）：被抛弃根的账与从映射现算重读一次仍都不成时不拒可写，当它什么都不引用	crates/singlefs-core/src/mount.rs	                                Some(placements) => placements,\n                                None => return Err(\n                                    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(\n                                        StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {\n                                            abandoned_root: RollbackTarget {\n                                                instance: root.instance,\n                                                checkpoint_txg: root.checkpoint_txg,\n                                            },\n                                        },\n                                    )),\n                                ),	                                Some(placements) => placements,\n                                None => Vec::new(),	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area	an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area_refuses_the_writable_mount_instead_of_panicking
 遗留改动代码轮 LR3（攻方变异 raise-no-restore）：抬 F 那一串的影子账交 Err 时不把分配器换回抬 F 之前那一份	crates/singlefs-core/src/mount.rs	            Ok(computed) => computed.abandoned_roots_unreadable,\n            Err(error) => {\n                *allocator = allocator_before_the_raise;\n                return Err(error);\n            }	            Ok(computed) => computed.abandoned_roots_unreadable,\n            Err(error) => {\n                return Err(error);\n            }	-p singlefs-harness --test an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping -- a_floor_raise_whose_shadow_ledger_meets	a_floor_raise_whose_shadow_ledger_meets_an_abandoned_root_unreadable_by_account_and_mapping_is_refused_and_restores_the_allocator
 遗留改动代码轮 LR2（攻方变异 map-first-instance-page-only）：被抛弃根的账读不出、从映射现算时只认实例表第 0 片	crates/singlefs-core/src/mount.rs	    for page in instance_table_page_pointers_as_far_as_readable(devices, root) {\n        located.push((page.locations, TransactionUnit::InstanceTable.span_slots()));	    for page in instance_table_page_pointers_as_far_as_readable(devices, root)\n        .into_iter()\n        .take(1)\n    {\n        located.push((page.locations, TransactionUnit::InstanceTable.span_slots()));	-p singlefs-harness --test an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping -- an_abandoned_root_with_a_two_page_instance_table	an_abandoned_root_with_a_two_page_instance_table_and_an_unreadable_account_isolates_both_pages_from_the_mapping
-三·2 挂载续做（m3-batch11 第 1 条）：抬 F 的界不认续做里推的那几串（准入没推、续做里推了的那一形交「没推」，二次崩溃崩在那一串里时 F 照没推判）	crates/singlefs-checker-tier/src/crash_injection.rs	        let first_pushed_by_the_continuation = self.pushed_by_the_continuation.first()?;	        let first_pushed_by_the_continuation = self.pushed_by_the_continuation.first().filter(|_| false)?;	-p singlefs-checker-tier --lib -- crash_injection::tests::a_floor_raise_pushed_inside_the_continuation_is_located_between_the_continuation_steps	a_floor_raise_pushed_inside_the_continuation_is_located_between_the_continuation_steps
-三·2 挂载续做（m3-batch11 第 1 条）：准入推了抬 F 之后又有续做时界不把续做的写算进去（界落在续做那一步的单元写上、取不出）	crates/singlefs-checker-tier/src/crash_injection.rs	                writes_of(&self.pushed_by_the_admission)\n                    + continuation_steps_after(None)\n	                writes_of(&self.pushed_by_the_admission)\n	-p singlefs-checker-tier --lib -- crash_injection::tests::floor_raises_pushed_by_the_mount_admission_are_located_before_the_continuation_that_follows_them	floor_raises_pushed_by_the_mount_admission_are_located_before_the_continuation_that_follows_them
+三·2 挂载续做（m3-batch11 第 1 条）：抬 F 的界不认续做里推的那几串（准入没推、续做里推了的那一形交「没推」，二次崩溃崩在那一串里时 F 照没推判）	crates/singlefs-harness/src/mount_transparency.rs	        let first_pushed_by_the_continuation = self.pushed_by_the_continuation.first()?;	        let first_pushed_by_the_continuation = self.pushed_by_the_continuation.first().filter(|_| false)?;	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_floor_raise_pushed_inside_the_continuation_is_located_between_the_continuation_steps
+三·2 挂载续做（m3-batch11 第 1 条）：准入推了抬 F 之后又有续做时界不把续做的写算进去（界落在续做那一步的单元写上、取不出）	crates/singlefs-harness/src/mount_transparency.rs	                writes_of(&self.pushed_by_the_admission)\n                    + continuation_steps_after(None)\n	                writes_of(&self.pushed_by_the_admission)\n	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	floor_raises_pushed_by_the_mount_admission_are_located_before_the_continuation_that_follows_them
 三·2 U12：挂载在续做里交回错时认不出续做的那几次发布（续做的写并进暖机段）	crates/singlefs-checker-tier/src/crash_injection.rs	        .find(|(_, writes)| writes_a_structure_only_the_continuation_changes(writes))	        .find(|(_, writes)| false && writes_a_structure_only_the_continuation_changes(writes))	-p singlefs-checker-tier --lib -- crash_injection::tests::a_mount_that_handed_back_an_error_inside_the_continuation_marks_the_continuation_by_the_persisted_publishes	a_mount_that_handed_back_an_error_inside_the_continuation_marks_the_continuation_by_the_persisted_publishes
 LR7：设备侧日志折清零段不守长度 0（声明长度 0 时铺满的判定交回「用掉 0 条」，折叠原地不动）	crates/singlefs-checker-tier/src/device_log.rs	    if declared_length == 0 {	    if false && declared_length == 0 {	-p singlefs-checker-tier --lib -- device_log::	device_log::tests::a_declared_zero_fill_of_length_zero_tiles_no_entry_and_leaves_the_log_unchanged
 LR7：设备侧日志折清零段时接在末端的那一条不查段尾（越过段尾的接续写并进来，「已铺满的末端停在段尾」那一句断言 panic）	crates/singlefs-checker-tier/src/device_log.rs	let continues_at_cursor = piece.start == covered_up_to && piece.end <= fill_end;	let continues_at_cursor = piece.start == covered_up_to;	-p singlefs-checker-tier --lib -- device_log::	device_log::tests::zero_fill_whose_piece_continuing_at_the_covered_end_runs_past_the_end_does_not_fold
@@ -2174,3 +2174,40 @@
 池级 checker 跟上（D23 已定项 15）：重放那一版不走新根段里的实例表指针	crates/singlefs-checker/src/walk.rs	        self.walk_instance_table_chain(&first_page_pointer, None);\n        let allocation_record_tree_root =	        let allocation_record_tree_root =	-p singlefs-harness --test replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment -- the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment	the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment
 池级 checker 跟上（D23 已定项 15）：重放那一版不走新根段里的分配记录树根	crates/singlefs-checker/src/walk.rs	            &version.new_root_segment[NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER];\n        if !parse_node_pointer(allocation_record_tree_root).all_zero {	            &version.new_root_segment[NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER];\n        if false && !parse_node_pointer(allocation_record_tree_root).all_zero {	-p singlefs-harness --test replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment -- the_replayed_version_invariant_walks_the_allocation_record_tree_root_of_the_new_root_segment	the_replayed_version_invariant_walks_the_allocation_record_tree_root_of_the_new_root_segment
 E171 装置 取号那一核「一份自证过的都没有」写成「不多于两份」、不带故障的盘上每次取号都拒（取代取甲时那一行「少于两槽」写成「不多于两槽」；O1-对拍那一栏该红，装置不记红时这一行没红）	crates/singlefs-core/src/transaction.rs	            if slots_of_this_device.self_verified.is_empty() {	            if slots_of_this_device.self_verified.len() <= 2 {	-p singlefs-checker-tier --bin e171_three_oracle_collusion	an_acquire_and_a_warm_up_on_a_fresh_pool_leave_the_comparison_column_without_a_red
+三·2 删除批第 12 批丙 1（用户 2026-10-03 定「丙 1 进下一批」）：卸载那一串回收的槽照旧扣到生效，续做推满仍放不下之后卸载在预演里取不到落点	crates/singlefs-core/src/mount.rs	            RaiseFloorEntry::Unmount => ReclaimedReuse::Immediately,	            RaiseFloorEntry::Unmount => ReclaimedReuse::HeldUntilFloorTakesEffect,	-p singlefs-harness --test admission_raises_the_floor_before_refusing	a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue
+三·2 删除批第 12 批丙 1：卸载那一串回收的槽照旧扣到生效，续做走到空、F 没抬、可删的单元在 defer 里的盘上卸载被拒	crates/singlefs-core/src/mount.rs	            RaiseFloorEntry::Unmount => ReclaimedReuse::Immediately,	            RaiseFloorEntry::Unmount => ReclaimedReuse::HeldUntilFloorTakesEffect,	-p singlefs-harness --test admission_raises_the_floor_before_refusing	a_mount_whose_continuation_walks_to_empty_without_raising_the_floor_is_unmounted_on_the_slots_its_unmount_sequence_reclaims
+三·2 删除批第 12 批丙 1 (a)：卸载那一串回收的槽照旧扣到生效，先写系统配置之后掉电那一刀摆不出来	crates/singlefs-core/src/mount.rs	            RaiseFloorEntry::Unmount => ReclaimedReuse::Immediately,	            RaiseFloorEntry::Unmount => ReclaimedReuse::HeldUntilFloorTakesEffect,	-p singlefs-harness --test admission_raises_the_floor_before_refusing	an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version
+三·2 删除批第 12 批丙 1 (b)：卸载那一串回收的槽照旧扣到生效，只剩一块盘那一刀摆不出来	crates/singlefs-core/src/mount.rs	            RaiseFloorEntry::Unmount => ReclaimedReuse::Immediately,	            RaiseFloorEntry::Unmount => ReclaimedReuse::HeldUntilFloorTakesEffect,	-p singlefs-harness --test admission_raises_the_floor_before_refusing	after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_surviving_device_recovers_its_own_newest_root_below_the_floor
+三·2 删除批第 12 批代码轮 V1：续做里推的抬 F 自己报了不是空间不够的错时原样交回，不装进 FloorRaiseFailedAfterTheMountsPublishes	crates/singlefs-core/src/mount.rs	                return Err(MountError::FloorRaiseFailedAfterTheMountsPublishes(\n                    Box::new(FloorRaiseFailedAfterTheMountsPublishes {\n                        cause,\n                        writes_of_persisted_publishes: [\n                            writes_of_the_publishes_before_the_continuation,\n                            continuation.writes_of_the_persisted_publishes_in_txg_order(),\n                        ]\n                        .concat(),\n                        floor_raises: floor_raises_pushed_by_the_admission\n                            .iter()\n                            .cloned()\n                            .chain(continuation.floor_raises)\n                            .collect(),\n                    }),\n                ));	                return Err(cause);	-p singlefs-harness --test admission_raises_the_floor_before_refusing	a_floor_raise_failing_on_a_device_error_in_the_continuation_hands_back_the_writes_of_the_mounts_publishes
+三·2 删除批第 12 批代码轮 V2：续做那一格交 Publish 时推过的每一串先写系统配置那一步不进错（准入推过的起点）	crates/singlefs-core/src/mount.rs	                writes_before_the_first_publish: system_configuration_writes_of_the_floor_raises(\n                    floor_raises_pushed_by_the_admission\n                        .iter()\n                        .chain(&continuation.floor_raises),\n                ),	                writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,	-p singlefs-harness --test admission_raises_the_floor_before_refusing	a_continuation_step_failing_after_the_admission_raised_the_floor_hands_back_every_landed_write
+三·2 删除批第 12 批代码轮 V2：续做那一格交 Publish 时推过的每一串先写系统配置那一步不进错（续做推过的起点）	crates/singlefs-core/src/mount.rs	                writes_before_the_first_publish: system_configuration_writes_of_the_floor_raises(\n                    floor_raises_pushed_by_the_admission\n                        .iter()\n                        .chain(&continuation.floor_raises),\n                ),	                writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,	-p singlefs-harness --test admission_raises_the_floor_before_refusing	a_continuation_step_failing_after_the_continuation_raised_the_floor_hands_back_every_landed_write
+三·2 删除批第 12 批代码轮 V2：续做那一格交 Publish 时推过的每一串先写系统配置那一步不进错（两边都推过的起点）	crates/singlefs-core/src/mount.rs	                writes_before_the_first_publish: system_configuration_writes_of_the_floor_raises(\n                    floor_raises_pushed_by_the_admission\n                        .iter()\n                        .chain(&continuation.floor_raises),\n                ),	                writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,	-p singlefs-harness --test admission_raises_the_floor_before_refusing	a_continuation_step_failing_after_both_the_admission_and_the_continuation_raised_the_floor_hands_back_every_landed_write
+三·2 删除批第 12 批代码轮 V3（攻方 MU1）：续做之前的写账去掉准入推的那几串空发布	crates/singlefs-core/src/mount.rs	            .map(|version| version.writes().clone())\n            .chain(\n                space_admission\n                    .floor_raises()\n                    .iter()\n                    .flat_map(|raised| raised.publishes.iter())\n                    .map(|publish| publish.writes.clone()),\n            )\n            .collect();	            .map(|version| version.writes().clone())\n            .collect();	-p singlefs-harness --test admission_raises_the_floor_before_refusing	a_continuation_step_failing_after_the_admission_raised_the_floor_hands_back_every_landed_write
+三·2 删除批第 12 批代码轮 V3（攻方 MU2）：续做的写账去掉续做推的那几串空发布	crates/singlefs-core/src/mount.rs	            .steps_published\n            .iter()\n            .chain(\n                self.floor_raises\n                    .iter()\n                    .flat_map(|raised| raised.publishes.iter()),\n            )\n            .collect();	            .steps_published\n            .iter()\n            .collect();	-p singlefs-harness --test admission_raises_the_floor_before_refusing	a_continuation_step_failing_after_the_continuation_raised_the_floor_hands_back_every_landed_write
+三·2 删除批第 12 批续做结局的守护：推满仍放不下时结局报成 WalkedToEmpty 而不是 StillShortAfterTheFloorRaises	crates/singlefs-core/src/mount.rs	                continuation.outcome =\n                    DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises {\n                        last_refusal: refusal,\n                        stop,\n                    };\n                return Ok(continuation);	                let _ = (refusal, stop);\n                return Ok(continuation);	-p singlefs-harness --test admission_raises_the_floor_before_refusing -- a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable	a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue
+三·2 删除批第 12 批 Q-F（用户 2026-10-03 定「算改动，改实现」）：意图那次照旧带上一版的改动计数	crates/singlefs-core/src/transaction.rs	                change_count: change_count_of_the_intent,	                change_count: record.change_count,	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version	the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_own_publish
+三·2 删除批第 12 批 Q-F：中间一批照旧带上一版的改动计数	crates/singlefs-core/src/transaction.rs	                change_count: change_count_of_the_batch,	                change_count: record.change_count,	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version	the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_own_publish
+三·2 删除批第 12 批代码轮 U8：墓碑预付的 H 照旧读内存里的墓碑树形状，不读根节点码 2 头	crates/singlefs-core/src/transaction.rs	            u64::from(root_header.level).saturating_sub(1)	            {\n                let _ = root_header;\n                resolved.tombstone_tree.internal_levels()\n            }	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version	the_tombstone_prepay_of_an_overwrite_reads_the_tombstone_tree_height_from_the_root_node_header
+三·2 挂载透明（m3-batch12 第 1 条）：挂载续做过时用户可见树根指针不认续做那几步、照旧比 R1 那一版	crates/singlefs-harness/src/mount_transparency.rs	            .filter(|(step, _)| step.0 == version.0 && step.1 <= version.1)	            .filter(|_step| false)	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	after_a_mount_that_continued_the_deletion_the_two_tree_root_pointers_are_those_of_its_last_continuation_step
+三·2 挂载透明（m3-batch12 第 1 条）：挂载抬到的 F 不认续做里推的那几串（准入没推、续做推了的那一形交「没推」）	crates/singlefs-harness/src/mount_transparency.rs	            .chain(output.floor_raises_pushed_by_the_continuation.iter())	            .chain(output.floor_raises_pushed_by_the_continuation.iter().take(0))	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_mount_whose_continuation_raised_the_floor_leaves_the_effective_floor_at_the_last_one_the_continuation_raised_to
+三·2 V6（m3-batch12 第 5 条）：准入与续做都推了抬 F 时界不算续做推的那几串的写（界落在续做那一步的写上、取不出）	crates/singlefs-harness/src/mount_transparency.rs	                    + continuation_steps_after(None)\n                    + writes_of(&self.pushed_by_the_continuation),	                    + continuation_steps_after(None),	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	floor_raises_pushed_by_both_the_admission_and_the_continuation_are_located_from_the_first_admission_raise
+三·2 删完那一版（m3-batch12 第 2 条）：崩溃后可写挂载之后那一次发布在删完那一版上照旧调覆盖写（被拒成写一个不在树里的 inode）	crates/singlefs-checker-tier/src/crash_injection.rs	                PoolVersion::WithTreesAndNoFile(current) => publish_new_inodes(\n                    &mut writer,\n                    &mut allocator,\n                    current,\n                    1,\n                    FIXED_WRITE_TIME_SECONDS,\n                    mounted.output.instance,\n                )	                PoolVersion::WithTreesAndNoFile(current) => publish_overwrite(\n                    &mut writer,\n                    &mut allocator,\n                    current,\n                    file,\n                    mounted.output.instance,\n                )	-p singlefs-checker-tier --lib -- crash_injection::tests::the_publish_after_a_writable_mount_on_the_emptied_version_creates_an_inode_and_is_published	the_publish_after_a_writable_mount_on_the_emptied_version_creates_an_inode_and_is_published
+三·2 U4（m3-batch12 第 3 条）：续做那一段不摆二次崩溃（Continuation 段跳过，与用例里 continue 跳过同一个效果）	crates/singlefs-checker-tier/src/crash_injection.rs	        if writes_of_the_phase.is_empty() {\n            continue;\n        }	        if writes_of_the_phase.is_empty() || phase == WritableMountPhase::Continuation {\n            continue;\n        }	-p singlefs-checker-tier --test crash_injection_writable_mount_after_the_crash	every_crash_state_is_followed_by_a_writable_mount_one_publish_the_checker_and_second_crashes_in_each_phase
+三·2 U4（m3-batch12 第 3 条）：续做那一段的二次崩溃不再挂（I-8.4 的 R-换 在续做段一次都不判）	crates/singlefs-checker-tier/src/crash_injection.rs	    tally.remounts_after_a_second_crash += 1;\n	    if phase == WritableMountPhase::Continuation {\n        return;\n    }\n    tally.remounts_after_a_second_crash += 1;\n	-p singlefs-checker-tier --lib -- crash_injection::tests::the_remount_after_each_second_crash_reads_the_same_content_as_the_uninterrupted_mount_in_every_mount_phase	the_remount_after_each_second_crash_reads_the_same_content_as_the_uninterrupted_mount_in_every_mount_phase
+三·2 U4（m3-batch12 第 3 条）：续做那一段的二次崩溃不判挂载透明	crates/singlefs-checker-tier/src/crash_injection.rs	    let floor_raises = floor_raises_in_the_mount_writes.at_a_second_crash(persisted_in_the_mount);	    if phase == WritableMountPhase::Continuation {\n        return Vec::new();\n    }\n    let floor_raises = floor_raises_in_the_mount_writes.at_a_second_crash(persisted_in_the_mount);	-p singlefs-checker-tier --lib -- crash_injection::tests::mount_transparency_is_judged_after_every_writable_mount_and_inside_every_mount_phase_without_a_red_field	mount_transparency_is_judged_after_every_writable_mount_and_inside_every_mount_phase_without_a_red_field
+三·2 挂载透明（m3-batch12 第 1 条）：崩溃注入判挂载透明时不交续做那几步的根指针（续做过的崩溃状态上用户可见树根指针照旧比 R1 那一版）	crates/singlefs-checker-tier/src/crash_injection.rs	        judge_mount_transparency(before_the_mount, after_the_action, action, continuation);	        judge_mount_transparency(\n            before_the_mount,\n            after_the_action,\n            action,\n            &UserVisibleTreeRootPointersOfTheContinuation::NOT_CONTINUED,\n        );	-p singlefs-checker-tier --lib -- crash_injection::tests::mount_transparency_is_judged_after_every_writable_mount_and_inside_every_mount_phase_without_a_red_field	mount_transparency_is_judged_after_every_writable_mount_and_inside_every_mount_phase_without_a_red_field
+三·2 V1（m3-batch12 第 4 条）：挂载交回 FloorRaiseFailedAfterTheMountsPublishes 时划不出续做（续做的写并进暖机段）	crates/singlefs-checker-tier/src/crash_injection.rs	        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {\n            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(\n                &failed.writes_of_persisted_publishes,\n                &[],\n            )\n        }	        MountError::FloorRaiseFailedAfterTheMountsPublishes(_failed) => None,	-p singlefs-checker-tier --lib -- crash_injection::tests::a_mount_whose_floor_raise_failed_after_a_continuation_step_marks_the_continuation_by_the_persisted_publishes	a_mount_whose_floor_raise_failed_after_a_continuation_step_marks_the_continuation_by_the_persisted_publishes
+代码轮 U13：模型把文件当一个数据单元（一份内容切几个数据单元恒取 1）	crates/singlefs-harness/src/model.rs	    content_length_in_bytes\n        .div_ceil(data_unit_payload_capacity_in_bytes())\n        .max(1)	    content_length_in_bytes.min(1).max(1)	-p singlefs-harness --test random_histories -- deleting_a_file_of_two_lower_leaves	deleting_a_file_of_two_lower_leaves_publishes_a_batch_keeping_data_units_that_the_model_compares
+代码轮 U13：模型的走空一步一律当最后一批（删除中的那一版不按剩下的数据单元数分批）	crates/singlefs-harness/src/model.rs	        let step = if data_units_remaining > MODEL_DELETION_BATCH_DATA_UNITS {	        let step = if data_units_remaining > MODEL_DELETION_BATCH_DATA_UNITS && false {	-p singlefs-harness --test random_histories -- deleting_a_file_of_two_lower_leaves	deleting_a_file_of_two_lower_leaves_then_rolling_back_to_its_intent_walks_two_steps_before_the_next_change
+代码轮 U13：胶水只解文件第 0 个数据单元（多个单元的文件内容接不全）	crates/singlefs-harness/src/model_comparison.rs	            Ok(payload) => content.extend_from_slice(payload),	            Ok(payload) => {\n                content.extend_from_slice(payload);\n                break;\n            }	-p singlefs-harness --test random_histories -- deleting_a_file_of_two_lower_leaves	deleting_a_file_of_two_lower_leaves_publishes_a_batch_keeping_data_units_that_the_model_compares
+代码轮 U13：模型数 extent 树下段的节点只数叶、不数内部节点（顺序写的占槽上界少算）	crates/singlefs-harness/src/model.rs	        nodes += nodes_on_this_level;	        nodes += 0;	-p singlefs-harness --lib -- model::tests::	the_lower_segment_of_a_file_has_a_leaf_per_144_data_units_and_an_internal_node_per_147_children
+删除批 #11 改钉值报告第二节候选：清单为空时撞上记账多于遍历那一形也被「已知红」接走（classify_failure 查清单之后一律接住第 0 条）	crates/singlefs-harness/src/history.rs	        .position(|form| (form.matches)(&observation))\n    {	        .position(|form| (form.matches)(&observation))\n        .or(Some(0))\n    {	-p singlefs-harness --lib -- history::tests::with_the_known_red_list_empty	with_the_known_red_list_empty_an_allocated_statistic_above_walked_is_a_new_finding_whatever_its_mechanism
+遗留改动代码轮 LR1（攻方变异 slot1-only-both）：见证读只看每块盘的槽 1，最新的偶数世代那一槽（槽 0）两盘都报读错照样可写挂载	crates/singlefs-core/src/recovery.rs	    for offset in [0, slot_spacing_in_bytes] {	    for offset in [slot_spacing_in_bytes] {	-p singlefs-harness --test a_writable_mount_with_system_configuration_slot_zero_unreadable	a_writable_mount_is_refused_when_slot_zero_of_every_device_reports_a_read_error
+遗留改动代码轮 LR1（攻方变异 slot1-only-both）：取号读见证值只看每块盘的槽 1，槽 0 两盘都报读错照样取号	crates/singlefs-core/src/recovery.rs	    for offset in [0, slot_spacing_in_bytes] {	    for offset in [slot_spacing_in_bytes] {	-p singlefs-harness --test a_writable_mount_with_system_configuration_slot_zero_unreadable	an_acquisition_is_refused_before_any_write_when_slot_zero_of_every_device_reports_a_read_error
+遗留改动代码轮 LR1 第二块盘那一格：判不出的前一半只看盘表次序第一块盘报没报读错	crates/singlefs-core/src/recovery.rs	        .any(|slots| slots.slots_reporting_a_read_error > 0)	        .take(1)\n        .any(|slots| slots.slots_reporting_a_read_error > 0)	-p singlefs-harness --test a_writable_mount_with_system_configuration_slot_zero_unreadable	a_writable_mount_is_refused_when_only_the_second_device_reports_a_read_error_on_slot_zero_and_the_first_reads_back_zeros
+遗留改动代码轮 LR1 逐盘核的 Err(device) 出口：取号之前的逐盘核读不出一份自证过的系统配置的盘跳过、不拒	crates/singlefs-core/src/mount.rs	        let Some(newest_self_verified) = newest_self_verified else {\n            return Err(device);\n        };	        let Some(newest_self_verified) = newest_self_verified else {\n            continue;\n        };	-p singlefs-harness --test a_writable_mount_with_system_configuration_slot_zero_unreadable	a_writable_mount_is_refused_when_the_slots_of_the_second_device_turn_unreadable_only_for_the_check_of_every_device
+遗留改动代码轮 LR1 读错形：判不出的后一半拿掉，盘 1 每次读都报错而盘 0 两槽都自证过那一形也判成判不出	crates/singlefs-core/src/recovery.rs	            .any(SystemConfigurationSlotsOfOneDevice::both_slots_self_verified)	            .any(|_slots_of_this_device| false)	-p singlefs-harness --test fsync_drop_and_devices_without_the_selected_version -- mount_writable_while_every_read_of_device_one_fails_is_refused_by_the_witness_before_any_write	mount_writable_while_every_read_of_device_one_fails_is_refused_by_the_witness_before_any_write
+挂载时读不出的槽最大值取区间头那一段	crates/singlefs-core/src/mount.rs	    let tail_of_the_interval = first_txg.0 - txgs_walked_at_each_end_of_the_interval..first_txg.0;	    let tail_of_the_interval = head_of_the_interval.clone();	-p singlefs-core --lib -- unreadable_ring_slots_inside_the_window_tests	walking_only_the_two_ends_of_the_interval_matches_walking_every_txg_of_it
+代码轮 U13 映射树分裂：胶水把中央映射树根之下的节点当成模型罩不到的池（panic）	crates/singlefs-harness/src/model_comparison.rs	        TransactionUnit::MappingTreeNodeBelowTheRoot(_) => return None,	        TransactionUnit::MappingTreeNodeBelowTheRoot(position) => {\n            panic!("模型今天只罩中央映射树一个节点的池：{position:?}")\n        }	-p singlefs-harness --test random_histories -- a_file_of_three_lower_leaves_grows_the_central_mapping_tree_below_its_root	a_file_of_three_lower_leaves_grows_the_central_mapping_tree_below_its_root_and_the_model_compares_every_version
+代码轮 U13 映射树分裂：模型照旧按一个节点算中央映射树的占槽（根之下的节点不进占槽上界）	crates/singlefs-harness/src/model.rs	            slots_written += (mapping_nodes - 1) * ModelUnitRole::MappingTree.span_in_slots()	            slots_written += 0 * ModelUnitRole::MappingTree.span_in_slots()	-p singlefs-harness --lib -- model::tests::a_sequential_write_of_three_lower_leaves_counts_the_central_mapping_tree_nodes	a_sequential_write_of_three_lower_leaves_counts_the_central_mapping_tree_nodes_below_the_root_in_the_occupied_slots
+代码轮 U13 映射树分裂：中央映射树节点数的上界不随条目数涨（叶切出来的右半不算，树恒一个节点）	crates/singlefs-harness/src/model.rs	        + 2 * (entries / smallest_half_of_a_split(central_mapping_tree_leaf_entry_capacity()));	        + 0 * (entries / smallest_half_of_a_split(central_mapping_tree_leaf_entry_capacity()));	-p singlefs-harness --lib -- model::tests::central_mapping_tree_nodes_grow_by_two_per_147_entries	central_mapping_tree_nodes_grow_by_two_per_147_entries_and_keep_the_previous_bound
````
