//! checker 档模块：crash_injection
//! 崩溃注入第二截那一次发布在「挂载续做推满仍放不下」那一版上走哪一臂（判决 `research/prompts/m3-formal-code-b20-r1-main-verification.md`
//! 第二节 K2 H2，改法 b）：第一个文件那条记录还在树里、nlink = 0（正在删），覆盖写在这一版上被写入口拒成写一个正在删的 inode
//! （D8（核心索引结构） 已定项 16 第 4 条），发布臂要建一个 inode；这一版本来就挤，建 inode 被单元区墙拒，是模型认得的拒。
//! 盘面照 harness 档 `admission_raises_the_floor_before_refusing.rs` 的造法（不为环窗口补足的窄盘、准入关着），
//! 覆盖写次数、单元数与批数取攻方探针 `research/prompts/m3-formal-code-b20-r1-opus-model/probes/b20_opus_k2_short_continuation.rs`
//! 量到的第一个短续做盘面，单元区槽数换成分得开那一支建几个的那一档（`UNIT_AREA_SLOTS`）。

#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
mod common_admission;

use common_admission::{
    plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    OVERWRITE_BYTES,
};
use singlefs_checker_tier::crash_injection::{
    mount_and_publish_once_on_the_crash_image, CrashPoint, WritableMountOutcome,
};
use singlefs_core::address::InodeNumber;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{mount_writable_with_space_admission, DeletionQueueContinuationOutcome};
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::transaction::{
    publish_new_inodes, publish_unlink_intent, publish_unlink_step, PoolVersion, PoolWriter,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::history::{
    HistoryDeviceWidth, HistoryExecution, HistorySeed, PerStepChecker, StepPosition,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::model::{ModelRefusalReason, ObservedRefusalReason};
use singlefs_harness::scenario::FIXED_WRITE_TIME_SECONDS;

/// 窄盘的单元区槽数（改法 B1，判决 `research/prompts/m3-formal-code-b21-r1-main-verification.md` 第二节「K2 改法 b 判别力」与第三节第 2 条）：
/// 续做照样推满仍放不下、第一个文件那条记录 nlink = 0，而这一档上建 0 个 inode 的空发布落得下、建 1 个落不下——
/// 攻方量过 136 槽时正确判法 `Explained(UnitAreaWall)`、只看 Some 是 `Unexplained`、那一支建 0 个是 `Published`
/// （`research/prompts/m3-formal-code-b21-r1-opus-model/out/k2b.txt` 里 `slots=136` 的三行）。128 槽（攻方探针按 100..=320 步 2 扫出的第一个短续做盘面）
/// 上建 0 个也被落点拒（`PlacementRefused`）、照样记 `UnitAreaWall`，分不开那一支建几个。
const UNIT_AREA_SLOTS: u64 = 136;

/// 第一个文件之后经挂着的会话覆盖写几次。
const OVERWRITES_BEFORE_THE_UNLINK: usize = 3;

/// 顺序写之后第一个文件占几个数据单元。
const DATA_UNITS_OF_THE_FIRST_FILE: usize = 14;

/// unlink 意图之后发几批（每批释放一个数据单元）再崩。
const DELETION_BATCHES_AFTER_THE_INTENT: usize = 1;

/// 在不补足的窄盘上起池、第一个文件覆盖写几次、顺序写长到几个数据单元，再不经会话发 unlink 意图与之后几批（批大小 1）就崩：
/// 交回崩在那一刻的整份盘面（照 `admission_raises_the_floor_before_refusing.rs` 的 `pool_crashed_between_deletion_batches`）。
fn crash_image_between_the_deletion_batches() -> MemoryPool {
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            HistoryDeviceWidth::UnitAreaOf384Slots,
            UNIT_AREA_SLOTS,
        );
    for _ in 0..OVERWRITES_BEFORE_THE_UNLINK {
        pool.overwrite(OVERWRITE_BYTES)
            .expect("准入关着、单元区 136 槽：覆盖写落得下");
    }
    pool.sequential_write((DATA_UNITS_OF_THE_FIRST_FILE - 1) * data_unit_payload_capacity() + 100)
        .expect("准入关着、单元区 136 槽：顺序写落得下");
    let parameters = pool.parameters.clone();
    let write_time_seconds = pool.write_time_seconds;
    let session = pool.session.as_mut().expect("顺序写之后会话还开着");
    let mut current = session
        .current
        .version_of_trees()
        .expect("第一个文件之后的一版有树")
        .clone();
    let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
    current = publish_unlink_intent(
        &mut writer,
        &mut session.allocator,
        &current,
        InodeNumber(FIRST_INODE_NUMBER),
        write_time_seconds,
        session.instance,
    )
    .expect("意图那次发布落得下");
    for _ in 0..DELETION_BATCHES_AFTER_THE_INTENT {
        current = publish_unlink_step(
            &mut writer,
            &mut session.allocator,
            &current,
            InodeNumber(FIRST_INODE_NUMBER),
            UnlinkBatchSize::CappedForTests(
                UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在只供测试的批大小范围里"),
            ),
            session.instance,
        )
        .expect("一批释放一个数据单元，落得下");
    }
    drop(writer);
    pool.session = None;
    pool.image()
}

/// 判法里 nlink 那一半（`crash_injection.rs` 的 `first_file_is_a_live_record`）：第一个文件那条记录在树里而 nlink = 0 时，
/// 发布臂建一个 inode，在这份挤的盘面上被单元区墙拒（`Explained(UnitAreaWall)`），不是覆盖写被写入口拒成写一个正在删的 inode（`Unexplained`）。
/// 判别力：判法只看 `inode_record` 是不是 Some（`crates/mutations.tsv` 一行）时走覆盖写，拒因是 `Unexplained`；那一支建 0 个 inode
/// （`crates/mutations.tsv` 一行）时空发布落得下、结局是 `Published`，不是 `PublishRefused`，红在解构结局那一处。
/// 「建 0 个」那一行只在空发布落得下的盘面上分得开：前提块另挂一次同一份盘面、真发一次建 0 个的空发布，要做成（改法 B1-c，判决
/// `research/prompts/m3-formal-code-b23-r1-main-verification.md` 第二节「K2 B1 对别的盘面参数」与第三节第 2 条）。攻方量过：用例绿的 76 份盘面上
/// 这条前提成立与「建 0 个」变异红逐份同真同假；单元区 128..=133 槽（覆盖写 3 次、14 个单元、1 批）用例照绿而「建 0 个」不红，
/// 这几档上空发布被落点拒、这条前提为假（`research/prompts/m3-formal-code-b23-r1-opus-model/out/b1-slots.txt`、`b1-neighbourhood.txt`）。
/// 判别力：`UNIT_AREA_SLOTS` 换成 133（`crates/mutations.tsv` 一行）时用例红在这条前提上。
// crash-case-check:not-a-crash-case 靶向用例只在写死的一份崩溃盘面上挂一次、发一次，不枚举崩溃态
#[test]
fn the_publish_after_a_mount_whose_deletion_continuation_stays_short_creates_an_inode_and_is_refused_at_the_unit_area_wall(
) {
    let crash_image = crash_image_between_the_deletion_batches();
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let parameters = execution.device_width.parameters();
    {
        let mut devices = plain_devices_on(&crash_image);
        let mounted = mount_writable_with_space_admission(
            &parameters,
            &mut devices,
            execution.space_admission,
        )
        .expect("前提：挂载续做推满仍放不下时挂载照样做成可写");
        assert!(
            matches!(
                mounted.output.deletion_queue_continuation_outcome,
                DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { .. }
            ),
            "前提：挂载续做推满抬 F 仍放不下、队列留着：{:?}",
            mounted.output.deletion_queue_continuation_outcome
        );
        let PoolVersion::WithFile(current) = &mounted.current else {
            panic!("前提：挂载交回的现行那一版是带文件的一版");
        };
        assert_eq!(
            current.inode_record.map(|record| record.nlink),
            Some(NLINK_OF_AN_INODE_BEING_DELETED),
            "前提：第一个文件那条记录还在树里、正在删（nlink = 0）"
        );
    }
    {
        let mut devices = plain_devices_on(&crash_image);
        let mut mounted = mount_writable_with_space_admission(
            &parameters,
            &mut devices,
            execution.space_admission,
        )
        .expect("前提：同一份盘面再挂一次照样做成可写");
        let PoolVersion::WithFile(current) = &mounted.current else {
            panic!("前提：再挂一次交回的现行那一版是带文件的一版");
        };
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let publish_creating_no_inode = publish_new_inodes(
            &mut writer,
            &mut mounted.allocator,
            current,
            0,
            FIXED_WRITE_TIME_SECONDS,
            mounted.output.instance,
        );
        assert!(
            publish_creating_no_inode.is_ok(),
            "前提：同一份盘面新挂一次，那一支建 0 个 inode 的空发布做得成（「建 0 个」变异在这份盘面上才分得开）：{:?}",
            publish_creating_no_inode.err()
        );
    }
    let mount_run = mount_and_publish_once_on_the_crash_image(
        &crash_image,
        &parameters,
        execution,
        HistorySeed(0),
        &CrashPoint {
            segment_index: 0,
            persisted_within_the_segment: Vec::new(),
            step: StepPosition::StartingPoint,
            operation_kind: None,
        },
    );
    let WritableMountOutcome::PublishRefused { member, reason } = &mount_run.outcome else {
        panic!(
            "这份盘面上挂载做成、之后那一次发布被拒：{:?}",
            mount_run.outcome
        );
    };
    assert_eq!(
        *reason,
        ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall),
        "正在删的第一个文件那一版上，发布臂建一个 inode、被单元区墙拒（模型认得的拒），不是覆盖写被拒成写一个正在删的 inode：{member}"
    );
}
