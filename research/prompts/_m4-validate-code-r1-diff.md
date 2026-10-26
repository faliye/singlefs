# m4-validate-code-r1：代码轮要攻的产品代码改动全文（2026-10-22）

出处：事务员清单 `~/.local/share/singlefs/work/m4-code-round-inventory/report.md`（sha256 9c65bb44…）。三份 diff 都以 HEAD 841b3beab 为后像，前像是这一段动手之前那一版；逐 hunk 的来源在那份报告 4.3 节。

## code-round.diff（sha256 3d8ffc766b24194d…）

```diff
diff --git a/crates/singlefs-core/src/admission.rs b/crates/singlefs-core/src/admission.rs
index c852de6..5aafb26 100644
--- a/crates/singlefs-core/src/admission.rs
+++ b/crates/singlefs-core/src/admission.rs
@@ -437,7 +437,8 @@ pub enum RingWindowOfTheReading {
 }
 
 impl RingWindowOfTheReading {
-    /// 可写挂载那一判的读法：要接的那一版没有树 ⇒ 还没起计；它的 `deleted_inodes` 不空 ⇒ 算作可动用；否则扣。
+    /// 可写挂载那一判的读法：要接的那一版没有树 ⇒ 还没起计；它的承诺类队列不空（`deleted_inodes`、销毁中的快照、截断条目，
+    /// `transaction::TransactionOutput::queue_of_committed_intents_is_empty`，判决 m4-trunc-r1 K1）⇒ 算作可动用；否则扣。
     #[must_use]
     pub fn of_a_writable_mount(
         version_to_build_on: Option<&TransactionOutput>,
@@ -445,7 +446,7 @@ impl RingWindowOfTheReading {
     ) -> Self {
         match version_to_build_on {
             None => RingWindowOfTheReading::NotCountedBeforeTheFirstFileVersion,
-            Some(version) if !version.deleted_inodes.is_empty() => {
+            Some(version) if !version.queue_of_committed_intents_is_empty() => {
                 RingWindowOfTheReading::UsableByTheWritableMountWhileTheDeletionQueueIsNotEmpty
             }
             Some(version) => RingWindowOfTheReading::DeductedOnEachDevice {
diff --git a/crates/singlefs-core/src/extent_tree.rs b/crates/singlefs-core/src/extent_tree.rs
index 08cbd83..246aebe 100644
--- a/crates/singlefs-core/src/extent_tree.rs
+++ b/crates/singlefs-core/src/extent_tree.rs
@@ -3,12 +3,15 @@
 //! - **上段**按 inode 号的位置寻址：叶 j 罩 inode 号 `[j × U, (j + 1) × U)`，U = [`EXTENT_TREE_UPPER_LEAF_INODES`]；层级 L 上第 j 个节点罩
 //!   `[j × U × F^L, (j + 1) × U × F^L)`，F = [`EXTENT_TREE_INTERNAL_FANOUT`]。上段的根是它最高那一层的第 0 个节点（罩从 inode 0 起的一段），
 //!   层级是罩得住有条目的最大 inode 号的最低一层。上段叶条目（[`ExtentUpperLeafEntry`]，113 字节）带一个标签字节区分
-//!   「没有单元（0）/ 下段根指针（1）/ 内联数据指针（2）」。
+//!   「没有单元（0）/ 下段根指针（1）/ 内联数据指针（2）/ 下段根指针、下段最后一个单元之后到 size 是洞（3）/
+//!   内联数据指针、单元 0 之后到 size 是洞（4）」（3、4 两个值：判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5，
+//!   用户 2026-10-16 定「加标签声明洞」）。
 //! - **下段**一个文件一棵，按数据单元号的位置寻址：叶罩 [`EXTENT_TREE_LOWER_LEAF_DATA_UNITS`] 个单元，层级 L 上一个节点罩 144 × F^L 个单元，
 //!   根是罩得住这个文件全部单元的最低一层的第 0 个节点。叶的条目是 extent 叶记录 112（key (0, inode, 文件字节偏移) + 数据指针 88），
 //!   内部条目是「孩子那一段的起点 key + 子指针」110。洞就是缺席：没有单元的一段不写节点，父节点那一格留空。
-//! - **只有一个数据单元的文件不建下段**：上段叶条目里直接放那个数据指针（标签 2）。写路径只写标签 1 与 2（今天每个文件版本从偏移 0 顺序写、
-//!   长度 0 的内容也写一个声明长度 0 的数据单元，一个单元都没有的文件写不出来）；标签 0 由读者与 checker 认（「这个 inode 没有单元」）。
+//! - **只有一个数据单元的文件不建下段**：上段叶条目里直接放那个数据指针（标签 2，后面有洞时标签 4）。写路径写标签 1 到 4（每个文件版本
+//!   从偏移 0 顺序写、长度 0 的内容也写一个声明长度 0 的数据单元；变长不写新单元，最后一个单元之后到 size 是洞，K5）；标签 0 由读者与
+//!   checker 认（「这个 inode 没有单元」，配 size > 0 是整个文件都是洞）。只有标签 3、4、0 许单元数少于 ⌈size ÷ 净荷⌉.max(1)。
 //!
 //! 节点头的 key 区间写这个节点按位置规定罩的那一段（D18（块里携带什么信息） 已定项 2 对按位置寻址的树那一句），key 宽恒是 24
 //! （上段与下段同一棵树、同一个 key 宽，头宽 163）：上段节点罩 `[(0, 首个 inode, 0), (0, 末个 inode, u64 最大)]`，
@@ -46,6 +49,10 @@ pub const EXTENT_UPPER_LEAF_ENTRY_TAG_NO_DATA_UNIT: u8 = 0;
 pub const EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT: u8 = 1;
 /// 上段叶条目的标签：载荷是这个文件唯一那个数据单元的数据指针（内联）。
 pub const EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT: u8 = 2;
+/// 上段叶条目的标签：载荷是这个文件下段根的节点指针，下段最后一个单元之后到 size 是洞（判决 m4-trunc-r1 K5）。
+pub const EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT_FOLLOWED_BY_A_HOLE: u8 = 3;
+/// 上段叶条目的标签：载荷是单元 0 的数据指针（内联），单元 0 之后到 size 是洞（判决 m4-trunc-r1 K5）。
+pub const EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT_FOLLOWED_BY_A_HOLE: u8 = 4;
 
 fn key_width() -> usize {
     usize::try_from(EXTENT_KEY_BYTES).expect("24")
@@ -312,12 +319,104 @@ pub fn lower_children_of_a_file_without_holes(
         .collect()
 }
 
-/// 上段叶条目的载荷：标签 0 / 1 / 2 三种。
+/// 一个文件最后一个存在的数据单元之后、到 size 之前是不是洞（判决 m4-trunc-r1 K5：洞只在尾部，标签声明它）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum TailOfAFile {
+    /// 标签 1、2：单元数不少于 ⌈size ÷ 净荷⌉.max(1)（截断中间态里可以多：size 之外还挂着的单元）。
+    NoHoleAfterTheLastDataUnit,
+    /// 标签 3、4、0：最后一个存在的单元之后到 size 是洞，单元数少于 ⌈size ÷ 净荷⌉.max(1)（标签 0：一个单元都没有，整个文件是洞）。
+    HoleAfterTheLastDataUnit,
+}
+
+impl TailOfAFile {
+    /// 写者按这一版的单元数与 size 取标签：单元数少于 ⌈size ÷ 净荷⌉.max(1) 就是尾部有洞。
+    #[must_use]
+    pub fn of_data_units_and_size(data_units: u64, size_in_bytes: u64) -> Self {
+        if data_units < size_in_bytes.div_ceil(payload_capacity_in_bytes()).max(1) {
+            TailOfAFile::HoleAfterTheLastDataUnit
+        } else {
+            TailOfAFile::NoHoleAfterTheLastDataUnit
+        }
+    }
+}
+
+/// 上段叶条目的载荷：标签 0 到 4 五种。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum ExtentUpperLeafTarget {
     NoDataUnit,
     LowerSegmentRoot(NodePointer),
     InlineDataUnit(DataPointer),
+    LowerSegmentRootFollowedByAHole(NodePointer),
+    InlineDataUnitFollowedByAHole(DataPointer),
+}
+
+impl ExtentUpperLeafTarget {
+    /// 指着下段根的那两种（标签 1、3）的下段根指针。
+    #[must_use]
+    pub const fn lower_segment_root(self) -> Option<NodePointer> {
+        match self {
+            ExtentUpperLeafTarget::LowerSegmentRoot(pointer)
+            | ExtentUpperLeafTarget::LowerSegmentRootFollowedByAHole(pointer) => Some(pointer),
+            ExtentUpperLeafTarget::NoDataUnit
+            | ExtentUpperLeafTarget::InlineDataUnit(_)
+            | ExtentUpperLeafTarget::InlineDataUnitFollowedByAHole(_) => None,
+        }
+    }
+
+    /// 内联那两种（标签 2、4）的数据指针。
+    #[must_use]
+    pub const fn inline_data_unit(self) -> Option<DataPointer> {
+        match self {
+            ExtentUpperLeafTarget::InlineDataUnit(pointer)
+            | ExtentUpperLeafTarget::InlineDataUnitFollowedByAHole(pointer) => Some(pointer),
+            ExtentUpperLeafTarget::NoDataUnit
+            | ExtentUpperLeafTarget::LowerSegmentRoot(_)
+            | ExtentUpperLeafTarget::LowerSegmentRootFollowedByAHole(_) => None,
+        }
+    }
+
+    /// 同一个下段根或内联单元、换成声明 `tail` 那一种标签（1 ↔ 3、2 ↔ 4）；标签 0 不变。写者按这一版的单元数与 size 取尾部
+    /// （[`TailOfAFile::of_data_units_and_size`]）之后经它取标签，读者解标签 3、4 也经它。
+    #[must_use]
+    pub const fn with_the_tail(self, tail: TailOfAFile) -> Self {
+        match (self, tail) {
+            (
+                ExtentUpperLeafTarget::LowerSegmentRoot(pointer)
+                | ExtentUpperLeafTarget::LowerSegmentRootFollowedByAHole(pointer),
+                TailOfAFile::NoHoleAfterTheLastDataUnit,
+            ) => ExtentUpperLeafTarget::LowerSegmentRoot(pointer),
+            (
+                ExtentUpperLeafTarget::LowerSegmentRoot(pointer)
+                | ExtentUpperLeafTarget::LowerSegmentRootFollowedByAHole(pointer),
+                TailOfAFile::HoleAfterTheLastDataUnit,
+            ) => ExtentUpperLeafTarget::LowerSegmentRootFollowedByAHole(pointer),
+            (
+                ExtentUpperLeafTarget::InlineDataUnit(pointer)
+                | ExtentUpperLeafTarget::InlineDataUnitFollowedByAHole(pointer),
+                TailOfAFile::NoHoleAfterTheLastDataUnit,
+            ) => ExtentUpperLeafTarget::InlineDataUnit(pointer),
+            (
+                ExtentUpperLeafTarget::InlineDataUnit(pointer)
+                | ExtentUpperLeafTarget::InlineDataUnitFollowedByAHole(pointer),
+                TailOfAFile::HoleAfterTheLastDataUnit,
+            ) => ExtentUpperLeafTarget::InlineDataUnitFollowedByAHole(pointer),
+            (ExtentUpperLeafTarget::NoDataUnit, _) => ExtentUpperLeafTarget::NoDataUnit,
+        }
+    }
+
+    /// 这个标签声明的尾部：3、4、0 有洞，1、2 没有。
+    #[must_use]
+    pub const fn tail(self) -> TailOfAFile {
+        match self {
+            ExtentUpperLeafTarget::LowerSegmentRoot(_)
+            | ExtentUpperLeafTarget::InlineDataUnit(_) => TailOfAFile::NoHoleAfterTheLastDataUnit,
+            ExtentUpperLeafTarget::NoDataUnit
+            | ExtentUpperLeafTarget::LowerSegmentRootFollowedByAHole(_)
+            | ExtentUpperLeafTarget::InlineDataUnitFollowedByAHole(_) => {
+                TailOfAFile::HoleAfterTheLastDataUnit
+            }
+        }
+    }
 }
 
 /// 上段叶条目 113：key 24（0, inode, 0）+ 标签 1 + 载荷 88（[`EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES`]）。
@@ -331,7 +430,7 @@ pub struct ExtentUpperLeafEntry {
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum ExtentUpperLeafEntryMalformed {
     NarrowerThanItsFieldTable,
-    /// 标签不是 0 / 1 / 2。
+    /// 标签不是 0 到 4。
     UnrecognizedTag(u8),
     /// key 的 locality 段或 offset 段不是 0，或载荷里该是零的字节不是零。
     NonZeroBytesWhereTheFieldTableSaysZero,
@@ -394,6 +493,15 @@ impl ExtentUpperLeafEntry {
                 writer.put_u8(EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT);
                 pointer.write_to(&mut writer);
             }
+            ExtentUpperLeafTarget::LowerSegmentRootFollowedByAHole(pointer) => {
+                writer.put_u8(EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT_FOLLOWED_BY_A_HOLE);
+                pointer.write_to(&mut writer);
+                writer.skip(usize::try_from(DATA_POINTER_BYTES - NODE_POINTER_BYTES).expect("2"));
+            }
+            ExtentUpperLeafTarget::InlineDataUnitFollowedByAHole(pointer) => {
+                writer.put_u8(EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT_FOLLOWED_BY_A_HOLE);
+                pointer.write_to(&mut writer);
+            }
         }
         writer.assert_position(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES, "extent 上段叶条目");
         writer.into_bytes()
@@ -443,6 +551,23 @@ impl ExtentUpperLeafEntry {
                 )
                 .map_err(ExtentUpperLeafEntryMalformed::PointerHeadFieldOutsideTheFirstVersion)?,
             ),
+            // 标签 3、4 的载荷与标签 1、2 同一个字段表（判决 m4-trunc-r1 K5 只加「之后到 size 是洞」这一句）：换成对应的没有洞的标签照同一段解
+            // （补齐与指针头部的判只有一份），再换回带洞的那一种。
+            EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT_FOLLOWED_BY_A_HOLE
+            | EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT_FOLLOWED_BY_A_HOLE => {
+                let mut the_same_entry_without_the_hole = bytes
+                    [..usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113")]
+                    .to_vec();
+                the_same_entry_without_the_hole[key_width()] =
+                    if tag == EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT_FOLLOWED_BY_A_HOLE {
+                        EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT
+                    } else {
+                        EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT
+                    };
+                Self::parse(&the_same_entry_without_the_hole)?
+                    .target
+                    .with_the_tail(TailOfAFile::HoleAfterTheLastDataUnit)
+            }
             unrecognized => {
                 return Err(ExtentUpperLeafEntryMalformed::UnrecognizedTag(unrecognized))
             }
@@ -999,7 +1124,7 @@ fn upper_leaf_entries(
                 ExtentUpperLeafEntryMalformed::UnrecognizedTag(_) => {
                     RecoveryFailure::InvariantViolated {
                         invariant: "E142 走读同款",
-                        detail: "extent 上段叶条目的标签不是 0 / 1 / 2",
+                        detail: "extent 上段叶条目的标签不是 0 到 4",
                     }
                 }
                 ExtentUpperLeafEntryMalformed::NonZeroBytesWhereTheFieldTableSaysZero => {
@@ -1197,12 +1322,12 @@ pub fn find_upper_leaf_entry(
     }
 }
 
-/// 一个文件在 extent 树里的样子：没有单元、内联的那一个单元、或下段。
+/// 一个文件在 extent 树里的样子：没有单元、内联的那一个单元、或下段；后两种带标签声明的尾部（K5）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum ExtentsOfAFileReadFromDisk {
     NoDataUnit,
-    Inline(DataPointer),
-    LowerSegment(ExtentLowerSegmentReadFromDisk),
+    Inline(DataPointer, TailOfAFile),
+    LowerSegment(ExtentLowerSegmentReadFromDisk, TailOfAFile),
 }
 
 impl ExtentsOfAFileReadFromDisk {
@@ -1211,8 +1336,18 @@ impl ExtentsOfAFileReadFromDisk {
     pub fn data_pointers(&self) -> Vec<(u64, DataPointer)> {
         match self {
             ExtentsOfAFileReadFromDisk::NoDataUnit => Vec::new(),
-            ExtentsOfAFileReadFromDisk::Inline(pointer) => vec![(0, *pointer)],
-            ExtentsOfAFileReadFromDisk::LowerSegment(segment) => segment.data_pointers.clone(),
+            ExtentsOfAFileReadFromDisk::Inline(pointer, _) => vec![(0, *pointer)],
+            ExtentsOfAFileReadFromDisk::LowerSegment(segment, _) => segment.data_pointers.clone(),
+        }
+    }
+
+    /// 标签声明的尾部：标签 0 整个文件是洞。
+    #[must_use]
+    pub const fn tail(&self) -> TailOfAFile {
+        match self {
+            ExtentsOfAFileReadFromDisk::NoDataUnit => TailOfAFile::HoleAfterTheLastDataUnit,
+            ExtentsOfAFileReadFromDisk::Inline(_, tail)
+            | ExtentsOfAFileReadFromDisk::LowerSegment(_, tail) => *tail,
         }
     }
 }
@@ -1238,20 +1373,23 @@ pub fn read_extent_tree(
     let upper = read_upper_segment(reading, root_pointer, read_node, &mut units_seen)?;
     let mut files = Vec::with_capacity(upper.entries.len());
     for entry in &upper.entries {
-        let extents = match entry.target {
-            ExtentUpperLeafTarget::NoDataUnit => ExtentsOfAFileReadFromDisk::NoDataUnit,
-            ExtentUpperLeafTarget::InlineDataUnit(pointer) => {
-                ExtentsOfAFileReadFromDisk::Inline(pointer)
-            }
-            ExtentUpperLeafTarget::LowerSegmentRoot(lower_root) => {
-                ExtentsOfAFileReadFromDisk::LowerSegment(read_lower_segment(
+        let tail = entry.target.tail();
+        let extents = match (
+            entry.target.inline_data_unit(),
+            entry.target.lower_segment_root(),
+        ) {
+            (Some(pointer), _) => ExtentsOfAFileReadFromDisk::Inline(pointer, tail),
+            (None, Some(lower_root)) => ExtentsOfAFileReadFromDisk::LowerSegment(
+                read_lower_segment(
                     reading,
                     entry.inode,
                     &lower_root,
                     read_node,
                     &mut units_seen,
-                )?)
-            }
+                )?,
+                tail,
+            ),
+            (None, None) => ExtentsOfAFileReadFromDisk::NoDataUnit,
         };
         files.push((entry.inode, extents));
     }
@@ -1311,7 +1449,8 @@ mod tests {
         );
     }
 
-    /// 上段叶条目 113 字节来回：三种标签各解回自己；不认识的标签、标签 1 的补齐非零都解不开。
+    /// 上段叶条目 113 字节来回：带洞的下段根那一种（标签 3）也解回自己；不认识的标签（5 起，3、4 是 K5 加的）、
+    /// 标签 1 与标签 3 的补齐非零都解不开。
     #[test]
     fn upper_leaf_entries_round_trip_and_refuse_an_unknown_tag() {
         let no_unit = ExtentUpperLeafEntry {
@@ -1347,22 +1486,59 @@ mod tests {
                 },
             }),
         };
-        for entry in [no_unit, lower, inline] {
+        let lower_followed_by_a_hole = ExtentUpperLeafEntry {
+            inode: 10,
+            target: ExtentUpperLeafTarget::LowerSegmentRootFollowedByAHole(
+                NodePointer::empty_root(),
+            ),
+        };
+        for entry in [no_unit, lower, inline, lower_followed_by_a_hole] {
             let bytes = entry.to_bytes();
             assert_eq!(bytes.len(), 113);
             assert_eq!(ExtentUpperLeafEntry::parse(&bytes), Ok(entry));
         }
+        assert_eq!(
+            lower_followed_by_a_hole.to_bytes()[24],
+            3,
+            "标签字节在偏移 24"
+        );
         let mut unknown = no_unit.to_bytes();
-        unknown[24] = 3;
+        unknown[24] = 5;
         assert_eq!(
             ExtentUpperLeafEntry::parse(&unknown),
-            Err(ExtentUpperLeafEntryMalformed::UnrecognizedTag(3))
+            Err(ExtentUpperLeafEntryMalformed::UnrecognizedTag(5))
+        );
+        for entry in [lower, lower_followed_by_a_hole] {
+            let mut padding = entry.to_bytes();
+            padding[112] = 1;
+            assert_eq!(
+                ExtentUpperLeafEntry::parse(&padding),
+                Err(ExtentUpperLeafEntryMalformed::NonZeroBytesWhereTheFieldTableSaysZero)
+            );
+        }
+    }
+
+    /// 写者取标签的那一判：单元数少于 ⌈size ÷ 净荷⌉.max(1) 是尾部有洞；相等或更多（截断中间态）没有洞。
+    #[test]
+    fn the_tail_has_a_hole_exactly_when_the_data_units_are_fewer_than_the_size_needs() {
+        let payload = payload_capacity_in_bytes();
+        assert_eq!(
+            TailOfAFile::of_data_units_and_size(1, 0),
+            TailOfAFile::NoHoleAfterTheLastDataUnit,
+            "size 0 一个单元"
+        );
+        assert_eq!(
+            TailOfAFile::of_data_units_and_size(1, payload),
+            TailOfAFile::NoHoleAfterTheLastDataUnit
+        );
+        assert_eq!(
+            TailOfAFile::of_data_units_and_size(1, payload + 1),
+            TailOfAFile::HoleAfterTheLastDataUnit
         );
-        let mut padding = lower.to_bytes();
-        padding[112] = 1;
         assert_eq!(
-            ExtentUpperLeafEntry::parse(&padding),
-            Err(ExtentUpperLeafEntryMalformed::NonZeroBytesWhereTheFieldTableSaysZero)
+            TailOfAFile::of_data_units_and_size(3, payload + 1),
+            TailOfAFile::NoHoleAfterTheLastDataUnit,
+            "截断中间态：size 之外还挂着单元，不是洞"
         );
     }
 
diff --git a/crates/singlefs-core/src/inode_tree.rs b/crates/singlefs-core/src/inode_tree.rs
index b6bf1ce..a04e209 100644
--- a/crates/singlefs-core/src/inode_tree.rs
+++ b/crates/singlefs-core/src/inode_tree.rs
@@ -590,6 +590,7 @@ mod tests {
             size,
             change_count,
             write_time_seconds: 1_788_000_000,
+            truncation_field: crate::records::InodeRecordTruncationField::NOT_WRITTEN,
         }
     }
 
diff --git a/crates/singlefs-core/src/lib.rs b/crates/singlefs-core/src/lib.rs
index ede253c..0d5005a 100644
--- a/crates/singlefs-core/src/lib.rs
+++ b/crates/singlefs-core/src/lib.rs
@@ -42,6 +42,7 @@ pub mod system_configuration;
 pub mod system_configuration_repair;
 pub mod tombstone_tree;
 pub mod transaction;
+pub mod truncate;
 pub mod unit;
 pub mod unlink;
 pub mod write_accounting;
diff --git a/crates/singlefs-core/src/make_filesystem.rs b/crates/singlefs-core/src/make_filesystem.rs
index fc1ad6e..42a9460 100644
--- a/crates/singlefs-core/src/make_filesystem.rs
+++ b/crates/singlefs-core/src/make_filesystem.rs
@@ -32,9 +32,10 @@ use crate::root_record::{RootRecord, UnmountMarker};
 use crate::root_ring::{region_length_in_bytes, region_start, ring_end, slot_offset, RootRingSlot};
 use crate::system_configuration::{
     journal_in_flight_record_limit, journal_in_flight_record_limit_fits_its_four_byte_field,
-    SystemConfiguration, SystemImmutableConfiguration, SystemImmutableSizes,
-    SystemMutableConfiguration, SystemRuntimeConfiguration, SystemRuntimeQuantities,
-    JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION, ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
+    InodeRecordTruncationFieldsOfThePool, SystemConfiguration, SystemImmutableConfiguration,
+    SystemImmutableSizes, SystemMutableConfiguration, SystemRuntimeConfiguration,
+    SystemRuntimeQuantities, JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
+    ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
 };
 use crate::unit::{
     build_index_node, build_packed_unit, PackedIdentity, WriteOrder, PACKED_TYPE_INSTANCE_TABLE,
@@ -638,6 +639,9 @@ pub fn make_filesystem<Device: BlockDevice>(
                 sizes: parameters.geometry,
                 journal_ring_start_slot: JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
                 root_ring_base_slot: ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
+                // mkfs 起就置上 incompat 位 2（用户 2026-10-17 定，判决 m4-trunc-r2 第六节）。
+                inode_record_truncation_fields:
+                    InodeRecordTruncationFieldsOfThePool::WRITTEN_BY_THIS_WRITER,
             },
             mutable: SystemMutableConfiguration,
             runtime: SystemRuntimeConfiguration,
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 7854a29..615ebad 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -34,6 +34,7 @@ use crate::logged_operations::{live_compaction_intent_of_the_entries, LoggedOper
 use crate::make_filesystem::MakeFilesystemParameters;
 use crate::mounted_session::{
     refusal_is_short_of_space, run_compaction_on, CompactionRun, CompactionRunReach,
+    DeviceKeepingWritesInMemory, DeviceReadOnceInThisMount, ReadRouteOfThisMount,
 };
 use crate::pointer::LocationEntry;
 use crate::pointer::{slot_shared_by_both_location_entries, LocationEntriesOnDifferentSlots};
@@ -59,8 +60,8 @@ use crate::recovery::{
     newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to,
     readable_roots, readable_roots_rereading_ring_slots_known_to_hold_a_root_once,
     readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once,
-    rebuild_version_in_the_unit_area_starting_at, replay_journal,
-    root_is_abandoned_by_the_instance_table, scan_journal,
+    rebuild_version_in_the_unit_area_starting_at, rebuild_version_judging_the_truncation_fields_as,
+    replay_journal, root_is_abandoned_by_the_instance_table, scan_journal,
     system_configuration_slots_of_one_device_read_once,
     system_configuration_witness_is_undecidable, tree_table_has_no_entries,
     user_visible_tree_root_pointers, verified_system_configuration_slots, BadRootRingSlotReading,
@@ -75,7 +76,7 @@ use crate::recovery::{
 };
 use crate::root_record::{RootRecord, UnmountMarker};
 use crate::root_ring::{target_for_publish, RootRingSlot};
-use crate::system_configuration::SystemImmutableSizes;
+use crate::system_configuration::{InodeRecordTruncationFieldsOfThePool, SystemImmutableSizes};
 use crate::transaction::{
     central_mapping_locations_in_the_version,
     copies_of_a_released_unit_failing_the_release_checksum_check,
@@ -1263,12 +1264,16 @@ fn instance_table_rewrite_of_the_row_publish(
 fn rebuild_previous_version<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
     unit_area_start: UnitAreaStart,
+    truncation_fields_of_the_pool: InodeRecordTruncationFieldsOfThePool,
     root: &RootRecord,
     record_standing_for_root: Option<crate::journal::JournalRecord>,
 ) -> Result<PreviousVersion, MountError> {
-    let rebuilt = rebuild_version_in_the_unit_area_starting_at(
+    // inode 记录的截断字段按择到的系统配置那一档判（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第六节）：不带 incompat 位 2 的池里
+    // 截断字段不是全零的记录判不可用，这一版建不出来。
+    let rebuilt = rebuild_version_judging_the_truncation_fields_as(
         devices,
         unit_area_start,
+        truncation_fields_of_the_pool,
         root,
         record_standing_for_root,
     )
@@ -5508,7 +5513,11 @@ fn rehearsed_continuation_refusal_is_returned_before_acquisition(refusal: &Publi
         | PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion { .. }
         | PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { .. }
         | PublishError::DeletedInodesQueueRefused(_)
-        | PublishError::BlockDevice(_) => false,
+        | PublishError::BlockDevice(_) | PublishError::SnapshotCreationRefused(_)
+        | PublishError::SnapshotDestructionRefused(_)
+        | PublishError::DeadlistEntryNotReleasable { .. }
+        | PublishError::EmptyClusterSegmentsBelowTheFloor(_)
+        | PublishError::TruncationRefused(_) => false,
     }
 }
 
@@ -8354,7 +8363,15 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
     // 退过去的那一版也建不出来，照报所选那一版的失败。之后的一切（F1 乙再判一次、取号之前逐盘核、写行、影子账与 C）都按退到的那一版。
     // 退到哪一版照 `F2_NEWEST`：[见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（判法在那个函数里）。
     let (chosen_root, effective_root, journal, selected_version_journal_position, previous) =
-        match rebuild_previous_version(devices, unit_area_start, &effective_root, own_record) {
+        match rebuild_previous_version(
+            devices,
+            unit_area_start,
+            system_configuration
+                .immutable
+                .inode_record_truncation_fields,
+            &effective_root,
+            own_record,
+        ) {
             Ok(previous) => (
                 chosen_root,
                 effective_root,
@@ -8384,6 +8401,9 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
                 match rebuild_previous_version(
                     devices,
                     unit_area_start,
+                    system_configuration
+                        .immutable
+                        .inode_record_truncation_fields,
                     &version_fallen_back_to,
                     record_standing_for_the_version_fallen_back_to,
                 ) {
@@ -9618,6 +9638,28 @@ mod non_empty_root_tests {
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
diff --git a/crates/singlefs-core/src/mounted_read.rs b/crates/singlefs-core/src/mounted_read.rs
index bed9308..4f5ae7a 100644
--- a/crates/singlefs-core/src/mounted_read.rs
+++ b/crates/singlefs-core/src/mounted_read.rs
@@ -52,12 +52,13 @@ use crate::recovery::{
     RecoveryFailure, TreeTableEntriesRefusal,
 };
 use crate::root_record::RootRecord;
+use crate::system_configuration::InodeRecordTruncationFieldsOfThePool;
 use crate::transaction::MultiLevelCodeTwoTree;
 use crate::unit::{
     data_unit_payload, data_unit_payload_capacity, parse_data_unit, parse_index_node,
     parse_packed_unit, unit_filesystem_identifier, DataUnitHeader, UnitError, PACKED_TYPE_INODE,
 };
-use crate::write_request_split::{data_unit_count_implied_by_the_file_size, DataUnitsOfTheInode};
+use crate::write_request_split::{data_units_read_within_the_file_size, DataUnitsOfTheInode};
 
 /// extent 叶记录 key 第三段（offset 段，文件字节偏移，D8（核心索引结构） 已定项 3）的起点。
 const EXTENT_KEY_THIRD_SEGMENT_OFFSET_IN_BYTES: usize = 16;
@@ -192,8 +193,11 @@ pub enum OpenPoolForReadFailure {
     },
     /// inode 叶容器的头与 inode 树内部条目里的身份引用不符（I-9.2（条目身份与子头相符））。
     InodeLeafContainerDisagreesWithTheInternalEntry { detail: &'static str },
-    /// inode 记录解不开（填充 / flags / 预留非零，I-9.7）。
+    /// inode 记录解不开（108–119 或 136–139 恒零区非零、截断字段 T = 0 而 N_w ≠ 0，I-9.7）。
     InodeRecordMalformed,
+    /// 系统配置不带 incompat 位 2 的池里读到截断字段不是全零的 inode 记录（用户 2026-10-17 定，判决
+    /// `research/prompts/m4-trunc-r2-main-verification.md` 第六节）：这条记录判不可用。`inode` 是那条记录的号。
+    InodeRecordCarriesATruncationFieldInAPoolWithoutIncompatBitTwo { inode: u64 },
     /// 节点里的一条记录 / 条目解不开：盘上自述的条目宽窄于字段表（这一版的读者不认这一版写的条目布局）。
     RecordMalformed { what: &'static str },
     /// 节点里的一条条目按字段表解得开，而它带的那条指针头部有一段不是第一版的取值（MAC / nonce 非 0、算法类型不是未加密、
@@ -347,11 +351,31 @@ pub struct MountedPoolForRead {
 /// 录制钩子挂在块设备接口上），装置侧的计数器在 `singlefs_harness::read_tally`。
 /// 核心层自己报的那几个数只罩打开之后的每一次读（[`ReadPathObservation`]）。
 ///
+/// inode 记录的截断字段照这个写者写的那一种池（带 incompat 位 2）收：调用方手里没有择好的系统配置时用它；只读挂载按择到的那一份判，走
+/// [`open_pool_for_read_judging_the_truncation_fields_as`]。
+///
 /// # Errors
 /// 见 [`OpenPoolForReadFailure`]：读不到 / 解不开 / 树表 0 条 / 树表里缺树 / 层级不对 / 叶容器身份不符。
 pub fn open_pool_for_read(
     reader: &dyn PoolReader,
     root: &RootRecord,
+) -> Result<MountedPoolForRead, OpenPoolForReadFailure> {
+    open_pool_for_read_judging_the_truncation_fields_as(
+        reader,
+        root,
+        InodeRecordTruncationFieldsOfThePool::WRITTEN_BY_THIS_WRITER,
+    )
+}
+
+/// 同 [`open_pool_for_read`]，inode 记录的截断字段按择到的系统配置那一档判：不带 incompat 位 2 的池里截断字段不是全零的记录判不可用
+/// （[`OpenPoolForReadFailure::InodeRecordCarriesATruncationFieldInAPoolWithoutIncompatBitTwo`]）。
+///
+/// # Errors
+/// 同 [`open_pool_for_read`]，另加上面那一种。
+pub fn open_pool_for_read_judging_the_truncation_fields_as(
+    reader: &dyn PoolReader,
+    root: &RootRecord,
+    truncation_fields_of_the_pool: InodeRecordTruncationFieldsOfThePool,
 ) -> Result<MountedPoolForRead, OpenPoolForReadFailure> {
     let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
     let filesystem_identifier_in_unit_headers =
@@ -529,10 +553,16 @@ pub fn open_pool_for_read(
             );
         }
         for record_bytes in &container.records {
-            inode_records.push(
-                InodeRecord::parse(record_bytes)
-                    .ok_or(OpenPoolForReadFailure::InodeRecordMalformed)?,
-            );
+            let record = InodeRecord::parse(record_bytes)
+                .ok_or(OpenPoolForReadFailure::InodeRecordMalformed)?;
+            if !record.truncation_field_fits_the_pool(truncation_fields_of_the_pool) {
+                return Err(
+                    OpenPoolForReadFailure::InodeRecordCarriesATruncationFieldInAPoolWithoutIncompatBitTwo {
+                        inode: record.inode,
+                    },
+                );
+            }
+            inode_records.push(record);
         }
     }
 
@@ -604,18 +634,26 @@ pub fn mount_read_only(reader: &dyn PoolReader) -> Result<MountedReadOnly, Mount
         true,
     )
     .map_err(MountReadOnlyFailure::Recovery)?;
-    let (mounted, chosen_root, effective_root) = match open_pool_for_read(reader, &effective_root) {
-        Ok(mounted) => (mounted, chosen_root, effective_root),
-        // 树表 0 条不是走读失败（这一版还没发布过文件版本），不退。
-        Err(OpenPoolForReadFailure::TreeTableHasNoEntries) => {
-            return Err(MountReadOnlyFailure::Open(
-                OpenPoolForReadFailure::TreeTableHasNoEntries,
-            ));
-        }
-        // 所选那一版走读失败、系统配置没见证它：退到 [见证到的那一版, 所选根) 里最新的那条根（F2 乙，与恢复、可写挂载同一份判法，
-        // `recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`），不重放、直接打开；打不开照报所选那一版的失败。
-        Err(failure_of_the_selected_version) => {
-            let version_fallen_back_to_opened =
+    let truncation_fields_of_the_pool = system_configuration
+        .immutable
+        .inode_record_truncation_fields;
+    let (mounted, chosen_root, effective_root) =
+        match open_pool_for_read_judging_the_truncation_fields_as(
+            reader,
+            &effective_root,
+            truncation_fields_of_the_pool,
+        ) {
+            Ok(mounted) => (mounted, chosen_root, effective_root),
+            // 树表 0 条不是走读失败（这一版还没发布过文件版本），不退。
+            Err(OpenPoolForReadFailure::TreeTableHasNoEntries) => {
+                return Err(MountReadOnlyFailure::Open(
+                    OpenPoolForReadFailure::TreeTableHasNoEntries,
+                ));
+            }
+            // 所选那一版走读失败、系统配置没见证它：退到 [见证到的那一版, 所选根) 里最新的那条根（F2 乙，与恢复、可写挂载同一份判法，
+            // `recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`），不重放、直接打开；打不开照报所选那一版的失败。
+            Err(failure_of_the_selected_version) => {
+                let version_fallen_back_to_opened =
                 newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
                     reader,
                     &system_configuration,
@@ -624,18 +662,24 @@ pub fn mount_read_only(reader: &dyn PoolReader) -> Result<MountedReadOnly, Mount
                     &records,
                 )
                 .and_then(|version_fallen_back_to| {
-                    open_pool_for_read(reader, &version_fallen_back_to)
-                        .ok()
+                    open_pool_for_read_judging_the_truncation_fields_as(
+                        reader,
+                        &version_fallen_back_to,
+                        truncation_fields_of_the_pool,
+                    )
+                    .ok()
                         .map(|mounted| (mounted, version_fallen_back_to))
                 });
-            match version_fallen_back_to_opened {
-                Some((mounted, version_fallen_back_to)) => {
-                    (mounted, version_fallen_back_to, version_fallen_back_to)
+                match version_fallen_back_to_opened {
+                    Some((mounted, version_fallen_back_to)) => {
+                        (mounted, version_fallen_back_to, version_fallen_back_to)
+                    }
+                    None => {
+                        return Err(MountReadOnlyFailure::Open(failure_of_the_selected_version))
+                    }
                 }
-                None => return Err(MountReadOnlyFailure::Open(failure_of_the_selected_version)),
             }
-        }
-    };
+        };
     Ok(MountedReadOnly {
         mounted,
         chosen_root,
@@ -734,18 +778,28 @@ impl MountedPoolForRead {
         let upper_height = upper_nodes_read;
         // 上段这一趟读到叶没有：停下时最后读的那个节点是层级 0 才算读到一片（上段停在内部节点时不算）。
         let upper_leaves = upper_leaves_read_at_open(level_of_the_node_read_last.get());
-        let data_units_of_the_inode = match entry.map(|found| found.target) {
-            None | Some(ExtentUpperLeafTarget::NoDataUnit) => DataUnitsOfTheInode::NoneWritten,
+        let target: Option<ExtentUpperLeafTarget> = entry.map(|found| found.target);
+        // 标签 3、4 与「标签 0 配 size > 0」最后一个存在的单元之后到 size 是洞（判决 m4-trunc-r1 K5）。
+        let data_units_of_the_inode = match target {
+            None => DataUnitsOfTheInode::NoneWritten,
             Some(
                 ExtentUpperLeafTarget::InlineDataUnit(_)
                 | ExtentUpperLeafTarget::LowerSegmentRoot(_),
             ) => DataUnitsOfTheInode::SomeWritten,
+            Some(
+                ExtentUpperLeafTarget::NoDataUnit
+                | ExtentUpperLeafTarget::InlineDataUnitFollowedByAHole(_)
+                | ExtentUpperLeafTarget::LowerSegmentRootFollowedByAHole(_),
+            ) => DataUnitsOfTheInode::HoleAfterTheLastDataUnit,
         };
         let (data_pointers, lower_height, lower_leaves): (Vec<(u64, DataPointer)>, u64, u64) =
-            match entry.map(|found| found.target) {
-                None | Some(ExtentUpperLeafTarget::NoDataUnit) => (Vec::new(), 0, 0),
-                Some(ExtentUpperLeafTarget::InlineDataUnit(pointer)) => (vec![(0, pointer)], 0, 0),
-                Some(ExtentUpperLeafTarget::LowerSegmentRoot(lower_root)) => {
+            match (
+                target.and_then(ExtentUpperLeafTarget::inline_data_unit),
+                target.and_then(ExtentUpperLeafTarget::lower_segment_root),
+            ) {
+                (None, None) => (Vec::new(), 0, 0),
+                (Some(pointer), _) => (vec![(0, pointer)], 0, 0),
+                (None, Some(lower_root)) => {
                     let segment = read_lower_segment(
                         &reading,
                         inode.0,
@@ -775,7 +829,7 @@ impl MountedPoolForRead {
             leaves: upper_leaves + lower_leaves,
         };
         let payload_capacity = payload_capacity_in_bytes();
-        let data_unit_records: Vec<ExtentLeafRecordInMountState> = data_pointers
+        let mut data_unit_records: Vec<ExtentLeafRecordInMountState> = data_pointers
             .into_iter()
             .map(|(unit, data_unit_pointer)| {
                 let key: [u8; 24] =
@@ -805,19 +859,26 @@ impl MountedPoolForRead {
                 );
             }
         }
-        // 文件没有洞、尾巴上也没有多出来的记录：写过内容的与写侧用同一条除法算单元数，一次都没写过的长度 0 就是 0 个
-        // （`write_request_split::data_unit_count_implied_by_the_file_size`），不在这里另写一个。
-        let data_units_implied_by_the_file_size =
-            data_unit_count_implied_by_the_file_size(inode_record.size, data_units_of_the_inode);
+        // 单元数按标签对 size（`write_request_split::data_units_read_within_the_file_size`，不在这里另写一个）：标签 1、2 不少于
+        // size 除出来的数，多出来的是截断中间态里 size 之外还挂着的单元——不读、不判声明长度，从记录里去掉（判决 m4-trunc-r1 K2）；
+        // 标签 3、4、0 少于它，之后到 size 是洞，读出 0（K5）。
         let extent_records = u64::try_from(data_unit_records.len()).expect("记录条数");
-        if extent_records != data_units_implied_by_the_file_size {
-            return Err(OpenFileFailure::ExtentRecordCountDoesNotMatchTheFileSize {
+        let data_units_within_the_size = data_units_read_within_the_file_size(
+            inode_record.size,
+            extent_records,
+            data_units_of_the_inode,
+        )
+        .map_err(
+            |refused| OpenFileFailure::ExtentRecordCountDoesNotMatchTheFileSize {
                 inode,
                 extent_records,
-                data_units_implied_by_the_file_size,
+                data_units_implied_by_the_file_size: refused.data_units_implied_by_the_file_size,
                 file_size_in_bytes: inode_record.size,
-            });
-        }
+            },
+        )?;
+        data_unit_records.truncate(
+            usize::try_from(data_units_within_the_size).expect("size 之内的单元数不多于记录条数"),
+        );
         Ok(OpenFileForRead {
             mount: self,
             inode_record,
@@ -927,12 +988,22 @@ impl OpenFileForRead<'_> {
         // 迭代次数的上界 = 这次读跨的单元数；循环体跨轮携带的只有上一个单元的锚点偏移与已拷出的字节。
         for unit_number in span.first.0..=span.last_inclusive.0 {
             let unit_index_in_file = DataUnitIndexInFile(unit_number);
-            let record = *self
+            let unit_starts_at_file_byte = unit_number * payload_capacity;
+            let Some(record) = self
                 .data_unit_records
                 .get(usize::try_from(unit_number).expect("单元序号装得进 usize"))
-                .expect(
-                    "单元序号由 (文件大小 − 1) ÷ 净荷容量界住，而记录条数在 open_file 里核过等于单元数",
+                .copied()
+            else {
+                // 尾部的洞（标签 3、4、0，判决 m4-trunc-r1 K5）：这一段答 0、不读盘。单元序号由 (文件大小 − 1) ÷ 净荷容量界住，
+                // 记录条数在 open_file 里按标签核过，记录之后到 size 只能是洞。
+                let hole_from = offset.0.max(unit_starts_at_file_byte);
+                let hole_until = end_exclusive.min(unit_starts_at_file_byte + payload_capacity);
+                bytes.resize(
+                    bytes.len() + usize::try_from(hole_until - hole_from).expect("一个单元以内"),
+                    0,
                 );
+                continue;
+            };
             let unit_bytes =
                 self.dereference_data_unit(reader, record, unit_index_in_file, &mut observation)?;
             let header = parse_data_unit(&unit_bytes).map_err(|error| {
@@ -956,7 +1027,6 @@ impl OpenFileForRead<'_> {
                         detail: "声明长度之后的补齐字节非零",
                     }
                 })?;
-            let unit_starts_at_file_byte = unit_number * payload_capacity;
             let copy_from = offset.0.max(unit_starts_at_file_byte) - unit_starts_at_file_byte;
             let copy_until = end_exclusive
                 .min(unit_starts_at_file_byte + u64::from(header.declared_length))
@@ -1354,6 +1424,7 @@ mod tests {
                 size: 0,
                 change_count: 1,
                 write_time_seconds: 1_788_000_000,
+                truncation_field: crate::records::InodeRecordTruncationField::NOT_WRITTEN,
             }],
             central_mapping_entries: Vec::new(),
             central_mapping_tree_reads_at_open: CentralMappingTreeReadsAtOpen {
diff --git a/crates/singlefs-core/src/records.rs b/crates/singlefs-core/src/records.rs
index 286dd15..25c6d14 100644
--- a/crates/singlefs-core/src/records.rs
+++ b/crates/singlefs-core/src/records.rs
@@ -15,6 +15,7 @@ use crate::pointer::{
     BirthSequence, DataPointer, LocationEntry, NodePointer, PointerHead,
     PointerHeadFieldOutsideTheFirstVersion,
 };
+use crate::system_configuration::InodeRecordTruncationFieldsOfThePool;
 use crate::unit::{PackedIdentity, WriteOrder, UNIT_CLASS_DATA};
 
 /// 统计量标签 = D5（快照 / 空间记账机制） 已定项 4 的行号（已定项 10 的登记表）。
@@ -32,6 +33,9 @@ pub const STATISTIC_NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;
 /// 第一版直落叶，seq 一律 1（D8（核心索引结构） 已定项 10）。
 pub const ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF: u32 = 1;
 
+/// inode 记录里截断字段（T 8 字节、N_w 8 字节）的起点（判决 m4-trunc-r2 R1）：之前 108–119、之后 136–139 两段仍恒 0。
+pub const INODE_RECORD_TRUNCATION_FIELD_OFFSET: u64 = 120;
+
 /// inode 记录偏移 48 那个 `blocks` 字段的计量单位：512 字节一块（D8（核心索引结构） 已定项 6）。
 pub const INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES: u64 = 512;
 
@@ -41,6 +45,77 @@ pub const NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES: u32 = 1;
 /// unlink 的意图那次发布把记录写成 nlink = 0、size 不变（D8（核心索引结构） 已定项 16 第 1 条）：从那一版起读者答「没有文件」、写入口拒它。
 pub const NLINK_OF_AN_INODE_BEING_DELETED: u32 = 0;
 
+/// inode 记录偏移 120 起的截断字段（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R1）：T（截断字段的 txg，偏移 120 起
+/// 8 字节小端，0 是没写过）与 N_w（截断字段的 size，偏移 128 起 8 字节小端）。存 size 不存单元数：单元数由 ⌈N_w ÷ 净荷⌉.max(1) 现算。
+/// 合法性只判「T = 0 ⇒ N_w = 0」（截到 0 时 T ≠ 0、N_w = 0）：两段私有，只能经 [`Self::NOT_WRITTEN`]、
+/// [`Self::written_by_the_truncation_intent_of`] 与 [`Self::from_the_two_fields_on_disk`] 进来，T = 0 而 N_w ≠ 0 的值造不出来。
+/// 什么时候写由 `truncate::truncation_field_after_the_truncation_intent` 判（R2 与第六节用户定案）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct InodeRecordTruncationField {
+    truncation_txg: CheckpointTxg,
+    truncated_size_in_bytes: u64,
+}
+
+impl InodeRecordTruncationField {
+    /// T = 0、N_w = 0：建出来之后没被截断意图写过，或整份写（顺序写、覆盖写）清零之后。
+    pub const NOT_WRITTEN: Self = Self {
+        truncation_txg: CheckpointTxg(0),
+        truncated_size_in_bytes: 0,
+    };
+
+    /// 截断意图那次写下的（这次 txg, N）。
+    ///
+    /// # Panics
+    /// `truncation_txg` 是 0：T = 0 是「没写过」，截断意图那次发布的 checkpoint_txg 恒大于 0（mkfs 那一版是第 0 代，之后每次发布加一）。
+    #[must_use]
+    pub fn written_by_the_truncation_intent_of(
+        truncation_txg: CheckpointTxg,
+        truncated_size_in_bytes: u64,
+    ) -> Self {
+        assert_ne!(
+            truncation_txg.0, 0,
+            "截断意图那次发布的 checkpoint_txg 不是 0（第 0 代是 mkfs 那一版，不发截断）"
+        );
+        Self {
+            truncation_txg,
+            truncated_size_in_bytes,
+        }
+    }
+
+    /// 盘上读来的两段 8 字节：T = 0 而 N_w ≠ 0 不合法 ⇒ `None`（R1）。
+    #[must_use]
+    pub const fn from_the_two_fields_on_disk(
+        truncation_txg: u64,
+        truncated_size_in_bytes: u64,
+    ) -> Option<Self> {
+        if truncation_txg == 0 && truncated_size_in_bytes != 0 {
+            return None;
+        }
+        Some(Self {
+            truncation_txg: CheckpointTxg(truncation_txg),
+            truncated_size_in_bytes,
+        })
+    }
+
+    /// T：写下字段的那次截断意图的 checkpoint_txg；没写过是 0。
+    #[must_use]
+    pub const fn truncation_txg(self) -> CheckpointTxg {
+        self.truncation_txg
+    }
+
+    /// N_w：写下字段的那次截断意图的目标 size；没写过是 0。
+    #[must_use]
+    pub const fn truncated_size_in_bytes(self) -> u64 {
+        self.truncated_size_in_bytes
+    }
+
+    /// T ≠ 0：被一次截断意图写过。
+    #[must_use]
+    pub const fn is_written(self) -> bool {
+        self.truncation_txg.0 != 0
+    }
+}
+
 /// inode 记录 140（D8（核心索引结构） 已定项 6 的偏移表）；时间是发布参数，不取系统时钟（可复现）。
 ///
 /// 偏移 48 的 `blocks` **不是字段**：条款定它 = ⌈`size` ÷ 512⌉（逻辑长度的块数，不表示分到的空间），
@@ -56,6 +131,8 @@ pub struct InodeRecord {
     pub size: u64,
     pub change_count: u64,
     pub write_time_seconds: u64,
+    /// 偏移 120 起两段 8 字节（判决 m4-trunc-r2 R1）。
+    pub truncation_field: InodeRecordTruncationField,
 }
 
 impl InodeRecord {
@@ -82,15 +159,26 @@ impl InodeRecord {
         writer.assert_position(96, "时间纳秒");
         writer.skip(12);
         writer.assert_position(108, "填充 / flags / 预留");
-        writer.skip(4 + 8 + 20);
+        writer.skip(4 + 8);
+        writer.assert_position(INODE_RECORD_TRUNCATION_FIELD_OFFSET, "截断字段 T 与 N_w");
+        writer.put_u64(self.truncation_field.truncation_txg().0);
+        writer.put_u64(self.truncation_field.truncated_size_in_bytes());
+        writer.assert_position(136, "预留");
+        writer.skip(4);
         writer.assert_position(INODE_RECORD_BYTES, "inode 记录");
         writer.into_bytes()
     }
-    /// 读者：三段填充 / flags / 预留（偏移 108 起）非零判不可用（I-9.7）。
+    /// 读者：偏移 108–119 与 136–139 两段恒零区非零、截断字段「T = 0 而 N_w ≠ 0」都判不可用（I-9.7，判决 m4-trunc-r2 R1）。
+    /// 池里认不认截断字段（incompat 位 2）不归这里判：[`Self::truncation_field_fits_the_pool`]。
     #[must_use]
     pub fn parse(bytes: &[u8]) -> Option<Self> {
+        let truncation_field_offset =
+            usize::try_from(INODE_RECORD_TRUNCATION_FIELD_OFFSET).expect("120");
         if bytes.len() != usize::try_from(INODE_RECORD_BYTES).expect("140")
-            || bytes[108..].iter().any(|byte| *byte != 0)
+            || bytes[108..truncation_field_offset]
+                .iter()
+                .chain(&bytes[truncation_field_offset + 16..])
+                .any(|byte| *byte != 0)
         {
             return None;
         }
@@ -104,6 +192,16 @@ impl InodeRecord {
         let write_time_seconds = reader.get_u64();
         reader.skip(16);
         let change_count = reader.get_u64();
+        let mut truncation_field_reader = ByteReader::at(
+            bytes,
+            usize::try_from(INODE_RECORD_TRUNCATION_FIELD_OFFSET).expect("120"),
+        );
+        let truncation_txg = truncation_field_reader.get_u64();
+        let truncated_size_in_bytes = truncation_field_reader.get_u64();
+        let truncation_field = InodeRecordTruncationField::from_the_two_fields_on_disk(
+            truncation_txg,
+            truncated_size_in_bytes,
+        )?;
         Some(Self {
             inode,
             object_birth,
@@ -111,8 +209,24 @@ impl InodeRecord {
             size,
             change_count,
             write_time_seconds,
+            truncation_field,
         })
     }
+
+    /// 这条记录的截断字段在这个池里收不收（incompat 位 2，用户 2026-10-17 定，判决 m4-trunc-r2 第六节）：池的系统配置带位 2 时
+    /// 什么合法的字段都收；不带位 2 时截断字段必须全零（T 与 N_w 都是 0），否则这条记录判不可用。
+    #[must_use]
+    pub fn truncation_field_fits_the_pool(
+        &self,
+        truncation_fields_of_the_pool: InodeRecordTruncationFieldsOfThePool,
+    ) -> bool {
+        match truncation_fields_of_the_pool {
+            InodeRecordTruncationFieldsOfThePool::Carried => true,
+            InodeRecordTruncationFieldsOfThePool::AllZero => {
+                self.truncation_field == InodeRecordTruncationField::NOT_WRITTEN
+            }
+        }
+    }
 }
 
 /// 墓碑记录 40（D18（块里携带什么信息） 已定项 24 第 2 条；字段表在 `.claude/kb/layout/03-third-txn.md`「二、墓碑记录」）：
@@ -819,6 +933,7 @@ mod tests {
             // 新池新建文件那条记录的取值：改动计数 = 这次发布的 checkpoint_txg 3（增补 2 第 11 行）。
             change_count: 3,
             write_time_seconds: 1_788_000_000,
+            truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
         };
         let bytes = record.to_bytes();
         assert_eq!(bytes.len(), 140);
@@ -1125,6 +1240,131 @@ mod tests {
         );
     }
 
+    fn record_with_the_truncation_field(
+        truncation_field: InodeRecordTruncationField,
+    ) -> InodeRecord {
+        InodeRecord {
+            inode: 1,
+            object_birth: CheckpointTxg(3),
+            nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
+            size: 100,
+            change_count: 9,
+            write_time_seconds: 1_788_000_000,
+            truncation_field,
+        }
+    }
+
+    /// 判决 m4-trunc-r2 R1：T 写在偏移 120 起 8 字节、N_w 写在 128 起 8 字节，都是小端，读者从同一处读回来；两段各取一个每个字节都不同的值，
+    /// 写错位置、读错位置、读成 0、两段对调都对不上。之前 108–119、之后 136–139 两段仍是 0。
+    #[test]
+    fn truncation_field_is_written_little_endian_at_offsets_120_and_128_and_read_back() {
+        let record = record_with_the_truncation_field(
+            InodeRecordTruncationField::written_by_the_truncation_intent_of(
+                CheckpointTxg(0x0807_0605_0403_0201),
+                0x1817_1615_1413_1211,
+            ),
+        );
+        let bytes = record.to_bytes();
+        assert_eq!(bytes.len(), 140);
+        assert_eq!(
+            bytes[120..128],
+            [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
+            "T 在偏移 120 起 8 字节、小端"
+        );
+        assert_eq!(
+            bytes[128..136],
+            [0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18],
+            "N_w 在偏移 128 起 8 字节、小端"
+        );
+        assert!(
+            bytes[108..120]
+                .iter()
+                .chain(&bytes[136..])
+                .all(|byte| *byte == 0),
+            "108–119 与 136–139 仍恒 0"
+        );
+        assert_eq!(InodeRecord::parse(&bytes), Some(record), "读回来逐字段相同");
+        let not_written = record_with_the_truncation_field(InodeRecordTruncationField::NOT_WRITTEN);
+        assert!(
+            not_written.to_bytes()[108..].iter().all(|byte| *byte == 0),
+            "没写过的字段：偏移 108 起整段都是 0"
+        );
+    }
+
+    /// 判决 m4-trunc-r2 R1：合法性只判「T = 0 ⇒ N_w = 0」。T = 0 而 N_w ≠ 0 拒；截到 0 是 T ≠ 0、N_w = 0，收。
+    #[test]
+    fn parse_refuses_a_truncation_field_whose_txg_is_zero_while_its_size_is_not() {
+        let mut size_without_txg =
+            record_with_the_truncation_field(InodeRecordTruncationField::NOT_WRITTEN).to_bytes();
+        size_without_txg[128] = 5;
+        assert_eq!(
+            InodeRecord::parse(&size_without_txg),
+            None,
+            "T = 0、N_w = 5：不合法"
+        );
+        let truncated_to_zero = record_with_the_truncation_field(
+            InodeRecordTruncationField::written_by_the_truncation_intent_of(CheckpointTxg(7), 0),
+        );
+        assert_eq!(
+            InodeRecord::parse(&truncated_to_zero.to_bytes()).map(|parsed| parsed.truncation_field),
+            Some(
+                InodeRecordTruncationField::written_by_the_truncation_intent_of(
+                    CheckpointTxg(7),
+                    0
+                )
+            ),
+            "截到 0：T = 7、N_w = 0 合法，读回来 T 仍是 7"
+        );
+        assert_eq!(
+            InodeRecordTruncationField::from_the_two_fields_on_disk(0, 5),
+            None
+        );
+    }
+
+    /// 判决 m4-trunc-r2 R1：截断字段两边那两段恒零区（108–119、136–139）任一字节非零都拒，截断字段那 16 字节里的非零（T 非零）收。
+    #[test]
+    fn parse_refuses_a_nonzero_byte_in_either_always_zero_area_around_the_truncation_field() {
+        let bytes =
+            record_with_the_truncation_field(InodeRecordTruncationField::NOT_WRITTEN).to_bytes();
+        for offset in (108..120).chain(136..140) {
+            let mut tampered = bytes.clone();
+            tampered[offset] = 1;
+            assert_eq!(
+                InodeRecord::parse(&tampered),
+                None,
+                "偏移 {offset} 在恒零区里，非零拒收"
+            );
+        }
+        for offset in 120..128 {
+            let mut txg_byte_set = bytes.clone();
+            txg_byte_set[offset] = 1;
+            assert!(
+                InodeRecord::parse(&txg_byte_set).is_some(),
+                "偏移 {offset} 是 T 的一个字节：T 非零、N_w = 0 合法"
+            );
+        }
+    }
+
+    /// 用户 2026-10-17 定（判决 m4-trunc-r2 第六节）：不带 incompat 位 2 的池里截断字段必须全零；带位 2 的池什么合法的字段都收。
+    #[test]
+    fn a_written_truncation_field_fits_only_a_pool_whose_system_configuration_carries_bit_two() {
+        let not_written = record_with_the_truncation_field(InodeRecordTruncationField::NOT_WRITTEN);
+        let written = record_with_the_truncation_field(
+            InodeRecordTruncationField::written_by_the_truncation_intent_of(CheckpointTxg(7), 0),
+        );
+        assert!(not_written
+            .truncation_field_fits_the_pool(InodeRecordTruncationFieldsOfThePool::AllZero));
+        assert!(not_written
+            .truncation_field_fits_the_pool(InodeRecordTruncationFieldsOfThePool::Carried));
+        assert!(
+            !written.truncation_field_fits_the_pool(InodeRecordTruncationFieldsOfThePool::AllZero),
+            "不带位 2 的池里 T ≠ 0 的记录判不可用"
+        );
+        assert!(
+            written.truncation_field_fits_the_pool(InodeRecordTruncationFieldsOfThePool::Carried)
+        );
+    }
+
     /// D8（核心索引结构） 已定项 16 第 4 条：nlink 由 `to_bytes` 照结构里的值写进偏移 36、由 `parse` 读回来——
     /// 删除中的记录（nlink = 0，size 不变）写出去读回来还是 0，读者与写入口才认得出它；活着的写 1。
     #[test]
@@ -1136,6 +1376,7 @@ mod tests {
             size: 40_000,
             change_count: 6,
             write_time_seconds: 1_788_000_000,
+            truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
         };
         for nlink in [
             NLINK_OF_AN_INODE_BEING_DELETED,
@@ -1174,6 +1415,7 @@ mod tests {
                 size,
                 change_count: 3,
                 write_time_seconds: 1_788_000_000,
+                truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
             }
             .to_bytes();
             u64::from_le_bytes(bytes[48..56].try_into().expect("偏移 48 起 8 字节"))
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 604cdfe..9b04943 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -89,10 +89,13 @@ use crate::retirement_tree::RetirementContainerIndexInTree;
 use crate::root_record::RootRecord;
 use crate::root_ring::{region_start, slot_offset, RootRingSlot, RootRingSlotsPerRegionOutOfRange};
 use crate::system_configuration::{
-    journal_in_flight_record_limit, journal_in_flight_record_limit_fits_its_four_byte_field,
-    journal_ring_geometry_fields_recorded_in_the_slot, unit_area_start_slot_recorded_in_the_slot,
-    IncompatBitmap, SystemConfiguration, SystemConfigurationSlotRefusal, SystemImmutableSizes,
-    JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION, ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
+    filesystem_identifier_self_described_by_the_slot, journal_in_flight_record_limit,
+    journal_in_flight_record_limit_fits_its_four_byte_field,
+    journal_ring_geometry_fields_recorded_in_the_slot, root_ring_region_count_recorded_in_the_slot,
+    unit_area_start_slot_recorded_in_the_slot, IncompatBitmap,
+    InodeRecordTruncationFieldsOfThePool, SystemConfiguration, SystemConfigurationSlotRefusal,
+    SystemImmutableSizes, JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
+    ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
 };
 use crate::tombstone_tree::TombstoneContainerIndexInTree;
 use crate::tombstone_tree::CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES;
@@ -3090,6 +3093,9 @@ pub struct RebuiltVersionWithItsSnapshotEntries {
 /// 同 [`rebuild_version_in_the_unit_area_starting_at`]（判全、报错都与它相同），另把树表里码 14、15 的条目配成对交出来
 /// （[`snapshot_tree_table_entries_paired_one_to_one`]，配不成对的树表在 [`tree_table_entries_each_kind_at_most_once`] 那一步就拒了）。
 ///
+/// 截断字段照这个写者写的那一种池（带 incompat 位 2）收：调用方手里没有择好的系统配置时用它；可写挂载按择到的那一份判，走
+/// [`rebuild_version_judging_the_truncation_fields_as`]。
+///
 /// # Errors
 /// 同 [`rebuild_version_in_the_unit_area_starting_at`]。
 pub fn rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at(
@@ -3097,6 +3103,45 @@ pub fn rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at(
     unit_area_start: UnitAreaStart,
     root: &RootRecord,
     record_standing_for_root: Option<JournalRecord>,
+) -> Result<RebuiltVersionWithItsSnapshotEntries, RebuildVersionFailure> {
+    rebuild_version_with_its_snapshot_entries_judging_the_truncation_fields_as(
+        reader,
+        unit_area_start,
+        InodeRecordTruncationFieldsOfThePool::WRITTEN_BY_THIS_WRITER,
+        root,
+        record_standing_for_root,
+    )
+}
+
+/// 同 [`rebuild_version_in_the_unit_area_starting_at`]，inode 记录的截断字段按择到的系统配置那一档判（`truncation_fields_of_the_pool`：
+/// 不带 incompat 位 2 的池里截断字段非零的记录判不可用，交回 `InvariantViolated`，detail 是
+/// [`INODE_RECORD_TRUNCATION_FIELD_IN_A_POOL_WITHOUT_INCOMPAT_BIT_TWO`]）。可写挂载建上一版走这一份。
+///
+/// # Errors
+/// 同 [`rebuild_version_in_the_unit_area_starting_at`]。
+pub fn rebuild_version_judging_the_truncation_fields_as(
+    reader: &dyn PoolReader,
+    unit_area_start: UnitAreaStart,
+    truncation_fields_of_the_pool: InodeRecordTruncationFieldsOfThePool,
+    root: &RootRecord,
+    record_standing_for_root: Option<JournalRecord>,
+) -> Result<RebuiltVersion, RebuildVersionFailure> {
+    rebuild_version_with_its_snapshot_entries_judging_the_truncation_fields_as(
+        reader,
+        unit_area_start,
+        truncation_fields_of_the_pool,
+        root,
+        record_standing_for_root,
+    )
+    .map(|rebuilt| rebuilt.version)
+}
+
+fn rebuild_version_with_its_snapshot_entries_judging_the_truncation_fields_as(
+    reader: &dyn PoolReader,
+    unit_area_start: UnitAreaStart,
+    truncation_fields_of_the_pool: InodeRecordTruncationFieldsOfThePool,
+    root: &RootRecord,
+    record_standing_for_root: Option<JournalRecord>,
 ) -> Result<RebuiltVersionWithItsSnapshotEntries, RebuildVersionFailure> {
     let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
     let data_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
@@ -3298,6 +3343,7 @@ pub fn rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at(
             },
             root,
             expected_filesystem_identifier,
+            truncation_fields_of_the_pool,
         )?;
         if records.is_empty() {
             return Err(RecoveryFailure::UnitMalformed {
@@ -5253,9 +5299,11 @@ impl ExtentsOfTheFirstFile {
 /// 第一个文件的单元从 0 起连号、没有洞（尾部的洞由标签声明）；第一个文件之外的文件只由按 inode 号写数据的入口从偏移 0 整份写，
 /// 单元从 0 起连号、至少一个、尾部没有洞（标签 1、2）。上段里可以没有第一个文件那一条（它删完了、或还没有单元）。
 ///
+/// 第一个文件那一条是标签 0（一个单元都没有）的收下，交回空的数据指针与「之后到 size 是洞」的尾部：整个文件是洞（判决 m4-trunc-r1 K5）。
+///
 /// # Errors
-/// 上段里没有第一个文件的条目、或还有别的 inode 的条目、第一个文件一个单元都没有、单元有洞 ⇒ `UnitMalformed`：
-/// 这几样今天的写路径都写不出来（只有它写 extent 树、每个文件版本从偏移 0 顺序写、长度 0 的内容也写一个单元），
+/// 第一个文件的单元有洞，或第一个文件之外的文件没有单元、单元有洞、尾部有洞 ⇒ `UnitMalformed`：
+/// 这几样今天的写路径都写不出来（每个文件版本从偏移 0 顺序写、长度 0 的内容也写一个单元；截断与变长只做第一个文件），
 /// 盘上读来的却可以是任何样子——第一版不支持，在任何落盘动作之前交回。
 pub(crate) fn extents_of_the_first_file(
     tree: ExtentTreeReadFromDisk,
@@ -5297,23 +5345,21 @@ pub(crate) fn extents_of_the_first_file(
             lower_nodes,
         });
     }
-    let [(inode, extents)] =
-        <[(u64, ExtentsOfAFileReadFromDisk); 1]>::try_from(files).map_err(|_files| {
-            RecoveryFailure::UnitMalformed {
-                what: "extent 树上段里不是恰好一条叶条目（第一版只有第一个文件有内容）",
-            }
-        })?;
-    if inode != FIRST_INODE_NUMBER {
-        return Err(RecoveryFailure::UnitMalformed {
-            what: "extent 树上段里那一条叶条目不是第一个文件的",
-        });
-    }
+    // 上段里没有第一个文件那一条：第一个文件删完之后的零条目根（D8（核心索引结构） 已定项 16 第 5 条），或删完之后只有别的文件有条目，
+    // 第一个文件没有数据单元、没有下段。
+    let Some(extents) = extents_of_the_first_file_on_disk else {
+        return Ok(extents_assembled_with_the_upper_segment(
+            upper,
+            Vec::new(),
+            TailOfAFile::NoHoleAfterTheLastDataUnit,
+            Vec::new(),
+            files_other_than_the_first,
+        ));
+    };
+    // 标签 0（一个数据单元都没有）收下：整个文件是洞（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5「标签 0 配 size > 0
+    // 是整个文件都是洞」），读出 0、不读盘；单元数对不对 size 由调用方按标签声明的尾部判（`data_unit_count_matches_the_inode_size`）。
+    // 与挂载态读者同一读法（`mounted_read` 把标签 0 当整个洞）。今天的写路径写不出这一形。
     let data_pointers_with_units = extents.data_pointers();
-    if data_pointers_with_units.is_empty() {
-        return Err(RecoveryFailure::UnitMalformed {
-            what: "extent 树里第一个文件一个数据单元都没有",
-        });
-    }
     if data_pointers_with_units
         .iter()
         .enumerate()
@@ -5945,14 +5991,21 @@ fn inode_internal_entry_parsed_and_judged(
     Ok((separator_key, identity, child))
 }
 
+/// 系统配置不带 incompat 位 2 的池里读到截断字段非零的 inode 记录（用户 2026-10-17 定，判决 m4-trunc-r2 第六节）：
+/// 走读交回的 [`RecoveryFailure::InvariantViolated`] 带的那一句，调用方按它认出这一种拒。
+pub const INODE_RECORD_TRUNCATION_FIELD_IN_A_POOL_WITHOUT_INCOMPAT_BIT_TWO: &str =
+    "系统配置不带 incompat 位 2 的池里 inode 记录的截断字段不是全零";
+
 /// 一片 inode 叶容器按指着它的那条内部条目核（I-9.2（条目身份与子头相符）：叶头的四元组身份等于条目的身份引用、记录宽 140、fsid 是本池；
-/// I-1.2：出生树与出生序号等于子指针的、诞生不晚于根、写序实例等于子指针的），再逐条解它的记录、判记录号不小于分隔 key 与容器号（I-9.4）。
-/// 交回解开的记录，按容器里的次序。
+/// I-1.2：出生树与出生序号等于子指针的、诞生不晚于根、写序实例等于子指针的），再逐条解它的记录、判记录号不小于分隔 key 与容器号（I-9.4）、
+/// 截断字段在这个池里收不收（`truncation_fields_of_the_pool`：系统配置不带 incompat 位 2 的池里截断字段非零的记录判不可用，
+/// 判决 `research/prompts/m4-trunc-r2-main-verification.md` 第六节）。交回解开的记录，按容器里的次序。
 fn inode_leaf_container_judged_against_its_entry(
     leaf: &PackedUnitHeader,
     entry: InodeInternalEntry,
     root: &RootRecord,
     expected_filesystem_identifier: u64,
+    truncation_fields_of_the_pool: InodeRecordTruncationFieldsOfThePool,
 ) -> Result<Vec<InodeRecord>, RecoveryFailure> {
     let InodeInternalEntry {
         separator_key,
@@ -5991,6 +6044,12 @@ fn inode_leaf_container_judged_against_its_entry(
                 detail: "inode 记录号小于分隔 key 或容器号",
             });
         }
+        if !record.truncation_field_fits_the_pool(truncation_fields_of_the_pool) {
+            return Err(RecoveryFailure::InvariantViolated {
+                invariant: "I-9.7",
+                detail: INODE_RECORD_TRUNCATION_FIELD_IN_A_POOL_WITHOUT_INCOMPAT_BIT_TWO,
+            });
+        }
         records.push(record);
     }
     Ok(records)
@@ -6224,7 +6283,10 @@ fn pointers_of_the_rebuilt_version_ascend_by_device(
         )
         .chain(output.accounting_tree.pointers.iter())
         .chain(output.deadlist_tree.pointers.iter())
-        .chain(output.central_mapping_tree.pointers.iter());
+        .chain(output.central_mapping_tree.pointers.iter())
+        .chain(tombstone_tree_pointers)
+        .chain(lineage_tree_pointers)
+        .chain(retirement_tree_pointers);
     for pointer in node_pointers {
         pointer
             .location_entries_ascend_by_device()
@@ -6271,6 +6333,28 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
     unit_area_start: UnitAreaStart,
     root: &RootRecord,
     mapping_fallbacks: &mut usize,
+) -> Result<Option<Vec<u8>>, RecoveryFailure> {
+    walk_to_file_judging_the_truncation_fields_as(
+        reader,
+        unit_area_start,
+        InodeRecordTruncationFieldsOfThePool::WRITTEN_BY_THIS_WRITER,
+        root,
+        mapping_fallbacks,
+    )
+}
+
+/// 同 [`walk_to_file_in_the_unit_area_starting_at`]，inode 记录的截断字段按择到的系统配置那一档判（不带 incompat 位 2 的池里
+/// 截断字段非零的记录判不可用，交回 `InvariantViolated`，detail 是 [`INODE_RECORD_TRUNCATION_FIELD_IN_A_POOL_WITHOUT_INCOMPAT_BIT_TWO`]）。
+/// 恢复与 [`walk_to_file`] 走这一份。
+///
+/// # Errors
+/// 同 [`walk_to_file_in_the_unit_area_starting_at`]。
+pub fn walk_to_file_judging_the_truncation_fields_as(
+    reader: &dyn PoolReader,
+    unit_area_start: UnitAreaStart,
+    truncation_fields_of_the_pool: InodeRecordTruncationFieldsOfThePool,
+    root: &RootRecord,
+    mapping_fallbacks: &mut usize,
 ) -> Result<Option<Vec<u8>>, RecoveryFailure> {
     let expected_filesystem_identifier = unit_filesystem_identifier(&root.filesystem_identifier);
     let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
@@ -6581,6 +6665,7 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
             },
             root,
             expected_filesystem_identifier,
+            truncation_fields_of_the_pool,
         )?;
         for record in records {
             if record.inode == FIRST_INODE_NUMBER {
@@ -6731,6 +6816,9 @@ pub fn read_the_first_file_of_a_snapshot(
         expected_filesystem_identifier,
     )?;
     inode_tree_root_level_is_one(&inode_root)?;
+    // 只读快照住在挂着的池里：可写挂载取号那一写起系统配置就带 incompat 位 2，截断字段照带位 2 的池收。
+    let truncation_fields_of_the_pool =
+        InodeRecordTruncationFieldsOfThePool::WRITTEN_BY_THIS_WRITER;
     let mut inode_record: Option<InodeRecord> = None;
     for entry in &inode_root.entries {
         let (separator_key, identity, child) = inode_internal_entry_parsed_and_judged(entry)?;
@@ -6754,6 +6842,7 @@ pub fn read_the_first_file_of_a_snapshot(
             },
             root,
             expected_filesystem_identifier,
+            truncation_fields_of_the_pool,
         )?;
         if let Some(record) = records
             .into_iter()
@@ -6816,9 +6905,13 @@ pub fn walk_to_file(
     root: &RootRecord,
     mapping_fallbacks: &mut usize,
 ) -> Result<Option<Vec<u8>>, RecoveryFailure> {
-    walk_to_file_in_the_unit_area_starting_at(
+    let system_configuration = choose_system_configuration(reader)?;
+    walk_to_file_judging_the_truncation_fields_as(
         reader,
-        unit_area_start_of_the_pool(reader)?,
+        unit_area_start_of_the_chosen_system_configuration(&system_configuration),
+        system_configuration
+            .immutable
+            .inode_record_truncation_fields,
         root,
         mapping_fallbacks,
     )
@@ -6834,9 +6927,13 @@ pub fn rebuild_version(
     root: &RootRecord,
     record_standing_for_root: Option<JournalRecord>,
 ) -> Result<RebuiltVersion, RebuildVersionFailure> {
-    rebuild_version_in_the_unit_area_starting_at(
+    let system_configuration = choose_system_configuration(reader)?;
+    rebuild_version_judging_the_truncation_fields_as(
         reader,
-        unit_area_start_of_the_pool(reader)?,
+        unit_area_start_of_the_chosen_system_configuration(&system_configuration),
+        system_configuration
+            .immutable
+            .inode_record_truncation_fields,
         root,
         record_standing_for_root,
     )
@@ -6852,9 +6949,13 @@ pub fn rebuild_version_with_its_snapshot_entries(
     root: &RootRecord,
     record_standing_for_root: Option<JournalRecord>,
 ) -> Result<RebuiltVersionWithItsSnapshotEntries, RebuildVersionFailure> {
-    rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at(
+    let system_configuration = choose_system_configuration(reader)?;
+    rebuild_version_with_its_snapshot_entries_judging_the_truncation_fields_as(
         reader,
-        unit_area_start_of_the_pool(reader)?,
+        unit_area_start_of_the_chosen_system_configuration(&system_configuration),
+        system_configuration
+            .immutable
+            .inode_record_truncation_fields,
         root,
         record_standing_for_root,
     )
@@ -7090,9 +7191,13 @@ pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport
         }
     };
     let unit_area_start = unit_area_start_of_the_chosen_system_configuration(&system_configuration);
-    let (outcome, effective_root) = match walk_to_file_in_the_unit_area_starting_at(
+    let truncation_fields_of_the_pool = system_configuration
+        .immutable
+        .inode_record_truncation_fields;
+    let (outcome, effective_root) = match walk_to_file_judging_the_truncation_fields_as(
         reader,
         unit_area_start,
+        truncation_fields_of_the_pool,
         &effective_root,
         &mut mapping_fallbacks,
     ) {
@@ -7119,9 +7224,10 @@ pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport
                 JournalPolicy::Ignore => None,
             }
             .and_then(|version_fallen_back_to| {
-                walk_to_file_in_the_unit_area_starting_at(
+                walk_to_file_judging_the_truncation_fields_as(
                     reader,
                     unit_area_start,
+                    truncation_fields_of_the_pool,
                     &version_fallen_back_to,
                     &mut mapping_fallbacks,
                 )
diff --git a/crates/singlefs-core/src/system_configuration.rs b/crates/singlefs-core/src/system_configuration.rs
index add3566..1e53773 100644
--- a/crates/singlefs-core/src/system_configuration.rs
+++ b/crates/singlefs-core/src/system_configuration.rs
@@ -28,8 +28,14 @@ pub const INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT: u8
 /// incompat 位 0 退役：带回退见证、系统配置不带 F 的那一版（D15（格式冻结政策） 已定项 4，用户 2026-09-26）。
 /// 位一旦用过不许回收（`.claude/kb/feature-bits.md`）；它不在 [`SUPPORTED_INCOMPAT_BITS`] 里，读者见到它按不认识的位拒。
 pub const INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT: u8 = 0x01;
+/// incompat 位 2 = inode 记录带截断字段（偏移 120 起 T 与 N_w，用户 2026-10-17 定新开一位，判决
+/// `research/prompts/m4-trunc-r2-main-verification.md` 第六节；位号的登记归阶段同步）：mkfs 起就置上，写者每次写系统配置都写它。
+/// 读者收只带位 1 的（0x02）与位 1、位 2 都带的（0x06）两种；不带位 2 的池里截断字段必须全零
+/// （[`InodeRecordTruncationFieldsOfThePool::AllZero`]）。
+pub const INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT: u8 = 0x04;
 pub const SUPPORTED_INCOMPAT_BITS: u8 =
-    INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT;
+    INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT
+        | INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT;
 const FEATURE_BITS_OFFSET: usize = 4 + 2;
 const FEATURE_BITMAP_BYTES: usize = 32;
 /// 写入者身份：实现标识 16 字节 ASCII 零补齐 + 版本 4（D17（实现分层与第三方管道） 债 3、I-1.5）。
@@ -277,6 +283,48 @@ pub struct SystemImmutableConfiguration {
     /// 根环起点（偏移 371，16 KiB 槽号；基址 = 该字段 × 16384，D22（单元原子性怎么合成） 已定项 16 第 1 句）。
     /// 写者写 [`ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION`]；读者收的只有它，判固定结构槽距的上界用它（同一槽自述的根环起点）。
     pub root_ring_base_slot: SlotNumber,
+    /// incompat 位 2（偏移 6 那一字节）：写者写 [`InodeRecordTruncationFieldsOfThePool::WRITTEN_BY_THIS_WRITER`]；读者两种都收，
+    /// 走读 inode 记录时按它判截断字段收不收（`records::InodeRecord::truncation_field_fits_the_pool`）。
+    pub inode_record_truncation_fields: InodeRecordTruncationFieldsOfThePool,
+}
+
+/// 一个池的 inode 记录带不带截断字段：系统配置 incompat 位 2 带不带（[`INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT`]）。
+/// 封闭集合，`match` 不写通配臂。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum InodeRecordTruncationFieldsOfThePool {
+    /// 不带位 2：inode 记录偏移 120–135 必须全零，读到非零的记录判不可用。
+    AllZero,
+    /// 带位 2（mkfs 起就置上）：偏移 120–135 是截断字段 T 与 N_w。
+    Carried,
+}
+
+impl InodeRecordTruncationFieldsOfThePool {
+    /// 写者写进系统配置的那一种：mkfs 与之后每一次写系统配置都带位 2（用户 2026-10-17 定、主 agent 定 mkfs 写这一位）。
+    pub const WRITTEN_BY_THIS_WRITER: Self = InodeRecordTruncationFieldsOfThePool::Carried;
+
+    /// incompat 位图第一个字节里位 2 带不带。
+    #[must_use]
+    pub const fn of_the_first_incompat_byte(first_incompat_byte: u8) -> Self {
+        if first_incompat_byte & INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT == 0 {
+            InodeRecordTruncationFieldsOfThePool::AllZero
+        } else {
+            InodeRecordTruncationFieldsOfThePool::Carried
+        }
+    }
+
+    /// 写进 incompat 位图第一个字节的那几位：位 1（布局身份）恒带，位 2 照这一种。
+    #[must_use]
+    pub const fn first_incompat_byte(self) -> u8 {
+        match self {
+            InodeRecordTruncationFieldsOfThePool::AllZero => {
+                INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT
+            }
+            InodeRecordTruncationFieldsOfThePool::Carried => {
+                INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT
+                    | INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT
+            }
+        }
+    }
 }
 
 impl SystemImmutableConfiguration {
@@ -476,7 +524,11 @@ impl SystemConfiguration {
         slot.write(SlotFieldMutability::ImmutableConfiguration, |writer| {
             writer.put(&SYSTEM_CONFIGURATION_MAGIC);
             writer.put_u16(FORMAT_VERSION);
-            writer.put_u8(INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT);
+            writer.put_u8(
+                self.immutable
+                    .inode_record_truncation_fields
+                    .first_incompat_byte(),
+            );
             writer.skip(95);
             writer.assert_position(
                 u64::try_from(FSID_OFFSET).expect("fsid 偏移"),
@@ -661,8 +713,13 @@ impl SystemConfiguration {
             usize::try_from(ROLLBACK_FLOOR_OFFSET).expect("字段表里的偏移装得进 usize"),
         );
         let rollback_floor = CheckpointTxg(rollback_floor_reader.get_u64());
+        let inode_record_truncation_fields =
+            InodeRecordTruncationFieldsOfThePool::of_the_first_incompat_byte(
+                bytes[FEATURE_BITS_OFFSET],
+            );
         Ok(Self {
             immutable: SystemImmutableConfiguration {
+                inode_record_truncation_fields,
                 filesystem_identifier,
                 this_device,
                 device_count,
@@ -732,7 +789,7 @@ fn incompat_bitmap_of(slot: &[u8]) -> IncompatBitmap {
 
 /// incompat 位图里不认识的位 ⇒ 挂不上（`.claude/rules/fs-design.md`）；且必须带布局身份位。
 /// 退役的位 0（系统配置不带 F 的旧镜像）不在 [`SUPPORTED_INCOMPAT_BITS`] 里，按不认识的位拒，
-/// 与新的布局身份位同时置上也拒（D15（格式冻结政策） 已定项 4）。
+/// 与新的布局身份位同时置上也拒（D15（格式冻结政策） 已定项 4）。位 2（inode 记录带截断字段）带不带都收：第一个字节收 0x02 与 0x06 两种。
 #[must_use]
 pub fn incompat_bits_are_mountable(slot: &[u8]) -> bool {
     let incompat = &slot[FEATURE_BITS_OFFSET..FEATURE_BITS_OFFSET + FEATURE_BITMAP_BYTES];
@@ -764,6 +821,7 @@ mod tests {
                 },
                 journal_ring_start_slot: JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
                 root_ring_base_slot: ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
+                inode_record_truncation_fields: InodeRecordTruncationFieldsOfThePool::Carried,
             },
             mutable: SystemMutableConfiguration,
             runtime: SystemRuntimeConfiguration,
@@ -776,6 +834,14 @@ mod tests {
         }
     }
 
+    /// 同 [`sample`]，系统配置不带 incompat 位 2（第一条布局线、截断字段之前的旧镜像）。
+    fn sample_without_the_truncation_fields_bit() -> SystemConfiguration {
+        let mut without = sample();
+        without.immutable.inode_record_truncation_fields =
+            InodeRecordTruncationFieldsOfThePool::AllZero;
+        without
+    }
+
     /// 把槽里 incompat 位图第一个字节改成 `first_incompat_byte`，整槽校验和重封：magic 与校验和两关照样过，拒它的只能是 incompat 判。
     fn slot_with_first_incompat_byte(first_incompat_byte: u8) -> Vec<u8> {
         let mut slot = sample().to_slot();
@@ -799,8 +865,8 @@ mod tests {
             "489 之后全是补齐 0：字段表之后不装别的东西"
         );
         assert_eq!(
-            slot[6], 0x02,
-            "incompat 只有位 1（新的布局身份），退役的位 0 不置"
+            slot[6], 0x06,
+            "incompat 带位 1（新的布局身份）与位 2（inode 记录带截断字段），退役的位 0 不置"
         );
         assert_eq!(
             &slot[FSID_OFFSET + 16..FSID_OFFSET + 16 + 11],
@@ -830,20 +896,20 @@ mod tests {
             "补齐区改一字节也失配：整槽校验和罩 4096"
         );
         let mut unknown = sample().to_slot();
-        unknown[6] |= 0x04;
+        unknown[6] |= 0x08;
         assert!(!incompat_bits_are_mountable(&unknown));
         let resealed_unknown = slot_with_first_incompat_byte(
-            INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT | 0x04,
+            INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT | 0x08,
         );
         let mut expected_bitmap = [0u8; FEATURE_BITMAP_BYTES];
         expected_bitmap[0] =
-            INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT | 0x04;
+            INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT | 0x08;
         assert_eq!(
             SystemConfiguration::parse_slot(&resealed_unknown),
             Err(SystemConfigurationSlotRefusal::IncompatBitsNotRecognized {
                 incompat_bitmap: IncompatBitmap(expected_bitmap),
             }),
-            "位 2 不认识、整槽重封过：报布局不认识、带着位图，不报自证不过"
+            "位 3 没分出去、整槽重封过：报布局不认识、带着位图，不报自证不过"
         );
     }
 
@@ -885,11 +951,53 @@ mod tests {
         );
         assert_eq!(
             SystemConfiguration::parse_slot(&new_layout_only),
-            Ok(sample()),
+            Ok(sample_without_the_truncation_fields_bit()),
             "只带位 1 的收"
         );
     }
 
+    /// incompat 位 2（用户 2026-10-17 定，判决 m4-trunc-r2 第六节）：读者收第一个字节 0x02 与 0x06 两种，读成池的截断字段那一档
+    /// （0x02 ⇒ 截断字段必须全零、0x06 ⇒ 带截断字段），写者照那一档写回同一个字节；只带位 2、不带布局身份位 1 的拒。三份槽都重封过整槽校验和。
+    #[test]
+    fn incompat_bit_two_is_read_into_the_truncation_fields_of_the_pool_and_written_back() {
+        assert_eq!(
+            SystemConfiguration::parse_slot(&slot_with_first_incompat_byte(0x02)),
+            Ok(sample_without_the_truncation_fields_bit()),
+            "0x02：不带位 2 的池，截断字段必须全零"
+        );
+        assert_eq!(
+            SystemConfiguration::parse_slot(&slot_with_first_incompat_byte(0x06)),
+            Ok(sample()),
+            "0x06：带位 2 的池"
+        );
+        assert_eq!(
+            sample_without_the_truncation_fields_bit().to_slot()[FEATURE_BITS_OFFSET],
+            0x02,
+            "不带位 2 那一档写回 0x02"
+        );
+        assert_eq!(
+            sample().to_slot()[FEATURE_BITS_OFFSET],
+            0x06,
+            "带位 2 那一档写回 0x06"
+        );
+        let mut expected_bitmap = [0u8; FEATURE_BITMAP_BYTES];
+        expected_bitmap[0] = INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT;
+        assert_eq!(
+            SystemConfiguration::parse_slot(&slot_with_first_incompat_byte(
+                INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT
+            )),
+            Err(SystemConfigurationSlotRefusal::IncompatBitsNotRecognized {
+                incompat_bitmap: IncompatBitmap(expected_bitmap),
+            }),
+            "只带位 2、不带布局身份位 1：拒"
+        );
+        assert_eq!(
+            InodeRecordTruncationFieldsOfThePool::WRITTEN_BY_THIS_WRITER,
+            InodeRecordTruncationFieldsOfThePool::Carried,
+            "写者写位 2"
+        );
+    }
+
     /// 系统配置里的回退下界 F（D22（单元原子性怎么合成） 已定项 9）：写在 [481, 489)、小端，读者从同一处读回来；
     /// 取一个每个字节都不同的值，写错位置、读错位置、读成 0 都对不上。
     #[test]
diff --git a/crates/singlefs-core/src/truncate.rs b/crates/singlefs-core/src/truncate.rs
new file mode 100644
index 0000000..ebe39b0
--- /dev/null
+++ b/crates/singlefs-core/src/truncate.rs
@@ -0,0 +1,695 @@
+//! 截断（truncate）与变长的计划（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2、K3、K5、K11、K12）：
+//! 意图那次要不要重写尾单元、之后每一批留几个单元、改 size 之后 extent 树哪几个节点重写。
+//!
+//! 和 `crate::unlink` 一样只算、不发一个写、不动分配器、不碰落点：条款的几条纪律（先落目标：意图那次写 size = N 并重写尾单元；
+//! 每批只从尾部放 K 个、不碰 inode 记录；最后一批放完剩下的、可以 0 个；罩着被重写的单元 k 的下段叶到根恒重写）是纯函数的后置条件，单测判得了。
+//!
+//! 下一步只由「现行那一版这个文件还挂着几个单元」、目标 size 与批大小定（[`next_truncation_step`]），不看这一串是从哪一步续上的：
+//! 续做与不崩溃一路做下来的计划逐步相同（D8（核心索引结构） 已定项 16 第 7 条 ①「计划相等」，截断照它的幂等纪律）。
+
+use crate::address::{CheckpointTxg, DataUnitIndexInFile};
+use crate::extent_tree::{
+    lower_segment_nodes_of_a_file_without_holes, upper_path_of_inode, ExtentLowerNodePosition,
+    ExtentTreeVersion, TailOfAFile,
+};
+use crate::records::InodeRecordTruncationField;
+use crate::transaction::{
+    role_of_extent_upper_node, ExtentTreeNodeOrigin, ExtentTreePlan, TransactionUnit,
+};
+use crate::unit::data_unit_payload_capacity;
+use crate::unlink::UnlinkBatchSize;
+use crate::write_request_split::data_unit_count_of_a_sequential_write;
+
+fn payload_capacity_in_bytes() -> u64 {
+    u64::try_from(data_unit_payload_capacity()).expect("32634")
+}
+
+/// size 为 `size_in_bytes` 的文件要几个数据单元：⌈size ÷ 净荷⌉.max(1)（与写侧切分同一条除法，截到 0 留一个声明长度 0 的单元，K3）。
+#[must_use]
+pub fn data_units_a_size_needs(size_in_bytes: u64) -> u64 {
+    data_unit_count_of_a_sequential_write(size_in_bytes)
+}
+
+/// 第 `unit` 个数据单元在 size 为 `size_in_bytes` 的文件里装几字节：min(size − 起点, 净荷)，起点在 size 之外是 0。
+#[must_use]
+pub fn declared_length_within_a_size(unit: DataUnitIndexInFile, size_in_bytes: u64) -> u64 {
+    let payload = payload_capacity_in_bytes();
+    size_in_bytes
+        .saturating_sub(unit.first_file_byte(payload).0)
+        .min(payload)
+}
+
+/// 截断目标 size 的上限（K12）：⌈N ÷ 净荷⌉ × 净荷装得进 u64。超了在任何写之前拒。
+#[must_use]
+pub fn target_size_rounded_up_to_a_whole_data_unit_fits(target_size_in_bytes: u64) -> bool {
+    let payload = payload_capacity_in_bytes();
+    target_size_in_bytes
+        .div_ceil(payload)
+        .checked_mul(payload)
+        .is_some()
+}
+
+/// 一次改 size 的请求走哪一条（K2、K5、K11、K12）：只看现在的 size 与目标 size，两样在操作期间不变（`.claude/rules/fs-design.md`
+/// 分支的三条约束第 1 条）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum SizeChangeOfARequest {
+    /// N = S：一次只改记录（改动计数、时间）的发布（K12）。
+    SizeUnchanged,
+    /// N > S：一次发布、不立意图（K5）。
+    Extension,
+    /// N < S：意图 + 若干批 + 最后一批（K2、K11：放 0 个单元的也走这一条）。
+    Truncation,
+}
+
+impl SizeChangeOfARequest {
+    #[must_use]
+    pub fn of(size_before_in_bytes: u64, target_size_in_bytes: u64) -> Self {
+        match target_size_in_bytes.cmp(&size_before_in_bytes) {
+            std::cmp::Ordering::Equal => SizeChangeOfARequest::SizeUnchanged,
+            std::cmp::Ordering::Greater => SizeChangeOfARequest::Extension,
+            std::cmp::Ordering::Less => SizeChangeOfARequest::Truncation,
+        }
+    }
+
+    /// 运行时报出走的是哪一条（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。
+    #[must_use]
+    pub const fn name(self) -> &'static str {
+        match self {
+            SizeChangeOfARequest::SizeUnchanged => "size_change_size_unchanged",
+            SizeChangeOfARequest::Extension => "size_change_extension",
+            SizeChangeOfARequest::Truncation => "size_change_truncation",
+        }
+    }
+}
+
+/// 改 size 那次要重写的那一个尾单元：哪一个、重写之后的声明长度（截断取旧净荷的前这么多字节，变长在旧净荷之后补零到这么长）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct TailDataUnitRewrite {
+    pub unit: DataUnitIndexInFile,
+    pub declared_length_in_bytes: u64,
+}
+
+/// 截断意图那次要不要重写尾单元（K3）：截到 N 之后最后一个单元是 k = ⌈N ÷ 净荷⌉.max(1) − 1，它存在、而它在旧 size 里的声明长度
+/// 不等于 N − k × 净荷（截到 0 时 k = 0、留一个声明长度 0 的单元）时重写它。k 不存在（N 落在尾部的洞里）不重写：洞前那个单元是满的。
+#[must_use]
+pub fn tail_data_unit_rewrite_of_a_truncation(
+    data_units_existing: u64,
+    size_before_in_bytes: u64,
+    target_size_in_bytes: u64,
+) -> Option<TailDataUnitRewrite> {
+    let unit = DataUnitIndexInFile(data_units_a_size_needs(target_size_in_bytes) - 1);
+    if unit.0 >= data_units_existing {
+        return None;
+    }
+    let declared_length_in_bytes = declared_length_within_a_size(unit, target_size_in_bytes);
+    (declared_length_within_a_size(unit, size_before_in_bytes) != declared_length_in_bytes)
+        .then_some(TailDataUnitRewrite {
+            unit,
+            declared_length_in_bytes,
+        })
+}
+
+/// 截断意图那次 inode 记录的截断字段（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R2 与第六节用户定案「放了单元才写、
+/// 整个替换」）：这次放单元之前存在的单元数多于 ⌈N ÷ 净荷⌉.max(1)（这次截断要放掉单元）时写成（这次 txg, N），否则照抄上一版
+/// （截到洞里、截在最后一个单元之内都不放单元）。各批与最后一批不碰记录，变长与 N = S 照抄，整份写清零，都不经这里。
+#[must_use]
+pub fn truncation_field_after_the_truncation_intent(
+    truncation_field_before: InodeRecordTruncationField,
+    data_units_existing_before_the_intent: u64,
+    target_size_in_bytes: u64,
+    txg_of_the_intent: CheckpointTxg,
+) -> InodeRecordTruncationField {
+    if data_units_existing_before_the_intent > data_units_a_size_needs(target_size_in_bytes) {
+        InodeRecordTruncationField::written_by_the_truncation_intent_of(
+            txg_of_the_intent,
+            target_size_in_bytes,
+        )
+    } else {
+        truncation_field_before
+    }
+}
+
+/// 变长那次要不要重写尾单元（K5）：最后一个存在的单元声明长度小于 min(N − 起点, 净荷) 时补零重写到那个长度；之后到 N 是洞、不写单元。
+///
+/// # Panics
+/// 一个单元都不存在：第一个文件至少一个数据单元（长度 0 的内容也写一个声明长度 0 的单元）。
+#[must_use]
+pub fn tail_data_unit_rewrite_of_an_extension(
+    data_units_existing: u64,
+    size_before_in_bytes: u64,
+    target_size_in_bytes: u64,
+) -> Option<TailDataUnitRewrite> {
+    let unit = DataUnitIndexInFile(
+        data_units_existing
+            .checked_sub(1)
+            .expect("第一个文件至少一个数据单元"),
+    );
+    let declared_length_in_bytes = declared_length_within_a_size(unit, target_size_in_bytes);
+    (declared_length_within_a_size(unit, size_before_in_bytes) < declared_length_in_bytes)
+        .then_some(TailDataUnitRewrite {
+            unit,
+            declared_length_in_bytes,
+        })
+}
+
+/// 截断意图之后的一步（K2、K11）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum TruncationStep {
+    /// 不是最后一批：从尾部放掉一批，留下前 `data_units_kept` 个；不碰 inode 记录，截断条目留着。
+    Batch { data_units_kept: u64 },
+    /// 最后一批：放掉剩下 size 之外的全部单元（可以 0 个），留下前 `data_units_kept` 个，同一次发布删截断条目。
+    FinalBatch { data_units_kept: u64 },
+}
+
+/// 现行那一版这个文件挂着 `data_units_existing` 个单元、截断目标是 `target_size_in_bytes` 时下一步是哪一批：size 之外还挂着的单元
+/// 多于一批 ⇒ 从尾部放一批；不多于一批 ⇒ 最后一批把它们全部放掉（可以 0 个：N 落在洞里、或截在单元边界上什么都不挂）。
+#[must_use]
+pub fn next_truncation_step(
+    data_units_existing: u64,
+    target_size_in_bytes: u64,
+    batch_size: UnlinkBatchSize,
+) -> TruncationStep {
+    let data_units_kept_at_the_end =
+        data_units_existing.min(data_units_a_size_needs(target_size_in_bytes));
+    let data_units_beyond_the_size = data_units_existing - data_units_kept_at_the_end;
+    let data_units_per_batch = batch_size.data_units_per_batch();
+    if data_units_beyond_the_size <= data_units_per_batch {
+        return TruncationStep::FinalBatch {
+            data_units_kept: data_units_kept_at_the_end,
+        };
+    }
+    TruncationStep::Batch {
+        data_units_kept: data_units_existing - data_units_per_batch,
+    }
+}
+
+/// 从挂着 `data_units_existing` 个单元起走到底的每一步，按先后：最后一步恒是 [`TruncationStep::FinalBatch`]。
+#[must_use]
+pub fn every_truncation_step_from(
+    data_units_existing: u64,
+    target_size_in_bytes: u64,
+    batch_size: UnlinkBatchSize,
+) -> Vec<TruncationStep> {
+    let mut steps = Vec::new();
+    let mut existing = data_units_existing;
+    // 迭代上界：每一批至少放一个单元（批大小至少 1），⌈挂着的 ÷ 批大小⌉ 轮之内走到最后一批。
+    loop {
+        let step = next_truncation_step(existing, target_size_in_bytes, batch_size);
+        steps.push(step);
+        match step {
+            TruncationStep::Batch { data_units_kept } => {
+                assert!(data_units_kept < existing, "每一批至少放一个单元");
+                existing = data_units_kept;
+            }
+            TruncationStep::FinalBatch { .. } => return steps,
+        }
+    }
+}
+
+/// 一个文件改 size 之前与之后的样子（[`plan_the_extent_tree_after_a_size_change`] 的输入）：单元数、这次重写哪一个单元、上段叶条目声明的尾部。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct SizeChangeOfTheFile {
+    pub data_units_before: u64,
+    pub data_units_after: u64,
+    pub rewritten_data_unit: Option<DataUnitIndexInFile>,
+    pub tail_before: TailOfAFile,
+    pub tail_after: TailOfAFile,
+}
+
+/// 截断（意图、各批、最后一批）与变长、只改记录那几次发布之后 inode `inode` 那棵 extent 树（K3：截断的 extent 计划自己写，
+/// 不调 `unlink::plan_the_extent_tree_after_a_deletion_batch`）：下段按这次之后的单元数排，一个节点照抄的条件是它罩着的、存在的单元
+/// 两版一样（罩的单元全在留下那一段里，或这次单元数没变）**而且**不罩着这次重写的那个单元——罩着被重写单元 k 的下段叶到根恒重写，
+/// 留下的单元数正在叶边界上（144 的倍数）时也一样；被截掉那一段里的节点换下。只剩一个单元时不建下段（内联）。上段这个文件那一串
+/// 在单元数变了、有单元重写、或上段叶条目的标签变了（尾部有没有洞）时重写，否则整棵照抄、什么都不换下。
+///
+/// # Panics
+/// 这次之后的单元数是 0 或多于之前的：截断留至少一个单元、变长不加单元。
+#[must_use]
+pub fn plan_the_extent_tree_after_a_size_change(
+    previous: &ExtentTreeVersion,
+    inode: u64,
+    change: SizeChangeOfTheFile,
+) -> ExtentTreePlan {
+    let SizeChangeOfTheFile {
+        data_units_before,
+        data_units_after,
+        rewritten_data_unit,
+        tail_before,
+        tail_after,
+    } = change;
+    assert!(
+        (1..=data_units_before).contains(&data_units_after),
+        "截断留至少一个单元、变长不加单元"
+    );
+    let covers_the_rewritten_unit = |position: &ExtentLowerNodePosition| {
+        let (first_data_unit, last_data_unit) = position.data_unit_range();
+        rewritten_data_unit.is_some_and(|unit| (first_data_unit..=last_data_unit).contains(&unit.0))
+    };
+    let covers_the_same_existing_units = |position: &ExtentLowerNodePosition| {
+        let (_, last_data_unit) = position.data_unit_range();
+        last_data_unit < data_units_after || data_units_after == data_units_before
+    };
+    let lower_nodes: Vec<(ExtentLowerNodePosition, ExtentTreeNodeOrigin)> =
+        lower_segment_nodes_of_a_file_without_holes(data_units_after)
+            .into_iter()
+            .map(|position| {
+                if covers_the_same_existing_units(&position)
+                    && !covers_the_rewritten_unit(&position)
+                {
+                    assert!(
+                        previous.lower_pointer_of(position).is_some(),
+                        "罩的存在单元两版一样的节点，上一版（单元不更少）同一个位置上必有"
+                    );
+                    (
+                        position,
+                        ExtentTreeNodeOrigin::CarriedFromThePreviousVersion,
+                    )
+                } else {
+                    (position, ExtentTreeNodeOrigin::RewrittenThisPublish)
+                }
+            })
+            .collect();
+    let upper_path_is_rewritten = rewritten_data_unit.is_some()
+        || data_units_after != data_units_before
+        || tail_before != tail_after;
+    let upper_root_level = previous.upper_root().level;
+    let path_of_the_file = upper_path_of_inode(inode, upper_root_level);
+    let upper_nodes = previous
+        .upper_nodes
+        .iter()
+        .map(|(position, _)| {
+            let origin = if upper_path_is_rewritten && path_of_the_file.contains(position) {
+                ExtentTreeNodeOrigin::RewrittenThisPublish
+            } else {
+                ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
+            };
+            (*position, origin)
+        })
+        .collect();
+    let carried_lower: Vec<ExtentLowerNodePosition> = lower_nodes
+        .iter()
+        .filter(|(_, origin)| *origin == ExtentTreeNodeOrigin::CarriedFromThePreviousVersion)
+        .map(|(position, _)| *position)
+        .collect();
+    let replaced_previous_roles = previous
+        .lower_nodes
+        .iter()
+        .filter(|(position, _)| !carried_lower.contains(position))
+        .map(|(position, _)| TransactionUnit::ExtentLowerNode(*position))
+        .chain(
+            previous
+                .upper_nodes
+                .iter()
+                .filter(|(position, _)| {
+                    upper_path_is_rewritten && path_of_the_file.contains(position)
+                })
+                .map(|(position, _)| role_of_extent_upper_node(*position, upper_root_level)),
+        )
+        .collect();
+    ExtentTreePlan {
+        lower_nodes,
+        upper_nodes,
+        replaced_previous_roles,
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::extent_tree::ExtentUpperNodePosition;
+    use crate::pointer::NodePointer;
+    use crate::transaction::FIRST_INODE_NUMBER;
+    use crate::unlink::UnlinkBatchDataUnitsCappedForTests;
+
+    const PAYLOAD_IN_BYTES: u64 = 32634;
+
+    fn capped(data_units: u64) -> UnlinkBatchSize {
+        UnlinkBatchSize::CappedForTests(
+            UnlinkBatchDataUnitsCappedForTests::new(data_units).expect("1..=144"),
+        )
+    }
+
+    fn extent_tree_of_a_file_without_holes(data_units: u64) -> ExtentTreeVersion {
+        ExtentTreeVersion {
+            upper_nodes: vec![(
+                ExtentUpperNodePosition { level: 0, index: 0 },
+                NodePointer::empty_root(),
+            )],
+            lower_nodes: lower_segment_nodes_of_a_file_without_holes(data_units)
+                .into_iter()
+                .map(|position| (position, NodePointer::empty_root()))
+                .collect(),
+        }
+    }
+
+    /// 判决 m4-trunc-r2 R2 与第六节用户定案：截断意图那次存在的单元数多于 ⌈N ÷ 净荷⌉.max(1) 才写（这次 txg, N）、整个替换，否则照抄上一版。
+    /// 期望值按条款手算：三个单元截到单元 0 中间（要一个，放两个）写；截到 0（要一个）写成 T ≠ 0、N_w = 0；截在最后一个单元之内（三个要三个）、
+    /// 截到洞里（一个单元的文件变长到三个净荷再截到两个净荷：要两个，有一个）照抄；恰好多一个写、恰好相等照抄（`>` 的边界）；
+    /// 上一版写过的字段在照抄时原样留着，写的时候整个换掉、不取 min。
+    #[test]
+    fn the_truncation_intent_writes_the_field_only_when_it_releases_data_units_and_otherwise_carries_it(
+    ) {
+        let txg_of_the_intent = CheckpointTxg(9);
+        let written_before = InodeRecordTruncationField::written_by_the_truncation_intent_of(
+            CheckpointTxg(5),
+            3 * PAYLOAD_IN_BYTES,
+        );
+        let written_now = |size| {
+            InodeRecordTruncationField::written_by_the_truncation_intent_of(txg_of_the_intent, size)
+        };
+        let cases: [(
+            InodeRecordTruncationField,
+            u64,
+            u64,
+            InodeRecordTruncationField,
+            &str,
+        ); 7] = [
+            (
+                InodeRecordTruncationField::NOT_WRITTEN,
+                3,
+                100,
+                written_now(100),
+                "三个单元截到单元 0 中间：放两个，写",
+            ),
+            (
+                InodeRecordTruncationField::NOT_WRITTEN,
+                3,
+                0,
+                written_now(0),
+                "截到 0：放两个，写成 T = 9、N_w = 0",
+            ),
+            (
+                InodeRecordTruncationField::NOT_WRITTEN,
+                3,
+                2 * PAYLOAD_IN_BYTES + 1,
+                InodeRecordTruncationField::NOT_WRITTEN,
+                "截在最后一个单元之内：不放单元，照抄",
+            ),
+            (
+                InodeRecordTruncationField::NOT_WRITTEN,
+                1,
+                2 * PAYLOAD_IN_BYTES,
+                InodeRecordTruncationField::NOT_WRITTEN,
+                "截到洞里：要两个、有一个，照抄",
+            ),
+            (
+                InodeRecordTruncationField::NOT_WRITTEN,
+                3,
+                2 * PAYLOAD_IN_BYTES,
+                written_now(2 * PAYLOAD_IN_BYTES),
+                "三个截到两个净荷整：恰好多一个，写",
+            ),
+            (
+                written_before,
+                2,
+                2 * PAYLOAD_IN_BYTES,
+                written_before,
+                "两个截到两个净荷整：恰好相等，照抄上一版写过的那一份",
+            ),
+            (
+                written_before,
+                3,
+                100,
+                written_now(100),
+                "上一版写过：这次放单元，整个换成这次的（txg, N），不取 min",
+            ),
+        ];
+        for (before, data_units_existing, target_size, expected, what) in cases {
+            assert_eq!(
+                truncation_field_after_the_truncation_intent(
+                    before,
+                    data_units_existing,
+                    target_size,
+                    txg_of_the_intent
+                ),
+                expected,
+                "{what}"
+            );
+        }
+    }
+
+    fn lower(level: u8, index: u64) -> ExtentLowerNodePosition {
+        ExtentLowerNodePosition { level, index }
+    }
+
+    /// 截断意图那次的计划：单元数不变、只重写尾单元 k（这里 k 由调用方给），尾部没有洞。
+    fn intent_rewriting(data_units: u64, unit: u64) -> ExtentTreePlan {
+        plan_the_extent_tree_after_a_size_change(
+            &extent_tree_of_a_file_without_holes(data_units),
+            FIRST_INODE_NUMBER,
+            SizeChangeOfTheFile {
+                data_units_before: data_units,
+                data_units_after: data_units,
+                rewritten_data_unit: Some(DataUnitIndexInFile(unit)),
+                tail_before: TailOfAFile::NoHoleAfterTheLastDataUnit,
+                tail_after: TailOfAFile::NoHoleAfterTheLastDataUnit,
+            },
+        )
+    }
+
+    /// 三个单元截到单元 0 中间（k = 0）：重写单元 0，声明长度 N；截到单元 1 中间（k = 1）：重写单元 1，声明长度 N − 净荷；
+    /// 截在单元边界上（N = 净荷）：单元 0 原来就满，不重写；截到 0：留单元 0、声明长度 0。
+    #[test]
+    fn the_truncation_intent_rewrites_the_unit_holding_the_new_end_unless_it_already_ends_there() {
+        let size_before = 2 * PAYLOAD_IN_BYTES + PAYLOAD_IN_BYTES / 2;
+        assert_eq!(
+            tail_data_unit_rewrite_of_a_truncation(3, size_before, 100),
+            Some(TailDataUnitRewrite {
+                unit: DataUnitIndexInFile(0),
+                declared_length_in_bytes: 100
+            })
+        );
+        assert_eq!(
+            tail_data_unit_rewrite_of_a_truncation(3, size_before, PAYLOAD_IN_BYTES + 7),
+            Some(TailDataUnitRewrite {
+                unit: DataUnitIndexInFile(1),
+                declared_length_in_bytes: 7
+            })
+        );
+        assert_eq!(
+            tail_data_unit_rewrite_of_a_truncation(3, size_before, PAYLOAD_IN_BYTES),
+            None,
+            "截在单元边界上：单元 0 原来就装满一个净荷"
+        );
+        assert_eq!(
+            tail_data_unit_rewrite_of_a_truncation(3, size_before, 0),
+            Some(TailDataUnitRewrite {
+                unit: DataUnitIndexInFile(0),
+                declared_length_in_bytes: 0
+            }),
+            "截到 0 留一个声明长度 0 的单元"
+        );
+        assert_eq!(
+            tail_data_unit_rewrite_of_a_truncation(
+                1,
+                3 * PAYLOAD_IN_BYTES,
+                2 * PAYLOAD_IN_BYTES - 1
+            ),
+            None,
+            "截到洞里：单元 1 不存在，不重写"
+        );
+    }
+
+    /// 变长：最后一个存在的单元补零到 min(N − 起点, 净荷)；原来就满的不重写。
+    #[test]
+    fn the_extension_rewrites_the_last_existing_unit_only_when_it_is_shorter_than_the_new_size_reaches(
+    ) {
+        assert_eq!(
+            tail_data_unit_rewrite_of_an_extension(1, 0, 10),
+            Some(TailDataUnitRewrite {
+                unit: DataUnitIndexInFile(0),
+                declared_length_in_bytes: 10
+            }),
+            "size 0 的文件变长到单元内"
+        );
+        assert_eq!(
+            tail_data_unit_rewrite_of_an_extension(1, 10, 3 * PAYLOAD_IN_BYTES),
+            Some(TailDataUnitRewrite {
+                unit: DataUnitIndexInFile(0),
+                declared_length_in_bytes: PAYLOAD_IN_BYTES
+            }),
+            "跨单元：单元 0 补满，之后是洞"
+        );
+        assert_eq!(
+            tail_data_unit_rewrite_of_an_extension(2, 2 * PAYLOAD_IN_BYTES, 5 * PAYLOAD_IN_BYTES),
+            None,
+            "最后一个单元原来就满"
+        );
+    }
+
+    /// 批大小压成 1：挂着五个、截到一个半单元（留两个）⇒ 放到四个、放到三个、最后一批放到两个；截到洞里或边界上什么都不挂 ⇒ 只有一个放 0 个的最后一批。
+    #[test]
+    fn truncation_batches_release_from_the_tail_and_the_final_batch_may_release_nothing() {
+        assert_eq!(
+            every_truncation_step_from(5, PAYLOAD_IN_BYTES + 1, capped(1)),
+            vec![
+                TruncationStep::Batch { data_units_kept: 4 },
+                TruncationStep::Batch { data_units_kept: 3 },
+                TruncationStep::FinalBatch { data_units_kept: 2 },
+            ]
+        );
+        assert_eq!(
+            every_truncation_step_from(2, 2 * PAYLOAD_IN_BYTES, capped(1)),
+            vec![TruncationStep::FinalBatch { data_units_kept: 2 }],
+            "截在最后一个单元末尾之后（落在洞里或边界上）：放 0 个"
+        );
+        assert_eq!(
+            every_truncation_step_from(3, 0, UnlinkBatchSize::OneExtentLowerLeaf),
+            vec![TruncationStep::FinalBatch { data_units_kept: 1 }],
+            "截到 0 留一个单元"
+        );
+    }
+
+    /// k = 0：单元 0 在下段第 0 片叶里，那片叶与下段根、上段那一串重写；别的叶照抄。
+    #[test]
+    fn rewriting_unit_zero_rewrites_the_first_lower_leaf_and_its_ancestors() {
+        let plan = intent_rewriting(300, 0);
+        assert_eq!(
+            plan.lower_nodes,
+            vec![
+                (lower(0, 0), ExtentTreeNodeOrigin::RewrittenThisPublish),
+                (
+                    lower(0, 1),
+                    ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
+                ),
+                (
+                    lower(0, 2),
+                    ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
+                ),
+                (lower(1, 0), ExtentTreeNodeOrigin::RewrittenThisPublish),
+            ]
+        );
+        assert_eq!(
+            plan.rewritten_roles(),
+            vec![
+                TransactionUnit::ExtentLowerNode(lower(0, 0)),
+                TransactionUnit::ExtentLowerNode(lower(1, 0)),
+                TransactionUnit::ExtentRoot,
+            ]
+        );
+    }
+
+    /// k = 1 落在只有一片下段叶的三单元文件里：那片叶（兼下段根）与上段那一串重写。
+    #[test]
+    fn rewriting_unit_one_of_a_three_unit_file_rewrites_its_only_lower_leaf() {
+        let plan = intent_rewriting(3, 1);
+        assert_eq!(
+            plan.rewritten_roles(),
+            vec![
+                TransactionUnit::ExtentLowerNode(lower(0, 0)),
+                TransactionUnit::ExtentRoot
+            ]
+        );
+        assert_eq!(
+            plan.replaced_previous_roles,
+            vec![
+                TransactionUnit::ExtentLowerNode(lower(0, 0)),
+                TransactionUnit::ExtentRoot
+            ]
+        );
+    }
+
+    /// 留下 144 个单元、单元 k = 143 正在第 0 片叶的末尾（叶边界上）：意图那次重写第 0 片叶与根；之后那一批截掉第 1 片叶时
+    /// 第 0 片叶照抄（单元 143 意图那次已经重写、那片叶已经指着它），它成了下段根。
+    #[test]
+    fn keeping_one_hundred_and_forty_four_units_rewrites_the_leaf_holding_unit_one_hundred_and_forty_three(
+    ) {
+        let intent = intent_rewriting(150, 143);
+        assert_eq!(
+            intent.rewritten_roles(),
+            vec![
+                TransactionUnit::ExtentLowerNode(lower(0, 0)),
+                TransactionUnit::ExtentLowerNode(lower(1, 0)),
+                TransactionUnit::ExtentRoot,
+            ],
+            "罩着被重写单元 k 的叶到根恒并进重写集"
+        );
+        let final_batch = plan_the_extent_tree_after_a_size_change(
+            &extent_tree_of_a_file_without_holes(150),
+            FIRST_INODE_NUMBER,
+            SizeChangeOfTheFile {
+                data_units_before: 150,
+                data_units_after: 144,
+                rewritten_data_unit: None,
+                tail_before: TailOfAFile::NoHoleAfterTheLastDataUnit,
+                tail_after: TailOfAFile::NoHoleAfterTheLastDataUnit,
+            },
+        );
+        assert_eq!(
+            final_batch.lower_nodes,
+            vec![(
+                lower(0, 0),
+                ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
+            )]
+        );
+        assert_eq!(
+            final_batch.rewritten_roles(),
+            vec![TransactionUnit::ExtentRoot]
+        );
+    }
+
+    /// 留下 288 个单元、k = 287 在第 1 片叶的末尾：意图那次重写第 1 片叶与下段根，第 0、2 片叶照抄。
+    #[test]
+    fn keeping_two_hundred_and_eighty_eight_units_rewrites_the_leaf_holding_unit_two_hundred_and_eighty_seven(
+    ) {
+        let plan = intent_rewriting(300, 287);
+        assert_eq!(
+            plan.lower_nodes,
+            vec![
+                (
+                    lower(0, 0),
+                    ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
+                ),
+                (lower(0, 1), ExtentTreeNodeOrigin::RewrittenThisPublish),
+                (
+                    lower(0, 2),
+                    ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
+                ),
+                (lower(1, 0), ExtentTreeNodeOrigin::RewrittenThisPublish),
+            ]
+        );
+    }
+
+    /// 没有单元重写、单元数与尾部都没变（最后一批放 0 个）：整棵照抄、什么都不换下。
+    #[test]
+    fn a_final_batch_releasing_nothing_carries_the_whole_extent_tree() {
+        let plan = plan_the_extent_tree_after_a_size_change(
+            &extent_tree_of_a_file_without_holes(3),
+            FIRST_INODE_NUMBER,
+            SizeChangeOfTheFile {
+                data_units_before: 3,
+                data_units_after: 3,
+                rewritten_data_unit: None,
+                tail_before: TailOfAFile::NoHoleAfterTheLastDataUnit,
+                tail_after: TailOfAFile::NoHoleAfterTheLastDataUnit,
+            },
+        );
+        assert!(plan.rewritten_roles().is_empty());
+        assert!(plan.replaced_previous_roles.is_empty());
+    }
+
+    /// K12：⌈N ÷ 净荷⌉ × 净荷装得进 u64 的上限；N 走哪一条只看 N 与 S。
+    #[test]
+    fn the_target_size_limit_and_the_three_branches_of_a_size_change() {
+        assert!(target_size_rounded_up_to_a_whole_data_unit_fits(0));
+        let largest_whole_units = u64::MAX / PAYLOAD_IN_BYTES * PAYLOAD_IN_BYTES;
+        assert!(target_size_rounded_up_to_a_whole_data_unit_fits(
+            largest_whole_units
+        ));
+        assert!(!target_size_rounded_up_to_a_whole_data_unit_fits(
+            largest_whole_units + 1
+        ));
+        assert_eq!(
+            SizeChangeOfARequest::of(5, 5),
+            SizeChangeOfARequest::SizeUnchanged
+        );
+        assert_eq!(
+            SizeChangeOfARequest::of(5, 6),
+            SizeChangeOfARequest::Extension
+        );
+        assert_eq!(
+            SizeChangeOfARequest::of(5, 4),
+            SizeChangeOfARequest::Truncation
+        );
+    }
+}
diff --git a/crates/singlefs-core/src/write_request_split.rs b/crates/singlefs-core/src/write_request_split.rs
index 78ce62a..abeaa6e 100644
--- a/crates/singlefs-core/src/write_request_split.rs
+++ b/crates/singlefs-core/src/write_request_split.rs
@@ -61,29 +61,60 @@ pub fn data_unit_count_of_a_sequential_write(content_length_in_bytes: u64) -> u6
     content_length_in_bytes.div_ceil(payload_capacity).max(1)
 }
 
-/// 读侧看一个 inode 在 extent 树上段叶里的那一条（D8（核心索引结构） 已定项 14）：指没指着数据单元。封闭集合，`match` 不写通配臂。
+/// 读侧看一个 inode 在 extent 树上段叶里的那一条（D8（核心索引结构） 已定项 14；标签 3、4 与「标签 0 配 size > 0」是判决
+/// `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5 加的）。封闭集合，`match` 不写通配臂。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum DataUnitsOfTheInode {
-    /// 上段叶里没有它的条目，或条目标签 0：建出来之后一次都没写过内容（`transaction::publish_new_inodes` 建的 inode
-    /// 长度 0、数据单元与 extent 树都不动）。
+    /// 上段叶里没有它的条目：建出来之后一次都没写过内容（`transaction::publish_new_inodes` 建的 inode 长度 0、数据单元与
+    /// extent 树都不动）。
     NoneWritten,
-    /// 条目指着下段根或一个内联的数据指针：写过内容。
+    /// 条目标签 1、2：指着下段根或一个内联的数据指针，最后一个单元之后没有洞。
     SomeWritten,
+    /// 条目标签 3、4、0：最后一个存在的单元之后到 size 是洞（标签 0 一个单元都没有、整个文件是洞）。
+    HoleAfterTheLastDataUnit,
 }
 
-/// 读侧按 inode 的文件大小该看到几条 extent 记录。写过内容的与写侧同一条除法（[`data_unit_count_of_a_sequential_write`]，
-/// 长度 0 的内容也写了一个声明长度 0 的单元）；一次都没写过的长度 0 ⇒ 0 条。一次都没写过而长度不是 0 的照除法给数，
-/// 与它的 0 条记录对不上，读侧报不一致。
-#[must_use]
-pub fn data_unit_count_implied_by_the_file_size(
+/// 读侧的单元数对不上 size：交回 size 按净荷容量算出来的单元数（报错用）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct DataUnitCountDoesNotMatchTheFileSize {
+    pub data_units_implied_by_the_file_size: u64,
+}
+
+/// 读侧：盘上这个 inode 有 `data_units_on_disk` 个数据单元（从 0 起连号）、size 是 `file_size_in_bytes` 时，size 之内要读几个单元
+/// （之后到 size 是洞，读出 0、不读盘）。size 按净荷容量算出的单元数是 ⌈size ÷ 净荷⌉.max(1)，与写侧切分同一条除法
+/// （[`data_unit_count_of_a_sequential_write`]）：
+/// - 没有条目：长度 0、0 个单元；
+/// - 标签 1、2：单元数不少于它（多出来的是截断中间态里 size 之外还挂着的单元，判决 K2「读者容忍」：不读、不判声明长度）；
+/// - 标签 3、4、0：单元数少于它（相等就该写成 1、2）。
+///
+/// # Errors
+/// 三样对不上 ⇒ [`DataUnitCountDoesNotMatchTheFileSize`]（没有条目而 size 不是 0 时带除法给的数）。
+pub fn data_units_read_within_the_file_size(
     file_size_in_bytes: u64,
+    data_units_on_disk: u64,
     data_units_of_the_inode: DataUnitsOfTheInode,
-) -> u64 {
+) -> Result<u64, DataUnitCountDoesNotMatchTheFileSize> {
+    let data_units_implied_by_the_file_size =
+        data_unit_count_of_a_sequential_write(file_size_in_bytes);
+    let refused = Err(DataUnitCountDoesNotMatchTheFileSize {
+        data_units_implied_by_the_file_size,
+    });
     match data_units_of_the_inode {
-        DataUnitsOfTheInode::NoneWritten if file_size_in_bytes == 0 => 0,
-        DataUnitsOfTheInode::NoneWritten | DataUnitsOfTheInode::SomeWritten => {
-            data_unit_count_of_a_sequential_write(file_size_in_bytes)
+        DataUnitsOfTheInode::NoneWritten if file_size_in_bytes == 0 && data_units_on_disk == 0 => {
+            Ok(0)
+        }
+        DataUnitsOfTheInode::NoneWritten => refused,
+        DataUnitsOfTheInode::SomeWritten
+            if data_units_on_disk >= data_units_implied_by_the_file_size =>
+        {
+            Ok(data_units_implied_by_the_file_size)
         }
+        DataUnitsOfTheInode::HoleAfterTheLastDataUnit
+            if data_units_on_disk < data_units_implied_by_the_file_size =>
+        {
+            Ok(data_units_on_disk)
+        }
+        DataUnitsOfTheInode::SomeWritten | DataUnitsOfTheInode::HoleAfterTheLastDataUnit => refused,
     }
 }
 
@@ -283,26 +314,56 @@ mod tests {
         assert_eq!(rejoined, content);
     }
 
-    /// 读侧的单元数：一次都没写过的 inode 长度 0 ⇒ 0 条；写过的长度 0 ⇒ 1 条（写侧的下界）；长度不是 0 的两种都按除法。
+    /// 读侧的单元数：没有条目的长度 0 ⇒ 0 个；标签 1、2 长度 0 ⇒ 1 个（写侧的下界），多出来的（截断中间态）只读 size 之内的；
+    /// 标签 3、4、0 少于除法给的数、读存在的那几个，不少于就拒；没有条目而长度不是 0 拒。
     #[test]
-    fn a_never_written_inode_of_length_zero_implies_no_data_unit_and_a_written_one_implies_one() {
+    fn the_reader_reads_the_data_units_within_the_size_and_refuses_counts_the_tag_does_not_allow() {
+        let refused = |implied: u64| {
+            Err(DataUnitCountDoesNotMatchTheFileSize {
+                data_units_implied_by_the_file_size: implied,
+            })
+        };
         assert_eq!(
-            data_unit_count_implied_by_the_file_size(0, DataUnitsOfTheInode::NoneWritten),
-            0
+            data_units_read_within_the_file_size(0, 0, DataUnitsOfTheInode::NoneWritten),
+            Ok(0)
         );
         assert_eq!(
-            data_unit_count_implied_by_the_file_size(0, DataUnitsOfTheInode::SomeWritten),
-            1
+            data_units_read_within_the_file_size(32_635, 0, DataUnitsOfTheInode::NoneWritten),
+            refused(2)
+        );
+        let without_a_hole = DataUnitsOfTheInode::SomeWritten;
+        assert_eq!(
+            data_units_read_within_the_file_size(0, 1, without_a_hole),
+            Ok(1)
+        );
+        assert_eq!(
+            data_units_read_within_the_file_size(32_635, 2, without_a_hole),
+            Ok(2),
+            "净荷容量加一个字节是两个单元"
+        );
+        assert_eq!(
+            data_units_read_within_the_file_size(32_635, 5, without_a_hole),
+            Ok(2),
+            "截断中间态：size 之外还挂着三个，只读 size 之内的两个"
+        );
+        assert_eq!(
+            data_units_read_within_the_file_size(32_635, 1, without_a_hole),
+            refused(2)
+        );
+        let with_a_hole = DataUnitsOfTheInode::HoleAfterTheLastDataUnit;
+        assert_eq!(
+            data_units_read_within_the_file_size(32_635, 1, with_a_hole),
+            Ok(1)
+        );
+        assert_eq!(
+            data_units_read_within_the_file_size(32_635, 0, with_a_hole),
+            Ok(0),
+            "标签 0 配 size > 0：整个文件是洞"
+        );
+        assert_eq!(
+            data_units_read_within_the_file_size(32_635, 2, with_a_hole),
+            refused(2),
+            "单元数与除法相等却声明有洞"
         );
-        for data_units_of_the_inode in [
-            DataUnitsOfTheInode::NoneWritten,
-            DataUnitsOfTheInode::SomeWritten,
-        ] {
-            assert_eq!(
-                data_unit_count_implied_by_the_file_size(32_635, data_units_of_the_inode),
-                2,
-                "{data_units_of_the_inode:?}：净荷容量加一个字节是两个单元"
-            );
-        }
     }
 }
diff --git a/crates/singlefs-format/src/lib.rs b/crates/singlefs-format/src/lib.rs
index b1247e5..68eb37d 100644
--- a/crates/singlefs-format/src/lib.rs
+++ b/crates/singlefs-format/src/lib.rs
@@ -200,9 +200,6 @@ pub const LOGGED_OPERATIONS_ENTRY_BYTES: u64 = 44;
 /// format-const: LOGGED_OPERATIONS_ENTRIES_PER_NODE
 pub const LOGGED_OPERATIONS_ENTRIES_PER_NODE: u64 = 369;
 
-/// 新池新建文件这次发布写出的记账行数（D5（快照 / 空间记账机制） 已定项 8，2026-09-14 用户定案 15 行）。
-pub const NEW_POOL_FILE_CREATION_ACCOUNTING_ROWS: u64 = 15;
-
 /// 树表条目：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + previous_snapshot_txg 8 + 诞生 txg 8 + 头 ID 8 + 预留 76（D8（核心索引结构） 已定项 8；重排与头 ID 2026-09-14 用户定案，加宽到 200 是 2026-09-16 用户定案）。format-const: TREE_TABLE_ENTRY_BYTES
 pub const TREE_TABLE_ENTRY_BYTES: u64 = 200;
 
```

## code-round-upper-bound.diff（sha256 93f3246434a9a134…）

```diff
diff --git a/crates/singlefs-core/src/logged_operations.rs b/crates/singlefs-core/src/logged_operations.rs
new file mode 100644
index 0000000..f2ad02b
--- /dev/null
+++ b/crates/singlefs-core/src/logged_operations.rs
@@ -0,0 +1,527 @@
+//! logged_ops 树的条目（D8（核心索引结构） 已定项 5 的意图树；字段表是判决 `research/prompts/m4-gc-r1-main-verification.md`
+//! 第三节 G1「单树、定宽联合条目」）：key 10 = 种类 2 + 签发 txg 8，value 34 定宽、按种类解，条目 44，一个码 2 节点装 369 条。
+//! 第一版装两种意图：整理意图（种类 1）：R（设备 4 + 起始槽 6 + 槽数 2）、`c0` 8、`segs0` 8、补零 6；截断意图（种类 2，判决
+//! `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1）：inode 号 8、对象出生代 8、目标 size 8、补零 10。别的种类码保留，读者拒收
+//! （照树表条目 flags「未知位一律拒收」的政策，D8（核心索引结构） 已定项 8）。
+//!
+//! **不存续做点、不存暂停标志**（判决 G1，攻方 F1 / F4）：暂停就是意图留着不推进；续做每批照目标态从 R 起点重扫，
+//! 删意图之前对整个 R 全量核目标态。
+//!
+//! 和 `crate::lineage_tree`、`crate::tombstone_tree` 一样只算、不发一个写：这里只有字段表的读写，签发、推进、删意图归发布入口。
+
+use singlefs_format::{
+    CLUSTER_SEGMENT_SLOTS, LOGGED_OPERATIONS_ENTRY_BYTES, LOGGED_OPERATIONS_KEY_BYTES,
+};
+
+use crate::address::{CheckpointTxg, DeviceIdentity, InodeNumber, SlotNumber};
+use crate::bytes::{ByteReader, ByteWriter};
+use crate::unit::index_node_entry_capacity;
+
+/// logged_ops 树的 key 宽：种类 2 + 签发 txg 8（判决 G1）。码 2 节点头里的自述 key 宽要与它相符。
+pub const LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES: usize = 10;
+
+/// logged_ops 条目的种类码：1 = 整理（判决 G1）；2 = 截断（[`LOGGED_OPERATION_KIND_TRUNCATION`]）。别的码保留：读者拒收，不当未来版本的条目猜着读。
+pub const LOGGED_OPERATION_KIND_COMPACTION: u16 = 1;
+
+/// logged_ops 条目的种类码：2 = 截断（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1）。
+pub const LOGGED_OPERATION_KIND_TRUNCATION: u16 = 2;
+
+/// 整理意图 value 末尾的补零：34 − (4 + 6 + 2 + 8 + 8)。读者遇到非零一律拒收（同树表条目预留位的政策）。
+const COMPACTION_INTENT_PADDING_BYTES: usize = 6;
+
+/// 截断意图 value 末尾的补零：34 − (8 + 8 + 8)。读者遇到非零一律拒收（同整理意图的补零）。
+const TRUNCATION_INTENT_PADDING_BYTES: usize = 10;
+
+/// 槽号在盘上占 6 字节（D3（空间分配） 已定项 7）：起始槽要小于它才写得下。
+const SIX_BYTE_SLOT_NUMBER_LIMIT: u64 = 1 << 48;
+
+/// logged_ops 条目的种类。封闭集合：加一种意图就加一个成员，`match` 不写通配臂。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
+pub enum LoggedOperationKind {
+    /// 整理意图（D26（后台整理与放置回收） 已定项 2；提示类，D8（核心索引结构） 已定项 5）。
+    Compaction,
+    /// 截断意图（承诺类，D8（核心索引结构） 已定项 5；判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1）。
+    Truncation,
+}
+
+impl LoggedOperationKind {
+    /// 写进 key 前 2 字节的种类码。
+    #[must_use]
+    pub const fn code(self) -> u16 {
+        match self {
+            LoggedOperationKind::Compaction => LOGGED_OPERATION_KIND_COMPACTION,
+            LoggedOperationKind::Truncation => LOGGED_OPERATION_KIND_TRUNCATION,
+        }
+    }
+
+    /// 盘上读到的种类码认不认得：认不得的（保留码）交回 `None`。
+    #[must_use]
+    pub const fn of_code(kind_code: u16) -> Option<Self> {
+        match kind_code {
+            LOGGED_OPERATION_KIND_COMPACTION => Some(LoggedOperationKind::Compaction),
+            LOGGED_OPERATION_KIND_TRUNCATION => Some(LoggedOperationKind::Truncation),
+            _reserved_kind_code => None,
+        }
+    }
+}
+
+/// 整理意图的区域 R：一块盘上一个 64 槽对齐的聚簇段（判决 G1：起始槽 64 对齐、槽数 = 64；第一版两盘同槽时 R 罩住每块盘上的同号槽，
+/// 攻方 K-n）。只从 [`CompactionRegion::of_one_cluster_segment`] 进来，所以槽数不存：恒是 [`CLUSTER_SEGMENT_SLOTS`]。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
+pub struct CompactionRegion {
+    device: DeviceIdentity,
+    start_slot: SlotNumber,
+}
+
+/// 造不出一个整理区域的两种起始槽。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CompactionRegionRefusal {
+    /// 起始槽不是 64 的倍数：R 不对齐（判决 G1「R 的解码拒不对齐的值」）。
+    StartOffAClusterSegmentBoundary { start_slot: SlotNumber },
+    /// 起始槽装不进盘上 6 字节的槽号。
+    StartBeyondTheSixByteSlotNumber { start_slot: SlotNumber },
+}
+
+impl CompactionRegion {
+    /// `device` 上从 `start_slot` 起的那一个聚簇段。
+    ///
+    /// # Errors
+    /// 起始槽不是 64 的倍数 ⇒ `StartOffAClusterSegmentBoundary`；装不进 6 字节 ⇒ `StartBeyondTheSixByteSlotNumber`。
+    pub fn of_one_cluster_segment(
+        device: DeviceIdentity,
+        start_slot: SlotNumber,
+    ) -> Result<Self, CompactionRegionRefusal> {
+        if !start_slot.0.is_multiple_of(CLUSTER_SEGMENT_SLOTS) {
+            return Err(CompactionRegionRefusal::StartOffAClusterSegmentBoundary { start_slot });
+        }
+        if start_slot.0 >= SIX_BYTE_SLOT_NUMBER_LIMIT {
+            return Err(CompactionRegionRefusal::StartBeyondTheSixByteSlotNumber { start_slot });
+        }
+        Ok(Self { device, start_slot })
+    }
+
+    #[must_use]
+    pub const fn device(self) -> DeviceIdentity {
+        self.device
+    }
+
+    #[must_use]
+    pub const fn start_slot(self) -> SlotNumber {
+        self.start_slot
+    }
+
+    /// 槽数：恒是一个聚簇段。
+    #[must_use]
+    pub const fn slot_count(self) -> u64 {
+        CLUSTER_SEGMENT_SLOTS
+    }
+
+    /// R 之后的第一个槽（开区间的右端）。
+    #[must_use]
+    pub const fn end_slot(self) -> SlotNumber {
+        SlotNumber(self.start_slot.0 + CLUSTER_SEGMENT_SLOTS)
+    }
+
+    /// 这个槽号在不在 R 里（只看槽号：两盘同槽时 R 罩住每块盘上的同号槽）。
+    #[must_use]
+    pub const fn contains_slot(self, slot: SlotNumber) -> bool {
+        slot.0 >= self.start_slot.0 && slot.0 < self.start_slot.0 + CLUSTER_SEGMENT_SLOTS
+    }
+}
+
+/// 一条整理意图（D26（后台整理与放置回收） 已定项 1 / 已定项 2；字段表判决 G1）。三个字段都是值，不是增量
+/// （D8（核心索引结构） 已定项 5 硬要求 1）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct CompactionIntent {
+    pub region: CompactionRegion,
+    /// `c0`：签发那一刻的分配代；目标态是「R 内不存在分配代 ≤ `c0` 且未释放的落点」（D26 已定项 2）。
+    pub allocation_generation_at_issue: CheckpointTxg,
+    /// `segs0`：签发那次发布写下的记账第 11 项（全空聚簇段数）里 R 所在盘的值；只作观测量，停机不靠它（判决 G1 / G4）。
+    pub empty_cluster_segments_at_issue: u64,
+}
+
+/// 一条截断意图（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K2）：把 `inode`（对象出生代 `object_birth`）
+/// 截到 `target_size_in_bytes` 字节。目标是值、不是增量（D8（核心索引结构） 已定项 5 硬要求 1）：意图那次发布已把记录的 size 写成它，
+/// 各批只从尾部放掉 size 之外还挂着的单元，最后一批与删这一条同一次发布。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct TruncationIntent {
+    pub inode: InodeNumber,
+    pub object_birth: CheckpointTxg,
+    pub target_size_in_bytes: u64,
+}
+
+/// logged_ops 条目的 value，按种类分。种类码从成员推出来，key 与 value 对不上的条目造不出来。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum LoggedOperation {
+    Compaction(CompactionIntent),
+    Truncation(TruncationIntent),
+}
+
+impl LoggedOperation {
+    #[must_use]
+    pub const fn kind(&self) -> LoggedOperationKind {
+        match self {
+            LoggedOperation::Compaction(_intent) => LoggedOperationKind::Compaction,
+            LoggedOperation::Truncation(_intent) => LoggedOperationKind::Truncation,
+        }
+    }
+}
+
+/// logged_ops 条目的 key：种类 + 签发 txg。全序按逐字段无符号整数比（D8（核心索引结构） 已定项 11：小端存储不构成 memcmp 序），
+/// 先按种类、再按签发先后。一次发布至多签发一条（判决 G1 e1），所以 key 不带序号。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
+pub struct LoggedOperationKey {
+    pub kind: LoggedOperationKind,
+    pub issued_txg: CheckpointTxg,
+}
+
+impl LoggedOperationKey {
+    #[must_use]
+    pub fn sort_key(&self) -> (u16, u64) {
+        (self.kind.code(), self.issued_txg.0)
+    }
+}
+
+/// logged_ops 树里的一条：签发那次发布的 txg 与它声明的意图。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct LoggedOperationEntry {
+    pub issued_txg: CheckpointTxg,
+    pub operation: LoggedOperation,
+}
+
+/// 读者拒收一条 logged_ops 条目的原因。条目是盘上读来的字节、不是我们的不变量：调用方把它当损坏处置。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum LoggedOperationEntryRefusal {
+    /// 条目宽（码 2 节点头里的盘上字段）不是字段表的 44。
+    WidthIsNotTheFieldTable { entry_bytes: usize },
+    /// 种类码是保留码。
+    UnknownKind { kind_code: u16 },
+    /// 整理意图的 R 起始槽不对齐、或装不进 6 字节。
+    CompactionRegionRefused(CompactionRegionRefusal),
+    /// 整理意图的 R 槽数不是一个聚簇段（64）。
+    CompactionRegionSlotCountIsNotOneClusterSegment { slot_count: u16 },
+    /// 整理意图 value 末尾那 6 字节补零有非零字节。
+    CompactionPaddingNotZero,
+    /// 截断意图 value 末尾那 10 字节补零有非零字节。
+    TruncationPaddingNotZero,
+}
+
+fn entry_bytes() -> usize {
+    usize::try_from(LOGGED_OPERATIONS_ENTRY_BYTES).expect("44")
+}
+
+impl LoggedOperationEntry {
+    #[must_use]
+    pub const fn key(&self) -> LoggedOperationKey {
+        LoggedOperationKey {
+            kind: self.operation.kind(),
+            issued_txg: self.issued_txg,
+        }
+    }
+
+    /// 条目 44：种类 0 | 签发 txg 2 | 整理：R 的设备 10 | R 的起始槽 14 | R 的槽数 20 | `c0` 22 | `segs0` 30 | 补零 38；
+    /// 截断：inode 号 10 | 对象出生代 18 | 目标 size 26 | 补零 34，小端。
+    #[must_use]
+    pub fn to_bytes(&self) -> Vec<u8> {
+        let mut writer = ByteWriter::new(entry_bytes());
+        writer.put_u16(self.operation.kind().code());
+        writer.put_u64(self.issued_txg.0);
+        writer.assert_position(LOGGED_OPERATIONS_KEY_BYTES, "logged_ops 条目的 key");
+        match &self.operation {
+            LoggedOperation::Compaction(intent) => {
+                writer.put_u32(intent.region.device().0);
+                writer.assert_position(14, "整理意图 R 的起始槽");
+                writer.put_six_byte_unsigned(intent.region.start_slot().0);
+                writer.assert_position(20, "整理意图 R 的槽数");
+                writer.put_u16(
+                    u16::try_from(intent.region.slot_count())
+                        .expect("一个聚簇段 64 槽装得进 2 字节"),
+                );
+                writer.assert_position(22, "整理意图的 c0");
+                writer.put_u64(intent.allocation_generation_at_issue.0);
+                writer.assert_position(30, "整理意图的 segs0");
+                writer.put_u64(intent.empty_cluster_segments_at_issue);
+                writer.assert_position(38, "整理意图的补零");
+                writer.skip(COMPACTION_INTENT_PADDING_BYTES);
+            }
+            LoggedOperation::Truncation(intent) => {
+                writer.put_u64(intent.inode.0);
+                writer.assert_position(18, "截断意图的对象出生代");
+                writer.put_u64(intent.object_birth.0);
+                writer.assert_position(26, "截断意图的目标 size");
+                writer.put_u64(intent.target_size_in_bytes);
+                writer.assert_position(34, "截断意图的补零");
+                writer.skip(TRUNCATION_INTENT_PADDING_BYTES);
+            }
+        }
+        writer.assert_position(LOGGED_OPERATIONS_ENTRY_BYTES, "logged_ops 条目");
+        writer.into_bytes()
+    }
+
+    /// logged_ops 条目 44 的读者。
+    ///
+    /// # Errors
+    /// 宽不是 44 ⇒ `WidthIsNotTheFieldTable`；种类码保留 ⇒ `UnknownKind`；整理意图的 R 起始槽不是 64 的倍数 ⇒
+    /// `CompactionRegionRefused`；R 槽数不是 64 ⇒ `CompactionRegionSlotCountIsNotOneClusterSegment`；补零非零 ⇒ `CompactionPaddingNotZero`；
+    /// 截断意图的补零非零 ⇒ `TruncationPaddingNotZero`。
+    pub fn parse(bytes: &[u8]) -> Result<Self, LoggedOperationEntryRefusal> {
+        if bytes.len() != entry_bytes() {
+            return Err(LoggedOperationEntryRefusal::WidthIsNotTheFieldTable {
+                entry_bytes: bytes.len(),
+            });
+        }
+        let mut reader = ByteReader::at(bytes, 0);
+        let kind_code = reader.get_u16();
+        let issued_txg = CheckpointTxg(reader.get_u64());
+        let kind = LoggedOperationKind::of_code(kind_code)
+            .ok_or(LoggedOperationEntryRefusal::UnknownKind { kind_code })?;
+        let operation = match kind {
+            LoggedOperationKind::Compaction => {
+                LoggedOperation::Compaction(parse_compaction_intent(&mut reader)?)
+            }
+            LoggedOperationKind::Truncation => {
+                LoggedOperation::Truncation(parse_truncation_intent(&mut reader)?)
+            }
+        };
+        Ok(Self {
+            issued_txg,
+            operation,
+        })
+    }
+}
+
+/// 整理意图那 34 字节 value（读者已在 key 之后）。
+fn parse_compaction_intent(
+    reader: &mut ByteReader<'_>,
+) -> Result<CompactionIntent, LoggedOperationEntryRefusal> {
+    let device = DeviceIdentity(reader.get_u32());
+    let start_slot = SlotNumber(reader.get_six_byte_unsigned());
+    let slot_count = reader.get_u16();
+    let allocation_generation_at_issue = CheckpointTxg(reader.get_u64());
+    let empty_cluster_segments_at_issue = reader.get_u64();
+    let padding = reader.take(COMPACTION_INTENT_PADDING_BYTES);
+    if u64::from(slot_count) != CLUSTER_SEGMENT_SLOTS {
+        return Err(
+            LoggedOperationEntryRefusal::CompactionRegionSlotCountIsNotOneClusterSegment {
+                slot_count,
+            },
+        );
+    }
+    if padding.iter().any(|byte| *byte != 0) {
+        return Err(LoggedOperationEntryRefusal::CompactionPaddingNotZero);
+    }
+    let region = CompactionRegion::of_one_cluster_segment(device, start_slot)
+        .map_err(LoggedOperationEntryRefusal::CompactionRegionRefused)?;
+    Ok(CompactionIntent {
+        region,
+        allocation_generation_at_issue,
+        empty_cluster_segments_at_issue,
+    })
+}
+
+/// 截断意图那 34 字节 value（读者已在 key 之后）。
+fn parse_truncation_intent(
+    reader: &mut ByteReader<'_>,
+) -> Result<TruncationIntent, LoggedOperationEntryRefusal> {
+    let inode = InodeNumber(reader.get_u64());
+    let object_birth = CheckpointTxg(reader.get_u64());
+    let target_size_in_bytes = reader.get_u64();
+    let padding = reader.take(TRUNCATION_INTENT_PADDING_BYTES);
+    if padding.iter().any(|padding_byte| *padding_byte != 0) {
+        return Err(LoggedOperationEntryRefusal::TruncationPaddingNotZero);
+    }
+    Ok(TruncationIntent {
+        inode,
+        object_birth,
+        target_size_in_bytes,
+    })
+}
+
+/// 一个 logged_ops 码 2 根（层级 0）装得下几条：(16384 − 含预留位的头 135) ÷ 44 的整数部分 = 369（判决 G1「一节点 369 条」）。
+#[must_use]
+pub fn logged_operations_one_root_node_holds() -> usize {
+    index_node_entry_capacity(LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES, entry_bytes())
+}
+
+/// logged_ops 根要装的条目：每条 44 字节（[`LoggedOperationEntry::to_bytes`]），按 key 的全序升序。
+/// logged_ops 第一版只有一个码 2 根、根之下不长层：全池活整理意图至多一条（判决 G1 e1）、活截断意图至多一条（判决 m4-trunc-r1 K1），
+/// 一个根装 369 条。
+///
+/// # Panics
+/// `entries` 不是按 key 严格升序、或多于一个根装得下的：条目由发布路径按「签发加一条、删掉一条」维护，至多一条活整理意图，
+/// 乱序或装不下说明调用方拼错了。
+#[must_use]
+pub fn logged_operations_root_entries(entries: &[LoggedOperationEntry]) -> Vec<Vec<u8>> {
+    assert!(
+        entries
+            .windows(2)
+            .all(|pair| pair[0].key().sort_key() < pair[1].key().sort_key()),
+        "logged_ops 的条目按 key 升序、不重复：发布路径按集合维护它"
+    );
+    assert!(
+        entries.len() <= logged_operations_one_root_node_holds(),
+        "logged_ops 至多一条活整理意图与一条活截断意图，一个根装得下"
+    );
+    entries.iter().map(LoggedOperationEntry::to_bytes).collect()
+}
+
+/// 从一个 logged_ops 根的条目读回意图（条目是盘上读来的字节，逐条照字段表解）。
+///
+/// # Errors
+/// 有一条过不了 [`LoggedOperationEntry::parse`]（宽不是 44、种类码保留、R 不对齐或槽数不是 64、补零非零）、
+/// 几条不按 key 严格升序、整理意图多于一条（全池至多一条，判决 G1 e1）、截断意图多于一条（全池至多一条，判决 m4-trunc-r1 K1）
+/// ⇒ [`LoggedOperationsRootRefusal`]。
+pub fn logged_operation_entries_of_the_root_entries(
+    entries: &[Vec<u8>],
+) -> Result<Vec<LoggedOperationEntry>, LoggedOperationsRootRefusal> {
+    let parsed: Vec<LoggedOperationEntry> = entries
+        .iter()
+        .map(|entry| LoggedOperationEntry::parse(entry))
+        .collect::<Result<_, _>>()
+        .map_err(LoggedOperationsRootRefusal::EntryRefused)?;
+    if !parsed
+        .windows(2)
+        .all(|pair| pair[0].key().sort_key() < pair[1].key().sort_key())
+    {
+        return Err(LoggedOperationsRootRefusal::EntriesNotInStrictlyAscendingKeyOrder);
+    }
+    let compaction_intents = parsed
+        .iter()
+        .filter(|entry| entry.operation.kind() == LoggedOperationKind::Compaction)
+        .count();
+    if compaction_intents > 1 {
+        return Err(LoggedOperationsRootRefusal::MoreThanOneCompactionIntent {
+            compaction_intents,
+        });
+    }
+    let truncation_intents = parsed
+        .iter()
+        .filter(|entry| entry.operation.kind() == LoggedOperationKind::Truncation)
+        .count();
+    if truncation_intents > 1 {
+        return Err(LoggedOperationsRootRefusal::MoreThanOneTruncationIntent {
+            truncation_intents,
+        });
+    }
+    Ok(parsed)
+}
+
+/// 读者拒收一个 logged_ops 根的原因：根里的字节是盘上读来的，调用方把它当损坏处置（可写挂载拒）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum LoggedOperationsRootRefusal {
+    EntryRefused(LoggedOperationEntryRefusal),
+    EntriesNotInStrictlyAscendingKeyOrder,
+    MoreThanOneCompactionIntent { compaction_intents: usize },
+    MoreThanOneTruncationIntent { truncation_intents: usize },
+}
+
+/// 一版 logged_ops 里那一条活整理意图（全池至多一条，判决 G1 e1）：签发它的那次发布的 txg 与意图本身。没有交回 `None`。
+///
+/// # Panics
+/// 有两条以上的整理意图：发布路径只在没有活意图时签发，盘上读回的条目由 [`logged_operation_entries_of_the_root_entries`] 拒过多于一条的。
+#[must_use]
+pub fn live_compaction_intent_of_the_entries(
+    entries: &[LoggedOperationEntry],
+) -> Option<(CheckpointTxg, CompactionIntent)> {
+    let mut intents = entries.iter().filter_map(|entry| match entry.operation {
+        LoggedOperation::Compaction(intent) => Some((entry.issued_txg, intent)),
+        LoggedOperation::Truncation(_intent) => None,
+    });
+    let live = intents.next();
+    assert!(
+        intents.next().is_none(),
+        "全池活整理意图至多一条（判决 G1 e1）：发布路径只在没有活意图时签发"
+    );
+    live
+}
+
+/// 一版 logged_ops 里那一条活截断意图（全池至多一条，判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1）：签发它的那次发布的
+/// txg 与意图本身。没有交回 `None`。
+///
+/// # Panics
+/// 有两条以上的截断意图：签发入口断言上一版没有截断条目，盘上读回的条目由 [`logged_operation_entries_of_the_root_entries`] 拒过多于一条的。
+#[must_use]
+pub fn live_truncation_intent_of_the_entries(
+    entries: &[LoggedOperationEntry],
+) -> Option<(CheckpointTxg, TruncationIntent)> {
+    let mut intents = entries.iter().filter_map(|entry| match entry.operation {
+        LoggedOperation::Truncation(intent) => Some((entry.issued_txg, intent)),
+        LoggedOperation::Compaction(_intent) => None,
+    });
+    let live = intents.next();
+    assert!(
+        intents.next().is_none(),
+        "全池活截断意图至多一条（判决 m4-trunc-r1 K1）：签发入口断言上一版没有截断条目"
+    );
+    live
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// 截断条目 44 字节逐字节钉住（判决 m4-trunc-r1 K1）：key = 种类 2 + 签发 txg；value = inode 号、对象出生代、目标 size、补零 10。
+    #[test]
+    fn a_truncation_entry_is_kind_two_then_the_issue_txg_inode_object_birth_target_size_and_ten_zero_bytes(
+    ) {
+        let entry = LoggedOperationEntry {
+            issued_txg: CheckpointTxg(0x0102_0304_0506_0708),
+            operation: LoggedOperation::Truncation(TruncationIntent {
+                inode: InodeNumber(0x1112_1314_1516_1718),
+                object_birth: CheckpointTxg(0x2122_2324_2526_2728),
+                target_size_in_bytes: 0x3132_3334_3536_3738,
+            }),
+        };
+        let bytes = entry.to_bytes();
+        let mut expected = vec![2, 0];
+        expected.extend_from_slice(&0x0102_0304_0506_0708u64.to_le_bytes());
+        expected.extend_from_slice(&0x1112_1314_1516_1718u64.to_le_bytes());
+        expected.extend_from_slice(&0x2122_2324_2526_2728u64.to_le_bytes());
+        expected.extend_from_slice(&0x3132_3334_3536_3738u64.to_le_bytes());
+        expected.extend_from_slice(&[0; 10]);
+        assert_eq!(bytes, expected, "截断条目 44 字节逐字节");
+        assert_eq!(LoggedOperationEntry::parse(&bytes), Ok(entry), "读回同一条");
+    }
+
+    /// 截断意图的补零有一个字节非零就拒收；一个根里两条截断意图拒收（全池至多一条）。
+    #[test]
+    fn a_truncation_entry_with_a_non_zero_padding_byte_or_a_second_truncation_entry_is_refused() {
+        let entry = |issued_txg: u64| LoggedOperationEntry {
+            issued_txg: CheckpointTxg(issued_txg),
+            operation: LoggedOperation::Truncation(TruncationIntent {
+                inode: InodeNumber(1),
+                object_birth: CheckpointTxg(3),
+                target_size_in_bytes: 100,
+            }),
+        };
+        let mut padded = entry(5).to_bytes();
+        padded[43] = 1;
+        assert_eq!(
+            LoggedOperationEntry::parse(&padded),
+            Err(LoggedOperationEntryRefusal::TruncationPaddingNotZero)
+        );
+        assert_eq!(
+            logged_operation_entries_of_the_root_entries(&[
+                entry(5).to_bytes(),
+                entry(6).to_bytes()
+            ]),
+            Err(LoggedOperationsRootRefusal::MoreThanOneTruncationIntent {
+                truncation_intents: 2
+            })
+        );
+        assert_eq!(
+            live_truncation_intent_of_the_entries(&[entry(5)]),
+            Some((CheckpointTxg(5), entry(5).operation_as_truncation())),
+        );
+        assert_eq!(live_compaction_intent_of_the_entries(&[entry(5)]), None);
+    }
+
+    impl LoggedOperationEntry {
+        fn operation_as_truncation(&self) -> TruncationIntent {
+            match self.operation {
+                LoggedOperation::Truncation(intent) => intent,
+                LoggedOperation::Compaction(_intent) => panic!("这里只造截断条目"),
+            }
+        }
+    }
+}
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 0b66c08..615ebad 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -24,20 +24,23 @@ use crate::allocator::{
 };
 use crate::block_device::BlockDevice;
 use crate::checksum::crc32_castagnoli;
+use crate::compaction_plan::{CompactionOfThisMount, CompactionSwitches};
 use crate::instance_table_sweep::{
     sweep_the_instance_table_at_the_end_of_the_mount, InstanceTableSweepAtTheEndOfTheMount,
     InstanceTableSweepConfiguration,
 };
 use crate::journal::back_chain_of;
+use crate::logged_operations::{live_compaction_intent_of_the_entries, LoggedOperationKind};
 use crate::make_filesystem::MakeFilesystemParameters;
 use crate::mounted_session::{
-    refusal_is_short_of_space, DeviceKeepingWritesInMemory, DeviceReadOnceInThisMount,
-    ReadRouteOfThisMount,
+    refusal_is_short_of_space, run_compaction_on, CompactionRun, CompactionRunReach,
+    DeviceKeepingWritesInMemory, DeviceReadOnceInThisMount, ReadRouteOfThisMount,
 };
 use crate::pointer::LocationEntry;
 use crate::pointer::{slot_shared_by_both_location_entries, LocationEntriesOnDifferentSlots};
 use crate::records::{
     LineageRecord, TreeTableEntry, TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE,
+    TREE_KIND_SNAPSHOT_INODE_ROOT,
 };
 use crate::recovery::read_unit_via_locations;
 use crate::recovery::{
@@ -57,8 +60,8 @@ use crate::recovery::{
     newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to,
     readable_roots, readable_roots_rereading_ring_slots_known_to_hold_a_root_once,
     readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once,
-    rebuild_version_in_the_unit_area_starting_at, replay_journal,
-    root_is_abandoned_by_the_instance_table, scan_journal,
+    rebuild_version_in_the_unit_area_starting_at, rebuild_version_judging_the_truncation_fields_as,
+    replay_journal, root_is_abandoned_by_the_instance_table, scan_journal,
     system_configuration_slots_of_one_device_read_once,
     system_configuration_witness_is_undecidable, tree_table_has_no_entries,
     user_visible_tree_root_pointers, verified_system_configuration_slots, BadRootRingSlotReading,
@@ -73,7 +76,7 @@ use crate::recovery::{
 };
 use crate::root_record::{RootRecord, UnmountMarker};
 use crate::root_ring::{target_for_publish, RootRingSlot};
-use crate::system_configuration::SystemImmutableSizes;
+use crate::system_configuration::{InodeRecordTruncationFieldsOfThePool, SystemImmutableSizes};
 use crate::transaction::{
     central_mapping_locations_in_the_version,
     copies_of_a_released_unit_failing_the_release_checksum_check,
@@ -1023,6 +1026,20 @@ pub struct MountOutput {
     /// 或这次挂载里判、抹、发到了哪儿（[`crate::instance_table_sweep::sweep_the_instance_table_at_the_end_of_the_mount`]）。
     /// 发成了清扫发布时 `Mounted::current` 是清扫发布那一版。
     pub instance_table_sweep: InstanceTableSweepAtTheEndOfTheMount,
+    /// 盘上那一条未完成的整理意图的续做（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1；D8（核心索引结构） 已定项 5
+    /// 第 2 条），连同这次挂载会话这一侧的整理状态（会话接着用，`mounted_session::MountedSession::of_the_mount`）。
+    pub compaction_continuation: CompactionContinuationAtTheMount,
+}
+
+/// 可写挂载续做整理意图（[`MountOutput::compaction_continuation`]）：次序在删除续做之后（判决 G8「删除续做 → 为用户请求推的抬 F → GC 批」，
+/// 挂载里没有用户请求）、实例表清扫之前；只推盘上那一条，不签新的。
+#[derive(Debug)]
+pub struct CompactionContinuationAtTheMount {
+    /// 推了的那一趟（停在哪、发了哪几版：发了的那几版接在删除续做之后，清扫没发布时最后一个就是 `Mounted::current`）。
+    /// 所选那一版的 logged_ops 没有活意图（常态）、或删除续做没走空（整理排在它后面）时 `None`，一个写都不发。
+    pub run: Option<CompactionRun>,
+    /// 这次挂载会话这一侧的整理状态（不合格段表、常驻隔离集、`B_gc` 数的发布数、观测）：续做时建的，会话接着用。
+    pub state_of_this_mount: CompactionOfThisMount,
 }
 
 /// 挂载续做停在哪儿。成员按调用方要做的决定分：照常写；队列还留着——之后会话里每一次用户改动先走空、走不动报空间不够
@@ -1247,12 +1264,16 @@ fn instance_table_rewrite_of_the_row_publish(
 fn rebuild_previous_version<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
     unit_area_start: UnitAreaStart,
+    truncation_fields_of_the_pool: InodeRecordTruncationFieldsOfThePool,
     root: &RootRecord,
     record_standing_for_root: Option<crate::journal::JournalRecord>,
 ) -> Result<PreviousVersion, MountError> {
-    let rebuilt = rebuild_version_in_the_unit_area_starting_at(
+    // inode 记录的截断字段按择到的系统配置那一档判（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第六节）：不带 incompat 位 2 的池里
+    // 截断字段不是全零的记录判不可用，这一版建不出来。
+    let rebuilt = rebuild_version_judging_the_truncation_fields_as(
         devices,
         unit_area_start,
+        truncation_fields_of_the_pool,
         root,
         record_standing_for_root,
     )
@@ -3479,7 +3500,11 @@ fn format_time_allocator<Device: BlockDevice>(
 }
 
 /// 一条有效根的用户可见状态有没有变（D16（发布语义） 已定项 1「非空」从盘上怎么认，2026-09-17 用户定案）：它树表里 inode 树与 extent 树的
-/// 根指针，与它前一条有效根树表里的，逐字节比；两棵树任一棵不同就算非空。最旧的有效根没有前一条，拿 `UserVisibleTreeRootPointers::ABSENT` 比。
+/// 根指针、加上只读快照条目（码 14 / 15）的根指针按盘上次序排成的那一串（用户 2026-10-14 定「建快照那次发布算非空」），与它前一条有效根
+/// 树表里的逐字节比；三样任一样不同就算非空。最旧的有效根没有前一条，拿 `UserVisibleTreeRootPointers::ABSENT` 比。
+/// 只比根指针，不比 flags：删快照的意图那次（只写「销毁中」的记号、传前驱）与只放 deadlist 条目的批不算非空，删掉那一对的最后一批算。
+/// 整理搬了用户可见树的根单元的那一批：树表照映射位置重写那一条的根指针（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3），
+/// 字节变了，算非空（用户 2026-10-13 定「照字节比，搬根算非空」；判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J6 核过）。
 #[must_use]
 pub fn user_visible_trees_changed(
     root_pointers: &UserVisibleTreeRootPointers,
@@ -3487,6 +3512,7 @@ pub fn user_visible_trees_changed(
 ) -> bool {
     root_pointers.inode_tree != previous_valid_root_pointers.inode_tree
         || root_pointers.extent_tree != previous_valid_root_pointers.extent_tree
+        || root_pointers.snapshot_tree_roots != previous_valid_root_pointers.snapshot_tree_roots
 }
 
 /// 算抬 F 的上限时读根环（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-09-26 定）：每个槽读一次；读坏的槽按这个进程知不知道
@@ -5487,7 +5513,11 @@ fn rehearsed_continuation_refusal_is_returned_before_acquisition(refusal: &Publi
         | PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion { .. }
         | PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { .. }
         | PublishError::DeletedInodesQueueRefused(_)
-        | PublishError::BlockDevice(_) => false,
+        | PublishError::BlockDevice(_) | PublishError::SnapshotCreationRefused(_)
+        | PublishError::SnapshotDestructionRefused(_)
+        | PublishError::DeadlistEntryNotReleasable { .. }
+        | PublishError::EmptyClusterSegmentsBelowTheFloor(_)
+        | PublishError::TruncationRefused(_) => false,
     }
 }
 
@@ -6929,6 +6959,29 @@ fn establish_instance<Device: BlockDevice>(
         writes_of_the_publishes_before_the_continuation,
         space_admission.floor_raises(),
     )?;
+    // 整理意图的续做（判决 G1；D8（核心索引结构） 已定项 5 第 2 条「续做是每次恢复都跑的常规动作」；判决 G8 次序：删除续做 → 抬 F → 整理）：
+    // 现行那一版 logged_ops 里有一条未完成的意图、删除队列走空了，就照分配器上的整理开关推它，推到它删了、停机原因或预算用完，不签新的。
+    // 整理自己不让挂载失败（提示类意图不挡挂载，D8 已定项 5），走到哪儿记进交回值；产品路径批大小条款没写，签任何东西、发任何写之前停下。
+    let mut compaction_state_of_this_mount = CompactionOfThisMount::of_a_new_mount();
+    let live_intent_to_continue = match &deletion_queue_continuation_outcome {
+        DeletionQueueContinuationOutcome::WalkedToEmpty => current
+            .version_of_trees()
+            .and_then(|version| live_compaction_intent_of_the_entries(&version.logged_operations)),
+        DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { .. } => None,
+    };
+    let compaction_run = if live_intent_to_continue.is_some() {
+        Some(run_compaction_on(
+            parameters,
+            devices.as_mut_slice(),
+            &mut allocator,
+            &mut compaction_state_of_this_mount,
+            &mut current,
+            instance,
+            CompactionRunReach::ContinueTheLiveIntentAtTheMount,
+        ))
+    } else {
+        None
+    };
     // 实例表清扫（D18（块里携带什么信息） 已定项 11「回收」一条「谁跑、什么时候跑」，用户 2026-10-03 定）：写行、暖机、抬 F、删除续做之后，
     // 交回之前判一次。清扫自己不让挂载失败（落点被拒这一轮不删、挂载照样交回可写，Q4），走到哪儿记进交回值。
     let instance_table_sweep = with_reads_straight_to_the_disk(&read_route, || {
@@ -6967,6 +7020,10 @@ fn establish_instance<Device: BlockDevice>(
             refusals_that_pushed_the_continuation_floor_raises,
             deletion_queue_continuation_outcome,
             instance_table_sweep,
+            compaction_continuation: CompactionContinuationAtTheMount {
+                run: compaction_run,
+                state_of_this_mount: compaction_state_of_this_mount,
+            },
         },
         allocator,
         current,
@@ -7052,7 +7109,9 @@ fn continue_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
         PoolVersion::WithoutFile(_) => return Ok(continuation),
         PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => version,
     };
-    if current_version_of_trees.deleted_inodes.is_empty() {
+    // 销毁中的快照（判决 S4：崩在意图与最后一批之间，下一次可写挂载续做）、截断条目（判决 m4-trunc-r1 K1）与 `deleted_inodes`
+    // 同一个走空走它们。
+    if current_version_of_trees.queue_of_committed_intents_is_empty() {
         return Ok(continuation);
     }
     let switches = allocator.deletion_switches();
@@ -8140,6 +8199,7 @@ pub fn mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread<
         space_admission,
         shadow_ledger,
         DeletionSwitches::PRODUCT,
+        CompactionSwitches::PRODUCT,
         before_the_one_reread,
         InstanceTableSweepConfiguration::PRODUCT,
     )
@@ -8164,6 +8224,30 @@ pub fn mount_writable_with_test_only_switches_and_deletion_switches<Device: Bloc
         space_admission,
         shadow_ledger,
         deletion_switches,
+        CompactionSwitches::PRODUCT,
+        BeforeTheOneReread::RereadImmediately,
+        InstanceTableSweepConfiguration::PRODUCT,
+    )
+}
+
+/// 同 [`mount_writable_with_test_only_switches`]，整理的只供测试的开关（`compaction_plan::CompactionSwitches`，判决 G6）也由调用方给：
+/// 装在交回的分配器上，挂载续做盘上那一条未完成的整理意图（[`MountOutput::compaction_continuation`]）与这次挂载之后的会话照它。
+/// 产品路径走 [`mount_writable`]，恒 `CompactionSwitches::PRODUCT`（批大小条款没写，续做在签任何东西之前停下）。
+///
+/// # Errors
+/// 同 [`mount_writable_with_test_only_switches`]：续做自己不报错（提示类意图不挡挂载，D8（核心索引结构） 已定项 5），走到哪儿记进交回值。
+pub fn mount_writable_with_compaction_switches<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    compaction_switches: CompactionSwitches,
+) -> Result<Mounted, MountError> {
+    mount_writable_with_every_test_only_switch(
+        parameters,
+        devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        DeletionSwitches::PRODUCT,
+        compaction_switches,
         BeforeTheOneReread::RereadImmediately,
         InstanceTableSweepConfiguration::PRODUCT,
     )
@@ -8189,18 +8273,49 @@ pub fn mount_writable_with_instance_table_sweep_configuration<Device: BlockDevic
         space_admission,
         shadow_ledger,
         DeletionSwitches::PRODUCT,
+        CompactionSwitches::PRODUCT,
         BeforeTheOneReread::RereadImmediately,
         instance_table_sweep,
     )
 }
 
-/// 可写挂载的全部只供测试的开关一处收：空间准入、影子账、删除开关、「重读一次」之前的钩子、实例表清扫的配置。
+/// 可写挂载择系统配置（S3，C707）：先按调用方给的本池 fsid 筛（[`choose_system_configuration_of_the_pool`]：自述别的 fsid 的槽当自证不过）；
+/// 筛完一块盘可择的都没有（`NoValidSystemConfiguration`：盘上没有一槽是调用方说的这个池的）才不筛再择一次，择到的 fsid 与调用方的不同，
+/// 由后面的参数核报成 [`MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`]（实审 A1b Q4，调用方交错了 fsid 那一形照旧报）。
+///
+/// # Errors
+/// 筛过之后择系统配置报的错（不是一槽都没有的那一种）；不筛那一次报的错。
+#[allow(
+    clippy::ptr_arg,
+    reason = "choose_system_configuration 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
+)]
+fn system_configuration_of_the_pool_named_by_the_caller<Device: BlockDevice>(
+    devices: &Vec<(DeviceIdentity, Device)>,
+    filesystem_identifier_handed_in: &[u8; 16],
+) -> Result<crate::system_configuration::SystemConfiguration, RecoveryFailure> {
+    let among_the_slots_of_the_pool =
+        choose_system_configuration_of_the_pool(devices, filesystem_identifier_handed_in);
+    if matches!(
+        among_the_slots_of_the_pool,
+        Err(RecoveryFailure::NoValidSystemConfiguration { .. })
+    ) {
+        return choose_system_configuration(devices);
+    }
+    among_the_slots_of_the_pool
+}
+
+/// 可写挂载的全部只供测试的开关一处收：空间准入、影子账、删除开关、整理开关、「重读一次」之前的钩子、实例表清扫的配置。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样只供测试的开关或挂载的输入，不成一个概念"
+)]
 fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
     space_admission: SpaceAdmission,
     shadow_ledger: ShadowLedger,
     deletion_switches: DeletionSwitches,
+    compaction_switches: CompactionSwitches,
     mut before_the_one_reread: BeforeTheOneReread<'_>,
     instance_table_sweep: InstanceTableSweepConfiguration,
 ) -> Result<Mounted, MountError> {
@@ -8248,7 +8363,15 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
     // 退过去的那一版也建不出来，照报所选那一版的失败。之后的一切（F1 乙再判一次、取号之前逐盘核、写行、影子账与 C）都按退到的那一版。
     // 退到哪一版照 `F2_NEWEST`：[见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（判法在那个函数里）。
     let (chosen_root, effective_root, journal, selected_version_journal_position, previous) =
-        match rebuild_previous_version(devices, unit_area_start, &effective_root, own_record) {
+        match rebuild_previous_version(
+            devices,
+            unit_area_start,
+            system_configuration
+                .immutable
+                .inode_record_truncation_fields,
+            &effective_root,
+            own_record,
+        ) {
             Ok(previous) => (
                 chosen_root,
                 effective_root,
@@ -8278,6 +8401,9 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
                 match rebuild_previous_version(
                     devices,
                     unit_area_start,
+                    system_configuration
+                        .immutable
+                        .inode_record_truncation_fields,
                     &version_fallen_back_to,
                     record_standing_for_the_version_fallen_back_to,
                 ) {
@@ -8347,6 +8473,15 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
         root_ring_of_the_rebuilt_allocator,
     } = rebuilt?;
     allocator.set_deletion_switches(deletion_switches);
+    allocator.set_compaction_switches(compaction_switches);
+    // 重建时读回活意图、恢复 R 排他（D26（后台整理与放置回收） 已定项 1 第 3 条：意图住 logged_ops，分配器读得到）：在写行、暖机、抬 F 与任何
+    // 续做之前装上，这次挂载里每一次发布的新落点都不进 R（暂停是会话那一侧的事，挂载里不暂停）。
+    if let Some((_issued_txg, intent)) = previous
+        .version_of_trees()
+        .and_then(|version| live_compaction_intent_of_the_entries(&version.logged_operations))
+    {
+        allocator.compaction_mut().issue(intent);
+    }
     let previous_row = PreviousInstanceRow {
         instance: effective_root.instance,
         selected_root_txg: effective_root.checkpoint_txg,
@@ -8425,6 +8560,13 @@ pub enum RollbackError {
     TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
         target: RollbackTarget,
     },
+    /// 现行那一版或回退目标那一版的树表里有只读快照条目（码 14 / 15）：第一版有快照就拒回退（用户 2026-10-13 定「第一版有快照就拒」，
+    /// 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S8 与第七节岔路 3）。`version` 说是哪一版先被认出带着快照
+    /// （现行那一版在任何读之前判，目标那一版在重建出来之后判），`snapshots` 是那一版码 14 的条数。在任何写之前拒，盘上逐字节不变。
+    SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses {
+        version: VersionOfTheRollback,
+        snapshots: usize,
+    },
     /// 回退目标引用的一个用户可见单元，在现行那一版的账里这块盘上没有它的记录（没有起点是这个槽、跨度相同的那一条）。
     UserVisibleUnitWithoutItsRecordInTheCurrentAccount {
         unit: TransactionUnit,
@@ -8467,6 +8609,22 @@ pub enum RollbackError {
     Publish(Box<PublishSequenceFailed>),
 }
 
+/// 回退牵涉的两版里的哪一版（[`RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum VersionOfTheRollback {
+    Current,
+    Target,
+}
+
+/// 一版树表里码 14（只读快照的 inode 根）的条数。
+fn snapshots_in_the_tree_table(version: &TransactionOutput) -> usize {
+    version
+        .tree_table_entries
+        .iter()
+        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
+        .count()
+}
+
 /// 复活的一份：哪个单元、哪块盘、落点、写回的分配代（回退目标那一版账里的）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct ResurrectedCopy {
@@ -8506,14 +8664,17 @@ pub const fn is_a_user_visible_unit(identity: TransactionUnit) -> bool {
         | TransactionUnit::AllocationTree
         | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
         | TransactionUnit::AccountingTree
+        | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
+        | TransactionUnit::DeadlistTree
         | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
         | TransactionUnit::MappingTree
         | TransactionUnit::TreeTable
         | TransactionUnit::InstanceTable
         | TransactionUnit::InstanceTablePageAfterTheFirst(_) => false,
         // `deleted_inodes` 根不在 D23 那个记号里（新根照样取 R_old 的，见 [`is_a_unit_taken_from_the_rollback_target`]）；
-        // 墓碑树、谱系树与退役树随现行那一版（退役树：判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条）。
+        // 墓碑树、谱系树、logged_ops 与退役树随现行那一版（退役树：判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条）。
         TransactionUnit::DeletedInodesRoot
+        | TransactionUnit::LoggedOperationsRoot
         | TransactionUnit::TombstoneLeafContainer(_)
         | TransactionUnit::TombstoneRoot
         | TransactionUnit::TombstoneNodeBelowTheRoot(_)
@@ -8546,6 +8707,8 @@ pub const fn is_a_unit_taken_from_the_rollback_target(identity: TransactionUnit)
         | TransactionUnit::AllocationTree
         | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
         | TransactionUnit::AccountingTree
+        | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
+        | TransactionUnit::DeadlistTree
         | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
         | TransactionUnit::MappingTree
         | TransactionUnit::TreeTable
@@ -8558,6 +8721,7 @@ pub const fn is_a_unit_taken_from_the_rollback_target(identity: TransactionUnit)
         | TransactionUnit::LineageRoot
         | TransactionUnit::LineageNodeBelowTheRoot(_)
         | TransactionUnit::RetirementLeafContainer(_)
+        | TransactionUnit::LoggedOperationsRoot
         | TransactionUnit::RetirementRoot
         | TransactionUnit::RetirementNodeBelowTheRoot(_) => is_a_user_visible_unit(identity),
     }
@@ -8919,7 +9083,8 @@ fn tree_identifier_watermark_of_the_rollback(
 }
 
 /// 回退那次发布接在后面的「上一版」：用户可见的那一半（数据单元与指针、extent 树、inode 树的叶容器与根、第一个文件那条 inode 记录）
-/// 取回退目标那一版的，别的（分配记录树与它的记录、记账树与记账行、中央映射树、树表、实例表各片）取现行那一版的；
+/// 取回退目标那一版的，别的（分配记录树与它的记录、记账树与记账行、中央映射树、树表、实例表各片）取现行那一版的——中央映射树只是节点取现行那一版的，
+/// 用户可见单元的条目由发布路径按目标那一版的位置装、位置不同的那几把 key 所在的路重写（[`mapping_keys_whose_target_locations_differ_from_the_current_mapping`]）；
 /// 根记录、末条记录、事务号取现行那一版的（这次发布接着它发，反向链、计数器、txg 都接着它）。
 /// 发布路径照这一版照抄用户可见单元（inode 树根指针从树表条目取，所以树表条目里 inode 树与 extent 树那两条换成目标那一版的）、
 /// 按分配器现算固定点单元、换下现行那一版被重写的固定点单元——正是「新根的用户可见树取 R_old 的，树表单元、分配记录树、中央映射树与
@@ -8974,6 +9139,7 @@ fn version_to_publish_the_rollback_after(
             } else {
                 entry.root
             },
+            previous_snapshot_txg: entry.previous_snapshot_txg,
             birth_txg: entry.birth_txg,
             head_identifier: entry.head_identifier,
         })
@@ -8996,12 +9162,25 @@ fn version_to_publish_the_rollback_after(
         extent_tree: target_version.extent_tree.clone(),
         accounting_entries: current.accounting_entries.clone(),
         accounting_tree: current.accounting_tree.clone(),
+        // 映射树的节点是现行那一版的（这次发布照它们 COW、换下的照它们释放）；用户可见单元的条目取目标那一版的位置：发布路径按这一版的
+        // 指针装条目（目标那一版重建时已按它自己的映射覆盖过提示），装着位置不同的那几把 key 的叶连同到根的路重写
+        // （[`mapping_keys_whose_target_locations_differ_from_the_current_mapping`]，判决 m4-segment-code-r1 第三节 F1）。
         central_mapping_tree: current.central_mapping_tree.clone(),
+        // 两版都不带快照（回退之前判过，判决 S8）：deadlist 与留存的映射条目照现行那一版（删完全部快照之后是空的）。
+        deadlist_entries: current.deadlist_entries.clone(),
+        deadlist_tree: current.deadlist_tree.clone(),
+        mapping_entries_kept_for_the_snapshots: current
+            .mapping_entries_kept_for_the_snapshots
+            .clone(),
+        snapshots_being_destroyed: current.snapshots_being_destroyed.clone(),
         tree_table_entries,
         tree_identifiers: current.tree_identifiers,
         inode_record: target_version.inode_record,
         // `deleted_inodes` 取 R_old 的（树表那一条上面换了），墓碑树随现行那一版（D23（journal 的角色与格式） 已定项 14）。
         deleted_inodes: target_version.deleted_inodes.clone(),
+        // logged_ops 的根与条目照现行那一版（发布路径照它的根判换下）；截断条目取 R_old 的那一换由回退那次发布做
+        // （`LoggedOperationsStepOfThePublish::TakeTheTruncationEntryOfTheRollbackTarget`，判决 m4-trunc-r1 K1）：未完成的整理意图跨回退留着、接着推。
+        logged_operations: current.logged_operations.clone(),
         tombstone_containers: current.tombstone_containers.clone(),
         tombstone_tree_index_nodes: current.tombstone_tree_index_nodes.clone(),
         // 谱系树同样随现行那一版（D18（块里携带什么信息） 已定项 24 第 5 条「随现行那一版、只增不删」）：回退这次往它上面追加。
@@ -9015,6 +9194,7 @@ fn version_to_publish_the_rollback_after(
         mapped_units,
         released: Vec::new(),
         quarantined_after_release_checksum_mismatch: Vec::new(),
+        units_relocated_by_this_publish: Vec::new(),
         key_order_mismatches: 0,
         writes: WritesByStructureKind::NOTHING_WRITTEN,
         highest_transaction_number_in_this_instance: current
@@ -9022,6 +9202,30 @@ fn version_to_publish_the_rollback_after(
     }
 }
 
+/// 回退那次发布要把中央映射里的条目换回目标那一版位置的那几把映射 key（判决 `research/prompts/m4-segment-code-r1-main-verification.md`
+/// 第三节 F1，`research/prompts/m4-gc-r1-main-verification.md` 第三节 G3「映射是唯一权威」）：目标那一版取来的用户可见单元
+/// （[`is_a_unit_taken_from_the_rollback_target`]）里，在现行那一版的中央映射里查到的不是目标那一版映射里那两条位置项的。
+/// 整理在两版之间搬过的单元就是这样：key 不变、现行那一版的条目指着目的落点，而回退释放的正是目的落点、复活的是源落点。
+/// 发布路径只按 key 进出定映射树重写哪些节点，key 相同的叶照抄上一版（现行那一版）的字节，条目就留在被释放的目的落点上；
+/// 交出这几把 key，装着它们的叶连同到根的路这次重写（与整理的一批同一条路，`transaction::publish_version_appending_a_lineage_record`）。
+/// 现行那一版查不到、条目切不动的也交出（重写一条路是合法的 COW）；目标那一版自己查不到的在 [`user_visible_units_through_the_mapping`] 那一判拒过。
+/// 只读两版内存里的映射叶，不读盘。
+fn mapping_keys_whose_target_locations_differ_from_the_current_mapping(
+    current: &TransactionOutput,
+    target_version: &TransactionOutput,
+) -> Vec<Vec<u8>> {
+    target_version
+        .mapped_units
+        .iter()
+        .filter(|(identity, _)| is_a_unit_taken_from_the_rollback_target(*identity))
+        .filter(|(_, key)| {
+            central_mapping_locations_in_the_version(current, key)
+                != central_mapping_locations_in_the_version(target_version, key)
+        })
+        .map(|(_, key)| key.clone())
+        .collect()
+}
+
 /// 管理员回退：挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14「管理员回退是挂着时的一次向前发布」）。
 /// 管理员从回退候选集里选 R_old（`target`）；文件系统在现行那一版（`current`，cur）后面发一次普通发布（D16（发布语义） 已定项 7 的
 /// 持久顺序）：新根的 inode 树、extent 树取 R_old 的；树表单元、分配记录树、中央映射树与记账统计量按这次发布照调整之后的分配器现算；
@@ -9057,6 +9261,16 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
             },
         );
     }
+    // 第一版有快照就拒回退（判决 S8、岔路 3）：现行那一版在内存里，在任何读之前判；目标那一版重建出来之后判。
+    let snapshots_in_the_current_version = snapshots_in_the_tree_table(current);
+    if snapshots_in_the_current_version > 0 {
+        return Err(
+            RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses {
+                version: VersionOfTheRollback::Current,
+                snapshots: snapshots_in_the_current_version,
+            },
+        );
+    }
     // 调用方的参数与盘表照可写挂载同一套核，写入口只按盘上那一份建（实审 A1b 留下的这一处）。
     let CallerInputsAgreeingWithTheDisk {
         system_configuration,
@@ -9127,6 +9341,15 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
     ) {
         return Err(RollbackError::CurrentAccountUnreadable { failure });
     }
+    let snapshots_in_the_target_version = snapshots_in_the_tree_table(&target_version);
+    if snapshots_in_the_target_version > 0 {
+        return Err(
+            RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses {
+                version: VersionOfTheRollback::Target,
+                snapshots: snapshots_in_the_target_version,
+            },
+        );
+    }
     if target_version.tree_identifiers != current.tree_identifiers {
         return Err(
             RollbackError::TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
@@ -9221,6 +9444,11 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
         },
     );
     let previous = version_to_publish_the_rollback_after(current, &target_version);
+    let mapping_keys_the_rollback_moves_back_to_the_target_locations =
+        mapping_keys_whose_target_locations_differ_from_the_current_mapping(
+            current,
+            &target_version,
+        );
     // 动分配器：先复活、再释放（释放代 = 新根的 txg），之后交给发布路径——它在这份账上照常释放 cur 被换下的固定点单元、取落点、
     // 装记账行。发布在落盘之前被拒时整个换回这一份之前的样子。
     let allocator_before_the_rollback = allocator.clone();
@@ -9278,6 +9506,14 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
         &previous,
         lineage_record,
         inode_number_watermark,
+        &mapping_keys_the_rollback_moves_back_to_the_target_locations,
+        // 截断条目取 R_old 那一版的（判决 m4-trunc-r1 K1）：截断批间回退到批间那一版，续做照它那一条截到底；
+        // 现行那一版截到一半而 R_old 没有截断条目，这一条随回退一起去掉，不截 R_old 的内容。
+        target_version
+            .logged_operations
+            .iter()
+            .copied()
+            .find(|entry| entry.operation.kind() == LoggedOperationKind::Truncation),
     ) {
         Ok(published) => {
             *current = published;
@@ -9364,6 +9600,44 @@ mod non_empty_root_tests {
         );
     }
 
+    /// 建快照那次发布算非空（用户 2026-10-14 定，D16（发布语义） 已定项 1「「非空」从盘上怎么认」）：它与前一条有效根只差快照条目的根指针
+    /// （活头两棵树照抄），照样算非空；去重的元组也带快照根，两条根只差一对快照条目时是两个不同状态。删快照的意图那次只改 flags、
+    /// 快照根指针不变，不算非空。
+    #[test]
+    fn a_root_that_only_adds_a_snapshot_pair_is_non_empty_and_counts_as_a_state_of_its_own() {
+        let before_the_snapshot = pointers(1, 2);
+        let with_the_snapshot = UserVisibleTreeRootPointers {
+            inode_tree: Some(vec![1; 86]),
+            extent_tree: Some(vec![2; 86]),
+            snapshot_tree_roots: vec![vec![1; 86], vec![2; 86]],
+        };
+        assert!(
+            user_visible_trees_changed(&with_the_snapshot, &before_the_snapshot),
+            "活头两棵树照抄、多了一对快照根：非空"
+        );
+        assert!(
+            !user_visible_trees_changed(&with_the_snapshot, &with_the_snapshot.clone()),
+            "快照根也照抄（删快照的意图那次、只放 deadlist 的批）：不算非空"
+        );
+        let distinct = newest_first_distinct_states(vec![
+            NonEmptyValidRoot {
+                checkpoint_txg: CheckpointTxg(5),
+                instance: InstanceGeneration(2),
+                user_visible_tree_root_pointers: before_the_snapshot,
+            },
+            NonEmptyValidRoot {
+                checkpoint_txg: CheckpointTxg(6),
+                instance: InstanceGeneration(2),
+                user_visible_tree_root_pointers: with_the_snapshot,
+            },
+        ]);
+        assert_eq!(
+            distinct,
+            vec![CheckpointTxg(6), CheckpointTxg(5)],
+            "只差一对快照根的两条根是两个不同状态"
+        );
+    }
+
     /// 上限取两者里小的那一个的另一半（上面那条两次都传 9，「每块盘上最新的有效根」从没胜出）：四个不同状态、第 4 新的是 3，
     /// 而有一块盘上最新的有效根只到 2（那块盘之后的根都读不出），上限取 2。只取第 4 新的那一条时上限是 3，越过那块盘上最新的有效根。
     #[test]
@@ -9390,6 +9664,7 @@ mod non_empty_root_tests {
         UserVisibleTreeRootPointers {
             inode_tree: Some(vec![inode_tree; 86]),
             extent_tree: Some(vec![extent_tree; 86]),
+            snapshot_tree_roots: Vec::new(),
         }
     }
 
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index bf8afb5..9b04943 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -41,13 +41,14 @@ use crate::allocator::{
 use crate::block_device::BlockDevice;
 use crate::checksum::crc32_castagnoli;
 use crate::code_two_tree::{
-    read_code_two_tree, CodeTwoTreeHeaderJudgement, CodeTwoTreeReadFromDisk,
+    read_code_two_tree, CodeTwoTreeHeaderJudgement, CodeTwoTreeReadFromDisk, CodeTwoTreeVersion,
 };
 use crate::container_tree::{
     ContainerTreeNodePosition, ContainerTreeShape, RecordContainer, IDENTITY_OF_A_CODE_TWO_CHILD,
 };
 use crate::deleted_inodes::inode_numbers_of_the_deleted_inodes_root_entries;
 use crate::deleted_inodes::DELETED_INODES_KEY_WIDTH_IN_BYTES;
+use crate::extent_tree::TailOfAFile;
 use crate::extent_tree::{
     read_extent_tree, ExtentLowerNodePosition, ExtentTreeHeaderJudgement, ExtentTreeReadFromDisk,
     ExtentTreeReading, ExtentTreeVersion, ExtentUpperSegmentReadFromDisk,
@@ -64,6 +65,10 @@ use crate::journal::{
     JournalRecordPlaceInPublish, FIRST_JOURNAL_COUNTER,
 };
 use crate::lineage_tree::LineageContainerIndexInTree;
+use crate::logged_operations::{
+    logged_operation_entries_of_the_root_entries, LoggedOperationEntry,
+    LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES,
+};
 use crate::make_filesystem::TREE_TABLE_KEY_WIDTH;
 use crate::pointer::{
     location_entries_ascend_by_device, DataPointer, LocationEntry, NodePointer,
@@ -73,27 +78,33 @@ use crate::records::{
     mapping_key_for_data, mapping_key_for_node, parse_inode_internal_entry_or_refusal,
     parse_mapping_entry, AccountingEntry, InodeRecord, LineageRecord, PointerCarryingEntryRefusal,
     RetirementRecord, TombstoneRecord, TreeTableEntry, NLINK_OF_AN_INODE_BEING_DELETED,
-    STATISTIC_INODE_WATERMARK, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST,
-    TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LINEAGE,
-    TREE_KIND_LIVELIST, TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_RETIREMENT,
+    SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION, STATISTIC_INODE_WATERMARK, TREE_KIND_ACCOUNTING,
+    TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT,
+    TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_LIVELIST, TREE_KIND_LOGGED_OPERATIONS,
+    TREE_KIND_RETIREMENT, TREE_KIND_SNAPSHOT_EXTENT_ROOT, TREE_KIND_SNAPSHOT_INODE_ROOT,
     TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_TOMBSTONE,
 };
+use crate::records::{DeadlistEntry, SnapshotDestructionMark, DEADLIST_ENTRY_KEY_BYTES};
 use crate::retirement_tree::RetirementContainerIndexInTree;
 use crate::root_record::RootRecord;
 use crate::root_ring::{region_start, slot_offset, RootRingSlot, RootRingSlotsPerRegionOutOfRange};
 use crate::system_configuration::{
-    journal_in_flight_record_limit, journal_in_flight_record_limit_fits_its_four_byte_field,
-    journal_ring_geometry_fields_recorded_in_the_slot, unit_area_start_slot_recorded_in_the_slot,
-    IncompatBitmap, SystemConfiguration, SystemConfigurationSlotRefusal, SystemImmutableSizes,
-    JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION, ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
+    filesystem_identifier_self_described_by_the_slot, journal_in_flight_record_limit,
+    journal_in_flight_record_limit_fits_its_four_byte_field,
+    journal_ring_geometry_fields_recorded_in_the_slot, root_ring_region_count_recorded_in_the_slot,
+    unit_area_start_slot_recorded_in_the_slot, IncompatBitmap,
+    InodeRecordTruncationFieldsOfThePool, SystemConfiguration, SystemConfigurationSlotRefusal,
+    SystemImmutableSizes, JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
+    ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
 };
 use crate::tombstone_tree::TombstoneContainerIndexInTree;
 use crate::tombstone_tree::CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES;
 use crate::transaction::{
-    role_of_allocation_record_tree_node, role_of_extent_upper_node, ContainerTreeIndexNodesVersion,
-    FileVersionTreeIdentifiers, InodeLeafContainerVersion, LineageContainerVersion,
-    MultiLevelCodeTwoTree, PublishedUnit, RetirementContainerVersion, TombstoneContainerVersion,
-    TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
+    role_of_allocation_record_tree_node, role_of_extent_upper_node,
+    take_the_mapping_positions_over_the_location_hints, ContainerTreeIndexNodesVersion,
+    FileOfAnInodeOtherThanTheFirst, FileVersionTreeIdentifiers, InodeLeafContainerVersion,
+    LineageContainerVersion, MultiLevelCodeTwoTree, PublishedUnit, RetirementContainerVersion,
+    TombstoneContainerVersion, TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
 };
 use crate::unit::{
     data_unit_payload, data_unit_payload_capacity, parse_data_unit, parse_index_node,
@@ -103,7 +114,7 @@ use crate::unit::{
     UNIT_CLASS_PACKED,
 };
 use crate::write_accounting::WritesByStructureKind;
-use crate::write_request_split::data_unit_count_of_a_sequential_write;
+use crate::write_request_split::{data_units_read_within_the_file_size, DataUnitsOfTheInode};
 
 /// 恢复路径读盘的口子。两个实现：文件后端的池（`Vec<(DeviceIdentity, Device)>`）与崩溃点重放枚举出的崩溃镜像（harness）。
 pub trait PoolReader {
@@ -2731,13 +2742,40 @@ pub fn tree_table_entry_count<Reader: PoolReader + ?Sized>(
 /// 树表里同一种树出现两条时，读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
 pub const TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE: &str = "树表里同一种树有两条";
 
+/// 树表里码 14（快照 inode 根）与码 15（快照 extent 根）的条数不相等时，读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
+pub const TREE_TABLE_SNAPSHOT_INODE_AND_EXTENT_ROOT_ENTRY_COUNTS_DIFFER: &str =
+    "树表里快照 inode 根与快照 extent 根的条数不相等";
+
+/// 树表里有一条码 15（快照 extent 根）的头 ID 不是任何一条码 14（快照 inode 根）的树 ID，或它指的那条码 14 前面已经有一条码 15 配上了：
+/// 读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
+pub const TREE_TABLE_SNAPSHOT_EXTENT_ROOT_ENTRY_WITHOUT_ITS_OWN_SNAPSHOT_INODE_ROOT_ENTRY: &str =
+    "树表里有一条快照 extent 根配不上一条只属于它的快照 inode 根";
+
+/// 树表里码 14（快照 inode 根）多于 [`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`] 条：读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，
+/// `what` 取这一句。
+pub const TREE_TABLE_CARRIES_MORE_SNAPSHOTS_THAN_THE_FIRST_VERSION_ALLOWS: &str =
+    "树表里的只读快照多于第一版允许的个数";
+
+/// 树表里有一条码 14（快照 inode 根）的头 ID 不等于同一张树表里活头 inode 树（码 2）条目的树 ID（表里没有码 2 条目时也报这一句）：
+/// 读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
+pub const TREE_TABLE_SNAPSHOT_INODE_ROOT_ENTRY_HEAD_IS_NOT_THE_LIVE_INODE_TREE: &str =
+    "树表里有一条快照 inode 根的头 ID 不是活头 inode 树的树 ID";
+
 /// 树表的条目读不下去是哪一种（[`tree_table_entries_each_kind_at_most_once`] 交回）。封闭集合，`match` 不写通配臂。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub(crate) enum TreeTableEntriesRefusal {
     /// 一条条目解不开（条目长度、flags 未知位、预留 76 非零、根指针头部 extent 偏移非 0，`TreeTableEntry::parse`）。
     EntryMalformed,
-    /// 同一种树有两条。
+    /// 同一种树有两条（码 14、15 两种快照条目除外）。
     OneKindOfTreeAppearsTwice,
+    /// 码 14 与码 15 的条数不相等。
+    SnapshotInodeAndExtentRootEntryCountsDiffer,
+    /// 一条码 15 的头 ID 不是任何一条码 14 的树 ID，或它指的那条码 14 已经被前面一条码 15 配上了。
+    SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry,
+    /// 码 14 多于 [`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`] 条。
+    MoreSnapshotsThanTheFirstVersionAllows,
+    /// 一条码 14 的头 ID 不等于这张树表里码 2 条目的树 ID（或表里没有码 2 条目）。
+    SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree,
 }
 
 impl From<TreeTableEntriesRefusal> for RecoveryFailure {
@@ -2749,17 +2787,110 @@ impl From<TreeTableEntriesRefusal> for RecoveryFailure {
             TreeTableEntriesRefusal::OneKindOfTreeAppearsTwice => RecoveryFailure::UnitMalformed {
                 what: TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE,
             },
+            TreeTableEntriesRefusal::SnapshotInodeAndExtentRootEntryCountsDiffer => {
+                RecoveryFailure::UnitMalformed {
+                    what: TREE_TABLE_SNAPSHOT_INODE_AND_EXTENT_ROOT_ENTRY_COUNTS_DIFFER,
+                }
+            }
+            TreeTableEntriesRefusal::SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry => {
+                RecoveryFailure::UnitMalformed {
+                    what: TREE_TABLE_SNAPSHOT_EXTENT_ROOT_ENTRY_WITHOUT_ITS_OWN_SNAPSHOT_INODE_ROOT_ENTRY,
+                }
+            }
+            TreeTableEntriesRefusal::MoreSnapshotsThanTheFirstVersionAllows => {
+                RecoveryFailure::UnitMalformed {
+                    what: TREE_TABLE_CARRIES_MORE_SNAPSHOTS_THAN_THE_FIRST_VERSION_ALLOWS,
+                }
+            }
+            TreeTableEntriesRefusal::SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree => {
+                RecoveryFailure::UnitMalformed {
+                    what: TREE_TABLE_SNAPSHOT_INODE_ROOT_ENTRY_HEAD_IS_NOT_THE_LIVE_INODE_TREE,
+                }
+            }
         }
     }
 }
 
+/// 一个只读快照在树表里的那一对条目：码 14（快照 inode 根）与头 ID 指着它的那条码 15（快照 extent 根），字段取值见
+/// [`TREE_KIND_SNAPSHOT_INODE_ROOT`] 与 [`TREE_KIND_SNAPSHOT_EXTENT_ROOT`]。重建只把它们读出来，不解它们指着的根。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct SnapshotTreeTableEntries {
+    pub inode_root: TreeTableEntry,
+    pub extent_root: TreeTableEntry,
+}
+
+/// 树表里的快照条目（码 14、15）配成对，按码 14 的盘上次序（即树 ID 升序，I-9.16 ①）交回，并判四样（判决
+/// `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1 与第七节岔路 1）：两种条数相等；每条码 15 的头 ID 恰是一条码 14 的树 ID、
+/// 且那条码 14 只被这一条码 15 配上（两样合起来是一一配对）；码 14 至多 [`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`] 条；每条码 14 的头 ID
+/// 等于这张树表里码 2（活头 inode 树）条目的树 ID。没有快照条目的树表交空表，一样都不判。
+///
+/// # Errors
+/// 按上面的次序第一样不成立的那一个：`SnapshotInodeAndExtentRootEntryCountsDiffer`、`SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry`、
+/// `MoreSnapshotsThanTheFirstVersionAllows`、`SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree`。
+pub(crate) fn snapshot_tree_table_entries_paired_one_to_one(
+    tree_table_entries: &[TreeTableEntry],
+) -> Result<Vec<SnapshotTreeTableEntries>, TreeTableEntriesRefusal> {
+    let snapshot_inode_root_entries: Vec<&TreeTableEntry> = tree_table_entries
+        .iter()
+        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
+        .collect();
+    let snapshot_extent_root_entries: Vec<&TreeTableEntry> = tree_table_entries
+        .iter()
+        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT)
+        .collect();
+    if snapshot_inode_root_entries.len() != snapshot_extent_root_entries.len() {
+        return Err(TreeTableEntriesRefusal::SnapshotInodeAndExtentRootEntryCountsDiffer);
+    }
+    // 每条码 14 配上的那条码 15，下标与 `snapshot_inode_root_entries` 一一对应。
+    let mut extent_root_paired_with: Vec<Option<TreeTableEntry>> =
+        vec![None; snapshot_inode_root_entries.len()];
+    for extent_root_entry in &snapshot_extent_root_entries {
+        // 头 ID 指着它、又还没被前面哪条码 15 配上的那条码 14。
+        let paired_inode_root_position = snapshot_inode_root_entries
+            .iter()
+            .zip(&extent_root_paired_with)
+            .position(|(inode_root_entry, extent_root_already_paired)| {
+                inode_root_entry.tree.0 == extent_root_entry.head_identifier
+                    && extent_root_already_paired.is_none()
+            })
+            .ok_or(
+                TreeTableEntriesRefusal::SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry,
+            )?;
+        extent_root_paired_with[paired_inode_root_position] = Some(**extent_root_entry);
+    }
+    if snapshot_inode_root_entries.len() > SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION {
+        return Err(TreeTableEntriesRefusal::MoreSnapshotsThanTheFirstVersionAllows);
+    }
+    let live_inode_tree = tree_table_entries
+        .iter()
+        .find(|entry| entry.kind == TREE_KIND_INODE)
+        .map(|entry| entry.tree);
+    if snapshot_inode_root_entries.iter().any(|inode_root_entry| {
+        Some(TreeIdentifier(inode_root_entry.head_identifier)) != live_inode_tree
+    }) {
+        return Err(TreeTableEntriesRefusal::SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree);
+    }
+    Ok(snapshot_inode_root_entries
+        .into_iter()
+        .zip(extent_root_paired_with)
+        .map(|(inode_root_entry, extent_root_entry)| SnapshotTreeTableEntries {
+            inode_root: *inode_root_entry,
+            extent_root: extent_root_entry.expect(
+                "两种条数相等、每条码 15 配上一条还没配过的码 14（上面两判）：鸽巢下每条码 14 都配上了",
+            ),
+        })
+        .collect())
+}
+
 /// 树表单元的条目逐条解开，并判每一种树至多一条（审阅第 27 条）。写者每种树只写一条；同一种出现两条时取哪一条条款没有写，
 /// 取首条、取末条、取首条非空各有一个读者照着读过，同一张树表在几处读出不同的树 ⇒ 判损坏，一条都不取。
+/// 码 14、15 两种快照条目例外：每个快照一对，可以有多条，改判 [`snapshot_tree_table_entries_paired_one_to_one`] 那四样。
 /// 读树表取条目的每一处都经这一处：[`rebuild_version`]、[`allocation_records_under_root`]、[`walk_to_file`]、
 /// [`user_visible_tree_root_pointers`]、`mounted_read::open_pool_for_read`。
 ///
 /// # Errors
-/// 按条目次序第一处读不下去的：解不开 ⇒ `EntryMalformed`；它的种类前面已经有一条 ⇒ `OneKindOfTreeAppearsTwice`。
+/// 按条目次序第一处读不下去的：解不开 ⇒ `EntryMalformed`；它的种类（码 14、15 除外）前面已经有一条 ⇒ `OneKindOfTreeAppearsTwice`；
+/// 条目都过了之后快照条目配不成对 ⇒ [`snapshot_tree_table_entries_paired_one_to_one`] 的那四个成员。
 pub(crate) fn tree_table_entries_each_kind_at_most_once(
     entry_bytes_of_the_tree_table: &[Vec<u8>],
 ) -> Result<Vec<TreeTableEntry>, TreeTableEntriesRefusal> {
@@ -2768,11 +2899,18 @@ pub(crate) fn tree_table_entries_each_kind_at_most_once(
     for entry_bytes in entry_bytes_of_the_tree_table {
         let entry =
             TreeTableEntry::parse(entry_bytes).ok_or(TreeTableEntriesRefusal::EntryMalformed)?;
+        let is_a_snapshot_entry = entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT
+            || entry.kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT;
+        if is_a_snapshot_entry {
+            entries.push(entry);
+            continue;
+        }
         if !kinds_seen.insert(entry.kind) {
             return Err(TreeTableEntriesRefusal::OneKindOfTreeAppearsTwice);
         }
         entries.push(entry);
     }
+    snapshot_tree_table_entries_paired_one_to_one(&entries)?;
     Ok(entries)
 }
 
@@ -2927,12 +3065,84 @@ impl From<RecoveryFailure> for RebuildVersionFailure {
 /// 记账行里没有 inode 号水位那一行 ⇒ `InodeNumberWatermarkRowMissingFromTheAccountingTree`。
 ///
 /// 单元区从 `unit_area_start` 起（择到的系统配置按环长现算的那个）：这一版的分配记录落不落在单元区里按它判。
+/// 树表里的只读快照条目（码 14、15）照盘上原样在交回那一版的 `tree_table_entries` 里（发布路径接着这一版发时照它们续上，
+/// `transaction` 装树表那一步）；要它们配成对的用 [`rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at`]。
 pub fn rebuild_version_in_the_unit_area_starting_at(
     reader: &dyn PoolReader,
     unit_area_start: UnitAreaStart,
     root: &RootRecord,
     record_standing_for_root: Option<JournalRecord>,
 ) -> Result<RebuiltVersion, RebuildVersionFailure> {
+    rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at(
+        reader,
+        unit_area_start,
+        root,
+        record_standing_for_root,
+    )
+    .map(|rebuilt| rebuilt.version)
+}
+
+/// 重建出来的一版，连同它树表里的只读快照条目（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct RebuiltVersionWithItsSnapshotEntries {
+    pub version: RebuiltVersion,
+    /// 码 14 与配对的码 15，按码 14 的树 ID 升序；树表里没有快照条目（含树表 0 条）时为空。只读出条目，不解它们指着的根。
+    pub snapshot_entries: Vec<SnapshotTreeTableEntries>,
+}
+
+/// 同 [`rebuild_version_in_the_unit_area_starting_at`]（判全、报错都与它相同），另把树表里码 14、15 的条目配成对交出来
+/// （[`snapshot_tree_table_entries_paired_one_to_one`]，配不成对的树表在 [`tree_table_entries_each_kind_at_most_once`] 那一步就拒了）。
+///
+/// 截断字段照这个写者写的那一种池（带 incompat 位 2）收：调用方手里没有择好的系统配置时用它；可写挂载按择到的那一份判，走
+/// [`rebuild_version_judging_the_truncation_fields_as`]。
+///
+/// # Errors
+/// 同 [`rebuild_version_in_the_unit_area_starting_at`]。
+pub fn rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at(
+    reader: &dyn PoolReader,
+    unit_area_start: UnitAreaStart,
+    root: &RootRecord,
+    record_standing_for_root: Option<JournalRecord>,
+) -> Result<RebuiltVersionWithItsSnapshotEntries, RebuildVersionFailure> {
+    rebuild_version_with_its_snapshot_entries_judging_the_truncation_fields_as(
+        reader,
+        unit_area_start,
+        InodeRecordTruncationFieldsOfThePool::WRITTEN_BY_THIS_WRITER,
+        root,
+        record_standing_for_root,
+    )
+}
+
+/// 同 [`rebuild_version_in_the_unit_area_starting_at`]，inode 记录的截断字段按择到的系统配置那一档判（`truncation_fields_of_the_pool`：
+/// 不带 incompat 位 2 的池里截断字段非零的记录判不可用，交回 `InvariantViolated`，detail 是
+/// [`INODE_RECORD_TRUNCATION_FIELD_IN_A_POOL_WITHOUT_INCOMPAT_BIT_TWO`]）。可写挂载建上一版走这一份。
+///
+/// # Errors
+/// 同 [`rebuild_version_in_the_unit_area_starting_at`]。
+pub fn rebuild_version_judging_the_truncation_fields_as(
+    reader: &dyn PoolReader,
+    unit_area_start: UnitAreaStart,
+    truncation_fields_of_the_pool: InodeRecordTruncationFieldsOfThePool,
+    root: &RootRecord,
+    record_standing_for_root: Option<JournalRecord>,
+) -> Result<RebuiltVersion, RebuildVersionFailure> {
+    rebuild_version_with_its_snapshot_entries_judging_the_truncation_fields_as(
+        reader,
+        unit_area_start,
+        truncation_fields_of_the_pool,
+        root,
+        record_standing_for_root,
+    )
+    .map(|rebuilt| rebuilt.version)
+}
+
+fn rebuild_version_with_its_snapshot_entries_judging_the_truncation_fields_as(
+    reader: &dyn PoolReader,
+    unit_area_start: UnitAreaStart,
+    truncation_fields_of_the_pool: InodeRecordTruncationFieldsOfThePool,
+    root: &RootRecord,
+    record_standing_for_root: Option<JournalRecord>,
+) -> Result<RebuiltVersionWithItsSnapshotEntries, RebuildVersionFailure> {
     let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
     let data_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
     // 树表 0 条的一版也要判：可写挂载接着它发的零单元发布照抄根记录里的实例表与树表指针。
@@ -2946,7 +3156,10 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
         })?;
     tree_table_key_width_is_the_tree_tables(&tree_table)?;
     if tree_table.entries.is_empty() {
-        return Ok(RebuiltVersion::WithoutFile);
+        return Ok(RebuiltVersionWithItsSnapshotEntries {
+            version: RebuiltVersion::WithoutFile,
+            snapshot_entries: Vec::new(),
+        });
     }
     let record =
         record_standing_for_root.ok_or(RebuildVersionFailure::NoRecordStandingForFileVersion)?;
@@ -2954,6 +3167,10 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
     let record_bytes = record.to_bytes();
     let tree_table_entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)
         .map_err(RecoveryFailure::from)?;
+    let snapshot_entries = snapshot_tree_table_entries_paired_one_to_one(&tree_table_entries)
+        .expect(
+            "tree_table_entries_each_kind_at_most_once 交回之前对同一串条目判过快照条目一一配对",
+        );
     // 可写挂载接着这一版发布，写者按种类装树表条目、断言树 ID 升序：盘上的次序与种类在这里先判（I-9.16），
     // 走得到那条断言的镜像在任何写之前拒。
     tree_table_entries_ascend_by_tree_identifier_and_in_the_issuing_order(&tree_table_entries)?;
@@ -3037,6 +3254,7 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
         },
     )?;
     let first_file_extents = extents_of_the_first_file(extent_tree_read)?;
+    let first_file_extents_tail = first_file_extents.tail;
     let mut data_pointers: Vec<DataPointer> =
         Vec::with_capacity(first_file_extents.data_pointers.len());
     let mut data_unit_contents_in_file_order: Vec<Vec<u8>> =
@@ -3125,6 +3343,7 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
             },
             root,
             expected_filesystem_identifier,
+            truncation_fields_of_the_pool,
         )?;
         if records.is_empty() {
             return Err(RecoveryFailure::UnitMalformed {
@@ -3161,13 +3380,16 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
             .into());
         }
         (Some(inode_record_of_the_first_file), _) => {
-            data_unit_count_matches_the_inode_size(
+            // size 之外还挂着的单元（截断中间态，判决 m4-trunc-r1 K2）不按声明长度核：只核 size 之内的那几个。
+            let data_units_within_the_size = data_unit_count_matches_the_inode_size(
                 data_pointers.len(),
+                first_file_extents_tail,
                 inode_record_of_the_first_file,
             )?;
             for (position, (data_pointer, content)) in data_pointers
                 .iter()
                 .zip(&data_unit_contents_in_file_order)
+                .take(data_units_within_the_size)
                 .enumerate()
             {
                 if content.is_empty() {
@@ -3267,6 +3489,45 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
             inode_numbers_of_the_deleted_inodes_root_entries(&node.entries)
         }
     };
+    // logged_ops 树（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1）：根是码 2 层级 0，条目 44 字节，逐条照字段表解
+    // （R 不对齐、槽数不是 64、补零非零、种类码保留、整理意图多于一条都拒：盘上读来的坏条目当损坏处置，可写挂载拒）。
+    let logged_operations_root_pointer = pointer_of(TREE_KIND_LOGGED_OPERATIONS)?;
+    let logged_operations_root_bytes =
+        if logged_operations_root_pointer == NodePointer::empty_root() {
+            None
+        } else {
+            Some(read_mapped_tree_node_via_hint_then_central_mapping(
+                reader,
+                &logged_operations_root_pointer,
+                MappedTreeNodeClass::IndexNode,
+                &central_mapping_locations_of_key,
+                &mut stale_location_hint_hops_not_exposed_by_this_reader,
+            )?)
+        };
+    let logged_operations: Vec<LoggedOperationEntry> = match &logged_operations_root_bytes {
+        None => Vec::new(),
+        Some(bytes) => {
+            let node = tree_root_checked_against_its_pointer(
+                bytes,
+                tree_identifiers.logged_operations,
+                LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES,
+                &logged_operations_root_pointer,
+                root,
+                expected_filesystem_identifier,
+            )?;
+            if node.level != 0 || node.entries.is_empty() {
+                return Err(RecoveryFailure::UnitMalformed {
+                    what: LOGGED_OPERATIONS_ROOT_IS_NOT_A_NON_EMPTY_LEVEL_ZERO_NODE,
+                }
+                .into());
+            }
+            logged_operation_entries_of_the_root_entries(&node.entries).map_err(|_refusal| {
+                RecoveryFailure::UnitMalformed {
+                    what: LOGGED_OPERATIONS_ROOT_ENTRIES_REFUSED_BY_THE_FIELD_TABLE,
+                }
+            })?
+        }
+    };
     // 墓碑树与谱系树（D18（块里携带什么信息） 已定项 24 第 1、5 条）：根恒为码 2，条目照 inode 树内部节点的 120 字节形态；根满长高之后
     // 根之下是类型段 0 的码 2 节点，层级 1 的节点的孩子是打包记录容器（墓碑类型 1、谱系类型 5）。两棵树读法相同，只差下面这几样。
     let mut read_container_tree_node = |pointer: &NodePointer, class: MappedTreeNodeClass| {
@@ -3415,6 +3676,44 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
     }) {
         return Err(RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree.into());
     }
+    // deadlist 树（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3：第三棵多层码 2 树）：树表条目根指针非零时整棵读回来，
+    // 同记账树（节点进映射、提示读不出经映射回退、照冷走读判全）；零根时一条条目都没有。接着这一版发布要按它规划、删快照的批要从它取条目。
+    let deadlist_pointer = pointer_of(TREE_KIND_DEADLIST)?;
+    let deadlist_tree = if deadlist_pointer == NodePointer::empty_root() {
+        CodeTwoTreeReadFromDisk {
+            version: CodeTwoTreeVersion::default(),
+            node_bytes: Vec::new(),
+            leaf_entries_in_key_order: Vec::new(),
+        }
+    } else {
+        read_code_two_tree(
+            &deadlist_pointer,
+            &MultiLevelCodeTwoTree::Deadlist.read_expectation(tree_identifiers.deadlist),
+            CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,
+            root,
+            expected_filesystem_identifier,
+            &mut |pointer: &NodePointer| {
+                read_mapped_tree_node_via_hint_then_central_mapping(
+                    reader,
+                    pointer,
+                    MappedTreeNodeClass::IndexNode,
+                    &central_mapping_locations_of_key,
+                    &mut stale_location_hint_hops_not_exposed_by_this_reader,
+                )
+            },
+        )?
+    };
+    let deadlist_entries = deadlist_entries_parsed(&deadlist_tree.leaf_entries_in_key_order)?;
+    // 删快照的意图写在被删那条码 14 自己身上（判决 S4，flags 位 0）：读回来，接着这一版的发布照它续写记号、走批。
+    let snapshots_being_destroyed: Vec<TreeIdentifier> = tree_table
+        .entries
+        .iter()
+        .filter_map(|entry_bytes| {
+            TreeTableEntry::parse_with_destruction_mark(entry_bytes.as_slice())
+        })
+        .filter(|(_, mark)| *mark == SnapshotDestructionMark::BeingDestroyed)
+        .map(|(entry, _)| entry.tree)
+        .collect();
     // 映射树是自举豁免：不经映射回退，只按父指针里的位置条目读；上面有提示读不出、经映射回退过的，这里拿的就是那时读进来的那一份。
     let mapping_tree = central_mapping_root.into_tree()?;
     mapping_entry_count_is_one_per_mapped_unit(
@@ -3423,9 +3722,12 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
             extent_tree_nodes: first_file_extents.nodes_in_bump_order.len(),
             allocation_record_tree_nodes: allocation_tree_read.nodes.len(),
             accounting_tree_nodes: accounting_tree.version.node_count(),
+            deadlist_tree_nodes: deadlist_tree.version.node_count(),
+            mapping_entries_kept_for_the_snapshots: deadlist_entries.len(),
             inode_leaf_containers: inode_root_node.entries.len(),
             data_units: first_file_extents.data_units_of_every_file(),
             deletion_tree_nodes: usize::from(deleted_inodes_root_bytes.is_some())
+                + usize::from(logged_operations_root_bytes.is_some())
                 + tombstone_tree_read
                     .as_ref()
                     .map_or(0, ContainerTreeRead::mapped_units)
@@ -3439,15 +3741,39 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
     )?;
     let mut mapping_keys: Vec<Vec<u8>> =
         Vec::with_capacity(mapping_tree.leaf_entries_in_key_order.len());
+    // 被活快照钉住的单元留存的映射条目（判决 S3 的 α）：每条 deadlist 条目的映射 key 一条，按映射 key 的序（叶条目本来的次序）。
+    let deadlist_mapping_keys: BTreeSet<&[u8]> = deadlist_entries
+        .iter()
+        .map(|entry| entry.mapping_key.as_slice())
+        .collect();
+    let mut mapping_entries_kept_for_the_snapshots: Vec<(Vec<u8>, [LocationEntry; 2])> =
+        Vec::with_capacity(deadlist_entries.len());
+    // 每条映射条目的位置（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3「重建时映射条目的位置覆盖提示」）：
+    // 这一版装完之后，指向进映射单元的每一处指针换成它（`transaction::take_the_mapping_positions_over_the_location_hints`）。
+    let mut mapping_position_of_key: BTreeMap<Vec<u8>, [LocationEntry; 2]> = BTreeMap::new();
     for entry in &mapping_tree.leaf_entries_in_key_order {
-        let (key, _locations) =
+        let (key, locations) =
             parse_mapping_entry(entry).ok_or(RecoveryFailure::EntryNarrowerThanItsFieldTable {
                 what: "映射条目",
                 entry_bytes: entry.len(),
                 field_table_bytes: usize::try_from(MAPPING_ENTRY_BYTES).expect("55"),
             })?;
+        if deadlist_mapping_keys.contains(key.as_slice()) {
+            mapping_entries_kept_for_the_snapshots.push((key.clone(), locations));
+        }
+        mapping_position_of_key.insert(key.clone(), locations);
         mapping_keys.push(key);
     }
+    // deadlist 条目按 key 在中央映射树里查不到：C300（deadlist 条目按 key 查映射查不到没有条款）没有条款，第一版拒这一版（删快照的批要从
+    // 那条映射条目取落点，取不到就放不了），在交给发布路径之前。
+    if mapping_entries_kept_for_the_snapshots.len() != deadlist_mapping_keys.len()
+        || deadlist_mapping_keys.len() != deadlist_entries.len()
+    {
+        return Err(RecoveryFailure::UnitMalformed {
+            what: "deadlist 条目的映射 key 在中央映射树里查不到，或两条 deadlist 条目同一把映射 key（C300 没有条款）",
+        }
+        .into());
+    }
     let data_role_of_position = |position: usize| {
         TransactionUnit::Data(DataUnitIndexInFile(
             u64::try_from(position).expect("单元序号"),
@@ -3513,41 +3839,63 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
             mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
         ));
     }
-    // 删除那两棵树接在记账树后面（按树 ID），与发布路径的 `mapped_units` 同序。
+    // deadlist 树的每个节点接在记账树后面（按树 ID），与发布路径的 `mapped_units` 同序。
+    let deadlist_shape = &deadlist_tree.version.shape;
+    for (node, pointer) in deadlist_shape
+        .nodes()
+        .iter()
+        .zip(&deadlist_tree.version.pointers)
+    {
+        mapped_units.push((
+            MultiLevelCodeTwoTree::Deadlist.role_of_node(node.position, deadlist_shape),
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
+        ));
+    }
+    // 删除那两棵树接在 deadlist 树后面（按树 ID），与发布路径的 `mapped_units` 同序。
     if deleted_inodes_root_bytes.is_some() {
         mapped_units.push((
             TransactionUnit::DeletedInodesRoot,
             mapping_key_for_node(UNIT_CLASS_INDEX_NODE, deleted_inodes_root_pointer),
         ));
     }
-    // 墓碑树、谱系树：每片容器、根之下的码 2 节点（bump 次序）、根，与发布路径的 `mapped_units` 同序。
-    for (tree_read, roles) in [
-        (
-            tombstone_tree_read
-                .as_ref()
-                .map(ContainerTreeRead::units_in_bump_order),
-            CONTAINER_TREE_ROLES_OF_THE_TOMBSTONE_TREE,
-        ),
-        (
-            lineage_tree_read
-                .as_ref()
-                .map(ContainerTreeRead::units_in_bump_order),
-            CONTAINER_TREE_ROLES_OF_THE_LINEAGE_TREE,
-        ),
-        (
-            retirement_tree_read
-                .as_ref()
-                .map(ContainerTreeRead::units_in_bump_order),
-            CONTAINER_TREE_ROLES_OF_THE_RETIREMENT_TREE,
-        ),
-    ] {
-        for (unit_of_the_tree, pointer, _) in tree_read.unwrap_or_default() {
-            mapped_units.push((
-                roles.of(unit_of_the_tree),
-                mapping_key_for_node(unit_of_the_tree.unit_class(), pointer),
-            ));
-        }
+    // 墓碑树、谱系树、logged_ops 根、退役树（按树 ID）：容器树每片容器、根之下的码 2 节点（bump 次序）、根，与发布路径的 `mapped_units` 同序。
+    let mapped_units_of_a_container_tree =
+        |tree_read: Option<Vec<(UnitOfAContainerTree, NodePointer, Vec<u8>)>>,
+         roles: ContainerTreeRolesOfOneTree| {
+            tree_read
+                .unwrap_or_default()
+                .into_iter()
+                .map(move |(unit_of_the_tree, pointer, _)| {
+                    (
+                        roles.of(unit_of_the_tree),
+                        mapping_key_for_node(unit_of_the_tree.unit_class(), pointer),
+                    )
+                })
+        };
+    mapped_units.extend(mapped_units_of_a_container_tree(
+        tombstone_tree_read
+            .as_ref()
+            .map(ContainerTreeRead::units_in_bump_order),
+        CONTAINER_TREE_ROLES_OF_THE_TOMBSTONE_TREE,
+    ));
+    mapped_units.extend(mapped_units_of_a_container_tree(
+        lineage_tree_read
+            .as_ref()
+            .map(ContainerTreeRead::units_in_bump_order),
+        CONTAINER_TREE_ROLES_OF_THE_LINEAGE_TREE,
+    ));
+    if logged_operations_root_bytes.is_some() {
+        mapped_units.push((
+            TransactionUnit::LoggedOperationsRoot,
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, logged_operations_root_pointer),
+        ));
     }
+    mapped_units.extend(mapped_units_of_a_container_tree(
+        retirement_tree_read
+            .as_ref()
+            .map(ContainerTreeRead::units_in_bump_order),
+        CONTAINER_TREE_ROLES_OF_THE_RETIREMENT_TREE,
+    ));
     let unit =
         |identity: TransactionUnit, locations: &[LocationEntry; 2], bytes: Vec<u8>| PublishedUnit {
             slot: locations[0].slot,
@@ -3648,20 +3996,34 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
                 .map(ContainerTreeRead::units_in_bump_order),
             CONTAINER_TREE_ROLES_OF_THE_LINEAGE_TREE,
         ),
-        (
-            retirement_tree_read
-                .as_ref()
-                .map(ContainerTreeRead::units_in_bump_order),
-            CONTAINER_TREE_ROLES_OF_THE_RETIREMENT_TREE,
-        ),
     ] {
         for (unit_of_the_tree, pointer, bytes) in tree_read.unwrap_or_default() {
             deletion_tree_units.push(unit(roles.of(unit_of_the_tree), &pointer.locations, bytes));
         }
     }
+    // logged_ops 根按树 ID 排在谱系树之后、退役树之前（与发布路径装出来的 `units` 同序）。
+    if let Some(bytes) = logged_operations_root_bytes {
+        deletion_tree_units.push(unit(
+            TransactionUnit::LoggedOperationsRoot,
+            &logged_operations_root_pointer.locations,
+            bytes,
+        ));
+    }
+    for (unit_of_the_tree, pointer, bytes) in retirement_tree_read
+        .as_ref()
+        .map(ContainerTreeRead::units_in_bump_order)
+        .unwrap_or_default()
+    {
+        deletion_tree_units.push(unit(
+            CONTAINER_TREE_ROLES_OF_THE_RETIREMENT_TREE.of(unit_of_the_tree),
+            &pointer.locations,
+            bytes,
+        ));
+    }
     let mut deletion_tree_units_not_yet_placed = Some(deletion_tree_units);
     for (tree, read) in [
         (MultiLevelCodeTwoTree::Accounting, &accounting_tree),
+        (MultiLevelCodeTwoTree::Deadlist, &deadlist_tree),
         (MultiLevelCodeTwoTree::CentralMapping, &mapping_tree),
     ] {
         if tree == MultiLevelCodeTwoTree::CentralMapping {
@@ -3698,7 +4060,7 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
         ),
     ]);
     let transaction_number_on_the_record_standing_for_this_root = record.transaction;
-    let rebuilt = TransactionOutput {
+    let mut rebuilt = TransactionOutput {
         root: *root,
         record,
         record_bytes,
@@ -3714,10 +4076,15 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
         accounting_entries,
         accounting_tree: accounting_tree.version,
         central_mapping_tree: mapping_tree.version,
+        deadlist_entries,
+        deadlist_tree: deadlist_tree.version,
+        mapping_entries_kept_for_the_snapshots,
+        snapshots_being_destroyed,
         tree_table_entries,
         tree_identifiers,
         inode_record,
         deleted_inodes,
+        logged_operations,
         tombstone_containers: tombstone_tree_read.as_ref().map_or_else(Vec::new, |read| {
             read.containers
                 .iter()
@@ -3761,6 +4128,7 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
         mapped_units,
         released: Vec::new(),
         quarantined_after_release_checksum_mismatch: Vec::new(),
+        units_relocated_by_this_publish: Vec::new(),
         key_order_mismatches: 0,
         writes: WritesByStructureKind::NOTHING_WRITTEN,
         // 重建出来的这一版属于**旧**实例。今天每条恢复路径（普通挂载、回退、切换）之后都要取新实例代号，
@@ -3773,17 +4141,32 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
         highest_transaction_number_in_this_instance:
             transaction_number_on_the_record_standing_for_this_root,
     };
+    // I-2.5 先判盘上读来的提示、再盖映射位置：盖过之后提示就没了，盘上逆序的提示被映射位置盖掉，池级 checker 照盘上字节判坏的镜像
+    // 可写挂载却会收下。
+    pointers_of_the_rebuilt_version_ascend_by_device(&rebuilt)?;
+    take_the_mapping_positions_over_the_location_hints(&mut rebuilt, |key| {
+        mapping_position_of_key.get(key).copied()
+    });
+    // 盖进来的映射位置同样按 I-2.5 判（它的射程含中央映射条目的 value）：下一次发布重写父节点时把它们写进指针，写者在那里断言升序。
     pointers_of_the_rebuilt_version_ascend_by_device(&rebuilt)?;
-    Ok(RebuiltVersion::of_a_version_of_trees(rebuilt))
+    Ok(RebuiltVersionWithItsSnapshotEntries {
+        version: RebuiltVersion::of_a_version_of_trees(rebuilt),
+        snapshot_entries,
+    })
 }
 
 /// 一条根的树表里用户可见的两棵树（inode 树、extent 树）的根指针，取树表条目里那 86 字节的盘上原样；树表里没有那棵树的条目时 `None`。
 /// 抬 F 的上限要数「非空」的有效根：一条有效根算非空 ⟺ 这两样与它前一条有效根的不同（D16（发布语义） 已定项 1「非空」从盘上怎么认，
 /// 2026-09-17 用户定案）；树表单元自己的落点每次发布都变，不拿它比。
+/// 只读快照的两种条目（码 14 快照 inode 根、码 15 快照 extent 根）也是用户可见的根（主 agent 2026-10-13 定：码 14 / 15 是用户可见角色，
+/// 判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S2）：它们的根指针按盘上次序记在 `snapshot_tree_roots`；
+/// 「非空」那一判连它一起比（用户 2026-10-14 定「建快照那次发布算非空」，`mount::user_visible_trees_changed`）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct UserVisibleTreeRootPointers {
     pub inode_tree: Option<Vec<u8>>,
     pub extent_tree: Option<Vec<u8>>,
+    /// 树表里每一条码 14、码 15 条目的根指针盘上原样，按条目的盘上次序（即树 ID 升序）；没有快照的树表为空。
+    pub snapshot_tree_roots: Vec<Vec<u8>>,
 }
 
 impl UserVisibleTreeRootPointers {
@@ -3791,6 +4174,7 @@ impl UserVisibleTreeRootPointers {
     pub const ABSENT: Self = Self {
         inode_tree: None,
         extent_tree: None,
+        snapshot_tree_roots: Vec::new(),
     };
 }
 
@@ -3825,6 +4209,13 @@ pub fn user_visible_tree_root_pointers(
             | TREE_KIND_LINEAGE
             | TREE_KIND_LOGGED_OPERATIONS
             | TREE_KIND_RETIREMENT => continue,
+            // 只读快照的两种条目：一个快照一对、可以有多条，按盘上次序记下（配对在 `tree_table_entries_each_kind_at_most_once` 判过）。
+            TREE_KIND_SNAPSHOT_INODE_ROOT | TREE_KIND_SNAPSHOT_EXTENT_ROOT => {
+                pointers
+                    .snapshot_tree_roots
+                    .push(TreeTableEntry::root_pointer_bytes(entry_bytes).to_vec());
+                continue;
+            }
             _ => {
                 return Err(RecoveryFailure::UnitMalformed {
                     what: "树的种类没登记",
@@ -4365,6 +4756,14 @@ pub(crate) fn root_assembled_from_the_new_root_segment_of_the_record(
     }
 }
 
+/// 重建一版时 logged_ops 根不是非空的层级 0 节点（`RecoveryFailure::UnitMalformed` 的 `what`，用例按它认）。
+pub const LOGGED_OPERATIONS_ROOT_IS_NOT_A_NON_EMPTY_LEVEL_ZERO_NODE: &str =
+    "logged_ops 根不是层级 0 的非空节点";
+/// 重建一版时 logged_ops 根里有条目过不了字段表（宽不是 44、种类码保留、整理意图的 R 不对齐或槽数不是 64、补零非零、整理意图多于一条、
+/// 不按 key 严格升序；`logged_operations::logged_operation_entries_of_the_root_entries`）：可写挂载拒（判决 G1「R 的解码拒不对齐的值」）。
+pub const LOGGED_OPERATIONS_ROOT_ENTRIES_REFUSED_BY_THE_FIELD_TABLE: &str =
+    "logged_ops 根里有条目过不了字段表";
+
 /// 每棵树的 key 宽（码 2 头里的自述 key 宽要与它相符）；day-1 只注册的三棵没有节点要解。
 fn key_width_for_kind(kind: u16) -> Option<usize> {
     match kind {
@@ -4378,11 +4777,12 @@ fn key_width_for_kind(kind: u16) -> Option<usize> {
         TREE_KIND_TOMBSTONE | TREE_KIND_LINEAGE | TREE_KIND_RETIREMENT => {
             Some(CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES)
         }
-        // logged_ops 的字段表留到第一个写者落地时定（D8（核心索引结构） 已定项 5、C364（整理意图记录没有字段表））。
-        TREE_KIND_LIVELIST
-        | TREE_KIND_SPARSE_SIDE_TABLE
-        | TREE_KIND_DEADLIST
-        | TREE_KIND_LOGGED_OPERATIONS => None,
+        // logged_ops 的 key = 种类 2 + 签发 txg 8（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1 字段表）。
+        TREE_KIND_LOGGED_OPERATIONS => Some(LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES),
+        TREE_KIND_LIVELIST | TREE_KIND_SPARSE_SIDE_TABLE | TREE_KIND_DEADLIST => None,
+        // 只读快照的两种条目指着的是建快照那一版活头 inode 树、extent 树的根：由快照的读路径按那两棵树解（这一批还没有），
+        // 冷走读不按种类码读它们（`walk_to_file` 对它们 `continue`）。
+        TREE_KIND_SNAPSHOT_INODE_ROOT | TREE_KIND_SNAPSHOT_EXTENT_ROOT => None,
         _ => None,
     }
 }
@@ -4875,6 +5275,8 @@ pub(crate) struct ExtentsOfAFileOtherThanTheFirst {
 /// （按 inode 号）、上段先叶后根）。
 pub(crate) struct ExtentsOfTheFirstFile {
     pub data_pointers: Vec<DataPointer>,
+    /// 上段叶条目标签声明的尾部（判决 m4-trunc-r1 K5）：标签 3、4 最后一个单元之后到 size 是洞。
+    pub tail: TailOfAFile,
     pub version: ExtentTreeVersion,
     /// 第一个文件之外各有数据单元的文件，按 inode 号升序。
     pub files_other_than_the_first: Vec<ExtentsOfAFileOtherThanTheFirst>,
@@ -4897,56 +5299,67 @@ impl ExtentsOfTheFirstFile {
 /// 第一个文件的单元从 0 起连号、没有洞（尾部的洞由标签声明）；第一个文件之外的文件只由按 inode 号写数据的入口从偏移 0 整份写，
 /// 单元从 0 起连号、至少一个、尾部没有洞（标签 1、2）。上段里可以没有第一个文件那一条（它删完了、或还没有单元）。
 ///
+/// 第一个文件那一条是标签 0（一个单元都没有）的收下，交回空的数据指针与「之后到 size 是洞」的尾部：整个文件是洞（判决 m4-trunc-r1 K5）。
+///
 /// # Errors
-/// 上段里没有第一个文件的条目、或还有别的 inode 的条目、第一个文件一个单元都没有、单元有洞 ⇒ `UnitMalformed`：
-/// 这几样今天的写路径都写不出来（只有它写 extent 树、每个文件版本从偏移 0 顺序写、长度 0 的内容也写一个单元），
+/// 第一个文件的单元有洞，或第一个文件之外的文件没有单元、单元有洞、尾部有洞 ⇒ `UnitMalformed`：
+/// 这几样今天的写路径都写不出来（每个文件版本从偏移 0 顺序写、长度 0 的内容也写一个单元；截断与变长只做第一个文件），
 /// 盘上读来的却可以是任何样子——第一版不支持，在任何落盘动作之前交回。
 pub(crate) fn extents_of_the_first_file(
     tree: ExtentTreeReadFromDisk,
 ) -> Result<ExtentsOfTheFirstFile, RecoveryFailure> {
     let ExtentTreeReadFromDisk { upper, files } = tree;
-    // 上段一条叶条目都没有：第一个文件删完之后的零条目根（D8（核心索引结构） 已定项 16 第 5 条），没有数据单元、没有下段。
-    if files.is_empty() {
-        let upper_root_level = upper
-            .nodes
-            .last()
-            .map(|(position, _, _)| position.level)
-            .expect("读回来的上段至少有根");
-        return Ok(ExtentsOfTheFirstFile {
-            data_pointers: Vec::new(),
-            version: ExtentTreeVersion {
-                upper_nodes: upper
-                    .nodes
-                    .iter()
-                    .map(|(position, pointer, _)| (*position, *pointer))
-                    .collect(),
-                lower_nodes: Vec::new(),
-            },
-            nodes_in_bump_order: upper
-                .nodes
+    let mut extents_of_the_first_file_on_disk: Option<ExtentsOfAFileReadFromDisk> = None;
+    let mut files_other_than_the_first: Vec<ExtentsOfAFileOtherThanTheFirst> = Vec::new();
+    // 迭代上界是上段叶条目数；按 inode 号升序（上段按位置读回来就是这个次序）。
+    for (inode, extents) in files {
+        if inode == FIRST_INODE_NUMBER {
+            extents_of_the_first_file_on_disk = Some(extents);
+            continue;
+        }
+        let data_pointers_of_the_file = extents.data_pointers();
+        let units_are_numbered_from_zero_without_a_hole = data_pointers_of_the_file
+            .iter()
+            .enumerate()
+            .all(|(position, (unit, _))| u64::try_from(position).expect("单元序号") == *unit);
+        if data_pointers_of_the_file.is_empty()
+            || extents.tail() != TailOfAFile::NoHoleAfterTheLastDataUnit
+            || !units_are_numbered_from_zero_without_a_hole
+        {
+            return Err(RecoveryFailure::UnitMalformed {
+                what: "extent 树里第一个文件之外的文件没有单元、单元有洞或尾部有洞（按 inode 号写数据写不出来，第一版不支持）",
+            });
+        }
+        let lower_nodes = match extents {
+            ExtentsOfAFileReadFromDisk::LowerSegment(segment, _) => segment.nodes,
+            ExtentsOfAFileReadFromDisk::NoDataUnit | ExtentsOfAFileReadFromDisk::Inline(..) => {
+                Vec::new()
+            }
+        };
+        files_other_than_the_first.push(ExtentsOfAFileOtherThanTheFirst {
+            inode: InodeNumber(inode),
+            data_pointers: data_pointers_of_the_file
                 .into_iter()
                 .map(|(_, pointer)| pointer)
                 .collect(),
             lower_nodes,
         });
     }
-    let [(inode, extents)] =
-        <[(u64, ExtentsOfAFileReadFromDisk); 1]>::try_from(files).map_err(|_files| {
-            RecoveryFailure::UnitMalformed {
-                what: "extent 树上段里不是恰好一条叶条目（第一版只有第一个文件有内容）",
-            }
-        })?;
-    if inode != FIRST_INODE_NUMBER {
-        return Err(RecoveryFailure::UnitMalformed {
-            what: "extent 树上段里那一条叶条目不是第一个文件的",
-        });
-    }
+    // 上段里没有第一个文件那一条：第一个文件删完之后的零条目根（D8（核心索引结构） 已定项 16 第 5 条），或删完之后只有别的文件有条目，
+    // 第一个文件没有数据单元、没有下段。
+    let Some(extents) = extents_of_the_first_file_on_disk else {
+        return Ok(extents_assembled_with_the_upper_segment(
+            upper,
+            Vec::new(),
+            TailOfAFile::NoHoleAfterTheLastDataUnit,
+            Vec::new(),
+            files_other_than_the_first,
+        ));
+    };
+    // 标签 0（一个数据单元都没有）收下：整个文件是洞（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5「标签 0 配 size > 0
+    // 是整个文件都是洞」），读出 0、不读盘；单元数对不对 size 由调用方按标签声明的尾部判（`data_unit_count_matches_the_inode_size`）。
+    // 与挂载态读者同一读法（`mounted_read` 把标签 0 当整个洞）。今天的写路径写不出这一形。
     let data_pointers_with_units = extents.data_pointers();
-    if data_pointers_with_units.is_empty() {
-        return Err(RecoveryFailure::UnitMalformed {
-            what: "extent 树里第一个文件一个数据单元都没有",
-        });
-    }
     if data_pointers_with_units
         .iter()
         .enumerate()
@@ -4956,9 +5369,10 @@ pub(crate) fn extents_of_the_first_file(
             what: "extent 树里第一个文件的单元有洞（第一版不支持）",
         });
     }
+    let tail = extents.tail();
     let lower_nodes = match extents {
-        ExtentsOfAFileReadFromDisk::LowerSegment(segment) => segment.nodes,
-        ExtentsOfAFileReadFromDisk::NoDataUnit | ExtentsOfAFileReadFromDisk::Inline(_) => {
+        ExtentsOfAFileReadFromDisk::LowerSegment(segment, _) => segment.nodes,
+        ExtentsOfAFileReadFromDisk::NoDataUnit | ExtentsOfAFileReadFromDisk::Inline(..) => {
             Vec::new()
         }
     };
@@ -5032,11 +5446,9 @@ fn extents_assembled_with_the_upper_segment(
             )
         }))
         .collect();
-    Ok(ExtentsOfTheFirstFile {
-        data_pointers: data_pointers_with_units
-            .into_iter()
-            .map(|(_, pointer)| pointer)
-            .collect(),
+    ExtentsOfTheFirstFile {
+        data_pointers,
+        tail,
         version,
         files_other_than_the_first,
         nodes_in_bump_order,
@@ -5420,6 +5832,10 @@ struct MappedUnitCounts {
     extent_tree_nodes: usize,
     allocation_record_tree_nodes: usize,
     accounting_tree_nodes: usize,
+    /// deadlist 树的节点数（零根时 0，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）。
+    deadlist_tree_nodes: usize,
+    /// 被活快照钉住、留存的映射条目数 = deadlist 条目数（判决 S3 的 α：每条 deadlist 条目的映射 key 一条）。
+    mapping_entries_kept_for_the_snapshots: usize,
     /// inode 树的叶容器数 = 它的根的条目数（根恒是层级 1，下面每条条目一片容器；层级另判）。
     inode_leaf_containers: usize,
     /// 数据单元数 = extent 树里第一个文件的数据指针数（下段叶记录条数，或内联的那一个）。
@@ -5438,18 +5854,39 @@ fn mapping_entry_count_is_one_per_mapped_unit(
         + counts.extent_tree_nodes
         + counts.allocation_record_tree_nodes
         + counts.accounting_tree_nodes
+        + counts.deadlist_tree_nodes
+        + counts.mapping_entries_kept_for_the_snapshots
         + counts.inode_leaf_containers
         + counts.data_units
         + counts.deletion_tree_nodes;
     if mapping_entries != expected {
         return Err(RecoveryFailure::InvariantViolated {
             invariant: "E142 走读同款",
-            detail: "映射条目数不是 1 + extent 树、分配记录树、记账树的节点数 + inode 叶容器数 + 数据单元数",
+            detail: "映射条目数不是 1 + extent 树、分配记录树、记账树、deadlist 树的节点数 + deadlist 条目数 + inode 叶容器数 + 数据单元数",
         });
     }
     Ok(())
 }
 
+/// deadlist 树叶里的条目逐条解开（43 字节，`records::DeadlistEntry`），次序照叶里的（key 升序）。
+///
+/// # Errors
+/// 有一条窄于 43 ⇒ `EntryNarrowerThanItsFieldTable`。
+fn deadlist_entries_parsed(
+    leaf_entries_in_key_order: &[Vec<u8>],
+) -> Result<Vec<DeadlistEntry>, RecoveryFailure> {
+    leaf_entries_in_key_order
+        .iter()
+        .map(|bytes| {
+            DeadlistEntry::parse(bytes).ok_or(RecoveryFailure::EntryNarrowerThanItsFieldTable {
+                what: "deadlist 条目",
+                entry_bytes: bytes.len(),
+                field_table_bytes: DEADLIST_ENTRY_KEY_BYTES,
+            })
+        })
+        .collect()
+}
+
 /// 分配记录的跨度不是 0、代不晚于根。已释放的记录合法（D3（空间分配） 已定项 7：改写不删），它的代是释放代，同样不许晚于根。
 fn allocation_record_generations_and_spans_are_judged(
     records: &[AllocationRecord],
@@ -5554,14 +5991,21 @@ fn inode_internal_entry_parsed_and_judged(
     Ok((separator_key, identity, child))
 }
 
+/// 系统配置不带 incompat 位 2 的池里读到截断字段非零的 inode 记录（用户 2026-10-17 定，判决 m4-trunc-r2 第六节）：
+/// 走读交回的 [`RecoveryFailure::InvariantViolated`] 带的那一句，调用方按它认出这一种拒。
+pub const INODE_RECORD_TRUNCATION_FIELD_IN_A_POOL_WITHOUT_INCOMPAT_BIT_TWO: &str =
+    "系统配置不带 incompat 位 2 的池里 inode 记录的截断字段不是全零";
+
 /// 一片 inode 叶容器按指着它的那条内部条目核（I-9.2（条目身份与子头相符）：叶头的四元组身份等于条目的身份引用、记录宽 140、fsid 是本池；
-/// I-1.2：出生树与出生序号等于子指针的、诞生不晚于根、写序实例等于子指针的），再逐条解它的记录、判记录号不小于分隔 key 与容器号（I-9.4）。
-/// 交回解开的记录，按容器里的次序。
+/// I-1.2：出生树与出生序号等于子指针的、诞生不晚于根、写序实例等于子指针的），再逐条解它的记录、判记录号不小于分隔 key 与容器号（I-9.4）、
+/// 截断字段在这个池里收不收（`truncation_fields_of_the_pool`：系统配置不带 incompat 位 2 的池里截断字段非零的记录判不可用，
+/// 判决 `research/prompts/m4-trunc-r2-main-verification.md` 第六节）。交回解开的记录，按容器里的次序。
 fn inode_leaf_container_judged_against_its_entry(
     leaf: &PackedUnitHeader,
     entry: InodeInternalEntry,
     root: &RootRecord,
     expected_filesystem_identifier: u64,
+    truncation_fields_of_the_pool: InodeRecordTruncationFieldsOfThePool,
 ) -> Result<Vec<InodeRecord>, RecoveryFailure> {
     let InodeInternalEntry {
         separator_key,
@@ -5600,26 +6044,51 @@ fn inode_leaf_container_judged_against_its_entry(
                 detail: "inode 记录号小于分隔 key 或容器号",
             });
         }
+        if !record.truncation_field_fits_the_pool(truncation_fields_of_the_pool) {
+            return Err(RecoveryFailure::InvariantViolated {
+                invariant: "I-9.7",
+                detail: INODE_RECORD_TRUNCATION_FIELD_IN_A_POOL_WITHOUT_INCOMPAT_BIT_TWO,
+            });
+        }
         records.push(record);
     }
     Ok(records)
 }
 
-/// 第一个文件的数据指针个数等于 inode size 按净荷容量除出来的单元数（D4（校验和位置） 已定项 5；与写侧切分同一条除法）——
-/// 文件没有洞，第一版不写稀疏文件。
+/// 第一个文件的数据指针个数按上段叶条目的标签对 inode size（D4（校验和位置） 已定项 5 的除法；与写侧切分、挂载态读者同一条，
+/// `write_request_split::data_units_read_within_the_file_size`）：标签 1、2 不少于 size 除出来的单元数（多出来的是截断中间态里
+/// size 之外还挂着的单元，判决 m4-trunc-r1 K2），标签 3、4 少于它（之后到 size 是洞，K5）。交回 size 之内要读、要核的单元数。
 fn data_unit_count_matches_the_inode_size(
     data_units: usize,
+    tail: TailOfAFile,
     inode_record: &InodeRecord,
-) -> Result<(), RecoveryFailure> {
-    if u64::try_from(data_units).expect("单元数")
-        != data_unit_count_of_a_sequential_write(inode_record.size)
-    {
-        return Err(RecoveryFailure::InvariantViolated {
-            invariant: "E142 走读同款",
-            detail: "这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符",
-        });
-    }
-    Ok(())
+) -> Result<usize, RecoveryFailure> {
+    let data_units_of_the_inode = match tail {
+        TailOfAFile::NoHoleAfterTheLastDataUnit => DataUnitsOfTheInode::SomeWritten,
+        TailOfAFile::HoleAfterTheLastDataUnit => DataUnitsOfTheInode::HoleAfterTheLastDataUnit,
+    };
+    data_units_read_within_the_file_size(
+        inode_record.size,
+        u64::try_from(data_units).expect("单元数"),
+        data_units_of_the_inode,
+    )
+    .map(|within| usize::try_from(within).expect("size 之内的单元数不多于单元数"))
+    .map_err(|_refused| RecoveryFailure::InvariantViolated {
+        invariant: "E142 走读同款",
+        detail: "这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符",
+    })
+}
+
+/// 冷走读拼出来的内容补到 size：尾部的洞读出 0、不读盘（判决 m4-trunc-r1 K5）。
+fn content_padded_with_the_hole_up_to_the_size(
+    mut content: Vec<u8>,
+    inode_record: &InodeRecord,
+) -> Vec<u8> {
+    content.resize(
+        usize::try_from(inode_record.size).expect("文件长度装得进 usize"),
+        0,
+    );
+    content
 }
 
 /// 一个文件（第一个文件，或第一个文件之外的文件）的第 `position` 个数据单元在查找路径上的样子：它的数据指针与 extent 树的号
@@ -5813,6 +6282,7 @@ fn pointers_of_the_rebuilt_version_ascend_by_device(
                 .map(|(_, pointer)| pointer),
         )
         .chain(output.accounting_tree.pointers.iter())
+        .chain(output.deadlist_tree.pointers.iter())
         .chain(output.central_mapping_tree.pointers.iter())
         .chain(tombstone_tree_pointers)
         .chain(lineage_tree_pointers)
@@ -5863,6 +6333,28 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
     unit_area_start: UnitAreaStart,
     root: &RootRecord,
     mapping_fallbacks: &mut usize,
+) -> Result<Option<Vec<u8>>, RecoveryFailure> {
+    walk_to_file_judging_the_truncation_fields_as(
+        reader,
+        unit_area_start,
+        InodeRecordTruncationFieldsOfThePool::WRITTEN_BY_THIS_WRITER,
+        root,
+        mapping_fallbacks,
+    )
+}
+
+/// 同 [`walk_to_file_in_the_unit_area_starting_at`]，inode 记录的截断字段按择到的系统配置那一档判（不带 incompat 位 2 的池里
+/// 截断字段非零的记录判不可用，交回 `InvariantViolated`，detail 是 [`INODE_RECORD_TRUNCATION_FIELD_IN_A_POOL_WITHOUT_INCOMPAT_BIT_TWO`]）。
+/// 恢复与 [`walk_to_file`] 走这一份。
+///
+/// # Errors
+/// 同 [`walk_to_file_in_the_unit_area_starting_at`]。
+pub fn walk_to_file_judging_the_truncation_fields_as(
+    reader: &dyn PoolReader,
+    unit_area_start: UnitAreaStart,
+    truncation_fields_of_the_pool: InodeRecordTruncationFieldsOfThePool,
+    root: &RootRecord,
+    mapping_fallbacks: &mut usize,
 ) -> Result<Option<Vec<u8>>, RecoveryFailure> {
     let expected_filesystem_identifier = unit_filesystem_identifier(&root.filesystem_identifier);
     let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
@@ -5923,10 +6415,18 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
     let mut by_kind: BTreeMap<u16, IndexNodeHeader> = BTreeMap::new();
     let mut mapped_units_of_each_container_tree: BTreeMap<u16, usize> = BTreeMap::new();
     let mut accounting_tree: Option<CodeTwoTreeReadFromDisk> = None;
+    let mut deadlist_tree: Option<CodeTwoTreeReadFromDisk> = None;
     let mut allocation_tree: Option<AllocationRecordTreeReadFromDisk> = None;
     let mut extent_tree: Option<(TreeIdentifier, ExtentsOfTheFirstFile)> = None;
     for entry in &entries {
         tree_identifier_of_the_entry_is_below_the_watermark(entry, root)?;
+        // 只读快照的两种条目（码 14、15）：它们的根由快照的读路径解（判决 `research/prompts/m4-batch2-code-r1-main-verification.md`
+        // 第二节 S2；这一批不解），冷走读读的是活头那一版的文件，跳过它们。
+        if entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT
+            || entry.kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT
+        {
+            continue;
+        }
         if entry.root == NodePointer::empty_root() {
             continue;
         }
@@ -5994,6 +6494,27 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
             )?);
             continue;
         }
+        // deadlist 树（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）：多层码 2 树，同记账树整棵读回来；
+        // 它的节点与它的条目各占映射条目（条目留存的那几条，判决 S3 的 α），数映射条目时要它们。
+        if entry.kind == TREE_KIND_DEADLIST {
+            deadlist_tree = Some(read_code_two_tree(
+                &entry.root,
+                &MultiLevelCodeTwoTree::Deadlist.read_expectation(entry.tree),
+                CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,
+                root,
+                expected_filesystem_identifier,
+                &mut |pointer: &NodePointer| {
+                    read_mapped_tree_node_via_hint_then_central_mapping(
+                        reader,
+                        pointer,
+                        MappedTreeNodeClass::IndexNode,
+                        &central_mapping_locations_of_key,
+                        mapping_fallbacks,
+                    )
+                },
+            )?);
+            continue;
+        }
         // 墓碑树与谱系树：从根往下读码 2 节点（根满长高之后不止一层）、数进映射的节点与容器，不读容器（它们的内容不影响读回的文件）。
         if entry.kind == TREE_KIND_TOMBSTONE
             || entry.kind == TREE_KIND_LINEAGE
@@ -6049,6 +6570,7 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
             .unwrap_or(0)
     };
     let deletion_tree_nodes = usize::from(by_kind.contains_key(&TREE_KIND_DELETED_INODES))
+        + usize::from(by_kind.contains_key(&TREE_KIND_LOGGED_OPERATIONS))
         + container_tree_nodes(TREE_KIND_TOMBSTONE)
         + container_tree_nodes(TREE_KIND_LINEAGE)
         + container_tree_nodes(TREE_KIND_RETIREMENT);
@@ -6101,6 +6623,12 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
             extent_tree_nodes: roots.extent.nodes_in_bump_order.len(),
             allocation_record_tree_nodes: roots.allocation.nodes.len(),
             accounting_tree_nodes: roots.accounting.version.node_count(),
+            deadlist_tree_nodes: deadlist_tree
+                .as_ref()
+                .map_or(0, |read| read.version.node_count()),
+            mapping_entries_kept_for_the_snapshots: deadlist_tree
+                .as_ref()
+                .map_or(0, |read| read.leaf_entries_in_key_order.len()),
             inode_leaf_containers: roots.inode.entries.len(),
             data_units: roots.extent.data_units_of_every_file(),
             deletion_tree_nodes,
@@ -6137,6 +6665,7 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
             },
             root,
             expected_filesystem_identifier,
+            truncation_fields_of_the_pool,
         )?;
         for record in records {
             if record.inode == FIRST_INODE_NUMBER {
@@ -6156,11 +6685,21 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
     // (locality 0, inode 1, 第 i 个单元第一个字节的文件偏移)、单元从 0 起连号，D8（核心索引结构） 已定项 3 / 已定项 14）。
     // 个数要等于 inode size 按净荷容量除出来的单元数（D4（校验和位置） 已定项 5；与写侧切分同一条除法）——文件没有洞，第一版不写稀疏文件。
     // 解引用先按位置提示、读不到再查映射（D19（块指针的结构与宽度预算） 已定项 5）。
-    data_unit_count_matches_the_inode_size(roots.extent.data_pointers.len(), &inode_record)?;
+    let data_units_within_the_size = data_unit_count_matches_the_inode_size(
+        roots.extent.data_pointers.len(),
+        roots.extent.tail,
+        &inode_record,
+    )?;
     let mut content: Vec<u8> =
         Vec::with_capacity(usize::try_from(inode_record.size).expect("文件长度装得进 usize"));
-    // 迭代次数的上界是这个文件的单元数（上面核过等于 inode size 除出来的单元数）；跨轮携带的只有已经拼出来的内容，每一个提前出口都是交回一个错。
-    for (position, pointer) in roots.extent.data_pointers.iter().enumerate() {
+    // 迭代次数的上界是 size 之内的单元数（上面按标签核过）；跨轮携带的只有已经拼出来的内容，每一个提前出口都是交回一个错。
+    for (position, pointer) in roots
+        .extent
+        .data_pointers
+        .iter()
+        .take(data_units_within_the_size)
+        .enumerate()
+    {
         let bytes = read_data_unit_via_hint_then_central_mapping(
             reader,
             pointer,
@@ -6185,7 +6724,175 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
         )?;
         content.extend_from_slice(payload);
     }
-    Ok(Some(content))
+    Ok(Some(content_padded_with_the_hole_up_to_the_size(
+        content,
+        &inode_record,
+    )))
+}
+
+/// 读一个只读快照里的第一个文件（[`read_the_first_file_of_a_snapshot`]）读出了什么。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum SnapshotFileReading {
+    /// 这条根的树表里没有树号是那个号的码 14 条目。
+    NoSnapshotWithThisTree,
+    /// 快照那一刻没有第一个文件（inode 1 没有记录，或已在删除中、nlink = 0）。
+    NoFile,
+    /// 第一个文件的内容。
+    File(Vec<u8>),
+}
+
+/// 读这条根的树表里只读快照 `snapshot`（码 14 的树号，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 5）指着的
+/// 第一个文件：从它那一对码 14 / 15 条目的两条根指针往下读 inode 树与 extent 树，节点按根指针头部的出生树核（判决 S1「读快照指着的根按根指针头部
+/// 的出生树比、不按条目自己的树号」），单元的位置提示读不出时经**这条根**的中央映射回退——快照引用、活头已换下的单元靠的就是留存在这里的
+/// 映射条目（判决 S3 的 α）。判定与冷走读那几道相同（inode 树根层级 1、叶容器与条目对得上、单元个数对得上 size、数据单元按查找路径核）。
+///
+/// # Errors
+/// 树表读不出、解不开、条目不合 I-9.16 或配不成对；快照的两棵树、叶容器或数据单元读不出（提示与映射都读不出）、解不开或判红。
+pub fn read_the_first_file_of_a_snapshot(
+    reader: &dyn PoolReader,
+    root: &RootRecord,
+    snapshot: TreeIdentifier,
+) -> Result<SnapshotFileReading, RecoveryFailure> {
+    let expected_filesystem_identifier = unit_filesystem_identifier(&root.filesystem_identifier);
+    let tree_table_bytes = read_unit_via_locations(
+        reader,
+        &root.tree_table.locations,
+        usize::try_from(NODE_BYTES).expect("16384"),
+    )?;
+    let tree_table =
+        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
+            what: "树表单元",
+        })?;
+    tree_table_key_width_is_the_tree_tables(&tree_table)?;
+    let entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)?;
+    let Some(pair) = snapshot_tree_table_entries_paired_one_to_one(&entries)?
+        .into_iter()
+        .find(|pair| pair.inode_root.tree == snapshot)
+    else {
+        return Ok(SnapshotFileReading::NoSnapshotWithThisTree);
+    };
+    let central_mapping_tree = CentralMappingTreeReadOnFirstUse {
+        reader,
+        root,
+        expected_filesystem_identifier,
+        tree: OnceCell::new(),
+    };
+    let central_mapping_locations_of_key =
+        |mapping_key: &[u8]| central_mapping_tree.locations_of_key(mapping_key);
+    let mut stale_location_hint_hops_not_exposed_by_this_reader = 0usize;
+    let extent_root_pointer = pair.extent_root.root;
+    let extents = extents_of_the_first_file(read_extent_tree(
+        &ExtentTreeReading {
+            tree: extent_root_pointer.head.birth_tree,
+            judgement: ExtentTreeHeaderJudgement::EveryHeaderAgainstItsReference,
+            root,
+            expected_filesystem_identifier,
+        },
+        &extent_root_pointer,
+        &mut |pointer: &NodePointer| {
+            read_mapped_tree_node_via_hint_then_central_mapping(
+                reader,
+                pointer,
+                MappedTreeNodeClass::IndexNode,
+                &central_mapping_locations_of_key,
+                &mut stale_location_hint_hops_not_exposed_by_this_reader,
+            )
+        },
+    )?)?;
+    let inode_root_pointer = pair.inode_root.root;
+    let inode_root_bytes = read_mapped_tree_node_via_hint_then_central_mapping(
+        reader,
+        &inode_root_pointer,
+        MappedTreeNodeClass::IndexNode,
+        &central_mapping_locations_of_key,
+        &mut stale_location_hint_hops_not_exposed_by_this_reader,
+    )?;
+    let inode_root = tree_root_checked_against_its_pointer(
+        &inode_root_bytes,
+        inode_root_pointer.head.birth_tree,
+        key_width_for_kind(TREE_KIND_INODE).expect("inode 树的 key 宽登记过"),
+        &inode_root_pointer,
+        root,
+        expected_filesystem_identifier,
+    )?;
+    inode_tree_root_level_is_one(&inode_root)?;
+    // 只读快照住在挂着的池里：可写挂载取号那一写起系统配置就带 incompat 位 2，截断字段照带位 2 的池收。
+    let truncation_fields_of_the_pool =
+        InodeRecordTruncationFieldsOfThePool::WRITTEN_BY_THIS_WRITER;
+    let mut inode_record: Option<InodeRecord> = None;
+    for entry in &inode_root.entries {
+        let (separator_key, identity, child) = inode_internal_entry_parsed_and_judged(entry)?;
+        let leaf_bytes = read_mapped_tree_node_via_hint_then_central_mapping(
+            reader,
+            &child,
+            MappedTreeNodeClass::PackedRecordUnit,
+            &central_mapping_locations_of_key,
+            &mut stale_location_hint_hops_not_exposed_by_this_reader,
+        )?;
+        let leaf =
+            parse_packed_unit(&leaf_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
+                what: "inode 叶容器",
+            })?;
+        let records = inode_leaf_container_judged_against_its_entry(
+            &leaf,
+            InodeInternalEntry {
+                separator_key,
+                identity,
+                child,
+            },
+            root,
+            expected_filesystem_identifier,
+            truncation_fields_of_the_pool,
+        )?;
+        if let Some(record) = records
+            .into_iter()
+            .find(|record| record.inode == FIRST_INODE_NUMBER)
+        {
+            inode_record = Some(record);
+        }
+    }
+    let Some(inode_record) = inode_record else {
+        return Ok(SnapshotFileReading::NoFile);
+    };
+    if inode_record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {
+        return Ok(SnapshotFileReading::NoFile);
+    }
+    let data_units_within_the_size = data_unit_count_matches_the_inode_size(
+        extents.data_pointers.len(),
+        extents.tail,
+        &inode_record,
+    )?;
+    let mut content: Vec<u8> =
+        Vec::with_capacity(usize::try_from(inode_record.size).expect("文件长度装得进 usize"));
+    // 迭代次数的上界是 size 之内的单元数（上面按标签核过）；跨轮携带的只有已经拼出来的内容。
+    for (position, pointer) in extents
+        .data_pointers
+        .iter()
+        .take(data_units_within_the_size)
+        .enumerate()
+    {
+        let bytes = read_data_unit_via_hint_then_central_mapping(
+            reader,
+            pointer,
+            &central_mapping_locations_of_key,
+            &mut stale_location_hint_hops_not_exposed_by_this_reader,
+        )?
+        .into_content()?;
+        let payload = data_unit_judged_against_its_pointer_and_the_inode_record(
+            &bytes,
+            DataUnitOfAFile {
+                position,
+                pointer,
+                extent_tree: extent_root_pointer.head.birth_tree,
+            },
+            &inode_record,
+            expected_filesystem_identifier,
+        )?;
+        content.extend_from_slice(payload);
+    }
+    Ok(SnapshotFileReading::File(
+        content_padded_with_the_hole_up_to_the_size(content, &inode_record),
+    ))
 }
 
 /// 同 [`walk_to_file_in_the_unit_area_starting_at`]，单元区起点先择一次系统配置现算（[`choose_system_configuration`]：
@@ -6198,9 +6905,13 @@ pub fn walk_to_file(
     root: &RootRecord,
     mapping_fallbacks: &mut usize,
 ) -> Result<Option<Vec<u8>>, RecoveryFailure> {
-    walk_to_file_in_the_unit_area_starting_at(
+    let system_configuration = choose_system_configuration(reader)?;
+    walk_to_file_judging_the_truncation_fields_as(
         reader,
-        unit_area_start_of_the_pool(reader)?,
+        unit_area_start_of_the_chosen_system_configuration(&system_configuration),
+        system_configuration
+            .immutable
+            .inode_record_truncation_fields,
         root,
         mapping_fallbacks,
     )
@@ -6216,9 +6927,35 @@ pub fn rebuild_version(
     root: &RootRecord,
     record_standing_for_root: Option<JournalRecord>,
 ) -> Result<RebuiltVersion, RebuildVersionFailure> {
-    rebuild_version_in_the_unit_area_starting_at(
+    let system_configuration = choose_system_configuration(reader)?;
+    rebuild_version_judging_the_truncation_fields_as(
         reader,
-        unit_area_start_of_the_pool(reader)?,
+        unit_area_start_of_the_chosen_system_configuration(&system_configuration),
+        system_configuration
+            .immutable
+            .inode_record_truncation_fields,
+        root,
+        record_standing_for_root,
+    )
+}
+
+/// 同 [`rebuild_version`]（单元区起点先择一次系统配置现算），另交出树表里的只读快照条目
+/// （[`rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at`]）。给手里没有择好的系统配置的调用方（用例）。
+///
+/// # Errors
+/// 同 [`rebuild_version`]。
+pub fn rebuild_version_with_its_snapshot_entries(
+    reader: &dyn PoolReader,
+    root: &RootRecord,
+    record_standing_for_root: Option<JournalRecord>,
+) -> Result<RebuiltVersionWithItsSnapshotEntries, RebuildVersionFailure> {
+    let system_configuration = choose_system_configuration(reader)?;
+    rebuild_version_with_its_snapshot_entries_judging_the_truncation_fields_as(
+        reader,
+        unit_area_start_of_the_chosen_system_configuration(&system_configuration),
+        system_configuration
+            .immutable
+            .inode_record_truncation_fields,
         root,
         record_standing_for_root,
     )
@@ -6454,9 +7191,13 @@ pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport
         }
     };
     let unit_area_start = unit_area_start_of_the_chosen_system_configuration(&system_configuration);
-    let (outcome, effective_root) = match walk_to_file_in_the_unit_area_starting_at(
+    let truncation_fields_of_the_pool = system_configuration
+        .immutable
+        .inode_record_truncation_fields;
+    let (outcome, effective_root) = match walk_to_file_judging_the_truncation_fields_as(
         reader,
         unit_area_start,
+        truncation_fields_of_the_pool,
         &effective_root,
         &mut mapping_fallbacks,
     ) {
@@ -6483,9 +7224,10 @@ pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport
                 JournalPolicy::Ignore => None,
             }
             .and_then(|version_fallen_back_to| {
-                walk_to_file_in_the_unit_area_starting_at(
+                walk_to_file_judging_the_truncation_fields_as(
                     reader,
                     unit_area_start,
+                    truncation_fields_of_the_pool,
                     &version_fallen_back_to,
                     &mut mapping_fallbacks,
                 )
@@ -6987,7 +7729,7 @@ mod allocation_record_pool_geometry_tests {
 }
 
 /// 读侧认得删除新写出的三棵树的 key 宽（里程碑三·2 第 7 条「读侧补新种类的 key 宽」）：根指针不全零时冷走读按它读根、核根的自述 key 宽；
-/// 不认得的种类在那一步拒成「树的种类没登记」。logged_ops 还没有字段表，照旧不认。
+/// 不认得的种类在那一步拒成「树的种类没登记」。logged_ops 的 key 是种类 2 + 签发 txg 8（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1）。
 #[cfg(test)]
 mod tree_kind_key_width_tests {
     use super::key_width_for_kind;
@@ -6998,8 +7740,8 @@ mod tree_kind_key_width_tests {
     };
 
     #[test]
-    fn deleted_inodes_tombstone_and_lineage_trees_have_eight_byte_keys_and_logged_operations_none()
-    {
+    fn deleted_inodes_tombstone_lineage_and_retirement_trees_have_eight_byte_keys_and_logged_operations_ten(
+    ) {
         assert_eq!(
             key_width_for_kind(TREE_KIND_DELETED_INODES),
             Some(8),
@@ -7022,8 +7764,8 @@ mod tree_kind_key_width_tests {
         );
         assert_eq!(
             key_width_for_kind(TREE_KIND_LOGGED_OPERATIONS),
-            None,
-            "logged_ops 没有字段表"
+            Some(10),
+            "logged_ops 的 key = 种类 2 + 签发 txg 8"
         );
         assert_eq!(
             key_width_for_kind(TREE_KIND_INODE),
diff --git a/crates/singlefs-reference-filesystem/src/lib.rs b/crates/singlefs-reference-filesystem/src/lib.rs
index 905dfab..4a6c610 100644
--- a/crates/singlefs-reference-filesystem/src/lib.rs
+++ b/crates/singlefs-reference-filesystem/src/lib.rs
@@ -15,6 +15,8 @@
 //! 只管串行提交；任意子集的丢法不做（r5 判决第四节第 8 条：串行提交下出局）。自动触发的攒批发布落地之前只接显式发布点（r6 判决第二节 U2 一行）。
 #![forbid(unsafe_code)]
 
+use std::collections::BTreeMap;
+
 /// 一个文件的全部字节。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct FileContent(pub Vec<u8>);
@@ -26,6 +28,15 @@ pub enum VisibleState {
     OneFile(FileContent),
 }
 
+/// 一个只读快照的名字：调用方按实现点名它的那个号交入（实现按码 14 条目的树号点名，判决
+/// `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 5）。这个包不解释它，只拿它当键。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub struct SnapshotName(pub u64);
+
+/// 活着的只读快照：每个快照名对应建它那一刻 POSIX 面上看得见的状态（判决 `research/prompts/m4-snapshot-r1-main-verification.md`
+/// 第三节 S7「RefFS 加 CreateSnapshot、DestroySnapshot、读快照，状态加快照集合」）。
+pub type Snapshots = BTreeMap<SnapshotName, VisibleState>;
+
 /// 第一版的操作。崩溃不在这里，它是 [`ReferenceFileSystem::crash_and_recover`]。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum Operation {
@@ -38,6 +49,14 @@ pub enum Operation {
     /// 回退到第 m 版的内容；m 由调用方按实现报的回退根在「根 → 版本」表里查出来，选得对不对这里不判。
     /// 回退是挂着时的一次向前发布：返回成功时抬起下界 lo，它的下标是发布末尾，它之后的内容是一个新版本。
     RollBackToVersion(VersionNumber),
+    /// 建一个只读快照，名字是 `SnapshotName`：捕获这一刻的状态。它是一次独立发布（判决 S2），返回成功时抬起下界 lo。
+    CreateSnapshot(SnapshotName),
+    /// 删一个只读快照（一次调用 = 意图 + 全部批，判决第七节岔路 2、4）：返回成功时它不在了，抬起下界 lo。POSIX 面上的文件不变。
+    DestroySnapshot(SnapshotName),
+    /// 把文件截到这么多字节（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K8）：比现在短就截掉尾部，比现在长就补 0。
+    /// 内容改动型（同 [`Operation::OverwriteTheFile`]），不是发布点：实现的意图与各批落在一次调用里，崩在里面读出的只有截前、截后两样（K2「先落目标」），
+    /// 就是这个操作施加之前与之后的状态，不用另起一类「一个操作的中间内容」。
+    Truncate(u64),
 }
 
 /// 调用方交入的结局。
@@ -88,11 +107,29 @@ pub enum OperationNotAccepted {
     CreatingTheFirstFileWhileAFileExistsIsNotInTheFirstVersion,
     /// 还没有文件时覆盖写：条款没写它在 POSIX 面上是什么，第一版不支持。
     OverwritingWithoutAFileIsNotInTheFirstVersion,
+    /// 还没有文件时截断：实现在任何写之前拒（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1「对 nlink = 0 或不在树里的 inode
+    /// 截断，在任何写之前拒」），调用方交的是「写之前拒」、走不到这里；走到了就是输入不合法。
+    TruncatingWithoutAFileIsNotInTheFirstVersion,
     /// 回退目标 m 不是 1 到「到这一步为止的版本数」之间的数。
     RollBackTargetIsNotAVersion {
         requested: VersionNumber,
         versions_so_far: usize,
     },
+    /// 建快照用了一个已经活着的快照名（允许的每一个快照集合里它都活着）。
+    SnapshotNameAlreadyTaken { name: SnapshotName },
+    /// 删一个不活着的快照（允许的每一个快照集合里它都不在）。
+    NoSnapshotWithThisName { name: SnapshotName },
+    /// 有活快照时回退：第一版有快照就拒回退（用户 2026-10-13 定，判决第三节 S8 与第七节岔路 3）；允许的每一个快照集合都不空。
+    RollBackWithSnapshotsIsNotInTheFirstVersion,
+}
+
+/// 读快照读出的内容不是允许的任何一个快照集合里那个快照的内容：判红；返回它时模型一个字段都没动。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct SnapshotReadOutsideTheAllowedReads {
+    pub name: SnapshotName,
+    pub read_out: VisibleState,
+    /// 允许的每一个快照集合里这个快照读出什么（`None` = 那个集合里没有它），去重。
+    pub allowed_reads: Vec<Option<VisibleState>>,
 }
 
 /// 允许集合里的一个状态与它的下标。
@@ -139,6 +176,32 @@ pub enum CrashNotAccepted {
     },
 }
 
+/// 抛弃最新根的那次恢复，被抛弃的那条根与 RefFS 最新那一版（[`ReferenceFileSystem::newest_version`]）是什么关系：调用方按它攒的
+/// 「根 → 版本」表交入（判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1，用户 2026-10-16 定「乙：翻译交进来」）。
+/// RefFS 只收操作序列与读出的内容，自己看不出被抛弃的是哪一条根：实现的一版常有几条根（可写挂载的写行与暖机、整理的各批、
+/// 为空间推的抬 F、卸载都映到写它时的现行版本；删快照一次调用写意图与全部批）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum AbandonedRoot {
+    /// 被抛弃的是写出最新那一版的那一条根（发布调用、fsync、回退、建快照的根，删快照的最后一批）：恢复落到它前一条根，
+    /// 在之前那一版上。
+    WroteTheNewestVersion,
+    /// 被抛弃的是映到最新那一版、但不是写出它的那一条根（这一版写出之后的写行与暖机、整理的批、抬 F、卸载，恢复重新起头之后写的根）：
+    /// 恢复落到的前一条根仍在这一版上。
+    WrittenAfterTheNewestVersion,
+}
+
+/// 抛弃最新根的那次恢复判不了；返回它时模型一个字段都没动。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum AbandonedRootRestartNotAccepted {
+    /// 允许集合判不了：只会是 hi 越界、或写了之后报错的操作后面到崩溃为止又起了操作两种（同
+    /// [`ReferenceFileSystem::allowed_set_for_a_crash_in`]）；读出的内容这一步不判，不会是判红那一种。
+    TheCrashIsNotAccepted(CrashNotAccepted),
+    /// 调用方说被抛弃的根写出了最新那一版，而允许集合里有 S_0（上一次恢复之后没有已确认的发布）：S_0 是上一次恢复读出、
+    /// 重新起头的那一版（还没崩溃过时是起点），它之前那一版在上一次恢复之前（还没崩溃过时没有），RefFS 不记那时的快照集合。
+    /// 判决第三节 J1 没写这一支取什么，第一版不支持。
+    TheRootThatWroteTheStateAtTheLastRecoveryIsNotInTheFirstVersion,
+}
+
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum EnteredOutcome {
     Succeeded,
@@ -151,6 +214,8 @@ enum EnteredKind {
     ExplicitPublish,
     Fsync,
     RollBack,
+    CreateSnapshot,
+    DestroySnapshot,
 }
 
 impl EnteredKind {
@@ -162,9 +227,90 @@ impl EnteredKind {
             EnteredKind::ExplicitPublish | EnteredKind::Fsync => true,
             // 回退是挂着时的一次向前发布：D23（journal 的角色与格式） .claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换
             EnteredKind::RollBack => true,
+            // 建快照是一次独立发布（判决 S2）；删快照一次调用发意图与全部批，返回成功时最后一批已落盘（判决第七节岔路 2、4）。
+            EnteredKind::CreateSnapshot | EnteredKind::DestroySnapshot => true,
             EnteredKind::ContentChange => false,
         }
     }
+
+    /// 这个操作写出一版时，最后一条根之前写的那几条根上它的快照改动是不是已经起了头：删快照一次调用写意图与全部批，
+    /// 抛弃最后一批的那次恢复落到意图或不是最后一批的批上，快照带销毁中记号、读照读，下一次用户改动先走空把它删完，
+    /// 前后两份快照集合都可能（判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1「删快照这种一个操作写多条根的取前后两份」）。
+    /// 别的发布最后一条之前的根是会话推的抬 F，快照集合还是之前那一版的。
+    fn its_earlier_roots_may_carry_its_snapshot_change(self) -> bool {
+        match self {
+            EnteredKind::DestroySnapshot => true,
+            EnteredKind::ContentChange
+            | EnteredKind::ExplicitPublish
+            | EnteredKind::Fsync
+            | EnteredKind::RollBack
+            | EnteredKind::CreateSnapshot => false,
+        }
+    }
+}
+
+/// 一个操作怎么改快照集合。不按「之后的快照集合」记：恢复之后允许的快照集合可以不止一个（判决
+/// `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F6），同一个操作在每一个上各施加一次。
+#[derive(Clone, Debug, PartialEq, Eq)]
+enum SnapshotChangeOfTheOperation {
+    Unchanged,
+    Created {
+        name: SnapshotName,
+        captured: VisibleState,
+    },
+    Destroyed {
+        name: SnapshotName,
+    },
+}
+
+impl SnapshotChangeOfTheOperation {
+    fn apply_to(&self, snapshots: &mut Snapshots) {
+        match self {
+            SnapshotChangeOfTheOperation::Unchanged => {}
+            SnapshotChangeOfTheOperation::Created { name, captured } => {
+                snapshots.insert(*name, captured.clone());
+            }
+            SnapshotChangeOfTheOperation::Destroyed { name } => {
+                snapshots.remove(name);
+            }
+        }
+    }
+}
+
+/// 实现做了这个操作（进了序列）说明它那一刻的快照集合满足什么：实现在任何写之前拒掉不满足的，拒掉的不进序列。
+/// 允许的快照集合里不满足的那几个由此排除（同一段历史里实现只有一个快照集合）。不看快照集合的操作没有这一样。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum SnapshotPreconditionOfTheOperation {
+    NameFree(SnapshotName),
+    SnapshotPresent(SnapshotName),
+    NoSnapshotAtAll,
+}
+
+impl SnapshotPreconditionOfTheOperation {
+    fn holds_in(self, snapshots: &Snapshots) -> bool {
+        match self {
+            SnapshotPreconditionOfTheOperation::NameFree(name) => !snapshots.contains_key(&name),
+            SnapshotPreconditionOfTheOperation::SnapshotPresent(name) => {
+                snapshots.contains_key(&name)
+            }
+            SnapshotPreconditionOfTheOperation::NoSnapshotAtAll => snapshots.is_empty(),
+        }
+    }
+
+    /// 允许的快照集合一个都不满足时不收这个操作的那一条。
+    fn refusal_when_no_allowed_snapshot_set_meets_it(self) -> OperationNotAccepted {
+        match self {
+            SnapshotPreconditionOfTheOperation::NameFree(name) => {
+                OperationNotAccepted::SnapshotNameAlreadyTaken { name }
+            }
+            SnapshotPreconditionOfTheOperation::SnapshotPresent(name) => {
+                OperationNotAccepted::NoSnapshotWithThisName { name }
+            }
+            SnapshotPreconditionOfTheOperation::NoSnapshotAtAll => {
+                OperationNotAccepted::RollBackWithSnapshotsIsNotInTheFirstVersion
+            }
+        }
+    }
 }
 
 #[derive(Clone, Debug, PartialEq, Eq)]
@@ -172,6 +318,14 @@ struct EnteredOperation {
     kind: EnteredKind,
     outcome: EnteredOutcome,
     state_after: VisibleState,
+    snapshot_change: SnapshotChangeOfTheOperation,
+}
+
+/// 往允许的快照集合里加一个，已经有相同的就不加（集合按内容去重）。
+fn push_distinct_snapshot_set(snapshot_sets: &mut Vec<Snapshots>, snapshots: Snapshots) {
+    if !snapshot_sets.contains(&snapshots) {
+        snapshot_sets.push(snapshots);
+    }
 }
 
 /// 序列里已确认的发布（返回成功的发布调用、fsync 与回退，原样重发成功的也算）各自发布之后的内容，按历史次序。
@@ -261,6 +415,11 @@ pub struct ReferenceFileSystem {
     operations_since_the_last_recovery: Vec<EnteredOperation>,
     /// 上一次恢复为止的版本，按历史次序：起点、之前各轮里已确认的发布之后的内容、各次恢复读出的那一份；第 m 版在位置 m − 1。
     versions_up_to_the_last_recovery: Vec<VisibleState>,
+    /// S_0 那一刻允许活着的快照集合，去重、至少一个：还没崩溃过时只有「没有快照」；恢复之后是允许集合里与读出的内容相同的每一个状态那一刻的
+    /// 快照集合（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F6），抛弃最新根的那次恢复之后按调用方交的
+    /// [`AbandonedRoot`] 取（判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1），再排除之后实现做了的操作说不通的那几个。
+    /// 序列里每个操作在每一个上各施加一次（[`SnapshotChangeOfTheOperation`]），第 k 个下标与 S_0 那一刻的第 k 个一一对应。
+    allowed_snapshot_sets_at_the_last_recovery: Vec<Snapshots>,
 }
 
 impl ReferenceFileSystem {
@@ -270,9 +429,79 @@ impl ReferenceFileSystem {
             operations_since_the_last_recovery: Vec::new(),
             // 起点是第 1 版：mkfs 写的根与 mkfs 之后可写挂载写的根映到它（r6 用户定案 H1-收严「m 的编号覆盖每一个写根的入口」）
             versions_up_to_the_last_recovery: vec![VisibleState::NoFile],
+            allowed_snapshot_sets_at_the_last_recovery: vec![Snapshots::new()],
         }
     }
 
+    /// S_j 那一刻的快照集合，与 S_0 那一刻允许的快照集合逐个对应（不去重）：每一个施加序列里前 j 个操作。
+    fn snapshot_set_of_each_allowed_start_at(&self, index: StateIndex) -> Vec<Snapshots> {
+        let operations = &self.operations_since_the_last_recovery[..index.0];
+        self.allowed_snapshot_sets_at_the_last_recovery
+            .iter()
+            .map(|snapshots_at_the_last_recovery| {
+                let mut snapshots = snapshots_at_the_last_recovery.clone();
+                for operation in operations {
+                    operation.snapshot_change.apply_to(&mut snapshots);
+                }
+                snapshots
+            })
+            .collect()
+    }
+
+    /// 序列里的操作全都做成时允许活着的快照集合，去重：恢复之后读出的文件定不了落在哪一个发布末尾时不止一个（判决第三节 F6）。
+    pub fn allowed_snapshot_sets(&self) -> Vec<Snapshots> {
+        let mut allowed = Vec::new();
+        for snapshots in self.snapshot_set_of_each_allowed_start_at(StateIndex(
+            self.operations_since_the_last_recovery.len(),
+        )) {
+            push_distinct_snapshot_set(&mut allowed, snapshots);
+        }
+        allowed
+    }
+
+    /// 读快照 `name` 允许读出什么（序列里的操作全都做成时）：允许的每一个快照集合里它活着时是建它那一刻的状态，不活着是 `None`；去重。
+    pub fn allowed_reads_of_the_snapshot(&self, name: SnapshotName) -> Vec<Option<VisibleState>> {
+        let mut allowed_reads = Vec::new();
+        for snapshots in self.allowed_snapshot_sets() {
+            let read = snapshots.get(&name).cloned();
+            if !allowed_reads.contains(&read) {
+                allowed_reads.push(read);
+            }
+        }
+        allowed_reads
+    }
+
+    /// 实现读快照 `name` 读出 `read_out`（序列里的操作全都做成时）：它要是允许的某一个快照集合里那个快照的内容；
+    /// 是的话只留下读得出它的那几个快照集合（实现只有一个快照集合，读出的内容说明是哪几个）。
+    ///
+    /// # Errors
+    /// 允许的快照集合里没有一个读得出它：判红，模型不动。
+    pub fn judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
+        &mut self,
+        name: SnapshotName,
+        read_out: &VisibleState,
+    ) -> Result<(), SnapshotReadOutsideTheAllowedReads> {
+        let current = self.snapshot_set_of_each_allowed_start_at(StateIndex(
+            self.operations_since_the_last_recovery.len(),
+        ));
+        let kept: Vec<Snapshots> = self
+            .allowed_snapshot_sets_at_the_last_recovery
+            .iter()
+            .zip(&current)
+            .filter(|(_, snapshots)| snapshots.get(&name) == Some(read_out))
+            .map(|(snapshots_at_the_last_recovery, _)| snapshots_at_the_last_recovery.clone())
+            .collect();
+        if kept.is_empty() {
+            return Err(SnapshotReadOutsideTheAllowedReads {
+                name,
+                read_out: read_out.clone(),
+                allowed_reads: self.allowed_reads_of_the_snapshot(name),
+            });
+        }
+        self.allowed_snapshot_sets_at_the_last_recovery = kept;
+        Ok(())
+    }
+
     /// 到这一刻为止的版本，按历史次序（第 m 版在位置 m − 1）。
     fn versions_so_far(&self) -> Vec<&VisibleState> {
         self.versions_up_to_the_last_recovery
@@ -345,27 +574,122 @@ impl ReferenceFileSystem {
             // 写了之后报错＝两可：D16（发布语义） .claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么
             OperationOutcome::FailedAfterWriting => EnteredOutcome::FailedAfterWriting,
         };
+        // 实现做了这个操作说明它那一刻的快照集合满足什么（[`SnapshotPreconditionOfTheOperation`]）：允许的快照集合里不满足的排除，
+        // 一个都不满足就不收。回退先判快照、再判目标版本（两样都不收时报快照那一条）。
+        let snapshot_precondition = match &operation {
+            Operation::CreateTheFirstFile(_)
+            | Operation::OverwriteTheFile(_)
+            | Operation::ExplicitPublish
+            | Operation::Fsync
+            | Operation::Truncate(_) => None,
+            // 第一版有快照就拒回退（判决 S8）：实现在任何写之前拒，调用方交的是「写之前拒」，走不到这里；允许的快照集合都不空时走到了就是输入不合法。
+            Operation::RollBackToVersion(_) => {
+                Some(SnapshotPreconditionOfTheOperation::NoSnapshotAtAll)
+            }
+            Operation::CreateSnapshot(name) => {
+                Some(SnapshotPreconditionOfTheOperation::NameFree(*name))
+            }
+            Operation::DestroySnapshot(name) => {
+                Some(SnapshotPreconditionOfTheOperation::SnapshotPresent(*name))
+            }
+        };
+        let allowed_snapshot_sets_at_the_last_recovery = match snapshot_precondition {
+            None => self.allowed_snapshot_sets_at_the_last_recovery.clone(),
+            Some(precondition) => {
+                let current = self.snapshot_set_of_each_allowed_start_at(StateIndex(
+                    self.operations_since_the_last_recovery.len(),
+                ));
+                let kept: Vec<Snapshots> = self
+                    .allowed_snapshot_sets_at_the_last_recovery
+                    .iter()
+                    .zip(&current)
+                    .filter(|(_, snapshots)| precondition.holds_in(snapshots))
+                    .map(|(snapshots_at_the_last_recovery, _)| {
+                        snapshots_at_the_last_recovery.clone()
+                    })
+                    .collect();
+                if kept.is_empty() {
+                    return Err(precondition.refusal_when_no_allowed_snapshot_set_meets_it());
+                }
+                kept
+            }
+        };
         let state_so_far = self.state_after_every_operation_so_far();
-        let (kind, state_after) = match operation {
+        let (kind, state_after, snapshot_change) = match operation {
             Operation::CreateTheFirstFile(content) => match state_so_far {
-                VisibleState::NoFile => (EnteredKind::ContentChange, VisibleState::OneFile(content)),
+                VisibleState::NoFile => (
+                    EnteredKind::ContentChange,
+                    VisibleState::OneFile(content),
+                    SnapshotChangeOfTheOperation::Unchanged,
+                ),
                 VisibleState::OneFile(_) => return Err(OperationNotAccepted::CreatingTheFirstFileWhileAFileExistsIsNotInTheFirstVersion),
             },
             Operation::OverwriteTheFile(content) => match state_so_far {
                 VisibleState::NoFile => return Err(OperationNotAccepted::OverwritingWithoutAFileIsNotInTheFirstVersion),
-                VisibleState::OneFile(_) => (EnteredKind::ContentChange, VisibleState::OneFile(content)),
+                VisibleState::OneFile(_) => (
+                    EnteredKind::ContentChange,
+                    VisibleState::OneFile(content),
+                    SnapshotChangeOfTheOperation::Unchanged,
+                ),
             },
-            Operation::ExplicitPublish => (EnteredKind::ExplicitPublish, state_so_far.clone()),
-            Operation::Fsync => (EnteredKind::Fsync, state_so_far.clone()),
-            Operation::RollBackToVersion(number) => {
-                (EnteredKind::RollBack, self.content_of_version(number)?)
-            }
+            // 截短或补 0（K8）：POSIX 面上洞与写过的 0 分不出来，内容就是整份字节。
+            Operation::Truncate(new_size_in_bytes) => match state_so_far {
+                VisibleState::NoFile => {
+                    return Err(OperationNotAccepted::TruncatingWithoutAFileIsNotInTheFirstVersion)
+                }
+                VisibleState::OneFile(FileContent(bytes)) => {
+                    let mut truncated = bytes.clone();
+                    truncated.resize(
+                        usize::try_from(new_size_in_bytes)
+                            .expect("64 位上 usize 与 u64 同宽：截到的长度装得进"),
+                        0,
+                    );
+                    (
+                        EnteredKind::ContentChange,
+                        VisibleState::OneFile(FileContent(truncated)),
+                        SnapshotChangeOfTheOperation::Unchanged,
+                    )
+                }
+            },
+            Operation::ExplicitPublish => (
+                EnteredKind::ExplicitPublish,
+                state_so_far.clone(),
+                SnapshotChangeOfTheOperation::Unchanged,
+            ),
+            Operation::Fsync => (
+                EnteredKind::Fsync,
+                state_so_far.clone(),
+                SnapshotChangeOfTheOperation::Unchanged,
+            ),
+            Operation::RollBackToVersion(number) => (
+                EnteredKind::RollBack,
+                self.content_of_version(number)?,
+                SnapshotChangeOfTheOperation::Unchanged,
+            ),
+            // 文件集合为空时建快照照收，快照读出「没有文件」（用户 2026-10-16 定「放开」，判决
+            // `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F5）。
+            Operation::CreateSnapshot(name) => (
+                EnteredKind::CreateSnapshot,
+                state_so_far.clone(),
+                SnapshotChangeOfTheOperation::Created {
+                    name,
+                    captured: state_so_far.clone(),
+                },
+            ),
+            Operation::DestroySnapshot(name) => (
+                EnteredKind::DestroySnapshot,
+                state_so_far.clone(),
+                SnapshotChangeOfTheOperation::Destroyed { name },
+            ),
         };
+        self.allowed_snapshot_sets_at_the_last_recovery =
+            allowed_snapshot_sets_at_the_last_recovery;
         self.operations_since_the_last_recovery
             .push(EnteredOperation {
                 kind,
                 outcome: entered_outcome,
                 state_after,
+                snapshot_change,
             });
         Ok(SubmittedOperation::Entered(OperationNumber(
             self.operations_since_the_last_recovery.len(),
@@ -428,30 +752,138 @@ impl ReferenceFileSystem {
                 recovered,
             });
         }
-        self.restart_from_the_recovered_state(crash_falls_in, recovered);
+        // 恢复之后允许活着的快照集合：允许集合里与读出的内容相同的每一个状态那一刻的快照集合（判决
+        // `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F6）。崩溃判定只比文件；建快照与删快照都是发布末尾，
+        // 写了之后报错或崩在里面的那一次落没落盘读出的文件定不了，两种结局都收，不取下标最大的那一个。
+        let indices_whose_state_is_the_recovered_one: Vec<StateIndex> = allowed
+            .allowed
+            .iter()
+            .filter(|allowed_state| allowed_state.state == recovered)
+            .map(|allowed_state| allowed_state.index)
+            .collect();
+        let allowed_snapshot_sets_after_the_recovery =
+            self.distinct_snapshot_sets_at(&indices_whose_state_is_the_recovered_one);
+        self.restart_from_the_recovered_state(
+            crash_falls_in,
+            recovered,
+            allowed_snapshot_sets_after_the_recovery,
+        );
         Ok(allowed)
     }
 
     /// 崩溃恢复抛弃最新根的那一次（恢复时有读故障）：不判读出的内容，以它重新起头（r6 判决第二节 U2 一行：
-    /// 「崩溃恢复抛弃最新根的那一次，RefFS 以读出内容重新起头、不判那一步」）。
+    /// 「崩溃恢复抛弃最新根的那一次，RefFS 以读出内容重新起头、不判那一步」）。重新起头时允许活着的快照集合按调用方交的
+    /// [`AbandonedRoot`] 取（判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1）：允许集合里每一个状态 S_j
+    /// 都可能是最新那一版，各取一份、并起来——被抛弃的是写出这一版的那一条，取之前那一版的（写出它的是删快照这种一个操作写多条根的，
+    /// 前后两份都取）；不是写出这一版的那一条，取 S_j 自己那一刻的。
     ///
     /// # Errors
-    /// hi 越界、或写了之后报错的操作后面到崩溃为止又起了操作（同 [`Self::allowed_set_for_a_crash_in`]），返回时模型不动。
+    /// [`AbandonedRootRestartNotAccepted`] 的两种，返回时模型不动。
     pub fn restart_after_a_recovery_that_abandoned_the_newest_root(
         &mut self,
         crash_falls_in: OperationNumber,
+        abandoned_root: AbandonedRoot,
         recovered: VisibleState,
-    ) -> Result<(), CrashNotAccepted> {
+    ) -> Result<(), AbandonedRootRestartNotAccepted> {
         // 只核 hi 判不判得了；读出的内容在不在允许集合里不判
-        self.allowed_set_for_a_crash_in(crash_falls_in)?;
-        self.restart_from_the_recovered_state(crash_falls_in, recovered);
+        let allowed = self
+            .allowed_set_for_a_crash_in(crash_falls_in)
+            .map_err(AbandonedRootRestartNotAccepted::TheCrashIsNotAccepted)?;
+        let mut indices_whose_snapshot_sets_the_restart_allows: Vec<StateIndex> = Vec::new();
+        for allowed_state in &allowed.allowed {
+            let indices_when_this_state_is_the_newest_version = self
+                .indices_whose_snapshot_sets_a_restart_allows_when_the_newest_version_is(
+                    allowed_state.index,
+                    abandoned_root,
+                )?;
+            indices_whose_snapshot_sets_the_restart_allows
+                .extend(indices_when_this_state_is_the_newest_version);
+        }
+        let allowed_snapshot_sets_after_the_recovery =
+            self.distinct_snapshot_sets_at(&indices_whose_snapshot_sets_the_restart_allows);
+        self.restart_from_the_recovered_state(
+            crash_falls_in,
+            recovered,
+            allowed_snapshot_sets_after_the_recovery,
+        );
         Ok(())
     }
 
+    /// S_j（`newest_state`）是最新那一版、被抛弃的根与它的关系是 `abandoned_root` 时，抛弃最新根的那次恢复之后允许哪几个下标那一刻的快照集合
+    /// （判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1）。
+    ///
+    /// # Errors
+    /// 被抛弃的根写出了 S_0 那一版：[`AbandonedRootRestartNotAccepted::TheRootThatWroteTheStateAtTheLastRecoveryIsNotInTheFirstVersion`]。
+    fn indices_whose_snapshot_sets_a_restart_allows_when_the_newest_version_is(
+        &self,
+        newest_state: StateIndex,
+        abandoned_root: AbandonedRoot,
+    ) -> Result<Vec<StateIndex>, AbandonedRootRestartNotAccepted> {
+        match abandoned_root {
+            // 恢复落到的前一条根仍在这一版上
+            AbandonedRoot::WrittenAfterTheNewestVersion => Ok(vec![newest_state]),
+            AbandonedRoot::WroteTheNewestVersion => {
+                // S_j 由第 j 个操作写出（j 是 lo 或某次发布的末尾）；j = 0 时它是上一次恢复重新起头的那一版，之前那一版 RefFS 不记
+                let Some(position_of_the_operation_that_wrote_it) = newest_state.0.checked_sub(1)
+                else {
+                    return Err(AbandonedRootRestartNotAccepted::TheRootThatWroteTheStateAtTheLastRecoveryIsNotInTheFirstVersion);
+                };
+                let index_of_the_version_before = self
+                    .index_of_the_last_confirmed_publish_before_the_operation_at(
+                        position_of_the_operation_that_wrote_it,
+                    );
+                let operation_that_wrote_it = self
+                    .operations_since_the_last_recovery
+                    .get(position_of_the_operation_that_wrote_it)
+                    .expect("允许集合里 S_0 之后的下标都是某个操作的末尾，不超过序列长度");
+                if operation_that_wrote_it
+                    .kind
+                    .its_earlier_roots_may_carry_its_snapshot_change()
+                {
+                    Ok(vec![index_of_the_version_before, newest_state])
+                } else {
+                    Ok(vec![index_of_the_version_before])
+                }
+            }
+        }
+    }
+
+    /// 第 `position_of_the_operation` + 1 个操作之前那一版的下标：上一次恢复之后、它之前最后一次已确认发布（返回成功的发布调用、fsync、
+    /// 回退、建快照、删快照）的末尾；没有时是 0（上一次恢复读出、重新起头的那一版，还没崩溃过时是起点）。它之前的操作都返回过，
+    /// 写了之后报错的只能是 hi 之前的最后一个（[`Self::allowed_set_for_a_crash_in`] 拒掉别的），所以它之前的发布末尾都已确认。
+    fn index_of_the_last_confirmed_publish_before_the_operation_at(
+        &self,
+        position_of_the_operation: usize,
+    ) -> StateIndex {
+        self.operations_since_the_last_recovery[..position_of_the_operation]
+            .iter()
+            .enumerate()
+            .rev()
+            .find(|(_, operation)| match operation.outcome {
+                EnteredOutcome::Succeeded => operation.kind.ends_a_publish(),
+                EnteredOutcome::FailedAfterWriting => false,
+            })
+            .map_or(StateIndex(0), |(earlier_position, _)| {
+                StateIndex(earlier_position + 1)
+            })
+    }
+
+    /// 这几个下标那一刻允许活着的快照集合，并起来、去重。
+    fn distinct_snapshot_sets_at(&self, indices: &[StateIndex]) -> Vec<Snapshots> {
+        let mut distinct_snapshot_sets = Vec::new();
+        for index in indices {
+            for snapshots in self.snapshot_set_of_each_allowed_start_at(*index) {
+                push_distinct_snapshot_set(&mut distinct_snapshot_sets, snapshots);
+            }
+        }
+        distinct_snapshot_sets
+    }
+
     fn restart_from_the_recovered_state(
         &mut self,
         crash_falls_in: OperationNumber,
         recovered: VisibleState,
+        allowed_snapshot_sets_after_the_recovery: Vec<Snapshots>,
     ) {
         // 前 hi − 1 个返回过；第 hi 个没返回，它是发布也不算已确认；第 hi 个之后的没发生。
         let returned_before_the_crash =
@@ -465,6 +897,7 @@ impl ReferenceFileSystem {
             .push(recovered.clone());
         // 每次恢复之后以读出的内容为起点接着走（r5 判决第二节 U2 ②）
         self.state_at_the_last_recovery = recovered;
+        self.allowed_snapshot_sets_at_the_last_recovery = allowed_snapshot_sets_after_the_recovery;
         self.operations_since_the_last_recovery.clear();
     }
 }
@@ -508,6 +941,236 @@ mod tests {
             .collect()
     }
 
+    /// 建快照捕获那一刻的文件，之后覆盖写不动它；读快照读出建它那一刻的内容；删掉之后读不到，活头的文件不变
+    /// （判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S7）。
+    #[test]
+    fn a_snapshot_reads_the_file_at_its_creation_until_it_is_destroyed() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(65_549)));
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        assert_eq!(
+            model.allowed_reads_of_the_snapshot(SnapshotName(65_549)),
+            vec![Some(file_with("A"))],
+            "快照读出建它那一刻的 A"
+        );
+        succeed(&mut model, Operation::DestroySnapshot(SnapshotName(65_549)));
+        assert_eq!(
+            model.allowed_reads_of_the_snapshot(SnapshotName(65_549)),
+            vec![None],
+            "删掉之后读不到"
+        );
+        assert_eq!(
+            model.allowed_snapshot_sets(),
+            vec![Snapshots::new()],
+            "快照集合空了"
+        );
+        assert_eq!(
+            model.state_after_every_operation_so_far(),
+            &file_with("B"),
+            "删快照不动活头的文件"
+        );
+    }
+
+    /// 建快照、删快照各是发布末尾（返回成功时抬起下界）：崩在建快照之后，允许集合的下界就是它；恢复读出那一刻的文件，快照接着活着。
+    #[test]
+    fn a_crash_after_a_snapshot_returned_keeps_the_snapshot_and_starts_from_it() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        let creation = succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        let crash = model.crash_after_every_operation_returned();
+        let allowed = model
+            .crash_and_recover(crash, file_with("A"))
+            .expect("A 在允许集合里");
+        assert_eq!(
+            allowed.lower_bound,
+            creation.state_after(),
+            "建快照返回成功：下界是它的末尾"
+        );
+        assert_eq!(
+            model.allowed_reads_of_the_snapshot(SnapshotName(7)),
+            vec![Some(file_with("A"))],
+            "恢复之后快照接着活着"
+        );
+    }
+
+    /// 文件集合为空时建快照照收（用户 2026-10-16 定「放开」，判决 `research/prompts/m4-segment-code-r1-main-verification.md`
+    /// 第三节 F5）：空池上建快照，之后写第一个文件，读快照读出「没有文件」，活头读出写的那份。
+    #[test]
+    fn a_snapshot_created_while_there_is_no_file_reads_no_file_after_the_first_file_is_written() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(3)));
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        assert_eq!(
+            model.allowed_reads_of_the_snapshot(SnapshotName(3)),
+            vec![Some(VisibleState::NoFile)],
+            "快照读出建它那一刻的「没有文件」"
+        );
+        assert_eq!(
+            model.state_after_every_operation_so_far(),
+            &file_with("A"),
+            "活头读出写的 A"
+        );
+    }
+
+    /// 建快照写了之后报错（做没做成两可），之后崩溃、恢复读出 A：建快照之前与之后两个发布末尾都是 A，读出的文件定不了那次发布落没落盘，
+    /// 快照有与没有两种结局都收（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F6）——之后删这个快照照收（落了盘），
+    /// 回退也照收（没落盘：第一版有快照就拒回退，只有快照集合可以是空的才收）。
+    #[test]
+    fn a_crash_after_a_snapshot_creation_that_failed_after_writing_allows_the_snapshot_to_be_there_or_not(
+    ) {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::Fsync);
+        enter(
+            &mut model,
+            Operation::CreateSnapshot(SnapshotName(7)),
+            OperationOutcome::FailedAfterWriting,
+        );
+        let crash = model.crash_after_every_operation_returned();
+        model
+            .crash_and_recover(crash, file_with("A"))
+            .expect("A 在允许集合里");
+        assert_eq!(
+            model.allowed_snapshot_sets(),
+            vec![
+                Snapshots::new(),
+                Snapshots::from([(SnapshotName(7), file_with("A"))])
+            ],
+            "快照没有与有两种快照集合都允许"
+        );
+        let mut the_creation_landed = model.clone();
+        succeed(
+            &mut the_creation_landed,
+            Operation::DestroySnapshot(SnapshotName(7)),
+        );
+        let mut the_creation_did_not_land = model.clone();
+        let rollback_target = the_creation_did_not_land.newest_version();
+        succeed(
+            &mut the_creation_did_not_land,
+            Operation::RollBackToVersion(rollback_target),
+        );
+        assert_eq!(
+            the_creation_did_not_land.allowed_snapshot_sets(),
+            vec![Snapshots::new()],
+            "回退做成说明快照没落盘：只剩空的那个快照集合"
+        );
+    }
+
+    /// 删快照写了之后报错（意图落了、批没落完，两可），之后崩溃、恢复读出 A：快照还在与已删两种结局都收（判决第三节 F6）——
+    /// 读快照读出建它那一刻的 A 照收、读出别的内容判红且模型不动；读出 A 之后只剩快照还在的那个集合，回退照拒；
+    /// 没读快照时回退照收（快照已删）。
+    #[test]
+    fn a_crash_after_a_snapshot_destruction_that_failed_after_writing_allows_the_snapshot_to_be_there_or_not(
+    ) {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
+        enter(
+            &mut model,
+            Operation::DestroySnapshot(SnapshotName(7)),
+            OperationOutcome::FailedAfterWriting,
+        );
+        let crash = model.crash_after_every_operation_returned();
+        model
+            .crash_and_recover(crash, file_with("A"))
+            .expect("A 在允许集合里");
+        assert_eq!(
+            model.allowed_snapshot_sets(),
+            vec![
+                Snapshots::from([(SnapshotName(7), file_with("A"))]),
+                Snapshots::new()
+            ],
+            "快照还在与已删两种快照集合都允许"
+        );
+        let mut the_destruction_did_not_land = model.clone();
+        let before_the_wrong_read = the_destruction_did_not_land.clone();
+        assert_eq!(
+            the_destruction_did_not_land
+                .judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
+                    SnapshotName(7),
+                    &file_with("B")
+                ),
+            Err(SnapshotReadOutsideTheAllowedReads {
+                name: SnapshotName(7),
+                read_out: file_with("B"),
+                allowed_reads: vec![Some(file_with("A")), None],
+            }),
+            "读出建它那一刻之外的内容判红"
+        );
+        assert_eq!(
+            the_destruction_did_not_land, before_the_wrong_read,
+            "判红的读不动模型"
+        );
+        the_destruction_did_not_land
+            .judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
+                SnapshotName(7),
+                &file_with("A"),
+            )
+            .expect("快照还在：读出建它那一刻的 A");
+        assert_eq!(
+            the_destruction_did_not_land.allowed_snapshot_sets(),
+            vec![Snapshots::from([(SnapshotName(7), file_with("A"))])],
+            "读出 A 之后只剩快照还在的那个集合"
+        );
+        let rollback_target_while_the_snapshot_is_kept =
+            the_destruction_did_not_land.newest_version();
+        assert_eq!(
+            the_destruction_did_not_land.submit(
+                Operation::RollBackToVersion(rollback_target_while_the_snapshot_is_kept),
+                OperationOutcome::Succeeded
+            ),
+            Err(OperationNotAccepted::RollBackWithSnapshotsIsNotInTheFirstVersion),
+            "快照还在：回退照拒"
+        );
+        let mut the_destruction_landed = model.clone();
+        let rollback_target_after_the_destruction_landed = the_destruction_landed.newest_version();
+        succeed(
+            &mut the_destruction_landed,
+            Operation::RollBackToVersion(rollback_target_after_the_destruction_landed),
+        );
+    }
+
+    /// 不收的三种：快照名已经活着、删一个不活着的快照、有活快照时回退；返回时模型不动。
+    #[test]
+    fn snapshot_operations_the_first_version_does_not_take_are_not_accepted_and_leave_the_model_unchanged(
+    ) {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(1)));
+        let before = model.clone();
+        assert_eq!(
+            model.submit(
+                Operation::CreateSnapshot(SnapshotName(1)),
+                OperationOutcome::Succeeded
+            ),
+            Err(OperationNotAccepted::SnapshotNameAlreadyTaken {
+                name: SnapshotName(1)
+            }),
+            "快照名已经活着"
+        );
+        assert_eq!(
+            model.submit(
+                Operation::DestroySnapshot(SnapshotName(2)),
+                OperationOutcome::Succeeded
+            ),
+            Err(OperationNotAccepted::NoSnapshotWithThisName {
+                name: SnapshotName(2)
+            }),
+            "删一个不活着的快照"
+        );
+        assert_eq!(
+            model.submit(
+                Operation::RollBackToVersion(VersionNumber(1)),
+                OperationOutcome::Succeeded
+            ),
+            Err(OperationNotAccepted::RollBackWithSnapshotsIsNotInTheFirstVersion),
+            "有活快照时回退"
+        );
+        assert_eq!(model, before, "不收的操作不动模型");
+    }
+
     #[test]
     fn an_overwrite_refused_before_any_write_did_not_happen_so_the_publish_after_it_does_not_allow_its_content(
     ) {
@@ -1063,8 +1726,13 @@ mod tests {
         succeed(&mut model, Operation::OverwriteTheFile(content("B")));
         succeed(&mut model, Operation::Fsync);
         let crash = model.crash_after_every_operation_returned();
+        // 被抛弃的是 fsync 写出最新那一版的那一条根
         model
-            .restart_after_a_recovery_that_abandoned_the_newest_root(crash, file_with("A"))
+            .restart_after_a_recovery_that_abandoned_the_newest_root(
+                crash,
+                AbandonedRoot::WroteTheNewestVersion,
+                file_with("A"),
+            )
             .expect("抛弃最新根的那一次恢复不判：fsync 之后读出 A 也照收");
         let next_crash = model.crash_after_every_operation_returned();
         let refused = model.crash_and_recover(next_crash, file_with("B"));
@@ -1076,4 +1744,279 @@ mod tests {
             "以读出的 A 重新起头，之后读回 B 判红：{refused:?}"
         );
     }
+
+    /// 攻方 R6 那一形（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H5，读法照 U2）：写 A、发布、建快照 7、
+    /// 覆盖写 B、发布，抛弃最新根（B 那次发布）的那次恢复读出 A。读出的 A 不在允许集合里也不判、照收；重新起头时的快照集合取被抛弃那一版
+    /// 之前那一版（建快照那一版）的，快照 7 照在，读它读出 A 不判红。
+    #[test]
+    fn a_recovery_that_abandoned_the_newest_root_keeps_a_snapshot_created_in_the_published_state_before_it(
+    ) {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        let crash = model.crash_after_every_operation_returned();
+        let allowed = model
+            .allowed_set_for_a_crash_in(crash)
+            .expect("崩溃落在崩溃本身里");
+        assert!(
+            !allowed.contains(&file_with("A")),
+            "读出的 A 不在允许集合里（只有 B 那次发布）：{allowed:?}"
+        );
+        // 被抛弃的是 B 那次发布写出最新那一版的那一条根
+        model
+            .restart_after_a_recovery_that_abandoned_the_newest_root(
+                crash,
+                AbandonedRoot::WroteTheNewestVersion,
+                file_with("A"),
+            )
+            .expect("抛弃最新根的那一次恢复不判读出的内容");
+        assert_eq!(
+            model.allowed_snapshot_sets(),
+            vec![Snapshots::from([(SnapshotName(7), file_with("A"))])],
+            "快照集合取被抛弃那一版之前那一版（建快照那一版）的：快照 7 照在"
+        );
+        model
+            .judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
+                SnapshotName(7),
+                &file_with("A"),
+            )
+            .expect("读快照 7 读出建它那一刻的 A 不判红");
+        assert_eq!(
+            model.state_after_every_operation_so_far(),
+            &file_with("A"),
+            "以读出的 A 重新起头"
+        );
+    }
+
+    /// 被抛弃的最新根就是建快照 7 的那一次发布：之前那一版（第一个文件那次发布）没有快照，重新起头之后快照 7 不在——
+    /// 读它读出 A 判红，同名再建照收（判决第三节 H5；`research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1 下
+    /// 调用方交「写出最新那一版的那一条」，取的是之前那一版的，不是这一版自己的）。
+    #[test]
+    fn a_recovery_that_abandoned_the_newest_root_written_by_a_snapshot_creation_restarts_without_that_snapshot(
+    ) {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
+        let crash = model.crash_after_every_operation_returned();
+        model
+            .restart_after_a_recovery_that_abandoned_the_newest_root(
+                crash,
+                AbandonedRoot::WroteTheNewestVersion,
+                file_with("A"),
+            )
+            .expect("抛弃最新根的那一次恢复不判读出的内容");
+        assert_eq!(
+            model.allowed_snapshot_sets(),
+            vec![Snapshots::new()],
+            "快照集合取建快照之前那一版的：没有快照"
+        );
+        let before_the_read = model.clone();
+        assert_eq!(
+            model.judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
+                SnapshotName(7),
+                &file_with("A")
+            ),
+            Err(SnapshotReadOutsideTheAllowedReads {
+                name: SnapshotName(7),
+                read_out: file_with("A"),
+                allowed_reads: vec![None],
+            }),
+            "快照 7 随被抛弃的那一版一起没了：读出 A 判红"
+        );
+        assert_eq!(model, before_the_read, "判红的读不动模型");
+        assert_eq!(
+            model.submit(
+                Operation::CreateSnapshot(SnapshotName(7)),
+                OperationOutcome::Succeeded
+            ),
+            Ok(SubmittedOperation::Entered(OperationNumber(1))),
+            "快照名 7 空着：同名再建照收"
+        );
+    }
+
+    /// 攻方第三轮 Q2 H5 那一格的第一格（`research/prompts/m4-segment-code-r3-opus-output.md`，判决
+    /// `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1）：写 A、发布、建快照 7，实现随后可写挂载一次，写行与暖机的根
+    /// 都映到建快照那一版；抛弃最新根（暖机那一条）的那次恢复落到它前一条根，仍在建快照那一版上，快照 7 在、读出 A。
+    /// 调用方交「不是写出这一版的那一条」，快照集合取这一版自己那一刻的：快照 7 照在，读它读出 A 不判红。
+    #[test]
+    fn a_recovery_that_abandoned_a_root_written_after_a_snapshot_creation_keeps_that_snapshot() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
+        let crash = model.crash_after_every_operation_returned();
+        model
+            .restart_after_a_recovery_that_abandoned_the_newest_root(
+                crash,
+                AbandonedRoot::WrittenAfterTheNewestVersion,
+                file_with("A"),
+            )
+            .expect("抛弃最新根的那一次恢复不判读出的内容");
+        assert_eq!(
+            model.allowed_snapshot_sets(),
+            vec![Snapshots::from([(SnapshotName(7), file_with("A"))])],
+            "被抛弃的不是写出建快照那一版的根：快照集合取这一版自己的，快照 7 照在"
+        );
+        model
+            .judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
+                SnapshotName(7),
+                &file_with("A"),
+            )
+            .expect("读快照 7 读出建它那一刻的 A 不判红");
+    }
+
+    /// 攻方第三轮 Q2 H5 那一格的第二格（判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1「删快照这种一个操作写多条根的
+    /// 取前后两份」）：写 A、发布、建快照 7、删快照 7（实现一次调用写意图与全部批），抛弃最后一批的那次恢复落到意图那条根，快照 7 带
+    /// 销毁中记号、读照读，下一次用户改动先走空把它删完。调用方交「写出这一版的那一条」，快照集合前后两份都收：读快照 7 读出 A 照收、
+    /// 只留快照在的那一份；走空之后同名再建也照收。
+    #[test]
+    fn a_recovery_that_abandoned_the_last_batch_of_a_snapshot_destruction_allows_the_snapshot_there_or_gone(
+    ) {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
+        succeed(&mut model, Operation::DestroySnapshot(SnapshotName(7)));
+        let crash = model.crash_after_every_operation_returned();
+        model
+            .restart_after_a_recovery_that_abandoned_the_newest_root(
+                crash,
+                AbandonedRoot::WroteTheNewestVersion,
+                file_with("A"),
+            )
+            .expect("抛弃最新根的那一次恢复不判读出的内容");
+        assert_eq!(
+            model.allowed_snapshot_sets(),
+            vec![
+                Snapshots::from([(SnapshotName(7), file_with("A"))]),
+                Snapshots::new(),
+            ],
+            "删快照之前那一版（快照 7 在）与删快照那一版（快照 7 不在）两份都收"
+        );
+        let mut read_before_the_walk = model.clone();
+        read_before_the_walk
+            .judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
+                SnapshotName(7),
+                &file_with("A"),
+            )
+            .expect("落到意图那条根：快照 7 带销毁中记号、读出 A 照收");
+        assert_eq!(
+            read_before_the_walk.allowed_snapshot_sets(),
+            vec![Snapshots::from([(SnapshotName(7), file_with("A"))])],
+            "读出 A 之后只留快照 7 在的那一份"
+        );
+        assert_eq!(
+            model.submit(
+                Operation::CreateSnapshot(SnapshotName(7)),
+                OperationOutcome::Succeeded
+            ),
+            Ok(SubmittedOperation::Entered(OperationNumber(1))),
+            "走空把快照 7 删完之后同名再建照收"
+        );
+    }
+
+    /// 调用方说被抛弃的根写出了最新那一版，而最新那一版是上一次恢复读出、重新起头的那一版（之后没有已确认的发布）：它之前那一版在
+    /// 上一次恢复之前，RefFS 不记那时的快照集合，判决第三节 J1 没写取什么——返回第一版不支持，模型不动。
+    #[test]
+    fn a_recovery_that_abandoned_the_root_that_wrote_the_state_at_the_last_recovery_is_not_in_the_first_version_and_the_model_does_not_move(
+    ) {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        model
+            .crash_and_recover(model.crash_after_every_operation_returned(), file_with("A"))
+            .expect("发布之后读出 A 在允许集合里");
+        let before_the_restart = model.clone();
+        assert_eq!(
+            model.restart_after_a_recovery_that_abandoned_the_newest_root(
+                model.crash_after_every_operation_returned(),
+                AbandonedRoot::WroteTheNewestVersion,
+                file_with("A"),
+            ),
+            Err(AbandonedRootRestartNotAccepted::TheRootThatWroteTheStateAtTheLastRecoveryIsNotInTheFirstVersion),
+            "允许集合只有 S_0：之前那一版在上一次恢复之前，第一版不支持"
+        );
+        assert_eq!(model, before_the_restart, "不收的那一次不动模型");
+    }
+
+    /// 截短（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K8）：截到 3 字节留下前 3 个。
+    #[test]
+    fn truncating_to_a_shorter_size_keeps_the_leading_bytes() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("abcdef")));
+        succeed(&mut model, Operation::Truncate(3));
+        assert_eq!(
+            model.state_after_every_operation_so_far(),
+            &file_with("abc")
+        );
+    }
+
+    /// 补 0（判决第三节 K8）：截到比现在长，多出来的读 0。
+    #[test]
+    fn truncating_to_a_longer_size_pads_with_zeros() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("ab")));
+        succeed(&mut model, Operation::Truncate(5));
+        assert_eq!(
+            model.state_after_every_operation_so_far(),
+            &VisibleState::OneFile(FileContent(vec![b'a', b'b', 0, 0, 0]))
+        );
+    }
+
+    /// 截断之后读快照（判决第三节 K4、K8）：快照读出截之前的内容，活头是截之后的。
+    #[test]
+    fn a_snapshot_created_before_a_truncation_reads_the_content_before_it() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("abcdef")));
+        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(65_549)));
+        succeed(&mut model, Operation::Truncate(2));
+        succeed(&mut model, Operation::ExplicitPublish);
+        assert_eq!(
+            model.allowed_reads_of_the_snapshot(SnapshotName(65_549)),
+            vec![Some(file_with("abcdef"))],
+            "快照读出截之前的内容"
+        );
+        assert_eq!(model.state_after_every_operation_so_far(), &file_with("ab"));
+    }
+
+    /// 崩在截断里（判决第三节 K2「先落目标」：意图那次 size 已是 N，崩在各批之间读出的就是截后）：截断交成内容改动加它那次发布，崩溃落在那次发布里，
+    /// 允许集合恰好两样——截前与截后；截到一半（某一批的边界）的内容不在里面。
+    #[test]
+    fn a_crash_inside_a_truncation_allows_exactly_the_content_before_and_after_it() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("abcdef")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        succeed(&mut model, Operation::Truncate(2));
+        let publish_of_the_truncation = succeed(&mut model, Operation::ExplicitPublish);
+        let allowed = model
+            .allowed_set_for_a_crash_in(publish_of_the_truncation)
+            .expect("崩溃落在截断那次发布里");
+        let allowed_states: Vec<VisibleState> = allowed
+            .allowed
+            .iter()
+            .map(|allowed_state| allowed_state.state.clone())
+            .collect();
+        assert_eq!(
+            allowed_states,
+            vec![file_with("abcdef"), file_with("ab")],
+            "截前、截后两样：{allowed:?}"
+        );
+        assert!(!allowed.contains(&file_with("abcd")), "截到一半的不在里面");
+    }
+
+    /// 还没有文件时截断（判决第三节 K1：实现在任何写之前拒）：不收、模型不动。
+    #[test]
+    fn truncating_before_any_file_exists_is_refused_and_the_model_does_not_move() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        let before = model.clone();
+        assert_eq!(
+            model.submit(Operation::Truncate(0), OperationOutcome::Succeeded),
+            Err(OperationNotAccepted::TruncatingWithoutAFileIsNotInTheFirstVersion)
+        );
+        assert_eq!(model, before, "拒时模型不动");
+    }
 }
```

## code-round-publish-lookup-covered.diff（sha256 4a4e457375c38fe6…）

```diff
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index 465772b..304ae77 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -17,6 +17,7 @@
 //! 缺根那一半是 C585（会话发布放进发布途中取下的旧快照） 用户定的改法 C）。
 //! 发布路径与抬 F 各自开写入口的做法不变。
 
+use std::cell::{Cell, RefCell};
 use std::collections::{BTreeMap, BTreeSet};
 
 use singlefs_format::{CLUSTER_SEGMENT_SLOTS, SLOT_BYTES};
@@ -56,7 +57,7 @@ use crate::mount::{
     OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn, ParametersAndDeviceTableOfTheMount,
     RaisedFloor, ShadowLedger,
 };
-use crate::recovery::{instance_table_chain_of_root, PoolReader};
+use crate::recovery::{instance_table_chain_of_root, PoolReader, ReadPassOverThePool};
 use crate::region_scan::{scan_region, AllocationInRegion, CentralMappingEntry, RegionScan};
 use crate::root_record::{RootRecord, UnmountMarker};
 use crate::root_ring::{target_for_publish, RootRingSlot};
@@ -68,9 +69,9 @@ use crate::transaction::{
     publish_unlink_intent, publish_version, walk_the_deletion_queue_until_it_is_empty,
     CentralMappingOfAVersionReadLeafByLeaf, CompactionIntentStep, CompactionStepRefused,
     DeletionAndLineageOfThePublish, DeletionQueueWalk, FirstFile, MappingLookup,
-    PlannedUnitsRewrittenAnyway, PoolVersion, PoolWriter, PublishError,
-    RegionCheckOfTheIntentDeletion, ReleaseChecksumCheck, RelocationBatchRefused,
-    TransactionOutput, TransactionUnit,
+    PlannedUnitsRewrittenAnyway, PoolVersion, PoolWriter, PublishError, ReadFailedOnTheDisk,
+    ReadFailingSelfVerification, RegionCheckOfTheIntentDeletion, ReleaseChecksumCheck,
+    RelocationBatchRefused, TransactionOutput, TransactionUnit,
 };
 use crate::unit::{unit_filesystem_identifier, UNIT_CLASS_DATA};
 use crate::unlink::DeletionSwitches;
@@ -1664,7 +1665,10 @@ pub fn refusal_is_short_of_space(refusal: &PublishError) -> bool {
         | PublishError::DeadlistEntryNotReleasable { .. }
         // 改 size 的对象不是活记录、目标 size 溢出、条款没写的那一格、尾单元读不出：推抬 F 改变不了。
         | PublishError::TruncationRefused(_)
-        | PublishError::BlockDevice(_) => false,
+        | PublishError::BlockDevice(_)
+        // 取号之前预演读错、读坏那两拒只在可写挂载取号之前那一判里造，发布路径不报它们。
+        | PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(_)
+        | PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(_) => false,
     }
 }
 
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index d32282e..c40b127 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -113,10 +113,11 @@ use crate::records::{
 use crate::records::{
     build_extent_record, build_inode_internal_entry, build_mapping_entry, data_key_tail,
     mapping_key_for_data, mapping_key_for_node, node_key_tail, parse_mapping_entry,
-    AccountingEntry, InodeRecord, LineageRecord, RetirementRecord, TombstoneRecord, TreeTableEntry,
-    ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF, NLINK_OF_AN_INODE_BEING_DELETED,
-    NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES, SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION,
-    STATISTIC_ALLOCATED_BYTES, STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_DEFER_QUEUE_BYTES,
+    AccountingEntry, InodeRecord, InodeRecordTruncationField, LineageRecord, RetirementRecord,
+    TombstoneRecord, TreeTableEntry, ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
+    NLINK_OF_AN_INODE_BEING_DELETED, NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
+    SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION, STATISTIC_ALLOCATED_BYTES,
+    STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_DEFER_QUEUE_BYTES,
     STATISTIC_EMPTY_CLUSTER_SEGMENTS, STATISTIC_FRAGMENTATION_RUNS, STATISTIC_FREE_BYTES,
     STATISTIC_INODE_WATERMARK, STATISTIC_NO_DEVICE_DIMENSION, STATISTIC_PENDING_DELETE_BYTES,
     STATISTIC_UNRECLAIMABLE_BYTES, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST,
@@ -130,7 +131,8 @@ use crate::recovery::{
     snapshot_tree_table_entries_paired_one_to_one,
     system_configuration_slots_of_one_device_read_once,
     system_configuration_witness_is_undecidable, tree_table_entry_count,
-    verified_system_configuration_slots, PoolReader, RecoveryFailure, SnapshotTreeTableEntries,
+    verified_system_configuration_slots, PoolReader, PoolReaderOfOneReadPass,
+    PoolReaderStraightFromTheDisk, RecoveryFailure, SnapshotTreeTableEntries,
     SystemConfigurationSlotsOfOneDevice,
 };
 use crate::retirement_tree::{
@@ -140,9 +142,10 @@ use crate::retirement_tree::{
 use crate::root_record::{RootRecord, UnmountMarker};
 use crate::root_ring::{slot_offset, target_for_publish};
 use crate::system_configuration::{
-    journal_in_flight_record_limit, SystemConfiguration, SystemImmutableConfiguration,
-    SystemMutableConfiguration, SystemRuntimeConfiguration, SystemRuntimeQuantities,
-    JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION, ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
+    journal_in_flight_record_limit, InodeRecordTruncationFieldsOfThePool, SystemConfiguration,
+    SystemImmutableConfiguration, SystemMutableConfiguration, SystemRuntimeConfiguration,
+    SystemRuntimeQuantities, JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
+    ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
 };
 use crate::tombstone_tree::{
     append_tombstone_to_the_open_container, TombstoneContainer, TombstoneContainerIndexInTree,
@@ -151,8 +154,8 @@ use crate::tombstone_tree::{
 use crate::truncate::{
     next_truncation_step, plan_the_extent_tree_after_a_size_change,
     tail_data_unit_rewrite_of_a_truncation, tail_data_unit_rewrite_of_an_extension,
-    target_size_rounded_up_to_a_whole_data_unit_fits, SizeChangeOfARequest, SizeChangeOfTheFile,
-    TailDataUnitRewrite, TruncationStep,
+    target_size_rounded_up_to_a_whole_data_unit_fits, truncation_field_after_the_truncation_intent,
+    SizeChangeOfARequest, SizeChangeOfTheFile, TailDataUnitRewrite, TruncationStep,
 };
 use crate::unit::{
     build_data_unit, build_index_node, build_packed_unit, data_unit_payload,
@@ -620,6 +623,9 @@ impl<Device: BlockDevice> PoolWriter<'_, Device> {
                 sizes: self.parameters.geometry,
                 journal_ring_start_slot: JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
                 root_ring_base_slot: ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
+                // 写者每次写系统配置都带 incompat 位 2（mkfs 起就置上）：不带位 2 的旧池可写挂载取号那一写起就带上它，之后才发得出截断意图。
+                inode_record_truncation_fields:
+                    InodeRecordTruncationFieldsOfThePool::WRITTEN_BY_THIS_WRITER,
             },
             mutable: SystemMutableConfiguration,
             runtime: SystemRuntimeConfiguration,
@@ -7584,7 +7590,8 @@ impl PublishPlan<'_> {
                 .container_tree_capacities
                 .retirement_container,
             deletion_and_lineage.container_tree_capacities.node_entries,
-        );
+        )
+        .map_err(PublishError::NextContainerNumberPastTheTopOfItsRange)?;
         // logged_ops（判决 G1）：整理的发布签发一条或删一条，别的照抄；装不下一个根走不到（全池活整理意图至多一条）。
         let logged_operations_before: &[LoggedOperationEntry] = match previous {
             Some(version) => &version.logged_operations,
@@ -7625,7 +7632,8 @@ impl PublishPlan<'_> {
     ///
     /// # Errors
     /// inode 记录的落法要的条款仓里还没定 ⇒ `InodeTreeWriteRefused`（`crate::inode_tree` 的三格）；
-    /// 记账树或中央映射树这次之后的形状算不出来 ⇒ `MultiLevelCodeTwoTreeRefused`。
+    /// 记账树或中央映射树这次之后的形状算不出来 ⇒ `MultiLevelCodeTwoTreeRefused`；
+    /// 墓碑树、谱系树或退役树要开新容器而最右那片的号已到顶 ⇒ `NextContainerNumberPastTheTopOfItsRange`（C841）。
     #[allow(
         clippy::too_many_arguments,
         reason = "参数各是一样东西：计划、上一版、树号、记账行的盘、节点容量、分配记录树、删除这一步、这次搬走的映射 key、释放判定那一臂"
@@ -8399,6 +8407,9 @@ fn inode_record_of_file_version(file: &FileVersionPlan<'_>) -> InodeRecord {
         size: u64::try_from(file.content.len()).expect("文件长度"),
         change_count: file.change_count,
         write_time_seconds: file.write_time_seconds,
+        // 整份写（顺序写、覆盖写）清零（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R2、第六节用户定案）：
+        // 这一版从偏移 0 重写了全部单元，没有比它早的单元要靠截断字段判死。
+        truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
     }
 }
 
@@ -8666,6 +8677,8 @@ pub fn publish_new_inodes<Device: BlockDevice>(
             size: 0,
             change_count,
             write_time_seconds,
+            // 没被截断过：截断字段是 0。
+            truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
         })
         .collect();
     publish_version(
@@ -9698,6 +9711,8 @@ pub fn publish_unlink_intent<Device: BlockDevice>(
                 size: record.size,
                 change_count: change_count_of_the_intent,
                 write_time_seconds,
+                // unlink 的意图照抄截断字段（判决 m4-trunc-r2 R2）。
+                truncation_field: record.truncation_field,
             },
             tombstone: TombstoneRecord::of_the_whole_object(inode, record.object_birth, txg),
         },
@@ -9795,26 +9810,23 @@ fn next_unlink_step_after(
                 // （用户 2026-10-03 定 Q-F 乙「各批照抄」）：批不带时间参数，挂载续做里的几批没有调用方给时间。
                 change_count: change_count_of_the_batch,
                 write_time_seconds: record.write_time_seconds,
+                // unlink 的各批照抄截断字段（判决 m4-trunc-r2 R2）。
+                truncation_field: record.truncation_field,
             },
             data_units_kept,
         },
         UnlinkStep::FinalBatch { .. } => DeletionStepOfThePublish::FinalBatch { inode },
     };
-    let deletion_and_lineage = DeletionAndLineageOfThePublish {
-        step,
-        container_tree_capacities: pool.container_tree_capacities(),
-        lineage: LineageStepOfThePublish::NoLineageRecord,
-        snapshot: SnapshotStepOfThePublish::NoNewSnapshot,
-        logged_operations: LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
-    };
-    publish_deletion_step(
-        pool,
-        allocator,
-        previous,
+    Ok((
         txg,
-        &deletion_and_lineage,
-        instance,
-    )
+        DeletionAndLineageOfThePublish {
+            step,
+            container_tree_capacities,
+            lineage: LineageStepOfThePublish::NoLineageRecord,
+            snapshot: SnapshotStepOfThePublish::NoNewSnapshot,
+            logged_operations: LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
+        },
+    ))
 }
 
 /// 改一个 inode 的 size（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K2、K3、K5、K11、K12）：会话里 `UserChange::Truncate`
@@ -9880,6 +9892,19 @@ pub fn publish_size_change<Device: BlockDevice>(
     let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
     // 改动计数取成局部量，理由同 [`publish_unlink_intent`] 那一处。
     let change_count_of_the_size_change = txg.0;
+    // 截断字段（判决 m4-trunc-r2 R2 与第六节用户定案）：截断意图那次按这次放不放单元判写不写（`truncate` 里的纯函数），
+    // 变长与 N = S 照抄上一版。
+    let truncation_field = match size_change {
+        SizeChangeOfARequest::Truncation => truncation_field_after_the_truncation_intent(
+            record.truncation_field,
+            data_units_existing,
+            target_size_in_bytes,
+            txg,
+        ),
+        SizeChangeOfARequest::SizeUnchanged | SizeChangeOfARequest::Extension => {
+            record.truncation_field
+        }
+    };
     let record_after_the_change = InodeRecord {
         inode: record.inode,
         object_birth: record.object_birth,
@@ -9887,6 +9912,7 @@ pub fn publish_size_change<Device: BlockDevice>(
         size: target_size_in_bytes,
         change_count: change_count_of_the_size_change,
         write_time_seconds,
+        truncation_field,
     };
     let (step, logged_operations) = match size_change {
         SizeChangeOfARequest::SizeUnchanged => (
@@ -10004,12 +10030,9 @@ pub fn publish_truncation_step<Device: BlockDevice>(
         allocator,
         previous,
         txg,
-        DeletionAndLineageOfThePublish {
-            step,
-            container_tree_capacities,
-            lineage: LineageStepOfThePublish::NoLineageRecord,
-        },
-    ))
+        &deletion_and_lineage,
+        instance,
+    )
 }
 
 /// 把删除队列从 `current` 走到空的每一步都只算不写（可写挂载在取号之前预演整个挂载续做，岔路单
@@ -14738,6 +14761,7 @@ mod tests {
                 size: 0,
                 change_count,
                 write_time_seconds: 1_788_000_000,
+                truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
             });
         }
         let inode_tree = write_records_into_leaf_containers(
```

