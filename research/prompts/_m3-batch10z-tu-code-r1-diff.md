# 根环实现第 10z 批第二十、二十一块：代码轮第一轮改动全文（2026-10-09）

两块实现员各交的 `crates.patch` 原样，按合入次序：第二十块、第二十一块。各块变异表追加行与整行替换行原样附在各块之后；合入验证时主 agent 把第二十一块追加的一行（主工作区 `crates/mutations.tsv` 第 3553 行）替换文里的 `**token == "\u{7b}"` 改成 `token.as_bytes() == [0x7b_u8]`（门禁只认 `\n` 转义），改后单跑那一行证红抓到，附在末尾。

## 第二十块

````diff
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index 180a992..d45a0fa 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -17,9 +17,15 @@
 //! 缺根那一半是 C585（会话发布放进发布途中取下的旧快照） 用户定的改法 C）。
 //! 发布路径与抬 F 各自开写入口的做法不变。
 
-use crate::address::{CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration};
+use std::collections::BTreeMap;
+
+use crate::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InodeNumber, InstanceGeneration,
+};
 use crate::allocator::{PlacementOnDevice, PoolAllocator};
-use crate::block_device::BlockDevice;
+use crate::block_device::{
+    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
+};
 use crate::make_filesystem::MakeFilesystemParameters;
 use crate::mount::{
     devices_behind_the_current_version, devices_without_a_self_verified_system_configuration,
@@ -166,13 +172,14 @@ pub struct NoSpaceAfterRaisingTheFloor {
     pub stop: FloorRaiseStop,
     /// 被拒之前发成的那几版（[`VersionsPublishedInThisChange`]）。
     pub versions_published: VersionsPublishedInThisChange,
-    /// 这一次是不是被宽护卡住（[`WideProtectionAtTheCeiling`]）：F 已在上限、C 那一圈还隔离着槽时点名这一形、带上两条出路，
+    /// 这一次是不是被宽护卡住（[`WideProtectionAtTheCeiling`]）：F 已在上限、C 那一圈还隔离着槽时点名这一形、带上出路（推环那一条预演过放得下才列），
     /// 别的形是 [`WideProtectionAtTheCeiling::NotWhatStopsTheAdmission`]，照旧只是空间不够。
     pub wide_protection_at_the_ceiling: WideProtectionAtTheCeiling,
 }
 
 /// 推满仍不够时，挡着准入的是不是「这个进程写掉那一槽才放」的那一份隔离（用户 2026-10-08 弹窗定「显式推环命令加带出路的错」：
-/// 不自动推环，准入因宽护卡住时报一个点名「被宽护卡住」的错，出路写两条）。
+/// 不自动推环，准入因宽护卡住时报一个点名「被宽护卡住」的错、带上出路；用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」：
+/// 推环那一条出路预演过、推完这次改动放得下才列）。
 ///
 /// 为什么不另开一个 [`UserChangeRefused`] 成员：checker 档的装置（`e158_root_choice_repair`、`e168_delete_ring_window_borrow`）对
 /// `UserChangeRefused` 与 `FloorRaiseStop` 写的是穷举 `match`，多一个成员它们就编不过，而 checker 档这一批不改（第 10z 批第十六块派发）。
@@ -191,7 +198,12 @@ pub enum WideProtectionAtTheCeiling {
 /// 那一槽写不掉，不管它就隔离到进程结束（第 10z 批第十四块实现员报告第六节，384 槽盘 22 形写全被拒）。
 ///
 /// 这里分不出 C 那一圈里哪一份是宽护、哪一份是窄的那几份：分得出要在根环表里另记一样（`PoolAllocator` 的 `{:?}` 随之变，checker 档 E171
-/// 记它），这一份不记；两条出路对整圈同样走得通。
+/// 记它），这一份不记；出路对整圈算。
+///
+/// **宽护是挡着准入的之一，不是唯一**：同一刻没有 C 那一圈，准入照样可以被拒（第 10z 批代码轮第一轮攻方
+/// `research/prompts/m3-batch10z-pqrs-code-r1-opus-output.md` 第一节：同一个池停在 R 之前、C 一槽都没有的对照臂，同一个长度也被拒的 27 格，
+/// 这里照样报这一形）。所以报这一形不说明写掉 R 那一槽之后放得下：推完放不放得下由 `pushing_the_root_ring` 那一份预演说，
+/// 放不下（推完上限退到环里最旧的根、推环自己换下的那几份要等环再转一圈才回收，同一节「读法」）就不列推环。
 #[derive(Debug)]
 pub struct StuckBehindTheWideProtection {
     /// 推环要写到的那一槽：C 那一圈还没解开、记着落点的槽里，从现行那一版起按 txg 往后最后一个轮到的（R 那一槽）。
@@ -201,33 +213,353 @@ pub struct StuckBehindTheWideProtection {
     pub checkpoint_txg_of_r: Option<CheckpointTxg>,
     /// 从现行那一版起要发几次 F 不变的空发布才写掉 R 那一槽（不超过环长 3S）。
     pub empty_publishes_until_the_ring_slot_of_r_is_written: u64,
-    /// C 那一圈每块盘上这一刻隔离着的槽数（按分配器里盘的次序，同一个槽只数一次）：宽护挡着的槽数。
+    /// C 那一圈每块盘上这一刻隔离着的槽数（按分配器里盘的次序，同一个槽只数一次）：宽护挡着的槽数。只数 C 那一圈罩着的，
+    /// 被抛弃根罩着、C 不罩的不在其内。
     pub isolated_slots_on_each_device: Vec<(DeviceIdentity, u64)>,
+    /// 在分配器的拷贝与只进内存的盘上预演推环、推完再预演这次改动（连同它会推的抬 F）的结局（[`RootRingPushRehearsal`]）：
+    /// 推环这条出路列不列按它定。
+    pub pushing_the_root_ring: RootRingPushRehearsal,
+}
+
+/// 「被宽护卡住」那一刻预演推环与推完之后这次改动的准入（用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」）：
+/// 在分配器的一份拷贝上、对只进内存的盘走一遍推环入口与推完之后这次改动剩下的发布（`rehearse_the_push_then_the_remainder_of_the_change`），
+/// 一个写都没发到盘上，会话不动。
+#[derive(Debug)]
+pub enum RootRingPushRehearsal {
+    /// 推环做成，推完之后这次改动放得下：出路列推环。`isolated_slots_released_by_the_push_on_each_device` 是推完之后每块盘隔离的槽数
+    /// 比推之前少了几个（按分配器里盘的次序；推完反而多的那块盘记 0）：C 那一圈与被抛弃根罩着同一个槽时，被抛弃根还在环里那一槽就不放，
+    /// 推环途中写掉被抛弃根那一槽时它独占的也放出来——与 `isolated_slots_on_each_device` 不是同一个数。
+    ChangeAdmittedAfterThePush {
+        isolated_slots_released_by_the_push_on_each_device: Vec<(DeviceIdentity, u64)>,
+    },
+    /// 推环做成，推完之后这次改动照旧被拒（`refusal` 是预演里交回的那一种，例如推完上限退到环里最旧的根、推满仍不够）：出路不列推环。
+    ChangeStillRefusedAfterThePush {
+        isolated_slots_released_by_the_push_on_each_device: Vec<(DeviceIdentity, u64)>,
+        refusal: Box<UserChangeRefused>,
+    },
+    /// 推环自己在预演里被拒（`RootRingPushRefused` 的哪一种原样）：出路不列推环。
+    PushRefused(Box<RootRingPushRefused>),
 }
 
 impl StuckBehindTheWideProtection {
-    /// 两条出路，调用方照抄就能报给用户：卸载重挂；调推环入口（会丢掉最近可退到的状态，调用即确认）。按这个次序。
+    /// 出路，调用方照抄就能报给用户：卸载重挂一定列；推环（会丢掉最近可退到的状态，调用即确认）只在预演过推完这次改动放得下时列
+    /// （[`RootRingPushRehearsal::ChangeAdmittedAfterThePush`]）。按这个次序。
     #[must_use]
-    pub fn ways_out(&self) -> [String; 2] {
-        let txg_of_r = self.checkpoint_txg_of_r.map_or_else(
-            || "那一槽没有读得出的有效根".to_string(),
-            |txg| format!("txg {}", txg.0),
-        );
-        [
-            format!(
-                "出路一：正常卸载（mount::unmount）之后再可写挂载一次。再挂时 R（{txg_of_r}）按新实例的实例表判成被抛弃，影子账只护它自己账里引用的落点，宽护不再出现；R 的账再挂时仍读不出就照旧拒可写（C694 的 B）。"
+    pub fn ways_out(&self) -> Vec<String> {
+        let way_one = match self.checkpoint_txg_of_r {
+            Some(txg_of_r) => format!(
+                "出路一：正常卸载（mount::unmount）之后再可写挂载一次。再挂时 R（txg {}）按新实例的实例表判成被抛弃，影子账只护它自己账里引用的落点，宽护不再出现。R 的账读不出若是盘上永久坏，卸载之后再挂会被拒（C694 的 B），而会话在卸载时已断开。",
+                txg_of_r.0
             ),
-            format!(
-                "出路二：调 MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states（推环到 R 那一槽，根环槽 {}.{}）：发 {} 次 F 不变的空发布写掉那一槽，宽护随之解开、隔离的 {} 个槽放出来；根环里更早的回退候选会被这一串整环写掉，回退候选集掉到少于 4 个不同状态（D16（发布语义） 已定项 1），调用即确认。",
-                self.ring_slot_of_r.region,
-                self.ring_slot_of_r.slot,
-                self.empty_publishes_until_the_ring_slot_of_r_is_written,
-                self.isolated_slots_on_each_device
-                    .iter()
-                    .map(|(_, isolated)| *isolated)
-                    .sum::<u64>(),
+            None => format!(
+                "出路一：正常卸载（mount::unmount）之后再可写挂载一次。R 那一槽（根环槽 {}.{}）没有读得出的有效根：再挂时择的是卸载推出来的最后一版，R 那几条记录不在它的水位之上，C 那一圈照它重算、不再有这一槽，宽护不再出现。",
+                self.ring_slot_of_r.region, self.ring_slot_of_r.slot
             ),
-        ]
+        };
+        let mut ways_out = vec![way_one];
+        match &self.pushing_the_root_ring {
+            RootRingPushRehearsal::ChangeAdmittedAfterThePush {
+                isolated_slots_released_by_the_push_on_each_device,
+            } => {
+                let released_on_each_device: Vec<String> =
+                    isolated_slots_released_by_the_push_on_each_device
+                        .iter()
+                        .map(|(device, released)| format!("盘 {} 上 {released} 个", device.0))
+                        .collect();
+                ways_out.push(format!(
+                    "出路二：调 MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states（推环到 R 那一槽，根环槽 {}.{}）：从现行那一版起发 {} 次 F 不变的空发布，写掉根环里接下来轮到的 {} 槽，R 那一槽是最后一槽；宽护随之解开，隔离的槽放出来 {} 个（{}）。这几槽里原来的根换成同一个用户状态的空发布根，回退候选集可以掉到少于 4 个不同状态（D16（发布语义） 已定项 1），调用即确认。预演过：推完之后这次改动放得下。",
+                    self.ring_slot_of_r.region,
+                    self.ring_slot_of_r.slot,
+                    self.empty_publishes_until_the_ring_slot_of_r_is_written,
+                    self.empty_publishes_until_the_ring_slot_of_r_is_written,
+                    isolated_slots_released_by_the_push_on_each_device
+                        .iter()
+                        .map(|(_, released)| *released)
+                        .sum::<u64>(),
+                    released_on_each_device.join("，"),
+                ));
+            }
+            RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. }
+            | RootRingPushRehearsal::PushRefused(_) => {}
+        }
+        ways_out
+    }
+}
+
+/// 预演用的盘：读穿到底下那块盘（这一刻盘上的字节），写与写零只记进内存、盖在之后的读上，屏障什么都不做——一个写都不发到底下那块盘。
+/// 「被宽护卡住」那一报预演推环与推完之后这次改动的准入用（[`rehearse_the_push_then_the_remainder_of_the_change`]）。
+/// 底下那块盘借成 `&dyn BlockDevice`：预演里走的会话发布、推环、抬 F 那几层按盘的类型实例化，预演里再判「被宽护卡住」时
+/// 拿这一层再包一层，借成 trait 对象之后类型不再往里长（泛型借底下的类型就是无穷层实例化）。
+struct DeviceKeepingWritesInMemory<'disk> {
+    disk: &'disk dyn BlockDevice,
+    physical_block_size: PhysicalBlockSizeInBytes,
+    /// 写过的物理块：块号（按 `physical_block_size`）→ 那一块现在的字节。写零也逐块记成全 0（发布路径不写零，只有 mkfs 写零）。
+    written_blocks: BTreeMap<u64, Vec<u8>>,
+}
+
+impl<'disk> DeviceKeepingWritesInMemory<'disk> {
+    fn over_the_disk(disk: &'disk dyn BlockDevice) -> Self {
+        DeviceKeepingWritesInMemory {
+            disk,
+            physical_block_size: disk.probe_physical_block_size(),
+            written_blocks: BTreeMap::new(),
+        }
+    }
+
+    fn block_bytes(&self) -> u64 {
+        u64::from(self.physical_block_size.0)
+    }
+
+    /// 与两个后端同一条约定（`block_device` 文件后端的 `check_aligned_and_in_range`）：按物理块对齐、不越过设备末尾。
+    /// 写不交给底下那块盘，这里不判就收得下一个真盘会拒的写。
+    fn check_aligned_and_in_range(
+        &self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<(), BlockDeviceError> {
+        let block_bytes = self.block_bytes();
+        if !offset.0.is_multiple_of(block_bytes) || !length.is_multiple_of(block_bytes) {
+            return Err(BlockDeviceError::Unaligned {
+                offset,
+                length,
+                physical_block_size: self.physical_block_size.0,
+            });
+        }
+        let device_size = self.disk.size_in_bytes();
+        match offset.0.checked_add(length) {
+            Some(end) if end <= device_size => Ok(()),
+            Some(_) | None => Err(BlockDeviceError::OutOfRange {
+                offset,
+                length,
+                device_size,
+            }),
+        }
+    }
+
+    /// 块号 `first_unwritten_block` 起、到 `end_block` 之前那一段没写过的块从底下那块盘读进 `buffer` 里对应的那一截
+    /// （`buffer` 从块号 `first_block_of_the_buffer` 起）。
+    fn read_unwritten_blocks_from_the_disk(
+        &self,
+        first_unwritten_block: u64,
+        end_block: u64,
+        first_block_of_the_buffer: u64,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        let block_bytes = self.block_bytes();
+        let start_in_buffer =
+            usize::try_from((first_unwritten_block - first_block_of_the_buffer) * block_bytes)
+                .expect("一次读的长度装得进 usize：它就是调用方给的缓冲区长度以内");
+        let end_in_buffer = usize::try_from((end_block - first_block_of_the_buffer) * block_bytes)
+            .expect("一次读的长度装得进 usize：它就是调用方给的缓冲区长度以内");
+        self.disk.read_at(
+            DeviceOffsetInBytes(first_unwritten_block * block_bytes),
+            &mut buffer[start_in_buffer..end_in_buffer],
+        )
+    }
+
+    fn keep_in_memory(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
+        let block_bytes = self.block_bytes();
+        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
+        let first_block = offset.0 / block_bytes;
+        // 迭代上界是这次写的块数；不跨轮带状态；没有提前出口。
+        for (block_offset, block) in (0u64..).zip(bytes.chunks(block_length)) {
+            self.written_blocks
+                .insert(first_block + block_offset, block.to_vec());
+        }
+    }
+}
+
+impl BlockDevice for DeviceKeepingWritesInMemory<'_> {
+    fn read_at(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        let length = u64::try_from(buffer.len()).expect("读长装得进 u64");
+        self.check_aligned_and_in_range(offset, length)?;
+        let block_bytes = self.block_bytes();
+        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
+        let first_block = offset.0 / block_bytes;
+        let end_block = first_block + length / block_bytes;
+        // 迭代上界是这次读的块数；跨轮带着还没读的那一段没写过的块从哪一块起；出口只有底下那块盘读错（原样交回）。
+        let mut unwritten_run_start: Option<u64> = None;
+        for block_index in first_block..end_block {
+            match self.written_blocks.get(&block_index) {
+                None => {
+                    unwritten_run_start.get_or_insert(block_index);
+                }
+                Some(written) => {
+                    if let Some(run_start) = unwritten_run_start.take() {
+                        self.read_unwritten_blocks_from_the_disk(
+                            run_start,
+                            block_index,
+                            first_block,
+                            buffer,
+                        )?;
+                    }
+                    let start_in_buffer = usize::try_from(block_index - first_block)
+                        .expect("块数装得进 usize")
+                        * block_length;
+                    buffer[start_in_buffer..start_in_buffer + block_length]
+                        .copy_from_slice(written);
+                }
+            }
+        }
+        if let Some(run_start) = unwritten_run_start {
+            self.read_unwritten_blocks_from_the_disk(run_start, end_block, first_block, buffer)?;
+        }
+        Ok(())
+    }
+
+    fn write_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+        _durability: WriteDurability,
+    ) -> Result<(), BlockDeviceError> {
+        self.check_aligned_and_in_range(
+            offset,
+            u64::try_from(bytes.len()).expect("写长装得进 u64"),
+        )?;
+        self.keep_in_memory(offset, bytes);
+        Ok(())
+    }
+
+    fn write_zeroes_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<(), BlockDeviceError> {
+        self.check_aligned_and_in_range(offset, length)?;
+        let zeros = vec![0u8; usize::try_from(length).expect("写零的长度装得进 usize")];
+        self.keep_in_memory(offset, &zeros);
+        Ok(())
+    }
+
+    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
+        Ok(())
+    }
+
+    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
+        self.physical_block_size
+    }
+
+    fn size_in_bytes(&self) -> u64 {
+        self.disk.size_in_bytes()
+    }
+}
+
+/// 被拒的那一次发布之后，这次用户改动还剩什么要发（「被宽护卡住」那一报推完环之后照它再预演一次，
+/// [`rehearse_the_push_then_the_remainder_of_the_change`]）。
+#[derive(Clone, Copy, Debug)]
+enum RemainderOfTheUserChange<'content> {
+    /// 先把现行那一版的 `deleted_inodes` 走空、再发这次改动（unlink 连同它之后的走空）：被拒的是走空的一步或这次改动本身。
+    WalkTheQueueThenPublishTheChange(UserChange<'content>),
+    /// 只剩走空：unlink 的意图已发成，被拒的是它之后走空的一步（别的改动之后队列本来就空）。
+    WalkTheQueue,
+}
+
+/// 每块盘上这一刻隔离的槽数（D28（挂载期承诺量） 已定项 1 第九项那一项），按分配器里盘的次序。只读内存。
+fn isolated_slots_of_each_device(allocator: &PoolAllocator) -> Vec<(DeviceIdentity, u64)> {
+    allocator
+        .devices
+        .iter()
+        .map(|device_map| (device_map.device, device_map.isolated_slots()))
+        .collect()
+}
+
+/// 「被宽护卡住」那一报之前的预演（用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」）：在分配器的一份拷贝上、对只进内存的盘
+/// （[`DeviceKeepingWritesInMemory`]）先走一遍推环入口（与真调推环入口同一段，[`MountedSession::push_the_root_ring_on`]：它自己的整串预演、
+/// 真发与末尾的断言都在），推完再把这次改动剩下的照会话发布那一段走一遍（`remainder`：准入被拒照样推抬 F，抬 F 的上限按推完的根环算——
+/// 读的是盖着推环那几次写的盘）。读盘照读底下那几块盘；写一个都不发到底下那几块盘；会话的分配器与现行那一版不动。
+///
+/// 递归至多一层：预演里推环做成之后 C 那一圈一槽都不记着（推环入口末尾那条断言），预演里的准入再被拒时 [`root_ring_push_target`]
+/// 交回 `None`、报 `NotWhatStopsTheAdmission`，不再进预演；推环在预演里被拒就不往下走。
+fn rehearse_the_push_then_the_remainder_of_the_change<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &[(DeviceIdentity, Device)],
+    allocator: &PoolAllocator,
+    current: &TransactionOutput,
+    instance: InstanceGeneration,
+    shadow_ledger: ShadowLedger,
+    remainder: RemainderOfTheUserChange<'_>,
+) -> RootRingPushRehearsal {
+    let mut devices_keeping_writes_in_memory: Vec<(
+        DeviceIdentity,
+        DeviceKeepingWritesInMemory<'_>,
+    )> = devices
+        .iter()
+        .map(|(identity, device)| {
+            let disk: &dyn BlockDevice = device;
+            (*identity, DeviceKeepingWritesInMemory::over_the_disk(disk))
+        })
+        .collect();
+    let mut rehearsed_allocator = allocator.clone();
+    let mut rehearsed_version = PoolVersion::of_a_version_of_trees(current.clone());
+    let isolated_before_the_push = isolated_slots_of_each_device(&rehearsed_allocator);
+    if let Err(refusal) = MountedSession::push_the_root_ring_on(
+        parameters,
+        &mut devices_keeping_writes_in_memory,
+        &mut rehearsed_allocator,
+        &mut rehearsed_version,
+        instance,
+    ) {
+        return RootRingPushRehearsal::PushRefused(Box::new(refusal));
+    }
+    let isolated_slots_released_by_the_push_on_each_device: Vec<(DeviceIdentity, u64)> =
+        isolated_before_the_push
+            .iter()
+            .zip(isolated_slots_of_each_device(&rehearsed_allocator))
+            .map(|((device, before), (device_after, after))| {
+                assert_eq!(
+                    *device, device_after,
+                    "推环前后数的是同一个分配器的拷贝：盘的次序不变"
+                );
+                (*device, before.saturating_sub(after))
+            })
+            .collect();
+    let remainder_after_the_push = match remainder {
+        RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change) => {
+            MountedSession::publish_the_change_raising_the_floor_for_space(
+                parameters,
+                &mut devices_keeping_writes_in_memory,
+                &mut rehearsed_allocator,
+                &mut rehearsed_version,
+                instance,
+                shadow_ledger,
+                change,
+            )
+            .map(|_published| ())
+        }
+        RemainderOfTheUserChange::WalkTheQueue => {
+            let mut version_to_walk = rehearsed_version.into_version_of_trees().expect(
+                "推环接在有树的一版后面、发的是带文件那一版的空发布（树表 0 条那一版推环入口先拒了），推完还是有树的一版",
+            );
+            publish_the_queue_until_it_is_empty_raising_the_floor_for_space(
+                parameters,
+                &mut devices_keeping_writes_in_memory,
+                &mut rehearsed_allocator,
+                &mut version_to_walk,
+                instance,
+                shadow_ledger,
+                RemainderOfTheUserChange::WalkTheQueue,
+                PublishedInThisChange {
+                    floor_raises: Vec::new(),
+                    refusals_that_pushed_the_floor_raises: Vec::new(),
+                    versions_published: Vec::new(),
+                },
+            )
+            .map(|_published_in_this_change| ())
+        }
+    };
+    match remainder_after_the_push {
+        Ok(()) => RootRingPushRehearsal::ChangeAdmittedAfterThePush {
+            isolated_slots_released_by_the_push_on_each_device,
+        },
+        Err(refusal) => RootRingPushRehearsal::ChangeStillRefusedAfterThePush {
+            isolated_slots_released_by_the_push_on_each_device,
+            refusal: Box::new(refusal),
+        },
     }
 }
 
@@ -281,10 +613,20 @@ fn root_ring_push_target(
 }
 
 /// 推满仍不够时报不报「被宽护卡住」（[`WideProtectionAtTheCeiling`]）：`stop` 是 F 已在上限、C 那一圈还有记着落点的槽、它们这一刻至少隔离着一个槽，
-/// 才报；别的都照旧。只读分配器（内存），不读盘：判的那一刻准入已经拒了，这里只定交回哪一种。
-fn wide_protection_at_the_ceiling(
+/// 才报；别的都照旧。判这三样只读分配器（内存）。报的时候再预演推环与推完之后这次改动剩下的（`remainder`，
+/// [`rehearse_the_push_then_the_remainder_of_the_change`]）：读盘、一个写都不发、会话不动，出路列不列推环按它定。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样东西：盘上择到的参数、盘表、分配器、被拒的那一次要接在后面的那一版、实例代号、影子账那一臂、这次改动还剩什么、不再推的原因"
+)]
+fn wide_protection_at_the_ceiling<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &[(DeviceIdentity, Device)],
     allocator: &PoolAllocator,
     current: &TransactionOutput,
+    instance: InstanceGeneration,
+    shadow_ledger: ShadowLedger,
+    remainder: RemainderOfTheUserChange<'_>,
     stop: &FloorRaiseStop,
 ) -> WideProtectionAtTheCeiling {
     match stop {
@@ -307,6 +649,15 @@ fn wide_protection_at_the_ceiling(
     let checkpoint_txg_of_r = allocator.root_ring_occupancy().and_then(|occupancy| {
         occupancy.checkpoint_txg_of_the_valid_root_recorded_on(target.ring_slot_of_r)
     });
+    let pushing_the_root_ring = rehearse_the_push_then_the_remainder_of_the_change(
+        parameters,
+        devices,
+        allocator,
+        current,
+        instance,
+        shadow_ledger,
+        remainder,
+    );
     WideProtectionAtTheCeiling::StuckBehindTheWideProtection(Box::new(
         StuckBehindTheWideProtection {
             ring_slot_of_r: target.ring_slot_of_r,
@@ -314,6 +665,7 @@ fn wide_protection_at_the_ceiling(
             empty_publishes_until_the_ring_slot_of_r_is_written: target
                 .publishes_until_the_ring_slot_of_r_is_written,
             isolated_slots_on_each_device,
+            pushing_the_root_ring,
         },
     ))
 }
@@ -326,9 +678,18 @@ pub struct RootRingPushed {
     pub ring_slots_written: Vec<RootRingSlot>,
 }
 
-/// 推环入口没做成。成员按调用方要做的决定分：换盘表或换回这个池的盘、这一形第一版不支持、不必推、空间不够推不过去、发布路径的错。
+/// 推环入口没做成。成员按调用方要做的决定分：等冻结着的那次发布重发、换盘表或换回这个池的盘、这一形第一版不支持、不必推、
+/// 空间不够推不过去、发布路径的错。
 #[derive(Debug)]
 pub enum RootRingPushRefused {
+    /// 分配器上冻结着一次落盘途中失败、还没原样重发的发布（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」；它的 txg 与实例代号）：
+    /// 推环的第一次空发布本来就会被拒（`PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet`），而整串预演走的是发布路径落盘之前那一段、
+    /// 不判冻结，会在冻结着的分配器拷贝上照样过。入口第一道就拒，在核盘表、预演与任何读写之前：盘上逐字节不变、分配器与会话不动。
+    /// 调用方要做的是先让那次发布原样重发（会话这一层没有重发的入口，等进程重开）。
+    PublishFrozenAfterAWriteFailureIsNotResentYet {
+        checkpoint_txg: CheckpointTxg,
+        instance: InstanceGeneration,
+    },
     /// 交进来的盘表没过挂着之后收盘表的那一套核（与抬 F 那一串的空发布同一套：身份交了不止一次、参数或盘表与盘上择到的系统配置不一致、
     /// 有盘不「可见」、有盘落后于现行那一版又缺它的单元或它的根；系统配置择不出来），原样带着那一种 `MountError`。
     /// 一个写都没发，盘上逐字节不变、会话不动。
@@ -336,7 +697,8 @@ pub enum RootRingPushRefused {
     /// 现行那一版树表 0 条：推环的空发布在这一版上是零单元发布（D16（发布语义） 已定项 9），会话层的推环在这一形上条款没写，第一版不支持。
     /// 会话在这一版上也不会被宽护卡住（用户改动在这一版上一律 `NoFileVersionToChange`）。在任何读写之前返回，盘上逐字节不变、会话不动。
     VersionWithoutFileNotSupportedInTheFirstVersion,
-    /// C 那一圈这一刻没有记着落点的槽：这个进程写掉哪一槽都放不出东西，不推。一个写都没发，盘上逐字节不变、会话不动。
+    /// C 那一圈这一刻没有记着落点的槽，或记着落点而它们罩着的槽这一刻一个都没隔离（全在现行账里仍分配、或已释放还在 defer 窗口里）：
+    /// 这个进程写掉哪一槽都放不出东西，不推。一个写都没发，盘上逐字节不变、会话不动。
     NothingHeldUntilThisProcessWritesARingSlot,
     /// 整串空发布在任何写之前的预演里（分配器的一份拷贝上走发布路径落盘之前那一段）第 `refused_publish_in_the_push` 次（从 1 数）被拒：
     /// 推到半路空发布取不到落点（`PublishError::PlacementRefused`）就是这里。一个写都没发，盘上逐字节不变、分配器与会话不动。
@@ -345,7 +707,9 @@ pub enum RootRingPushRefused {
         publishes_in_the_push: usize,
         cause: PublishError,
     },
-    /// 预演过了、真发时有一次发不出去（块设备错这一类；预演与真发只在读盘核换下的那一份对不上时分叉）：[`RootRingPushFailedAfterTheRehearsal`]。
+    /// 预演过了、真发时有一次发不出去（块设备错这一类：落盘途中失败的那一次冻结在分配器上，之后再调入口拒成
+    /// `PublishFrozenAfterAWriteFailureIsNotResentYet`；冻结着时入口第一道就拒，不进预演，所以不在这里。空间上预演与真发只在
+    /// 读盘核换下的那一份对不上时分叉）：[`RootRingPushFailedAfterTheRehearsal`]。
     PublishFailedAfterTheRehearsal(Box<RootRingPushFailedAfterTheRehearsal>),
 }
 
@@ -580,6 +944,40 @@ impl MountedSession {
                 },
             );
         }
+        Self::publish_the_change_raising_the_floor_for_space(
+            parameters,
+            devices,
+            allocator,
+            current,
+            *instance,
+            *shadow_ledger,
+            change,
+        )
+    }
+
+    /// [`Self::publish_user_change`] 核过盘表之后那一半：先走空、再发这次改动（unlink 连同它之后的走空），被拒是空间不够就推抬 F 再判。
+    /// 「被宽护卡住」那一报的预演在只进内存的盘与分配器的拷贝上走同一段（[`rehearse_the_push_then_the_remainder_of_the_change`]），
+    /// 所以拆成不收 `self` 的一段。
+    fn publish_the_change_raising_the_floor_for_space<Device: BlockDevice>(
+        parameters: &MakeFilesystemParameters,
+        devices: &mut Vec<(DeviceIdentity, Device)>,
+        allocator: &mut PoolAllocator,
+        current: &mut PoolVersion,
+        instance: InstanceGeneration,
+        shadow_ledger: ShadowLedger,
+        change: UserChange<'_>,
+    ) -> Result<UserChangePublished, UserChangeRefused> {
+        let current_version_of_trees = match &*current {
+            PoolVersion::WithFile(version_of_trees)
+            | PoolVersion::WithTreesAndNoFile(version_of_trees) => version_of_trees,
+            PoolVersion::WithoutFile(_) => {
+                return Err(no_file_version_to_change(PublishedInThisChange {
+                    floor_raises: Vec::new(),
+                    refusals_that_pushed_the_floor_raises: Vec::new(),
+                    versions_published: Vec::new(),
+                }))
+            }
+        };
         let mut published_in_this_change = PublishedInThisChange {
             floor_raises: Vec::new(),
             refusals_that_pushed_the_floor_raises: Vec::new(),
@@ -593,8 +991,9 @@ impl MountedSession {
                 devices,
                 allocator,
                 &mut walked,
-                *instance,
-                *shadow_ledger,
+                instance,
+                shadow_ledger,
+                RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),
                 published_in_this_change,
             );
             *current = PoolVersion::of_a_version_of_trees(walked);
@@ -624,7 +1023,7 @@ impl MountedSession {
                 allocator,
                 version_to_change,
                 change,
-                *instance,
+                instance,
             ) {
                 Ok(published) => {
                     // unlink：意图发成之后接着把全部批走完（一次调用 = 走空 + 意图 + 全部批）；别的改动之后队列本来就空，一个写都不发。
@@ -637,8 +1036,9 @@ impl MountedSession {
                         devices,
                         allocator,
                         &mut published,
-                        *instance,
-                        *shadow_ledger,
+                        instance,
+                        shadow_ledger,
+                        RemainderOfTheUserChange::WalkTheQueue,
                         published_in_this_change,
                     );
                     *current = PoolVersion::of_a_version_of_trees(published);
@@ -662,7 +1062,9 @@ impl MountedSession {
                     devices,
                     allocator,
                     version_to_change,
-                    *shadow_ledger,
+                    instance,
+                    shadow_ledger,
+                    RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),
                     publishes_pushed,
                     published_in_this_change,
                     refusal,
@@ -676,17 +1078,21 @@ impl MountedSession {
     /// 直到这个进程写掉 C 那一圈还没解开、记着落点的每一槽（R 那一槽最后轮到），宽护随之解开（`allocator::PoolAllocator::record_root_written_by_this_process`
     /// 摘下那一槽记着的落点、清掉别处不再护着的隔离位）。要几次见 [`StuckBehindTheWideProtection::empty_publishes_until_the_ring_slot_of_r_is_written`]，至多环长 3S。
     ///
-    /// **调用即确认丢掉最近可退到的状态**：R 那一槽是环里最后轮到的一槽，推到它要把根环里更早的回退候选整环写掉，换上的是同一个用户状态的
-    /// 空发布根；推完之后回退候选集可以掉到少于 4 个不同状态，D16（发布语义） 已定项 1「回退候选集保留最近 4 个可退到的不同状态」只在调这个入口时破
-    /// （第 10z 批调查报告 10z-p 第六节第 1 条：384 槽盘那一形推完只剩 2 个）。不自动调：准入被宽护卡住时只报 [`StuckBehindTheWideProtection`]、写明这一条出路。
+    /// **调用即确认丢掉最近可退到的状态**：R 那一槽是 C 那一圈里最后轮到的一槽，推到它要写掉根环里从现行那一版起接下来轮到的那几槽（次数就是槽数，
+    /// 不超过一环），那几槽里原来的根（回退候选）换成同一个用户状态的空发布根；推完之后回退候选集可以掉到少于 4 个不同状态，
+    /// D16（发布语义） 已定项 1「回退候选集保留最近 4 个可退到的不同状态」只在调这个入口时破
+    /// （第 10z 批调查报告 10z-p 第六节第 1 条：384 槽盘那一形推完只剩 2 个）。不自动调：准入被宽护卡住时只报 [`StuckBehindTheWideProtection`]，
+    /// 预演过推完之后那次改动放得下才写明这一条出路。
     ///
+    /// 第一道看分配器上冻结着没重发的发布，冻结着就拒（`PublishFrozenAfterAWriteFailureIsNotResentYet`），在核盘表与预演之前。
+    /// C 那一圈记着落点、而它们罩着的槽这一刻一个都没隔离时不推（写掉那几槽放不出东西，`NothingHeldUntilThisProcessWritesARingSlot`），在任何写之前。
     /// 整串先在分配器的一份拷贝上预演（发布路径落盘之前那一段，与抬 F 那一串的预演同一个做法）：推到半路空发布取不到落点（没挡的槽不够这一串）
     /// 时一个写都不发，盘上逐字节不变、分配器与会话不动。预演过了才真发；真发时有一次发不出去，前面那几次已落盘、`current` 停在最后一次落盘的那一版上。
     /// 这些空发布不过准入两判（没有普通分配，D28（挂载期承诺量） 已定项 1），只受「取得到落点」约束。交进来的盘表照抬 F 那一串的
     /// 空发布同一套核（`mount::parameters_of_the_pool_agreeing_with_the_caller_inputs`），在任何写之前；写入口只按盘上那一份参数建。
     ///
     /// # Errors
-    /// [`RootRingPushRefused`] 的五种。
+    /// [`RootRingPushRefused`] 的六种。
     pub fn push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states<
         Device: BlockDevice,
     >(
@@ -700,6 +1106,37 @@ impl MountedSession {
             shadow_ledger: _,
             parameters_and_device_table,
         } = self;
+        Self::push_the_root_ring_on(
+            parameters_and_device_table.parameters_on_disk(),
+            devices,
+            allocator,
+            current,
+            *instance,
+        )
+    }
+
+    /// [`Self::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states`] 的本体，盘上择到的参数由调用方给：
+    /// 「被宽护卡住」那一报的预演在只进内存的盘与分配器的拷贝上走同一段（[`rehearse_the_push_then_the_remainder_of_the_change`]）。
+    ///
+    /// # Errors
+    /// 同那个入口。
+    fn push_the_root_ring_on<Device: BlockDevice>(
+        parameters_on_disk: &MakeFilesystemParameters,
+        devices: &mut Vec<(DeviceIdentity, Device)>,
+        allocator: &mut PoolAllocator,
+        current: &mut PoolVersion,
+        instance: InstanceGeneration,
+    ) -> Result<RootRingPushed, RootRingPushRefused> {
+        // D23（journal 的角色与格式） 已定项 14「这一版的失败处置」：冻结着的那次发布等原样重发。下面的整串预演走的是发布路径落盘之前那一段、
+        // 不判冻结，在冻结着的分配器拷贝上照样过，真发第一次才被拒（第 10z 批代码轮第一轮攻方第二节「一处报法不准」）。
+        if let Some(frozen) = allocator.frozen_publish() {
+            return Err(
+                RootRingPushRefused::PublishFrozenAfterAWriteFailureIsNotResentYet {
+                    checkpoint_txg: frozen.checkpoint_txg(),
+                    instance: frozen.instance(),
+                },
+            );
+        }
         let start = match &*current {
             PoolVersion::WithFile(version_of_trees)
             | PoolVersion::WithTreesAndNoFile(version_of_trees) => version_of_trees.clone(),
@@ -708,7 +1145,7 @@ impl MountedSession {
             }
         };
         let parameters_of_the_pool = parameters_of_the_pool_agreeing_with_the_caller_inputs(
-            parameters_and_device_table.parameters_on_disk(),
+            parameters_on_disk,
             devices,
             &start,
         )
@@ -717,6 +1154,16 @@ impl MountedSession {
         let Some(target) = root_ring_push_target(allocator, start.root.checkpoint_txg) else {
             return Err(RootRingPushRefused::NothingHeldUntilThisProcessWritesARingSlot);
         };
+        // C 那一圈记着落点、它们罩着的槽却一个都没隔离（全在现行账里仍分配、或已释放还在 defer 窗口里）：写掉那几槽放不出东西，
+        // 与没有记着落点的槽同一个结局（判决 `research/prompts/m3-batch10z-pqrs-code-r1-main-verification.md` 第二节「K1-1 三个量」那一行，正推注入级那一形）。
+        let slots_the_push_would_release_isolation_of =
+            allocator.isolated_slots_covered_by(&target.placements);
+        if slots_the_push_would_release_isolation_of
+            .iter()
+            .all(|(_, isolated)| *isolated == 0)
+        {
+            return Err(RootRingPushRefused::NothingHeldUntilThisProcessWritesARingSlot);
+        }
         let publishes_in_the_push =
             usize::try_from(target.publishes_until_the_ring_slot_of_r_is_written)
                 .expect("至多环长 3S ≤ 48 次");
@@ -729,7 +1176,7 @@ impl MountedSession {
                 let (_, next) = prepare_the_version_publish(
                     &rehearsal_writer,
                     &mut copy,
-                    &empty_publish_plan_after_file_version(&rehearsed, *instance),
+                    &empty_publish_plan_after_file_version(&rehearsed, instance),
                     Some(&rehearsed),
                     rehearsed.tree_identifiers,
                     ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
@@ -754,7 +1201,7 @@ impl MountedSession {
             match publish_version(
                 &mut writer,
                 allocator,
-                empty_publish_plan_after_file_version(&last_published, *instance),
+                empty_publish_plan_after_file_version(&last_published, instance),
                 Some(&last_published),
             ) {
                 Ok(next) => {
@@ -818,18 +1265,21 @@ struct PublishedInThisChange {
 /// 一次发布被拒之后会话怎么接（D16（发布语义） 已定项 1「准入」那一行）：不是空间不够的那两种（[`refusal_is_short_of_space`]）原样交回 `Publish`；
 /// 是就为被拒的那一次发布推一串抬 F 的空发布（这次准入已推 `publishes_pushed_in_this_admission` 次，预算照
 /// `mount::push_one_floor_raise_within_the_admission_budget`），推成交回记上这一串之后的 `published_in_this_change` 与这一串推了几次空发布，
-/// 不再推交回 `NoSpaceAfterRaisingTheFloor`，抬 F 自己报错交回 `FloorRaiseFailedWhilePushingForSpace`。
+/// 不再推交回 `NoSpaceAfterRaisingTheFloor`（报不报「被宽护卡住」、预演推环之后这次改动剩下的 `remainder` 放不放得下，
+/// [`wide_protection_at_the_ceiling`]），抬 F 自己报错交回 `FloorRaiseFailedWhilePushingForSpace`。
 /// 推的那一串就地推进 `current`（被拒的那一次要接在它后面的那一版）。
 #[allow(
     clippy::too_many_arguments,
-    reason = "参数各有名字与类型：写入口要的参数与盘表、分配器、现行那一版、影子账那一臂、预算已用的数、已落盘的、这一次被拒，不成一个概念"
+    reason = "参数各有名字与类型：写入口要的参数与盘表、分配器、现行那一版、实例代号、影子账那一臂、这次改动还剩什么、预算已用的数、已落盘的、这一次被拒，不成一个概念"
 )]
 fn push_one_floor_raise_for_a_refusal_for_space<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
     allocator: &mut PoolAllocator,
     current: &mut TransactionOutput,
+    instance: InstanceGeneration,
     shadow_ledger: ShadowLedger,
+    remainder: RemainderOfTheUserChange<'_>,
     publishes_pushed_in_this_admission: usize,
     published_in_this_change: PublishedInThisChange,
     refusal: PublishError,
@@ -874,7 +1324,14 @@ fn push_one_floor_raise_for_a_refusal_for_space<Device: BlockDevice>(
                 refusals_that_pushed_the_floor_raises,
                 // 先借 `stop` 判这一形，再把它挪进下一个字段（字段按写的次序求值）。
                 wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(
-                    allocator, current, &stop,
+                    parameters,
+                    devices.as_slice(),
+                    allocator,
+                    current,
+                    instance,
+                    shadow_ledger,
+                    remainder,
+                    &stop,
                 ),
                 stop,
                 versions_published,
@@ -897,7 +1354,12 @@ fn push_one_floor_raise_for_a_refusal_for_space<Device: BlockDevice>(
 /// 被它以空间拒时推抬 F 腾得出槽）；推满仍放不下、抬 F 自己报错、被拒不是空间不够，交回会话要报的那一种拒绝。
 /// 每一次被拒的删除类发布各算一次准入（与 unlink 意图那次、别的用户改动那次一样，预算数的是为被判的那一次发布推了几次空发布）：
 /// 这一趟走空有一步落了盘，下一次被拒的就是后面那一次发布，这次准入已推的数从 0 数起。`current` 停在最后一次落盘的那一版上
-/// （删除那一步或推的空发布）；发成的每一步记进 `versions_published`。
+/// （删除那一步或推的空发布）；发成的每一步记进 `versions_published`。`remainder` 是走空之后这次改动还剩什么（推满仍不够、被宽护卡住时
+/// 预演推环之后照它再发一次，[`wide_protection_at_the_ceiling`]）。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各有名字与类型：写入口要的参数与盘表、分配器、现行那一版、实例代号、影子账那一臂、这次改动还剩什么、已落盘的，不成一个概念"
+)]
 fn publish_the_queue_until_it_is_empty_raising_the_floor_for_space<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
@@ -905,6 +1367,7 @@ fn publish_the_queue_until_it_is_empty_raising_the_floor_for_space<Device: Block
     current: &mut TransactionOutput,
     instance: InstanceGeneration,
     shadow_ledger: ShadowLedger,
+    remainder: RemainderOfTheUserChange<'_>,
     published_in_this_change: PublishedInThisChange,
 ) -> Result<PublishedInThisChange, UserChangeRefused> {
     let mut published_in_this_change = published_in_this_change;
@@ -939,7 +1402,9 @@ fn publish_the_queue_until_it_is_empty_raising_the_floor_for_space<Device: Block
                 devices,
                 allocator,
                 current,
+                instance,
                 shadow_ledger,
+                remainder,
                 publishes_pushed_in_this_admission,
                 published_in_this_change,
                 refusal,
diff --git a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
index a93d70d..3d85bc5 100644
--- a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
+++ b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
@@ -33,6 +33,12 @@
 //!   第 10z 批第十四块报告第六节那几形卡住时，`NoSpaceAfterRaisingTheFloor` 点名「被宽护卡住」、带 R 的 txg、挡着的槽数与两条出路；
 //!   调推环入口之后 R 那一槽写掉、隔离回 0、之后写放得下；卸载重挂那一条出路也走得通；推到半路取不到落点时在任何写之前拒。
 //! - 被抛弃根那一圈遇到 defer 槽不双扣（第十四块报告第七节第 2 条）：挂载那一刻与根槽读坏时的抬 F 都只隔离分配位没置着的槽，回收时照根环表补。
+//! - 出路按预演定（用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」，判决 `research/prompts/m3-batch10z-pqrs-code-r1-main-verification.md`
+//!   第二、三节；第 10z 批第二十块实现员定形，被攻过零轮）：攻方 K1 那一形（`research/prompts/m3-batch10z-pqrs-code-r1-opus-model/atk_part.rs`）
+//!   推完放得下的写列两条出路、照推环走完放得下，推完放不下的写（含没有 C 那一圈也被拒的长度、实现员三形长 60 与 80 个单元）只列卸载再挂、
+//!   照它走完放得下；出路一按 R 那一槽有没有读得出的有效根分两种说法，出路二写推几次、写掉几槽、放出来的隔离槽数（推完前后之差，两槽 C 重叠时不重数）；
+//!   推环入口冻结着时第一道就拒、不核盘表时盘体对调照推（攻方 M1、M2）、两槽 C 推到较晚的那一槽（攻方 M12）；C 记着落点而一槽都没隔离、
+//!   停因是预算用完或抬 F 取不到落点时不报被宽护卡住，记着空串的槽不算（正推 K4-2 活下来的几个）；根槽读坏的被抛弃根那一圈补满整段跨度。
 // 池（384 槽那一档小盘）与 R 那块盘都经 `tests/common_unwitnessed_publish/mod.rs` 起（它转出 `tests/common_admission_pool/mod.rs` 的全部）：
 // 再声明 `common_admission` 就把 `common_admission_pool` 当模块装两遍（clippy `duplicate_mod`）。
 mod common_unwitnessed_publish;
@@ -67,8 +73,8 @@ use singlefs_core::mount::{
     RollbackTarget, ShadowLedger, StillUnreadableAfterOneReread, Unmounted,
 };
 use singlefs_core::mounted_session::{
-    MountedSession, NoSpaceAfterRaisingTheFloor, RootRingPushRefused, StuckBehindTheWideProtection,
-    UserChangeRefused, WideProtectionAtTheCeiling,
+    MountedSession, NoSpaceAfterRaisingTheFloor, RootRingPushRefused, RootRingPushRehearsal,
+    RootRingPushed, StuckBehindTheWideProtection, UserChangeRefused, WideProtectionAtTheCeiling,
 };
 use singlefs_core::pointer::{LocationEntry, NodePointer};
 use singlefs_core::recovery::{
@@ -3845,18 +3851,31 @@ fn wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
         "{form:?}：宽护挡着的槽数大于 0、不多于两块盘隔离的槽数：{isolated_by_the_wide_protection}"
     );
     let ways_out = stuck.ways_out();
-    assert!(
-        ways_out[0].contains("卸载") && ways_out[0].contains("再可写挂载"),
-        "{form:?}：出路一是卸载重挂：{}",
-        ways_out[0]
+    assert_eq!(
+        ways_out.len(),
+        2,
+        "{form:?}：预演过推完之后这次覆盖写放得下，两条出路都列：{ways_out:?}"
     );
-    assert!(
-        ways_out[1].contains(
-            "push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states"
-        ) && ways_out[1].contains("少于 4 个不同状态"),
-        "{form:?}：出路二是推环入口、写明回退候选集会掉到少于 4 个不同状态：{}",
-        ways_out[1]
+    let why = format!("{form:?}");
+    assert_the_way_out_by_unmounting_names_r_abandoned_and_warns_of_a_permanently_unreadable_account(
+        &ways_out[0],
+        recorded.key.1,
+        &why,
+    );
+    let released_in_the_rehearsal = released_by_the_push_if_the_change_is_admitted_after_it(&stuck)
+        .unwrap_or_else(|| {
+            panic!(
+                "{form:?}：预演推完之后这次覆盖写放得下，而预演的结局是 {:?}",
+                stuck.pushing_the_root_ring
+            )
+        });
+    assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_r_and_the_released_slots(
+        &stuck,
+        &ways_out[1],
+        &released_in_the_rehearsal,
+        &why,
     );
+    let isolated_before_the_push = isolated_slots_on_each_device_of(&pool.session().allocator);
     assert!(
         ring_slots_recorded_as_holding_an_abandoned_root_by(&pool.session().allocator).is_empty(),
         "{form:?}：这几形根环表里没有被抛弃根（隔离全是 C 那一圈的）"
@@ -3895,6 +3914,14 @@ fn wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
         0,
         "{form:?}：推环之后隔离回到不含宽护的数（根环表里没有被抛弃根）"
     );
+    assert_eq!(
+        released_in_the_rehearsal,
+        isolated_slots_released_between(
+            &isolated_before_the_push,
+            &isolated_slots_on_each_device_of(allocator)
+        ),
+        "{form:?}：出路二说放出来的隔离槽数就是推完前后每块盘隔离数之差"
+    );
     assert_eq!(
         slots_counted_both_as_allocated_and_as_isolated(allocator),
         0,
@@ -4507,3 +4534,1460 @@ fn defer_slots_an_abandoned_root_whose_ring_slot_reads_bad_references_are_not_co
     );
     assert_none_of_the_placements_is_free(allocator, &recorded_for_c, "C 记着");
 }
+
+// ---------------------------------------------------------------------------------------------------------------------------
+// 第 10z 批第二十块（用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」；判决
+// `research/prompts/m3-batch10z-pqrs-code-r1-main-verification.md` 第二、三节）：出路按预演定、推环入口先判冻结、出路的说法、
+// 推环入口与「被宽护卡住」那一报在 C 有落点而无隔离槽时的结局，与拦住改回去的几条。
+
+/// 推一次环（会话的推环入口），做成时把推出来的每条根记成现行内容。
+fn push_the_root_ring_of<Device: DeviceOverASparseImage>(
+    pool: &mut PoolUnderTest<Device>,
+) -> Result<RootRingPushed, RootRingPushRefused> {
+    let mut session = pool.session.take().expect("挂着的会话");
+    let pushed = session
+        .push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states(
+            &mut pool.devices,
+        );
+    pool.session = Some(session);
+    if let Ok(pushed) = &pushed {
+        for version in &pushed.versions_published {
+            pool.note_root_of_the_current_content(&version.root);
+        }
+    }
+    pushed
+}
+
+/// 每块盘隔离的槽数（按分配器里盘的次序）。
+fn isolated_slots_on_each_device_of(allocator: &PoolAllocator) -> Vec<(DeviceIdentity, u64)> {
+    allocator
+        .devices
+        .iter()
+        .map(|device_map| (device_map.device, device_map.isolated_slots()))
+        .collect()
+}
+
+/// 推环前后每块盘隔离的槽数少了几个（推完多了就当场判红：推环只写掉 C 那一圈的槽、放隔离）。
+fn isolated_slots_released_between(
+    before: &[(DeviceIdentity, u64)],
+    after: &[(DeviceIdentity, u64)],
+) -> Vec<(DeviceIdentity, u64)> {
+    before
+        .iter()
+        .zip(after)
+        .map(|((device, isolated_before), (device_after, isolated_after))| {
+            assert_eq!(device, device_after, "前后数的是同一个分配器：盘的次序不变");
+            (
+                *device,
+                isolated_before.checked_sub(*isolated_after).unwrap_or_else(|| {
+                    panic!("盘 {} 推完隔离的槽数（{isolated_after}）不多于推之前（{isolated_before}）", device.0)
+                }),
+            )
+        })
+        .collect()
+}
+
+/// 出路里推环那一条预演的结局是「推完放得下」时，交回预演里每块盘放出来的隔离槽数；别的结局交 `None`。
+fn released_by_the_push_if_the_change_is_admitted_after_it(
+    stuck: &StuckBehindTheWideProtection,
+) -> Option<Vec<(DeviceIdentity, u64)>> {
+    match &stuck.pushing_the_root_ring {
+        RootRingPushRehearsal::ChangeAdmittedAfterThePush {
+            isolated_slots_released_by_the_push_on_each_device,
+        } => Some(isolated_slots_released_by_the_push_on_each_device.clone()),
+        RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. }
+        | RootRingPushRehearsal::PushRefused(_) => None,
+    }
+}
+
+/// 出路一那句话在 R 那一槽记着有效根时的说法：R（带 txg）再挂时判成被抛弃；R 的账读不出若是盘上永久坏，卸载之后再挂会被拒、会话已断开。
+fn assert_the_way_out_by_unmounting_names_r_abandoned_and_warns_of_a_permanently_unreadable_account(
+    way_out_by_unmounting: &str,
+    checkpoint_txg_of_r: CheckpointTxg,
+    why: &str,
+) {
+    for expected in [
+        "正常卸载".to_string(),
+        "再可写挂载".to_string(),
+        format!(
+            "R（txg {}）按新实例的实例表判成被抛弃",
+            checkpoint_txg_of_r.0
+        ),
+        "R 的账读不出若是盘上永久坏，卸载之后再挂会被拒".to_string(),
+        "会话在卸载时已断开".to_string(),
+    ] {
+        assert!(
+            way_out_by_unmounting.contains(&expected),
+            "{why}：出路一写明「{expected}」：{way_out_by_unmounting}"
+        );
+    }
+}
+
+/// 出路一那句话在 R 那一槽没有读得出的有效根时的说法：点名那一槽、说再挂之后 C 那一圈不再有这一槽，不说 R 判成被抛弃
+/// （攻方 `research/prompts/m3-batch10z-pqrs-code-r1-opus-output.md` 第一节末：实测卸载再挂之后 C 那一圈直接没了）。
+fn assert_the_way_out_by_unmounting_says_the_ring_slot_of_r_holds_no_readable_root(
+    way_out_by_unmounting: &str,
+    stuck: &StuckBehindTheWideProtection,
+    why: &str,
+) {
+    for expected in [
+        "正常卸载".to_string(),
+        "再可写挂载".to_string(),
+        format!(
+            "R 那一槽（根环槽 {}.{}）没有读得出的有效根",
+            stuck.ring_slot_of_r.region, stuck.ring_slot_of_r.slot
+        ),
+        "C 那一圈照它重算、不再有这一槽".to_string(),
+    ] {
+        assert!(
+            way_out_by_unmounting.contains(&expected),
+            "{why}：出路一写明「{expected}」：{way_out_by_unmounting}"
+        );
+    }
+    assert!(
+        !way_out_by_unmounting.contains("判成被抛弃"),
+        "{why}：R 没有根，出路一不说它判成被抛弃：{way_out_by_unmounting}"
+    );
+}
+
+/// 出路二那句话：推环入口的名字、推几次、写掉环里几槽、R 那一槽的槽号、放出来的隔离槽数（总数与逐盘，与预演的数相同）、回退候选的代价，
+/// 不写「整环」。
+fn assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_r_and_the_released_slots(
+    stuck: &StuckBehindTheWideProtection,
+    way_out_by_the_push: &str,
+    released_on_each_device: &[(DeviceIdentity, u64)],
+    why: &str,
+) {
+    let publishes = stuck.empty_publishes_until_the_ring_slot_of_r_is_written;
+    let released: u64 = released_on_each_device
+        .iter()
+        .map(|(_, released_on_the_device)| *released_on_the_device)
+        .sum();
+    let mut expected_phrases = vec![
+        "push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states"
+            .to_string(),
+        format!("发 {publishes} 次 F 不变的空发布"),
+        format!("写掉根环里接下来轮到的 {publishes} 槽"),
+        format!(
+            "根环槽 {}.{}",
+            stuck.ring_slot_of_r.region, stuck.ring_slot_of_r.slot
+        ),
+        format!("隔离的槽放出来 {released} 个"),
+        "少于 4 个不同状态".to_string(),
+    ];
+    expected_phrases.extend(released_on_each_device.iter().map(
+        |(device, released_on_the_device)| {
+            format!("盘 {} 上 {released_on_the_device} 个", device.0)
+        },
+    ));
+    for expected in &expected_phrases {
+        assert!(
+            way_out_by_the_push.contains(expected.as_str()),
+            "{why}：出路二写明「{expected}」：{way_out_by_the_push}"
+        );
+    }
+    assert!(
+        !way_out_by_the_push.contains("整环"),
+        "{why}：出路二写推几次、写掉几槽，不写「整环」：{way_out_by_the_push}"
+    );
+}
+
+/// 一次写被拒、报「被宽护卡住」：这次调用里推了几串抬 F、发成了几版，与报的那一份。
+struct WriteStuckBehindTheWideProtection {
+    floor_raises_pushed: usize,
+    versions_published: usize,
+    stuck: StuckBehindTheWideProtection,
+}
+
+/// 被拒是推满仍不够、停在上限、报「被宽护卡住」（`why` 只用在断言消息里）。
+fn stuck_behind_the_wide_protection_reported_by(
+    refusal: UserChangeRefused,
+    why: &str,
+) -> WriteStuckBehindTheWideProtection {
+    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
+        panic!("{why}：被拒是推满仍不够：{refusal:?}");
+    };
+    assert!(
+        matches!(
+            no_space.stop,
+            FloorRaiseStop::FloorAlreadyAtTheCeiling { .. }
+        ),
+        "{why}：推抬 F 停在上限：{:?}",
+        no_space.stop
+    );
+    let NoSpaceAfterRaisingTheFloor {
+        floor_raises,
+        versions_published,
+        wide_protection_at_the_ceiling,
+        ..
+    } = *no_space;
+    let WideProtectionAtTheCeiling::StuckBehindTheWideProtection(stuck) =
+        wide_protection_at_the_ceiling
+    else {
+        panic!("{why}：F 在上限、C 那一圈还隔离着槽：报「被宽护卡住」，而报的是 {wide_protection_at_the_ceiling:?}");
+    };
+    WriteStuckBehindTheWideProtection {
+        floor_raises_pushed: floor_raises.len(),
+        versions_published: versions_published.len(),
+        stuck: *stuck,
+    }
+}
+
+/// 攻方 K1 那一形（`research/prompts/m3-batch10z-pqrs-code-r1-opus-model/atk_part.rs` 的 `atk_k1_record`，`atk_k1_push_trace` 的 pre = 4、fill = 16）：
+/// 第一个文件之后前缀覆盖写几次，文件每次长几个数据单元、长到几个，再录一次长几个单元的顺序写 R。
+const OVERWRITES_BEFORE_GROWING_THE_FILE_UNTIL_THE_DEVICES_ARE_NEARLY_FULL: u64 = 4;
+const DATA_UNITS_THE_FILE_GROWS_BY_EACH_TIME_BEFORE_R: usize = 5;
+const DATA_UNITS_THE_FILE_GROWS_TO_BEFORE_R: usize = 16;
+const DATA_UNITS_R_GROWS_THE_FILE_BY: usize = 3;
+
+/// 攻方 K1 那一形崩在 R 第一条记录之后的盘面（不带任何读故障的自然崩溃点：可写挂载择 P，R 那几条记录在水位之上、没施加，
+/// C 那一圈只有 R 那一槽，R 那一槽没有根）。
+struct PoolCrashedAfterTheFirstRecordOfALongSequentialWrite {
+    crashed: MemoryPool,
+    content_of_each_root: BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>,
+    write_time_seconds: u64,
+    data_units_of_the_file: usize,
+}
+
+fn pool_crashed_after_the_first_record_of_a_long_sequential_write(
+) -> PoolCrashedAfterTheFirstRecordOfALongSequentialWrite {
+    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
+    let stream = SharedStream::retaining_contents();
+    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
+        parameters_of_the_384_slot_small_devices(),
+        device_bytes_of_the_384_slot_small_devices(),
+        |identity, device| {
+            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
+        },
+    );
+    for overwrite_number in 1..=OVERWRITES_BEFORE_GROWING_THE_FILE_UNTIL_THE_DEVICES_ARE_NEARLY_FULL
+    {
+        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
+            panic!("前缀第 {overwrite_number} 次覆盖写落得下：{refusal:?}")
+        });
+    }
+    let mut data_units_of_the_file = 1;
+    // 迭代上界是长到的单元数；跨轮带着文件现在几个单元；出口：再长一步就越过长到的单元数。
+    while data_units_of_the_file + DATA_UNITS_THE_FILE_GROWS_BY_EACH_TIME_BEFORE_R
+        <= DATA_UNITS_THE_FILE_GROWS_TO_BEFORE_R
+    {
+        data_units_of_the_file += DATA_UNITS_THE_FILE_GROWS_BY_EACH_TIME_BEFORE_R;
+        pool.sequential_write(data_units_of_the_file * unit_bytes)
+            .unwrap_or_else(|refusal| {
+                panic!("文件长到 {data_units_of_the_file} 个单元落得下：{refusal:?}")
+            });
+    }
+    assert_eq!(
+        data_units_of_the_file, DATA_UNITS_THE_FILE_GROWS_TO_BEFORE_R,
+        "文件长到攻方那一形的单元数"
+    );
+    let base = pool.image();
+    let operations_before_r = stream.operation_count();
+    let published_r = pool
+        .sequential_write((data_units_of_the_file + DATA_UNITS_R_GROWS_THE_FILE_BY) * unit_bytes)
+        .unwrap_or_else(|refusal| panic!("录的那次顺序写（R）落得下：{refusal:?}"));
+    assert!(
+        published_r.floor_raises.is_empty(),
+        "R 那次没推抬 F：录下来的写全是 R 自己的"
+    );
+    let operations = stream.retained_operations();
+    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
+        &operations[operations_before_r..],
+        &fixed_geometry_of_the_384_slot_small_devices(),
+    );
+    let end_of_the_first_record = writes
+        .iter()
+        .position(|write| write.kind == StepKind::JournalRecord)
+        .expect("R 写了记录")
+        + 1;
+    let mut crashed = base;
+    crashed.apply_writes(&writes[..end_of_the_first_record]);
+    PoolCrashedAfterTheFirstRecordOfALongSequentialWrite {
+        crashed,
+        content_of_each_root: pool.content_of_each_root.clone(),
+        write_time_seconds: pool.write_time_seconds + 100,
+        data_units_of_the_file,
+    }
+}
+
+impl PoolCrashedAfterTheFirstRecordOfALongSequentialWrite {
+    /// 拿崩溃那一刻的盘面起两块新盘、可写挂载一次（空间准入判着）。
+    fn mounted(&self) -> PoolUnderTest<SparseBlockDevice> {
+        let mut pool = PoolUnderTest {
+            parameters: parameters_of_the_384_slot_small_devices(),
+            device_bytes: device_bytes_of_the_384_slot_small_devices(),
+            devices: plain_devices_on(&self.crashed),
+            session: None,
+            write_time_seconds: self.write_time_seconds,
+            content_of_the_current_version: Vec::new(),
+            content_of_each_root: self.content_of_each_root.clone(),
+        };
+        pool.crash_and_mount_writable()
+            .expect("崩在 R 第一条记录之后：可写挂载择 P、做得成");
+        let holding = pool
+            .session()
+            .allocator
+            .root_ring_occupancy()
+            .expect("挂载建的分配器装着根环表")
+            .ring_slots_holding_placements_named_by_unapplied_journal_records();
+        assert_eq!(
+            holding.len(),
+            1,
+            "C 那一圈只有 R 那一槽（阳性对照）：{:?}",
+            holding.keys().collect::<Vec<_>>()
+        );
+        pool
+    }
+
+    /// 挂上之后写一次长 `data_units_grown_by` 个单元的顺序写：被拒、报「被宽护卡住」（这一次调用里会话先推一串抬 F、把 F 抬到上限）。
+    /// 交回被拒之后挂着的池：再试同一个写时 F 已在上限，那一次调用一个写都不推。
+    fn mounted_and_stuck_once_by_a_write_grown_by(
+        &self,
+        data_units_grown_by: usize,
+    ) -> PoolUnderTest<SparseBlockDevice> {
+        let mut pool = self.mounted();
+        let refusal = pool
+            .sequential_write(self.bytes_of_the_file_grown_by(data_units_grown_by))
+            .expect_err("挂上之后第一次长文件：宽护挡着准入");
+        stuck_behind_the_wide_protection_reported_by(
+            refusal,
+            &format!("挂上之后第一次长 {data_units_grown_by} 个单元"),
+        );
+        pool
+    }
+
+    /// 把文件长到比 P 多 `data_units` 个数据单元的顺序写的字节数（攻方的 g）。
+    fn bytes_of_the_file_grown_by(&self, data_units: usize) -> usize {
+        (self.data_units_of_the_file + data_units)
+            * singlefs_core::unit::data_unit_payload_capacity()
+    }
+}
+
+/// 攻方 K1 那一形里长到这几个单元的写：宽护挡着、推完放得下的那一档（攻方第一节：g ≤ 19 推完第一次就放得下）；推完照旧被拒的那一档（g = 20）；
+/// 没有 C 那一圈的对照臂同一个长度也被拒的那一档（g = 36，对照臂 g = 36 起被拒）。
+const DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY: usize = 13;
+const DATA_UNITS_A_WRITE_STILL_REFUSED_AFTER_THE_PUSH_GROWS_THE_FILE_BY: usize = 20;
+const DATA_UNITS_A_WRITE_REFUSED_EVEN_WITHOUT_THE_WIDE_PROTECTION_GROWS_THE_FILE_BY: usize = 36;
+
+/// 甲（攻方 K1 那一形 g = 13）：报「被宽护卡住」之前预演推环与推完之后这次改动、推完放得下——两条出路都列：出路一说 R 那一槽没有读得出的有效根，
+/// 出路二写明推几次、写掉几槽、R 那一槽、放出来的隔离槽数。挂上之后第一次被拒时会话推了一串抬 F、把 F 抬到上限；再试同一个写，
+/// 那一次调用一个写都不推，判的就是预演：一个写都不发、会话的分配器不动。照推环走完，放出来的隔离槽数就是推完前后
+/// 每块盘隔离数之差，同一个写放得下，池级 checker 不红 I-7.4、I-4.8。
+#[test]
+fn a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push(
+) {
+    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
+    let mut pool = crashed.mounted_and_stuck_once_by_a_write_grown_by(
+        DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY,
+    );
+    let length = crashed
+        .bytes_of_the_file_grown_by(DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY);
+    let image_before_the_write = pool.image();
+    let allocator_before_the_write = format!("{:?}", pool.session().allocator);
+    let refusal = pool
+        .sequential_write(length)
+        .expect_err("长 13 个单元再试：宽护照旧挡着准入");
+    let WriteStuckBehindTheWideProtection {
+        floor_raises_pushed,
+        versions_published,
+        stuck,
+    } = stuck_behind_the_wide_protection_reported_by(refusal, "长 13 个单元再试");
+    assert_eq!(
+        (floor_raises_pushed, versions_published),
+        (0, 0),
+        "第一次被拒时已把 F 抬到上限：再试这一次一串抬 F 都没推、一版都没发（阳性对照：下面两句判的只剩预演）"
+    );
+    assert!(
+        pool.image() == image_before_the_write,
+        "预演推环与推完之后这次改动：一个写都不发，两块盘逐字节不变"
+    );
+    assert_eq!(
+        format!("{:?}", pool.session().allocator),
+        allocator_before_the_write,
+        "预演在分配器的拷贝上：会话的分配器整份不变"
+    );
+    assert_eq!(
+        stuck.checkpoint_txg_of_r, None,
+        "崩在 R 第一条记录之后：R 那一槽没有读得出的有效根（阳性对照：出路一要说的是这一种）"
+    );
+    let ways_out = stuck.ways_out();
+    assert_eq!(ways_out.len(), 2, "推完放得下：两条出路都列：{ways_out:?}");
+    assert_the_way_out_by_unmounting_says_the_ring_slot_of_r_holds_no_readable_root(
+        &ways_out[0],
+        &stuck,
+        "长 13 个单元",
+    );
+    let released_in_the_rehearsal = released_by_the_push_if_the_change_is_admitted_after_it(&stuck)
+        .unwrap_or_else(|| {
+            panic!(
+                "预演推完之后这次改动放得下，而预演的结局是 {:?}",
+                stuck.pushing_the_root_ring
+            )
+        });
+    assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_r_and_the_released_slots(
+        &stuck,
+        &ways_out[1],
+        &released_in_the_rehearsal,
+        "长 13 个单元",
+    );
+    let isolated_before_the_push = isolated_slots_on_each_device_of(&pool.session().allocator);
+    let pushed = push_the_root_ring_of(&mut pool)
+        .unwrap_or_else(|push_refusal| panic!("照出路二推环做得成：{push_refusal:?}"));
+    assert_eq!(
+        u64::try_from(pushed.versions_published.len()).expect("次数"),
+        stuck.empty_publishes_until_the_ring_slot_of_r_is_written,
+        "推环发的空发布次数就是报里写的那个数"
+    );
+    let released_by_the_push = isolated_slots_released_between(
+        &isolated_before_the_push,
+        &isolated_slots_on_each_device_of(&pool.session().allocator),
+    );
+    assert_eq!(
+        released_in_the_rehearsal, released_by_the_push,
+        "出路二说放出来的隔离槽数就是推完前后每块盘隔离数之差"
+    );
+    pool.sequential_write(length)
+        .unwrap_or_else(|refusal_after_the_push| {
+            panic!("推完之后同一个写放得下：{refusal_after_the_push:?}")
+        });
+    let violations = violations_of_unit_reuse_on(&pool.image());
+    assert!(
+        violations.is_empty(),
+        "推完写下之后 I-7.4、I-4.8 不红：{violations:?}"
+    );
+}
+
+/// 甲（攻方 K1 那一形）：推完照旧放不下的写，报「被宽护卡住」时只列卸载再挂（预演推完之后照旧被拒），照它走：正常卸载、再可写挂载，
+/// 同一个写放得下，池级 checker 不红 I-7.4、I-4.8。
+fn a_write_stuck_behind_the_wide_protection_the_push_does_not_let_through_lists_only_unmounting_which_lets_it_through(
+    data_units_grown_by: usize,
+) {
+    let why = format!("长 {data_units_grown_by} 个单元");
+    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
+    let mut pool = crashed.mounted();
+    let length = crashed.bytes_of_the_file_grown_by(data_units_grown_by);
+    let refusal = pool.sequential_write(length).expect_err("宽护挡着准入");
+    let WriteStuckBehindTheWideProtection { stuck, .. } =
+        stuck_behind_the_wide_protection_reported_by(refusal, &why);
+    assert!(
+        matches!(
+            stuck.pushing_the_root_ring,
+            RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. }
+        ),
+        "{why}：预演推环做得成、推完之后这次改动照旧被拒：{:?}",
+        stuck.pushing_the_root_ring
+    );
+    let ways_out = stuck.ways_out();
+    assert_eq!(
+        ways_out.len(),
+        1,
+        "{why}：推完放不下，只列卸载再挂：{ways_out:?}"
+    );
+    assert_the_way_out_by_unmounting_says_the_ring_slot_of_r_holds_no_readable_root(
+        &ways_out[0],
+        &stuck,
+        &why,
+    );
+    let mut session = pool.session.take().expect("挂着的会话");
+    let unmounted = unmount(
+        &pool.parameters,
+        &mut pool.devices,
+        &mut session.allocator,
+        &mut session.current,
+        ShadowLedger::On,
+    );
+    match unmounted {
+        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
+            for version in &raised.publishes {
+                pool.note_root_of_the_current_content(&version.root);
+            }
+        }
+        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
+            panic!("{why}：现行那一版带文件，卸载要推一串抬 F：{current:?}")
+        }
+        Err(unmount_refusal) => panic!("{why}：照出路一正常卸载做得成：{unmount_refusal:?}"),
+    }
+    pool.crash_and_mount_writable()
+        .unwrap_or_else(|mount_refusal| {
+            panic!("{why}：卸载之后再可写挂载做得成：{mount_refusal:?}")
+        });
+    pool.sequential_write(length)
+        .unwrap_or_else(|refusal_after_mounting_again| {
+            panic!("{why}：卸载再挂之后同一个写放得下：{refusal_after_mounting_again:?}")
+        });
+    let violations = violations_of_unit_reuse_on(&pool.image());
+    assert!(
+        violations.is_empty(),
+        "{why}：卸载再挂写下之后 I-7.4、I-4.8 不红：{violations:?}"
+    );
+}
+
+/// 攻方 K1 那一形 g = 20：宽护确实挡着（对照臂同一个长度放得下），推完之后上限退到环里最旧的根，照旧被拒。
+#[test]
+fn a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through(
+) {
+    a_write_stuck_behind_the_wide_protection_the_push_does_not_let_through_lists_only_unmounting_which_lets_it_through(
+        DATA_UNITS_A_WRITE_STILL_REFUSED_AFTER_THE_PUSH_GROWS_THE_FILE_BY,
+    );
+}
+
+/// 攻方 K1 那一形 g = 36：没有 C 那一圈的对照臂同一个长度也被拒（宽护是挡着的之一、不是唯一），照样报「被宽护卡住」、只列卸载再挂。
+#[test]
+fn a_write_refused_even_without_the_wide_protection_lists_only_unmounting_which_lets_it_through() {
+    a_write_stuck_behind_the_wide_protection_the_push_does_not_let_through_lists_only_unmounting_which_lets_it_through(
+        DATA_UNITS_A_WRITE_REFUSED_EVEN_WITHOUT_THE_WIDE_PROTECTION_GROWS_THE_FILE_BY,
+    );
+}
+
+/// 甲（攻方第一节「实现员自己的三形上……长到 60、80 个单元的推完照旧被拒」）：实现员三形卡住之后，把文件写成这么多个数据单元的顺序写
+/// 报「被宽护卡住」、只列卸载再挂；R 那一槽记着有效根，出路一照 R 的 txg 说它判成被抛弃、写明账永久读不出时的后果。
+fn a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
+    form: FormStuckBehindTheWideProtectionOnThe384SlotDevices,
+    data_units: usize,
+) {
+    let why = format!("{form:?} 写成 {data_units} 个单元");
+    let PoolStuckBehindTheWideProtection {
+        recorded, mut pool, ..
+    } = pool_stuck_behind_the_wide_protection(form);
+    let refusal = pool
+        .sequential_write(data_units * singlefs_core::unit::data_unit_payload_capacity())
+        .expect_err("宽护挡着准入");
+    let WriteStuckBehindTheWideProtection { stuck, .. } =
+        stuck_behind_the_wide_protection_reported_by(refusal, &why);
+    assert!(
+        matches!(
+            stuck.pushing_the_root_ring,
+            RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. }
+        ),
+        "{why}：预演推环做得成、推完之后这次改动照旧被拒：{:?}",
+        stuck.pushing_the_root_ring
+    );
+    let ways_out = stuck.ways_out();
+    assert_eq!(
+        ways_out.len(),
+        1,
+        "{why}：推完放不下，只列卸载再挂：{ways_out:?}"
+    );
+    assert_eq!(
+        stuck.checkpoint_txg_of_r,
+        Some(recorded.key.1),
+        "{why}：R 那一槽记着录下来的 R（阳性对照：出路一要说的是这一种）"
+    );
+    assert_the_way_out_by_unmounting_names_r_abandoned_and_warns_of_a_permanently_unreadable_account(
+        &ways_out[0],
+        recorded.key.1,
+        &why,
+    );
+}
+
+/// 长 60 个数据单元、80 个数据单元的顺序写（攻方 `atk_k1_implementer_forms_sweep` 推完仍拒的那两档）。
+const DATA_UNITS_OF_A_LONG_WRITE_STILL_REFUSED_AFTER_THE_PUSH: usize = 60;
+const DATA_UNITS_OF_A_LONGER_WRITE_STILL_REFUSED_AFTER_THE_PUSH: usize = 80;
+
+#[test]
+fn a_sixty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting() {
+    a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
+            overwrites_before: 20,
+        },
+        DATA_UNITS_OF_A_LONG_WRITE_STILL_REFUSED_AFTER_THE_PUSH,
+    );
+}
+
+#[test]
+fn an_eighty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting() {
+    a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
+            overwrites_before: 20,
+        },
+        DATA_UNITS_OF_A_LONGER_WRITE_STILL_REFUSED_AFTER_THE_PUSH,
+    );
+}
+
+#[test]
+fn a_sixty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting(
+) {
+    a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
+            overwrites_before: 40,
+        },
+        DATA_UNITS_OF_A_LONG_WRITE_STILL_REFUSED_AFTER_THE_PUSH,
+    );
+}
+
+#[test]
+fn an_eighty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting(
+) {
+    a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
+            overwrites_before: 40,
+        },
+        DATA_UNITS_OF_A_LONGER_WRITE_STILL_REFUSED_AFTER_THE_PUSH,
+    );
+}
+
+#[test]
+fn a_sixty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting() {
+    a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::FirstPublishOfADeletion {
+            overwrites_before: 160,
+        },
+        DATA_UNITS_OF_A_LONG_WRITE_STILL_REFUSED_AFTER_THE_PUSH,
+    );
+}
+
+#[test]
+fn an_eighty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting() {
+    a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::FirstPublishOfADeletion {
+            overwrites_before: 160,
+        },
+        DATA_UNITS_OF_A_LONGER_WRITE_STILL_REFUSED_AFTER_THE_PUSH,
+    );
+}
+
+/// 一份挂着的会话逐字段拷一份（同一刻的分配器、现行那一版、实例代号、影子账那一臂、认下的池）：同一个盘面上起几份池各推各的。
+fn copy_of_the_session(session: &MountedSession) -> MountedSession {
+    MountedSession {
+        allocator: session.allocator.clone(),
+        current: session.current.clone(),
+        instance: session.instance,
+        shadow_ledger: session.shadow_ledger,
+        parameters_and_device_table: session.parameters_and_device_table.clone(),
+    }
+}
+
+/// 整池的改盘调用（写、写零、屏障，几块盘共用一个数）数到第 `failing_call` 次（从 1 数）时报块设备错、不落盘，只这一次；`None` 只数不报。
+struct ChangingCallsOfThePool {
+    calls: Cell<u64>,
+    failing_call: Option<u64>,
+}
+
+struct DeviceFailingOneChangingCall {
+    inner: SparseBlockDevice,
+    calls_of_the_pool: Rc<ChangingCallsOfThePool>,
+}
+
+impl DeviceFailingOneChangingCall {
+    fn count_one_changing_call(&self) -> Result<(), BlockDeviceError> {
+        let call = self.calls_of_the_pool.calls.get() + 1;
+        self.calls_of_the_pool.calls.set(call);
+        if self.calls_of_the_pool.failing_call == Some(call) {
+            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
+                "用例在这一次改盘调用上注入的块设备错",
+            )));
+        }
+        Ok(())
+    }
+}
+
+impl BlockDevice for DeviceFailingOneChangingCall {
+    fn read_at(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        self.inner.read_at(offset, buffer)
+    }
+
+    fn write_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+        durability: WriteDurability,
+    ) -> Result<(), BlockDeviceError> {
+        self.count_one_changing_call()?;
+        self.inner.write_at(offset, bytes, durability)
+    }
+
+    fn write_zeroes_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<(), BlockDeviceError> {
+        self.count_one_changing_call()?;
+        self.inner.write_zeroes_at(offset, length)
+    }
+
+    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
+        self.count_one_changing_call()?;
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
+impl DeviceOverASparseImage for DeviceFailingOneChangingCall {
+    fn sparse_image(&self) -> &SparseDevice {
+        &self.inner.image
+    }
+}
+
+/// `stuck_pool` 那一刻的盘面与会话拷一份，盘外面包上数改盘调用的那一层（第 `failing_call` 次报错，`None` 只数）。
+fn pool_failing_one_changing_call_copied_from(
+    stuck_pool: &PoolUnderTest<SparseBlockDevice>,
+    failing_call: Option<u64>,
+) -> (
+    PoolUnderTest<DeviceFailingOneChangingCall>,
+    Rc<ChangingCallsOfThePool>,
+) {
+    let calls_of_the_pool = Rc::new(ChangingCallsOfThePool {
+        calls: Cell::new(0),
+        failing_call,
+    });
+    let devices = plain_devices_on(&stuck_pool.image())
+        .into_iter()
+        .map(|(identity, inner)| {
+            (
+                identity,
+                DeviceFailingOneChangingCall {
+                    inner,
+                    calls_of_the_pool: Rc::clone(&calls_of_the_pool),
+                },
+            )
+        })
+        .collect();
+    (
+        PoolUnderTest {
+            parameters: stuck_pool.parameters.clone(),
+            device_bytes: stuck_pool.device_bytes,
+            devices,
+            session: Some(copy_of_the_session(stuck_pool.session())),
+            write_time_seconds: stuck_pool.write_time_seconds,
+            content_of_the_current_version: stuck_pool.content_of_the_current_version.clone(),
+            content_of_each_root: stuck_pool.content_of_each_root.clone(),
+        },
+        calls_of_the_pool,
+    )
+}
+
+/// 推环入口先判冻结（判决第二节「K2-1 报法」；攻方第二节）：主形卡住之后推环，真发到半路一次改盘调用报块设备错——前面那几次已落盘，
+/// 会话的现行那一版停在最后一次落盘的那一版上（攻方 M1），失败那一次冻结在分配器上。再推一次：入口第一道就拒成点名冻结的那一种、
+/// 带冻结那一次的 txg 与实例代号，在核盘表与预演之前，两块盘逐字节不变、分配器与会话不动。改之前预演在冻结着的分配器拷贝上照样过，
+/// 报成 `PublishFailedAfterTheRehearsal`、拒因是冻结。
+#[test]
+fn pushing_the_root_ring_again_after_a_block_device_error_halfway_is_refused_as_frozen_before_any_write(
+) {
+    let PoolStuckBehindTheWideProtection {
+        pool: stuck_pool, ..
+    } = pool_stuck_behind_the_wide_protection(
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
+            overwrites_before: 40,
+        },
+    );
+    let (mut counting_pool, counted_calls) =
+        pool_failing_one_changing_call_copied_from(&stuck_pool, None);
+    let pushed_without_a_fault = push_the_root_ring_of(&mut counting_pool)
+        .unwrap_or_else(|refusal| panic!("不注入故障时推环做得成：{refusal:?}"));
+    assert!(
+        pushed_without_a_fault.versions_published.len() >= 3,
+        "推环要发好几次（阳性对照：半路失败时前面有几次已落盘）：{} 次",
+        pushed_without_a_fault.versions_published.len()
+    );
+    let failing_call = counted_calls.calls.get() / 2;
+    let (mut failing_pool, _) =
+        pool_failing_one_changing_call_copied_from(&stuck_pool, Some(failing_call));
+    let Err(RootRingPushRefused::PublishFailedAfterTheRehearsal(failed)) =
+        push_the_root_ring_of(&mut failing_pool)
+    else {
+        panic!("第 {failing_call} 次改盘调用报块设备错：推环真发到半路发不出去");
+    };
+    assert!(
+        failed.failed_publish_in_the_push > 1
+            && failed.failed_publish_in_the_push <= failed.publishes_in_the_push,
+        "失败在半路（前面有几次已落盘）：第 {} 次，共 {} 次",
+        failed.failed_publish_in_the_push,
+        failed.publishes_in_the_push
+    );
+    assert_eq!(
+        failed.versions_published.len(),
+        failed.failed_publish_in_the_push - 1,
+        "失败那一次之前的每一次都落了盘、交了回来"
+    );
+    let last_landed = failed
+        .versions_published
+        .last()
+        .expect("失败在第二次及以后：前面至少一次已落盘");
+    assert_eq!(
+        *failing_pool.session().current.root(),
+        last_landed.root,
+        "真发半路失败：会话的现行那一版停在最后一次落盘的那一版上"
+    );
+    let frozen = failing_pool
+        .session()
+        .allocator
+        .frozen_publish()
+        .map(|frozen| (frozen.checkpoint_txg(), frozen.instance()))
+        .expect("落盘途中失败的那一次冻结在分配器上（阳性对照）");
+    assert_eq!(
+        frozen.0,
+        CheckpointTxg(last_landed.root.checkpoint_txg.0 + 1),
+        "冻结的是最后一次落盘之后的那一次"
+    );
+    let image_before_pushing_again = failing_pool.image();
+    let allocator_before_pushing_again = format!("{:?}", failing_pool.session().allocator);
+    let current_before_pushing_again = *failing_pool.session().current.root();
+    match push_the_root_ring_of(&mut failing_pool) {
+        Err(RootRingPushRefused::PublishFrozenAfterAWriteFailureIsNotResentYet {
+            checkpoint_txg,
+            instance,
+        }) => assert_eq!(
+            (checkpoint_txg, instance),
+            frozen,
+            "拒成点名冻结的那一种，带冻结那一次的 txg 与实例代号"
+        ),
+        other => panic!("冻结着再推一次：入口第一道拒成点名冻结的那一种，而交回 {other:?}"),
+    }
+    assert!(
+        failing_pool.image() == image_before_pushing_again,
+        "冻结着再推一次：两块盘逐字节不变"
+    );
+    assert_eq!(
+        format!("{:?}", failing_pool.session().allocator),
+        allocator_before_pushing_again,
+        "冻结着再推一次：分配器整份不变"
+    );
+    assert_eq!(
+        *failing_pool.session().current.root(),
+        current_before_pushing_again,
+        "冻结着再推一次：会话的现行那一版不变"
+    );
+}
+
+/// 推环入口核交进来的盘表（攻方 M2）：主形卡住之后两块盘的盘体对调（身份照旧、盘上的本盘设备号对不上），推环在任何写之前拒成盘表那一种，
+/// 两块盘逐字节不变、分配器与会话不动。不核时推环照常发、本池的写落到对调了的盘上。
+#[test]
+fn pushing_the_root_ring_with_the_two_device_bodies_swapped_is_refused_before_any_write() {
+    let PoolStuckBehindTheWideProtection { mut pool, .. } = pool_stuck_behind_the_wide_protection(
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
+            overwrites_before: 40,
+        },
+    );
+    {
+        let (first, second) = pool.devices.split_at_mut(1);
+        std::mem::swap(&mut first[0].1, &mut second[0].1);
+    }
+    let image_before = pool.image();
+    let allocator_before = format!("{:?}", pool.session().allocator);
+    let current_before = *pool.session().current.root();
+    let pushed = push_the_root_ring_of(&mut pool);
+    assert!(
+        matches!(pushed, Err(RootRingPushRefused::DevicesHandedIn(_))),
+        "盘体对调：推环拒成交进来的盘表没过核，而交回 {pushed:?}"
+    );
+    assert!(pool.image() == image_before, "推环被拒：两块盘逐字节不变");
+    assert_eq!(
+        format!("{:?}", pool.session().allocator),
+        allocator_before,
+        "推环被拒：分配器整份不变"
+    );
+    assert_eq!(
+        *pool.session().current.root(),
+        current_before,
+        "推环被拒：会话的现行那一版不变"
+    );
+}
+
+/// C 那一圈（这一刻记着落点的那几槽）罩着的槽里这一刻隔离着的槽数，两块盘合计。
+fn isolated_slots_covered_by_the_ring_slots_holding_placements(allocator: &PoolAllocator) -> u64 {
+    let placements: Vec<PlacementOnDevice> = allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表")
+        .ring_slots_holding_placements_named_by_unapplied_journal_records()
+        .into_values()
+        .flatten()
+        .collect();
+    allocator
+        .isolated_slots_covered_by(&placements)
+        .iter()
+        .map(|(_, isolated)| *isolated)
+        .sum()
+}
+
+/// 判决第二节「K1-1 三个量」正推注入级那一形：C 那一圈记着落点、它们罩着的槽却一个都没隔离。主形（建 inode、前缀 40 次）卡住之后，
+/// 把 C 那几槽记着的落点换成现行账里仍分配着的那几个落点（分配位置着、不隔离；原来那一圈置的隔离位照旧留着、空间照旧不够）。
+fn pool_stuck_while_the_ring_slots_hold_placements_none_of_which_is_isolated(
+) -> PoolUnderTest<SparseBlockDevice> {
+    let PoolStuckBehindTheWideProtection { mut pool, .. } = pool_stuck_behind_the_wide_protection(
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
+            overwrites_before: 40,
+        },
+    );
+    let allocator = &mut pool.session.as_mut().expect("挂着的会话").allocator;
+    let still_allocated: Vec<PlacementOnDevice> = allocator
+        .records()
+        .iter()
+        .filter(|record| !record.is_released)
+        .map(|record| PlacementOnDevice {
+            device: record.device,
+            slot: record.slot,
+            span: u64::from(record.span_slots),
+        })
+        .collect();
+    assert!(
+        !still_allocated.is_empty(),
+        "现行账里有仍分配着的落点（阳性对照）"
+    );
+    let replaced: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> = allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表")
+        .ring_slots_holding_placements_named_by_unapplied_journal_records()
+        .into_keys()
+        .map(|ring_slot| (ring_slot, still_allocated.clone()))
+        .collect();
+    assert!(
+        !replaced.is_empty(),
+        "主形卡住时 C 那一圈记着落点（阳性对照）"
+    );
+    allocator.note_placements_named_by_unapplied_journal_records(replaced);
+    assert_eq!(
+        isolated_slots_covered_by_the_ring_slots_holding_placements(allocator),
+        0,
+        "注入之后 C 那一圈罩着的槽一个都没隔离（阳性对照）"
+    );
+    assert!(
+        isolated_slots_of(allocator) > 0,
+        "原来那一圈置的隔离位照旧留着（空间照旧不够）"
+    );
+    pool
+}
+
+/// C 那一圈记着落点而一个槽都没隔离：推满仍不够、F 在上限时照旧只报空间不够，不报「被宽护卡住」（正推 `noIsoCheck`）。
+#[test]
+fn no_space_while_the_ring_slots_hold_placements_none_of_which_is_isolated_is_not_reported_as_the_wide_protection(
+) {
+    let mut pool = pool_stuck_while_the_ring_slots_hold_placements_none_of_which_is_isolated();
+    let refusal = pool
+        .overwrite(OVERWRITE_BYTES)
+        .expect_err("空间照旧不够：覆盖写被拒");
+    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
+        panic!("被拒是推满仍不够：{refusal:?}");
+    };
+    assert!(
+        matches!(
+            no_space.stop,
+            FloorRaiseStop::FloorAlreadyAtTheCeiling { .. }
+        ),
+        "推抬 F 停在上限（阳性对照：与被宽护卡住同一种停法）：{:?}",
+        no_space.stop
+    );
+    assert!(
+        matches!(
+            no_space.wide_protection_at_the_ceiling,
+            WideProtectionAtTheCeiling::NotWhatStopsTheAdmission
+        ),
+        "C 那一圈罩着的槽一个都没隔离：不报被宽护卡住，而报 {:?}",
+        no_space.wide_protection_at_the_ceiling
+    );
+}
+
+/// C 那一圈记着落点而一个槽都没隔离：推环入口在任何写之前拒成「写掉哪一槽都放不出东西」那一种，两块盘逐字节不变、分配器不动。
+#[test]
+fn pushing_the_root_ring_while_the_ring_slots_hold_placements_none_of_which_is_isolated_writes_nothing(
+) {
+    let mut pool = pool_stuck_while_the_ring_slots_hold_placements_none_of_which_is_isolated();
+    let image_before = pool.image();
+    let allocator_before = format!("{:?}", pool.session().allocator);
+    let pushed = push_the_root_ring_of(&mut pool);
+    assert!(
+        matches!(
+            pushed,
+            Err(RootRingPushRefused::NothingHeldUntilThisProcessWritesARingSlot)
+        ),
+        "C 那一圈罩着的槽一个都没隔离：不推，而交回 {pushed:?}"
+    );
+    assert!(pool.image() == image_before, "不推：两块盘逐字节不变");
+    assert_eq!(
+        format!("{:?}", pool.session().allocator),
+        allocator_before,
+        "不推：分配器整份不变"
+    );
+}
+
+/// 记着空串的槽不算 C 的槽（正推 `emptyHeld`、`slotsWrittenEmpty`）：主形（建 inode、前缀 40 次）卡住之后，在现行那一版那一槽
+/// （从现行那一版起最后轮到的一槽，满一环）记一个空串。再写一次照旧报「被宽护卡住」，R 那一槽与要推的次数都照旧是录下来的 R 那一槽；
+/// 推环推到 R 那一槽为止，交回写掉的那几槽就是 C 那一圈原来那一槽。
+#[test]
+fn a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_writes_up_to() {
+    let PoolStuckBehindTheWideProtection {
+        recorded,
+        mut pool,
+        stuck,
+    } = pool_stuck_behind_the_wide_protection(
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
+            overwrites_before: 40,
+        },
+    );
+    let ring_slot_of_the_current_version = target_for_publish(
+        pool.session().current.root().checkpoint_txg,
+        pool.parameters.geometry.root_ring_slots_per_region,
+    );
+    {
+        let allocator = &mut pool.session.as_mut().expect("挂着的会话").allocator;
+        let mut holding = allocator
+            .root_ring_occupancy()
+            .expect("挂载建的分配器装着根环表")
+            .ring_slots_holding_placements_named_by_unapplied_journal_records();
+        assert_eq!(
+            holding.keys().copied().collect::<Vec<RootRingSlot>>(),
+            vec![stuck.ring_slot_of_r],
+            "主形 C 那一圈只有 R 那一槽（阳性对照）"
+        );
+        holding.insert(ring_slot_of_the_current_version, Vec::new());
+        allocator.note_placements_named_by_unapplied_journal_records(holding);
+    }
+    let refusal = pool
+        .overwrite(OVERWRITE_BYTES)
+        .expect_err("宽护照旧挡着：覆盖写被拒");
+    let WriteStuckBehindTheWideProtection {
+        stuck: stuck_again, ..
+    } = stuck_behind_the_wide_protection_reported_by(refusal, "多记一个空串之后再写");
+    assert_eq!(
+        (
+            stuck_again.ring_slot_of_r,
+            stuck_again.empty_publishes_until_the_ring_slot_of_r_is_written
+        ),
+        (
+            target_for_publish(
+                recorded.key.1,
+                recorded.pool.parameters.geometry.root_ring_slots_per_region
+            ),
+            stuck.empty_publishes_until_the_ring_slot_of_r_is_written
+        ),
+        "记着空串的那一槽什么都不罩：R 那一槽与要推的次数照旧"
+    );
+    let pushed = push_the_root_ring_of(&mut pool)
+        .unwrap_or_else(|push_refusal| panic!("推环做得成：{push_refusal:?}"));
+    assert_eq!(
+        u64::try_from(pushed.versions_published.len()).expect("次数"),
+        stuck.empty_publishes_until_the_ring_slot_of_r_is_written,
+        "推到 R 那一槽为止，不推到记着空串的那一槽"
+    );
+    assert_eq!(
+        pushed.ring_slots_written,
+        vec![stuck.ring_slot_of_r],
+        "交回写掉的那几槽：C 那一圈原来那一槽（记着空串的那一槽不算）"
+    );
+}
+
+/// 两槽 C（攻方 M12，照攻方 `research/prompts/m3-batch10z-pqrs-code-r1-opus-model/atk_part3.rs` 的 `atk_k2_two_c_slots_push` 造法：丙并中间那一形，
+/// Q 顺序写、R 建 inode，挂载那一刻 Q 的根与末条记录、R 的账读回全零，C 那一圈有 Q、R 两槽）：挂上就推环，推的次数是到较晚轮到的 R 那一槽的距离，
+/// 交回写掉的两槽，推完 C 那一圈一槽不剩、隔离回 0，池级 checker 不红 I-7.4、I-4.8。R 那一槽取最先轮到的那一槽时推环停在 Q 那一槽、R 还记着。
+#[test]
+fn pushing_the_root_ring_with_two_ring_slots_holding_placements_writes_up_to_the_later_one_and_releases_both(
+) {
+    let recorded = record_a_sequential_write_as_q_and_an_inode_creation_as_r_after_overwrites(
+        OVERWRITES_BEFORE_THE_PUBLISHES_RECORDED_AS_Q_AND_R,
+    );
+    let crash_image = recorded.crash_image_with_the_root_of_r_persisted_and_its_rotation_lost();
+    let ranges = recorded.ranges_reading_back_all_zero_at_the_mount(&crash_image);
+    let mut faulted_devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
+    let mounted = mount_writable_with_space_admission(
+        &recorded.pool.parameters,
+        &mut faulted_devices,
+        SpaceAdmission::JudgedByTheFormula,
+    )
+    .expect("Q 的根与末条记录、R 的账读回全零：可写挂载退到 P、做得成");
+    let image_after_the_mount =
+        image_of_devices_reading_back_all_zero(&faulted_devices, recorded.pool.device_bytes);
+    let (_, session) = MountedSession::of_the_mount(mounted);
+    let mut pool = PoolUnderTest {
+        parameters: recorded.pool.parameters.clone(),
+        device_bytes: recorded.pool.device_bytes,
+        devices: plain_devices_on(&image_after_the_mount),
+        session: Some(session),
+        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
+        content_of_the_current_version: Vec::new(),
+        content_of_each_root: BTreeMap::new(),
+    };
+    let slots_per_region = recorded.pool.parameters.geometry.root_ring_slots_per_region;
+    let ring_slot_of_q = target_for_publish(recorded.key_of_q.1, slots_per_region);
+    let ring_slot_of_r = target_for_publish(recorded.key_of_r.1, slots_per_region);
+    let holding_before_the_push: Vec<RootRingSlot> = pool
+        .session()
+        .allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表")
+        .ring_slots_holding_placements_named_by_unapplied_journal_records()
+        .into_keys()
+        .collect();
+    let mut q_and_r = vec![ring_slot_of_q, ring_slot_of_r];
+    q_and_r.sort();
+    assert_eq!(
+        holding_before_the_push, q_and_r,
+        "C 那一圈有 Q、R 两槽（阳性对照）"
+    );
+    let current_txg = pool.session().current.root().checkpoint_txg;
+    let publishes_until_the_ring_slot_of = |ring_slot: RootRingSlot| -> usize {
+        (1..=singlefs_format::ROOT_RING_REGIONS * slots_per_region.count())
+            .position(|publishes_ahead| {
+                target_for_publish(
+                    CheckpointTxg(current_txg.0 + publishes_ahead),
+                    slots_per_region,
+                ) == ring_slot
+            })
+            .expect("一环之内每一槽都轮到一次")
+            + 1
+    };
+    assert!(
+        publishes_until_the_ring_slot_of(ring_slot_of_q)
+            < publishes_until_the_ring_slot_of(ring_slot_of_r),
+        "Q 那一槽比 R 那一槽先轮到（阳性对照：取最先轮到的与最后轮到的推的次数不同）"
+    );
+    let pushed = push_the_root_ring_of(&mut pool)
+        .unwrap_or_else(|refusal| panic!("两槽 C 推环做得成：{refusal:?}"));
+    assert_eq!(
+        pushed.versions_published.len(),
+        publishes_until_the_ring_slot_of(ring_slot_of_r),
+        "推的次数是到较晚轮到的 R 那一槽的距离"
+    );
+    assert_eq!(
+        pushed.ring_slots_written, q_and_r,
+        "交回写掉的那几槽：Q、R 两槽"
+    );
+    let allocator = &pool.session().allocator;
+    assert!(
+        allocator
+            .root_ring_occupancy()
+            .expect("挂载建的分配器装着根环表")
+            .ring_slots_holding_placements_named_by_unapplied_journal_records()
+            .is_empty(),
+        "推完 C 那一圈一槽不剩"
+    );
+    assert_eq!(
+        isolated_slots_of(allocator),
+        0,
+        "推完隔离回 0（这一形根环表里没有被抛弃根）"
+    );
+    let violations = violations_of_unit_reuse_on(&pool.image());
+    assert!(
+        violations.is_empty(),
+        "推完之后 I-7.4、I-4.8 不红：{violations:?}"
+    );
+}
+
+/// 每块盘上挑几个这一刻空闲的槽（单元区里槽号最小的那几个）置隔离位（注入级），交回这几个槽（一槽跨度一个）。
+fn isolate_free_slots(
+    allocator: &mut PoolAllocator,
+    free_slots_on_each_device: u64,
+) -> Vec<PlacementOnDevice> {
+    let mut placements: Vec<PlacementOnDevice> = Vec::new();
+    for device_map in &mut allocator.devices {
+        let unit_area_start = device_map.unit_area_start().slot().0;
+        let free_slots: Vec<SlotNumber> = (unit_area_start
+            ..unit_area_start + device_map.unit_area_slots())
+            .map(SlotNumber)
+            .filter(|slot| device_map.is_free(*slot))
+            .take(usize::try_from(free_slots_on_each_device).expect("几个槽"))
+            .collect();
+        assert_eq!(
+            u64::try_from(free_slots.len()).expect("几个槽"),
+            free_slots_on_each_device,
+            "盘 {} 上有这么多个空闲的槽（阳性对照）",
+            device_map.device.0
+        );
+        for slot in free_slots {
+            device_map.isolate(slot, 1);
+            placements.push(PlacementOnDevice {
+                device: device_map.device,
+                slot,
+                span: 1,
+            });
+        }
+    }
+    placements
+}
+
+/// 每块盘上挑几个这一刻空闲的槽、置隔离位，记成 `ring_slot` 那一槽 C 记着的落点（一槽跨度一个），并进 C 那一圈原来记着的那几槽。
+/// 注入级：可写挂载自己造不出这一份，用例拿它摆出 C 那一圈多一槽的形。
+fn hold_isolated_free_slots_on_a_further_ring_slot(
+    allocator: &mut PoolAllocator,
+    ring_slot: RootRingSlot,
+    free_slots_on_each_device: u64,
+) {
+    let placements = isolate_free_slots(allocator, free_slots_on_each_device);
+    let mut holding = allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表")
+        .ring_slots_holding_placements_named_by_unapplied_journal_records();
+    assert!(
+        !holding.contains_key(&ring_slot),
+        "注入的那一槽原来没记着落点（阳性对照）"
+    );
+    holding.insert(ring_slot, placements);
+    allocator.note_placements_named_by_unapplied_journal_records(holding);
+}
+
+/// 两槽 C、落点重叠时出路二说放出来的隔离槽数（判决第二节「K1-1 三个量」正推注入级那一形：重叠时放出来的数偏大）：攻方 K1 那一形挂上之后
+/// 长 13 个单元被拒一次（会话推一串抬 F、把 F 抬到上限），在 R 那一槽之前先轮到的一槽（现行那一版之后第一次发布写的那一槽）注入 C 的第二槽，记着 R 那一槽一半的落点（与 R 那一槽重叠）
+/// 加每块盘几个隔离着的空闲槽；另在每块盘隔离一个 C 那一圈不罩的空闲槽。再写同一个长度报「被宽护卡住」、推完放得下：
+/// 报的挡着的槽数只数 C 罩着的（攻方 M7：取全部隔离时多数那几个），出路二说放出来的槽数等于它（重叠的不重数、C 不罩的不算），
+/// 照推环走完就是推完前后每块盘隔离数之差，同一个写放得下。
+#[test]
+fn the_slots_the_way_out_by_the_push_releases_with_two_ring_slots_holding_overlapping_placements_are_what_the_push_releases(
+) {
+    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
+    let mut pool = crashed.mounted_and_stuck_once_by_a_write_grown_by(
+        DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY,
+    );
+    let slots_per_region = pool.parameters.geometry.root_ring_slots_per_region;
+    let ring_slot_written_first = target_for_publish(
+        CheckpointTxg(pool.session().current.root().checkpoint_txg.0 + 1),
+        slots_per_region,
+    );
+    {
+        let allocator = &mut pool.session.as_mut().expect("挂着的会话").allocator;
+        let holding = allocator
+            .root_ring_occupancy()
+            .expect("挂载建的分配器装着根环表")
+            .ring_slots_holding_placements_named_by_unapplied_journal_records();
+        let (ring_slot_of_r, placements_of_r) =
+            holding.iter().next().expect("C 那一圈只有 R 那一槽");
+        assert_ne!(
+            *ring_slot_of_r, ring_slot_written_first,
+            "R 那一槽不是推环第一次就写的那一槽（阳性对照：注入的那一槽先轮到）"
+        );
+        let overlapping_with_r: Vec<PlacementOnDevice> =
+            placements_of_r.iter().step_by(2).copied().collect();
+        hold_isolated_free_slots_on_a_further_ring_slot(allocator, ring_slot_written_first, 2);
+        let mut two_ring_slots = allocator
+            .root_ring_occupancy()
+            .expect("挂载建的分配器装着根环表")
+            .ring_slots_holding_placements_named_by_unapplied_journal_records();
+        two_ring_slots
+            .get_mut(&ring_slot_written_first)
+            .expect("刚注入的那一槽")
+            .extend(overlapping_with_r);
+        allocator.note_placements_named_by_unapplied_journal_records(two_ring_slots);
+        isolate_free_slots(allocator, 1);
+    }
+    assert!(
+        isolated_slots_covered_by_the_ring_slots_holding_placements(&pool.session().allocator)
+            < isolated_slots_of(&pool.session().allocator),
+        "每块盘另有一个隔离着、C 那一圈不罩的槽（阳性对照：推环放不出它，挡着的槽数与放出来的槽数都不算它）"
+    );
+    let length = crashed
+        .bytes_of_the_file_grown_by(DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY);
+    let refusal = pool.sequential_write(length).expect_err("两槽 C 挡着准入");
+    let WriteStuckBehindTheWideProtection { stuck, .. } =
+        stuck_behind_the_wide_protection_reported_by(refusal, "两槽 C、落点重叠");
+    let released_in_the_rehearsal = released_by_the_push_if_the_change_is_admitted_after_it(&stuck)
+        .unwrap_or_else(|| {
+            panic!(
+                "两槽 C：预演推完之后这次改动放得下，而预演的结局是 {:?}",
+                stuck.pushing_the_root_ring
+            )
+        });
+    assert_eq!(
+        released_in_the_rehearsal, stuck.isolated_slots_on_each_device,
+        "根环表里没有被抛弃根：推环放出来的就是 C 那一圈罩着的、隔离着的槽，两槽重叠的那几个只数一次"
+    );
+    let ways_out = stuck.ways_out();
+    assert_eq!(ways_out.len(), 2, "推完放得下：两条出路都列：{ways_out:?}");
+    assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_r_and_the_released_slots(
+        &stuck,
+        &ways_out[1],
+        &released_in_the_rehearsal,
+        "两槽 C、落点重叠",
+    );
+    let isolated_before_the_push = isolated_slots_on_each_device_of(&pool.session().allocator);
+    let pushed = push_the_root_ring_of(&mut pool)
+        .unwrap_or_else(|push_refusal| panic!("两槽 C 推环做得成：{push_refusal:?}"));
+    assert_eq!(
+        pushed.ring_slots_written.len(),
+        2,
+        "推环写掉 C 那一圈的两槽：{:?}",
+        pushed.ring_slots_written
+    );
+    assert_eq!(
+        released_in_the_rehearsal,
+        isolated_slots_released_between(
+            &isolated_before_the_push,
+            &isolated_slots_on_each_device_of(&pool.session().allocator),
+        ),
+        "出路二说放出来的隔离槽数就是推完前后每块盘隔离数之差"
+    );
+    pool.sequential_write(length)
+        .unwrap_or_else(|refusal_after_the_push| {
+            panic!("推完之后同一个写放得下：{refusal_after_the_push:?}")
+        });
+}
+
+/// 不再推的原因是一次准入的发布数做满时不报「被宽护卡住」（正推 `budgetAsStuck`）：攻方 K1 那一形长 20 个单元被宽护挡着，推环之后再试，
+/// 会话推两串抬 F 就碰到一次准入 8 次发布的预算（攻方 `k1-trace`）；在现行那一版那一槽（满一环才轮到）注入 C 一槽、每块盘隔离两个空闲槽，
+/// 再试：照旧停在预算上，C 那一圈这一刻隔离着槽，报的照旧只是空间不够。
+#[test]
+fn no_space_when_the_publishes_of_one_admission_run_out_while_the_ring_slots_hold_isolated_placements_is_not_reported_as_the_wide_protection(
+) {
+    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
+    let mut pool = crashed.mounted();
+    let length = crashed.bytes_of_the_file_grown_by(
+        DATA_UNITS_A_WRITE_STILL_REFUSED_AFTER_THE_PUSH_GROWS_THE_FILE_BY,
+    );
+    pool.sequential_write(length)
+        .expect_err("长 20 个单元：宽护挡着准入");
+    push_the_root_ring_of(&mut pool).unwrap_or_else(|refusal| panic!("推环做得成：{refusal:?}"));
+    let ring_slot_of_the_current_version = target_for_publish(
+        pool.session().current.root().checkpoint_txg,
+        pool.parameters.geometry.root_ring_slots_per_region,
+    );
+    hold_isolated_free_slots_on_a_further_ring_slot(
+        &mut pool.session.as_mut().expect("挂着的会话").allocator,
+        ring_slot_of_the_current_version,
+        2,
+    );
+    let refusal = pool
+        .sequential_write(length)
+        .expect_err("推完之后长 20 个单元照旧被拒");
+    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
+        panic!("被拒是推满仍不够：{refusal:?}");
+    };
+    assert!(
+        matches!(
+            no_space.stop,
+            FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. }
+        ),
+        "不再推的原因是一次准入的发布数做满（阳性对照）：{:?}",
+        no_space.stop
+    );
+    assert!(
+        isolated_slots_covered_by_the_ring_slots_holding_placements(&pool.session().allocator) > 0,
+        "这一刻 C 那一圈还隔离着槽（阳性对照：只差停因）"
+    );
+    assert!(
+        matches!(
+            no_space.wide_protection_at_the_ceiling,
+            WideProtectionAtTheCeiling::NotWhatStopsTheAdmission
+        ),
+        "停因是预算用完：不报被宽护卡住，而报 {:?}",
+        no_space.wide_protection_at_the_ceiling
+    );
+}
+
+/// 挂上之后先覆盖写几次：之后有 4 个以上不同的非空状态，准入上限高过 F，被拒时会话真去推抬 F。
+const OVERWRITES_BEFORE_THE_FLOOR_RAISE_FINDS_NO_PLACEMENT: usize = 5;
+
+/// 不再推的原因是抬 F 那一串自己也取不到落点时不报「被宽护卡住」（正推 `spaceAsStuck`）：攻方 K1 那一形挂上之后覆盖写几次，
+/// 再把每块盘没挡的槽全隔离掉（注入级），覆盖写一次：准入拒、推抬 F，抬 F 那一串在预演里取不到落点；C 那一圈这一刻隔离着槽，
+/// 报的照旧只是空间不够。
+#[test]
+fn no_space_when_the_floor_raise_finds_no_placement_while_the_ring_slots_hold_isolated_placements_is_not_reported_as_the_wide_protection(
+) {
+    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
+    let mut pool = crashed.mounted();
+    for overwrite_number in 1..=OVERWRITES_BEFORE_THE_FLOOR_RAISE_FINDS_NO_PLACEMENT {
+        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
+            panic!("挂上之后第 {overwrite_number} 次覆盖写落得下：{refusal:?}")
+        });
+    }
+    for device_map in &mut pool.session.as_mut().expect("挂着的会话").allocator.devices {
+        let unit_area_start = device_map.unit_area_start().slot().0;
+        for slot in unit_area_start..unit_area_start + device_map.unit_area_slots() {
+            if !device_map.is_blocked_for_commit_generated(SlotNumber(slot)) {
+                device_map.isolate(SlotNumber(slot), 1);
+            }
+        }
+        assert_eq!(
+            device_map.unblocked_slots(),
+            0,
+            "盘 {} 上没挡的槽全隔离掉了（阳性对照）",
+            device_map.device.0
+        );
+    }
+    let refusal = pool
+        .overwrite(OVERWRITE_BYTES)
+        .expect_err("没挡的槽全隔离掉：覆盖写被拒");
+    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
+        panic!("被拒是推满仍不够：{refusal:?}");
+    };
+    assert!(
+        matches!(
+            no_space.stop,
+            FloorRaiseStop::FloorRaiseRefusedForSpace { .. }
+        ),
+        "不再推的原因是抬 F 那一串自己也取不到落点（阳性对照）：{:?}",
+        no_space.stop
+    );
+    assert!(
+        isolated_slots_covered_by_the_ring_slots_holding_placements(&pool.session().allocator) > 0,
+        "这一刻 C 那一圈还隔离着槽（阳性对照：只差停因）"
+    );
+    assert!(
+        matches!(
+            no_space.wide_protection_at_the_ceiling,
+            WideProtectionAtTheCeiling::NotWhatStopsTheAdmission
+        ),
+        "停因是抬 F 那一串取不到落点：不报被宽护卡住，而报 {:?}",
+        no_space.wide_protection_at_the_ceiling
+    );
+}
+
+/// 根槽读坏的被抛弃根那一圈照记着的整段跨度补隔离（正推 K4-1：`crates/mutations.tsv` 里「第10z批第九块 K3 loop-span1」那一行改之后不红——
+/// 挂载那一刻 C 记着的落点两槽都已隔离着，抬 F 那一圈补的那一遍什么都不改）：挂上之后，C 记着的一个跨两槽、两槽都隔离着的落点，
+/// 把后一槽的隔离位清掉（注入级：后一槽分配位没置着、空闲），再让 C 的根槽读报错、正常卸载：抬 F 那一串照根环表里记着的落点补隔离，
+/// 后一槽也隔离着。只隔起点那一槽时后一槽照旧空闲、发得出去。
+#[test]
+fn unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_isolates_the_later_slot_of_its_recorded_two_slot_placement_left_unisolated(
+) {
+    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
+    let fault =
+        ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
+    let (mut pool, recorded) = mounted_while_the_abandoned_root_reads(&built, &fault);
+    let (device, later_slot) = {
+        let allocator = &pool.session().allocator;
+        recorded
+            .iter()
+            .filter(|placement| placement.span >= 2)
+            .find_map(|placement| {
+                let device_map = allocator
+                    .devices
+                    .iter()
+                    .find(|device_map| device_map.device == placement.device)
+                    .expect("池里有这块盘");
+                (placement.slot.0..placement.slot.0 + placement.span)
+                    .all(|slot| device_map.is_isolated(SlotNumber(slot)))
+                    .then_some((
+                        placement.device,
+                        SlotNumber(placement.slot.0 + placement.span - 1),
+                    ))
+            })
+            .expect("C 记着的落点里有一个跨两槽以上、每一槽都隔离着的（阳性对照）")
+    };
+    {
+        let device_map = pool
+            .session
+            .as_mut()
+            .expect("挂着的会话")
+            .allocator
+            .devices
+            .iter_mut()
+            .find(|device_map| device_map.device == device)
+            .expect("池里有这块盘");
+        device_map.clear_isolation_of_slot(later_slot);
+        assert!(
+            device_map.is_free(later_slot),
+            "后一槽清掉隔离位之后空闲：分配位没置着（阳性对照）"
+        );
+    }
+    fault.is_active.set(true);
+    assert_the_root_ring_slot_is_unreadable(&pool, &built.abandoned_root);
+    let (_, allocator_after_the_unmount) =
+        unmount_expecting_the_floor_raised(&mut pool, "C 的根槽读不出");
+    let device_map = allocator_after_the_unmount
+        .devices
+        .iter()
+        .find(|device_map| device_map.device == device)
+        .expect("池里有这块盘");
+    assert!(
+        device_map.is_isolated(later_slot),
+        "抬 F 那一串照根环表里记着的落点补隔离、补满整段跨度：后一槽（盘 {}、槽 {}）隔离着",
+        device.0,
+        later_slot.0
+    );
+}
````

### 第二十块变异表追加（25 行）

````tsv
第10z批第二十块 甲 去掉预演准入那一判、推完照旧被拒也列推环（长 20 个单元）	crates/singlefs-core/src/mounted_session.rs	        Err(refusal) => RootRingPushRehearsal::ChangeStillRefusedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device,\n            refusal: Box::new(refusal),\n        },	        Err(_refusal) => RootRingPushRehearsal::ChangeAdmittedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device,\n        },	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- lists_only_unmounting	a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through
第10z批第二十块 甲 去掉预演准入那一判、推完照旧被拒也列推环（长 36 个单元）	crates/singlefs-core/src/mounted_session.rs	        Err(refusal) => RootRingPushRehearsal::ChangeStillRefusedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device,\n            refusal: Box::new(refusal),\n        },	        Err(_refusal) => RootRingPushRehearsal::ChangeAdmittedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device,\n        },	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_write_refused_even_without_the_wide_protection_lists_only_unmounting_which_lets_it_through	a_write_refused_even_without_the_wide_protection_lists_only_unmounting_which_lets_it_through
第10z批第二十块 甲 去掉预演准入那一判、推完照旧被拒也列推环（回退形写成 60 个单元）	crates/singlefs-core/src/mounted_session.rs	        Err(refusal) => RootRingPushRehearsal::ChangeStillRefusedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device,\n            refusal: Box::new(refusal),\n        },	        Err(_refusal) => RootRingPushRehearsal::ChangeAdmittedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device,\n        },	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_sixty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting	a_sixty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting
第10z批第二十块 甲 去掉预演准入那一判、推完照旧被拒也列推环（回退形写成 80 个单元）	crates/singlefs-core/src/mounted_session.rs	        Err(refusal) => RootRingPushRehearsal::ChangeStillRefusedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device,\n            refusal: Box::new(refusal),\n        },	        Err(_refusal) => RootRingPushRehearsal::ChangeAdmittedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device,\n        },	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- an_eighty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting	an_eighty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting
第10z批第二十块 甲 去掉预演准入那一判、推完照旧被拒也列推环（建 inode 形写成 60 个单元）	crates/singlefs-core/src/mounted_session.rs	        Err(refusal) => RootRingPushRehearsal::ChangeStillRefusedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device,\n            refusal: Box::new(refusal),\n        },	        Err(_refusal) => RootRingPushRehearsal::ChangeAdmittedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device,\n        },	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_sixty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting	a_sixty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting
第10z批第二十块 甲 去掉预演准入那一判、推完照旧被拒也列推环（建 inode 形写成 80 个单元）	crates/singlefs-core/src/mounted_session.rs	        Err(refusal) => RootRingPushRehearsal::ChangeStillRefusedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device,\n            refusal: Box::new(refusal),\n        },	        Err(_refusal) => RootRingPushRehearsal::ChangeAdmittedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device,\n        },	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- an_eighty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting	an_eighty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting
第10z批第二十块 甲 去掉预演准入那一判、推完照旧被拒也列推环（删除形写成 60 个单元）	crates/singlefs-core/src/mounted_session.rs	        Err(refusal) => RootRingPushRehearsal::ChangeStillRefusedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device,\n            refusal: Box::new(refusal),\n        },	        Err(_refusal) => RootRingPushRehearsal::ChangeAdmittedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device,\n        },	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_sixty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting	a_sixty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting
第10z批第二十块 甲 去掉预演准入那一判、推完照旧被拒也列推环（删除形写成 80 个单元）	crates/singlefs-core/src/mounted_session.rs	        Err(refusal) => RootRingPushRehearsal::ChangeStillRefusedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device,\n            refusal: Box::new(refusal),\n        },	        Err(_refusal) => RootRingPushRehearsal::ChangeAdmittedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device,\n        },	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- an_eighty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting	an_eighty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting
第10z批第二十块 甲 预演推完放得下也不列推环	crates/singlefs-core/src/mounted_session.rs	                ways_out.push(format!(\n                    "出路二	                drop(format!(\n                    "出路二	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push	a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push
第10z批第二十块 出路二写的推环次数与写掉的槽数恒为 0（正推 textCountZero）	crates/singlefs-core/src/mounted_session.rs	                    self.empty_publishes_until_the_ring_slot_of_r_is_written,\n                    self.empty_publishes_until_the_ring_slot_of_r_is_written,\n	                    0,\n                    0,\n	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push	a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push
第10z批第二十块 出路二的根环槽写成槽号.区域号（正推 textRegionSlot）	crates/singlefs-core/src/mounted_session.rs	                    self.ring_slot_of_r.region,\n                    self.ring_slot_of_r.slot,\n                    self.empty_publishes_until_the_ring_slot_of_r_is_written,	                    self.ring_slot_of_r.slot,\n                    self.ring_slot_of_r.region,\n                    self.empty_publishes_until_the_ring_slot_of_r_is_written,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push	a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push
第10z批第二十块 出路一在 R 那一槽没有根时照旧说 R 判成被抛弃（攻方第一节末）	crates/singlefs-core/src/mounted_session.rs	"出路一：正常卸载（mount::unmount）之后再可写挂载一次。R 那一槽（根环槽 {}.{}）没有读得出的有效根：再挂时择的是卸载推出来的最后一版，R 那几条记录不在它的水位之上，C 那一圈照它重算、不再有这一槽，宽护不再出现。",	"出路一：正常卸载（mount::unmount）之后再可写挂载一次。再挂时 R（那一槽没有读得出的有效根，根环槽 {}.{}）按新实例的实例表判成被抛弃，影子账只护它自己账里引用的落点，宽护不再出现。",	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push	a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push
第10z批第二十块 出路一不写 R 的账永久读不出时再挂被拒、会话已断开（正推 K1-2）	crates/singlefs-core/src/mounted_session.rs	R 的账读不出若是盘上永久坏，卸载之后再挂会被拒（C694 的 B），而会话在卸载时已断开。	R 的账再挂时仍读不出就照旧拒可写（C694 的 B）。	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_sixty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting	a_sixty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting
第10z批第二十块 出路二放出来的槽数取推之前的全部隔离（不是推完前后之差）	crates/singlefs-core/src/mounted_session.rs	                (*device, before.saturating_sub(after))	                (*device, before.saturating_sub(after).max(*before))	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- the_slots_the_way_out_by_the_push_releases_with_two_ring_slots_holding_overlapping_placements_are_what_the_push_releases	the_slots_the_way_out_by_the_push_releases_with_two_ring_slots_holding_overlapping_placements_are_what_the_push_releases
第10z批第二十块 攻方 M7 报的挡着的槽数取全部隔离（不限 C 那一圈罩着的）	crates/singlefs-core/src/mounted_session.rs	    let isolated_slots_on_each_device = allocator.isolated_slots_covered_by(&target.placements);	    let isolated_slots_on_each_device: Vec<(DeviceIdentity, u64)> = allocator.devices.iter().map(|map| (map.device, map.isolated_slots())).collect();	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- the_slots_the_way_out_by_the_push_releases_with_two_ring_slots_holding_overlapping_placements_are_what_the_push_releases	the_slots_the_way_out_by_the_push_releases_with_two_ring_slots_holding_overlapping_placements_are_what_the_push_releases
第10z批第二十块 推环入口不先判冻结（攻方第二节：预演在冻结着的分配器拷贝上照样过）	crates/singlefs-core/src/mounted_session.rs	        if let Some(frozen) = allocator.frozen_publish() {\n            return Err(\n                RootRingPushRefused::PublishFrozenAfterAWriteFailureIsNotResentYet {	        if let Some(frozen) = allocator.frozen_publish().filter(|_| false) {\n            return Err(\n                RootRingPushRefused::PublishFrozenAfterAWriteFailureIsNotResentYet {	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- pushing_the_root_ring_again_after_a_block_device_error_halfway_is_refused_as_frozen_before_any_write	pushing_the_root_ring_again_after_a_block_device_error_halfway_is_refused_as_frozen_before_any_write
第10z批第二十块 攻方 M1 推环真发失败不把会话的 current 挪到最后一次落盘的那一版	crates/singlefs-core/src/mounted_session.rs	                    *current = PoolVersion::of_a_version_of_trees(last_published);\n                    return Err(RootRingPushRefused::PublishFailedAfterTheRehearsal(	                    let _ = &last_published;\n                    return Err(RootRingPushRefused::PublishFailedAfterTheRehearsal(	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- pushing_the_root_ring_again_after_a_block_device_error_halfway_is_refused_as_frozen_before_any_write	pushing_the_root_ring_again_after_a_block_device_error_halfway_is_refused_as_frozen_before_any_write
第10z批第二十块 攻方 M2 推环入口不核交进来的盘表（直接用会话持着的参数）	crates/singlefs-core/src/mount.rs	    Ok(\n        caller_inputs_agreeing_with_the_disk(caller_parameters, devices, current)?\n            .parameters_of_the_pool,\n    )	    let _ = (devices, current);\n    Ok(caller_parameters.clone())	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- pushing_the_root_ring_with_the_two_device_bodies_swapped_is_refused_before_any_write	pushing_the_root_ring_with_the_two_device_bodies_swapped_is_refused_before_any_write
第10z批第二十块 C 记着落点而一槽都没隔离也报被宽护卡住（正推 noIsoCheck）	crates/singlefs-core/src/mounted_session.rs	    if isolated_slots_on_each_device\n        .iter()\n        .all(|(_, isolated)| *isolated == 0)	    if isolated_slots_on_each_device\n        .iter()\n        .all(|(_, isolated)| *isolated == u64::MAX)	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- no_space_while_the_ring_slots_hold_placements_none_of_which_is_isolated_is_not_reported_as_the_wide_protection	no_space_while_the_ring_slots_hold_placements_none_of_which_is_isolated_is_not_reported_as_the_wide_protection
第10z批第二十块 C 记着落点而一槽都没隔离时推环入口照推	crates/singlefs-core/src/mounted_session.rs	        if slots_the_push_would_release_isolation_of\n            .iter()\n            .all(|(_, isolated)| *isolated == 0)	        if slots_the_push_would_release_isolation_of\n            .iter()\n            .all(|(_, isolated)| *isolated == u64::MAX)	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- pushing_the_root_ring_while_the_ring_slots_hold_placements_none_of_which_is_isolated_writes_nothing	pushing_the_root_ring_while_the_ring_slots_hold_placements_none_of_which_is_isolated_writes_nothing
第10z批第二十块 记着空串的槽也算 C 那一圈记着落点的槽（正推 emptyHeld）	crates/singlefs-core/src/allocator.rs	            .filter(|(_, placements)| !placements.is_empty())\n            .map(|(ring_slot, placements)| (*ring_slot, placements.clone()))	            .map(|(ring_slot, placements)| (*ring_slot, placements.clone()))	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_writes_up_to	a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_writes_up_to
第10z批第二十块 推环交回写掉的那几槽恒空（正推 slotsWrittenEmpty）	crates/singlefs-core/src/mounted_session.rs	            ring_slots_written: target.ring_slots,	            ring_slots_written: Vec::new(),	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_writes_up_to	a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_writes_up_to
第10z批第二十块 攻方 M12 R 那一槽取最先轮到的那一槽（两槽 C）	crates/singlefs-core/src/mounted_session.rs	        .max()?;\n    Some(RootRingPushTarget {	        .min()?;\n    Some(RootRingPushTarget {	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- pushing_the_root_ring_with_two_ring_slots_holding_placements_writes_up_to_the_later_one_and_releases_both	pushing_the_root_ring_with_two_ring_slots_holding_placements_writes_up_to_the_later_one_and_releases_both
第10z批第二十块 停因是一次准入的发布数做满也报被宽护卡住（正推 budgetAsStuck）	crates/singlefs-core/src/mounted_session.rs	        FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => {}\n        FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. }\n        | FloorRaiseStop::FloorRaiseRefusedForSpace { .. } => {	        FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } | FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. } => {}\n        FloorRaiseStop::FloorRaiseRefusedForSpace { .. } => {	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- no_space_when_the_publishes_of_one_admission_run_out_while_the_ring_slots_hold_isolated_placements_is_not_reported_as_the_wide_protection	no_space_when_the_publishes_of_one_admission_run_out_while_the_ring_slots_hold_isolated_placements_is_not_reported_as_the_wide_protection
第10z批第二十块 停因是抬 F 那一串取不到落点也报被宽护卡住（正推 spaceAsStuck）	crates/singlefs-core/src/mounted_session.rs	        FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => {}\n        FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. }\n        | FloorRaiseStop::FloorRaiseRefusedForSpace { .. } => {	        FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } | FloorRaiseStop::FloorRaiseRefusedForSpace { .. } => {}\n        FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. } => {	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- no_space_when_the_floor_raise_finds_no_placement_while_the_ring_slots_hold_isolated_placements_is_not_reported_as_the_wide_protection	no_space_when_the_floor_raise_finds_no_placement_while_the_ring_slots_hold_isolated_placements_is_not_reported_as_the_wide_protection
````

### 第二十块变异表整行替换（6 行）

````tsv
第10z批第十六块 推满仍不够时不分宽护：F 在上限、C 那一圈隔离着也照旧只报空间不够（回退、前缀 20 次）	crates/singlefs-core/src/mounted_session.rs	                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(\n                    parameters,\n                    devices.as_slice(),\n                    allocator,\n                    current,\n                    instance,\n                    shadow_ledger,\n                    remainder,\n                    &stop,\n                ),	                wide_protection_at_the_ceiling: WideProtectionAtTheCeiling::NotWhatStopsTheAdmission,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- wide_protection	a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through
第10z批第十六块 推满仍不够时不分宽护：F 在上限、C 那一圈隔离着也照旧只报空间不够（建 inode、前缀 40 次）	crates/singlefs-core/src/mounted_session.rs	                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(\n                    parameters,\n                    devices.as_slice(),\n                    allocator,\n                    current,\n                    instance,\n                    shadow_ledger,\n                    remainder,\n                    &stop,\n                ),	                wide_protection_at_the_ceiling: WideProtectionAtTheCeiling::NotWhatStopsTheAdmission,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through	an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through
第10z批第十六块 推满仍不够时不分宽护：F 在上限、C 那一圈隔离着也照旧只报空间不够（删除、前缀 160 次）	crates/singlefs-core/src/mounted_session.rs	                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(\n                    parameters,\n                    devices.as_slice(),\n                    allocator,\n                    current,\n                    instance,\n                    shadow_ledger,\n                    remainder,\n                    &stop,\n                ),	                wide_protection_at_the_ceiling: WideProtectionAtTheCeiling::NotWhatStopsTheAdmission,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through	a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through
第10z批第十六块 推满仍不够时不分宽护：F 在上限、C 那一圈隔离着也照旧只报空间不够（卸载重挂那一条出路）	crates/singlefs-core/src/mounted_session.rs	                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(\n                    parameters,\n                    devices.as_slice(),\n                    allocator,\n                    current,\n                    instance,\n                    shadow_ledger,\n                    remainder,\n                    &stop,\n                ),	                wide_protection_at_the_ceiling: WideProtectionAtTheCeiling::NotWhatStopsTheAdmission,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_r_and_lets_writes_through	unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_r_and_lets_writes_through
第10z批第十六块 推满仍不够时不分宽护：F 在上限、C 那一圈隔离着也照旧只报空间不够（推到半路取不到落点那一形）	crates/singlefs-core/src/mounted_session.rs	                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(\n                    parameters,\n                    devices.as_slice(),\n                    allocator,\n                    current,\n                    instance,\n                    shadow_ledger,\n                    remainder,\n                    &stop,\n                ),	                wide_protection_at_the_ceiling: WideProtectionAtTheCeiling::NotWhatStopsTheAdmission,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write	pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write
第10z批第九块 K3 loop-span1 根槽这一遍读坏的被抛弃根记着的落点补隔离只隔起点那一槽（正推 loop-span1：跨两槽的落点后一槽空闲）	crates/singlefs-core/src/mount.rs	        allocator.isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(\n            placement.device,\n            placement.slot,\n            placement.span,\n        );\n    }\n    // C 那一份不豁免	        allocator.isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(\n            placement.device,\n            placement.slot,\n            1,\n        );\n    }\n    // C 那一份不豁免	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_isolates_the_later_slot_of_its_recorded_two_slot_placement_left_unisolated	unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_isolates_the_later_slot_of_its_recorded_two_slot_placement_left_unisolated
````

## 第二十一块

````diff
diff --git a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
index d2086c6..0a05b83 100644
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -2520,7 +2520,7 @@ mod tests {
         }
     }
 
-    /// `code` 拆成的词：标识符与关键字（`r#` 原始标识符取它后面的名字）、生命周期（连着它前面的 `'`）、数字（照标识符那样收一段）、`::`，
+    /// `code` 拆成的词：标识符与关键字（`r#` 原始标识符取它后面的名字）、生命周期（连着它前面的 `'`）、数字（照标识符那样收一段）、`::`、`!=`，
     /// 其余标点一个字一个词；注释（行注释、可以嵌套的块注释）、字符串与字符字面量（普通的、原始的 `r#"…"#`、字节的）不出词。
     /// 扫源码的用例照它找条目、`use` 引进来的名字与花括号的层数：字面量与注释里的 `mod`、`use`、花括号都不算。
     /// 非 ASCII 的字只落在注释、字面量与标识符里，切出来的词因此总在字符边界上。
@@ -2578,6 +2578,10 @@ mod tests {
             } else if rest.starts_with("::") {
                 tokens.push("::");
                 position += 2;
+            } else if rest.starts_with("!=") {
+                // 不等号整个一个词：拆成 `!`、`=` 时，条目层 `const 名: bool = 甲 != 乙;` 的 `甲 !` 会被认成调宏
+                tokens.push("!=");
+                position += 2;
             } else {
                 tokens.push(&rest[..first_character.len_utf8()]);
                 position += first_character.len_utf8();
@@ -2614,49 +2618,209 @@ mod tests {
         panic!("卡池模块有与它的左花括号配对的右花括号")
     }
 
-    /// 一条 `use`（`statement` 从 `use` 那个词到分号，至多一层花括号）引进来的每个名字连同引它的路径：名字是后面紧跟逗号、
-    /// 右花括号或分号的标识符；路径是它前面连着的 `段::` 那几段，在花括号里的再接上左花括号之前那一段，去掉收尾的 `::`。
-    fn names_brought_in_by_a_use_statement<'code>(
-        statement: &[&'code str],
-    ) -> Vec<(String, &'code str)> {
-        let is_identifier = |token: &str| {
-            token.starts_with(|character: char| character.is_alphanumeric() || character == '_')
-        };
-        let opening_brace = statement.iter().position(|token| *token == "\u{7b}");
-        statement
+    /// 一条 `use` 引进来的一样东西。
+    enum BroughtInByAUse {
+        /// 引进一个名字：`source` 是被引的那一处的全路径（`甲::乙`；花括号里的 `self` 取花括号之前那一段），
+        /// `name` 是它在这里叫什么（有 `as` 的取改成的那个名字）。
+        Name { source: String, name: String },
+        /// 通配（`甲::乙::*`）：引进来哪些名字，扫这一个模块的源码看不见。
+        Glob { source: String },
+    }
+
+    /// `outer_path` 后面接上 `segments`，用 `::` 连起来（`outer_path` 空着就只是 `segments`）。
+    fn path_joined(outer_path: &str, segments: &[&str]) -> String {
+        outer_path
+            .split("::")
+            .filter(|segment| !segment.is_empty())
+            .chain(segments.iter().copied())
+            .collect::<Vec<&str>>()
+            .join("::")
+    }
+
+    /// `statement`（一条 `use` 从 `use` 那个词到分号的词）从 `start` 起的一棵 use 树（`甲::乙`、`甲::乙 as 丙`、`甲::self`、`甲::*`、
+    /// `甲::{…}`，花括号可以套几层）引进来的东西，路径前面接着外层花括号之前那一段 `outer_path`；连同这棵树收尾之后那个词的位置
+    /// （逗号、右花括号或分号）。
+    fn brought_in_by_a_use_tree(
+        statement: &[&str],
+        start: usize,
+        outer_path: &str,
+    ) -> (Vec<BroughtInByAUse>, usize) {
+        let mut segments: Vec<&str> = Vec::new();
+        let mut position = start;
+        loop {
+            let token = *statement.get(position).expect("use 树在分号之前收尾");
+            if token == "::" {
+                position += 1;
+            } else if token == "*" {
+                let glob = BroughtInByAUse::Glob {
+                    source: path_joined(outer_path, &segments),
+                };
+                return (vec![glob], position + 1);
+            } else if token == "\u{7b}" {
+                let group_path = path_joined(outer_path, &segments);
+                let mut brought_in = Vec::new();
+                position += 1;
+                while statement[position] != "\u{7d}" {
+                    let (brought_in_by_the_inner_tree, after_the_inner_tree) =
+                        brought_in_by_a_use_tree(statement, position, &group_path);
+                    brought_in.extend(brought_in_by_the_inner_tree);
+                    position =
+                        after_the_inner_tree + usize::from(statement[after_the_inner_tree] == ",");
+                }
+                return (brought_in, position + 1);
+            } else {
+                segments.push(token);
+                position += 1;
+                if statement.get(position) == Some(&"::") {
+                    continue;
+                }
+                let is_self = token == "self";
+                let source = path_joined(
+                    outer_path,
+                    &segments[..segments.len() - usize::from(is_self)],
+                );
+                let name_without_a_rename = if is_self {
+                    source
+                        .rsplit("::")
+                        .next()
+                        .expect("rsplit 至少交一段")
+                        .to_string()
+                } else {
+                    token.to_string()
+                };
+                let (name, after_the_tree) = if statement.get(position) == Some(&"as") {
+                    (statement[position + 1].to_string(), position + 2)
+                } else {
+                    (name_without_a_rename, position)
+                };
+                return (vec![BroughtInByAUse::Name { source, name }], after_the_tree);
+            }
+        }
+    }
+
+    /// 钉住那两段（借之前那一步、借据丢掉时那一步）按名字调的函数，各连同它该从哪一处 `use` 进卡池模块。
+    const FUNCTIONS_THE_PINNED_BODIES_CALL_BY_NAME: [(&str, &str); 4] = [
+        ("super::super", "device_found_idle_by_a_poll"),
+        ("super::super", "poll_type_before_returning_a_lease"),
+        ("super::super", "device_wait_before_returning_a_lease"),
+        (
+            "singlefs_harness::gpu_memory_ledger",
+            "wait_for_the_device_and_take_in_the_outcome",
+        ),
+    ];
+
+    /// `code`（这份源码测试那一段之外，或用例造的一段）的卡池模块里，能把钉住那两段按路径首段解析的 `std`、`wgpu` 与按名字调的
+    /// 那几个函数换掉的写法，一处一句；一处没有交空。判法与名单在 `the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call`。
+    fn ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(
+        code: &str,
+    ) -> Vec<String> {
+        let module_tokens = card_pool_module_tokens_with_their_brace_depth(code);
+        let tokens: Vec<&str> = module_tokens.iter().map(|(_depth, token)| *token).collect();
+        let mut found = Vec::new();
+        if tokens.contains(&"mod") {
+            found.push(
+                "卡池模块里另起模块：叫 std、wgpu 的直接遮住外部 crate，别的名字的经 #[macro_use] 把宏漏到这一层、mod 名字; 与 #[path] 的源码不在这一段里"
+                    .to_string(),
+            );
+        }
+        for pair in tokens.windows(2) {
+            if pair == ["extern", "crate"] {
+                found.push(
+                    "卡池模块里写 extern crate：extern crate … as std 遮住外部 crate".to_string(),
+                );
+            }
+        }
+        for pair in module_tokens.windows(2) {
+            if pair[0].0 == 0
+                && pair[1].1 == "!"
+                && pair[0]
+                    .1
+                    .starts_with(|character: char| character.is_alphanumeric() || character == '_')
+            {
+                found.push(format!(
+                    "卡池模块的条目这一层调宏或定义宏 {}!：宏展开出来的条目也能叫 std、wgpu",
+                    pair[0].1
+                ));
+            }
+        }
+        let use_statements: Vec<&[&str]> = tokens
             .iter()
             .enumerate()
-            .skip(1)
-            .filter(|(leaf_index, token)| {
-                is_identifier(token)
-                    && statement
-                        .get(leaf_index + 1)
-                        .is_some_and(|next| matches!(*next, "," | "\u{7d}" | ";"))
+            .filter(|(_index, token)| **token == "use")
+            .map(|(use_index, _token)| {
+                let length = tokens[use_index..]
+                    .iter()
+                    .position(|token| *token == ";")
+                    .expect("use 以分号收尾")
+                    + 1;
+                &tokens[use_index..use_index + length]
             })
-            .map(|(leaf_index, leaf)| {
-                let mut path_start = leaf_index;
-                while path_start >= 3
-                    && statement[path_start - 1] == "::"
-                    && is_identifier(statement[path_start - 2])
-                {
-                    path_start -= 2;
-                }
-                let mut path = statement[path_start..leaf_index].join("");
-                if let Some(brace) = opening_brace.filter(|brace| *brace < leaf_index) {
-                    path = format!("{}{path}", statement[1..brace].join(""));
+            .collect();
+        let mut brought_in = Vec::new();
+        for statement in &use_statements {
+            let (brought_in_by_the_statement, end_of_the_tree) =
+                brought_in_by_a_use_tree(statement, 1, "");
+            assert_eq!(
+                end_of_the_tree,
+                statement.len() - 1,
+                "use 树读到分号收尾：{statement:?}"
+            );
+            brought_in.extend(brought_in_by_the_statement);
+        }
+        for brought_in_by_a_use in &brought_in {
+            match brought_in_by_a_use {
+                BroughtInByAUse::Glob { source } => found.push(format!(
+                    "卡池模块里通配 use {source}::*：引进来的名字（叫 std、wgpu 的模块也算）扫这一段看不见"
+                )),
+                BroughtInByAUse::Name { source, name } => {
+                    if matches!(name.as_str(), "std" | "wgpu") {
+                        found.push(format!(
+                            "卡池模块经 use 引进叫 {name} 的名字（引的是 {source}），遮住外部 crate"
+                        ));
+                    }
                 }
-                (path.trim_end_matches("::").to_string(), *leaf)
-            })
-            .collect()
+            }
+        }
+        for (expected_prefix, function) in FUNCTIONS_THE_PINNED_BODIES_CALL_BY_NAME {
+            let sources_brought_in_under_the_name: Vec<&str> = brought_in
+                .iter()
+                .filter_map(|brought_in_by_a_use| match brought_in_by_a_use {
+                    BroughtInByAUse::Name { source, name } if name == function => {
+                        Some(source.as_str())
+                    }
+                    BroughtInByAUse::Name { .. } | BroughtInByAUse::Glob { .. } => None,
+                })
+                .collect();
+            let expected_source = format!("{expected_prefix}::{function}");
+            if sources_brought_in_under_the_name != [expected_source.as_str()] {
+                found.push(format!(
+                    "钉住那两段按名字调的 {function} 要由一条 use 从 {expected_source} 引进卡池模块，引进来的是 {sources_brought_in_under_the_name:?}"
+                ));
+            }
+        }
+        found
     }
 
     /// 第 10z 批第十八块（第 10z 批第十四、十五块代码轮判决 K4，攻方：卡池模块里加 `mod wgpu { … }` 或 `mod std { … }` 遮住方法体里的
     /// `wgpu::PollType::Poll`、`std::panic::catch_unwind`，方法体一个字不动、`cargo check` 0 警告编过，整句比照绿）：钉住的两段（借之前那一步、
     /// 借据丢掉时那一步）里按路径首段解析的 `std`、`wgpu` 与按名字调的那几个函数，在卡池模块这一层只落到外部 crate 与 `use` 点名的那一处。
-    /// 这一层能把一个名字换掉的只有这几样，逐样拒：另起模块、`extern crate`、在条目这一层调宏或定义宏（宏展开出来的条目也能叫 `std`、`wgpu`）、
-    /// `use` 改名（`as`）、引 `self`、通配（`*`，通配引进来的模块同样遮住外部 crate）、`use` 引进叫 `std` 或 `wgpu` 的名字；
-    /// 那几个函数各由一条 `use` 从钉住的那一处引进来（引了它，模块里再起同名的条目编译不过；去掉 `use` 另起一个就是这里红）。
-    /// 扫这份源码（测试那一段之外，注释与字面量不算），不要卡。
+    /// 这一层能把一个名字换掉的写法，逐样拒：`use` 引进叫 `std` 或 `wgpu` 的名字（改名 `as`、花括号里的 `self`、几层花括号里的都照引进来的名字认）、
+    /// `extern crate`；那几个函数各由一条 `use` 从钉住的那一处引进来（引了它，模块里再起同名的条目编译不过；去掉 `use` 另起一个、
+    /// 或 `use 别处::某函数 as 那个名字` 就是这里红）。扫这份源码（测试那一段之外，注释与字面量不算），不要卡。
+    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K5，正推：以后合法的写法被判红）逐类判过：`!=` 被当成调宏是误判，改了；
+    /// `use … as 别的名字`、两层花括号、`use 甲::{self, …}`、`extern "C" { … }` 换不掉钉住那两段用的名字，是误判，改了
+    /// （`the_card_pool_scan_reads_an_item_level_not_equal_as_a_comparison_and_still_finds_an_item_level_macro`、
+    /// `the_card_pool_scan_accepts_renames_nested_groups_and_self_that_bring_in_neither_std_nor_wgpu`、
+    /// `the_card_pool_scan_finds_std_wgpu_or_a_pinned_function_brought_in_under_a_rename_a_nested_group_or_self`、
+    /// `the_card_pool_scan_accepts_an_extern_block_and_still_finds_extern_crate`）。
+    ///
+    /// 故意拒的写法（换不换得掉那几个名字，扫这一个模块的源码判不出），一种一条，各写为什么
+    /// （`the_card_pool_scan_refuses_globs_item_level_macros_and_inner_modules_even_when_they_name_nothing_pinned`）：
+    /// - 通配 `use 甲::*`：引进来哪些名字要看 `甲` 里有什么，`甲` 里有叫 `std`、`wgpu` 的模块就遮住外部 crate；要用就逐个点名；
+    /// - 条目这一层调宏或定义宏（`thread_local!` 也算）：宏名按文本作用域解析，卡池模块外面先定义的同名 `macro_rules!` 会顶替标准库那一个，
+    ///   展开出来的条目也能叫 `std`、`wgpu`；要用就放进函数体里；
+    /// - 另起模块（`#[cfg(test)] mod …` 也算）：叫 `std`、`wgpu` 的直接遮住外部 crate，别的名字的经 `#[macro_use]` 把里面的宏漏到这一层，
+    ///   `mod 名字;` 与 `#[path]` 的源码不在这一段里；单测写在这份文件的测试模块里。
     ///
     /// 拦不住的写法（扫这一个模块的源码看不见），一种一条：
     /// - 过程宏（派生宏、属性宏）在卡池模块里展开出来的条目；
@@ -2671,89 +2835,170 @@ mod tests {
             .split(TESTS_MODULE_OPENING)
             .next()
             .expect("split 至少交一段");
-        let module_tokens = card_pool_module_tokens_with_their_brace_depth(code);
-        let tokens: Vec<&str> = module_tokens.iter().map(|(_depth, token)| *token).collect();
-        let declaring_keywords: Vec<&str> = tokens
-            .iter()
-            .copied()
-            .filter(|token| matches!(*token, "mod" | "extern"))
-            .collect();
         assert_eq!(
-            declaring_keywords,
-            Vec::<&str>::new(),
-            "卡池模块里不另起模块、不写 extern crate：`mod wgpu`、`mod std`、`extern crate … as std` 都遮住钉住那两段里按路径首段解析的外部 crate"
+            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(code),
+            Vec::<String>::new(),
+            "卡池模块这一层不换掉钉住那两段按路径首段解析的 std、wgpu 与按名字调的那几个函数"
         );
-        let item_level_macros: Vec<&str> = module_tokens
-            .windows(2)
-            .filter(|pair| {
-                pair[0].0 == 0
-                    && pair[1].1 == "!"
-                    && pair[0].1.starts_with(|character: char| {
-                        character.is_alphanumeric() || character == '_'
-                    })
-            })
-            .map(|pair| pair[0].1)
-            .collect();
+    }
+
+    /// 用例造的卡池模块：头上照真源码那样把钉住那两段按名字调的四个函数各 `use` 进来，再接 `items`（花括号写成 \u 转义）。
+    fn card_pool_module_with_the_pinned_uses_and(items: &str) -> String {
+        let mut module = String::from(concat!(
+            "mod card_pool \u{7b}\n",
+            "    use super::super::\u{7b}\n",
+            "        device_found_idle_by_a_poll, device_wait_before_returning_a_lease,\n",
+            "        poll_type_before_returning_a_lease,\n",
+            "    \u{7d};\n",
+            "    use singlefs_harness::gpu_memory_ledger::wait_for_the_device_and_take_in_the_outcome;\n",
+        ));
+        module.push_str(items);
+        module.push_str("\n\u{7d}\n");
+        module
+    }
+
+    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K5，正推 `K5_const_ne`、`K5_const_ne_ident`）：条目这一层的
+    /// `const 名: bool = 甲 != 乙;` 是比较，不是调宏 `甲!`；条目这一层真调宏（`thread_local!`）照旧拒。不要卡。
+    #[test]
+    fn the_card_pool_scan_reads_an_item_level_not_equal_as_a_comparison_and_still_finds_an_item_level_macro(
+    ) {
+        let comparisons = card_pool_module_with_the_pinned_uses_and(concat!(
+            "    const FIRST_LIMIT: u8 = 1;\n",
+            "    const SECOND_LIMIT: u8 = 2;\n",
+            "    const THE_LIMITS_DIFFER: bool = FIRST_LIMIT != SECOND_LIMIT;\n",
+            "    const THE_LITERALS_DIFFER: bool = 1 != 2;\n",
+            "    const THE_LIMITS_DIFFER_WITHOUT_SPACES: bool = FIRST_LIMIT!=SECOND_LIMIT;",
+        ));
         assert_eq!(
-            item_level_macros,
-            Vec::<&str>::new(),
-            "卡池模块的条目这一层不调宏、不定义宏：宏展开出来的条目也能叫 std、wgpu"
+            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&comparisons),
+            Vec::<String>::new(),
+            "条目这一层的 != 是比较，不算调宏：{comparisons}"
         );
-        let use_statements: Vec<&[&str]> = tokens
-            .iter()
-            .enumerate()
-            .filter(|(_index, token)| **token == "use")
-            .map(|(use_index, _token)| {
-                let length = tokens[use_index..]
-                    .iter()
-                    .position(|token| *token == ";")
-                    .expect("use 以分号收尾")
-                    + 1;
-                &tokens[use_index..use_index + length]
-            })
-            .collect();
-        for statement in &use_statements {
-            assert!(
-                !statement
-                    .iter()
-                    .any(|token| matches!(*token, "as" | "self" | "*")),
-                "卡池模块的 use 不改名、不引 self、不通配（这几样都能把一个外部 crate 的名字换成别的模块）：{statement:?}"
-            );
-            assert!(
-                statement.iter().filter(|token| **token == "\u{7b}").count() <= 1,
-                "卡池模块的 use 至多一层花括号（引进来的名字照这一层认）：{statement:?}"
+        let item_level_macro = card_pool_module_with_the_pinned_uses_and(
+            "    thread_local! \u{7b} static COUNT: std::cell::Cell<u8> = std::cell::Cell::new(0); \u{7d}",
+        );
+        assert_ne!(
+            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&item_level_macro),
+            Vec::<String>::new(),
+            "条目这一层调宏照旧拒：{item_level_macro}"
+        );
+    }
+
+    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K5，正推 `K5_use_as`、`K5_nested_use_braces`、`K5_pub_use_self_group`）：
+    /// 改成别的名字、两层花括号、花括号里的 `self` 引进来的名字既不叫 `std`、`wgpu`，也不是钉住那两段调的函数，不拒。不要卡。
+    #[test]
+    fn the_card_pool_scan_accepts_renames_nested_groups_and_self_that_bring_in_neither_std_nor_wgpu(
+    ) {
+        let module = card_pool_module_with_the_pinned_uses_and(concat!(
+            "    use std::sync::Arc as SharedArc;\n",
+            "    use std::collections::\u{7b}BTreeMap, btree_map::\u{7b}Entry\u{7d}\u{7d};\n",
+            "    use std::sync::\u{7b}self, Mutex\u{7d};\n",
+            "    use ::std::fmt::\u{7b}Display as Shown,\u{7d};",
+        ));
+        assert_eq!(
+            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&module),
+            Vec::<String>::new(),
+            "这几条 use 引进来的是 SharedArc、BTreeMap、Entry、sync、Mutex、Shown：{module}"
+        );
+    }
+
+    /// 第 10z 批第二十一块（同上一条的反面）：改名、套在里层花括号里、花括号里的 `self` 引进叫 `std`、`wgpu` 的名字照旧拒；
+    /// 钉住那两段调的函数换成 `use 别处::某函数 as 那个名字` 也拒。不要卡。
+    #[test]
+    fn the_card_pool_scan_finds_std_wgpu_or_a_pinned_function_brought_in_under_a_rename_a_nested_group_or_self(
+    ) {
+        for (form, items) in [
+            (
+                "改名成 std",
+                "    use super::a_module_shadowing_the_crate as std;",
+            ),
+            (
+                "里层花括号里的 wgpu",
+                "    use super::\u{7b}outer::\u{7b}wgpu, other\u{7d}\u{7d};",
+            ),
+            (
+                "花括号里的 self 引进 wgpu",
+                "    use super::shadowing::wgpu::\u{7b}self\u{7d};",
+            ),
+            (
+                "花括号里的 self 改名成 std",
+                "    use super::\u{7b}shadowing::\u{7b}self as std\u{7d}\u{7d};",
+            ),
+        ] {
+            let module = card_pool_module_with_the_pinned_uses_and(items);
+            assert_ne!(
+                ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&module),
+                Vec::<String>::new(),
+                "{form} 拒：{module}"
             );
         }
-        let brought_in: Vec<(String, &str)> = use_statements
-            .iter()
-            .flat_map(|statement| names_brought_in_by_a_use_statement(statement))
-            .collect();
-        let crate_roots_brought_in: Vec<&(String, &str)> = brought_in
-            .iter()
-            .filter(|(_prefix, name)| matches!(*name, "std" | "wgpu"))
-            .collect();
+        let pinned_function_renamed_in = card_pool_module_with_the_pinned_uses_and("")
+            .replacen("device_found_idle_by_a_poll, ", "", 1)
+            .replacen(
+                "    use singlefs_harness",
+                "    use super::super::a_poll_that_always_finds_the_device_idle as device_found_idle_by_a_poll;\n    use singlefs_harness",
+                1,
+            );
+        assert!(
+            !pinned_function_renamed_in.contains("device_found_idle_by_a_poll, "),
+            "造的模块去掉了从钉住那一处引的 device_found_idle_by_a_poll：{pinned_function_renamed_in}"
+        );
         assert_eq!(
-            crate_roots_brought_in,
-            Vec::<&(String, &str)>::new(),
-            "卡池模块不经 use 引进叫 std、wgpu 的名字"
+            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(
+                &pinned_function_renamed_in
+            ),
+            vec![
+                "钉住那两段按名字调的 device_found_idle_by_a_poll 要由一条 use 从 super::super::device_found_idle_by_a_poll 引进卡池模块，引进来的是 [\"super::super::a_poll_that_always_finds_the_device_idle\"]"
+                    .to_string()
+            ],
+            "别处的函数改名成钉住那两段调的名字引进来，拒：{pinned_function_renamed_in}"
+        );
+    }
+
+    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K5，正推 `K5_extern_c_block`）：`extern "C" { … }` 里声明的外部函数
+    /// 在值的名字空间，遮不住按路径首段解析的 `std`、`wgpu`，与 `use` 进来的同名函数撞了编译不过，不拒；`extern crate` 照旧拒。不要卡。
+    #[test]
+    fn the_card_pool_scan_accepts_an_extern_block_and_still_finds_extern_crate() {
+        let extern_block = card_pool_module_with_the_pinned_uses_and(
+            "    extern \"C\" \u{7b} fn abs(input: i32) -> i32; \u{7d}",
+        );
+        assert_eq!(
+            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&extern_block),
+            Vec::<String>::new(),
+            "extern 块不拒：{extern_block}"
+        );
+        let extern_crate =
+            card_pool_module_with_the_pinned_uses_and("    extern crate alloc as std;");
+        assert_ne!(
+            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&extern_crate),
+            Vec::<String>::new(),
+            "extern crate 照旧拒：{extern_crate}"
         );
-        for (expected_prefix, function) in [
-            ("super::super", "device_found_idle_by_a_poll"),
-            ("super::super", "poll_type_before_returning_a_lease"),
-            ("super::super", "device_wait_before_returning_a_lease"),
+    }
+
+    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K5，正推 `K5_use_wildcard_enum`、`K5_thread_local_macro`、
+    /// `K5_cfg_test_mod_inside`）：故意拒的三类（为什么写在
+    /// `the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call` 的名单里），
+    /// 写的东西本身不叫 `std`、`wgpu` 也拒。不要卡。
+    #[test]
+    fn the_card_pool_scan_refuses_globs_item_level_macros_and_inner_modules_even_when_they_name_nothing_pinned(
+    ) {
+        for (form, items) in [
+            ("通配", "    use std::cmp::Ordering::*;"),
             (
-                "singlefs_harness::gpu_memory_ledger",
-                "wait_for_the_device_and_take_in_the_outcome",
+                "条目这一层调宏",
+                "    thread_local! \u{7b} static COUNT: std::cell::Cell<u8> = std::cell::Cell::new(0); \u{7d}",
+            ),
+            (
+                "另起模块",
+                "    #[cfg(test)]\n    mod unit_checks_of_the_pool \u{7b} \u{7d}",
             ),
         ] {
-            assert_eq!(
-                brought_in
-                    .iter()
-                    .filter(|(_prefix, name)| *name == function)
-                    .map(|(prefix, _name)| prefix.as_str())
-                    .collect::<Vec<&str>>(),
-                vec![expected_prefix],
-                "钉住那两段按名字调的 {function} 由一条 use 从 {expected_prefix} 引进卡池模块（引了它，模块里再起同名的条目编译不过）"
+            let module = card_pool_module_with_the_pinned_uses_and(items);
+            assert_ne!(
+                ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&module),
+                Vec::<String>::new(),
+                "{form} 故意拒：{module}"
             );
         }
     }
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
index 7ee421b..77162ee 100644
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -4659,7 +4659,11 @@ mod tests {
 
     /// 这份源文件测试模块之外的代码行（注释行不算）。
     fn code_lines_outside_the_tests() -> Vec<String> {
-        let source = include_str!("crash_amplification.rs");
+        code_lines_outside_the_tests_of(include_str!("crash_amplification.rs"))
+    }
+
+    /// `source`（这份源文件，或用例在它上面改过的一份）测试模块之外的代码行（注释行不算）。
+    fn code_lines_outside_the_tests_of(source: &str) -> Vec<String> {
         let tests_start = source
             .find(TESTS_MODULE_OPENING)
             .expect("这份文件有测试模块");
@@ -4828,18 +4832,104 @@ mod tests {
     /// `line` 里 `stop_is_requested` 作为整个名字出现了几次（前后不连着字母、数字、下划线：`stop_is_requested_by_the_environment`、
     /// `_stop_is_requested` 不算）。
     fn whole_name_uses_of_the_stop_request(line: &str) -> usize {
-        let name = concat!("stop_is_", "requested");
+        whole_word_positions(line, STOP_REQUEST_PARAMETER_NAME).len()
+    }
+
+    /// 收叫停的那几层给叫停参数起的名字。
+    const STOP_REQUEST_PARAMETER_NAME: &str = concat!("stop_is_", "requested");
+
+    /// `text` 里 `word` 作为整个名字出现的每一处起点（前后不连着字母、数字、下划线）。
+    fn whole_word_positions(text: &str, word: &str) -> Vec<usize> {
         let is_part_of_a_name = |neighbour: Option<char>| {
             neighbour.is_some_and(|neighbour_character| {
                 neighbour_character.is_alphanumeric() || neighbour_character == '_'
             })
         };
-        line.match_indices(name)
-            .filter(|(position, _name)| {
-                !is_part_of_a_name(line[..*position].chars().next_back())
-                    && !is_part_of_a_name(line[position + name.len()..].chars().next())
+        text.match_indices(word)
+            .filter(|(position, _word)| {
+                !is_part_of_a_name(text[..*position].chars().next_back())
+                    && !is_part_of_a_name(text[position + word.len()..].chars().next())
             })
-            .count()
+            .map(|(position, _word)| position)
+            .collect()
+    }
+
+    /// 叫停参数的类型，去掉空白。
+    const STOP_REQUEST_TYPE_WITHOUT_SPACES: &str = "&(dynFn()->bool+Sync)";
+
+    /// `text` 跳过空白读下去恰以 `pattern_without_spaces` 起头时，那一截（连同中间的空白）收尾之后的位置；不是就交 `None`。
+    fn end_of_a_prefix_ignoring_whitespace(
+        text: &str,
+        pattern_without_spaces: &str,
+    ) -> Option<usize> {
+        let mut pattern_characters = pattern_without_spaces.chars().peekable();
+        for (offset, character) in text.char_indices() {
+            let Some(expected_character) = pattern_characters.peek() else {
+                return Some(offset);
+            };
+            if character.is_whitespace() {
+                continue;
+            }
+            if character != *expected_character {
+                return None;
+            }
+            pattern_characters.next();
+        }
+        pattern_characters.peek().is_none().then_some(text.len())
+    }
+
+    /// 源码里写着叫停类型的一处（参数、`let` 的类型标注、字段都算）。
+    struct PlaceTakingTheStopRequest<'code> {
+        /// 往前最近的 `fn` 后面那个名字（往前没有 `fn` 的交空串）。
+        layer: &'code str,
+        /// 类型前面 `名字:` 的那个名字（前面不是 `名字:` 的交空串）。
+        parameter: &'code str,
+        /// 类型收尾之后的位置。
+        end_of_the_type: usize,
+    }
+
+    /// `code` 里每一处叫停类型，照源码次序：类型里的空白随意、可以跨行，参数与别的参数挤在签名那一行、参数换了名字的都找得到。
+    fn places_taking_the_stop_request(code: &str) -> Vec<PlaceTakingTheStopRequest<'_>> {
+        let is_identifier_character =
+            |character: char| character.is_alphanumeric() || character == '_';
+        let identifier_at_the_end_of = |text: &str| -> usize {
+            text.char_indices()
+                .rev()
+                .take_while(|(_index, character)| is_identifier_character(*character))
+                .last()
+                .map_or(text.len(), |(index, _character)| index)
+        };
+        code.match_indices('&')
+            .filter_map(|(ampersand, _ampersand)| {
+                let end_of_the_type = ampersand
+                    + end_of_a_prefix_ignoring_whitespace(
+                        &code[ampersand..],
+                        STOP_REQUEST_TYPE_WITHOUT_SPACES,
+                    )?;
+                let before_the_type = code[..ampersand].trim_end();
+                let parameter = before_the_type
+                    .strip_suffix(':')
+                    .filter(|before_the_colon| !before_the_colon.ends_with(':'))
+                    .map_or("", |before_the_colon| {
+                        let before_the_name = before_the_colon.trim_end();
+                        &before_the_name[identifier_at_the_end_of(before_the_name)..]
+                    });
+                let layer = whole_word_positions(&code[..ampersand], "fn")
+                    .last()
+                    .map_or("", |fn_keyword| {
+                        let after_the_keyword = code[fn_keyword + 2..ampersand].trim_start();
+                        let name_length = after_the_keyword
+                            .find(|character: char| !is_identifier_character(character))
+                            .unwrap_or(after_the_keyword.len());
+                        &after_the_keyword[..name_length]
+                    });
+                Some(PlaceTakingTheStopRequest {
+                    layer,
+                    parameter,
+                    end_of_the_type,
+                })
+            })
+            .collect()
     }
 
     /// 没有行为用例的那几层，扫源码钉住它们怎么用收到的叫停（`every_layer_handed_the_stop_request_hands_it_on_or_asks_it`）。
@@ -4855,13 +4945,31 @@ mod tests {
         },
     }
 
-    /// `lines` 每一行起头处在第几层花括号里（第一行起头算第 0 层）：字符串、字符字面量与行注释里的花括号不算，字符串跨行时照跨着数。
+    /// 数花括号时，读到的这个字在源码的哪一种地方（跨行时带到下一行）。
+    #[derive(Clone, Copy)]
+    enum PlaceInTheSource {
+        Code,
+        InsideAString,
+        /// 原始字符串（`r"…"`、`r#"…"#`、`br"…"`、`cr"…"`）：收尾是一个 `"` 后面跟这么多个 `#`。
+        InsideARawString {
+            closing_hashes: usize,
+        },
+        /// 块注释，可以嵌套：`nesting` 是还没收尾的 `/*` 有几层。
+        InsideABlockComment {
+            nesting: usize,
+        },
+    }
+
+    /// `lines` 每一行起头处在第几层花括号里（第一行起头算第 0 层）：字符串（普通的、原始的）、字符字面量、行注释与块注释（可以嵌套）里的
+    /// 花括号不算，字符串、块注释跨行时照跨着数。
     ///
     /// # Panics
-    /// 右花括号多于左花括号；碰到原始字符串（`r"…"`、`r#"…"#`，这里不认它，层数会数错）。
-    fn brace_depth_at_the_start_of_each_line(lines: &[&String]) -> Vec<usize> {
+    /// 右花括号多于左花括号。
+    fn brace_depth_at_the_start_of_each_line(lines: &[&str]) -> Vec<usize> {
+        let is_identifier_character =
+            |character: char| character.is_alphanumeric() || character == '_';
         let mut depth = 0_usize;
-        let mut is_inside_a_string = false;
+        let mut place = PlaceInTheSource::Code;
         let mut depths = Vec::with_capacity(lines.len());
         for line in lines {
             depths.push(depth);
@@ -4869,36 +4977,88 @@ mod tests {
             let mut index = 0;
             while index < characters.len() {
                 let character = characters[index];
-                if is_inside_a_string {
-                    if character == '\\' {
-                        index += 1;
-                    } else if character == '"' {
-                        is_inside_a_string = false;
+                let next_character = characters.get(index + 1).copied();
+                match place {
+                    PlaceInTheSource::InsideAString => {
+                        if character == '\\' {
+                            index += 1;
+                        } else if character == '"' {
+                            place = PlaceInTheSource::Code;
+                        }
                     }
-                } else if character == '"' {
-                    let previous = index.checked_sub(1).map(|before| characters[before]);
-                    assert!(
-                        !(previous == Some('r') || previous == Some('#')),
-                        "这一层里有原始字符串，花括号的层数要先认它再数：{line}"
-                    );
-                    is_inside_a_string = true;
-                } else if character == '\'' {
-                    // 字符字面量（`'x'`、`'\''`、`'\u{7b}'`）整个跳过；生命周期（`'static`）只跳这个引号
-                    if characters.get(index + 1) == Some(&'\\') {
-                        index += 2;
-                        while characters.get(index + 1).is_some_and(|next| *next != '\'') {
+                    PlaceInTheSource::InsideARawString { closing_hashes } => {
+                        let closing_hashes_follow = characters
+                            .get(index + 1..index + 1 + closing_hashes)
+                            .is_some_and(|after_the_quote| {
+                                after_the_quote.iter().all(|after| *after == '#')
+                            });
+                        if character == '"' && closing_hashes_follow {
+                            index += closing_hashes;
+                            place = PlaceInTheSource::Code;
+                        }
+                    }
+                    PlaceInTheSource::InsideABlockComment { nesting } => {
+                        if character == '/' && next_character == Some('*') {
+                            index += 1;
+                            place = PlaceInTheSource::InsideABlockComment {
+                                nesting: nesting + 1,
+                            };
+                        } else if character == '*' && next_character == Some('/') {
+                            index += 1;
+                            place = match nesting.checked_sub(1) {
+                                Some(0) | None => PlaceInTheSource::Code,
+                                Some(nesting_left) => PlaceInTheSource::InsideABlockComment {
+                                    nesting: nesting_left,
+                                },
+                            };
+                        }
+                    }
+                    PlaceInTheSource::Code => {
+                        if character == '"' {
+                            place = PlaceInTheSource::InsideAString;
+                        } else if character == '\'' {
+                            // 字符字面量（`'x'`、`'\''`、`'\u{7b}'`）整个跳过；生命周期（`'static`）只跳这个引号
+                            if next_character == Some('\\') {
+                                index += 2;
+                                while characters.get(index + 1).is_some_and(|next| *next != '\'') {
+                                    index += 1;
+                                }
+                                index += 1;
+                            } else if characters.get(index + 2) == Some(&'\'') {
+                                index += 2;
+                            }
+                        } else if character == '/' && next_character == Some('/') {
+                            break;
+                        } else if character == '/' && next_character == Some('*') {
                             index += 1;
+                            place = PlaceInTheSource::InsideABlockComment { nesting: 1 };
+                        } else if is_identifier_character(character) {
+                            // 一整个词跳过；词是 r、br、cr 而后面紧跟若干个 # 与一个 " 的，是原始字符串的开头
+                            let word_end = characters[index..]
+                                .iter()
+                                .position(|after| !is_identifier_character(*after))
+                                .map_or(characters.len(), |word_length| index + word_length);
+                            let word: String = characters[index..word_end].iter().collect();
+                            let opening_hashes = characters[word_end..]
+                                .iter()
+                                .take_while(|after| **after == '#')
+                                .count();
+                            if matches!(word.as_str(), "r" | "br" | "cr")
+                                && characters.get(word_end + opening_hashes) == Some(&'"')
+                            {
+                                index = word_end + opening_hashes;
+                                place = PlaceInTheSource::InsideARawString {
+                                    closing_hashes: opening_hashes,
+                                };
+                            } else {
+                                index = word_end - 1;
+                            }
+                        } else if character == '\u{7b}' {
+                            depth += 1;
+                        } else if character == '\u{7d}' {
+                            depth = depth.checked_sub(1).expect("右花括号不多于左花括号");
                         }
-                        index += 1;
-                    } else if characters.get(index + 2) == Some(&'\'') {
-                        index += 2;
                     }
-                } else if character == '/' && characters.get(index + 1) == Some(&'/') {
-                    break;
-                } else if character == '\u{7b}' {
-                    depth += 1;
-                } else if character == '\u{7d}' {
-                    depth = depth.checked_sub(1).expect("右花括号不多于左花括号");
                 }
                 index += 1;
             }
@@ -4980,21 +5140,44 @@ mod tests {
     /// `judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from` 照旧整行比那一行，另要它是函数体顶层的一句
     /// （不在任何块里：`if false { … }`、`if false && … { … }` 包着的红）、`stop_request` 在这一层只由它 `let` 绑一次
     /// （后面再 `let` 一次就换掉了记住的叫停）、`StopRequestRememberedInThisRun` 在这一层只出现在它里面（块里 `use` 或另起同名类型会遮住它）。
+    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K4，正推 K4-3：名单外三种写法拦不住）：
+    /// 收叫停的层改成按叫停类型找（`&(dyn Fn() -> bool + Sync)`，空白随意、可以跨行），名单记层名连同参数名——参数与别的参数挤在签名那一行
+    /// （`the_stop_request_scan_finds_a_new_layer_whose_stop_parameter_shares_the_signature_line`）、参数换了名字
+    /// （`the_stop_request_scan_finds_a_new_layer_whose_stop_parameter_is_renamed`）的新层都在名单外、红；没编进 `gpu` 特性时那一份
+    /// `judge_my_blocks_on_gpu`（参数 `_stop_is_requested`）因此也进了名单，整句钉成只交错；长的那一层数 `stop_request` 被 `let` 绑了几次
+    /// 改成看每个 `let`（`if let`、`while let` 也算）到 `=` 或 `;` 之前那一截里有没有整个名字 `stop_request`——解构
+    /// `let (stop_request, …) = …`、`let` 后面多几个空格都数得到
+    /// （`the_stop_request_scan_finds_the_remembered_stop_request_rebound_by_a_tuple_pattern_or_a_let_with_extra_spaces`）；
+    /// 数花括号的层数认得原始字符串与块注释（`brace_depth_reads_past_raw_strings_block_comments_and_strings_spanning_lines`），
+    /// 长的那一层里写原始字符串不再让这条用例当场红，`if false { /* } */` 也骗不过层数。
     ///
     /// 拦不住的写法，一种一条：
     /// - 同一行原样交下去而下一层的函数体里不用（下一层由它自己那一条管；第 ① 段在卡上判那一层里 `stop_request` 往下怎么用由
     ///   `the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run` 管，也只看文本）；
-    /// - 长的那一层：钉住那一行之前提早返回，或让它之后的路走不到；`stop_request` 经 `let` 之外的绑定（`match` 的臂、闭包参数、`for`、
-    ///   `if let`）在块里换成别的；
-    /// - 被调的那一层自己改了（短的三层整句比钉的是调谁、怎么交，不是被调的函数里做什么）；harness 档的 `StopRequestRememberedInThisRun::new` 改了；
-    /// - 在函数上面加 `#[cfg]` 让另一份同名函数编进来（另一份的叫停参数写成 `_stop_is_requested` 就不进下面六层的名单）；
+    /// - 长的那一层：钉住那一行之前提早返回，或让它之后的路走不到；`stop_request` 经 `let` 之外的绑定（`match` 的臂、闭包参数、`for`）
+    ///   在块里换成别的；
+    /// - 被调的那一层自己改了（短的几层整句比钉的是调谁、怎么交，不是被调的函数里做什么）；harness 档的 `StopRequestRememberedInThisRun::new` 改了；
+    /// - 叫停换成别的类型写法（`&(dyn Sync + Fn() -> bool)`、`&dyn Fn() -> bool`、`impl Fn() -> bool`、泛型参数、类型别名）的新层，
+    ///   或在函数上面加 `#[cfg]` 让这样另写类型的同名函数编进来：按类型找只认这一种写法；
     /// - 长的那一层里宏展开出来的调用与绑定。
     #[test]
     fn every_layer_handed_the_stop_request_hands_it_on_or_asks_it() {
-        let stop_parameter = concat!("stop_is_", "requested: &(dyn Fn() -> bool + Sync),");
-        let pinned_layers_without_a_behaviour_test: [(&str, StopRequestUseThatIsPinned); 4] = [
+        assert_eq!(
+            ways_a_layer_handed_the_stop_request_could_drop_it(include_str!(
+                "crash_amplification.rs"
+            )),
+            Vec::<String>::new(),
+            "收叫停的每一层照钉住的那样往下交或问"
+        );
+    }
+
+    /// `source`（这份源文件，或用例在它上面改过的一份）测试模块之外，收叫停的层没照钉住的那样往下交或问的地方，一处一句；一处没有交空。
+    /// 判法与名单在 `every_layer_handed_the_stop_request_hands_it_on_or_asks_it`。
+    fn ways_a_layer_handed_the_stop_request_could_drop_it(source: &str) -> Vec<String> {
+        let pinned_layers_without_a_behaviour_test: [(&str, &str, StopRequestUseThatIsPinned); 5] = [
             (
                 "judge_my_blocks_on_gpu_cards",
+                STOP_REQUEST_PARAMETER_NAME,
                 StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
                     ")->Result<(CheckingTally,GpuJudgeOutcome),String>\u{7b}",
                     "matchjudge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from(",
@@ -5007,6 +5190,7 @@ mod tests {
             ),
             (
                 "judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from",
+                STOP_REQUEST_PARAMETER_NAME,
                 StopRequestUseThatIsPinned::TopLevelStatementOfALongBody {
                     line: concat!(
                         "let stop_request = StopRequestRememberedInThisRun::new(stop_is_",
@@ -5018,6 +5202,7 @@ mod tests {
             ),
             (
                 "check_selected_blocks",
+                STOP_REQUEST_PARAMETER_NAME,
                 StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
                     ")->Result<CheckingTally,VerdictStoreError>\u{7b}",
                     "check_blocks_where_until(store,flow,plan,",
@@ -5028,6 +5213,7 @@ mod tests {
             ),
             (
                 "judge_my_blocks_on_gpu",
+                STOP_REQUEST_PARAMETER_NAME,
                 StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
                     ")->Result<GpuJudgingOfMyBlocks,String>\u{7b}",
                     "judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from(",
@@ -5035,35 +5221,36 @@ mod tests {
                     "requested,)"
                 )),
             ),
+            (
+                "judge_my_blocks_on_gpu",
+                concat!("_stop_is_", "requested"),
+                StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
+                    ")->Result<GpuJudgingOfMyBlocks,String>\u{7b}",
+                    "Err(\"配置写了ENABLE_GPU=1，这份二进制没有编进gpu特性（cargo…--featuresverdict-store,gpu）\".to_string())"
+                )),
+            ),
         ];
-        let lines = code_lines_outside_the_tests();
+        let code = code_lines_outside_the_tests_of(source).join("\n") + "\n";
+        let mut found = Vec::new();
         let mut layers_handed_the_stop_request = Vec::new();
-        for (parameter_line_index, _parameter_line) in lines
-            .iter()
-            .enumerate()
-            .filter(|(_line_index, line)| line.trim() == stop_parameter)
-        {
-            let signature = lines[..parameter_line_index]
-                .iter()
-                .rev()
-                .find(|line| line.contains("fn "))
-                .expect("参数行之前有函数签名");
-            let layer = signature
-                .split("fn ")
-                .nth(1)
-                .and_then(|after_fn| after_fn.split(['(', '<']).next())
-                .expect("签名里 fn 之后是函数名")
-                .to_string();
-            let rest_of_the_layer: Vec<&String> = lines[parameter_line_index + 1..]
-                .iter()
-                .take_while(|line| line.as_str() != "\u{7d}")
+        for place in places_taking_the_stop_request(&code) {
+            let PlaceTakingTheStopRequest {
+                layer,
+                parameter,
+                end_of_the_type,
+            } = place;
+            let after_the_type = &code[end_of_the_type..];
+            let rest_of_the_layer: Vec<&str> = after_the_type[..after_the_type
+                .find(TOP_LEVEL_CLOSING_BRACE_LINE)
+                .unwrap_or(after_the_type.len())]
+                .lines()
                 .collect();
             match pinned_layers_without_a_behaviour_test
                 .iter()
-                .find(|(layer_without_a_behaviour_test, _pinned)| {
-                    *layer_without_a_behaviour_test == layer.as_str()
+                .find(|(pinned_layer, pinned_parameter, _pinned)| {
+                    *pinned_layer == layer && *pinned_parameter == parameter
                 })
-                .map(|(_layer, pinned)| pinned)
+                .map(|(_layer, _parameter, pinned)| pinned)
             {
                 Some(StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(expected_body)) => {
                     let body_without_spaces: String = rest_of_the_layer
@@ -5071,10 +5258,15 @@ mod tests {
                         .flat_map(|line| line.chars())
                         .filter(|character| !character.is_whitespace())
                         .collect();
-                    assert_eq!(
-                        body_without_spaces, *expected_body,
-                        "{layer} 参数行之后整句等于钉住的那一句（死分支里留一份原样的调用、真调用换实参，都多出或换掉一截）：{rest_of_the_layer:#?}"
-                    );
+                    // 参数后面的逗号（参数单独占一行时有）不算函数体
+                    let body_after_the_parameter = body_without_spaces
+                        .strip_prefix(',')
+                        .unwrap_or(&body_without_spaces);
+                    if body_after_the_parameter != *expected_body {
+                        found.push(format!(
+                            "{layer} 参数之后整句等于钉住的那一句（死分支里留一份原样的调用、真调用换实参，都多出或换掉一截）：{rest_of_the_layer:#?}"
+                        ));
+                    }
                 }
                 Some(StopRequestUseThatIsPinned::TopLevelStatementOfALongBody {
                     line: pinned_line,
@@ -5087,61 +5279,181 @@ mod tests {
                         .filter(|(_offset, line)| whole_name_uses_of_the_stop_request(line) > 0)
                         .map(|(offset, line)| (offset, line.trim()))
                         .collect();
-                    assert_eq!(
-                        lines_using_it
-                            .iter()
-                            .map(|(_offset, line)| *line)
-                            .collect::<Vec<&str>>(),
-                        vec![*pinned_line],
-                        "{layer} 用到叫停的只有钉住的那一行、原样（只提一次名字而永不叫停的写法多一行或那一行变了）：{rest_of_the_layer:#?}"
-                    );
-                    let depths = brace_depth_at_the_start_of_each_line(&rest_of_the_layer);
-                    let (pinned_offset, _pinned_line) = lines_using_it[0];
-                    assert_eq!(
-                        depths[pinned_offset], 1,
-                        "{layer} 的 {pinned_line} 是函数体顶层的一句，不在 `if false` 一类的块里（参数行之后那一行开函数体，算第 1 层）"
-                    );
-                    let let_bindings = rest_of_the_layer
+                    if lines_using_it
                         .iter()
-                        .filter(|line| {
-                            let statement = line.trim_start();
-                            [
-                                format!("let {binding} "),
-                                format!("let {binding}:"),
-                                format!("let mut {binding}"),
-                            ]
-                            .iter()
-                            .any(|binding_opening| statement.starts_with(binding_opening.as_str()))
+                        .map(|(_offset, line)| *line)
+                        .collect::<Vec<&str>>()
+                        != [*pinned_line]
+                    {
+                        found.push(format!(
+                            "{layer} 用到叫停的只有钉住的那一行、原样（只提一次名字而永不叫停的写法多一行或那一行变了）：{rest_of_the_layer:#?}"
+                        ));
+                    }
+                    let depths = brace_depth_at_the_start_of_each_line(&rest_of_the_layer);
+                    for (pinned_offset, _line) in &lines_using_it {
+                        if depths[*pinned_offset] != 1 {
+                            found.push(format!(
+                                "{layer} 的 {pinned_line} 是函数体顶层的一句，不在 `if false` 一类的块里（参数之后那一行开函数体，算第 1 层），读到的是第 {} 层",
+                                depths[*pinned_offset]
+                            ));
+                        }
+                    }
+                    let rest_of_the_layer_text = rest_of_the_layer.join("\n");
+                    let let_bindings = whole_word_positions(&rest_of_the_layer_text, "let")
+                        .into_iter()
+                        .filter(|let_keyword| {
+                            let after_the_keyword = &rest_of_the_layer_text[let_keyword + 3..];
+                            let pattern_end = after_the_keyword
+                                .find(['=', ';'])
+                                .unwrap_or(after_the_keyword.len());
+                            !whole_word_positions(&after_the_keyword[..pattern_end], binding)
+                                .is_empty()
                         })
                         .count();
-                    assert_eq!(
-                        let_bindings, 1,
-                        "{layer} 里 {binding} 只由钉住的那一行 let 绑一次（后面再 let 一次就换掉了记住的叫停）"
-                    );
+                    if let_bindings != 1 {
+                        found.push(format!(
+                            "{layer} 里 {binding} 只由钉住的那一行 let 绑一次（后面再 let 一次、解构、if let 就换掉了记住的叫停），读到 {let_bindings} 次"
+                        ));
+                    }
                     let lines_naming_the_type = rest_of_the_layer
                         .iter()
                         .filter(|line| line.contains(type_name))
                         .count();
-                    assert_eq!(
-                        lines_naming_the_type, 1,
-                        "{layer} 里只有钉住的那一行提到 {type_name}（块里 use 或另起同名的类型会遮住它）"
-                    );
+                    if lines_naming_the_type != 1 {
+                        found.push(format!(
+                            "{layer} 里只有钉住的那一行提到 {type_name}（块里 use 或另起同名的类型会遮住它），读到 {lines_naming_the_type} 行"
+                        ));
+                    }
                 }
                 None => {}
             }
-            layers_handed_the_stop_request.push(layer);
+            layers_handed_the_stop_request.push((layer, parameter));
         }
-        assert_eq!(
-            layers_handed_the_stop_request,
-            vec![
-                "check_blocks_where_until",
+        let expected_layers = [
+            ("check_blocks_where_until", STOP_REQUEST_PARAMETER_NAME),
+            (
                 "check_pending_blocks_in_pieces",
-                "judge_my_blocks_on_gpu_cards",
+                STOP_REQUEST_PARAMETER_NAME,
+            ),
+            ("judge_my_blocks_on_gpu_cards", STOP_REQUEST_PARAMETER_NAME),
+            (
                 "judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from",
-                "check_selected_blocks",
-                "judge_my_blocks_on_gpu",
-            ],
-            "收叫停的是这六层（多一层少一层都要回来看这条用例罩没罩到）"
+                STOP_REQUEST_PARAMETER_NAME,
+            ),
+            ("check_selected_blocks", STOP_REQUEST_PARAMETER_NAME),
+            ("judge_my_blocks_on_gpu", STOP_REQUEST_PARAMETER_NAME),
+            ("judge_my_blocks_on_gpu", concat!("_stop_is_", "requested")),
+        ];
+        if layers_handed_the_stop_request != expected_layers {
+            found.push(format!(
+                "收叫停的是这六层，连同没编进 gpu 特性时那一份 judge_my_blocks_on_gpu（多一层少一层、参数换了名字都要回来看这条用例罩没罩到），读到的是 {layers_handed_the_stop_request:?}"
+            ));
+        }
+        found
+    }
+
+    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K4，正推 K4-3 第 1 种 `bypass_extra_layer_one_line`）：新加一层收叫停、
+    /// 叫停参数与别的参数挤在签名那一行里，照样找得到这一层，它不在名单里就红。扫这份源码改过的一份，不要卡。
+    #[test]
+    fn the_stop_request_scan_finds_a_new_layer_whose_stop_parameter_shares_the_signature_line() {
+        let new_layer = concat!(
+            "fn an_extra_layer_never_asking(first_input: u8, stop_is_",
+            "requested: &(dyn Fn() -> bool + Sync)) -> bool \u{7b} first_input == 0 \u{7d}"
+        );
+        let source_with_the_new_layer =
+            format!("{}\n{new_layer}\n", include_str!("crash_amplification.rs"));
+        let found = ways_a_layer_handed_the_stop_request_could_drop_it(&source_with_the_new_layer);
+        assert!(
+            found
+                .iter()
+                .any(|finding| finding.contains("(\"an_extra_layer_never_asking\", ")),
+            "签名一行里收叫停的新层进了读到的名单、与钉住的名单对不上：{found:#?}"
+        );
+    }
+
+    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K4，正推 K4-3 第 2 种 `bypass_extra_layer_other_name`）：新加一层收叫停、
+    /// 叫停参数换了名字，照样找得到这一层，它不在名单里就红。扫这份源码改过的一份，不要卡。
+    #[test]
+    fn the_stop_request_scan_finds_a_new_layer_whose_stop_parameter_is_renamed() {
+        let new_layer = concat!(
+            "fn an_extra_layer_with_a_renamed_stop(\n",
+            "    first_input: u8,\n",
+            "    stop: &(dyn Fn() -> bool + Sync),\n",
+            ") -> bool \u{7b}\n",
+            "    first_input == 0\n",
+            "\u{7d}"
+        );
+        let source_with_the_new_layer =
+            format!("{}\n{new_layer}\n", include_str!("crash_amplification.rs"));
+        let found = ways_a_layer_handed_the_stop_request_could_drop_it(&source_with_the_new_layer);
+        assert!(
+            found
+                .iter()
+                .any(|finding| finding
+                    .contains("(\"an_extra_layer_with_a_renamed_stop\", \"stop\")")),
+            "参数换了名字的新层进了读到的名单、与钉住的名单对不上：{found:#?}"
+        );
+    }
+
+    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K4，正推 K4-3 第 3 种 `bypass_tuple_let_rebind`、`bypass_let_extra_spaces`）：
+    /// 长的那一层钉住那一行之后解构重绑 `stop_request`、或 `let` 与名字之间多几个空格重绑，都数成又 `let` 了一次。扫这份源码改过的一份，不要卡。
+    #[test]
+    fn the_stop_request_scan_finds_the_remembered_stop_request_rebound_by_a_tuple_pattern_or_a_let_with_extra_spaces(
+    ) {
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
+        for (form, rebinding) in [
+            (
+                "解构",
+                "    let (stop_request, _unused) = (the_never_stopping_request(), 0);\n",
+            ),
+            (
+                "多几个空格",
+                "    let   stop_request  =  the_never_stopping_request();\n",
+            ),
+        ] {
+            let rebound = source.replacen(pinned_line, &format!("{pinned_line}{rebinding}"), 1);
+            let found = ways_a_layer_handed_the_stop_request_could_drop_it(&rebound);
+            assert!(
+                found.iter().any(|finding| finding.contains(
+                    "里 stop_request 只由钉住的那一行 let 绑一次（后面再 let 一次、解构、if let 就换掉了记住的叫停），读到 2 次"
+                )),
+                "{form}重绑数成第二次 let：{found:#?}"
+            );
+        }
+    }
+
+    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮正推 K5：数花括号遇到原始字符串当场 `assert!`）：原始字符串（里面的 `"`、
+    /// 收尾的反斜杠、跨行）、块注释（嵌套、里面的右花括号）、跨行的普通字符串里的花括号都不算层数。不要卡。
+    #[test]
+    fn brace_depth_reads_past_raw_strings_block_comments_and_strings_spanning_lines() {
+        let lines = [
+            "let path = r\"C:\\\";",
+            "if path.is_empty() \u{7b}",
+            "    let quoted = r#\"say \"\u{7b}\" here\"#;",
+            "    let bytes = br##\"\u{7d}\"#\u{7d}\"##;",
+            "    let spanning = r#\"",
+            "\u{7b}\u{7b}",
+            "\"#;",
+            "    /* a \u{7d} in a comment */",
+            "    let nested = /* outer /* inner \u{7d} */ still \u{7d} */ 0;",
+            "    let plain = \"",
+            "\u{7b}\";",
+            "\u{7d}",
+            "after",
+        ];
+        assert_eq!(
+            brace_depth_at_the_start_of_each_line(&lines),
+            vec![0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
+            "只有 if 那一行的左花括号与倒数第二行的右花括号算层数：{lines:#?}"
         );
     }
 
````

### 第二十一块变异表追加（13 行）

````tsv
第10z批第二十一块 GPU 取卡 K5 卡池扫源码把 != 拆成 ! 与 =（条目层的不等号又被认成调宏，正推 K5_const_ne）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	            } else if rest.starts_with("!=") {	            } else if rest.starts_with("!=") && false {	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scan_reads_an_item_level_not_equal_as_a_comparison_and_still_finds_an_item_level_macro	the_card_pool_scan_reads_an_item_level_not_equal_as_a_comparison_and_still_finds_an_item_level_macro
第10z批第二十一块 GPU 取卡 K5 卡池扫源码 use 里改回不许 as、self（改成别的名字、{self, …} 又判红，正推 K5_use_as、K5_pub_use_self_group）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	        for statement in &use_statements {\n            let (brought_in_by_the_statement, end_of_the_tree) =	        for statement in &use_statements {\n            if statement.iter().any(|token| matches!(*token, "as" | "self")) {\n                found.push(format!("变异：use 不许改名、不许引 self：{statement:?}"));\n            }\n            let (brought_in_by_the_statement, end_of_the_tree) =	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scan_accepts_renames_nested_groups_and_self_that_bring_in_neither_std_nor_wgpu	the_card_pool_scan_accepts_renames_nested_groups_and_self_that_bring_in_neither_std_nor_wgpu
第10z批第二十一块 GPU 取卡 K5 卡池扫源码 use 里改回至多一层花括号（两层花括号又判红，正推 K5_nested_use_braces）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	            brought_in.extend(brought_in_by_the_statement);\n        }	            if statement.iter().filter(|token| **token == "\u{7b}").count() > 1 {\n                found.push(format!("变异：use 至多一层花括号：{statement:?}"));\n            }\n            brought_in.extend(brought_in_by_the_statement);\n        }	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scan_accepts_renames_nested_groups_and_self_that_bring_in_neither_std_nor_wgpu	the_card_pool_scan_accepts_renames_nested_groups_and_self_that_bring_in_neither_std_nor_wgpu
第10z批第二十一块 GPU 取卡 K5 卡池扫源码改回见 extern 就拒（extern "C" 块又判红，正推 K5_extern_c_block）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	            if pair == ["extern", "crate"] {	            if pair[0] == "extern" {	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scan_accepts_an_extern_block_and_still_finds_extern_crate	the_card_pool_scan_accepts_an_extern_block_and_still_finds_extern_crate
第10z批第二十一块 GPU 取卡 K5 卡池扫源码 use 改名时取原名、不取改成的名字（use … as std 漏过）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                    (statement[position + 1].to_string(), position + 2)	                    (name_without_a_rename, position + 2)	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scan_finds_std_wgpu_or_a_pinned_function_brought_in_under_a_rename_a_nested_group_or_self	the_card_pool_scan_finds_std_wgpu_or_a_pinned_function_brought_in_under_a_rename_a_nested_group_or_self
第10z批第二十一块 GPU 取卡 K5 卡池扫源码花括号里的 self 取名成 self（use …::wgpu::{self} 漏过）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                let name_without_a_rename = if is_self {	                let name_without_a_rename = if false && is_self {	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scan_finds_std_wgpu_or_a_pinned_function_brought_in_under_a_rename_a_nested_group_or_self	the_card_pool_scan_finds_std_wgpu_or_a_pinned_function_brought_in_under_a_rename_a_nested_group_or_self
第10z批第二十一块 GPU 取卡 K5 卡池扫源码不再拒通配（故意拒的那一类放过，正推 K5_use_wildcard_enum）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                BroughtInByAUse::Glob { source } => found.push(format!(	                BroughtInByAUse::Glob { source } => drop(format!(	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scan_refuses_globs_item_level_macros_and_inner_modules_even_when_they_name_nothing_pinned	the_card_pool_scan_refuses_globs_item_level_macros_and_inner_modules_even_when_they_name_nothing_pinned
第10z批第二十一块 崩溃放量 K4 叫停扫源码只认参数单独占一行的层（签名一行里收叫停的新层漏过，正推 K4-3 第 1 种）	crates/singlefs-checker-tier/src/crash_amplification.rs	                Some(PlaceTakingTheStopRequest {	                let line_before_the_type = code[..ampersand].lines().last().unwrap_or("");\n                if !line_before_the_type.trim_start().starts_with(parameter) {\n                    return None;\n                }\n                Some(PlaceTakingTheStopRequest {	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_scan_finds_a_new_layer_whose_stop_parameter_shares_the_signature_line	the_stop_request_scan_finds_a_new_layer_whose_stop_parameter_shares_the_signature_line
第10z批第二十一块 崩溃放量 K4 叫停扫源码只认叫 stop_is_requested 的参数（参数换名的新层漏过，正推 K4-3 第 2 种）	crates/singlefs-checker-tier/src/crash_amplification.rs	                Some(PlaceTakingTheStopRequest {	                if !parameter.ends_with(STOP_REQUEST_PARAMETER_NAME) {\n                    return None;\n                }\n                Some(PlaceTakingTheStopRequest {	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_scan_finds_a_new_layer_whose_stop_parameter_is_renamed	the_stop_request_scan_finds_a_new_layer_whose_stop_parameter_is_renamed
第10z批第二十一块 崩溃放量 K4 叫停扫源码改回只数 let stop_request 起头的写法（解构、多空格重绑漏过，正推 K4-3 第 3 种）	crates/singlefs-checker-tier/src/crash_amplification.rs	                            !whole_word_positions(&after_the_keyword[..pattern_end], binding)\n                                .is_empty()	                            after_the_keyword.starts_with(&format!(" {binding} "))\n                                || after_the_keyword.starts_with(&format!(" {binding}:"))\n                                || after_the_keyword.starts_with(&format!(" mut {binding}"))	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_scan_finds_the_remembered_stop_request_rebound_by_a_tuple_pattern_or_a_let_with_extra_spaces	the_stop_request_scan_finds_the_remembered_stop_request_rebound_by_a_tuple_pattern_or_a_let_with_extra_spaces
第10z批第二十一块 崩溃放量 K5 数花括号不认原始字符串（原始字符串里的引号与花括号算错层数）	crates/singlefs-checker-tier/src/crash_amplification.rs	                            if matches!(word.as_str(), "r" | "br" | "cr")	                            if false && matches!(word.as_str(), "r" | "br" | "cr")	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::brace_depth_reads_past_raw_strings_block_comments_and_strings_spanning_lines	brace_depth_reads_past_raw_strings_block_comments_and_strings_spanning_lines
第10z批第二十一块 崩溃放量 K4 数花括号不认块注释（if false { /* } */ 骗过层数）	crates/singlefs-checker-tier/src/crash_amplification.rs	                            place = PlaceInTheSource::InsideABlockComment { nesting: 1 };	                            index -= 1;	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::brace_depth_reads_past_raw_strings_block_comments_and_strings_spanning_lines	brace_depth_reads_past_raw_strings_block_comments_and_strings_spanning_lines
第10z批第二十一块 崩溃放量 K4 没编进 gpu 特性那一份 judge_my_blocks_on_gpu 不交错、当判过了（不问叫停）	crates/singlefs-checker-tier/src/crash_amplification.rs	    _stop_is_requested: &(dyn Fn() -> bool + Sync),\n) -> Result<GpuJudgingOfMyBlocks, String> {\n    Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())\n}	    _stop_is_requested: &(dyn Fn() -> bool + Sync),\n) -> Result<GpuJudgingOfMyBlocks, String> {\n    // 变异：没编进 gpu 特性这一份也当判过了、不问叫停\n    Ok(todo!())\n}	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::every_layer_handed_the_stop_request_hands_it_on_or_asks_it	every_layer_handed_the_stop_request_hands_it_on_or_asks_it
````

## 主 agent 合入时改的一行（`crates/mutations.tsv` 第 3553 行，合后原样）

````tsv
第10z批第二十一块 GPU 取卡 K5 卡池扫源码 use 里改回至多一层花括号（两层花括号又判红，正推 K5_nested_use_braces）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	            brought_in.extend(brought_in_by_the_statement);\n        }	            if statement.iter().filter(|token| token.as_bytes() == [0x7b_u8]).count() > 1 {\n                found.push(format!("变异：use 至多一层花括号：{statement:?}"));\n            }\n            brought_in.extend(brought_in_by_the_statement);\n        }	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_scan_accepts_renames_nested_groups_and_self_that_bring_in_neither_std_nor_wgpu	the_card_pool_scan_accepts_renames_nested_groups_and_self_that_bring_in_neither_std_nor_wgpu
````
