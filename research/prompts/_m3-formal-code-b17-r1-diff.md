# 第 17 批代码轮被判的改动全文（2026-10-04）

17-1 的补丁原样（`/tmp/claude-1000/m3-formal-17-1/patch/crates.patch`，sha256 99261b5d…，`git apply --check` 对合入前的主仓过、已打进主仓）与它的变异追加；17-4 只改了一份测试文件里造坏镜像的那个函数，那份文件相对 HEAD 还带着别的会话的改动，这里只抄 17-4 改后的函数原文与它的断言。

## 17-1 `crates.patch`

````diff
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
index e2b5b66..fc3bc66 100644
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -70,7 +70,10 @@ pub const SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY: &str =
 /// `research/prompts/m3-inv-b-r1-main-verification.md`，用户 2026-10-01 定案），**只往末尾追加**：层 0 的违例位按下标 i ↔ 这张表第 i 条，
 /// 前 49 条的下标不动。位宽改到 ≥ 128 之前，下标 64 起的那一条在层 0 的 GPU 判器上没有位（另一个实现员在 checker 档改）。
 /// 第 66 条（下标 65）是 I-3.5（里程碑三第 8 批）：GPU 判器内核还没判它（`crash_judge_gpu` 的覆盖表另列）。
-pub const IMPLEMENTED_INVARIANTS: [&str; 66] = [
+///
+/// 第 67、68 条（下标 66、67）是 C673（三 oracle 合谋测试的补 oracle 改法没做） ⑤ ⑥ 那两条：I-9.27（树表条目的头 ID 取值）与
+/// I-5.5（从 mkfs 水位建树那一版的 t1 落点），同样只往末尾追加；GPU 判器内核两条都判。
+pub const IMPLEMENTED_INVARIANTS: [&str; 68] = [
     "I-1.1",
     "I-1.2",
     "I-1.3",
@@ -137,6 +140,8 @@ pub const IMPLEMENTED_INVARIANTS: [&str; 66] = [
     ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
     SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
     REFERENCE_INTERVALS_ARE_EXACT,
+    HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
+    FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
 ];
 
 /// I-1.12：走读跟随的一条指针，偏移 34 的出生树等于被指单元头里的出生树——码 1、码 3 取头偏移 43；码 2 取哪个字段随 C289 定，
@@ -182,6 +187,13 @@ pub const SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION: &str = "I-8.
 /// 接不上的那一对只判用户可见树之外的单元（`research/prompts/m3-birth-floor-r3-main-verification.md`，用户 2026-10-03 定丙）。
 /// ③ ④（deadlist）这一版不判。
 pub const REFERENCE_INTERVALS_ARE_EXACT: &str = "I-3.5";
+/// I-9.27：走读到的任一版树表里，条目偏移 116 的头 ID 按种类取值——inode 树（种类 2）等于它自己的树 ID；extent 树（1）、`deleted_inodes`（9）、
+/// 墓碑树（10）等于这张树表里某一条种类 2 条目的树 ID（没有种类 2 条目时判红）；种类 3、4、6、7、8、11、12 等于 0；别的种类码不判。
+/// 不拿字面 65537 比（D8（核心索引结构） 已定项 8「头 ID 8 字节的取值」句末的关系判据，判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K1）。
+pub const HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES: &str = "I-9.27";
+/// I-5.5：从 mkfs 水位第一次建树的那一版（树表非空、种类 1 条目的树 ID 是 65536、每条诞生 txg 都等于这一版的 `checkpoint_txg`）引用的数据单元里，
+/// 每块盘上落点槽号最小的那一个等于单元区起始槽号 U + 4（D3（空间分配） 已定项 8 射程，判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K2）。
+pub const FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK: &str = "I-5.5";
 
 /// 判定累加器：每条不变量记评估了几次、第一处违例、整条不适用的理由，以及射程里有一部分判不了的理由。
 #[derive(Default)]
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 996bef7..879c532 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -11,7 +11,8 @@ use singlefs_format::{
     DATA_UNIT_BYTES, EXTENT_LEAF_RECORD_BYTES, EXTENT_TREE_INTERNAL_ENTRY_BYTES,
     EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES, INODE_INTERNAL_ENTRY, JOURNAL_RECORD_BYTES,
     JOURNAL_SAFETY_FACTOR, MAPPING_ENTRY_BYTES, NODE_BYTES, NODE_POINTER_BYTES,
-    NONCE_MAC_ALGORITHM_RESERVED_BYTES, SLOT_BYTES, TREE_TABLE_ENTRY_BYTES,
+    NONCE_MAC_ALGORITHM_RESERVED_BYTES, SLOT_BYTES, TREE_IDENTIFIER_WATERMARK_AT_MKFS,
+    TREE_TABLE_ENTRY_BYTES,
 };
 
 use crate::position_addressed::{
@@ -37,6 +38,8 @@ use crate::image::{
     PointerLocation, PointerView, PoolGeometry,
     ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
     DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
+    FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
+    HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
     INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
     INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
     JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
@@ -1444,6 +1447,12 @@ impl<'reader> Walk<'reader> {
         version_name: &str,
     ) {
         judge_tree_table_entries_ordering(&mut self.judgements, &tree_table.entries, version_name);
+        // I-9.27 只看同一张表里条目之间的关系、与哪一版无关：共享这片树表的别的版本不重判（与 I-9.16 同）。
+        judge_tree_table_entry_head_identifiers(
+            &mut self.judgements,
+            &tree_table.entries,
+            version_name,
+        );
         let entries_of_the_walk: Vec<TreeTableEntryOfTheWalk> = tree_table
             .entries
             .iter()
@@ -3355,6 +3364,58 @@ const ALLOCATION_RECORD_GENERATION_OFFSET: usize = 12;
 const TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8;
 /// 树表条目里种类 2 字节的偏移（树 ID 8 + 条目长度 2 之后）。
 const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 8 + 2;
+/// 树表条目里头 ID 8 字节的偏移（树 ID 8 + 条目长度 2 + 种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8 + 诞生 txg 8，
+/// D8（核心索引结构） 已定项 8 的字段表），I-9.27 判它。
+const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8 + 8;
+/// 从 mkfs 水位第一次建树的那一版，每块盘上落点最小的数据单元比单元区起始槽号 U 多几槽（D3（空间分配） 已定项 8 射程：
+/// mkfs 的实例表占 U、U + 1，树表占 U + 2，第一个 32768 对齐的空槽对从 U + 4 起；环长不是默认值时 U 现算、仍是 U + 4），I-5.5 判它。
+const FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START: u64 = 4;
+
+/// I-9.27：树表条目的头 ID 按种类该等于什么（`.claude/kb/invariants.md` I-9.27 那一行；D8（核心索引结构） 已定项 8「头 ID 8 字节的取值」
+/// 句末的关系判据：判的是关系，不拿字面 65537 比）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum HeadIdentifierOfATreeTableEntry {
+    /// 种类 2（inode 树）：等于这条条目自己的树 ID。
+    ItsOwnTreeIdentifier,
+    /// 种类 1（extent 树）、9（`deleted_inodes`）、10（墓碑树）：等于同一张树表里种类 2 条目的树 ID；
+    /// 种类 2 条目多于一条时放宽成「等于其中某一条的」（收严留给多头那一批），一条都没有时判红。
+    TreeIdentifierOfAnInodeTreeEntryOfTheSameTable,
+    /// 种类 3、4、6、7、8、11、12（无归属，D5（快照 / 空间记账机制） 已定项 9）：等于 0。
+    Zero,
+    /// 别的种类码：那一行不判。
+    NotJudged,
+}
+
+impl HeadIdentifierOfATreeTableEntry {
+    const fn of_tree_kind(kind: u16) -> Self {
+        match kind {
+            TREE_KIND_INODE => Self::ItsOwnTreeIdentifier,
+            TREE_KIND_EXTENT | TREE_KIND_DELETED_INODES | TREE_KIND_TOMBSTONE => {
+                Self::TreeIdentifierOfAnInodeTreeEntryOfTheSameTable
+            }
+            TREE_KIND_ALLOCATION
+            | TREE_KIND_ACCOUNTING
+            | TREE_KIND_LIVELIST
+            | TREE_KIND_SPARSE_SIDE_TABLE
+            | TREE_KIND_DEADLIST
+            | TREE_KIND_LINEAGE
+            | TREE_KIND_LOGGED_OPERATIONS => Self::Zero,
+            _ => Self::NotJudged,
+        }
+    }
+
+    /// 判红时写进第一处违例的那半句：这一条的头 ID 该等于什么。
+    fn expectation(self, tree: u64, inode_tree_identifiers: &[u64]) -> String {
+        match self {
+            Self::ItsOwnTreeIdentifier => format!("inode 树条目的头 ID 该等于它自己的树 ID {tree}"),
+            Self::TreeIdentifierOfAnInodeTreeEntryOfTheSameTable => format!(
+                "该等于同一张树表里种类 2 条目的树 ID（这张表里的种类 2 条目：{inode_tree_identifiers:?}）"
+            ),
+            Self::Zero => "该等于 0".to_string(),
+            Self::NotJudged => "这一种不判".to_string(),
+        }
+    }
+}
 
 /// I-9.16（树表条目按树 ID 严格升序且合发号次序）：一版的树表判一次，两道任一道不成立即判红，第一处违例写明是哪一版、哪两条（下标从 0 数）。
 /// ① 盘上相邻两条的树 ID（条目打头 8 字节）严格升序；② 按发号次序（[`TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`]），
@@ -3418,6 +3479,55 @@ fn judge_tree_table_entries_ordering(
     });
 }
 
+/// I-9.27（树表条目的头 ID 取值）：一版的树表判一次（树表被几版共享时只在第一个走到它的那一版判，同 I-9.16），每条条目按种类判一格
+/// （[`HeadIdentifierOfATreeTableEntry`]），第一处违例写明是哪一版、第几条（下标从 0 数）。树表 0 条的版本一格也不判。
+/// checker 自己按字段表切一份，不与实现共用代码（D13（验证路线） 已定项 5）。
+/// 条目宽不到头 ID 那 8 字节（盘上字段，`index_node_view` 只保证 ≥ key 宽 8）时读不出头 ID：这张树表不判，记成射程里判不了的那一部分
+/// （同 I-9.16 读不出种类那一格的处置）。
+fn judge_tree_table_entry_head_identifiers(
+    judgements: &mut Judgements,
+    tree_table_entries: &[Vec<u8>],
+    version_name: &str,
+) {
+    let head_identifiers_are_readable = tree_table_entries
+        .iter()
+        .all(|entry| entry.len() >= TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8);
+    if !head_identifiers_are_readable {
+        judgements.part_of_the_range_not_judged(
+            HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
+            "树表单元自述的条目宽不到头 ID 那 8 字节（条目偏移 116 起）：这张树表的头 ID 读不出",
+        );
+        return;
+    }
+    let tree_identifier_of = |entry: &Vec<u8>| read_u64(entry, 0);
+    let kind_of = |entry: &Vec<u8>| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET);
+    let inode_tree_identifiers: Vec<u64> = tree_table_entries
+        .iter()
+        .filter(|entry| kind_of(entry) == TREE_KIND_INODE)
+        .map(tree_identifier_of)
+        .collect();
+    for (index, entry) in tree_table_entries.iter().enumerate() {
+        let tree = tree_identifier_of(entry);
+        let kind = kind_of(entry);
+        let head_identifier = read_u64(entry, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
+        let expected = HeadIdentifierOfATreeTableEntry::of_tree_kind(kind);
+        let holds = match expected {
+            HeadIdentifierOfATreeTableEntry::ItsOwnTreeIdentifier => head_identifier == tree,
+            HeadIdentifierOfATreeTableEntry::TreeIdentifierOfAnInodeTreeEntryOfTheSameTable => {
+                inode_tree_identifiers.contains(&head_identifier)
+            }
+            HeadIdentifierOfATreeTableEntry::Zero => head_identifier == 0,
+            HeadIdentifierOfATreeTableEntry::NotJudged => continue,
+        };
+        judgements.judge(HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES, holds, || {
+            format!(
+                "{version_name}的树表：第 {index} 条（树 ID {tree}、种类 {kind}）的头 ID 是 {head_identifier}，{}",
+                expected.expectation(tree, &inode_tree_identifiers)
+            )
+        });
+    }
+}
+
 /// 一条分配记录在盘上的样子。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 struct AllocationRecordView {
@@ -4001,6 +4111,132 @@ fn collect_tree_references(
     }
 }
 
+/// 走读到的一版里 I-5.5 要的三样：称呼（判红时写进第一处违例）、`checkpoint_txg` 与它的树表指针
+/// （根环里的根取根记录偏移 36 起的 86 字节，由记录施加出来的那一版取记录新根段里的那一条）。
+struct VersionToJudgeTheFirstDataUnitOf<'record> {
+    name: String,
+    checkpoint_txg: u64,
+    tree_table_pointer: &'record [u8],
+}
+
+/// I-5.5（从 mkfs 水位建树那一版的 t1 落点）：走读到的每一版（回退候选集里每条根，含由记录施加出来的那几版）各自重读一遍树表，
+/// ① 树表非空，且种类 1（extent 树）条目的树 ID 等于 mkfs 种下的水位 65536；② 树表里每条条目的诞生 txg（条目偏移 108）都等于这一版的
+/// `checkpoint_txg`——两样都成立就是从 mkfs 水位第一次建树的那一版：它的 extent 树引用的数据单元里，每块盘上落点槽号最小的那一个
+/// 要等于 U + 4（U 是所选系统配置偏移 417 的单元区起始槽号）。这一版没有数据单元时不判，不满足 ①② 的版本不判。只读、不判别的不变量：
+/// 主走读按 `visited_units` 只走一遍共享子树，共享的 extent 树只算进第一条走到它的版本，这一条要的是「这一版自己引用了哪些数据单元」
+/// （与 [`references_of_root`] 同一个缘故）。
+///
+/// 下面几格 `.claude/kb/invariants.md` I-5.5 那一行没写，判不了，记成射程里判不了的那一部分（没有违例时整条报不适用、带理由），不替条款定：
+/// 树表读不出、条目宽不到字段表的 200 字节（认不出这一版是不是这一类）；种类 1 条目不止一条（① 读哪一条）；
+/// 认出来的那一版 extent 树读不全（数据单元数不全）；它有数据单元、而池里某块盘上一个都没有（那块盘上「最小的那一个」没有对象）。
+fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
+    reader: &dyn ImageReader,
+    versions: &[VersionToJudgeTheFirstDataUnitOf<'_>],
+    unit_area_start_slot: u64,
+    judgements: &mut Judgements,
+) {
+    let invariant = FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK;
+    let expected_slot = unit_area_start_slot + FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START;
+    let devices = reader.devices();
+    let mut cache = IndexNodeCache::new();
+    for version in versions {
+        let tree_table_pointer = parse_node_pointer(version.tree_table_pointer);
+        if tree_table_pointer.all_zero {
+            continue;
+        }
+        let Some(tree_table) =
+            read_index_node_without_judging(reader, &tree_table_pointer, &mut cache)
+        else {
+            judgements.part_of_the_range_not_judged(
+                invariant,
+                "有一版的树表读不出：认不出它是不是从 mkfs 水位第一次建树的那一版",
+            );
+            continue;
+        };
+        if tree_table.entries.is_empty() {
+            continue;
+        }
+        if tree_table
+            .entries
+            .iter()
+            .any(|entry| entry.len() < tree_table_entry_bytes())
+        {
+            judgements.part_of_the_range_not_judged(
+                invariant,
+                "有一版的树表条目宽不到字段表的 200 字节：认不出它是不是从 mkfs 水位第一次建树的那一版",
+            );
+            continue;
+        }
+        let extent_entries: Vec<&Vec<u8>> = tree_table
+            .entries
+            .iter()
+            .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_EXTENT)
+            .collect();
+        let extent_entry = match extent_entries.as_slice() {
+            [] => continue,
+            [only_extent_entry] => *only_extent_entry,
+            [_, _, ..] => {
+                judgements.part_of_the_range_not_judged(
+                    invariant,
+                    "有一版的树表里种类 1（extent 树）条目不止一条：认法 ① 读哪一条，I-5.5 那一行没写",
+                );
+                continue;
+            }
+        };
+        let is_built_from_the_mkfs_watermark = read_u64(extent_entry, 0)
+            == TREE_IDENTIFIER_WATERMARK_AT_MKFS
+            && tree_table.entries.iter().all(|entry| {
+                read_u64(entry, TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET) == version.checkpoint_txg
+            });
+        if !is_built_from_the_mkfs_watermark {
+            continue;
+        }
+        let extent_root = parse_node_pointer(&extent_entry[14..100]);
+        if extent_root.all_zero {
+            continue;
+        }
+        let Some((_, data_pointers)) =
+            extent_tree_without_judging(reader, &extent_root, &mut cache)
+        else {
+            judgements.part_of_the_range_not_judged(
+                invariant,
+                "从 mkfs 水位第一次建树的那一版 extent 树读不全：它引用的数据单元数不全",
+            );
+            continue;
+        };
+        let mut smallest_data_unit_slot_on_each_device: BTreeMap<u32, u64> = BTreeMap::new();
+        for location in data_pointers
+            .iter()
+            .filter(|pointer| !pointer.all_zero)
+            .flat_map(|pointer| pointer.locations)
+        {
+            smallest_data_unit_slot_on_each_device
+                .entry(location.device)
+                .and_modify(|smallest_slot| *smallest_slot = (*smallest_slot).min(location.slot))
+                .or_insert(location.slot);
+        }
+        if smallest_data_unit_slot_on_each_device.is_empty() {
+            continue;
+        }
+        for device in &devices {
+            match smallest_data_unit_slot_on_each_device.get(device) {
+                Some(&smallest_slot) => {
+                    judgements.judge(invariant, smallest_slot == expected_slot, || {
+                        format!(
+                            "{}（从 mkfs 水位第一次建树的那一版）：盘 {device} 上落点最小的数据单元在槽 {smallest_slot}，不是单元区起始槽号 {unit_area_start_slot} + {FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START} = {expected_slot}",
+                            version.name
+                        )
+                    });
+                }
+                None => judgements.part_of_the_range_not_judged(
+                    invariant,
+                    "从 mkfs 水位第一次建树的那一版有数据单元、而池里有一块盘上一个都没有：那块盘上判不判，I-5.5 那一行没写",
+                ),
+            }
+        }
+    }
+}
+
 /// 候选集里的一条根：它在 `roots` 里的下标、它的 checkpoint_txg 与这一遍取到的引用集合。
 struct ScannedCandidateRoot {
     root_index: usize,
@@ -7625,6 +7861,36 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
         &walk.instance_table_rows,
         &mut walk.judgements,
     );
+    // I-5.5：回退候选集里每条根与由记录施加出来的那几版，各自重读树表与 extent 树（只读），认出从 mkfs 水位第一次建树的那一版再判落点；
+    // U 取所选系统配置偏移 417（`geometry` 是第一块择到系统配置的盘的那一份，I-7.13 判过它是环长现算的那个槽）。
+    let versions_to_judge_the_first_data_unit_of: Vec<VersionToJudgeTheFirstDataUnitOf<'_>> =
+        candidate_indexes
+            .iter()
+            .map(|index| {
+                let root = &roots[*index].2;
+                VersionToJudgeTheFirstDataUnitOf {
+                    name: format!("根（实例 {}、txg {}）", root.instance, root.checkpoint_txg),
+                    checkpoint_txg: root.checkpoint_txg,
+                    tree_table_pointer: &root.record_bytes[36..122],
+                }
+            })
+            .chain(versions_applied_only_by_records.iter().map(|version| {
+                VersionToJudgeTheFirstDataUnitOf {
+                    name: format!(
+                        "由记录施加出来的那一版（实例 {}、txg {}）",
+                        version.instance, version.checkpoint_txg
+                    ),
+                    checkpoint_txg: version.checkpoint_txg,
+                    tree_table_pointer: version.tree_table_and_mapping_root_pointers().0,
+                }
+            }))
+            .collect();
+    judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
+        reader,
+        &versions_to_judge_the_first_data_unit_of,
+        geometry.unit_area_start_slot,
+        &mut walk.judgements,
+    );
     if walk.units_judged_against_an_older_candidate_root == 0 {
         walk.judgements.not_applicable(
             UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index 83f9f31..25d4f64 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -1460,6 +1460,36 @@ fn w_judge_tree_table_entries_ordering(n: Node) {
     w_judge(INVARIANT_I_9_16, true);
 }
 
+// 树表条目的头 ID（walk.rs judge_tree_table_entry_head_identifiers，I-9.27）：inode 树等于自己的树 ID；extent 树、deleted_inodes、墓碑树
+// 等于同一张表里某一条 inode 树条目的树 ID（一条都没有就红）；3、4、6、7、8、11、12 等于 0；别的种类不判。条目宽不到头 ID 那 8 字节时整张表不判
+fn w_judge_tree_table_entry_head_identifiers(n: Node) {
+    if (n.entry_width < TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8u) {
+        return;
+    }
+    for (var entry_index = 0u; entry_index < n.entry_count; entry_index = entry_index + 1u) {
+        let entry = entry_offset(n, entry_index);
+        let tree = rd64(n.base, entry);
+        let kind = rd16(n.base, entry + 10u);
+        let head = rd64(n.base, entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
+        if (kind == TREE_KIND_INODE) {
+            w_judge(INVARIANT_I_9_27, eq64(head, tree));
+        } else if (kind == TREE_KIND_EXTENT || kind == TREE_KIND_DELETED_INODES || kind == TREE_KIND_TOMBSTONE) {
+            var names_an_inode_tree_of_this_table = false;
+            for (var other_index = 0u; other_index < n.entry_count; other_index = other_index + 1u) {
+                let other = entry_offset(n, other_index);
+                if (rd16(n.base, other + 10u) == TREE_KIND_INODE && eq64(rd64(n.base, other), head)) {
+                    names_an_inode_tree_of_this_table = true;
+                }
+            }
+            w_judge(INVARIANT_I_9_27, names_an_inode_tree_of_this_table);
+        } else if (kind == TREE_KIND_ALLOCATION || kind == TREE_KIND_ACCOUNTING || kind == TREE_KIND_LIVELIST
+            || kind == TREE_KIND_SPARSE_SIDE_TABLE || kind == TREE_KIND_DEADLIST || kind == TREE_KIND_LINEAGE
+            || kind == TREE_KIND_LOGGED_OPERATIONS) {
+            w_judge(INVARIANT_I_9_27, is_zero64(head));
+        }
+    }
+}
+
 // 数据单元（walk.rs walk_extent_data_pointer）
 fn w_walk_extent_data_pointer(base: u32, o: u32, inode: vec2<u32>, offset: vec2<u32>, tree: vec2<u32>, fsid_low: vec2<u32>) {
     let p = parse_data_pointer(base, o);
@@ -2573,6 +2603,7 @@ fn w_walk_tree_table_and_mapping(tree_table_pointer: Ptr, mapping_root_pointer:
     }
     let tt = read.n;
     w_judge_tree_table_entries_ordering(tt);
+    w_judge_tree_table_entry_head_identifiers(tt);
     let parent_valid = w_parent_valid;
     let parent_device = w_parent_device;
     let parent_slot = w_parent_slot;
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
index 697ff08..9c741b7 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
@@ -137,6 +137,29 @@ const INVARIANT_I_7_5: u32 = 63u;
 
 const INVARIANT_I_8_5: u32 = 64u;
 
+// 第 67、68 条：C673 ⑤ ⑥ 那两条（只往末尾追加；下标 65 的 I-3.5 内核还没判，这里没有它的常量）
+const INVARIANT_I_9_27: u32 = 66u;
+
+const INVARIANT_I_5_5: u32 = 67u;
+
+// I-9.27 读的另外四个种类码（walk.rs 的 TREE_KIND_*；1..8 在 crash_judge_common.wgsl）
+const TREE_KIND_DELETED_INODES: u32 = 9u;
+
+const TREE_KIND_TOMBSTONE: u32 = 10u;
+
+const TREE_KIND_LINEAGE: u32 = 11u;
+
+const TREE_KIND_LOGGED_OPERATIONS: u32 = 12u;
+
+// 树表条目里头 ID 8 字节的偏移（walk.rs 的 TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET）
+const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: u32 = 116u;
+
+// mkfs 种下的树 ID 水位（singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS），I-5.5 认法 ① 拿 extent 树条目的树 ID 比它
+const TREE_IDENTIFIER_WATERMARK_AT_MKFS: u32 = 65536u;
+
+// 从 mkfs 水位第一次建树那一版，每块盘上落点最小的数据单元比单元区起始槽号多几槽（walk.rs 的 FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START）
+const FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START: u32 = 4u;
+
 // 违例位：位 i 在第 i / 32 字的第 i % 32 位（位 i ↔ IMPLEMENTED_INVARIANTS[i]），存进结论的第 POOL_CHECKER_VIOLATED_FIRST_WORD 字起。
 var<private> violated: array<u32, POOL_CHECKER_VIOLATED_WORDS>;
 
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
index 810e12e..2efcda3 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -40,6 +40,37 @@ fn r_note(p: Ptr) {
     }
 }
 
+// I-5.5 那一遍（judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark）开着时，r_extent_tree 读到的每条数据指针
+// 按盘表下标记下每块盘上最小的落点槽号（walk.rs 同名函数里的 smallest_data_unit_slot_on_each_device）；不在盘表里的盘不记
+var<private> r_collecting_data_unit_slots: bool;
+
+var<private> r_has_data_unit_on_device: array<u32, 8>;
+
+var<private> r_smallest_data_unit_slot_on_device: array<vec2<u32>, 8>;
+
+fn r_note_data_unit_slot(device: u32, slot: vec2<u32>) {
+    let device_count = hdr(3u);
+    for (var device_index = 0u; device_index < device_count && device_index < MAX_DEVICES; device_index = device_index + 1u) {
+        if (hdr(12u + device_index) != device) {
+            continue;
+        }
+        if (r_has_data_unit_on_device[device_index] == 0u || lt64(slot, r_smallest_data_unit_slot_on_device[device_index])) {
+            r_smallest_data_unit_slot_on_device[device_index] = slot;
+            r_has_data_unit_on_device[device_index] = 1u;
+        }
+    }
+}
+
+// extent 树里的一条数据指针：照 r_note 记；I-5.5 那一遍开着时再记两条位置条目的落点（全零指针不记，同 walk.rs 过滤 all_zero）
+fn r_note_data_pointer(p: Ptr) {
+    r_note(p);
+    if (!r_collecting_data_unit_slots || p.all_zero) {
+        return;
+    }
+    r_note_data_unit_slot(p.device0, p.slot0);
+    r_note_data_unit_slot(p.device1, p.slot1);
+}
+
 // read_index_node_without_judging：第一份校验和对得上的那一份，按 index_node_view 解
 fn r_read_index_node(p: Ptr) -> Node {
     let s = read_unit_via_locations(p, NODE_BYTES);
@@ -247,7 +278,7 @@ fn r_extent_tree(base: u32, offset: u32) -> bool {
                         sws(child + 8u, entry_inode.y);
                         pending = pending + 1u;
                     } else if (tag == 2u) {
-                        r_note(parse_data_pointer(n.base, o + 25u));
+                        r_note_data_pointer(parse_data_pointer(n.base, o + 25u));
                     } else {
                         return false;
                     }
@@ -257,7 +288,7 @@ fn r_extent_tree(base: u32, offset: u32) -> bool {
                     return false;
                 }
                 for (var e = 0u; e < n.entry_count; e = e + 1u) {
-                    r_note(parse_data_pointer(n.base, entry_offset(n, e) + 24u));
+                    r_note_data_pointer(parse_data_pointer(n.base, entry_offset(n, e) + 24u));
                 }
             }
             continue;
@@ -1740,11 +1771,76 @@ fn judge_instance_table_rows_against_the_roots_referencing_them() {
 }
 
 // 第三批的入口
+// I-5.5（walk.rs judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark）的一版：树表指针在 (base, offset)、它的 txg。
+// 认法 ① 树表非空、种类 1 条目恰好一条且树 ID 是 mkfs 水位 65536；② 每条诞生 txg（条目偏移 108）都等于这一版的 txg。认出来、extent 树读得全时，
+// 盘表里每块有它数据单元的盘上最小落点要等于 U + 4（U 取交接段里所选系统配置的单元区起始槽号）。CPU 记成射程里判不了的那几格
+// （树表读不出、条目宽不到 200、种类 1 不止一条、extent 树读不全、某块盘上没有数据单元）在这里一格都不置位
+fn judge_first_data_unit_of_a_version(base: u32, tree_table_pointer_offset: u32, txg: vec2<u32>) {
+    let tree_table_pointer = parse_node_pointer(base, tree_table_pointer_offset);
+    if (tree_table_pointer.all_zero) {
+        return;
+    }
+    let tt = r_read_index_node(tree_table_pointer);
+    if (!tt.ok || tt.entry_count == 0u || tt.entry_width < 200u) {
+        return;
+    }
+    var extent_entry_count = 0u;
+    var extent_entry = 0u;
+    var every_birth_is_this_version = true;
+    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
+        let entry = entry_offset(tt, entry_index);
+        if (rd16(tt.base, entry + 10u) == TREE_KIND_EXTENT) {
+            extent_entry_count = extent_entry_count + 1u;
+            extent_entry = entry;
+        }
+        if (!eq64(rd64(tt.base, entry + 108u), txg)) {
+            every_birth_is_this_version = false;
+        }
+    }
+    if (extent_entry_count != 1u || !every_birth_is_this_version
+        || !eq64(rd64(tt.base, extent_entry), u64_of(TREE_IDENTIFIER_WATERMARK_AT_MKFS))) {
+        return;
+    }
+    if (parse_node_pointer(tt.base, extent_entry + 14u).all_zero) {
+        return;
+    }
+    for (var device_index = 0u; device_index < MAX_DEVICES; device_index = device_index + 1u) {
+        r_has_data_unit_on_device[device_index] = 0u;
+    }
+    r_noting = false;
+    r_collecting_data_unit_slots = true;
+    let read_the_whole_extent_tree = r_extent_tree(tt.base, extent_entry + 14u);
+    r_collecting_data_unit_slots = false;
+    if (!read_the_whole_extent_tree) {
+        return;
+    }
+    let expected_slot = add64(handoff_unit_area_start_slot(), u64_of(FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START));
+    let device_count = hdr(3u);
+    for (var device_index = 0u; device_index < device_count && device_index < MAX_DEVICES; device_index = device_index + 1u) {
+        if (r_has_data_unit_on_device[device_index] == 1u && !eq64(r_smallest_data_unit_slot_on_device[device_index], expected_slot)) {
+            violate(INVARIANT_I_5_5);
+        }
+    }
+}
+
+// I-5.5：回退候选集里每条根（树表指针在根记录偏移 36）与由记录施加出来的那几版（记录新根段打头的树表指针）
+fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark() {
+    for (var candidate = 0u; candidate < c_candidate_count; candidate = candidate + 1u) {
+        let root = checker_root_of(sw(C_CANDIDATES + candidate));
+        judge_first_data_unit_of_a_version(root.base, 36u, root.txg);
+    }
+    for (var version = 0u; version < c_version_count; version = version + 1u) {
+        let record = checker_record_at(sw(R_DATA + version));
+        judge_first_data_unit_of_a_version(record.base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET, record.txg);
+    }
+}
+
 fn scan_stage(fsid_low: vec2<u32>) {
     judge_merged_version_total_order(fsid_low);
     judge_release_generation_and_tree_table_birth();
     judge_allocation_records_disjoint();
     judge_allocation_generations_against_unit_births();
+    judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark();
     judge_rollback_floor_raises_against_their_ceilings();
     judge_accounting(fsid_low);
     judge_instance_table_rows_against_the_roots_referencing_them();
diff --git a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
index 761b06b..9d675b5 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
@@ -3,7 +3,7 @@
 //! 七类 oracle、记录核对器两条判据，判法与 CPU 判器（`crate::crash::judge_crash_image`）逐条相同；GPU 与 CPU 逐状态比是它的判别力自证。
 //! 内核分五份源码：`crash_judge_common.wgsl`（各内核共用的解析与查表）、`crash_judge_recovery.wgsl`（恢复、oracle、记录核对器）、
 //! `crash_judge_checker_common.wgsl`（池级 checker 两个内核共用的读法与交接段）、`crash_judge_checker.wgsl`（走读内核：第一、二批不变量，
-//! 连同里程碑三二·1 那一批里的十五条）、`crash_judge_checker_scan.wgsl`（扫描内核：第三批与 I-3.12）；判不了的状态（表越过内核容量）置最高位、交 CPU 判。
+//! 连同里程碑三二·1 那一批里的十五条与 I-9.27）、`crash_judge_checker_scan.wgsl`（扫描内核：第三批、I-3.12 与 I-5.5）；判不了的状态（表越过内核容量）置最高位、交 CPU 判。
 //!
 //! 一个状态的结论 [`VERDICT_WORDS`] 个 u32：第 0 字是红位（位 0..6 看 journal 那一遍 oracle 的七类、位 8..14 不看那一遍、
 //! 位 16 根在而记录都不在、位 17 恢复自称的状态少单元、位 31 判不了）；第 1 字判不了的原因（判得了时低 16 位是看 journal 那一遍
@@ -59,20 +59,22 @@ const _: () = assert!(
 );
 
 /// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：前 65 条——三批 49 条，加里程碑三二·1 那一批 16 条
-/// （判决 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行），对拍逐位比、不屏蔽。
+/// （判决 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行），再加 C673（三 oracle 合谋测试的补 oracle 改法没做）
+/// ⑤ ⑥ 那两条（下标 66 是 I-9.27，走读内核挨着 I-9.16 按表判；下标 67 是 I-5.5，扫描内核每一版重读树表与 extent 树判），对拍逐位比、不屏蔽。
 pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[
     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
     26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
-    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64,
+    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 66, 67,
 ];
 
-/// 池级 checker 已经判、GPU 内核还没判的不变量（下标接在覆盖表之后）：下标 65 是 I-3.5（引用区间的精确性，里程碑三第 8 批进池级 checker，
+/// 池级 checker 已经判、GPU 内核还没判的不变量：下标 65 是 I-3.5（引用区间的精确性，里程碑三第 8 批进池级 checker，
 /// 派发没含 GPU 内核那一半，交主 agent 定）。内核那一位恒 0：对拍逐位比照旧不屏蔽，CPU 在哪个状态上判红它，对拍就在那个状态上报不同；
 /// 崩溃放量由 GPU 判的那几趟判不到它（`crash_amplification` 按违例位取不变量）。
 pub const POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET: &[usize] = &[65];
 
-// 覆盖表加上「内核还没判」那张表是池级 checker 的每一条、按下标次序：新立一条不变量而两张表都没跟上时这里编译不过，
-// 不悄悄让 GPU 那一位恒 0；还没判的那几条要在「内核还没判」那张表里点名。
+// 覆盖表与「内核还没判」那张表合起来是池级 checker 的每一条：每个下标恰好在其中一张表里、两张表各按下标升序。
+// 新立一条不变量而两张表都没跟上时这里编译不过，不悄悄让 GPU 那一位恒 0；还没判的那几条要在「内核还没判」那张表里点名。
+// 「内核还没判」那几条不必在覆盖表的末尾：I-3.5（下标 65）之后立的 I-9.27、I-5.5 内核判了，排在它后面。
 const _: () = {
     assert!(
         POOL_CHECKER_COVERED_INVARIANT_INDEXES.len()
@@ -80,23 +82,52 @@ const _: () = {
             == singlefs_checker::image::IMPLEMENTED_INVARIANTS.len(),
         "池级 checker 新立的不变量 GPU 内核还没判：补内核，再把下标补进覆盖表（暂不补内核的，点名进「内核还没判」那张表）"
     );
-    let mut position = 0;
+    let mut position = 1;
     while position < POOL_CHECKER_COVERED_INVARIANT_INDEXES.len() {
         assert!(
-            POOL_CHECKER_COVERED_INVARIANT_INDEXES[position] == position,
-            "覆盖表按下标次序、一条不缺"
+            POOL_CHECKER_COVERED_INVARIANT_INDEXES[position - 1]
+                < POOL_CHECKER_COVERED_INVARIANT_INDEXES[position],
+            "覆盖表按下标升序"
         );
         position += 1;
     }
-    let mut position_not_judged = 0;
+    let mut position_not_judged = 1;
     while position_not_judged < POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET.len() {
         assert!(
-            POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET[position_not_judged]
-                == POOL_CHECKER_COVERED_INVARIANT_INDEXES.len() + position_not_judged,
-            "「内核还没判」那张表接在覆盖表之后、按下标次序"
+            POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET[position_not_judged - 1]
+                < POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET[position_not_judged],
+            "「内核还没判」那张表按下标升序"
         );
         position_not_judged += 1;
     }
+    // 每个下标恰好在一张表里：两张表都升序、合起来条数等于不变量条数，再要求每个下标至少在一张表里，就一条不缺、一条不重。
+    let mut invariant_index = 0;
+    while invariant_index < singlefs_checker::image::IMPLEMENTED_INVARIANTS.len() {
+        let mut is_listed = false;
+        let mut covered_position = 0;
+        while covered_position < POOL_CHECKER_COVERED_INVARIANT_INDEXES.len() {
+            if POOL_CHECKER_COVERED_INVARIANT_INDEXES[covered_position] == invariant_index {
+                is_listed = true;
+            }
+            covered_position += 1;
+        }
+        let mut not_judged_position = 0;
+        while not_judged_position
+            < POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET.len()
+        {
+            if POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET[not_judged_position]
+                == invariant_index
+            {
+                is_listed = true;
+            }
+            not_judged_position += 1;
+        }
+        assert!(
+            is_listed,
+            "池级 checker 的每一条要么在覆盖表里、要么在「内核还没判」那张表里"
+        );
+        invariant_index += 1;
+    }
 };
 
 /// 一个状态结论里池级 checker 的违例位（[`POOL_CHECKER_VIOLATED_BIT_CAPACITY`] 位，位 i ↔ `IMPLEMENTED_INVARIANTS[i]`）。
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 422e0e7..64622ad 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -1149,6 +1149,138 @@ fn record_new_pool_with_a_previous_snapshot_txg_on_a_tree_other_than_the_inode_t
     )
 }
 
+/// I-9.27（树表条目的头 ID 取值）：发布那张树表里 extent 树那条（种类 1）的头 ID（条目偏移 116）写成 0、重封到根——
+/// 它该等于同一张表里 inode 树条目的树 ID。
+fn record_new_pool_with_the_head_identifier_of_the_extent_tree_entry_written_zero(
+    tag: &str,
+) -> RecordedFlow {
+    let tree_table = |bytes: &[u8]| {
+        unit_class(bytes) == 2
+            && index_node_key_width(bytes) == 8
+            && index_node_entry_width(bytes) == 200
+    };
+    record_new_pool_with_a_resealed_field(
+        tag,
+        "new-pool-extent-tree-entry-head-identifier-zero",
+        &tree_table,
+        false,
+        &|bytes| {
+            let header_end = 86 + 2 * usize::from(bytes[51]);
+            let count = usize::from(u16::from_le_bytes(
+                bytes[header_end - 4..header_end - 2].try_into().expect("2"),
+            ));
+            let entry = (0..count)
+                .map(|entry| 131 + entry * 200)
+                .find(|offset| {
+                    u16::from_le_bytes(bytes[offset + 10..offset + 12].try_into().expect("2")) == 1
+                })
+                .expect("树表里有 extent 树那条");
+            assert_ne!(
+                bytes[entry + 116..entry + 124],
+                [0u8; 8],
+                "extent 树那条的头 ID 本来不是 0"
+            );
+            bytes[entry + 116..entry + 124].fill(0);
+        },
+    )
+}
+
+/// I-5.5（从 mkfs 水位建树那一版的 t1 落点）：新池新建文件那一版唯一的数据单元（两份副本，单元区起始槽号 + 4）挪到再往后两槽
+/// （下一个 32 KiB 对齐的槽对，这条流上空着）——两次单元写的偏移、流里每一处指着它的位置条目（extent 上段叶内联的数据指针、
+/// 中央映射叶、journal 记录）的槽号一起改，单元重封到根、记录重封。树表与诞生 txg 一个字节不动，这一版照样被认出，
+/// 每块盘上落点最小的数据单元在 U + 6、不是 U + 4。
+fn record_new_pool_with_the_first_data_unit_two_slots_further(tag: &str) -> RecordedFlow {
+    const SLOTS_MOVED: u64 = 2;
+    let mut flow = record_new_pool_file_creation(tag, "new-pool-first-data-unit-two-slots-further");
+    let data_unit_copies: Vec<usize> = flow
+        .writes
+        .iter()
+        .enumerate()
+        .filter(|(_, write)| {
+            write.kind == StepKind::UnitWrite
+                && write.bytes().is_some_and(|bytes| unit_class(bytes) == 1)
+        })
+        .map(|(index, _)| index)
+        .collect();
+    assert_eq!(
+        data_unit_copies.len(),
+        2,
+        "新池新建文件只写一个数据单元，两块盘各一份"
+    );
+    // (旧的位置条目 14 字节, 新的槽号 6 字节)
+    let renamed_locations: Vec<(Vec<u8>, [u8; 6])> = data_unit_copies
+        .iter()
+        .map(|index| {
+            let write = &flow.writes[*index];
+            let slot = write.offset.0 / 16384;
+            let mut location = Vec::with_capacity(14);
+            location.extend_from_slice(&write.device.0.to_le_bytes());
+            location.extend_from_slice(&slot.to_le_bytes()[..6]);
+            location.extend_from_slice(&crc32c(write.bytes().expect("单元写带字节")).to_le_bytes());
+            let mut new_slot = [0u8; 6];
+            new_slot.copy_from_slice(&(slot + SLOTS_MOVED).to_le_bytes()[..6]);
+            (location, new_slot)
+        })
+        .collect();
+    for index in &data_unit_copies {
+        flow.writes[*index].offset.0 += SLOTS_MOVED * 16384;
+    }
+    let old_locations: Vec<Vec<u8>> = renamed_locations
+        .iter()
+        .map(|(location, _)| location.clone())
+        .collect();
+    let names_the_data_unit = |bytes: &[u8]| {
+        old_locations.iter().any(|location| {
+            bytes
+                .windows(14)
+                .any(|window| window == location.as_slice())
+        })
+    };
+    let rename = move |bytes: &mut [u8]| {
+        for (location, new_slot) in &renamed_locations {
+            let mut position = 0;
+            while position + 14 <= bytes.len() {
+                if bytes[position..position + 14] == location[..] {
+                    bytes[position + 4..position + 10].copy_from_slice(new_slot);
+                    position += 14;
+                } else {
+                    position += 1;
+                }
+            }
+        }
+    };
+    let unit_referrers: Vec<usize> = flow
+        .writes
+        .iter()
+        .enumerate()
+        .filter(|(_, write)| {
+            write.kind == StepKind::UnitWrite
+                && write
+                    .bytes()
+                    .is_some_and(|bytes| unit_class(bytes) != 1 && names_the_data_unit(bytes))
+        })
+        .map(|(index, _)| index)
+        .collect();
+    let journal_referrers: Vec<usize> = flow
+        .writes
+        .iter()
+        .enumerate()
+        .filter(|(_, write)| {
+            write.kind == StepKind::JournalRecord && write.bytes().is_some_and(names_the_data_unit)
+        })
+        .map(|(index, _)| index)
+        .collect();
+    assert!(
+        !unit_referrers.is_empty(),
+        "流里有单元指着那个数据单元（extent 上段叶、中央映射叶）"
+    );
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &unit_referrers, &rename);
+    for index in journal_referrers {
+        rewrite_journal_record_and_reseal(&mut flow, index, &rename);
+    }
+    flow
+}
+
 /// I-3.13（记账行代段等于这一版的 txg）：发布那片记账叶第二条的代段（key 偏移 14）加 1、重封到根。取中间那条：
 /// 首末两条的 key 是叶头里的区间，改了它们 I-1.1 先红。
 fn record_new_pool_with_an_accounting_row_of_another_generation(tag: &str) -> RecordedFlow {
@@ -1948,7 +2080,7 @@ fn verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several(
 /// 在扫描内核读回违例位之后种第 128 位（下标 127）：
 /// 两份种过的内核判出的每个状态，违例位等于没种的那一份判出的再并上种的位（第 65 位每个状态都有，第 128 位只在扫描内核判完的状态上有），
 /// 其余的字逐字相同。违例位的高 64 位在走读内核写出、扫描内核读回又写出、Rust 读出这三处任一处被截掉，这条红。
-/// 池级 checker 今天 66 条（下标 0..=65；内核判前 65 条，下标 65 的 I-3.5 内核还没判，`crash_judge_gpu` 的「内核还没判」那张表），种的是第 67 位（下标 66）。
+/// 池级 checker 今天 68 条（下标 0..=67；内核判下标 65 之外的每一条，下标 65 的 I-3.5 内核还没判，`crash_judge_gpu` 的「内核还没判」那张表），种的是第 69 位（下标 68）。
 #[test]
 fn a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels() {
     const FIRST_BIT_OF_THE_HIGH_HALF: usize = 64;
@@ -2496,6 +2628,28 @@ fn the_gpu_judge_agrees_with_the_cpu_when_a_tree_table_entry_other_than_the_inod
     );
 }
 
+/// I-9.27：extent 树那条树表条目的头 ID 写成 0。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_extent_tree_entry_carries_a_head_identifier_other_than_the_inode_tree(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_extent_tree_entry_carries_a_head_identifier_other_than_the_inode_tree",
+        &record_new_pool_with_the_head_identifier_of_the_extent_tree_entry_written_zero("crash-judge-gpu-i927"),
+        "I-9.27",
+    );
+}
+
+/// I-5.5：从 mkfs 水位建树的那一版，数据单元不落在单元区起始槽号 + 4。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_first_tree_building_version_places_its_data_unit_past_the_unit_area_start_plus_four(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_first_tree_building_version_places_its_data_unit_past_the_unit_area_start_plus_four",
+        &record_new_pool_with_the_first_data_unit_two_slots_further("crash-judge-gpu-i55"),
+        "I-5.5",
+    );
+}
+
 /// I-3.13：记账行的代段不是这一版的 txg。
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_when_an_accounting_row_carries_another_generation_than_its_version(
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index ddaaa2f..3945070 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -5839,8 +5839,7 @@ pub fn publish_new_inodes<Device: BlockDevice>(
 /// 墓碑树开放容器追加一条整对象墓碑（死亡 txg = 这次发布的 checkpoint_txg，D18（块里携带什么信息） 已定项 24 第 2 条）；不立 logged_ops。
 /// 数据单元与 extent 树照抄。一个事务（不写用户数据，不切）。改动计数设成这次发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88
 /// 「最后一次改动所在发布的 checkpoint_txg」：用户 2026-10-03 定 Q-F「算改动，改实现」，意图那次与每一批都算一次改动）；
-/// 写入时间不改：记录里没有 ctime 字段，`write_time_seconds` 由调用方给，删除步没有确定的来源，改它会让同一份崩溃镜像各自续做出不同的盘
-/// （D8（核心索引结构） 已定项 16 第 7 条 ①b）。
+/// 写入时间照抄这条记录上一版的值（D8（核心索引结构） 已定项 6 写入时间的取值：unlink 的意图与各批照抄这条记录上一版的写入时间）。
 ///
 /// # Errors
 /// `inode` 不是树里一条活着的记录（没有、或已经 nlink = 0）⇒ `UnlinkOfAnInodeThatIsNotALiveRecord`；其余同 `publish_version`。
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 5c623b3..a055e70 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -14,6 +14,8 @@ use common::{
 use singlefs_checker::image::{
     InvariantVerdict, ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
     DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
+    FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
+    HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
     INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
     INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
     JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
@@ -1308,9 +1310,65 @@ fn known_bad_images(clean: &MemoryPool) -> Vec<(&'static str, Mutation)> {
                 );
             }),
         ),
+        // I-5.5（判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K2）：A 那一版是从 mkfs 水位第一次建树的那一版，
+        // 它唯一的数据单元（50180 = 单元区起始槽号 50176 + 4）搬到再往后两槽，每块盘上落点最小的数据单元就不在 U + 4。
+        // 分配记录与记账照旧记着 50180，这份镜像上不只红 I-5.5；这一格只要它判违例。
+        (
+            FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
+            Box::new(move_the_data_unit_of_the_first_file_two_slots_further),
+        ),
     ]
 }
 
+/// 新池新建文件写出的那个数据单元（`DATA_UNIT`，32 KiB）原样搬到往后两槽（写完新池新建文件那份镜像上空着，下一个 32 KiB 对齐的槽对）：
+/// 两盘各写一份；`UNITS` 里引用它的单元（extent 树根里的数据指针、中央映射叶）把位置条目的槽号改成新的、重封，
+/// 新的整单元校验和沿引用链补到根槽。50180 那两份原样留着：journal 记录的点名项仍指着它。
+fn move_the_data_unit_of_the_first_file_two_slots_further(image: &mut MemoryPool) {
+    const DATA_UNIT_BYTES: usize = 32768;
+    let moved_to = DATA_UNIT + 2;
+    assert!(
+        UNITS.iter().all(|(start, _)| *start != moved_to),
+        "槽 {moved_to} 在写完新池新建文件的镜像上空着"
+    );
+    let data_unit = read(image, 0, DATA_UNIT * SLOT, DATA_UNIT_BYTES);
+    for device in DEVICES {
+        write(image, device, moved_to * SLOT, &data_unit);
+    }
+    let checksum = crc32_castagnoli(&data_unit);
+    let mut parents_changed = Vec::new();
+    for (container, length) in UNITS.iter().copied() {
+        if container == DATA_UNIT {
+            continue;
+        }
+        let before = read(image, 0, container * SLOT, length);
+        let mut bytes = before.clone();
+        let mut touched = false;
+        for device in DEVICES {
+            touched |= replace_all(
+                &mut bytes,
+                &location_pattern(device, DATA_UNIT, checksum),
+                &location_pattern(device, moved_to, checksum),
+            );
+        }
+        if touched {
+            reseal_unit(&mut bytes);
+            for device in DEVICES {
+                write(image, device, container * SLOT, &bytes);
+            }
+            parents_changed.push((
+                container,
+                crc32_castagnoli(&before),
+                crc32_castagnoli(&bytes),
+            ));
+        }
+    }
+    assert!(
+        !parents_changed.is_empty(),
+        "写完新池新建文件的镜像上有单元指着那个数据单元"
+    );
+    propagate(image, &UNITS, parents_changed);
+}
+
 /// 发布 B（txg 4）之后的干净镜像：A 的十二个单元（十四个槽）在这次发布里改写成已释放、释放代 4，树表是 B 写的那一版，
 /// A 那一版仍被根环里 A 的根指着。I-3.9（释放代落在停止引用它的那一格区间里）与 I-9.14（树表条目的诞生 txg 跨根不变）
 /// 要的跨根比较到这里才有内容：写完新池新建文件的那份镜像上只有 A 的根有树表条目（mkfs 种的第 0 版树表是空的、
@@ -6282,6 +6340,38 @@ fn known_bad_images_of_pointer_journal_and_generation_fields() -> Vec<(&'static
                 );
             }),
         ),
+        // I-9.27（判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K1）：A 的树表（50252）第 0 条（extent 树，种类 1）的
+        // 头 ID（条目偏移 116）从同一张表里 inode 树条目的树 ID 改成 0，重封、补到根。头 ID 别的判定都不读，只红 I-9.27。
+        (
+            HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
+            Box::new(|image: &mut MemoryPool| {
+                mutate_unit(
+                    image,
+                    TREE_TABLE,
+                    |bytes| {
+                        let entry = tree_table_entry_offset(0);
+                        assert_eq!(
+                            u16::from_le_bytes([
+                                bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET],
+                                bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET + 1]
+                            ]),
+                            TREE_KIND_EXTENT,
+                            "A 的树表第 0 条是 extent 树"
+                        );
+                        let inode_tree_entry =
+                            tree_table_entry_offset(inode_tree_entry_index_in(bytes));
+                        let field = entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET;
+                        assert_eq!(
+                            get_u64(bytes, field),
+                            get_u64(bytes, inode_tree_entry),
+                            "extent 树那条的头 ID 本来等于 inode 树条目的树 ID"
+                        );
+                        set_u64(bytes, field, 0);
+                    },
+                    true,
+                );
+            }),
+        ),
     ]
 }
 
@@ -6333,6 +6423,12 @@ fn set_the_extent_offset_of_the_instance_table_pointer_in_the_records_of_these_r
     }
 }
 
+/// 树表条目里头 ID 那 8 字节的偏移：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8 + 诞生 txg 8
+/// （D8（核心索引结构） 已定项 8 的字段表）。
+const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8 + 8;
+/// 树的种类码里 extent 树那一个（D8（核心索引结构） 已定项 9 的登记表）。
+const TREE_KIND_EXTENT: u16 = 1;
+
 /// 上面那批坏镜像各自**只**红在自己那一条上；干净镜像上那几条要真被评估过且成立（不是「不适用」）。
 #[test]
 fn each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant() {
diff --git a/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs b/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
new file mode 100644
index 0000000..00e7097
--- /dev/null
+++ b/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
@@ -0,0 +1,431 @@
+//! 池级 checker 的两条（C673（三 oracle 合谋测试的补 oracle 改法没做） ⑤ ⑥，判决 `research/prompts/c673-clauses-r1-main-verification.md`
+//! 第二节 K1、K2）：
+//! - I-9.27（树表条目的头 ID 取值）照同一张树表里的关系判、不拿字面 65537 比：崩溃恢复丢掉实例 2 的第一个文件版本之后，
+//!   实例 3 从根环水位 65548 发树（inode 树 65549）的那段合法历史上一条不红；
+//! - I-5.5（从 mkfs 水位建树那一版的 t1 落点）在新池新建文件（默认 768 MiB 环与 128 MiB 环）、只暖机过的实例 2 写第一个文件的那一版上
+//!   真被评估过且成立；第一个文件版本离开根环之后只剩覆盖写那几版（诞生 txg 不等于这一版的 txg），一版都不判。
+//!
+//! 盘一律是内存稀疏盘（4 GiB × 2），不落镜像文件。
+
+mod common;
+
+use common::{
+    crash_state_devices, file_content, geometry, memory_pool_of_sparse_devices, parameters,
+    unreadable_root_slot_of, with_unreadable_ranges, without_unreadable_ranges,
+    FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
+    FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
+};
+use singlefs_checker::image::{
+    InvariantVerdict, FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
+    HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
+};
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::make_filesystem::{
+    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
+};
+use singlefs_core::mount::mount_writable;
+use singlefs_core::root_ring::{slot_offset, target_for_publish};
+use singlefs_core::transaction::{
+    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion,
+    PoolWriter, TransactionOutput,
+};
+use singlefs_format::{
+    DATA_UNIT_BYTES, ROOT_RING_REGIONS, ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
+};
+use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
+use singlefs_harness::{RecordingBlockDevice, SharedStream};
+
+type MemoryDevice = RecordingBlockDevice<SparseBlockDevice>;
+
+/// 根环一圈的槽数（区域 3 × 每区 8 槽）：接着发这么多版之后，这之前那一版的根槽一定被新根盖掉了。
+const ROOT_RING_SLOT_COUNT: u64 = ROOT_RING_REGIONS * ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM;
+/// 从 mkfs 水位第一次建树的那一版，每块盘上落点最小的数据单元比单元区起始槽号多几槽（D3（空间分配） 已定项 8 射程，测试这边另写一份）。
+const FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START: u64 = 4;
+/// 非默认环长的那一种：128 MiB 环，单元区起始槽号现算（攻方在这一格上量过 9216，`research/prompts/c673-clauses-r1-opus-model/out/probe.out`）。
+const JOURNAL_RING_BYTES_OF_128_MEBIBYTES: u64 = 128 << 20;
+
+/// 两块空的内存稀疏盘，写录进同一条流。
+fn memory_devices(stream: &SharedStream) -> Vec<(DeviceIdentity, MemoryDevice)> {
+    (0..2u32)
+        .map(|number| {
+            (
+                DeviceIdentity(number),
+                RecordingBlockDevice::with_shared_stream(
+                    DeviceIdentity(number),
+                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
+                    stream.clone(),
+                ),
+            )
+        })
+        .collect()
+}
+
+fn verdict_of(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
+    check_pool_image(image)
+        .into_iter()
+        .find(|(identifier, _)| *identifier == invariant)
+        .expect("池级 checker 的清单里有这一条")
+        .1
+}
+
+/// 判红的那几条与各自第一处违例。
+fn violations_of(image: &MemoryPool) -> Vec<(&'static str, String)> {
+    check_pool_image(image)
+        .into_iter()
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
+            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+        })
+        .collect()
+}
+
+/// mkfs 同一个进程里那条流（mkfs → 取号 → 暖机两代 → 第一个文件版本），交回那份镜像、单元区起始槽号与第一个文件版本。
+fn new_pool_file(
+    make_filesystem_parameters: &MakeFilesystemParameters,
+) -> (MemoryPool, u64, TransactionOutput) {
+    let stream = SharedStream::retaining_contents();
+    let mut devices = memory_devices(&stream);
+    let genesis = make_filesystem(make_filesystem_parameters, &mut devices).expect("mkfs");
+    let mut allocator =
+        allocator_after_make_filesystem(make_filesystem_parameters, &devices, &genesis);
+    let first_file = {
+        let mut writer = PoolWriter::new(make_filesystem_parameters, &mut devices);
+        let instance = acquire_instance(&mut writer).expect("取号");
+        let warmed_up = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
+        publish_first_file(
+            &mut writer,
+            &mut allocator,
+            warmed_up.roots.last().expect("暖机两代根"),
+            FirstFile {
+                content: &file_content(),
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
+            },
+            instance,
+            &warmed_up.last_record_bytes,
+        )
+        .expect("新池新建文件")
+    };
+    (
+        memory_pool_of_sparse_devices(&devices),
+        genesis.unit_area_start.slot().0,
+        first_file,
+    )
+}
+
+/// 一个镜像上 I-5.5 真被评估过且成立、别的不变量一条都不红；`history` 是这份镜像怎么来的（写进断言消息）。
+/// I-5.5 先判：数据单元落点被挪了的实现上，红的第一条断言就是它。
+fn assert_the_first_data_unit_placement_holds_and_nothing_reddens(
+    image: &MemoryPool,
+    history: &str,
+) {
+    assert_eq!(
+        verdict_of(
+            image,
+            FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK
+        ),
+        InvariantVerdict::Holds,
+        "{history}：I-5.5 要真被评估过且成立"
+    );
+    assert_eq!(
+        violations_of(image),
+        Vec::new(),
+        "{history}：池级 checker 一条都不红"
+    );
+    assert_eq!(
+        verdict_of(image, HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES),
+        InvariantVerdict::Holds,
+        "{history}：I-9.27 要真被评估过且成立"
+    );
+}
+
+/// 攻方原型 `k1_recovered_first_file_version_issues_trees_from_the_ring_watermark` 那段历史（`writable_mount_of_a_formatted_pool.rs`
+/// 那条用例的内存盘版本）：只做过 mkfs 的池可写挂载两次（实例 1 只暖机，实例 2 写行、暖机），实例 2 发第一个文件版本（txg 5，树 65536..65547），
+/// 崩在 txg 5 的根槽 FUA 之后、系统配置轮换之前；可写挂载一次，txg 5 的根槽与它那条记录点名的数据单元盘 0 那一份读不出——择根落到 (2, 4)，
+/// 实例 3 的水位按根环与环里记录取 max = 65548，从它发第一个文件版本（inode 树 65549，头 ID 都是 65549）。故障解除之后接着覆盖写，
+/// 直到被丢掉的 (2, 5) 那一槽被新根盖掉（不留着它：它引用的数据单元槽被实例 3 复用了，那一形是 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到））。
+/// 这份镜像合法：池级 checker 一条不红，I-9.27 真被评估过且成立（字面 65537 的读法在这里误红 4 条）；树 ID 不从 65536 起，I-5.5 一版都不判。
+#[test]
+fn a_first_file_version_issuing_its_trees_from_the_ring_watermark_after_a_recovery_keeps_the_head_identifier_relation(
+) {
+    let stream = SharedStream::retaining_contents();
+    let mut devices = memory_devices(&stream);
+    make_filesystem(&parameters(), &mut devices).expect("mkfs");
+    let mkfs_operation_count = stream.operations().len();
+    let after_mkfs = memory_pool_of_sparse_devices(&devices);
+    drop(mount_writable(&parameters(), &mut devices).expect("实例 1：只暖机"));
+    let second_instance = mount_writable(&parameters(), &mut devices).expect("实例 2");
+    assert_eq!(second_instance.output.instance, InstanceGeneration(2));
+    let PoolVersion::WithoutFile(version_of_the_second_instance) = &second_instance.current else {
+        panic!("只暖机过的池上现行那一版树表 0 条")
+    };
+    let mut allocator_of_the_second_instance = second_instance.allocator.clone();
+    let dropped_first_file = {
+        let publish_parameters = parameters();
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        publish_first_file(
+            &mut writer,
+            &mut allocator_of_the_second_instance,
+            &version_of_the_second_instance.root,
+            FirstFile {
+                content: &file_content(),
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
+            },
+            second_instance.output.instance,
+            &version_of_the_second_instance.record_bytes,
+        )
+        .expect("实例 2 发第一个文件版本")
+    };
+    let dropped_txg = dropped_first_file.root.checkpoint_txg;
+    assert_eq!(
+        (dropped_txg, dropped_first_file.tree_identifiers.inode.0),
+        (CheckpointTxg(5), 65537),
+        "实例 2 写行 txg 3、暖机 txg 4，第一个文件版本 txg 5，从 mkfs 水位发树"
+    );
+    let operations = stream.retained_operations();
+    let (writes, _segments) = writes_and_segments(&operations[mkfs_operation_count..], &geometry());
+    let target = target_for_publish(dropped_txg, geometry().root_ring_slots_per_region);
+    let region_device =
+        parameters().region_devices[usize::try_from(target.region).expect("区域号")];
+    let root_slot_offset = slot_offset(target, geometry().fixed_structure_slot_spacing);
+    let root_slot_write = writes
+        .iter()
+        .position(|write| {
+            write.is_force_unit_access
+                && write.device == region_device
+                && write.offset == root_slot_offset
+        })
+        .expect("录制流里有 txg 5 的根槽 FUA 写");
+    let persisted: Vec<bool> = (0..writes.len())
+        .map(|write_index| write_index <= root_slot_write)
+        .collect();
+    let crashed = crash_state_devices(
+        &after_mkfs,
+        &writes,
+        &persisted,
+        &SharedStream::retaining_contents(),
+    );
+    let named_data_unit = dropped_first_file.data_pointers[0].locations[0];
+    let unreadable = SharedUnreadableRanges::new(
+        vec![
+            unreadable_root_slot_of(dropped_txg, FailingReadsOfARange::Every),
+            UnreadableRange {
+                device: named_data_unit.device,
+                offset_in_bytes: named_data_unit.slot.to_device_offset().0,
+                length_in_bytes: DATA_UNIT_BYTES,
+                failing_reads: FailingReadsOfARange::Every,
+            },
+        ],
+        UnreadableRangeReadBack::DeviceError,
+    );
+    let mut reopened_with_the_fault = with_unreadable_ranges(crashed, &unreadable);
+    let third_instance = mount_writable(&parameters(), &mut reopened_with_the_fault)
+        .expect("txg 5 的根读不出、那条记录验不过：择根落到 (2, 4)，照常可写挂载");
+    let mut reopened = without_unreadable_ranges(reopened_with_the_fault);
+    let PoolVersion::WithoutFile(version_of_the_third_instance) = &third_instance.current else {
+        panic!("落到实例 2 的暖机根：现行那一版树表 0 条")
+    };
+    assert_eq!(
+        version_of_the_third_instance.root.tree_identifier_watermark, 65548,
+        "实例 3 那一版带根环与环里记录的水位 max：实例 2 的记录带的 65548"
+    );
+    let mut allocator_of_the_third_instance = third_instance.allocator.clone();
+    let first_file_of_the_third_instance = {
+        let publish_parameters = parameters();
+        let mut writer = PoolWriter::new(&publish_parameters, reopened.as_mut_slice());
+        publish_first_file(
+            &mut writer,
+            &mut allocator_of_the_third_instance,
+            &version_of_the_third_instance.root,
+            FirstFile {
+                content: &file_content(),
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
+            },
+            third_instance.output.instance,
+            &version_of_the_third_instance.record_bytes,
+        )
+        .expect("水位 65548、树表 0 条：照样发得出第一个文件版本")
+    };
+    assert_eq!(
+        first_file_of_the_third_instance.tree_identifiers.inode.0, 65549,
+        "实例 3 从 65548 发树：extent 树 65548、inode 树 65549"
+    );
+    let mut previous = first_file_of_the_third_instance;
+    while previous.root.checkpoint_txg.0 < dropped_txg.0 + ROOT_RING_SLOT_COUNT {
+        let content_bytes = 1000 + usize::try_from(previous.root.checkpoint_txg.0).expect("小");
+        let next = {
+            let publish_parameters = parameters();
+            let mut writer = PoolWriter::new(&publish_parameters, reopened.as_mut_slice());
+            publish_overwrite(
+                &mut writer,
+                &mut allocator_of_the_third_instance,
+                &previous,
+                FirstFile {
+                    content: &file_content()[..content_bytes],
+                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 180,
+                },
+                third_instance.output.instance,
+            )
+            .expect("覆盖写")
+        };
+        previous = next;
+    }
+    let image = memory_pool_of_sparse_devices(&reopened);
+    assert_eq!(
+        violations_of(&image),
+        Vec::new(),
+        "树 ID 从根环水位发的合法镜像：池级 checker 一条都不红"
+    );
+    assert_eq!(
+        verdict_of(&image, HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES),
+        InvariantVerdict::Holds,
+        "I-9.27 在头 ID 65549 的树表上真被评估过且成立"
+    );
+    assert!(
+        matches!(
+            verdict_of(
+                &image,
+                FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK
+            ),
+            InvariantVerdict::NotApplicable(_)
+        ),
+        "根环里每一版的 extent 树都不是 65536：没有哪一版是从 mkfs 水位第一次建树的那一版，I-5.5 一版都不判"
+    );
+}
+
+/// 新池新建文件（默认 768 MiB 环，U = 50176）：那一版是从 mkfs 水位第一次建树的那一版，数据单元落在 U + 4 = 50180。
+#[test]
+fn a_new_pool_file_on_the_default_journal_ring_holds_the_first_data_unit_placement() {
+    let (image, unit_area_start_slot, _first_file) = new_pool_file(&parameters());
+    assert_eq!(unit_area_start_slot, 50176, "默认环长的单元区起始槽号");
+    assert_the_first_data_unit_placement_holds_and_nothing_reddens(
+        &image,
+        "默认环长的新池新建文件",
+    );
+}
+
+/// 新池新建文件（128 MiB 环）：U 按环长现算（不是 50176），t1 仍是 U + 4。
+#[test]
+fn a_new_pool_file_on_a_128_mebibyte_journal_ring_holds_the_first_data_unit_placement() {
+    let mut make_filesystem_parameters = parameters();
+    make_filesystem_parameters.geometry.journal_ring_bytes = JOURNAL_RING_BYTES_OF_128_MEBIBYTES;
+    let (image, unit_area_start_slot, first_file) = new_pool_file(&make_filesystem_parameters);
+    assert_ne!(
+        unit_area_start_slot, 50176,
+        "128 MiB 环的单元区起始槽号不是默认环的"
+    );
+    assert_the_first_data_unit_placement_holds_and_nothing_reddens(
+        &image,
+        "128 MiB 环的新池新建文件",
+    );
+    assert_eq!(
+        first_file.data_pointers[0].locations[0].slot.0,
+        unit_area_start_slot + FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START,
+        "实现把数据单元放在 U + 4"
+    );
+}
+
+/// 只做过 mkfs 的池可写挂载两次（实例 1 只暖机，实例 2 写行、暖机），实例 2 写第一个文件（txg 5）：
+/// 树 ID 仍从 mkfs 水位 65536 起发、树表每条诞生 txg 都是 5 ⇒ 这一版被认出（射程「实例 1、2、3 从 mkfs 水位写的第一个文件」），落点照判、成立。
+#[test]
+fn the_first_file_of_an_instance_after_a_warm_up_only_mount_is_recognized_and_holds_the_first_data_unit_placement(
+) {
+    let stream = SharedStream::retaining_contents();
+    let mut devices = memory_devices(&stream);
+    make_filesystem(&parameters(), &mut devices).expect("mkfs");
+    drop(mount_writable(&parameters(), &mut devices).expect("实例 1：只暖机"));
+    let second_instance = mount_writable(&parameters(), &mut devices).expect("实例 2");
+    let PoolVersion::WithoutFile(version_of_the_second_instance) = &second_instance.current else {
+        panic!("只暖机过的池上现行那一版树表 0 条")
+    };
+    let mut allocator_of_the_second_instance = second_instance.allocator.clone();
+    let first_file = {
+        let publish_parameters = parameters();
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        publish_first_file(
+            &mut writer,
+            &mut allocator_of_the_second_instance,
+            &version_of_the_second_instance.root,
+            FirstFile {
+                content: &file_content(),
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
+            },
+            second_instance.output.instance,
+            &version_of_the_second_instance.record_bytes,
+        )
+        .expect("实例 2 发第一个文件版本")
+    };
+    assert_eq!(
+        (
+            first_file.root.instance,
+            first_file.root.checkpoint_txg,
+            first_file.tree_identifiers.highest().0
+        ),
+        (InstanceGeneration(2), CheckpointTxg(5), 65547),
+        "实例 2 的第一个文件版本 txg 5，树从 mkfs 水位发 65536..65547"
+    );
+    assert_the_first_data_unit_placement_holds_and_nothing_reddens(
+        &memory_pool_of_sparse_devices(&devices),
+        "只暖机过的实例 2 写第一个文件",
+    );
+}
+
+/// 新池新建文件之后接着覆盖写，直到第一个文件版本（txg 3）与两代暖机根都被新根盖掉：根环里只剩覆盖写那几版，
+/// 树表条目的诞生 txg 都是 3、不等于这一版的 txg（认法 ② 不成立），一版都不判——I-5.5 报不适用，池级 checker 一条都不红。
+#[test]
+fn once_the_first_file_version_leaves_the_root_ring_the_overwrite_versions_are_not_judged_for_the_first_data_unit_placement(
+) {
+    let stream = SharedStream::retaining_contents();
+    let mut devices = memory_devices(&stream);
+    let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
+    let mut allocator = allocator_after_make_filesystem(&parameters(), &devices, &genesis);
+    let publish_parameters = parameters();
+    let mut writer = PoolWriter::new(&publish_parameters, &mut devices);
+    let instance = acquire_instance(&mut writer).expect("取号");
+    let warmed_up = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
+    let mut previous = publish_first_file(
+        &mut writer,
+        &mut allocator,
+        warmed_up.roots.last().expect("暖机两代根"),
+        FirstFile {
+            content: &file_content(),
+            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
+        },
+        instance,
+        &warmed_up.last_record_bytes,
+    )
+    .expect("新池新建文件");
+    let first_file_txg = previous.root.checkpoint_txg;
+    while previous.root.checkpoint_txg.0 < first_file_txg.0 + ROOT_RING_SLOT_COUNT {
+        let content_bytes = 1000 + usize::try_from(previous.root.checkpoint_txg.0).expect("小");
+        previous = publish_overwrite(
+            &mut writer,
+            &mut allocator,
+            &previous,
+            FirstFile {
+                content: &file_content()[..content_bytes],
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
+            },
+            instance,
+        )
+        .expect("覆盖写");
+    }
+    drop(writer);
+    let image = memory_pool_of_sparse_devices(&devices);
+    assert!(
+        matches!(
+            verdict_of(
+                &image,
+                FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK
+            ),
+            InvariantVerdict::NotApplicable(_)
+        ),
+        "根环里只剩覆盖写那几版（诞生 txg 3 ≠ 这一版的 txg）：I-5.5 一版都不判"
+    );
+    assert_eq!(
+        violations_of(&image),
+        Vec::new(),
+        "覆盖写一圈之后的镜像：池级 checker 一条都不红"
+    );
+}
diff --git a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
index 66a38b7..d7ba656 100644
--- a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
+++ b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
@@ -571,7 +571,11 @@ fn region_device(txg: u64) -> DeviceIdentity {
 /// I-3.5（引用区间的精确性） 在这里报不适用是因为回退候选集里相邻两版之间一个新出现的单元都没有：第一次可写挂载不写行，
 /// 写行那次发布（txg 1）与暖机（txg 2）都是零单元发布，树表、实例表照抄 mkfs 那一版、映射根全零，后一版引用的单元前一版都引用过，
 /// 出现规则一个单元都判不到（mkfs 之后那一格的理由不同，见 `NOT_APPLICABLE_RIGHT_AFTER_MKFS`）。
-const NOT_APPLICABLE_WITHOUT_FILE: [&str; 28] = [
+/// I-9.27（树表条目的头 ID 取值） 与 I-5.5（从 mkfs 水位建树那一版的 t1 落点） 三张没有文件的表都有：每条根的树表都是 0 条，
+/// 头 ID 一格都没有，也没有哪一版是从 mkfs 水位第一次建树的那一版（第一个文件版本之后两条都真被评估过且成立）。
+const NOT_APPLICABLE_WITHOUT_FILE: [&str; 30] = [
+    "I-9.27",
+    "I-5.5",
     "I-1.10",
     "I-3.1",
     "I-3.9",
@@ -607,7 +611,9 @@ const NOT_APPLICABLE_WITHOUT_FILE: [&str; 28] = [
 /// 可写挂载之后环里有记录（写行 / 暖机的空发布），它们就真被评估过了——所以这两条只在 mkfs 之后那一格。
 /// I-3.5（引用区间的精确性） 两张表都有，这一格的理由是回退候选集只剩 mkfs 的第 0 代根一版（环里没有记录、根环里只有它），
 /// 凑不出相邻的一对。
-const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 32] = [
+const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 34] = [
+    "I-9.27",
+    "I-5.5",
     "I-1.10",
     "I-3.1",
     "I-3.9",
@@ -668,7 +674,9 @@ const NOT_APPLICABLE_WITH_ONE_FILE_VERSION_OF_THE_ONLY_INSTANCE: [&str; 6] =
 /// I-3.9 判它的释放代；I-5.4 判这棵树里的记录两两不相交——两条都真被评估过且成立。
 /// I-3.5（引用区间的精确性） 也不在这张表里：写行那次发布（实例 2、txg 3）把实例表 COW 到新落点、写了自己那棵分配记录树，
 /// 这几个单元前一版（实例 1 的暖机根，txg 2）不引用、诞生代号 3 大于 2，出现规则真判到了它们且成立。
-const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 21] = [
+const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 23] = [
+    "I-9.27",
+    "I-5.5",
     "I-3.1",
     "I-3.11",
     "I-5.2",
````

## 17-1 `mutations-append.tsv`

````
C673 ⑤ I-9.27 判别力（攻方 M1）：发布写树表时非 0 的头 ID 一律写字面 65537，树 ID 从根环水位发的那一版上头 ID 与 inode 树 65549 对不上	crates/singlefs-core/src/transaction.rs	            birth_txg: plan.tree_birth_txg,\n            head_identifier,\n        };	            birth_txg: plan.tree_birth_txg,\n            head_identifier: if head_identifier == 0 { 0 } else { singlefs_format::TREE_IDENTIFIER_INODE },\n        };	-p singlefs-harness --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement --test checker_known_bad_images	a_first_file_version_issuing_its_trees_from_the_ring_watermark_after_a_recovery_keeps_the_head_identifier_relation
C673 ⑥ I-5.5 判别力（实审 A2c 第 15 条那一形）：用户数据落点不从单元区起点起扫，新池新建文件的数据单元不在 U + 4（默认环）	crates/singlefs-core/src/allocator.rs	        let mut candidate = self.unit_area_start.slot().0;\n        let end = self.unit_area_end_slot();	        let mut candidate = self.unit_area_start.slot().0 + CLUSTER_SEGMENT_SLOTS;\n        let end = self.unit_area_end_slot();	-p singlefs-harness --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement	a_new_pool_file_on_the_default_journal_ring_holds_the_first_data_unit_placement
C673 ⑥ I-5.5 判别力（实审 A2c 第 15 条那一形）：用户数据落点不从单元区起点起扫，128 MiB 环的新池新建文件数据单元不在 U + 4	crates/singlefs-core/src/allocator.rs	        let mut candidate = self.unit_area_start.slot().0;\n        let end = self.unit_area_end_slot();	        let mut candidate = self.unit_area_start.slot().0 + CLUSTER_SEGMENT_SLOTS;\n        let end = self.unit_area_end_slot();	-p singlefs-harness --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement	a_new_pool_file_on_a_128_mebibyte_journal_ring_holds_the_first_data_unit_placement
C673 ⑥ I-5.5 判别力（实审 A2c 第 15 条那一形）：用户数据落点不从单元区起点起扫，只暖机过的实例 2 写第一个文件那一版数据单元不在 U + 4	crates/singlefs-core/src/allocator.rs	        let mut candidate = self.unit_area_start.slot().0;\n        let end = self.unit_area_end_slot();	        let mut candidate = self.unit_area_start.slot().0 + CLUSTER_SEGMENT_SLOTS;\n        let end = self.unit_area_end_slot();	-p singlefs-harness --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement	the_first_file_of_an_instance_after_a_warm_up_only_mount_is_recognized_and_holds_the_first_data_unit_placement
C673 ⑥ I-5.5 判法：认法 ② 不看诞生 txg（覆盖写那几版也被当成从 mkfs 水位第一次建树的那一版来判）	crates/singlefs-checker/src/walk.rs	                read_u64(entry, TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET) == version.checkpoint_txg	                read_u64(entry, TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET) == version.checkpoint_txg || true	-p singlefs-harness --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement	once_the_first_file_version_leaves_the_root_ring_the_overwrite_versions_are_not_judged_for_the_first_data_unit_placement
C673 ⑥ I-5.5 判法：落点判成恒成立（数据单元搬走的坏镜像不红）	crates/singlefs-checker/src/walk.rs	                    judgements.judge(invariant, smallest_slot == expected_slot, || {	                    judgements.judge(invariant, true, || {	-p singlefs-harness --test checker_known_bad_images -- the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target	the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target
C673 ⑤ I-9.27 判法：extent 树、deleted_inodes、墓碑树条目的头 ID 不比 inode 树的树 ID（头 ID 写 0 的坏镜像不红）	crates/singlefs-checker/src/walk.rs	                inode_tree_identifiers.contains(&head_identifier)	                true	-p singlefs-harness --test checker_known_bad_images -- each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant
C673 ⑤ I-9.27 判法：extent 树、deleted_inodes、墓碑树条目的头 ID 拿条目自己的树 ID 比（树 ID 从根环水位发的合法镜像上误红）	crates/singlefs-checker/src/walk.rs	                inode_tree_identifiers.contains(&head_identifier)	                head_identifier == tree	-p singlefs-harness --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement	a_first_file_version_issuing_its_trees_from_the_ring_watermark_after_a_recovery_keeps_the_head_identifier_relation
````

## 17-4 `crates/singlefs-harness/tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs` 第 129 行起（改后原文）

````rust
fn swap_the_kinds_of_the_tombstone_and_lineage_entries(tree_table: &mut [u8]) {
    assert_eq!(
        kind_of_entry(tree_table, INODE_ENTRY_INDEX),
        TREE_KIND_INODE
    );
    assert_eq!(
        kind_of_entry(tree_table, TOMBSTONE_ENTRY_INDEX),
        TREE_KIND_TOMBSTONE
    );
    assert_eq!(
        kind_of_entry(tree_table, LINEAGE_ENTRY_INDEX),
        TREE_KIND_LINEAGE
    );
    let inode_tree_identifier = tree_identifier_of_entry(tree_table, INODE_ENTRY_INDEX);
    assert_eq!(
        head_identifier_of_entry(tree_table, TOMBSTONE_ENTRY_INDEX),
        inode_tree_identifier,
        "互换之前墓碑树的头 ID 等于 inode 树的树 ID"
    );
    assert_eq!(
        head_identifier_of_entry(tree_table, LINEAGE_ENTRY_INDEX),
        0,
        "互换之前谱系树的头 ID 是 0"
    );
    // 每条条目的头 ID 跟着它的新种类：种类 11（谱系树）写 0，种类 10（墓碑树）写 inode 树的树 ID。
    for (index, kind, head_identifier) in [
        (TOMBSTONE_ENTRY_INDEX, TREE_KIND_LINEAGE, 0),
        (
            LINEAGE_ENTRY_INDEX,
            TREE_KIND_TOMBSTONE,
            inode_tree_identifier,
        ),
    ] {
        let entry_start = entry_range_in_node(tree_table, index).start;
        let kind_start = entry_start + TREE_TABLE_ENTRY_KIND_OFFSET;
        tree_table[kind_start..kind_start + 2].copy_from_slice(&kind.to_le_bytes());
        let head_identifier_start = entry_start + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET;
        tree_table[head_identifier_start..head_identifier_start + 8]
            .copy_from_slice(&head_identifier.to_le_bytes());
    }
    assert_eq!(
        head_identifier_of_entry(tree_table, TOMBSTONE_ENTRY_INDEX),
        0,
        "互换之后第 8 条是种类 11（谱系树），头 ID 合它的新种类：0"
    );
    assert_eq!(
        head_identifier_of_entry(tree_table, LINEAGE_ENTRY_INDEX),
        inode_tree_identifier,
        "互换之后第 9 条是种类 10（墓碑树），头 ID 合它的新种类：inode 树的树 ID"
    );
}

/// 探针乙：livelist 与稀疏旁表两条整条互换位置（200 字节连同树 ID、种类、根指针一起搬）。种类仍跟着自己的号，
/// 按发号次序 65541 < 65542 照旧成立；盘上次序变成 …、65542、65541、…。首末两条没动，树表头里的 key 区间照旧贴紧。
fn swap_the_places_of_the_livelist_and_sparse_side_table_entries(tree_table: &mut [u8]) {
    let livelist = entry_range_in_node(tree_table, LIVELIST_ENTRY_INDEX);
    let sparse_side_table = entry_range_in_node(tree_table, SPARSE_SIDE_TABLE_ENTRY_INDEX);
    let livelist_bytes = tree_table[livelist.clone()].to_vec();
    let sparse_side_table_bytes = tree_table[sparse_side_table.clone()].to_vec();
    tree_table[livelist].copy_from_slice(&sparse_side_table_bytes);
    tree_table[sparse_side_table].copy_from_slice(&livelist_bytes);
}

/// 写完第一个文件的干净镜像，以及那一版的记录（重建上一版要它顶着）。
fn clean_image_after_the_first_file(tag: &str) -> (MemoryPool, JournalRecord) {
    let pool = build_pool(tag);
    (pool.memory_pool(), pool.output.record.clone())
}

/// 改最新那条根的树表单元（两份），重封、把新的整单元校验和补进根记录里树表指针的两条位置条目，再把根写回它的槽，
/// 同 (实例, txg) 的 journal 记录的新根段照抄改过的根（I-8.10（记录新根段与同代根相同）：写者发布时两处带同一份树表指针）：
````
