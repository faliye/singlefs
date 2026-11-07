//! 崩溃放量共用的流程步骤。几个测试文件调这里的同一个函数，就得到同一个身份、同一串路径段：前缀节点在库里只核一次，
//! 别的流直接复用（前缀共享是剪枝的基础，没有它每次放量都从头核）。
//! 崩溃点的文件是这一份（`file!()`），不是调它的测试文件：改调用方的测试文件不动这里的节点。GPU 结论的版本与设点的文件无关；
//! CPU 结论的版本是 CPU 判器闭包并上设点文件出发的闭包（J8），设在这里的点几条流同一把。
//! 多记录流的 B、C 两次顺序写也在这里设点（J3：删除流与多记录流共享到 C）；不录崩溃点的崩溃点重放那一路调同一个 [`sequential_write`]，
//! 两份造法只有这一份写法。
//! 挂在标准前缀节点上的观察者登记在这里（J4，[`OBSERVERS_ON_THE_STANDARD_PREFIX`]，只有名字）；观察者的代码与照表跑的放量入口在
//! `tests/common_crash_point_observers/mod.rs`：这一份不引它，公共模块设的点的 CPU 判器版本因此不收观察者代码（H4「观察者代码不进
//! 计划哈希、不进判器版本」）。删除流的录法（意图、各批）也在这里：删除流设点与观察者另读的输入照录都用它。
#![allow(
    dead_code,
    reason = "共用的步骤模块：每个测试文件只调其中几步，没调的在那个测试二进制里就是没用到"
)]

use super::common::{parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::CheckpointTxg;
use singlefs_core::address::{InodeNumber, InstanceGeneration};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, RaisedFloor,
    RollbackTarget, ShadowLedger, UnmountRaisedTheFloor, Unmounted,
};
use singlefs_core::mount::{MountOutput, Mounted};
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::transaction::publish_overwrite;
use singlefs_core::transaction::{
    publish_sequential_write, publish_unlink_intent, publish_unlink_step, FirstFile, PoolVersion,
    PoolWriter, PublishError, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_verification::crash_amplification::CrashPointRecorder;
use singlefs_verification::scenario::deletion_history_batch_size;
use singlefs_verification::RecordedPublishEntry;
use std::sync::Arc;

/// 这一份的仓内路径，与 `file!()` 归一之后相同；测试拿它核身份里的文件。
pub const THIS_FILE: &str = "crates/singlefs-verification/tests/common_crash_points/mod.rs";

/// 恰好要 `data_units` 个数据单元的内容：最后一个单元装一半（多记录流与删除流的 B、C 两次顺序写都取这一份字节）。
pub fn content_needing(data_units: usize, seed: usize) -> Vec<u8> {
    let payload_capacity = data_unit_payload_capacity();
    (0..(data_units - 1) * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 11 + seed * 3 + 7) % 251).expect("小于 256"))
        .collect()
}

/// 顺序写（多记录流与删除流的 B、C）：把现行那一版的内容换成 `content`，实例 1、调用方时间比第一个文件晚 60 秒。
/// 崩溃点重放那一路直接调它，录崩溃点的那一路经 [`sequential_write_step`] 调它：两份造法同一份写法（J3，「两份造法写表摘要相等」的用例核它）。
pub fn sequential_write(pool: &mut BuiltPool, content: &[u8]) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_sequential_write(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        InstanceGeneration(1),
    )
    .expect("顺序写");
    pool.output = output.clone();
    output
}

/// [`sequential_write`] 设一个崩溃点（设点文件是这一份）：多记录流的 B、C 与删除流的 B、C 是同一个身份、同一个节点，共享到 C。
pub fn sequential_write_step(
    recorder: &mut CrashPointRecorder,
    pool: &mut BuiltPool,
    content: &[u8],
) -> TransactionOutput {
    recorder.step(file!(), "sequential_write", || {
        sequential_write(pool, content)
    })
}

/// 多记录流到 C 为止的三次带文件的发布（A 在 `build_pool` 里）：每一版的输出与 B、C 的内容。
pub struct MultiRecordPublishesRecorded {
    pub first: TransactionOutput,
    pub second: TransactionOutput,
    pub second_content: Vec<u8>,
    pub third: TransactionOutput,
    pub third_content: Vec<u8>,
}

impl MultiRecordPublishesRecorded {
    /// A、B、C 三版带文件的版本（oracle 认的那张表，按发布的次序）。
    pub fn versions(&self) -> Vec<singlefs_verification::memory_pool::PublishedVersion> {
        let version = |output: &TransactionOutput, content: Vec<u8>| {
            singlefs_verification::memory_pool::PublishedVersion {
                instance: output.root.instance,
                checkpoint_txg: output.root.checkpoint_txg,
                content,
            }
        };
        vec![
            version(&self.first, super::common::file_content()),
            version(&self.second, self.second_content.clone()),
            version(&self.third, self.third_content.clone()),
        ]
    }
}

/// 多记录流到 C 为止（录崩溃点）：取号加暖机、新池新建文件两个点，B（两个数据单元、两条记录）、C（三个数据单元、三条记录）
/// 两次顺序写各一个点，全设在这一份里：多记录流与接在它后面的删除流共享到 C（J3）。
pub fn record_the_multi_record_publishes(
    recorder: &mut CrashPointRecorder,
    pool: &mut BuiltPool,
) -> MultiRecordPublishesRecorded {
    record_pool_build(recorder, pool);
    let first = pool.output.clone();
    let second_content = content_needing(2, 1);
    let second = sequential_write_step(recorder, pool, &second_content);
    let third_content = content_needing(3, 2);
    let third = sequential_write_step(recorder, pool, &third_content);
    MultiRecordPublishesRecorded {
        first,
        second,
        second_content,
        third,
        third_content,
    }
}

// ============================== 删除流的录法（多记录流到 C 之后 unlink 第一个文件） ==============================
// 删除流自己的崩溃枚举用例与挂在标准前缀上的可写挂载续做观察者都要它：观察者另读的输入从录一遍删除流得来，单跑多记录流那一项也照录
// （J4、M4 第 10 条「单跑照录别项流程」）。

/// unlink 的是第一个文件。
pub const UNLINKED_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);

/// 意图那次的调用方时间比顺序写晚：与那条记录现有的时间不同，意图写的是不是调用方给的那一个分辨得出。
pub const SECONDS_FROM_THE_SEQUENTIAL_WRITES_TO_THE_INTENT: u64 = 120;

/// 删除段的批数（不含最后一批）：三个数据单元按批大小 1 走「留 2、留 1、最后一批」（K-U6 翻面观测：删除段少于两批不算数）。
pub const DELETION_BATCHES_BEFORE_THE_FINAL_BATCH: usize = 2;

/// unlink 第一个文件的意图那次发布（实例 1）。
pub fn publish_the_unlink_intent(pool: &mut BuiltPool) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        UNLINKED_FILE,
        FIXED_WRITE_TIME_SECONDS + SECONDS_FROM_THE_SEQUENTIAL_WRITES_TO_THE_INTENT,
        InstanceGeneration(1),
    )
    .expect("unlink 意图");
    pool.output = output.clone();
    output
}

/// 删除的下一批（批大小照只供测试的 `deletion_history_batch_size`，实例 1）。
pub fn publish_the_next_unlink_step(pool: &mut BuiltPool) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_unlink_step(
        &mut writer,
        &mut pool.allocator,
        &previous,
        UNLINKED_FILE,
        deletion_history_batch_size(),
        InstanceGeneration(1),
    )
    .expect("unlink 的一批");
    pool.output = output.clone();
    output
}

// ============================== 固定脚本到 E 之后经会话 unlink 第一个文件（里程碑三三·4） ==============================
// 固定脚本（取号 → 暖机 → A → B → 重开可写挂载 → C → 回退 D → 五次覆盖写 → 抬 F → E）那几步照旧由上面那几个 `*_step` 设点，
// 这一步接在 E 后面、设点也在这一份：节点 `publish_unlink_through_the_session.<周期>.<步>` 归这一份文件。

/// 经会话 unlink 第一个文件那一次调用发成的每一版（意图、各批、最后一批，按先后），与发完那几版之后会话交回的发布路径的拒绝（没有是 `None`）。
pub struct UnlinkThroughTheSession {
    pub versions_published: Vec<TransactionOutput>,
    pub refusal_after_the_versions_published: Option<PublishError>,
}

/// 经会话入口 unlink 第一个文件（`UserChange::Unlink`：先把现行那一版的队列走空、发意图、再把队列走空，D8（核心索引结构） 已定项 16 第 6 条）：
/// 会话接在 `mount_output` 那一次可写挂载后面（固定脚本 B 之后重开的那一次，同一个实例），现行那一版是池上现在那一版（固定脚本的 E），
/// 分配器是池上现在那一份（带着那次挂载装上的删除开关，产品批大小）。发完把池的现行版本与分配器换成会话的。
/// 发布路径的拒绝（`UserChangeRefused::Publish`）不在这里断言：被拒之前发成的那几版已落盘、交回在这里，接着照录。改坏「最后一批与删意图同事务」
/// （D8（核心索引结构） 已定项 5）时最后一批之后号还留在队列里，走空再发一步被拒，录下来的流红在判器上（池级 checker I-9.5），不红在搭建上。
///
/// # Panics
/// 会话交回发布路径之外的拒绝（盘表、可见、本盘设备号、落后、空间不够），或发完之后会话的现行那一版没有树。
pub fn unlink_the_first_file_through_the_session(
    pool: &mut BuiltPool,
    mount_output: MountOutput,
) -> UnlinkThroughTheSession {
    let (_mount_output, mut session) = MountedSession::of_the_mount(Mounted {
        output: mount_output,
        allocator: pool.allocator.clone(),
        current: PoolVersion::WithFile(Arc::new(pool.output.clone())),
    });
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let unlinked = session.publish_user_change(
        devices,
        UserChange::Unlink {
            inode: UNLINKED_FILE,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS
                + SECONDS_FROM_THE_SEQUENTIAL_WRITES_TO_THE_INTENT,
        },
    );
    let through_the_session = match unlinked {
        Ok(published) => UnlinkThroughTheSession {
            versions_published: published
                .versions_published
                .into_iter()
                .map(Arc::unwrap_or_clone)
                .collect(),
            refusal_after_the_versions_published: None,
        },
        Err(UserChangeRefused::Publish {
            cause,
            versions_published,
            ..
        }) => UnlinkThroughTheSession {
            versions_published: versions_published
                .into_iter()
                .map(Arc::unwrap_or_clone)
                .collect(),
            refusal_after_the_versions_published: Some(cause),
        },
        Err(refusal) => panic!("会话 unlink 第一个文件被拒在发布路径之外：{refusal:?}"),
    };
    pool.allocator = session.allocator;
    pool.output = match session.current {
        PoolVersion::WithFile(current) | PoolVersion::WithTreesAndNoFile(current) => {
            Arc::unwrap_or_clone(current)
        }
        PoolVersion::WithoutFile(_) => {
            panic!("会话 unlink 之后现行那一版有树（意图那一版起每一版都有树），实际树表 0 条")
        }
    };
    through_the_session
}

/// [`unlink_the_first_file_through_the_session`] 设一个崩溃点（设点文件是这一份），接在固定脚本 E 那一步后面。
pub fn unlink_the_first_file_through_the_session_step(
    recorder: &mut CrashPointRecorder,
    pool: &mut BuiltPool,
    mount_output: MountOutput,
) -> UnlinkThroughTheSession {
    recorder.step(file!(), "publish_unlink_through_the_session", || {
        unlink_the_first_file_through_the_session(pool, mount_output)
    })
}

// ============================== 挂在标准前缀上的观察者：登记表（J4） ==============================

/// 挂在标准前缀节点上的观察者（封闭的一张：加一种就加一个成员，`common_crash_point_observers::amplify_a_flow_through_the_standard_prefix`
/// 里的 `match` 跟着补）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverOfTheStandardPrefix {
    /// 每个状态上可写挂载一次、续做走到空、挂完的盘过池级 checker（`common_crash_point_observers::observe_the_mount_continuation`；
    /// 可写挂载挂在标准前缀上，
    /// 共享前缀的每个状态都做，不得例外）。
    WritableMountAndContinuation,
}

impl ObserverOfTheStandardPrefix {
    /// 观察者的名字：进它那一组判法单元的组名、计数与判红那一项（不进结论种类：`ConclusionKind::Observer` 不带名字，是哪一个观察者由版本担）。
    pub fn name(self) -> &'static str {
        match self {
            Self::WritableMountAndContinuation => "writable_mount_and_continuation",
        }
    }
}

/// 登记表（H4）：标准前缀的节点（这份文件里设点的入口名：[`record_the_multi_record_publishes`] 设的取号加暖机、新池新建文件、
/// B、C 两次顺序写）→ 挂的观察者。单跑一项照表跑它经过的每个节点上登记的全部观察者；共享节点上的观察者对经过它的每一项、每一个状态都跑
/// （块键相同的块在库里复用）。观察者代码不进计划哈希、不进判器版本，它的身份是它的入口 fn 的闭包键（`common_crash_point_observers::observe_the_mount_continuation`，登记在 `singlefs_verification::judging_units_entries::OBSERVER_ENTRIES`）。
pub static OBSERVERS_ON_THE_STANDARD_PREFIX: &[(&str, &[ObserverOfTheStandardPrefix])] = &[
    (
        "acquire_instance_and_warm_up",
        &[ObserverOfTheStandardPrefix::WritableMountAndContinuation],
    ),
    (
        "publish_first_file",
        &[ObserverOfTheStandardPrefix::WritableMountAndContinuation],
    ),
    (
        "sequential_write",
        &[ObserverOfTheStandardPrefix::WritableMountAndContinuation],
    ),
];

// ============================== 标准前缀之后的几步：快照、整理、生命周期闭环（里程碑四一·4、二·3、三·1） ==============================
// 几条快照流、两条整理流与生命周期闭环流在 A 之后各接几步，每一步的入口名照下面这几个（快照那几步的入口名住在验证包的
// `common_snapshot_prefix::SnapshotFlowStepKind::entry_name`，整理与闭环那几步住在 `common_compaction_prefix::CompactionFlowStepKind::entry_name`）；
// 设点都在这一份（[`crash_points_of_a_flow_after_the_first_file`]），几条流里同一个入口录出逐字节相同的那一段就是同一个节点、共享判定。

/// 挂在快照那几步节点上的观察者（封闭的一张：加一种就加一个成员，`common_snapshot_crash_point_observers::amplify_a_snapshot_flow` 里的
/// `match` 跟着补）。观察者的代码在 `tests/common_snapshot_crash_point_observers/mod.rs`，这一份只登记名字（同标准前缀那一张的口径：观察者代码
/// 不进这份设点文件出发的 CPU 判器版本）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverOfTheSnapshotSteps {
    /// 每个状态上读恢复落到的那一版的每个活快照，读出的要是建它那一刻的内容（只记计数）。
    SnapshotOfTheLandedVersion,
    /// 每个状态上可写挂载一次、续做把 unlink 与删快照走到空，挂完之后快照这几样与录流时同一串删除的最后一批那一版逐项相同；挂完的盘过池级 checker。
    MountContinuationUnderASnapshot,
}

impl ObserverOfTheSnapshotSteps {
    /// 观察者的名字：进它的结论种类（`ConclusionKind::Observer`）、计数与判红那一项；崩溃点重放那一路的同一个观察者也用这个名字。
    pub fn name(self) -> &'static str {
        match self {
            Self::SnapshotOfTheLandedVersion => "snapshot_of_the_landed_version",
            Self::MountContinuationUnderASnapshot => "mount_continuation_under_a_snapshot",
        }
    }
}

/// 快照那几步的节点 → 挂的观察者（入口名与 `common_snapshot_prefix::SnapshotFlowStepKind::entry_name` 逐个相同，
/// `common_snapshot_crash_point_observers::amplify_a_snapshot_flow` 开跑前核）。几条快照流经过哪几个就照表跑哪几个上的观察者。
pub static OBSERVERS_ON_THE_SNAPSHOT_STEPS: &[(&str, &[ObserverOfTheSnapshotSteps])] = &[
    (
        "publish_create_snapshot",
        &[
            ObserverOfTheSnapshotSteps::SnapshotOfTheLandedVersion,
            ObserverOfTheSnapshotSteps::MountContinuationUnderASnapshot,
        ],
    ),
    (
        "publish_overwrite_under_the_snapshot",
        &[
            ObserverOfTheSnapshotSteps::SnapshotOfTheLandedVersion,
            ObserverOfTheSnapshotSteps::MountContinuationUnderASnapshot,
        ],
    ),
    (
        "publish_unlink_intent_under_the_snapshot",
        &[
            ObserverOfTheSnapshotSteps::SnapshotOfTheLandedVersion,
            ObserverOfTheSnapshotSteps::MountContinuationUnderASnapshot,
        ],
    ),
    (
        "publish_unlink_step_under_the_snapshot",
        &[
            ObserverOfTheSnapshotSteps::SnapshotOfTheLandedVersion,
            ObserverOfTheSnapshotSteps::MountContinuationUnderASnapshot,
        ],
    ),
    (
        "publish_snapshot_destruction_intent",
        &[
            ObserverOfTheSnapshotSteps::SnapshotOfTheLandedVersion,
            ObserverOfTheSnapshotSteps::MountContinuationUnderASnapshot,
        ],
    ),
    (
        "publish_snapshot_destruction_step",
        &[
            ObserverOfTheSnapshotSteps::SnapshotOfTheLandedVersion,
            ObserverOfTheSnapshotSteps::MountContinuationUnderASnapshot,
        ],
    ),
];

/// A 之后各步的崩溃点（节点按 `<流程>.<周期>.<步>` 编码，`crash_amplification::CrashPointRecorder`）：取号加暖机、新池新建文件两段照标准前缀设，
/// 之后每一步按 `entries_after_the_first_file`（入口名、它发出的那段录制流，首尾相接、接在 A 之后、罩到流尾）设，设点文件都是这一份——
/// 快照那几步、整理那几步、生命周期闭环那几步的节点在几条流之间共享（里程碑四一·4 交判第 2 条）。流已经录完，按每一步记下的区间补记。
pub fn crash_points_of_a_flow_after_the_first_file(
    pool: &BuiltPool,
    entries_after_the_first_file: &[(&str, std::ops::Range<usize>)],
    writes: &[singlefs_verification::memory_pool::RetainedWrite],
    segments: &[Vec<usize>],
    stream_indexes: &[usize],
) -> Vec<singlefs_verification::crash_amplification::CrashPointSpan> {
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    let start_of_the_steps_after_the_first_file = entries_after_the_first_file.first().map_or(
        pool.stream.operation_count(),
        |(_entry_name, operations)| operations.start,
    );
    recorder.record_span(
        file!(),
        "acquire_instance_and_warm_up",
        pool.mkfs_operation_count..pool.warm_up_operation_count,
    );
    recorder.record_span(
        file!(),
        "publish_first_file",
        pool.warm_up_operation_count..start_of_the_steps_after_the_first_file,
    );
    for (entry_name, operations) in entries_after_the_first_file {
        recorder.record_span(file!(), entry_name, operations.clone());
    }
    recorder.into_crash_points(writes, segments, stream_indexes)
}

/// `build_pool` 已经发出去的两段：取号加暖机、新池新建文件。
pub fn record_pool_build(recorder: &mut CrashPointRecorder, pool: &BuiltPool) {
    recorder.record_span(
        file!(),
        "acquire_instance_and_warm_up",
        pool.mkfs_operation_count..pool.warm_up_operation_count,
    );
    recorder.record_span(
        file!(),
        "publish_first_file",
        pool.warm_up_operation_count..pool.stream.operation_count(),
    );
}

/// 覆盖写：把现行那一版的内容换成 `content`。
pub fn overwrite_step(
    recorder: &mut CrashPointRecorder,
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    recorder.step(file!(), "publish_overwrite", || {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &pool.output,
            FirstFile {
                content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            instance,
        )
        .expect("覆盖写")
    })
}

/// 关掉镜像再可写挂载（重开：取号、写行、暖机）。
pub fn mount_writable_step(recorder: &mut CrashPointRecorder, pool: &mut BuiltPool) -> Mounted {
    recorder.step(file!(), "mount_writable", || {
        let mut devices = pool.reopen_recorded();
        let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
        pool.devices = Some(devices);
        mounted
    })
}

/// 挂着的时候用一次向前发布回退到 `target`。
pub fn roll_back_step(
    recorder: &mut CrashPointRecorder,
    pool: &mut BuiltPool,
    target: RollbackTarget,
) {
    recorder.step(file!(), "roll_back_by_a_forward_publish", || {
        let rollback_parameters = parameters();
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        roll_back_by_a_forward_publish(
            &rollback_parameters,
            open_devices,
            &mut pool.allocator,
            &mut pool.output,
            target,
        )
        .expect("挂着的时候回退");
    });
}

/// 抬回退下界 F 到 `new_floor`：先把新 F 写进每块盘的系统配置，再推带新 F 的空发布直到每块盘上都有一条（D16 已定项 1「抬 F 那一串」）。
pub fn raise_rollback_floor_step(
    recorder: &mut CrashPointRecorder,
    pool: &mut BuiltPool,
    new_floor: CheckpointTxg,
) -> RaisedFloor {
    recorder.step(file!(), "raise_rollback_floor", || {
        let mut current = pool.output.clone();
        let raise_devices = pool.devices.as_mut().expect("镜像还开着");
        let raised = raise_rollback_floor(
            &parameters(),
            raise_devices,
            &mut pool.allocator,
            &mut current,
            new_floor,
            ShadowLedger::On,
        )
        .expect("抬 F");
        pool.output = current;
        raised
    })
}

/// 正常卸载（影子账开着），录制流上把这一段记成卸载入口发的（C557：带卸载记号的根槽写只许落在卸载入口里）：
/// 带文件的一版上卸载必抬 F，交回那一串；现行版本换成那一串最后落盘的那一次。
pub fn unmount_step(
    recorder: &mut CrashPointRecorder,
    pool: &mut BuiltPool,
) -> UnmountRaisedTheFloor {
    recorder.step(file!(), "unmount", || {
        let mut current = PoolVersion::WithFile(Arc::new(pool.output.clone()));
        let stream = pool.stream.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let allocator = &mut pool.allocator;
        let unmounted = stream
            .record_entry(RecordedPublishEntry::Unmount, || {
                unmount(
                    &parameters(),
                    devices,
                    allocator,
                    &mut current,
                    ShadowLedger::On,
                )
            })
            .expect("正常卸载");
        pool.output = current
            .into_file_version()
            .expect("带文件的一版卸载之后仍带文件");
        match unmounted {
            Unmounted::FloorRaisedToTheCurrentVersion(raised) => raised,
            Unmounted::NothingWrittenOnAVersionWithoutFile {
                current: reported_version,
            } => {
                panic!("带文件的一版上卸载报成「树表 0 条、一个字节都不写」：{reported_version:?}")
            }
        }
    })
}

/// 经 `run_from_environment` 跑的用例的环境守卫（K5）：与 `crash_amplification` 里改环境的单测同一份，住在库里，这里转出去给各个测试文件用。
#[allow(
    unused_imports,
    reason = "共用的步骤模块：只有改环境、经 run_from_environment 跑的测试文件用这两个守卫，别的测试二进制里转出去没人用"
)]
pub use singlefs_verification::crash_amplification::{
    CrashAmplificationEnvironment, EnvironmentRestoredOnDrop,
};

/// 调用方的显卡先后写成环境变量的值（经 `run_from_environment` 跑的这一路与给定状态引擎那几条用例同一份写法）。
#[path = "../common_gpu_cards_of_the_caller/mod.rs"]
pub mod common_gpu_cards_of_the_caller;

/// 清过场（服务给了卡）、用例自己写的多机配置（`SINGLEFS_MULTI_HOST_CONFIG` 指的那一份）开了 GPU，环境变量 `GPU_CARDS_BY_PRIORITY`
/// 却没设（用例的环境守卫清掉了）时：照调用方给这个进程的显卡先后（`gpu_card_configuration::GpuConfigurationOfTheCaller`，服务交的那一张）
/// 设上这个环境变量（一张卡都没点名是空串：本机能用的卡全用），交回的守卫丢掉时还原；别的情形不动环境、交 `None`。
/// 用例配置里写没写显卡先后都一样：配置里的卡不拿来挑卡，没经服务给卡、配置又开了 GPU 的，清没清场都照「清场与分卡由服务做」拒
/// （`cards_by_priority_from`）。没清场的不设：要卡的那一趟没经服务给卡就该拒，没要卡的那一趟（不带 gpu 特性）不取卡。
/// 经环境变量交过去的就是调用方的那一份，与卡池挑的卡对得上。
///
/// # Panics
/// 调用方的多机配置读不了、显卡先后写坏了，或清场之下没经服务给卡而调用方的多机配置开了 GPU（GPU 用例整条经 `task-server/task submit` 交）。
fn cards_of_the_caller_for_a_test_configuration_with_the_gpu() -> Option<EnvironmentRestoredOnDrop>
{
    use singlefs_verification::gpu_card_configuration::{
        configuration_value, GpuQuiesce, CARDS_BY_PRIORITY_KEY, GPU_SWITCH_KEY,
        MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
    };
    if std::env::var_os(CARDS_BY_PRIORITY_KEY).is_some() {
        return None;
    }
    let test_configuration = std::path::PathBuf::from(std::env::var_os(
        MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
    )?);
    let gpu_is_enabled = configuration_value(&test_configuration, GPU_SWITCH_KEY)
        .is_some_and(|value| value.trim().trim_matches('"') == "1");
    match GpuQuiesce::of_this_process() {
        GpuQuiesce::QuiescedByTheWrapper => {}
        GpuQuiesce::NotQuiesced => return None,
    }
    if !gpu_is_enabled {
        return None;
    }
    let cards_of_the_caller = common_gpu_cards_of_the_caller::card_priority_value_of_the_caller();
    let mut restored = EnvironmentRestoredOnDrop::saved_then_cleared(&[CARDS_BY_PRIORITY_KEY]);
    restored.set_variable(
        CARDS_BY_PRIORITY_KEY,
        std::ffi::OsStr::new(&cards_of_the_caller),
    );
    Some(restored)
}

/// 一条已经录好的流（起点镜像、写表、段、被判的根槽写、版本表、崩溃点）按环境跑一趟崩溃放量：展开上限是这条流自己的那一个
/// （`expand_up_to_of_the_flow`：全量域的流传引擎常量 `EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION`，测机制的用例传自己的小上限），
/// 环境里设了缩域变量就再往小缩（[`expand_up_to_of_the_flow_narrowed_by_the_environment`]），库是 prune-server 里那一份
/// （这个进程设了一次性库就是那一份，`crash_library::DisposableLibrary`），判法摘要取两个判器的闭包；健康的流一个红状态都不该有。只排派活计划的那一趟（`placement_only`）不判，原样交回。
/// 打一行 `CRASH_AMPLIFICATION flow=<名> …`，交回这一趟的结果。
#[allow(
    clippy::too_many_arguments,
    reason = "一条流的七样各自独立，收成一个结构体只是把七个字段挪个地方"
)]
pub fn amplify_prepared_flow(
    flow_name: &str,
    base: &singlefs_verification::memory_pool::MemoryPool,
    writes: &[singlefs_verification::memory_pool::RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[singlefs_verification::memory_pool::PublishedVersion],
    crash_points: Vec<singlefs_verification::crash_amplification::CrashPointSpan>,
    expand_up_to_of_the_flow: usize,
) -> singlefs_verification::crash_amplification::EnvironmentRun {
    amplify_prepared_flow_and_hand_back_its_line(
        flow_name,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        crash_points,
        expand_up_to_of_the_flow,
    )
    .0
}

/// 一条流自己的展开上限，环境里设了缩域变量（`EXPAND_UP_TO_ENVIRONMENT_VARIABLE`，
/// `ExpandUpToWritesPerSegment::narrowing_from_the_environment`）就取两个里小的那个：只往小缩，不放宽。
///
/// # Panics
/// 缩域变量设了而认不出（同 `ExpandUpToWritesPerSegment::narrowing_from_the_environment`）。
#[must_use]
pub fn expand_up_to_of_the_flow_narrowed_by_the_environment(
    expand_up_to_of_the_flow: usize,
) -> singlefs_verification::crash_amplification::ExpandUpToWritesPerSegment {
    use singlefs_verification::crash_amplification::ExpandUpToWritesPerSegment;
    ExpandUpToWritesPerSegment(
        ExpandUpToWritesPerSegment::narrowing_from_the_environment()
            .map_or(expand_up_to_of_the_flow, |narrowed| {
                narrowed.0.min(expand_up_to_of_the_flow)
            }),
    )
}

/// 同 [`amplify_prepared_flow`]，版本表另带文件从那一版起没有了的那几版（删除流的意图那一版，`CrashFlow::versions_without_a_file`）。
#[allow(
    clippy::too_many_arguments,
    reason = "一条流的八样各自独立，收成一个结构体只是把八个字段挪个地方"
)]
pub fn amplify_prepared_flow_against_the_version_table(
    flow_name: &str,
    base: &singlefs_verification::memory_pool::MemoryPool,
    writes: &[singlefs_verification::memory_pool::RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[singlefs_verification::memory_pool::PublishedVersion],
    versions_without_a_file: &[singlefs_verification::memory_pool::PublishedVersionWithoutAFile],
    crash_points: Vec<singlefs_verification::crash_amplification::CrashPointSpan>,
    expand_up_to_of_the_flow: usize,
) -> singlefs_verification::crash_amplification::EnvironmentRun {
    amplify_prepared_flow_against_the_version_table_and_hand_back_its_line(
        flow_name,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        versions_without_a_file,
        crash_points,
        expand_up_to_of_the_flow,
        &[],
    )
    .0
}

/// 同 [`amplify_prepared_flow`]，另交回打出去的那一行 `CRASH_AMPLIFICATION flow=<名> …`（只排派活计划的那一趟不打，是 `None`）。
/// 行末接 `reuse_and_scale_fields_of_the_line` 的几个字段（复用数、全域与否、展开上限、ignore 的跳过数），被叫停的那一行也接。
#[allow(
    clippy::too_many_arguments,
    reason = "一条流的七样各自独立，收成一个结构体只是把七个字段挪个地方"
)]
pub fn amplify_prepared_flow_and_hand_back_its_line(
    flow_name: &str,
    base: &singlefs_verification::memory_pool::MemoryPool,
    writes: &[singlefs_verification::memory_pool::RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[singlefs_verification::memory_pool::PublishedVersion],
    crash_points: Vec<singlefs_verification::crash_amplification::CrashPointSpan>,
    expand_up_to_of_the_flow: usize,
) -> (
    singlefs_verification::crash_amplification::EnvironmentRun,
    Option<String>,
) {
    amplify_prepared_flow_against_the_version_table_and_hand_back_its_line(
        flow_name,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        &[],
        crash_points,
        expand_up_to_of_the_flow,
        &[],
    )
}

/// 同 [`amplify_prepared_flow_and_hand_back_its_line`]，版本表另带「没有文件的那几版」，连同挂在节点上的观察者（没挂是空的；
/// 照登记表挂的那一份由 `common_crash_point_observers::amplify_a_flow_through_the_standard_prefix` 交进来）。
#[allow(
    clippy::too_many_arguments,
    reason = "一条流的八样各自独立，另加挂在节点上的观察者；收成一个结构体只是把字段挪个地方"
)]
pub fn amplify_prepared_flow_against_the_version_table_and_hand_back_its_line(
    flow_name: &str,
    base: &singlefs_verification::memory_pool::MemoryPool,
    writes: &[singlefs_verification::memory_pool::RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[singlefs_verification::memory_pool::PublishedVersion],
    versions_without_a_file: &[singlefs_verification::memory_pool::PublishedVersionWithoutAFile],
    crash_points: Vec<singlefs_verification::crash_amplification::CrashPointSpan>,
    expand_up_to_of_the_flow: usize,
    observers: &[singlefs_verification::crash_amplification::ObserverOnTheNodes<'_>],
) -> (
    singlefs_verification::crash_amplification::EnvironmentRun,
    Option<String>,
) {
    use singlefs_verification::crash_amplification::{
        reuse_and_scale_fields_of_the_line, run_from_environment_with_observers, CrashFlow,
    };
    use singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
    let limit = expand_up_to_of_the_flow_narrowed_by_the_environment(expand_up_to_of_the_flow);
    let expansion = move |_segment_index: usize, segment: &[usize]| limit.expansion_of(segment);
    let flow = CrashFlow {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        versions_without_a_file,
        expansion: &expansion,
        crash_points,
    };
    let card_priority_of_the_caller = cards_of_the_caller_for_a_test_configuration_with_the_gpu();
    let started = std::time::Instant::now();
    let run = run_from_environment_with_observers(
        &flow,
        flow_name,
        observers,
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        limit,
    )
    .expect("按环境跑得完");
    let seconds = started.elapsed().as_secs_f64();
    drop(card_priority_of_the_caller);
    if run.placement_only || run.recorded_only {
        return (run, None);
    }
    let reuse_and_scale = reuse_and_scale_fields_of_the_line(&run);
    if run.stopped_early {
        // 被叫停的那一趟：判过的块在库里、第 ②③ 段没做；只报判了多少，不断言对账
        let line = format!(
            "CRASH_AMPLIFICATION flow={flow_name} mode={} states_checked_here={} blocks_checked_here={} unjudged_blocks={} red={} stopped_early=true{reuse_and_scale}",
            run.mode.name(),
            run.checking.states_checked,
            run.checking.blocks_checked,
            run.unjudged_blocks,
            run.checking.red_states
        );
        println!("{line}");
        assert_eq!(
            run.checking.red_states, 0,
            "健康的流 {flow_name} 一个红状态都不该有：{:?}",
            run.checking.red_texts
        );
        return (run, Some(line));
    }
    let judge_line = judge_fields_of_the_summary_line(&run);
    let judging_units_fingerprint = run.judged_conclusions().version_text();
    let line = format!(
        "CRASH_AMPLIFICATION flow={flow_name} mode={} states_checked_here={} blocks_checked_here={} unjudged_blocks={} red={} violations_in_the_library={} judging_units_fingerprint={judging_units_fingerprint} seconds={seconds:.1}{reuse_and_scale}{judge_line}",
        run.mode.name(),
        run.checking.states_checked,
        run.checking.blocks_checked,
        run.unjudged_blocks,
        run.checking.red_states,
        run.violations.len()
    );
    println!("{line}");
    assert_eq!(
        run.checking.red_states, 0,
        "健康的流 {flow_name} 一个红状态都不该有：{:?}",
        run.checking.red_texts
    );
    // 这一趟复用的块（库里这一判法单元指纹早先判过的）判红过也要红：只看这一趟判出的红数，红过一次的流再跑一遍就成了绿
    if run.unjudged_blocks == 0 {
        assert!(
            run.violations.is_empty(),
            "健康的流 {flow_name}：库里这条流的块有 {} 个红状态（含早先判过、这一趟复用的），前几个：{:?}",
            run.violations.len(),
            run.violations.iter().take(3).collect::<Vec<_>>()
        );
    }
    if let Some(comparison) = &run.comparison {
        assert_eq!(
            comparison.disagreements, 0,
            "流 {flow_name}：核对红而判器绿：{:?}",
            comparison.disagreement_samples
        );
    }
    // 计数向量每一趟都判（J7）：这一趟判的与库里复用的块，读回来的计数向量与计划对得上
    assert!(
        run.count_vector_disagreements.is_empty(),
        "流 {flow_name}：计数向量与计划对不上：{:?}",
        run.count_vector_disagreements
    );
    (run, Some(line))
}

/// 汇总行里说第 ① 段在哪判的那几个字段：GPU 判的带用了几张卡、判了多少、多快、写库花了多久；CPU 判的只有 `judge=cpu`。
#[must_use]
pub fn judge_fields_of_the_summary_line(
    run: &singlefs_verification::crash_amplification::EnvironmentRun,
) -> String {
    match &run.gpu_judge {
        Some(gpu) => format!(
            " judge=gpu gpu_cards={} gpu_states={} gpu_compile_seconds={:.1} gpu_judge_seconds={:.1} gpu_states_per_second={:.0} gpu_seconds_storing={:.1}",
            gpu.cards_used,
            gpu.states_judged,
            gpu.compile_seconds,
            gpu.judge_seconds,
            gpu.states_per_second(),
            gpu.seconds_storing
        ),
        None => " judge=cpu".to_string(),
    }
}

/// 一次录下的整条写表当一个崩溃点（起点镜像不枚举、只录一次发布那几条流）：名字是这条流的名字，设点的文件给 `file!()`。
pub fn one_crash_point_over_every_write(
    code_file: &str,
    name: &str,
    write_count: usize,
) -> Vec<singlefs_verification::crash_amplification::CrashPointSpan> {
    vec![singlefs_verification::crash_amplification::CrashPointSpan {
        name: name.to_string(),
        code_file_in_repository:
            singlefs_verification::crash_amplification::repository_relative_file(code_file),
        writes: 0..write_count,
        ignored: false,
    }]
}
