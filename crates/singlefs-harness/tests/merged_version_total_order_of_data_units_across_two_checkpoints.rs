//! I-1.8（归并后版本全序） 给码 1 的全序键：C464（I-1.8 对码 1 的全序键写窄了） 用户 2026-10-26 定「① 加诞生代号打头」——码 1 的全序键
//! 是 (诞生代号, 实例代号, 事务号)，与码 3 同形。一个事务跨两个 checkpoint、两次都写同一个 key 的码 1 数据单元：两组的写序逐字节相同、
//! 诞生代号不同，两版都是合法的已发布版本；池级 checker 判 I-1.8 成立，I-1.13（扫描重建与遍历方向的索引相同） 择诞生代号大的那一版、
//! 真判成立。键里不带诞生代号时这一张红 I-1.8（池级 checker）、I-1.13 报不适用（扫描重建同键两版择不出），判别力由
//! `crates/mutations.tsv` 里点名这两条用例的变异行钉着。
//!
//! 「对象 key、诞生代号、写序三样都相同而归并键不同」的必红镜像在码 1 上造不出：码 1 的归并键就是对象 key、诞生代号、fsid、写序四段，
//! 扫描项的 fsid 都是本池的。三样都相同的真重号并进同一组，红在 I-1.8 的 ① 那一半（同一组的成员载荷相同）——钉住键改了之后真重号照样红。
//! 镜像的造法在 `tests/common_data_unit_versions_across_two_checkpoints/mod.rs`（checker 档的 GPU 对拍用同一份）。
mod common;
mod common_data_unit_versions_across_two_checkpoints;

use common_data_unit_versions_across_two_checkpoints::{
    birth_txg_of, image_with_a_second_version_of_the_data_unit, write_order_of, BirthTxgOfTheCopy,
};
use singlefs_checker::image::{InvariantVerdict, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX};
use singlefs_checker::walk::check_pool_image;
use singlefs_harness::memory_pool::MemoryPool;

/// I-1.8 在池级 checker 清单里的名字。
const MERGED_VERSION_TOTAL_ORDER: &str = "I-1.8";

fn verdict_of(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(name, _)| *name == invariant)
        .unwrap_or_else(|| panic!("{invariant} 在池级 checker 的清单里"))
        .1
}

/// 合法那一张上判违例的不变量（名字与说明）：照抄的那一版放进一处没写过的槽，别的不变量不该因它红。
fn violated_invariants(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(name, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((name, detail)),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

/// 一个事务跨两个 checkpoint 写同一个 key：照抄的那一版与遍历方向那一版写序逐字节相同、诞生代号差一，池级 checker 判 I-1.8 成立，
/// 别的不变量也不红。判别力：池级 checker 码 1 的全序键去掉诞生代号（`crates/mutations.tsv`「C464 池级 checker 码 1 全序键去掉诞生代号」）时
/// 这一条红在 I-1.8。
#[test]
fn two_published_versions_of_a_data_unit_sharing_the_write_order_across_two_checkpoints_keep_the_total_order(
) {
    let image = image_with_a_second_version_of_the_data_unit(
        "c464-data-unit-across-two-checkpoints-total-order",
        BirthTxgOfTheCopy::OneCheckpointEarlier,
    );
    assert_eq!(
        (
            write_order_of(&image.copied_version) == write_order_of(&image.walked_version),
            birth_txg_of(&image.copied_version) + 1,
        ),
        (true, birth_txg_of(&image.walked_version)),
        "搭建自证：照抄的那一版写序与遍历方向那一版逐字节相同、诞生代号早一个 checkpoint"
    );
    assert_eq!(
        verdict_of(&image.base, MERGED_VERSION_TOTAL_ORDER),
        InvariantVerdict::Holds,
        "对照：照抄之前 I-1.8 成立"
    );
    assert_eq!(
        verdict_of(&image.planted, MERGED_VERSION_TOTAL_ORDER),
        InvariantVerdict::Holds,
        "两组写序相同、诞生代号不同：全序键 (诞生代号, 实例代号, 事务号) 分得开它们，I-1.8 成立"
    );
    assert_eq!(
        violated_invariants(&image.planted),
        Vec::<(&str, String)>::new(),
        "照抄的那一版是一个合法的已发布旧版本：没有一条不变量判违例"
    );
}

/// 同一张镜像上扫描重建择诞生代号大的那一版（遍历方向那一版），I-1.13 真判成立（不是不适用）。
/// 判别力：扫描重建码 1 的择新键去掉诞生代号（`crates/mutations.tsv`「C464 扫描重建码 1 择新键去掉诞生代号」）时两版择新键相同、
/// 扫描重建停下，I-1.13 报不适用，这一条红。
#[test]
fn the_scan_rebuild_takes_the_version_born_later_of_two_data_unit_versions_sharing_the_write_order()
{
    let image = image_with_a_second_version_of_the_data_unit(
        "c464-data-unit-across-two-checkpoints-scan-rebuild",
        BirthTxgOfTheCopy::OneCheckpointEarlier,
    );
    assert_eq!(
        verdict_of(&image.base, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX),
        InvariantVerdict::Holds,
        "对照：照抄之前 I-1.13 成立"
    );
    assert_eq!(
        verdict_of(&image.planted, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX),
        InvariantVerdict::Holds,
        "扫描重建择诞生代号 {} 的那一版（遍历方向那一版），不择早一个 checkpoint 的照抄那一版（槽 {}）：三张表与遍历方向逐项相同",
        birth_txg_of(&image.walked_version),
        image.copied_slot
    );
}

/// 对象 key、诞生代号、写序三样都相同、载荷不同的真重号：归并键与遍历方向那一版相同、并进同一组，I-1.8 红在 ①（同一组的成员载荷相同），
/// 扫描重建报同一版副本载荷不同的不适用。键加了诞生代号之后真重号照样红。
#[test]
fn a_data_unit_copy_sharing_the_object_key_birth_txg_and_write_order_with_another_payload_reddens_the_merged_group(
) {
    let image = image_with_a_second_version_of_the_data_unit(
        "c464-data-unit-true-duplicate",
        BirthTxgOfTheCopy::SameAsTheWalkedVersion,
    );
    match verdict_of(&image.planted, MERGED_VERSION_TOTAL_ORDER) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains("归并成一组的成员载荷不同"),
            "红在 ①（三样都相同的两份并进同一组、载荷不同），不是 ②：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("对象 key、诞生代号、写序都相同而载荷不同，I-1.8 该判违例；实际 {other:?}")
        }
    }
    assert_eq!(
        verdict_of(&image.planted, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX),
        InvariantVerdict::NotApplicable(
            "扫描重建：同一版的副本头里的载荷校验和不同，I-1.8 ① 判损坏、不择"
        ),
        "扫描重建见到同一版的副本载荷不同，停下、不择"
    );
}
