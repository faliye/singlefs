//! checker 档模块：crash、crash_state_enumeration、writable_mount_phases、crash_amplification
//! 近满盘上截断成 0 的流（里程碑三三·4 第二条：安全设计轮第三轮判决 `research/prompts/m2-safety-r3-main-verification.md` 第二节「崩在删除类发布中间」
//! 那一格没跑，用户 2026-10-18 定新写这条流、登记放量分项、跑全域）。
//!
//! 池：单元区 384 槽那一档小盘（`tests/common_admission/mod.rs` 的准入用例与崩在抬 F 那一串中间的流同一档，两块内存稀疏盘），mkfs、取号、暖机、
//! 第一个文件之后崩了再挂（实例 2），再经会话把第一个文件顺序写到贴边：写得进的最多数据单元数 N（N + 1 个经会话推满抬 F 仍被拒，
//! `UserChangeRefused::NoSpaceAfterRaisingTheFloor`；N 现扫，不写死，写这条流时是 113）。基镜像是贴边那一版落盘之后的盘面，录的是之后把第一个文件
//! 截成 0 的三次删除类发布（判决 `research/prompts/m4-trunc-r1-main-verification.md` K2：意图 → 一批 → 最后一批；批大小取 ⌈(N − 1) ÷ 2⌉ 个数据单元，
//! 意图之后在 size 之外挂着的 N − 1 个单元恰好分两批放完）。删除类发布不判准入式子、照判落点（`transaction.rs` 里 `demand_is_judged` 那一判，
//! D28（挂载期承诺量） 已定项 1），这条流就是看贴边的池上这三次发布的每个崩溃状态。
//!
//! 每个状态：恢复两遍、多版本 oracle（贴边那一版及之前每一版有文件；意图起的三版文件长 0）、池级 checker、记录核对器；另挂一个交回镜像的观察者，
//! 在这个状态上可写挂载一次（空间准入判着、删除开关的批大小同录流），共用走空把截断走完：续做发的次数等于恢复落到的那一版之后录流还剩的截断发布数，
//! 挂完 logged_ops 里没有截断条目、第一个文件读出落到那一版的内容（意图那次已把 size 写成 0），挂完的盘交判段跑池级 checker。
//! 挂载做不成（准入在贴边的池上拒了挂载）当场 panic，记成红项。
//!
//! **比已有的流多罩了什么**：已有的三条截断流（`crash_enumeration_truncate_to_zero_stream.rs` 那一族）都在 4 GiB 的大盘上，准入从来不紧；
//! 崩在抬 F 那一串中间的流在同一档小盘上、只录抬 F。这一条是同一档小盘贴边之后的意图、一批、最后一批：意图那次的单元写（单元 0 重写成声明长度 0、
//! 下段节点、inode 叶、记账与分配记录）只能落在贴边之后剩下的空槽里，三次发布各自的单元段、记录段、根槽写与系统配置轮换在近满的池上第一次被枚举到。
//! **多跑的一步**：每个状态上可写挂载并续做截断（大盘那三条流只在崩溃放量那一路挂这个观察者，崩溃点重放那一路不挂）。基线镜像、写表、段序列都与已有流不同，
//! 各自全量枚举。
//!
//! 快档只展开写数小于 10 的段；全量标 ignored，登记成崩溃枚举用例。系统配置槽写是原地覆写，取三态，其余写取两态。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
// 池经 `common_admission_pool` 搭、不经 `common_admission`：后者引随机历史脚手架与故障注入，引了就进这条登记用例的判法闭包。
#[path = "../../singlefs-harness/tests/common_admission_pool/mod.rs"]
mod common_admission_pool;
#[cfg(feature = "verdict-store")]
mod common_crash_points;

use common_admission_pool::{
    device_bytes_of_the_384_slot_small_devices, fixed_geometry_of_the_384_slot_small_devices,
    parameters_of_the_384_slot_small_devices, plain_device, plain_devices_on, PoolUnderTest,
};
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_amplification::{
    ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver, ObserverDeduplication,
    ReadRangeOnTheCrashImage,
};
use singlefs_checker_tier::crash_point_replay_provenance::{
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow, RunOfTheFlow,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
    CrashPointReplayObserverHandingBackAnImage,
};
use singlefs_checker_tier::writable_mount_phases::materialized_crash_image;
use singlefs_core::address::{CheckpointTxg, FileOffsetInBytes, InodeNumber, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::logged_operations::live_truncation_intent_of_the_entries;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, DeletionQueueContinuationOutcome,
    ShadowLedger,
};
use singlefs_core::mounted_read::open_pool_for_read;
use singlefs_core::mounted_session::{MountedSession, UserChangeRefused};
use singlefs_core::recovery::RecoveryReport;
use singlefs_core::transaction::{
    publish_size_change, publish_truncation_step, PoolVersion, PoolWriter, PublishError,
    TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};
use singlefs_harness::memory_pool::{
    writes_and_segments, CrashImage, MemoryPool, PublishedVersion, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_truncate_to_zero_on_a_nearly_full_pool_stream.rs";

/// 崩溃点重放项名第一个 `::` 之前那一截（来历表里这条流的名字）。
const FLOW_NAME_AT_THE_ENUMERATION: &str = "truncate_to_zero_on_a_nearly_full_pool";

/// 崩溃放量的流名（来历表与放量入口 ITEMS 里这条流的名字）。
const FLOW_NAME_AT_THE_AMPLIFICATION: &str = "truncate-to-zero-on-a-nearly-full-pool";

/// 扫贴边那一格时顺序写的数据单元数上界：单元区每盘 1391 槽、一个数据单元占 2 槽，写到 696 个之前一定被拒；扫到这里还写得进就是没找到贴边那一格。
const DATA_UNITS_SWEPT_AT_MOST: u64 = 700;

/// 不判违例之外，在这条流的状态上至少真被评估过一次的不变量（阴性结果要与「代码没跑到」分开，C467（新补的四条不变量的覆盖率没被穷举用例钉住）
/// 那三条连同这条流自己要的截断条目有账那一条）。
const INVARIANTS_EVALUATED_ON_SOME_STATE: [&str; 4] = ["I-1.8", "I-7.3", "I-8.6", "I-4.10"];

/// 判段在可写挂载续做截断之后的盘上判池级 checker、一条不变量都不判违例时加一的计数。
const POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER: &str = "pool_checker_clean_after_the_mount";

/// 这条流贴边之后的三步（封闭集合：按计划定死，不按截断条目删了没有停——最后一批漏删条目时录流照样录完，红在判器与形状核对上）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TruncationStepAtTheEdge {
    /// 截断意图：一次发布写记录的 size = 0、签截断条目、单元 0 重写成声明长度 0。
    Intent,
    /// 不是最后一批的一批：从尾部放一批，截断条目留着。
    Batch,
    /// 最后一批：放完剩下的、同一次发布删截断条目。
    LastBatch,
}

/// 贴边之后按计划发的三步。
const STEPS_AT_THE_EDGE: [TruncationStepAtTheEdge; 3] = [
    TruncationStepAtTheEdge::Intent,
    TruncationStepAtTheEdge::Batch,
    TruncationStepAtTheEdge::LastBatch,
];

impl TruncationStepAtTheEdge {
    /// 恢复落到这一步那一版之后，可写挂载的续做还要发几次截断的发布。
    fn truncation_publishes_left_after(self) -> usize {
        match self {
            Self::Intent => 2,
            Self::Batch => 1,
            Self::LastBatch => 0,
        }
    }

    /// 接着 `current` 发这一步。
    fn publish(
        self,
        writer: &mut PoolWriter<'_, RecordingBlockDevice<SparseBlockDevice>>,
        allocator: &mut PoolAllocator,
        current: &TransactionOutput,
        instance: InstanceGeneration,
        batch_size: UnlinkBatchSize,
        write_time_seconds: u64,
    ) -> Result<TransactionOutput, PublishError> {
        match self {
            Self::Intent => publish_size_change(
                writer,
                allocator,
                current,
                InodeNumber(FIRST_INODE_NUMBER),
                0,
                write_time_seconds,
                instance,
            ),
            Self::Batch | Self::LastBatch => {
                publish_truncation_step(writer, allocator, current, batch_size, instance)
            }
        }
    }
}

/// 净荷容量（32634 字节）。
fn payload_capacity_in_bytes() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634 装得进 u64")
}

/// 顺序写成 `data_units` 个数据单元的内容长度。
fn content_length_of_data_units(data_units: u64) -> usize {
    usize::try_from(data_units * payload_capacity_in_bytes()).expect("几十 MiB 装得进 usize")
}

/// 同一个池拷一份：两块盘的镜像、会话（分配器、现行那一版）与每条根带的内容都拷，改拷贝不动原来那份。
fn copy_of_the_pool(pool: &PoolUnderTest<SparseBlockDevice>) -> PoolUnderTest<SparseBlockDevice> {
    let session = pool.session();
    PoolUnderTest {
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
        devices: plain_devices_on(&pool.image()),
        session: Some(MountedSession {
            allocator: session.allocator.clone(),
            current: session.current.clone(),
            instance: session.instance,
            shadow_ledger: session.shadow_ledger,
            parameters_and_device_table: session.parameters_and_device_table.clone(),
            compaction: session.compaction.clone(),
        }),
        write_time_seconds: pool.write_time_seconds,
        content_of_the_current_version: pool.content_of_the_current_version.clone(),
        content_of_each_root: pool.content_of_each_root.clone(),
    }
}

/// 第一个文件之后崩了再挂（实例 2）的那个池，`wrap` 给每块稀疏盘包上用例要的那一层。录制那一份与扫贴边那一份同一种起法，盘上逐字节相同。
fn pool_after_the_remount<Device: common_admission_pool::DeviceOverASparseImage>(
    wrap: impl Fn(singlefs_core::address::DeviceIdentity, SparseBlockDevice) -> Device,
) -> PoolUnderTest<Device> {
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        wrap,
    );
    pool.crash_and_mount_writable()
        .unwrap_or_else(|error| panic!("第一个文件之后崩了再挂：{error:?}"));
    pool
}

/// 顺序写 `data_units` 个数据单元写不写得进（在池的拷贝上写，原池不动）：写成是真；推满抬 F 仍被拒是假；别的拒绝 panic。
fn sequential_write_of_data_units_is_accepted(
    pool: &PoolUnderTest<SparseBlockDevice>,
    data_units: u64,
) -> bool {
    let mut copy = copy_of_the_pool(pool);
    match copy.sequential_write(content_length_of_data_units(data_units)) {
        Ok(_) => true,
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(_)) => false,
        Err(refusal) => {
            panic!("顺序写 {data_units} 个数据单元只该因为空间不够被拒，实际 {refusal:?}")
        }
    }
}

/// 贴边那一格：经会话顺序写得进的最多数据单元数 N（N 写得进、N + 1 推满抬 F 仍被拒）。二分在 [1, 上界] 上找，二分只当找法：
/// 找到的那一格由两次真写核过（N 写得进、N + 1 写不进），写得进与否不随单元数单调时这一格照样是一处贴边。
fn data_units_at_the_admission_edge() -> u64 {
    let pool = pool_after_the_remount(plain_device);
    let mut accepted = 1u64;
    let mut refused = DATA_UNITS_SWEPT_AT_MOST;
    assert!(
        sequential_write_of_data_units_is_accepted(&pool, accepted),
        "崩了再挂之后一个数据单元的顺序写写得进"
    );
    assert!(
        !sequential_write_of_data_units_is_accepted(&pool, refused),
        "{refused} 个数据单元的顺序写写不进（单元区每盘 1391 槽）：写得进就是没找到贴边那一格"
    );
    // 迭代上界：⌈log2(上界)⌉ 次；跨轮携带的不变量：accepted 写得进、refused 写不进、accepted < refused。
    while refused - accepted > 1 {
        let middle = accepted + (refused - accepted) / 2;
        if sequential_write_of_data_units_is_accepted(&pool, middle) {
            accepted = middle;
        } else {
            refused = middle;
        }
        assert!(accepted < refused, "二分的两端不交叉");
    }
    accepted
}

/// 批大小：意图之后 size 之外挂着 N − 1 个单元（单元 0 重写成声明长度 0 留着），⌈(N − 1) ÷ 2⌉ 一批恰好一批加最后一批放完。
fn deletion_switches_splitting_the_truncation_into_two_batches(
    data_units_at_the_edge: u64,
) -> DeletionSwitches {
    let data_units_outside_the_size = data_units_at_the_edge
        .checked_sub(1)
        .expect("贴边那一格至少一个数据单元");
    assert!(
        data_units_outside_the_size >= 2,
        "贴边那一格至少三个数据单元，截断才分得出一批与最后一批：{data_units_at_the_edge}"
    );
    let mut switches = DeletionSwitches::PRODUCT;
    switches.batch_size = UnlinkBatchSize::CappedForTests(
        UnlinkBatchDataUnitsCappedForTests::new(data_units_outside_the_size.div_ceil(2))
            .expect("批大小落在只供测试的 1..=144 里（贴边那一格不到 290 个数据单元）"),
    );
    switches
}

/// 录下来的那一段：贴边那一版落盘之后的盘面、截断三步的写表与段、最后一次根槽写、版本表、每一步的输出，与可写挂载观察者另读的输入。
struct RecordedTruncationAtTheEdge {
    data_units_at_the_edge: u64,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    steps: Vec<(TruncationStepAtTheEdge, TransactionOutput)>,
    mount_inputs: MountContinuingTheTruncationInputs,
}

/// 可写挂载续做截断那个观察者另读的输入：挂载参数、删除开关，与每一版（根身份、第一个文件那一刻的内容、落到它之后续做还差几次截断的发布）。
struct MountContinuingTheTruncationInputs {
    parameters: MakeFilesystemParameters,
    switches: DeletionSwitches,
    versions: Vec<VersionOfTheFlow>,
}

/// 流里有文件的一版。
struct VersionOfTheFlow {
    root: (InstanceGeneration, CheckpointTxg),
    first_file_content: Vec<u8>,
    truncation_publishes_left: usize,
}

impl MountContinuingTheTruncationInputs {
    /// 进观察者的版本与去重键：参数与开关的调试写法、逐版一行（根身份、内容的 SHA-256、续做还差几次），取 SHA-256。
    fn digest(&self) -> [u8; DIGEST_BYTES] {
        let mut text = format!("{:?}\n{:?}\n", self.parameters, self.switches);
        for version in &self.versions {
            text.push_str(&format!(
                "version instance={} txg={} content={:?} truncation_publishes_left={}\n",
                version.root.0 .0,
                version.root.1 .0,
                sha256_digest(&version.first_file_content),
                version.truncation_publishes_left
            ));
        }
        sha256_digest(text.as_bytes())
    }
}

/// 扫出贴边那一格，另起一个录写流的池走到同一格（同一种起法、同一串写，盘上逐字节相同），录截断那三步。
fn record_the_truncation_at_the_admission_edge() -> RecordedTruncationAtTheEdge {
    let data_units_at_the_edge = data_units_at_the_admission_edge();
    let stream = SharedStream::retaining_contents();
    let stream_for_the_devices = stream.clone();
    let mut pool = pool_after_the_remount(move |identity, device| {
        RecordingBlockDevice::with_shared_stream(identity, device, stream_for_the_devices.clone())
    });
    pool.sequential_write(content_length_of_data_units(data_units_at_the_edge))
        .unwrap_or_else(|refusal| {
            panic!("录写流的池上顺序写到贴边那一格（{data_units_at_the_edge} 个数据单元）：{refusal:?}")
        });
    let base = pool.image();
    let operations_before_the_truncation = stream.operation_count();
    let mut versions = pool.published_versions();
    let switches =
        deletion_switches_splitting_the_truncation_into_two_batches(data_units_at_the_edge);
    let write_time_seconds_of_the_intent = pool.write_time_seconds + 1;
    let mut session = pool.session.take().expect("崩了再挂之后有可写会话");
    let mut steps = Vec::new();
    {
        let PoolVersion::WithFile(current) = &mut session.current else {
            panic!("贴边那一版带文件");
        };
        let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
        writer.set_container_tree_capacities(switches.container_tree_capacities());
        for step in STEPS_AT_THE_EDGE {
            let published = step
                .publish(
                    &mut writer,
                    &mut session.allocator,
                    current,
                    session.instance,
                    switches.batch_size,
                    write_time_seconds_of_the_intent,
                )
                .unwrap_or_else(|refusal| {
                    panic!("贴边的池上截断的 {step:?} 那一步（删除类发布只判落点）：{refusal:?}")
                });
            versions.push(PublishedVersion {
                instance: published.root.instance,
                checkpoint_txg: published.root.checkpoint_txg,
                content: Vec::new(),
            });
            *current = published.clone();
            steps.push((step, published));
        }
    }
    pool.session = Some(session);
    let recorded = stream.retained_operations()[operations_before_the_truncation..].to_vec();
    let (writes, segments) =
        writes_and_segments(&recorded, &fixed_geometry_of_the_384_slot_small_devices());
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("截断那三步写了根槽");
    let mount_inputs = MountContinuingTheTruncationInputs {
        parameters: pool.parameters.clone(),
        switches,
        versions: versions
            .iter()
            .map(|version| VersionOfTheFlow {
                root: (version.instance, version.checkpoint_txg),
                first_file_content: version.content.clone(),
                truncation_publishes_left: steps
                    .iter()
                    .find(|(_, output)| {
                        (output.root.instance, output.root.checkpoint_txg)
                            == (version.instance, version.checkpoint_txg)
                    })
                    .map_or(0, |(step, _)| step.truncation_publishes_left_after()),
            })
            .collect(),
    };
    println!(
        "TRUNCATE_TO_ZERO_ON_A_NEARLY_FULL_POOL data_units_at_the_edge={data_units_at_the_edge} batch_size={:?} segments={:?}",
        switches.batch_size,
        segments.iter().map(Vec::len).collect::<Vec<usize>>()
    );
    RecordedTruncationAtTheEdge {
        data_units_at_the_edge,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        steps,
        mount_inputs,
    }
}

/// 录流的形状（判器判完之后再核：改坏实现时先红在判器上）：三步各写一次根槽；意图之后截断条目在、N 个单元都还挂着；一批之后条目在、
/// 少了一批；最后一批之后条目删了、只剩单元 0。
fn assert_the_shape_of_the_steps(recorded: &RecordedTruncationAtTheEdge) {
    let root_writes = recorded
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .count();
    assert_eq!(
        root_writes,
        STEPS_AT_THE_EDGE.len(),
        "截断那三步各写一次根槽"
    );
    let batch = recorded
        .data_units_at_the_edge
        .checked_sub(1)
        .expect("至少一个单元")
        .div_ceil(2);
    let expected: Vec<(TruncationStepAtTheEdge, bool, u64)> = vec![
        (
            TruncationStepAtTheEdge::Intent,
            true,
            recorded.data_units_at_the_edge,
        ),
        (
            TruncationStepAtTheEdge::Batch,
            true,
            recorded.data_units_at_the_edge - batch,
        ),
        (TruncationStepAtTheEdge::LastBatch, false, 1),
    ];
    let found: Vec<(TruncationStepAtTheEdge, bool, u64)> = recorded
        .steps
        .iter()
        .map(|(step, output)| {
            (
                *step,
                live_truncation_intent_of_the_entries(&output.logged_operations).is_some(),
                u64::try_from(output.data_pointers.len()).expect("单元数装得进 u64"),
            )
        })
        .collect();
    assert_eq!(
        found, expected,
        "每一步之后（截断条目在不在、第一个文件挂着几个数据单元）"
    );
}

/// 可写挂载续做截断那个观察者看一个状态（文件头那一段）：在崩溃镜像落成的盘的副本上可写挂载一次（空间准入判着、删除开关同录流），
/// 共用走空把截断走完；续做发的次数等于恢复落到的那一版之后录流还剩的截断发布数；挂完 logged_ops 里没有截断条目、第一个文件经挂载态读者读出那一版的内容。
/// 挂载发的写录在一条新的录制流上，连同挂完的盘交回：判段在挂完的盘上跑池级 checker。
///
/// # Panics
/// 可写挂载做不成（贴边的池上准入拒了挂载），恢复落到流外的一版，或上面任何一样对不上。
// gpu-exempt: writable-mount
// gpu-exempt: continuation
fn observe_the_mount_continuing_the_truncation_at_the_edge(
    inputs: &MountContinuingTheTruncationInputs,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) -> ImageAfterTheObserver {
    let effective_root = report
        .effective_root
        .expect("每个状态都择到根（oracle 另判）");
    let landed = inputs
        .versions
        .iter()
        .find(|version| version.root == effective_root)
        .unwrap_or_else(|| panic!("恢复落到 {effective_root:?}，不是这条流里有文件的一版"));
    let materialized = materialized_crash_image(image);
    let stream = SharedStream::retaining_contents_and_reads();
    let mut devices: Vec<_> = materialized
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                materialized.device_size_in_bytes,
                PhysicalBlockSizeInBytes(inputs.parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect();
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &inputs.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        inputs.switches,
    )
    .unwrap_or_else(|error| {
        panic!("贴边的池上每个崩溃状态都可写挂载得成（落到 {effective_root:?}）：{error:?}")
    });
    assert!(
        matches!(
            mounted.output.deletion_queue_continuation_outcome,
            DeletionQueueContinuationOutcome::WalkedToEmpty
        ),
        "落到 {effective_root:?}：共用走空把队列走到空：{:?}",
        mounted.output.deletion_queue_continuation_outcome
    );
    assert_eq!(
        mounted.output.deletion_queue_continuation.len(),
        landed.truncation_publishes_left,
        "恢复落到 {effective_root:?}：录流在这一版之后还剩几次截断的发布，续做就发几次"
    );
    counts.add(
        &format!(
            "truncation_continuation_publishes_{}",
            landed.truncation_publishes_left
        ),
        1,
    );
    let disks_after_the_observer = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: materialized.device_size_in_bytes,
    };
    let current = mounted
        .current
        .file_version()
        .expect("这条流里每一版都带文件，挂完的现行那一版也带");
    assert!(
        live_truncation_intent_of_the_entries(&current.logged_operations).is_none(),
        "落到 {effective_root:?}：挂完 logged_ops 里没有截断条目"
    );
    let opened = open_pool_for_read(&disks_after_the_observer, &current.root)
        .unwrap_or_else(|failure| panic!("落到 {effective_root:?}：挂完打得开挂载态：{failure:?}"));
    let file = opened
        .open_file(&disks_after_the_observer, InodeNumber(FIRST_INODE_NUMBER))
        .unwrap_or_else(|failure| {
            panic!("落到 {effective_root:?}：挂完打得开第一个文件：{failure:?}")
        });
    let size = file.inode_record().size;
    let read = file
        .read_at(&disks_after_the_observer, FileOffsetInBytes(0), size)
        .unwrap_or_else(|failure| panic!("落到 {effective_root:?}：从 0 读到 size：{failure:?}"))
        .bytes;
    assert!(
        read == landed.first_file_content,
        "落到 {effective_root:?}：挂完第一个文件读出那一版的内容（{} 字节），实际 {} 字节",
        landed.first_file_content.len(),
        read.len()
    );
    let (writes_on_the_crash_image, _segments_of_the_mount) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    ImageAfterTheObserver {
        writes_on_the_crash_image,
        disks_after_the_observer,
        ranges_read_on_the_crash_image: ReadRangeOnTheCrashImage::of_the_recorded_read_ranges(
            &stream.read_ranges(),
        ),
        invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
        counter_when_the_pool_checker_is_clean: POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER
            .to_string(),
    }
}

/// 快档的展开：写数小于 10 的段任意真子集，别的段不展开（只以整段持久进入后面的状态）。
fn expansion_of_the_segments_shorter_than_ten(
    _segment_index: usize,
    segment: &[usize],
) -> CrashPointReplaySegmentExpansion {
    if segment.len() < 10 {
        CrashPointReplaySegmentExpansion::EveryProperSubset
    } else {
        CrashPointReplaySegmentExpansion::NotExpanded
    }
}

/// 按 `expansion` 枚举这条流，每个状态交可写挂载续做截断那个观察者；项名是 `<流名>::<标签>`。
fn enumerate_the_truncation_at_the_edge(
    recorded: &RecordedTruncationAtTheEdge,
    item_label: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> Option<CrashPointReplayTally> {
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts|
     -> ImageAfterTheObserver {
        observe_the_mount_continuing_the_truncation_at_the_edge(
            &recorded.mount_inputs,
            image,
            report,
            counts,
        )
    };
    let item_name = format!("{FLOW_NAME_AT_THE_ENUMERATION}::{item_label}");
    enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(&recorded.versions),
        expansion,
        &[],
        // 观察者另读的输入：挂载参数、删除开关与逐版的内容、续做还差几次（`MountContinuingTheTruncationInputs::digest`）
        &[CrashPointReplayObserverHandingBackAnImage {
            name: "mount_continuing_the_truncation_at_the_edge",
            extra_inputs_digest: recorded.mount_inputs.digest(),
            observe: &observe,
            deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        }],
    )
    .judged()
}

/// 判器判的几样一个都不红：oracle 两遍、走读、记录核对器两条判据、池级 checker 的每一条不变量；每个状态都可写挂载过、挂完的盘池级 checker 干净；
/// 名单里的不变量都真被评估过。
fn assert_the_judges_and_the_mount_are_clean(tally: &CrashPointReplayTally) {
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
    for invariant in INVARIANTS_EVALUATED_ON_SOME_STATE {
        assert!(
            tally
                .checker_evaluated_states
                .get(invariant)
                .copied()
                .unwrap_or(0)
                > 0,
            "{invariant} 至少在一个状态上真被评估过：{}",
            tally.checker_counts_by_invariant()
        );
    }
    assert_eq!(tally.observed_states, tally.states, "每个状态都看过");
    assert_eq!(
        tally
            .observer_counts
            .count_named(POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER),
        tally.states,
        "每个状态都挂得上、挂完的盘池级 checker 0 违例：{:?}",
        tally.observer_counts
    );
    let continuation_publishes: u64 = (0..=2)
        .map(|publishes| {
            tally
                .observer_counts
                .count_named(&format!("truncation_continuation_publishes_{publishes}"))
        })
        .sum();
    assert_eq!(
        continuation_publishes, tally.states,
        "每个状态的续做都发了 0、1 或 2 次：{:?}",
        tally.observer_counts
    );
    for publishes in 0..=2 {
        assert!(
            tally
                .observer_counts
                .count_named(&format!("truncation_continuation_publishes_{publishes}"))
                > 0,
            "续做发 {publishes} 次的状态真跑到了：{:?}",
            tally.observer_counts
        );
    }
}

/// 平时跑的那一份：写数不小于 10 的段不展开，其余每段任意真子集；每个状态可写挂载并续做截断。
/// 判器先判，录流的形状在它之后核：改坏实现时先红在判器上。
#[test]
fn every_crash_state_of_a_truncation_to_zero_at_the_admission_edge_reads_the_size_before_or_after_and_remounts_to_finish_it(
) {
    let recorded = record_the_truncation_at_the_admission_edge();
    let Some(tally) = enumerate_the_truncation_at_the_edge(
        &recorded,
        "fast",
        &expansion_of_the_segments_shorter_than_ten,
    ) else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_TRUNCATE_TO_ZERO_ON_A_NEARLY_FULL_POOL_FAST states={} observer {:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_the_judges_and_the_mount_are_clean(&tally);
    assert_eq!(
        tally.states,
        crash_state_count_with_torn_in_place_overwrites(
            &recorded.base,
            &recorded.writes,
            &recorded.segments,
            &expansion_of_the_segments_shorter_than_ten,
        ),
        "展开的段按崩溃点重放的枚举域数（原地覆写取三态）"
    );
    assert_the_shape_of_the_steps(&recorded);
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow(
        RunOfTheFlow::CrashPointReplayEnumeration,
        FLOW_NAME_AT_THE_ENUMERATION,
        &tally,
    );
}

/// 全量：每段都展开，每个状态两遍恢复 + checker + 可写挂载续做截断。登记成崩溃枚举用例，
/// checker-tier-crash-point-replay 认下面打印的 `CRASH_POINT_REPLAY_TRUNCATE_TO_ZERO_ON_A_NEARLY_FULL_POOL` 行里 `exhaustive=true`。
#[test]
#[ignore = "全量约 2.7 亿个状态（按草稿副本录出的段形 [28,2,1,2,20,2,1,2,14,2,1,2] 推的两态闭式，崩溃点重放的枚举域以用例打出的为准）、每个两遍恢复 + checker + 可写挂载续做；门禁 checker-tier-crash-point-replay --full 在 release 下跑，可双机分片"]
fn full_enumeration_of_the_truncate_to_zero_on_a_nearly_full_pool_stream_is_exhaustive_and_clean() {
    let recorded = record_the_truncation_at_the_admission_edge();
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &full_expansion,
    );
    let Some(tally) = enumerate_the_truncation_at_the_edge(&recorded, "full", &full_expansion)
    else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_TRUNCATE_TO_ZERO_ON_A_NEARLY_FULL_POOL states={} closed_form={closed_form} exhaustive={} violations={} data_units_at_the_edge={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        recorded.data_units_at_the_edge,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_the_judges_and_the_mount_are_clean(&tally);
    assert_eq!(tally.states, closed_form);
    assert_the_shape_of_the_steps(&recorded);
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow(
        RunOfTheFlow::CrashPointReplayEnumeration,
        FLOW_NAME_AT_THE_ENUMERATION,
        &tally,
    );
}

/// 崩溃放量的形态（用户 2026-09-29 定：崩溃注入之外的崩溃枚举用例都进流水线、由 GPU 判）：贴边那一版的盘面是起点，只录截断那三步，一个点罩全部写；
/// 判器是流水线的（两遍恢复、oracle、池级 checker、记录核对器）。可写挂载续做那一遍不在流水线里，归上面的崩溃枚举用例。展开上限从环境取（没设 6）。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_of_the_truncate_to_zero_on_a_nearly_full_pool_stream_is_clean() {
    let recorded = record_the_truncation_at_the_admission_edge();
    let crash_points = common_crash_points::one_crash_point_over_every_write(
        file!(),
        "truncation_at_the_admission_edge",
        recorded.writes.len(),
    );
    common_crash_points::amplify_prepared_flow(
        FLOW_NAME_AT_THE_AMPLIFICATION,
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        &recorded.versions,
        crash_points,
        6,
    );
}
