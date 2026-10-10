#!/usr/bin/env python3
"""给攻方的仓副本加臂开关（环境变量选臂，不设 = 冻结副本原样）。用法：patch_arms.py <仓副本根>。
OPUS_C_NEWER_ROOTS：C 另并上环里比所选那一版新、按实例表有效、读得出的根自己的账（按它的 txg 对应的根环槽归组，解开时机同 C）。攻方自提，零轮。
OPUS_F1_AFTER_F2：可写挂载 F2 退到见证到的那一版之后，按退到的那一版当水位再判一次 F1。攻方自提，零轮。
OPUS_F2_NEWEST：F2 退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根。攻方自提，零轮。
OPUS_C_MAPPING（与 OPUS_C_NEWER_ROOTS 合用）：C 的账（记录拼的根、较新的根）读不出时照被抛弃根同一份办法（从中央映射现算、重读一次、看 B 的凭据、仍不成拒）。攻方自提，零轮。
每处锚点必须恰好命中一次，否则退出 3。"""
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
mount = root / "crates/singlefs-core/src/mount.rs"
text = mount.read_text()
anchor = """                named_by_ring_slot
                    .entry(ring_slot)
                    .or_default()
                    .extend(placements_in_the_account);
            }
            named_by_ring_slot
"""
arm = """                named_by_ring_slot
                    .entry(ring_slot)
                    .or_default()
                    .extend(placements_in_the_account);
            }
            if std::env::var_os("OPUS_C_NEWER_ROOTS").is_some() {
                for root in roots.iter().filter(|root| {
                    !is_abandoned(root)
                        && (root.checkpoint_txg, root.instance)
                            > (selected_version.checkpoint_txg, selected_version.instance)
                }) {
                    match placements_referenced_by_root(devices, unit_area_start, root) {
                        Some(placements) => named_by_ring_slot
                            .entry(target_for_publish(root.checkpoint_txg, slots_per_region))
                            .or_default()
                            .extend(placements.into_iter().map(|(device, slot, span)| {
                                PlacementOnDevice { device, slot, span }
                            })),
                        None if std::env::var_os("OPUS_C_MAPPING").is_some() => {
                            let placements = placements_of_an_abandoned_root_whose_account_is_unreadable(
                                devices,
                                unit_area_start,
                                root,
                                &current_records,
                                before_the_one_reread,
                            )?;
                            named_by_ring_slot
                                .entry(target_for_publish(root.checkpoint_txg, slots_per_region))
                                .or_default()
                                .extend(placements.into_iter().map(|(device, slot, span)| {
                                    PlacementOnDevice { device, slot, span }
                                }));
                        }
                        None => eprintln!("OPUS_ARM newer-root account unreadable txg={}", root.checkpoint_txg.0),
                    }
                }
            }
            if std::env::var_os("OPUS_C_MAPPING").is_some() {
                let mut first_readable_record_of_each_publish = BTreeMap::new();
                for record in records.values().filter(|record| {
                    (record.instance, record.checkpoint_txg)
                        > (selected_version.instance, selected_version.checkpoint_txg)
                }) {
                    first_readable_record_of_each_publish
                        .entry((record.instance, record.checkpoint_txg))
                        .or_insert(record);
                }
                for ((_instance, checkpoint_txg), record) in first_readable_record_of_each_publish {
                    let root_of_the_publish =
                        crate::recovery::root_assembled_from_the_new_root_segment_of_the_record(
                            record,
                            selected_version.filesystem_identifier,
                        );
                    if placements_referenced_by_root(devices, unit_area_start, &root_of_the_publish).is_none() {
                        let placements = placements_of_an_abandoned_root_whose_account_is_unreadable(
                            devices,
                            unit_area_start,
                            &root_of_the_publish,
                            &current_records,
                            before_the_one_reread,
                        )?;
                        named_by_ring_slot
                            .entry(target_for_publish(checkpoint_txg, slots_per_region))
                            .or_default()
                            .extend(placements.into_iter().map(|(device, slot, span)| {
                                PlacementOnDevice { device, slot, span }
                            }));
                    }
                }
            }
            named_by_ring_slot
"""
if text.count(anchor) != 1:
    sys.exit(f"锚点命中 {text.count(anchor)} 次")
text = text.replace(anchor, arm)
print("patched OPUS_C_NEWER_ROOTS")
# OPUS_F1_AFTER_F2：可写挂载 F2 退到见证到的那一版之后，按退到的那一版当水位再判一次 F1（攻方自提，零轮；只判一遍、不重读）。
anchor_f1 = """                    Ok(previous) => (
                        witnessed_version,
                        witnessed_version,
"""
arm_f1 = """                    Ok(_)
                        if std::env::var_os("OPUS_F1_AFTER_F2").is_some()
                            && !publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify(
                                &*devices,
                                &records,
                                &witnessed_version,
                            )
                            .is_empty() =>
                    {
                        let publish = publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify(
                            &*devices,
                            &records,
                            &witnessed_version,
                        )[0];
                        return Err(MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
                            StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify {
                                publish,
                            },
                        )));
                    }
                    Ok(previous) => (
                        witnessed_version,
                        witnessed_version,
"""
if text.count(anchor_f1) != 1:
    sys.exit(f"F1 锚点命中 {text.count(anchor_f1)} 次")
text = text.replace(anchor_f1, arm_f1)
print("patched OPUS_F1_AFTER_F2")
mount.write_text(text)
# OPUS_F2_NEWEST：F2 退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（实例表读不出时只认恰好相等的那条）。攻方自提，零轮。
recovery = root / "crates/singlefs-core/src/recovery.rs"
rtext = recovery.read_text()
anchor_f2 = """    let witnessed_publish = record_at_the_witnessed_counter?;
    let chosen = order_of_the_root_choice(chosen_root);
    readable_roots(
        reader,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .find(|root| {
        let order = order_of_the_root_choice(root);
        order == witnessed_publish && order < chosen
    })
}
"""
arm_f2 = """    let witnessed_publish = record_at_the_witnessed_counter?;
    let chosen = order_of_the_root_choice(chosen_root);
    let roots = readable_roots(
        reader,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    if std::env::var_os("OPUS_F2_NEWEST").is_some() {
        let chosen_table = instance_table_chain_of_root(reader, chosen_root).ok().map(|chain| chain.records);
        return roots
            .into_iter()
            .filter(|root| {
                let order = order_of_the_root_choice(root);
                order >= witnessed_publish
                    && order < chosen
                    && !chosen_table.as_ref().map_or(order != witnessed_publish, |table| {
                        root_is_abandoned_by_the_instance_table(root, table)
                    })
            })
            .max_by_key(order_of_the_root_choice);
    }
    roots.into_iter().find(|root| {
        let order = order_of_the_root_choice(root);
        order == witnessed_publish && order < chosen
    })
}
"""
if rtext.count(anchor_f2) != 1:
    sys.exit(f"F2 锚点命中 {rtext.count(anchor_f2)} 次")
recovery.write_text(rtext.replace(anchor_f2, arm_f2))
print("patched OPUS_F2_NEWEST")
