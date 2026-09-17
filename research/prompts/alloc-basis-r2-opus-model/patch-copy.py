#!/usr/bin/env python3
"""把 crates 副本改成「按环境变量切臂 / 造坏镜像」的形态（只改草稿目录里的副本，不碰仓）。

每处替换旧串必须恰好命中一次，否则退出码 3 且一个字节不写（与 research/scripts/replace-once.py 同义）。
环境变量都不设时，改过的副本与原版行为逐字相同（run-probe.sh 用 S1/S3 的产物逐字节比对自证）。

  SFPROBE_LEAK_TXG=t        坏镜像 ①：txg t 的覆盖写不释放上一版的 extent 根（槽泄漏：不释放、记账不减）
  SFPROBE_DEFER_PLUS=1      坏镜像 ③：记账第 5 项（defer）在盘 0 上多报 1 槽
  SFPROBE_ARM=yiprime       乙′：写者记账第 1 项 = 仍分配（占着 − defer）；checker 的 I-3.1 = 「仍分配 == 最新根可达」∧「候选并集 − 最新根可达 ≤ defer」
  SFPROBE_I52=rewritten     I-5.2 改判「空闲 + 第 1 项 + defer == 单元区」（乙′ 要跟着改的那一条；不设时按今天的「空闲 + 第 1 项 == 单元区」判）
  SFPROBE_ARM=jiaa_weak / jiaa_exact  甲-a：I-3.1 = 「并集 ≤ 记账」∧「记账 − 并集 ≤ 上界」，上界取记账第 5 项（weak）或从最新根分配记录重数「已释放 ∧ 释放代 > max(F, 环里最旧有效根) ∧ 没有候选根引用」（exact）
  SFPROBE_T1=1              甲-T1：分配器每次发布前按此刻盘上的环与 F_生效 回收，记账行按这条根持久之后的环与 F_生效 算
  SFPROBE_CHECKER_FLOOR=effective  F-生 的 checker：候选集的 F 按恢复的定义现算（各盘所带 F 最大值的最小值），不取最新根自己带的 F
"""
import sys

COPY = sys.argv[1]


# 甲-T1（第一轮攻方腿的回收时点）：记账行按「这条根持久之后的环与 F_生效」判可再分配，分配器在每次发布之前按「此刻盘上的环与 F_生效」回收。
# F_生效 按恢复的定义（各盘所带 F 最大值的最小值，数全部可读根）现算，所以它同时就是 F-生 的回收时点。
PROBE_T1_FLOOR_FN = """/// PROBE 甲-T1：now = 此刻盘上的环；after = 这条根持久之后的环（盖掉同一个槽里的旧根、加上这条）。
fn probe_t1_floor<Device: BlockDevice>(pool: &PoolWriter<'_, Device>, plan: &PublishPlan<'_>, previous: Option<&TransactionOutput>, after: bool) -> CheckpointTxg {
    let parameters = pool.parameters;
    let mut roots: Vec<(u64, u32, u64)> = crate::recovery::readable_roots(&*pool.devices, &parameters.region_devices, &parameters.geometry, &parameters.filesystem_identifier)
        .into_iter().map(|root| (root.checkpoint_txg.0, root.instance.0, root.rollback_floor.0)).collect();
    let mut rows: Vec<(u32, u64)> = previous
        .and_then(|version| version.units.iter().find(|unit| unit.identity == TransactionUnit::InstanceTable))
        .and_then(|unit| crate::instance_table::InstanceTableRecords::parse(&unit.bytes))
        .map(|table| table.rows.iter().map(|row| (row.instance.0, row.selected_root_txg.0)).collect())
        .unwrap_or_default();
    if after {
        let target = target_for_publish(plan.txg);
        roots.retain(|(txg, _, _)| target_for_publish(CheckpointTxg(*txg)) != target);
        roots.push((plan.txg.0, plan.instance.0, plan.rollback_floor.0));
        if let InstanceTablePlan::Rewrite(records) = &plan.instance_table {
            rows = records.iter().filter_map(|bytes| crate::instance_table::InstanceRow::parse(bytes)).map(|row| (row.instance.0, row.selected_root_txg.0)).collect();
        }
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

PATCHES = [
    (
        "crates/singlefs-core/src/transaction.rs",
        "        None => format_time_tree_table_to_release(allocator, &rewritten),\n    };\n",
        "        None => format_time_tree_table_to_release(allocator, &rewritten),\n    };\n"
        "    // PROBE 坏镜像 ①\n"
        "    let release = {\n"
        "        let mut release = release;\n"
        "        if let Ok(leak) = std::env::var(\"SFPROBE_LEAK_TXG\") {\n"
        "            if leak.parse::<u64>().ok() == Some(plan.txg.0) && plan.file.is_some() && release.len() > 1 {\n"
        "                release.remove(1);\n"
        "            }\n"
        "        }\n"
        "        release\n"
        "    };\n",
    ),
    (
        "crates/singlefs-core/src/transaction.rs",
        "            device_map.allocated_slots() * SLOT_BYTES,\n",
        "            (device_map.allocated_slots() - probe_pending - if std::env::var(\"SFPROBE_ARM\").as_deref() == Ok(\"yiprime\") { device_map.deferred_slots() - probe_pending } else { 0 }) * SLOT_BYTES,\n",
    ),
    (
        "crates/singlefs-core/src/transaction.rs",
        "            device_map.deferred_slots() * SLOT_BYTES,\n",
        "            (device_map.deferred_slots() - probe_pending + u64::from(std::env::var(\"SFPROBE_DEFER_PLUS\").is_ok() && device_map.device.0 == 0)) * SLOT_BYTES,\n",
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
        "    let probe_floor_after = std::env::var(\"SFPROBE_T1\").is_ok().then(|| probe_t1_floor(&*pool, plan, previous, true));\n"
        "    let mut accounting_entries = vec![\n",
    ),
    (
        "crates/singlefs-core/src/transaction.rs",
        "    let allocator_before_this_publish = allocator.clone();\n",
        "    if std::env::var(\"SFPROBE_T1\").is_ok() {\n"
        "        let floor_now = probe_t1_floor(&*pool, &plan, previous, false);\n"
        "        allocator.reclaim_released_up_to(floor_now);\n"
        "    }\n"
        "    let allocator_before_this_publish = allocator.clone();\n",
    ),
    (
        "crates/singlefs-core/src/transaction.rs",
        "/// 上一版里某个角色的单元字节（照抄进这一版的 `units`）。\n",
        PROBE_T1_FLOOR_FN + "/// 上一版里某个角色的单元字节（照抄进这一版的 `units`）。\n",
    ),
    (
        "crates/singlefs-core/src/allocator.rs",
        "    #[must_use]\n    pub fn open_segment(&self) -> Option<SlotNumber> {\n",
        "    pub fn is_reclaimed(&self, device: DeviceIdentity, slot: SlotNumber) -> bool {\n        self.reclaimed.contains(&(device, slot))\n    }\n    #[must_use]\n    pub fn open_segment(&self) -> Option<SlotNumber> {\n",
    ),
    (
        "crates/singlefs-checker/src/walk.rs",
        "    let newest_failures = walk.walk_failures.clone();\n",
        "    let newest_failures = walk.walk_failures.clone();\n    let newest_references = walk.references.clone();\n",
    ),
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
    (
        "crates/singlefs-checker/src/walk.rs",
        "            judgements.judge(\"I-3.1\", allocated == Some(walked), || {\n",
        "            let deferred = accounting.get(&(5, *device)).copied().unwrap_or(0);\n"
        "            let newest_walked: u64 = newest_references.keys().filter(|(owner, _, _)| owner == device).map(|(_, _, span)| span * SLOT_BYTES).sum();\n"
        "            let arm_yiprime = std::env::var(\"SFPROBE_ARM\").as_deref() == Ok(\"yiprime\");\n"
        "            let i31_holds = if arm_yiprime { allocated == Some(newest_walked) && walked.saturating_sub(newest_walked) <= deferred } else { allocated == Some(walked) };\n"
        "            judgements.judge(\"I-3.1\", i31_holds, || {\n"
        "                if arm_yiprime { return format!(\"乙′ 盘 {device}：记账第 1 项 {allocated:?}，最新根可达 {newest_walked}；候选并集 {walked}，defer {deferred}\"); }\n",
    ),
    (
        "crates/singlefs-checker/src/walk.rs",
        "(Some(free), Some(allocated), Some(capacity)) if free + allocated == capacity), || {\n",
        "(Some(free), Some(allocated), Some(capacity)) if free + allocated + if std::env::var(\"SFPROBE_I52\").as_deref() == Ok(\"rewritten\") { deferred } else { 0 } == capacity), || {\n",
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
        "    let newest_references = walk.references.clone();\n",
        "    let newest_references = walk.references.clone();\n    let probe_allocation_rows = walk.probe_allocation_rows.clone();\n",
    ),
    (
        "crates/singlefs-checker/src/walk.rs",
        "            let i31_holds = if arm_yiprime {",
        "            let probe_arm = std::env::var(\"SFPROBE_ARM\").unwrap_or_default();\n"
        "            let probe_oldest_valid = roots.iter().filter(|(_, _, root)| !instance_table_rows.iter().any(|(row_instance, row_txg)| *row_instance == root.instance && root.checkpoint_txg > *row_txg)).map(|(_, _, root)| root.checkpoint_txg).min().unwrap_or(0);\n"
        "            let probe_floor = newest_rollback_floor.max(probe_oldest_valid);\n"
        "            let probe_exact_bound: u64 = probe_allocation_rows.iter().filter(|row| read_u32(row, 0) == *device).filter(|row| read_u16(row, 10) & 0x8000 != 0 && read_u64(row, 12) > probe_floor).filter(|row| { let start = crate::read_six_byte_unsigned(row, 4); let span = u64::from(read_u16(row, 10) & 0x7fff); !per_device.get(device).is_some_and(|ranges| ranges.iter().any(|(slot, width, _)| *slot < start + span && start < *slot + *width)) }).map(|row| u64::from(read_u16(row, 10) & 0x7fff) * SLOT_BYTES).sum();\n"
        "            let jiaa_bound = if probe_arm == \"jiaa_weak\" { Some(deferred) } else if probe_arm == \"jiaa_exact\" { Some(probe_exact_bound) } else { None };\n"
        "            let i31_holds = if let Some(bound) = jiaa_bound { allocated.is_some_and(|value| walked <= value && value - walked <= bound) } else if arm_yiprime {",
    ),
]

for relative, old, new in PATCHES:
    path = f"{COPY}/{relative}"
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    count = text.count(old)
    if count != 1:
        print(f"✗ {relative}：旧串命中 {count} 次，一个字节没写")
        print("→ 副本不是 run-probe.sh 记的那个版本，重拷再打补丁")
        sys.exit(3)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text.replace(old, new, 1))
print(f"patched {len(PATCHES)} sites")
