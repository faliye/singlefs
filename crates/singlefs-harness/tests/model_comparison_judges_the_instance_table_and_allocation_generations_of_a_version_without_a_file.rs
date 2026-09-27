//! 实审 B3c-3（代码审阅第 12 条「对拍双向比、每一版比内容和实例表」，用户定案）：树表 0 条的一版上写行那次发布
//! （`transaction::publish_instance_table_on_version_without_file`，可写挂载在树表 0 条的一版上取号之后的第一次发布）
//! 交回这次写出的单元与这一版的全部分配记录（`VersionWithoutFilePublishOutput::units`、`allocation_records`），
//! 对拍拿它们比这一版的整张实例表与每个角色的分配代（两个方向），不再只比「这次重写了哪几个角色」。
//! 对拍读的是实现交出的东西，不从挂着的分配器或盘上另读（`.claude/rules/fs-design.md`「审计与被审计不用同一段代码」同向）。
//!
//! 池：两块 4 GiB 的稀疏盘。mkfs 之后第一次可写挂载：上一个实例是 mkfs 的 0、要写的行为空，写零单元发布与暖机（树表仍 0 条）；
//! 第二次可写挂载在那一版上取号 2、写实例 1 那一行：写行那次发布重写实例表链（一片）与分配记录树，之后暖机（零单元）。
//! 每条用例都在第二次挂载交回的东西上改一处（实现写错了会交出的样子），拿模型判。
//!
//! 暖机的零单元发布一个字节都不写、不经分配器，输出不带这一版的实例表与分配记录：那几版照旧只比重写的角色集合、实例表照计数
//! （`AllocationRecordsOfTheVersionWithoutFile::SameAsThePreviousVersionNotHandedInByAZeroUnitPublish`），这里把它们的次数钉住。

use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::instance_table::InstanceRow;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::transaction::{
    AllocationRecordsOfTheVersionWithoutFile, PoolVersion, TransactionUnit,
    VersionWithoutFilePublishOutput,
};
use singlefs_core::unit::{build_packed_unit, parse_packed_unit};
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::model::{
    IdealModel, ModelAnswer, ModelDeviceIdentity, ModelDisagreementAspect, ModelJudgementCounts,
    ModelPoolGeometry, ObservedOutcome,
};
use singlefs_harness::model_comparison::observed_mount;

const DEVICE_WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::FourGibibytes;

/// 第二次可写挂载（在树表 0 条的一版上写行）交回的东西，与走到它之前的模型、模型对这一步的答案。
struct SecondMountOnAVersionWithoutFile {
    parameters: MakeFilesystemParameters,
    model: IdealModel,
    answer: ModelAnswer,
    mounted: Mounted,
}

/// mkfs → 第一次可写挂载（零单元发布与暖机，拿模型判过）→ 第二次可写挂载（写行与暖机，还没判）。
fn second_mount_on_a_version_without_file() -> SecondMountOnAVersionWithoutFile {
    let parameters = DEVICE_WIDTH.parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(
                        DEVICE_WIDTH.device_bytes(),
                        PhysicalBlockSizeInBytes(512),
                    ),
                )
            })
            .collect();
    make_filesystem(&parameters, &mut devices).expect("两块 4 GiB 的稀疏盘上 mkfs");
    let mut model = IdealModel::after_make_filesystem(ModelPoolGeometry {
        devices: vec![ModelDeviceIdentity(0), ModelDeviceIdentity(1)],
        device_size_in_bytes: DEVICE_WIDTH.device_bytes(),
    });
    let first_answer = model.answer_mount_writable();
    let first = mount_writable(&parameters, &mut devices).expect("mkfs 之后第一次可写挂载");
    assert!(
        first.output.rows_written.is_empty()
            && matches!(first.output.row_publish, PoolVersion::WithoutFile(_)),
        "上一个实例是 mkfs 的 0：要写的行为空，第一次发布是树表 0 条上的零单元发布"
    );
    model
        .judge_and_advance(
            &first_answer,
            &ObservedOutcome::Succeeded(observed_mount(&first)),
        )
        .expect("第一次可写挂载与模型对得上");
    drop(first);
    model.close_session();
    let answer = model.answer_mount_writable();
    let mounted = mount_writable(&parameters, &mut devices).expect("第二次可写挂载");
    SecondMountOnAVersionWithoutFile {
        parameters,
        model,
        answer,
        mounted,
    }
}

/// 第二次挂载的写行那次发布：树表 0 条的一版上写行。
fn row_publish_of(mounted: &mut Mounted) -> &mut VersionWithoutFilePublishOutput {
    match &mut mounted.output.row_publish {
        PoolVersion::WithoutFile(row_publish) => row_publish,
        PoolVersion::WithFile(_) => {
            panic!("第二次挂载接在树表 0 条的一版上：写行那次发布也是树表 0 条的一版")
        }
    }
}

/// 写行那次发布交回的这一版全部分配记录。
fn allocation_records_of(
    row_publish: &mut VersionWithoutFilePublishOutput,
) -> &mut Vec<singlefs_core::allocator::AllocationRecord> {
    match &mut row_publish.allocation_records {
        AllocationRecordsOfTheVersionWithoutFile::WrittenIntoTheAllocationRecordTreeByThisPublish(
            records,
        ) => records,
        AllocationRecordsOfTheVersionWithoutFile::SameAsThePreviousVersionNotHandedInByAZeroUnitPublish => {
            panic!("写行那次发布重写了分配记录树，交回这一版的全部分配记录")
        }
    }
}

/// 拿模型判第二次挂载：对得上交回计数，对不上交回哪一格。
fn judge_the_second_mount(
    second_mount: &mut SecondMountOnAVersionWithoutFile,
) -> Result<ModelJudgementCounts, ModelDisagreementAspect> {
    let observed = ObservedOutcome::Succeeded(observed_mount(&second_mount.mounted));
    second_mount
        .model
        .judge_and_advance(&second_mount.answer, &observed)
        .map_err(|disagreement| disagreement.aspect)
}

/// 写行这一版照实交回时：写行那一版比过整张实例表（1 张）与三个角色的分配代（实例表一片、分配记录树的根、mkfs 写的树表，
/// 每个角色每块盘一条，共 6 条）；暖机那几次零单元发布照旧只比重写的角色集合、实例表照计数。
/// 改之前写行那一版也走「比不了」那两臂：比过的实例表是 0 张、比不了的多 1 次。
#[test]
fn the_row_publish_on_a_version_without_a_file_is_compared_by_its_whole_instance_table_and_every_role_both_ways(
) {
    let mut second_mount = second_mount_on_a_version_without_file();
    let row_publish = row_publish_of(&mut second_mount.mounted);
    assert_eq!(
        row_publish
            .units
            .iter()
            .map(|unit| unit.identity)
            .collect::<Vec<TransactionUnit>>(),
        row_publish.rewritten,
        "交回的单元就是这次写出的那几个，与重写的角色逐项同序"
    );
    assert!(
        row_publish.units[0].identity == TransactionUnit::InstanceTable
            && row_publish
                .rewritten
                .contains(&TransactionUnit::AllocationTree),
        "写行那次重写实例表链（一片）与分配记录树（含根）：{:?}",
        row_publish.rewritten
    );
    let warm_up_publishes = second_mount.mounted.output.warm_up_publishes.len();
    assert_eq!(
        warm_up_publishes, 1,
        "两块盘：写行那条根落一块盘，再暖机一次落另一块盘，本实例的根就覆盖了每块盘"
    );
    let counts = judge_the_second_mount(&mut second_mount).expect("照实交回的这一版与模型对得上");
    assert_eq!(counts.roots_compared, 2, "写行一条、暖机一条：{counts:?}");
    assert_eq!(
        counts.instance_tables_compared, 1,
        "写行那一版比过整张实例表：{counts:?}"
    );
    assert_eq!(
        counts.allocation_records_compared, 6,
        "写行那一版三个角色、每块盘一条：{counts:?}"
    );
    assert_eq!(
        counts.instance_tables_not_in_the_output, 1,
        "只剩暖机那一次零单元发布的实例表比不了：{counts:?}"
    );
    assert_eq!(
        counts.rewritten_role_sets_compared, 1,
        "只剩暖机那一次零单元发布比重写的角色集合：{counts:?}"
    );
}

/// 写行那次发布写出的实例表单元里上一个实例那一行的「所选根 txg」多 1（单元照样解得开）：对拍报「这一版的实例表」。
#[test]
fn a_wrong_row_in_the_instance_table_written_by_the_row_publish_is_reported() {
    let mut second_mount = second_mount_on_a_version_without_file();
    let filesystem_identifier = second_mount.parameters.filesystem_identifier;
    let row_publish = row_publish_of(&mut second_mount.mounted);
    let page = row_publish
        .units
        .iter_mut()
        .find(|unit| unit.identity == TransactionUnit::InstanceTable)
        .expect("写行那次写出实例表第 0 片");
    let header = parse_packed_unit(&page.bytes).expect("实现写出的实例表单元解得开");
    let mut records = header.records.clone();
    let row = InstanceRow::parse(&records[0]).expect("第 0 条是上一个实例那一行");
    records[0] = InstanceRow {
        instance: row.instance,
        selected_root_txg: CheckpointTxg(row.selected_root_txg.0 + 1),
        applied_transaction_high_water: row.applied_transaction_high_water,
    }
    .to_bytes();
    page.bytes = build_packed_unit(
        header.identity,
        u16::try_from(header.record_width).expect("88"),
        &records,
        header.birth_txg,
        &filesystem_identifier,
        header.write_order,
        header.birth_sequence,
    );
    assert_eq!(
        judge_the_second_mount(&mut second_mount).map(|_| ()),
        Err(ModelDisagreementAspect::InstanceTableOfTheVersion)
    );
}

/// 写行那次发布交回的单元里少了实例表那一片（写行重写整条链，交回的却接不上）：对拍报「这一版的实例表」，不当成比不了。
#[test]
fn a_row_publish_that_hands_in_no_page_of_the_instance_table_is_reported() {
    let mut second_mount = second_mount_on_a_version_without_file();
    let row_publish = row_publish_of(&mut second_mount.mounted);
    row_publish
        .units
        .retain(|unit| !unit.identity.is_a_page_of_the_instance_table());
    assert_eq!(
        judge_the_second_mount(&mut second_mount).map(|_| ()),
        Err(ModelDisagreementAspect::InstanceTableOfTheVersion)
    );
}

/// 写行那次发布交回的分配记录里，新写的实例表那一片在盘 0 上的那条分配代少 1：对拍报「单元的分配代」。
#[test]
fn a_wrong_allocation_generation_of_the_instance_table_written_by_the_row_publish_is_reported() {
    let mut second_mount = second_mount_on_a_version_without_file();
    let row_publish = row_publish_of(&mut second_mount.mounted);
    let slot = row_publish
        .units
        .iter()
        .find(|unit| unit.identity == TransactionUnit::InstanceTable)
        .expect("写行那次写出实例表第 0 片")
        .slot;
    let record = allocation_records_of(row_publish)
        .iter_mut()
        .find(|record| record.device == DeviceIdentity(0) && record.slot == slot)
        .expect("新写的那一片在盘 0 上有一条分配记录");
    record.generation = CheckpointTxg(record.generation.0 - 1);
    assert_eq!(
        judge_the_second_mount(&mut second_mount).map(|_| ()),
        Err(ModelDisagreementAspect::AllocationGeneration)
    );
}

/// 这一版没重写的 mkfs 树表单元（`units` 里没有它，按根记录里树表指针的位置条目找）在盘 1 上那条分配代多 1：对拍报「单元的分配代」。
#[test]
fn a_wrong_allocation_generation_of_the_tree_table_the_row_publish_did_not_rewrite_is_reported() {
    let mut second_mount = second_mount_on_a_version_without_file();
    let row_publish = row_publish_of(&mut second_mount.mounted);
    let tree_table_location = row_publish
        .root
        .tree_table
        .locations
        .into_iter()
        .find(|location| location.device == DeviceIdentity(1))
        .expect("树表指针在盘 1 上有一条位置条目");
    let record = allocation_records_of(row_publish)
        .iter_mut()
        .find(|record| {
            record.device == tree_table_location.device && record.slot == tree_table_location.slot
        })
        .expect("mkfs 写的树表在盘 1 上有一条分配记录");
    assert_eq!(record.generation, CheckpointTxg(0), "mkfs 写的，分配代 0");
    record.generation = CheckpointTxg(1);
    assert_eq!(
        judge_the_second_mount(&mut second_mount).map(|_| ()),
        Err(ModelDisagreementAspect::AllocationGeneration)
    );
}
