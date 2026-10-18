# m3-b44-code-r1 被判的改动（2026-10-15）

`git diff HEAD -- crates/singlefs-core crates/singlefs-format`，HEAD c1e9b324，25 个文件 +1268/−421。

```diff
diff --git a/crates/singlefs-core/src/address.rs b/crates/singlefs-core/src/address.rs
index 1dff9d63..a06907fd 100644
--- a/crates/singlefs-core/src/address.rs
+++ b/crates/singlefs-core/src/address.rs
@@ -85,11 +85,6 @@ impl SlotNumber {
     }
 }
 
-impl JournalSequenceNumber {
-    /// 计数器只有 6 字节宽；超过它的值在盘上写不下。
-    pub const COUNTER_LIMIT: u64 = 1 << 48;
-}
-
 #[cfg(test)]
 mod tests {
     use super::*;
@@ -119,6 +114,5 @@ mod tests {
             counter: 1,
         };
         assert!(earlier_instance < later_instance);
-        assert_eq!(JournalSequenceNumber::COUNTER_LIMIT, 281_474_976_710_656);
     }
 }
diff --git a/crates/singlefs-core/src/admission.rs b/crates/singlefs-core/src/admission.rs
index f7cb9b32..4f019fcf 100644
--- a/crates/singlefs-core/src/admission.rs
+++ b/crates/singlefs-core/src/admission.rs
@@ -292,7 +292,7 @@ pub const TOMBSTONE_PREPAY_BYTES_PER_LIVE_OBJECT_ON_ONE_REPLICA: u64 =
     DATA_UNIT_BYTES.div_ceil(TOMBSTONE_CONTAINER_RECORDS);
 
 /// 墓碑预付里全池常驻那一份的固定部分（每份副本、以 16 KiB 槽计）：一个开放墓碑容器（32 KiB 码 3，2 槽）加墓碑树根（1 槽）。
-/// 式子是 (2 + H + 1) × 16384，H 另算（[`committed_reservation_for_tombstones`]）。
+/// 式子是 (2 + H + 1) × 16384，H 另算（[`committed_reservation_for_tombstones_and_retirement_records`]）。
 pub const TOMBSTONE_PREPAY_OPEN_CONTAINER_SLOTS_ON_ONE_REPLICA: u64 = DATA_UNIT_BYTES / SLOT_BYTES;
 pub const TOMBSTONE_PREPAY_ROOT_SLOTS_ON_ONE_REPLICA: u64 = 1;
 
@@ -669,8 +669,10 @@ impl AdmissionReading {
     /// D28（挂载期承诺量） 已定项 2 要的是「已发布统计量 − 在飞已批准」。这几个计数就是每次发布写进记账行的那几个数
     /// （装记账行读的是同一组），发布与发布之间与最近一次发布的记账行逐项相等；在飞已批准第一版恒为零——一次发布在同一个调用里
     /// 准入、分配、落盘，调用与调用之间没有在飞的东西（在飞 overlay 没有实现，C87（准入读的合成值无定义））。
-    /// ⚠️ 两处例外计数先于记账行变：挂载重建时按回收门槛回收、按影子账隔离之后到写行那次发布之前；抬 F 回收的槽扣住到 F 生效之前
+    /// ⚠️ 有记账树的版本上只有两处例外计数先于记账行变：挂载重建时按回收门槛回收、按影子账隔离之后到写行那次发布之前；抬 F 回收的槽扣住到 F 生效之前
     /// （计数上已回到空闲、分配器却不发它们，式子里没有一项装它们）。那两段里准入读哪一个，条款没写。
+    /// 树表 0 条的那一版没有记账树，上面没有记账行可比：那一版上的零单元发布（`mount::publish_empty_after`、`mount::establish_instance`）
+    /// 与写行发布（`transaction::prepare_the_row_publish_on_a_version_without_file`）都在根落盘之后记根、按回收门槛回收，计数变而不写记账行。
     #[must_use]
     pub fn of_allocator(
         allocator: &PoolAllocator,
@@ -898,7 +900,7 @@ pub fn units_of_a_publish_to_land(
 
 /// 「这次的单元落得下」逐设备合取（C545（空间准入罩不住分裂与聚簇段层）：D3（空间分配） 已定项 5「字节总量够」与「聚簇段里有合法空槽」
 /// 之间没有条款连起来，用户 2026-09-27 定准入先拒）：每块盘都要两样都够才放行——
-/// ① 数据单元：段外成对的空槽 ≥ 这次的数据单元个数（数据单元在这次的角色里排最前、先取落点，`transaction::PublishShape::rewritten_roles`）；
+/// ① 数据单元：段外成对的空槽 ≥ 这次的数据单元个数（数据单元在这次的角色里排最前、先取落点，`transaction::PublishPlan::resolve` 排角色那一段：`rewritten_roles` 从写的文件内容的数据单元起）；
 /// ② 提交内生块：没挡的槽减去数据单元取走的那几对 ≥ 这次提交内生块至多要的槽。
 ///
 /// ② 只数槽：两槽的 inode 叶容器要成对、各盘的落点要同槽（开段各盘一致、回落各盘同槽），这两样今天的落点规则里有、这一判不判——
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 1329943e..27732325 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -8,7 +8,8 @@
 //! 落点按设备取（D3（空间分配） 已定项 8 第 1 条「在每一块被选中的设备上各自取」）：每块盘按自己的空闲图各答一个，各盘一致才分配；
 //! 有的盘答不出、或各盘答得不同，在动任何状态之前拒绝（`PlacementRefusal`）——`Placement` 两盘同槽，各盘不同槽第一版不支持。
 //! 记账（D5（快照 / 空间记账机制） 已定项 7 / 已定项 8）：已分配 = 落点之和、容量 = 单元区、runs = 空闲槽的连续段数（逐设备）、
-//! 全空聚簇段数逐设备——全部在分配那一刻增量维护，运行时不扫盘（`.claude/rules/fs-design.md` 第一格）。
+//! 逐段的已用槽数——这几样在分配那一刻增量维护，运行时不扫盘（`.claude/rules/fs-design.md` 第一格）。全空聚簇段数逐设备不是增量维护的：
+//! `DeviceFreeMap::empty_segments` 每次被读都把内存里逐段的已用槽数数一遍（不读盘，代价随段数长）。
 
 use singlefs_format::{
     ALLOCATION_RECORD_BYTES, CLUSTER_SEGMENT_SLOTS, JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES,
@@ -65,11 +66,6 @@ impl AllocationRecord {
         writer.assert_position(20, "分配记录");
         writer.into_bytes()
     }
-    /// key = (设备 4, 槽号 6)。
-    #[must_use]
-    pub fn key_bytes(&self) -> Vec<u8> {
-        self.to_bytes()[..10].to_vec()
-    }
     /// 按字段比：设备身份，再槽号（D8（核心索引结构） 已定项 11）。
     #[must_use]
     pub fn sort_key(&self) -> (u32, u64) {
@@ -1588,7 +1584,7 @@ pub struct PoolAllocator {
         Option<AllocationRecordTreeOfTheVersionWithoutFile>,
     /// 复用窗口，只供测试的开关（`ReuseWindow`）。只住内存：重开之后按产品路径起步（`rebuild_from_records` 走 `new`）。
     reuse_window: ReuseWindow,
-    /// 复用窗口置 0 时在 `release` 里当场回收掉的落点数（逐盘各算一条，与 `reclaim_released_up_to` 的返回值口径不同）：
+    /// 复用窗口置 0 时在 `release` 里当场回收掉的落点数（两盘同槽的一个落点算一条，就是 `reclaim_released_up_to` 交回的条数）：
     /// 开关走没走到，读这个数就看得出（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。
     placements_reclaimed_on_release_by_the_forced_zero_reuse_window: u64,
     /// 这个分配器实例上提交内生块走回落（`CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot`，D3（空间分配） 已定项 8 ②）的次数，
@@ -3147,7 +3143,7 @@ mod tests {
     }
 
     /// 新落点罩住一条没回收的记录（已释放、还在 defer 窗口里）：位图与记录分叉了，在记录这一层就断言，不等到位图的断言。
-    /// 不用 `should_panic`：门禁 59 号按「test 名 ... FAILED」认红，`should_panic` 那一行多一段「- should panic」认不出。
+    /// 用 `catch_unwind` 接住、再按消息断言：位图那一层的断言同样会 panic，要钉住拦下它的是记录这一层那一条。
     #[test]
     fn placement_over_record_that_was_not_reclaimed_is_asserted_on_the_records() {
         let mut pool = rebuilt_pool(records_on_both_devices(&[
diff --git a/crates/singlefs-core/src/block_device.rs b/crates/singlefs-core/src/block_device.rs
index cde55803..fcd461cf 100644
--- a/crates/singlefs-core/src/block_device.rs
+++ b/crates/singlefs-core/src/block_device.rs
@@ -1,9 +1,10 @@
-//! 块设备抽象：读、写、屏障、FUA 写、探测 `physical_block_size`、写零。
+//! 块设备抽象：四个动作（读、写——普通写或 FUA 写、写零、屏障），外加三问（探测 `physical_block_size`、设备字节数、
+//! journal 环里可能有字节的记录槽）。
 //!
 //! 第一版两个后端（D17（实现分层与第三方管道） 已定项 5）：文件后端（测试镜像）与 `O_DIRECT` 后端（虚机里的真块设备）。
 //! 两个后端的持久语义相同：屏障 = `sync_data`（块设备上是一次 FLUSH），FUA 写 = 写完立刻 `sync_data`。
 //!
-//! 「写零」是第六个动作（用户 2026-09-19 定案，`records/2026-09-19-里程碑二遗留收拢.md` 第五之二节第 4 行：
+//! 「写零」这个动作是后加的（用户 2026-09-19 定案，`records/2026-09-19-里程碑二遗留收拢.md` 第五之二节第 4 行：
 //! 问「块设备抽象加不加写零」答「加」）：一次调用覆盖 `[offset, offset + length)` 一整段，真设备上对应 WRITE ZEROES。
 //! **一次调用就是一个动作**——底下拆成几次 I/O 由后端自己定（[`ZERO_FILL_CHUNK_BYTES`]），上层（录制流、段序列）看到的是一步。
 //!
@@ -111,7 +112,9 @@ impl std::error::Error for BlockDeviceError {}
 /// 比它大只再快 1–2 ms 而暂存区从 4 MiB 长到 64 MiB（`O_DIRECT` 那条路每次调用要一块对齐暂存区）。
 pub const ZERO_FILL_CHUNK_BYTES: u64 = 4 * 1024 * 1024;
 
-/// 六个动作，只有这六个。介质是什么这里看不见。
+/// 七个方法：四个动作（[`BlockDevice::read_at`]、[`BlockDevice::write_at`]——FUA 是它的 [`WriteDurability`] 参数、不是另一个方法——、
+/// [`BlockDevice::write_zeroes_at`]、[`BlockDevice::barrier`]），三个只答问题、不碰介质的（物理块大小、设备字节数、journal 记录槽的提示）。
+/// 介质是什么这里看不见。
 pub trait BlockDevice {
     fn read_at(
         &self,
@@ -360,8 +363,9 @@ fn zero_fill_chunk_length(length: u64) -> usize {
 /// 两个后端共用的拆块循环：`zeros` 是后端备好的全 0 暂存区（它的长度就是块大小），
 /// 从 `offset` 起按块顺序交给 `write_chunk`，最后一块按剩余长度截短。
 ///
-/// 长度按物理块对齐由调用方先核过（[`check_aligned_and_in_range`]），而块大小 4 MiB 是 512 与 4096 的整数倍，
-/// 所以每一块都还是对齐的。
+/// 长度按物理块对齐由调用方先核过（[`check_aligned_and_in_range`]）；每一块的起点都还对齐，前提是物理块大小整除块大小 4 MiB
+/// （512、4096 都整除）。这个前提这里不判，物理块大小的来处也都不判它：[`PhysicalBlockSizeSource::Declared`] 收调用方给的任何值，
+/// sysfs 读到的值也不核；不整除时第二块起的起点不对齐。
 fn write_zeroes_in_chunks<WriteChunk>(
     offset: DeviceOffsetInBytes,
     length: u64,
diff --git a/crates/singlefs-core/src/code_two_tree.rs b/crates/singlefs-core/src/code_two_tree.rs
index 941fc9da..56749732 100644
--- a/crates/singlefs-core/src/code_two_tree.rs
+++ b/crates/singlefs-core/src/code_two_tree.rs
@@ -309,20 +309,6 @@ impl CodeTwoTreePlan {
             .map(|(node, _)| node.position)
             .collect()
     }
-
-    /// 这个位置上的节点从哪来。
-    ///
-    /// # Panics
-    /// 这个位置不在这一版里：调用方从同一份计划里取的位置。
-    #[must_use]
-    pub fn origin_of(&self, position: CodeTwoTreeNodePosition) -> CodeTwoTreeNodeOrigin {
-        let index = self
-            .shape
-            .nodes()
-            .binary_search_by_key(&position, |node| node.position)
-            .expect("位置取自同一份计划");
-        self.origins[index]
-    }
 }
 
 /// 这次之后的树算不出来，在任何落盘动作之前交回。
@@ -1373,11 +1359,20 @@ mod tests {
             key(5),
             "最左分隔 key 压低到新插的 key"
         );
+        let right_leaf_index_in_the_plan = after
+            .shape
+            .nodes()
+            .iter()
+            .position(|node| {
+                node.position
+                    == CodeTwoTreeNodePosition {
+                        level: 0,
+                        index_in_level: 1,
+                    }
+            })
+            .expect("右叶在这一版里");
         assert_eq!(
-            after.origin_of(CodeTwoTreeNodePosition {
-                level: 0,
-                index_in_level: 1
-            }),
+            after.origins[right_leaf_index_in_the_plan],
             CodeTwoTreeNodeOrigin::CarriedFrom(CodeTwoTreeNodePosition {
                 level: 0,
                 index_in_level: 1
@@ -1444,23 +1439,29 @@ mod tests {
 
     #[test]
     fn the_level_byte_bounds_the_height() {
-        assert_eq!(u8::MAX.checked_add(1), None);
         let mut root = PlanningNode::Internal {
             level: u8::MAX,
             children: vec![
                 PlanningChild {
                     separator_key: key(1),
-                    node: PlanningNode::empty_leaf(),
+                    node: PlanningNode::Leaf {
+                        keys: vec![key(1)],
+                        carried_from: None,
+                    },
                 },
                 PlanningChild {
                     separator_key: key(2),
-                    node: PlanningNode::empty_leaf(),
+                    node: PlanningNode::Leaf {
+                        keys: vec![key(2)],
+                        carried_from: None,
+                    },
                 },
             ],
             carried_from: None,
         };
-        // 根的层级是 255、孩子是空叶（只为造出这一格，层级不连贯）：插一把 key 进孩子不分裂，根不切；
-        // 把内部容量压到 1 让根装不下，根分裂要长出第 256 层 ⇒ 拒绝。
+        // 根的层级是 255、孩子是各装一把 key 的叶（只为造出这一格，层级不连贯）：插一把 key 进孩子不分裂，根不切；
+        // 把内部容量压到 1 让根装不下，根分裂要长出第 256 层 ⇒ 拒绝。孩子不是空叶，根的拒绝被放行时切开照常走完、
+        // 交回 `Ok`，红在下面这条断言上，不红在切开那一步取左半最小 key 的 `expect` 上。
         let refused = insert_at_the_root(&mut root, key(3), capacity(4, 1));
         assert_eq!(refused, Err(CodeTwoTreeRefusal::HeightBeyondTheLevelField));
     }
diff --git a/crates/singlefs-core/src/container_tree.rs b/crates/singlefs-core/src/container_tree.rs
index 3f2922c9..05031134 100644
--- a/crates/singlefs-core/src/container_tree.rs
+++ b/crates/singlefs-core/src/container_tree.rs
@@ -272,7 +272,7 @@ impl ContainerTreeShape {
     }
 
     /// 这几片容器（叶序）的祖先的并，按 bump 次序（层级升序、同层从左到右，根最末）：这几片被重写时 COW 的就是它们（叶 + 全部祖先）。
-    /// 只有最右那一片时就是 [`Self::rightmost_path`]。没有容器时空。
+    /// 只有最右那一片时就是最右那一条路径（每一层最右那一个节点）。没有容器时空。
     ///
     /// # Panics
     /// 某个下标不小于容器数：调用方只拿这一版里的容器问。
@@ -313,19 +313,6 @@ impl ContainerTreeShape {
         ancestors.into_iter().collect()
     }
 
-    /// 最右那一条路径上的节点（每一层最右那一个），按 bump 次序（层级 1 先、根最末）：最右那片容器被重写时 COW 的祖先就是它们。
-    #[must_use]
-    pub fn rightmost_path(&self) -> Vec<ContainerTreeNodePosition> {
-        self.children_of_each_node_at_each_level
-            .iter()
-            .enumerate()
-            .map(|(level_index, nodes)| ContainerTreeNodePosition {
-                level: u8::try_from(level_index + 1).expect("层数在构造时判过装得进一字节"),
-                index_in_level: u32::try_from(nodes.len() - 1).expect("同层节点数装得进 u32"),
-            })
-            .collect()
-    }
-
     /// 最右端新开一片容器之后的形状（末尾分裂、根满长高，D8（核心索引结构） 已定项 6）：最右那个层级 1 节点装得下（孩子少于
     /// `entries_per_node`）就挂进去；装满了就在这一层最右端新开一个只有它一个孩子的节点，再往上一层给最右那个节点添这个新孩子，
     /// 一层层往上；最高那一层（根）也满了就再长一层，新根两个孩子（旧根、新开的右半）。零根时第一片容器开一个层级 1 的根。
@@ -467,12 +454,23 @@ impl<Record> ContainerTreeAfterThisPublish<Record> {
     }
 }
 
+/// 要开一片新容器时下一个容器号装不下：最右那片的号（盘上读来的 8 字节）已是 `u64::MAX`，加一越过顶（C841，用户 2026-10-15 定
+/// 「写路径报类型化错误」：开新容器那一次发布报这个错、不 panic，读盘不变）。容器号从 1 起逐片加一，健康的树走不到顶，
+/// 坏镜像、外来镜像才有。`tree` 是要开新容器的那棵树（墓碑树、谱系树或退役树），`rightmost_container` 是最右那片的号。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct NextContainerNumberPastTheTopOfItsRange {
+    pub tree: TreeIdentifier,
+    pub rightmost_container: u64,
+}
+
 /// 往容器树的开放容器（最右那一片）追加一条记录：[`append_records_to_the_open_container`] 只给一条的那一格。
+///
+/// # Errors
+/// 同 [`append_records_to_the_open_container`]。
 #[allow(
     clippy::too_many_arguments,
     reason = "参数各是一样东西：上一版的容器与形状、这条记录、txg、树、记录类型、一片装几条、一个节点装几条"
 )]
-#[must_use]
 pub fn append_record_to_the_open_container<Record: Clone>(
     containers_before_this_publish: &[RecordContainer<Record>],
     shape_before_this_publish: &ContainerTreeShape,
@@ -482,7 +480,7 @@ pub fn append_record_to_the_open_container<Record: Clone>(
     record_type: u16,
     records_per_container: usize,
     node_entries: ContainerTreeNodeEntries,
-) -> ContainerTreeAfterThisPublish<Record> {
+) -> Result<ContainerTreeAfterThisPublish<Record>, NextContainerNumberPastTheTopOfItsRange> {
     append_records_to_the_open_container(
         containers_before_this_publish,
         shape_before_this_publish,
@@ -500,6 +498,10 @@ pub fn append_record_to_the_open_container<Record: Clone>(
 /// 打包记录类型 = `record_type`，形状照 [`ContainerTreeShape::after_a_container_is_opened_at_the_right_end`] 往上挂（根满长高）。
 /// 这次追加过的每一片进重写清单（叶序升序、各一次）；别的容器一个字节都不动。`records` 为空时整棵照抄（[`ContainerTreeAfterThisPublish::carried`]）。
 ///
+/// # Errors
+/// 要开新容器而最右那片的号已是 `u64::MAX` ⇒ [`NextContainerNumberPastTheTopOfItsRange`]（C841）：在开那一片之前交回，
+/// 只算、不写，调用方照样一个字节都没写。最右那片还装得下时照常追加，不看它的号。
+///
 /// # Panics
 /// `shape_before_this_publish` 里的容器数与 `containers_before_this_publish` 不等：两样出自同一版。`records_per_container` 为 0：
 /// 三棵树的容量开关的构造函数都判过至少 1。
@@ -507,7 +509,6 @@ pub fn append_record_to_the_open_container<Record: Clone>(
     clippy::too_many_arguments,
     reason = "参数各是一样东西：上一版的容器与形状、这几条记录、txg、树、记录类型、一片装几条、一个节点装几条"
 )]
-#[must_use]
 pub fn append_records_to_the_open_container<Record: Clone>(
     containers_before_this_publish: &[RecordContainer<Record>],
     shape_before_this_publish: &ContainerTreeShape,
@@ -517,7 +518,7 @@ pub fn append_records_to_the_open_container<Record: Clone>(
     record_type: u16,
     records_per_container: usize,
     node_entries: ContainerTreeNodeEntries,
-) -> ContainerTreeAfterThisPublish<Record> {
+) -> Result<ContainerTreeAfterThisPublish<Record>, NextContainerNumberPastTheTopOfItsRange> {
     assert_eq!(
         shape_before_this_publish.containers(),
         containers_before_this_publish.len(),
@@ -543,9 +544,19 @@ pub fn append_records_to_the_open_container<Record: Clone>(
                 .push(record.clone());
         } else {
             // 容器号沿叶序严格递增、从 1 起（0 不用），下一个号 = 树里最大号 + 1：最右那片的号就是最大号。
-            let next_container_number = containers
-                .last()
-                .map_or(1, |rightmost| rightmost.identity.container + 1);
+            // 最右那片的号是盘上读来的 8 字节：加一越过顶的交回类型化错误（C841），不 panic。
+            let next_container_number = match containers.last() {
+                None => 1,
+                Some(rightmost) => {
+                    let rightmost_container = rightmost.identity.container;
+                    rightmost_container.checked_add(1).ok_or(
+                        NextContainerNumberPastTheTopOfItsRange {
+                            tree,
+                            rightmost_container,
+                        },
+                    )?
+                }
+            };
             containers.push(RecordContainer {
                 identity: PackedIdentity {
                     birth_tree: tree,
@@ -568,12 +579,12 @@ pub fn append_records_to_the_open_container<Record: Clone>(
             "形状里层级 1 的孩子数之和就是容器数"
         );
     }
-    ContainerTreeAfterThisPublish {
+    Ok(ContainerTreeAfterThisPublish {
         containers,
         rewritten_containers,
         shape,
         shape_before_this_publish: shape_before_this_publish.clone(),
-    }
+    })
 }
 
 #[cfg(test)]
@@ -677,7 +688,8 @@ mod tests {
             1,
             1,
             capped(2),
-        );
+        )
+        .expect("最右那片的号 2，加一装得下");
         assert!(after.grew_a_level_this_publish());
         assert_eq!(after.containers[2].identity.container, 3);
         assert_eq!(
@@ -738,7 +750,8 @@ mod tests {
             3,
             2,
             capped(2),
-        );
+        )
+        .expect("最右那片的号 2，加一装得下");
         assert_eq!(after.containers.len(), 3);
         assert_eq!(after.containers[0], before[0], "装满了的第一片原样");
         assert_eq!(after.containers[1].records, vec![12, 13], "第二片装满");
@@ -765,8 +778,8 @@ mod tests {
         );
         assert_eq!(
             after.shape.ancestors_of_containers(&[2]),
-            after.shape.rightmost_path(),
-            "只看最右那一片时祖先就是最右那一条路径"
+            vec![position(1, 1), position(2, 0)],
+            "只看最右那一片时祖先就是最右那一条路径：右半 (1, 1) 与根 (2, 0)，旧根 (1, 0) 不在其内"
         );
         let carried = append_records_to_the_open_container(
             &after.containers,
@@ -777,7 +790,8 @@ mod tests {
             3,
             2,
             capped(2),
-        );
+        )
+        .expect("一条都不追加，不开新容器");
         assert!(carried.rewritten_containers.is_empty() && carried.rewritten_nodes().is_empty());
         assert_eq!(
             carried.containers, after.containers,
@@ -785,6 +799,55 @@ mod tests {
         );
     }
 
+    /// C841：最右那片的号是 `u64::MAX`（坏镜像上读来的）。它还装得下时照常追加进它；装满了、要开新容器时交回
+    /// `NextContainerNumberPastTheTopOfItsRange`、点名树与最右那片的号，不 panic。改之前这一步加一溢出 panic（工作区开着 overflow-checks）。
+    #[test]
+    fn opening_a_container_after_one_numbered_at_the_top_of_its_range_returns_an_error() {
+        let rightmost_at_the_top = RecordContainer {
+            identity: PackedIdentity {
+                birth_tree: TreeIdentifier(65545),
+                record_type: 1,
+                container: u64::MAX,
+                container_birth: CheckpointTxg(5),
+            },
+            records: vec![10_u64],
+        };
+        let before = vec![rightmost_at_the_top];
+        let shape_before = shape_after_opening(1, 2);
+        let appended_into_the_open_container = append_records_to_the_open_container(
+            &before,
+            &shape_before,
+            &[11_u64],
+            CheckpointTxg(9),
+            TreeIdentifier(65545),
+            1,
+            2,
+            capped(2),
+        )
+        .expect("最右那片还装得下：不开新容器，不看它的号");
+        assert_eq!(
+            appended_into_the_open_container.containers[0].records,
+            vec![10, 11]
+        );
+        assert_eq!(
+            append_records_to_the_open_container(
+                &before,
+                &shape_before,
+                &[11_u64],
+                CheckpointTxg(9),
+                TreeIdentifier(65545),
+                1,
+                1,
+                capped(2),
+            ),
+            Err(NextContainerNumberPastTheTopOfItsRange {
+                tree: TreeIdentifier(65545),
+                rightmost_container: u64::MAX,
+            }),
+            "最右那片装满、要开新容器：号加一越过顶，交回类型化错误"
+        );
+    }
+
     /// 只供测试的扇出至少 2、至多产品扇出 135。
     #[test]
     fn the_node_entries_capped_for_tests_lie_between_two_and_the_product_fanout() {
diff --git a/crates/singlefs-core/src/extent_tree.rs b/crates/singlefs-core/src/extent_tree.rs
index 9157645a..967fc097 100644
--- a/crates/singlefs-core/src/extent_tree.rs
+++ b/crates/singlefs-core/src/extent_tree.rs
@@ -88,11 +88,14 @@ pub fn lower_span_in_data_units(level: u8) -> u64 {
 }
 
 /// 上段罩得住 inode 号 `largest_inode` 的最低一层（上段根的层级）。
+///
+/// # Panics
+/// `largest_inode` 是 `u64::MAX`：[`upper_span_in_inodes`] 从第 8 层起饱和在 `u64::MAX`，跨度大于号才算罩得住，饱和的跨度罩不住这个号。
 #[must_use]
 pub fn upper_root_level_for(largest_inode: u64) -> u8 {
     (0..=u8::MAX)
         .find(|level| upper_span_in_inodes(*level) > largest_inode)
-        .expect("第 8 层起罩的 inode 号已超过 u64 的上界")
+        .expect("第 8 层起跨度饱和在 u64::MAX，罩得住小于 u64::MAX 的每个 inode 号；走到这里说明调用方传了 u64::MAX")
 }
 
 /// 下段罩得住 `data_units` 个单元（单元号 0 .. data_units − 1）的最低一层（下段根的层级）。
@@ -193,15 +196,6 @@ impl ExtentLowerNodePosition {
             extent_key_bytes(inode, last_offset),
         )
     }
-
-    /// 这个位置下面那一层的孩子里，罩着单元号 `data_unit` 的那一个。
-    #[must_use]
-    pub fn child_holding_data_unit(self, data_unit: u64) -> Self {
-        Self {
-            level: self.level - 1,
-            index: data_unit / lower_span_in_data_units(self.level - 1),
-        }
-    }
 }
 
 /// 一个 `data_units` 个单元（单元号 0 .. data_units − 1、没有洞）的文件，下段的全部节点，按 bump 次序（先叶后根、同层按位置升序）。
@@ -560,20 +554,6 @@ impl ExtentTreeVersion {
         self.upper_nodes.last().expect("带文件的一版里上段恒有根").0
     }
 
-    /// 上段的高（上段根的层级 + 1，D8（核心索引结构） 已定项 11 ⑤ 的读法）。
-    #[must_use]
-    pub fn upper_height(&self) -> u64 {
-        u64::from(self.upper_root().level) + 1
-    }
-
-    /// 第一个文件下段的高；内联（没有下段）时 0。
-    #[must_use]
-    pub fn lower_height(&self) -> u64 {
-        self.lower_nodes
-            .last()
-            .map_or(0, |(position, _)| u64::from(position.level) + 1)
-    }
-
     #[must_use]
     pub fn upper_pointer_of(&self, position: ExtentUpperNodePosition) -> Option<NodePointer> {
         self.upper_nodes
@@ -1203,7 +1183,12 @@ pub fn read_extent_tree(
 
 #[cfg(test)]
 mod tests {
+    use singlefs_format::TREE_IDENTIFIER_EXTENT;
+
     use super::*;
+    use crate::address::DeviceIdentity;
+    use crate::pointer::{LocationEntry, PointerHead};
+    use crate::unit::WriteOrder;
 
     /// 下段：两个单元就要建下段（根兼叶），144 个还是一片叶，145 个长出层级 1（两片叶加一个根）。
     #[test]
@@ -1260,7 +1245,32 @@ mod tests {
             inode: 8,
             target: ExtentUpperLeafTarget::LowerSegmentRoot(NodePointer::empty_root()),
         };
-        for entry in [no_unit, lower] {
+        let inline = ExtentUpperLeafEntry {
+            inode: 9,
+            target: ExtentUpperLeafTarget::InlineDataUnit(DataPointer {
+                head: PointerHead {
+                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+                    birth_txg: CheckpointTxg(3),
+                },
+                locations: [
+                    LocationEntry {
+                        device: DeviceIdentity(0),
+                        slot: SlotNumber(50180),
+                        unit_checksum: 1,
+                    },
+                    LocationEntry {
+                        device: DeviceIdentity(1),
+                        slot: SlotNumber(50181),
+                        unit_checksum: 2,
+                    },
+                ],
+                write_order: WriteOrder {
+                    instance: InstanceGeneration(1),
+                    transaction: 4,
+                },
+            }),
+        };
+        for entry in [no_unit, lower, inline] {
             let bytes = entry.to_bytes();
             assert_eq!(bytes.len(), 113);
             assert_eq!(ExtentUpperLeafEntry::parse(&bytes), Ok(entry));
diff --git a/crates/singlefs-core/src/inode_tree.rs b/crates/singlefs-core/src/inode_tree.rs
index 75b4975d..b6bf1ceb 100644
--- a/crates/singlefs-core/src/inode_tree.rs
+++ b/crates/singlefs-core/src/inode_tree.rs
@@ -117,10 +117,17 @@ impl InodeLeafContainer {
     }
 }
 
+/// inode 树的 key 宽：inode 号 8 字节（D8（核心索引结构） 已定项 6：叶记录的 key、根条目的分隔 key 都是 inode 号）。
+/// 写侧装根、读侧核根（`crate::recovery` 的 `key_width_for_kind`、`crate::mounted_read`）都引这一个。
+pub const INODE_KEY_WIDTH_IN_BYTES: usize = 8;
+
 /// 一个码 2 根装得下几条 inode 内部条目：扇出 135（D8（核心索引结构） 已定项 6：分隔 key 8 + 身份引用 26 + 子指针 86 = 120）。
 #[must_use]
 pub fn leaf_containers_one_root_node_holds() -> usize {
-    index_node_entry_capacity(8, usize::try_from(INODE_INTERNAL_ENTRY).expect("120"))
+    index_node_entry_capacity(
+        INODE_KEY_WIDTH_IN_BYTES,
+        usize::try_from(INODE_INTERNAL_ENTRY).expect("120"),
+    )
 }
 
 /// 这一版的一片叶容器从哪来。
@@ -172,21 +179,6 @@ impl InodeLeafContainersAfterThisPublish {
             .map(|retired| retired.identity)
             .collect()
     }
-
-    /// 这一版第 `index` 片照抄的话，它在上一版里的叶序；这次重写的交回 `None`。
-    ///
-    /// # Panics
-    /// `index` 不在这一版里：调用方只拿这一版的叶序问。
-    #[must_use]
-    pub fn previous_index_of_a_carried_container(
-        &self,
-        index: InodeLeafContainerIndexInTree,
-    ) -> Option<InodeLeafContainerIndexInTree> {
-        match self.origins[index.position()] {
-            InodeLeafContainerOrigin::RewrittenThisPublish => None,
-            InodeLeafContainerOrigin::CarriedFrom(previous_index) => Some(previous_index),
-        }
-    }
 }
 
 /// 左从右借的下限：一片叶删完一条之后记录数少于它、这次又没并过、它有右邻，就从右邻借（E176 M3 那一臂的主取样点
diff --git a/crates/singlefs-core/src/instance_table_sweep.rs b/crates/singlefs-core/src/instance_table_sweep.rs
index ab7d7aab..64c74704 100644
--- a/crates/singlefs-core/src/instance_table_sweep.rs
+++ b/crates/singlefs-core/src/instance_table_sweep.rs
@@ -25,12 +25,9 @@ use std::collections::{BTreeMap, BTreeSet};
 use singlefs_format::{
     DATA_UNIT_HEADER_BYTES, INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE,
     INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT, INSTANCE_TABLE_PAGE_RECORDS, PACKED_UNIT_HEADER_BYTES,
-    SLOT_BYTES,
 };
 
-use crate::address::{
-    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
-};
+use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
 use crate::allocator::{absolute_slot_count_of_device, PoolAllocator};
 use crate::block_device::{BlockDevice, BlockDeviceError, WriteDurability};
 use crate::checksum::wide_checksum_field_holds;
@@ -851,12 +848,6 @@ pub fn sweep_the_instance_table_while_mounted<Device: BlockDevice>(
     )
 }
 
-/// 一个落点的设备内偏移（给用例读头用）。
-#[must_use]
-pub fn device_offset_of_the_head(placement: PlacementOfAUnitHead) -> DeviceOffsetInBytes {
-    DeviceOffsetInBytes(placement.slot.0 * SLOT_BYTES)
-}
-
 #[cfg(test)]
 mod tests {
     use super::*;
diff --git a/crates/singlefs-core/src/journal.rs b/crates/singlefs-core/src/journal.rs
index a3d850cc..11ffa9f5 100644
--- a/crates/singlefs-core/src/journal.rs
+++ b/crates/singlefs-core/src/journal.rs
@@ -274,9 +274,10 @@ impl JournalRecord {
         bytes
     }
 
-    /// 读者：magic、类型、整条校验和、fsid、载荷校验和四关，外加四条当损坏的（D23（journal 的角色与格式） 已定项 4 / 已定项 7；最后一条是代码审阅第 29 条）：
-    /// 记录标志位 0 之外有位为 1、本次发布内序号为 0、提交标记不是 0 也不是 1，另有新根段四条指针头部里加密、压缩与 extent 偏移那几段不是第一版的取值
-    /// （代码审阅第 29 条），以及新根段的卸载记号那 4 字节位 0 之外有位为 1（与根记录 flags 同判，`root_record::parse_slot`；条款没写这一格，
+    /// 读者：magic、整条校验和、类型、记录长度字段（等于 4096）、fsid、载荷校验和（连同点名项不越过记录末尾）六关，
+    /// 外加五条当损坏的（D23（journal 的角色与格式） 已定项 4 / 已定项 7）：记录标志位 0 之外有位为 1、提交标记不是 0 也不是 1、
+    /// 本次发布内序号为 0、新根段四条指针头部里加密、压缩与 extent 偏移那几段不是第一版的取值（代码审阅第 29 条），
+    /// 以及新根段的卸载记号那 4 字节位 0 之外有位为 1（与根记录 flags 同判，`root_record::parse_slot`；条款没写这一格，
     /// 照根记录与记录标志「其余位读到非 0 当损坏」的同一个处置取）。当损坏就是与校验和不过同一个结局——这条记录不算在，前缀在它之前断
     /// （已定项 22 断号即止）。不查反向链，也不查一次发布之内跳不跳号：那两样要看别的记录，是前缀取法的事。
     #[must_use]
diff --git a/crates/singlefs-core/src/lineage_tree.rs b/crates/singlefs-core/src/lineage_tree.rs
index 5cb089ba..f617ced1 100644
--- a/crates/singlefs-core/src/lineage_tree.rs
+++ b/crates/singlefs-core/src/lineage_tree.rs
@@ -11,7 +11,7 @@ use singlefs_format::LINEAGE_CONTAINER_RECORDS;
 use crate::address::{CheckpointTxg, TreeIdentifier};
 use crate::container_tree::{
     append_record_to_the_open_container, ContainerTreeAfterThisPublish, ContainerTreeNodeEntries,
-    ContainerTreeShape, RecordContainer,
+    ContainerTreeShape, NextContainerNumberPastTheTopOfItsRange, RecordContainer,
 };
 use crate::records::LineageRecord;
 use crate::unit::PACKED_TYPE_LINEAGE;
@@ -96,7 +96,9 @@ pub type LineageTreeAfterThisPublish = ContainerTreeAfterThisPublish<LineageReco
 
 /// 往谱系树的开放容器（最右那一片）追加一条谱系记录（管理员回退那次发布）：装得下就追加进最右那片；装满了（或零根）就开一片新容器，
 /// 容器号 = 树里最大号 + 1（零根时 1），出生代 = 这次发布的 checkpoint_txg，出生树 = `lineage_tree`，打包记录类型 5；根满时长高。
-#[must_use]
+///
+/// # Errors
+/// 要开新容器而最右那片的号（盘上读来的）已是 `u64::MAX` ⇒ [`NextContainerNumberPastTheTopOfItsRange`]（C841）。
 pub fn append_lineage_record_to_the_open_container(
     containers_before_this_publish: &[LineageContainer],
     shape_before_this_publish: &ContainerTreeShape,
@@ -105,7 +107,7 @@ pub fn append_lineage_record_to_the_open_container(
     lineage_tree: TreeIdentifier,
     capacity: LineageContainerCapacity,
     node_entries: ContainerTreeNodeEntries,
-) -> LineageTreeAfterThisPublish {
+) -> Result<LineageTreeAfterThisPublish, NextContainerNumberPastTheTopOfItsRange> {
     append_record_to_the_open_container(
         containers_before_this_publish,
         shape_before_this_publish,
@@ -143,7 +145,8 @@ mod tests {
             LINEAGE_TREE,
             LineageContainerCapacity::FromTheContainerFormat,
             ContainerTreeNodeEntries::FromTheNodeFormat,
-        );
+        )
+        .expect("容器号从 1 起逐片加一，远在顶之下");
         let identity = PackedIdentity {
             birth_tree: LINEAGE_TREE,
             record_type: PACKED_TYPE_LINEAGE,
@@ -174,7 +177,8 @@ mod tests {
             LINEAGE_TREE,
             LineageContainerCapacity::FromTheContainerFormat,
             ContainerTreeNodeEntries::FromTheNodeFormat,
-        );
+        )
+        .expect("容器号从 1 起逐片加一，远在顶之下");
         assert_eq!(
             second.containers,
             vec![LineageContainer {
diff --git a/crates/singlefs-core/src/make_filesystem.rs b/crates/singlefs-core/src/make_filesystem.rs
index 97f78454..fc1ad6ed 100644
--- a/crates/singlefs-core/src/make_filesystem.rs
+++ b/crates/singlefs-core/src/make_filesystem.rs
@@ -250,7 +250,8 @@ fn tree_table_genesis_placement_at(unit_area_start: UnitAreaStart) -> Placement
     }
 }
 
-/// 实例表 kind 1 链指针记录（88 字节）：kind 1 + flags 0 + 指向下一片的指针 86，第一片「无下一片」指针全 0。
+/// 实例表 kind 1 链指针记录（88 字节）：kind 1 + 有无下一片 1 + 指向下一片的指针 86（D18（块里携带什么信息） 已定项 11）；
+/// mkfs 写出的这一片「无下一片」（那一字节写 0）、指针全 0。
 #[must_use]
 pub fn instance_table_chain_record() -> Vec<u8> {
     let mut writer = ByteWriter::new(usize::try_from(INSTANCE_ROW_BYTES).expect("88"));
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 8699bbec..5c41185b 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -398,8 +398,8 @@ pub enum StillUnreadableAfterOneReread {
     /// 被退掉的那条）——的账（树表或分配记录树）读不出、那次发布读得出的记录又不齐（本次发布内序号缺一条，或没有带末条标志的那条），
     /// 照被抛弃根那一份从中央映射现算也不成，重读一次仍不成，现行账里又没有 C694 的 B 那道凭据（`C_MAPPING` 收窄，判决
     /// `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K2 W1 / W2 H1、K2 W2 H3 两行与第三节第 1 条）。
-    /// 较新可读根那次发布一条记录都读不出时，只在系统配置里取过号的实例代号不比那条根新时走到这一拒（攻方
-    /// `research/prompts/m3-batch37-43-code-r1-opus-output.md` 第一节 Y2，用户 2026-10-08 定「两形都修」）。
+    /// 较新可读根那次发布一条记录都读不出时，只在那条根的树表与中央映射树根两个指针都没有「两份都是更新实例的单元」时走到这一拒
+    /// （C834，用户 2026-10-15 定「同一指针两份都更新」，`research/prompts/c834-r3-main-verification.md`「四、用户定案」）。
     /// `unapplied_publish` 点名那次发布（那条较新可读根就是它自己那次发布）。与 C393 那一拒（[`Self::AccountOfAnAbandonedRoot`]）分开报：
     /// 这一拒列在 D23（journal 的角色与格式） 已定项 14 那张单子上，C393 那一拒不在（用户 2026-10-08 定「乙：core 另起错误成员」）。
     /// 在取号之前拒，盘上逐字节不变，只读挂载照常。
@@ -850,7 +850,7 @@ pub struct RollbackTarget {
 }
 
 /// 只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：关掉影子账，只被被抛弃根（崩溃恢复抛弃的时间线）引用的槽照常可分配——
-/// C314（回退可以复用被抛弃的根引用的单元） 那两格必红靠它强制进入；产品路径恒 `On`。
+/// C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 那两格必红靠它强制进入；产品路径恒 `On`。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum ShadowLedger {
     On,
@@ -2017,8 +2017,8 @@ fn placements_in_the_accounts_of_unapplied_publishes<Device: BlockDevice>(
 /// 账里的落点按它的 txg 在根环里对应的槽（[`target_for_publish`]）归组交回，调用方并进 C 的集合、同一个时机解开。账读不出时照没施加的发布那一份办：
 /// 那次发布读得出的记录不齐就从中央映射现算（[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]，仍不成拒），记录齐就不交账（交 F1 乙）、
 /// 改交挂载那一刻现行账（`current_records`）里仍分配着的落点（丙，用户 2026-10-08 定，函数体里那一支的注释）；
-/// 一条记录都读不出的，映射现算得出就交；现算不成时，`journal_instance_in_the_system_configuration`（挂载择到的系统配置里取过号的实例代号，
-/// `SystemRuntimeQuantities::journal_instance`）不比那条根新就照记录不齐那一支办（仍不成拒），比它新就不交也不拒（函数体里那一支的注释）。
+/// 一条记录都读不出的，映射现算得出就交；现算不成时，那条根的树表与中央映射树根两个指针里有一个两份都已被更新实例的单元盖掉
+/// （[`one_pointer_has_both_copies_overwritten_by_a_later_instance`]）就不交也不拒，否则照记录不齐那一支办（仍不成拒；函数体里那一支的注释）。
 /// `roots` 是重建分配器读根环那一遍读出的根（影子账、根环表用的同一份）。
 ///
 /// # Errors
@@ -2026,7 +2026,7 @@ fn placements_in_the_accounts_of_unapplied_publishes<Device: BlockDevice>(
 #[allow(
     clippy::ptr_arg,
     clippy::too_many_arguments,
-    reason = "ptr_arg：placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、根环里读出的根、判被抛弃的谓词、读阶段扫出的记录、所选那一版、每区根环槽数、现行账、系统配置里取过号的实例代号、重读前的钩子"
+    reason = "ptr_arg：placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、根环里读出的根、判被抛弃的谓词、读阶段扫出的记录、所选那一版、每区根环槽数、现行账、重读前的钩子"
 )]
 fn placements_in_the_accounts_of_newer_readable_roots<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
@@ -2037,7 +2037,6 @@ fn placements_in_the_accounts_of_newer_readable_roots<Device: BlockDevice>(
     selected_version: &RootRecord,
     slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
     current_records: &[AllocationRecord],
-    journal_instance_in_the_system_configuration: InstanceGeneration,
     before_the_one_reread: &mut BeforeTheOneReread<'_>,
 ) -> Result<PlacementsInTheAccountsOfNewerReadableRoots, MountError> {
     let mut in_the_accounts_of_newer_roots_by_ring_slot: BTreeMap<
@@ -2063,20 +2062,22 @@ fn placements_in_the_accounts_of_newer_readable_roots<Device: BlockDevice>(
                             })
                             .collect();
                     if readable_records_of_its_publish.is_empty() {
-                        // 那次发布一条记录都读不出：映射现算得出就并进来（攻方 H1 里账读不出的那几格要它）。现算不成时看系统配置里取过号的
-                        // 实例代号：不比那条根新，照「记录不齐」那一支办（重读一次、看 C694 的 B 凭据，仍不成拒，攻方
-                        // `research/prompts/m3-batch37-43-code-r1-opus-output.md` 第一节 Y2，臂 `OPUS_ARM_REFUSE_UNMAPPED=2`，用户 2026-10-08 定「两形都修」）——
-                        // 那条根之后没有别的挂载取过号，它的单元还在，不拒就会在写行那次发布里写掉它们；比那条根新，不拒、不重读——
-                        // 那条根之后已有一次挂载取过号、写掉了它的单元、记录槽也被那次挂载的记录占了（入库用例 `urJ` 那一形），映射本来就现算不成，
-                        // 拒了就再也做不成可写挂载。
+                        // 那次发布一条记录都读不出：映射现算得出就并进来（攻方 H1 里账读不出的那几格要它）。现算不成时只看那条根自己的
+                        // 树表与中央映射树根两个指针（C834，用户 2026-10-15 定「同一指针两份都更新」，`research/prompts/c834-r3-main-verification.md`
+                        // 「四、用户定案」，被攻过零轮；替换用户 2026-10-08 定的 Y2 收严那一判，不再读系统配置里取过号的实例代号——那个号随盘序变）：
+                        // 有一个指针两份都已被更新实例的单元盖掉，那条根那个单元两份都没了、本来就走不完，映射也就现算不成，拒了就再也做不成
+                        // 可写挂载（入库用例 `urJ` 那一形），不拒、不重读；否则照「记录不齐」那一支办（重读一次、看 C694 的 B 凭据，仍不成拒）——
+                        // 它的单元可能还在（只是这一次读不出，或只被盖掉一份），不拒就会在写行那次发布里写掉它们。
                         match placements_referenced_by_root_computed_from_the_mapping(
                             devices,
                             unit_area_start,
                             root,
                         ) {
                             Some(computed) => computed,
-                            None if journal_instance_in_the_system_configuration
-                                <= root.instance =>
+                            None
+                                if !one_pointer_has_both_copies_overwritten_by_a_later_instance(
+                                    devices, root,
+                                ) =>
                             {
                                 placements_of_an_unapplied_publish_whose_account_is_unreadable(
                                     devices,
@@ -2146,6 +2147,65 @@ fn placements_in_the_accounts_of_newer_readable_roots<Device: BlockDevice>(
     })
 }
 
+/// C834 那一判的凭据（用户 2026-10-15 定「同一指针两份都更新」，`research/prompts/c834-r3-main-verification.md`「四、用户定案」，被攻过零轮）：
+/// `root` 的根记录里树表与中央映射树根两个指针中有一个，它两条位置条目指着的两个槽都读出自证过的单元头、写序里的实例代号比 `root` 的实例代号大
+/// （[`instance_in_the_self_verified_unit_header_at`]）。全零的空指针没有槽可读，不算证据。只读盘、不写盘。
+/// 只要一份不够（攻方 `research/prompts/c834-r3-opus-output.md` 第二节 T2：盖掉一份、另一份这一次读回全零时那条根还走得完）。
+#[allow(
+    clippy::ptr_arg,
+    reason = "PoolReader 实现在 Vec<(DeviceIdentity, Device)> 上，切片不能转成 &dyn PoolReader"
+)]
+fn one_pointer_has_both_copies_overwritten_by_a_later_instance<Device: BlockDevice>(
+    devices: &Vec<(DeviceIdentity, Device)>,
+    root: &RootRecord,
+) -> bool {
+    [&root.tree_table, &root.mapping_root]
+        .into_iter()
+        .filter(|pointer| **pointer != crate::pointer::NodePointer::empty_root())
+        .any(|pointer| {
+            pointer.locations.iter().all(|location| {
+                instance_in_the_self_verified_unit_header_at(devices, location)
+                    .is_some_and(|instance| instance > root.instance)
+            })
+        })
+}
+
+/// `location` 那一槽起读出的自证过的单元头（头校验和与载荷校验和都过）里写序的实例代号：先按索引节点读 16 KiB，解不开再读 32 KiB、
+/// 依次按数据单元与打包记录单元解；读不出（越过盘尾、没有那块盘）或三类都解不开交 `None`。只读盘、不写盘。
+#[allow(
+    clippy::ptr_arg,
+    reason = "PoolReader 实现在 Vec<(DeviceIdentity, Device)> 上，切片不能转成 &dyn PoolReader"
+)]
+fn instance_in_the_self_verified_unit_header_at<Device: BlockDevice>(
+    devices: &Vec<(DeviceIdentity, Device)>,
+    location: &LocationEntry,
+) -> Option<InstanceGeneration> {
+    let offset = location.slot.to_device_offset();
+    let index_node_instance = PoolReader::read(
+        devices,
+        location.device,
+        offset,
+        usize::try_from(NODE_BYTES).expect("16384"),
+    )
+    .and_then(|bytes| crate::unit::parse_index_node(&bytes).ok())
+    .map(|header| header.instance);
+    if index_node_instance.is_some() {
+        return index_node_instance;
+    }
+    let unit_bytes = PoolReader::read(
+        devices,
+        location.device,
+        offset,
+        usize::try_from(DATA_UNIT_BYTES).expect("32768"),
+    )?;
+    crate::unit::parse_data_unit(&unit_bytes)
+        .map(|header| header.write_order.instance)
+        .or_else(|_not_a_data_unit| {
+            crate::unit::parse_packed_unit(&unit_bytes).map(|header| header.write_order.instance)
+        })
+        .ok()
+}
+
 /// [`placements_in_the_accounts_of_newer_readable_roots`] 交回的两样。
 struct PlacementsInTheAccountsOfNewerReadableRoots {
     /// 较新可读根的账里的落点，按那条根的 txg 在根环里对应的槽（[`target_for_publish`]）归组。
@@ -2799,8 +2859,8 @@ fn txgs_ring_slots_bad_at_mount_can_hold(
 /// 树表 0 条那一版的账重建不出（`allocator_of_version_without_file`）；
 /// 读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，两遍逐槽并起来整个环仍一条都没有 ⇒
 /// `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`（用户 2026-10-05 定「重读」、Q3 另起拒因、Q4 rr2、R1 按槽并两遍）；
-/// C 读一次没施加的发布或一条较新可读根的账读不出、那次发布读得出的记录不齐（较新可读根那次发布一条记录都读不出、系统配置里取过号的实例代号
-/// 又不比它新的，也算这一支）、从中央映射也现算不成、重读一次仍不成
+/// C 读一次没施加的发布或一条较新可读根的账读不出、那次发布读得出的记录不齐（较新可读根那次发布一条记录都读不出、树表与中央映射树根
+/// 两个指针都没有两份都是更新实例的单元的，也算这一支）、从中央映射也现算不成、重读一次仍不成
 /// （[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]）⇒
 /// `NewerStateStillUnreadableAfterOneReread(AccountOfAnUnappliedPublishWithIncompleteReadableRecords)`。
 /// 都在取号之前，盘上逐字节不变。
@@ -3121,7 +3181,6 @@ fn rebuilt_allocator<Device: BlockDevice>(
                 selected_version,
                 slots_per_region,
                 &current_records,
-                system_configuration.quantities.journal_instance,
                 before_the_one_reread,
             )?;
             for (ring_slot, placements_in_the_newer_roots_account) in
@@ -3872,7 +3931,8 @@ pub fn raise_rollback_floor<Device: BlockDevice>(
 /// 这一串写的根在 flags 里带卸载记号（D22（单元原子性怎么合成） 已定项 7）。与准入那一格差在回收出来的槽里释放代不高于每块盘上最新有效根的
 /// 那一截不扣到生效、预演与真发都能用，其余照样扣（用户 2026-10-03 定甲 GA，[`RaiseFloorEntry::reuse_of_the_reclaimed_slots`]）：
 /// 挂载续做推满仍放不下、槽被吃光的盘上卸载照样做成，靠的就是这一串的预演算上那一截。第一版几何（R = 3、区域归属 0 / 1 / 0）下推 2 或 3 次：
-/// 现行 txg 除以 3 余 1 时 3 次，余 0 或余 2 时 2 次。不写干净关闭标记，下一次挂载照旧走恢复。做完之后这个写入口不再发布。
+/// 现行 txg 除以 3 余 1 时 3 次，余 0 或余 2 时 2 次。不写干净关闭标记，下一次挂载照旧走恢复。盘表、分配器与现行那一版只借给它
+/// （`&mut`），返回之后调用方手里照样有；卸载之后不再拿它们发布，由调用方守，这里不拦。
 /// 现行那一版树表 0 条时一个字节都不写，卸载照常做成（[`Unmounted::NothingWrittenOnAVersionWithoutFile`]）。
 ///
 /// # Errors
@@ -6802,7 +6862,7 @@ pub fn mount_writable_with_space_admission<Device: BlockDevice>(
 }
 
 /// 同 [`mount_writable_with_space_admission`]，影子账那一臂也由调用方给（只供测试的开关 [`ShadowLedger`]，`.claude/rules/fs-design.md`
-/// 五条硬要求第 2 条：C314（回退可以复用被抛弃的根引用的单元） 那一格的必红靠它强制进入——被抛弃的根由崩溃恢复造出）。
+/// 五条硬要求第 2 条：C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 那一格的必红靠它强制进入——被抛弃的根由崩溃恢复造出）。
 /// 产品路径走 [`mount_writable`]，恒 `On`。
 ///
 /// # Errors
@@ -8428,6 +8488,28 @@ mod non_empty_root_tests {
         );
     }
 
+    /// 上限取两者里小的那一个的另一半（上面那条两次都传 9，「每块盘上最新的有效根」从没胜出）：四个不同状态、第 4 新的是 3，
+    /// 而有一块盘上最新的有效根只到 2（那块盘之后的根都读不出），上限取 2。只取第 4 新的那一条时上限是 3，越过那块盘上最新的有效根。
+    #[test]
+    fn the_ceiling_is_the_newest_valid_root_on_every_device_when_it_is_older_than_the_fourth_newest_distinct_state(
+    ) {
+        let four_states = newest_first_distinct_states(vec![
+            non_empty_root(3, 0xA),
+            non_empty_root(4, 0xB),
+            non_empty_root(8, 0xC),
+            non_empty_root(9, 0xD),
+        ]);
+        assert_eq!(
+            ceiling_from_newest_and_non_empty_roots(
+                CheckpointTxg(2),
+                four_states,
+                Some(CheckpointTxg(1))
+            ),
+            Some(CheckpointTxg(2)),
+            "每块盘上最新的有效根 2 低于第 4 新的 3：上限取 2"
+        );
+    }
+
     fn pointers(inode_tree: u8, extent_tree: u8) -> UserVisibleTreeRootPointers {
         UserVisibleTreeRootPointers {
             inode_tree: Some(vec![inode_tree; 86]),
@@ -9703,12 +9785,20 @@ mod shadow_ledger_exempts_and_isolates_slot_by_slot_tests {
     /// 被抛弃根的那一版、现行账记录的分配代：这几条用例只看豁免与判重，代取多少都一样。
     const ALLOCATION_GENERATION: CheckpointTxg = CheckpointTxg(5);
 
-    /// 只在内存里的盘：写过的页才存，没写过的读回全零；越过末尾报越界。
-    struct SparseDeviceInMemory {
+    /// 只在内存里的盘：写过的页才存，没写过的读回全零；越过末尾报越界。同文件里经真发布搭池的那几组用例
+    /// （[`super::pool_in_memory_after_the_first_file`]）也用它。
+    pub(super) struct SparseDeviceInMemory {
         pages: BTreeMap<u64, Vec<u8>>,
     }
 
     impl SparseDeviceInMemory {
+        /// 一页都没写过的盘：读哪里都是全零。
+        pub(super) fn blank() -> Self {
+            Self {
+                pages: BTreeMap::new(),
+            }
+        }
+
         fn refuse_past_the_end(
             offset: DeviceOffsetInBytes,
             length: u64,
@@ -10153,4 +10243,428 @@ mod shadow_ledger_exempts_and_isolates_slot_by_slot_tests {
             "起点 + 1 被候选根（mkfs 那一版）的实例表 (起点, 跨 2) 罩着：不隔离"
         );
     }
+
+    /// 先看豁免、再判重（判决 `research/prompts/code-backlog-b2-code-r1-main-verification.md` 第三节 A1）：起点 s 同时被候选根（mkfs 那一版的
+    /// 实例表 (s, 跨 2)）、一条根环表里记着的被抛弃根 (s, 跨 1) 与水位之上没施加的记录 (s, 跨 1) 引用，现行账里没有它。被抛弃根那一圈豁免它
+    /// （候选根引用），没施加的记录那一圈只豁免现行账，要隔离它。先判重后看豁免时，被抛弃根那一圈把 s 记进判重集合，后一圈判重跳过，s 发得出去。
+    #[test]
+    fn slot_an_abandoned_roots_pass_exempted_for_a_candidate_is_still_isolated_for_the_unapplied_journal_records(
+    ) {
+        let pool = formatted_pool();
+        let start = pool.unit_area_start.slot();
+        let root = abandoned_root(&pool, 1, 8);
+        let recorded = BTreeMap::from([(
+            (root.instance, root.checkpoint_txg),
+            vec![placement_on_device_zero(start, 1)],
+        )]);
+        let allocator = allocator_after_the_shadow_ledger(
+            &pool,
+            &ShadowLedgerInputs {
+                roots: &[pool.make_filesystem_root, root],
+                abandoned_instances: &[root.instance],
+                current_records: &[],
+                recorded_placements_of_abandoned_roots: &recorded,
+                placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[],
+                placements_named_by_unapplied_journal_records: &[placement_on_device_zero(
+                    start, 1,
+                )],
+            },
+        );
+        assert_eq!(
+            isolated_and_free_on_device_zero(&allocator, start),
+            (true, false),
+            "s 只被候选根豁免给被抛弃根那一圈，没施加的记录那一圈照隔离：隔离着、发不出去"
+        );
+    }
+
+    /// 同上，豁免 s 的换成根槽这一遍读坏的被抛弃根那一圈（记着 (s, 跨 1)）。
+    #[test]
+    fn slot_the_pass_for_abandoned_roots_whose_ring_slot_read_bad_exempted_is_still_isolated_for_the_unapplied_journal_records(
+    ) {
+        let pool = formatted_pool();
+        let start = pool.unit_area_start.slot();
+        let allocator = allocator_after_the_shadow_ledger(
+            &pool,
+            &ShadowLedgerInputs {
+                roots: &[pool.make_filesystem_root],
+                abandoned_instances: &[],
+                current_records: &[],
+                recorded_placements_of_abandoned_roots: &BTreeMap::new(),
+                placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[
+                    placement_on_device_zero(start, 1),
+                ],
+                placements_named_by_unapplied_journal_records: &[placement_on_device_zero(
+                    start, 1,
+                )],
+            },
+        );
+        assert_eq!(
+            isolated_and_free_on_device_zero(&allocator, start),
+            (true, false),
+            "s 只被候选根豁免给根槽读坏的被抛弃根那一圈，没施加的记录那一圈照隔离：隔离着、发不出去"
+        );
+    }
+}
+
+/// 经真发布搭的池（mkfs、取号、暖机、第一个文件，之后同一个进程里覆盖写）：管理员回退那一道与「每块盘上最新的有效根」两种算法的对拍
+/// 两组用例共用。两块 4 GiB 的内存稀疏盘（[`super::shadow_ledger_exempts_and_isolates_slot_by_slot_tests`] 里那一种），几何同那一组的 mkfs 参数。
+#[cfg(test)]
+mod pool_in_memory_after_the_first_file {
+    use super::shadow_ledger_exempts_and_isolates_slot_by_slot_tests::SparseDeviceInMemory;
+    use super::{roll_back_by_a_forward_publish, RollbackError, RollbackTarget, RolledBack};
+    use crate::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
+    use crate::allocator::PoolAllocator;
+    use crate::block_device::{BlockDevice, WriteDurability};
+    use crate::make_filesystem::{
+        allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
+    };
+    use crate::root_ring::RootRingSlotsPerRegion;
+    use crate::system_configuration::SystemImmutableSizes;
+    use crate::transaction::{
+        acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
+        TransactionOutput,
+    };
+
+    /// 第一个文件的写入时刻；之后每次覆盖写晚一秒。
+    const FIRST_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
+    /// 第一个文件的字节数；之后每次覆盖写少一个字节（都在一个数据单元以内）。
+    const FIRST_FILE_BYTES: usize = 3000;
+
+    /// 池：mkfs 参数、两块盘、这个进程的分配器、取到的实例代号、现行那一版。
+    pub(super) struct PoolInMemory {
+        pub(super) parameters: MakeFilesystemParameters,
+        pub(super) devices: Vec<(DeviceIdentity, SparseDeviceInMemory)>,
+        pub(super) allocator: PoolAllocator,
+        pub(super) instance: InstanceGeneration,
+        pub(super) current: TransactionOutput,
+        write_time_seconds: u64,
+    }
+
+    /// `length` 个字节的文件内容，`seed` 不同内容就不同。
+    fn content_of(length: usize, seed: u64) -> Vec<u8> {
+        (0..length)
+            .map(|index| {
+                let index = u64::try_from(index).expect("长度装得进 u64");
+                u8::try_from((index * 131 + seed * 7 + 1) % 251).expect("小于 256")
+            })
+            .collect()
+    }
+
+    /// mkfs、取号、暖机、第一个文件，都在同一个进程里。
+    pub(super) fn pool_after_the_first_file() -> PoolInMemory {
+        let parameters = MakeFilesystemParameters {
+            filesystem_identifier: [7; 16],
+            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
+            geometry: SystemImmutableSizes {
+                physical_block_size: 512,
+                minimum_input_output_bytes: 512,
+                fixed_structure_slot_spacing: 4096,
+                journal_ring_bytes: singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
+                root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
+            },
+        };
+        let mut devices: Vec<(DeviceIdentity, SparseDeviceInMemory)> =
+            [DeviceIdentity(0), DeviceIdentity(1)]
+                .into_iter()
+                .map(|identity| (identity, SparseDeviceInMemory::blank()))
+                .collect();
+        let genesis =
+            make_filesystem(&parameters, &mut devices).expect("4 GiB 两盘照默认几何建得成");
+        let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
+        let first_content = content_of(FIRST_FILE_BYTES, 0);
+        let (instance, current) = {
+            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+            let instance = acquire_instance(&mut writer).expect("mkfs 刚建的池上取号做得成");
+            let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机做得成");
+            let current = publish_first_file(
+                &mut writer,
+                &mut allocator,
+                warmed.roots.last().expect("暖机写了两条根"),
+                FirstFile {
+                    content: &first_content,
+                    write_time_seconds: FIRST_WRITE_TIME_SECONDS,
+                },
+                instance,
+                &warmed.last_record_bytes,
+            )
+            .expect("第一个文件落得下");
+            (instance, current)
+        };
+        PoolInMemory {
+            parameters,
+            devices,
+            allocator,
+            instance,
+            current,
+            write_time_seconds: FIRST_WRITE_TIME_SECONDS,
+        }
+    }
+
+    impl PoolInMemory {
+        /// 同一个进程里覆盖写一次，新的一版记成现行那一版、交回它。
+        pub(super) fn overwrite(&mut self) -> TransactionOutput {
+            self.write_time_seconds += 1;
+            let content = content_of(FIRST_FILE_BYTES - 1, self.write_time_seconds);
+            let mut writer = PoolWriter::new(&self.parameters, self.devices.as_mut_slice());
+            let output = publish_overwrite(
+                &mut writer,
+                &mut self.allocator,
+                &self.current,
+                FirstFile {
+                    content: &content,
+                    write_time_seconds: self.write_time_seconds,
+                },
+                self.instance,
+            )
+            .expect("4 GiB 两盘上覆盖写落得下");
+            self.current = output.clone();
+            output
+        }
+
+        /// 挂着时回退到 `target`（[`roll_back_by_a_forward_publish`]），做成时现行那一版换成回退那次发布。
+        pub(super) fn roll_back(
+            &mut self,
+            target: RollbackTarget,
+        ) -> Result<RolledBack, RollbackError> {
+            roll_back_by_a_forward_publish(
+                &self.parameters,
+                &mut self.devices,
+                &mut self.allocator,
+                &mut self.current,
+                target,
+            )
+        }
+
+        /// `device` 上 `offset` 起 `length` 个字节，不经任何发布。
+        pub(super) fn bytes_on(
+            &self,
+            device: DeviceIdentity,
+            offset: DeviceOffsetInBytes,
+            length: usize,
+        ) -> Vec<u8> {
+            let (_, block_device) = self
+                .devices
+                .iter()
+                .find(|(identity, _)| *identity == device)
+                .expect("池里有这块盘");
+            let mut bytes = vec![0; length];
+            block_device
+                .read_at(offset, &mut bytes)
+                .expect("读的那一段在盘里");
+            bytes
+        }
+
+        /// 把 `bytes` 直接写到 `device` 上 `offset` 起那一段，不经任何发布（用例拿它摆坏份与读不出的根槽）。
+        pub(super) fn write_bytes_directly_on(
+            &mut self,
+            device: DeviceIdentity,
+            offset: DeviceOffsetInBytes,
+            bytes: &[u8],
+        ) {
+            let (_, block_device) = self
+                .devices
+                .iter_mut()
+                .find(|(identity, _)| *identity == device)
+                .expect("池里有这块盘");
+            block_device
+                .write_at(offset, bytes, WriteDurability::Plain)
+                .expect("写的那一段在盘里");
+        }
+    }
+}
+
+#[cfg(test)]
+mod rollback_does_not_verify_units_the_current_version_shares_with_the_target_tests {
+    use super::pool_in_memory_after_the_first_file::pool_after_the_first_file;
+    use super::RollbackTarget;
+    use crate::address::DeviceIdentity;
+    use crate::transaction::QuarantinedCopyReading;
+    use singlefs_format::DATA_UNIT_BYTES;
+
+    /// 改坏一份数据单元时翻的那个字节在单元里的下标（净荷里，单元头之后）。
+    const BYTE_FLIPPED_INSIDE_THE_DATA_UNIT: usize = 4096;
+
+    /// 「cur 那一版引用的单元不进逐盘验」（判决 `research/prompts/code-backlog-b2-code-r1-main-verification.md` 第三节 A2；D23（journal 的角色与格式）
+    /// 已定项 14「其余 cur 的账里仍分配的单元……不在逐盘验之内」）：第一个文件 A；盘 1 上 A 的数据单元那一份改坏，覆盖写 B 释放它时读盘核出对不上、
+    /// 隔离；那一份写回原样、回退到 A（回退那次发布的新根引用这个数据单元）；再把盘 1 那一份改坏、再回退到 A。现行那一版与 A 共用这个单元，它在现行账里
+    /// 分配着、分配器发不出去，不进逐盘验：回退照做。判别力：那一道去掉（`true || !is_referenced_by_the_current_version`）时它进隔离单元的逐盘验、
+    /// 拒成 `IsolatedUnitCopyUnreadableOrMismatched`，这一条红。两条位置项不同槽的映射条目怎么认条款没写（K2.4），这里只取同槽。
+    #[test]
+    fn rolling_back_to_a_version_sharing_a_unit_with_a_bad_copy_with_the_current_version_goes_through(
+    ) {
+        let mut pool = pool_after_the_first_file();
+        let first = pool.current.clone();
+        let target = RollbackTarget {
+            instance: first.root.instance,
+            checkpoint_txg: first.root.checkpoint_txg,
+        };
+        let data_locations = first.data_pointers[0].locations;
+        assert_eq!(
+            data_locations[0].slot, data_locations[1].slot,
+            "前提：A 的数据单元两份同槽"
+        );
+        let data_slot = data_locations[0].slot;
+        let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
+        let intact_copy = pool.bytes_on(
+            DeviceIdentity(1),
+            data_slot.to_device_offset(),
+            data_unit_bytes,
+        );
+        let mut corrupted_copy = intact_copy.clone();
+        corrupted_copy[BYTE_FLIPPED_INSIDE_THE_DATA_UNIT] ^= 0xff;
+        pool.write_bytes_directly_on(
+            DeviceIdentity(1),
+            data_slot.to_device_offset(),
+            &corrupted_copy,
+        );
+        let second = pool.overwrite();
+        assert!(
+            second
+                .quarantined_after_release_checksum_mismatch
+                .iter()
+                .any(|copy| copy.device == DeviceIdentity(1)
+                    && copy.placement.slot == data_slot
+                    && copy.reading == QuarantinedCopyReading::ChecksumMismatch),
+            "前提：B 释放 A 的数据单元时核出盘 1 那一份对不上、隔离：{:?}",
+            second.quarantined_after_release_checksum_mismatch
+        );
+        pool.write_bytes_directly_on(
+            DeviceIdentity(1),
+            data_slot.to_device_offset(),
+            &intact_copy,
+        );
+        pool.roll_back(target)
+            .expect("隔离单元两份都对得上：第一次回退照做");
+        assert!(
+            pool.current.data_pointers.iter().any(|pointer| pointer
+                .locations
+                .iter()
+                .any(|location| location.slot == data_slot)),
+            "前提：第一次回退之后现行那一版引用 A 的数据单元"
+        );
+        pool.write_bytes_directly_on(
+            DeviceIdentity(1),
+            data_slot.to_device_offset(),
+            &corrupted_copy,
+        );
+        let second_rollback = pool.roll_back(target);
+        assert!(
+            second_rollback.is_ok(),
+            "现行那一版与目标共用的单元不进逐盘验，盘 1 那一份坏着回退照做：{:?}",
+            second_rollback.err()
+        );
+    }
+}
+
+#[cfg(test)]
+mod newest_valid_root_on_every_device_agrees_between_the_ceiling_and_the_effective_floor_tests {
+    use super::pool_in_memory_after_the_first_file::pool_after_the_first_file;
+    use super::rollback_floor_ceiling;
+    use crate::address::{CheckpointTxg, DeviceIdentity};
+    use crate::recovery::{
+        choose_root, choose_system_configuration, effective_rollback_floor,
+        instance_table_chain_of_root, read_root_ring_slot, RootRingSlotReading,
+    };
+    use crate::root_ring::{slot_offset, target_for_publish};
+    use std::collections::BTreeSet;
+
+    /// 第一个文件（txg 3）之后覆盖写几次：现行那一版落到 txg 12，根环（每区 8 槽、环长 24）还没转满。
+    const OVERWRITES_AFTER_THE_FIRST_FILE: u64 = 9;
+    /// 盘 1（区域 1，txg 除以 3 余 1）上读回全零的两条根：那块盘上最新的两条。
+    const ROOTS_ON_DEVICE_ONE_READING_BACK_ZERO: [CheckpointTxg; 2] =
+        [CheckpointTxg(7), CheckpointTxg(10)];
+    /// 那两条读不出之后盘 1 上最新的有效根。
+    const ROOT_LEFT_NEWEST_ON_DEVICE_ONE: CheckpointTxg = CheckpointTxg(4);
+
+    /// 「每块盘上最新的有效根」两种算法连起来比（代码审阅 D 组第 7 条，`.claude/kb/milestone/03-third-txn.md` 五·10）：抬 F 的上限 [`rollback_floor_ceiling`] 取每块盘上
+    /// 最新有效根的 txg 的最小值，生效的回退下界 [`effective_rollback_floor`] 取每块盘上最新有效根带的 F 的最大值。txg 12 的池上让盘 1
+    /// 最新的两条根读回全零，盘 1 上最新的有效根退到 txg 4（低于第 4 新的不同状态 txg 8），再把 txg 4 那条根带的 F 改成 4（别的根都带 0）：
+    /// 两边认的盘 1 上最新有效根是同一条时，上限与生效的 F 都是 4。只调两边的公开入口。
+    #[test]
+    fn the_ceiling_and_the_effective_floor_take_the_same_root_as_the_newest_valid_one_on_a_device()
+    {
+        let mut pool = pool_after_the_first_file();
+        for _overwrite in 0..OVERWRITES_AFTER_THE_FIRST_FILE {
+            pool.overwrite();
+        }
+        assert_eq!(
+            pool.current.root.checkpoint_txg,
+            CheckpointTxg(12),
+            "前提：第一个文件 txg 3，之后覆盖写 9 次落到 txg 12"
+        );
+        let sizes = pool.parameters.geometry;
+        let region_devices = pool.parameters.region_devices;
+        let filesystem_identifier = pool.parameters.filesystem_identifier;
+        let root_slot_bytes = usize::try_from(sizes.physical_block_size).expect("根槽宽");
+        for checkpoint_txg in ROOTS_ON_DEVICE_ONE_READING_BACK_ZERO
+            .into_iter()
+            .chain([ROOT_LEFT_NEWEST_ON_DEVICE_ONE])
+        {
+            let ring_slot = target_for_publish(checkpoint_txg, sizes.root_ring_slots_per_region);
+            assert_eq!(
+                region_devices[usize::try_from(ring_slot.region).expect("区域号")],
+                DeviceIdentity(1),
+                "前提：txg {checkpoint_txg:?} 的根落在盘 1"
+            );
+        }
+        for checkpoint_txg in ROOTS_ON_DEVICE_ONE_READING_BACK_ZERO {
+            let ring_slot = target_for_publish(checkpoint_txg, sizes.root_ring_slots_per_region);
+            pool.write_bytes_directly_on(
+                DeviceIdentity(1),
+                slot_offset(ring_slot, sizes.fixed_structure_slot_spacing),
+                &vec![0; root_slot_bytes],
+            );
+        }
+        let ring_slot_of_the_root_left_newest = target_for_publish(
+            ROOT_LEFT_NEWEST_ON_DEVICE_ONE,
+            sizes.root_ring_slots_per_region,
+        );
+        let RootRingSlotReading::SelfVerified(mut root_left_newest) = read_root_ring_slot(
+            &pool.devices,
+            &region_devices,
+            &sizes,
+            &filesystem_identifier,
+            ring_slot_of_the_root_left_newest,
+        ) else {
+            panic!("前提：txg 4 那条根在它的根环槽里自证过");
+        };
+        root_left_newest.rollback_floor = root_left_newest.checkpoint_txg;
+        pool.write_bytes_directly_on(
+            DeviceIdentity(1),
+            slot_offset(
+                ring_slot_of_the_root_left_newest,
+                sizes.fixed_structure_slot_spacing,
+            ),
+            &root_left_newest.to_slot(root_slot_bytes),
+        );
+        let system_configuration =
+            choose_system_configuration(&pool.devices).expect("两块盘的系统配置读得出");
+        let newest_root =
+            choose_root(&pool.devices, &system_configuration).expect("根环里有自证过的根");
+        let newest_roots_table = instance_table_chain_of_root(&pool.devices, &newest_root)
+            .expect("最新那条根的实例表读得出")
+            .records;
+        let ceiling = rollback_floor_ceiling(
+            &pool.devices,
+            &system_configuration,
+            CheckpointTxg(0),
+            &newest_roots_table,
+            &BTreeSet::new(),
+        )
+        .expect("有效根都读得出树表：上限算得出");
+        let effective_floor = effective_rollback_floor(
+            &pool.devices,
+            &region_devices,
+            &sizes,
+            &filesystem_identifier,
+        );
+        assert_eq!(
+            ceiling, ROOT_LEFT_NEWEST_ON_DEVICE_ONE,
+            "上限取盘 1 上最新的有效根 txg 4（盘 0 最新 txg 12，第 4 新的不同状态 txg 8）"
+        );
+        assert_eq!(
+            effective_floor, ceiling,
+            "生效的 F 取盘 1 上最新有效根带的 4：两边认的是同一条根"
+        );
+    }
 }
diff --git a/crates/singlefs-core/src/mounted_read.rs b/crates/singlefs-core/src/mounted_read.rs
index f989c7f5..4411be65 100644
--- a/crates/singlefs-core/src/mounted_read.rs
+++ b/crates/singlefs-core/src/mounted_read.rs
@@ -34,6 +34,7 @@ use crate::extent_tree::{
     extent_key_bytes, find_upper_leaf_entry, read_lower_segment, ExtentTreeHeaderJudgement,
     ExtentTreeReading, ExtentUpperLeafTarget,
 };
+use crate::inode_tree::INODE_KEY_WIDTH_IN_BYTES;
 use crate::pointer::{
     DataPointer, LocationEntry, NodePointer, PointerHeadFieldOutsideTheFirstVersion,
 };
@@ -58,14 +59,10 @@ use crate::unit::{
 };
 use crate::write_request_split::{data_unit_count_implied_by_the_file_size, DataUnitsOfTheInode};
 
-/// extent 叶记录 key 三段各 8 字节里 inode 那一段的起点（D8（核心索引结构） 已定项 3 的 (locality_id, inode, offset)）。
-const EXTENT_KEY_INODE_SEGMENT_OFFSET_IN_BYTES: usize = 8;
 /// extent 叶记录 key 第三段（offset 段，文件字节偏移，D8（核心索引结构） 已定项 3）的起点。
 const EXTENT_KEY_THIRD_SEGMENT_OFFSET_IN_BYTES: usize = 16;
 /// inode 树根的层级：根是内部节点，每条条目指一片码 3 叶容器（D8（核心索引结构） 已定项 6）。
 const INODE_TREE_ROOT_LEVEL: u8 = 1;
-/// inode 树 key 宽 8（`recovery::key_width_for_kind` 同一份登记）。
-const INODE_KEY_WIDTH_IN_BYTES: usize = 8;
 
 /// 读路径的运行时观测点：并行线二验收第 4 条要的两个数（这次读了几个单元、位置提示过期的多跳次数），
 /// 加上 D17（实现分层与第三方管道） 已定项 5 射程 ① 要的「这次 API 调用发了几个设备级操作」。
@@ -151,17 +148,6 @@ pub struct ExtentLeafRecordInMountState {
 }
 
 impl ExtentLeafRecordInMountState {
-    /// key 第二段：这条记录属于哪个 inode。
-    #[must_use]
-    pub fn inode_number(&self) -> InodeNumber {
-        InodeNumber(u64::from_le_bytes(
-            self.key[EXTENT_KEY_INODE_SEGMENT_OFFSET_IN_BYTES
-                ..EXTENT_KEY_INODE_SEGMENT_OFFSET_IN_BYTES + 8]
-                .try_into()
-                .expect("key 24 字节，第二段落在 [8, 16)"),
-        ))
-    }
-
     /// key 第三段：这条记录指的那个数据单元第一个字节在文件里的偏移（D8（核心索引结构） 已定项 3）。
     #[must_use]
     pub fn file_offset_in_bytes(&self) -> FileOffsetInBytes {
@@ -172,12 +158,6 @@ impl ExtentLeafRecordInMountState {
                 .expect("key 24 字节，第三段落在 [16, 24)"),
         ))
     }
-
-    /// 这条记录指的那个数据单元的指针（位置条目是提示，D19（块指针的结构与宽度预算） 已定项 5）。
-    #[must_use]
-    pub const fn data_unit_pointer(&self) -> DataPointer {
-        self.data_unit_pointer
-    }
 }
 
 /// 中央映射里的一条：27 字节 key 加两条位置条目（D19（块指针的结构与宽度预算） 已定项 10）。
@@ -313,10 +293,19 @@ pub struct ExtentTreeReadsAtOpen {
     pub node_reads: u64,
     /// 这一路的高：上段的高加这个文件下段的高（内联时下段是 0）。
     pub height: u64,
-    /// 叶片数：读到的层级 0 的节点个数（上段那一片叶加下段的叶）。
+    /// 叶片数：读到的层级 0 的节点个数（上段那一片叶加下段的叶）。上段停在内部节点、一片叶都没读到时上段那一份是 0。
     pub leaves: u64,
 }
 
+/// 打开文件时上段那一趟读到了几片叶：上段停下时最后读的那个节点是层级 0（叶）就是 1，停在内部节点（中途没有孩子罩着这个 inode）是 0。
+/// `level_of_the_upper_node_read_last` 是上段停下时最后读的那个节点头里的层级。
+fn upper_leaves_read_at_open(level_of_the_upper_node_read_last: Option<u8>) -> u64 {
+    match level_of_the_upper_node_read_last {
+        Some(0) => 1,
+        Some(_) | None => 0,
+    }
+}
+
 /// 打开挂载态时读中央映射树读了几个节点（每个节点一次，不按位置条目数）与树高（根的层级加一）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct CentralMappingTreeReadsAtOpen {
@@ -717,21 +706,30 @@ impl MountedPoolForRead {
             |mapping_key: &[u8]| Ok(self.central_mapping_lookup(mapping_key));
         let mut tree_node_stale_location_hint_hops: usize = 0;
         let mut node_reads: u64 = 0;
+        // 最近读到的那个节点头里的层级：上段那一趟走完时它说的是上段停在哪一层（0 是叶）。解不开的节点交 `None`，
+        // 那一趟随即在 `find_upper_leaf_entry` 里按解不开报错，用不到它。
+        let level_of_the_node_read_last: Cell<Option<u8>> = Cell::new(None);
         let mut read_node = |pointer: &NodePointer| {
             node_reads += 1;
-            read_mapped_tree_node_via_hint_then_central_mapping(
+            let bytes = read_mapped_tree_node_via_hint_then_central_mapping(
                 reader,
                 pointer,
                 MappedTreeNodeClass::IndexNode,
                 &central_mapping_locations_of_key,
                 &mut tree_node_stale_location_hint_hops,
-            )
+            )?;
+            level_of_the_node_read_last
+                .set(parse_index_node(&bytes).ok().map(|header| header.level));
+            Ok(bytes)
         };
         let (entry, upper_nodes_read) =
             find_upper_leaf_entry(&reading, &self.extent_tree_root, inode.0, &mut read_node)
                 .map_err(OpenFileFailure::ExtentTreeWalk)?;
-        // 上段每一层读一个节点（叶那一片在内），所以上段的高就是这一路读的上段节点数。
+        // 上段从根往下每一层读一个节点：走到叶时读的个数就是上段的高；中途某个内部节点没有孩子罩着这个 inode 时
+        // （`find_upper_leaf_entry` 交回 `None`）停在那一层、叶没读到，这里记的是这一路实际读过的层数，少于上段的高。
         let upper_height = upper_nodes_read;
+        // 上段这一趟读到叶没有：停下时最后读的那个节点是层级 0 才算读到一片（上段停在内部节点时不算）。
+        let upper_leaves = upper_leaves_read_at_open(level_of_the_node_read_last.get());
         let data_units_of_the_inode = match entry.map(|found| found.target) {
             None | Some(ExtentUpperLeafTarget::NoDataUnit) => DataUnitsOfTheInode::NoneWritten,
             Some(
@@ -770,7 +768,7 @@ impl MountedPoolForRead {
         let extent_tree_reads_at_open = ExtentTreeReadsAtOpen {
             node_reads,
             height: upper_height + lower_height,
-            leaves: 1 + lower_leaves,
+            leaves: upper_leaves + lower_leaves,
         };
         let payload_capacity = payload_capacity_in_bytes();
         let data_unit_records: Vec<ExtentLeafRecordInMountState> = data_pointers
@@ -872,12 +870,6 @@ impl OpenFileForRead<'_> {
         self.tree_node_stale_location_hint_hops_at_open
     }
 
-    /// 这个文件按位次排好的数据单元记录（第 i 项是文件第 i 个数据单元）。
-    #[must_use]
-    pub fn data_unit_records(&self) -> &[ExtentLeafRecordInMountState] {
-        &self.data_unit_records
-    }
-
     /// 这个文件有几个数据单元。
     #[must_use]
     pub fn data_unit_count(&self) -> u64 {
@@ -1232,4 +1224,157 @@ mod tests {
             125_000
         );
     }
+
+    /// 两块盘同一个槽上是同一个节点的字节，别的位置都读不出：打开文件那一趟只读得到这一个节点。
+    struct TwoDevicesHoldingOneNode {
+        slot: SlotNumber,
+        node_bytes: Vec<u8>,
+    }
+
+    /// 两块盘各 4 GiB（与坏盘输入那一档的盘同宽）。
+    const DEVICE_BYTES_OF_THE_TWO_DEVICES: u64 = 4 << 30;
+
+    impl PoolReader for TwoDevicesHoldingOneNode {
+        fn device_identities(&self) -> Vec<DeviceIdentity> {
+            vec![DeviceIdentity(0), DeviceIdentity(1)]
+        }
+        fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
+            self.device_identities()
+                .contains(&device)
+                .then_some(DEVICE_BYTES_OF_THE_TWO_DEVICES)
+        }
+        fn read(
+            &self,
+            device: DeviceIdentity,
+            offset: crate::address::DeviceOffsetInBytes,
+            length: usize,
+        ) -> Option<Vec<u8>> {
+            (self.device_identities().contains(&device)
+                && offset == self.slot.to_device_offset()
+                && length == self.node_bytes.len())
+            .then(|| self.node_bytes.clone())
+        }
+        fn journal_record_offsets_hint(
+            &self,
+            _device: DeviceIdentity,
+            _ring_start: crate::address::DeviceOffsetInBytes,
+            _ring_bytes: u64,
+        ) -> Option<Vec<crate::address::DeviceOffsetInBytes>> {
+            unreachable!("打开文件那一趟只走 extent 树，不扫 journal 环")
+        }
+    }
+
+    /// `ExtentTreeReadsAtOpen::leaves` 只数真读到的叶：extent 上段的根是层级 1 的内部节点，只有罩着
+    /// inode 0–142 的那一片叶一个孩子；打开 inode 200（活着、size 0、没写过内容）时上段在根那一层就找不到罩着它的孩子、停在根，
+    /// 一片叶都没读——`leaves` 是 0，读了 1 个节点、这一路的高 1。改之前上段那一份恒记 1。
+    #[test]
+    fn opening_a_file_whose_upper_segment_stops_at_an_internal_node_counts_no_leaf() {
+        use crate::address::{CheckpointTxg, InstanceGeneration};
+        use crate::extent_tree::{
+            build_upper_internal_node, ExtentTreeNodeIdentity, ExtentUpperNodePosition,
+        };
+        use crate::pointer::{BirthSequence, PointerHead};
+        use crate::records::NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES;
+        use crate::root_record::UnmountMarker;
+
+        const EXTENT_TREE: TreeIdentifier = TreeIdentifier(65538);
+        const INODE_PAST_THE_FIRST_UPPER_LEAF: u64 = 200;
+        let filesystem_identifier = [7_u8; 16];
+        let instance = InstanceGeneration(1);
+        let birth_txg_of_the_extent_nodes = CheckpointTxg(3);
+        let upper_root_slot = SlotNumber(60_000);
+        let upper_root_birth_sequence = BirthSequence(4);
+        let pointer_head = PointerHead {
+            birth_tree: EXTENT_TREE,
+            birth_txg: birth_txg_of_the_extent_nodes,
+        };
+        let first_upper_leaf = ExtentUpperNodePosition { level: 0, index: 0 };
+        assert!(
+            first_upper_leaf.inode_range().1 < INODE_PAST_THE_FIRST_UPPER_LEAF,
+            "inode 200 不在最左那片上段叶罩的那一段里"
+        );
+        // 那片叶这一趟不读：它的指针只要解得开，位置与校验和随便。
+        let pointer_to_the_leaf_left_unread = NodePointer {
+            head: pointer_head,
+            locations: [DeviceIdentity(0), DeviceIdentity(1)].map(|device| LocationEntry {
+                device,
+                slot: SlotNumber(60_002),
+                unit_checksum: 0,
+            }),
+            instance,
+            birth_sequence: BirthSequence(3),
+        };
+        let upper_root_bytes = build_upper_internal_node(
+            &ExtentTreeNodeIdentity {
+                tree: EXTENT_TREE,
+                birth_txg: birth_txg_of_the_extent_nodes,
+                filesystem_identifier: &filesystem_identifier,
+                instance,
+            },
+            first_upper_leaf.parent(),
+            &[(first_upper_leaf, pointer_to_the_leaf_left_unread)],
+            upper_root_birth_sequence,
+        );
+        let extent_tree_root = NodePointer {
+            head: pointer_head,
+            locations: [DeviceIdentity(0), DeviceIdentity(1)].map(|device| LocationEntry {
+                device,
+                slot: upper_root_slot,
+                unit_checksum: crc32_castagnoli(&upper_root_bytes),
+            }),
+            instance,
+            birth_sequence: upper_root_birth_sequence,
+        };
+        let mounted = MountedPoolForRead {
+            root: RootRecord {
+                filesystem_identifier,
+                unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
+                instance,
+                checkpoint_txg: CheckpointTxg(5),
+                tree_table: NodePointer::empty_root(),
+                tree_identifier_watermark: singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS,
+                rollback_floor: CheckpointTxg(0),
+                instance_table: NodePointer::empty_root(),
+                mapping_root: NodePointer::empty_root(),
+                allocation_record_tree_root: NodePointer::empty_root(),
+            },
+            filesystem_identifier_in_unit_headers: unit_filesystem_identifier(
+                &filesystem_identifier,
+            ),
+            extent_tree: EXTENT_TREE,
+            extent_tree_root,
+            inode_records: vec![InodeRecord {
+                inode: INODE_PAST_THE_FIRST_UPPER_LEAF,
+                object_birth: birth_txg_of_the_extent_nodes,
+                nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
+                size: 0,
+                change_count: 1,
+                write_time_seconds: 1_788_000_000,
+            }],
+            central_mapping_entries: Vec::new(),
+            central_mapping_tree_reads_at_open: CentralMappingTreeReadsAtOpen {
+                node_reads: 1,
+                height: 1,
+            },
+            observation_since_open: Cell::new(ReadPathObservation::default()),
+            tree_node_stale_location_hint_hops_at_open: 0,
+        };
+        let reader = TwoDevicesHoldingOneNode {
+            slot: upper_root_slot,
+            node_bytes: upper_root_bytes,
+        };
+        let file = mounted
+            .open_file(&reader, InodeNumber(INODE_PAST_THE_FIRST_UPPER_LEAF))
+            .expect("上段停在根、没有这个 inode 的条目：一个数据单元都没有，size 0 对得上");
+        assert_eq!(file.data_unit_count(), 0, "没写过内容");
+        assert_eq!(
+            file.extent_tree_reads_at_open(),
+            ExtentTreeReadsAtOpen {
+                node_reads: 1,
+                height: 1,
+                leaves: 0,
+            },
+            "上段停在层级 1 的根：读了它一个节点，一片叶都没读"
+        );
+    }
 }
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index 0d822163..5c10af68 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -930,6 +930,8 @@ pub fn refusal_is_short_of_space(refusal: &PublishError) -> bool {
         // 一次发布切出来的记录多于上限：推抬 F 腾不出记录的位置，要把改动切小（代码审阅第 26 条）；txg 到顶是坏镜像（第 36 条）。
         | PublishError::JournalRecordsOfThePublishExceedTheLimit { .. }
         | PublishError::NextCheckpointTxgPastTheTopOfItsRange { .. }
+        // 墓碑树、谱系树或退役树最右那片的容器号已到顶（C841）：坏镜像，推抬 F 不会让号变小。
+        | PublishError::NextContainerNumberPastTheTopOfItsRange(_)
         // 写删除中的 inode、写已经删完的 inode、unlink 一个不是活记录的号：推抬 F 也不会让它复活。
         | PublishError::WriteToAnInodeBeingDeleted { .. }
         | PublishError::WriteToAnInodeThatIsNotInTheTree { .. }
diff --git a/crates/singlefs-core/src/pointer.rs b/crates/singlefs-core/src/pointer.rs
index deb3b6e1..6d12b9b3 100644
--- a/crates/singlefs-core/src/pointer.rs
+++ b/crates/singlefs-core/src/pointer.rs
@@ -67,7 +67,9 @@ pub fn slot_shared_by_both_location_entries(
 }
 
 /// 一条盘上指针的两条位置条目不按设备身份严格升序（I-2.5（位置条目按设备身份升序）：相同或逆序一律判损坏）。
-/// **这是一份坏镜像，不是我们的不变量被破坏**：写者（[`NodePointer::write_to`] / [`DataPointer::write_to`]）按升序写、在那里断言，
+/// 例外只有一个：整条全零的码 2 / 码 3 指针（[`NodePointer::empty_root`]，「注册了但还没有根」，两条位置条目都是盘 0）整条豁免，
+/// 读者（[`NodePointer::location_entries_ascend_by_device`]）与写者（[`NodePointer::write_to`]）同一个豁免；数据指针没有全零写法，不豁免。
+/// **这是一份坏镜像，不是我们的不变量被破坏**：写者（[`NodePointer::write_to`] / [`DataPointer::write_to`]）按严格升序写、在那里断言，
 /// 而照抄上一版的发布会把盘上读来的指针原样写回去（树表条目、多层码 2 树的内部条目、根记录）——不在读进来的地方判，
 /// 坏镜像就走到写者的断言上（C476（盘上内容可控时走得到的 panic 有 21 处） 普查「缺口」那张表 `pointer.rs` 那一行，实测坐实）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
@@ -321,8 +323,8 @@ impl NodePointer {
         let start = writer.position();
         self.head.write_to(writer);
         assert!(
-            self.locations[0].device <= self.locations[1].device,
-            "位置条目按设备身份升序（I-2.5）：盘上读来、要照抄写回的指针在 recovery::rebuild_version 判过"
+            self.location_entries_ascend_by_device().is_ok(),
+            "位置条目按设备身份严格升序（I-2.5），整条全零的空指针（empty_root）豁免：盘上读来、要照抄写回的指针在 recovery::rebuild_version 判过"
         );
         for location in &self.locations {
             location.write_to(writer);
@@ -396,8 +398,8 @@ impl DataPointer {
         let start = writer.position();
         self.head.write_to(writer);
         assert!(
-            self.locations[0].device <= self.locations[1].device,
-            "位置条目按设备身份升序（I-2.5）：盘上读来、要照抄写回的指针在 recovery::rebuild_version 判过"
+            location_entries_ascend_by_device(&self.locations).is_ok(),
+            "数据指针的位置条目按设备身份严格升序（I-2.5），没有全零豁免：盘上读来、要照抄写回的指针在 recovery::rebuild_version 判过"
         );
         for location in &self.locations {
             location.write_to(writer);
@@ -492,4 +494,65 @@ mod tests {
         );
         assert_eq!(NodePointer::empty_root().locations[1].slot, SlotNumber(0));
     }
+
+    /// 两条位置条目都在盘 1 的一条非空码 2 / 码 3 指针：两条设备身份相同，不是全零的空指针，写出时断言（I-2.5 严格升序）。
+    /// 改之前断言是 `<=`，同盘两条照写不误。
+    #[test]
+    #[should_panic(
+        expected = "位置条目按设备身份严格升序（I-2.5），整条全零的空指针（empty_root）豁免"
+    )]
+    fn a_non_empty_node_pointer_with_both_location_entries_on_one_device_is_refused_by_the_writer()
+    {
+        let on_device_one = LocationEntry {
+            device: DeviceIdentity(1),
+            slot: SlotNumber(50178),
+            unit_checksum: 0xAB,
+        };
+        let pointer = NodePointer {
+            head: PointerHead {
+                birth_tree: TreeIdentifier(3),
+                birth_txg: CheckpointTxg(7),
+            },
+            locations: [on_device_one, on_device_one],
+            instance: InstanceGeneration(1),
+            birth_sequence: BirthSequence(1),
+        };
+        let mut writer = ByteWriter::new(86);
+        pointer.write_to(&mut writer);
+    }
+
+    /// 整条全零的空指针（`empty_root`，mkfs 写进映射树根与分配记录树根的那一形）两条位置条目都是盘 0：照写，写出来 86 字节全 0。
+    #[test]
+    fn the_all_zero_empty_root_is_written_although_both_location_entries_name_device_zero() {
+        let mut writer = ByteWriter::new(86);
+        NodePointer::empty_root().write_to(&mut writer);
+        assert!(
+            writer.into_bytes().iter().all(|byte| *byte == 0),
+            "空指针写出来整条全 0"
+        );
+    }
+
+    /// 两条位置条目都在盘 0 的数据指针：数据指针没有全零豁免，写出时断言。
+    #[test]
+    #[should_panic(expected = "数据指针的位置条目按设备身份严格升序（I-2.5），没有全零豁免")]
+    fn a_data_pointer_with_both_location_entries_on_one_device_is_refused_by_the_writer() {
+        let on_device_zero = LocationEntry {
+            device: DeviceIdentity(0),
+            slot: SlotNumber(50200),
+            unit_checksum: 0xCD,
+        };
+        let pointer = DataPointer {
+            head: PointerHead {
+                birth_tree: TreeIdentifier(3),
+                birth_txg: CheckpointTxg(7),
+            },
+            locations: [on_device_zero, on_device_zero],
+            write_order: WriteOrder {
+                instance: InstanceGeneration(1),
+                transaction: 4,
+            },
+        };
+        let mut writer = ByteWriter::new(88);
+        pointer.write_to(&mut writer);
+    }
 }
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 0ea26180..37a742ed 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -52,7 +52,9 @@ use crate::extent_tree::{
     read_extent_tree, ExtentTreeHeaderJudgement, ExtentTreeReadFromDisk, ExtentTreeReading,
     ExtentTreeVersion, ExtentsOfAFileReadFromDisk,
 };
-use crate::inode_tree::{InodeLeafContainer, InodeLeafContainerIndexInTree};
+use crate::inode_tree::{
+    InodeLeafContainer, InodeLeafContainerIndexInTree, INODE_KEY_WIDTH_IN_BYTES,
+};
 use crate::instance_table::{
     InstanceTableChainRecord, InstanceTablePage, InstanceTablePageIndex, InstanceTableRecords,
 };
@@ -630,7 +632,7 @@ impl<'reader> CentralMappingTreeWithBytesReadOnFirstUse<'reader> {
 enum SystemConfigurationSlotReading {
     /// 三关都过、S 在区间里：这一槽可择。
     Mountable(SystemConfiguration),
-    /// 读不到，或 magic / 整槽校验和不过：这一槽不可择，换一槽换一盘还可以试。
+    /// 读不到，或 magic / 整槽校验和不过，或节点大小不是格式常量（当自证不过，C840）：这一槽不可择，换一槽换一盘还可以试。
     NotSelfDescribing,
     /// magic 与整槽校验和都过、incompat 位图不认识：这一槽同样不可择，记下它，池里一份可择的都没有时报布局不认识。
     IncompatBitsNotRecognized(IncompatBitmap),
@@ -955,9 +957,11 @@ fn mountable_slot_or_refusal(
                 system_configuration,
             ))
         }
-        Err(SystemConfigurationSlotRefusal::NotSelfDescribing) => {
-            Ok(SystemConfigurationSlotReading::NotSelfDescribing)
-        }
+        // 节点大小不是格式常量的槽当自证不过（C840，用户 2026-10-15 定）：与整槽校验和不过同一路，换一槽换一盘还可以试。
+        Err(
+            SystemConfigurationSlotRefusal::NotSelfDescribing
+            | SystemConfigurationSlotRefusal::NodeBytesNotTheFormatConstant { .. },
+        ) => Ok(SystemConfigurationSlotReading::NotSelfDescribing),
         Err(SystemConfigurationSlotRefusal::IncompatBitsNotRecognized { incompat_bitmap }) => Ok(
             SystemConfigurationSlotReading::IncompatBitsNotRecognized(incompat_bitmap),
         ),
@@ -2042,7 +2046,8 @@ pub fn every_device_reaches_the_unit_area_start(
 /// # Errors
 /// 树表读不到、解不开，同一种树有两条（[`TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE`]）；
 /// 分配记录树根提示读不出且映射里没有它（`MappingMiss`）、映射落点也读不出（`MappingStillUnreadable`）、
-/// 映射树根读不出或解不开、分配记录树根解不开；条目宽或结构值判红（见 [`allocation_records_of_node`]）。
+/// 映射树根读不出或解不开、分配记录树根解不开；条目宽判红（见 [`read_allocation_record_tree`]）或结构值判红
+/// （见 `allocation_records_fit_the_pool_geometry`）。
 ///
 /// 单元区从 `unit_area_start` 起（择到的系统配置按环长现算的那个，[`unit_area_start_of_the_chosen_system_configuration`]）：
 /// 分配记录落不落在单元区里按它判。
@@ -3832,7 +3837,7 @@ pub(crate) fn root_assembled_from_the_new_root_segment_of_the_record(
 fn key_width_for_kind(kind: u16) -> Option<usize> {
     match kind {
         TREE_KIND_EXTENT => Some(24),
-        TREE_KIND_INODE => Some(8),
+        TREE_KIND_INODE => Some(INODE_KEY_WIDTH_IN_BYTES),
         TREE_KIND_ALLOCATION => Some(10),
         TREE_KIND_ACCOUNTING => Some(22),
         // `deleted_inodes` 的 key 是 inode 号 8 字节（D8（核心索引结构） 已定项 16 第 1 条）；墓碑树与谱系树的根条目照 inode 树内部节点的
@@ -4449,12 +4454,20 @@ pub(crate) fn extents_of_the_first_file(
 /// 每盘不少于 `placements_on_each_device_at_least` 个落点（走读传的是这套几何下新池新建文件之后每盘的落点数，
 /// [`placements_on_each_device_after_the_new_pool_file_creation`]）。同盘同槽两条记录（代不同）在集合里是两个元素、两盘对称就过——第二轮攻方腿打中，
 /// 走读自己不判 key 严格递增，这里逐盘核槽号不重复。
+///
+/// # Panics
+/// `device_identities` 为空：调用方要交一张非空的盘表。产品调用方交的是读者的盘表（`PoolReader::device_identities`），
+/// 零块盘的池走到这里之前在读实例表单元那一步已经报错返回；空盘表在这里没有「每盘一条」可判，不交回 `true` 也不交回 `false`。
 #[must_use]
 pub fn allocation_records_are_one_per_device(
     records: &[AllocationRecord],
     device_identities: &[DeviceIdentity],
     placements_on_each_device_at_least: PlacementsOnEachDevice,
 ) -> bool {
+    assert!(
+        !device_identities.is_empty(),
+        "盘表不许为空：产品调用方交读者的盘表，零块盘的池在读实例表单元那一步已经报错返回"
+    );
     let mut placements_per_device: BTreeMap<DeviceIdentity, BTreeSet<(u64, u16, u64, bool)>> =
         BTreeMap::new();
     let mut slots_per_device: BTreeMap<DeviceIdentity, BTreeSet<u64>> = BTreeMap::new();
@@ -4484,7 +4497,9 @@ pub fn allocation_records_are_one_per_device(
         return false;
     }
     let mut placement_sets = placements_per_device.values();
-    let first_device_placements = placement_sets.next().expect("上面核过每块盘都有记录");
+    let first_device_placements = placement_sets
+        .next()
+        .expect("开头断言过盘表非空，上面又核过盘表里每块盘都有记录：至少有一块盘的落点集合");
     first_device_placements.len() >= placements_on_each_device_at_least.0
         && placement_sets.all(|placements| placements == first_device_placements)
 }
@@ -5932,6 +5947,14 @@ mod allocation_records_per_device_tests {
         );
     }
 
+    /// 记录与盘表都空：盘表非空是这个函数的前提（文档注释的 panic 条件），在开头那一句断言上 panic、消息说的就是这个前提。
+    /// 改之前走到取第一块盘的落点集合那一句 `expect`，消息写的「上面核过每块盘都有记录」在空输入上不成立（外部代码审查第一条）。
+    #[test]
+    #[should_panic(expected = "盘表不许为空")]
+    fn an_empty_device_table_with_no_records_panics_on_the_non_empty_device_table_precondition() {
+        let _ = allocation_records_are_one_per_device(&[], &[], PlacementsOnEachDevice(0));
+    }
+
     #[test]
     fn exactly_the_placements_of_the_default_geometry_are_accepted_and_one_fewer_is_rejected() {
         let placements_on_each_device = placements_on_each_device_of_the_default_geometry();
@@ -6294,6 +6317,7 @@ mod allocation_record_pool_geometry_tests {
 #[cfg(test)]
 mod tree_kind_key_width_tests {
     use super::key_width_for_kind;
+    use crate::inode_tree::INODE_KEY_WIDTH_IN_BYTES;
     use crate::records::{
         TREE_KIND_DELETED_INODES, TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_LOGGED_OPERATIONS,
         TREE_KIND_RETIREMENT, TREE_KIND_TOMBSTONE,
@@ -6327,6 +6351,13 @@ mod tree_kind_key_width_tests {
             None,
             "logged_ops 没有字段表"
         );
-        assert_eq!(key_width_for_kind(TREE_KIND_INODE), Some(8));
+        assert_eq!(
+            key_width_for_kind(TREE_KIND_INODE),
+            Some(INODE_KEY_WIDTH_IN_BYTES)
+        );
+        assert_eq!(
+            INODE_KEY_WIDTH_IN_BYTES, 8,
+            "inode 树的 key = inode 号 8 字节（D8 已定项 6）"
+        );
     }
 }
diff --git a/crates/singlefs-core/src/retirement_tree.rs b/crates/singlefs-core/src/retirement_tree.rs
index c44d1de7..569ef415 100644
--- a/crates/singlefs-core/src/retirement_tree.rs
+++ b/crates/singlefs-core/src/retirement_tree.rs
@@ -16,7 +16,7 @@ use singlefs_format::RETIREMENT_CONTAINER_RECORDS;
 use crate::address::{CheckpointTxg, TreeIdentifier};
 use crate::container_tree::{
     append_records_to_the_open_container, ContainerTreeAfterThisPublish, ContainerTreeNodeEntries,
-    ContainerTreeShape, RecordContainer,
+    ContainerTreeShape, NextContainerNumberPastTheTopOfItsRange, RecordContainer,
 };
 use crate::records::RetirementRecord;
 use crate::unit::{PackedIdentity, PACKED_TYPE_RETIREMENT};
@@ -119,7 +119,9 @@ pub fn retirement_records_of_this_publish(
 /// 往退役树的开放容器（最右那一片）追加这次发布的退役记录（删空摘掉、被左邻吸收的叶容器身份，各一条）：
 /// 装得下就追加进最右那片；装满了（或零根）就开一片新容器，容器号 = 树里最大号 + 1（零根时 1），出生代 = 这次发布的 checkpoint_txg，
 /// 出生树 = `retirement_tree`，打包记录类型 3；根满时长高。一条都没有时整棵照抄（这次不重写退役树）。
-#[must_use]
+///
+/// # Errors
+/// 要开新容器而最右那片的号（盘上读来的）已是 `u64::MAX` ⇒ [`NextContainerNumberPastTheTopOfItsRange`]（C841）。
 pub fn append_retirement_records_to_the_open_container(
     containers_before_this_publish: &[RetirementContainer],
     shape_before_this_publish: &ContainerTreeShape,
@@ -128,7 +130,7 @@ pub fn append_retirement_records_to_the_open_container(
     retirement_tree: TreeIdentifier,
     capacity: RetirementContainerCapacity,
     node_entries: ContainerTreeNodeEntries,
-) -> RetirementTreeAfterThisPublish {
+) -> Result<RetirementTreeAfterThisPublish, NextContainerNumberPastTheTopOfItsRange> {
     append_records_to_the_open_container(
         containers_before_this_publish,
         shape_before_this_publish,
@@ -187,7 +189,8 @@ mod tests {
                 RETIREMENT_TREE,
                 capacity,
                 node_entries,
-            );
+            )
+            .expect("容器号从 1 起逐片加一，远在顶之下");
         }
         after
     }
@@ -205,7 +208,8 @@ mod tests {
             RETIREMENT_TREE,
             RetirementContainerCapacity::FromTheContainerFormat,
             ContainerTreeNodeEntries::FromTheNodeFormat,
-        );
+        )
+        .expect("容器号从 1 起逐片加一，远在顶之下");
         let identity = PackedIdentity {
             birth_tree: RETIREMENT_TREE,
             record_type: PACKED_TYPE_RETIREMENT,
@@ -230,7 +234,8 @@ mod tests {
             RETIREMENT_TREE,
             RetirementContainerCapacity::FromTheContainerFormat,
             ContainerTreeNodeEntries::FromTheNodeFormat,
-        );
+        )
+        .expect("容器号从 1 起逐片加一，远在顶之下");
         assert_eq!(
             second.containers,
             vec![RetirementContainer {
@@ -259,7 +264,8 @@ mod tests {
             RETIREMENT_TREE,
             RetirementContainerCapacity::FromTheContainerFormat,
             ContainerTreeNodeEntries::FromTheNodeFormat,
-        );
+        )
+        .expect("容器号从 1 起逐片加一，远在顶之下");
         assert_eq!(after.containers, before.containers);
         assert!(!after.is_the_root_rewritten());
         assert!(after.rewritten_nodes().is_empty());
@@ -282,7 +288,8 @@ mod tests {
             RETIREMENT_TREE,
             capped(2),
             ContainerTreeNodeEntries::FromTheNodeFormat,
-        );
+        )
+        .expect("容器号从 1 起逐片加一，远在顶之下");
         assert_eq!(third.containers.len(), 2, "第一片满 2 条 ⇒ 开第二片");
         assert_eq!(third.containers[0], two.containers[0], "满了的那片原样");
         assert_eq!(third.containers[1].identity.container, 2, "最大号 1 + 1");
@@ -299,7 +306,8 @@ mod tests {
             RETIREMENT_TREE,
             capped(2),
             ContainerTreeNodeEntries::FromTheNodeFormat,
-        );
+        )
+        .expect("容器号从 1 起逐片加一，远在顶之下");
         assert_eq!(
             spanning
                 .containers
@@ -331,7 +339,8 @@ mod tests {
             RETIREMENT_TREE,
             capped(1),
             ContainerTreeNodeEntries::FromTheNodeFormat,
-        );
+        )
+        .expect("容器号从 1 起逐片加一，远在顶之下");
         assert_eq!(grown.shape.root_level(), 2, "第 136 片：根满长一层");
         assert!(grown.grew_a_level_this_publish());
         assert!(grown.previous_root_is_carried_down_unchanged());
@@ -354,7 +363,8 @@ mod tests {
             RETIREMENT_TREE,
             RetirementContainerCapacity::FromTheContainerFormat,
             ContainerTreeNodeEntries::FromTheNodeFormat,
-        );
+        )
+        .expect("容器号从 1 起逐片加一，远在顶之下");
         let second = append_retirement_records_to_the_open_container(
             &first.containers,
             &first.shape,
@@ -363,7 +373,8 @@ mod tests {
             RETIREMENT_TREE,
             RetirementContainerCapacity::FromTheContainerFormat,
             ContainerTreeNodeEntries::FromTheNodeFormat,
-        );
+        )
+        .expect("容器号从 1 起逐片加一，远在顶之下");
         assert_eq!(
             second.containers[0].records,
             vec![retired_leaf(234, 10), retired_leaf(234, 12)]
diff --git a/crates/singlefs-core/src/root_ring.rs b/crates/singlefs-core/src/root_ring.rs
index 0c651791..36296ad5 100644
--- a/crates/singlefs-core/src/root_ring.rs
+++ b/crates/singlefs-core/src/root_ring.rs
@@ -116,21 +116,6 @@ pub fn target_for_publish(
     }
 }
 
-/// 一圈窗口（D16（发布语义） 已定项 1「一圈窗口里最旧有效根」，用户 2026-10-02 定）：锚 N 之下的 txg 是不是落在窗口 txg > N − 3S 里
-/// （3S 是环长：R 个区域 × S 个槽）。写成 txg + 3S > N，N < 3S 时整个环都在窗口里、不减出负数。核心层判窗口的每一处
-/// （分配器的回收门槛、影子账的候选过滤、挂载时读不出的槽）都走它；锚取什么由调用方定（挂载时环里读得出的最新有效根，进程内这个进程记到的最新根）。
-#[must_use]
-pub fn txg_is_inside_the_one_revolution_window(
-    checkpoint_txg: CheckpointTxg,
-    window_anchor: CheckpointTxg,
-    slots_per_region: RootRingSlotsPerRegion,
-) -> bool {
-    checkpoint_txg
-        .0
-        .saturating_add(ROOT_RING_REGIONS * slots_per_region.count())
-        > window_anchor.0
-}
-
 /// 一个区域占多少字节：S 个槽 × 槽距。区域之间隔着 P × chunk，这一段之外的字节不归根环。
 /// mkfs 按它清根环、`ring_end` 按它算末端，两处同一个定义。
 #[must_use]
diff --git a/crates/singlefs-core/src/system_configuration.rs b/crates/singlefs-core/src/system_configuration.rs
index 33bcb38a..5f93868c 100644
--- a/crates/singlefs-core/src/system_configuration.rs
+++ b/crates/singlefs-core/src/system_configuration.rs
@@ -40,6 +40,9 @@ pub const CHECKSUM_ALGORITHM_CRC32_CASTAGNOLI: u8 = 1;
 /// 自举头：magic 4 + 格式版本 2 + feature bits 96 + fsid 16 + 写入者身份 20 + 校验和算法标识 1 + 本盘设备号 4 + 设备数 4 + 槽世代号 8。
 pub const SYSTEM_CONFIGURATION_CHECKSUM_OFFSET: usize = 4 + 2 + 96 + 16 + 20 + 1 + 4 + 4 + 8;
 const FSID_OFFSET: usize = 4 + 2 + 96;
+/// 几何段开头「节点大小」那 4 字节（字段表 `layout/01-first-txn.md` 一：加密预留 114 字节之后、单元大小之前）。
+/// 写侧 [`SystemConfiguration::to_slot`] 写它之前断言位置、读侧 [`SystemConfiguration::parse_slot`] 按它切。
+const NODE_BYTES_OFFSET: u64 = 301;
 /// 几何段里「每区槽数 S」那一字节（字段表 `layout/01-first-txn.md` 一：R 在 361、S 在 362）。
 /// 写侧 [`SystemConfiguration::to_slot`] 写它之前断言位置、读侧 [`SystemConfiguration::parse_slot`] 按它切，
 /// 一个偏移只有这一处定义。
@@ -253,10 +256,11 @@ impl SystemImmutableConfiguration {
 /// 系统可变配置：4 字节，改它 = 重建索引、不丢数据（D22（单元原子性怎么合成） 已定项 26 第二档）。
 ///
 /// 字段表里归这一档的只有 1 行：几何段的「节点大小 4」。
-/// **没有字段**：节点大小唯一的权威定义是格式常量 `NODE_BYTES`（`singlefs-format`，门禁 27 号按 kb 的
-/// format-const 标记钉住它的值），mkfs 今天不收它当参数——
+/// **没有字段**：节点大小唯一的权威定义是格式常量 `NODE_BYTES`（`singlefs-format`，门禁 code-source-discipline 的
+/// format-constants 格按 kb 的 format-const 标记钉住它的值），mkfs 今天不收它当参数——
 /// `grep -rn 'NODE_BYTES' crates/*/src` 命中的全是读，[`crate::make_filesystem::MakeFilesystemParameters`]
 /// 里没有这一项；改它要走的那条重建索引的路第一版也没有（已定项 26 第二档逐字「第一版没有这条路」）。
+/// 读者照样读回盘上那 4 字节，不等于格式常量就当这一槽自证不过（[`SystemConfigurationSlotRefusal::NodeBytesNotTheFormatConstant`]）。
 /// 真有了 mkfs 参数，值落在这里，格式常量退回去当默认值。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct SystemMutableConfiguration;
@@ -471,6 +475,7 @@ impl SystemConfiguration {
             writer.skip(80); // 主密钥槽：内联进槽，加密关时全 0
         });
         slot.write(SlotFieldMutability::MutableConfiguration, |writer| {
+            writer.assert_position(NODE_BYTES_OFFSET, "节点大小");
             writer.put_u32(self.mutable.node_bytes());
         });
         slot.write(SlotFieldMutability::ImmutableConfiguration, |writer| {
@@ -547,11 +552,13 @@ impl SystemConfiguration {
     /// 读者：magic、整槽校验和、incompat 位三关都过才解。magic 或整槽校验和不过是
     /// [`SystemConfigurationSlotRefusal::NotSelfDescribing`]（这一槽不可择）；这两关都过、incompat 位图不认识是
     /// [`SystemConfigurationSlotRefusal::IncompatBitsNotRecognized`]（这一槽同样不可择，但它是一份完整的系统配置，布局不认识）；
+    /// 布局认得、节点大小那 4 字节不等于格式常量 `NODE_BYTES` 是
+    /// [`SystemConfigurationSlotRefusal::NodeBytesNotTheFormatConstant`]（当这一槽自证不过，与整槽校验和不过同一个处置）；
     /// 解开之后按 D22 已定项 1 的字段表判每区槽数 S 的区间，越界是
     /// [`SystemConfigurationSlotRefusal::RootRingSlotsPerRegionOutOfRange`]（整池拒绝挂载）。
     ///
     /// # Errors
-    /// 见三个成员各自的说明。
+    /// 见四个成员各自的说明。
     pub fn parse_slot(bytes: &[u8]) -> Result<Self, SystemConfigurationSlotRefusal> {
         if bytes.len() < slot_bytes() || bytes[..4] != SYSTEM_CONFIGURATION_MAGIC {
             return Err(SystemConfigurationSlotRefusal::NotSelfDescribing);
@@ -564,6 +571,18 @@ impl SystemConfiguration {
                 incompat_bitmap: incompat_bitmap_of(bytes),
             });
         }
+        let recorded_node_bytes = ByteReader::at(
+            bytes,
+            usize::try_from(NODE_BYTES_OFFSET).expect("字段表里的偏移装得进 usize"),
+        )
+        .get_u32();
+        if u64::from(recorded_node_bytes) != NODE_BYTES {
+            return Err(
+                SystemConfigurationSlotRefusal::NodeBytesNotTheFormatConstant {
+                    recorded_node_bytes,
+                },
+            );
+        }
         let root_ring_slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(
             u64::from(bytes[usize::try_from(ROOT_RING_SLOTS_PER_REGION_OFFSET).expect("362")]),
         )
@@ -626,8 +645,8 @@ impl SystemConfiguration {
                 journal_ring_start_slot,
                 root_ring_base_slot,
             },
-            // 节点大小那 4 字节不读回来：它今天只可能是格式常量 `NODE_BYTES`，读回来会多出一个
-            // 「盘上的值与格式常量不符」的状态而没人判它。有了 mkfs 参数那天连着判定一起加。
+            // 节点大小那 4 字节上面判过等于格式常量 `NODE_BYTES`，内存里不另存一份；有了 mkfs 参数那天值落进这一档、
+            // 判定连着换成判它在 mkfs 收的范围里（C840，用户 2026-10-15 定「当自证不过」）。
             mutable: SystemMutableConfiguration,
             // 可调值那 36 字节不读回来：挂载选项通道不存在（C422（挂载选项通道不存在）），
             // 读回来的值没有任何使用者。
@@ -654,6 +673,11 @@ pub enum SystemConfigurationSlotRefusal {
     /// 布局不是这个读者认得的那一版（退役的位 0 那一版旧镜像，或更新的实现写的，D15（格式冻结政策） 已定项 4）。
     /// 与 [`Self::NotSelfDescribing`] 分开报，调用方据此把「布局不认识」与「数据坏了」分开。带着那一槽的 incompat 位图原样。
     IncompatBitsNotRecognized { incompat_bitmap: IncompatBitmap },
+    /// magic、整槽校验和、incompat 位三关都过，而节点大小那 4 字节（偏移 301）不等于格式常量 `NODE_BYTES`
+    /// ⇒ **当这一槽自证不过**：这一槽不可择，换一槽、换一盘还可以试，与 [`Self::NotSelfDescribing`] 同一个处置
+    /// （C840，用户 2026-10-15 弹窗定「当自证不过」）。单列成员是为了报得出拒在节点大小这一格、带着盘上读到的值；
+    /// 节点照格式常量读，不收别的节点大小（D22（单元原子性怎么合成） 已定项 26 第二档：改节点大小的路第一版没有）。
+    NodeBytesNotTheFormatConstant { recorded_node_bytes: u32 },
     /// 自述的每区槽数 S 落在格式承诺的区间之外 ⇒ **整池拒绝挂载**，不换一槽再试：S 是池级字段，
     /// 两盘四槽写的是同一个值，换一槽读到的还是它（D22（单元原子性怎么合成） 已定项 9 的射程：
     /// 字段表里只有本盘设备号是盘级的）。
@@ -912,6 +936,45 @@ mod tests {
         }
     }
 
+    /// 节点大小那 4 字节（偏移 301）改成别的值、整槽校验和重封（magic、校验和、incompat 三关照样过）：读者拒成点名节点大小的成员，
+    /// 带着盘上读到的值；写回格式常量的那一份照收（C840）。
+    #[test]
+    fn self_describing_slot_declaring_a_node_size_other_than_the_format_constant_is_refused_by_name(
+    ) {
+        let node_bytes_field = usize::try_from(NODE_BYTES_OFFSET).expect("301");
+        let mut slot = sample().to_slot();
+        assert_eq!(
+            u32::from_le_bytes(
+                slot[node_bytes_field..node_bytes_field + 4]
+                    .try_into()
+                    .expect("4 字节")
+            ),
+            16384,
+            "写者在偏移 301 写格式常量 NODE_BYTES"
+        );
+        slot[node_bytes_field..node_bytes_field + 4].copy_from_slice(&8192u32.to_le_bytes());
+        let digest = wide_checksum_with_field_zeroed(
+            &slot,
+            slot_bytes(),
+            SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
+        );
+        slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
+            .copy_from_slice(&digest);
+        assert!(
+            wide_checksum_field_holds(&slot, 4096, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET),
+            "整槽校验和重封过，拒它的只能是节点大小那一判"
+        );
+        assert_eq!(
+            SystemConfiguration::parse_slot(&slot),
+            Err(
+                SystemConfigurationSlotRefusal::NodeBytesNotTheFormatConstant {
+                    recorded_node_bytes: 8192,
+                }
+            ),
+            "节点大小 8192 不是格式常量：拒成点名节点大小的成员"
+        );
+    }
+
     #[test]
     fn slot_spacing_rounds_4096_up_to_a_multiple_of_minimum_input_output_size() {
         assert_eq!(SystemImmutableSizes::slot_spacing_for(512), 4096);
diff --git a/crates/singlefs-core/src/tombstone_tree.rs b/crates/singlefs-core/src/tombstone_tree.rs
index b3212463..9cceba20 100644
--- a/crates/singlefs-core/src/tombstone_tree.rs
+++ b/crates/singlefs-core/src/tombstone_tree.rs
@@ -16,8 +16,8 @@ use singlefs_format::TOMBSTONE_CONTAINER_RECORDS;
 
 use crate::address::{CheckpointTxg, TreeIdentifier};
 use crate::container_tree::{
-    append_record_to_the_open_container, containers_one_container_tree_root_holds,
-    ContainerTreeAfterThisPublish, ContainerTreeNodeEntries, ContainerTreeShape, RecordContainer,
+    append_record_to_the_open_container, ContainerTreeAfterThisPublish, ContainerTreeNodeEntries,
+    ContainerTreeShape, NextContainerNumberPastTheTopOfItsRange, RecordContainer,
 };
 use crate::records::TombstoneRecord;
 use crate::unit::PACKED_TYPE_TOMBSTONE;
@@ -100,12 +100,6 @@ impl TombstoneContainerCapacity {
     }
 }
 
-/// 一个码 2 节点按产品格式装得下几条墓碑树条目：与 inode 树内部节点同一形态（分隔 key 8 + 身份引用 26 + 子指针 86 = 120），扇出 135。
-#[must_use]
-pub fn tombstone_containers_one_root_node_holds() -> usize {
-    containers_one_container_tree_root_holds()
-}
-
 /// 这次发布之后墓碑树的样子：叶容器、这次重写哪几片（叶序）、这次前后的形状（`crate::container_tree`）。
 pub type TombstoneTreeAfterThisPublish = ContainerTreeAfterThisPublish<TombstoneRecord>;
 
@@ -113,7 +107,9 @@ pub type TombstoneTreeAfterThisPublish = ContainerTreeAfterThisPublish<Tombstone
 /// 最右那片还装得下（记录数小于 `capacity`）就追加进去；装满了（或树是空的）就开一片新容器：
 /// 容器号 = 树里最大号 + 1（零根时 1），出生代 = 这次发布的 checkpoint_txg，出生树 = `tombstone_tree`，打包记录类型 1；
 /// 根满时长高（`node_entries` 是一个码 2 节点装满几条）。别的容器一个字节都不动。
-#[must_use]
+///
+/// # Errors
+/// 要开新容器而最右那片的号（盘上读来的）已是 `u64::MAX` ⇒ [`NextContainerNumberPastTheTopOfItsRange`]（C841）。
 pub fn append_tombstone_to_the_open_container(
     containers_before_this_publish: &[TombstoneContainer],
     shape_before_this_publish: &ContainerTreeShape,
@@ -122,7 +118,7 @@ pub fn append_tombstone_to_the_open_container(
     tombstone_tree: TreeIdentifier,
     capacity: TombstoneContainerCapacity,
     node_entries: ContainerTreeNodeEntries,
-) -> TombstoneTreeAfterThisPublish {
+) -> Result<TombstoneTreeAfterThisPublish, NextContainerNumberPastTheTopOfItsRange> {
     append_record_to_the_open_container(
         containers_before_this_publish,
         shape_before_this_publish,
@@ -140,7 +136,8 @@ mod tests {
     use super::*;
     use crate::address::InodeNumber;
     use crate::container_tree::{
-        ContainerTreeNodeEntriesCappedForTests, ContainerTreeNodePosition,
+        containers_one_container_tree_root_holds, ContainerTreeNodeEntriesCappedForTests,
+        ContainerTreeNodePosition,
     };
     use crate::unit::PackedIdentity;
     use singlefs_format::TREE_IDENTIFIER_TOMBSTONE;
@@ -178,7 +175,8 @@ mod tests {
                 TOMBSTONE_TREE,
                 capacity,
                 node_entries,
-            );
+            )
+            .expect("容器号从 1 起逐片加一，远在顶之下");
         }
         after
     }
@@ -194,7 +192,8 @@ mod tests {
             TOMBSTONE_TREE,
             TombstoneContainerCapacity::FromTheContainerFormat,
             ContainerTreeNodeEntries::FromTheNodeFormat,
-        );
+        )
+        .expect("容器号从 1 起逐片加一，远在顶之下");
         assert_eq!(
             after.containers,
             vec![TombstoneContainer {
@@ -224,7 +223,8 @@ mod tests {
             TOMBSTONE_TREE,
             capped(2),
             ContainerTreeNodeEntries::FromTheNodeFormat,
-        );
+        )
+        .expect("容器号从 1 起逐片加一，远在顶之下");
         assert_eq!(after.containers.len(), 1, "压到 2 条一片：第二条还装得下");
         assert_eq!(
             after.containers[0].identity, first.containers[0].identity,
@@ -250,7 +250,8 @@ mod tests {
             TOMBSTONE_TREE,
             capped(2),
             ContainerTreeNodeEntries::FromTheNodeFormat,
-        );
+        )
+        .expect("容器号从 1 起逐片加一，远在顶之下");
         assert_eq!(after.containers.len(), 2, "第一片满 2 条 ⇒ 开第二片");
         assert_eq!(after.containers[0], two.containers[0], "满了的那片原样");
         assert_eq!(after.containers[1].identity.container, 2, "最大号 1 + 1");
@@ -266,7 +267,7 @@ mod tests {
     /// 长高那次旧根留在 (层级 1, 第 0 个) 照抄，新开的容器、右半 (1, 1) 与新根 (2, 0) 重写。
     #[test]
     fn opening_a_container_past_what_one_root_holds_grows_the_tree_one_level() {
-        let capacity_of_the_root = tombstone_containers_one_root_node_holds();
+        let capacity_of_the_root = containers_one_container_tree_root_holds();
         assert_eq!(capacity_of_the_root, 135, "与 inode 树根同一形态：扇出 135");
         let full_root =
             after_appending(135, capped(1), ContainerTreeNodeEntries::FromTheNodeFormat);
@@ -279,7 +280,8 @@ mod tests {
             TOMBSTONE_TREE,
             capped(1),
             ContainerTreeNodeEntries::FromTheNodeFormat,
-        );
+        )
+        .expect("容器号从 1 起逐片加一，远在顶之下");
         assert_eq!(grown.shape.root_level(), 2, "第 136 片：根满长一层");
         assert!(grown.grew_a_level_this_publish());
         let node_entries_of_two = ContainerTreeNodeEntries::CappedForTests(
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 43b23bf5..1f45a27e 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -1,6 +1,8 @@
 //! 事务层（里程碑步 3 / 步 4 / 步 5）：一个共享的提交状态机 + 一个封闭的提交步骤枚举
 //! （D17（实现分层与第三方管道） 已定项 2；`.claude/rules/fs-design.md`「一个事务层，所有结构共用」）。
-//! 三条路径都从同一个枚举走：取号 = 逐盘一次系统配置槽写 + 一道屏障（D23（journal 的角色与格式） 已定项 16，C322（取号那一步的屏障怎么放没有条款） 2026-09-14 定案）；
+//! 暖机与发布的每一步都从同一个枚举走；取号只有屏障经它，取号的系统配置槽写走私有的 `PoolWriter::write_system_configuration_slot`：
+//! 取号 = 屏障 → 逐盘一次系统配置槽写 → 屏障（D23（journal 的角色与格式） 已定项 16，C322（取号那一步的屏障怎么放没有条款） 2026-09-14 定案；
+//! 写之前那道是代码审阅第 19 条加的）；
 //! 暖机（D16（发布语义） 已定项 8）= 屏障 → 空记录 → 屏障 → 根槽 FUA → 系统配置槽轮换 → 屏障；
 //! 新池新建文件（D16（发布语义） 已定项 7）= 单元写 × 12（默认几何）→ 屏障 → journal 记录 → 屏障 → 根槽 FUA → 系统配置槽轮换 → 屏障。
 //! 覆盖写（里程碑「覆盖写、释放、回退与复用」步 1 / 步 2）走同一条骨架：新数据单元 COW 到新落点、十个提交内生块与树表 COW 出新版本，
@@ -56,7 +58,8 @@ use crate::code_two_tree::{
 };
 use crate::container_tree::{
     ContainerTreeAfterThisPublish, ContainerTreeCapacities, ContainerTreeNodeOrigin,
-    ContainerTreeNodePosition, ContainerTreeShape, RecordContainer, IDENTITY_OF_A_CODE_TWO_CHILD,
+    ContainerTreeNodePosition, ContainerTreeShape, NextContainerNumberPastTheTopOfItsRange,
+    RecordContainer, IDENTITY_OF_A_CODE_TWO_CHILD,
 };
 use crate::deleted_inodes::{
     deleted_inodes_root_entries, DeletedInodesQueueRefusal, DELETED_INODES_KEY_WIDTH_IN_BYTES,
@@ -70,7 +73,7 @@ use crate::extent_tree::{
 use crate::inode_tree::{
     remove_record_from_leaf_containers, write_records_into_leaf_containers, InodeLeafContainer,
     InodeLeafContainerIndexInTree, InodeLeafContainerOrigin, InodeLeafContainersAfterThisPublish,
-    InodeTreeWriteRefusal,
+    InodeTreeWriteRefusal, INODE_KEY_WIDTH_IN_BYTES,
 };
 use crate::instance_table::{
     instance_table_page_records, instance_table_pages_for_rows, instance_table_rows_of_each_page,
@@ -665,7 +668,8 @@ impl<Device: BlockDevice> PoolWriter<'_, Device> {
 pub enum AcquisitionRollback {
     /// 第一块盘的取号写就报错了：没有写出过带新号的系统配置，没有要回卷的。
     NothingWritten,
-    /// 已写出的那几份都回卷成了旧代号；这次挂载只读，盘上若还留着新号，下一次取号跳过它。
+    /// 已写出的那几份都发了回卷写（写回旧代号）、设备都报了成功；回卷写之后没有屏障，返回时它们未必已持久。
+    /// 这次挂载只读，盘上若还留着新号（回卷写没落盘也是这样），下一次取号跳过它。
     RolledBack,
     /// 回卷写或回卷写之前那道屏障报错：只读挂载。
     RollbackFailed(BlockDeviceError),
@@ -3990,6 +3994,10 @@ pub enum PublishError {
     /// 要建在上面的那一版的 checkpoint_txg 已是 `u64::MAX`，这次发布的 txg（它加一）装不下（代码审阅第 36 条）：
     /// 那一版的 txg 是盘上读来的 8 字节，坏镜像、外来镜像才走得到。在任何读写之前返回，盘上逐字节不变。
     NextCheckpointTxgPastTheTopOfItsRange { version_to_build_on: CheckpointTxg },
+    /// 这次发布要在墓碑树、谱系树或退役树里开一片新容器，而那棵树最右那片的号（盘上读来的 8 字节）已是 `u64::MAX`，下一个号装不下
+    /// （C841，用户 2026-10-15 定「写路径报类型化错误」）：坏镜像、外来镜像才走得到。在算这次发布的样子那一步交回
+    /// （[`PublishPlan::resolve`] 与它之前的普通分配那一半），分配器、块设备都还没被碰过，盘上逐字节不变。推抬 F 不会让号变小，不是空间不够。
+    NextContainerNumberPastTheTopOfItsRange(NextContainerNumberPastTheTopOfItsRange),
     /// 覆盖写、顺序写要写的那个 inode 在上一版里是删除中的（nlink = 0：unlink 的意图已经发布，D8（核心索引结构） 已定项 16 第 4 条
     /// 「写入口拒 nlink = 0 的 inode」）：从意图那一版起它对读者已经是「没有文件」，再写一版就把 nlink 写回 1、复活一个正在回收的对象。
     /// 在任何读写之前返回，盘上逐字节不变。
@@ -4469,87 +4477,6 @@ pub fn tombstone_tree_internal_levels_read_from_the_root_node_header(
     }
 }
 
-/// 一次发布重写哪些角色，只由三件事定：这次写不写文件内容（数据单元 + extent 树根）、重写几片 inode 叶容器、
-/// 实例表是重写还是照抄。计划的身份字段（新实例代号、表的字节、txg）都不进来——可写挂载要在取号之前算写行那次发布的准入，
-/// 而那时新实例代号还没取（增补 2 第 20a 行）。
-///
-/// 映射条目那一条准入要的是**这次之后 inode 树一共几片叶容器**（映射节点每次发布整片重写，照抄的那几片也各占一条），
-/// 形状答不了它——「重写几片」与「一共几片」是两个数。那个数各条路径自己给：发布路径从 `PublishPlan::resolve`
-/// 算出来的树取；取号之前那一串走的是同一条发布路径（`mount` 的 `dry_run_of_the_publishes_after_acquisition`），也从那里取。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
-pub struct PublishShape {
-    /// 这次重写几个数据单元；0 = 这次不写文件内容（数据单元与 extent 树根照抄上一版）。
-    pub rewritten_data_units: usize,
-    /// 这次重写几片 inode 叶容器；0 = 这次一个字节都不碰 inode 树（连根都不重写）。
-    pub rewritten_inode_leaf_containers: usize,
-    /// 这次重写的实例表链有几片；0 = 实例表照抄。
-    pub rewritten_instance_table_pages: usize,
-}
-
-impl PublishShape {
-    /// 写行那次发布、实例表链这次之后只有一片时的形状（行数不超过 369，新池新建文件之后的多数挂载都是它）：
-    /// 多于一片的见 [`PublishShape::row_publish_rewriting_instance_table_pages`]。
-    pub const ROW_PUBLISH: PublishShape =
-        PublishShape::row_publish_rewriting_instance_table_pages(1);
-
-    /// 写行那次发布的形状：不写文件内容、不碰 inode 树、实例表整条链重写成 `instance_table_pages` 片
-    /// （`publish_rows_on_file_version` 交给 `publish_version` 的那张计划同形）。
-    #[must_use]
-    pub const fn row_publish_rewriting_instance_table_pages(instance_table_pages: usize) -> Self {
-        PublishShape {
-            rewritten_data_units: 0,
-            rewritten_inode_leaf_containers: 0,
-            rewritten_instance_table_pages: instance_table_pages,
-        }
-    }
-
-    /// 暖机那几次空发布的形状：不写文件内容、不碰 inode 树、实例表照抄（`publish_empty_after` 交给 `publish_version` 的那张计划同形，
-    /// 那里有一条断言钉住两者相等）。记账树已经存在 ⇒ 照样重写固定点单元（D16（发布语义） 已定项 9；新池几何下分配记录树 5 个节点 + 记账 + 映射 + 树表 = 8 个，字节表八）。
-    pub const EMPTY_PUBLISH: PublishShape = PublishShape {
-        rewritten_data_units: 0,
-        rewritten_inode_leaf_containers: 0,
-        rewritten_instance_table_pages: 0,
-    };
-
-    /// 这次发布重写的角色，按 bump 次序：用户数据先取（自己的政策）；提交内生块按树 ID 升序、树内先叶后根、映射树倒数第二、树表最末
-    /// （D3（空间分配） 已定项 10 ⑤）。实例表单元归树 0，排在全部提交内生块之前（mkfs 也是先实例表后树表；里程碑步 3 的决策点），
-    /// 多于一片时尾片先（[`instance_table_page_roles_in_bump_order`]）。
-    ///
-    /// ⚠️ **叶容器在这里按 0 起的连号排，不是它们在树里的真实叶序**：形状只答「几个角色」，
-    /// 答不了「改的是哪几片」——那要看上一版的树长什么样。
-    /// 真实的角色清单由 [`PublishPlan::rewritten_roles`] 给。
-    #[must_use]
-    pub fn rewritten_roles(self) -> Vec<TransactionUnit> {
-        let mut roles = Vec::new();
-        for position in 0..self.rewritten_data_units {
-            roles.push(TransactionUnit::Data(DataUnitIndexInFile(
-                u64::try_from(position).expect("单元序号"),
-            )));
-        }
-        roles.extend(instance_table_page_roles_in_bump_order(
-            self.rewritten_instance_table_pages,
-        ));
-        if self.rewritten_data_units > 0 {
-            roles.push(TransactionUnit::ExtentRoot);
-        }
-        for position in 0..self.rewritten_inode_leaf_containers {
-            roles.push(TransactionUnit::InodeLeafContainer(
-                InodeLeafContainerIndexInTree::of_position(position),
-            ));
-        }
-        if self.rewritten_inode_leaf_containers > 0 {
-            roles.push(TransactionUnit::InodeRoot);
-        }
-        roles.extend([
-            TransactionUnit::AllocationTree,
-            TransactionUnit::AccountingTree,
-            TransactionUnit::MappingTree,
-            TransactionUnit::TreeTable,
-        ]);
-        roles
-    }
-}
-
 /// 一次发布在动分配器、动盘之前先算完的东西：这次之后 inode 树是哪几片叶容器、这次重写哪些角色（按 bump 次序）。
 /// 只读——算它不发一个写、不动分配器，所以条款没写的那几格（`crate::inode_tree`）在这里交回时盘上逐字节不变。
 #[derive(Clone, Debug)]
@@ -4586,24 +4513,6 @@ pub struct ResolvedPublish {
 }
 
 impl ResolvedPublish {
-    /// 这次发布的形状（准入按它数角色）。
-    #[must_use]
-    pub fn shape(&self) -> PublishShape {
-        PublishShape {
-            rewritten_data_units: self
-                .rewritten_roles
-                .iter()
-                .filter(|identity| matches!(identity, TransactionUnit::Data(_)))
-                .count(),
-            rewritten_inode_leaf_containers: self.inode_tree.rewritten.len(),
-            rewritten_instance_table_pages: self
-                .rewritten_roles
-                .iter()
-                .filter(|identity| identity.is_a_page_of_the_instance_table())
-                .count(),
-        }
-    }
-
     /// 一棵多层码 2 树这次之后的计划。
     #[must_use]
     pub fn multi_level_tree(&self, tree: MultiLevelCodeTwoTree) -> &CodeTwoTreePlan {
@@ -4871,21 +4780,6 @@ impl ExtentTreePlan {
             )
             .collect()
     }
-
-    /// 这一版全部 extent 树角色，按 bump 次序。
-    #[must_use]
-    pub fn roles(&self) -> Vec<TransactionUnit> {
-        let upper_root_level = self.upper_root_level();
-        self.lower_nodes
-            .iter()
-            .map(|(position, _)| TransactionUnit::ExtentLowerNode(*position))
-            .chain(
-                self.upper_nodes
-                    .iter()
-                    .map(|(position, _)| role_of_extent_upper_node(*position, upper_root_level)),
-            )
-            .collect()
-    }
 }
 
 /// 这次发布之后的 extent 树。写文件内容的发布：第一个文件有 `data_units` 个单元（从 0 起连号、没有洞）——一个就内联、不建下段，
@@ -5290,7 +5184,8 @@ impl PublishPlan<'_> {
     /// [`prepare_the_version_publish`] 在走分配记录树的固定点之前判「这次的单元落得下」也读它。
     ///
     /// # Errors
-    /// inode 记录的落法要的条款仓里还没定 ⇒ `InodeTreeWriteRefused`（`crate::inode_tree` 的三格）。
+    /// inode 记录的落法要的条款仓里还没定 ⇒ `InodeTreeWriteRefused`（`crate::inode_tree` 的三格）；
+    /// 墓碑树或退役树要开新容器而最右那片的号已到顶 ⇒ `NextContainerNumberPastTheTopOfItsRange`（C841）。
     fn ordinary_allocation_part(
         &self,
         previous: Option<&TransactionOutput>,
@@ -5406,6 +5301,7 @@ impl PublishPlan<'_> {
                         .tombstone_container,
                     deletion_and_lineage.container_tree_capacities.node_entries,
                 )
+                .map_err(PublishError::NextContainerNumberPastTheTopOfItsRange)?
             }
             DeletionStepOfThePublish::Batch { .. }
             | DeletionStepOfThePublish::FinalBatch { .. }
@@ -5440,7 +5336,8 @@ impl PublishPlan<'_> {
                 .container_tree_capacities
                 .retirement_container,
             deletion_and_lineage.container_tree_capacities.node_entries,
-        );
+        )
+        .map_err(PublishError::NextContainerNumberPastTheTopOfItsRange)?;
         Ok(OrdinaryAllocationPartOfThePublish {
             inode_tree,
             extent_tree,
@@ -5462,7 +5359,8 @@ impl PublishPlan<'_> {
     ///
     /// # Errors
     /// inode 记录的落法要的条款仓里还没定 ⇒ `InodeTreeWriteRefused`（`crate::inode_tree` 的三格）；
-    /// 记账树或中央映射树这次之后的形状算不出来 ⇒ `MultiLevelCodeTwoTreeRefused`。
+    /// 记账树或中央映射树这次之后的形状算不出来 ⇒ `MultiLevelCodeTwoTreeRefused`；
+    /// 墓碑树、谱系树或退役树要开新容器而最右那片的号已到顶 ⇒ `NextContainerNumberPastTheTopOfItsRange`（C841）。
     pub fn resolve(
         &self,
         previous: Option<&TransactionOutput>,
@@ -5476,7 +5374,7 @@ impl PublishPlan<'_> {
         let deletion_roles_rewritten = ordinary_part.deletion_roles_rewritten();
         let retirement_roles_rewritten = ordinary_part.retirement_roles_rewritten();
         let lineage_tree =
-            self.lineage_tree_after_this_publish(previous, trees, deletion_and_lineage);
+            self.lineage_tree_after_this_publish(previous, trees, deletion_and_lineage)?;
         let previous_roles_without_a_successor = previous
             .map(|previous_version| {
                 ordinary_part
@@ -5581,12 +5479,15 @@ impl PublishPlan<'_> {
     }
 
     /// 这次之后谱系树的样子：管理员回退那次往开放容器追加那一条（容器号同墓碑规则，根满时长高），别的发布照抄上一版。
+    ///
+    /// # Errors
+    /// 要开新容器而最右那片的号已到顶 ⇒ `NextContainerNumberPastTheTopOfItsRange`（C841）。
     fn lineage_tree_after_this_publish(
         &self,
         previous: Option<&TransactionOutput>,
         trees: &FileVersionTreeIdentifiers,
         deletion_and_lineage: &DeletionAndLineageOfThePublish,
-    ) -> LineageTreeAfterThisPublish {
+    ) -> Result<LineageTreeAfterThisPublish, PublishError> {
         let containers_before: Vec<LineageContainer> = previous
             .map(|version| {
                 version
@@ -5612,10 +5513,12 @@ impl PublishPlan<'_> {
                         .lineage_container,
                     deletion_and_lineage.container_tree_capacities.node_entries,
                 )
+                .map_err(PublishError::NextContainerNumberPastTheTopOfItsRange)
             }
-            LineageStepOfThePublish::NoLineageRecord => {
-                LineageTreeAfterThisPublish::carried(containers_before, shape_before)
-            }
+            LineageStepOfThePublish::NoLineageRecord => Ok(LineageTreeAfterThisPublish::carried(
+                containers_before,
+                shape_before,
+            )),
         }
     }
 
@@ -6099,7 +6002,7 @@ pub fn publish_sequential_write<Device: BlockDevice>(
     let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
     // 改动计数与新池新建文件号先取成局部量、字段也换了次序：`crates/mutations.tsv` 有几条变异按 `publish_overwrite` 里
     // `change_count: txg.0,`、`transaction: previous.highest_transaction_number_in_this_instance + 1,` 与那几行字段的原样
-    // 锚着，同一份文件里再出现一次它们就腐化（门禁 59 号；`publish_new_inodes` 里那句注释是同一件事）。
+    // 锚着，同一份文件里再出现一次它们就腐化（门禁 checker-tier-crates-mutation-replay 判锚点恰好命中一次；`publish_new_inodes` 里那句注释是同一件事）。
     let change_count = txg.0;
     let new_pool_file_creation_of_this_request =
         previous.highest_transaction_number_in_this_instance + 1;
@@ -6163,7 +6066,7 @@ pub fn publish_new_inodes<Device: BlockDevice>(
         .watermark_this_publish_starts_from(first_new_inode_number);
     // 改动计数与事务号都先取成局部量：`crates/mutations.tsv` 有几条变异按 `change_count: txg.0,` 与
     // `transaction: previous.highest_transaction_number_in_this_instance + 1,` 这两串字面锚在 `publish_overwrite` 上，
-    // 同一份文件里再出现一次它们就腐化（门禁 59 号；`publish_first_file` 里那句注释是同一件事）。
+    // 同一份文件里再出现一次它们就腐化（门禁 checker-tier-crates-mutation-replay 判锚点恰好命中一次；`publish_sequential_write` 里那句注释是同一件事）。
     // 改动计数 = 最后一次改动所在发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88）；建出来那次就是这次发布。
     let change_count = txg.0;
     let transaction_number = previous.highest_transaction_number_in_this_instance + 1;
@@ -6497,7 +6400,7 @@ fn publish_version_of_trees_holding_one_data_unit<Device: BlockDevice>(
     )
 }
 
-/// 发布一版：先做准入（extent 树装得下这个文件的数据单元），
+/// 发布一版：先做空间准入（这次的单元落不落得下，再是 D28（挂载期承诺量） 已定项 1 的式子那一判，[`prepare_the_version_publish`]），
 /// 再释放上一版被换下的角色的落点、分配、装单元、
 /// 按 D16（发布语义） 已定项 7 的持久顺序落盘。准入之后任何一步失败，分配器退回到进来时的样子——这次发布没有成立，
 /// 释放与分配都不算数（第二轮攻方腿：落点被拒（当时叫 `NoSpaceFor`）在释放之后、分配到一半返回，留下半新的池，拿同一个上一版重试撞断言）；
@@ -6508,8 +6411,11 @@ fn publish_version_of_trees_holding_one_data_unit<Device: BlockDevice>(
 /// （回退到树表 0 条的一版之后）走 `publish_first_file`，它按要建在上面的那一版的水位发。
 ///
 /// # Errors
-/// `InodeTreeWriteRefused`、`ContentExceedsDataUnit`、`MultiLevelCodeTwoTreeRefused`、
-/// `AllocationRecordTreeRewriteSetDidNotSettle`、释放判定路径的四种错、`PlacementRefused`、块设备错。
+/// 冻结着一次没重发的发布 ⇒ `PublishFrozenAfterAWriteFailureIsNotResentYet`；`InodeTreeWriteRefused`、`DeletedInodesQueueRefused`、
+/// `SpaceAdmissionRefused`、`MultiLevelCodeTwoTreeRefused`、`AllocationRecordTreeRewriteSetDidNotSettle`、
+/// `JournalRecordsOfThePublishExceedTheLimit`、释放判定路径与释放之前读盘核的几种错（`Release` 与 `MappingEntry` 起头的那几个成员）、
+/// `PlacementRefused`、块设备错。内容装不进一个数据单元（`ContentExceedsDataUnit`）不由它报：按一个数据单元写的
+/// [`publish_first_file`]、[`publish_overwrite`] 在走发布本体之前自己判。
 ///
 /// # Panics
 /// 没有上一版、而计划写的新水位不是 mkfs 那条流第一次发布之后的 65549：调用方把一个水位早已推高的池当成了 mkfs 那条流，
@@ -7638,7 +7544,7 @@ fn build_inode_tree_units(
     let root_unit = build_index_node(
         inode_tree_identifier,
         1,
-        8,
+        INODE_KEY_WIDTH_IN_BYTES,
         &smallest_key,
         &largest_key,
         txg,
@@ -9618,7 +9524,7 @@ mod tests {
         // （inode 树那个计数器上面一个都没用过，所以从 0 起）。
         let mut records = Vec::new();
         // 这里不写 `change_count: txg.0,`：`crates/mutations.tsv` 按那串字面锚在 `publish_overwrite` 上，
-        // 同一份文件里出现两次它就腐化（门禁 59 号）。
+        // 同一份文件里出现两次它就腐化（门禁 checker-tier-crates-mutation-replay 判锚点恰好命中一次）。
         let change_count = txg.0;
         for inode in 1..=(INODE_LEAF_RECORDS + 1) {
             records.push(InodeRecord {
@@ -9696,9 +9602,33 @@ mod tests {
 
     #[test]
     fn every_transaction_unit_names_its_class_tree_and_placement_rule() {
-        for identity in TransactionUnit::IN_BUMP_ORDER {
-            assert_eq!(identity.tag().len(), 2);
-        }
+        assert_eq!(
+            TransactionUnit::IN_BUMP_ORDER
+                .iter()
+                .map(|unit| unit.unit_class())
+                .collect::<Vec<_>>(),
+            [
+                UNIT_CLASS_DATA,
+                UNIT_CLASS_INDEX_NODE,
+                UNIT_CLASS_PACKED,
+                UNIT_CLASS_INDEX_NODE,
+                UNIT_CLASS_INDEX_NODE,
+                UNIT_CLASS_INDEX_NODE,
+                UNIT_CLASS_INDEX_NODE,
+                UNIT_CLASS_INDEX_NODE,
+            ],
+            "数据单元码 1、inode 叶容器码 3，extent 根、inode 根、分配记录树、记账树、映射树、树表码 2"
+        );
+        assert_eq!(
+            TransactionUnit::InstanceTable.unit_class(),
+            UNIT_CLASS_PACKED,
+            "实例表单元是码 3 打包容器"
+        );
+        assert_eq!(
+            TransactionUnit::InstanceTablePageAfterTheFirst(InstanceTablePageIndex(1)).unit_class(),
+            UNIT_CLASS_PACKED,
+            "实例表链第一片之后的每一片同样是码 3"
+        );
         assert_eq!(
             TransactionUnit::AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition {
                 level: 0,
diff --git a/crates/singlefs-core/src/unit.rs b/crates/singlefs-core/src/unit.rs
index d8a5884b..1b230d4b 100644
--- a/crates/singlefs-core/src/unit.rs
+++ b/crates/singlefs-core/src/unit.rs
@@ -147,10 +147,23 @@ pub fn index_node_header_bytes(key_width: usize) -> usize {
 }
 
 /// 一个码 2 节点装得下多少条定宽条目：(16384 − 头 − 预留) / 条目宽。写者在装节点之前拿它判「装不下」并报错，不走到断言。
+///
+/// # Panics
+/// 条目宽是 0（没有「装几条」可算），或 key 宽大到含预留位的头宽过一个节点（16384 减头宽会下溢）。调用方交的都是格式常量
+/// （各种树的 key 宽与条目宽），盘上读来的宽不进这里；两种输入都是调用方的 bug。
 #[must_use]
 pub fn index_node_entry_capacity(key_width: usize, entry_width: usize) -> usize {
+    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
+    assert!(
+        entry_width > 0,
+        "条目宽是 0：index_node_entry_capacity 只收格式常量里的条目宽"
+    );
+    assert!(
+        key_width <= node_bytes && index_node_header_bytes(key_width) <= node_bytes,
+        "key 宽 {key_width} 的码 2 节点头宽过一个节点：index_node_entry_capacity 只收格式常量里的 key 宽"
+    );
     let entries_start = index_node_header_bytes(key_width);
-    (usize::try_from(NODE_BYTES).expect("16384") - entries_start) / entry_width
+    (node_bytes - entries_start) / entry_width
 }
 
 #[allow(
@@ -578,6 +591,38 @@ mod tests {
     use super::*;
     use crate::checksum::wide_checksum_field_holds;
 
+    /// 条目宽 0：在开头那一句断言上 panic，不走到除法上除零（外部代码审查第二条）。
+    #[test]
+    #[should_panic(expected = "条目宽是 0")]
+    fn index_node_entry_capacity_of_a_zero_entry_width_panics_on_its_precondition() {
+        let _ = index_node_entry_capacity(8, 0);
+    }
+
+    /// key 宽 8135：含预留位的头宽 86 + 2 × 8135 + 29 = 16385，宽过一个节点；在断言上 panic，不走到 16384 减头宽的下溢上。
+    #[test]
+    #[should_panic(expected = "的码 2 节点头宽过一个节点")]
+    fn index_node_entry_capacity_of_a_key_width_whose_header_overruns_the_node_panics_on_its_precondition(
+    ) {
+        let _ = index_node_entry_capacity(8135, 8);
+    }
+
+    /// key 宽 8134：头宽 86 + 2 × 8134 + 29 = 16383，还装得下一个节点；条目宽 2 时一条都装不下，交 0（写者据此报「装不下」），不 panic。
+    /// 头宽的那一判是「不宽过节点」，不是「留得下一条」。
+    #[test]
+    fn index_node_entry_capacity_of_the_widest_key_whose_header_fits_the_node_is_zero_when_no_entry_fits(
+    ) {
+        assert_eq!(
+            index_node_header_bytes(8134),
+            16383,
+            "头宽 86 + 2 × 8134 + 29"
+        );
+        assert_eq!(
+            index_node_entry_capacity(8134, 2),
+            0,
+            "16384 − 16383 = 1 字节，条目宽 2 一条都装不下"
+        );
+    }
+
     #[test]
     fn empty_tree_table_node_has_key_width_at_51_and_a_sealed_header() {
         let filesystem_identifier = [9u8; 16];
diff --git a/crates/singlefs-core/src/unlink.rs b/crates/singlefs-core/src/unlink.rs
index 9a1d7b59..ca2d6640 100644
--- a/crates/singlefs-core/src/unlink.rs
+++ b/crates/singlefs-core/src/unlink.rs
@@ -134,30 +134,6 @@ pub fn next_unlink_step(data_units_remaining: u64, batch_size: UnlinkBatchSize)
     }
 }
 
-/// 从还剩 `data_units_remaining` 个数据单元起走到底的每一步，按先后：最后一步恒是 [`UnlinkStep::FinalBatch`]。
-#[must_use]
-pub fn every_unlink_step_from(
-    data_units_remaining: u64,
-    batch_size: UnlinkBatchSize,
-) -> Vec<UnlinkStep> {
-    let mut steps = Vec::new();
-    let mut remaining = data_units_remaining;
-    // 迭代上界：每一轮剩下的单元数至少少 1（批大小至少 1），⌈剩下 ÷ 批大小⌉ 轮之内走到最后一批。
-    loop {
-        let step = next_unlink_step(remaining, batch_size);
-        steps.push(step);
-        match step {
-            UnlinkStep::Batch {
-                data_units_kept, ..
-            } => {
-                assert!(data_units_kept < remaining, "每一批至少释放一个单元");
-                remaining = data_units_kept;
-            }
-            UnlinkStep::FinalBatch { .. } => return steps,
-        }
-    }
-}
-
 /// 删除的一批（[`UnlinkStep::Batch`]）之后 inode `inode` 那棵 extent 树：下段截到前 `data_units_kept` 个单元——
 /// 罩的单元全在留下那一段里的节点一个字节都不变、照抄；罩着被截掉那一段的，在新下段里只剩每层最右那一个（被截掉那一片下段叶与它的祖先），
 /// 它们重写；整个落在被截掉那一段里的节点不再出现、换下（K-R3-1：删除批只重写被截掉那一片下段叶与它的祖先）。
@@ -276,6 +252,29 @@ mod tests {
 
     const PAYLOAD_IN_BYTES: u64 = 32634;
 
+    /// 从还剩 `data_units_remaining` 个数据单元起走到底的每一步，按先后：最后一步恒是 [`UnlinkStep::FinalBatch`]。
+    fn every_unlink_step_from(
+        data_units_remaining: u64,
+        batch_size: UnlinkBatchSize,
+    ) -> Vec<UnlinkStep> {
+        let mut steps = Vec::new();
+        let mut remaining = data_units_remaining;
+        // 迭代上界：每一轮剩下的单元数至少少 1（批大小至少 1），⌈剩下 ÷ 批大小⌉ 轮之内走到最后一批。
+        loop {
+            let step = next_unlink_step(remaining, batch_size);
+            steps.push(step);
+            match step {
+                UnlinkStep::Batch {
+                    data_units_kept, ..
+                } => {
+                    assert!(data_units_kept < remaining, "每一批至少释放一个单元");
+                    remaining = data_units_kept;
+                }
+                UnlinkStep::FinalBatch { .. } => return steps,
+            }
+        }
+    }
+
     fn capped(data_units: u64) -> UnlinkBatchSize {
         UnlinkBatchSize::CappedForTests(
             UnlinkBatchDataUnitsCappedForTests::new(data_units).expect("1..=144"),
diff --git a/crates/singlefs-format/src/lib.rs b/crates/singlefs-format/src/lib.rs
index d8b5f331..91fb1f13 100644
--- a/crates/singlefs-format/src/lib.rs
+++ b/crates/singlefs-format/src/lib.rs
@@ -1,7 +1,7 @@
 //! 格式常量：新池新建文件要写的每个宽度都在这里。
 //!
 //! 每个常量的值来自 `.claude/kb/layout/01-first-txn.md` 与各决策分项；带 `format-const:` 注的常量
-//! 与 kb 里同名的 `<!-- format-const: NAME = N -->` 标记由门禁 27 号绑住，改值要三处一起动。
+//! 与 kb 里同名的 `<!-- format-const: NAME = N -->` 标记由门禁 code-source-discipline 的 format-constants 格绑住，改值要三处一起动。
 //! 值还是预想（等一次实测或一次定案）的常量，前一行写 `// placeholder: <分项或欠账编号（简称）> —— 为什么还是预想`，
 //! 门禁「格式常量文件里的占位」数它们、并核每个占位都指到一条真实存在的分项或欠账。
 //!
@@ -74,7 +74,7 @@ pub const ACCOUNTING_TREE_INDEX_NODE_HEADER_BYTES: u64 = 159;
 pub const CENTRAL_MAPPING_TREE_INDEX_NODE_HEADER_BYTES: u64 = 169;
 
 /// 位置条目：设备身份 4 + 16 KiB 槽号 6 + 密文校验和 4（D19（块指针的结构与宽度预算） 已定项 4）。format-const: LOC_ENTRY
-pub const LOC_ENTRY: u64 = 14; // naming-lint:external 名字由 kb 的 format-const 登记位定（D19 已定项 4），门禁 27 号按这个名字绑值
+pub const LOC_ENTRY: u64 = 14; // naming-lint:external 名字由 kb 的 format-const 登记位定（D19 已定项 4），门禁 code-source-discipline 的 format-constants 格按这个名字绑值
 
 /// 每条指针带几份位置条目（D2（RAID 条带策略） 已定项 6：每次写至少 w = 2 列）。
 pub const LOCATION_ENTRIES_PER_POINTER: u64 = 2;
@@ -185,9 +185,6 @@ pub const EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES: u64 = 113;
 /// extent 树上段一片叶罩几个 inode 号：(16384 − 163) ÷ 113 的整数部分（一个 inode 号至多一条上段叶条目，叶永远装得下）。
 pub const EXTENT_TREE_UPPER_LEAF_INODES: u64 = 143;
 
-/// 新池新建文件这次发布写出的记账行数（D5（快照 / 空间记账机制） 已定项 8，2026-09-14 用户定案 15 行）。
-pub const NEW_POOL_FILE_CREATION_ACCOUNTING_ROWS: u64 = 15;
-
 /// 树表条目：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + previous_snapshot_txg 8 + 诞生 txg 8 + 头 ID 8 + 预留 76（D8（核心索引结构） 已定项 8；重排与头 ID 2026-09-14 用户定案，加宽到 200 是 2026-09-16 用户定案）。format-const: TREE_TABLE_ENTRY_BYTES
 pub const TREE_TABLE_ENTRY_BYTES: u64 = 200;
 
@@ -262,7 +259,8 @@ pub const SYSTEM_CONFIGURATION_BYTES: u64 = 489;
 /// 系统配置每盘恒 2 个槽、槽 = 世代号 mod 2（D22（单元原子性怎么合成） 已定项 16）。
 pub const SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE: u64 = 2;
 
-/// 固定结构槽距的下限；槽距 = max(4096, mkfs 时探测的 io_min)（D2（RAID 条带策略） 已定项 19）。
+/// 固定结构槽距的下限；槽距 = 4096 向上取整到 mkfs 时探测的 io_min 的整数倍（D2（RAID 条带策略） 已定项 19；io_min 是 2 的幂时
+/// 就是 max(4096, io_min)，io_min = 3072 时是 6144；实现的算式在 `SystemImmutableSizes::slot_spacing_for`）。
 pub const FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES: u64 = 4096;
 
 /// 系统配置槽宽是格式常量 4096（D22（单元原子性怎么合成） 已定项 2，2026-09-14 三方第一轮 + 用户定案；此前是探测到的 physical_block_size）。format-const: SYSTEM_CONFIGURATION_SLOT_BYTES
@@ -323,7 +321,7 @@ pub const TEST_IMAGE_DEFAULT_BYTES: u64 = 4_294_967_296;
 mod tests {
     use super::*;
 
-    /// 字节表零到七的合计与这里的推导式必须一个数不差；写成加法、不写减法（减法在变异下会编译期溢出）。
+    /// 字节表零到七的合计与这里的推导式必须一个数不差：段宽的合计写成加法，扇出与一片装几条要从节点宽、单元宽里扣掉头宽，写成减法。
     #[test]
     fn widths_match_the_new_pool_file_creation_byte_table() {
         assert_eq!(
@@ -503,7 +501,7 @@ mod tests {
         );
     }
 
-    /// 这六个宽写成整数字面量，门禁 27 号才核得了它们与 kb 的登记值（C94（登记的格式常量与后来的定案对不上））；
+    /// 这六个宽写成整数字面量，门禁 code-source-discipline 的 format-constants 格才核得了它们与 kb 的登记值（C94（登记的格式常量与后来的定案对不上））；
     /// 原来定义它们的算式留在这里，要一个数不差。写成加法、不写减法（减法在变异下会编译期溢出）。
     #[test]
     fn pointer_record_and_entry_width_literals_equal_the_field_sums_they_stand_for() {
@@ -550,7 +548,7 @@ mod tests {
     }
 
     /// 这八个量的值原来写成算式；D13（验证路线） 已定项 5「判据是发射物里有没有分支与算术」不许，改成整数字面量
-    /// （门禁 27 号也只认字面量）。原来的算式原样留在这里，与字面量一个数不差。
+    /// （门禁 code-source-discipline 的 format-constants 格也只认字面量）。原来的算式原样留在这里，与字面量一个数不差。
     #[test]
     fn arithmetic_initializer_literals_equal_the_expressions_they_replaced() {
         assert_eq!(
@@ -599,7 +597,7 @@ mod tests {
             + NONCE_MAC_ALGORITHM_RESERVED_BYTES
     }
 
-    /// 每棵树的码 2 节点头宽写成整数字面量（门禁 27 号要核），与字段表三段在那棵树的 key 宽上加起来的一个数不差；
+    /// 每棵树的码 2 节点头宽写成整数字面量（门禁 code-source-discipline 的 format-constants 格要核），与字段表三段在那棵树的 key 宽上加起来的一个数不差；
     /// 哪棵树取哪个 key 宽照 `.claude/kb/layout/01-first-txn.md` 那几行：inode 树根与树表单元都是 8，两者各有各的名字。
     #[test]
     fn tree_index_node_header_literals_equal_the_header_formula_at_each_tree_key_width() {
```
