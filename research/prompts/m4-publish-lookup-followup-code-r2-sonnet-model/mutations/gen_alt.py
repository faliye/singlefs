#!/usr/bin/env python3
"""gen_alt.py：写出第二轮正推腿自己的「换一种同样坏的写法」变异表 alt-mutations.tsv（六段同 crates/mutations.tsv）。"""
H = "-p singlefs-harness --test publish_lookup_by_identity -- --exact "
T_REWRITE = "a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site"
T_SNAP = "a_whole_rewrite_under_a_snapshot_and_its_destruction_look_up_kept_mapping_entries_through_indexes_built_once"
T_DEAD = "rewrites_of_eight_and_of_thirty_two_data_units_under_a_snapshot_with_a_deadlist_of_several_leaves_build_one_index_per_pass_and_count_each_tree_apart"
T_RELOC = "relocation_batches_of_two_and_of_eight_units_build_their_indexes_once_and_look_up_each_planned_unit_twice"
CORE = "crates/singlefs-core/src/transaction.rs"
rows = []
def add(name, old, new, test):
    rows.append((name, CORE, old, new, H + test, test))

# ALT1：删快照放条目：进一处挪回每条条目一次，并且每条现建一份（每项建 + 每项进）
add("ALT1 删快照：每条条目进一次那一处并现建一份留存条目索引",
 """    let inside_the_destruction = (!entries_released.is_empty()).then(|| {
        lookup_work::entering(LookupSite::KeptMappingEntriesReleasedByTheSnapshotDestruction)
    });
    let mut kept_mapping_entries_by_key = None;
    for entry in entries_released {
        let locations = kept_mapping_entries_by_key
            .get_or_insert_with(|| previous_version.mapping_entries_kept_for_the_snapshots_by_key())
""",
 """    let inside_the_destruction: Option<lookup_work::InsideALookupSite> = None;
    let mut kept_mapping_entries_by_key = None;
    for entry in entries_released {
        let _per_entry = lookup_work::entering(LookupSite::KeptMappingEntriesReleasedByTheSnapshotDestruction);
        let locations = kept_mapping_entries_by_key
            .insert(previous_version.mapping_entries_kept_for_the_snapshots_by_key())
""", T_SNAP)

# ALT2：装要写的单元：每个重写的角色进一次那一处并现建一份 units 索引
add("ALT2 装要写的单元：每个角色进一次那一处并现建一份units索引",
 """    let inside_the_written_units =
        lookup_work::entering(LookupSite::UnitsOfTheRolesWrittenByThisPublish);
    let units_by_role = FirstEntryOfEachIdentity::of_the_table(&units, |unit| unit.identity);
    let written_units: Vec<&PublishedUnit> = relocated_copies
        .iter()
        .map(|(copy, _, _)| copy)
        .chain(rewritten.iter().map(|identity| {
            units_by_role
                .first_entry_of(identity)
                .expect("重写的角色都装了单元")
        }))
        .collect();
    drop(inside_the_written_units);
""",
 """    let units_by_role = FirstEntryOfEachIdentity::of_the_table(&units, |unit| unit.identity);
    let written_units: Vec<&PublishedUnit> = relocated_copies
        .iter()
        .map(|(copy, _, _)| copy)
        .chain(rewritten.iter().map(|identity| {
            let _per_role = lookup_work::entering(LookupSite::UnitsOfTheRolesWrittenByThisPublish);
            FirstEntryOfEachIdentity::of_the_table(&units, |unit| unit.identity)
                .first_entry_of(identity)
                .expect("重写的角色都装了单元")
        }))
        .collect();
""", T_REWRITE)

# ALT3：释放判定路径的留给快照清单：每个角色现建一份索引，那一处只进一次
add("ALT3 释放判定路径：留给快照的清单每个角色现建一份索引",
 """    roles
        .into_iter()
        .filter(|identity| {
            kept_for_the_snapshots_by_role
                .first_entry_of(identity)
                .is_none()
        })
        .collect()
}

/// 这个角色在上一版里有没有落点。""",
 """    let _ = &kept_for_the_snapshots_by_role;
    roles
        .into_iter()
        .filter(|identity| {
            FirstEntryOfEachIdentity::of_the_table(kept_for_the_snapshots, |role| *role)
                .first_entry_of(identity)
                .is_none()
        })
        .collect()
}

/// 这个角色在上一版里有没有落点。""", T_DEAD)

# ALT4：留给快照清单：索引照建照查，另外再多做一遍逐个 contains（计数看不见的线性）
add("ALT4 留给快照的清单：索引照用，另加一遍逐个contains（计数看不见）",
 """        .filter(|identity| {
            kept_for_the_snapshots_by_role
                .first_entry_of(identity)
                .is_none()
        })
        .collect()
}

/// 这个角色在上一版里有没有落点。""",
 """        .filter(|identity| {
            kept_for_the_snapshots_by_role
                .first_entry_of(identity)
                .is_none()
                && !kept_for_the_snapshots.contains(identity)
        })
        .collect()
}

/// 这个角色在上一版里有没有落点。""", T_DEAD)

# ALT5：映射位置换进 units：索引照用，另加一遍线性 position
add("ALT5 映射位置换进units：索引照用，另加一遍线性position（计数看不见）",
 """        if let Some(position) = position_in_the_units_of_each_role.get(&identity) {
            version.units[*position].slot = slot;
        }""",
 """        let _linear = version.units.iter().position(|unit| unit.identity == identity);
        if let Some(position) = position_in_the_units_of_each_role.get(&identity) {
            version.units[*position].slot = slot;
        }""", T_RELOC)

# ALT6/7/8：三棵树各自单独退回线性
for tree, label in [("CentralMapping", "中央映射树"), ("Deadlist", "deadlist树"), ("Accounting", "记账树")]:
    add(f"ALT-{tree} 照抄多层码2树节点：只有{label}退回在units上线性找",
     """                let carried = units_of_the_previous_version
                    .expect(
                        "照抄的节点只出现在接着上一版的发布里：上一版的单元索引与上一版一起交进来",
                    )
                    .unit(tree.role_of_node(*previous_position, &previous_tree.shape));""",
     f"""                let carried = if !matches!(tree, MultiLevelCodeTwoTree::{tree}) {{
                    units_of_the_previous_version
                        .expect("照抄的节点只出现在接着上一版的发布里：上一版的单元索引与上一版一起交进来")
                        .unit(tree.role_of_node(*previous_position, &previous_tree.shape))
                }} else {{
                    previous_version.unit(tree.role_of_node(*previous_position, &previous_tree.shape))
                }};""", T_DEAD if tree != "CentralMapping" else T_REWRITE)

# ALT12：映射位置换进units：一个有位置的都没有也照建索引、照进
add("ALT12 映射位置换进units：一个有位置的都没有也照建索引",
 """    if new_slot_of_each_mapped_role.is_empty() {
        return;
    }
""", "", T_RELOC)

# ALT14：记录点名的数据角色：每个角色进一次那一处并现建一份索引
add("ALT14 记录点名的数据角色：每个角色进一次那一处并现建一份索引",
 """    let _inside_the_named_data_roles =
        lookup_work::entering(LookupSite::DataRolesNamedBeforeTheLastRecord);
    let data_roles_named_before_the_last_record_by_role = FirstEntryOfEachIdentity::of_the_table(
        data_roles_named_before_the_last_record,
        |identity| *identity,
    );
    rewritten
        .iter()
        .copied()
        .filter(|identity| {
            data_roles_named_before_the_last_record_by_role
                .first_entry_of(identity)
                .is_none()
        })
        .collect()""",
 """    rewritten
        .iter()
        .copied()
        .filter(|identity| {
            let _per_role = lookup_work::entering(LookupSite::DataRolesNamedBeforeTheLastRecord);
            FirstEntryOfEachIdentity::of_the_table(data_roles_named_before_the_last_record, |role| *role)
                .first_entry_of(identity)
                .is_none()
        })
        .collect()""", T_DEAD)

with open("alt-mutations.tsv", "w", encoding="utf-8") as out:
    for name, f, old, new, args, test in rows:
        out.write("\t".join([name, f, old.replace("\n", "\\n"), new.replace("\n", "\\n"), args, test]) + "\n")
print(len(rows), "rows")
