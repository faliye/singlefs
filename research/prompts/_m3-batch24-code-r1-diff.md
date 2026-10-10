# 删除批第 24 批合入的改动原样（2026-10-06）

## 实现员 k：crates.patch（/tmp/claude-1000/m3-batch24/k/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 3e1dbff..c6b3edb 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -2066,7 +2066,8 @@ impl<'reader> Walk<'reader> {
 
     /// 墓碑树或谱系树的码 2 根（或根长高之后的内部节点）往下走：条目 120 字节（分隔 key 8 + 身份引用 26 + 子指针 86），
     /// 类型段 = `record_type` 的走到码 3 容器（判容器头、出生树、容器格式，单元记进这一版走到的落点），类型段 0 的走进下一层码 2 节点
-    /// （层级 = 父层级 − 1），别的类型段判 `identity_invariant` 红、不往下走。每条条目的身份按 `identity_invariant`（墓碑树 I-9.23、
+    /// （层级 = 父层级 − 1），别的类型段判 `identity_invariant` 红、不往下走；孩子头里自证过的类标签与类型段说的那一类不符的同样判红、
+    /// 不往下走（[`Self::judge_a_container_tree_entry_whose_child_is_of_another_class`]，孩子读多长按它头里的类标签定、不按类型段定）。每条条目的身份按 `identity_invariant`（墓碑树 I-9.23、
     /// 谱系树 I-9.17）单独判定，身份不符不记走读失败（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T7 取 a）。
     /// 用 checker 自己的读法，不调 `singlefs_core`（D13（验证路线） 已定项 5）。
     /// 递归的上界：每进一层类型段 0，层级减一（层级 0 的码 2 节点在这两棵树里没有条款定的形状，记走读失败）。
@@ -2120,6 +2121,31 @@ impl<'reader> Walk<'reader> {
                 &child,
                 &what,
             );
+            let class_the_entry_type_names =
+                match ContainerTreeEntryType::of(entry_type, record_type) {
+                    ContainerTreeEntryType::InternalNode => Some(MappingKeyUnitClass::IndexNode),
+                    ContainerTreeEntryType::Container => {
+                        Some(MappingKeyUnitClass::PackedRecordUnit)
+                    }
+                    ContainerTreeEntryType::Unregistered(_) => None,
+                };
+            if let Some(class_the_entry_type_names) = class_the_entry_type_names {
+                if let Some(child_class) = child_unit_class_other_than_the_entry_type_names(
+                    self.reader,
+                    &child,
+                    Some(class_the_entry_type_names),
+                ) {
+                    self.judge_a_container_tree_entry_whose_child_is_of_another_class(
+                        &child,
+                        entry_type,
+                        class_the_entry_type_names,
+                        child_class,
+                        identity_invariant,
+                        name,
+                    );
+                    continue;
+                }
+            }
             match ContainerTreeEntryType::of(entry_type, record_type) {
                 ContainerTreeEntryType::InternalNode => {
                     // 「子单元是码 2」与 I-1.6（类标签登记且三处相等） 重合，照 I-9.2（条目身份与子头相符） 的先例留作合取
@@ -2173,6 +2199,33 @@ impl<'reader> Walk<'reader> {
         }
     }
 
+    /// 墓碑树、谱系树里一条条目的类型段说孩子是一类（类型段 0 ⇒ 码 2，类型段 = 打包记录类型 ⇒ 码 3），孩子头里自证过的类标签是另一类：
+    /// I-9.23 / I-9.17 字面「0 ⇒ 子单元是码 2 … 1（谱系 5）⇒ 子单元是码 3」不成立，按 `identity_invariant` 判红，
+    /// 「身份不符单独判定，不再记走读失败」：不往下走、不记走读失败。孩子按它自己那一类占的槽先登记引用（同类型段不认得那一支：
+    /// 不登记的话它占的槽不进引用并集，I-3.1（已分配统计对得上） 连带红）。
+    fn judge_a_container_tree_entry_whose_child_is_of_another_class(
+        &mut self,
+        child: &PointerView,
+        entry_type: u16,
+        class_the_entry_type_names: MappingKeyUnitClass,
+        child_class: MappingKeyUnitClass,
+        identity_invariant: &'static str,
+        name: &str,
+    ) {
+        let span_in_slots = slots_spanned_by_a_unit_of_the_class(child_class)
+            .expect("child_unit_class_other_than_the_entry_type_names 只交回有长度的登记码");
+        self.note_references_of_a_pointer_followed_by_the_walk(
+            child,
+            span_in_slots,
+            &format!("{name}内部条目的子指针"),
+        );
+        self.judgements.judge(identity_invariant, false, || {
+            format!(
+                "{name}内部条目类型段 {entry_type} 说孩子是 {class_the_entry_type_names:?}，孩子头里的类标签是 {child_class:?}（按那一类的长度读回、整单元校验和对得上）"
+            )
+        });
+    }
+
     /// 墓碑树、谱系树里类型段 0 那条条目下面的码 2 节点：层级要是父层级 − 1，条目宽 120 才往下走（[`Self::walk_container_tree_root`]）。
     #[allow(
         clippy::too_many_arguments,
@@ -2372,6 +2425,36 @@ impl<'reader> Walk<'reader> {
                 &child,
                 "inode 内部条目的子指针",
             );
+            // I-9.2 判孩子的类按孩子头里的类标签认，不按类型段定读多长（同 I-9.17 / I-9.23，判决
+            // `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K2 N4 那一行「inode 树 I-9.2 同形」）：
+            // 孩子自证过的类与类型段说的不符（类型段不认得的，孩子读得出）判 I-9.2 红，按它自己那一类占的槽登记引用，不往下走、不记走读失败。
+            let class_the_entry_type_names = match entry_type {
+                InodeInternalEntryType::LeafContainer => {
+                    Some(MappingKeyUnitClass::PackedRecordUnit)
+                }
+                InodeInternalEntryType::InternalNode => Some(MappingKeyUnitClass::IndexNode),
+                InodeInternalEntryType::Unregistered(_) => None,
+            };
+            if let Some(child_class) = child_unit_class_other_than_the_entry_type_names(
+                self.reader,
+                &child,
+                class_the_entry_type_names,
+            ) {
+                let span_in_slots = slots_spanned_by_a_unit_of_the_class(child_class).expect(
+                    "child_unit_class_other_than_the_entry_type_names 只交回有长度的登记码",
+                );
+                self.note_references_of_a_pointer_followed_by_the_walk(
+                    &child,
+                    span_in_slots,
+                    "inode 树的孩子",
+                );
+                self.judgements.judge("I-9.2", false, || {
+                    format!(
+                        "inode 内部条目类型段 {record_type} 说孩子是 {class_the_entry_type_names:?}，孩子头里的类标签是 {child_class:?}（按那一类的长度读回、整单元校验和对得上）"
+                    )
+                });
+                continue;
+            }
             let (length, span_in_slots) = match entry_type {
                 InodeInternalEntryType::LeafContainer => (data_unit_bytes(), 2),
                 InodeInternalEntryType::InternalNode | InodeInternalEntryType::Unregistered(_) => {
@@ -3920,6 +4003,61 @@ fn read_unit_without_judging(
     })
 }
 
+/// 共同前缀里类标签的偏移（D18（块里携带什么信息） 已定项 7 / 11：三种码同一处）。
+const UNIT_HEADER_CLASS_TAG_OFFSET: usize = 6;
+
+/// 一类单元的整单元长度：码 1 数据单元与码 3 打包记录单元 32 KiB，码 2 节点 16 KiB；登记表之外的码没有长度（交 None）。
+fn unit_bytes_of_the_class(class: MappingKeyUnitClass) -> Option<usize> {
+    match class {
+        MappingKeyUnitClass::DataUnit | MappingKeyUnitClass::PackedRecordUnit => {
+            Some(data_unit_bytes())
+        }
+        MappingKeyUnitClass::IndexNode => Some(node_bytes()),
+        MappingKeyUnitClass::Unregistered(_) => None,
+    }
+}
+
+/// 一类单元占几个 16 KiB 槽（引用集合按它记）：32 KiB 的两槽，16 KiB 的一槽。
+fn slots_spanned_by_a_unit_of_the_class(class: MappingKeyUnitClass) -> Option<u64> {
+    unit_bytes_of_the_class(class)
+        .map(|unit_bytes| u64::try_from(unit_bytes).expect("16384 或 32768") / SLOT_BYTES)
+}
+
+/// 树里一条内部条目的孩子是不是条目类型段说的那一类：只读孩子的单元头取类标签，不按类型段定读多长（判决
+/// `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K2 N4 那一行：按类型段定长度时，类型段与孩子真实的类不符，
+/// 按那个长度读回的整单元校验和对不上，身份那一判跑不到、却记了 I-2.1 与走读失败）。
+/// 逐条位置条目：头里的类标签就是 `class_the_entry_type_names` 的，交回 None（照类型段那一支走，读不出照旧归 I-2.1）；
+/// 是另一类登记过的码、且按那一类的长度读回的整单元校验和对得上，交回那一类；类标签不在登记表里、读不出、校验和对不上的看下一条位置条目。
+/// 类型段不认得的条目（`class_the_entry_type_names` 是 None）没有「它说的那一类」，交回孩子自证过的那一类。
+/// 只认按它自己的长度自证过的孩子：头里的类标签坏了的单元按那个长度对不上校验和，不会被当成「另一类」。只读、不判。
+fn child_unit_class_other_than_the_entry_type_names(
+    reader: &dyn ImageReader,
+    child: &PointerView,
+    class_the_entry_type_names: Option<MappingKeyUnitClass>,
+) -> Option<MappingKeyUnitClass> {
+    for location in &child.locations {
+        let offset = location.slot * SLOT_BYTES;
+        // 头按最短的单元（码 2，16 KiB）那么长读：读者只认整块的读，32 KiB 的单元头也在这前 16 KiB 里。
+        let Some(header) = reader.read(location.device, offset, node_bytes()) else {
+            continue;
+        };
+        let class_in_the_header = MappingKeyUnitClass::of(header[UNIT_HEADER_CLASS_TAG_OFFSET]);
+        if Some(class_in_the_header) == class_the_entry_type_names {
+            return None;
+        }
+        let Some(unit_bytes) = unit_bytes_of_the_class(class_in_the_header) else {
+            continue;
+        };
+        let verifies_at_the_length_of_its_class = reader
+            .read(location.device, offset, unit_bytes)
+            .is_some_and(|content| crc32_castagnoli_table(&content) == location.checksum);
+        if verifies_at_the_length_of_its_class {
+            return Some(class_in_the_header);
+        }
+    }
+    None
+}
+
 /// 这一遍里读过的码 2 单元：(设备身份, 起点槽, 位置条目里的整单元校验和) → 解开的节点（解不开记 None）。
 /// 候选集里几条根常指着同一个单元（暖机那几代都指着 mkfs 种的第 0 版树表，共享子树在每条根上都要算一遍），
 /// 按位置条目记一份，省掉重复的整单元 CRC——层 0 每个崩溃状态都要跑这一遍。
@@ -9017,6 +9155,40 @@ impl TombstoneContainerCapacityKnob {
     }
 }
 
+/// 删除线新九条（I-9.17–I-9.23、I-9.29；I-9.24 的在 [`judge_deletion_state_of_the_versions`] 末尾）一次都没判到时各自报的不适用理由：
+/// 每条一句、说清这张镜像上为什么没有它判的对象，不落到 `Judgements::into_report` 的缺省理由「这个镜像上没有它判的对象」。
+/// 公开给「逐条判定、不在豁免里的一律要求成立」的用例逐字认（豁免只认这一句，别的不适用照红）。
+/// I-9.18（删除队列里的号有谱系上的墓碑）：每一版的队列都是空的。
+pub const NO_QUEUED_INODE_IN_ANY_WALKED_VERSION: &str =
+    "走读到的每一版 deleted_inodes 里都没有号：没有队列里的号可判";
+/// I-9.19（谱系上的墓碑不指活对象）：没有一条谱系上的墓碑在同一版 inode 树里有同号同对象出生代的记录。
+pub const NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD: &str =
+    "走读到的每一版里都没有在谱系上、而 inode 树里还有同号同对象出生代记录的墓碑";
+/// I-9.20（谱系上的墓碑不重号且记录合法）：每一版墓碑树都是零根。
+pub const NO_TOMBSTONE_IN_ANY_WALKED_VERSION: &str =
+    "走读到的每一版墓碑树都是零根：没有墓碑记录可判";
+/// I-9.29（同号只许一个出生代）：没有一版收到号（inode 记录与谱系上的墓碑都没有）。
+pub const NO_INODE_RECORD_OR_TOMBSTONE_ON_THE_LINEAGE_TO_COMPARE: &str =
+    "走读到的每一版里都没有 inode 记录与谱系上的墓碑：没有号可比";
+/// I-9.21（产品镜像里非最右墓碑容器装满），旧入口：没有非最右的容器。
+pub const NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER: &str =
+    "走读到的每一版墓碑树都是零根或只一片容器：没有非最右的容器";
+/// I-9.22（测试镜像里墓碑容器不超旋钮值），旧入口：当没装旋钮，这一条不判。
+pub const NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY: &str =
+    "池级 checker 的旧入口，当没装只供测试的墓碑容量旋钮：旋钮那一条不判";
+/// I-9.21，新入口给了旋钮值：产品容量那一条不判。
+pub const PRODUCT_TOMBSTONE_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOB: &str =
+    "这张镜像用了只供测试的墓碑容量旋钮（新入口给了旋钮值）：产品容量那一条不判";
+/// I-9.22，新入口给了旋钮值：每一版墓碑树都是零根。
+pub const NO_TOMBSTONE_CONTAINER_UNDER_THE_KNOB: &str =
+    "走读到的每一版墓碑树都是零根：没有墓碑容器可判";
+/// I-9.17（谱系树条目身份与子头相符）：主走读一条谱系树内部条目都没走到。
+pub const NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
+    "走读到的每一版谱系树都是零根：没有谱系树内部条目可判";
+/// I-9.23（墓碑树条目身份与子头相符）：主走读一条墓碑树内部条目都没走到。
+pub const NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
+    "走读到的每一版墓碑树都是零根：没有墓碑树内部条目可判";
+
 /// 墓碑与谱系那一遍在一版上判 I-9.18、I-9.19、I-9.20、I-9.29 要的那一组：一版的树表读不出、墓碑树读不全时这几条在那一版判不了
 /// （I-9.21 / I-9.22 只记装没装旋钮的那一条，[`capacity_invariant_of`]）。
 const TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS: [&str; 4] = [
@@ -9375,39 +9547,39 @@ fn judge_tombstones_and_the_lineage_of_the_versions(
     // 一次都没判到时的理由（判到过的照判定报，`Judgements::not_applicable` 不改已经评估过的）。
     judgements.not_applicable(
         QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
-        "走读到的每一版 deleted_inodes 里都没有号：没有队列里的号可判",
+        NO_QUEUED_INODE_IN_ANY_WALKED_VERSION,
     );
     judgements.not_applicable(
         TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
-        "走读到的每一版里都没有在谱系上、而 inode 树里还有同号同对象出生代记录的墓碑",
+        NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD,
     );
     judgements.not_applicable(
         TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
-        "走读到的每一版墓碑树都是零根：没有墓碑记录可判",
+        NO_TOMBSTONE_IN_ANY_WALKED_VERSION,
     );
     judgements.not_applicable(
         ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
-        "走读到的每一版里都没有 inode 记录与谱系上的墓碑：没有号可比",
+        NO_INODE_RECORD_OR_TOMBSTONE_ON_THE_LINEAGE_TO_COMPARE,
     );
     match capacity_knob {
         None => {
             judgements.not_applicable(
                 NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
-                "走读到的每一版墓碑树都是零根或只一片容器：没有非最右的容器",
+                NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER,
             );
             judgements.not_applicable(
                 TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
-                "池级 checker 的旧入口，当没装只供测试的墓碑容量旋钮：旋钮那一条不判",
+                NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY,
             );
         }
         Some(_) => {
             judgements.not_applicable(
                 NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
-                "这张镜像用了只供测试的墓碑容量旋钮（新入口给了旋钮值）：产品容量那一条不判",
+                PRODUCT_TOMBSTONE_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOB,
             );
             judgements.not_applicable(
                 TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
-                "走读到的每一版墓碑树都是零根：没有墓碑容器可判",
+                NO_TOMBSTONE_CONTAINER_UNDER_THE_KNOB,
             );
         }
     }
@@ -10594,11 +10766,11 @@ pub fn check_pool_image_given_the_tombstone_container_capacity_knob(
     // I-9.17、I-9.23 在主走读里每条内部条目判；一条条目都没走到时的理由。
     judgements.not_applicable(
         LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
-        "走读到的每一版谱系树都是零根：没有谱系树内部条目可判",
+        NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION,
     );
     judgements.not_applicable(
         TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
-        "走读到的每一版墓碑树都是零根：没有墓碑树内部条目可判",
+        NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION,
     );
     // I-4.4（意图只描述目标态）：第一版的意图载体没有可装增量的字段，报不适用并写理由（用户 2026-10-05 定）。
     judgements.not_applicable(
diff --git a/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs b/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs
index ec01f6f..e69c16d 100644
--- a/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs
+++ b/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs
@@ -17,7 +17,7 @@
 //! （E142 第 22 次第二段的 P8G：两块 8 GiB 盘，`research/prompts/e142-r22-prereg.md` 读法 R67），别的参数照旧，
 //! `make_filesystem` 收不收由它自己判（环不超过盘的四分之一）。`name=impl_config` 的 `image_bytes=` 打的是这一次的每盘字节数。
 //! 不读环境、不碰真设备、不写任何文件。退出码：0 = 正常；1 = 命令行参数不是上面两种之一，或每盘字节数解得开而
-//! 两块盘内存里的单元区空闲图在这台机器上分配不出（例：`--device-bytes 18446744073709551615`，建盘之前判），
+//! 两块盘内存里的单元区空闲图合起来超过定死的上界或算不出（例：`--device-bytes 18446744073709551615`，建盘之前按算术判），
 //! 或 `make_filesystem` 不收（例：`--device-bytes 0`），三种都在 stderr 打原因（最后一种打 mkfs 交回的错）；
 //! 101 = mkfs 收了、后面某一步（取号、暖机、新池新建文件）在内存盘上没走通，panic 带那一步与原因。
 
@@ -139,13 +139,18 @@ fn device_bytes_from_the_command_line(arguments: &[String]) -> Result<u64, Strin
 /// 每槽三个布尔数组（已分配、隔离、扣到 F 生效）3 字节，每 64 槽一段的四个 `u64` 计数数组合起来每槽半个字节，向上取整。
 const FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST: u64 = 4;
 
-/// 两块盘的单元区空闲图在这台机器上分配得出：mkfs 收下之后建分配器（`allocator_after_make_filesystem`）照每盘单元区槽数分配每槽一格的空闲图，
-/// 那一步分配不出时进程当场 abort（退 134：`--device-bytes 18446744073709551615` 建盘就是这样，代码轮判决
-/// `research/prompts/m3-batch20-code-r1-main-verification.md` 第二节 K1 那一格）。这里在建盘之前按 [`FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST`]
-/// 可失败地把两块盘那么多一起试着要一次，要到了就放掉；要不到交回给人看的一句。
-/// 单元区起点不在聚簇段边界上、盘末尾在单元区起点之前的不试，mkfs 自己拒（`UnitAreaStartOffTheClusterSegmentBoundaryUnsupported`、
+/// 两块盘的单元区空闲图合起来至多这么多字节，超过的每盘字节数建盘之前就拒（1 GiB：每盘单元区至多 2²⁷ 槽、单元区 2 TiB，
+/// 跑前登记过的 4 GiB 与 P8G 的 8 GiB 两档都远在里面；这个数是推的，没量过哪一档开始分配不出）。按算术判、不试分配：
+/// 判据不随这台机器的内存与 overcommit 设定变（第 23 批 X 那一版试分配要不到才拒，主 agent 定改成按算术判）。
+const FREE_MAP_BYTES_OF_THE_TWO_DEVICES_AT_MOST: u64 = 1 << 30;
+
+/// 两块盘的单元区空闲图大小在 [`FREE_MAP_BYTES_OF_THE_TWO_DEVICES_AT_MOST`] 之内：mkfs 收下之后建分配器（`allocator_after_make_filesystem`）照每盘单元区槽数
+/// 分配每槽一格的空闲图，那一步分配不出时进程当场 abort（退 134：`--device-bytes 18446744073709551615` 建盘就是这样，代码轮判决
+/// `research/prompts/m3-batch20-code-r1-main-verification.md` 第二节 K1 那一格）。这里在建盘之前按 [`FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST`] 算：
+/// 每盘单元区槽数 × 每槽字节数 × 盘数，用 checked 运算，乘法溢出或超过上界交回给人看的一句。
+/// 单元区起点不在聚簇段边界上、盘末尾在单元区起点之前的不在这里判，mkfs 自己拒（`UnitAreaStartOffTheClusterSegmentBoundaryUnsupported`、
 /// `UnitAreaBeyondDevice`），照旧打 mkfs 交回的错。
-fn free_maps_of_the_two_devices_fit_in_memory(
+fn free_maps_of_the_two_devices_stay_within_the_bound(
     device_bytes: u64,
     journal_ring_bytes: u64,
 ) -> Result<(), String> {
@@ -156,23 +161,19 @@ fn free_maps_of_the_two_devices_fit_in_memory(
     else {
         return Ok(());
     };
-    let refusal = |why: String| {
+    let refusal = |why: &str| {
         format!(
-            "每盘 {device_bytes} 字节：单元区 {unit_area_slots} 槽，内存里的空闲图每盘至多要 {FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST} × {unit_area_slots} 字节，{DEVICE_COUNT} 块盘在这台机器上分配不出（{why}）"
+            "每盘 {device_bytes} 字节：单元区 {unit_area_slots} 槽，内存里的空闲图每盘至多要 {FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST} × {unit_area_slots} 字节，{DEVICE_COUNT} 块盘合起来{why}"
         )
     };
-    let free_map_bytes_per_device = unit_area_slots
+    let free_map_bytes_of_the_two_devices = unit_area_slots
         .checked_mul(FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST)
-        .and_then(|bytes| usize::try_from(bytes).ok())
-        .ok_or_else(|| refusal("字节数超出这台机器的地址宽度".to_string()))?;
-    // 两块盘的空闲图同时活着：两份一起要到才算要得到。迭代上界是盘数。
-    let mut reserved_free_maps: Vec<Vec<u8>> = Vec::new();
-    for _device_number in 0..DEVICE_COUNT {
-        let mut free_map: Vec<u8> = Vec::new();
-        free_map
-            .try_reserve_exact(free_map_bytes_per_device)
-            .map_err(|error| refusal(error.to_string()))?;
-        reserved_free_maps.push(free_map);
+        .and_then(|bytes_per_device| bytes_per_device.checked_mul(u64::from(DEVICE_COUNT)))
+        .ok_or_else(|| refusal("算不出（乘法溢出 u64）"))?;
+    if free_map_bytes_of_the_two_devices > FREE_MAP_BYTES_OF_THE_TWO_DEVICES_AT_MOST {
+        return Err(refusal(&format!(
+            " {free_map_bytes_of_the_two_devices} 字节，超过上界 {FREE_MAP_BYTES_OF_THE_TWO_DEVICES_AT_MOST} 字节"
+        )));
     }
     Ok(())
 }
@@ -205,7 +206,7 @@ fn main() {
     };
 
     let parameters = e142_parameters(PHYSICAL_BLOCK_SIZE_IN_BYTES, MINIMUM_INPUT_OUTPUT_BYTES);
-    if let Err(refusal) = free_maps_of_the_two_devices_fit_in_memory(
+    if let Err(refusal) = free_maps_of_the_two_devices_stay_within_the_bound(
         device_bytes,
         parameters.geometry.journal_ring_bytes,
     ) {
@@ -332,10 +333,13 @@ fn main() {
 mod tests {
     use super::{
         device_bytes_from_the_command_line, ending_of_an_unfinished_path,
-        free_maps_of_the_two_devices_fit_in_memory, two_recording_memory_devices,
-        EndingOfAnUnfinishedPath, MINIMUM_INPUT_OUTPUT_BYTES, PHYSICAL_BLOCK_SIZE_IN_BYTES,
+        free_maps_of_the_two_devices_stay_within_the_bound, two_recording_memory_devices,
+        EndingOfAnUnfinishedPath, DEVICE_COUNT, FREE_MAP_BYTES_OF_THE_TWO_DEVICES_AT_MOST,
+        FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST, MINIMUM_INPUT_OUTPUT_BYTES,
+        PHYSICAL_BLOCK_SIZE_IN_BYTES,
     };
-    use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
+    use singlefs_core::allocator::{unit_area_slots_of_device_starting_at, UnitAreaStart};
+    use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES};
     use singlefs_harness::scenario::{
         e142_parameters, run_new_pool_file_creation, NewPoolFileCreationPathFailure,
         NewPoolFileCreationPathStep,
@@ -409,27 +413,30 @@ mod tests {
         assert_eq!(stream.operation_count(), 0, "mkfs 拒在任何写之前");
     }
 
-    /// `--device-bytes 18446744073709551615` 解得开（第二种写法），两块盘的单元区空闲图分配不出：建盘之前照命令行给错了收场、打原因，
+    /// `--device-bytes 18446744073709551615` 解得开（第二种写法），两块盘的单元区空闲图按算术超过上界：建盘之前照命令行给错了收场、打原因，
     /// 不走到建分配器那一步 abort（退 134，代码轮判决 `research/prompts/m3-batch20-code-r1-main-verification.md` 第二节 K1 那一格）。
     /// 跑前登记的 4 GiB 与 P8G 的 8 GiB 两档照旧放过；每盘 0 字节不在这里拒，交给 mkfs 判（上一条）。
     #[test]
-    fn the_largest_device_bytes_parse_but_their_free_maps_cannot_be_allocated_and_the_bin_ends_as_a_refused_command_line(
+    fn the_largest_device_bytes_parse_but_their_free_maps_exceed_the_bound_and_the_bin_ends_as_a_refused_command_line(
     ) {
         let device_bytes = device_bytes_from_the_command_line(&arguments(&[
             "--device-bytes",
             "18446744073709551615",
         ]))
         .expect("2⁶⁴ − 1 是一个十进制字节数，解得开");
-        match free_maps_of_the_two_devices_fit_in_memory(device_bytes, JOURNAL_RING_DEFAULT_BYTES) {
-            Ok(()) => panic!("每盘 2⁶⁴ − 1 字节的单元区空闲图不该分配得出"),
+        match free_maps_of_the_two_devices_stay_within_the_bound(
+            device_bytes,
+            JOURNAL_RING_DEFAULT_BYTES,
+        ) {
+            Ok(()) => panic!("每盘 2⁶⁴ − 1 字节的单元区空闲图不该在上界之内"),
             Err(refusal) => assert!(
-                refusal.contains("分配不出") && refusal.contains("每盘 18446744073709551615 字节"),
-                "原因点名每盘字节数与分配不出：{refusal}"
+                refusal.contains("超过上界") && refusal.contains("每盘 18446744073709551615 字节"),
+                "原因点名每盘字节数与超过上界：{refusal}"
             ),
         }
         for device_bytes_that_fit in [4_294_967_296, 8_589_934_592, 0] {
             assert_eq!(
-                free_maps_of_the_two_devices_fit_in_memory(
+                free_maps_of_the_two_devices_stay_within_the_bound(
                     device_bytes_that_fit,
                     JOURNAL_RING_DEFAULT_BYTES
                 ),
@@ -439,6 +446,38 @@ mod tests {
         }
     }
 
+    /// 上界那一格：两块盘的空闲图合起来恰好等于上界的每盘字节数放过，再多一个槽就拒（判据是算术，不随机器变）。
+    #[test]
+    fn device_bytes_at_the_free_map_bound_pass_and_one_slot_more_is_refused() {
+        let unit_area_start = UnitAreaStart::following_the_journal_ring(JOURNAL_RING_DEFAULT_BYTES)
+            .expect("默认环长的单元区起点在聚簇段边界上");
+        let slot_bytes = SLOT_BYTES;
+        let unit_area_slots_at_the_bound = FREE_MAP_BYTES_OF_THE_TWO_DEVICES_AT_MOST
+            / (FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST * u64::from(DEVICE_COUNT));
+        let device_bytes_at_the_bound =
+            (unit_area_start.slot().0 + unit_area_slots_at_the_bound) * slot_bytes;
+        assert_eq!(
+            unit_area_slots_of_device_starting_at(device_bytes_at_the_bound, unit_area_start),
+            Ok(unit_area_slots_at_the_bound),
+            "这么多字节的盘单元区恰好是上界那么多槽"
+        );
+        assert_eq!(
+            free_maps_of_the_two_devices_stay_within_the_bound(
+                device_bytes_at_the_bound,
+                JOURNAL_RING_DEFAULT_BYTES
+            ),
+            Ok(()),
+            "恰好在上界上放过"
+        );
+        match free_maps_of_the_two_devices_stay_within_the_bound(
+            device_bytes_at_the_bound + slot_bytes,
+            JOURNAL_RING_DEFAULT_BYTES,
+        ) {
+            Ok(()) => panic!("比上界多一个槽要拒"),
+            Err(refusal) => assert!(refusal.contains("超过上界"), "原因写超过上界：{refusal}"),
+        }
+    }
+
     /// mkfs 收了而后面某一步没走通的，照旧当实现的毛病（panic），不混进命令行给错了那一种。
     #[test]
     fn a_failure_after_make_filesystem_stays_an_implementation_failure() {
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index ccad789..1ad669e 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -9475,3 +9475,535 @@ fn a_leaf_holding_the_same_inode_number_twice_reddens_only_the_container_number_
         "违例说明写出不严格递增的那一对"
     );
 }
+
+/// 容器树与 inode 树的码 2 节点条目宽（分隔 key 8 + 身份引用 26 + 子指针 86），身份引用里出生树在条目偏移 8、容器号与出生代在 18 起那 16 字节。
+const INTERNAL_ENTRY_BYTES: usize = 120;
+const INTERNAL_ENTRY_IDENTITY_BIRTH_TREE: std::ops::Range<usize> = 8..16;
+const INTERNAL_ENTRY_IDENTITY_CONTAINER_AND_BIRTH: std::ops::Range<usize> = 18..34;
+/// 墓碑树、谱系树根长高之后根的条目类型段（指下一层码 2 节点）；墓碑容器、谱系容器的打包记录类型（D18（块里携带什么信息） 已定项 24）。
+const CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE: u16 = 0;
+const PACKED_RECORD_TYPE_TOMBSTONE: u16 = 1;
+const PACKED_RECORD_TYPE_LINEAGE: u16 = 5;
+const PACKED_RECORD_TYPE_INODE: u16 = 2;
+/// 码 3 单元头里容器号的偏移（类身份段：出生树 43、打包记录类型 51、容器号 53、容器出生代 61）。
+const PACKED_UNIT_CONTAINER_NUMBER_OFFSET: usize = 53;
+
+/// 最新的根那一版树表里种类 `kind` 那棵树（墓碑树、谱系树、inode 树）的码 2 根第 `entry_index` 条条目改一处（`change` 拿到那 120 字节），
+/// 根重封、新的整单元校验和沿引用链补到根槽；交回根的层级。
+fn change_an_entry_of_the_root_of_kind(
+    image: &mut MemoryPool,
+    kind: u16,
+    entry_index: usize,
+    change: impl FnOnce(&mut [u8]),
+) -> u8 {
+    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
+    let root_slot = common_pointer_head_corruption::the_slot_of(&root_pointer);
+    let mut level_of_the_root = 0;
+    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
+        image,
+        root_slot,
+        common_pointer_head_corruption::node_bytes(),
+        |root| {
+            level_of_the_root = root[INDEX_NODE_LEVEL_OFFSET];
+            let start = 115
+                + 2 * usize::from(root[INDEX_NODE_KEY_WIDTH_OFFSET])
+                + entry_index * INTERNAL_ENTRY_BYTES;
+            change(&mut root[start..start + INTERNAL_ENTRY_BYTES]);
+        },
+    );
+    level_of_the_root
+}
+
+fn entry_type_of(entry: &[u8]) -> u16 {
+    u16::from_le_bytes([
+        entry[CONTAINER_TREE_ENTRY_TYPE_OFFSET],
+        entry[CONTAINER_TREE_ENTRY_TYPE_OFFSET + 1],
+    ])
+}
+
+/// 条目类型段从 `from` 改成 `to`（改之前先认它是 `from`）。
+fn change_the_entry_type(entry: &mut [u8], from: u16, to: u16) {
+    assert_eq!(entry_type_of(entry), from, "改之前条目类型段是 {from}");
+    set_u16(entry, CONTAINER_TREE_ENTRY_TYPE_OFFSET, to);
+}
+
+/// 条目身份引用的出生树、容器号、出生代三段清零（类型段 0 的条目该有的样子），子指针不动。
+fn zero_the_three_identity_fields(entry: &mut [u8]) {
+    entry[INTERNAL_ENTRY_IDENTITY_BIRTH_TREE].fill(0);
+    entry[INTERNAL_ENTRY_IDENTITY_CONTAINER_AND_BIRTH].fill(0);
+}
+
+/// 判红的说明里点名孩子头里的类与类型段说的那一类不符（`walk` 按孩子头里的类标签定读多长）。
+fn assert_the_violation_names_the_child_class(
+    image: &MemoryPool,
+    invariant: &str,
+    entry_type: u16,
+    child_class_in_the_header: &str,
+) {
+    assert!(
+        matches!(
+            verdict(image, invariant),
+            InvariantVerdict::Violated(detail)
+                if detail.contains(&format!("类型段 {entry_type} 说孩子是"))
+                    && detail.contains(&format!("孩子头里的类标签是 {child_class_in_the_header}"))
+        ),
+        "{invariant} 的违例说明写出类型段 {entry_type} 与孩子头里的类 {child_class_in_the_header}：{:?}",
+        verdict(image, invariant)
+    );
+}
+
+/// N4-a（判决 `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K2 N4 那一行）：三个单元的文件发完意图之后，墓碑树根（层级 1）
+/// 第一条条目的类型段 1 → 0，孩子还是那片码 3 墓碑容器。I-9.23 字面「0 ⇒ 子单元是码 2」不成立；孩子读多长按它头里的类标签定，
+/// 不按类型段：只红 I-9.23，不连带 I-4.8、I-7.2、I-7.4（改之前按 16 KiB 读码 3 孩子读不出，这一小句放过、又记走读失败）。
+#[test]
+fn a_tombstone_tree_entry_of_type_zero_over_a_container_reddens_only_the_tombstone_entry_invariant()
+{
+    let mut image = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+        "known-bad-tombstone-entry-type-zero-over-a-container",
+    );
+    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            PACKED_RECORD_TYPE_TOMBSTONE,
+            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+        );
+    });
+    assert_eq!(level, 1, "墓碑树根直接管容器");
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
+        "类型段 0 的条目指着码 3 容器：只红 I-9.23，不连带 I-4.8、I-7.2、I-7.4：{verdicts:?}"
+    );
+    assert_the_violation_names_the_child_class(
+        &image,
+        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+        "PackedRecordUnit",
+    );
+}
+
+/// N4-i：同 N4-a，再把身份引用的出生树、容器号、出生代三段清零——这条条目除了孩子的类之外都是一条合法的类型段 0 条目。
+/// 改之前孩子读不出、「子单元是码 2」那一小句放过，I-9.23 报成立（漏判）；现在只红 I-9.23。
+#[test]
+fn a_tombstone_tree_entry_of_type_zero_with_a_zero_identity_over_a_container_reddens_only_the_tombstone_entry_invariant(
+) {
+    let mut image = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+        "known-bad-tombstone-entry-type-zero-zero-identity-over-a-container",
+    );
+    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            PACKED_RECORD_TYPE_TOMBSTONE,
+            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+        );
+        zero_the_three_identity_fields(entry);
+    });
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
+        "身份三段为 0 的类型段 0 条目指着码 3 容器：只红 I-9.23：{verdicts:?}"
+    );
+}
+
+/// N4-c（谱系树那一侧，I-9.17 与 I-9.23 同形）：回退到 A 之后，谱系树根第一条条目的类型段 5 → 0，孩子还是那片码 3 谱系容器。只红 I-9.17。
+#[test]
+fn a_lineage_tree_entry_of_type_zero_over_a_container_reddens_only_the_lineage_entry_invariant() {
+    let rolled_back =
+        image_after_rolling_back_to_the_first_root("known-bad-lineage-entry-type-zero");
+    let mut image = rolled_back.image.clone();
+    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_LINEAGE, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            PACKED_RECORD_TYPE_LINEAGE,
+            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+        );
+    });
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
+        "谱系树类型段 0 的条目指着码 3 容器：只红 I-9.17：{verdicts:?}"
+    );
+    assert_the_violation_names_the_child_class(
+        &image,
+        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+        "PackedRecordUnit",
+    );
+}
+
+/// I-9.17 类型段不认得那一支（M1 谱系那一侧，判决 `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K1 覆盖缺口那一行）：
+/// 回退到 A 之后，谱系树根第一条条目的类型段 5 → 7。孩子先登记引用再判红：只红 I-9.17，不连带 I-3.1。
+#[test]
+fn a_lineage_tree_entry_with_an_unregistered_type_reddens_only_the_lineage_entry_invariant() {
+    let rolled_back =
+        image_after_rolling_back_to_the_first_root("known-bad-lineage-entry-unregistered-type");
+    let mut image = rolled_back.image.clone();
+    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_LINEAGE, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            PACKED_RECORD_TYPE_LINEAGE,
+            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
+        );
+    });
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
+        "谱系树根条目的类型段不认得：只红 I-9.17，不连带 I-3.1：{verdicts:?}"
+    );
+    assert!(
+        matches!(
+            verdict(&image, LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER),
+            InvariantVerdict::Violated(detail) if detail.contains("类型段 7 不在 {0, 5} 里")
+        ),
+        "违例说明写出不认得的类型段"
+    );
+}
+
+/// I-9.2（inode 树条目身份与子头相符，与 I-9.17 / I-9.23 同形）：新池新建文件那一版，inode 树根（层级 1）第一条条目的类型段 2 → 0，
+/// 孩子还是那片码 3 叶容器。I-9.2 字面「类型段 0 ⇒ 子单元偏移 6 = 2」不成立；孩子读多长按它头里的类标签定：只红 I-9.2，
+/// 不连带 I-2.1（改之前按 16 KiB 读 32 KiB 的叶容器，校验和对不上，I-2.1 红、记走读失败，I-9.2 一次都没判到）。
+#[test]
+fn an_inode_tree_entry_of_type_zero_over_a_leaf_container_reddens_only_the_inode_entry_invariant() {
+    let mut image = build_pool("known-bad-inode-entry-type-zero-over-a-leaf").memory_pool();
+    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            PACKED_RECORD_TYPE_INODE,
+            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+        );
+    });
+    assert_eq!(level, 1, "inode 树根直接管叶容器");
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-9.2"],
+        "inode 树类型段 0 的条目指着码 3 叶容器：只红 I-9.2：{verdicts:?}"
+    );
+    assert_the_violation_names_the_child_class(
+        &image,
+        "I-9.2",
+        CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+        "PackedRecordUnit",
+    );
+}
+
+/// 同上，再把身份引用三段清零：除了孩子的类，这是一条合法的类型段 0 条目。只红 I-9.2。
+#[test]
+fn an_inode_tree_entry_of_type_zero_with_a_zero_identity_over_a_leaf_container_reddens_only_the_inode_entry_invariant(
+) {
+    let mut image =
+        build_pool("known-bad-inode-entry-type-zero-zero-identity-over-a-leaf").memory_pool();
+    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            PACKED_RECORD_TYPE_INODE,
+            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+        );
+        zero_the_three_identity_fields(entry);
+    });
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-9.2"],
+        "身份三段为 0 的 inode 树类型段 0 条目指着码 3 叶容器：只红 I-9.2：{verdicts:?}"
+    );
+}
+
+/// 写这几份长高的墓碑树时容器与节点压到的大小（照 `container_trees_grow_a_level_when_the_root_is_full.rs`）：一片 1 条、一个码 2 节点 2 条，
+/// 第 3 片容器根满长一层。池级 checker 走带旋钮值的入口，旋钮值就是一片 1 条。
+const TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;
+
+fn capacities_of_one_record_and_two_entries(
+) -> singlefs_core::container_tree::ContainerTreeCapacities {
+    singlefs_core::container_tree::ContainerTreeCapacities {
+        tombstone_container:
+            singlefs_core::tombstone_tree::TombstoneContainerCapacity::CappedForTests(
+                singlefs_core::tombstone_tree::TombstoneRecordsPerContainerCappedForTests::new(
+                    TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
+                )
+                .expect("1..=815"),
+            ),
+        lineage_container: singlefs_core::lineage_tree::LineageContainerCapacity::CappedForTests(
+            singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(1)
+                .expect("1..=2039"),
+        ),
+        node_entries: singlefs_core::container_tree::ContainerTreeNodeEntries::CappedForTests(
+            singlefs_core::container_tree::ContainerTreeNodeEntriesCappedForTests::new(2)
+                .expect("2..=135"),
+        ),
+    }
+}
+
+/// unlink 一个空 inode 到底（意图，再走空），写入口装一片 1 条、一个节点 2 条的旋钮。
+fn unlink_an_empty_inode_with_one_record_per_tombstone_container(pool: &mut BuiltPool, inode: u64) {
+    let publish_parameters = parameters();
+    let previous = pool.output.clone();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+    writer.set_container_tree_capacities(capacities_of_one_record_and_two_entries());
+    let mut current = singlefs_core::transaction::publish_unlink_intent(
+        &mut writer,
+        &mut pool.allocator,
+        &previous,
+        singlefs_core::address::InodeNumber(inode),
+        common_deletion_in_progress::WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
+        common_deletion_in_progress::INSTANCE_OF_THE_PUBLISHES,
+    )
+    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
+    singlefs_core::transaction::publish_unlink_steps_until_the_queue_is_empty(
+        &mut writer,
+        &mut pool.allocator,
+        &mut current,
+        singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
+        common_deletion_in_progress::INSTANCE_OF_THE_PUBLISHES,
+    )
+    .expect("空 inode 一批就走空");
+    pool.output = current;
+}
+
+/// 建 5 个空 inode（2..=6 号），在旋钮下 unlink `inodes` 里那几个：两个时墓碑树根层级 1、两片容器；三个时根满长到层级 2。
+fn pool_with_empty_inodes_unlinked_under_the_tombstone_knob(
+    tag: &str,
+    inodes: &[u64],
+) -> BuiltPool {
+    let mut pool = common_deletion_in_progress::pool_with_new_inodes(tag, 5);
+    for inode in inodes {
+        unlink_an_empty_inode_with_one_record_per_tombstone_container(&mut pool, *inode);
+    }
+    pool
+}
+
+/// 池级 checker 带旋钮值（一片 1 条）的入口。
+fn check_pool_image_under_the_tombstone_knob(
+    image: &MemoryPool,
+) -> Vec<(&'static str, InvariantVerdict)> {
+    singlefs_checker::walk::check_pool_image_given_the_tombstone_container_capacity_knob(
+        image,
+        Some(
+            singlefs_checker::walk::TombstoneContainerCapacityKnob::new(
+                TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
+            )
+            .expect("旋钮值在 1..=815 里"),
+        ),
+    )
+}
+
+/// 带旋钮值的入口判出来的这一条的判定。
+fn verdict_under_the_tombstone_knob(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
+    check_pool_image_under_the_tombstone_knob(image)
+        .into_iter()
+        .find(|(identifier, _)| *identifier == invariant)
+        .expect("清单里有")
+        .1
+}
+
+/// 墓碑树长到层级 2 的干净镜像（2、3、4 号在旋钮下删完）；带旋钮值的入口上一条都不红。
+fn image_with_a_tombstone_tree_of_level_two(tag: &str) -> MemoryPool {
+    let image =
+        pool_with_empty_inodes_unlinked_under_the_tombstone_knob(tag, &[2, 3, 4]).memory_pool();
+    let verdicts = check_pool_image_under_the_tombstone_knob(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        Vec::<&str>::new(),
+        "长到层级 2 的干净镜像（带旋钮值）一条都不红：{verdicts:?}"
+    );
+    image
+}
+
+/// N4-f：墓碑树长到层级 2 之后，根第一条条目（类型段 0，孩子是层级 1 的码 2 节点）类型段 0 → 1。I-9.23 字面「1 ⇒ 子单元是码 3」不成立；
+/// 孩子读多长按它头里的类标签定（改之前按 32 KiB 读 16 KiB 的节点，I-2.1 红、记走读失败，身份那一判跑不到、I-9.23 报成立）。
+/// 孩子按码 2 一槽登记引用、不往下走：只红 I-9.23。
+#[test]
+fn a_tombstone_tree_entry_of_the_container_type_over_a_node_reddens_only_the_tombstone_entry_invariant(
+) {
+    let mut image = image_with_a_tombstone_tree_of_level_two(
+        "known-bad-tombstone-entry-container-type-over-a-node",
+    );
+    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+            PACKED_RECORD_TYPE_TOMBSTONE,
+        );
+    });
+    assert_eq!(level, 2, "墓碑树根长到层级 2");
+    let verdicts = check_pool_image_under_the_tombstone_knob(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
+        "类型段 1 的条目指着码 2 节点：只红 I-9.23，不连带 I-2.1、I-3.1、I-3.11、I-4.8、I-5.1、I-7.2、I-7.4：{verdicts:?}"
+    );
+    assert!(
+        matches!(
+            verdict_under_the_tombstone_knob(&image, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER),
+            InvariantVerdict::Violated(detail)
+                if detail.contains("类型段 1 说孩子是") && detail.contains("孩子头里的类标签是 IndexNode")
+        ),
+        "违例说明写出类型段 1 与孩子头里的码 2"
+    );
+}
+
+/// N4-j：同 N4-f，墓碑树长到层级 2 之后再建 30 次（根环转过一圈），根之下那几个节点只剩现行这棵墓碑树引用、别的候选版不再替它们登记引用。只红 I-9.23。
+#[test]
+fn a_tombstone_tree_entry_of_the_container_type_over_a_node_after_the_root_ring_turned_over_reddens_only_the_tombstone_entry_invariant(
+) {
+    let mut pool = pool_with_empty_inodes_unlinked_under_the_tombstone_knob(
+        "known-bad-tombstone-entry-container-type-over-a-node-ring-turned-over",
+        &[2, 3, 4],
+    );
+    common_deletion_in_progress::turn_the_root_ring_over_by_creating_inodes(&mut pool);
+    let clean = pool.memory_pool();
+    assert_eq!(
+        violated_invariants(&check_pool_image_under_the_tombstone_knob(&clean)),
+        Vec::<&str>::new(),
+        "根环转过一圈之后的干净镜像（带旋钮值）一条都不红"
+    );
+    let mut image = clean.clone();
+    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+            PACKED_RECORD_TYPE_TOMBSTONE,
+        );
+    });
+    let verdicts = check_pool_image_under_the_tombstone_knob(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
+        "根环转过一圈之后类型段 1 的条目指着码 2 节点：只红 I-9.23：{verdicts:?}"
+    );
+}
+
+/// M1（判决第二节 K1 覆盖缺口那一行：类型段 0 那一支的身份判定没有用例钉）：墓碑树长到层级 2 之后，根第一条条目（类型段 0、孩子是码 2 节点）
+/// 身份引用的容器号写成 1——类型段 0 要身份三段为 0。只红 I-9.23。
+#[test]
+fn a_tombstone_tree_entry_of_type_zero_with_a_container_number_reddens_only_the_tombstone_entry_invariant(
+) {
+    let mut image = image_with_a_tombstone_tree_of_level_two(
+        "known-bad-tombstone-entry-type-zero-container-number",
+    );
+    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
+        assert_eq!(
+            entry_type_of(entry),
+            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+            "根长高之后第一条条目类型段 0"
+        );
+        assert_eq!(
+            get_u64(entry, CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET),
+            0,
+            "类型段 0 的条目容器号是 0"
+        );
+        set_u64(entry, CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET, 1);
+    });
+    let verdicts = check_pool_image_under_the_tombstone_knob(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
+        "类型段 0 的条目身份容器号不为 0：只红 I-9.23：{verdicts:?}"
+    );
+    assert!(
+        matches!(
+            verdict_under_the_tombstone_knob(&image, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER),
+            InvariantVerdict::Violated(detail) if detail.contains("0 / 1 / 0（要三段为 0）")
+        ),
+        "违例说明写出身份三段"
+    );
+}
+
+/// M5（判决第二节 K1 覆盖缺口那一行：I-9.21 / I-9.22「容器号沿叶序严格递增」没有用例钉）：旋钮下 2、3 号删完，墓碑树根层级 1、两片容器；
+/// 第二片容器头里的容器号与根里指它那条条目的身份容器号一起改成第一片的号（身份照旧相符，分隔 key 不动）。带旋钮值的入口判 I-9.22：只红它。
+#[test]
+fn two_tombstone_containers_with_one_container_number_redden_only_the_capacity_knob_invariant() {
+    let pool = pool_with_empty_inodes_unlinked_under_the_tombstone_knob(
+        "known-bad-tombstone-containers-share-a-number",
+        &[2, 3],
+    );
+    let mut image = pool.memory_pool();
+    assert_eq!(
+        verdict_under_the_tombstone_knob(
+            &image,
+            TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE
+        ),
+        InvariantVerdict::Holds,
+        "两片容器的干净镜像上 I-9.22 真被评估过且成立"
+    );
+    let root_pointer =
+        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_TOMBSTONE).root;
+    let root =
+        singlefs_core::unit::parse_index_node(&common_pointer_head_corruption::unit_bytes_at(
+            &image,
+            common_pointer_head_corruption::the_slot_of(&root_pointer),
+            common_pointer_head_corruption::node_bytes(),
+        ))
+        .expect("墓碑树根解得开");
+    assert_eq!(
+        (root.level, root.entries.len()),
+        (1, 2),
+        "墓碑树根层级 1、两片容器"
+    );
+    let first_container_number = get_u64(
+        &root.entries[0],
+        CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET,
+    );
+    let second_container_pointer =
+        singlefs_core::pointer::NodePointer::read_from(&mut singlefs_core::bytes::ByteReader::at(
+            &root.entries[1],
+            CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET,
+        ));
+    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
+        &mut image,
+        common_pointer_head_corruption::the_slot_of(&second_container_pointer),
+        common_pointer_head_corruption::data_unit_bytes(),
+        |container| {
+            assert!(
+                get_u64(container, PACKED_UNIT_CONTAINER_NUMBER_OFFSET) > first_container_number,
+                "第二片的容器号大于第一片的"
+            );
+            set_u64(
+                container,
+                PACKED_UNIT_CONTAINER_NUMBER_OFFSET,
+                first_container_number,
+            );
+        },
+    );
+    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 1, |entry| {
+        set_u64(
+            entry,
+            CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET,
+            first_container_number,
+        );
+    });
+    let verdicts = check_pool_image_under_the_tombstone_knob(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE],
+        "两片墓碑容器同号：只红 I-9.22：{verdicts:?}"
+    );
+}
+
+/// 孩子头里的类标签只在「按那一类的长度读回、整单元校验和对得上」时才算孩子的类（`walk` 认孩子类的读法）：新池新建文件那一版，
+/// inode 叶容器（32 KiB）头里的类标签 3 → 2，不重封，整单元校验和沿引用链补上（位置条目对得上这 32 KiB）。类标签说码 2，
+/// 按 16 KiB 读回对不上校验和，不当成「另一类的孩子」：照类型段 2 那一支读 32 KiB，I-9.2 判类标签不是 3（违例说明写「（类 2）」），
+/// 孩子照两槽登记引用，I-3.1 不连带红。
+#[test]
+fn an_inode_leaf_whose_class_tag_alone_says_index_node_is_read_at_the_length_of_its_entry_type() {
+    let mut image = build_pool("known-bad-inode-leaf-class-tag-alone").memory_pool();
+    mutate_unit(&mut image, INODE_LEAF, |bytes| bytes[6] = 2, false);
+    assert!(
+        matches!(
+            verdict(&image, "I-9.2"),
+            InvariantVerdict::Violated(detail) if detail.contains("（类 2）不符")
+        ),
+        "I-9.2 照类型段 2 那一支判出孩子头的类不是 3：{:?}",
+        verdict(&image, "I-9.2")
+    );
+    assert!(
+        !matches!(verdict(&image, "I-3.1"), InvariantVerdict::Violated(_)),
+        "叶容器照两槽登记引用：I-3.1 不红：{:?}",
+        verdict(&image, "I-3.1")
+    );
+}
diff --git a/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs b/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs
index f963b53..c9ddbf6 100644
--- a/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs
+++ b/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs
@@ -14,7 +14,7 @@ use common::build_pool;
 use common_deletion_in_progress::{
     accounting_tree_slot_of_the_newest_root,
     append_a_copy_of_the_first_tombstone_of_the_newest_root,
-    change_the_first_tombstone_of_the_newest_root,
+    change_the_first_tombstone_of_the_newest_root, change_the_inode_record,
     change_the_pending_deletion_occupancy_row_of_the_accounting_tree_at,
     drop_the_pending_deletion_occupancy_row,
     drop_the_record_of_the_first_file_and_keep_its_data_units,
@@ -1282,3 +1282,217 @@ fn unlinking_the_same_inode_again_after_a_rollback_before_its_intent_holds_the_t
         );
     }
 }
+
+/// 判红的那几条，按名字排好（几条一起红时只比集合，不比 checker 报的次序）。
+fn violated_invariants_in_name_order(image: &MemoryPool) -> Vec<&'static str> {
+    let mut violated = violated_invariants(image);
+    violated.sort_unstable();
+    violated
+}
+
+/// M2（判决 `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K1 覆盖缺口那一行：I-9.29「与那一版谱系上墓碑的出生代都算」
+/// 墓碑那一侧没有用例钉）：2、3 号分两次建（对象出生代不同），3 号删完、根环转过一圈之后，那条墓碑的号 3 → 2、出生代不动（还是 3 号的）。
+/// 2 号的记录与谱系上的墓碑给出两个出生代：I-9.29 红；I-9.10 墓碑那一个等号按号认，同样红。墓碑的出生代与 2 号记录的不同，I-9.19 不判它。
+#[test]
+fn a_tombstone_on_the_lineage_carrying_another_object_birth_for_a_live_number_reddens_the_one_birth_per_number_invariant(
+) {
+    let pool = pool_with_inode_three_unlinked_and_the_root_ring_turned_over(
+        "deletion-checker-tombstone-on-the-lineage-second-birth",
+    );
+    let object_birth_of_inode_two = object_birth_of(&pool.output, 2).expect("2 号活着").0;
+    let clean = pool.memory_pool();
+    assert_the_base_image_is_clean(&clean, "3 号删完、根环转过一圈之后");
+    let mut image = clean.clone();
+    change_the_first_tombstone_of_the_newest_root(&mut image, |tombstone| {
+        assert_eq!(tombstone_field(tombstone, 0), 3, "那条墓碑的对象 ID 是 3");
+        assert_ne!(
+            tombstone_field(tombstone, TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET),
+            object_birth_of_inode_two,
+            "2、3 号分两次建，对象出生代不同"
+        );
+        set_tombstone_field(tombstone, 0, 2);
+    });
+    assert_eq!(
+        violated_invariants_in_name_order(&image),
+        vec!["I-9.10", ONE_OBJECT_BIRTH_PER_INODE_NUMBER],
+        "2 号的记录与谱系上的墓碑两个出生代：红 I-9.29 与 I-9.10 墓碑那一个等号"
+    );
+    let detail = first_violation_of(&image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER);
+    assert!(
+        detail.contains("inode 2 在回退候选集里对应不止一个对象出生代"),
+        "违例说明点名两个出生代的那个号：{detail}"
+    );
+}
+
+/// M3（覆盖缺口那一行：I-9.20「死亡 txg ≤ 这一版的 txg」上界没有用例钉）：3 号删完、根环转过一圈之后，那条墓碑的死亡 txg 改成最新根的 txg + 1。
+/// 它晚于每一版的 txg、不在任何一版的谱系上：I-9.18（队列空）、I-9.19、I-9.29 都不看它，只红 I-9.20。
+#[test]
+fn a_tombstone_that_died_after_every_version_reddens_only_the_tombstone_record_invariant() {
+    let pool = pool_with_inode_three_unlinked_and_the_root_ring_turned_over(
+        "deletion-checker-tombstone-dies-after-every-version",
+    );
+    let after_the_newest_txg = pool.output.root.checkpoint_txg.0 + 1;
+    let clean = pool.memory_pool();
+    assert_the_base_image_is_clean(&clean, "3 号删完、根环转过一圈之后");
+    let mut image = clean.clone();
+    change_the_first_tombstone_of_the_newest_root(&mut image, |tombstone| {
+        set_tombstone_field(
+            tombstone,
+            TOMBSTONE_RECORD_DEATH_TXG_OFFSET,
+            after_the_newest_txg,
+        );
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
+        detail.contains(&format!("死亡 txg {after_the_newest_txg} 不在")),
+        "违例说明写出那条墓碑的死亡 txg：{detail}"
+    );
+}
+
+/// M4（覆盖缺口那一行：I-9.4 第四句「记录号沿记录序严格递增」只钉了同号两条、倒序没有钉）：一次建两个空 inode（叶 [1, 2, 3]），
+/// 后两条的号对调成 [1, 3, 2]（两条都是同一次建的空 inode，除了号逐字节相同）。只红 I-9.4。
+#[test]
+fn a_leaf_holding_two_inode_numbers_in_descending_order_reddens_only_the_container_number_invariant(
+) {
+    let clean = pool_with_new_inodes("deletion-checker-leaf-in-descending-order", 2).memory_pool();
+    assert_the_base_image_is_clean(&clean, "一次建两个空 inode 之后");
+    let mut image = clean.clone();
+    change_the_inode_record(&mut image, 3, |record| {
+        record[..8].copy_from_slice(&2u64.to_le_bytes())
+    });
+    change_the_inode_record(&mut image, 2, |record| {
+        record[..8].copy_from_slice(&3u64.to_le_bytes())
+    });
+    assert_eq!(violated_invariants(&image), vec!["I-9.4"], "只红 I-9.4");
+    let detail = first_violation_of(&image, "I-9.4");
+    assert!(
+        detail.contains("inode 3 之后是 inode 2"),
+        "违例说明写出倒序的那一对：{detail}"
+    );
+}
+
+/// M6（覆盖缺口那一行：I-9.18 配墓碑时「N 的 inode 记录偏移 8 的对象出生代」没有用例钉）：空 inode 2 发完意图之后，那条墓碑的对象出生代减一。
+/// 队列里的 2 号没有 (2, 它记录的出生代) 的墓碑：I-9.18 红；同号两个出生代、墓碑那一个等号不符，I-9.29、I-9.10 同样红。
+#[test]
+fn a_queued_inode_whose_tombstone_names_another_object_birth_reddens_the_queued_tombstone_invariant(
+) {
+    let clean = image_after_the_intent_of_an_empty_inode("deletion-checker-queued-tombstone-birth");
+    assert_the_base_image_is_clean(&clean, "空 inode 2 发完意图之后");
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
+    assert_eq!(
+        violated_invariants_in_name_order(&image),
+        vec![
+            "I-9.10",
+            QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
+            ONE_OBJECT_BIRTH_PER_INODE_NUMBER
+        ],
+        "墓碑的出生代不是 2 号记录的：红 I-9.18，连带 I-9.29 与 I-9.10 墓碑那一个等号"
+    );
+    let detail = first_violation_of(&image, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE);
+    assert!(
+        detail.contains("deleted_inodes 里的号 2"),
+        "违例说明点名队列里那个号：{detail}"
+    );
+}
+
+/// M7（覆盖缺口那一行：I-9.18 配墓碑时「[0, 2⁶⁴ − 1]」整对象区间没有用例钉）：空 inode 2 发完意图之后，那条墓碑的区间止改成 5。
+/// 队列里的 2 号没有整对象墓碑：I-9.18 红；第一版只许整对象，I-9.20 同样红。
+#[test]
+fn a_queued_inode_whose_tombstone_does_not_cover_the_whole_object_reddens_the_queued_tombstone_invariant(
+) {
+    let clean = image_after_the_intent_of_an_empty_inode("deletion-checker-queued-tombstone-range");
+    assert_the_base_image_is_clean(&clean, "空 inode 2 发完意图之后");
+    let mut image = clean.clone();
+    change_the_first_tombstone_of_the_newest_root(&mut image, |tombstone| {
+        set_tombstone_field(tombstone, TOMBSTONE_RECORD_RANGE_END_OFFSET, 5);
+    });
+    assert_eq!(
+        violated_invariants_in_name_order(&image),
+        vec![
+            QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
+            TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED
+        ],
+        "墓碑不是整对象：红 I-9.18，连带 I-9.20"
+    );
+    let detail = first_violation_of(&image, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE);
+    assert!(
+        detail.contains("没有在谱系上的整对象墓碑"),
+        "违例说明写出没有整对象墓碑：{detail}"
+    );
+}
+
+/// 码 2 节点头里条目数的偏移（82 + 2 × key 宽）、头里最小 key 的偏移（52）与声明长度（偏移 8，2 字节），D18（块里携带什么信息） 已定项 18。
+fn index_node_entry_count_offset(key_width: usize) -> usize {
+    82 + 2 * key_width
+}
+const UNIT_DECLARED_LENGTH_OFFSET: usize = 8;
+const DELETED_INODES_ENTRY_BYTES: usize = 8;
+
+/// 队列同号两条（判决 `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K2 N7 那一行「同号两条归哪一条管，进下一批查清」）：
+/// 空 inode 2 发完意图之后，`deleted_inodes` 根（码 2、层级 0、key 宽 8、条目宽 8）{2} 末尾再追加一条 2（条目数 1 → 2、声明长度 8 → 16，
+/// 头里的 key 区间 [2, 2] 不变）。池级 checker 读队列时按号收成集合，I-9.24（删除队列至多一条 key） 看到的是一个号、照旧成立；
+/// 管它的是 I-1.1（码 2 节点的条目按 key 严格递增）。只红 I-1.1。
+#[test]
+fn a_deletion_queue_holding_one_number_twice_reddens_only_the_key_order_invariant() {
+    let clean = image_after_the_intent_of_an_empty_inode("deletion-checker-queue-one-number-twice");
+    assert_the_base_image_is_clean(&clean, "空 inode 2 发完意图之后");
+    let mut image = clean.clone();
+    let queue_slot =
+        root_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_DELETED_INODES);
+    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
+        &mut image,
+        queue_slot,
+        node_bytes(),
+        |root| {
+            let key_width = usize::from(root[51]);
+            assert_eq!(
+                key_width, DELETED_INODES_ENTRY_BYTES,
+                "deleted_inodes 的 key 宽 8"
+            );
+            let count_offset = index_node_entry_count_offset(key_width);
+            let count = usize::from(u16::from_le_bytes([
+                root[count_offset],
+                root[count_offset + 1],
+            ]));
+            assert_eq!(count, 1, "队列 {{2}} 一条");
+            let first = 115 + 2 * key_width;
+            let queued = root[first..first + DELETED_INODES_ENTRY_BYTES].to_vec();
+            assert_eq!(queued, 2u64.to_le_bytes(), "队列里是 2 号");
+            let second = first + DELETED_INODES_ENTRY_BYTES;
+            root[second..second + DELETED_INODES_ENTRY_BYTES].copy_from_slice(&queued);
+            root[count_offset..count_offset + 2].copy_from_slice(&2u16.to_le_bytes());
+            root[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
+                &u16::try_from(2 * DELETED_INODES_ENTRY_BYTES)
+                    .expect("16")
+                    .to_le_bytes(),
+            );
+        },
+    );
+    assert_eq!(
+        violated_invariants(&image),
+        vec!["I-1.1"],
+        "队列同号两条：只红 I-1.1（I-9.24 按号收成一个）"
+    );
+    assert_eq!(
+        verdict_of(&image, DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY),
+        InvariantVerdict::Holds,
+        "I-9.24 看到的是一个号：成立"
+    );
+}
diff --git a/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs b/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs
index b2f2226..5048c6f 100644
--- a/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs
+++ b/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs
@@ -12,8 +12,19 @@ use common::{
     abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, parameters,
     BuiltPool, FIXED_WRITE_TIME_SECONDS,
 };
-use singlefs_checker::image::{InvariantVerdict, INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION};
-use singlefs_checker::walk::check_pool_image;
+use singlefs_checker::image::{
+    InvariantVerdict, INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION,
+    NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+    QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
+    TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
+    TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
+    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+};
+use singlefs_checker::walk::{
+    check_pool_image, NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER, NO_QUEUED_INODE_IN_ANY_WALKED_VERSION,
+    NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY, NO_TOMBSTONE_IN_ANY_WALKED_VERSION,
+    NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD, NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION,
+};
 use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
 };
@@ -34,6 +45,34 @@ use singlefs_core::system_configuration::SystemConfiguration;
 use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
 use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
 
+/// 没有删除的会话上，删除线那几条各自报的不适用理由（池级 checker `walk` 里为每条写的那一句）：不变量 → 只认的那一句。
+const DELETION_INVARIANTS_WITHOUT_AN_OBJECT_ON_A_SESSION_WITHOUT_DELETION: [(&str, &str); 6] = [
+    (
+        QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
+        NO_QUEUED_INODE_IN_ANY_WALKED_VERSION,
+    ),
+    (
+        TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
+        NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD,
+    ),
+    (
+        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
+        NO_TOMBSTONE_IN_ANY_WALKED_VERSION,
+    ),
+    (
+        NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+        NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER,
+    ),
+    (
+        TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
+        NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY,
+    ),
+    (
+        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION,
+    ),
+];
+
 fn content_of(length: usize, seed: usize) -> Vec<u8> {
     (0..length)
         .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
@@ -325,6 +364,22 @@ fn raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slo
             );
             continue;
         }
+        if let Some((_, reason)) =
+            DELETION_INVARIANTS_WITHOUT_AN_OBJECT_ON_A_SESSION_WITHOUT_DELETION
+                .iter()
+                .find(|(exempted, _)| exempted == invariant)
+        {
+            // 删除线那几条（I-9.18–I-9.23）在这条没有删除的会话上没有对象：每一版的队列都空、墓碑树都是零根，报不适用；
+            // 每条只认池级 checker 为它写的那一句理由，别的不适用（含缺省理由「这个镜像上没有它判的对象」）照红
+            // （判别力在 `checker_known_bad_images.rs` 与 `pool_checker_judges_the_deletion_state_of_every_walked_version.rs`）。
+            // 回退 D 写出了谱系树，I-9.17 在 E 之后的镜像上真被评估过、要成立；I-9.24（空队列报成立）与 I-9.29 同样要成立。
+            assert_eq!(
+                *verdict,
+                InvariantVerdict::NotApplicable(reason),
+                "{invariant} 在 E 之后的镜像上报「{reason}」的不适用"
+            );
+            continue;
+        }
         assert_eq!(
             *verdict,
             InvariantVerdict::Holds,
````

## 实现员 k：mutations-append.tsv（/tmp/claude-1000/m3-batch24/k/patch/mutations-append.tsv）

````text
第24批 K N4：容器树条目不认孩子头里的类（类型段 0 指码 3 照旧按 16 KiB 读、连带走读失败）	crates/singlefs-checker/src/walk.rs	            if let Some(class_the_entry_type_names) = class_the_entry_type_names {	            if let Some(class_the_entry_type_names) = class_the_entry_type_names.filter(|_| false) {	-p singlefs-harness --test checker_known_bad_images -- a_tombstone_tree_entry_of_type_zero_over_a_container_reddens_only_the_tombstone_entry_invariant	a_tombstone_tree_entry_of_type_zero_over_a_container_reddens_only_the_tombstone_entry_invariant
第24批 K N4：容器树类型段 0 那一支不认孩子头里的类（身份三段为 0 的类型段 0 条目指码 3，I-9.23 报成立）	crates/singlefs-checker/src/walk.rs	                ContainerTreeEntryType::InternalNode => Some(MappingKeyUnitClass::IndexNode),	                ContainerTreeEntryType::InternalNode => None,	-p singlefs-harness --test checker_known_bad_images -- a_tombstone_tree_entry_of_type_zero_with_a_zero_identity_over_a_container_reddens_only_the_tombstone_entry_invariant	a_tombstone_tree_entry_of_type_zero_with_a_zero_identity_over_a_container_reddens_only_the_tombstone_entry_invariant
第24批 K N4：容器树类型段 0 那一支不认孩子头里的类（谱系树那一侧，I-9.17 连带走读失败）	crates/singlefs-checker/src/walk.rs	                ContainerTreeEntryType::InternalNode => Some(MappingKeyUnitClass::IndexNode),	                ContainerTreeEntryType::InternalNode => None,	-p singlefs-harness --test checker_known_bad_images -- a_lineage_tree_entry_of_type_zero_over_a_container_reddens_only_the_lineage_entry_invariant	a_lineage_tree_entry_of_type_zero_over_a_container_reddens_only_the_lineage_entry_invariant
第24批 K N4：容器树类型段 = 打包记录类型那一支不认孩子头里的类（类型段 1 指码 2 照旧按 32 KiB 读，I-2.1 等连带红）	crates/singlefs-checker/src/walk.rs	                    ContainerTreeEntryType::Container => {\n                        Some(MappingKeyUnitClass::PackedRecordUnit)\n                    }	                    ContainerTreeEntryType::Container => {\n                        None\n                    }	-p singlefs-harness --test checker_known_bad_images -- a_tombstone_tree_entry_of_the_container_type_over_a_node_reddens_only_the_tombstone_entry_invariant	a_tombstone_tree_entry_of_the_container_type_over_a_node_reddens_only_the_tombstone_entry_invariant
第24批 K N4：容器树类型段 = 打包记录类型那一支不认孩子头里的类（根环转过一圈之后那一形）	crates/singlefs-checker/src/walk.rs	                    ContainerTreeEntryType::Container => {\n                        Some(MappingKeyUnitClass::PackedRecordUnit)\n                    }	                    ContainerTreeEntryType::Container => {\n                        None\n                    }	-p singlefs-harness --test checker_known_bad_images -- a_tombstone_tree_entry_of_the_container_type_over_a_node_after_the_root_ring_turned_over_reddens_only_the_tombstone_entry_invariant	a_tombstone_tree_entry_of_the_container_type_over_a_node_after_the_root_ring_turned_over_reddens_only_the_tombstone_entry_invariant
第24批 K I-9.2 同形：inode 树类型段 0 那一支当孩子该是码 3（指码 3 叶容器照旧按 16 KiB 读、I-2.1 红）	crates/singlefs-checker/src/walk.rs	                InodeInternalEntryType::InternalNode => Some(MappingKeyUnitClass::IndexNode),	                InodeInternalEntryType::InternalNode => Some(MappingKeyUnitClass::PackedRecordUnit),	-p singlefs-harness --test checker_known_bad_images -- an_inode_tree_entry_of_type_zero_over_a_leaf_container_reddens_only_the_inode_entry_invariant	an_inode_tree_entry_of_type_zero_over_a_leaf_container_reddens_only_the_inode_entry_invariant
第24批 K I-9.2 同形：孩子头里的类与类型段说的不符也当相符（身份三段为 0 的 inode 条目指码 3 叶容器）	crates/singlefs-checker/src/walk.rs	        if Some(class_in_the_header) == class_the_entry_type_names {	        if Some(class_in_the_header) != class_the_entry_type_names {	-p singlefs-harness --test checker_known_bad_images -- an_inode_tree_entry_of_type_zero_with_a_zero_identity_over_a_leaf_container_reddens_only_the_inode_entry_invariant	an_inode_tree_entry_of_type_zero_with_a_zero_identity_over_a_leaf_container_reddens_only_the_inode_entry_invariant
第24批 K 孩子的类不验整单元校验和（只改了类标签的叶容器被当成码 2 孩子、按一槽登记）	crates/singlefs-checker/src/walk.rs	            .is_some_and(|content| crc32_castagnoli_table(&content) == location.checksum);	            .is_some();	-p singlefs-harness --test checker_known_bad_images -- an_inode_leaf_whose_class_tag_alone_says_index_node_is_read_at_the_length_of_its_entry_type	an_inode_leaf_whose_class_tag_alone_says_index_node_is_read_at_the_length_of_its_entry_type
第24批 K M1：容器树类型段 0 那一支身份判定恒真	crates/singlefs-checker/src/walk.rs	                        && child_class.is_none_or(|class| class == 2);	                        && child_class.is_none_or(|class| class == 2) || true;	-p singlefs-harness --test checker_known_bad_images -- a_tombstone_tree_entry_of_type_zero_with_a_container_number_reddens_only_the_tombstone_entry_invariant	a_tombstone_tree_entry_of_type_zero_with_a_container_number_reddens_only_the_tombstone_entry_invariant
第24批 K M1：谱系树类型段不认得那一支不判红	crates/singlefs-checker/src/walk.rs	                    self.judgements.judge(identity_invariant, false, || {\n                        format!("{name}内部条目的类型段 {unregistered_type} 不在 {{0, {record_type}}} 里")	                    self.judgements.judge(identity_invariant, true, || {\n                        format!("{name}内部条目的类型段 {unregistered_type} 不在 {{0, {record_type}}} 里")	-p singlefs-harness --test checker_known_bad_images -- a_lineage_tree_entry_with_an_unregistered_type_reddens_only_the_lineage_entry_invariant	a_lineage_tree_entry_with_an_unregistered_type_reddens_only_the_lineage_entry_invariant
第24批 K M5：墓碑容器号沿叶序严格递增那一句恒真	crates/singlefs-checker/src/walk.rs	        judgements.judge(invariant, left.container < right.container, || {	        judgements.judge(invariant, true || left.container < right.container, || {	-p singlefs-harness --test checker_known_bad_images -- two_tombstone_containers_with_one_container_number_redden_only_the_capacity_knob_invariant	two_tombstone_containers_with_one_container_number_redden_only_the_capacity_knob_invariant
第24批 K M2：I-9.29 不收谱系上墓碑的出生代（墓碑一侧记到别的号下）	crates/singlefs-checker/src/walk.rs	                        .entry((tombstone_tree.head_identifier, tombstone.object))	                        .entry((tombstone_tree.head_identifier, u64::MAX))	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version -- a_tombstone_on_the_lineage_carrying_another_object_birth_for_a_live_number_reddens_the_one_birth_per_number_invariant	a_tombstone_on_the_lineage_carrying_another_object_birth_for_a_live_number_reddens_the_one_birth_per_number_invariant
第24批 K M3：I-9.20 不比死亡 txg 的上界	crates/singlefs-checker/src/walk.rs	                tombstone.object_birth <= tombstone.death_txg && tombstone.death_txg <= version_txg;	                tombstone.object_birth <= tombstone.death_txg;	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version -- a_tombstone_that_died_after_every_version_reddens_only_the_tombstone_record_invariant	a_tombstone_that_died_after_every_version_reddens_only_the_tombstone_record_invariant
第24批 K M4：I-9.4 第四句只要求前后不同（倒序放过）	crates/singlefs-checker/src/walk.rs	judge("I-9.4", inode > previous_inode	judge("I-9.4", inode != previous_inode	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version -- a_leaf_holding_two_inode_numbers_in_descending_order_reddens_only_the_container_number_invariant	a_leaf_holding_two_inode_numbers_in_descending_order_reddens_only_the_container_number_invariant
第24批 K M6：I-9.18 配墓碑时不比对象出生代	crates/singlefs-checker/src/walk.rs	                tombstone.object_birth == record.object_birth\n                    && tombstone.covers_the_whole_object()	                tombstone.covers_the_whole_object()	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version -- a_queued_inode_whose_tombstone_names_another_object_birth_reddens_the_queued_tombstone_invariant	a_queued_inode_whose_tombstone_names_another_object_birth_reddens_the_queued_tombstone_invariant
第24批 K M7：I-9.18 配墓碑时不比整对象区间	crates/singlefs-checker/src/walk.rs	                tombstone.object_birth == record.object_birth\n                    && tombstone.covers_the_whole_object()	                tombstone.object_birth == record.object_birth	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version -- a_queued_inode_whose_tombstone_does_not_cover_the_whole_object_reddens_the_queued_tombstone_invariant	a_queued_inode_whose_tombstone_does_not_cover_the_whole_object_reddens_the_queued_tombstone_invariant
第24批 K 队列同号两条：码 2 节点条目 key 只要求不减（同 key 两条放过）	crates/singlefs-checker/src/lib.rs	    let key_of = |entry: &Vec<u8>| schema.fields(&entry[..view.key_width]);\n    for pair in view.entries.windows(2) {\n        if key_of(&pair[0]) >= key_of(&pair[1]) {\n            return Err(Verdict::KeysNotStrictlyAscending);\n        }\n    }\n    if let	    let key_of = |entry: &Vec<u8>| schema.fields(&entry[..view.key_width]);\n    for pair in view.entries.windows(2) {\n        if key_of(&pair[0]) > key_of(&pair[1]) {\n            return Err(Verdict::KeysNotStrictlyAscending);\n        }\n    }\n    if let	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version -- a_deletion_queue_holding_one_number_twice_reddens_only_the_key_order_invariant	a_deletion_queue_holding_one_number_twice_reddens_only_the_key_order_invariant
第24批 K E142 导出：空闲图按算术判时上界不起作用（比上界多一个槽照样放过）	crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs	    if free_map_bytes_of_the_two_devices > FREE_MAP_BYTES_OF_THE_TWO_DEVICES_AT_MOST {	    if free_map_bytes_of_the_two_devices > u64::MAX {	-p singlefs-checker-tier --bin e142_new_pool_file_creation_write_dump -- device_bytes_at_the_free_map_bound	device_bytes_at_the_free_map_bound_pass_and_one_slot_more_is_refused
第24批 K E142 导出：恰好在上界上的也拒（上界那一格差一）	crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs	    if free_map_bytes_of_the_two_devices > FREE_MAP_BYTES_OF_THE_TWO_DEVICES_AT_MOST {	    if free_map_bytes_of_the_two_devices >= FREE_MAP_BYTES_OF_THE_TWO_DEVICES_AT_MOST {	-p singlefs-checker-tier --bin e142_new_pool_file_creation_write_dump -- device_bytes_at_the_free_map_bound	device_bytes_at_the_free_map_bound_pass_and_one_slot_more_is_refused
第24批 K 新九条的不适用理由：I-9.23 没有内部条目时不写专门理由（落到缺省理由「这个镜像上没有它判的对象」）	crates/singlefs-checker/src/walk.rs	    judgements.not_applicable(\n        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,\n        NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION,\n    );	    let _ = NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION;	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
第24批 K 新九条的不适用理由：I-9.18 队列空时不写专门理由（落到缺省理由「这个镜像上没有它判的对象」）	crates/singlefs-checker/src/walk.rs	    judgements.not_applicable(\n        QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,\n        NO_QUEUED_INODE_IN_ANY_WALKED_VERSION,\n    );	    let _ = NO_QUEUED_INODE_IN_ANY_WALKED_VERSION;	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
````

## 实现员 k：mutations-delete.txt（/tmp/claude-1000/m3-batch24/k/patch/mutations-delete.txt）

````text
第23批 X E142 导出：建盘之前试分配空闲图只要 0 字节（每盘 2⁶⁴ − 1 字节放过去、建分配器那一步 abort 退 134）
````

## 实现员 g：crates.patch（/tmp/claude-1000/m3-batch24/g/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index f0f05f3..307b354 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -2545,7 +2545,7 @@ fn w_walk_central_mapping_entries(entries: u32, fsid_low: vec2<u32>) {
     }
 }
 
-// ── inode 树（walk_inode_root / walk_inode_internal_node / walk_inode_leaf_container）：根层级 1，孩子只接叶容器；类型段 0 的孩子这一版判不了
+// ── inode 树（walk_inode_root / walk_inode_internal_node / walk_inode_leaf_container）：根层级 1，孩子只接叶容器；类型段 0 而孩子不是别的一类（w_child_unit_class_other_than_the_entry_type_names）的这一版判不了
 fn w_cached_leaves_below(device: u32, slot: vec2<u32>, into_from: u32) -> u32 {
     // 把父单元 (device, slot) 下面缓存的叶容器接进叶序（从 into_from 起追加），交回追加了几条
     var appended = 0u;
@@ -2603,6 +2603,21 @@ fn w_walk_inode_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: v
             length = DATA_UNIT_BYTES;
             span = 2u;
         }
+        // I-9.2 判孩子的类按孩子头里的类标签认、不按类型段定读多长（同墓碑树、谱系树那两条，walk.rs walk_inode_internal_node；判决
+        // `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 N4 那一行「inode 树 I-9.2 同形」）：孩子自证过的类与类型段说的不符
+        // （类型段 2 ⇒ 码 3、类型段 0 ⇒ 码 2；类型段不认得的，孩子读得出就算），判 I-9.2 红，按它自己那一类占的槽登记引用，不往下走、不记走读失败
+        var class_the_inode_entry_type_names = NONE;
+        if (record_type == 2u) {
+            class_the_inode_entry_type_names = 3u;
+        } else if (record_type == 0u) {
+            class_the_inode_entry_type_names = 2u;
+        }
+        let class_of_the_inode_child = w_child_unit_class_other_than_the_entry_type_names(child, class_the_inode_entry_type_names);
+        if (class_of_the_inode_child != NONE) {
+            w_note_pointer_references(child, w_slots_of_a_unit_of_class(class_of_the_inode_child));
+            w_judge(INVARIANT_I_9_2, class_of_the_inode_child == class_the_inode_entry_type_names);
+            continue;
+        }
         w_note_pointer_references(child, span);
         let s = w_read_referenced_unit(child, length);
         if (s.version == NONE) {
@@ -2894,6 +2909,10 @@ fn w_walk_tree_table_entry(tt: Node, e: u32, fsid_low: vec2<u32>) {
 // 别的类型段先登记引用（层级 1 的节点按码 3 容器两槽、层级 ≥ 2 的按码 2 节点一槽）再判该条目损坏、不往下走。每条条目的身份单独判定
 // （墓碑树 I-9.23、谱系树 I-9.17），身份不符不记走读失败（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T7 取 a）：
 // 类型段 0 判身份引用三段为 0 且孩子（读得出时）是码 2；类型段 = record_type 判容器头四段与身份引用逐字相等、子指针出生树相等、类标签 3。
+// 孩子是哪一类按它头里的类标签认、不按类型段定读多长（w_child_unit_class_other_than_the_entry_type_names，判决
+// `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 N4 那一行：类型段 0 指着码 3 容器、类型段 = record_type 指着码 2 节点时，
+// 按类型段的长度读回的整单元校验和对不上，身份那一判跑不到、却记了 I-2.1 与走读失败）：头里是另一类、按那一类的长度自证得过的，
+// 判身份红、按它自己那一类占的槽登记引用（码 2 一槽、码 1 / 码 3 两槽）、不往下走、不记走读失败；别的照旧按类型段走。
 // 往下记的落点都记在正在走的那个码 2 节点下面（w_parent_*）。walk.rs 是递归，这里是同一个先序的显式栈，
 // 深度超过 w_container_tree_frames 的 12 层判不了；两个种类只经这一处调用点
 struct WalkContainerTreeFrame {
@@ -2905,6 +2924,53 @@ struct WalkContainerTreeFrame {
 
 var<private> w_container_tree_frames: array<WalkContainerTreeFrame, 12>;
 
+// 一份落点上读回的单元按 unit_class 那一类的长度（码 2 16 KiB，码 1 / 码 3 32 KiB）算的整单元校验和对得上 checksum；别的类没有长度，恒不成立
+fn w_unit_verifies_at_the_length_of_its_class(s: Slice, unit_class: u32, checksum: u32) -> bool {
+    if (unit_class == 2u) {
+        return slice_crc_matches(s, NODE_BYTES, checksum);
+    }
+    if (unit_class == 1u || unit_class == 3u) {
+        return slice_crc_matches(s, DATA_UNIT_BYTES, checksum);
+    }
+    return false;
+}
+
+// 树里一条内部条目的孩子是不是条目类型段说的那一类（walk.rs 的 child_unit_class_other_than_the_entry_type_names）：只读孩子的单元头取类标签，
+// 不按类型段定读多长。两条位置条目依次看：头按最短的单元（码 2，16 KiB）那么长读，头里的类标签就是 class_the_entry_type_names 的交回 NONE
+// （照类型段那一支走，读不出照旧归 I-2.1）；是另一类登记过的码、且按那一类的长度读回的整单元校验和对得上，交回那一类；
+// 读不出、类标签不在登记表里、校验和对不上的看下一条位置条目。class_the_entry_type_names 是 NONE（类型段不认得）时交回孩子自证过的那一类
+fn w_child_unit_class_other_than_the_entry_type_names(child: Ptr, class_the_entry_type_names: u32) -> u32 {
+    let first_copy = read_at(child.device0, slot_to_bytes(child.slot0), NODE_BYTES);
+    if (first_copy.version != NONE) {
+        let class_in_the_first_header = rd8(first_copy.base, 6u);
+        if (class_in_the_first_header == class_the_entry_type_names) {
+            return NONE;
+        }
+        if (w_unit_verifies_at_the_length_of_its_class(first_copy, class_in_the_first_header, child.sum0)) {
+            return class_in_the_first_header;
+        }
+    }
+    let second_copy = read_at(child.device1, slot_to_bytes(child.slot1), NODE_BYTES);
+    if (second_copy.version != NONE) {
+        let class_in_the_second_header = rd8(second_copy.base, 6u);
+        if (class_in_the_second_header == class_the_entry_type_names) {
+            return NONE;
+        }
+        if (w_unit_verifies_at_the_length_of_its_class(second_copy, class_in_the_second_header, child.sum1)) {
+            return class_in_the_second_header;
+        }
+    }
+    return NONE;
+}
+
+// 一个单元按它的类占几槽：码 2 节点一槽，码 1 数据单元与码 3 打包记录单元两槽
+fn w_slots_of_a_unit_of_class(unit_class: u32) -> u32 {
+    if (unit_class == 2u) {
+        return 1u;
+    }
+    return 2u;
+}
+
 fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: vec2<u32>, record_type: u32, fsid_low: vec2<u32>) {
     var identity_invariant = INVARIANT_I_9_23;
     if (record_type == PACKED_TYPE_LINEAGE) {
@@ -2944,6 +3010,13 @@ fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, roo
         w_judge_pointer_head(child);
         w_judge_pointer_extent_offset(child);
         if (entry_type == 0u) {
+            // 孩子头里是码 2 之外的一类、按那一类的长度自证得过：身份不符，按它自己那一类占的槽登记引用，不往下走、不记走读失败
+            let class_of_the_child_of_an_internal_entry = w_child_unit_class_other_than_the_entry_type_names(child, 2u);
+            if (class_of_the_child_of_an_internal_entry != NONE) {
+                w_note_pointer_references(child, w_slots_of_a_unit_of_class(class_of_the_child_of_an_internal_entry));
+                w_judge(identity_invariant, class_of_the_child_of_an_internal_entry == 2u);
+                continue;
+            }
             // 类型段 0：身份引用的出生树、容器号、出生代三段为 0；「子单元是码 2」与 I-1.6 重合、照 I-9.2 的先例留作合取，读不出的孩子这一小句不判
             let child_unit = read_unit_via_locations(child, NODE_BYTES);
             let child_is_a_code_two_node_or_unreadable = child_unit.version == NONE || rd8(child_unit.base, 6u) == 2u;
@@ -2983,6 +3056,13 @@ fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, roo
             w_judge(identity_invariant, false);
             continue;
         }
+        // 类型段 = record_type：孩子头里是码 3 之外的一类、按那一类的长度自证得过，身份不符（「类标签 3」那一小句），不按数据单元长度去读、不往下走
+        let class_of_the_child_of_a_container_entry = w_child_unit_class_other_than_the_entry_type_names(child, 3u);
+        if (class_of_the_child_of_a_container_entry != NONE) {
+            w_note_pointer_references(child, w_slots_of_a_unit_of_class(class_of_the_child_of_a_container_entry));
+            w_judge(identity_invariant, class_of_the_child_of_a_container_entry == 3u);
+            continue;
+        }
         w_note_pointer_references(child, 2u);
         let s = w_read_referenced_unit(child, DATA_UNIT_BYTES);
         if (s.version == NONE) {
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
index d930706..3c01dd1 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -2584,7 +2584,7 @@ fn judge_inode_number_watermark_above_every_published_tombstone() {
 
 // ───────────────────────── 墓碑与谱系那一遍（walk.rs judge_tombstones_and_the_lineage_of_the_versions）─────────────────────────
 // 删除态扫描那一份版本清单里每一版重读树表、谱系树、墓碑树、inode 树与 deleted_inodes（只读，不进 w_failures）：判 I-9.18、I-9.19、I-9.20、
-// I-9.21 与 I-9.10 墓碑那一个等号，把各版的 (inode 树, 号) → 对象出生代收进跨版本那张表（D_OBJECT_BIRTHS），同一个 (树, 号) 见到第二个出生代就判 I-9.29。
+// I-9.21 与 I-9.10 墓碑那一个等号，把各版的 (inode 树, 号) → 对象出生代收进跨版本那张表（D_OBJECT_BIRTHS），同一个 (树, 号) 见到第二个出生代就判 I-9.29（最新根的实例表读不出时不比）。
 // CPU 记成射程里判不了的那几格（树表读不出、条目宽不到 200、谱系树条目不是恰好一条、谱系树或墓碑树读不全、谱系接不成一条链、头 ID 配不上恰好一棵
 // inode 树或恰好一棵墓碑树、队列根读不出、队列里的号没有记录、inode 树读不全）在这里一格都不置位；CPU 按 (树表单元, txg) 去重只扫一次，
 // 违例位是并集，这里不去重。I-9.21 只按没装旋钮那一支判（层 0 与放量走池级 checker 的旧入口），I-9.22 不置位。不按已发布与否滤墓碑（第二节 T4）。
@@ -2596,6 +2596,12 @@ var<private> d_lineage_record_count: u32;
 
 var<private> d_object_birth_count: u32;
 
+// I-9.29 跨版本那张表比不比：最新根的实例表沿链整张读得出（r_instance_rows，与 walk.rs 的 instance_table_rows_of_root_without_judging 同一个读法）
+// 才比；链上有一片读不出、解不开时回退候选集滤不掉被崩溃恢复抛弃的根，被抛弃的线上同号可以是另一个对象，CPU 报部分没判
+// （walk.rs 的 AbandonedRootsFilteringOfTheCandidateSet；`.claude/kb/invariants.md` I-9.29 那一行「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」），
+// 这里一个号都不记、不置位。不记也就不会撞上那张表的上限：CPU 那一支一个号都不比，这里没有判不了的理由
+var<private> d_object_births_are_compared: bool;
+
 // I-9.20 不重号那一句两两比：一棵墓碑树的墓碑多于这么多条就判不了（比较次数随条数平方涨）
 const DELETION_MAX_TOMBSTONES_COMPARED_PAIRWISE: u32 = 4096u;
 
@@ -2779,8 +2785,11 @@ fn d_txg_is_on_the_lineage(txg: vec2<u32>, version_txg: vec2<u32>) -> bool {
 }
 
 // I-9.29 跨版本那张表记一笔：(inode 树, 号) 第一次见到时记下对象出生代，再见到而出生代不同就判红（walk.rs 收齐之后每个号判一次
-// births.len() == 1，与这里见到第二个不同的出生代就红是同一判）。表满判不了
+// births.len() == 1，与这里见到第二个不同的出生代就红是同一判）。表满判不了。最新根的实例表读不出时整张表不比（d_object_births_are_compared）
 fn d_note_object_birth(inode_tree: vec2<u32>, inode: vec2<u32>, object_birth: vec2<u32>) {
+    if (!d_object_births_are_compared) {
+        return;
+    }
     for (var birth_index = 0u; birth_index < d_object_birth_count; birth_index = birth_index + 1u) {
         let row = D_OBJECT_BIRTHS + birth_index * D_OBJECT_BIRTH_WORDS;
         if (sw(row) == inode_tree.x && sw(row + 1u) == inode_tree.y && sw(row + 2u) == inode.x && sw(row + 3u) == inode.y) {
@@ -3050,9 +3059,10 @@ fn d_judge_tombstones_and_the_lineage_of_a_version(base: u32, tree_table_pointer
 }
 
 // 墓碑与谱系那一遍：删除态扫描那一份版本清单（回退候选集里每条根，与由记录施加出来的那几版），每一版从自己的 txg 与自己的谱系树起算；
-// 全部扫完时 I-9.29 那张表已经判完（d_note_object_birth 见到第二个出生代当场判）
+// 全部扫完时 I-9.29 那张表已经判完（d_note_object_birth 见到第二个出生代当场判）；最新根的实例表读不出时那张表不比
 fn judge_tombstones_and_the_lineage_of_the_versions() {
     d_object_birth_count = 0u;
+    d_object_births_are_compared = r_instance_rows(checker_root_of(c_newest_index).base);
     for (var candidate = 0u; candidate < c_candidate_count; candidate = candidate + 1u) {
         let root = checker_root_of(sw(C_CANDIDATES + candidate));
         d_judge_tombstones_and_the_lineage_of_a_version(root.base, 36u, root.txg);
diff --git a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
index 49ebf00..3459c7d 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
@@ -70,9 +70,12 @@ const _: () = assert!(
 /// `judge_deletion_state_of_the_versions`），73 的 I-4.4 第一版池级 checker 恒报不适用、内核不置位；I-9.6 墓碑那一半（水位 > 所选根墓碑树里
 /// 已发布墓碑的最大对象 ID）也在扫描内核里判、并进下标 41 那一位。对拍逐位比、不屏蔽。
 /// 下标 74–82 是里程碑三·3 墓碑与谱系那九条（I-9.17、I-9.18、I-9.19、I-9.20、I-9.21、I-9.22、I-9.23、I-9.24、I-9.29，CPU 第 22 批、GPU 第 23 批）：
-/// 走读内核在墓碑树、谱系树的每条内部条目上判 I-9.17、I-9.23（身份不符置位、不记走读失败）；扫描内核的删除态扫描读队列时判 I-9.24，
+/// 走读内核在墓碑树、谱系树的每条内部条目上判 I-9.17、I-9.23（身份不符置位、不记走读失败；孩子是哪一类按它头里的类标签认、不按类型段定读多长，
+/// 头里是另一类、按那一类的长度自证得过的按它自己那一类登记引用、不往下走，第 24 批跟 CPU 同改，inode 树内部条目的 I-9.2 同改）；
+/// 扫描内核的删除态扫描读队列时判 I-9.24，
 /// 墓碑与谱系那一遍在同一份版本清单上判 I-9.18、I-9.19、I-9.20、I-9.21（没装旋钮那一支）与 I-9.10 墓碑那一个等号（并进下标 43），
-/// I-9.29 收各版的号 → 对象出生代进一张跨版本的表判；一棵树的容器、一版的谱系记录、跨版本那张表、两两比不重号的墓碑总数越过上限的判不了。
+/// I-9.29 收各版的号 → 对象出生代进一张跨版本的表判，最新根的实例表沿链读不出时那张表不比、不置位（CPU 报部分没判，第 24 批）；
+/// 一棵树的容器、一版的谱系记录、跨版本那张表、两两比不重号的墓碑总数越过上限的判不了。
 /// 下标 79 的 I-9.22 照下标 73 的先例恒不置位：它只在池级 checker 带旋钮值的入口上判，层 0 与放量走旧入口（`check_pool_image`，恒报不适用），
 /// GPU 也没有旋钮值。I-9.4 第四句（同一片 inode 叶容器里记录号严格递增）走读内核一并判、并进下标 40。
 /// 用户 2026-10-04 定「改回要求全覆盖」：没有「内核还没判」那一张表。
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 63b584d..13dcb06 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -18,6 +18,10 @@
 //! （连同 I-9.10 墓碑那一个等号）各一条改坏墓碑或记录的流，I-9.24 并在队列 {1, 7} 那一条上，I-9.21 一条墓碑容器旋钮压到一片 1 条的流
 //! （旋钮下合法、CPU 判器按旧入口判就红），I-9.4 第四句一条叶里记录对调的流，删 → 回退 → 再删一条合法流钉九条不红；零条目的 extent 上段根
 //! 与 inode 树根条目宽窄于字段表各一条，恢复两边都答没有文件。I-9.22 层 0 恒不适用，没有流。
+//! 第 24 批：条目类型段与孩子的真实类不符的四形（攻方 N4-a、N4-i、N4-f、N4-j：层级 1 的墓碑树根条目类型段 0 指码 3 容器，身份三段带着或清零；
+//! 旋钮压出来的层级 2 墓碑树根条目类型段 1 指码 2 节点，根环转没转过一圈各一条）钉 CPU 红 I-9.23、不红走读失败连带的那几条，GPU 逐位跟上，
+//! inode 树根条目类型段 0 指码 3 叶容器（I-9.2 同形）一条同钉；
+//! I-9.29 判不了那一形（被崩溃恢复抛弃的根与现行线同号不同出生代、最新根的实例表两份翻坏）一条流整份比。
 //! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
 //! extent 下段叶的数据指针、分配记录树内部条目子指针上各一条坏流整份比（core 读这几条指针都判、GPU 恢复内核照判）；这几处除实例表指针外
 //! 另各有一条 MAC 第 0 字节非 0 的坏流（core 连加密、压缩那几段一并判，GPU 恢复内核同判）；extent 上段叶内联的数据指针另有一条算法类型写 1 的坏流
@@ -39,14 +43,19 @@ mod common_pointer_head_corruption;
 
 use std::time::Instant;
 
-use common::{build_pool, file_content, geometry, parameters, BuiltPool};
+use common::{
+    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, file_content,
+    geometry, parameters, BuiltPool,
+};
 use common_crash_points::{mount_writable_step, overwrite_step, record_pool_build, roll_back_step};
 use common_deletion_in_progress::{
     content_of_three_data_units, pool_with_a_three_unit_file, pool_with_new_inodes,
-    publish_the_intent, publish_the_next_step_in_batches_of_one, roll_back_to, unlink_to_the_end,
-    FIRST_FILE, INSTANCE_OF_THE_PUBLISHES, WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
+    publish_the_intent, publish_the_next_step_in_batches_of_one, roll_back_to,
+    turn_the_root_ring_over_by_creating_inodes, unlink_to_the_end, FIRST_FILE,
+    INSTANCE_OF_THE_PUBLISHES, WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
 };
 use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
+use singlefs_checker::walk::check_pool_image;
 use singlefs_checker_tier::crash::{
     judge_layer0_state_range, layer0_plan_state_count, Layer0OracleViolationKind,
     Layer0SegmentExpansion, Layer0StateJudgement,
@@ -71,7 +80,9 @@ use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
 use singlefs_checker_tier::gpu_unit_checks::GpuCard;
 use singlefs_checker_tier::verdict_store::{JudgeVersion, VerdictStore};
 use singlefs_core::address::{CheckpointTxg, InodeNumber, InstanceGeneration};
-use singlefs_core::container_tree::{ContainerTreeCapacities, ContainerTreeNodeEntries};
+use singlefs_core::container_tree::{
+    ContainerTreeCapacities, ContainerTreeNodeEntries, ContainerTreeNodeEntriesCappedForTests,
+};
 use singlefs_core::lineage_tree::LineageContainerCapacity;
 use singlefs_core::mount::RollbackTarget;
 use singlefs_core::tombstone_tree::{
@@ -5579,6 +5590,7 @@ fn the_gpu_judge_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inod
 /// 码 2 节点条目里身份引用的出生树（分隔 key 8 之后）与容器号（类型段 2 之后）。
 const CONTAINER_TREE_ENTRY_BIRTH_TREE_OFFSET: usize = 8;
 const CONTAINER_TREE_ENTRY_CONTAINER_NUMBER_OFFSET: usize = 18;
+const CONTAINER_TREE_ENTRY_CONTAINER_BIRTH_OFFSET: usize = 26;
 /// 墓碑树、谱系树条目类型段条款之外的一个值（墓碑 {0, 1}、谱系 {0, 5} 之外）。
 const UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE: u16 = 7;
 /// 墓碑记录 40 字节：对象 ID 0、对象出生代 8、死亡 txg 32（D18（块里携带什么信息） 已定项 24 第 2 条）。
@@ -5589,6 +5601,10 @@ const TOMBSTONE_RECORD_DEATH_TXG_OFFSET: usize = 32;
 const OBJECT_IDENTIFIER_WITHOUT_A_RECORD: u64 = 0;
 /// I-9.21 那条流里墓碑容器旋钮压到的每片记录数：两次 unlink 各占一片，最左那片不是最右的、只装 1 条。
 const TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;
+/// N4-f、N4-j 那两条流里容器树旋钮压到的每个码 2 节点条目数：三片容器装不进一个根，根长到层级 2。
+const CONTAINER_TREE_ENTRIES_PER_NODE_UNDER_THE_KNOB: usize = 2;
+/// 码 2 节点头里层级那一字节的偏移（D18（块里携带什么信息） 已定项 18：42 树 ID / 50 层级 / 51 key 宽）。
+const INDEX_NODE_LEVEL_OFFSET: usize = 50;
 
 /// 墓碑树、谱系树、inode 树的码 2 节点里第一条条目的起点（key 是 8 字节的容器号或 inode 号）。
 fn first_entry_of_a_container_number_node() -> usize {
@@ -5696,8 +5712,10 @@ fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type(
     )
 }
 
-/// I-9.23 类型段 0 那一支：删除中间态的流上墓碑树根（层级 1）第一条条目的类型段 1 → 0、重封到根。身份引用的出生树不是 0、孩子是码 3：
-/// CPU 判 I-9.23 红，再因为层级 1 的节点里出现类型段 0 记走读失败；GPU 走读内核同判同记。
+/// I-9.23 类型段 0 那一支（攻方 N4-a 那一形，`research/prompts/m3-batch22-code-r1-opus-output.md` 第一节）：删除中间态的流上墓碑树根（层级 1）
+/// 第一条条目的类型段 1 → 0、重封到根。身份引用的出生树不是 0、孩子是码 3：CPU 只读孩子的单元头得码 3、与类型段 0 不符，判 I-9.23 红、
+/// 按两槽登记引用、不往下走、不记走读失败（第 24 批；原来按节点长度读不出孩子，再因为层级 1 的节点里出现类型段 0 记走读失败，I-4.8、I-7.2、I-7.4 连带红）；
+/// GPU 走读内核同判同登记。
 fn record_deletion_with_the_tombstone_root_entry_of_type_zero(tag: &str) -> RecordedFlow {
     record_deletion_with_a_resealed_field(
         tag,
@@ -5718,6 +5736,62 @@ fn record_deletion_with_the_tombstone_root_entry_of_type_zero(tag: &str) -> Reco
     )
 }
 
+/// 容器树节点第一条条目的身份引用里出生树、容器号、容器出生代三段写 0（类型段那两字节不动）。
+fn clear_the_identity_of_the_first_entry_but_its_type(bytes: &mut [u8]) {
+    let first_entry = first_entry_of_a_container_number_node();
+    for field in [
+        CONTAINER_TREE_ENTRY_BIRTH_TREE_OFFSET,
+        CONTAINER_TREE_ENTRY_CONTAINER_NUMBER_OFFSET,
+        CONTAINER_TREE_ENTRY_CONTAINER_BIRTH_OFFSET,
+    ] {
+        bytes[first_entry + field..first_entry + field + 8].fill(0);
+    }
+}
+
+/// I-9.23 类型段 0 那一支、身份三段也清零（攻方 N4-i 那一形）：删除中间态的流上墓碑树根（层级 1）第一条条目的类型段 1 → 0、
+/// 出生树、容器号、容器出生代三段写 0、重封到根。身份引用那一半成立，只剩「子单元是码 2」那一小句不成立：孩子是码 3 容器。
+/// CPU 只读孩子的单元头得码 3，判 I-9.23 红、不往下走、不记走读失败（第 24 批；原来按节点长度读不出孩子、这一小句不判，I-9.23 报成立，
+/// 层级 1 的节点里出现类型段 0 记走读失败）；GPU 走读内核同判。
+fn record_deletion_with_the_tombstone_root_entry_of_type_zero_and_no_identity(
+    tag: &str,
+) -> RecordedFlow {
+    record_deletion_with_a_resealed_field(
+        tag,
+        "deletion-tombstone-root-entry-type-zero-no-identity",
+        &is_the_tombstone_tree_root,
+        &|bytes| {
+            set_the_type_of_the_first_entry(bytes, 0);
+            clear_the_identity_of_the_first_entry_but_its_type(bytes);
+        },
+    )
+}
+
+/// I-9.2 同形（判决 `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 N4 那一行「inode 树 I-9.2 同形一并查」）：
+/// 新池新建文件的流上 inode 树根（层级 1）第一条条目的类型段 2 → 0、重封到根。条目说它指下一层码 2 节点，孩子是码 3 叶容器：
+/// CPU 只读孩子的单元头得码 3、按 32 KiB 自证得过，判 I-9.2 红、按两槽登记引用、不往下走、不记走读失败（第 24 批；原来按节点长度读、
+/// 校验和对不上，I-2.1 红、记走读失败）；GPU 走读内核同判同登记（原来类型段 0 的孩子整个状态判不了）。
+fn record_new_pool_with_the_inode_root_entry_of_type_zero_naming_its_leaf_container(
+    tag: &str,
+) -> RecordedFlow {
+    let inode_tree_root_over_leaf_containers = |bytes: &[u8]| {
+        unit_class(bytes) == 2
+            && usize::from(index_node_key_width(bytes)) == INODE_TREE_KEY_BYTES
+            && index_node_entry_width(bytes) == INODE_INTERNAL_ENTRY_BYTES
+            && read_u16_at(
+                bytes,
+                index_node_entries_start(INODE_TREE_KEY_BYTES)
+                    + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET,
+            ) == PACKED_RECORD_TYPE_INODE
+    };
+    record_new_pool_with_a_resealed_field(
+        tag,
+        "new-pool-inode-root-entry-type-zero-names-its-leaf-container",
+        &inode_tree_root_over_leaf_containers,
+        false,
+        &|bytes| set_the_type_of_the_first_entry(bytes, 0),
+    )
+}
+
 /// I-9.18：删除中间态的流上墓碑容器里那一条墓碑的对象 ID 1 → 0、重封到根。队列 {1} 里的号 1 在墓碑树里没有墓碑：CPU 判 I-9.18 红；
 /// 号 0 没有记录，I-9.10 墓碑那一个等号与 I-9.19 没有对象，号 0 不越过水位，I-9.6 墓碑那一半照旧成立。
 fn record_deletion_with_the_tombstone_naming_an_object_without_a_record(tag: &str) -> RecordedFlow {
@@ -5907,28 +5981,7 @@ fn record_two_empty_inodes_unlinked_with_one_tombstone_per_container(tag: &str)
         node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
     };
     for inode in [FIRST_FILE.0 + 1, FIRST_FILE.0 + 2] {
-        let publish_parameters = parameters();
-        let devices = pool.devices.as_mut().expect("镜像还开着");
-        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
-        writer.set_container_tree_capacities(capacities);
-        let mut current = publish_unlink_intent(
-            &mut writer,
-            &mut pool.allocator,
-            &pool.output,
-            InodeNumber(inode),
-            WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
-            INSTANCE_OF_THE_PUBLISHES,
-        )
-        .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
-        publish_unlink_steps_until_the_queue_is_empty(
-            &mut writer,
-            &mut pool.allocator,
-            &mut current,
-            UnlinkBatchSize::OneExtentLowerLeaf,
-            INSTANCE_OF_THE_PUBLISHES,
-        )
-        .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的各批：{refusal:?}"));
-        pool.output = current;
+        unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, inode, capacities);
     }
     let versions = versions_with_the_first_file_through_the_last_publish(&pool);
     recorded_flow_of_the_pool(
@@ -5938,6 +5991,270 @@ fn record_two_empty_inodes_unlinked_with_one_tombstone_per_container(tag: &str)
     )
 }
 
+/// 写入口装上只供测试的容器树旋钮 `capacities`，把空 inode `inode` unlink 到底：意图那一次，再走空（空 inode 没有数据单元，直接是最后一批）。
+fn unlink_an_empty_inode_to_the_end_under_the_capacities(
+    pool: &mut BuiltPool,
+    inode: u64,
+    capacities: ContainerTreeCapacities,
+) {
+    let publish_parameters = parameters();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+    writer.set_container_tree_capacities(capacities);
+    let mut current = publish_unlink_intent(
+        &mut writer,
+        &mut pool.allocator,
+        &pool.output,
+        InodeNumber(inode),
+        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
+        INSTANCE_OF_THE_PUBLISHES,
+    )
+    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
+    publish_unlink_steps_until_the_queue_is_empty(
+        &mut writer,
+        &mut pool.allocator,
+        &mut current,
+        UnlinkBatchSize::OneExtentLowerLeaf,
+        INSTANCE_OF_THE_PUBLISHES,
+    )
+    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的各批：{refusal:?}"));
+    pool.output = current;
+}
+
+/// 墓碑容器一片 1 条、容器树一个节点 2 条（`crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs` 的
+/// `capacities_of_one_record_and_two_entries`）：unlink 第三个 inode 时墓碑树根满、长到层级 2。
+fn capacities_of_one_tombstone_per_container_and_two_entries_per_node() -> ContainerTreeCapacities {
+    ContainerTreeCapacities {
+        tombstone_container: TombstoneContainerCapacity::CappedForTests(
+            TombstoneRecordsPerContainerCappedForTests::new(
+                TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
+            )
+            .expect("1..=815"),
+        ),
+        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
+        node_entries: ContainerTreeNodeEntries::CappedForTests(
+            ContainerTreeNodeEntriesCappedForTests::new(
+                CONTAINER_TREE_ENTRIES_PER_NODE_UNDER_THE_KNOB,
+            )
+            .expect("2..=135"),
+        ),
+    }
+}
+
+/// 层级 2 的墓碑树根：码 2、key 宽 8、条目宽 120、层级 2、第一条条目类型段 0（指层级 1 的码 2 节点）。
+/// 这几条流里谱系树是零根、inode 树只有一片叶容器（根层级 1），层级 2 的这种节点只有墓碑树根。
+fn is_a_tombstone_tree_root_of_level_two(bytes: &[u8]) -> bool {
+    unit_class(bytes) == 2
+        && usize::from(index_node_key_width(bytes)) == CONTAINER_NUMBER_KEY_BYTES
+        && index_node_entry_width(bytes) == CONTAINER_TREE_ENTRY_BYTES
+        && bytes[INDEX_NODE_LEVEL_OFFSET] == 2
+        && read_u16_at(
+            bytes,
+            first_entry_of_a_container_number_node() + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET,
+        ) == 0
+}
+
+/// 新池新建文件、建五个空 inode（2–6 号），写入口装 [`capacities_of_one_tombstone_per_container_and_two_entries_per_node`]，
+/// unlink 2、3、4 号各到底：三片墓碑容器各装 1 条，第三次 unlink 的意图那次墓碑树根长到层级 2（根两条类型段 0 的条目，
+/// 各指一个层级 1 的码 2 节点）。旋钮下的合法镜像；CPU 判器走旧入口当没装旋钮，I-9.21 红（同 I-9.21 那条流）。
+/// `turn_the_root_ring_over` 时再建 `PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER`（`common_deletion_in_progress`）次空 inode：
+/// 墓碑树长高之前那几版都转出回退候选集，层级 1 那两个节点只剩现行这棵墓碑树（与之后照抄它的各版）引用。
+fn record_a_tombstone_tree_grown_to_level_two(
+    tag: &str,
+    name: &'static str,
+    turn_the_root_ring_over: bool,
+) -> RecordedFlow {
+    let mut pool = pool_with_new_inodes(tag, 5);
+    let capacities = capacities_of_one_tombstone_per_container_and_two_entries_per_node();
+    for inode in [FIRST_FILE.0 + 1, FIRST_FILE.0 + 2, FIRST_FILE.0 + 3] {
+        unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, inode, capacities);
+    }
+    assert_eq!(
+        pool.output.tombstone_tree_index_nodes.shape.root_level(),
+        2,
+        "第三片墓碑容器让根长到层级 2"
+    );
+    if turn_the_root_ring_over {
+        turn_the_root_ring_over_by_creating_inodes(&mut pool);
+    }
+    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
+    recorded_flow_of_the_pool(&pool, name, versions)
+}
+
+/// I-9.23 类型段 = 记录类型那一支、孩子其实是码 2 节点（攻方 N4-f 那一形）：[`record_a_tombstone_tree_grown_to_level_two`] 的流上
+/// 层级 2 的墓碑树根第一条条目类型段 0 → 1、重封到根。条目说它指一片墓碑容器，孩子是层级 1 的码 2 节点：
+/// CPU 只读孩子的单元头得码 2、与类型段 1 不符，判 I-9.23 红、按孩子自述的类登记一槽引用、不往下走、不按数据单元长度去读、不记走读失败
+/// （第 24 批；原来按 32 KiB 读那个 16 KiB 的节点，I-2.1 红、记走读失败，I-4.8、I-7.2、I-7.4 连带红）；GPU 走读内核同判同登记。
+/// `turn_the_root_ring_over` 是攻方 N4-j 那一形：同一刀落在根环转过一圈之后，那个节点下面的容器没有别的版本再引用。
+fn record_a_tombstone_tree_of_level_two_whose_first_root_entry_names_its_node_as_a_container(
+    tag: &str,
+    name: &'static str,
+    turn_the_root_ring_over: bool,
+) -> RecordedFlow {
+    let mut flow = record_a_tombstone_tree_grown_to_level_two(tag, name, turn_the_root_ring_over);
+    let targets = last_unit_copies(&flow, name, &is_a_tombstone_tree_root_of_level_two, false);
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
+        set_the_type_of_the_first_entry(bytes, PACKED_RECORD_TYPE_TOMBSTONE);
+    });
+    flow
+}
+
+/// inode 记录里对象出生代那 8 字节的偏移（D18（块里携带什么信息） 已定项 16 的 inode 记录字段表：inode 号 0、对象出生代 8）。
+const INODE_RECORD_OBJECT_BIRTH_OFFSET: usize = 8;
+/// 实例表单元的打包记录类型（D18（块里携带什么信息） 已定项 11）。
+const PACKED_RECORD_TYPE_INSTANCE_TABLE: u16 = 4;
+/// 翻坏一个单元的载荷时翻的那一字节离单元末尾几字节（`checker_known_bad_images.rs` 的 `break_both_copies_of_the_unit` 同一处）：
+/// 不重封，位置条目里的整单元校验和对不上，两份都读不出。
+const BYTES_FROM_THE_END_OF_THE_FLIPPED_PAYLOAD_BYTE: usize = 100;
+
+/// I-9.29（同号只许一个出生代） 判不了那一形（`.claude/kb/invariants.md` I-9.29 那一行「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」；
+/// 第 23 批 X 的 harness 坏镜像 `an_abandoned_root_naming_another_object_birth_leaves_the_one_birth_invariant_partially_judged_once_the_newest_instance_table_does_not_read_back`
+/// 同一段历史，录成流）：A (1, 3)、B (1, 4) → 重开取号 2（写行、暖机）→ C (2, 8)；崩溃恢复抛弃 C
+/// （`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：C 的根槽与数据单元暂时读不出、见证 C 的系统配置槽写回轮换之前那一代，
+/// 择根落到 C 之前那条根，实例 3 写行、暖机，再把暂存的字节原样写回）。录成流之后 C 那片 inode 叶容器两份里第一个文件那条记录的对象出生代加一、
+/// 重封到 C 的根（C 那一版与现行线上各版同号不同出生代），再把最新根（实例 3）指着的实例表两份各翻一个载荷字节、不重封。
+/// 实例 3 的根落了、C 写回了的状态上，候选集滤不掉被抛弃的 C：CPU 报 I-9.29 部分没判、不红，GPU 扫描内核那张表不比、不置位
+/// （照旧比的话 C 那条记录与现行线同号不同出生代，GPU 红 I-9.29、CPU 不红）。交回流，与翻坏实例表之前这条流整条落盘之后的镜像（候选集滤得掉 C 的对照）。
+fn record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_unreadable(
+    tag: &str,
+) -> (RecordedFlow, MemoryPool) {
+    let mut pool = build_pool(tag);
+    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
+    record_pool_build(&mut recorder, &pool);
+    let mut versions = vec![PublishedVersion {
+        instance: InstanceGeneration(1),
+        checkpoint_txg: CheckpointTxg(3),
+        content: file_content(),
+    }];
+    let second = overwrite_step(
+        &mut recorder,
+        &mut pool,
+        &second_content(),
+        InstanceGeneration(1),
+    );
+    let second_txg = second.root.checkpoint_txg.0;
+    versions.push(PublishedVersion {
+        instance: InstanceGeneration(1),
+        checkpoint_txg: CheckpointTxg(second_txg),
+        content: second_content(),
+    });
+    pool.output = second;
+    let mounted = mount_writable_step(&mut recorder, &mut pool);
+    pool.allocator = mounted.allocator;
+    pool.output = mounted
+        .current
+        .into_file_version()
+        .expect("B 之后重开，现行那一版带文件");
+    for txg in second_txg + 1..=pool.output.root.checkpoint_txg.0 {
+        versions.push(PublishedVersion {
+            instance: InstanceGeneration(2),
+            checkpoint_txg: CheckpointTxg(txg),
+            content: second_content(),
+        });
+    }
+    let operations_before_the_third = pool.stream.operation_count() - pool.mkfs_operation_count;
+    let third = overwrite_step(
+        &mut recorder,
+        &mut pool,
+        &third_content(),
+        InstanceGeneration(2),
+    );
+    let operations_through_the_third = pool.stream.operation_count() - pool.mkfs_operation_count;
+    versions.push(PublishedVersion {
+        instance: InstanceGeneration(2),
+        checkpoint_txg: third.root.checkpoint_txg,
+        content: third_content(),
+    });
+    pool.output = third.clone();
+    let abandoning =
+        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &third);
+    assert_eq!(
+        (
+            abandoning.output.instance,
+            abandoning.output.effective_root.checkpoint_txg.0 + 1,
+            abandoning.output.row_publish.root().checkpoint_txg.0,
+        ),
+        (
+            InstanceGeneration(3),
+            third.root.checkpoint_txg.0,
+            third.root.checkpoint_txg.0 + 1
+        ),
+        "恢复落到 C 之前那条根，实例 3 写行接在 C 的 txg 之后"
+    );
+    for txg in
+        abandoning.output.row_publish.root().checkpoint_txg.0..=pool.output.root.checkpoint_txg.0
+    {
+        versions.push(PublishedVersion {
+            instance: InstanceGeneration(3),
+            checkpoint_txg: CheckpointTxg(txg),
+            content: second_content(),
+        });
+    }
+    let operations = pool.retained_operations();
+    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
+        &operations[pool.mkfs_operation_count..],
+        &geometry(),
+    );
+    let judged_root_index = writes
+        .iter()
+        .rposition(|write| write.kind == StepKind::RootRecordFua)
+        .expect("根槽写");
+    let mut flow = RecordedFlow {
+        name: "abandoned-root-naming-another-object-birth-newest-instance-table-unreadable",
+        base: pool.memory_pool_after_mkfs(),
+        writes,
+        segments,
+        judged_root_index,
+        versions,
+    };
+    let inode_leaf_of_the_third: Vec<usize> = stream_indexes
+        .iter()
+        .enumerate()
+        .filter(|(write_index, stream_index)| {
+            (operations_before_the_third..operations_through_the_third).contains(*stream_index)
+                && flow.writes[*write_index].kind == StepKind::UnitWrite
+                && flow.writes[*write_index].bytes().is_some_and(|bytes| {
+                    unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_INODE
+                })
+        })
+        .map(|(write_index, _)| write_index)
+        .collect();
+    assert_eq!(
+        inode_leaf_of_the_third.len(),
+        2,
+        "C 那次发布写一片 inode 叶容器、两份副本"
+    );
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &inode_leaf_of_the_third, &|bytes| {
+        let object_birth =
+            offset_of_the_inode_record(bytes, FIRST_FILE.0) + INODE_RECORD_OBJECT_BIRTH_OFFSET;
+        let another_object_birth = read_u64_at(bytes, object_birth) + 1;
+        bytes[object_birth..object_birth + 8].copy_from_slice(&another_object_birth.to_le_bytes());
+    });
+    let mut image_with_the_newest_instance_table_readable = flow.base.clone();
+    image_with_the_newest_instance_table_readable.apply_writes(&flow.writes);
+    let is_an_instance_table_unit = |bytes: &[u8]| {
+        unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_INSTANCE_TABLE
+    };
+    let newest_instance_table = last_unit_copies(
+        &flow,
+        "实例 3 写行那次写的实例表",
+        &is_an_instance_table_unit,
+        false,
+    );
+    for write_index in newest_instance_table {
+        assert!(
+            stream_indexes[write_index] >= operations_through_the_third,
+            "最后写的那张实例表是实例 3 写行那次写的"
+        );
+        let WrittenContents::Bytes(bytes) = &mut flow.writes[write_index].contents else {
+            panic!("单元写是普通写");
+        };
+        let flipped = bytes.len() - BYTES_FROM_THE_END_OF_THE_FLIPPED_PAYLOAD_BYTE;
+        bytes[flipped] ^= 0xff;
+    }
+    (flow, image_with_the_newest_instance_table_readable)
+}
+
 /// 删 → 回退 → 再删的合法历史（第 22 批 C 报告的合法历史之一）：新池新建文件、建空 inode 2（txg 4），把它 unlink 到底，挂着回退到 txg 4
 /// （谱系 (回退那次的 txg, 4)；墓碑树随现行那一版，头一条墓碑的死亡 txg 落在被回退抛弃的那一段里、不在谱系上），再把它 unlink 到底
 /// （又一条 (2, 同一个对象出生代) 的墓碑，在谱系上）。不按谱系滤的写法在这里判 I-9.20 不重号红。
@@ -5986,7 +6303,7 @@ fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_carries_an_unre
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_carries_an_identity()
 {
-    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
         "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_carries_an_identity",
         &record_deletion_with_the_tombstone_root_entry_of_type_zero(
             "crash-judge-gpu-i923-tombstone-entry-type-zero",
@@ -6129,3 +6446,156 @@ fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_an_unlink_rolled_back_and
         );
     }
 }
+
+/// 墓碑树、谱系树条目的类型段与孩子的真实类不符时，原来的读法跟着连带红的那几条（攻方 N4 那几形，
+/// `research/prompts/m3-batch22-code-r1-opus-output.md` 第一节）：按类型段定的长度读孩子读不出（I-2.1），记走读失败让最新根走不完
+/// （I-7.2、I-4.8、I-7.4）。
+const INVARIANTS_A_WALK_FAILURE_REDDENS: [&str; 4] = ["I-2.1", "I-4.8", "I-7.2", "I-7.4"];
+
+/// 墓碑树、谱系树条目的类型段与孩子的真实类不符的流（攻方 N4 那几形）：CPU 池级 checker 红 `identity_invariant`，
+/// [`INVARIANTS_A_WALK_FAILURE_REDDENS`] 一个状态都不红（孩子的类标签只读单元头、不按类型段定读多长，不符就判身份、不往下走、不记走读失败，
+/// 判决 `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 N4 那一行）；GPU 每个状态逐位相同。
+fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
+    test_name: &str,
+    flow: &RecordedFlow,
+    identity_invariant: &'static str,
+) {
+    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, flow, expand_up_to());
+    assert!(
+        comparison
+            .cpu_checker_red_invariants
+            .contains(identity_invariant),
+        "流 {} 上 CPU 池级 checker 要红 {identity_invariant}（红过的：{:?}）",
+        flow.name,
+        comparison.cpu_checker_red_invariants
+    );
+    for invariant in INVARIANTS_A_WALK_FAILURE_REDDENS {
+        assert!(
+            !comparison.cpu_checker_red_invariants.contains(invariant),
+            "流 {} 上身份不符不记走读失败，CPU 池级 checker 不红 {invariant}（红过的：{:?}）",
+            flow.name,
+            comparison.cpu_checker_red_invariants
+        );
+    }
+    assert!(
+        comparison.mismatches.is_empty(),
+        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
+    );
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_without_an_identity_names_a_container(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_without_an_identity_names_a_container",
+        &record_deletion_with_the_tombstone_root_entry_of_type_zero_and_no_identity(
+            "crash-judge-gpu-i923-tombstone-entry-type-zero-no-identity",
+        ),
+        "I-9.23",
+    );
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_below_the_root(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_below_the_root",
+        &record_a_tombstone_tree_of_level_two_whose_first_root_entry_names_its_node_as_a_container(
+            "crash-judge-gpu-i923-level-two-entry-names-a-node",
+            "tombstone-tree-of-level-two-first-root-entry-names-its-node-as-a-container",
+            false,
+        ),
+        "I-9.23",
+    );
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_below_the_root_after_the_root_ring_turned_over(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_below_the_root_after_the_root_ring_turned_over",
+        &record_a_tombstone_tree_of_level_two_whose_first_root_entry_names_its_node_as_a_container(
+            "crash-judge-gpu-i923-level-two-entry-names-a-node-ring-turned-over",
+            "tombstone-tree-of-level-two-first-root-entry-names-its-node-as-a-container-ring-turned-over",
+            true,
+        ),
+        "I-9.23",
+    );
+}
+
+/// 池级 checker（CPU，旧入口）在一份镜像上判 `invariant` 的结果。
+fn pool_checker_verdict_on_the_image(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
+    check_pool_image(image)
+        .into_iter()
+        .find(|(name, _)| *name == invariant)
+        .map(|(_, verdict)| verdict)
+        .unwrap_or_else(|| panic!("池级 checker 报了 {invariant}"))
+}
+
+/// I-9.29 判不了那一形的流（[`record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_unreadable`]）：
+/// CPU 一侧先钉流造对了——翻坏实例表之前整条落盘的镜像上 C 被滤掉、I-9.29 真被评估过且成立；翻坏之后整条落盘的镜像上 I-9.29 报部分没判。
+/// 流上 C 还是最新根的那几个状态，C 的实例表读得出、C 进候选集，同号不同出生代照比，CPU 红 I-9.29；GPU 每个状态逐位相同
+/// （实例 3 的根落了、C 写回了的状态上 GPU 照旧比的话红 I-9.29 而 CPU 不红）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_object_birth_is_not_compared_once_the_newest_instance_table_does_not_read_back(
+) {
+    let (flow, image_with_the_newest_instance_table_readable) =
+        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_unreadable(
+            "crash-judge-gpu-i929-newest-instance-table-unreadable",
+        );
+    assert_eq!(
+        pool_checker_verdict_on_the_image(&image_with_the_newest_instance_table_readable, "I-9.29"),
+        InvariantVerdict::Holds,
+        "最新根的实例表读得出：被抛弃的 C 按它滤掉、不进候选集，现行线上各版同号同出生代，I-9.29 真被评估过且成立"
+    );
+    let mut image_with_the_newest_instance_table_unreadable = flow.base.clone();
+    image_with_the_newest_instance_table_unreadable.apply_writes(&flow.writes);
+    match pool_checker_verdict_on_the_image(&image_with_the_newest_instance_table_unreadable, "I-9.29") {
+        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
+            unjudged.starts_with("最新根的实例表读不出"),
+            "部分没判的说明点名最新根的实例表读不出：{unjudged}"
+        ),
+        other_verdict @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::NotApplicable(_)) => panic!(
+            "最新根的实例表两份都翻坏、候选集滤不掉被抛弃的 C：I-9.29 要报部分没判，实际 {other_verdict:?}"
+        ),
+    }
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_object_birth_is_not_compared_once_the_newest_instance_table_does_not_read_back",
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
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container()
+{
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
+        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container",
+        &record_new_pool_with_the_inode_root_entry_of_type_zero_naming_its_leaf_container(
+            "crash-judge-gpu-i92-inode-root-entry-type-zero",
+        ),
+        "I-9.2",
+    );
+}
````

## 实现员 g：mutations-append.tsv（/tmp/claude-1000/m3-batch24/g/patch/mutations-append.tsv）

````text
第 24 批 GPU 判器 N4：墓碑树、谱系树类型段 0 的条目不按孩子头里的类认（层级 1 墓碑树根条目类型段 0、身份三段清零那条流上 CPU 红 I-9.23 不记走读失败，GPU 照旧按节点长度读不出孩子、I-9.23 不置位、记走读失败）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            if (class_of_the_child_of_an_internal_entry != NONE) {	            if (false && class_of_the_child_of_an_internal_entry != NONE) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_without_an_identity_names_a_container	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_without_an_identity_names_a_container
第 24 批 GPU 判器 N4：墓碑树、谱系树类型段 0 的条目不按孩子头里的类认（层级 1 墓碑树根条目类型段 0 指码 3 容器那条流上 CPU 只红 I-9.23，GPU 再记走读失败、I-4.8、I-7.2、I-7.4 连带红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            if (class_of_the_child_of_an_internal_entry != NONE) {	            if (false && class_of_the_child_of_an_internal_entry != NONE) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_carries_an_identity	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_carries_an_identity
第 24 批 GPU 判器 N4：墓碑树、谱系树类型段 = 记录类型的条目不按孩子头里的类认（层级 2 墓碑树根条目类型段 1 指码 2 节点那条流上 GPU 按 32 KiB 读那个节点、I-2.1 红、记走读失败，CPU 不）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        if (class_of_the_child_of_a_container_entry != NONE) {	        if (false && class_of_the_child_of_a_container_entry != NONE) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_below_the_root	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_below_the_root
第 24 批 GPU 判器 N4：墓碑树、谱系树类型段 = 记录类型的条目不按孩子头里的类认（根环转过一圈之后层级 2 墓碑树根条目类型段 1 指码 2 节点那条流上 GPU 按 32 KiB 读那个节点、I-2.1 红、记走读失败，CPU 不）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        if (class_of_the_child_of_a_container_entry != NONE) {	        if (false && class_of_the_child_of_a_container_entry != NONE) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_below_the_root_after_the_root_ring_turned_over	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_below_the_root_after_the_root_ring_turned_over
第 24 批 GPU 判器 N4：inode 树内部条目不按孩子头里的类认（inode 树根条目类型段 0 指码 3 叶容器那条流上 CPU 红 I-9.2，GPU 照旧把类型段 0 的孩子整个状态判不了）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        if (class_of_the_inode_child != NONE) {	        if (false && class_of_the_inode_child != NONE) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container	the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container
第 24 批 GPU 判器 I-9.29：最新根的实例表读不出时照旧比同号出生代（被抛弃的根与现行线同号不同出生代、最新根的实例表两份翻坏那条流上 CPU 报部分没判，GPU 红 I-9.29）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    d_object_births_are_compared = r_instance_rows(checker_root_of(c_newest_index).base);	    d_object_births_are_compared = true;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_object_birth_is_not_compared_once_the_newest_instance_table_does_not_read_back	the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_object_birth_is_not_compared_once_the_newest_instance_table_does_not_read_back
````
