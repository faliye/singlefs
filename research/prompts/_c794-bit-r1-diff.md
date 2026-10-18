# C794 盘上隔离位：实改补丁全文（2026-10-15）

来源：`~/.local/share/singlefs/work/c794-isolation-bit-measure/patch.diff`（sha256 7493e5015222b1a2…），底子是主工作区 2026-10-15 的样子，不合入。

```diff
diff --git a/crates/singlefs-checker-tier/src/bad_disk_input.rs b/crates/singlefs-checker-tier/src/bad_disk_input.rs
index c193882..4ce7058 100644
--- a/crates/singlefs-checker-tier/src/bad_disk_input.rs
+++ b/crates/singlefs-checker-tier/src/bad_disk_input.rs
@@ -134,10 +134,10 @@ const STATISTIC_LABEL_NOT_IN_THE_REGISTRY: u16 = 60_000;
 
 /// 「分配记录的槽号落在单元区之外」那一条坏法把槽号改成这个数（远在 `UNIT_AREA_START_SLOT` 之下，普查 R6）。
 const SLOT_FAR_BELOW_THE_UNIT_AREA: u64 = 1024;
-/// 分配记录里跨度那个 u16 的最高位是「已释放」标志（`singlefs-core` 的 `ALLOCATION_RECORD_RELEASED_FLAG`），
-/// 低 15 位才是跨度。「跨度越过单元区末尾」那一条坏法把整个字段写成这个数：跨度取低 15 位的最大值、已释放位清掉
-/// （还着的记录才会被重建拿去 `mark_allocated`，普查 R6 / R7）。
-const SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED: u16 = 0x7FFF;
+/// 分配记录里跨度那个 u16 的最高位是「已释放」标志（`singlefs-core` 的 `ALLOCATION_RECORD_RELEASED_FLAG`）、次高位是隔离标志
+/// （`ALLOCATION_RECORD_QUARANTINED_FLAG`，C794 盘上隔离位），低 14 位才是跨度。「跨度越过单元区末尾」那一条坏法把整个字段写成这个数：
+/// 跨度取低 14 位的最大值、两个标志位都清掉（还着的记录才会被重建拿去 `mark_allocated`，普查 R6 / R7）。
+const SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED: u16 = 0x3FFF;
 /// 跨度那个 u16 里的「已释放」位（同上，`singlefs-core` 的 `ALLOCATION_RECORD_RELEASED_FLAG`）。
 const SPAN_FIELD_RELEASED_BIT: u16 = 0x8000;
 /// 「只在第二块盘上写成已释放」那一条坏法把释放代改成这个数（普查 R10）：回收判定是「释放代 ≤ max(F_生效, 环里最旧有效根)」，
@@ -2120,6 +2120,7 @@ pub fn release_check_of_the_instance_table_placement_released_on_the_second_devi
             span_slots,
             generation: CheckpointTxg(newest_root.checkpoint_txg),
             is_released: false,
+            is_quarantined: false,
         },
         AllocationRecord {
             device: second_location.device,
@@ -2127,6 +2128,7 @@ pub fn release_check_of_the_instance_table_placement_released_on_the_second_devi
             span_slots,
             generation: CheckpointTxg(RELEASE_GENERATION_ABOVE_EVERY_ROOT),
             is_released: true,
+            is_quarantined: false,
         },
     ];
     let devices = image
@@ -2399,7 +2401,7 @@ enum AllocationRecordDamage {
     InstanceTablePlacementReleasedOnTheSecondDeviceOnly,
 }
 
-/// 把「根槽 → 树表 → 分配记录树根 → …… → 最左那片叶」这条链上那片叶的第一条分配记录改成「起点贴着单元区末尾、跨度 32767 槽」
+/// 把「根槽 → 树表 → 分配记录树根 → …… → 最左那片叶」这条链上那片叶的第一条分配记录改成「起点贴着单元区末尾、跨度 16383 槽」
 /// （越过单元区末尾，普查 R6 / R7），并把链上的校验和逐道重算：叶两道 → 上一层第 0 条条目子指针里的整单元校验和、那一层两道 →
 /// …… → 分配记录树根两道 → 树表条目里那条根指针的整单元校验和 → 树表节点两道 → 根槽里树表指针的整单元校验和 → 根槽的自证校验和。
 ///
@@ -2410,7 +2412,7 @@ enum AllocationRecordDamage {
 /// **字节由调用方从盘上读来、改完自己写回去**：`allocation_tree_nodes_from_the_root` 是从根起、每层沿第 0 条条目往下读到的那一路
 /// （根在前、叶在末）。坏盘输入那一路读的是 [`MemoryPool`]，步 4 回退那一路读的是录制设备，两边共用这一份，偏移与重算口径只有这一处
 /// （`code-discipline.md`「重复要生成，不许手抄」）。跨度字段整个写成
-/// [`SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED`]：跨度取低 15 位的最大值、已释放位跟着清掉
+/// [`SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED`]：跨度取低 14 位的最大值、已释放位与隔离位跟着清掉
 /// （只有还着的记录才会被影子账拿去隔离）。
 ///
 /// 交回「坏在哪」那句话；树表里没有分配记录树、这一路的末一个不是叶、中间有一层没有条目、或者叶里一条记录都没有时交回 `None`。
@@ -2481,7 +2483,7 @@ pub fn move_the_first_allocation_record_past_the_end_of_the_unit_area_and_reseal
     reseal_wide_checksum_field(root_slot, cover_end, ROOT_SELF_CHECKSUM_OFFSET);
 
     Some(format!(
-        "分配器：这条根那棵账里第一条分配记录的槽号 {old_slot} → {last_slot_of_the_unit_area}（单元区末尾是 {unit_area_end_slot}）、跨度字段 {old_span_field:#06x} → {SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED:#06x}（跨度 {} → 32767 槽，已释放位清掉）；链上每一道校验和重算",
+        "分配器：这条根那棵账里第一条分配记录的槽号 {old_slot} → {last_slot_of_the_unit_area}（单元区末尾是 {unit_area_end_slot}）、跨度字段 {old_span_field:#06x} → {SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED:#06x}（跨度 {} → 16383 槽，已释放位与隔离位清掉）；链上每一道校验和重算",
         old_span_field & SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED
     ))
 }
@@ -2562,8 +2564,8 @@ fn damage_an_allocation_record_in_the_leaf(
                 &SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED.to_le_bytes(),
             );
             format!(
-                "分配器：第一条分配记录的跨度字段 {old:#06x} → {SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED:#06x}（跨度 {} → 32767 槽，已释放位清掉）",
-                old & 0x7FFF
+                "分配器：第一条分配记录的跨度字段 {old:#06x} → {SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED:#06x}（跨度 {} → 16383 槽，已释放位与隔离位清掉）",
+                old & SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED
             )
         }
         AllocationRecordDamage::SameSlotAsTheSecondRecord => {
diff --git a/crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs b/crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs
index dbcaa0d..11d376e 100644
--- a/crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs
+++ b/crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs
@@ -5745,6 +5745,7 @@ mod tests {
                 span_slots: 1,
                 generation: txg,
                 is_released: true,
+                is_quarantined: false,
             },
             AllocationRecord {
                 device: DeviceIdentity(0),
@@ -5752,6 +5753,7 @@ mod tests {
                 span_slots: 1,
                 generation: txg,
                 is_released: false,
+                is_quarantined: false,
             },
         ];
         let allocator = PoolAllocator::rebuild_from_records(devices, records);
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index fcf3085..4b89744 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -2020,7 +2020,7 @@ fn w_walk_allocation_record_tree(root_pointer: Ptr, tree: vec2<u32>, fsid_low: v
                     let o = entry_offset(n, e);
                     let device = rd32(n.base, o);
                     let slot = rd48(n.base, o + 4u);
-                    let span = rd16(n.base, o + 10u) & 0x7FFFu;
+                    let span = rd16(n.base, o + 10u) & ALLOCATION_RECORD_SPAN_VALUE_MASK;
                     var fits = !pending_node.is_root && device == pending_node.device && span > 0u && le64(leaf_first, slot);
                     if (fits) {
                         fits = le64(add64(slot, u64_of(span - 1u)), leaf_last);
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
index f73eda7..c0551f0 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -39,7 +39,6 @@ var<private> c_seen_node_walked_to_its_leaves: array<u32, 8>;
 
 var<private> c_examined_count: u32;
 
-var<private> c_harvested: bool;
 
 var<private> c_exempted: array<u32, 8>;
 
@@ -169,7 +168,7 @@ fn r_seen_insert(device: u32, slot: vec2<u32>) -> bool {
     return true;
 }
 
-fn c_append_allocation_record(device: u32, slot: vec2<u32>, span: u32, generation: vec2<u32>, released: u32) -> bool {
+fn c_append_allocation_record(device: u32, slot: vec2<u32>, span: u32, generation: vec2<u32>, released: u32, quarantined: u32) -> bool {
     if (c_alloc_record_count >= C_MAX_ALLOC_RECORDS) {
         undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (20u << 8u);
         return false;
@@ -182,7 +181,7 @@ fn c_append_allocation_record(device: u32, slot: vec2<u32>, span: u32, generatio
     sws(row + 4u, generation.x);
     sws(row + 5u, generation.y);
     sws(row + 6u, released);
-    sws(row + 7u, 0u);
+    sws(row + 7u, quarantined);
     c_alloc_record_count = c_alloc_record_count + 1u;
     return true;
 }
@@ -230,7 +229,8 @@ fn r_allocation_tree(base: u32, offset: u32) -> bool {
             for (var e = 0u; e < n.entry_count; e = e + 1u) {
                 let o = entry_offset(n, e);
                 let span_field = rd16(n.base, o + 10u);
-                if (!c_append_allocation_record(rd32(n.base, o), rd48(n.base, o + 4u), span_field & 0x7FFFu, rd64(n.base, o + 12u), span_field >> 15u)) {
+                if (!c_append_allocation_record(rd32(n.base, o), rd48(n.base, o + 4u), span_field & ALLOCATION_RECORD_SPAN_VALUE_MASK, rd64(n.base, o + 12u),
+                        (span_field >> ALLOCATION_RECORD_RELEASED_BIT) & 1u, (span_field >> ALLOCATION_RECORD_QUARANTINED_BIT) & 1u)) {
                     ok = false;
                     break;
                 }
@@ -1582,77 +1582,8 @@ fn judge_rollback_floor_raises_against_their_ceilings() {
     }
 }
 
-// 位置项里指着 (盘, 槽) 的校验和：走过的版本收到的，查不到时把根环里没走过的根整遍走一次补进来（只补这一次）
-fn c_location_entry_checksum_known(device: u32, slot: vec2<u32>, fsid_low: vec2<u32>) -> bool {
-    for (var i = 0u; i < w_checksum_count; i = i + 1u) {
-        if (sw(C_CHECKSUMS + i * C_CHECKSUM_WORDS) == device && sw(C_CHECKSUMS + i * C_CHECKSUM_WORDS + 1u) == slot.x && sw(C_CHECKSUMS + i * C_CHECKSUM_WORDS + 2u) == slot.y) {
-            return true;
-        }
-    }
-    // 根环里没走过的根那一遍走读内核已经走过（`hand_off_to_the_scan_kernel`）、位置项校验和在 C_HARVEST；
-    // 与 CPU 同一个懒法：第一次查不到才开始用它
-    if (!c_harvested) {
-        c_harvested = true;
-    }
-    for (var i = 0u; i < w_harvest_count; i = i + 1u) {
-        if (sw(C_HARVEST + i * C_CHECKSUM_WORDS) == device && sw(C_HARVEST + i * C_CHECKSUM_WORDS + 1u) == slot.x && sw(C_HARVEST + i * C_CHECKSUM_WORDS + 2u) == slot.y) {
-            return true;
-        }
-    }
-    return false;
-}
-
-fn c_location_entry_checksum_contains(device: u32, slot: vec2<u32>, checksum: u32) -> bool {
-    for (var i = 0u; i < w_checksum_count; i = i + 1u) {
-        if (sw(C_CHECKSUMS + i * C_CHECKSUM_WORDS) == device && sw(C_CHECKSUMS + i * C_CHECKSUM_WORDS + 1u) == slot.x && sw(C_CHECKSUMS + i * C_CHECKSUM_WORDS + 2u) == slot.y
-            && sw(C_CHECKSUMS + i * C_CHECKSUM_WORDS + 3u) == checksum) {
-            return true;
-        }
-    }
-    if (!c_harvested) {
-        return false;
-    }
-    for (var i = 0u; i < w_harvest_count; i = i + 1u) {
-        if (sw(C_HARVEST + i * C_CHECKSUM_WORDS) == device && sw(C_HARVEST + i * C_CHECKSUM_WORDS + 1u) == slot.x && sw(C_HARVEST + i * C_CHECKSUM_WORDS + 2u) == slot.y
-            && sw(C_HARVEST + i * C_CHECKSUM_WORDS + 3u) == checksum) {
-            return true;
-        }
-    }
-    return false;
-}
-
-// 隔离的记录那一份读回来对不对：读得出、整单元 CRC 在指着它的位置项里；没有位置项指着它时退回自证（check_unit）
-fn c_copy_matches_its_location_entry(device: u32, slot: vec2<u32>, span: u32, fsid_low: vec2<u32>) -> bool {
-    let s = read_at(device, slot_to_bytes(slot), span << SLOT_SHIFT);
-    if (s.version == NONE) {
-        return false;
-    }
-    var checksum = s.c0;
-    if (span == 2u) {
-        checksum = s.c1;
-    }
-    if (c_location_entry_checksum_known(device, slot, fsid_low)) {
-        return c_location_entry_checksum_contains(device, slot, checksum);
-    }
-    // check_unit：magic、flags、类 1 / 2 / 3、长度按类、头校验和、载荷 CRC、版本、预留位
-    if (rd32(s.base, 0u) != MAGIC_SFSU || rd8(s.base, 7u) != 0u || rd16(s.base, 4u) != 1u) {
-        return false;
-    }
-    let unit_class = rd8(s.base, 6u);
-    var expected_span = 2u;
-    if (unit_class == 2u) {
-        expected_span = 1u;
-    } else if (unit_class != 1u && unit_class != 3u) {
-        return false;
-    }
-    if (span != expected_span) {
-        return false;
-    }
-    return (s.c2 & 11u) == 11u;
-}
-
-// quarantined_slots_exempted_per_device：最新根的分配记录树里未释放、没有根引用起在它那一槽的记录，按单元归组；有一份读回来对不上，整组豁免
-fn c_quarantine_exempted(newest_base: u32, reference_limit: u32, fsid_low: vec2<u32>) {
+// quarantined_slots_exempted_per_device：最新根的分配记录树里未释放、带隔离位、没有根引用起在它那一槽的记录，逐条按它那块盘豁免（C794 盘上隔离位）
+fn c_quarantine_exempted(newest_base: u32, reference_limit: u32) {
     let device_count = hdr(3u);
     for (var d = 0u; d < 8u; d = d + 1u) {
         c_exempted[d] = 0u;
@@ -1665,7 +1596,6 @@ fn c_quarantine_exempted(newest_base: u32, reference_limit: u32, fsid_low: vec2<
     if (!tt.ok || tt.entry_width < 200u) {
         return;
     }
-    var groups = 0u;
     r_noting = false;
     for (var e = 0u; e < tt.entry_count; e = e + 1u) {
         let o = entry_offset(tt, e);
@@ -1680,55 +1610,12 @@ fn c_quarantine_exempted(newest_base: u32, reference_limit: u32, fsid_low: vec2<
             let a = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
             let device = sw(a);
             let slot = vec2<u32>(sw(a + 1u), sw(a + 2u));
-            let span = sw(a + 3u);
-            if (sw(a + 6u) != 0u || c_referenced_start(device, slot, reference_limit)) {
+            if (sw(a + 6u) != 0u || sw(a + 7u) == 0u || c_referenced_start(device, slot, reference_limit)) {
                 continue;
             }
-            var device_index = NONE;
             for (var d = 0u; d < device_count; d = d + 1u) {
                 if (hdr(12u + d) == device) {
-                    device_index = d;
-                }
-            }
-            if (device_index == NONE) {
-                continue;
-            }
-            var found = false;
-            for (var k = 0u; k < groups; k = k + 1u) {
-                let group = C_GROUPS + k * 4u;
-                if (sw(group) == slot.x && sw(group + 1u) == slot.y && sw(group + 2u) == span) {
-                    sws(group + 3u, sw(group + 3u) | (1u << device_index));
-                    found = true;
-                }
-            }
-            if (!found) {
-                if (groups >= C_MAX_GROUPS) {
-                    undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (33u << 8u);
-                    return;
-                }
-                sws(C_GROUPS + groups * 4u, slot.x);
-                sws(C_GROUPS + groups * 4u + 1u, slot.y);
-                sws(C_GROUPS + groups * 4u + 2u, span);
-                sws(C_GROUPS + groups * 4u + 3u, 1u << device_index);
-                groups = groups + 1u;
-            }
-        }
-    }
-    for (var k = 0u; k < groups; k = k + 1u) {
-        let group = C_GROUPS + k * 4u;
-        let slot = vec2<u32>(sw(group), sw(group + 1u));
-        let span = sw(group + 2u);
-        let mask = sw(group + 3u);
-        var some_copy_fails = false;
-        for (var d = 0u; d < device_count; d = d + 1u) {
-            if ((mask & (1u << d)) != 0u && !c_copy_matches_its_location_entry(hdr(12u + d), slot, span, fsid_low)) {
-                some_copy_fails = true;
-            }
-        }
-        if (some_copy_fails) {
-            for (var d = 0u; d < device_count; d = d + 1u) {
-                if ((mask & (1u << d)) != 0u) {
-                    c_exempted[d] = c_exempted[d] + span;
+                    c_exempted[d] = c_exempted[d] + sw(a + 3u);
                 }
             }
         }
@@ -1840,14 +1727,13 @@ fn c_slots_referenced(device: u32, limit: u32) -> vec2<u32> {
     return total;
 }
 
-fn judge_accounting(fsid_low: vec2<u32>) {
+fn judge_accounting() {
     if (!w_accounting_seen) {
         return;
     }
     let device_count = hdr(3u);
     let newest_root = checker_root_of(c_newest_index);
     let written_under_effective_floor = eq64(newest_root.floor, c_effective_floor);
-    c_harvested = false;
     // 两遍同一个调用点：第一遍到走读的引用数为止，存下它的豁免数、算完 F 之下的推迟，第二遍到最新根自己的引用数为止
     var exempted_all: array<u32, 8>;
     for (var exemption_pass = 0u; exemption_pass < 2u; exemption_pass = exemption_pass + 1u) {
@@ -1859,7 +1745,7 @@ fn judge_accounting(fsid_low: vec2<u32>) {
             c_deferred_below_the_floor(newest_root.base);
             reference_limit = c_newest_reference_count;
         }
-        c_quarantine_exempted(newest_root.base, reference_limit, fsid_low);
+        c_quarantine_exempted(newest_root.base, reference_limit);
     }
     for (var d = 0u; d < device_count; d = d + 1u) {
         let device = hdr(12u + d);
@@ -4371,7 +4257,7 @@ fn scan_stage(fsid_low: vec2<u32>) {
     judge_tombstones_and_the_lineage_of_the_versions();
     judge_inode_number_watermark_above_every_published_tombstone();
     judge_rollback_floor_raises_against_their_ceilings();
-    judge_accounting(fsid_low);
+    judge_accounting();
     judge_instance_table_rows_against_the_roots_referencing_them();
 }
 
diff --git a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
index ea89810..bb56d38 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
@@ -489,8 +489,7 @@ fn slice_of_version(version: u32, location: u32) -> Slice {
 // 正推说的「空槽加下一已写槽前 16 KiB」同一形）：CPU 的稀疏读者读出「16 KiB 全零 + 紧接着的下一个落点的前 16 KiB」。GPU 拿全零 16 KiB 那一条的第 8 字
 // （VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES）异或下一个落点那一版的前 16 KiB 的 CRC，拼出整段的 CRC 放进 c1，c2 只留「有 32 KiB」与拼读两位
 // （SLICE_SPLICED_ACROSS_THE_END_OF_A_LOCATION）：字节只有前 16 KiB 是这一读的，按位置条目比 CRC 对得上时交 CPU 判（slice_crc_matches）。
-// 拼读切片后 16 KiB 没有字节（base 指的是全零 16 KiB 那一条版本字节，再往后不是下一个落点的字节），调用方只能比 CRC 或读前 16 KiB
-// （crash_judge_checker_scan.wgsl 的 c_copy_matches_its_location_entry 比 CRC、读的单元头在前 16 KiB 里）。
+// 拼读切片后 16 KiB 没有字节（base 指的是全零 16 KiB 那一条版本字节，再往后不是下一个落点的字节），调用方只能比 CRC 或读前 16 KiB。
 // 下一个落点查不到版本（undecidable 已记）、空段止于盘尾（CPU 越过盘尾读不到）的交 no_slice。「16 KiB 节点加下一槽」不用拼：单元槽落点本来就是
 // 32 KiB 长（盘尾那一槽 16 KiB，CPU 读 32 KiB 越过盘尾），版本字节里后 16 KiB 就是叠上去的下一槽
 fn read_at(device: u32, offset: vec2<u32>, length: u32) -> Slice {
@@ -2320,7 +2319,7 @@ fn read_allocation_tree(root_pointer: Ptr, root_pointer_base: u32, root_pointer_
                     let device = rd32(n.base, o);
                     let slot = rd48(n.base, o + 4u);
                     let span_field = rd16(n.base, o + 10u);
-                    let span = span_field & 0x7FFFu;
+                    let span = span_field & ALLOCATION_RECORD_SPAN_VALUE_MASK;
                     let generation = rd64(n.base, o + 12u);
                     // 落在这片叶里：同盘、叶序号 = 槽 / 812、末槽不越过叶末槽
                     let leaf_index = divmod64(slot, u64_of(ALLOCATION_LEAF_SLOTS)).quotient;
@@ -2332,6 +2331,11 @@ fn read_allocation_tree(root_pointer: Ptr, root_pointer_base: u32, root_pointer_
                         walk_failure_point = FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY;
                         return out;
                     }
+                    // 已释放位与隔离位同时置着：allocation_record_tree 的读者在同一处拒（ALLOCATION_RECORD_RELEASED_AND_QUARANTINED_AT_ONCE_UNDEFINED，C794 盘上隔离位）
+                    if (((span_field >> ALLOCATION_RECORD_RELEASED_BIT) & (span_field >> ALLOCATION_RECORD_QUARANTINED_BIT) & 1u) == 1u) {
+                        walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
+                        return out;
+                    }
                     if (allocation_count >= MAX_ALLOCATION_RECORDS) {
                         undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
                         return out;
@@ -3400,7 +3404,7 @@ fn walk_to_file(root: RootFields, cfg: SysConfig, expected: u32) -> WalkResult {
     for (var r = 0u; r < allocation_count; r = r + 1u) {
         let device = sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS);
         let slot = vec2<u32>(sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS + 1u), sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS + 2u));
-        let span = sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS + 3u) & 0x7FFFu;
+        let span = sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS + 3u) & ALLOCATION_RECORD_SPAN_VALUE_MASK;
         if (!device_is_in_the_pool(device)) {
             return walk_failed(FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY);
         }
@@ -3416,7 +3420,7 @@ fn walk_to_file(root: RootFields, cfg: SysConfig, expected: u32) -> WalkResult {
         if (r > 0u) {
             let previous_device = sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS);
             let previous_slot = vec2<u32>(sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS + 1u), sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS + 2u));
-            let previous_span = sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS + 3u) & 0x7FFFu;
+            let previous_span = sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS + 3u) & ALLOCATION_RECORD_SPAN_VALUE_MASK;
             if (previous_device == device && lt64(slot, add64(previous_slot, u64_of(previous_span)))) {
                 return walk_failed(FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY);
             }
@@ -3457,7 +3461,7 @@ fn walk_to_file(root: RootFields, cfg: SysConfig, expected: u32) -> WalkResult {
     }
     // 分配记录跨度非 0、代不晚于根
     for (var r2 = 0u; r2 < allocation_count; r2 = r2 + 1u) {
-        let span = sw(R_ALLOCATION + r2 * ALLOCATION_RECORD_WORDS + 3u) & 0x7FFFu;
+        let span = sw(R_ALLOCATION + r2 * ALLOCATION_RECORD_WORDS + 3u) & ALLOCATION_RECORD_SPAN_VALUE_MASK;
         let generation = vec2<u32>(sw(R_ALLOCATION + r2 * ALLOCATION_RECORD_WORDS + 4u), sw(R_ALLOCATION + r2 * ALLOCATION_RECORD_WORDS + 5u));
         if (span == 0u || lt64(root.txg, generation)) {
             return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
diff --git a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
index b1303e9..7689d8b 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
@@ -280,11 +280,29 @@ pub fn judge_table_layout_wgsl_constants() -> String {
 #[must_use]
 pub fn kernel_prelude_wgsl_constants() -> String {
     format!(
-        "{}{}{}{}",
+        "{}{}{}{}{}",
         verdict_layout_wgsl_constants(),
         judge_table_layout_wgsl_constants(),
         segment_expansion_code_wgsl_constants(),
-        failure_point_wgsl_constants()
+        failure_point_wgsl_constants(),
+        allocation_record_span_field_wgsl_constants()
+    )
+}
+
+/// 分配记录跨度段（D3（空间分配） 已定项 7 / 已定项 11，盘上 2 字节）里已释放标志那一位的位号：最高位。
+pub const ALLOCATION_RECORD_RELEASED_BIT: u32 = 15;
+/// 同一段里隔离标志那一位的位号：次高位（C794 盘上隔离位，副本里量改动面的那一版）。
+pub const ALLOCATION_RECORD_QUARANTINED_BIT: u32 = 14;
+/// 同一段去掉两个标志之后跨度值的掩码。
+pub const ALLOCATION_RECORD_SPAN_VALUE_MASK: u32 = (1 << ALLOCATION_RECORD_QUARANTINED_BIT) - 1;
+
+/// 分配记录跨度段的解法在内核里的常量（WGSL 源码）：恢复内核与池级 checker 两个内核解跨度段都取这三个，不各写一份魔数。
+#[must_use]
+pub fn allocation_record_span_field_wgsl_constants() -> String {
+    format!(
+        "const ALLOCATION_RECORD_RELEASED_BIT: u32 = {ALLOCATION_RECORD_RELEASED_BIT}u;\n\
+         const ALLOCATION_RECORD_QUARANTINED_BIT: u32 = {ALLOCATION_RECORD_QUARANTINED_BIT}u;\n\
+         const ALLOCATION_RECORD_SPAN_VALUE_MASK: u32 = {ALLOCATION_RECORD_SPAN_VALUE_MASK}u;\n"
     )
 }
 
@@ -1091,6 +1109,31 @@ mod tests {
         );
     }
 
+    /// 分配记录跨度段的三个常量进了内核前面那份常量、各恰好一行；跨度掩码不罩已释放、隔离两个标志位（C794 盘上隔离位）。
+    #[test]
+    fn allocation_record_span_field_constants_reach_the_kernel_prelude_and_the_span_mask_leaves_both_flag_bits_out(
+    ) {
+        let prelude = super::kernel_prelude_wgsl_constants();
+        for line in [
+            "const ALLOCATION_RECORD_RELEASED_BIT: u32 = 15u;",
+            "const ALLOCATION_RECORD_QUARANTINED_BIT: u32 = 14u;",
+            "const ALLOCATION_RECORD_SPAN_VALUE_MASK: u32 = 16383u;",
+        ] {
+            assert_eq!(
+                prelude.matches(line).count(),
+                1,
+                "内核前面那份常量里恰好一行 {line}"
+            );
+        }
+        assert_eq!(
+            super::ALLOCATION_RECORD_SPAN_VALUE_MASK
+                & ((1 << super::ALLOCATION_RECORD_RELEASED_BIT)
+                    | (1 << super::ALLOCATION_RECORD_QUARANTINED_BIT)),
+            0,
+            "跨度掩码不罩两个标志位"
+        );
+    }
+
     /// 派活参数四个字各是什么：序号拆成低、高 32 位，状态数，每个状态的草稿区字数。
     #[test]
     fn dispatch_words_carry_the_first_ordinal_the_state_count_and_the_scratch_size() {
diff --git a/crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs b/crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs
index a68b866..228ee95 100644
--- a/crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs
+++ b/crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs
@@ -2437,34 +2437,34 @@ fn one_crash_before_the_row_root_then_a_mount_keeps_the_allocated_statistic_with
 // ─── harness 档 `tests/release_checksum_quarantine.rs` ───
 
 /// 与 harness 档 `crates/singlefs-harness/tests/release_checksum_quarantine.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
-/// `scenarios_of_release_checksum_quarantine::quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read`）。
+/// `scenarios_of_release_checksum_quarantine::quarantined_record_is_exempted_from_the_allocated_statistics_by_its_quarantine_flag_whether_or_not_its_copies_read_back`）。
 #[test]
-fn quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read(
+fn quarantined_record_is_exempted_from_the_allocated_statistics_by_its_quarantine_flag_whether_or_not_its_copies_read_back(
 ) {
     judge_the_images_a_harness_scenario_hands_out(
-        "quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read",
-        scenarios_of_release_checksum_quarantine::quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read,
+        "quarantined_record_is_exempted_from_the_allocated_statistics_by_its_quarantine_flag_whether_or_not_its_copies_read_back",
+        scenarios_of_release_checksum_quarantine::quarantined_record_is_exempted_from_the_allocated_statistics_by_its_quarantine_flag_whether_or_not_its_copies_read_back,
     );
 }
 
 /// 与 harness 档 `crates/singlefs-harness/tests/release_checksum_quarantine.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
-/// `scenarios_of_release_checksum_quarantine::unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not`）。
+/// `scenarios_of_release_checksum_quarantine::unit_with_one_failing_copy_carries_the_quarantine_flag_on_every_device_and_stays_exempted_once_both_copies_read_back`）。
 #[test]
-fn unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not() {
+fn unit_with_one_failing_copy_carries_the_quarantine_flag_on_every_device_and_stays_exempted_once_both_copies_read_back(
+) {
     judge_the_images_a_harness_scenario_hands_out(
-        "unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not",
-        scenarios_of_release_checksum_quarantine::unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not,
+        "unit_with_one_failing_copy_carries_the_quarantine_flag_on_every_device_and_stays_exempted_once_both_copies_read_back",
+        scenarios_of_release_checksum_quarantine::unit_with_one_failing_copy_carries_the_quarantine_flag_on_every_device_and_stays_exempted_once_both_copies_read_back,
     );
 }
 
 /// 与 harness 档 `crates/singlefs-harness/tests/release_checksum_quarantine.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
-/// `scenarios_of_release_checksum_quarantine::quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_against_the_checksum_in_its_location_entry`）。
+/// `scenarios_of_release_checksum_quarantine::quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_by_its_quarantine_flag`）。
 #[test]
-fn quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_against_the_checksum_in_its_location_entry(
-) {
+fn quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_by_its_quarantine_flag() {
     judge_the_images_a_harness_scenario_hands_out(
-        "quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_against_the_checksum_in_its_location_entry",
-        scenarios_of_release_checksum_quarantine::quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_against_the_checksum_in_its_location_entry,
+        "quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_by_its_quarantine_flag",
+        scenarios_of_release_checksum_quarantine::quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_by_its_quarantine_flag,
     );
 }
 
@@ -3514,8 +3514,8 @@ const SCENARIOS_PAIRED_ON_THE_GPU_AND_THE_CPU: [(&str, ScenarioHandingOutImages)
         scenarios_of_named_unit_verification_stands_in_for_the_barrier_before_the_record::with_the_barrier_before_the_record_a_publish_reusing_reclaimed_slots_keeps_every_single_withheld_state_free_of_checker_violations,
     ),
     (
-        "quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read",
-        scenarios_of_release_checksum_quarantine::quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read,
+        "quarantined_record_is_exempted_from_the_allocated_statistics_by_its_quarantine_flag_whether_or_not_its_copies_read_back",
+        scenarios_of_release_checksum_quarantine::quarantined_record_is_exempted_from_the_allocated_statistics_by_its_quarantine_flag_whether_or_not_its_copies_read_back,
     ),
     (
         "the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries",
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 90c478f..c32b098 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -410,9 +410,6 @@ struct Walk<'reader> {
     filesystem_identifier_low: u64,
     /// (设备, 起点槽, 跨度槽) → 第一次引用它的是谁。
     references: BTreeMap<(u32, u64, u64), String>,
-    /// (设备, 起点槽) → 走过的指针里指向它的那几条位置项带的单元校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1 读盘核比的那一个）：
-    /// 隔离的记录按它判那一份对不对得上（`quarantined_slots_exempted_per_device`）。同一个槽被复用过时几版指着不同的单元，都留着。
-    location_entry_checksums: BTreeMap<(u32, u64), BTreeSet<u32>>,
     /// (设备, 起点槽)：整个走读（各版合起来）走过的单元，再遇到就不再往下走——不同版之间共享没改过的单元是合法的 COW，
     /// 走过一遍就够。同一版里再遇到同一个单元不是共享，那一格由 `placements_referenced_in_this_version` 判。
     visited_units: BTreeSet<(u32, u64)>,
@@ -1157,7 +1154,6 @@ impl<'reader> Walk<'reader> {
             judgements,
             filesystem_identifier_low,
             references: BTreeMap::new(),
-            location_entry_checksums: BTreeMap::new(),
             visited_units: BTreeSet::new(),
             placements_referenced_in_this_version: BTreeMap::new(),
             walk_parent_unit: None,
@@ -1241,10 +1237,6 @@ impl<'reader> Walk<'reader> {
         self.references
             .entry((location.device, location.slot, span))
             .or_insert_with(|| what.to_string());
-        self.location_entry_checksums
-            .entry((location.device, location.slot))
-            .or_default()
-            .insert(location.checksum);
     }
 
     /// 开始走一版（根环里的一条根，或由记录施加出来的一版）：这一版引用过哪些落点从空的记起。
@@ -4254,9 +4246,10 @@ fn judge_instance_carriers(
     }
 }
 
-/// 分配记录 20（D3（空间分配） 已定项 7 / 已定项 11）：key (设备身份 4, 16 KiB 槽号 6) + value (跨度段 2，最高位是已释放标志,
-/// 分配代或释放代 8)。checker 按字段表自己解一份，不与实现共用代码（D13（验证路线） 已定项 5）。
+/// 分配记录 20（D3（空间分配） 已定项 7 / 已定项 11）：key (设备身份 4, 16 KiB 槽号 6) + value (跨度段 2，最高位是已释放标志、
+/// 次高位是隔离标志（C794 盘上隔离位）, 分配代或释放代 8)。checker 按字段表自己解一份，不与实现共用代码（D13（验证路线） 已定项 5）。
 const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;
+const ALLOCATION_RECORD_QUARANTINED_FLAG: u16 = 0x4000;
 const ALLOCATION_RECORD_SPAN_FIELD_OFFSET: usize = 10;
 const ALLOCATION_RECORD_GENERATION_OFFSET: usize = 12;
 /// 树表条目 200（D8（核心索引结构） 已定项 8）：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 +
@@ -4490,11 +4483,13 @@ fn judge_tree_table_entry_head_identifiers(
 struct AllocationRecordView {
     device: u32,
     slot: u64,
-    /// 跨度段去掉已释放标志之后的值：这条记录罩住 [槽号, 槽号 + 跨度) 这些槽。
+    /// 跨度段去掉已释放、隔离两个标志之后的值：这条记录罩住 [槽号, 槽号 + 跨度) 这些槽。
     span_slots: u64,
     /// 仍分配时是分配代；已释放时是释放代（D3（空间分配） 已定项 7）。
     generation: u64,
     is_released: bool,
+    /// 写者释放之前读盘核出这个单元对不上、把记录留在已分配时置的隔离位（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）。
+    is_quarantined: bool,
 }
 
 fn parse_allocation_record(entry: &[u8]) -> AllocationRecordView {
@@ -4502,9 +4497,12 @@ fn parse_allocation_record(entry: &[u8]) -> AllocationRecordView {
     AllocationRecordView {
         device: read_u32(entry, 0),
         slot: read_six_byte_unsigned(entry, 4),
-        span_slots: u64::from(span_field & !ALLOCATION_RECORD_RELEASED_FLAG),
+        span_slots: u64::from(
+            span_field & !(ALLOCATION_RECORD_RELEASED_FLAG | ALLOCATION_RECORD_QUARANTINED_FLAG),
+        ),
         generation: read_u64(entry, ALLOCATION_RECORD_GENERATION_OFFSET),
         is_released: span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0,
+        is_quarantined: span_field & ALLOCATION_RECORD_QUARANTINED_FLAG != 0,
     }
 }
 
@@ -6858,45 +6856,6 @@ fn judge_allocation_record_ranges(
     }
 }
 
-/// 根环里没被走过的那几条根（低于 F 的、被抛弃的）：判隔离的记录缺位置项时才走一遍，只为收它们指针里的位置项。
-struct RootsNotWalked<'reader> {
-    reader: &'reader dyn ImageReader,
-    filesystem_identifier_low: u64,
-    mount_root_instance: u32,
-    mount_root_checkpoint_txg: u64,
-    root_records: Vec<Vec<u8>>,
-}
-
-/// 判隔离的记录要的位置项校验和：(盘, 起点槽) → 指着那一份的各条位置项带的单元校验和。先用走过的那几版收到的；
-/// 查一份没收到时，把根环里没走过的根整遍走一次（判定丢掉，只收位置项）补进来，只补这一次。
-struct LocationEntryChecksumsForQuarantine<'reader> {
-    known: BTreeMap<(u32, u64), BTreeSet<u32>>,
-    roots_not_walked_yet: Option<RootsNotWalked<'reader>>,
-}
-
-impl LocationEntryChecksumsForQuarantine<'_> {
-    fn at(&mut self, device: u32, slot: u64) -> Option<&BTreeSet<u32>> {
-        if !self.known.contains_key(&(device, slot)) {
-            if let Some(roots) = self.roots_not_walked_yet.take() {
-                let mut harvesting_walk = Walk::starting_with(
-                    roots.reader,
-                    Judgements::default(),
-                    roots.filesystem_identifier_low,
-                    roots.mount_root_instance,
-                    roots.mount_root_checkpoint_txg,
-                );
-                for record in &roots.root_records {
-                    harvesting_walk.walk_root(record, false);
-                }
-                for (place, checksums) in harvesting_walk.location_entry_checksums {
-                    self.known.entry(place).or_default().extend(checksums);
-                }
-            }
-        }
-        self.known.get(&(device, slot))
-    }
-}
-
 /// 收口表第 43 行取丁-defer（用户 2026-09-28 定，被攻过一轮；`research/prompts/abandoned-floor-r1-main-verification.md` M1）：
 /// I-3.1（已分配统计对得上） 的「实际遍历」并上这样的槽——F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的分配记录树里
 /// 罩住它的记录是已释放、释放代高于回收门槛 `reclaim_threshold` = max(F_生效, 环里最旧有效根) 的（还在 defer 里，分配器还不许回收，
@@ -7041,22 +7000,18 @@ fn deferred_slots_among_the_references_of_the_selected_roots(
 
 /// 隔离的记录（D19（块指针的结构与宽度预算） 已定项 5：释放之前读盘核出对不上的那个单元，每块盘上的分配记录都留在已分配，
 /// 映射条目去掉、没有任何根再引用它）在 I-3.1（已分配统计对得上） 与 I-3.11（已分配减 defer 等于最新根走读） 上怎么认
-/// （用户 2026-09-25 定；**按单元判**是主 agent 同日定的读法，I-3.1 / I-3.11 两行的措辞随后由书记员改）：已分配而没有根引用的记录，
-/// 它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出、或与指着它的位置项里的单元校验和对不上（与 D19 已定项 5 硬规则 1
-/// 写者读盘核的是同一个判据：实四丙交回 ⑤，丢一次写落在复用槽上时那一份是一个完整的旧单元、自证照样过，按自证判就不豁免），
-/// 这个单元在每块盘上的那几条记录都豁免；这个单元每一份都读得出且对得上，照旧判违例。位置项取走过的每一版里指着那一份的
-/// （[`LocationEntryChecksumsForQuarantine`]），走过的版本都不指着它时再走一遍根环里没走过的根（低于 F 的、被抛弃的）补上。按单元而不按份判：写者只要一份核出对不上就把两块盘的记录一起留在已分配
-/// （D19 已定项 5「另一块盘上那一份对得上也一起留，各盘的账保持对称」），按份判的话对得上那块盘上的那一条恒红。
+/// （C794 盘上隔离位，副本里量改动面的那一版：改用户 2026-09-25「核出坏的才豁免」，条款没改）：已分配而没有根引用的记录，
+/// 跨度段的隔离位置着就豁免、没置照旧判违例——checker 不再自己读那一份比校验和，一次瞬时坏读造成的隔离（两份都完好）与写者漏释放
+/// 只差这一位（`research/prompts/c794-r1-main-verification.md` H2/H3：不改格式时两份镜像逐扇区相同）。按记录判，不按单元归组：
+/// 写者两块盘一起置位（D19 已定项 5「各盘的账保持对称」），只一块盘上带位的那一条豁免、另一条照判。
 ///
-/// 读的是 `root` 那棵分配记录树（树表里种类 3 那一条指着的根，按位置整棵走下去，`allocation_record_tree_without_judging`）：未释放、`referenced_start_slots`
-/// 里没有哪个引用起在它那一槽的记录，按单元（起点槽、跨度——第一版一个单元两盘同槽）归组；一组的每一份按 (盘, 槽, 跨度) 读那一整份单元、
-/// 算整单元 CRC-32C 与位置项里的比——有一份读不出或对不上，整组豁免。逐盘交回豁免的槽数；
+/// 读的是 `root` 那棵分配记录树（树表里种类 3 那一条指着的根，按位置整棵走下去，`allocation_record_tree_without_judging`）：
+/// 未释放、带隔离位、`referenced_start_slots` 里没有哪个引用起在它那一槽的记录，逐盘交回豁免的槽数；
 /// 树表或分配记录树读不出、位置对不上时那一棵一格都不豁免（照旧判）。
 fn quarantined_slots_exempted_per_device(
     reader: &dyn ImageReader,
     root: &crate::RootView,
     referenced_start_slots: &BTreeSet<(u32, u64)>,
-    location_entry_checksums: &mut LocationEntryChecksumsForQuarantine<'_>,
     cache: &mut IndexNodeCache,
 ) -> BTreeMap<u32, u64> {
     let mut exempted: BTreeMap<u32, u64> = BTreeMap::new();
@@ -7068,8 +7023,6 @@ fn quarantined_slots_exempted_per_device(
     else {
         return exempted;
     };
-    // 一个单元（起点槽、跨度）→ 记着它、没有根引用的那几块盘。
-    let mut unreferenced_units: BTreeMap<(u64, u64), Vec<u32>> = BTreeMap::new();
     for entry in &tree_table.entries {
         if entry.len() < tree_table_entry_bytes() || read_u16(entry, 10) != TREE_KIND_ALLOCATION {
             continue;
@@ -7084,38 +7037,13 @@ fn quarantined_slots_exempted_per_device(
             continue;
         };
         for record in records {
-            if record.is_released || referenced_start_slots.contains(&(record.device, record.slot))
+            if record.is_released
+                || !record.is_quarantined
+                || referenced_start_slots.contains(&(record.device, record.slot))
             {
                 continue;
             }
-            unreferenced_units
-                .entry((record.slot, record.span_slots))
-                .or_default()
-                .push(record.device);
-        }
-    }
-    for ((slot, span_slots), devices_of_the_unit) in unreferenced_units {
-        let mut copy_matches_its_location_entry = |device: u32| {
-            let Some(unit) = usize::try_from(span_slots * SLOT_BYTES)
-                .ok()
-                .and_then(|unit_bytes| reader.read(device, slot * SLOT_BYTES, unit_bytes))
-            else {
-                return false;
-            };
-            match location_entry_checksums.at(device, slot) {
-                Some(checksums) => checksums.contains(&crc32_castagnoli_table(&unit)),
-                // 走过的版本与根环里别的根都没有一条位置项指着这一份（引用它的那几版已经转出根环）：D19 已定项 5 的判据没有东西可比，
-                // 这一格条款没写，照改之前的读法退回自证（magic、头校验和、载荷 CRC）——交主 agent（实七报告）。
-                None => check_unit(&unit).is_ok(),
-            }
-        };
-        let some_copy_fails = devices_of_the_unit
-            .iter()
-            .any(|device| !copy_matches_its_location_entry(*device));
-        if some_copy_fails {
-            for device in devices_of_the_unit {
-                *exempted.entry(device).or_insert(0) += span_slots;
-            }
+            *exempted.entry(record.device).or_insert(0) += record.span_slots;
         }
     }
     exempted
@@ -12743,21 +12671,6 @@ pub fn check_pool_image_given_the_container_capacity_knobs(
     );
     let referenced_units_judged_against_their_pointer =
         walk.referenced_units_judged_against_their_pointer;
-    let mut location_entry_checksums = LocationEntryChecksumsForQuarantine {
-        known: std::mem::take(&mut walk.location_entry_checksums),
-        roots_not_walked_yet: Some(RootsNotWalked {
-            reader,
-            filesystem_identifier_low,
-            mount_root_instance: roots[newest_index].2.instance,
-            mount_root_checkpoint_txg: roots[newest_index].2.checkpoint_txg,
-            root_records: roots
-                .iter()
-                .enumerate()
-                .filter(|(index, _)| !candidate_indexes.contains(index))
-                .map(|(_, (_, _, root))| root.record_bytes.clone())
-                .collect(),
-        }),
-    };
     let mut judgements = walk.judgements;
     judgements.judge("I-7.2", newest_failures.is_empty(), || {
         format!("最新的根走不完：{}", newest_failures.join("；"))
@@ -13052,14 +12965,13 @@ pub fn check_pool_image_given_the_container_capacity_knobs(
         .keys()
         .map(|(device, start_slot, _)| (*device, *start_slot))
         .collect();
-    // 隔离的记录（已分配而没有根引用、checker 自己读那一份也读不出或与位置项里的校验和对不上）在这两条上豁免：I-3.1 对全部走过的版本的引用、
+    // 隔离的记录（已分配而没有根引用、跨度段的隔离位置着）在这两条上豁免：I-3.1 对全部走过的版本的引用、
     // I-3.11 对最新根这一遍的引用各算一份（`quarantined_slots_exempted_per_device`）。
     let newest_root_view = &roots[newest_index].2;
     let quarantined_exempted_against_every_walked_version = quarantined_slots_exempted_per_device(
         reader,
         newest_root_view,
         &start_slots_referenced_by_every_walked_version,
-        &mut location_entry_checksums,
         &mut index_node_cache,
     );
     let walked_slots_of_every_walked_version: BTreeSet<(u32, u64)> = walk
@@ -13110,7 +13022,6 @@ pub fn check_pool_image_given_the_container_capacity_knobs(
         reader,
         newest_root_view,
         &start_slots_referenced_by_the_newest_root,
-        &mut location_entry_checksums,
         &mut index_node_cache,
     );
     // 最新根的记账是按它自己带的 F 记的（抬 F 的回收在写第一条带新 F 的根之前做，`mount::raise_rollback_floor`）：它带的 F 低于
@@ -13690,6 +13601,7 @@ mod tests {
                         span_slots: 1,
                         generation,
                         is_released,
+                        is_quarantined: false,
                     },
                     reclaim_threshold,
                 )
@@ -13841,6 +13753,7 @@ mod tests {
             span_slots,
             generation,
             is_released,
+            is_quarantined: false,
         }
     }
 
diff --git a/crates/singlefs-core/src/allocation_record_tree.rs b/crates/singlefs-core/src/allocation_record_tree.rs
index b972fa2..72b36a4 100644
--- a/crates/singlefs-core/src/allocation_record_tree.rs
+++ b/crates/singlefs-core/src/allocation_record_tree.rs
@@ -604,6 +604,11 @@ impl AllocationRecordTreeReadFromDisk {
     }
 }
 
+/// 一条分配记录跨度段的已释放位与隔离位同时置着（C794 盘上隔离位）：写者不写这一组合（释放清掉隔离位），而条款没定读到它时怎么办——
+/// 第一版不认，在任何写之前拒绝（可写挂载与冷启动走读都停在这里），盘上不变。GPU 恢复内核同一处报同一个失败点。
+pub const ALLOCATION_RECORD_RELEASED_AND_QUARANTINED_AT_ONCE_UNDEFINED: &str =
+    "分配记录的已释放位与隔离位同时置着：条款没定这一组合，第一版不认";
+
 fn violated(detail: &'static str) -> RecoveryFailure {
     RecoveryFailure::InvariantViolated {
         invariant: "I-1.1",
@@ -739,6 +744,11 @@ impl AllocationRecordTreeReading<'_> {
                         what: "分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽",
                     });
                 }
+                if record.is_released && record.is_quarantined {
+                    return Err(violated(
+                        ALLOCATION_RECORD_RELEASED_AND_QUARANTINED_AT_ONCE_UNDEFINED,
+                    ));
+                }
                 records.push(record);
             }
             let keys_ascend = records[first_new_record..]
@@ -915,6 +925,7 @@ mod tests {
             span_slots,
             generation: CheckpointTxg(1),
             is_released: false,
+            is_quarantined: false,
         }
     }
 
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 1329943..e366c0d 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -28,6 +28,10 @@ use crate::unlink::DeletionSwitches;
 
 /// 跨度段的最高位借作已释放标志（D3（空间分配） 已定项 7 / 已定项 11）：0 = 仍分配、1 = 已释放，不占表达跨度值的位。
 pub const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;
+/// 跨度段的次高位借作隔离标志（C794 盘上隔离位，副本里量改动面的那一版，条款还没改）：只在仍分配的记录上置——
+/// 释放之前读盘核两次都读不出或都对不上、整单元留在已分配的那几条（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1），
+/// 池级 checker 按这一位豁免 I-3.1 / I-3.11。已释放的记录上恒 0；不占表达跨度值的位（跨度第一版恒为 1 或 2）。
+pub const ALLOCATION_RECORD_QUARANTINED_FLAG: u16 = 0x4000;
 
 /// 分配记录 20（D3（空间分配） 已定项 7 / 已定项 11）：key (设备 4, 槽号 6) + value (跨度 2 含已释放标志, 分配代或释放代 8)。
 /// 释放时条目不删、不点删：改写成已释放 + 释放代，留到该落点被重新分配时覆盖（覆盖是步 5 回收接上时的事：
@@ -42,20 +46,27 @@ pub struct AllocationRecord {
     /// 仍分配时是分配代；已释放时是释放代。
     pub generation: CheckpointTxg,
     pub is_released: bool,
+    /// 仍分配、而写者释放之前读盘核出这个单元对不上留下的（[`ALLOCATION_RECORD_QUARANTINED_FLAG`]）；已释放时恒 false。
+    pub is_quarantined: bool,
 }
 
 impl AllocationRecord {
     #[must_use]
     pub fn to_bytes(&self) -> Vec<u8> {
         assert_eq!(
-            self.span_slots & ALLOCATION_RECORD_RELEASED_FLAG,
+            self.span_slots
+                & (ALLOCATION_RECORD_RELEASED_FLAG | ALLOCATION_RECORD_QUARANTINED_FLAG),
             0,
             "跨度值不许占到标志位"
         );
-        let span_field = if self.is_released {
-            self.span_slots | ALLOCATION_RECORD_RELEASED_FLAG
-        } else {
-            self.span_slots
+        assert!(
+            !(self.is_released && self.is_quarantined),
+            "隔离位只在仍分配的记录上：释放把它清掉（PoolAllocator::release_leaving_the_record_allocated_on），读盘时两位都置的记录在分配记录树的读者那里拒掉"
+        );
+        let span_field = match (self.is_released, self.is_quarantined) {
+            (true, _) => self.span_slots | ALLOCATION_RECORD_RELEASED_FLAG,
+            (false, true) => self.span_slots | ALLOCATION_RECORD_QUARANTINED_FLAG,
+            (false, false) => self.span_slots,
         };
         let mut writer = ByteWriter::new(20);
         writer.put_u32(self.device.0);
@@ -90,9 +101,11 @@ impl AllocationRecord {
         Some(Self {
             device,
             slot,
-            span_slots: span_field & !ALLOCATION_RECORD_RELEASED_FLAG,
+            span_slots: span_field
+                & !(ALLOCATION_RECORD_RELEASED_FLAG | ALLOCATION_RECORD_QUARANTINED_FLAG),
             generation,
             is_released: span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0,
+            is_quarantined: span_field & ALLOCATION_RECORD_QUARANTINED_FLAG != 0,
         })
     }
 }
@@ -1836,6 +1849,7 @@ impl PoolAllocator {
                         span_slots: u16::try_from(placement.span).expect("跨度 2 字节"),
                         generation,
                         is_released: false,
+                        is_quarantined: false,
                     });
                 }
             }
@@ -1851,10 +1865,11 @@ impl PoolAllocator {
         self.release_leaving_the_record_allocated_on(placement, release_generation, &[]);
     }
 
-    /// 同 [`Self::release`]，只是 `devices_whose_record_stays_allocated` 那几块盘上的那条分配记录一个字节都不动、留在「已分配」：
+    /// 同 [`Self::release`]，只是 `devices_whose_record_stays_allocated` 那几块盘上的那条分配记录留在「已分配」、置上隔离位
+    /// （[`ALLOCATION_RECORD_QUARANTINED_FLAG`]，C794 盘上隔离位）：
     /// 释放之前按位置项读盘核校验和、核出对不上（读不出也算）的那一份（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-24 定案）——
     /// 那一份的槽不还回空闲池；记录落盘即跨重挂，重挂之后从分配记录树重建的分配器照样认它已分配，准入里照已分配算、不另进式子。
-    /// 逐盘各算：另一块盘上核得上的那一份照常释放。
+    /// 逐盘各算：另一块盘上核得上的那一份照常释放（释放连带清掉隔离位：已释放的记录上恒 0）。
     pub fn release_leaving_the_record_allocated_on(
         &mut self,
         placement: Placement,
@@ -1862,9 +1877,6 @@ impl PoolAllocator {
         devices_whose_record_stays_allocated: &[DeviceIdentity],
     ) {
         for device in &mut self.devices {
-            if devices_whose_record_stays_allocated.contains(&device.device) {
-                continue;
-            }
             let record = self
                 .records
                 .iter_mut()
@@ -1879,7 +1891,12 @@ impl PoolAllocator {
                 placement.span,
                 "释放的跨度与分配记录不符：这块盘的记录跨度由 transaction::placement_registered_unreleased_on_every_device 逐盘判过"
             );
+            if devices_whose_record_stays_allocated.contains(&device.device) {
+                record.is_quarantined = true;
+                continue;
+            }
             record.is_released = true;
+            record.is_quarantined = false;
             record.generation = release_generation;
             device.mark_released(placement.slot, placement.span);
         }
@@ -1896,6 +1913,32 @@ impl PoolAllocator {
         }
     }
 
+    /// 回退目标引用的隔离单元（`mount::roll_back_by_a_forward_publish` 的隔离单元那一判）被这次回退重新引用：每块盘上那条仍分配的记录
+    /// 清掉隔离位（C794 盘上隔离位，条款没写这一格，取最保守的读法——位只留在没有根引用的记录上，活单元的记录不带着旧位，免得它日后
+    /// 漏释放时被这一位豁免掉）。记录的分配代、跨度、位图与计数都不动。
+    ///
+    /// # Panics
+    /// 某块盘上没有起点是这个槽的记录、它是已释放的、或跨度对不上：回退在任何写之前逐盘判过（隔离单元每块盘上都仍分配、同分配代）。
+    pub fn clear_quarantine_of_the_records_of(&mut self, placement: Placement) {
+        for device in &self.devices {
+            let record = self
+                .records
+                .iter_mut()
+                .find(|record| record.device == device.device && record.slot == placement.slot)
+                .expect("隔离单元每块盘上都有记录：回退在任何写之前逐盘判过");
+            assert!(
+                !record.is_released,
+                "隔离单元的记录仍分配：回退在任何写之前逐盘判过"
+            );
+            assert_eq!(
+                u64::from(record.span_slots),
+                placement.span,
+                "隔离单元的跨度与记录相同：回退在任何写之前逐盘判过"
+            );
+            record.is_quarantined = false;
+        }
+    }
+
     /// 管理员回退的复活（D23（journal 的角色与格式） 已定项 14「复活」）：`device` 上起点是这个落点的那条已释放记录改回已分配，
     /// 分配代写回 `allocation_generation`（回退目标那一版账里的分配代）。按盘改：回退逐盘判哪几块盘上的记录是已释放的。
     /// 那一槽还在 defer 窗口里的，从 defer 挪回仍分配；已回收、还没被复用（位图上空着、记录照旧写着已释放）的，重新在位图上标占。
@@ -2857,6 +2900,68 @@ mod tests {
         assert_eq!(pool.devices[1].free_runs(), 4, "两盘同构");
     }
 
+    /// 释放之前读盘核出对不上那几块盘（C794 盘上隔离位）：记录留在已分配、分配代照旧、置上隔离位，跨度段的次高位读回来还是它；
+    /// 别的盘照常释放、不带隔离位。回退重新引用它时清位，记录其余几样不动。
+    #[test]
+    fn quarantined_record_stays_allocated_with_the_quarantine_flag_in_the_second_highest_span_bit_until_cleared(
+    ) {
+        let mut pool = pool_after_mkfs();
+        let data = pool.allocate_user_data(CheckpointTxg(3)).expect("t1");
+        pool.release_leaving_the_record_allocated_on(data, CheckpointTxg(4), &[DeviceIdentity(0)]);
+        let record_on = |allocator_after_the_release: &PoolAllocator, device: DeviceIdentity| {
+            *allocator_after_the_release
+                .record_for(device, data.slot)
+                .expect("释放只改写记录、不删")
+        };
+        let quarantined = record_on(&pool, DeviceIdentity(0));
+        assert_eq!(
+            (
+                quarantined.is_released,
+                quarantined.is_quarantined,
+                quarantined.generation
+            ),
+            (false, true, CheckpointTxg(3)),
+            "核出对不上那块盘：仍分配、带隔离位、分配代照旧"
+        );
+        assert_eq!(
+            u16::from_le_bytes([quarantined.to_bytes()[10], quarantined.to_bytes()[11]]),
+            2 | ALLOCATION_RECORD_QUARANTINED_FLAG,
+            "隔离位进跨度段次高位"
+        );
+        assert_eq!(
+            AllocationRecord::parse(&quarantined.to_bytes()),
+            Some(quarantined),
+            "隔离位读回来还是它，跨度值不含标志位"
+        );
+        let released = record_on(&pool, DeviceIdentity(1));
+        assert_eq!(
+            (released.is_released, released.is_quarantined),
+            (true, false),
+            "另一块盘照常释放、不带隔离位"
+        );
+        let isolated = pool.allocate_user_data(CheckpointTxg(4)).expect("t2");
+        pool.release_leaving_the_record_allocated_on(
+            isolated,
+            CheckpointTxg(5),
+            &[DeviceIdentity(0), DeviceIdentity(1)],
+        );
+        pool.clear_quarantine_of_the_records_of(isolated);
+        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+            assert_eq!(
+                *pool.record_for(device, isolated.slot).expect("记录在"),
+                AllocationRecord {
+                    device,
+                    slot: isolated.slot,
+                    span_slots: 2,
+                    generation: CheckpointTxg(4),
+                    is_released: false,
+                    is_quarantined: false,
+                },
+                "清位只动隔离位：仍分配、分配代照旧"
+            );
+        }
+    }
+
     #[test]
     fn releasing_a_placement_keeps_the_slots_occupied_and_moves_them_into_the_defer_queue() {
         let mut pool = pool_after_mkfs();
@@ -2908,6 +3013,7 @@ mod tests {
                     span_slots: 2,
                     generation: CheckpointTxg(0),
                     is_released: false,
+                    is_quarantined: false,
                 },
                 AllocationRecord {
                     device: DeviceIdentity(0),
@@ -2919,6 +3025,7 @@ mod tests {
                         CheckpointTxg(0)
                     },
                     is_released: released,
+                    is_quarantined: false,
                 },
             ]
         };
@@ -2954,6 +3061,7 @@ mod tests {
                             span_slots: *span_slots,
                             generation: CheckpointTxg(*generation),
                             is_released: *is_released,
+                            is_quarantined: false,
                         },
                     )
             })
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 8699bbe..32d9f0b 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -3368,6 +3368,7 @@ fn format_time_allocator<Device: BlockDevice>(
                 span_slots: u16::try_from(placement.span).expect("单元的跨度是 1 或 2 槽"),
                 generation: CheckpointTxg(0),
                 is_released: false,
+                is_quarantined: false,
             })
         })
         .collect();
@@ -8295,6 +8296,14 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
             copy.allocation_generation,
         );
     }
+    // 回退目标引用的隔离单元这次重新被引用：清掉它每块盘上那条记录的隔离位（C794 盘上隔离位；条款没写，取最保守的读法，
+    // `PoolAllocator::clear_quarantine_of_the_records_of`）。改的记录落在这次发布重写的叶里：发布路径按记录比上一版判哪几片叶变了。
+    for (unit, locations) in &isolated_units_to_check_on_every_device {
+        allocator.clear_quarantine_of_the_records_of(Placement {
+            slot: locations[0].slot,
+            span: unit.span_slots(),
+        });
+    }
     for (_, placement) in &released_user_visible_units {
         let devices_whose_copy_failed_the_checksum: Vec<DeviceIdentity> =
             quarantined_user_visible_copies
@@ -9853,6 +9862,7 @@ mod shadow_ledger_exempts_and_isolates_slot_by_slot_tests {
             span_slots,
             generation: ALLOCATION_GENERATION,
             is_released: false,
+            is_quarantined: false,
         }
     }
 
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 0ea2618..aa87e50 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -4445,7 +4445,7 @@ pub(crate) fn extents_of_the_first_file(
     })
 }
 
-/// 分配记录「每个落点每盘各一条」（两盘同槽、同一批字段）：同一块盘上一个槽只许一条记录，每块盘各自的（槽, 跨度, 代, 已释放）集合相同，
+/// 分配记录「每个落点每盘各一条」（两盘同槽、同一批字段）：同一块盘上一个槽只许一条记录，每块盘各自的（槽, 跨度, 代, 已释放, 隔离）集合相同（隔离位写者两盘一起置，C794 盘上隔离位），
 /// 每盘不少于 `placements_on_each_device_at_least` 个落点（走读传的是这套几何下新池新建文件之后每盘的落点数，
 /// [`placements_on_each_device_after_the_new_pool_file_creation`]）。同盘同槽两条记录（代不同）在集合里是两个元素、两盘对称就过——第二轮攻方腿打中，
 /// 走读自己不判 key 严格递增，这里逐盘核槽号不重复。
@@ -4455,8 +4455,19 @@ pub fn allocation_records_are_one_per_device(
     device_identities: &[DeviceIdentity],
     placements_on_each_device_at_least: PlacementsOnEachDevice,
 ) -> bool {
-    let mut placements_per_device: BTreeMap<DeviceIdentity, BTreeSet<(u64, u16, u64, bool)>> =
-        BTreeMap::new();
+    /// 一条分配记录去掉设备身份之后的样子：各盘比的就是这一份。
+    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+    struct PlacementAsRecordedOnOneDevice {
+        slot: u64,
+        span_slots: u16,
+        generation: u64,
+        is_released: bool,
+        is_quarantined: bool,
+    }
+    let mut placements_per_device: BTreeMap<
+        DeviceIdentity,
+        BTreeSet<PlacementAsRecordedOnOneDevice>,
+    > = BTreeMap::new();
     let mut slots_per_device: BTreeMap<DeviceIdentity, BTreeSet<u64>> = BTreeMap::new();
     for record in records {
         if !slots_per_device
@@ -4469,12 +4480,13 @@ pub fn allocation_records_are_one_per_device(
         placements_per_device
             .entry(record.device)
             .or_default()
-            .insert((
-                record.slot.0,
-                record.span_slots,
-                record.generation.0,
-                record.is_released,
-            ));
+            .insert(PlacementAsRecordedOnOneDevice {
+                slot: record.slot.0,
+                span_slots: record.span_slots,
+                generation: record.generation.0,
+                is_released: record.is_released,
+                is_quarantined: record.is_quarantined,
+            });
     }
     if placements_per_device.len() != device_identities.len()
         || device_identities
@@ -5884,6 +5896,7 @@ mod allocation_records_per_device_tests {
             span_slots: 2,
             generation: CheckpointTxg(3),
             is_released,
+            is_quarantined: false,
         }
     }
 
@@ -6158,7 +6171,7 @@ mod allocation_records_per_device_tests {
 /// 同一块盘上两条记录罩不罩同一个槽。四样各钉一格，加一格合法的。
 ///
 /// 为什么在这一层钉而不是只钉在坏镜像上：跨度上界那一条，坏盘输入那一档的盘是 4 GiB（单元区到槽 262144），
-/// 而那条坏法能写进跨度字段的最大值是 0x7FFF = 32767 槽——从单元区起点起也够不到末尾，
+/// 而那条坏法能写进跨度字段的最大值是 0x3FFF = 16383 槽（跨度段的最高两位借作已释放与隔离标志）——从单元区起点起也够不到末尾，
 /// 它实际打中的是「两条记录罩住同一个槽」。跨度上界那一条因此在镜像那一层零覆盖，只能在这里钉。
 #[cfg(test)]
 mod allocation_record_pool_geometry_tests {
@@ -6211,6 +6224,7 @@ mod allocation_record_pool_geometry_tests {
             span_slots,
             generation: CheckpointTxg(3),
             is_released: false,
+            is_quarantined: false,
         }
     }
 
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 43b23bf..68d0721 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -8139,7 +8139,8 @@ fn publish_admitted<Device: BlockDevice>(
 
     // 释放先于分配（D3（空间分配） 已定项 7）：被换下的落点进 defer 队列、槽仍占着，这次的落点不会落到它们上面。
     // 释放之前读盘核校验和核出对不上（读不出也算）的那几份：逻辑上照样释放（映射条目这次不再写它），那一份在它那块盘上的分配记录
-    // 留在已分配、不改成已释放（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-24 定案），核得上的那一份照常释放。
+    // 留在已分配、不改成已释放（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-24 定案）、置上隔离位（C794 盘上隔离位），
+    // 核得上的那一份照常释放。
     for placement in release {
         let devices_whose_copy_failed_the_checksum: Vec<DeviceIdentity> = quarantine
             .iter()
@@ -8280,7 +8281,8 @@ fn publish_admitted<Device: BlockDevice>(
 
     // t5 分配记录树（字节表五；D8（核心索引结构） 已定项 14 按绝对槽号按位置寻址）：mkfs 的两个单元分配代 0、其余是各自分配那次发布的 txg，
     // 每盘各一条，按 (设备, 槽号) 升序分装进各自那片叶。重写哪几个节点在准入之前走到固定点定下来（`settle_the_allocation_record_tree`）：
-    // 这里取完落点之后内容变了的节点都在里面（释放之前读盘核出对不上的那几份留在已分配，只会让变了的更少）。
+    // 这里取完落点之后内容变了的节点都在里面（释放之前读盘核出对不上的那几份留在已分配、置上隔离位：记录照样变了，变在固定点按整份释放
+    // 预演时已经算进去的同一片叶里）。
     let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();
     allocation_records.sort_by_key(AllocationRecord::sort_key);
     let allocation_geometry = AllocationRecordTreeGeometry::of_allocator(allocator);
diff --git a/crates/singlefs-harness/src/history.rs b/crates/singlefs-harness/src/history.rs
index d331632..fb11593 100644
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -1827,6 +1827,7 @@ impl RecordReuse {
 /// 一次发布改写的记录只有三种，代都是这次发布的 txg：新分配的（分配代）、复用改写已回收的（分配代改成这次的，D3（空间分配） 的
 /// 「value = 分配代」）、这次释放的（释放代，`crates/singlefs-core/src/transaction.rs` 的 `allocator.release(*placement, txg);`）。
 /// 盘上 checker 判不出分配代没改（33 条不变量里只有 I-3.9 看代，只看已释放的；增补 2 收口表第 44 行），这里在执行器里判。
+/// 只差隔离位的那一条不算改写（C794 盘上隔离位）：释放核出对不上置位、回退重新引用隔离单元清位，都留着原来的分配代。
 #[must_use]
 pub fn records_changed_with_a_generation_outside_the_publish_txgs(
     before: &[AllocationRecord],
@@ -1835,9 +1836,20 @@ pub fn records_changed_with_a_generation_outside_the_publish_txgs(
     last_publish_txg: CheckpointTxg,
 ) -> Vec<AllocationRecord> {
     let records_before: BTreeSet<&AllocationRecord> = before.iter().collect();
+    let differs_from_a_record_before_only_in_the_quarantine_flag = |record: &AllocationRecord| {
+        records_before.contains(&AllocationRecord {
+            device: record.device,
+            slot: record.slot,
+            span_slots: record.span_slots,
+            generation: record.generation,
+            is_released: record.is_released,
+            is_quarantined: !record.is_quarantined,
+        })
+    };
     after
         .iter()
         .filter(|record| !records_before.contains(record))
+        .filter(|record| !differs_from_a_record_before_only_in_the_quarantine_flag(record))
         .filter(|record| {
             record.generation < first_publish_txg || record.generation > last_publish_txg
         })
@@ -7242,9 +7254,48 @@ mod tests {
             span_slots,
             generation: CheckpointTxg(generation),
             is_released,
+            is_quarantined: false,
         }
     }
 
+    /// 只差隔离位的记录不算这次发布改写的（C794 盘上隔离位）：释放核出对不上的那一份留在已分配、置上隔离位，分配代照旧是 3，
+    /// txg 12 那次发布不报它；同一条记录换了分配代（不只差隔离位）照报。
+    #[test]
+    fn record_that_only_gains_the_quarantine_flag_keeps_its_allocation_generation_and_is_not_reported(
+    ) {
+        let quarantined_record = |generation: u64| AllocationRecord {
+            device: DeviceIdentity(0),
+            slot: singlefs_core::address::SlotNumber(50180),
+            span_slots: 2,
+            generation: CheckpointTxg(generation),
+            is_released: false,
+            is_quarantined: true,
+        };
+        let before = vec![record(50180, 2, 3, false)];
+        let quarantined = vec![quarantined_record(3)];
+        assert_eq!(
+            records_changed_with_a_generation_outside_the_publish_txgs(
+                &before,
+                &quarantined,
+                CheckpointTxg(12),
+                CheckpointTxg(12)
+            ),
+            Vec::new(),
+            "只置上隔离位，分配代 3 照旧：不是这次发布改写的"
+        );
+        let quarantined_with_another_generation = vec![quarantined_record(5)];
+        assert_eq!(
+            records_changed_with_a_generation_outside_the_publish_txgs(
+                &before,
+                &quarantined_with_another_generation,
+                CheckpointTxg(12),
+                CheckpointTxg(12)
+            ),
+            quarantined_with_another_generation,
+            "代也变了：不只差隔离位，照报"
+        );
+    }
+
     /// 一次发布（txg 12）改写或新增的记录，代都要是 12：新分配的、复用改写已回收的、这次释放的都对就一条都不报；复用改写时代没改
     /// （还写着释放代 5，代码三方第一轮攻方的 N2）就报那一条。没动过的记录（代 3、代 0）不看。
     #[test]
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index c39064c..6d903f8 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -654,11 +654,15 @@ const EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET: usize = 163 + 24 + 1 + 42;
 /// （D8（核心索引结构） 已定项 8 的字段表）。
 const TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8;
 /// 分配记录树叶（key 宽 10）：条目数在 82 + 2 × 10、条目区从 115 + 2 × 10 起，每条 20 = key (设备 4, 槽号 6) +
-/// value (跨度段 2，最高位是已释放标志, 分配代或释放代 8)。
+/// value (跨度段 2，最高位是已释放标志、次高位是隔离标志（C794 盘上隔离位）, 分配代或释放代 8)。
 const ALLOCATION_NODE_ENTRY_COUNT_OFFSET: usize = 102;
 const ALLOCATION_NODE_ENTRY_START: usize = 135;
 const ALLOCATION_RECORD_BYTES: usize = 20;
 const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;
+const ALLOCATION_RECORD_QUARANTINED_FLAG: u16 = 0x4000;
+/// 跨度段里两个标志位合起来：改跨度的坏法照留它们。
+const ALLOCATION_RECORD_FLAGS: u16 =
+    ALLOCATION_RECORD_RELEASED_FLAG | ALLOCATION_RECORD_QUARANTINED_FLAG;
 
 fn get_u64(bytes: &[u8], offset: usize) -> u64 {
     u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
@@ -746,7 +750,7 @@ fn rewrite_released_generations(bytes: &mut [u8], from: u64, to: u64) -> usize {
     rewritten
 }
 
-/// 分配记录树叶里槽号是 `slot` 的记录（两盘各一条）跨度改成 `span_slots`，已释放标志照留；返回改了几条。
+/// 分配记录树叶里槽号是 `slot` 的记录（两盘各一条）跨度改成 `span_slots`，已释放、隔离两个标志照留；返回改了几条。
 fn set_allocation_record_span(bytes: &mut [u8], slot: u64, span_slots: u16) -> usize {
     let count = usize::from(u16::from_le_bytes([
         bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
@@ -762,7 +766,7 @@ fn set_allocation_record_span(bytes: &mut [u8], slot: u64, span_slots: u16) -> u
             set_u16(
                 bytes,
                 record + 10,
-                (span_field & ALLOCATION_RECORD_RELEASED_FLAG) | span_slots,
+                (span_field & ALLOCATION_RECORD_FLAGS) | span_slots,
             );
             rewritten += 1;
         }
@@ -3335,6 +3339,7 @@ fn hang_an_empty_leaf_below_the_root_of_the_allocation_record_tree(image: &mut M
                 span_slots: 1,
                 generation: CheckpointTxg(3),
                 is_released: false,
+                is_quarantined: false,
             }
             .to_bytes(),
         );
@@ -6606,12 +6611,12 @@ fn widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(
     for record_index in 0..record_count.saturating_sub(1) {
         let (start, device, slot, span_field) = record_at(bytes, record_index);
         let (_, next_device, next_slot, _) = record_at(bytes, record_index + 1);
-        let span_slots = span_field & !ALLOCATION_RECORD_RELEASED_FLAG;
+        let span_slots = span_field & !ALLOCATION_RECORD_FLAGS;
         if device == next_device && slot + u64::from(span_slots) == next_slot {
             set_u16(
                 bytes,
                 start + 10,
-                (span_field & ALLOCATION_RECORD_RELEASED_FLAG) | (span_slots + 1),
+                (span_field & ALLOCATION_RECORD_FLAGS) | (span_slots + 1),
             );
             return 1;
         }
diff --git a/crates/singlefs-harness/tests/checker_quarantine_flag_exempts_only_flagged_unreferenced_records.rs b/crates/singlefs-harness/tests/checker_quarantine_flag_exempts_only_flagged_unreferenced_records.rs
new file mode 100644
index 0000000..1ee239b
--- /dev/null
+++ b/crates/singlefs-harness/tests/checker_quarantine_flag_exempts_only_flagged_unreferenced_records.rs
@@ -0,0 +1,363 @@
+//! 池级 checker 按分配记录的隔离位豁免 I-3.1（已分配统计对得上） 与 I-3.11（已分配减 defer 等于最新根走读）（C794 盘上隔离位，
+//! 副本里量改动面的那一版：改用户 2026-09-25「核出坏的才豁免」，条款没改）。判决 `research/prompts/c794-r1-main-verification.md`
+//! H5（采纳攻方）：样本是手写故障设备造的瞬时隔离（两份都完好）、同一份把隔离位清掉，与三种真泄漏各一份——漏释放、悬空单元、
+//! 活单元记成已释放；前者不许有一条不变量红，后面四份照红。
+//!
+//! 共同的历史：新池新建文件 A（txg 3）；覆盖写 B（txg 4）换下 A 的数据单元；覆盖写 C、D、E（txg 5–7）；抬 F 到 4（A 的根掉出候选集，
+//! 那两条记录从此没有任何有效根引用，I-3.1 也够得着）。各用例只在 B 前后或抬 F 之前动一手。镜像经 `check_pool_image` 直接判
+//! （池级 checker 自己的单测：造一张已知该判什么的镜像、断言判器的结论）。
+
+mod common;
+
+use common::{
+    build_pool, parameters, publish_overwrite_in_process, BuiltPool, Recorded,
+    FIXED_WRITE_TIME_SECONDS,
+};
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
+};
+use singlefs_core::allocator::Placement;
+use singlefs_core::block_device::{
+    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
+};
+use singlefs_core::mount::{raise_rollback_floor, ShadowLedger};
+use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
+use singlefs_harness::fault_injection::injected_block_device_error;
+use std::cell::Cell;
+
+/// A 的数据单元的跨度（两槽）。
+const DATA_UNIT_SPAN_SLOTS: u64 = 2;
+
+fn content_of(length: usize, seed: usize) -> Vec<u8> {
+    (0..length)
+        .map(|index| u8::try_from((index * 11 + seed) % 247).expect("小于 256"))
+        .collect()
+}
+
+/// 一块盘上某一个偏移的读先报块设备错 `reads_to_refuse` 次，之后照转；盘上的字节一个不动。
+struct ReadsAtOneOffsetRefusedAFewTimes<Inner: BlockDevice> {
+    inner: Inner,
+    refused_offset: DeviceOffsetInBytes,
+    reads_to_refuse: Cell<u64>,
+}
+
+impl<Inner: BlockDevice> BlockDevice for ReadsAtOneOffsetRefusedAFewTimes<Inner> {
+    fn read_at(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        if offset == self.refused_offset && self.reads_to_refuse.get() > 0 {
+            self.reads_to_refuse.set(self.reads_to_refuse.get() - 1);
+            return Err(injected_block_device_error("读"));
+        }
+        self.inner.read_at(offset, buffer)
+    }
+    fn write_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+        durability: WriteDurability,
+    ) -> Result<(), BlockDeviceError> {
+        self.inner.write_at(offset, bytes, durability)
+    }
+    fn write_zeroes_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<(), BlockDeviceError> {
+        self.inner.write_zeroes_at(offset, length)
+    }
+    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
+        self.inner.barrier()
+    }
+    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
+        self.inner.probe_physical_block_size()
+    }
+    fn size_in_bytes(&self) -> u64 {
+        self.inner.size_in_bytes()
+    }
+}
+
+/// 新池新建文件之后覆盖写 B：A 的数据单元在每块盘上那一份的读先报错 `reads_to_refuse_on_each_device` 次（0 是不坏）。
+/// 交回池（`output` 换成 B）与 A 的数据单元那个槽。
+fn pool_after_overwriting_the_first_version(
+    tag: &str,
+    reads_to_refuse_on_each_device: u64,
+) -> (BuiltPool, SlotNumber) {
+    let mut pool = build_pool(tag);
+    let first = pool.output.clone();
+    let data_slot = first.data_pointers[0].locations[0].slot;
+    let mut wrapped: Vec<(DeviceIdentity, ReadsAtOneOffsetRefusedAFewTimes<Recorded>)> = pool
+        .devices
+        .take()
+        .expect("镜像还开着")
+        .into_iter()
+        .map(|(identity, device)| {
+            (
+                identity,
+                ReadsAtOneOffsetRefusedAFewTimes {
+                    inner: device,
+                    refused_offset: data_slot.to_device_offset(),
+                    reads_to_refuse: Cell::new(reads_to_refuse_on_each_device),
+                },
+            )
+        })
+        .collect();
+    let second = {
+        let publish_parameters = parameters();
+        let mut writer = PoolWriter::new(&publish_parameters, wrapped.as_mut_slice());
+        publish_overwrite(
+            &mut writer,
+            &mut pool.allocator,
+            &first,
+            FirstFile {
+                content: &content_of(2500, 17),
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
+            },
+            InstanceGeneration(1),
+        )
+        .expect("覆盖写 B 照成（读不出按对不上处置，发布照成）")
+    };
+    pool.devices = Some(
+        wrapped
+            .into_iter()
+            .map(|(identity, device)| (identity, device.inner))
+            .collect(),
+    );
+    assert_eq!(second.root.checkpoint_txg, CheckpointTxg(4));
+    pool.output = second;
+    (pool, data_slot)
+}
+
+/// 接着覆盖写 C、D、E（txg 5–7）；`output` 换成 E。
+fn overwrite_three_more_times(pool: &mut BuiltPool) {
+    let mut current = pool.output.clone();
+    for (seed, seconds) in [(6, 120), (7, 180), (8, 240)] {
+        current = publish_overwrite_in_process(
+            pool,
+            &current,
+            &content_of(2700, seed),
+            FIXED_WRITE_TIME_SECONDS + seconds,
+            InstanceGeneration(1),
+        )
+        .expect("接着覆盖写");
+    }
+    assert_eq!(current.root.checkpoint_txg, CheckpointTxg(7));
+    pool.output = current;
+}
+
+/// 抬 F 到 4：A 的根（txg 3）掉出候选集。
+fn raise_the_floor_to_four(pool: &mut BuiltPool) {
+    let publish_parameters = parameters();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let mut current: TransactionOutput = pool.output.clone();
+    let raised = raise_rollback_floor(
+        &publish_parameters,
+        devices,
+        &mut pool.allocator,
+        &mut current,
+        CheckpointTxg(4),
+        ShadowLedger::On,
+    )
+    .expect("抬 F 到 4");
+    assert!(!raised.publishes.is_empty(), "抬 F 发了空发布");
+    pool.output = current;
+}
+
+/// 池级 checker 对这张镜像的判定：(判红的不变量, I-3.1 的判定, I-3.11 的判定)。
+fn verdicts_of(pool: &BuiltPool) -> (Vec<&'static str>, InvariantVerdict, InvariantVerdict) {
+    let verdicts = check_pool_image(&pool.memory_pool());
+    let verdict_of = |name: &str| {
+        verdicts
+            .iter()
+            .find(|(invariant, _)| *invariant == name)
+            .map(|(_, verdict)| verdict.clone())
+            .expect("池级 checker 交回这一条的判定")
+    };
+    let violated = verdicts
+        .iter()
+        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
+        .map(|(invariant, _)| *invariant)
+        .collect();
+    (violated, verdict_of("I-3.1"), verdict_of("I-3.11"))
+}
+
+/// 两块盘各自那条分配记录（槽 `slot`）：(是不是已释放, 隔离位, 分配代或释放代)。
+fn records_of_slot(pool: &BuiltPool, slot: SlotNumber) -> Vec<(bool, bool, CheckpointTxg)> {
+    [DeviceIdentity(0), DeviceIdentity(1)]
+        .into_iter()
+        .map(|device| {
+            let record = pool
+                .allocator
+                .record_for(device, slot)
+                .expect("这个槽每块盘上都有记录");
+            (record.is_released, record.is_quarantined, record.generation)
+        })
+        .collect()
+}
+
+/// 瞬时隔离：A 的数据单元两块盘那一份的读各先报错两次（第一读、重读都读不出），B 按对不上处置、隔离那个单元，盘上一个字节都没动。
+/// 两条记录留在已分配、置上隔离位；之后两份都完好、没有任何根引用。池级 checker 按隔离位豁免：一条不变量都不红，I-3.1、I-3.11 判过且成立。
+/// 改之前（按 checker 自己读那一份判）两份都对得上、不豁免，I-3.11 从 B 起红、抬 F 之后 I-3.1 也红（C794）。
+#[test]
+fn transient_quarantine_of_two_intact_copies_carries_the_quarantine_flag_and_no_invariant_is_violated(
+) {
+    let (mut pool, data_slot) =
+        pool_after_overwriting_the_first_version("checker-quarantine-flag-transient", 2);
+    assert_eq!(
+        pool.output
+            .quarantined_after_release_checksum_mismatch
+            .len(),
+        2,
+        "前提：两块盘那一份都读不出、都隔离"
+    );
+    assert_eq!(
+        records_of_slot(&pool, data_slot),
+        vec![
+            (false, true, CheckpointTxg(3)),
+            (false, true, CheckpointTxg(3)),
+        ],
+        "两条记录留在已分配、带隔离位、分配代照旧 3"
+    );
+    overwrite_three_more_times(&mut pool);
+    raise_the_floor_to_four(&mut pool);
+    let (violated, allocated_statistic, allocated_minus_deferred) = verdicts_of(&pool);
+    assert_eq!(
+        violated,
+        Vec::<&str>::new(),
+        "瞬时隔离的合法镜像：一条都不红"
+    );
+    assert_eq!(
+        allocated_statistic,
+        InvariantVerdict::Holds,
+        "I-3.1 判过且成立"
+    );
+    assert_eq!(
+        allocated_minus_deferred,
+        InvariantVerdict::Holds,
+        "I-3.11 判过且成立"
+    );
+}
+
+/// 同一段瞬时隔离，B 之后把那两条记录的隔离位清掉（C 起写进盘上的账里就没有这一位）：它与写者漏释放逐字节同形
+/// （判决 H3：不改格式时两份镜像逐扇区相同），照红 I-3.1、I-3.11。豁免改成「没有根引用的一律豁免」时这一份变绿。
+#[test]
+fn transient_quarantine_with_the_quarantine_flag_cleared_violates_both_allocated_statistics() {
+    let (mut pool, data_slot) =
+        pool_after_overwriting_the_first_version("checker-quarantine-flag-cleared", 2);
+    pool.allocator
+        .clear_quarantine_of_the_records_of(Placement {
+            slot: data_slot,
+            span: DATA_UNIT_SPAN_SLOTS,
+        });
+    overwrite_three_more_times(&mut pool);
+    assert_eq!(
+        records_of_slot(&pool, data_slot),
+        vec![
+            (false, false, CheckpointTxg(3)),
+            (false, false, CheckpointTxg(3)),
+        ],
+        "前提：两条记录仍分配、隔离位清掉"
+    );
+    raise_the_floor_to_four(&mut pool);
+    let (violated, _, _) = verdicts_of(&pool);
+    assert!(
+        violated.contains(&"I-3.1") && violated.contains(&"I-3.11"),
+        "隔离位清掉的那两条记录不豁免：I-3.1、I-3.11 都红：{violated:?}"
+    );
+}
+
+/// 真泄漏一：漏释放。B 照常释放 A 的数据单元（两份都读得出、对得上），之后把那两条记录改回仍分配、分配代 3（写者漏了这一次释放）。
+/// 曾被 A 引用、此后没有任何根引用，不带隔离位：I-3.1、I-3.11 照红。
+#[test]
+fn forgotten_release_of_a_once_referenced_unit_violates_both_allocated_statistics() {
+    let (mut pool, data_slot) =
+        pool_after_overwriting_the_first_version("checker-quarantine-flag-forgotten-release", 0);
+    assert_eq!(
+        pool.output.quarantined_after_release_checksum_mismatch,
+        Vec::new(),
+        "前提：B 照常释放，一份都不隔离"
+    );
+    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+        pool.allocator.resurrect_released_record(
+            device,
+            Placement {
+                slot: data_slot,
+                span: DATA_UNIT_SPAN_SLOTS,
+            },
+            CheckpointTxg(3),
+        );
+    }
+    overwrite_three_more_times(&mut pool);
+    assert_eq!(
+        records_of_slot(&pool, data_slot),
+        vec![
+            (false, false, CheckpointTxg(3)),
+            (false, false, CheckpointTxg(3)),
+        ],
+        "前提：两条记录仍分配、不带隔离位"
+    );
+    raise_the_floor_to_four(&mut pool);
+    let (violated, _, _) = verdicts_of(&pool);
+    assert!(
+        violated.contains(&"I-3.1") && violated.contains(&"I-3.11"),
+        "漏释放：I-3.1、I-3.11 都红：{violated:?}"
+    );
+}
+
+/// 真泄漏二：悬空单元。B 之后分配器多发一个用户数据落点（分配代 5，C 那一次），从没有任何根引用它、槽里也没写过。
+/// 不带隔离位：I-3.1、I-3.11 照红（改之前按 checker 自己读那一份判，全零的槽自证不过、被当成隔离豁免掉）。
+#[test]
+fn dangling_unit_no_root_ever_referenced_violates_both_allocated_statistics() {
+    let (mut pool, _) =
+        pool_after_overwriting_the_first_version("checker-quarantine-flag-dangling-unit", 0);
+    let dangling = pool
+        .allocator
+        .allocate_user_data(CheckpointTxg(5))
+        .expect("单元区里有的是空槽");
+    overwrite_three_more_times(&mut pool);
+    assert_eq!(
+        records_of_slot(&pool, dangling.slot),
+        vec![
+            (false, false, CheckpointTxg(5)),
+            (false, false, CheckpointTxg(5)),
+        ],
+        "前提：悬空单元的两条记录仍分配、不带隔离位"
+    );
+    raise_the_floor_to_four(&mut pool);
+    let (violated, _, _) = verdicts_of(&pool);
+    assert!(
+        violated.contains(&"I-3.1") && violated.contains(&"I-3.11"),
+        "悬空单元：I-3.1、I-3.11 都红：{violated:?}"
+    );
+}
+
+/// 真泄漏三：活单元记成已释放。E 之后、抬 F 之前，分配器把 E 还引用着的数据单元记成已释放（释放代 8，抬 F 的第一次空发布），
+/// 抬 F 那几次空发布把它写进盘上的账。I-3.11 照红（「已分配减 defer」少了它、最新根走读还数着它）。
+#[test]
+fn live_unit_recorded_as_released_violates_the_allocated_minus_deferred_invariant() {
+    let (mut pool, _) =
+        pool_after_overwriting_the_first_version("checker-quarantine-flag-live-unit-released", 0);
+    overwrite_three_more_times(&mut pool);
+    let live_data_slot = pool.output.data_pointers[0].locations[0].slot;
+    pool.allocator.release(
+        Placement {
+            slot: live_data_slot,
+            span: DATA_UNIT_SPAN_SLOTS,
+        },
+        CheckpointTxg(8),
+    );
+    raise_the_floor_to_four(&mut pool);
+    assert_eq!(
+        pool.output.data_pointers[0].locations[0].slot, live_data_slot,
+        "前提：最新根还引用那个数据单元"
+    );
+    let (violated, _, _) = verdicts_of(&pool);
+    assert!(
+        violated.contains(&"I-3.11"),
+        "活单元记成已释放：I-3.11 红：{violated:?}"
+    );
+}
diff --git a/crates/singlefs-harness/tests/common_corrupted_allocation_record/mod.rs b/crates/singlefs-harness/tests/common_corrupted_allocation_record/mod.rs
index c58198d..11f38dd 100644
--- a/crates/singlefs-harness/tests/common_corrupted_allocation_record/mod.rs
+++ b/crates/singlefs-harness/tests/common_corrupted_allocation_record/mod.rs
@@ -22,10 +22,10 @@ const ROOT_SELF_CHECKSUM_OFFSET: usize = ROOT_TREE_TABLE_POINTER_OFFSET + 86 + 8
 /// 根记录（371 字节）里三条指针的偏移：树表紧跟 magic 4 + fsid 16 + flags 4 + 实例代号 4 + txg 8；
 /// 自证校验和在 138，实例表与中央映射树根紧跟在它之后（`singlefs-core` 的 `root_record.rs` 同一张表，这里按字段表另写一份）。
 const ROOT_TREE_TABLE_POINTER_OFFSET: usize = 4 + 16 + 4 + 4 + 8;
-/// 分配记录里跨度那个 u16 的最高位是「已释放」标志（`singlefs-core` 的 `ALLOCATION_RECORD_RELEASED_FLAG`），
-/// 低 15 位才是跨度。「跨度越过单元区末尾」那一条坏法把整个字段写成这个数：跨度取低 15 位的最大值、已释放位清掉
-/// （还着的记录才会被重建拿去 `mark_allocated`，普查 R6 / R7）。
-const SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED: u16 = 0x7FFF;
+/// 分配记录里跨度那个 u16 的最高位是「已释放」标志（`singlefs-core` 的 `ALLOCATION_RECORD_RELEASED_FLAG`）、次高位是隔离标志
+/// （`ALLOCATION_RECORD_QUARANTINED_FLAG`，C794 盘上隔离位），低 14 位才是跨度。「跨度越过单元区末尾」那一条坏法把整个字段写成这个数：
+/// 跨度取低 14 位的最大值、两个标志位都清掉（还着的记录才会被重建拿去 `mark_allocated`，普查 R6 / R7）。
+const SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED: u16 = 0x3FFF;
 const TREE_KIND_ALLOCATION_RECORDS: u16 = 3;
 /// 树表条目里「树的种类」那个 u16 的偏移。
 const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
@@ -107,7 +107,7 @@ struct IndexNodeEntryLayout {
     entry_width: usize,
 }
 
-/// 把「根槽 → 树表 → 分配记录树根 → …… → 最左那片叶」这条链上那片叶的第一条分配记录改成「起点贴着单元区末尾、跨度 32767 槽」
+/// 把「根槽 → 树表 → 分配记录树根 → …… → 最左那片叶」这条链上那片叶的第一条分配记录改成「起点贴着单元区末尾、跨度 16383 槽」
 /// （越过单元区末尾，普查 R6 / R7），并把链上的校验和逐道重算：叶两道 → 上一层第 0 条条目子指针里的整单元校验和、那一层两道 →
 /// …… → 分配记录树根两道 → 树表条目里那条根指针的整单元校验和 → 树表节点两道 → 根槽里树表指针的整单元校验和 → 根槽的自证校验和。
 ///
@@ -118,7 +118,7 @@ struct IndexNodeEntryLayout {
 /// **字节由调用方从盘上读来、改完自己写回去**：`allocation_tree_nodes_from_the_root` 是从根起、每层沿第 0 条条目往下读到的那一路
 /// （根在前、叶在末）。坏盘输入那一路读的是 [`MemoryPool`]，步 4 回退那一路读的是录制设备，两边共用这一份，偏移与重算口径只有这一处
 /// （`code-discipline.md`「重复要生成，不许手抄」）。跨度字段整个写成
-/// [`SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED`]：跨度取低 15 位的最大值、已释放位跟着清掉
+/// [`SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED`]：跨度取低 14 位的最大值、已释放位与隔离位跟着清掉
 /// （只有还着的记录才会被影子账拿去隔离）。
 ///
 /// 交回「坏在哪」那句话；树表里没有分配记录树、这一路的末一个不是叶、中间有一层没有条目、或者叶里一条记录都没有时交回 `None`。
@@ -189,7 +189,7 @@ pub fn move_the_first_allocation_record_past_the_end_of_the_unit_area_and_reseal
     reseal_wide_checksum_field(root_slot, cover_end, ROOT_SELF_CHECKSUM_OFFSET);
 
     Some(format!(
-        "分配器：这条根那棵账里第一条分配记录的槽号 {old_slot} → {last_slot_of_the_unit_area}（单元区末尾是 {unit_area_end_slot}）、跨度字段 {old_span_field:#06x} → {SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED:#06x}（跨度 {} → 32767 槽，已释放位清掉）；链上每一道校验和重算",
+        "分配器：这条根那棵账里第一条分配记录的槽号 {old_slot} → {last_slot_of_the_unit_area}（单元区末尾是 {unit_area_end_slot}）、跨度字段 {old_span_field:#06x} → {SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED:#06x}（跨度 {} → 16383 槽，已释放位与隔离位清掉）；链上每一道校验和重算",
         old_span_field & SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED
     ))
 }
diff --git a/crates/singlefs-harness/tests/common_images_for_the_checker_tier/release_checksum_quarantine.rs b/crates/singlefs-harness/tests/common_images_for_the_checker_tier/release_checksum_quarantine.rs
index 489a31a..b6bd9b8 100644
--- a/crates/singlefs-harness/tests/common_images_for_the_checker_tier/release_checksum_quarantine.rs
+++ b/crates/singlefs-harness/tests/common_images_for_the_checker_tier/release_checksum_quarantine.rs
@@ -124,12 +124,11 @@ pub fn pool_whose_quarantined_records_no_root_references_with_the_newest_data_sl
     (pool, data_slot, newest_data_slot)
 }
 
-/// 用例 10：I-3.1（已分配统计对得上） 与 I-3.11（已分配减 defer 等于最新根走读） 怎么认隔离的记录（用户 2026-09-25 定，
-/// D19（块指针的结构与宽度预算） 已定项 5 的隔离；按单元判是主 agent 同日定的读法）：已分配而没有任何根引用的记录，它罩住的单元
-/// 只要有一份 checker 自己读出来读不出或校验和对不上，那个单元每块盘上的记录都豁免；每一份都读得出且对得上的照旧判违例。
-/// 1. 两盘那一份都改坏：两条记录豁免，I-3.1、I-3.11 都成立（改之前 checker 不认隔离，两条都红）；
-/// 2. 把那两份改回对得上（再翻一次同一个字节）：同样两条记录不再豁免，I-3.1、I-3.11 照红。
-pub fn quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read(
+/// 用例 10：I-3.1（已分配统计对得上） 与 I-3.11（已分配减 defer 等于最新根走读） 怎么认隔离的记录（C794 盘上隔离位，改用户 2026-09-25
+/// 「核出坏的才豁免」，条款没改）：已分配而没有任何根引用的记录，跨度段的隔离位置着就豁免，与那一份此刻读不读得出、对不对得上无关。
+/// 1. 两盘那一份都改坏：写者两条记录都置了隔离位，两条记录豁免，I-3.1、I-3.11 都成立；
+/// 2. 把那两份改回对得上（再翻一次同一个字节）：隔离位还在，两条记录照样豁免（改之前按读盘判，这一份照红）。
+pub fn quarantined_record_is_exempted_from_the_allocated_statistics_by_its_quarantine_flag_whether_or_not_its_copies_read_back(
     images: &mut PoolCheckerImages,
 ) {
     let (mut pool, data_slot) = pool_whose_quarantined_records_no_root_references(
@@ -137,7 +136,7 @@ pub fn quarantined_record_is_exempted_from_the_allocated_statistics_only_while_i
         &[DeviceIdentity(0), DeviceIdentity(1)],
     );
     let image_with_the_damaged_copies = pool.memory_pool();
-    // I-3.1、I-3.11：隔离的那两条记录没有根引用，checker 读那一份自证不过 ⇒ 豁免
+    // I-3.1、I-3.11：隔离的那两条记录没有根引用、带隔离位 ⇒ 豁免
     images.hand_out(
         "两盘那一份都改坏",
         || image_with_the_damaged_copies.clone(),
@@ -152,20 +151,20 @@ pub fn quarantined_record_is_exempted_from_the_allocated_statistics_only_while_i
         corrupt_the_copy_on(devices, DeviceIdentity(1), data_slot);
     }
     let image_with_the_copies_intact_again = pool.memory_pool();
-    // I-3.1、I-3.11：那两份改回对得上，同样两条记录读得出且对得上 ⇒ 不豁免、照红
+    // I-3.1、I-3.11：那两份改回对得上，两条记录的隔离位还在 ⇒ 照样豁免
     images.hand_out(
         "那两份改回对得上",
         || image_with_the_copies_intact_again.clone(),
         PoolCheckerExpectation::only_the_named_invariants()
-            .and_violated("I-3.1")
-            .and_violated("I-3.11"),
+            .and_holds("I-3.1")
+            .and_holds("I-3.11"),
     );
 }
 
-/// 用例 11（按单元判，主 agent 2026-09-25 定）：只盘 0 那一份改坏——覆盖写把两块盘的记录一起留在已分配（D19 已定项 5），
-/// 盘 1 那一份对得上。那个单元有一份坏，两块盘上的两条记录都豁免：I-3.1、I-3.11 都成立（按份判的话盘 1 那条恒红）。
-/// 把盘 0 那一份改回对得上：单元两份都好，两条记录照红。
-pub fn unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not(
+/// 用例 11：只盘 0 那一份改坏——覆盖写把两块盘的记录一起留在已分配（D19 已定项 5）、两条都置隔离位，盘 1 那一份对得上。
+/// 两块盘上的两条记录都带隔离位、都豁免：I-3.1、I-3.11 都成立。把盘 0 那一份改回对得上：隔离位还在，两条记录照样豁免
+/// （改之前按读盘判，单元两份都好就照红）。
+pub fn unit_with_one_failing_copy_carries_the_quarantine_flag_on_every_device_and_stays_exempted_once_both_copies_read_back(
     images: &mut PoolCheckerImages,
 ) {
     let (mut pool, data_slot) = pool_whose_quarantined_records_no_root_references(
@@ -173,7 +172,7 @@ pub fn unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_in
         &[DeviceIdentity(0)],
     );
     let image_with_one_damaged_copy = pool.memory_pool();
-    // I-3.1、I-3.11：单元有一份坏 ⇒ 两块盘上的两条记录都豁免
+    // I-3.1、I-3.11：两块盘上的两条记录都带隔离位 ⇒ 都豁免
     images.hand_out(
         "只盘 0 那一份改坏",
         || image_with_one_damaged_copy.clone(),
@@ -187,22 +186,21 @@ pub fn unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_in
         data_slot,
     );
     let image_with_both_copies_intact = pool.memory_pool();
-    // I-3.1、I-3.11：单元两份都好 ⇒ 不豁免、照红
+    // I-3.1、I-3.11：单元两份都好，隔离位还在 ⇒ 照样豁免
     images.hand_out(
         "单元两份都好",
         || image_with_both_copies_intact.clone(),
         PoolCheckerExpectation::only_the_named_invariants()
-            .and_violated("I-3.1")
-            .and_violated("I-3.11"),
+            .and_holds("I-3.1")
+            .and_holds("I-3.11"),
     );
 }
 
 /// 用例 14（实七；实四丙交回 ⑤ 那一形）：隔离的记录罩住的那一份是一个**完整的别的单元**——magic、头校验和、载荷 CRC 都过得去（自证过），
 /// 与指着它的位置项里的单元校验和对不上。丢一次写落在复用过的槽上时就是这样：槽里留着上一个单元，写者释放之前读盘核按位置项核出对不上、
-/// 隔离（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）。checker 的豁免与写者同一个判据（对位置项里的校验和）：两条记录豁免，
-/// I-3.1、I-3.11 都成立；按自证判（改之前的读法）两份都过得去、不豁免，两条都红。
-/// 指着那一份的位置项只在 A 的根（txg 3）里，而 F 已抬到 4、A 不在候选集里：checker 要把根环里没走过的根也走一遍才收得到它。
-pub fn quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_against_the_checksum_in_its_location_entry(
+/// 隔离（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）、两条记录置隔离位。checker 按隔离位豁免（C794 盘上隔离位），不再读那一份：
+/// 两条记录豁免，I-3.1、I-3.11 都成立。
+pub fn quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_by_its_quarantine_flag(
     images: &mut PoolCheckerImages,
 ) {
     let (mut pool, data_slot, newest_data_slot) =
@@ -236,7 +234,7 @@ pub fn quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_ag
         }
     }
     let image = pool.memory_pool();
-    // I-3.1、I-3.11：隔离的那两份自证过得去、与 A 的根里的位置项对不上 ⇒ 按位置项判、豁免
+    // I-3.1、I-3.11：隔离的那两条记录带隔离位 ⇒ 豁免
     images.hand_out(
         "隔离的那两份是一个完整的别的单元",
         || image.clone(),
diff --git a/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs b/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
index af67ada..459dec8 100644
--- a/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
+++ b/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
@@ -1702,7 +1702,7 @@ fn rewrite_allocation_records_in_the_subtree(
     devices: &mut Devices,
     pointer: &NodePointer,
     is_the_record_to_change: &dyn Fn(&AllocationRecord) -> bool,
-    change: &dyn Fn(&mut AllocationRecord),
+    change_the_entry_bytes: &dyn Fn(&mut [u8]),
     rewritten_nodes: &mut Vec<(NodePointer, u32)>,
     changed_records: &mut usize,
 ) -> Option<u32> {
@@ -1718,11 +1718,10 @@ fn rewrite_allocation_records_in_the_subtree(
     for entry_index in 0..header.entries.len() {
         let entry_range = entry_range_in_node(&node, entry_index);
         if header.level == 0 {
-            let mut record = AllocationRecord::parse(&node[entry_range.clone()])
+            let record = AllocationRecord::parse(&node[entry_range.clone()])
                 .expect("分配记录叶的条目宽是 20");
             if is_the_record_to_change(&record) {
-                change(&mut record);
-                node[entry_range].copy_from_slice(&record.to_bytes());
+                change_the_entry_bytes(&mut node[entry_range]);
                 *changed_records += 1;
                 is_this_node_changed = true;
             }
@@ -1734,7 +1733,7 @@ fn rewrite_allocation_records_in_the_subtree(
             devices,
             &child,
             is_the_record_to_change,
-            change,
+            change_the_entry_bytes,
             rewritten_nodes,
             changed_records,
         ) {
@@ -1758,6 +1757,24 @@ fn rewrite_allocation_records_of_the_newest_root(
     devices: &mut Devices,
     is_the_record_to_change: impl Fn(&AllocationRecord) -> bool,
     change: impl Fn(&mut AllocationRecord),
+) -> usize {
+    rewrite_allocation_record_entries_of_the_newest_root(
+        devices,
+        is_the_record_to_change,
+        |entry: &mut [u8]| {
+            let mut record = AllocationRecord::parse(entry).expect("分配记录叶的条目宽是 20");
+            change(&mut record);
+            entry.copy_from_slice(&record.to_bytes());
+        },
+    )
+}
+
+/// 同 [`rewrite_allocation_records_of_the_newest_root`]，只是改的是那条记录在叶里的 20 字节本身：造编码写不出来的组合
+/// （已释放位与隔离位同时置着）用这一条。
+fn rewrite_allocation_record_entries_of_the_newest_root(
+    devices: &mut Devices,
+    is_the_record_to_change: impl Fn(&AllocationRecord) -> bool,
+    change_the_entry_bytes: impl Fn(&mut [u8]),
 ) -> usize {
     let (ring_slot, mut root) = newest_root_in_the_ring(devices);
     let mut tree_table = read_bytes(
@@ -1789,7 +1806,7 @@ fn rewrite_allocation_records_of_the_newest_root(
         devices,
         &allocation_tree_root_pointer,
         &is_the_record_to_change,
-        &change,
+        &change_the_entry_bytes,
         &mut rewritten_nodes,
         &mut changed_records,
     )
@@ -1921,6 +1938,42 @@ fn drop_the_entry_of_the_largest_key_from_the_central_mapping_root_of_the_newest
     write_root_into_the_ring_slot(devices, ring_slot, &root);
 }
 
+/// 场景（C794 盘上隔离位）：最新那条根的分配记录树里已释放的那几条记录（两盘对称）跨度段再置上隔离位——已释放位与隔离位同时置着，
+/// 写者不写这一组合（释放清掉隔离位），条款没定读到它时怎么办。指着改过的节点的每一道校验和重算。
+/// 预期：可写挂载在任何写之前拒成 I-1.1「已释放位与隔离位同时置着」那一句（`ALLOCATION_RECORD_RELEASED_AND_QUARANTINED_AT_ONCE_UNDEFINED`），
+/// 盘上逐字节不变；不拒的话重建出来的分配器在下一次改写那条记录时撞上编码的断言。
+#[test]
+fn an_allocation_record_carrying_both_the_released_and_the_quarantine_flag_is_refused_by_the_writable_mount_before_any_write(
+) {
+    let (mut devices, _first) = pool_after_the_first_file();
+    let changed_records = rewrite_allocation_record_entries_of_the_newest_root(
+        &mut devices,
+        |record| record.is_released,
+        |entry| {
+            let span_field = u16::from_le_bytes([entry[10], entry[11]]);
+            entry[10..12].copy_from_slice(
+                &(span_field | singlefs_core::allocator::ALLOCATION_RECORD_QUARANTINED_FLAG)
+                    .to_le_bytes(),
+            );
+        },
+    );
+    assert!(
+        changed_records >= 2 && changed_records % 2 == 0,
+        "前提：第一个文件那一版的账里有已释放的记录，两盘对称（改了 {changed_records} 条）"
+    );
+    let refusal = writable_mount_refused_before_any_write(&mut devices);
+    assert!(
+        matches!(
+            refusal,
+            MountError::Recovery(RecoveryFailure::InvariantViolated {
+                invariant: "I-1.1",
+                detail,
+            }) if detail == singlefs_core::allocation_record_tree::ALLOCATION_RECORD_RELEASED_AND_QUARANTINED_AT_ONCE_UNDEFINED
+        ),
+        "已释放位与隔离位同时置着：拒成条款没定的那一句：{refusal:?}"
+    );
+}
+
 /// 最新那条根（txg 4）的分配记录树里罩这一版树表单元那个槽的两条记录（两盘同槽各一条，代 4、还着）代改成 5，晚于根；两盘一起改，
 /// 「每个落点每盘一条」那一判（比的是各盘的（槽, 跨度, 代, 已释放）集合）照过。指着改过的节点的每一道校验和重算。
 fn move_the_allocation_records_of_the_tree_table_slot_after_the_root(devices: &mut Devices) {
diff --git a/crates/singlefs-harness/tests/release_checksum_quarantine.rs b/crates/singlefs-harness/tests/release_checksum_quarantine.rs
index 5873ae0..38bfebd 100644
--- a/crates/singlefs-harness/tests/release_checksum_quarantine.rs
+++ b/crates/singlefs-harness/tests/release_checksum_quarantine.rs
@@ -15,8 +15,8 @@
 //! 6. 隔离之后重挂：那一槽的记录从盘上读回来仍是已分配，能回收的都回收了它也不回空闲池、不被再发出去；
 //! 7. 准入照已分配算：与不改坏的同一段历史比，两块盘的已分配各多那一份的两槽、可用各少两槽，被抛弃根独占量那一项是 0；
 //! 8. 位置项指的盘不在池里、9. 两条位置项指同一块盘：当映射条目损坏，在任何落盘动作之前拒绝，盘上不变；
-//! 10. 池级 checker 的 I-3.1 / I-3.11 怎么认隔离的记录：已分配而没有根引用的记录，checker 自己读它罩住的那一份
-//!     也读不出或自证不过才豁免，读得出且对得上的照旧判违例。
+//! 10. 池级 checker 的 I-3.1 / I-3.11 怎么认隔离的记录：已分配而没有根引用的记录，跨度段的隔离位置着就豁免
+//!     （C794 盘上隔离位），与那一份此刻读不读得出无关；隔离位由写者在隔离时置（用例 1 钉置位、用例 6 钉它跨重挂）。
 //! 12. 读得出而核出对不上一次（瞬时坏读）、重读对得上：照常释放，一份都不隔离（对不上也先重读一次）；
 //! 13. 每一读都核出对不上：重读恰好一次，之后按对不上处置。
 mod common;
@@ -89,6 +89,19 @@ fn records_of_slot(
         .collect()
 }
 
+/// 两块盘各自那条分配记录（槽 `slot`）的隔离位（C794 盘上隔离位）。
+fn quarantine_flags_of_slot(pool: &BuiltPool, slot: SlotNumber) -> Vec<bool> {
+    [DeviceIdentity(0), DeviceIdentity(1)]
+        .into_iter()
+        .map(|device| {
+            pool.allocator
+                .record_for(device, slot)
+                .expect("释放只改写记录、不删")
+                .is_quarantined
+        })
+        .collect()
+}
+
 fn is_free_on_each_device(pool: &BuiltPool, slot: SlotNumber) -> Vec<bool> {
     pool.allocator
         .devices
@@ -151,6 +164,11 @@ fn released_unit_whose_copies_fail_the_checksum_in_its_mapping_entry_is_quaranti
         ],
         "两盘的分配记录都留在已分配、分配代照旧"
     );
+    assert_eq!(
+        quarantine_flags_of_slot(&pool, data_slot),
+        vec![true, true],
+        "留在已分配的两条记录都置上隔离位（C794 盘上隔离位）"
+    );
     assert!(
         second
             .allocation_records
@@ -528,6 +546,11 @@ fn release_checksum_read_that_fails_once_is_read_again_and_the_intact_copy_is_re
         ],
         "两盘都照常改写成已释放、释放代 4"
     );
+    assert_eq!(
+        quarantine_flags_of_slot(&pool, data_slot),
+        vec![false, false],
+        "照常释放的记录不带隔离位"
+    );
 }
 
 /// 用例 5：两盘那一份都一直读不到：各先重读一次（恰好一次，不多次重试），还读不出就按对不上处置——
@@ -700,20 +723,27 @@ fn quarantined_copy_stays_allocated_across_the_remount_and_is_never_handed_out_a
     let reopened = pool.memory_pool();
     let system_configuration = choose_system_configuration(&reopened).expect("系统配置");
     let newest = choose_root(&reopened, &system_configuration).expect("最新根");
-    let records_on_disk: Vec<(DeviceIdentity, bool, CheckpointTxg)> =
+    let records_on_disk: Vec<(DeviceIdentity, bool, bool, CheckpointTxg)> =
         allocation_records_under_root(&reopened, &newest)
             .expect("最新根那棵账读得出")
             .into_iter()
             .filter(|record| record.slot == data_slot)
-            .map(|record| (record.device, record.is_released, record.generation))
+            .map(|record| {
+                (
+                    record.device,
+                    record.is_released,
+                    record.is_quarantined,
+                    record.generation,
+                )
+            })
             .collect();
     assert_eq!(
         records_on_disk,
         vec![
-            (DeviceIdentity(0), false, CheckpointTxg(3)),
-            (DeviceIdentity(1), false, CheckpointTxg(3)),
+            (DeviceIdentity(0), false, true, CheckpointTxg(3)),
+            (DeviceIdentity(1), false, true, CheckpointTxg(3)),
         ],
-        "重挂之后最新根那棵账里，那一槽两盘的记录仍是已分配、分配代 3"
+        "重挂之后最新根那棵账里，那一槽两盘的记录仍是已分配、带隔离位、分配代 3"
     );
     for slot in [data_slot, SlotNumber(data_slot.0 + 1)] {
         assert_eq!(
@@ -925,22 +955,22 @@ fn two_mapping_locations_on_the_same_device_are_refused_as_a_damaged_mapping_ent
     assert!(records_unchanged, "分配器一条记录都没改写");
 }
 
-/// 场景与它交给池级 checker 的镜像见 `scenarios_of_release_checksum_quarantine::quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
+/// 场景与它交给池级 checker 的镜像见 `scenarios_of_release_checksum_quarantine::quarantined_record_is_exempted_from_the_allocated_statistics_by_its_quarantine_flag_whether_or_not_its_copies_read_back`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
 #[test]
-fn quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read(
+fn quarantined_record_is_exempted_from_the_allocated_statistics_by_its_quarantine_flag_whether_or_not_its_copies_read_back(
 ) {
-    scenarios_of_release_checksum_quarantine::quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read(&mut PoolCheckerImages::left_to_the_checker_tier());
+    scenarios_of_release_checksum_quarantine::quarantined_record_is_exempted_from_the_allocated_statistics_by_its_quarantine_flag_whether_or_not_its_copies_read_back(&mut PoolCheckerImages::left_to_the_checker_tier());
 }
 
-/// 场景与它交给池级 checker 的镜像见 `scenarios_of_release_checksum_quarantine::unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
+/// 场景与它交给池级 checker 的镜像见 `scenarios_of_release_checksum_quarantine::unit_with_one_failing_copy_carries_the_quarantine_flag_on_every_device_and_stays_exempted_once_both_copies_read_back`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
 #[test]
-fn unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not() {
-    scenarios_of_release_checksum_quarantine::unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not(&mut PoolCheckerImages::left_to_the_checker_tier());
+fn unit_with_one_failing_copy_carries_the_quarantine_flag_on_every_device_and_stays_exempted_once_both_copies_read_back(
+) {
+    scenarios_of_release_checksum_quarantine::unit_with_one_failing_copy_carries_the_quarantine_flag_on_every_device_and_stays_exempted_once_both_copies_read_back(&mut PoolCheckerImages::left_to_the_checker_tier());
 }
 
-/// 场景与它交给池级 checker 的镜像见 `scenarios_of_release_checksum_quarantine::quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_against_the_checksum_in_its_location_entry`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
+/// 场景与它交给池级 checker 的镜像见 `scenarios_of_release_checksum_quarantine::quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_by_its_quarantine_flag`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
 #[test]
-fn quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_against_the_checksum_in_its_location_entry(
-) {
-    scenarios_of_release_checksum_quarantine::quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_against_the_checksum_in_its_location_entry(&mut PoolCheckerImages::left_to_the_checker_tier());
+fn quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_by_its_quarantine_flag() {
+    scenarios_of_release_checksum_quarantine::quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_by_its_quarantine_flag(&mut PoolCheckerImages::left_to_the_checker_tier());
 }
diff --git a/crates/singlefs-harness/tests/rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs b/crates/singlefs-harness/tests/rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs
index 396285e..4c37541 100644
--- a/crates/singlefs-harness/tests/rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs
+++ b/crates/singlefs-harness/tests/rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs
@@ -6,7 +6,8 @@
 //! 两块盘那条记录都留在已分配、分配代照旧 3（隔离）；挂着时回退到 A。A 引用的数据单元在 B 的账里仍分配、同分配代，B 不引用它。
 //! - 盘 1 那一份仍坏：在任何写之前拒成 `IsolatedUnitCopyUnreadableOrMismatched`（盘 1、那个槽），盘上逐字节不变、现行版本与分配器不动。
 //!   不验它（改之前的样子）时回退做成，新根引用一个只有一份读得出的单元。
-//! - 盘 1 那一份写回原样（两份都对得上）：回退照做，冷启动读回 A 的内容——逐盘验只拒对不上的，不拒隔离本身。
+//! - 盘 1 那一份写回原样（两份都对得上）：回退照做，冷启动读回 A 的内容——逐盘验只拒对不上的，不拒隔离本身；
+//!   回退之后那两条记录清掉隔离位（C794 盘上隔离位：新根又引用它，位只留在没有根引用的记录上，条款没写这一格、取最保守的读法）。
 
 mod common;
 
@@ -124,8 +125,8 @@ fn pool_with_the_data_unit_of_the_first_version_isolated(
             .record_for(device, data_slot)
             .expect("隔离只是记录留在已分配，不删");
         assert!(
-            !record.is_released && record.generation == CheckpointTxg(3),
-            "前提：{device:?} 上那条记录仍分配、分配代照旧 3：{record:?}"
+            !record.is_released && record.is_quarantined && record.generation == CheckpointTxg(3),
+            "前提：{device:?} 上那条记录仍分配、带隔离位、分配代照旧 3：{record:?}"
         );
     }
     assert!(
@@ -203,6 +204,16 @@ fn rolling_back_to_the_version_referencing_an_isolated_unit_whose_failing_copy_w
             .all(|copy| copy.placement.slot != data_slot),
         "隔离单元在账里本来就仍分配，不进复活集"
     );
+    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+        let record = pool
+            .allocator
+            .record_for(device, data_slot)
+            .expect("回退不删记录");
+        assert!(
+            !record.is_released && !record.is_quarantined && record.generation == CheckpointTxg(3),
+            "回退之后新根引用它：{device:?} 上那条记录仍分配、隔离位清掉、分配代照旧 3：{record:?}"
+        );
+    }
     let cold = recover(&pool.memory_pool(), JournalPolicy::Consult);
     assert!(
         matches!(
```
