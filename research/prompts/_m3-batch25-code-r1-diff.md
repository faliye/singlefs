# 删除批第 25 批合入的改动原样（2026-10-06）

## 实现员 n：crates.patch（/tmp/claude-1000/m3-batch25/n/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index 307b354..8839db6 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -1571,7 +1571,13 @@ fn w_judge_packed_container(s: Slice, expected_type: u32, fsid_low: vec2<u32>) -
     return out;
 }
 
-// walk.rs walk_instance_table_chain（挂载根那一张另判行：I-3.8 的行那一半，行记进 C_INSTANCE_ROWS）
+// 主走读沿最新根的实例表链走没走到末片（链指针记录说「没有下一片」）：walk.rs 的 Walk::abandoned_roots_filtering。走到了，C_INSTANCE_ROWS
+// 就是整张表的行、候选集按它滤掉了被崩溃恢复抛弃的根；没走到就滤不全。经交接段交给扫描内核，I-9.29 跨版本那张表比不比读它（判决
+// `research/prompts/m3-batch23-code-r1-main-verification.md` 第三节第 1 条：门与候选集的滤法用同一遍走读，不另读一遍）。择根之后、走最新根之前清掉
+var<private> w_newest_instance_table_chain_reached_its_last_page: bool;
+
+// walk.rs walk_instance_table_chain（挂载根那一张另判行：I-3.8 的行那一半，行记进 C_INSTANCE_ROWS；走到末片记进
+// w_newest_instance_table_chain_reached_its_last_page）
 fn w_walk_instance_table_chain(first: Ptr, judge_rows: bool, fsid_low: vec2<u32>) {
     var pointer = first;
     var page_index = vec2<u32>(0u, 0u);
@@ -1657,6 +1663,9 @@ fn w_walk_instance_table_chain(first: Ptr, judge_rows: bool, fsid_low: vec2<u32>
             }
             chain_last = chain_last && chain_last_in_this_page;
             pages = pages + 1u;
+            if (chain == 0u) {
+                w_newest_instance_table_chain_reached_its_last_page = true;
+            }
         }
         if (chain != 1u) {
             break;
@@ -2661,12 +2670,12 @@ fn w_walk_inode_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: v
                 let blocks = rd64(p.base, ro + 48u);
                 let logical_blocks = shr64(add64_saturating(size, u64_of(511u)), 9u);
                 w_judge(INVARIANT_I_9_15, eq64(blocks, logical_blocks));
-                // I-9.26：对象出生代（记录偏移 8）≤ 改动计数（偏移 88）≤ 这片容器头的诞生代号（偏移 73）
+                // I-9.26：对象出生代（记录偏移 INODE_RECORD_OBJECT_BIRTH_OFFSET）≤ 改动计数（偏移 88）≤ 这片容器头的诞生代号（偏移 73）
                 let change_count = rd64(p.base, ro + 88u);
-                w_judge(INVARIANT_I_9_26, le64(rd64(p.base, ro + 8u), change_count) && le64(change_count, p.birth_txg));
+                w_judge(INVARIANT_I_9_26, le64(rd64(p.base, ro + INODE_RECORD_OBJECT_BIRTH_OFFSET), change_count) && le64(change_count, p.birth_txg));
                 if (w_context == 0u) {
                     // inode → 对象出生代（后写的覆盖先写的）
-                    let birth = rd64(p.base, ro + 8u);
+                    let birth = rd64(p.base, ro + INODE_RECORD_OBJECT_BIRTH_OFFSET);
                     var found = false;
                     for (var i = 0u; i < w_inode_birth_count; i = i + 1u) {
                         if (sw(C_INODE_BIRTHS + i * 4u) == inode.x && sw(C_INODE_BIRTHS + i * 4u + 1u) == inode.y) {
@@ -3678,6 +3687,7 @@ fn walk_and_hand_off(g: CheckerSlot, fsid_low: vec2<u32>) -> u32 {
     c_newest_index = newest;
     w_mount_instance = newest_root.instance;
     w_mount_txg = newest_root.txg;
+    w_newest_instance_table_chain_reached_its_last_page = false;
     w_start(0u, true);
     var phase = WALK_PHASE_NEWEST_ROOT;
     var cursor = 0u;
@@ -4014,6 +4024,7 @@ fn walk_and_hand_off(g: CheckerSlot, fsid_low: vec2<u32>) -> u32 {
                 sws(H_BASE + H_MOUNT_TXG_LO, w_mount_txg.x);
                 sws(H_BASE + H_MOUNT_TXG_HI, w_mount_txg.y);
                 sws(H_BASE + H_ROOT_COUNT, root_count);
+                sws(H_BASE + H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE, select(0u, 1u, w_newest_instance_table_chain_reached_its_last_page));
                 sws(H_BASE + H_UNIT_AREA_START_LO, g.unit_area_start_slot.x);
                 sws(H_BASE + H_UNIT_AREA_START_HI, g.unit_area_start_slot.y);
                 sws(H_BASE + H_REGIONS, g.regions);
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
index 27c91d4..49f9428 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
@@ -723,6 +723,9 @@ const H_FSID2: u32 = 26u;
 const H_FSID3: u32 = 27u;
 const H_BELOW_FLOOR_SLOT_COUNT: u32 = 28u;
 const H_SLOTS_PER_REGION: u32 = 29u;     // 根环每区槽数 S：I-7.9 的一圈窗口宽 3S = H_REGIONS × 它
+// 1 = 主走读沿最新根的实例表链走到了末片（走读内核的 w_newest_instance_table_chain_reached_its_last_page）：扫描内核 I-9.29 跨版本那张表比不比读它。
+// 第 30、31 两个字原来空着（H_BELOW_FLOOR_SLOTS 从 H_BASE + 32 起），占第 30 个，交接段不变长
+const H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE: u32 = 30u;
 const H_MAX_BELOW_FLOOR_SLOTS: u32 = 4096u;
 const H_BELOW_FLOOR_SLOTS: u32 = 235504u;   // 盘、槽 lo/hi
 const H_END: u32 = 247792u;
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
index 7d057e7..ada31db 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -2089,7 +2089,7 @@ fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
 }
 
 // ───────────────────────── 删除态扫描（walk.rs judge_deletion_state_of_the_versions）与 I-9.6 墓碑那一半 ─────────────────────────
-// I-9.5、I-3.4、I-4.7、I-4.6 在回退候选集里每条根与由记录施加出来的那几版上各重读一遍树表与几棵树（只读）。CPU 记成射程里判不了的那几格
+// I-9.5、I-3.4、I-4.7、I-4.6、I-9.24 在回退候选集里每条根与由记录施加出来的那几版上各重读一遍树表与几棵树（只读）。CPU 记成射程里判不了的那几格
 // （树表读不出、条目宽不到 200、deleted_inodes 的头 ID 配不上恰好一条 inode 树、队列根读不出或形状不对、inode 树 / extent 树 / 记账树读不全、
 // 头 ID 指着这棵 inode 树的 extent 树不是恰好一条）在这里一格都不置位；CPU 按树表单元去重只扫一次，违例位是并集，这里不去重。
 // I-4.4 第一版报不适用（check_pool_image 恒报），内核不置它那一位。
@@ -2502,7 +2502,7 @@ fn d_judge_deletion_state_of_a_version(base: u32, tree_table_pointer_offset: u32
         }
         d_judge_pending_deletion_occupancy_of_an_accounting_tree(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET);
     }
-    // I-9.5、I-4.7、I-4.6（walk.rs judge_deletion_queue_of_a_version）：每一条 deleted_inodes 条目
+    // I-9.24、I-9.5、I-4.7、I-4.6（walk.rs judge_deletion_queue_of_a_version）：每一条 deleted_inodes 条目
     for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
         let entry = entry_offset(tt, entry_index);
         if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_DELETED_INODES) {
@@ -2614,7 +2614,7 @@ fn judge_inode_number_watermark_above_every_published_tombstone() {
 
 // ───────────────────────── 墓碑与谱系那一遍（walk.rs judge_tombstones_and_the_lineage_of_the_versions）─────────────────────────
 // 删除态扫描那一份版本清单里每一版重读树表、谱系树、墓碑树、inode 树与 deleted_inodes（只读，不进 w_failures）：判 I-9.18、I-9.19、I-9.20、
-// I-9.21 与 I-9.10 墓碑那一个等号，把各版的 (inode 树, 号) → 对象出生代收进跨版本那张表（D_OBJECT_BIRTHS），同一个 (树, 号) 见到第二个出生代就判 I-9.29（最新根的实例表读不出时不比）。
+// I-9.21 与 I-9.10 墓碑那一个等号，把各版的 (inode 树, 号) → 对象出生代收进跨版本那张表（D_OBJECT_BIRTHS），同一个 (树, 号) 见到第二个出生代就判 I-9.29（主走读没沿最新根的实例表链走到末片时不比）。
 // CPU 记成射程里判不了的那几格（树表读不出、条目宽不到 200、谱系树条目不是恰好一条、谱系树或墓碑树读不全、谱系接不成一条链、头 ID 配不上恰好一棵
 // inode 树或恰好一棵墓碑树、队列根读不出、队列里的号没有记录、inode 树读不全）在这里一格都不置位；CPU 按 (树表单元, txg) 去重只扫一次，
 // 违例位是并集，这里不去重。I-9.21 只按没装旋钮那一支判（层 0 与放量走池级 checker 的旧入口），I-9.22 不置位。不按已发布与否滤墓碑（第二节 T4）。
@@ -2626,10 +2626,13 @@ var<private> d_lineage_record_count: u32;
 
 var<private> d_object_birth_count: u32;
 
-// I-9.29 跨版本那张表比不比：最新根的实例表沿链整张读得出（r_instance_rows，与 walk.rs 的 instance_table_rows_of_root_without_judging 同一个读法）
-// 才比；链上有一片读不出、解不开时回退候选集滤不掉被崩溃恢复抛弃的根，被抛弃的线上同号可以是另一个对象，CPU 报部分没判
+// I-9.29 跨版本那张表比不比：走读内核的主走读沿最新根的实例表链走到了末片（交接段 H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE，
+// 与候选集的滤法用同一遍走读，同 walk.rs 的 Walk::abandoned_roots_filtering）才比；没走到末片（一片读不出、解不开、身份不对、链指针记录不合法、
+// 链成了环）时回退候选集滤不掉被崩溃恢复抛弃的根，被抛弃的线上同号可以是另一个对象，CPU 报部分没判
 // （walk.rs 的 AbandonedRootsFilteringOfTheCandidateSet；`.claude/kb/invariants.md` I-9.29 那一行「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」），
-// 这里一个号都不记、不置位。不记也就不会撞上那张表的上限：CPU 那一支一个号都不比，这里没有判不了的理由
+// 这里一个号都不记、不置位。不记也就不会撞上那张表的上限：CPU 那一支一个号都不比，这里没有判不了的理由。
+// 不用 r_instance_rows 判：它拒收的比主走读多（链指针记录之前有不是行的记录、载荷 CRC 对不上），候选集已经滤全时会说没滤全
+// （判决 `research/prompts/m3-batch23-code-r1-main-verification.md` 第二节 K2 N5）
 var<private> d_object_births_are_compared: bool;
 
 // I-9.20 不重号那一句两两比：一棵墓碑树的墓碑多于这么多条就判不了（比较次数随条数平方涨）
@@ -2815,7 +2818,7 @@ fn d_txg_is_on_the_lineage(txg: vec2<u32>, version_txg: vec2<u32>) -> bool {
 }
 
 // I-9.29 跨版本那张表记一笔：(inode 树, 号) 第一次见到时记下对象出生代，再见到而出生代不同就判红（walk.rs 收齐之后每个号判一次
-// births.len() == 1，与这里见到第二个不同的出生代就红是同一判）。表满判不了。最新根的实例表读不出时整张表不比（d_object_births_are_compared）
+// births.len() == 1，与这里见到第二个不同的出生代就红是同一判）。表满判不了。主走读没沿最新根的实例表链走到末片时整张表不比（d_object_births_are_compared）
 fn d_note_object_birth(inode_tree: vec2<u32>, inode: vec2<u32>, object_birth: vec2<u32>) {
     if (!d_object_births_are_compared) {
         return;
@@ -3089,10 +3092,10 @@ fn d_judge_tombstones_and_the_lineage_of_a_version(base: u32, tree_table_pointer
 }
 
 // 墓碑与谱系那一遍：删除态扫描那一份版本清单（回退候选集里每条根，与由记录施加出来的那几版），每一版从自己的 txg 与自己的谱系树起算；
-// 全部扫完时 I-9.29 那张表已经判完（d_note_object_birth 见到第二个出生代当场判）；最新根的实例表读不出时那张表不比
+// 全部扫完时 I-9.29 那张表已经判完（d_note_object_birth 见到第二个出生代当场判）；主走读没沿最新根的实例表链走到末片时那张表不比
 fn judge_tombstones_and_the_lineage_of_the_versions() {
     d_object_birth_count = 0u;
-    d_object_births_are_compared = r_instance_rows(checker_root_of(c_newest_index).base);
+    d_object_births_are_compared = sw(H_BASE + H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE) == 1u;
     for (var candidate = 0u; candidate < c_candidate_count; candidate = candidate + 1u) {
         let root = checker_root_of(sw(C_CANDIDATES + candidate));
         d_judge_tombstones_and_the_lineage_of_a_version(root.base, 36u, root.txg);
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 2db3cc2..3a605cb 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -21,7 +21,8 @@
 //! 第 24 批：条目类型段与孩子的真实类不符的四形（攻方 N4-a、N4-i、N4-f、N4-j：层级 1 的墓碑树根条目类型段 0 指码 3 容器，身份三段带着或清零；
 //! 旋钮压出来的层级 2 墓碑树根条目类型段 1 指码 2 节点，根环转没转过一圈各一条）钉 CPU 红 I-9.23、不红走读失败连带的那几条，GPU 逐位跟上，
 //! inode 树根条目类型段 0 指码 3 叶容器（I-9.2 同形）一条同钉；
-//! I-9.29 判不了那一形（被崩溃恢复抛弃的根与现行线同号不同出生代、最新根的实例表两份翻坏）一条流整份比。
+//! I-9.29 判不了那一形（被崩溃恢复抛弃的根与现行线同号不同出生代、最新根的实例表两份翻坏）一条流整份比；
+//! 第 25 批：同一段历史、最新根的实例表链指针记录之前多一条 kind 2（攻方 N5-a，主走读走到末片、I-9.29 照比）一条流整份比。
 //! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
 //! extent 下段叶的数据指针、分配记录树内部条目子指针上各一条坏流整份比（core 读这几条指针都判、GPU 恢复内核照判）；这几处除实例表指针外
 //! 另各有一条 MAC 第 0 字节非 0 的坏流（core 连加密、压缩那几段一并判，GPU 恢复内核同判）；extent 上段叶内联的数据指针另有一条算法类型写 1 的坏流
@@ -6181,11 +6182,11 @@ const BYTES_FROM_THE_END_OF_THE_FLIPPED_PAYLOAD_BYTE: usize = 100;
 /// 同一段历史，录成流）：A (1, 3)、B (1, 4) → 重开取号 2（写行、暖机）→ C (2, 8)；崩溃恢复抛弃 C
 /// （`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：C 的根槽与数据单元暂时读不出、见证 C 的系统配置槽写回轮换之前那一代，
 /// 择根落到 C 之前那条根，实例 3 写行、暖机，再把暂存的字节原样写回）。录成流之后 C 那片 inode 叶容器两份里第一个文件那条记录的对象出生代加一、
-/// 重封到 C 的根（C 那一版与现行线上各版同号不同出生代），再把最新根（实例 3）指着的实例表两份各翻一个载荷字节、不重封。
-/// 实例 3 的根落了、C 写回了的状态上，候选集滤不掉被抛弃的 C：CPU 报 I-9.29 部分没判、不红，GPU 扫描内核那张表不比、不置位
-/// （照旧比的话 C 那条记录与现行线同号不同出生代，GPU 红 I-9.29、CPU 不红）。交回流，与翻坏实例表之前这条流整条落盘之后的镜像（候选集滤得掉 C 的对照）。
-fn record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_unreadable(
+/// 重封到 C 的根（C 那一版与现行线上各版同号不同出生代），再照 `change` 改最新根（实例 3）指着的实例表（[`ChangeOfTheNewestInstanceTable`]）。
+/// 交回流，与改实例表之前这条流整条落盘之后的镜像（候选集滤得掉 C 的对照）。
+fn record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
     tag: &str,
+    change: ChangeOfTheNewestInstanceTable,
 ) -> (RecordedFlow, MemoryPool) {
     let mut pool = build_pool(tag);
     let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
@@ -6269,7 +6270,7 @@ fn record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance
         .rposition(|write| write.kind == StepKind::RootRecordFua)
         .expect("根槽写");
     let mut flow = RecordedFlow {
-        name: "abandoned-root-naming-another-object-birth-newest-instance-table-unreadable",
+        name: change.flow_name(),
         base: pool.memory_pool_after_mkfs(),
         writes,
         segments,
@@ -6310,20 +6311,93 @@ fn record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance
         &is_an_instance_table_unit,
         false,
     );
-    for write_index in newest_instance_table {
+    for write_index in &newest_instance_table {
         assert!(
-            stream_indexes[write_index] >= operations_through_the_third,
+            stream_indexes[*write_index] >= operations_through_the_third,
             "最后写的那张实例表是实例 3 写行那次写的"
         );
-        let WrittenContents::Bytes(bytes) = &mut flow.writes[write_index].contents else {
-            panic!("单元写是普通写");
-        };
-        let flipped = bytes.len() - BYTES_FROM_THE_END_OF_THE_FLIPPED_PAYLOAD_BYTE;
-        bytes[flipped] ^= 0xff;
+    }
+    match change {
+        ChangeOfTheNewestInstanceTable::BothCopiesFlippedWithoutResealing => {
+            for write_index in newest_instance_table {
+                let WrittenContents::Bytes(bytes) = &mut flow.writes[write_index].contents else {
+                    panic!("单元写是普通写");
+                };
+                let flipped = bytes.len() - BYTES_FROM_THE_END_OF_THE_FLIPPED_PAYLOAD_BYTE;
+                bytes[flipped] ^= 0xff;
+            }
+        }
+        ChangeOfTheNewestInstanceTable::RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord => {
+            corrupt_units_and_reseal_up_to_the_root(
+                &mut flow,
+                &newest_instance_table,
+                &insert_a_record_of_an_unregistered_kind_before_the_chain_record,
+            );
+        }
     }
     (flow, image_with_the_newest_instance_table_readable)
 }
 
+/// 最新根（实例 3）指着的那张实例表在流里怎么改（两份副本一起改）。
+#[derive(Clone, Copy, Debug)]
+enum ChangeOfTheNewestInstanceTable {
+    /// 两份各翻一个载荷字节、不重封：位置条目里的整单元校验和对不上，两份都读不出（第 23 批 X 那一形，攻方 N5-d）。
+    /// 实例 3 的根落了、C 写回了的状态上，候选集滤不掉被抛弃的 C：CPU 报 I-9.29 部分没判、不红，GPU 扫描内核那张表不比、不置位
+    /// （照旧比的话 C 那条记录与现行线同号不同出生代，GPU 红 I-9.29、CPU 不红）。
+    BothCopiesFlippedWithoutResealing,
+    /// 链指针记录之前插一条没登记种类（kind 2）的记录、重封到根（攻方 N5-a，`research/prompts/m3-batch23-code-r1-opus-output.md`「一、N5 打中」）：
+    /// 主走读照收 kind 0 的行、沿链走到末片，候选集滤掉了被抛弃的 C，CPU 与 GPU 都比、都不红 I-9.29。
+    RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord,
+}
+
+impl ChangeOfTheNewestInstanceTable {
+    const fn flow_name(self) -> &'static str {
+        match self {
+            Self::BothCopiesFlippedWithoutResealing => {
+                "abandoned-root-naming-another-object-birth-newest-instance-table-unreadable"
+            }
+            Self::RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord => {
+                "abandoned-root-naming-another-object-birth-newest-instance-table-with-an-unregistered-record-kind"
+            }
+        }
+    }
+}
+
+/// 实例表一条记录的字节数（D18（块里携带什么信息） 已定项 11）；第 0 字节是种类：1 链指针记录，2 没登记。
+const INSTANCE_TABLE_RECORD_BYTES: usize = 88;
+const INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD: u8 = 1;
+const INSTANCE_TABLE_RECORD_KIND_UNREGISTERED: u8 = 2;
+
+/// 实例表一片里链指针记录（最后一条）往后挪一条，原位插一条全零、种类 2 的记录，记录数与声明长度各加一条（重封归调用方）。
+fn insert_a_record_of_an_unregistered_kind_before_the_chain_record(bytes: &mut [u8]) {
+    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
+    assert!(record_count >= 1, "实例表一片里至少有链指针记录");
+    let chain_record_start =
+        PACKED_UNIT_RECORDS_START + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
+    assert_eq!(
+        bytes[chain_record_start], INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD,
+        "实例表一片的最后一条是链指针记录"
+    );
+    let chain_record =
+        bytes[chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES].to_vec();
+    bytes[chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES].fill(0);
+    bytes[chain_record_start] = INSTANCE_TABLE_RECORD_KIND_UNREGISTERED;
+    bytes[chain_record_start + INSTANCE_TABLE_RECORD_BYTES
+        ..chain_record_start + 2 * INSTANCE_TABLE_RECORD_BYTES]
+        .copy_from_slice(&chain_record);
+    let new_record_count = record_count + 1;
+    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2].copy_from_slice(
+        &u16::try_from(new_record_count)
+            .expect("一片装得下")
+            .to_le_bytes(),
+    );
+    bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
+        &u16::try_from(new_record_count * INSTANCE_TABLE_RECORD_BYTES)
+            .expect("一片装得下")
+            .to_le_bytes(),
+    );
+}
+
 /// 删 → 回退 → 再删的合法历史（第 22 批 C 报告的合法历史之一）：新池新建文件、建空 inode 2（txg 4），把它 unlink 到底，挂着回退到 txg 4
 /// （谱系 (回退那次的 txg, 4)；墓碑树随现行那一版，头一条墓碑的死亡 txg 落在被回退抛弃的那一段里、不在谱系上），再把它 unlink 到底
 /// （又一条 (2, 同一个对象出生代) 的墓碑，在谱系上）。不按谱系滤的写法在这里判 I-9.20 不重号红。
@@ -6607,7 +6681,8 @@ fn pool_checker_verdict_on_the_image(image: &MemoryPool, invariant: &str) -> Inv
         .unwrap_or_else(|| panic!("池级 checker 报了 {invariant}"))
 }
 
-/// I-9.29 判不了那一形的流（[`record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_unreadable`]）：
+/// I-9.29 判不了那一形的流（[`record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed`] 取
+/// [`ChangeOfTheNewestInstanceTable::BothCopiesFlippedWithoutResealing`]）：
 /// CPU 一侧先钉流造对了——翻坏实例表之前整条落盘的镜像上 C 被滤掉、I-9.29 真被评估过且成立；翻坏之后整条落盘的镜像上 I-9.29 报部分没判。
 /// 流上 C 还是最新根的那几个状态，C 的实例表读得出、C 进候选集，同号不同出生代照比，CPU 红 I-9.29；GPU 每个状态逐位相同
 /// （实例 3 的根落了、C 写回了的状态上 GPU 照旧比的话红 I-9.29 而 CPU 不红）。
@@ -6615,8 +6690,9 @@ fn pool_checker_verdict_on_the_image(image: &MemoryPool, invariant: &str) -> Inv
 fn the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_object_birth_is_not_compared_once_the_newest_instance_table_does_not_read_back(
 ) {
     let (flow, image_with_the_newest_instance_table_readable) =
-        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_unreadable(
+        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
             "crash-judge-gpu-i929-newest-instance-table-unreadable",
+            ChangeOfTheNewestInstanceTable::BothCopiesFlippedWithoutResealing,
         );
     assert_eq!(
         pool_checker_verdict_on_the_image(&image_with_the_newest_instance_table_readable, "I-9.29"),
@@ -6657,6 +6733,49 @@ fn the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_objec
     );
 }
 
+/// 攻方 N5-a 那一形的流（[`record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed`] 取
+/// [`ChangeOfTheNewestInstanceTable::RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord`]；判决
+/// `research/prompts/m3-batch23-code-r1-main-verification.md` 第三节第 1 条「GPU 同改，对拍补 N5-a」）：
+/// CPU 一侧先钉流造对了——整条落盘的镜像上主走读沿最新根的实例表链走到了末片、C 被滤掉，I-9.29 真被评估过且成立
+/// （判不了的门另用只读不判的读法时这里报部分没判）。流上 C 还是最新根的那几个状态 CPU 红 I-9.29；GPU 每个状态逐位相同。
+/// 这条流上门开不开在违例位上看不出（C 滤掉了，比也不红）：GPU 门的开向由走读内核交接段那一个字与 CPU 同一遍走读钉住，
+/// 关向由上一条（两份都读不出）的流钉住。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_is_filtered_by_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind(
+) {
+    let (flow, _) =
+        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
+            "crash-judge-gpu-i929-newest-instance-table-unregistered-record-kind",
+            ChangeOfTheNewestInstanceTable::RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord,
+        );
+    let mut image_with_the_record_of_an_unregistered_kind = flow.base.clone();
+    image_with_the_record_of_an_unregistered_kind.apply_writes(&flow.writes);
+    assert_eq!(
+        pool_checker_verdict_on_the_image(&image_with_the_record_of_an_unregistered_kind, "I-9.29"),
+        InvariantVerdict::Holds,
+        "最新根的实例表链指针记录之前多一条 kind 2：主走读收下了全部行、走到了末片，被抛弃的 C 滤掉了，I-9.29 真被评估过且成立"
+    );
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_is_filtered_by_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind",
+    ) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    assert!(
+        comparison.cpu_checker_red_invariants.contains("I-9.29"),
+        "流 {} 上 C 还是最新根的状态 CPU 红 I-9.29（红过的：{:?}）",
+        flow.name,
+        comparison.cpu_checker_red_invariants
+    );
+    assert!(
+        comparison.mismatches.is_empty(),
+        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
+    );
+}
+
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container()
 {
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index f6576dc..535c799 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -401,6 +401,11 @@ struct Walk<'reader> {
     /// 最新根指着的实例表里的行：别的根按它判有效（回退之后被抛弃时间线的根不进 I-3.1 的并集），
     /// I-1.8（归并后版本全序） 的已发布谓词也读它。
     instance_table_rows: Vec<InstanceTableRow>,
+    /// 主走读沿最新根的实例表链走没走到末片（链指针记录说「没有下一片」的那一片）：走到了，`instance_table_rows` 就是整张表的行，
+    /// 回退候选集按它滤掉了被崩溃恢复抛弃的根；没走到（链上一片读不出、解不开、链指针记录不合法、链成了环）就滤不全。
+    /// I-9.29（同号只许一个出生代） 判不了的门读它，与候选集的滤法用同一遍走读（判决 `research/prompts/m3-batch23-code-r1-main-verification.md`
+    /// 第三节第 1 条，攻方改法 A）。
+    abandoned_roots_filtering: AbandonedRootsFilteringOfTheCandidateSet,
     /// 挂载根（池级 checker 里就是最新的那条根）的实例代号与 txg：已发布谓词的另外两个输入
     /// （I-1.2（块头写序已发布） / I-4.2（无被引用未提交块） 在遍历方向上各判一格要它）。
     mount_root_instance: u32,
@@ -961,6 +966,8 @@ impl<'reader> Walk<'reader> {
             accounting: BTreeMap::new(),
             accounting_seen: false,
             instance_table_rows: Vec::new(),
+            abandoned_roots_filtering:
+                AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable,
             mount_root_instance,
             mount_root_checkpoint_txg,
             referenced_units_judged_against_their_pointer: 0,
@@ -2829,7 +2836,8 @@ impl<'reader> Walk<'reader> {
     /// 沿链走一张实例表（D18（块里携带什么信息） 已定项 11：根记录直接持有第 0 片，第 k 片末尾的链指针记录指着第 k + 1 片）。
     /// 每一片判 I-2.5（位置条目升序）、I-2.1（校验和）、码 3 容器那几条（`judge_packed_container`）、I-1.1（身份四元组 =
     /// (0, 4, 片序号, 0)：链指针指到了别的单元、链成了环，都在这一格红）与 I-3.8 的链指针记录那一半（最后一条是合法的链指针记录）；
-    /// 挂载根那一张（`mount_root_instance_when_newest` 给了实例代号）把各片的记录交给 `judge_instance_table_rows` 判行那一半。
+    /// 挂载根那一张（`mount_root_instance_when_newest` 给了实例代号）把各片的记录交给 `judge_instance_table_rows` 判行那一半，
+    /// 走到末片（链指针记录说「没有下一片」）时记下候选集滤得全（`abandoned_roots_filtering`）。
     /// 迭代上界：第 k 片要求容器号是 k，走过的单元不再走（`visited_units`），每一轮是盘上一个不同的单元。
     /// 提前出口，链都不再往下走：读不出、身份不对（被引用的单元坏了或换了人，记进走读失败；头用不了由 `judge_unit_header` 记）；
     /// 容器记录不合法、链指针记录不合法（这一片自己写错了，只红 I-1.7 / I-3.8 那一格）；走到已经走过的单元（前一条根走过这条链）。
@@ -2918,7 +2926,13 @@ impl<'reader> Walk<'reader> {
                 });
             pages.push(records);
             match chain {
-                ChainRecordView::LastPage => break,
+                ChainRecordView::LastPage => {
+                    if mount_root_instance_when_newest.is_some() {
+                        self.abandoned_roots_filtering =
+                            AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable;
+                    }
+                    break;
+                }
                 ChainRecordView::NextPage(next_page) => {
                     pointer = next_page;
                     page_index += 1;
@@ -9326,14 +9340,17 @@ const TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS: [&str; 4] = [
 ];
 
 /// 回退候选集滤没滤得掉被崩溃恢复抛弃的根：候选集按最新根指着的实例表判被抛弃的根（D23（journal 的角色与格式） 已定项 14 回退段的候选集规则），
-/// 那张表读不出时一行都滤不掉，被抛弃时间线上的根也进了候选集。I-9.29（同号只许一个出生代） 跨版本比，这时记判不了
+/// 那张表读不全时滤不全，被抛弃时间线上的根可能进了候选集。I-9.29（同号只许一个出生代） 跨版本比，这时记判不了
 /// （`.claude/kb/invariants.md` I-9.29 那一行「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」）：被抛弃的线上同号可以是另一个对象。
+/// 读没读全看的是候选集滤法用的那一遍（主走读，`Walk::walk_instance_table_chain`），不另读一遍：另一个读法（只读不判的
+/// [`instance_table_rows_of_root_without_judging`]）拒收的比主走读多（链指针记录之前有不是行的记录、载荷 CRC 对不上），
+/// 用它当门会在候选集已经滤全时报判不了（判决 `research/prompts/m3-batch23-code-r1-main-verification.md` 第二节 K2 N5）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum AbandonedRootsFilteringOfTheCandidateSet {
-    /// 最新根的实例表沿链整张读得出（[`instance_table_rows_of_root_without_judging`] 交回了行）：被抛弃的根都按它滤掉了。
+    /// 主走读沿最新根的实例表链走到了末片（链指针记录说「没有下一片」）：整张表的行都收下了，被抛弃的根都按它滤掉了。
     FilteredByTheNewestRootsInstanceTable,
-    /// 最新根的实例表链上有一片读不出、解不开（同一个读法交回 None）：被抛弃的根可能留在候选集里。
-    UnfilteredAsTheNewestRootsInstanceTableDoesNotReadBack,
+    /// 主走读沿最新根的实例表链没走到末片（一片读不出、解不开、身份不对、链指针记录不合法、链成了环）：被抛弃的根可能留在候选集里。
+    UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable,
 }
 
 /// 装没装旋钮决定墓碑容器容量判哪一条：没装（旧入口）判产品那一条 I-9.21，装了判测试那一条 I-9.22。
@@ -9670,11 +9687,11 @@ fn judge_tombstones_and_the_lineage_of_the_versions(
         AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable => {
             judge_one_object_birth_per_inode_number(&object_births, judgements);
         }
-        AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheNewestRootsInstanceTableDoesNotReadBack => {
+        AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable => {
             judgements.partially_judged(
                 ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
                 format!(
-                    "最新根的实例表读不出（链上有一片读不出或解不开）：回退候选集里滤不掉被崩溃恢复抛弃的根，收到的 {} 个号一个都没比",
+                    "最新根的实例表读不出（主走读沿链没走到末片：有一片读不出、解不开或链指针记录不合法）：回退候选集里滤不掉被崩溃恢复抛弃的根，收到的 {} 个号一个都没比",
                     object_births.len()
                 ),
             );
@@ -10558,14 +10575,8 @@ pub fn check_pool_image_given_the_tombstone_container_capacity_knob(
         })
     };
     // 上面那张表是主走读沿链判下来的行，链断在哪一片就只有那一片之前的行：读不全时滤不掉的根认不出来。
-    // 滤没滤全按 I-7.9 与记录核对器读一条根自己的实例表那个读法判（链上哪一片读不出、解不开 ⇒ None）。
-    let abandoned_roots_filtering = match instance_table_rows_of_root_without_judging(
-        reader,
-        &roots[newest_index].2.record_bytes,
-    ) {
-        Some(_) => AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable,
-        None => AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheNewestRootsInstanceTableDoesNotReadBack,
-    };
+    // 滤没滤全就看这同一遍走读走没走到末片（`Walk::walk_instance_table_chain` 记下的），不另读一遍。
+    let abandoned_roots_filtering = walk.abandoned_roots_filtering;
     // 回退候选集的下界取 F 生效值（D16（发布语义） 已定项 1「生效」，SysPre）：max(各幸存盘最新持久有效根所带 F 的最大值,
     // 池里自证过的系统配置槽带的 F)。有效 = 按最新根指着的实例表判不是被抛弃的；每块盘取落在它上面的根环区域里 (txg, 实例) 最大的那一条
     // （最新根的实例表读不出时一行都没有、不滤）；被抛弃时间线上的根带的 F 也算进来（D16（发布语义） 已定项 1，用户 2026-09-28 定「算进」）。
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 16e378d..09cb6ac 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -3986,6 +3986,177 @@ fn an_abandoned_root_naming_another_object_birth_leaves_the_one_birth_invariant_
     }
 }
 
+/// 码 3 打包单元头里声明长度（2 字节）与记录数（2 字节）的偏移，载荷 CRC（4 字节）的偏移与头校验和的端点（`reseal_unit` 码 3 那一支的两个数；
+/// D18（块里携带什么信息） 已定项 11）。
+const PACKED_UNIT_DECLARED_LENGTH_OFFSET: usize = 8;
+const PACKED_UNIT_RECORD_COUNT_OFFSET: usize = 69;
+const PACKED_UNIT_PAYLOAD_CRC_OFFSET: usize = 89;
+const PACKED_UNIT_HEADER_END: usize = 107;
+/// 实例表一条记录的字节数；第 0 字节是种类（0 行、1 链指针记录，别的没登记），链指针记录第 1 字节是「有无下一片」（0 或 1）。
+const INSTANCE_TABLE_RECORD_BYTES: usize = 88;
+const INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD: u8 = 1;
+const INSTANCE_TABLE_RECORD_KIND_UNREGISTERED: u8 = 2;
+const CHAIN_RECORD_HAS_NEXT_PAGE_OFFSET: usize = 1;
+
+/// [`image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth`] 那一份（被抛弃的 C 与现行线同号不同出生代）上
+/// 再改最新根（实例 3）指着的实例表（两盘同槽，`change` 两份一起改），`reseal` 为真时重封它的两道校验和；新的整单元校验和补到引用它的根槽。
+/// 交回改过的镜像与那张实例表的槽号。
+fn image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
+    tag: &str,
+    change: impl Fn(&mut Vec<u8>),
+    reseal: bool,
+) -> (MemoryPool, u64) {
+    let mut image =
+        image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth(
+            tag,
+        );
+    let instance_table_slot = instance_table_slot_of_the_newest_root(&image);
+    mutate_unit_of(
+        &mut image,
+        &[(instance_table_slot, INSTANCE_TABLE_UNIT_BYTES)],
+        instance_table_slot,
+        |bytes| {
+            let record_count = usize::from(u16::from_le_bytes([
+                bytes[PACKED_UNIT_RECORD_COUNT_OFFSET],
+                bytes[PACKED_UNIT_RECORD_COUNT_OFFSET + 1],
+            ]));
+            assert!(
+                record_count >= 2,
+                "实例 3 的实例表一片里有行、末尾一条链指针记录：{record_count} 条"
+            );
+            let chain_record_start =
+                INSTANCE_TABLE_FIRST_ROW_START + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
+            assert_eq!(
+                (
+                    bytes[chain_record_start],
+                    bytes[chain_record_start + CHAIN_RECORD_HAS_NEXT_PAGE_OFFSET]
+                ),
+                (INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD, 0),
+                "实例 3 的实例表只有一片：最后一条是链指针记录、没有下一片"
+            );
+            change(bytes);
+        },
+        reseal,
+    );
+    (image, instance_table_slot)
+}
+
+/// 攻方 N5-a（`research/prompts/m3-batch23-code-r1-opus-output.md`「一、N5 打中」；判决 `research/prompts/m3-batch23-code-r1-main-verification.md`
+/// 第三节第 1 条）：最新根的实例表在链指针记录之前多插一条没登记种类（kind 2）的记录，重封、补到根槽。主走读照收 kind 0 的行、沿链走到末片，
+/// 候选集按这张表滤掉了被抛弃的 C：I-9.29（同号只许一个出生代） 真被评估过且成立。这张镜像别的几条红不红不在这里钉
+/// （实例表记录种类合不合法归根环线，同一判决第二节「附带：N5-a 那张镜像」）。
+/// 判别力：判不了的门另用只读不判的读法（它见到链指针记录之前不是行的记录就交回 None）时，这里报部分没判。
+#[test]
+fn a_record_of_an_unregistered_kind_before_the_chain_record_of_the_newest_instance_table_still_filters_the_abandoned_root_so_the_one_birth_invariant_holds(
+) {
+    let (image, instance_table_slot) =
+        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
+            "known-bad-one-birth-unregistered-record-kind-before-the-chain-record",
+            |bytes| {
+                let record_count = usize::from(u16::from_le_bytes([
+                    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET],
+                    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET + 1],
+                ]));
+                let chain_record_start = INSTANCE_TABLE_FIRST_ROW_START
+                    + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
+                let chain_record = bytes
+                    [chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES]
+                    .to_vec();
+                let mut record_of_an_unregistered_kind = vec![0u8; INSTANCE_TABLE_RECORD_BYTES];
+                record_of_an_unregistered_kind[0] = INSTANCE_TABLE_RECORD_KIND_UNREGISTERED;
+                bytes[chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES]
+                    .copy_from_slice(&record_of_an_unregistered_kind);
+                bytes[chain_record_start + INSTANCE_TABLE_RECORD_BYTES
+                    ..chain_record_start + 2 * INSTANCE_TABLE_RECORD_BYTES]
+                    .copy_from_slice(&chain_record);
+                let new_record_count = record_count + 1;
+                set_u16(
+                    bytes,
+                    PACKED_UNIT_RECORD_COUNT_OFFSET,
+                    u16::try_from(new_record_count).expect("一片装得下"),
+                );
+                set_u16(
+                    bytes,
+                    PACKED_UNIT_DECLARED_LENGTH_OFFSET,
+                    u16::try_from(new_record_count * INSTANCE_TABLE_RECORD_BYTES)
+                        .expect("一片装得下"),
+                );
+            },
+            true,
+        );
+    assert_eq!(
+        verdict(&image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER),
+        InvariantVerdict::Holds,
+        "最新根的实例表（槽 {instance_table_slot}）链指针记录之前多一条 kind 2：主走读收下了全部行、走到了末片，被抛弃的 C 滤掉了，I-9.29 要真被评估过且成立"
+    );
+}
+
+/// 攻方 N5-c（出处同上）：最新根的实例表载荷 CRC 那 4 字节翻一位、只重封头校验和，整单元校验和补到根槽。主走读头判得过，载荷 CRC 只红 I-2.3、
+/// 记录照用、沿链走到末片，候选集滤掉了被抛弃的 C：只红 I-2.3，I-9.29 真被评估过且成立。
+/// 判别力：判不了的门另用只读不判的读法（它连载荷 CRC 一起判，对不上就交回 None）时，这里报部分没判。
+#[test]
+fn a_payload_crc_mismatch_on_the_newest_instance_table_reddens_only_the_payload_crc_and_still_filters_the_abandoned_root_so_the_one_birth_invariant_holds(
+) {
+    let (image, instance_table_slot) =
+        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
+            "known-bad-one-birth-newest-instance-table-payload-crc",
+            |bytes| {
+                bytes[PACKED_UNIT_PAYLOAD_CRC_OFFSET] ^= 1;
+                singlefs_core::unit::seal_header_checksum(bytes, PACKED_UNIT_HEADER_END);
+            },
+            false,
+        );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-2.3"],
+        "最新根的实例表（槽 {instance_table_slot}）载荷 CRC 对不上、头与整单元校验和都对：判红的该只有 I-2.3：{verdicts:?}"
+    );
+    assert_eq!(
+        verdict(&image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER),
+        InvariantVerdict::Holds,
+        "主走读照用这一片的行、走到了末片，被抛弃的 C 滤掉了：I-9.29 要真被评估过且成立"
+    );
+}
+
+/// 攻方 N5-b（出处同上）：最新根的实例表链指针记录「有无下一片」写 2，重封、补到根槽。主走读收下这一片的行（I-3.8 红）就停，不知道后面还有没有片，
+/// 走不到末片：I-9.29 报部分没判，不报成立（这一片其实就是整张表，可镜像上分不出「只有一片」与「后面还有片」）。
+/// 判别力：把「链指针记录不合法」当成走到了末片时，候选集按这一片的行滤掉 C，I-9.29 报成立。
+#[test]
+fn a_malformed_chain_record_on_the_newest_instance_table_leaves_the_one_birth_invariant_partially_judged(
+) {
+    let (image, instance_table_slot) =
+        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
+            "known-bad-one-birth-newest-instance-table-malformed-chain-record",
+            |bytes| {
+                let record_count = usize::from(u16::from_le_bytes([
+                    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET],
+                    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET + 1],
+                ]));
+                let chain_record_start = INSTANCE_TABLE_FIRST_ROW_START
+                    + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
+                bytes[chain_record_start + CHAIN_RECORD_HAS_NEXT_PAGE_OFFSET] = 2;
+            },
+            true,
+        );
+    let verdicts = check_pool_image(&image);
+    assert!(
+        violated_invariants(&verdicts).contains(&"I-3.8"),
+        "最新根的实例表（槽 {instance_table_slot}）最后一条不是合法的链指针记录：I-3.8 红：{verdicts:?}"
+    );
+    match verdict(&image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER) {
+        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
+            unjudged.starts_with("最新根的实例表读不出"),
+            "部分没判的说明点名最新根的实例表读不出：{unjudged}"
+        ),
+        other_verdict @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::NotApplicable(_)) => panic!(
+            "主走读没沿最新根的实例表链走到末片（槽 {instance_table_slot} 链指针记录不合法）：I-9.29 要报部分没判，实际 {other_verdict:?}"
+        ),
+    }
+}
+
 /// 发布 B 写出的数据单元：它的分配记录在 B 那一版分配记录树的两片叶（`ALLOCATION_LEAVES_AFTER_OVERWRITE`）里，未释放、分配代 4，
 /// 单元头里的诞生代号也是 4（落点由 `overwrite_in_one_instance.rs` 的验收钉住）。
 const DATA_UNIT_AFTER_OVERWRITE: u64 = 50182;
````

## 实现员 n：mutations-append.tsv（/tmp/claude-1000/m3-batch25/n/patch/mutations-append.tsv）

````text
第 25 批 I-9.29 门（CPU）：判不了的门改回另用只读不判的读法读最新根的实例表（攻方 N5-a：链指针记录之前多一条 kind 2，主走读走到了末片、候选集滤掉了被抛弃的 C，门却报部分没判）	crates/singlefs-checker/src/walk.rs	    let abandoned_roots_filtering = walk.abandoned_roots_filtering;	    let abandoned_roots_filtering = match (walk.abandoned_roots_filtering, instance_table_rows_of_root_without_judging(reader, &roots[newest_index].2.record_bytes)) { (_, Some(_)) => AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable, (_, None) => AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable };	-p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table	a_record_of_an_unregistered_kind_before_the_chain_record_of_the_newest_instance_table_still_filters_the_abandoned_root_so_the_one_birth_invariant_holds
第 25 批 I-9.29 门（CPU）：判不了的门改回另用只读不判的读法读最新根的实例表（攻方 N5-c：载荷 CRC 对不上、整单元校验和对得上，主走读照用这一片的行，门却报部分没判）	crates/singlefs-checker/src/walk.rs	    let abandoned_roots_filtering = walk.abandoned_roots_filtering;	    let abandoned_roots_filtering = match (walk.abandoned_roots_filtering, instance_table_rows_of_root_without_judging(reader, &roots[newest_index].2.record_bytes)) { (_, Some(_)) => AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable, (_, None) => AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable };	-p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table	a_payload_crc_mismatch_on_the_newest_instance_table_reddens_only_the_payload_crc_and_still_filters_the_abandoned_root_so_the_one_birth_invariant_holds
第 25 批 I-9.29 门（CPU）：最新根的实例表链指针记录不合法也当主走读走到了末片（攻方 N5-b：分不出后面还有没有片，I-9.29 报成立而不是部分没判）	crates/singlefs-checker/src/walk.rs	                ChainRecordView::Malformed => break,\n            }\n        }\n        self.walk_parent_unit = parent_unit_when_the_chain_starts;	                ChainRecordView::Malformed => {\n                    if mount_root_instance_when_newest.is_some() {\n                        self.abandoned_roots_filtering = AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable;\n                    }\n                    break;\n                }\n            }\n        }\n        self.walk_parent_unit = parent_unit_when_the_chain_starts;	-p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table	a_malformed_chain_record_on_the_newest_instance_table_leaves_the_one_birth_invariant_partially_judged
第 25 批 GPU 判器 I-9.29 对拍 N5-a 那条流的 CPU 一侧：判不了的门改回另用只读不判的读法（整条落盘的镜像上 I-9.29 报部分没判，不是成立）	crates/singlefs-checker/src/walk.rs	    let abandoned_roots_filtering = walk.abandoned_roots_filtering;	    let abandoned_roots_filtering = match (walk.abandoned_roots_filtering, instance_table_rows_of_root_without_judging(reader, &roots[newest_index].2.record_bytes)) { (_, Some(_)) => AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable, (_, None) => AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable };	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_is_filtered_by_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind	the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_is_filtered_by_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind
第 25 批 GPU 判器 I-9.29：走读内核沿最新根的实例表链走到末片也不记（扫描内核那张表恒不比；N5-a 那条流上 C 还是最新根的状态 CPU 红 I-9.29、GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	                w_newest_instance_table_chain_reached_its_last_page = true;	                w_newest_instance_table_chain_reached_its_last_page = false;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_is_filtered_by_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind	the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_is_filtered_by_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind
````

## 实现员 n：mutations-replacements.tsv（/tmp/claude-1000/m3-batch25/n/patch/mutations-replacements.tsv）

````text
第 5 批 GPU 判器：I-9.26 改动计数不比	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	                w_judge(INVARIANT_I_9_26, le64(rd64(p.base, ro + INODE_RECORD_OBJECT_BIRTH_OFFSET), change_count) && le64(change_count, p.birth_txg));	                w_judge(INVARIANT_I_9_26, true);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_inode_change_count_runs_past_its_container_birth	the_gpu_judge_agrees_with_the_cpu_when_an_inode_change_count_runs_past_its_container_birth
第23批 X I-9.29：最新根的实例表读不出时当它滤过了被抛弃的根，照旧跨版本比（被抛弃的 C 同号不同出生代，I-9.29 判红而不是部分没判）	crates/singlefs-checker/src/walk.rs	                AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable,	                AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable,	-p singlefs-harness --test checker_known_bad_images -- an_abandoned_root_naming_another_object_birth	an_abandoned_root_naming_another_object_birth_leaves_the_one_birth_invariant_partially_judged_once_the_newest_instance_table_does_not_read_back
第 24 批 GPU 判器 I-9.29：最新根的实例表读不出时照旧比同号出生代（被抛弃的根与现行线同号不同出生代、最新根的实例表两份翻坏那条流上 CPU 报部分没判，GPU 红 I-9.29）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    d_object_births_are_compared = sw(H_BASE + H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE) == 1u;	    d_object_births_are_compared = true;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_object_birth_is_not_compared_once_the_newest_instance_table_does_not_read_back	the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_object_birth_is_not_compared_once_the_newest_instance_table_does_not_read_back
````
