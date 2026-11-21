//! 快照下整份重写与删快照取留存映射条目、整理搬迁两个与八个单元一批时，索引各建一次、计划里每个单元查两次。

use super::*;

/// 池里有一个快照的样子：快照之后整份重写过的文件的旧单元进了 deadlist、映射条目留存。
struct PoolWithASnapshot {
    pool: PoolBuilt,
    snapshot: TreeIdentifier,
}

/// 第一个文件之外 `files` 个四单元的文件，建一个快照（经会话）。
fn pool_with_a_snapshot_of_files(files: u64) -> PoolWithASnapshot {
    let mut pool = pool_with_files_written_once(files, 4);
    let created = pool
        .session
        .publish_user_change(&mut pool.devices, UserChange::CreateSnapshot)
        .unwrap_or_else(|refusal| panic!("建快照：{refusal:?}"));
    drop(created);
    let snapshot = version_of_the_session(&pool.session)
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .expect("建了一个快照")
        .tree;
    PoolWithASnapshot { pool, snapshot }
}

/// 一个快照下整份重写一个文件（一次发布），再删这个快照（经会话）：
/// 重写那一次，快照判定那一圈每走一遍换下的用户可见角色各查一次映射 key，进 deadlist 的各经批量读者查一次映射条目（建三份索引：按角色的 key 索引两份、
/// 批量读者的单元索引一份；进 deadlist 的 key 走到的叶各解一次）；删快照那几批，放掉的每条条目按 key 经索引取一次它留存的映射条目。
/// 另判：删快照放条目的每一遍恰好建一份留存条目按 key 的索引（建索引次数等于进那一处的次数，
/// 查的次数是遍数乘放掉的条数）；重写那一次按 inode 号找文件等几处每走一遍建一份（`assert_the_lookups_of_one_publish_build_one_index_per_pass`）。
/// 判别力：`crates/mutations.tsv` 里点名这一条的几行（M2：进 deadlist 那一支退回线性找 key、逐 key 查；MA1）。
pub fn run_the_scenario_where_a_whole_rewrite_under_a_snapshot_and_its_destruction_look_up_kept_mapping_entries_through_indexes_built_once(
) {
    let PoolWithASnapshot { mut pool, snapshot } = pool_with_a_snapshot_of_files(8);
    let rewritten_inode = pool.inodes[2];
    let (previous, published, lookup_work) = whole_rewrite_published_once(
        &mut pool,
        rewritten_inode,
        rewritten_inode.0 + 3_000_017,
        4,
        crate::common::FIXED_WRITE_TIME_SECONDS + 7_000,
    );
    let killed_keys: Vec<&[u8]> = published
        .deadlist_entries
        .iter()
        .filter(|entry| {
            !previous
                .deadlist_entries
                .iter()
                .any(|before| before.mapping_key == entry.mapping_key)
        })
        .map(|entry| entry.mapping_key.as_slice())
        .collect();
    let killed = u64::try_from(killed_keys.len()).expect("条数");
    let routed_leaves =
        distinct_mapping_leaves_routed_to_by(&previous, killed_keys.iter().copied());
    let replaced_user_visible_roles = u64::try_from(
        previous
            .mapped_units
            .iter()
            .filter(|(role, _)| {
                published.rewritten.contains(role) && asks_the_snapshot_lifetime_judgement(*role)
            })
            .count(),
    )
    .expect("角色数");
    let snapshot_judgement = lookup_work.of_the_site(LookupSite::SnapshotJudgement);
    println!(
        "publish_lookup_by_identity rewrite_under_a_snapshot killed={killed} routed_leaves={routed_leaves} replaced_user_visible_roles={replaced_user_visible_roles} snapshot_judgement={snapshot_judgement:?}"
    );
    assert!(
        killed >= 4,
        "快照下整份重写：旧的四个数据单元都进了 deadlist（{killed} 条）"
    );
    assert_the_work_is_whole_passes_of(
        snapshot_judgement,
        (
            3,
            routed_leaves,
            replaced_user_visible_roles + killed + routed_leaves,
        ),
        &format!(
            "快照判定那一圈：走一遍建三份索引、进 deadlist 的 {killed} 把 key 走到的 {routed_leaves} 片叶各解一次；换下的 {replaced_user_visible_roles} 个用户可见角色各查一次 key，\
             进 deadlist 的各再查一次 key、每片叶查一次单元"
        ),
    );
    assert_the_lookups_of_one_publish_build_one_index_per_pass(
        &published,
        &lookup_work,
        4,
        "快照下整份重写一个四单元的文件",
    );

    let kept_before = version_of_the_session(&pool.session)
        .mapping_entries_kept_for_the_snapshots
        .len();
    let (destroyed, destruction_work) = lookup_work_while(|| {
        pool.session
            .publish_user_change(&mut pool.devices, UserChange::DestroySnapshot { snapshot })
    });
    destroyed.unwrap_or_else(|refusal| panic!("删快照：{refusal:?}"));
    assert_every_lookup_examines_at_most_a_binary_search(&destruction_work, "删快照");
    let kept_after = version_of_the_session(&pool.session)
        .mapping_entries_kept_for_the_snapshots
        .len();
    let released = u64::try_from(kept_before - kept_after).expect("条数");
    let kept_entries_released = destruction_work
        .of_the_site(LookupSite::KeptMappingEntriesReleasedByTheSnapshotDestruction);
    println!(
        "publish_lookup_by_identity snapshot_destruction released={released} kept_entries_released={kept_entries_released:?} files_of_the_snapshot_judgement={:?}",
        destruction_work.of_the_site(LookupSite::FilesOfTheUserVisibleRolesOfTheSnapshotJudgement)
    );
    // 八个四单元的文件、重写两个：留存的条目一批就放完（最后一批取全部），放条目的每一遍都放这同一批。
    let passes_that_release = assert_one_index_built_per_entry(
        kept_entries_released,
        "删快照：放条目的每一遍建一份留存条目按 key 的索引",
    );
    assert!(
        released >= 4 && kept_entries_released.index_lookups == passes_that_release * released,
        "删快照放掉 {released} 条留存的映射条目（之前 {kept_before} 条、之后 {kept_after} 条）：发布算到定点时放条目的每一遍（{passes_that_release} 遍），\
         放掉的每条按 key 经那一遍建一次的索引取一次：{kept_entries_released:?}"
    );
}

/// `version` 里第一个文件之外 `inode` 那个文件前 `data_units` 个数据单元的搬迁计划项：映射 key、映射位置上的源落点。
fn planned_relocations_of_the_data_units(
    version: &TransactionOutput,
    inode: InodeNumber,
    data_units: u64,
) -> Vec<PlannedRelocation> {
    (0..data_units)
        .map(|unit| {
            let role = TransactionUnit::DataOfAFileOtherThanTheFirst {
                inode,
                unit: DataUnitIndexInFile(unit),
            };
            let (_, mapping_key) = version
                .mapped_units
                .iter()
                .find(|(mapped, _)| *mapped == role)
                .expect("写过的数据单元进映射");
            let MappingLookup::Found(locations) =
                central_mapping_locations_in_the_version(version, mapping_key)
            else {
                panic!("{role:?} 在这一版的映射里查得到");
            };
            PlannedRelocation {
                mapping_key: mapping_key.clone(),
                source: Placement {
                    slot: locations[0].slot,
                    span: role.span_slots(),
                },
            }
        })
        .collect()
}

/// `version` 里留存的、数据单元那一类的映射条目，从第 `skipped` 条起取 `count` 条的搬迁计划项。
fn planned_relocations_of_kept_data_units(
    version: &TransactionOutput,
    skipped: usize,
    count: usize,
) -> Vec<PlannedRelocation> {
    let planned: Vec<PlannedRelocation> = version
        .mapping_entries_kept_for_the_snapshots
        .iter()
        .filter(|(key, _)| key.first() == Some(&UNIT_CLASS_DATA))
        .skip(skipped)
        .take(count)
        .map(|(key, locations)| PlannedRelocation {
            mapping_key: key.clone(),
            source: Placement {
                slot: locations[0].slot,
                span: 2,
            },
        })
        .collect();
    assert_eq!(planned.len(), count, "留存的数据单元映射条目够 {count} 条");
    planned
}

/// 发一批整理（`transaction::publish_relocation_batch`，搬 `planned` 那几项），会话接到这一版上：判核源落点那一处的计数——
/// 建索引：按 key 的索引、批量读者的单元索引各一份，计划里有留存的映射条目时再建一份留存条目按 key 的索引，不随批大小涨；
/// 解叶：进映射那几项的 key 走到的叶各解一次；查：每一项按 key 查一次，留存的那几项再按 key 在留存条目里查一次，进映射的那几项按角色取一次字节，
/// 每片解的叶按角色取一次单元，合起来是项数的两倍加解叶次数。
fn relocate_a_batch_and_check_the_relocation_sources(
    pool: &mut PoolBuilt,
    planned: &[PlannedRelocation],
    kept_items: u64,
) {
    let parameters = crate::common::parameters();
    let previous = version_of_the_session(&pool.session).clone();
    let items = u64::try_from(planned.len()).expect("项数");
    let role_keys: Vec<&[u8]> = planned
        .iter()
        .map(|item| item.mapping_key.as_slice())
        .filter(|key| {
            previous
                .mapped_units
                .iter()
                .any(|(_, mapped)| mapped.as_slice() == *key)
        })
        .collect();
    let routed_leaves = distinct_mapping_leaves_routed_to_by(&previous, role_keys.iter().copied());
    let session = &mut pool.session;
    let devices = &mut pool.devices;
    let (relocated, lookup_work) = lookup_work_while(|| {
        let mut writer = PoolWriter::new(&parameters, devices);
        publish_relocation_batch(
            &mut writer,
            &mut session.allocator,
            &previous,
            planned,
            session.instance,
        )
    });
    let relocated = relocated.unwrap_or_else(|refusal| panic!("搬 {items} 项的一批：{refusal:?}"));
    assert_every_lookup_examines_at_most_a_binary_search(
        &lookup_work,
        &format!("搬 {items} 项的一批"),
    );
    let relocated_kept = relocated
        .units_relocated_by_this_publish
        .iter()
        .filter(|unit| unit.holder == RelocatedUnitHolder::KeptForTheSnapshots)
        .count();
    assert_eq!(
        (
            relocated.units_relocated_by_this_publish.len(),
            u64::try_from(relocated_kept).expect("项数")
        ),
        (planned.len(), kept_items),
        "计划里每一项都搬了，留存的那几项按留存的搬"
    );
    session.current = PoolVersion::of_a_version_of_trees(relocated);
    let relocation_sources = lookup_work.of_the_site(LookupSite::RelocationSources);
    println!(
        "publish_lookup_by_identity relocation_batch items={items} kept_items={kept_items} routed_leaves={routed_leaves} relocation_sources={relocation_sources:?}"
    );
    let index_builds_of_the_batch = if kept_items == 0 { 2 } else { 3 };
    assert_eq!(
        (
            relocation_sources.index_builds,
            relocation_sources.mapping_leaf_decodes,
            relocation_sources.index_lookups
        ),
        (
            index_builds_of_the_batch,
            routed_leaves,
            2 * items + routed_leaves
        ),
        "搬 {items} 项（其中留存的 {kept_items} 项）的一批：建 {index_builds_of_the_batch} 份索引、进映射那几项的 key 走到的 {routed_leaves} 片叶各解一次、\
         查项数两倍加解叶次数那么多次（建索引次数、解叶次数、查的次数）：{relocation_sources:?}"
    );
    // 搬走的、这一版的树引用着的那几项（映射 key 是这一版某个角色的），在上一版的拷贝里换成目的地时按角色找 `units` 里那一项：
    // 建一份按角色的索引，每一项查一次。
    let mapping_positions =
        lookup_work.of_the_site(LookupSite::UnitsOfTheMappedRolesTakingTheirMappingPositions);
    let roles_relocated = u64::try_from(role_keys.len()).expect("项数");
    assert_eq!(
        (
            mapping_positions.index_builds,
            mapping_positions.index_lookups,
            mapping_positions.times_entered
        ),
        (1, roles_relocated, 1),
        "搬 {items} 项的一批，其中 {roles_relocated} 项是这一版的角色：把映射位置换进 `units` 时建一份按角色的索引、每个这样的角色查一次\
         （建索引次数、查的次数、进的次数）：{mapping_positions:?}"
    );
    assert_the_units_of_the_published_version_are_looked_up_through_one_index(
        version_of_the_session(&pool.session),
        &lookup_work,
        &format!("搬 {items} 项的一批"),
    );
}

/// 快照下整份重写过两个文件的池上，整理发三批：两项（一项进映射、一项留存）、八项（四项进映射、四项留存）与三项（都进映射、没有留存的）。
/// 有留存项的两批建的索引都是那三份、没有留存项的那批两份，不随批大小涨（每项现建索引的那一处按项数涨，留存条目的索引不等要找就建的多一份）；
/// 查的次数是项数的两倍加解叶次数（找留存条目或取字节改回线性的少了那几次）；解叶恰好是进映射那几项的 key 走到的叶（落点改回逐 key 查的按项数解）。
/// 每批把映射位置换进上一版的拷贝时，按角色找 `units` 那一项走建一次的索引；装这一版要写与点名的单元走同一份这一版的索引。
/// 判别力：`crates/mutations.tsv` 里点名这一条的几行（RA、RC、M7
/// ：映射位置换进 `units`、装要写与点名的单元改回线性找，留存条目索引不等要找就建）。
pub fn run_the_scenario_where_relocation_batches_of_two_and_of_eight_units_build_their_indexes_once_and_look_up_each_planned_unit_twice(
) {
    let PoolWithASnapshot { mut pool, .. } = pool_with_a_snapshot_of_files(8);
    for (position, rewritten_inode) in [pool.inodes[1], pool.inodes[2]].into_iter().enumerate() {
        whole_rewrite_published_once(
            &mut pool,
            rewritten_inode,
            rewritten_inode.0 + 5_000_011,
            4,
            crate::common::FIXED_WRITE_TIME_SECONDS
                + 9_000
                + u64::try_from(position).expect("序号"),
        );
    }
    let mut planned_batch_of_two = planned_relocations_of_the_data_units(
        version_of_the_session(&pool.session),
        pool.inodes[3],
        1,
    );
    planned_batch_of_two.extend(planned_relocations_of_kept_data_units(
        version_of_the_session(&pool.session),
        0,
        1,
    ));
    relocate_a_batch_and_check_the_relocation_sources(&mut pool, &planned_batch_of_two, 1);
    let mut planned_batch_of_eight = planned_relocations_of_the_data_units(
        version_of_the_session(&pool.session),
        pool.inodes[4],
        4,
    );
    planned_batch_of_eight.extend(planned_relocations_of_kept_data_units(
        version_of_the_session(&pool.session),
        1,
        4,
    ));
    relocate_a_batch_and_check_the_relocation_sources(&mut pool, &planned_batch_of_eight, 4);
    // 一项留存的都没有的一批：留存条目按 key 的索引不建，只建两份。
    let planned_batch_without_kept_units = planned_relocations_of_the_data_units(
        version_of_the_session(&pool.session),
        pool.inodes[5],
        3,
    );
    relocate_a_batch_and_check_the_relocation_sources(
        &mut pool,
        &planned_batch_without_kept_units,
        0,
    );
}
