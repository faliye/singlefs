//! 整份重写一个文件时，换下的与照抄的每个单元在每一处调用点都经建一次的索引查，点名几处之外建的索引个数有上限，多层树照抄的节点按树分开数。

use super::*;

/// extent 树的节点（`build_extent_tree` 照抄的那几种）。
fn is_an_extent_tree_node(identity: TransactionUnit) -> bool {
    matches!(
        identity,
        TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
    )
}

/// 第一个文件之外一个文件的下段节点。
fn is_a_lower_node_of_a_file_other_than_the_first(identity: TransactionUnit) -> bool {
    matches!(
        identity,
        TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
    )
}

/// 第一个文件之外 48 个八单元的文件（中央映射树长到不止一片叶）上整份重写其中一个。
/// 重写之前，拿那个文件的数据单元与下段节点各调一次释放的三处（`transaction` 的三个公开入口，只读）：每一处各自建两份索引（按角色的 key 索引、
/// 批量读者的单元索引）、那几把 key 走到的叶各解一次、每个角色查一次 key、每片叶查一次单元；释放判定路径那一处每个角色按 inode 号经索引找一次上一版的文件。
/// 再整份重写它（一次发布）：快照判定那一圈（池里没有快照）每建一份索引至少查换下的数据单元那么多次；照抄的 extent 树节点、第一个文件之外文件的
/// 下段节点、多层码 2 树的节点各按角色（按 inode 号）经上一版建一次的索引查一次（发布算到定点时同一处可以走几遍，判整数遍）。
/// 改回线性的那一处查的次数掉到 0，每查一个角色现建索引的那一处建的次数按角色数涨，落点改回逐 key 查的那一处解叶次数按 key 数涨。
/// 七·6 后续批另判：核两条位置项与读盘核校验和经「经映射释放的角色清单」每走一遍各建一份按 inode 号找文件的读者与留给快照的角色清单索引；
/// 那一次发布里三棵多层码 2 树照抄的节点按树分开数，装要写与点名的单元、按 inode 号找文件、留给快照的角色清单与记录点名的数据角色都每走一遍建一份
/// （`assert_the_lookups_of_one_publish_build_one_index_per_pass`）；整棵 extent 树装的时候都在照抄那一处里，每个节点现建一份索引的建的次数记在那里。
/// 另判：点名的几处之外（`None`）每装一次建的索引至多一个常数份
/// （`assert_the_index_builds_outside_the_named_sites_stay_bounded`）。
/// 判别力：`crates/mutations.tsv` 里点名这一条的几行（RB、RD、RE、RF、M3、M4：释放与照抄那几处改回线性；MA2、MA3、MA5：装要写与点名的单元改回线性找；NA2：装记录点名的单元每点名一个角色现建一份索引、建在点名那一处之外）。
pub fn run_the_scenario_where_a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site(
) {
    let mut pool = pool_with_files_written_once(48, 8);
    let rewritten_inode = pool.inodes[7];
    let before_the_rewrite = version_of_the_session(&pool.session).clone();
    assert!(
        before_the_rewrite.central_mapping_tree.shape.nodes().len() > 1,
        "中央映射树不止一个节点：照抄多层码 2 树的节点走得到"
    );
    let file = before_the_rewrite
        .file_other_than_the_first(rewritten_inode)
        .expect("写过的文件在这一版里");
    let roles: Vec<TransactionUnit> = file
        .data_roles()
        .into_iter()
        .chain(file.extent_lower_node_roles())
        .collect();
    let replaced_roles = u64::try_from(roles.len()).expect("角色数");
    let routed_leaves = distinct_mapping_leaves_routed_to_by(
        &before_the_rewrite,
        before_the_rewrite
            .mapped_units
            .iter()
            .filter(|(role, _)| roles.contains(role))
            .map(|(_, key)| key.as_slice()),
    );
    assert!(
        replaced_roles >= 9,
        "八个数据单元连同下段节点：{replaced_roles}"
    );
    let one_pass_of_a_release_site = (2, routed_leaves, replaced_roles + routed_leaves);
    let (_, release_judgement) = lookup_work_while(|| {
        placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots(
            &before_the_rewrite,
            &pool.session.allocator,
            FileContentOfThePublish::NotRewrittenWhole,
            &roles,
            &[],
        )
        .expect("这个文件的单元在上一版里都分配着、映射里查得到")
    });
    let pool_devices: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    let (_, two_pool_devices_check) = lookup_work_while(|| {
        refuse_mapping_entries_that_do_not_name_two_pool_devices(
            &before_the_rewrite,
            &roles,
            &[],
            &pool_devices,
            FileContentOfThePublish::NotRewrittenWhole,
        )
        .expect("每条映射条目的两条位置项各指池里一块不同的盘")
    });
    let (_, release_checksum_check) = lookup_work_while(|| {
        copies_failing_the_release_checksum_check(
            &before_the_rewrite,
            &roles,
            &[],
            pool.devices.as_slice(),
            FileContentOfThePublish::NotRewrittenWhole,
        )
        .expect("位置项都指池里的盘")
    });
    println!(
        "publish_lookup_by_identity release_sites replaced_roles={replaced_roles} routed_leaves={routed_leaves} release_judgement={release_judgement:?} two_pool_devices_check={two_pool_devices_check:?} release_checksum_check={release_checksum_check:?}"
    );
    for (site, lookup_work) in [
        (
            LookupSite::MappingEntriesOfTheReleaseJudgement,
            &release_judgement,
        ),
        (
            LookupSite::MappingEntriesCheckedForTwoPoolDevices,
            &two_pool_devices_check,
        ),
        (
            LookupSite::MappingEntriesCheckedAgainstTheReleaseChecksums,
            &release_checksum_check,
        ),
    ] {
        let work = lookup_work.of_the_site(site);
        assert_eq!(
            (work.index_builds, work.mapping_leaf_decodes, work.index_lookups),
            one_pass_of_a_release_site,
            "{site:?}：建两份索引（按角色的 key 索引、批量读者的单元索引）、{replaced_roles} 个角色的 key 走到的 {routed_leaves} 片叶各解一次、\
             每个角色查一次 key、每片叶查一次单元（建索引次数、解叶次数、查的次数）：{work:?}"
        );
    }
    let files_of_the_replaced_roles =
        release_judgement.of_the_site(LookupSite::FilesOfTheReplacedRolesInThePreviousVersion);
    assert_eq!(
        (
            files_of_the_replaced_roles.index_builds,
            files_of_the_replaced_roles.index_lookups
        ),
        (1, replaced_roles),
        "释放判定路径：{replaced_roles} 个角色在上一版里有没有落点，各按 inode 号经一份建一次的索引找一次文件：{files_of_the_replaced_roles:?}"
    );
    // 留给快照的角色清单（这里是空的）在释放判定路径与经映射释放的角色清单那一处各建一次按角色的索引，换下的每个角色查一次。
    let kept_roles_of_the_release_judgement =
        release_judgement.of_the_site(LookupSite::RolesKeptForTheSnapshotsInTheReleaseJudgement);
    assert_eq!(
        (
            kept_roles_of_the_release_judgement.index_builds,
            kept_roles_of_the_release_judgement.index_lookups,
            kept_roles_of_the_release_judgement.times_entered
        ),
        (1, replaced_roles, 1),
        "释放判定路径：留给快照的角色清单建一次按角色的索引，{replaced_roles} 个换下的角色各查一次（建索引次数、查的次数、进的次数）：\
         {kept_roles_of_the_release_judgement:?}"
    );
    // 核两条位置项与读盘核校验和都经「经映射释放的角色清单」（读盘核校验和那一处先调核两条位置项，清单走两遍）：
    // 每走一遍按 inode 号找文件的读者建一份、留给快照的角色清单建一份，换下的每个角色各查一次。
    for (which, lookup_work) in [
        ("核两条位置项", &two_pool_devices_check),
        ("读盘核校验和", &release_checksum_check),
    ] {
        for site in [
            LookupSite::FilesOfTheMappedRolesReleasedByThisPublish,
            LookupSite::RolesKeptForTheSnapshotsAmongTheMappedRolesReleased,
        ] {
            let work = lookup_work.of_the_site(site);
            let passes = assert_one_index_built_per_entry(work, &format!("{which}：{site:?}"));
            assert_eq!(
                work.index_lookups,
                passes * replaced_roles,
                "{which}：{site:?} 走了 {passes} 遍，每遍 {replaced_roles} 个换下的角色各查一次：{work:?}"
            );
        }
    }

    let (_, published, lookup_work) = whole_rewrite_published_once(
        &mut pool,
        rewritten_inode,
        rewritten_inode.0 + 1_000_003,
        8,
        crate::common::FIXED_WRITE_TIME_SECONDS + 5_000,
    );
    println!("publish_lookup_by_identity whole_rewrite lookup_work={lookup_work:?}");
    let snapshot_judgement = lookup_work.of_the_site(LookupSite::SnapshotJudgement);
    assert!(
        snapshot_judgement.index_builds >= 1
            && snapshot_judgement.index_lookups >= 8 * snapshot_judgement.index_builds,
        "快照判定那一圈（池里没有快照）：每走一遍建一份按角色的索引，换下的用户可见角色（至少八个数据单元）各经它查一次映射 key：{snapshot_judgement:?}"
    );
    for (site, carried, which) in [
        (
            LookupSite::CarriedExtentTreeNodes,
            carried_units_of_the_kind(&published, is_an_extent_tree_node),
            "照抄的 extent 树节点",
        ),
        (
            LookupSite::FilesOfTheCarriedExtentLowerNodes,
            carried_units_of_the_kind(&published, is_a_lower_node_of_a_file_other_than_the_first),
            "照抄的第一个文件之外文件的下段节点",
        ),
    ] {
        assert_the_work_is_whole_passes_of(
            lookup_work.of_the_site(site),
            (0, 0, carried),
            &format!(
                "{site:?}：这一版{which}有 {carried} 个，走一遍每个经上一版建一次的索引查一次（这一处不另建索引、不解叶）"
            ),
        );
    }
    assert_the_carried_nodes_of_each_multi_level_tree_are_looked_up_through_one_index(
        &published,
        &lookup_work,
    );
    assert_the_lookups_of_one_publish_build_one_index_per_pass(
        &published,
        &lookup_work,
        8,
        "整份重写一个八单元的文件（池里没有快照）",
    );
    assert_the_index_builds_outside_the_named_sites_stay_bounded(
        &lookup_work,
        "整份重写一个八单元的文件（池里没有快照）",
    );
}
