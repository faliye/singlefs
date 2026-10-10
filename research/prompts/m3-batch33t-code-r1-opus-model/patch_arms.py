"""攻方副本上的臂开关（只在环境变量设了时生效；不设时与冻结副本逐行为相同）：
OPUS_F2_OFF=1      F2 的退回判法恒交 None（三处都不退，即 F2 落地之前的行为）。
OPUS_F2_EXACT=1    F2 只退到见证计数器那条记录所属的那一版本身（根环里 (txg, 实例) 恰好相等的那条根）；那条记录读不出就不退。攻方提的收严，零轮。
OPUS_F1_OFF=1      F1 恒交空（不拒）。
OPUS_F1_ANY=1      F1 判「点名单元验不过」改成「某个点名单元两份都验不过」（一份验得过就算验得过）。攻方提的收严，零轮。
OPUS_C_ACCOUNT=1   C 另把水位之上每一次没施加的发布、用它任一条读得出的记录的新根段拼成的根的账（影子账同一份 placements_referenced_by_root）
                   里的落点并进隔离集合。攻方提的收严，零轮。
用法：python3 patch_arms.py <仓副本根>
"""
import sys, pathlib
root = pathlib.Path(sys.argv[1]) / "crates/singlefs-core/src"
def once(path, old, new):
    text = path.read_text()
    assert text.count(old) == 1, (path, old[:60], text.count(old))
    path.write_text(text.replace(old, new))
rec = root / "recovery.rs"
once(rec, "    if (selected_version.instance, selected_version.checkpoint_txg)\n        != (chosen_root.instance, chosen_root.checkpoint_txg)\n    {\n        return None;\n    }\n",
     "    if std::env::var_os(\"OPUS_F2_OFF\").is_some() {\n        return None;\n    }\n    if (selected_version.instance, selected_version.checkpoint_txg)\n        != (chosen_root.instance, chosen_root.checkpoint_txg)\n    {\n        return None;\n    }\n")
once(rec, "    let ceiling = record_at_the_witnessed_counter.unwrap_or(chosen);\n",
     "    if std::env::var_os(\"OPUS_F2_EXACT\").is_some() {\n        let exact = record_at_the_witnessed_counter?;\n        return readable_roots(\n            reader,\n            &system_configuration.immutable.region_devices,\n            &system_configuration.immutable.sizes,\n            &system_configuration.immutable.filesystem_identifier,\n        )\n        .into_iter()\n        .find(|root| order_of_the_root_choice(root) == exact && exact < chosen);\n    }\n    let ceiling = record_at_the_witnessed_counter.unwrap_or(chosen);\n")
mnt = root / "mount.rs"
once(mnt, "    let mut records_of_each_publish: BTreeMap<\n",
     "    if std::env::var_os(\"OPUS_F1_OFF\").is_some() {\n        return Vec::new();\n    }\n    let mut records_of_each_publish: BTreeMap<\n")
once(mnt, "    let mut named_by_ring_slot: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> = BTreeMap::new();\n",
     """    let mut named_by_ring_slot: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> = BTreeMap::new();
    if std::env::var_os("OPUS_C_ACCOUNT").is_some() {
        let mut seen: BTreeSet<(InstanceGeneration, CheckpointTxg)> = BTreeSet::new();
        for record in records.values().filter(|record| {
            (record.instance, record.checkpoint_txg)
                > (selected_version.instance, selected_version.checkpoint_txg)
        }) {
            if !seen.insert((record.instance, record.checkpoint_txg)) {
                continue;
            }
            let pseudo_root = RootRecord {
                filesystem_identifier: selected_version.filesystem_identifier,
                unmount_marker: record.new_unmount_marker,
                instance: record.instance,
                checkpoint_txg: record.checkpoint_txg,
                tree_table: record.new_tree_table,
                tree_identifier_watermark: record.new_tree_identifier_watermark,
                rollback_floor: record.new_rollback_floor,
                instance_table: record.new_instance_table,
                mapping_root: record.new_mapping_root,
                allocation_record_tree_root: record.new_allocation_record_tree_root,
            };
            if let Some(placements) = placements_referenced_by_root(devices, unit_area_start, &pseudo_root) {
                named_by_ring_slot
                    .entry(target_for_publish(record.checkpoint_txg, slots_per_region))
                    .or_default()
                    .extend(placements.into_iter().map(|(device, slot, span)| PlacementOnDevice { device, slot, span }));
            }
        }
    }
""")
once(rec, "/// 取前缀并施加（D23（journal 的角色与格式） 已定项 14 / 已定项 15）；返回扫描报告与施加之后的根。\n",
     """pub(crate) fn some_copy_of_every_named_unit_of_the_record_verifies<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    record: &JournalRecord,
) -> bool {
    record.named.iter().all(|named| {
        let Some(unit_bytes) = unit_bytes_for_class(named.unit_class) else {
            return false;
        };
        named.locations.iter().any(|location| {
            reader
                .read(location.device, location.slot.to_device_offset(), unit_bytes)
                .is_some_and(|bytes| crc32_castagnoli(&bytes) == location.unit_checksum)
        })
    })
}

/// 取前缀并施加（D23（journal 的角色与格式） 已定项 14 / 已定项 15）；返回扫描报告与施加之后的根。
""")
once(mnt, "                !crate::recovery::every_named_unit_of_the_record_verifies(reader, record)\n",
     "                if std::env::var_os(\"OPUS_F1_ANY\").is_some() {\n                    !crate::recovery::some_copy_of_every_named_unit_of_the_record_verifies(reader, record)\n                } else {\n                    !crate::recovery::every_named_unit_of_the_record_verifies(reader, record)\n                }\n")
print("patched")
