# 删除批第 22 批合入的改动原样（2026-10-05）

## 实现员 c：crates.patch（/tmp/claude-1000/m3-batch22/c/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
index 6266361..206b9bb 100644
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -85,7 +85,13 @@ pub const SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY: &str =
 /// 第 70–74 条（下标 69–73）是里程碑三·3 删除线那五条（判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节，
 /// 用户 2026-10-05 定 I-4.4、I-4.6 取乙）：I-9.5、I-3.4、I-4.7、I-4.6、I-4.4，同样只往末尾追加；GPU 判器的扫描内核判前四条
 /// （下标 69–72，删除态扫描，第 18 批 P2），下标 73 的 I-4.4 第一版恒报不适用、内核不置位（`crash_judge_gpu` 的覆盖表）。
-pub const IMPLEMENTED_INVARIANTS: [&str; 74] = [
+///
+/// 第 75–83 条（下标 74–82）是里程碑三·3 墓碑与谱系那九条（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节与
+/// 第四节用户定案，次序照第二节 T13 那一格）：I-9.17、I-9.18、I-9.19、I-9.20、I-9.21、I-9.22、I-9.23、I-9.24、I-9.29，同样只往末尾追加。
+/// I-9.17、I-9.23 在主走读里判（墓碑树、谱系树的内部条目），其余在删除态扫描里每一版各判（I-9.29 跨版本比）；
+/// I-9.21、I-9.22 的旋钮值从镜像外给（`walk::check_pool_image_given_the_tombstone_container_capacity_knob`）。
+/// GPU 判器的内核这一批不置这九位（`crash_judge_gpu` 的覆盖表照登记，内核第 23 批补）。
+pub const IMPLEMENTED_INVARIANTS: [&str; 83] = [
     "I-1.1",
     "I-1.2",
     "I-1.3",
@@ -160,6 +166,15 @@ pub const IMPLEMENTED_INVARIANTS: [&str; 74] = [
     INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
     DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
     INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
+    LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+    QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
+    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
+    TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
+    NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+    TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
+    TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
+    ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
 ];
 
 /// I-1.12：走读跟随的一条指针，偏移 34 的出生树等于被指单元头里的出生树——码 1、码 3 取头偏移 43；码 2 取哪个字段随 C289 定，
@@ -235,6 +250,29 @@ pub const DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE: &str = "I-
 pub const INTENT_DESCRIBES_ONLY_THE_TARGET_STATE: &str = "I-4.4";
 /// I-4.4 在第一版报不适用的理由，照 `.claude/kb/invariants.md` I-4.4 那一行的状态列。
 pub const INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION: &str = "第一版不适用：意图载体（nlink = 0 的记录、`deleted_inodes` 的 key、墓碑）没有可装增量的字段；logged_ops 有字段表时实现";
+/// I-9.17（谱系树条目身份与子头相符）：谱系树每条内部条目的类型段 ∈ {0, 5}；0 ⇒ 子单元是码 2、身份引用三段为 0；5 ⇒ 子容器头四段与身份引用逐字相等、
+/// 子指针头部出生树相等、子容器打包记录类型 5。身份不符单独判红，不记走读失败（判决第二节 T7 取 a）。
+pub const LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER: &str = "I-9.17";
+/// I-9.18（删除队列里的号有谱系上的墓碑）：每一版 `deleted_inodes` 里的号 N，这一版墓碑树里有 (N, N 的记录的对象出生代, 整对象) 且死亡 txg
+/// 在这一版的父链上（每版从自己的 txg 与自己树表里的谱系树起算，判决第二节 T3 取 a）；N 在 inode 树里没有记录的那一形报射程里判不了。
+pub const QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE: &str = "I-9.18";
+/// I-9.19（谱系上的墓碑不指活对象）：每一版墓碑树里在谱系上的墓碑，这一版 inode 树里有号与对象出生代都相同的记录时，那条记录 nlink = 0。
+pub const TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT: &str = "I-9.19";
+/// I-9.20（谱系上的墓碑不重号且记录合法）：在谱系上的墓碑按 (号, 对象出生代) 不重复；全部墓碑每条整对象 [0, 2⁶⁴ − 1]、
+/// 对象出生代 ≤ 死亡 txg ≤ 这一版的 txg；每片墓碑容器记录数 ≤ 815（判决第二节 T14 与第四节用户定案「补」）。
+pub const TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED: &str =
+    "I-9.20";
+/// I-9.21（产品镜像里非最右墓碑容器装满）：没装只供测试的容量旋钮时，每一版墓碑树里容器号沿叶序严格递增、最右那片之外每片记录数 == 815。
+pub const NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE: &str = "I-9.21";
+/// I-9.22（测试镜像里墓碑容器不超旋钮值）：装了旋钮（新入口给了旋钮值）时，每一版墓碑树里容器号沿叶序严格递增、每片记录数 ≤ 旋钮值。
+pub const TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE: &str = "I-9.22";
+/// I-9.23（墓碑树条目身份与子头相符）：与 I-9.17 同形，类型段 ∈ {0, 1}、子容器打包记录类型 1。
+pub const TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER: &str = "I-9.23";
+/// I-9.24（删除队列至多一条 key）：每一版树表里每一条 `deleted_inodes` 条目都判，不管头 ID 配不配得上；空队列的一版报成立（判决第二节 T8 取 a）。
+pub const DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY: &str = "I-9.24";
+/// I-9.29（同号只许一个出生代）：回退候选集里各版的 inode 记录与各版自己谱系上的墓碑合起来，同一棵 inode 树里同一个号只许一个对象出生代
+/// （判决第二节 T10 乙、第四节 T15「写」：D8（核心索引结构） 已定项 6「同一条时间线上 inode 号不复用」）。
+pub const ONE_OBJECT_BIRTH_PER_INODE_NUMBER: &str = "I-9.29";
 
 /// 判定累加器：每条不变量记评估了几次、第一处违例、整条不适用的理由，以及射程里有一部分判不了的理由。
 #[derive(Default)]
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index ba84c7f..8ed0629 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -12,7 +12,7 @@ use singlefs_format::{
     EXTENT_TREE_INTERNAL_ENTRY_BYTES, EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES, INODE_INTERNAL_ENTRY,
     JOURNAL_RECORD_BYTES, JOURNAL_SAFETY_FACTOR, MAPPING_ENTRY_BYTES, NODE_BYTES,
     NODE_POINTER_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES, SLOT_BYTES,
-    TREE_IDENTIFIER_WATERMARK_AT_MKFS, TREE_TABLE_ENTRY_BYTES,
+    TOMBSTONE_CONTAINER_RECORDS, TREE_IDENTIFIER_WATERMARK_AT_MKFS, TREE_TABLE_ENTRY_BYTES,
 };
 
 use crate::position_addressed::{
@@ -42,7 +42,7 @@ use crate::image::{
     ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
     DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
     DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
-    DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
+    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY, DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
     FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
     HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
     INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
@@ -50,10 +50,17 @@ use crate::image::{
     INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
     INTENT_DESCRIBES_ONLY_THE_TARGET_STATE, INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION,
     JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
-    LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH, MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
-    NAMED_ENTRY_MATCHES_THE_UNIT_HEADER, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
+    LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
+    LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER, MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
+    NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
+    NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+    ONE_OBJECT_BIRTH_PER_INODE_NUMBER, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
     POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER, PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
-    REFERENCE_INTERVALS_ARE_EXACT, SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
+    QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE, REFERENCE_INTERVALS_ARE_EXACT,
+    SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
+    TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
+    TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
+    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
     TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
     TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
     UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
@@ -128,6 +135,31 @@ impl InodeInternalEntryType {
     }
 }
 
+/// 墓碑树、谱系树内部条目的类型段 0：子单元是下一层码 2 节点（根长高之后，D18（块里携带什么信息） 已定项 24 第 1、5 条）。
+const CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE: u16 = 0;
+
+/// 墓碑树、谱系树内部条目的类型段（I-9.23（墓碑树条目身份与子头相符） 类型段 ∈ {0, 1}、I-9.17（谱系树条目身份与子头相符） ∈ {0, 5}）：
+/// 盘上读来的 2 字节，未知值做成显式成员。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum ContainerTreeEntryType {
+    /// 类型段 0：子单元是码 2 节点，身份引用的出生树、容器号、出生代三段为 0。
+    InternalNode,
+    /// 类型段等于这棵树的打包记录类型（墓碑 1、谱系 5）：子单元是码 3 容器。
+    Container,
+    /// 条款之外的值：I-9.17 / I-9.23 判该条目损坏，子单元不往下走。
+    Unregistered(u16),
+}
+
+impl ContainerTreeEntryType {
+    fn of(code: u16, record_type_of_the_tree: u16) -> Self {
+        match code {
+            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE => Self::InternalNode,
+            container_type if container_type == record_type_of_the_tree => Self::Container,
+            unregistered => Self::Unregistered(unregistered),
+        }
+    }
+}
+
 /// 叶序里的一片 inode 叶容器：容器号与它装的最小、最大 inode 号（I-9.4（容器号不超最小 key） 第二、三句沿叶序逐对比）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 struct InodeLeafContainerInLeafOrder {
@@ -1953,7 +1985,8 @@ impl<'reader> Walk<'reader> {
             );
         }
         // 墓碑树、谱系树：码 2 根之下是码 3 容器（类型段墓碑 {0, 1}、谱系 {0, 5}，D18（块里携带什么信息） 已定项 24）。
-        // 这里只走到每片容器、判它的头与容器格式（单元算进可达与已有统计）；墓碑与谱系的专门不变量（I-9.17–I-9.24）归下一步。
+        // 这里走到每片容器、判每条内部条目的身份（墓碑树 I-9.23、谱系树 I-9.17）、容器的头与容器格式（单元算进可达与已有统计）；
+        // 记录本身的那几条（I-9.18–I-9.22、I-9.24、I-9.29 与 I-9.10 墓碑那一个等号）在删除态扫描里每一版各判。
         let node_locations = parse_node_pointer(&entry[14..100]).locations;
         let node_placement = (node_locations[0].device, node_locations[0].slot);
         if kind == TREE_KIND_TOMBSTONE {
@@ -1962,6 +1995,7 @@ impl<'reader> Walk<'reader> {
                 node_placement,
                 tree,
                 PACKED_TYPE_TOMBSTONE,
+                TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
                 "墓碑树",
             );
         }
@@ -1971,6 +2005,7 @@ impl<'reader> Walk<'reader> {
                 node_placement,
                 tree,
                 PACKED_TYPE_LINEAGE,
+                LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
                 "谱系树",
             );
         }
@@ -2031,7 +2066,9 @@ impl<'reader> Walk<'reader> {
 
     /// 墓碑树或谱系树的码 2 根（或根长高之后的内部节点）往下走：条目 120 字节（分隔 key 8 + 身份引用 26 + 子指针 86），
     /// 类型段 = `record_type` 的走到码 3 容器（判容器头、出生树、容器格式，单元记进这一版走到的落点），类型段 0 的走进下一层码 2 节点
-    /// （层级 = 父层级 − 1），别的类型段记走读失败、不往下走。用 checker 自己的读法，不调 `singlefs_core`（D13（验证路线） 已定项 5）。
+    /// （层级 = 父层级 − 1），别的类型段判 `identity_invariant` 红、不往下走。每条条目的身份按 `identity_invariant`（墓碑树 I-9.23、
+    /// 谱系树 I-9.17）单独判定，身份不符不记走读失败（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T7 取 a）。
+    /// 用 checker 自己的读法，不调 `singlefs_core`（D13（验证路线） 已定项 5）。
     /// 递归的上界：每进一层类型段 0，层级减一（层级 0 的码 2 节点在这两棵树里没有条款定的形状，记走读失败）。
     ///
     /// 往下记的落点都记在 `node_placement` 这个节点下面（`placements_referenced_below_a_walked_unit`）：别的版本共享这个节点时
@@ -2042,10 +2079,17 @@ impl<'reader> Walk<'reader> {
         node_placement: (u32, u64),
         tree: u64,
         record_type: u16,
+        identity_invariant: &'static str,
         name: &str,
     ) {
         let previous_parent_unit = self.start_walking_below(node_placement);
-        self.walk_the_entries_of_a_container_tree_node(node, tree, record_type, name);
+        self.walk_the_entries_of_a_container_tree_node(
+            node,
+            tree,
+            record_type,
+            identity_invariant,
+            name,
+        );
         self.stop_walking_below(previous_parent_unit);
     }
 
@@ -2055,6 +2099,7 @@ impl<'reader> Walk<'reader> {
         node: &crate::IndexNodeView,
         tree: u64,
         record_type: u16,
+        identity_invariant: &'static str,
         name: &str,
     ) {
         for entry in &node.entries {
@@ -2075,93 +2120,172 @@ impl<'reader> Walk<'reader> {
                 &child,
                 &what,
             );
-            if entry_type == 0 {
-                if node.level <= 1 {
-                    self.walk_failures.push(format!(
-                        "{name}层级 {} 的节点有类型段 0 的条目：孩子会是层级 0 的码 2 节点，这两棵树里没有条款定的形状",
-                        node.level
-                    ));
-                    continue;
-                }
-                let schema = KEY_SCHEMA_CONTAINER_NUMBER;
-                let Some(child_node) = self.read_index_node(
-                    &entry[34..120],
-                    tree,
-                    schema,
-                    &format!("{name}根之下的内部节点"),
-                    KeyRangeReading::FirstAndLastEntries,
-                    EntryFieldTableRegistration::Registered,
-                ) else {
-                    continue;
-                };
-                self.judgements
-                    .judge("I-1.1", child_node.level + 1 == node.level, || {
+            match ContainerTreeEntryType::of(entry_type, record_type) {
+                ContainerTreeEntryType::InternalNode => {
+                    // 「子单元是码 2」与 I-1.6（类标签登记且三处相等） 重合，照 I-9.2（条目身份与子头相符） 的先例留作合取
+                    // （第二节 T7）：读不出的孩子这一小句不判，读不出归 I-2.1 与走读失败。
+                    let (birth_tree, _, container, container_birth) = identity;
+                    let child_class = read_unit_without_judging(self.reader, &child, node_bytes())
+                        .map(|unit| unit[6]);
+                    let entry_holds = birth_tree == 0
+                        && container == 0
+                        && container_birth == 0
+                        && child_class.is_none_or(|class| class == 2);
+                    self.judgements.judge(identity_invariant, entry_holds, || {
                         format!(
-                            "{name}内部节点层级 {} 不是父层级 {} − 1",
-                            child_node.level, node.level
+                            "{name}内部条目类型段 0：孩子的类标签 {child_class:?}（要 2），身份引用的出生树 / 容器号 / 出生代 {birth_tree} / {container} / {container_birth}（要三段为 0）"
                         )
                     });
-                if child_node.level + 1 == node.level
-                    && child_node.entry_width == inode_internal_entry_bytes()
-                {
-                    self.walk_container_tree_root(
-                        &child_node,
-                        (child.locations[0].device, child.locations[0].slot),
+                    self.walk_the_internal_node_below_a_container_tree_entry(
+                        node,
+                        entry,
+                        &child,
                         tree,
                         record_type,
+                        identity_invariant,
                         name,
                     );
                 }
-                continue;
-            }
-            if entry_type != record_type {
-                self.walk_failures.push(format!(
-                    "{name}内部条目的类型段 {entry_type} 不在 {{0, {record_type}}} 里"
-                ));
-                continue;
-            }
-            self.note_references_of_a_pointer_followed_by_the_walk(&child, 2, &what);
-            let Some(unit) = read_referenced_unit(
-                self.reader,
-                &mut self.judgements,
-                &child.locations,
-                data_unit_bytes(),
-                &what,
-            ) else {
-                self.walk_failures
-                    .push(format!("{name}的容器两份都读不到对得上的"));
-                continue;
-            };
-            let child_placement = (child.locations[0].device, child.locations[0].slot);
-            if !self.visited_units.insert(child_placement) {
-                self.merge_placements_below_a_unit_walked_by_an_earlier_version(child_placement);
-                continue;
+                ContainerTreeEntryType::Container => {
+                    self.walk_the_container_below_a_container_tree_entry(
+                        &child,
+                        identity,
+                        tree,
+                        record_type,
+                        identity_invariant,
+                        name,
+                    );
+                }
+                ContainerTreeEntryType::Unregistered(unregistered_type) => {
+                    // 先登记引用再判红：不登记的话这个孩子占的槽不进引用并集，I-3.1（已分配统计对得上） 连带红（第二节 T7）。
+                    // 孩子的形状由这一层定：层级 1 的节点下面是码 3 容器（两槽），层级 ≥ 2 的下面是码 2 节点（一槽）。
+                    let span_in_slots = if node.level >= 2 { 1 } else { 2 };
+                    self.note_references_of_a_pointer_followed_by_the_walk(
+                        &child,
+                        span_in_slots,
+                        &what,
+                    );
+                    self.judgements.judge(identity_invariant, false, || {
+                        format!("{name}内部条目的类型段 {unregistered_type} 不在 {{0, {record_type}}} 里")
+                    });
+                }
             }
-            let container_name = format!("{name}的容器");
-            self.judgements
-                .judge("I-1.3", read_u64(&unit, 43) == tree, || {
-                    format!(
-                        "{container_name}的出生树 {} 不是 {tree}",
-                        read_u64(&unit, 43)
-                    )
-                });
-            let header_identity = (
-                read_u64(&unit, 43),
-                read_u16(&unit, 51),
-                read_u64(&unit, 53),
-                read_u64(&unit, 61),
+        }
+    }
+
+    /// 墓碑树、谱系树里类型段 0 那条条目下面的码 2 节点：层级要是父层级 − 1，条目宽 120 才往下走（[`Self::walk_container_tree_root`]）。
+    #[allow(
+        clippy::too_many_arguments,
+        reason = "条目、孩子指针与树的三样都是这一步要的输入，各有名字与类型，不为凑短签名收成结构体"
+    )]
+    fn walk_the_internal_node_below_a_container_tree_entry(
+        &mut self,
+        node: &crate::IndexNodeView,
+        entry: &[u8],
+        child: &PointerView,
+        tree: u64,
+        record_type: u16,
+        identity_invariant: &'static str,
+        name: &str,
+    ) {
+        if node.level <= 1 {
+            self.walk_failures.push(format!(
+                "{name}层级 {} 的节点有类型段 0 的条目：孩子会是层级 0 的码 2 节点，这两棵树里没有条款定的形状",
+                node.level
+            ));
+            return;
+        }
+        let Some(child_node) = self.read_index_node(
+            &entry[34..120],
+            tree,
+            KEY_SCHEMA_CONTAINER_NUMBER,
+            &format!("{name}根之下的内部节点"),
+            KeyRangeReading::FirstAndLastEntries,
+            EntryFieldTableRegistration::Registered,
+        ) else {
+            return;
+        };
+        self.judgements
+            .judge("I-1.1", child_node.level + 1 == node.level, || {
+                format!(
+                    "{name}内部节点层级 {} 不是父层级 {} − 1",
+                    child_node.level, node.level
+                )
+            });
+        if child_node.level + 1 == node.level
+            && child_node.entry_width == inode_internal_entry_bytes()
+        {
+            self.walk_container_tree_root(
+                &child_node,
+                (child.locations[0].device, child.locations[0].slot),
+                tree,
+                record_type,
+                identity_invariant,
+                name,
             );
-            if header_identity != identity {
-                self.walk_failures.push(format!(
-                    "{name}内部条目的身份 {identity:?} 与容器头 {header_identity:?} 不符"
-                ));
-            }
-            self.judge_birth_identity_of_a_referenced_unit(&unit, &child, &container_name);
-            // 容器格式（类标签、记录类型、记录宽与声明长度）按 I-1.7 那一套判；记录本身的不变量归 I-9.17 起那几条。
-            let _records = self.judge_packed_container(&unit, record_type, &container_name);
         }
     }
 
+    /// 墓碑树、谱系树里类型段 = `record_type` 那条条目下面的码 3 容器：先登记引用、读容器，按 `identity_invariant` 判条目身份
+    /// （容器头四段与身份引用逐字相等、子指针头部出生树相等、类标签 3 与打包记录类型重合 I-1.6 / I-1.7 的那几小句留作合取）；
+    /// 这一遍第一次走到时再判树 ID（I-1.3）、出生身份（I-1.2 / I-4.2）与容器格式（I-1.7）。
+    fn walk_the_container_below_a_container_tree_entry(
+        &mut self,
+        child: &PointerView,
+        identity: (u64, u16, u64, u64),
+        tree: u64,
+        record_type: u16,
+        identity_invariant: &'static str,
+        name: &str,
+    ) {
+        let what = format!("{name}内部条目的子指针");
+        self.note_references_of_a_pointer_followed_by_the_walk(child, 2, &what);
+        let Some(unit) = read_referenced_unit(
+            self.reader,
+            &mut self.judgements,
+            &child.locations,
+            data_unit_bytes(),
+            &what,
+        ) else {
+            self.walk_failures
+                .push(format!("{name}的容器两份都读不到对得上的"));
+            return;
+        };
+        let header_identity_of_the_container = (
+            read_u64(&unit, 43),
+            read_u16(&unit, 51),
+            read_u64(&unit, 53),
+            read_u64(&unit, 61),
+        );
+        let container_entry_holds = unit[6] == 3
+            && header_identity_of_the_container == identity
+            && header_identity_of_the_container.1 == record_type
+            && child.birth_tree == identity.0;
+        self.judgements
+            .judge(identity_invariant, container_entry_holds, || {
+                format!(
+                    "{name}内部条目的身份 {identity:?} 与容器头 {header_identity_of_the_container:?}（类 {}、子指针出生树 {}）不符",
+                    unit[6], child.birth_tree
+                )
+            });
+        let child_placement = (child.locations[0].device, child.locations[0].slot);
+        if !self.visited_units.insert(child_placement) {
+            self.merge_placements_below_a_unit_walked_by_an_earlier_version(child_placement);
+            return;
+        }
+        let container_name = format!("{name}的容器");
+        self.judgements
+            .judge("I-1.3", read_u64(&unit, 43) == tree, || {
+                format!(
+                    "{container_name}的出生树 {} 不是 {tree}",
+                    read_u64(&unit, 43)
+                )
+            });
+        self.judge_birth_identity_of_a_referenced_unit(&unit, child, &container_name);
+        // 容器格式（类标签、记录类型、记录宽与声明长度）按 I-1.7 那一套判；记录本身的不变量归删除态扫描（I-9.18 起那几条）。
+        let _records = self.judge_packed_container(&unit, record_type, &container_name);
+    }
+
     /// I-9.1：inode 树根恒码 2（读到这里时它已经按码 2 判过头）；层级 ≥ 1 的根往下走（[`Self::walk_inode_internal_node`]），
     /// 走完按叶序判 I-9.4（容器号不超最小 key） 的第二、三句（[`Self::judge_container_numbers_along_the_leaf_order`]）。
     fn walk_inode_root(
@@ -2180,7 +2304,7 @@ impl<'reader> Walk<'reader> {
     }
 
     /// I-9.4（容器号不超最小 key） 的第二、三句：沿叶序（这一版的 inode 树从左到右）逐对比，容器号严格递增，且左容器的最大 key < 右容器号
-    /// （这条逼出合并只许左吸收右 / 左从右借）。第一句（记录 inode ≥ 容器号）在走叶容器时逐条判。
+    /// （这条逼出合并只许左吸收右 / 左从右借）。第一句（记录 inode ≥ 容器号）与第四句（同一片容器里记录号沿记录序严格递增）在走叶容器时逐条判。
     /// 叶序里有别的版本已经走过的叶（缓存的容器号与 key 区间）；读不出的叶不在叶序里，它两边的叶直接相邻着比
     /// （合法镜像上容器号与 key 沿叶序单调，隔一片比照样成立）。
     fn judge_container_numbers_along_the_leaf_order(
@@ -2335,7 +2459,7 @@ impl<'reader> Walk<'reader> {
     }
 
     /// 一片 inode 叶容器（码 3 打包记录类型 2）：这一遍第一次走到时判树 ID（I-1.3）、出生身份（I-1.2 / I-4.2）、容器（I-1.7 等）与容器里
-    /// 每条记录（I-9.13、I-9.7、I-9.4 第一句、I-9.15），交回它的容器号与装的最小、最大 inode 号；别的版本已经走过时不重读，
+    /// 每条记录（I-9.13、I-9.7、I-9.4 第一句与第四句、I-9.15），交回它的容器号与装的最小、最大 inode 号；别的版本已经走过时不重读，
     /// 从 `inode_leaf_order_below_a_walked_unit` 接上它在叶序里的那一格，它下面的落点并进这一版（I-5.1）。
     fn walk_inode_leaf_container(
         &mut self,
@@ -2370,8 +2494,19 @@ impl<'reader> Walk<'reader> {
             });
             let mut smallest_inode_in_this_container: Option<u64> = None;
             let mut largest_inode_in_this_container: Option<u64> = None;
+            let mut inode_of_the_previous_record: Option<u64> = None;
             for record in records {
                 let inode = read_u64(&record, 0);
+                // I-9.4 第四句：同一片打包记录类型 2 容器里记录号沿记录序严格递增（判决
+                // `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T11 取甲）。与第二、三句合起来就是整棵树的 key 沿叶序严格递增。
+                if let Some(previous_inode) = inode_of_the_previous_record {
+                    self.judgements.judge("I-9.4", inode > previous_inode, || {
+                        format!(
+                            "容器 {container} 里记录号沿记录序不严格递增：inode {previous_inode} 之后是 inode {inode}"
+                        )
+                    });
+                }
+                inode_of_the_previous_record = Some(inode);
                 self.judgements.judge(
                     "I-9.7",
                     record[108..140].iter().all(|byte| *byte == 0),
@@ -7966,15 +8101,18 @@ fn slots_referenced_per_device(
 }
 
 /// 删除态扫描要判的一版（I-9.5（删除队列里的号仍在树里） 那一行的共用判法：回退候选集里每条根，加由记录施加出来的那几版）：
-/// 称呼（判红与判不了时写进说明）与它的树表指针（根环里的根取根记录偏移 36 起的 86 字节，由记录施加出来的那一版取记录新根段里的那一条）。
+/// 称呼（判红与判不了时写进说明）、它的 `checkpoint_txg`（「在谱系上」从这一版自己的 txg 起算，I-9.20 比死亡 txg 的上界）
+/// 与它的树表指针（根环里的根取根记录偏移 36 起的 86 字节，由记录施加出来的那一版取记录新根段里的那一条）。
 struct VersionToScanTheDeletionStateOf<'record> {
     name: String,
+    checkpoint_txg: u64,
     tree_table_pointer: &'record [u8],
 }
 
-/// 删除态扫描在一版上读出的一条 inode 记录里要的两段（D8（核心索引结构） 已定项 6 字段表：nlink 偏移 36、size 偏移 40）。
+/// 删除态扫描在一版上读出的一条 inode 记录里要的三段（D8（核心索引结构） 已定项 6 字段表：对象出生代偏移 8、nlink 偏移 36、size 偏移 40）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 struct InodeRecordOfTheDeletionScan {
+    object_birth: u64,
     nlink: u32,
     size_in_bytes: u64,
 }
@@ -7995,6 +8133,13 @@ enum DeletionScanGap {
     ExtentTreeUnreadable,
     AccountingTreeUnreadable,
     TombstoneTreeUnreadable,
+    TombstoneTreeOfTheVersionUnreadable,
+    TombstoneTreeHeadIdentifierNamesNoSingleInodeTree,
+    InodeTreeNamedByNoSingleTombstoneTreeEntry,
+    LineageTreeEntryNotSingle,
+    LineageTreeUnreadable,
+    LineageRecordsInconsistent,
+    QueuedInodeWithoutARecord,
 }
 
 impl DeletionScanGap {
@@ -8033,11 +8178,35 @@ impl DeletionScanGap {
             DeletionScanGap::TombstoneTreeUnreadable => {
                 "所选根的树表或墓碑树读不全：水位大于墓碑对象 ID 那一半判不了"
             }
+            DeletionScanGap::TombstoneTreeOfTheVersionUnreadable => {
+                "走读到的某一版墓碑树读不全（节点或容器读不出、层级或条目宽不对、类型段不认得、容器的打包记录类型或记录宽不是登记的）：那一版的墓碑判不了"
+            }
+            DeletionScanGap::TombstoneTreeHeadIdentifierNamesNoSingleInodeTree => {
+                "走读到的某一版墓碑树条目偏移 116 的头 ID 配不上同一张树表里恰好一条 inode 树条目：那一版墓碑对 inode 记录的那几判判不了"
+            }
+            DeletionScanGap::InodeTreeNamedByNoSingleTombstoneTreeEntry => {
+                "走读到的某一版树表里有一棵 inode 树，同一张表里偏移 116 的头 ID 指着它的墓碑树条目不是恰好一条：那一版这棵树的队列号判不了"
+            }
+            DeletionScanGap::LineageTreeEntryNotSingle => {
+                "走读到的某一版树表里谱系树条目不是恰好一条：那一版的「在谱系上」判不了"
+            }
+            DeletionScanGap::LineageTreeUnreadable => {
+                "走读到的某一版谱系树读不全（节点或容器读不出、层级或条目宽不对、类型段不认得、容器的打包记录类型或记录宽不是登记的）：那一版的「在谱系上」判不了"
+            }
+            DeletionScanGap::LineageRecordsInconsistent => {
+                "走读到的某一版谱系记录不合法（有一条 T_old 不小于 T_rb，或同一个 T_rb 有两条）：那一版的父链接不上，「在谱系上」判不了"
+            }
+            DeletionScanGap::QueuedInodeWithoutARecord => {
+                "走读到的某一版 deleted_inodes 里有一个号在同一版 inode 树里没有记录：那个号要的对象出生代没有，它的谱系上墓碑判不了（这一形由 I-9.5 判红）"
+            }
         }
     }
 }
 
-/// 删除态扫描读 I-9.5、I-4.7、I-4.6 三条要的那一组：一版里任一样判不了，三条在那一版都判不了。
+/// 删除态扫描读 I-9.5、I-4.7、I-4.6 三条要的那一组：树表、`deleted_inodes` 与 inode 树那几样一版里任一样判不了（树表读不出、条目窄、
+/// `deleted_inodes` 与 inode 树两个方向配不上、队列或记录读不出），三条在那一版都判不了；extent 树那一侧的三种（extent 树指着这棵 inode 树的
+/// 不是恰好一条、extent 树读不全、extent 树的头 ID 配不上 inode 树）只记 I-4.7、I-4.6，I-9.5 不读 extent 树、照判
+/// （`research/prompts/m3-batch21-code-r1-main-verification.md` 第二节 K1 那一行）。
 const DELETION_QUEUE_SCAN_INVARIANTS: [&str; 3] = [
     DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
     INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
@@ -8135,9 +8304,9 @@ fn inode_records_without_judging(
 
 /// inode 树一个内部节点下面的记录逐条收进 `records`：类型段 2 的叶容器（码 3 打包记录类型 2，记录宽 140）读它的每条记录，
 /// 类型段 0 的内部节点（层级 = 父层级 − 1，且 ≥ 1）往下走。读不出、形状不对的交回 [`DeletionScanGap::InodeTreeUnreadable`]。
-/// 同一个号出现两条时后读到的那条盖掉先读到的。同一片叶里同号两条今天没有不变量管：I-9.4 只管记录号不小于容器号与跨容器的序，
-/// 容器里记录号严格递增没有哪一条判（代码轮判决 `research/prompts/m3-batch20-code-r1-main-verification.md` 第二节 S4a，条款缺口，
-/// 归里程碑三·3 的设计轮那一问）。递归的上界：每下一层层级减一。
+/// 同一个号出现两条时后读到的那条盖掉先读到的：号的唯一性由主走读的 I-9.4 判——同一片叶里同号两条、乱序由第四句（同一片容器里
+/// 记录号沿记录序严格递增，判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T11 取甲），跨叶的由第二、三句
+/// （沿叶序容器号严格递增、左容器最大 key < 右容器号），这一遍只读不判。递归的上界：每下一层层级减一。
 fn collect_inode_records_below(
     reader: &dyn ImageReader,
     node: &crate::IndexNodeView,
@@ -8166,6 +8335,7 @@ fn collect_inode_records_below(
                     records.insert(
                         read_u64(record, 0),
                         InodeRecordOfTheDeletionScan {
+                            object_birth: read_u64(record, INODE_RECORD_OBJECT_BIRTH_OFFSET),
                             nlink: read_u32(record, INODE_RECORD_NLINK_OFFSET),
                             size_in_bytes: read_u64(record, INODE_RECORD_SIZE_OFFSET),
                         },
@@ -8316,10 +8486,10 @@ fn judge_deletion_state_of_the_versions(
             }
         };
         if let Some(gap) = gap {
-            for invariant in DELETION_QUEUE_SCAN_INVARIANTS
-                .into_iter()
-                .chain([PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO])
-            {
+            for invariant in DELETION_QUEUE_SCAN_INVARIANTS.into_iter().chain([
+                PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
+                DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
+            ]) {
                 judgements.part_of_the_range_not_judged(invariant, gap.reason());
             }
         }
@@ -8341,6 +8511,10 @@ fn judge_deletion_state_of_the_versions(
         DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
         "走读到的每一版 inode 树里都没有 nlink = 0 的记录",
     );
+    judgements.not_applicable(
+        DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
+        "走读到的每一版树表里都没有 deleted_inodes 条目（树表指针全零或 0 条）：没有队列可判",
+    );
 }
 
 /// I-3.4（可用空间扣待删占用） 只落「记账标签 4 恒 0」（判决第二节 F5 三格）：这一版树表里每一棵记账树（种类 4）走全了，
@@ -8399,6 +8573,8 @@ fn judge_pending_deletion_occupancy_of_a_version(
 /// 不是恰好一条的记判不了。
 /// I-9.5 每一版每个方向各判一次（队列与 nlink = 0 的号集合两两比，干净的一版也真比过）；I-4.7 每个处于删除中的号判一次、
 /// I-4.6 每条 nlink = 0 的记录判一次（一个都没有时没有对象）。
+/// I-9.24（删除队列至多一条 key）：每一条 `deleted_inodes` 条目都判，读队列在配对之前，配不配得上 inode 树都判；空队列报成立
+/// （判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T8 取 a）。
 fn judge_deletion_queue_of_a_version(
     reader: &dyn ImageReader,
     version_name: &str,
@@ -8415,17 +8591,38 @@ fn judge_deletion_queue_of_a_version(
         .filter(|entry| kind_of(entry) == TREE_KIND_DELETED_INODES)
     {
         let inode_tree = head_identifier_of(&deleted_inodes_entry);
+        let queue_read_back = deleted_inodes_queue_without_judging(
+            reader,
+            &parse_node_pointer(&deleted_inodes_entry[14..100]),
+            index_cache,
+        );
+        match &queue_read_back {
+            Ok(queue) => {
+                judgements.judge(
+                    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
+                    queue.len() <= 1,
+                    || {
+                        format!(
+                        "{version_name}：deleted_inodes（树 {}）里有 {} 条 key {queue:?}，多于一条",
+                        read_u64(deleted_inodes_entry, 0),
+                        queue.len()
+                    )
+                    },
+                );
+            }
+            Err(gap) => {
+                judgements.part_of_the_range_not_judged(
+                    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
+                    gap.reason(),
+                );
+            }
+        }
         let inode_tree_entries: Vec<&Vec<u8>> = tree_table_entries
             .iter()
             .filter(|entry| kind_of(entry) == TREE_KIND_INODE && read_u64(entry, 0) == inode_tree)
             .collect();
         let read_back = match inode_tree_entries.as_slice() {
-            [inode_tree_entry] => deleted_inodes_queue_without_judging(
-                reader,
-                &parse_node_pointer(&deleted_inodes_entry[14..100]),
-                index_cache,
-            )
-            .and_then(|queue| {
+            [inode_tree_entry] => queue_read_back.and_then(|queue| {
                 inode_records_without_judging(
                     reader,
                     &parse_node_pointer(&inode_tree_entry[14..100]),
@@ -8759,89 +8956,1016 @@ fn largest_published_tombstone_object_identifier(
     Ok(largest_object_identifier)
 }
 
-/// 池级 checker 的入口：每条第一版不变量都报，没评估到的报「不适用」并带理由。
-#[must_use]
-pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {
-    let system_configurations = chosen_system_configurations(reader);
-    let mut root_ring_judgements = Judgements::default();
-    let chosen: Vec<_> = system_configurations
-        .iter()
-        .filter_map(|(device, chosen)| {
-            chosen
-                .clone()
-                .map(|(view, geometry)| (*device, view, geometry))
-        })
-        .collect();
-    // 一块盘的两个系统配置槽都无效时**不早退**：恢复会改用别的盘上那一份、照常挂上
-    // （`crates/singlefs-core/src/recovery.rs` 的 `choose_system_configuration`，D22（单元原子性怎么合成） 已定项 8
-    // 「系统配置每盘放一份」买的就是这份冗余）。早退会让这种镜像上每一条不变量都报「不适用」，
-    // 而它是一个挂得上的合法镜像——故障注入让一块盘的两个槽先后写失败就造得出它，判定会被静默放过
-    // （.claude/kb/checks-owed.md 的 C461）。⚠️ 「哪块盘的系统配置全废了」今天没有编号报得出来，仍欠在 C461。
-    // 任一盘任一槽自证过而带这一版读者不收的池级值（格式版本、加密类型、槽距、physical_block_size、环长、journal 环起点、
-    // 根环起点、单元区起始槽号）：实现整池拒绝挂载
-    // （`RecoveryFailure::SystemConfigurationValueRefused`），checker 报违例、别的不变量同「挂不上的镜像」一样不作保
-    // （用户 2026-09-27 定系统配置越界整池拒，实审 A3-checker-2）。只有一部分槽带越界值时不拿别的槽照判下去。
-    let system_configuration_slot_readings_of_each_device =
-        system_configuration_slot_readings(reader);
-    let any_system_configuration_slot_carries_a_refused_value =
-        judge_system_configuration_values_the_reader_accepts(
-            &system_configuration_slot_readings_of_each_device,
-            &mut root_ring_judgements,
-        );
-    if any_system_configuration_slot_carries_a_refused_value {
-        for invariant in crate::image::IMPLEMENTED_INVARIANTS {
-            root_ring_judgements.not_applicable(
-                invariant,
-                "池里有一槽自证过的系统配置带这一版读者不收的池级值：实现整池拒绝挂载，这不是一个挂得上的镜像",
-            );
-        }
-        return root_ring_judgements.into_report();
+/// 墓碑记录 40 字节里另外四段的偏移（D18（块里携带什么信息） 已定项 24 第 2 条：对象 ID 8 + 对象出生代 8 + 区间起 8 + 区间止 8 + 死亡 txg 8），
+/// 按字段表另写一份、不用实现的解析。
+const TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET: usize = 8;
+const TOMBSTONE_RECORD_RANGE_START_OFFSET: usize = 16;
+const TOMBSTONE_RECORD_RANGE_END_OFFSET: usize = 24;
+const TOMBSTONE_RECORD_DEATH_TXG_OFFSET: usize = 32;
+/// 第一版墓碑只许整对象：区间 [0, 2⁶⁴ − 1]（D18（块里携带什么信息） 已定项 24 第 2 条）。
+const TOMBSTONE_WHOLE_OBJECT_RANGE_START: u64 = 0;
+const TOMBSTONE_WHOLE_OBJECT_RANGE_END: u64 = u64::MAX;
+/// 谱系记录 16 字节：T_rb 8（回退那次发布的 txg）+ T_old 8（回退目标的 txg），读作「T_rb 的父是 T_old」（D18（块里携带什么信息） 已定项 24 第 5 条）。
+const LINEAGE_RECORD_ROLLBACK_TXG_OFFSET: usize = 0;
+const LINEAGE_RECORD_PARENT_TXG_OFFSET: usize = 8;
+
+/// 只供测试的墓碑容器容量旋钮压到的每片记录数（D18（块里携带什么信息） 已定项 24 第 3 条：旋钮不落盘，池级 checker 从镜像外拿；
+/// 判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T5 取 K1）。取值 1..=815，构造时验。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct TombstoneContainerCapacityKnob {
+    records_per_container: u64,
+}
+
+impl TombstoneContainerCapacityKnob {
+    /// 1..=815 之外交回 `None`（815 = ⌊(32768 − 136) ÷ 40⌋，产品容量 `TOMBSTONE_CONTAINER_RECORDS`）。
+    #[must_use]
+    pub fn new(records_per_container: u64) -> Option<Self> {
+        (1..=TOMBSTONE_CONTAINER_RECORDS)
+            .contains(&records_per_container)
+            .then_some(Self {
+                records_per_container,
+            })
     }
-    // I-8.1 的三个字段等式（在飞上限、最坏占用、F = 格式常量）任一盘任一可择槽有一格不等：实现择系统配置时同判、整池拒绝挂载
-    // （`RecoveryFailure::SystemConfigurationValueRefused`），checker 与上面 I-7.13 那一支同口径——报 I-8.1 违例、别的不变量一律不作保
-    // （`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 4 行）。三格先判进一份只装它们的判定里：整池不作保时
-    // 别的不变量（连同已判过成立的 I-7.13）都报不适用；三格都成立时再判进这一份，照常往下判。
-    let mut judgements_of_the_journal_ring_geometry_fields_alone = Judgements::default();
-    judge_journal_ring_geometry_fields_agree(
-        &system_configuration_slot_readings_of_each_device,
-        &mut judgements_of_the_journal_ring_geometry_fields_alone,
-    );
-    if judgements_of_the_journal_ring_geometry_fields_alone
-        .violation_count(JOURNAL_RING_GEOMETRY_FIELDS_AGREE)
-        > 0
-    {
-        for invariant in crate::image::IMPLEMENTED_INVARIANTS {
-            judgements_of_the_journal_ring_geometry_fields_alone.not_applicable(
-                invariant,
-                "池里有一槽可择的系统配置 journal 环几何三个字段（在飞上限、最坏占用、F）有一格不等：实现整池拒绝挂载，这不是一个挂得上的镜像",
-            );
+
+    #[must_use]
+    pub const fn records_per_container(self) -> u64 {
+        self.records_per_container
+    }
+}
+
+/// 墓碑与谱系那一遍在一版上判 I-9.18、I-9.19、I-9.20、I-9.29 要的那一组：一版的树表读不出、墓碑树读不全时这几条在那一版判不了
+/// （I-9.21 / I-9.22 只记装没装旋钮的那一条，[`capacity_invariant_of`]）。
+const TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS: [&str; 4] = [
+    QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
+    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
+    TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
+    ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
+];
+
+/// 装没装旋钮决定墓碑容器容量判哪一条：没装（旧入口）判产品那一条 I-9.21，装了判测试那一条 I-9.22。
+const fn capacity_invariant_of(
+    capacity_knob: Option<TombstoneContainerCapacityKnob>,
+) -> &'static str {
+    match capacity_knob {
+        None => NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+        Some(_) => TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
+    }
+}
+
+/// 一条墓碑记录的五段。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct TombstoneRecordOfTheScan {
+    object: u64,
+    object_birth: u64,
+    range_start: u64,
+    range_end: u64,
+    death_txg: u64,
+}
+
+impl TombstoneRecordOfTheScan {
+    fn of(record: &[u8]) -> Self {
+        Self {
+            object: read_u64(record, TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET),
+            object_birth: read_u64(record, TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET),
+            range_start: read_u64(record, TOMBSTONE_RECORD_RANGE_START_OFFSET),
+            range_end: read_u64(record, TOMBSTONE_RECORD_RANGE_END_OFFSET),
+            death_txg: read_u64(record, TOMBSTONE_RECORD_DEATH_TXG_OFFSET),
         }
-        return judgements_of_the_journal_ring_geometry_fields_alone.into_report();
     }
-    judge_journal_ring_geometry_fields_agree(
-        &system_configuration_slot_readings_of_each_device,
-        &mut root_ring_judgements,
-    );
-    // I-8.5（V-池）：全池皆废判红；一块盘交得出可择的系统配置就照常往下判（单盘全废走旁路报告，不进违例通道）。
-    // 带越界值的槽、环几何三格不等的槽整池拒挂，上面那两支整池报不适用、不判这一条。
-    // 一块盘都交不出可择的系统配置、而有槽自证过（incompat 位不认得、R / S 越界这类自证过而不可择的槽）：这不是一个挂得上的镜像，
-    // I-8.5 同别的不变量一起报不适用，不判成成立（`InvariantVerdict` 的「不适用必须带理由，不许静默算成立」）；一个槽都没自证过的照判红。
-    if chosen.is_empty() {
-        let mut judgement_of_some_device_carrying_a_self_verified_slot = Judgements::default();
-        judge_some_device_carries_a_self_verified_system_configuration(
-            &system_configuration_slot_readings_of_each_device,
-            &mut judgement_of_some_device_carrying_a_self_verified_slot,
-        );
-        match judgement_of_some_device_carrying_a_self_verified_slot
-            .first_violation_of(SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION)
-        {
-            Some(detail) => {
-                let detail = detail.to_owned();
-                root_ring_judgements.judge(
-                    SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
-                    false,
-                    || detail,
-                );
+
+    fn covers_the_whole_object(self) -> bool {
+        self.range_start < self.range_end
+            && self.range_start == TOMBSTONE_WHOLE_OBJECT_RANGE_START
+            && self.range_end == TOMBSTONE_WHOLE_OBJECT_RANGE_END
+    }
+}
+
+/// 一片墓碑容器：头里的容器号（I-9.21 / I-9.22 沿叶序比它，不沿根的条目读：根长高之后根的条目是类型段 0、容器号恒 0）与它的记录。
+struct TombstoneContainerOfTheScan {
+    container: u64,
+    records: Vec<TombstoneRecordOfTheScan>,
+}
+
+/// 一版树表里的一棵墓碑树：树 ID、头 ID（I-9.27：等于这张表里那棵 inode 树的树 ID）与按叶序读出的容器（读不全交回判不了的理由）。
+struct TombstoneTreeOfTheScan {
+    tree: u64,
+    head_identifier: u64,
+    containers: Result<Vec<TombstoneContainerOfTheScan>, DeletionScanGap>,
+}
+
+/// 一条谱系记录：T_rb 的父是 T_old。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct LineageRecordOfTheScan {
+    rollback_txg: u64,
+    parent_txg: u64,
+}
+
+/// 一版的父链在 txg 轴上占的那几段（每段闭区间）：「在谱系上」= 死亡 txg 落在其中一段里。
+struct LineageChainOfAVersion {
+    segments_on_the_chain: Vec<(u64, u64)>,
+}
+
+impl LineageChainOfAVersion {
+    fn holds_txg(&self, txg: u64) -> bool {
+        self.segments_on_the_chain
+            .iter()
+            .any(|(lowest_txg, highest_txg)| (*lowest_txg..=*highest_txg).contains(&txg))
+    }
+}
+
+/// 一版里一棵 inode 树的全部记录（号 → 记录），读不全交回判不了的理由。
+type InodeRecordsOfAnInodeTreeOfTheScan =
+    Result<BTreeMap<u64, InodeRecordOfTheDeletionScan>, DeletionScanGap>;
+
+/// 跨版本那张表（I-9.29）：(inode 树的树 ID, inode 号) → 对象出生代 → 第一次在哪一版的什么地方见到它。
+type ObjectBirthsOfEachInodeNumber = BTreeMap<(u64, u64), BTreeMap<u64, String>>;
+
+/// 「在谱系上」的父链（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T3 取 a）：从这一版自己的 txg 起、
+/// 按这一版树表里谱系树的记录往回走——有谱系记录 (T_rb, T_old) 的那个 txg 的父是 T_old，其余 txg 的父是 txg − 1（I-9.18 那一行）。
+/// 记录按 T_rb 从大到小过一遍：T_rb 大于此刻走到的 txg 的不在这条链上（写在被后来的回退抛弃的分支上，或晚于这一版），跳过；
+/// 否则 [T_rb, 此刻] 整段在链上、跳到 T_old。最后 [0, 此刻] 也在链上。按段走是 O(L log L)，不按 txg 一步步走。
+/// T_old ≥ T_rb、同一个 T_rb 两条的谱系接不成一条链，交回 [`DeletionScanGap::LineageRecordsInconsistent`]。
+fn lineage_chain_of_a_version(
+    version_txg: u64,
+    lineage_records: &[LineageRecordOfTheScan],
+) -> Result<LineageChainOfAVersion, DeletionScanGap> {
+    let mut records_with_the_latest_rollback_first = lineage_records.to_vec();
+    records_with_the_latest_rollback_first
+        .sort_unstable_by_key(|record| std::cmp::Reverse(record.rollback_txg));
+    let some_record_is_not_younger_than_its_parent = records_with_the_latest_rollback_first
+        .iter()
+        .any(|record| record.parent_txg >= record.rollback_txg);
+    let some_rollback_has_two_records = records_with_the_latest_rollback_first
+        .windows(2)
+        .any(|adjacent| adjacent[0].rollback_txg == adjacent[1].rollback_txg);
+    if some_record_is_not_younger_than_its_parent || some_rollback_has_two_records {
+        return Err(DeletionScanGap::LineageRecordsInconsistent);
+    }
+    let mut segments_on_the_chain = Vec::new();
+    let mut txg_walked_to = version_txg;
+    // 迭代上界是谱系记录条数；跨轮携带的是此刻走到的 txg，每接上一段就跳到更小的 T_old（上面判过 T_old < T_rb ≤ 此刻），只减不增。
+    for record in &records_with_the_latest_rollback_first {
+        if record.rollback_txg > txg_walked_to {
+            continue;
+        }
+        segments_on_the_chain.push((record.rollback_txg, txg_walked_to));
+        txg_walked_to = record.parent_txg;
+    }
+    segments_on_the_chain.push((0, txg_walked_to));
+    Ok(LineageChainOfAVersion {
+        segments_on_the_chain,
+    })
+}
+
+/// 墓碑树或谱系树（根恒码 2、条目 120 字节，类型段 0 指下一层码 2 节点、类型段 = `record_type` 指码 3 容器）整棵读、不判，
+/// 按叶序交回每片容器；节点层级或条目宽不对、类型段不认得、容器的打包记录类型或记录宽不是登记的那一个都交回 `gap`
+/// （读不出、形状不对由主走读判：I-2.1、I-1.7、I-9.17、I-9.23 等，这一遍只如实记判不了——I-1.7 的坏镜像因此只红 I-1.7）。
+/// 根指针全零（零根）交回空。
+fn containers_of_a_container_tree_without_judging(
+    reader: &dyn ImageReader,
+    root_pointer: &PointerView,
+    record_type: u16,
+    gap: DeletionScanGap,
+    index_cache: &mut IndexNodeCache,
+    packed_cache: &mut PackedUnitCache,
+) -> Result<Vec<crate::PackedUnitView>, DeletionScanGap> {
+    let mut containers = Vec::new();
+    if root_pointer.all_zero {
+        return Ok(containers);
+    }
+    let root = read_index_node_without_judging(reader, root_pointer, index_cache).ok_or(gap)?;
+    collect_containers_of_a_container_tree_below(
+        reader,
+        &root,
+        record_type,
+        gap,
+        index_cache,
+        packed_cache,
+        &mut containers,
+    )?;
+    Ok(containers)
+}
+
+/// [`containers_of_a_container_tree_without_judging`] 的一个码 2 节点：条目逐条，类型段 0 的往下一层（层级 = 父层级 − 1 且 ≥ 1），
+/// 容器按条目次序追加进 `containers`。递归的上界：每下一层层级减一。
+fn collect_containers_of_a_container_tree_below(
+    reader: &dyn ImageReader,
+    node: &crate::IndexNodeView,
+    record_type: u16,
+    gap: DeletionScanGap,
+    index_cache: &mut IndexNodeCache,
+    packed_cache: &mut PackedUnitCache,
+    containers: &mut Vec<crate::PackedUnitView>,
+) -> Result<(), DeletionScanGap> {
+    if node.level == 0 || node.entry_width < inode_internal_entry_bytes() {
+        return Err(gap);
+    }
+    let registered_width = registered_record_width(record_type);
+    for entry in &node.entries {
+        let child_pointer = parse_node_pointer(&entry[34..120]);
+        match ContainerTreeEntryType::of(read_u16(entry, 16), record_type) {
+            ContainerTreeEntryType::InternalNode => {
+                if child_pointer.all_zero {
+                    return Err(gap);
+                }
+                let child = read_index_node_without_judging(reader, &child_pointer, index_cache)
+                    .ok_or(gap)?;
+                if node.level.checked_sub(1) != Some(child.level) || child.level == 0 {
+                    return Err(gap);
+                }
+                collect_containers_of_a_container_tree_below(
+                    reader,
+                    &child,
+                    record_type,
+                    gap,
+                    index_cache,
+                    packed_cache,
+                    containers,
+                )?;
+            }
+            ContainerTreeEntryType::Container => {
+                let container =
+                    read_packed_unit_without_judging(reader, &child_pointer, packed_cache)
+                        .ok_or(gap)?;
+                if container.view.record_type != record_type
+                    || Some(container.view.record_width) != registered_width
+                {
+                    return Err(gap);
+                }
+                containers.push(container.view);
+            }
+            ContainerTreeEntryType::Unregistered(_) => return Err(gap),
+        }
+    }
+    Ok(())
+}
+
+/// 一版的墓碑树按叶序读出的每片容器与记录，只读不判。
+fn tombstone_containers_without_judging(
+    reader: &dyn ImageReader,
+    root_pointer: &PointerView,
+    index_cache: &mut IndexNodeCache,
+    packed_cache: &mut PackedUnitCache,
+) -> Result<Vec<TombstoneContainerOfTheScan>, DeletionScanGap> {
+    let containers = containers_of_a_container_tree_without_judging(
+        reader,
+        root_pointer,
+        PACKED_TYPE_TOMBSTONE,
+        DeletionScanGap::TombstoneTreeOfTheVersionUnreadable,
+        index_cache,
+        packed_cache,
+    )?;
+    Ok(containers
+        .iter()
+        .map(|container| TombstoneContainerOfTheScan {
+            container: container.container,
+            records: container
+                .records
+                .iter()
+                .map(|record| TombstoneRecordOfTheScan::of(record))
+                .collect(),
+        })
+        .collect())
+}
+
+/// 一版的谱系树里的全部谱系记录，只读不判。
+fn lineage_records_without_judging(
+    reader: &dyn ImageReader,
+    root_pointer: &PointerView,
+    index_cache: &mut IndexNodeCache,
+    packed_cache: &mut PackedUnitCache,
+) -> Result<Vec<LineageRecordOfTheScan>, DeletionScanGap> {
+    let containers = containers_of_a_container_tree_without_judging(
+        reader,
+        root_pointer,
+        PACKED_TYPE_LINEAGE,
+        DeletionScanGap::LineageTreeUnreadable,
+        index_cache,
+        packed_cache,
+    )?;
+    Ok(containers
+        .iter()
+        .flat_map(|container| container.records.iter())
+        .map(|record| LineageRecordOfTheScan {
+            rollback_txg: read_u64(record, LINEAGE_RECORD_ROLLBACK_TXG_OFFSET),
+            parent_txg: read_u64(record, LINEAGE_RECORD_PARENT_TXG_OFFSET),
+        })
+        .collect())
+}
+
+/// 墓碑与谱系那几条（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节与第四节用户定案）：删除态扫描那一份版本清单里
+/// 每一版（树表与 txg 都相同的几版只扫一次——「在谱系上」从这一版自己的 txg 起算，txg 不同就各扫一遍）重读树表、谱系树、墓碑树、
+/// inode 树与 `deleted_inodes`，只读不判、不进 `walk_failures`：判 I-9.18、I-9.19、I-9.20、I-9.21 / I-9.22（看装没装旋钮）与 I-9.10 墓碑那一个等号，
+/// 把各版的号 → 对象出生代收进一张跨版本的表，全部扫完判 I-9.29。读不全、配不上的报射程里判不了（同 I-9.5 那一行的共用判法）；
+/// 不按已发布与否滤墓碑（第二节 T4）。I-9.10 墓碑那一个等号读不全、配不上时不记判不了，与它数据单元那一半同口径
+/// （C701（别的判法静默跳过读不出的部分））。
+fn judge_tombstones_and_the_lineage_of_the_versions(
+    reader: &dyn ImageReader,
+    versions: &[VersionToScanTheDeletionStateOf<'_>],
+    capacity_knob: Option<TombstoneContainerCapacityKnob>,
+    index_cache: &mut IndexNodeCache,
+    judgements: &mut Judgements,
+) {
+    let mut packed_cache = PackedUnitCache::new();
+    let mut tree_tables_and_txgs_scanned: BTreeSet<(u32, u64, u32, u64)> = BTreeSet::new();
+    let mut object_births: ObjectBirthsOfEachInodeNumber = BTreeMap::new();
+    let capacity_invariant = capacity_invariant_of(capacity_knob);
+    for version in versions {
+        let tree_table_pointer = parse_node_pointer(version.tree_table_pointer);
+        if tree_table_pointer.all_zero {
+            continue;
+        }
+        let first_location = tree_table_pointer.locations[0];
+        if !tree_tables_and_txgs_scanned.insert((
+            first_location.device,
+            first_location.slot,
+            first_location.checksum,
+            version.checkpoint_txg,
+        )) {
+            continue;
+        }
+        // 树表读不出、条目比字段表窄：这一版的墓碑与谱系判不了；树表 0 条的一版没有这几棵树，不判。
+        let tree_table = read_index_node_without_judging(reader, &tree_table_pointer, index_cache);
+        let tree_table_gap = if let Some(tree_table_read_back) = &tree_table {
+            tree_table_read_back
+                .entries
+                .iter()
+                .any(|entry| entry.len() < tree_table_entry_bytes())
+                .then_some(DeletionScanGap::TreeTableEntryShorterThanTheFieldTable)
+        } else {
+            Some(DeletionScanGap::TreeTableUnreadable)
+        };
+        if let Some(gap) = tree_table_gap {
+            for invariant in TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS
+                .into_iter()
+                .chain([capacity_invariant])
+            {
+                judgements.part_of_the_range_not_judged(invariant, gap.reason());
+            }
+            continue;
+        }
+        let Some(tree_table) = tree_table else {
+            continue;
+        };
+        if tree_table.entries.is_empty() {
+            continue;
+        }
+        judge_tombstones_and_the_lineage_of_a_version(
+            reader,
+            version,
+            &tree_table.entries,
+            capacity_knob,
+            index_cache,
+            &mut packed_cache,
+            &mut object_births,
+            judgements,
+        );
+    }
+    judge_one_object_birth_per_inode_number(&object_births, judgements);
+    // 一次都没判到时的理由（判到过的照判定报，`Judgements::not_applicable` 不改已经评估过的）。
+    judgements.not_applicable(
+        QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
+        "走读到的每一版 deleted_inodes 里都没有号：没有队列里的号可判",
+    );
+    judgements.not_applicable(
+        TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
+        "走读到的每一版里都没有在谱系上、而 inode 树里还有同号同对象出生代记录的墓碑",
+    );
+    judgements.not_applicable(
+        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
+        "走读到的每一版墓碑树都是零根：没有墓碑记录可判",
+    );
+    judgements.not_applicable(
+        ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
+        "走读到的每一版里都没有 inode 记录与谱系上的墓碑：没有号可比",
+    );
+    match capacity_knob {
+        None => {
+            judgements.not_applicable(
+                NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+                "走读到的每一版墓碑树都是零根或只一片容器：没有非最右的容器",
+            );
+            judgements.not_applicable(
+                TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
+                "池级 checker 的旧入口，当没装只供测试的墓碑容量旋钮：旋钮那一条不判",
+            );
+        }
+        Some(_) => {
+            judgements.not_applicable(
+                NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+                "这张镜像用了只供测试的墓碑容量旋钮（新入口给了旋钮值）：产品容量那一条不判",
+            );
+            judgements.not_applicable(
+                TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
+                "走读到的每一版墓碑树都是零根：没有墓碑容器可判",
+            );
+        }
+    }
+}
+
+/// [`judge_tombstones_and_the_lineage_of_the_versions`] 的一版：读谱系（父链）、每棵 inode 树的记录与每棵墓碑树，再逐条判。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "版本、树表条目、旋钮、两份缓存、跨版本那张表与判定累加器各有名字与类型，不为凑短签名收成结构体"
+)]
+fn judge_tombstones_and_the_lineage_of_a_version(
+    reader: &dyn ImageReader,
+    version: &VersionToScanTheDeletionStateOf<'_>,
+    tree_table_entries: &[Vec<u8>],
+    capacity_knob: Option<TombstoneContainerCapacityKnob>,
+    index_cache: &mut IndexNodeCache,
+    packed_cache: &mut PackedUnitCache,
+    object_births: &mut ObjectBirthsOfEachInodeNumber,
+    judgements: &mut Judgements,
+) {
+    let entries_of_kind = |kind: u16| -> Vec<&Vec<u8>> {
+        tree_table_entries
+            .iter()
+            .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == kind)
+            .collect()
+    };
+    // 一张树表恰好一条谱系树条目（I-9.16 ④）：这一版的父链从它的记录与这一版自己的 txg 起算。
+    let lineage_chain = match entries_of_kind(TREE_KIND_LINEAGE).as_slice() {
+        [lineage_entry] => lineage_records_without_judging(
+            reader,
+            &parse_node_pointer(&lineage_entry[14..100]),
+            index_cache,
+            packed_cache,
+        )
+        .and_then(|records| lineage_chain_of_a_version(version.checkpoint_txg, &records)),
+        [] | [_, _, ..] => Err(DeletionScanGap::LineageTreeEntryNotSingle),
+    };
+    let mut inode_trees: Vec<(u64, InodeRecordsOfAnInodeTreeOfTheScan)> = Vec::new();
+    for inode_tree_entry in entries_of_kind(TREE_KIND_INODE) {
+        inode_trees.push((
+            read_u64(inode_tree_entry, 0),
+            inode_records_without_judging(
+                reader,
+                &parse_node_pointer(&inode_tree_entry[14..100]),
+                index_cache,
+                packed_cache,
+            ),
+        ));
+    }
+    let mut tombstone_trees: Vec<TombstoneTreeOfTheScan> = Vec::new();
+    for tombstone_tree_entry in entries_of_kind(TREE_KIND_TOMBSTONE) {
+        tombstone_trees.push(TombstoneTreeOfTheScan {
+            tree: read_u64(tombstone_tree_entry, 0),
+            head_identifier: read_u64(
+                tombstone_tree_entry,
+                TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET,
+            ),
+            containers: tombstone_containers_without_judging(
+                reader,
+                &parse_node_pointer(&tombstone_tree_entry[14..100]),
+                index_cache,
+                packed_cache,
+            ),
+        });
+    }
+    for (inode_tree, records) in &inode_trees {
+        match records {
+            Ok(records) => {
+                for (inode, record) in records {
+                    object_births
+                        .entry((*inode_tree, *inode))
+                        .or_default()
+                        .entry(record.object_birth)
+                        .or_insert_with(|| format!("{}的 inode 记录", version.name));
+                }
+            }
+            Err(gap) => {
+                judgements
+                    .part_of_the_range_not_judged(ONE_OBJECT_BIRTH_PER_INODE_NUMBER, gap.reason());
+            }
+        }
+    }
+    for tombstone_tree in &tombstone_trees {
+        let containers = match &tombstone_tree.containers {
+            Ok(containers) => containers,
+            Err(gap) => {
+                for invariant in TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS
+                    .into_iter()
+                    .chain([capacity_invariant_of(capacity_knob)])
+                {
+                    judgements.part_of_the_range_not_judged(invariant, gap.reason());
+                }
+                continue;
+            }
+        };
+        judge_tombstone_container_capacity_of_a_version(
+            &version.name,
+            tombstone_tree.tree,
+            containers,
+            capacity_knob,
+            judgements,
+        );
+        judge_every_tombstone_of_a_version_is_well_formed(
+            version,
+            tombstone_tree.tree,
+            containers,
+            judgements,
+        );
+        match &lineage_chain {
+            Ok(lineage_chain) => judge_tombstones_on_the_lineage_of_a_version_are_distinct(
+                &version.name,
+                tombstone_tree.tree,
+                containers,
+                lineage_chain,
+                judgements,
+            ),
+            Err(gap) => judgements.part_of_the_range_not_judged(
+                TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
+                gap.reason(),
+            ),
+        }
+        // 墓碑树经头 ID 配同一张表里树 ID 等于它的那一棵 inode 树（同 extent 树，I-9.27）：配不上、或那棵树的记录读不全的，墓碑对记录那几判判不了。
+        let paired_inode_trees: Vec<&(u64, InodeRecordsOfAnInodeTreeOfTheScan)> = inode_trees
+            .iter()
+            .filter(|(inode_tree, _)| *inode_tree == tombstone_tree.head_identifier)
+            .collect();
+        let records = match paired_inode_trees.as_slice() {
+            [(_, Ok(records))] => records,
+            [(_, Err(gap))] => {
+                for invariant in [
+                    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
+                    ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
+                ] {
+                    judgements.part_of_the_range_not_judged(invariant, gap.reason());
+                }
+                continue;
+            }
+            [] | [_, _, ..] => {
+                for invariant in [
+                    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
+                    ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
+                ] {
+                    judgements.part_of_the_range_not_judged(
+                        invariant,
+                        DeletionScanGap::TombstoneTreeHeadIdentifierNamesNoSingleInodeTree.reason(),
+                    );
+                }
+                continue;
+            }
+        };
+        judge_tombstones_against_the_object_births_of_the_records_by_number(
+            &version.name,
+            containers,
+            records,
+            judgements,
+        );
+        match &lineage_chain {
+            Ok(lineage_chain) => {
+                judge_tombstones_on_the_lineage_name_no_live_object(
+                    &version.name,
+                    containers,
+                    records,
+                    lineage_chain,
+                    judgements,
+                );
+                for tombstone in containers
+                    .iter()
+                    .flat_map(|container| container.records.iter())
+                    .filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))
+                {
+                    object_births
+                        .entry((tombstone_tree.head_identifier, tombstone.object))
+                        .or_default()
+                        .entry(tombstone.object_birth)
+                        .or_insert_with(|| {
+                            format!(
+                                "{}墓碑树里谱系上的墓碑（死于 txg {}）",
+                                version.name, tombstone.death_txg
+                            )
+                        });
+                }
+            }
+            Err(gap) => {
+                for invariant in [
+                    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
+                    ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
+                ] {
+                    judgements.part_of_the_range_not_judged(invariant, gap.reason());
+                }
+            }
+        }
+    }
+    for (inode_tree, records) in &inode_trees {
+        judge_queued_inodes_of_an_inode_tree_have_tombstones_on_the_lineage(
+            reader,
+            version,
+            tree_table_entries,
+            *inode_tree,
+            records,
+            &tombstone_trees,
+            &lineage_chain,
+            index_cache,
+            judgements,
+        );
+    }
+}
+
+/// I-9.18（删除队列里的号有谱系上的墓碑）在一版的一棵 inode 树上：经头 ID 配这棵树的 `deleted_inodes`（恰好一条）与墓碑树（恰好一条），
+/// 队列里每个号 N 要在墓碑树里有 (N, N 的记录的对象出生代, 整对象) 且死亡 txg 在这一版的父链上；N 没有记录的那一形判不了、不判红
+/// （`.claude/kb/invariants.md` I-9.18 那一行：这一形只让 I-9.5 红）。队列空的一版没有对象。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "版本、树表条目、这棵 inode 树与它的记录、墓碑树、父链、缓存与判定累加器各有名字与类型，不为凑短签名收成结构体"
+)]
+fn judge_queued_inodes_of_an_inode_tree_have_tombstones_on_the_lineage(
+    reader: &dyn ImageReader,
+    version: &VersionToScanTheDeletionStateOf<'_>,
+    tree_table_entries: &[Vec<u8>],
+    inode_tree: u64,
+    records: &InodeRecordsOfAnInodeTreeOfTheScan,
+    tombstone_trees: &[TombstoneTreeOfTheScan],
+    lineage_chain: &Result<LineageChainOfAVersion, DeletionScanGap>,
+    index_cache: &mut IndexNodeCache,
+    judgements: &mut Judgements,
+) {
+    let invariant = QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE;
+    let deleted_inodes_entries: Vec<&Vec<u8>> = tree_table_entries
+        .iter()
+        .filter(|entry| {
+            read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_DELETED_INODES
+                && read_u64(entry, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET) == inode_tree
+        })
+        .collect();
+    let queue = match deleted_inodes_entries.as_slice() {
+        [deleted_inodes_entry] => deleted_inodes_queue_without_judging(
+            reader,
+            &parse_node_pointer(&deleted_inodes_entry[14..100]),
+            index_cache,
+        ),
+        [] | [_, _, ..] => Err(DeletionScanGap::InodeTreeNamedByNoSingleDeletedInodesEntry),
+    };
+    let queue = match queue {
+        Ok(queue) => queue,
+        Err(gap) => {
+            judgements.part_of_the_range_not_judged(invariant, gap.reason());
+            return;
+        }
+    };
+    if queue.is_empty() {
+        return;
+    }
+    let records = match records {
+        Ok(records) => records,
+        Err(gap) => {
+            judgements.part_of_the_range_not_judged(invariant, gap.reason());
+            return;
+        }
+    };
+    let paired_tombstone_trees: Vec<&TombstoneTreeOfTheScan> = tombstone_trees
+        .iter()
+        .filter(|tombstone_tree| tombstone_tree.head_identifier == inode_tree)
+        .collect();
+    let (tombstone_tree, containers) = match paired_tombstone_trees.as_slice() {
+        [tombstone_tree] => match &tombstone_tree.containers {
+            Ok(containers) => (tombstone_tree.tree, containers),
+            Err(gap) => {
+                judgements.part_of_the_range_not_judged(invariant, gap.reason());
+                return;
+            }
+        },
+        [] | [_, _, ..] => {
+            judgements.part_of_the_range_not_judged(
+                invariant,
+                DeletionScanGap::InodeTreeNamedByNoSingleTombstoneTreeEntry.reason(),
+            );
+            return;
+        }
+    };
+    let lineage_chain = match lineage_chain {
+        Ok(lineage_chain) => lineage_chain,
+        Err(gap) => {
+            judgements.part_of_the_range_not_judged(invariant, gap.reason());
+            return;
+        }
+    };
+    for inode in &queue {
+        let Some(record) = records.get(inode) else {
+            judgements.part_of_the_range_not_judged(
+                invariant,
+                DeletionScanGap::QueuedInodeWithoutARecord.reason(),
+            );
+            continue;
+        };
+        let tombstones_of_the_inode: Vec<TombstoneRecordOfTheScan> = containers
+            .iter()
+            .flat_map(|container| container.records.iter())
+            .filter(|tombstone| tombstone.object == *inode)
+            .copied()
+            .collect();
+        let has_a_whole_object_tombstone_on_the_lineage =
+            tombstones_of_the_inode.iter().any(|tombstone| {
+                tombstone.object_birth == record.object_birth
+                    && tombstone.covers_the_whole_object()
+                    && lineage_chain.holds_txg(tombstone.death_txg)
+            });
+        judgements.judge(invariant, has_a_whole_object_tombstone_on_the_lineage, || {
+            format!(
+                "{}：deleted_inodes 里的号 {inode}（记录的对象出生代 {}）在墓碑树（树 {tombstone_tree}）里没有在谱系上的整对象墓碑；同号的墓碑 {tombstones_of_the_inode:?}",
+                version.name, record.object_birth
+            )
+        });
+    }
+}
+
+/// I-9.10（对象出生代三处一致） 墓碑那一个等号（判决第二节 T2 取 a）：这一版墓碑树里每一条墓碑按号认，这一版 inode 树里有这个号的记录时，
+/// 墓碑的对象出生代 == 记录的。不看谱系（回退到删之前那一形的墓碑在谱系外，也比），删完之后记录不在、这一判没有对象。
+fn judge_tombstones_against_the_object_births_of_the_records_by_number(
+    version_name: &str,
+    containers: &[TombstoneContainerOfTheScan],
+    records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>,
+    judgements: &mut Judgements,
+) {
+    for tombstone in containers
+        .iter()
+        .flat_map(|container| container.records.iter())
+    {
+        let Some(record) = records.get(&tombstone.object) else {
+            continue;
+        };
+        judgements.judge("I-9.10", record.object_birth == tombstone.object_birth, || {
+            format!(
+                "{version_name}：墓碑（号 {}、对象出生代 {}、死于 txg {}）与这一版 inode 记录的对象出生代 {} 不符",
+                tombstone.object, tombstone.object_birth, tombstone.death_txg, record.object_birth
+            )
+        });
+    }
+}
+
+/// I-9.19（谱系上的墓碑不指活对象）：这一版在谱系上的每条墓碑，这一版 inode 树里有号与对象出生代都相同的记录时，那条记录 nlink = 0。
+fn judge_tombstones_on_the_lineage_name_no_live_object(
+    version_name: &str,
+    containers: &[TombstoneContainerOfTheScan],
+    records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>,
+    lineage_chain: &LineageChainOfAVersion,
+    judgements: &mut Judgements,
+) {
+    for tombstone in containers
+        .iter()
+        .flat_map(|container| container.records.iter())
+        .filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))
+    {
+        let Some(record) = records
+            .get(&tombstone.object)
+            .filter(|record| record.object_birth == tombstone.object_birth)
+        else {
+            continue;
+        };
+        judgements.judge(
+            TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
+            record.nlink == 0,
+            || {
+                format!(
+                    "{version_name}：谱系上的墓碑（号 {}、对象出生代 {}、死于 txg {}）指着 nlink {} 的记录",
+                    tombstone.object, tombstone.object_birth, tombstone.death_txg, record.nlink
+                )
+            },
+        );
+    }
+}
+
+/// I-9.20（谱系上的墓碑不重号且记录合法） 管全部墓碑的那几句（判决第二节 T14、第四节用户定案「补」）：每条整对象 [0, 2⁶⁴ − 1]
+/// （区间起 < 区间止是它的推论，照条款原文合取）、对象出生代 ≤ 死亡 txg ≤ 这一版的 txg；每片容器记录数 ≤ 815（与 I-1.7 重合，留作合取）。
+fn judge_every_tombstone_of_a_version_is_well_formed(
+    version: &VersionToScanTheDeletionStateOf<'_>,
+    tombstone_tree: u64,
+    containers: &[TombstoneContainerOfTheScan],
+    judgements: &mut Judgements,
+) {
+    let invariant = TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED;
+    let version_name = &version.name;
+    let version_txg = version.checkpoint_txg;
+    for container in containers {
+        let record_count =
+            u64::try_from(container.records.len()).expect("一片容器的记录数装得进 u64");
+        judgements.judge(invariant, record_count <= TOMBSTONE_CONTAINER_RECORDS, || {
+            format!(
+                "{version_name}：墓碑树（树 {tombstone_tree}）的容器 {} 装了 {record_count} 条，多于 {TOMBSTONE_CONTAINER_RECORDS}",
+                container.container
+            )
+        });
+        for tombstone in &container.records {
+            judgements.judge(invariant, tombstone.covers_the_whole_object(), || {
+                format!(
+                    "{version_name}：墓碑（号 {}、对象出生代 {}）的区间 [{}, {}] 不是整对象 [0, 2⁶⁴ − 1]",
+                    tombstone.object, tombstone.object_birth, tombstone.range_start, tombstone.range_end
+                )
+            });
+            let death_txg_in_range =
+                tombstone.object_birth <= tombstone.death_txg && tombstone.death_txg <= version_txg;
+            judgements.judge(invariant, death_txg_in_range, || {
+                format!(
+                    "{version_name}：墓碑（号 {}、对象出生代 {}）的死亡 txg {} 不在 [对象出生代, 这一版的 txg {version_txg}] 里",
+                    tombstone.object, tombstone.object_birth, tombstone.death_txg
+                )
+            });
+        }
+    }
+}
+
+/// I-9.20 不重号那一句（只判在谱系上的，判决第二节 T14）：这一版在谱系上的墓碑按 (号, 对象出生代) 不重复。
+fn judge_tombstones_on_the_lineage_of_a_version_are_distinct(
+    version_name: &str,
+    tombstone_tree: u64,
+    containers: &[TombstoneContainerOfTheScan],
+    lineage_chain: &LineageChainOfAVersion,
+    judgements: &mut Judgements,
+) {
+    let mut objects_on_the_lineage: BTreeSet<(u64, u64)> = BTreeSet::new();
+    for tombstone in containers
+        .iter()
+        .flat_map(|container| container.records.iter())
+        .filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))
+    {
+        let is_the_first_on_the_lineage =
+            objects_on_the_lineage.insert((tombstone.object, tombstone.object_birth));
+        judgements.judge(
+            TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
+            is_the_first_on_the_lineage,
+            || {
+                format!(
+                    "{version_name}：墓碑树（树 {tombstone_tree}）里在谱系上的墓碑 (号 {}、对象出生代 {}) 不止一条（又一条死于 txg {}）",
+                    tombstone.object, tombstone.object_birth, tombstone.death_txg
+                )
+            },
+        );
+    }
+}
+
+/// I-9.21 / I-9.22（判决第二节 T5 取 K1、T6 每版）：这一版墓碑树按叶序的容器，容器号严格递增（读容器头，不沿根的条目读）；
+/// 没装旋钮判 I-9.21——最右那片之外每片记录数 == 815，零根或只一片容器没有对象；装了旋钮判 I-9.22——每片记录数 ≤ 旋钮值，零根没有对象。
+fn judge_tombstone_container_capacity_of_a_version(
+    version_name: &str,
+    tombstone_tree: u64,
+    containers: &[TombstoneContainerOfTheScan],
+    capacity_knob: Option<TombstoneContainerCapacityKnob>,
+    judgements: &mut Judgements,
+) {
+    let invariant = capacity_invariant_of(capacity_knob);
+    let judged_when_the_tree_has_at_least = match capacity_knob {
+        None => 2,
+        Some(_) => 1,
+    };
+    if containers.len() < judged_when_the_tree_has_at_least {
+        return;
+    }
+    for adjacent in containers.windows(2) {
+        let (left, right) = (&adjacent[0], &adjacent[1]);
+        judgements.judge(invariant, left.container < right.container, || {
+            format!(
+                "{version_name}：墓碑树（树 {tombstone_tree}）沿叶序容器号不严格递增：容器 {} 之后是容器 {}",
+                left.container, right.container
+            )
+        });
+    }
+    for (position_in_leaf_order, container) in containers.iter().enumerate() {
+        let record_count =
+            u64::try_from(container.records.len()).expect("一片容器的记录数装得进 u64");
+        match capacity_knob {
+            None => {
+                let is_the_rightmost = position_in_leaf_order + 1 == containers.len();
+                if is_the_rightmost {
+                    continue;
+                }
+                judgements.judge(invariant, record_count == TOMBSTONE_CONTAINER_RECORDS, || {
+                    format!(
+                        "{version_name}：墓碑树（树 {tombstone_tree}）里不是最右的容器 {} 装了 {record_count} 条，不是 {TOMBSTONE_CONTAINER_RECORDS}（满了才关）",
+                        container.container
+                    )
+                });
+            }
+            Some(knob) => {
+                let knob_records = knob.records_per_container();
+                judgements.judge(invariant, record_count <= knob_records, || {
+                    format!(
+                        "{version_name}：墓碑树（树 {tombstone_tree}）的容器 {} 装了 {record_count} 条，多于旋钮值 {knob_records}",
+                        container.container
+                    )
+                });
+            }
+        }
+    }
+}
+
+/// I-9.29（同号只许一个出生代）：各版的 inode 记录与各版自己谱系上的墓碑合起来，同一棵 inode 树里每个号只对应一个对象出生代
+/// （判决第二节 T10 乙）。每个号判一次。
+fn judge_one_object_birth_per_inode_number(
+    object_births: &ObjectBirthsOfEachInodeNumber,
+    judgements: &mut Judgements,
+) {
+    for ((inode_tree, inode), births) in object_births {
+        judgements.judge(ONE_OBJECT_BIRTH_PER_INODE_NUMBER, births.len() == 1, || {
+            let where_each_birth_was_seen: Vec<String> = births
+                .iter()
+                .map(|(object_birth, first_seen)| format!("对象出生代 {object_birth}（{first_seen}）"))
+                .collect();
+            format!(
+                "inode 树（树 {inode_tree}）里 inode {inode} 在回退候选集里对应不止一个对象出生代：{}",
+                where_each_birth_was_seen.join("、")
+            )
+        });
+    }
+}
+
+/// 池级 checker 的入口：每条第一版不变量都报，没评估到的报「不适用」并带理由。当没装只供测试的墓碑容量旋钮（产品镜像）：
+/// I-9.21（产品镜像里非最右墓碑容器装满） 照产品容量判，I-9.22（测试镜像里墓碑容器不超旋钮值） 报不适用。
+#[must_use]
+pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {
+    check_pool_image_given_the_tombstone_container_capacity_knob(reader, None)
+}
+
+/// 池级 checker 带旋钮值的入口（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T5 取 K1）：旋钮不落盘
+/// （D18（块里携带什么信息） 已定项 24 第 3 条），写这张镜像时压没压墓碑容器容量由调用方从镜像外给。`None` 与 [`check_pool_image`] 相同；
+/// `Some` 时 I-9.22 按旋钮值判、I-9.21 报不适用。
+#[must_use]
+pub fn check_pool_image_given_the_tombstone_container_capacity_knob(
+    reader: &dyn ImageReader,
+    tombstone_container_capacity_knob: Option<TombstoneContainerCapacityKnob>,
+) -> Vec<(&'static str, InvariantVerdict)> {
+    let system_configurations = chosen_system_configurations(reader);
+    let mut root_ring_judgements = Judgements::default();
+    let chosen: Vec<_> = system_configurations
+        .iter()
+        .filter_map(|(device, chosen)| {
+            chosen
+                .clone()
+                .map(|(view, geometry)| (*device, view, geometry))
+        })
+        .collect();
+    // 一块盘的两个系统配置槽都无效时**不早退**：恢复会改用别的盘上那一份、照常挂上
+    // （`crates/singlefs-core/src/recovery.rs` 的 `choose_system_configuration`，D22（单元原子性怎么合成） 已定项 8
+    // 「系统配置每盘放一份」买的就是这份冗余）。早退会让这种镜像上每一条不变量都报「不适用」，
+    // 而它是一个挂得上的合法镜像——故障注入让一块盘的两个槽先后写失败就造得出它，判定会被静默放过
+    // （.claude/kb/checks-owed.md 的 C461）。⚠️ 「哪块盘的系统配置全废了」今天没有编号报得出来，仍欠在 C461。
+    // 任一盘任一槽自证过而带这一版读者不收的池级值（格式版本、加密类型、槽距、physical_block_size、环长、journal 环起点、
+    // 根环起点、单元区起始槽号）：实现整池拒绝挂载
+    // （`RecoveryFailure::SystemConfigurationValueRefused`），checker 报违例、别的不变量同「挂不上的镜像」一样不作保
+    // （用户 2026-09-27 定系统配置越界整池拒，实审 A3-checker-2）。只有一部分槽带越界值时不拿别的槽照判下去。
+    let system_configuration_slot_readings_of_each_device =
+        system_configuration_slot_readings(reader);
+    let any_system_configuration_slot_carries_a_refused_value =
+        judge_system_configuration_values_the_reader_accepts(
+            &system_configuration_slot_readings_of_each_device,
+            &mut root_ring_judgements,
+        );
+    if any_system_configuration_slot_carries_a_refused_value {
+        for invariant in crate::image::IMPLEMENTED_INVARIANTS {
+            root_ring_judgements.not_applicable(
+                invariant,
+                "池里有一槽自证过的系统配置带这一版读者不收的池级值：实现整池拒绝挂载，这不是一个挂得上的镜像",
+            );
+        }
+        return root_ring_judgements.into_report();
+    }
+    // I-8.1 的三个字段等式（在飞上限、最坏占用、F = 格式常量）任一盘任一可择槽有一格不等：实现择系统配置时同判、整池拒绝挂载
+    // （`RecoveryFailure::SystemConfigurationValueRefused`），checker 与上面 I-7.13 那一支同口径——报 I-8.1 违例、别的不变量一律不作保
+    // （`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 4 行）。三格先判进一份只装它们的判定里：整池不作保时
+    // 别的不变量（连同已判过成立的 I-7.13）都报不适用；三格都成立时再判进这一份，照常往下判。
+    let mut judgements_of_the_journal_ring_geometry_fields_alone = Judgements::default();
+    judge_journal_ring_geometry_fields_agree(
+        &system_configuration_slot_readings_of_each_device,
+        &mut judgements_of_the_journal_ring_geometry_fields_alone,
+    );
+    if judgements_of_the_journal_ring_geometry_fields_alone
+        .violation_count(JOURNAL_RING_GEOMETRY_FIELDS_AGREE)
+        > 0
+    {
+        for invariant in crate::image::IMPLEMENTED_INVARIANTS {
+            judgements_of_the_journal_ring_geometry_fields_alone.not_applicable(
+                invariant,
+                "池里有一槽可择的系统配置 journal 环几何三个字段（在飞上限、最坏占用、F）有一格不等：实现整池拒绝挂载，这不是一个挂得上的镜像",
+            );
+        }
+        return judgements_of_the_journal_ring_geometry_fields_alone.into_report();
+    }
+    judge_journal_ring_geometry_fields_agree(
+        &system_configuration_slot_readings_of_each_device,
+        &mut root_ring_judgements,
+    );
+    // I-8.5（V-池）：全池皆废判红；一块盘交得出可择的系统配置就照常往下判（单盘全废走旁路报告，不进违例通道）。
+    // 带越界值的槽、环几何三格不等的槽整池拒挂，上面那两支整池报不适用、不判这一条。
+    // 一块盘都交不出可择的系统配置、而有槽自证过（incompat 位不认得、R / S 越界这类自证过而不可择的槽）：这不是一个挂得上的镜像，
+    // I-8.5 同别的不变量一起报不适用，不判成成立（`InvariantVerdict` 的「不适用必须带理由，不许静默算成立」）；一个槽都没自证过的照判红。
+    if chosen.is_empty() {
+        let mut judgement_of_some_device_carrying_a_self_verified_slot = Judgements::default();
+        judge_some_device_carries_a_self_verified_system_configuration(
+            &system_configuration_slot_readings_of_each_device,
+            &mut judgement_of_some_device_carrying_a_self_verified_slot,
+        );
+        match judgement_of_some_device_carrying_a_self_verified_slot
+            .first_violation_of(SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION)
+        {
+            Some(detail) => {
+                let detail = detail.to_owned();
+                root_ring_judgements.judge(
+                    SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
+                    false,
+                    || detail,
+                );
             }
             None => root_ring_judgements.not_applicable(
                 SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
@@ -9378,6 +10502,7 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
                 let root = &roots[*index].2;
                 VersionToScanTheDeletionStateOf {
                     name: format!("根（实例 {}、txg {}）", root.instance, root.checkpoint_txg),
+                    checkpoint_txg: root.checkpoint_txg,
                     tree_table_pointer: &root.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],
                 }
             })
@@ -9387,6 +10512,7 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
                         "由记录施加出来的那一版（实例 {}、txg {}）",
                         version.instance, version.checkpoint_txg
                     ),
+                    checkpoint_txg: version.checkpoint_txg,
                     tree_table_pointer: version.tree_table_and_mapping_root_pointers().0,
                 }
             }))
@@ -9397,6 +10523,23 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
         &mut index_node_cache,
         &mut judgements,
     );
+    // 墓碑与谱系那几条（I-9.18–I-9.22、I-9.29、I-9.10 墓碑那一个等号）：同一份版本清单，每一版从自己的 txg 与自己的谱系树起算。
+    judge_tombstones_and_the_lineage_of_the_versions(
+        reader,
+        &versions_to_scan_the_deletion_state_of,
+        tombstone_container_capacity_knob,
+        &mut index_node_cache,
+        &mut judgements,
+    );
+    // I-9.17、I-9.23 在主走读里每条内部条目判；一条条目都没走到时的理由。
+    judgements.not_applicable(
+        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        "走读到的每一版谱系树都是零根：没有谱系树内部条目可判",
+    );
+    judgements.not_applicable(
+        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        "走读到的每一版墓碑树都是零根：没有墓碑树内部条目可判",
+    );
     // I-4.4（意图只描述目标态）：第一版的意图载体没有可装增量的字段，报不适用并写理由（用户 2026-10-05 定）。
     judgements.not_applicable(
         INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
diff --git a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
index 685ccdd..2f3dcff 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
@@ -59,7 +59,7 @@ const _: () = assert!(
     "违例位读成一个 u128"
 );
 
-/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 74 条——三批 49 条，加里程碑三二·1 那一批 16 条
+/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 83 条——三批 49 条，加里程碑三二·1 那一批 16 条
 /// （判决 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行），下标 65 的 I-3.5（引用区间的精确性，
 /// 走读内核在回退候选集的版本都走完之后逐对判），再加 C673（三 oracle 合谋测试的补 oracle 改法没做）⑤ ⑥ 那两条
 /// （下标 66 是 I-9.27，走读内核挨着 I-9.16 按表判；下标 67 是 I-5.5，扫描内核每一版重读树表与 extent 树判），
@@ -68,11 +68,15 @@ const _: () = assert!(
 /// 72 的 I-4.6 由扫描内核的删除态扫描在回退候选集的每一版上判（判哪几版、经头 ID 配对、读不全不置位，照 `singlefs_checker::walk` 的
 /// `judge_deletion_state_of_the_versions`），73 的 I-4.4 第一版池级 checker 恒报不适用、内核不置位；I-9.6 墓碑那一半（水位 > 所选根墓碑树里
 /// 已发布墓碑的最大对象 ID）也在扫描内核里判、并进下标 41 那一位。对拍逐位比、不屏蔽。
+/// 下标 74–82 是里程碑三·3 墓碑与谱系那九条（I-9.17、I-9.18、I-9.19、I-9.20、I-9.21、I-9.22、I-9.23、I-9.24、I-9.29，第 22 批）：
+/// 照下标 73 的先例先登记、内核这一批不置这九位，GPU 侧第 23 批补（走读内核判 I-9.17、I-9.23，扫描内核判其余七条）；
+/// 补上之前 CPU 判红而 GPU 不置位的状态在对拍里会对不上。
 /// 用户 2026-10-04 定「改回要求全覆盖」：没有「内核还没判」那一张表。
 pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[
     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
     26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
     50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
+    74, 75, 76, 77, 78, 79, 80, 81, 82,
 ];
 
 // 覆盖表就是池级 checker 的每一条，按下标升序：第 k 项是 k。新立一条不变量而内核没跟上时这里编译不过，不悄悄让 GPU 那一位恒 0。
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index d2fa7be..b954dc0 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -15,21 +15,27 @@ use common::{
 };
 use singlefs_checker::image::{
     InvariantVerdict, ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
-    DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
+    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY, DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
     FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
     HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
     INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
     INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
     INTENT_DESCRIBES_ONLY_THE_TARGET_STATE, INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION,
     JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
-    LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH, NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
-    POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER, PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
+    LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
+    LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER, NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
+    NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+    ONE_OBJECT_BIRTH_PER_INODE_NUMBER, POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
+    PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
     REFERENCE_INTERVALS_ARE_EXACT, ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES,
     ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
     SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
     SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
     SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
     SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION,
+    TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
+    TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
+    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
     TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
     TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
     UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
@@ -1841,10 +1847,61 @@ fn expected_verdict_with_one_record_per_transaction(invariant: &str) -> Invarian
         "I-4.7" => InvariantVerdict::NotApplicable(NO_INODE_BEING_DELETED_IN_ANY_WALKED_VERSION),
         "I-4.6" => InvariantVerdict::NotApplicable(NO_RECORD_OF_NLINK_ZERO_IN_ANY_WALKED_VERSION),
         "I-4.4" => InvariantVerdict::NotApplicable(INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION),
+        other_invariant => {
+            expected_verdict_of_the_tombstone_and_lineage_invariants_without_deletion(
+                other_invariant,
+            )
+        }
+    }
+}
+
+/// 没有删除、没有回退的镜像上墓碑与谱系那九条（里程碑三·3，判决 `research/prompts/m3-delete-checker-r2-main-verification.md`）的判定：
+/// 墓碑树、谱系树都是零根，I-9.17–I-9.23 没有对象、报不适用带理由（I-9.22 是旧入口当没装旋钮）；I-9.24（队列空的一版报成立）与
+/// I-9.29（inode 记录各号只一个出生代）真被评估过且成立。别的不变量交回成立。
+fn expected_verdict_of_the_tombstone_and_lineage_invariants_without_deletion(
+    invariant: &str,
+) -> InvariantVerdict {
+    match invariant {
+        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER => {
+            InvariantVerdict::NotApplicable(NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION)
+        }
+        QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE => {
+            InvariantVerdict::NotApplicable(NO_QUEUED_INODE_IN_ANY_WALKED_VERSION)
+        }
+        TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT => {
+            InvariantVerdict::NotApplicable(NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD)
+        }
+        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED => {
+            InvariantVerdict::NotApplicable(NO_TOMBSTONE_IN_ANY_WALKED_VERSION)
+        }
+        NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE => {
+            InvariantVerdict::NotApplicable(NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER)
+        }
+        TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE => {
+            InvariantVerdict::NotApplicable(NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY)
+        }
+        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER => {
+            InvariantVerdict::NotApplicable(NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION)
+        }
         _ => InvariantVerdict::Holds,
     }
 }
 
+/// 墓碑与谱系那几条没有对象时报的理由（`walk` 的 `judge_tombstones_and_the_lineage_of_the_versions` 与 `check_pool_image` 末尾）。
+const NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
+    "走读到的每一版谱系树都是零根：没有谱系树内部条目可判";
+const NO_QUEUED_INODE_IN_ANY_WALKED_VERSION: &str =
+    "走读到的每一版 deleted_inodes 里都没有号：没有队列里的号可判";
+const NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD: &str =
+    "走读到的每一版里都没有在谱系上、而 inode 树里还有同号同对象出生代记录的墓碑";
+const NO_TOMBSTONE_IN_ANY_WALKED_VERSION: &str = "走读到的每一版墓碑树都是零根：没有墓碑记录可判";
+const NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER: &str =
+    "走读到的每一版墓碑树都是零根或只一片容器：没有非最右的容器";
+const NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY: &str =
+    "池级 checker 的旧入口，当没装只供测试的墓碑容量旋钮：旋钮那一条不判";
+const NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
+    "走读到的每一版墓碑树都是零根：没有墓碑树内部条目可判";
+
 /// 没抬过 F 的镜像上 I-7.9（回退下界 F 不高于抬 F 的上限） 报的不适用理由。
 const NO_ROOT_RAISED_THE_FLOOR: &str = "根环里没有抬 F 的根：每条根带的 F 都不高于同一实例里它前一条根带的（回退与新实例的第一条根不算抬）";
 
@@ -2919,6 +2976,47 @@ fn devices_whose_own_device_numbers_are_swapped_redden_only_the_own_device_numbe
     );
 }
 
+/// 墓碑与谱系那九条（第 22 批）的坏镜像各要一份自己的基底（意图之后、根环转过一圈之后、回退之后、旋钮压小写出的），不进上面那张
+/// 只改干净镜像一处的表；这里登记每一条的坏镜像在哪个用例里（那几个用例各自断言只红它自己）。
+const TOMBSTONE_AND_LINEAGE_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE: [(&str, &str); 9] = [
+    (
+        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        "checker_known_bad_images.rs: a_lineage_tree_entry_whose_identity_names_another_container_reddens_only_the_lineage_entry_invariant",
+    ),
+    (
+        QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
+        "pool_checker_judges_the_deletion_state_of_every_walked_version.rs: a_queued_inode_whose_tombstone_died_on_a_branch_a_rollback_left_reddens_only_the_queued_tombstone_invariant",
+    ),
+    (
+        TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
+        "pool_checker_judges_the_deletion_state_of_every_walked_version.rs: a_tombstone_on_the_lineage_naming_a_live_inode_reddens_only_the_live_object_invariant",
+    ),
+    (
+        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
+        "pool_checker_judges_the_deletion_state_of_every_walked_version.rs: a_tombstone_that_does_not_cover_the_whole_object_reddens_only_the_tombstone_record_invariant",
+    ),
+    (
+        NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+        "container_trees_grow_a_level_when_the_root_is_full.rs: a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant",
+    ),
+    (
+        TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
+        "container_trees_grow_a_level_when_the_root_is_full.rs: a_tombstone_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant",
+    ),
+    (
+        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        "checker_known_bad_images.rs: a_tombstone_tree_entry_whose_identity_names_another_container_reddens_only_the_tombstone_entry_invariant",
+    ),
+    (
+        DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
+        "pool_checker_judges_the_deletion_state_of_every_walked_version.rs: a_deletion_queue_of_two_inodes_being_deleted_with_their_tombstones_reddens_only_the_at_most_one_key_invariant",
+    ),
+    (
+        ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
+        "inode_number_watermark_pressed_back_reissues_a_number.rs: pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_only_the_one_birth_per_number_invariant",
+    ),
+];
+
 /// 清单里不要求坏镜像的几条：I-4.4（意图只描述目标态） 第一版报不适用（用户 2026-10-05 定，`.claude/kb/milestone/03-third-txn.md` 三·3 那一行
 /// 「I-4.4 不要求坏镜像与变异行」）。
 const INVARIANTS_WITHOUT_A_BAD_IMAGE: [&str; 1] = [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE];
@@ -2975,7 +3073,13 @@ fn the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target()
             "I-4.4" => {
                 InvariantVerdict::NotApplicable(INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION)
             }
-            _ => InvariantVerdict::Holds,
+            // 墓碑与谱系那九条：这一份上没有删除、没有回退，墓碑树与谱系树都是零根。坏镜像见
+            // `TOMBSTONE_AND_LINEAGE_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE`。
+            other_invariant => {
+                expected_verdict_of_the_tombstone_and_lineage_invariants_without_deletion(
+                    other_invariant,
+                )
+            }
         };
         assert_eq!(found, expected, "干净镜像上 {invariant} 的判定");
     }
@@ -3012,6 +3116,7 @@ fn the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target()
         .chain(known_bad_images_of_the_tail_slot_witnesses().iter())
         .chain(known_bad_images_of_the_replayed_version().iter())
         .chain(common_deletion_in_progress::known_bad_images_after_the_intent_of_a_three_unit_file().iter())
+        .chain(common_deletion_in_progress::known_bad_images_after_the_intent_of_an_empty_inode().iter())
         .chain(
             common_deletion_in_progress::known_bad_images_after_the_first_batch_of_a_three_unit_file()
                 .iter(),
@@ -3019,6 +3124,11 @@ fn the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target()
         .chain(common_deletion_in_progress::known_bad_images_after_one_new_inode().iter())
         .chain(common_deletion_in_progress::known_bad_images_of_the_pending_deletion_occupancy().iter())
         .map(|(invariant, _)| *invariant)
+        .chain(
+            TOMBSTONE_AND_LINEAGE_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE
+                .iter()
+                .map(|(invariant, _)| *invariant),
+        )
         .collect();
     let listed: std::collections::BTreeSet<&str> = singlefs_checker::image::IMPLEMENTED_INVARIANTS
         .iter()
@@ -3664,7 +3774,12 @@ fn birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollbac
     for (invariant, found) in check_pool_image(&rolled_back.image) {
         assert_eq!(
             found,
-            expected_verdict_with_one_record_per_transaction(invariant),
+            if invariant == LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER {
+                // 回退那次发布写出谱系树（一片容器、一条谱系记录）：它的根条目真被评估过且成立。
+                InvariantVerdict::Holds
+            } else {
+                expected_verdict_with_one_record_per_transaction(invariant)
+            },
             "回退到 A 之后的干净镜像上 {invariant} 的判定"
         );
     }
@@ -8968,3 +9083,195 @@ fn a_file_version_whose_root_record_carries_a_non_zero_allocation_record_tree_ro
         "红在最新那条根"
     );
 }
+
+/// 码 2 节点头里的层级（偏移 50）与 key 宽（偏移 51），条目区从 115 + 2 × key 宽 起（D18（块里携带什么信息） 已定项 18）；
+/// 墓碑树、谱系树的根条目里类型段在条目偏移 16、身份引用的容器号在 18（分隔 key 8 + 出生树 8 + 类型段 2 之后，D8（核心索引结构） 已定项 6 的条目形态）。
+const INDEX_NODE_LEVEL_OFFSET: usize = 50;
+const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
+const CONTAINER_TREE_ENTRY_TYPE_OFFSET: usize = 16;
+const CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET: usize = 18;
+/// 墓碑树、谱系树根条目类型段里条款之外的一个值（I-9.23 登记 {0, 1}、I-9.17 登记 {0, 5}）。
+const CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED: u16 = 7;
+
+/// 最新的根那一版树表里种类 `kind` 那棵容器树（墓碑树、谱系树）的码 2 根第 0 条条目改一处（`change` 拿到那 120 字节），
+/// 根重封、新的整单元校验和沿引用链补到根槽。这几份镜像上这棵树的根是层级 1、直接管容器。
+fn change_the_first_entry_of_the_container_tree_root_of_kind(
+    image: &mut MemoryPool,
+    kind: u16,
+    change: impl FnOnce(&mut [u8]),
+) {
+    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
+    let root_slot = common_pointer_head_corruption::the_slot_of(&root_pointer);
+    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
+        image,
+        root_slot,
+        common_pointer_head_corruption::node_bytes(),
+        |root| {
+            assert_eq!(
+                root[INDEX_NODE_LEVEL_OFFSET], 1,
+                "种类 {kind} 那棵容器树的根直接管容器"
+            );
+            let entries_start = 115 + 2 * usize::from(root[INDEX_NODE_KEY_WIDTH_OFFSET]);
+            change(
+                &mut root[entries_start
+                    ..entries_start
+                        + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
+                        + CONTAINER_TREE_ENTRY_CHILD_POINTER_BYTES],
+            );
+        },
+    );
+}
+
+/// 一条容器树根条目里身份引用的容器号加一（容器头不动）。
+fn add_one_to_the_identity_container_number(entry: &mut [u8]) {
+    let field = CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET;
+    let container = get_u64(entry, field);
+    set_u64(entry, field, container + 1);
+}
+
+/// I-9.23（墓碑树条目身份与子头相符） 只红自己的那一形（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T7 取 a）：
+/// 三个单元的文件发完意图之后，墓碑树根第一条条目身份引用里的容器号加一、容器头不动。身份不符单独判定、不再记走读失败：
+/// 改之前红的是 I-4.8、I-7.2、I-7.4（最新根走不完），现在只红 I-9.23。
+#[test]
+fn a_tombstone_tree_entry_whose_identity_names_another_container_reddens_only_the_tombstone_entry_invariant(
+) {
+    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+        "known-bad-tombstone-entry-identity",
+    );
+    assert_eq!(
+        verdict(&clean, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER),
+        InvariantVerdict::Holds,
+        "意图之后的干净镜像上 I-9.23 真被评估过且成立"
+    );
+    let mut image = clean.clone();
+    change_the_first_entry_of_the_container_tree_root_of_kind(
+        &mut image,
+        TREE_KIND_TOMBSTONE,
+        add_one_to_the_identity_container_number,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
+        "墓碑树根条目的身份与容器头不符：只红 I-9.23，不连带 I-4.8、I-7.2、I-7.4：{verdicts:?}"
+    );
+}
+
+/// I-9.23 类型段不认得那一支：墓碑树根第一条条目的类型段 1 → 7。孩子先登记引用再判红（判决第二节 T7「类型段不认得时先登记引用再判红」）：
+/// 不登记的话那片容器占的槽不进引用并集，I-3.1（已分配统计对得上） 连带红。只红 I-9.23。
+#[test]
+fn a_tombstone_tree_entry_with_an_unregistered_type_reddens_only_the_tombstone_entry_invariant() {
+    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+        "known-bad-tombstone-entry-unregistered-type",
+    );
+    let mut image = clean.clone();
+    change_the_first_entry_of_the_container_tree_root_of_kind(
+        &mut image,
+        TREE_KIND_TOMBSTONE,
+        |entry| {
+            assert_eq!(
+                u16::from_le_bytes([
+                    entry[CONTAINER_TREE_ENTRY_TYPE_OFFSET],
+                    entry[CONTAINER_TREE_ENTRY_TYPE_OFFSET + 1]
+                ]),
+                1,
+                "墓碑树根条目的类型段是 1"
+            );
+            set_u16(
+                entry,
+                CONTAINER_TREE_ENTRY_TYPE_OFFSET,
+                CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
+            );
+        },
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
+        "墓碑树根条目的类型段不认得：只红 I-9.23，不连带 I-3.1：{verdicts:?}"
+    );
+}
+
+/// I-9.17（谱系树条目身份与子头相符） 只红自己的那一形：挂着的时候回退到 A 之后（回退那次发布写出谱系树），谱系树根第一条条目身份引用里的
+/// 容器号加一、容器头不动。
+#[test]
+fn a_lineage_tree_entry_whose_identity_names_another_container_reddens_only_the_lineage_entry_invariant(
+) {
+    let rolled_back =
+        image_after_rolling_back_to_the_first_root("known-bad-lineage-entry-identity");
+    let mut image = rolled_back.image.clone();
+    change_the_first_entry_of_the_container_tree_root_of_kind(
+        &mut image,
+        TREE_KIND_LINEAGE,
+        add_one_to_the_identity_container_number,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
+        "谱系树根条目的身份与容器头不符：只红 I-9.17：{verdicts:?}"
+    );
+}
+
+/// I-1.7（打包容器合法与判定顺序） 记录宽那一步对类型 1（墓碑，登记 40）：三个单元的文件发完意图之后，墓碑容器头里的记录宽 40 → 39。
+/// 墓碑与谱系那一遍、I-9.6 墓碑那一半读容器时遇到记录宽不是登记的那一个，那一版判不了、不判红：只红 I-1.7。
+#[test]
+fn a_tombstone_container_whose_record_width_is_not_forty_reddens_only_the_packed_container_invariant(
+) {
+    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+        "known-bad-tombstone-record-width",
+    );
+    let mut image = clean.clone();
+    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
+        &mut image,
+        TREE_KIND_TOMBSTONE,
+        39,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-1.7"],
+        "墓碑容器的记录宽不是 40：只红 I-1.7：{verdicts:?}"
+    );
+}
+
+/// I-1.7 记录宽那一步对类型 5（谱系，登记 16）：回退到 A 之后，谱系容器头里的记录宽 16 → 15。只红 I-1.7。
+#[test]
+fn a_lineage_container_whose_record_width_is_not_sixteen_reddens_only_the_packed_container_invariant(
+) {
+    let rolled_back = image_after_rolling_back_to_the_first_root("known-bad-lineage-record-width");
+    let mut image = rolled_back.image.clone();
+    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
+        &mut image,
+        TREE_KIND_LINEAGE,
+        15,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-1.7"],
+        "谱系容器的记录宽不是 16：只红 I-1.7：{verdicts:?}"
+    );
+}
+
+/// I-9.4（容器号不超最小 key） 第四句（同一片打包记录类型 2 容器里记录号沿记录序严格递增，判决第二节 T11 取甲）只红自己的那一形
+/// （S4a：`research/prompts/m3-batch20-code-r1-main-verification.md` 第二节）：第一个文件之后建一个空 inode 的那一版，叶 [1, 2] 改成
+/// [1 nlink 0, 1 nlink 1]、队列空。改之前一条都不红。
+#[test]
+fn a_leaf_holding_the_same_inode_number_twice_reddens_only_the_container_number_invariant() {
+    let clean =
+        common_deletion_in_progress::pool_with_new_inodes("known-bad-leaf-same-inode-twice", 1)
+            .memory_pool();
+    let mut image = clean.clone();
+    common_deletion_in_progress::turn_the_leaf_into_two_records_of_the_first_file(&mut image);
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-9.4"],
+        "同一片叶里 1 号两条：只红 I-9.4：{verdicts:?}"
+    );
+    assert!(
+        matches!(verdict(&image, "I-9.4"), InvariantVerdict::Violated(detail) if detail.contains("inode 1 之后是 inode 1")),
+        "违例说明写出不严格递增的那一对"
+    );
+}
diff --git a/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs b/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs
index cfb9389..8eeea43 100644
--- a/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs
+++ b/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs
@@ -1,4 +1,5 @@
-//! 删除中间态的搭建与池级 checker 删除线（I-9.5、I-3.4、I-4.7、I-4.6、I-9.6 墓碑那一半）的坏镜像：
+//! 删除中间态的搭建与池级 checker 删除线（I-9.5、I-3.4、I-4.7、I-4.6、I-9.6 墓碑那一半，墓碑与谱系那几条 I-9.18–I-9.20、I-9.24、
+//! I-9.10 墓碑那一个等号）的坏镜像：
 //! 新池新建文件之后把第一个文件顺序写成三个数据单元、按只供测试的批大小 1 发意图与几批（D8（核心索引结构） 已定项 16），
 //! 或建几个空 inode；坏镜像在发布路径写出的一版上找到要改的单元（从最新的根读树表往下找），改一处、重封，
 //! 新的整单元校验和沿引用链补到根槽（`common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum`）。
@@ -12,16 +13,18 @@ use singlefs_checker::image::{
     PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
 };
 use singlefs_checker::walk::check_pool_image;
-use singlefs_core::address::{InodeNumber, InstanceGeneration, SlotNumber};
+use singlefs_core::address::{CheckpointTxg, InodeNumber, InstanceGeneration, SlotNumber};
 use singlefs_core::bytes::ByteReader;
 use singlefs_core::checksum::crc32_castagnoli;
+use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
 use singlefs_core::pointer::NodePointer;
 use singlefs_core::records::{
     TREE_KIND_ACCOUNTING, TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE,
     TREE_KIND_LIVELIST, TREE_KIND_TOMBSTONE,
 };
 use singlefs_core::transaction::{
-    publish_new_inodes, publish_unlink_intent, publish_unlink_step, PoolWriter, TransactionOutput,
+    publish_new_inodes, publish_unlink_intent, publish_unlink_step,
+    publish_unlink_steps_until_the_queue_is_empty, PoolWriter, TransactionOutput,
     FIRST_INODE_NUMBER,
 };
 use singlefs_core::unit::{data_unit_payload_capacity, parse_index_node, seal_header_checksum};
@@ -97,8 +100,18 @@ fn index_node_largest_key_offset(key_width: usize) -> usize {
 }
 /// 码 3 单元头里写序的实例代号（4 字节，偏移 93）：已发布谓词按它与诞生代号判（I-1.2）。
 const PACKED_UNIT_WRITE_ORDER_INSTANCE_OFFSET: usize = 93;
-/// 墓碑记录 40 字节，对象 ID 在 0（D18（块里携带什么信息） 已定项 24）。
+/// 墓碑记录 40 字节：对象 ID 0、对象出生代 8、区间起 16、区间止 24、死亡 txg 32（D18（块里携带什么信息） 已定项 24 第 2 条）。
+const TOMBSTONE_RECORD_BYTES: usize = 40;
 const TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET: usize = 0;
+pub const TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET: usize = 8;
+pub const TOMBSTONE_RECORD_RANGE_END_OFFSET: usize = 24;
+pub const TOMBSTONE_RECORD_DEATH_TXG_OFFSET: usize = 32;
+/// 码 3 单元头里记录宽（2 字节，偏移 71，D18（块里携带什么信息） 已定项 16）：I-1.7 比它与登记表的宽。
+pub const PACKED_UNIT_RECORD_WIDTH_OFFSET: usize = 71;
+/// 根环一圈的根槽数（三个区域、每区 `RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM` 8 槽，`common::parameters`）再多几次：
+/// 发这么多次之后，更早的版本都不在回退候选集里（根槽被盖掉）。攻方 `research/prompts/m3-delete-checker-r2-opus-output.md` 第五节
+/// 「再建 30 次」那一造法：坏镜像要改的墓碑容器被意图那一版共用时，改它会让意图那一版连带别的条红，先让那一版转出候选集再改。
+pub const PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER: u64 = 30;
 
 pub fn batches_of_one_data_unit() -> UnlinkBatchSize {
     UnlinkBatchSize::CappedForTests(
@@ -223,6 +236,118 @@ pub fn pool_with_a_first_file_of_size_zero_and_one_data_unit(tag: &str) -> Built
     pool
 }
 
+/// 在池上发 `inode` 的 unlink 意图（意图那一次：记录 nlink 写成 0、队列加这个号、墓碑树开放容器追加一条），交回这一次发布。
+pub fn publish_the_intent_of(pool: &mut BuiltPool, inode: u64) -> TransactionOutput {
+    let publish_parameters = parameters();
+    let previous = pool.output.clone();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+    let output = publish_unlink_intent(
+        &mut writer,
+        &mut pool.allocator,
+        &previous,
+        InodeNumber(inode),
+        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
+        INSTANCE_OF_THE_PUBLISHES,
+    )
+    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
+    pool.output = output.clone();
+    output
+}
+
+/// 把 `inode` unlink 到底：意图那一次（交回它），再按产品批大小发到删除队列空（记录删掉，墓碑留在墓碑树里）。
+pub fn unlink_to_the_end(pool: &mut BuiltPool, inode: u64) -> TransactionOutput {
+    let intent = publish_the_intent_of(pool, inode);
+    let publish_parameters = parameters();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+    let mut current = pool.output.clone();
+    publish_unlink_steps_until_the_queue_is_empty(
+        &mut writer,
+        &mut pool.allocator,
+        &mut current,
+        UnlinkBatchSize::OneExtentLowerLeaf,
+        INSTANCE_OF_THE_PUBLISHES,
+    )
+    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的各批：{refusal:?}"));
+    pool.output = current;
+    intent
+}
+
+/// 挂着的时候回退到 `target` 那一版（一次向前发布：新根的 inode 树、extent 树、`deleted_inodes` 取 `target` 的，墓碑树随现行那一版，
+/// 谱系树追加 (这次发布的 txg, `target` 的 txg)，D23（journal 的角色与格式） 已定项 14、D18（块里携带什么信息） 已定项 24 第 5 条）。
+/// 交回回退那次发布。
+pub fn roll_back_to(pool: &mut BuiltPool, target: &TransactionOutput) -> TransactionOutput {
+    let publish_parameters = parameters();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    roll_back_by_a_forward_publish(
+        &publish_parameters,
+        devices,
+        &mut pool.allocator,
+        &mut pool.output,
+        RollbackTarget {
+            instance: target.root.instance,
+            checkpoint_txg: target.root.checkpoint_txg,
+        },
+    )
+    .unwrap_or_else(|refusal| {
+        panic!(
+            "回退到 txg {} 那一版：{refusal:?}",
+            target.root.checkpoint_txg.0
+        )
+    });
+    pool.output.clone()
+}
+
+/// 接着发 [`PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER`] 次建一个空 inode：之前那几版都转出回退候选集。
+pub fn turn_the_root_ring_over_by_creating_inodes(pool: &mut BuiltPool) {
+    // 迭代上界是常量 30。
+    for _ in 0..PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER {
+        publish_new_inodes_and_advance_the_pool(pool, 1);
+    }
+}
+
+/// 这一版 inode 树里 `inode` 那条记录的对象出生代；没有这条记录交回 `None`。
+pub fn object_birth_of(output: &TransactionOutput, inode: u64) -> Option<CheckpointTxg> {
+    output
+        .inode_leaf_container_contents()
+        .iter()
+        .flat_map(|container| container.records.iter())
+        .find(|record| record.inode == inode)
+        .map(|record| record.object_birth)
+}
+
+/// 新池新建文件之后建一个空 inode（2 号），再发它的 unlink 意图：队列 {2}、2 号 nlink 0、size 0、没有数据单元，墓碑树一条墓碑。
+pub fn image_after_the_intent_of_an_empty_inode(tag: &str) -> MemoryPool {
+    let mut pool = pool_with_new_inodes(tag, 1);
+    publish_the_intent_of(&mut pool, 2);
+    pool.memory_pool()
+}
+
+/// 新池新建文件之后分两次各建一个空 inode（2 号、3 号，对象出生代不同），把 3 号 unlink 到底（记录删掉，墓碑 (3, 3 号的出生代) 在谱系上），
+/// 再建 [`PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER`] 次：3 号的记录还在的那几版（含意图那一版，它与删完之后各版共用那片墓碑容器）都转出回退候选集。
+pub fn pool_with_inode_three_unlinked_and_the_root_ring_turned_over(tag: &str) -> BuiltPool {
+    let mut pool = pool_with_new_inodes(tag, 1);
+    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+    unlink_to_the_end(&mut pool, 3);
+    turn_the_root_ring_over_by_creating_inodes(&mut pool);
+    pool
+}
+
+/// 新池新建文件之后建一个空 inode（2 号），把它 unlink 到底，再回退到发意图之前那一版（2 号又活着、nlink 1；墓碑树随现行那一版，
+/// 那条墓碑的死亡 txg 落在被回退抛弃的那一段里、不在谱系上），再建 [`PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER`] 次：
+/// 那条墓碑还在谱系上的几版（意图与删完那两版）都转出回退候选集。
+pub fn pool_with_inode_two_unlinked_then_rolled_back_before_the_intent_and_the_root_ring_turned_over(
+    tag: &str,
+) -> BuiltPool {
+    let mut pool = pool_with_new_inodes(tag, 1);
+    let before_the_intent = pool.output.clone();
+    unlink_to_the_end(&mut pool, 2);
+    roll_back_to(&mut pool, &before_the_intent);
+    turn_the_root_ring_over_by_creating_inodes(&mut pool);
+    pool
+}
+
 /// 判违例的不变量，按 checker 报的次序。
 pub fn violated_invariants(image: &MemoryPool) -> Vec<&'static str> {
     check_pool_image(image)
@@ -398,10 +523,13 @@ pub fn set_the_nlink_of_the_queued_first_file_back_to_one(image: &mut MemoryPool
     });
 }
 
-/// I-9.5 正方向「队列里的号没有记录」那一形（判决第二节最后一行：只红 I-9.5）：意图之后的 `deleted_inodes` 根 {1} 改成 {1, 7}
-/// （条目数 1 → 2、声明长度 8 → 16、头里最大 key 1 → 7），7 号在 inode 树里没有记录、extent 树里也没有单元。
-pub fn put_an_inode_number_without_a_record_into_the_deletion_queue(image: &mut MemoryPool) {
+/// I-9.5 正方向「队列里的号没有记录」那一形（判决第二节最后一行：只红 I-9.5）：空 inode 2 发完意图之后的 `deleted_inodes` 根 {2} 改成 {7}
+/// （条目与头里的最小、最大 key 2 → 7），7 号在 inode 树里没有记录、extent 树里也没有单元。队列只一个号（I-9.24 照旧成立）；
+/// 2 号 nlink 0 而不在队列里，反方向也红在 I-9.5 上；2 号没有数据单元、size 0，I-4.7、I-4.6 照旧成立；
+/// 7 号没有记录，I-9.18 那一个号判不了（`.claude/kb/invariants.md` I-9.18 那一行）。
+pub fn replace_the_queued_inode_with_a_number_without_a_record(image: &mut MemoryPool) {
     const INODE_NUMBER_WITHOUT_A_RECORD: u64 = 7;
+    const QUEUED_EMPTY_INODE: u64 = 2;
     let deleted_inodes_root = tree_table_entry_of_kind(image, TREE_KIND_DELETED_INODES).root;
     change_the_unit_and_propagate_its_checksum(
         image,
@@ -413,17 +541,183 @@ pub fn put_an_inode_number_without_a_record_into_the_deletion_queue(image: &mut
             assert_eq!(get_u16(root, count_offset), 1, "意图之后队列里一个号");
             assert_eq!(
                 get_u64(root, entries_start),
-                FIRST_INODE_NUMBER,
-                "队列里是 1"
+                QUEUED_EMPTY_INODE,
+                "队列里是 2"
             );
+            set_u64(
+                root,
+                INDEX_NODE_SMALLEST_KEY_OFFSET,
+                INODE_NUMBER_WITHOUT_A_RECORD,
+            );
+            set_u64(
+                root,
+                index_node_largest_key_offset(DELETED_INODES_KEY_BYTES),
+                INODE_NUMBER_WITHOUT_A_RECORD,
+            );
+            set_u64(root, entries_start, INODE_NUMBER_WITHOUT_A_RECORD);
+        },
+    );
+}
+
+/// I-9.24（删除队列至多一条 key）只红自己的那一形：一次发布建了两个空 inode（2、3 号，对象出生代相同）、发完 2 号的意图之后，
+/// 手改三处让 3 号也成一个合法的删除中号——`deleted_inodes` 根 {2} 加 3（条目数 1 → 2、声明长度 8 → 16、头里最大 key 2 → 3）、
+/// 3 号记录 nlink 1 → 0、墓碑容器追加 (3, 3 号的对象出生代, 整对象, 意图那次的 txg)（记录数 1 → 2、声明长度 40 → 80）。
+/// 少改哪一处都会连带别的条（I-9.5、I-9.18、I-9.19）。
+pub fn put_a_second_inode_being_deleted_into_the_deletion_queue(image: &mut MemoryPool) {
+    const SECOND_QUEUED_INODE: u64 = 3;
+    let deleted_inodes_root = tree_table_entry_of_kind(image, TREE_KIND_DELETED_INODES).root;
+    change_the_unit_and_propagate_its_checksum(
+        image,
+        the_slot_of(&deleted_inodes_root),
+        node_bytes(),
+        |root| {
+            let count_offset = index_node_entry_count_offset(DELETED_INODES_KEY_BYTES);
+            let entries_start = index_node_entries_start(DELETED_INODES_KEY_BYTES);
+            assert_eq!(get_u16(root, count_offset), 1, "意图之后队列里一个号");
+            assert_eq!(get_u64(root, entries_start), 2, "队列里是 2");
             set_u16(root, count_offset, 2);
             set_u16(root, UNIT_DECLARED_LENGTH_OFFSET, 2 * 8);
             set_u64(
                 root,
-                INDEX_NODE_SMALLEST_KEY_OFFSET + DELETED_INODES_KEY_BYTES,
-                INODE_NUMBER_WITHOUT_A_RECORD,
+                index_node_largest_key_offset(DELETED_INODES_KEY_BYTES),
+                SECOND_QUEUED_INODE,
+            );
+            set_u64(root, entries_start + 8, SECOND_QUEUED_INODE);
+        },
+    );
+    change_the_inode_record(image, SECOND_QUEUED_INODE, |record| {
+        assert_eq!(
+            get_u32(record, INODE_RECORD_NLINK_OFFSET),
+            1,
+            "改之前 3 号的 nlink 是 1"
+        );
+        set_u32(record, INODE_RECORD_NLINK_OFFSET, 0);
+    });
+    let container_slot = first_tombstone_container_slot_of_the_newest_root(image);
+    change_the_unit_and_propagate_its_checksum(
+        image,
+        container_slot,
+        data_unit_bytes(),
+        |container| {
+            assert_eq!(
+                get_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET),
+                1,
+                "意图之后墓碑容器里一条墓碑"
+            );
+            assert_eq!(
+                get_u64(container, PACKED_UNIT_RECORDS_START),
+                2,
+                "那条墓碑的对象 ID 是 2"
+            );
+            let first = PACKED_UNIT_RECORDS_START;
+            let second = first + TOMBSTONE_RECORD_BYTES;
+            let mut tombstone_of_the_second = container[first..second].to_vec();
+            set_u64(
+                &mut tombstone_of_the_second,
+                TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET,
+                SECOND_QUEUED_INODE,
+            );
+            container[second..second + TOMBSTONE_RECORD_BYTES]
+                .copy_from_slice(&tombstone_of_the_second);
+            set_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET, 2);
+            set_u16(
+                container,
+                UNIT_DECLARED_LENGTH_OFFSET,
+                u16::try_from(2 * TOMBSTONE_RECORD_BYTES).expect("80"),
             );
-            set_u64(root, entries_start + 8, INODE_NUMBER_WITHOUT_A_RECORD);
+        },
+    );
+}
+
+/// 最新的根那一版墓碑树第一片容器里第一条墓碑改一处（`change` 拿到的是那 40 字节），重封、补校验和。
+pub fn change_the_first_tombstone_of_the_newest_root(
+    image: &mut MemoryPool,
+    change: impl FnOnce(&mut [u8]),
+) {
+    let container_slot = first_tombstone_container_slot_of_the_newest_root(image);
+    change_the_unit_and_propagate_its_checksum(
+        image,
+        container_slot,
+        data_unit_bytes(),
+        |container| {
+            let first = PACKED_UNIT_RECORDS_START;
+            change(&mut container[first..first + TOMBSTONE_RECORD_BYTES]);
+        },
+    );
+}
+
+/// 最新的根那一版墓碑树第一片容器里第一条墓碑原样再追加一条（记录数 1 → 2、声明长度 40 → 80）：同一对 (号, 对象出生代)、同一个死亡 txg 两条。
+pub fn append_a_copy_of_the_first_tombstone_of_the_newest_root(image: &mut MemoryPool) {
+    let container_slot = first_tombstone_container_slot_of_the_newest_root(image);
+    change_the_unit_and_propagate_its_checksum(
+        image,
+        container_slot,
+        data_unit_bytes(),
+        |container| {
+            assert_eq!(
+                get_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET),
+                1,
+                "墓碑容器里一条墓碑"
+            );
+            let first = PACKED_UNIT_RECORDS_START;
+            let second = first + TOMBSTONE_RECORD_BYTES;
+            let tombstone = container[first..second].to_vec();
+            container[second..second + TOMBSTONE_RECORD_BYTES].copy_from_slice(&tombstone);
+            set_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET, 2);
+            set_u16(
+                container,
+                UNIT_DECLARED_LENGTH_OFFSET,
+                u16::try_from(2 * TOMBSTONE_RECORD_BYTES).expect("80"),
+            );
+        },
+    );
+}
+
+/// 一条墓碑（40 字节）里 `offset` 那 8 字节读成数。
+pub fn tombstone_field(tombstone: &[u8], offset: usize) -> u64 {
+    get_u64(tombstone, offset)
+}
+
+/// 一条墓碑（40 字节）里 `offset` 那 8 字节改成 `value`。
+pub fn set_tombstone_field(tombstone: &mut [u8], offset: usize, value: u64) {
+    set_u64(tombstone, offset, value);
+}
+
+/// I-9.4 第四句（同一片打包记录类型 2 容器里记录号沿记录序严格递增）只红自己的那一形（S4a，代码轮判决
+/// `research/prompts/m3-batch20-code-r1-main-verification.md` 第二节）：新池新建文件之后建一个空 inode 的那一版，叶 [1, 2] 改成
+/// [1 nlink 0, 1 nlink 1]——第二条整条换成 1 号那条原样，第一条 nlink 改 0；队列空。扫描里同号后者盖前者（1 号 nlink 1），
+/// I-9.5、I-4.7 照旧成立。
+pub fn turn_the_leaf_into_two_records_of_the_first_file(image: &mut MemoryPool) {
+    let (leaf_slot, first_index) = inode_leaf_holding(image, FIRST_INODE_NUMBER);
+    assert_eq!(first_index, 0, "1 号是叶里第 0 条");
+    change_the_unit_and_propagate_its_checksum(image, leaf_slot, data_unit_bytes(), |leaf| {
+        assert_eq!(
+            get_u16(leaf, PACKED_UNIT_RECORD_COUNT_OFFSET),
+            2,
+            "叶里两条：1 号与空 inode 2"
+        );
+        let first = PACKED_UNIT_RECORDS_START;
+        let second = first + INODE_RECORD_BYTES;
+        let record_of_the_first_file = leaf[first..second].to_vec();
+        leaf[second..second + INODE_RECORD_BYTES].copy_from_slice(&record_of_the_first_file);
+        set_u32(leaf, first + INODE_RECORD_NLINK_OFFSET, 0);
+    });
+}
+
+/// 最新的根那一版种类 `kind`（墓碑树或谱系树）的树根第一条条目指的那片容器头里的记录宽改成 `record_width`（I-1.7 只看头字段，
+/// 声明长度、记录区不动）。
+pub fn set_the_record_width_of_the_first_container_of_the_tree_of_kind(
+    image: &mut MemoryPool,
+    kind: u16,
+    record_width: u16,
+) {
+    let container_slot = first_container_slot_of_the_tree_of_kind_of_the_newest_root(image, kind);
+    change_the_unit_and_propagate_its_checksum(
+        image,
+        container_slot,
+        data_unit_bytes(),
+        |container| {
+            set_u16(container, PACKED_UNIT_RECORD_WIDTH_OFFSET, record_width);
         },
     );
 }
@@ -557,32 +851,25 @@ pub fn drop_the_pending_deletion_occupancy_row(image: &mut MemoryPool) {
     change_the_pending_deletion_occupancy_row_of_the_accounting_tree_at(image, slot, None);
 }
 
-/// I-9.6 墓碑那一半：意图之后墓碑树第一片容器里那条墓碑的对象 ID 1 → 水位（记账里的 inode 号水位 2）：水位不再大于全部墓碑的对象 ID，
-/// inode 树里最大的 key 仍是 1。
-pub fn raise_the_tombstone_object_identifier_to_the_inode_number_watermark(image: &mut MemoryPool) {
-    let container_slot = first_tombstone_container_slot_of_the_newest_root(image);
-    change_the_unit_and_propagate_its_checksum(
-        image,
-        container_slot,
-        data_unit_bytes(),
-        |container| {
-            assert_eq!(
-                get_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET),
-                1,
-                "意图之后墓碑容器里一条墓碑"
-            );
-            assert_eq!(
-                get_u64(container, PACKED_UNIT_RECORDS_START),
-                FIRST_INODE_NUMBER,
-                "那条墓碑的对象 ID 是 1"
-            );
-            set_u64(
-                container,
-                PACKED_UNIT_RECORDS_START,
-                INODE_NUMBER_WATERMARK_AFTER_THE_FIRST_FILE,
-            );
-        },
-    );
+/// I-9.6 墓碑那一半：3 号删完、根环转过一圈之后（`pool_with_inode_three_unlinked_and_the_root_ring_turned_over`）墓碑树第一片容器里
+/// 那条墓碑的对象 ID 3 → `inode_number_watermark`（记账里的 inode 号水位）：水位不再大于全部墓碑的对象 ID，inode 树里最大的 key 仍小于水位。
+/// 那条墓碑所在的容器被意图那一版共用，意图那一版要先转出候选集，不然它的队列 {3} 找不到 3 号的墓碑、I-9.18 连带红。
+pub fn raise_the_tombstone_object_identifier_to_the_inode_number_watermark(
+    image: &mut MemoryPool,
+    inode_number_watermark: u64,
+) {
+    change_the_first_tombstone_of_the_newest_root(image, |tombstone| {
+        assert_eq!(
+            get_u64(tombstone, TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET),
+            3,
+            "那条墓碑的对象 ID 是 3"
+        );
+        set_u64(
+            tombstone,
+            TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET,
+            inode_number_watermark,
+        );
+    });
 }
 
 /// 意图之后记账里的 inode 号水位（第一个文件用掉 1 号）。
@@ -590,16 +877,24 @@ const INODE_NUMBER_WATERMARK_AFTER_THE_FIRST_FILE: u64 = 2;
 
 /// 最新的根那一版墓碑树根第一条条目指的那片墓碑容器的槽（这几份镜像上墓碑树根是层级 1、孩子是容器）。
 pub fn first_tombstone_container_slot_of_the_newest_root(image: &MemoryPool) -> SlotNumber {
-    let tombstone_root = tree_table_entry_of_kind(image, TREE_KIND_TOMBSTONE).root;
+    first_container_slot_of_the_tree_of_kind_of_the_newest_root(image, TREE_KIND_TOMBSTONE)
+}
+
+/// 最新的根那一版种类 `kind`（墓碑树或谱系树）的树根第一条条目指的那片容器的槽（这几份镜像上树根是层级 1、孩子是容器）。
+pub fn first_container_slot_of_the_tree_of_kind_of_the_newest_root(
+    image: &MemoryPool,
+    kind: u16,
+) -> SlotNumber {
+    let container_tree_root = tree_table_entry_of_kind(image, kind).root;
     let first_entry = parse_index_node(&unit_bytes_at(
         image,
-        the_slot_of(&tombstone_root),
+        the_slot_of(&container_tree_root),
         node_bytes(),
     ))
-    .expect("墓碑树根解得开")
+    .expect("容器树根解得开")
     .entries
     .first()
-    .expect("墓碑树根至少一条条目")
+    .expect("容器树根至少一条条目")
     .clone();
     child_slot_of_a_container_tree_entry(&first_entry)
 }
@@ -852,22 +1147,21 @@ pub fn root_slot_of_the_tree_of_kind_of_the_newest_root(
     the_slot_of(&tree_table_entry_of_kind(image, kind).root)
 }
 
-/// 意图之后那一版上的坏镜像（`image_after_the_intent_of_a_three_unit_file`）。
+/// 意图之后那一版上的坏镜像（`image_after_the_intent_of_a_three_unit_file`）。1 号 nlink 0 → 1 那一份必与 I-9.19（谱系上的墓碑不指活对象）
+/// 同红（墓碑 (1, 1 号的出生代) 在谱系上而记录活着），登记在 I-9.5 名下。
 pub fn known_bad_images_after_the_intent_of_a_three_unit_file() -> Vec<(&'static str, Mutation)> {
-    vec![
-        (
-            DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
-            Box::new(put_an_inode_number_without_a_record_into_the_deletion_queue),
-        ),
-        (
-            DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
-            Box::new(set_the_nlink_of_the_queued_first_file_back_to_one),
-        ),
-        (
-            "I-9.6",
-            Box::new(raise_the_tombstone_object_identifier_to_the_inode_number_watermark),
-        ),
-    ]
+    vec![(
+        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+        Box::new(set_the_nlink_of_the_queued_first_file_back_to_one),
+    )]
+}
+
+/// 空 inode 2 发完意图之后那一版上的坏镜像（`image_after_the_intent_of_an_empty_inode`）。
+pub fn known_bad_images_after_the_intent_of_an_empty_inode() -> Vec<(&'static str, Mutation)> {
+    vec![(
+        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+        Box::new(replace_the_queued_inode_with_a_number_without_a_record),
+    )]
 }
 
 /// 意图与第一批之后那一版上的坏镜像（`image_after_the_first_batch_of_a_three_unit_file`）。
diff --git a/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs b/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
index d592a73..beebcd0 100644
--- a/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
+++ b/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
@@ -1,6 +1,8 @@
 //! 墓碑树与谱系树根满长高（D18（块里携带什么信息） 已定项 24 第 1、5 条「根满时长高，照 D8（核心索引结构） 已定项 6」，用户定案 K-R3-3）：
 //! 码 2 节点装满时末尾分裂，根满时长一层——新根与分出的右半各一个码 2，旧根一个字节不动、留在原位置当根之下的节点；
 //! 只供测试的旋钮把容器与节点压小（一片 1 条、一个节点 2 条），旋钮不落盘；重建读层级 > 1 的根。池是两块 4 GiB 盘。
+//! 池级 checker 从镜像外拿墓碑容器的旋钮值（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T5 取 K1）：
+//! 这些镜像走带旋钮值的入口，I-9.22（测试镜像里墓碑容器不超旋钮值） 按旋钮值判；同一张镜像交给旧入口（当没装旋钮）I-9.21 红。
 
 mod common;
 mod common_deletion_in_progress;
@@ -10,8 +12,14 @@ use common::{build_pool, file_content, parameters, BuiltPool};
 use common_deletion_in_progress::{
     pool_with_new_inodes, INSTANCE_OF_THE_PUBLISHES, WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
 };
-use singlefs_checker::image::InvariantVerdict;
-use singlefs_checker::walk::check_pool_image;
+use singlefs_checker::image::{
+    InvariantVerdict, NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+    TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
+};
+use singlefs_checker::walk::{
+    check_pool_image, check_pool_image_given_the_tombstone_container_capacity_knob,
+    TombstoneContainerCapacityKnob,
+};
 use singlefs_core::address::InodeNumber;
 use singlefs_core::admission::{
     central_mapping_entries_of_one_lineage_container_cow,
@@ -39,6 +47,9 @@ use singlefs_core::transaction::{
 };
 use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchSize};
 
+/// 写这些镜像时墓碑容器旋钮压到的每片记录数（`capacities_of_one_record_and_two_entries` 那一个），交给池级 checker 带旋钮值的入口。
+const TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;
+
 /// 一片容器 1 条、一个码 2 节点 2 条：第 3 片容器根满长一层，第 5 片长到层级 3。
 fn capacities_of_one_record_and_two_entries() -> ContainerTreeCapacities {
     ContainerTreeCapacities {
@@ -107,7 +118,28 @@ fn rebuilt_from_disk(pool: &BuiltPool) -> TransactionOutput {
     }
 }
 
-fn violations(pool: &BuiltPool) -> Vec<String> {
+fn tombstone_container_capacity_knob(records_per_container: u64) -> TombstoneContainerCapacityKnob {
+    TombstoneContainerCapacityKnob::new(records_per_container).expect("旋钮值在 1..=815 里")
+}
+
+/// 池级 checker 带旋钮值的入口判出来的违例：`records_per_container` 是写这张镜像时墓碑容器旋钮压到的每片记录数。
+fn violations_given_the_knob(pool: &BuiltPool, records_per_container: u64) -> Vec<String> {
+    check_pool_image_given_the_tombstone_container_capacity_knob(
+        &pool.memory_pool(),
+        Some(tombstone_container_capacity_knob(records_per_container)),
+    )
+    .into_iter()
+    .filter_map(|(invariant, verdict)| match verdict {
+        InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
+        InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_) => None,
+    })
+    .collect()
+}
+
+/// 池级 checker 旧入口（当没装旋钮）判出来的违例。
+fn violations_without_the_knob(pool: &BuiltPool) -> Vec<String> {
     check_pool_image(&pool.memory_pool())
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
@@ -203,7 +235,11 @@ fn the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place(
         },
         "冷启动恢复（映射条目数把墓碑树每个节点都算上）读回第一个文件"
     );
-    assert_eq!(violations(&pool), Vec::<String>::new(), "池级 checker 全绿");
+    assert_eq!(
+        violations_given_the_knob(&pool, TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB),
+        Vec::<String>::new(),
+        "池级 checker（带旋钮值）全绿"
+    );
 }
 
 /// 再 unlink 两个：第 5 片时层级 1 与根都满，长到层级 3（新开 (1, 2)、(2, 1)、新根 (3, 0)）；重建读回层级 3 的根，池级 checker 全绿。
@@ -233,7 +269,11 @@ fn the_fifth_tombstone_container_grows_the_tree_to_three_levels_and_rebuild_read
         ),
         "从盘上重建读回层级 3 的墓碑树"
     );
-    assert_eq!(violations(&pool), Vec::<String>::new(), "池级 checker 全绿");
+    assert_eq!(
+        violations_given_the_knob(&pool, TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB),
+        Vec::<String>::new(),
+        "池级 checker（带旋钮值）全绿"
+    );
 }
 
 /// 旋钮不落盘：压小扇出写出来的两层树，换回产品扇出（135）接着写，照盘上的形状往最右那个层级 1 节点里挂（它只有 1 个孩子），不再长高。
@@ -256,7 +296,11 @@ fn a_tree_grown_under_the_capped_fanout_takes_the_next_container_under_the_produ
         "产品扇出下 (1, 1) 还装得下：挂进去、重写它与根，根层级仍 2"
     );
     assert_eq!(next.tombstone_tree_index_nodes.shape.root_level(), 2);
-    assert_eq!(violations(&pool), Vec::<String>::new(), "池级 checker 全绿");
+    assert_eq!(
+        violations_given_the_knob(&pool, TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB),
+        Vec::<String>::new(),
+        "池级 checker（带旋钮值）全绿"
+    );
 }
 
 /// 谱系树同一条（第 5 条「谱系树根满同样长高」）：一片 1 条、一个节点 2 条，回退三次之后谱系树两层；从盘上重建读回层级 2 的根；
@@ -335,5 +379,112 @@ fn the_third_rollback_grows_the_lineage_tree_and_the_checkpoint_cost_counts_the_
         },
         "冷启动恢复读回第一个文件"
     );
-    assert_eq!(violations(&pool), Vec::<String>::new(), "池级 checker 全绿");
+    assert_eq!(
+        violations_given_the_knob(&pool, TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB),
+        Vec::<String>::new(),
+        "池级 checker（带旋钮值）全绿"
+    );
+}
+
+/// I-9.21（产品镜像里非最右墓碑容器装满） 只红自己的那一形（判决第二节 T5 那一格：旋钮压小的合法镜像上按产品容量判就红）：
+/// 墓碑容器压到一片 1 条，unlink 2、3、4 号各占一片容器；这张镜像交给旧入口（当没装旋钮），不是最右的两片各只装 1 条、不是 815，
+/// I-9.21 红、别的一条不红（I-9.22 报不适用）；交给带旋钮值 1 的入口则全绿，I-9.22 真被评估过且成立、I-9.21 报不适用。
+#[test]
+fn a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant() {
+    let mut pool = pool_with_new_inodes("tombstone-tree-judged-as-a-product-image", 5);
+    let capacities = capacities_of_one_record_and_two_entries();
+    for inode in 2..=4 {
+        unlink(&mut pool, inode, capacities);
+    }
+    let verdicts_without_the_knob = check_pool_image(&pool.memory_pool());
+    let violated_without_the_knob: Vec<&str> = verdicts_without_the_knob
+        .iter()
+        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
+        .map(|(invariant, _)| *invariant)
+        .collect();
+    assert_eq!(
+        violated_without_the_knob,
+        [NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE],
+        "旧入口当没装旋钮：只红 I-9.21"
+    );
+    assert!(
+        violations_without_the_knob(&pool)
+            .iter()
+            .any(|violation| violation.contains("装了 1 条，不是 815")),
+        "违例说明写出那片容器装了几条：{:?}",
+        violations_without_the_knob(&pool)
+    );
+    let verdicts_given_the_knob = check_pool_image_given_the_tombstone_container_capacity_knob(
+        &pool.memory_pool(),
+        Some(tombstone_container_capacity_knob(
+            TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
+        )),
+    );
+    for (invariant, verdict) in &verdicts_given_the_knob {
+        let expected_holds =
+            *invariant == TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE;
+        let expected_not_applicable =
+            *invariant == NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE;
+        if expected_holds {
+            assert_eq!(
+                *verdict,
+                InvariantVerdict::Holds,
+                "带旋钮值：I-9.22 真被评估过且成立"
+            );
+        }
+        if expected_not_applicable {
+            assert!(
+                matches!(verdict, InvariantVerdict::NotApplicable(_)),
+                "带旋钮值：I-9.21 报不适用，实际 {verdict:?}"
+            );
+        }
+        assert!(
+            !matches!(verdict, InvariantVerdict::Violated(_)),
+            "带旋钮值：{invariant} 不红，实际 {verdict:?}"
+        );
+    }
+}
+
+/// I-9.22（测试镜像里墓碑容器不超旋钮值） 只红自己的那一形：墓碑容器压到一片 2 条写出来的镜像（unlink 2、3 号同在一片容器里），
+/// 交给带旋钮值的入口时报旋钮值 1——那片容器装了 2 条、多于旋钮值，I-9.22 红、别的一条不红（I-9.21 报不适用）。同一张镜像报旋钮值 2 全绿。
+#[test]
+fn a_tombstone_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant(
+) {
+    let mut pool = pool_with_new_inodes("tombstone-container-above-the-knob", 5);
+    let capacities_of_two_records = ContainerTreeCapacities {
+        tombstone_container: TombstoneContainerCapacity::CappedForTests(
+            TombstoneRecordsPerContainerCappedForTests::new(2).expect("1..=815"),
+        ),
+        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
+        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
+    };
+    for inode in 2..=3 {
+        unlink(&mut pool, inode, capacities_of_two_records);
+    }
+    assert_eq!(
+        violations_given_the_knob(&pool, 2),
+        Vec::<String>::new(),
+        "报写的时候那个旋钮值 2：全绿"
+    );
+    let verdicts = check_pool_image_given_the_tombstone_container_capacity_knob(
+        &pool.memory_pool(),
+        Some(tombstone_container_capacity_knob(1)),
+    );
+    let violated: Vec<&str> = verdicts
+        .iter()
+        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
+        .map(|(invariant, _)| *invariant)
+        .collect();
+    assert_eq!(
+        violated,
+        [TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE],
+        "报旋钮值 1：只红 I-9.22"
+    );
+    assert!(
+        violations_given_the_knob(&pool, 1)
+            .iter()
+            .any(|violation| violation.contains("装了 2 条，多于旋钮值 1")),
+        "违例说明写出那片容器装了几条与旋钮值：{:?}",
+        violations_given_the_knob(&pool, 1)
+    );
 }
diff --git a/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs b/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs
index 3c6b9c7..3461dd6 100644
--- a/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs
+++ b/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs
@@ -10,6 +10,10 @@
 //! 第二节「N7 附带」那一行交给 C479 那一问当材料）。
 //! p + n 不大于墓碑最大对象号、又大于各条根里 inode 树最大 key 的那一形，I-9.6 墓碑那一半判得出，钉在
 //! [`creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant`]。
+//! C479 那一问用户 2026-10-05 定「写」（D8（核心索引结构） 已定项 6「同一条时间线上 inode 号不复用」），立 I-9.29（同号只许一个出生代，
+//! 判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T10 乙）：重发的号在回退候选集里还有一版留着旧对象时它红，
+//! 下面三条用例都红它；I-9.6 拦不住的「压回的正是发过的最大号」那一形只红它，钉在
+//! [`pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_only_the_one_birth_per_number_invariant`]。
 
 mod common;
 mod common_deletion_in_progress;
@@ -20,7 +24,7 @@ use common_deletion_in_progress::{
     pool_with_new_inodes, verdict_of, violated_invariants, INSTANCE_OF_THE_PUBLISHES,
     WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
 };
-use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::image::{InvariantVerdict, ONE_OBJECT_BIRTH_PER_INODE_NUMBER};
 use singlefs_core::address::{CheckpointTxg, InodeNumber};
 use singlefs_core::mount::{
     raise_rollback_floor_to_the_admission_ceiling, RaiseToTheAdmissionCeiling, ShadowLedger,
@@ -80,7 +84,8 @@ fn object_birth_of(output: &TransactionOutput, inode: u64) -> Option<u64> {
 
 /// 新池新建文件之后建 inode 2、3、4（水位 5），再建一个 inode：开关把水位压回 3 时新建的那个拿到 3 号——
 /// inode 树里原来那条 3 号记录被原地换成新对象（对象出生代换成这次发布的 txg），记账里的水位写成 4，不大于树里最大的 key 4，
-/// I-9.6 判红；同一段流开关关着时新建的拿 5 号、水位 6，I-9.6 成立、一条违例都没有。
+/// I-9.6 判红；回退候选集里更早那一版还留着 3 号的旧对象，I-9.29（同号只许一个出生代） 也红。
+/// 同一段流开关关着时新建的拿 5 号、水位 6，I-9.6 成立、一条违例都没有。
 #[test]
 fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant(
 ) {
@@ -115,9 +120,20 @@ fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_r
     }
     assert_eq!(
         violated_invariants(&pressed_image),
-        vec!["I-9.6"],
-        "只红 I-9.6"
+        vec!["I-9.6", ONE_OBJECT_BIRTH_PER_INODE_NUMBER],
+        "红 I-9.6 与 I-9.29"
     );
+    match verdict_of(&pressed_image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER) {
+        InvariantVerdict::Violated(detail) => assert!(
+            detail.contains("inode 3 在回退候选集里对应不止一个对象出生代"),
+            "I-9.29 的说明点名被重发的那个号：{detail}"
+        ),
+        other_verdict @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
+            panic!("3 号被重发而更早那一版还留着旧对象，I-9.29 应当判红，实际 {other_verdict:?}")
+        }
+    }
 
     let mut control_pool = pool_with_new_inodes("watermark-not-pressed-back-control", 3);
     let (control, control_image) = create_one_inode_with_the_watermark_from(
@@ -260,6 +276,7 @@ fn pool_whose_inode_three_is_unlinked_and_whose_floor_is_raised_past_its_record(
 /// 不抬 F 时池级 checker 还走着意图那一条根（3 号记录 nlink = 0 还在树里），树内最大 key 那一半先判出 3，说明写的是树那一半（S 的探针量过），
 /// 判不出墓碑那一半拦不拦得住。
 /// 判别力：墓碑那一半不比水位（`walk.rs` 里那一判恒真）时 I-9.6 成立，这一条红；关开关那一半由红转绿是判别力自证。
+/// 2 号被重发时回退候选集里抬 F 之后那几版（覆盖写三次的）还留着 2 号的旧对象，I-9.29（同号只许一个出生代） 也红。
 #[test]
 fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant(
 ) {
@@ -293,8 +310,8 @@ fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombs
     }
     assert_eq!(
         violated_invariants(&pressed_image),
-        vec!["I-9.6"],
-        "只红 I-9.6"
+        vec!["I-9.6", ONE_OBJECT_BIRTH_PER_INODE_NUMBER],
+        "红 I-9.6 与 I-9.29"
     );
 
     let (mut control_pool, _) =
@@ -321,3 +338,54 @@ fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombs
         "开关关着的同一段流一条违例都没有"
     );
 }
+
+/// C479 变体 a（第 19 批 S 的探针：`/tmp/claude-1000/m3-batch19/s/probe/run2-probe.log` 那一形，I-9.6 拦不住的「压回的正是发过的最大号」）：
+/// 新池新建文件之后建 inode 2、3、4（水位 5），开关把水位压回 4 再建一个——4 号那条活记录被原地换成新对象（对象出生代换成这次发布的 txg），
+/// 记账的水位写成 5，仍大于树里最大的 key 4 与墓碑（没有）：I-9.6 成立。回退候选集里建 2、3、4 那一版还留着 4 号的旧对象，
+/// I-9.29（同号只许一个出生代） 判红，别的一条不红。
+#[test]
+fn pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_only_the_one_birth_per_number_invariant(
+) {
+    let mut pressed_pool =
+        pool_with_new_inodes("watermark-pressed-back-to-the-largest-issued-number", 3);
+    let object_birth_before = object_birth_of(&pressed_pool.output, 4).expect("4 号活着");
+    let (pressed, pressed_image) =
+        create_one_inode_with_the_watermark_from(&mut pressed_pool, pressed_back_to(4));
+    assert_eq!(
+        pressed.inode_number_watermark(),
+        5,
+        "从压回去的 4 起发一个，水位写成 5"
+    );
+    assert_eq!(
+        object_birth_of(&pressed, 4),
+        Some(pressed.root.checkpoint_txg.0),
+        "4 号被重发：树里那条记录换成这次发布建的新对象"
+    );
+    assert_ne!(
+        Some(object_birth_before),
+        object_birth_of(&pressed, 4),
+        "重发前后 4 号的对象出生代不同"
+    );
+    assert_eq!(
+        verdict_of(&pressed_image, "I-9.6"),
+        InvariantVerdict::Holds,
+        "水位 5 仍大于树里最大的 key 4：I-9.6 拦不住这一形"
+    );
+    assert_eq!(
+        violated_invariants(&pressed_image),
+        vec![ONE_OBJECT_BIRTH_PER_INODE_NUMBER],
+        "只红 I-9.29"
+    );
+    match verdict_of(&pressed_image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER) {
+        InvariantVerdict::Violated(detail) => assert!(
+            detail.contains("inode 4 在回退候选集里对应不止一个对象出生代")
+                && detail.contains(&format!("对象出生代 {object_birth_before}")),
+            "I-9.29 的说明点名被重发的那个号与它的旧出生代：{detail}"
+        ),
+        other_verdict @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
+            panic!("4 号被重发而更早那一版还留着旧对象，I-9.29 应当判红，实际 {other_verdict:?}")
+        }
+    }
+}
diff --git a/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs b/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs
index 643f0d7..f963b53 100644
--- a/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs
+++ b/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs
@@ -1,5 +1,7 @@
 //! 池级 checker 的删除线（里程碑三·3）：I-9.5（删除队列里的号仍在树里，两个方向）、I-3.4（只落「记账标签 4 恒 0」）、
-//! I-4.7（对象取丙）、I-4.6（取乙：nlink = 0 的记录单元数对 size）、I-4.4（第一版不适用）与 I-9.6 墓碑那一半。
+//! I-4.7（对象取丙）、I-4.6（取乙：nlink = 0 的记录单元数对 size）、I-4.4（第一版不适用）与 I-9.6 墓碑那一半；
+//! 墓碑与谱系那几条（判决 `research/prompts/m3-delete-checker-r2-main-verification.md`）：I-9.18、I-9.19、I-9.20、I-9.24 与
+//! I-9.10 墓碑那一个等号，「在谱系上」每一版从自己的 txg 与自己的谱系树起算（T3 取 a）。
 //! 读法照判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节与岔路单 `research/prompts/m3-delete-checker-r1-forks.md`：
 //! 判回退候选集里每一版（加由记录施加出来的几版），不只判最新根；读不全的那一版报射程里判不了、不判红。
 //! 坏镜像各一，每份只红自己那一条；搭建与坏镜像的改法在 `common_deletion_in_progress`（`checker_known_bad_images.rs` 那一串 `.chain` 也登记它们）。
@@ -11,6 +13,8 @@ mod common_pointer_head_corruption;
 use common::build_pool;
 use common_deletion_in_progress::{
     accounting_tree_slot_of_the_newest_root,
+    append_a_copy_of_the_first_tombstone_of_the_newest_root,
+    change_the_first_tombstone_of_the_newest_root,
     change_the_pending_deletion_occupancy_row_of_the_accounting_tree_at,
     drop_the_pending_deletion_occupancy_row,
     drop_the_record_of_the_first_file_and_keep_its_data_units,
@@ -18,30 +22,40 @@ use common_deletion_in_progress::{
     first_tombstone_container_slot_of_the_newest_root,
     give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at,
     image_after_the_first_batch_of_a_three_unit_file, image_after_the_intent_of_a_three_unit_file,
+    image_after_the_intent_of_an_empty_inode,
     let_the_empty_livelist_entry_stand_in_for_the_extent_tree_in_the_tree_table_at,
     move_the_tombstone_container_to_an_unpublished_instance_and_raise_its_object_identifier,
-    narrow_the_entries_of_the_tree_table_at_to_the_key_width,
+    narrow_the_entries_of_the_tree_table_at_to_the_key_width, object_birth_of,
     pool_with_a_first_file_of_size_zero_and_one_data_unit, pool_with_a_three_unit_file,
+    pool_with_inode_three_unlinked_and_the_root_ring_turned_over,
+    pool_with_inode_two_unlinked_then_rolled_back_before_the_intent_and_the_root_ring_turned_over,
     pool_with_new_inodes, publish_new_inodes_and_advance_the_pool, publish_the_intent,
-    publish_the_next_step_in_batches_of_one,
-    put_an_inode_number_without_a_record_into_the_deletion_queue,
+    publish_the_intent_of, publish_the_next_step_in_batches_of_one,
+    put_a_second_inode_being_deleted_into_the_deletion_queue,
     raise_the_size_of_the_first_file_by_one_data_unit_payload,
     raise_the_size_of_the_first_file_in_the_leaf_at,
     raise_the_tombstone_object_identifier_to_the_inode_number_watermark,
     remove_the_deleted_inodes_entry_from_the_tree_table_at,
+    replace_the_queued_inode_with_a_number_without_a_record, roll_back_to,
     root_slot_of_the_tree_of_kind_of_the_newest_root,
     set_the_head_identifier_of_the_tree_table_entry_of_kind_at,
     set_the_nlink_of_an_inode_record_in_the_leaf_at_to_zero,
     set_the_nlink_of_an_inode_record_to_zero, set_the_nlink_of_the_queued_first_file_back_to_one,
-    set_the_pending_deletion_occupancy_row_to_one, tree_table_slot_of_the_newest_root, verdict_of,
-    violated_invariants, zero_both_copies_of_the_unit_at,
+    set_the_pending_deletion_occupancy_row_to_one, set_tombstone_field, tombstone_field,
+    tree_table_slot_of_the_newest_root, unlink_to_the_end, verdict_of, violated_invariants,
+    zero_both_copies_of_the_unit_at, TOMBSTONE_RECORD_DEATH_TXG_OFFSET,
+    TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET, TOMBSTONE_RECORD_RANGE_END_OFFSET,
 };
 use common_pointer_head_corruption::{data_unit_bytes, node_bytes, unit_bytes_at};
 use singlefs_checker::image::{
     InvariantVerdict, DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
     DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
-    INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT, INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
-    INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
+    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY, INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
+    INTENT_DESCRIBES_ONLY_THE_TARGET_STATE, INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION,
+    ONE_OBJECT_BIRTH_PER_INODE_NUMBER, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
+    QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
+    TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
+    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
 };
 use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
 use singlefs_core::records::{TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT};
@@ -94,6 +108,8 @@ const NOT_JUDGED_TREE_TABLE_UNREADABLE: &str = "树表读不出";
 const NOT_JUDGED_TREE_TABLE_ENTRY_SHORTER_THAN_THE_FIELD_TABLE: &str =
     "树表条目宽不到字段表的 200 字节";
 const NOT_JUDGED_TOMBSTONE_TREE_UNREADABLE: &str = "所选根的树表或墓碑树读不全";
+const NOT_JUDGED_QUEUED_INODE_WITHOUT_A_RECORD: &str =
+    "deleted_inodes 里有一个号在同一版 inode 树里没有记录";
 
 /// 干净的那一版上一条违例都没有：坏镜像「只红自己」从这里比起。
 fn assert_the_base_image_is_clean(image: &MemoryPool, base: &str) {
@@ -104,15 +120,15 @@ fn assert_the_base_image_is_clean(image: &MemoryPool, base: &str) {
     );
 }
 
-/// I-9.5 正方向，只红自己的那一形（判决第二节最后一行）：意图之后队列 {1} 改成 {1, 7}，7 号在 inode 树里没有记录、extent 树里也没有单元。
-/// I-9.18（删除队列里的号有谱系上的墓碑） 这一版 checker 还没实现（不在 `IMPLEMENTED_INVARIANTS` 里），这一形上它不判。
+/// I-9.5 正方向，只红自己的那一形（判决第二节最后一行）：空 inode 2 发完意图之后队列 {2} 改成 {7}，7 号在 inode 树里没有记录、
+/// extent 树里也没有单元（队列只一个号，I-9.24 照旧成立；2 号 nlink 0 而不在队列里，反方向也红在 I-9.5 上）。
+/// I-9.18（删除队列里的号有谱系上的墓碑） 对 7 号判不了、不判红（`.claude/kb/invariants.md` I-9.18 那一行：这一形只让 I-9.5 红）。
 #[test]
 fn a_queue_number_without_an_inode_record_reddens_only_the_deletion_queue_invariant() {
-    let clean =
-        image_after_the_intent_of_a_three_unit_file("deletion-checker-queue-without-record");
-    assert_the_base_image_is_clean(&clean, "意图之后");
+    let clean = image_after_the_intent_of_an_empty_inode("deletion-checker-queue-without-record");
+    assert_the_base_image_is_clean(&clean, "空 inode 2 发完意图之后");
     let mut image = clean.clone();
-    put_an_inode_number_without_a_record_into_the_deletion_queue(&mut image);
+    replace_the_queued_inode_with_a_number_without_a_record(&mut image);
     assert_eq!(
         violated_invariants(&image),
         vec![DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES],
@@ -126,6 +142,11 @@ fn a_queue_number_without_an_inode_record_reddens_only_the_deletion_queue_invari
         detail.contains("deleted_inodes 里的号 7") && detail.contains("没有记录"),
         "违例说明点名队列里那个没有记录的号：{detail}"
     );
+    let reason = reason_not_judged(&image, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE);
+    assert!(
+        reason.contains(NOT_JUDGED_QUEUED_INODE_WITHOUT_A_RECORD),
+        "I-9.18 对队列里没有记录的号判不了、不判红：{reason}"
+    );
 }
 
 /// I-9.5 反方向（C708 那一形，也是判决里说的僵尸记录：nlink 0、单元全放、不在队列）：第一个文件之后建一个空 inode（2 号，没有数据单元），
@@ -266,19 +287,27 @@ fn an_accounting_tree_without_the_pending_deletion_occupancy_row_reddens_only_th
 }
 
 /// I-9.6 墓碑那一半（`.claude/kb/invariants.md` I-9.6 那一行：水位 > 所选根的墓碑树里全部已发布的墓碑记录的对象 ID）：
-/// 意图之后那条墓碑的对象 ID 1 → 2（记账里的 inode 号水位），inode 树里最大的 key 仍是 1——只判「水位 > 树内最大 key」那一半时它全绿。
+/// 3 号删完、根环转过一圈之后那条墓碑的对象 ID 3 → 记账里的 inode 号水位，inode 树里最大的 key 仍小于水位——只判「水位 > 树内最大 key」
+/// 那一半时它全绿。在意图那一版上改会连带 I-9.18（意图那一版的队列 {3} 找不到 3 号的墓碑），所以等意图那一版转出候选集再改
+/// （判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T9、攻方第五节的造法）。
 #[test]
 fn a_tombstone_whose_object_identifier_reaches_the_inode_number_watermark_reddens_only_the_watermark_invariant(
 ) {
-    let clean =
-        image_after_the_intent_of_a_three_unit_file("deletion-checker-tombstone-at-watermark");
-    assert_the_base_image_is_clean(&clean, "意图之后");
+    let pool = pool_with_inode_three_unlinked_and_the_root_ring_turned_over(
+        "deletion-checker-tombstone-at-watermark",
+    );
+    let inode_number_watermark = pool.output.inode_number_watermark();
+    let clean = pool.memory_pool();
+    assert_the_base_image_is_clean(&clean, "3 号删完、根环转过一圈之后");
     let mut image = clean.clone();
-    raise_the_tombstone_object_identifier_to_the_inode_number_watermark(&mut image);
+    raise_the_tombstone_object_identifier_to_the_inode_number_watermark(
+        &mut image,
+        inode_number_watermark,
+    );
     assert_eq!(violated_invariants(&image), vec!["I-9.6"], "只红 I-9.6");
     let detail = first_violation_of(&image, "I-9.6");
     assert!(
-        detail.contains("墓碑记录的最大对象 ID 2"),
+        detail.contains(&format!("墓碑记录的最大对象 ID {inode_number_watermark}")),
         "违例说明写出墓碑那一半：{detail}"
     );
 }
@@ -386,6 +415,7 @@ fn an_intermediate_version_whose_inode_leaf_does_not_read_back_is_not_judged_rat
 
 /// 阳性对照：三个单元的文件按批大小 1 删完（意图、留 2、留 1、最后一批），每一步的镜像上一条违例都没有；
 /// I-9.5、I-3.4、I-4.7、I-4.6 真被评估过且成立（最后一批之后，回退候选集里意图与各批那几版还在，删除中的号仍有对象），
+/// 墓碑与谱系那几条（I-9.10 墓碑那一个等号、I-9.18、I-9.19、I-9.20、I-9.23、I-9.24、I-9.29）也真被评估过且成立，
 /// I-4.4 报不适用、理由照 `.claude/kb/invariants.md` I-4.4 那一行。
 #[test]
 fn every_published_step_of_unlinking_a_three_unit_file_holds_the_deletion_invariants() {
@@ -410,6 +440,13 @@ fn every_published_step_of_unlinking_a_three_unit_file_holds_the_deletion_invari
             INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
             DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
             "I-9.6",
+            "I-9.10",
+            QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
+            TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
+            TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
+            TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+            DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
+            ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
         ] {
             assert_eq!(
                 verdict_of(image, invariant),
@@ -912,8 +949,8 @@ fn an_intermediate_version_with_data_units_of_an_inode_without_a_record_reddens_
 }
 
 /// C708（树里 nlink = 0 的 inode 不在删除队列里，池级 checker 判不出） 还差的那一形：I-9.5 正方向「队列里的号在树里 nlink ≠ 0」。
-/// 意图之后队列 {1} 不动，1 号那条记录的 nlink 0 → 1。这一形必与 I-9.18（删除队列里的号有谱系上的墓碑）、I-9.19（谱系上的墓碑不指活对象）
-/// 同红（设计轮判决第二节最后一行），那两条这一版 checker 还没实现（不在 `IMPLEMENTED_INVARIANTS` 里），不在判红集合里。
+/// 意图之后队列 {1} 不动，1 号那条记录的 nlink 0 → 1。按条款同红 I-9.19（谱系上的墓碑不指活对象）：墓碑 (1, 1 号的出生代) 在谱系上而那条记录
+/// nlink 1（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T9 那一行）；I-9.18 只要有那条墓碑、不看 nlink，照旧成立。
 #[test]
 fn a_queue_number_whose_inode_record_has_nlink_one_reddens_the_deletion_queue_invariant() {
     let clean =
@@ -928,8 +965,17 @@ fn a_queue_number_whose_inode_record_has_nlink_one_reddens_the_deletion_queue_in
     );
     assert_eq!(
         violated,
-        vec![DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES],
-        "这一版 checker 上判红的只有 I-9.5"
+        vec![
+            DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+            TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT
+        ],
+        "判红的是 I-9.5 与 I-9.19"
+    );
+    let live_object_detail =
+        first_violation_of(&image, TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT);
+    assert!(
+        live_object_detail.contains("号 1") && live_object_detail.contains("nlink 1 的记录"),
+        "I-9.19 的说明点名那条墓碑与活着的记录：{live_object_detail}"
     );
     let detail = first_violation_of(
         &image,
@@ -940,3 +986,299 @@ fn a_queue_number_whose_inode_record_has_nlink_one_reddens_the_deletion_queue_in
         "违例说明点名队列里那个号与它的 nlink：{detail}"
     );
 }
+
+/// I-9.24（删除队列至多一条 key） 只红自己的那一形（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T8）：
+/// 一次建了两个空 inode（2、3 号）、发完 2 号的意图之后，把 3 号也做成一个合法的删除中号——队列 {2, 3}、3 号记录 nlink 0、墓碑容器里
+/// 追加 3 号的整对象墓碑（死于意图那次）。I-9.5 两个方向、I-4.7、I-4.6、I-9.18、I-9.19、I-9.20 都成立，只有队列多于一条 key。
+/// 干净的那一版（队列 {2}）I-9.24 真被评估过且成立。
+#[test]
+fn a_deletion_queue_of_two_inodes_being_deleted_with_their_tombstones_reddens_only_the_at_most_one_key_invariant(
+) {
+    let mut pool = pool_with_new_inodes("deletion-checker-two-keys-in-the-queue", 2);
+    publish_the_intent_of(&mut pool, 2);
+    let clean = pool.memory_pool();
+    assert_the_base_image_is_clean(&clean, "一次建了两个空 inode、发完 2 号的意图之后");
+    assert_eq!(
+        verdict_of(&clean, DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY),
+        InvariantVerdict::Holds,
+        "队列 {{2}} 的那一版 I-9.24 真被评估过且成立"
+    );
+    let mut image = clean.clone();
+    put_a_second_inode_being_deleted_into_the_deletion_queue(&mut image);
+    assert_eq!(
+        violated_invariants(&image),
+        vec![DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY],
+        "只红 I-9.24"
+    );
+    let detail = first_violation_of(&image, DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY);
+    assert!(
+        detail.contains("有 2 条 key"),
+        "违例说明写出队列里有几条 key：{detail}"
+    );
+}
+
+/// I-9.19（谱系上的墓碑不指活对象） 只红自己的那一形（攻方 `research/prompts/m3-delete-checker-r2-opus-output.md` 第五节第 7 行）：
+/// 2、3 号分两次建（对象出生代不同），3 号删完、根环转过一圈之后，把 3 号那条墓碑的 (号, 对象出生代) 改成活着的 2 号的。
+/// I-9.10 墓碑那一个等号按号认、出生代相同，成立；队列空，I-9.18 没有对象；I-9.20 不重号；2 < 水位。
+#[test]
+fn a_tombstone_on_the_lineage_naming_a_live_inode_reddens_only_the_live_object_invariant() {
+    let pool = pool_with_inode_three_unlinked_and_the_root_ring_turned_over(
+        "deletion-checker-tombstone-naming-a-live-inode",
+    );
+    let object_birth_of_inode_two = object_birth_of(&pool.output, 2).expect("2 号活着").0;
+    let clean = pool.memory_pool();
+    assert_the_base_image_is_clean(&clean, "3 号删完、根环转过一圈之后");
+    let mut image = clean.clone();
+    change_the_first_tombstone_of_the_newest_root(&mut image, |tombstone| {
+        assert_eq!(tombstone_field(tombstone, 0), 3, "那条墓碑的对象 ID 是 3");
+        set_tombstone_field(tombstone, 0, 2);
+        set_tombstone_field(
+            tombstone,
+            TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET,
+            object_birth_of_inode_two,
+        );
+    });
+    assert_eq!(
+        violated_invariants(&image),
+        vec![TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT],
+        "只红 I-9.19"
+    );
+    let detail = first_violation_of(&image, TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT);
+    assert!(
+        detail.contains("号 2") && detail.contains("nlink 1 的记录"),
+        "违例说明点名那条墓碑与活着的记录：{detail}"
+    );
+}
+
+/// I-9.20（谱系上的墓碑不重号且记录合法） 区间那一句只红自己的那一形（攻方第五节第 8 行）：3 号删完、根环转过一圈之后，
+/// 那条墓碑的区间止 2⁶⁴ − 1 → 5（第一版只许整对象）。
+#[test]
+fn a_tombstone_that_does_not_cover_the_whole_object_reddens_only_the_tombstone_record_invariant() {
+    let pool = pool_with_inode_three_unlinked_and_the_root_ring_turned_over(
+        "deletion-checker-tombstone-range-not-whole",
+    );
+    let clean = pool.memory_pool();
+    assert_the_base_image_is_clean(&clean, "3 号删完、根环转过一圈之后");
+    let mut image = clean.clone();
+    change_the_first_tombstone_of_the_newest_root(&mut image, |tombstone| {
+        assert_eq!(
+            tombstone_field(tombstone, TOMBSTONE_RECORD_RANGE_END_OFFSET),
+            u64::MAX,
+            "整对象的区间止是 2⁶⁴ − 1"
+        );
+        set_tombstone_field(tombstone, TOMBSTONE_RECORD_RANGE_END_OFFSET, 5);
+    });
+    assert_eq!(
+        violated_invariants(&image),
+        vec![TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED],
+        "只红 I-9.20"
+    );
+    let detail = first_violation_of(
+        &image,
+        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
+    );
+    assert!(
+        detail.contains("[0, 5]") && detail.contains("不是整对象"),
+        "违例说明写出那条墓碑的区间：{detail}"
+    );
+}
+
+/// I-9.20 死亡 txg 那一句（第四节用户定案「补」：对象出生代 ≤ 死亡 txg ≤ 这一版的 txg；攻方第五节「打中五」）只红自己的那一形：
+/// 3 号删完、根环转过一圈之后，那条墓碑的死亡 txg 改成 0（早于对象出生代）。0 在每一版的父链上，I-9.18、I-9.19 都看不出这一形。
+#[test]
+fn a_tombstone_that_died_before_its_object_was_born_reddens_only_the_tombstone_record_invariant() {
+    let pool = pool_with_inode_three_unlinked_and_the_root_ring_turned_over(
+        "deletion-checker-tombstone-dies-before-birth",
+    );
+    let clean = pool.memory_pool();
+    assert_the_base_image_is_clean(&clean, "3 号删完、根环转过一圈之后");
+    let mut image = clean.clone();
+    change_the_first_tombstone_of_the_newest_root(&mut image, |tombstone| {
+        set_tombstone_field(tombstone, TOMBSTONE_RECORD_DEATH_TXG_OFFSET, 0);
+    });
+    assert_eq!(
+        violated_invariants(&image),
+        vec![TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED],
+        "只红 I-9.20"
+    );
+    let detail = first_violation_of(
+        &image,
+        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
+    );
+    assert!(
+        detail.contains("死亡 txg 0 不在"),
+        "违例说明写出那条墓碑的死亡 txg：{detail}"
+    );
+}
+
+/// I-9.20 不重号那一句只红自己的那一形：3 号删完、根环转过一圈之后，墓碑容器里那条墓碑原样再追加一条（两条都在谱系上）。
+#[test]
+fn two_tombstones_of_one_object_on_the_lineage_redden_only_the_tombstone_record_invariant() {
+    let pool = pool_with_inode_three_unlinked_and_the_root_ring_turned_over(
+        "deletion-checker-tombstone-twice-on-the-lineage",
+    );
+    let clean = pool.memory_pool();
+    assert_the_base_image_is_clean(&clean, "3 号删完、根环转过一圈之后");
+    let mut image = clean.clone();
+    append_a_copy_of_the_first_tombstone_of_the_newest_root(&mut image);
+    assert_eq!(
+        violated_invariants(&image),
+        vec![TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED],
+        "只红 I-9.20"
+    );
+    let detail = first_violation_of(
+        &image,
+        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
+    );
+    assert!(
+        detail.contains("号 3") && detail.contains("不止一条"),
+        "违例说明点名重复的那一对：{detail}"
+    );
+}
+
+/// I-9.10（对象出生代三处一致） 墓碑那一个等号只红自己的那一形（判决第二节 T2 取 a：按号认、每版各比自己的）：2 号 unlink 到底、
+/// 回退到发意图之前（2 号又活着），根环转过一圈之后，那条墓碑（死于被回退抛弃的那一段，谱系外）的对象出生代减一。
+/// 只取谱系上的墓碑（T2 的 c）看不见它；墓碑在谱系外，I-9.29 不算它、I-9.19 不判它。
+#[test]
+fn a_tombstone_off_the_lineage_whose_object_birth_differs_from_the_live_record_reddens_only_the_object_birth_invariant(
+) {
+    let pool =
+        pool_with_inode_two_unlinked_then_rolled_back_before_the_intent_and_the_root_ring_turned_over(
+            "deletion-checker-tombstone-off-the-lineage-birth",
+        );
+    let clean = pool.memory_pool();
+    assert_the_base_image_is_clean(&clean, "2 号删完、回退到意图之前、根环转过一圈之后");
+    assert_eq!(
+        verdict_of(&clean, "I-9.10"),
+        InvariantVerdict::Holds,
+        "回退之后 2 号活着、谱系外那条墓碑的出生代与它相同：I-9.10 成立"
+    );
+    let mut image = clean.clone();
+    change_the_first_tombstone_of_the_newest_root(&mut image, |tombstone| {
+        assert_eq!(tombstone_field(tombstone, 0), 2, "那条墓碑的对象 ID 是 2");
+        let object_birth = tombstone_field(tombstone, TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET);
+        set_tombstone_field(
+            tombstone,
+            TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET,
+            object_birth - 1,
+        );
+    });
+    assert_eq!(violated_invariants(&image), vec!["I-9.10"], "只红 I-9.10");
+    let detail = first_violation_of(&image, "I-9.10");
+    assert!(
+        detail.contains("墓碑（号 2"),
+        "违例说明点名那条墓碑：{detail}"
+    );
+}
+
+/// I-9.18（删除队列里的号有谱系上的墓碑） 只红自己的那一形（攻方第五节「打中五」之后的造法：死亡 txg 改成被回退抛弃那一段里的一个 txg，
+/// 在 [对象出生代, 这一版的 txg] 里、不在谱系上）：建 2 号，再建 3 号（那一版 X），回退到建 3 号之前，发 2 号的意图；
+/// 那条墓碑的死亡 txg 改成 X 的 txg。I-9.20 的「对象出生代 ≤ 死亡 txg ≤ 这一版的 txg」照旧成立，墓碑在谱系外、I-9.19 与 I-9.29 不看它。
+#[test]
+fn a_queued_inode_whose_tombstone_died_on_a_branch_a_rollback_left_reddens_only_the_queued_tombstone_invariant(
+) {
+    let mut pool = pool_with_new_inodes("deletion-checker-queued-tombstone-off-the-lineage", 1);
+    let before_the_abandoned_branch = pool.output.clone();
+    let on_the_abandoned_branch = publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+    roll_back_to(&mut pool, &before_the_abandoned_branch);
+    publish_the_intent_of(&mut pool, 2);
+    let clean = pool.memory_pool();
+    assert_the_base_image_is_clean(&clean, "回退之后发了 2 号的意图");
+    assert_eq!(
+        verdict_of(&clean, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE),
+        InvariantVerdict::Holds,
+        "队列 {{2}} 的那一版 I-9.18 真被评估过且成立"
+    );
+    let txg_on_the_abandoned_branch = on_the_abandoned_branch.root.checkpoint_txg.0;
+    let mut image = clean.clone();
+    change_the_first_tombstone_of_the_newest_root(&mut image, |tombstone| {
+        assert_eq!(tombstone_field(tombstone, 0), 2, "那条墓碑的对象 ID 是 2");
+        set_tombstone_field(
+            tombstone,
+            TOMBSTONE_RECORD_DEATH_TXG_OFFSET,
+            txg_on_the_abandoned_branch,
+        );
+    });
+    assert_eq!(
+        violated_invariants(&image),
+        vec![QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE],
+        "只红 I-9.18"
+    );
+    let detail = first_violation_of(&image, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE);
+    assert!(
+        detail.contains("deleted_inodes 里的号 2") && detail.contains("没有在谱系上的整对象墓碑"),
+        "违例说明点名队列里那个号：{detail}"
+    );
+}
+
+/// 「在谱系上」每一版从自己的 txg 与自己的谱系树起算（判决第二节 T3 取 a；攻方第一节「打中一」「打中二」那两段合法历史）：
+/// 2 号 unlink 到底，回退到发意图之前，再回退到删完那一版。第一次回退之后，意图那一版（队列 {2}）还在候选集里，它自己的谱系树里
+/// 还没有这次回退那条记录，墓碑在它的链上，I-9.18 成立（一律从所选根起算会在这里误红）；第二次回退之后，第一次回退那一版（2 号活着）
+/// 自己的父链跳过意图那次的 txg，I-9.19 不判它（一律从所选根起算、或不按谱系滤会在这里误红）。两步都一条违例都没有。
+#[test]
+fn unlinking_then_rolling_back_before_the_intent_and_back_to_the_emptied_version_holds_every_lineage_invariant(
+) {
+    let mut pool = pool_with_new_inodes("deletion-checker-unlink-and-two-rollbacks", 1);
+    let before_the_intent = pool.output.clone();
+    unlink_to_the_end(&mut pool, 2);
+    let emptied = pool.output.clone();
+    roll_back_to(&mut pool, &before_the_intent);
+    let after_the_rollback_before_the_intent = pool.memory_pool();
+    assert_eq!(
+        violated_invariants(&after_the_rollback_before_the_intent),
+        Vec::<&str>::new(),
+        "回退到意图之前：没有违例"
+    );
+    assert_eq!(
+        verdict_of(
+            &after_the_rollback_before_the_intent,
+            QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE
+        ),
+        InvariantVerdict::Holds,
+        "意图那一版还在候选集里，I-9.18 真被评估过且成立"
+    );
+    roll_back_to(&mut pool, &emptied);
+    let after_the_rollback_to_the_emptied_version = pool.memory_pool();
+    assert_eq!(
+        violated_invariants(&after_the_rollback_to_the_emptied_version),
+        Vec::<&str>::new(),
+        "再回退到删完那一版：没有违例"
+    );
+    assert_eq!(
+        verdict_of(
+            &after_the_rollback_to_the_emptied_version,
+            TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT
+        ),
+        InvariantVerdict::Holds,
+        "意图那一版里墓碑指着 nlink 0 的记录，I-9.19 真被评估过且成立"
+    );
+}
+
+/// 「删 → 回退到删之前 → 再删」的合法历史（`.claude/kb/invariants.md` I-9.20 那一行：不按谱系过滤的写法在这一段上判红）：墓碑树里有两条
+/// (2, 2 号的出生代)，第一条死于被回退抛弃的那一段、不在谱系上，不重号那一句只判谱系上的，成立；一条违例都没有。
+#[test]
+fn unlinking_the_same_inode_again_after_a_rollback_before_its_intent_holds_the_tombstone_record_invariant(
+) {
+    let mut pool = pool_with_new_inodes("deletion-checker-unlink-rollback-unlink", 1);
+    let before_the_intent = pool.output.clone();
+    unlink_to_the_end(&mut pool, 2);
+    roll_back_to(&mut pool, &before_the_intent);
+    unlink_to_the_end(&mut pool, 2);
+    let image = pool.memory_pool();
+    assert_eq!(
+        violated_invariants(&image),
+        Vec::<&str>::new(),
+        "删、回退、再删：没有违例"
+    );
+    for invariant in [
+        "I-9.10",
+        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
+        ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
+    ] {
+        assert_eq!(
+            verdict_of(&image, invariant),
+            InvariantVerdict::Holds,
+            "{invariant} 真被评估过且成立"
+        );
+    }
+}
diff --git a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
index 5272dad..e934c34 100644
--- a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
+++ b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
@@ -555,8 +555,8 @@ fn region_device(txg: u64) -> DeviceIdentity {
     parameters().region_devices[usize::try_from(target.region).expect("区域号")]
 }
 
-/// 最新的根下面还没有记账树、inode 树与分配记录树时（mkfs 之后、零单元发布之后）checker 报不适用的那 36 条；
-/// 其余 38 条要真被评估过且成立（条数按今天的 `IMPLEMENTED_INVARIANTS` 74 条数）。映射 key 与单元头相符（`MAPPING_KEY_MATCHES_THE_UNIT_HEADER`）报不适用是因为中央映射树的根指针全零，
+/// 最新的根下面还没有记账树、inode 树与分配记录树时（mkfs 之后、零单元发布之后）checker 报不适用的那 45 条；
+/// 其余 38 条要真被评估过且成立（条数按今天的 `IMPLEMENTED_INVARIANTS` 83 条数）。映射 key 与单元头相符（`MAPPING_KEY_MATCHES_THE_UNIT_HEADER`）报不适用是因为中央映射树的根指针全零，
 /// 一条映射条目都没有。I-3.11（已分配减 defer 等于最新根走读） 与 I-3.1（已分配统计对得上） 同一个理由：最新根下面没有记账树。I-9.14（树表条目的诞生 txg 跨根不变） 在这里报不适用是因为 mkfs 种的第 0 版树表是空的：
 /// 一棵树的条目都没有，跨根比不出来（有了文件版本之后也只有一条根有条目，见 `NOT_APPLICABLE_WITH_ONE_FILE_VERSION`）；
 /// I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 同一个理由，比的是根环里全部读得出的根（不按实例表判有效）：根环里每条根的树表都是
@@ -578,7 +578,10 @@ fn region_device(txg: u64) -> DeviceIdentity {
 /// 删除线那五条（里程碑三·3）五张没有文件的表都有：I-9.5（删除队列里的号仍在树里） 与 I-3.4（可用空间扣待删占用） 要树表里的
 /// `deleted_inodes`、inode 树与记账树，每一版树表都是 0 条（第一个文件版本之后两条都真被评估过且成立）；I-4.7（删除中 inode 有账） 与
 /// I-4.6（中间态其余不变量成立） 没有处于删除中的号；I-4.4（意图只描述目标态） 第一版恒报不适用（下面每一张表都有它）。
-const NOT_APPLICABLE_WITHOUT_FILE: [&str; 36] = [
+/// 墓碑与谱系那九条（里程碑三·3，第 22 批）五张没有文件的表都有：I-9.17–I-9.23 要墓碑树、谱系树（每一版树表都是 0 条，有了文件版本也是零根，
+/// 下面两张有文件的表照样有这七条），I-9.24（删除队列至多一条 key） 要 `deleted_inodes` 条目、I-9.29（同号只许一个出生代） 要 inode 记录
+/// （第一个文件版本之后两条都真被评估过且成立）。
+const NOT_APPLICABLE_WITHOUT_FILE: [&str; 45] = [
     "I-9.27",
     "I-5.5",
     "I-1.10",
@@ -615,6 +618,15 @@ const NOT_APPLICABLE_WITHOUT_FILE: [&str; 36] = [
     "I-4.7",
     "I-4.6",
     "I-4.4",
+    "I-9.17",
+    "I-9.18",
+    "I-9.19",
+    "I-9.20",
+    "I-9.21",
+    "I-9.22",
+    "I-9.23",
+    "I-9.24",
+    "I-9.29",
 ];
 
 /// mkfs 刚写完那一份比 `NOT_APPLICABLE_WITHOUT_FILE` 多报四条不适用：journal 环整段是 0，一条自证过的记录都没有 ⇒ I-8.6（反向链算法）、
@@ -622,8 +634,8 @@ const NOT_APPLICABLE_WITHOUT_FILE: [&str; 36] = [
 /// 可写挂载之后环里有记录（写行 / 暖机的空发布），它们就真被评估过了——所以这四条只在 mkfs 之后那一格。
 /// I-3.5（引用区间的精确性） 两张表都有，这一格的理由是回退候选集只剩 mkfs 的第 0 代根一版（环里没有记录、根环里只有它），
 /// 凑不出相邻的一对。
-/// 删除线那五条的理由同 `NOT_APPLICABLE_WITHOUT_FILE`。
-const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 40] = [
+/// 删除线那五条与墓碑与谱系那九条的理由同 `NOT_APPLICABLE_WITHOUT_FILE`。
+const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 49] = [
     "I-9.27",
     "I-5.5",
     "I-1.10",
@@ -664,6 +676,15 @@ const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 40] = [
     "I-4.7",
     "I-4.6",
     "I-4.4",
+    "I-9.17",
+    "I-9.18",
+    "I-9.19",
+    "I-9.20",
+    "I-9.21",
+    "I-9.22",
+    "I-9.23",
+    "I-9.24",
+    "I-9.29",
 ];
 
 /// 写完第一个文件版本之后仍报不适用的那六条（I-4.1 的理由见 `NOT_APPLICABLE_WITHOUT_FILE`）：
@@ -677,15 +698,18 @@ const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 40] = [
 /// 三条比得出来的镜像都在 `checker_known_bad_images.rs`（再覆盖写一次：树表两版、事务号 1 与 2；
 /// B 之后再接一事务两条记录的阳性对照与坏镜像）。
 /// I-4.7（删除中 inode 有账） 与 I-4.6（中间态其余不变量成立）——没有删除，没有处于删除中的号、没有 nlink = 0 的记录
-/// （阳性对照与坏镜像在 `pool_checker_judges_the_deletion_state_of_every_walked_version.rs`）；I-4.4（意图只描述目标态） 第一版恒报不适用。
-const NOT_APPLICABLE_WITH_ONE_FILE_VERSION: [&str; 9] = [
-    "I-7.9", "I-8.7", "I-8.8", "I-9.14", "I-4.1", "I-9.28", "I-4.7", "I-4.6", "I-4.4",
+/// （阳性对照与坏镜像在 `pool_checker_judges_the_deletion_state_of_every_walked_version.rs`）；I-4.4（意图只描述目标态） 第一版恒报不适用；
+/// I-9.17–I-9.23——没有删除、没有回退，墓碑树与谱系树都是零根。
+const NOT_APPLICABLE_WITH_ONE_FILE_VERSION: [&str; 16] = [
+    "I-7.9", "I-8.7", "I-8.8", "I-9.14", "I-4.1", "I-9.28", "I-4.7", "I-4.6", "I-4.4", "I-9.17",
+    "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23",
 ];
 
 /// 只有实例 1 的池上第一个文件版本之后：比 `NOT_APPLICABLE_WITH_ONE_FILE_VERSION` 多 I-3.12（实例表行 T 低于引用它的根的 txg）——
 /// 实例表一行都没有（写过行的那一份上它真被评估过且成立）。
-const NOT_APPLICABLE_WITH_ONE_FILE_VERSION_OF_THE_ONLY_INSTANCE: [&str; 10] = [
+const NOT_APPLICABLE_WITH_ONE_FILE_VERSION_OF_THE_ONLY_INSTANCE: [&str; 17] = [
     "I-7.9", "I-8.7", "I-8.8", "I-9.14", "I-3.12", "I-4.1", "I-9.28", "I-4.7", "I-4.6", "I-4.4",
+    "I-9.17", "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23",
 ];
 
 /// 树表 0 条的一版上写行那次发布之后：比 `NOT_APPLICABLE_WITHOUT_FILE` 少七条——I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）、
@@ -700,8 +724,8 @@ const NOT_APPLICABLE_WITH_ONE_FILE_VERSION_OF_THE_ONLY_INSTANCE: [&str; 10] = [
 /// 判点名项与单元头逐字段相同——两条都真被评估过且成立。
 /// I-3.5（引用区间的精确性） 也不在这张表里：写行那次发布（实例 2、txg 3）把实例表 COW 到新落点、写了自己那棵分配记录树，
 /// 这几个单元前一版（实例 1 的暖机根，txg 2）不引用、诞生代号 3 大于 2，出现规则真判到了它们且成立。
-/// 删除线那五条的理由同 `NOT_APPLICABLE_WITHOUT_FILE`。
-const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 29] = [
+/// 删除线那五条与墓碑与谱系那九条的理由同 `NOT_APPLICABLE_WITHOUT_FILE`。
+const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 38] = [
     "I-9.27",
     "I-5.5",
     "I-3.1",
@@ -731,6 +755,15 @@ const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 29] = [
     "I-4.7",
     "I-4.6",
     "I-4.4",
+    "I-9.17",
+    "I-9.18",
+    "I-9.19",
+    "I-9.20",
+    "I-9.21",
+    "I-9.22",
+    "I-9.23",
+    "I-9.24",
+    "I-9.29",
 ];
 
 /// 这一步的镜像上 checker 的每一条判决逐条核：`not_applicable` 里的报不适用，其余每一条都真被评估过且成立（一条违例都没有）。
@@ -1115,9 +1148,9 @@ fn plant_a_root_without_its_allocation_record_tree(
 /// 验收：取号 1、不写行；写行那次发布 (1, 1)、jsn 1、事务号 0、反向链 0、零单元、落盘 1；暖机一次 (1, 2)、jsn 2、反向链接 jsn 1、落盘 0；
 /// 分配器只有 mkfs 的两个单元（每盘 3 槽）；第一个文件版本 (1, 3)、jsn 3、事务号 1，数据单元落 50180、换下 mkfs 那片树表；
 /// mkfs 之后的录制流与新池新建文件那条逐字节相同、写出的那一版也相同；冷启动读回文件；checker 一条违例都没有（条数按今天的
-/// `IMPLEMENTED_INVARIANTS` 74 条数）——mkfs 之后 34 条成立（环是空的，I-8.6、I-8.9、I-8.10、I-8.12 也报不适用）、挂载之后 38 条成立，
-/// 没有记账树 / inode 树 / 分配记录树 / 映射条目的那几条报不适用，第一个文件版本之后 64 条成立、I-7.9、I-9.14、I-9.28、I-8.7、I-8.8、
-/// I-3.12、I-4.1、I-4.7、I-4.6 与 I-4.4 仍报不适用（树表条目只出现在这一条根的树表里；同一实例只有一条非 0 事务号的记录；没有哪个事务号落了两条记录、
+/// `IMPLEMENTED_INVARIANTS` 83 条数）——mkfs 之后 34 条成立（环是空的，I-8.6、I-8.9、I-8.10、I-8.12 也报不适用）、挂载之后 38 条成立，
+/// 没有记账树 / inode 树 / 分配记录树 / 映射条目的那几条报不适用，第一个文件版本之后 66 条成立、I-7.9、I-9.14、I-9.28、I-8.7、I-8.8、
+/// I-3.12、I-4.1、I-4.7、I-4.6、I-4.4 与墓碑树、谱系树那七条（I-9.17–I-9.23）仍报不适用（树表条目只出现在这一条根的树表里；同一实例只有一条非 0 事务号的记录；没有哪个事务号落了两条记录、
 /// 每条都带提交标记；只有实例 1，实例表一行都没有；最新根之上没有记录；没有删除；I-4.4 第一版恒报不适用）。
 #[test]
 fn writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold(
````

## 实现员 c：mutations-append.tsv（/tmp/claude-1000/m3-batch22/c/patch/mutations-append.tsv）

````text
第22批 C I-9.23：墓碑树条目身份比较恒真（身份不符不再判红）	crates/singlefs-checker/src/walk.rs	let container_entry_holds = unit[6] == 3	let container_entry_holds = true || unit[6] == 3	-p singlefs-harness --test checker_known_bad_images	a_tombstone_tree_entry_whose_identity_names_another_container_reddens_only_the_tombstone_entry_invariant
第22批 C I-9.17：谱系树条目身份比较恒真（身份不符不再判红）	crates/singlefs-checker/src/walk.rs	let container_entry_holds = unit[6] == 3	let container_entry_holds = true || unit[6] == 3	-p singlefs-harness --test checker_known_bad_images	a_lineage_tree_entry_whose_identity_names_another_container_reddens_only_the_lineage_entry_invariant
第22批 C I-9.23：类型段不认得的条目按一槽登记引用（层级 1 的孩子是两槽的容器，I-3.1 连带红）	crates/singlefs-checker/src/walk.rs	let span_in_slots = if node.level >= 2 { 1 } else { 2 };	let span_in_slots = if node.level >= 2 { 1 } else { 1 };	-p singlefs-harness --test checker_known_bad_images	a_tombstone_tree_entry_with_an_unregistered_type_reddens_only_the_tombstone_entry_invariant
第22批 C I-9.4 第四句：同一片容器里记录号只要求不减（同号两条放过）	crates/singlefs-checker/src/walk.rs	judge("I-9.4", inode > previous_inode	judge("I-9.4", inode >= previous_inode	-p singlefs-harness --test checker_known_bad_images	a_leaf_holding_the_same_inode_number_twice_reddens_only_the_container_number_invariant
第22批 C I-1.7：谱系记录宽登记成 15	crates/singlefs-checker/src/walk.rs	PACKED_TYPE_LINEAGE => Some(16),	PACKED_TYPE_LINEAGE => Some(15),	-p singlefs-harness --test checker_known_bad_images	a_lineage_container_whose_record_width_is_not_sixteen_reddens_only_the_packed_container_invariant
第22批 C I-1.7：墓碑记录宽登记成 39	crates/singlefs-checker/src/walk.rs	PACKED_TYPE_TOMBSTONE => Some(40),	PACKED_TYPE_TOMBSTONE => Some(39),	-p singlefs-harness --test checker_known_bad_images	a_tombstone_container_whose_record_width_is_not_forty_reddens_only_the_packed_container_invariant
第22批 C I-9.18：队列号的墓碑不看在不在谱系上	crates/singlefs-checker/src/walk.rs	&& lineage_chain.holds_txg(tombstone.death_txg)\n	&& (lineage_chain.holds_txg(tombstone.death_txg) || true)\n	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	a_queued_inode_whose_tombstone_died_on_a_branch_a_rollback_left_reddens_only_the_queued_tombstone_invariant
第22批 C T3：父链从这一版的 txg 减一起算（这一版自己那次发布的墓碑不在链上）	crates/singlefs-checker/src/walk.rs	let mut txg_walked_to = version_txg;	let mut txg_walked_to = version_txg.saturating_sub(1);	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	unlinking_then_rolling_back_before_the_intent_and_back_to_the_emptied_version_holds_every_lineage_invariant
第22批 C T3：「在谱系上」恒真（不按谱系过滤，删、回退、再删上 I-9.20 误红）	crates/singlefs-checker/src/walk.rs	(*lowest_txg..=*highest_txg).contains(&txg)	(*lowest_txg..=*highest_txg).contains(&txg) || true	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	unlinking_the_same_inode_again_after_a_rollback_before_its_intent_holds_the_tombstone_record_invariant
第22批 C T3：「在谱系上」恒真（不按谱系过滤，回退到删之前那一版上 I-9.19 误红）	crates/singlefs-checker/src/walk.rs	(*lowest_txg..=*highest_txg).contains(&txg)	(*lowest_txg..=*highest_txg).contains(&txg) || true	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	unlinking_then_rolling_back_before_the_intent_and_back_to_the_emptied_version_holds_every_lineage_invariant
第22批 C I-9.19：谱系上的墓碑指着 nlink 1 的记录也放过	crates/singlefs-checker/src/walk.rs	            record.nlink == 0,	            record.nlink <= 1,	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	a_tombstone_on_the_lineage_naming_a_live_inode_reddens_only_the_live_object_invariant
第22批 C I-9.20：区间止不比 2⁶⁴ − 1	crates/singlefs-checker/src/walk.rs	&& self.range_end == TOMBSTONE_WHOLE_OBJECT_RANGE_END	&& self.range_end <= TOMBSTONE_WHOLE_OBJECT_RANGE_END	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	a_tombstone_that_does_not_cover_the_whole_object_reddens_only_the_tombstone_record_invariant
第22批 C I-9.20：死亡 txg 不比对象出生代	crates/singlefs-checker/src/walk.rs	tombstone.object_birth <= tombstone.death_txg && tombstone.death_txg <= version_txg;	tombstone.death_txg <= version_txg;	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	a_tombstone_that_died_before_its_object_was_born_reddens_only_the_tombstone_record_invariant
第22批 C I-9.20：谱系上同一对 (号, 出生代) 两条也放过	crates/singlefs-checker/src/walk.rs	is_the_first_on_the_lineage,	is_the_first_on_the_lineage || true,	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	two_tombstones_of_one_object_on_the_lineage_redden_only_the_tombstone_record_invariant
第22批 C I-9.10 墓碑那一个等号：墓碑出生代只要不大于记录的	crates/singlefs-checker/src/walk.rs	judge("I-9.10", record.object_birth == tombstone.object_birth	judge("I-9.10", record.object_birth >= tombstone.object_birth	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	a_tombstone_off_the_lineage_whose_object_birth_differs_from_the_live_record_reddens_only_the_object_birth_invariant
第22批 C I-9.24：队列放过两条 key	crates/singlefs-checker/src/walk.rs	queue.len() <= 1	queue.len() <= 2	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	a_deletion_queue_of_two_inodes_being_deleted_with_their_tombstones_reddens_only_the_at_most_one_key_invariant
第22批 C I-9.18：队列里没有记录的号判红而不是判不了	crates/singlefs-checker/src/walk.rs	            judgements.part_of_the_range_not_judged(\n                invariant,\n                DeletionScanGap::QueuedInodeWithoutARecord.reason(),\n            );	            judgements.judge(invariant, false, String::new);	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	a_queue_number_without_an_inode_record_reddens_only_the_deletion_queue_invariant
第22批 C I-9.21：非最右墓碑容器只要求不超 815	crates/singlefs-checker/src/walk.rs	record_count == TOMBSTONE_CONTAINER_RECORDS	record_count <= TOMBSTONE_CONTAINER_RECORDS	-p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full	a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant
第22批 C I-9.22：墓碑容器按产品容量 815 比、不按旋钮值	crates/singlefs-checker/src/walk.rs	record_count <= knob_records	record_count <= TOMBSTONE_CONTAINER_RECORDS	-p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full	a_tombstone_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant
第22批 C I-9.29：同号几个出生代也放过	crates/singlefs-checker/src/walk.rs	births.len() == 1	!births.is_empty()	-p singlefs-harness --test inode_number_watermark_pressed_back_reissues_a_number	pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_only_the_one_birth_per_number_invariant
````

## 实现员 c：mutations-replacements.tsv（/tmp/claude-1000/m3-batch22/c/patch/mutations-replacements.tsv）

````text
第8批 走读：墓碑树、谱系树容器的落点记在外面那片树表下面（共享根节点的老一版并不进容器，抬 F 的合法镜像上 I-3.5 误红）	crates/singlefs-checker/src/walk.rs	        let previous_parent_unit = self.start_walking_below(node_placement);\n        self.walk_the_entries_of_a_container_tree_node(\n            node,\n            tree,\n            record_type,\n            identity_invariant,\n            name,\n        );\n        self.stop_walking_below(previous_parent_unit);	        self.walk_the_entries_of_a_container_tree_node(\n            node,\n            tree,\n            record_type,\n            identity_invariant,\n            name,\n        );\n        let _ = node_placement;	-p singlefs-harness --test checker_known_bad_images	raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant
````

## 实现员 t：crates.patch（/tmp/claude-1000/m3-batch22/t/patch/crates.patch）

````diff
diff --git a/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs b/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs
--- a/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs
+++ b/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs
@@ -6,6 +6,9 @@
 //! 「每批取 max(144, ⌈剩下 ÷ 3⌉)」那种批数封顶的写法在 432 个单元以内与产品逐项相同，433 个单元第一批就换下 145 个
 //! （判决 `research/prompts/m3-batch19-code-r1-main-verification.md` 第二节 K1 N6 那一行，攻方 `research/prompts/m3-batch19-code-r1-opus-output.md`
 //! 「N6」一节）。
+//! 建池的几点罩不住批数封顶在 32 以上的那一族（4608 个单元以内与产品逐项相同），另一条用例不建池、直接比产品那一步
+//! `unlink::next_unlink_step` 在剩下 145 到 2⁴⁰ 个单元上都只换下一批（判决 `research/prompts/m3-batch21-code-r1-main-verification.md`
+//! 第二节「K2 N7 每批换下数随规模变」那一行，攻方 `research/prompts/m3-batch21-code-r1-opus-output.md` 第一节）。
 
 mod common;
 
@@ -18,7 +21,7 @@
     publish_unlink_intent, publish_unlink_step, PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
 };
 use singlefs_core::unit::data_unit_payload_capacity;
-use singlefs_core::unlink::UnlinkBatchSize;
+use singlefs_core::unlink::{next_unlink_step, UnlinkBatchSize, UnlinkStep};
 use singlefs_format::{
     EXTENT_TREE_LOWER_LEAF_DATA_UNITS, JOURNAL_NAMED_ENTRIES_PER_RECORD, ROOT_RING_REGIONS,
     SLOT_BYTES,
@@ -30,7 +33,7 @@
 const INSTANCE_OF_THE_PROCESS: InstanceGeneration = InstanceGeneration(1);
 
 /// 测试池的盘数：每个单元每块盘各写一份（`admission::UnitsOfAPublishToLand` 的文档：第一版每个单元落池里每一块盘），
-/// journal 记录这里也按每块盘一份算（往多里算）。
+/// journal 记录这里也按每块盘一份算（往多里算）。上界那一处现核它等于建出来的池的盘数（镜像几块、分配器管几块盘）。
 const DEVICES_OF_THE_TEST_POOL: u64 = 2;
 
 /// 任一份文件里写调用最多的那次发布，至多比一个单元的文件里写调用最多的那次多几个写调用：从代码算，不随文件的单元数长。
@@ -45,6 +48,18 @@
 /// 按槽号按位置寻址，一批换下的记录与这次重写的单元的新旧落点散在几片叶上，片数随落点怎么排有起落，W 式子里分配记录树那一项罩着它）。
 fn write_calls_the_largest_publish_of_any_file_may_add_over_a_one_unit_file() -> u64 {
     let pool = build_pool("unlink-writes-per-publish-bound-from-the-ring-window");
+    let devices_of_the_built_pool = (
+        pool.devices.as_ref().expect("镜像还开着").len(),
+        pool.allocator.devices.len(),
+    );
+    assert_eq!(
+        (
+            u64::try_from(devices_of_the_built_pool.0).expect("盘数装得进 u64"),
+            u64::try_from(devices_of_the_built_pool.1).expect("盘数装得进 u64"),
+        ),
+        (DEVICES_OF_THE_TEST_POOL, DEVICES_OF_THE_TEST_POOL),
+        "上界按测试池的盘数 {DEVICES_OF_THE_TEST_POOL} 乘：建出来的池（镜像块数、分配器管的盘数）就是这么多块盘"
+    );
     let slots_per_region = parameters().geometry.root_ring_slots_per_region;
     let ring_window_in_slots = ring_window_on_one_device(
         &pool.allocator,
@@ -305,3 +320,30 @@
         "1、2304、4608 个数据单元的文件，写调用最多的那次发布"
     );
 }
+
+/// 不建池时取的剩下单元数：建池那两条取到的 145、433 两点，慢档最大那份文件多一个单元的 4609，再往上到 2⁴⁰。
+const DATA_UNITS_REMAINING_JUDGED_WITHOUT_A_POOL: [u64; 7] =
+    [145, 433, 4_609, 9_216, 1_000_000, 1 << 32, 1 << 40];
+
+/// 产品批大小在多大的文件上都是一片下段叶（D8（核心索引结构） 已定项 16 第 2 条「每批释放尾部 K 个单元」、K 产品取
+/// `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`）：不建池，直接比发布路径取下一步的那一个纯函数 `next_unlink_step(剩下, OneExtentLowerLeaf)`，
+/// 剩下 145 到 2⁴⁰ 个单元的每一点都是「这一批换下 144 个、留下剩下 − 144 个、size 改成留下的单元数 × 净荷容量」。
+/// 判别力：每批取 max(144, ⌈剩下 ÷ 32⌉)（批数封顶 32，攻方 `research/prompts/m3-batch21-code-r1-opus-model/patches/c2cap32.patch`）时
+/// 剩下 4609 个单元那一点这一批换下 145 个，这一条红；那一刀在建池的 1 到 4608 个单元上与产品逐项相同，另两条照绿。
+/// 批数封顶在 N 而 N 小于 2⁴⁰ ÷ 144 的同一族都在 2⁴⁰ 那一点红。批大小不经这个函数、在发布路径另算的写法这一条碰不到，靠建池的那两条。
+#[test]
+fn the_product_unlink_batch_releases_one_extent_lower_leaf_from_a_file_of_any_size_up_to_two_to_the_fortieth_data_units(
+) {
+    let payload_capacity = u64::try_from(data_unit_payload_capacity()).expect("净荷容量装得进 u64");
+    for data_units_remaining in DATA_UNITS_REMAINING_JUDGED_WITHOUT_A_POOL {
+        let data_units_kept = data_units_remaining - EXTENT_TREE_LOWER_LEAF_DATA_UNITS;
+        assert_eq!(
+            next_unlink_step(data_units_remaining, UnlinkBatchSize::OneExtentLowerLeaf),
+            UnlinkStep::Batch {
+                data_units_kept,
+                size_after_the_batch_in_bytes: data_units_kept * payload_capacity,
+            },
+            "剩下 {data_units_remaining} 个数据单元时这一批恰好换下 {EXTENT_TREE_LOWER_LEAF_DATA_UNITS} 个、留下 {data_units_kept} 个"
+        );
+    }
+}
diff --git a/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs b/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
--- a/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
+++ b/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
@@ -602,8 +602,12 @@
     }
 }
 
-/// 卸载之前现行那一版换下的槽（释放代就是那一版的 txg）：卸载半途崩溃（只落下盘 0 的系统配置写）之后，F_生效 是那一版的 txg，
-/// 盘 1 上最新的根早它一个 txg、还引用这几个槽，重建分配器时回收、扣到这次挂载的暖机落满每块盘（`mount::rebuilt_allocator`）。
+/// 现行那一版换下的槽（释放代就是那一版的 txg）。卸载半途崩溃（系统配置已带新 F、带新 F 的根没落满每块盘）之后 F_生效 是卸载之前现行那一版的
+/// txg；盘 1 单独恢复择到的根可以就是那一版，也可以早它一个或两个 txg：攻方在单元区 176 槽的窄盘上覆盖写 K = 0..6 次量过，卸载只落下第一个写时
+/// K = 1、4 就是现行那一版，K = 0、2、5 早一个，K = 3、6 早两个；覆盖写 3 次、卸载落下 15 个写那一形也早两个（`research/prompts/m3-batch21-code-r1-opus-output.md`
+/// 第二节「放开扫」与「改法 G5」两段）。早它的时候盘 1 上那条根还引用这几个槽，重建分配器时回收、扣到这次挂载的暖机落满每块盘
+/// （`mount::rebuilt_allocator`）；早两个 txg 时中间那一版换下的槽同样扣着，只看现行那一版换下的判不出「只扣现行那一版换下的」那一刀，见
+/// [`after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_the_version_before_the_unmount_released_in_either_warm_up`]。
 fn slots_released_by_the_current_version(pool: &PoolUnderTest<SparseBlockDevice>) -> Vec<u64> {
     pool.current_file_version()
         .released
@@ -673,13 +677,19 @@
 
 /// 放开挪到第一次暖机之后那一刀（判决 `research/prompts/m3-batch19-code-r1-main-verification.md` 第二节「K2 N3 / N4 错实现漏红」那一行，
 /// 攻方同一份报告改法表 G1）：不补环窗口、准入关着造的窄盘（单元区 176 槽），第一个文件之后覆盖写 3 次；卸载落下 15 个写就崩，两块盘都在、
-/// 再可写挂载（准入判着、影子账开、批大小 1）做成，这次挂载的暖机推两次。写行那次与两次暖机每一次写下的单元一个都不落在卸载之前现行那一版
-/// 换下的槽上：那几个槽的释放代是现行那一版的 txg，高于盘 1 上最新的根、不高于 F_生效，扣到暖机在每块盘上都落了根才放开（D16（发布语义）
-/// 已定项 1「正常卸载」那一行，`mount::rebuilt_allocator` 扣、`mount::establish_instance` 暖机循环之后放开）。
+/// 再可写挂载（准入判着、影子账开、批大小 1）做成，这次挂载的暖机推两次。卸载崩溃镜像上盘 1 单独恢复择到的根早卸载之前现行那一版两个 txg
+/// （判决 `research/prompts/m3-batch21-code-r1-main-verification.md` 第二节「K2 N5 卸载崩点与覆盖写次数放开扫」那一行），它之后的两版
+/// （最后两次覆盖写）换下的槽释放代都高于盘 1 上最新的根、不高于 F_生效：写行那次与两次暖机每一次写下的单元一个都不落在这两版换下的槽上，
+/// 它们扣到暖机在每块盘上都落了根才放开（D16（发布语义） 已定项 1「正常卸载」那一行当场可用的只到 min(F_生效, 每块盘上最新有效根的 txg 最小者)，
+/// `mount::rebuilt_allocator` 扣、`mount::establish_instance` 暖机循环之后放开）。
 /// 判别力：第一次暖机之后就放开、取号之前的预演同样在那里放开（预演对真发那一判不分叉）时，第二次暖机的单元写进换下的槽，最后一项红
 /// （攻方原型上量过：这份历史写进换下的槽 2 个，`research/prompts/m3-batch19-code-r1-opus-model/out/n34-176-3-1-a036.txt` 第 150 行
 /// `inR=2`；丢盘 0 之后剩下那块盘读文件报 `DataUnitChecksumMismatchEverywhere`）。只在真发那一处放开、预演不放开时，挂载在预演对真发那一判上
 /// panic，这一条同样红。卸载只落下第一个写的那几条暖机只推一次，「第一次暖机之后」与「暖机落满之后」是同一刻，判不出这一刀。
+/// 重建时当场放开到 min(F_生效, max(每块盘上最新有效根的 txg 最小者, F_生效 − 1))、只扣现行那一版换下的（攻方
+/// `research/prompts/m3-batch21-code-r1-opus-model/patches/w2.patch`）时，写行那次的单元写进倒数第二次覆盖写换下的槽，最后一项红；
+/// 只比现行那一版换下的槽时这一刀照绿（攻方量过：丢盘 0 之后盘 1 读文件报 `DataUnitChecksumMismatchEverywhere`）。
+/// 测试名里「卸载之前那一版」说的是盘 1 上最新的根之后的每一版，名字没改：变异表里两行按这个名字点名。
 #[test]
 fn after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_the_version_before_the_unmount_released_in_either_warm_up(
 ) {
@@ -689,21 +699,55 @@
             device_width,
             176,
         );
+    let mut slots_released_by_every_version: Vec<(CheckpointTxg, Vec<u64>)> = vec![(
+        pool.session().current.root().checkpoint_txg,
+        slots_released_by_the_current_version(&pool),
+    )];
     for overwrite_index in 0..OVERWRITES_BEFORE_AN_UNMOUNT_WHOSE_NEXT_MOUNT_WARMS_UP_TWICE {
         pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
             panic!("第一个文件之后第 {overwrite_index} 次覆盖写：{refusal:?}")
         });
+        slots_released_by_every_version.push((
+            pool.session().current.root().checkpoint_txg,
+            slots_released_by_the_current_version(&pool),
+        ));
     }
-    let released_before_the_unmount = slots_released_by_the_current_version(&pool);
-    assert!(
-        !released_before_the_unmount.is_empty(),
-        "卸载之前现行那一版换下过槽（最后一次覆盖写换下上一版的单元）"
-    );
+    let current_txg_before_the_unmount = pool.session().current.root().checkpoint_txg;
     let after_the_unmount = image_after_an_unmount_crashing_after(
         &pool,
         device_width,
         UNMOUNT_WRITES_LANDED_BEFORE_A_MOUNT_THAT_WARMS_UP_TWICE,
     );
+    let newest_root_on_device_one = recover(
+        &image_without(&after_the_unmount, DeviceIdentity(0)),
+        JournalPolicy::Consult,
+    )
+    .effective_root
+    .map(|(_, checkpoint_txg)| checkpoint_txg)
+    .expect("卸载崩溃镜像上盘 1 单独恢复择得到根");
+    let slots_released_by_every_version_after_the_newest_root_on_device_one: Vec<(
+        CheckpointTxg,
+        Vec<u64>,
+    )> = slots_released_by_every_version
+        .into_iter()
+        .filter(|(released_by, _)| *released_by > newest_root_on_device_one)
+        .collect();
+    assert_eq!(
+        slots_released_by_every_version_after_the_newest_root_on_device_one
+            .iter()
+            .map(|(released_by, slots)| (*released_by, slots.is_empty()))
+            .collect::<Vec<_>>(),
+        vec![
+            (CheckpointTxg(current_txg_before_the_unmount.0 - 1), false),
+            (current_txg_before_the_unmount, false),
+        ],
+        "盘 1 上最新的根（txg {newest_root_on_device_one:?}）早卸载之前现行那一版两个 txg，它之后的两版是最后两次覆盖写、各自换下过槽"
+    );
+    let released_after_the_newest_root_on_device_one: Vec<u64> =
+        slots_released_by_every_version_after_the_newest_root_on_device_one
+            .into_iter()
+            .flat_map(|(_, slots)| slots)
+            .collect();
     let mut remount_devices = plain_devices_on(&after_the_unmount);
     let mounted = mount_writable_through(
         RemountEntry::WithDeletionBatchesOfOne,
@@ -727,7 +771,9 @@
                             .units
                             .iter()
                             .flat_map(|unit| unit.slot.0..unit.slot.0 + unit.identity.span_slots())
-                            .filter(|slot| released_before_the_unmount.contains(slot))
+                            .filter(|slot| {
+                                released_after_the_newest_root_on_device_one.contains(slot)
+                            })
                             .collect()
                     })
                     .unwrap_or_default()
@@ -736,7 +782,8 @@
     assert_eq!(
         slots_taken_into_the_released_ones_by_every_publish,
         vec![Vec::<u64>::new(); 3],
-        "写行那次、第一次暖机、第二次暖机写下的单元（按这个次序一次一项）都不落在卸载之前现行那一版换下的槽 {released_before_the_unmount:?} 上"
+        "写行那次、第一次暖机、第二次暖机写下的单元（按这个次序一次一项）都不落在盘 1 上最新的根之后每一版换下的槽 \
+         {released_after_the_newest_root_on_device_one:?} 上"
     );
 }
 
````

## 实现员 t：mutations-append.tsv（/tmp/claude-1000/m3-batch22/t/patch/mutations-append.tsv）

````text
删除批第 22 批 N7：unlink 的产品批大小取 max(144, ⌈剩下 ÷ 32⌉)（批数封顶 32，攻方 m3-batch21-code-r1 c2cap32），剩 4609 个单元时一批换下 145 个；建池的 1 到 4608 个单元上与产品逐项相同	crates/singlefs-core/src/unlink.rs	    let data_units_per_batch = batch_size.data_units_per_batch();\n    if data_units_remaining <= data_units_per_batch {	    let data_units_per_batch = match batch_size {\n        UnlinkBatchSize::OneExtentLowerLeaf => batch_size\n            .data_units_per_batch()\n            .max(data_units_remaining.div_ceil(32)),\n        UnlinkBatchSize::CappedForTests(_) => batch_size.data_units_per_batch(),\n    };\n    if data_units_remaining <= data_units_per_batch {	-p singlefs-harness --test unlink_writes_per_transaction_do_not_grow_with_the_file	the_product_unlink_batch_releases_one_extent_lower_leaf_from_a_file_of_any_size_up_to_two_to_the_fortieth_data_units
删除批第 22 批 N5：可写挂载重建分配器时当场放开到 min(F_生效, max(每块盘上最新有效根的 txg 最小者, F_生效 − 1))、只扣现行那一版换下的（攻方 m3-batch21-code-r1 W2），覆盖写 3 次、卸载落下 15 个写那一形写行那次写进倒数第二次覆盖写换下的槽	crates/singlefs-core/src/mount.rs	            effective_floor.min(newest_valid_root_on_every_device),	            effective_floor.min(newest_valid_root_on_every_device.max(CheckpointTxg(\n                effective_floor.0.saturating_sub(1),\n            ))),	-p singlefs-harness --test the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads -- after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times	after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_the_version_before_the_unmount_released_in_either_warm_up
删除批第 22 批：unlink 写调用上界那份测试建池时分配器多管一块盘（建出来的池不再是 DEVICES_OF_THE_TEST_POOL 的 2 块盘），上界那一处核盘数的断言红	crates/singlefs-harness/tests/common/mod.rs	        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),	        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),\n        DeviceFreeMap::new(DeviceIdentity(2), IMAGE_BYTES),	-p singlefs-harness --test unlink_writes_per_transaction_do_not_grow_with_the_file	unlink_of_files_of_one_to_four_batches_releases_exactly_one_batch_or_the_rest_per_publish_and_its_largest_publish_does_not_grow_with_the_file
````
