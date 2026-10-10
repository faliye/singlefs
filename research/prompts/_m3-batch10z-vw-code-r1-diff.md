# 根环实现第 10z 批第二十二、二十三块：代码轮第一轮改动全文（2026-10-09）

两块实现员各交的 `crates.patch` 原样，按合入次序：第二十二块、第二十三块。各块变异表追加行与整行替换行原样附在各块之后；合入时没有改动任何一行。

## 第二十二块

````diff
diff --git a/crates/singlefs-core/src/block_device.rs b/crates/singlefs-core/src/block_device.rs
index 76dbf09..076d023 100644
--- a/crates/singlefs-core/src/block_device.rs
+++ b/crates/singlefs-core/src/block_device.rs
@@ -310,8 +310,9 @@ impl FileBackedBlockDevice {
     }
 }
 
-/// 请求要按物理块对齐、不越过设备末尾；两个后端共用这一份判据。
-fn check_aligned_and_in_range(
+/// 请求要按物理块对齐、不越过设备末尾；两个后端与预演用的只进内存的盘（`mounted_session` 的 `DeviceKeepingWritesInMemory`：
+/// 写不交给底下那块盘，不判就收得下一个真盘会拒的写）共用这一份判据。
+pub(crate) fn check_aligned_and_in_range(
     offset: DeviceOffsetInBytes,
     length: u64,
     physical_block_size: PhysicalBlockSizeInBytes,
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index d45a0fa..b8eb77e 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -24,7 +24,8 @@ use crate::address::{
 };
 use crate::allocator::{PlacementOnDevice, PoolAllocator};
 use crate::block_device::{
-    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
+    check_aligned_and_in_range, BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes,
+    WriteDurability,
 };
 use crate::make_filesystem::MakeFilesystemParameters;
 use crate::mount::{
@@ -172,14 +173,16 @@ pub struct NoSpaceAfterRaisingTheFloor {
     pub stop: FloorRaiseStop,
     /// 被拒之前发成的那几版（[`VersionsPublishedInThisChange`]）。
     pub versions_published: VersionsPublishedInThisChange,
-    /// 这一次是不是被宽护卡住（[`WideProtectionAtTheCeiling`]）：F 已在上限、C 那一圈还隔离着槽时点名这一形、带上出路（推环那一条预演过放得下才列），
+    /// 这一次是不是被宽护卡住（[`WideProtectionAtTheCeiling`]）：F 已在上限、C 那一圈还隔离着槽时点名这一形、带上算出路要的东西
+    /// （出路在调用方要的时候算，[`MountedSession::ways_out_of_the_wide_protection`]：推环那一条预演过放得下才列），
     /// 别的形是 [`WideProtectionAtTheCeiling::NotWhatStopsTheAdmission`]，照旧只是空间不够。
     pub wide_protection_at_the_ceiling: WideProtectionAtTheCeiling,
 }
 
 /// 推满仍不够时，挡着准入的是不是「这个进程写掉那一槽才放」的那一份隔离（用户 2026-10-08 弹窗定「显式推环命令加带出路的错」：
 /// 不自动推环，准入因宽护卡住时报一个点名「被宽护卡住」的错、带上出路；用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」：
-/// 推环那一条出路预演过、推完这次改动放得下才列）。
+/// 推环那一条出路预演过、推完这次改动放得下才列；同日弹窗定「改：预演延到要出路时才做」：报的时候不预演，调用方要出路时另调
+/// [`MountedSession::ways_out_of_the_wide_protection`] 算，只要错误码、不看出路的调用方不付这一笔）。
 ///
 /// 为什么不另开一个 [`UserChangeRefused`] 成员：checker 档的装置（`e158_root_choice_repair`、`e168_delete_ring_window_borrow`）对
 /// `UserChangeRefused` 与 `FloorRaiseStop` 写的是穷举 `match`，多一个成员它们就编不过，而 checker 档这一批不改（第 10z 批第十六块派发）。
@@ -202,8 +205,12 @@ pub enum WideProtectionAtTheCeiling {
 ///
 /// **宽护是挡着准入的之一，不是唯一**：同一刻没有 C 那一圈，准入照样可以被拒（第 10z 批代码轮第一轮攻方
 /// `research/prompts/m3-batch10z-pqrs-code-r1-opus-output.md` 第一节：同一个池停在 R 之前、C 一槽都没有的对照臂，同一个长度也被拒的 27 格，
-/// 这里照样报这一形）。所以报这一形不说明写掉 R 那一槽之后放得下：推完放不放得下由 `pushing_the_root_ring` 那一份预演说，
-/// 放不下（推完上限退到环里最旧的根、推环自己换下的那几份要等环再转一圈才回收，同一节「读法」）就不列推环。
+/// 这里照样报这一形）。所以报这一形不说明写掉 R 那一槽之后放得下：推完放不放得下由调用方要出路时那一份预演说
+/// （[`MountedSession::ways_out_of_the_wide_protection`]），放不下（推完上限退到环里最旧的根、推环自己换下的那几份要等环再转一圈才回收，
+/// 同一节「读法」）就不列推环。
+///
+/// 报这一形只读分配器（内存）：不拷分配器、不预演、一个盘都不读。预演要的「被拒在哪一版上」「这次改动还剩什么」记在这里（私有，
+/// 只有会话造得出这一份），这次改动的内容拷一份进来（错误不借调用方的内容）。
 #[derive(Debug)]
 pub struct StuckBehindTheWideProtection {
     /// 推环要写到的那一槽：C 那一圈还没解开、记着落点的槽里，从现行那一版起按 txg 往后最后一个轮到的（R 那一槽）。
@@ -216,12 +223,15 @@ pub struct StuckBehindTheWideProtection {
     /// C 那一圈每块盘上这一刻隔离着的槽数（按分配器里盘的次序，同一个槽只数一次）：宽护挡着的槽数。只数 C 那一圈罩着的，
     /// 被抛弃根罩着、C 不罩的不在其内。
     pub isolated_slots_on_each_device: Vec<(DeviceIdentity, u64)>,
-    /// 在分配器的拷贝与只进内存的盘上预演推环、推完再预演这次改动（连同它会推的抬 F）的结局（[`RootRingPushRehearsal`]）：
-    /// 推环这条出路列不列按它定。
-    pub pushing_the_root_ring: RootRingPushRehearsal,
+    /// 被拒的那一次要接在后面的那一版（报这一形那一刻会话的现行那一版）的实例代号与 txg：要出路时会话不在这一版上就不预演
+    /// （[`WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion`]）。
+    instance_of_the_version_refused_on: InstanceGeneration,
+    checkpoint_txg_of_the_version_refused_on: CheckpointTxg,
+    /// 被拒的那一次之后这次用户改动还剩什么要发（内容拷一份）：要出路时推完环照它再预演一次。
+    remainder_of_the_change: RemainderOfTheUserChangeKeptByTheRefusal,
 }
 
-/// 「被宽护卡住」那一刻预演推环与推完之后这次改动的准入（用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」）：
+/// 调用方要出路时预演推环与推完之后这次改动的准入（用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」「改：预演延到要出路时才做」）：
 /// 在分配器的一份拷贝上、对只进内存的盘走一遍推环入口与推完之后这次改动剩下的发布（`rehearse_the_push_then_the_remainder_of_the_change`），
 /// 一个写都没发到盘上，会话不动。
 #[derive(Debug)]
@@ -241,11 +251,33 @@ pub enum RootRingPushRehearsal {
     PushRefused(Box<RootRingPushRefused>),
 }
 
-impl StuckBehindTheWideProtection {
+/// 调用方要出路时算出来的（[`MountedSession::ways_out_of_the_wide_protection`]）：那一刻的预演结局与照它列的出路。
+#[derive(Debug)]
+pub struct WaysOutOfTheWideProtection {
+    /// 在分配器的拷贝与只进内存的盘上预演推环、推完再预演这次改动（连同它会推的抬 F）的结局：推环这条出路列不列按它定。
+    pub pushing_the_root_ring: RootRingPushRehearsal,
     /// 出路，调用方照抄就能报给用户：卸载重挂一定列；推环（会丢掉最近可退到的状态，调用即确认）只在预演过推完这次改动放得下时列
     /// （[`RootRingPushRehearsal::ChangeAdmittedAfterThePush`]）。按这个次序。
-    #[must_use]
-    pub fn ways_out(&self) -> Vec<String> {
+    pub ways_out: Vec<String>,
+}
+
+/// 要出路时没预演。
+#[derive(Debug)]
+pub enum WaysOutOfTheWideProtectionRefused {
+    /// 会话的现行那一版已不是报「被宽护卡住」时被拒的那一次要接在后面的那一版（之后又发过：推过环、别的改动发成、走空走了几步；或这一份报
+    /// 不是这个会话的）：报里的 R 那一槽、要推的次数与挡着的槽数说的是那一刻，照这一刻预演会与它们对不上，按哪一刻算条款没写，第一版不支持。
+    /// 一个盘都没读、一个写都没发，会话不动。调用方要做的是再发一次这次改动、拿新的那一份报再要出路。
+    SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion {
+        instance_refused_on: InstanceGeneration,
+        checkpoint_txg_refused_on: CheckpointTxg,
+        instance_of_the_current_version: InstanceGeneration,
+        checkpoint_txg_of_the_current_version: CheckpointTxg,
+    },
+}
+
+impl StuckBehindTheWideProtection {
+    /// 照这一份报与要出路时那一份预演列出路：卸载重挂一定列；推环只在预演过推完这次改动放得下时列。按这个次序。
+    fn ways_out_after(&self, pushing_the_root_ring: &RootRingPushRehearsal) -> Vec<String> {
         let way_one = match self.checkpoint_txg_of_r {
             Some(txg_of_r) => format!(
                 "出路一：正常卸载（mount::unmount）之后再可写挂载一次。再挂时 R（txg {}）按新实例的实例表判成被抛弃，影子账只护它自己账里引用的落点，宽护不再出现。R 的账读不出若是盘上永久坏，卸载之后再挂会被拒（C694 的 B），而会话在卸载时已断开。",
@@ -257,7 +289,7 @@ impl StuckBehindTheWideProtection {
             ),
         };
         let mut ways_out = vec![way_one];
-        match &self.pushing_the_root_ring {
+        match pushing_the_root_ring {
             RootRingPushRehearsal::ChangeAdmittedAfterThePush {
                 isolated_slots_released_by_the_push_on_each_device,
             } => {
@@ -287,9 +319,9 @@ impl StuckBehindTheWideProtection {
 }
 
 /// 预演用的盘：读穿到底下那块盘（这一刻盘上的字节），写与写零只记进内存、盖在之后的读上，屏障什么都不做——一个写都不发到底下那块盘。
-/// 「被宽护卡住」那一报预演推环与推完之后这次改动的准入用（[`rehearse_the_push_then_the_remainder_of_the_change`]）。
-/// 底下那块盘借成 `&dyn BlockDevice`：预演里走的会话发布、推环、抬 F 那几层按盘的类型实例化，预演里再判「被宽护卡住」时
-/// 拿这一层再包一层，借成 trait 对象之后类型不再往里长（泛型借底下的类型就是无穷层实例化）。
+/// 要出路时预演推环与推完之后这次改动的准入用（[`rehearse_the_push_then_the_remainder_of_the_change`]）。
+/// 底下那块盘只读借成 `&dyn BlockDevice`：改盘的三个动作要 `&mut`，类型上调不到。请求先过两个后端那一份判据
+/// （`block_device::check_aligned_and_in_range`），真盘会拒的这里照样拒。
 struct DeviceKeepingWritesInMemory<'disk> {
     disk: &'disk dyn BlockDevice,
     physical_block_size: PhysicalBlockSizeInBytes,
@@ -310,30 +342,18 @@ impl<'disk> DeviceKeepingWritesInMemory<'disk> {
         u64::from(self.physical_block_size.0)
     }
 
-    /// 与两个后端同一条约定（`block_device` 文件后端的 `check_aligned_and_in_range`）：按物理块对齐、不越过设备末尾。
-    /// 写不交给底下那块盘，这里不判就收得下一个真盘会拒的写。
-    fn check_aligned_and_in_range(
+    /// 两个后端那一份判据（`block_device::check_aligned_and_in_range`），设备大小取底下那块盘的。
+    fn check_request(
         &self,
         offset: DeviceOffsetInBytes,
         length: u64,
     ) -> Result<(), BlockDeviceError> {
-        let block_bytes = self.block_bytes();
-        if !offset.0.is_multiple_of(block_bytes) || !length.is_multiple_of(block_bytes) {
-            return Err(BlockDeviceError::Unaligned {
-                offset,
-                length,
-                physical_block_size: self.physical_block_size.0,
-            });
-        }
-        let device_size = self.disk.size_in_bytes();
-        match offset.0.checked_add(length) {
-            Some(end) if end <= device_size => Ok(()),
-            Some(_) | None => Err(BlockDeviceError::OutOfRange {
-                offset,
-                length,
-                device_size,
-            }),
-        }
+        check_aligned_and_in_range(
+            offset,
+            length,
+            self.physical_block_size,
+            self.disk.size_in_bytes(),
+        )
     }
 
     /// 块号 `first_unwritten_block` 起、到 `end_block` 之前那一段没写过的块从底下那块盘读进 `buffer` 里对应的那一截
@@ -376,7 +396,7 @@ impl BlockDevice for DeviceKeepingWritesInMemory<'_> {
         buffer: &mut [u8],
     ) -> Result<(), BlockDeviceError> {
         let length = u64::try_from(buffer.len()).expect("读长装得进 u64");
-        self.check_aligned_and_in_range(offset, length)?;
+        self.check_request(offset, length)?;
         let block_bytes = self.block_bytes();
         let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
         let first_block = offset.0 / block_bytes;
@@ -417,10 +437,7 @@ impl BlockDevice for DeviceKeepingWritesInMemory<'_> {
         bytes: &[u8],
         _durability: WriteDurability,
     ) -> Result<(), BlockDeviceError> {
-        self.check_aligned_and_in_range(
-            offset,
-            u64::try_from(bytes.len()).expect("写长装得进 u64"),
-        )?;
+        self.check_request(offset, u64::try_from(bytes.len()).expect("写长装得进 u64"))?;
         self.keep_in_memory(offset, bytes);
         Ok(())
     }
@@ -430,7 +447,7 @@ impl BlockDevice for DeviceKeepingWritesInMemory<'_> {
         offset: DeviceOffsetInBytes,
         length: u64,
     ) -> Result<(), BlockDeviceError> {
-        self.check_aligned_and_in_range(offset, length)?;
+        self.check_request(offset, length)?;
         let zeros = vec![0u8; usize::try_from(length).expect("写零的长度装得进 usize")];
         self.keep_in_memory(offset, &zeros);
         Ok(())
@@ -449,7 +466,7 @@ impl BlockDevice for DeviceKeepingWritesInMemory<'_> {
     }
 }
 
-/// 被拒的那一次发布之后，这次用户改动还剩什么要发（「被宽护卡住」那一报推完环之后照它再预演一次，
+/// 被拒的那一次发布之后，这次用户改动还剩什么要发（「被宽护卡住」那一报记一份拷贝，要出路时推完环之后照它再预演一次，
 /// [`rehearse_the_push_then_the_remainder_of_the_change`]）。
 #[derive(Clone, Copy, Debug)]
 enum RemainderOfTheUserChange<'content> {
@@ -459,6 +476,122 @@ enum RemainderOfTheUserChange<'content> {
     WalkTheQueue,
 }
 
+/// [`RemainderOfTheUserChange`] 记进「被宽护卡住」那一份报里的拷贝：错误活得比调用方借给发布的内容长，内容拷一份。
+#[derive(Debug)]
+enum RemainderOfTheUserChangeKeptByTheRefusal {
+    WalkTheQueueThenPublishTheChange(UserChangeKeptByTheRefusal),
+    WalkTheQueue,
+}
+
+/// [`UserChange`] 的一份拷贝，内容自有。
+#[derive(Debug)]
+enum UserChangeKeptByTheRefusal {
+    Overwrite {
+        content: Vec<u8>,
+        write_time_seconds: u64,
+    },
+    SequentialWrite {
+        content: Vec<u8>,
+        write_time_seconds: u64,
+    },
+    NewInodes {
+        count: u64,
+        write_time_seconds: u64,
+    },
+    Unlink {
+        inode: InodeNumber,
+        write_time_seconds: u64,
+    },
+}
+
+impl UserChangeKeptByTheRefusal {
+    fn kept_from(change: UserChange<'_>) -> Self {
+        match change {
+            UserChange::Overwrite(file) => UserChangeKeptByTheRefusal::Overwrite {
+                content: file.content.to_vec(),
+                write_time_seconds: file.write_time_seconds,
+            },
+            UserChange::SequentialWrite(file) => UserChangeKeptByTheRefusal::SequentialWrite {
+                content: file.content.to_vec(),
+                write_time_seconds: file.write_time_seconds,
+            },
+            UserChange::NewInodes {
+                count,
+                write_time_seconds,
+            } => UserChangeKeptByTheRefusal::NewInodes {
+                count,
+                write_time_seconds,
+            },
+            UserChange::Unlink {
+                inode,
+                write_time_seconds,
+            } => UserChangeKeptByTheRefusal::Unlink {
+                inode,
+                write_time_seconds,
+            },
+        }
+    }
+
+    fn as_user_change(&self) -> UserChange<'_> {
+        match self {
+            UserChangeKeptByTheRefusal::Overwrite {
+                content,
+                write_time_seconds,
+            } => UserChange::Overwrite(FirstFile {
+                content,
+                write_time_seconds: *write_time_seconds,
+            }),
+            UserChangeKeptByTheRefusal::SequentialWrite {
+                content,
+                write_time_seconds,
+            } => UserChange::SequentialWrite(FirstFile {
+                content,
+                write_time_seconds: *write_time_seconds,
+            }),
+            UserChangeKeptByTheRefusal::NewInodes {
+                count,
+                write_time_seconds,
+            } => UserChange::NewInodes {
+                count: *count,
+                write_time_seconds: *write_time_seconds,
+            },
+            UserChangeKeptByTheRefusal::Unlink {
+                inode,
+                write_time_seconds,
+            } => UserChange::Unlink {
+                inode: *inode,
+                write_time_seconds: *write_time_seconds,
+            },
+        }
+    }
+}
+
+impl RemainderOfTheUserChangeKeptByTheRefusal {
+    fn kept_from(remainder: RemainderOfTheUserChange<'_>) -> Self {
+        match remainder {
+            RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change) => {
+                RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueueThenPublishTheChange(
+                    UserChangeKeptByTheRefusal::kept_from(change),
+                )
+            }
+            RemainderOfTheUserChange::WalkTheQueue => {
+                RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueue
+            }
+        }
+    }
+
+    fn as_remainder(&self) -> RemainderOfTheUserChange<'_> {
+        match self {
+            RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueueThenPublishTheChange(change) => {
+                RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change.as_user_change())
+            }
+            RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueue => {
+                RemainderOfTheUserChange::WalkTheQueue
+            }
+        }
+    }
+}
+
 /// 每块盘上这一刻隔离的槽数（D28（挂载期承诺量） 已定项 1 第九项那一项），按分配器里盘的次序。只读内存。
 fn isolated_slots_of_each_device(allocator: &PoolAllocator) -> Vec<(DeviceIdentity, u64)> {
     allocator
@@ -468,13 +601,14 @@ fn isolated_slots_of_each_device(allocator: &PoolAllocator) -> Vec<(DeviceIdenti
         .collect()
 }
 
-/// 「被宽护卡住」那一报之前的预演（用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」）：在分配器的一份拷贝上、对只进内存的盘
+/// 要出路时的预演（用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」「改：预演延到要出路时才做」，
+/// [`MountedSession::ways_out_of_the_wide_protection`]）：在分配器的一份拷贝上、对只进内存的盘
 /// （[`DeviceKeepingWritesInMemory`]）先走一遍推环入口（与真调推环入口同一段，[`MountedSession::push_the_root_ring_on`]：它自己的整串预演、
 /// 真发与末尾的断言都在），推完再把这次改动剩下的照会话发布那一段走一遍（`remainder`：准入被拒照样推抬 F，抬 F 的上限按推完的根环算——
 /// 读的是盖着推环那几次写的盘）。读盘照读底下那几块盘；写一个都不发到底下那几块盘；会话的分配器与现行那一版不动。
 ///
-/// 递归至多一层：预演里推环做成之后 C 那一圈一槽都不记着（推环入口末尾那条断言），预演里的准入再被拒时 [`root_ring_push_target`]
-/// 交回 `None`、报 `NotWhatStopsTheAdmission`，不再进预演；推环在预演里被拒就不往下走。
+/// 预演里的准入再被拒时报不报「被宽护卡住」只读内存（[`wide_protection_at_the_ceiling`] 不预演）；推环做成之后 C 那一圈一槽都不记着
+/// （推环入口末尾那条断言），报的是 `NotWhatStopsTheAdmission`。推环在预演里被拒就不往下走。
 fn rehearse_the_push_then_the_remainder_of_the_change<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &[(DeviceIdentity, Device)],
@@ -613,19 +747,12 @@ fn root_ring_push_target(
 }
 
 /// 推满仍不够时报不报「被宽护卡住」（[`WideProtectionAtTheCeiling`]）：`stop` 是 F 已在上限、C 那一圈还有记着落点的槽、它们这一刻至少隔离着一个槽，
-/// 才报；别的都照旧。判这三样只读分配器（内存）。报的时候再预演推环与推完之后这次改动剩下的（`remainder`，
-/// [`rehearse_the_push_then_the_remainder_of_the_change`]）：读盘、一个写都不发、会话不动，出路列不列推环按它定。
-#[allow(
-    clippy::too_many_arguments,
-    reason = "参数各是一样东西：盘上择到的参数、盘表、分配器、被拒的那一次要接在后面的那一版、实例代号、影子账那一臂、这次改动还剩什么、不再推的原因"
-)]
-fn wide_protection_at_the_ceiling<Device: BlockDevice>(
-    parameters: &MakeFilesystemParameters,
-    devices: &[(DeviceIdentity, Device)],
+/// 才报；别的都照旧。判这三样与报的那几个数只读分配器（内存）。报的时候不预演（用户 2026-10-09 弹窗定「改：预演延到要出路时才做」）：
+/// 只记下被拒在哪一版上（`current`）与这次改动还剩什么（`remainder`，内容拷一份），调用方要出路时
+/// [`MountedSession::ways_out_of_the_wide_protection`] 照它们预演。
+fn wide_protection_at_the_ceiling(
     allocator: &PoolAllocator,
     current: &TransactionOutput,
-    instance: InstanceGeneration,
-    shadow_ledger: ShadowLedger,
     remainder: RemainderOfTheUserChange<'_>,
     stop: &FloorRaiseStop,
 ) -> WideProtectionAtTheCeiling {
@@ -649,15 +776,6 @@ fn wide_protection_at_the_ceiling<Device: BlockDevice>(
     let checkpoint_txg_of_r = allocator.root_ring_occupancy().and_then(|occupancy| {
         occupancy.checkpoint_txg_of_the_valid_root_recorded_on(target.ring_slot_of_r)
     });
-    let pushing_the_root_ring = rehearse_the_push_then_the_remainder_of_the_change(
-        parameters,
-        devices,
-        allocator,
-        current,
-        instance,
-        shadow_ledger,
-        remainder,
-    );
     WideProtectionAtTheCeiling::StuckBehindTheWideProtection(Box::new(
         StuckBehindTheWideProtection {
             ring_slot_of_r: target.ring_slot_of_r,
@@ -665,7 +783,9 @@ fn wide_protection_at_the_ceiling<Device: BlockDevice>(
             empty_publishes_until_the_ring_slot_of_r_is_written: target
                 .publishes_until_the_ring_slot_of_r_is_written,
             isolated_slots_on_each_device,
-            pushing_the_root_ring,
+            instance_of_the_version_refused_on: current.root.instance,
+            checkpoint_txg_of_the_version_refused_on: current.root.checkpoint_txg,
+            remainder_of_the_change: RemainderOfTheUserChangeKeptByTheRefusal::kept_from(remainder),
         },
     ))
 }
@@ -956,7 +1076,7 @@ impl MountedSession {
     }
 
     /// [`Self::publish_user_change`] 核过盘表之后那一半：先走空、再发这次改动（unlink 连同它之后的走空），被拒是空间不够就推抬 F 再判。
-    /// 「被宽护卡住」那一报的预演在只进内存的盘与分配器的拷贝上走同一段（[`rehearse_the_push_then_the_remainder_of_the_change`]），
+    /// 要出路时的预演在只进内存的盘与分配器的拷贝上走同一段（[`rehearse_the_push_then_the_remainder_of_the_change`]），
     /// 所以拆成不收 `self` 的一段。
     fn publish_the_change_raising_the_floor_for_space<Device: BlockDevice>(
         parameters: &MakeFilesystemParameters,
@@ -1062,7 +1182,6 @@ impl MountedSession {
                     devices,
                     allocator,
                     version_to_change,
-                    instance,
                     shadow_ledger,
                     RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),
                     publishes_pushed,
@@ -1082,7 +1201,7 @@ impl MountedSession {
     /// 不超过一环），那几槽里原来的根（回退候选）换成同一个用户状态的空发布根；推完之后回退候选集可以掉到少于 4 个不同状态，
     /// D16（发布语义） 已定项 1「回退候选集保留最近 4 个可退到的不同状态」只在调这个入口时破
     /// （第 10z 批调查报告 10z-p 第六节第 1 条：384 槽盘那一形推完只剩 2 个）。不自动调：准入被宽护卡住时只报 [`StuckBehindTheWideProtection`]，
-    /// 预演过推完之后那次改动放得下才写明这一条出路。
+    /// 调用方要出路时（[`Self::ways_out_of_the_wide_protection`]）预演过推完之后那次改动放得下才写明这一条出路。
     ///
     /// 第一道看分配器上冻结着没重发的发布，冻结着就拒（`PublishFrozenAfterAWriteFailureIsNotResentYet`），在核盘表与预演之前。
     /// C 那一圈记着落点、而它们罩着的槽这一刻一个都没隔离时不推（写掉那几槽放不出东西，`NothingHeldUntilThisProcessWritesARingSlot`），在任何写之前。
@@ -1115,8 +1234,67 @@ impl MountedSession {
         )
     }
 
+    /// 被宽护卡住之后调用方要出路时调（用户 2026-10-09 弹窗定「改：预演延到要出路时才做」）：照 `stuck` 那一份报记着的「这次改动还剩什么」，
+    /// 在分配器的一份拷贝上、对只进内存的盘预演推环与推完之后这次改动剩下的发布（[`rehearse_the_push_then_the_remainder_of_the_change`]：
+    /// 与真调推环入口、再发同一个改动走同一段），照结局列出路（推完放得下才列推环）。报 `stuck` 的那一次不付这一笔。
+    ///
+    /// 收 `&self`：预演只读交进来的盘、在分配器的拷贝上走，会话的分配器、现行那一版一样都不动——类型上就动不了（`PoolAllocator` 里没有内部可变），
+    /// 调用方不必为了要出路把会话交出去。盘表只读借：只进内存的盘只读借底下那块盘，写一个都发不到它上面。
+    /// 交进来的盘表照推环入口那一套核（在预演的推环里，核不过就是推环被拒、出路只列卸载再挂），与照出路二真推时同一个结局。
+    ///
+    /// 第一道看会话还在不在被拒的那一版上（实例代号与 txg 都与报里记的相同）：不在就不预演，一个盘都不读。
+    ///
+    /// # Errors
+    /// [`WaysOutOfTheWideProtectionRefused`] 的一种。
+    pub fn ways_out_of_the_wide_protection<Device: BlockDevice>(
+        &self,
+        devices: &[(DeviceIdentity, Device)],
+        stuck: &StuckBehindTheWideProtection,
+    ) -> Result<WaysOutOfTheWideProtection, WaysOutOfTheWideProtectionRefused> {
+        let is_the_version_refused_on = |version_of_trees: &TransactionOutput| {
+            version_of_trees.root.instance == stuck.instance_of_the_version_refused_on
+                && version_of_trees.root.checkpoint_txg
+                    == stuck.checkpoint_txg_of_the_version_refused_on
+        };
+        let version_refused_on = match &self.current {
+            PoolVersion::WithFile(version_of_trees)
+            | PoolVersion::WithTreesAndNoFile(version_of_trees)
+                if is_the_version_refused_on(version_of_trees) =>
+            {
+                version_of_trees
+            }
+            PoolVersion::WithFile(_)
+            | PoolVersion::WithTreesAndNoFile(_)
+            | PoolVersion::WithoutFile(_) => {
+                let current_root = self.current.root();
+                return Err(
+                    WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion {
+                        instance_refused_on: stuck.instance_of_the_version_refused_on,
+                        checkpoint_txg_refused_on: stuck.checkpoint_txg_of_the_version_refused_on,
+                        instance_of_the_current_version: current_root.instance,
+                        checkpoint_txg_of_the_current_version: current_root.checkpoint_txg,
+                    },
+                );
+            }
+        };
+        let pushing_the_root_ring = rehearse_the_push_then_the_remainder_of_the_change(
+            self.parameters_and_device_table.parameters_on_disk(),
+            devices,
+            &self.allocator,
+            version_refused_on,
+            self.instance,
+            self.shadow_ledger,
+            stuck.remainder_of_the_change.as_remainder(),
+        );
+        let ways_out = stuck.ways_out_after(&pushing_the_root_ring);
+        Ok(WaysOutOfTheWideProtection {
+            pushing_the_root_ring,
+            ways_out,
+        })
+    }
+
     /// [`Self::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states`] 的本体，盘上择到的参数由调用方给：
-    /// 「被宽护卡住」那一报的预演在只进内存的盘与分配器的拷贝上走同一段（[`rehearse_the_push_then_the_remainder_of_the_change`]）。
+    /// 要出路时的预演在只进内存的盘与分配器的拷贝上走同一段（[`rehearse_the_push_then_the_remainder_of_the_change`]）。
     ///
     /// # Errors
     /// 同那个入口。
@@ -1265,19 +1443,18 @@ struct PublishedInThisChange {
 /// 一次发布被拒之后会话怎么接（D16（发布语义） 已定项 1「准入」那一行）：不是空间不够的那两种（[`refusal_is_short_of_space`]）原样交回 `Publish`；
 /// 是就为被拒的那一次发布推一串抬 F 的空发布（这次准入已推 `publishes_pushed_in_this_admission` 次，预算照
 /// `mount::push_one_floor_raise_within_the_admission_budget`），推成交回记上这一串之后的 `published_in_this_change` 与这一串推了几次空发布，
-/// 不再推交回 `NoSpaceAfterRaisingTheFloor`（报不报「被宽护卡住」、预演推环之后这次改动剩下的 `remainder` 放不放得下，
+/// 不再推交回 `NoSpaceAfterRaisingTheFloor`（报不报「被宽护卡住」，报的话记下这次改动剩下的 `remainder`，要出路时照它预演，
 /// [`wide_protection_at_the_ceiling`]），抬 F 自己报错交回 `FloorRaiseFailedWhilePushingForSpace`。
 /// 推的那一串就地推进 `current`（被拒的那一次要接在它后面的那一版）。
 #[allow(
     clippy::too_many_arguments,
-    reason = "参数各有名字与类型：写入口要的参数与盘表、分配器、现行那一版、实例代号、影子账那一臂、这次改动还剩什么、预算已用的数、已落盘的、这一次被拒，不成一个概念"
+    reason = "参数各有名字与类型：写入口要的参数与盘表、分配器、现行那一版、影子账那一臂、这次改动还剩什么、预算已用的数、已落盘的、这一次被拒，不成一个概念"
 )]
 fn push_one_floor_raise_for_a_refusal_for_space<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
     allocator: &mut PoolAllocator,
     current: &mut TransactionOutput,
-    instance: InstanceGeneration,
     shadow_ledger: ShadowLedger,
     remainder: RemainderOfTheUserChange<'_>,
     publishes_pushed_in_this_admission: usize,
@@ -1324,14 +1501,7 @@ fn push_one_floor_raise_for_a_refusal_for_space<Device: BlockDevice>(
                 refusals_that_pushed_the_floor_raises,
                 // 先借 `stop` 判这一形，再把它挪进下一个字段（字段按写的次序求值）。
                 wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(
-                    parameters,
-                    devices.as_slice(),
-                    allocator,
-                    current,
-                    instance,
-                    shadow_ledger,
-                    remainder,
-                    &stop,
+                    allocator, current, remainder, &stop,
                 ),
                 stop,
                 versions_published,
@@ -1355,7 +1525,7 @@ fn push_one_floor_raise_for_a_refusal_for_space<Device: BlockDevice>(
 /// 每一次被拒的删除类发布各算一次准入（与 unlink 意图那次、别的用户改动那次一样，预算数的是为被判的那一次发布推了几次空发布）：
 /// 这一趟走空有一步落了盘，下一次被拒的就是后面那一次发布，这次准入已推的数从 0 数起。`current` 停在最后一次落盘的那一版上
 /// （删除那一步或推的空发布）；发成的每一步记进 `versions_published`。`remainder` 是走空之后这次改动还剩什么（推满仍不够、被宽护卡住时
-/// 预演推环之后照它再发一次，[`wide_protection_at_the_ceiling`]）。
+/// 记进报里，要出路时预演推环之后照它再发一次，[`wide_protection_at_the_ceiling`]）。
 #[allow(
     clippy::too_many_arguments,
     reason = "参数各有名字与类型：写入口要的参数与盘表、分配器、现行那一版、实例代号、影子账那一臂、这次改动还剩什么、已落盘的，不成一个概念"
@@ -1402,7 +1572,6 @@ fn publish_the_queue_until_it_is_empty_raising_the_floor_for_space<Device: Block
                 devices,
                 allocator,
                 current,
-                instance,
                 shadow_ledger,
                 remainder,
                 publishes_pushed_in_this_admission,
@@ -1412,3 +1581,441 @@ fn publish_the_queue_until_it_is_empty_raising_the_floor_for_space<Device: Block
         publishes_pushed_in_this_admission += publishes_of_this_raise;
     }
 }
+
+#[cfg(test)]
+mod tests {
+    //! 只进内存的盘（[`DeviceKeepingWritesInMemory`]）：写与写零只进内存、盖在之后的读上，读没写过的块穿到底下那块盘，底下那块盘一字不动；
+    //! 请求判据与两个后端是同一份（`block_device::check_aligned_and_in_range`）。差分对拍照正推腿的做法
+    //! （`research/prompts/m3-batch10z-tu-code-r1-sonnet-model/src/core-mounted-session-debug.diff` 的 `sonnet_k5_differential`），
+    //! 参照换成一块把每个写直接写进去的字节盘（同一份请求判据）。
+    use super::*;
+
+    /// 物理块 512 字节、16 块的盘。
+    const PHYSICAL_BLOCK_BYTES: u32 = 512;
+    const BLOCKS_OF_THE_DISK: u64 = 16;
+
+    /// 底下那块盘（也是差分对拍的参照）：整块字节放内存，请求判据照两个后端那一份。
+    struct DeviceOfBytes {
+        bytes: Vec<u8>,
+        physical_block_size: PhysicalBlockSizeInBytes,
+    }
+
+    impl DeviceOfBytes {
+        /// 每个字节按位置与 `seed` 填一个图样：读回哪一块看得出来。
+        fn filled_with_a_pattern(seed: u8) -> Self {
+            let device_bytes =
+                usize::try_from(u64::from(PHYSICAL_BLOCK_BYTES) * BLOCKS_OF_THE_DISK)
+                    .expect("盘长装得进 usize");
+            DeviceOfBytes {
+                bytes: (0..device_bytes)
+                    .map(|position| {
+                        u8::try_from(position % 251)
+                            .expect("小于 256")
+                            .wrapping_add(seed)
+                    })
+                    .collect(),
+                physical_block_size: PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
+            }
+        }
+
+        fn range_of(
+            &self,
+            offset: DeviceOffsetInBytes,
+            length: u64,
+        ) -> Result<std::ops::Range<usize>, BlockDeviceError> {
+            check_aligned_and_in_range(
+                offset,
+                length,
+                self.physical_block_size,
+                self.size_in_bytes(),
+            )?;
+            let start = usize::try_from(offset.0).expect("判过不越过盘尾");
+            let end = start + usize::try_from(length).expect("判过不越过盘尾");
+            Ok(start..end)
+        }
+    }
+
+    impl BlockDevice for DeviceOfBytes {
+        fn read_at(
+            &self,
+            offset: DeviceOffsetInBytes,
+            buffer: &mut [u8],
+        ) -> Result<(), BlockDeviceError> {
+            let range = self.range_of(offset, u64::try_from(buffer.len()).expect("读长"))?;
+            buffer.copy_from_slice(&self.bytes[range]);
+            Ok(())
+        }
+
+        fn write_at(
+            &mut self,
+            offset: DeviceOffsetInBytes,
+            bytes: &[u8],
+            _durability: WriteDurability,
+        ) -> Result<(), BlockDeviceError> {
+            let range = self.range_of(offset, u64::try_from(bytes.len()).expect("写长"))?;
+            self.bytes[range].copy_from_slice(bytes);
+            Ok(())
+        }
+
+        fn write_zeroes_at(
+            &mut self,
+            offset: DeviceOffsetInBytes,
+            length: u64,
+        ) -> Result<(), BlockDeviceError> {
+            let range = self.range_of(offset, length)?;
+            self.bytes[range].fill(0);
+            Ok(())
+        }
+
+        fn barrier(&mut self) -> Result<(), BlockDeviceError> {
+            Ok(())
+        }
+
+        fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
+            self.physical_block_size
+        }
+
+        fn size_in_bytes(&self) -> u64 {
+            u64::try_from(self.bytes.len()).expect("盘长装得进 u64")
+        }
+    }
+
+    fn offset_of_block(block: u64) -> DeviceOffsetInBytes {
+        DeviceOffsetInBytes(block * u64::from(PHYSICAL_BLOCK_BYTES))
+    }
+
+    fn bytes_of_blocks(blocks: u64) -> usize {
+        usize::try_from(blocks * u64::from(PHYSICAL_BLOCK_BYTES)).expect("块数装得进 usize")
+    }
+
+    /// 从 `first_block` 起读 `blocks` 块；读缓冲先填 0xAA，没读到的那一截露得出来。
+    fn read_blocks(device: &dyn BlockDevice, first_block: u64, blocks: u64) -> Vec<u8> {
+        let mut buffer = vec![0xAA_u8; bytes_of_blocks(blocks)];
+        device
+            .read_at(offset_of_block(first_block), &mut buffer)
+            .expect("对齐、没越过盘尾的读做得成");
+        buffer
+    }
+
+    fn bytes_of_the_disk_blocks(disk: &DeviceOfBytes, first_block: u64, blocks: u64) -> Vec<u8> {
+        let start = bytes_of_blocks(first_block);
+        disk.bytes[start..start + bytes_of_blocks(blocks)].to_vec()
+    }
+
+    #[test]
+    fn a_write_kept_in_memory_reads_back_and_leaves_the_disk_below_unchanged() {
+        let disk = DeviceOfBytes::filled_with_a_pattern(0);
+        let bytes_before = disk.bytes.clone();
+        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
+        let written = vec![0x5A_u8; bytes_of_blocks(1)];
+        kept.write_at(
+            offset_of_block(3),
+            &written,
+            WriteDurability::ForceUnitAccess,
+        )
+        .expect("对齐、没越过盘尾的写做得成");
+        assert_eq!(
+            read_blocks(&kept, 3, 1),
+            written,
+            "写过的块读回写进去的字节"
+        );
+        assert!(
+            disk.bytes == bytes_before,
+            "写只进内存：底下那块盘一个字节都没变"
+        );
+    }
+
+    #[test]
+    fn a_read_of_blocks_never_written_passes_through_to_the_disk_below() {
+        let disk = DeviceOfBytes::filled_with_a_pattern(7);
+        let kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
+        assert_eq!(
+            read_blocks(&kept, 5, 3),
+            bytes_of_the_disk_blocks(&disk, 5, 3),
+            "没写过的块读穿到底下那块盘"
+        );
+    }
+
+    #[test]
+    fn a_read_across_written_and_unwritten_blocks_stitches_the_kept_blocks_with_the_disk_below() {
+        let disk = DeviceOfBytes::filled_with_a_pattern(11);
+        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
+        let block_two = vec![0x22_u8; bytes_of_blocks(1)];
+        let block_four = vec![0x44_u8; bytes_of_blocks(1)];
+        let blocks_seven_and_eight: Vec<u8> = (0..bytes_of_blocks(2))
+            .map(|position| u8::try_from(position % 199).expect("小于 256"))
+            .collect();
+        for (first_block, bytes) in [
+            (2, &block_two),
+            (4, &block_four),
+            (7, &blocks_seven_and_eight),
+        ] {
+            kept.write_at(offset_of_block(first_block), bytes, WriteDurability::Plain)
+                .expect("对齐、没越过盘尾的写做得成");
+        }
+        let mut expected = bytes_of_the_disk_blocks(&disk, 1, 9);
+        for (first_block, bytes) in [
+            (2, &block_two),
+            (4, &block_four),
+            (7, &blocks_seven_and_eight),
+        ] {
+            let start = bytes_of_blocks(first_block - 1);
+            expected[start..start + bytes.len()].copy_from_slice(bytes);
+        }
+        assert!(
+            read_blocks(&kept, 1, 9) == expected,
+            "块 1 到 9 一次读：写过的块（2、4、7、8）读回写进去的字节，没写过的（1、3、5、6、9，含末尾那一段）读穿到底下那块盘"
+        );
+    }
+
+    #[test]
+    fn zeroes_written_in_memory_read_back_as_zero_over_the_disk_below() {
+        let disk = DeviceOfBytes::filled_with_a_pattern(13);
+        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
+        kept.write_zeroes_at(offset_of_block(3), u64::from(PHYSICAL_BLOCK_BYTES) * 3)
+            .expect("对齐、没越过盘尾的写零做得成");
+        let mut expected = bytes_of_the_disk_blocks(&disk, 2, 5);
+        expected[bytes_of_blocks(1)..bytes_of_blocks(4)].fill(0);
+        assert!(
+            read_blocks(&kept, 2, 5) == expected,
+            "写零的块 3 到 5 读回全 0，两头的块 2、6 读穿到底下那块盘"
+        );
+        assert!(
+            bytes_of_the_disk_blocks(&disk, 3, 3)
+                .iter()
+                .any(|byte| *byte != 0),
+            "底下那块盘那一段不是全 0（阳性对照：读回的 0 是写零记下的）"
+        );
+    }
+
+    #[test]
+    fn a_request_off_the_physical_block_boundaries_is_refused_before_anything_is_kept() {
+        let disk = DeviceOfBytes::filled_with_a_pattern(17);
+        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
+        let unaligned_offset = DeviceOffsetInBytes(u64::from(PHYSICAL_BLOCK_BYTES) + 100);
+        let one_block = vec![0x33_u8; bytes_of_blocks(1)];
+        let short_of_a_block = vec![0x33_u8; 100];
+        let refusals = [
+            (
+                "偏移不在物理块边界上的写",
+                kept.write_at(unaligned_offset, &one_block, WriteDurability::Plain),
+                unaligned_offset,
+                u64::from(PHYSICAL_BLOCK_BYTES),
+            ),
+            (
+                "长度不是物理块整数倍的写",
+                kept.write_at(
+                    offset_of_block(2),
+                    &short_of_a_block,
+                    WriteDurability::Plain,
+                ),
+                offset_of_block(2),
+                100,
+            ),
+            (
+                "长度不是物理块整数倍的写零",
+                kept.write_zeroes_at(offset_of_block(4), 100),
+                offset_of_block(4),
+                100,
+            ),
+            (
+                "偏移不在物理块边界上的读",
+                kept.read_at(unaligned_offset, &mut vec![0_u8; bytes_of_blocks(1)]),
+                unaligned_offset,
+                u64::from(PHYSICAL_BLOCK_BYTES),
+            ),
+        ];
+        for (request, outcome, offset, length) in refusals {
+            assert!(
+                matches!(
+                    outcome,
+                    Err(BlockDeviceError::Unaligned {
+                        offset: refused_offset,
+                        length: refused_length,
+                        physical_block_size: PHYSICAL_BLOCK_BYTES,
+                    }) if refused_offset == offset && refused_length == length
+                ),
+                "{request}：拒成 Unaligned，带偏移 {} 与长度 {length}，而交回 {outcome:?}",
+                offset.0
+            );
+        }
+        assert!(
+            read_blocks(&kept, 0, BLOCKS_OF_THE_DISK) == disk.bytes,
+            "被拒的写一个块都没记：整盘读回底下那块盘的字节"
+        );
+    }
+
+    #[test]
+    fn a_request_past_the_end_of_the_disk_below_is_refused_and_one_ending_at_the_end_is_kept() {
+        let disk = DeviceOfBytes::filled_with_a_pattern(19);
+        let device_size = disk.size_in_bytes();
+        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
+        let last_block = vec![0x77_u8; bytes_of_blocks(1)];
+        kept.write_at(
+            offset_of_block(BLOCKS_OF_THE_DISK - 1),
+            &last_block,
+            WriteDurability::Plain,
+        )
+        .expect("最后一块正好写到盘尾：放行");
+        assert_eq!(
+            read_blocks(&kept, BLOCKS_OF_THE_DISK - 1, 1),
+            last_block,
+            "最后一块读回写进去的字节"
+        );
+        let two_blocks = vec![0x66_u8; bytes_of_blocks(2)];
+        let offset_near_the_top =
+            DeviceOffsetInBytes(u64::MAX - u64::from(PHYSICAL_BLOCK_BYTES) + 1);
+        let refusals = [
+            (
+                "从最后一块起写两块（越过盘尾一块）",
+                kept.write_at(
+                    offset_of_block(BLOCKS_OF_THE_DISK - 1),
+                    &two_blocks,
+                    WriteDurability::Plain,
+                ),
+                offset_of_block(BLOCKS_OF_THE_DISK - 1),
+                u64::from(PHYSICAL_BLOCK_BYTES) * 2,
+            ),
+            (
+                "从盘尾起写零一块",
+                kept.write_zeroes_at(
+                    offset_of_block(BLOCKS_OF_THE_DISK),
+                    u64::from(PHYSICAL_BLOCK_BYTES),
+                ),
+                offset_of_block(BLOCKS_OF_THE_DISK),
+                u64::from(PHYSICAL_BLOCK_BYTES),
+            ),
+            (
+                "偏移加长度越过 u64 的读",
+                kept.read_at(offset_near_the_top, &mut vec![0_u8; bytes_of_blocks(2)]),
+                offset_near_the_top,
+                u64::from(PHYSICAL_BLOCK_BYTES) * 2,
+            ),
+        ];
+        for (request, outcome, offset, length) in refusals {
+            assert!(
+                matches!(
+                    outcome,
+                    Err(BlockDeviceError::OutOfRange {
+                        offset: refused_offset,
+                        length: refused_length,
+                        device_size: refused_device_size,
+                    }) if refused_offset == offset && refused_length == length && refused_device_size == device_size
+                ),
+                "{request}：拒成 OutOfRange，带偏移 {}、长度 {length} 与盘长 {device_size}，而交回 {outcome:?}",
+                offset.0
+            );
+        }
+        let mut expected = disk.bytes.clone();
+        expected[bytes_of_blocks(BLOCKS_OF_THE_DISK - 1)..].copy_from_slice(&last_block);
+        assert!(
+            read_blocks(&kept, 0, BLOCKS_OF_THE_DISK) == expected,
+            "被拒的写一个块都没记：整盘读回只有最后一块是写进去的"
+        );
+    }
+
+    /// 差分对拍那一串请求的个数。
+    const REQUESTS_OF_THE_RANDOM_STREAM: u64 = 20_000;
+
+    /// 线性同余，取高 33 位（只要确定、够散）。
+    fn next_random(state: &mut u64) -> u64 {
+        *state = state
+            .wrapping_mul(6_364_136_223_846_793_005)
+            .wrapping_add(1_442_695_040_888_963_407);
+        *state >> 33
+    }
+
+    /// 偏移：多半落在物理块边界上、盘以内，少数不对齐、正好在盘尾、越过盘尾、贴着 u64 的顶。
+    fn random_offset(state: &mut u64, device_size: u64) -> u64 {
+        let block = u64::from(PHYSICAL_BLOCK_BYTES);
+        match next_random(state) % 8 {
+            0 => next_random(state) % (device_size + 2 * block),
+            1 => u64::MAX - next_random(state) % 1024,
+            2 => device_size,
+            _ => (next_random(state) % (device_size / block)) * block,
+        }
+    }
+
+    /// 长度：多半是一到几个物理块，少数是 0、不对齐、跨过盘尾。
+    fn random_length(state: &mut u64, device_size: u64) -> u64 {
+        let block = u64::from(PHYSICAL_BLOCK_BYTES);
+        match next_random(state) % 8 {
+            0 => 0,
+            1 => next_random(state) % (2 * device_size),
+            _ => (1 + next_random(state) % 4) * block,
+        }
+    }
+
+    #[test]
+    fn the_device_keeping_writes_in_memory_agrees_with_a_device_written_through_on_a_random_request_stream(
+    ) {
+        let disk = DeviceOfBytes::filled_with_a_pattern(23);
+        let bytes_before = disk.bytes.clone();
+        let device_size = disk.size_in_bytes();
+        let mut written_through = DeviceOfBytes::filled_with_a_pattern(23);
+        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
+        let mut state = 1_u64;
+        let mut refused_requests = 0_u64;
+        let mut kept_writes = 0_u64;
+        // 迭代上界是请求个数；跨轮带着两块盘与随机数的状态；出口只有断言。
+        for request_number in 1..=REQUESTS_OF_THE_RANDOM_STREAM {
+            let offset = DeviceOffsetInBytes(random_offset(&mut state, device_size));
+            let length = random_length(&mut state, device_size);
+            let length_in_memory = usize::try_from(length).expect("长度不过两倍盘长");
+            let (request, through, in_memory) = match next_random(&mut state) % 3 {
+                0 => {
+                    let mut through_buffer = vec![0xAA_u8; length_in_memory];
+                    let mut in_memory_buffer = vec![0xAA_u8; length_in_memory];
+                    let through = written_through.read_at(offset, &mut through_buffer);
+                    let in_memory = kept.read_at(offset, &mut in_memory_buffer);
+                    assert!(
+                        through_buffer == in_memory_buffer,
+                        "第 {request_number} 个请求（读 {} 起 {length} 字节）：读回的字节相同",
+                        offset.0
+                    );
+                    ("读", through, in_memory)
+                }
+                1 => {
+                    let bytes: Vec<u8> = (0..length_in_memory)
+                        .map(|_| u8::try_from(next_random(&mut state) % 256).expect("小于 256"))
+                        .collect();
+                    (
+                        "写",
+                        written_through.write_at(offset, &bytes, WriteDurability::Plain),
+                        kept.write_at(offset, &bytes, WriteDurability::ForceUnitAccess),
+                    )
+                }
+                _ => (
+                    "写零",
+                    written_through.write_zeroes_at(offset, length),
+                    kept.write_zeroes_at(offset, length),
+                ),
+            };
+            assert_eq!(
+                format!("{through:?}"),
+                format!("{in_memory:?}"),
+                "第 {request_number} 个请求（{request} {} 起 {length} 字节）：交回值相同",
+                offset.0
+            );
+            match through {
+                Ok(()) => kept_writes += u64::from(request != "读"),
+                Err(_) => refused_requests += 1,
+            }
+            if next_random(&mut state).is_multiple_of(50) {
+                kept.barrier().expect("屏障什么都不做");
+            }
+        }
+        assert!(
+            refused_requests > 0 && kept_writes > 0,
+            "这一串里既有被拒的请求，也有做成的写与写零（阳性对照）：拒 {refused_requests} 个、写 {kept_writes} 个"
+        );
+        assert!(
+            read_blocks(&kept, 0, BLOCKS_OF_THE_DISK) == written_through.bytes,
+            "整盘读回与写穿的那块盘相同"
+        );
+        assert!(
+            disk.bytes == bytes_before,
+            "一串写之后底下那块盘一个字节都没变"
+        );
+    }
+}
diff --git a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
index 3d85bc5..2e3f9ca 100644
--- a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
+++ b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
@@ -47,7 +47,7 @@ use std::cell::Cell;
 use std::rc::Rc;
 
 use common_unwitnessed_publish::{
-    checker_violations_on, device_bytes_of_the_384_slot_small_devices,
+    checker_violations_on, content_of, device_bytes_of_the_384_slot_small_devices,
     devices_reading_back_all_zero_on, devices_reading_back_all_zero_on_until_lifted,
     fixed_geometry_of_the_384_slot_small_devices, image_of_devices_reading_back_all_zero,
     parameters_of_the_384_slot_small_devices, plain_device, plain_devices_on, pool_after_the_mount,
@@ -56,7 +56,7 @@ use common_unwitnessed_publish::{
     RangesReadingBackAllZero, ReadBackAllZeroLift, RecordedPublish, OVERWRITE_BYTES,
 };
 use singlefs_core::address::{
-    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InodeNumber, InstanceGeneration, SlotNumber,
 };
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::allocator::{AllocationRecord, PlacementOnDevice, PoolAllocator};
@@ -74,7 +74,8 @@ use singlefs_core::mount::{
 };
 use singlefs_core::mounted_session::{
     MountedSession, NoSpaceAfterRaisingTheFloor, RootRingPushRefused, RootRingPushRehearsal,
-    RootRingPushed, StuckBehindTheWideProtection, UserChangeRefused, WideProtectionAtTheCeiling,
+    RootRingPushed, StuckBehindTheWideProtection, UserChange, UserChangeRefused,
+    WaysOutOfTheWideProtection, WaysOutOfTheWideProtectionRefused, WideProtectionAtTheCeiling,
 };
 use singlefs_core::pointer::{LocationEntry, NodePointer};
 use singlefs_core::recovery::{
@@ -83,7 +84,7 @@ use singlefs_core::recovery::{
 };
 use singlefs_core::root_record::RootRecord;
 use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
-use singlefs_core::transaction::{PoolVersion, PublishError};
+use singlefs_core::transaction::{FirstFile, PoolVersion, PublishError, FIRST_INODE_NUMBER};
 use singlefs_harness::memory_pool::{
     writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
     SparseDevice,
@@ -3850,25 +3851,27 @@ fn wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
             && isolated_by_the_wide_protection <= isolated_slots_of(&pool.session().allocator),
         "{form:?}：宽护挡着的槽数大于 0、不多于两块盘隔离的槽数：{isolated_by_the_wide_protection}"
     );
-    let ways_out = stuck.ways_out();
+    let why = format!("{form:?}");
+    let WaysOutOfTheWideProtection {
+        pushing_the_root_ring,
+        ways_out,
+    } = ways_out_of(&pool, &stuck, &why);
     assert_eq!(
         ways_out.len(),
         2,
         "{form:?}：预演过推完之后这次覆盖写放得下，两条出路都列：{ways_out:?}"
     );
-    let why = format!("{form:?}");
     assert_the_way_out_by_unmounting_names_r_abandoned_and_warns_of_a_permanently_unreadable_account(
         &ways_out[0],
         recorded.key.1,
         &why,
     );
-    let released_in_the_rehearsal = released_by_the_push_if_the_change_is_admitted_after_it(&stuck)
-        .unwrap_or_else(|| {
-            panic!(
-                "{form:?}：预演推完之后这次覆盖写放得下，而预演的结局是 {:?}",
-                stuck.pushing_the_root_ring
-            )
-        });
+    let released_in_the_rehearsal = released_by_the_push_if_the_change_is_admitted_after_it(
+        &pushing_the_root_ring,
+    )
+    .unwrap_or_else(|| {
+        panic!("{form:?}：预演推完之后这次覆盖写放得下，而预演的结局是 {pushing_the_root_ring:?}")
+    });
     assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_r_and_the_released_slots(
         &stuck,
         &ways_out[1],
@@ -4058,35 +4061,38 @@ fn unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_th
 /// 推到半路取不到落点那一形人为压到每块盘没挡的槽几个（调查报告第七节第 1 条 `PROBE_FILL_ISO_LEAVE=40`）。
 const UNBLOCKED_SLOTS_LEFT_ON_EACH_DEVICE_BEFORE_THE_PUSH: u64 = 40;
 
-/// 推到半路空发布取不到落点（调查报告第七节第 1 条造的那一次）：主形被宽护卡住之后，把每块盘没挡的槽隔离到只剩 40 个（人为的），
-/// 推环要发的那一串在第 7 次左右取不到落点。推环入口在任何写之前拒（预演里被拒，`PlacementRefused`）：两块盘逐字节不变、
-/// 分配器（`{:?}` 整份）与会话的现行那一版都不变——不留半截不一致的状态。
-#[test]
-fn pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write() {
-    let PoolStuckBehindTheWideProtection {
-        mut pool, stuck, ..
-    } = pool_stuck_behind_the_wide_protection(
+/// 主形卡住之后把每块盘没挡的槽隔离到只剩 [`UNBLOCKED_SLOTS_LEFT_ON_EACH_DEVICE_BEFORE_THE_PUSH`] 个（注入级）。
+fn pool_stuck_with_too_few_unblocked_slots_for_the_whole_push() -> PoolStuckBehindTheWideProtection
+{
+    let mut stuck_pool = pool_stuck_behind_the_wide_protection(
         FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
             overwrites_before: OVERWRITES_BEFORE_THE_ROLLBACK_SIX_VERSIONS_BACK,
         },
     );
-    {
-        let session = pool.session.as_mut().expect("挂着的会话");
-        for device_map in &mut session.allocator.devices {
-            let unit_area_start = device_map.unit_area_start().slot().0;
-            // 迭代上界是单元区槽数；跨轮只带没挡的槽数；出口：只剩下要留的那几个。
-            for slot in unit_area_start..unit_area_start + device_map.unit_area_slots() {
-                if device_map.unblocked_slots()
-                    <= UNBLOCKED_SLOTS_LEFT_ON_EACH_DEVICE_BEFORE_THE_PUSH
-                {
-                    break;
-                }
-                if !device_map.is_blocked_for_commit_generated(SlotNumber(slot)) {
-                    device_map.isolate(SlotNumber(slot), 1);
-                }
+    let session = stuck_pool.pool.session.as_mut().expect("挂着的会话");
+    for device_map in &mut session.allocator.devices {
+        let unit_area_start = device_map.unit_area_start().slot().0;
+        // 迭代上界是单元区槽数；跨轮只带没挡的槽数；出口：只剩下要留的那几个。
+        for slot in unit_area_start..unit_area_start + device_map.unit_area_slots() {
+            if device_map.unblocked_slots() <= UNBLOCKED_SLOTS_LEFT_ON_EACH_DEVICE_BEFORE_THE_PUSH {
+                break;
+            }
+            if !device_map.is_blocked_for_commit_generated(SlotNumber(slot)) {
+                device_map.isolate(SlotNumber(slot), 1);
             }
         }
     }
+    stuck_pool
+}
+
+/// 推到半路空发布取不到落点（调查报告第七节第 1 条造的那一次）：主形被宽护卡住之后，把每块盘没挡的槽隔离到只剩 40 个（人为的），
+/// 推环要发的那一串在第 7 次左右取不到落点。推环入口在任何写之前拒（预演里被拒，`PlacementRefused`）：两块盘逐字节不变、
+/// 分配器（`{:?}` 整份）与会话的现行那一版都不变——不留半截不一致的状态。
+#[test]
+fn pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write() {
+    let PoolStuckBehindTheWideProtection {
+        mut pool, stuck, ..
+    } = pool_stuck_with_too_few_unblocked_slots_for_the_whole_push();
     let image_before = pool.image();
     let allocator_before = format!("{:?}", pool.session().allocator);
     let current_root_before = *pool.session().current.root();
@@ -4130,6 +4136,48 @@ fn pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before
     );
 }
 
+/// 要出路时预演里推环自己被拒（攻方 `research/prompts/m3-batch10z-tu-code-r1-opus-output.md` 第五节变异 K：自然历史里没走到这一支，注入级）：
+/// 主形卡住之后把每块盘没挡的槽隔离到只剩 40 个，要出路：预演交回推环被拒（预演里那一串取不到落点）、出路只列卸载再挂，
+/// 要出路那一次两块盘逐字节不变；照出路二真调推环入口，拒成同一种，`Debug` 逐字相同。
+#[test]
+fn ways_out_when_the_push_finds_no_placements_in_the_rehearsal_list_only_unmounting_and_the_push_is_refused_the_same_way(
+) {
+    let PoolStuckBehindTheWideProtection {
+        mut pool, stuck, ..
+    } = pool_stuck_with_too_few_unblocked_slots_for_the_whole_push();
+    let image_before = pool.image();
+    let WaysOutOfTheWideProtection {
+        pushing_the_root_ring,
+        ways_out,
+    } = ways_out_of(&pool, &stuck, "没挡的槽只剩 40 个");
+    assert!(pool.image() == image_before, "要出路：两块盘逐字节不变");
+    let RootRingPushRehearsal::PushRefused(rehearsed_push_refusal) = pushing_the_root_ring else {
+        panic!("预演里推环那一串取不到落点：预演交回推环被拒，而交回 {pushing_the_root_ring:?}");
+    };
+    assert!(
+        matches!(
+            *rehearsed_push_refusal,
+            RootRingPushRefused::RefusedByTheRehearsalBeforeAnyWrite {
+                cause: PublishError::PlacementRefused { .. },
+                ..
+            }
+        ),
+        "预演里推环拒成推环自己那一串预演取不到落点（阳性对照：注入打到了）：{rehearsed_push_refusal:?}"
+    );
+    assert_eq!(
+        ways_out.len(),
+        1,
+        "推环在预演里被拒：只列卸载再挂：{ways_out:?}"
+    );
+    let real_push_refusal =
+        push_the_root_ring_of(&mut pool).expect_err("照出路二真调推环入口：同样被拒");
+    assert_eq!(
+        format!("{real_push_refusal:?}"),
+        format!("{rehearsed_push_refusal:?}"),
+        "预演里推环的拒绝与真调推环入口的拒绝逐字相同"
+    );
+}
+
 /// 推满仍不够、F 已在上限而没有 C 那一圈（崩在静止点之后再挂，水位之上没有没施加的记录）：照旧只报空间不够、不报「被宽护卡住」；
 /// 这时调推环入口也不推（C 那一圈没有记着落点的槽），两块盘逐字节不变。池照 `admission_raises_the_floor_before_refusing.rs` 里
 /// 「截断之后同样大小写回」那一条起（单元区 256 槽两块盘，崩了再挂、覆盖写 3 次、顺序写把文件长到会话报空间不够）。
@@ -4587,11 +4635,68 @@ fn isolated_slots_released_between(
         .collect()
 }
 
+/// 要出路（会话的 `ways_out_of_the_wide_protection`，盘表交这一刻池里的那一份）；会话不在被拒的那一版上就当场判红（`why` 只用在断言消息里）。
+fn ways_out_of<Device: DeviceOverASparseImage>(
+    pool: &PoolUnderTest<Device>,
+    stuck: &StuckBehindTheWideProtection,
+    why: &str,
+) -> WaysOutOfTheWideProtection {
+    pool.session()
+        .ways_out_of_the_wide_protection(&pool.devices, stuck)
+        .unwrap_or_else(|refusal| {
+            panic!("{why}：会话还在被拒的那一版上，要出路做得成：{refusal:?}")
+        })
+}
+
+/// 照出路二真走一遍（这一刻盘面与会话的一份拷贝上推环、再发同一个改动 `change`），交回再发被拒的那一份；推环被拒或再发做成就当场判红。
+fn refusal_of_the_same_change_after_pushing_the_root_ring_on_a_copy_of(
+    pool: &PoolUnderTest<SparseBlockDevice>,
+    change: UserChange<'_>,
+    why: &str,
+) -> UserChangeRefused {
+    let (mut copy, _) = pool_failing_one_changing_call_copied_from(pool, None);
+    push_the_root_ring_of(&mut copy)
+        .unwrap_or_else(|push_refusal| panic!("{why}：照出路二推环做得成：{push_refusal:?}"));
+    let mut session = copy.session.take().expect("挂着的会话");
+    match session.publish_user_change(&mut copy.devices, change) {
+        Ok(published) => panic!(
+            "{why}：预演说推完之后这次改动照旧被拒，照出路二推完再发却做成了：{:?}",
+            published.versions_published.len()
+        ),
+        Err(refusal) => refusal,
+    }
+}
+
+/// 预演说推完照旧被拒时（`ChangeStillRefusedAfterThePush`），预演交回的拒绝与照出路二真推完、再发同一个改动交回的拒绝 `Debug` 逐字相同
+/// （攻方 `research/prompts/m3-batch10z-tu-code-r1-opus-model/` 的 `atk_follow_way_two` 那一判 `dbg_same`：只进内存的盘读不回自己写过的块、
+/// 写进内存时块号错一位，列不列推环不变，变的是这一份拒绝）。
+fn assert_the_rehearsed_refusal_is_the_refusal_after_pushing_the_root_ring(
+    pool: &PoolUnderTest<SparseBlockDevice>,
+    pushing_the_root_ring: &RootRingPushRehearsal,
+    change: UserChange<'_>,
+    why: &str,
+) {
+    let RootRingPushRehearsal::ChangeStillRefusedAfterThePush {
+        refusal: rehearsed_refusal,
+        ..
+    } = pushing_the_root_ring
+    else {
+        panic!("{why}：预演推环做得成、推完之后这次改动照旧被拒：{pushing_the_root_ring:?}");
+    };
+    let refusal_after_the_push =
+        refusal_of_the_same_change_after_pushing_the_root_ring_on_a_copy_of(pool, change, why);
+    assert_eq!(
+        format!("{rehearsed_refusal:?}"),
+        format!("{refusal_after_the_push:?}"),
+        "{why}：预演交回的拒绝与推环之后真再发同一个改动交回的拒绝逐字相同"
+    );
+}
+
 /// 出路里推环那一条预演的结局是「推完放得下」时，交回预演里每块盘放出来的隔离槽数；别的结局交 `None`。
 fn released_by_the_push_if_the_change_is_admitted_after_it(
-    stuck: &StuckBehindTheWideProtection,
+    pushing_the_root_ring: &RootRingPushRehearsal,
 ) -> Option<Vec<(DeviceIdentity, u64)>> {
-    match &stuck.pushing_the_root_ring {
+    match pushing_the_root_ring {
         RootRingPushRehearsal::ChangeAdmittedAfterThePush {
             isolated_slots_released_by_the_push_on_each_device,
         } => Some(isolated_slots_released_by_the_push_on_each_device.clone()),
@@ -4869,17 +4974,20 @@ const DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY: usize = 13;
 const DATA_UNITS_A_WRITE_STILL_REFUSED_AFTER_THE_PUSH_GROWS_THE_FILE_BY: usize = 20;
 const DATA_UNITS_A_WRITE_REFUSED_EVEN_WITHOUT_THE_WIDE_PROTECTION_GROWS_THE_FILE_BY: usize = 36;
 
-/// 甲（攻方 K1 那一形 g = 13）：报「被宽护卡住」之前预演推环与推完之后这次改动、推完放得下——两条出路都列：出路一说 R 那一槽没有读得出的有效根，
+/// 甲（攻方 K1 那一形 g = 13）：要出路时预演推环与推完之后这次改动、推完放得下——两条出路都列：出路一说 R 那一槽没有读得出的有效根，
 /// 出路二写明推几次、写掉几槽、R 那一槽、放出来的隔离槽数。挂上之后第一次被拒时会话推了一串抬 F、把 F 抬到上限；再试同一个写，
-/// 那一次调用一个写都不推，判的就是预演：一个写都不发、会话的分配器不动。照推环走完，放出来的隔离槽数就是推完前后
-/// 每块盘隔离数之差，同一个写放得下，池级 checker 不红 I-7.4、I-4.8。
+/// 那一次调用一串抬 F 都不推。报「被宽护卡住」那一次不预演（用户 2026-10-09 弹窗定「改：预演延到要出路时才做」）：一块只进内存的盘都没建
+/// （只进内存的盘一建就探底下那块盘的物理块大小，发布路径别处不探：数探的次数）；要出路那一次每块盘建一块、一个写都不发、会话的分配器不动。
+/// 照推环走完，放出来的隔离槽数就是推完前后每块盘隔离数之差，同一个写放得下，池级 checker 不红 I-7.4、I-4.8。
 #[test]
 fn a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push(
 ) {
     let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
-    let mut pool = crashed.mounted_and_stuck_once_by_a_write_grown_by(
+    let stuck_once = crashed.mounted_and_stuck_once_by_a_write_grown_by(
         DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY,
     );
+    let (mut pool, calls_of_the_pool) =
+        pool_failing_one_changing_call_copied_from(&stuck_once, None);
     let length = crashed
         .bytes_of_the_file_grown_by(DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY);
     let image_before_the_write = pool.image();
@@ -4895,11 +5003,35 @@ fn a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_
     assert_eq!(
         (floor_raises_pushed, versions_published),
         (0, 0),
-        "第一次被拒时已把 F 抬到上限：再试这一次一串抬 F 都没推、一版都没发（阳性对照：下面两句判的只剩预演）"
+        "第一次被拒时已把 F 抬到上限：再试这一次一串抬 F 都没推、一版都没发（阳性对照：盘上的改动只可能来自预演）"
+    );
+    assert_eq!(
+        (
+            calls_of_the_pool.probes_of_the_physical_block_size.get(),
+            calls_of_the_pool.calls.get()
+        ),
+        (0, 0),
+        "报「被宽护卡住」那一次不预演：一块只进内存的盘都没建、一个改盘调用都没有"
     );
     assert!(
         pool.image() == image_before_the_write,
-        "预演推环与推完之后这次改动：一个写都不发，两块盘逐字节不变"
+        "报「被宽护卡住」那一次：两块盘逐字节不变"
+    );
+    let WaysOutOfTheWideProtection {
+        pushing_the_root_ring,
+        ways_out,
+    } = ways_out_of(&pool, &stuck, "长 13 个单元");
+    assert_eq!(
+        (
+            calls_of_the_pool.probes_of_the_physical_block_size.get(),
+            calls_of_the_pool.calls.get()
+        ),
+        (u64::try_from(pool.devices.len()).expect("盘数"), 0),
+        "要出路那一次预演：每块盘建一块只进内存的盘，一个改盘调用都没发到底下的盘"
+    );
+    assert!(
+        pool.image() == image_before_the_write,
+        "要出路那一次预演推环与推完之后这次改动：两块盘逐字节不变"
     );
     assert_eq!(
         format!("{:?}", pool.session().allocator),
@@ -4910,20 +5042,17 @@ fn a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_
         stuck.checkpoint_txg_of_r, None,
         "崩在 R 第一条记录之后：R 那一槽没有读得出的有效根（阳性对照：出路一要说的是这一种）"
     );
-    let ways_out = stuck.ways_out();
     assert_eq!(ways_out.len(), 2, "推完放得下：两条出路都列：{ways_out:?}");
     assert_the_way_out_by_unmounting_says_the_ring_slot_of_r_holds_no_readable_root(
         &ways_out[0],
         &stuck,
         "长 13 个单元",
     );
-    let released_in_the_rehearsal = released_by_the_push_if_the_change_is_admitted_after_it(&stuck)
-        .unwrap_or_else(|| {
-            panic!(
-                "预演推完之后这次改动放得下，而预演的结局是 {:?}",
-                stuck.pushing_the_root_ring
-            )
-        });
+    let released_in_the_rehearsal =
+        released_by_the_push_if_the_change_is_admitted_after_it(&pushing_the_root_ring)
+            .unwrap_or_else(|| {
+                panic!("预演推完之后这次改动放得下，而预演的结局是 {pushing_the_root_ring:?}")
+            });
     assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_r_and_the_released_slots(
         &stuck,
         &ways_out[1],
@@ -4957,8 +5086,8 @@ fn a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_
     );
 }
 
-/// 甲（攻方 K1 那一形）：推完照旧放不下的写，报「被宽护卡住」时只列卸载再挂（预演推完之后照旧被拒），照它走：正常卸载、再可写挂载，
-/// 同一个写放得下，池级 checker 不红 I-7.4、I-4.8。
+/// 甲（攻方 K1 那一形）：推完照旧放不下的写，被宽护卡住、要出路时只列卸载再挂（预演推完之后照旧被拒）；预演交回的拒绝与照出路二
+/// 在一份拷贝上真推完、再发同一个写交回的拒绝逐字相同。照出路一走：正常卸载、再可写挂载，同一个写放得下，池级 checker 不红 I-7.4、I-4.8。
 fn a_write_stuck_behind_the_wide_protection_the_push_does_not_let_through_lists_only_unmounting_which_lets_it_through(
     data_units_grown_by: usize,
 ) {
@@ -4969,15 +5098,17 @@ fn a_write_stuck_behind_the_wide_protection_the_push_does_not_let_through_lists_
     let refusal = pool.sequential_write(length).expect_err("宽护挡着准入");
     let WriteStuckBehindTheWideProtection { stuck, .. } =
         stuck_behind_the_wide_protection_reported_by(refusal, &why);
+    let WaysOutOfTheWideProtection {
+        pushing_the_root_ring,
+        ways_out,
+    } = ways_out_of(&pool, &stuck, &why);
     assert!(
         matches!(
-            stuck.pushing_the_root_ring,
+            pushing_the_root_ring,
             RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. }
         ),
-        "{why}：预演推环做得成、推完之后这次改动照旧被拒：{:?}",
-        stuck.pushing_the_root_ring
+        "{why}：预演推环做得成、推完之后这次改动照旧被拒：{pushing_the_root_ring:?}"
     );
-    let ways_out = stuck.ways_out();
     assert_eq!(
         ways_out.len(),
         1,
@@ -4988,6 +5119,16 @@ fn a_write_stuck_behind_the_wide_protection_the_push_does_not_let_through_lists_
         &stuck,
         &why,
     );
+    let content_of_the_refused_write = content_of(length, pool.write_time_seconds);
+    assert_the_rehearsed_refusal_is_the_refusal_after_pushing_the_root_ring(
+        &pool,
+        &pushing_the_root_ring,
+        UserChange::SequentialWrite(FirstFile {
+            content: &content_of_the_refused_write,
+            write_time_seconds: pool.write_time_seconds,
+        }),
+        &why,
+    );
     let mut session = pool.session.take().expect("挂着的会话");
     let unmounted = unmount(
         &pool.parameters,
@@ -5040,7 +5181,8 @@ fn a_write_refused_even_without_the_wide_protection_lists_only_unmounting_which_
 }
 
 /// 甲（攻方第一节「实现员自己的三形上……长到 60、80 个单元的推完照旧被拒」）：实现员三形卡住之后，把文件写成这么多个数据单元的顺序写
-/// 报「被宽护卡住」、只列卸载再挂；R 那一槽记着有效根，出路一照 R 的 txg 说它判成被抛弃、写明账永久读不出时的后果。
+/// 被宽护卡住、要出路时只列卸载再挂；R 那一槽记着有效根，出路一照 R 的 txg 说它判成被抛弃、写明账永久读不出时的后果。
+/// 预演交回的拒绝与照出路二在一份拷贝上真推完、再发同一个写交回的拒绝逐字相同。
 fn a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
     form: FormStuckBehindTheWideProtectionOnThe384SlotDevices,
     data_units: usize,
@@ -5049,20 +5191,31 @@ fn a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmountin
     let PoolStuckBehindTheWideProtection {
         recorded, mut pool, ..
     } = pool_stuck_behind_the_wide_protection(form);
-    let refusal = pool
-        .sequential_write(data_units * singlefs_core::unit::data_unit_payload_capacity())
-        .expect_err("宽护挡着准入");
+    let length = data_units * singlefs_core::unit::data_unit_payload_capacity();
+    let refusal = pool.sequential_write(length).expect_err("宽护挡着准入");
     let WriteStuckBehindTheWideProtection { stuck, .. } =
         stuck_behind_the_wide_protection_reported_by(refusal, &why);
+    let WaysOutOfTheWideProtection {
+        pushing_the_root_ring,
+        ways_out,
+    } = ways_out_of(&pool, &stuck, &why);
     assert!(
         matches!(
-            stuck.pushing_the_root_ring,
+            pushing_the_root_ring,
             RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. }
         ),
-        "{why}：预演推环做得成、推完之后这次改动照旧被拒：{:?}",
-        stuck.pushing_the_root_ring
+        "{why}：预演推环做得成、推完之后这次改动照旧被拒：{pushing_the_root_ring:?}"
+    );
+    let content_of_the_refused_write = content_of(length, pool.write_time_seconds);
+    assert_the_rehearsed_refusal_is_the_refusal_after_pushing_the_root_ring(
+        &pool,
+        &pushing_the_root_ring,
+        UserChange::SequentialWrite(FirstFile {
+            content: &content_of_the_refused_write,
+            write_time_seconds: pool.write_time_seconds,
+        }),
+        &why,
     );
-    let ways_out = stuck.ways_out();
     assert_eq!(
         ways_out.len(),
         1,
@@ -5158,9 +5311,12 @@ fn copy_of_the_session(session: &MountedSession) -> MountedSession {
 }
 
 /// 整池的改盘调用（写、写零、屏障，几块盘共用一个数）数到第 `failing_call` 次（从 1 数）时报块设备错、不落盘，只这一次；`None` 只数不报。
+/// 另数整池的读与探物理块大小（只进内存的盘一建就探一次底下那块盘，发布路径别处不探）。
 struct ChangingCallsOfThePool {
     calls: Cell<u64>,
     failing_call: Option<u64>,
+    reads: Cell<u64>,
+    probes_of_the_physical_block_size: Cell<u64>,
 }
 
 struct DeviceFailingOneChangingCall {
@@ -5187,6 +5343,8 @@ impl BlockDevice for DeviceFailingOneChangingCall {
         offset: DeviceOffsetInBytes,
         buffer: &mut [u8],
     ) -> Result<(), BlockDeviceError> {
+        let reads = &self.calls_of_the_pool.reads;
+        reads.set(reads.get() + 1);
         self.inner.read_at(offset, buffer)
     }
 
@@ -5215,6 +5373,8 @@ impl BlockDevice for DeviceFailingOneChangingCall {
     }
 
     fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
+        let probes = &self.calls_of_the_pool.probes_of_the_physical_block_size;
+        probes.set(probes.get() + 1);
         self.inner.probe_physical_block_size()
     }
 
@@ -5240,6 +5400,8 @@ fn pool_failing_one_changing_call_copied_from(
     let calls_of_the_pool = Rc::new(ChangingCallsOfThePool {
         calls: Cell::new(0),
         failing_call,
+        reads: Cell::new(0),
+        probes_of_the_physical_block_size: Cell::new(0),
     });
     let devices = plain_devices_on(&stuck_pool.image())
         .into_iter()
@@ -5777,18 +5939,21 @@ fn the_slots_the_way_out_by_the_push_releases_with_two_ring_slots_holding_overla
     let refusal = pool.sequential_write(length).expect_err("两槽 C 挡着准入");
     let WriteStuckBehindTheWideProtection { stuck, .. } =
         stuck_behind_the_wide_protection_reported_by(refusal, "两槽 C、落点重叠");
-    let released_in_the_rehearsal = released_by_the_push_if_the_change_is_admitted_after_it(&stuck)
-        .unwrap_or_else(|| {
-            panic!(
-                "两槽 C：预演推完之后这次改动放得下，而预演的结局是 {:?}",
-                stuck.pushing_the_root_ring
-            )
-        });
+    let WaysOutOfTheWideProtection {
+        pushing_the_root_ring,
+        ways_out,
+    } = ways_out_of(&pool, &stuck, "两槽 C、落点重叠");
+    let released_in_the_rehearsal =
+        released_by_the_push_if_the_change_is_admitted_after_it(&pushing_the_root_ring)
+            .unwrap_or_else(|| {
+                panic!(
+                    "两槽 C：预演推完之后这次改动放得下，而预演的结局是 {pushing_the_root_ring:?}"
+                )
+            });
     assert_eq!(
         released_in_the_rehearsal, stuck.isolated_slots_on_each_device,
         "根环表里没有被抛弃根：推环放出来的就是 C 那一圈罩着的、隔离着的槽，两槽重叠的那几个只数一次"
     );
-    let ways_out = stuck.ways_out();
     assert_eq!(ways_out.len(), 2, "推完放得下：两条出路都列：{ways_out:?}");
     assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_r_and_the_released_slots(
         &stuck,
@@ -5991,3 +6156,212 @@ fn unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_isolat
         later_slot.0
     );
 }
+
+/// 要出路时会话已不在报「被宽护卡住」时被拒的那一版上（之后推过环）：按哪一刻算条款没写、第一版不支持，拒成点名这一形的那一种、
+/// 带被拒那一版与现行那一版的实例代号与 txg；不预演：一块只进内存的盘都没建、一个盘都没读、一个改盘调用都没有，两块盘逐字节不变。
+#[test]
+fn asking_for_the_ways_out_after_the_session_moved_past_the_version_refused_on_is_refused_without_reading_a_device(
+) {
+    let PoolStuckBehindTheWideProtection {
+        mut pool, stuck, ..
+    } = pool_stuck_behind_the_wide_protection(
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
+            overwrites_before: 40,
+        },
+    );
+    let refused_on = *pool.session().current.root();
+    push_the_root_ring_of(&mut pool).unwrap_or_else(|refusal| panic!("推环做得成：{refusal:?}"));
+    let (counting_pool, calls_of_the_pool) =
+        pool_failing_one_changing_call_copied_from(&pool, None);
+    let current_after_the_push = *counting_pool.session().current.root();
+    assert!(
+        current_after_the_push.checkpoint_txg > refused_on.checkpoint_txg,
+        "推环之后会话已在后面的一版上（阳性对照）"
+    );
+    let image_before = counting_pool.image();
+    match counting_pool
+        .session()
+        .ways_out_of_the_wide_protection(&counting_pool.devices, &stuck)
+    {
+        Err(
+            WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion {
+                instance_refused_on,
+                checkpoint_txg_refused_on,
+                instance_of_the_current_version,
+                checkpoint_txg_of_the_current_version,
+            },
+        ) => assert_eq!(
+            (
+                instance_refused_on,
+                checkpoint_txg_refused_on,
+                instance_of_the_current_version,
+                checkpoint_txg_of_the_current_version
+            ),
+            (
+                refused_on.instance,
+                refused_on.checkpoint_txg,
+                current_after_the_push.instance,
+                current_after_the_push.checkpoint_txg
+            ),
+            "带被拒那一版与现行那一版的实例代号与 txg"
+        ),
+        Ok(found) => panic!("会话已不在被拒的那一版上：不预演，而交回 {found:?}"),
+    }
+    assert_eq!(
+        (
+            calls_of_the_pool.probes_of_the_physical_block_size.get(),
+            calls_of_the_pool.reads.get(),
+            calls_of_the_pool.calls.get()
+        ),
+        (0, 0, 0),
+        "不预演：一块只进内存的盘都没建、一个盘都没读、一个改盘调用都没有"
+    );
+    assert!(counting_pool.image() == image_before, "两块盘逐字节不变");
+}
+
+/// 卡在走空里那两形的造法：攻方 K1 那一形挂上之后长 13 个单元被拒一次（会话推一串抬 F、把 F 抬到上限），再把每块盘没挡的槽标成已分配、
+/// 只留 [`UNBLOCKED_SLOTS_LEFT_FOR_THE_WALK_AFTER_THE_UNLINK_INTENT_TO_BE_STUCK`] 个（注入级，位图上占着、不写记录；照
+/// `admission_raises_the_floor_before_refusing.rs` 的 `pool_ready_to_unlink_with_few_unblocked_slots`）。
+/// 实现员副本上的草稿探针（留 0–1000 个 × 先覆盖写 0–20 次）：只有不先覆盖写、留 29–31 个那三格 unlink 卡在意图之后的走空里；
+/// 留 0–18 个卡在意图那一次，留 20–28 个推的抬 F 自己落不下、不报被宽护卡住，留 32 个起做成。
+const UNBLOCKED_SLOTS_LEFT_FOR_THE_WALK_AFTER_THE_UNLINK_INTENT_TO_BE_STUCK: u64 = 30;
+
+fn pool_whose_unlink_of_the_first_file_is_stuck_in_the_walk_after_the_intent() -> (
+    PoolUnderTest<SparseBlockDevice>,
+    StuckBehindTheWideProtection,
+) {
+    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
+    let mut pool = crashed.mounted_and_stuck_once_by_a_write_grown_by(
+        DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY,
+    );
+    for device_map in &mut pool.session.as_mut().expect("挂着的会话").allocator.devices {
+        let unit_area_start = device_map.unit_area_start().slot().0;
+        // 迭代上界是单元区槽数；跨轮只带没挡的槽数；出口：只剩下要留的那几个。
+        for slot in unit_area_start..unit_area_start + device_map.unit_area_slots() {
+            if device_map.unblocked_slots()
+                <= UNBLOCKED_SLOTS_LEFT_FOR_THE_WALK_AFTER_THE_UNLINK_INTENT_TO_BE_STUCK
+            {
+                break;
+            }
+            if !device_map.is_blocked_for_commit_generated(SlotNumber(slot)) {
+                device_map.mark_allocated(SlotNumber(slot), 1);
+            }
+        }
+    }
+    pool.write_time_seconds += 1;
+    let refusal = pool
+        .publish(
+            UserChange::Unlink {
+                inode: InodeNumber(FIRST_INODE_NUMBER),
+                write_time_seconds: pool.write_time_seconds,
+            },
+            None,
+        )
+        .expect_err("没挡的槽只剩 30 个：unlink 被拒");
+    let WriteStuckBehindTheWideProtection {
+        versions_published,
+        stuck,
+        ..
+    } = stuck_behind_the_wide_protection_reported_by(refusal, "unlink 意图之后的走空");
+    assert_eq!(
+        versions_published, 1,
+        "意图那一次发成、被拒的是它之后走空的一步（阳性对照：卡在走空里）"
+    );
+    let current = pool
+        .session()
+        .current
+        .version_of_trees()
+        .expect("意图那一版有树");
+    assert_eq!(
+        current.deleted_inodes,
+        vec![InodeNumber(FIRST_INODE_NUMBER)],
+        "现行那一版是意图那一版：队列里是第一个文件"
+    );
+    (pool, stuck)
+}
+
+/// 报里记着的「这次改动还剩什么」从 `{:?}` 读（私有字段，只有会话造得出这一份报）。
+fn remainder_of_the_change_recorded_in(stuck: &StuckBehindTheWideProtection) -> String {
+    let debug = format!("{stuck:?}");
+    let start = debug
+        .find("remainder_of_the_change: ")
+        .expect("报的 `{:?}` 里有这次改动还剩什么那一项");
+    debug[start..].to_string()
+}
+
+/// 要出路时预演里推环自己被拒（卡在走空里那两形：没挡的槽只剩那么几个，推环那一串第二次就取不到落点）：只列卸载再挂，
+/// 预演交回的推环拒绝与真调推环入口的拒绝逐字相同。
+fn assert_the_push_is_refused_in_the_rehearsal_as_it_is_for_real(
+    pool: &mut PoolUnderTest<SparseBlockDevice>,
+    stuck: &StuckBehindTheWideProtection,
+    why: &str,
+) {
+    let WaysOutOfTheWideProtection {
+        pushing_the_root_ring,
+        ways_out,
+    } = ways_out_of(pool, stuck, why);
+    let RootRingPushRehearsal::PushRefused(rehearsed_push_refusal) = pushing_the_root_ring else {
+        panic!("{why}：没挡的槽只剩那么几个，预演里推环被拒，而预演交回 {pushing_the_root_ring:?}");
+    };
+    assert_eq!(ways_out.len(), 1, "{why}：只列卸载再挂：{ways_out:?}");
+    let real_push_refusal = push_the_root_ring_of(pool).expect_err("真调推环入口：同样被拒");
+    assert_eq!(
+        format!("{real_push_refusal:?}"),
+        format!("{rehearsed_push_refusal:?}"),
+        "{why}：预演里推环的拒绝与真调推环入口的拒绝逐字相同"
+    );
+}
+
+/// unlink 意图之后的走空被宽护卡住（攻方第五节变异 H：这一处记成「走空再发这次改动」时，推完之后预演会把 unlink 再发一次）：
+/// 报里记着的这次改动还剩什么只是走空。预演里推环被拒（造这一形要把没挡的槽压到推环也取不到落点：推完之后那一段这一形走不到，
+/// 只钉报里记的那一项与出路）。
+#[test]
+fn an_unlink_whose_walk_after_the_intent_is_stuck_behind_the_wide_protection_keeps_only_the_walk_as_what_remains_of_the_change(
+) {
+    let (mut pool, stuck) =
+        pool_whose_unlink_of_the_first_file_is_stuck_in_the_walk_after_the_intent();
+    let remainder = remainder_of_the_change_recorded_in(&stuck);
+    assert!(
+        remainder.starts_with("remainder_of_the_change: WalkTheQueue }"),
+        "意图已发成、被拒的是它之后走空的一步：报里记着的这次改动还剩什么只是走空，而记的是 {remainder}"
+    );
+    assert_the_push_is_refused_in_the_rehearsal_as_it_is_for_real(
+        &mut pool,
+        &stuck,
+        "unlink 意图之后的走空",
+    );
+}
+
+/// 一次改动之前的走空被宽护卡住（攻方第五节变异 F：这一处记成「只剩走空」时，推完之后预演不再发这次改动）：unlink 卡在意图之后的走空里，
+/// 再覆盖写一次——先走空那一步就被拒、被宽护卡住；报里记着的这次改动还剩什么是走空再发这次覆盖写。预演里推环被拒（同上一条）。
+#[test]
+fn an_overwrite_whose_walk_before_it_is_stuck_behind_the_wide_protection_keeps_the_walk_then_the_overwrite_as_what_remains_of_the_change(
+) {
+    let (mut pool, _stuck_in_the_unlink) =
+        pool_whose_unlink_of_the_first_file_is_stuck_in_the_walk_after_the_intent();
+    let refusal = pool
+        .overwrite(OVERWRITE_BYTES)
+        .expect_err("队列里还有第一个文件：覆盖写之前先走空，走空那一步照旧被拒");
+    let WriteStuckBehindTheWideProtection {
+        versions_published,
+        stuck,
+        ..
+    } = stuck_behind_the_wide_protection_reported_by(refusal, "覆盖写之前的走空");
+    assert_eq!(
+        versions_published, 0,
+        "走空那一步就被拒：一版都没发成（阳性对照：卡在这次改动之前的走空里）"
+    );
+    let remainder = remainder_of_the_change_recorded_in(&stuck);
+    assert!(
+        remainder.starts_with(
+            "remainder_of_the_change: WalkTheQueueThenPublishTheChange(Overwrite {"
+        ),
+        "被拒的是这次改动之前走空的一步：报里记着的这次改动还剩什么是走空再发这次覆盖写，而记的是 {}",
+        remainder.chars().take(120).collect::<String>()
+    );
+    assert_the_push_is_refused_in_the_rehearsal_as_it_is_for_real(
+        &mut pool,
+        &stuck,
+        "覆盖写之前的走空",
+    );
+}
````

### 第二十二块变异表追加（23 行）

````tsv
第10z批第二十二块 报「被宽护卡住」那一次又预演一遍（结局扔掉）：报错那一步照旧建只进内存的盘、拷分配器、走推环	crates/singlefs-core/src/mounted_session.rs	                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(\n                    allocator, current, remainder, &stop,\n                ),	                wide_protection_at_the_ceiling: {\n                    drop(rehearse_the_push_then_the_remainder_of_the_change(\n                        parameters,\n                        devices.as_slice(),\n                        allocator,\n                        current,\n                        current.root.instance,\n                        shadow_ledger,\n                        remainder,\n                    ));\n                    wide_protection_at_the_ceiling(allocator, current, remainder, &stop)\n                },	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push	a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push
第10z批第二十二块 要出路时不预演、恒列推环（推完照旧被拒的长 20 个单元也列两条）	crates/singlefs-core/src/mounted_session.rs	        let pushing_the_root_ring = rehearse_the_push_then_the_remainder_of_the_change(\n            self.parameters_and_device_table.parameters_on_disk(),\n            devices,\n            &self.allocator,\n            version_refused_on,\n            self.instance,\n            self.shadow_ledger,\n            stuck.remainder_of_the_change.as_remainder(),\n        );	        let _ = (devices, version_refused_on);\n        let pushing_the_root_ring = RootRingPushRehearsal::ChangeAdmittedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device: Vec::new(),\n        };	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through	a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through
第10z批第二十二块 要出路时不看会话还在不在被拒的那一版上（推过环之后照这一刻预演）	crates/singlefs-core/src/mounted_session.rs	                if is_the_version_refused_on(version_of_trees) =>	                if true || is_the_version_refused_on(version_of_trees) =>	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- asking_for_the_ways_out_after_the_session_moved_past_the_version_refused_on_is_refused_without_reading_a_device	asking_for_the_ways_out_after_the_session_moved_past_the_version_refused_on_is_refused_without_reading_a_device
第10z批第二十二块 报里记的被拒那一版 txg 错一位（会话没动也要不到出路）	crates/singlefs-core/src/mounted_session.rs	            checkpoint_txg_of_the_version_refused_on: current.root.checkpoint_txg,	            checkpoint_txg_of_the_version_refused_on: CheckpointTxg(current.root.checkpoint_txg.0 + 1),	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push	a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push
第10z批第二十二块 报里记的顺序写内容丢掉（推完照旧被拒的长 20 个单元预演成放得下）	crates/singlefs-core/src/mounted_session.rs	            UserChange::SequentialWrite(file) => UserChangeKeptByTheRefusal::SequentialWrite {\n                content: file.content.to_vec(),	            UserChange::SequentialWrite(file) => UserChangeKeptByTheRefusal::SequentialWrite {\n                content: Vec::new(),	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through	a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through
第10z批第二十二块 报里把覆盖写记成顺序写	crates/singlefs-core/src/mounted_session.rs	            UserChange::Overwrite(file) => UserChangeKeptByTheRefusal::Overwrite {	            UserChange::Overwrite(file) => UserChangeKeptByTheRefusal::SequentialWrite {	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- an_overwrite_whose_walk_before_it_is_stuck_behind_the_wide_protection_keeps_the_walk_then_the_overwrite_as_what_remains_of_the_change	an_overwrite_whose_walk_before_it_is_stuck_behind_the_wide_protection_keeps_the_walk_then_the_overwrite_as_what_remains_of_the_change
第10z批第二十二块 攻方 K 预演里推环被拒当成推完放得下（要出路时只列卸载那一形列两条）	crates/singlefs-core/src/mounted_session.rs	        return RootRingPushRehearsal::PushRefused(Box::new(refusal));	        let _ = refusal;\n        return RootRingPushRehearsal::ChangeAdmittedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device: Vec::new(),\n        };	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- ways_out_when_the_push_finds_no_placements_in_the_rehearsal_list_only_unmounting_and_the_push_is_refused_the_same_way	ways_out_when_the_push_finds_no_placements_in_the_rehearsal_list_only_unmounting_and_the_push_is_refused_the_same_way
第10z批第二十二块 攻方 F 走空之前被拒时报里只记走空（推完之后预演不再发这次改动）	crates/singlefs-core/src/mounted_session.rs	                RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),\n                published_in_this_change,	                RemainderOfTheUserChange::WalkTheQueue,\n                published_in_this_change,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- an_overwrite_whose_walk_before_it_is_stuck_behind_the_wide_protection_keeps_the_walk_then_the_overwrite_as_what_remains_of_the_change	an_overwrite_whose_walk_before_it_is_stuck_behind_the_wide_protection_keeps_the_walk_then_the_overwrite_as_what_remains_of_the_change
第10z批第二十二块 攻方 H unlink 意图之后走空被拒时报里记成走空再发这次改动（推完之后预演把 unlink 再发一次）	crates/singlefs-core/src/mounted_session.rs	                        RemainderOfTheUserChange::WalkTheQueue,\n                        published_in_this_change,	                        RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),\n                        published_in_this_change,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- an_unlink_whose_walk_after_the_intent_is_stuck_behind_the_wide_protection_keeps_only_the_walk_as_what_remains_of_the_change	an_unlink_whose_walk_after_the_intent_is_stuck_behind_the_wide_protection_keeps_only_the_walk_as_what_remains_of_the_change
第10z批第二十二块 攻方 B 只进内存的盘读不回自己写过的块（预演交回的拒绝换成别的原因）	crates/singlefs-core/src/mounted_session.rs	            match self.written_blocks.get(&block_index) {	            match self.written_blocks.get(&(block_index | (1 << 63))) {	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through	a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through
第10z批第二十二块 攻方 D 只进内存的盘写进内存时块号错一位（预演交回的拒绝换成别的原因）	crates/singlefs-core/src/mounted_session.rs	                .insert(first_block + block_offset, block.to_vec());	                .insert(first_block + block_offset + 1, block.to_vec());	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through	a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through
第10z批第二十二块 只进内存的盘读不回自己写过的块（拼接那一条）	crates/singlefs-core/src/mounted_session.rs	            match self.written_blocks.get(&block_index) {	            match self.written_blocks.get(&(block_index | (1 << 63))) {	-p singlefs-core --lib -- mounted_session::tests::	a_read_across_written_and_unwritten_blocks_stitches_the_kept_blocks_with_the_disk_below
第10z批第二十二块 正推 overlayIgnored 只进内存的盘读时不认写过的块	crates/singlefs-core/src/mounted_session.rs	                Some(written) => {\n                    if let Some(run_start) = unwritten_run_start.take() {	                Some(_written) if true => {\n                    unwritten_run_start.get_or_insert(block_index);\n                }\n                Some(written) => {\n                    if let Some(run_start) = unwritten_run_start.take() {	-p singlefs-core --lib -- mounted_session::tests::	a_read_across_written_and_unwritten_blocks_stitches_the_kept_blocks_with_the_disk_below
第10z批第二十二块 正推 keepOffByOne 只进内存的盘记块号错一位	crates/singlefs-core/src/mounted_session.rs	                .insert(first_block + block_offset, block.to_vec());	                .insert(first_block + block_offset + 1, block.to_vec());	-p singlefs-core --lib -- mounted_session::tests::	a_read_across_written_and_unwritten_blocks_stitches_the_kept_blocks_with_the_disk_below
第10z批第二十二块 正推 zeroesNotKept 只进内存的盘写零不记	crates/singlefs-core/src/mounted_session.rs	        self.keep_in_memory(offset, &zeros);	        let _ = &zeros;	-p singlefs-core --lib -- mounted_session::tests::	zeroes_written_in_memory_read_back_as_zero_over_the_disk_below
第10z批第二十二块 正推 zeroesNotKept 只进内存的盘写零不记（差分对拍那一条）	crates/singlefs-core/src/mounted_session.rs	        self.keep_in_memory(offset, &zeros);	        let _ = &zeros;	-p singlefs-core --lib -- mounted_session::tests::	the_device_keeping_writes_in_memory_agrees_with_a_device_written_through_on_a_random_request_stream
第10z批第二十二块 正推 writeNotKept 只进内存的盘写不记	crates/singlefs-core/src/mounted_session.rs	        self.keep_in_memory(offset, bytes);\n        Ok(())	        let _ = bytes;\n        Ok(())	-p singlefs-core --lib -- mounted_session::tests::	a_write_kept_in_memory_reads_back_and_leaves_the_disk_below_unchanged
第10z批第二十二块 攻方 C 只进内存的盘读的末尾那一段没写过的块不从底下的盘读	crates/singlefs-core/src/mounted_session.rs	        if let Some(run_start) = unwritten_run_start {\n            self.read_unwritten_blocks_from_the_disk(run_start, end_block, first_block, buffer)?;\n        }	        let _ = unwritten_run_start;	-p singlefs-core --lib -- mounted_session::tests::	a_read_of_blocks_never_written_passes_through_to_the_disk_below
第10z批第二十二块 K5 两个后端与只进内存的盘共用的对齐判恒放行（block_device 自己那一条）	crates/singlefs-core/src/block_device.rs	    let block = u64::from(physical_block_size.0);\n    if !offset.0.is_multiple_of(block) || !length.is_multiple_of(block) {	    let block = u64::from(physical_block_size.0);\n    if offset.0 < u64::MAX {\n        return Ok(());\n    }\n    if !offset.0.is_multiple_of(block) || !length.is_multiple_of(block) {	-p singlefs-core --lib -- block_device::tests::unaligned_and_out_of_range_requests_are_rejected_before_touching_the_file	unaligned_and_out_of_range_requests_are_rejected_before_touching_the_file
第10z批第二十二块 K5 两个后端与只进内存的盘共用的对齐判恒放行（只进内存的盘不对齐那一条）	crates/singlefs-core/src/block_device.rs	    let block = u64::from(physical_block_size.0);\n    if !offset.0.is_multiple_of(block) || !length.is_multiple_of(block) {	    let block = u64::from(physical_block_size.0);\n    if offset.0 < u64::MAX {\n        return Ok(());\n    }\n    if !offset.0.is_multiple_of(block) || !length.is_multiple_of(block) {	-p singlefs-core --lib -- mounted_session::tests::	a_request_off_the_physical_block_boundaries_is_refused_before_anything_is_kept
第10z批第二十二块 K5 两个后端与只进内存的盘共用的对齐判恒放行（只进内存的盘越过盘尾那一条）	crates/singlefs-core/src/block_device.rs	    let block = u64::from(physical_block_size.0);\n    if !offset.0.is_multiple_of(block) || !length.is_multiple_of(block) {	    let block = u64::from(physical_block_size.0);\n    if offset.0 < u64::MAX {\n        return Ok(());\n    }\n    if !offset.0.is_multiple_of(block) || !length.is_multiple_of(block) {	-p singlefs-core --lib -- mounted_session::tests::	a_request_past_the_end_of_the_disk_below_is_refused_and_one_ending_at_the_end_is_kept
第10z批第二十二块 正推 lengthUnaligned 共用的对齐判只判偏移	crates/singlefs-core/src/block_device.rs	    if !offset.0.is_multiple_of(block) || !length.is_multiple_of(block) {	    if !offset.0.is_multiple_of(block) {	-p singlefs-core --lib -- mounted_session::tests::	a_request_off_the_physical_block_boundaries_is_refused_before_anything_is_kept
第10z批第二十二块 正推 rangeEndLt 共用的越界判把正好到盘尾的也拒	crates/singlefs-core/src/block_device.rs	        Some(end) if end <= device_size => Ok(()),	        Some(end) if end < device_size => Ok(()),	-p singlefs-core --lib -- mounted_session::tests::	a_request_past_the_end_of_the_disk_below_is_refused_and_one_ending_at_the_end_is_kept
````

### 第二十二块变异表整行替换（5 行）

````tsv
第10z批第十六块 推满仍不够时不分宽护：F 在上限、C 那一圈隔离着也照旧只报空间不够（回退、前缀 20 次）	crates/singlefs-core/src/mounted_session.rs	                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(\n                    allocator, current, remainder, &stop,\n                ),	                wide_protection_at_the_ceiling: WideProtectionAtTheCeiling::NotWhatStopsTheAdmission,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- wide_protection	a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through
第10z批第十六块 推满仍不够时不分宽护：F 在上限、C 那一圈隔离着也照旧只报空间不够（建 inode、前缀 40 次）	crates/singlefs-core/src/mounted_session.rs	                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(\n                    allocator, current, remainder, &stop,\n                ),	                wide_protection_at_the_ceiling: WideProtectionAtTheCeiling::NotWhatStopsTheAdmission,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through	an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through
第10z批第十六块 推满仍不够时不分宽护：F 在上限、C 那一圈隔离着也照旧只报空间不够（删除、前缀 160 次）	crates/singlefs-core/src/mounted_session.rs	                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(\n                    allocator, current, remainder, &stop,\n                ),	                wide_protection_at_the_ceiling: WideProtectionAtTheCeiling::NotWhatStopsTheAdmission,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through	a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through
第10z批第十六块 推满仍不够时不分宽护：F 在上限、C 那一圈隔离着也照旧只报空间不够（卸载重挂那一条出路）	crates/singlefs-core/src/mounted_session.rs	                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(\n                    allocator, current, remainder, &stop,\n                ),	                wide_protection_at_the_ceiling: WideProtectionAtTheCeiling::NotWhatStopsTheAdmission,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_r_and_lets_writes_through	unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_r_and_lets_writes_through
第10z批第十六块 推满仍不够时不分宽护：F 在上限、C 那一圈隔离着也照旧只报空间不够（推到半路取不到落点那一形）	crates/singlefs-core/src/mounted_session.rs	                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(\n                    allocator, current, remainder, &stop,\n                ),	                wide_protection_at_the_ceiling: WideProtectionAtTheCeiling::NotWhatStopsTheAdmission,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write	pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write
````

## 第二十三块

````diff
diff --git a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
index 0a05b83..2aa1f6c 100644
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -2219,10 +2219,7 @@ mod tests {
     #[test]
     fn the_entries_outside_the_card_pool_start_their_devices_as_outside_the_card_pool() {
         let source = include_str!("gpu_unit_checks.rs");
-        let code = source
-            .split(TESTS_MODULE_OPENING)
-            .next()
-            .expect("split 至少交一段");
+        let code = code_before_the_tests_module(source);
         let body_of = |signature: &str| {
             let start = code
                 .find(signature)
@@ -2342,10 +2339,7 @@ mod tests {
     #[test]
     fn the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first() {
         let source = include_str!("gpu_unit_checks.rs");
-        let code = source
-            .split(TESTS_MODULE_OPENING)
-            .next()
-            .expect("split 至少交一段");
+        let code = code_before_the_tests_module(source);
         let body_until = |signature: &str, closing_line: &str| -> String {
             let start = code
                 .find(signature)
@@ -2425,10 +2419,7 @@ mod tests {
     #[test]
     fn the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking() {
         let source = include_str!("gpu_unit_checks.rs");
-        let code = source
-            .split(TESTS_MODULE_OPENING)
-            .next()
-            .expect("split 至少交一段");
+        let code = code_before_the_tests_module(source);
         let signature = "fn take_in_the_devices_found_idle(&self) \u{7b}";
         assert_eq!(
             code.matches(signature).count(),
@@ -2491,7 +2482,7 @@ mod tests {
             .then(|| first_character.len_utf8() + 2)
     }
 
-    /// `text` 是原始字符串去掉 `r`、`br` 之后那一截（以若干个 `#` 与一个 `"` 起头）时，到收尾的 `"` 与同样多个 `#` 为止的长度。
+    /// `text` 是原始字符串去掉 `r`、`br`、`cr` 之后那一截（以若干个 `#` 与一个 `"` 起头）时，到收尾的 `"` 与同样多个 `#` 为止的长度。
     fn length_of_the_raw_string_at_the_start(text: &str) -> usize {
         let hashes = text.len() - text.trim_start_matches('#').len();
         let closing = format!("\"{}", "#".repeat(hashes));
@@ -2520,11 +2511,31 @@ mod tests {
         }
     }
 
-    /// `code` 拆成的词：标识符与关键字（`r#` 原始标识符取它后面的名字）、生命周期（连着它前面的 `'`）、数字（照标识符那样收一段）、`::`、`!=`，
-    /// 其余标点一个字一个词；注释（行注释、可以嵌套的块注释）、字符串与字符字面量（普通的、原始的 `r#"…"#`、字节的）不出词。
+    /// 拼成关键字的原始标识符（`r#mod`、`r#use`）是名字，不是关键字：分词时这几个连着 `r#` 出词，免得被当成关键字（第 10z 批第二十三块，
+    /// 正推 K4-2 L14）；别的原始标识符（`r#std`）与不带 `r#` 的同名，取它后面的名字。
+    const KEYWORDS_A_RAW_IDENTIFIER_CAN_SPELL: [&str; 48] = [
+        "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "do",
+        "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl", "in",
+        "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
+        "return", "static", "struct", "trait", "true", "try", "type", "typeof", "unsafe",
+        "unsized", "use", "virtual", "where", "while", "yield",
+    ];
+
+    /// `code` 拆成的词：标识符与关键字（`r#` 原始标识符取它后面的名字，拼成关键字的连着 `r#`）、生命周期（连着它前面的 `'`）、
+    /// 数字（照标识符那样收一段）、`::`、`!=`，其余标点一个字一个词；注释（行注释、可以嵌套的块注释）、字符串与字符字面量
+    /// （普通的、原始的 `r#"…"#`、`br`、`cr`，字节的、C 的）不出词。认原始字符串的写法与崩溃放量那一份数花括号
+    /// （`crash_amplification.rs` 的 `lines_read_past_comments_and_literals`）相同：`r`、`br`、`cr` 后面紧跟若干个 `#` 与一个 `"`。
     /// 扫源码的用例照它找条目、`use` 引进来的名字与花括号的层数：字面量与注释里的 `mod`、`use`、花括号都不算。
     /// 非 ASCII 的字只落在注释、字面量与标识符里，切出来的词因此总在字符边界上。
     fn rust_tokens_without_comments_and_literals(code: &str) -> Vec<&str> {
+        rust_tokens_with_their_offsets(code)
+            .into_iter()
+            .map(|(_offset, token)| token)
+            .collect()
+    }
+
+    /// [`rust_tokens_without_comments_and_literals`] 的词，各连同它在 `code` 里起头的字节位置。
+    fn rust_tokens_with_their_offsets(code: &str) -> Vec<(usize, &str)> {
         let is_identifier_character =
             |character: char| character.is_alphanumeric() || character == '_';
         let mut tokens = Vec::new();
@@ -2547,7 +2558,7 @@ mod tests {
                         let lifetime_length = 1 + rest[1..]
                             .find(|character: char| !is_identifier_character(character))
                             .unwrap_or(rest.len() - 1);
-                        tokens.push(&rest[..lifetime_length]);
+                        tokens.push((position, &rest[..lifetime_length]));
                         position += lifetime_length;
                     }
                 }
@@ -2557,14 +2568,23 @@ mod tests {
                     .unwrap_or(rest.len());
                 let word = &rest[..word_length];
                 let after_the_word = &rest[word_length..];
-                if matches!(word, "r" | "br")
+                if matches!(word, "r" | "br" | "cr")
                     && after_the_word.trim_start_matches('#').starts_with('"')
                 {
                     position += word_length + length_of_the_raw_string_at_the_start(after_the_word);
                 } else if word == "r" && after_the_word.starts_with('#') {
-                    // 原始标识符 `r#名字`：名字下一圈照标识符收
-                    position += word_length + 1;
-                } else if word == "b" && after_the_word.starts_with('"') {
+                    // 原始标识符 `r#名字`：拼成关键字的连着 `r#` 出词（`r#mod` 是名字，不是关键字），别的名字下一圈照标识符收
+                    let name = &after_the_word[1..];
+                    let name_length = name
+                        .find(|character: char| !is_identifier_character(character))
+                        .unwrap_or(name.len());
+                    if KEYWORDS_A_RAW_IDENTIFIER_CAN_SPELL.contains(&&name[..name_length]) {
+                        tokens.push((position, &rest[..word_length + 1 + name_length]));
+                        position += word_length + 1 + name_length;
+                    } else {
+                        position += word_length + 1;
+                    }
+                } else if matches!(word, "b" | "c") && after_the_word.starts_with('"') {
                     position +=
                         word_length + length_of_the_quoted_literal_at_the_start(after_the_word);
                 } else if word == "b" && after_the_word.starts_with('\'') {
@@ -2572,24 +2592,59 @@ mod tests {
                         + length_of_the_character_literal_at_the_start(after_the_word)
                             .expect("b 后面紧跟引号的是字节字面量");
                 } else {
-                    tokens.push(word);
+                    tokens.push((position, word));
                     position += word_length;
                 }
             } else if rest.starts_with("::") {
-                tokens.push("::");
+                tokens.push((position, "::"));
                 position += 2;
             } else if rest.starts_with("!=") {
                 // 不等号整个一个词：拆成 `!`、`=` 时，条目层 `const 名: bool = 甲 != 乙;` 的 `甲 !` 会被认成调宏
-                tokens.push("!=");
+                tokens.push((position, "!="));
                 position += 2;
             } else {
-                tokens.push(&rest[..first_character.len_utf8()]);
+                tokens.push((position, &rest[..first_character.len_utf8()]));
                 position += first_character.len_utf8();
             }
         }
         tokens
     }
 
+    /// `source`（这份源文件，或用例在它上面改过的一份）测试模块头两行之前的那一段。头两行只认代码里的（花括号第 0 层，注释与字面量之外）：
+    /// 照原文找第一处时，注释或字符串里写一份头两行就把切口挪到那里，它与真测试模块之间的代码哪一条扫源码的用例都看不见
+    /// （第 10z 批第二十三块，攻方 K4-2；崩溃放量那一份是 `crash_amplification.rs` 的 `text_outside_the_tests_module`）。
+    ///
+    /// # Panics
+    /// 代码里测试模块的头两行不是恰好一处。
+    fn code_before_the_tests_module(source: &str) -> &str {
+        let opening_tokens = rust_tokens_without_comments_and_literals(TESTS_MODULE_OPENING);
+        let tokens = rust_tokens_with_their_offsets(source);
+        let mut depth = 0_usize;
+        let mut openings = Vec::new();
+        for (index, (offset, token)) in tokens.iter().enumerate() {
+            if depth == 0
+                && tokens[index..]
+                    .iter()
+                    .map(|(_offset, later_token)| *later_token)
+                    .take(opening_tokens.len())
+                    .eq(opening_tokens.iter().copied())
+            {
+                openings.push(*offset);
+            }
+            if *token == "\u{7b}" {
+                depth += 1;
+            } else if *token == "\u{7d}" {
+                depth = depth.checked_sub(1).expect("右花括号不多于左花括号");
+            }
+        }
+        assert_eq!(
+            openings.len(),
+            1,
+            "代码里测试模块的头两行恰好一处（注释、字符串里的不算），读到的字节位置：{openings:?}"
+        );
+        &source[..openings[0]]
+    }
+
     /// 这份源码（测试那一段之外）里卡池模块（`mod card_pool`）的词，各带它在模块里第几层花括号里：从模块的左花括号之后
     /// 到与它配对的右花括号之前，模块的条目在第 0 层（左花括号算它外面那一层，右花括号算它收回到的那一层）。
     fn card_pool_module_tokens_with_their_brace_depth(code: &str) -> Vec<(usize, &str)> {
@@ -2709,6 +2764,86 @@ mod tests {
         ),
     ];
 
+    /// 钉住那两段（借之前那一步、借据丢掉时那一步）的签名，按词。
+    const SIGNATURES_OF_THE_PINNED_BODIES: [&[&str]; 2] = [
+        &["fn", "take_in_the_devices_found_idle"],
+        &["impl", "Drop", "for", "GpuCardLease"],
+    ];
+
+    /// 卡池模块的词（`module_tokens`，各带花括号层数）里第 `index` 个词所在的作用域罩不罩得到钉住那两段：在模块这一层（第 0 层）的罩得到；
+    /// 在花括号里的（函数体、块）只罩它那一对花括号里，那一对花括号里有钉住那两段的签名才罩得到。
+    fn does_the_scope_of_the_token_reach_the_pinned_bodies(
+        module_tokens: &[(usize, &str)],
+        index: usize,
+    ) -> bool {
+        let Some(enclosing_depth) = module_tokens[index].0.checked_sub(1) else {
+            return true;
+        };
+        let opening = module_tokens[..index]
+            .iter()
+            .rposition(|(depth, token)| *depth == enclosing_depth && *token == "\u{7b}")
+            .expect("第 1 层往里的词前面有开这一层的左花括号");
+        let closing = index
+            + module_tokens[index..]
+                .iter()
+                .position(|(depth, token)| *depth == enclosing_depth && *token == "\u{7d}")
+                .expect("第 1 层往里的词后面有收这一层的右花括号");
+        let block: Vec<&str> = module_tokens[opening..=closing]
+            .iter()
+            .map(|(_depth, token)| *token)
+            .collect();
+        SIGNATURES_OF_THE_PINNED_BODIES.iter().any(|signature| {
+            block
+                .windows(signature.len())
+                .any(|window| window == *signature)
+        })
+    }
+
+    /// `tokens` 里第 `index` 个词（一个条目的头一个词）前面挂着的属性，各取 `#[` 之后的头一个词（`cfg`、`cfg_attr`、`allow`……），
+    /// 照源码倒着；属性与条目之间的 `pub`、`pub(…)` 跳过。
+    fn names_of_the_attributes_right_before<'token>(
+        tokens: &[&'token str],
+        index: usize,
+    ) -> Vec<&'token str> {
+        let mut item_start = index;
+        if item_start > 0 && tokens[item_start - 1] == "pub" {
+            item_start -= 1;
+        } else if item_start > 1 && tokens[item_start - 1] == ")" {
+            let visibility_opening = tokens[..item_start - 1]
+                .iter()
+                .rposition(|token| *token == "(");
+            if let Some(parenthesis) = visibility_opening.filter(|parenthesis| {
+                parenthesis.checked_sub(1).map(|before| tokens[before]) == Some("pub")
+            }) {
+                item_start = parenthesis - 1;
+            }
+        }
+        let mut names = Vec::new();
+        while item_start > 0 && tokens[item_start - 1] == "]" {
+            let mut unclosed_brackets = 0_usize;
+            let mut attribute_opening = None;
+            for token_index in (0..item_start).rev() {
+                if tokens[token_index] == "]" {
+                    unclosed_brackets += 1;
+                } else if tokens[token_index] == "[" {
+                    unclosed_brackets -= 1;
+                    if unclosed_brackets == 0 {
+                        attribute_opening = Some(token_index);
+                        break;
+                    }
+                }
+            }
+            let Some(bracket) = attribute_opening
+                .filter(|bracket| bracket.checked_sub(1).map(|before| tokens[before]) == Some("#"))
+            else {
+                break;
+            };
+            names.extend(tokens.get(bracket + 1).copied());
+            item_start = bracket - 1;
+        }
+        names
+    }
+
     /// `code`（这份源码测试那一段之外，或用例造的一段）的卡池模块里，能把钉住那两段按路径首段解析的 `std`、`wgpu` 与按名字调的
     /// 那几个函数换掉的写法，一处一句；一处没有交空。判法与名单在 `the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call`。
     fn ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(
@@ -2730,12 +2865,18 @@ mod tests {
                 );
             }
         }
-        for pair in module_tokens.windows(2) {
+        for (index, pair) in module_tokens.windows(2).enumerate() {
+            // 条目这一层的宏前面是上一个条目的收尾、属性的右方括号，或者模块的开头；跟在 = 后面、属性方括号里的是表达式宏
+            // （`const _: () = assert!(…);`、`#[doc = include_str!(…)]`），展开不出条目（第 10z 批第二十三块，正推 K4-2 L7–L9）
+            let token_before = index
+                .checked_sub(1)
+                .map(|index_before| module_tokens[index_before].1);
             if pair[0].0 == 0
                 && pair[1].1 == "!"
                 && pair[0]
                     .1
                     .starts_with(|character: char| character.is_alphanumeric() || character == '_')
+                && token_before.is_none_or(|token| matches!(token, ";" | "\u{7d}" | "]"))
             {
                 found.push(format!(
                     "卡池模块的条目这一层调宏或定义宏 {}!：宏展开出来的条目也能叫 std、wgpu",
@@ -2743,18 +2884,32 @@ mod tests {
                 ));
             }
         }
-        let use_statements: Vec<&[&str]> = tokens
+        // 一条 use 后面不会紧跟 <：`impl Trait + use<'a>` 里的 use 是精确捕获（第 10z 批第二十三块，正推 K4-2 L10）；
+        // 块里的 use 只在那一对花括号里有效，块里没有钉住那两段的签名就罩不到它们（攻方 K4-2 g2）
+        let use_indices: Vec<usize> = tokens
             .iter()
             .enumerate()
-            .filter(|(_index, token)| **token == "use")
-            .map(|(use_index, _token)| {
-                let length = tokens[use_index..]
-                    .iter()
-                    .position(|token| *token == ";")
-                    .expect("use 以分号收尾")
-                    + 1;
-                &tokens[use_index..use_index + length]
+            .filter(|(use_index, token)| {
+                **token == "use"
+                    && tokens.get(use_index + 1) != Some(&"<")
+                    && does_the_scope_of_the_token_reach_the_pinned_bodies(
+                        &module_tokens,
+                        *use_index,
+                    )
             })
+            .map(|(use_index, _token)| use_index)
+            .collect();
+        let use_statement_at = |use_index: usize| -> &[&str] {
+            let length = tokens[use_index..]
+                .iter()
+                .position(|token| *token == ";")
+                .expect("use 以分号收尾")
+                + 1;
+            &tokens[use_index..use_index + length]
+        };
+        let use_statements: Vec<&[&str]> = use_indices
+            .iter()
+            .map(|use_index| use_statement_at(*use_index))
             .collect();
         let mut brought_in = Vec::new();
         for statement in &use_statements {
@@ -2767,13 +2922,39 @@ mod tests {
             );
             brought_in.extend(brought_in_by_the_statement);
         }
+        for use_index in &use_indices {
+            let attributes = names_of_the_attributes_right_before(&tokens, *use_index);
+            let Some(conditional_attribute) = attributes
+                .iter()
+                .find(|attribute| matches!(**attribute, "cfg" | "cfg_attr"))
+            else {
+                continue;
+            };
+            let (brought_in_by_the_statement, _end_of_the_tree) =
+                brought_in_by_a_use_tree(use_statement_at(*use_index), 1, "");
+            for brought_in_by_a_use in &brought_in_by_the_statement {
+                match brought_in_by_a_use {
+                    BroughtInByAUse::Name { source: _, name }
+                        if FUNCTIONS_THE_PINNED_BODIES_CALL_BY_NAME
+                            .iter()
+                            .any(|(_prefix, function)| function == name) =>
+                    {
+                        found.push(format!(
+                            "卡池模块里挂 #[{conditional_attribute}] 的 use 引进钉住那两段按名字调的 {name}：编译时这条 use 可以被去掉，按名字调的就落到模块里另起的同名函数上"
+                        ));
+                    }
+                    BroughtInByAUse::Name { .. } | BroughtInByAUse::Glob { .. } => {}
+                }
+            }
+        }
         for brought_in_by_a_use in &brought_in {
             match brought_in_by_a_use {
                 BroughtInByAUse::Glob { source } => found.push(format!(
                     "卡池模块里通配 use {source}::*：引进来的名字（叫 std、wgpu 的模块也算）扫这一段看不见"
                 )),
                 BroughtInByAUse::Name { source, name } => {
-                    if matches!(name.as_str(), "std" | "wgpu") {
+                    // use wgpu; 引的就是外部 crate 自己，遮不住它（攻方 K4-2 g1）
+                    if matches!(name.as_str(), "std" | "wgpu") && source != name {
                         found.push(format!(
                             "卡池模块经 use 引进叫 {name} 的名字（引的是 {source}），遮住外部 crate"
                         ));
@@ -2813,6 +2994,15 @@ mod tests {
     /// `the_card_pool_scan_accepts_renames_nested_groups_and_self_that_bring_in_neither_std_nor_wgpu`、
     /// `the_card_pool_scan_finds_std_wgpu_or_a_pinned_function_brought_in_under_a_rename_a_nested_group_or_self`、
     /// `the_card_pool_scan_accepts_an_extern_block_and_still_finds_extern_crate`）。
+    /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4）：挂 `#[cfg]`、`#[cfg_attr]` 的 `use` 引进钉住那两段调的函数判红——编译时
+    /// 那条 `use` 去掉，按名字调的就落到模块里另起的同名函数上（正推 K4-1 B12、B13，
+    /// `the_card_pool_scan_finds_a_pinned_function_brought_in_by_a_use_under_a_cfg`）；换不掉那几个名字的几样判绿
+    /// （`the_card_pool_scan_accepts_writings_that_cannot_replace_what_the_pinned_bodies_call`）：`use wgpu;` 引的就是外部 crate 自己；
+    /// 块里的 `use`（函数体里 `use core as std;`）只罩那一对花括号，里面没有钉住那两段的签名就不拒，有就照拒
+    /// （`the_card_pool_scan_finds_a_rename_in_a_block_holding_a_pinned_body_and_an_item_level_macro_under_an_attribute`）；
+    /// 属性与常量、静态量初始值里的表达式宏（`include_str!`、`assert!`、`concat!`）展开不出条目；`impl … + use<'a>` 的 `use` 是精确捕获；
+    /// 分词认 `cr#"…"#`、`c"…"`（与崩溃放量那一份认法相同），拼成关键字的原始标识符（`r#mod`）是名字。测试模块只从代码里那一处头两行切
+    /// （`the_card_pool_scans_cut_the_tests_module_at_its_opening_in_code_not_at_a_copy_in_a_comment_or_a_string`）。
     ///
     /// 故意拒的写法（换不换得掉那几个名字，扫这一个模块的源码判不出），一种一条，各写为什么
     /// （`the_card_pool_scan_refuses_globs_item_level_macros_and_inner_modules_even_when_they_name_nothing_pinned`）：
@@ -2831,10 +3021,7 @@ mod tests {
     fn the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call()
     {
         let source = include_str!("gpu_unit_checks.rs");
-        let code = source
-            .split(TESTS_MODULE_OPENING)
-            .next()
-            .expect("split 至少交一段");
+        let code = code_before_the_tests_module(source);
         assert_eq!(
             ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(code),
             Vec::<String>::new(),
@@ -2859,9 +3046,16 @@ mod tests {
 
     /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K5，正推 `K5_const_ne`、`K5_const_ne_ident`）：条目这一层的
     /// `const 名: bool = 甲 != 乙;` 是比较，不是调宏 `甲!`；条目这一层真调宏（`thread_local!`）照旧拒。不要卡。
+    /// 第 10z 批第二十三块：条目层的宏另看前一个词（跟在 `=` 后面的不算）之后，`!=` 拆成两个词也不再被认成调宏，两道各自拦得住这一形；
+    /// 分词把 `!=` 当一个词这一道另由头一句断言钉住。
     #[test]
     fn the_card_pool_scan_reads_an_item_level_not_equal_as_a_comparison_and_still_finds_an_item_level_macro(
     ) {
+        assert_eq!(
+            rust_tokens_without_comments_and_literals("FIRST_LIMIT!=SECOND_LIMIT"),
+            ["FIRST_LIMIT", "!=", "SECOND_LIMIT"],
+            "分词把不等号当一个词：拆成 ! 与 = 时 甲 ! 读起来像调宏"
+        );
         let comparisons = card_pool_module_with_the_pinned_uses_and(concat!(
             "    const FIRST_LIMIT: u8 = 1;\n",
             "    const SECOND_LIMIT: u8 = 2;\n",
@@ -3003,6 +3197,159 @@ mod tests {
         }
     }
 
+    /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4，攻方 K4-2）：卡池模块之前写一份测试模块的头两行在块注释里（或跨行的字符串里），
+    /// 扫源码的用例照旧切到真测试模块的头两行：切出来的那一段只多了写进去的那几行。不要卡。
+    #[test]
+    fn the_card_pool_scans_cut_the_tests_module_at_its_opening_in_code_not_at_a_copy_in_a_comment_or_a_string(
+    ) {
+        let source = include_str!("gpu_unit_checks.rs");
+        assert_eq!(
+            source.matches(TESTS_MODULE_OPENING).count(),
+            1,
+            "这份源码里测试模块的头两行原文只有代码里那一处"
+        );
+        let code = code_before_the_tests_module(source);
+        assert!(
+            source[code.len()..].starts_with(TESTS_MODULE_OPENING),
+            "切口在真测试模块的头两行"
+        );
+        let card_pool_opening = "    mod card_pool \u{7b}\n";
+        assert_eq!(
+            code.matches(card_pool_opening).count(),
+            1,
+            "测试那一段之外恰好一处卡池模块"
+        );
+        for (form, copy_of_the_opening) in [
+            (
+                "块注释",
+                format!("    /*\n{TESTS_MODULE_OPENING}\n    */\n"),
+            ),
+            (
+                "跨行的字符串",
+                format!(
+                    "    pub const A_COPY_OF_THE_OPENING: &str = \"\n{TESTS_MODULE_OPENING}\n\";\n"
+                ),
+            ),
+        ] {
+            let with_a_copy = source.replacen(
+                card_pool_opening,
+                &format!("{copy_of_the_opening}{card_pool_opening}"),
+                1,
+            );
+            assert_eq!(
+                code_before_the_tests_module(&with_a_copy).len(),
+                code.len() + copy_of_the_opening.len(),
+                "卡池模块之前{form}里写一份测试模块的头两行，切口不挪"
+            );
+        }
+    }
+
+    /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4，正推 K4-1 B12、B13：钉住那两段调的 `device_found_idle_by_a_poll` 改由一条挂
+    /// `#[cfg(any())]` 的 `use` 引进、模块里另起一个同名函数，编译时那条 `use` 去掉、按名字调的是另起的那一个，扫源码照绿）：挂 `#[cfg]`、
+    /// 展开成 `cfg` 的 `#[cfg_attr]`、`pub(crate)` 之前的 `#[cfg]`，都红。不要卡。
+    #[test]
+    fn the_card_pool_scan_finds_a_pinned_function_brought_in_by_a_use_under_a_cfg() {
+        for (form, conditional_use) in [
+            (
+                "cfg",
+                "    #[cfg(any())]\n    use super::super::device_found_idle_by_a_poll;\n",
+            ),
+            (
+                "展开成 cfg 的 cfg_attr",
+                "    #[cfg_attr(all(), cfg(any()))]\n    use super::super::device_found_idle_by_a_poll;\n",
+            ),
+            (
+                "pub(crate) 之前的 cfg，另隔一条别的属性",
+                "    #[cfg(any())]\n    #[allow(unused_imports, reason = \"造的\")]\n    pub(crate) use super::super::device_found_idle_by_a_poll;\n",
+            ),
+        ] {
+            let module = card_pool_module_with_the_pinned_uses_and(
+                "    fn device_found_idle_by_a_poll(_outcome: &u8) -> bool \u{7b} true \u{7d}",
+            )
+            .replacen("device_found_idle_by_a_poll, ", "", 1)
+            .replacen(
+                "    use singlefs_harness",
+                &format!("{conditional_use}    use singlefs_harness"),
+                1,
+            );
+            let found = ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&module);
+            assert!(
+                found.iter().any(|finding| finding
+                    .contains("的 use 引进钉住那两段按名字调的 device_found_idle_by_a_poll")),
+                "挂 {form} 的 use 引进钉住那两段调的函数，红：{found:#?}\n{module}"
+            );
+        }
+    }
+
+    /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4，攻方 K4-2 g1、g2，正推 K4-2 L7–L10、L13b、L14）：换不掉钉住那两段用的名字的写法，
+    /// 一样一个模块，都判绿。不要卡。
+    #[test]
+    fn the_card_pool_scan_accepts_writings_that_cannot_replace_what_the_pinned_bodies_call() {
+        for (form, items) in [
+            ("引外部 crate 自己", "    use wgpu;\n    use ::std;"),
+            (
+                "别的函数体里块作用域的改名",
+                "    fn a_helper_with_its_own_std() -> u8 \u{7b} use core as std; std::convert::identity(0) \u{7d}",
+            ),
+            (
+                "属性里的表达式宏",
+                "    #[doc = include_str!(\"card_pool.md\")]\n    const POOL_SIZE: u8 = 2;",
+            ),
+            ("常量初始值里的表达式宏", "    const _: () = assert!(1 < 2);"),
+            (
+                "静态量初始值里的表达式宏",
+                "    static JOINED_NAME: &str = concat!(\"card\", \"pool\");",
+            ),
+            (
+                "精确捕获",
+                "    fn borrowed_names<'pool>(names: &'pool [u8]) -> impl Iterator<Item = &'pool u8> + use<'pool> \u{7b} names.iter() \u{7d}",
+            ),
+            (
+                "C 风格原始字符串里有引号与花括号",
+                "    const QUOTED: &core::ffi::CStr = cr#\"say \"\u{7b}\" here\"#;\n    const PLAIN: &core::ffi::CStr = c\"say \u{7d} here\";",
+            ),
+            (
+                "拼成关键字的原始标识符",
+                "    fn r#mod() -> u8 \u{7b} 0 \u{7d}\n    fn r#use(r#extern: u8) -> u8 \u{7b} r#extern \u{7d}",
+            ),
+            (
+                "挂 cfg 的 use 引的不是钉住那两段调的函数",
+                "    #[cfg(any())]\n    use std::sync::Arc as SharedArcWhenConfigured;",
+            ),
+        ] {
+            let module = card_pool_module_with_the_pinned_uses_and(items);
+            assert_eq!(
+                ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&module),
+                Vec::<String>::new(),
+                "{form} 换不掉钉住那两段用的名字，判绿：{module}"
+            );
+        }
+    }
+
+    /// 第 10z 批第二十三块（上一条的反面）：块里的改名所在那一对花括号里有钉住那两段的签名（`impl Drop for GpuCardLease` 挪进函数体）照拒；
+    /// 属性下面的条目层宏照拒。不要卡。
+    #[test]
+    fn the_card_pool_scan_finds_a_rename_in_a_block_holding_a_pinned_body_and_an_item_level_macro_under_an_attribute(
+    ) {
+        for (form, items) in [
+            (
+                "块里改名、同一块里有借据丢掉时那一步",
+                "    fn holding_the_lease_drop() \u{7b}\n        use core as std;\n        impl Drop for GpuCardLease \u{7b} fn drop(&mut self) \u{7b}\u{7d} \u{7d}\n    \u{7d}",
+            ),
+            (
+                "属性下面的条目层宏",
+                "    #[allow(unused, reason = \"造的\")]\n    thread_local! \u{7b} static COUNT: std::cell::Cell<u8> = std::cell::Cell::new(0); \u{7d}",
+            ),
+        ] {
+            let module = card_pool_module_with_the_pinned_uses_and(items);
+            assert_ne!(
+                ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&module),
+                Vec::<String>::new(),
+                "{form} 拒：{module}"
+            );
+        }
+    }
+
     /// 第 10z 批第十八块（第 10z 批第十四、十五块代码轮正推 K4：钉住的那一句里 `started.device()` 交的是哪一台不在整句里，
     /// `started.device_condition()` 同理）：一张卡交出去的设备与设备的情况就是它自己那两格。要真卡上起好的设备才造得出 `GpuCard`，
     /// 换不成不要卡的行为用例，这里扫这份源码（测试那一段之外、去掉注释行与空白）：两个方法各恰好一处，方法体整个是那一格。不要卡。
@@ -3013,10 +3360,7 @@ mod tests {
     #[test]
     fn a_card_hands_out_its_own_device_and_its_own_device_condition() {
         let source = include_str!("gpu_unit_checks.rs");
-        let code = source
-            .split(TESTS_MODULE_OPENING)
-            .next()
-            .expect("split 至少交一段");
+        let code = code_before_the_tests_module(source);
         for (signature, expected_body) in [
             (
                 "pub fn device(&self) -> &wgpu::Device \u{7b}",
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
index 77162ee..46d0e5f 100644
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -4625,26 +4625,9 @@ mod tests {
     /// [`ENVIRONMENT_VARIABLES_READ_BY_A_RUN`] 里：用例的环境守卫照这张表清、还原，漏一个就会读到调用方留下的值。
     #[test]
     fn every_environment_variable_this_file_reads_by_name_is_in_the_list_a_run_reads() {
-        let source = std::fs::read_to_string(
-            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/crash_amplification.rs"),
-        )
-        .expect("读得了这份源文件");
-        let tests_start = source
-            .find(TESTS_MODULE_OPENING)
-            .expect("这份文件有测试模块");
-        let tests_end = tests_start
-            + source[tests_start..]
-                .find(TOP_LEVEL_CLOSING_BRACE_LINE)
-                .expect("测试模块在顶格的右括号处收尾");
-        let outside_the_tests = format!("{}{}", &source[..tests_start], &source[tests_end..]);
-        let mut names_read = Vec::new();
-        for opening in ["env::var(\"", "env::var_os(\""] {
-            for (position, _) in outside_the_tests.match_indices(opening) {
-                let rest = &outside_the_tests[position + opening.len()..];
-                let name = &rest[..rest.find('"').expect("变量名的串收得了尾")];
-                names_read.push(name.to_string());
-            }
-        }
+        let names_read = environment_variables_read_by_name_outside_the_tests(include_str!(
+            "crash_amplification.rs"
+        ));
         assert!(
             names_read.len() >= 10,
             "按名字读的环境变量少得不对，扫法坏了：{names_read:?}"
@@ -4657,21 +4640,69 @@ mod tests {
         }
     }
 
-    /// 这份源文件测试模块之外的代码行（注释行不算）。
-    fn code_lines_outside_the_tests() -> Vec<String> {
-        code_lines_outside_the_tests_of(include_str!("crash_amplification.rs"))
+    /// `source`（这份源文件，或用例在它上面改过的一份）测试模块之外按名字读的环境变量（`env::var("…")`、`env::var_os("…")`），照源码次序。
+    fn environment_variables_read_by_name_outside_the_tests(source: &str) -> Vec<String> {
+        let outside_the_tests = text_outside_the_tests_module(source);
+        let mut names_read = Vec::new();
+        for opening in ["env::var(\"", "env::var_os(\""] {
+            for (position, _) in outside_the_tests.match_indices(opening) {
+                let rest = &outside_the_tests[position + opening.len()..];
+                let name = &rest[..rest.find('"').expect("变量名的串收得了尾")];
+                names_read.push(name.to_string());
+            }
+        }
+        names_read
     }
 
-    /// `source`（这份源文件，或用例在它上面改过的一份）测试模块之外的代码行（注释行不算）。
-    fn code_lines_outside_the_tests_of(source: &str) -> Vec<String> {
-        let tests_start = source
-            .find(TESTS_MODULE_OPENING)
-            .expect("这份文件有测试模块");
-        let tests_end = tests_start
-            + source[tests_start..]
-                .find(TOP_LEVEL_CLOSING_BRACE_LINE)
-                .expect("测试模块在顶格的右括号处收尾");
-        format!("{}{}", &source[..tests_start], &source[tests_end..])
+    /// `source`（这份源文件，或用例在它上面改过的一份）去掉测试模块之后的文本：从测试模块的头两行起，到与它配对的顶格右括号那一行止，
+    /// 连同这两处一起去掉。头两行只认代码里的：注释、字符串里写的一份不算（照原文找第一处时，块注释里写一份就把切口挪到注释那里，
+    /// 注释与真测试模块之间的代码被当成测试切掉，第 10z 批第二十三块）。
+    ///
+    /// # Panics
+    /// 代码里（花括号第 0 层、注释与字面量之外）测试模块的头两行不是恰好一处，或它之后没有收尾的顶格右括号行。
+    fn text_outside_the_tests_module(source: &str) -> String {
+        let lines: Vec<&str> = source.split('\n').collect();
+        let read_lines = lines_read_past_comments_and_literals(&lines);
+        let (attribute_line, module_line) = TESTS_MODULE_OPENING
+            .split_once('\n')
+            .expect("测试模块的头两行是两行");
+        let openings: Vec<usize> = read_lines
+            .windows(2)
+            .enumerate()
+            .filter(|(_line_index, pair)| {
+                pair[0].brace_depth_at_the_start == 0
+                    && pair[0].without_comments_and_literals.trim_end() == attribute_line
+                    && pair[1].without_comments_and_literals.trim_end() == module_line
+            })
+            .map(|(line_index, _pair)| line_index)
+            .collect();
+        assert_eq!(
+            openings.len(),
+            1,
+            "代码里测试模块的头两行恰好一处（注释、字符串里的不算），读到的行号（从 0 数）：{openings:?}"
+        );
+        let opening = openings[0];
+        let closing = opening
+            + 1
+            + read_lines[opening + 1..]
+                .iter()
+                .position(|read_line| {
+                    read_line.brace_depth_at_the_start == 1
+                        && read_line.without_comments_and_literals.trim_end()
+                            == TOP_LEVEL_CLOSING_BRACE_LINE.trim()
+                })
+                .expect("测试模块在顶格的右括号行收尾");
+        lines[..opening]
+            .iter()
+            .chain(&lines[closing + 1..])
+            .copied()
+            .collect::<Vec<&str>>()
+            .join("\n")
+    }
+
+    /// 这份源文件测试模块之外的代码行（注释行不算）。
+    fn code_lines_outside_the_tests() -> Vec<String> {
+        text_outside_the_tests_module(include_str!("crash_amplification.rs"))
             .lines()
             .filter(|line| !line.trim_start().starts_with("//"))
             .map(str::to_string)
@@ -4884,11 +4915,16 @@ mod tests {
         layer: &'code str,
         /// 类型前面 `名字:` 的那个名字（前面不是 `名字:` 的交空串）。
         parameter: &'code str,
+        /// 参数名前面紧挨着的是不是属性的右方括号（`#[cfg(any())] stop_is_requested: …`）。
+        is_the_parameter_under_an_attribute: bool,
+        /// 类型起头（`&`）的位置。
+        start_of_the_type: usize,
         /// 类型收尾之后的位置。
         end_of_the_type: usize,
     }
 
-    /// `code` 里每一处叫停类型，照源码次序：类型里的空白随意、可以跨行，参数与别的参数挤在签名那一行、参数换了名字的都找得到。
+    /// `code`（注释与字面量已换成空格）里每一处叫停类型，照源码次序：类型里的空白随意、可以跨行，参数与别的参数挤在签名那一行、
+    /// 参数换了名字的都找得到。
     fn places_taking_the_stop_request(code: &str) -> Vec<PlaceTakingTheStopRequest<'_>> {
         let is_identifier_character =
             |character: char| character.is_alphanumeric() || character == '_';
@@ -4907,12 +4943,18 @@ mod tests {
                         STOP_REQUEST_TYPE_WITHOUT_SPACES,
                     )?;
                 let before_the_type = code[..ampersand].trim_end();
-                let parameter = before_the_type
+                let before_the_colon = before_the_type
                     .strip_suffix(':')
                     .filter(|before_the_colon| !before_the_colon.ends_with(':'))
-                    .map_or("", |before_the_colon| {
-                        let before_the_name = before_the_colon.trim_end();
-                        &before_the_name[identifier_at_the_end_of(before_the_name)..]
+                    .map(str::trim_end);
+                let parameter = before_the_colon.map_or("", |before_the_name| {
+                    &before_the_name[identifier_at_the_end_of(before_the_name)..]
+                });
+                let is_the_parameter_under_an_attribute =
+                    before_the_colon.is_some_and(|before_the_name| {
+                        before_the_name[..identifier_at_the_end_of(before_the_name)]
+                            .trim_end()
+                            .ends_with(']')
                     });
                 let layer = whole_word_positions(&code[..ampersand], "fn")
                     .last()
@@ -4926,12 +4968,78 @@ mod tests {
                 Some(PlaceTakingTheStopRequest {
                     layer,
                     parameter,
+                    is_the_parameter_under_an_attribute,
+                    start_of_the_type: ampersand,
                     end_of_the_type,
                 })
             })
             .collect()
     }
 
+    /// 叫停那种闭包的形（不问参数、交 `bool`），去掉空白；`FnMut`、`FnOnce` 与函数指针 `fn() -> bool` 也算。
+    const STOP_REQUEST_SHAPES_WITHOUT_SPACES: [&str; 4] = [
+        "Fn()->bool",
+        "FnMut()->bool",
+        "FnOnce()->bool",
+        "fn()->bool",
+    ];
+
+    /// `code`（注释与字面量已换成空格）里写着叫停那种闭包的形、却不在 `places` 哪一处钉住的类型里的地方，一处一句（往前最近的 `fn`
+    /// 后面那个名字连同那一行）：叫停写成 `&(dyn Fn() -> bool + Sync)` 之外的类型（多加 `Send`、`'static`，`&mut`、`&'static`，
+    /// `Box<dyn …>`，`dyn Sync + Fn() -> bool`，`&dyn Fn() -> bool`，`impl`、泛型、这份源文件里的类型别名）收叫停的层，按钉住的类型找不到，
+    /// 照这个形找得到。
+    fn stop_request_shapes_outside_the_pinned_type(
+        code: &str,
+        places: &[PlaceTakingTheStopRequest<'_>],
+    ) -> Vec<String> {
+        let is_identifier_character =
+            |character: char| character.is_alphanumeric() || character == '_';
+        let mut found = Vec::new();
+        for shape in STOP_REQUEST_SHAPES_WITHOUT_SPACES {
+            let first_word = &shape[..shape.find('(').expect("形里有左括号")];
+            for position in whole_word_positions(code, first_word) {
+                let Some(shape_length) =
+                    end_of_a_prefix_ignoring_whitespace(&code[position..], shape)
+                else {
+                    continue;
+                };
+                if code[position + shape_length..]
+                    .chars()
+                    .next()
+                    .is_some_and(is_identifier_character)
+                {
+                    continue;
+                }
+                if places.iter().any(|place| {
+                    (place.start_of_the_type..place.end_of_the_type).contains(&position)
+                }) {
+                    continue;
+                }
+                let layer =
+                    whole_word_positions(&code[..position], "fn")
+                        .last()
+                        .map_or("", |fn_keyword| {
+                            let after_the_keyword = code[fn_keyword + 2..position].trim_start();
+                            let name_length = after_the_keyword
+                                .find(|character: char| !is_identifier_character(character))
+                                .unwrap_or(after_the_keyword.len());
+                            &after_the_keyword[..name_length]
+                        });
+                let line_start = code[..position]
+                    .rfind('\n')
+                    .map_or(0, |newline| newline + 1);
+                let line_end = code[position..]
+                    .find('\n')
+                    .map_or(code.len(), |newline| position + newline);
+                found.push(format!(
+                    "{layer} 往后有一处叫停写成钉住的 &(dyn Fn() -> bool + Sync) 之外的类型，按钉住的类型找不到这一层：{}",
+                    code[line_start..line_end].trim()
+                ));
+            }
+        }
+        found
+    }
+
     /// 没有行为用例的那几层，扫源码钉住它们怎么用收到的叫停（`every_layer_handed_the_stop_request_hands_it_on_or_asks_it`）。
     enum StopRequestUseThatIsPinned {
         /// 函数体短：参数行之后到函数收尾（返回类型与函数体，去掉注释行与空白）整个等于这一句。
@@ -4960,24 +5068,63 @@ mod tests {
         },
     }
 
+    /// 读过的一个字算源码里的哪一类（`lines_read_past_comments_and_literals`）。
+    #[derive(Clone, Copy)]
+    enum KindOfACharacterInTheSource {
+        Code,
+        /// 行注释、块注释里的字，连同 `//`、`/*`、`*/`。
+        Comment,
+        /// 字符串（普通的、原始的）与字符字面量里的字，连同引号与原始字符串的 `r`、`br`、`cr` 和 `#`。
+        Literal,
+    }
+
+    /// 一行读过注释与字面量之后的样子（`lines_read_past_comments_and_literals`）。
+    struct LineReadPastCommentsAndLiterals {
+        /// 这一行起头处在第几层花括号里。
+        brace_depth_at_the_start: usize,
+        /// 注释里的字换成空格（一个字换成与它的 UTF-8 字节数一样多的空格，字节位置不变），字面量照原样。
+        without_comments: String,
+        /// 注释与字面量里的字都换成空格，字节位置不变：扫源码的用例照它找名字、数 `let`、数花括号，注释与字符串里写的不算。
+        without_comments_and_literals: String,
+    }
+
     /// `lines` 每一行起头处在第几层花括号里（第一行起头算第 0 层）：字符串（普通的、原始的）、字符字面量、行注释与块注释（可以嵌套）里的
     /// 花括号不算，字符串、块注释跨行时照跨着数。
     ///
     /// # Panics
     /// 右花括号多于左花括号。
     fn brace_depth_at_the_start_of_each_line(lines: &[&str]) -> Vec<usize> {
+        lines_read_past_comments_and_literals(lines)
+            .iter()
+            .map(|read_line| read_line.brace_depth_at_the_start)
+            .collect()
+    }
+
+    /// `lines` 逐行读过注释与字面量：每一行起头处的花括号层数，连同去掉注释、去掉注释与字面量的两份（字节位置与原行相同）。
+    /// 字符串（普通的、原始的 `r"…"`、`r#"…"#`、`br`、`cr`）、字符字面量、行注释与块注释（可以嵌套）里的花括号不算，
+    /// 字符串、块注释跨行时照跨着读。与卡池扫源码那一份分词（`gpu_unit_checks.rs` 的 `rust_tokens_without_comments_and_literals`）
+    /// 认原始字符串的写法相同：`r`、`br`、`cr` 后面紧跟若干个 `#` 与一个 `"`。
+    ///
+    /// # Panics
+    /// 右花括号多于左花括号。
+    fn lines_read_past_comments_and_literals(
+        lines: &[&str],
+    ) -> Vec<LineReadPastCommentsAndLiterals> {
         let is_identifier_character =
             |character: char| character.is_alphanumeric() || character == '_';
         let mut depth = 0_usize;
         let mut place = PlaceInTheSource::Code;
-        let mut depths = Vec::with_capacity(lines.len());
+        let mut read_lines = Vec::with_capacity(lines.len());
         for line in lines {
-            depths.push(depth);
+            let brace_depth_at_the_start = depth;
             let characters: Vec<char> = line.chars().collect();
+            let mut kinds = vec![KindOfACharacterInTheSource::Code; characters.len()];
             let mut index = 0;
             while index < characters.len() {
                 let character = characters[index];
                 let next_character = characters.get(index + 1).copied();
+                let read_from = index;
+                let place_before = place;
                 match place {
                     PlaceInTheSource::InsideAString => {
                         if character == '\\' {
@@ -5028,6 +5175,7 @@ mod tests {
                                 index += 2;
                             }
                         } else if character == '/' && next_character == Some('/') {
+                            kinds[index..].fill(KindOfACharacterInTheSource::Comment);
                             break;
                         } else if character == '/' && next_character == Some('*') {
                             index += 1;
@@ -5061,9 +5209,56 @@ mod tests {
                     }
                 }
                 index += 1;
+                let kind_read = match place_before {
+                    PlaceInTheSource::InsideAString | PlaceInTheSource::InsideARawString { .. } => {
+                        KindOfACharacterInTheSource::Literal
+                    }
+                    PlaceInTheSource::InsideABlockComment { .. } => {
+                        KindOfACharacterInTheSource::Comment
+                    }
+                    PlaceInTheSource::Code => match place {
+                        PlaceInTheSource::InsideAString
+                        | PlaceInTheSource::InsideARawString { .. } => {
+                            KindOfACharacterInTheSource::Literal
+                        }
+                        PlaceInTheSource::InsideABlockComment { .. } => {
+                            KindOfACharacterInTheSource::Comment
+                        }
+                        // 一次读过不止一个字的引号是整个字符字面量；生命周期只读过那一个引号
+                        PlaceInTheSource::Code if character == '\'' && index > read_from + 1 => {
+                            KindOfACharacterInTheSource::Literal
+                        }
+                        PlaceInTheSource::Code => KindOfACharacterInTheSource::Code,
+                    },
+                };
+                kinds[read_from..index.min(characters.len())].fill(kind_read);
             }
+            let mut without_comments = String::with_capacity(line.len());
+            let mut without_comments_and_literals = String::with_capacity(line.len());
+            for (character, kind) in characters.iter().zip(&kinds) {
+                let spaces = " ".repeat(character.len_utf8());
+                match kind {
+                    KindOfACharacterInTheSource::Code => {
+                        without_comments.push(*character);
+                        without_comments_and_literals.push(*character);
+                    }
+                    KindOfACharacterInTheSource::Comment => {
+                        without_comments.push_str(&spaces);
+                        without_comments_and_literals.push_str(&spaces);
+                    }
+                    KindOfACharacterInTheSource::Literal => {
+                        without_comments.push(*character);
+                        without_comments_and_literals.push_str(&spaces);
+                    }
+                }
+            }
+            read_lines.push(LineReadPastCommentsAndLiterals {
+                brace_depth_at_the_start,
+                without_comments,
+                without_comments_and_literals,
+            });
         }
-        depths
+        read_lines
     }
 
     /// 第 10z 批第十五块（第 10z 批第十二、十三块代码轮判决 K4：扫源码的用例被领块之前那一问写成 `if false && stop_is_requested()` 骗过，
@@ -5150,6 +5345,15 @@ mod tests {
     /// （`the_stop_request_scan_finds_the_remembered_stop_request_rebound_by_a_tuple_pattern_or_a_let_with_extra_spaces`）；
     /// 数花括号的层数认得原始字符串与块注释（`brace_depth_reads_past_raw_strings_block_comments_and_strings_spanning_lines`），
     /// 长的那一层里写原始字符串不再让这条用例当场红，`if false { /* } */` 也骗不过层数。
+    /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4）：测试模块只从代码里那一处头两行切（块注释里写一份头两行、藏住它与真测试模块
+    /// 之间的新层，攻方 K4-2 v2，`the_tests_module_is_cut_at_its_opening_in_code_not_at_a_copy_in_a_comment_or_a_string`）；
+    /// 找层、数 `let`、数用到叫停的行都在注释与字面量换成空格之后的代码上做，整句、整行比在去掉注释的代码上做——签名行尾注释里的 `fn`
+    /// 不再被当成层名（攻方 v4），行尾注释、字符串里的 `let stop_request` 与钉住那一行、短层函数体上的行尾注释不再判红
+    /// （`the_stop_request_scan_reads_layer_names_rebindings_and_pinned_lines_past_comments_and_strings`）；钉住那一行与叫停参数上挂属性
+    /// （`#[cfg(any())]` 编译时去掉那一句、那个参数，正推 K4-1 A01）判红（`the_stop_request_scan_finds_an_attribute_on_the_pinned_line_or_on_a_stop_parameter`）；
+    /// 叫停写成钉住之外的类型（多加 `Send`、`'static`，`&mut`、`&'static`，类型里夹注释，`Box<dyn …>`，`dyn Sync + Fn() -> bool`，
+    /// `&dyn Fn() -> bool`，`impl`、泛型、这份源文件里的类型别名，函数指针）的新层照 `Fn() -> bool` 这个形找到、判红
+    /// （`the_stop_request_scan_finds_new_layers_whose_stop_type_is_written_another_way`，正推 K4-2 C02–C12）。
     ///
     /// 拦不住的写法，一种一条：
     /// - 同一行原样交下去而下一层的函数体里不用（下一层由它自己那一条管；第 ① 段在卡上判那一层里 `stop_request` 往下怎么用由
@@ -5157,8 +5361,9 @@ mod tests {
     /// - 长的那一层：钉住那一行之前提早返回，或让它之后的路走不到；`stop_request` 经 `let` 之外的绑定（`match` 的臂、闭包参数、`for`）
     ///   在块里换成别的；
     /// - 被调的那一层自己改了（短的几层整句比钉的是调谁、怎么交，不是被调的函数里做什么）；harness 档的 `StopRequestRememberedInThisRun::new` 改了；
-    /// - 叫停换成别的类型写法（`&(dyn Sync + Fn() -> bool)`、`&dyn Fn() -> bool`、`impl Fn() -> bool`、泛型参数、类型别名）的新层，
-    ///   或在函数上面加 `#[cfg]` 让这样另写类型的同名函数编进来：按类型找只认这一种写法；
+    /// - 叫停的类型写成别的源文件、别的 crate 里定义的别名或 trait（`&crate::某处::叫停类型`），或换成不带 `Fn() -> bool` 字样的类型
+    ///   （`&StopRequestRememberedInThisRun`、自己的 trait）的新层：只扫这一份源文件、只认这个形；
+    /// - 在函数上面加 `#[cfg]`，让照上一条另写类型的同名函数编进来（同样写成钉住的类型或这份源文件里看得见的形的，读到的名单多一层、红）；
     /// - 长的那一层里宏展开出来的调用与绑定。
     #[test]
     fn every_layer_handed_the_stop_request_hands_it_on_or_asks_it() {
@@ -5230,19 +5435,45 @@ mod tests {
                 )),
             ),
         ];
-        let code = code_lines_outside_the_tests_of(source).join("\n") + "\n";
+        let outside_the_tests = text_outside_the_tests_module(source) + "\n";
+        let outside_the_tests_lines: Vec<&str> = outside_the_tests.split('\n').collect();
+        let read_lines = lines_read_past_comments_and_literals(&outside_the_tests_lines);
+        // 两份与 outside_the_tests 逐字节同位：找名字、数 let 与花括号照 code（注释与字面量都不算），整句、整行比照 without_comments
+        let without_comments = read_lines
+            .iter()
+            .map(|read_line| read_line.without_comments.as_str())
+            .collect::<Vec<&str>>()
+            .join("\n");
+        let code = read_lines
+            .iter()
+            .map(|read_line| read_line.without_comments_and_literals.as_str())
+            .collect::<Vec<&str>>()
+            .join("\n");
         let mut found = Vec::new();
         let mut layers_handed_the_stop_request = Vec::new();
-        for place in places_taking_the_stop_request(&code) {
+        let places = places_taking_the_stop_request(&code);
+        for place in &places {
             let PlaceTakingTheStopRequest {
                 layer,
                 parameter,
+                is_the_parameter_under_an_attribute,
+                start_of_the_type: _,
                 end_of_the_type,
-            } = place;
-            let after_the_type = &code[end_of_the_type..];
-            let rest_of_the_layer: Vec<&str> = after_the_type[..after_the_type
+            } = *place;
+            if is_the_parameter_under_an_attribute {
+                found.push(format!(
+                    "{layer} 的叫停参数 {parameter} 前面挂着属性（#[cfg(any())] 让参数编译时去掉，函数体里的名字就落到别处同名的条目上）"
+                ));
+            }
+            let layer_length = code[end_of_the_type..]
                 .find(TOP_LEVEL_CLOSING_BRACE_LINE)
-                .unwrap_or(after_the_type.len())]
+                .unwrap_or(code.len() - end_of_the_type);
+            let rest_of_the_layer: Vec<&str> = without_comments
+                [end_of_the_type..end_of_the_type + layer_length]
+                .lines()
+                .collect();
+            let rest_of_the_layer_code: Vec<&str> = code
+                [end_of_the_type..end_of_the_type + layer_length]
                 .lines()
                 .collect();
             match pinned_layers_without_a_behaviour_test
@@ -5273,11 +5504,11 @@ mod tests {
                     binding,
                     type_name,
                 }) => {
-                    let lines_using_it: Vec<(usize, &str)> = rest_of_the_layer
+                    let lines_using_it: Vec<(usize, &str)> = rest_of_the_layer_code
                         .iter()
                         .enumerate()
                         .filter(|(_offset, line)| whole_name_uses_of_the_stop_request(line) > 0)
-                        .map(|(offset, line)| (offset, line.trim()))
+                        .map(|(offset, _line)| (offset, rest_of_the_layer[offset].trim()))
                         .collect();
                     if lines_using_it
                         .iter()
@@ -5289,7 +5520,7 @@ mod tests {
                             "{layer} 用到叫停的只有钉住的那一行、原样（只提一次名字而永不叫停的写法多一行或那一行变了）：{rest_of_the_layer:#?}"
                         ));
                     }
-                    let depths = brace_depth_at_the_start_of_each_line(&rest_of_the_layer);
+                    let depths = brace_depth_at_the_start_of_each_line(&rest_of_the_layer_code);
                     for (pinned_offset, _line) in &lines_using_it {
                         if depths[*pinned_offset] != 1 {
                             found.push(format!(
@@ -5297,8 +5528,18 @@ mod tests {
                                 depths[*pinned_offset]
                             ));
                         }
+                        let code_before_the_pinned_line =
+                            rest_of_the_layer_code[..*pinned_offset].join("\n");
+                        if !code_before_the_pinned_line
+                            .trim_end()
+                            .ends_with([';', '\u{7b}', '\u{7d}'])
+                        {
+                            found.push(format!(
+                                "{layer} 的 {pinned_line} 前面紧挨着的是上一句的收尾（分号、左右花括号），不是属性：#[cfg(any())] 挂在它上面编译时整句去掉，下面用的 {binding} 就落到别处同名的条目上"
+                            ));
+                        }
                     }
-                    let rest_of_the_layer_text = rest_of_the_layer.join("\n");
+                    let rest_of_the_layer_text = rest_of_the_layer_code.join("\n");
                     let let_bindings = whole_word_positions(&rest_of_the_layer_text, "let")
                         .into_iter()
                         .filter(|let_keyword| {
@@ -5315,7 +5556,7 @@ mod tests {
                             "{layer} 里 {binding} 只由钉住的那一行 let 绑一次（后面再 let 一次、解构、if let 就换掉了记住的叫停），读到 {let_bindings} 次"
                         ));
                     }
-                    let lines_naming_the_type = rest_of_the_layer
+                    let lines_naming_the_type = rest_of_the_layer_code
                         .iter()
                         .filter(|line| line.contains(type_name))
                         .count();
@@ -5329,6 +5570,7 @@ mod tests {
             }
             layers_handed_the_stop_request.push((layer, parameter));
         }
+        found.extend(stop_request_shapes_outside_the_pinned_type(&code, &places));
         let expected_layers = [
             ("check_blocks_where_until", STOP_REQUEST_PARAMETER_NAME),
             (
@@ -5457,6 +5699,237 @@ mod tests {
         );
     }
 
+    /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4，攻方 K4-2 v2）：真测试模块头两行之前写一份在块注释里（或跨行的字符串里），
+    /// 中间放一层收叫停而不问的新层、一个按名字读名单外环境变量的函数：测试模块只从代码里那一处切，两样都照样扫得到。扫这份源码改过的一份，不要卡。
+    #[test]
+    fn the_tests_module_is_cut_at_its_opening_in_code_not_at_a_copy_in_a_comment_or_a_string() {
+        let source = include_str!("crash_amplification.rs");
+        assert_eq!(
+            source.matches(TESTS_MODULE_OPENING).count(),
+            1,
+            "这份源码里测试模块的头两行原文只有代码里那一处"
+        );
+        let new_layer = concat!(
+            "pub fn an_extra_layer_never_asking(first_input: u8, stop_is_",
+            "requested: &(dyn Fn() -> bool + Sync)) -> bool \u{7b} let _ = stop_is_",
+            "requested; first_input == 0 \u{7d}\n"
+        );
+        let reading_a_variable_left_out = concat!(
+            "pub fn reads_a_variable_left_out_of_the_list() -> bool \u{7b} ",
+            "std::env::var(\"SINGLEFS_A_VARIABLE_LEFT_OUT_OF_THE_LIST\").is_ok() \u{7d}\n"
+        );
+        for (form, copy_of_the_opening) in [
+            ("块注释", format!("/*\n{TESTS_MODULE_OPENING}\n*/\n")),
+            (
+                "跨行的字符串",
+                format!(
+                    "pub const A_COPY_OF_THE_OPENING: &str = \"\n{TESTS_MODULE_OPENING}\n\";\n"
+                ),
+            ),
+        ] {
+            let hiding = source.replacen(
+                TESTS_MODULE_OPENING,
+                &format!(
+                    "{copy_of_the_opening}{new_layer}{reading_a_variable_left_out}{TESTS_MODULE_OPENING}"
+                ),
+                1,
+            );
+            let found = ways_a_layer_handed_the_stop_request_could_drop_it(&hiding);
+            assert!(
+                found
+                    .iter()
+                    .any(|finding| finding.contains("(\"an_extra_layer_never_asking\", ")),
+                "{form}里写一份测试模块的头两行，藏不住它与真测试模块之间收叫停的新层：{found:#?}"
+            );
+            let names_read = environment_variables_read_by_name_outside_the_tests(&hiding);
+            assert!(
+                names_read
+                    .iter()
+                    .any(|name| name == "SINGLEFS_A_VARIABLE_LEFT_OUT_OF_THE_LIST"),
+                "{form}里写一份测试模块的头两行，藏不住它与真测试模块之间按名字读的环境变量：{names_read:?}"
+            );
+        }
+    }
+
+    /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4，正推 K4-1 A01：长的那一层钉住那一行上面挂 `#[cfg(any())]`，模块里另放一个
+    /// 叫 `stop_request` 的静态量，编译时那一行去掉、下面用的是静态量，整行比、层数、`let` 次数照绿）：钉住那一行前面紧挨着的（跳过注释）
+    /// 要是上一句的收尾，挂着属性（一行的、跨行的、属性与那一行之间夹注释行的）都红；叫停参数前面挂属性同样红。扫这份源码改过的一份，不要卡。
+    #[test]
+    fn the_stop_request_scan_finds_an_attribute_on_the_pinned_line_or_on_a_stop_parameter() {
+        let source = include_str!("crash_amplification.rs");
+        let pinned_line = concat!(
+            "    let stop_request = StopRequestRememberedInThisRun::new(stop_is_",
+            "requested);\n"
+        );
+        assert_eq!(
+            source.matches(pinned_line).count(),
+            1,
+            "长的那一层钉住的那一行在这份源码里恰好一处"
+        );
+        for (form, attribute) in [
+            ("一行的属性", "    #[cfg(any())]\n"),
+            ("跨行的属性", "    #[cfg(\n        any()\n    )]\n"),
+            (
+                "属性与那一行之间夹注释行",
+                "    #[cfg(any())]\n    // 编译时这一句去掉\n",
+            ),
+        ] {
+            let under_an_attribute =
+                source.replacen(pinned_line, &format!("{attribute}{pinned_line}"), 1);
+            let found = ways_a_layer_handed_the_stop_request_could_drop_it(&under_an_attribute);
+            assert!(
+                found
+                    .iter()
+                    .any(|finding| finding.contains("前面紧挨着的是上一句的收尾")),
+                "钉住那一行上面挂{form}，红：{found:#?}"
+            );
+        }
+        let stop_parameter = concat!("\n    stop_is_", "requested: &(dyn Fn() -> bool + Sync),\n");
+        let parameter_under_an_attribute = source.replacen(
+            stop_parameter,
+            &format!("\n    #[cfg(any())]{stop_parameter}"),
+            1,
+        );
+        assert_ne!(
+            parameter_under_an_attribute, source,
+            "造的源码在叫停参数上挂了属性"
+        );
+        let found =
+            ways_a_layer_handed_the_stop_request_could_drop_it(&parameter_under_an_attribute);
+        assert!(
+            found.iter().any(|finding| finding
+                .contains(concat!("的叫停参数 stop_is_", "requested 前面挂着属性"))),
+            "叫停参数上面挂属性，红：{found:#?}"
+        );
+    }
+
+    /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4，正推 K4-2 C02–C12）：新加一层收叫停、叫停写成钉住的
+    /// `&(dyn Fn() -> bool + Sync)` 之外的类型，照 `Fn() -> bool` 这个形（`FnMut`、`FnOnce`、函数指针也算）找到、红；类型里夹注释的，
+    /// 注释不算，照钉住的类型找到这一层、它不在名单里红。扫这份源码改过的一份，不要卡。
+    #[test]
+    fn the_stop_request_scan_finds_new_layers_whose_stop_type_is_written_another_way() {
+        let source = include_str!("crash_amplification.rs");
+        for (marker, new_layer) in [
+            (
+                "an_extra_layer_whose_stop_is_also_send",
+                "fn an_extra_layer_whose_stop_is_also_send(stop: &(dyn Fn() -> bool + Send + Sync)) -> bool \u{7b} false \u{7d}",
+            ),
+            (
+                "an_extra_layer_whose_stop_is_static",
+                "fn an_extra_layer_whose_stop_is_static(stop: &(dyn Fn() -> bool + Sync + 'static)) -> bool \u{7b} false \u{7d}",
+            ),
+            (
+                "an_extra_layer_whose_stop_is_borrowed_mutably",
+                "fn an_extra_layer_whose_stop_is_borrowed_mutably(stop: &mut (dyn Fn() -> bool + Sync)) -> bool \u{7b} false \u{7d}",
+            ),
+            (
+                "an_extra_layer_whose_stop_lives_for_the_whole_run",
+                "fn an_extra_layer_whose_stop_lives_for_the_whole_run(stop: &'static (dyn Fn() -> bool + Sync)) -> bool \u{7b} false \u{7d}",
+            ),
+            (
+                "an_extra_layer_whose_stop_type_holds_a_comment",
+                "fn an_extra_layer_whose_stop_type_holds_a_comment(stop: &(dyn Fn() -> bool /* 叫停 */ + Sync)) -> bool \u{7b} false \u{7d}",
+            ),
+            (
+                "an_extra_layer_whose_stop_is_boxed",
+                "fn an_extra_layer_whose_stop_is_boxed(stop: Box<dyn Fn() -> bool + Sync>) -> bool \u{7b} false \u{7d}",
+            ),
+            (
+                "an_extra_layer_whose_stop_names_sync_first",
+                "fn an_extra_layer_whose_stop_names_sync_first(stop: &(dyn Sync + Fn() -> bool)) -> bool \u{7b} false \u{7d}",
+            ),
+            (
+                "an_extra_layer_whose_stop_is_a_bare_trait_object",
+                "fn an_extra_layer_whose_stop_is_a_bare_trait_object(stop: &dyn Fn() -> bool) -> bool \u{7b} false \u{7d}",
+            ),
+            (
+                "an_extra_layer_whose_stop_is_an_impl",
+                "fn an_extra_layer_whose_stop_is_an_impl(stop: impl Fn() -> bool) -> bool \u{7b} false \u{7d}",
+            ),
+            (
+                "an_extra_layer_whose_stop_is_generic",
+                "fn an_extra_layer_whose_stop_is_generic<Stop: Fn() -> bool + Sync>(stop: &Stop) -> bool \u{7b} false \u{7d}",
+            ),
+            (
+                "StopRequestOfAnExtraLayer",
+                "type StopRequestOfAnExtraLayer = dyn Fn() -> bool + Sync;\nfn an_extra_layer_whose_stop_is_an_alias(stop: &StopRequestOfAnExtraLayer) -> bool \u{7b} false \u{7d}",
+            ),
+            (
+                "an_extra_layer_whose_stop_is_fn_mut",
+                "fn an_extra_layer_whose_stop_is_fn_mut(stop: &mut dyn FnMut() -> bool) -> bool \u{7b} false \u{7d}",
+            ),
+            (
+                "an_extra_layer_whose_stop_is_a_function_pointer",
+                "fn an_extra_layer_whose_stop_is_a_function_pointer(stop: fn() -> bool) -> bool \u{7b} false \u{7d}",
+            ),
+        ] {
+            let source_with_the_new_layer = format!("{source}\n{new_layer}\n");
+            let found =
+                ways_a_layer_handed_the_stop_request_could_drop_it(&source_with_the_new_layer);
+            assert!(
+                found.iter().any(|finding| finding.contains(marker)),
+                "叫停写成别的类型的新层 {marker} 找得到、红：{found:#?}"
+            );
+        }
+    }
+
+    /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4，攻方 v4、正推 K4-2 A11、A12、B02）：注释与字符串里写的不算——签名行尾注释里的
+    /// `fn` 不被当成层名，字符串、行尾注释里的 `let stop_request` 不算又绑一次，钉住那一行、短层函数体上的行尾注释不算改了那一句，都判绿。
+    /// 扫这份源码改过的一份，不要卡。
+    #[test]
+    fn the_stop_request_scan_reads_layer_names_rebindings_and_pinned_lines_past_comments_and_strings(
+    ) {
+        let source = include_str!("crash_amplification.rs");
+        let pinned_line = concat!(
+            "    let stop_request = StopRequestRememberedInThisRun::new(stop_is_",
+            "requested);\n"
+        );
+        let signature_lines = concat!(
+            "    cards_taken_from: GpuCardsSource,\n    stop_is_",
+            "requested: &(dyn"
+        );
+        let call_line_of_a_short_layer =
+            "        &|block_sequence| selected_block_sequences.contains(&block_sequence),\n";
+        for (anchor, replacement, form) in [
+            (
+                signature_lines,
+                signature_lines.replacen(",\n", ", // 判法版本，fn 外面定\n", 1),
+                "签名行尾注释里写 fn",
+            ),
+            (
+                pinned_line,
+                format!("{pinned_line}    let _quoted = \"let stop_request = the_never_stopping_request();\";\n"),
+                "字符串里写 let stop_request",
+            ),
+            (
+                pinned_line,
+                format!("{pinned_line}    let _counted_once = 0; // let stop_request = 换掉记住的叫停\n"),
+                "行尾注释里写 let stop_request",
+            ),
+            (
+                pinned_line,
+                pinned_line.replacen(";\n", "; // 记住的叫停\n", 1),
+                "钉住那一行带行尾注释",
+            ),
+            (
+                call_line_of_a_short_layer,
+                call_line_of_a_short_layer.replacen(",\n", ", // 只判点了名的块\n", 1),
+                "短层函数体上带行尾注释",
+            ),
+        ] {
+            assert!(
+                source.contains(anchor),
+                "{form}：这份源码里有要改的那一处 {anchor}"
+            );
+            let with_comments_and_strings = source.replacen(anchor, &replacement, 1);
+            assert_eq!(
+                ways_a_layer_handed_the_stop_request_could_drop_it(&with_comments_and_strings),
+                Vec::<String>::new(),
+                "{form}，判绿"
+            );
+        }
+    }
+
     fn registered(classes: &[GpuComputationClass]) -> BTreeSet<GpuComputationClass> {
         classes.iter().copied().collect()
     }
````

### 第二十三块变异表追加（16 行）

````tsv
第10z批第二十三块 GPU 取卡 K4 卡池扫源码照原文找第一处测试模块头两行切（块注释、字符串里写一份就把切口挪过去，攻方 K4-2）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	        &source[..openings[0]]	        &source[..source.find(TESTS_MODULE_OPENING).expect("变异：照原文找第一处")]	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scans_cut_the_tests_module_at_its_opening_in_code_not_at_a_copy_in_a_comment_or_a_string	the_card_pool_scans_cut_the_tests_module_at_its_opening_in_code_not_at_a_copy_in_a_comment_or_a_string
第10z批第二十三块 GPU 取卡 K4 卡池扫源码不看 use 上挂的 cfg（挂 #[cfg(any())] 的 use 引进钉住的函数、另起同名函数漏过，正推 K4-1 B12）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	            let attributes = names_of_the_attributes_right_before(&tokens, *use_index);	            let attributes: Vec<&str> = Vec::new();	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scan_finds_a_pinned_function_brought_in_by_a_use_under_a_cfg	the_card_pool_scan_finds_a_pinned_function_brought_in_by_a_use_under_a_cfg
第10z批第二十三块 GPU 取卡 K4 卡池扫源码 use wgpu; 引外部 crate 自己也拒（攻方 g1 又判红）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                    if matches!(name.as_str(), "std" | "wgpu") && source != name {	                    if matches!(name.as_str(), "std" | "wgpu") {	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scan_accepts_writings_that_cannot_replace_what_the_pinned_bodies_call	the_card_pool_scan_accepts_writings_that_cannot_replace_what_the_pinned_bodies_call
第10z批第二十三块 GPU 取卡 K4 卡池扫源码块里的 use 一律当罩到钉住那两段（别的函数体里 use core as std; 又判红，攻方 g2）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	        SIGNATURES_OF_THE_PINNED_BODIES.iter().any(|signature| {	        true || SIGNATURES_OF_THE_PINNED_BODIES.iter().any(|signature| {	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scan_accepts_writings_that_cannot_replace_what_the_pinned_bodies_call	the_card_pool_scan_accepts_writings_that_cannot_replace_what_the_pinned_bodies_call
第10z批第二十三块 GPU 取卡 K4 卡池扫源码块里的 use 一律当罩不到（钉住那一段挪进函数体、同一块里 use core as std 漏过）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	        SIGNATURES_OF_THE_PINNED_BODIES.iter().any(|signature| {	        false && SIGNATURES_OF_THE_PINNED_BODIES.iter().any(|signature| {	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scan_finds_a_rename_in_a_block_holding_a_pinned_body_and_an_item_level_macro_under_an_attribute	the_card_pool_scan_finds_a_rename_in_a_block_holding_a_pinned_body_and_an_item_level_macro_under_an_attribute
第10z批第二十三块 GPU 取卡 K4 卡池扫源码条目层宏不看前一个词（属性与常量初始值里的表达式宏又判红，正推 K4-2 L7–L9）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	            let token_before = index\n                .checked_sub(1)\n                .map(|index_before| module_tokens[index_before].1);	            let token_before: Option<&str> = None;	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scan_accepts_writings_that_cannot_replace_what_the_pinned_bodies_call	the_card_pool_scan_accepts_writings_that_cannot_replace_what_the_pinned_bodies_call
第10z批第二十三块 GPU 取卡 K4 卡池扫源码把 impl … + use<'a> 的 use 当成一条 use 读（精确捕获当场 panic，正推 K4-2 L10）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                    && tokens.get(use_index + 1) != Some(&"<")	                    && tokens.get(use_index + 1) != Some(&"变异：不认精确捕获")	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scan_accepts_writings_that_cannot_replace_what_the_pinned_bodies_call	the_card_pool_scan_accepts_writings_that_cannot_replace_what_the_pinned_bodies_call
第10z批第二十三块 GPU 取卡 K4 卡池扫源码分词不认 cr 原始字符串（与崩溃放量那一份认法不一致，正推 K4-2 L13b）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                if matches!(word, "r" | "br" | "cr")	                if matches!(word, "r" | "br")	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scan_accepts_writings_that_cannot_replace_what_the_pinned_bodies_call	the_card_pool_scan_accepts_writings_that_cannot_replace_what_the_pinned_bodies_call
第10z批第二十三块 GPU 取卡 K4 卡池扫源码拼成关键字的原始标识符照关键字读（r#mod 又被当成另起模块，正推 K4-2 L14）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                    if KEYWORDS_A_RAW_IDENTIFIER_CAN_SPELL.contains(&&name[..name_length]) {	                    if false && KEYWORDS_A_RAW_IDENTIFIER_CAN_SPELL.contains(&&name[..name_length]) {	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scan_accepts_writings_that_cannot_replace_what_the_pinned_bodies_call	the_card_pool_scan_accepts_writings_that_cannot_replace_what_the_pinned_bodies_call
第10z批第二十三块 崩溃放量 K4 测试模块照原文找第一处头两行切（块注释里写一份藏住它与真测试模块之间的新层，攻方 K4-2 v2）	crates/singlefs-checker-tier/src/crash_amplification.rs	        let opening = openings[0];	        let text_start = source.find(TESTS_MODULE_OPENING).expect("变异：照原文找第一处");\n        let text_end = text_start + source[text_start..].find(TOP_LEVEL_CLOSING_BRACE_LINE).expect("变异：照原文找收尾");\n        return format!("{}{}", &source[..text_start], &source[text_end + 2..]);\n        let opening = openings[0];	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_tests_module_is_cut_at_its_opening_in_code_not_at_a_copy_in_a_comment_or_a_string	the_tests_module_is_cut_at_its_opening_in_code_not_at_a_copy_in_a_comment_or_a_string
第10z批第二十三块 崩溃放量 K4 长的那一层钉住那一行前面挂属性不看（#[cfg(any())] 挂在钉住那一行上漏过，正推 K4-1 A01）	crates/singlefs-checker-tier/src/crash_amplification.rs	                        if !code_before_the_pinned_line	                        if false && !code_before_the_pinned_line	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_scan_finds_an_attribute_on_the_pinned_line_or_on_a_stop_parameter	the_stop_request_scan_finds_an_attribute_on_the_pinned_line_or_on_a_stop_parameter
第10z批第二十三块 崩溃放量 K4 叫停参数前面挂属性不看（#[cfg(any())] 挂在叫停参数上漏过）	crates/singlefs-checker-tier/src/crash_amplification.rs	            if is_the_parameter_under_an_attribute {	            if false && is_the_parameter_under_an_attribute {	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_scan_finds_an_attribute_on_the_pinned_line_or_on_a_stop_parameter	the_stop_request_scan_finds_an_attribute_on_the_pinned_line_or_on_a_stop_parameter
第10z批第二十三块 崩溃放量 K4 叫停写成钉住之外的类型不找（+ Send、&mut、Box<dyn …> 一类的新层漏过，正推 K4-2 C02–C12）	crates/singlefs-checker-tier/src/crash_amplification.rs	        found.extend(stop_request_shapes_outside_the_pinned_type(&code, &places));	        drop(stop_request_shapes_outside_the_pinned_type(&code, &places));	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_scan_finds_new_layers_whose_stop_type_is_written_another_way	the_stop_request_scan_finds_new_layers_whose_stop_type_is_written_another_way
第10z批第二十三块 崩溃放量 K4 字面量不从找名字的那一份里换掉（字符串里的 let stop_request 又数成再绑一次，正推 K4-2 A12）	crates/singlefs-checker-tier/src/crash_amplification.rs	                    KindOfACharacterInTheSource::Literal => {\n                        without_comments.push(*character);\n                        without_comments_and_literals.push_str(&spaces);	                    KindOfACharacterInTheSource::Literal => {\n                        without_comments.push(*character);\n                        without_comments_and_literals.push(*character);	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_scan_reads_layer_names_rebindings_and_pinned_lines_past_comments_and_strings	the_stop_request_scan_reads_layer_names_rebindings_and_pinned_lines_past_comments_and_strings
第10z批第二十三块 崩溃放量 K4 注释不从找名字的那一份里换掉（签名行尾注释里的 fn 又被当成层名，攻方 v4）	crates/singlefs-checker-tier/src/crash_amplification.rs	                    KindOfACharacterInTheSource::Comment => {\n                        without_comments.push_str(&spaces);\n                        without_comments_and_literals.push_str(&spaces);	                    KindOfACharacterInTheSource::Comment => {\n                        without_comments.push_str(&spaces);\n                        without_comments_and_literals.push(*character);	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_scan_reads_layer_names_rebindings_and_pinned_lines_past_comments_and_strings	the_stop_request_scan_reads_layer_names_rebindings_and_pinned_lines_past_comments_and_strings
第10z批第二十三块 崩溃放量 K4 整句、整行比在没去掉注释的原文上做（钉住那一行、短层函数体上的行尾注释又判红，正推 K4-2 A11、B02）	crates/singlefs-checker-tier/src/crash_amplification.rs	            let rest_of_the_layer: Vec<&str> = without_comments	            let rest_of_the_layer: Vec<&str> = outside_the_tests	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_scan_reads_layer_names_rebindings_and_pinned_lines_past_comments_and_strings	the_stop_request_scan_reads_layer_names_rebindings_and_pinned_lines_past_comments_and_strings
````
