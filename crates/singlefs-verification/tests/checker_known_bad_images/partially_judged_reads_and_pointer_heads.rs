//! 读不回时只判一部分的不变量、全零的树表指针、指针头里第一版之外的字段、容器指针的 extent 偏移、根槽的保留字节。

use super::*;

/// 两块盘上同一个槽那一份单元整个写 0（不重封、不补引用链）。
pub(super) fn zero_both_copies_of_the_unit(image: &mut MemoryPool, slot: u64, length: usize) {
    for device in DEVICES {
        write(image, device, slot * SLOT, &vec![0u8; length]);
    }
}

/// 一份坏镜像上 I-5.5 报部分没判、说明里点名最新那条根（实例 1、txg 3）与 `why_not_judged`，判红的照旧是 `violated`。
fn assert_the_first_data_unit_invariant_is_partially_judged(
    image: &MemoryPool,
    why_not_judged: &str,
    violated: &[&str],
) {
    let verdicts = check_pool_image(image);
    assert_eq!(
        violated_invariants(&verdicts),
        violated,
        "判红的照改之前那几条：{verdicts:?}"
    );
    let expected_unjudged =
        format!("判到的那几版落点都对得上；这几版没判：根（实例 1、txg 3）{why_not_judged}");
    assert_eq!(
        verdict(
            image,
            FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK
        ),
        InvariantVerdict::PartiallyJudged(expected_unjudged),
        "I-5.5 报部分没判，逐版列出没判的那一版与为什么"
    );
}

/// 读不出的单元（两份全零、不补引用链）由 I-2.1、I-4.8、I-7.2、I-7.4 判红。
const VIOLATED_WHEN_A_UNIT_OF_THE_NEWEST_ROOT_IS_ZEROED: [&str; 4] =
    ["I-2.1", "I-4.8", "I-7.2", "I-7.4"];

/// I-5.5（攻方 a1）：最新那条根的树表单元两份全零——认不出那一版是不是从 mkfs 水位建树的那一版，部分没判。
#[test]
fn a_tree_table_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged() {
    let mut image = build_pool("known-bad-first-data-unit-tree-table-zeroed").memory_pool();
    zero_both_copies_of_the_unit(&mut image, TREE_TABLE, 16384);
    assert_the_first_data_unit_invariant_is_partially_judged(
        &image,
        "：树表读不出，认不出它是不是从 mkfs 水位第一次建树的那一版",
        &VIOLATED_WHEN_A_UNIT_OF_THE_NEWEST_ROOT_IS_ZEROED,
    );
}

/// I-5.5（攻方 c1）：extent 树根两份全零——那一版引用的数据单元数不全，部分没判。
#[test]
fn an_extent_tree_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged() {
    let mut image = build_pool("known-bad-first-data-unit-extent-root-zeroed").memory_pool();
    zero_both_copies_of_the_unit(&mut image, EXTENT_ROOT, 16384);
    assert_the_first_data_unit_invariant_is_partially_judged(
        &image,
        "（从 mkfs 水位第一次建树的那一版）：extent 树读不全，它引用的数据单元数不全",
        &VIOLATED_WHEN_A_UNIT_OF_THE_NEWEST_ROOT_IS_ZEROED,
    );
}

/// I-5.5（攻方 e3）：分配记录树根两份全零——落点算不出，部分没判。
#[test]
fn an_allocation_record_tree_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged(
) {
    let mut image = build_pool("known-bad-first-data-unit-allocation-root-zeroed").memory_pool();
    zero_both_copies_of_the_unit(&mut image, ALLOCATION_RECORD_TREE_ROOT, 16384);
    assert_the_first_data_unit_invariant_is_partially_judged(
        &image,
        "（从 mkfs 水位第一次建树的那一版）：分配记录树读不全，落点算不出",
        &VIOLATED_WHEN_A_UNIT_OF_THE_NEWEST_ROOT_IS_ZEROED,
    );
}

/// I-5.4 挂法（正推 C2，判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-5.4 挂法（C2）」那一行）：
/// 分配记录树根两份全零——候选集里一棵分配记录树都走不到叶，带树一版「根记录里分配记录树根全零」那一格成立而不计进评估，I-5.4 报部分没判、
/// 点名没计的那一版（一格成立一次不压过不适用，正推探针 `research/prompts/m3-formal-code-b26-r1-sonnet-model/out/probe-i54-new.txt`）。
/// 判红的照旧是读不出的单元那几条。
#[test]
fn an_allocation_record_tree_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
) {
    let mut image = build_pool("known-bad-disjointness-allocation-root-zeroed").memory_pool();
    zero_both_copies_of_the_unit(&mut image, ALLOCATION_RECORD_TREE_ROOT, 16384);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        VIOLATED_WHEN_A_UNIT_OF_THE_NEWEST_ROOT_IS_ZEROED,
        "判红的照改之前那几条：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-5.4"),
        InvariantVerdict::PartiallyJudged(
            "候选集里没有一条根的分配记录树走得到叶，记录之间互不相交一格都没判；带树一版根记录里分配记录树根全零那一格成立而没计：根（实例 1、txg 3）"
                .to_owned()
        ),
        "一棵分配记录树都没走到叶：I-5.4 不报成立，报部分没判"
    );
}

/// I-5.4 按版本（判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2「I-5.4『成立要至少一棵树走到叶』」那一行的零轮收严，
/// 正推探针 `research/prompts/m3-formal-code-b29-r1-sonnet-model/probes/b29_l4_append.rs` 的 `b29_l2_i54_after_zeroing_each_unit_of_the_second_version`）：
/// 覆盖写之后（发布 B）把 B 那一版的分配记录树根两份清零——A 那一版的分配记录树照样走到叶，B 那一版自己的走不到；
/// B 带树、根记录里分配记录树根全零那一格成立而不计进评估，I-5.4 报部分没判、点名 B（A 的树走到了叶不替 B 计成立）。
#[test]
fn an_allocation_record_tree_of_the_newest_version_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged_while_the_previous_version_reads_back(
) {
    let clean = image_after_the_overwrite("known-bad-disjointness-newest-allocation-root-zeroed");
    assert_eq!(
        verdict(&clean, "I-5.4"),
        InvariantVerdict::Holds,
        "覆盖写之后的干净镜像上 A、B 两版的分配记录树都走到叶，I-5.4 成立"
    );
    let allocation_root_of_the_newest_version = common_pointer_head_corruption::the_slot_of(
        &common_pointer_head_corruption::tree_table_entry_of_kind(&clean, TREE_KIND_ALLOCATION)
            .root,
    );
    assert_ne!(
        allocation_root_of_the_newest_version.0, ALLOCATION_RECORD_TREE_ROOT,
        "B 写出了自己的分配记录树根，不与 A 共用"
    );
    let mut image = clean.clone();
    zero_both_copies_of_the_unit(
        &mut image,
        allocation_root_of_the_newest_version.0,
        usize::try_from(NODE_BYTES).expect("16384"),
    );
    assert_eq!(
        verdict(&image, "I-5.4"),
        InvariantVerdict::PartiallyJudged(
            "带树一版根记录里分配记录树根全零那一格成立而没计（那一版自己的分配记录树读不出、走不到叶，别的版本的树走到了叶不替它作保）：根（实例 1、txg 4）"
                .to_owned()
        ),
        "B 那一版的分配记录树走不到叶：A 的树走到了叶不替 B 计成立，I-5.4 报部分没判"
    );
}

/// 覆盖写之后的镜像上把 `slot` 那张树表两份清零：I-5.4 报部分没判，点名树表读不出的那几版（`versions_named`），判红的照旧是读不出的单元那几条
/// （`violated`）。这几版整版进评估，不因别的版本的树走到了叶就报成立。
fn assert_a_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
    tag: &str,
    slot: u64,
    violated: &[&str],
    versions_named: &str,
) {
    let clean = image_after_the_overwrite(tag);
    assert_eq!(
        verdict(&clean, "I-5.4"),
        InvariantVerdict::Holds,
        "覆盖写之后的干净镜像上 I-5.4 成立"
    );
    let mut image = clean.clone();
    zero_both_copies_of_the_unit(
        &mut image,
        slot,
        usize::try_from(NODE_BYTES).expect("16384"),
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        violated,
        "判红的照改之前那几条：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-5.4"),
        InvariantVerdict::PartiallyJudged(format!(
            "树表指针非零而树表读不出的版本，认不出带不带树：根记录里分配记录树根全零那一格与它自己的分配记录树都没判（别的版本的树走到了叶不替它作保）：{versions_named}"
        )),
        "树表读不出的那几版：别的版本的树走到了叶不替它们计成立，I-5.4 报部分没判、逐版点名"
    );
}

/// I-5.4 按版本，树表读不出（改法 F1）：覆盖写之后把 B（实例 1、txg 4）的树表两份清零。
#[test]
fn the_tree_table_of_the_newest_version_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
) {
    assert_a_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
        "known-bad-disjointness-newest-tree-table-zeroed",
        TREE_TABLE_AFTER_OVERWRITE,
        &["I-2.1", "I-4.8", "I-7.2", "I-7.4"],
        "根（实例 1、txg 4）",
    );
}

/// I-5.4 按版本，树表读不出（改法 F1）：覆盖写之后把 A（实例 1、txg 3，新池新建文件那一版）的树表两份清零。
/// A 的单元没走到、遍历少算：I-3.1 照 C697（已分配统计在短环加大文件上误红） 的三判，记账多于遍历落在上沿之内，不红（改三判之前红在这一边）。
#[test]
fn the_tree_table_of_the_previous_version_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
) {
    assert_a_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
        "known-bad-disjointness-previous-tree-table-zeroed",
        TREE_TABLE,
        &["I-2.1", "I-4.8", "I-7.4"],
        "根（实例 1、txg 3）",
    );
}

/// I-5.4 按版本，树表读不出（改法 F1）：覆盖写之后把第 0 代树表（mkfs 写的、树表 0 条，第 0 代根与暖机那两条根都指着它）两份清零。
#[test]
fn the_genesis_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
) {
    assert_a_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
        "known-bad-disjointness-genesis-tree-table-zeroed",
        GENESIS_TREE_TABLE,
        &["I-2.1", "I-4.8", "I-7.4"],
        "根（实例 0、txg 0）、根（实例 1、txg 1）、根（实例 1、txg 2）",
    );
}

/// 只做过 mkfs 的池可写挂载两次（第二次写实例 1 那一行）、再在同一个进程里发第一个文件版本：写行那次发布（实例 2、txg 3）与暖机
/// （实例 2、txg 4）是树表 0 条（照抄 mkfs 那片）、根记录直接持有分配记录树的那一版（C512（树表 0 条的一版上被换下的单元记在哪）），
/// 第一个文件版本（实例 2、txg 5）的分配记录树住它自己树表种类 3 那一条。返回镜像与根记录那一项指着的分配记录树根所在的槽（两盘同槽）。
fn image_after_the_first_file_on_the_version_whose_rows_were_written(
    tag: &str,
) -> (MemoryPool, u64) {
    let mut pool = format_pool(tag);
    let mut devices = pool.reopen_recorded();
    let first_instance =
        mount_writable(&parameters(), &mut devices).expect("只做过 mkfs 的池可写挂载：不写行");
    drop(first_instance);
    let row_written = mount_writable(&parameters(), &mut devices).expect("再挂一次：写行");
    let PoolVersion::WithoutFile(version) = &row_written.current else {
        panic!("写行之后现行那一版树表 0 条")
    };
    assert_eq!(
        version.root.tree_table, pool.genesis.root.tree_table,
        "写行那一版的树表照抄 mkfs 那片 0 条的"
    );
    assert_ne!(
        version.root.allocation_record_tree_root,
        singlefs_core::pointer::NodePointer::empty_root(),
        "写行那一版的分配记录树根住根记录"
    );
    let root_record_allocation_tree_root_slot =
        slot_shared_by_both_copies(version.root.allocation_record_tree_root.locations);
    let mut allocator = row_written.allocator.clone();
    {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        let first_file = publish_first_file(
            &mut writer,
            &mut allocator,
            &version.root,
            FirstFile {
                content: &common::file_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            row_written.output.instance,
            &version.record_bytes,
        )
        .expect("写行那一版上发第一个文件版本");
        assert_eq!(
            first_file.root.checkpoint_txg,
            CheckpointTxg(5),
            "第一个文件版本是 txg 5"
        );
    }
    drop(devices);
    (pool.memory_pool(), root_record_allocation_tree_root_slot)
}

/// I-5.4 按版本，树表 0 条、根记录直接持有的分配记录树读不出（改法 F1 的同形，`research/prompts/m3-formal-code-b30-r1-opus-output.md`「另记一处同形、没造盘面（推的）」那一段记下、没造盘面）：
/// 写行那一版之上发了第一个文件版本的镜像上，把写行那一版根记录那一项指着的分配记录树根两份清零。判红的是读不出的单元那几条，
/// 写行那两版（txg 3、4）照样进评估：第一个文件版本的树走到了叶，I-5.4 也不报成立。
/// I-3.1 照 C697（已分配统计在短环加大文件上误红） 的三判不红：读不出的那棵树后面没走到、遍历少算，记账多于遍历落在上沿之内（改三判之前红在这一边）。
#[test]
fn the_allocation_record_tree_held_by_the_root_record_of_a_version_without_trees_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
) {
    let (clean, root_record_allocation_tree_root_slot) =
        image_after_the_first_file_on_the_version_whose_rows_were_written(
            "known-bad-disjointness-root-record-allocation-tree-zeroed",
        );
    assert_eq!(
        violated_invariants(&check_pool_image(&clean)),
        Vec::<&str>::new(),
        "写行那一版之上发了第一个文件版本的干净镜像上一条都不红"
    );
    assert_eq!(
        verdict(&clean, "I-5.4"),
        InvariantVerdict::Holds,
        "干净镜像上 I-5.4 真被评估过且成立"
    );
    let mut image = clean.clone();
    zero_both_copies_of_the_unit(
        &mut image,
        root_record_allocation_tree_root_slot,
        usize::try_from(NODE_BYTES).expect("16384"),
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.1", "I-4.8", "I-7.4"],
        "判红的照改之前那几条（I-3.1 除外）：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-5.4"),
        InvariantVerdict::PartiallyJudged(
            "树表 0 条、分配记录树由根记录直接持有的版本，那棵树读不出、走不到叶，它自己的分配记录一条都没判（别的版本的树走到了叶不替它作保）：根（实例 2、txg 3）、根（实例 2、txg 4）"
                .to_owned()
        ),
        "写行那两版：第一个文件版本的树走到了叶不替它们计成立，I-5.4 报部分没判、逐版点名"
    );
}

/// 写行那一版之上发了第一个文件版本的镜像（[`image_after_the_first_file_on_the_version_whose_rows_were_written`]）上，
/// 把 checkpoint_txg 为 `txg` 的那条根的树表指针整条写零、重封根槽（每条根照抄进它那几条记录的新根段，I-8.10 照旧成立）。
/// 第一版没有一处写者写得出这种根（core 的根记录只从 mkfs 的树表、上一版照抄或这次发布写出的树表取这条指针）。
/// 返回镜像与根记录那一项指着的分配记录树根所在的槽。
fn image_with_the_tree_table_pointer_of_a_root_written_all_zero(
    tag: &str,
    txg: CheckpointTxg,
) -> (MemoryPool, u64) {
    let (mut image, root_record_allocation_tree_root_slot) =
        image_after_the_first_file_on_the_version_whose_rows_were_written(tag);
    rewrite_the_root_slot_of(&mut image, txg, |bytes| {
        bytes[ROOT_RECORD_TREE_TABLE_POINTER_RANGE].fill(0);
    });
    (image, root_record_allocation_tree_root_slot)
}

/// 根记录里树表指针那 86 字节（D22（单元原子性怎么合成） 已定项 7 的偏移表：树表指针 36 起、树 ID 水位 122）。
pub(super) const ROOT_RECORD_TREE_TABLE_POINTER_RANGE: std::ops::Range<usize> = 36..122;

/// 树表指针全零的根（P2-b 的 y4 形）：
/// 最新那条根（第一个文件版本，实例 2、txg 5）的
/// 树表指针整条写零。core 的根记录读者照它读盘 0 槽 0、读不出，可写挂载拒成 `Recovery(UnitUnreadable { slot: SlotNumber(0) })`；
/// 池级 checker 照 core 跟过去、不把它读成「这条根没有树表」：盘 0 槽 0 那两份对不上（I-2.1），
/// 最新的根走不完（I-7.2），从它出发的遍历读不出（I-4.8、I-7.4），I-7.2 的说明写明是树表指针全零。
#[test]
fn an_all_zero_tree_table_pointer_in_the_newest_root_is_followed_like_the_core_and_the_newest_root_does_not_walk_through(
) {
    let (image, _) = image_with_the_tree_table_pointer_of_a_root_written_all_zero(
        "known-bad-all-zero-tree-table-pointer-in-the-newest-root",
        CheckpointTxg(5),
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.1", "I-4.8", "I-7.2", "I-7.4"],
        "最新那条根的树表指针全零：照 core 跟过去读不出，不再一条都不红：{verdicts:?}"
    );
    // I-3.21（走到的引用起点是最新根计入的分配记录的起点）：树表指针全零与树表读不出同一处置，报部分没判，不落进「树表 0 条」那一支报不适用
    // （判决 `research/prompts/m3-close-b19-reason-r3-main-verification.md` 第二节 Q2 第 1 问「定」）。
    assert_eq!(
        verdict(
            &image,
            WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS
        ),
        InvariantVerdict::PartiallyJudged(
            "最新根（txg 5）的树表指针全零：树表读不出（core 照它读盘 0 槽 0），最新根的记录无从读起，走到的引用起点没比"
                .to_owned()
        ),
        "最新根的树表指针全零：I-3.21 报部分没判"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-7.2") else {
        panic!("上面判过 I-7.2 红");
    };
    assert!(
        detail.contains("根（实例 2、txg 5）的树表（根记录里树表指针全零"),
        "I-7.2 写明走不完的是最新那条根、因为树表指针全零：{detail}"
    );
}

/// 同上（攻方报告 P2-b 的 y1 形）：写行之后暖机那条根（实例 2、txg 4，树表 0 条、根记录直接持有分配记录树）的树表指针整条写零。
/// 池级 checker 照 core 跟过去、不把它读成「这条根没有树表」：
/// ① 从这条候选根出发的遍历读不出（I-2.1、I-4.8、I-7.4 红），最新的根照样走得完（I-7.2 不红）；
/// ② 只读的几处判法把它与树表读不出同一处置、点名这一版：I-5.4、I-5.5 与 I-9.28 报部分没判，删除态与墓碑谱系两遍报射程里判不了。
#[test]
fn an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged(
) {
    let (image, _) = image_with_the_tree_table_pointer_of_a_root_written_all_zero(
        "known-bad-all-zero-tree-table-pointer-in-an-older-candidate-root",
        CheckpointTxg(4),
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.1", "I-4.8", "I-7.4"],
        "暖机那条根的树表指针全零：从它出发的遍历读不出，最新的根照样走得完：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-5.4"),
        InvariantVerdict::PartiallyJudged(
            "树表指针全零的版本（core 照它读盘 0 槽 0、读不出），认不出带不带树：根记录里分配记录树根全零那一格与它自己的分配记录树都没判（别的版本的树走到了叶不替它作保）：根（实例 2、txg 4）"
                .to_owned()
        ),
        "I-5.4 不再当这一版没有树表：与树表读不出同一处置、点名它"
    );
    let InvariantVerdict::PartiallyJudged(first_data_unit_unjudged) = verdict(
        &image,
        FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
    ) else {
        panic!("I-5.5 报部分没判：{verdicts:?}");
    };
    assert!(
        first_data_unit_unjudged.contains(
            "根（实例 2、txg 4）：树表指针全零（core 照它读盘 0 槽 0、读不出），认不出它是不是从 mkfs 水位第一次建树的那一版"
        ),
        "I-5.5 点名树表指针全零的那一版：{first_data_unit_unjudged}"
    );
    let InvariantVerdict::PartiallyJudged(tree_identifier_unjudged) = verdict(
        &image,
        TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
    ) else {
        panic!("I-9.28 报部分没判：{verdicts:?}");
    };
    assert!(
        tree_identifier_unjudged.contains(
            "（实例 2、txg 4）：树表指针全零（core 的根记录读者照它读盘 0 槽 0、读不出）"
        ),
        "I-9.28 点名树表指针全零的那条根：{tree_identifier_unjudged}"
    );
    // 单子里有树表指针全零的根，前缀不再说「树表指针非零」（判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md`
    // 第二节 P2-e：两种都说得通的说法）。
    assert!(
        tree_identifier_unjudged.contains(
            "这几条读得出、自证过的根树表条目没读全（树表指针全零、树表读不出或有条目比字段表短），没进比较："
        ),
        "I-9.28 部分没判的前缀把树表指针全零的根也说得通：{tree_identifier_unjudged}"
    );
    let deletion_scan_gap = InvariantVerdict::PartiallyJudged(
        "走读到的某一版树表指针全零（core 照它读盘 0 槽 0、读不出）：那一版的删除态射程里判不了"
            .to_string(),
    );
    assert_eq!(
        verdict(&image, DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY),
        deletion_scan_gap,
        "删除态扫描报射程里判不了，不再当这一版没有队列"
    );
    assert_eq!(
        verdict(&image, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE),
        deletion_scan_gap,
        "墓碑与谱系那一遍报射程里判不了，不再当这一版没有这几棵树"
    );
}

/// 同上（攻方报告 P2-b 的 y2 形）：y1 那张镜像上再把写行那两版（txg 3、4）根记录那一项共用的分配记录树根两份清零。
/// I-5.4 的部分没判点名 txg 3 与 txg 4 两版（txg 4 不被读成没有树表、不进「树表 0 条」那一支）；
/// 两版都点名：txg 4 在「树表指针全零」那一句里，txg 3 在「根记录直接持有的那棵树读不出」那一句里。
#[test]
fn an_all_zero_tree_table_pointer_beside_an_unreadable_root_record_allocation_tree_names_both_versions_in_the_disjointness_invariant(
) {
    let (mut image, root_record_allocation_tree_root_slot) =
        image_with_the_tree_table_pointer_of_a_root_written_all_zero(
            "known-bad-all-zero-tree-table-pointer-beside-an-unreadable-root-record-allocation-tree",
            CheckpointTxg(4),
        );
    zero_both_copies_of_the_unit(
        &mut image,
        root_record_allocation_tree_root_slot,
        usize::try_from(NODE_BYTES).expect("16384"),
    );
    assert_eq!(
        verdict(&image, "I-5.4"),
        InvariantVerdict::PartiallyJudged(
            "树表指针全零的版本（core 照它读盘 0 槽 0、读不出），认不出带不带树：根记录里分配记录树根全零那一格与它自己的分配记录树都没判（别的版本的树走到了叶不替它作保）：根（实例 2、txg 4）；树表 0 条、分配记录树由根记录直接持有的版本，那棵树读不出、走不到叶，它自己的分配记录一条都没判（别的版本的树走到了叶不替它作保）：根（实例 2、txg 3）"
                .to_owned()
        ),
        "写行那两版都点名：txg 4 树表指针全零，txg 3 根记录直接持有的那棵树读不出"
    );
}

/// 新池新建文件那一版里一条走读跟随的指针头部 `change` 那一段改成非第一版的取值、重封、校验和沿引用链补到根槽：池级 checker 只红 I-2.4，
/// 违例那一句是 `expected_violation`（点名是哪条指针、哪一段）。core 读这条指针时拒（判这条指针所在的结构损坏）；池级 checker 判
/// MAC / nonce 与 extent 偏移，也判这三段，红的只有指针头覆盖那一条不变量。
fn assert_a_pointer_head_field_outside_the_first_version_reddens_only_the_header_coverage_invariant(
    image: &MemoryPool,
    expected_violation: &str,
) {
    let verdicts = check_pool_image(image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.4"],
        "只红 I-2.4：{verdicts:?}"
    );
    assert_eq!(
        verdict(image, "I-2.4"),
        InvariantVerdict::Violated(expected_violation.to_owned()),
        "I-2.4 的违例那一句写清是哪条指针、哪一段"
    );
}

/// 新池新建文件那一版的 extent 上段叶第一条条目内联的数据指针：头部 `change` 那一段改掉。
fn pool_whose_inline_data_pointer_head_carries(
    tag: &str,
    change: common_pointer_head_corruption::PointerHeadFieldChange,
) -> MemoryPool {
    let mut image = build_pool(tag).memory_pool();
    let singlefs_core::extent_tree::ExtentUpperLeafTarget::InlineDataUnit(data_pointer) =
        common_pointer_head_corruption::first_upper_leaf_entry(&image).target
    else {
        panic!("新池新建文件那一版只有一个数据单元，上段叶第一条内联它的数据指针");
    };
    let extent_root =
        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root;
    common_pointer_head_corruption::change_a_pointer_head_field_inside_the_unit(
        &mut image,
        common_pointer_head_corruption::the_slot_of(&extent_root),
        common_pointer_head_corruption::node_bytes(),
        &common_pointer_head_corruption::data_pointer_bytes(&data_pointer),
        change,
    );
    image
}

/// I-2.4 指针头部算法类型（改法 F2）：extent 上段叶内联的数据指针算法类型 0 → 1（登记过的加密算法，第一版不加密，D9（加密） 已定项 10）。
#[test]
fn an_inline_data_pointer_naming_an_encryption_algorithm_reddens_only_the_header_coverage_invariant(
) {
    let image = pool_whose_inline_data_pointer_head_carries(
        "known-bad-pointer-head-algorithm-type",
        common_pointer_head_corruption::PointerHeadFieldChange::AlgorithmTypeToOne,
    );
    assert_a_pointer_head_field_outside_the_first_version_reddens_only_the_header_coverage_invariant(
        &image,
        "extent 记录的数据指针：指针头部算法类型是 1，第一版不加密、这 1 字节恒 0",
    );
}

/// I-2.4 指针头部压缩算法码（改法 F2）：树表里 inode 树那条条目的根指针压缩算法码 0 → 1（登记表第一版只有码 0，D9（加密） 已定项 14）。
#[test]
fn a_tree_table_root_pointer_naming_an_unregistered_compression_algorithm_reddens_only_the_header_coverage_invariant(
) {
    let mut image = build_pool("known-bad-pointer-head-compression-algorithm-code").memory_pool();
    let inode_root =
        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
    let tree_table = common_pointer_head_corruption::newest_root(&image).tree_table;
    common_pointer_head_corruption::change_a_pointer_head_field_inside_the_unit(
        &mut image,
        common_pointer_head_corruption::the_slot_of(&tree_table),
        common_pointer_head_corruption::node_bytes(),
        &common_pointer_head_corruption::node_pointer_bytes(&inode_root),
        common_pointer_head_corruption::PointerHeadFieldChange::CompressionAlgorithmCodeToOne,
    );
    assert_a_pointer_head_field_outside_the_first_version_reddens_only_the_header_coverage_invariant(
        &image,
        "树 65537（种类 2）的根：指针头部压缩算法码是 1，第一版登记表只有码 0（不压缩）",
    );
}

/// I-2.4 指针头部压后长度（改法 F2）：extent 上段叶内联的数据指针压缩算法码仍 0、压后长度 0 → 1（码 0 时恒 0，D9（加密） 已定项 14）。
#[test]
fn an_inline_data_pointer_carrying_a_compressed_length_without_compression_reddens_only_the_header_coverage_invariant(
) {
    let image = pool_whose_inline_data_pointer_head_carries(
        "known-bad-pointer-head-compressed-length",
        common_pointer_head_corruption::PointerHeadFieldChange::CompressedLengthToOneWithoutCompression,
    );
    assert_a_pointer_head_field_outside_the_first_version_reddens_only_the_header_coverage_invariant(
        &image,
        "extent 记录的数据指针：指针头部压后长度是 1，压缩算法码 0 时这 2 字节恒 0",
    );
}

/// 树表里种类 `kind` 那棵容器树（墓碑树、谱系树）的码 2 根第 0 条条目里指向容器的子指针（条目里分隔 key 8 + 身份引用 26 之后那 86 字节）：
/// 头部 extent 偏移 0 → 1，根重封、新的整单元校验和沿引用链补到根槽
/// （攻方 `research/prompts/m3-formal-code-b29-r1-opus-model/tests/b29_opus_tombstone_container_pointer_head.rs` 的造法）。
fn set_the_extent_offset_of_the_first_container_pointer_in_the_container_tree_root_of_kind(
    image: &mut MemoryPool,
    kind: u16,
) {
    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
    let root_slot = common_pointer_head_corruption::the_slot_of(&root_pointer);
    let node_length = common_pointer_head_corruption::node_bytes();
    let root = singlefs_core::unit::parse_index_node(
        &common_pointer_head_corruption::unit_bytes_at(image, root_slot, node_length),
    )
    .expect("容器树根解得开");
    assert_eq!(root.level, 1, "种类 {kind} 那棵容器树的根直接管容器");
    let first_entry = root.entries.first().expect("根里至少一条条目");
    let child_pointer = first_entry[CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
        ..CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET + CONTAINER_TREE_ENTRY_CHILD_POINTER_BYTES]
        .to_vec();
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        root_slot,
        node_length,
        |unit| {
            let occurrences: Vec<usize> = (0..=unit.len() - child_pointer.len())
                .filter(|start| unit[*start..*start + child_pointer.len()] == child_pointer[..])
                .collect();
            assert_eq!(occurrences.len(), 1, "根里这条子指针恰好出现一次");
            let field = occurrences[0] + POINTER_EXTENT_OFFSET_FIELD_OFFSET;
            assert_eq!(
                unit[field..field + 2],
                [0, 0],
                "第一版指针的 extent 偏移恒 0"
            );
            unit[field..field + 2].copy_from_slice(&1u16.to_le_bytes());
        },
    );
}

pub(super) const CONTAINER_TREE_ENTRY_CHILD_POINTER_BYTES: usize = 86;

/// E1〔判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L4 第一行〕：unlink 意图之后，墓碑树码 2 根里指向墓碑容器的
/// 子指针头部 extent 偏移改 1。走读跟随这条指针时按 I-2.4 判 extent 偏移（`crates/singlefs-checker/invariants/04-校验和.md` I-2.4 那一行），只红 I-2.4
/// （`walk_container_tree_root` 若只判 MAC / nonce，I-2.4 报成立、一条都不红）。
#[test]
fn an_extent_offset_on_the_container_pointer_of_the_tombstone_tree_root_reddens_only_the_pointer_head_invariant(
) {
    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-container-pointer-extent-offset",
    );
    assert_eq!(
        verdict(&clean, "I-2.4"),
        InvariantVerdict::Holds,
        "unlink 意图之后的干净镜像上 I-2.4 成立"
    );
    let mut image = clean.clone();
    set_the_extent_offset_of_the_first_container_pointer_in_the_container_tree_root_of_kind(
        &mut image,
        TREE_KIND_TOMBSTONE,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.4"],
        "墓碑树根里指向容器的子指针 extent 偏移非 0：只红 I-2.4：{verdicts:?}"
    );
}

/// E1 谱系树那一半：回退到 A 之后（回退那次发布追加一条谱系记录、写出谱系树根与一片谱系容器），谱系树码 2 根里指向谱系容器的子指针
/// 头部 extent 偏移改 1，只红 I-2.4（墓碑树、谱系树走同一个 `walk_container_tree_root`）。
#[test]
fn an_extent_offset_on_the_container_pointer_of_the_lineage_tree_root_reddens_only_the_pointer_head_invariant(
) {
    let rolled_back = image_after_rolling_back_to_the_first_root(
        "known-bad-lineage-container-pointer-extent-offset",
    );
    assert_eq!(
        verdict(&rolled_back.image, "I-2.4"),
        InvariantVerdict::Holds,
        "回退到 A 之后的干净镜像上 I-2.4 成立"
    );
    let mut image = rolled_back.image.clone();
    set_the_extent_offset_of_the_first_container_pointer_in_the_container_tree_root_of_kind(
        &mut image,
        TREE_KIND_LINEAGE,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.4"],
        "谱系树根里指向容器的子指针 extent 偏移非 0：只红 I-2.4：{verdicts:?}"
    );
}

/// I-5.5（攻方 d1）：第一个文件的数据指针第二条位置条目从盘 1 改到盘 0——盘 1 上一个数据单元都没有，该盘跳过、不记，
/// 盘 0 上照判、成立；位置条目不按设备升序，I-2.5 照旧红。
#[test]
fn a_data_pointer_with_no_location_on_one_device_skips_that_device_and_the_first_data_unit_invariant_holds(
) {
    let clean = build_pool("known-bad-first-data-unit-no-location-on-device-one").memory_pool();
    let data_unit_checksum = crc32_castagnoli(&read(&clean, 0, DATA_UNIT * SLOT, 32768));
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        EXTENT_ROOT,
        |bytes| {
            assert!(
                replace_all(
                    bytes,
                    &location_pattern(1, DATA_UNIT, data_unit_checksum),
                    &location_pattern(0, DATA_UNIT, data_unit_checksum),
                ),
                "extent 根里有第一个文件的数据指针"
            );
        },
        true,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.5"],
        "判红的只有位置条目次序那一条：{verdicts:?}"
    );
    assert_eq!(
        verdict(
            &image,
            FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK
        ),
        InvariantVerdict::Holds,
        "盘 1 上没有数据单元：该盘跳过、不记，盘 0 上判到且成立"
    );
}

/// 根记录里算法类型（437）、nonce（438 起 12）、MAC（450 起 16）的偏移（D22（单元原子性怎么合成） 已定项 7；回收门槛 t 与钉槽 p 在 428..437）。
const ROOT_RECORD_ALGORITHM_OFFSET: usize = 437;

const ROOT_RECORD_NONCE_OFFSET: usize = 438;

const ROOT_RECORD_MAC_OFFSET: usize = 450;

/// 发布 B（txg 4）那条根的根槽留位里改一个字节：只红 I-2.4，第一处违例写明是哪一字节。
fn assert_a_root_slot_reserved_byte_reddens_only_the_header_checksum_coverage_invariant(
    tag: &str,
    offset_in_the_root_record: usize,
    value: u8,
) {
    let clean = image_after_the_overwrite(tag);
    assert_eq!(
        verdict(&clean, "I-2.4"),
        InvariantVerdict::Holds,
        "干净镜像上 I-2.4 真被评估过且成立"
    );
    let mut image = clean.clone();
    rewrite_the_root_slot_of(&mut image, CheckpointTxg(4), |bytes| {
        bytes[offset_in_the_root_record] = value;
    });
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.4"],
        "判红的只有 I-2.4：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-2.4") else {
        panic!("I-2.4 要判红");
    };
    assert!(
        detail.contains("txg 4")
            && detail.contains(&format!(
                "根槽偏移 {offset_in_the_root_record} 的字节是 {value:#04x}"
            )),
        "第一处违例写明是哪条根、哪一字节：{detail}"
    );
}

/// I-2.4 根槽留位（攻方 m4）：算法类型写成 1。
#[test]
fn a_root_slot_whose_algorithm_type_is_one_reddens_only_the_header_checksum_coverage_invariant() {
    assert_a_root_slot_reserved_byte_reddens_only_the_header_checksum_coverage_invariant(
        "known-bad-root-slot-algorithm-type",
        ROOT_RECORD_ALGORITHM_OFFSET,
        1,
    );
}

/// I-2.4 根槽留位（攻方 m5）：nonce 第二个字节写成 0x5a。
#[test]
fn a_root_slot_whose_nonce_has_a_non_zero_byte_reddens_only_the_header_checksum_coverage_invariant()
{
    assert_a_root_slot_reserved_byte_reddens_only_the_header_checksum_coverage_invariant(
        "known-bad-root-slot-nonce",
        ROOT_RECORD_NONCE_OFFSET + 1,
        0x5a,
    );
}

/// I-2.4 根槽留位（攻方 m6）：MAC 第十个字节写成 0x5a。
#[test]
fn a_root_slot_whose_mac_has_a_non_zero_byte_reddens_only_the_header_checksum_coverage_invariant() {
    assert_a_root_slot_reserved_byte_reddens_only_the_header_checksum_coverage_invariant(
        "known-bad-root-slot-mac",
        ROOT_RECORD_MAC_OFFSET + 9,
        0x5a,
    );
}

/// 根记录里「树表 0 条那一版的分配记录树根」86 字节的偏移（D22（单元原子性怎么合成） 已定项 7）。
const ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET: usize = 342;

/// 树表条目里根指针 86 字节的偏移（树 ID 8 + 条目长度 2 + 种类 2 + flags 2 之后）。
const TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET: usize = 14;

/// 带树的一版根记录里分配记录树根写成非零指针（Y1 `allocation_record_tree_root` 那一行）：新池新建文件那条根（txg 3）
/// 那 86 字节抄成同一版树表里分配记录那一条的根指针——指着一份合法的分配记录节点（50249），这一版就有了两棵分配记录树。
/// 判在 I-5.4 下（这一格归属交主 agent 定，见 `walk::judge_allocation_records_disjoint`）。
/// 走读另有两条红，照实钉住：走读把根记录这一项当「树表 0 条那一版的分配记录树」（树 0）走下去，50249 头里是树 65538，红 I-1.3；
/// 同一版里 50249 被根记录这一项与树表种类 3 那一条各引用一次，红 I-5.1。I-5.4 另红一条、点名这一版有两棵分配记录树。
#[test]
fn a_file_version_whose_root_record_carries_a_non_zero_allocation_record_tree_root_reddens_the_disjointness_invariant_besides_the_walk(
) {
    let clean =
        build_pool("known-bad-file-version-with-an-allocation-record-tree-root").memory_pool();
    assert_eq!(
        verdict(&clean, "I-5.4"),
        InvariantVerdict::Holds,
        "干净镜像上 I-5.4 真被评估过且成立"
    );
    let tree_table = read(&clean, 0, TREE_TABLE * SLOT, 16384);
    let allocation_entry = tree_table_entry_offset(ALLOCATION_TREE_TABLE_ENTRY_INDEX);
    let allocation_root_pointer =
        tree_table[allocation_entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET
            ..allocation_entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET + NODE_POINTER_BYTES]
            .to_vec();
    let mut image = clean.clone();
    rewrite_the_root_slot_of(&mut image, CheckpointTxg(3), |bytes| {
        let field = &mut bytes[ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET
            ..ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET + NODE_POINTER_BYTES];
        assert!(
            field.iter().all(|byte| *byte == 0),
            "带文件的一版这一项本来全零"
        );
        field.copy_from_slice(&allocation_root_pointer);
    });
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-1.3", "I-5.1", "I-5.4"],
        "判红的是 I-5.4 加走读带出来的 I-1.3、I-5.1：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-5.4"),
        InvariantVerdict::Violated(
            "根（实例 1、txg 3）的树表有 12 条（带树的一版），根记录里树表 0 条那一版的分配记录树根却不是全零：这一版有两棵分配记录树"
                .to_owned()
        ),
        "红在最新那条根"
    );
}
