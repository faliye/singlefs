//! 回退之后按版本身份解引用只被回退目标那一代引用的旧版本（`.claude/kb/checks-owed.md` C196（中央映射树的根住哪全仓无条款）：
//! D19（块指针的结构与宽度预算） 已定项 6「映射按版本身份解引用」、已定项 11「中央映射树的根住根记录」）。
//! 映射装着只被旧版本引用的单元，回退到根环里更旧的那一代时映射必须跟着一起回退，否则那一代保住的单元在映射里没有条目、解不出落点。
//!
//! 固定脚本到 C（`build_through_third_publish`：A (1, 3)、B (1, 4)、重开写行与暖机、C (2, 8)），挂着的时候回退到 A：
//! A 的数据单元那把映射 key 只在 A 那一代的映射里，C 的映射里没有它（B、C 各写了自己的数据单元）。
//! 回退那次发布 D (2, 9) 之后，D 的中央映射树——内存里那一版，与从盘上照 D 的根重建出来的那一版——都按这把 key 解出 A 的数据单元那两条位置条目。
//! 判别力：回退那次发布接在后面的「上一版」里用户可见单元的映射 key 照现行那一版（C）带（映射不随根回退），D 的映射里装的是 C 的数据单元那把 key，
//! 按 A 的 key 解不出，这一条红。

mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/rollback_by_a_forward_publish.rs"]
mod scenarios_of_rollback_by_a_forward_publish;

use scenarios_of_rollback_by_a_forward_publish::{
    build_through_third_publish, first_root, roll_back, ThroughTheThirdPublish,
};
use singlefs_core::address::DataUnitIndexInFile;
use singlefs_core::recovery::{rebuild_version, RebuiltVersion};
use singlefs_core::transaction::{
    central_mapping_locations_in_the_version, MappingLookup, TransactionOutput, TransactionUnit,
};

/// 一版的映射里第一个数据单元那个角色的映射 key（这一版自己发的或照上一版带的）。
fn mapping_key_of_the_first_data_unit(version: &TransactionOutput) -> Vec<u8> {
    version
        .mapped_units
        .iter()
        .find(|(identity, _)| *identity == TransactionUnit::Data(DataUnitIndexInFile::FIRST))
        .map(|(_, key)| key.clone())
        .expect("带文件的一版里第一个数据单元有映射 key")
}

#[test]
fn after_rolling_back_two_versions_the_mapping_of_the_new_root_resolves_the_data_unit_only_the_target_version_references(
) {
    let ThroughTheThirdPublish {
        mut pool,
        first,
        third,
    } = build_through_third_publish("rollback-dereferences-the-target-only-version");
    let key_of_the_first_versions_data_unit = mapping_key_of_the_first_data_unit(&first);
    let locations_of_the_first_versions_data_unit = first.data_pointers[0].locations;
    assert_eq!(
        central_mapping_locations_in_the_version(&first, &key_of_the_first_versions_data_unit),
        MappingLookup::Found(locations_of_the_first_versions_data_unit),
        "A 自己的映射按它数据单元那把 key 解出它的两条位置条目"
    );
    assert_eq!(
        central_mapping_locations_in_the_version(&third, &key_of_the_first_versions_data_unit),
        MappingLookup::NodeMalformedOrNoEntryWithThisKey,
        "C 的映射里没有 A 的数据单元那把 key：那个单元只被 A 那一代引用"
    );
    roll_back(&mut pool, first_root()).expect("回退到 A");
    let rolled_back = pool.output.clone();
    assert_eq!(
        (
            rolled_back.root.instance.0,
            rolled_back.root.checkpoint_txg.0
        ),
        (2, 9),
        "回退那次发布是 D (2, 9)"
    );
    assert_eq!(
        central_mapping_locations_in_the_version(
            &rolled_back,
            &key_of_the_first_versions_data_unit
        ),
        MappingLookup::Found(locations_of_the_first_versions_data_unit),
        "D 那一版（内存里）的映射按 A 的 key 解出 A 的数据单元：映射随根回退"
    );
    let image = pool.memory_pool();
    let RebuiltVersion::WithFile(rebuilt_from_the_disk) =
        rebuild_version(&image, &rolled_back.root, Some(rolled_back.record.clone()))
            .expect("照 D 的根从盘上重建")
    else {
        panic!("D 那一版带文件")
    };
    assert_eq!(
        central_mapping_locations_in_the_version(
            &rebuilt_from_the_disk,
            &key_of_the_first_versions_data_unit
        ),
        MappingLookup::Found(locations_of_the_first_versions_data_unit),
        "照 D 的根从盘上读回的中央映射树按 A 的 key 解出 A 的数据单元那两条位置条目"
    );
}
