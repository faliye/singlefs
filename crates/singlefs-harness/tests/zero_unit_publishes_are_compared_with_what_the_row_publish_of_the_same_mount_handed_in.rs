//! 实审 B3c-4（代码审阅第 12 条「对拍双向比、每一版比内容和实例表」，用户定案；接在 B3c-3 后面，主 agent 定走乙「胶水按指针往下带」）：
//! 树表 0 条的一版上的零单元发布（写行之后的暖机、`publish_without_units`）一个字节都不写、不经分配器，输出不带这一版的实例表与
//! 分配记录（`AllocationRecordsOfTheVersionWithoutFile::SameAsThePreviousVersionNotHandedInByAZeroUnitPublish`）。它的根记录照抄上一版的
//! 实例表指针与分配记录树根指针：两条指针与同一次挂载里前一版逐字段相同，胶水就拿前一版交回的那一份（写行那次交回的单元与全部分配记录）
//! 比这一版的整张实例表与每个角色的分配代（两个方向，与写行那一版同一套判法）；不相同判红，不算比不了
//! （`model_comparison::observed_root_of_version_without_file_carrying`、`observed_mount_carrying_to_zero_unit_publishes`，
//! 执行器 `history` 在会话里记着这次挂载最近那一份）。一次挂载里第一版就是零单元发布的（mkfs 之后第一次可写挂载：取号之后第一次发布
//! 接的是从盘上恢复的那一版，没有单元字节）没有可带的，照旧走比不了的两臂计数。
//!
//! 池：两块 4 GiB 的稀疏盘。mkfs 之后第一次可写挂载（零单元发布与暖机），第二次可写挂载在那一版上取号 2、写实例 1 那一行、暖机一次。

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
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
use singlefs_harness::history::{
    execute_history, AppliedEffect, GeneratedHistory, HistoryDeviceWidth, HistoryEnding,
    HistoryOperation, HistorySeed, HistoryStartingPoint, StepOutcome,
};
use singlefs_harness::model::{
    IdealModel, ModelAnswer, ModelDeviceIdentity, ModelDisagreement, ModelDisagreementAspect,
    ModelJudgementCounts, ModelPoolGeometry, ObservedEffect, ObservedOutcome, ObservedRoot,
};
use singlefs_harness::model_comparison::{
    model_instance, model_instance_row, model_root_key,
    observed_mount_carrying_to_zero_unit_publishes, observed_root_of_version_without_file_carrying,
};

const DEVICE_WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::FourGibibytes;

/// 一次可写挂载交回的东西，与走到它之前的模型、模型对这一步的答案。
struct MountToJudge {
    parameters: MakeFilesystemParameters,
    model: IdealModel,
    answer: ModelAnswer,
    mounted: Mounted,
}

fn two_formatted_sparse_devices(
    parameters: &MakeFilesystemParameters,
) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
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
    make_filesystem(parameters, &mut devices).expect("两块 4 GiB 的稀疏盘上 mkfs");
    devices
}

fn model_after_make_filesystem() -> IdealModel {
    IdealModel::after_make_filesystem(ModelPoolGeometry {
        devices: vec![ModelDeviceIdentity(0), ModelDeviceIdentity(1)],
        device_size_in_bytes: DEVICE_WIDTH.device_bytes(),
    })
}

/// mkfs → 第一次可写挂载（零单元发布与暖机，还没判）。
fn first_mount_after_make_filesystem() -> (MountToJudge, Vec<(DeviceIdentity, SparseBlockDevice)>) {
    let parameters = DEVICE_WIDTH.parameters();
    let mut devices = two_formatted_sparse_devices(&parameters);
    let model = model_after_make_filesystem();
    let answer = model.answer_mount_writable();
    let mounted = mount_writable(&parameters, &mut devices).expect("mkfs 之后第一次可写挂载");
    (
        MountToJudge {
            parameters,
            model,
            answer,
            mounted,
        },
        devices,
    )
}

/// mkfs → 第一次可写挂载（拿模型照往下带的比法判过）→ 第二次可写挂载（写行与暖机，还没判）。
fn second_mount_on_a_version_without_file() -> MountToJudge {
    let (mut first, mut devices) = first_mount_after_make_filesystem();
    let (observed, _) = observed_mount_carrying_to_zero_unit_publishes(&first.mounted);
    first
        .model
        .judge_and_advance(&first.answer, &ObservedOutcome::Succeeded(observed))
        .expect("第一次可写挂载与模型对得上");
    let mut model = first.model;
    model.close_session();
    let answer = model.answer_mount_writable();
    let mounted = mount_writable(&first.parameters, &mut devices).expect("第二次可写挂载");
    MountToJudge {
        parameters: first.parameters,
        model,
        answer,
        mounted,
    }
}

fn row_publish_of(mounted: &Mounted) -> &VersionWithoutFilePublishOutput {
    match &mounted.output.row_publish {
        PoolVersion::WithoutFile(row_publish) => row_publish,
        PoolVersion::WithFile(_) => {
            panic!("第二次挂载接在树表 0 条的一版上：写行那次发布也是树表 0 条的一版")
        }
    }
}

fn only_warm_up_of(mounted: &mut Mounted) -> &mut VersionWithoutFilePublishOutput {
    assert_eq!(
        mounted.output.warm_up_publishes.len(),
        1,
        "两块盘：写行那条根落一块盘，再暖机一次落另一块盘"
    );
    match &mut mounted.output.warm_up_publishes[0] {
        PoolVersion::WithoutFile(warm_up) => warm_up,
        PoolVersion::WithFile(_) => panic!("树表 0 条的一版上暖机是零单元发布"),
    }
}

/// 拿模型判这次挂载：交给模型的是 `roots` 那几条根（写行与暖机，按次序），取号与写的行照实现交回的。
fn judge_the_mount_with_roots(
    mount: &mut MountToJudge,
    roots: Vec<ObservedRoot>,
) -> Result<ModelJudgementCounts, ModelDisagreement> {
    let observed = ObservedEffect::Mount {
        instance: model_instance(mount.mounted.output.instance),
        rows_written: mount
            .mounted
            .output
            .rows_written
            .iter()
            .map(model_instance_row)
            .collect(),
        roots,
    };
    mount
        .model
        .judge_and_advance(&mount.answer, &ObservedOutcome::Succeeded(observed))
}

/// 写行那一版照实交回；暖机那一版拿 `row_publish_to_carry`（写行那一版交回的东西，用例里改坏一处）往下带着比。
fn judge_the_warm_up_carried_from(
    mount: &mut MountToJudge,
    row_publish_to_carry: &VersionWithoutFilePublishOutput,
) -> Result<ModelJudgementCounts, ModelDisagreement> {
    let (row_publish_observed, _) =
        observed_root_of_version_without_file_carrying(row_publish_of(&mount.mounted), None);
    let (_, carried) = observed_root_of_version_without_file_carrying(row_publish_to_carry, None);
    assert!(
        carried.is_some(),
        "写行那一版交回了单元与全部分配记录，之后可以往下带"
    );
    let (warm_up_observed, _) = observed_root_of_version_without_file_carrying(
        only_warm_up_of(&mut mount.mounted),
        carried.as_ref(),
    );
    judge_the_mount_with_roots(mount, vec![row_publish_observed, warm_up_observed])
}

fn warm_up_key(mount: &mut MountToJudge) -> String {
    let warm_up = only_warm_up_of(&mut mount.mounted);
    format!(
        "{:?}",
        model_root_key(warm_up.root.instance, warm_up.root.checkpoint_txg)
    )
}

/// 写行之后同一次挂载里的暖机（零单元发布）：两条指针与写行那一版相同，拿写行那次交回的那一份比整张实例表与三个角色的分配代
/// （实例表一片、分配记录树的根、mkfs 写的树表，每块盘一条）——写行、暖机两版各比一张表、各 6 条分配记录，「比不了」的两臂一次都不走。
#[test]
fn the_warm_up_after_the_row_publish_of_the_same_mount_is_compared_by_the_whole_instance_table_and_every_role(
) {
    let mut second_mount = second_mount_on_a_version_without_file();
    let _ = only_warm_up_of(&mut second_mount.mounted);
    let (observed, carried_after_the_mount) =
        observed_mount_carrying_to_zero_unit_publishes(&second_mount.mounted);
    let counts = second_mount
        .model
        .judge_and_advance(&second_mount.answer, &ObservedOutcome::Succeeded(observed))
        .expect("照实交回的写行与暖机与模型对得上");
    assert_eq!(counts.roots_compared, 2, "写行一条、暖机一条：{counts:?}");
    assert_eq!(
        (
            counts.instance_tables_not_in_the_output,
            counts.rewritten_role_sets_compared
        ),
        (0, 0),
        "暖机那一版不再走「比不了」的两臂：{counts:?}"
    );
    assert_eq!(
        (
            counts.instance_tables_compared,
            counts.allocation_records_compared
        ),
        (2, 12),
        "写行、暖机两版各比一张实例表、三个角色每块盘一条：{counts:?}"
    );
    assert!(
        carried_after_the_mount.is_some(),
        "这次挂载之后的零单元发布接着从写行那一份往下带"
    );
}

/// 带下来的那一份里实例表第 0 片上一个实例那一行的「所选根 txg」多 1（单元照样解得开；写行那一版自己照实比、对得上）：
/// 对拍在暖机那一版上报「这一版的实例表」。
#[test]
fn a_wrong_row_in_the_instance_table_carried_to_the_warm_up_is_reported() {
    let mut second_mount = second_mount_on_a_version_without_file();
    let filesystem_identifier = second_mount.parameters.filesystem_identifier;
    let mut row_publish_with_a_wrong_row = row_publish_of(&second_mount.mounted).clone();
    let page = row_publish_with_a_wrong_row
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
    let key_of_the_warm_up = warm_up_key(&mut second_mount);
    let disagreement =
        judge_the_warm_up_carried_from(&mut second_mount, &row_publish_with_a_wrong_row)
            .expect_err("带下来的实例表错了一行");
    assert_eq!(
        disagreement.aspect,
        ModelDisagreementAspect::InstanceTableOfTheVersion
    );
    assert!(
        disagreement.model_answer.starts_with(&key_of_the_warm_up),
        "红在暖机那一版上：{disagreement:?}"
    );
}

/// 带下来的那一份分配记录里，写行写出的实例表那一片在盘 0 上的那条分配代少 1：对拍在暖机那一版上报「单元的分配代」。
#[test]
fn a_wrong_allocation_generation_carried_to_the_warm_up_is_reported() {
    let mut second_mount = second_mount_on_a_version_without_file();
    let mut row_publish_with_a_wrong_generation = row_publish_of(&second_mount.mounted).clone();
    let slot = row_publish_with_a_wrong_generation
        .units
        .iter()
        .find(|unit| unit.identity == TransactionUnit::InstanceTable)
        .expect("写行那次写出实例表第 0 片")
        .slot;
    let AllocationRecordsOfTheVersionWithoutFile::WrittenIntoTheAllocationRecordTreeByThisPublish(
        records,
    ) = &mut row_publish_with_a_wrong_generation.allocation_records
    else {
        panic!("写行那次发布重写了分配记录树，交回这一版的全部分配记录")
    };
    let record = records
        .iter_mut()
        .find(|record| record.device == DeviceIdentity(0) && record.slot == slot)
        .expect("新写的那一片在盘 0 上有一条分配记录");
    record.generation = CheckpointTxg(record.generation.0 - 1);
    let key_of_the_warm_up = warm_up_key(&mut second_mount);
    let disagreement =
        judge_the_warm_up_carried_from(&mut second_mount, &row_publish_with_a_wrong_generation)
            .expect_err("带下来的分配代错了一条");
    assert_eq!(
        disagreement.aspect,
        ModelDisagreementAspect::AllocationGeneration
    );
    assert!(
        disagreement.model_answer.starts_with(&key_of_the_warm_up),
        "红在暖机那一版上：{disagreement:?}"
    );
}

/// 暖机交回的根里实例表指针换成写行那一版的树表指针（在交回的输出上改，不改盘）：零单元发布该照抄前一版的那一条，
/// 对拍报「这一版的实例表」，不当比不了。
#[test]
fn a_zero_unit_publish_whose_root_names_another_instance_table_than_the_version_before_it_is_reported(
) {
    let mut second_mount = second_mount_on_a_version_without_file();
    let tree_table_pointer = row_publish_of(&second_mount.mounted).root.tree_table;
    only_warm_up_of(&mut second_mount.mounted)
        .root
        .instance_table = tree_table_pointer;
    let (observed, _) = observed_mount_carrying_to_zero_unit_publishes(&second_mount.mounted);
    let disagreement = second_mount
        .model
        .judge_and_advance(&second_mount.answer, &ObservedOutcome::Succeeded(observed))
        .expect_err("零单元发布改了实例表指针");
    assert_eq!(
        disagreement.aspect,
        ModelDisagreementAspect::InstanceTableOfTheVersion,
        "{disagreement:?}"
    );
}

/// 暖机交回的根里分配记录树根指针换成写行那一版的实例表指针（在交回的输出上改，不改盘）：对拍报「单元的分配代」，不当比不了。
#[test]
fn a_zero_unit_publish_whose_root_names_another_allocation_record_tree_than_the_version_before_it_is_reported(
) {
    let mut second_mount = second_mount_on_a_version_without_file();
    let instance_table_pointer = row_publish_of(&second_mount.mounted).root.instance_table;
    only_warm_up_of(&mut second_mount.mounted)
        .root
        .allocation_record_tree_root = instance_table_pointer;
    let (observed, _) = observed_mount_carrying_to_zero_unit_publishes(&second_mount.mounted);
    let disagreement = second_mount
        .model
        .judge_and_advance(&second_mount.answer, &ObservedOutcome::Succeeded(observed))
        .expect_err("零单元发布改了分配记录树根指针");
    assert_eq!(
        disagreement.aspect,
        ModelDisagreementAspect::AllocationGeneration,
        "{disagreement:?}"
    );
}

/// 一次挂载里第一版就是零单元发布（mkfs 之后第一次可写挂载：要写的行为空，取号之后第一次发布接的是从盘上恢复的 mkfs 那一版，
/// 没有单元字节）：没有可带的，那一版与之后的暖机照旧走「比不了」的两臂，次数就是这次挂载写出的根数。
#[test]
fn a_mount_whose_first_version_is_a_zero_unit_publish_counts_every_version_as_not_in_the_output() {
    let (mut first, _devices) = first_mount_after_make_filesystem();
    assert!(
        first.mounted.output.rows_written.is_empty()
            && matches!(
                first.mounted.output.row_publish,
                PoolVersion::WithoutFile(_)
            ),
        "上一个实例是 mkfs 的 0：要写的行为空，第一次发布是树表 0 条上的零单元发布"
    );
    let versions_of_the_mount = 1 + first.mounted.output.warm_up_publishes.len();
    assert_eq!(versions_of_the_mount, 2, "零单元发布一次、暖机一次");
    let (observed, carried_after_the_mount) =
        observed_mount_carrying_to_zero_unit_publishes(&first.mounted);
    assert!(
        carried_after_the_mount.is_none(),
        "这次挂载一版都没交回单元字节，之后没有可带的"
    );
    let counts = first
        .model
        .judge_and_advance(&first.answer, &ObservedOutcome::Succeeded(observed))
        .expect("第一次可写挂载与模型对得上");
    assert_eq!(
        (
            counts.instance_tables_not_in_the_output,
            counts.rewritten_role_sets_compared,
            counts.instance_tables_compared
        ),
        (2, 2, 0),
        "两版都比不了：{counts:?}"
    );
}

/// 执行器里的会话记着这次挂载最近那一份：mkfs 之后可写挂载两次、再零单元发布两次。第一次挂载的两版没有可带的、照计数；
/// 第二次挂载的写行与暖机、之后两次零单元发布都比整张实例表——比不了的只剩第一次挂载那两版，比过的实例表 4 张。
#[test]
fn the_session_carries_the_row_publish_down_to_the_zero_unit_publishes_after_the_mount() {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishWithoutUnits,
            HistoryOperation::PublishWithoutUnits,
        ],
    };
    let run = execute_history(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        matches!(
            run.outcomes.as_slice(),
            [
                StepOutcome::Applied(AppliedEffect::Mounted {
                    instance: InstanceGeneration(1),
                    publishes: 2,
                    ..
                }),
                StepOutcome::Applied(AppliedEffect::Mounted {
                    instance: InstanceGeneration(2),
                    publishes: 2,
                    ..
                }),
                StepOutcome::Applied(AppliedEffect::Published { .. }),
                StepOutcome::Applied(AppliedEffect::Published { .. }),
            ]
        ),
        "两次挂载各写两条根、两次零单元发布都做成：{:?}",
        run.outcomes
    );
    let counts = &run.tally.model_counts;
    assert_eq!(
        (
            counts.instance_tables_not_in_the_output,
            counts.rewritten_role_sets_compared
        ),
        (2, 2),
        "比不了的只剩第一次挂载那两版：{counts:?}"
    );
    assert_eq!(
        counts.instance_tables_compared, 4,
        "第二次挂载的写行、暖机与两次零单元发布各比一张实例表：{counts:?}"
    );
}
