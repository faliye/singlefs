#!/usr/bin/env python3
"""alloc-basis-r3 Opus 攻方腿：把 crates 副本改成「按环境变量切臂 / 造坏镜像」的形态（只改草稿目录里的副本，不碰仓）。

每处替换旧串必须恰好命中一次，否则退出码 3 且一个字节不写（与 research/scripts/replace-once.py 同义）。
环境变量都不设时，改过的副本与原版行为逐字相同（run-probe.sh 用一段产物逐字节比对自证）。

  SFPROBE_T1=1                        甲-T1 / G7 的记账行：按「这条根持久之后的环与 F_生效」判可再分配（分配器那一半由探针在每次发布之前做）
  SFPROBE_CHECKER_FLOOR=effective     checker 候选集的 F 按恢复的定义现算（各盘所带 F 最大值的最小值），不取最新根自己带的 F
  SFPROBE_G8=1                        checker 多报两条：G8（从最新根的分配记录数「已释放 ∧ 释放代 > 这条根持久之后的回收门槛」== 第 5 项，逐盘）
                                      与 G8′（第 1 项 − 第 5 项 == 最新根走读得到的引用，逐盘；不用回收谓词，本腿提的，被攻过零轮）
  SFPROBE_BUG_RELEASE_CARRIED_INSTANCE_TABLE_AT=t  坏镜像：txg t 那次发布照抄实例表单元，却把它的落点也放进释放表（活单元记成已释放）
改可见性（不改行为）：mount.rs 的 isolate_slots_referenced_only_by_abandoned_roots 与 abandoned_by_table 改成 pub，探针要在崩在两条带新 F 的根之间的手工抬 F 里照原样调它们；
allocator.rs 加三个只读访问器（is_reclaimed、isolated_slot_numbers、held_slot_numbers）。
"""
import sys

COPY = sys.argv[1]
VISIBILITY_ONLY = len(sys.argv) > 2 and sys.argv[2] == "--visibility-only"
# 只改可见性与加只读访问器的那几处（不改行为）：用「--visibility-only」只打这几处，给对照那份副本用。
VISIBILITY_MARKERS = ("pub fn is_reclaimed", "pub fn isolated_slot_numbers", "pub fn abandoned_by_table", "pub fn isolate_slots_referenced_only_by_abandoned_roots", "pub fn read_instance_table_rows_for_probe")

PROBE_T1_FLOOR_FN = """/// PROBE 甲-T1 / G7：这条根持久之后的环（盖掉同一个槽里的旧根、加上这条）上的回收门槛 max(F_生效, 环里最旧有效根)。
fn probe_t1_floor<Device: BlockDevice>(pool: &PoolWriter<'_, Device>, plan: &PublishPlan<'_>, previous: Option<&TransactionOutput>) -> CheckpointTxg {
    let parameters = pool.parameters;
    let mut roots: Vec<(u64, u32, u64)> = crate::recovery::readable_roots(&*pool.devices, &parameters.region_devices, &parameters.geometry, &parameters.filesystem_identifier)
        .into_iter().map(|root| (root.checkpoint_txg.0, root.instance.0, root.rollback_floor.0)).collect();
    let mut rows: Vec<(u32, u64)> = previous
        .and_then(|version| version.units.iter().find(|unit| unit.identity == TransactionUnit::InstanceTable))
        .and_then(|unit| crate::instance_table::InstanceTableRecords::parse(&unit.bytes))
        .map(|table| table.rows.iter().map(|row| (row.instance.0, row.selected_root_txg.0)).collect())
        .unwrap_or_else(|| {
            previous
                .and_then(|version| crate::recovery::read_instance_table_rows_for_probe(&*pool.devices, &version.root))
                .unwrap_or_default()
        });
    let target = target_for_publish(plan.txg);
    roots.retain(|(txg, _, _)| target_for_publish(CheckpointTxg(*txg)) != target);
    roots.push((plan.txg.0, plan.instance.0, plan.rollback_floor.0));
    if let InstanceTablePlan::Rewrite(records) = &plan.instance_table {
        rows = records.iter().filter_map(|bytes| crate::instance_table::InstanceRow::parse(bytes)).map(|row| (row.instance.0, row.selected_root_txg.0)).collect();
    }
    let mut highest_per_device: BTreeMap<u32, u64> = BTreeMap::new();
    for (txg, _, floor) in &roots {
        let device = parameters.region_devices[usize::try_from(target_for_publish(CheckpointTxg(*txg)).region).expect("区域")].0;
        let highest = highest_per_device.entry(device).or_insert(*floor);
        *highest = (*highest).max(*floor);
    }
    let effective = highest_per_device.values().copied().min().unwrap_or(0);
    let oldest = roots.iter().filter(|(txg, instance, _)| !rows.iter().any(|(row_instance, row_txg)| row_instance == instance && txg > row_txg)).map(|(txg, _, _)| *txg).min();
    CheckpointTxg(oldest.map_or(effective, |oldest| effective.max(oldest)))
}

"""

READ_ROWS_FN = """/// PROBE：一条根指着的实例表里的 (实例, T)（探针在单实例进程里拿不到上一版的实例表单元时从盘上读；按位置条目的校验和取第一份对得上的）。
pub fn read_instance_table_rows_for_probe<Reader: PoolReader + ?Sized>(reader: &Reader, root: &RootRecord) -> Option<Vec<(u32, u64)>> {
    let data_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    for location in &root.instance_table.locations {
        let Some(bytes) = reader.read(location.device, location.slot.to_device_offset(), data_bytes) else { continue };
        if crc32_castagnoli(&bytes) != location.unit_checksum { continue; }
        return InstanceTableRecords::parse(&bytes).map(|table| table.rows.iter().map(|row| (row.instance.0, row.selected_root_txg.0)).collect());
    }
    None
}

"""

PATCHES = [
    # ---- 甲-T1 / G7 的记账行 ----
    (
        "crates/singlefs-core/src/transaction.rs",
        "            device_map.allocated_slots() * SLOT_BYTES,\n",
        "            (device_map.allocated_slots() - probe_pending) * SLOT_BYTES,\n",
    ),
    (
        "crates/singlefs-core/src/transaction.rs",
        "            device_map.deferred_slots() * SLOT_BYTES,\n",
        "            (device_map.deferred_slots() - probe_pending) * SLOT_BYTES,\n",
    ),
    (
        "crates/singlefs-core/src/transaction.rs",
        "            device_map.free_slots() * SLOT_BYTES,\n",
        "            (device_map.free_slots() + probe_pending) * SLOT_BYTES,\n",
    ),
    (
        "crates/singlefs-core/src/transaction.rs",
        "    for device_map in &allocator.devices {\n",
        "    for device_map in &allocator.devices {\n"
        "        let probe_pending: u64 = probe_floor_after.map_or(0, |floor| allocator.records().iter().filter(|record| record.device == device_map.device && record.is_released && record.generation <= floor && !allocator.is_reclaimed(record.device, record.slot)).map(|record| u64::from(record.span_slots)).sum());\n",
    ),
    (
        "crates/singlefs-core/src/transaction.rs",
        "    let mut accounting_entries = vec![\n",
        "    let probe_floor_after = std::env::var(\"SFPROBE_T1\").is_ok().then(|| probe_t1_floor(&*pool, plan, previous));\n"
        "    let mut accounting_entries = vec![\n",
    ),
    (
        "crates/singlefs-core/src/transaction.rs",
        "/// 上一版里某个角色的单元字节（照抄进这一版的 `units`）。\n",
        PROBE_T1_FLOOR_FN + "/// 上一版里某个角色的单元字节（照抄进这一版的 `units`）。\n",
    ),
    # ---- 坏镜像：活单元记成已释放 ----
    (
        "crates/singlefs-core/src/transaction.rs",
        "        None => format_time_tree_table_to_release(allocator, &rewritten),\n    };\n",
        "        None => format_time_tree_table_to_release(allocator, &rewritten),\n    };\n"
        "    let release = {\n"
        "        let mut release = release;\n"
        "        if let (Ok(at), Some(previous_version), InstanceTablePlan::Carry(_)) = (std::env::var(\"SFPROBE_BUG_RELEASE_CARRIED_INSTANCE_TABLE_AT\"), previous, &plan.instance_table) {\n"
        "            if at.parse::<u64>().ok() == Some(plan.txg.0) {\n"
        "                release.push(Placement { slot: previous_version.root.instance_table.locations[0].slot, span: 2 });\n"
        "            }\n"
        "        }\n"
        "        release\n"
        "    };\n",
    ),
    # ---- 访问器与可见性 ----
    (
        "crates/singlefs-core/src/allocator.rs",
        "    #[must_use]\n    pub fn open_segment(&self) -> Option<SlotNumber> {\n",
        "    pub fn is_reclaimed(&self, device: DeviceIdentity, slot: SlotNumber) -> bool {\n        self.reclaimed.contains(&(device, slot))\n    }\n"
        "    #[must_use]\n    pub fn open_segment(&self) -> Option<SlotNumber> {\n",
    ),
    (
        "crates/singlefs-core/src/allocator.rs",
        "    /// 影子账隔离的槽数（D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」的这块盘那一项）。\n",
        "    pub fn isolated_slot_numbers(&self) -> Vec<u64> {\n        self.isolated.iter().enumerate().filter(|(_, bit)| **bit).map(|(index, _)| UNIT_AREA_START_SLOT + index as u64).collect()\n    }\n"
        "    pub fn held_slot_numbers(&self) -> Vec<u64> {\n        self.held_until_floor_takes_effect.iter().enumerate().filter(|(_, bit)| **bit).map(|(index, _)| UNIT_AREA_START_SLOT + index as u64).collect()\n    }\n"
        "    /// 影子账隔离的槽数（D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」的这块盘那一项）。\n",
    ),
    (
        "crates/singlefs-core/src/mount.rs",
        "fn abandoned_by_table(root: &RootRecord, table: &InstanceTableRecords) -> bool {\n",
        "pub fn abandoned_by_table(root: &RootRecord, table: &InstanceTableRecords) -> bool {\n",
    ),
    (
        "crates/singlefs-core/src/mount.rs",
        "fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(\n",
        "pub fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(\n",
    ),
    (
        "crates/singlefs-core/src/recovery.rs",
        "/// 一条根指着的实例表（行与链指针）；单元读不出或解不开都是 `None`（读不出的由走读另报）。\n",
        READ_ROWS_FN + "/// 一条根指着的实例表（行与链指针）；单元读不出或解不开都是 `None`（读不出的由走读另报）。\n",
    ),
    # ---- checker：候选集的 F ----
    (
        "crates/singlefs-checker/src/walk.rs",
        "    let newest_rollback_floor = u64::from_le_bytes(\n        roots[newest_index].2.record_bytes[130..138]\n            .try_into()\n            .expect(\"8 字节\"),\n    );\n",
        "    let newest_rollback_floor = if std::env::var(\"SFPROBE_CHECKER_FLOOR\").as_deref() == Ok(\"effective\") {\n"
        "        let mut highest_per_device: BTreeMap<u32, u64> = BTreeMap::new();\n"
        "        for (region, _, root) in &roots {\n"
        "            let carried = u64::from_le_bytes(root.record_bytes[130..138].try_into().expect(\"8 字节\"));\n"
        "            let device = geometry.region_devices[usize::try_from(*region).expect(\"区域\")];\n"
        "            let highest = highest_per_device.entry(device).or_insert(carried);\n"
        "            *highest = (*highest).max(carried);\n"
        "        }\n"
        "        highest_per_device.values().copied().min().unwrap_or(0)\n"
        "    } else {\n"
        "        u64::from_le_bytes(\n        roots[newest_index].2.record_bytes[130..138]\n            .try_into()\n            .expect(\"8 字节\"),\n    )\n"
        "    };\n",
    ),
    # ---- checker：G8 与 G8′ ----
    (
        "crates/singlefs-checker/src/image.rs",
        "pub const IMPLEMENTED_INVARIANTS: [&str; 26] = [\n",
        "pub const IMPLEMENTED_INVARIANTS: [&str; 28] = [\n    \"G8\", \"G8p\",\n",
    ),
    (
        "crates/singlefs-checker/src/walk.rs",
        "    instance_table_rows: Vec<(u32, u64)>,\n}\n",
        "    instance_table_rows: Vec<(u32, u64)>,\n    probe_allocation_rows: Vec<Vec<u8>>,\n}\n",
    ),
    (
        "crates/singlefs-checker/src/walk.rs",
        "        instance_table_rows: Vec::new(),\n    };\n",
        "        instance_table_rows: Vec::new(),\n        probe_allocation_rows: Vec::new(),\n    };\n",
    ),
    (
        "crates/singlefs-checker/src/walk.rs",
        "            TREE_KIND_ACCOUNTING => {\n                self.accounting_seen = true;\n",
        "            TREE_KIND_ALLOCATION => {\n                self.probe_allocation_rows.extend(node.entries.iter().map(|row| row.to_vec()));\n            }\n"
        "            TREE_KIND_ACCOUNTING => {\n                self.accounting_seen = true;\n",
    ),
    (
        "crates/singlefs-checker/src/walk.rs",
        "    let newest_failures = walk.walk_failures.clone();\n",
        "    let newest_failures = walk.walk_failures.clone();\n"
        "    let probe_newest_references = walk.references.clone();\n"
        "    let probe_newest_allocation_rows = walk.probe_allocation_rows.clone();\n",
    ),
    (
        "crates/singlefs-checker/src/walk.rs",
        "    // I-3.1 / I-5.2：最新根下面记账树的「已分配」「空闲」逐盘对遍历得到的和与容量。\n",
        "    if std::env::var(\"SFPROBE_G8\").is_ok() && accounting_seen {\n"
        "        let probe_oldest_valid = roots.iter().filter(|(_, _, root)| !instance_table_rows.iter().any(|(row_instance, row_txg)| *row_instance == root.instance && root.checkpoint_txg > *row_txg)).map(|(_, _, root)| root.checkpoint_txg).min().unwrap_or(0);\n"
        "        let probe_threshold = newest_rollback_floor.max(probe_oldest_valid);\n"
        "        for device in &devices {\n"
        "            let deferred_stat = accounting.get(&(5, *device)).copied();\n"
        "            let counted: u64 = probe_newest_allocation_rows.iter().filter(|row| read_u32(row, 0) == *device && read_u16(row, 10) & 0x8000 != 0 && read_u64(row, 12) > probe_threshold).map(|row| u64::from(read_u16(row, 10) & 0x7fff) * SLOT_BYTES).sum();\n"
        "            judgements.judge(\"G8\", deferred_stat == Some(counted), || format!(\"盘 {device}：第 5 项 {deferred_stat:?}，分配记录里已释放 ∧ 释放代 > {probe_threshold} 的 {counted}\"));\n"
        "            let allocated_stat = accounting.get(&(STATISTIC_ALLOCATED_BYTES, *device)).copied();\n"
        "            let newest_walked: u64 = probe_newest_references.keys().filter(|(owner, _, _)| owner == device).map(|(_, _, span)| span * SLOT_BYTES).sum();\n"
        "            judgements.judge(\"G8p\", matches!((allocated_stat, deferred_stat), (Some(allocated), Some(deferred)) if allocated >= deferred && allocated - deferred == newest_walked), || format!(\"盘 {device}：第 1 项 {allocated_stat:?} − 第 5 项 {deferred_stat:?} ≠ 最新根走读 {newest_walked}\"));\n"
        "        }\n"
        "    }\n"
        "    // I-3.1 / I-5.2：最新根下面记账树的「已分配」「空闲」逐盘对遍历得到的和与容量。\n",
    ),
]

applied = 0
for relative, old, new in PATCHES:
    if VISIBILITY_ONLY and not any(marker in new for marker in VISIBILITY_MARKERS):
        continue
    applied += 1
    path = f"{COPY}/{relative}"
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    count = text.count(old)
    if count != 1:
        print(f"✗ {relative}：旧串命中 {count} 次，一个字节没写：{old[:60]!r}")
        print("→ 副本不是 run-probe.sh 记的那个版本，重拷再打补丁")
        sys.exit(3)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text.replace(old, new, 1))
print(f"patched {applied} of {len(PATCHES)} sites (visibility_only={VISIBILITY_ONLY})")
