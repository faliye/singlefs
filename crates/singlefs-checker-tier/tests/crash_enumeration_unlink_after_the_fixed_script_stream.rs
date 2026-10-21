//! checker 档模块：crash、crash_state_enumeration、crash_identity、crash_amplification
//! 里程碑三三·4（用户 2026-10-18 定「两条都新写」那一条）：固定脚本（`crash_enumeration_fixed_script_stream.rs` 的「取号 → 暖机 → A → B →
//! 重开可写挂载（取号、写行、暖机两次）→ C → 挂着回退到 A（D）→ 五次覆盖写 → 抬 F 到 11 → E」，不卸载）之后，经会话入口 unlink 第一个文件、
//! 走空删除队列（`mounted_session::UserChange::Unlink`：先走空、发意图、再走空到底，D8（核心索引结构） 已定项 16 第 6 条；E 一个数据单元，
//! 产品批大小下意图之后直接是最后一批），会话发完之后对整段回收的那一段发 discard（两块盘各一次）。整条录制流按 D13（验证路线） 已定项 4
//! 枚举崩溃状态：每个状态跑恢复 + 多版本 oracle、池级 checker、记录核对器；版本表是固定脚本那十五版有文件，意图那一版起没有文件
//! （D8（核心索引结构） 已定项 16 第 4 条），最后一批不进表、照比它旧的最新那一项（意图那一版）判。另在工作线程上每个状态可写挂载一次
//! （公共模块 `tests/common_crash_point_observers/mod.rs` 的续做观察者，与多记录流之后那条删除流同一个）：续做把队列走到空，步数等于恢复落到的
//! 那一版还差几步，挂完的盘过池级 checker（D8（核心索引结构） 已定项 5「续做是每次恢复都跑的常规动作」、已定项 16 第 7 条 ③）。
//!
//! 节点按 `<流程>.<周期>.<步>` 编码（崩溃放量那一路，`crash_amplification::CrashPointRecorder`）：固定脚本那几步由公共模块
//! `tests/common_crash_points/mod.rs` 照旧设点（与 `crash_amplification_fixed_script_through_unmount.rs` 是同一串节点，共享到 E），
//! 会话 unlink 那一步也设在公共模块（`unlink_the_first_file_through_the_session_step`）；挂在标准前缀上的观察者照登记表跑。
//!
//! **比已有的流多罩了什么**：已有的删除类崩溃流都接在一次挂载里直接调发布入口的流后面（多记录流之后、快照之后），没有一条经会话入口 unlink，
//! 也没有一条接在重开、回退、抬 F 之后；这里意图那次（13 个单元）与最后一批（14 个单元：删 inode 记录与最后一个单元、退役树开第一片容器）
//! 落在第二个实例、回退之后又抬过 F 的池上，单元段、记录段、根槽写、之后的系统配置槽轮换，与会话发完之后的 discard 段（两次原地写，取三态）
//! 第一次被枚举到。多跑的一步是每个状态上的可写挂载（取号、写行、暖机、续做）与挂完的盘上那一遍池级 checker。
//! 与固定脚本那条流的基线镜像相同、写表与段序列只在 E 之后不同；版本表、被判的根不同，崩溃点重放那一路不复用它的块，另立全量。
//!
//! 快档是甲二（每段原地写各取它的几态、任意组合 × 单元写全不落或全落，`crash::quick_tier_expansion`）；全量标 ignored，登记成崩溃枚举用例。
//! 判器先判、形状后核：改坏「最后一批与删意图同事务」（D8（核心索引结构） 已定项 5）时最后一批之后号还在队列里、会话走空再发一步被拒，
//! 录流照样录完（公共模块那一步不断言发布路径的拒绝），红在判器上（池级 checker I-9.5、可写挂载续做），形状与计数在判干净之后才核。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_crash_point_observers;
mod common_crash_points;

use common::{build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use common_crash_point_observers::{
    assert_the_mounted_disks_and_the_continuation_numbers_were_checked_on_every_state,
    observe_the_mount_continuation, DeletionPublishContinuation, MountContinuationInputs,
};
use common_crash_points::{
    unlink_the_first_file_through_the_session, UnlinkThroughTheSession, UNLINKED_FILE,
};
use singlefs_checker_tier::crash::{
    crash_state_count, crash_state_count_with_torn_in_place_overwrites, full_expansion,
    quick_tier_expansion, CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion,
    CrashPointReplayTally, CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_amplification::{
    ObserveACrashStateHandingBackAnImage, ObserverDeduplication,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
    CrashPointReplayObserverHandingBackAnImage,
};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_fixed_script_stream.rs";
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, MountOutput, Mounted,
    RollbackTarget, ShadowLedger,
};
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::recovery::RecoveryReport;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, CrashImage, MemoryPool, PublishedVersion,
    PublishedVersionWithoutAFile, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

const SECOND_FILE_BYTES: usize = 4100;
const THIRD_FILE_BYTES: usize = 2500;
/// E 的数据单元落回最低的可再分配偶数槽对：mkfs 实例表那两槽（释放代 5，抬 F 到 11 回收了），同固定脚本。
const REUSED_DATA_UNIT_SLOT_OF_E: u64 = 50176;
/// 回退之后五次覆盖写（txg 10–14）各用的内容种子，同固定脚本。
const SEEDS_OF_THE_FIVE_OVERWRITES_AFTER_THE_ROLLBACK: [usize; 5] = [13, 17, 19, 23, 29];
/// E（txg 17）的内容种子。
const SEED_OF_E: usize = 31;
/// E 那一版的 txg：意图 18、最后一批 19。
const TXG_OF_E: u64 = 17;

/// 段序列：前 71 段同固定脚本到 E（`crash_enumeration_fixed_script_stream.rs` 的到 E 再卸载那一串去掉卸载那 9 段；取号写两轮，开头与 B 之后重开
/// 两次取号各两段）；之后意图那次
/// 26 个单元写（13 个单元两盘各一份）、记录 2、根槽 1、轮换 2，最后一批 28 个单元写（14 个单元）、记录 2、根槽 1、轮换 2，
/// 会话发完之后的 discard（两块盘各一次）自成一段。
const SEGMENT_SIZES: [usize; 80] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2, 16, 2, 1, 2,
    24, 2, 1, 2, 24, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 2,
    16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 26, 2, 1, 2, 28, 2, 1, 2, 2,
];
/// 段序列里最后那一段（discard）之前的段数：最后一批的轮换为止。
const SEGMENTS_THROUGH_THE_FINAL_BATCH: usize = 79;
/// 写表的写数：到 E 441（同固定脚本：到 D 199 + 五次覆盖写 165 + 抬 F 44 + E 33；取号写一轮时到 D 195、到 E 437）+ 意图 31 + 最后一批 33 + discard 2。
const WRITES: usize = 441 + 31 + 33 + 2;

/// 甲二快档的状态数：到 E 那几次发布同固定脚本，意图与最后一批各 13（上一次的轮换 3² − 1 = 8、只有单元写的那一段 1、记录 3、根槽 1），
/// 最后一次根槽写之后是最后一批的轮换 8 与 discard 那一段 8（两次原地写各取三态），再加全部持久那一个。
/// 取号写两轮之后两次取号各多一个 2 写段（3² − 1 = 8）：取号段归它之后那次发布，暖机 txg 1 12 → 20、写行 txg 5 21 → 29，278 → 294。
const QUICK_TIER_STATES: u64 = 294;
/// 甲二快档每次写只取两态时的状态数（每个两写原地段 3；取号写一轮时 163）。
const QUICK_TIER_STATES_WITH_TWO_STATES_PER_WRITE: u64 = 169;
const QUICK_TIER_STATES_BY_PUBLISH: &str = "instance1_txg1=20 instance1_txg2=12 \
     instance1_txg3=13 instance1_txg4=13 instance2_txg5=29 instance2_txg6=13 instance2_txg7=13 \
     instance2_txg8=13 instance2_txg9=13 instance2_txg10=13 instance2_txg11=13 instance2_txg12=13 \
     instance2_txg13=13 instance2_txg14=13 instance2_txg15=21 instance2_txg16=13 instance2_txg17=13 \
     instance2_txg18=13 instance2_txg19=13 after_the_last_root=16 every_write_persisted=1";
/// 全量每次写只取两态时的闭式 1 + Σ(2^|段| − 1)，八十段（取号写一轮时七十八段、2_013_790_337，两次取号各 +3）。
const FULL_STATES_WITH_TWO_STATES_PER_WRITE: u64 = 2_013_790_343;
/// 全量枚举的状态数：取三态的是二十四个系统配置槽 2 写段与 discard 那一段，每段 3² − 1 比两态多 5（取号写一轮时二十二个，23 × 5）。
const FULL_STATES: u64 = FULL_STATES_WITH_TWO_STATES_PER_WRITE + 25 * 5;
/// 全量的状态按发布分：到 E 同固定脚本（取号段归它之后那次发布：暖机 txg 1 12 → 20、写行 txg 5 262163 → 262171）；
/// 意图 8 + (2^26 − 1) + 3 + 1，最后一批 8 + (2^28 − 1) + 3 + 1，最后一次根槽写之后 8 + 8。
const FULL_STATES_BY_PUBLISH: &str = "instance1_txg1=20 instance1_txg2=12 \
     instance1_txg3=16777227 instance1_txg4=16777227 instance2_txg5=262171 instance2_txg6=65547 \
     instance2_txg7=65547 instance2_txg8=16777227 instance2_txg9=16777227 instance2_txg10=268435467 \
     instance2_txg11=268435467 instance2_txg12=268435467 instance2_txg13=268435467 \
     instance2_txg14=268435467 instance2_txg15=65555 instance2_txg16=65547 instance2_txg17=268435467 \
     instance2_txg18=67108875 instance2_txg19=268435467 after_the_last_root=16 every_write_persisted=1";

fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

fn third_content() -> Vec<u8> {
    (0..THIRD_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
        .collect()
}

fn later_content(seed: usize) -> Vec<u8> {
    (0..3000 + seed)
        .map(|index| u8::try_from((index * 5 + seed) % 251).expect("小于 256"))
        .collect()
}

fn version(instance: u32, txg: u64, content: Vec<u8>) -> PublishedVersion {
    PublishedVersion {
        instance: InstanceGeneration(instance),
        checkpoint_txg: CheckpointTxg(txg),
        content,
    }
}

/// 固定脚本到 E 的十五版带文件的版本（A、B、重开那三次、C、D、五次覆盖写、抬 F 的两次空发布、E），按发布的次序。
fn versions_of_the_fixed_script_through_e() -> Vec<PublishedVersion> {
    let mut versions = vec![
        version(1, 3, file_content()),
        version(1, 4, second_content()),
    ];
    versions.extend((5..=7).map(|txg| version(2, txg, second_content())));
    versions.push(version(2, 8, third_content()));
    versions.push(version(2, 9, file_content()));
    for (txg, seed) in (10..).zip(SEEDS_OF_THE_FIVE_OVERWRITES_AFTER_THE_ROLLBACK) {
        versions.push(version(2, txg, later_content(seed)));
    }
    let latest_before_the_floor_raise =
        later_content(SEEDS_OF_THE_FIVE_OVERWRITES_AFTER_THE_ROLLBACK[4]);
    versions.extend((15..=16).map(|txg| version(2, txg, latest_before_the_floor_raise.clone())));
    versions.push(version(2, TXG_OF_E, later_content(SEED_OF_E)));
    versions
}

/// 覆盖写（不录崩溃点那一路）：把现行那一版的内容换成 `content`，现行版本换成这一次。
fn overwrite(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &pool.output,
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

/// 固定脚本到 E（不录崩溃点那一路，写法照 `crash_enumeration_fixed_script_stream.rs` 的 `prepare`）：交回 B 之后重开那一次可写挂载写出的东西
/// （会话接在它后面）与 E 那一版。
fn run_the_fixed_script_through_e(pool: &mut BuiltPool) -> (MountOutput, TransactionOutput) {
    overwrite(pool, &second_content(), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let Mounted {
        output: mount_output,
        allocator,
        current: current_after_the_mount,
    } = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = allocator;
    pool.output = current_after_the_mount
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    overwrite(pool, &third_content(), InstanceGeneration(2));
    {
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        roll_back_by_a_forward_publish(
            &parameters(),
            open_devices,
            &mut pool.allocator,
            &mut pool.output,
            RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
            },
        )
        .expect("挂着的时候回退到 A");
    }
    for seed in SEEDS_OF_THE_FIVE_OVERWRITES_AFTER_THE_ROLLBACK {
        overwrite(pool, &later_content(seed), InstanceGeneration(2));
    }
    {
        let mut current = pool.output.clone();
        let raise_devices = pool.devices.as_mut().expect("镜像还开着");
        let raised = raise_rollback_floor(
            &parameters(),
            raise_devices,
            &mut pool.allocator,
            &mut current,
            CheckpointTxg(11),
            ShadowLedger::On,
        )
        .expect("抬 F");
        assert_eq!(raised.publishes.len(), 2, "txg 15 落盘 0、txg 16 落盘 1");
        pool.output = current;
    }
    let e = overwrite(pool, &later_content(SEED_OF_E), InstanceGeneration(2));
    (mount_output, e)
}

/// E 那一版：txg 17、数据单元落回 mkfs 实例表那两槽（同固定脚本）。
fn assert_e_is_the_version_of_the_fixed_script(e: &TransactionOutput) {
    assert_eq!(
        (
            e.root.checkpoint_txg.0,
            e.data_pointers[0].locations[0].slot.0
        ),
        (TXG_OF_E, REUSED_DATA_UNIT_SLOT_OF_E),
        "E 是 txg 17、数据单元落回 mkfs 实例表那两槽（释放代 5，抬 F 到 11 回收了）"
    );
}

/// 会话 unlink 发成的两版（意图、最后一批）不看队列空不空的那几样：意图那一版 nlink = 0、号在队列里、一个数据单元都还在；
/// 最后一批删了记录与最后一个单元。最后一批之后队列空不空、会话有没有拒在这里不判：那是「最后一批与删意图同事务」，留给判器红。
fn assert_the_shape_of_the_unlink_steps(unlink: &UnlinkThroughTheSession) {
    let [intent, final_batch] = &unlink.versions_published[..] else {
        panic!(
            "一个数据单元的文件经会话 unlink 发成意图与最后一批两版，实际 {} 版",
            unlink.versions_published.len()
        );
    };
    let intent_record = intent
        .inode_record
        .expect("意图那一版第一个文件那条记录还在");
    assert_eq!(
        (
            intent.root.checkpoint_txg.0,
            intent_record.nlink,
            intent.deleted_inodes.clone(),
            intent.data_pointers.len()
        ),
        (
            TXG_OF_E + 1,
            NLINK_OF_AN_INODE_BEING_DELETED,
            vec![UNLINKED_FILE],
            1
        ),
        "意图那一版：txg 18、nlink = 0、号在队列里、一个数据单元都还在"
    );
    assert!(
        final_batch.root.checkpoint_txg.0 == TXG_OF_E + 2
            && final_batch.inode_record.is_none()
            && final_batch.data_pointers.is_empty(),
        "最后一批（txg 19）删记录、释放最后一个单元"
    );
}

/// 可写挂载续做观察者另读的输入：固定脚本那十五版有文件，意图那一版落到之后续做还差 1 步，最后一批 0 步。
fn mount_continuation_inputs_of(
    versions: &[PublishedVersion],
    unlink: &UnlinkThroughTheSession,
) -> MountContinuationInputs {
    let steps = unlink.versions_published.len();
    MountContinuationInputs {
        versions_with_a_file: versions
            .iter()
            .map(|version| (version.instance, version.checkpoint_txg))
            .collect(),
        deletion_publishes: unlink
            .versions_published
            .iter()
            .enumerate()
            .map(|(step_index, step)| DeletionPublishContinuation {
                version: (step.root.instance, step.root.checkpoint_txg),
                continuation_steps_from_this_version: steps - 1 - step_index,
            })
            .collect(),
    }
}

struct Prepared {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    /// 最后一批那一次的根槽写。
    judged_root_index: usize,
    /// 固定脚本那十五版有文件。
    versions: Vec<PublishedVersion>,
    /// 意图那一版起没有文件（最后一批不进表，照比它旧的最新那一项判）。
    versions_without_a_file: Vec<PublishedVersionWithoutAFile>,
    unlink: UnlinkThroughTheSession,
    mount_continuation_inputs: MountContinuationInputs,
}

impl Prepared {
    fn version_table(&self) -> CrashPointReplayVersionTable<'_> {
        CrashPointReplayVersionTable::new(&self.versions, &self.versions_without_a_file)
    }
}

/// 交给判器之前只核录流那一边不随「最后一批与删意图同事务」变的形状：E、会话发成的两版、最后一批为止的段序列、十九条根槽写。
fn prepare(tag: &str) -> Prepared {
    let mut pool = build_pool(tag);
    let (mount_output, e) = run_the_fixed_script_through_e(&mut pool);
    assert_e_is_the_version_of_the_fixed_script(&e);
    let unlink = unlink_the_first_file_through_the_session(&mut pool, mount_output);
    assert_the_shape_of_the_unlink_steps(&unlink);
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    assert_eq!(
        sizes.get(..SEGMENTS_THROUGH_THE_FINAL_BATCH),
        Some(&SEGMENT_SIZES[..SEGMENTS_THROUGH_THE_FINAL_BATCH]),
        "最后一批的轮换为止的段序列：前 71 段同固定脚本到 E，之后意图 26、2、1、2，最后一批 28、2、1、2"
    );
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        root_indexes.len(),
        19,
        "固定脚本到 E 十七条、意图与最后一批各一条根槽写"
    );
    let versions = versions_of_the_fixed_script_through_e();
    let intent = unlink
        .versions_published
        .first()
        .expect("会话至少发成意图那一版");
    let versions_without_a_file = vec![PublishedVersionWithoutAFile {
        instance: intent.root.instance,
        checkpoint_txg: intent.root.checkpoint_txg,
    }];
    let mount_continuation_inputs = mount_continuation_inputs_of(&versions, &unlink);
    Prepared {
        base,
        writes,
        segments,
        judged_root_index: *root_indexes.last().expect("至少一条根槽写"),
        versions,
        versions_without_a_file,
        unlink,
        mount_continuation_inputs,
    }
}

/// 判器那一半：oracle、不看 journal 那一遍、走读、记录核对器两条、池级 checker 每条不变量都不判违例；删除队列那几条
/// （I-9.5 队列里的号仍在树里、I-9.18 队列里的号有谱系上的墓碑、I-9.24 队列至多一条 key）真被评估过。
fn assert_clean(tally: &CrashPointReplayTally) {
    assert_eq!(tally.violations, 0, "oracle：{:?}", tally.first_violation);
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0, "走读一次都不失败");
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant} 判违例：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
    for must_evaluate in ["I-9.5", "I-9.18", "I-9.24"] {
        assert!(
            tally
                .checker_evaluated_states
                .get(must_evaluate)
                .copied()
                .unwrap_or(0)
                > 0,
            "{must_evaluate} 至少在一个状态上真被评估过：{}",
            tally.checker_counts_by_invariant()
        );
    }
}

/// 可写挂载续做观察者那一份计数（判干净之后核）：每个状态都挂过一次、续做 0 步或 1 步、落到意图那一版续做 1 步的状态真跑到了、
/// 挂完的盘都过了池级 checker、续做那一步核了代与事务号。
fn assert_every_state_was_mounted_and_continued_to_empty(tally: &CrashPointReplayTally) {
    let counts = &tally.observer_counts;
    assert_eq!(
        tally.observed_states, tally.states,
        "每个状态都在工作线程上看过"
    );
    assert_eq!(
        counts.count_named("mount_continuation_steps_0")
            + counts.count_named("mount_continuation_steps_1"),
        tally.states,
        "每个状态都可写挂载过一次，续做 0 步或 1 步"
    );
    assert!(
        counts.count_named("mount_continuation_steps_1") > 0
            && counts.count_named("mount_before_the_first_file") > 0,
        "恢复落到意图那一版、挂载续做最后一批的状态，与落到 A 之前的状态都真跑到了：{counts:?}"
    );
    assert_the_mounted_disks_and_the_continuation_numbers_were_checked_on_every_state(
        counts,
        tally.states,
    );
}

/// 录流那一边随「最后一批与删意图同事务」变的形状（判干净之后核）：会话没拒、最后一批之后队列空了，会话发完之后的 discard 自成最后一段
/// （两块盘各一次）；整条段序列与写数。
fn assert_the_tail_of_the_stream(prepared: &Prepared) {
    let final_batch = prepared
        .unlink
        .versions_published
        .last()
        .expect("会话发成了意图与最后一批");
    assert!(
        prepared
            .unlink
            .refusal_after_the_versions_published
            .is_none()
            && final_batch.deleted_inodes.is_empty(),
        "会话把队列走到空、没拒：{:?}，最后一批之后队列 {:?}",
        prepared.unlink.refusal_after_the_versions_published,
        final_batch.deleted_inodes
    );
    let sizes: Vec<usize> = prepared.segments.iter().map(Vec::len).collect();
    assert_eq!(sizes, SEGMENT_SIZES, "整条段序列");
    assert_eq!(prepared.writes.len(), WRITES, "整张写表的写数");
    let last_segment = prepared.segments.last().expect("至少一段");
    assert!(
        last_segment
            .iter()
            .all(|index| prepared.writes[*index].kind == StepKind::Discard),
        "最后一段是会话发完之后的 discard"
    );
}

fn mount_continuation_observer<'prepared>(
    prepared: &'prepared Prepared,
    observe: &'prepared ObserveACrashStateHandingBackAnImage<'prepared>,
) -> CrashPointReplayObserverHandingBackAnImage<'prepared> {
    CrashPointReplayObserverHandingBackAnImage {
        name: "mount_continuation",
        extra_inputs_digest: prepared.mount_continuation_inputs.digest(),
        observe,
        deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
    }
}

/// 按 `expansion` 枚举，每个状态在工作线程上可写挂载一次（续做观察者）。只录那一趟交回 `None`，不判。
fn enumerate(
    prepared: &Prepared,
    item_name: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> Option<CrashPointReplayTally> {
    let observe_the_mount =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_mount_continuation(
                &prepared.mount_continuation_inputs,
                image,
                report,
                counts,
            )
        };
    enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: item_name,
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        prepared.version_table(),
        expansion,
        &[],
        &[mount_continuation_observer(prepared, &observe_the_mount)],
    )
    .judged()
}

/// 平时跑的那一份（甲二快档）：每个状态恢复、oracle、池级 checker、记录核对器，再可写挂载一次核续做走到空。
#[test]
fn every_crash_state_of_the_quick_tier_of_an_unlink_after_the_fixed_script_recovers_into_the_version_table_and_the_mount_continues_to_empty(
) {
    let prepared = prepare("unlink-after-the-fixed-script-quick");
    let Some(tally) = enumerate(
        &prepared,
        "unlink_after_the_fixed_script::quick_tier",
        &quick_tier_expansion,
    ) else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_UNLINK_AFTER_THE_FIXED_SCRIPT_FAST states={} observer {:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_clean(&tally);
    assert_every_state_was_mounted_and_continued_to_empty(&tally);
    assert_the_tail_of_the_stream(&prepared);
    assert_eq!(
        (
            tally.states,
            crash_state_count(&prepared.writes, &prepared.segments, &quick_tier_expansion)
        ),
        (
            crash_state_count_with_torn_in_place_overwrites(
                &prepared.base,
                &prepared.writes,
                &prepared.segments,
                &quick_tier_expansion
            ),
            QUICK_TIER_STATES_WITH_TWO_STATES_PER_WRITE
        ),
        "甲二按崩溃点重放的枚举域数（原地覆写取三态）；每次写只取两态时的甲二闭式"
    );
    assert_eq!(tally.states, QUICK_TIER_STATES, "甲二的状态数");
    assert_eq!(
        tally.states_by_publish_text(),
        QUICK_TIER_STATES_BY_PUBLISH,
        "甲二的状态按发布分"
    );
    assert_eq!(
        tally.states_by_publish.values().sum::<u64>(),
        tally.states,
        "按发布分的各格加起来就是状态数"
    );
    assert!(
        tally.file_read_states > 0 && tally.no_file_states > 0,
        "读回文件与答没有文件的状态都有"
    );
}

/// 全量：八十段全展开，闭式 [`FULL_STATES`] 个状态，每个两遍恢复 + checker + 记录核对器 + 一次可写挂载。登记成崩溃枚举用例，
/// 门禁 checker-tier-crash-point-replay --full 在 release 下跑，认下面打印的 `CRASH_POINT_REPLAY_UNLINK_AFTER_THE_FIXED_SCRIPT` 行里
/// `exhaustive=true`；带断点续跑、可双机分片（同固定脚本那一条）。
#[test]
#[ignore = "全量 2013790468 个状态（二十亿多）、每个两遍恢复 + checker + 一次可写挂载；门禁 checker-tier-crash-point-replay --full 在 release 下跑，带断点续跑、可双机分片"]
fn full_enumeration_of_the_unlink_after_the_fixed_script_stream_is_exhaustive_and_clean() {
    let prepared = prepare("unlink-after-the-fixed-script-full");
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    let Some(tally) = enumerate(
        &prepared,
        "unlink_after_the_fixed_script::full",
        &full_expansion,
    ) else {
        return;
    };
    let checker_violations: u64 = tally.checker_violated_states.values().sum();
    println!(
        "CRASH_POINT_REPLAY_UNLINK_AFTER_THE_FIXED_SCRIPT states={} closed_form={closed_form} exhaustive={} violations={} ignored_violations={} failed={} root_persisted_states={} no_file={} file_read={} record_root_without_record={} record_claimed_state_missing_unit={} checker_violations={checker_violations} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.ignored_violations,
        tally.failed_states,
        tally.root_persisted_states,
        tally.no_file_states,
        tally.file_read_states,
        tally.record_root_without_record,
        tally.record_claimed_state_missing_unit,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_clean(&tally);
    assert_every_state_was_mounted_and_continued_to_empty(&tally);
    assert_the_tail_of_the_stream(&prepared);
    assert_eq!(tally.states, closed_form, "枚举到的状态数要等于闭式");
    assert_eq!(
        (closed_form_state_count(&prepared.segments), closed_form),
        (FULL_STATES_WITH_TWO_STATES_PER_WRITE, FULL_STATES),
        "两态闭式 1 + Σ(2^|段| − 1)；取三态的二十五段各多 5"
    );
    assert_eq!(
        tally.states_by_publish_text(),
        FULL_STATES_BY_PUBLISH,
        "全量的状态按发布分"
    );
}

/// 录制器那一份造法录下的流：起点镜像、写表、段、崩溃点、被判的根槽写（最后一批的那一条）、版本表与「没有文件的那几版」。
#[cfg(feature = "verdict-store")]
struct RecordedThroughTheCrashPointRecorder {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    crash_points: Vec<singlefs_checker_tier::crash_amplification::CrashPointSpan>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    versions_without_a_file: Vec<PublishedVersionWithoutAFile>,
}

/// 录制器那一份造法：固定脚本那十个点全由公共模块设（与 `crash_amplification_fixed_script_through_unmount.rs` 同一串入口，共享到 E），
/// 会话 unlink 那一个点也设在公共模块。
#[cfg(feature = "verdict-store")]
fn record_through_the_crash_point_recorder(tag: &str) -> RecordedThroughTheCrashPointRecorder {
    use common_crash_points::{
        mount_writable_step, overwrite_step, raise_rollback_floor_step, record_pool_build,
        roll_back_step, unlink_the_first_file_through_the_session_step,
    };
    use singlefs_checker_tier::crash_amplification::CrashPointRecorder;
    let mut pool = build_pool(tag);
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    record_pool_build(&mut recorder, &pool);
    pool.output = overwrite_step(
        &mut recorder,
        &mut pool,
        &second_content(),
        InstanceGeneration(1),
    );
    let Mounted {
        output: mount_output,
        allocator,
        current,
    } = mount_writable_step(&mut recorder, &mut pool);
    pool.allocator = allocator;
    pool.output = current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    pool.output = overwrite_step(
        &mut recorder,
        &mut pool,
        &third_content(),
        InstanceGeneration(2),
    );
    roll_back_step(
        &mut recorder,
        &mut pool,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
    );
    for seed in SEEDS_OF_THE_FIVE_OVERWRITES_AFTER_THE_ROLLBACK {
        pool.output = overwrite_step(
            &mut recorder,
            &mut pool,
            &later_content(seed),
            InstanceGeneration(2),
        );
    }
    let raised = raise_rollback_floor_step(&mut recorder, &mut pool, CheckpointTxg(11));
    assert_eq!(raised.publishes.len(), 2, "txg 15 落盘 0、txg 16 落盘 1");
    let e = overwrite_step(
        &mut recorder,
        &mut pool,
        &later_content(SEED_OF_E),
        InstanceGeneration(2),
    );
    assert_e_is_the_version_of_the_fixed_script(&e);
    pool.output = e;
    let unlink =
        unlink_the_first_file_through_the_session_step(&mut recorder, &mut pool, mount_output);
    assert_the_shape_of_the_unlink_steps(&unlink);
    let intent = unlink
        .versions_published
        .first()
        .expect("会话至少发成意图那一版");
    let versions_without_a_file = vec![PublishedVersionWithoutAFile {
        instance: intent.root.instance,
        checkpoint_txg: intent.root.checkpoint_txg,
    }];
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments, stream_indexes) =
        singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes(
            &operations[pool.mkfs_operation_count..],
            &geometry(),
        );
    let crash_points = recorder.into_crash_points(&writes, &segments, &stream_indexes);
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽写");
    RecordedThroughTheCrashPointRecorder {
        base,
        writes,
        segments,
        crash_points,
        judged_root_index,
        versions: versions_of_the_fixed_script_through_e(),
        versions_without_a_file,
    }
}

/// 崩溃放量的形态：按环境跑一趟，健康的流零红；配置开了 GPU 就是 GPU 判器判。版本表带「没有文件的那几版」（意图那一版）。
/// 节点：每个屏障段一个点，十一个入口各管的写数与固定脚本到 E 那几个入口相同、会话 unlink 那一个入口在最后；全设在公共模块。
/// 登记表挂在标准前缀节点上的观察者（可写挂载续做）照表跑。展开上限默认 6。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_of_the_unlink_after_the_fixed_script_stream_is_clean() {
    let recorded = record_through_the_crash_point_recorder(
        "crash-amplification-unlink-after-the-fixed-script",
    );
    assert_eq!(
        recorded.crash_points.len(),
        recorded.segments.len(),
        "每个屏障段一个崩溃点"
    );
    let mut entries: Vec<(&str, usize)> = Vec::new();
    for point in &recorded.crash_points {
        let entry = point.name.split('.').next().expect("名字以入口起头");
        let count = point.writes.end - point.writes.start;
        match entries.last_mut() {
            Some((last, total)) if *last == entry => *total += count,
            _ => entries.push((entry, count)),
        }
    }
    assert_eq!(
        entries,
        vec![
            ("acquire_instance_and_warm_up", 14),
            ("publish_first_file", 29),
            ("publish_overwrite", 29),
            ("mount_writable", 69),
            ("publish_overwrite", 29),
            ("roll_back_by_a_forward_publish", 29),
            ("publish_overwrite", 33 * 5),
            ("raise_rollback_floor", 44),
            ("publish_overwrite", 33),
            ("publish_unlink_through_the_session", 31 + 33 + 2),
        ],
        "十个入口各管的写数（连着的五次覆盖写归拢成一行）：到 E 同固定脚本，会话 unlink 那一个管意图、最后一批与 discard"
    );
    assert!(
        recorded
            .crash_points
            .iter()
            .all(|point| point.code_file_in_repository == common_crash_points::THIS_FILE),
        "十个入口都在公共模块里设点"
    );
    common_crash_point_observers::amplify_a_flow_through_the_standard_prefix(
        "unlink-after-the-fixed-script",
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        &recorded.versions,
        &recorded.versions_without_a_file,
        recorded.crash_points,
        6,
    );
}

/// 两份造法写表摘要相等：崩溃点重放那一份（[`prepare`]，不录崩溃点）与录制器那一份录下的起点镜像与整张带屏障的写表逐字节相同、
/// 段一样切、被判的根槽写与版本表相同。摘要打一行 `CRASH_POINT_REPLAY_WRITE_TABLE`。
#[cfg(feature = "verdict-store")]
#[test]
fn the_unlink_after_the_fixed_script_writes_the_same_table_through_the_crash_point_recorder_as_without_it(
) {
    use singlefs_checker_tier::crash_identity::{step_write_table_with_barriers, SectorImage};
    let without_the_recorder =
        prepare("unlink-after-the-fixed-script-write-table-without-the-recorder");
    let through_the_recorder = record_through_the_crash_point_recorder(
        "unlink-after-the-fixed-script-write-table-through-the-recorder",
    );
    let digests_of = |base: &MemoryPool, writes: &[RetainedWrite], segments: &[Vec<usize>]| {
        (
            SectorImage::of_base(base).digest(),
            step_write_table_with_barriers(writes, segments, 0..writes.len()),
        )
    };
    let (base_digest, write_table_digest) = digests_of(
        &without_the_recorder.base,
        &without_the_recorder.writes,
        &without_the_recorder.segments,
    );
    assert_eq!(
        digests_of(
            &through_the_recorder.base,
            &through_the_recorder.writes,
            &through_the_recorder.segments
        ),
        (base_digest, write_table_digest),
        "两份造法的起点镜像与整张带屏障的写表逐字节相同"
    );
    assert_eq!(
        through_the_recorder.segments, without_the_recorder.segments,
        "段一样切"
    );
    assert_eq!(
        through_the_recorder.judged_root_index, without_the_recorder.judged_root_index,
        "被判的根槽写都是最后一批的那一条"
    );
    assert_eq!(
        (
            &through_the_recorder.versions,
            &through_the_recorder.versions_without_a_file
        ),
        (
            &without_the_recorder.versions,
            &without_the_recorder.versions_without_a_file
        ),
        "版本表与「没有文件的那几版」相同"
    );
    let hexadecimal =
        |digest: &[u8]| -> String { digest.iter().map(|byte| format!("{byte:02x}")).collect() };
    println!(
        "CRASH_POINT_REPLAY_WRITE_TABLE flow=unlink-after-the-fixed-script base_digest={} write_table_digest={}",
        hexadecimal(&base_digest),
        hexadecimal(&write_table_digest)
    );
}
