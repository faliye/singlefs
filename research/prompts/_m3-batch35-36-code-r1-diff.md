# 删除批第 35 批 W 与第 36 批 V 合入的改动原样（2026-10-08）

## m3-batch35/w：crates.patch（/home/fy5090/.local/share/singlefs/work/m3-batch35/w/patch/crates.patch）

````diff
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -262,8 +262,10 @@
     /// 空闲槽数，独立维护：分配时减、回收放回时加，不由「容量 − 已分配」现算（D5（快照 / 空间记账机制） 已定项 4 的 ⚠️：
     /// 那样 I-5.2（空闲统计对得上） 是恒真式；三方代码第一轮攻方腿打中）。已释放而还在 defer 窗口里的不算空闲。
     free_slots: u64,
-    /// 影子账（D23（journal 的角色与格式） 已定项 14 回退段；D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」）：
-    /// 只被被抛弃根引用的槽，当前账里既不是已分配也不是 defer，分配器却不许发出去，直到被抛弃的根离开根环。只住内存、不进记账行。
+    /// 影子账（D23（journal 的角色与格式） 已定项 14 回退段与「可写挂载时的预隔离」；D28（挂载期承诺量） 已定项 1 第九项）隔离的槽，两部分置同一位：
+    /// 只被被抛弃根引用的槽，隔离到那条被抛弃的根离开根环；可写挂载那一刻水位之上没施加的 journal 记录点名的落点与那几次发布的账里的落点
+    /// （C694 的 C，现行账里仍分配着的除外），隔离到这个进程写掉那条记录的 txg 对应的根环槽（`RootRingOccupancy` 里记着的那一份）。
+    /// 隔离位独立于分配位（[`Self::isolate`]）。只住内存、不进记账行。
     isolated: Vec<bool>,
     isolated_per_segment: Vec<u64>,
     isolated_slots: u64,
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -1485,8 +1485,9 @@
 /// 那条根引用的单元已经没有要护的，只计数、不隔离、不拒（C694 的 B，判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 2 条：
 /// 凭据只读内存里的现行账，不读那一槽）；没有这样的记录（那几槽只是这一次读不出）就在任何写之前拒：挂载在取号之前拒可写、抬 F 拒这一次抬
 /// （[`StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot`]）。候选根读不出就当它什么都不豁免（隔离只会多不会少）。
-/// 同一次也隔离水位之上没施加的 journal 记录点名的落点（`placements_named_by_unapplied_journal_records`，C694 的 C）：豁免与被抛弃根那一份
-/// 同一个集合（候选集里的根与现行账引用的槽），D28（挂载期承诺量） 已定项 1 第九项按分配器实际隔离的槽数取，两部分都在里面。
+/// 同一次也隔离水位之上没施加的 journal 记录点名的落点与那几次发布的账里的落点（`placements_named_by_unapplied_journal_records`，C694 的 C）：
+/// 只豁免现行账里仍分配着的槽，不豁免候选集里的根引用的槽（根环线 K4-1，函数体里那一圈的注释），D28（挂载期承诺量） 已定项 1 第九项
+/// 按分配器实际隔离的槽数取，两部分都在里面。
 /// 隔离位独立于分配位（`DeviceFreeMap::isolate`），所以候选集缩小（抬 F）之后重算只会多隔离几个槽，不用撤销；
 /// 被抛弃根引用的只在它离开根环的那一次发布里清，水位之上的记录点名的只在这个进程写掉那条记录的 txg 对应的根环槽那一次发布里清
 /// （`PoolAllocator::record_root_written_by_this_process`，按交回的每条根引用的落点与分配器根环表记着的那一份）。
@@ -1506,11 +1507,13 @@
     placements_named_by_unapplied_journal_records: &[PlacementOnDevice],
     before_the_one_reread: &mut BeforeTheOneReread<'_>,
 ) -> Result<ShadowLedgerComputed, MountError> {
-    let mut referenced_by_candidates: BTreeSet<(u32, u64)> = current_records
+    let referenced_by_the_current_account: BTreeSet<(u32, u64)> = current_records
         .iter()
         .filter(|record| !record.is_released)
         .map(|record| (record.device.0, record.slot.0))
         .collect();
+    let mut referenced_by_candidates: BTreeSet<(u32, u64)> =
+        referenced_by_the_current_account.clone();
     // 候选 = 按实例表有效 ∧ txg ≥ F（D16（发布语义） 已定项 1：影子账的候选过滤是原来同取一圈窗口的六处之一，用户 2026-10-04 定 RGall
     // 一起退回整个环：窗口外的有效根照样是候选，它引用的槽豁免）。
     for root in roots
@@ -1561,9 +1564,14 @@
                 .collect(),
         );
     }
+    // C 那一份不豁免候选根引用的槽（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节「K1 / K2 T1 C 的候选豁免」那一行，
+    // 根环线 K4-1）：水位之上没施加的那次发布的根读得出、账读得出时它自己就是候选根（按实例表有效、txg ≥ F），豁免它引用的槽就是不护它的单元——
+    // 它比所选那一版新，它新分配的单元不在现行账里、回收门槛也罩不到，挂载之后当空闲槽发出去。只豁免现行账里仍分配着的槽：它们这时发不出去，
+    // 这次挂载里被释放、回收出来的时候，分配器按还没解开的这一份补隔离（`PoolAllocator::record_root_written_by_this_process`）；
+    // 不豁免它们，第九项与「已分配」就把同一个槽数两遍。
     for placement in placements_named_by_unapplied_journal_records {
         let key = (placement.device.0, placement.slot.0);
-        if referenced_by_candidates.contains(&key) || !isolated.insert(key) {
+        if referenced_by_the_current_account.contains(&key) || !isolated.insert(key) {
             continue;
         }
         allocator.isolate_abandoned(placement.device, placement.slot, placement.span);
@@ -1652,7 +1660,8 @@
 /// 水位之下的记录要么施加过、要么比所选那一版旧，不在其内。一条记录交出的落点：它点名的每个单元的两条位置条目（跨度按单元种类），
 /// 与它新根段里四条指针（树表、中央映射树根、实例表、分配记录树根；空指针不算）的位置条目——一次发布的每条记录都带整次发布的新根段
 /// （已定项 15），末条读不出时前几条的新根段照样说得出那次发布的树表与映射根在哪。落点不在那块盘的单元区里的（坏盘上的记录）不交。
-/// 只读内存里的记录、不读盘；读盘只问盘的字节数（落点在不在单元区）。
+/// 只读内存里的记录、不读盘；读盘只问盘的字节数（落点在不在单元区）。那几次发布的账里的落点另由
+/// [`placements_in_the_accounts_of_unapplied_publishes`] 读盘交出，调用方（`rebuilt_allocator`）并进同一份。
 #[allow(
     clippy::ptr_arg,
     reason = "placement_lies_in_the_unit_area_of_its_device 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
@@ -1719,6 +1728,62 @@
     named_by_ring_slot
 }
 
+/// C 的账那一份（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节「K2 T1 C 的盲区」那一行与第三节第 1 条，攻方 `C_ACCOUNT`）：
+/// 水位之上每一次没施加的发布（同一 (实例代号, checkpoint_txg) 的记录），用它读得出的记录里计数器最小的那一条的新根段拼一条根
+/// （每条记录都带整次发布的新根段，D23（journal 的角色与格式） 已定项 15；[`crate::recovery::root_assembled_from_the_new_root_segment_of_the_record`]），
+/// 按影子账同一份读法（[`placements_referenced_by_root`]）读它的账，账里的落点按那次发布的 txg 在根环里对应的槽（[`target_for_publish`]）归组交回，
+/// 调用方并进 C 的集合、与记录点名的那一份同一个时机解开。账读不出、解不开的那次发布不交：那时它的单元多半也验不过，交 F1 乙。
+/// 它补的是 C 只看读得出的记录点名了什么那一处盲区：那次发布的根槽与它的某一条记录（末条、中间一条或第一条）读回全零、单元读得出时，
+/// F1 乙不拒，缺的那条记录单独点名的单元又不在别的记录里；账里是那次发布引用的全部落点。水位与 C 同一个（[`placements_named_by_unapplied_journal_records`]）。
+#[allow(
+    clippy::ptr_arg,
+    reason = "placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
+)]
+fn placements_in_the_accounts_of_unapplied_publishes<Device: BlockDevice>(
+    devices: &Vec<(DeviceIdentity, Device)>,
+    unit_area_start: UnitAreaStart,
+    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
+    selected_version: &RootRecord,
+    slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
+) -> BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> {
+    // 记录表按 (实例代号, 计数器) 升序：同一次发布里先遇到的就是计数器最小的那一条。
+    let mut first_readable_record_of_each_publish: BTreeMap<
+        (InstanceGeneration, CheckpointTxg),
+        &crate::journal::JournalRecord,
+    > = BTreeMap::new();
+    for record in records.values().filter(|record| {
+        (record.instance, record.checkpoint_txg)
+            > (selected_version.instance, selected_version.checkpoint_txg)
+    }) {
+        first_readable_record_of_each_publish
+            .entry((record.instance, record.checkpoint_txg))
+            .or_insert(record);
+    }
+    let mut in_the_accounts_by_ring_slot: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> =
+        BTreeMap::new();
+    for ((_instance, checkpoint_txg), record) in first_readable_record_of_each_publish {
+        let root_of_the_publish =
+            crate::recovery::root_assembled_from_the_new_root_segment_of_the_record(
+                record,
+                selected_version.filesystem_identifier,
+            );
+        let Some(placements) =
+            placements_referenced_by_root(devices, unit_area_start, &root_of_the_publish)
+        else {
+            continue;
+        };
+        in_the_accounts_by_ring_slot
+            .entry(target_for_publish(checkpoint_txg, slots_per_region))
+            .or_default()
+            .extend(
+                placements
+                    .into_iter()
+                    .map(|(device, slot, span)| PlacementOnDevice { device, slot, span }),
+            );
+    }
+    in_the_accounts_by_ring_slot
+}
+
 /// C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算（用户 2026-09-28 定，被攻过零轮）：被抛弃根的树表或分配记录树读不出时，
 /// 不读分配记录、不读树表，按这条根现算它引用的全部落点：根记录直接指着的豁免单元（实例表链上认得出的每一片、
 /// 树表、中央映射树的每个节点）加中央映射树里每一条映射条目的落点（数据单元、打包单元、四棵树的节点，含分配记录树自己的节点）。
@@ -1787,7 +1852,9 @@
 /// 一条根引用着的落点，逐盘：带文件的一版从它那一版的分配记录里取（账里已释放的是它换下的上一版的，不算它引用）；
 /// **树表 0 条的一版**引用的就是根记录直接指着的那几个单元——实例表（连同链上后面各片）、树表，写过行的一版还有它自己那片
 /// 分配记录树节点（根指针住根记录那一项，C512（树表 0 条的一版上被换下的单元记在哪））——两盘同槽，不读分配记录树：
-/// 那一片读不出时它自己的落点照样认得出。读不出、解不开交回 `None`（调用方只计数，不拒绝挂载）；
+/// 那一片读不出时它自己的落点照样认得出。读不出、解不开交回 `None`，怎么办归调用方：影子账读被抛弃根计一次读不出、从中央映射现算，
+/// 仍不成重读一次，再不成看 B 的凭据放行或拒（[`placements_of_an_abandoned_root_whose_account_is_unreadable`]）；候选根读不出就当它什么都不豁免；
+/// C 的账那一份读不出就不加（[`placements_in_the_accounts_of_unapplied_publishes`]）；
 /// 解不开包括根记录里那几条指针的两条位置条目不同槽、槽号与跨度不在那块盘的单元区里（代码审阅第 32 条）。
 ///
 /// 树表 0 条那一臂不能省：写过行的树表 0 条根指着的是一片新实例表，它既不在任何分配记录树里、也不在 mkfs 那两个单元里，
@@ -2366,14 +2433,32 @@
     );
     // C694 的 C（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 1 条）：水位之上没施加的记录点名的落点与影子账同一次隔离、
     // 豁免同一个集合。影子账关着（只供测试）时它也不隔离：那一臂要的是「什么都不护」的对照。
+    // 同一份里再并上那几次没施加的发布的账里的落点（C 的账那一份，[`placements_in_the_accounts_of_unapplied_publishes`]），解开时机相同。
     let placements_named_by_unapplied_journal_records = match shadow_ledger {
-        ShadowLedger::On => placements_named_by_unapplied_journal_records(
-            devices,
-            unit_area_start,
-            records,
-            selected_version,
-            slots_per_region,
-        ),
+        ShadowLedger::On => {
+            let mut named_by_ring_slot = placements_named_by_unapplied_journal_records(
+                devices,
+                unit_area_start,
+                records,
+                selected_version,
+                slots_per_region,
+            );
+            for (ring_slot, placements_in_the_account) in
+                placements_in_the_accounts_of_unapplied_publishes(
+                    devices,
+                    unit_area_start,
+                    records,
+                    selected_version,
+                    slots_per_region,
+                )
+            {
+                named_by_ring_slot
+                    .entry(ring_slot)
+                    .or_default()
+                    .extend(placements_in_the_account);
+            }
+            named_by_ring_slot
+        }
         ShadowLedger::Off => BTreeMap::new(),
     };
     let shadow_ledger_computed = match shadow_ledger {
@@ -5967,8 +6052,10 @@
 
 /// F1 乙（用户 2026-10-07 定「缺记录就拒可写」；判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 6 条）：水位之上、没被施加的
 /// 每一次发布（同一 (实例代号, checkpoint_txg) 的记录），读得出的那几条的「本次发布内序号」不是 1..N 连着（I-8.9：一次发布的序号依次是 1..N），
-/// 或其中没有一条带末条标志，而它们点名的单元又有一个验不过（两份读不出或校验和对不上，与施加前验点名单元同一份判法
-/// `recovery::every_named_unit_of_the_record_verifies`）——交回这样的发布，按 (实例代号, checkpoint_txg) 升序。
+/// 或其中没有一条带末条标志，而它们点名的单元又有一个验不过——某个点名单元两份都读不出或校验和对不上（`F1_ANY`，判决
+/// `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T4 那一行与第三节第 1 条；`recovery::every_named_unit_of_the_record_has_a_copy_that_verifies`：
+/// 一份坏是单故障，按两份都要验过判会让写记录中途断电、根没落的那次发布一份副本坏时每次可写挂载都拒；施加前验点名单元仍两份都要验过，
+/// 不是这一份）——交回这样的发布，按 (实例代号, checkpoint_txg) 升序。
 /// 缺的那条记录点名了哪几个单元挂载时没有别的东西记着（C 隔离不到，判决第二节 K2「C 的盲区」那一行），而那次发布可能已落过根、
 /// 可能被下一次恢复择到：调用方重读一次仍这样就在取号之前拒可写。正常崩溃在写记录中途（单元在记录之前落盘，D16（发布语义） 已定项 7）
 /// 读得出的记录点名的单元验得过，不交回。水位与 C 同一个（`placements_named_by_unapplied_journal_records`）。
@@ -6009,7 +6096,9 @@
                     == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
             });
             let some_named_unit_does_not_verify = records_of_the_publish.iter().any(|record| {
-                !crate::recovery::every_named_unit_of_the_record_verifies(reader, record)
+                !crate::recovery::every_named_unit_of_the_record_has_a_copy_that_verifies(
+                    reader, record,
+                )
             });
             !(ordinals_run_from_one_without_a_gap && carries_a_last_record)
                 && some_named_unit_does_not_verify
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -3521,8 +3521,8 @@
     "所选根的 checkpoint_txg 是 u64 的最大值：接在它后面的那次发布取不到 txg";
 
 /// 一条记录点名的每个单元的两份（两条位置条目）都读得出、整单元 CRC-32C 都等于条目里的（前缀第四条「两份都验过才施加」，
-/// D23（journal 的角色与格式） 已定项 14）；点名项的单元种类不认识就算验不过。施加前验点名单元（[`replay_journal`]）与可写挂载判
-/// 「水位之上一次缺记录的发布点名的单元验不过」（`mount::publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`）同用这一份。
+/// D23（journal 的角色与格式） 已定项 14）；点名项的单元种类不认识就算验不过。只给施加前验点名单元（[`replay_journal`]）用；
+/// 可写挂载判「水位之上一次缺记录的发布点名的单元验不过」用的是另一份（[`every_named_unit_of_the_record_has_a_copy_that_verifies`]）。
 pub(crate) fn every_named_unit_of_the_record_verifies<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     record: &JournalRecord,
@@ -3531,18 +3531,50 @@
         let Some(unit_bytes) = unit_bytes_for_class(named.unit_class) else {
             return false;
         };
-        named.locations.iter().all(|location| {
-            reader
-                .read(
-                    location.device,
-                    location.slot.to_device_offset(),
-                    unit_bytes,
-                )
-                .is_some_and(|bytes| crc32_castagnoli(&bytes) == location.unit_checksum)
-        })
+        named
+            .locations
+            .iter()
+            .all(|location| copy_named_by_the_location_entry_verifies(reader, location, unit_bytes))
+    })
+}
+
+/// 一条记录点名的每个单元都至少有一份（一条位置条目）读得出、整单元 CRC-32C 等于条目里的；点名项的单元种类不认识就算验不过。
+/// 可写挂载判「水位之上一次缺记录的发布点名的单元验不过」（F1 乙，`mount::publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`）
+/// 只用这一份：某个点名单元两份都验不过才算验不过（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T4 那一行与第三节第 1 条，
+/// 攻方 `F1_ANY`）。一份副本坏是单故障（D22（单元原子性怎么合成） 已定项 8 射程），按「两份都验过」判会让写记录中途断电、根没落的那次发布
+/// 一份副本坏时每次可写挂载都拒、没有出口。施加前的点名单元验证照旧两份都要验过（[`every_named_unit_of_the_record_verifies`]），两处不共用。
+pub(crate) fn every_named_unit_of_the_record_has_a_copy_that_verifies<
+    Reader: PoolReader + ?Sized,
+>(
+    reader: &Reader,
+    record: &JournalRecord,
+) -> bool {
+    record.named.iter().all(|named| {
+        let Some(unit_bytes) = unit_bytes_for_class(named.unit_class) else {
+            return false;
+        };
+        named
+            .locations
+            .iter()
+            .any(|location| copy_named_by_the_location_entry_verifies(reader, location, unit_bytes))
     })
 }
 
+/// 点名项一条位置条目指的那一份读得出、整单元 CRC-32C 等于条目里的。
+fn copy_named_by_the_location_entry_verifies<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    location: &LocationEntry,
+    unit_bytes: usize,
+) -> bool {
+    reader
+        .read(
+            location.device,
+            location.slot.to_device_offset(),
+            unit_bytes,
+        )
+        .is_some_and(|bytes| crc32_castagnoli(&bytes) == location.unit_checksum)
+}
+
 /// 取前缀并施加（D23（journal 的角色与格式） 已定项 14 / 已定项 15）；返回扫描报告与施加之后的根。
 ///
 /// 链首锚在所选根覆盖的最后一条：所选根那次发布里带「本次发布末条」标志的那一条（已定项 14 注 1，读法乙；
@@ -3695,6 +3727,27 @@
     Ok((report, rebuilt))
 }
 
+/// 一条记录的新根段拼出的那次发布的根：每条记录都带整次发布的新根段（D23（journal 的角色与格式） 已定项 15），七个字段取记录里的，
+/// fsid 新根段里没有、取调用方给的。可写挂载读水位之上没施加的那次发布的账用它（`mount::placements_in_the_accounts_of_unapplied_publishes`）。
+/// [`replay_journal`] 施加末条之后拼根是同一个拼法、在那里就地写着：那一段被变异表多行逐字锚着，这一批不挪它。
+pub(crate) fn root_assembled_from_the_new_root_segment_of_the_record(
+    record: &JournalRecord,
+    filesystem_identifier: [u8; 16],
+) -> RootRecord {
+    RootRecord {
+        filesystem_identifier,
+        unmount_marker: record.new_unmount_marker,
+        instance: record.instance,
+        checkpoint_txg: record.checkpoint_txg,
+        tree_table: record.new_tree_table,
+        tree_identifier_watermark: record.new_tree_identifier_watermark,
+        rollback_floor: record.new_rollback_floor,
+        instance_table: record.new_instance_table,
+        mapping_root: record.new_mapping_root,
+        allocation_record_tree_root: record.new_allocation_record_tree_root,
+    }
+}
+
 /// 每棵树的 key 宽（码 2 头里的自述 key 宽要与它相符）；day-1 只注册的三棵没有节点要解。
 fn key_width_for_kind(kind: u16) -> Option<usize> {
     match kind {
@@ -4358,14 +4411,30 @@
 /// 而节点自己的落点也有记录：节点数从「每块盘一条根到叶的路径加根」起，按分完之后的记录现算的节点数调，到两者相等为止。
 /// 默认几何（4 GiB 盘、默认环）每盘 14（字节表五：28 条分配记录）；1024 MiB 环那套几何（单元区起点 66560）提交内生块多两个、每盘 16。
 ///
+/// 盘的字节数是块设备报的、或崩溃镜像自述的，不是不变量（正推「发现 6」，判决 `research/prompts/m3-batch33t-code-r1-main-verification.md`
+/// 第三节第 1 条「排演函数小盘不 panic」）：建排演用的空闲图之前逐盘判够不够长，不够就交错误。
+///
+/// # Errors
+/// 按 `device_sizes_in_bytes` 次序第一块完整槽数不到单元区起点的盘 ⇒ [`NewPoolFileCreationRehearsalRefused::DeviceEndsBeforeTheUnitAreaStart`]；
+/// 空分配器上分不出这几个落点（单元区装不下新池新建文件那十几个单元，这套几何上写不出带文件的一版）⇒
+/// [`NewPoolFileCreationRehearsalRefused::UnitAreaCannotHoldTheNewPoolFileCreation`]。
+///
 /// # Panics
-/// 空分配器上分不出这几个落点（单元区装不下新池新建文件那十几个单元）：mkfs 判过单元区够不够（`UnitAreaBeyondDevice`），
-/// 能挂、能走读的池都装得下；或节点数调了 [`ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS`] 轮还不相等。
-#[must_use]
+/// 节点数调了 [`ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS`] 轮还不相等。
 pub fn placements_on_each_device_after_the_new_pool_file_creation(
     device_sizes_in_bytes: &[(DeviceIdentity, u64)],
     unit_area_start: UnitAreaStart,
-) -> PlacementsOnEachDevice {
+) -> Result<PlacementsOnEachDevice, NewPoolFileCreationRehearsalRefused> {
+    if let Some((device, device_bytes)) = device_sizes_in_bytes.iter().find(|(_, device_bytes)| {
+        unit_area_slots_of_device_starting_at(*device_bytes, unit_area_start).is_err()
+    }) {
+        return Err(
+            NewPoolFileCreationRehearsalRefused::DeviceEndsBeforeTheUnitAreaStart {
+                device: *device,
+                device_bytes: *device_bytes,
+            },
+        );
+    }
     let geometry = AllocationRecordTreeGeometry::of_device_sizes_in_bytes(device_sizes_in_bytes);
     let new_pool_file_creation_txg = CheckpointTxg(1);
     // 新池新建文件的落点都在单元区开头的两段里（mkfs 与数据单元在第一段，提交内生块开在最低的全空段、即第二段）：空闲图只建到那两段为止，
@@ -4404,9 +4473,9 @@
                 rehearsing.allocate_commit_generated(footprint, new_pool_file_creation_txg)
             }
         };
-        placement.expect(
-            "空分配器上分得出新池新建文件那十几个单元：mkfs 判过单元区装得下（UnitAreaBeyondDevice），能挂、能走读的池都装得下",
-        );
+        placement
+            .map(|_placement| ())
+            .ok_or(NewPoolFileCreationRehearsalRefused::UnitAreaCannotHoldTheNewPoolFileCreation)
     };
     for unit in [
         TransactionUnit::Data(DataUnitIndexInFile(0)),
@@ -4414,7 +4483,7 @@
         TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST),
         TransactionUnit::InodeRoot,
     ] {
-        allocate(&mut allocator, unit);
+        allocate(&mut allocator, unit)?;
     }
     let units_before_the_allocation_record_tree = allocator.clone();
     let device_count = device_sizes_in_bytes.len();
@@ -4423,20 +4492,22 @@
     for _round in 0..ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS {
         let mut rehearsal = units_before_the_allocation_record_tree.clone();
         for _node in 0..allocation_record_tree_nodes {
-            allocate(&mut rehearsal, TransactionUnit::AllocationTree);
+            allocate(&mut rehearsal, TransactionUnit::AllocationTree)?;
         }
         for unit in [
             TransactionUnit::AccountingTree,
             TransactionUnit::MappingTree,
             TransactionUnit::TreeTable,
         ] {
-            allocate(&mut rehearsal, unit);
+            allocate(&mut rehearsal, unit)?;
         }
         let nodes_holding_the_records =
             crate::allocation_record_tree::nodes_holding_records(&geometry, rehearsal.records())
                 .len();
         if nodes_holding_the_records == allocation_record_tree_nodes {
-            return PlacementsOnEachDevice(rehearsal.records().len() / device_count);
+            return Ok(PlacementsOnEachDevice(
+                rehearsal.records().len() / device_count,
+            ));
         }
         allocation_record_tree_nodes = nodes_holding_the_records;
     }
@@ -4445,6 +4516,18 @@
     );
 }
 
+/// [`placements_on_each_device_after_the_new_pool_file_creation`] 排演不出来的两种情形（调用方据此报恢复失败，不 panic）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum NewPoolFileCreationRehearsalRefused {
+    /// 这块盘的完整槽数不到单元区起点（盘的字节数是块设备报的、或崩溃镜像自述的）。
+    DeviceEndsBeforeTheUnitAreaStart {
+        device: DeviceIdentity,
+        device_bytes: u64,
+    },
+    /// 空分配器上分不出新池新建文件那十几个单元：这套几何上写不出带文件的一版。
+    UnitAreaCannotHoldTheNewPoolFileCreation,
+}
+
 /// [`placements_on_each_device_after_the_new_pool_file_creation`] 调分配记录树节点数的轮数上界：猜的节点数每一轮只会因为节点自己的落点
 /// 多落进一片叶而加几个，推的，没量过会不会到第三轮。
 const ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS: usize = 8;
@@ -4554,18 +4637,41 @@
             )
         })
         .collect();
+    let not_one_placement_per_device = RecoveryFailure::InvariantViolated {
+        invariant: "E142 走读同款",
+        detail: "分配记录不是每个落点每盘各一条：各盘的（槽, 跨度, 代, 已释放）集合不同，或少于这套几何下新池新建文件之后每盘的落点数",
+    };
+    // 先判记录本身（下限取 0），再排演下限：排演要按盘的字节数建空闲图，记录判空、各盘不同这些不用它就判得出（正推「发现 6」）。
+    if !allocation_records_are_one_per_device(records, device_identities, PlacementsOnEachDevice(0))
+    {
+        return Err(not_one_placement_per_device);
+    }
+    let placements_on_each_device_at_least =
+        match placements_on_each_device_after_the_new_pool_file_creation(
+            &device_sizes_in_bytes,
+            unit_area_start,
+        ) {
+            Ok(placements_on_each_device_at_least) => placements_on_each_device_at_least,
+            Err(NewPoolFileCreationRehearsalRefused::DeviceEndsBeforeTheUnitAreaStart {
+                device,
+                device_bytes,
+            }) => {
+                return Err(RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart {
+                    device,
+                    device_bytes,
+                })
+            }
+            // 这套几何上写不出带文件的一版，分配记录就不够这一版的下限。
+            Err(NewPoolFileCreationRehearsalRefused::UnitAreaCannotHoldTheNewPoolFileCreation) => {
+                return Err(not_one_placement_per_device)
+            }
+        };
     if !allocation_records_are_one_per_device(
         records,
         device_identities,
-        placements_on_each_device_after_the_new_pool_file_creation(
-            &device_sizes_in_bytes,
-            unit_area_start,
-        ),
+        placements_on_each_device_at_least,
     ) {
-        return Err(RecoveryFailure::InvariantViolated {
-            invariant: "E142 走读同款",
-            detail: "分配记录不是每个落点每盘各一条：各盘的（槽, 跨度, 代, 已释放）集合不同，或少于这套几何下新池新建文件之后每盘的落点数",
-        });
+        return Err(not_one_placement_per_device);
     }
     Ok(())
 }
@@ -5298,15 +5404,17 @@
 }
 
 /// 所选那一版走读失败之后退回哪一版（F2 乙，用户 2026-10-07 定「三处都做」；判决 `research/prompts/m3-c694-r2-main-verification.md`
-/// 第三节第 7 条）：系统配置没见证过所选那一版时，退回根环里比所选根旧、不比系统配置见证到的那一次发布新的最新一条根；见证过、见证判不出、
-/// 或找不到那样一条根时 `None`（照报原来的失败）。恢复（[`recover`]）、可写挂载读阶段之后建上一版那一步（`mount`）、只读挂载择根
+/// 第三节第 7 条）：系统配置没见证过所选那一版时，退回系统配置见证到的那一版本身——根环里 (checkpoint_txg, 实例代号) 与计数器 = c_见证 的
+/// 那条记录恰好相等、读得出、比所选根旧的那条根（`F2_EXACT`，判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 那一行与
+/// 第三节第 1 条：不退到比它更旧的根，那样已确认的那一版根槽读回全零时会退过它、记录读不出时会退到被新实例抛弃了的根）；见证过、见证判不出、
+/// c_见证 = 0（没有见证到的记录）、那条记录读不出、或那条根读不出时 `None`（照报原来的失败）。恢复（[`recover`]）、可写挂载读阶段之后建上一版那一步（`mount`）、只读挂载择根
 /// （`mounted_read::mount_read_only`）三处都调这一份，退过去之后不重放、直接走那一版。所选那一版是施加过记录、由记录重建的
 /// （`selected_version` 不是 `chosen_root`）时不退：条款写的是「所选根走读失败」，施加过记录的那一版走不通条款没写（函数体第一句）。
 ///
 /// 「见证过」的判法取攻方 `strict`（判决同一条）：c_见证 = 每块盘两槽里自证过的系统配置 journal tail 的最大值（与可写挂载判据 N-配置
 /// 同一个读数，`mount::newer_publish_witness`）。所选那一版那次发布带末条标志的记录读得出（任一份）⇒ 它的计数器 ≤ c_见证 即见证过；
 /// 读不出而计数器 = c_见证 的那条记录读得出 ⇒ 它的 (checkpoint_txg, 实例代号) 不比所选那一版旧即见证过；两条都读不出当见证过、不退。
-/// 退到的上限：计数器 = c_见证 的那条记录读得出时取它的 (checkpoint_txg, 实例代号)，读不出时取所选根的。根环里的根按择根同一个序
+/// 退到哪一条：计数器 = c_见证 的那条记录的 (checkpoint_txg, 实例代号) 恰好相等的那条根。根环里的根按择根同一个序
 /// （checkpoint_txg 为主、实例代号破平局）比。
 ///
 /// 系统配置见证判不出（R2：有槽设备报读错、又没有一块盘两槽都自证过，[`system_configuration_witness_is_undecidable`]）时不退：
@@ -5385,8 +5493,10 @@
     if is_witnessed {
         return None;
     }
+    // 只退到见证到的那一版本身（`F2_EXACT`）：计数器 = c_见证 的那条记录读不出就不退；c_见证 = 0 时也走这一支——计数器从 1 起
+    // （`journal::FIRST_JOURNAL_COUNTER`），环里没有计数器 0 的记录。
+    let witnessed_publish = record_at_the_witnessed_counter?;
     let chosen = order_of_the_root_choice(chosen_root);
-    let ceiling = record_at_the_witnessed_counter.unwrap_or(chosen);
     readable_roots(
         reader,
         &system_configuration.immutable.region_devices,
@@ -5394,11 +5504,10 @@
         &system_configuration.immutable.filesystem_identifier,
     )
     .into_iter()
-    .filter(|root| {
+    .find(|root| {
         let order = order_of_the_root_choice(root);
-        order < chosen && order <= ceiling
+        order == witnessed_publish && order < chosen
     })
-    .max_by_key(order_of_the_root_choice)
 }
 
 /// 整条恢复路径。报出去的 `root` 恒是所选的那条根；施加记录之后走的是重建出来的根。所选那一版走读失败、系统配置没见证它时退回见证到的那一版
@@ -5544,9 +5653,11 @@
 mod allocation_records_per_device_tests {
     use super::{
         allocation_records_are_one_per_device,
-        placements_on_each_device_after_the_new_pool_file_creation, PlacementsOnEachDevice,
+        allocation_records_are_one_placement_per_device_on_every_device,
+        placements_on_each_device_after_the_new_pool_file_creation,
+        NewPoolFileCreationRehearsalRefused, PlacementsOnEachDevice, PoolReader, RecoveryFailure,
     };
-    use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
+    use crate::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
     use crate::allocator::{AllocationRecord, UnitAreaStart};
 
     const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
@@ -5568,6 +5679,7 @@
             &both_devices_of_four_gibibytes(),
             UnitAreaStart::of_the_default_journal_ring(),
         )
+        .expect("4 GiB 盘够长到默认环的单元区起点、装得下新池新建文件")
     }
 
     fn unit_area_start_after_a_one_gibibyte_journal_ring() -> UnitAreaStart {
@@ -5625,7 +5737,7 @@
                 &both_devices_of_four_gibibytes(),
                 unit_area_start_after_a_one_gibibyte_journal_ring(),
             ),
-            PlacementsOnEachDevice(16),
+            Ok(PlacementsOnEachDevice(16)),
             "单元区起点 66560：mkfs 与数据单元在槽 81 那片叶（W = 812），提交内生块在槽 82 那片叶，分配记录树每盘多一片叶、多两个节点，提交内生块 13 个"
         );
     }
@@ -5659,7 +5771,8 @@
         let placements_on_each_device = placements_on_each_device_after_the_new_pool_file_creation(
             &both_devices_of_four_gibibytes(),
             unit_area_start,
-        );
+        )
+        .expect("4 GiB 盘够长到 1024 MiB 环的单元区起点、装得下新池新建文件");
         assert!(allocation_records_are_one_per_device(
             &shape_with_placements_on_each_device(
                 unit_area_start.slot().0,
@@ -5754,6 +5867,101 @@
             placements_on_each_device_of_the_default_geometry(),
         ));
     }
+
+    /// 1 MiB 的盘：默认环的单元区起点（槽 50176）在它末尾之后。
+    const ONE_MEBIBYTE: u64 = 1 << 20;
+
+    /// 只交得出「池里有哪几块盘、每块多大」的读者：盘 0 是 4 GiB，盘 1 只有 1 MiB。
+    struct PoolWhoseSecondDeviceEndsBeforeTheUnitAreaStart;
+
+    impl PoolReader for PoolWhoseSecondDeviceEndsBeforeTheUnitAreaStart {
+        fn device_identities(&self) -> Vec<DeviceIdentity> {
+            BOTH_DEVICES.to_vec()
+        }
+        fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
+            match device {
+                DeviceIdentity(0) => Some(FOUR_GIBIBYTES),
+                DeviceIdentity(1) => Some(ONE_MEBIBYTE),
+                DeviceIdentity(_) => None,
+            }
+        }
+        fn read(
+            &self,
+            _device: DeviceIdentity,
+            _offset: DeviceOffsetInBytes,
+            _length: usize,
+        ) -> Option<Vec<u8>> {
+            unreachable!("分配记录每盘一条那一道判只读盘的身份与大小，不读盘上的字节")
+        }
+        fn journal_record_offsets_hint(
+            &self,
+            _device: DeviceIdentity,
+            _ring_start: DeviceOffsetInBytes,
+            _ring_bytes: u64,
+        ) -> Option<Vec<DeviceOffsetInBytes>> {
+            unreachable!("分配记录每盘一条那一道判不扫 journal 环")
+        }
+    }
+
+    /// 正推「发现 6」：一块盘比单元区起点短时，排演新池新建文件交回那块盘，不在建空闲图那一步 panic。
+    /// 判别力：排演不先判盘长（变异「排演不判盘短于单元区起点」）时这一条 panic、红。
+    #[test]
+    fn rehearsal_on_a_device_shorter_than_the_unit_area_start_returns_that_device_instead_of_panicking(
+    ) {
+        assert_eq!(
+            placements_on_each_device_after_the_new_pool_file_creation(
+                &[
+                    (DeviceIdentity(0), FOUR_GIBIBYTES),
+                    (DeviceIdentity(1), ONE_MEBIBYTE)
+                ],
+                UnitAreaStart::of_the_default_journal_ring(),
+            ),
+            Err(
+                NewPoolFileCreationRehearsalRefused::DeviceEndsBeforeTheUnitAreaStart {
+                    device: DeviceIdentity(1),
+                    device_bytes: ONE_MEBIBYTE,
+                }
+            ),
+            "盘 1 只有 1 MiB，到不了默认环的单元区起点"
+        );
+    }
+
+    /// 同一块短盘上走读判分配记录每盘一条：记录是空的先判记录（不必排演就判得出），每盘一条而盘短报那块盘到不了单元区起点；两样都不 panic。
+    /// 判别力：同上一条的变异，第二个断言 panic、红；排演挪回判记录之前（变异「排演先于判记录」）时第一个断言报的是盘短、红——
+    /// 「不作为实参先求值」由第一个断言钉着。
+    #[test]
+    fn walk_judging_allocation_records_on_a_device_shorter_than_the_unit_area_start_returns_an_error_instead_of_panicking(
+    ) {
+        assert_eq!(
+            allocation_records_are_one_placement_per_device_on_every_device(
+                &PoolWhoseSecondDeviceEndsBeforeTheUnitAreaStart,
+                UnitAreaStart::of_the_default_journal_ring(),
+                &[],
+                &BOTH_DEVICES,
+            ),
+            Err(RecoveryFailure::InvariantViolated {
+                invariant: "E142 走读同款",
+                detail: "分配记录不是每个落点每盘各一条：各盘的（槽, 跨度, 代, 已释放）集合不同，或少于这套几何下新池新建文件之后每盘的落点数",
+            }),
+            "一条分配记录都没有：先判记录，报每盘一条不成立"
+        );
+        assert_eq!(
+            allocation_records_are_one_placement_per_device_on_every_device(
+                &PoolWhoseSecondDeviceEndsBeforeTheUnitAreaStart,
+                UnitAreaStart::of_the_default_journal_ring(),
+                &[
+                    record(DeviceIdentity(0), 50176, false),
+                    record(DeviceIdentity(1), 50176, false)
+                ],
+                &BOTH_DEVICES,
+            ),
+            Err(RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart {
+                device: DeviceIdentity(1),
+                device_bytes: ONE_MEBIBYTE,
+            }),
+            "每盘一条而盘 1 短于单元区起点：报那块盘，不 panic"
+        );
+    }
 }
 
 /// 盘上读来的分配记录对这个池的几何判的那四样（panic 面普查 R6 / R8 / R9）：设备身份、槽号下界、跨度上界、
diff --git a/crates/singlefs-harness/src/mount_transparency.rs b/crates/singlefs-harness/src/mount_transparency.rs
--- a/crates/singlefs-harness/src/mount_transparency.rs
+++ b/crates/singlefs-harness/src/mount_transparency.rs
@@ -32,8 +32,9 @@
     choose_root, choose_system_configuration, chosen_system_configuration_slot_bytes,
     effective_rollback_floor, highest_root_instance, highest_root_txg, instance_table_of_root,
     readable_roots, recover, replay_journal, root_record_bytes_of_the_version, scan_journal,
-    tree_table_entry_bytes_of_root, verified_system_configuration_slots, walk_to_file,
-    JournalPolicy, PoolReader, RecoveryOutcome, RecoveryReport,
+    tree_table_entry_bytes_of_root, verified_system_configuration_slots,
+    version_witnessed_by_the_system_configuration_to_fall_back_to, walk_to_file, JournalPolicy,
+    PoolReader, RecoveryOutcome, RecoveryReport,
 };
 use singlefs_core::root_record::{RootRecord, UnmountMarker};
 use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableConfiguration};
@@ -129,8 +130,12 @@
 
 /// 对 `reader` 跑一次只读恢复（看 journal），再读挂载透明要比的字段。`compared` 定拿哪一版来比（[`ComparedVersion`]）。
 ///
+/// 拿来比的那一版（与恢复报告里的 `effective_root` 那一版）按恢复同一套读法再算一遍：择根、重放；重放出来的那一版走读失败时，
+/// 照恢复同一份 F2 判法（`recovery::version_witnessed_by_the_system_configuration_to_fall_back_to`）退到系统配置见证到的那一版，
+/// 那一版走得通就是它（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第三节第 1 条：F2 退了就比退到的那一版）。
+///
 /// # Panics
-/// 同一份盘、同一套读法再择一次根、再重放一次，落到的那一版与恢复报告里的不同：两次读同一份内存盘，读法是同一组函数，走不到。
+/// 同一份盘、同一套读法再算一遍，落到的那一版与恢复报告里的不同：两次读同一份内存盘，读法是同一组函数，走不到。
 #[must_use]
 pub fn read_recovery_fields(
     reader: &dyn PoolReader,
@@ -198,7 +203,7 @@
     let Some(chosen) = choose_root(reader, &system_configuration) else {
         return fields;
     };
-    let Ok((_, recovered_root)) = replay_journal(
+    let Ok((_, replayed_root)) = replay_journal(
         reader,
         &chosen,
         immutable.sizes.journal_ring_bytes,
@@ -207,10 +212,27 @@
     ) else {
         return fields;
     };
+    let walks_to_a_file_or_no_file = |root: &RootRecord| {
+        let mut mapping_fallbacks = 0usize;
+        walk_to_file(reader, root, &mut mapping_fallbacks).is_ok()
+    };
+    let recovered_root = if walks_to_a_file_or_no_file(&replayed_root) {
+        replayed_root
+    } else {
+        version_witnessed_by_the_system_configuration_to_fall_back_to(
+            reader,
+            &system_configuration,
+            &chosen,
+            &replayed_root,
+            &records,
+        )
+        .filter(walks_to_a_file_or_no_file)
+        .unwrap_or(replayed_root)
+    };
     assert_eq!(
         Some((recovered_root.instance, recovered_root.checkpoint_txg)),
         fields.report.effective_root,
-        "同一份盘上再择一次根、再重放一次，要落到恢复报告里那一版"
+        "同一份盘上再择一次根、再重放一次（走读失败时照 F2 退到见证到的那一版），要落到恢复报告里那一版"
     );
     fields.compared_version = match compared {
         ComparedVersion::TheRecoveredVersion => Some(version_fields(
diff --git a/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs b/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs
--- a/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs
+++ b/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs
@@ -16,8 +16,14 @@
 use singlefs_core::block_device::{
     BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
 };
+use singlefs_core::journal::{record_offset, JournalRecordPlaceInPublish};
 use singlefs_core::mount::Mounted;
 use singlefs_core::mounted_session::MountedSession;
+use singlefs_core::recovery::{
+    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, scan_journal,
+    RootRingSlotReading,
+};
+use singlefs_core::root_ring::slot_offset;
 use singlefs_harness::memory_pool::{
     writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
 };
@@ -200,6 +206,72 @@
         self
     }
 
+    /// 加上 `image` 上 (实例, txg) 为 `key` 的那条根所在的根槽（根环里自证过、正是这条根的那一槽，一块盘一份）。
+    ///
+    /// # Panics
+    /// `image` 上择不到系统配置，或根环里没有自证过的这条根。
+    #[must_use]
+    pub fn with_the_root_slot_of(
+        mut self,
+        image: &MemoryPool,
+        key: (InstanceGeneration, CheckpointTxg),
+    ) -> Self {
+        let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
+        let sizes = &configuration.immutable.sizes;
+        let ring_slot = every_root_ring_slot(sizes)
+            .into_iter()
+            .find(|ring_slot| {
+                matches!(
+                    read_root_ring_slot(
+                        image,
+                        &configuration.immutable.region_devices,
+                        sizes,
+                        &configuration.immutable.filesystem_identifier,
+                        *ring_slot,
+                    ),
+                    RootRingSlotReading::SelfVerified(root) if (root.instance, root.checkpoint_txg) == key
+                )
+            })
+            .expect("根环里有自证过的这条根");
+        self.0.push((
+            configuration.immutable.region_devices
+                [usize::try_from(ring_slot.region).expect("区域号")],
+            slot_offset(ring_slot, sizes.fixed_structure_slot_spacing).0,
+            u64::from(sizes.physical_block_size),
+        ));
+        self
+    }
+
+    /// 加上 `image` 上 `key` 那次发布带末条标志的那条记录（每块盘同一个偏移各一份）。
+    ///
+    /// # Panics
+    /// `image` 上择不到系统配置，或环里读不出那次发布带末条标志的记录。
+    #[must_use]
+    pub fn with_the_last_record_of_the_publish(
+        mut self,
+        image: &MemoryPool,
+        key: (InstanceGeneration, CheckpointTxg),
+    ) -> Self {
+        let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
+        let last_record = scan_journal(image, &configuration)
+            .into_values()
+            .find(|record| {
+                (record.instance, record.checkpoint_txg) == key
+                    && record.place_in_publish
+                        == JournalRecordPlaceInPublish::LastRecordOfThePublish
+            })
+            .expect("环里读得出那次发布带末条标志的记录");
+        let offset = record_offset(
+            last_record.counter,
+            configuration.immutable.sizes.journal_ring_bytes,
+        );
+        for device in image.devices.keys() {
+            self.0
+                .push((*device, offset.0, singlefs_format::JOURNAL_RECORD_BYTES));
+        }
+        self
+    }
+
     fn on_device(&self, device: DeviceIdentity) -> Vec<(u64, u64)> {
         self.0
             .iter()
diff --git a/crates/singlefs-harness/tests/read_recovery_fields_follow_the_fallback_to_the_witnessed_version.rs b/crates/singlefs-harness/tests/read_recovery_fields_follow_the_fallback_to_the_witnessed_version.rs
new file mode 100644
--- /dev/null
+++ b/crates/singlefs-harness/tests/read_recovery_fields_follow_the_fallback_to_the_witnessed_version.rs
@@ -0,0 +1,149 @@
+//! 挂载透明读字段（`mount_transparency::read_recovery_fields`）在恢复按 F2 乙退回见证到的那一版时照同一份判法算期望
+//! （判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 那一行「`read_recovery_fields` 断言再择根、再重放落到
+//! `report.effective_root`，F2 退回后两边不等、panic」与第三节第 1 条）：两种 `ComparedVersion` 都不 panic，拿来比的那一版与恢复报告一致。
+//!
+//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次覆盖写 R（P 是它之前那一版，已确认）。两形：
+//! - S1（用户 2026-10-07 定 F2 时接受的代价那一形）：R 已确认，较新的系统配置槽（R 那次轮换写的）与 R 的单元读回全零，恢复读回 P；
+//! - 设计里那一退：崩在 R 的根槽落盘之后、轮换之前，R 的单元读回全零，恢复读回 P。
+mod common_unwitnessed_publish;
+
+use common_unwitnessed_publish::{
+    devices_reading_back_all_zero_on, record_one_publish_after_overwrites, PublishRecordedAsR,
+    RangesReadingBackAllZero, RecordedPublish,
+};
+use singlefs_core::recovery::RecoveryOutcome;
+use singlefs_harness::memory_pool::MemoryPool;
+use singlefs_harness::mount_transparency::{
+    read_recovery_fields, ComparedVersion, PosixContentRead,
+};
+use singlefs_harness::segments::StepKind;
+
+const OVERWRITES_BEFORE_R: u64 = 5;
+
+/// S1：R 整次发布都落了，读的那一刻 R 那次轮换写的两个系统配置槽与 R 的单元读回全零。交回盘面与读回全零的那几段。
+fn confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero(
+    recorded: &RecordedPublish,
+) -> (MemoryPool, RangesReadingBackAllZero) {
+    let rotation_of_r: Vec<_> = recorded
+        .writes
+        .iter()
+        .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
+        .collect();
+    (
+        recorded.image_with_the_whole_publish_persisted(),
+        RangesReadingBackAllZero::default()
+            .with_the_writes(&recorded.unit_writes())
+            .with_the_writes(&rotation_of_r),
+    )
+}
+
+/// 设计里那一退：崩在 R 的根槽落盘之后、轮换之前，读的那一刻 R 的单元读回全零。交回盘面与读回全零的那几段。
+fn unwitnessed_root_whose_units_read_back_zero(
+    recorded: &RecordedPublish,
+) -> (MemoryPool, RangesReadingBackAllZero) {
+    (
+        recorded.crash_image_with_the_root_persisted_and_the_rotation_lost(),
+        RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes()),
+    )
+}
+
+/// 比恢复落到的那一版：读字段不 panic，恢复读回 P，拿来比的那一版就是 P、内容与恢复报告读出的相同。
+fn assert_the_recovered_version_read_is_the_witnessed_version(
+    recorded: &RecordedPublish,
+    image: &MemoryPool,
+    ranges: &RangesReadingBackAllZero,
+) {
+    let devices = devices_reading_back_all_zero_on(image, ranges);
+    let fields = read_recovery_fields(&devices, ComparedVersion::TheRecoveredVersion);
+    assert_eq!(
+        fields.report.outcome,
+        RecoveryOutcome::FileRead {
+            root: recorded.previous_key,
+            content: recorded.previous_content.clone(),
+        },
+        "恢复按 F2 退回见证到的 P、读回 P 的内容"
+    );
+    let compared = fields
+        .compared_version
+        .expect("恢复落到了一条根：有拿来比的那一版");
+    assert_eq!(
+        Some((compared.root.instance, compared.root.checkpoint_txg)),
+        fields.report.effective_root,
+        "拿来比的那一版就是恢复报告里那一版"
+    );
+    assert_eq!(
+        compared.content,
+        PosixContentRead::File(recorded.previous_content.clone()),
+        "拿来比的那一版读出的内容与恢复报告读出的相同"
+    );
+}
+
+/// 比恢复落到的那一版之前那一版：读字段不 panic，拿来比的那一版比恢复退到的 P 旧。
+fn assert_the_version_before_the_recovered_one_is_older_than_the_witnessed_version(
+    recorded: &RecordedPublish,
+    image: &MemoryPool,
+    ranges: &RangesReadingBackAllZero,
+) {
+    let devices = devices_reading_back_all_zero_on(image, ranges);
+    let fields = read_recovery_fields(&devices, ComparedVersion::TheVersionBeforeTheRecoveredOne);
+    assert_eq!(
+        fields.report.effective_root,
+        Some(recorded.previous_key),
+        "恢复按 F2 退回见证到的 P"
+    );
+    let compared = fields
+        .compared_version
+        .expect("环里有比 P 更旧的根：有拿来比的那一版");
+    assert!(
+        (compared.root.checkpoint_txg, compared.root.instance)
+            < (recorded.previous_key.1, recorded.previous_key.0),
+        "拿来比的是恢复退到的 P 之前那一版，不是被退掉的 R 之前那一版（P）：{:?}",
+        (compared.root.instance, compared.root.checkpoint_txg)
+    );
+}
+
+/// 判别力：读字段不照 F2 退（变异「读字段再算一遍时不照 F2 退」）时它的断言 panic、这一条红。
+#[test]
+fn reading_the_recovered_version_of_a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_compares_the_witnessed_version(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let (image, ranges) =
+        confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero(&recorded);
+    assert_the_recovered_version_read_is_the_witnessed_version(&recorded, &image, &ranges);
+}
+
+/// 判别力：同上一条的变异。
+#[test]
+fn reading_the_version_before_the_recovered_one_of_a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_compares_the_version_before_the_witnessed_one(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let (image, ranges) =
+        confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero(&recorded);
+    assert_the_version_before_the_recovered_one_is_older_than_the_witnessed_version(
+        &recorded, &image, &ranges,
+    );
+}
+
+/// 判别力：同上。
+#[test]
+fn reading_the_recovered_version_of_an_unwitnessed_root_whose_units_read_back_zero_compares_the_witnessed_version(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let (image, ranges) = unwitnessed_root_whose_units_read_back_zero(&recorded);
+    assert_the_recovered_version_read_is_the_witnessed_version(&recorded, &image, &ranges);
+}
+
+/// 判别力：同上。
+#[test]
+fn reading_the_version_before_the_recovered_one_of_an_unwitnessed_root_whose_units_read_back_zero_compares_the_version_before_the_witnessed_one(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let (image, ranges) = unwitnessed_root_whose_units_read_back_zero(&recorded);
+    assert_the_version_before_the_recovered_one_is_older_than_the_witnessed_version(
+        &recorded, &image, &ranges,
+    );
+}
diff --git a/crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs b/crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs
--- a/crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs
+++ b/crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs
@@ -1,14 +1,19 @@
 //! F2 乙（用户 2026-10-07 定「三处都做」；判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 7 条）：所选那一版走读失败、系统配置没见证它时，
 //! 退回系统配置见证到的那一版；恢复（`recovery::recover`）、可写挂载读阶段之后建上一版那一步、只读挂载（`mounted_read::mount_read_only`）三处同一份判法
 //! （`recovery::version_witnessed_by_the_system_configuration_to_fall_back_to`）。「见证过」照攻方 `strict`：所选那一版带末条标志的记录读得出 ⇒
-//! 计数器 ≤ c_见证；读不出 ⇒ 拿计数器 = c_见证 的那条记录比 (txg, 实例)；两条都读不出当见证过、不退。
+//! 计数器 ≤ c_见证；读不出 ⇒ 拿计数器 = c_见证 的那条记录比 (txg, 实例)；两条都读不出当见证过、不退。退只退到见证到的那一版本身
+//! （`F2_EXACT`，判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 那一行与第三节第 1 条）：根环里与计数器 = c_见证 的那条记录
+//! (txg, 实例) 恰好相等、读得出的那条根；那条记录读不出、或那条根读不出，都不退。
 //!
-//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次覆盖写 R（P 是它之前那一版）。
+//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次覆盖写 R（P 是它之前那一版，已确认）。
 //! - 第二轮攻方 5.2 两形：崩在 R 的根槽落盘之后、系统配置轮换之前；取新实例的那次挂载里 R 的根槽与单元读回全零（`ur`）或连 R 的记录也读回全零
 //!   （`urJ`），那次挂载写行那次发布的根落盘之前断电，R 的单元已被那次挂载写掉；断电之后三处都读回 P。`ur` 那一形今天的 C 隔离得住 R 的单元，
 //!   这里关着影子账（只供测试的开关 `ShadowLedger::Off`，连 C 一起关）造出 R 的单元被写掉的盘面；`urJ` 那一形 C 本来就看不见。
 //! - 5.3 两形（R 已确认：整次发布连系统配置轮换都落了）：S2——R 的末条（唯一一条）记录与单元读回全零，两条都读不出、当见证过、不退，照报读不出；
 //!   S1——较新的那个系统配置槽（R 那次轮换写的）与 R 的单元读回全零，c_见证 落回 P，R 算没见证、读回 P：用户 2026-10-07 定案接受的代价。
+//! - 代码轮攻方（`research/prompts/m3-batch33t-code-r1-opus-output.md` 第一节）两形：A——崩在 R 的根槽之后、轮换之前，读的那一刻 R 的单元与 P 的根槽
+//!   读回全零（见证到的就是 P，P 的根读不出）；B——取新实例的那次挂载里 `ur`、崩在那次挂载写行那次发布的根槽之后，下一次读时写行那次发布的单元
+//!   与 P 的末条记录（计数器 = c_见证 的那一条）读回全零。两形都照报读不出，不退到 P 之前那一版、也不退到被新实例抛弃的 R。
 mod common_unwitnessed_publish;
 
 use common_unwitnessed_publish::{
@@ -17,23 +22,26 @@
     recorded_devices_reading_back_all_zero_on, units_of_r_overwritten_on, PublishRecordedAsR,
     RangesReadingBackAllZero, RecordedPublish,
 };
-use singlefs_core::address::DeviceIdentity;
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::mount::{mount_writable, mount_writable_with_test_only_switches, ShadowLedger};
 use singlefs_core::mounted_read::mount_read_only;
 use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
-use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool};
+use singlefs_core::root_record::RootRecord;
+use singlefs_harness::memory_pool::{
+    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite,
+};
 use singlefs_harness::segments::StepKind;
 use singlefs_harness::SharedStream;
 
 const OVERWRITES_BEFORE_R: u64 = 5;
 
-/// 取新实例的那次挂载里 `ranges` 读回全零、影子账按 `shadow_ledger`，写行那次发布的根落盘之前断电：交回断电那一刻的盘面。
-fn image_of_a_power_loss_before_the_row_root_of_the_mount_that_took_a_new_instance(
+/// 取新实例的那次挂载里 `ranges` 读回全零、影子账按 `shadow_ledger`：交回那次挂载的全部写（按落盘次序）与写行那次发布的根槽写在其中的下标。
+fn writes_of_the_mount_that_took_a_new_instance(
     recorded: &RecordedPublish,
     ranges: &RangesReadingBackAllZero,
     shadow_ledger: ShadowLedger,
-) -> MemoryPool {
+) -> (Vec<RetainedWrite>, usize) {
     let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
     let stream = SharedStream::retaining_contents();
     let mut devices = recorded_devices_reading_back_all_zero_on(&crash_image, ranges, &stream);
@@ -52,7 +60,18 @@
         .iter()
         .position(|write| write.kind == StepKind::RootRecordFua)
         .expect("那次挂载写行那次发布有根槽写");
-    let mut image = crash_image;
+    (mount_writes, row_root)
+}
+
+/// 取新实例的那次挂载里 `ranges` 读回全零、影子账按 `shadow_ledger`，写行那次发布的根落盘之前断电：交回断电那一刻的盘面。
+fn image_of_a_power_loss_before_the_row_root_of_the_mount_that_took_a_new_instance(
+    recorded: &RecordedPublish,
+    ranges: &RangesReadingBackAllZero,
+    shadow_ledger: ShadowLedger,
+) -> MemoryPool {
+    let (mount_writes, row_root) =
+        writes_of_the_mount_that_took_a_new_instance(recorded, ranges, shadow_ledger);
+    let mut image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
     image.apply_writes(&mount_writes[..row_root]);
     assert_ne!(
         units_of_r_overwritten_on(recorded, &image),
@@ -62,22 +81,24 @@
     image
 }
 
-/// 三处都读回 P：恢复读回 P 的内容、落到 P；只读挂载择到并打开 P；可写挂载建在 P 上。
+/// 三处都读回 P：`image` 上 `ranges` 读回全零时，恢复读回 P 的内容、落到 P；只读挂载择到并打开 P；可写挂载建在 P 上。
 fn assert_recovery_and_both_mounts_read_back_the_previous_version(
     recorded: &RecordedPublish,
     image: &MemoryPool,
+    ranges: &RangesReadingBackAllZero,
 ) {
-    let report = recover(image, JournalPolicy::Consult);
+    let devices = devices_reading_back_all_zero_on(image, ranges);
+    let report = recover(&devices, JournalPolicy::Consult);
     assert_eq!(
         report.outcome,
         RecoveryOutcome::FileRead {
             root: recorded.previous_key,
             content: recorded.previous_content.clone(),
         },
-        "恢复：所选那一版 R 走读失败、系统配置没见证它，退回见证到的 P、读回 P 的内容（今天：Failed(UnitUnreadable)）"
+        "恢复：所选那一版 R 走读失败、系统配置没见证它，退回见证到的 P、读回 P 的内容（F2 之前：Failed(UnitUnreadable)）"
     );
     assert_eq!(report.effective_root, Some(recorded.previous_key));
-    let read_only = mount_read_only(image).expect("只读挂载退回 P、打得开");
+    let read_only = mount_read_only(&devices).expect("只读挂载退回 P、打得开");
     assert_eq!(
         (
             read_only.effective_root.instance,
@@ -86,8 +107,8 @@
         recorded.previous_key,
         "只读挂载：退回 P"
     );
-    let mut devices = devices_reading_back_all_zero_on(image, &RangesReadingBackAllZero::default());
-    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
+    let mut devices_of_the_mount = devices_reading_back_all_zero_on(image, ranges);
+    let mounted = mount_writable(&recorded.pool.parameters, &mut devices_of_the_mount)
         .expect("可写挂载退回 P、建得出上一版");
     assert_eq!(
         (
@@ -99,6 +120,39 @@
     );
 }
 
+/// 三处都照报读不出：`image` 上 `ranges` 读回全零时，恢复落到 `chosen` 上报走读失败、只读挂载打不开、可写挂载在任何写之前拒（两块盘逐字节不变）。
+fn assert_recovery_and_both_mounts_report_the_chosen_version_unreadable(
+    recorded: &RecordedPublish,
+    image: &MemoryPool,
+    ranges: &RangesReadingBackAllZero,
+    chosen: (InstanceGeneration, CheckpointTxg),
+) {
+    let devices = devices_reading_back_all_zero_on(image, ranges);
+    let report = recover(&devices, JournalPolicy::Consult);
+    assert!(
+        matches!(
+            report.outcome,
+            RecoveryOutcome::Failed { root: Some(root), .. } if root == chosen
+        ),
+        "恢复落到所选的 {chosen:?}、照报走读失败，不悄悄读回别的一版：{:?}",
+        report.outcome
+    );
+    assert!(
+        mount_read_only(&devices).is_err(),
+        "只读挂载照报打不开 {chosen:?}，不退到别的一版"
+    );
+    let mut devices_of_the_mount = devices_reading_back_all_zero_on(image, ranges);
+    assert!(
+        mount_writable(&recorded.pool.parameters, &mut devices_of_the_mount).is_err(),
+        "可写挂载照拒，不在别的一版上取号"
+    );
+    assert!(
+        image_of_devices_reading_back_all_zero(&devices_of_the_mount, recorded.pool.device_bytes)
+            == *image,
+        "可写挂载在任何写之前拒：两块盘逐字节不变"
+    );
+}
+
 /// `ur`：R 的记录读得出（末条，计数器大于 c_见证）。判别力：「末条读得出就当见证过」（变异「F2 末条那一支恒见证过」）时三处都不退、这一条红。
 #[test]
 fn a_root_whose_units_were_overwritten_while_its_record_reads_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts(
@@ -113,7 +167,11 @@
         &ranges,
         ShadowLedger::Off,
     );
-    assert_recovery_and_both_mounts_read_back_the_previous_version(&recorded, &image);
+    assert_recovery_and_both_mounts_read_back_the_previous_version(
+        &recorded,
+        &image,
+        &RangesReadingBackAllZero::default(),
+    );
 }
 
 /// `urJ`：R 的记录那次也读回全零，取新实例的那次挂载把它的记录槽写成自己的写行记录（计数器接在 P 后面）：R 的末条读不出，
@@ -133,7 +191,89 @@
         &ranges,
         ShadowLedger::On,
     );
-    assert_recovery_and_both_mounts_read_back_the_previous_version(&recorded, &image);
+    assert_recovery_and_both_mounts_read_back_the_previous_version(
+        &recorded,
+        &image,
+        &RangesReadingBackAllZero::default(),
+    );
+}
+
+/// 设计里那一退（代码轮攻方 `Ru`）：崩在 R 的根槽之后、轮换之前，读的那一刻 R 的单元读回全零：R 走读失败、没见证，见证到的 P 根与单元都好，
+/// 三处都退到 P：`F2_EXACT` 不把设计那一退收掉。判别力：末条那一支恒见证过（变异「F2 末条那一支恒见证过（设计那一退）」）时三处都不退、这一条红。
+#[test]
+fn a_root_whose_units_read_back_zero_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let ranges = RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes());
+    assert_recovery_and_both_mounts_read_back_the_previous_version(
+        &recorded,
+        &crash_image,
+        &ranges,
+    );
+}
+
+/// 代码轮攻方 A 形：崩在 R 的根槽之后、轮换之前，读的那一刻 R 的单元与见证到的 P 的根槽读回全零。P 的记录与单元都好，只是这一刻 P 的根读不出：
+/// 只退到见证到的那一版本身，P 的根读不出就不退——三处照报 R 读不出，不退过已确认的 P、落到 P 之前那一版（F2_EXACT 之前三处都落到它，
+/// 可写挂载还在它上面取号、把 P 抛弃）。判别力：退回「比所选根旧、不超过见证上限的最新可读根」（变异「F2 退到见证上限之下最新的可读根」）时
+/// 三处都读回 P 之前那一版、这一条红。
+#[test]
+fn a_root_whose_units_and_the_witnessed_versions_root_slot_read_back_zero_is_reported_unreadable_in_recovery_and_both_mounts(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&recorded.unit_writes())
+        .with_the_root_slot_of(&crash_image, recorded.previous_key);
+    assert_recovery_and_both_mounts_report_the_chosen_version_unreadable(
+        &recorded,
+        &crash_image,
+        &ranges,
+        recorded.key,
+    );
+}
+
+/// 代码轮攻方 B 形：取新实例的那次挂载里 R 的根槽与单元读回全零（`ur`），那次挂载落在 P、取号，崩在写行那次发布（Row）的根槽落盘之后、
+/// 它那次系统配置轮换之前；下一次读时 Row 写出的单元与 P 的末条记录（计数器 = c_见证 的那一条）读回全零。Row 走读失败、没见证，
+/// 见证到的那条记录读不出就不退——三处照报 Row 读不出，不退到那次挂载已按实例表抛弃了的 R（F2_EXACT 之前上限退成所选根、落到 R）。
+/// 判别力：那条记录读不出时上限取所选根自己（变异「F2 见证记录读不出时上限取所选根」）时三处都读回 R、这一条红。
+#[test]
+fn a_row_publish_whose_units_and_the_witnessed_last_record_read_back_zero_is_reported_unreadable_instead_of_falling_back_to_the_abandoned_root(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let ranges_at_the_mount = RangesReadingBackAllZero::default()
+        .with_the_writes(&[recorded.root_slot_write()])
+        .with_the_writes(&recorded.unit_writes());
+    let (mount_writes, row_root) = writes_of_the_mount_that_took_a_new_instance(
+        &recorded,
+        &ranges_at_the_mount,
+        ShadowLedger::On,
+    );
+    let mut image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    image.apply_writes(&mount_writes[..=row_root]);
+    let row = RootRecord::parse_slot(
+        mount_writes[row_root].bytes().expect("根槽写带字节"),
+        &recorded.pool.parameters.filesystem_identifier,
+    )
+    .expect("写行那次发布的根自证");
+    let row_key = (row.instance, row.checkpoint_txg);
+    assert!(
+        row_key.0 > recorded.key.0 && row_key.1 > recorded.key.1,
+        "写行那次发布是新实例的、txg 在 R 之后：{row_key:?}"
+    );
+    let row_unit_writes: Vec<&RetainedWrite> = mount_writes[..row_root]
+        .iter()
+        .filter(|write| write.kind == StepKind::UnitWrite)
+        .collect();
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&row_unit_writes)
+        .with_the_last_record_of_the_publish(&image, recorded.previous_key);
+    assert_recovery_and_both_mounts_report_the_chosen_version_unreadable(
+        &recorded, &image, &ranges, row_key,
+    );
 }
 
 /// 5.3 的 S2：R 已确认，R 的记录（末条，也是计数器 = c_见证 的那一条）与单元读回全零：两条都读不出，当见证过、不退——恢复照报读不出、
@@ -147,29 +287,11 @@
     let ranges = RangesReadingBackAllZero::default()
         .with_the_writes(&recorded.unit_writes())
         .with_the_writes(&recorded.record_writes_of_the_record(1));
-    let devices = devices_reading_back_all_zero_on(&confirmed, &ranges);
-    let report = recover(&devices, JournalPolicy::Consult);
-    assert!(
-        matches!(
-            report.outcome,
-            RecoveryOutcome::Failed { root: Some(root), .. } if root == recorded.key
-        ),
-        "恢复落到已确认的 R、照报走读失败，不悄悄读回 P：{:?}",
-        report.outcome
-    );
-    assert!(
-        mount_read_only(&devices).is_err(),
-        "只读挂载照报打不开 R，不退回 P"
-    );
-    let mut devices_of_the_mount = devices_reading_back_all_zero_on(&confirmed, &ranges);
-    assert!(
-        mount_writable(&recorded.pool.parameters, &mut devices_of_the_mount).is_err(),
-        "可写挂载照拒，不在 P 上取号"
-    );
-    assert!(
-        image_of_devices_reading_back_all_zero(&devices_of_the_mount, recorded.pool.device_bytes)
-            == confirmed,
-        "可写挂载在任何写之前拒：两块盘逐字节不变"
+    assert_recovery_and_both_mounts_report_the_chosen_version_unreadable(
+        &recorded,
+        &confirmed,
+        &ranges,
+        recorded.key,
     );
 }
 
@@ -177,7 +299,7 @@
 /// 计数器大于它，算没见证，恢复与只读挂载读回 P——用户 2026-10-07 定案接受的代价（F2 乙「三处都做」，交用户时写明：已确认的版本在较新系统配置槽与
 /// 它的单元同时读回全零时悄悄读回上一版）。这一条钉住这个结局；条款改了它跟着改。判别力：同 `ur` 那一条的变异（末条那一支恒见证过）时照报读不出、这一条红。
 #[test]
-fn user_accepted_cost_of_2026_10_07_a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_reads_back_the_previous_version(
+fn a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_reads_back_the_previous_version(
 ) {
     let recorded =
         record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
diff --git a/crates/singlefs-harness/tests/writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero.rs b/crates/singlefs-harness/tests/writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero.rs
new file mode 100644
--- /dev/null
+++ b/crates/singlefs-harness/tests/writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero.rs
@@ -0,0 +1,129 @@
+//! C 的账那一份（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节「K2 T1 C 的盲区」那一行与第三节第 1 条，攻方 `C_ACCOUNT`）：
+//! 可写挂载读阶段、重建分配器之后，水位之上每一次没施加的发布，用它任一条读得出的记录的新根段拼一条根、按影子账同一份读法读它的账，
+//! 账里的落点并进 C 的集合（账读不出就不加，交 F1 乙），解开时机照 C（`ring`）。
+//!
+//! 造法（`tests/common_unwitnessed_publish/mod.rs`，攻方报告第三节第 1 小节那一形）：384 槽小盘、第一个文件之后覆盖写 1 次，再录一次三条记录的发布 R
+//! （顺序写三个数据单元），崩在 R 的根槽落盘之后、系统配置轮换之前；取新实例的那次可写挂载 M1 里 R 的根槽与 R 的某一条记录（末条 `rL`、中间一条 `rM`、
+//! 第一条 `rF`）读回全零，R 的单元读得出。F1 乙不拒（读得出的那两条点名的单元验得过），缺的那条记录单独点名的单元不在别的记录里，
+//! R 的根槽读不出、影子账也不知道 R 是一条根；读得出的记录的新根段说得出 R 的树表在哪，R 的账（单元都好）读得出。
+//! M1 之后照常覆盖写一次、再正常卸载：R 的单元一份都没被写掉，卸载之后池级 checker 不红 I-7.4（被抛弃根引用的单元被重新分配）。
+mod common_unwitnessed_publish;
+
+use common_unwitnessed_publish::{
+    checker_violations_on, devices_reading_back_all_zero_on,
+    image_of_devices_reading_back_all_zero, plain_devices_on, pool_after_the_mount,
+    record_one_publish_after_overwrites, units_of_r_overwritten_on, PublishRecordedAsR,
+    RangesReadingBackAllZero, OVERWRITE_BYTES,
+};
+use singlefs_core::address::DeviceIdentity;
+use singlefs_core::mount::{mount_writable, unmount, ShadowLedger};
+use singlefs_harness::memory_pool::MemoryPool;
+
+/// 覆盖写几次之后录 R：攻方那一形量过 k = 1 与 4（两者逐格同数），取 1。
+const OVERWRITES_BEFORE_R: u64 = 1;
+
+/// M1 里 R 的根槽与 R 的第 `hidden_record` 条记录读回全零；M1 之后覆盖写一次、正常卸载，逐步核 R 的单元没被写掉、卸载之后 I-7.4 不红。
+/// 交回 M1 之后每块盘上分配器隔离的槽数（报告里写的「M1 隔离槽数」）。
+fn assert_the_units_of_the_publish_stay_unwritten_through_an_overwrite_and_the_unmount_when_record_reads_back_zero(
+    hidden_record: usize,
+) -> Vec<(DeviceIdentity, u64)> {
+    let recorded = record_one_publish_after_overwrites(
+        OVERWRITES_BEFORE_R,
+        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
+    );
+    assert_eq!(recorded.record_count(), 3, "顺序写三个数据单元切成三条记录");
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&[recorded.root_slot_write()])
+        .with_the_writes(&recorded.record_writes_of_the_record(hidden_record));
+    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
+    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
+        .expect("R 缺一条记录、读得出的记录点名的单元验得过：F1 乙不拒，可写挂载做得成");
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "R 的根槽读回全零、缺一条记录：R 整次不施加，所选那一版是 P"
+    );
+    let isolated_slots_after_the_mount: Vec<(DeviceIdentity, u64)> = mounted
+        .allocator
+        .devices
+        .iter()
+        .map(|device_map| (device_map.device, device_map.isolated_slots()))
+        .collect();
+    let image_after_the_mount =
+        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
+    let mut pool = pool_after_the_mount(&recorded, &image_after_the_mount, mounted);
+    pool.overwrite(OVERWRITE_BYTES)
+        .expect("384 槽小盘上 M1 之后覆盖写一次落得下");
+    assert_eq!(
+        units_of_r_overwritten_on(&recorded, &pool.image()),
+        Vec::<(DeviceIdentity, u64)>::new(),
+        "M1 之后第一次覆盖写没取到 R 的单元（C 隔离了 R 的账里的落点）"
+    );
+    let session = pool.session();
+    let mut devices_of_the_unmount = plain_devices_on(&pool.image());
+    let mut allocator_of_the_unmount = session.allocator.clone();
+    let mut current_of_the_unmount = session.current.clone();
+    unmount(
+        &recorded.pool.parameters,
+        &mut devices_of_the_unmount,
+        &mut allocator_of_the_unmount,
+        &mut current_of_the_unmount,
+        ShadowLedger::On,
+    )
+    .expect("M1 之后覆盖写一次的池正常卸载做得成");
+    let image_after_the_unmount = MemoryPool {
+        devices: devices_of_the_unmount
+            .iter()
+            .map(|(identity, device)| (*identity, device.image.clone()))
+            .collect(),
+        device_size_in_bytes: recorded.pool.device_bytes,
+    };
+    assert_eq!(
+        units_of_r_overwritten_on(&recorded, &image_after_the_unmount),
+        Vec::<(DeviceIdentity, u64)>::new(),
+        "卸载那一串空发布也没取到 R 的单元"
+    );
+    let reallocated_units_of_abandoned_roots: Vec<String> =
+        checker_violations_on(&image_after_the_unmount)
+            .into_iter()
+            .filter(|violation| violation.starts_with("I-7.4"))
+            .collect();
+    assert_eq!(
+        reallocated_units_of_abandoned_roots,
+        Vec::<String>::new(),
+        "卸载之后池级 checker 不红 I-7.4：被抛弃的 R 引用的单元没被重新分配"
+    );
+    isolated_slots_after_the_mount
+}
+
+/// `rL`：末条读回全零，末条单独点名的第三个数据单元与那几个元数据单元只在 R 的账里。
+/// 判别力：C 不并 R 的账（变异「C 不并没施加的发布的账（末条读回全零）」）时 M1 之后第一次覆盖写就写掉 R 的单元（攻方量到 2 份）、这一条红。
+#[test]
+fn a_publish_whose_root_slot_and_last_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount(
+) {
+    let isolated_slots =
+        assert_the_units_of_the_publish_stay_unwritten_through_an_overwrite_and_the_unmount_when_record_reads_back_zero(3);
+    println!("M1 隔离槽数（rL）：{isolated_slots:?}");
+}
+
+/// `rM`：中间一条读回全零，它单独点名的第二个数据单元只在 R 的账里。判别力：C 不并 R 的账（变异「C 不并没施加的发布的账（中间一条读回全零）」）时这一条红。
+#[test]
+fn a_publish_whose_root_slot_and_middle_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount(
+) {
+    let isolated_slots =
+        assert_the_units_of_the_publish_stay_unwritten_through_an_overwrite_and_the_unmount_when_record_reads_back_zero(2);
+    println!("M1 隔离槽数（rM）：{isolated_slots:?}");
+}
+
+/// `rF`：第一条读回全零，它单独点名的第一个数据单元只在 R 的账里。判别力：C 不并 R 的账（变异「C 不并没施加的发布的账（第一条读回全零）」）时这一条红。
+#[test]
+fn a_publish_whose_root_slot_and_first_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount(
+) {
+    let isolated_slots =
+        assert_the_units_of_the_publish_stay_unwritten_through_an_overwrite_and_the_unmount_when_record_reads_back_zero(1);
+    println!("M1 隔离槽数（rF）：{isolated_slots:?}");
+}
diff --git a/crates/singlefs-harness/tests/writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten.rs b/crates/singlefs-harness/tests/writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten.rs
--- a/crates/singlefs-harness/tests/writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten.rs
+++ b/crates/singlefs-harness/tests/writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten.rs
@@ -20,10 +20,11 @@
 use singlefs_core::journal::JournalRecord;
 use singlefs_core::mount::mount_writable;
 use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
+use singlefs_core::root_record::RootRecord;
 use singlefs_core::root_ring::target_for_publish;
 use singlefs_core::unit::unit_filesystem_identifier;
 use singlefs_format::SLOT_BYTES;
-use singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes;
+use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, RetainedWrite};
 use singlefs_harness::segments::StepKind;
 use singlefs_harness::SharedStream;
 
@@ -94,6 +95,79 @@
     );
 }
 
+/// 根环线 K4-1（判决 `research/prompts/m3-batch10y-code-r1-main-verification.md` 第二节，删除线判决 `research/prompts/m3-batch33t-code-r1-main-verification.md`
+/// 第二节「K1 / K2 T1 C 的候选豁免」那一行）：取新实例的那次可写挂载里只有 R 的中央映射树根那个单元读回全零（`common_unwitnessed_publish` 那块盘
+/// 每次只让 R 的一个单元读回全零，根环线正推的造法），R 的根槽与账都读得出、走读失败，所选那一版退到 P——R 按实例表有效、txg ≥ F，是回退候选根。
+/// 写行那次发布的根落盘之前断电：那次挂载写下的一切都没碰 R 的单元，恢复择到 R、读回 R 写的内容。
+/// 判别力：C 那一圈照旧豁免候选根引用的槽（变异「C 豁免候选根引用的槽」）时 R 的单元正是候选根 R 自己引用的，一个都不隔离，
+/// 那次挂载在根落盘之前写掉它们（根环线正推量到 14 份），两条断言都红。
+#[test]
+fn a_power_loss_before_the_row_root_of_the_mount_still_reads_back_a_candidate_root_whose_mapping_root_read_back_zero_at_that_mount(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let root_of_r = RootRecord::parse_slot(
+        recorded.root_slot_write().bytes().expect("根槽写带字节"),
+        &recorded.pool.parameters.filesystem_identifier,
+    )
+    .expect("R 的根自证");
+    let mapping_root_writes_of_r: Vec<&RetainedWrite> = recorded
+        .unit_writes()
+        .into_iter()
+        .filter(|write| {
+            root_of_r.mapping_root.locations.iter().any(|location| {
+                location.device == write.device && location.slot.to_device_offset() == write.offset
+            })
+        })
+        .collect();
+    assert_eq!(
+        mapping_root_writes_of_r.len(),
+        2,
+        "R 写了自己的中央映射树根，两块盘各一份"
+    );
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let stream = SharedStream::retaining_contents();
+    let mut devices = recorded_devices_reading_back_all_zero_on(
+        &crash_image,
+        &RangesReadingBackAllZero::default().with_the_writes(&mapping_root_writes_of_r),
+        &stream,
+    );
+    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
+        .expect("R 的映射根读回全零、系统配置没见证 R：可写挂载退到 P、做得成");
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "R 的映射根读回全零、走读失败，所选那一版退到见证到的 P"
+    );
+    let (mount_writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
+        &stream.retained_operations(),
+        &fixed_geometry_of_the_384_slot_small_devices(),
+    );
+    let row_root = mount_writes
+        .iter()
+        .position(|write| write.kind == StepKind::RootRecordFua)
+        .expect("那次挂载写行那次发布有根槽写");
+    let mut image_before_the_row_root = crash_image.clone();
+    image_before_the_row_root.apply_writes(&mount_writes[..row_root]);
+    assert_eq!(
+        units_of_r_overwritten_on(&recorded, &image_before_the_row_root),
+        Vec::<(DeviceIdentity, u64)>::new(),
+        "写行那次发布的根落盘之前，那次挂载写下的一切都没碰 R 的单元（C 不豁免候选根 R 自己引用的槽）"
+    );
+    let report = recover(&image_before_the_row_root, JournalPolicy::Consult);
+    assert_eq!(
+        report.outcome,
+        RecoveryOutcome::FileRead {
+            root: recorded.key,
+            content: recorded.content.clone(),
+        },
+        "写行那次的根没落：恢复择到 R、读回 R 写的内容"
+    );
+}
+
 /// 第九项（D28（挂载期承诺量） 已定项 1）随 C 扩成分配器实际隔离的集合（判决第三节第 3 条）：M1 之后每块盘上分配器隔离的槽数，
 /// 就是 R 写出的单元在那块盘上占的槽数（R 的根槽那一次读回全零，环里没有读得出的被抛弃根，影子账那一部分是 0，隔离的全是 C 的），
 /// 准入读数的第九项逐盘等于它。判别力：C 不隔离时两样都是 0，「隔离数等于 R 的单元槽数」那一条红。
diff --git a/crates/singlefs-harness/tests/writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero.rs b/crates/singlefs-harness/tests/writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero.rs
--- a/crates/singlefs-harness/tests/writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero.rs
+++ b/crates/singlefs-harness/tests/writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero.rs
@@ -14,10 +14,12 @@
     record_one_publish_after_overwrites, PublishRecordedAsR, RangesReadingBackAllZero,
     RecordedPublish,
 };
+use singlefs_core::journal::JournalRecord;
 use singlefs_core::mount::{
     mount_writable, MountError, RollbackTarget, StillUnreadableAfterOneReread,
 };
-use singlefs_harness::memory_pool::RetainedWrite;
+use singlefs_core::unit::unit_filesystem_identifier;
+use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite};
 use singlefs_harness::segments::StepKind;
 
 const OVERWRITES_BEFORE_R: u64 = 5;
@@ -122,3 +124,107 @@
         "R 的末条没落：R 整次不施加，所选那一版是 P"
     );
 }
+
+/// 写记录中途断电、只落了第一条记录（两份）：R 的单元都落了（单元在记录之前落盘），第二、三条记录、根槽与轮换都没落。
+fn crash_image_with_only_the_first_record_of_the_publish(recorded: &RecordedPublish) -> MemoryPool {
+    let first_record = recorded
+        .writes
+        .iter()
+        .position(|write| write.kind == StepKind::JournalRecord)
+        .expect("R 写了记录");
+    let mut crash_image = recorded.base.clone();
+    crash_image.apply_writes(&recorded.writes[..first_record]);
+    let first_record_writes: Vec<RetainedWrite> = recorded
+        .record_writes_of_the_record(1)
+        .into_iter()
+        .cloned()
+        .collect();
+    crash_image.apply_writes(&first_record_writes);
+    crash_image
+}
+
+/// R 第一条记录点名的第一个单元的两份写（两块盘各一份，按盘序）。
+fn unit_writes_of_the_first_unit_named_by_the_first_record(
+    recorded: &RecordedPublish,
+) -> Vec<&RetainedWrite> {
+    let first_record = JournalRecord::parse(
+        recorded.record_writes_of_the_record(1)[0]
+            .bytes()
+            .expect("记录写带字节"),
+        unit_filesystem_identifier(&recorded.pool.parameters.filesystem_identifier),
+    )
+    .expect("R 的第一条记录解得开");
+    let named = first_record
+        .named
+        .first()
+        .expect("R 的第一条记录点名了单元");
+    named
+        .locations
+        .iter()
+        .map(|location| {
+            recorded
+                .unit_writes()
+                .into_iter()
+                .find(|write| {
+                    write.device == location.device
+                        && write.offset == location.slot.to_device_offset()
+                })
+                .expect("点名的单元是 R 写出的")
+        })
+        .collect()
+}
+
+/// 代码轮攻方 T4 那一形（`research/prompts/m3-batch33t-code-r1-opus-output.md` 第二节第 2 小节）：写记录中途断电、根从没落，第一条记录点名的一个单元
+/// 只有一份（盘 0 那一份）读回全零——单故障，另一份验得过。F1 判「点名单元验不过」要某个点名单元两份都验不过（`F1_ANY`），这里不拒，可写挂载做成、
+/// 所选那一版是 P。判别力：退回两份都要验过（变异「F1 点名单元一份坏就算验不过」）时每次可写挂载都拒、没有出口，这一条红。
+#[test]
+fn a_power_loss_while_writing_the_records_with_one_copy_of_a_named_unit_reading_back_zero_still_mounts_writable(
+) {
+    let recorded = recorded_publish_of_three_records();
+    let crash_image = crash_image_with_only_the_first_record_of_the_publish(&recorded);
+    let both_copies = unit_writes_of_the_first_unit_named_by_the_first_record(&recorded);
+    let ranges = RangesReadingBackAllZero::default().with_the_writes(&both_copies[..1]);
+    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
+    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
+        .expect("点名的单元只坏一份、另一份验得过：F1 不拒，可写挂载做成");
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "R 只落了第一条记录：R 整次不施加，所选那一版是 P"
+    );
+}
+
+/// 同上一形，那个点名单元两份都读回全零：缺记录、点名的单元两份都验不过，重读一次仍这样，在取号之前拒可写，盘上逐字节不变。
+/// 判别力：`F1_ANY` 那一份判法恒交「验得过」（变异「F1 点名单元恒验得过」）时两份都坏也不拒，这一条红。
+#[test]
+fn a_power_loss_while_writing_the_records_with_both_copies_of_a_named_unit_reading_back_zero_refuses_the_writable_mount_before_any_write(
+) {
+    let recorded = recorded_publish_of_three_records();
+    let crash_image = crash_image_with_only_the_first_record_of_the_publish(&recorded);
+    let both_copies = unit_writes_of_the_first_unit_named_by_the_first_record(&recorded);
+    let ranges = RangesReadingBackAllZero::default().with_the_writes(&both_copies);
+    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
+    let refusal = mount_writable(&recorded.pool.parameters, &mut devices)
+        .expect_err("缺记录、点名的单元两份都验不过：重读一次仍这样，拒可写");
+    assert!(
+        matches!(
+            &refusal,
+            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
+                if matches!(
+                    still_unreadable.as_ref(),
+                    StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify {
+                        publish: RollbackTarget { instance, checkpoint_txg },
+                    } if (*instance, *checkpoint_txg) == recorded.key
+                )
+        ),
+        "拒因是 F1 那一个成员、点名 R {:?}：{refusal:?}",
+        recorded.key
+    );
+    assert!(
+        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes) == crash_image,
+        "在取号之前拒：两块盘逐字节不变"
+    );
+}
````

### m3-batch35/w：mutations-append.tsv

````
删除线第35批 K4-1 C 那一圈照旧豁免候选根引用的槽（R 的映射根读回全零、R 是候选根，那次挂载根落盘之前写掉 R 的单元）	crates/singlefs-core/src/mount.rs	        if referenced_by_the_current_account.contains(&key) || !isolated.insert(key) {	        if referenced_by_candidates.contains(&key) || !isolated.insert(key) {	-p singlefs-harness --test writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten -- a_power_loss_before_the_row_root_of_the_mount_still_reads_back_a_candidate_root_whose_mapping_root_read_back_zero_at_that_mount	a_power_loss_before_the_row_root_of_the_mount_still_reads_back_a_candidate_root_whose_mapping_root_read_back_zero_at_that_mount
删除线第35批 C_ACCOUNT C 不并没施加的发布的账（根槽与末条读回全零）	crates/singlefs-core/src/mount.rs	                named_by_ring_slot\n                    .entry(ring_slot)\n                    .or_default()\n                    .extend(placements_in_the_account);	                let _ = (ring_slot, placements_in_the_account);	-p singlefs-harness --test writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero -- a_publish_whose_root_slot_and_last_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount	a_publish_whose_root_slot_and_last_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount
删除线第35批 C_ACCOUNT C 不并没施加的发布的账（根槽与中间一条读回全零）	crates/singlefs-core/src/mount.rs	                named_by_ring_slot\n                    .entry(ring_slot)\n                    .or_default()\n                    .extend(placements_in_the_account);	                let _ = (ring_slot, placements_in_the_account);	-p singlefs-harness --test writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero -- a_publish_whose_root_slot_and_middle_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount	a_publish_whose_root_slot_and_middle_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount
删除线第35批 C_ACCOUNT C 不并没施加的发布的账（根槽与第一条读回全零）	crates/singlefs-core/src/mount.rs	                named_by_ring_slot\n                    .entry(ring_slot)\n                    .or_default()\n                    .extend(placements_in_the_account);	                let _ = (ring_slot, placements_in_the_account);	-p singlefs-harness --test writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero -- a_publish_whose_root_slot_and_first_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount	a_publish_whose_root_slot_and_first_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount
删除线第35批 F1_ANY F1 点名单元一份坏就算验不过（写记录中途断电、一份副本坏，每次可写挂载都拒）	crates/singlefs-core/src/recovery.rs	            .any(|location| copy_named_by_the_location_entry_verifies(reader, location, unit_bytes))	            .all(|location| copy_named_by_the_location_entry_verifies(reader, location, unit_bytes))	-p singlefs-harness --test writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero -- a_power_loss_while_writing_the_records_with_one_copy_of_a_named_unit_reading_back_zero_still_mounts_writable	a_power_loss_while_writing_the_records_with_one_copy_of_a_named_unit_reading_back_zero_still_mounts_writable
删除线第35批 F1_ANY F1 点名单元恒验得过（两份都坏也不拒）	crates/singlefs-core/src/mount.rs	                !crate::recovery::every_named_unit_of_the_record_has_a_copy_that_verifies(	                false && !crate::recovery::every_named_unit_of_the_record_has_a_copy_that_verifies(	-p singlefs-harness --test writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero -- a_power_loss_while_writing_the_records_with_both_copies_of_a_named_unit_reading_back_zero_refuses_the_writable_mount_before_any_write	a_power_loss_while_writing_the_records_with_both_copies_of_a_named_unit_reading_back_zero_refuses_the_writable_mount_before_any_write
删除线第35批 F2_EXACT F2 退到见证上限之下任一条可读根（A 形：见证到的那一版根槽读回全零时退过它）	crates/singlefs-core/src/recovery.rs	        order == witnessed_publish && order < chosen	        order <= witnessed_publish && order < chosen	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- a_root_whose_units_and_the_witnessed_versions_root_slot_read_back_zero_is_reported_unreadable_in_recovery_and_both_mounts	a_root_whose_units_and_the_witnessed_versions_root_slot_read_back_zero_is_reported_unreadable_in_recovery_and_both_mounts
删除线第35批 F2_EXACT 退回 F2_EXACT 之前的上限取法（B 形：见证记录读不出时上限取所选根、退到被抛弃的根）	crates/singlefs-core/src/recovery.rs	    let witnessed_publish = record_at_the_witnessed_counter?;\n    let chosen = order_of_the_root_choice(chosen_root);\n    readable_roots(\n        reader,\n        &system_configuration.immutable.region_devices,\n        &system_configuration.immutable.sizes,\n        &system_configuration.immutable.filesystem_identifier,\n    )\n    .into_iter()\n    .find(|root| {\n        let order = order_of_the_root_choice(root);\n        order == witnessed_publish && order < chosen\n    })	    let chosen = order_of_the_root_choice(chosen_root);\n    let ceiling = record_at_the_witnessed_counter.unwrap_or(chosen);\n    readable_roots(\n        reader,\n        &system_configuration.immutable.region_devices,\n        &system_configuration.immutable.sizes,\n        &system_configuration.immutable.filesystem_identifier,\n    )\n    .into_iter()\n    .filter(|root| {\n        let order = order_of_the_root_choice(root);\n        order < chosen && order <= ceiling\n    })\n    .max_by_key(order_of_the_root_choice)	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- a_row_publish_whose_units_and_the_witnessed_last_record_read_back_zero_is_reported_unreadable_instead_of_falling_back_to_the_abandoned_root	a_row_publish_whose_units_and_the_witnessed_last_record_read_back_zero_is_reported_unreadable_instead_of_falling_back_to_the_abandoned_root
删除线第35批 F2 乙末条读得出那一支恒见证过（设计里那一退不退）	crates/singlefs-core/src/recovery.rs	        (Some(last_record_counter), _) => last_record_counter <= witnessed_journal_counter,	        (Some(last_record_counter), _) => last_record_counter <= witnessed_journal_counter || true,	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- a_root_whose_units_read_back_zero_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts	a_root_whose_units_read_back_zero_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts
删除线第35批 读字段再算一遍时不照 F2 退（S1、比恢复落到的那一版，断言 panic）	crates/singlefs-harness/src/mount_transparency.rs	    let recovered_root = if walks_to_a_file_or_no_file(&replayed_root) {	    let recovered_root = if true || walks_to_a_file_or_no_file(&replayed_root) {	-p singlefs-harness --test read_recovery_fields_follow_the_fallback_to_the_witnessed_version -- reading_the_recovered_version_of_a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_compares_the_witnessed_version	reading_the_recovered_version_of_a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_compares_the_witnessed_version
删除线第35批 读字段再算一遍时不照 F2 退（S1、比恢复落到的那一版之前那一版，断言 panic）	crates/singlefs-harness/src/mount_transparency.rs	    let recovered_root = if walks_to_a_file_or_no_file(&replayed_root) {	    let recovered_root = if true || walks_to_a_file_or_no_file(&replayed_root) {	-p singlefs-harness --test read_recovery_fields_follow_the_fallback_to_the_witnessed_version -- reading_the_version_before_the_recovered_one_of_a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_compares_the_version_before_the_witnessed_one	reading_the_version_before_the_recovered_one_of_a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_compares_the_version_before_the_witnessed_one
删除线第35批 读字段再算一遍时不照 F2 退（设计里那一退、比恢复落到的那一版，断言 panic）	crates/singlefs-harness/src/mount_transparency.rs	    let recovered_root = if walks_to_a_file_or_no_file(&replayed_root) {	    let recovered_root = if true || walks_to_a_file_or_no_file(&replayed_root) {	-p singlefs-harness --test read_recovery_fields_follow_the_fallback_to_the_witnessed_version -- reading_the_recovered_version_of_an_unwitnessed_root_whose_units_read_back_zero_compares_the_witnessed_version	reading_the_recovered_version_of_an_unwitnessed_root_whose_units_read_back_zero_compares_the_witnessed_version
删除线第35批 读字段再算一遍时不照 F2 退（设计里那一退、比恢复落到的那一版之前那一版，断言 panic）	crates/singlefs-harness/src/mount_transparency.rs	    let recovered_root = if walks_to_a_file_or_no_file(&replayed_root) {	    let recovered_root = if true || walks_to_a_file_or_no_file(&replayed_root) {	-p singlefs-harness --test read_recovery_fields_follow_the_fallback_to_the_witnessed_version -- reading_the_version_before_the_recovered_one_of_an_unwitnessed_root_whose_units_read_back_zero_compares_the_version_before_the_witnessed_one	reading_the_version_before_the_recovered_one_of_an_unwitnessed_root_whose_units_read_back_zero_compares_the_version_before_the_witnessed_one
删除线第35批 排演不判盘短于单元区起点（建空闲图那一步 panic）	crates/singlefs-core/src/recovery.rs	        unit_area_slots_of_device_starting_at(*device_bytes, unit_area_start).is_err()	        unit_area_slots_of_device_starting_at(*device_bytes, unit_area_start).is_err() && false	-p singlefs-core --lib -- allocation_records_per_device_tests::rehearsal_on_a_device_shorter_than_the_unit_area_start_returns_that_device_instead_of_panicking	rehearsal_on_a_device_shorter_than_the_unit_area_start_returns_that_device_instead_of_panicking
删除线第35批 排演先于判记录（记录是空的、盘短时报盘短，不报记录）	crates/singlefs-core/src/recovery.rs	    if !allocation_records_are_one_per_device(records, device_identities, PlacementsOnEachDevice(0))\n    {\n        return Err(not_one_placement_per_device);\n    }\n		-p singlefs-core --lib -- allocation_records_per_device_tests::walk_judging_allocation_records_on_a_device_shorter_than_the_unit_area_start_returns_an_error_instead_of_panicking	walk_judging_allocation_records_on_a_device_shorter_than_the_unit_area_start_returns_an_error_instead_of_panicking
````

### m3-batch35/w：mutations-replacements.tsv

````
增补 2 第 61 行（C519 已知丢失）：施加前的点名单元验证改成任一条位置条目验过即过（改法 Z2 被悄悄落地时已知丢失那条先红）	crates/singlefs-core/src/recovery.rs	            .all(|location| copy_named_by_the_location_entry_verifies(reader, location, unit_bytes))	            .any(|location| copy_named_by_the_location_entry_verifies(reader, location, unit_bytes))	-p singlefs-harness --test whole_device_loss_after_warm_up -- c519_known_loss	c519_known_loss_after_the_warm_up_covers_both_devices_losing_the_acknowledged_version_root_device_falls_back_to_the_row_publish_version
第33批T C692 G2 下限退成默认几何的常数 14（1024 MiB 环那套几何每盘少两条判不出）	crates/singlefs-core/src/recovery.rs	                rehearsal.records().len() / device_count,	                14,	-p singlefs-core --lib -- allocation_records_per_device_tests	the_one_gibibyte_journal_ring_geometry_has_sixteen_placements_on_each_device_after_the_new_pool_file_creation
第33批T C692 G2 下限少算一个落点（默认几何每盘少一条判不出）	crates/singlefs-core/src/recovery.rs	                rehearsal.records().len() / device_count,	                rehearsal.records().len() / device_count - 1,	-p singlefs-core --lib -- allocation_records_per_device_tests	the_default_geometry_has_fourteen_placements_on_each_device_after_the_new_pool_file_creation
第33批T F2 乙末条读得出那一支恒见证过（S1 那一形不退）	crates/singlefs-core/src/recovery.rs	        (Some(last_record_counter), _) => last_record_counter <= witnessed_journal_counter,	        (Some(last_record_counter), _) => last_record_counter <= witnessed_journal_counter || true,	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_reads_back_the_previous_version	a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_reads_back_the_previous_version
````

### m3-batch35/w：mutations-delete.txt

````
第33批T F2 乙两条都读不出就退（攻方 loose）
第33批T F2 乙施加过记录、由记录重建的那一版走读失败也退（条款只写了所选根）
````

## m3-batch36/v：crates.patch（/home/fy5090/.local/share/singlefs/work/m3-batch36/v/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -464,8 +464,8 @@
     /// 走过的实例表各片（按读回那一份的落点）→ 头里的诞生代号：I-3.5 要它。不进 `birth_txg_of_walked_units`——
     /// 那张表 I-4.9 也读，而实例表单元按 I-1.2 那一行的例外不进 I-4.9。
     birth_txg_of_walked_instance_table_pages: BTreeMap<(u32, u64), u64>,
-    /// 已经走完的每一版（根环里的候选根与由记录施加出来的那几版）：I-3.5 在全部走完之后逐对比。
-    versions_walked_for_the_appearance_rule: Vec<VersionReferencesForTheAppearanceRule>,
+    /// 已经走完的每一版（根环里的候选根与由记录施加出来的那几版）：I-3.5 在全部走完之后按那一刻的折叠表折成单元、逐对比。
+    versions_walked_for_the_appearance_rule: Vec<VersionPlacementsForTheAppearanceRule>,
 }
 
 /// 用户可见树的种类码：回退取回退目标那一版的几棵（extent、inode、`deleted_inodes`，`crates/singlefs-core/src/mount.rs`
@@ -519,11 +519,24 @@
     })
 }
 
+/// 走完的一版留给 I-3.5 的东西：它是哪一版、引用过哪些落点（还没折成单元）、树表里几棵用户可见树的根、走读断没断。
+/// 落点到全部版本都走完才折：折叠表（`Walk::unit_of_each_placement`）在后走的版本读回第 1 条那一份时还会改
+/// （[`Walk::read_a_unit_to_walk_and_the_placement_it_was_read_from`]），每版走完当场折的话，先走那一版存下的单元跟不上这一改，
+/// I-3.5 随根环次序翻面（判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S1 那一行与第三节第 1 条，攻方 F1）。
+/// GPU 走读内核同一个取法（每版只存落点表的下标，判时按落点表那一行此刻的单元认）。
+struct VersionPlacementsForTheAppearanceRule {
+    instance: u32,
+    checkpoint_txg: u64,
+    placements: Vec<(u32, u64)>,
+    user_visible_tree_roots: Vec<UserVisibleTreeRootOfAVersion>,
+    references_are_complete: bool,
+}
+
 /// 走完的一版在 I-3.5 眼里的样子：它是哪一版、引用了哪些单元（各落在哪棵用户可见树里）、树表里几棵用户可见树的根、走读断没断。
 struct VersionReferencesForTheAppearanceRule {
     instance: u32,
     checkpoint_txg: u64,
-    /// 这一版引用的单元（落点折成第 0 条位置条目的落点）→ 它落在哪棵用户可见树里（不在任何一棵里是 `None`）。
+    /// 这一版引用的单元（落点按全部版本走完之后的折叠表折成单元）→ 它落在哪棵用户可见树里（不在任何一棵里是 `None`）。
     units: BTreeMap<(u32, u64), Option<UserVisibleTreeOfTheWalk>>,
     user_visible_tree_roots: Vec<UserVisibleTreeRootOfAVersion>,
     /// 这一版走读一条走读失败都没记：记了就是引用集合数不全，它参与的每一对报不适用。
@@ -1219,37 +1232,57 @@
         self.user_visible_tree_roots_of_this_version.clear();
     }
 
-    /// 一版走完：把它引用的单元（落点折成单元，带用户可见树的标签）、树表里几棵用户可见树的根与走读断没断记下，留给 I-3.5 逐对比。
-    /// `walk_failures_before` 是走这一版之前走读失败记了几条。
+    /// 一版走完：把它引用过的落点、树表里几棵用户可见树的根与走读断没断记下，留给 I-3.5 在全部版本走完之后折成单元、逐对比
+    /// （[`VersionPlacementsForTheAppearanceRule`]）。`walk_failures_before` 是走这一版之前走读失败记了几条。
     fn note_the_version_just_walked_for_the_appearance_rule(
         &mut self,
         instance: u32,
         checkpoint_txg: u64,
         walk_failures_before: usize,
     ) {
-        let units = self
-            .placements_referenced_in_this_version
-            .keys()
-            .map(|placement| {
-                let unit = *self.unit_of_each_placement.get(placement).expect(
-                    "这一版引用过的每个落点都经 `note_references_of_a_pointer_followed_by_the_walk` 记过（这一版或先走的那一版）",
-                );
-                (
-                    unit,
-                    self.user_visible_tree_of_each_placement.get(placement).copied(),
-                )
-            })
-            .collect();
         self.versions_walked_for_the_appearance_rule
-            .push(VersionReferencesForTheAppearanceRule {
+            .push(VersionPlacementsForTheAppearanceRule {
                 instance,
                 checkpoint_txg,
-                units,
+                placements: self
+                    .placements_referenced_in_this_version
+                    .keys()
+                    .copied()
+                    .collect(),
                 user_visible_tree_roots: self.user_visible_tree_roots_of_this_version.clone(),
                 references_are_complete: self.walk_failures.len() == walk_failures_before,
             });
     }
 
+    /// 全部版本走完之后，每一版引用过的落点按此刻的折叠表折成单元（带用户可见树的标签）：I-3.5 逐对比的输入
+    /// （[`VersionPlacementsForTheAppearanceRule`] 说为什么不在每版走完时折）。同一个单元由几个落点折来时，标签取落点序最大的那个落点的。
+    fn versions_folded_into_units_by_the_final_fold_table(
+        &self,
+    ) -> Vec<VersionReferencesForTheAppearanceRule> {
+        self.versions_walked_for_the_appearance_rule
+            .iter()
+            .map(|version| VersionReferencesForTheAppearanceRule {
+                instance: version.instance,
+                checkpoint_txg: version.checkpoint_txg,
+                units: version
+                    .placements
+                    .iter()
+                    .map(|placement| {
+                        let unit = *self.unit_of_each_placement.get(placement).expect(
+                            "这一版引用过的每个落点都经 `note_references_of_a_pointer_followed_by_the_walk` 记过（这一版或先走的那一版），折叠表只改不删",
+                        );
+                        (
+                            unit,
+                            self.user_visible_tree_of_each_placement.get(placement).copied(),
+                        )
+                    })
+                    .collect(),
+                user_visible_tree_roots: version.user_visible_tree_roots.clone(),
+                references_are_complete: version.references_are_complete,
+            })
+            .collect()
+    }
+
     /// 往下走 `unit` 里的指针之前调：之后跟着指针记下的落点都算在 `unit` 下面（`placements_referenced_below_a_walked_unit`）。
     /// 交回之前那一个，走完用 [`Self::stop_walking_below`] 换回去；两次调用之间的循环不许提前返回。
     fn start_walking_below(&mut self, unit: (u32, u64)) -> Option<(u32, u64)> {
@@ -11378,7 +11411,7 @@
             });
     }
     judge_the_appearance_rule_of_adjacent_candidate_versions(
-        &walk.versions_walked_for_the_appearance_rule,
+        &walk.versions_folded_into_units_by_the_final_fold_table(),
         &walk.birth_txg_of_walked_units,
         &walk.birth_txg_of_walked_instance_table_pages,
         &walk.instance_table_rows,
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -3145,7 +3145,9 @@
 // 头按最短的单元（码 2，16 KiB）那么长读，头里的类标签是登记过的码就只按那一类的长度比；没登记的码、或头读不出，16 KiB 与 32 KiB 两种长度都对不上
 // 才算对不上（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行的改法 F-α）。比 32 KiB 时另按 32 KiB 读一次
 // （CPU 那一边按每种长度各读一次）：没写过的槽 read_at 按 16 KiB 交回的全零单元只按它自己那一槽的长度用，下一槽有落点时不拿它的 32 KiB 检查字去比
-// （判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第二节 K1 / K2 P4 F-零槽那一行、第三节第 2 条，攻方改法 Z3）
+// （判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第二节 K1 / K2 P4 F-零槽那一行、第三节第 2 条，攻方改法 Z3）。
+// 空段最后一槽那一读是拼读（read_at）：这里只比整段 CRC、不往下读字节，拼出的 CRC 照比（slice_whole_read_crc_matches，不记判不了；
+// 判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第三节第 1 条，正推说的「空槽加下一已写槽前 16 KiB」那一形）
 fn w_copy_matches_its_location_checksum_at_the_length_its_header_names(device: u32, slot: vec2<u32>, checksum: u32) -> bool {
     let copy = read_at(device, slot_to_bytes(slot), NODE_BYTES);
     if (copy.version == NONE) {
@@ -3156,7 +3158,7 @@
         return w_unit_verifies_at_the_length_of_its_class(copy, class_in_the_header, checksum);
     }
     return slice_crc_matches(copy, NODE_BYTES, checksum)
-        || slice_crc_matches(read_at(device, slot_to_bytes(slot), DATA_UNIT_BYTES), DATA_UNIT_BYTES, checksum);
+        || slice_whole_read_crc_matches(read_at(device, slot_to_bytes(slot), DATA_UNIT_BYTES), checksum);
 }
 
 // 走读不往下读的孩子（墓碑树、谱系树、inode 树内部条目里类型段不认得的，与孩子自证成类型段之外另一类的）：每一份副本照判 I-2.1，
diff --git a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
--- a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
@@ -40,7 +40,16 @@
 
 const WRITE_WORDS: u32 = 12u;
 const LOCATION_WORDS: u32 = 12u;
-const VERSION_WORDS: u32 = 8u;
+const VERSION_WORDS: u32 = 9u;
+// 版本表第 8 字（crash_judge_tables::VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES）：单元槽的一版，「前 16 KiB 后面接 16 KiB 全零」那 32 KiB 的
+// CRC-32C 异或 16 KiB 全零的 CRC-32C。它再异或一段 16 KiB 的 CRC-32C，就是「这一版前 16 KiB 接那一段」32 KiB 的 CRC-32C（CRC 对异或线性）
+const VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES: u32 = 8u;
+// 版本表第 1 条全零单元（16 KiB，crash_judge_tables::ZERO_UNIT_VERSIONS）：拼读前一半是空槽时读它
+const ZERO_UNIT_VERSION_OF_SIXTEEN_KIBIBYTES: u32 = 1u;
+// zero_unit_version_over_a_slot_nothing_wrote 交的码（不是版本下标）：起点是空段的最后一槽、读两槽，后一半要 read_at 拼
+const ZERO_SIXTEEN_KIBIBYTES_FOLLOWED_BY_THE_NEXT_SLOT: u32 = 0xFFFFFFFEu;
+// 切片第 2 个检查字（c2）的位 5（版本表的标志位只用到位 4）：这一读跨过了落点的末尾（read_at 拼读），c1 是拼出来的整段 CRC
+const SLICE_SPLICED_ACROSS_THE_END_OF_A_LOCATION: u32 = 32u;
 const LATER_WORDS: u32 = 4u;
 const PUBLISH_WORDS: u32 = 8u;
 const EXPECTED_WORDS: u32 = 6u;
@@ -81,6 +90,8 @@
 const UNDECIDABLE_TOO_DEEP: u32 = 4u;
 const UNDECIDABLE_TOO_MANY_DEVICES: u32 = 5u;
 const UNDECIDABLE_TOO_MANY_ROOTS: u32 = 6u;
+// read_at 拼读那一读的整段 CRC 对得上位置条目（slice_crc_matches）：往下要读的字节跨过落点末尾，表里没有
+const UNDECIDABLE_SPLICED_READ_MATCHES: u32 = 7u;
 
 // 格式常量
 const SLOT_SHIFT: u32 = 14u;                 // 16384 = 1 << 14
@@ -446,7 +457,13 @@
     return s;
 }
 // 读 (盘, 偏移, 长度)：起点是落点起点、长度不超过落点的才读得到；起点不是落点的，单元区里没写过的槽读出全零
-// （zero_unit_version_over_a_slot_nothing_wrote）。切片的字节与检查字照版本表那一条（slice_of_version 同一个读法），长度按落点或全零单元
+// （zero_unit_version_over_a_slot_nothing_wrote）。切片的字节与检查字照版本表那一条（slice_of_version 同一个读法），长度按落点或全零单元。
+// 空段的最后一槽按 32 KiB 读（拼读，判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S2 那一行与第三节第 1 条，第 32 批攻方 Z2；
+// 正推说的「空槽加下一已写槽前 16 KiB」同一形）：CPU 的稀疏读者读出「16 KiB 全零 + 紧接着的下一个落点的前 16 KiB」。GPU 拿全零 16 KiB 那一条的第 8 字
+// （VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES）异或下一个落点那一版的前 16 KiB 的 CRC，拼出整段的 CRC 放进 c1，c2 只留「有 32 KiB」与拼读两位
+// （SLICE_SPLICED_ACROSS_THE_END_OF_A_LOCATION）：字节只有前 16 KiB 是这一读的，按位置条目比 CRC 对得上时交 CPU 判（slice_crc_matches）。
+// 下一个落点查不到版本（undecidable 已记）、空段止于盘尾（CPU 越过盘尾读不到）的交 no_slice。「16 KiB 节点加下一槽」不用拼：单元槽落点本来就是
+// 32 KiB 长（盘尾那一槽 16 KiB，CPU 读 32 KiB 越过盘尾），版本字节里后 16 KiB 就是叠上去的下一槽
 fn read_at(device: u32, offset: vec2<u32>, length: u32) -> Slice {
     // 落点表按 (盘, 偏移) 升序二分，lo 落在排在 (device, offset) 之后的第一个落点上；lo − 1 是最后一个不排在它之后的落点，
     // 也就是最后一次走「lo = mid + 1」的那个 mid：它在不在这块盘上、起点是不是恰为 offset，在那一次记下
@@ -472,16 +489,33 @@
     let location = lo - 1u;
     var version = NONE;
     var unit_length = 0u;
+    // 要查版本的落点：读的那个落点，或拼读时紧接着的下一个落点（lo）——一个调用点，内核展开不多一份查版本
+    var location_to_look_up = NONE;
+    var spliced = false;
     if (before_starts_at_the_offset) {
         unit_length = location_word(location, 3u);
         if (length <= unit_length) {
-            version = version_of_location(location);
+            location_to_look_up = location;
         }
     } else {
         version = zero_unit_version_over_a_slot_nothing_wrote(location, offset, length);
         unit_length = DATA_UNIT_BYTES >> (version & 1u);
+        // 拼读的后一半是紧接着的下一个落点 lo（zero_unit_version_over_a_slot_nothing_wrote 交这个码时空段不止于盘尾，止于 lo 的起点）
+        spliced = version == ZERO_SIXTEEN_KIBIBYTES_FOLLOWED_BY_THE_NEXT_SLOT;
+        if (spliced) {
+            location_to_look_up = lo;
+        }
+    }
+    var version_looked_up = NONE;
+    if (location_to_look_up != NONE) {
+        version_looked_up = version_of_location(location_to_look_up);
     }
-    if (version == NONE || length > unit_length) {
+    if (before_starts_at_the_offset) {
+        version = version_looked_up;
+    } else if (spliced) {
+        version = select(ZERO_UNIT_VERSION_OF_SIXTEEN_KIBIBYTES, NONE, version_looked_up == NONE);
+    }
+    if (version == NONE || version == ZERO_SIXTEEN_KIBIBYTES_FOLLOWED_BY_THE_NEXT_SLOT || (!spliced && length > unit_length)) {
         return no_slice();
     }
     var s: Slice;
@@ -492,13 +526,23 @@
     s.c1 = tab(T_VERSIONS, version * VERSION_WORDS + 3u);
     s.c2 = tab(T_VERSIONS, version * VERSION_WORDS + 4u);
     s.c3 = tab(T_VERSIONS, version * VERSION_WORDS + 5u);
+    if (spliced) {
+        s.length = DATA_UNIT_BYTES;
+        s.c1 = tab(T_VERSIONS, version * VERSION_WORDS + VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES)
+            ^ tab(T_VERSIONS, version_looked_up * VERSION_WORDS + 2u);
+        s.c2 = 4u | SLICE_SPLICED_ACROSS_THE_END_OF_A_LOCATION;
+        s.c3 = 0u;
+    }
     return s;
 }
 // 单元区里一个没有落点的 16 KiB 槽（判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第三节第 2 条，攻方改法 F-零槽）：
 // 基线与枚举写表里每次写碰过的槽都有落点，没有落点就是这条流的每个崩溃状态里都没写过，CPU 的稀疏读者（memory_pool 的 PoolReader）读出全零。
 // before 是同一块盘上排在 offset 前面最近的落点：它的第 9、10 字是它之后、下一个落点之前在单元区里、整 16 KiB 在盘内、谁都没写过的槽 [第 9 字, 第 10 字)，
 // 第 11 字是从哪一槽起整 32 KiB 不在盘内（crash_judge_tables::UnitSlotsNothingWroteAfterALocation）。读的起点按槽对齐、读的几槽都在那一段里，
-// 交回版本表最前面那两条全零单元之一（crash_judge_tables::ZERO_UNIT_VERSIONS：第 0 条 32 KiB，盘尾只装得下 16 KiB 的槽交第 1 条 16 KiB），别的交 NONE
+// 交回版本表最前面那两条全零单元之一（crash_judge_tables::ZERO_UNIT_VERSIONS：第 0 条 32 KiB，盘尾只装得下 16 KiB 的槽交第 1 条 16 KiB）；
+// 起点是空段的最后一槽、读两槽、整 32 KiB 在盘内的交 ZERO_SIXTEEN_KIBIBYTES_FOLLOWED_BY_THE_NEXT_SLOT（read_at 拼读）：那时空段止于下一个落点的起点
+// （空段的终点取同一块盘上下一个落点的起点与盘尾里小的那个，crash_judge_tables::unit_slots_nothing_wrote_after_each_location；整 32 KiB 在盘内 ⇒ 下一槽不过盘尾），
+// 下一个落点就是 before + 1、是单元槽（空段在单元区里，之后的落点都是单元槽）。别的交 NONE
 fn zero_unit_version_over_a_slot_nothing_wrote(before: u32, offset: vec2<u32>, length: u32) -> u32 {
     // 起点不按槽对齐、或槽号装不进 32 位（偏移 ≥ 2⁴⁶，盘装不下）的不认：槽号 = 偏移 >> SLOT_SHIFT 装进 32 位要偏移 < 2^(32 + SLOT_SHIFT) = 2⁴⁶，
     // 也就是偏移的高 32 位 offset.y < 2^SLOT_SHIFT、offset.y >> SLOT_SHIFT 为 0，即下面守卫里的那一项（判决 `research/prompts/m3-batch32-code-r1-main-verification.md`
@@ -508,10 +552,15 @@
     let first_slot_past_the_gap = location_word(before, 10u);
     // 读的几槽都在空段 [第 9 字, 第 10 字) 里：先判起点不越过空段的终点，再拿差比，不做会回绕的 first_slot + slots_read
     if (((offset.x & (NODE_BYTES - 1u)) | (offset.y >> SLOT_SHIFT)) != 0u || first_slot < location_word(before, 9u)
-        || first_slot > first_slot_past_the_gap || first_slot_past_the_gap - first_slot < slots_read) {
+        || first_slot > first_slot_past_the_gap) {
         return NONE;
     }
-    return select(0u, 1u, first_slot >= location_word(before, 11u));
+    let slots_left_in_the_gap = first_slot_past_the_gap - first_slot;
+    let whole_data_unit_fits = first_slot < location_word(before, 11u);
+    if (slots_left_in_the_gap < slots_read) {
+        return select(NONE, ZERO_SIXTEEN_KIBIBYTES_FOLLOWED_BY_THE_NEXT_SLOT, slots_left_in_the_gap == 1u && slots_read == 2u && whole_data_unit_fits);
+    }
+    return select(0u, 1u, !whole_data_unit_fits);
 }
 fn read_location(location: u32) -> Slice {
     let version = version_of_location(location);
@@ -603,6 +652,12 @@
         && eq64(a.birth_tree, b.birth_tree) && eq64(a.birth_txg, b.birth_txg);
 }
 // 位置条目里的整单元校验和对得上这一读的 CRC 吗
+// 一读的 32 KiB 整段 CRC 对不对得上（只比 CRC、不往下读字节：拼读那一读拼出的 CRC 照比，不记判不了）
+fn slice_whole_read_crc_matches(s: Slice, checksum: u32) -> bool {
+    return (s.c2 & 4u) != 0u && s.c1 == checksum;
+}
+// 拼读（read_at）那一读拼出的整段 CRC 对得上：往下要读跨过落点末尾的那几个字节，表里没有，记判不了、交 CPU 判（判决
+// `research/prompts/m3-batch33s-code-r1-main-verification.md` 第三节第 1 条「做不出的那一支记判不了、回落 CPU」）；对不上的照判对不上
 fn slice_crc_matches(s: Slice, unit_bytes: u32, checksum: u32) -> bool {
     if (unit_bytes == NODE_BYTES) {
         return s.c0 == checksum;
@@ -610,17 +665,22 @@
     if ((s.c2 & 4u) == 0u) {
         return false;
     }
+    if (s.c1 == checksum && (s.c2 & SLICE_SPLICED_ACROSS_THE_END_OF_A_LOCATION) != 0u) {
+        undecidable = UNDECIDABLE_SPLICED_READ_MATCHES;
+        return false;
+    }
     return s.c1 == checksum;
 }
 // 按两条位置条目读一个单元（recovery::read_unit_via_locations）
+// 两条位置条目在一个循环里读（一个 read_at 调用点：read_at 被这个函数展开的份数减半，扫描内核才装得下拼读，判决
+// `research/prompts/m3-batch33s-code-r1-main-verification.md` 第三节第 1 条「扫描内核余量只剩 17741，先量再加」）；次序与两次调用相同，第一条对得上就不读第二条
 fn read_unit_via_locations(p: Ptr, unit_bytes: u32) -> Slice {
-    let s0 = read_at(p.device0, slot_to_bytes(p.slot0), unit_bytes);
-    if (s0.version != NONE && slice_crc_matches(s0, unit_bytes, p.sum0)) {
-        return s0;
-    }
-    let s1 = read_at(p.device1, slot_to_bytes(p.slot1), unit_bytes);
-    if (s1.version != NONE && slice_crc_matches(s1, unit_bytes, p.sum1)) {
-        return s1;
+    for (var copy = 0u; copy < 2u; copy = copy + 1u) {
+        let is_the_second = copy == 1u;
+        let s = read_at(select(p.device0, p.device1, is_the_second), slot_to_bytes(select(p.slot0, p.slot1, is_the_second)), unit_bytes);
+        if (s.version != NONE && slice_crc_matches(s, unit_bytes, select(p.sum0, p.sum1, is_the_second))) {
+            return s;
+        }
     }
     return no_slice();
 }
diff --git a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
--- a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
@@ -139,7 +139,7 @@
 /// 一个判器在卡上要占多少显存（MiB，向上取整），不算 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`] 那一份（那一份每张卡只付一次，归卡池的账）：
 /// 表，加一次派活带 `requested_states_per_dispatch` 与这条流状态数里小的那个（至少 1）的草稿区，
 /// 加每次派活的结论与回读缓冲（每个状态各 [`VERDICT_WORDS`] 个字）。用例向卡池借卡时按它借
-/// （`crate::gpu_unit_checks::gpu_test_start::GpuCardPool::lease_card_waiting_for_room`）；
+/// （`crate::gpu_unit_checks::gpu_test_start::GpuCardPool::lease_card_or_report_not_run`，此刻借不到报本次未跑）；
 /// 借来的卡额度是它加那一份，判器照 [`states_in_flight_within`] 开出来的草稿区不少于这里算进去的状态数。
 #[must_use]
 pub fn mebibytes_a_judge_needs(
@@ -1338,24 +1338,25 @@
         }
     }
 
-    /// 三个内核此刻展开之后的表达式数与离预算的余量（2026-10-07 在删除线第 33 批的仓副本上用 naga =30.0.1 现量的：走读内核一个单元的各把键
-    /// 按读回那一份的落点认、全零单元守卫按槽号装不装得进 32 位判、F-α 那一处比 32 KiB 时另按 32 KiB 读、恢复内核的 oracle 认「没有文件」的那几版之后）：
+    /// 三个内核此刻展开之后的表达式数与离预算的余量（2026-10-08 在删除线第 36 批的仓副本上用 naga =30.0.1 现量的：恢复内核补 F2（F2_EXACT 那一形，
+    /// 走读改成一个调用点走两遍）、`read_at` 拼读空段最后一槽、`read_unit_via_locations` 两条位置条目并成一个 `read_at` 调用点之后；
+    /// 改之前是恢复 320 015、扫描 382 259、走读 337 872）：
     /// 改了哪个内核的源码，这里先红，量过新数、确认余量还够之后再改钉值，免得余量悄悄耗光、等上卡时才撞预算。
     #[test]
     fn each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget() {
         for (name, source, pinned_expressions, pinned_margin) in [
-            ("recovery", recovery_kernel_source(), 320_015, 79_985),
+            ("recovery", recovery_kernel_source(), 318_224, 81_776),
             (
                 "checker-scan",
                 crate::crash_judge_gpu::checker_scan_kernel_source(),
-                382_259,
-                17_741,
+                352_195,
+                47_805,
             ),
             (
                 "checker-walk",
                 crate::crash_judge_gpu::checker_walk_kernel_source(),
-                337_872,
-                62_128,
+                346_138,
+                53_862,
             ),
         ] {
             let size = super::kernel_inlined_size(&source);
diff --git a/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl b/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
--- a/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
@@ -40,11 +40,140 @@
     out.has_effective = true;
     out.effective_instance = effective.instance;
     out.effective_txg = effective.txg;
-    let expected = expected_version_index(effective.instance, effective.txg);
-    let walked = walk_to_file(effective, cfg_choice.cfg, expected);
-    out.outcome = walked.outcome;
-    out.failure = walked.failure;
-    out.content_matches = walked.content_matches;
+    // 走读失败、看 journal 的那一遍再走一次退回见证到的那一版（recovery::recover 的 F2）：一个调用点走两遍，内核展开只多一份判法、不多一份走读
+    var root_to_walk = effective;
+    for (var walk_round = 0u; walk_round < 2u; walk_round = walk_round + 1u) {
+        let walked = walk_to_file(root_to_walk, cfg_choice.cfg, expected_version_index(root_to_walk.instance, root_to_walk.txg));
+        // 第一遍照报；退回的那一版走不通时照报第一遍的失败（落到的根也不换）
+        if (walk_round == 0u || walked.outcome != OUTCOME_FAILED) {
+            out.effective_instance = root_to_walk.instance;
+            out.effective_txg = root_to_walk.txg;
+            out.outcome = walked.outcome;
+            out.failure = walked.failure;
+            out.content_matches = walked.content_matches;
+        }
+        if (walk_round == 1u || walked.outcome != OUTCOME_FAILED || !consult) {
+            break;
+        }
+        let fallback = version_witnessed_by_the_system_configuration_to_fall_back_to(cfg_choice.cfg, chosen.root, effective);
+        if (!fallback.found) {
+            break;
+        }
+        root_to_walk = fallback.root;
+    }
+    return out;
+}
+
+// recovery::system_configuration_slots_of_one_device_read_once 收进 self_verified 的那一判：SystemConfiguration::parse_slot 交 Ok
+// （magic、整槽校验和、incompat 位认得、每区槽数在 [4, 16]；读者收不收那几个池级值不在这一判里）且 fsid 与所选系统配置的相同
+fn system_configuration_slot_self_verified_with_the_fsid(s: Slice, fsid: vec4<u32>) -> bool {
+    if (s.version == NONE || s.length < SYSTEM_CONFIGURATION_SLOT_BYTES || rd32(s.base, 0u) != MAGIC_SFSB || s.c0 != 1u) {
+        return false;
+    }
+    let incompat0 = rd8(s.base, 6u);
+    if ((incompat0 & 0xFDu) != 0u || !all_zero(s.base, 7u, 31u) || (incompat0 & 2u) == 0u) {
+        return false;
+    }
+    let slots_per_region = rd8(s.base, 362u);
+    return slots_per_region >= 4u && slots_per_region <= 16u && fsid_matches(s.base, 102u, fsid);
+}
+// 系统配置槽里 journal tail 的偏移（system_configuration.rs 的 TAIL_OFFSET）
+const SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET: u32 = 469u;
+// 内存池按扇区对齐判读请求（memory_pool 的 check_request_against_the_device_contract）：槽距不按 512 对齐的槽 1 读不到、算设备报错
+const SECTOR_BYTES_OF_THE_MEMORY_POOL: u32 = 512u;
+
+struct FallbackVersion {
+    found: bool,
+    root: RootFields,
+};
+// recovery::version_witnessed_by_the_system_configuration_to_fall_back_to，攻方 F2_EXACT 那一形（判决
+// `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 行、第三节第 2 条）：所选那一版（一条记录都没施加的所选根本身）
+// 走读失败、系统配置没见证它时，只退到计数器 = c_见证 的那条记录所属那一版本身——根环里 (txg, 实例) 与它恰好相等、读得出的那条根
+// （R_ROOTS 按根环次序记着 choose_root 自证过的根，取第一条）。见证判不出、见证过、c_见证 = 0、那条记录读不出、那条根读不出都不退。
+// 「见证过」：所选那一版带末条标志的记录读得出 ⇒ 它的计数器 ≤ c_见证；读不出而 c_见证 那条记录读得出 ⇒ 那条记录的 (txg, 实例) 不比所选那一版旧；
+// 两条都读不出当见证过
+fn version_witnessed_by_the_system_configuration_to_fall_back_to(cfg: SysConfig, chosen: RootFields, selected: RootFields) -> FallbackVersion {
+    var out: FallbackVersion;
+    out.found = false;
+    out.root = invalid_root();
+    if (selected.instance != chosen.instance || !eq64(selected.txg, chosen.txg)) {
+        return out;
+    }
+    // c_见证：每块盘两个系统配置槽（偏移 0 与所选系统配置的槽距）里自证过的 journal tail 取最大；见证判不出（R2）：有槽读不到、又没有一块盘两槽都自证过
+    let device_count = hdr(3u);
+    var witnessed_counter = vec2<u32>(0u, 0u);
+    var some_slot_reports_a_read_error = false;
+    var some_device_has_both_slots_self_verified = false;
+    for (var d = 0u; d < device_count; d = d + 1u) {
+        var self_verified_slots = 0u;
+        for (var slot = 0u; slot < 2u; slot = slot + 1u) {
+            let offset = select(0u, cfg.spacing, slot == 1u);
+            if ((offset % SECTOR_BYTES_OF_THE_MEMORY_POOL) != 0u) {
+                some_slot_reports_a_read_error = true;
+                continue;
+            }
+            let s = read_at(hdr(12u + d), u64_of(offset), SYSTEM_CONFIGURATION_SLOT_BYTES);
+            if (!system_configuration_slot_self_verified_with_the_fsid(s, cfg.fsid)) {
+                continue;
+            }
+            self_verified_slots = self_verified_slots + 1u;
+            let tail = rd64(s.base, SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET);
+            if (lt64(witnessed_counter, tail)) {
+                witnessed_counter = tail;
+            }
+        }
+        if (self_verified_slots == 2u) {
+            some_device_has_both_slots_self_verified = true;
+        }
+    }
+    if (some_slot_reports_a_read_error && !some_device_has_both_slots_self_verified) {
+        return out;
+    }
+    var has_last_record_of_the_selected_version = false;
+    var last_record_counter = vec2<u32>(0u, 0u);
+    var has_witnessed_record = false;
+    var witnessed_record_txg = vec2<u32>(0u, 0u);
+    var witnessed_record_instance = 0u;
+    for (var i = 0u; i < record_count; i = i + 1u) {
+        let r = record_at(i);
+        if (r.instance == selected.instance && eq64(r.txg, selected.txg) && r.last
+            && (!has_last_record_of_the_selected_version || lt64(last_record_counter, r.counter))) {
+            has_last_record_of_the_selected_version = true;
+            last_record_counter = r.counter;
+        }
+        if (eq64(r.counter, witnessed_counter)
+            && (!has_witnessed_record || lt64(witnessed_record_txg, r.txg)
+                || (eq64(witnessed_record_txg, r.txg) && witnessed_record_instance < r.instance))) {
+            has_witnessed_record = true;
+            witnessed_record_txg = r.txg;
+            witnessed_record_instance = r.instance;
+        }
+    }
+    let witnessed_record_is_older_than_the_selected_version = lt64(witnessed_record_txg, selected.txg)
+        || (eq64(witnessed_record_txg, selected.txg) && witnessed_record_instance < selected.instance);
+    var is_witnessed = true;
+    if (has_last_record_of_the_selected_version) {
+        is_witnessed = le64(last_record_counter, witnessed_counter);
+    } else if (has_witnessed_record) {
+        is_witnessed = !witnessed_record_is_older_than_the_selected_version;
+    }
+    if (is_witnessed || is_zero64(witnessed_counter) || !has_witnessed_record) {
+        return out;
+    }
+    for (var k = 0u; k < root_count; k = k + 1u) {
+        var s: Slice = no_slice();
+        s.version = sw(R_ROOTS + k * 2u);
+        s.base = B[T_VERSION_BYTES] + tab(T_VERSIONS, s.version * VERSION_WORDS + 1u);
+        s.length = cfg.physical_block_size;
+        s.c0 = tab(T_VERSIONS, s.version * VERSION_WORDS + 2u);
+        let root = parse_root(s, cfg.fsid);
+        if (root.instance == witnessed_record_instance && eq64(root.txg, witnessed_record_txg)
+            && (lt64(root.txg, chosen.txg) || (eq64(root.txg, chosen.txg) && root.instance < chosen.instance))) {
+            out.found = true;
+            out.root = root;
+            return out;
+        }
+    }
     return out;
 }
 struct NewestRoot {
diff --git a/crates/singlefs-checker-tier/src/crash_judge_tables.rs b/crates/singlefs-checker-tier/src/crash_judge_tables.rs
--- a/crates/singlefs-checker-tier/src/crash_judge_tables.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_tables.rs
@@ -38,8 +38,9 @@
 use crate::crash_facts::{persisted_writes_of_state, SegmentFact, TornImageFact};
 
 /// 表的形态版本：落点、版本、检查字或发布表的布局变了就抬它（4：版本表最前面两条全零单元、落点第 9–11 字记它之后没写过的槽，
-/// [`ZERO_UNIT_VERSIONS`]；5：期望版本表第 5 字记这一版有没有文件，[`EXPECTED_VERSION_WITHOUT_A_FILE`]）。
-pub const JUDGE_TABLES_VERSION: u32 = 5;
+/// [`ZERO_UNIT_VERSIONS`]；5：期望版本表第 5 字记这一版有没有文件，[`EXPECTED_VERSION_WITHOUT_A_FILE`]；6：版本表每条多一个第 8 字，
+/// [`VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES`]）。
+pub const JUDGE_TABLES_VERSION: u32 = 6;
 
 /// 一个落点最多被几次写盖住：GPU 内核里组合掩码是两个 u32（版本表里一个版本的第 0 字是低 32 位、第 7 字是高 32 位）。
 /// 错位复用那条历史把系统配置 0 号槽写了 34 次，32 位装不下。
@@ -1325,7 +1326,20 @@
                 JudgeLocationKind::UnitSlot,
                 &zero_unit[..usize::try_from(zero_unit_bytes).expect("单元长装得进 usize")],
             );
-            versions.extend([0, 0, checks[0], checks[1], checks[2], checks[3], NONE, 0]);
+            versions.extend([
+                0,
+                0,
+                checks[0],
+                checks[1],
+                checks[2],
+                checks[3],
+                NONE,
+                0,
+                crc_shifted_past_sixteen_zero_kibibytes(
+                    JudgeLocationKind::UnitSlot,
+                    &zero_unit[..usize::try_from(zero_unit_bytes).expect("单元长装得进 usize")],
+                ),
+            ]);
         }
         version_bytes.extend(bytes_to_words(&zero_unit));
         let gaps = self.unit_slots_nothing_wrote_after_each_location();
@@ -1357,6 +1371,7 @@
                     version.checks[3],
                     index(equal_masks.len() / 2),
                     combination_hi,
+                    crc_shifted_past_sixteen_zero_kibibytes(location.kind, &version.bytes),
                 ]);
                 version_bytes.extend(bytes_to_words(&version.bytes));
                 for mask in &version.sector_equals {
@@ -1471,8 +1486,32 @@
 pub const WRITE_WORDS: usize = 12;
 /// 落点表每条几个 u32。
 pub const LOCATION_WORDS: usize = 12;
-/// 版本表每条几个 u32。
-pub const VERSION_WORDS: usize = 8;
+/// 版本表每条几个 u32：组合低 32 位、字节起点、四个检查字（[`version_checks`]）、逐扇区等不等掩码起点、组合高 32 位、
+/// 第 8 字 [`VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES`]。
+pub const VERSION_WORDS: usize = 9;
+/// 版本表第 8 字（[`crc_shifted_past_sixteen_zero_kibibytes`]）：GPU 读者 `read_at` 拼读跨过落点末尾的 32 KiB 时拼整段 CRC 用
+/// （`crash_judge_common.wgsl` 同名常量）。
+pub const VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES: usize = 8;
+
+/// 单元槽的一版字节的前 16 KiB 后面接 16 KiB 全零，那 32 KiB 的 CRC-32C 异或 16 KiB 全零的 CRC-32C（版本表第 8 字）。CRC-32C 对异或线性：
+/// 它再异或任意一段 16 KiB 的 CRC-32C，就是「这一版前 16 KiB 接那一段」32 KiB 的 CRC-32C（第 32 批攻方 Z2，判决
+/// `research/prompts/m3-batch33s-code-r1-main-verification.md` 第三节第 1 条）；GPU 读者拼「16 KiB 节点加下一槽」「空槽加下一已写槽前 16 KiB」
+/// 两形的整段 CRC 靠它，不在内核里做 CRC 的乘法。别的落点（不是单元槽、或不到 16 KiB）记 0：不会被拼读。
+#[must_use]
+pub fn crc_shifted_past_sixteen_zero_kibibytes(kind: JudgeLocationKind, bytes: &[u8]) -> u32 {
+    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
+    match kind {
+        JudgeLocationKind::UnitSlot if bytes.len() >= node_bytes => {
+            let mut followed_by_zeros = bytes[..node_bytes].to_vec();
+            followed_by_zeros.resize(2 * node_bytes, 0);
+            crc32_castagnoli(&followed_by_zeros) ^ crc32_castagnoli(&vec![0u8; node_bytes])
+        }
+        JudgeLocationKind::UnitSlot
+        | JudgeLocationKind::SystemConfiguration
+        | JudgeLocationKind::RootSlot
+        | JudgeLocationKind::JournalRecord => 0,
+    }
+}
 /// 更晚重叠写表每条几个 u32。
 pub const LATER_WORDS: usize = 4;
 /// 发布表每条几个 u32。
@@ -1566,3 +1605,46 @@
     }
     words
 }
+
+#[cfg(test)]
+mod tests {
+    use super::{crc_shifted_past_sixteen_zero_kibibytes, JudgeLocationKind};
+    use singlefs_core::checksum::crc32_castagnoli;
+
+    const SIXTEEN_KIBIBYTES: usize = 16384;
+
+    fn sixteen_kibibytes_with_the_pattern(multiplier: usize, modulus: usize) -> Vec<u8> {
+        (0..SIXTEEN_KIBIBYTES)
+            .map(|index| u8::try_from(index * multiplier % modulus).expect("小于 256"))
+            .collect()
+    }
+
+    /// 版本表第 8 字异或任意一段 16 KiB 的 CRC-32C，就是「这一版前 16 KiB 接那一段」32 KiB 的 CRC-32C：GPU 读者拼读空段最后一槽
+    /// （前一半全零 16 KiB、后一半是下一个落点的前 16 KiB）靠这一条；前一半不是全零时同样成立。不是单元槽的落点记 0。
+    #[test]
+    fn the_shifted_crc_of_a_version_xored_with_the_crc_of_the_next_sixteen_kibibytes_is_the_crc_of_the_thirty_two_kibibytes(
+    ) {
+        let next = sixteen_kibibytes_with_the_pattern(13, 253);
+        for first in [
+            vec![0u8; SIXTEEN_KIBIBYTES],
+            sixteen_kibibytes_with_the_pattern(7, 251),
+        ] {
+            let mut thirty_two_kibibytes = first.clone();
+            thirty_two_kibibytes.extend_from_slice(&next);
+            assert_eq!(
+                crc_shifted_past_sixteen_zero_kibibytes(JudgeLocationKind::UnitSlot, &first)
+                    ^ crc32_castagnoli(&next),
+                crc32_castagnoli(&thirty_two_kibibytes),
+                "拼出的 32 KiB CRC 与连续读出的 32 KiB 的 CRC 相同"
+            );
+        }
+        assert_eq!(
+            crc_shifted_past_sixteen_zero_kibibytes(
+                JudgeLocationKind::JournalRecord,
+                &next[..4096]
+            ),
+            0,
+            "journal 记录槽不会被拼读，记 0"
+        );
+    }
+}
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -58,6 +58,10 @@
 //! 挂载之后由记录施加出来的那一版各一条），树表指针整条全零照 core 跟过去（根与它的记录、由记录施加出来的那一版的记录各一条），都整份比。
 //! 第 36 批：下一次挂载先施加的那一版照 I-4.1 那一份重放施加的第一次发布认（改法 R1）——三条记录的发布首条没落（断号）、
 //! 前缀停在后面那次发布的点名单元上（第一次那一版照认）各一条流，CPU 一侧先在整条落盘的镜像上钉形，再整份比。
+//! 第 36 批（删除线）：GPU 恢复内核照 CPU 的 F2（F2_EXACT 那一形）退回见证到的那一版——设计里那一退（R 走读失败、V 已确认，落 V）与攻方 A 形
+//! （V 的根槽也读不出，报读不出、不退到 V⁻）各一条流；I-3.5 按全部版本走完之后的折叠表取单元（H-S1-fold A 臂）一条流；read_at 拼读那三形
+//! （空段右端一槽按 32 KiB 读的数据指针记判不了回落 CPU、「16 KiB 节点加下一槽」照表判、F-α 的「空槽加下一已写槽前 16 KiB」拿拼出的 CRC 照比）各一条流；
+//! 删完那条流钉 CPU 一个违例都没有，C744 ① 那条流钉两种版本同在的状态至少一个；借卡一律经 `lease_card_or_report_not_run`（借不到报本次未跑）。
 //! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
 //! extent 下段叶的数据指针、分配记录树内部条目子指针上各一条坏流整份比（core 读这几条指针都判、GPU 恢复内核照判）；这几处除实例表指针外
 //! 另各有一条 MAC 第 0 字节非 0 的坏流（core 连加密、压缩那几段一并判，GPU 恢复内核同判）；extent 上段叶内联的数据指针另有一条算法类型写 1 的坏流
@@ -86,7 +90,7 @@
 
 use common::{
     abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, file_content,
-    geometry, parameters, BuiltPool,
+    geometry, overwrite_in_process_as_instance_one_and_advance_the_pool, parameters, BuiltPool,
 };
 use common_crash_points::{mount_writable_step, overwrite_step, record_pool_build, roll_back_step};
 use common_deletion_in_progress::{
@@ -100,6 +104,7 @@
 use singlefs_checker_tier::crash::{
     judge_layer0_state_range_against_the_version_table, layer0_plan_state_count,
     Layer0OracleViolationKind, Layer0SegmentExpansion, Layer0StateJudgement, Layer0VersionTable,
+    LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE,
 };
 use singlefs_checker_tier::crash_amplification::{
     gpu_recovery_words, gpu_verdict_items, judge_my_blocks_on_gpu_cards, plan_crash_points,
@@ -130,6 +135,7 @@
 };
 use singlefs_core::lineage_tree::LineageContainerCapacity;
 use singlefs_core::mount::RollbackTarget;
+use singlefs_core::recovery::RecoveryOutcome;
 use singlefs_core::tombstone_tree::{
     TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests,
 };
@@ -469,6 +475,22 @@
     flow: &mut RecordedFlow,
     root_write: usize,
 ) {
+    for copy in copies_of_the_tree_table_named_by_the_root_write(flow, root_write) {
+        match &mut flow.writes[copy].contents {
+            WrittenContents::Bytes(bytes) => {
+                let last = bytes.len() - 1;
+                bytes[last] ^= 0x01;
+            }
+            WrittenContents::Zeros { .. } => panic!("单元写是普通写"),
+        }
+    }
+}
+
+/// 第 `root_write` 次写（根槽写）指着的树表单元的两份副本（写下标，位置条目 0、1 的次序）。
+fn copies_of_the_tree_table_named_by_the_root_write(
+    flow: &RecordedFlow,
+    root_write: usize,
+) -> Vec<usize> {
     let root_bytes = flow.writes[root_write]
         .bytes()
         .expect("根槽写带字节")
@@ -499,15 +521,7 @@
         })
         .collect();
     assert_ne!(copies[0], copies[1], "树表单元的两份副本是两次写");
-    for copy in copies {
-        match &mut flow.writes[copy].contents {
-            WrittenContents::Bytes(bytes) => {
-                let last = bytes.len() - 1;
-                bytes[last] ^= 0x01;
-            }
-            WrittenContents::Zeros { .. } => panic!("单元写是普通写"),
-        }
-    }
+    copies
 }
 
 /// 改一次写里的几个字节之后把那个结构的整槽校验和字段重封（CRC-32C 住字段前 4 字节、后 28 字节 0）：拦住它的只剩被改的字段本身。
@@ -2067,6 +2081,21 @@
     );
 }
 
+/// 一次恢复报出的那条根（(实例代号, checkpoint_txg)）：读出、没有文件两种结局带的那一条，失败时带的那一条（择不到根时没有）。
+fn root_reported_by_the_recovery(outcome: &RecoveryOutcome) -> Option<(u32, u64)> {
+    match outcome {
+        RecoveryOutcome::FileRead { root, .. } | RecoveryOutcome::NoFile { root } => {
+            Some((root.0 .0, root.1 .0))
+        }
+        RecoveryOutcome::Failed { root, .. } => root.map(|(instance, txg)| (instance.0, txg.0)),
+    }
+}
+
+/// 恢复字（[`recovery_words_of`]）第 2、3 字的结局类别：没有文件、读出、失败。
+const RECOVERY_WORD_CLASS_NO_FILE: u32 = 0;
+const RECOVERY_WORD_CLASS_FILE_READ: u32 = 1;
+const RECOVERY_WORD_CLASS_FAILED: u32 = 2;
+
 fn oracle_index(kind: Layer0OracleViolationKind) -> u32 {
     u32::try_from(
         Layer0OracleViolationKind::EVERY_KIND
@@ -2150,6 +2179,12 @@
     cpu_red_items_of_each_state: Vec<String>,
     /// GPU 在每个状态上交回的结论字（按状态序号）：要按判不了的原因分开看、或照流水线那样判不了就回落 CPU 的用例读它。
     gpu_words_of_each_state: Vec<[u32; VERDICT_WORDS]>,
+    /// CPU 看 journal 那一遍在每个状态上报的根（`RecoveryOutcome` 里的那一条，(实例代号, checkpoint_txg)；退回见证到的那一版时是退到的那一版，
+    /// 择不到根时 `None`），按状态序号：要钉「落到哪一版、所选根是谁」的用例读它。
+    cpu_consulted_reported_root_of_each_state: Vec<Option<(u32, u64)>>,
+    /// CPU 在每个状态上的恢复字（[`recovery_words_of`]：看 journal 那一遍实际走的那条根的实例与 txg 低 32 位、两遍的结局类别 0 没有文件 / 1 读出 / 2 失败），
+    /// 按状态序号。
+    cpu_recovery_words_of_each_state: Vec<[u32; 4]>,
     /// 看 journal 那一遍落到的根或两遍的结局类别与 CPU 不同的状态数（算进不一致）。
     recovery_mismatches: u64,
     mismatches: Vec<String>,
@@ -2247,10 +2282,10 @@
     );
     let packed = PackedJudgeTables::of(&tables);
     let states_per_dispatch = states_of_one_scratch_lane_on_every_card_of(cards);
-    let lease = cards.lease_card_waiting_for_room(
+    let lease = cards.lease_card_or_report_not_run(
         flow.name,
         mebibytes_a_judge_needs(&packed, states_per_dispatch),
-    );
+    )?;
     let gpu_started = Instant::now();
     let gpu = on_the_card_or_report_not_run_when_out_of_memory(
         flow.name,
@@ -2277,7 +2312,15 @@
     let slice = state_count.div_ceil(threads * 8).max(1);
     let next_slice = std::sync::atomic::AtomicU64::new(0);
     let slice_count = state_count.div_ceil(slice);
-    type JudgedOnCpu = (u32, u128, String, String, [u32; 4], u128);
+    type JudgedOnCpu = (
+        u32,
+        u128,
+        String,
+        String,
+        [u32; 4],
+        u128,
+        Option<(u32, u64)>,
+    );
     let mut slices: Vec<(u64, Vec<JudgedOnCpu>)> = std::thread::scope(|scope| {
         let handles: Vec<_> = (0..threads.min(slice_count))
             .map(|_| {
@@ -2316,6 +2359,7 @@
                                     ),
                                     recovery_words_of(judgement),
                                     expected_pool_checker_held_bits(judgement),
+                                    root_reported_by_the_recovery(&judgement.consulted.outcome),
                                 ));
                             },
                         );
@@ -2346,16 +2390,32 @@
         cpu_checker_bits_of_each_state: Vec::with_capacity(cpu.len()),
         cpu_red_items_of_each_state: Vec::with_capacity(cpu.len()),
         gpu_words_of_each_state: gpu.clone(),
+        cpu_consulted_reported_root_of_each_state: Vec::with_capacity(cpu.len()),
+        cpu_recovery_words_of_each_state: Vec::with_capacity(cpu.len()),
         recovery_mismatches: 0,
         mismatches: Vec::new(),
         checker_mismatches: Vec::new(),
     };
     for (
         ordinal,
-        (cpu_bits, cpu_checker_bits, cpu_items, cpu_report, cpu_recovery, cpu_checker_held_bits),
+        (
+            cpu_bits,
+            cpu_checker_bits,
+            cpu_items,
+            cpu_report,
+            cpu_recovery,
+            cpu_checker_held_bits,
+            cpu_consulted_reported_root,
+        ),
     ) in cpu.iter().enumerate()
     {
         comparison
+            .cpu_consulted_reported_root_of_each_state
+            .push(*cpu_consulted_reported_root);
+        comparison
+            .cpu_recovery_words_of_each_state
+            .push(*cpu_recovery);
+        comparison
             .cpu_checker_held_invariants
             .extend(invariant_names(*cpu_checker_held_bits));
         comparison
@@ -2702,11 +2762,13 @@
     let packed = PackedJudgeTables::of(&tables);
     // 两个判器同时活着：借的是窄的那一档（256 个）加宽的那一档（三条缓冲）各自要的，两份表各算一份（约 6.45 GiB）
     let states_per_lane = states_of_one_scratch_lane_on_every_card_of(cards);
-    let lease = cards.lease_card_waiting_for_room(
+    let Some(lease) = cards.lease_card_or_report_not_run(
         "verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several",
         mebibytes_a_judge_needs(&packed, 256)
             + mebibytes_a_judge_needs(&packed, 3 * states_per_lane),
-    );
+    ) else {
+        return;
+    };
     let card = lease.card();
     assert_eq!(
         states_per_scratch_lane(card.device().limits().max_storage_buffer_binding_size),
@@ -2827,10 +2889,12 @@
     let state_count = tables.state_count;
     let packed = PackedJudgeTables::of(&tables);
     // 两个判器先后建；前一个的缓冲什么时候还给驱动看 wgpu 收拾的时机，借两份
-    let lease = cards.lease_card_waiting_for_room(
+    let Some(lease) = cards.lease_card_or_report_not_run(
         "a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels",
         2 * mebibytes_a_judge_needs(&packed, usize::MAX),
-    );
+    ) else {
+        return;
+    };
     let card = lease.card();
     let Some((unplanted, planted)) = on_the_card_or_report_not_run_when_out_of_memory(
         "a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels",
@@ -3029,13 +3093,15 @@
         })
         .expect("建得出表"),
     );
-    let lease = cards.lease_card_waiting_for_room(
+    let Some(lease) = cards.lease_card_or_report_not_run(
         "the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest",
         mebibytes_a_judge_needs(
             &tables_of_the_flow,
             states_of_one_scratch_lane_on_every_card_of(cards),
         ),
-    );
+    ) else {
+        return;
+    };
     let card = std::sync::Arc::new(lease.card().clone());
     let plan = plan_crash_points(&flow);
     let version = JudgeVersion([7u8; 32]);
@@ -3749,10 +3815,12 @@
     .expect("建得出表");
     let state_count = tables.state_count;
     let packed = PackedJudgeTables::of(&tables);
-    let lease = cards.lease_card_waiting_for_room(
+    let Some(lease) = cards.lease_card_or_report_not_run(
         "the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count",
         mebibytes_a_judge_needs(&packed, usize::MAX),
-    );
+    ) else {
+        return;
+    };
     let Some(verdicts) = on_the_card_or_report_not_run_when_out_of_memory(
         "the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count",
         lease.card(),
@@ -11229,17 +11297,54 @@
 /// [`record_three_unit_file_applied_only_by_its_records_after_a_mount`]）。挂载写了实例 2 的行、覆盖写的记录落了而根槽没落的状态上，
 /// txg 4 是由记录施加出来的那一版、覆盖写那一版是下一次挂载先施加的那一版，两样同在（推的，读录制的次序）：扫描内核
 /// `judge_allocation_records_disjoint` 与 `judge_allocation_generations_against_unit_births` 的末圈与前两段都走到。每个状态 GPU 与 CPU 逐位相同。
+/// 两样同在的状态至少一个（判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S6 行）：CPU 看 journal 那一遍报的根（所选根，
+/// 根环里最新的那条）是实例 2 的（挂载写行那一版起每条根的实例表都带实例 1 那一行 (1, 4)，txg 4 就是由记录施加出来的那一版），
+/// 又从它往上施加了记录（实际走的那条根不是它：覆盖写那一版是下一次挂载先施加的那一版）。
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_on_states_holding_a_version_applied_by_its_records_beside_the_version_the_next_mount_applies_first(
 ) {
-    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
-        "the_gpu_judge_agrees_with_the_cpu_on_states_holding_a_version_applied_by_its_records_beside_the_version_the_next_mount_applies_first",
-        &record_three_unit_file_applied_only_by_its_records_after_a_mount(
-            "crash-judge-gpu-c744-version-applied-by-records-beside-the-next-mount-version",
-            "three-unit-file-applied-only-by-its-records-beside-the-version-the-next-mount-applies-first",
-            &leave_the_records_before_the_mount_as_they_are,
-        ),
-        &[],
+    const INSTANCE_OF_THE_MOUNT: u32 = 2;
+    let test_name = "the_gpu_judge_agrees_with_the_cpu_on_states_holding_a_version_applied_by_its_records_beside_the_version_the_next_mount_applies_first";
+    let flow = record_three_unit_file_applied_only_by_its_records_after_a_mount(
+        "crash-judge-gpu-c744-version-applied-by-records-beside-the-next-mount-version",
+        "three-unit-file-applied-only-by-its-records-beside-the-version-the-next-mount-applies-first",
+        &leave_the_records_before_the_mount_as_they_are,
+    );
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
+        return;
+    };
+    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
+        return;
+    };
+    let states_holding_both_versions = comparison
+        .cpu_consulted_reported_root_of_each_state
+        .iter()
+        .zip(&comparison.cpu_recovery_words_of_each_state)
+        .filter(|(reported_root, recovery_words)| {
+            let effective_root_reported = recovery_words[0] != u32::MAX;
+            reported_root.is_some_and(|(instance, txg)| {
+                instance == INSTANCE_OF_THE_MOUNT
+                    && effective_root_reported
+                    && (recovery_words[0], u64::from(recovery_words[1]))
+                        != (instance, txg & 0xFFFF_FFFF)
+            })
+        })
+        .count();
+    eprintln!(
+        "CRASH_JUDGE_GPU_C744 flow={} states={} states_holding_both_versions={states_holding_both_versions}",
+        flow.name, comparison.states
+    );
+    assert!(
+        states_holding_both_versions > 0,
+        "流 {}：至少一个状态同时带由记录施加出来的那一版（txg 4）与下一次挂载先施加的那一版（所选根是实例 {INSTANCE_OF_THE_MOUNT} 的、又施加了记录）",
+        flow.name
+    );
+    assert!(
+        comparison.mismatches.is_empty(),
+        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
     );
 }
 
@@ -11321,7 +11426,9 @@
 /// 删完的流（[`record_new_pool_through_the_last_deletion_batch`]）带「没有文件的那几版」：意图那一版（txg 5）起没有文件，三批（txg 6–8）不进表、
 /// 照比它旧的最新那一项（意图那一版）判（D8（核心索引结构） 已定项 16 第 4 条；第 33 批 R 报告「停下交主 agent」第二项）。
 /// 恢复落到 txg 5–8 报没有文件的状态上 CPU 的 oracle 一个违例都不记；GPU 恢复内核的期望版本表第 5 字记这一版有没有文件、`classify_oracle` 照认，
-/// 每个状态逐位相同（不认的话这几版报没有文件判成「更旧的根下面有文件」）。
+/// 每个状态逐位相同（不认的话这几版报没有文件判成「更旧的根下面有文件」）。另钉 CPU 两遍 oracle 与记录核对器在每个状态上一个违例都没有、
+/// 至少一个状态看 journal 那一遍落到意图那一版 (1, 5) 报没有文件（判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S5 行：
+/// 「没有文件的那几版」写错身份时 CPU、GPU 一起判违例、逐位仍相同，只比两边对不对得上看不出来）。
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_on_the_deleted_flow_against_a_version_table_without_a_file_from_the_intent_on(
 ) {
@@ -11346,11 +11453,526 @@
     ) else {
         return;
     };
+    assert_eq!(
+        comparison.cpu_red, 0,
+        "流 {}：CPU 两遍 oracle 与记录核对器在每个状态上一个违例都没有（意图那一版起没有文件，表里照认）",
+        flow.name
+    );
+    let intent_version = (
+        INSTANCE_OF_THE_PUBLISHES.0,
+        u32::try_from(versions_without_a_file[0].checkpoint_txg.0).expect("txg 5 装得进 u32"),
+    );
+    assert!(
+        comparison
+            .cpu_recovery_words_of_each_state
+            .iter()
+            .any(|words| (words[0], words[1]) == intent_version
+                && words[2] == RECOVERY_WORD_CLASS_NO_FILE),
+        "流 {}：至少一个状态看 journal 那一遍落到意图那一版 {intent_version:?}、报没有文件",
+        flow.name
+    );
     assert!(
         comparison.mismatches.is_empty(),
         "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
         flow.name,
         comparison.mismatches.len(),
         mismatch_listing(&comparison)
+    );
+}
+
+// ─── 第 36 批（删除线）：GPU 恢复内核的 F2（F2_EXACT 那一形）、I-3.5 按全部版本走完之后的折叠表取单元（H-S1-fold）、read_at 拼读（H-S2-tail） ───
+
+/// 根槽写里自证校验和字段的偏移（`reseal_root_record` 重封的那 32 字节）。
+const ROOT_RECORD_CHECKSUM_FIELD_OFFSET: usize = 138;
+
+/// 流里写出 txg `txg` 那条根的根槽写（恰好一次）。
+fn root_write_of_txg(flow: &RecordedFlow, txg: u64) -> usize {
+    let root_writes: Vec<usize> = flow
+        .writes
+        .iter()
+        .enumerate()
+        .filter(|(_, write)| {
+            write.kind == StepKind::RootRecordFua
+                && write.bytes().is_some_and(|bytes| {
+                    u64::from_le_bytes(
+                        bytes[ROOT_RECORD_CHECKPOINT_TXG_OFFSET
+                            ..ROOT_RECORD_CHECKPOINT_TXG_OFFSET + 8]
+                            .try_into()
+                            .expect("8 字节"),
+                    ) == txg
+                })
+        })
+        .map(|(index, _)| index)
+        .collect();
+    let [root_write] = root_writes.as_slice() else {
+        panic!("流里写出 txg {txg} 那条根的根槽写恰好一次：{root_writes:?}")
+    };
+    *root_write
+}
+
+/// F2 那两条流的底：新池新建文件（txg 3）→ 同一个进程、实例 1 里覆盖写成第二份内容（V，txg 4）→ 再覆盖写成第三份内容（R，txg 5），整条录下来；
+/// R 那次发布的树表两份副本各改坏末字节、不重封（[`break_both_copies_of_the_tree_table_named_by_the_root_write`]）：R 的根槽落了的状态上 R 走读失败。
+/// 每次发布按单元、记录、根槽、系统配置轮换的次序落盘（`transaction::persist_publish_writes`），R 的根槽落了而它的系统配置轮换没落的状态上，
+/// c_见证 是 V 那次发布末条记录的计数器，系统配置见证不到 R。
+fn record_two_overwrites_whose_second_tree_table_does_not_read_back(
+    tag: &str,
+    name: &'static str,
+) -> RecordedFlow {
+    let mut pool = build_pool(tag);
+    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &second_content());
+    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &third_content());
+    let mut flow = recorded_flow_of_the_pool(
+        &pool,
+        name,
+        vec![
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(3),
+                content: file_content(),
+            },
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(4),
+                content: second_content(),
+            },
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(5),
+                content: third_content(),
+            },
+        ],
+    );
+    assert_eq!(
+        root_write_of_txg(&flow, R_OF_THE_F2_FLOWS.1),
+        flow.judged_root_index,
+        "流里最后一条根槽写是 R 的"
+    );
+    let root_of_r = flow.judged_root_index;
+    break_both_copies_of_the_tree_table_named_by_the_root_write(&mut flow, root_of_r);
+    flow
+}
+
+/// F2 那两条流上的三版：V⁻（新池新建文件）、V（第一次覆盖写，已确认）、R（第二次覆盖写，走读失败），(实例代号, txg)。
+const V_MINUS_OF_THE_F2_FLOWS: (u32, u64) = (1, 3);
+const V_OF_THE_F2_FLOWS: (u32, u64) = (1, 4);
+const R_OF_THE_F2_FLOWS: (u32, u64) = (1, 5);
+
+/// 看 journal 那一遍 oracle 判「落到更旧的根」的那一项（`Layer0StateJudgement::red_items` 里的写法）。
+fn consulted_oracle_item(kind: Layer0OracleViolationKind) -> String {
+    format!("{LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE}:{}", kind.name())
+}
+
+/// 看 journal 那一遍在状态 `ordinal` 上实际走的那条根是 `version`（恢复字只带 txg 低 32 位）、结局类别是 `class`。
+fn consulted_recovery_lands_on(
+    comparison: &Comparison,
+    ordinal: usize,
+    version: (u32, u64),
+    class: u32,
+) -> bool {
+    let words = comparison.cpu_recovery_words_of_each_state[ordinal];
+    (words[0], u64::from(words[1]), words[2]) == (version.0, version.1, class)
+}
+
+/// F2 设计里那一退（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 行「设计那一退照旧落 V」）：R 的根槽落了、走读失败，
+/// 系统配置没见证 R、见证到 V，CPU 恢复退到 V 本身、读出第二份内容，oracle 判「落到更旧的根」（oracle 不认 F2 那一退，判决第三节第 4 条不改）。
+/// GPU 恢复内核照 `recovery::version_witnessed_by_the_system_configuration_to_fall_back_to` 的 F2_EXACT 那一形退（`crash_judge_recovery.wgsl` 的
+/// 同名函数），每个状态逐位相同、落到同一条根。没有它的 GPU 在这几个状态上报读不出（第 33 批 T 合入之后对拍红的那 52 条流同一个病根）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_whose_newest_root_does_not_read_back_falls_back_to_the_version_the_system_configuration_witnessed(
+) {
+    let test_name = "the_gpu_judge_agrees_with_the_cpu_that_a_recovery_whose_newest_root_does_not_read_back_falls_back_to_the_version_the_system_configuration_witnessed";
+    let flow = record_two_overwrites_whose_second_tree_table_does_not_read_back(
+        "crash-judge-gpu-f2-fall-back-to-the-witnessed-version",
+        "two-overwrites-second-tree-table-unreadable-falls-back-to-the-witnessed-version",
+    );
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
+        return;
+    };
+    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
+        return;
+    };
+    let recovered_to_an_older_root =
+        consulted_oracle_item(Layer0OracleViolationKind::RecoveredToAnOlderRoot);
+    let states_falling_back_to_v = (0..comparison.cpu_red_items_of_each_state.len())
+        .filter(|ordinal| {
+            comparison.cpu_red_items_of_each_state[*ordinal].contains(&recovered_to_an_older_root)
+                && consulted_recovery_lands_on(
+                    &comparison,
+                    *ordinal,
+                    V_OF_THE_F2_FLOWS,
+                    RECOVERY_WORD_CLASS_FILE_READ,
+                )
+        })
+        .count();
+    eprintln!(
+        "CRASH_JUDGE_GPU_F2 flow={} states={} states_falling_back_to_v={states_falling_back_to_v}",
+        flow.name, comparison.states
+    );
+    assert!(
+        states_falling_back_to_v > 0,
+        "流 {}：R 的根槽落了而系统配置没见证 R 的状态上，CPU 看 journal 那一遍退到 V {V_OF_THE_F2_FLOWS:?}、读出第二份内容",
+        flow.name
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
+/// 攻方 A 形（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 行，攻方 `research/prompts/m3-batch33t-code-r1-opus-output.md`
+/// 第一节 1）：同上一条流，V 的根槽写自证校验和字段翻一位（V 的根槽读不出），R 走读失败、系统配置见证到 V 的状态上，F2_EXACT 只退到 V 本身、
+/// V 的根读不出就不退——CPU 报读不出（落到的根照旧是 R），一个状态都不退到 V⁻；GPU 恢复内核同判，每个状态逐位相同。
+/// 退到「比所选根旧、不超过见证上限的最新可读根」（第 33 批 T 的写法）会落到 V⁻、把已确认的 V 越过去，这条红。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_past_the_witnessed_version_whose_root_does_not_read_back(
+) {
+    let test_name = "the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_past_the_witnessed_version_whose_root_does_not_read_back";
+    let mut flow = record_two_overwrites_whose_second_tree_table_does_not_read_back(
+        "crash-judge-gpu-f2-witnessed-root-unreadable",
+        "two-overwrites-second-tree-table-unreadable-and-the-witnessed-root-unreadable",
+    );
+    let root_of_v = root_write_of_txg(&flow, V_OF_THE_F2_FLOWS.1);
+    let WrittenContents::Bytes(root_bytes) = &mut flow.writes[root_of_v].contents else {
+        panic!("根槽写是普通写");
+    };
+    root_bytes[ROOT_RECORD_CHECKSUM_FIELD_OFFSET] ^= 0x01;
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
+        return;
+    };
+    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
+        return;
+    };
+    let read_failed = consulted_oracle_item(Layer0OracleViolationKind::ReadFailed);
+    let states_reporting_r_unreadable = (0..comparison.cpu_red_items_of_each_state.len())
+        .filter(|ordinal| {
+            comparison.cpu_red_items_of_each_state[*ordinal].contains(&read_failed)
+                && comparison.cpu_consulted_reported_root_of_each_state[*ordinal]
+                    == Some(R_OF_THE_F2_FLOWS)
+                && consulted_recovery_lands_on(
+                    &comparison,
+                    *ordinal,
+                    R_OF_THE_F2_FLOWS,
+                    RECOVERY_WORD_CLASS_FAILED,
+                )
+        })
+        .count();
+    let states_falling_back_past_v = (0..comparison.cpu_red_items_of_each_state.len())
+        .filter(|ordinal| {
+            comparison.cpu_consulted_reported_root_of_each_state[*ordinal]
+                == Some(V_MINUS_OF_THE_F2_FLOWS)
+                && comparison.cpu_red_items_of_each_state[*ordinal].contains(
+                    &consulted_oracle_item(Layer0OracleViolationKind::RecoveredToAnOlderRoot),
+                )
+        })
+        .count();
+    eprintln!(
+        "CRASH_JUDGE_GPU_F2 flow={} states={} states_reporting_r_unreadable={states_reporting_r_unreadable} states_falling_back_past_v={states_falling_back_past_v}",
+        flow.name, comparison.states
+    );
+    assert!(
+        states_reporting_r_unreadable > 0,
+        "流 {}：R 走读失败、V 的根读不出的状态上 CPU 看 journal 那一遍报读不出、报的根是 R {R_OF_THE_F2_FLOWS:?}",
+        flow.name
+    );
+    assert_eq!(
+        states_falling_back_past_v, 0,
+        "流 {}：一个状态都不退到 V⁻ {V_MINUS_OF_THE_F2_FLOWS:?}（F2 只退到见证到的那一版本身）",
+        flow.name
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
+/// extent 树的种类码（`singlefs_core::records::TREE_KIND_EXTENT`）。
+const TREE_KIND_OF_THE_EXTENT_TREE: u16 = 1;
+
+/// H-S1-fold 的 A 臂（判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S1 行与第三节第 1 条；攻方
+/// `research/prompts/m3-batch33s-code-r1-opus-output.md` 第一节 1.2 `A-newer-version-pointer-garbled`，录成流）：删除中间态的流上
+/// 意图那一版（txg 5，extent 树照抄 txg 4 的）树表里 extent 树根指针第 0 条位置条目的整单元校验和异或几位（单元本身不动），重封那张树表、沿引用链补到根槽：
+/// 意图那一版读 extent 树根时第 0 条那一份对不上、读回第 1 条那一份（I-3.5 的折叠表把那个落点改记成它自己）。
+fn record_deletion_whose_intent_version_reads_the_shared_extent_root_from_its_second_copy(
+    tag: &str,
+) -> RecordedFlow {
+    const TXG_OF_THE_INTENT: u64 = 5;
+    let name = "deletion-intent-version-reads-the-shared-extent-root-from-its-second-copy";
+    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
+    let root_of_the_intent = root_write_of_txg(&flow, TXG_OF_THE_INTENT);
+    let tree_table_copies =
+        copies_of_the_tree_table_named_by_the_root_write(&flow, root_of_the_intent);
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &tree_table_copies, &|bytes| {
+        let checksum = only_tree_table_entry_of_kind(bytes, TREE_KIND_OF_THE_EXTENT_TREE)
+            + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET
+            + POINTER_LOCATION_CHECKSUM_OFFSETS[0];
+        for byte in &mut bytes[checksum..checksum + 4] {
+            *byte ^= 0x5a;
+        }
+    });
+    flow
+}
+
+/// H-S1-fold A 臂：CPU 池级 checker 判 I-3.5 时按全部版本走完之后的折叠表取每一版的单元（攻方 F1），txg 4 那一版先走、经第 0 条读回的
+/// 那个落点在意图那一版读回第 1 条之后同算一个单元，意图那一版不算新引用它，I-3.5 一个状态都不红；第 0 条那一份照判 I-2.1 红。GPU 走读内核
+/// 本来就按最终的落点表认，每个状态逐位相同（CPU 每版走完当场折时多红 I-3.5，随根环次序翻面）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_an_intent_version_reading_the_shared_extent_root_from_its_second_copy_does_not_redden_the_appearance_rule(
+) {
+    let flow =
+        record_deletion_whose_intent_version_reads_the_shared_extent_root_from_its_second_copy(
+            "crash-judge-gpu-i35-intent-reads-the-shared-extent-root-from-its-second-copy",
+        );
+    let Some(cards) = gpu_card_pool_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_that_an_intent_version_reading_the_shared_extent_root_from_its_second_copy_does_not_redden_the_appearance_rule",
+    ) else {
+        return;
+    };
+    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
+        return;
+    };
+    assert!(
+        comparison.cpu_checker_red_invariants.contains("I-2.1")
+            && !comparison.cpu_checker_red_invariants.contains("I-3.5"),
+        "流 {} 上 CPU 红 I-2.1、一个状态都不红 I-3.5（红过的：{:?}）",
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
+/// 三个数据单元那条流（txg 4）下段叶第一条记录的数据指针：第 0 条位置条目的槽写成 `slot`、整单元校验和写成 `checksum`（盘不动），
+/// 重封那片叶、沿引用链补到根。
+fn redirect_the_first_location_of_the_first_lower_leaf_data_pointer(
+    flow: &mut RecordedFlow,
+    name: &str,
+    slot: u64,
+    checksum: u32,
+) {
+    let extent_lower_leaf = |bytes: &[u8]| {
+        unit_class(bytes) == 2
+            && usize::from(index_node_key_width(bytes)) == EXTENT_KEY_BYTES
+            && index_node_entry_width(bytes) == EXTENT_LOWER_LEAF_RECORD_BYTES
+    };
+    let leaf_copies = last_unit_copies(flow, name, &extent_lower_leaf, false);
+    corrupt_units_and_reseal_up_to_the_root(flow, &leaf_copies, &|bytes| {
+        let location = index_node_entries_start(EXTENT_KEY_BYTES)
+            + EXTENT_KEY_BYTES
+            + POINTER_LOCATION_ENTRY_OFFSETS[0];
+        bytes[location + LOCATION_ENTRY_SLOT_OFFSET
+            ..location + LOCATION_ENTRY_SLOT_OFFSET + LOCATION_ENTRY_SLOT_BYTES]
+            .copy_from_slice(&slot.to_le_bytes()[..LOCATION_ENTRY_SLOT_BYTES]);
+        bytes[location + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET
+            ..location + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET + 4]
+            .copy_from_slice(&checksum.to_le_bytes());
+    });
+}
+
+/// 两块盘的基镜像在槽 `slot` 写进 `bytes`（流里谁都没写过它，每个状态都在）。
+fn write_into_the_base_on_both_devices(flow: &mut RecordedFlow, slot: u64, bytes: &[u8]) {
+    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+        flow.base.devices.get_mut(&device).expect("盘在池里").write(
+            singlefs_core::address::DeviceOffsetInBytes(slot * SLOT_BYTES),
+            bytes,
+        );
+    }
+}
+
+/// 拼读那三形下一槽（[`SLOT_NOTHING_IN_THE_FLOW_WROTE`] + 1）在基镜像里写的 16 KiB。
+fn bytes_of_the_slot_after_the_gap() -> Vec<u8> {
+    vec![0x5au8; usize::try_from(SLOT_BYTES).expect("16384")]
+}
+
+/// 「16 KiB 全零接下一槽那 16 KiB」那 32 KiB 的 CRC-32C：空段最后一槽按 32 KiB 读，CPU 的稀疏读者读出的就是它。
+fn checksum_of_zeros_followed_by_the_slot_after_the_gap() -> u32 {
+    let mut thirty_two_kibibytes = vec![0u8; usize::try_from(SLOT_BYTES).expect("16384")];
+    thirty_two_kibibytes.extend_from_slice(&bytes_of_the_slot_after_the_gap());
+    crc32c(&thirty_two_kibibytes)
+}
+
+/// H-S2-tail（判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S2 行、第三节第 1 条；攻方 `research/prompts/m3-batch33s-code-r1-opus-output.md`
+/// 第二节 2.2 `S2-crc-of-zeros-then-next-slot`，录成流）：三个数据单元的流上，基镜像在 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`] + 1 写进 16 KiB 非零字节，
+/// txg 4 那一版下段叶第一条数据指针第 0 条位置条目改指空段右端那一槽 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`]、校验和写成「16 KiB 全零接下一槽」的 CRC。
+fn record_three_unit_file_whose_first_data_pointer_names_the_last_slot_of_a_gap(
+    tag: &str,
+) -> RecordedFlow {
+    let name = "three-unit-file-first-data-pointer-naming-the-last-slot-of-a-gap";
+    let mut flow = record_new_pool_through_a_three_unit_file(tag, name);
+    assert_nothing_in_the_flow_wrote_the_slot(&flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
+    write_into_the_base_on_both_devices(
+        &mut flow,
+        SLOT_NOTHING_IN_THE_FLOW_WROTE + 1,
+        &bytes_of_the_slot_after_the_gap(),
+    );
+    redirect_the_first_location_of_the_first_lower_leaf_data_pointer(
+        &mut flow,
+        name,
+        SLOT_NOTHING_IN_THE_FLOW_WROTE,
+        checksum_of_zeros_followed_by_the_slot_after_the_gap(),
+    );
+    flow
+}
+
+/// GPU 内核 read_at 拼读那一读的整段 CRC 对得上位置条目时记的判不了原因（`crash_judge_common.wgsl` 的 `UNDECIDABLE_SPLICED_READ_MATCHES`）。
+const GPU_UNDECIDABLE_SPLICED_READ_MATCHES: u32 = 7;
+
+/// H-S2-tail：CPU 的稀疏读者从空段右端那一槽读 32 KiB，读出「16 KiB 全零 + 下一槽」、CRC 对得上，第 0 条那一份当数据单元读、往下判它的头。
+/// GPU 读者 `read_at` 拼出整段 CRC（全零 16 KiB 那一条的版本表第 8 字异或下一个落点那一版前 16 KiB 的 CRC），对得上之后往下要读的字节跨过落点末尾、
+/// 表里没有，池级 checker 与恢复内核都记判不了（[`GPU_UNDECIDABLE_SPLICED_READ_MATCHES`]），照流水线那样回落 CPU 之后每个状态的红项相同；
+/// 判了的状态逐位相同。拼读之前 GPU 守卫拒这一读、判第 0 条读不出，多红 I-2.1。
+#[test]
+fn the_gpu_judge_leaves_to_the_cpu_a_data_pointer_whose_first_location_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot(
+) {
+    assert_the_gpu_judge_leaves_to_the_cpu_only_states_undecidable_for(
+        "the_gpu_judge_leaves_to_the_cpu_a_data_pointer_whose_first_location_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot",
+        &record_three_unit_file_whose_first_data_pointer_names_the_last_slot_of_a_gap(
+            "crash-judge-gpu-s2-tail-data-pointer-last-slot-of-a-gap",
+        ),
+        &[GPU_UNDECIDABLE_SPLICED_READ_MATCHES],
+        &[GPU_UNDECIDABLE_SPLICED_READ_MATCHES],
+    );
+}
+
+/// 正推「16 KiB 节点加下一槽」那一形（`research/prompts/m3-batch33s-code-r1-sonnet-output.md` K1-S2 末段，录成流）：三个数据单元的流上，基镜像在
+/// [`SLOT_NOTHING_IN_THE_FLOW_WROTE`] 写进 txg 4 那片下段叶（码 2、16 KiB）的原样字节、下一槽写进 16 KiB 非零字节，下段叶第一条数据指针第 0 条位置条目
+/// 改指那一槽、校验和写成「节点 16 KiB 接下一槽」那 32 KiB 的 CRC。
+fn record_three_unit_file_whose_first_data_pointer_names_a_node_followed_by_a_written_slot(
+    tag: &str,
+) -> RecordedFlow {
+    let name = "three-unit-file-first-data-pointer-naming-a-node-followed-by-a-written-slot";
+    let mut flow = record_new_pool_through_a_three_unit_file(tag, name);
+    assert_nothing_in_the_flow_wrote_the_slot(&flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
+    let extent_lower_leaf = |bytes: &[u8]| {
+        unit_class(bytes) == 2
+            && usize::from(index_node_key_width(bytes)) == EXTENT_KEY_BYTES
+            && index_node_entry_width(bytes) == EXTENT_LOWER_LEAF_RECORD_BYTES
+    };
+    let node = flow.writes[last_unit_copies(&flow, name, &extent_lower_leaf, false)[0]]
+        .bytes()
+        .expect("单元写带字节")
+        .to_vec();
+    assert_eq!(
+        u64::try_from(node.len()).expect("单元长装得进 u64"),
+        SLOT_BYTES,
+        "码 2 节点 16 KiB"
+    );
+    write_into_the_base_on_both_devices(&mut flow, SLOT_NOTHING_IN_THE_FLOW_WROTE, &node);
+    write_into_the_base_on_both_devices(
+        &mut flow,
+        SLOT_NOTHING_IN_THE_FLOW_WROTE + 1,
+        &bytes_of_the_slot_after_the_gap(),
+    );
+    let mut node_followed_by_the_next_slot = node;
+    node_followed_by_the_next_slot.extend_from_slice(&bytes_of_the_slot_after_the_gap());
+    redirect_the_first_location_of_the_first_lower_leaf_data_pointer(
+        &mut flow,
+        name,
+        SLOT_NOTHING_IN_THE_FLOW_WROTE,
+        crc32c(&node_followed_by_the_next_slot),
+    );
+    flow
+}
+
+/// 「16 KiB 节点加下一槽」：CPU 从那一槽读 32 KiB，CRC 对得上，第 0 条那一份当数据单元读（头是码 2，红）。GPU 不用拼：单元槽落点本来就是 32 KiB 长
+/// （`crash_judge_tables` 的落点表，盘尾那一槽才 16 KiB），版本字节的后 16 KiB 就是基镜像里的下一槽，整段 CRC 照表比、往下照读，每个状态逐位相同，
+/// 一个状态都不记判不了。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_data_pointer_names_a_node_with_the_checksum_of_the_node_and_the_next_slot(
+) {
+    let flow =
+        record_three_unit_file_whose_first_data_pointer_names_a_node_followed_by_a_written_slot(
+            "crash-judge-gpu-s2-node-followed-by-a-written-slot",
+        );
+    let Some(cards) = gpu_card_pool_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_data_pointer_names_a_node_with_the_checksum_of_the_node_and_the_next_slot",
+    ) else {
+        return;
+    };
+    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
+        return;
+    };
+    assert!(
+        comparison.cpu_checker_red > 0,
+        "流 {}：第 0 条那一份当数据单元读、头是码 2，CPU 池级 checker 要有红",
+        flow.name
+    );
+    assert!(
+        comparison.mismatches.is_empty(),
+        "流 {}：{} 个状态 GPU 与 CPU 不一致或判不了（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
+    );
+}
+
+/// 正推「空槽加下一已写槽前 16 KiB」那一形（同上 K1-S2 末段；第 32 批攻方表里的 `zero16+next`，录成流）：删除中间态的流上墓碑树根第一条条目类型段 1 → 7、
+/// 子指针第 1 条位置条目指 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`]、整单元校验和写成「16 KiB 全零接下一槽」的 CRC，下一槽在两块盘的基镜像里写进 16 KiB 非零字节。
+fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_naming_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot(
+    tag: &str,
+) -> RecordedFlow {
+    let name =
+        "deletion-tombstone-root-entry-type-seven-second-location-naming-the-last-slot-of-a-gap";
+    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
+    assert_nothing_in_the_flow_wrote_the_slot(&flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
+    write_into_the_base_on_both_devices(
+        &mut flow,
+        SLOT_NOTHING_IN_THE_FLOW_WROTE + 1,
+        &bytes_of_the_slot_after_the_gap(),
+    );
+    let checksum = checksum_of_zeros_followed_by_the_slot_after_the_gap();
+    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
+        set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE);
+        set_the_slot_and_checksum_of_a_location_in_the_first_entry(
+            bytes,
+            POINTER_LOCATION_ENTRY_OFFSETS[1],
+            SLOT_NOTHING_IN_THE_FLOW_WROTE,
+            checksum,
+        );
+    });
+    flow
+}
+
+/// 「空槽加下一已写槽前 16 KiB」：CPU 头里类标签 0 不登记，按 32 KiB 读出「16 KiB 零 + 下一槽」、对得上，那一份 I-2.1 成立，红 I-9.23。
+/// GPU F-α 那一处比 32 KiB 时拿 `read_at` 拼出的整段 CRC 比（只比 CRC、不往下读，不记判不了），每个状态逐位相同（拼读之前那一读读不到、
+/// 多红 I-2.1、I-4.8、I-7.4）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot(
+) {
+    let flow = record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_naming_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot(
+        "crash-judge-gpu-s2-tombstone-entry-type-seven-last-slot-of-a-gap",
+    );
+    let Some(cards) = gpu_card_pool_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot",
+    ) else {
+        return;
+    };
+    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
+        return;
+    };
+    assert!(
+        comparison.cpu_checker_red_invariants.contains("I-9.23")
+            && !comparison.cpu_checker_red_invariants.contains("I-2.1"),
+        "流 {} 上 CPU 红 I-9.23、一个状态都不红 I-2.1（红过的：{:?}）",
+        flow.name,
+        comparison.cpu_checker_red_invariants
+    );
+    assert!(
+        comparison.mismatches.is_empty(),
+        "流 {}：{} 个状态 GPU 与 CPU 不一致或判不了（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
     );
 }
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -14910,3 +14910,287 @@
         "同一张镜像只差指着先走过的节点的是哪一条位置条目：红的同一批"
     );
 }
+
+// ─── 第 36 批（删除线）：I-3.5 按全部版本走完之后的折叠表取单元（H-S1-fold）、两份都读不出而第 0 条落点早被别的版本走过 ───
+
+/// 一版的树表单元的槽（第一版两盘同槽）。
+fn tree_table_slot_of_the_version(
+    version: &singlefs_core::transaction::TransactionOutput,
+) -> singlefs_core::address::SlotNumber {
+    common_pointer_head_corruption::the_slot_of(&version.root.tree_table)
+}
+
+/// 那一版的树表单元里种类 `kind` 那条条目的根指针在单元里的起点（整条 86 字节逐字找，恰好一处）；那条根指针全零时 `None`。
+fn root_pointer_start_in_the_tree_table_of(
+    image: &MemoryPool,
+    version: &singlefs_core::transaction::TransactionOutput,
+    kind: u16,
+) -> Option<usize> {
+    let tree_table = common_pointer_head_corruption::unit_bytes_at(
+        image,
+        tree_table_slot_of_the_version(version),
+        common_pointer_head_corruption::node_bytes(),
+    );
+    let root_pointer = singlefs_core::unit::parse_index_node(&tree_table)
+        .expect("树表解得开")
+        .entries
+        .iter()
+        .filter_map(|entry| singlefs_core::records::TreeTableEntry::parse(entry))
+        .find(|entry| entry.kind == kind)
+        .expect("树表里有这种树")
+        .root;
+    let pointer_bytes = common_pointer_head_corruption::node_pointer_bytes(&root_pointer);
+    if pointer_bytes.iter().all(|byte| *byte == 0) {
+        return None;
+    }
+    let starts: Vec<usize> = (0..=tree_table.len() - pointer_bytes.len())
+        .filter(|start| tree_table[*start..*start + pointer_bytes.len()] == pointer_bytes[..])
+        .collect();
+    let [start] = starts.as_slice() else {
+        panic!("种类 {kind} 的根指针在树表里恰好一处：{starts:?}")
+    };
+    Some(*start)
+}
+
+/// 指针里第 0 条位置条目的起点、每条位置条目的宽、条目里整单元校验和的偏移（D19（块指针的结构与宽度预算） 已定项 7 的字段表）。
+const NODE_POINTER_FIRST_LOCATION_OFFSET: usize = 50;
+const NODE_POINTER_LOCATION_BYTES: usize = 14;
+const NODE_POINTER_LOCATION_CHECKSUM_OFFSET: usize = 10;
+
+/// 那一版树表里种类 `kind` 那条根指针第 `location_indexes` 这几条位置条目的整单元校验和各异或几位（指着的单元本身不动），重封那张树表、
+/// 沿引用链补到根槽：这一版读那棵树的根时那几份对不上（I-2.1 红）、读回别的那一份。
+fn garble_location_checksums_of_a_root_pointer_in_the_tree_table_of(
+    image: &mut MemoryPool,
+    version: &singlefs_core::transaction::TransactionOutput,
+    kind: u16,
+    location_indexes: &[usize],
+) {
+    let start = root_pointer_start_in_the_tree_table_of(image, version, kind)
+        .expect("这一版那棵树的根指针不是全零");
+    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
+        image,
+        tree_table_slot_of_the_version(version),
+        common_pointer_head_corruption::node_bytes(),
+        |tree_table| {
+            for location_index in location_indexes {
+                let checksum = start
+                    + NODE_POINTER_FIRST_LOCATION_OFFSET
+                    + location_index * NODE_POINTER_LOCATION_BYTES
+                    + NODE_POINTER_LOCATION_CHECKSUM_OFFSET;
+                for byte in &mut tree_table[checksum..checksum + 4] {
+                    *byte ^= 0x5a;
+                }
+            }
+        },
+    );
+}
+
+/// 三个单元的文件之后发意图、删第一批：交回镜像与顺序写（txg 4）、意图（txg 5，extent 树照抄 txg 4 的、树表另写一份）两版。
+fn image_with_the_written_and_the_intent_versions_after_the_first_batch(
+    tag: &str,
+) -> (
+    MemoryPool,
+    singlefs_core::transaction::TransactionOutput,
+    singlefs_core::transaction::TransactionOutput,
+) {
+    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(tag);
+    let written = pool.output.clone();
+    let intent = common_deletion_in_progress::publish_the_intent(&mut pool);
+    common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool);
+    (pool.memory_pool(), written, intent)
+}
+
+fn verdict_of_the_invariant(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
+    check_pool_image(image)
+        .into_iter()
+        .find(|(identifier, _)| *identifier == invariant)
+        .map(|(_, verdict)| verdict)
+        .expect("清单里有")
+}
+
+/// H-S1-fold 的 A 臂（判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S1 行与第三节第 1 条，攻方
+/// `research/prompts/m3-batch33s-code-r1-opus-output.md` 第一节 1.2）：意图那一版（txg 5，较新、根环次序在 txg 4 之后走）树表里 extent 树根指针
+/// 第 `location_index` 条位置条目的校验和坏，读回另一份。池级 checker 判 I-3.5 时按全部版本走完之后的折叠表取每版的单元（攻方 F1）：
+/// txg 4 那一版先走、经第 0 条读回的那个落点，在意图那一版读回第 1 条那一份、把它改记成自己之后同算一个单元，意图那一版不算新引用它。
+/// 两种盘序（坏的是第 0 / 1 条）I-3.5 都成立、I-2.1 都红（改之前每版走完当场折，坏的是第 0 条时多红 I-3.5，与 GPU 走读内核的取法分叉）。
+#[test]
+fn an_intent_version_reading_the_shared_extent_root_from_either_copy_leaves_the_appearance_rule_holding(
+) {
+    let (clean, _written, intent) =
+        image_with_the_written_and_the_intent_versions_after_the_first_batch(
+            "known-bad-i35-intent-reads-the-shared-extent-root-from-either-copy",
+        );
+    for location_index in 0..2 {
+        let mut image = clean.clone();
+        garble_location_checksums_of_a_root_pointer_in_the_tree_table_of(
+            &mut image,
+            &intent,
+            singlefs_core::records::TREE_KIND_EXTENT,
+            &[location_index],
+        );
+        let verdicts = check_pool_image(&image);
+        let set = violated_invariant_set(&verdicts);
+        assert!(
+            set.contains("I-2.1"),
+            "坏的是第 {location_index} 条：那一份照判 I-2.1 红：{verdicts:?}"
+        );
+        assert_eq!(
+            verdict_of_the_invariant(&image, REFERENCE_INTERVALS_ARE_EXACT),
+            InvariantVerdict::Holds,
+            "坏的是第 {location_index} 条：I-3.5 按全部版本走完之后的折叠表取单元，成立：{verdicts:?}"
+        );
+    }
+}
+
+/// 攻方 S1 扫描（同上第一节 1.3）的一种历史：`versions` 是按发布次序的各版（新池新建文件那一版也算），每一对相邻、树表各写一份的版本里，
+/// 较新那一版树表里根指针与较旧那一版同种类的根指针逐字相同的每一棵树，各造一份坏镜像：较新那一版那条根指针第 0 条位置条目的校验和坏。
+/// 每一份 I-3.5 都不红。交回造了几份。
+fn assert_garbling_the_first_location_of_each_shared_root_pointer_of_the_newer_version_leaves_the_appearance_rule_unviolated(
+    clean: &MemoryPool,
+    versions: &[singlefs_core::transaction::TransactionOutput],
+) -> usize {
+    let mut images_judged = 0;
+    for pair in versions.windows(2) {
+        let (older, newer) = (&pair[0], &pair[1]);
+        if tree_table_slot_of_the_version(older) == tree_table_slot_of_the_version(newer) {
+            continue;
+        }
+        for entry in &newer.tree_table_entries {
+            if root_pointer_start_in_the_tree_table_of(clean, newer, entry.kind).is_none() {
+                continue;
+            }
+            let pointer_bytes = common_pointer_head_corruption::node_pointer_bytes(&entry.root);
+            let shared_with_the_older_version =
+                older.tree_table_entries.iter().any(|older_entry| {
+                    older_entry.kind == entry.kind
+                        && common_pointer_head_corruption::node_pointer_bytes(&older_entry.root)
+                            == pointer_bytes
+                });
+            if !shared_with_the_older_version {
+                continue;
+            }
+            let mut image = clean.clone();
+            garble_location_checksums_of_a_root_pointer_in_the_tree_table_of(
+                &mut image,
+                newer,
+                entry.kind,
+                &[0],
+            );
+            let verdict = verdict_of_the_invariant(&image, REFERENCE_INTERVALS_ARE_EXACT);
+            assert!(
+                !matches!(verdict, InvariantVerdict::Violated(_)),
+                "txg {} → {}、种类 {} 的根指针第 0 条校验和坏：I-3.5 不红（{verdict:?}）",
+                older.root.checkpoint_txg.0,
+                newer.root.checkpoint_txg.0,
+                entry.kind
+            );
+            images_judged += 1;
+        }
+    }
+    images_judged
+}
+
+/// 攻方 S1 扫描 `write3-intent-batch`：三个单元的文件、意图、第一批（3 份坏镜像：txg 4 → 5 的 extent 树，5 → 6 的墓碑树与 `deleted_inodes`）。
+#[test]
+fn garbling_each_shared_root_pointer_of_a_newer_version_through_an_intent_and_a_batch_leaves_the_appearance_rule_unviolated(
+) {
+    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(
+        "known-bad-i35-scan-write-intent-batch",
+    );
+    let mut versions = vec![pool.output.clone()];
+    versions.push(common_deletion_in_progress::publish_the_intent(&mut pool));
+    versions.push(common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool));
+    assert_eq!(
+        assert_garbling_the_first_location_of_each_shared_root_pointer_of_the_newer_version_leaves_the_appearance_rule_unviolated(
+            &pool.memory_pool(),
+            &versions,
+        ),
+        3,
+        "攻方这一种历史扫的格数"
+    );
+}
+
+/// 攻方 S1 扫描 `write3-unlink-to-the-end`：三个单元的文件、意图、按批大小 1 删到队列空（6 份坏镜像）。
+#[test]
+fn garbling_each_shared_root_pointer_of_a_newer_version_through_an_unlink_to_the_end_leaves_the_appearance_rule_unviolated(
+) {
+    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(
+        "known-bad-i35-scan-unlink-to-the-end",
+    );
+    let mut versions = vec![pool.output.clone()];
+    versions.push(common_deletion_in_progress::publish_the_intent(&mut pool));
+    while !pool.output.deleted_inodes.is_empty() {
+        versions
+            .push(common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool));
+    }
+    assert_eq!(
+        assert_garbling_the_first_location_of_each_shared_root_pointer_of_the_newer_version_leaves_the_appearance_rule_unviolated(
+            &pool.memory_pool(),
+            &versions,
+        ),
+        6,
+        "攻方这一种历史扫的格数"
+    );
+}
+
+/// 攻方 S1 扫描 `write3-new-inodes-x3`：三个单元的文件之后建三次空 inode（3 份坏镜像，都是 extent 树）。
+#[test]
+fn garbling_each_shared_root_pointer_of_a_newer_version_through_three_publishes_of_new_inodes_leaves_the_appearance_rule_unviolated(
+) {
+    let mut pool =
+        common_deletion_in_progress::pool_with_a_three_unit_file("known-bad-i35-scan-new-inodes");
+    let mut versions = vec![pool.output.clone()];
+    for _publish in 0..3 {
+        versions.push(
+            common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1),
+        );
+    }
+    assert_eq!(
+        assert_garbling_the_first_location_of_each_shared_root_pointer_of_the_newer_version_leaves_the_appearance_rule_unviolated(
+            &pool.memory_pool(),
+            &versions,
+        ),
+        3,
+        "攻方这一种历史扫的格数"
+    );
+}
+
+/// 正推 O1-1（`research/prompts/m3-batch33s-code-r1-sonnet-output.md` K1-S1 末段两处观察第一处；判决 `research/prompts/m3-batch33s-code-r1-main-verification.md`
+/// 第三节第 1 条「正推第 4 条那一形补坏镜像」）：顺序写那一版（txg 4，根环次序在最新的意图那一版之后走）树表里 extent 树根指针两条位置条目的校验和都坏，
+/// 第 0 条落点上那个单元（两版共享的 extent 树根）已被意图那一版先走过。「走过」按读回那一份的落点认：两份都读不出就没有读回的落点，
+/// 记走读失败、两份照判 I-2.1，不当成「别的版本先走过」（改之前读之前先按第 0 条落点查走过表，这一形判成先走过、并进前一版、两份都不判、不记走读失败）。
+#[test]
+fn a_root_pointer_whose_two_copies_do_not_read_back_and_whose_first_location_names_a_unit_walked_before_records_a_walk_failure(
+) {
+    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(
+        "known-bad-two-copies-unreadable-first-location-walked-before",
+    );
+    let written = pool.output.clone();
+    let intent = common_deletion_in_progress::publish_the_intent(&mut pool);
+    let shared_extent_root = |version: &singlefs_core::transaction::TransactionOutput| {
+        version
+            .tree_table_entries
+            .iter()
+            .find(|entry| entry.kind == singlefs_core::records::TREE_KIND_EXTENT)
+            .expect("树表里有 extent 树")
+            .root
+    };
+    assert_eq!(
+        common_pointer_head_corruption::node_pointer_bytes(&shared_extent_root(&written)),
+        common_pointer_head_corruption::node_pointer_bytes(&shared_extent_root(&intent)),
+        "意图那一版 extent 树照抄顺序写那一版的"
+    );
+    let mut image = pool.memory_pool();
+    garble_location_checksums_of_a_root_pointer_in_the_tree_table_of(
+        &mut image,
+        &written,
+        singlefs_core::records::TREE_KIND_EXTENT,
+        &[0, 1],
+    );
+    let verdicts = check_pool_image(&image);
+    let set = violated_invariant_set(&verdicts);
+    assert!(
+        set.contains("I-2.1") && set.contains("I-4.8"),
+        "两份照判 I-2.1 红，走读失败记下、遍历读不出红 I-4.8：{verdicts:?}"
+    );
+}
diff --git a/crates/singlefs-harness/tests/unlink_crash_image_remounted_twice_continues_to_identical_disks.rs b/crates/singlefs-harness/tests/unlink_crash_image_remounted_twice_continues_to_identical_disks.rs
new file mode 100644
--- /dev/null
+++ b/crates/singlefs-harness/tests/unlink_crash_image_remounted_twice_continues_to_identical_disks.rs
@@ -0,0 +1,137 @@
+//! unlink 的崩溃镜像各自重挂载续做，两份盘逐字节相同（D8（核心索引结构） 已定项 16 第 7 条 ①b）：删除意图刚发完那一刻的镜像、
+//! 第一批写到中间（单元与记录落了、根槽没落）的崩溃镜像，各自拷两份、各挂一次可写挂载续做到队列空，两份盘逐字节相同。
+//! 两批之间那一份干净镜像在 `unlink_mount_continuation_and_the_emptied_version.rs` 的
+//! `two_copies_of_the_image_between_two_batches_continue_to_byte_identical_disks`（第 34 批 U 交回：①b 只钉了那一形，判决
+//! `research/prompts/m3-batch33r-code-r1-main-verification.md` 第三节第 4 条，这两形补在这里）。三个单元的第一个文件、只供测试的批大小 1。
+
+mod common;
+mod common_deletion_in_progress;
+mod common_pointer_head_corruption;
+
+use common::{crash_state_devices, geometry, memory_pool_of_sparse_devices, parameters};
+use common_deletion_in_progress::{
+    batches_of_one_data_unit, pool_with_a_three_unit_file, publish_the_intent,
+    publish_the_next_step_in_batches_of_one,
+};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::container_tree::ContainerTreeNodeEntries;
+use singlefs_core::lineage_tree::LineageContainerCapacity;
+use singlefs_core::mount::{
+    mount_writable_with_test_only_switches_and_deletion_switches, ShadowLedger,
+};
+use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
+use singlefs_core::transaction::PoolVersion;
+use singlefs_core::unlink::DeletionSwitches;
+use singlefs_harness::memory_pool::{
+    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite,
+};
+use singlefs_harness::segments::StepKind;
+use singlefs_harness::SharedStream;
+
+fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
+    DeletionSwitches {
+        batch_size: batches_of_one_data_unit(),
+        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
+        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
+        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
+    }
+}
+
+/// 一份崩溃镜像（`base` 叠上 `writes` 里 `persisted` 的那几条）在自己的两块内存盘上可写挂载一次、按批大小 1 续做到队列空，交回挂完之后的整份盘。
+///
+/// # Panics
+/// 挂不上、或挂完那一版不是删空的一版（续做没走到队列空）。
+fn remount_a_copy_of_the_crash_image_and_continue_to_the_empty_queue(
+    base: &MemoryPool,
+    writes: &[RetainedWrite],
+    persisted: &[bool],
+) -> MemoryPool {
+    let stream = SharedStream::new();
+    let mut devices = crash_state_devices(base, writes, persisted, &stream);
+    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
+        &parameters(),
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        deletion_switches_with_batches_of_one(),
+    )
+    .expect("这一份崩溃镜像可写挂载、续做");
+    assert!(
+        matches!(mounted.current, PoolVersion::WithTreesAndNoFile(_)),
+        "续做走到队列空：挂完那一版是删空的一版"
+    );
+    memory_pool_of_sparse_devices(&devices)
+}
+
+/// 删除意图刚发完（一批都没删）那一刻的镜像：两份各自挂一次、续做三批（留 2、留 1、最后一批），两份盘逐字节相同。
+#[test]
+fn a_crash_image_right_after_the_unlink_intent_remounted_twice_continues_to_byte_identical_disks() {
+    let mut pool = pool_with_a_three_unit_file("unlink-intent-image-remounted-twice");
+    publish_the_intent(&mut pool);
+    let image_right_after_the_intent = pool.memory_pool();
+    let first_copy = remount_a_copy_of_the_crash_image_and_continue_to_the_empty_queue(
+        &image_right_after_the_intent,
+        &[],
+        &[],
+    );
+    let second_copy = remount_a_copy_of_the_crash_image_and_continue_to_the_empty_queue(
+        &image_right_after_the_intent,
+        &[],
+        &[],
+    );
+    assert!(
+        first_copy == second_copy,
+        "意图刚发完的镜像各自重挂载续做，两份盘逐字节相同（D8 已定项 16 第 7 条 ①b）"
+    );
+    assert!(first_copy != image_right_after_the_intent, "续做真的写了盘");
+}
+
+/// 第一批写到中间的崩溃镜像：意图之后那一批的单元写、journal 记录都落了，根槽与系统配置轮换没落（发布按单元、记录、根槽、系统配置的次序落盘，
+/// `transaction::persist_publish_writes`）。两份各自挂一次：恢复由那一批的记录施加出它（D23（journal 的角色与格式） 已定项 15），再续做剩下的两批，
+/// 两份盘逐字节相同。
+#[test]
+fn a_crash_image_with_the_first_batch_written_up_to_its_root_remounted_twice_continues_to_byte_identical_disks(
+) {
+    let mut pool = pool_with_a_three_unit_file("unlink-first-batch-crash-image-remounted-twice");
+    publish_the_intent(&mut pool);
+    let image_after_the_intent = pool.memory_pool();
+    let operations_before_the_first_batch = pool.retained_operations().len();
+    publish_the_next_step_in_batches_of_one(&mut pool);
+    let operations = pool.retained_operations();
+    let (writes_of_the_first_batch, _segments, _stream_indexes) =
+        writes_and_segments_with_stream_indexes(
+            &operations[operations_before_the_first_batch..],
+            &geometry(),
+        );
+    let root_write = writes_of_the_first_batch
+        .iter()
+        .position(|write| write.kind == StepKind::RootRecordFua)
+        .expect("第一批有一条根槽写");
+    assert!(
+        writes_of_the_first_batch[..root_write]
+            .iter()
+            .any(|write| write.kind == StepKind::JournalRecord)
+            && writes_of_the_first_batch[..root_write]
+                .iter()
+                .any(|write| write.kind == StepKind::UnitWrite),
+        "根槽写之前那一批的单元与记录都写了"
+    );
+    let persisted_up_to_the_root: Vec<bool> = (0..writes_of_the_first_batch.len())
+        .map(|write_index| write_index < root_write)
+        .collect();
+    let first_copy = remount_a_copy_of_the_crash_image_and_continue_to_the_empty_queue(
+        &image_after_the_intent,
+        &writes_of_the_first_batch,
+        &persisted_up_to_the_root,
+    );
+    let second_copy = remount_a_copy_of_the_crash_image_and_continue_to_the_empty_queue(
+        &image_after_the_intent,
+        &writes_of_the_first_batch,
+        &persisted_up_to_the_root,
+    );
+    assert!(
+        first_copy == second_copy,
+        "第一批写到中间的崩溃镜像各自重挂载续做，两份盘逐字节相同（D8 已定项 16 第 7 条 ①b）"
+    );
+    assert!(first_copy != image_after_the_intent, "续做真的写了盘");
+}
````

### m3-batch36/v：mutations-append.tsv

````
第36批 V H-S1-fold I-3.5 每版走完当场折（A 臂两种盘序）	crates/singlefs-checker/src/walk.rs	                    .placements_referenced_in_this_version\n                    .keys()\n                    .copied()\n                    .collect(),	                    .placements_referenced_in_this_version\n                    .keys()\n                    .map(|placement| self.unit_of_each_placement[placement])\n                    .collect(),	-p singlefs-harness --test checker_known_bad_images -- an_intent_version_reading_the_shared_extent_root_from_either_copy_leaves_the_appearance_rule_holding	an_intent_version_reading_the_shared_extent_root_from_either_copy_leaves_the_appearance_rule_holding
第36批 V H-S1-fold I-3.5 每版走完当场折（扫意图与一批）	crates/singlefs-checker/src/walk.rs	                    .placements_referenced_in_this_version\n                    .keys()\n                    .copied()\n                    .collect(),	                    .placements_referenced_in_this_version\n                    .keys()\n                    .map(|placement| self.unit_of_each_placement[placement])\n                    .collect(),	-p singlefs-harness --test checker_known_bad_images -- garbling_each_shared_root_pointer_of_a_newer_version_through_an_intent_and_a_batch_leaves_the_appearance_rule_unviolated	garbling_each_shared_root_pointer_of_a_newer_version_through_an_intent_and_a_batch_leaves_the_appearance_rule_unviolated
第36批 V H-S1-fold I-3.5 每版走完当场折（扫删到底）	crates/singlefs-checker/src/walk.rs	                    .placements_referenced_in_this_version\n                    .keys()\n                    .copied()\n                    .collect(),	                    .placements_referenced_in_this_version\n                    .keys()\n                    .map(|placement| self.unit_of_each_placement[placement])\n                    .collect(),	-p singlefs-harness --test checker_known_bad_images -- garbling_each_shared_root_pointer_of_a_newer_version_through_an_unlink_to_the_end_leaves_the_appearance_rule_unviolated	garbling_each_shared_root_pointer_of_a_newer_version_through_an_unlink_to_the_end_leaves_the_appearance_rule_unviolated
第36批 V H-S1-fold I-3.5 每版走完当场折（扫建三次空 inode）	crates/singlefs-checker/src/walk.rs	                    .placements_referenced_in_this_version\n                    .keys()\n                    .copied()\n                    .collect(),	                    .placements_referenced_in_this_version\n                    .keys()\n                    .map(|placement| self.unit_of_each_placement[placement])\n                    .collect(),	-p singlefs-harness --test checker_known_bad_images -- garbling_each_shared_root_pointer_of_a_newer_version_through_three_publishes_of_new_inodes_leaves_the_appearance_rule_unviolated	garbling_each_shared_root_pointer_of_a_newer_version_through_three_publishes_of_new_inodes_leaves_the_appearance_rule_unviolated
第36批 V H-S1-fold I-3.5 每版走完当场折（对拍 A 臂流）	crates/singlefs-checker/src/walk.rs	                    .placements_referenced_in_this_version\n                    .keys()\n                    .copied()\n                    .collect(),	                    .placements_referenced_in_this_version\n                    .keys()\n                    .map(|placement| self.unit_of_each_placement[placement])\n                    .collect(),	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_an_intent_version_reading_the_shared_extent_root_from_its_second_copy_does_not_redden_the_appearance_rule	the_gpu_judge_agrees_with_the_cpu_that_an_intent_version_reading_the_shared_extent_root_from_its_second_copy_does_not_redden_the_appearance_rule
第36批 V O1-1 两份都读不出时按第 0 条落点认先走过	crates/singlefs-checker/src/walk.rs	        self.note_references_of_a_pointer_followed_by_the_walk(&pointer, 1, what);	        let first_placement = (pointer.locations[0].device, pointer.locations[0].slot);\n        if self.visited_units.contains(&first_placement) {\n            self.merge_placements_below_a_unit_walked_by_an_earlier_version(first_placement);\n            return IndexNodeReading::WalkedByAnEarlierVersion { placement_it_was_read_from: first_placement };\n        }\n        self.note_references_of_a_pointer_followed_by_the_walk(&pointer, 1, what);	-p singlefs-harness --test checker_known_bad_images -- a_root_pointer_whose_two_copies_do_not_read_back_and_whose_first_location_names_a_unit_walked_before_records_a_walk_failure	a_root_pointer_whose_two_copies_do_not_read_back_and_whose_first_location_names_a_unit_walked_before_records_a_walk_failure
第36批 V ①b 每批写入时间带进程内计数（意图之后的镜像）	crates/singlefs-core/src/transaction.rs	                write_time_seconds: record.write_time_seconds,	                write_time_seconds: record.write_time_seconds + { static UNLINK_STEPS_PUBLISHED_IN_THIS_PROCESS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0); UNLINK_STEPS_PUBLISHED_IN_THIS_PROCESS.fetch_add(1, std::sync::atomic::Ordering::Relaxed) },	-p singlefs-harness --test unlink_crash_image_remounted_twice_continues_to_identical_disks -- a_crash_image_right_after_the_unlink_intent_remounted_twice_continues_to_byte_identical_disks	a_crash_image_right_after_the_unlink_intent_remounted_twice_continues_to_byte_identical_disks
第36批 V ①b 每批写入时间带进程内计数（第一批写到中间的镜像）	crates/singlefs-core/src/transaction.rs	                write_time_seconds: record.write_time_seconds,	                write_time_seconds: record.write_time_seconds + { static UNLINK_STEPS_PUBLISHED_IN_THIS_PROCESS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0); UNLINK_STEPS_PUBLISHED_IN_THIS_PROCESS.fetch_add(1, std::sync::atomic::Ordering::Relaxed) },	-p singlefs-harness --test unlink_crash_image_remounted_twice_continues_to_identical_disks -- a_crash_image_with_the_first_batch_written_up_to_its_root_remounted_twice_continues_to_byte_identical_disks	a_crash_image_with_the_first_batch_written_up_to_its_root_remounted_twice_continues_to_byte_identical_disks
第36批 V GPU 恢复内核删掉 F2 那一支	crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl	        if (walk_round == 1u || walked.outcome != OUTCOME_FAILED || !consult) {	        if (true) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_recovery_whose_newest_root_does_not_read_back_falls_back_to_the_version_the_system_configuration_witnessed	the_gpu_judge_agrees_with_the_cpu_that_a_recovery_whose_newest_root_does_not_read_back_falls_back_to_the_version_the_system_configuration_witnessed
第36批 V GPU 恢复内核 F2 退到见证上限之下第一条读得出的根	crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl	        if (root.instance == witnessed_record_instance && eq64(root.txg, witnessed_record_txg)	        if ((lt64(root.txg, witnessed_record_txg) || (eq64(root.txg, witnessed_record_txg) && root.instance <= witnessed_record_instance))	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_past_the_witnessed_version_whose_root_does_not_read_back	the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_past_the_witnessed_version_whose_root_does_not_read_back
第36批 V read_at 空段最后一槽不拼读（数据指针那一形）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	        return select(NONE, ZERO_SIXTEEN_KIBIBYTES_FOLLOWED_BY_THE_NEXT_SLOT, slots_left_in_the_gap == 1u && slots_read == 2u && whole_data_unit_fits);	        return NONE;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_leaves_to_the_cpu_a_data_pointer_whose_first_location_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot	the_gpu_judge_leaves_to_the_cpu_a_data_pointer_whose_first_location_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot
第36批 V read_at 空段最后一槽不拼读（F-α 那一形）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	        return select(NONE, ZERO_SIXTEEN_KIBIBYTES_FOLLOWED_BY_THE_NEXT_SLOT, slots_left_in_the_gap == 1u && slots_read == 2u && whole_data_unit_fits);	        return NONE;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot
第36批 V 拼读 CRC 对得上照往下读不记判不了	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	    if (s.c1 == checksum && (s.c2 & SLICE_SPLICED_ACROSS_THE_END_OF_A_LOCATION) != 0u) {	    if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_leaves_to_the_cpu_a_data_pointer_whose_first_location_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot	the_gpu_judge_leaves_to_the_cpu_a_data_pointer_whose_first_location_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot
第36批 V F-α 拼读 CRC 对得上记判不了	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        || slice_whole_read_crc_matches(read_at(device, slot_to_bytes(slot), DATA_UNIT_BYTES), checksum);	        || slice_crc_matches(read_at(device, slot_to_bytes(slot), DATA_UNIT_BYTES), DATA_UNIT_BYTES, checksum);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot
第36批 V 单元槽落点一律 16 KiB（节点加下一槽那一形）	crates/singlefs-checker-tier/src/crash_judge_tables.rs	            let length = if offset + DATA_UNIT_BYTES <= device_bytes {	            let length = if false {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_data_pointer_names_a_node_with_the_checksum_of_the_node_and_the_next_slot	the_gpu_judge_agrees_with_the_cpu_when_a_data_pointer_names_a_node_with_the_checksum_of_the_node_and_the_next_slot
第36批 V 版本表第 8 字不异或全零 16 KiB 的 CRC	crates/singlefs-checker-tier/src/crash_judge_tables.rs	            crc32_castagnoli(&followed_by_zeros) ^ crc32_castagnoli(&vec![0u8; node_bytes])	            crc32_castagnoli(&followed_by_zeros)	-p singlefs-checker-tier --lib -- crash_judge_tables::tests::the_shifted_crc_of_a_version_xored_with_the_crc_of_the_next_sixteen_kibibytes_is_the_crc_of_the_thirty_two_kibibytes	the_shifted_crc_of_a_version_xored_with_the_crc_of_the_next_sixteen_kibibytes_is_the_crc_of_the_thirty_two_kibibytes
````
