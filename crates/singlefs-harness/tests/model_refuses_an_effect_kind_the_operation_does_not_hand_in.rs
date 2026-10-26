//! 理想模型（`singlefs_harness::model::IdealModel::judge_and_advance`）先按 (答案的操作种类, 交来的做成结局种类) 判兼容、再判字段：
//! 冷启动只收冷启动读回、可写挂载只收挂载、发布类只收发布；接错了的胶水报 [`ModelDisagreementAspect::EffectKindOfTheOperation`]，
//! 不记成一次成功，模型状态不动（`records/2026-10-24-singlefs-harness-恢复验证审计.md`「H03：模型允许冷启动答案被空发布结果替代」）。
//!
//! 冷启动那两条只用模型（mkfs 之后的模型自己答的冷启动）；挂载与发布那两条要一次真的可写挂载交回的根：两块 4 GiB 的稀疏盘上 mkfs、
//! 可写挂载，挂载拿模型判过，交来的根换成另一种结局再交一次。

use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::model::{
    IdealModel, ModelAnswer, ModelDeviceIdentity, ModelDisagreementAspect, ModelPoolGeometry,
    ModelReadBack, ObservedEffect, ObservedOutcome, ObservedReadBack,
};
use singlefs_harness::model_comparison::observed_mount;

const DEVICE_WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::FourGibibytes;
const FIRST_FILE_CONTENT: [u8; 3000] = [0x5a; 3000];

fn model_after_make_filesystem() -> IdealModel {
    IdealModel::after_make_filesystem(ModelPoolGeometry {
        devices: vec![ModelDeviceIdentity(0), ModelDeviceIdentity(1)],
        device_size_in_bytes: DEVICE_WIDTH.device_bytes(),
    })
}

/// 模型答的冷启动读回原样换成实现那一侧的观测（没有文件的那一形）。
fn observed_read_back_the_model_expects(expected: &ModelReadBack) -> ObservedReadBack {
    match expected {
        ModelReadBack::NoFile { root } => ObservedReadBack::NoFile { root: *root },
        ModelReadBack::FileRead { root, content } => ObservedReadBack::FileRead {
            root: *root,
            content: content.to_vec(),
        },
    }
}

/// 对照：同一个冷启动答案配它自己答的读回，判绿、记一次成功（拒收那一条不是因为冷启动本身判不了）。
#[test]
fn model_accepts_the_cold_start_read_back_it_expects_for_a_cold_start() {
    let mut model = model_after_make_filesystem();
    let answer = model.answer_cold_start_recover();
    let expected = answer
        .expected_read_back
        .as_ref()
        .expect("冷启动的答案带读回");
    let counts = model
        .judge_and_advance(
            &answer,
            &ObservedOutcome::Succeeded(ObservedEffect::ColdStart {
                read_back: observed_read_back_the_model_expects(expected),
            }),
        )
        .expect("冷启动读回与模型答的相同");
    assert_eq!(counts.successes_matched, 1, "记一次成功");
}

#[test]
fn model_refuses_an_empty_publish_handed_in_for_a_cold_start_and_stays_where_it_was() {
    let mut model = model_after_make_filesystem();
    let model_before = model.clone();
    let answer = model.answer_cold_start_recover();
    let judged = model.judge_and_advance(
        &answer,
        &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
            roots: Vec::new(),
            reported_ceiling: None,
        }),
    );
    let disagreement = judged.expect_err("冷启动只收冷启动读回：空发布不许记成一次成功");
    assert_eq!(
        disagreement.aspect,
        ModelDisagreementAspect::EffectKindOfTheOperation,
        "拒在判兼容那一格：{} / {}",
        disagreement.model_answer,
        disagreement.implementation_answer
    );
    assert_eq!(model, model_before, "判红时模型状态一点不动");
}

/// mkfs → 可写挂载（取号 1，零单元发布与暖机）：交回挂载交回的东西、mkfs 之后的模型与它对这次挂载的答（还没判）。
fn first_mount_on_a_fresh_pool() -> (Mounted, IdealModel, ModelAnswer) {
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
    let model = model_after_make_filesystem();
    let answer = model.answer_mount_writable();
    let mounted = mount_writable(&parameters, &mut devices).expect("mkfs 之后第一次可写挂载");
    (mounted, model, answer)
}

/// 可写挂载交回的根原样当成一串发布交来：根列表与模型答的逐条相同，只有结局种类不对。
#[test]
fn model_refuses_a_publish_handed_in_for_a_writable_mount_and_stays_where_it_was() {
    let (mounted, mut model, answer) = first_mount_on_a_fresh_pool();
    let model_before = model.clone();
    let ObservedEffect::Mount { roots, .. } = observed_mount(&mounted) else {
        panic!("observed_mount 交回的是挂载");
    };
    let judged = model.judge_and_advance(
        &answer,
        &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
            roots,
            reported_ceiling: None,
        }),
    );
    let disagreement = judged.expect_err("可写挂载只收挂载：根列表对得上的发布也不许记成一次成功");
    assert_eq!(
        disagreement.aspect,
        ModelDisagreementAspect::EffectKindOfTheOperation,
        "拒在判兼容那一格：{} / {}",
        disagreement.model_answer,
        disagreement.implementation_answer
    );
    assert_eq!(model, model_before, "判红时模型状态一点不动");
}

#[test]
fn model_refuses_a_cold_start_read_back_handed_in_for_a_publish_and_stays_where_it_was() {
    let (mounted, mut model, mount_answer) = first_mount_on_a_fresh_pool();
    model
        .judge_and_advance(
            &mount_answer,
            &ObservedOutcome::Succeeded(observed_mount(&mounted)),
        )
        .expect("第一次可写挂载与模型答的相同");
    let model_before = model.clone();
    let answer = model
        .answer_publish_first_file(&FIRST_FILE_CONTENT)
        .expect("第一次挂载的会话开着");
    let newest_root = model.answer_cold_start_recover();
    let read_back = observed_read_back_the_model_expects(
        newest_root
            .expected_read_back
            .as_ref()
            .expect("冷启动的答案带读回"),
    );
    let judged = model.judge_and_advance(
        &answer,
        &ObservedOutcome::Succeeded(ObservedEffect::ColdStart { read_back }),
    );
    let disagreement = judged.expect_err("发布只收发布：冷启动读回不许记成一次成功");
    assert_eq!(
        disagreement.aspect,
        ModelDisagreementAspect::EffectKindOfTheOperation,
        "拒在判兼容那一格，不是落到冷启动那一支的「模型答不了」：{} / {}",
        disagreement.model_answer,
        disagreement.implementation_answer
    );
    assert_eq!(model, model_before, "判红时模型状态一点不动");
}
