# m4-close-code-r2：这一轮被判的产品代码改动全文（主 agent 从两份补丁里按文件抽，不另拼）

## 容器小树代价第二件（m4-close-code-r1-fix-b2-writer-2）

### `crates/singlefs-core/src/admission.rs`

```diff
diff --git a/crates/singlefs-core/src/admission.rs b/crates/singlefs-core/src/admission.rs
index 1189dfd..d3cac23 100644
--- a/crates/singlefs-core/src/admission.rs
+++ b/crates/singlefs-core/src/admission.rs
@@ -34,7 +34,7 @@
 //! 准入不够、或准入放行而落点取不到时先推空发布抬 F 再判（D16（发布语义） 已定项 1「准入」那一行，C283（准入失败时不先推发布就报 ENOSPC））
 //! 不在这里做：发布那一处归挂着的会话（`mounted_session`），可写挂载那一处归 `mount::establish_instance`（写行之后推）。
 
-use std::collections::BTreeSet;
+use std::collections::{BTreeMap, BTreeSet};
 use std::num::NonZeroU64;
 
 use singlefs_format::{
@@ -42,7 +42,7 @@ use singlefs_format::{
     SLOT_BYTES, TOMBSTONE_CONTAINER_RECORDS,
 };
 
-use crate::address::{CheckpointTxg, DataUnitIndexInFile, DeviceIdentity};
+use crate::address::{CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, InodeNumber};
 use crate::allocation_record_tree::{span_in_slots_at_level, AllocationRecordTreeGeometry};
 use crate::allocator::PoolAllocator;
 use crate::code_two_tree::{
@@ -51,7 +51,8 @@ use crate::code_two_tree::{
 };
 use crate::inode_tree::InodeLeafContainer;
 use crate::records::{
-    AccountingEntry, NLINK_OF_AN_INODE_BEING_DELETED, STATISTIC_COMMITTED_RESERVATION_BYTES,
+    AccountingEntry, InodeRecord, NLINK_OF_AN_INODE_BEING_DELETED,
+    STATISTIC_COMMITTED_RESERVATION_BYTES,
 };
 use crate::root_ring::RootRingSlotsPerRegion;
 use crate::transaction::{
@@ -275,29 +276,95 @@ pub fn committed_reservation_recorded_in(
     })
 }
 
-/// n_live（D3（空间分配） 已定项 2）：一版 inode 树里 nlink ≥ 1 的记录数。
-/// 管理员回退那一版按 D23（journal 的角色与格式） 已定项 14 写的是「R_old 那一版 inode 树各叶容器的记录数之和减 `deleted_inodes` 的 key 数」：
-/// 回退那一版的 inode 树与 `deleted_inodes` 都取 R_old 的，队列 = inode 树里 nlink = 0 的号集合（D8（核心索引结构） 已定项 16 第 3 条）⇒
-/// 两种写法是同一个数；这里按 nlink 数，盘上读来的队列与记录对不上时也不会减成负数。
-#[must_use]
-pub fn live_objects_of_the_inode_tree(inode_leaf_containers: &[InodeLeafContainer]) -> u64 {
-    let live_records = inode_leaf_containers
-        .iter()
-        .flat_map(|container| container.records.iter())
-        .filter(|record| record.nlink != NLINK_OF_AN_INODE_BEING_DELETED)
-        .count();
-    u64::try_from(live_records).expect("一棵 inode 树的记录数装得进 u64")
+/// 一版 inode 树里的 n_live 与 n_记录：记账标签 6 的两项（D3（空间分配） 已定项 2）。n_live 是 nlink ≥ 1 的记录数；n_记录 是全部记录数，
+/// 含 nlink = 0 的（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条：退役记录写在最后一批，意图到最后一批之间那个对象
+/// n_live 已不含它，退役那一份预付挂 n_live 就在用到之前释放了）。管理员回退那一版按 D23（journal 的角色与格式） 已定项 14 写的是
+/// 「R_old 那一版 inode 树各叶容器的记录数之和减 `deleted_inodes` 的 key 数」：回退那一版的 inode 树与 `deleted_inodes` 都取 R_old 的，
+/// 队列 = inode 树里 nlink = 0 的号集合（D8（核心索引结构） 已定项 16 第 3 条）⇒ 两种写法是同一个数；这里按 nlink 数。
+/// 准入（运行时决策路径）读它，所以随一版带着、在提交时增量维护（`.claude/rules/fs-design.md`「空间统计与配额落在第一格：必须在事务提交时
+/// 增量维护出来」）：发布按这次写进与删掉的那几条记录改上一版的（[`Self::after_writing`]、[`Self::after_removing`]），只有冷恢复读出一版时
+/// 逐片逐条数一遍当起始值（[`Self::counted_over_the_inode_tree`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct InodeRecordCounts {
+    pub live_objects: u64,
+    pub records: u64,
 }
 
-/// n_记录（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条）：一版 inode 树里全部记录数，含 nlink = 0 的
-/// （退役记录写在最后一批，意图到最后一批之间那个对象 n_live 已不含它，退役那一份预付挂 n_live 就在用到之前释放了）。
-#[must_use]
-pub fn inode_records_of_the_inode_tree(inode_leaf_containers: &[InodeLeafContainer]) -> u64 {
-    let records: usize = inode_leaf_containers
-        .iter()
-        .map(|container| container.records.len())
-        .sum();
-    u64::try_from(records).expect("一棵 inode 树的记录数装得进 u64")
+impl InodeRecordCounts {
+    /// 一条记录都没有的 inode 树（mkfs 之后、头一个文件版本之前）。
+    pub const OF_AN_EMPTY_INODE_TREE: InodeRecordCounts = InodeRecordCounts {
+        live_objects: 0,
+        records: 0,
+    };
+
+    /// 逐片逐条数一遍：冷恢复从盘上读出一版时算起始值（挂载一次）。测试对照维护出来的值用它自己另写的数法，不用这一个。
+    #[must_use]
+    pub fn counted_over_the_inode_tree<'containers>(
+        inode_leaf_containers: impl IntoIterator<Item = &'containers InodeLeafContainer>,
+    ) -> Self {
+        let mut counts = Self::OF_AN_EMPTY_INODE_TREE;
+        // 迭代上界是叶容器片数乘一片的记录数；跨轮携带的是到这一片为止的两个数。
+        for container in inode_leaf_containers {
+            counts.records +=
+                u64::try_from(container.records.len()).expect("一片叶容器的记录数装得进 u64");
+            counts.live_objects += u64::try_from(
+                container
+                    .records
+                    .iter()
+                    .filter(|record| record.nlink != NLINK_OF_AN_INODE_BEING_DELETED)
+                    .count(),
+            )
+            .expect("一片叶容器的记录数装得进 u64");
+        }
+        counts
+    }
+
+    /// 这次发布逐条写进 `written`（号已在树里的原地换掉、不在的接在末尾，同 `crate::inode_tree::write_records_into_leaf_containers`）之后的两个数：
+    /// 换掉的那一条按它换掉之前是不是活的改 n_live，接在末尾的 n_记录加一。`record_before_this_publish` 交回上一版里那个号的记录（没有就 `None`），
+    /// 调用方按叶序折半找，不遍历；同一次里写了两次的号，后一次按前一次写的算。代价是这次写的条数（乘一次折半找）。
+    #[must_use]
+    pub fn after_writing(
+        self,
+        written: &[InodeRecord],
+        record_before_this_publish: impl Fn(InodeNumber) -> Option<InodeRecord>,
+    ) -> Self {
+        let mut counts = self;
+        let mut written_so_far: BTreeMap<u64, InodeRecord> = BTreeMap::new();
+        // 迭代上界是这次写的条数；跨轮携带的是到这一条为止的两个数与这次已写的那几条。
+        for record in written {
+            let replaced = written_so_far
+                .get(&record.inode)
+                .copied()
+                .or_else(|| record_before_this_publish(InodeNumber(record.inode)));
+            match replaced {
+                Some(replaced_record) => {
+                    counts.live_objects = counts.live_objects
+                        - u64::from(is_live(&replaced_record))
+                        + u64::from(is_live(record));
+                }
+                None => {
+                    counts.records += 1;
+                    counts.live_objects += u64::from(is_live(record));
+                }
+            }
+            written_so_far.insert(record.inode, *record);
+        }
+        counts
+    }
+
+    /// 这次发布删掉 `removed` 那一条（unlink 最后一批，`crate::inode_tree::remove_record_from_leaf_containers`）之后的两个数。
+    #[must_use]
+    pub fn after_removing(self, removed: &InodeRecord) -> Self {
+        Self {
+            live_objects: self.live_objects - u64::from(is_live(removed)),
+            records: self.records - 1,
+        }
+    }
+}
+
+/// 这条记录算不算 n_live 里的一个：nlink ≥ 1（nlink = 0 是删除中，D8（核心索引结构） 已定项 16）。
+fn is_live(record: &InodeRecord) -> bool {
+    record.nlink != NLINK_OF_AN_INODE_BEING_DELETED
 }
 
 /// 墓碑预付里每个活对象、每份副本摊的字节：⌈一个 32 KiB 墓碑容器 ÷ 一容器装的墓碑条数 815⌉ = ⌈32768 ÷ 815⌉ = 41
```

### `crates/singlefs-core/src/container_tree.rs`

```diff
diff --git a/crates/singlefs-core/src/container_tree.rs b/crates/singlefs-core/src/container_tree.rs
index e8b5dd6..eb4729b 100644
--- a/crates/singlefs-core/src/container_tree.rs
+++ b/crates/singlefs-core/src/container_tree.rs
@@ -5,7 +5,11 @@
 //! `crate::retirement_tree`）把这两样填进来。
 //!
 //! 一次发布可以往开放容器里追加不止一条（退役树：一次删除连着吸收几片，[`append_records_to_the_open_container`]）：开放容器装满就开下一片，
-//! 这次重写的码 2 节点是每片重写的容器的祖先的并（[`ContainerTreeShape::ancestors_of_containers`]），只追加一条时它就是最右那一条路径。
+//! 这次重写的码 2 节点是每片重写的容器的祖先的并（[`ContainerTreeShape::ancestors_of_the_rightmost_containers`]），只追加一条时它就是最右那一条路径。
+//! 重写的容器恒是最右端连着的那几片（[`rewritten_containers_are_the_rightmost`]），所以每层重写的节点是这一层最右端的一段：发布路径上
+//! 求祖先、判哪一片哪个节点重写了，都只读最右端那几段，不随容器总数涨（`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」
+//! 运行时决策路径那一格）。最右端开一片按值原地改形状（[`ContainerTreeShape::after_a_container_is_opened_at_the_right_end`]），这次之前那一版
+//! 只记每层节点数（[`ContainerTreeNodeCounts`]）。
 //!
 //! 根满时长高（D18（块里携带什么信息） 已定项 24 第 1、5 条「根满时长高，照 D8（核心索引结构） 已定项 6」，用户定案 K-R3-3）：
 //! 码 2 节点装满时末尾分裂——左半保留全部原有条目、一个字节不动，右半从触发分裂的那条新条目起；根装满时长一层，新根 1 个码 2、
@@ -439,17 +443,47 @@ pub open spec fn levels_after_opening_at_the_right_end(levels: Seq<Seq<usize>>,
     }
 }
 
-/// `levels` 里不在 `previous_levels` 的每个节点，都是 `rewritten_containers` 里某一片的祖先：照抄的节点上一版都有（[`ContainerTreeAfterThisPublish::node_origins`]）。
+/// 一版形状每一层有几个节点（层下标 0 在前）：上一版的形状在这次发布里只记这几个数（[`ContainerTreeNodeCounts`]）。
+pub open spec fn node_counts_of(levels: Seq<Seq<usize>>) -> Seq<int> {
+    Seq::new(levels.len(), |level_index: int| levels[level_index].len() as int)
+}
+
+/// 每层节点数是 `node_counts` 的那一版里，层下标 `level_index` 上有第 `node` 个节点。
+pub open spec fn node_is_counted(node_counts: Seq<int>, level_index: int, node: int) -> bool {
+    0 <= level_index < node_counts.len() && 0 <= node < node_counts[level_index]
+}
+
+/// 每层节点数是 `node_counts` 的那一版里这个位置上有节点。
+pub open spec fn position_is_counted(node_counts: Seq<int>, position: ContainerTreeNodePosition) -> bool {
+    1 <= position.level <= node_counts.len() && position.index_in_level < node_counts[position.level - 1]
+}
+
+/// `levels` 里不在上一版（每层节点数 `previous_node_counts`）的每个节点，都是 `rewritten_containers` 里某一片的祖先：照抄的节点上一版都有
+/// （[`ContainerTreeAfterThisPublish::node_origins`]）。
 pub open spec fn every_node_absent_from_the_previous_version_is_rewritten(
-    previous_levels: Seq<Seq<usize>>,
+    previous_node_counts: Seq<int>,
     levels: Seq<Seq<usize>>,
     rewritten_containers: Seq<usize>,
 ) -> bool {
     forall|level_index: int, node: int|
-        #[trigger] node_is_in_the_levels(levels, level_index, node) && !node_is_in_the_levels(previous_levels, level_index, node)
+        #[trigger] node_is_in_the_levels(levels, level_index, node) && !node_is_counted(previous_node_counts, level_index, node)
             ==> node_is_an_ancestor_of_one_of(levels, level_index, node, rewritten_containers)
 }
 
+/// 一共 `total` 片时最右端那 `count` 片容器，按叶序。
+pub open spec fn rightmost_containers(total: int, count: int) -> Seq<usize> {
+    Seq::new(count as nat, |listed: int| (total - count + listed) as usize)
+}
+
+/// 重写清单就是最右端那几片（一共 `total` 片）：按叶序一片挨一片、最后一片是最右那片。发布里重写的就是这样几片：
+/// 追加只往最右那片里装、装满了在最右端开新的（[`append_records_to_the_open_container`]）。
+pub open spec fn rewritten_containers_are_the_rightmost(rewritten_containers: Seq<usize>, total: int) -> bool {
+    &&& rewritten_containers.len() <= total
+    &&& forall|listed: int|
+        0 <= listed < rewritten_containers.len() ==> #[trigger] rewritten_containers[listed] == total - rewritten_containers.len()
+            + listed
+}
+
 /// 一组容器的记录按叶序依次连起来（「key 到条目」在规格里的样子）。
 pub open spec fn records_of_containers<Record>(containers: Seq<RecordContainer<Record>>) -> Seq<Record>
     decreases containers.len(),
@@ -967,20 +1001,20 @@ proof fn lemma_opening_extends_the_levels_at_the_right_end(levels: Seq<Seq<usize
 
 /// 多重写几片，罩住的照样罩住。
 proof fn lemma_more_rewritten_containers_keep_every_new_node_rewritten(
-    previous_levels: Seq<Seq<usize>>,
+    previous_node_counts: Seq<int>,
     levels: Seq<Seq<usize>>,
     rewritten_containers: Seq<usize>,
     added: usize,
 )
     requires
-        every_node_absent_from_the_previous_version_is_rewritten(previous_levels, levels, rewritten_containers),
+        every_node_absent_from_the_previous_version_is_rewritten(previous_node_counts, levels, rewritten_containers),
     ensures
-        every_node_absent_from_the_previous_version_is_rewritten(previous_levels, levels, rewritten_containers.push(added)),
+        every_node_absent_from_the_previous_version_is_rewritten(previous_node_counts, levels, rewritten_containers.push(added)),
 {
     let longer = rewritten_containers.push(added);
     assert forall|level_index: int, node: int|
-        #[trigger] node_is_in_the_levels(levels, level_index, node) && !node_is_in_the_levels(
-            previous_levels,
+        #[trigger] node_is_in_the_levels(levels, level_index, node) && !node_is_counted(
+            previous_node_counts,
             level_index,
             node,
         ) implies node_is_an_ancestor_of_one_of(levels, level_index, node, longer) by {
@@ -997,7 +1031,7 @@ proof fn lemma_more_rewritten_containers_keep_every_new_node_rewritten(
 
 /// 最右端新开一片容器、把它记进重写清单之后，新出现的节点照样都罩着重写的某一片。
 proof fn lemma_opening_keeps_every_new_node_rewritten(
-    previous_levels: Seq<Seq<usize>>,
+    previous_node_counts: Seq<int>,
     levels: Seq<Seq<usize>>,
     entries_per_node: int,
     rewritten_containers: Seq<usize>,
@@ -1007,10 +1041,10 @@ proof fn lemma_opening_keeps_every_new_node_rewritten(
         container_count(levels) + 1 <= usize::MAX,
         2 <= entries_per_node <= usize::MAX,
         forall|listed: int| 0 <= listed < rewritten_containers.len() ==> #[trigger] rewritten_containers[listed] < container_count(levels),
-        every_node_absent_from_the_previous_version_is_rewritten(previous_levels, levels, rewritten_containers),
+        every_node_absent_from_the_previous_version_is_rewritten(previous_node_counts, levels, rewritten_containers),
     ensures
         every_node_absent_from_the_previous_version_is_rewritten(
-            previous_levels,
+            previous_node_counts,
             levels_after_opening_at_the_right_end(levels, entries_per_node),
             rewritten_containers.push(container_count(levels) as usize),
         ),
@@ -1023,8 +1057,8 @@ proof fn lemma_opening_keeps_every_new_node_rewritten(
     lemma_opening_keeps_the_shape_well_formed(levels, entries_per_node);
     lemma_opening_extends_the_levels_at_the_right_end(levels, entries_per_node);
     assert forall|level_index: int, node: int|
-        #[trigger] node_is_in_the_levels(opened, level_index, node) && !node_is_in_the_levels(
-            previous_levels,
+        #[trigger] node_is_in_the_levels(opened, level_index, node) && !node_is_counted(
+            previous_node_counts,
             level_index,
             node,
         ) implies node_is_an_ancestor_of_one_of(opened, level_index, node, longer) by {
@@ -1102,22 +1136,46 @@ proof fn lemma_node_is_marked_by_its_children(levels: Seq<Seq<usize>>, level_ind
                 child,
                 containers,
             );
-        lemma_children_count_prefix_sum_is_monotone(levels[level_index], node + 1, node + 1);
-        lemma_children_count_sum_is_not_negative(levels[level_index].take(node));
-        lemma_covering_node(levels[level_index], child);
-        if level_index == 0 {
-            let listed = choose|listed: int| 0 <= listed < containers.len() && #[trigger] containers[listed] == child;
-            assert(node_is_an_ancestor_of_container(levels, 0, node, containers[listed] as int));
-        } else {
-            let listed = choose|listed: int|
-                0 <= listed < containers.len() && #[trigger] node_is_an_ancestor_of_container(
-                    levels,
-                    (level_index - 1) as nat,
-                    child,
-                    containers[listed] as int,
-                );
-            assert(node_is_an_ancestor_of_container(levels, level_index as nat, node, containers[listed] as int));
-        }
+        lemma_a_node_over_a_marked_child_is_an_ancestor(levels, level_index, node, child, containers);
+    }
+}
+
+/// 一个节点罩住的孩子里有一个被标过，它就是 `containers` 某一片的祖先：[`lemma_node_is_marked_by_its_children`] 从孩子推到节点的那一半，
+/// 单列一个引理，那一个的查询只剩从节点推到孩子那一半（整 crate 一起验时两半挤在一个查询里，在登记的 rlimit 上验不过）。
+proof fn lemma_a_node_over_a_marked_child_is_an_ancestor(
+    levels: Seq<Seq<usize>>,
+    level_index: int,
+    node: int,
+    child: int,
+    containers: Seq<usize>,
+)
+    requires
+        container_tree_shape_is_well_formed(levels),
+        0 <= level_index < levels.len(),
+        forall|listed: int| 0 <= listed < containers.len() ==> #[trigger] containers[listed] < container_count(levels),
+        child_lies_under_node(levels[level_index], node, child),
+        child_is_marked(levels, level_index, child, containers),
+    ensures
+        node_is_an_ancestor_of_one_of(levels, level_index, node, containers),
+{
+    lemma_children_count_prefix_sum_is_monotone(levels[level_index], node + 1, node + 1);
+    lemma_children_count_sum_is_not_negative(levels[level_index].take(node));
+    lemma_covering_node(levels[level_index], child);
+    // 两支都在 `node_is_an_ancestor_of_one_of` 自己的触发式（层下标写成 `level_index as nat`）上给出见证。
+    if level_index == 0 {
+        let listed = choose|listed: int| 0 <= listed < containers.len() && #[trigger] containers[listed] == child;
+        assert(node_is_an_ancestor_of_container(levels, level_index as nat, node, containers[listed] as int));
+        assert(0 <= listed < containers.len());
+    } else {
+        let listed = choose|listed: int|
+            0 <= listed < containers.len() && #[trigger] node_is_an_ancestor_of_container(
+                levels,
+                (level_index - 1) as nat,
+                child,
+                containers[listed] as int,
+            );
+        assert(node_is_an_ancestor_of_container(levels, level_index as nat, node, containers[listed] as int));
+        assert(0 <= listed < containers.len());
     }
 }
 
@@ -1211,6 +1269,72 @@ proof fn lemma_ancestors_in_level(
     }
 }
 
+/// 一层前 `node_count` 个节点都不是 `containers` 某一片的祖先时，那几个里的祖先位置是空的。
+proof fn lemma_no_ancestor_among_the_first(levels: Seq<Seq<usize>>, level_index: int, containers: Seq<usize>, node_count: int)
+    requires
+        forall|node: int|
+            0 <= node < node_count ==> !(#[trigger] node_is_an_ancestor_of_one_of(levels, level_index, node, containers)),
+    ensures
+        ancestors_in_level(levels, level_index, containers, node_count) == Seq::<ContainerTreeNodePosition>::empty(),
+    decreases node_count,
+{
+    if node_count > 0 {
+        assert(!node_is_an_ancestor_of_one_of(levels, level_index, node_count - 1, containers));
+        lemma_no_ancestor_among_the_first(levels, level_index, containers, node_count - 1);
+    }
+}
+
+/// 最右端那 `count` 片里有没有第 `child` 片：有就是它落在最右端那几片的头一片到最右那片之间（一共 `total` 片）。
+proof fn lemma_rightmost_containers_hold_exactly_the_ones_from_their_first(total: int, count: int, child: int)
+    requires
+        0 <= count <= total <= usize::MAX,
+    ensures
+        (exists|listed: int|
+            0 <= listed < rightmost_containers(total, count).len() && #[trigger] rightmost_containers(total, count)[listed] == child)
+            <==> (total - count <= child < total),
+{
+    let containers = rightmost_containers(total, count);
+    if total - count <= child < total {
+        let listed = child - (total - count);
+        assert(containers[listed] == child);
+    }
+    if exists|listed: int| 0 <= listed < containers.len() && #[trigger] containers[listed] == child {
+        let listed = choose|listed: int| 0 <= listed < containers.len() && #[trigger] containers[listed] == child;
+        assert(containers[listed] == total - count + listed);
+    }
+}
+
+/// 重写清单是最右端那几片时，它就是 [`rightmost_containers`] 那一份。
+proof fn lemma_rewritten_containers_are_the_rightmost_ones(rewritten_containers: Seq<usize>, total: int)
+    requires
+        rewritten_containers_are_the_rightmost(rewritten_containers, total),
+        total <= usize::MAX,
+    ensures
+        rewritten_containers == rightmost_containers(total, rewritten_containers.len() as int),
+{
+    assert(rewritten_containers =~= rightmost_containers(total, rewritten_containers.len() as int));
+}
+
+/// 一片都没给时哪一层都没有祖先。
+proof fn lemma_no_containers_have_no_ancestors(levels: Seq<Seq<usize>>, level_count: int)
+    ensures
+        ancestors_below_level(levels, Seq::<usize>::empty(), level_count) == Seq::<ContainerTreeNodePosition>::empty(),
+    decreases level_count,
+{
+    if level_count > 0 {
+        lemma_no_containers_have_no_ancestors(levels, level_count - 1);
+        assert forall|node: int| 0 <= node < levels[level_count - 1].len() implies !(#[trigger] node_is_an_ancestor_of_one_of(
+            levels,
+            level_count - 1,
+            node,
+            Seq::<usize>::empty(),
+        )) by {}
+        lemma_no_ancestor_among_the_first(levels, level_count - 1, Seq::<usize>::empty(), levels[level_count - 1].len() as int);
+        assert(Seq::<ContainerTreeNodePosition>::empty() + Seq::<ContainerTreeNodePosition>::empty()
+            =~= Seq::<ContainerTreeNodePosition>::empty());
+    }
+}
+
 /// 前几层的祖先位置里，「在这张单子里」就是「是某一片的祖先」；单子里的层级不超过层数。
 pub proof fn lemma_ancestors_below_level_contains(levels: Seq<Seq<usize>>, containers: Seq<usize>, level_count: int)
     requires
@@ -1351,10 +1475,15 @@ proof fn lemma_records_of_containers_after_appending_to_the_last<Record>(
 
 /// 一版容器树的形状：每一层每个码 2 节点有几个孩子，自层级 1（孩子是容器）到根（最高那一层，恰好一个节点）。零根时一层都没有。
 /// 不变量（类型不变量 [`container_tree_shape_is_well_formed`]）：每个节点至少一个孩子；层级 1 的孩子数之和 = 容器数，层级 ℓ > 1 的孩子数之和 =
-/// 层级 ℓ − 1 的节点数；最高那一层只有一个节点。
+/// 层级 ℓ − 1 的节点数；最高那一层只有一个节点；`container_total` 等于容器数。
+///
+/// 容器数记成一个字段、构造时算好、最右端开一片时加一：问片数（[`Self::containers`]）与求祖先（[`Self::ancestor_suffix_starts_of_the_rightmost_containers`]，
+/// 从最右端往左减要知道层级 1 之下一共几片）都在发布路径上，现加层级 1 的孩子数要读层级 1 的每个节点，代价随容器总数涨
+/// （`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」运行时决策路径那一格）。
 #[derive(Debug, PartialEq, Eq)]
 pub struct ContainerTreeShape {
     children_of_each_node_at_each_level: Vec<Vec<usize>>,
+    container_total: usize,
 }
 
 impl Clone for ContainerTreeShape {
@@ -1365,7 +1494,10 @@ impl Clone for ContainerTreeShape {
         proof {
             use_type_invariant(self);
         }
-        Self { children_of_each_node_at_each_level: copy_of_the_children_counts(&self.children_of_each_node_at_each_level) }
+        Self {
+            children_of_each_node_at_each_level: copy_of_the_children_counts(&self.children_of_each_node_at_each_level),
+            container_total: self.container_total,
+        }
     }
 }
 
@@ -1423,33 +1555,6 @@ fn level_of_a_single_node(children: usize) -> (level: Vec<usize>)
     level
 }
 
-/// 一层前 `node_count` 个节点的孩子数之和。
-fn children_count_sum_of_the_first(children_of_each_node: &[usize], node_count: usize) -> (sum: usize)
-    requires
-        node_count <= children_of_each_node.len(),
-        children_count_sum(children_of_each_node@) <= usize::MAX,
-    ensures
-        sum == children_count_sum(children_of_each_node@.take(node_count as int)),
-{
-    let mut sum: usize = 0;
-    let mut node: usize = 0;
-    while node < node_count
-        invariant
-            node <= node_count <= children_of_each_node.len(),
-            children_count_sum(children_of_each_node@) <= usize::MAX,
-            sum == children_count_sum(children_of_each_node@.take(node as int)),
-        decreases node_count - node,
-    {
-        proof {
-            lemma_children_count_sum_of_one_more(children_of_each_node@, node as int);
-            lemma_children_count_prefix_sum_is_monotone(children_of_each_node@, node as int + 1, node as int + 1);
-        }
-        sum += children_of_each_node[node];
-        node += 1;
-    }
-    sum
-}
-
 /// 一层孩子数之和；越过 `usize` 交回 `None`。
 fn checked_children_count_sum(children_of_each_node: &[usize]) -> (sum: Option<usize>)
     ensures
@@ -1596,10 +1701,10 @@ fn each_level_counts_the_nodes_below(children_of_each_node_at_each_level: &[Vec<
 }
 
 impl ContainerTreeShape {
-    /// 类型不变量：形状良好（[`container_tree_shape_is_well_formed`]）。
+    /// 类型不变量：形状良好（[`container_tree_shape_is_well_formed`]），记着的容器数就是形状里的容器数。
     #[verifier::type_invariant]
     spec fn shape_is_well_formed(self) -> bool {
-        container_tree_shape_is_well_formed(self.levels())
+        container_tree_shape_is_well_formed(self.levels()) && self.container_total == container_count(self.levels())
     }
 
     /// 每层每个节点的孩子数（规格里用）。
@@ -1613,7 +1718,7 @@ impl ContainerTreeShape {
         ensures
             shape.levels() == Seq::<Seq<usize>>::empty(),
     {
-        let shape = Self { children_of_each_node_at_each_level: Vec::new() };
+        let shape = Self { children_of_each_node_at_each_level: Vec::new(), container_total: 0 };
         assert(shape.levels() =~= Seq::<Seq<usize>>::empty());
         shape
     }
@@ -1635,18 +1740,26 @@ impl ContainerTreeShape {
         let level_count = children_of_each_node_at_each_level.len();
         let the_top_level_is_one_root = level_count == 0 || children_of_each_node_at_each_level[level_count - 1].len() == 1;
         let levels_fit_one_byte = u8::try_from(level_count).is_ok();
-        let containers_fit_usize = level_count == 0 || checked_children_count_sum(&children_of_each_node_at_each_level[0]).is_some();
+        let container_total_if_it_fits_usize = if level_count == 0 {
+            Some(0)
+        } else {
+            checked_children_count_sum(&children_of_each_node_at_each_level[0])
+        };
         proof {
             if level_count > 0 {
                 assert(levels[level_count - 1] == children_of_each_node_at_each_level@[level_count - 1]@);
                 assert(levels[0] == children_of_each_node_at_each_level@[0]@);
             }
         }
-        if every_node_has_a_child && each_level_counts_the_nodes_below && the_top_level_is_one_root && levels_fit_one_byte
-            && containers_fit_usize {
-            Some(Self { children_of_each_node_at_each_level })
-        } else {
-            None
+        match container_total_if_it_fits_usize {
+            Some(container_total) => {
+                if every_node_has_a_child && each_level_counts_the_nodes_below && the_top_level_is_one_root && levels_fit_one_byte {
+                    Some(Self { children_of_each_node_at_each_level, container_total })
+                } else {
+                    None
+                }
+            },
+            None => None,
         }
     }
 
@@ -1676,7 +1789,7 @@ impl ContainerTreeShape {
         u64::from(self.root_level()).saturating_sub(1)
     }
 
-    /// 这棵树有几片容器。
+    /// 这棵树有几片容器：读记着的那个数，不逐个加层级 1 的孩子数。
     #[must_use]
     pub fn containers(&self) -> (containers: usize)
         ensures
@@ -1685,15 +1798,7 @@ impl ContainerTreeShape {
         proof {
             use_type_invariant(self);
         }
-        if self.children_of_each_node_at_each_level.is_empty() {
-            return 0;
-        }
-        let level_one = &self.children_of_each_node_at_each_level[0];
-        proof {
-            assert(level_one@ == self.levels()[0]);
-            lemma_children_count_sum_of_the_whole_level(level_one@);
-        }
-        children_count_sum_of_the_first(level_one, level_one.len())
+        self.container_total
     }
 
     /// 层级 `level` 上有几个节点（层级 0 是容器、超过根的层级 0 个）。
@@ -1829,289 +1934,479 @@ impl ContainerTreeShape {
         }
     }
 
-    /// 这个节点的孩子在下一层（层级 1 的孩子是容器，按叶序）里的下标区间。
-    ///
-    /// # Panics
-    /// 这个位置上没有节点（`requires` 排除的那一种）：调用方只拿 [`Self::positions_in_bump_order`] 里的位置问。
+    /// 每一层每个节点的孩子在下一层（层级 1 的孩子是容器，按叶序）里的下标区间：外层按层下标（层级 1 在前），里层同层从左到右。
+    /// 每层从最左那个节点起顺着累加孩子数，一层只走一遍：装一棵容器树这一版的单元时每个节点都要它（`crate::transaction` 的
+    /// `build_container_tree_units`），一个一个问、每个都从这一层最左加起，整层就是这一层节点数的平方。
     #[must_use]
-    pub fn children_of(&self, position: ContainerTreeNodePosition) -> (children: Range<usize>)
-        requires
-            position_is_in_the_shape(self.levels(), position),
+    pub fn children_of_every_node(&self) -> (children: Vec<Vec<Range<usize>>>)
         ensures
-            children.start == children_count_sum(self.levels()[position.level - 1].take(position.index_in_level as int)),
-            children.end == children.start + self.levels()[position.level - 1][position.index_in_level as int],
+            children@.len() == self.levels().len(),
+            forall|level_index: int|
+                0 <= level_index < children@.len() ==> (#[trigger] children@[level_index])@.len() == self.levels()[level_index].len(),
+            forall|level_index: int, node: int|
+                #[trigger] node_is_in_the_levels(self.levels(), level_index, node) ==> {
+                    let range = children@[level_index]@[node];
+                    &&& range.start == children_count_sum(self.levels()[level_index].take(node))
+                    &&& range.end == range.start + self.levels()[level_index][node]
+                },
     {
         proof {
             use_type_invariant(self);
         }
-        let level_index = usize::from(position.level) - 1;
-        let nodes = &self.children_of_each_node_at_each_level[level_index];
-        let index = usize::try_from(position.index_in_level).expect("同层序号装得进 usize");
-        proof {
-            assert(nodes@ == self.levels()[level_index as int]);
-            if level_index == 0 {
-                assert(children_count_sum(nodes@) == container_count(self.levels()));
-            } else {
-                assert(children_count_sum(self.levels()[level_index as int]) == self.levels()[level_index - 1].len());
-                assert(self.levels()[level_index - 1].len() == self.children_of_each_node_at_each_level@[level_index - 1].len());
+        let ghost levels = self.levels();
+        let mut children: Vec<Vec<Range<usize>>> = Vec::new();
+        let mut level_index: usize = 0;
+        while level_index < self.children_of_each_node_at_each_level.len()
+            invariant
+                levels == self.levels(),
+                container_tree_shape_is_well_formed(levels),
+                level_index <= levels.len(),
+                children@.len() == level_index,
+                forall|done_level_index: int|
+                    0 <= done_level_index < level_index ==> (#[trigger] children@[done_level_index])@.len()
+                        == levels[done_level_index].len(),
+                forall|done_level_index: int, node: int|
+                    0 <= done_level_index < level_index && #[trigger] node_is_in_the_levels(levels, done_level_index, node) ==> {
+                        let range = children@[done_level_index]@[node];
+                        &&& range.start == children_count_sum(levels[done_level_index].take(node))
+                        &&& range.end == range.start + levels[done_level_index][node]
+                    },
+            decreases self.children_of_each_node_at_each_level.len() - level_index,
+        {
+            let nodes = &self.children_of_each_node_at_each_level[level_index];
+            proof {
+                assert(nodes@ == levels[level_index as int]);
+                if level_index == 0 {
+                    assert(children_count_sum(nodes@) == container_count(levels));
+                } else {
+                    assert(children_count_sum(levels[level_index as int]) == levels[level_index - 1].len());
+                    assert(levels[level_index - 1].len() == self.children_of_each_node_at_each_level@[level_index - 1].len());
+                }
+                assert(nodes@.take(0) =~= Seq::<usize>::empty());
+            }
+            // 迭代上界是这一层的节点数；跨轮携带的是下一个节点的头一个孩子（这一层前几个节点的孩子数之和）。
+            let mut ranges: Vec<Range<usize>> = Vec::new();
+            let mut first_child: usize = 0;
+            let mut node: usize = 0;
+            while node < nodes.len()
+                invariant
+                    nodes@ == levels[level_index as int],
+                    children_count_sum(nodes@) <= usize::MAX,
+                    node <= nodes.len(),
+                    first_child == children_count_sum(nodes@.take(node as int)),
+                    ranges@.len() == node,
+                    forall|done_node: int|
+                        0 <= done_node < node ==> {
+                            let range = #[trigger] ranges@[done_node];
+                            &&& range.start == children_count_sum(nodes@.take(done_node))
+                            &&& range.end == range.start + nodes@[done_node]
+                        },
+                decreases nodes.len() - node,
+            {
+                proof {
+                    lemma_children_count_sum_of_one_more(nodes@, node as int);
+                    lemma_children_count_prefix_sum_is_monotone(nodes@, node as int + 1, node as int + 1);
+                }
+                let children_end = first_child + nodes[node];
+                ranges.push(first_child..children_end);
+                first_child = children_end;
+                node += 1;
             }
-            lemma_children_count_sum_of_one_more(nodes@, index as int);
-            lemma_children_count_prefix_sum_is_monotone(nodes@, index as int + 1, index as int + 1);
+            let ghost children_before = children@;
+            children.push(ranges);
+            proof {
+                assert forall|done_level_index: int, done_node: int|
+                    0 <= done_level_index < level_index + 1 && #[trigger] node_is_in_the_levels(levels, done_level_index, done_node)
+                        implies {
+                        let range = children@[done_level_index]@[done_node];
+                        &&& range.start == children_count_sum(levels[done_level_index].take(done_node))
+                        &&& range.end == range.start + levels[done_level_index][done_node]
+                    } by {
+                    if done_level_index < level_index {
+                        assert(children@[done_level_index] == children_before[done_level_index]);
+                    }
+                }
+            }
+            level_index += 1;
         }
-        let first_child = children_count_sum_of_the_first(nodes, index);
-        first_child..first_child + nodes[index]
+        children
     }
 
-    /// 这几片容器（叶序）的祖先的并，按 bump 次序（层级升序、同层从左到右，根最末）：这几片被重写时 COW 的就是它们（叶 + 全部祖先）。
-    /// 只有最右那一片时就是最右那一条路径（每一层最右那一个节点）。没有容器时空。
-    ///
-    /// 一层一层往上标：先标出给的容器，再给每一层的每个节点看它罩住的孩子里有没有标过的，有就标上它、记下它的位置。
-    /// 「在这张单子里就是某一片的祖先」由 [`lemma_ancestors_below_level_contains`] 推。
+    /// 每一层有几个节点：这次发布把上一版的形状记成这几个数（[`ContainerTreeNodeCounts`]），不照抄每个节点的孩子数。
+    #[must_use]
+    pub fn node_counts(&self) -> (node_counts: ContainerTreeNodeCounts)
+        ensures
+            node_counts.counts() == node_counts_of(self.levels()),
+    {
+        let ghost levels = self.levels();
+        let mut nodes_at_each_level: Vec<usize> = Vec::new();
+        let mut level_index: usize = 0;
+        while level_index < self.children_of_each_node_at_each_level.len()
+            invariant
+                levels == self.levels(),
+                levels.len() == self.children_of_each_node_at_each_level@.len(),
+                level_index <= levels.len(),
+                nodes_at_each_level@.len() == level_index,
+                forall|done_level_index: int|
+                    0 <= done_level_index < level_index ==> #[trigger] nodes_at_each_level@[done_level_index] as int
+                        == levels[done_level_index].len(),
+            decreases self.children_of_each_node_at_each_level.len() - level_index,
+        {
+            proof {
+                assert(levels[level_index as int] == self.children_of_each_node_at_each_level@[level_index as int]@);
+            }
+            nodes_at_each_level.push(self.children_of_each_node_at_each_level[level_index].len());
+            level_index += 1;
+        }
+        let node_counts = ContainerTreeNodeCounts { nodes_at_each_level };
+        proof {
+            assert(node_counts.counts() =~= node_counts_of(levels));
+        }
+        node_counts
+    }
+
+    /// 最右端那 `rightmost_count` 片容器在每一层的祖先：每一层都是这一层最右端的一段（一段后缀），交回每层那一段的起点（层下标 0 在前）。
+    /// 一层一层往上，每层被标过的是下一层从某一个起往右的全部（层级 1 之下就是这几片容器）：从这一层最右端往左逐个减去孩子数，减到不大于
+    /// 被标过的头一个就停——停的那个节点罩住它，往右的每个节点都罩着被标过的，往左的一个都不罩——这一层的那一段就从停的那个节点起，
+    /// 上一层被标过的也就从它起。每层只读最右端到那一段起点的几个节点：给的片数是 k、扇出是 f 时每层不超过 ⌈k ÷ f⌉ + 1 个，代价只随
+    /// 给的片数与层数涨、不随容器总数涨（它在发布路径上，运行时决策路径，`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」）。
+    /// 发布里重写的就是最右端连着的几片（[`rewritten_containers_are_the_rightmost`]），所以只收这样的几片、不收散在左边的。
     ///
     /// # Panics
-    /// 某个下标不小于容器数（`requires` 排除的那一种）：调用方只拿这一版里的容器问。
+    /// 给的片数为 0 或多于容器数（`requires` 排除的那一种）：调用方只拿这一版里重写了的几片问。
     #[must_use]
-    pub fn ancestors_of_containers(&self, containers: &[usize]) -> (ancestors: Vec<ContainerTreeNodePosition>)
+    pub fn ancestor_suffix_starts_of_the_rightmost_containers(&self, rightmost_count: usize) -> (starts: Vec<usize>)
         requires
-            shape_fits_the_position_fields(self.levels()),
-            forall|listed: int| 0 <= listed < containers.len() ==> #[trigger] containers[listed] < container_count(self.levels()),
+            1 <= rightmost_count <= container_count(self.levels()),
         ensures
-            ancestors@ == ancestors_below_level(self.levels(), containers@, self.levels().len() as int),
+            starts@.len() == self.levels().len(),
+            forall|level_index: int| 0 <= level_index < starts@.len() ==> #[trigger] starts@[level_index] < self.levels()[level_index].len(),
+            forall|level_index: int, node: int|
+                #[trigger] node_is_in_the_levels(self.levels(), level_index, node) ==> (node_is_an_ancestor_of_one_of(
+                    self.levels(),
+                    level_index,
+                    node,
+                    rightmost_containers(container_count(self.levels()), rightmost_count as int),
+                ) <==> node >= starts@[level_index]),
     {
         proof {
             use_type_invariant(self);
         }
         let ghost levels = self.levels();
+        let ghost containers = rightmost_containers(container_count(levels), rightmost_count as int);
         let container_total = self.containers();
-        let mut marked_below: Vec<bool> = Vec::new();
-        while marked_below.len() < container_total
-            invariant
-                marked_below.len() <= container_total,
-                forall|container: int| 0 <= container < marked_below.len() ==> !(#[trigger] marked_below@[container]),
-            decreases container_total - marked_below.len(),
-        {
-            marked_below.push(false);
-        }
-        let mut listed: usize = 0;
-        while listed < containers.len()
-            invariant
-                listed <= containers.len(),
-                container_total == container_count(levels),
-                forall|any_listed: int| 0 <= any_listed < containers.len() ==> #[trigger] containers[any_listed] < container_total,
-                marked_below.len() == container_total,
-                forall|container: int|
-                    0 <= container < container_total ==> (#[trigger] marked_below@[container] <==> exists|done: int|
-                        0 <= done < listed && #[trigger] containers[done] == container),
-            decreases containers.len() - listed,
-        {
-            let container = containers[listed];
-            assert!(
-                container < container_total,
-                "问祖先的容器 {container} 在这一版里（{container_total} 片）"
-            );
-            marked_below[container] = true;
-            listed += 1;
-            proof {
-                assert forall|any_container: int| 0 <= any_container < container_total implies (#[trigger] marked_below@[any_container]
-                    <==> exists|done: int| 0 <= done < listed && #[trigger] containers[done] == any_container) by {
-                    if any_container == container {
-                        assert(containers[listed - 1] == any_container);
-                    }
-                }
-            }
-        }
+        assert!(
+            1 <= rightmost_count && rightmost_count <= container_total,
+            "问祖先的是这一版最右端的 {rightmost_count} 片（{container_total} 片里）"
+        );
+        let mut first_marked: usize = container_total - rightmost_count;
         proof {
-            assert forall|container: int| 0 <= container < container_total implies (#[trigger] marked_below@[container]
-                == child_is_marked(levels, 0, container, containers@)) by {}
+            assert forall|child: int| 0 <= child < container_total implies (#[trigger] child_is_marked(levels, 0, child, containers)
+                <==> child >= first_marked) by {
+                lemma_rightmost_containers_hold_exactly_the_ones_from_their_first(
+                    container_count(levels),
+                    rightmost_count as int,
+                    child,
+                );
+            }
+            assert forall|listed: int| 0 <= listed < containers.len() implies #[trigger] containers[listed] < container_count(levels) by {}
         }
-        let mut ancestors: Vec<ContainerTreeNodePosition> = Vec::new();
+        let mut starts: Vec<usize> = Vec::new();
+        let mut width: usize = container_total;
         let mut level_index: usize = 0;
         while level_index < self.children_of_each_node_at_each_level.len()
             invariant
                 levels == self.levels(),
                 container_tree_shape_is_well_formed(levels),
-                shape_fits_the_position_fields(levels),
+                containers == rightmost_containers(container_count(levels), rightmost_count as int),
                 forall|listed: int| 0 <= listed < containers.len() ==> #[trigger] containers[listed] < container_count(levels),
                 level_index <= levels.len(),
-                marked_below.len() == if level_index == 0 {
+                width == if level_index == 0 {
                     container_count(levels)
                 } else {
                     levels[level_index - 1].len() as int
                 },
+                first_marked < width,
                 forall|child: int|
-                    0 <= child < marked_below.len() ==> (#[trigger] marked_below@[child] == child_is_marked(
-                        levels,
-                        level_index as int,
-                        child,
-                        containers@,
-                    )),
-                ancestors@ == ancestors_below_level(levels, containers@, level_index as int),
+                    0 <= child < width ==> (#[trigger] child_is_marked(levels, level_index as int, child, containers) <==> child
+                        >= first_marked),
+                starts@.len() == level_index,
+                forall|done_level_index: int|
+                    0 <= done_level_index < level_index ==> #[trigger] starts@[done_level_index] < levels[done_level_index].len(),
+                forall|done_level_index: int, node: int|
+                    0 <= done_level_index < level_index && #[trigger] node_is_in_the_levels(levels, done_level_index, node) ==> (
+                    node_is_an_ancestor_of_one_of(levels, done_level_index, node, containers) <==> node >= starts@[done_level_index]),
             decreases self.children_of_each_node_at_each_level.len() - level_index,
         {
-            let level = u8::try_from(level_index + 1).expect("层数在构造时判过装得进一字节");
             let nodes = &self.children_of_each_node_at_each_level[level_index];
             proof {
-                lemma_every_level_has_at_most_as_many_nodes_as_containers(levels, level_index as int);
                 assert(nodes@ == levels[level_index as int]);
                 if level_index == 0 {
-                    assert(children_count_sum(nodes@) == marked_below.len());
+                    assert(children_count_sum(nodes@) == width);
                 } else {
                     assert(children_count_sum(levels[level_index as int]) == levels[level_index - 1].len());
                 }
+                assert(nodes@.take(nodes@.len() as int) =~= nodes@);
             }
-            let ghost ancestors_below = ancestors@;
-            let mut marked_here: Vec<bool> = Vec::new();
-            let mut first_child: usize = 0;
-            let mut node: usize = 0;
-            while node < nodes.len()
+            // 从最右端往左逐个减去孩子数，减到不大于被标过的头一个就停（迭代上界是这一层的节点数）。
+            let mut first_scanned: usize = nodes.len();
+            let mut first_child: usize = width;
+            while first_child > first_marked
                 invariant
                     nodes@ == levels[level_index as int],
-                    level_index < levels.len(),
-                    level == level_index + 1,
-                    nodes@.len() <= u32::MAX,
-                    children_count_sum(nodes@) == marked_below.len(),
-                    forall|child: int|
-                        0 <= child < marked_below.len() ==> (#[trigger] marked_below@[child] == child_is_marked(
-                            levels,
-                            level_index as int,
-                            child,
-                            containers@,
-                        )),
-                    container_tree_shape_is_well_formed(levels),
-                    shape_fits_the_position_fields(levels),
-                    forall|listed: int| 0 <= listed < containers.len() ==> #[trigger] containers[listed] < container_count(levels),
-                    node <= nodes.len(),
-                    first_child == children_count_sum(nodes@.take(node as int)),
-                    marked_here.len() == node,
-                    forall|done_node: int|
-                        0 <= done_node < node ==> (#[trigger] marked_here@[done_node] == node_is_an_ancestor_of_one_of(
-                            levels,
-                            level_index as int,
-                            done_node,
-                            containers@,
-                        )),
-                    ancestors_below == ancestors_below_level(levels, containers@, level_index as int),
-                    ancestors@ == ancestors_below + ancestors_in_level(levels, level_index as int, containers@, node as int),
-                decreases nodes.len() - node,
+                    children_count_sum(nodes@) == width,
+                    first_marked < width,
+                    first_scanned <= nodes@.len(),
+                    first_child == children_count_sum(nodes@.take(first_scanned as int)),
+                    first_scanned == nodes@.len() || first_marked < children_count_sum(nodes@.take(first_scanned as int + 1)),
+                decreases first_scanned,
             {
                 proof {
-                    lemma_children_count_sum_of_one_more(nodes@, node as int);
-                    lemma_children_count_prefix_sum_is_monotone(nodes@, node as int + 1, node as int + 1);
+                    assert(nodes@.take(0) =~= Seq::<usize>::empty());
+                    assert(first_scanned > 0);
+                    lemma_children_count_sum_of_one_more(nodes@, first_scanned as int - 1);
+                    lemma_children_count_sum_is_not_negative(nodes@.take(first_scanned as int - 1));
                 }
-                let children_end = first_child + nodes[node];
-                let mut is_marked = false;
-                let mut child = first_child;
-                while child < children_end
-                    invariant
-                        first_child <= child <= children_end <= marked_below.len(),
-                        children_end == first_child + nodes@[node as int],
-                        is_marked <==> exists|seen: int| first_child <= seen < child && #[trigger] marked_below@[seen],
-                    decreases children_end - child,
-                {
-                    if marked_below[child] {
-                        is_marked = true;
-                    }
-                    child += 1;
-                }
-                proof {
-                    lemma_node_is_marked_by_its_children(levels, level_index as int, node as int, containers@);
-                    if is_marked {
-                        let seen = choose|seen: int| first_child <= seen < children_end && #[trigger] marked_below@[seen];
-                        assert(child_lies_under_node(levels[level_index as int], node as int, seen));
-                        assert(child_is_marked(levels, level_index as int, seen, containers@));
-                    }
-                    if node_is_an_ancestor_of_one_of(levels, level_index as int, node as int, containers@) {
-                        let seen = choose|seen: int|
-                            child_lies_under_node(levels[level_index as int], node as int, seen) && #[trigger] child_is_marked(
+                first_scanned -= 1;
+                first_child -= nodes[first_scanned];
+            }
+            proof {
+                assert(first_scanned < nodes@.len());
+                assert forall|unscanned: int| 0 <= unscanned < first_scanned implies !(#[trigger] node_is_an_ancestor_of_one_of(
+                    levels,
+                    level_index as int,
+                    unscanned,
+                    containers,
+                )) by {
+                    lemma_node_is_marked_by_its_children(levels, level_index as int, unscanned, containers);
+                    if node_is_an_ancestor_of_one_of(levels, level_index as int, unscanned, containers) {
+                        let child = choose|child: int|
+                            child_lies_under_node(levels[level_index as int], unscanned, child) && #[trigger] child_is_marked(
                                 levels,
                                 level_index as int,
-                                seen,
-                                containers@,
+                                child,
+                                containers,
                             );
-                        assert(marked_below@[seen]);
+                        lemma_children_count_prefix_sum_is_monotone(nodes@, unscanned + 1, first_scanned as int);
+                        lemma_children_count_sum_is_not_negative(nodes@.take(unscanned));
+                        assert(0 <= child < width);
+                        assert(child >= first_marked);
                     }
                 }
-                marked_here.push(is_marked);
-                if is_marked {
-                    let position = ContainerTreeNodePosition {
-                        level,
-                        index_in_level: u32::try_from(node).expect("同层节点数装得进 u32"),
-                    };
-                    proof {
-                        assert(position == position_at(level_index as int, node as int));
+                assert forall|scanned: int| first_scanned <= scanned < nodes@.len() implies #[trigger] node_is_an_ancestor_of_one_of(
+                    levels,
+                    level_index as int,
+                    scanned,
+                    containers,
+                ) by {
+                    lemma_node_is_marked_by_its_children(levels, level_index as int, scanned, containers);
+                    lemma_children_count_sum_of_one_more(nodes@, scanned);
+                    lemma_children_count_prefix_sum_is_monotone(nodes@, first_scanned as int + 1, scanned + 1);
+                    lemma_children_count_prefix_sum_is_monotone(nodes@, scanned + 1, nodes@.len() as int);
+                    assert(levels[level_index as int][scanned] >= 1);
+                    let child = children_count_sum(nodes@.take(scanned + 1)) - 1;
+                    assert(child_lies_under_node(levels[level_index as int], scanned, child));
+                    assert(0 <= child < width);
+                    assert(child >= first_marked);
+                    assert(child_is_marked(levels, level_index as int, child, containers));
+                }
+            }
+            let ghost starts_before = starts@;
+            starts.push(first_scanned);
+            proof {
+                assert forall|done_level_index: int, node: int|
+                    0 <= done_level_index < level_index + 1 && #[trigger] node_is_in_the_levels(levels, done_level_index, node) implies (
+                    node_is_an_ancestor_of_one_of(levels, done_level_index, node, containers) <==> node >= starts@[done_level_index]) by {
+                    if done_level_index < level_index {
+                        assert(starts@[done_level_index] == starts_before[done_level_index]);
                     }
-                    ancestors.push(position);
                 }
-                first_child = children_end;
-                node += 1;
-                proof {
-                    assert(ancestors@ =~= ancestors_below + ancestors_in_level(levels, level_index as int, containers@, node as int));
+                assert forall|done_level_index: int| 0 <= done_level_index < level_index + 1 implies #[trigger] starts@[done_level_index]
+                    < levels[done_level_index].len() by {
+                    if done_level_index < level_index {
+                        assert(starts@[done_level_index] == starts_before[done_level_index]);
+                    }
                 }
             }
-            marked_below = marked_here;
+            first_marked = first_scanned;
+            width = nodes.len();
             level_index += 1;
             proof {
-                assert forall|child: int| 0 <= child < marked_below.len() implies (#[trigger] marked_below@[child] == child_is_marked(
+                assert forall|child: int| 0 <= child < width implies (#[trigger] child_is_marked(
                     levels,
                     level_index as int,
                     child,
-                    containers@,
-                )) by {}
+                    containers,
+                ) <==> child >= first_marked) by {
+                    assert(node_is_in_the_levels(levels, level_index as int - 1, child));
+                }
             }
         }
-        ancestors
+        starts
     }
 
-    /// 最右端新开一片容器之后的形状（末尾分裂、根满长高，D8（核心索引结构） 已定项 6）：最右那个层级 1 节点装得下（孩子少于
-    /// `entries_per_node`）就挂进去；装满了就在这一层最右端新开一个只有它一个孩子的节点，再往上一层给最右那个节点添这个新孩子，
-    /// 一层层往上；最高那一层（根）也满了就再长一层，新根两个孩子（旧根、新开的右半）。零根时第一片容器开一个层级 1 的根。
-    /// 左边的节点一个都不动。
-    ///
-    /// 先从层级 1 往上找停的那一层（最右节点装得下的第一层，都满就停在根那一层），再逐层照抄：停的那一层之下最右端各加一个
-    /// 只有一个孩子的节点，停的那一层装得下就给最右节点加一个孩子、装不下就也加一个节点并再长一层两个孩子的新根。
+    /// 最右端那 `rightmost_count` 片容器的祖先的并，按 bump 次序（层级升序、每层那一段从左到右，根最末）：这几片被重写时 COW 的就是它们
+    /// （叶 + 全部祖先）。只有最右那一片时就是最右那一条路径（每一层最右那一个节点）。每层那一段是一段后缀
+    /// （[`Self::ancestor_suffix_starts_of_the_rightmost_containers`]），读的节点只有交回的这几个。
+    /// 「在这张单子里就是某一片的祖先」由 [`lemma_ancestors_below_level_contains`] 推。
     ///
     /// # Panics
-    /// `entries_per_node` 小于 2（新根装不下两个孩子）：开关的构造函数判过。
+    /// 给的片数为 0 或多于容器数（`requires` 排除的那一种）。
     #[must_use]
-    pub fn after_a_container_is_opened_at_the_right_end(&self, entries_per_node: usize) -> (opened: Self)
+    pub fn ancestors_of_the_rightmost_containers(&self, rightmost_count: usize) -> (ancestors: Vec<ContainerTreeNodePosition>)
         requires
-            entries_per_node >= 2,
-            container_count(self.levels()) + 1 <= usize::MAX,
+            shape_fits_the_position_fields(self.levels()),
+            1 <= rightmost_count <= container_count(self.levels()),
         ensures
-            opened.levels() == levels_after_opening_at_the_right_end(self.levels(), entries_per_node as int),
-            container_count(opened.levels()) == container_count(self.levels()) + 1,
+            ancestors@ == ancestors_below_level(
+                self.levels(),
+                rightmost_containers(container_count(self.levels()), rightmost_count as int),
+                self.levels().len() as int,
+            ),
     {
         proof {
             use_type_invariant(self);
-            lemma_opening_keeps_the_shape_well_formed(self.levels(), entries_per_node as int);
         }
-        assert!(
-            entries_per_node >= 2,
-            "容器树一个码 2 节点至少装 2 条：长高那次的新根要装得下左右两半"
-        );
-        let level_count = self.children_of_each_node_at_each_level.len();
+        let ghost levels = self.levels();
+        let ghost containers = rightmost_containers(container_count(levels), rightmost_count as int);
+        let starts = self.ancestor_suffix_starts_of_the_rightmost_containers(rightmost_count);
+        let mut ancestors: Vec<ContainerTreeNodePosition> = Vec::new();
+        let mut level_index: usize = 0;
+        while level_index < self.children_of_each_node_at_each_level.len()
+            invariant
+                levels == self.levels(),
+                container_tree_shape_is_well_formed(levels),
+                shape_fits_the_position_fields(levels),
+                containers == rightmost_containers(container_count(levels), rightmost_count as int),
+                starts@.len() == levels.len(),
+                forall|any_level_index: int|
+                    0 <= any_level_index < starts@.len() ==> #[trigger] starts@[any_level_index] < levels[any_level_index].len(),
+                forall|any_level_index: int, node: int|
+                    #[trigger] node_is_in_the_levels(levels, any_level_index, node) ==> (node_is_an_ancestor_of_one_of(
+                        levels,
+                        any_level_index,
+                        node,
+                        containers,
+                    ) <==> node >= starts@[any_level_index]),
+                level_index <= levels.len(),
+                ancestors@ == ancestors_below_level(levels, containers, level_index as int),
+            decreases self.children_of_each_node_at_each_level.len() - level_index,
+        {
+            let level = u8::try_from(level_index + 1).expect("层数在构造时判过装得进一字节");
+            let node_count = self.children_of_each_node_at_each_level[level_index].len();
+            let first_ancestor = starts[level_index];
+            proof {
+                lemma_every_level_has_at_most_as_many_nodes_as_containers(levels, level_index as int);
+                assert(node_count == levels[level_index as int].len());
+                assert forall|unlisted: int| 0 <= unlisted < first_ancestor implies !(#[trigger] node_is_an_ancestor_of_one_of(
+                    levels,
+                    level_index as int,
+                    unlisted,
+                    containers,
+                )) by {
+                    assert(node_is_in_the_levels(levels, level_index as int, unlisted));
+                }
+                lemma_no_ancestor_among_the_first(levels, level_index as int, containers, first_ancestor as int);
+            }
+            let ghost ancestors_below = ancestors@;
+            proof {
+                assert(ancestors@ =~= ancestors_below + ancestors_in_level(levels, level_index as int, containers, first_ancestor as int));
+            }
+            let mut node: usize = first_ancestor;
+            while node < node_count
+                invariant
+                    levels == self.levels(),
+                    shape_fits_the_position_fields(levels),
+                    container_tree_shape_is_well_formed(levels),
+                    level_index < levels.len(),
+                    level == level_index + 1,
+                    node_count == levels[level_index as int].len(),
+                    node_count <= u32::MAX,
+                    first_ancestor <= node <= node_count,
+                    forall|any_node: int|
+                        #[trigger] node_is_in_the_levels(levels, level_index as int, any_node) ==> (node_is_an_ancestor_of_one_of(
+                            levels,
+                            level_index as int,
+                            any_node,
+                            containers,
+                        ) <==> any_node >= first_ancestor),
+                    ancestors@ == ancestors_below + ancestors_in_level(levels, level_index as int, containers, node as int),
+                decreases node_count - node,
+            {
+                let position = ContainerTreeNodePosition {
+                    level,
+                    index_in_level: u32::try_from(node).expect("同层节点数装得进 u32"),
+                };
+                proof {
+                    assert(position == position_at(level_index as int, node as int));
+                    assert(node_is_in_the_levels(levels, level_index as int, node as int));
+                    assert(node_is_an_ancestor_of_one_of(levels, level_index as int, node as int, containers));
+                }
+                ancestors.push(position);
+                node += 1;
+                proof {
+                    assert(ancestors@ =~= ancestors_below + ancestors_in_level(levels, level_index as int, containers, node as int));
+                }
+            }
+            level_index += 1;
+        }
+        ancestors
+    }
+
+    /// 最右端新开一片容器之后的形状（末尾分裂、根满长高，D8（核心索引结构） 已定项 6）：最右那个层级 1 节点装得下（孩子少于
+    /// `entries_per_node`）就挂进去；装满了就在这一层最右端新开一个只有它一个孩子的节点，再往上一层给最右那个节点添这个新孩子，
+    /// 一层层往上；最高那一层（根）也满了就再长一层，新根两个孩子（旧根、新开的右半）。零根时第一片容器开一个层级 1 的根。
+    /// 左边的节点一个都不动。
+    ///
+    /// 先从层级 1 往上找停的那一层（最右节点装得下的第一层，都满就停在根那一层），再只改停的那一层与它之下各层的最右端：之下各层末尾各加一个
+    /// 只有一个孩子的节点，停的那一层装得下就给最右节点加一个孩子、装不下就也加一个节点并再长一层两个孩子的新根。形状按值交进来、原地改：
+    /// 它在发布路径上（运行时决策路径，`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」：代价不许随盘容量增长），
+    /// 每开一片只碰停的那一层及其之下各层的末尾，不逐层照抄全部节点，代价只随层数涨。
+    ///
+    /// # Panics
+    /// `entries_per_node` 小于 2（新根装不下两个孩子）：开关的构造函数判过。
+    #[must_use]
+    pub fn after_a_container_is_opened_at_the_right_end(self, entries_per_node: usize) -> (opened: Self)
+        requires
+            entries_per_node >= 2,
+            container_count(self.levels()) + 1 <= usize::MAX,
+        ensures
+            opened.levels() == levels_after_opening_at_the_right_end(self.levels(), entries_per_node as int),
+            container_count(opened.levels()) == container_count(self.levels()) + 1,
+    {
+        proof {
+            use_type_invariant(&self);
+            lemma_opening_keeps_the_shape_well_formed(self.levels(), entries_per_node as int);
+        }
+        assert!(
+            entries_per_node >= 2,
+            "容器树一个码 2 节点至少装 2 条：长高那次的新根要装得下左右两半"
+        );
+        let ghost levels = self.levels();
+        let Self { children_of_each_node_at_each_level: levels_before_opening, container_total: containers_before_opening } = self;
+        let container_total = containers_before_opening;
+        let mut levels_after = levels_before_opening;
+        let level_count = levels_after.len();
         if level_count == 0 {
             // 零根：第一片容器开一个层级 1 的根。
-            let levels_after = vec![level_of_a_single_node(1)];
+            levels_after.push(level_of_a_single_node(1));
             proof {
                 assert(levels_of_children_counts(levels_after@) =~= levels_after_opening_at_the_right_end(
-                    self.levels(),
+                    levels,
                     entries_per_node as int,
                 ));
             }
-            return Self { children_of_each_node_at_each_level: levels_after };
+            return Self { children_of_each_node_at_each_level: levels_after, container_total: container_total + 1 };
         }
-        let ghost levels = self.levels();
         proof {
             lemma_opening_stop_level(levels, entries_per_node as int, 0);
-            assert(levels[0] == self.children_of_each_node_at_each_level@[0]@);
+            assert(levels[0] == levels_after@[0]@);
         }
         // 迭代上界是层数：每一轮往上一层，到最右节点装得下的那一层、或最高那一层停。
         let mut stop: usize = 0;
-        while stop + 1 < level_count && self.children_of_each_node_at_each_level[stop][self.children_of_each_node_at_each_level[stop].len() - 1]
-            >= entries_per_node
+        while stop + 1 < level_count && levels_after[stop][levels_after[stop].len() - 1] >= entries_per_node
             invariant
-                levels == self.levels(),
+                levels == levels_of_children_counts(levels_after@),
                 container_tree_shape_is_well_formed(levels),
                 level_count == levels.len(),
                 stop < level_count,
@@ -2123,32 +2418,30 @@ impl ContainerTreeShape {
             decreases level_count - stop,
         {
             proof {
-                assert(levels[stop as int] == self.children_of_each_node_at_each_level@[stop as int]@);
+                assert(levels[stop as int] == levels_after@[stop as int]@);
             }
             stop += 1;
             proof {
-                assert(levels[stop as int] == self.children_of_each_node_at_each_level@[stop as int]@);
+                assert(levels[stop as int] == levels_after@[stop as int]@);
             }
         }
-        let stop_nodes = &self.children_of_each_node_at_each_level[stop];
         proof {
-            assert(levels[stop as int] == stop_nodes@);
+            assert(levels[stop as int] == levels_after@[stop as int]@);
             assert(levels[stop as int].len() >= 1);
         }
-        let grows = stop_nodes[stop_nodes.len() - 1] >= entries_per_node;
-        let mut levels_after: Vec<Vec<usize>> = Vec::new();
+        let grows = levels_after[stop][levels_after[stop].len() - 1] >= entries_per_node;
+        // 只改停的那一层与它之下各层：迭代上界是 stop + 1（不超过层数），每一轮只碰这一层的最右端；停的那一层之上原样留着。
         let mut level_index: usize = 0;
-        while level_index < level_count
+        while level_index <= stop
             invariant
-                levels == self.levels(),
                 container_tree_shape_is_well_formed(levels),
                 level_count == levels.len(),
+                levels_after.len() == level_count,
                 stop == opening_stop_level_from(levels, entries_per_node as int, 0),
                 stop < level_count,
                 levels[stop as int].len() >= 1,
                 grows == (levels[stop as int].last() >= entries_per_node),
-                level_index <= level_count,
-                levels_after.len() == level_index,
+                level_index <= stop + 1,
                 forall|done_level_index: int|
                     0 <= done_level_index < level_index ==> (#[trigger] levels_after@[done_level_index])@ == level_after_opening(
                         levels,
@@ -2156,25 +2449,34 @@ impl ContainerTreeShape {
                         grows,
                         done_level_index,
                     ),
-            decreases level_count - level_index,
+                forall|untouched_level_index: int|
+                    level_index <= untouched_level_index < level_count ==> (#[trigger] levels_after@[untouched_level_index])@
+                        == levels[untouched_level_index],
+            decreases stop + 1 - level_index,
         {
-            let mut nodes = copy_of_one_level(&self.children_of_each_node_at_each_level[level_index]);
             proof {
-                assert(nodes@ == levels[level_index as int]);
-                assert(nodes@.len() >= 1);
+                assert(levels_after@[level_index as int]@ == levels[level_index as int]);
+                assert(levels[level_index as int].len() >= 1);
             }
-            if level_index < stop || (level_index == stop && grows) {
-                nodes.push(1);
-            } else if level_index == stop {
-                let rightmost = nodes.len() - 1;
-                nodes[rightmost] += 1;
+            if level_index < stop || grows {
+                levels_after[level_index].push(1);
+            } else {
+                let rightmost = levels_after[level_index].len() - 1;
+                proof {
+                    assert(levels_after@[level_index as int]@[rightmost as int] < entries_per_node);
+                }
+                levels_after[level_index][rightmost] += 1;
             }
             proof {
-                assert(nodes@ =~= level_after_opening(levels, stop as int, grows, level_index as int));
+                assert(levels_after@[level_index as int]@ =~= level_after_opening(levels, stop as int, grows, level_index as int));
             }
-            levels_after.push(nodes);
             level_index += 1;
         }
+        proof {
+            assert forall|untouched_level_index: int| stop < untouched_level_index < level_count implies (#[trigger] levels_after@[
+                untouched_level_index
+            ])@ == level_after_opening(levels, stop as int, grows, untouched_level_index) by {}
+        }
         if grows {
             // 根也满了：长一层，新根挂两个孩子（旧根、刚在最右端新开的右半）。
             levels_after.push(level_of_a_single_node(2));
@@ -2185,7 +2487,85 @@ impl ContainerTreeShape {
                 entries_per_node as int,
             ));
         }
-        Self { children_of_each_node_at_each_level: levels_after }
+        Self { children_of_each_node_at_each_level: levels_after, container_total: container_total + 1 }
+    }
+}
+
+/// 一版容器树每一层有几个码 2 节点（层级 1 在前、根那一层在最后；零根时一层都没有）。这次发布对上一版的形状只问四样：某个位置上
+/// 有没有节点、它是不是根、根在哪、根的层级（[`ContainerTreeAfterThisPublish::node_origins`]、
+/// [`ContainerTreeAfterThisPublish::grew_a_level_this_publish`]、`crate::transaction` 按上一版的角色认照抄的节点），四样都只看层数与
+/// 每层节点数，所以上一版只记这几个数，不照抄每个节点的孩子数：它在发布路径上（运行时决策路径，`.claude/rules/fs-design.md`
+/// 「记账是事务的副产品，不是事后的遍历」），大小只随层数涨、不随节点总数涨。
+#[derive(Clone, Debug, PartialEq, Eq)]
+#[verifier::allow(autoderive_clone_without_spec)]
+pub struct ContainerTreeNodeCounts {
+    nodes_at_each_level: Vec<usize>,
+}
+
+impl ContainerTreeNodeCounts {
+    /// 每层的节点数（规格里用）。
+    pub closed spec fn counts(self) -> Seq<int> {
+        Seq::new(self.nodes_at_each_level@.len(), |level_index: int| self.nodes_at_each_level@[level_index] as int)
+    }
+
+    /// 根节点码 2 头里的层级：零根 0，根的孩子是容器时 1，长高一次加一（层数）。
+    ///
+    /// # Panics
+    /// 层数装不进一字节（`requires` 排除的那一种）：出自 [`ContainerTreeShape::node_counts`]，那份形状构造时判过。
+    #[must_use]
+    pub fn root_level(&self) -> (root_level: u8)
+        requires
+            self.counts().len() <= 255,
+        ensures
+            root_level == self.counts().len(),
+    {
+        u8::try_from(self.nodes_at_each_level.len()).expect("上一版形状的层数在那份形状构造时判过装得进码 2 头的一字节层级")
+    }
+
+    /// 根的位置（最高那一层的第 0 个）；零根时 `None`。
+    #[must_use]
+    pub fn root_position(&self) -> (root_position: Option<ContainerTreeNodePosition>)
+        requires
+            self.counts().len() <= 255,
+        ensures
+            self.counts().len() == 0 ==> root_position is None,
+            self.counts().len() > 0 ==> root_position == Some(position_at(self.counts().len() - 1, 0)),
+    {
+        match self.root_level() {
+            0 => None,
+            root_level => Some(ContainerTreeNodePosition { level: root_level, index_in_level: 0 }),
+        }
+    }
+
+    /// 这个位置上的节点是不是那一版的根。
+    #[must_use]
+    pub fn is_the_root(&self, position: ContainerTreeNodePosition) -> (is_the_root: bool)
+        requires
+            self.counts().len() <= 255,
+        ensures
+            is_the_root == (self.counts().len() > 0 && position == position_at(self.counts().len() - 1, 0)),
+    {
+        match self.root_position() {
+            Some(root_of_that_version) => position.level == root_of_that_version.level && position.index_in_level
+                == root_of_that_version.index_in_level,
+            None => false,
+        }
+    }
+
+    /// 这个位置在那一版里有没有节点。
+    #[must_use]
+    pub fn has_a_node_at(&self, position: ContainerTreeNodePosition) -> (has_a_node: bool)
+        ensures
+            has_a_node == position_is_counted(self.counts(), position),
+    {
+        let level = usize::from(position.level);
+        if level < 1 || level > self.nodes_at_each_level.len() {
+            return false;
+        }
+        match usize::try_from(position.index_in_level) {
+            Ok(index) => index < self.nodes_at_each_level[level - 1],
+            Err(_) => false,
+        }
     }
 }
 
@@ -2220,15 +2600,16 @@ pub enum ContainerTreeNodeOrigin {
     CarriedFromThePreviousVersion,
 }
 
-/// 这次发布之后一棵容器树的样子：叶容器（按叶序）、这次重写哪几片（叶序）、这次之后与这次之前的形状。
-/// 码 2 节点重写哪几个由它们推：有容器重写时最右那一条路径（COW 叶 + 祖先），没有时一个都不重写。
+/// 这次发布之后一棵容器树的样子：叶容器（按叶序）、这次重写哪几片（叶序）、这次之后的形状与这次之前那一版每层的节点数。
+/// 码 2 节点重写哪几个由它们推：有容器重写时是重写的那几片的全部祖先（COW 叶 + 祖先），没有时一个都不重写。
+/// 这次之前那一版只记每层节点数（[`ContainerTreeNodeCounts`]）：发布对它只问哪个位置上有节点、哪个是根，不照抄它的形状。
 #[derive(Clone, Debug, PartialEq, Eq)]
 #[verifier::allow(autoderive_clone_without_spec)]
 pub struct ContainerTreeAfterThisPublish<Record> {
     pub containers: Vec<RecordContainer<Record>>,
     pub rewritten_containers: Vec<usize>,
     pub shape: ContainerTreeShape,
-    pub shape_before_this_publish: ContainerTreeShape,
+    pub node_counts_before_this_publish: ContainerTreeNodeCounts,
 }
 
 /// 两组容器逐片相同：身份相同、记录逐条相同。
@@ -2294,7 +2675,7 @@ fn copy_of_the_containers<Record: Copy>(containers: &[RecordContainer<Record>])
 }
 
 impl<Record: Copy> ContainerTreeAfterThisPublish<Record> {
-    /// 这次不追加的发布：容器、形状都照抄上一版。
+    /// 这次不追加的发布：容器、形状都照抄上一版，这次之前那一版只记每层节点数。
     ///
     /// # Panics
     /// 形状里的容器数与容器的片数不等（`requires` 排除的那一种）：两样出自同一版。
@@ -2306,14 +2687,14 @@ impl<Record: Copy> ContainerTreeAfterThisPublish<Record> {
             carried.containers@ == containers@,
             carried.rewritten_containers@.len() == 0,
             carried.shape.levels() == shape.levels(),
-            carried.shape_before_this_publish.levels() == shape.levels(),
+            carried.node_counts_before_this_publish.counts() == node_counts_of(shape.levels()),
     {
         assert!(
             shape.containers() == containers.len(),
             "形状里层级 1 的孩子数之和就是容器数"
         );
-        let shape_before_this_publish = shape.clone();
-        Self { containers, rewritten_containers: Vec::new(), shape_before_this_publish, shape }
+        let node_counts_before_this_publish = shape.node_counts();
+        Self { containers, rewritten_containers: Vec::new(), shape, node_counts_before_this_publish }
     }
 }
 
@@ -2338,18 +2719,90 @@ impl<Record> ContainerTreeAfterThisPublish<Record> {
         self.shape.internal_levels()
     }
 
+    /// 这次重写的容器从第几片起（重写的是最右端连着的那几片，[`rewritten_containers_are_the_rightmost`]）；一片都没重写时 `None`。
+    /// 发布路径问「第几片这次重写了没有」按它比：序号不小于它就重写了，不在重写清单上逐个找（`crate::transaction` 列映射单元、装容器单元）。
+    ///
+    /// # Panics
+    /// 重写清单不是最右端连着的那几片（`requires` 排除的那一种）：造它的只有 [`append_records_to_the_open_container`] 与 [`Self::carried`]，
+    /// 两处的 `ensures` 都写着它是。逐片核一遍，代价是重写的片数。
+    #[must_use]
+    pub fn first_rewritten_container(&self) -> (first: Option<usize>)
+        requires
+            rewritten_containers_are_the_rightmost(self.rewritten_containers@, self.containers@.len() as int),
+        ensures
+            first is None <==> self.rewritten_containers@.len() == 0,
+            first matches Some(first_position) ==> first_position == self.containers@.len() - self.rewritten_containers@.len()
+                && forall|position: int|
+                0 <= position < self.containers@.len() ==> (#[trigger] self.rewritten_containers@.contains(position as usize)
+                    <==> position >= first_position),
+    {
+        let rightmost_count = self.rightmost_count_of_the_rewritten_containers(self.containers.len());
+        if rightmost_count == 0 {
+            return None;
+        }
+        let first_position = self.containers.len() - rightmost_count;
+        proof {
+            let rewritten = self.rewritten_containers@;
+            assert forall|position: int| 0 <= position < self.containers@.len() implies (#[trigger] rewritten.contains(
+                position as usize,
+            ) <==> position >= first_position) by {
+                if position >= first_position {
+                    assert(rewritten[position - first_position] == position);
+                }
+                if rewritten.contains(position as usize) {
+                    let found = choose|found: int| 0 <= found < rewritten.len() && rewritten[found] == position as usize;
+                    assert(rewritten[found] == first_position + found);
+                }
+            }
+        }
+        Some(first_position)
+    }
+
+    /// 重写清单有几片，连带逐片核它就是最右端连着的那几片（一共 `total` 片）：代价是重写的片数。
+    ///
+    /// # Panics
+    /// 重写清单不是最右端连着的那几片（`requires` 排除的那一种）：见 [`Self::first_rewritten_container`]。
+    fn rightmost_count_of_the_rewritten_containers(&self, total: usize) -> (rightmost_count: usize)
+        requires
+            rewritten_containers_are_the_rightmost(self.rewritten_containers@, total as int),
+        ensures
+            rightmost_count == self.rewritten_containers@.len(),
+    {
+        let rightmost_count = self.rewritten_containers.len();
+        assert!(
+            rightmost_count <= total,
+            "重写的 {rightmost_count} 片是这一版 {total} 片里最右端的那几片"
+        );
+        let mut listed: usize = 0;
+        while listed < rightmost_count
+            invariant
+                rewritten_containers_are_the_rightmost(self.rewritten_containers@, total as int),
+                rightmost_count == self.rewritten_containers@.len(),
+                rightmost_count <= total,
+                listed <= rightmost_count,
+            decreases rightmost_count - listed,
+        {
+            assert!(
+                self.rewritten_containers[listed] == total - rightmost_count + listed,
+                "重写清单是最右端连着的那几片：造它的只有 append_records_to_the_open_container 与 carried"
+            );
+            listed += 1;
+        }
+        rightmost_count
+    }
+
     /// 这次长了一层：上一版有根、这次之后的根层级比它高。那一次上一版的根不重写、换成根之下的节点留在原位置，不被这次换下。
     #[must_use]
     pub fn grew_a_level_this_publish(&self) -> (grew_a_level: bool)
         requires
             self.shape.levels().len() <= 255,
-            self.shape_before_this_publish.levels().len() <= 255,
+            self.node_counts_before_this_publish.counts().len() <= 255,
         ensures
-            grew_a_level == (self.shape_before_this_publish.levels().len() > 0 && self.shape.levels().len()
-                > self.shape_before_this_publish.levels().len()),
+            grew_a_level == (self.node_counts_before_this_publish.counts().len() > 0 && self.shape.levels().len()
+                > self.node_counts_before_this_publish.counts().len()),
     {
-        self.shape_before_this_publish.root_level() > 0
-            && self.shape.root_level() > self.shape_before_this_publish.root_level()
+        self.node_counts_before_this_publish.root_level() > 0
+            && self.shape.root_level() > self.node_counts_before_this_publish.root_level()
     }
 
     /// 这次长了一层、而上一版的根一个字节不动地留在原位置当根之下的节点（它下面没有容器重写）：那一次新根在上一版里没有落点，
@@ -2358,23 +2811,20 @@ impl<Record> ContainerTreeAfterThisPublish<Record> {
     pub fn previous_root_is_carried_down_unchanged(&self) -> (is_carried_down_unchanged: bool)
         requires
             shape_fits_the_position_fields(self.shape.levels()),
-            self.shape_before_this_publish.levels().len() <= 255,
-            forall|listed: int|
-                0 <= listed < self.rewritten_containers.len() ==> #[trigger] self.rewritten_containers@[listed] < container_count(
-                    self.shape.levels(),
-                ),
+            self.node_counts_before_this_publish.counts().len() <= 255,
+            rewritten_containers_are_the_rightmost(self.rewritten_containers@, container_count(self.shape.levels())),
         ensures
-            is_carried_down_unchanged == (self.shape_before_this_publish.levels().len() > 0 && self.shape.levels().len()
-                > self.shape_before_this_publish.levels().len() && !ancestors_below_level(
+            is_carried_down_unchanged == (self.node_counts_before_this_publish.counts().len() > 0 && self.shape.levels().len()
+                > self.node_counts_before_this_publish.counts().len() && !ancestors_below_level(
                 self.shape.levels(),
                 self.rewritten_containers@,
                 self.shape.levels().len() as int,
-            ).contains(position_at(self.shape_before_this_publish.levels().len() - 1, 0))),
+            ).contains(position_at(self.node_counts_before_this_publish.counts().len() - 1, 0))),
     {
         if !self.grew_a_level_this_publish() {
             return false;
         }
-        match self.shape_before_this_publish.root_position() {
+        match self.node_counts_before_this_publish.root_position() {
             Some(previous_root) => {
                 let rewritten_nodes = self.rewritten_nodes();
                 !position_is_listed(&rewritten_nodes, previous_root)
@@ -2383,15 +2833,16 @@ impl<Record> ContainerTreeAfterThisPublish<Record> {
         }
     }
 
-    /// 这次重写的码 2 节点，按 bump 次序（层级升序，根最末）：重写的每片容器的全部祖先（COW 叶 + 祖先）。
+    /// 这次重写的码 2 节点，按 bump 次序（层级升序，根最末）：重写的每片容器的全部祖先（COW 叶 + 祖先）。重写的是最右端连着的那几片，
+    /// 每层的祖先是一段后缀（[`ContainerTreeShape::ancestors_of_the_rightmost_containers`]），只读那几段。
+    ///
+    /// # Panics
+    /// 重写清单不是最右端连着的那几片（`requires` 排除的那一种）：见 [`Self::first_rewritten_container`]。
     #[must_use]
     pub fn rewritten_nodes(&self) -> (rewritten_nodes: Vec<ContainerTreeNodePosition>)
         requires
             shape_fits_the_position_fields(self.shape.levels()),
-            forall|listed: int|
-                0 <= listed < self.rewritten_containers.len() ==> #[trigger] self.rewritten_containers@[listed] < container_count(
-                    self.shape.levels(),
-                ),
+            rewritten_containers_are_the_rightmost(self.rewritten_containers@, container_count(self.shape.levels())),
         ensures
             rewritten_nodes@ == ancestors_below_level(
                 self.shape.levels(),
@@ -2399,23 +2850,33 @@ impl<Record> ContainerTreeAfterThisPublish<Record> {
                 self.shape.levels().len() as int,
             ),
     {
-        self.shape.ancestors_of_containers(self.rewritten_containers.as_slice())
+        let rightmost_count = self.rightmost_count_of_the_rewritten_containers(self.shape.containers());
+        proof {
+            lemma_rewritten_containers_are_the_rightmost_ones(self.rewritten_containers@, container_count(self.shape.levels()));
+        }
+        if rightmost_count == 0 {
+            proof {
+                assert(self.rewritten_containers@ =~= Seq::<usize>::empty());
+                lemma_no_containers_have_no_ancestors(self.shape.levels(), self.shape.levels().len() as int);
+            }
+            return Vec::new();
+        }
+        self.shape.ancestors_of_the_rightmost_containers(rightmost_count)
     }
 
-    /// 这一版每个码 2 节点从哪来，按 bump 次序。
+    /// 这一版每个码 2 节点从哪来，按 bump 次序。这次重写的是重写的那几片的祖先：每层是一段后缀，按每层那一段的起点判
+    /// （[`ContainerTreeShape::ancestor_suffix_starts_of_the_rightmost_containers`]），不在重写的节点清单上逐个找。
     ///
     /// # Panics
     /// 一个照抄的位置在上一版里没有节点（`requires` 排除的那一种）：容器与节点都只在最右端新开，最右那一条路径之外的位置上一版都有。
+    /// 重写清单不是最右端连着的那几片：见 [`Self::first_rewritten_container`]。
     #[must_use]
     pub fn node_origins(&self) -> (node_origins: Vec<(ContainerTreeNodePosition, ContainerTreeNodeOrigin)>)
         requires
             shape_fits_the_position_fields(self.shape.levels()),
-            forall|listed: int|
-                0 <= listed < self.rewritten_containers.len() ==> #[trigger] self.rewritten_containers@[listed] < container_count(
-                    self.shape.levels(),
-                ),
+            rewritten_containers_are_the_rightmost(self.rewritten_containers@, container_count(self.shape.levels())),
             every_node_absent_from_the_previous_version_is_rewritten(
-                self.shape_before_this_publish.levels(),
+                self.node_counts_before_this_publish.counts(),
                 self.shape.levels(),
                 self.rewritten_containers@,
             ),
@@ -2430,8 +2891,8 @@ impl<Record> ContainerTreeAfterThisPublish<Record> {
                         self.rewritten_containers@,
                         self.shape.levels().len() as int,
                     ).contains(origin.0))
-                    &&& origin.1 is CarriedFromThePreviousVersion ==> position_is_in_the_shape(
-                        self.shape_before_this_publish.levels(),
+                    &&& origin.1 is CarriedFromThePreviousVersion ==> position_is_counted(
+                        self.node_counts_before_this_publish.counts(),
                         origin.0,
                     )
                 },
@@ -2446,10 +2907,23 @@ impl<Record> ContainerTreeAfterThisPublish<Record> {
                 self.rewritten_containers@,
                 self.shape.levels().len() as int,
             );
+            lemma_rewritten_containers_are_the_rightmost_ones(self.rewritten_containers@, container_count(self.shape.levels()));
         }
-        let rewritten = self.rewritten_nodes();
-        let positions = self.shape.positions_in_bump_order();
         let ghost levels = self.shape.levels();
+        let ghost rewritten = ancestors_below_level(levels, self.rewritten_containers@, levels.len() as int);
+        let rightmost_count = self.rightmost_count_of_the_rewritten_containers(self.shape.containers());
+        // 每层重写的那一段从哪个节点起；一片都没重写时一个节点都不重写。
+        let suffix_starts = if rightmost_count == 0 {
+            None
+        } else {
+            Some(self.shape.ancestor_suffix_starts_of_the_rightmost_containers(rightmost_count))
+        };
+        proof {
+            if rightmost_count == 0 {
+                assert(self.rewritten_containers@ =~= Seq::<usize>::empty());
+            }
+        }
+        let positions = self.shape.positions_in_bump_order();
         let mut node_origins: Vec<(ContainerTreeNodePosition, ContainerTreeNodeOrigin)> = Vec::new();
         let mut listed: usize = 0;
         while listed < positions.len()
@@ -2458,12 +2932,24 @@ impl<Record> ContainerTreeAfterThisPublish<Record> {
                 container_tree_shape_is_well_formed(levels),
                 shape_fits_the_position_fields(levels),
                 positions@ == bump_order_positions(levels, levels.len() as int),
-                rewritten@ == ancestors_below_level(levels, self.rewritten_containers@, levels.len() as int),
+                rewritten == ancestors_below_level(levels, self.rewritten_containers@, levels.len() as int),
+                self.rewritten_containers@ == rightmost_containers(container_count(levels), rightmost_count as int),
+                rightmost_count == self.rewritten_containers@.len(),
                 every_node_absent_from_the_previous_version_is_rewritten(
-                    self.shape_before_this_publish.levels(),
+                    self.node_counts_before_this_publish.counts(),
                     levels,
                     self.rewritten_containers@,
                 ),
+                match suffix_starts {
+                    Some(starts) => rightmost_count >= 1 && starts@.len() == levels.len() && forall|level_index: int, node: int|
+                        #[trigger] node_is_in_the_levels(levels, level_index, node) ==> (node_is_an_ancestor_of_one_of(
+                            levels,
+                            level_index,
+                            node,
+                            self.rewritten_containers@,
+                        ) <==> node >= starts@[level_index]),
+                    None => rightmost_count == 0,
+                },
                 forall|any_listed: int|
                     0 <= any_listed < positions.len() ==> {
                         let position = #[trigger] positions@[any_listed];
@@ -2472,7 +2958,7 @@ impl<Record> ContainerTreeAfterThisPublish<Record> {
                     },
                 forall|level_index: int, node: int|
                     #[trigger] node_is_in_the_levels(levels, level_index, node) && level_index < levels.len() ==> (
-                    rewritten@.contains(position_at(level_index, node)) <==> node_is_an_ancestor_of_one_of(
+                    rewritten.contains(position_at(level_index, node)) <==> node_is_an_ancestor_of_one_of(
                         levels,
                         level_index,
                         node,
@@ -2484,31 +2970,44 @@ impl<Record> ContainerTreeAfterThisPublish<Record> {
                     0 <= done < listed ==> {
                         let origin = #[trigger] node_origins@[done];
                         &&& origin.0 == positions@[done]
-                        &&& (origin.1 is RewrittenThisPublish <==> rewritten@.contains(origin.0))
-                        &&& origin.1 is CarriedFromThePreviousVersion ==> position_is_in_the_shape(
-                            self.shape_before_this_publish.levels(),
+                        &&& (origin.1 is RewrittenThisPublish <==> rewritten.contains(origin.0))
+                        &&& origin.1 is CarriedFromThePreviousVersion ==> position_is_counted(
+                            self.node_counts_before_this_publish.counts(),
                             origin.0,
                         )
                     },
             decreases positions.len() - listed,
         {
             let position = positions[listed];
-            if position_is_listed(&rewritten, position) {
+            let level_index = usize::from(position.level) - 1;
+            let index_in_level = usize::try_from(position.index_in_level).expect("同层序号装得进 usize");
+            proof {
+                assert(1 <= position.level <= levels.len());
+                assert(node_is_in_the_levels(levels, level_index as int, index_in_level as int));
+                assert(position == position_at(level_index as int, index_in_level as int));
+            }
+            let is_rewritten = match &suffix_starts {
+                Some(starts) => index_in_level >= starts[level_index],
+                None => false,
+            };
+            proof {
+                if rightmost_count == 0 {
+                    assert(!node_is_an_ancestor_of_one_of(levels, level_index as int, index_in_level as int, self.rewritten_containers@));
+                }
+                assert(is_rewritten == rewritten.contains(position));
+            }
+            if is_rewritten {
                 node_origins.push((position, ContainerTreeNodeOrigin::RewrittenThisPublish));
             } else {
                 proof {
-                    let level_index = position.level - 1;
-                    let node = position.index_in_level as int;
-                    assert(node_is_in_the_levels(levels, level_index, node));
-                    assert(position == position_at(level_index, node));
-                    assert(!node_is_an_ancestor_of_one_of(levels, level_index, node, self.rewritten_containers@));
-                    assert(node_is_in_the_levels(self.shape_before_this_publish.levels(), level_index, node));
+                    assert(!node_is_an_ancestor_of_one_of(levels, level_index as int, index_in_level as int, self.rewritten_containers@));
+                    assert(node_is_counted(self.node_counts_before_this_publish.counts(), level_index as int, index_in_level as int));
                 }
                 let level = position.level;
-                let index_in_level = position.index_in_level;
+                let index_in_level_of_the_position = position.index_in_level;
                 assert!(
-                    self.shape_before_this_publish.has_a_node_at(position),
-                    "照抄的容器树节点（层级 {level}、同层第 {index_in_level} 个）在上一版里有：只有最右那一条路径上的节点是新开的"
+                    self.node_counts_before_this_publish.has_a_node_at(position),
+                    "照抄的容器树节点（层级 {level}、同层第 {index_in_level_of_the_position} 个）在上一版里有：只有最右那一条路径上的节点是新开的"
                 );
                 node_origins.push((position, ContainerTreeNodeOrigin::CarriedFromThePreviousVersion));
             }
@@ -2550,7 +3049,8 @@ pub open spec fn container_is_opened_by_this_publish<Record>(
 
 /// 追加到这一条为止，容器、形状与重写清单对上一版与已追加的记录逐条成立的那几样（[`append_records_to_the_open_container`] 的循环不变量与 `ensures`）：
 /// 「key 到条目」——全部记录依次连起来 = 上一版的接上已追加的；最右那片之前的容器原样；最右那片身份不变、原有记录是前缀；
-/// 新开的容器身份照参数、容器号逐片加一；形状在最右端长出来、容器数等于片数；重写清单升序、罩住新开的每一片、最后一项是最右那片；
+/// 新开的容器身份照参数、容器号逐片加一；形状在最右端长出来、容器数等于片数；重写清单升序、罩住新开的每一片、最后一项是最右那片，
+/// 而且就是最右端连着的那几片（[`rewritten_containers_are_the_rightmost`]：发布路径按它判第几片、哪个节点重写了，不在清单上逐个找）；
 /// 一条都没追加时什么都没动；不在上一版的节点都罩着重写的某一片（照抄的节点上一版都有）。
 pub open spec fn appending_progress<Record>(
     containers_before_this_publish: Seq<RecordContainer<Record>>,
@@ -2595,11 +3095,12 @@ pub open spec fn appending_progress<Record>(
     &&& appended_records.len() == 0 ==> rewritten_containers.len() == 0 && containers_are_the_same(before, containers)
         && levels == shape_before_this_publish
     &&& appended_records.len() > 0 ==> rewritten_containers.len() >= 1 && rewritten_containers.last() == containers.len() - 1
-    &&& every_node_absent_from_the_previous_version_is_rewritten(shape_before_this_publish, levels, rewritten_containers)
+    &&& rewritten_containers_are_the_rightmost(rewritten_containers, containers.len() as int)
+    &&& every_node_absent_from_the_previous_version_is_rewritten(node_counts_of(shape_before_this_publish), levels, rewritten_containers)
 }
 
 /// [`append_records_to_the_open_container`] 的 `ensures`：交回的样子对上一版、这次给的记录与参数照 [`appending_progress`] 成立，
-/// 这次之前的形状照抄上一版。
+/// 这次之前那一版记的每层节点数就是上一版形状的。
 pub open spec fn appending_keeps_the_key_to_entry_mapping<Record>(
     containers_before_this_publish: Seq<RecordContainer<Record>>,
     shape_before_this_publish: Seq<Seq<usize>>,
@@ -2610,7 +3111,7 @@ pub open spec fn appending_keeps_the_key_to_entry_mapping<Record>(
     records_per_container: int,
     after: ContainerTreeAfterThisPublish<Record>,
 ) -> bool {
-    &&& after.shape_before_this_publish.levels() == shape_before_this_publish
+    &&& after.node_counts_before_this_publish.counts() == node_counts_of(shape_before_this_publish)
     &&& appending_progress(
         containers_before_this_publish,
         shape_before_this_publish,
@@ -2667,6 +3168,12 @@ proof fn lemma_appending_progress_at_the_start<Record>(
     ) by {
         assert(shape_before_this_publish[level_index].len() >= 1);
     }
+    assert forall|level_index: int, node: int|
+        #[trigger] node_is_in_the_levels(shape_before_this_publish, level_index, node) implies node_is_counted(
+        node_counts_of(shape_before_this_publish),
+        level_index,
+        node,
+    ) by {}
 }
 
 /// 往还装得下的最右那片里追加一条之后，[`appending_progress`] 照样成立。
@@ -2776,7 +3283,7 @@ proof fn lemma_appending_into_the_open_container_keeps_the_progress<Record>(
         assert(rewritten_after.last() == last as usize);
         assert(rewritten_after.len() == 1);
         lemma_more_rewritten_containers_keep_every_new_node_rewritten(
-            shape_before_this_publish,
+            node_counts_of(shape_before_this_publish),
             levels,
             rewritten_containers,
             last as usize,
@@ -2800,7 +3307,16 @@ proof fn lemma_appending_into_the_open_container_keeps_the_progress<Record>(
         0 <= earlier < later < rewritten_after.len() ==> #[trigger] rewritten_after[earlier] < #[trigger] rewritten_after[later]);
     assert(forall|listed: int| 0 <= listed < rewritten_after.len() ==> #[trigger] rewritten_after[listed] < containers_after.len());
     assert(appended_after.len() > 0 ==> rewritten_after.len() >= 1 && rewritten_after.last() == containers_after.len() - 1);
-    assert(every_node_absent_from_the_previous_version_is_rewritten(shape_before_this_publish, levels, rewritten_after));
+    assert(every_node_absent_from_the_previous_version_is_rewritten(node_counts_of(shape_before_this_publish), levels, rewritten_after));
+    assert forall|listed: int| 0 <= listed < rewritten_after.len() implies #[trigger] rewritten_after[listed] == containers_after.len()
+        - rewritten_after.len() + listed by {
+        if appended_records.len() == 0 {
+            assert(rewritten_after[0] == last as usize);
+        } else {
+            assert(rewritten_after[listed] == rewritten_containers[listed]);
+        }
+    }
+    assert(rewritten_containers_are_the_rightmost(rewritten_after, containers_after.len() as int));
 }
 
 /// 最右那片装满（或树是空的）、开一片新容器追加一条之后，[`appending_progress`] 照样成立。
@@ -2908,7 +3424,12 @@ proof fn lemma_opening_a_container_keeps_the_progress<Record>(
     lemma_levels_extend_transitively(shape_before_this_publish, levels, levels_after);
     assert forall|listed: int| 0 <= listed < rewritten_containers.len() implies #[trigger] rewritten_containers[listed]
         < container_count(levels) by {}
-    lemma_opening_keeps_every_new_node_rewritten(shape_before_this_publish, levels, entries_per_node, rewritten_containers);
+    lemma_opening_keeps_every_new_node_rewritten(
+        node_counts_of(shape_before_this_publish),
+        levels,
+        entries_per_node,
+        rewritten_containers,
+    );
     assert(rewritten_after == rewritten_containers.push(containers.len() as usize));
     assert forall|position: int| before_count <= position < containers_after.len() implies #[trigger] rewritten_after.contains(
         position as usize,
@@ -2931,6 +3452,15 @@ proof fn lemma_opening_a_container_keeps_the_progress<Record>(
             assert(rewritten_after[earlier] == rewritten_containers[earlier]);
         }
     }
+    assert forall|listed: int| 0 <= listed < rewritten_after.len() implies #[trigger] rewritten_after[listed] == containers_after.len()
+        - rewritten_after.len() + listed by {
+        if listed < rewritten_containers.len() {
+            assert(rewritten_after[listed] == rewritten_containers[listed]);
+        } else {
+            assert(rewritten_after[listed] == containers.len());
+        }
+    }
+    assert(rewritten_containers_are_the_rightmost(rewritten_after, containers_after.len() as int));
     if appended_records.len() == 0 {
         assert(levels == shape_before_this_publish);
     }
@@ -3185,7 +3715,7 @@ pub fn append_records_to_the_open_container<Record: Copy>(
             containers,
             rewritten_containers,
             shape,
-            shape_before_this_publish: shape_before_this_publish.clone(),
+            node_counts_before_this_publish: shape_before_this_publish.node_counts(),
         },
     )
 }
@@ -3225,6 +3755,17 @@ mod tests {
         }
     }
 
+    /// 一个节点的孩子在下一层里的下标区间：取 [`ContainerTreeShape::children_of_every_node`] 里那一项。
+    fn children_of_the_node(
+        shape: &ContainerTreeShape,
+        node: ContainerTreeNodePosition,
+    ) -> Range<usize> {
+        let level_index = usize::from(node.level) - 1;
+        let index_in_level =
+            usize::try_from(node.index_in_level).expect("测试里的同层序号装得进 usize");
+        shape.children_of_every_node()[level_index][index_in_level].clone()
+    }
+
     /// 扇出压到 2：第 1、2 片挂在层级 1 的根下；第 3 片根满、长一层（旧根留在 (1, 0)、右半 (1, 1) 一个孩子、新根 (2, 0) 两个孩子）；
     /// 第 5 片两层都满、再长一层：(1, 2) 满了开 (1, 3)、(2, 1) 也满了开 (2, 2)，根 (2, 0) 那一层原先只有两个节点、新根 (3, 0)。
     #[test]
@@ -3240,9 +3781,9 @@ mod tests {
             vec![position(1, 0), position(1, 1), position(2, 0)],
             "第 3 片：根满长一层，旧根在 (1, 0)、右半在 (1, 1)"
         );
-        assert_eq!(three.children_of(position(1, 0)), 0..2);
-        assert_eq!(three.children_of(position(1, 1)), 2..3);
-        assert_eq!(three.children_of(position(2, 0)), 0..2);
+        assert_eq!(children_of_the_node(&three, position(1, 0)), 0..2);
+        assert_eq!(children_of_the_node(&three, position(1, 1)), 2..3);
+        assert_eq!(children_of_the_node(&three, position(2, 0)), 0..2);
         assert_eq!(three.internal_levels(), 1);
         let five = shape_after_opening(5, 2);
         assert_eq!(
@@ -3257,8 +3798,8 @@ mod tests {
             ],
             "第 5 片：层级 1、2 都满，长到层级 3"
         );
-        assert_eq!(five.children_of(position(3, 0)), 0..2);
-        assert_eq!(five.children_of(position(2, 1)), 2..3);
+        assert_eq!(children_of_the_node(&five, position(3, 0)), 0..2);
+        assert_eq!(children_of_the_node(&five, position(2, 1)), 2..3);
         assert_eq!(five.containers(), 5);
     }
 
@@ -3391,8 +3932,9 @@ mod tests {
             after.rewritten_nodes(),
             vec![position(1, 0), position(1, 1), position(2, 0)]
         );
+        assert_eq!(after.first_rewritten_container(), Some(1));
         assert_eq!(
-            after.shape.ancestors_of_containers(&[2]),
+            after.shape.ancestors_of_the_rightmost_containers(1),
             vec![position(1, 1), position(2, 0)],
             "只看最右那一片时祖先就是最右那一条路径：右半 (1, 1) 与根 (2, 0)，旧根 (1, 0) 不在其内"
         );
@@ -3408,12 +3950,293 @@ mod tests {
         )
         .expect("一条都不追加，不开新容器");
         assert!(carried.rewritten_containers.is_empty() && carried.rewritten_nodes().is_empty());
+        assert_eq!(carried.first_rewritten_container(), None);
         assert_eq!(
             carried.containers, after.containers,
             "一条都不追加时整棵照抄"
         );
     }
 
+    /// 求祖先只读最右端那几片的路径，不按容器总数开表、不逐层扫全层（`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」
+    /// 运行时决策路径那一格：代价不随容器总数涨）。这份形状只有最右端那一截是真的：记着 2^20 片容器，层级 1 最右两个节点罩住最后 3 + 2 片、
+    /// 层级 2 最右那个节点罩住层级 1 的最后两个、根罩住层级 2 的两个；层级 1、2 最左那个节点的孩子数写成 `usize::MAX`，故意不满足形状的不变量——
+    /// 从最左读起的写法（按全部容器数开一张标记表，再逐层从最左扫全层；或从最右往左一路减到最左）一读到它就越过标记表或加减溢出、panic。
+    /// 问最右那两片：每层从最右端往左读到那一段起点就停的写法交回最右那一条路径，每层的那一段起点也只到那里。
+    #[test]
+    fn ancestors_of_the_rightmost_containers_read_only_the_nodes_on_their_paths() {
+        const CONTAINER_TOTAL: usize = 1 << 20;
+        let only_the_right_end_is_readable = ContainerTreeShape {
+            children_of_each_node_at_each_level: vec![
+                vec![usize::MAX, 3, 2],
+                vec![usize::MAX, 2],
+                vec![2],
+            ],
+            container_total: CONTAINER_TOTAL,
+        };
+        assert_eq!(
+            only_the_right_end_is_readable.ancestors_of_the_rightmost_containers(2),
+            vec![position(1, 2), position(2, 1), position(3, 0)],
+            "最右那两片的祖先：层级 1 的 (1, 2)、层级 2 的 (2, 1) 与根 (3, 0)；读不到最左那两个节点"
+        );
+        assert_eq!(
+            only_the_right_end_is_readable.ancestor_suffix_starts_of_the_rightmost_containers(2),
+            vec![2, 1, 0],
+            "每层那一段从最右那条路径上的节点起"
+        );
+    }
+
+    /// 对拍的参照：按全部容器数（层级 1 的孩子数逐个加起来）开一张标记表、标出给的容器，再逐层从最左扫全层，
+    /// 看每个节点罩住的那一段里有没有标过的。
+    fn ancestors_by_marking_every_container(
+        shape: &ContainerTreeShape,
+        containers: &[usize],
+    ) -> Vec<ContainerTreeNodePosition> {
+        let container_total: usize = shape
+            .children_of_each_node_at_each_level
+            .first()
+            .map_or(0, |level_one| level_one.iter().sum());
+        let mut marked_below = vec![false; container_total];
+        for &container in containers {
+            marked_below[container] = true;
+        }
+        let mut ancestors = Vec::new();
+        for (level_index, nodes) in shape.children_of_each_node_at_each_level.iter().enumerate() {
+            let level = u8::try_from(level_index + 1).expect("测试里的形状不到 255 层");
+            let mut marked_here = Vec::with_capacity(nodes.len());
+            let mut first_child = 0;
+            for (node, &children) in nodes.iter().enumerate() {
+                let is_marked = marked_below[first_child..first_child + children]
+                    .iter()
+                    .any(|&marked| marked);
+                if is_marked {
+                    ancestors.push(position(
+                        level,
+                        u32::try_from(node).expect("测试里的形状同层不到 2^32 个节点"),
+                    ));
+                }
+                marked_here.push(is_marked);
+                first_child += children;
+            }
+            marked_below = marked_here;
+        }
+        ancestors
+    }
+
+    /// 对拍用的一批形状：扇出 2、3 与产品扇出各开 1 到 40 片（151 份里的 120 份）；再加先按扇出 2 开 9 片、换回产品扇出接着开到 40 片的 31 份
+    /// （只供测试的扇出旋钮不落盘，换回产品扇出照盘上那份形状往最右那条路径上加，左边留着扇出 2 的满节点）。
+    fn shapes_for_the_ancestor_differential() -> Vec<ContainerTreeShape> {
+        let mut shapes = Vec::new();
+        for entries in [2, 3, 135] {
+            for containers in 1..=40 {
+                shapes.push(shape_after_opening(containers, entries));
+            }
+        }
+        let mut switched_to_the_product_fanout = shape_after_opening(9, 2);
+        for _ in 10..=40 {
+            switched_to_the_product_fanout = switched_to_the_product_fanout
+                .after_a_container_is_opened_at_the_right_end(
+                    ContainerTreeNodeEntries::FromTheNodeFormat.entries_per_node(),
+                );
+            shapes.push(switched_to_the_product_fanout.clone());
+        }
+        shapes
+    }
+
+    /// 改前改后对拍：同一批形状上，最右端 1 到 n 片（n 是容器数）各问一次，每层只读那一段后缀的求祖先与按全部容器数开标记表、逐层扫全层的
+    /// 求祖先交回的位置逐项相同（集合与次序）。151 份形状各问 n 组：扇出 2、3、产品扇出各 Σ(n = 1..=40) = 820 组，换扇出那 31 份
+    /// Σ(n = 10..=40) = 775 组，合计 3235 组。
+    #[test]
+    fn ancestors_read_from_the_right_end_equal_the_ancestors_found_by_marking_every_container() {
+        let mut compared: usize = 0;
+        for shape in shapes_for_the_ancestor_differential() {
+            let container_total = shape.containers();
+            for rightmost_count in 1..=container_total {
+                let containers: Vec<usize> =
+                    (container_total - rightmost_count..container_total).collect();
+                assert_eq!(
+                    shape.ancestors_of_the_rightmost_containers(rightmost_count),
+                    ancestors_by_marking_every_container(&shape, &containers),
+                    "形状 {shape:?} 上问最右端 {rightmost_count} 片"
+                );
+                compared += 1;
+            }
+        }
+        assert_eq!(compared, 3_235, "对拍了 151 份形状上最右端的每一组");
+    }
+
+    /// 同一批形状上：最右端 1 到 n 片重写时，`node_origins` 按每层那一段起点判出的「这次重写」恰好是按全部容器数开标记表找出的祖先，
+    /// 别的都是照抄（上一版取同一份形状，每个位置上一版都有）。一片都不重写时一个节点都不重写。合计 3235 + 151 组。
+    #[test]
+    fn node_origins_mark_exactly_the_ancestors_of_the_rightmost_rewritten_containers() {
+        let mut compared: usize = 0;
+        for shape in shapes_for_the_ancestor_differential() {
+            let container_total = shape.containers();
+            for rightmost_count in 0..=container_total {
+                let rewritten_containers: Vec<usize> =
+                    (container_total - rightmost_count..container_total).collect();
+                let expected_rewritten =
+                    ancestors_by_marking_every_container(&shape, &rewritten_containers);
+                let after: ContainerTreeAfterThisPublish<u64> = ContainerTreeAfterThisPublish {
+                    containers: (0..container_total)
+                        .map(|_| RecordContainer {
+                            identity: IDENTITY_OF_A_CODE_TWO_CHILD,
+                            records: Vec::new(),
+                        })
+                        .collect(),
+                    rewritten_containers,
+                    node_counts_before_this_publish: shape.node_counts(),
+                    shape: shape.clone(),
+                };
+                let origins = after.node_origins();
+                assert_eq!(
+                    origins.iter().map(|(node, _)| *node).collect::<Vec<_>>(),
+                    shape.positions_in_bump_order(),
+                    "形状 {shape:?}：按 bump 次序列全部节点"
+                );
+                for (node, origin) in origins {
+                    let expected_origin = if expected_rewritten.contains(&node) {
+                        ContainerTreeNodeOrigin::RewrittenThisPublish
+                    } else {
+                        ContainerTreeNodeOrigin::CarriedFromThePreviousVersion
+                    };
+                    assert_eq!(
+                        origin, expected_origin,
+                        "形状 {shape:?} 上最右端 {rightmost_count} 片重写时节点 {node:?}"
+                    );
+                }
+                compared += 1;
+            }
+        }
+        assert_eq!(
+            compared, 3_386,
+            "对拍了 151 份形状上最右端的每一组与一片都不重写的那一组"
+        );
+    }
+
+    /// 同一批形状上，每一层顺着累加出的孩子区间与从这一层最左把前面几个节点的孩子数加起来的区间逐项相同。
+    #[test]
+    fn children_of_every_node_equal_the_prefix_sums_of_each_level() {
+        for shape in shapes_for_the_ancestor_differential() {
+            let children = shape.children_of_every_node();
+            assert_eq!(
+                children.len(),
+                shape.children_of_each_node_at_each_level.len()
+            );
+            for (level_index, nodes) in shape.children_of_each_node_at_each_level.iter().enumerate()
+            {
+                let expected: Vec<Range<usize>> = (0..nodes.len())
+                    .map(|node| {
+                        let first_child: usize = nodes[..node].iter().sum();
+                        first_child..first_child + nodes[node]
+                    })
+                    .collect();
+                assert_eq!(
+                    children[level_index], expected,
+                    "形状 {shape:?} 层下标 {level_index}"
+                );
+            }
+        }
+    }
+
+    /// 最右端开一片原地改形状、不逐层照抄：三层的形状，层级 1 有 13500 个满节点（容器 1822500 片）、层级 2 有 100 个满节点、根 100 个孩子，
+    /// 开第 1822501 片时层级 1、2 末尾各加一个节点、根加一个孩子。每层的 `Vec` 都预留了多一个的余量，原地改不重新分配：开片之后每层还是原来那一块
+    /// 内存。逐层照抄的写法（改前那种：每层 `copy_of_one_level` 抄一份新的）每层都换了一块，这里就红。
+    #[test]
+    fn opening_a_container_changes_the_levels_in_place_without_copying_them() {
+        const LEVEL_ONE_NODES: usize = 13_500;
+        const LEVEL_TWO_NODES: usize = 100;
+        let mut level_one = Vec::with_capacity(LEVEL_ONE_NODES + 1);
+        level_one.resize(LEVEL_ONE_NODES, 135);
+        let mut level_two = Vec::with_capacity(LEVEL_TWO_NODES + 1);
+        level_two.resize(LEVEL_TWO_NODES, 135);
+        let root = vec![LEVEL_TWO_NODES];
+        let mut levels = Vec::with_capacity(4);
+        levels.extend([level_one, level_two, root]);
+        let shape = ContainerTreeShape::from_children_of_each_node_at_each_level(levels)
+            .expect("每层孩子数之和等于下一层的节点数、根一个节点");
+        let buffers_before: Vec<*const usize> = shape
+            .children_of_each_node_at_each_level
+            .iter()
+            .map(|nodes| nodes.as_ptr())
+            .collect();
+        let opened = shape.after_a_container_is_opened_at_the_right_end(
+            ContainerTreeNodeEntries::FromTheNodeFormat.entries_per_node(),
+        );
+        assert_eq!(opened.containers(), LEVEL_ONE_NODES * 135 + 1);
+        assert_eq!(opened.nodes_at_level(1), LEVEL_ONE_NODES + 1);
+        assert_eq!(opened.nodes_at_level(2), LEVEL_TWO_NODES + 1);
+        assert_eq!(
+            opened.children_of_each_node_at_each_level[2],
+            vec![LEVEL_TWO_NODES + 1]
+        );
+        let buffers_after: Vec<*const usize> = opened
+            .children_of_each_node_at_each_level
+            .iter()
+            .map(|nodes| nodes.as_ptr())
+            .collect();
+        assert_eq!(
+            buffers_after, buffers_before,
+            "开片之后每一层还是原来那一块内存：原地改，不逐层照抄"
+        );
+    }
+
+    /// 这次之前那一版只记每层节点数：一次追加长了一层，记着的是上一版的两层（层级 1 两个节点、根一个），据它判哪个位置上一版有节点、哪个是上一版的根。
+    #[test]
+    fn the_version_before_this_publish_is_kept_as_the_node_count_of_each_level() {
+        let before_shape = shape_after_opening(3, 2);
+        let containers: Vec<RecordContainer<u64>> = (1..=3)
+            .map(|number| RecordContainer {
+                identity: PackedIdentity {
+                    birth_tree: TreeIdentifier(65545),
+                    record_type: 1,
+                    container: number,
+                    container_birth: CheckpointTxg(5),
+                },
+                records: vec![number],
+            })
+            .collect();
+        let after = append_records_to_the_open_container(
+            &containers,
+            &before_shape,
+            &[4_u64, 5],
+            CheckpointTxg(9),
+            TreeIdentifier(65545),
+            1,
+            1,
+            capped(2),
+        )
+        .expect("最右那片的号 3，加一装得下");
+        assert_eq!(
+            after.node_counts_before_this_publish,
+            ContainerTreeNodeCounts {
+                nodes_at_each_level: vec![2, 1]
+            }
+        );
+        assert_eq!(
+            after.node_counts_before_this_publish,
+            before_shape.node_counts()
+        );
+        assert!(after
+            .node_counts_before_this_publish
+            .has_a_node_at(position(1, 1)));
+        assert!(!after
+            .node_counts_before_this_publish
+            .has_a_node_at(position(1, 2)));
+        assert!(!after
+            .node_counts_before_this_publish
+            .has_a_node_at(position(3, 0)));
+        assert!(after
+            .node_counts_before_this_publish
+            .is_the_root(position(2, 0)));
+        assert_eq!(
+            after.node_counts_before_this_publish.root_position(),
+            Some(position(2, 0))
+        );
+        assert_eq!(after.rewritten_containers, vec![3, 4]);
+        assert_eq!(after.first_rewritten_container(), Some(3));
+        assert!(after.grew_a_level_this_publish());
+    }
+
     /// C841：最右那片的号是 `u64::MAX`（坏镜像上读来的）。它还装得下时照常追加进它；装满了、要开新容器时交回
     /// `NextContainerNumberPastTheTopOfItsRange`、点名树与最右那片的号，不 panic。改之前这一步加一溢出 panic（工作区开着 overflow-checks）。
     #[test]
```

### `crates/singlefs-core/src/mount.rs`

```diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 1c48af3..ba386e5 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -98,13 +98,13 @@ use crate::transaction::{
     walk_the_deletion_queue_until_it_is_empty,
     write_the_raised_floor_into_every_system_configuration,
     write_the_witnessed_expected_acquisition, AcquisitionFailed, CodeTwoTreeNodeCapacities,
-    CopyQuarantinedAfterReleaseChecksumMismatch, DeletionAndLineageOfThePublish, DeletionQueueWalk,
-    ExpectedInstanceAcquisitionFailed, InstanceGenerationPastTheTopOfItsRange,
-    InstanceTableOnlyPublishPlan, InstanceTablePlan, InstanceTableRewrite, MappingLookup,
-    PoolVersion, PoolWriter, PublishError, PublishPlan, PublishedUnit,
-    RaisedFloorSystemConfigurationWriteFailed, ReadFailedOnTheDisk, ReadFailingSelfVerification,
-    ReleaseChecksumCheck, TransactionOutput, TransactionUnit, VersionWithoutFilePublishOutput,
-    ZeroUnitPublishPlan,
+    ContainerTreeRootUnitPositions, CopyQuarantinedAfterReleaseChecksumMismatch,
+    DeletionAndLineageOfThePublish, DeletionQueueWalk, ExpectedInstanceAcquisitionFailed,
+    InstanceGenerationPastTheTopOfItsRange, InstanceTableOnlyPublishPlan, InstanceTablePlan,
+    InstanceTableRewrite, MappingLookup, PoolVersion, PoolWriter, PublishError, PublishPlan,
+    PublishedUnit, RaisedFloorSystemConfigurationWriteFailed, ReadFailedOnTheDisk,
+    ReadFailingSelfVerification, ReleaseChecksumCheck, TransactionOutput, TransactionUnit,
+    VersionWithoutFilePublishOutput, ZeroUnitPublishPlan,
 };
 use crate::unlink::DeletionSwitches;
 use crate::write_accounting::{WriteCallsAndBytes, WritesByStructureKind, WrittenStructureKind};
@@ -9909,14 +9909,17 @@ fn version_to_publish_the_rollback_after(
             head_identifier: entry.head_identifier,
         })
         .collect();
+    let units: Vec<PublishedUnit> = user_visible_units_of_the_target
+        .chain(other_units_of_the_current_version)
+        .collect();
+    // 装好这一版的 units 之后记下两棵容器树根单元的下标（下一次准入读 H 按它直接取）。
+    let container_tree_root_unit_positions = ContainerTreeRootUnitPositions::of_the_units(&units);
     TransactionOutput {
         root: current.root,
         record: current.record.clone(),
         record_bytes: current.record_bytes.clone(),
         earlier_records_of_this_publish: Vec::new(),
-        units: user_visible_units_of_the_target
-            .chain(other_units_of_the_current_version)
-            .collect(),
+        units,
         rewritten: Vec::new(),
         data_pointers: target_version.data_pointers.clone(),
         // 第一个文件之外各文件的数据单元与下段同第一个文件的，取回退目标那一版的（用户可见单元）。
@@ -9964,6 +9967,9 @@ fn version_to_publish_the_rollback_after(
         writes: WritesByStructureKind::NOTHING_WRITTEN,
         highest_transaction_number_in_this_instance: current
             .highest_transaction_number_in_this_instance,
+        container_tree_root_unit_positions,
+        // 回退那一版的 inode 树取 R_old 那一版的，两个数也照抄它的。
+        inode_record_counts: target_version.inode_record_counts,
     }
 }
 
```

### `crates/singlefs-core/src/recovery.rs`

```diff
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 764678c..ef2ed4d 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -32,6 +32,7 @@ use crate::address::{
     CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InodeNumber,
     InstanceGeneration, SlotNumber, TreeIdentifier,
 };
+use crate::admission::InodeRecordCounts;
 use crate::allocation_record_tree::{
     read_allocation_record_tree, AllocationRecordTreeGeometry, AllocationRecordTreeHeaderJudgement,
     AllocationRecordTreeReadFromDisk,
@@ -107,9 +108,10 @@ use crate::tombstone_tree::CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES;
 use crate::transaction::{
     role_of_allocation_record_tree_node, role_of_extent_upper_node,
     take_the_mapping_positions_over_the_location_hints, ContainerTreeIndexNodesVersion,
-    FileOfAnInodeOtherThanTheFirst, FileVersionTreeIdentifiers, InodeLeafContainerVersion,
-    LineageContainerVersion, MultiLevelCodeTwoTree, PublishedUnit, RetirementContainerVersion,
-    TombstoneContainerVersion, TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
+    ContainerTreeRootUnitPositions, FileOfAnInodeOtherThanTheFirst, FileVersionTreeIdentifiers,
+    InodeLeafContainerVersion, LineageContainerVersion, MultiLevelCodeTwoTree, PublishedUnit,
+    RetirementContainerVersion, TombstoneContainerVersion, TransactionOutput, TransactionUnit,
+    FIRST_INODE_NUMBER,
 };
 use crate::unit::{
     data_unit_payload, data_unit_payload_capacity, parse_data_unit, parse_index_node,
@@ -3432,6 +3434,14 @@ fn rebuild_version_with_its_snapshot_entries_judging_the_truncation_fields_as(
     let central_mapping_tree = trees_read.mapping_tree.version.clone();
     let deadlist_tree_version = trees_read.deadlist_tree.version.clone();
     let units = units_of_the_version(trees_read, root, bootstrap_units);
+    // 装好这一版的 units 之后记下两棵容器树根单元的下标（挂载之后头一次准入读 H 按它直接取）。
+    let container_tree_root_unit_positions = ContainerTreeRootUnitPositions::of_the_units(&units);
+    // n_live 与 n_记录的起始值：冷恢复读出这一版时逐片逐条数一遍（挂载一次），之后每次发布在提交时增量维护。
+    let inode_record_counts = InodeRecordCounts::counted_over_the_inode_tree(
+        inode_leaf_containers
+            .iter()
+            .map(|container| &container.contents),
+    );
     let transaction_number_on_the_record_standing_for_this_root = record.transaction;
     let mut rebuilt = TransactionOutput {
         root: *root,
@@ -3481,6 +3491,8 @@ fn rebuild_version_with_its_snapshot_entries_judging_the_truncation_fields_as(
         // 「上一条记录的事务号 + 1」，33 条里只有 I-8.7 judged 出来。
         highest_transaction_number_in_this_instance:
             transaction_number_on_the_record_standing_for_this_root,
+        container_tree_root_unit_positions,
+        inode_record_counts,
     };
     // I-2.5 先判盘上读来的提示、再盖映射位置：盖过之后提示就没了，盘上逆序的提示被映射位置盖掉，池级 checker 照盘上字节判坏的镜像
     // 可写挂载却会收下。
```

### `crates/singlefs-core/src/transaction.rs`

```diff
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 6d76238..344d66b 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -35,11 +35,11 @@ use crate::admission::{
     admit_the_units_landing_on_every_device,
     checkpoint_cost_of_the_version_to_build_on_with_node_capacities,
     committed_reservation_for_tombstones_and_retirement_records,
-    demand_of_the_roles_on_each_device, inode_records_of_the_inode_tree,
-    live_objects_of_the_inode_tree, publish_has_an_ordinary_allocation, units_of_a_publish_to_land,
-    worst_case_occupancy_of_a_compaction_batch, AdmissionRefusedOnSomeDevices,
-    BytesSummedOverAllReplicas, EmptyClusterSegmentsBelowTheFloorOnSomeDevices,
-    EmptyClusterSegmentsConjunctReach, PlacementRoomOnOneDevice, ReplicaCount, SpaceAdmission,
+    demand_of_the_roles_on_each_device, publish_has_an_ordinary_allocation,
+    units_of_a_publish_to_land, worst_case_occupancy_of_a_compaction_batch,
+    AdmissionRefusedOnSomeDevices, BytesSummedOverAllReplicas,
+    EmptyClusterSegmentsBelowTheFloorOnSomeDevices, EmptyClusterSegmentsConjunctReach,
+    InodeRecordCounts, PlacementRoomOnOneDevice, ReplicaCount, SpaceAdmission,
     TombstoneAndRetirementPrepayBasis,
 };
 use crate::allocation_record_tree::{
@@ -3299,6 +3299,33 @@ pub struct TransactionOutput {
     /// 这个实例到这次发布为止用过的最大事务号（空发布写 0、不推进它）。下一次发布取它加一，
     /// 而不是取上一条记录上的事务号加一（D23（journal 的角色与格式） 已定项 7：事务号按实例计数、从 1 起）。
     pub highest_transaction_number_in_this_instance: u64,
+    /// 墓碑树、退役树根单元在 `units` 里的下标（[`ContainerTreeRootUnitPositions`]）：准入读两个 H 时按它直接取根单元，不在 `units` 上逐项找。
+    pub container_tree_root_unit_positions: ContainerTreeRootUnitPositions,
+    /// 这一版 inode 树里的 n_live 与 n_记录（[`InodeRecordCounts`]）：每次发布在提交时增量维护，准入读它，不遍历 inode 树。
+    pub inode_record_counts: InodeRecordCounts,
+}
+
+/// 一版里墓碑树、退役树根单元在 `units` 里的下标；那棵树零根（没有根单元）时 `None`。准入读两个 H（D3（空间分配） 已定项 2「各从自己的
+/// 树根节点码 2 头现读」）按它直接取根单元的字节：它在运行时决策路径上（ENOSPC 准入，`.claude/rules/fs-design.md`「记账是事务的副产品，
+/// 不是事后的遍历」），在 `units` 上逐项找是这一版的单元数（每片容器、每个码 2 节点各一项），随容器总数涨。装好一版的 `units` 时找一次记下
+/// （发布、从盘上重建、管理员回退各一处），之后 `units` 不再挪动。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct ContainerTreeRootUnitPositions {
+    pub tombstone_root: Option<usize>,
+    pub retirement_root: Option<usize>,
+}
+
+impl ContainerTreeRootUnitPositions {
+    /// 在装好的 `units` 里各找一次两个根单元（取最先那一项，与 [`TransactionOutput::unit`] 同一项）。
+    #[must_use]
+    pub fn of_the_units(units: &[PublishedUnit]) -> Self {
+        let first_position_of =
+            |identity: TransactionUnit| units.iter().position(|unit| unit.identity == identity);
+        Self {
+            tombstone_root: first_position_of(TransactionUnit::TombstoneRoot),
+            retirement_root: first_position_of(TransactionUnit::RetirementRoot),
+        }
+    }
 }
 
 /// [`TransactionOutput::unit`] 与 [`FirstEntryOfEachIdentity::unit`] 查不到时的 panic 消息：两条查法查的是同一张 `units`。
@@ -3981,15 +4008,36 @@ impl ContainerTreeIndexNodesVersion {
     }
 
     /// 根之下 `position` 上那个节点这一版的指针；那个位置上没有根之下的节点交回 `None`。
+    /// `pointers_below_the_root` 按 bump 次序列根之下的每个节点各一项（根最末、不在其中），所以这个位置那一项就在「这一层之前各层的节点数之和
+    /// + 同层序号」那一格，按下标直接取：装一版的单元时每个照抄的节点都问一次（`build_deletion_tree_units`），在整张上逐项找合起来是节点数的平方。
+    ///
+    /// # Panics
+    /// 那一格上记的位置不是问的这一个：这张与形状出自同一次发布（`publish_admitted` 按 `node_origins` 的 bump 次序装）或同一次从盘上重建
+    /// （`recovery` 按 bump 次序读），都按 bump 次序列根之下的全部节点。
     #[must_use]
     pub fn pointer_below_the_root(
         &self,
         position: ContainerTreeNodePosition,
     ) -> Option<NodePointer> {
-        self.pointers_below_the_root
-            .iter()
-            .find(|(node_position, _)| *node_position == position)
-            .map(|(_, pointer)| *pointer)
+        if !self.shape.has_a_node_at(position) || self.shape.is_the_root(position) {
+            return None;
+        }
+        let nodes_on_the_levels_below: usize = (1..position.level)
+            .map(|level| self.shape.nodes_at_level(level))
+            .sum();
+        let index_in_bump_order = nodes_on_the_levels_below
+            + usize::try_from(position.index_in_level).expect("同层序号装得进 usize");
+        let (listed_position, pointer) = *self
+            .pointers_below_the_root
+            .get(index_in_bump_order)
+            .expect(
+                "根之下的每个节点在指针表里各一项、按 bump 次序：形状里有、又不是根的位置都在表里",
+            );
+        assert_eq!(
+            listed_position, position,
+            "根之下节点的指针表按 bump 次序排：第 {index_in_bump_order} 项就是这个位置"
+        );
+        Some(pointer)
     }
 }
 
@@ -4133,6 +4181,25 @@ impl TransactionOutput {
             .copied()
     }
 
+    /// 这一版 inode 树里号是 `inode` 的那条记录（删除中的也在），按叶序折半找：叶容器左起按 key 序、各片的记录按号升序、两片之间不交叠，
+    /// 先找最后一条号不小于它的那一片，再在那一片里折半找。与 [`TransactionOutput::inode_record_of`] 交回同一条；发布路径改 n_live 与 n_记录时
+    /// 每写一条找一次（运行时决策路径，不遍历 inode 树）。叶容器都非空（记录数为 0 的容器不落盘，删空的那次摘掉），按最后一条找不会落空。
+    #[must_use]
+    pub fn inode_record_found_by_its_leaf(&self, inode: InodeNumber) -> Option<InodeRecord> {
+        let holding = self.inode_leaf_containers.partition_point(|container| {
+            container
+                .contents
+                .records
+                .last()
+                .is_some_and(|last| last.inode < inode.0)
+        });
+        let records = &self.inode_leaf_containers.get(holding)?.contents.records;
+        records
+            .binary_search_by_key(&inode.0, |record| record.inode)
+            .ok()
+            .map(|position| records[position])
+    }
+
     /// 这一版 inode 树里的叶容器内容，交给 `crate::inode_tree` 算下一次发布的落法。
     #[must_use]
     pub fn inode_leaf_container_contents(&self) -> Vec<InodeLeafContainer> {
@@ -4198,6 +4265,83 @@ impl TransactionOutput {
         steps
     }
 
+    /// 这一版墓碑树根单元：按装这一版时记下的下标取（[`ContainerTreeRootUnitPositions`]），不在 `units` 上找；墓碑树零根时 `None`。
+    ///
+    /// # Panics
+    /// 记下的下标上不是墓碑树根单元：`units` 装好之后才记下标，之后不再挪动（三处装 `units` 的地方都是装好再找）。
+    #[must_use]
+    pub fn tombstone_root_unit(&self) -> Option<&PublishedUnit> {
+        self.unit_at_the_recorded_position(
+            self.container_tree_root_unit_positions.tombstone_root,
+            TransactionUnit::TombstoneRoot,
+        )
+    }
+
+    /// 这一版退役树根单元，同 [`TransactionOutput::tombstone_root_unit`]；退役树零根时 `None`。
+    ///
+    /// # Panics
+    /// 同 [`TransactionOutput::tombstone_root_unit`]。
+    #[must_use]
+    pub fn retirement_root_unit(&self) -> Option<&PublishedUnit> {
+        self.unit_at_the_recorded_position(
+            self.container_tree_root_unit_positions.retirement_root,
+            TransactionUnit::RetirementRoot,
+        )
+    }
+
+    /// 这一版墓碑树的内部层数 H：从它的根节点码 2 头现读，层级减一（D3（空间分配） 已定项 2「各从自己的树根节点码 2 头现读」）；零根 0。
+    /// 根单元按装这一版时记下的下标取（[`TransactionOutput::tombstone_root_unit`]），不在 `units` 上找：准入读它（运行时决策路径）。
+    ///
+    /// # Panics
+    /// 这一版有墓碑容器而没有记下墓碑树根单元，或那份字节解不开：单元是这个进程装出来的，或从盘上重建时读出、核过的
+    /// （`recovery::rebuild_version` 把删除那两棵树的每个节点都放进 `units`）。
+    #[must_use]
+    pub fn tombstone_tree_internal_levels_read_from_its_root_node_header(&self) -> u64 {
+        if self.tombstone_containers.is_empty() {
+            return 0;
+        }
+        let root_unit = self
+            .tombstone_root_unit()
+            .expect("有墓碑容器就有墓碑树根单元：装 units 时记下了它的下标");
+        let root_header = parse_index_node(&root_unit.bytes)
+            .expect("墓碑树根单元是这个进程装的，或重建时读出、核过的");
+        u64::from(root_header.level).saturating_sub(1)
+    }
+
+    /// 这一版退役树的内部层数 H，读法同 [`TransactionOutput::tombstone_tree_internal_levels_read_from_its_root_node_header`]；零根 0。
+    ///
+    /// # Panics
+    /// 这一版有退役容器而没有记下退役树根单元，或那份字节解不开（同墓碑树那一份）。
+    #[must_use]
+    pub fn retirement_tree_internal_levels_read_from_its_root_node_header(&self) -> u64 {
+        if self.retirement_containers.is_empty() {
+            return 0;
+        }
+        let root_unit = self
+            .retirement_root_unit()
+            .expect("有退役容器就有退役树根单元：装 units 时记下了它的下标");
+        let root_header = parse_index_node(&root_unit.bytes)
+            .expect("退役树根单元是这个进程装的，或重建时读出、核过的");
+        u64::from(root_header.level).saturating_sub(1)
+    }
+
+    fn unit_at_the_recorded_position(
+        &self,
+        recorded_position: Option<usize>,
+        identity: TransactionUnit,
+    ) -> Option<&PublishedUnit> {
+        let position = recorded_position?;
+        let unit = self
+            .units
+            .get(position)
+            .expect("记下的根单元下标在 units 里：装好 units 之后才记，之后不再挪动");
+        assert_eq!(
+            unit.identity, identity,
+            "units 第 {position} 项是装这一版时记下的那个根单元：装好之后才记，之后不再挪动"
+        );
+        Some(unit)
+    }
+
     /// 这一版里角色是 `identity` 的那个单元（`units` 里最先那一项）。在 `units` 上线性找：一次调用里只查几次的用它；
     /// 要对约 P 个角色各查一次的（照抄全部单元、整理读全部映射条目）先建 [`TransactionOutput::units_by_role`]（七·6）。
     #[must_use]
@@ -6729,13 +6873,15 @@ impl ContainerTreeRoles {
         node_below_the_root: TransactionUnit::RetirementNodeBelowTheRoot,
     };
 
-    /// 形状为 `shape` 的那一版里，`position` 上那个码 2 节点的角色：是那一版的根就是根，别的按位置。
+    /// 根在 `root_of_that_version`（零根 `None`）的那一版里，`position` 上那个码 2 节点的角色：是那一版的根就是根，别的按位置。
+    /// 根的位置由调用方从那一版的形状（[`ContainerTreeShape::root_position`]）或上一版记着的每层节点数
+    /// （[`crate::container_tree::ContainerTreeNodeCounts::root_position`]）取一次。
     fn of_node(
         self,
         position: ContainerTreeNodePosition,
-        shape: &ContainerTreeShape,
+        root_of_that_version: Option<ContainerTreeNodePosition>,
     ) -> TransactionUnit {
-        if shape.is_the_root(position) {
+        if root_of_that_version == Some(position) {
             self.root
         } else {
             (self.node_below_the_root)(position)
@@ -6748,13 +6894,14 @@ fn container_tree_roles_rewritten<Record>(
     tree: &ContainerTreeAfterThisPublish<Record>,
     roles: ContainerTreeRoles,
 ) -> Vec<TransactionUnit> {
+    let root = tree.shape.root_position();
     tree.rewritten_containers
         .iter()
         .map(|position| (roles.container)(*position))
         .chain(
             tree.rewritten_nodes()
                 .into_iter()
-                .map(|position| roles.of_node(position, &tree.shape)),
+                .map(|position| roles.of_node(position, root)),
         )
         .collect()
 }
@@ -6770,9 +6917,12 @@ fn mapped_units_of_a_container_tree<Record>(
 ) -> Vec<(TransactionUnit, Vec<u8>)> {
     let mut mapped_units = Vec::new();
     let mut birth_sequence = 0;
+    // 重写的是最右端连着的那几片：序号不小于头一片就重写了，不在重写清单上逐个找（运行时决策路径，`.claude/rules/fs-design.md`
+    // 「记账是事务的副产品，不是事后的遍历」：映射 key 集决定中央映射树重写哪几个节点，进分配）。
+    let first_rewritten_container = tree.first_rewritten_container();
     for position in 0..tree.containers.len() {
         let identity = (roles.container)(position);
-        if tree.rewritten_containers.contains(&position) {
+        if first_rewritten_container.is_some_and(|first| position >= first) {
             mapped_units.push((
                 identity,
                 key_of_a_unit_born_in_this_publish(UNIT_CLASS_PACKED, birth_sequence),
@@ -6782,8 +6932,10 @@ fn mapped_units_of_a_container_tree<Record>(
             mapped_units.push((identity, key_carried_from_the_previous_version(identity)));
         }
     }
+    let root = tree.shape.root_position();
+    let root_before_this_publish = tree.node_counts_before_this_publish.root_position();
     for (position, origin) in tree.node_origins() {
-        let identity = roles.of_node(position, &tree.shape);
+        let identity = roles.of_node(position, root);
         match origin {
             ContainerTreeNodeOrigin::RewrittenThisPublish => {
                 mapped_units.push((
@@ -6796,7 +6948,7 @@ fn mapped_units_of_a_container_tree<Record>(
                 mapped_units.push((
                     identity,
                     key_carried_from_the_previous_version(
-                        roles.of_node(position, &tree.shape_before_this_publish),
+                        roles.of_node(position, root_before_this_publish),
                     ),
                 ));
             }
@@ -6807,8 +6959,9 @@ fn mapped_units_of_a_container_tree<Record>(
 
 /// 记账标签 6（已承诺预留里墓碑与退役记录那两份，D3（空间分配） 已定项 2）在这次发布之后那一版的值：
 /// 副本数 × ((2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录)：H_墓碑、H_退役是这次之后墓碑树、退役树的内部层数，
-/// n_live 是这次之后 inode 树里 nlink ≥ 1 的记录数（[`crate::admission::live_objects_of_the_inode_tree`]），n_记录 是这次之后 inode 树里全部记录数
-/// （含 nlink = 0 的，[`crate::admission::inode_records_of_the_inode_tree`]；判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条）。
+/// n_live 是这次之后 inode 树里 nlink ≥ 1 的记录数，n_记录 是这次之后 inode 树里全部记录数（含 nlink = 0 的；判决
+/// `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条）：两个数读 [`ResolvedPublish::inode_record_counts`]（随一版带着、
+/// 提交时增量维护，[`InodeRecordCounts`]），不遍历 inode 树；两个 H 按装 `units` 时记下的下标取根单元（[`ContainerTreeRootUnitPositions`]）。
 /// `TransactionOutput` 恒有树表条目（`deleted_inodes` 那一条在内），所以走不到「没有 `deleted_inodes` 条目取 0」那一支——那一支是树表 0 条的一版，
 /// 它不走这条发布路径、没有记账树。记账行写它，发布路径判式子读它（「写入类发布按发布之后的标签判」）：一个函数算，两处读同一个数。
 /// 两个 H 照条款字面从各自根节点的码 2 头现读（[`tombstone_tree_internal_levels_read_from_the_root_node_header`]、
@@ -6823,17 +6976,18 @@ pub fn committed_reservation_for_tombstones_and_retirement_records_after_this_pu
         TombstoneAndRetirementPrepayBasis::VersionWithADeletedInodesTreeTableEntry {
             tombstone_tree_internal_levels:
                 tombstone_tree_internal_levels_read_from_the_root_node_header(resolved, previous),
-            live_objects: live_objects_of_the_inode_tree(&resolved.inode_tree.containers),
+            live_objects: resolved.inode_record_counts.live_objects,
             retirement_tree_internal_levels:
                 retirement_tree_internal_levels_read_from_the_root_node_header(resolved, previous),
-            inode_records: inode_records_of_the_inode_tree(&resolved.inode_tree.containers),
+            inode_records: resolved.inode_record_counts.records,
         },
         replicas,
     )
 }
 
 /// 这次发布之后退役树的内部层数（同 [`tombstone_tree_internal_levels_read_from_the_root_node_header`] 的读法）：这次重写根（有身份消失）时
-/// 新根的头由同一份形状编出，按构造就是 `resolved.retirement_tree.internal_levels()`；这次照抄退役树时读上一版那份 `RetirementRoot` 单元字节的码 2 头。
+/// 新根的头由同一份形状编出，按构造就是 `resolved.retirement_tree.internal_levels()`；这次照抄退役树时读上一版那份 `RetirementRoot` 单元字节的码 2 头
+/// （[`TransactionOutput::retirement_tree_internal_levels_read_from_its_root_node_header`]：根单元按装 `units` 时记下的下标取，不在 `units` 上找）。
 ///
 /// # Panics
 /// 上一版有退役容器而它的单元里没有 `RetirementRoot`，或那份字节解不开：单元是这个进程装出来的，或从盘上重建时读出、核过的。
@@ -6845,22 +6999,18 @@ pub fn retirement_tree_internal_levels_read_from_the_root_node_header(
     if resolved.retirement_tree.is_the_root_rewritten() {
         return resolved.retirement_tree.internal_levels();
     }
-    match previous {
-        Some(previous_version) if !previous_version.retirement_containers.is_empty() => {
-            let root_header =
-                parse_index_node(&previous_version.unit(TransactionUnit::RetirementRoot).bytes)
-                    .expect("退役树根单元是这个进程装的，或重建时读出、核过的");
-            u64::from(root_header.level).saturating_sub(1)
-        }
-        // 上一版退役树零根，这次又没重写它：这次之后仍是零根。
-        Some(_) | None => 0,
-    }
+    // 上一版退役树零根（或没有上一版），这次又没重写它：这次之后仍是零根，读出来是 0。
+    previous.map_or(
+        0,
+        TransactionOutput::retirement_tree_internal_levels_read_from_its_root_node_header,
+    )
 }
 
 /// 这次发布之后墓碑树的内部层数 H（D3（空间分配） 已定项 2「H 从墓碑树根节点码 2 头现读」，代码轮 U8，
 /// `research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：根节点头的层级减一（零根时 0）。
 /// 这次重写根（意图那次往开放容器追加墓碑）：新根的头由同一份形状编出（`build_index_node` 的层级取根在形状里的层级），按构造就是
-/// `resolved.tombstone_tree.internal_levels()`，不另读；这次照抄墓碑树（写入类发布、各批）：读上一版那份 `TombstoneRoot` 单元字节的码 2 头，
+/// `resolved.tombstone_tree.internal_levels()`，不另读；这次照抄墓碑树（写入类发布、各批）：读上一版那份 `TombstoneRoot` 单元字节的码 2 头
+/// （[`TransactionOutput::tombstone_tree_internal_levels_read_from_its_root_node_header`]：根单元按装 `units` 时记下的下标取，不在 `units` 上找），
 /// 照 [`TransactionOutput::height_read_from_the_root_node_header`] 的先例，不读内存里那份形状。
 ///
 /// # Panics
@@ -6874,16 +7024,11 @@ pub fn tombstone_tree_internal_levels_read_from_the_root_node_header(
     if resolved.tombstone_tree.is_the_root_rewritten() {
         return resolved.tombstone_tree.internal_levels();
     }
-    match previous {
-        Some(previous_version) if !previous_version.tombstone_containers.is_empty() => {
-            let root_header =
-                parse_index_node(&previous_version.unit(TransactionUnit::TombstoneRoot).bytes)
-                    .expect("墓碑树根单元是这个进程装的，或重建时读出、核过的");
-            u64::from(root_header.level).saturating_sub(1)
-        }
-        // 上一版墓碑树零根（没有容器就没有根），这次又没重写它：这次之后仍是零根。
-        Some(_) | None => 0,
-    }
+    // 上一版墓碑树零根（没有容器就没有根，或没有上一版），这次又没重写它：这次之后仍是零根，读出来是 0。
+    previous.map_or(
+        0,
+        TransactionOutput::tombstone_tree_internal_levels_read_from_its_root_node_header,
+    )
 }
 
 /// 一次发布在动分配器、动盘之前先算完的东西：这次之后 inode 树是哪几片叶容器、这次重写哪些角色（按 bump 次序）。
@@ -6891,6 +7036,9 @@ pub fn tombstone_tree_internal_levels_read_from_the_root_node_header(
 #[derive(Clone, Debug)]
 pub struct ResolvedPublish {
     pub inode_tree: InodeLeafContainersAfterThisPublish,
+    /// 这次之后 inode 树里的 n_live 与 n_记录：上一版的按这次写进、删掉的那几条改出来（[`InodeRecordCounts::after_writing`]、
+    /// [`InodeRecordCounts::after_removing`]），准入读它，不遍历这次之后的叶容器。
+    pub inode_record_counts: InodeRecordCounts,
     /// 这次之后 extent 树长什么样（D8（核心索引结构） 已定项 14 的两段）。
     pub extent_tree: ExtentTreePlan,
     /// 这次之后第一个文件之外有数据单元的文件长什么样（数据单元与下段；它们在上段里的条目归 `extent_tree` 的上段）。
@@ -7927,6 +8075,8 @@ struct MultiLevelTreesOfThisPublish {
 /// 与写的文件内容切成的一单元事务（没有文件版本的发布为空）。
 struct OrdinaryAllocationPartOfThePublish {
     inode_tree: InodeLeafContainersAfterThisPublish,
+    /// 这次之后 inode 树里的 n_live 与 n_记录（[`ResolvedPublish::inode_record_counts`]）。
+    inode_record_counts: InodeRecordCounts,
     extent_tree: ExtentTreePlan,
     /// 第一个文件之外有数据单元的文件这次之后的样子（这次写的那一个整份重写，别的照抄）。
     files_other_than_the_first: FilesOtherThanTheFirstPlan,
@@ -8230,6 +8380,31 @@ impl PublishPlan<'_> {
                 trees.inode,
             )?,
         };
+        // n_live 与 n_记录随一版带着、按这次写进与删掉的那几条改（准入读它，`.claude/rules/fs-design.md`「空间统计与配额落在第一格」）：
+        // 上一版里那个号的记录按叶序折半找，不遍历 inode 树。
+        let inode_record_counts_before = previous
+            .map_or(InodeRecordCounts::OF_AN_EMPTY_INODE_TREE, |version| {
+                version.inode_record_counts
+            });
+        let record_before_this_publish = |inode: InodeNumber| {
+            previous.and_then(|version| version.inode_record_found_by_its_leaf(inode))
+        };
+        let inode_record_counts = match deletion_and_lineage.step {
+            DeletionStepOfThePublish::FinalBatch { inode } => inode_record_counts_before.after_removing(
+                &record_before_this_publish(inode)
+                    .expect("最后一批删的那条记录在上一版的 inode 树里：不在的那一种 remove_record_from_leaf_containers 已经拒了"),
+            ),
+            DeletionStepOfThePublish::Intent { .. }
+            | DeletionStepOfThePublish::Batch { .. }
+            | DeletionStepOfThePublish::TruncationIntent { .. }
+            | DeletionStepOfThePublish::TruncationBatch { .. }
+            | DeletionStepOfThePublish::TruncationFinalBatch { .. }
+            | DeletionStepOfThePublish::SizeChangeWithoutAnIntent { .. }
+            | DeletionStepOfThePublish::PartialWrite { .. }
+            | DeletionStepOfThePublish::NotADeletion => {
+                inode_record_counts_before.after_writing(&record_writes, record_before_this_publish)
+            }
+        };
         let file_content_transactions = self
             .file_content_transactions_and_the_rewritten_data_units(
                 deletion_and_lineage
@@ -8432,15 +8607,17 @@ impl PublishPlan<'_> {
                     .collect()
             })
             .unwrap_or_default();
-        let tombstone_shape_before = previous
-            .map_or_else(ContainerTreeShape::zero_root, |version| {
-                version.tombstone_tree_index_nodes.shape.clone()
+        // 上一版的形状借过来用，不先照抄一份：追加那一支只拿它的引用（新一版的形状在追加里从它造），照抄那一支才要一份自己的。
+        let zero_root_shape = ContainerTreeShape::zero_root();
+        let tombstone_shape_before: &ContainerTreeShape = previous
+            .map_or(&zero_root_shape, |version| {
+                &version.tombstone_tree_index_nodes.shape
             });
         let tombstone_tree = match deletion_and_lineage.step {
             DeletionStepOfThePublish::Intent { tombstone, .. } => {
                 append_tombstone_to_the_open_container(
                     &tombstone_containers_before,
-                    &tombstone_shape_before,
+                    tombstone_shape_before,
                     tombstone,
                     self.txg,
                     trees.tombstone,
@@ -8460,7 +8637,7 @@ impl PublishPlan<'_> {
             | DeletionStepOfThePublish::PartialWrite { .. }
             | DeletionStepOfThePublish::NotADeletion => TombstoneTreeAfterThisPublish::carried(
                 tombstone_containers_before,
-                tombstone_shape_before,
+                tombstone_shape_before.clone(),
             ),
         };
         // 退役树（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 3–5 条）：这次身份消失的每一片 inode 叶容器
@@ -8475,13 +8652,13 @@ impl PublishPlan<'_> {
                     .collect()
             })
             .unwrap_or_default();
-        let retirement_shape_before = previous
-            .map_or_else(ContainerTreeShape::zero_root, |version| {
-                version.retirement_tree_index_nodes.shape.clone()
+        let retirement_shape_before: &ContainerTreeShape = previous
+            .map_or(&zero_root_shape, |version| {
+                &version.retirement_tree_index_nodes.shape
             });
         let retirement_tree = append_retirement_records_to_the_open_container(
             &retirement_containers_before,
-            &retirement_shape_before,
+            retirement_shape_before,
             &retirement_records_of_this_publish(&inode_tree.retired_identities(), self.txg),
             self.txg,
             trees.retirement,
@@ -8508,6 +8685,7 @@ impl PublishPlan<'_> {
         };
         Ok(OrdinaryAllocationPartOfThePublish {
             inode_tree,
+            inode_record_counts,
             extent_tree,
             files_other_than_the_first,
             data_roles_rewritten,
@@ -8573,6 +8751,7 @@ impl PublishPlan<'_> {
         );
         let OrdinaryAllocationPartOfThePublish {
             inode_tree,
+            inode_record_counts,
             extent_tree,
             files_other_than_the_first,
             data_roles_rewritten,
@@ -8717,6 +8896,7 @@ impl PublishPlan<'_> {
         rewritten_roles.push(TransactionUnit::TreeTable);
         Ok(ResolvedPublish {
             inode_tree,
+            inode_record_counts,
             extent_tree,
             files_other_than_the_first,
             allocation_record_tree,
@@ -8760,14 +8940,16 @@ impl PublishPlan<'_> {
                     .collect()
             })
             .unwrap_or_default();
-        let shape_before = previous.map_or_else(ContainerTreeShape::zero_root, |version| {
-            version.lineage_tree_index_nodes.shape.clone()
+        // 上一版的形状借过来用（同墓碑树那一处）：追加那一支只拿引用，照抄那一支才要一份自己的。
+        let zero_root_shape = ContainerTreeShape::zero_root();
+        let shape_before: &ContainerTreeShape = previous.map_or(&zero_root_shape, |version| {
+            &version.lineage_tree_index_nodes.shape
         });
         match deletion_and_lineage.lineage {
             LineageStepOfThePublish::AdministratorRollback { record, .. } => {
                 append_lineage_record_to_the_open_container(
                     &containers_before,
-                    &shape_before,
+                    shape_before,
                     record,
                     self.txg,
                     trees.lineage,
@@ -8780,7 +8962,7 @@ impl PublishPlan<'_> {
             }
             LineageStepOfThePublish::NoLineageRecord => Ok(LineageTreeAfterThisPublish::carried(
                 containers_before,
-                shape_before,
+                shape_before.clone(),
             )),
         }
     }
@@ -13924,10 +14106,12 @@ where
         birth_sequence,
     };
     let mut containers = Vec::with_capacity(after.containers.len());
+    // 重写的是最右端连着的那几片：序号不小于头一片就重写了，不在重写清单上逐个找。
+    let first_rewritten_container = after.first_rewritten_container();
     for (position, contents) in after.containers.iter().enumerate() {
         let index = (to_build.index_of_position)(position);
         let role = (to_build.roles.container)(position);
-        let node = if after.rewritten_containers.contains(&position) {
+        let node = if first_rewritten_container.is_some_and(|first| position >= first) {
             let birth_sequence = sequences.next(to_build.tree, txg, instance);
             let bytes = build_packed_unit(
                 contents.identity,
@@ -13966,9 +14150,15 @@ where
         BTreeMap::new();
     let mut nodes_below_the_root = Vec::new();
     let mut root = None;
+    let root_position = after.shape.root_position();
+    let root_before_this_publish = after.node_counts_before_this_publish.root_position();
+    // 每个节点的孩子区间每层顺着累加一遍（一个一个问、每个都从这一层最左加起是这一层节点数的平方）。
+    let children_of_every_node = after.shape.children_of_every_node();
     for (position, origin) in after.node_origins() {
-        let identity = to_build.roles.of_node(position, &after.shape);
-        let children = after.shape.children_of(position);
+        let identity = to_build.roles.of_node(position, root_position);
+        let children = children_of_every_node[usize::from(position.level) - 1]
+            [usize::try_from(position.index_in_level).expect("同层序号装得进 usize")]
+        .clone();
         // 这个节点的条目：(分隔 key, 身份引用, 子指针)，按孩子的次序；连同每个孩子那一枝最右那片容器的号。
         let entries_and_largest_container_numbers: Vec<((u64, PackedIdentity, NodePointer), u64)> =
             children
@@ -14051,7 +14241,7 @@ where
                 }
             }
             ContainerTreeNodeOrigin::CarriedFromThePreviousVersion => DeletionTreeNodeUnit {
-                pointer: if after.shape_before_this_publish.is_the_root(position) {
+                pointer: if root_before_this_publish == Some(position) {
                     (to_build.carried_root_pointer)()
                 } else {
                     (to_build.carried_node_below_the_root_pointer)(position)
@@ -14067,7 +14257,7 @@ where
                 largest_container_number,
             ),
         );
-        if after.shape.is_the_root(position) {
+        if root_position == Some(position) {
             root = Some(node);
         } else {
             nodes_below_the_root.push(ContainerTreeNodeBelowTheRootUnit {
@@ -14075,7 +14265,7 @@ where
                 identity,
                 identity_in_the_previous_version: to_build
                     .roles
-                    .of_node(position, &after.shape_before_this_publish),
+                    .of_node(position, root_before_this_publish),
                 node,
             });
         }
@@ -16390,6 +16580,8 @@ fn publish_admitted<Device: BlockDevice>(
     } = written_records
         .pop()
         .expect("一次发布至少一条记录（`roles_named_by_each_record_of_the_publish` 至少给一项）");
+    // 装好这一版的 units 之后记下两棵容器树根单元的下标：下一次准入读 H 按它直接取（运行时决策路径，不在 units 上找）。
+    let container_tree_root_unit_positions = ContainerTreeRootUnitPositions::of_the_units(&units);
     let output = TransactionOutput {
         root,
         record,
@@ -16505,6 +16697,8 @@ fn publish_admitted<Device: BlockDevice>(
             .max(last_transaction_of_this_publish),
         // 落盘之后由 `persist_the_publish_or_freeze_it` 换成这次真写出去的账。
         writes: WritesByStructureKind::NOTHING_WRITTEN,
+        container_tree_root_unit_positions,
+        inode_record_counts: resolved.inode_record_counts,
     };
     Ok((writes, output))
 }
@@ -16695,6 +16889,66 @@ fn build_multi_level_tree(
 mod tests {
     use super::*;
 
+    /// 根之下节点的指针按位置直接取，不在指针表上逐项找（运行时决策路径之外装单元时每个照抄的节点都问一次，逐项找合起来是节点数的平方）：
+    /// 层级 1 有 20000 个节点的三层形状（根之下 20200 个节点），指针表按 bump 次序、每项的出生序号就是它的下标；表头那一项故意记成要问的
+    /// 那个位置、带另一个指针——逐项找的写法先碰到它、交回那个错的，按下标取的写法只看「这一层之前各层的节点数之和 + 同层序号」那一格。
+    #[test]
+    fn a_node_pointer_below_the_root_is_taken_at_its_position_in_bump_order() {
+        const LEVEL_ONE_NODES: usize = 20_000;
+        const LEVEL_TWO_NODES: usize = 200;
+        let shape = ContainerTreeShape::from_children_of_each_node_at_each_level(vec![
+            vec![3; LEVEL_ONE_NODES],
+            vec![100; LEVEL_TWO_NODES],
+            vec![LEVEL_TWO_NODES],
+        ])
+        .expect("每层孩子数之和等于下一层的节点数、根一个节点");
+        let pointer_numbered = |number: u32| NodePointer {
+            head: NodePointer::empty_root().head,
+            locations: NodePointer::empty_root().locations,
+            instance: NodePointer::empty_root().instance,
+            birth_sequence: BirthSequence(number),
+        };
+        let asked = ContainerTreeNodePosition {
+            level: 2,
+            index_in_level: 7,
+        };
+        let mut pointers_below_the_root: Vec<(ContainerTreeNodePosition, NodePointer)> = shape
+            .positions_in_bump_order()
+            .into_iter()
+            .filter(|position| !shape.is_the_root(*position))
+            .zip(0..)
+            .map(|(position, number)| (position, pointer_numbered(number)))
+            .collect();
+        pointers_below_the_root[0] = (asked, pointer_numbered(u32::MAX));
+        let version = ContainerTreeIndexNodesVersion {
+            shape,
+            pointers_below_the_root,
+        };
+        assert_eq!(
+            version.pointer_below_the_root(asked),
+            Some(pointer_numbered(
+                u32::try_from(LEVEL_ONE_NODES + 7).expect("两万多装得进 u32")
+            )),
+            "层级 2 第 7 个在 bump 次序里是层级 1 的 20000 个之后的第 7 项"
+        );
+        assert_eq!(
+            version.pointer_below_the_root(ContainerTreeNodePosition {
+                level: 3,
+                index_in_level: 0
+            }),
+            None,
+            "根不在根之下的指针表里"
+        );
+        assert_eq!(
+            version.pointer_below_the_root(ContainerTreeNodePosition {
+                level: 2,
+                index_in_level: u32::try_from(LEVEL_TWO_NODES).expect("两百装得进 u32")
+            }),
+            None,
+            "形状里没有的位置"
+        );
+    }
+
     /// D5（快照 / 空间记账机制） 已定项 14 的左边界：birth == S.txg 时快照引用它（进 deadlist），birth == S.txg + 1 时不引用（释放）；
     /// 映射 key 不是 27 字节（birth 取不到）默认进 deadlist（「判据的输入取不到时默认进 deadlist」）。没打过快照（p = 0）时一律释放。
     #[test]
```

## 整理门（m4-close-code-r1-fix-d-writer）

### `crates/singlefs-core/src/verified_model.rs`

```diff
diff --git a/crates/singlefs-core/src/verified_model.rs b/crates/singlefs-core/src/verified_model.rs
index ffec375..ca4a049 100644
--- a/crates/singlefs-core/src/verified_model.rs
+++ b/crates/singlefs-core/src/verified_model.rs
@@ -2,7 +2,7 @@
 //! `singlefs-format` 常量的规格。
 //!
 //! 出处：判决 `research/prompts/m4-exit5-relocation-proof-r3-main-verification.md` 第三节「层与射程」「抽象状态」
-//! （用户 2026-10-26 定「core 直验，连胶水一起证」「里程碑四全验」）。这份文件是规格底座：只放 spec，不接协议核，也不被产品代码调用；
+//! （用户 2026-10-26 定「core 直验，连胶水一起证」「里程碑四全验」）。这份文件是规格底座：只放 spec 与钉住它们取值的见证（proof fn），不接协议核，也不被产品代码调用；
 //! 把这里的模型接到 `singlefs-replay-prefix` 的 `gc_ring_interval_proofs`（搬迁协议的根环版）与 `gc_crash_proofs`（回收门槛）归后面的簇。
 //! 与协议核的抽象状态对得上的地方，每一处在注释里写一行指过去；两边的差别（协议层是数学整数、没有设备，core 是 u64、两块盘各一份）
 //! 也写在那一行。
@@ -198,13 +198,79 @@ pub open spec fn roots_ascend_by_checkpoint_txg(roots: Seq<RingRootModel>) -> bo
         0 <= earlier < later < roots.len() ==> roots[earlier].checkpoint_txg.0 < roots[later].checkpoint_txg.0
 }
 
-// 对到 gc_ring_interval_proofs 的 run_ring_is_shaped。
+// 对到 gc_ring_interval_proofs 的 run_ring_is_shaped，逐条：
+// - 至少一条根、根环至少容一条：那边的 roots.len() >= 1 与 window >= 1；
+// - 持久 F 不高于内存 F：那边的 persisted_floor <= in_memory_floor；
+// - 内存 F 不高于最后一条根的号（内存 F 之上至少留一条根）：那边的 in_memory_floor < roots.len()。那边的 F 是根序列的下标，这里是 checkpoint 号，
+//   根按号严格递增时「内存 F 之上至少一条根」就是最后一条根的号不低于内存 F；
+// - 持久 F 之上的根装得进根环（受保护的根都在最后 window 条里）：那边的 roots.len() - persisted_floor <= window。按号递增时受保护的根是一段后缀，
+//   这一段不长于 window；
+// - 按号严格递增、持久 F 是最后一条根带的 F：那边没有（F 是下标、根不带 F），这里换算要用。
+// 「内存 F 不高于最后一条根的号」与「持久 F 之上的根装得进根环」这两条是 `research/prompts/m4-close-code-r1-opus-s2-output.md`
+// （T8「Y5 那座桥缺着」）点出这里原先少的；见证在
+// witness_the_root_ring_shape_keeps_a_root_above_the_floor_and_fits_the_window。
 pub open spec fn root_ring_is_shaped(ring: RootRingModel) -> bool {
     &&& ring.roots.len() >= 1
     &&& ring.window >= 1
     &&& roots_ascend_by_checkpoint_txg(ring.roots)
     &&& ring.persisted_floor == ring.roots.last().rollback_floor
     &&& ring.persisted_floor.0 <= ring.in_memory_floor.0
+    &&& ring.in_memory_floor.0 <= ring.roots.last().checkpoint_txg.0
+    &&& forall|index: int| #[trigger] root_is_protected(ring, index) ==> ring.roots.len() - ring.window <= index
+}
+
+// root_ring_is_shaped 补的两条各拒一个原先算成形的根环：两条根（checkpoint 号 1、2，都带 F = 1）、持久 F 与内存 F 都是 1、根环容 2 条，成形；
+// 只把内存 F 抬到 3（高过最后一条根，那边的内存 F 之上一条根都不剩），或只把根环容量改成 1（持久 F 之上两条根装不进），原先那五条都还满足，
+// 补的两条各拒其中一个。
+pub proof fn witness_the_root_ring_shape_keeps_a_root_above_the_floor_and_fits_the_window() {
+    let mapping = CentralMappingModel { entries: Map::<Seq<u8>, MappingValueModel>::empty() };
+    let older = RingRootModel {
+        checkpoint_txg: CheckpointTxg(1),
+        rollback_floor: CheckpointTxg(1),
+        mapping,
+        publisher: RootPublisherModel::UserPublish,
+    };
+    let newer = RingRootModel {
+        checkpoint_txg: CheckpointTxg(2),
+        rollback_floor: CheckpointTxg(1),
+        mapping,
+        publisher: RootPublisherModel::UserPublish,
+    };
+    let roots = seq![older, newer];
+    assert(roots_ascend_by_checkpoint_txg(roots)) by {
+        assert forall|earlier: int, later: int|
+            #![trigger roots[earlier], roots[later]]
+            0 <= earlier < later < roots.len() implies roots[earlier].checkpoint_txg.0 < roots[later].checkpoint_txg.0 by {
+            assert(earlier == 0 && later == 1);
+        }
+    }
+    assert(roots.last() == newer);
+    let shaped = RootRingModel {
+        roots,
+        in_memory_floor: CheckpointTxg(1),
+        persisted_floor: CheckpointTxg(1),
+        head: mapping,
+        window: 2,
+    };
+    assert(root_ring_is_shaped(shaped));
+    let in_memory_floor_above_every_root = RootRingModel {
+        roots,
+        in_memory_floor: CheckpointTxg(3),
+        persisted_floor: CheckpointTxg(1),
+        head: mapping,
+        window: 2,
+    };
+    assert(!root_ring_is_shaped(in_memory_floor_above_every_root));
+    let window_shorter_than_the_protected_roots = RootRingModel {
+        roots,
+        in_memory_floor: CheckpointTxg(1),
+        persisted_floor: CheckpointTxg(1),
+        head: mapping,
+        window: 1,
+    };
+    assert(roots[0] == older);
+    assert(root_is_protected(window_shorter_than_the_protected_roots, 0));
+    assert(!root_ring_is_shaped(window_shorter_than_the_protected_roots));
 }
 
 // 持久 F 之上的根：回收与覆写要避开它们罩着的每个槽。
```

### `crates/singlefs-replay-prefix/src/README.md`

```diff
diff --git a/crates/singlefs-replay-prefix/src/README.md b/crates/singlefs-replay-prefix/src/README.md
index 16bf78b..389056f 100644
--- a/crates/singlefs-replay-prefix/src/README.md
+++ b/crates/singlefs-replay-prefix/src/README.md
@@ -64,7 +64,7 @@
 | Relocate | key 在现行映射里；源、目的区间都在盘上、跨度相同、字节相同 | 被搬 key 改区间不改内容；其余 key 不变；根与读者继续指旧区间 |
 | UserPublish | 新视图在此刻的盘上读对 | 只换现行 |
 | UserCheckpoint | 无 | 现行落成新根；根数超过容量时最旧的候选被挤出（内存 F 加一），持久 F 跟上新的内存 F；新 F 之上的旧根原本就受保护 |
-| CompactionCheckpoint | 无 | 现行落成新根；只带持久 F：根数超过容量时持久 F 之上最旧的根被挤出（持久 F 加一），内存 F 不低于它；持久 F 只升不降，新持久 F 之上的旧根原本就受保护 |
+| CompactionCheckpoint | 现行的逻辑内容与最后一个根相同（映射可以不同） | 现行落成新根；只带持久 F：根数超过容量时持久 F 之上最旧的根被挤出（持久 F 加一），内存 F 不低于它；持久 F 只升不降，新持久 F 之上的旧根原本就受保护 |
 | Rollback（按版本号） | 版本号是一个回退候选的 | 现行换成那个根的视图，它在持久 F 之上、读对 |
 | RaiseFloorInMemory（按版本号） | 同上 | 只改内存 F；保护范围看持久 F，不变 |
 | AcquireReader / ReleaseReader | 释放时下标有效 | 追加现行，或删去一个读者 |
@@ -79,7 +79,7 @@
 - `lemma_the_in_memory_floor_and_the_persisted_floor`：只在内存里抬 F 不改任何槽的保护；崩溃让内存 F 回到持久 F；用户那一侧的发布落成根时两者相等；整理的发布落成根时持久 F 至多挤上一格、内存 F 不降。
 - `lemma_a_compaction_publish_over_a_floor_raised_only_in_memory_loses_one_candidate_across_a_crash`：协议核实现员推的那个例子（根环容 3 个根、F 只在内存里抬到版本 2、插一次整理的发布再崩溃）：整理那份崩溃之后的候选是版本 1、2，只比不整理那份少 1 个；整理的发布改回带内存 F 就只剩版本 2（必红世界表里那一行）。
 - 三个反例在这一版上被拒：`lemma_both_slots_of_a_two_slot_unit_of_a_rollback_target_are_refused`（第三轮攻方的两槽单元第二槽，连同回退目标还在读的起点槽）、`lemma_reclaiming_the_source_after_a_compaction_publish_is_refused_while_the_old_root_is_a_candidate`（第一轮攻方的整理发布之后回收源槽）、`lemma_a_floor_raised_only_in_memory_keeps_the_old_root_protected_until_a_publish_carries_it`（只看内存 F 就放开回收）。另有两条非空见证：根环只容一个根时旧根被挤出、源槽可回收；两个逻辑版本与一个旧读者同时满足不变量。
-- G7（`theorem_compaction_keeps_what_the_user_sees`）在用户看得见的那一层（根的内容与名字、内存 F、持久 F、现行内容）上说：同一串用户历史两份都接受了每一步、整理那份另插 k 次整理的发布，之后现行内容相同；整理那份的回退候选（按版本号）是不整理那份候选的后缀、至多少 k 个；抬 F 的上限恰多 k。`lemma_run_ring_steps_project_onto_the_observed_operations` 说根环的每一步在这一层上是哪一步（不发布的步骤不改这一层）；`lemma_compaction_never_admits_an_operation_the_plain_history_refuses` 说整理那份接受的点名不整理那份也接受。历史里可以有崩溃（两份同时崩溃，`lemma_a_crash_of_both_keeps_the_relation`）：对应关系另带持久 F 那一层（持久 F 之上的根同样是后缀、至多少 k 个）与「两份最后一个根的内容相同」，崩溃之后回退候选照样是后缀、至多少 k 个，现行内容照样相同。为此整理的发布只在现行内容与最后一个根相同时发（`compaction_publish_is_accepted`，第三轮辩方穷举同样只在这时插整理的发布）。
+- G7（`theorem_compaction_keeps_what_the_user_sees`）在用户看得见的那一层（根的内容与名字、内存 F、持久 F、现行内容）上说：同一串用户历史两份都接受了每一步、整理那份另插 k 次整理的发布，之后现行内容相同；整理那份的回退候选（按版本号）是不整理那份候选的后缀、至多少 k 个；抬 F 的上限恰多 k。`lemma_run_ring_steps_project_onto_the_observed_operations` 说根环的每一步在这一层上是哪一步（不发布的步骤不改这一层）；`lemma_compaction_never_admits_an_operation_the_plain_history_refuses` 说整理那份接受的点名不整理那份也接受。历史里可以有崩溃（两份同时崩溃，`lemma_a_crash_of_both_keeps_the_relation`）：对应关系另带持久 F 那一层（持久 F 之上的根同样是后缀、至多少 k 个）与「两份最后一个根的内容相同」，崩溃之后回退候选照样是后缀、至多少 k 个，现行内容照样相同。为此整理的发布只在现行内容与最后一个根相同时发（`compaction_publish_is_accepted`，第三轮辩方穷举同样只在这时插整理的发布）。这一条是根环协议里整理那一步的门：投影引理说根环允许的整理的发布恰是这一层接受的，G7 要的「整理那份也接受每一步」由此从协议推出；`lemma_a_compaction_publish_is_refused_while_the_head_is_ahead_of_the_last_root` 是门拦下的那一段历史（用户改了现行、还没落成根时整理的发布不发，搬迁之后、改动落成根之后照发）。core 里整理的发布接在上一版后面只搬单元，攒批的会话在整理起跑之前先把待发排空。
 - `compaction_plan_proofs` 的 P4 按区间：目的区间两两不重叠、不与现行映射的区间重叠、不压住回退候选与读者的区间、跨度等于源区间时，一批与一批批搬到 R 空都不改现行映射与回退候选、读者每个 key 读到的单元。第三轮攻方的「目的地压住回退目标两槽单元的第二槽」在这个前提下不合格（`lemma_a_destination_on_the_second_slot_of_a_rollback_target_unit_breaks_the_premise`）。
 - `lemma_the_reclaim_threshold_leaves_the_released_run_unreferenced`：门槛 `released && generation <= floor` 放行、释放代的含义与 floor 的下界两条前提成立时，记录区间里每个槽都没被引用。两条前提是分配器与 floor 来历那一侧的义务，还没证。
 
@@ -92,7 +92,7 @@ core 的 `CheckpointTxg` 是单字段 u64 newtype，派生 Ord。将 `record.gen
 ## 验证结果应该怎样阅读
 
 - 整 crate 的 verified 数以 `.claude/gate.d/verus-pins.tsv` 的 crate 行为准，0 errors 说明登记的这些义务在工具和规格信任边界内成立。
-- 必红表中注入 assert(false) 检查证明函数确实进入验证；按起点比、门里去掉回退候选、门只看内存 F、回收门恒真、P4 前提不看回退候选、G7 的 k 改成 k − 1 或改成逐项相同、整理的发布改回带内存 F、整理的发布不要求现行内容与最后一个根相同、G7 的对应去掉持久 F 那一层或去掉最后一个根相同、门槛引理去掉一条前提这些变异检查关键前提对结论有作用。锚点覆盖不等于每条语义都穷尽。
+- 必红表中注入 assert(false) 检查证明函数确实进入验证；按起点比、门里去掉回退候选、门只看内存 F、回收门恒真、P4 前提不看回退候选、G7 的 k 改成 k − 1 或改成逐项相同、整理的发布改回带内存 F、整理的发布不要求现行内容与最后一个根相同、根环里整理的发布不设门或要映射也相同、G7 的对应去掉持久 F 那一层或去掉最后一个根相同、门槛引理去掉一条前提这些变异检查关键前提对结论有作用。锚点覆盖不等于每条语义都穷尽。
 - requires 入口注入 false 失败只是未检出入口空真；不是一般可满足性证明。构造见证提供具体非空实例。
 - 独立参考及边界测试验证 core 适配层的具体输入；有限测试不能把适配层提升为已证的一般转换。
 - 协议到 core 的精化（设备屏障/FUA、根与 journal 编解码、校验、真实恢复选择、双盘与隔离、失败重试、u64 的溢出边界）与门槛引理的两条前提仍未证。不能称为整个文件系统的完整崩溃恢复证明。
```

### `crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs`

```diff
diff --git a/crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs b/crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs
index b9ed739..8c57726 100644
--- a/crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs
+++ b/crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs
@@ -18,7 +18,8 @@
 //! - 三个反例在这一版上被拒：第三轮攻方的两槽单元第二槽、第一轮攻方的整理发布之后回收源槽，与只看内存 F 就放开回收那一种。
 //! - G7：整理不改现行内容；整理那份的回退候选（按版本号）是不整理那份的后缀、至多少 k 个，抬 F 的上限恰多 k
 //!   （`theorem_compaction_keeps_what_the_user_sees`，k 是这一段整理占的发布数）。历史里可以有崩溃：崩溃之后回退候选照样是不整理那份的后缀、
-//!   至多少 k 个，现行内容照样相同；整理的发布只在现行内容与最后一个根相同时发（整理不把用户还没落成根的改动先落盘）。
+//!   至多少 k 个，现行内容照样相同；整理的发布只在现行内容与最后一个根相同时发（整理不把用户还没落成根的改动先落盘）：
+//!   这一条是根环协议里整理那一步的门，投影引理把它带到 G7 那一层（G7 要的「整理那份也接受每一步」不靠协议外的假设）。
 //!
 //! 槽号与跨度用 int（协议层的数学整数）；接 core 的 u64 时要另证起点加跨度不溢出。这是协议模型，不是 core 的内存布局：
 //! 没有设备编号、根环槽、journal 编码与屏障。回收门槛 `generation <= floor` 蕴涵这里的「没被引用」那一条在 `gc_crash_proofs`。
@@ -234,6 +235,11 @@ pub open spec fn relocation_is_enabled(state: RunRingState, key: u64, destinatio
 
 // 协议允许的步骤。整理的写（目的地）与回收都走 Write、Reclaim 这两道门；搬迁只换现行映射，要求目的区间已持久、跨度相同、
 // 字节与源相同（完整字节相等强于校验和相等）。
+// 整理的发布只在现行的逻辑内容与最后一个根相同时发（映射可以不同：搬迁只换映射）：现行领先最后一个根（用户的改动还没落成根）时它不发，
+// 不把那些改动先落盘。这是 G7 那一条前提（compaction_publish_is_accepted）在根环这一层的门，投影引理把它带到用户看得见的那一层。
+// core 里整理的发布接在上一版后面（`singlefs-core` 的 `transaction.rs` 里 `publish_compaction_step_after_its_check` 照 `previous` 的树只搬单元），
+// 落成的根与上一个根逻辑内容相同；攒批的会话在整理起跑之前先把待发排空（`mounted_session.rs` 里
+// `MountedSessionWithPendingChanges::run_compaction_in_the_background`），排空不成就不跑。
 pub open spec fn run_ring_step_is_enabled(state: RunRingState, step: RunRingStep) -> bool {
     match step {
         RunRingStep::Write { slot, bytes: _ } => slot_is_unreferenced(state, slot),
@@ -241,7 +247,8 @@ pub open spec fn run_ring_step_is_enabled(state: RunRingState, step: RunRingStep
         RunRingStep::Relocate { key, destination } => relocation_is_enabled(state, key, destination),
         RunRingStep::UserPublish { new_view } => view_reads_its_logical_content(new_view, state.disk),
         RunRingStep::UserCheckpoint => true,
-        RunRingStep::CompactionCheckpoint => true,
+        RunRingStep::CompactionCheckpoint => state.roots.len() >= 1
+            && state.head.logical == state.roots[state.roots.len() - 1].view.logical,
         RunRingStep::Rollback { version } => user_version_is_a_candidate(state.roots, state.in_memory_floor, version),
         RunRingStep::RaiseFloorInMemory { version } => user_version_is_a_candidate(
             state.roots,
@@ -1076,6 +1083,78 @@ pub proof fn lemma_a_compaction_publish_over_a_floor_raised_only_in_memory_loses
     }
 }
 
+// 整理的发布那一道门拦下的历史（`research/prompts/m4-close-code-r1-opus-s2-output.md` T8 G7 那一段手推的反例：用户改现行、整理发布、崩溃）：
+// 起点一个用户根（版本 0，key 7 在槽 10、字节 42；槽 20 也写着 42），现行就是它。把 key 7 搬到槽 20 之后现行的映射与根不同、逻辑内容相同，
+// 整理的发布照发；用户删掉 key 7（UserPublish，还没落成根）之后整理的发布不发——发了就把这次删除先落成根，崩溃之后整理那份重现的是删过的内容、
+// 不整理那份是版本 0；这次删除落成根（UserCheckpoint）之后又照发。
+pub proof fn lemma_a_compaction_publish_is_refused_while_the_head_is_ahead_of_the_last_root()
+    ensures
+        ({
+            let logical = Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]]);
+            let before = RunView { mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }), logical };
+            let deleted = RunView { mapping: Map::<u64, SlotRun>::empty(), logical: Map::<u64, Seq<Seq<u8>>>::empty() };
+            let start = ring_state_of(
+                Map::<int, Seq<u8>>::empty().insert(10int, seq![42u8]).insert(20int, seq![42u8]),
+                seq![RingRoot { view: before, name: RootName::UserVersion { version: 0 } }],
+                0,
+                0,
+                before,
+                Seq::empty(),
+                2,
+                1,
+            );
+            let moved = run_ring_next(start, RunRingStep::Relocate { key: 7, destination: SlotRun { start: 20, span: 1 } });
+            let changed = run_ring_next(start, RunRingStep::UserPublish { new_view: deleted });
+            let carried = run_ring_next(changed, RunRingStep::UserCheckpoint);
+            &&& every_protected_view_reads_right(start)
+            &&& run_ring_step_is_enabled(start, RunRingStep::Relocate { key: 7, destination: SlotRun { start: 20, span: 1 } })
+            &&& moved.head.mapping != moved.roots[0].view.mapping
+            &&& run_ring_step_is_enabled(moved, RunRingStep::CompactionCheckpoint)
+            &&& run_ring_step_is_enabled(start, RunRingStep::UserPublish { new_view: deleted })
+            &&& !run_ring_step_is_enabled(changed, RunRingStep::CompactionCheckpoint)
+            &&& run_ring_step_is_enabled(carried, RunRingStep::CompactionCheckpoint)
+        }),
+{
+    let logical = Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]]);
+    let before = RunView { mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }), logical };
+    let deleted = RunView { mapping: Map::<u64, SlotRun>::empty(), logical: Map::<u64, Seq<Seq<u8>>>::empty() };
+    let start = ring_state_of(
+        Map::<int, Seq<u8>>::empty().insert(10int, seq![42u8]).insert(20int, seq![42u8]),
+        seq![RingRoot { view: before, name: RootName::UserVersion { version: 0 } }],
+        0,
+        0,
+        before,
+        Seq::empty(),
+        2,
+        1,
+    );
+    assert(before.mapping.dom() =~= before.logical.dom());
+    assert(run_bytes_on_disk(start.disk, before.mapping[7u64]) =~= seq![seq![42u8]]);
+    assert(view_reads_its_logical_content(before, start.disk));
+    assert(start.roots[0].view == before);
+    assert(every_protected_view_reads_right(start));
+    let destination = SlotRun { start: 20, span: 1 };
+    assert(run_bytes_on_disk(start.disk, destination) =~= run_bytes_on_disk(start.disk, start.head.mapping[7u64]));
+    assert(relocation_is_enabled(start, 7, destination));
+    let moved = run_ring_next(start, RunRingStep::Relocate { key: 7, destination });
+    assert(moved.roots[0].view == before);
+    assert(moved.head.mapping[7u64] == destination);
+    assert(moved.roots[0].view.mapping[7u64] == (SlotRun { start: 10, span: 1 }));
+    assert(moved.head.mapping != moved.roots[0].view.mapping);
+    assert(moved.head.logical == moved.roots[0].view.logical);
+    assert(deleted.mapping.dom() =~= deleted.logical.dom());
+    assert(view_reads_its_logical_content(deleted, start.disk));
+    let changed = run_ring_next(start, RunRingStep::UserPublish { new_view: deleted });
+    assert(changed.roots[0].view == before);
+    assert(changed.roots[0].view.logical.contains_key(7u64));
+    assert(!changed.head.logical.contains_key(7u64));
+    assert(changed.head.logical != changed.roots[0].view.logical);
+    let carried = run_ring_next(changed, RunRingStep::UserCheckpoint);
+    assert(carried.roots.len() == 2);
+    assert(carried.roots[1].view == deleted);
+    assert(carried.head == deleted);
+}
+
 // 多个逻辑版本的见证：两个根的 key 集合不同（用户在旧根之后写出了新 key），一个读者拿着旧版本，不变量照样成立；
 // 防止「所有视图共享同一份逻辑内容」那一形回来，也防止历史定理只在空映射上成立。
 pub proof fn witness_two_logical_versions_and_an_old_reader_satisfy_the_invariant()
@@ -1147,7 +1226,9 @@ pub proof fn witness_two_logical_versions_and_an_old_reader_satisfy_the_invarian
 // 历史里可以有崩溃（两份同时崩溃；用户 2026-10-26 定整理的发布只带持久 F，G7 连崩溃一起成立）：崩溃让内存 F 回到持久 F、现行回到最后一个根，
 // 所以持久 F 之上的根也要一层一层对得上，两份最后一个根的内容也要相同。整理的发布只在现行内容与最后一个根相同时发
 // （compaction_publish_is_accepted；第三轮辩方穷举 g7_named_roots.py 同样只在这时插整理的发布）：不然它把用户还没落成根的改动先落盘，
-// 崩溃之后整理那份的现行比不整理那份新。
+// 崩溃之后整理那份的现行比不整理那份新。根环那一层的整理的发布带着同一道门（run_ring_step_is_enabled），
+// lemma_run_ring_steps_project_onto_the_observed_operations 说两道门相同；门拦下的那一段历史在
+// lemma_a_compaction_publish_is_refused_while_the_head_is_ahead_of_the_last_root。
 // 只说两份都接受了每一步的历史：同一步两份都拒的是两边的空操作，删掉它不改结论；不整理那份接受、整理那份拒的
 // 就是候选移出窗口本身，之后两份不再是同一串用户历史。整理那份接受、不整理那份拒的从不出现
 // （lemma_compaction_never_admits_an_operation_the_plain_history_refuses）。
@@ -1306,7 +1387,8 @@ pub open spec fn compaction_publish_is_accepted(observation: RingObservation) ->
 }
 
 // 根环的每一步在用户看得见的那一层上是什么：不发布的步骤（写、回收、搬迁、读者取得与释放）不改这一层；用户发布、两种发布、
-// 按版本号回退、只在内存里抬 F 与崩溃各是这一层的一步，根环允许的这一层也接受。
+// 按版本号回退、只在内存里抬 F 与崩溃各是这一层的一步，根环允许的这一层也接受。整理的发布、回退与抬 F 三样，根环允许的恰是这一层接受的：
+// G7 要「整理那份也接受每一步」，其中整理的发布那一条（compaction_publish_is_accepted）由此从根环的门推出，不是另加的假设。
 pub proof fn lemma_run_ring_steps_project_onto_the_observed_operations(state: RunRingState, step: RunRingStep)
     ensures
         match step {
@@ -1323,8 +1405,12 @@ pub proof fn lemma_run_ring_steps_project_onto_the_observed_operations(state: Ru
                 observe_the_run_ring(state),
                 UserOperation::Checkpoint,
             ),
-            RunRingStep::CompactionCheckpoint => observe_the_run_ring(run_ring_next(state, step))
-                == observation_after_a_compaction_publish(observe_the_run_ring(state)),
+            RunRingStep::CompactionCheckpoint => {
+                &&& run_ring_step_is_enabled(state, step) == compaction_publish_is_accepted(observe_the_run_ring(state))
+                &&& observe_the_run_ring(run_ring_next(state, step)) == observation_after_a_compaction_publish(
+                    observe_the_run_ring(state),
+                )
+            },
             RunRingStep::Rollback { version } => {
                 &&& run_ring_step_is_enabled(state, step) == user_operation_is_accepted(
                     observe_the_run_ring(state),
@@ -1362,6 +1448,11 @@ pub proof fn lemma_run_ring_steps_project_onto_the_observed_operations(state: Ru
             assert(observed_roots_of(after.roots) =~= observed_roots_of(state.roots).push(
                 ObservedRoot { logical: state.head.logical, name: RootName::CompactionPublish },
             ));
+            if state.roots.len() >= 1 {
+                assert(observed_roots_of(state.roots)[state.roots.len() - 1] == observed_root_of(
+                    state.roots[state.roots.len() - 1],
+                ));
+            }
         },
         RunRingStep::Rollback { version } => {
             if run_ring_step_is_enabled(state, step) {
```

