//! 池级 checker 的 I-3.5（引用区间的精确性） 缺口条款里「取号跳过」那一判的第四条件（C699（取号跳过改认出生代号），
//! 「行 + R 的实例表指针出生代号」）：
//! 相邻两版 (R⁻, R) 之间有洞时，实例表行接得上的三条之外，还要 R 的实例表指针出生 txg 等于 R.txg——R 所在实例自己更早的版本存在过的，不算跳过。
//!
//! 镜像照 E198（取号跳过认出生代号的前提） 装置的几何与搭法逐字节造（`tests/common_number_acquisition_skip_histories/mod.rs`），参数取 E198 两段产物与
//! 攻方模型（`research/prompts/m3-close-b19-c699-r1-opus-b-model/attack_model_appended.rs` 的 rs-two-files）里的样张。
//! 洞分类只在 I-3.5 的违例说明里看得见（`walk.rs` 那一句格式串里的 `HoleInTheCandidateSet` 成员名）：分类那几条用例把洞右边那一版 R 指着的实例表单元
//! 头里的诞生代号改小到 R⁻.txg（实例表不在用户可见树里，接不接得上都照 (a) 判它、必报违例），读违例说明里那一对的分类；
//! 根记录里实例表指针头的出生 txg 不改，第四条件读的是指针那一份。另两条用例看判定本身：R 新写或带回来的用户可见树单元诞生代号不大于 R⁻.txg 时，
//! 今天只认行的判法认作接得上、照 (a) 判它、I-3.5 红；加了第四条件认作接不上、N3 不判它、I-3.5 成立。
mod common;
mod common_number_acquisition_skip_histories;

use common_number_acquisition_skip_histories::{
    has_a_readable_root_of,
    image_whose_first_overwrite_after_an_unreadable_row_publish_and_warm_ups_holds_a_data_unit_born_at_the_previous_version,
    lower_the_birth_txg_of_the_instance_table_of_the_version, newest_row_bad_history,
    read_the_root_slots_of_these_versions_bad, readable_root_of, readable_roots_on,
    MountConfiguration, NumberAcquisitionSkipHistory,
};
use singlefs_checker::image::{InvariantVerdict, REFERENCE_INTERVALS_ARE_EXACT};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::InstanceGeneration;
use singlefs_verification::memory_pool::MemoryPool;

/// `HoleInTheCandidateSet` 两个有洞成员在违例说明里的样子（`{hole:?}` 打的成员名，外面是格式串里的全角括号）。
const HOLE_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS: &str = "HoleBridgedByTheInstanceTableRows";
const HOLE_NOT_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS: &str = "HoleNotBridgedByTheInstanceTableRows";

fn reference_interval_verdict(image: &MemoryPool) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(identifier, _)| *identifier == REFERENCE_INTERVALS_ARE_EXACT)
        .expect("池级 checker 的清单里有 I-3.5")
        .1
}

/// 洞左边那一组 R⁻（一个实例）与洞右边那一版 R：(实例, txg)。
struct HolePair {
    previous: (u32, u64),
    version: (u32, u64),
}

impl HolePair {
    /// 违例说明里这一对连同它的洞分类那一段（`walk.rs` 的 `judge_the_appearance_rule_of_adjacent_candidate_versions` 那一句格式串）。
    fn in_the_violation_detail(&self, hole_class: &str) -> String {
        format!(
            "相邻两版 (实例 {{{}}}、txg {}) → (实例 {}、txg {})（{hole_class}）",
            self.previous.0, self.previous.1, self.version.0, self.version.1
        )
    }
}

/// I-3.5 判违例，第一处违例是 `pair` 那一对、洞分类是 `hole_class`。
fn assert_the_first_reference_interval_violation_classifies_the_hole_as(
    image: &MemoryPool,
    pair: &HolePair,
    hole_class: &str,
) {
    let expected = pair.in_the_violation_detail(hole_class);
    match reference_interval_verdict(image) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains(&expected),
            "I-3.5 第一处违例该是「{expected}」：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("实例表单元诞生代号改小到 R⁻.txg：I-3.5 应当判违例（「{expected}」），实际 {other:?}")
        }
    }
}

/// 洞的形状：R⁻ 与 R 都读得出、洞里点名的那几版都读不出，R 的实例表指针出生 txg 是 `pointer_birth_txg`。
fn assert_the_hole_between(
    image: &MemoryPool,
    pair: &HolePair,
    unreadable_versions_in_the_hole: &[(u32, u64)],
    pointer_birth_txg: u64,
) {
    assert!(
        has_a_readable_root_of(image, pair.previous.0, pair.previous.1),
        "R⁻ {:?} 的根读得出",
        pair.previous
    );
    for (instance, txg) in unreadable_versions_in_the_hole.iter().copied() {
        assert!(
            !has_a_readable_root_of(image, instance, txg),
            "洞里 (实例 {instance}, txg {txg}) 的根读不出"
        );
    }
    assert_eq!(
        readable_root_of(image, pair.version.0, pair.version.1)
            .instance_table
            .head
            .birth_txg
            .0,
        pointer_birth_txg,
        "R {:?} 的实例表指针出生 txg",
        pair.version
    );
}

/// E198 第一段 C1 那一张（nrb、last 10、k 3、bad 1：产物 `research/results/e198-instance-table-pointer-birth-skip-premise-segment-one-2026-11-02.out`
/// 第 62 行与有效性闸 C1 那一行）：实例 2 写行那一版 (2, 11) 的根槽读坏，洞对 (1@10, 2@12)、R 是暖机那一版、R 的实例表指针出生 txg 11。
/// 最新根的实例表有行 (1, 10)、实例 2 紧接实例 1：三条行条件都成立，今天的判法认作接得上；R 不是新建实例表的那一版（真值是中间有存在过的版本），
/// 加了第四条件认作接不上。去掉第四条件（变异「C699 甲：取号跳过去掉第四条件」）这一条红。
#[test]
fn the_hole_after_an_unreadable_row_publish_slot_is_not_a_number_acquisition_skip_because_the_later_version_did_not_create_its_instance_table(
) {
    let (history, _overwrites) = newest_row_bad_history();
    let pair = HolePair {
        previous: (1, 10),
        version: (2, 12),
    };
    let mut image = history.image();
    lower_the_birth_txg_of_the_instance_table_of_the_version(
        &mut image,
        pair.version.0,
        pair.version.1,
        pair.previous.1,
    );
    read_the_root_slots_of_these_versions_bad(&mut image, &[(2, 11)]);
    assert_the_hole_between(&image, &pair, &[(2, 11)], 11);
    assert_the_first_reference_interval_violation_classifies_the_hole_as(
        &image,
        &pair,
        HOLE_NOT_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS,
    );
}

/// E198 第一段 rs1 组主点那一张（产物同一份第 470 行，`rs1.main.S4.ring1048576.target5.actions-` 的 `R=14`）：实例 1 到 txg 10 → 实例 2 产品挂载
/// （写行 11、暖机 12、13）→ 回退到 (实例 1, 5)（回退那一版 (2, 14)）；实例 2 在 [11, 14) 的根槽读坏，洞对 (1@10, 2@14)、R 的实例表指针出生 txg 11
/// （回退照抄写行那一版的实例表）。行接得上、第四条件不成立：认作接不上。折成单元那一步丢了指针出生 txg、每一版都当新建实例表的那一版
/// （变异「C699 甲：折版本时实例表指针出生 txg 换成这一版的 txg」）这一条红。
#[test]
fn the_hole_before_a_rollback_publish_whose_earlier_versions_are_unreadable_is_not_a_number_acquisition_skip(
) {
    let mut history = NumberAcquisitionSkipHistory::after_making_the_filesystem();
    let mut first = history.mount(MountConfiguration::Product).session;
    history.publish_first_file(&mut first);
    history.overwrite_the_first_file_until(&mut first, 10);
    drop(first);
    let second = history.mount(MountConfiguration::Product);
    assert_eq!(second.row_publish_txg, 11, "实例 2 写行那次发布是 txg 11");
    let mut session = second.session;
    let rollback_txg = history.roll_back_to_the_first_instance_at(&mut session, 5);
    assert_eq!(rollback_txg, 14, "暖机 12、13 之后回退那一版是 txg 14");
    let pair = HolePair {
        previous: (1, 10),
        version: (2, rollback_txg),
    };
    let mut image = history.image();
    lower_the_birth_txg_of_the_instance_table_of_the_version(
        &mut image,
        pair.version.0,
        pair.version.1,
        pair.previous.1,
    );
    let unreadable = [(2, 11), (2, 12), (2, 13)];
    read_the_root_slots_of_these_versions_bad(&mut image, &unreadable);
    assert_the_hole_between(&image, &pair, &unreadable, 11);
    assert_the_first_reference_interval_violation_classifies_the_hole_as(
        &image,
        &pair,
        HOLE_NOT_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS,
    );
}

/// 合法的取号跳过（E198 第二段 skr 组主点、锚 C2 那一张：`skr.main.S4.ring1048576.last6` 的 k = 0）：实例 1 到 txg 6 → 实例 2 产品挂载崩在写行那次
/// 根槽写之前（(2, 7) 只落了记录）→ 实例 3 产品挂载（写行 8，行 (1, 6)、(2, 0, 0)）。洞对 (1@6, 3@8)、R 是实例 3 写行那一版、实例表指针出生 txg 8：
/// 四个条件都成立，加了第四条件照旧认作接得上，不丢判别力。第四条件恒不成立（变异「C699 甲：取号跳过第四条件恒不成立」）这一条红。
#[test]
fn a_number_acquisition_skip_over_a_crashed_row_publish_stays_bridged_because_the_later_version_created_its_instance_table(
) {
    let mut history = NumberAcquisitionSkipHistory::after_making_the_filesystem();
    let mut first = history.mount(MountConfiguration::Product).session;
    history.publish_first_file(&mut first);
    history.overwrite_the_first_file_until(&mut first, 6);
    assert_eq!(first.last_txg(), 6, "实例 1 最后一版是 txg 6");
    drop(first);
    let crashed_row_publish_txg = history.mount_crashing_before_its_first_root_slot_write();
    assert_eq!(
        crashed_row_publish_txg, 7,
        "实例 2 写行那次发布取 txg 7、根槽没落"
    );
    let third = history.mount(MountConfiguration::Product);
    assert_eq!(
        (third.session.instance, third.row_publish_txg),
        (InstanceGeneration(3), 8),
        "实例 3 写行那次发布是 txg 8"
    );
    let pair = HolePair {
        previous: (1, 6),
        version: (3, 8),
    };
    let mut image = history.image();
    lower_the_birth_txg_of_the_instance_table_of_the_version(
        &mut image,
        pair.version.0,
        pair.version.1,
        pair.previous.1,
    );
    assert_the_hole_between(&image, &pair, &[(2, 7)], 8);
    assert_the_first_reference_interval_violation_classifies_the_hole_as(
        &image,
        &pair,
        HOLE_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS,
    );
}

/// 判定本身（nrb 那一段，读坏实例 2 写行与暖机那三槽）：洞对 (1@10, 2@14)、R 是挂载之后第一次覆盖写、实例表指针出生 txg 11。
/// R 新写的数据单元（落在用户可见树里）诞生代号改小到 10：今天的判法认作接得上、照 (a) 判它、extent 树根生在 14 不是整棵继承、I-3.5 红；
/// 加了第四条件认作接不上、N3 不判它，R 新引用的别的单元（树表、实例表这些）都生在 10 之后，I-3.5 成立。
/// 候选根那一处记成「每条根都是新建实例表的那一版」（变异「C699 甲：候选根的实例表指针出生 txg 记成那条根的 txg」）这一条红。
/// GPU 对拍用例（`crash_judge_gpu_judges_the_number_acquisition_skip_like_the_cpu_judge.rs`）判同一张镜像。
#[test]
fn a_user_visible_unit_born_at_the_previous_version_after_an_unreadable_row_publish_and_warm_ups_is_not_judged(
) {
    let image = image_whose_first_overwrite_after_an_unreadable_row_publish_and_warm_ups_holds_a_data_unit_born_at_the_previous_version();
    assert_the_hole_between(
        &image,
        &HolePair {
            previous: (1, 10),
            version: (2, 14),
        },
        &[(2, 11), (2, 12), (2, 13)],
        11,
    );
    assert_eq!(
        reference_interval_verdict(&image),
        InvariantVerdict::Holds,
        "R 不是新建实例表的那一版：洞接不上，R 的数据单元落在用户可见树里、不判"
    );
}

/// rs-two-files 那一形（那一组上池级 checker 在 105 张上红 I-3.5 25 张、甲 0 对；
/// 造法照模型 `attack_build_the_rollback_with_two_files`）：实例 1 新建文件 → 建 inode 2 → 写 inode 2 → 覆盖写第一个文件到 txg 10 → 实例 2 产品挂载
/// （写行 11、暖机 12、13）→ 回退到 (实例 1, 7)（回退那一版 (2, 14)）→ 覆盖写 inode 2（(2, 15)）；实例 2 在 [11, 15) 的根槽读坏。
/// 洞对 (1@10, 2@15)：R 带回 txg 7 那一版第一个文件的数据单元（诞生代号 7，1@10 不引用它）、写 inode 2 换了 extent 树根（不是整棵继承）。
/// 今天的判法认作接得上、照 (a) 判那个数据单元、I-3.5 红（推的：判决说这 25 张红由认错引起是推的，以这条用例在变异下量到的为准）；
/// 加了第四条件（R 的实例表指针出生 txg 11 ≠ 15）认作接不上、不判它，I-3.5 成立。
/// 最新根那一处记成「是新建实例表的那一版」（变异「C699 甲：最新根的实例表指针出生 txg 记成它自己的 txg」）这一条红。
#[test]
fn a_rollback_that_brings_back_an_old_data_unit_of_the_first_file_after_unreadable_slots_is_not_judged_through_a_number_acquisition_skip(
) {
    let mut history = NumberAcquisitionSkipHistory::after_making_the_filesystem();
    let mut first = history.mount(MountConfiguration::Product).session;
    history.publish_first_file(&mut first);
    let second_inode = history.create_an_inode(&mut first);
    let second_inode_written = history.overwrite_an_inode(&mut first, second_inode);
    let rollback_target_txg = 7;
    assert!(
        second_inode_written.root.checkpoint_txg.0 <= rollback_target_txg,
        "inode 2 在回退目标 txg {rollback_target_txg} 之前写过（写在 txg {}）",
        second_inode_written.root.checkpoint_txg.0
    );
    history.overwrite_the_first_file_until(&mut first, 10);
    assert_eq!(first.last_txg(), 10, "实例 1 最后一版是 txg 10");
    drop(first);
    let second = history.mount(MountConfiguration::Product);
    assert_eq!(second.row_publish_txg, 11, "实例 2 写行那次发布是 txg 11");
    let mut session = second.session;
    let rollback_txg =
        history.roll_back_to_the_first_instance_at(&mut session, rollback_target_txg);
    let after_the_rollback = history.overwrite_an_inode(&mut session, second_inode);
    let version_txg = after_the_rollback.root.checkpoint_txg.0;
    assert_eq!(version_txg, rollback_txg + 1, "回退之后紧接着写 inode 2");
    let pair = HolePair {
        previous: (1, 10),
        version: (2, version_txg),
    };
    let unreadable: Vec<(u32, u64)> = (11..version_txg).map(|txg| (2, txg)).collect();
    let mut image = history.image();
    read_the_root_slots_of_these_versions_bad(&mut image, &unreadable);
    assert_the_hole_between(&image, &pair, &unreadable, 11);
    assert_eq!(
        reference_interval_verdict(&image),
        InvariantVerdict::Holds,
        "R 不是新建实例表的那一版：洞接不上，R 带回来的第一个文件的老数据单元落在用户可见树里、不判"
    );
}

/// 残余那一形 sw-read（E198 第二段产物第 1524 行 `sw-read.main.S4.ring1048576.a4.d0.threshold0.accumulatefalse` 的 `version_txg=20`）：
/// 每次挂载都在末尾有一行就扫。实例 1 到 txg 4 → 实例 2 覆盖写到 16 → 实例 3 挂载（写行 17、暖机 18、19）末尾清扫、发清扫那一版 (3, 20)，
/// 新建实例表、出生 txg 20，行 (2, 16) 照留；实例 3 在清扫之前那几版的根槽读坏。洞对 (2@16, 3@20)，真值是中间有存在过的版本，
/// 可四个条件都成立：加了第四条件照旧认作接得上（已知行为、不在这一件里修：判决 `research/prompts/m3-close-b19-c699-r1-main-verification.md`
/// 第一节 sw-read-control 那一行与第二节「残余两形」那一行，病根在「指针出生 txg 等于 R.txg 就说明 R 是新实例的首版」这个前提，清扫那一版也满足它）。
/// 修了这一形（臂丁之类）这一条会红，那时照新判法改它。
#[test]
fn the_hole_before_a_sweep_publish_whose_earlier_versions_are_unreadable_is_still_taken_for_a_number_acquisition_skip(
) {
    let mut history = NumberAcquisitionSkipHistory::after_making_the_filesystem();
    let configuration = MountConfiguration::SweptAtTheEndWhenAnyRowIsLeft;
    let mut first = history.mount(configuration).session;
    history.publish_first_file(&mut first);
    history.overwrite_the_first_file_until(&mut first, 4);
    drop(first);
    let second = history.mount(configuration);
    assert!(!second.sweep_deleted_rows, "实例 2 的挂载末尾没删行");
    let mut session = second.session;
    history.overwrite_the_first_file_until(&mut session, 16);
    assert_eq!(session.last_txg(), 16, "实例 2 最后一版是 txg 16");
    drop(session);
    let third = history.mount(configuration);
    assert!(third.sweep_deleted_rows, "实例 3 的挂载末尾清扫删了行");
    let sweep_txg = third.session.last_txg();
    assert_eq!(
        sweep_txg, 20,
        "清扫那一版是 txg 20（写行 17、暖机 18、19 之后）"
    );
    let pair = HolePair {
        previous: (2, 16),
        version: (3, sweep_txg),
    };
    let mut image = history.image();
    let unreadable: Vec<(u32, u64)> = readable_roots_on(&image)
        .iter()
        .filter(|root| root.instance == InstanceGeneration(3) && root.checkpoint_txg.0 < sweep_txg)
        .map(|root| (root.instance.0, root.checkpoint_txg.0))
        .collect::<std::collections::BTreeSet<(u32, u64)>>()
        .into_iter()
        .collect();
    assert_eq!(
        unreadable,
        vec![(3, 17), (3, 18), (3, 19)],
        "清扫之前实例 3 的那几版是写行与暖机"
    );
    lower_the_birth_txg_of_the_instance_table_of_the_version(
        &mut image,
        pair.version.0,
        pair.version.1,
        pair.previous.1,
    );
    read_the_root_slots_of_these_versions_bad(&mut image, &unreadable);
    assert_the_hole_between(&image, &pair, &unreadable, sweep_txg);
    assert_the_first_reference_interval_violation_classifies_the_hole_as(
        &image,
        &pair,
        HOLE_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS,
    );
}
