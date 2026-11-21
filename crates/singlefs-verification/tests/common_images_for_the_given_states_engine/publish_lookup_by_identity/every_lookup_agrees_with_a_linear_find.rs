//! 等价：两档池上跑同一串操作，每一版的每一次按身份查与测试里的线性查找参照逐项相同，整理读一版取的映射条目与逐 key 查的参照相同、批量读者恰好解进映射的 key 走到的那几片叶。

use super::*;

/// 按身份查的几样与测试里的线性查找参照比过多少次。
#[derive(Debug, Default)]
struct LookupsCompared {
    versions: u64,
    unit_lookups: u64,
    mapping_key_lookups: u64,
    file_lookups: u64,
    central_mapping_lookups: u64,
    compaction_reads_of_the_central_mapping: u64,
    versions_whose_central_mapping_tree_has_several_nodes: u64,
}

/// 整理读一版该取到的映射条目，用逐 key 查（`central_mapping_locations_in_the_version`，每把 key 线性找叶、整片重解）当参照：
/// 每个进映射单元按 `mapped_units` 的次序查，查到的收，后面接这一版留存的映射条目。
fn mapping_entries_found_key_by_key(version: &TransactionOutput) -> Vec<CentralMappingEntry> {
    version
        .mapped_units
        .iter()
        .filter_map(
            |(_, key)| match central_mapping_locations_in_the_version(version, key) {
                MappingLookup::Found(locations) => Some(CentralMappingEntry {
                    key: key.clone(),
                    locations,
                }),
                MappingLookup::NodeMalformedOrNoEntryWithThisKey
                | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => None,
            },
        )
        .chain(
            version
                .mapping_entries_kept_for_the_snapshots
                .iter()
                .map(|(key, locations)| CentralMappingEntry {
                    key: key.clone(),
                    locations: *locations,
                }),
        )
        .collect()
}

/// 一版里每一种按身份查都与线性查找参照（`iter().find`、逐 key 的 `central_mapping_locations_in_the_version`）比一遍：表里的每个身份，
/// 再加一个哪一版都没有的身份；整理读这一版取的映射条目与逐 key 查的参照逐项比，批量读者解的叶数与 key 走到的叶数比。
fn compare_every_lookup_with_a_linear_find(
    version: &TransactionOutput,
    compared: &mut LookupsCompared,
) {
    compared.versions += 1;
    let absent_role = TransactionUnit::Data(DataUnitIndexInFile(u64::MAX));
    let units_by_role = version.units_by_role();
    for role in version
        .units
        .iter()
        .map(|unit| unit.identity)
        .chain([absent_role])
    {
        let found_linearly = version.units.iter().find(|unit| unit.identity == role);
        assert!(
            is_the_same_entry(units_by_role.first_entry_of(&role), found_linearly),
            "txg {}：units 按角色 {role:?} 查到的与线性找的不是同一项",
            version.root.checkpoint_txg.0
        );
        compared.unit_lookups += 1;
    }
    assert_the_index_lists_the_first_entry_of_every_identity_once(
        &units_by_role,
        &version.units,
        |unit| unit.identity,
        &format!("txg {} 的 units 按角色", version.root.checkpoint_txg.0),
    );
    let mapped_units_by_role = version.mapped_units_by_role();
    for role in version
        .mapped_units
        .iter()
        .map(|(role, _)| *role)
        .chain([absent_role])
    {
        let found_linearly = version
            .mapped_units
            .iter()
            .find(|(mapped, _)| *mapped == role);
        assert!(
            is_the_same_entry(mapped_units_by_role.first_entry_of(&role), found_linearly),
            "txg {}：mapped_units 按角色 {role:?} 查到的与线性找的不是同一项",
            version.root.checkpoint_txg.0
        );
        if let Some((_, key)) = found_linearly {
            assert_eq!(
                mapped_units_by_role.mapping_key_of(role),
                key.as_slice(),
                "txg {}：角色 {role:?} 的映射 key 与线性找的那一把相同",
                version.root.checkpoint_txg.0
            );
        }
        compared.mapping_key_lookups += 1;
    }
    assert_the_index_lists_the_first_entry_of_every_identity_once(
        &mapped_units_by_role,
        &version.mapped_units,
        |(role, _)| *role,
        &format!(
            "txg {} 的 mapped_units 按角色",
            version.root.checkpoint_txg.0
        ),
    );
    // 整理发批按映射 key 找角色走的索引：每一把 key（连同一把哪一版都没有的）查到的与在 `mapped_units` 上按 key 线性找的是同一项。
    let mapped_units_by_key = version.mapped_units_by_key();
    let absent_key = vec![0xff_u8; MAPPING_KEY_BYTES];
    for key in version
        .mapped_units
        .iter()
        .map(|(_, key)| key)
        .chain([&absent_key])
    {
        let found_linearly = version
            .mapped_units
            .iter()
            .find(|(_, mapped_key)| mapped_key == key);
        assert!(
            is_the_same_entry(
                mapped_units_by_key.first_entry_of(&key.as_slice()),
                found_linearly
            ),
            "txg {}：mapped_units 按 key {key:02x?} 查到的与线性找的不是同一项",
            version.root.checkpoint_txg.0
        );
        compared.mapping_key_lookups += 1;
    }
    assert_the_index_lists_the_first_entry_of_every_identity_once(
        &mapped_units_by_key,
        &version.mapped_units,
        |(_, key)| key.as_slice(),
        &format!(
            "txg {} 的 mapped_units 按 key",
            version.root.checkpoint_txg.0
        ),
    );
    // 释放的三处与快照判定那一圈进 deadlist 那一支按角色取映射条目走的批量形态：每个进映射的角色与「线性找 key、逐 key 查」逐项相同。
    let mut read_by_role = MappedUnitsOfAVersionReadByRole::of_the_version(version);
    for (role, _) in &version.mapped_units {
        let (_, key_found_linearly) = version
            .mapped_units
            .iter()
            .find(|(mapped, _)| mapped == role)
            .expect("角色就取自 mapped_units");
        assert_eq!(
            read_by_role.locations_of_role(*role),
            central_mapping_locations_in_the_version(version, key_found_linearly),
            "txg {}：角色 {role:?} 按角色批量取的映射条目与线性找 key、逐 key 查的不同",
            version.root.checkpoint_txg.0
        );
        compared.central_mapping_lookups += 1;
    }
    let files_by_inode = version.files_other_than_the_first_by_inode();
    for inode in version
        .files_other_than_the_first
        .iter()
        .map(|file| file.inode)
        .chain([InodeNumber(u64::MAX)])
    {
        assert!(
            is_the_same_entry(
                files_by_inode.first_entry_of(&inode),
                version.file_other_than_the_first(inode)
            ),
            "txg {}：按 inode 号 {} 查到的文件与线性找的不是同一项",
            version.root.checkpoint_txg.0,
            inode.0
        );
        compared.file_lookups += 1;
    }
    assert_the_index_lists_the_first_entry_of_every_identity_once(
        &files_by_inode,
        &version.files_other_than_the_first,
        |file| file.inode,
        &format!("txg {} 的文件按 inode 号", version.root.checkpoint_txg.0),
    );
    let mut read_leaf_by_leaf = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);
    for key in version
        .mapped_units
        .iter()
        .map(|(_, key)| key)
        .chain(
            version
                .mapping_entries_kept_for_the_snapshots
                .iter()
                .map(|(key, _)| key),
        )
        .chain([&absent_key])
    {
        assert_eq!(
            read_leaf_by_leaf.locations_of_key(key),
            central_mapping_locations_in_the_version(version, key),
            "txg {}：key {key:02x?} 批量读与逐 key 查不同",
            version.root.checkpoint_txg.0
        );
        compared.central_mapping_lookups += 1;
    }
    assert_eq!(
        central_mapping_as_the_compaction_reads_it(version),
        mapping_entries_found_key_by_key(version),
        "txg {}：整理读这一版取的映射条目与逐 key 查的参照逐项相同（不漏、不错、次序同）",
        version.root.checkpoint_txg.0
    );
    assert_the_compaction_read_decodes_exactly_the_routed_leaves(version);
    compared.compaction_reads_of_the_central_mapping += 1;
    if version.central_mapping_tree.shape.nodes().len() > 1 {
        compared.versions_whose_central_mapping_tree_has_several_nodes += 1;
    }
}

/// 这几块盘此刻的整份镜像（交池级 checker）。
fn memory_pool_of_the_sparse_devices(devices: &SparseDevices) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: crate::common::IMAGE_BYTES,
    }
}

/// 两块盘镜像（写过的扇区）的 sha256，十六进制：同一串操作写出的镜像逐字节相同，可在两份仓副本里比这一行。
fn digest_of_the_images(devices: &SparseDevices) -> String {
    let mut written = Vec::new();
    for (identity, device) in devices {
        written.extend_from_slice(&identity.0.to_le_bytes());
        for (sector, bytes) in device.image.written_sectors() {
            written.extend_from_slice(&sector.to_le_bytes());
            written.extend_from_slice(bytes);
        }
    }
    singlefs_verification::sha256::sha256_digest(&written)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// 等价那一串操作：造池（每一版都比）、整理推到停（每一版都比）、整理之后整份重写一个留着的文件、unlink 一个整份重写过的文件，
/// 最后池级 checker 判镜像。交回比过的次数与整理发了几批。
fn compare_every_lookup_along_a_history_on(
    images: &mut PoolCheckerImages,
    shape: PoolOfFiles,
) -> (LookupsCompared, u64) {
    let mut compared = LookupsCompared::default();
    let PoolBuilt {
        mut devices,
        mut session,
        inodes,
    } = build_a_pool_of_files(shape, &mut |version| {
        compare_every_lookup_with_a_linear_find(version, &mut compared);
    });
    let mut on_every_version = |version: &TransactionOutput| {
        compare_every_lookup_with_a_linear_find(version, &mut compared);
    };
    session
        .allocator
        .set_compaction_switches(product_switches_with_the_batch_size(
            COMPACTION_BATCH_UNITS_OF_THE_EQUIVALENCE_POOLS,
        ));
    let mut batches_published = 0;
    // 迭代上界是 COMPACTION_RUNS_AT_MOST；跨轮携带的是会话与发了几批；一次调用一版都没发就出圈。
    for _ in 0..COMPACTION_RUNS_AT_MOST {
        let (run, _) = run_compaction_and_count_its_lookups(&mut session, &mut devices);
        for version in &run.versions_published {
            on_every_version(version);
        }
        batches_published += run.batches_published;
        match run.outcome {
            CompactionRunOutcome::Stopped(_) => {}
            CompactionRunOutcome::Failed(failure) => {
                panic!("整理的一次发布没发成（{shape:?}）：{failure:?}")
            }
        }
        if run.versions_published.is_empty() {
            break;
        }
    }
    let mut write_time_seconds = crate::common::FIXED_WRITE_TIME_SECONDS + 1_000_000;
    let kept_file = inodes[0];
    let content = pseudo_random_content_five_bytes_short_of_data_units(
        shape.data_units_per_file,
        kept_file.0 + 2_000_003,
    );
    write_time_seconds += 1;
    publish_and_look_at_every_version(
        &mut session,
        &mut devices,
        UserChange::SequentialWriteOfAnInode {
            inode: kept_file,
            content: &content,
            write_time_seconds,
        },
        &mut on_every_version,
    );
    write_time_seconds += 1;
    publish_and_look_at_every_version(
        &mut session,
        &mut devices,
        UserChange::Unlink {
            inode: inodes[1],
            write_time_seconds,
        },
        &mut on_every_version,
    );
    drop(session);
    images.hand_out(
        "身份查找历史最终镜像",
        || memory_pool_of_the_sparse_devices(&devices),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    println!(
        "publish_lookup_by_identity shape={shape:?} batches_published={batches_published} images_sha256={}",
        digest_of_the_images(&devices)
    );
    (compared, batches_published)
}

/// 十六个四单元的文件（64 个数据单元，中央映射树一片叶）：每一版的每一次按身份查都与线性查找相同，整理真发了批，镜像池级 checker 判绿。
/// 判别力：`crates/mutations.tsv` 里以「七·6 按身份查：」起头的几行（索引漏了一类单元、照抄取错角色、文件按错的号查、批量读取最先那条），
/// 以「七·6 整理读版本：」起头的几行（整理读版本隔一把 key 才查、漏查第一把 key、绕开批量读者逐 key 查）。
pub fn run_the_scenario_where_every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units(
    images: &mut PoolCheckerImages,
) {
    let (compared, batches_published) = compare_every_lookup_along_a_history_on(
        images,
        PoolOfFiles {
            files: 16,
            data_units_per_file: 4,
        },
    );
    assert!(
        batches_published >= 1,
        "整理真搬了单元：读版本走到了批量读映射（{compared:?}）"
    );
    assert!(
        compared.versions >= 40
            && compared.file_lookups >= 16
            && compared.unit_lookups >= 64
            && compared.compaction_reads_of_the_central_mapping == compared.versions,
        "每一版都比过：{compared:?}"
    );
}

/// 四十八个八单元的文件（384 个数据单元，中央映射树不止一片叶）：同上，叶不止一片时批量读每片叶各解一次、各查各的。
pub fn run_the_scenario_where_every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_forty_eight_files_of_eight_data_units(
    images: &mut PoolCheckerImages,
) {
    let (compared, batches_published) = compare_every_lookup_along_a_history_on(
        images,
        PoolOfFiles {
            files: 48,
            data_units_per_file: 8,
        },
    );
    assert!(
        batches_published >= 1,
        "整理真搬了单元：读版本走到了批量读映射（{compared:?}）"
    );
    assert!(
        compared.versions_whose_central_mapping_tree_has_several_nodes >= 1,
        "这一档的中央映射树长到了不止一个节点：{compared:?}"
    );
}
