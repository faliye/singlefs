//! 退役记录与谱系记录：两棵树的条目、记录的形式与完整性、容器的记录宽度，谱系记录里回退前的 inode 号水位与水位步进行。

use super::*;

/// 退役树那五条（I-9.31–I-9.35）。
const RETIREMENT_INVARIANTS: [&str; 5] = [
    RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
];

/// 三个单元的文件按批大小 1 删到底（意图、三批）：最后一批删掉 inode 1 的记录，inode 树唯一那片叶容器空了、身份消失，
/// 退役树开第一片容器、写一条 (那片叶的身份, 最后一批的 txg)（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 3 条）。
/// 交回池、删之前那片叶的身份与最后一批那次发布。
fn pool_with_the_three_unit_file_unlinked_to_the_end(
    tag: &str,
) -> (
    BuiltPool,
    singlefs_core::unit::PackedIdentity,
    TransactionOutput,
) {
    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(tag);
    let intent = common_deletion_in_progress::publish_the_intent(&mut pool);
    let [leaf] = intent.inode_leaf_containers.as_slice() else {
        panic!("意图那一版 inode 树一片叶容器")
    };
    let leaf_identity = leaf.contents.identity;
    // 迭代上界：三个数据单元按批大小 1 删，三批删到底。
    for _ in 0..2 {
        common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool);
    }
    let final_batch =
        common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool);
    assert!(final_batch.deleted_inodes.is_empty(), "第三批是最后一批");
    (pool, leaf_identity, final_batch)
}

/// 这一版退役树按叶序的全部退役记录。
fn retirement_records_of(output: &TransactionOutput) -> Vec<RetirementRecord> {
    output
        .retirement_containers
        .iter()
        .flat_map(|container| container.contents.records.iter().copied())
        .collect()
}

/// 写入那一侧与 checker 那一侧对着同一份干净镜像：唯一那片叶容器在最后一批身份消失，那次发布往退役树写恰好一条
/// (那片叶的身份, 最后一批的 txg)；之前各版退役树零根。checker 上退役树那五条都真被评估过且成立，别的条也不红
/// （判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 3、8 条）。
#[test]
fn the_final_batch_that_empties_the_only_leaf_writes_one_retirement_record_and_every_retirement_invariant_holds(
) {
    let (pool, leaf_identity, final_batch) =
        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-clean");
    assert!(
        final_batch.inode_leaf_containers.is_empty(),
        "最后一批之后 inode 树零条目根"
    );
    assert_eq!(
        retirement_records_of(&final_batch),
        [RetirementRecord {
            retired: leaf_identity,
            death_txg: final_batch.root.checkpoint_txg,
        }],
        "身份消失的那次发布写一条退役记录，死亡 txg = 这次发布的 txg"
    );
    let verdicts = check_pool_image(&pool.memory_pool());
    let retirement_verdicts: Vec<(&str, InvariantVerdict)> = RETIREMENT_INVARIANTS
        .iter()
        .map(|invariant| (*invariant, verdict_among(&verdicts, invariant)))
        .collect();
    assert_eq!(
        retirement_verdicts,
        RETIREMENT_INVARIANTS
            .iter()
            .map(|invariant| (*invariant, InvariantVerdict::Holds))
            .collect::<Vec<_>>(),
        "退役树那五条都真被评估过且成立"
    );
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "干净镜像上一条都不红：{verdicts:?}"
    );
}

/// I-9.31（退役树条目身份与子头相符） 只红自己的那一形：删到底那一份上，退役树根第一条条目身份引用里的容器号加一、容器头不动。
#[test]
fn a_retirement_tree_entry_whose_identity_names_another_container_reddens_only_the_retirement_entry_invariant(
) {
    let (pool, _, _) =
        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-entry-identity");
    let mut image = pool.memory_pool();
    change_the_first_entry_of_the_container_tree_root_of_kind(
        &mut image,
        TREE_KIND_RETIREMENT,
        add_one_to_the_identity_container_number,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "退役树根条目的身份与容器头不符：只红 I-9.31：{verdicts:?}"
    );
}

/// 删到底之后建 1 个空 inode（新开一片叶容器，身份出生代 = 那次发布的 txg），再建 [`common_deletion_in_progress::PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER`]
/// 次：删的那几版与建新叶那几版都转出回退候选集（根环一圈 24 槽）。退役树那片容器在删到底那一版之后再没重写过。
/// 交回池、新叶的身份、建新叶那次发布的 txg。
fn pool_with_the_file_unlinked_to_the_end_a_new_leaf_and_the_root_ring_turned_over(
    tag: &str,
) -> (
    BuiltPool,
    singlefs_core::unit::PackedIdentity,
    CheckpointTxg,
) {
    let (mut pool, _, _) = pool_with_the_three_unit_file_unlinked_to_the_end(tag);
    let new_leaf_version =
        common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let [new_leaf] = new_leaf_version.inode_leaf_containers.as_slice() else {
        panic!("建 1 个空 inode 之后 inode 树一片叶容器")
    };
    let new_leaf_identity = new_leaf.contents.identity;
    common_deletion_in_progress::turn_the_root_ring_over_by_creating_inodes(&mut pool);
    (
        pool,
        new_leaf_identity,
        new_leaf_version.root.checkpoint_txg,
    )
}

/// 根环一圈 3 区 × 8 槽：最新那一版往前数这么多版之外的都不在回退候选集里。
const ROOTS_IN_THE_RING: u64 = 24;

/// I-9.32（退役记录合法） 只红自己的那一形：删的那几版转出候选集之后（I-9.35 在记录死的那一版上不再比），那条退役记录被退役身份的
/// 打包记录类型 2 → 1。I-9.33、I-9.34 按身份比，类型 1 的身份既不重也不是活叶，照旧成立。
#[test]
fn a_retirement_record_naming_a_container_of_record_type_one_reddens_only_the_well_formed_retirement_record_invariant(
) {
    let (pool, _, _) =
        pool_with_the_file_unlinked_to_the_end_a_new_leaf_and_the_root_ring_turned_over(
            "known-bad-retirement-record-type",
        );
    let clean = pool.memory_pool();
    let clean_verdicts = check_pool_image(&clean);
    assert_eq!(
        verdict_among(&clean_verdicts, EVERY_RETIREMENT_RECORD_IS_WELL_FORMED),
        InvariantVerdict::Holds,
        "改之前 I-9.32 真被评估过且成立"
    );
    assert_eq!(violated_invariants(&clean_verdicts), Vec::<&str>::new());
    let mut image = clean.clone();
    common_deletion_in_progress::change_the_first_retirement_record_of_the_newest_root(
        &mut image,
        |record| common_deletion_in_progress::set_the_retired_record_type(record, 1),
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [EVERY_RETIREMENT_RECORD_IS_WELL_FORMED],
        "被退役身份的类型不是 2：只红 I-9.32：{verdicts:?}"
    );
}

/// I-9.33（谱系上的退役记录按身份不重复） 只红自己的那一形：删到底那一份上，退役容器里那条记录原样再追加一条。
/// I-9.35 按身份集比，两条同身份收成一个，照旧成立。
#[test]
fn a_second_copy_of_the_retirement_record_reddens_only_the_distinct_retirement_records_invariant() {
    let (pool, _, _) =
        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-record-copy");
    let mut image = pool.memory_pool();
    common_deletion_in_progress::append_a_retirement_record_to_the_first_container_of_the_newest_root(
        &mut image,
        <[u8]>::to_vec,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT],
        "谱系上两条退役记录指同一个身份：只红 I-9.33：{verdicts:?}"
    );
}

/// I-9.34（谱系上的退役记录不指本版活叶） 只红自己的那一形：删的那几版与建新叶那几版转出候选集之后，那条退役记录改成指新叶的身份、
/// 死亡 txg 改成建新叶那次发布的 txg + 1（在每一版父链上、大于新叶的容器出生代、不是候选集里哪一版的 txg）。I-9.32 的几道照旧成立，
/// I-9.35 在死亡 txg 那一版与它的子版上都比不了（父版不在候选集里），只红 I-9.34。
#[test]
fn a_retirement_record_naming_the_live_leaf_reddens_only_the_no_live_leaf_invariant() {
    let (pool, new_leaf_identity, new_leaf_txg) =
        pool_with_the_file_unlinked_to_the_end_a_new_leaf_and_the_root_ring_turned_over(
            "known-bad-retirement-record-live-leaf",
        );
    let death_txg = new_leaf_txg.0 + 1;
    assert!(
        death_txg + ROOTS_IN_THE_RING <= pool.output.root.checkpoint_txg.0,
        "死亡 txg 那一版与它的子版都已转出候选集"
    );
    let new_leaf_identity_bytes = new_leaf_identity_in_record_order(new_leaf_identity);
    let mut image = pool.memory_pool();
    assert_eq!(
        common_deletion_in_progress::identity_of_the_first_inode_leaf_of_the_newest_root(&image),
        new_leaf_identity_bytes,
        "最新那一版 inode 树唯一那片叶就是建新叶那次开的"
    );
    common_deletion_in_progress::change_the_first_retirement_record_of_the_newest_root(
        &mut image,
        |record| {
            record[..common_deletion_in_progress::RETIREMENT_RECORD_IDENTITY_BYTES]
                .copy_from_slice(&new_leaf_identity_bytes);
            common_deletion_in_progress::set_retirement_record_field(
                record,
                common_deletion_in_progress::RETIREMENT_RECORD_DEATH_TXG_OFFSET,
                death_txg,
            );
        },
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF],
        "谱系上的退役记录指这一版的活叶：只红 I-9.34：{verdicts:?}"
    );
    // 退役树与这一版 inode 树点名同一片叶容器：I-1.13 不另报一条违例，报不适用、理由点名 I-9.34。这一张镜像上那片叶在死亡 txg 之后又写过一版，
    // 是扫描方向看得见的那一形（`rebuild::ScanRebuildNotDecided::RetiredLeafContainerHasAVersionBornAfterItsDeath` 的理由点名 I-9.34）；
    // 看不见的那一形（叶死后没再写过）进 `walk::INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED`，
    // 它的镜像在下一条用例里（`a_retirement_record_naming_the_live_leaf_the_scan_cannot_see_…`）。
    match verdict(&image, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX) {
        InvariantVerdict::NotApplicable(reason) => assert!(
            reason.contains("I-9.34"),
            "I-1.13 报不适用、理由点名 I-9.34，实际 {reason}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("I-9.34 红时 I-1.13 报不适用（用户 2026-10-14 定乙），实际 {other:?}")
        }
    }
}

/// 退役树与这一版 inode 树点名同一片叶容器、扫描方向看不见的那一形：
/// 同一份删到底、建新叶、根环转过一圈的镜像上，那条退役记录改指最新那一版唯一那片叶，死亡 txg 取最新那一版的 txg——那片叶择出的那一版
/// 诞生代号不晚于死亡 txg（叶死后没再写过），扫描重建按退役记录不认领它、不报「条款没写」，遍历方向照样读得到它的记录。
/// I-9.34 红（I-9.35 在最新那一版上跟着红：死于这一版的记录点名的叶并没有从父版消失）；I-1.13 不另报一条违例，报不适用、
/// 理由是遍历方向损坏表里 I-9.34 那一句（`walk::INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED`），不是扫描重建自己的不适用。
#[test]
fn a_retirement_record_naming_the_live_leaf_the_scan_cannot_see_leaves_the_scan_rebuild_invariant_not_applicable_naming_the_no_live_leaf_invariant(
) {
    let (pool, _, _) =
        pool_with_the_file_unlinked_to_the_end_a_new_leaf_and_the_root_ring_turned_over(
            "known-bad-retirement-record-live-leaf-the-scan-cannot-see",
        );
    let newest_txg = pool.output.root.checkpoint_txg.0;
    let mut image = pool.memory_pool();
    let live_leaf_identity_bytes =
        common_deletion_in_progress::identity_of_the_first_inode_leaf_of_the_newest_root(&image);
    common_deletion_in_progress::change_the_first_retirement_record_of_the_newest_root(
        &mut image,
        |record| {
            record[..common_deletion_in_progress::RETIREMENT_RECORD_IDENTITY_BYTES]
                .copy_from_slice(&live_leaf_identity_bytes);
            common_deletion_in_progress::set_retirement_record_field(
                record,
                common_deletion_in_progress::RETIREMENT_RECORD_DEATH_TXG_OFFSET,
                newest_txg,
            );
        },
    );
    let verdicts = check_pool_image(&image);
    assert!(
        violated_invariants(&verdicts)
            .contains(&RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF),
        "谱系上的退役记录指这一版的活叶：I-9.34 红：{verdicts:?}"
    );
    match verdict(&image, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX) {
        InvariantVerdict::NotApplicable(reason) => assert!(
            reason.contains("I-9.34") && reason.contains("扫描重建按退役记录不认领那片叶"),
            "I-1.13 报不适用、理由是遍历方向损坏表里 I-9.34 那一句（扫描方向看不见，不是扫描重建自己停下），实际 {reason}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("I-9.34 红时 I-1.13 报不适用（用户 2026-10-14 定乙），实际 {other:?}")
        }
    }
}

/// 一个打包记录容器身份按退役记录里被退役身份那 26 字节排（出生树 8、类型 2、容器号 8、容器出生代 8，小端）。
fn new_leaf_identity_in_record_order(identity: singlefs_core::unit::PackedIdentity) -> Vec<u8> {
    RetirementRecord {
        retired: identity,
        death_txg: CheckpointTxg(0),
    }
    .to_bytes()[..common_deletion_in_progress::RETIREMENT_RECORD_IDENTITY_BYTES]
        .to_vec()
}

/// I-9.35（退役完备）漏写那一形，只红自己：删到底那一份上，那条退役记录的死亡 txg 往前挪一（最后一批 → 第二批的 txg）。
/// 最后一批那一版比父版少了那片叶、死于这一版的记录一条都没有；第二批那一版看不到这片容器（退役树那时零根）；I-9.32 的几道照旧成立。
#[test]
fn a_retirement_record_moved_off_the_txg_the_leaf_died_at_reddens_only_the_retirement_completeness_invariant(
) {
    let (pool, _, final_batch) =
        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-record-missing");
    let mut image = pool.memory_pool();
    common_deletion_in_progress::change_the_first_retirement_record_of_the_newest_root(
        &mut image,
        |record| {
            common_deletion_in_progress::set_retirement_record_field(
                record,
                common_deletion_in_progress::RETIREMENT_RECORD_DEATH_TXG_OFFSET,
                final_batch.root.checkpoint_txg.0 - 1,
            );
        },
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION],
        "身份消失的那一版没记退役：只红 I-9.35：{verdicts:?}"
    );
}

/// I-9.35（退役完备）多写那一形，只红自己：删到底那一份上，退役容器追加一条与那条同死亡 txg、容器号加一（这一版父版里没有的身份）。
/// I-9.33 按身份比不重、I-9.34 这一版没有活叶，照旧成立。
#[test]
fn a_retirement_record_naming_a_leaf_the_parent_version_did_not_have_reddens_only_the_retirement_completeness_invariant(
) {
    let (pool, _, _) =
        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-record-extra");
    let mut image = pool.memory_pool();
    common_deletion_in_progress::append_a_retirement_record_to_the_first_container_of_the_newest_root(
        &mut image,
        |first| {
            let mut extra = first.to_vec();
            let container = common_deletion_in_progress::retirement_record_field(
                &extra,
                common_deletion_in_progress::RETIREMENT_RECORD_CONTAINER_OFFSET,
            );
            common_deletion_in_progress::set_retirement_record_field(
                &mut extra,
                common_deletion_in_progress::RETIREMENT_RECORD_CONTAINER_OFFSET,
                container + 1,
            );
            extra
        },
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION],
        "死于这一版的退役记录指父版里没有的身份：只红 I-9.35：{verdicts:?}"
    );
}

/// 退役记录里被退役身份的容器出生代那 8 字节的偏移（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 2 条：出生树 0、
/// 打包记录类型 8、容器号 10、容器出生代 18、死亡 txg 26）。
const RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET: usize = 18;

/// I-9.32（退役记录合法）「容器出生代 < 死亡 txg」那一句只红自己（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F4，
/// 攻方第二次抽样 K7 那条变异）：删的那几版转出候选集之后（I-9.35 在记录死的那一版上不再比），那条退役记录被退役身份的容器出生代改成它自己的
/// 死亡 txg。改了出生代的身份既不重也不是活叶，I-9.33、I-9.34 照旧成立。
#[test]
fn a_retirement_record_whose_container_birth_is_not_below_its_death_txg_reddens_only_the_well_formed_retirement_record_invariant(
) {
    let (pool, _, _) =
        pool_with_the_file_unlinked_to_the_end_a_new_leaf_and_the_root_ring_turned_over(
            "known-bad-retirement-record-born-at-its-death",
        );
    let mut image = pool.memory_pool();
    common_deletion_in_progress::change_the_first_retirement_record_of_the_newest_root(
        &mut image,
        |record| {
            let death_txg = common_deletion_in_progress::retirement_record_field(
                record,
                common_deletion_in_progress::RETIREMENT_RECORD_DEATH_TXG_OFFSET,
            );
            common_deletion_in_progress::set_retirement_record_field(
                record,
                RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET,
                death_txg,
            );
        },
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [EVERY_RETIREMENT_RECORD_IS_WELL_FORMED],
        "被退役身份的容器出生代不小于死亡 txg：只红 I-9.32：{verdicts:?}"
    );
}

/// I-9.32「死亡 txg ≤ 这一版的 txg」那一句只红自己（同上）：同一份基底上那条退役记录的死亡 txg 改成最新那一版的 txg + 1。
/// 每一版里它都晚于那一版、不在谱系上，I-9.33、I-9.34 不判它；没有一版的 txg 等于它，I-9.35 照旧成立。
#[test]
fn a_retirement_record_dying_after_the_newest_version_reddens_only_the_well_formed_retirement_record_invariant(
) {
    let (pool, _, _) =
        pool_with_the_file_unlinked_to_the_end_a_new_leaf_and_the_root_ring_turned_over(
            "known-bad-retirement-record-dies-after-the-newest-version",
        );
    let death_txg_after_the_newest_version = pool.output.root.checkpoint_txg.0 + 1;
    let mut image = pool.memory_pool();
    common_deletion_in_progress::change_the_first_retirement_record_of_the_newest_root(
        &mut image,
        |record| {
            common_deletion_in_progress::set_retirement_record_field(
                record,
                common_deletion_in_progress::RETIREMENT_RECORD_DEATH_TXG_OFFSET,
                death_txg_after_the_newest_version,
            );
        },
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [EVERY_RETIREMENT_RECORD_IS_WELL_FORMED],
        "死亡 txg 晚于每一版：只红 I-9.32：{verdicts:?}"
    );
}

/// I-1.7（打包容器合法与判定顺序） 记录宽那一步对类型 3（退役，登记 34）：三个单元的文件删到底之后，
/// 退役容器头里的记录宽 34 → 33。墓碑与谱系那一遍读退役容器时遇到记录宽不是登记的那一个，退役那几条在那一版判不了、不判红：只红 I-1.7。
#[test]
fn a_retirement_container_whose_record_width_is_not_thirty_four_reddens_only_the_packed_container_invariant(
) {
    let (pool, _, _) =
        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-record-width");
    let mut image = pool.memory_pool();
    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
        &mut image,
        TREE_KIND_RETIREMENT,
        u16::try_from(singlefs_format::RETIREMENT_RECORD_BYTES - 1).expect("33"),
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-1.7"],
        "退役容器的记录宽不是 34：只红 I-1.7：{verdicts:?}"
    );
}

/// 写入口装 `capacities`（只供测试的容器树旋钮），把空 inode `inode` unlink 到底：意图那一次，再走空（空 inode 没有数据单元，直接是最后一批）。
pub(super) fn unlink_an_empty_inode_to_the_end_under_the_capacities(
    pool: &mut BuiltPool,
    inode: u64,
    capacities: singlefs_core::container_tree::ContainerTreeCapacities,
) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    writer.set_container_tree_capacities(capacities);
    let current = singlefs_core::transaction::publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        singlefs_core::address::InodeNumber(inode),
        common_deletion_in_progress::WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        common_deletion_in_progress::INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
    let mut current = std::sync::Arc::new(current);
    singlefs_core::transaction::publish_unlink_steps_until_the_queue_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
        common_deletion_in_progress::INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的各批：{refusal:?}"));
    pool.output = std::sync::Arc::unwrap_or_clone(current);
}

/// 写这几份退役树时退役容器压到的每片记录数（谱系、墓碑与节点照产品）。
const RETIREMENT_RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;

/// 新池新建文件之外再建 233 个空 inode（叶 [1..=233]、[234]），写入口装退役容器一片 1 条的旋钮：unlink 234 到底（最右那片删空，退役一条），
/// 再建 1 个（235，最左那片满了，开新叶）、unlink 235 到底（又退役一条）。退役树两片容器各 1 条——攻方 `opus_probe_container_closed_early`
/// 退役那一形（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第二节 K3-4）。旋钮下的合法镜像。
fn pool_with_two_leaves_retired_one_record_per_retirement_container(tag: &str) -> BuiltPool {
    let capacities = singlefs_core::container_tree::ContainerTreeCapacities {
        tombstone_container:
            singlefs_core::tombstone_tree::TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container:
            singlefs_core::lineage_tree::LineageContainerCapacity::FromTheContainerFormat,
        retirement_container:
            singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
                singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(
                    RETIREMENT_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
                )
                .expect("1..=959"),
            ),
        node_entries: singlefs_core::container_tree::ContainerTreeNodeEntries::FromTheNodeFormat,
    };
    let mut pool = common_deletion_in_progress::pool_with_new_inodes(tag, 233);
    unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, 234, capacities);
    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, 235, capacities);
    pool
}

/// I-9.36（产品镜像里非最右谱系与退役容器装满）退役树那一形：旋钮压小写出的两片各 1 条的退役树，交给池级 checker 的产品入口
/// （当没装旋钮），不是最右的那片只装 1 条、不是 959：只红 I-9.36（墓碑树同形红 I-9.21）；交给带旋钮值的入口（退役一片 1 条）一条都不红、
/// I-9.37 真被评估过且成立。
#[test]
fn two_leaves_retired_one_record_per_retirement_container_judged_as_a_product_image_reddens_only_the_full_container_invariant(
) {
    let pool = pool_with_two_leaves_retired_one_record_per_retirement_container(
        "known-bad-retirement-container-closed-early",
    );
    assert_eq!(
        pool.output
            .retirement_containers
            .iter()
            .map(|container| container.contents.records.len())
            .collect::<Vec<_>>(),
        [1, 1],
        "退役树两片容器各 1 条"
    );
    let image = pool.memory_pool();
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE],
        "产品入口：不是最右的退役容器只装 1 条：只红 I-9.36：{verdicts:?}"
    );
    let verdicts_given_the_knob =
        singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
            &image,
            singlefs_checker::walk::ContainerCapacityKnobs {
                tombstone: None,
                lineage: None,
                retirement: Some(
                    singlefs_checker::walk::RetirementContainerCapacityKnob::new(
                        RETIREMENT_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
                    )
                    .expect("1..=959"),
                ),
            },
        );
    assert_eq!(
        violated_invariants(&verdicts_given_the_knob),
        Vec::<&str>::new(),
        "带旋钮值的入口：旋钮下的合法镜像一条都不红：{verdicts_given_the_knob:?}"
    );
    assert_eq!(
        verdict_among(
            &verdicts_given_the_knob,
            LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE
        ),
        InvariantVerdict::Holds,
        "I-9.37 在退役树上真被评估过且成立"
    );
}

/// 新池新建文件之后建 1 个空 inode（txg 4），分配器装上谱系容器一片 `lineage_records_per_container` 条的旋钮（回退的发布路径照分配器上的开关取容量），
/// 挂着回退到第一个文件那一版两次：谱系树两条记录。攻方 `opus_probe_container_closed_early` 谱系那一形。
fn pool_rolled_back_twice_under_the_lineage_knob(
    tag: &str,
    lineage_records_per_container: u64,
) -> BuiltPool {
    let mut pool = build_pool(tag);
    let first_file_version = pool.output.clone();
    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    pool.allocator
        .set_deletion_switches(singlefs_core::unlink::DeletionSwitches {
            batch_size: singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
            tombstone_container_capacity:
                singlefs_core::tombstone_tree::TombstoneContainerCapacity::FromTheContainerFormat,
            lineage_container_capacity:
                singlefs_core::lineage_tree::LineageContainerCapacity::CappedForTests(
                    singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(
                        lineage_records_per_container,
                    )
                    .expect("1..=1359"),
                ),
            retirement_container_capacity:
                singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
            container_tree_node_entries:
                singlefs_core::container_tree::ContainerTreeNodeEntries::FromTheNodeFormat,
        });
    common_deletion_in_progress::roll_back_to(&mut pool, &first_file_version);
    common_deletion_in_progress::roll_back_to(&mut pool, &first_file_version);
    pool
}

/// 谱系容器一片 1 条时写这几份谱系树。
const LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;

fn lineage_record_counts_of_each_container(pool: &BuiltPool) -> Vec<usize> {
    pool.output
        .lineage_containers
        .iter()
        .map(|container| container.contents.records.len())
        .collect()
}

fn check_pool_image_given_the_lineage_knob(
    image: &MemoryPool,
    lineage_records_per_container: u64,
) -> Vec<(&'static str, InvariantVerdict)> {
    singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
        image,
        singlefs_checker::walk::ContainerCapacityKnobs {
            tombstone: None,
            lineage: Some(
                singlefs_checker::walk::LineageContainerCapacityKnob::new(
                    lineage_records_per_container,
                )
                .expect("1..=1359"),
            ),
            retirement: None,
        },
    )
}

/// I-9.36 谱系树那一形：旋钮压小写出的两片各 1 条的谱系树，产品入口判不是最右的那片只装 1 条、不是 1359：只红 I-9.36；
/// 带旋钮值的入口（谱系一片 1 条）一条都不红、I-9.37 真被评估过且成立。
#[test]
fn two_rollbacks_one_record_per_lineage_container_judged_as_a_product_image_reddens_only_the_full_container_invariant(
) {
    let pool = pool_rolled_back_twice_under_the_lineage_knob(
        "known-bad-lineage-container-closed-early",
        LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
    );
    assert_eq!(
        lineage_record_counts_of_each_container(&pool),
        [1, 1],
        "谱系树两片容器各 1 条"
    );
    let image = pool.memory_pool();
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE],
        "产品入口：不是最右的谱系容器只装 1 条：只红 I-9.36：{verdicts:?}"
    );
    let verdicts_given_the_knob = check_pool_image_given_the_lineage_knob(
        &image,
        LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
    );
    assert_eq!(
        violated_invariants(&verdicts_given_the_knob),
        Vec::<&str>::new(),
        "带旋钮值的入口：旋钮下的合法镜像一条都不红：{verdicts_given_the_knob:?}"
    );
    assert_eq!(
        verdict_among(
            &verdicts_given_the_knob,
            LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE
        ),
        InvariantVerdict::Holds,
        "I-9.37 在谱系树上真被评估过且成立"
    );
}

/// I-9.37（测试镜像里谱系与退役容器不超旋钮值） 只红自己：谱系容器一片 2 条写出的谱系树（一片 2 条），告诉带旋钮值的入口旋钮是 1：
/// 那片装了 2 条、多于旋钮值，只红 I-9.37（照 I-9.22 那一条的坏镜像）。产品入口上只一片容器，I-9.36 没有对象，一条都不红。
#[test]
fn a_lineage_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant()
{
    let pool = pool_rolled_back_twice_under_the_lineage_knob(
        "known-bad-lineage-container-past-the-knob",
        2,
    );
    assert_eq!(
        lineage_record_counts_of_each_container(&pool),
        [2],
        "谱系树一片容器 2 条"
    );
    let image = pool.memory_pool();
    assert_eq!(
        violated_invariants(&check_pool_image(&image)),
        Vec::<&str>::new(),
        "产品入口：只一片谱系容器，一条都不红"
    );
    let verdicts = check_pool_image_given_the_lineage_knob(
        &image,
        LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
    );
    assert_eq!(
        violated_invariants(&verdicts),
        [LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE],
        "谱系容器装了 2 条、旋钮值 1：只红 I-9.37：{verdicts:?}"
    );
}

/// 合法的回退不红（回退照抄现行那一版的退役树、I-9.33 只比谱系上的）：删到底（退役一条 (叶, F)）→
/// 回退到意图那一版（那片叶又活了，退役树照抄、仍是那一条）→ 再删到底（同一片叶再死一次，追加 (叶, F_again)）。
/// 两条同身份，但 F 不在 F_again 那一版的父链上；每一版 checker 都一条不红，退役树那五条都成立。
#[test]
fn unlinking_rolling_back_and_unlinking_again_writes_a_second_record_of_the_same_leaf_and_no_invariant_reddens(
) {
    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(
        "known-bad-retirement-rollback-and-again",
    );
    let intent = common_deletion_in_progress::publish_the_intent(&mut pool);
    let [leaf] = intent.inode_leaf_containers.as_slice() else {
        panic!("意图那一版 inode 树一片叶容器")
    };
    let leaf_identity = leaf.contents.identity;
    // 迭代上界：三批删到底。
    let mut first_final_batch = intent.clone();
    for _ in 0..3 {
        first_final_batch =
            common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool);
    }
    let rollback = common_deletion_in_progress::roll_back_to(&mut pool, &intent);
    assert_eq!(
        retirement_records_of(&rollback),
        retirement_records_of(&first_final_batch),
        "回退那一版退役树照抄现行那一版"
    );
    assert_eq!(
        rollback
            .inode_leaf_containers
            .iter()
            .map(|container| container.contents.identity)
            .collect::<Vec<_>>(),
        [leaf_identity],
        "回退之后那片叶又在 inode 树里"
    );
    assert_eq!(
        violated_invariants(&check_pool_image(&pool.memory_pool())),
        Vec::<&str>::new(),
        "回退那一版一条都不红"
    );
    // 迭代上界：三批删到底。
    let mut second_final_batch = rollback.clone();
    for _ in 0..3 {
        second_final_batch =
            common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool);
    }
    assert!(second_final_batch.deleted_inodes.is_empty(), "再删也是三批");
    assert_eq!(
        retirement_records_of(&second_final_batch),
        [
            RetirementRecord {
                retired: leaf_identity,
                death_txg: first_final_batch.root.checkpoint_txg,
            },
            RetirementRecord {
                retired: leaf_identity,
                death_txg: second_final_batch.root.checkpoint_txg,
            },
        ],
        "同一片叶死第二次再追加一条，第一条不动"
    );
    let verdicts = check_pool_image(&pool.memory_pool());
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "再删到底那一版一条都不红：{verdicts:?}"
    );
    for invariant in RETIREMENT_INVARIANTS {
        assert_eq!(
            verdict_among(&verdicts, invariant),
            InvariantVerdict::Holds,
            "{invariant} 真被评估过且成立"
        );
    }
}

/// I-9.35 的父版取谱系（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 8 条、正推 K5.1：这一版谱系树里有 T_rb 等于
/// 这一版 txg 的记录时父版是那条的 T_old，否则 txg − 1）：删到底（inode 树零条目根）之后建 1 个空 inode（开一片新叶），再回退到删到底那一版，
/// 回退那一版没有那片新叶。父版取 T_old（删到底那一版，同样没有叶），没有叶身份消失、也没有退役记录；按 txg − 1 取父版会拿建新叶那一版比、
/// 判它漏记一片。回退那一版一条都不红，I-9.35 真被评估过且成立。
#[test]
fn rolling_back_past_the_birth_of_a_leaf_compares_with_the_rollback_target_and_no_invariant_reddens(
) {
    let (mut pool, _, final_batch) = pool_with_the_three_unit_file_unlinked_to_the_end(
        "known-bad-retirement-rollback-past-a-leaf-birth",
    );
    let new_leaf_version =
        common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    assert_eq!(
        new_leaf_version.inode_leaf_containers.len(),
        1,
        "建 1 个空 inode 开一片新叶"
    );
    let rollback = common_deletion_in_progress::roll_back_to(&mut pool, &final_batch);
    assert!(
        rollback.inode_leaf_containers.is_empty(),
        "回退那一版 inode 树零条目根"
    );
    assert_eq!(
        retirement_records_of(&rollback),
        retirement_records_of(&final_batch),
        "回退照抄现行那一版的退役树，不写退役记录"
    );
    let verdicts = check_pool_image(&pool.memory_pool());
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "回退到新叶出生之前那一版一条都不红：{verdicts:?}"
    );
    assert_eq!(
        verdict_among(
            &verdicts,
            RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION
        ),
        InvariantVerdict::Holds,
        "I-9.35 真被评估过且成立"
    );
}

/// I-9.27 种类 13（退役树；头 ID 写 inode 树的树 ID，同墓碑树）：删到底那一份上，最新那一版树表里退役树那一条的头 ID 从 inode 树的
/// 树 ID 改成 0、配不上任何一条 inode 树条目。只红 I-9.27；退役记录对叶容器的那两判（I-9.34、I-9.35）在那一版判不了、理由写头 ID 配不上，
/// 不按「这棵 inode 树没有退役记录」判漏记。
#[test]
fn a_retirement_tree_entry_whose_head_identifier_names_no_inode_tree_reddens_only_the_head_identifier_invariant(
) {
    let (pool, _, _) =
        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-head-zero");
    let mut image = pool.memory_pool();
    let tree_table_slot = common_deletion_in_progress::tree_table_slot_of_the_newest_root(&image);
    common_deletion_in_progress::set_the_head_identifier_of_the_tree_table_entry_of_kind_at(
        &mut image,
        tree_table_slot,
        TREE_KIND_RETIREMENT,
        0,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES],
        "退役树条目的头 ID 配不上 inode 树：只红 I-9.27：{verdicts:?}"
    );
    for invariant in [
        RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
        RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
    ] {
        let found = verdict_among(&verdicts, invariant);
        assert!(
            matches!(&found, InvariantVerdict::PartiallyJudged(unjudged) if unjudged.contains("头 ID 配不上")),
            "{invariant} 在头 ID 配不上的那一版判不了：{found:?}"
        );
    }
}

/// I-9.17（谱系树条目身份与子头相符） 只红自己的那一形：挂着的时候回退到 A 之后（回退那次发布写出谱系树），谱系树根第一条条目身份引用里的
/// 容器号加一、容器头不动。
#[test]
fn a_lineage_tree_entry_whose_identity_names_another_container_reddens_only_the_lineage_entry_invariant(
) {
    let rolled_back =
        image_after_rolling_back_to_the_first_root("known-bad-lineage-entry-identity");
    let mut image = rolled_back.image.clone();
    change_the_first_entry_of_the_container_tree_root_of_kind(
        &mut image,
        TREE_KIND_LINEAGE,
        add_one_to_the_identity_container_number,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "谱系树根条目的身份与容器头不符：只红 I-9.17：{verdicts:?}"
    );
}

/// I-1.7（打包容器合法与判定顺序） 记录宽那一步对类型 1（墓碑，登记 40）：三个单元的文件发完意图之后，墓碑容器头里的记录宽 40 → 39。
/// 墓碑与谱系那一遍、I-9.6 墓碑那一半读容器时遇到记录宽不是登记的那一个，那一版判不了、不判红：只红 I-1.7。
#[test]
fn a_tombstone_container_whose_record_width_is_not_forty_reddens_only_the_packed_container_invariant(
) {
    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-record-width",
    );
    let mut image = clean.clone();
    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
        &mut image,
        TREE_KIND_TOMBSTONE,
        39,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-1.7"],
        "墓碑容器的记录宽不是 40：只红 I-1.7：{verdicts:?}"
    );
}

/// I-1.7 记录宽那一步对类型 5（谱系，登记 24）：回退到 A 之后，谱系容器头里的记录宽 24 → 15。只红 I-1.7（I-9.30 读不出谱系，那一判报射程里判不了）。
#[test]
fn a_lineage_container_whose_record_width_is_not_the_registered_one_reddens_only_the_packed_container_invariant(
) {
    let rolled_back = image_after_rolling_back_to_the_first_root("known-bad-lineage-record-width");
    let mut image = rolled_back.image.clone();
    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
        &mut image,
        TREE_KIND_LINEAGE,
        15,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-1.7"],
        "谱系容器的记录宽不是 24：只红 I-1.7：{verdicts:?}"
    );
}

/// I-9.4（容器号不超最小 key） 第四句（同一片打包记录类型 2 容器里记录号沿记录序严格递增）只红自己的那一形
/// （S4a）：第一个文件之后建一个空 inode 的那一版，叶 [1, 2] 改成
/// [1 nlink 0, 1 nlink 1]、队列空；inode 树根头里的 key 区间跟着改成 [1, 1]（子树覆盖区间，D18（块里携带什么信息） 已定项 2，C478：
/// 不跟着改的话根头里还是 [1, 2]，I-1.1 也红，这一形就不只红 I-9.4 了）。
#[test]
fn a_leaf_holding_the_same_inode_number_twice_reddens_only_the_container_number_invariant() {
    let clean =
        common_deletion_in_progress::pool_with_new_inodes("known-bad-leaf-same-inode-twice", 1)
            .memory_pool();
    let mut image = clean.clone();
    common_deletion_in_progress::turn_the_leaf_into_two_records_of_the_first_file(&mut image);
    let inode_root_slot = common_pointer_head_corruption::the_slot_of(
        &common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_INODE).root,
    );
    common_subtree_coverage_key_range::rewrite_the_largest_key_of_the_node_at(
        &mut image,
        inode_root_slot,
        1,
    );
    let verdicts = check_pool_image(&image);
    // I-3.20 (b) 跟着红（C70，判决 `research/prompts/m3-close-b19-design-r2-main-verification.md` 第二节 Q8）：叶 [1, 2] 改成
    // [1 nlink 0, 1 nlink 1] 之后 inode 树按号数只剩一条、活对象一个，闭式现算 2 × (6 × 16384 + 41 + 35) = 196760；
    // 记账里的标签 6 是写者按两条活记录写的 196912。改的就是这一版的记录数，这一条红是镜像本身的样子，不是镜像改宽了。
    assert_eq!(
        violated_invariants(&verdicts),
        [
            "I-9.4",
            FREE_ALLOCATED_AND_UNRECLAIMABLE_SUM_TO_THE_CAPACITY_AND_THE_COMMITTED_RESERVATION_MATCHES_ITS_CLOSED_FORM
        ],
        "同一片叶里 1 号两条：红 I-9.4，I-3.20 的标签 6 跟着对不上闭式：{verdicts:?}"
    );
    assert!(
        matches!(verdict(&image, "I-9.4"), InvariantVerdict::Violated(detail) if detail.contains("inode 1 之后是 inode 1")),
        "违例说明写出不严格递增的那一对"
    );
}

// ─── I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上）的 W3 C2aL 形态 ───

/// 谱系记录里 H_cur 那 8 字节的偏移（T_rb 8、T_old 8 之后）与一片打包记录单元的记录区起点（D18（块里携带什么信息） 已定项 16）。
const LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET: usize = 16;

const PACKED_UNIT_FIRST_RECORD_OFFSET: usize = 136;

/// 记账树叶（key 宽 22）里条目数的偏移、第一条条目的偏移、一条条目宽与值、代段、设备段的偏移，单元头里声明长度的偏移
/// （偏移口径同 `set_accounting`）。
const ACCOUNTING_LEAF_ENTRY_COUNT_OFFSET: usize = 82 + 44;

const ACCOUNTING_LEAF_FIRST_ENTRY_OFFSET: usize = 159;

const ACCOUNTING_ENTRY_WIDTH: usize = 34;

const ACCOUNTING_ENTRY_DEVICE_OFFSET: usize = 10;

const ACCOUNTING_ENTRY_GENERATION_OFFSET: usize = 14;

const ACCOUNTING_ENTRY_VALUE_OFFSET: usize = 22;

const UNIT_DECLARED_LENGTH_OFFSET_OF_AN_INDEX_NODE: usize = 8;

/// inode 号水位（D5（快照 / 空间记账机制） 已定项 4 第 12 项）不带设备维。
const INODE_NUMBER_WATERMARK_STATISTIC: u16 = 12;

const NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;

/// 合法镜像：崩溃恢复抛弃建过三个 inode（水位 5）的那一版、它的根读得出（`common::rollback_after_a_recovery_abandoned_a_root_that_created_inodes`），
/// 实例 2 再建一个（2 号重发，被抛弃那段里发过的号，D8（核心索引结构） 已定项 6 射程允许；水位 3），再回退到 A（谱系记录 H_cur 3）。
/// 一条违例都没有：I-9.30 真被判过且成立（台阶 (3, 2)、(重发那次, 3)；谱系上 T_rb − 1 之前最大的台阶 3 等于 H_cur；被抛弃那段的台阶随它的根留在
/// 盘上、不在最新根的记账里），I-3.13 真被判过且成立（台阶行代段早于这一版，跳过；每次发布写的那一行代段等于这一版的 txg）。
#[test]
fn a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_and_created_again_holds_the_watermark_step_and_generation_invariants(
) {
    let history = common::rollback_after_a_recovery_abandoned_a_root_that_created_inodes(
        "known-bad-watermark-steps-legal-cross-abandon-reissue",
    );
    let verdicts = check_pool_image(&history.pool.memory_pool());
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "合法镜像一条违例都没有：{verdicts:?}"
    );
    for invariant in [
        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
        ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
    ] {
        assert_eq!(
            verdicts
                .iter()
                .find(|(name, _)| *name == invariant)
                .map(|(_, found)| found),
            Some(&InvariantVerdict::Holds),
            "{invariant} 真被判过且成立"
        );
    }
}

/// [`a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_and_created_again_holds_the_watermark_step_and_generation_invariants`]
/// 那份合法镜像，最新根（回退那一版）谱系树第一片容器里第一条谱系记录的 H_cur 改成 `watermark_rolled_back_from`（重封、补到根）。
fn image_after_the_legal_cross_abandon_rollback_with_the_watermark_rolled_back_from(
    tag: &str,
    watermark_rolled_back_from: u64,
) -> MemoryPool {
    let history = common::rollback_after_a_recovery_abandoned_a_root_that_created_inodes(tag);
    let mut image = history.pool.memory_pool();
    let lineage_container =
        common_deletion_in_progress::first_container_slot_of_the_tree_of_kind_of_the_newest_root(
            &image,
            TREE_KIND_LINEAGE,
        );
    let offset = PACKED_UNIT_FIRST_RECORD_OFFSET
        + LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET;
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        &mut image,
        lineage_container,
        32_768,
        |bytes| {
            assert_eq!(
                u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8")),
                3,
                "回退那一版谱系记录的 H_cur 是 cur 的水位 3"
            );
            bytes[offset..offset + 8].copy_from_slice(&watermark_rolled_back_from.to_le_bytes());
        },
    );
    image
}

/// I-9.30 ④（谱系那一判按「等于」比，W3 判决 `research/prompts/m3-w3-r2-main-verification.md` 第二节攻方 K3）：合法镜像上谱系记录的 H_cur 3 → 2。
/// T_rb − 1 之前最大的台阶是 3，H_cur 小于它——按「被台阶盖住」比（max ≥ H_cur）放过这一形，按「等于」比判红。只红 I-9.30。
#[test]
fn a_lineage_record_whose_watermark_rolled_back_from_is_below_the_last_step_reddens_only_the_watermark_step_invariant(
) {
    let image = image_after_the_legal_cross_abandon_rollback_with_the_watermark_rolled_back_from(
        "known-bad-watermark-steps-lineage-watermark-below-the-step",
        2,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE],
        "谱系记录的 H_cur 比台阶小：只红 I-9.30：{verdicts:?}"
    );
}

/// I-9.30 ④ 的「H_cur 为 0 也判红」（D18（块里携带什么信息） 已定项 24 第 5 条）：合法镜像上谱系记录的 H_cur 3 → 0。只红 I-9.30。
#[test]
fn a_lineage_record_whose_watermark_rolled_back_from_is_zero_reddens_only_the_watermark_step_invariant(
) {
    let image = image_after_the_legal_cross_abandon_rollback_with_the_watermark_rolled_back_from(
        "known-bad-watermark-steps-lineage-watermark-zero",
        0,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE],
        "谱系记录的 H_cur 为 0：只红 I-9.30：{verdicts:?}"
    );
}

/// 最新根记账树（根兼叶）里 inode 号水位那一统计量、代段 `generation` 的那一行：`change` 拿到那一行的 34 字节改值，`None` 把这一行从叶里拿掉
/// （后面的行往前挪、条目数与声明长度各减一行）；重封、补到根。
fn change_the_inode_number_watermark_row_of_the_newest_root(
    image: &mut MemoryPool,
    generation: u64,
    new_value: Option<u64>,
) {
    let slot = common_deletion_in_progress::accounting_tree_slot_of_the_newest_root(image);
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        slot,
        16_384,
        |node| {
            assert_eq!(node[50], 0, "这几份镜像上记账树只有根兼叶");
            let read_u16_at = |unit_bytes: &[u8], offset: usize| {
                u16::from_le_bytes(unit_bytes[offset..offset + 2].try_into().expect("2"))
            };
            let row_count = usize::from(read_u16_at(node, ACCOUNTING_LEAF_ENTRY_COUNT_OFFSET));
            let row_index = (0..row_count)
                .find(|index| {
                    let row = ACCOUNTING_LEAF_FIRST_ENTRY_OFFSET + index * ACCOUNTING_ENTRY_WIDTH;
                    read_u16_at(node, row) == INODE_NUMBER_WATERMARK_STATISTIC
                        && node[row + ACCOUNTING_ENTRY_DEVICE_OFFSET
                            ..row + ACCOUNTING_ENTRY_DEVICE_OFFSET + 4]
                            == NO_DEVICE_DIMENSION.to_le_bytes()
                        && node[row + ACCOUNTING_ENTRY_GENERATION_OFFSET
                            ..row + ACCOUNTING_ENTRY_GENERATION_OFFSET + 8]
                            == generation.to_le_bytes()
                })
                .unwrap_or_else(|| panic!("记账树里有 inode 号水位代段 {generation} 那一行"));
            let row = ACCOUNTING_LEAF_FIRST_ENTRY_OFFSET + row_index * ACCOUNTING_ENTRY_WIDTH;
            match new_value {
                Some(value) => node
                    [row + ACCOUNTING_ENTRY_VALUE_OFFSET..row + ACCOUNTING_ENTRY_VALUE_OFFSET + 8]
                    .copy_from_slice(&value.to_le_bytes()),
                None => {
                    assert!(
                        row_index > 0 && row_index + 1 < row_count,
                        "这一行不在首末：拿掉它头里的 key 区间不变"
                    );
                    let end =
                        ACCOUNTING_LEAF_FIRST_ENTRY_OFFSET + row_count * ACCOUNTING_ENTRY_WIDTH;
                    node.copy_within(row + ACCOUNTING_ENTRY_WIDTH..end, row);
                    node[end - ACCOUNTING_ENTRY_WIDTH..end].fill(0);
                    node[ACCOUNTING_LEAF_ENTRY_COUNT_OFFSET
                        ..ACCOUNTING_LEAF_ENTRY_COUNT_OFFSET + 2]
                        .copy_from_slice(
                            &u16::try_from(row_count - 1).expect("行数").to_le_bytes(),
                        );
                    node[UNIT_DECLARED_LENGTH_OFFSET_OF_AN_INDEX_NODE
                        ..UNIT_DECLARED_LENGTH_OFFSET_OF_AN_INDEX_NODE + 2]
                        .copy_from_slice(
                            &u16::try_from((row_count - 1) * ACCOUNTING_ENTRY_WIDTH)
                                .expect("声明长度")
                                .to_le_bytes(),
                        );
                }
            }
        },
    );
}

/// I-9.30 ④ 管「台阶行少了一级」：合法镜像上最新根记账里重发那次（2 号重发、水位 3）留的那级台阶拿掉。谱系记录的 H_cur 3 对不上
/// T_rb − 1 之前最大的台阶 2，判红；台阶行不算进记账行数（恢复读得回这一版）、I-3.13 不判台阶行。只红 I-9.30。
#[test]
fn an_inode_number_watermark_step_dropped_from_the_accounting_tree_reddens_only_the_watermark_step_invariant(
) {
    let history = common::rollback_after_a_recovery_abandoned_a_root_that_created_inodes(
        "known-bad-watermark-steps-a-step-dropped",
    );
    let mut image = history.pool.memory_pool();
    change_the_inode_number_watermark_row_of_the_newest_root(
        &mut image,
        history.created_again.root.checkpoint_txg.0,
        None,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE],
        "台阶行少了重发那一级：只红 I-9.30：{verdicts:?}"
    );
}

/// I-9.30 ① 管「水位低于台阶」：第一个文件之后建两个空 inode（txg 4，水位 4）、再覆盖写一次（txg 5）——最新根记账里台阶 (3, 2)、(4, 4) 与这一版那一行 (5, 4)。
/// 台阶 (4, 4) 的水位抬到 5：记账里的水位 4 低于它，判红；那一级之后出生的记录一条都没有（覆盖写不建 inode），② 不红；水位 4 仍大于树里最大的 key 3，
/// I-9.6 不红；台阶行 I-3.13 不判。只红 I-9.30。
#[test]
fn an_inode_number_watermark_step_raised_above_the_watermark_reddens_only_the_watermark_step_invariant(
) {
    let mut pool = common_deletion_in_progress::pool_with_new_inodes(
        "known-bad-watermark-steps-a-step-above-the-watermark",
        2,
    );
    let created = pool.output.clone();
    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, b"after the new inodes");
    assert_eq!(
        (
            created.root.checkpoint_txg.0,
            created.inode_number_watermark(),
            pool.output.inode_number_watermark()
        ),
        (4, 4, 4),
        "建两个空 inode 那一版 txg 4、水位 4；覆盖写不改水位"
    );
    let mut image = pool.memory_pool();
    assert_eq!(
        violated_invariants(&check_pool_image(&image)),
        Vec::<&str>::new(),
        "改之前一条都不红"
    );
    change_the_inode_number_watermark_row_of_the_newest_root(&mut image, 4, Some(5));
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE],
        "台阶抬到水位之上：只红 I-9.30：{verdicts:?}"
    );
}
