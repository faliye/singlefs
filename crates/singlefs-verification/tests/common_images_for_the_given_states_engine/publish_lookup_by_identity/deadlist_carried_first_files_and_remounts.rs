//! 快照下带几片叶 deadlist 的重写、照抄第一个文件的整份重写、重挂池子与没有一把有位置的 key 时，建几次索引、按树怎么数。

use super::*;

/// deadlist 树长到几片叶的那个池：第一个文件之外一个小文件，后面跟一串大文件；快照之后整份重写前面那一段大文件，换下的单元进 deadlist。
/// 每个大文件换下约三十几条 deadlist 条目，二十四个约八百条；一片 deadlist 叶装约三百七十条（16 KiB 节点、43 字节条目，推的），按 key 递增追加时
/// 叶分裂成各约一半，于是长到四五片叶（用例开头现判至少三片叶）。
const DATA_UNITS_OF_THE_SMALL_FILE_OF_THE_DEADLIST_POOL: usize = 8;

const DATA_UNITS_OF_A_LARGE_FILE_OF_THE_DEADLIST_POOL: usize = 32;

const LARGE_FILES_OF_THE_DEADLIST_POOL: usize = 26;

const LARGE_FILES_REWRITTEN_UNDER_THE_SNAPSHOT: usize = 24;

/// 这个池里 deadlist 树至少几个节点（根加至少三片叶）：整份重写一个文件只改一两片叶与根，照抄的就不止一个。
const DEADLIST_TREE_NODES_AT_LEAST: usize = 4;

/// 记账树压小之后一片叶装几行（只供测试的 `CodeTwoTreeNodeCapacities::CappedForTests`）：两块盘的记账行是池级 3 行、每块盘 6 行，
/// 加 inode 号水位的台阶行；造池那几次发布之后不到 20 行（用例开头现判；推的是 16 行：建完 inode 量到 56 行，减去 40 次建 inode 各一级台阶），装得进一片叶。
/// 产品容量下一片叶装 477 行，两块盘的池里记账树长不到第二个节点、照抄不到。
const ACCOUNTING_ROWS_ONE_CAPPED_LEAF_HOLDS: usize = 20;

/// 记账树压小之后一个内部节点装几个孩子。
const ACCOUNTING_CHILDREN_ONE_CAPPED_INTERNAL_NODE_HOLDS: usize = 4;

/// 量之前一次建一个 inode 建几次：每次多一级台阶行，记账树长到几片叶（量到的是 56 行、5 个节点）；之后的整份重写只改池级与每块盘那几行（代段换成这次的 txg），
/// 只装旧台阶行的那几片叶与它们的父节点照抄。
const INODE_CREATIONS_BEFORE_THE_MEASURED_REWRITES: u64 = 40;

/// 记账树照抄的节点至少几个（照抄到不止一个节点）。
const CARRIED_ACCOUNTING_NODES_AT_LEAST: u64 = 2;

/// 快照下 deadlist 树长到几片叶的池上，整份重写那个八单元的小文件、再整份重写一个没重写过的三十二单元的大文件（各一次发布，一次写的单元数在两档之间变）：
/// - 三棵多层码 2 树照抄的节点按树分开数：deadlist 树照抄了不止一个节点，
///   每个经上一版建一次的索引查一次；
/// - 留给快照的角色清单两处、记录点名的数据角色那一处、按 inode 号找文件三处、装要写与点名的单元，
///   两档都是每走一遍建一份索引、查的次数至少是这次重写的数据单元数（建索引的次数不随一次写的单元数涨；退回逐个比的那一处一次都不查）。
///
/// 另造：量之前把记账树压小（一片叶 [`ACCOUNTING_ROWS_ONE_CAPPED_LEAF_HOLDS`] 行），一次建一个 inode 建
/// [`INODE_CREATIONS_BEFORE_THE_MEASURED_REWRITES`] 次，台阶行把记账树撑到几片叶；两次重写都照这个容量装，三棵树都照抄了节点
/// （记账树至少 [`CARRIED_ACCOUNTING_NODES_AT_LEAST`] 个），每棵各判整数遍。另判：点名的几处之外（`None`）建的索引两档都至多一个常数份
/// （量到的是 5 与 6：多出的那一份不随单元数涨，单元数是四倍）。
///
/// 判别力：`crates/mutations.tsv` 里点名这一条的几行（MA4：记账树与 deadlist 树照抄的节点退回在 `units` 上线性找；
/// 留给快照的角色清单两处、记录点名的数据角色那一处退回 `.contains`；
/// NA1：只有记账树照抄的节点退回线性找）。
pub fn run_the_scenario_where_rewrites_of_eight_and_of_thirty_two_data_units_under_a_snapshot_with_a_deadlist_of_several_leaves_build_one_index_per_pass_and_count_each_tree_apart(
) {
    let data_units_of_each_file: Vec<usize> =
        std::iter::once(DATA_UNITS_OF_THE_SMALL_FILE_OF_THE_DEADLIST_POOL)
            .chain(std::iter::repeat_n(
                DATA_UNITS_OF_A_LARGE_FILE_OF_THE_DEADLIST_POOL,
                LARGE_FILES_OF_THE_DEADLIST_POOL,
            ))
            .collect();
    let mut pool = pool_with_files_of_the_sizes(&data_units_of_each_file);
    pool.session
        .publish_user_change(&mut pool.devices, UserChange::CreateSnapshot)
        .unwrap_or_else(|refusal| panic!("建快照：{refusal:?}"));
    let large_files: Vec<InodeNumber> = pool.inodes[1..].to_vec();
    // 每重写一个大文件写入时间往后推一秒。
    for (write_time_seconds, inode) in (crate::common::FIXED_WRITE_TIME_SECONDS + 20_000..)
        .zip(&large_files[..LARGE_FILES_REWRITTEN_UNDER_THE_SNAPSHOT])
    {
        let content = pseudo_random_content_five_bytes_short_of_data_units(
            DATA_UNITS_OF_A_LARGE_FILE_OF_THE_DEADLIST_POOL,
            inode.0 + 7_000_001,
        );
        pool.session
            .publish_user_change(
                &mut pool.devices,
                UserChange::SequentialWriteOfAnInode {
                    inode: *inode,
                    content: &content,
                    write_time_seconds,
                },
            )
            .unwrap_or_else(|refusal| panic!("快照下整份重写 inode {}：{refusal:?}", inode.0));
    }
    let deadlist_before = version_of_the_session(&pool.session);
    println!(
        "publish_lookup_by_identity deadlist_pool deadlist_entries={} deadlist_tree_nodes={}",
        deadlist_before.deadlist_entries.len(),
        deadlist_before.deadlist_tree.shape.nodes().len()
    );
    assert!(
        deadlist_before.deadlist_tree.shape.nodes().len() >= DEADLIST_TREE_NODES_AT_LEAST,
        "deadlist 树长到至少 {DEADLIST_TREE_NODES_AT_LEAST} 个节点：{} 条 deadlist 条目，{} 个节点",
        deadlist_before.deadlist_entries.len(),
        deadlist_before.deadlist_tree.shape.nodes().len()
    );
    let accounting_tree_capped = CodeTwoTreeNodeCapacities::CappedForTests {
        accounting: CodeTwoTreeNodeCapacity {
            leaf_entries: ACCOUNTING_ROWS_ONE_CAPPED_LEAF_HOLDS,
            internal_entries: ACCOUNTING_CHILDREN_ONE_CAPPED_INTERNAL_NODE_HOLDS,
        },
        central_mapping: MultiLevelCodeTwoTree::CentralMapping.node_capacity_of_the_node_format(),
    };
    assert!(
        version_of_the_session(&pool.session)
            .accounting_entries
            .len()
            <= ACCOUNTING_ROWS_ONE_CAPPED_LEAF_HOLDS,
        "压小之前记账树一片叶装得下（{} 行）：压小的容量从一个合规的上一版接着长",
        version_of_the_session(&pool.session)
            .accounting_entries
            .len()
    );
    // 每建一个 inode 写入时间往后推一秒。
    for write_time_seconds in (crate::common::FIXED_WRITE_TIME_SECONDS + 25_000..)
        .take(usize::try_from(INODE_CREATIONS_BEFORE_THE_MEASURED_REWRITES).expect("次数"))
    {
        one_inode_created_with_node_capacities(
            &mut pool,
            write_time_seconds,
            accounting_tree_capped,
        );
    }
    println!(
        "publish_lookup_by_identity deadlist_pool accounting_entries={} accounting_tree_nodes={}",
        version_of_the_session(&pool.session)
            .accounting_entries
            .len(),
        version_of_the_session(&pool.session)
            .accounting_tree
            .node_count()
    );
    let small_file = pool.inodes[0];
    let large_file_not_rewritten = large_files[LARGE_FILES_REWRITTEN_UNDER_THE_SNAPSHOT];
    for (inode, data_units, write_time_seconds) in [
        (
            small_file,
            DATA_UNITS_OF_THE_SMALL_FILE_OF_THE_DEADLIST_POOL,
            crate::common::FIXED_WRITE_TIME_SECONDS + 30_000,
        ),
        (
            large_file_not_rewritten,
            DATA_UNITS_OF_A_LARGE_FILE_OF_THE_DEADLIST_POOL,
            crate::common::FIXED_WRITE_TIME_SECONDS + 30_001,
        ),
    ] {
        let (_, published, lookup_work) = whole_rewrite_published_once_with_node_capacities(
            &mut pool,
            inode,
            inode.0 + 9_000_011,
            data_units,
            write_time_seconds,
            accounting_tree_capped,
        );
        let what = format!("快照下整份重写一个 {data_units} 单元的文件（deadlist 树几片叶）");
        let [carried_accounting_nodes, carried_deadlist_nodes, carried_mapping_nodes] =
            assert_the_carried_nodes_of_each_multi_level_tree_are_looked_up_through_one_index(
                &published,
                &lookup_work,
            );
        println!(
            "publish_lookup_by_identity deadlist_pool_rewrite data_units={data_units} carried_accounting_nodes={carried_accounting_nodes} \
             carried_deadlist_nodes={carried_deadlist_nodes} carried_mapping_nodes={carried_mapping_nodes} lookup_work={lookup_work:?}"
        );
        assert!(
            carried_deadlist_nodes >= 2,
            "{what}：deadlist 树照抄了不止一个节点（{carried_deadlist_nodes} 个）"
        );
        assert!(
            carried_accounting_nodes >= CARRIED_ACCOUNTING_NODES_AT_LEAST
                && carried_mapping_nodes >= 1,
            "{what}：记账树照抄了至少 {CARRIED_ACCOUNTING_NODES_AT_LEAST} 个节点（{carried_accounting_nodes} 个）、中央映射树也照抄了\
             （{carried_mapping_nodes} 个）：三棵树都判整数遍"
        );
        assert_the_lookups_of_one_publish_build_one_index_per_pass(
            &published,
            &lookup_work,
            u64::try_from(data_units).expect("单元数"),
            &what,
        );
        assert_the_index_builds_outside_the_named_sites_stay_bounded(&lookup_work, &what);
    }
}

/// 照抄第一个文件的那两个池里，第一个文件各写几个数据单元。
const DATA_UNITS_OF_THE_FIRST_FILE_OF_EACH_POOL: [usize; 2] = [8, 64];

/// 照抄第一个文件的那两个池里第一个文件之外的文件（各整份写一次，量的时候各整份重写一次）：一次写的单元数在两档之间变，这次重写的角色数跟着变。
const DATA_UNITS_OF_THE_FILES_REWRITTEN_WHILE_THE_FIRST_FILE_IS_CARRIED: [usize; 2] = [2, 16];

/// 第一个文件分别写成 8 个与 64 个数据单元的两个池上，各整份重写一个两单元与一个十六单元的文件（第一个文件照抄；一次写的单元数在两档之间变，
/// 这次重写的角色数跟着变）：装这一版时第一个文件的每个数据单元问它在不在这次重写的角色里，经这次重写的角色清单建一次的索引各查一次
/// ——每装一次进一次、建一份，查第一个文件的数据单元数那么多次，每次查至多一次
/// 折半查找（不是在清单上逐个比，那样比较次数到第一个文件的数据单元数乘这次重写的角色数）；点名的几处之外建的索引四次一样多。
/// 判别力：`crates/mutations.tsv` 里点名这一条的那一行（退回在这次重写的角色清单上逐个 `contains`）。
// harness-test-granularity:one-scenario 两个池是同一个场景的两档：判的是查的次数随第一个文件的单元数变、建的次数与点名的几处之外的份数不变，单造一档比不出
pub fn run_the_scenario_where_whole_rewrites_carrying_a_first_file_of_eight_and_of_sixty_four_data_units_ask_for_each_of_its_data_units_through_one_index_of_the_rewritten_roles(
) {
    let mut index_builds_outside_the_named_sites_of_each_rewrite = Vec::new();
    for first_file_data_units in DATA_UNITS_OF_THE_FIRST_FILE_OF_EACH_POOL {
        let mut pool = pool_with_files_of_the_sizes(
            &DATA_UNITS_OF_THE_FILES_REWRITTEN_WHILE_THE_FIRST_FILE_IS_CARRIED,
        );
        let first_file_content =
            pseudo_random_content_five_bytes_short_of_data_units(first_file_data_units, 0x51f1);
        pool.session
            .publish_user_change(
                &mut pool.devices,
                UserChange::SequentialWrite(FirstFile {
                    content: &first_file_content,
                    write_time_seconds: crate::common::FIXED_WRITE_TIME_SECONDS + 40_000,
                }),
            )
            .unwrap_or_else(|refusal| {
                panic!("第一个文件写 {first_file_data_units} 个数据单元：{refusal:?}")
            });
        let inodes = pool.inodes.clone();
        // 每重写一个文件写入时间往后推一秒。
        for ((inode, data_units), write_time_seconds) in inodes
            .iter()
            .zip(DATA_UNITS_OF_THE_FILES_REWRITTEN_WHILE_THE_FIRST_FILE_IS_CARRIED)
            .zip(crate::common::FIXED_WRITE_TIME_SECONDS + 40_001..)
        {
            let (_, published, lookup_work) = whole_rewrite_published_once(
                &mut pool,
                *inode,
                inode.0 + 11_000_027,
                data_units,
                write_time_seconds,
            );
            let what = format!(
                "第一个文件 {first_file_data_units} 个数据单元时整份重写一个 {data_units} 单元的文件"
            );
            let first_file_data_units_carried =
                u64::try_from(published.data_pointers.len()).expect("单元数");
            let rewritten_roles = u64::try_from(published.rewritten.len()).expect("角色数");
            let assemblies = lookup_work
                .of_the_site(LookupSite::UnitsOfTheRolesWrittenByThisPublish)
                .times_entered;
            let rewritten_roles_asked = lookup_work
                .of_the_site(LookupSite::RewrittenRolesAmongTheDataUnitsOfTheCarriedFirstFile);
            println!(
                "publish_lookup_by_identity carried_first_file first_file_data_units={first_file_data_units_carried} rewritten_roles={rewritten_roles} \
                 comparisons_of_a_scan_of_the_rewritten_roles_at_most={} rewritten_roles_asked={rewritten_roles_asked:?}",
                first_file_data_units_carried * rewritten_roles
            );
            assert_eq!(
                first_file_data_units_carried,
                u64::try_from(first_file_data_units).expect("单元数"),
                "{what}：这一版照抄了第一个文件的全部数据单元"
            );
            assert_eq!(
                (
                    rewritten_roles_asked.times_entered,
                    rewritten_roles_asked.index_builds,
                    rewritten_roles_asked.index_lookups
                ),
                (
                    assemblies,
                    assemblies,
                    assemblies * first_file_data_units_carried
                ),
                "{what}：装了 {assemblies} 次，每装一次这次重写的 {rewritten_roles} 个角色建一份按角色的索引，第一个文件的 \
                 {first_file_data_units_carried} 个数据单元各查一次（进的次数、建索引次数、查的次数）：{rewritten_roles_asked:?}"
            );
            index_builds_outside_the_named_sites_of_each_rewrite.push(
                assert_the_index_builds_outside_the_named_sites_stay_bounded(&lookup_work, &what),
            );
        }
    }
    assert!(
        index_builds_outside_the_named_sites_of_each_rewrite
            .windows(2)
            .all(|pair| pair[0] == pair[1]),
        "第一个文件两档、一次写的单元数两档，点名的几处之外的（装的次数，建索引的次数）一样：{index_builds_outside_the_named_sites_of_each_rewrite:?}"
    );
}

/// 重挂的那两个池里第一个文件之外有几个文件。
const FILES_OF_EACH_REMOUNTED_POOL: [u64; 2] = [4, 16];

/// 重挂的那两个池里每个文件几个数据单元。
const DATA_UNITS_OF_EACH_FILE_OF_THE_REMOUNTED_POOLS: usize = 2;

/// 第一个文件之外 4 个与 16 个两单元文件的两个池，各不卸载直接重挂一次（可写挂载从盘上重建这一版），读这次挂载在产品里做的按身份查：
/// - 把映射位置换进重建出来那一版的 `units`（`take_the_mapping_positions_over_the_location_hints`）：
///   进 1 次、建 1 份按角色的索引、查有位置的进映射角色那么多次（这一版每把映射 key 都查得到映射条目，就是进映射的角色数）；
/// - 数据单元照 inode 记录核时按 inode 号找记录：进 1 次、建 1 份按 inode 号的索引，第一个文件与
///   第一个文件之外每个文件各查一次（文件数加一，随文件数线性涨；在全部记录上逐个找则是文件数的平方）。
///
/// 每次查至多一次折半查找。两档的进、建次数一样，查的次数跟着进映射的角色数与文件数变。
/// 判别力：`crates/mutations.tsv` 里点名这一条的两行（NA3：恢复那一侧每把有位置的 key 调一次；
/// 按 inode 号找记录退回在全部记录上逐个找）。
// harness-test-granularity:one-scenario 两个池是同一个场景的两档：判的是查的次数随文件数变、进与建的次数不变，单造一档比不出
pub fn run_the_scenario_where_remounting_pools_of_four_and_of_sixteen_files_builds_one_index_for_the_mapping_positions_and_one_for_the_inode_records(
) {
    let parameters = crate::common::parameters();
    for files in FILES_OF_EACH_REMOUNTED_POOL {
        let PoolBuilt {
            mut devices,
            session,
            inodes: _,
        } = pool_with_files_written_once(files, DATA_UNITS_OF_EACH_FILE_OF_THE_REMOUNTED_POOLS);
        let before_the_remount = version_of_the_session(&session).clone();
        drop(session);
        let mapped_roles = u64::try_from(before_the_remount.mapped_units.len()).expect("角色数");
        assert!(
            before_the_remount.mapped_units.iter().all(|(_, key)| matches!(
                central_mapping_locations_in_the_version(&before_the_remount, key),
                MappingLookup::Found(_)
            )),
            "重挂之前那一版每把映射 key 都查得到映射条目：有位置的进映射角色就是全部 {mapped_roles} 个"
        );
        assert_eq!(
            u64::try_from(before_the_remount.files_other_than_the_first.len()).expect("文件数"),
            files,
            "第一个文件之外的 {files} 个文件都有数据单元"
        );
        let (mounted, lookup_work) = lookup_work_while(|| {
            mount_writable_with_compaction_switches(
                &parameters,
                &mut devices,
                CompactionSwitches::PRODUCT,
            )
        });
        mounted.unwrap_or_else(|error| panic!("重挂 {files} 个文件的池：{error:?}"));
        assert_every_lookup_examines_at_most_a_binary_search(
            &lookup_work,
            &format!("重挂 {files} 个文件的池"),
        );
        let mapping_positions =
            lookup_work.of_the_site(LookupSite::UnitsOfTheMappedRolesTakingTheirMappingPositions);
        let inode_records =
            lookup_work.of_the_site(LookupSite::InodeRecordsOfTheFilesJudgedByTheRecovery);
        println!(
            "publish_lookup_by_identity remount files={files} mapped_roles={mapped_roles} mapping_positions={mapping_positions:?} \
             inode_records={inode_records:?}"
        );
        assert_eq!(
            (
                mapping_positions.times_entered,
                mapping_positions.index_builds,
                mapping_positions.index_lookups
            ),
            (1, 1, mapped_roles),
            "重挂 {files} 个文件的池：重建这一版时把映射位置换进 `units` 进 1 次、建 1 份按角色的索引，{mapped_roles} 个有位置的进映射角色各查一次\
             （进的次数、建索引次数、查的次数）：{mapping_positions:?}"
        );
        assert_eq!(
            (
                inode_records.times_entered,
                inode_records.index_builds,
                inode_records.index_lookups
            ),
            (1, 1, files + 1),
            "重挂 {files} 个文件的池：数据单元照 inode 记录核时进 1 次、建 1 份按 inode 号的索引，第一个文件与 {files} 个别的文件各找一次记录\
             （进的次数、建索引次数、查的次数）：{inode_records:?}"
        );
    }
}

/// 把映射位置换进 `units` 时一把有位置的 key 都没有：那一处不进、不建索引，这一版一个字节不变（`take_the_mapping_positions_over_the_location_hints`
/// 末尾那一句「一把有位置的 key 都没有的不进、不建」）。
/// 判别力：`crates/mutations.tsv` 里点名这一条的那一行（ALT12：一把都没有也照建索引）。
pub fn run_the_scenario_where_taking_the_mapping_positions_without_any_key_that_has_a_position_builds_no_index_and_leaves_the_version_unchanged(
) {
    let pool = pool_with_files_written_once(2, 2);
    let before = version_of_the_session(&pool.session).clone();
    let mut version = before.clone();
    let ((), lookup_work) = lookup_work_while(|| {
        take_the_mapping_positions_over_the_location_hints(&mut version, |_| None);
    });
    assert_eq!(
        lookup_work.of_the_site(LookupSite::UnitsOfTheMappedRolesTakingTheirMappingPositions),
        LookupWork::default(),
        "一把有位置的 key 都没有：把映射位置换进 `units` 那一处不进、不建、不查"
    );
    assert!(
        version == before,
        "一把有位置的 key 都没有：这一版一个字节不变"
    );
}
