# 删除线第 56 批（代码轮 m3-batch52-54-code-r1 判决的 G1–G6）：代码轮第一轮改动全文（2026-10-10）

三个实现员交的补丁原样，按打的次序（甲 → 乙 → 丙），底是主工作区合入第 52–55 批与根环线第二十六块之后的样子。

## m3-batch56/recovery-rollback

### crates.patch

`````diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -7857,6 +7857,22 @@
     })
 }
 
+/// 管理员回退那次发布的树 ID 水位（D23（journal 的角色与格式） 已定项 14「水位」一格）：max(cur 那一版内存里的水位, 根环里读得出的**全部**根的水位)。
+/// 与 inode 号水位（[`inode_number_watermark_of_the_rollback`]）不同，被崩溃恢复抛弃的根照旧算：树 ID 发过就不再发，被抛弃的时间线上发过的号
+/// 照样不能再发给别的树。`_current_table_not_consulted` 是 cur 的实例表，这里不读它：收它是为了与 inode 号水位那一段同形，
+/// 构造态单测交一条按它被抛弃的根，「改成只取没被抛弃的根」这一错写得出来、会红（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`
+/// 第三节 G5：回退用例在走得到的历史上分不出这两种取法）。
+fn tree_identifier_watermark_of_the_rollback(
+    ring_roots: &[RootRecord],
+    _current_table_not_consulted: &InstanceTableRecords,
+    tree_identifier_watermark_of_the_current_version: u64,
+) -> u64 {
+    ring_roots
+        .iter()
+        .map(|root| root.tree_identifier_watermark)
+        .fold(tree_identifier_watermark_of_the_current_version, u64::max)
+}
+
 /// 回退那次发布接在后面的「上一版」：用户可见的那一半（数据单元与指针、extent 树、inode 树的叶容器与根、第一个文件那条 inode 记录）
 /// 取回退目标那一版的，别的（分配记录树与它的记录、记账树与记账行、中央映射树、树表、实例表各片）取现行那一版的；
 /// 根记录、末条记录、事务号取现行那一版的（这次发布接着它发，反向链、计数器、txg 都接着它）。
@@ -8128,10 +8144,11 @@
         &system_configuration.immutable.sizes,
         &system_configuration.immutable.filesystem_identifier,
     );
-    let tree_identifier_watermark = ring_roots
-        .iter()
-        .map(|root| root.tree_identifier_watermark)
-        .fold(current.root.tree_identifier_watermark, u64::max);
+    let tree_identifier_watermark = tree_identifier_watermark_of_the_rollback(
+        &ring_roots,
+        &current_table,
+        current.root.tree_identifier_watermark,
+    );
     // inode 号水位（Opt2，D23（journal 的角色与格式） 已定项 14「水位」一格）：环里只取按 cur 的实例表没被崩溃恢复抛弃的根；
     // max 里 cur 那一版内存里的那一项不能去——只取环，回退那次就重发号（C143）。
     let inode_number_watermark = inode_number_watermark_of_the_rollback(
@@ -9423,4 +9440,110 @@
             "水位取环与 cur 的最大，不只取环"
         );
     }
+}
+
+/// 管理员回退取树 ID 水位那一段（D23（journal 的角色与格式） 已定项 14「水位」一格：树 ID 水位照旧取环里读得出的全部根）的构造态单测：
+/// 回退用例在走得到的历史上分不出「全部根」与「只取没被崩溃恢复抛弃的根」（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`
+/// 第二节 K3、第三节 G5），这里直接交一条按 cur 的实例表被抛弃、树 ID 水位高于 cur 的根。
+#[cfg(test)]
+mod tree_identifier_watermark_of_the_rollback_tests {
+    use super::{abandoned_by_table, tree_identifier_watermark_of_the_rollback};
+    use crate::address::{CheckpointTxg, InstanceGeneration};
+    use crate::instance_table::{InstanceRow, InstanceTableRecords};
+    use crate::pointer::NodePointer;
+    use crate::root_record::{RootRecord, UnmountMarker};
+
+    /// cur 那一版内存里的树 ID 水位。
+    const TREE_IDENTIFIER_WATERMARK_OF_THE_CURRENT_VERSION: u64 = 65_549;
+    /// 环里被抛弃的那条根（实例 1、txg 7）的树 ID 水位：高于 cur 的。
+    const TREE_IDENTIFIER_WATERMARK_OF_THE_ABANDONED_RING_ROOT_ABOVE_THE_CURRENT_VERSION: u64 =
+        65_560;
+    /// 环里一条没被抛弃的根（实例 1、txg 6）的树 ID 水位：低于 cur 的。
+    const TREE_IDENTIFIER_WATERMARK_OF_THE_RING_ROOT_NOT_ABANDONED_BELOW_THE_CURRENT_VERSION: u64 =
+        65_540;
+    /// 环里被抛弃的另一条根（实例 1、txg 8）的树 ID 水位：低于 cur 的。
+    const TREE_IDENTIFIER_WATERMARK_OF_THE_ABANDONED_RING_ROOT_BELOW_THE_CURRENT_VERSION: u64 =
+        65_545;
+    const INSTANCE_OF_THE_RING_ROOTS: InstanceGeneration = InstanceGeneration(1);
+    /// cur 的实例表里实例 1 那一行的所选根 txg：txg 大于它的实例 1 的根被崩溃恢复抛弃。
+    const SELECTED_ROOT_TXG_OF_THE_CURRENT_TABLE: CheckpointTxg = CheckpointTxg(6);
+
+    fn ring_root(checkpoint_txg: CheckpointTxg, tree_identifier_watermark: u64) -> RootRecord {
+        RootRecord {
+            filesystem_identifier: [7; 16],
+            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
+            instance: INSTANCE_OF_THE_RING_ROOTS,
+            checkpoint_txg,
+            tree_table: NodePointer::empty_root(),
+            tree_identifier_watermark,
+            rollback_floor: CheckpointTxg(0),
+            instance_table: NodePointer::empty_root(),
+            mapping_root: NodePointer::empty_root(),
+            allocation_record_tree_root: NodePointer::empty_root(),
+        }
+    }
+
+    fn current_table() -> InstanceTableRecords {
+        InstanceTableRecords {
+            rows: vec![InstanceRow {
+                instance: INSTANCE_OF_THE_RING_ROOTS,
+                selected_root_txg: SELECTED_ROOT_TXG_OF_THE_CURRENT_TABLE,
+                applied_transaction_high_water: 0,
+            }],
+        }
+    }
+
+    /// 环里一条没被抛弃的根（txg 6，水位 65540，低于 cur）与一条按 cur 的实例表被抛弃的根（txg 7 > 6，水位 65560，高于 cur 的 65549）：
+    /// 取 65560——被抛弃的那条照旧算。
+    #[test]
+    fn an_abandoned_ring_root_whose_tree_identifier_watermark_is_above_the_current_version_sets_the_watermark(
+    ) {
+        let abandoned_root = ring_root(
+            CheckpointTxg(7),
+            TREE_IDENTIFIER_WATERMARK_OF_THE_ABANDONED_RING_ROOT_ABOVE_THE_CURRENT_VERSION,
+        );
+        let root_not_abandoned = ring_root(
+            CheckpointTxg(6),
+            TREE_IDENTIFIER_WATERMARK_OF_THE_RING_ROOT_NOT_ABANDONED_BELOW_THE_CURRENT_VERSION,
+        );
+        let current_table = current_table();
+        assert!(
+            abandoned_by_table(&abandoned_root, &current_table)
+                && !abandoned_by_table(&root_not_abandoned, &current_table),
+            "前提：txg 7 那条按 cur 的实例表被抛弃，txg 6 那条没有"
+        );
+        assert_eq!(
+            tree_identifier_watermark_of_the_rollback(
+                &[root_not_abandoned, abandoned_root],
+                &current_table,
+                TREE_IDENTIFIER_WATERMARK_OF_THE_CURRENT_VERSION,
+            ),
+            TREE_IDENTIFIER_WATERMARK_OF_THE_ABANDONED_RING_ROOT_ABOVE_THE_CURRENT_VERSION,
+            "树 ID 水位取环里读得出的全部根：被崩溃恢复抛弃的那条照旧算"
+        );
+    }
+
+    /// 环里两条根（txg 6 没被抛弃、水位 65540；txg 8 被抛弃、水位 65545）都低于 cur 的 65549：取 cur 的。
+    #[test]
+    fn ring_roots_all_below_the_current_version_leave_the_current_tree_identifier_watermark() {
+        let current_table = current_table();
+        assert_eq!(
+            tree_identifier_watermark_of_the_rollback(
+                &[
+                    ring_root(
+                        CheckpointTxg(6),
+                        TREE_IDENTIFIER_WATERMARK_OF_THE_RING_ROOT_NOT_ABANDONED_BELOW_THE_CURRENT_VERSION,
+                    ),
+                    ring_root(
+                        CheckpointTxg(8),
+                        TREE_IDENTIFIER_WATERMARK_OF_THE_ABANDONED_RING_ROOT_BELOW_THE_CURRENT_VERSION,
+                    ),
+                ],
+                &current_table,
+                TREE_IDENTIFIER_WATERMARK_OF_THE_CURRENT_VERSION,
+            ),
+            TREE_IDENTIFIER_WATERMARK_OF_THE_CURRENT_VERSION,
+            "水位取环与 cur 的最大，不只取环"
+        );
+    }
 }
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -2913,14 +2913,11 @@
             )
         },
     )?;
+    // 次序：条目宽、逐条判代与 seq、数条目——先判得了自己那一条的，再数（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`
+    // 第三节 G1）。数条目要按统计量与代段认台阶行，代写到根之后的台阶行认不出、被算进 3 + 6 × 盘数，「代晚于根」这一坏法就报成了条目数不对。
     every_accounting_entry_is_as_wide_as_its_field_table(
         &accounting_tree.leaf_entries_in_key_order,
     )?;
-    accounting_entry_count_is_three_plus_six_per_device(
-        &accounting_tree.leaf_entries_in_key_order,
-        root.checkpoint_txg,
-        device_identities.len(),
-    )?;
     let mut accounting_entries: Vec<AccountingEntry> =
         Vec::with_capacity(accounting_tree.leaf_entries_in_key_order.len());
     for bytes in &accounting_tree.leaf_entries_in_key_order {
@@ -2928,6 +2925,11 @@
             bytes, root,
         )?);
     }
+    accounting_entry_count_is_three_plus_six_per_device(
+        &accounting_tree.leaf_entries_in_key_order,
+        root.checkpoint_txg,
+        device_identities.len(),
+    )?;
     // 记账行的统计量标签是盘上读来的 2 字节：这一版交出去之前判一次「inode 号水位那一行在」，往里就信
     // （`TransactionOutput::inode_number_watermark` 那句 expect 依赖的就是这一判；panic 面普查 R11）。
     // 要的是这一版自己写的那一行（代段 = 根的 txg）：那一统计量另有代段更早的台阶行（D5（快照 / 空间记账机制） 已定项 2 射程），
@@ -4779,7 +4781,8 @@
 
 /// 记账条目数是 3 + 6 × 盘数（D5（快照 / 空间记账机制） 已定项 8：池级三行加每盘六行），inode 号水位的台阶行不算在里面：
 /// 它们是那一统计量代段早于这一版 txg（`root_txg`）的行，随根一直留着、条数随建过 inode 的发布数涨（D5 已定项 2 射程「inode 号水位的台阶行」）。
-/// 解不开的条目算进 3 + 6 × 盘数；调用方先过 [`every_accounting_entry_is_as_wide_as_its_field_table`]，走到这里的每条都解得开。
+/// 解不开的条目算进 3 + 6 × 盘数；调用方先过 [`every_accounting_entry_is_as_wide_as_its_field_table`] 与逐条的
+/// [`accounting_entry_parsed_and_judged_against_the_root`]，走到这里的每条都解得开、代不晚于根：代写到根之后的台阶行在那一判先红，不落到这里被算进条数。
 fn accounting_entry_count_is_three_plus_six_per_device(
     accounting_entry_bytes: &[Vec<u8>],
     root_txg: CheckpointTxg,
@@ -5387,9 +5390,15 @@
         allocation_records,
         &device_identities,
     )?;
+    // 次序与重建（`rebuild_version_in_the_unit_area_starting_at`）相同：分配记录逐条判代与跨度，记账条目宽、逐条判代与 seq，
+    // 之后才数记账条目与映射条目——先判得了自己那一条的，再数（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第三节 G1）。
+    allocation_record_generations_and_spans_are_judged(allocation_records, root)?;
     every_accounting_entry_is_as_wide_as_its_field_table(
         &roots.accounting.leaf_entries_in_key_order,
     )?;
+    for entry_bytes in &roots.accounting.leaf_entries_in_key_order {
+        accounting_entry_parsed_and_judged_against_the_root(entry_bytes, root)?;
+    }
     accounting_entry_count_is_three_plus_six_per_device(
         &roots.accounting.leaf_entries_in_key_order,
         root.checkpoint_txg,
@@ -5406,10 +5415,6 @@
             deletion_tree_nodes,
         },
     )?;
-    allocation_record_generations_and_spans_are_judged(allocation_records, root)?;
-    for entry_bytes in &roots.accounting.leaf_entries_in_key_order {
-        accounting_entry_parsed_and_judged_against_the_root(entry_bytes, root)?;
-    }
 
     // inode 树：根（码 2、层级 1）→ 内部条目 → 叶容器（码 3）→ 记录。
     inode_tree_root_level_is_one(&roots.inode)?;
diff --git a/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs b/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
--- a/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
+++ b/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
@@ -30,8 +30,8 @@
     PointerHeadFieldOutsideTheFirstVersion,
 };
 use singlefs_core::records::{
-    mapping_key_for_node, parse_mapping_entry, TreeTableEntry, STATISTIC_INODE_WATERMARK,
-    TREE_KIND_ACCOUNTING, TREE_KIND_INODE, TREE_KIND_RETIREMENT,
+    mapping_key_for_node, parse_mapping_entry, AccountingEntry, TreeTableEntry,
+    STATISTIC_INODE_WATERMARK, TREE_KIND_ACCOUNTING, TREE_KIND_INODE, TREE_KIND_RETIREMENT,
 };
 use singlefs_core::recovery::{
     choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover, JournalPolicy,
@@ -1567,6 +1567,114 @@
     );
 }
 
+// ─── 记账条目逐条判代与 seq 先于记账条目数：代写到根之后的台阶行认不出 ───
+
+/// 台阶行那个池（[`pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row`]）最新那条根的 txg。
+const TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW: CheckpointTxg = CheckpointTxg(4);
+/// 台阶行的代改成的值：晚于根的 txg 4。
+const STEP_ROW_GENERATION_AFTER_THE_ROOT: CheckpointTxg = CheckpointTxg(5);
+
+/// 码 2 记账叶根里恰好一条满足 `is_the_entry_to_change` 的条目交给 `change` 改，之后全部条目按 key 重排、节点头里的最小 / 最大 key
+/// （从偏移 52 起，各一个 key 宽）换成首末两条的 key：条目宽、条目数不变，key 照旧严格递增，判全那一道（节点头、key 区间与次序）照过。
+fn rewrite_one_entry_of_an_accounting_leaf_root_keeping_the_key_order(
+    node: &mut [u8],
+    is_the_entry_to_change: impl Fn(&AccountingEntry) -> bool,
+    change: impl FnOnce(&mut AccountingEntry),
+) {
+    let key_width = usize::from(node[51]);
+    let header_end = 86 + 2 * key_width;
+    let entry_count = usize::from(u16::from_le_bytes([
+        node[header_end - 4],
+        node[header_end - 3],
+    ]));
+    let entry_ranges: Vec<std::ops::Range<usize>> = (0..entry_count)
+        .map(|entry_index| entry_range_in_node(node, entry_index))
+        .collect();
+    let mut entries: Vec<AccountingEntry> = entry_ranges
+        .iter()
+        .map(|range| {
+            AccountingEntry::parse(&node[range.clone()]).expect("记账叶根的条目宽是 34，每条解得开")
+        })
+        .collect();
+    let indices_to_change: Vec<usize> = (0..entries.len())
+        .filter(|entry_index| is_the_entry_to_change(&entries[*entry_index]))
+        .collect();
+    assert_eq!(indices_to_change.len(), 1, "恰好一条条目要改");
+    change(&mut entries[indices_to_change[0]]);
+    entries.sort_by_key(AccountingEntry::sort_key);
+    for (range, entry) in entry_ranges.iter().zip(&entries) {
+        node[range.clone()].copy_from_slice(&entry.to_bytes());
+    }
+    let smallest_key = entries.first().expect("记账叶根至少一条").key_bytes();
+    let largest_key = entries.last().expect("记账叶根至少一条").key_bytes();
+    node[52..52 + key_width].copy_from_slice(&smallest_key);
+    node[52 + key_width..52 + 2 * key_width].copy_from_slice(&largest_key);
+}
+
+/// 最新那条根（txg 4）记账树根里那一级台阶行（inode 号水位、代段 3）的代改成 5（晚于根），按 key 重排到这一版自己那一行（代 4）之后；
+/// 节点与指着它的每一道校验和重算。攻方原型 `research/prompts/m3-batch52-54-code-r1-opus-model/probes/corrupt_on_disk_k1_append.rs` 的 ② 那一形。
+fn pool_whose_accounting_step_row_carries_a_generation_after_the_root() -> Devices {
+    let mut devices =
+        pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row();
+    rewrite_the_root_node_of_a_tree_of_the_newest_root(
+        &mut devices,
+        TREE_KIND_ACCOUNTING,
+        |node| {
+            rewrite_one_entry_of_an_accounting_leaf_root_keeping_the_key_order(
+                node,
+                |entry| {
+                    entry.statistic == STATISTIC_INODE_WATERMARK
+                        && entry.generation < TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW
+                },
+                |step_row| step_row.generation = STEP_ROW_GENERATION_AFTER_THE_ROOT,
+            );
+        },
+    );
+    devices
+}
+
+/// 记账条目逐条判那一个成员：代晚于根或 seq 为 0。
+fn accounting_entry_generation_after_the_root_or_sequence_zero() -> RecoveryFailure {
+    RecoveryFailure::InvariantViolated {
+        invariant: "E142 走读同款",
+        detail: "记账条目的代晚于根或 seq 为 0",
+    }
+}
+
+/// 场景：第一个文件之后再建一个空 inode（txg 4），记账树根 16 条里那一级台阶行（代段 3）的代改成 5，晚于根的 txg 4；条目按 key 重排、
+/// 节点与指着它的每一道校验和重算。
+/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「记账条目的代晚于根或 seq 为 0」。数条目按「统计量是 inode 号水位且代早于根」认台阶行，
+/// 代写到根之后就认不出、被算进 3 + 6 × 盘数（16 ≠ 15）：数条目排在逐条判代之前时，这一坏法报成「记账条目数（inode 号水位的台阶行不算）
+/// 不是 3 + 6 × 盘数」（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第二节 K1-1、第三节 G1）。
+#[test]
+fn an_accounting_step_row_whose_generation_is_after_the_root_is_refused_for_its_generation_by_the_recovery(
+) {
+    let devices = pool_whose_accounting_step_row_carries_a_generation_after_the_root();
+    let report = recover(&devices, JournalPolicy::Consult);
+    assert_eq!(
+        report.outcome,
+        RecoveryOutcome::Failed {
+            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
+            failure: accounting_entry_generation_after_the_root_or_sequence_zero(),
+        },
+        "只读恢复在 txg 4 那条根上报记账条目的代晚于根，不报条目数"
+    );
+}
+
+/// 场景：同上一条的坏镜像，交给可写挂载。
+/// 预期：可写挂载在任何写之前交回「记账条目的代晚于根或 seq 为 0」：重建那一版读记账树时同样先逐条判代与 seq、后数条目
+/// （判法与上一条同一个理由）。
+#[test]
+fn an_accounting_step_row_whose_generation_is_after_the_root_is_refused_for_its_generation_by_the_writable_mount(
+) {
+    let mut devices = pool_whose_accounting_step_row_carries_a_generation_after_the_root();
+    let refusal = writable_mount_refused_before_any_write(&mut devices);
+    assert!(
+        matches!(&refusal, MountError::Recovery(failure) if *failure == accounting_entry_generation_after_the_root_or_sequence_zero()),
+        "可写挂载报记账条目的代晚于根，不报条目数：{refusal:?}"
+    );
+}
+
 // ─── 第 35 条续：单元区槽数那一处减法 ───
 
 /// 场景：700 MiB 的盘（单元区起点在 784 MiB）。预期：单元区槽数交回「盘末尾在单元区起点之前」，带盘的字节数与起点槽号；
`````

### mutations-append.tsv

`````text
第56批 G1 冷走读数记账条目排回逐条判代与 seq 之前（代写到根之后的台阶行认不出、被算进 3 + 6 × 盘数，代晚于根报成条目数）	crates/singlefs-core/src/recovery.rs	    for entry_bytes in &roots.accounting.leaf_entries_in_key_order {\n        accounting_entry_parsed_and_judged_against_the_root(entry_bytes, root)?;\n    }\n    accounting_entry_count_is_three_plus_six_per_device(\n        &roots.accounting.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_count,\n    )?;	    accounting_entry_count_is_three_plus_six_per_device(\n        &roots.accounting.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_count,\n    )?;\n    for entry_bytes in &roots.accounting.leaf_entries_in_key_order {\n        accounting_entry_parsed_and_judged_against_the_root(entry_bytes, root)?;\n    }	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- an_accounting_step_row_whose_generation_is_after_the_root	an_accounting_step_row_whose_generation_is_after_the_root_is_refused_for_its_generation_by_the_recovery
第56批 G1 重建数记账条目排回逐条判代与 seq 之前（代写到根之后的台阶行认不出、被算进 3 + 6 × 盘数，代晚于根报成条目数）	crates/singlefs-core/src/recovery.rs	    let mut accounting_entries: Vec<AccountingEntry> =\n        Vec::with_capacity(accounting_tree.leaf_entries_in_key_order.len());\n    for bytes in &accounting_tree.leaf_entries_in_key_order {\n        accounting_entries.push(accounting_entry_parsed_and_judged_against_the_root(\n            bytes, root,\n        )?);\n    }\n    accounting_entry_count_is_three_plus_six_per_device(\n        &accounting_tree.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_identities.len(),\n    )?;	    accounting_entry_count_is_three_plus_six_per_device(\n        &accounting_tree.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_identities.len(),\n    )?;\n    let mut accounting_entries: Vec<AccountingEntry> =\n        Vec::with_capacity(accounting_tree.leaf_entries_in_key_order.len());\n    for bytes in &accounting_tree.leaf_entries_in_key_order {\n        accounting_entries.push(accounting_entry_parsed_and_judged_against_the_root(\n            bytes, root,\n        )?);\n    }	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- an_accounting_step_row_whose_generation_is_after_the_root	an_accounting_step_row_whose_generation_is_after_the_root_is_refused_for_its_generation_by_the_writable_mount
第56批 G5 回退取树 ID 水位改成只取没被崩溃恢复抛弃的根（构造态）	crates/singlefs-core/src/mount.rs	    ring_roots\n        .iter()\n        .map(|root| root.tree_identifier_watermark)\n        .fold(tree_identifier_watermark_of_the_current_version, u64::max)	    ring_roots_not_abandoned_by_crash_recovery(ring_roots, _current_table_not_consulted)\n        .iter()\n        .map(|root| root.tree_identifier_watermark)\n        .fold(tree_identifier_watermark_of_the_current_version, u64::max)	-p singlefs-core --lib -- tree_identifier_watermark_of_the_rollback_tests	an_abandoned_ring_root_whose_tree_identifier_watermark_is_above_the_current_version_sets_the_watermark
第56批 G5 回退取树 ID 水位只取环、不与 cur 内存里的取 max（构造态）	crates/singlefs-core/src/mount.rs	        .fold(tree_identifier_watermark_of_the_current_version, u64::max)	        .fold(0, u64::max)	-p singlefs-core --lib -- tree_identifier_watermark_of_the_rollback_tests	ring_roots_all_below_the_current_version_leave_the_current_tree_identifier_watermark
`````

### mutations-replacements.tsv

`````text
第52批 冷走读记账条目数排回条目宽之前（台阶行解不开被算进 3 + 6 × 盘数，条目宽坏法报成条目数）	crates/singlefs-core/src/recovery.rs	    every_accounting_entry_is_as_wide_as_its_field_table(\n        &roots.accounting.leaf_entries_in_key_order,\n    )?;\n    for entry_bytes in &roots.accounting.leaf_entries_in_key_order {\n        accounting_entry_parsed_and_judged_against_the_root(entry_bytes, root)?;\n    }\n    accounting_entry_count_is_three_plus_six_per_device(\n        &roots.accounting.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_count,\n    )?;	    accounting_entry_count_is_three_plus_six_per_device(\n        &roots.accounting.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_count,\n    )?;\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &roots.accounting.leaf_entries_in_key_order,\n    )?;\n    for entry_bytes in &roots.accounting.leaf_entries_in_key_order {\n        accounting_entry_parsed_and_judged_against_the_root(entry_bytes, root)?;\n    }	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_recovery	a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_recovery
第52批 重建记账条目数排回条目宽之前（台阶行解不开被算进 3 + 6 × 盘数，条目宽坏法报成条目数）	crates/singlefs-core/src/recovery.rs	    every_accounting_entry_is_as_wide_as_its_field_table(\n        &accounting_tree.leaf_entries_in_key_order,\n    )?;\n    let mut accounting_entries: Vec<AccountingEntry> =\n        Vec::with_capacity(accounting_tree.leaf_entries_in_key_order.len());\n    for bytes in &accounting_tree.leaf_entries_in_key_order {\n        accounting_entries.push(accounting_entry_parsed_and_judged_against_the_root(\n            bytes, root,\n        )?);\n    }\n    accounting_entry_count_is_three_plus_six_per_device(\n        &accounting_tree.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_identities.len(),\n    )?;	    accounting_entry_count_is_three_plus_six_per_device(\n        &accounting_tree.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_identities.len(),\n    )?;\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &accounting_tree.leaf_entries_in_key_order,\n    )?;\n    let mut accounting_entries: Vec<AccountingEntry> =\n        Vec::with_capacity(accounting_tree.leaf_entries_in_key_order.len());\n    for bytes in &accounting_tree.leaf_entries_in_key_order {\n        accounting_entries.push(accounting_entry_parsed_and_judged_against_the_root(\n            bytes, root,\n        )?);\n    }	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_writable_mount	a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_writable_mount
C586：回退那次的树 ID 水位只取环里读得出的根，不与内存里现行那一版的取 max	crates/singlefs-core/src/mount.rs	        .fold(tree_identifier_watermark_of_the_current_version, u64::max)	        .fold(0, u64::max)	-p singlefs-harness --test unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over -- the_rollback_takes_the_tree_identifier_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_every_root_carrying_it	the_rollback_takes_the_tree_identifier_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_every_root_carrying_it
`````

## m3-batch56/container-judges

### crates.patch

`````diff
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
index 0267acd..2316a92 100644
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -106,8 +106,9 @@ pub const SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY: &str =
 ///
 /// 第 91、92 条（下标 90、91）是谱系树与退役树的容器容量那两条（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2，
 /// 第 53 批，与 I-9.21、I-9.22 同形）：I-9.36、I-9.37，同样只往末尾追加。在墓碑与谱系那一遍里每一版各判；I-9.37 的旋钮值从镜像外给
-/// （`walk::check_pool_image_given_the_container_capacity_knobs`）。GPU 判器的扫描内核同判 I-9.36，I-9.37 照 I-9.22 恒不置位
-/// （层 0 与放量走旧入口，拿不到旋钮值）。
+/// （`walk::check_pool_image_given_the_container_capacity_knobs`）。GPU 判器的扫描内核同判 I-9.36；第 56 批 G6 起旋钮值经扫描内核源码前面的常量
+/// 传进 GPU（`crash_judge_gpu` 的 `JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`），装了旋钮的那一棵 GPU 同判 I-9.37
+/// （墓碑旋钮同判 I-9.22）；层 0 与放量走产品那一份（三棵都没装），I-9.22、I-9.37 在那里照旧不置位。
 pub const IMPLEMENTED_INVARIANTS: [&str; 92] = [
     "I-1.1",
     "I-1.2",
@@ -288,9 +289,11 @@ pub const TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT: &str = "I-9.19";
 /// 对象出生代 ≤ 死亡 txg ≤ 这一版的 txg；每片墓碑容器记录数 ≤ 815（判决第二节 T14 与第四节用户定案「补」）。
 pub const TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED: &str =
     "I-9.20";
-/// I-9.21（产品镜像里非最右墓碑容器装满）：没装只供测试的容量旋钮时，每一版墓碑树里容器号沿叶序严格递增、最右那片之外每片记录数 == 815。
+/// I-9.21（产品镜像里非最右墓碑容器装满）：没装只供测试的容量旋钮时，每一版墓碑树里沿叶序第一片容器号是 1、相邻两片容器号加一、
+/// 每片（含最右那片）至少一条（第 56 批 G2）、最右那片之外每片记录数 == 815；零根或只一片容器没有对象。
 pub const NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE: &str = "I-9.21";
-/// I-9.22（测试镜像里墓碑容器不超旋钮值）：装了旋钮（新入口给了旋钮值）时，每一版墓碑树里容器号沿叶序严格递增、每片记录数 ≤ 旋钮值。
+/// I-9.22（测试镜像里墓碑容器不超旋钮值）：装了旋钮（新入口给了旋钮值）时，每一版墓碑树里沿叶序第一片容器号是 1、相邻两片容器号加一、
+/// 每片至少一条（第 56 批 G2）、每片记录数 ≤ 旋钮值；零根没有对象。
 pub const TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE: &str = "I-9.22";
 /// I-9.23（墓碑树条目身份与子头相符）：与 I-9.17 同形，类型段 ∈ {0, 1}、子容器打包记录类型 1。
 pub const TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER: &str = "I-9.23";
@@ -327,12 +330,13 @@ pub const RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF: &str = "I-9.34";
 /// 读得出才判）：父版叶身份集 − 本版叶身份集 = 本版谱系上、死亡 txg == 本版 txg 的退役记录的身份集，两个方向各报。最新根的实例表读不出、
 /// 滤不掉被崩溃恢复抛弃的根时报部分没判（照 I-9.29 的门）。被攻过零轮（判决第四节）。
 pub const RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION: &str = "I-9.35";
-/// I-9.36（产品镜像里非最右谱系与退役容器装满，与 I-9.21 同形）：那一棵没装只供测试的容量旋钮时，每一版谱系树与退役树里容器号沿叶序严格递增、
-/// 最右那片之外每片记录数 == 产品容量（谱系 1359、退役 959；开放容器满了才关，D18（块里携带什么信息） 已定项 24）。
+/// I-9.36（产品镜像里非最右谱系与退役容器装满，与 I-9.21 同形）：那一棵没装只供测试的容量旋钮时，每一版谱系树与退役树里沿叶序第一片容器号是 1、
+/// 相邻两片容器号加一、每片（含最右那片）至少一条（第 56 批 G2）、最右那片之外每片记录数 == 产品容量（谱系 1359、退役 959；开放容器满了才关，
+/// D18（块里携带什么信息） 已定项 24）；零根或只一片容器没有对象。
 pub const NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE: &str =
     "I-9.36";
-/// I-9.37（测试镜像里谱系与退役容器不超旋钮值，与 I-9.22 同形）：那一棵装了旋钮（新入口给了旋钮值）时，每一版那棵树里容器号沿叶序严格递增、
-/// 每片记录数 ≤ 旋钮值。
+/// I-9.37（测试镜像里谱系与退役容器不超旋钮值，与 I-9.22 同形）：那一棵装了旋钮（新入口给了旋钮值）时，每一版那棵树里沿叶序第一片容器号是 1、
+/// 相邻两片容器号加一、每片至少一条（第 56 批 G2）、每片记录数 ≤ 旋钮值；零根没有对象。
 pub const LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE: &str =
     "I-9.37";
 
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index e95abeb..be1b0c3 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -10210,6 +10210,10 @@ enum AbandonedRootsFilteringOfTheCandidateSet {
     UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable,
 }
 
+/// 墓碑树、谱系树、退役树沿叶序第一片容器的容器号（D18（块里携带什么信息） 已定项 24 第 1、5 条：容器号从 1 起，下一个号 = 树里最大号 + 1，
+/// 零根时 1；I-9.21、I-9.22、I-9.36、I-9.37 判它，第 56 批 G2）。
+const FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE: u64 = 1;
+
 /// 装没装旋钮决定墓碑容器容量判哪一条：没装（旧入口）判产品那一条 I-9.21，装了判测试那一条 I-9.22。
 const fn capacity_invariant_of(
     capacity_knob: Option<TombstoneContainerCapacityKnob>,
@@ -11970,7 +11974,10 @@ fn judge_tombstones_on_the_lineage_of_a_version_are_distinct(
 }
 
 /// I-9.36 / I-9.37（第 53 批，判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2，与 I-9.21 / I-9.22 同形）：这一版谱系树或
-/// 退役树按叶序的容器（容器号, 记录数），容器号严格递增（读容器头，不沿根的条目读）；没装旋钮判 I-9.36——最右那片之外每片记录数 == 产品容量
+/// 退役树按叶序的容器（容器号, 记录数），读容器头、不沿根的条目读。有对象时（没装旋钮至少两片，装了至少一片）判容器号与记录数的三句
+/// （第 56 批 G2，判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第三节）：沿叶序第一片容器号是 1、相邻两片加一、
+/// 每片（含最右那片）至少一条——写侧只经两个追加函数开新片，新号 = 最右号 + 1、零根时 1，有新记录才开、没有删片的路
+/// （D18（块里携带什么信息） 已定项 24 第 1、5 条）；再按旋钮：没装判 I-9.36——最右那片之外每片记录数 == 产品容量
 /// （谱系 1359、退役 959；开放容器满了才关），零根或只一片容器没有对象；装了旋钮判 I-9.37——每片记录数 ≤ 旋钮值，零根没有对象。
 fn judge_lineage_or_retirement_container_capacity_of_a_version(
     version_name: &str,
@@ -11989,11 +11996,25 @@ fn judge_lineage_or_retirement_container_capacity_of_a_version(
     if container_numbers_and_record_counts.len() < judged_when_the_tree_has_at_least {
         return;
     }
+    if let Some((first_container_in_leaf_order, _)) = container_numbers_and_record_counts.first() {
+        let first_container_in_leaf_order = *first_container_in_leaf_order;
+        judgements.judge(
+            invariant,
+            first_container_in_leaf_order == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,
+            || {
+                format!(
+                    "{version_name}：{tree_name}（树 {tree}）沿叶序第一片容器的容器号是 {first_container_in_leaf_order}，不是 {FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE}（容器号从 1 起）"
+                )
+            },
+        );
+    }
     for adjacent in container_numbers_and_record_counts.windows(2) {
         let (left_container, right_container) = (adjacent[0].0, adjacent[1].0);
-        judgements.judge(invariant, left_container < right_container, || {
+        let right_container_follows_the_left_by_one =
+            left_container.checked_add(1) == Some(right_container);
+        judgements.judge(invariant, right_container_follows_the_left_by_one, || {
             format!(
-                "{version_name}：{tree_name}（树 {tree}）沿叶序容器号不严格递增：容器 {left_container} 之后是容器 {right_container}"
+                "{version_name}：{tree_name}（树 {tree}）沿叶序容器号不是逐片加一：容器 {left_container} 之后是容器 {right_container}（下一个号 = 最右号 + 1）"
             )
         });
     }
@@ -12001,6 +12022,12 @@ fn judge_lineage_or_retirement_container_capacity_of_a_version(
         container_numbers_and_record_counts.iter().enumerate()
     {
         let record_count = u64::try_from(*record_count).expect("一片容器的记录数装得进 u64");
+        let lineage_or_retirement_container_holds_a_record = record_count > 0;
+        judgements.judge(invariant, lineage_or_retirement_container_holds_a_record, || {
+            format!(
+                "{version_name}：{tree_name}（树 {tree}）的容器 {container} 一条记录都没有（有新记录才开新片）"
+            )
+        });
         match knob_records_per_container {
             None => {
                 let is_the_rightmost =
@@ -12026,8 +12053,10 @@ fn judge_lineage_or_retirement_container_capacity_of_a_version(
     }
 }
 
-/// I-9.21 / I-9.22（判决第二节 T5 取 K1、T6 每版）：这一版墓碑树按叶序的容器，容器号严格递增（读容器头，不沿根的条目读）；
-/// 没装旋钮判 I-9.21——最右那片之外每片记录数 == 815，零根或只一片容器没有对象；装了旋钮判 I-9.22——每片记录数 ≤ 旋钮值，零根没有对象。
+/// I-9.21 / I-9.22（判决第二节 T5 取 K1、T6 每版）：这一版墓碑树按叶序的容器，读容器头、不沿根的条目读。有对象时（没装旋钮至少两片，
+/// 装了至少一片）判容器号与记录数的三句（第 56 批 G2，同 [`judge_lineage_or_retirement_container_capacity_of_a_version`]）：沿叶序第一片容器号是 1、
+/// 相邻两片加一、每片（含最右那片）至少一条；再按旋钮：没装判 I-9.21——最右那片之外每片记录数 == 815，零根或只一片容器没有对象；
+/// 装了旋钮判 I-9.22——每片记录数 ≤ 旋钮值，零根没有对象。
 fn judge_tombstone_container_capacity_of_a_version(
     version_name: &str,
     tombstone_tree: u64,
@@ -12043,18 +12072,43 @@ fn judge_tombstone_container_capacity_of_a_version(
     if containers.len() < judged_when_the_tree_has_at_least {
         return;
     }
+    if let Some(first_in_leaf_order) = containers.first() {
+        judgements.judge(
+            invariant,
+            first_in_leaf_order.container == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,
+            || {
+                format!(
+                    "{version_name}：墓碑树（树 {tombstone_tree}）沿叶序第一片容器的容器号是 {}，不是 {FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE}（容器号从 1 起）",
+                    first_in_leaf_order.container
+                )
+            },
+        );
+    }
     for adjacent in containers.windows(2) {
         let (left, right) = (&adjacent[0], &adjacent[1]);
-        judgements.judge(invariant, left.container < right.container, || {
-            format!(
-                "{version_name}：墓碑树（树 {tombstone_tree}）沿叶序容器号不严格递增：容器 {} 之后是容器 {}",
-                left.container, right.container
-            )
-        });
+        let right_tombstone_container_follows_the_left_by_one =
+            left.container.checked_add(1) == Some(right.container);
+        judgements.judge(
+            invariant,
+            right_tombstone_container_follows_the_left_by_one,
+            || {
+                format!(
+                    "{version_name}：墓碑树（树 {tombstone_tree}）沿叶序容器号不是逐片加一：容器 {} 之后是容器 {}（下一个号 = 最右号 + 1）",
+                    left.container, right.container
+                )
+            },
+        );
     }
     for (position_in_leaf_order, container) in containers.iter().enumerate() {
         let record_count =
             u64::try_from(container.records.len()).expect("一片容器的记录数装得进 u64");
+        let tombstone_container_holds_a_record = record_count > 0;
+        judgements.judge(invariant, tombstone_container_holds_a_record, || {
+            format!(
+                "{version_name}：墓碑树（树 {tombstone_tree}）的容器 {} 一条记录都没有（有新记录才开新片）",
+                container.container
+            )
+        });
         match capacity_knob {
             None => {
                 let is_the_rightmost = position_in_leaf_order + 1 == containers.len();
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
index aea78e0..258593a 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -1,7 +1,8 @@
 // GPU 判器的池级 checker 扫描内核（接在 crash_judge_common.wgsl 与 crash_judge_checker_common.wgsl 之后）：走读内核判完、交接段有效时，
 // 判第三批——扫描方向与记账那九条（I-1.8、I-3.9、I-9.14、I-5.4、I-3.10、I-7.9、I-3.1、I-5.2、I-3.11），再加 I-3.12（每条根各自那张实例表）、
 // I-5.5 与 I-9.28（根表里每一条根的树表），再加里程碑三·3 删除线的删除态扫描（I-9.5、I-3.4、I-4.7、I-4.6、I-9.24，回退候选集每一版）、
-// 墓碑与谱系那一遍（I-9.18、I-9.19、I-9.20、I-9.21、I-9.29 与 I-9.10 墓碑那一个等号，同一份版本清单；退役记录 I-9.32–I-9.35 也在这一遍）与 I-9.6 墓碑那一半
+// 墓碑与谱系那一遍（I-9.18、I-9.19、I-9.20、I-9.21 / I-9.22、I-9.29 与 I-9.10 墓碑那一个等号，同一份版本清单；退役记录 I-9.32–I-9.35、
+// 谱系与退役容器 I-9.36 / I-9.37 也在这一遍）与 I-9.6 墓碑那一半
 // （所选根的墓碑树），违例位并进违例位那几个字，走到第几步那一字写成 5、判不了时下一字写原因。只读不判的读法照 walk.rs 的 *_without_judging 各写一份。
 
 
@@ -2165,6 +2166,9 @@ const DELETION_TOMBSTONE_RECORD_RANGE_START_OFFSET: u32 = 16u;
 const DELETION_TOMBSTONE_RECORD_RANGE_END_OFFSET: u32 = 24u;
 const DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET: u32 = 32u;
 const DELETION_TOMBSTONE_CONTAINER_RECORDS: u32 = 815u;
+// 墓碑树、谱系树、退役树沿叶序第一片容器的容器号（walk.rs FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE，D18（块里携带什么信息） 已定项 24 第 1、5 条：
+// 容器号从 1 起，下一个号 = 最右号 + 1）
+const D_FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE: u32 = 1u;
 // 谱系记录 24 字节（类型 5 登记 24）：T_rb 0、T_old 8、H_cur 16（回退之前的现行那一版的 inode 号水位），读作「T_rb 的父是 T_old」
 // （walk.rs LINEAGE_RECORD_*_OFFSET，D18（块里携带什么信息） 已定项 24 第 5 条）
 const DELETION_LINEAGE_RECORD_BYTES: u32 = 24u;
@@ -2682,7 +2686,10 @@ fn judge_inode_number_watermark_above_every_published_tombstone() {
 // I-9.21 与 I-9.10 墓碑那一个等号，把各版的 (inode 树, 号) → 对象出生代收进跨版本那张表（D_OBJECT_BIRTHS），同一个 (树, 号) 见到第二个出生代就判 I-9.29（主走读没沿最新根的实例表链走到末片时不比）。
 // CPU 记成射程里判不了的那几格（树表读不出、条目宽不到 200、谱系树条目不是恰好一条、谱系树或墓碑树读不全、谱系接不成一条链、头 ID 配不上恰好一棵
 // inode 树或恰好一棵墓碑树、队列根读不出、队列里的号没有记录、inode 树读不全）在这里一格都不置位；CPU 按 (树表单元, txg) 去重只扫一次，
-// 违例位是并集，这里不去重。I-9.21 只按没装旋钮那一支判（层 0 与放量走池级 checker 的旧入口），I-9.22 不置位。不按已发布与否滤墓碑（第二节 T4）。
+// 违例位是并集，这里不去重。三棵容器树装没装只供测试的容量旋钮看扫描内核源码前面的三个常量 D_TOMBSTONE_CONTAINER_CAPACITY_KNOB、
+// D_LINEAGE_CONTAINER_CAPACITY_KNOB、D_RETIREMENT_CONTAINER_CAPACITY_KNOB（每片记录数，0 是没装；crash_judge_gpu.rs 的
+// container_capacity_knob_wgsl_constants 生成，第 56 批 G6）：没装的判产品那一条（I-9.21、I-9.36），装了的判旋钮那一条（I-9.22、I-9.37），
+// 同 walk.rs check_pool_image_given_the_container_capacity_knobs。层 0 与放量编的是三个都为 0 的那一份。不按已发布与否滤墓碑（第二节 T4）。
 // 表越过上限（一棵树的容器、一版的谱系记录、跨版本那张表、两两比不重号的墓碑总数）判不了，交 CPU
 
 var<private> d_container_count: u32;
@@ -3028,7 +3035,7 @@ fn d_read_the_lineage_of_a_version(tt: Node, version_txg: vec2<u32>) -> bool {
         PACKED_TYPE_LINEAGE, DELETION_LINEAGE_RECORD_BYTES)) {
         return false;
     }
-    d_judge_lineage_or_retirement_container_capacity(DELETION_LINEAGE_CONTAINER_RECORDS);
+    d_judge_lineage_or_retirement_container_capacity(DELETION_LINEAGE_CONTAINER_RECORDS, D_LINEAGE_CONTAINER_CAPACITY_KNOB);
     for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
         let base = d_container_base(container_index);
         for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
@@ -3137,17 +3144,54 @@ fn d_tombstone_covers_the_whole_object(base: u32, o: u32) -> bool {
     return lt64(range_start, range_end) && is_zero64(range_start) && eq64(range_end, vec2<u32>(NONE, NONE));
 }
 
-// I-9.21（walk.rs judge_tombstone_container_capacity_of_a_version 没装旋钮那一支）：读进 D_CONTAINERS_OF_A_CONTAINER_TREE 的墓碑容器至少两片时，
-// 沿叶序容器号严格递增（读容器头，不沿根的条目读），最右那片之外每片记录数 == 815；零根或只一片没有对象
-fn d_judge_tombstone_container_capacity() {
-    if (d_container_count < 2u) {
-        return;
+// 容器号与记录数的三句（walk.rs judge_tombstone_container_capacity_of_a_version、judge_lineage_or_retirement_container_capacity_of_a_version
+// 有对象之后那三判，第 56 批 G2）：读进 D_CONTAINERS_OF_A_CONTAINER_TREE 的容器（读容器头，不沿根的条目读）沿叶序第一片容器号是 1、
+// 相邻两片容器号加一（左片号加一不进位越过 2⁶⁴ − 1：同 walk.rs 的 checked_add）、每片（含最右那片）至少一条；不成立置 invariant 那一位。
+// 有没有对象（没装旋钮至少两片、装了至少一片）由调用方先判
+fn d_judge_container_numbers_from_one_by_one_and_no_empty_container(invariant: u32) {
+    if (!eq64(d_container_number(0u), u64_of(D_FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE))) {
+        violate(invariant);
     }
     for (var container_index = 1u; container_index < d_container_count; container_index = container_index + 1u) {
-        if (!lt64(d_container_number(container_index - 1u), d_container_number(container_index))) {
-            violate(INVARIANT_I_9_21);
+        let left_container = d_container_number(container_index - 1u);
+        let right_container = d_container_number(container_index);
+        if (!(lt64(left_container, right_container) && eq64(add64(left_container, u64_of(1u)), right_container))) {
+            violate(invariant);
+        }
+    }
+    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
+        if (d_container_record_count(container_index) == 0u) {
+            violate(invariant);
+        }
+    }
+}
+
+// 装了只供测试的容量旋钮（knob_records 条一片）的那一棵（walk.rs 那两个判法装了旋钮那一支，I-9.22 / I-9.37）：至少一片时判容器号与记录数的三句，
+// 每片记录数 ≤ knob_records；零根没有对象
+fn d_judge_containers_under_the_capacity_knob(knob_records: u32, invariant: u32) {
+    if (d_container_count == 0u) {
+        return;
+    }
+    d_judge_container_numbers_from_one_by_one_and_no_empty_container(invariant);
+    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
+        if (d_container_record_count(container_index) > knob_records) {
+            violate(invariant);
         }
     }
+}
+
+// I-9.21 / I-9.22（walk.rs judge_tombstone_container_capacity_of_a_version）：墓碑旋钮 D_TOMBSTONE_CONTAINER_CAPACITY_KNOB 不为 0 时判 I-9.22
+// （d_judge_containers_under_the_capacity_knob）；为 0（没装）时读进 D_CONTAINERS_OF_A_CONTAINER_TREE 的墓碑容器至少两片才判 I-9.21：
+// 容器号与记录数的三句，最右那片之外每片记录数 == 815；零根或只一片没有对象
+fn d_judge_tombstone_container_capacity() {
+    if (D_TOMBSTONE_CONTAINER_CAPACITY_KNOB != 0u) {
+        d_judge_containers_under_the_capacity_knob(D_TOMBSTONE_CONTAINER_CAPACITY_KNOB, INVARIANT_I_9_22);
+        return;
+    }
+    if (d_container_count < 2u) {
+        return;
+    }
+    d_judge_container_numbers_from_one_by_one_and_no_empty_container(INVARIANT_I_9_21);
     for (var container_index = 0u; container_index + 1u < d_container_count; container_index = container_index + 1u) {
         if (d_container_record_count(container_index) != DELETION_TOMBSTONE_CONTAINER_RECORDS) {
             violate(INVARIANT_I_9_21);
@@ -3155,19 +3199,20 @@ fn d_judge_tombstone_container_capacity() {
     }
 }
 
-// I-9.36（walk.rs judge_lineage_or_retirement_container_capacity_of_a_version 没装旋钮那一支，第 53 批，与 I-9.21 同形）：读进
-// D_CONTAINERS_OF_A_CONTAINER_TREE 的谱系或退役容器至少两片时，沿叶序容器号严格递增（读容器头，不沿根的条目读），最右那片之外每片记录数 ==
-// 产品容量 product_records（谱系 1359、退役 959）；零根或只一片没有对象。I-9.37 不置位（同 I-9.22）。谱系树在 d_read_the_lineage_of_a_version 读完容器那一处调，
-// 退役树在墓碑与谱系那一遍读完退役容器那一处调（与 walk.rs 一样：谱系树要恰好一条条目、容器读得全才判）
-fn d_judge_lineage_or_retirement_container_capacity(product_records: u32) {
-    if (d_container_count < 2u) {
+// I-9.36 / I-9.37（walk.rs judge_lineage_or_retirement_container_capacity_of_a_version，第 53 批，与 I-9.21 / I-9.22 同形）：knob_records 不为 0
+// （那一棵装了旋钮，第 56 批 G6）时判 I-9.37（d_judge_containers_under_the_capacity_knob）；为 0 时读进 D_CONTAINERS_OF_A_CONTAINER_TREE 的谱系或退役容器
+// 至少两片才判 I-9.36：容器号与记录数的三句，最右那片之外每片记录数 == 产品容量 product_records（谱系 1359、退役 959）；零根或只一片没有对象。
+// 谱系树在 d_read_the_lineage_of_a_version 读完容器那一处调，退役树在墓碑与谱系那一遍读完退役容器那一处调（与 walk.rs 一样：谱系树要恰好一条条目、
+// 容器读得全才判）
+fn d_judge_lineage_or_retirement_container_capacity(product_records: u32, knob_records: u32) {
+    if (knob_records != 0u) {
+        d_judge_containers_under_the_capacity_knob(knob_records, INVARIANT_I_9_37);
         return;
     }
-    for (var container_index = 1u; container_index < d_container_count; container_index = container_index + 1u) {
-        if (!lt64(d_container_number(container_index - 1u), d_container_number(container_index))) {
-            violate(INVARIANT_I_9_36);
-        }
+    if (d_container_count < 2u) {
+        return;
     }
+    d_judge_container_numbers_from_one_by_one_and_no_empty_container(INVARIANT_I_9_36);
     for (var container_index = 0u; container_index + 1u < d_container_count; container_index = container_index + 1u) {
         if (d_container_record_count(container_index) != product_records) {
             violate(INVARIANT_I_9_36);
@@ -3832,7 +3877,7 @@ fn d_judge_tombstones_and_the_lineage_of_a_version(base: u32, tree_table_pointer
         }
         if (kind != TREE_KIND_TOMBSTONE) {
             if (is_a_retirement_tree) {
-                d_judge_lineage_or_retirement_container_capacity(DELETION_RETIREMENT_CONTAINER_RECORDS);
+                d_judge_lineage_or_retirement_container_capacity(DELETION_RETIREMENT_CONTAINER_RECORDS, D_RETIREMENT_CONTAINER_CAPACITY_KNOB);
             }
             d_judge_the_retirement_records_and_their_completeness(tt, entry, version_txg, lineage_holds, parent);
             continue;
diff --git a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
index 6f54846..d29365f 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
@@ -520,7 +520,8 @@ impl<'card> GpuCrashJudge<'card> {
 
     /// 同 [`Self::on_card_with_packed_tables`]，编的是调用方给的三份内核源码。判器的结论按判法摘要
     /// （`crate::crash_judge_gpu::gpu_judge_digest`，罩 [`JudgeKernelSources::of_the_judge`] 那三份）入库，
-    /// 编别的源码判出的结论与那个摘要对不上：只给对拍用例在内核里种假违例位用，判定入库的路径不调它。
+    /// 编别的源码判出的结论与那个摘要对不上：只给对拍用例用（在内核里种假违例位；把三棵容器树的旋钮值编进扫描内核，
+    /// [`JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`]），判定入库的路径不调它。
     ///
     /// # Panics
     /// 同 [`Self::new`]。
@@ -1349,6 +1350,9 @@ mod tests {
     /// 恢复 332 638 → 332 666、走读 346 307 → 346 278、扫描 366 870 → 387 154（第 50 批补做实现员副本上用同一个草稿工具量的，它在主工作区那一版上量回
     /// 331 792 / 346 138 / 352 195；四条一开始各留调用点时扫描量到 454 049、超预算，改成单调用点才落回预算里）；第 53 批扫描内核判 I-9.36
     /// （谱系树、退役树读完容器各一个调用点）扫描 387 154 → 387 470（第 53 批实现员副本上用同一个草稿工具量的，它在改之前那一版上量回 387 154）；
+    /// 第 56 批 G2、G6 扫描内核给三棵容器树补「第一片容器号是 1、相邻加一、每片至少一条」、装了旋钮那一支（I-9.22、I-9.37，旋钮值是扫描内核源码前面的常量，
+    /// 值不同不改展开之后的大小）扫描 387 470 → 388 815（第 56 批实现员副本上用同一个数法 `kernel_inlined_size` 量的：草稿工具在改之前那一份上量回
+    /// 332 666 / 387 470 / 346 278，与旧钉值逐个相等；改之后编进旋钮值的那一份同为 388 815，三份都经 naga 解析、校验得过）；
     /// 第 36 批之前是恢复 320 015、扫描 382 259、走读 337 872）：
     /// 改了哪个内核的源码，这里先红，量过新数、确认余量还够之后再改钉值，免得余量悄悄耗光、等上卡时才撞预算。
     #[test]
@@ -1358,8 +1362,8 @@ mod tests {
             (
                 "checker-scan",
                 crate::crash_judge_gpu::checker_scan_kernel_source(),
-                387_470,
-                12_530,
+                388_815,
+                11_185,
             ),
             (
                 "checker-walk",
diff --git a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
index bed7196..b0c67a1 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
@@ -18,6 +18,7 @@
 //! 这一份只放判法：内核原文、表怎么打包、派活参数与结论各是哪几个字、草稿区每个状态多大。怎么上卡、一次派活带几个状态、
 //! 几张卡怎么用，在 `crate::crash_judge_dispatch`——那一份不在判法闭包里（用户 2026-09-29 定「派活和判法拆出来最好」），
 //! 改它不换判法版本；这一份不许引它。
+use singlefs_checker::walk::ContainerCapacityKnobs;
 use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};
 
 use crate::crash_judge_tables::JudgeTablesWords;
@@ -76,8 +77,9 @@ const _: () = assert!(
 /// 墓碑与谱系那一遍在同一份版本清单上判 I-9.18、I-9.19、I-9.20、I-9.21（没装旋钮那一支）与 I-9.10 墓碑那一个等号（并进下标 43），
 /// I-9.29 收各版的号 → 对象出生代进一张跨版本的表判，最新根的实例表沿链读不出时那张表不比、不置位（CPU 报部分没判，第 24 批）；
 /// 一棵树的容器、一版的谱系记录、跨版本那张表、两两比不重号的墓碑总数越过上限的判不了。
-/// 下标 79 的 I-9.22 照下标 73 的先例恒不置位：它只在池级 checker 带旋钮值的入口上判，层 0 与放量走旧入口（`check_pool_image`，恒报不适用），
-/// GPU 也没有旋钮值。I-9.4 第四句（同一片 inode 叶容器里记录号严格递增）走读内核一并判、并进下标 40。
+/// 下标 79 的 I-9.22 只在池级 checker 带旋钮值的入口上判，层 0 与放量走旧入口（`check_pool_image`，恒报不适用）、编的是三棵容器树都没装旋钮的
+/// 那一份内核（[`JudgeKernelSources::of_the_judge`]），那里恒不置位；对拍用例拿 [`JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`]
+/// 把旋钮值编进扫描内核时同判（第 56 批 G6）。I-9.4 第四句（同一片 inode 叶容器里记录号严格递增）走读内核一并判、并进下标 40。
 /// 下标 83 是根环线第 10j 批新立的 I-3.14（实例表每片记录形态合法）：走读内核在实例表链走到的每一片上判四格（kind 只有 0 或 1、
 /// 链指针记录只在最后一条、行的 flags 为 0、最后一条是合法的链指针记录），不成立的一片照 CPU 当解不开——行不收、链不往下走（第 10k 批）。
 /// 下标 84 是第 49 批新立的 I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上）：扫描内核先读最新根的记账树收 inode 号水位那一统计量的行、
@@ -90,7 +92,8 @@ const _: () = assert!(
 /// 用户 2026-09-29 定「配了 GPU 就一定要 GPU 判，不退回 CPU」，第 50 批补做把这四条搬上 GPU、不记判不了；读不全的那一部分不置位（CPU 报射程里判不了），
 /// 叶身份表与两两比的退役记录越过上限判不了。
 /// 下标 90、91 是第 53 批谱系树与退役树的容器容量那两条（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2）：扫描内核在
-/// 墓碑与谱系那一遍判 I-9.36（谱系树读完容器那一处、退役树读完容器那一处，各一个调用点）；下标 91 的 I-9.37 照下标 79 的先例恒不置位。
+/// 墓碑与谱系那一遍判 I-9.36（谱系树读完容器那一处、退役树读完容器那一处，各一个调用点）；下标 91 的 I-9.37 同下标 79：编进了旋钮值的那一份内核
+/// 才判、没装旋钮的那一份恒不置位。第 56 批 G2 给 I-9.21、I-9.22、I-9.36、I-9.37 补「第一片容器号是 1、相邻加一、每片至少一条」，与 CPU 同判。
 /// 用户 2026-10-04 定「改回要求全覆盖」：没有「内核还没判」那一张表。
 pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[
     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
@@ -191,11 +194,14 @@ pub mod verdict_bits {
     pub const UNDECIDABLE: u32 = 31;
 }
 
-/// 判法代码的摘要：结论字号的常量与五份内核源码，连同一个版本前缀。
+/// 判法代码的摘要：结论字号的常量、三棵容器树都没装旋钮的那一份旋钮常量与五份内核源码，连同一个版本前缀。
 #[must_use]
 pub fn gpu_judge_digest() -> [u8; DIGEST_BYTES] {
     let mut message = b"gpu-crash-judge-v5\n".to_vec();
     message.extend_from_slice(verdict_layout_wgsl_constants().as_bytes());
+    message.extend_from_slice(
+        container_capacity_knob_wgsl_constants(ContainerCapacityKnobs::PRODUCT).as_bytes(),
+    );
     message.extend_from_slice(GPU_JUDGE_COMMON_SOURCE.as_bytes());
     message.extend_from_slice(GPU_JUDGE_RECOVERY_SOURCE.as_bytes());
     message.extend_from_slice(GPU_JUDGE_CHECKER_COMMON_SOURCE.as_bytes());
@@ -222,12 +228,43 @@ pub fn checker_walk_kernel_source() -> String {
     )
 }
 
-/// 池级 checker 扫描内核的整份源码。
+/// 一棵容器树的旋钮在扫描内核里的值：每片记录数，没装旋钮写 0（扫描内核按 0 判产品那一条）。
+fn knob_records_in_the_kernel(knob_records_per_container: Option<u64>) -> u32 {
+    knob_records_per_container.map_or(0, |records_per_container| {
+        u32::try_from(records_per_container).expect("旋钮值构造时验过在 1..=1359 里，装得进 u32")
+    })
+}
+
+/// 三棵容器树只供测试的容量旋钮在扫描内核里的常量（WGSL 源码，第 56 批 G6）：每片记录数，0 是没装旋钮。旋钮不落盘，从镜像外给
+/// （同 `singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs`）；接在扫描内核源码的结论字号常量之后。
+/// 三个都为 0 的那一份（[`ContainerCapacityKnobs::PRODUCT`]）进判法摘要。
+#[must_use]
+pub fn container_capacity_knob_wgsl_constants(knobs: ContainerCapacityKnobs) -> String {
+    format!(
+        "const D_TOMBSTONE_CONTAINER_CAPACITY_KNOB: u32 = {}u;\n\
+         const D_LINEAGE_CONTAINER_CAPACITY_KNOB: u32 = {}u;\n\
+         const D_RETIREMENT_CONTAINER_CAPACITY_KNOB: u32 = {}u;\n",
+        knob_records_in_the_kernel(knobs.tombstone.map(|knob| knob.records_per_container())),
+        knob_records_in_the_kernel(knobs.lineage.map(|knob| knob.records_per_container())),
+        knob_records_in_the_kernel(knobs.retirement.map(|knob| knob.records_per_container())),
+    )
+}
+
+/// 池级 checker 扫描内核的整份源码（三棵容器树都没装旋钮：层 0 与放量编的那一份）。
 #[must_use]
 pub fn checker_scan_kernel_source() -> String {
+    checker_scan_kernel_source_given_the_container_capacity_knobs(ContainerCapacityKnobs::PRODUCT)
+}
+
+/// 池级 checker 扫描内核的整份源码，三棵容器树的旋钮值编进内核（[`container_capacity_knob_wgsl_constants`]）。
+#[must_use]
+pub fn checker_scan_kernel_source_given_the_container_capacity_knobs(
+    knobs: ContainerCapacityKnobs,
+) -> String {
     format!(
-        "{}\n{GPU_JUDGE_COMMON_SOURCE}\n{GPU_JUDGE_CHECKER_COMMON_SOURCE}\n{GPU_JUDGE_CHECKER_SCAN_SOURCE}",
-        verdict_layout_wgsl_constants()
+        "{}\n{}\n{GPU_JUDGE_COMMON_SOURCE}\n{GPU_JUDGE_CHECKER_COMMON_SOURCE}\n{GPU_JUDGE_CHECKER_SCAN_SOURCE}",
+        verdict_layout_wgsl_constants(),
+        container_capacity_knob_wgsl_constants(knobs)
     )
 }
 
@@ -244,10 +281,18 @@ impl JudgeKernelSources {
     /// 判法摘要 [`gpu_judge_digest`] 罩住的那三份。
     #[must_use]
     pub fn of_the_judge() -> Self {
+        Self::of_the_judge_given_the_container_capacity_knobs(ContainerCapacityKnobs::PRODUCT)
+    }
+
+    /// 同 [`Self::of_the_judge`]，三棵容器树的旋钮值编进扫描内核（第 56 批 G6）：判出的池级 checker 违例位要与
+    /// `singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs` 带同一组旋钮值逐位相同。旋钮值不进判法摘要：
+    /// 只给对拍用例判旋钮压小写出的镜像，判定入库的路径编 [`Self::of_the_judge`]。
+    #[must_use]
+    pub fn of_the_judge_given_the_container_capacity_knobs(knobs: ContainerCapacityKnobs) -> Self {
         Self {
             recovery: recovery_kernel_source(),
             checker_walk: checker_walk_kernel_source(),
-            checker_scan: checker_scan_kernel_source(),
+            checker_scan: checker_scan_kernel_source_given_the_container_capacity_knobs(knobs),
         }
     }
 }
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index c597a1e..266c9f0 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -81,6 +81,9 @@
 //! CPU 判器线程数 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`（没设取本机可用并行度）：设了而认不出（线程数 0 也算）就 panic，不退回没设时的那一档。
 //! 第 50 批（退役树）：退役记录那四条 GPU 同判、不记判不了——删完的流（最后一批开出退役树、一条退役记录）上退役记录死亡 txg 加一（I-9.32、I-9.35）、
 //! 同一条再追加一条（I-9.33）、死亡 txg 减一（只 I-9.35 漏记），左吸收右的流上退役记录改指幸存的左片（I-9.34、I-9.35）各一条流，都整份比。
+//! 第 56 批（G2、G3、G6）：三棵容器树的旋钮值编进 GPU 扫描内核（`JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`），
+//! CPU 那一侧池级 checker 的违例位改按带同一组旋钮值的新入口判：墓碑、谱系、退役各一条旋钮下的合法流（I-9.22 / I-9.37 两边判过而成立），
+//! 谱系多于旋钮值、谱系两片同号、退役两片同号、谱系跳号（1、3）、谱系从 2 起（2、3）、谱系最右多一片 0 条各一条坏流（CPU 红 I-9.37），都整份比。
 #![cfg(all(feature = "verdict-store", feature = "gpu"))]
 
 #[path = "../../singlefs-harness/tests/common/mod.rs"]
@@ -105,7 +108,10 @@ use common_deletion_in_progress::{
     INSTANCE_OF_THE_PUBLISHES, WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
 };
 use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
-use singlefs_checker::walk::check_pool_image;
+use singlefs_checker::walk::{
+    check_pool_image, check_pool_image_given_the_container_capacity_knobs, ContainerCapacityKnobs,
+    LineageContainerCapacityKnob, RetirementContainerCapacityKnob, TombstoneContainerCapacityKnob,
+};
 use singlefs_checker_tier::crash::{
     judge_layer0_state_range_against_the_version_table, layer0_plan_state_count,
     Layer0OracleViolationKind, Layer0SegmentExpansion, Layer0StateJudgement, Layer0VersionTable,
@@ -2256,7 +2262,7 @@ fn compare_on_every_state_against_the_version_table(
     )
 }
 
-/// [`compare_on_every_state_of_these_segments`] 的本体，版本表照 `versions_without_a_file` 另带「没有文件的那几版」。
+/// [`compare_on_every_state_of_these_segments`] 的本体，版本表照 `versions_without_a_file` 另带「没有文件的那几版」；三棵容器树都没装旋钮。
 fn compare_on_every_state_of_these_segments_against_the_version_table(
     cards: &'static GpuCardPool,
     flow: &RecordedFlow,
@@ -2264,6 +2270,73 @@ fn compare_on_every_state_of_these_segments_against_the_version_table(
     segments: &[Vec<usize>],
     expansion: &(dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion + Sync),
 ) -> Option<Comparison> {
+    compare_on_every_state_of_these_segments_against_the_version_table_given_the_container_capacity_knobs(
+        cards,
+        flow,
+        versions_without_a_file,
+        segments,
+        expansion,
+        ContainerCapacityKnobs::PRODUCT,
+    )
+}
+
+/// 同 [`compare_on_every_state`]，三棵容器树的旋钮值 `knobs` 编进 GPU 扫描内核（`JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`，
+/// 第 56 批 G6），CPU 那一侧的池级 checker 违例位改按带同一组旋钮值的新入口（`check_pool_image_given_the_container_capacity_knobs`）在同一张崩溃镜像上判；
+/// 红位与恢复字照旧按 CPU 判器（`judge_crash_image`，池级 checker 走产品入口的那一份不进比较）。
+fn compare_on_every_state_given_the_container_capacity_knobs(
+    cards: &'static GpuCardPool,
+    flow: &RecordedFlow,
+    knobs: ContainerCapacityKnobs,
+) -> Option<Comparison> {
+    let limit = expand_up_to();
+    let expansion = move |_segment_index: usize, segment: &[usize]| {
+        if segment.len() <= limit {
+            Layer0SegmentExpansion::EveryProperSubset
+        } else {
+            Layer0SegmentExpansion::NotExpanded
+        }
+    };
+    compare_on_every_state_of_these_segments_against_the_version_table_given_the_container_capacity_knobs(
+        cards,
+        flow,
+        &[],
+        &flow.segments,
+        &expansion,
+        knobs,
+    )
+}
+
+/// 池级 checker 一份判定的违例位与判过而成立的位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`）。
+fn violated_and_held_bits_of(verdicts: &[(&'static str, InvariantVerdict)]) -> (u128, u128) {
+    let index_of = |invariant: &str| {
+        IMPLEMENTED_INVARIANTS
+            .iter()
+            .position(|candidate| *candidate == invariant)
+            .expect("池级 checker 报的不变量在清单里")
+    };
+    verdicts.iter().fold(
+        (0u128, 0u128),
+        |(violated, held), (invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(_) => (violated | (1u128 << index_of(invariant)), held),
+            InvariantVerdict::Holds => (violated, held | (1u128 << index_of(invariant))),
+            InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => {
+                (violated, held)
+            }
+        },
+    )
+}
+
+/// [`compare_on_every_state_of_these_segments_against_the_version_table`] 的本体：`knobs` 是 [`ContainerCapacityKnobs::PRODUCT`] 时编判法摘要罩住的那三份内核、
+/// CPU 池级 checker 取判器那一份（产品入口）；否则见 [`compare_on_every_state_given_the_container_capacity_knobs`]。
+fn compare_on_every_state_of_these_segments_against_the_version_table_given_the_container_capacity_knobs(
+    cards: &'static GpuCardPool,
+    flow: &RecordedFlow,
+    versions_without_a_file: &[PublishedVersionWithoutAFile],
+    segments: &[Vec<usize>],
+    expansion: &(dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion + Sync),
+    knobs: ContainerCapacityKnobs,
+) -> Option<Comparison> {
+    let kernel_sources = JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs(knobs);
     let judge_flow = JudgeFlow {
         base: &flow.base,
         writes: &flow.writes,
@@ -2297,8 +2370,13 @@ fn compare_on_every_state_of_these_segments_against_the_version_table(
         lease.card(),
         "建判器与判这条流",
         || {
-            GpuCrashJudge::on_card_with_packed_tables(lease.card(), &packed, states_per_dispatch)
-                .judge_states(0..state_count)
+            GpuCrashJudge::on_card_compiling_these_kernel_sources(
+                lease.card(),
+                &packed,
+                states_per_dispatch,
+                &kernel_sources,
+            )
+            .judge_states(0..state_count)
         },
     )?;
     let gpu_elapsed = gpu_started.elapsed();
@@ -2351,10 +2429,23 @@ fn compare_on_every_state_of_these_segments_against_the_version_table(
                             ),
                             expansion,
                             range,
-                            &mut |_ordinal, _image, judgement| {
+                            &mut |_ordinal, image, judgement| {
+                                let (pool_checker_bits, pool_checker_held_bits) =
+                                    if knobs == ContainerCapacityKnobs::PRODUCT {
+                                        (
+                                            expected_pool_checker_bits(judgement),
+                                            expected_pool_checker_held_bits(judgement),
+                                        )
+                                    } else {
+                                        violated_and_held_bits_of(
+                                            &check_pool_image_given_the_container_capacity_knobs(
+                                                image, knobs,
+                                            ),
+                                        )
+                                    };
                                 judged.push((
                                     expected_bits(judgement),
-                                    expected_pool_checker_bits(judgement),
+                                    pool_checker_bits,
                                     format!("{:?}", judgement.red_items()),
                                     format!(
                                         "consulted={:?} effective={:?} ignored={:?}",
@@ -2363,7 +2454,7 @@ fn compare_on_every_state_of_these_segments_against_the_version_table(
                                         judgement.ignored.outcome
                                     ),
                                     recovery_words_of(judgement),
-                                    expected_pool_checker_held_bits(judgement),
+                                    pool_checker_held_bits,
                                     root_reported_by_the_recovery(&judgement.consulted.outcome),
                                 ));
                             },
@@ -12820,3 +12911,416 @@ fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregiste
         mismatch_listing(&comparison)
     );
 }
+
+// ───────────── 第 56 批 G2、G3、G6：容器号从 1 起、逐片加一、每片至少一条；旋钮值编进 GPU 扫描内核（I-9.21、I-9.22、I-9.36、I-9.37）─────────────
+
+/// 打包记录单元头里的容器号（类身份段：出生树 43、打包记录类型 51、容器号 53、容器出生代 61）。
+const PACKED_UNIT_CONTAINER_NUMBER_OFFSET_IN_THE_HEADER: usize = 53;
+
+/// 只这一棵容器树装 `records_per_container` 条一片的旋钮，另两棵当没装（交给 GPU 扫描内核与 CPU 新入口的同一组旋钮值）。
+fn knobs_of_the_lineage_tree(records_per_container: u64) -> ContainerCapacityKnobs {
+    ContainerCapacityKnobs {
+        tombstone: None,
+        lineage: Some(LineageContainerCapacityKnob::new(records_per_container).expect("1..=1359")),
+        retirement: None,
+    }
+}
+
+fn knobs_of_the_retirement_tree(records_per_container: u64) -> ContainerCapacityKnobs {
+    ContainerCapacityKnobs {
+        tombstone: None,
+        lineage: None,
+        retirement: Some(
+            RetirementContainerCapacityKnob::new(records_per_container).expect("1..=959"),
+        ),
+    }
+}
+
+fn knobs_of_the_tombstone_tree(records_per_container: u64) -> ContainerCapacityKnobs {
+    ContainerCapacityKnobs {
+        tombstone: Some(
+            TombstoneContainerCapacityKnob::new(records_per_container).expect("1..=815"),
+        ),
+        lineage: None,
+        retirement: None,
+    }
+}
+
+/// 带旋钮值的那一条（墓碑树 I-9.22，谱系树与退役树 I-9.37）。
+const KNOB_CAPACITY_INVARIANT_OF_THE_TOMBSTONE_TREE: &str =
+    singlefs_checker::image::TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE;
+const KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES: &str =
+    singlefs_checker::image::LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE;
+
+/// 旋钮下合法的流：GPU 扫描内核编进 `knobs`，每个状态的池级 checker 违例位与 CPU 新入口带同一组旋钮值逐位相同；
+/// CPU 一个状态都不红 `knob_capacity_invariant`、至少一个状态上判过而成立（旋钮那一支真走到了）。
+fn assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_legal_flow(
+    test_name: &str,
+    flow: &RecordedFlow,
+    knobs: ContainerCapacityKnobs,
+    knob_capacity_invariant: &'static str,
+) {
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
+        return;
+    };
+    let Some(comparison) =
+        compare_on_every_state_given_the_container_capacity_knobs(cards, flow, knobs)
+    else {
+        return;
+    };
+    assert!(
+        !comparison
+            .cpu_checker_red_invariants
+            .contains(knob_capacity_invariant)
+            && comparison
+                .cpu_checker_held_invariants
+                .contains(knob_capacity_invariant),
+        "流 {} 上 CPU 新入口带旋钮值：{knob_capacity_invariant} 判过而成立、一个状态都不红（红过的：{:?}）",
+        flow.name,
+        comparison.cpu_checker_red_invariants
+    );
+    assert!(
+        comparison.mismatches.is_empty(),
+        "流 {}（GPU 扫描内核编进旋钮值）：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
+    );
+}
+
+/// 旋钮下坏的流：同上，CPU 新入口要红 `knob_capacity_invariant`，GPU 红在同样的状态、同样的位上。
+fn assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
+    test_name: &str,
+    flow: &RecordedFlow,
+    knobs: ContainerCapacityKnobs,
+    knob_capacity_invariant: &'static str,
+) {
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
+        return;
+    };
+    let Some(comparison) =
+        compare_on_every_state_given_the_container_capacity_knobs(cards, flow, knobs)
+    else {
+        return;
+    };
+    assert!(
+        comparison
+            .cpu_checker_red_invariants
+            .contains(knob_capacity_invariant),
+        "流 {} 上 CPU 新入口带旋钮值要红 {knob_capacity_invariant}（红过的：{:?}）",
+        flow.name,
+        comparison.cpu_checker_red_invariants
+    );
+    assert!(
+        comparison.mismatches.is_empty(),
+        "流 {}（GPU 扫描内核编进旋钮值）：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
+    );
+}
+
+/// G6：退役树两片各 1 条（[`record_two_leaves_retired_with_one_retirement_record_per_container`]），GPU 扫描内核编进退役旋钮 1：
+/// 旋钮下的合法流，I-9.37 两边都判过而成立、一个状态都不红（CPU 产品入口在这条流上红 I-9.36，另一条用例钉）。
+#[test]
+fn the_gpu_judge_given_the_retirement_knob_agrees_with_the_cpu_on_two_retirement_containers_of_one_record(
+) {
+    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_legal_flow(
+        "the_gpu_judge_given_the_retirement_knob_agrees_with_the_cpu_on_two_retirement_containers_of_one_record",
+        &record_two_leaves_retired_with_one_retirement_record_per_container(
+            "crash-judge-gpu-retirement-knob-two-containers",
+        ),
+        knobs_of_the_retirement_tree(1),
+        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
+    );
+}
+
+/// G6：谱系树两片各 1 条（[`record_two_rollbacks_with_one_lineage_record_per_container`]），GPU 扫描内核编进谱系旋钮 1：合法流。
+#[test]
+fn the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_on_two_lineage_containers_of_one_record(
+) {
+    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_legal_flow(
+        "the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_on_two_lineage_containers_of_one_record",
+        &record_two_rollbacks_with_one_lineage_record_per_container(
+            "crash-judge-gpu-lineage-knob-two-containers",
+        ),
+        knobs_of_the_lineage_tree(1),
+        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
+    );
+}
+
+/// G6：墓碑树两片各 1 条（[`record_two_empty_inodes_unlinked_with_one_tombstone_per_container`]），GPU 扫描内核编进墓碑旋钮 1：合法流，
+/// I-9.22 两边都判过而成立（CPU 产品入口在这条流上红 I-9.21）。
+#[test]
+fn the_gpu_judge_given_the_tombstone_knob_agrees_with_the_cpu_on_two_tombstone_containers_of_one_record(
+) {
+    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_legal_flow(
+        "the_gpu_judge_given_the_tombstone_knob_agrees_with_the_cpu_on_two_tombstone_containers_of_one_record",
+        &record_two_empty_inodes_unlinked_with_one_tombstone_per_container(
+            "crash-judge-gpu-tombstone-knob-two-containers",
+        ),
+        knobs_of_the_tombstone_tree(1),
+        KNOB_CAPACITY_INVARIANT_OF_THE_TOMBSTONE_TREE,
+    );
+}
+
+/// 新池新建文件、建 1 个空 inode（txg 4），分配器装上谱系容器一片 `lineage_records_per_container` 条的旋钮，挂着回退到第一个文件那一版
+/// （txg 3）`rollbacks` 次（同 [`record_two_rollbacks_with_one_lineage_record_per_container`]）。
+fn record_rollbacks_under_the_lineage_knob(
+    tag: &str,
+    name: &'static str,
+    lineage_records_per_container: u64,
+    rollbacks: usize,
+) -> RecordedFlow {
+    let mut pool = build_pool(tag);
+    let first_file_version = pool.output.clone();
+    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+    pool.allocator
+        .set_deletion_switches(singlefs_core::unlink::DeletionSwitches {
+            batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
+            tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
+            lineage_container_capacity: LineageContainerCapacity::CappedForTests(
+                singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(
+                    lineage_records_per_container,
+                )
+                .expect("1..=1359"),
+            ),
+            retirement_container_capacity:
+                singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
+            container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
+        });
+    // 迭代上界：`rollbacks`。
+    for _ in 0..rollbacks {
+        roll_back_to(&mut pool, &first_file_version);
+    }
+    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
+    recorded_flow_of_the_pool(&pool, name, versions)
+}
+
+/// G3 ③ 在 GPU 上（I-9.37 多于旋钮值那一判）：谱系容器一片 2 条写出两条谱系记录（一片 2 条），GPU 与 CPU 新入口都给谱系旋钮 1：那片装了 2 条，
+/// CPU 红 I-9.37，GPU 同判。
+#[test]
+fn the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_a_lineage_container_holds_more_records_than_the_knob(
+) {
+    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_a_lineage_container_holds_more_records_than_the_knob",
+        &record_rollbacks_under_the_lineage_knob(
+            "crash-judge-gpu-lineage-container-past-the-knob",
+            "two-rollbacks-two-lineage-records-in-one-container",
+            2,
+            2,
+        ),
+        knobs_of_the_lineage_tree(1),
+        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
+    );
+}
+
+fn is_a_container_of_record_type(bytes: &[u8], record_type: u16) -> bool {
+    unit_class(bytes) == 3 && packed_record_type(bytes) == record_type
+}
+
+/// 容器树根（层级 1）第 `entry_index` 条条目身份引用里的容器号改成 `container_number`（分隔 key 与子指针不动）。
+fn set_the_container_number_of_an_entry(
+    bytes: &mut [u8],
+    entry_index: usize,
+    container_number: u64,
+) {
+    assert!(
+        entry_index < usize::from(index_node_entry_count(bytes)),
+        "根里有第 {entry_index} 条条目"
+    );
+    let field = first_entry_of_a_container_number_node()
+        + entry_index * usize::from(CONTAINER_TREE_ENTRY_BYTES)
+        + CONTAINER_TREE_ENTRY_CONTAINER_NUMBER_OFFSET;
+    bytes[field..field + 8].copy_from_slice(&container_number.to_le_bytes());
+}
+
+/// 流里最后写的那片 `record_type` 容器（两份副本）头里的容器号与最后写的那棵树根（两份副本）里指它的那一条（第 `entry_index` 条）条目的身份容器号
+/// 一起改成 `container_number`，各自重封到根（身份照旧与子头相符）。最后写的那片容器只被最后那几版的根指着、那几版的根就是最后写的那个根。
+fn renumber_the_last_container_of_the_flow(
+    flow: &mut RecordedFlow,
+    record_type: u16,
+    entry_index: usize,
+    container_number: u64,
+) {
+    let containers = last_unit_copies(
+        flow,
+        "最后写的那片容器",
+        &|bytes| is_a_container_of_record_type(bytes, record_type),
+        false,
+    );
+    corrupt_units_and_reseal_up_to_the_root(flow, &containers, &|bytes| {
+        bytes[PACKED_UNIT_CONTAINER_NUMBER_OFFSET_IN_THE_HEADER
+            ..PACKED_UNIT_CONTAINER_NUMBER_OFFSET_IN_THE_HEADER + 8]
+            .copy_from_slice(&container_number.to_le_bytes());
+    });
+    let roots = last_unit_copies(
+        flow,
+        "最后写的那棵容器树根",
+        &|bytes| is_a_container_tree_root_over_containers_of_record_type(bytes, record_type),
+        false,
+    );
+    corrupt_units_and_reseal_up_to_the_root(flow, &roots, &|bytes| {
+        set_the_container_number_of_an_entry(bytes, entry_index, container_number);
+    });
+}
+
+/// G3 ①（GPU 那一份）：谱系树两片各 1 条，第二片容器与根里指它的条目的容器号 2 → 1（两片同号，攻方 `mut-w-same`）。GPU 与 CPU 新入口都给谱系旋钮 1：
+/// 记录数都在旋钮之内，CPU 只因「相邻两片加一」红 I-9.37，GPU 同判。
+#[test]
+fn the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_two_lineage_containers_share_a_number(
+) {
+    let mut flow = record_rollbacks_under_the_lineage_knob(
+        "crash-judge-gpu-lineage-containers-share-a-number",
+        "two-rollbacks-lineage-containers-numbered-one-and-one",
+        1,
+        2,
+    );
+    renumber_the_last_container_of_the_flow(&mut flow, PACKED_RECORD_TYPE_LINEAGE, 1, 1);
+    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_two_lineage_containers_share_a_number",
+        &flow,
+        knobs_of_the_lineage_tree(1),
+        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
+    );
+}
+
+/// G3 ①（GPU 那一份）：退役树两片各 1 条，第二片的容器号 2 → 1（两片同号）。GPU 与 CPU 新入口都给退役旋钮 1，CPU 红 I-9.37，GPU 同判。
+#[test]
+fn the_gpu_judge_given_the_retirement_knob_agrees_with_the_cpu_when_two_retirement_containers_share_a_number(
+) {
+    let mut flow = record_two_leaves_retired_with_one_retirement_record_per_container(
+        "crash-judge-gpu-retirement-containers-share-a-number",
+    );
+    flow.name = "two-leaves-retired-retirement-containers-numbered-one-and-one";
+    renumber_the_last_container_of_the_flow(&mut flow, PACKED_RECORD_TYPE_RETIREMENT, 1, 1);
+    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_given_the_retirement_knob_agrees_with_the_cpu_when_two_retirement_containers_share_a_number",
+        &flow,
+        knobs_of_the_retirement_tree(1),
+        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
+    );
+}
+
+/// G2（GPU 那一份，攻方 `mut-w-gap2`）：谱系树两片各 1 条，第二片的容器号 2 → 3（1、3）。CPU 新入口红 I-9.37（不是逐片加一），GPU 同判。
+#[test]
+fn the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_lineage_containers_skip_a_number()
+{
+    let mut flow = record_rollbacks_under_the_lineage_knob(
+        "crash-judge-gpu-lineage-containers-skip-a-number",
+        "two-rollbacks-lineage-containers-numbered-one-and-three",
+        1,
+        2,
+    );
+    renumber_the_last_container_of_the_flow(&mut flow, PACKED_RECORD_TYPE_LINEAGE, 1, 3);
+    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_lineage_containers_skip_a_number",
+        &flow,
+        knobs_of_the_lineage_tree(1),
+        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
+    );
+}
+
+/// G2（GPU 那一份，攻方 `mut-w-start2`）：谱系树两片各 1 条，流里每一片谱系容器头里的容器号与每个谱系树根里每条条目的身份容器号都加 1（2、3）：
+/// 回退一次那一版的谱系树只一片、号 2，回退两次那一版两片 2、3。CPU 新入口红 I-9.37（第一片不是 1），GPU 同判。
+#[test]
+fn the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_lineage_containers_start_from_two()
+{
+    let mut flow = record_rollbacks_under_the_lineage_knob(
+        "crash-judge-gpu-lineage-containers-start-from-two",
+        "two-rollbacks-lineage-containers-numbered-two-and-three",
+        1,
+        2,
+    );
+    let every_lineage_container: Vec<usize> = flow
+        .writes
+        .iter()
+        .enumerate()
+        .filter(|(_, write)| {
+            write.kind == StepKind::UnitWrite
+                && write.bytes().is_some_and(|bytes| {
+                    is_a_container_of_record_type(bytes, PACKED_RECORD_TYPE_LINEAGE)
+                })
+        })
+        .map(|(index, _)| index)
+        .collect();
+    assert_eq!(
+        every_lineage_container.len(),
+        4,
+        "两片谱系容器各写一次、每次两份副本"
+    );
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &every_lineage_container, &|bytes| {
+        let container_number =
+            read_u64_at(bytes, PACKED_UNIT_CONTAINER_NUMBER_OFFSET_IN_THE_HEADER);
+        bytes[PACKED_UNIT_CONTAINER_NUMBER_OFFSET_IN_THE_HEADER
+            ..PACKED_UNIT_CONTAINER_NUMBER_OFFSET_IN_THE_HEADER + 8]
+            .copy_from_slice(&(container_number + 1).to_le_bytes());
+    });
+    let every_lineage_root: Vec<usize> = flow
+        .writes
+        .iter()
+        .enumerate()
+        .filter(|(_, write)| {
+            write.kind == StepKind::UnitWrite && write.bytes().is_some_and(is_the_lineage_tree_root)
+        })
+        .map(|(index, _)| index)
+        .collect();
+    assert_eq!(
+        every_lineage_root.len(),
+        4,
+        "两次回退各写一个谱系树根、每次两份副本"
+    );
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &every_lineage_root, &|bytes| {
+        // 迭代上界：根里的条目数（这条流里至多 2）。
+        for entry_index in 0..usize::from(index_node_entry_count(bytes)) {
+            let field = first_entry_of_a_container_number_node()
+                + entry_index * usize::from(CONTAINER_TREE_ENTRY_BYTES)
+                + CONTAINER_TREE_ENTRY_CONTAINER_NUMBER_OFFSET;
+            let container_number = read_u64_at(bytes, field);
+            set_the_container_number_of_an_entry(bytes, entry_index, container_number + 1);
+        }
+    });
+    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_lineage_containers_start_from_two",
+        &flow,
+        knobs_of_the_lineage_tree(1),
+        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
+    );
+}
+
+/// G2（GPU 那一份，攻方 `mut-w-eager`）：谱系容器一片 1 条回退三次（三片各 1 条），最后写的那片清空（记录数与声明长度写 0、记录区写 0）：
+/// 最右多一片 0 条。CPU 新入口红 I-9.37（每片至少一条），GPU 同判。
+#[test]
+fn the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_the_rightmost_lineage_container_is_empty(
+) {
+    let mut flow = record_rollbacks_under_the_lineage_knob(
+        "crash-judge-gpu-lineage-rightmost-container-empty",
+        "three-rollbacks-rightmost-lineage-container-empty",
+        1,
+        3,
+    );
+    let last_container = last_unit_copies(
+        &flow,
+        "最后写的那片谱系容器",
+        &|bytes| is_a_container_of_record_type(bytes, PACKED_RECORD_TYPE_LINEAGE),
+        false,
+    );
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &last_container, &|bytes| {
+        assert_eq!(
+            read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET),
+            1,
+            "最后写的那片谱系容器一条记录"
+        );
+        bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2]
+            .copy_from_slice(&0u16.to_le_bytes());
+        bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2]
+            .copy_from_slice(&0u16.to_le_bytes());
+        bytes[PACKED_UNIT_RECORDS_START..].fill(0);
+    });
+    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_the_rightmost_lineage_container_is_empty",
+        &flow,
+        knobs_of_the_lineage_tree(1),
+        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
+    );
+}
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 09b0882..979b679 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -16370,3 +16370,806 @@ fn an_inode_number_watermark_step_raised_above_the_watermark_reddens_only_the_wa
         "台阶抬到水位之上：只红 I-9.30：{verdicts:?}"
     );
 }
+
+// ───────────────────────── 第 56 批 G2、G3：容器号从 1 起、逐片加一、每片至少一条（I-9.21、I-9.22、I-9.36、I-9.37） ─────────────────────────
+
+/// I-9.21 / I-9.22 / I-9.36 / I-9.37 判的三棵容器树（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第三节 G2、G3）：
+/// 墓碑树判 I-9.21 / I-9.22，谱系树与退役树判 I-9.36 / I-9.37。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum ContainerTreeOfTheCapacityInvariants {
+    Tombstone,
+    Lineage,
+    Retirement,
+}
+
+impl ContainerTreeOfTheCapacityInvariants {
+    fn tree_kind(self) -> u16 {
+        match self {
+            ContainerTreeOfTheCapacityInvariants::Tombstone => TREE_KIND_TOMBSTONE,
+            ContainerTreeOfTheCapacityInvariants::Lineage => TREE_KIND_LINEAGE,
+            ContainerTreeOfTheCapacityInvariants::Retirement => TREE_KIND_RETIREMENT,
+        }
+    }
+
+    /// 带旋钮值的入口判这一棵的那一条：墓碑树 I-9.22，谱系树与退役树 I-9.37。
+    fn knob_capacity_invariant(self) -> &'static str {
+        match self {
+            ContainerTreeOfTheCapacityInvariants::Tombstone => {
+                TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE
+            }
+            ContainerTreeOfTheCapacityInvariants::Lineage
+            | ContainerTreeOfTheCapacityInvariants::Retirement => {
+                LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE
+            }
+        }
+    }
+
+    /// 违例说明里怎么称呼这一棵（`walk.rs` 的说明照这个写）。
+    fn name_in_the_violation(self) -> &'static str {
+        match self {
+            ContainerTreeOfTheCapacityInvariants::Tombstone => "墓碑树",
+            ContainerTreeOfTheCapacityInvariants::Lineage => "谱系树",
+            ContainerTreeOfTheCapacityInvariants::Retirement => "退役树",
+        }
+    }
+
+    /// 池级 checker 带旋钮值的入口：只这一棵装 `records_per_container` 条一片的旋钮，另两棵当没装。
+    fn checker_knobs(
+        self,
+        records_per_container: u64,
+    ) -> singlefs_checker::walk::ContainerCapacityKnobs {
+        let is_this_tree = |tree: ContainerTreeOfTheCapacityInvariants| tree == self;
+        singlefs_checker::walk::ContainerCapacityKnobs {
+            tombstone: is_this_tree(ContainerTreeOfTheCapacityInvariants::Tombstone).then(|| {
+                singlefs_checker::walk::TombstoneContainerCapacityKnob::new(records_per_container)
+                    .expect("1..=815")
+            }),
+            lineage: is_this_tree(ContainerTreeOfTheCapacityInvariants::Lineage).then(|| {
+                singlefs_checker::walk::LineageContainerCapacityKnob::new(records_per_container)
+                    .expect("1..=1359")
+            }),
+            retirement: is_this_tree(ContainerTreeOfTheCapacityInvariants::Retirement).then(|| {
+                singlefs_checker::walk::RetirementContainerCapacityKnob::new(records_per_container)
+                    .expect("1..=959")
+            }),
+        }
+    }
+
+    /// 写入口：只这一棵压到 `records_per_container` 条一片，另两棵与节点照产品。
+    fn write_capacities(
+        self,
+        records_per_container: u64,
+    ) -> singlefs_core::container_tree::ContainerTreeCapacities {
+        let is_this_tree = |tree: ContainerTreeOfTheCapacityInvariants| tree == self;
+        singlefs_core::container_tree::ContainerTreeCapacities {
+            tombstone_container: if is_this_tree(ContainerTreeOfTheCapacityInvariants::Tombstone) {
+                singlefs_core::tombstone_tree::TombstoneContainerCapacity::CappedForTests(
+                    singlefs_core::tombstone_tree::TombstoneRecordsPerContainerCappedForTests::new(
+                        records_per_container,
+                    )
+                    .expect("1..=815"),
+                )
+            } else {
+                singlefs_core::tombstone_tree::TombstoneContainerCapacity::FromTheContainerFormat
+            },
+            lineage_container: if is_this_tree(ContainerTreeOfTheCapacityInvariants::Lineage) {
+                singlefs_core::lineage_tree::LineageContainerCapacity::CappedForTests(
+                    singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(
+                        records_per_container,
+                    )
+                    .expect("1..=1359"),
+                )
+            } else {
+                singlefs_core::lineage_tree::LineageContainerCapacity::FromTheContainerFormat
+            },
+            retirement_container: if is_this_tree(ContainerTreeOfTheCapacityInvariants::Retirement)
+            {
+                singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
+                    singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(
+                        records_per_container,
+                    )
+                    .expect("1..=959"),
+                )
+            } else {
+                singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat
+            },
+            node_entries:
+                singlefs_core::container_tree::ContainerTreeNodeEntries::FromTheNodeFormat,
+        }
+    }
+
+    /// 这一版这一棵每片容器的记录数，按叶序。
+    fn record_counts_of_each_container(self, output: &TransactionOutput) -> Vec<usize> {
+        match self {
+            ContainerTreeOfTheCapacityInvariants::Tombstone => output
+                .tombstone_containers
+                .iter()
+                .map(|container| container.contents.records.len())
+                .collect(),
+            ContainerTreeOfTheCapacityInvariants::Lineage => output
+                .lineage_containers
+                .iter()
+                .map(|container| container.contents.records.len())
+                .collect(),
+            ContainerTreeOfTheCapacityInvariants::Retirement => output
+                .retirement_containers
+                .iter()
+                .map(|container| container.contents.records.len())
+                .collect(),
+        }
+    }
+}
+
+/// 在这一棵树里写 `records` 条记录、写入口只这一棵压到 `records_per_container` 条一片（另两棵照产品）：
+/// 墓碑树——建 `records` 个空 inode（2 号起），逐个 unlink 到底，每个一条墓碑；
+/// 谱系树——建 1 个空 inode，分配器装上谱系旋钮，挂着回退到第一个文件那一版 `records` 次，每次一条谱系记录；
+/// 退役树——再建 232 个空 inode（叶 [1..=233]、[234]），unlink 234 到底（最右那片删空、退役一条），之后每次建 1 个（最左那片满了，开新叶）、
+/// unlink 它到底（又退役一条），共 `records` 次。
+fn pool_with_records_in_one_container_tree_under_its_knob(
+    tree: ContainerTreeOfTheCapacityInvariants,
+    tag: &str,
+    records_per_container: u64,
+    records: u64,
+) -> BuiltPool {
+    let capacities = tree.write_capacities(records_per_container);
+    let pool = match tree {
+        ContainerTreeOfTheCapacityInvariants::Tombstone => {
+            let mut pool = common_deletion_in_progress::pool_with_new_inodes(tag, records);
+            // 迭代上界：`records`。
+            for inode in 2..2 + records {
+                unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, inode, capacities);
+            }
+            pool
+        }
+        ContainerTreeOfTheCapacityInvariants::Lineage => {
+            let mut pool = build_pool(tag);
+            let first_file_version = pool.output.clone();
+            common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+            pool.allocator
+                .set_deletion_switches(singlefs_core::unlink::DeletionSwitches {
+                    batch_size: singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
+                    tombstone_container_capacity: capacities.tombstone_container,
+                    lineage_container_capacity: capacities.lineage_container,
+                    retirement_container_capacity: capacities.retirement_container,
+                    container_tree_node_entries: capacities.node_entries,
+                });
+            // 迭代上界：`records`。
+            for _ in 0..records {
+                common_deletion_in_progress::roll_back_to(&mut pool, &first_file_version);
+            }
+            pool
+        }
+        ContainerTreeOfTheCapacityInvariants::Retirement => {
+            let leaf_records = singlefs_format::INODE_LEAF_RECORDS;
+            let mut pool = common_deletion_in_progress::pool_with_new_inodes(tag, leaf_records);
+            // 迭代上界：`records`。第 0 次删的是建出来的最后一个号（独占最右那片叶），之后每次先建一个再删它。
+            for retirement_index in 0..records {
+                if retirement_index > 0 {
+                    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(
+                        &mut pool, 1,
+                    );
+                }
+                let inode = pool.output.inode_number_watermark() - 1;
+                unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, inode, capacities);
+            }
+            pool
+        }
+    };
+    assert_eq!(
+        tree.record_counts_of_each_container(&pool.output)
+            .iter()
+            .sum::<usize>(),
+        usize::try_from(records).expect("装得进"),
+        "{tree:?} 里一共 {records} 条记录"
+    );
+    pool
+}
+
+/// 同上，再接着建 30 次空 inode，把写容器树的那几版都转出回退候选集：留在候选集里的每一版都是同一棵容器树（照抄、不重写），
+/// 改坏最新那一版的容器树就是改坏候选集里每一版的，没有一版还拿着改之前的身份或记录。交回改之前的镜像，带旋钮值的入口上一条都不红、
+/// 那一条（I-9.22 / I-9.37）真被评估过且成立。
+fn image_with_records_in_one_container_tree_and_the_root_ring_turned_over(
+    tree: ContainerTreeOfTheCapacityInvariants,
+    tag: &str,
+    records: u64,
+) -> MemoryPool {
+    let mut pool = pool_with_records_in_one_container_tree_under_its_knob(tree, tag, 1, records);
+    common_deletion_in_progress::turn_the_root_ring_over_by_creating_inodes(&mut pool);
+    assert_eq!(
+        tree.record_counts_of_each_container(&pool.output),
+        vec![1; usize::try_from(records).expect("装得进")],
+        "{tree:?} 一片 1 条：{records} 片"
+    );
+    let image = pool.memory_pool();
+    let verdicts = singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
+        &image,
+        tree.checker_knobs(1),
+    );
+    assert_eq!(
+        violated_invariants(&verdicts),
+        Vec::<&str>::new(),
+        "改之前（{tree:?}，带旋钮值）一条都不红：{verdicts:?}"
+    );
+    assert_eq!(
+        verdict_among(&verdicts, tree.knob_capacity_invariant()),
+        InvariantVerdict::Holds,
+        "改之前 {} 真被评估过且成立",
+        tree.knob_capacity_invariant()
+    );
+    image
+}
+
+/// 最新的根那一版这一棵树（根层级 1、直接管容器）每片容器的槽，按叶序。
+fn container_slots_of_the_newest_tree(
+    image: &MemoryPool,
+    tree: ContainerTreeOfTheCapacityInvariants,
+) -> Vec<singlefs_core::address::SlotNumber> {
+    let root_pointer =
+        common_pointer_head_corruption::tree_table_entry_of_kind(image, tree.tree_kind()).root;
+    let root =
+        singlefs_core::unit::parse_index_node(&common_pointer_head_corruption::unit_bytes_at(
+            image,
+            common_pointer_head_corruption::the_slot_of(&root_pointer),
+            common_pointer_head_corruption::node_bytes(),
+        ))
+        .expect("容器树根解得开");
+    assert_eq!(root.level, 1, "{tree:?} 的根直接管容器");
+    root.entries
+        .iter()
+        .map(|entry| {
+            common_pointer_head_corruption::the_slot_of(
+                &singlefs_core::pointer::NodePointer::read_from(
+                    &mut singlefs_core::bytes::ByteReader::at(
+                        entry,
+                        CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET,
+                    ),
+                ),
+            )
+        })
+        .collect()
+}
+
+/// 最新的根那一版这一棵树每片容器头里的容器号与根里指它那条条目的身份容器号一起改成 `container_numbers_in_leaf_order` 里的号
+/// （身份照旧与子头相符；分隔 key 与子指针不动），改过的单元重封、新的整单元校验和沿引用链补到根。
+fn renumber_the_containers_of_the_newest_tree(
+    image: &mut MemoryPool,
+    tree: ContainerTreeOfTheCapacityInvariants,
+    container_numbers_in_leaf_order: &[u64],
+) {
+    let container_slots = container_slots_of_the_newest_tree(image, tree);
+    assert_eq!(
+        container_slots.len(),
+        container_numbers_in_leaf_order.len(),
+        "{tree:?} 的容器片数与要写的号数相同"
+    );
+    for (entry_index, (container_slot, container_number)) in container_slots
+        .into_iter()
+        .zip(container_numbers_in_leaf_order)
+        .enumerate()
+    {
+        common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
+            image,
+            container_slot,
+            common_pointer_head_corruption::data_unit_bytes(),
+            |container| {
+                set_u64(
+                    container,
+                    PACKED_UNIT_CONTAINER_NUMBER_OFFSET,
+                    *container_number,
+                )
+            },
+        );
+        change_an_entry_of_the_root_of_kind(image, tree.tree_kind(), entry_index, |entry| {
+            set_u64(
+                entry,
+                CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET,
+                *container_number,
+            );
+        });
+    }
+}
+
+/// 打包记录单元的记录区起点（136；声明长度的偏移是上面的 `PACKED_UNIT_DECLARED_LENGTH_OFFSET`）。
+const PACKED_UNIT_RECORDS_START: usize = 136;
+
+/// 最新的根那一版这一棵树最右那片容器清空：记录数与声明长度写 0、记录区写 0（容器号、身份不动），重封、补校验和。
+fn empty_the_rightmost_container_of_the_newest_tree(
+    image: &mut MemoryPool,
+    tree: ContainerTreeOfTheCapacityInvariants,
+) {
+    let rightmost_slot = *container_slots_of_the_newest_tree(image, tree)
+        .last()
+        .expect("至少一片容器");
+    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
+        image,
+        rightmost_slot,
+        common_pointer_head_corruption::data_unit_bytes(),
+        |container| {
+            assert_eq!(
+                u16::from_le_bytes([
+                    container[PACKED_UNIT_RECORD_COUNT_OFFSET],
+                    container[PACKED_UNIT_RECORD_COUNT_OFFSET + 1]
+                ]),
+                1,
+                "最右那片容器一条记录"
+            );
+            set_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET, 0);
+            set_u16(container, PACKED_UNIT_DECLARED_LENGTH_OFFSET, 0);
+            container[PACKED_UNIT_RECORDS_START..].fill(0);
+        },
+    );
+}
+
+/// 带旋钮值的入口（只这一棵装一片 1 条）上只红这一棵的那一条（I-9.22 / I-9.37），违例说明里有 `detail_fragment`。
+fn assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
+    image: &MemoryPool,
+    tree: ContainerTreeOfTheCapacityInvariants,
+    detail_fragment: &str,
+) {
+    let verdicts = singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
+        image,
+        tree.checker_knobs(1),
+    );
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [tree.knob_capacity_invariant()],
+        "{tree:?}：只红 {}：{verdicts:?}",
+        tree.knob_capacity_invariant()
+    );
+    assert!(
+        matches!(
+            verdict_among(&verdicts, tree.knob_capacity_invariant()),
+            InvariantVerdict::Violated(detail)
+                if detail.contains(tree.name_in_the_violation()) && detail.contains(detail_fragment)
+        ),
+        "{tree:?}：违例说明写出「{detail_fragment}」：{:?}",
+        verdict_among(&verdicts, tree.knob_capacity_invariant())
+    );
+}
+
+/// 攻方 `mut-w-gap2` 那一形（写侧下一个号 = 最右号 + 2）：两片容器 1、2 号改成 1、3 号。
+fn assert_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant(
+    tree: ContainerTreeOfTheCapacityInvariants,
+    tag: &str,
+) {
+    let mut image =
+        image_with_records_in_one_container_tree_and_the_root_ring_turned_over(tree, tag, 2);
+    renumber_the_containers_of_the_newest_tree(&mut image, tree, &[1, 3]);
+    assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
+        &image,
+        tree,
+        "沿叶序容器号不是逐片加一：容器 1 之后是容器 3",
+    );
+}
+
+/// 攻方 `mut-w-start2` 那一形（零根时的号写成 2）：两片容器 1、2 号改成 2、3 号。
+fn assert_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant(
+    tree: ContainerTreeOfTheCapacityInvariants,
+    tag: &str,
+) {
+    let mut image =
+        image_with_records_in_one_container_tree_and_the_root_ring_turned_over(tree, tag, 2);
+    renumber_the_containers_of_the_newest_tree(&mut image, tree, &[2, 3]);
+    assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
+        &image,
+        tree,
+        "沿叶序第一片容器的容器号是 2，不是 1",
+    );
+}
+
+/// 攻方 `mut-w-eager` 那一形（满了就先开下一片、最右多一片 0 条）：三片各 1 条，最右那片清空。
+fn assert_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant(
+    tree: ContainerTreeOfTheCapacityInvariants,
+    tag: &str,
+) {
+    let mut image =
+        image_with_records_in_one_container_tree_and_the_root_ring_turned_over(tree, tag, 3);
+    empty_the_rightmost_container_of_the_newest_tree(&mut image, tree);
+    assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
+        &image,
+        tree,
+        "容器 3 一条记录都没有",
+    );
+}
+
+/// 攻方 `mut-w-same` 那一形（下一个号 = 最右号）：两片容器 1、2 号改成 1、1 号。
+fn assert_containers_numbered_one_and_one_redden_only_the_knob_capacity_invariant(
+    tree: ContainerTreeOfTheCapacityInvariants,
+    tag: &str,
+) {
+    let mut image =
+        image_with_records_in_one_container_tree_and_the_root_ring_turned_over(tree, tag, 2);
+    renumber_the_containers_of_the_newest_tree(&mut image, tree, &[1, 1]);
+    assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
+        &image,
+        tree,
+        "沿叶序容器号不是逐片加一：容器 1 之后是容器 1",
+    );
+}
+
+#[test]
+fn tombstone_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant() {
+    assert_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant(
+        ContainerTreeOfTheCapacityInvariants::Tombstone,
+        "known-bad-tombstone-containers-numbered-one-and-three",
+    );
+}
+
+#[test]
+fn lineage_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant() {
+    assert_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant(
+        ContainerTreeOfTheCapacityInvariants::Lineage,
+        "known-bad-lineage-containers-numbered-one-and-three",
+    );
+}
+
+#[test]
+fn retirement_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant() {
+    assert_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant(
+        ContainerTreeOfTheCapacityInvariants::Retirement,
+        "known-bad-retirement-containers-numbered-one-and-three",
+    );
+}
+
+#[test]
+fn tombstone_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant() {
+    assert_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant(
+        ContainerTreeOfTheCapacityInvariants::Tombstone,
+        "known-bad-tombstone-containers-numbered-two-and-three",
+    );
+}
+
+#[test]
+fn lineage_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant() {
+    assert_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant(
+        ContainerTreeOfTheCapacityInvariants::Lineage,
+        "known-bad-lineage-containers-numbered-two-and-three",
+    );
+}
+
+#[test]
+fn retirement_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant() {
+    assert_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant(
+        ContainerTreeOfTheCapacityInvariants::Retirement,
+        "known-bad-retirement-containers-numbered-two-and-three",
+    );
+}
+
+#[test]
+fn tombstone_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant() {
+    assert_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant(
+        ContainerTreeOfTheCapacityInvariants::Tombstone,
+        "known-bad-tombstone-containers-with-an-empty-rightmost-one",
+    );
+}
+
+#[test]
+fn lineage_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant() {
+    assert_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant(
+        ContainerTreeOfTheCapacityInvariants::Lineage,
+        "known-bad-lineage-containers-with-an-empty-rightmost-one",
+    );
+}
+
+#[test]
+fn retirement_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant() {
+    assert_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant(
+        ContainerTreeOfTheCapacityInvariants::Retirement,
+        "known-bad-retirement-containers-with-an-empty-rightmost-one",
+    );
+}
+
+/// G3 ①：谱系树两片同号（墓碑树那一形是 `two_tombstone_containers_with_one_container_number_redden_only_the_capacity_knob_invariant`）。
+#[test]
+fn lineage_containers_numbered_one_and_one_redden_only_the_knob_capacity_invariant() {
+    assert_containers_numbered_one_and_one_redden_only_the_knob_capacity_invariant(
+        ContainerTreeOfTheCapacityInvariants::Lineage,
+        "known-bad-lineage-containers-numbered-one-and-one",
+    );
+}
+
+/// G3 ①：退役树两片同号。
+#[test]
+fn retirement_containers_numbered_one_and_one_redden_only_the_knob_capacity_invariant() {
+    assert_containers_numbered_one_and_one_redden_only_the_knob_capacity_invariant(
+        ContainerTreeOfTheCapacityInvariants::Retirement,
+        "known-bad-retirement-containers-numbered-one-and-one",
+    );
+}
+
+/// G3 ③：退役容器压到一片 2 条写出两条退役记录（一片 2 条），带旋钮值的入口报退役旋钮 1：那片装了 2 条、多于旋钮值，只红 I-9.37；
+/// 报写的时候那个旋钮值 2 一条不红。
+#[test]
+fn a_retirement_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant(
+) {
+    let tree = ContainerTreeOfTheCapacityInvariants::Retirement;
+    let pool = pool_with_records_in_one_container_tree_under_its_knob(
+        tree,
+        "known-bad-retirement-container-past-the-knob",
+        2,
+        2,
+    );
+    assert_eq!(
+        tree.record_counts_of_each_container(&pool.output),
+        [2],
+        "退役树一片容器 2 条"
+    );
+    let image = pool.memory_pool();
+    let verdicts_given_the_written_knob =
+        singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
+            &image,
+            tree.checker_knobs(2),
+        );
+    assert_eq!(
+        violated_invariants(&verdicts_given_the_written_knob),
+        Vec::<&str>::new(),
+        "报写的时候那个旋钮值 2：一条不红：{verdicts_given_the_written_knob:?}"
+    );
+    assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
+        &image,
+        tree,
+        "装了 2 条，多于旋钮值 1",
+    );
+}
+
+/// G3 ②：产品容量下退役 960 片叶（每次建一个号开新叶、再删到底）：退役树 [959, 1]，同时 960 条墓碑把墓碑树写成 [815, 145]。
+/// 产品入口一条不红，I-9.36（退役树：第一片装满 959、最右那片不要求满）与 I-9.21 真被评估过且成立——959 与谱系的 1359 对调、
+/// 最右那片也要求满，这里就红。
+#[test]
+#[ignore = "harness 慢档用例：产品容量下退役 960 次（约 2900 次发布）"]
+fn nine_hundred_sixty_retirements_at_product_capacity_fill_every_retirement_container_but_the_rightmost_and_redden_nothing(
+) {
+    const RETIREMENTS_THAT_OPEN_A_SECOND_CONTAINER: u64 = 960;
+    let tree = ContainerTreeOfTheCapacityInvariants::Retirement;
+    let leaf_records = singlefs_format::INODE_LEAF_RECORDS;
+    let mut pool = common_deletion_in_progress::pool_with_new_inodes(
+        "known-good-retirement-containers-at-product-capacity",
+        leaf_records - 1,
+    );
+    // 迭代上界：常量 960。每一轮建一个号（最左那片叶满了，开新叶），再删到底：那片叶删空、退役一条，墓碑一条。
+    for _ in 0..RETIREMENTS_THAT_OPEN_A_SECOND_CONTAINER {
+        common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+        let inode = pool.output.inode_number_watermark() - 1;
+        unlink_an_empty_inode_to_the_end_under_the_capacities(
+            &mut pool,
+            inode,
+            singlefs_core::container_tree::ContainerTreeCapacities::FROM_THE_FORMAT,
+        );
+    }
+    assert_eq!(
+        tree.record_counts_of_each_container(&pool.output),
+        [959, 1],
+        "退役树第一片装满 959、第二片 1 条"
+    );
+    assert_eq!(
+        ContainerTreeOfTheCapacityInvariants::Tombstone
+            .record_counts_of_each_container(&pool.output),
+        [815, 145],
+        "墓碑树第一片装满 815、第二片 145 条"
+    );
+    let verdicts = check_pool_image(&pool.memory_pool());
+    assert_eq!(
+        violated_invariants(&verdicts),
+        Vec::<&str>::new(),
+        "产品容量的多容器合法镜像：一条不红：{verdicts:?}"
+    );
+    for invariant in [
+        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+        NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+    ] {
+        assert_eq!(
+            verdict_among(&verdicts, invariant),
+            InvariantVerdict::Holds,
+            "{invariant} 真被评估过且成立"
+        );
+    }
+}
+
+/// G3 ②：产品容量下挂着回退 1360 次（每次退到紧前那一版）：谱系树 [1359, 1]。产品入口一条不红，I-9.36（谱系树：第一片装满 1359、
+/// 最右那片不要求满）真被评估过且成立——1359 与退役的 959 对调、最右那片也要求满，这里就红。
+#[test]
+#[ignore = "harness 慢档用例：产品容量下回退 1360 次"]
+fn one_thousand_three_hundred_sixty_rollbacks_at_product_capacity_fill_every_lineage_container_but_the_rightmost_and_redden_nothing(
+) {
+    const ROLLBACKS_THAT_OPEN_A_SECOND_CONTAINER: u64 = 1360;
+    let tree = ContainerTreeOfTheCapacityInvariants::Lineage;
+    let mut pool = build_pool("known-good-lineage-containers-at-product-capacity");
+    let mut previous = pool.output.clone();
+    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+    // 迭代上界：常量 1360。每一次退到紧前那一版（两版轮流），一次一条谱系记录。
+    for _ in 0..ROLLBACKS_THAT_OPEN_A_SECOND_CONTAINER {
+        let target = previous;
+        previous = pool.output.clone();
+        common_deletion_in_progress::roll_back_to(&mut pool, &target);
+    }
+    assert_eq!(
+        tree.record_counts_of_each_container(&pool.output),
+        [1359, 1],
+        "谱系树第一片装满 1359、第二片 1 条"
+    );
+    let verdicts = check_pool_image(&pool.memory_pool());
+    assert_eq!(
+        violated_invariants(&verdicts),
+        Vec::<&str>::new(),
+        "产品容量的多容器合法镜像：一条不红：{verdicts:?}"
+    );
+    assert_eq!(
+        verdict_among(
+            &verdicts,
+            NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE
+        ),
+        InvariantVerdict::Holds,
+        "I-9.36 真被评估过且成立"
+    );
+}
+
+/// G3 ④ 判不了那一句的理由（`walk.rs` 的 `DeletionScanGap::reason`）里点得出是哪一形的那一段。
+const NOT_JUDGED_LINEAGE_TREE_UNREADABLE: &str = "谱系树读不全";
+const NOT_JUDGED_RETIREMENT_TREE_UNREADABLE: &str = "退役树读不全";
+const NOT_JUDGED_TREE_TABLE_UNREADABLE: &str = "树表读不出";
+
+/// 带旋钮值的入口上 I-9.37 报的不适用理由（判出违例或成立都不是这一形）。
+fn reason_the_knob_capacity_invariant_is_not_judged(
+    image: &MemoryPool,
+    tree: ContainerTreeOfTheCapacityInvariants,
+) -> &'static str {
+    let verdicts = singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
+        image,
+        tree.checker_knobs(1),
+    );
+    match verdict_among(&verdicts, tree.knob_capacity_invariant()) {
+        InvariantVerdict::NotApplicable(reason) => reason,
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::PartiallyJudged(_)) => panic!(
+            "{tree:?}：{} 要报射程里判不了（不适用带理由），实际 {other:?}",
+            tree.knob_capacity_invariant()
+        ),
+    }
+}
+
+/// G3 ④ 谱系树读不全：旋钮下回退两次（谱系 [1, 1]），第二片谱系容器两份全零（不补引用链）。回退一次那一版的谱系树读得全、照判；
+/// 最新那一版读不全，I-9.37 报射程里判不了、不报成立。
+#[test]
+fn an_unreadable_lineage_container_leaves_the_knob_capacity_invariant_not_judged() {
+    let tree = ContainerTreeOfTheCapacityInvariants::Lineage;
+    let mut image = pool_with_records_in_one_container_tree_under_its_knob(
+        tree,
+        "known-bad-lineage-container-unreadable",
+        1,
+        2,
+    )
+    .memory_pool();
+    let second_container_slot = container_slots_of_the_newest_tree(&image, tree)[1];
+    common_deletion_in_progress::zero_both_copies_of_the_unit_at(
+        &mut image,
+        second_container_slot,
+        common_pointer_head_corruption::data_unit_bytes(),
+    );
+    let reason = reason_the_knob_capacity_invariant_is_not_judged(&image, tree);
+    assert!(
+        reason.contains(NOT_JUDGED_LINEAGE_TREE_UNREADABLE),
+        "谱系树读不全的那一版 I-9.37 判不了：{reason}"
+    );
+}
+
+/// G3 ④ 退役树读不全：旋钮下退役两次（退役 [1, 1]），第二片退役容器两份全零。I-9.37 报射程里判不了。
+#[test]
+fn an_unreadable_retirement_container_leaves_the_knob_capacity_invariant_not_judged() {
+    let tree = ContainerTreeOfTheCapacityInvariants::Retirement;
+    let mut image = pool_with_records_in_one_container_tree_under_its_knob(
+        tree,
+        "known-bad-retirement-container-unreadable",
+        1,
+        2,
+    )
+    .memory_pool();
+    let second_container_slot = container_slots_of_the_newest_tree(&image, tree)[1];
+    common_deletion_in_progress::zero_both_copies_of_the_unit_at(
+        &mut image,
+        second_container_slot,
+        common_pointer_head_corruption::data_unit_bytes(),
+    );
+    let reason = reason_the_knob_capacity_invariant_is_not_judged(&image, tree);
+    assert!(
+        reason.contains(NOT_JUDGED_RETIREMENT_TREE_UNREADABLE),
+        "退役树读不全的那一版 I-9.37 判不了：{reason}"
+    );
+}
+
+/// G3 ④ 整版树表读不出：旋钮下回退一次（谱系 [1]），再建一个号；回退那一版的树表两份全零（不补引用链）。那一版谱系与退役都判不了，
+/// I-9.37 报射程里判不了；最新那一版照判。
+#[test]
+fn an_unreadable_tree_table_leaves_the_knob_capacity_invariant_not_judged() {
+    let tree = ContainerTreeOfTheCapacityInvariants::Lineage;
+    let mut pool = pool_with_records_in_one_container_tree_under_its_knob(
+        tree,
+        "known-bad-tree-table-unreadable-under-the-lineage-knob",
+        1,
+        1,
+    );
+    let rolled_back_once_tree_table_slot = pool.output.unit(TransactionUnit::TreeTable).slot;
+    let rolled_back_once_txg = pool.output.root.checkpoint_txg;
+    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+    let mut image = pool.memory_pool();
+    assert_ne!(
+        common_deletion_in_progress::tree_table_slot_of_the_newest_root(&image),
+        rolled_back_once_tree_table_slot,
+        "最新那一版（txg {} 之后又建了一个号）的树表另写一份",
+        rolled_back_once_txg.0
+    );
+    common_deletion_in_progress::zero_both_copies_of_the_unit_at(
+        &mut image,
+        rolled_back_once_tree_table_slot,
+        common_pointer_head_corruption::node_bytes(),
+    );
+    let reason = reason_the_knob_capacity_invariant_is_not_judged(&image, tree);
+    assert!(
+        reason.contains(NOT_JUDGED_TREE_TABLE_UNREADABLE),
+        "树表读不出的那一版 I-9.37 判不了：{reason}"
+    );
+}
+
+/// 旧的墓碑旋钮入口（`check_pool_image_given_the_tombstone_container_capacity_knob`）只给墓碑旋钮：谱系树与退役树当没装旋钮。
+/// 这一棵在旋钮下写成两片各 1 条，交给旧的墓碑旋钮入口（墓碑旋钮给产品容量 815：退役那份历史 unlink 两次写的两条墓碑同在一片，旋钮给得再小
+/// I-9.22 会跟着红）：这一棵按产品容量判，只红 I-9.36、说明点名这一棵；I-9.37 报不适用。
+fn assert_the_old_tombstone_knob_entry_judges_the_tree_by_its_product_capacity(
+    tree: ContainerTreeOfTheCapacityInvariants,
+    tag: &str,
+) {
+    let image =
+        pool_with_records_in_one_container_tree_under_its_knob(tree, tag, 1, 2).memory_pool();
+    let verdicts =
+        singlefs_checker::walk::check_pool_image_given_the_tombstone_container_capacity_knob(
+            &image,
+            Some(
+                singlefs_checker::walk::TombstoneContainerCapacityKnob::new(
+                    singlefs_format::TOMBSTONE_CONTAINER_RECORDS,
+                )
+                .expect("1..=815"),
+            ),
+        );
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE],
+        "{tree:?}：旧的墓碑旋钮入口按产品容量判这一棵，只红 I-9.36：{verdicts:?}"
+    );
+    assert!(
+        matches!(
+            verdict_among(
+                &verdicts,
+                NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE
+            ),
+            InvariantVerdict::Violated(detail) if detail.contains(tree.name_in_the_violation())
+        ),
+        "{tree:?}：I-9.36 的说明点名这一棵"
+    );
+    assert!(
+        matches!(
+            verdict_among(
+                &verdicts,
+                LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE
+            ),
+            InvariantVerdict::NotApplicable(_)
+        ),
+        "{tree:?}：I-9.37 报不适用"
+    );
+}
+
+/// G3 ⑤：旧的墓碑旋钮入口给谱系那一棵 `None`。
+#[test]
+fn the_old_tombstone_knob_entry_judges_the_lineage_tree_by_its_product_capacity() {
+    assert_the_old_tombstone_knob_entry_judges_the_tree_by_its_product_capacity(
+        ContainerTreeOfTheCapacityInvariants::Lineage,
+        "known-bad-old-tombstone-knob-entry-lineage",
+    );
+}
+
+/// G3 ⑤：旧的墓碑旋钮入口给退役那一棵 `None`。
+#[test]
+fn the_old_tombstone_knob_entry_judges_the_retirement_tree_by_its_product_capacity() {
+    assert_the_old_tombstone_knob_entry_judges_the_tree_by_its_product_capacity(
+        ContainerTreeOfTheCapacityInvariants::Retirement,
+        "known-bad-old-tombstone-knob-entry-retirement",
+    );
+}
diff --git a/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs b/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
index 637785e..3a4fbfa 100644
--- a/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
+++ b/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
@@ -4,6 +4,7 @@
 //! 池级 checker 从镜像外拿墓碑容器的旋钮值（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T5 取 K1）：
 //! 这些镜像走带旋钮值的入口，I-9.22（测试镜像里墓碑容器不超旋钮值） 按旋钮值判；同一张镜像交给旧入口（当没装旋钮）I-9.21 红。
 //! 谱系容器同样压到一片 1 条，带旋钮值的入口一并给谱系的旋钮值（I-9.37，第 53 批；不给的话回退两次写出的两片谱系容器按产品容量判 I-9.36 红）。
+//! 第 56 批（G2）：三棵容器树都压到一片 1 条的一段固定种子随机历史（建号、unlink、回退），每一步交带旋钮值的入口一条不红（慢档）。
 
 mod common;
 mod common_deletion_in_progress;
@@ -512,3 +513,201 @@ fn a_tombstone_container_holding_more_records_than_the_knob_reddens_only_the_kno
         violations_given_the_knob(&pool, 1)
     );
 }
+
+/// 这一段随机历史的种子与步数（固定：同一个种子逐位复现同一段历史；种子 1–8 在实现员副本上各跑一遍，取三棵树都写出至少三片容器、
+/// 退役树也长高了的 7）。
+const RANDOM_HISTORY_SEED_UNDER_EVERY_CONTAINER_KNOB: u64 = 7;
+const RANDOM_HISTORY_OPERATIONS_UNDER_EVERY_CONTAINER_KNOB: u64 = 60;
+/// 回退的目标从最近这么多版里挑：每步至多三次发布，挑到的那一版留在根环一圈里（回退候选集）。
+const ROLLBACK_TARGETS_KEPT: usize = 8;
+
+/// 三棵容器树都压到一片 1 条、一个码 2 节点 2 条（第 3 片容器根满长一层）。
+fn capacities_of_one_record_in_every_container_tree_and_two_entries() -> ContainerTreeCapacities {
+    ContainerTreeCapacities {
+        tombstone_container: TombstoneContainerCapacity::CappedForTests(
+            TombstoneRecordsPerContainerCappedForTests::new(1).expect("1..=815"),
+        ),
+        lineage_container: LineageContainerCapacity::CappedForTests(
+            LineageRecordsPerContainerCappedForTests::new(1).expect("1..=1359"),
+        ),
+        retirement_container:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
+                singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(1)
+                    .expect("1..=959"),
+            ),
+        node_entries: ContainerTreeNodeEntries::CappedForTests(
+            ContainerTreeNodeEntriesCappedForTests::new(2).expect("2..=135"),
+        ),
+    }
+}
+
+/// 同一组旋钮值交给池级 checker 的新入口（三棵都是一片 1 条）。
+fn knobs_of_one_record_in_every_container_tree() -> ContainerCapacityKnobs {
+    ContainerCapacityKnobs {
+        tombstone: Some(tombstone_container_capacity_knob(1)),
+        lineage: Some(LineageContainerCapacityKnob::new(1).expect("旋钮值在 1..=1359 里")),
+        retirement: Some(
+            singlefs_checker::walk::RetirementContainerCapacityKnob::new(1)
+                .expect("旋钮值在 1..=959 里"),
+        ),
+    }
+}
+
+/// 合法镜像不红（第 56 批 G2 验收，判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第三节；攻方
+/// `opus_probe_k2_random_knob_histories` 那一形）：三棵容器树都压到一片 1 条、一个节点 2 条，建满一片叶再多一个号（最右那片叶独占它），
+/// 之后固定种子随机建一个号、unlink（多半删最大的活号：最右那片叶删空就退役；也从记录最少的叶里删、随便删一个）、挂着回退到最近几版之一；
+/// 每一步之后带同一组旋钮值交池级 checker 的新入口，一条都不许红。历史走完三棵树各至少三片容器（容器号从 1 起逐片加一、每片至少一条那三判
+/// 真判到了多片），I-9.22、I-9.37 真被评估过且成立。
+#[test]
+#[ignore = "harness 慢档用例：60 步随机历史、每步一次池级 checker，debug 下约两分钟"]
+fn a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step() {
+    let mut random = singlefs_harness::history::SeededRandomSource::from_seed(
+        RANDOM_HISTORY_SEED_UNDER_EVERY_CONTAINER_KNOB,
+    );
+    let capacities = capacities_of_one_record_in_every_container_tree_and_two_entries();
+    let mut pool = pool_with_new_inodes(
+        "container-trees-random-history-under-every-knob",
+        singlefs_format::INODE_LEAF_RECORDS,
+    );
+    pool.allocator.set_deletion_switches(DeletionSwitches {
+        batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
+        tombstone_container_capacity: capacities.tombstone_container,
+        lineage_container_capacity: capacities.lineage_container,
+        retirement_container_capacity: capacities.retirement_container,
+        container_tree_node_entries: capacities.node_entries,
+    });
+    let mut recent_versions: Vec<TransactionOutput> = vec![pool.output.clone()];
+    let mut knob_invariants_held: Vec<&str> = Vec::new();
+    // 迭代上界：常量 60 步。跨步携带的状态：池与最近几版（至多 ROLLBACK_TARGETS_KEPT 版，最旧的先丢）。
+    for step in 0..RANDOM_HISTORY_OPERATIONS_UNDER_EVERY_CONTAINER_KNOB {
+        let choice = random.below(100);
+        let live_inodes: Vec<u64> = pool
+            .output
+            .inode_leaf_containers
+            .iter()
+            .flat_map(|leaf| leaf.contents.records.iter().map(|record| record.inode))
+            .filter(|inode| *inode != singlefs_core::transaction::FIRST_INODE_NUMBER)
+            .collect();
+        let rollback_targets: Vec<&TransactionOutput> = recent_versions
+            .iter()
+            .filter(|version| version.root.checkpoint_txg != pool.output.root.checkpoint_txg)
+            .collect();
+        if choice < 20 || live_inodes.is_empty() {
+            common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+        } else if choice < 85 || rollback_targets.is_empty() {
+            let inode = if choice < 25 {
+                let fewest_records = pool
+                    .output
+                    .inode_leaf_containers
+                    .iter()
+                    .filter(|leaf| {
+                        leaf.contents.records.iter().any(|record| {
+                            record.inode != singlefs_core::transaction::FIRST_INODE_NUMBER
+                        })
+                    })
+                    .min_by_key(|leaf| leaf.contents.records.len())
+                    .expect("有活着的号就有这样一片叶");
+                let candidates: Vec<u64> = fewest_records
+                    .contents
+                    .records
+                    .iter()
+                    .map(|record| record.inode)
+                    .filter(|inode| *inode != singlefs_core::transaction::FIRST_INODE_NUMBER)
+                    .collect();
+                candidates[usize::try_from(
+                    random.below(u64::try_from(candidates.len()).expect("装得进 u64")),
+                )
+                .expect("装得进 usize")]
+            } else if choice < 75 {
+                *live_inodes.iter().max().expect("非空")
+            } else {
+                live_inodes[usize::try_from(
+                    random.below(u64::try_from(live_inodes.len()).expect("装得进 u64")),
+                )
+                .expect("装得进 usize")]
+            };
+            unlink(&mut pool, inode, capacities);
+        } else {
+            let target = rollback_targets[usize::try_from(
+                random.below(u64::try_from(rollback_targets.len()).expect("装得进 u64")),
+            )
+            .expect("装得进 usize")]
+            .clone();
+            let publish_parameters = parameters();
+            let devices = pool.devices.as_mut().expect("镜像还开着");
+            roll_back_by_a_forward_publish(
+                &publish_parameters,
+                devices,
+                &mut pool.allocator,
+                &mut pool.output,
+                RollbackTarget {
+                    instance: target.root.instance,
+                    checkpoint_txg: target.root.checkpoint_txg,
+                },
+            )
+            .unwrap_or_else(|refusal| {
+                panic!(
+                    "第 {step} 步回退到 txg {}：{refusal:?}",
+                    target.root.checkpoint_txg.0
+                )
+            });
+        }
+        recent_versions.push(pool.output.clone());
+        if recent_versions.len() > ROLLBACK_TARGETS_KEPT {
+            recent_versions.remove(0);
+        }
+        let verdicts = check_pool_image_given_the_container_capacity_knobs(
+            &pool.memory_pool(),
+            knobs_of_one_record_in_every_container_tree(),
+        );
+        let violated: Vec<String> = verdicts
+            .iter()
+            .filter_map(|(invariant, verdict)| match verdict {
+                InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
+                InvariantVerdict::Holds
+                | InvariantVerdict::NotApplicable(_)
+                | InvariantVerdict::PartiallyJudged(_) => None,
+            })
+            .collect();
+        assert_eq!(
+            violated,
+            Vec::<String>::new(),
+            "第 {step} 步之后的合法镜像（带同一组旋钮值）一条都不红"
+        );
+        knob_invariants_held.extend(
+            verdicts
+                .iter()
+                .filter(|(_, verdict)| *verdict == InvariantVerdict::Holds)
+                .map(|(invariant, _)| *invariant)
+                .filter(|invariant| {
+                    *invariant == TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE
+                        || *invariant
+                            == singlefs_checker::image::LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE
+                }),
+        );
+    }
+    let container_counts = (
+        pool.output.tombstone_containers.len(),
+        pool.output.lineage_containers.len(),
+        pool.output.retirement_containers.len(),
+    );
+    eprintln!(
+        "CONTAINER_TREES_RANDOM_HISTORY seed={RANDOM_HISTORY_SEED_UNDER_EVERY_CONTAINER_KNOB} containers(tombstone, lineage, retirement)={container_counts:?} levels(tombstone, lineage, retirement)=({}, {}, {})",
+        pool.output.tombstone_tree_index_nodes.shape.root_level(),
+        pool.output.lineage_tree_index_nodes.shape.root_level(),
+        pool.output.retirement_tree_index_nodes.shape.root_level()
+    );
+    assert!(
+        container_counts.0 >= 3 && container_counts.1 >= 3 && container_counts.2 >= 3,
+        "历史走完三棵树各至少三片容器：{container_counts:?}"
+    );
+    for invariant in [
+        TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
+        singlefs_checker::image::LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
+    ] {
+        assert!(
+            knob_invariants_held.contains(&invariant),
+            "{invariant} 真被评估过且成立"
+        );
+    }
+}
`````

### mutations-append.tsv

`````text
第56批 G2 I-9.37：谱系与退役容器沿叶序第一片不判容器号是 1（攻方 mut-w-start2）	crates/singlefs-checker/src/walk.rs	            first_container_in_leaf_order == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,	            true,	-p singlefs-harness --test checker_known_bad_images -- containers_numbered_two_and_three	lineage_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant
第56批 G2 I-9.22：墓碑容器沿叶序第一片不判容器号是 1（攻方 mut-w-start2）	crates/singlefs-checker/src/walk.rs	            first_in_leaf_order.container == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,	            true,	-p singlefs-harness --test checker_known_bad_images -- containers_numbered_two_and_three	tombstone_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant
第56批 G2 I-9.37：谱系与退役容器相邻两片只判严格递增、不判加一（攻方 mut-w-gap2）	crates/singlefs-checker/src/walk.rs	            left_container.checked_add(1) == Some(right_container);	            left_container < right_container;	-p singlefs-harness --test checker_known_bad_images -- containers_numbered_one_and_three	retirement_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant
第56批 G2 I-9.22：墓碑容器相邻两片只判严格递增、不判加一（攻方 mut-w-gap2）	crates/singlefs-checker/src/walk.rs	            left.container.checked_add(1) == Some(right.container);	            left.container < right.container;	-p singlefs-harness --test checker_known_bad_images -- containers_numbered_one_and_three	tombstone_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant
第56批 G3 I-9.37：谱系与退役容器相邻两片的容器号整句恒真（攻方 c-incr-true）	crates/singlefs-checker/src/walk.rs	        judgements.judge(invariant, right_container_follows_the_left_by_one, || {	        judgements.judge(invariant, true, || {	-p singlefs-harness --test checker_known_bad_images -- containers_numbered_one_and_one	lineage_containers_numbered_one_and_one_redden_only_the_knob_capacity_invariant
第56批 G3 I-9.37：谱系与退役容器相邻两片的容器号整句恒真（正推 w1）	crates/singlefs-checker/src/walk.rs	        judgements.judge(invariant, right_container_follows_the_left_by_one, || {	        judgements.judge(invariant, true || right_container_follows_the_left_by_one, || {	-p singlefs-harness --test checker_known_bad_images -- containers_numbered_one_and_one	retirement_containers_numbered_one_and_one_redden_only_the_knob_capacity_invariant
第56批 G2 I-9.37：谱系与退役容器不判每片至少一条（攻方 mut-w-eager）	crates/singlefs-checker/src/walk.rs	        let lineage_or_retirement_container_holds_a_record = record_count > 0;	        let lineage_or_retirement_container_holds_a_record = true;	-p singlefs-harness --test checker_known_bad_images -- containers_with_an_empty_rightmost_one	lineage_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant
第56批 G2 I-9.22：墓碑容器不判每片至少一条（攻方 mut-w-eager）	crates/singlefs-checker/src/walk.rs	        let tombstone_container_holds_a_record = record_count > 0;	        let tombstone_container_holds_a_record = true;	-p singlefs-harness --test checker_known_bad_images -- containers_with_an_empty_rightmost_one	tombstone_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant
第56批 G3 I-9.36：谱系树产品容量写成退役的 959（正推 w3）	crates/singlefs-checker/src/walk.rs	            LineageOrRetirementTree::Lineage => LINEAGE_CONTAINER_RECORDS,	            LineageOrRetirementTree::Lineage => RETIREMENT_CONTAINER_RECORDS,	-p singlefs-harness --release --test checker_known_bad_images -- --include-ignored one_thousand_three_hundred_sixty_rollbacks_at_product_capacity_fill_every_lineage_container_but_the_rightmost_and_redden_nothing	one_thousand_three_hundred_sixty_rollbacks_at_product_capacity_fill_every_lineage_container_but_the_rightmost_and_redden_nothing
第56批 G3 I-9.36：退役树产品容量写成谱系的 1359（正推 w3b）	crates/singlefs-checker/src/walk.rs	            LineageOrRetirementTree::Retirement => RETIREMENT_CONTAINER_RECORDS,	            LineageOrRetirementTree::Retirement => LINEAGE_CONTAINER_RECORDS,	-p singlefs-harness --release --test checker_known_bad_images -- --include-ignored nine_hundred_sixty_retirements_at_product_capacity_fill_every_retirement_container_but_the_rightmost_and_redden_nothing	nine_hundred_sixty_retirements_at_product_capacity_fill_every_retirement_container_but_the_rightmost_and_redden_nothing
第56批 G3 I-9.36：谱系与退役树最右那片也要求装满（正推 w7）	crates/singlefs-checker/src/walk.rs	                if is_the_rightmost {\n                    continue;\n                }\n                let product_records = tree_kind	                if false && is_the_rightmost {\n                    continue;\n                }\n                let product_records = tree_kind	-p singlefs-harness --release --test checker_known_bad_images -- --include-ignored one_thousand_three_hundred_sixty_rollbacks_at_product_capacity_fill_every_lineage_container_but_the_rightmost_and_redden_nothing	one_thousand_three_hundred_sixty_rollbacks_at_product_capacity_fill_every_lineage_container_but_the_rightmost_and_redden_nothing
第56批 G3 I-9.37：谱系树读不全时不报射程里判不了（正推 w4）	crates/singlefs-checker/src/walk.rs	        Err(gap) => judgements.part_of_the_range_not_judged(\n            lineage_or_retirement_capacity_invariant_of(lineage_capacity_knob),\n            gap.reason(),\n        ),	        Err(_gap) => {}	-p singlefs-harness --test checker_known_bad_images -- an_unreadable_lineage_container_leaves_the_knob_capacity_invariant_not_judged	an_unreadable_lineage_container_leaves_the_knob_capacity_invariant_not_judged
第56批 G3 I-9.37：退役树读不全时不报射程里判不了（正推 w5）	crates/singlefs-checker/src/walk.rs	                for invariant in RETIREMENT_SCAN_INVARIANTS.into_iter().chain([\n                    lineage_or_retirement_capacity_invariant_of(retirement_capacity_knob),\n                ]) {	                for invariant in RETIREMENT_SCAN_INVARIANTS {	-p singlefs-harness --test checker_known_bad_images -- an_unreadable_retirement_container_leaves_the_knob_capacity_invariant_not_judged	an_unreadable_retirement_container_leaves_the_knob_capacity_invariant_not_judged
第56批 G3 I-9.36 / I-9.37：整版树表读不出时不报射程里判不了（正推 w6，树表读不出那一支）	crates/singlefs-checker/src/walk.rs	                .chain(lineage_and_retirement_capacity_invariants)\n            {\n                judgements.part_of_the_range_not_judged(invariant, gap.reason());	            {\n                judgements.part_of_the_range_not_judged(invariant, gap.reason());	-p singlefs-harness --test checker_known_bad_images -- an_unreadable_tree_table_leaves_the_knob_capacity_invariant_not_judged	an_unreadable_tree_table_leaves_the_knob_capacity_invariant_not_judged
第56批 G3 旧的墓碑旋钮入口给谱系旋钮（正推 w10）	crates/singlefs-checker/src/walk.rs	            lineage: None,\n            retirement: None,\n        },\n    )\n}\n\n/// 池级 checker 带三棵容器树旋钮值的入口	            lineage: LineageContainerCapacityKnob::new(1),\n            retirement: None,\n        },\n    )\n}\n\n/// 池级 checker 带三棵容器树旋钮值的入口	-p singlefs-harness --test checker_known_bad_images -- the_old_tombstone_knob_entry_judges_the_lineage_tree_by_its_product_capacity	the_old_tombstone_knob_entry_judges_the_lineage_tree_by_its_product_capacity
第56批 G3 旧的墓碑旋钮入口给退役旋钮	crates/singlefs-checker/src/walk.rs	            lineage: None,\n            retirement: None,\n        },\n    )\n}\n\n/// 池级 checker 带三棵容器树旋钮值的入口	            lineage: None,\n            retirement: RetirementContainerCapacityKnob::new(1),\n        },\n    )\n}\n\n/// 池级 checker 带三棵容器树旋钮值的入口	-p singlefs-harness --test checker_known_bad_images -- the_old_tombstone_knob_entry_judges_the_retirement_tree_by_its_product_capacity	the_old_tombstone_knob_entry_judges_the_retirement_tree_by_its_product_capacity
第56批 G3 I-9.37：退役树旋钮值加一（正推 w12）	crates/singlefs-checker/src/walk.rs	                .collect::<Vec<(u64, usize)>>(),\n            retirement_capacity_knob,	                .collect::<Vec<(u64, usize)>>(),\n            retirement_capacity_knob.map(|knob| knob + 1),	-p singlefs-harness --test checker_known_bad_images -- a_retirement_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant	a_retirement_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant
第56批 G2 I-9.37：谱系与退役容器相邻两片要求加二（合法的随机历史上误报）	crates/singlefs-checker/src/walk.rs	            left_container.checked_add(1) == Some(right_container);	            left_container.checked_add(2) == Some(right_container);	-p singlefs-harness --release --test container_trees_grow_a_level_when_the_root_is_full -- --include-ignored a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step	a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step
第56批 G2 GPU 判器：第一片容器号不判是 1（攻方 mut-w-start2）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    if (!eq64(d_container_number(0u), u64_of(D_FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE))) {	    if (false && !eq64(d_container_number(0u), u64_of(D_FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE))) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_lineage_containers_start_from_two	the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_lineage_containers_start_from_two
第56批 G2 GPU 判器：相邻两片只判严格递增、不判加一（攻方 mut-w-gap2）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (!(lt64(left_container, right_container) && eq64(add64(left_container, u64_of(1u)), right_container))) {	        if (!lt64(left_container, right_container)) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_lineage_containers_skip_a_number	the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_lineage_containers_skip_a_number
第56批 G3 GPU 判器：相邻两片的容器号整句不判（攻方 c-incr-true，两片同号）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (!(lt64(left_container, right_container) && eq64(add64(left_container, u64_of(1u)), right_container))) {	        if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_retirement_knob_agrees_with_the_cpu_when_two_retirement_containers_share_a_number	the_gpu_judge_given_the_retirement_knob_agrees_with_the_cpu_when_two_retirement_containers_share_a_number
第56批 G2 GPU 判器：不判每片至少一条（攻方 mut-w-eager）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (d_container_record_count(container_index) == 0u) {	        if (false && d_container_record_count(container_index) == 0u) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_the_rightmost_lineage_container_is_empty	the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_the_rightmost_lineage_container_is_empty
第56批 G6 GPU 判器：旋钮那一支按旋钮值加一判	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (d_container_record_count(container_index) > knob_records) {	        if (d_container_record_count(container_index) > knob_records + 1u) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_a_lineage_container_holds_more_records_than_the_knob	the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_a_lineage_container_holds_more_records_than_the_knob
第56批 G6 GPU 判器：谱系树不接旋钮值、照产品容量判	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    d_judge_lineage_or_retirement_container_capacity(DELETION_LINEAGE_CONTAINER_RECORDS, D_LINEAGE_CONTAINER_CAPACITY_KNOB);	    d_judge_lineage_or_retirement_container_capacity(DELETION_LINEAGE_CONTAINER_RECORDS, 0u);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_on_two_lineage_containers_of_one_record	the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_on_two_lineage_containers_of_one_record
第56批 G6 GPU 判器：退役树不接旋钮值、照产品容量判	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	                d_judge_lineage_or_retirement_container_capacity(DELETION_RETIREMENT_CONTAINER_RECORDS, D_RETIREMENT_CONTAINER_CAPACITY_KNOB);	                d_judge_lineage_or_retirement_container_capacity(DELETION_RETIREMENT_CONTAINER_RECORDS, 0u);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_retirement_knob_agrees_with_the_cpu_on_two_retirement_containers_of_one_record	the_gpu_judge_given_the_retirement_knob_agrees_with_the_cpu_on_two_retirement_containers_of_one_record
第56批 G6 GPU 判器：墓碑树不接旋钮值、照产品容量判	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    if (D_TOMBSTONE_CONTAINER_CAPACITY_KNOB != 0u) {	    if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_tombstone_knob_agrees_with_the_cpu_on_two_tombstone_containers_of_one_record	the_gpu_judge_given_the_tombstone_knob_agrees_with_the_cpu_on_two_tombstone_containers_of_one_record
第56批 G6 GPU 判器：谱系旋钮常量不照旋钮值生成（恒写 0）	crates/singlefs-checker-tier/src/crash_judge_gpu.rs	        knob_records_in_the_kernel(knobs.lineage.map(|knob| knob.records_per_container())),	        knob_records_in_the_kernel(knobs.lineage.map(|_knob| 0).filter(|_| false)),	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_two_lineage_containers_share_a_number	the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_two_lineage_containers_share_a_number
`````

### mutations-replacements.tsv

`````text
第24批 K M5：墓碑容器号沿叶序严格递增那一句恒真	crates/singlefs-checker/src/walk.rs	            right_tombstone_container_follows_the_left_by_one,	            true || right_tombstone_container_follows_the_left_by_one,	-p singlefs-harness --test checker_known_bad_images -- two_tombstone_containers_with_one_container_number_redden_only_the_capacity_knob_invariant	two_tombstone_containers_with_one_container_number_redden_only_the_capacity_knob_invariant
第53批 F2 GPU 判器 I-9.36：谱系树读完容器不判容量	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        return false;\n    }\n    d_judge_lineage_or_retirement_container_capacity(DELETION_LINEAGE_CONTAINER_RECORDS, D_LINEAGE_CONTAINER_CAPACITY_KNOB);	        return false;\n    }	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_lineage_container_closes_before_it_is_full	the_gpu_judge_agrees_with_the_cpu_when_a_lineage_container_closes_before_it_is_full
`````

## m3-batch56/m3-reference

### crates.patch

`````diff
diff --git a/crates/singlefs-core/src/inode_tree.rs b/crates/singlefs-core/src/inode_tree.rs
--- a/crates/singlefs-core/src/inode_tree.rs
+++ b/crates/singlefs-core/src/inode_tree.rs
@@ -1289,4 +1289,293 @@
             })
         );
     }
+
+    /// 等价性留档要比的两种改写（`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」第二行：在所有输入上与原式同值的变异，
+    /// 把等价性写成测试留档）。判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第三节 G4：两条腿都推等价，
+    /// 前提是「相邻两片之和 ≥ 234」——E176 M2、M3 的断言；追加只在最右那片满了才新开一片，删一条之后能并的一对之和恰好 233、
+    /// 并完那片满 233 条，删空的那一片删之前只有 1 条、它的两个邻居都满着。
+    #[derive(Clone, Copy, Debug)]
+    enum MergeRuleRewriting {
+        /// 正推腿 m3、攻方 `k4-no-chain`：从某片起的每一趟并掉一对就停，不从幸存那片接着查（借完那一趟也是）。
+        EveryPassStopsAfterAbsorbingOnePair,
+        /// 正推腿 m4：删空的那一片摘掉之后不从它的左邻起并。
+        NothingIsAbsorbedAfterDroppingTheEmptiedContainer,
+    }
+
+    /// 改写之下从 `start_position` 那片起的一趟并：交回并没并过。
+    fn absorb_pass_under_the_rewriting(
+        tree: &mut LeafContainersBeingMerged,
+        start_position: usize,
+        rewriting: MergeRuleRewriting,
+    ) -> bool {
+        match rewriting {
+            MergeRuleRewriting::EveryPassStopsAfterAbsorbingOnePair => {
+                if start_position + 1 < tree.containers.len()
+                    && tree.pair_fits_in_one_container(start_position)
+                {
+                    tree.absorb_the_right_neighbour(start_position);
+                    true
+                } else if start_position > 0 && tree.pair_fits_in_one_container(start_position - 1)
+                {
+                    tree.absorb_the_right_neighbour(start_position - 1);
+                    true
+                } else {
+                    false
+                }
+            }
+            MergeRuleRewriting::NothingIsAbsorbedAfterDroppingTheEmptiedContainer => {
+                tree.absorb_until_no_pair_fits(start_position)
+            }
+        }
+    }
+
+    /// 改写之下的左从右借：借的条数、挪哪几条、右片的分隔 key 与实现相同，借完那一趟并照改写走。
+    fn borrow_under_the_rewriting(
+        tree: &mut LeafContainersBeingMerged,
+        borrower_position: usize,
+        rewriting: MergeRuleRewriting,
+    ) {
+        match rewriting {
+            MergeRuleRewriting::EveryPassStopsAfterAbsorbingOnePair => {
+                let lender_position = borrower_position + 1;
+                if lender_position >= tree.containers.len()
+                    || tree.records_at(borrower_position)
+                        >= records_below_which_a_leaf_borrows_from_its_right_neighbour()
+                {
+                    return;
+                }
+                let moved_count =
+                    (tree.records_at(lender_position) - tree.records_at(borrower_position)) / 2;
+                let moved_records: Vec<InodeRecord> = tree.containers[lender_position]
+                    .contents
+                    .records
+                    .drain(..moved_count)
+                    .collect();
+                let smallest_kept_by_the_lender =
+                    tree.containers[lender_position].contents.records[0].inode;
+                tree.containers[lender_position]
+                    .contents
+                    .separator_key_in_the_parent_entry = smallest_kept_by_the_lender;
+                tree.containers[lender_position].is_changed_this_publish = true;
+                tree.containers[borrower_position]
+                    .contents
+                    .records
+                    .extend(moved_records);
+                tree.containers[borrower_position].is_changed_this_publish = true;
+                absorb_pass_under_the_rewriting(tree, lender_position, rewriting);
+            }
+            MergeRuleRewriting::NothingIsAbsorbedAfterDroppingTheEmptiedContainer => {
+                tree.borrow_from_the_right_neighbour_if_below_the_floor(borrower_position);
+            }
+        }
+    }
+
+    /// 改写之下删一条：删法、删空摘片、先并后借的次序与 [`remove_record_from_leaf_containers`] 相同，只有改写点到的那一处换掉。
+    fn remove_record_under_the_rewriting(
+        containers_before_this_publish: &[InodeLeafContainer],
+        inode: InodeNumber,
+        rewriting: MergeRuleRewriting,
+    ) -> InodeLeafContainersAfterThisPublish {
+        let holding_position =
+            position_of_the_container_holding(containers_before_this_publish, inode)
+                .expect("族里每一次删的号都取自那棵树")
+                .position();
+        let mut tree =
+            LeafContainersBeingMerged::from_the_previous_version(containers_before_this_publish);
+        tree.containers[holding_position]
+            .contents
+            .records
+            .retain(|kept| kept.inode != inode.0);
+        tree.containers[holding_position].is_changed_this_publish = true;
+        if tree.records_at(holding_position) == 0 {
+            tree.retire(holding_position);
+            let has_new_neighbours_on_both_sides =
+                holding_position > 0 && holding_position < tree.containers.len();
+            match rewriting {
+                MergeRuleRewriting::EveryPassStopsAfterAbsorbingOnePair => {
+                    if has_new_neighbours_on_both_sides {
+                        absorb_pass_under_the_rewriting(&mut tree, holding_position - 1, rewriting);
+                    }
+                }
+                MergeRuleRewriting::NothingIsAbsorbedAfterDroppingTheEmptiedContainer => {}
+            }
+        } else if !absorb_pass_under_the_rewriting(&mut tree, holding_position, rewriting) {
+            borrow_under_the_rewriting(&mut tree, holding_position, rewriting);
+        }
+        tree.after_this_publish()
+    }
+
+    /// 等价性那一族每片记录数的取值：1（删空）、2（借后吸收里的第三片）、116 / 117 / 118（删一条可并、借、借 1 条）、233（满片）。
+    const RECORD_COUNTS_OF_THE_EQUIVALENCE_FAMILY: [usize; 6] = [1, 2, 116, 117, 118, 233];
+    /// 那一族的树至多这么多片。
+    const CONTAINERS_IN_THE_EQUIVALENCE_FAMILY_AT_MOST: usize = 4;
+
+    /// 1 到 4 片、每片记录数取自 [`RECORD_COUNTS_OF_THE_EQUIVALENCE_FAMILY`]、相邻两片之和都 ≥ 234 的每一组记录数。
+    fn record_counts_where_every_adjacent_pair_overflows_one_container() -> Vec<Vec<usize>> {
+        let capacity = usize::try_from(INODE_LEAF_RECORDS).expect("233");
+        let mut of_the_current_length: Vec<Vec<usize>> = vec![Vec::new()];
+        let mut family = Vec::new();
+        // 迭代上界：CONTAINERS_IN_THE_EQUIVALENCE_FAMILY_AT_MOST 轮，每轮多一片。
+        for _ in 0..CONTAINERS_IN_THE_EQUIVALENCE_FAMILY_AT_MOST {
+            of_the_current_length = of_the_current_length
+                .iter()
+                .flat_map(|prefix| {
+                    RECORD_COUNTS_OF_THE_EQUIVALENCE_FAMILY
+                        .iter()
+                        .filter(|count| prefix.last().is_none_or(|last| last + **count > capacity))
+                        .map(|count| {
+                            let mut longer = prefix.clone();
+                            longer.push(*count);
+                            longer
+                        })
+                })
+                .collect();
+            family.extend(of_the_current_length.iter().cloned());
+        }
+        family
+    }
+
+    /// 记录数照 `record_counts` 的一棵树：inode 号从 1 起连号，每片容器号与分隔 key = 它的第一条（分裂纪律写出来的形状）。
+    fn tree_of_record_counts(record_counts: &[usize]) -> Vec<InodeLeafContainer> {
+        let mut first_inode_of_the_next_container = 1_u64;
+        record_counts
+            .iter()
+            .map(|count| {
+                let first_inode = first_inode_of_the_next_container;
+                first_inode_of_the_next_container += u64::try_from(*count).expect("至多 233");
+                InodeLeafContainer {
+                    identity: PackedIdentity {
+                        birth_tree: INODE_TREE,
+                        record_type: PACKED_TYPE_INODE,
+                        container: first_inode,
+                        container_birth: TREE_BIRTH_TXG,
+                    },
+                    separator_key_in_the_parent_entry: first_inode,
+                    records: (first_inode..first_inode_of_the_next_container)
+                        .map(|inode| record(inode, 0, 3))
+                        .collect(),
+                }
+            })
+            .collect()
+    }
+
+    /// 每片删第一条与最后一条（只有一条的那片删一次）：删哪一片决定落法，删片里哪一条只改挪走哪几个号。
+    fn inodes_removed_from_each_container(containers: &[InodeLeafContainer]) -> Vec<InodeNumber> {
+        let mut removed = Vec::new();
+        for container in containers {
+            let smallest = container.smallest_inode_number().expect("族里每片至少一条");
+            let largest = container.largest_inode_number().expect("族里每片至少一条");
+            removed.push(smallest);
+            if largest != smallest {
+                removed.push(largest);
+            }
+        }
+        removed
+    }
+
+    /// 这次删除里有片的分隔 key 变了（只有借会改：右片改成它这一版的最小 key）。
+    fn some_separator_key_moved(
+        before: &[InodeLeafContainer],
+        after: &InodeLeafContainersAfterThisPublish,
+    ) -> bool {
+        after.containers.iter().any(|kept| {
+            before.iter().any(|earlier| {
+                earlier.identity == kept.identity
+                    && earlier.separator_key_in_the_parent_entry
+                        != kept.separator_key_in_the_parent_entry
+            })
+        })
+    }
+
+    /// 正推腿 m3、攻方 `k4-no-chain`（连锁只并一轮）的等价性留档：相邻两片之和都 ≥ 234 的每一棵树上删每一片的头尾一条，
+    /// 实现与「每一趟并掉一对就停」逐片相同（容器、重写清单、来处、退役，整份比）——能并的一对之和恰好 233、并完那片满着，
+    /// 再与哪个邻居相加都 > 233；借之后借方与借出片之和不变、仍 > 233，借出片吸收右邻之后同理。
+    /// 族里删一条并、借后吸收、删空摘片后从左邻起那一趟各至少走到一次；删完之后相邻两片之和仍 ≥ 234（前提对删除封闭）。
+    #[test]
+    fn stopping_every_absorb_pass_after_one_pair_changes_nothing_when_every_adjacent_pair_overflows_one_container(
+    ) {
+        let rewriting = MergeRuleRewriting::EveryPassStopsAfterAbsorbingOnePair;
+        let mut removals_that_absorbed_without_borrowing = 0;
+        let mut removals_that_borrowed_and_then_absorbed = 0;
+        let mut removals_that_dropped_a_container_between_two_neighbours = 0;
+        for record_counts in record_counts_where_every_adjacent_pair_overflows_one_container() {
+            let before = tree_of_record_counts(&record_counts);
+            for inode in inodes_removed_from_each_container(&before) {
+                let holding_position = position_of_the_container_holding(&before, inode)
+                    .expect("取自这棵树")
+                    .position();
+                let implementation = remove_record_from_leaf_containers(&before, inode)
+                    .expect("族里每一次删的号都在树里");
+                assert_eq!(
+                    implementation,
+                    remove_record_under_the_rewriting(&before, inode, rewriting),
+                    "各片 {record_counts:?} 删 {inode:?}：实现与「每一趟并掉一对就停」不同"
+                );
+                for pair in implementation.containers.windows(2) {
+                    assert!(
+                        pair[0].records.len() + pair[1].records.len() >= 234,
+                        "各片 {record_counts:?} 删 {inode:?}：删完之后相邻两片之和 < 234（前提不再成立）"
+                    );
+                }
+                let emptied = record_counts[holding_position] == 1;
+                let borrowed = some_separator_key_moved(&before, &implementation);
+                if emptied && holding_position > 0 && holding_position + 1 < record_counts.len() {
+                    removals_that_dropped_a_container_between_two_neighbours += 1;
+                }
+                if !emptied && !implementation.retired.is_empty() {
+                    if borrowed {
+                        removals_that_borrowed_and_then_absorbed += 1;
+                    } else {
+                        removals_that_absorbed_without_borrowing += 1;
+                    }
+                }
+            }
+        }
+        assert!(
+            removals_that_absorbed_without_borrowing >= 1,
+            "族里有删一条就并的"
+        );
+        assert!(
+            removals_that_borrowed_and_then_absorbed >= 1,
+            "族里有借完借出的那一片接着并的"
+        );
+        assert!(
+            removals_that_dropped_a_container_between_two_neighbours >= 1,
+            "族里有删空摘片之后两边还有邻居、从左邻起看一趟的"
+        );
+    }
+
+    /// 正推腿 m4（删空摘片后不再并）的等价性留档：相邻两片之和都 ≥ 234 的每一棵树上删每一片的头尾一条，实现与「删空的那一片摘掉之后
+    /// 不从它的左邻起并」逐片相同（整份比）——删空的那一片删之前只有 1 条，它的两个邻居都满 233 条，摘掉之后那一对之和 466，
+    /// 左邻再与它的左邻之和也 > 233。族里删空摘片之后两边还有邻居的至少走到一次。
+    #[test]
+    fn not_absorbing_after_dropping_an_emptied_container_changes_nothing_when_every_adjacent_pair_overflows_one_container(
+    ) {
+        let rewriting = MergeRuleRewriting::NothingIsAbsorbedAfterDroppingTheEmptiedContainer;
+        let mut removals_that_dropped_a_container_between_two_neighbours = 0;
+        for record_counts in record_counts_where_every_adjacent_pair_overflows_one_container() {
+            let before = tree_of_record_counts(&record_counts);
+            for inode in inodes_removed_from_each_container(&before) {
+                let holding_position = position_of_the_container_holding(&before, inode)
+                    .expect("取自这棵树")
+                    .position();
+                assert_eq!(
+                    remove_record_from_leaf_containers(&before, inode)
+                        .expect("族里每一次删的号都在树里"),
+                    remove_record_under_the_rewriting(&before, inode, rewriting),
+                    "各片 {record_counts:?} 删 {inode:?}：实现与「删空摘片之后不再并」不同"
+                );
+                if record_counts[holding_position] == 1
+                    && holding_position > 0
+                    && holding_position + 1 < record_counts.len()
+                {
+                    removals_that_dropped_a_container_between_two_neighbours += 1;
+                }
+            }
+        }
+        assert!(
+            removals_that_dropped_a_container_between_two_neighbours >= 1,
+            "族里有删空摘片之后两边还有邻居的"
+        );
+    }
 }
diff --git a/crates/singlefs-harness/tests/inode_leaf_merge_and_borrow_against_an_independent_reference.rs b/crates/singlefs-harness/tests/inode_leaf_merge_and_borrow_against_an_independent_reference.rs
--- a/crates/singlefs-harness/tests/inode_leaf_merge_and_borrow_against_an_independent_reference.rs
+++ b/crates/singlefs-harness/tests/inode_leaf_merge_and_borrow_against_an_independent_reference.rs
@@ -1,13 +1,24 @@
-//! inode 树叶容器的合并与借在发布路径上逐次对一份独立参照（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F1）：
-//! 参照按条款另写一份，不引 `singlefs_core::inode_tree` 的合并、借、摘片代码，只从发布出来的那一版读记录与身份——
-//! D8（核心索引结构） 已定项 6：插入落在最右的叶、满 233 条就在末尾分裂，右半容器号 = 那条新记录的 inode 号、出生代 = 这次发布的 txg；
+//! inode 树叶容器的合并与借在发布路径上逐次对一份独立参照（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F1；
+//! 历史补段、比分隔 key、各形至少走到一次照 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第三节 G4）：
+//! 参照按条款另写一份，不引 `singlefs_core::inode_tree` 的合并、借、摘片代码，只从发布出来的那一版读记录、身份与分隔 key——
+//! D8（核心索引结构） 已定项 6：插入落在最右的叶、满 233 条就在末尾分裂，右半容器号 = 那条新记录的 inode 号、出生代 = 这次发布的 txg，
+//! 新片的分隔 key = 它的容器号，左从右借的那次发布右片的分隔 key 改成它这一版的最小 key；
 //! 合并时机照 E176（inode 树叶合并三候选的容量与写代价） M3 那一臂（`research/e7-index-bench/src/bin/e176_inode_leaf_merge_capacity_and_rewrites.rs`
 //! 的 `remove`）：删空的整片摘掉、身份消失，它的新左右邻从左邻起并；没删空就从它起并——先查它与右邻、再查左邻与它，两片之和 ≤ 233 就左吸收右、
 //! 被吸收的那一片身份消失，并完从幸存那片接着查；从它起一次都没并、它少于 ⌈233 ÷ 2⌉ 条、它有右邻时从右邻借最小的 ⌊(右 − 左) ÷ 2⌋ 条，
 //! 两片都保留身份，再从借出的那一片起并。
-//! 历史（种子固定）：随机切几批建 583–599 个空 inode（叶 [233, 233, ≥ 117]），再一次次 unlink「可删条数最多的那片叶」（并列取最左）里随机的一个号，
-//! 直到参照见到一次「删完之后与左邻、右邻两对都装得下」——M3 的方向只有这一形分得出来（攻方第二次抽样 K4-4：三片各 117 条删中间一条，
-//! 两种写法退役不同的叶、池级 checker 都不红）；再随机 unlink 几个活着的号。每次用户改动之后逐片比记录号与身份，逐次比这次退役的身份集与死亡 txg。
+//! 历史（种子固定）分五段：
+//! 一、随机切几批建 583–599 个空 inode（叶 [233, 233, ≥ 117]），再一次次 unlink「可删条数最多的那片叶」（并列取最左）里随机的一个号，
+//!    直到参照见到一次「删完之后与左邻、右邻两对都装得下」——M3 的方向只有这一形分得出来（攻方第二次抽样 K4-4：三片各 117 条删中间一条，
+//!    两种写法退役不同的叶、池级 checker 都不红）；先右之后剩 [117, 233]。
+//! 二、删空摘片：再建 1–8 个（最右那片满着，新开一片），一次次 unlink 最大的活号，直到最右那片删空摘掉（它的左邻满 233 条，删到最后一条之前
+//!    左邻那一对都装不下）；之后回到 [117, 233]。
+//! 三、借与借后吸收：再建 1–58 个（新开第三片），unlink 第一片里随机的一个号：第一片剩 116 条、从第二片借 ⌊(233 − 116) ÷ 2⌋ = 58 条，
+//!    第二片剩 175 条，与第三片之和 ≤ 233，借出的那一片接着吸收第三片。
+//! 四、再随机 unlink 几个活着的号。
+//! 五、左吸收：一次次 unlink 最大的活号，直到最右那片与左邻之和恰好 233（它没有右邻，只有左邻那一对装得下），左邻吸收它。
+//! 每次用户改动之后逐片比记录号、身份与分隔 key，逐次比这次退役的身份集与死亡 txg；末了断言两对都装得下（先右）、删空摘片、借、借后吸收、
+//! 左吸收这五形参照里各至少走到一次。
 
 mod common;
 mod common_deletion_in_progress;
@@ -33,8 +44,15 @@
 const NEW_INODES_PER_BATCH_AT_MOST: u64 = 260;
 /// 删的那一段至多这么多次 unlink：三片从 [233, 233, 133] 删到各 117 条至多 250 次（参照的推算），见到那一形之后另删几次。
 const UNLINKS_UNTIL_BOTH_PAIRS_FIT_AT_MOST: u64 = 260;
-/// 见到「两对都装得下」之后再随机 unlink 几个活着的号。
-const RANDOM_UNLINKS_AFTER_BOTH_PAIRS_FIT: u64 = 6;
+/// 第二段新开的那一片至多装这么多条（1 起随机）；一条一条删最大的活号，删这么多次它就删空。
+const RECORDS_OF_THE_LEAF_EMPTIED_BY_UNLINKING_THE_LARGEST_AT_MOST: u64 = 8;
+/// 第三段新开的第三片至多装这么多条（1 起随机）：第一片 116 条从第二片 233 条借 ⌊(233 − 116) ÷ 2⌋ = 58 条，第二片剩 175 条，
+/// 与第三片之和 ≤ 233 要第三片 ≤ 233 − 175 = 58。
+const RECORDS_OF_THE_LEAF_THE_LENDER_ABSORBS_AT_MOST: u64 = 58;
+/// 第四段随机 unlink 几个活着的号。
+const RANDOM_UNLINKS_BEFORE_ONLY_THE_LEFT_PAIR_FITS: u64 = 6;
+/// 第五段至多这么多次 unlink 最大的活号：最右那片至多 233 条，删到与左邻之和恰好 233 之前删不完它。
+const UNLINKS_UNTIL_ONLY_THE_LEFT_PAIR_FITS_AT_MOST: u64 = 233;
 
 /// 一片叶容器的身份四段（出生树、打包记录类型、容器号、容器出生代）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
@@ -45,19 +63,29 @@
     container_birth: u64,
 }
 
-/// 参照里的一片叶：身份与它装的 inode 号（升序）。
+/// 参照里的一片叶：身份、它在父节点条目里的分隔 key 与它装的 inode 号（升序）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 struct ReferenceLeaf {
     identity: ReferenceLeafIdentity,
+    separator_key: u64,
     inode_numbers: Vec<u64>,
 }
 
-/// 参照：按叶序的叶，加上这次改动里身份消失的那几片（按消失的先后）与「两对都装得下」见过几次。
+/// 参照：按叶序的叶，加上这次改动里身份消失的那几片（按消失的先后），与五形各走到几次。
 struct ReferenceLeafShape {
     inode_tree: u64,
     leaves: Vec<ReferenceLeaf>,
     retired_in_this_change: Vec<ReferenceLeafIdentity>,
+    /// 删完之后与左邻、右邻两对都装得下（M3 先右）。
     times_both_neighbour_pairs_fit: u64,
+    /// 右邻那一对装不下（或没有右邻）、左邻那一对装得下，左邻吸收它。
+    times_only_the_left_neighbour_pair_fit: u64,
+    /// 删空的那一片整片摘掉。
+    times_an_emptied_leaf_was_dropped: u64,
+    /// 少于下限的那一片从右邻借。
+    times_a_leaf_borrowed_from_its_right_neighbour: u64,
+    /// 借完从借出的那一片起并，并掉了至少一对。
+    times_the_lender_absorbed_after_lending: u64,
 }
 
 fn leaf_capacity() -> usize {
@@ -70,7 +98,7 @@
 }
 
 impl ReferenceLeafShape {
-    /// 从发布出来的那一版读起：每片叶的身份与记录号。
+    /// 从发布出来的那一版读起：每片叶的身份、分隔 key 与记录号。
     fn of_the_version(version: &TransactionOutput) -> Self {
         let leaves = leaves_of_the_version(version);
         let inode_tree = leaves
@@ -83,10 +111,14 @@
             leaves,
             retired_in_this_change: Vec::new(),
             times_both_neighbour_pairs_fit: 0,
+            times_only_the_left_neighbour_pair_fit: 0,
+            times_an_emptied_leaf_was_dropped: 0,
+            times_a_leaf_borrowed_from_its_right_neighbour: 0,
+            times_the_lender_absorbed_after_lending: 0,
         }
     }
 
-    /// 一条新号追加到最右那片；最右那片满 233 条（或树是空的）就新开一片：容器号 = 这个号、出生代 = 这次发布的 txg。
+    /// 一条新号追加到最右那片；最右那片满 233 条（或树是空的）就新开一片：容器号与分隔 key = 这个号、出生代 = 这次发布的 txg。
     fn append(&mut self, inode: u64, txg_of_the_publish: u64) {
         let rightmost_is_full = self
             .leaves
@@ -100,6 +132,7 @@
                     container: inode,
                     container_birth: txg_of_the_publish,
                 },
+                separator_key: inode,
                 inode_numbers: Vec::new(),
             });
         }
@@ -114,12 +147,20 @@
         self.leaves[position].inode_numbers.len()
     }
 
+    fn record_counts(&self) -> Vec<usize> {
+        self.leaves
+            .iter()
+            .map(|leaf| leaf.inode_numbers.len())
+            .collect()
+    }
+
     fn drop_leaf(&mut self, position: usize) -> ReferenceLeaf {
         let dropped = self.leaves.remove(position);
         self.retired_in_this_change.push(dropped.identity);
         dropped
     }
 
+    /// 左片吸收右邻：左片的身份与分隔 key 不变，右邻的记录接在后面。
     fn left_absorbs_its_right_neighbour(&mut self, left_position: usize) {
         let absorbed = self.drop_leaf(left_position + 1);
         self.leaves[left_position]
@@ -143,6 +184,7 @@
             if right_pair_fits {
                 self.left_absorbs_its_right_neighbour(position);
             } else if left_pair_fits {
+                self.times_only_the_left_neighbour_pair_fit += 1;
                 self.left_absorbs_its_right_neighbour(position - 1);
                 position -= 1;
             } else {
@@ -152,18 +194,27 @@
         }
     }
 
-    /// 均分借：`position` 那片少于下限、有右邻时，右邻最小的 ⌊(右 − 左) ÷ 2⌋ 条挪到它末尾，再从右邻起并。
+    /// 均分借：`position` 那片少于下限、有右邻时，右邻最小的 ⌊(右 − 左) ÷ 2⌋ 条挪到它末尾，右邻的分隔 key 改成它这一版的最小 key，
+    /// 再从右邻起并。
     fn borrow_from_the_right_if_below_the_floor(&mut self, position: usize) {
         if position + 1 >= self.leaves.len() || self.records_at(position) >= borrow_floor() {
             return;
         }
+        self.times_a_leaf_borrowed_from_its_right_neighbour += 1;
         let moved_count = (self.records_at(position + 1) - self.records_at(position)) / 2;
         let moved: Vec<u64> = self.leaves[position + 1]
             .inode_numbers
             .drain(..moved_count)
             .collect();
         self.leaves[position].inode_numbers.extend(moved);
-        self.merge_from(position + 1);
+        let lender = &mut self.leaves[position + 1];
+        lender.separator_key = *lender
+            .inode_numbers
+            .first()
+            .expect("借出的条数少于右邻的条数（两片之和 > 233、左片 < 117）：右邻借完还有记录");
+        if self.merge_from(position + 1) {
+            self.times_the_lender_absorbed_after_lending += 1;
+        }
     }
 
     /// unlink 最后一批删掉 `inode` 那条记录。
@@ -177,6 +228,7 @@
             .inode_numbers
             .retain(|kept| *kept != inode);
         if self.leaves[position].inode_numbers.is_empty() {
+            self.times_an_emptied_leaf_was_dropped += 1;
             self.drop_leaf(position);
             if position > 0 && position < self.leaves.len() {
                 self.merge_from(position - 1);
@@ -201,6 +253,24 @@
             .expect("取到的最大值来自其中一片")
     }
 
+    /// 第 `position` 片里随机一个可删的号（不是第一个文件那条）。
+    fn random_removable_inode_of_the_leaf(
+        &self,
+        position: usize,
+        random: &mut SeededRandomSource,
+    ) -> u64 {
+        let removable: Vec<u64> = self.leaves[position]
+            .inode_numbers
+            .iter()
+            .copied()
+            .filter(|inode| *inode != FIRST_FILE_INODE)
+            .collect();
+        let pick =
+            usize::try_from(random.below(u64::try_from(removable.len()).expect("一片至多 233 条")))
+                .expect("小于 233");
+        removable[pick]
+    }
+
     fn live_removable_inodes(&self) -> Vec<u64> {
         self.leaves
             .iter()
@@ -208,6 +278,20 @@
             .filter(|inode| *inode != FIRST_FILE_INODE)
             .collect()
     }
+
+    /// 最大的活号：最右那片的最后一条。
+    fn largest_live_inode(&self) -> u64 {
+        let largest = *self
+            .leaves
+            .last()
+            .and_then(|leaf| leaf.inode_numbers.last())
+            .expect("树里至少一条记录");
+        assert_ne!(
+            largest, FIRST_FILE_INODE,
+            "最大的活号是第一个文件那条：历史不删它"
+        );
+        largest
+    }
 }
 
 fn identity_of(identity: &singlefs_core::unit::PackedIdentity) -> ReferenceLeafIdentity {
@@ -219,13 +303,14 @@
     }
 }
 
-/// 发布出来的那一版里按叶序的每片叶：身份与记录号。
+/// 发布出来的那一版里按叶序的每片叶：身份、父节点条目里的分隔 key 与记录号。
 fn leaves_of_the_version(version: &TransactionOutput) -> Vec<ReferenceLeaf> {
     version
         .inode_leaf_containers
         .iter()
         .map(|container| ReferenceLeaf {
             identity: identity_of(&container.contents.identity),
+            separator_key: container.contents.separator_key_in_the_parent_entry,
             inode_numbers: container
                 .contents
                 .records
@@ -246,8 +331,8 @@
         .collect()
 }
 
-/// 一次用户改动之后：发布路径那一版的每片叶（身份、记录号）与参照逐片相同；这次改动追加的退役记录（退役树只追加）与参照这次身份消失的那几片
-/// 逐个相同（身份按集合比、条数也比），死亡 txg 都是这一版的 txg。
+/// 一次用户改动之后：发布路径那一版的每片叶（身份、分隔 key、记录号）与参照逐片相同；这次改动追加的退役记录（退役树只追加）与参照
+/// 这次身份消失的那几片逐个相同（身份按集合比、条数也比），死亡 txg 都是这一版的 txg。
 fn assert_the_version_matches_the_reference(
     what_changed: &str,
     retirement_records_before: &[(ReferenceLeafIdentity, u64)],
@@ -260,12 +345,20 @@
             .iter()
             .map(|leaf| leaf.inode_numbers.len())
             .collect::<Vec<_>>(),
+        reference.record_counts(),
+        "{what_changed}：每片叶的记录数与参照不同"
+    );
+    assert_eq!(
+        leaves
+            .iter()
+            .map(|leaf| leaf.separator_key)
+            .collect::<Vec<_>>(),
         reference
             .leaves
             .iter()
-            .map(|leaf| leaf.inode_numbers.len())
+            .map(|leaf| leaf.separator_key)
             .collect::<Vec<_>>(),
-        "{what_changed}：每片叶的记录数与参照不同"
+        "{what_changed}：每片叶的分隔 key 与参照不同"
     );
     assert_eq!(
         leaves, reference.leaves,
@@ -314,11 +407,7 @@
 /// unlink `inode` 到底，参照在最后一批删掉那条记录，再比最后一批那一版。
 fn unlink_and_compare(pool: &mut BuiltPool, inode: u64, reference: &mut ReferenceLeafShape) {
     let retirement_records_before = retirement_records_of(&pool.output);
-    let counts_before: Vec<usize> = reference
-        .leaves
-        .iter()
-        .map(|leaf| leaf.inode_numbers.len())
-        .collect();
+    let counts_before = reference.record_counts();
     unlink_to_the_end(pool, inode);
     reference.remove(inode);
     assert_the_version_matches_the_reference(
@@ -338,6 +427,8 @@
     let mut pool = build_pool("inode-leaf-merge-and-borrow-against-an-independent-reference");
     let mut random = SeededRandomSource::from_seed(SEED_OF_THE_HISTORY);
     let mut reference = ReferenceLeafShape::of_the_version(&pool.output);
+
+    // 一、建到 [233, 233, ≥ 117]，删可删条数最多的那片，直到两对都装得下（先右）。
     let mut left_to_create = NEW_INODES_AT_LEAST + random.below(NEW_INODES_ABOVE_THE_LEAST_BELOW);
     // 迭代上界：每一批至少建 1 个，left_to_create 至多 598。
     while left_to_create > 0 {
@@ -345,36 +436,122 @@
         create_and_compare(&mut pool, count, &mut reference);
         left_to_create -= count;
     }
-    let mut unlinks = 0;
+    let mut unlinks_until_both_pairs_fit = 0;
     // 迭代上界：UNLINKS_UNTIL_BOTH_PAIRS_FIT_AT_MOST 次；跨轮携带的是参照与池，每轮各删一个号。
     while reference.times_both_neighbour_pairs_fit == 0 {
         assert!(
-            unlinks < UNLINKS_UNTIL_BOTH_PAIRS_FIT_AT_MOST,
-            "删了 {unlinks} 次还没走到「两对都装得下」那一形：各片 {:?}",
-            reference
-                .leaves
-                .iter()
-                .map(|leaf| leaf.inode_numbers.len())
-                .collect::<Vec<_>>()
+            unlinks_until_both_pairs_fit < UNLINKS_UNTIL_BOTH_PAIRS_FIT_AT_MOST,
+            "删了 {unlinks_until_both_pairs_fit} 次还没走到「两对都装得下」那一形：各片 {:?}",
+            reference.record_counts()
         );
         let position = reference.position_of_the_leaf_with_the_most_removable_records();
-        let removable: Vec<u64> = reference.leaves[position]
-            .inode_numbers
-            .iter()
-            .copied()
-            .filter(|inode| *inode != FIRST_FILE_INODE)
-            .collect();
-        let pick =
-            usize::try_from(random.below(u64::try_from(removable.len()).expect("一片至多 233 条")))
-                .expect("小于 233");
-        unlink_and_compare(&mut pool, removable[pick], &mut reference);
-        unlinks += 1;
+        let inode = reference.random_removable_inode_of_the_leaf(position, &mut random);
+        unlink_and_compare(&mut pool, inode, &mut reference);
+        unlinks_until_both_pairs_fit += 1;
     }
-    // 迭代上界：常量 RANDOM_UNLINKS_AFTER_BOTH_PAIRS_FIT。
-    for _ in 0..RANDOM_UNLINKS_AFTER_BOTH_PAIRS_FIT {
+
+    // 二、最右那片后面新开一片 1–8 条，删最大的活号直到它删空摘掉。
+    let records_of_the_leaf_to_empty =
+        1 + random.below(RECORDS_OF_THE_LEAF_EMPTIED_BY_UNLINKING_THE_LARGEST_AT_MOST);
+    let free_slots_of_the_rightmost_leaf =
+        leaf_capacity() - reference.records_at(reference.leaves.len() - 1);
+    create_and_compare(
+        &mut pool,
+        u64::try_from(free_slots_of_the_rightmost_leaf).expect("至多 233")
+            + records_of_the_leaf_to_empty,
+        &mut reference,
+    );
+    let mut unlinks_of_the_largest_until_a_drop = 0;
+    // 迭代上界：records_of_the_leaf_to_empty 次；每轮删最右那片的最后一条。
+    while reference.times_an_emptied_leaf_was_dropped == 0 {
+        assert!(
+            unlinks_of_the_largest_until_a_drop < records_of_the_leaf_to_empty,
+            "删了 {unlinks_of_the_largest_until_a_drop} 次最大的活号，{records_of_the_leaf_to_empty} 条的那一片还没删空：各片 {:?}",
+            reference.record_counts()
+        );
+        let largest_live_inode = reference.largest_live_inode();
+        unlink_and_compare(&mut pool, largest_live_inode, &mut reference);
+        unlinks_of_the_largest_until_a_drop += 1;
+    }
+
+    // 三、[117, 233] 后面新开第三片 1–58 条，删第一片的一条：借 58 条，借出的那一片再吸收第三片。
+    assert_eq!(
+        reference.record_counts(),
+        vec![borrow_floor(), leaf_capacity()],
+        "第三段从 [117, 233] 起：先右吸收之后、新开的那一片删空摘掉之后"
+    );
+    let records_of_the_leaf_the_lender_absorbs =
+        1 + random.below(RECORDS_OF_THE_LEAF_THE_LENDER_ABSORBS_AT_MOST);
+    create_and_compare(
+        &mut pool,
+        records_of_the_leaf_the_lender_absorbs,
+        &mut reference,
+    );
+    let inode_unlinked_from_the_first_leaf =
+        reference.random_removable_inode_of_the_leaf(0, &mut random);
+    unlink_and_compare(
+        &mut pool,
+        inode_unlinked_from_the_first_leaf,
+        &mut reference,
+    );
+    assert_eq!(
+        (
+            reference.times_a_leaf_borrowed_from_its_right_neighbour,
+            reference.times_the_lender_absorbed_after_lending
+        ),
+        (1, 1),
+        "第一片 116 条从第二片借 58 条，第二片剩 175 条再吸收第三片 {records_of_the_leaf_the_lender_absorbs} 条：各片 {:?}",
+        reference.record_counts()
+    );
+
+    // 四、随机 unlink 几个活着的号。
+    // 迭代上界：常量 RANDOM_UNLINKS_BEFORE_ONLY_THE_LEFT_PAIR_FITS。
+    for _ in 0..RANDOM_UNLINKS_BEFORE_ONLY_THE_LEFT_PAIR_FITS {
         let live = reference.live_removable_inodes();
         let pick = usize::try_from(random.below(u64::try_from(live.len()).expect("装得进 u64")))
             .expect("小于活着的号数");
         unlink_and_compare(&mut pool, live[pick], &mut reference);
     }
+
+    // 五、删最大的活号，直到最右那片与左邻之和恰好 233、只有左邻那一对装得下。
+    let rightmost_position = reference.leaves.len() - 1;
+    assert!(
+        rightmost_position > 0 && reference.records_at(rightmost_position - 1) < leaf_capacity(),
+        "第五段要最右那片有一个没装满的左邻（左邻满着就删到最右那片删空、走不到左吸收）：各片 {:?}",
+        reference.record_counts()
+    );
+    let mut unlinks_of_the_largest_until_the_left_pair_fits = 0;
+    // 迭代上界：UNLINKS_UNTIL_ONLY_THE_LEFT_PAIR_FITS_AT_MOST 次；每轮删最右那片的最后一条。
+    while reference.times_only_the_left_neighbour_pair_fit == 0 {
+        assert!(
+            unlinks_of_the_largest_until_the_left_pair_fits
+                < UNLINKS_UNTIL_ONLY_THE_LEFT_PAIR_FITS_AT_MOST,
+            "删了 {unlinks_of_the_largest_until_the_left_pair_fits} 次最大的活号还没走到左吸收：各片 {:?}",
+            reference.record_counts()
+        );
+        let largest_live_inode = reference.largest_live_inode();
+        unlink_and_compare(&mut pool, largest_live_inode, &mut reference);
+        unlinks_of_the_largest_until_the_left_pair_fits += 1;
+    }
+
+    assert!(
+        reference.times_both_neighbour_pairs_fit >= 1,
+        "两对都装得下（先右）至少走到一次"
+    );
+    assert!(
+        reference.times_an_emptied_leaf_was_dropped >= 1,
+        "删空摘片至少走到一次"
+    );
+    assert!(
+        reference.times_a_leaf_borrowed_from_its_right_neighbour >= 1,
+        "借至少走到一次"
+    );
+    assert!(
+        reference.times_the_lender_absorbed_after_lending >= 1,
+        "借后吸收至少走到一次"
+    );
+    assert!(
+        reference.times_only_the_left_neighbour_pair_fit >= 1,
+        "左吸收（只有左邻那一对装得下）至少走到一次"
+    );
 }
`````

### mutations-append.tsv

`````text
第 56 批 G4 正推 m2：借完不从借出的那一片接着并（参照用例第三段借后吸收）	crates/singlefs-core/src/inode_tree.rs	        borrower.is_changed_this_publish = true;\n        self.absorb_until_no_pair_fits(lender_position);\n    }	        borrower.is_changed_this_publish = true;\n    }	-p singlefs-harness --test inode_leaf_merge_and_borrow_against_an_independent_reference -- every_publish_of_a_random_history_leaves_the_leaf_shape_and_the_retired_leaves_of_the_independent_reference	every_publish_of_a_random_history_leaves_the_leaf_shape_and_the_retired_leaves_of_the_independent_reference
第 56 批 G4 攻方 k4-no-absorb-after-borrow：删掉借完那一趟并（参照用例第三段借后吸收）	crates/singlefs-core/src/inode_tree.rs	        self.absorb_until_no_pair_fits(lender_position);\n		-p singlefs-harness --test inode_leaf_merge_and_borrow_against_an_independent_reference -- every_publish_of_a_random_history_leaves_the_leaf_shape_and_the_retired_leaves_of_the_independent_reference	every_publish_of_a_random_history_leaves_the_leaf_shape_and_the_retired_leaves_of_the_independent_reference
第 56 批 G4 正推 m5：左吸收整支关掉（参照用例第五段只有左邻那一对装得下）	crates/singlefs-core/src/inode_tree.rs	            let left_pair_fits =\n                current_position > 0 && self.pair_fits_in_one_container(current_position - 1);	            let left_pair_fits = false;	-p singlefs-harness --test inode_leaf_merge_and_borrow_against_an_independent_reference -- every_publish_of_a_random_history_leaves_the_leaf_shape_and_the_retired_leaves_of_the_independent_reference	every_publish_of_a_random_history_leaves_the_leaf_shape_and_the_retired_leaves_of_the_independent_reference
第 56 批 G4 正推 m9：借完不改右邻的分隔 key（参照用例比分隔 key）	crates/singlefs-core/src/inode_tree.rs	        lender.contents.separator_key_in_the_parent_entry = lender\n            .contents\n            .records\n            .first()\n            .map(|record| record.inode)\n            .expect("右邻借出的条数少于它的记录数：借完至少还剩一条");\n		-p singlefs-harness --test inode_leaf_merge_and_borrow_against_an_independent_reference -- every_publish_of_a_random_history_leaves_the_leaf_shape_and_the_retired_leaves_of_the_independent_reference	every_publish_of_a_random_history_leaves_the_leaf_shape_and_the_retired_leaves_of_the_independent_reference
第 56 批 G4 连锁只并一轮的等价留档会红：合并判据放宽一条（两片之和 ≤ 234 就并，并完满着的那片还能与只有 1 条的邻居再并一轮）	crates/singlefs-core/src/inode_tree.rs	            <= INODE_LEAF_RECORDS\n	            <= INODE_LEAF_RECORDS + 1\n	-p singlefs-core --lib -- inode_tree::tests	stopping_every_absorb_pass_after_one_pair_changes_nothing_when_every_adjacent_pair_overflows_one_container
第 56 批 G4 删空摘片后不再并的等价留档会红：合并判据放宽到两片之和 ≤ 466（删空摘片之后两边的满片也并）	crates/singlefs-core/src/inode_tree.rs	            <= INODE_LEAF_RECORDS\n	            <= 2 * INODE_LEAF_RECORDS\n	-p singlefs-core --lib -- inode_tree::tests	not_absorbing_after_dropping_an_emptied_container_changes_nothing_when_every_adjacent_pair_overflows_one_container
`````

