//! 发布路径与整理读版本按身份查（里程碑四七·6，`.claude/kb/milestone/04-fourth-txn.md`「七　GPU 垃圾回收」作业点表七·6 那一行）：
//! 照抄上一版的单元（`TransactionOutput::units_by_role`）、照抄角色的映射 key（`TransactionOutput::mapped_units_by_role`）、
//! 照抄的第一个文件之外的文件（`TransactionOutput::files_other_than_the_first_by_inode`）、整理读一版时逐 key 查中央映射
//! （`transaction::CentralMappingOfAVersionReadLeafByLeaf`）由在 `Vec` 上 `iter().find` 换成按身份建一次的索引。这里钉三样：
//! - 等价：两档池（十六个四单元的文件、四十八个八单元的文件，后一档的中央映射树不止一片叶）上跑同一串操作（按 inode 号写、整份重写四分之三的文件、
//!   抬 F、整理到停、整理之后再写、unlink 一个文件），每一版的每一次按身份查都与测试里的线性查找参照逐项相同，池级 checker 一条违例都没有；
//!   映射叶解不开、条目窄于字段表、同一把 key 有两条，三种叶上批量读与逐 key 查逐项相同。整理读每一版取的映射条目
//!   （`mounted_session::central_mapping_as_the_compaction_reads_it`）与逐 key 查的参照逐项相同（不漏、不错、次序同），批量读者解的叶
//!   恰好是进映射的 key 走到的那几片。
//! - 数的是产品调用点上做的事（七·6 代码轮第二轮判决第三节）：核心层在特性 `lookup-work-counters` 下按处记下按身份查了几次、每次看过
//!   索引几项、建了几次索引、解了几次中央映射叶（`singlefs_core::transaction::lookup_work`，harness 档的 dev 依赖打开这个特性），
//!   用例只读这些计数，不在测试里另建索引来数，不读挂钟、不读 CPU 时间。每一次用户改动、每一趟整理：每次查至多看折半查找那么多项；
//!   整份重写一个文件时快照判定那一圈与释放那三处都经索引查了换下的每个数据单元；整理每发一批，计划里每一项都经按 key 的索引查过；
//!   整理读一版恰好解进映射的 key 走到的那几片叶、各解一次，只建一次单元索引。
//! - 增长：表长四倍时每次查平均看过的索引项数涨不到门槛的倍数（合成表一条快档用例；512 与 2048 个数据单元的两档池上每一趟整理，
//!   harness 慢档用例），整理每一趟建索引的次数也涨不到门槛的倍数。
//!
//! 池都在内存里的两块 4 GiB 稀疏盘上造（`SparseBlockDevice`），配方照 E185 装置的 `construct_the_pool`：每个文件整份写进去，
//! 再整份重写下标不是 4 的倍数的那几个，抬 F 到换下的落点都回收，留下每段约四分之一还活着的聚簇段。
//!
//! 七·6 后续批（`research/prompts/m4-publish-lookup-followup-code-r1-main-verification.md` 第三节第 1–4 条）另钉：整理发批与恢复把映射位置换进
//! `units`、装一次发布要写与记录点名的单元、留给快照的角色清单两处、记录点名的数据角色那一处都经建一次的索引查；按 inode 号找文件、
//! 删快照取留存条目那几处在发布路径上建索引的次数恰好等于走了几遍；三棵多层码 2 树照抄的节点按树分开数；`singlefs-core` 构建脚本
//! 交出的特性只认 cargo 这一次真开的。
//!
//! 七·6 后续批代码轮第二轮判决（`research/prompts/m4-publish-lookup-followup-code-r2-main-verification.md` 第三节第 1–7 条、第五节用户定的三条）另钉：
//! 恢复重建一版把映射位置换进 `units` 每次重挂建一份、查有位置的进映射角色那么多次；恢复按 inode 号找 inode 记录与照抄第一个文件时问数据单元
//! 在不在这次重写的角色里各建一次索引；点名的几处之外（`None`）建的索引不随一次写的单元数涨；记账树照抄到不止一个节点时按树分开数；
//! 一把有位置的 key 都没有就不建；构建脚本真开着时报开着，`RUSTFLAGS` 的 `--cfg feature=…` 与它自己的 cfg 不符时停下。
//!
//! 已知边界（判决第三节第 4 条，不判）：计数只数索引上的动作（建、查、走），「索引照用、旁边再加一遍线性找」在计数里看不见——把映射位置换进
//! `units`、装要写的单元、装记录点名的单元、留给快照的角色清单两处、记录点名的数据角色那一处都是这样。判它要数产品里每一次身份比较
//! （给 `TransactionUnit` 的相等比较计数，碰全部调用点），或量挂钟（这份文件不读挂钟）；这一类写法是对抗写法，正常改代码不会在用了索引之后
//! 再逐个比一遍，所以登记在这里、不进 `crates/mutations.tsv`。攻方 NA4（记录点名的数据角色那一处索引之外再逐个 `contains`）、正推 ALT4
//! （留给快照的角色清单那一处同样）、正推 ALT5（映射位置换进 `units` 时索引之外再线性 `position` 一遍）三条都落在这一类，跑这份文件全绿。

//! 本档只核实现行为；同名场景交出的镜像由 checker 档给定状态引擎两段判。

mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/publish_lookup_by_identity.rs"]
mod scenarios;

#[test]
fn every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units(
) {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units);
}

#[test]
fn every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_forty_eight_files_of_eight_data_units(
) {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_forty_eight_files_of_eight_data_units);
}

#[test]
fn first_entry_of_each_identity_keeps_the_first_of_repeated_identities_and_misses_absent_ones() {
    scenarios::first_entry_of_each_identity_keeps_the_first_of_repeated_identities_and_misses_absent_ones();
}

#[test]
fn leaf_by_leaf_mapping_read_agrees_with_the_per_key_lookup_on_a_malformed_leaf_a_narrow_leaf_and_repeated_keys(
) {
    scenarios::leaf_by_leaf_mapping_read_agrees_with_the_per_key_lookup_on_a_malformed_leaf_a_narrow_leaf_and_repeated_keys();
}

#[test]
fn first_entry_of_each_identity_examines_less_than_twice_as_many_entries_per_lookup_on_a_table_four_times_as_long(
) {
    scenarios::first_entry_of_each_identity_examines_less_than_twice_as_many_entries_per_lookup_on_a_table_four_times_as_long();
}

#[test]
#[ignore = "harness 慢档用例：造 2048 个数据单元的池要发约 460 次发布，debug 下单条超过 60 秒"]
fn compaction_batches_decode_each_routed_mapping_leaf_once_and_examine_index_entries_sublinearly_from_512_to_2048_data_units(
) {
    scenarios::compaction_batches_decode_each_routed_mapping_leaf_once_and_examine_index_entries_sublinearly_from_512_to_2048_data_units();
}

#[test]
fn a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site(
) {
    scenarios::a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site();
}

#[test]
fn a_whole_rewrite_under_a_snapshot_and_its_destruction_look_up_kept_mapping_entries_through_indexes_built_once(
) {
    scenarios::a_whole_rewrite_under_a_snapshot_and_its_destruction_look_up_kept_mapping_entries_through_indexes_built_once();
}

#[test]
fn relocation_batches_of_two_and_of_eight_units_build_their_indexes_once_and_look_up_each_planned_unit_twice(
) {
    scenarios::relocation_batches_of_two_and_of_eight_units_build_their_indexes_once_and_look_up_each_planned_unit_twice();
}

#[test]
fn first_entry_of_each_identity_keeps_the_first_entry_of_identities_repeated_across_a_long_shuffled_table(
) {
    scenarios::first_entry_of_each_identity_keeps_the_first_entry_of_identities_repeated_across_a_long_shuffled_table();
}

#[test]
fn lookup_work_counts_one_entry_per_step_of_a_walk_from_either_end_and_one_lookup_per_search() {
    scenarios::lookup_work_counts_one_entry_per_step_of_a_walk_from_either_end_and_one_lookup_per_search();
}

#[test]
fn rewrites_of_eight_and_of_thirty_two_data_units_under_a_snapshot_with_a_deadlist_of_several_leaves_build_one_index_per_pass_and_count_each_tree_apart(
) {
    scenarios::rewrites_of_eight_and_of_thirty_two_data_units_under_a_snapshot_with_a_deadlist_of_several_leaves_build_one_index_per_pass_and_count_each_tree_apart();
}

#[test]
fn whole_rewrites_carrying_a_first_file_of_eight_and_of_sixty_four_data_units_ask_for_each_of_its_data_units_through_one_index_of_the_rewritten_roles(
) {
    scenarios::whole_rewrites_carrying_a_first_file_of_eight_and_of_sixty_four_data_units_ask_for_each_of_its_data_units_through_one_index_of_the_rewritten_roles();
}

#[test]
fn remounting_pools_of_four_and_of_sixteen_files_builds_one_index_for_the_mapping_positions_and_one_for_the_inode_records(
) {
    scenarios::remounting_pools_of_four_and_of_sixteen_files_builds_one_index_for_the_mapping_positions_and_one_for_the_inode_records();
}

#[test]
fn taking_the_mapping_positions_without_any_key_that_has_a_position_builds_no_index_and_leaves_the_version_unchanged(
) {
    scenarios::taking_the_mapping_positions_without_any_key_that_has_a_position_builds_no_index_and_leaves_the_version_unchanged();
}

#[test]
fn the_core_build_script_reports_the_lookup_work_counters_feature_that_cargo_enabled_for_this_test_package(
) {
    scenarios::the_core_build_script_reports_the_lookup_work_counters_feature_that_cargo_enabled_for_this_test_package();
}

#[test]
fn the_core_build_script_finds_the_features_its_rustflags_open_while_its_own_cfg_says_off() {
    scenarios::the_core_build_script_finds_the_features_its_rustflags_open_while_its_own_cfg_says_off();
}

#[test]
fn the_core_build_script_reports_only_the_features_cargo_enabled_even_with_the_feature_variable_brought_in_from_outside(
) {
    scenarios::the_core_build_script_reports_only_the_features_cargo_enabled_even_with_the_feature_variable_brought_in_from_outside();
}
