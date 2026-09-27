//! checker 档模块：无（自己逐个造崩溃状态，只用 harness 档的内存池与池级 checker）
//! 里程碑「覆盖写、释放、回退与复用」步 4（管理员回退改成挂着时的向前发布）：回退那次发布 D 的每个崩溃点各恢复一次。
//! 它逐个前缀造崩溃状态，归 checker 档（`.claude/rules/verification.md`「定义与名字」）；
//! 同一条脚本的日常用例在 `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs`，
//! 这里用到的几个搭建函数从那里抄了一份（两档宁可各留一份，不让 harness 依赖 checker 档）。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{
    build_pool, file_content, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::mount::{
    mount_writable, roll_back_by_a_forward_publish, RollbackError, RollbackTarget, RolledBack,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_harness::memory_pool::MemoryPool;

const SECOND_FILE_BYTES: usize = 4100;
const THIRD_FILE_BYTES: usize = 2500;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn second_content() -> Vec<u8> {
    content_of(SECOND_FILE_BYTES, 3)
}

fn third_content() -> Vec<u8> {
    content_of(THIRD_FILE_BYTES, 11)
}

fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// 固定脚本到 C 的池（D 之前的样子）。
struct ThroughTheThirdPublish {
    pool: BuiltPool,
}

/// 固定脚本到 C：A、B（实例 1）、重开取号 2、写行、暖机两次、C（实例 2）。
fn build_through_third_publish(tag: &str) -> ThroughTheThirdPublish {
    let mut pool = build_pool(tag);
    overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(2));
    ThroughTheThirdPublish { pool }
}

fn first_root() -> RollbackTarget {
    RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
    }
}

/// 挂着的时候回退：交回回退的结局，做成时 `pool.output` 已换成回退那次发布之后的一版。
fn roll_back(pool: &mut BuiltPool, target: RollbackTarget) -> Result<RolledBack, RollbackError> {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &publish_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        target,
    )
}

/// 池级 checker 一条违例都没有；交回全部判定，调用方再点名要真被判过（不是「不适用」）的那几条。
fn verdicts_without_any_violation(
    image: &MemoryPool,
    step: &str,
) -> Vec<(&'static str, InvariantVerdict)> {
    let verdicts = check_pool_image(image);
    let violated: Vec<(&str, &InvariantVerdict)> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, verdict)| (*invariant, verdict))
        .collect();
    assert!(
        violated.is_empty(),
        "{step}：池级 checker 一条违例都没有：{violated:?}"
    );
    verdicts
}

/// 验收「崩在 D 的每个崩溃点」：录制流里 D 那一段的每一个前缀各恢复一次——D 的记录还没持久（它那一段写之前、屏障之前）⇒ 恢复到 C (2, 8)、
/// 读回第三次的内容；D 的记录已持久、根槽还没 ⇒ 恢复由记录重建 D 的根（D23（journal 的角色与格式） 已定项 15）；根槽已持久 ⇒ 择 D。
/// 后两种都读回第一次的内容；前缀越长只会从前一种走到后一种、不回头。每个前缀 checker 全绿。
#[test]
fn every_crash_point_of_the_rollback_publish_recovers_to_the_third_version_or_to_the_rollback_publish_and_every_image_is_green(
) {
    let ThroughTheThirdPublish { mut pool, .. } =
        build_through_third_publish("step-four-forward-crash-points");
    let operations_before = pool.retained_operations().len();
    roll_back(&mut pool, first_root()).expect("回退到 A");
    let operations = pool.retained_operations();
    assert!(operations.len() > operations_before + 10, "D 写了一段");
    let mut recovered_to_the_rollback_publish_at: Option<usize> = None;
    for prefix in operations_before..=operations.len() {
        let mut image =
            MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
        image.apply(&operations[..prefix]);
        let report = recover(&image, JournalPolicy::Consult);
        let (effective_root, content) = match &report.outcome {
            RecoveryOutcome::FileRead { content, .. } => (
                report.effective_root.expect("读回了文件就有实际走的那条根"),
                content.clone(),
            ),
            other @ (RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. }) => {
                panic!("前缀 {prefix}：恢复要读回文件，交回 {other:?}")
            }
        };
        if effective_root == (InstanceGeneration(2), CheckpointTxg(9)) {
            assert_eq!(content, file_content(), "前缀 {prefix}：D 读回第一次的内容");
            recovered_to_the_rollback_publish_at.get_or_insert(prefix);
        } else {
            assert_eq!(
                effective_root,
                (InstanceGeneration(2), CheckpointTxg(8)),
                "前缀 {prefix}：D 没立住就是 C"
            );
            assert_eq!(
                content,
                third_content(),
                "前缀 {prefix}：C 读回第三次的内容"
            );
            assert!(
                recovered_to_the_rollback_publish_at.is_none(),
                "前缀 {prefix}：恢复到过 D 之后前缀更长又回到 C"
            );
        }
        verdicts_without_any_violation(&image, &format!("D 的前缀 {prefix}"));
    }
    let recovered_to_the_rollback_publish_at =
        recovered_to_the_rollback_publish_at.expect("整条 D 落完之后恢复到 D");
    assert!(
        recovered_to_the_rollback_publish_at < operations.len(),
        "D 的根槽写之前就已经恢复到 D（记录重建那一格）：从前缀 {recovered_to_the_rollback_publish_at} 起"
    );
}
