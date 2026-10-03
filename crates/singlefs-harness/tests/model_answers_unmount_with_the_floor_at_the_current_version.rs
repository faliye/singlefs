//! 理想模型给正常卸载的答（`IdealModel::answer_unmount`）与对拍（`model_comparison::observed_unmount`）：卸载那一串带新 F 的空发布，
//! 根上带的 F 是卸载开始时现行那一版的 txg（D16（发布语义） 已定项 1「正常卸载」与「抬 F 的上限」那一行「卸载抬 F」：抬到现行那一版的 txg，
//! 不另判上限；射程：卸载之后 F = 现行那一版的 txg）；第一版几何（R = 3、区域归属 0 / 1 / 0）下现行 txg 除以 3 余 1 时推 3 次，
//! 余 0 或余 2 时推 2 次。E171（三oracle合谋测试） 第 42 行（`crates/mutations.tsv` 里「卸载把 F 抬到现行那一版带的 F」那一行的形态）
//! 此前三条 oracle 都放行：模型没有卸载的答，对拍停在卸载之前。
//!
//! 池：两块 4 GiB 的稀疏盘。mkfs → 第一次可写挂载（取号 1，零单元发布 txg 1 与暖机 txg 2，拿模型判过）→ 同一次挂载里第一个文件版本
//! （txg 3，拿模型判过）→ 各条用例自己的后几步；树表 0 条那一版上的卸载（用户 2026-10-04 定：一个字节都不写，模型答零条根）只走第一次挂载。每一步都由实现做、交回的东西换成观测拿模型判；判的是实现交出的东西，不另读盘。

use std::collections::BTreeMap;

use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable, unmount, Mounted, ShadowLedger, UnmountRaisedTheFloor, Unmounted,
};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::model::{
    IdealModel, ModelAnswer, ModelCheckpointTxg, ModelDeviceIdentity, ModelDisagreement,
    ModelDisagreementAspect, ModelJudgementCounts, ModelPoolGeometry, ObservedEffect,
    ObservedOutcome,
};
use singlefs_harness::model_comparison::{
    observed_mount, observed_root_of_file_version, observed_unmount,
};

const DEVICE_WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::FourGibibytes;

/// 文件内容：一个数据单元装得下。
const FILE_CONTENT_BYTES: usize = 3000;
const FIRST_FILE_FILL_BYTE: u8 = 0x5A;
const OVERWRITE_FILL_BYTE: u8 = 0xB0;
const FIXED_WRITE_TIME_SECONDS: u64 = 1_700_000_000;

/// 一次可写挂载之后的池与走到这一刻的模型（每一步都判过、对得上）。
struct MountedPoolJudgedByTheModel {
    parameters: MakeFilesystemParameters,
    devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    mounted: Mounted,
    model: IdealModel,
}

/// 拿模型判一步；对不上就带着哪一格 panic（证红时看的就是这一格）。
fn judged_by_the_model(
    model: &mut IdealModel,
    answer: &ModelAnswer,
    observed: &ObservedOutcome,
) -> ModelJudgementCounts {
    model
        .judge_and_advance(answer, observed)
        .unwrap_or_else(|disagreement: ModelDisagreement| {
            panic!(
                "模型与实现对不上：{}（{:?}）：模型 {}；实现 {}",
                disagreement.aspect.name(),
                disagreement.aspect,
                disagreement.model_answer,
                disagreement.implementation_answer
            )
        })
}

/// mkfs → 第一次可写挂载（零单元发布 txg 1、暖机 txg 2），挂载拿模型判过；现行那一版树表 0 条。
fn first_mount_on_a_fresh_pool() -> MountedPoolJudgedByTheModel {
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
    let answer = model.answer_mount_writable();
    let mounted = mount_writable(&parameters, &mut devices).expect("mkfs 之后第一次可写挂载");
    judged_by_the_model(
        &mut model,
        &answer,
        &ObservedOutcome::Succeeded(observed_mount(&mounted)),
    );
    MountedPoolJudgedByTheModel {
        parameters,
        devices,
        mounted,
        model,
    }
}

/// 在 [`first_mount_on_a_fresh_pool`] 那一版（树表 0 条）上发第一个文件版本（txg 3），拿模型判过；交回它。
fn publish_the_first_file_judged(pool: &mut MountedPoolJudgedByTheModel) -> TransactionOutput {
    let (version_to_build_on, record_bytes) = match &pool.mounted.current {
        PoolVersion::WithoutFile(version_without_file) => (
            version_without_file.root,
            version_without_file.record_bytes.clone(),
        ),
        PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => {
            panic!("刚 mkfs 的池第一次挂载之后现行那一版树表 0 条")
        }
    };
    let content = vec![FIRST_FILE_FILL_BYTE; FILE_CONTENT_BYTES];
    let answer = pool
        .model
        .answer_publish_first_file(&content)
        .expect("模型里第一次挂载的会话开着");
    let first_file = {
        let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut pool.mounted.allocator,
            &version_to_build_on,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            pool.mounted.output.instance,
            &record_bytes,
        )
        .expect("第一个文件版本")
    };
    judged_by_the_model(
        &mut pool.model,
        &answer,
        &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
            roots: vec![observed_root_of_file_version(&first_file)],
            reported_ceiling: None,
        }),
    );
    first_file
}

/// 在 `previous` 后面覆盖写一版（txg + 1），拿模型判过；交回它。
fn publish_an_overwrite_judged(
    pool: &mut MountedPoolJudgedByTheModel,
    previous: &TransactionOutput,
) -> TransactionOutput {
    let content = vec![OVERWRITE_FILL_BYTE; FILE_CONTENT_BYTES];
    let answer = pool
        .model
        .answer_publish_overwrite(&content)
        .expect("模型里会话开着、现行那一版带文件");
    let overwrite = {
        let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.mounted.allocator,
            previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            pool.mounted.output.instance,
        )
        .expect("覆盖写")
    };
    judged_by_the_model(
        &mut pool.model,
        &answer,
        &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
            roots: vec![observed_root_of_file_version(&overwrite)],
            reported_ceiling: None,
        }),
    );
    overwrite
}

/// 在现行那一版 `current` 上正常卸载（影子账开着，同随机历史的执行器）；推了那一串的交回它。
fn unmount_raising_the_floor(
    pool: &mut MountedPoolJudgedByTheModel,
    current: TransactionOutput,
) -> UnmountRaisedTheFloor {
    let mut current = PoolVersion::WithFile(current);
    let unmounted = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut pool.mounted.allocator,
        &mut current,
        ShadowLedger::On,
    )
    .expect("正常卸载");
    let Unmounted::FloorRaisedToTheCurrentVersion(raised) = unmounted else {
        panic!("带文件的一版上卸载报成「树表 0 条、一个字节都不写」");
    };
    raised
}

/// 卸载那一串写出的根的 (txg, 根上带的 F)，按次序。
fn txgs_and_floors_of(raised: &UnmountRaisedTheFloor) -> Vec<(u64, u64)> {
    raised
        .publishes
        .iter()
        .map(|publish| (publish.root.checkpoint_txg.0, publish.root.rollback_floor.0))
        .collect()
}

/// 第一个文件版本 txg 3（除以 3 余 0）上卸载：推 2 次空发布（txg 4、5），两条根都带 F = 3；模型答的就是这两条，对拍比过两条根、
/// 对得上；卸载之后模型里这个进程没有可写会话，F 生效值是 3。
#[test]
fn unmount_on_the_first_file_at_txg_three_pushes_two_empty_publishes_carrying_floor_three_and_the_model_agrees(
) {
    let mut pool = first_mount_on_a_fresh_pool();
    let first_file = publish_the_first_file_judged(&mut pool);
    assert_eq!(
        (
            first_file.root.checkpoint_txg,
            first_file.root.rollback_floor
        ),
        (CheckpointTxg(3), CheckpointTxg(0)),
        "第一次挂载写 txg 1、2，第一个文件版本 txg 3，F 还是 0"
    );
    let answer = pool
        .model
        .answer_unmount()
        .expect("模型里会话开着、现行那一版有树");
    let raised = unmount_raising_the_floor(&mut pool, first_file);
    let txgs_and_floors_handed_in = txgs_and_floors_of(&raised);
    // 先拿模型判（对拍那一格先红），再钉实现交回的数。
    let counts = judged_by_the_model(
        &mut pool.model,
        &answer,
        &ObservedOutcome::Succeeded(observed_unmount(
            &Unmounted::FloorRaisedToTheCurrentVersion(raised),
        )),
    );
    assert_eq!(
        (counts.successes_matched, counts.roots_compared),
        (1, 2),
        "卸载那一步做成、比过两条根：{counts:?}"
    );
    assert_eq!(
        txgs_and_floors_handed_in,
        [(4, 3), (5, 3)],
        "现行 txg 3 除以 3 余 0：卸载推 2 次，每条根带 F = 现行那一版的 txg 3"
    );
    assert!(
        !pool.model.has_writable_session(),
        "卸载之后这个进程没有可写会话"
    );
    assert_eq!(
        pool.model.effective_rollback_floor(),
        ModelCheckpointTxg(3),
        "卸载之后 F 生效值 = 卸载开始时现行那一版的 txg"
    );
}

/// 第一个文件版本之后覆盖写一版（txg 4，除以 3 余 1）再卸载：推 3 次空发布（txg 5、6、7；txg 5、6 落盘 0，txg 7 才落到盘 1），
/// 三条根都带 F = 4；模型答的也是 3 条，对拍对得上。
#[test]
fn unmount_on_an_overwrite_at_txg_four_pushes_three_empty_publishes_carrying_floor_four_and_the_model_agrees(
) {
    let mut pool = first_mount_on_a_fresh_pool();
    let first_file = publish_the_first_file_judged(&mut pool);
    let overwrite = publish_an_overwrite_judged(&mut pool, &first_file);
    assert_eq!(
        overwrite.root.checkpoint_txg,
        CheckpointTxg(4),
        "覆盖写接在 txg 3 后面"
    );
    let answer = pool
        .model
        .answer_unmount()
        .expect("模型里会话开着、现行那一版有树");
    let raised = unmount_raising_the_floor(&mut pool, overwrite);
    let txgs_and_floors_handed_in = txgs_and_floors_of(&raised);
    // 先拿模型判（对拍那一格先红），再钉实现交回的数。
    let counts = judged_by_the_model(
        &mut pool.model,
        &answer,
        &ObservedOutcome::Succeeded(observed_unmount(
            &Unmounted::FloorRaisedToTheCurrentVersion(raised),
        )),
    );
    assert_eq!(
        (counts.successes_matched, counts.roots_compared),
        (1, 3),
        "卸载那一步做成、比过三条根：{counts:?}"
    );
    assert_eq!(
        txgs_and_floors_handed_in,
        [(5, 4), (6, 4), (7, 4)],
        "现行 txg 4 除以 3 余 1：卸载推 3 次，每条根带 F = 4"
    );
}

/// 卸载那一串的根带的是现行那一版自己带的 F（0），不是它的 txg（3）——E171 第 42 行那种实现交回的样子（在交回的根上改出来）：
/// 对拍报「根带的 F」那一格。
#[test]
fn an_unmount_whose_roots_carry_the_floor_of_the_current_version_instead_of_its_txg_is_reported_at_the_rollback_floor(
) {
    let mut pool = first_mount_on_a_fresh_pool();
    let first_file = publish_the_first_file_judged(&mut pool);
    let floor_of_the_current_version = first_file.root.rollback_floor;
    let answer = pool
        .model
        .answer_unmount()
        .expect("模型里会话开着、现行那一版有树");
    let mut raised = unmount_raising_the_floor(&mut pool, first_file);
    for publish in &mut raised.publishes {
        publish.root.rollback_floor = floor_of_the_current_version;
    }
    let observed = ObservedOutcome::Succeeded(observed_unmount(
        &Unmounted::FloorRaisedToTheCurrentVersion(raised),
    ));
    assert_eq!(
        pool.model
            .judge_and_advance(&answer, &observed)
            .map(|_| ())
            .map_err(|disagreement| disagreement.aspect),
        Err(ModelDisagreementAspect::RollbackFloor)
    );
}

/// 每块盘上写过的扇区与内容（逐字节比卸载前后用）。
fn written_sectors_of_every_device(
    pool: &MountedPoolJudgedByTheModel,
) -> Vec<BTreeMap<u64, Vec<u8>>> {
    pool.devices
        .iter()
        .map(|(_, device)| device.image.written_sectors().clone())
        .collect()
}

/// 在现行那一版树表 0 条的 [`first_mount_on_a_fresh_pool`] 上正常卸载（影子账开着，同随机历史的执行器）：
/// 实现一个字节都不写，交回 `NothingWrittenOnAVersionWithoutFile`；交回它。
fn unmount_on_a_version_without_trees(pool: &mut MountedPoolJudgedByTheModel) -> Unmounted {
    let unmounted = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut pool.mounted.allocator,
        &mut pool.mounted.current,
        ShadowLedger::On,
    )
    .expect("树表 0 条的一版上正常卸载");
    assert!(
        matches!(
            unmounted,
            Unmounted::NothingWrittenOnAVersionWithoutFile { .. }
        ),
        "树表 0 条的一版上卸载报成「推了一串空发布」"
    );
    unmounted
}

/// 现行那一版树表 0 条（第一次挂载之后还没写过文件）：用户 2026-10-04 定「不推，条款照实现写」（2026-09-26 那条定案）——
/// 实现一个字节都不写、F 不动，模型答零条根、会话关；对拍拿实现交回的零条根比，对得上；盘上逐字节不变，F 生效值还是 0。
#[test]
fn unmount_on_a_version_without_trees_writes_nothing_and_the_model_answers_zero_roots_and_agrees() {
    let mut pool = first_mount_on_a_fresh_pool();
    let floor_before = pool.model.effective_rollback_floor();
    let sectors_before = written_sectors_of_every_device(&pool);
    let answer = pool
        .model
        .answer_unmount()
        .expect("模型里会话开着、现行那一版树表 0 条也答");
    let unmounted = unmount_on_a_version_without_trees(&mut pool);
    assert_eq!(
        written_sectors_of_every_device(&pool),
        sectors_before,
        "树表 0 条的一版上卸载一个字节都不写"
    );
    // 先拿模型判（对拍那一格先红），再钉模型自己往前走的结果。
    let counts = judged_by_the_model(
        &mut pool.model,
        &answer,
        &ObservedOutcome::Succeeded(observed_unmount(&unmounted)),
    );
    assert_eq!(
        (counts.successes_matched, counts.roots_compared),
        (1, 0),
        "卸载那一步做成、零条根可比：{counts:?}"
    );
    assert!(
        !pool.model.has_writable_session(),
        "卸载之后这个进程没有可写会话"
    );
    assert_eq!(
        pool.model.effective_rollback_floor(),
        floor_before,
        "树表 0 条的一版上卸载不抬 F"
    );
    assert_eq!(floor_before, ModelCheckpointTxg(0), "mkfs 之后 F 还是 0");
}

/// 实现在树表 0 条的一版上推了空发布（假想的实现错：在带文件的池上卸载交回的那两条根，当作这一格实现交回的）：
/// 模型答零条根，对拍报「写出的根的条数」。
#[test]
fn an_unmount_that_pushes_empty_publishes_on_a_version_without_trees_is_reported_at_the_publish_count(
) {
    let mut pool_with_a_file = first_mount_on_a_fresh_pool();
    let first_file = publish_the_first_file_judged(&mut pool_with_a_file);
    let raised = unmount_raising_the_floor(&mut pool_with_a_file, first_file);
    let observed = ObservedOutcome::Succeeded(observed_unmount(
        &Unmounted::FloorRaisedToTheCurrentVersion(raised),
    ));
    let pool_without_a_file = first_mount_on_a_fresh_pool();
    let answer = pool_without_a_file
        .model
        .answer_unmount()
        .expect("模型里会话开着、现行那一版树表 0 条也答");
    assert_eq!(
        pool_without_a_file
            .model
            .clone()
            .judge_and_advance(&answer, &observed)
            .map(|_| ())
            .map_err(|disagreement| disagreement.aspect),
        Err(ModelDisagreementAspect::PublishCount)
    );
}
