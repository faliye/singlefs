import sys
p = sys.argv[1]
s = open(p).read()
def rep(old, new, count=1):
    global s
    n = s.count(old)
    assert n == count, (old[:80], n)
    s = s.replace(old, new)

# (a) F_eff: count abandoned roots' F when AF_ABANDONED_F=count
rep("""    let effective_rollback_floor = verified_system_configurations_of_this_pool
        .iter()
        .flat_map(|(_, slots)| slots.iter().map(|view| view.rollback_floor))
        .fold(highest_floor_on_the_newest_valid_roots, u64::max);""",
"""    // AF（abandoned-floor-r1 攻方副本）：AF_ABANDONED_F=count 时被抛弃根带的 F 也算进 F_生效（第 ② 行「算进」）。
    let af_highest_floor_on_abandoned_roots = if std::env::var("AF_ABANDONED_F").unwrap_or_default() == "count" {
        roots
            .iter()
            .filter(|(_, _, root)| abandoned_by_the_newest_roots_table(root))
            .map(|(_, _, root)| root.rollback_floor)
            .max()
            .unwrap_or(0)
    } else {
        0
    };
    let effective_rollback_floor = verified_system_configurations_of_this_pool
        .iter()
        .flat_map(|(_, slots)| slots.iter().map(|view| view.rollback_floor))
        .fold(highest_floor_on_the_newest_valid_roots.max(af_highest_floor_on_abandoned_roots), u64::max);""")

# (b) 甲
rep("""            let below_floor = if row43_experiment == "walk_nearest_root_below_floor" {""",
"""            let below_floor = if row43_experiment == "walk_nearest_root_below_floor"
                || std::env::var("AF_CANDIDATE").unwrap_or_default() == "jia"
            {""")

# (c) 丁：I-3.1 的「实际遍历」并上额外的槽
rep("""    if accounting_seen {
        for device in &devices {
            let quarantined_exempted = quarantined_exempted_against_every_walked_version""",
"""    // AF（abandoned-floor-r1 攻方副本）：丁的几种读法，只加到 I-3.1 的「遍历」一边。
    let af_candidate = std::env::var("AF_CANDIDATE").unwrap_or_default();
    let mut af_extra_slots_per_device: BTreeMap<u32, u64> = BTreeMap::new();
    if af_candidate.starts_with("ding") {
        let walked_slots: BTreeSet<(u32, u64)> = walk
            .references
            .keys()
            .flat_map(|(device, start, span)| (*start..*start + *span).map(move |slot| (*device, slot)))
            .collect();
        let references_of_root = |root: &crate::RootView| -> BTreeSet<(u32, u64)> {
            let mut single = Walk::starting_with(
                reader,
                Judgements::default(),
                filesystem_identifier_low,
                root.instance,
                root.checkpoint_txg,
            );
            single.walk_root(&root.record_bytes, false);
            single
                .references
                .keys()
                .flat_map(|(device, start, span)| (*start..*start + *span).map(move |slot| (*device, slot)))
                .collect()
        };
        // 最新根分配记录树里的记录：(设备, 槽) → (已释放, 释放代或分配代)。
        let mut record_of_slot: BTreeMap<(u32, u64), (bool, u64)> = BTreeMap::new();
        let newest_view = &roots[newest_index].2;
        let tree_table_pointer = parse_node_pointer(&newest_view.record_bytes[36..122]);
        let mut af_cache = IndexNodeCache::new();
        if let Some(tree_table) = read_index_node_without_judging(reader, &tree_table_pointer, &mut af_cache) {
            for entry in &tree_table.entries {
                if entry.len() < tree_table_entry_bytes() || read_u16(entry, 10) != TREE_KIND_ALLOCATION {
                    continue;
                }
                let allocation_root = parse_node_pointer(&entry[14..100]);
                if allocation_root.all_zero {
                    continue;
                }
                if let Some((_, records)) = allocation_record_tree_without_judging(reader, &allocation_root, &mut af_cache) {
                    for record in &records {
                        for slot in record.slot..record.slot + record.span_slots {
                            record_of_slot.insert((record.device, slot), (record.is_released, record.generation));
                        }
                    }
                }
            }
        }
        let mut extra: BTreeSet<(u32, u64)> = BTreeSet::new();
        match af_candidate.as_str() {
            // 丁1：并上影子账隔离集（被抛弃根引用 − 候选根引用 − 当前账里仍分配的）；ding_isolated_released 只取其中账里是已释放记录的。
            "ding_isolated" | "ding_isolated_released" => {
                for (_, _, root) in roots.iter().filter(|(_, _, root)| abandoned_by_the_newest_roots_table(root)) {
                    for key in references_of_root(root) {
                        if walked_slots.contains(&key) {
                            continue;
                        }
                        let unreleased_in_current = matches!(record_of_slot.get(&key), Some((false, _)));
                        if unreleased_in_current {
                            continue;
                        }
                        if af_candidate == "ding_isolated_released" && !matches!(record_of_slot.get(&key), Some((true, _))) {
                            continue;
                        }
                        extra.insert(key);
                    }
                }
            }
            // 丁2：并上 F 之下（没被抛弃）的根引用、仍在 defer 里（已释放、释放代 > F_生效）的记录的槽。
            "ding_defer_below_floor" => {
                for (_, _, root) in roots.iter().filter(|(_, _, root)| {
                    !abandoned_by_the_newest_roots_table(root) && root.checkpoint_txg < effective_rollback_floor
                }) {
                    for key in references_of_root(root) {
                        if walked_slots.contains(&key) {
                            continue;
                        }
                        if let Some((true, generation)) = record_of_slot.get(&key) {
                            if *generation > effective_rollback_floor {
                                extra.insert(key);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        for (device, _) in &extra {
            *af_extra_slots_per_device.entry(*device).or_insert(0) += 1;
        }
        if std::env::var_os("AF_DUMP").is_some() {
            eprintln!("AF-DUMP ding={af_candidate} extra_slots={:?}", af_extra_slots_per_device);
        }
    }
    if accounting_seen {
        for device in &devices {
            let quarantined_exempted = quarantined_exempted_against_every_walked_version""")

rep("""            let walked = slots_referenced_by_every_walked_version
                .get(device)
                .copied()
                .unwrap_or(0)
                * SLOT_BYTES
                + quarantined_exempted;""",
"""            let walked = (slots_referenced_by_every_walked_version
                .get(device)
                .copied()
                .unwrap_or(0)
                + af_extra_slots_per_device.get(device).copied().unwrap_or(0))
                * SLOT_BYTES
                + quarantined_exempted;""")
open(p, 'w').write(s)
print("walk.rs patched")
