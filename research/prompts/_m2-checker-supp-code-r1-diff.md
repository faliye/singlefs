# 附录二：池级 checker 补三条、事务号按实例计数、checker 与恢复的挂载口径对齐（`git diff -- crates/singlefs-checker/src/ crates/singlefs-core/src/` 原样；生成时刻 2026-09-21 18:40 UTC / 2026-09-22 03:40 JST，基准 HEAD 11a551b，工作区未暂存改动）

## 一、diff（`git status --porcelain -- crates/singlefs-checker/src/ crates/singlefs-core/src/` 五处全是 ` M`，无 `??`，diff 范围内没有新增文件，因此本附录没有「新文件全文」一节）

```diff
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
index fc0866c..09a9902 100644
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -33,10 +33,11 @@ pub enum InvariantVerdict {
 }
 
 /// 第一版 checker 判的不变量，按这个次序报；每次都全部报出来，没评估到的报「不适用」。
-pub const IMPLEMENTED_INVARIANTS: [&str; 29] = [
-    "I-1.1", "I-1.3", "I-1.4", "I-1.6", "I-1.7", "I-2.1", "I-2.3", "I-2.4", "I-2.5", "I-3.1",
-    "I-3.8", "I-3.9", "I-4.8", "I-5.1", "I-5.2", "I-5.4", "I-7.1", "I-7.2", "I-7.4", "I-7.6",
-    "I-7.7", "I-7.8", "I-9.1", "I-9.2", "I-9.4", "I-9.7", "I-9.10", "I-9.13", "I-9.14",
+pub const IMPLEMENTED_INVARIANTS: [&str; 32] = [
+    "I-1.1", "I-1.3", "I-1.4", "I-1.6", "I-1.7", "I-1.8", "I-2.1", "I-2.3", "I-2.4", "I-2.5",
+    "I-3.1", "I-3.8", "I-3.9", "I-4.8", "I-5.1", "I-5.2", "I-5.4", "I-7.1", "I-7.2", "I-7.3",
+    "I-7.4", "I-7.6", "I-7.7", "I-7.8", "I-8.6", "I-9.1", "I-9.2", "I-9.4", "I-9.7", "I-9.10",
+    "I-9.13", "I-9.14",
 ];
 
 /// 判定累加器：每条不变量记评估了几次、第一处违例、以及整条不适用的理由。
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index f4f63c5..cbb40d9 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -1,7 +1,8 @@
-//! 池级 checker：从根环里每一条自证过的根走下去（遍历方向），再扫一遍单元头（扫描方向），判第一版的 29 条不变量。
+//! 池级 checker：从根环里每一条自证过的根走下去（遍历方向），再扫一遍单元头与 journal 环（扫描方向），判第一版的 32 条不变量。
 //! 最新的那条根另判 I-7.2（能不能完整走完）；引用集合按全部有效根取并集——根环里更早的根仍是可回退的状态，
 //! 它们引用的单元仍占着空间（I-3.1 与 I-5.1 的这一读法 2026-09-14 用户收尾弹窗定甲，见 records 2026-09-13-总审核 十一·七）。
 
+use std::cmp::Ordering;
 use std::collections::{BTreeMap, BTreeSet};
 
 use singlefs_format::{
@@ -16,9 +17,9 @@ use crate::image::{
     ImageReader, InvariantVerdict, Judgements, PointerView, PoolGeometry,
 };
 use crate::{
-    check_index_node_keys, check_journal_record, checksum_field_holds, crc32_castagnoli_table,
-    index_node_view, key_schema_for_tree_kind, read_six_byte_unsigned, read_u16, read_u32,
-    read_u64, KEY_SCHEMA_MAPPING, KEY_SCHEMA_TREE_TABLE,
+    back_chain_of_record_header, check_index_node_keys, check_journal_record, checksum_field_holds,
+    crc32_castagnoli_table, index_node_view, key_schema_for_tree_kind, read_six_byte_unsigned,
+    read_u16, read_u32, read_u64, KEY_SCHEMA_MAPPING, KEY_SCHEMA_TREE_TABLE,
 };
 
 const TREE_KIND_EXTENT: u16 = 1;
@@ -74,8 +75,20 @@ struct Walk<'reader> {
     /// 最新根下面记账树里的行：(统计量, 设备) → 值。
     accounting: BTreeMap<(u16, u32), u64>,
     accounting_seen: bool,
-    /// 最新根指着的实例表里的 (实例代号, T)：别的根按它判有效（回退之后被抛弃时间线的根不进 I-3.1 的并集）。
-    instance_table_rows: Vec<(u32, u64)>,
+    /// 最新根指着的实例表里的行：别的根按它判有效（回退之后被抛弃时间线的根不进 I-3.1 的并集），
+    /// I-1.8（归并后版本全序） 的已发布谓词也读它。
+    instance_table_rows: Vec<InstanceTableRow>,
+}
+
+/// 实例表里的一行（D18（块里携带什么信息） 已定项 11 的字段表：`kind` 1 + 实例代号 4 + 所选根的 `checkpoint_txg` 8 +
+/// 属于该实例的最大已施加事务号 W 8 + flags 1 + 预留 66）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct InstanceTableRow {
+    instance: u32,
+    /// 这一行记的 T：该实例被施加到哪一代为止（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的 `T_pub`）。
+    published_checkpoint_txg: u64,
+    /// 这一行记的 W：属于该实例的、被这次重放施加的最大事务号。
+    applied_transaction_high_water_mark: u64,
 }
 
 impl Walk<'_> {
@@ -436,10 +449,15 @@ impl Walk<'_> {
                         unique = false;
                     }
                     instances.push(instance);
-                    self.instance_table_rows.push((
+                    self.instance_table_rows.push(InstanceTableRow {
                         instance,
-                        u64::from_le_bytes(row[5..13].try_into().expect("8 字节")),
-                    ));
+                        published_checkpoint_txg: u64::from_le_bytes(
+                            row[5..13].try_into().expect("8 字节"),
+                        ),
+                        applied_transaction_high_water_mark: u64::from_le_bytes(
+                            row[13..21].try_into().expect("8 字节"),
+                        ),
+                    });
                     if instance >= mount_root_instance {
                         below_mount_root = false;
                     }
@@ -1288,6 +1306,480 @@ pub fn allocation_record_count_under_root(
     Some(records)
 }
 
+/// 单元头校验和字段的偏移（共同前缀，三类同一处）。
+const UNIT_HEADER_CHECKSUM_OFFSET: usize = 10;
+/// 写序那 10 字节里事务号低 48 位从第 4 字节起（D18（块里携带什么信息） 已定项 7：实例代号 4 + 事务号 6）。
+const WRITE_ORDER_TRANSACTION_OFFSET: usize = 4;
+
+/// I-1.8（归并后版本全序） 罩得到的两类单元。码 2 不进这一条（`.claude/kb/invariants.md` I-1.8 逐字「码 2 不进这条」：
+/// 它的写序已收窄成只有实例代号，多版在条目级合并、不在容器级择版本）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum ContentUnitClass {
+    /// 码 1 数据单元。
+    Data,
+    /// 码 3 打包记录单元。
+    PackedRecord,
+}
+
+/// 一类内容单元的头里，这一条要读的那几段。码 1 按 D18（块里携带什么信息） 已定项 7 的字段表（五元组 42、诞生代号 75、
+/// fsid 83、写序 91、载荷 CRC 101、明文头末尾 105）；码 3 按已定项 11 的类身份段表（标签 42、出生树 43、类型 51、容器号 53、
+/// 容器出生代 61、记录数 69、记录宽 71、诞生代号 73、fsid 81、载荷校验和 89、写序 93、出生序号 103、明文头末尾 107）。
+struct ContentUnitHeaderOffsets {
+    plain_header_end: usize,
+    payload_checksum: usize,
+    filesystem_identifier: usize,
+    birth_txg: usize,
+    write_order: usize,
+    /// 类身份段里与版本无关的那一段：码 1 的五元组、码 3 的（标签, 出生树, 类型, 容器号, 容器出生代）。
+    object_key: std::ops::Range<usize>,
+    /// 归并键的两段：「类身份段全部字段含写序」**去掉载荷校验和那一段**。
+    /// 载荷校验和不进归并键——它正是「同一组的成员载荷相同」要比的那个量，进了键这一条就恒真。
+    merge_key_before_payload_checksum: std::ops::Range<usize>,
+    merge_key_after_payload_checksum: std::ops::Range<usize>,
+}
+
+impl ContentUnitClass {
+    fn of(unit_class_tag: u8) -> Option<Self> {
+        match unit_class_tag {
+            1 => Some(ContentUnitClass::Data),
+            3 => Some(ContentUnitClass::PackedRecord),
+            _outside_this_invariants_reach => None,
+        }
+    }
+
+    fn header_offsets(self) -> ContentUnitHeaderOffsets {
+        match self {
+            ContentUnitClass::Data => ContentUnitHeaderOffsets {
+                plain_header_end: 105,
+                payload_checksum: 101,
+                filesystem_identifier: 83,
+                birth_txg: 75,
+                write_order: 91,
+                object_key: 42..75,
+                merge_key_before_payload_checksum: 42..101,
+                merge_key_after_payload_checksum: 105..105,
+            },
+            ContentUnitClass::PackedRecord => ContentUnitHeaderOffsets {
+                plain_header_end: 107,
+                payload_checksum: 89,
+                filesystem_identifier: 81,
+                birth_txg: 73,
+                write_order: 93,
+                object_key: 42..69,
+                merge_key_before_payload_checksum: 42..89,
+                merge_key_after_payload_checksum: 93..107,
+            },
+        }
+    }
+
+    fn name(self) -> &'static str {
+        match self {
+            ContentUnitClass::Data => "码 1 数据",
+            ContentUnitClass::PackedRecord => "码 3 打包记录",
+        }
+    }
+}
+
+/// 已发布谓词（I-1.2（块头写序已发布） 的谓词，C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 /
+/// D18（块里携带什么信息） 已定项 1）的输入：挂载根（池级 checker 里就是最新的那条根）的实例代号与 txg，
+/// 加上它指着的那一版实例表里的行。I-1.8（归并后版本全序） 的射程逐字是「可读**已发布**单元」，扫描方向要拿这个谓词过滤。
+struct PublishedPredicate<'rows> {
+    mount_root_instance: u32,
+    mount_root_checkpoint_txg: u64,
+    instance_table_rows: &'rows [InstanceTableRow],
+}
+
+impl PublishedPredicate<'_> {
+    /// 写序 (i, n) 与诞生代号 b 判没判成已发布。i > 挂载根实例 ⇒ I-1.2 判损坏，那不是「已发布」⇒ 不进 I-1.8 的射程
+    /// （判损坏归 I-1.2，这一条不替它报）。
+    fn holds_for(
+        &self,
+        unit_class: ContentUnitClass,
+        birth_txg: u64,
+        instance: u32,
+        transaction: u64,
+    ) -> bool {
+        match instance.cmp(&self.mount_root_instance) {
+            Ordering::Greater => false,
+            Ordering::Equal => birth_txg <= self.mount_root_checkpoint_txg,
+            Ordering::Less => match self
+                .instance_table_rows
+                .iter()
+                .find(|row| row.instance == instance)
+            {
+                None => true,
+                Some(row) => match unit_class {
+                    ContentUnitClass::Data => {
+                        birth_txg <= row.published_checkpoint_txg
+                            || transaction <= row.applied_transaction_high_water_mark
+                    }
+                    ContentUnitClass::PackedRecord => birth_txg <= row.published_checkpoint_txg,
+                },
+            },
+        }
+    }
+}
+
+/// I-1.8 ② 的那个全序键。
+#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
+enum TotalOrderKey {
+    /// 码 3：(诞生代号, 实例代号, 事务号) 三元，诞生代号打头（`.claude/kb/invariants.md` I-1.8 那一行写死）。
+    OfPackedRecordUnit(Vec<u64>),
+    /// 码 1：**两处条款给的是两个键，没定哪一个作数，这一半停在这里**——I-1.8 那一行逐字「码 1 的键是写序」，
+    /// 而 D18（块里携带什么信息） 已定项 1 的射程逐字「同 key 已发布版本的定序靠诞生代号 + 每事务每 key 一种净效果就够，
+    /// 写序只在同 txg 跨事务改同 key 时顺带定序」。按前者判，同一实例的两次覆盖写在盘上就能撞出写序相同的两版
+    /// （`second_transaction_step_five_reuse.rs` 那条脚本上真有一对），按后者判它们由诞生代号分得开。
+    /// 哪一个作数要定案，定之前码 1 不进 ② 这一半；① 那一半（同一组的成员载荷相同）码 1 照判。
+    UndecidedForDataUnit,
+}
+
+/// 扫描方向读到的一个码 1 / 码 3 已发布单元（「可读」的后半截由 `payload_checksum_holds_on_disk` 现算）。
+struct ScannedContentUnit {
+    device: u32,
+    slot: u64,
+    unit_class: ContentUnitClass,
+    /// 「类身份段全部字段含写序」去掉载荷校验和那一段：同一份内容的两个副本在这上面逐字节相同。
+    merge_key: Vec<u8>,
+    /// 类身份段里与版本无关的那一段：I-1.8 里「同 key」的那个 key。
+    object_key: Vec<u8>,
+    total_order_key: TotalOrderKey,
+    payload_checksum: u32,
+}
+
+impl ScannedContentUnit {
+    fn describe(&self) -> String {
+        format!(
+            "盘 {} 槽 {} 的{}单元",
+            self.device,
+            self.slot,
+            self.unit_class.name()
+        )
+    }
+}
+
+/// 一个候选槽上的头解成扫描项：magic、类标签在这一条的射程内、头校验和过、fsid 与本池相同、按已发布谓词判成已发布。
+/// 任一步不成立就不是这一条判的对象（各自另有不变量说话：I-1.6 / I-2.4 / I-1.4 / I-1.2）。
+fn content_unit_of_header(
+    header: &[u8],
+    device: u32,
+    slot: u64,
+    filesystem_identifier_low: u64,
+    published: &PublishedPredicate<'_>,
+) -> Option<ScannedContentUnit> {
+    if header.len() < UNIT_HEADER_SCAN_BYTES || &header[..4] != b"SFSU" {
+        return None;
+    }
+    let unit_class = ContentUnitClass::of(header[6])?;
+    let offsets = unit_class.header_offsets();
+    if !checksum_field_holds(
+        header,
+        offsets.plain_header_end,
+        UNIT_HEADER_CHECKSUM_OFFSET,
+    ) || read_u64(header, offsets.filesystem_identifier) != filesystem_identifier_low
+    {
+        return None;
+    }
+    let birth_txg = read_u64(header, offsets.birth_txg);
+    let instance = read_u32(header, offsets.write_order);
+    let transaction =
+        read_six_byte_unsigned(header, offsets.write_order + WRITE_ORDER_TRANSACTION_OFFSET);
+    if !published.holds_for(unit_class, birth_txg, instance, transaction) {
+        return None;
+    }
+    let total_order_key = match unit_class {
+        ContentUnitClass::Data => TotalOrderKey::UndecidedForDataUnit,
+        ContentUnitClass::PackedRecord => {
+            TotalOrderKey::OfPackedRecordUnit(vec![birth_txg, u64::from(instance), transaction])
+        }
+    };
+    Some(ScannedContentUnit {
+        device,
+        slot,
+        unit_class,
+        merge_key: [
+            &header[offsets.merge_key_before_payload_checksum],
+            &header[offsets.merge_key_after_payload_checksum],
+        ]
+        .concat(),
+        object_key: header[offsets.object_key].to_vec(),
+        total_order_key,
+        payload_checksum: read_u32(header, offsets.payload_checksum),
+    })
+}
+
+/// 扫描方向：候选槽上的码 1 / 码 3 已发布单元。32768 的单元占两个槽，第二个槽上没有 magic，扫描自然跳过它。
+fn scanned_content_units(
+    reader: &dyn ImageReader,
+    geometry: &PoolGeometry,
+    filesystem_identifier_low: u64,
+    published: &PublishedPredicate<'_>,
+) -> Vec<ScannedContentUnit> {
+    let mut units = Vec::new();
+    for device in reader.devices() {
+        let slots = reader.candidate_unit_slots(device).unwrap_or_else(|| {
+            let end = reader.device_bytes(device).unwrap_or(0) / SLOT_BYTES;
+            (geometry.unit_area_start_slot..end).collect()
+        });
+        for slot in slots {
+            let Some(header) = reader.read(device, slot * SLOT_BYTES, UNIT_HEADER_SCAN_BYTES)
+            else {
+                continue;
+            };
+            if let Some(unit) =
+                content_unit_of_header(&header, device, slot, filesystem_identifier_low, published)
+            {
+                units.push(unit);
+            }
+        }
+    }
+    units
+}
+
+/// 这一份在盘上读得出来、而且它的载荷与它自己头里那个载荷校验和对得上——「可读」的后半截。
+/// 头校验和过而载荷对不上的那一份不可读（撕裂、被盖），不进 I-1.8 的归并组。
+fn payload_checksum_holds_on_disk(reader: &dyn ImageReader, unit: &ScannedContentUnit) -> bool {
+    let offsets = unit.unit_class.header_offsets();
+    let Some(bytes) = reader.read(unit.device, unit.slot * SLOT_BYTES, data_unit_bytes()) else {
+        return false;
+    };
+    bytes.len() > offsets.plain_header_end
+        && read_u32(&bytes, offsets.payload_checksum)
+            == crc32_castagnoli_table(&bytes[offsets.plain_header_end..])
+}
+
+/// I-1.8（归并后版本全序）：码 1 / 3 的可读已发布单元按「类身份段全部字段含写序」归并成组之后，
+/// ① **同一组的成员载荷相同**（码 3 由载荷校验和判、码 1 由载荷 CRC 判），不同即判损坏、不择；
+/// ② **同 key 的各组两两全序键不等**——码 3 的键是 (诞生代号, 实例代号, 事务号) 三元。
+///    **② 这一半今天只判码 3**：码 1 的键在两处条款里不是同一个，见 `TotalOrderKey::UndecidedForDataUnit`。
+/// 射程 w = 2（每次写两盘各一份，D2（RAID 条带策略） 已定项 6）：正常镜像上一组就是同一个单元的两个副本。
+/// **「可读」的后半截只在要判红的那一步才现算**：一组里载荷校验和字段全相同时 ① 已经成立、不必把载荷读回来；
+/// 字段不同才逐份验载荷——整单元 CRC 是这里最贵的一步，而层 0 每个崩溃状态都要跑这一遍。
+/// 交回判了几次：一次都没判到时调用方报「不适用」，不报成立（`test-discipline.md`「检查本身也可能是错的」）。
+fn judge_merged_version_total_order(
+    reader: &dyn ImageReader,
+    units: &[ScannedContentUnit],
+    judgements: &mut Judgements,
+) -> u64 {
+    let mut groups: BTreeMap<&Vec<u8>, Vec<&ScannedContentUnit>> = BTreeMap::new();
+    for unit in units {
+        groups.entry(&unit.merge_key).or_default().push(unit);
+    }
+    let mut judged = 0u64;
+    for members in groups.values() {
+        if members.len() < 2 {
+            continue;
+        }
+        let first_payload_checksum = members[0].payload_checksum;
+        let every_member_carries_the_same_payload_checksum = members
+            .iter()
+            .all(|member| member.payload_checksum == first_payload_checksum);
+        // 校验和字段全相同时这一条已经成立，`||` 短路掉后半截：那半截要把每一份载荷整个读回来算 CRC。
+        let payloads_are_the_same = every_member_carries_the_same_payload_checksum || {
+            let readable_payload_checksums: BTreeSet<u32> = members
+                .iter()
+                .filter(|member| payload_checksum_holds_on_disk(reader, member))
+                .map(|member| member.payload_checksum)
+                .collect();
+            readable_payload_checksums.len() <= 1
+        };
+        judged += 1;
+        judgements.judge("I-1.8", payloads_are_the_same, || {
+            let members_and_their_payloads: Vec<String> = members
+                .iter()
+                .map(|member| {
+                    format!(
+                        "{} 载荷校验和 {:#010x}",
+                        member.describe(),
+                        member.payload_checksum
+                    )
+                })
+                .collect();
+            format!(
+                "归并成一组的成员载荷不同（{}）：同一组的成员载荷必须相同，不同即判损坏、不择",
+                members_and_their_payloads.join("、")
+            )
+        });
+    }
+    // 一组里的成员共享归并键，对象 key 与全序键都是归并键的子段 ⇒ 取第一个成员当这一组的代表。
+    // 码 1 的全序键没定（`TotalOrderKey::UndecidedForDataUnit`），② 这一半只判码 3；对象 key 首字节是类标签，
+    // 一个 key 上的各组必然同类，不会把两类混进同一个桶。
+    let mut groups_by_object_key: BTreeMap<&Vec<u8>, Vec<&ScannedContentUnit>> = BTreeMap::new();
+    for members in groups.values() {
+        if members[0].total_order_key == TotalOrderKey::UndecidedForDataUnit {
+            continue;
+        }
+        groups_by_object_key
+            .entry(&members[0].object_key)
+            .or_default()
+            .push(members[0]);
+    }
+    for representatives in groups_by_object_key.values() {
+        if representatives.len() < 2 {
+            continue;
+        }
+        let distinct_total_order_keys: BTreeSet<&TotalOrderKey> = representatives
+            .iter()
+            .map(|representative| &representative.total_order_key)
+            .collect();
+        judged += 1;
+        judgements.judge(
+            "I-1.8",
+            distinct_total_order_keys.len() == representatives.len(),
+            || {
+                let groups_on_this_key: Vec<String> = representatives
+                    .iter()
+                    .map(|representative| {
+                        format!(
+                            "{} 全序键 {:?}",
+                            representative.describe(),
+                            representative.total_order_key
+                        )
+                    })
+                    .collect();
+                format!(
+                    "同一个 key 上归并出 {} 组，两两全序键不等这一条不成立（{}）",
+                    representatives.len(),
+                    groups_on_this_key.join("、")
+                )
+            },
+        );
+    }
+    judged
+}
+
+/// mkfs 把第 0 代种进根环（D22（单元原子性怎么合成） 已定项 8）：I-7.3（环健康性） 的例外那一支按这个代号判。
+const GENESIS_CHECKPOINT_TXG: u64 = 0;
+
+/// I-7.3（环健康性）：S（根槽里自证校验和通过的记录集合）中除代号最大者外，至少还存在一条更早代号的记录，
+/// 不存在即判红——某次提交把上一代直接覆盖了，轮换逻辑已失效。**例外只有一个**：S 全部是第 0 代时判绿
+/// （`.claude/kb/invariants.md` I-7.3 那一行的例外原样照搬；「崩溃后回退到这个态的镜像同样判绿」落在同一支上——
+/// 判据只读 S 里的代号，不读这个态是怎么来的）。
+/// 调用点在 I-7.1（根槽有效集非空） 之后：S 空时这一条报「不适用」，没有「代号最大者」可谈。
+fn judge_root_ring_health(roots: &[(u64, u64, crate::RootView)], judgements: &mut Judgements) {
+    let newest_checkpoint_txg = roots
+        .iter()
+        .map(|(_, _, root)| root.checkpoint_txg)
+        .max()
+        .expect("S 非空：`check_pool_image` 判过 I-7.1 之后 S 空就返回了，走不到这里");
+    let earlier_generation_exists = roots
+        .iter()
+        .any(|(_, _, root)| root.checkpoint_txg < newest_checkpoint_txg);
+    let every_root_is_the_genesis_generation = roots
+        .iter()
+        .all(|(_, _, root)| root.checkpoint_txg == GENESIS_CHECKPOINT_TXG);
+    judgements.judge(
+        "I-7.3",
+        earlier_generation_exists || every_root_is_the_genesis_generation,
+        || {
+            let generations: Vec<u64> = roots
+                .iter()
+                .map(|(_, _, root)| root.checkpoint_txg)
+                .collect();
+            format!(
+                "根环里自证过的根代号 {generations:?}：最大的是 {newest_checkpoint_txg}，除它之外一条更早代号的记录都没有，而它们又不全是第 0 代 ⇒ 上一代被直接覆盖了，下一次撕裂无路可退"
+            )
+        },
+    );
+}
+
+/// journal 记录的 jsn 计数器从 1 起、全池接着走（D23（journal 的角色与格式） 已定项 9 / 已定项 19 注 3：
+/// 新实例从前缀末 + 1 接着写、不归零）。
+const FIRST_JOURNAL_COUNTER: u64 = 1;
+
+/// 扫描方向读到的一条 journal 记录：它的实例代号、落在哪个环槽、反向链字段，以及「下一条记录的反向链该等于的值」。
+struct ScannedJournalRecord {
+    instance: u32,
+    slot: u64,
+    back_chain: u32,
+    /// CRC32C(这一条的 307 字节头，`header_csum` 那 32 字节按零参与)——I-8.6（反向链算法） 的算式。
+    chain_value_the_next_record_must_carry: u32,
+}
+
+/// 一块盘的 journal 环里自证过的记录，按 jsn 计数器索引。计数器与环槽一一对应（记录 n 落 `(计数器 − 1) mod 槽数 × 4096`，
+/// D23（journal 的角色与格式） 已定项 18）⇒ 同一块盘上两条自证过的记录不会撞同一个计数器。
+fn scanned_journal_records_of_device(
+    reader: &dyn ImageReader,
+    geometry: &PoolGeometry,
+    device: u32,
+    filesystem_identifier_low: u64,
+) -> BTreeMap<u64, ScannedJournalRecord> {
+    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
+    let slots = reader
+        .candidate_journal_slots(device)
+        .unwrap_or_else(|| (0..geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES).collect());
+    let mut records = BTreeMap::new();
+    for slot in slots {
+        let offset = geometry.journal_ring_start_slot * SLOT_BYTES + slot * JOURNAL_RECORD_BYTES;
+        let Some(bytes) = reader.read(device, offset, record_bytes) else {
+            continue;
+        };
+        let Ok(record) = check_journal_record(&bytes, filesystem_identifier_low) else {
+            continue;
+        };
+        records.insert(
+            record.counter,
+            ScannedJournalRecord {
+                instance: record.instance,
+                slot,
+                back_chain: record.back_chain,
+                chain_value_the_next_record_must_carry: back_chain_of_record_header(&bytes),
+            },
+        );
+    }
+    records
+}
+
+/// I-8.6（反向链算法）：任一 journal 记录的反向链 = CRC32C(**本实例内逻辑前一条**记录的 307 字节头，`header_csum` 那 32 字节
+/// 按零参与)；**本实例写出的第一条记录反向链恒 0**。逐盘判：环两盘互为镜像，一块盘上的链坏了另一块不该跟着坏。
+/// 「本实例内逻辑前一条」= 同一实例、计数器小 1 的那一条，三种情形穷举：
+/// ① 计数器 == 1 ⇒ 全池第一条，必是本实例第一条 ⇒ 反向链恒 0；
+/// ② 计数器 − 1 那一条在同一块盘上自证过、实例代号相同 ⇒ 反向链 = 它的头算出来的链值；
+/// ③ 计数器 − 1 那一条自证过而实例代号不同 ⇒ 这一条是本实例写出的第一条（一个实例的计数器连续、接着上一个实例走，
+///    D23（journal 的角色与格式） 已定项 19 注 3）⇒ 反向链恒 0。**跨实例边界的两条记录之间不比链值**——I-8.6 明写不判那一格。
+/// 计数器 − 1 那一条读不出、自证不过、或环转过一圈之后那一格坐着上一圈的记录（计数器对不上）⇒ 本实例内逻辑前一条不在盘上，
+/// 这一条判不了，跳过。
+fn judge_journal_back_chain(
+    reader: &dyn ImageReader,
+    geometry: &PoolGeometry,
+    filesystem_identifier_low: u64,
+    judgements: &mut Judgements,
+) {
+    let mut judged_records = 0u64;
+    for device in reader.devices() {
+        let records =
+            scanned_journal_records_of_device(reader, geometry, device, filesystem_identifier_low);
+        for (counter, record) in &records {
+            let expected_back_chain = if *counter == FIRST_JOURNAL_COUNTER {
+                Some(0)
+            } else {
+                match records.get(&(counter - 1)) {
+                    Some(previous) if previous.instance == record.instance => {
+                        Some(previous.chain_value_the_next_record_must_carry)
+                    }
+                    Some(_record_of_another_instance) => Some(0),
+                    None => None,
+                }
+            };
+            let Some(expected_back_chain) = expected_back_chain else {
+                continue;
+            };
+            judged_records += 1;
+            judgements.judge("I-8.6", record.back_chain == expected_back_chain, || {
+                format!(
+                    "盘 {device} journal 环槽 {} 的记录（实例 {}、计数器 {counter}）反向链 {:#010x}，本实例内逻辑前一条的头算出来的是 {expected_back_chain:#010x}",
+                    record.slot, record.instance, record.back_chain
+                )
+            });
+        }
+    }
+    if judged_records == 0 {
+        judgements.not_applicable(
+            "I-8.6",
+            "环里没有一条自证过的记录找得到本实例内逻辑前一条：链判不了",
+        );
+    }
+}
+
 /// 池级 checker 的入口：每条第一版不变量都报，没评估到的报「不适用」并带理由。
 #[must_use]
 pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {
@@ -1301,11 +1793,16 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
                 .map(|(view, geometry)| (*device, view, geometry))
         })
         .collect();
-    if chosen.is_empty() || chosen.len() != system_configurations.len() {
+    // 一块盘的两个系统配置槽都无效时**不早退**：恢复会改用别的盘上那一份、照常挂上
+    // （`crates/singlefs-core/src/recovery.rs` 的 `choose_system_configuration`，D22（单元原子性怎么合成） 已定项 8
+    // 「系统配置每盘放一份」买的就是这份冗余）。早退会让这种镜像上每一条不变量都报「不适用」，
+    // 而它是一个挂得上的合法镜像——故障注入让一块盘的两个槽先后写失败就造得出它，判定会被静默放过
+    // （.claude/kb/checks-owed.md 的 C461）。⚠️ 「哪块盘的系统配置全废了」今天没有编号报得出来，仍欠在 C461。
+    if chosen.is_empty() {
         for invariant in crate::image::IMPLEMENTED_INVARIANTS {
             root_ring_judgements.not_applicable(
                 invariant,
-                "有一块盘两个系统配置槽都无效：这不是一个挂得上的镜像",
+                "池里没有一块盘交得出有效的系统配置：这不是一个挂得上的镜像",
             );
         }
         return root_ring_judgements.into_report();
@@ -1344,14 +1841,31 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
         distinct.len() == regions.min(devices.len()) && heaviest <= pigeonhole && region_devices.iter().all(|device| devices.contains(device)),
         || format!("根环区域的设备 {region_devices:?}：不同值 {}、最重的盘背 {heaviest} 个（上界 {pigeonhole}）", distinct.len()),
     );
+    let filesystem_identifier_low = u64::from_le_bytes(
+        geometry.filesystem_identifier[..8]
+            .try_into()
+            .expect("8 字节"),
+    );
     let roots = valid_roots(reader, &geometry);
     judge_instance_carriers(reader, &geometry, &roots, &mut root_ring_judgements);
+    // I-8.6 只读 journal 环，不读根：根环全灭的镜像上它照样判得了（那一格归 I-7.1）。
+    judge_journal_back_chain(
+        reader,
+        &geometry,
+        filesystem_identifier_low,
+        &mut root_ring_judgements,
+    );
     root_ring_judgements.judge("I-7.1", !roots.is_empty(), || {
         "根环里一条自证过的根都没有".to_string()
     });
     if roots.is_empty() {
+        root_ring_judgements.not_applicable(
+            "I-7.3",
+            "根环里一条自证过的根都没有：S 空，没有「代号最大者」可谈",
+        );
         return root_ring_judgements.into_report();
     }
+    judge_root_ring_health(&roots, &mut root_ring_judgements);
     let newest_index = roots
         .iter()
         .enumerate()
@@ -1361,11 +1875,7 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
     let mut walk = Walk {
         reader,
         judgements: root_ring_judgements,
-        filesystem_identifier_low: u64::from_le_bytes(
-            geometry.filesystem_identifier[..8]
-                .try_into()
-                .expect("8 字节"),
-        ),
+        filesystem_identifier_low,
         references: BTreeMap::new(),
         visited_units: BTreeSet::new(),
         walk_failures: Vec::new(),
@@ -1411,8 +1921,8 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
         .iter()
         .enumerate()
         .filter(|(index, (_, _, root))| {
-            let abandoned = instance_table_rows.iter().any(|(row_instance, row_txg)| {
-                *row_instance == root.instance && root.checkpoint_txg > *row_txg
+            let abandoned = instance_table_rows.iter().any(|row| {
+                row.instance == root.instance && root.checkpoint_txg > row.published_checkpoint_txg
             });
             let below_floor = root.checkpoint_txg < newest_rollback_floor;
             let walked = *index == newest_index || (!abandoned && !below_floor);
@@ -1456,6 +1966,21 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
     judgements.judge("I-7.2", newest_failures.is_empty(), || {
         format!("最新的根走不完：{}", newest_failures.join("；"))
     });
+    // I-1.8：扫描方向按已发布谓词过滤之后归并成组。谓词要挂载根的 (实例代号, txg) 与它指着的那一版实例表，
+    // 所以这一遍排在走读之后（`instance_table_rows` 是走最新根时取的）。
+    let published = PublishedPredicate {
+        mount_root_instance: roots[newest_index].2.instance,
+        mount_root_checkpoint_txg: roots[newest_index].2.checkpoint_txg,
+        instance_table_rows: &instance_table_rows,
+    };
+    let content_units =
+        scanned_content_units(reader, &geometry, filesystem_identifier_low, &published);
+    if judge_merged_version_total_order(reader, &content_units, &mut judgements) == 0 {
+        judgements.not_applicable(
+            "I-1.8",
+            "扫描方向没有两份归并到一起、或两组归并到同一个 key 的码 1 / 码 3 已发布单元：归并与定序都比不出来",
+        );
+    }
     // 这两遍各自按候选集里每条根读树表与树节点：共用一份按位置条目记的缓存，候选根常指着同一个单元，层 0 每个崩溃状态都跑。
     let mut index_node_cache = IndexNodeCache::new();
     judge_release_generation_and_tree_table_birth(
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index d8274c0..6834b76 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -692,6 +692,8 @@ pub fn raise_rollback_floor<Device: BlockDevice>(
                 txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
                 counter: current.record.counter + 1,
                 transaction: 0,
+                highest_transaction_number_before_this_publish: current
+                    .highest_transaction_number_in_this_instance,
                 instance: current.root.instance,
                 back_chain: back_chain_of(&current.record_bytes),
                 file: None,
@@ -745,6 +747,8 @@ fn publish_rows_on_file_version<Device: BlockDevice>(
             txg: identity.txg,
             counter: identity.counter,
             transaction: 0,
+            // 新实例的第一条记录（反向链恒 0），事务号按实例各算各的，从这里重新从 1 起。
+            highest_transaction_number_before_this_publish: 0,
             instance: identity.instance,
             back_chain: 0,
             file: None,
@@ -771,6 +775,8 @@ fn publish_empty_after<Device: BlockDevice>(
                 txg: CheckpointTxg(current_file_version.root.checkpoint_txg.0 + 1),
                 counter: current_file_version.record.counter + 1,
                 transaction: 0,
+                highest_transaction_number_before_this_publish: current_file_version
+                    .highest_transaction_number_in_this_instance,
                 instance,
                 back_chain: back_chain_of(&current_file_version.record_bytes),
                 file: None,
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index f0f2e82..3f8a9e1 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -115,8 +115,14 @@ impl<Device: BlockDevice> PoolReader for Vec<(DeviceIdentity, Device)> {
 /// 恢复走不下去的原因，按调用方能据以行动的粒度分。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum RecoveryFailure {
-    /// 某块盘两个系统配置槽都无效。
-    NoValidSystemConfiguration { device: DeviceIdentity },
+    /// 池里没有一块盘交得出有效的系统配置：每块盘的两个槽都无效。
+    /// 只有一块盘两个槽全废、别的盘还交得出一份时**不报这个**——系统配置每盘放一份买的就是这份冗余
+    /// （D22（单元原子性怎么合成） 已定项 8 第 1 条）。
+    /// 带的是 [`PoolReader::device_identities`] 次序里第一块两个槽都无效的盘；这个成员报出来时池里每块盘都是这样，
+    /// 所以它就是次序里的第一块盘。
+    NoValidSystemConfiguration {
+        first_device_with_no_valid_system_configuration_slot: DeviceIdentity,
+    },
     /// 各盘的系统配置 fsid 或设备数对不上。
     SystemConfigurationsDisagree,
     /// 根环里一条自证过的根都没有。
@@ -217,11 +223,18 @@ fn read_unit_via_locations(
 }
 
 /// 每盘两槽：槽 0 在偏移 0；槽 1 的偏移按槽 0 里记的槽距，槽 0 无效时按最小槽距 4096 试。
+/// 一块盘两个槽都无效时**跳过这块盘**，接着看别的盘：系统配置每盘放一份买的就是这份冗余
+/// （D22（单元原子性怎么合成） 已定项 8 第 1 条；E87 的 8 种失效组合里「掉了那块盘」这一格要可挂）。
+///
+/// # Errors
+/// 池里每块盘的两个槽都无效（[`RecoveryFailure::NoValidSystemConfiguration`]）；
+/// 交得出系统配置的几块盘之间 fsid 或设备数对不上、或者池里一块盘都没有（[`RecoveryFailure::SystemConfigurationsDisagree`]）。
 pub fn choose_system_configuration(
     reader: &dyn PoolReader,
 ) -> Result<SystemConfiguration, RecoveryFailure> {
     let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
     let mut chosen: Option<SystemConfiguration> = None;
+    let mut first_device_with_no_valid_system_configuration_slot: Option<DeviceIdentity> = None;
     for device in reader.device_identities() {
         let slot_zero = reader
             .read(device, DeviceOffsetInBytes(0), slot_bytes)
@@ -241,7 +254,13 @@ pub fn choose_system_configuration(
             .read(device, DeviceOffsetInBytes(spacing), slot_bytes)
             .and_then(|bytes| SystemConfiguration::parse_slot(&bytes));
         let best_on_device = match (slot_zero, slot_one) {
-            (None, None) => return Err(RecoveryFailure::NoValidSystemConfiguration { device }),
+            (None, None) => {
+                // 这块盘上的两份都废了，但别的盘各自还带着一份完整的系统配置：跳过它，别让整池挂不上。
+                if first_device_with_no_valid_system_configuration_slot.is_none() {
+                    first_device_with_no_valid_system_configuration_slot = Some(device);
+                }
+                continue;
+            }
             (Some(only), None) | (None, Some(only)) => only,
             (Some(zero), Some(one)) => {
                 if one.quantities.slot_generation > zero.quantities.slot_generation {
@@ -263,7 +282,16 @@ pub fn choose_system_configuration(
             }
         }
     }
-    chosen.ok_or(RecoveryFailure::SystemConfigurationsDisagree)
+    if let Some(system_configuration) = chosen {
+        return Ok(system_configuration);
+    }
+    match first_device_with_no_valid_system_configuration_slot {
+        Some(device) => Err(RecoveryFailure::NoValidSystemConfiguration {
+            first_device_with_no_valid_system_configuration_slot: device,
+        }),
+        // 池里一块盘都没有：没有哪一块盘点得出名，保持这个函数原先对空池的判定不变（见报告里的设计问题）。
+        None => Err(RecoveryFailure::SystemConfigurationsDisagree),
+    }
 }
 
 /// 根环三个区域全部槽里自证过的根，逐个交给 `visit`（择根与取号共用这一段遍历）。
@@ -678,6 +706,7 @@ pub fn rebuild_version(
             instance_table_bytes,
         ),
     ];
+    let transaction_number_on_the_record_standing_for_this_root = record.transaction;
     Ok(RebuiltVersion::WithFile(TransactionOutput {
         root: *root,
         record,
@@ -694,6 +723,12 @@ pub fn rebuild_version(
         released: Vec::new(),
         key_order_mismatches: 0,
         writes: WritesByStructureKind::NOTHING_WRITTEN,
+        // 重建出来的这一版属于**旧**实例。今天每条恢复路径（普通挂载、回退、切换）之后都要取新实例代号，
+        // 而事务号按实例各算各的、从 1 重新起（D23（journal 的角色与格式） 已定项 7），所以这个值不会被拿去接着发布——
+        // 写行那次发布显式传 0。将来真要在同一个实例上续发，得由调用方扫环算出这个实例的最大非 0 事务号传进来，
+        // 这里这条记录上的事务号在它是空发布时是 0，单独拿它续号会重号。
+        highest_transaction_number_in_this_instance:
+            transaction_number_on_the_record_standing_for_this_root,
     }))
 }
 
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index a1de31d..b63b5d8 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -792,6 +792,9 @@ pub struct TransactionOutput {
     pub key_order_mismatches: u64,
     /// 这次发布交给设备的写，按结构种类（增补 1）；从盘上重建的版本不是这个进程写出的，是空账。
     pub writes: WritesByStructureKind,
+    /// 这个实例到这次发布为止用过的最大事务号（空发布写 0、不推进它）。下一次发布取它加一，
+    /// 而不是取上一条记录上的事务号加一（D23（journal 的角色与格式） 已定项 7：事务号按实例计数、从 1 起）。
+    pub highest_transaction_number_in_this_instance: u64,
 }
 
 impl TransactionOutput {
@@ -1077,6 +1080,13 @@ pub struct PublishPlan<'content> {
     pub counter: u64,
     /// 事务号，按实例计数从 1 起，空发布写 0（D23（journal 的角色与格式） 已定项 7 / 已定项 19 ①）。
     pub transaction: u64,
+    /// 这个实例在这次发布之前用过的最大事务号。`publish_version` 拿它与 `transaction` 取大的存进
+    /// `TransactionOutput`，下一次发布从那里加一。
+    ///
+    /// 为什么不直接取上一条记录的事务号加一：空发布在记录上写 0（已定项 19 ①），取「上一条记录 + 1」会让计数
+    /// 退回 1，同一个实例的事务号重号，而 D23（journal 的角色与格式） 已定项 7 逐字要求「事务号按实例计数、从 1 起」，
+    /// 并且实例表行的 W 能当精确前缀正是靠这条纪律。
+    pub highest_transaction_number_before_this_publish: u64,
     pub instance: InstanceGeneration,
     /// 反向链：上一条记录头的 CRC；本实例的第一条恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
     pub back_chain: u32,
@@ -1191,6 +1201,8 @@ pub fn publish_first_file<Device: BlockDevice>(
             txg,
             counter: previous_counter + 1,
             transaction: FIRST_TRANSACTION_NUMBER,
+            // 这个实例的第一个事务：在它之前只有暖机那两次空发布，事务号都是 0。
+            highest_transaction_number_before_this_publish: 0,
             instance,
             back_chain: back_chain_of(previous_record_bytes),
             file: Some(FileVersionPlan {
@@ -1232,7 +1244,9 @@ pub fn publish_overwrite<Device: BlockDevice>(
         PublishPlan {
             txg,
             counter: previous.record.counter + 1,
-            transaction: previous.record.transaction + 1,
+            transaction: previous.highest_transaction_number_in_this_instance + 1,
+            highest_transaction_number_before_this_publish: previous
+                .highest_transaction_number_in_this_instance,
             instance,
             back_chain: back_chain_of(&previous.record_bytes),
             file: Some(FileVersionPlan {
@@ -2173,6 +2187,9 @@ fn publish_admitted<Device: BlockDevice>(
         mapped_units,
         released: release.to_vec(),
         key_order_mismatches,
+        highest_transaction_number_in_this_instance: plan
+            .highest_transaction_number_before_this_publish
+            .max(plan.transaction),
         writes: pool
             .writes_by_structure_kind
             .since(&writes_before_this_publish),
```
