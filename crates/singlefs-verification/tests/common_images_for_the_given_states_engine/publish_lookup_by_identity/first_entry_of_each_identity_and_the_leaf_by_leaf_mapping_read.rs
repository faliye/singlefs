//! 按身份建的索引本身：每个身份只留最先的那一条、缺的查不到、长的打乱表里同样；映射叶解不开、条目窄、同一把 key 两条时批量读与逐 key 查相同；表长四倍时每次查看过的项数涨不到门槛；查的计数按步数与次数记。

use super::*;

/// 同一身份出现几次时索引取最先那一项（与 `iter().find` 同一项），表里没有的身份查不到。
pub fn run_the_scenario_where_first_entry_of_each_identity_keeps_the_first_of_repeated_identities_and_misses_absent_ones(
) {
    let table: Vec<(u64, &str)> = vec![
        (7, "seven first"),
        (3, "three first"),
        (7, "seven second"),
        (3, "three second"),
        (9, "nine"),
    ];
    let index = FirstEntryOfEachIdentity::of_the_table(&table, |(identity, _)| *identity);
    for identity in [7_u64, 3, 9, 4] {
        let found_linearly = table.iter().find(|(candidate, _)| *candidate == identity);
        assert!(
            is_the_same_entry(index.first_entry_of(&identity), found_linearly),
            "身份 {identity}：索引查到的 {:?} 与线性找的 {found_linearly:?} 不是同一项",
            index.first_entry_of(&identity)
        );
    }
    assert_eq!(
        index.first_entry_of(&7).map(|(_, name)| *name),
        Some("seven first"),
        "重复的身份取最先那一项"
    );
    assert_eq!(index.first_entry_of(&4), None, "表里没有的身份查不到");
}

/// 换掉一版中央映射树唯一那片叶（根兼叶）的字节：照这一版那片叶的头重装，条目换成 `entries`、条目宽换成 `entry_width`。
fn version_with_the_mapping_leaf_rebuilt(
    version: &TransactionOutput,
    entry_width: u16,
    entries: &[Vec<u8>],
) -> TransactionOutput {
    let mut rebuilt = version.clone();
    let leaf = rebuilt
        .units
        .iter_mut()
        .find(|unit| unit.identity == TransactionUnit::MappingTree)
        .expect("这一版有中央映射树的根");
    let header = parse_index_node(&leaf.bytes).expect("这一版的映射叶解得开");
    leaf.bytes = Arc::from(build_index_node(
        header.tree,
        header.level,
        header.key_width,
        &header.smallest_key,
        &header.largest_key,
        header.birth_txg,
        &crate::common::E142_FILESYSTEM_IDENTIFIER,
        header.instance,
        header.birth_sequence,
        entry_width,
        entries,
    ));
    rebuilt
}

/// 一版里每一把进映射的 key（连同一把哪一版都没有的）批量读与逐 key 查逐项相同；交回比了几把。
fn compare_the_leaf_by_leaf_read_with_the_per_key_lookup(version: &TransactionOutput) -> usize {
    let mut read_leaf_by_leaf = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);
    let absent_key = vec![0xff_u8; MAPPING_KEY_BYTES];
    let keys: Vec<&Vec<u8>> = version
        .mapped_units
        .iter()
        .map(|(_, key)| key)
        .chain([&absent_key])
        .collect();
    for key in &keys {
        assert_eq!(
            read_leaf_by_leaf.locations_of_key(key),
            central_mapping_locations_in_the_version(version, key),
            "key {key:02x?}：批量读与逐 key 查不同"
        );
    }
    keys.len()
}

/// 中央映射树只有一片叶（根兼叶）的一版上，把那片叶换成三种：解不开（全零）、条目窄于字段表（每条截掉最后一个字节）、
/// 同一把 key 有两条（一条改过位置条目的放在原条目前面，一条放在后面）；三种叶上批量读与逐 key 查逐项相同，
/// 同一把 key 两条时取最后一条（放在后面的那一条改过的位置条目查得到）。
pub fn run_the_scenario_where_leaf_by_leaf_mapping_read_agrees_with_the_per_key_lookup_on_a_malformed_leaf_a_narrow_leaf_and_repeated_keys(
) {
    let (mut devices, mut session) = formatted_and_mounted(CompactionSwitches::PRODUCT);
    let first_inode = version_of_the_session(&session).inode_number_watermark();
    let content = pseudo_random_content_five_bytes_short_of_data_units(3, 0x7006);
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: crate::common::FIXED_WRITE_TIME_SECONDS + 1,
            },
        )
        .unwrap_or_else(|refusal| panic!("建一个 inode：{refusal:?}"));
    session
        .publish_user_change(
            &mut devices,
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(first_inode),
                content: &content,
                write_time_seconds: crate::common::FIXED_WRITE_TIME_SECONDS + 2,
            },
        )
        .unwrap_or_else(|refusal| panic!("给它写三个数据单元：{refusal:?}"));
    let version = version_of_the_session(&session).clone();
    assert_eq!(
        version.central_mapping_tree.shape.nodes().len(),
        1,
        "这一版的中央映射树只有根兼叶"
    );
    let leaf_header = parse_index_node(&version.unit(TransactionUnit::MappingTree).bytes)
        .expect("这一版的映射叶解得开");
    let entry_width = u16::try_from(leaf_header.entry_width).expect("条目宽装得进 u16");
    let original_entries = leaf_header.entries.clone();
    assert!(
        original_entries.len() >= 4,
        "映射叶里有好几条：三个数据单元、下段、inode 树、分配记录树……"
    );
    let compared_on_the_original = compare_the_leaf_by_leaf_read_with_the_per_key_lookup(&version);

    // 只换掉 malformed 那一份的根：克隆出来的一版与 version 共用同一份 Arc<[u8]> 单元字节，拿不到可写引用就地清零，所以整份换成同长的全零。
    let mut malformed = version.clone();
    let malformed_mapping_root = malformed
        .units
        .iter_mut()
        .find(|unit| unit.identity == TransactionUnit::MappingTree)
        .expect("这一版有中央映射树的根");
    malformed_mapping_root.bytes = Arc::from(vec![0; malformed_mapping_root.bytes.len()]);
    compare_the_leaf_by_leaf_read_with_the_per_key_lookup(&malformed);
    assert_eq!(
        CentralMappingOfAVersionReadLeafByLeaf::of_the_version(&malformed)
            .locations_of_key(&version.mapped_units[0].1),
        MappingLookup::NodeMalformedOrNoEntryWithThisKey,
        "叶解不开：进映射的 key 也查不到"
    );

    let narrow_entries: Vec<Vec<u8>> = original_entries
        .iter()
        .map(|entry| entry[..entry.len() - 1].to_vec())
        .collect();
    let narrow = version_with_the_mapping_leaf_rebuilt(&version, entry_width - 1, &narrow_entries);
    compare_the_leaf_by_leaf_read_with_the_per_key_lookup(&narrow);
    assert_eq!(
        CentralMappingOfAVersionReadLeafByLeaf::of_the_version(&narrow)
            .locations_of_key(&version.mapped_units[0].1),
        MappingLookup::EntryNarrowerThanItsFieldTable {
            entry_bytes: usize::from(entry_width - 1)
        },
        "条目窄于字段表：报那条条目的字节数"
    );

    let repeated_position = original_entries.len() / 2;
    let mut altered_entry = original_entries[repeated_position].clone();
    let last_byte = altered_entry.len() - 1;
    altered_entry[last_byte] ^= 0xff;
    for altered_entry_goes_after_the_original in [false, true] {
        let mut entries = original_entries.clone();
        let insert_at = if altered_entry_goes_after_the_original {
            repeated_position + 1
        } else {
            repeated_position
        };
        entries.insert(insert_at, altered_entry.clone());
        let repeated = version_with_the_mapping_leaf_rebuilt(&version, entry_width, &entries);
        compare_the_leaf_by_leaf_read_with_the_per_key_lookup(&repeated);
        let key = original_entries[repeated_position][..MAPPING_KEY_BYTES].to_vec();
        let expected_last_entry = if altered_entry_goes_after_the_original {
            &altered_entry
        } else {
            &original_entries[repeated_position]
        };
        let (_, expected_locations) =
            parse_mapping_entry(expected_last_entry).expect("55 字节的映射条目切得动");
        match CentralMappingOfAVersionReadLeafByLeaf::of_the_version(&repeated)
            .locations_of_key(&key)
        {
            MappingLookup::Found(locations) => assert_eq!(
                locations,
                expected_locations,
                "同一把 key 两条：取后面那一条（改过的放在{}）",
                if altered_entry_goes_after_the_original {
                    "后面"
                } else {
                    "前面"
                }
            ),
            other @ (MappingLookup::NodeMalformedOrNoEntryWithThisKey
            | MappingLookup::EntryNarrowerThanItsFieldTable { .. }) => {
                panic!("重复的那把 key 查得到：{other:?}")
            }
        }
    }
    assert!(compared_on_the_original >= 4, "原样那一版每把 key 都比过");
}

/// 在 `table` 上建按身份的索引（`FirstEntryOfEachIdentity`，发布与整理用的同一个），把表里每一项的身份连同一个表里没有的
/// `absent_identity` 各查一次：表里的查得到、`absent_identity` 查不到。交回这几次查在产品里记下的计数（建索引那一次不在其内）。
fn lookup_work_of_looking_up_every_identity<Entry, Identity: Ord + Copy>(
    table: &[Entry],
    identity_of: impl Fn(&Entry) -> Identity,
    absent_identity: Identity,
) -> LookupWork {
    let index = FirstEntryOfEachIdentity::of_the_table(table, &identity_of);
    let ((), lookup_work) = lookup_work_while(|| {
        // 迭代上界是表长加一。
        for identity in table.iter().map(&identity_of).chain([absent_identity]) {
            assert_eq!(
                index.first_entry_of(&identity).is_some(),
                identity != absent_identity,
                "表里的身份查得到，表里没有的查不到"
            );
        }
    });
    assert_every_lookup_examines_at_most_a_binary_search(&lookup_work, "合成表上每个身份查一次");
    lookup_work.of_every_site()
}

/// 合成表的两档：512 个与 2048 个不同的身份，按一个与表长互素的步长打乱次序。
const IDENTITIES_OF_THE_SMALLER_SYNTHETIC_TABLE: u64 = 512;

const IDENTITIES_OF_THE_LARGER_SYNTHETIC_TABLE: u64 = 2048;

/// 打乱合成表次序的步长：素数，与 2 的幂的表长互素，`(位置 × 步长) mod 表长` 走遍每个身份恰好一次。
const STRIDE_THAT_SHUFFLES_THE_SYNTHETIC_TABLE: u64 = 7919;

fn synthetic_table_of_identities(identities: u64) -> Vec<u64> {
    (0..identities)
        .map(|position| position * STRIDE_THAT_SHUFFLES_THE_SYNTHETIC_TABLE % identities)
        .collect()
}

/// 512 个与 2048 个身份的合成表上，按身份的索引每次查平均看过的索引项数涨不到门槛的倍数（读产品里的计数，不读挂钟与 CPU 时间）。
/// 判别力：`crates/mutations.tsv` 里「七·6 按身份查：索引从大到小逐项找」那一行。
pub fn run_the_scenario_where_first_entry_of_each_identity_examines_less_than_twice_as_many_entries_per_lookup_on_a_table_four_times_as_long(
) {
    let smaller = lookup_work_of_looking_up_every_identity(
        &synthetic_table_of_identities(IDENTITIES_OF_THE_SMALLER_SYNTHETIC_TABLE),
        |identity| *identity,
        u64::MAX,
    );
    let larger = lookup_work_of_looking_up_every_identity(
        &synthetic_table_of_identities(IDENTITIES_OF_THE_LARGER_SYNTHETIC_TABLE),
        |identity| *identity,
        u64::MAX,
    );
    let growth_in_percent = growth_in_percent_of_the_entries_examined_per_lookup(smaller, larger);
    println!(
        "publish_lookup_by_identity synthetic_table smaller={smaller:?} larger={larger:?} growth_in_percent={growth_in_percent}"
    );
    assert!(
        growth_in_percent < HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE,
        "表长四倍，每次查平均看过的索引项数涨了 {growth_in_percent}%，门槛 {HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE}%（小表 {smaller:?}，大表 {larger:?}）"
    );
}

/// 4096 项、64 个身份各出现 64 次（次序按与表长互素的步长打乱）的长表上，索引对每个身份取的是表里它最先那一项（比地址，同 `iter().find`），
/// 按身份升序交出的每一项也是。短表上不稳定排序碰巧也保序，长表上同一身份的几项会被它换了次序。
/// 判别力：`crates/mutations.tsv` 里正推 M1（稳定排序换成不稳定排序）。
pub fn run_the_scenario_where_first_entry_of_each_identity_keeps_the_first_entry_of_identities_repeated_across_a_long_shuffled_table(
) {
    const ENTRIES: u64 = 4096;
    const DISTINCT_IDENTITIES: u64 = 64;
    let table: Vec<(u64, u64)> = (0..ENTRIES)
        .map(|position| {
            (
                position * STRIDE_THAT_SHUFFLES_THE_SYNTHETIC_TABLE % ENTRIES % DISTINCT_IDENTITIES,
                position,
            )
        })
        .collect();
    let index = FirstEntryOfEachIdentity::of_the_table(&table, |(identity, _)| *identity);
    for identity in (0..DISTINCT_IDENTITIES).chain([DISTINCT_IDENTITIES]) {
        let found_linearly = table.iter().find(|(candidate, _)| *candidate == identity);
        assert!(
            is_the_same_entry(index.first_entry_of(&identity), found_linearly),
            "身份 {identity}：索引查到的 {:?} 不是表里它最先那一项 {found_linearly:?}",
            index.first_entry_of(&identity)
        );
    }
    assert_the_index_lists_the_first_entry_of_every_identity_once(
        &index,
        &table,
        |(identity, _)| *identity,
        "4096 项、64 个身份的长表",
    );
}

/// 计数器自己会红：在 512 个身份的合成表上建一次索引记一次建、从前往后与从后往前各走一遍各记 512 项看过、
/// 每个身份连同一个表里没有的各查一次记 513 次查、每次看过的项数在 1 与折半查找的层数之间。
/// 判别力：`crates/mutations.tsv` 里点名这一条的几行（RG：删掉从前往后走那一句计数）。
pub fn run_the_scenario_where_lookup_work_counts_one_entry_per_step_of_a_walk_from_either_end_and_one_lookup_per_search(
) {
    let table = synthetic_table_of_identities(IDENTITIES_OF_THE_SMALLER_SYNTHETIC_TABLE);
    let (index, building) =
        lookup_work_while(|| FirstEntryOfEachIdentity::of_the_table(&table, |identity| *identity));
    assert_eq!(
        building.of_every_site(),
        LookupWork {
            index_builds: 1,
            ..LookupWork::default()
        },
        "建一次索引记一次建、别的都不记"
    );
    let (walked_forward, forward) = lookup_work_while(|| index.first_entries().count());
    let (walked_backward, backward) = lookup_work_while(|| index.first_entries().rev().count());
    let entries = u64::try_from(table.len()).expect("项数");
    for (walked, work, direction) in [
        (walked_forward, forward, "从前往后"),
        (walked_backward, backward, "从后往前"),
    ] {
        assert_eq!(
            (u64::try_from(walked).expect("项数"), work.of_every_site()),
            (
                entries,
                LookupWork {
                    index_entries_examined: entries,
                    ..LookupWork::default()
                }
            ),
            "{direction}走一遍：走过 {entries} 项、记 {entries} 项看过，不记查与建"
        );
    }
    let (_, searching) = lookup_work_while(|| {
        // 迭代上界是表长加一。
        for identity in table.iter().copied().chain([u64::MAX]) {
            let _ = index.first_entry_of(&identity);
        }
    });
    let searched = searching.of_every_site();
    assert!(
        searched.index_lookups == entries + 1
            && searched.index_builds == 0
            && searched.index_entries_examined > entries
            && searched.index_entries_examined
                <= (entries + 1) * binary_digits_of(entries)
            && searched.largest_index_looked_up == entries,
        "每个身份连同一个表里没有的各查一次：记 {} 次查，每次看过的项在 1 与 {} 之间，最大的索引 {entries} 项：{searched:?}",
        entries + 1,
        binary_digits_of(entries)
    );
}
