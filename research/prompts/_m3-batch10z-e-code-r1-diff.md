# 根环实现第 10z 批第五块：代码轮第一轮改动全文（2026-10-08）

实现员交的 `crates.patch` 原样，打在删除线第 42、43 批合后的主工作区上（第四块先、第五块后）。变异表追加、替换附在后面。

````diff
diff --git a/crates/singlefs-harness/src/gpu_memory_ledger.rs b/crates/singlefs-harness/src/gpu_memory_ledger.rs
index 4105de0..72fb503 100644
--- a/crates/singlefs-harness/src/gpu_memory_ledger.rs
+++ b/crates/singlefs-harness/src/gpu_memory_ledger.rs
@@ -1560,7 +1560,32 @@ pub struct DeviceConditionOfACard {
     out_of_memory_reports: Mutex<Vec<OutOfMemoryReport>>,
     /// 此刻在这个设备上编内核的活，各有几处在编。
     works_compiling: Mutex<BTreeMap<WorkOnTheCardIdentifier, usize>>,
-    lost: Mutex<Option<String>>,
+    /// 设备丢了的话：记了就不抹（重起的是新设备、新的一份）。
+    lost: Mutex<Option<LossOfTheDevice>>,
+}
+
+/// 设备为什么丢的，按接 Out of Memory 的那一层要做的决定分（设备丢失的回调里认：wgpu 给的那一句是不是它的「显存不够」）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CauseOfTheDeviceLoss {
+    /// wgpu 因显存不够丢了设备（它把建管线、提交这类调用上的 Out of Memory 当致命）：因设备丢了而失败的活都说得成显存不够，
+    /// 不必等别的活把它范围里接到的那一句交回（第 10z 批代码轮判决 K1-4 攻方甲形：同一个设备上不在同一组的活）。
+    GpuMemoryShort,
+    /// 别的原因（驱动重置、wgpu 说设备丢了、驱动出了意外）。
+    Other,
+}
+
+/// 记下的一次设备丢失：wgpu 给的原因与那一句（`why`），与认出来的缘由。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct LossOfTheDevice {
+    why: String,
+    cause: CauseOfTheDeviceLoss,
+}
+
+/// 一段活交回那一刻（活做完或 panic 之后，等同一组的其余几段之前）设备丢没丢（[`DeviceConditionOfACard::device_when_the_work_returns`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum DeviceWhenTheWorkReturned {
+    Lost,
+    NotLost,
 }
 
 /// 编内核那一段活着时这个设备上记着「在给这段活编」，丢掉时（编完或 panic）抹掉。
@@ -1640,20 +1665,25 @@ impl DeviceConditionOfACard {
         compile()
     }
 
-    /// 记设备丢了（wgpu 的设备丢失回调调它）。
-    pub fn record_the_device_lost(&self, why: String) {
-        *self.lost.lock().unwrap_or_else(PoisonError::into_inner) = Some(why);
+    /// 记设备丢了（wgpu 的设备丢失回调调它）：`why` 是 wgpu 给的原因与那一句，`cause` 是回调认出来的缘由。
+    pub fn record_the_device_lost(&self, why: String, cause: CauseOfTheDeviceLoss) {
+        *self.lost.lock().unwrap_or_else(PoisonError::into_inner) =
+            Some(LossOfTheDevice { why, cause });
     }
 
-    /// 设备丢了的话为什么丢（没丢是 `None`）。
-    #[must_use]
-    pub fn lost_reason(&self) -> Option<String> {
+    fn loss_of_the_device(&self) -> Option<LossOfTheDevice> {
         self.lost
             .lock()
             .unwrap_or_else(PoisonError::into_inner)
             .clone()
     }
 
+    /// 设备丢了的话为什么丢（没丢是 `None`）。
+    #[must_use]
+    pub fn lost_reason(&self) -> Option<String> {
+        self.loss_of_the_device().map(|loss| loss.why)
+    }
+
     /// 设备丢了没有（卡池照它判留着的设备还能不能拿：丢了就重起）。
     #[must_use]
     pub fn is_lost(&self) -> bool {
@@ -1663,14 +1693,28 @@ impl DeviceConditionOfACard {
             .is_some()
     }
 
+    /// 一段活交回的那一刻（活做完或 panic 了、错误范围弹出之前）记下设备丢没丢：之后认它的失败时，设备丢了那一支只给这一刻设备已经丢了的活
+    /// （第 10z 批代码轮判决 K1-4 攻方乙形：活自己真 panic 的那一刻设备没丢，之后同组别的段撞上致命的 Out of Memory、设备丢了，
+    /// 等齐再认不把这段活说成显存不够）。
+    #[must_use]
+    pub fn device_when_the_work_returns(&self) -> DeviceWhenTheWorkReturned {
+        if self.is_lost() {
+            DeviceWhenTheWorkReturned::Lost
+        } else {
+            DeviceWhenTheWorkReturned::NotLost
+        }
+    }
+
     /// 一段活收完：`caught_in_the_scope`（这条线程上的错误范围接到的 Out of Memory，`None` 是没接到）记进设备，交 `ending` 报到、等同一个设备上
     /// 一起开的其余几段都报到，再认这段活从记到 `reports_before` 次那一刻起出的坏事能不能说成显存不够
-    /// （[`Self::out_of_memory_explaining_a_failure_of_the_work`]）。先等再认（第 10z 批代码轮判决 K1-3 攻方：一块的几段并发，一段因设备丢了先失败，
+    /// （[`Self::out_of_memory_explaining_a_failure_of_the_work`]，`device_when_the_work_returned` 是这段活交回那一刻
+    /// [`Self::device_when_the_work_returns`] 记的）。先记再报到、先等再认（第 10z 批代码轮判决 K1-3 攻方：一块的几段并发，一段因设备丢了先失败，
     /// 同块另一段范围里接到的 Out of Memory 要等那一段收完才记进设备；先认就认不出、判红）。
     pub fn end_the_work_and_explain_its_failure(
         &self,
         work: &WorkOnTheCardOfThisThread,
         reports_before: usize,
+        device_when_the_work_returned: DeviceWhenTheWorkReturned,
         caught_in_the_scope: Option<String>,
         ending: WorkEndingTogether<'_>,
     ) -> Option<String> {
@@ -1678,17 +1722,24 @@ impl DeviceConditionOfACard {
             self.record_an_out_of_memory(caught);
         }
         ending.end_and_wait_for_the_others();
-        self.out_of_memory_explaining_a_failure_of_the_work(work.identifier(), reports_before)
+        self.out_of_memory_explaining_a_failure_of_the_work(
+            work.identifier(),
+            reports_before,
+            device_when_the_work_returned,
+        )
     }
 
     /// 段活 `work` 从记到 `reports_before` 次那一刻起出了一样坏事、能不能说成是它的显存不够：这之后记了算在它上面的 Out of Memory，
-    /// 交最后那一句；设备丢了、这之后记过 Out of Memory（算在哪段活上都行：wgpu 把建管线、提交这类调用上的 Out of Memory 当致命、丢掉设备，
-    /// 丢了设备谁都做不成），交两样都说的一句；别的交 `None`（别的活上的 Out of Memory、这段活开始之前的，都说不成）。
+    /// 交最后那一句。别的只在这段活交回那一刻设备已经丢了（`device_when_the_work_returned`）时才说得成：设备因显存不够丢的
+    /// （[`CauseOfTheDeviceLoss::GpuMemoryShort`]，这段活之后记没记过都一样：同一个设备上不在同一组的活撞上的，范围里那一句要等那段活交回），
+    /// 交说它的一句；为别的丢的而这之后记过 Out of Memory（算在哪段活上都行：wgpu 把建管线、提交这类调用上的 Out of Memory 当致命、丢掉设备，
+    /// 丢了设备谁都做不成），交两样都说的一句；别的交 `None`（别的活上的 Out of Memory、这段活开始之前的、交回那一刻设备还没丢的，都说不成）。
     #[must_use]
     pub fn out_of_memory_explaining_a_failure_of_the_work(
         &self,
         work: WorkOnTheCardIdentifier,
         reports_before: usize,
+        device_when_the_work_returned: DeviceWhenTheWorkReturned,
     ) -> Option<String> {
         let reports = self
             .out_of_memory_reports
@@ -1702,11 +1753,25 @@ impl DeviceConditionOfACard {
         {
             return Some(report.what.clone());
         }
-        match (self.lost_reason(), reports_since.last()) {
-            (Some(lost), Some(report)) => {
-                Some(format!("设备丢了（{lost}），丢之前报过 {}", report.what))
+        match device_when_the_work_returned {
+            DeviceWhenTheWorkReturned::NotLost => None,
+            DeviceWhenTheWorkReturned::Lost => {
+                // 交回那一刻照 `device_when_the_work_returns` 记成丢了的，丢失一定还在（只记不抹）；没记过丢失就说不成
+                let loss = self.loss_of_the_device()?;
+                match (loss.cause, reports_since.last()) {
+                    (
+                        CauseOfTheDeviceLoss::GpuMemoryShort | CauseOfTheDeviceLoss::Other,
+                        Some(report),
+                    ) => Some(format!(
+                        "设备丢了（{}），丢之前报过 {}",
+                        loss.why, report.what
+                    )),
+                    (CauseOfTheDeviceLoss::GpuMemoryShort, None) => {
+                        Some(format!("设备因显存不够丢了（{}）", loss.why))
+                    }
+                    (CauseOfTheDeviceLoss::Other, None) => None,
+                }
             }
-            (None, Some(_) | None) | (Some(_), None) => None,
         }
     }
 }
@@ -1806,33 +1871,93 @@ impl Drop for WorkEndingTogether<'_> {
     }
 }
 
+/// 卡线程这一刻要不要停（每领一块之前问一次，[`judge_blocks_on_one_card`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum StopOfTheCards {
+    /// 照常领。
+    NotRequested,
+    /// 调用方叫停：不领新的块，只领判完的块在等的那几块（比判完的块号小、交回队里的），判完的块照块序都写得进库、库里不留空洞；
+    /// 没判的留给下一趟（第 10z 批代码轮判决 K1-5：收尾再起的那一轮里叫停，上一轮判完的块在等交回的那一块）。
+    RequestedByTheCaller,
+    /// 写库失败或有卡拒判（这一趟判红）：一块都不再领。
+    TheRunFailed,
+}
+
 /// 几张卡一起判的那一队块（崩溃放量产品路径第 ① 段，`singlefs_checker_tier::crash_amplification`；块号是要判的块在那一趟 `pending` 里的次序）：
 /// 每张卡领序号最小的、还没领走的那一块；一张卡报了 Out of Memory 把手里那一块交回，交回的序号比没领过的都小，下一张来领的卡先领到它
-/// （第 10z 批，第 10y 批代码轮判决 K2-2 P2：一张卡 Out of Memory 不停整趟）。不碰卡：单测在 harness 档随时跑（文件头）。
+/// （第 10z 批，第 10y 批代码轮判决 K2-2 P2：一张卡 Out of Memory 不停整趟）。记着判完的块里块号最大的（几轮接着算）：调用方叫停之后
+/// 只领比它小的（[`StopOfTheCards::RequestedByTheCaller`]）。不碰卡：单测在 harness 档随时跑（文件头）。
 pub struct BlocksForTheCards {
-    blocks_not_taken: Mutex<BTreeSet<usize>>,
+    queue: Mutex<QueueOfTheBlocks>,
+}
+
+struct QueueOfTheBlocks {
+    blocks_not_taken: BTreeSet<usize>,
+    /// 判完的块里块号最大的（[`judge_blocks_on_one_card`] 判完一块就记）：队里比它小的，判完的块在等。
+    largest_judged: Option<usize>,
+}
+
+impl QueueOfTheBlocks {
+    /// 队头那一块 `first` 此刻照 `stop` 领不领得。
+    fn may_take(&self, first: usize, stop: StopOfTheCards) -> bool {
+        match stop {
+            StopOfTheCards::NotRequested => true,
+            StopOfTheCards::RequestedByTheCaller => {
+                self.largest_judged.is_some_and(|largest| first < largest)
+            }
+            StopOfTheCards::TheRunFailed => false,
+        }
+    }
 }
 
 impl BlocksForTheCards {
-    /// 一队块：`pending_indices` 里的每一块（一轮只判上一轮剩下的那几块，[`after_a_round_of_the_cards`]）。
+    /// 一队块：`pending_indices` 里的每一块，还没有判完的。
     #[must_use]
     pub fn new(pending_indices: impl IntoIterator<Item = usize>) -> Self {
         Self {
-            blocks_not_taken: Mutex::new(pending_indices.into_iter().collect()),
+            queue: Mutex::new(QueueOfTheBlocks {
+                blocks_not_taken: pending_indices.into_iter().collect(),
+                largest_judged: None,
+            }),
         }
     }
 
-    fn blocks_not_taken_locked(&self) -> MutexGuard<'_, BTreeSet<usize>> {
+    fn queue_locked(&self) -> MutexGuard<'_, QueueOfTheBlocks> {
         // 持锁只为取、放一个序号，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
-        self.blocks_not_taken
-            .lock()
-            .unwrap_or_else(PoisonError::into_inner)
+        self.queue.lock().unwrap_or_else(PoisonError::into_inner)
+    }
+
+    /// 照 `stop` 领一块：没叫停领序号最小的、还没领走的（交回的也算）；调用方叫停了，那一块比判完的块号小（判完的块在等它）才领；
+    /// 判红了不领。领不得或都领走了是 `None`。
+    #[must_use]
+    pub fn take_a_block(&self, stop: StopOfTheCards) -> Option<usize> {
+        let mut queue = self.queue_locked();
+        let first = queue.blocks_not_taken.first().copied()?;
+        if queue.may_take(first, stop) {
+            queue.blocks_not_taken.pop_first()
+        } else {
+            None
+        }
     }
 
-    /// 领一块：序号最小的、还没领走的（交回的也算）；都领走了是 `None`。
+    /// 照 `stop` 此刻还有没有领得的块（起下一张卡之前问：没有就不起）。
     #[must_use]
-    pub fn take_a_block(&self) -> Option<usize> {
-        self.blocks_not_taken_locked().pop_first()
+    pub fn any_left_to_take(&self, stop: StopOfTheCards) -> bool {
+        let queue = self.queue_locked();
+        queue
+            .blocks_not_taken
+            .first()
+            .is_some_and(|first| queue.may_take(*first, stop))
+    }
+
+    /// 判完了一块（交去写库之前记）。
+    fn record_a_judged_block(&self, pending_index: usize) {
+        let mut queue = self.queue_locked();
+        queue.largest_judged = Some(
+            queue
+                .largest_judged
+                .map_or(pending_index, |largest| largest.max(pending_index)),
+        );
     }
 
     /// 领了没判完的一块交回队里（报了 Out of Memory 的卡交）。
@@ -1840,7 +1965,7 @@ impl BlocksForTheCards {
     /// # Panics
     /// 这一块此刻就在队里（交了两次，或没领过）。
     pub fn give_back(&self, pending_index: usize) {
-        let is_newly_given_back = self.blocks_not_taken_locked().insert(pending_index);
+        let is_newly_given_back = self.queue_locked().blocks_not_taken.insert(pending_index);
         assert!(
             is_newly_given_back,
             "块 {pending_index} 交回时已经在队里：一块领一次只交回一次"
@@ -1850,13 +1975,29 @@ impl BlocksForTheCards {
     /// 还有没领走的块没有（没领过的或交回的）。
     #[must_use]
     pub fn any_left(&self) -> bool {
-        !self.blocks_not_taken_locked().is_empty()
+        !self.queue_locked().blocks_not_taken.is_empty()
     }
 
     /// 此刻还没领走的块，按序号（几张卡都收工之后，就是这一轮没判的那几块）。
     #[must_use]
     pub fn blocks_not_taken(&self) -> Vec<usize> {
-        self.blocks_not_taken_locked().iter().copied().collect()
+        self.queue_locked()
+            .blocks_not_taken
+            .iter()
+            .copied()
+            .collect()
+    }
+
+    /// 一轮卡都收工之后，下一轮的那一队：这一轮没领走的，连同判完的块里块号最大的（下一轮叫停了照样领判完的块在等的那几块）。
+    #[must_use]
+    pub fn left_for_another_round(&self) -> Self {
+        let queue = self.queue_locked();
+        Self {
+            queue: Mutex::new(QueueOfTheBlocks {
+                blocks_not_taken: queue.blocks_not_taken.clone(),
+                largest_judged: queue.largest_judged,
+            }),
+        }
     }
 }
 
@@ -1885,7 +2026,7 @@ pub enum BlockHandedToTheStore {
 /// 一张卡怎么收的工（[`judge_blocks_on_one_card`]）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum CardLeftTheBlocks {
-    /// 队里没有要领的了，或被叫停（`must_stop`）。
+    /// 队里没有领得的了（领完了，或照叫停领不得）。
     NothingLeftOrStopped,
     /// 判一块时报了 Out of Memory：那一块已经交回队里。
     OutOfMemory { reason: String },
@@ -1895,24 +2036,27 @@ pub enum CardLeftTheBlocks {
     TheStoreIsGone,
 }
 
-/// 一张卡的线程判块的那一圈（崩溃放量产品路径第 ① 段的卡线程，判器建好之后）：每领一块之前问 `must_stop`；领一块（[`BlocksForTheCards::take_a_block`]），
-/// 交 `judge_a_block` 判；判完交 `hand_to_the_store`（写库按块序，[`BlocksStoredInOrder`]）；报了 Out of Memory 就把这一块交回队里、这张卡收工，
-/// 不再领（只停这一张卡，第 10z 批）；判不了就收工、这一块不交回。循环的次数不超过队里的块数加交回的次数（每次领走一块，每张卡至多交回一次）。
+/// 一张卡的线程判块的那一圈（崩溃放量产品路径第 ① 段的卡线程，判器建好之后）：每领一块之前问 `stop_of_the_cards`、照它领一块
+/// （[`BlocksForTheCards::take_a_block`]，叫停了只领判完的块在等的），交 `judge_a_block` 判；判完记下、交 `hand_to_the_store`
+/// （写库按块序，[`BlocksStoredInOrder`]）；报了 Out of Memory 就把这一块交回队里、这张卡收工，不再领（只停这一张卡，第 10z 批）；
+/// 判不了就收工、这一块不交回。循环的次数不超过队里的块数加交回的次数（每次领走一块，每张卡至多交回一次）。
 pub fn judge_blocks_on_one_card<Judged>(
     blocks: &BlocksForTheCards,
-    must_stop: &dyn Fn() -> bool,
+    stop_of_the_cards: &dyn Fn() -> StopOfTheCards,
     judge_a_block: &mut dyn FnMut(usize) -> BlockJudgedOnACard<Judged>,
     hand_to_the_store: &mut dyn FnMut(usize, Judged) -> BlockHandedToTheStore,
 ) -> CardLeftTheBlocks {
-    while !must_stop() {
-        let Some(pending_index) = blocks.take_a_block() else {
-            break;
-        };
+    while let Some(pending_index) = blocks.take_a_block(stop_of_the_cards()) {
         match judge_a_block(pending_index) {
-            BlockJudgedOnACard::Judged(judged) => match hand_to_the_store(pending_index, judged) {
-                BlockHandedToTheStore::Handed => {}
-                BlockHandedToTheStore::TheStoreIsGone => return CardLeftTheBlocks::TheStoreIsGone,
-            },
+            BlockJudgedOnACard::Judged(judged) => {
+                blocks.record_a_judged_block(pending_index);
+                match hand_to_the_store(pending_index, judged) {
+                    BlockHandedToTheStore::Handed => {}
+                    BlockHandedToTheStore::TheStoreIsGone => {
+                        return CardLeftTheBlocks::TheStoreIsGone
+                    }
+                }
+            }
             BlockJudgedOnACard::OutOfMemory { reason } => {
                 blocks.give_back(pending_index);
                 return CardLeftTheBlocks::OutOfMemory { reason };
@@ -1987,7 +2131,7 @@ impl<Block> BlocksStoredInOrder<Block> {
     }
 }
 
-/// 调用方叫没叫停（每领一块之前问的那一个）。
+/// 调用方叫没叫停（一轮卡都收工之后读的那一次，[`RoundsOfTheCards::after_the_round`]；卡线程每领一块之前问的是 [`StopOfTheCards`]）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum StopOfTheRun {
     Requested,
@@ -1999,21 +2143,27 @@ pub enum StopOfTheRun {
 pub enum AfterARoundOfTheCards {
     /// 要判的块都写进了库。
     EveryBlockStored,
-    /// 这一轮没有卡报 Out of Memory、还有块没写：被叫停，没判的留给下一趟。
+    /// 被叫停，判完的块都写进了库（没有在等的），没判的留给下一趟；这一轮有卡报了 Out of Memory 也一样（第 10z 批代码轮判决 K1-5：
+    /// 交回的块是卡上领走之前报的还是之后报的，结局不随先后变）。
     StoppedOnRequest,
     /// 这一轮有卡报了 Out of Memory、还有块没写、没被叫停，而还有没报过的卡：收尾这一步再起一轮，只用没报过的卡判剩下的块
     /// （第 10z 批代码轮判决 K2-1 第 5 条：交回的块在别的卡都收工之后也要有人判，状态全装在第一张卡时也要起第二张）。
     AnotherRoundWithoutTheCardsShortOfMemory,
-    /// 有卡报了 Out of Memory、还有块没写，而报过的卡之外没有卡了（或被叫停）：这一趟报本次未跑，写进库的块在库里。
+    /// 被叫停，这一轮有卡报了 Out of Memory、交回的块前面判完的块在等它，而还有没报过的卡：再起一轮，只用没报过的卡判判完的块在等的那几块
+    /// （叫停照旧：卡只领那几块，[`StopOfTheCards::RequestedByTheCaller`]），判完的都入库再照叫停收尾（第 10z 批代码轮判决 K1-5）。
+    AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor,
+    /// 有卡报了 Out of Memory、还有块没写，而报过的卡之外没有卡了（被叫停的，是判完的块在等的那几块没有卡能判）：这一趟报本次未跑，
+    /// 写进库的块在库里。
     NotRunTheGpuMemoryWasShort,
 }
 
-/// 一轮卡都收工之后：`blocks_left_unstored` 块还没写进库（共要判的减照块序写进的，[`BlocksStoredInOrder::blocks_stored`]），这一轮
-/// `cards_short_of_memory_this_round` 张卡报了 Out of Memory，到这一轮为止没报过的卡还有 `cards_not_short_of_memory` 张。
-/// 每再起一轮至少少一张卡，轮数不超过卡数。
+/// 一轮卡都收工之后：`blocks_left_unstored` 块还没写进库（共要判的减照块序写进的，[`BlocksStoredInOrder::blocks_stored`]），其中
+/// `blocks_waiting` 块判完了在等前面的块，这一轮 `cards_short_of_memory_this_round` 张卡报了 Out of Memory，到这一轮为止没报过的卡还有
+/// `cards_not_short_of_memory` 张。每再起一轮至少少一张卡，轮数不超过卡数。
 #[must_use]
 pub fn after_a_round_of_the_cards(
     blocks_left_unstored: usize,
+    blocks_waiting: usize,
     cards_short_of_memory_this_round: usize,
     stop: StopOfTheRun,
     cards_not_short_of_memory: usize,
@@ -2025,7 +2175,13 @@ pub fn after_a_round_of_the_cards(
         return AfterARoundOfTheCards::StoppedOnRequest;
     }
     match stop {
-        StopOfTheRun::Requested => AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort,
+        StopOfTheRun::Requested if blocks_waiting == 0 => AfterARoundOfTheCards::StoppedOnRequest,
+        StopOfTheRun::Requested if cards_not_short_of_memory == 0 => {
+            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort
+        }
+        StopOfTheRun::Requested => {
+            AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor
+        }
         StopOfTheRun::NotRequested if cards_not_short_of_memory == 0 => {
             AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort
         }
@@ -2035,13 +2191,145 @@ pub fn after_a_round_of_the_cards(
     }
 }
 
+/// 第 ① 段几轮卡之间带着的（崩溃放量产品路径，`singlefs_checker_tier::crash_amplification` 的外层循环；不碰卡，单测在 harness 档随时跑）：
+/// 按块序写库的那一份（几轮共用：上一轮判完、等着前面那一块的，下一轮那一块判完照块序一起写）、下一轮的那一队块、报过 Out of Memory 的卡
+/// 与头一句、收过工的卡。一轮怎么走：[`Self::take_the_blocks_of_the_round`] 交出这一轮的那一队，卡上的线程判、写库那条线程照
+/// [`Self::stored_in_order`] 写，卡都收工之后交 [`Self::after_the_round`]。
+pub struct RoundsOfTheCards<Block> {
+    pending_blocks: usize,
+    stored_in_order: BlocksStoredInOrder<Block>,
+    blocks_of_the_next_round: BlocksForTheCards,
+    cards_short_of_memory: BTreeSet<u32>,
+    first_out_of_memory: Option<String>,
+    cards_that_finished: BTreeSet<u32>,
+}
+
+impl<Block> RoundsOfTheCards<Block> {
+    /// 头一轮之前：`pending_blocks` 块要判（块号 `0..pending_blocks`），一张卡都没用过。
+    #[must_use]
+    pub fn new(pending_blocks: usize) -> Self {
+        Self {
+            pending_blocks,
+            stored_in_order: BlocksStoredInOrder::new(),
+            blocks_of_the_next_round: BlocksForTheCards::new(0..pending_blocks),
+            cards_short_of_memory: BTreeSet::new(),
+            first_out_of_memory: None,
+            cards_that_finished: BTreeSet::new(),
+        }
+    }
+
+    /// 这张卡（`card_index` 是 `nvidia-smi` 给它的序号）这一趟报过 Out of Memory 没有：报过的不再用（每再起一轮至少少一张卡）。
+    #[must_use]
+    pub fn is_short_of_memory(&self, card_index: u32) -> bool {
+        self.cards_short_of_memory.contains(&card_index)
+    }
+
+    /// 这一趟报过 Out of Memory 的卡（序号，进度行用）。
+    #[must_use]
+    pub fn cards_short_of_memory(&self) -> &BTreeSet<u32> {
+        &self.cards_short_of_memory
+    }
+
+    /// 下一轮要判的块号（按序号）。
+    #[must_use]
+    pub fn blocks_of_the_next_round(&self) -> Vec<usize> {
+        self.blocks_of_the_next_round.blocks_not_taken()
+    }
+
+    /// 交出这一轮的那一队（一轮一次；交出之后这里留一队空的，[`Self::after_the_round`] 换上这一轮剩下的）。
+    #[must_use]
+    pub fn take_the_blocks_of_the_round(&mut self) -> BlocksForTheCards {
+        std::mem::replace(
+            &mut self.blocks_of_the_next_round,
+            BlocksForTheCards::new([]),
+        )
+    }
+
+    /// 写库那条线程照它写（几轮共用一份）。
+    pub fn stored_in_order(&mut self) -> &mut BlocksStoredInOrder<Block> {
+        &mut self.stored_in_order
+    }
+
+    /// 照块序写进库了几块。
+    #[must_use]
+    pub fn blocks_stored(&self) -> usize {
+        self.stored_in_order.blocks_stored()
+    }
+
+    /// 判完了、等着前面那一块的有几块。
+    #[must_use]
+    pub fn blocks_waiting(&self) -> usize {
+        self.stored_in_order.blocks_waiting()
+    }
+
+    /// 这一趟头一张报 Out of Memory 的卡报的那一句（没有卡报过是 `None`）。
+    #[must_use]
+    pub fn first_out_of_memory(&self) -> Option<&str> {
+        self.first_out_of_memory.as_deref()
+    }
+
+    /// 收过工的卡有几张：一张卡在几轮里各收工一次只算一张（第 10z 批代码轮判决 K1-5：`cards_used` 算两遍）。
+    #[must_use]
+    pub fn cards_used(&self) -> usize {
+        self.cards_that_finished.len()
+    }
+
+    /// 一轮卡都收工（写库没出错、没有卡拒判）之后：`blocks` 是这一轮的那一队，`cards_of_the_round` 是这一轮挑卡时没报过的卡（装没装状态都算），
+    /// `cards_finished` 是这一轮收了工的卡，`cards_out_of_memory` 是这一轮报了 Out of Memory 的卡与那一句（按收到的次序），`stop` 是这一刻
+    /// 调用方叫没叫停。记下报了的卡、收工的卡，换上下一轮的那一队（[`BlocksForTheCards::left_for_another_round`]），照
+    /// [`after_a_round_of_the_cards`] 交这一趟怎么走。
+    ///
+    /// # Panics
+    /// 这一轮没有卡报 Out of Memory，却还有判完的块在等前面的块（判完的块在等的那几块卡都该领了，叫停了也一样），或没人叫停却还有块没领。
+    pub fn after_the_round(
+        &mut self,
+        blocks: &BlocksForTheCards,
+        cards_of_the_round: &[u32],
+        cards_finished: impl IntoIterator<Item = u32>,
+        cards_out_of_memory: &[(u32, String)],
+        stop: StopOfTheRun,
+    ) -> AfterARoundOfTheCards {
+        self.cards_that_finished.extend(cards_finished);
+        if cards_out_of_memory.is_empty() {
+            assert_eq!(
+                self.stored_in_order.blocks_waiting(),
+                0,
+                "没有卡报 Out of Memory：判完的块每一块都写进了库（叫停了卡也领判完的块在等的那几块）"
+            );
+            assert!(
+                !blocks.any_left() || stop == StopOfTheRun::Requested,
+                "没人叫停，却有 {} 块没判",
+                blocks.blocks_not_taken().len()
+            );
+        }
+        for (index, reason) in cards_out_of_memory {
+            self.cards_short_of_memory.insert(*index);
+            self.first_out_of_memory
+                .get_or_insert_with(|| reason.clone());
+        }
+        self.blocks_of_the_next_round = blocks.left_for_another_round();
+        let cards_not_short_of_memory = cards_of_the_round
+            .iter()
+            .filter(|index| !self.cards_short_of_memory.contains(index))
+            .count();
+        after_a_round_of_the_cards(
+            self.pending_blocks - self.stored_in_order.blocks_stored(),
+            self.stored_in_order.blocks_waiting(),
+            cards_out_of_memory.len(),
+            stop,
+            cards_not_short_of_memory,
+        )
+    }
+}
+
 #[cfg(test)]
 mod tests {
     use super::DeviceStartedOnDemand;
     use super::{
         after_a_round_of_the_cards, judge_blocks_on_one_card, AfterARoundOfTheCards,
         BlockHandedToTheStore, BlockJudgedOnACard, BlocksForTheCards, BlocksStoredInOrder,
-        CardLeftTheBlocks, StopOfTheRun, WorksEndingTogetherOnADevice,
+        CardLeftTheBlocks, CauseOfTheDeviceLoss, DeviceWhenTheWorkReturned, RoundsOfTheCards,
+        StopOfTheCards, StopOfTheRun, WorksEndingTogetherOnADevice,
     };
     use super::{
         attempt_on_a_card_without_room_for_one_state, card_memory_reader_of_one_report_per_reading,
@@ -2059,9 +2347,10 @@ mod tests {
         scratch_booking_refusal_outcome, scratch_states_to_book_on_each_card,
         states_in_one_dispatch_capped_by_the_state_count, BookedByThisProcessOnACard,
         BookingDecision, BookingRefusal, CardMemoryReading, CardRoomForThisProcess,
-        DriverShareOnTheCard, GpuCardsSource, GpuMemoryLedger, NeedLargerThanEveryCard,
-        OccupiedByThisProcess, ScratchAttemptOnACard, ScratchBeforeBooking,
-        ScratchBookingRefusalOutcome, ScratchPlacementDecision, ScratchStatesToBook,
+        DriverShareOnTheCard, GpuCardsSource, GpuMemoryBooking, GpuMemoryLedger,
+        NeedLargerThanEveryCard, OccupiedByThisProcess, ScratchAttemptOnACard,
+        ScratchBeforeBooking, ScratchBookingRefusalOutcome, ScratchPlacementDecision,
+        ScratchStatesToBook,
     };
     use std::sync::atomic::{AtomicUsize, Ordering};
     use std::sync::{Arc, Mutex};
@@ -3765,7 +4054,8 @@ mod tests {
         assert_eq!(
             condition.out_of_memory_explaining_a_failure_of_the_work(
                 this_work.identifier(),
-                reports_before
+                reports_before,
+                DeviceWhenTheWorkReturned::NotLost,
             ),
             None,
             "别的线程上那段活的 Out of Memory：说不成这段活的显存不够"
@@ -3774,7 +4064,8 @@ mod tests {
         assert_eq!(
             condition.out_of_memory_explaining_a_failure_of_the_work(
                 this_work.identifier(),
-                reports_before
+                reports_before,
+                DeviceWhenTheWorkReturned::NotLost,
             ),
             Some("这段活自己判一块时".to_string()),
             "这段活自己这条线程上记的：说得成"
@@ -3800,7 +4091,8 @@ mod tests {
         assert_eq!(
             condition.out_of_memory_explaining_a_failure_of_the_work(
                 compiling_work.identifier(),
-                reports_before
+                reports_before,
+                DeviceWhenTheWorkReturned::NotLost,
             ),
             Some("另起的线程上编恢复内核时没接住的".to_string()),
             "编内核的那一刻另起的线程上记的：算这段活的"
@@ -3814,27 +4106,37 @@ mod tests {
         assert_eq!(
             condition.out_of_memory_explaining_a_failure_of_the_work(
                 compiling_work.identifier(),
-                reports_after_compiling
+                reports_after_compiling,
+                DeviceWhenTheWorkReturned::NotLost,
             ),
             None,
             "编完之后没有活的线程上记的：说不成"
         );
     }
 
-    /// 第 10z 批（第 10y 批代码轮正推）：设备丢了，而这段活开始之后没记过 Out of Memory（早先一段活上处理过的一次不算），说不成显存不够；
-    /// 这段活开始之后记过（别的活上的也算：丢了设备谁都做不成），说成设备因它丢了。
+    /// 第 10z 批（第 10y 批代码轮正推）：设备为别的原因丢了，而这段活开始之后没记过 Out of Memory（早先一段活上处理过的一次不算），
+    /// 说不成显存不够；这段活开始之后记过（别的活上的也算：丢了设备谁都做不成），说成设备因它丢了。
     #[test]
     fn a_lost_device_explains_a_failure_only_with_an_out_of_memory_since_the_work_began() {
         let condition = DeviceConditionOfACard::default();
         condition.record_an_out_of_memory("早先处理过的一次".to_string());
         let work = WorkOnTheCardOfThisThread::begin();
         let reports_before = condition.out_of_memory_reports();
-        condition.record_the_device_lost("Unknown：驱动重置".to_string());
+        condition
+            .record_the_device_lost("Unknown：驱动重置".to_string(), CauseOfTheDeviceLoss::Other);
+        let device_when_the_work_returned = condition.device_when_the_work_returns();
         assert_eq!(
-            condition
-                .out_of_memory_explaining_a_failure_of_the_work(work.identifier(), reports_before),
+            device_when_the_work_returned,
+            DeviceWhenTheWorkReturned::Lost
+        );
+        assert_eq!(
+            condition.out_of_memory_explaining_a_failure_of_the_work(
+                work.identifier(),
+                reports_before,
+                device_when_the_work_returned
+            ),
             None,
-            "设备丢了而这段活开始之后没报过：不是显存不够"
+            "设备为别的原因丢了而这段活开始之后没报过：不是显存不够"
         );
         std::thread::scope(|scope| {
             scope.spawn(|| {
@@ -3844,7 +4146,11 @@ mod tests {
         });
         assert!(
             condition
-                .out_of_memory_explaining_a_failure_of_the_work(work.identifier(), reports_before)
+                .out_of_memory_explaining_a_failure_of_the_work(
+                    work.identifier(),
+                    reports_before,
+                    device_when_the_work_returned
+                )
                 .is_some_and(|reason| reason.starts_with("设备丢了")),
             "设备丢了、这段活开始之后报过：说成设备因它丢了"
         );
@@ -4118,7 +4424,8 @@ mod tests {
         assert_eq!(
             condition.out_of_memory_explaining_a_failure_of_the_work(
                 outer_work.identifier(),
-                reports_before
+                reports_before,
+                DeviceWhenTheWorkReturned::NotLost,
             ),
             Some("里面那段收完之后外面那段撞满".to_string())
         );
@@ -4142,23 +4449,35 @@ mod tests {
         assert_eq!(
             condition.out_of_memory_explaining_a_failure_of_the_work(
                 compiling_work.identifier(),
-                reports_before
+                reports_before,
+                DeviceWhenTheWorkReturned::NotLost,
             ),
             Some("外面那一处还在编时没接住的".to_string())
         );
     }
 
-    /// 第 10z 批第三块（第 10z 批代码轮正推 K3 第 3 条）：记了丢失的设备是丢了的，没记的不是（卡池照它判留着的设备要不要重起）。
+    /// 第 10z 批第三块（第 10z 批代码轮正推 K3 第 3 条）：记了丢失的设备是丢了的，没记的不是（卡池照它判留着的设备要不要重起）；
+    /// 一段活交回那一刻照它记设备丢没丢。
     #[test]
     fn a_device_recorded_lost_is_lost_and_another_is_not() {
         let condition = DeviceConditionOfACard::default();
         assert!(!condition.is_lost(), "没记丢失");
-        condition.record_the_device_lost("Unknown：驱动重置".to_string());
+        assert_eq!(
+            condition.device_when_the_work_returns(),
+            DeviceWhenTheWorkReturned::NotLost
+        );
+        condition
+            .record_the_device_lost("Unknown：驱动重置".to_string(), CauseOfTheDeviceLoss::Other);
         assert!(condition.is_lost(), "记了丢失");
+        assert_eq!(
+            condition.device_when_the_work_returns(),
+            DeviceWhenTheWorkReturned::Lost
+        );
     }
 
-    /// 第 10z 批第三块（第 10z 批代码轮判决 K1-3 攻方）：一块的两段在同一个设备上并发，一段因设备丢了先失败、先来认；另一段范围里接到的
-    /// Out of Memory 要等那一段先收完才记进设备。先失败的那一段等两段都报到再认：认得出是设备因显存不够丢了，不判红。
+    /// 第 10z 批第三块（第 10z 批代码轮判决 K1-3 攻方）：一块的两段在同一个设备上并发，一段因设备丢了（wgpu 说设备丢了，不是它的显存不够）
+    /// 先失败、先来认；另一段范围里接到的 Out of Memory 要等那一段先收完才记进设备。先失败的那一段等两段都报到再认：认得出设备丢之前报过
+    /// Out of Memory，不判红。
     #[test]
     fn a_piece_failing_on_a_lost_device_is_told_after_another_piece_records_its_out_of_memory() {
         let condition = &DeviceConditionOfACard::default();
@@ -4169,10 +4488,14 @@ mod tests {
             let failed_piece = scope.spawn(move || {
                 let work = WorkOnTheCardOfThisThread::begin();
                 let reports_before = condition.out_of_memory_reports();
-                condition.record_the_device_lost("Unknown：提交时显存不够".to_string());
+                condition.record_the_device_lost(
+                    "Unknown：Device is lost".to_string(),
+                    CauseOfTheDeviceLoss::Other,
+                );
                 condition.end_the_work_and_explain_its_failure(
                     &work,
                     reports_before,
+                    condition.device_when_the_work_returns(),
                     None,
                     failed_piece_ending,
                 )
@@ -4191,6 +4514,7 @@ mod tests {
                 condition.end_the_work_and_explain_its_failure(
                     &work,
                     reports_before,
+                    condition.device_when_the_work_returns(),
                     Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string()),
                     caught_piece_ending,
                 )
@@ -4201,8 +4525,235 @@ mod tests {
             explained
                 .as_deref()
                 .is_some_and(|reason| reason.starts_with("设备丢了")),
-            "等另一段记进设备之后再认：说成设备因显存不够丢了，{explained:?}"
+            "等另一段记进设备之后再认：说成设备丢之前报过 Out of Memory，{explained:?}"
+        );
+    }
+
+    /// wgpu 因显存不够丢设备时给的那一句：`wgpu_hal::DeviceError::OutOfMemory` 说的（wgpu-hal 30.0.1 `src/lib.rs` 第 382 行），
+    /// 丢失回调照 `{reason:?}：{message}` 记（`singlefs_checker_tier::gpu_unit_checks`）。
+    const LOST_FOR_LACK_OF_MEMORY: &str = "Unknown：Out of memory";
+
+    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K1-4 攻方甲形，`attack_10zc_endings.rs` 的 `cross-group`）：同一个设备上不在同一组的两段活
+    /// （同一测试进程里两条用例共用卡池的设备）：X 的范围里接到 Out of Memory、wgpu 因它丢了设备，X 还没收完；Y 因设备丢了先失败、先收完，
+    /// 等的只是它自己。设备是因显存不够丢的，Y 认成显存不够、不判红；X 收完认它自己那一句。同组对照照旧认得出。
+    #[test]
+    fn a_work_failing_on_a_device_another_group_lost_for_lack_of_memory_is_told_out_of_memory() {
+        for same_group in [false, true] {
+            let condition = &DeviceConditionOfACard::default();
+            let shared = &WorksEndingTogetherOnADevice::new();
+            let alone_x = &WorksEndingTogetherOnADevice::new();
+            let alone_y = &WorksEndingTogetherOnADevice::new();
+            let (ending_x, ending_y) = if same_group {
+                (shared.join(), shared.join())
+            } else {
+                (alone_x.join(), alone_y.join())
+            };
+            let (y_began_sender, y_began) = std::sync::mpsc::channel::<()>();
+            let (y_ended_sender, y_ended) = std::sync::mpsc::channel::<()>();
+            let (x_told, y_told) = std::thread::scope(|scope| {
+                let x = scope.spawn(move || {
+                    let work = WorkOnTheCardOfThisThread::begin();
+                    let reports_before = condition.out_of_memory_reports();
+                    y_began
+                        .recv_timeout(std::time::Duration::from_secs(60))
+                        .expect("Y 一分钟之内开了活");
+                    condition.record_the_device_lost(
+                        LOST_FOR_LACK_OF_MEMORY.to_string(),
+                        CauseOfTheDeviceLoss::GpuMemoryShort,
+                    );
+                    let device_when_the_work_returned = condition.device_when_the_work_returns();
+                    if !same_group {
+                        // 不在同一组：Y 先收完，X 收得比 Y 晚
+                        y_ended
+                            .recv_timeout(std::time::Duration::from_secs(60))
+                            .expect("Y 一分钟之内收完");
+                    }
+                    condition.end_the_work_and_explain_its_failure(
+                        &work,
+                        reports_before,
+                        device_when_the_work_returned,
+                        Some("序号 0 的卡判一块时报了 Out of Memory".to_string()),
+                        ending_x,
+                    )
+                });
+                let y = scope.spawn(move || {
+                    let work = WorkOnTheCardOfThisThread::begin();
+                    let reports_before = condition.out_of_memory_reports();
+                    y_began_sender.send(()).expect("X 在等 Y 开活");
+                    let waiting_since = std::time::Instant::now();
+                    while !condition.is_lost() {
+                        assert!(
+                            waiting_since.elapsed() < std::time::Duration::from_secs(60),
+                            "X 一分钟都没丢设备"
+                        );
+                        std::thread::yield_now();
+                    }
+                    let explained = condition.end_the_work_and_explain_its_failure(
+                        &work,
+                        reports_before,
+                        condition.device_when_the_work_returns(),
+                        None,
+                        ending_y,
+                    );
+                    // 同组时 X 不等这一句，发不出去也无妨
+                    let _ = y_ended_sender.send(());
+                    explained
+                });
+                (x.join().expect("X 不 panic"), y.join().expect("Y 不 panic"))
+            });
+            assert_eq!(
+                x_told,
+                Some("序号 0 的卡判一块时报了 Out of Memory".to_string()),
+                "同组 {same_group}：X 认它自己那一句"
+            );
+            assert!(
+                y_told.is_some(),
+                "同组 {same_group}：设备因显存不够丢了，Y 认成显存不够、不判红"
+            );
+        }
+    }
+
+    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K1-4 攻方乙形 `genuine-panic-widened`、正推 `explain_demo.rs`）：一块两段 A、B 同组。
+    /// A 的活自己真 panic 了，交回那一刻设备没丢、设备上一条 Out of Memory 都没有；A 报到之后 B 的范围里接到 Out of Memory（设备丢没丢两样都试），
+    /// B 收完把它记进设备。A 等齐再认：只认算在它自己头上的，说不成显存不够（照样往外抛、判红）；B 认得出它自己那一句。
+    #[test]
+    fn a_genuine_panic_while_the_device_was_there_is_not_told_out_of_memory_after_another_piece_runs_out(
+    ) {
+        for b_loses_the_device in [false, true] {
+            let condition = &DeviceConditionOfACard::default();
+            let together = &WorksEndingTogetherOnADevice::new();
+            let ending_a = together.join();
+            let ending_b = together.join();
+            let (a_told, b_told) = std::thread::scope(|scope| {
+                let a = scope.spawn(move || {
+                    let work = WorkOnTheCardOfThisThread::begin();
+                    let reports_before = condition.out_of_memory_reports();
+                    // A 的活 panic 了（真毛病）：交回这一刻设备没丢
+                    let device_when_the_work_returned = condition.device_when_the_work_returns();
+                    condition.end_the_work_and_explain_its_failure(
+                        &work,
+                        reports_before,
+                        device_when_the_work_returned,
+                        None,
+                        ending_a,
+                    )
+                });
+                let b = scope.spawn(move || {
+                    let work = WorkOnTheCardOfThisThread::begin();
+                    let reports_before = condition.out_of_memory_reports();
+                    let waiting_since = std::time::Instant::now();
+                    while together.works_not_yet_ended() > 1 {
+                        assert!(
+                            waiting_since.elapsed() < std::time::Duration::from_secs(60),
+                            "A 一分钟都没报到"
+                        );
+                        std::thread::yield_now();
+                    }
+                    if b_loses_the_device {
+                        condition.record_the_device_lost(
+                            LOST_FOR_LACK_OF_MEMORY.to_string(),
+                            CauseOfTheDeviceLoss::GpuMemoryShort,
+                        );
+                    }
+                    condition.end_the_work_and_explain_its_failure(
+                        &work,
+                        reports_before,
+                        condition.device_when_the_work_returns(),
+                        Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string()),
+                        ending_b,
+                    )
+                });
+                (a.join().expect("A 不 panic"), b.join().expect("B 不 panic"))
+            });
+            assert_eq!(
+                a_told, None,
+                "B 丢设备 {b_loses_the_device}：A 交回那一刻设备没丢，它的 panic 说不成显存不够"
+            );
+            assert_eq!(
+                b_told,
+                Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string()),
+                "B 丢设备 {b_loses_the_device}：B 认它自己那一句"
+            );
+        }
+    }
+
+    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K3 攻方 `record-the-caught-after-waiting`）：一段范围里接到的 Out of Memory 先记进设备再报到：
+    /// 它报到的那一刻（还在等同组别的段）设备上已经记着，先收完的别的段这时来认就认得出。改成等完之后再记，报到的那一刻设备上一条都没有。
+    #[test]
+    fn a_piece_records_what_its_scope_caught_before_it_reports_its_ending() {
+        let condition: &'static DeviceConditionOfACard =
+            Box::leak(Box::new(DeviceConditionOfACard::default()));
+        let together: &'static WorksEndingTogetherOnADevice =
+            Box::leak(Box::new(WorksEndingTogetherOnADevice::new()));
+        let held_ending = together.join();
+        let caught_piece_ending = together.join();
+        let caught_piece = std::thread::spawn(move || {
+            let work = WorkOnTheCardOfThisThread::begin();
+            let reports_before = condition.out_of_memory_reports();
+            condition.end_the_work_and_explain_its_failure(
+                &work,
+                reports_before,
+                condition.device_when_the_work_returns(),
+                Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string()),
+                caught_piece_ending,
+            )
+        });
+        let waiting_since = std::time::Instant::now();
+        while together.works_not_yet_ended() > 1 {
+            assert!(
+                waiting_since.elapsed() < std::time::Duration::from_secs(60),
+                "接到 Out of Memory 的那一段一分钟都没报到"
+            );
+            std::thread::yield_now();
+        }
+        let reports_when_it_reported = condition.out_of_memory_reports();
+        drop(held_ending);
+        let explained = caught_piece.join().expect("那一段不 panic");
+        assert_eq!(
+            reports_when_it_reported, 1,
+            "报到的那一刻范围里接到的已经记进设备"
         );
+        assert_eq!(
+            explained,
+            Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string())
+        );
+    }
+
+    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K3 攻方 `endings-notify-one`）：一组三段，两段先报到、在等；最后一段报到时叫醒等着的每一段，
+    /// 两段都在一分钟之内等完（一块的段数是核对线程数，本机 32）。只叫醒一段时另一段一直等（这条用例一分钟之后判红）。
+    #[test]
+    fn the_last_piece_to_report_wakes_every_piece_waiting_for_it() {
+        let together: &'static WorksEndingTogetherOnADevice =
+            Box::leak(Box::new(WorksEndingTogetherOnADevice::new()));
+        let waiting_endings = [together.join(), together.join()];
+        let last_ending = together.join();
+        let (ended_sender, ended_receiver) = std::sync::mpsc::channel();
+        for ending in waiting_endings {
+            let ended_sender = ended_sender.clone();
+            std::thread::spawn(move || {
+                ending.end_and_wait_for_the_others();
+                let _ = ended_sender.send(());
+            });
+        }
+        drop(ended_sender);
+        let waiting_since = std::time::Instant::now();
+        while together.works_not_yet_ended() > 1 {
+            assert!(
+                waiting_since.elapsed() < std::time::Duration::from_secs(60),
+                "先报到的两段一分钟都没报到"
+            );
+            std::thread::yield_now();
+        }
+        // 报到之后到进等待之间还有几条指令：给那两段时间都等上
+        std::thread::sleep(std::time::Duration::from_millis(200));
+        last_ending.end_and_wait_for_the_others();
+        for waiting_piece in 0..2 {
+            assert_eq!(
+                ended_receiver.recv_timeout(std::time::Duration::from_secs(60)),
+                Ok(()),
+                "等着的第 {waiting_piece} 段一分钟之内被叫醒、等完"
+            );
+        }
     }
 
     /// 第 10z 批第三块：一段领了报到、没报到就丢掉（那一段 panic 了），算它报到了，别的段不等它。
@@ -4230,20 +4781,148 @@ mod tests {
     /// 交回的都领走、队里也领完才算没有剩下的。
     #[test]
     fn a_block_given_back_by_a_card_out_of_memory_goes_to_the_next_card_that_takes_one() {
+        let go_on = StopOfTheCards::NotRequested;
         let blocks = BlocksForTheCards::new(0..3);
-        assert_eq!(blocks.take_a_block(), Some(0));
-        assert_eq!(blocks.take_a_block(), Some(1));
+        assert_eq!(blocks.take_a_block(go_on), Some(0));
+        assert_eq!(blocks.take_a_block(go_on), Some(1));
         blocks.give_back(0);
         assert!(blocks.any_left());
-        assert_eq!(blocks.take_a_block(), Some(0), "交回的先领");
-        assert_eq!(blocks.take_a_block(), Some(2));
-        assert_eq!(blocks.take_a_block(), None);
+        assert_eq!(blocks.take_a_block(go_on), Some(0), "交回的先领");
+        assert_eq!(blocks.take_a_block(go_on), Some(2));
+        assert_eq!(blocks.take_a_block(go_on), None);
         assert!(!blocks.any_left());
         blocks.give_back(1);
         assert!(blocks.any_left(), "队里领完了、还有交回的");
         assert_eq!(blocks.blocks_not_taken(), vec![1]);
-        assert_eq!(blocks.take_a_block(), Some(1));
-        assert_eq!(blocks.take_a_block(), None);
+        assert_eq!(blocks.take_a_block(go_on), Some(1));
+        assert_eq!(blocks.take_a_block(go_on), None);
+    }
+
+    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K1-5、K3 攻方 `stop-checked-after-taking-a-block`、正推 M18）：调用方叫停之后卡只领判完的块
+    /// 在等的那几块（比判完的块号小、交回队里的），别的留在队里给下一趟；一块都没判完时一块都不领、队里原样；判红了连判完的块在等的也不领；
+    /// 下一轮的那一队记着判完的块号，叫停了照样领判完的块在等的。
+    #[test]
+    fn a_card_asked_to_stop_takes_only_the_blocks_judged_blocks_wait_for_and_leaves_the_rest() {
+        let blocks = BlocksForTheCards::new(0..5);
+        let mut judged_while_stopped = Vec::new();
+        let left_while_stopped = judge_blocks_on_one_card(
+            &blocks,
+            &|| StopOfTheCards::RequestedByTheCaller,
+            &mut |pending_index| {
+                judged_while_stopped.push(pending_index);
+                BlockJudgedOnACard::Judged(pending_index)
+            },
+            &mut |_pending_index, _judged| BlockHandedToTheStore::Handed,
+        );
+        assert_eq!(left_while_stopped, CardLeftTheBlocks::NothingLeftOrStopped);
+        assert!(
+            judged_while_stopped.is_empty(),
+            "叫停了、没有判完的块在等：一块都不领"
+        );
+        assert_eq!(
+            blocks.blocks_not_taken(),
+            vec![0, 1, 2, 3, 4],
+            "叫停时没领的块原样留在队里"
+        );
+        // 第一张卡领了块 0；第二张判完块 1、2 之后被叫停
+        assert_eq!(blocks.take_a_block(StopOfTheCards::NotRequested), Some(0));
+        let judged_by_the_second_card = std::cell::RefCell::new(Vec::new());
+        let left_by_the_second_card = judge_blocks_on_one_card(
+            &blocks,
+            &|| {
+                if judged_by_the_second_card.borrow().len() >= 2 {
+                    StopOfTheCards::RequestedByTheCaller
+                } else {
+                    StopOfTheCards::NotRequested
+                }
+            },
+            &mut |pending_index| {
+                judged_by_the_second_card.borrow_mut().push(pending_index);
+                BlockJudgedOnACard::Judged(pending_index)
+            },
+            &mut |_pending_index, _judged| BlockHandedToTheStore::Handed,
+        );
+        assert_eq!(
+            left_by_the_second_card,
+            CardLeftTheBlocks::NothingLeftOrStopped
+        );
+        assert_eq!(
+            judged_by_the_second_card.into_inner(),
+            vec![1, 2],
+            "叫停之后块 3 比判完的块 2 大：不领"
+        );
+        // 第一张卡在块 0 上报了 Out of Memory、交回
+        blocks.give_back(0);
+        assert!(
+            blocks.any_left_to_take(StopOfTheCards::RequestedByTheCaller),
+            "交回的块 0 比判完的块 2 小：叫停了也领得"
+        );
+        assert!(
+            !blocks.any_left_to_take(StopOfTheCards::TheRunFailed),
+            "判红了一块都不领"
+        );
+        assert_eq!(blocks.take_a_block(StopOfTheCards::TheRunFailed), None);
+        let next_round = blocks.left_for_another_round();
+        assert_eq!(next_round.blocks_not_taken(), vec![0, 3, 4]);
+        assert_eq!(
+            next_round.take_a_block(StopOfTheCards::RequestedByTheCaller),
+            Some(0),
+            "下一轮的那一队记着判完的块 2：叫停了照样领块 0"
+        );
+        assert_eq!(
+            next_round.take_a_block(StopOfTheCards::RequestedByTheCaller),
+            None,
+            "块 3 留给下一趟"
+        );
+        assert_eq!(next_round.blocks_not_taken(), vec![3, 4]);
+    }
+
+    /// 第 10z 批第五块（第 10z 批第三块代码轮正推 M30、M31）：判不了一块的卡收工、那一块不交回，不再领；判完的一块交不到写库那一头，
+    /// 这张卡收工、不再领。
+    #[test]
+    fn a_card_stops_taking_blocks_after_a_refusal_and_after_the_store_is_gone() {
+        let blocks_of_the_refusing_card = BlocksForTheCards::new(0..5);
+        let mut taken = Vec::new();
+        let left_by_the_refusing_card = judge_blocks_on_one_card(
+            &blocks_of_the_refusing_card,
+            &|| StopOfTheCards::NotRequested,
+            &mut |pending_index| {
+                taken.push(pending_index);
+                if pending_index == 1 {
+                    BlockJudgedOnACard::<usize>::Refused {
+                        reason: "判不了块 1".to_string(),
+                    }
+                } else {
+                    BlockJudgedOnACard::Judged(pending_index)
+                }
+            },
+            &mut |_pending_index, _judged| BlockHandedToTheStore::Handed,
+        );
+        assert_eq!(
+            left_by_the_refusing_card,
+            CardLeftTheBlocks::Refused {
+                reason: "判不了块 1".to_string()
+            }
+        );
+        assert_eq!(taken, vec![0, 1], "判不了的那一块之后不再领");
+        assert_eq!(
+            blocks_of_the_refusing_card.blocks_not_taken(),
+            vec![2, 3, 4],
+            "判不了的那一块不交回"
+        );
+        let blocks_of_the_card_without_a_store = BlocksForTheCards::new(0..5);
+        let mut handed = 0;
+        let left_without_a_store = judge_blocks_on_one_card(
+            &blocks_of_the_card_without_a_store,
+            &|| StopOfTheCards::NotRequested,
+            &mut |pending_index| BlockJudgedOnACard::Judged(pending_index),
+            &mut |_pending_index, _judged| {
+                handed += 1;
+                BlockHandedToTheStore::TheStoreIsGone
+            },
+        );
+        assert_eq!(left_without_a_store, CardLeftTheBlocks::TheStoreIsGone);
+        assert_eq!(handed, 1, "写库那一头走了：不再领");
     }
 
     /// 第 10z 批第三块：判完的块按块序写库，后面的先到就等着；写库失败之后到的不再写。
@@ -4271,34 +4950,298 @@ mod tests {
         assert_eq!(failing.storing_error(), Some("写判定块失败"));
     }
 
-    /// 第 10z 批第三块：一轮卡收工之后这一趟怎么走，逐种。
+    /// 第 10z 批第三块、第五块：一轮卡收工之后这一趟怎么走，逐种（参数：没写的块、其中判完在等的、这一轮报了的卡、叫没叫停、没报过的卡）。
     #[test]
     fn after_a_round_another_round_starts_only_when_a_card_ran_out_and_another_card_is_left() {
         assert_eq!(
-            after_a_round_of_the_cards(0, 1, StopOfTheRun::NotRequested, 1),
+            after_a_round_of_the_cards(0, 0, 1, StopOfTheRun::NotRequested, 1),
             AfterARoundOfTheCards::EveryBlockStored,
             "有卡报了而别的卡判完了：照常交回"
         );
         assert_eq!(
-            after_a_round_of_the_cards(3, 0, StopOfTheRun::Requested, 2),
+            after_a_round_of_the_cards(3, 0, 0, StopOfTheRun::Requested, 2),
             AfterARoundOfTheCards::StoppedOnRequest
         );
         assert_eq!(
-            after_a_round_of_the_cards(3, 1, StopOfTheRun::NotRequested, 1),
+            after_a_round_of_the_cards(3, 0, 1, StopOfTheRun::NotRequested, 1),
             AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory
         );
         assert_eq!(
-            after_a_round_of_the_cards(3, 1, StopOfTheRun::NotRequested, 0),
+            after_a_round_of_the_cards(3, 2, 1, StopOfTheRun::NotRequested, 1),
+            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory,
+            "没叫停：等着的块随剩下的一起判"
+        );
+        assert_eq!(
+            after_a_round_of_the_cards(3, 0, 1, StopOfTheRun::NotRequested, 0),
             AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort,
             "报过的卡之外没有卡了"
         );
         assert_eq!(
-            after_a_round_of_the_cards(3, 1, StopOfTheRun::Requested, 1),
+            after_a_round_of_the_cards(3, 0, 1, StopOfTheRun::Requested, 1),
+            AfterARoundOfTheCards::StoppedOnRequest,
+            "叫停了、判完的块都写进了库：照叫停收尾，不再起一轮"
+        );
+        assert_eq!(
+            after_a_round_of_the_cards(3, 2, 1, StopOfTheRun::Requested, 1),
+            AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor,
+            "叫停了、判完的块在等交回的那一块：再起一轮只判它"
+        );
+        assert_eq!(
+            after_a_round_of_the_cards(3, 2, 1, StopOfTheRun::Requested, 0),
             AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort,
-            "叫停了不再起一轮"
+            "叫停了、判完的块在等而没有卡能判了"
         );
     }
 
+    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K1-5 攻方乙、正推第 4 点）：几轮之间带着的那几样：报了 Out of Memory 的卡记下、下一轮不再用，
+    /// 下一轮只判这一轮没领走的，头一句 Out of Memory 留着；一张卡两轮各收工一次只算一张。
+    #[test]
+    fn rounds_keep_the_cards_short_of_memory_out_and_count_a_card_finishing_twice_once() {
+        let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(2);
+        assert_eq!(rounds.blocks_of_the_next_round(), vec![0, 1]);
+        let blocks_of_the_first_round = rounds.take_the_blocks_of_the_round();
+        // 头一轮：卡 0 领了块 0、报了 Out of Memory 交回；卡 1 判完块 1（等着块 0）收工
+        assert_eq!(
+            blocks_of_the_first_round.take_a_block(StopOfTheCards::NotRequested),
+            Some(0)
+        );
+        assert_eq!(
+            blocks_of_the_first_round.take_a_block(StopOfTheCards::NotRequested),
+            Some(1)
+        );
+        blocks_of_the_first_round.record_a_judged_block(1);
+        rounds.stored_in_order().arrive(1, 1, &mut |_block| Ok(()));
+        blocks_of_the_first_round.give_back(0);
+        assert_eq!(
+            rounds.after_the_round(
+                &blocks_of_the_first_round,
+                &[0, 1],
+                [1],
+                &[(0, "显存不够：卡 0 判块 0".to_string())],
+                StopOfTheRun::NotRequested,
+            ),
+            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory
+        );
+        assert!(rounds.is_short_of_memory(0), "报了的卡 0 不再用");
+        assert!(!rounds.is_short_of_memory(1));
+        assert_eq!(rounds.first_out_of_memory(), Some("显存不够：卡 0 判块 0"));
+        assert_eq!(
+            rounds.blocks_of_the_next_round(),
+            vec![0],
+            "下一轮只判没领走的块 0，判完的块 1 不再判"
+        );
+        let blocks_of_the_second_round = rounds.take_the_blocks_of_the_round();
+        assert_eq!(
+            blocks_of_the_second_round.take_a_block(StopOfTheCards::NotRequested),
+            Some(0)
+        );
+        rounds.stored_in_order().arrive(0, 0, &mut |_block| Ok(()));
+        assert_eq!(rounds.blocks_stored(), 2);
+        assert_eq!(
+            rounds.after_the_round(
+                &blocks_of_the_second_round,
+                &[1],
+                [1],
+                &[],
+                StopOfTheRun::NotRequested
+            ),
+            AfterARoundOfTheCards::EveryBlockStored
+        );
+        assert_eq!(rounds.cards_used(), 1, "卡 1 两轮各收工一次只算一张");
+    }
+
+    /// 正推 `probes/killers.rs` 那几条用的一张假 5090：照它此刻卡上有什么给读数（这个进程只经账占卡），驱动那一份 [`DRIVER`]。
+    #[derive(Clone, Default)]
+    struct CardUnderTheLedger {
+        device_started: bool,
+        /// 起着的设备驱动实占的。
+        driver_occupied_mebibytes: u64,
+        /// 别的进程占的。
+        others_mebibytes: u64,
+        /// 活着的借据建上卡的合计。
+        built_mebibytes: u64,
+        /// 销了账、卡上还没还的。
+        returned_still_on_the_card_mebibytes: u64,
+        /// 丢了的设备留在卡上的。
+        left_by_a_lost_device_mebibytes: u64,
+    }
+
+    impl CardUnderTheLedger {
+        fn occupied_by_this_process_mebibytes(&self) -> u64 {
+            (if self.device_started {
+                self.driver_occupied_mebibytes
+            } else {
+                0
+            }) + self.built_mebibytes
+                + self.returned_still_on_the_card_mebibytes
+                + self.left_by_a_lost_device_mebibytes
+        }
+
+        fn reading(&self) -> CardMemoryReading {
+            let occupied = self.occupied_by_this_process_mebibytes();
+            CardMemoryReading {
+                free_mebibytes: AN_EMPTY_RTX_5090_HAS - self.others_mebibytes - occupied,
+                total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
+                occupied_by_this_process: if occupied > 0 {
+                    OccupiedByThisProcess::OnlyThroughTheBooks {
+                        mebibytes: occupied,
+                    }
+                } else {
+                    OccupiedByThisProcess::NotListed
+                },
+            }
+        }
+    }
+
+    /// 账接一张 [`CardUnderTheLedger`]：没配额度、没配优先级（驱动之外照能占的算）。
+    fn ledger_on_a_card_under_the_ledger() -> (Arc<Mutex<CardUnderTheLedger>>, GpuMemoryLedger) {
+        let card = Arc::new(Mutex::new(CardUnderTheLedger::default()));
+        let read_card = Arc::clone(&card);
+        let ledger = GpuMemoryLedger::new(
+            1,
+            Box::new(move || Ok(vec![read_card.lock().expect("假卡不 panic").reading()])),
+            Box::new(|_card_position, may_hold| {
+                card_ledger_capacity_mebibytes(None, None, may_hold, may_hold, DRIVER)
+            }),
+            DRIVER,
+        );
+        (card, ledger)
+    }
+
+    fn room_now(ledger: &GpuMemoryLedger) -> u64 {
+        ledger.card_rooms_mebibytes_now(&[0]).expect("假卡读得出")[0]
+    }
+
+    /// 借一笔 10 000、起设备、建满：之后的几条从这里起。
+    fn ten_thousand_built_on_a_started_device<'ledger>(
+        card: &Mutex<CardUnderTheLedger>,
+        ledger: &'ledger GpuMemoryLedger,
+    ) -> GpuMemoryBooking<'ledger> {
+        let lease = ledger
+            .book_on_the_card(0, 10_000)
+            .expect("空卡借得到 10 000");
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = true;
+            card.driver_occupied_mebibytes = DRIVER;
+            card.built_mebibytes = 10_000;
+        }
+        ledger.record_the_device_started_on_the_card(0);
+        lease
+    }
+
+    /// 设备丢了：驱动那一份与建上的留在卡上当旧设备那一堆，别的进程占了 `others_mebibytes`。
+    fn the_device_is_lost_with_others_taking(
+        card: &Mutex<CardUnderTheLedger>,
+        others_mebibytes: u64,
+    ) {
+        let mut card = card.lock().expect("假卡不 panic");
+        card.device_started = false;
+        card.driver_occupied_mebibytes = 0;
+        card.left_by_a_lost_device_mebibytes = DRIVER;
+        card.others_mebibytes = others_mebibytes;
+    }
+
+    /// 第 10z 批第五块（第 10z 批第三块代码轮正推 M01，`probes/killers.rs`）：长借据建满又还了、卡上还给驱动之后，新借一笔建满，
+    /// 能借的照读数把还了的那一份压到这个进程占着的为止：新借的建上的算进占着的，少的只是驱动那一份（甲的代价）。读数不进账就只剩 16 990。
+    #[test]
+    fn a_reading_after_the_card_took_a_returned_lease_back_lets_a_later_lease_count_once() {
+        let (card, ledger) = ledger_on_a_card_under_the_ledger();
+        let first = ledger.book_on_the_card(0, 10_000).expect("空卡借得到");
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = true;
+            card.driver_occupied_mebibytes = DRIVER;
+            card.built_mebibytes = 10_000;
+        }
+        ledger.record_the_device_started_on_the_card(0);
+        drop(first);
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.built_mebibytes = 0;
+            card.returned_still_on_the_card_mebibytes = 10_000;
+        }
+        let _ = room_now(&ledger);
+        card.lock()
+            .expect("假卡不 panic")
+            .returned_still_on_the_card_mebibytes = 0;
+        let _ = room_now(&ledger);
+        let _second = ledger.book_on_the_card(0, 5_000).expect("借得到 5 000");
+        card.lock().expect("假卡不 panic").built_mebibytes = 5_000;
+        assert_eq!(
+            room_now(&ledger),
+            AN_EMPTY_RTX_5090_HAS - DRIVER - 5_000 - DRIVER,
+            "第二笔建满：能借的 21 990（读数不压就是 16 990）"
+        );
+    }
+
+    /// 第 10z 批第五块（第 10z 批第三块代码轮正推 M05）：设备丢了、重起被拒一次之后，再被问三次而卡上读数没变，能借的不变
+    /// （被拒之后记成「丢了、还没重起」，不记回「起着」而每问一次多扣一份）。
+    #[test]
+    fn a_refused_restart_asked_again_takes_no_further_driver_share_off_the_room() {
+        let (card, ledger) = ledger_on_a_card_under_the_ledger();
+        let _lease = ten_thousand_built_on_a_started_device(&card, &ledger);
+        // 新设备放不下：卡上 32 110 − 19 000 − 2 560 − 10 000 = 550，驱动那一份要 2 560
+        the_device_is_lost_with_others_taking(&card, 19_000);
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            Err(BookingRefusal::NoCardHasRoomNow {
+                need_mebibytes: DRIVER,
+                card_rooms_mebibytes: vec![0],
+            })
+        );
+        card.lock().expect("假卡不 panic").others_mebibytes = 0;
+        let room_after_one_refusal = room_now(&ledger);
+        for _ in 0..3 {
+            let _ = ledger.prepare_the_driver_share_before_starting_the_device(0);
+        }
+        assert_eq!(
+            room_now(&ledger),
+            room_after_one_refusal,
+            "再问三次、读数没变：能借的不变"
+        );
+    }
+
+    /// 第 10z 批第五块（第 10z 批第三块代码轮正推 M13）：设备丢了、重起被拒之后又借了一笔，之后卡上又被占：再来起设备照此刻判、放不下就拒
+    /// （借一笔不把驱动那一份记回「登记了、没起」而直接放行）。
+    #[test]
+    fn a_lease_booked_after_a_refused_restart_keeps_the_restart_judged_again() {
+        let (card, ledger) = ledger_on_a_card_under_the_ledger();
+        let _lease = ten_thousand_built_on_a_started_device(&card, &ledger);
+        the_device_is_lost_with_others_taking(&card, 19_000);
+        assert!(ledger
+            .prepare_the_driver_share_before_starting_the_device(0)
+            .is_err());
+        card.lock().expect("假卡不 panic").others_mebibytes = 0;
+        let _another = ledger
+            .book_on_the_card(0, 5_000)
+            .expect("别的进程走了，借得到");
+        card.lock().expect("假卡不 panic").others_mebibytes = 19_000;
+        assert!(
+            ledger
+                .prepare_the_driver_share_before_starting_the_device(0)
+                .is_err(),
+            "被拒之后借了一笔，再来起设备照此刻判：放不下"
+        );
+    }
+
+    /// 第 10z 批第五块（第 10z 批第三块代码轮正推 M10）：设备丢了要重起，借着的 10 000 连新设备的驱动那一份恰好放得下就起，差 1 MiB 就拒。
+    #[test]
+    fn a_restart_is_allowed_when_it_fits_exactly_and_refused_one_mebibyte_short() {
+        for (others_mebibytes, fits) in [(16_990u64, true), (16_991, false)] {
+            let (card, ledger) = ledger_on_a_card_under_the_ledger();
+            let _lease = ten_thousand_built_on_a_started_device(&card, &ledger);
+            the_device_is_lost_with_others_taking(&card, others_mebibytes);
+            assert_eq!(
+                ledger
+                    .prepare_the_driver_share_before_starting_the_device(0)
+                    .is_ok(),
+                fits,
+                "别的进程占着 {others_mebibytes}"
+            );
+        }
+    }
+
     /// P2 那几条用例的假卡按拍走的挂钟（次序照攻方模型 `research/prompts/m3-batch10z-a-code-r1-opus-model/p2_blocks_10z.rs`）：每张卡判一块占几拍；
     /// 挂钟只在没有一张卡在跑（都停在判块上、或收了工）时往前走，最先判完的那一张先往下，同一拍卡号大的先。起卡的那一方也算在跑，
     /// 卡一张一张起完它才让开，卡领第一块的次序因此照卡号。
@@ -4383,56 +5326,84 @@ mod tests {
         }
     }
 
-    /// 假卡上几轮判完一队块的结局：照块序写进库的块、每块判了几遍、最后一轮之后怎么走。
+    /// 假卡那一趟里调用方什么时候叫停。
+    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
+    enum CallerStopOfTheFakeRun {
+        Never,
+        /// 几轮合计判完第 n 块（从 1 数）的那一刻。
+        WhenTheBlocksJudgedReach(usize),
+        /// 第 0 张卡报 Out of Memory 的那一刻（交回那一块之前）。
+        WhenCard0RunsOutOfMemory,
+        /// 第 r 轮（从 0 数）起卡之前。
+        BeforeRound(usize),
+    }
+
+    /// 假卡上几轮判完一队块的结局：照块序写进库的块、每块判了几遍、最后一轮之后怎么走、留给下一趟的块、用到几张卡。
     struct BlocksJudgedOnFakeCards {
         stored: Vec<usize>,
         judged_times: Vec<usize>,
         after_the_last_round: AfterARoundOfTheCards,
+        blocks_left_for_the_next_run: Vec<usize>,
+        cards_used: usize,
     }
 
-    /// 几张假卡几轮判 `pending_blocks` 块（照崩溃放量产品路径第 ① 段的走法：一轮里卡一张一张起，从一队块里领、判、交去按块序写库；
-    /// 一轮收工之后照 [`after_a_round_of_the_cards`] 走）：第 `card` 张判一块占 `ticks_of_each_card[card]` 拍；头一轮只起前
-    /// `cards_holding_states_in_the_first_round` 张（装了状态的），再起的一轮起没报过 Out of Memory 的第一张（收尾那一步再起一张还有空的卡）；
-    /// 第 0 张判它领的第 `out_of_memory_at_block_of_card_0` 块（从 1 数）到末尾时报 Out of Memory，报了之后再给它块就 panic（报了就收工）。
+    /// 几张假卡几轮判 `pending_blocks` 块（照崩溃放量产品路径第 ① 段的走法：几轮之间带着的交 [`RoundsOfTheCards`]，一轮里卡一张一张起，
+    /// 从一队块里照叫停领、判、交去按块序写库；一轮收工之后照 [`RoundsOfTheCards::after_the_round`] 走）：第 `card` 张判一块占
+    /// `ticks_of_each_card[card]` 拍；头一轮只起前 `cards_holding_states_in_the_first_round` 张（装了状态的），再起的一轮起没报过 Out of Memory
+    /// 的第一张（收尾那一步再起一张还有空的卡）；第 0 张判它领的第 `out_of_memory_at_block_of_card_0` 块（从 1 数）到末尾时报 Out of Memory，
+    /// 报了之后再给它块就 panic（报了就收工）；调用方照 `caller_stop` 叫停。
     fn blocks_judged_on_fake_cards(
         pending_blocks: usize,
         ticks_of_each_card: &[u64],
         cards_holding_states_in_the_first_round: usize,
         out_of_memory_at_block_of_card_0: usize,
+        caller_stop: CallerStopOfTheFakeRun,
     ) -> BlocksJudgedOnFakeCards {
-        let stored_in_order: Mutex<BlocksStoredInOrder<usize>> =
-            Mutex::new(BlocksStoredInOrder::new());
+        let card_index = |card: usize| u32::try_from(card).expect("假卡没几张");
+        let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(pending_blocks);
         let stored = Mutex::new(Vec::new());
         let judged_times = Mutex::new(vec![0usize; pending_blocks]);
         let blocks_taken_by_card_0 = AtomicUsize::new(0);
-        let mut cards_short_of_memory: Vec<usize> = Vec::new();
-        let mut blocks_of_the_round: Vec<usize> = (0..pending_blocks).collect();
+        let blocks_judged = AtomicUsize::new(0);
+        let stop_is_requested = std::sync::atomic::AtomicBool::new(false);
         let mut round = 0;
         loop {
+            if caller_stop == CallerStopOfTheFakeRun::BeforeRound(round) {
+                stop_is_requested.store(true, Ordering::SeqCst);
+            }
+            let cards_not_short_of_memory: Vec<usize> = (0..ticks_of_each_card.len())
+                .filter(|card| !rounds.is_short_of_memory(card_index(*card)))
+                .collect();
             let cards_of_the_round: Vec<usize> = if round == 0 {
                 (0..cards_holding_states_in_the_first_round).collect()
             } else {
-                (0..ticks_of_each_card.len())
-                    .filter(|card| !cards_short_of_memory.contains(card))
-                    .take(1)
-                    .collect()
+                cards_not_short_of_memory.iter().copied().take(1).collect()
             };
-            let blocks = BlocksForTheCards::new(blocks_of_the_round.clone());
+            let blocks = rounds.take_the_blocks_of_the_round();
             let clock = CardsOnAClockOfTicks::started_by_the_one_starting_the_cards();
+            // 写库那一份只借给这一轮的卡：这一轮收工之后才交 `after_the_round`
+            let stored_in_order = Mutex::new(rounds.stored_in_order());
             let cards_left: Vec<(usize, CardLeftTheBlocks)> = std::thread::scope(|scope| {
                 let mut card_threads = Vec::new();
                 for card in cards_of_the_round.iter().copied() {
                     clock.a_card_joins();
                     let (blocks, clock, stored_in_order, stored, judged_times) =
                         (&blocks, &clock, &stored_in_order, &stored, &judged_times);
-                    let blocks_taken_by_card_0 = &blocks_taken_by_card_0;
+                    let (blocks_taken_by_card_0, blocks_judged, stop_is_requested) =
+                        (&blocks_taken_by_card_0, &blocks_judged, &stop_is_requested);
                     card_threads.push((
                         card,
                         scope.spawn(move || {
                             let _running = CardRunningOnTheClock(clock);
                             judge_blocks_on_one_card(
                                 blocks,
-                                &|| false,
+                                &|| {
+                                    if stop_is_requested.load(Ordering::SeqCst) {
+                                        StopOfTheCards::RequestedByTheCaller
+                                    } else {
+                                        StopOfTheCards::NotRequested
+                                    }
+                                },
                                 &mut |pending_index| {
                                     let taken = match card {
                                         0 => blocks_taken_by_card_0.fetch_add(1, Ordering::SeqCst) + 1,
@@ -4444,11 +5415,20 @@ mod tests {
                                     );
                                     clock.judge_for(card, ticks_of_each_card[card]);
                                     if card == 0 && taken == out_of_memory_at_block_of_card_0 {
+                                        if caller_stop == CallerStopOfTheFakeRun::WhenCard0RunsOutOfMemory {
+                                            stop_is_requested.store(true, Ordering::SeqCst);
+                                        }
                                         return BlockJudgedOnACard::OutOfMemory {
                                             reason: format!("显存不够：第 0 张卡判块 {pending_index}"),
                                         };
                                     }
                                     judged_times.lock().expect("判块的线程不 panic")[pending_index] += 1;
+                                    let judged_so_far = blocks_judged.fetch_add(1, Ordering::SeqCst) + 1;
+                                    if caller_stop
+                                        == CallerStopOfTheFakeRun::WhenTheBlocksJudgedReach(judged_so_far)
+                                    {
+                                        stop_is_requested.store(true, Ordering::SeqCst);
+                                    }
                                     BlockJudgedOnACard::Judged(pending_index)
                                 },
                                 &mut |pending_index, judged| {
@@ -4473,24 +5453,42 @@ mod tests {
                     .map(|(card, thread)| (card, thread.join().expect("假卡的线程不 panic")))
                     .collect()
             });
-            let short_this_round: Vec<usize> = cards_left
+            let cards_out_of_memory: Vec<(u32, String)> = cards_left
                 .iter()
-                .filter(|(_, left)| matches!(left, CardLeftTheBlocks::OutOfMemory { .. }))
-                .map(|(card, _)| *card)
+                .filter_map(|(card, left)| match left {
+                    CardLeftTheBlocks::OutOfMemory { reason } => {
+                        Some((card_index(*card), reason.clone()))
+                    }
+                    CardLeftTheBlocks::NothingLeftOrStopped
+                    | CardLeftTheBlocks::Refused { .. }
+                    | CardLeftTheBlocks::TheStoreIsGone => None,
+                })
                 .collect();
-            cards_short_of_memory.extend(&short_this_round);
-            blocks_of_the_round = blocks.blocks_not_taken();
-            let blocks_stored = stored_in_order
-                .lock()
-                .expect("写库不 panic")
-                .blocks_stored();
-            match after_a_round_of_the_cards(
-                pending_blocks - blocks_stored,
-                short_this_round.len(),
-                StopOfTheRun::NotRequested,
-                ticks_of_each_card.len() - cards_short_of_memory.len(),
+            let cards_finished: Vec<u32> = cards_left
+                .iter()
+                .filter(|(_, left)| *left == CardLeftTheBlocks::NothingLeftOrStopped)
+                .map(|(card, _)| card_index(*card))
+                .collect();
+            let stop = if stop_is_requested.load(Ordering::SeqCst) {
+                StopOfTheRun::Requested
+            } else {
+                StopOfTheRun::NotRequested
+            };
+            let cards_of_the_round_by_index: Vec<u32> = cards_not_short_of_memory
+                .iter()
+                .map(|card| card_index(*card))
+                .collect();
+            match rounds.after_the_round(
+                &blocks,
+                &cards_of_the_round_by_index,
+                cards_finished,
+                &cards_out_of_memory,
+                stop,
             ) {
-                AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory => round += 1,
+                AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory
+                | AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor => {
+                    round += 1;
+                }
                 after_the_last_round @ (AfterARoundOfTheCards::EveryBlockStored
                 | AfterARoundOfTheCards::StoppedOnRequest
                 | AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort) => {
@@ -4498,9 +5496,15 @@ mod tests {
                         stored: stored.into_inner().expect("写库不 panic"),
                         judged_times: judged_times.into_inner().expect("判块的线程不 panic"),
                         after_the_last_round,
+                        blocks_left_for_the_next_run: rounds.blocks_of_the_next_round(),
+                        cards_used: rounds.cards_used(),
                     };
                 }
             }
+            assert!(
+                round <= ticks_of_each_card.len(),
+                "每再起一轮至少少一张卡：轮数不超过卡数"
+            );
         }
     }
 
@@ -4518,6 +5522,7 @@ mod tests {
                     &ticks_of_each_card,
                     2,
                     out_of_memory_at_block_of_card_0,
+                    CallerStopOfTheFakeRun::Never,
                 );
                 let cell = format!(
                     "快慢 {ticks_of_each_card:?}、第 0 张卡第 {out_of_memory_at_block_of_card_0} 块报"
@@ -4545,7 +5550,7 @@ mod tests {
     /// 收尾那一步再起一轮、第二张判剩下的，20 块都照块序写进库、没有一块判两遍。改前第二张闲着、这一趟报本次未跑。
     #[test]
     fn with_every_state_on_the_first_card_the_blocks_it_leaves_are_judged_on_the_second_card() {
-        let judged = blocks_judged_on_fake_cards(20, &[1, 1], 1, 5);
+        let judged = blocks_judged_on_fake_cards(20, &[1, 1], 1, 5, CallerStopOfTheFakeRun::Never);
         assert_eq!(
             judged.after_the_last_round,
             AfterARoundOfTheCards::EveryBlockStored
@@ -4557,11 +5562,110 @@ mod tests {
     /// 第 10z 批第三块：只有一张卡，它在第 5 块报了：没有卡能判了，这一趟报本次未跑，前 4 块在库里。
     #[test]
     fn with_one_card_only_a_block_it_gives_back_leaves_the_run_not_run() {
-        let judged = blocks_judged_on_fake_cards(20, &[1], 1, 5);
+        let judged = blocks_judged_on_fake_cards(20, &[1], 1, 5, CallerStopOfTheFakeRun::Never);
         assert_eq!(
             judged.after_the_last_round,
             AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort
         );
         assert_eq!(judged.stored, vec![0, 1, 2, 3]);
     }
+
+    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K1-5，攻方 `attack_10zc_rounds.rs` 的 `stop-in-the-closing-round` 四格）：两张卡、5 块，
+    /// 第 0 张领块 0、判得慢（10 拍）、判到末尾报 Out of Memory 交回；第 1 张先判完块 1–4、收工（块 1–4 等着块 0）。叫停落在四个时刻：
+    /// 不叫停、头一轮第 1 张判完它最后一块那一刻、收尾再起的那一轮起卡之前、收尾那一轮判完块 0 那一刻。每一格都不 panic，判完的块照块序都入库、
+    /// 没有一块判两遍，第 1 张卡两轮各收工一次只算一张。改前「收尾那一轮起卡之前」那一格在轮末的断言上 panic（等着的 4 块）、
+    /// 「头一轮最后一块」那一格报本次未跑、4 块判完不入库。
+    #[test]
+    fn a_stop_around_the_closing_round_stores_every_block_judged_before_it() {
+        for caller_stop in [
+            CallerStopOfTheFakeRun::Never,
+            CallerStopOfTheFakeRun::WhenTheBlocksJudgedReach(4),
+            CallerStopOfTheFakeRun::BeforeRound(1),
+            CallerStopOfTheFakeRun::WhenTheBlocksJudgedReach(5),
+        ] {
+            let judged = blocks_judged_on_fake_cards(5, &[10, 1], 2, 1, caller_stop);
+            assert_eq!(
+                judged.after_the_last_round,
+                AfterARoundOfTheCards::EveryBlockStored,
+                "{caller_stop:?}"
+            );
+            assert_eq!(
+                judged.stored,
+                vec![0, 1, 2, 3, 4],
+                "{caller_stop:?}：判完的块都入库"
+            );
+            assert_eq!(
+                judged.judged_times,
+                vec![1; 5],
+                "{caller_stop:?}：每块判一遍"
+            );
+            assert_eq!(judged.cards_used, 1, "{caller_stop:?}：第 1 张卡只算一张");
+        }
+    }
+
+    /// 第 10z 批第五块（第 10z 批第三块代码轮正推 `rounds_sim4.rs` 的 `STOPSIM stop_from_round=Some(2)`）：两张卡、10 块，第 0 张很慢、
+    /// 判第一块时报 Out of Memory，第 1 张把别的 9 块都判完先收工；收尾再起的那一轮起卡之前叫停：那一轮照样判交回的块 0，10 块都入库、不 panic。
+    #[test]
+    fn a_stop_before_the_closing_round_still_judges_the_block_nine_judged_blocks_wait_for() {
+        let judged = blocks_judged_on_fake_cards(
+            10,
+            &[300, 1],
+            2,
+            1,
+            CallerStopOfTheFakeRun::BeforeRound(1),
+        );
+        assert_eq!(
+            judged.after_the_last_round,
+            AfterARoundOfTheCards::EveryBlockStored
+        );
+        assert_eq!(judged.stored, (0..10).collect::<Vec<usize>>());
+        assert_eq!(judged.judged_times, vec![1; 10]);
+    }
+
+    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K1-5「没判的留给下一趟」）：20 块，第 1 张卡判完块 1–3 时被叫停，第 0 张在块 0 上报了交回：
+    /// 收尾那一轮只判块 0，块 0–3 照块序入库，块 4–19 一块都不判、留给下一趟，这一趟照叫停收尾。
+    #[test]
+    fn a_stop_leaves_the_blocks_not_judged_to_the_next_run_after_storing_the_judged_ones() {
+        let judged = blocks_judged_on_fake_cards(
+            20,
+            &[10, 1],
+            2,
+            1,
+            CallerStopOfTheFakeRun::WhenTheBlocksJudgedReach(3),
+        );
+        assert_eq!(
+            judged.after_the_last_round,
+            AfterARoundOfTheCards::StoppedOnRequest
+        );
+        assert_eq!(judged.stored, vec![0, 1, 2, 3]);
+        assert_eq!(
+            judged.judged_times,
+            [vec![1; 4], vec![0; 16]].concat(),
+            "叫停之后只判判完的块在等的块 0"
+        );
+        assert_eq!(
+            judged.blocks_left_for_the_next_run,
+            (4..20).collect::<Vec<usize>>()
+        );
+    }
+
+    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K1-5）：同一轮里第 0 张卡在块 0 上报了 Out of Memory、交回，那一刻叫停；第 1 张还在判块 1：
+    /// 判完块 1 之后照样领交回的块 0（块 1 在等它），块 0、1 入库，这一趟照叫停收尾、不报本次未跑（与交回的块落到收尾那一轮时同一个结局）。
+    #[test]
+    fn a_stop_with_a_block_given_back_in_the_same_round_stores_the_judged_block_and_stops() {
+        let judged = blocks_judged_on_fake_cards(
+            4,
+            &[1, 2],
+            2,
+            1,
+            CallerStopOfTheFakeRun::WhenCard0RunsOutOfMemory,
+        );
+        assert_eq!(
+            judged.after_the_last_round,
+            AfterARoundOfTheCards::StoppedOnRequest
+        );
+        assert_eq!(judged.stored, vec![0, 1]);
+        assert_eq!(judged.judged_times, vec![1, 1, 0, 0]);
+        assert_eq!(judged.blocks_left_for_the_next_run, vec![2, 3]);
+    }
 }
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
index 7023980..38e8cf1 100644
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -48,8 +48,10 @@
 //!    第 ① 段在几张卡上判时，一张卡建判器或判一块时报了 Out of Memory，只停那一张：它手里那一块交回队里，别的卡照判
 //!    （第 10z 批，第 10y 批代码轮判决 K2-2 P2）；剩下的块别的卡判完了这一趟照常交回。别的卡都已收工、还有交回的块没判时，收尾那一步
 //!    再起一轮，只用没报过的卡（照这一刻能借的重新装状态、借草稿区）判剩下的块，判完的块照块序都入库（第 10z 批第三块，第 10z 批代码轮判决
-//!    K2-1 第 5 条：状态全装在第一张卡时也起第二张）；报过的卡之外没有卡能判才报本次未跑。卡线程那一圈与按块序写库、几轮怎么走在 harness 档
-//!    （`singlefs_harness::gpu_memory_ledger::judge_blocks_on_one_card`、`BlocksStoredInOrder`、`after_a_round_of_the_cards`），不要卡就测得到。
+//!    K2-1 第 5 条：状态全装在第一张卡时也起第二张）；报过的卡之外没有卡能判才报本次未跑。被叫停时卡照样领判完的块在等的那几块（交回的），
+//!    判完的块照块序都入库再照叫停收尾，没判的留给下一趟（第 10z 批第五块，第 10z 批第三块代码轮判决 K1-5）。卡线程那一圈与按块序写库、
+//!    几轮之间带着的与怎么走在 harness 档（`singlefs_harness::gpu_memory_ledger::judge_blocks_on_one_card`、`BlocksStoredInOrder`、
+//!    `RoundsOfTheCards`），不要卡就测得到。
 //!    单元校验和对拍时一块的几段在同一个设备上并发，认一段是不是显存不够之前等同块的几段都收完（第 10z 批第三块，攻方：一段因设备丢了先失败，
 //!    同块另一段范围里接到的 Out of Memory 还没记进设备时不判红）。
 //!    没起过卡池的进程照旧不进账（[`GpuCardsSource::DevicesStartedForThisRun`]：这一趟自己在点名的卡上起设备），调用方是这几种：
@@ -69,12 +71,6 @@ use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
 use singlefs_core::recovery::{recover, JournalPolicy};
 use singlefs_core::root_record::ROOT_MAGIC;
 #[cfg(feature = "gpu")]
-use singlefs_harness::gpu_memory_ledger::{
-    after_a_round_of_the_cards, judge_blocks_on_one_card, AfterARoundOfTheCards,
-    BlockHandedToTheStore, BlockJudgedOnACard, BlocksForTheCards, BlocksStoredInOrder,
-    CardLeftTheBlocks, StopOfTheRun,
-};
-#[cfg(feature = "gpu")]
 use singlefs_harness::gpu_memory_ledger::{
     decide_scratch_before_booking, gpu_not_run_line, mebibytes_of_buffers_held_at_once,
     most_states_whose_need_fits_the_capacity, reason_for_a_booking_without_room_now,
@@ -83,6 +79,12 @@ use singlefs_harness::gpu_memory_ledger::{
     ScratchBeforeBooking, ScratchBookingRefusalOutcome, ScratchStatesToBook,
     PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD,
 };
+#[cfg(feature = "gpu")]
+use singlefs_harness::gpu_memory_ledger::{
+    judge_blocks_on_one_card, AfterARoundOfTheCards, BlockHandedToTheStore, BlockJudgedOnACard,
+    BlocksForTheCards, BlocksStoredInOrder, CardLeftTheBlocks, RoundsOfTheCards, StopOfTheCards,
+    StopOfTheRun,
+};
 use singlefs_harness::gpu_memory_ledger::{WorkEndingTogether, WorksEndingTogetherOnADevice};
 use singlefs_harness::hexadecimal::hexadecimal_text;
 use singlefs_harness::memory_pool::{
@@ -1933,7 +1935,7 @@ pub struct GpuJudgeOutcome {
     pub judge_seconds: f64,
     /// 写库的挂钟（CPU，单线程，按块序）。
     pub seconds_storing: f64,
-    /// 用到的每张卡一行，按优先级。
+    /// 用到的卡每收工一次一行，按优先级（一张卡在收尾再起的那一轮里又收工一次就有两行；用到几张卡看 `cards_used`）。
     pub cards: Vec<GpuCardOutcome>,
 }
 
@@ -2106,16 +2108,24 @@ struct CardsAtWork<'work> {
     /// 每张卡草稿区那一笔借据的借据号（判器建好那一刻核账用）；不进账的是 `None`。
     scratch_lease_identifiers: &'work [Option<u64>],
     blocks: BlocksForTheCards,
-    /// 写库失败、有卡拒判之后置上：卡上的线程判完手里那一块就停。一张卡报 Out of Memory 不置（只停那一张，第 10z 批）。
+    /// 写库失败、有卡拒判之后置上：卡上的线程判完手里那一块就停、一块都不再领。一张卡报 Out of Memory 不置（只停那一张，第 10z 批）。
     stop: std::sync::atomic::AtomicBool,
-    /// 调用方要停（每领一块之前问一次）：不再领新的块，手里那一块判完照旧交去写库。
+    /// 调用方要停（每领一块之前问一次）：不再领新的块，只领判完的块在等的那几块（第 10z 批第五块），手里那一块判完照旧交去写库。
     stop_is_requested: &'work (dyn Fn() -> bool + Sync),
 }
 
 #[cfg(feature = "gpu")]
 impl CardsAtWork<'_> {
-    fn must_stop(&self) -> bool {
-        self.stop.load(std::sync::atomic::Ordering::SeqCst) || (self.stop_is_requested)()
+    /// 卡上的线程这一刻照什么停（[`judge_blocks_on_one_card`]、[`start_the_next_card_with_states`] 问）：判红了一块都不再领，
+    /// 调用方叫停了只领判完的块在等的，都没有照常领。
+    fn stop_of_the_cards(&self) -> StopOfTheCards {
+        if self.stop.load(std::sync::atomic::Ordering::SeqCst) {
+            StopOfTheCards::TheRunFailed
+        } else if (self.stop_is_requested)() {
+            StopOfTheCards::RequestedByTheCaller
+        } else {
+            StopOfTheCards::NotRequested
+        }
     }
 }
 
@@ -2169,9 +2179,10 @@ fn on_the_card_catching_out_of_memory<Outcome>(
 }
 
 /// 同 [`on_the_card_catching_out_of_memory`]，这段活与同一个设备上别的几段一起开（`ending` 是它的报到，单元校验和对拍时一块切成的几段）：
-/// 这条线程上的错误范围接到的 Out of Memory 记进设备之后报到，等那几段都报到了再认设备上记的
+/// 活交回那一刻记下设备丢没丢，这条线程上的错误范围接到的 Out of Memory 记进设备之后报到，等那几段都报到了再认设备上记的
 /// （`singlefs_harness::gpu_memory_ledger::DeviceConditionOfACard::end_the_work_and_explain_its_failure`，第 10z 批第三块：一段因设备丢了先失败、
-/// 同块另一段范围里接到的还没记进设备时，先认就认不出、判红）。
+/// 同块另一段范围里接到的还没记进设备时，先认就认不出、判红；第 10z 批第五块：设备因显存不够丢的，不在同一组也认得出；
+/// 交回那一刻设备还没丢的，之后别的段丢了设备不把它说成显存不够）。
 #[cfg(feature = "gpu")]
 fn on_the_card_catching_out_of_memory_ending_together<Outcome>(
     card: &crate::gpu_unit_checks::GpuCard,
@@ -2185,6 +2196,8 @@ fn on_the_card_catching_out_of_memory_ending_together<Outcome>(
         .device()
         .push_error_scope(wgpu::ErrorFilter::OutOfMemory);
     let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
+    // 活交回的这一刻设备丢没丢：之后等同组几段时别的段丢了设备，不把这段活真正的 panic 说成显存不够（第 10z 批第五块）
+    let device_when_the_work_returned = card.device_condition().device_when_the_work_returns();
     let out_of_memory_in_the_scope =
         pollster::block_on(out_of_memory_scope.pop()).map(|error| error.to_string());
     let out_of_memory_on_the_device = card
@@ -2192,6 +2205,7 @@ fn on_the_card_catching_out_of_memory_ending_together<Outcome>(
         .end_the_work_and_explain_its_failure(
             &work_on_the_card,
             out_of_memory_reports_before,
+            device_when_the_work_returned,
             out_of_memory_in_the_scope
                 .as_ref()
                 .map(|caught| format!("序号 {} 的卡{what_ran_out}报了 {caught}", card.index)),
@@ -2279,7 +2293,7 @@ fn judge_on_one_card_and_start_the_next<'scope, 'work>(
     };
     let card_left = judge_blocks_on_one_card(
         &work.blocks,
-        &|| work.must_stop(),
+        &|| work.stop_of_the_cards(),
         &mut |pending_index| {
             let (point_index, local) = &work.pending[pending_index];
             let point = &work.plan.points[*point_index];
@@ -2438,8 +2452,8 @@ fn scratch_step_before_booking(
     }
 }
 
-/// 第 `position` 张卡备好了（或建判器时报了 Out of Memory、不判）：后面还有装了状态的卡、队里还有没领的块、没被叫停，就起下一张的线程
-/// （卡一张一张备，不同时编内核）。
+/// 第 `position` 张卡备好了（或建判器时报了 Out of Memory、不判）：后面还有装了状态的卡、队里照此刻的叫停还有领得的块
+/// （[`CardsAtWork::stop_of_the_cards`]），就起下一张的线程（卡一张一张备，不同时编内核）。
 #[cfg(feature = "gpu")]
 fn start_the_next_card_with_states<'scope, 'work>(
     scope: &'scope std::thread::Scope<'scope, 'work>,
@@ -2450,7 +2464,7 @@ fn start_the_next_card_with_states<'scope, 'work>(
     let next_position =
         (position + 1..work.cards.len()).find(|candidate| work.states_placed[*candidate] > 0);
     if let Some(next_position) = next_position {
-        if work.blocks.any_left() && !work.must_stop() {
+        if work.blocks.any_left_to_take(work.stop_of_the_cards()) {
             let sender = sender.clone();
             scope.spawn(move || {
                 judge_on_one_card_and_start_the_next(scope, work, next_position, sender);
@@ -2619,7 +2633,9 @@ fn card_capacities_of_this_run(
 /// 判完的块按块序一块一块写库（后判完的先到就等前面的）：中途被停，写进库的块下一次不再判；
 /// 判定向量的编号按块序登记，与用了几张卡无关。配置开了 GPU 就一定由 GPU 判（用户 2026-09-29 定，不接受默认降级）：
 /// 建不出表（有清零写、写与落点对不上）、有状态判不了（表越过内核容量），都报错，不退回 CPU 判器；报错之前判完的块照旧在库里。
-/// 每领一块之前先问 `stop_is_requested`：要停就不再领，已经领的块判完、写进库再收尾（领的块按块序连着，库里不留空洞）。
+/// 每领一块之前先问 `stop_is_requested`：要停就不再领新的，已经领的块判完、写进库再收尾（库里不留空洞）；有卡报了 Out of Memory 交回的块、
+/// 判完的块在等它的，叫停了也照领照判（交回的块落到收尾再起的那一轮里也一样），判完的块照块序都入库、没判的留给下一趟（第 10z 批第五块，
+/// 第 10z 批第三块代码轮判决 K1-5）。
 ///
 /// # Errors
 /// 库读写失败；建不出表；没有一张卡装得下这条流的表与一个状态的草稿区；有状态 GPU 判不了；卡上 wgpu 报了 Out of Memory
@@ -2915,17 +2931,16 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
     }
     let state_count = tables.state_count;
     let tables = crate::crash_judge_dispatch::PackedJudgeTables::of(&tables);
-    // 几轮卡共用：上一轮判完、等着前面那一块的，下一轮那一块判完照块序一起写
-    let mut stored_in_order: BlocksStoredInOrder<GpuJudgedBlock> = BlocksStoredInOrder::new();
-    let mut blocks_of_the_round: Vec<usize> = (0..pending.len()).collect();
-    let mut cards_short_of_memory: BTreeSet<u32> = BTreeSet::new();
-    let mut first_out_of_memory: Option<String> = None;
+    // 几轮之间带着的（harness 档，不要卡就测得到）：按块序写库的那一份（上一轮判完、等着前面那一块的，下一轮那一块判完照块序一起写）、
+    // 下一轮的那一队块、报过 Out of Memory 的卡与头一句、收过工的卡
+    let mut rounds: RoundsOfTheCards<GpuJudgedBlock> = RoundsOfTheCards::new(pending.len());
     // 这一趟本机每张卡的优先级（头一轮那一份，后面几轮只少不多）：收尾时每张卡的账照它排
     let mut card_indices_by_priority: Vec<u32> = Vec::new();
     let started = Instant::now();
     // 每再起一轮至少少一张卡（报过 Out of Memory 的不再用），轮数不超过卡数
     loop {
-        let pending_states: u64 = blocks_of_the_round
+        let pending_states: u64 = rounds
+            .blocks_of_the_next_round()
             .iter()
             .map(|pending_index| {
                 let (_, local) = &pending[*pending_index];
@@ -2934,13 +2949,13 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
             .sum();
         let cards_of_the_round: Vec<std::sync::Arc<crate::gpu_unit_checks::GpuCard>> = cards
             .iter()
-            .filter(|card| !cards_short_of_memory.contains(&card.index))
+            .filter(|card| !rounds.is_short_of_memory(card.index))
             .cloned()
             .collect();
         let run_cards: Vec<CardCapacityOfThisRun> =
             card_capacities_of_this_run(cards_taken_from, &cards_of_the_round, &tables)?
                 .into_iter()
-                .filter(|card| !cards_short_of_memory.contains(&card.index))
+                .filter(|card| !rounds.is_short_of_memory(card.index))
                 .collect();
         if card_indices_by_priority.is_empty() {
             card_indices_by_priority = run_cards.iter().map(|card| card.index).collect();
@@ -3006,7 +3021,7 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
         }
         let Some(scratch) = scratch.filter(|scratch| !scratch.cards_for_the_judges.is_empty())
         else {
-            if let Some(reason) = first_out_of_memory {
+            if let Some(reason) = rounds.first_out_of_memory().map(str::to_string) {
                 // 报过 Out of Memory 的卡之外没有一张装得下：没有卡能判剩下的块了
                 println!(
                     "{}",
@@ -3027,7 +3042,7 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
             states_placed: &scratch.states_placed,
             scratch_booked_mebibytes: &scratch.scratch_booked_mebibytes,
             scratch_lease_identifiers: &scratch.scratch_lease_identifiers,
-            blocks: BlocksForTheCards::new(blocks_of_the_round.iter().copied()),
+            blocks: rounds.take_the_blocks_of_the_round(),
             stop: std::sync::atomic::AtomicBool::new(false),
             stop_is_requested,
         };
@@ -3046,7 +3061,7 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
                 &mut wall_clock_lines,
                 &clock,
                 pending.len(),
-                &mut stored_in_order,
+                rounds.stored_in_order(),
                 &mut |block| {
                     let storing_started = Instant::now();
                     let (point_index, local) = &pending[block.pending_index];
@@ -3073,53 +3088,36 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
                 },
                 &mut |line| eprintln!("{line}"),
             );
-            if stored_in_order.storing_error().is_none()
-                && received.refusal.is_none()
-                && received.cards_out_of_memory.is_empty()
-            {
-                // 没有卡报 Out of Memory：领走的块（序号连着、从这一轮最小的那一块起）每一块都写进了库；没被叫停时领走的就是全部
-                assert_eq!(
-                    stored_in_order.blocks_waiting(),
-                    0,
-                    "领走的块每一块都写进了库"
-                );
-                assert!(
-                    !work.blocks.any_left() || stop_is_requested(),
-                    "没人叫停，却有 {} 块没判",
-                    work.blocks.blocks_not_taken().len()
-                );
-            }
             received
         });
-        outcome.cards.extend(received.finished_cards);
-        if let Some(error) = stored_in_order.storing_error() {
+        if let Some(error) = rounds.stored_in_order().storing_error() {
             return Err(error.to_string());
         }
         if let Some(reason) = received.refusal {
             return Err(reason);
         }
-        for (index, reason) in &received.cards_out_of_memory {
-            cards_short_of_memory.insert(*index);
-            first_out_of_memory.get_or_insert_with(|| reason.clone());
-        }
-        blocks_of_the_round = work.blocks.blocks_not_taken();
-        let cards_not_short_of_memory = run_cards
+        let finished_card_indices: Vec<u32> = received
+            .finished_cards
             .iter()
-            .filter(|card| !cards_short_of_memory.contains(&card.index))
-            .count();
+            .map(|card| card.index)
+            .collect();
+        outcome.cards.extend(received.finished_cards);
         let stop = if stop_is_requested() {
             StopOfTheRun::Requested
         } else {
             StopOfTheRun::NotRequested
         };
-        match after_a_round_of_the_cards(
-            pending.len() - stored_in_order.blocks_stored(),
-            received.cards_out_of_memory.len(),
+        let run_card_indices: Vec<u32> = run_cards.iter().map(|card| card.index).collect();
+        // 收工之后的不变量（没有卡报 Out of Memory 时判完的块都写进了库、没叫停时块都领走了）也在这一步里判
+        match rounds.after_the_round(
+            &work.blocks,
+            &run_card_indices,
+            finished_card_indices,
+            &received.cards_out_of_memory,
             stop,
-            cards_not_short_of_memory,
         ) {
             AfterARoundOfTheCards::EveryBlockStored => {
-                if first_out_of_memory.is_some() {
+                if rounds.first_out_of_memory().is_some() {
                     eprintln!(
                         "CRASH_AMPLIFICATION_GPU_OUT_OF_MEMORY_ON_A_CARD 有卡报了 Out of Memory、只停那一张，别的卡判完了全部 {} 块",
                         pending.len()
@@ -3130,14 +3128,24 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
             AfterARoundOfTheCards::StoppedOnRequest => break,
             AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory => {
                 eprintln!(
-                    "CRASH_AMPLIFICATION_GPU_ANOTHER_ROUND 报了 Out of Memory 的卡 {cards_short_of_memory:?} 收工之后还有 {} 块没写进库，收尾再起一轮、用别的卡判剩下的 {} 块",
-                    pending.len() - stored_in_order.blocks_stored(),
-                    blocks_of_the_round.len()
+                    "CRASH_AMPLIFICATION_GPU_ANOTHER_ROUND 报了 Out of Memory 的卡 {:?} 收工之后还有 {} 块没写进库，收尾再起一轮、用别的卡判剩下的 {} 块",
+                    rounds.cards_short_of_memory(),
+                    pending.len() - rounds.blocks_stored(),
+                    rounds.blocks_of_the_next_round().len()
+                );
+            }
+            AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor => {
+                eprintln!(
+                    "CRASH_AMPLIFICATION_GPU_ANOTHER_ROUND_AFTER_THE_STOP 叫停了；报了 Out of Memory 的卡 {:?} 交回的块前面判完的 {} 块在等，再起一轮、用别的卡只判它们在等的那几块，判完照叫停收尾",
+                    rounds.cards_short_of_memory(),
+                    rounds.blocks_waiting()
                 );
             }
             AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort => {
-                let reason = first_out_of_memory
-                    .expect("这一轮有卡报了 Out of Memory 才会走到本次未跑这一支");
+                let reason = rounds
+                    .first_out_of_memory()
+                    .expect("这一轮有卡报了 Out of Memory 才会走到本次未跑这一支")
+                    .to_string();
                 println!(
                     "{}",
                     gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, &reason)
@@ -3151,7 +3159,8 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
             .iter()
             .position(|index| *index == card.index)
     });
-    outcome.cards_used = outcome.cards.len();
+    // 一张卡在收尾再起的那一轮里又收工一次，`outcome.cards` 里有它两行，用到的卡只算一张（第 10z 批第五块）
+    outcome.cards_used = rounds.cards_used();
     outcome.compile_seconds = outcome.cards.iter().map(|card| card.build_seconds).sum();
     outcome.judge_seconds = (started.elapsed().as_secs_f64() - outcome.compile_seconds).max(0.0);
     outcome.red_states = tally.red_states;
@@ -3634,6 +3643,68 @@ mod tests {
         );
     }
 
+    /// 第 10z 批第五块（第 10z 批第三块代码轮正推 K3 补充 2 第 2 条：第 D 条的接线）：一块切成的几段领的是同一组报到，先收完的那一段等到
+    /// 同一块别的段都报到了才交回（认它是不是显存不够之前，同块别的段范围里接到的都记进了设备）。不要卡、不碰库：判一段与写一块都用假的；
+    /// 段 0 一来就报到、等，别的段等段 0 报到之后过 100 毫秒再报到。每段各领一组时段 0 一报到就交回，那一刻别的段都还没报到。
+    #[test]
+    fn the_pieces_of_one_block_report_their_endings_in_one_group() {
+        let pending: Vec<(usize, Range<u64>)> = vec![(0, 0..3)];
+        let points = vec![NamesOfACrashPoint {
+            name: "一个点".to_string(),
+            path_number: "0".to_string(),
+        }];
+        let pieces_ended = std::sync::atomic::AtomicUsize::new(0);
+        let pieces_ended_when_the_first_piece_returned = std::sync::Mutex::new(None);
+        check_pending_blocks_in_pieces(
+            &pending,
+            &points,
+            3,
+            &|| false,
+            &|_point_index, local, ending| {
+                if local.start == 0 {
+                    pieces_ended.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
+                    ending.end_and_wait_for_the_others();
+                    *pieces_ended_when_the_first_piece_returned
+                        .lock()
+                        .expect("段 0 只有一条线程写") =
+                        Some(pieces_ended.load(std::sync::atomic::Ordering::SeqCst));
+                } else {
+                    let waiting_since = std::time::Instant::now();
+                    while pieces_ended.load(std::sync::atomic::Ordering::SeqCst) == 0 {
+                        assert!(
+                            waiting_since.elapsed() < std::time::Duration::from_secs(60),
+                            "段 0 一分钟都没报到"
+                        );
+                        std::thread::yield_now();
+                    }
+                    std::thread::sleep(std::time::Duration::from_millis(100));
+                    pieces_ended.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
+                    ending.end_and_wait_for_the_others();
+                }
+                JudgedBlock {
+                    items_of_each_state: vec![
+                        Vec::new();
+                        usize::try_from(local.end - local.start)
+                            .expect("一段装得进")
+                    ],
+                    local,
+                    red_texts: Vec::new(),
+                    gpu_cpu_disagreements: 0,
+                    gpu_memory_was_short: None,
+                }
+            },
+            &mut |_point_index, _local, _items_of_each_state, _violating| Ok(()),
+        )
+        .expect("假的写库不失败");
+        assert_eq!(
+            pieces_ended_when_the_first_piece_returned
+                .into_inner()
+                .expect("段 0 的线程不 panic"),
+            Some(3),
+            "段 0 等到同一块的三段都报到才交回"
+        );
+    }
+
     /// 第 10z 批（第 10y 批代码轮正推 K3 列的第 7 处）：池里一张卡此刻照这一刻能借的算带得了几个状态，空着时照空着时能借的算。不要卡。
     #[cfg(feature = "gpu")]
     #[test]
diff --git a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
index 593c165..4d61e24 100644
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -16,7 +16,7 @@ use singlefs_checker::{
 pub use singlefs_harness::gpu_memory_ledger::DeviceConditionOfACard;
 use singlefs_harness::gpu_memory_ledger::{
     card_memory_reader_of_one_report_per_reading, pci_bus_identifier_without_the_domain,
-    BusesWithDevicesStartedOutsideTheBooks, CardMemoryReader,
+    BusesWithDevicesStartedOutsideTheBooks, CardMemoryReader, CauseOfTheDeviceLoss,
     NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS,
 };
 use singlefs_harness::sha256::DIGEST_BYTES;
@@ -118,14 +118,28 @@ fn handle_an_uncaptured_wgpu_error(condition: &DeviceConditionOfACard, error: wg
     panic!("wgpu error: {error}\n")
 }
 
-/// 设备丢失的回调（装在每个设备上，[`start_the_card`]）：记进这个设备的情况，连同 wgpu 给的丢失原因与那一句（卡池下一次借这张卡时照它重起，
-/// 丢了之后撞上的 Out of Memory 照它认成显存不够）。
+/// 设备丢失的回调（装在每个设备上，[`start_the_card`]）：记进这个设备的情况，连同 wgpu 给的丢失原因与那一句、照那一句认出的缘由
+/// （[`cause_of_the_device_loss`]；卡池下一次借这张卡时照它重起，因显存不够丢的、或丢了之后撞上 Out of Memory 的，照它认成显存不够）。
 fn record_the_loss_of_the_device(
     condition: &DeviceConditionOfACard,
     reason: wgpu::DeviceLostReason,
     message: &str,
 ) {
-    condition.record_the_device_lost(format!("{reason:?}：{message}"));
+    condition.record_the_device_lost(
+        format!("{reason:?}：{message}"),
+        cause_of_the_device_loss(message),
+    );
+}
+
+/// wgpu 丢设备时给的那一句是哪一种：与它因显存不够丢设备时给的那一句（`wgpu::hal::DeviceError::OutOfMemory`，wgpu-core 把它当致命、
+/// 照它的这一句丢设备）相同的是显存不够（第 10z 批第五块，第 10z 批第三块代码轮判决 K1-4 攻方甲形）。wgpu 不另给丢失的缘由
+/// （回调的原因只有 `Unknown`、`Destroyed`），只能比那一句；那一句取自 wgpu 自己，不在这里另抄一份。
+fn cause_of_the_device_loss(message: &str) -> CauseOfTheDeviceLoss {
+    if message == wgpu::hal::DeviceError::OutOfMemory.to_string() {
+        CauseOfTheDeviceLoss::GpuMemoryShort
+    } else {
+        CauseOfTheDeviceLoss::Other
+    }
 }
 
 /// 一张卡起不来的两种。
@@ -1778,12 +1792,12 @@ mod tests {
         PciNvidiaDisplayControllers, PCI_DEVICES_SUBDIRECTORY_NAME,
     };
     use super::{
-        card_memory_reader_of_the_card_pool, failure_of_a_device_request,
+        card_memory_reader_of_the_card_pool, cause_of_the_device_loss, failure_of_a_device_request,
         handle_an_uncaptured_wgpu_error, has_devices_started_outside_the_card_pool_on_the_bus,
         record_the_loss_of_the_device, started_with_its_owner_recorded, CardStartFailure,
         DeviceConditionOfACard, DeviceOwnerOnTheCard,
     };
-    use singlefs_harness::gpu_memory_ledger::OccupiedByThisProcess;
+    use singlefs_harness::gpu_memory_ledger::{CauseOfTheDeviceLoss, OccupiedByThisProcess};
 
     /// 假的 sysfs 里 PCI 总线那一截的样子。
     enum FakePciBus<'devices> {
@@ -2039,6 +2053,43 @@ mod tests {
         );
     }
 
+    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K1-4 攻方甲形，wgpu-core 30.0.1 `src/device/resource.rs` 第 706 行、wgpu-hal 30.0.1
+    /// `src/lib.rs` 第 382 行）：wgpu 因显存不够丢设备时给的那一句认成显存不够，别的认成别的原因；回调照它记，之后一段因设备丢了而失败的活、
+    /// 不在同一组而这段活开始之后一条 Out of Memory 都没记过，也说成显存不够。不要卡。
+    #[test]
+    fn a_device_lost_for_lack_of_memory_explains_a_failure_on_it_as_gpu_memory_short() {
+        let out_of_memory = wgpu::hal::DeviceError::OutOfMemory.to_string();
+        assert_eq!(
+            cause_of_the_device_loss(&out_of_memory),
+            CauseOfTheDeviceLoss::GpuMemoryShort
+        );
+        for other in [
+            wgpu::hal::DeviceError::Lost.to_string(),
+            wgpu::hal::DeviceError::Unexpected.to_string(),
+            "驱动重置".to_string(),
+        ] {
+            assert_eq!(
+                cause_of_the_device_loss(&other),
+                CauseOfTheDeviceLoss::Other,
+                "{other}"
+            );
+        }
+        let condition = DeviceConditionOfACard::default();
+        let work = singlefs_harness::gpu_memory_ledger::WorkOnTheCardOfThisThread::begin();
+        let reports_before = condition.out_of_memory_reports();
+        record_the_loss_of_the_device(&condition, wgpu::DeviceLostReason::Unknown, &out_of_memory);
+        assert!(
+            condition
+                .out_of_memory_explaining_a_failure_of_the_work(
+                    work.identifier(),
+                    reports_before,
+                    condition.device_when_the_work_returns()
+                )
+                .is_some(),
+            "设备因显存不够丢了：说成显存不够"
+        );
+    }
+
     /// 第 10z 批第三块（第 10z 批代码轮正推 K3 第 1 条）：不走卡池的两个取卡入口起设备都照不归卡池起（起之前记下总线号），卡池起设备只有一处、
     /// 照归卡池起。扫这份源码（测试那一段之外），不要卡。
     #[test]
````

## 变异表追加（`mutations-append.tsv` 原样）

````tsv
第10z批第五块 GPU 取卡 K1-4 设备因显存不够丢了、这段活开始之后没记过 Out of Memory 就说不成显存不够（攻方甲形：不在同一组的活判红）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                    (CauseOfTheDeviceLoss::GpuMemoryShort, None) => {\n                        Some(format!("设备因显存不够丢了（{}）", loss.why))	                    (CauseOfTheDeviceLoss::GpuMemoryShort, None) => {\n                        None	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_work_failing_on_a_device_another_group_lost_for_lack_of_memory_is_told_out_of_memory	a_work_failing_on_a_device_another_group_lost_for_lack_of_memory_is_told_out_of_memory
第10z批第五块 GPU 取卡 K1-4 活交回那一刻设备没丢也记成丢了（攻方乙形：真 panic 等齐之后说成显存不够）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        if self.is_lost() {\n            DeviceWhenTheWorkReturned::Lost	        if true || self.is_lost() {\n            DeviceWhenTheWorkReturned::Lost	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_genuine_panic_while_the_device_was_there_is_not_told_out_of_memory_after_another_piece_runs_out	a_genuine_panic_while_the_device_was_there_is_not_told_out_of_memory_after_another_piece_runs_out
第10z批第五块 GPU 取卡 K1-4 交回那一刻设备没丢的活照设备丢了那一支认	crates/singlefs-harness/src/gpu_memory_ledger.rs	            DeviceWhenTheWorkReturned::NotLost => None,\n            DeviceWhenTheWorkReturned::Lost => {	            DeviceWhenTheWorkReturned::NotLost | DeviceWhenTheWorkReturned::Lost => {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_genuine_panic_while_the_device_was_there_is_not_told_out_of_memory_after_another_piece_runs_out	a_genuine_panic_while_the_device_was_there_is_not_told_out_of_memory_after_another_piece_runs_out
第10z批第五块 GPU 取卡 K3 范围里接到的挪到等完之后再记（攻方 record-the-caught-after-waiting）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        if let Some(caught) = caught_in_the_scope {\n            self.record_an_out_of_memory(caught);\n        }\n        ending.end_and_wait_for_the_others();	        ending.end_and_wait_for_the_others();\n        if let Some(caught) = caught_in_the_scope {\n            self.record_an_out_of_memory(caught);\n        }	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_piece_records_what_its_scope_caught_before_it_reports_its_ending	a_piece_records_what_its_scope_caught_before_it_reports_its_ending
第10z批第五块 GPU 取卡 K3 最后一段报到只叫醒一段（攻方 endings-notify-one：三段以上挂住）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            self.every_work_ended.notify_all();	            self.every_work_ended.notify_one();	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_last_piece_to_report_wakes_every_piece_waiting_for_it	the_last_piece_to_report_wakes_every_piece_waiting_for_it
第10z批第五块 崩溃放量 K3 卡线程先领块再问叫停（攻方 stop-checked-after-taking-a-block：领了不判、块丢了）	crates/singlefs-harness/src/gpu_memory_ledger.rs	    while let Some(pending_index) = blocks.take_a_block(stop_of_the_cards()) {	    while let Some(pending_index) = blocks.take_a_block(StopOfTheCards::NotRequested) {\n        if stop_of_the_cards() != StopOfTheCards::NotRequested {\n            break;\n        }	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_card_asked_to_stop_takes_only_the_blocks_judged_blocks_wait_for_and_leaves_the_rest	a_card_asked_to_stop_takes_only_the_blocks_judged_blocks_wait_for_and_leaves_the_rest
第10z批第五块 崩溃放量 K3 卡线程那一圈不问叫停（正推 M18）	crates/singlefs-harness/src/gpu_memory_ledger.rs	    while let Some(pending_index) = blocks.take_a_block(stop_of_the_cards()) {	    while let Some(pending_index) = blocks.take_a_block(StopOfTheCards::NotRequested) {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_card_asked_to_stop_takes_only_the_blocks_judged_blocks_wait_for_and_leaves_the_rest	a_card_asked_to_stop_takes_only_the_blocks_judged_blocks_wait_for_and_leaves_the_rest
第10z批第五块 崩溃放量 K1-5 叫停之后判完的块在等的那几块也不领（收尾那一轮里叫停判完的块不入库、轮末断言 panic）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                self.largest_judged.is_some_and(|largest| first < largest)	                self.largest_judged.is_some_and(|largest| first < largest && false)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_stop_before_the_closing_round_still_judges_the_block_nine_judged_blocks_wait_for	a_stop_before_the_closing_round_still_judges_the_block_nine_judged_blocks_wait_for
第10z批第五块 崩溃放量 K1-5 判完一块不记块号（叫停之后认不出交回的块有判完的块在等）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                blocks.record_a_judged_block(pending_index);\n		-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_card_asked_to_stop_takes_only_the_blocks_judged_blocks_wait_for_and_leaves_the_rest	a_card_asked_to_stop_takes_only_the_blocks_judged_blocks_wait_for_and_leaves_the_rest
第10z批第五块 崩溃放量 K1-5 下一轮的那一队不带判完的块号（收尾那一轮起卡之前叫停，交回的块没人判）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                largest_judged: queue.largest_judged,	                largest_judged: None,	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_stop_leaves_the_blocks_not_judged_to_the_next_run_after_storing_the_judged_ones	a_stop_leaves_the_blocks_not_judged_to_the_next_run_after_storing_the_judged_ones
第10z批第五块 崩溃放量 K1-5 叫停而判完的块在等交回的块时不再起一轮（报本次未跑、判完的块不入库）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        StopOfTheRun::Requested => {\n            AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor\n        }	        StopOfTheRun::Requested => AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort,	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_stop_around_the_closing_round_stores_every_block_judged_before_it	a_stop_around_the_closing_round_stores_every_block_judged_before_it
第10z批第五块 崩溃放量 K1-5 叫停、判完的块都入库而这一轮有卡报了时报本次未跑（结局随交回的先后变）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        StopOfTheRun::Requested if blocks_waiting == 0 => AfterARoundOfTheCards::StoppedOnRequest,	        StopOfTheRun::Requested if blocks_waiting == 0 => {\n            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort\n        }	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_stop_with_a_block_given_back_in_the_same_round_stores_the_judged_block_and_stops	a_stop_with_a_block_given_back_in_the_same_round_stores_the_judged_block_and_stops
第10z批第五块 崩溃放量 K1-5 一张卡两轮各收工一次算两张（攻方乙、正推第 4 点：cards_used 算两遍）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        self.cards_that_finished.extend(cards_finished);	        for index in cards_finished {\n            let finished_before = self.cards_that_finished.contains(&index);\n            self.cards_that_finished\n                .insert(if finished_before { index + 1_000 } else { index });\n        }	-p singlefs-harness --lib -- gpu_memory_ledger::tests::rounds_keep_the_cards_short_of_memory_out_and_count_a_card_finishing_twice_once	rounds_keep_the_cards_short_of_memory_out_and_count_a_card_finishing_twice_once
第10z批第五块 崩溃放量 K3 报过 Out of Memory 的卡下一轮照用（正推 K3 补充 2 第 1 条：第 2937 行的过滤）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        self.cards_short_of_memory.contains(&card_index)	        self.cards_short_of_memory.contains(&card_index) && false	-p singlefs-harness --lib -- gpu_memory_ledger::tests::rounds_keep_the_cards_short_of_memory_out_and_count_a_card_finishing_twice_once	rounds_keep_the_cards_short_of_memory_out_and_count_a_card_finishing_twice_once
第10z批第五块 崩溃放量 K3 下一轮重判全部块（正推 K3 补充 2 第 1 条：第 3105 行）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        self.blocks_of_the_next_round = blocks.left_for_another_round();	        self.blocks_of_the_next_round = BlocksForTheCards::new(0..self.pending_blocks);	-p singlefs-harness --lib -- gpu_memory_ledger::tests::rounds_keep_the_cards_short_of_memory_out_and_count_a_card_finishing_twice_once	rounds_keep_the_cards_short_of_memory_out_and_count_a_card_finishing_twice_once
第10z批第五块 显存账 K3 读数不进账（正推 M01）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            books_of_the_card.take_in_a_reading(*reading);	            let _ = reading;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_reading_after_the_card_took_a_returned_lease_back_lets_a_later_lease_count_once	a_reading_after_the_card_took_a_returned_lease_back_lets_a_later_lease_count_once
第10z批第五块 显存账 K3 设备丢了重起时驱动那一份仍记成起着（正推 M05）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        self.driver_share = DriverShareOnTheCard::DeviceLostNotStartedAgain;	        self.driver_share = DriverShareOnTheCard::DeviceStarted;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_refused_restart_asked_again_takes_no_further_driver_share_off_the_room	a_refused_restart_asked_again_takes_no_further_driver_share_off_the_room
第10z批第五块 显存账 K3 重起判差 1 MiB 也放行（正推 M10）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        if capacity_mebibytes < booked.leases_mebibytes {	        if capacity_mebibytes + 1 < booked.leases_mebibytes {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_restart_is_allowed_when_it_fits_exactly_and_refused_one_mebibyte_short	a_restart_is_allowed_when_it_fits_exactly_and_refused_one_mebibyte_short
第10z批第五块 显存账 K3 重起被拒之后借一笔把驱动那一份记回登记了、没起（正推 M13）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            DriverShareOnTheCard::RegisteredDeviceNotStarted\n            | DriverShareOnTheCard::DeviceStarted\n            | DriverShareOnTheCard::DeviceLostNotStartedAgain => {}\n        }\n    }	            DriverShareOnTheCard::RegisteredDeviceNotStarted | DriverShareOnTheCard::DeviceStarted => {}\n            DriverShareOnTheCard::DeviceLostNotStartedAgain => {\n                self.driver_share = DriverShareOnTheCard::RegisteredDeviceNotStarted;\n            }\n        }\n    }	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_booked_after_a_refused_restart_keeps_the_restart_judged_again	a_lease_booked_after_a_refused_restart_keeps_the_restart_judged_again
第10z批第五块 崩溃放量 K3 卡判不了一块也接着领（正推 M30）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            BlockJudgedOnACard::Refused { reason } => return CardLeftTheBlocks::Refused { reason },	            BlockJudgedOnACard::Refused { reason: _ } => {}	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_card_stops_taking_blocks_after_a_refusal_and_after_the_store_is_gone	a_card_stops_taking_blocks_after_a_refusal_and_after_the_store_is_gone
第10z批第五块 崩溃放量 K3 写库那一头走了卡也接着领（正推 M31）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                    BlockHandedToTheStore::TheStoreIsGone => {\n                        return CardLeftTheBlocks::TheStoreIsGone\n                    }	                    BlockHandedToTheStore::TheStoreIsGone => {}	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_card_stops_taking_blocks_after_a_refusal_and_after_the_store_is_gone	a_card_stops_taking_blocks_after_a_refusal_and_after_the_store_is_gone
第10z批第五块 崩溃放量 K3 判不了的那一块也交回队里（攻方 a-refused-block-is-given-back）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            BlockJudgedOnACard::Refused { reason } => return CardLeftTheBlocks::Refused { reason },	            BlockJudgedOnACard::Refused { reason } => {\n                blocks.give_back(pending_index);\n                return CardLeftTheBlocks::Refused { reason };\n            }	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_card_stops_taking_blocks_after_a_refusal_and_after_the_store_is_gone	a_card_stops_taking_blocks_after_a_refusal_and_after_the_store_is_gone
第10z批第五块 GPU 取卡 K1-4 丢失回调不认 wgpu 因显存不够丢设备的那一句（攻方甲形）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	    if message == wgpu::hal::DeviceError::OutOfMemory.to_string() {	    if false && message == wgpu::hal::DeviceError::OutOfMemory.to_string() {	-p singlefs-checker-tier --features verdict-store,gpu --lib -- gpu_unit_checks::tests::a_device_lost_for_lack_of_memory_explains_a_failure_on_it_as_gpu_memory_short	a_device_lost_for_lack_of_memory_explains_a_failure_on_it_as_gpu_memory_short
第10z批第五块 崩溃放量 K3 一块的几段各领一组报到（正推 K3 补充 2 第 2 条：第 D 条的接线，等于不等）	crates/singlefs-checker-tier/src/crash_amplification.rs	            pieces.iter().map(|_| ending_together.join()).collect();	            pieces\n                .iter()\n                .map(|_| Box::leak(Box::new(WorksEndingTogetherOnADevice::new())).join())\n                .collect();	-p singlefs-checker-tier --features verdict-store --lib -- crash_amplification::tests::the_pieces_of_one_block_report_their_endings_in_one_group	the_pieces_of_one_block_report_their_endings_in_one_group
````

## 变异表替换（`mutations-replacements.tsv` 原样）

````tsv
崩溃放量 卡上不理会调用方的叫停	crates/singlefs-checker-tier/src/crash_amplification.rs	        &|| work.stop_of_the_cards(),	        &|| if work.stop.load(Ordering::SeqCst) { StopOfTheCards::TheRunFailed } else { StopOfTheCards::NotRequested },	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest	the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest
第10y批 GPU 取卡 K1-3 设备因 Out of Memory 丢了也说不成显存不够	crates/singlefs-harness/src/gpu_memory_ledger.rs	                match (loss.cause, reports_since.last()) {	                match (loss.cause, None::<&OutOfMemoryReport>) {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_explains_a_failure_only_with_an_out_of_memory_since_the_work_began	a_lost_device_explains_a_failure_only_with_an_out_of_memory_since_the_work_began
第10z批 崩溃放量 K2-2 P2 一张卡交回的块不再给别的卡领	crates/singlefs-harness/src/gpu_memory_ledger.rs	        let is_newly_given_back = self.queue_locked().blocks_not_taken.insert(pending_index);	        let is_newly_given_back = !self.queue_locked().blocks_not_taken.contains(&pending_index);	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_block_given_back_by_a_card_out_of_memory_goes_to_the_next_card_that_takes_one	a_block_given_back_by_a_card_out_of_memory_goes_to_the_next_card_that_takes_one
第10z批第三块 GPU 取卡 K1-3 一块的几段先认后等（设备丢了先失败的那一段认不出同块另一段范围里接到的 Out of Memory，攻方）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        ending.end_and_wait_for_the_others();\n        self.out_of_memory_explaining_a_failure_of_the_work(\n            work.identifier(),\n            reports_before,\n            device_when_the_work_returned,\n        )	        let explained = self.out_of_memory_explaining_a_failure_of_the_work(\n            work.identifier(),\n            reports_before,\n            device_when_the_work_returned,\n        );\n        ending.end_and_wait_for_the_others();\n        explained	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_piece_failing_on_a_lost_device_is_told_after_another_piece_records_its_out_of_memory	a_piece_failing_on_a_lost_device_is_told_after_another_piece_records_its_out_of_memory
第10z批第三块 GPU 取卡 K3 设备丢失的回调不记丢失（正推 K3 第 3 条）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	    condition.record_the_device_lost(\n        format!("{reason:?}：{message}"),\n        cause_of_the_device_loss(message),\n    );	    let _ = (condition, reason, message);	-p singlefs-checker-tier --features verdict-store,gpu --lib -- gpu_unit_checks::tests::the_device_lost_callback_records_the_device_as_lost_with_the_reason_and_the_message	the_device_lost_callback_records_the_device_as_lost_with_the_reason_and_the_message
````
