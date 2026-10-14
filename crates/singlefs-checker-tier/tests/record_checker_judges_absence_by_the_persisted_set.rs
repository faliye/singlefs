//! checker 档模块：crash、crash_state_enumeration、crash_point_replay_progress、crash_given_states、crash_library、crash_amplification、gpu_unit_checks、crash_judge_tables、crash_judge_dispatch
//! C561（记录核对器的复用豁免在复用只落一半时假红）：记录核对器第二条判据按扇区、拿枚举器给的持久集合判一份单元副本缺不缺席
//! （D13（验证路线） 已定项 7：记录核对器的入参含持久集合；判据原文在 `crash::check_records_against` 的文档注释上）。
//!
//! C561 那一行要的四格各一条用例，前三格都在同一条可达历史 `UOOUOMSU` 上（A、B 之后：卸载重挂、覆盖写两次、卸载重挂、覆盖写、
//! 进程重开重挂、写一个十几字节的小文件、卸载重挂；崩溃点重放规模第二、三轮攻方的历史，字母的意思照他们的探针）：
//! - B（txg 4）在相邻两槽写两个 16K 节点 u、u2；txg 17 的 32K 数据单元 l 复用这两槽；txg 25 在同样两槽写两个 16K 节点 y、x，
//!   y、x 与 σ 那一段的其余 14 个单元写同段（σ：16 个单元写；C577（发布返回之前、系统配置轮换之后一道屏障）之前上一次发布的
//!   两个系统配置槽轮换也在这一段，2 个原地写 + 16 个单元写，C577 之后它们过了那道屏障、在 σ 前一段）；
//! - 假红状态 X：σ 之前整段持久（上一次发布的轮换在内）、σ 的单元写只落 y（两盘）——u2 那一槽上是 l 的零尾，l 落了、合法复用，
//!   按整份判的那一版判 u2 缺席（假红），按扇区、拿持久集合判不缺席；
//! - 状态 Y（C507（记录核对器的复用豁免比登记的候选宽，把真洞变哑） 那一格的错位形）：X 再把 u2 与 l 摘成没落盘——u2 那一槽上是从没写过的零，
//!   与 X 崩溃镜像逐字节相同，只有持久集合分得开：u2 真的缺席；
//! - 那处假绿：X 只把 l 摘成没落盘——y 整份在位、只盖住 l 的一半，按整份开脱的那一版整份放过，l 的另一半（u2 那一槽）不是它的
//!   又没有落了的写解释，判缺席；同偏移那一格（σ 的单元写全不落、u 与 l 都没落）判缺席；
//! - C513（复用豁免不判那次复用合不合法） 那三个状态（复用窗口置 0、txg 5 的数据单元落回 txg 4 的那一对槽）：解释那一槽的后写落了、
//!   但过不了回收谓词，照判缺席。
//!
//! σ 那一段全部 2^16 个状态（C577 之前 2^18）的记录核对器判定另有一条标了 ignore 的全量（崩溃枚举，用户要求时由主 agent 经门禁 checker-tier-crash-point-replay --full 跑，提交时的快档只核它的全绿标记）。
//! 全量与它的小段那一条都经给定状态表那一类的判定库两段跑（用户 2026-10-13 定「能走两段的都走两段」）：掩码区间切成固定的几片、一片一张给定状态表，
//! 录入段写进库、判段从库里读回判（配了 GPU 走 GPU 判器，没配由 CPU 判器判），结论按块存、判过的片下一趟复用，打断重跑同一条命令只补没判的片。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[cfg(feature = "verdict-store")]
mod common_crash_points;

use common::{
    build_pool, file_content, geometry, parameters, publish_overwrite_in_process, BuiltPool,
};
use singlefs_checker_tier::crash_state_enumeration::crash_point_replay_worker_threads;
#[cfg(feature = "gpu")]
use singlefs_checker_tier::crash_state_enumeration::{
    sample_crash_states_on_the_cpu, CrashPointReplayItem,
};

/// 这份测试文件的仓内路径：崩溃点重放取样过库的设点文件（GPU 对拍那一条的 CPU 一侧用）、σ 那几张给定状态表的设点文件。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs";

/// 这一趟掩码切片的工作线程数与它从哪来（崩溃点重放判段同一个读法：`SINGLEFS_CRASH_POINT_REPLAY_THREADS`，`scripts/hosts/capped.sh` 设它）。
#[derive(Clone, Copy, Debug)]
struct WorkerThreadsOfTheRun {
    worker_threads: std::num::NonZeroUsize,
    worker_threads_source: &'static str,
}

impl WorkerThreadsOfTheRun {
    fn from_environment() -> Self {
        let (worker_threads, worker_threads_source) = crash_point_replay_worker_threads();
        Self {
            worker_threads: std::num::NonZeroUsize::new(worker_threads).expect("至少 1 个"),
            worker_threads_source,
        }
    }
}
use singlefs_core::address::{CheckpointTxg, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::ReuseWindow;
use singlefs_core::mount::{mount_writable, unmount, ShadowLedger, Unmounted};
use singlefs_core::recovery::PoolReader;
use singlefs_core::transaction::PoolVersion;
use singlefs_harness::memory_pool::{
    closed_form_state_count, root_identity_written_by, writes_and_segments,
    writes_and_segments_with_stream_indexes_and_entries, CrashImage, MemoryPool, PublishedVersion,
    RecordCheck, RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordedEntrySpan, RecordedPublishEntry};

/// 16K 节点槽的宽：l 是两槽宽的数据单元，u、u2、x、y 各一槽。
const NODE_SLOT_BYTES: u64 = 16 * 1024;

#[cfg(feature = "gpu")]
const CPU_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_JUDGE_CPU_THREADS";

/// σ 对拍里 CPU 判器线程数变量的值（`None` 是没设）：设了照它，没设取每台默认重活线程数（`thread_cap_value` 是 `SINGLEFS_THREAD_CAP`
/// 的值，设了照它，再没设取 `.claude/heavy-thread-default` 的登记值，判法在 `singlefs_harness::heavy_threads`）；设了而不是正整数（0 也算）
/// 就 panic、消息带变量名与原值，不悄悄换成默认（与 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO` 同一判法，经
/// `singlefs_harness::numeric_environment_variable::number_from_the_variable_value_panicking_when_unreadable`）。
///
/// # Panics
/// 设了而不是正整数；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数。
#[cfg(feature = "gpu")]
fn cpu_threads_from_the_value_of_the_variable(
    value: Option<&str>,
    thread_cap_value: Option<&str>,
) -> std::num::NonZeroUsize {
    singlefs_harness::heavy_threads::own_variable_or_default_heavy_threads(
        CPU_THREADS_ENVIRONMENT_VARIABLE,
        value,
        thread_cap_value,
    )
}

/// CPU 判器线程数变量没设：取登记的每台默认重活线程数，不取本机核数；设了照它。
#[cfg(feature = "gpu")]
#[test]
fn crash_judge_cpu_threads_of_the_sigma_comparison_take_the_registered_heavy_thread_default_when_unset(
) {
    assert_eq!(
        cpu_threads_from_the_value_of_the_variable(None, None),
        singlefs_harness::heavy_threads::registered_heavy_threads(),
        "SINGLEFS_CRASH_JUDGE_CPU_THREADS 与 SINGLEFS_THREAD_CAP 都没设取 .claude/heavy-thread-default 的登记值"
    );
    assert_eq!(
        cpu_threads_from_the_value_of_the_variable(None, Some("5")).get(),
        5
    );
    assert_eq!(
        cpu_threads_from_the_value_of_the_variable(Some("3"), Some("5")).get(),
        3
    );
}

/// 这条流的入口文件不引随机历史脚手架（`singlefs_harness` 的 `history` 模块）：登记的崩溃枚举用例按模块引用收判法代码，
/// 入口文件一引它，`history.rs` 连同它带进来的模型、对拍、场景、故障注入就都进了这一格的判法摘要，改随机历史就让这一格整格重判。
/// 进程环境变量的读法经 `numeric_environment_variable` 直接引（`history` 不转出它）。
/// 读源文件判、不调 `crash_identity` 的闭包函数：引 `crash_identity` 本身就会把它收进这一格的判法代码。
#[test]
fn this_flow_does_not_reference_the_random_history_scaffolding() {
    let source_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/record_checker_judges_absence_by_the_persisted_set.rs"
    );
    let source = std::fs::read_to_string(source_path)
        .unwrap_or_else(|error| panic!("读不了这条流自己的源文件 {source_path}：{error}"));
    let random_history_module = ["singlefs_harness", "history"].join("::");
    assert!(
        !source.contains(&random_history_module),
        "这条流的入口文件引了 {random_history_module}：随机历史脚手架会进这一格的判法摘要"
    );
}

#[cfg(feature = "gpu")]
#[test]
#[should_panic(expected = "SINGLEFS_CRASH_JUDGE_CPU_THREADS=\"sixteen\" 认不出")]
fn an_unreadable_crash_judge_cpu_threads_value_panics_naming_the_variable_and_the_value() {
    let _threads = cpu_threads_from_the_value_of_the_variable(Some("sixteen"), None);
}

fn second_content() -> Vec<u8> {
    (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 覆盖写 `O` 的内容（攻方探针的 `later_content(80 + k)`，k 是字母在历史里的位置）。
fn overwrite_content(letter_position: usize) -> Vec<u8> {
    let seed = 80 + letter_position;
    (0..3000 + seed)
        .map(|index| u8::try_from((index * 5 + seed) % 251).expect("小于 256"))
        .collect()
}

/// 小文件 `S` 的内容：10 + k 个同一个字节。
fn small_file_content(letter_position: usize) -> Vec<u8> {
    vec![u8::try_from(letter_position % 200).expect("小于 200") + 1; 10 + letter_position]
}

/// 历史里一步用户动作（攻方探针的字母）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UserAction {
    /// `O`：覆盖写一次（约 3 KiB）。
    Overwrite,
    /// `S`：覆盖写成一个十几字节的小文件。
    SmallFile,
    /// `M`：进程重开、可写挂载（新实例：取号、写行、暖机）。
    ReopenTheProcessAndMount,
    /// `U`：正常卸载（录成卸载入口发的那一段），再进程重开、可写挂载。
    UnmountThenReopenAndMount,
}

/// `UOOUOMSU`。
const HISTORY_UOOUOMSU: [UserAction; 8] = [
    UserAction::UnmountThenReopenAndMount,
    UserAction::Overwrite,
    UserAction::Overwrite,
    UserAction::UnmountThenReopenAndMount,
    UserAction::Overwrite,
    UserAction::ReopenTheProcessAndMount,
    UserAction::SmallFile,
    UserAction::UnmountThenReopenAndMount,
];

struct PoolUnderHistory {
    pool: BuiltPool,
    instance: InstanceGeneration,
    /// 现行那一版的文件内容：挂载与卸载推出的空发布沿用它。
    current_content: Vec<u8>,
    /// 每一版（实例、txg、内容），崩溃放量的 oracle 按它认恢复落到的是哪一版；从第一个文件那一版起记。
    versions: Vec<PublishedVersion>,
}

impl PoolUnderHistory {
    fn overwrite(&mut self, content: &[u8]) {
        let previous = self.pool.output.clone();
        self.pool.output = publish_overwrite_in_process(
            &mut self.pool,
            &previous,
            content,
            common::FIXED_WRITE_TIME_SECONDS + 60,
            self.instance,
        )
        .expect("覆盖写");
        self.current_content = content.to_vec();
        self.versions.push(PublishedVersion {
            instance: self.instance,
            checkpoint_txg: self.pool.output.root.checkpoint_txg,
            content: content.to_vec(),
        });
    }

    fn reopen_and_mount(&mut self) {
        let previous_txg = self.pool.output.root.checkpoint_txg;
        let mut devices = self.pool.reopen_recorded();
        let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
        self.pool.devices = Some(devices);
        self.pool.allocator = mounted.allocator;
        self.instance = mounted.current.root().instance;
        let mounted_txg = mounted.current.root().checkpoint_txg;
        self.pool.output = mounted
            .current
            .into_file_version()
            .expect("这段历史上每一版都带文件");
        // 取号、写行、暖机各推一版，内容不变
        for txg in previous_txg.0 + 1..=mounted_txg.0 {
            self.versions.push(PublishedVersion {
                instance: self.instance,
                checkpoint_txg: CheckpointTxg(txg),
                content: self.current_content.clone(),
            });
        }
    }

    fn unmount(&mut self) {
        let mut current = PoolVersion::WithFile(self.pool.output.clone());
        let stream = self.pool.stream.clone();
        let devices = self.pool.devices.as_mut().expect("镜像还开着");
        let allocator = &mut self.pool.allocator;
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
        let Unmounted::FloorRaisedToTheCurrentVersion(raised) = unmounted else {
            panic!("带文件的一版上卸载要推一串抬 F，这次却一个字节都没写")
        };
        // 卸载推的每一次空发布一版，内容不变
        for publish in &raised.publishes {
            self.versions.push(PublishedVersion {
                instance: self.instance,
                checkpoint_txg: publish.root.checkpoint_txg,
                content: self.current_content.clone(),
            });
        }
        self.pool.output = current
            .into_file_version()
            .expect("带文件的一版卸载之后仍带文件");
    }
}

/// 一条历史的录制流：mkfs 之后的基线、写表、段，与每一版（崩溃放量的 oracle 用）。
struct RecordedHistory {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    #[cfg_attr(
        not(feature = "verdict-store"),
        allow(
            dead_code,
            reason = "只有崩溃放量那条用例读它，那条挂 verdict-store 特性"
        )
    )]
    versions: Vec<PublishedVersion>,
}

fn record_history(tag: &str, actions: &[UserAction]) -> RecordedHistory {
    let pool = build_pool(tag);
    let first_file_version = PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: pool.output.root.checkpoint_txg,
        content: file_content(),
    };
    let mut history = PoolUnderHistory {
        pool,
        instance: InstanceGeneration(1),
        current_content: file_content(),
        versions: vec![first_file_version],
    };
    history.overwrite(&second_content());
    for (letter_position, action) in actions.iter().enumerate() {
        match action {
            UserAction::Overwrite => history.overwrite(&overwrite_content(letter_position)),
            UserAction::SmallFile => history.overwrite(&small_file_content(letter_position)),
            UserAction::ReopenTheProcessAndMount => history.reopen_and_mount(),
            UserAction::UnmountThenReopenAndMount => {
                history.unmount();
                history.reopen_and_mount();
            }
        }
    }
    let recorded_pool = &history.pool;
    let operations = recorded_pool.retained_operations();
    let entry_spans: Vec<RecordedEntrySpan> = recorded_pool
        .stream
        .entry_spans()
        .into_iter()
        .map(|span| RecordedEntrySpan {
            entry: span.entry,
            operations: span.operations.start - recorded_pool.mkfs_operation_count
                ..span.operations.end - recorded_pool.mkfs_operation_count,
        })
        .collect();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &operations[recorded_pool.mkfs_operation_count..],
        &entry_spans,
        &geometry(),
    );
    RecordedHistory {
        base: recorded_pool.memory_pool_after_mkfs(),
        writes,
        segments,
        versions: history.versions.clone(),
    }
}

/// 写表下标 `index` 那次写属于哪次发布：它后面第一条根槽写写出的根的 checkpoint_txg。
fn publish_txg_of(writes: &[RetainedWrite], index: usize) -> Option<CheckpointTxg> {
    writes[index..]
        .iter()
        .find(|write| write.kind == StepKind::RootRecordFua)
        .map(|root_write| root_identity_written_by(root_write.bytes().expect("根槽写带着字节")).1)
}

/// 同一个单元在另一块盘上的那一份：单元的两份紧挨着写（先 0 盘后 1 盘）。
fn copy_on_the_other_device(writes: &[RetainedWrite], index: usize) -> usize {
    let write = &writes[index];
    [index.checked_sub(1), Some(index + 1)]
        .into_iter()
        .flatten()
        .find(|neighbour| {
            writes.get(*neighbour).is_some_and(|candidate| {
                candidate.kind == StepKind::UnitWrite
                    && candidate.offset == write.offset
                    && candidate.device != write.device
            })
        })
        .expect("单元写两盘各一份、紧挨着写")
}

/// C561 那一形的五个角色（都是 0 盘那一份的写表下标）与 σ 的段号。
struct MisalignedReuseChain {
    sigma: usize,
    /// u：B 在 l 头一槽的节点。
    first_node_of_b: usize,
    /// u2：B 在 l 第二槽的节点。
    second_node_of_b: usize,
    /// l：txg 17 的 32K 数据单元，复用 u、u2 两槽。
    data_unit: usize,
    /// y：txg 25 在 l 头一槽的节点。
    node_over_the_head_of_the_data_unit: usize,
    /// x：txg 25 在 l 第二槽的节点。
    node_over_the_tail_of_the_data_unit: usize,
}

fn unit_writes_on_device_zero_at(
    writes: &[RetainedWrite],
    indexes: std::ops::Range<usize>,
    slot_offset: u64,
    length_in_bytes: u64,
) -> Vec<usize> {
    indexes
        .filter(|index| {
            let write = &writes[*index];
            write.kind == StepKind::UnitWrite
                && write.device.0 == 0
                && write.offset.0 == slot_offset
                && write.length_in_bytes() == length_in_bytes
        })
        .collect()
}

/// 在写表里找「两个 16K 节点 → 一个 32K 数据单元 → 两个 16K 节点」那一串：必须恰好一串，发布各是 B（txg 4）、txg 17、txg 25，
/// 后两个节点同段（σ）。
fn misaligned_reuse_chain(recorded: &RecordedHistory) -> MisalignedReuseChain {
    let writes = &recorded.writes;
    let mut chains = Vec::new();
    for data_unit in 0..writes.len() {
        let write = &writes[data_unit];
        if write.kind != StepKind::UnitWrite
            || write.device.0 != 0
            || write.length_in_bytes() != 2 * NODE_SLOT_BYTES
        {
            continue;
        }
        let head = write.offset.0;
        let tail = head + NODE_SLOT_BYTES;
        let nodes_before = |slot_offset| {
            unit_writes_on_device_zero_at(writes, 0..data_unit, slot_offset, NODE_SLOT_BYTES)
        };
        let nodes_after = |slot_offset| {
            unit_writes_on_device_zero_at(
                writes,
                data_unit + 1..writes.len(),
                slot_offset,
                NODE_SLOT_BYTES,
            )
        };
        if let ([.., first_node_of_b], [.., second_node_of_b], [over_head, ..], [over_tail, ..]) = (
            nodes_before(head).as_slice(),
            nodes_before(tail).as_slice(),
            nodes_after(head).as_slice(),
            nodes_after(tail).as_slice(),
        ) {
            chains.push((
                *first_node_of_b,
                *second_node_of_b,
                data_unit,
                *over_head,
                *over_tail,
            ));
        }
    }
    let [(first_node_of_b, second_node_of_b, data_unit, over_head, over_tail)] = chains.as_slice()
    else {
        panic!("UOOUOMSU 的写表里「两节点 → 数据单元 → 两节点」要恰好一串，找到 {chains:?}");
    };
    let segment_of = |write_index: usize| {
        recorded
            .segments
            .iter()
            .position(|segment| segment.contains(&write_index))
            .expect("每次写都在某一段里")
    };
    assert_eq!(
        (
            publish_txg_of(writes, *first_node_of_b),
            publish_txg_of(writes, *second_node_of_b),
            publish_txg_of(writes, *data_unit),
            publish_txg_of(writes, *over_head),
            publish_txg_of(writes, *over_tail),
        ),
        (
            Some(CheckpointTxg(4)),
            Some(CheckpointTxg(4)),
            Some(CheckpointTxg(17)),
            Some(CheckpointTxg(25)),
            Some(CheckpointTxg(25)),
        ),
        "u、u2 是 B 的，l 是 txg 17 的，y、x 是 txg 25 的"
    );
    let sigma = segment_of(*over_head);
    assert_eq!(segment_of(*over_tail), sigma, "y、x 同段");
    let sigma_segment = &recorded.segments[sigma];
    assert_eq!(
        (
            sigma_segment.len(),
            sigma_segment
                .iter()
                .filter(|write_index| writes[**write_index].kind == StepKind::UnitWrite)
                .count()
        ),
        (16, 16),
        "σ：16 个单元写（C577 之后上一次发布的系统配置槽轮换过了发布末尾那道屏障、在 σ 前一段；改之前 2 个原地写 + 16 个单元写）"
    );
    MisalignedReuseChain {
        sigma,
        first_node_of_b: *first_node_of_b,
        second_node_of_b: *second_node_of_b,
        data_unit: *data_unit,
        node_over_the_head_of_the_data_unit: *over_head,
        node_over_the_tail_of_the_data_unit: *over_tail,
    }
}

/// σ 之前整段持久、σ 的原地写全落（C577 之后 σ 里没有原地写）、σ 的单元写一个都不落。
fn persisted_before_the_units_of_sigma(recorded: &RecordedHistory, sigma: usize) -> Vec<bool> {
    let mut persisted = vec![false; recorded.writes.len()];
    for segment in &recorded.segments[..sigma] {
        for write_index in segment {
            persisted[*write_index] = true;
        }
    }
    for write_index in &recorded.segments[sigma] {
        if recorded.writes[*write_index].kind != StepKind::UnitWrite {
            persisted[*write_index] = true;
        }
    }
    persisted
}

/// 一个单元的两份（0 盘那一份的下标给进来）一起设成落或不落。
fn set_both_copies(
    persisted: &mut [bool],
    writes: &[RetainedWrite],
    index_on_device_zero: usize,
    is_persisted: bool,
) {
    persisted[index_on_device_zero] = is_persisted;
    persisted[copy_on_the_other_device(writes, index_on_device_zero)] = is_persisted;
}

/// 这几条逐个摆状态的用例判状态的给定状态引擎（`crash_amplification::GivenStatesEngine`，用户 2026-10-13 定「能走两段的都走两段」）：
/// 摆出来的状态作一张给定状态表，录入段写进给定状态表那一类的判定库，判段从库里读回判（看 journal 那一遍恢复、记录核对器；配了 GPU 走 GPU 判器，
/// 没配由 CPU 判器判），结论按表存进库、判过的复用。项名按用例起，每条用例一个。
///
/// # Panics
/// 引擎起不了（配了 GPU 卡用不上、结论版本与编译时对不上……都不退回 CPU）。
fn given_states_engine_of_this_test(
    test_name: &str,
) -> singlefs_checker_tier::crash_amplification::GivenStatesEngine {
    let _one_test_at_a_time =
        singlefs_checker_tier::crash_amplification::one_test_changes_the_environment_at_a_time();
    singlefs_checker_tier::crash_amplification::GivenStatesEngine::start_from_environment(
        singlefs_checker_tier::crash_amplification::GivenStatesItem {
            item_name: format!("record_checker_judges_absence::{test_name}"),
            setting_file: THIS_FILE.to_string(),
            seeds_and_scale: "history=fixed".to_string(),
        },
        &singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        &singlefs_checker_tier::crash_library::repository_root_of_this_crate(),
    )
    .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
}

/// 一遍恢复落到的根在结论里那两个字换回 (实例, txg)：txg 只带低 32 位（这几段历史的 txg 都在几十以内），没落到根是 `None`。
fn effective_root_of_the_landed_root_words(
    landed: singlefs_checker_tier::crash_judge_conclusions::LandedRootWords,
) -> Option<(InstanceGeneration, CheckpointTxg)> {
    (landed != singlefs_checker_tier::crash_judge_conclusions::LandedRootWords::NOT_LANDED).then(
        || {
            (
                InstanceGeneration(landed.instance),
                CheckpointTxg(u64::from(landed.checkpoint_txg_low)),
            )
        },
    )
}

/// 一个状态上恢复实际走的根与记录核对器的判定。
type EffectiveRootAndRecordCheck = (Option<(InstanceGeneration, CheckpointTxg)>, RecordCheck);

/// 恢复实际走的根与记录核对器的判定，逐状态（与 `persisted_sets` 同序）：这几个状态作一张给定状态表交 `engine` 两段判
/// （[`given_states_engine_of_this_test`]）。只录那一趟、表归别的机器判的那一趟交 `None`（用例当场返回）。
///
/// # Panics
/// 引擎判不了这张表（不退回别处判）。
fn judge_the_states(
    engine: &singlefs_checker_tier::crash_amplification::GivenStatesEngine,
    recorded: &RecordedHistory,
    table_name: &str,
    persisted_sets: Vec<Vec<bool>>,
) -> Option<Vec<EffectiveRootAndRecordCheck>> {
    use singlefs_checker_tier::crash_given_states::{
        GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
    };
    let state_count = persisted_sets.len();
    let judged = engine
        .judge_the_given_states(&GivenStatesOfATable {
            table_name: table_name.to_string(),
            base: &recorded.base,
            writes: &recorded.writes,
            segments: &recorded.segments,
            seams: &[],
            persisted_sets,
        })
        .unwrap_or_else(|reason| {
            panic!("给定状态表 {table_name} 判不了（不退回别处判）：{reason}")
        });
    match judged.source {
        GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost => {}
        GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost
        | GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => return None,
    }
    assert_eq!(
        judged.states.len(),
        state_count,
        "判器逐状态交回结论，与表里的状态同序"
    );
    Some(
        judged
            .states
            .iter()
            .map(|state| {
                (
                    effective_root_of_the_landed_root_words(state.words.consulted_pass_landed_root),
                    state.record_check(),
                )
            })
            .collect(),
    )
}

/// 恢复实际走的根与记录核对器的判定（一个状态的 [`judge_the_states`]）。
fn judge(
    engine: &singlefs_checker_tier::crash_amplification::GivenStatesEngine,
    recorded: &RecordedHistory,
    table_name: &str,
    persisted: &[bool],
) -> Option<EffectiveRootAndRecordCheck> {
    judge_the_states(engine, recorded, table_name, vec![persisted.to_vec()])
        .map(|mut judged| judged.remove(0))
}

/// 收尾：关库、打引擎那一行；GPU 判不了的状态照第三轮正文 H6 判红。
fn finish_the_engine(engine: singlefs_checker_tier::crash_amplification::GivenStatesEngine) {
    assert_eq!(
        engine.finish().red_items(),
        Vec::<String>::new(),
        "GPU 判不了的状态照第三轮正文 H6 判红"
    );
}

/// 两个状态的崩溃镜像在写表碰过的每一处上逐字节相同（其余地方两边都是基线）。
fn crash_images_are_identical(recorded: &RecordedHistory, first: &[bool], second: &[bool]) -> bool {
    let first_image = CrashImage {
        base: &recorded.base,
        writes: &recorded.writes,
        persisted: first.to_vec(),
    };
    let second_image = CrashImage {
        base: &recorded.base,
        writes: &recorded.writes,
        persisted: second.to_vec(),
    };
    recorded.writes.iter().all(|write| {
        let length = usize::try_from(write.length_in_bytes()).expect("写长装得进 usize");
        PoolReader::read(&first_image, write.device, write.offset, length)
            == PoolReader::read(&second_image, write.device, write.offset, length)
    })
}

/// X 与 Y：崩溃镜像逐字节相同，持久集合不同；X 里 u2 那一槽是落了的 l 的零尾（合法复用，不缺席），Y 里 l 与 u2 都没落（u2 真的缺席）。
/// 按整份判的那一版把 X 判成缺席（假红）；只看盘上字节的按扇区判把 Y 判成不缺席（C507 那一格的洞放过）——两个判法各在一格上红。
#[test]
fn the_false_red_and_the_misaligned_hole_have_identical_crash_images_and_only_the_hole_is_missing()
{
    let recorded = record_history("c561-false-red-and-hole", &HISTORY_UOOUOMSU);
    let chain = misaligned_reuse_chain(&recorded);
    let writes = &recorded.writes;
    let slot_length = usize::try_from(NODE_SLOT_BYTES).expect("16K 装得进 usize");
    let mut data_unit_over_the_second_node = vec![0u8; slot_length];
    writes[chain.data_unit]
        .contents
        .copy_range_into(slot_length, &mut data_unit_over_the_second_node);
    assert!(
        data_unit_over_the_second_node.iter().all(|byte| *byte == 0),
        "前提：l 落在 u2 那一槽上的 16K 全是零填充（数据单元的内容只有约 3 KiB）"
    );

    let before_the_units_of_sigma = persisted_before_the_units_of_sigma(&recorded, chain.sigma);
    let mut false_red = before_the_units_of_sigma;
    set_both_copies(
        &mut false_red,
        writes,
        chain.node_over_the_head_of_the_data_unit,
        true,
    );
    let mut misaligned_hole = false_red.clone();
    set_both_copies(&mut misaligned_hole, writes, chain.second_node_of_b, false);
    set_both_copies(&mut misaligned_hole, writes, chain.data_unit, false);
    assert!(
        crash_images_are_identical(&recorded, &false_red, &misaligned_hole),
        "X 与 Y 的崩溃镜像逐字节相同：只看盘上字节分不开它们"
    );

    let engine = given_states_engine_of_this_test("false_red_and_misaligned_hole");
    let Some(judged) = judge_the_states(
        &engine,
        &recorded,
        "false-red-and-misaligned-hole",
        vec![false_red, misaligned_hole],
    ) else {
        return;
    };
    finish_the_engine(engine);
    let [(false_red_root, false_red_check), (hole_root, hole_check)] = judged.as_slice() else {
        panic!("两个状态两份结论，交回 {} 份", judged.len());
    };
    let (false_red_root, false_red_check, hole_root, hole_check) =
        (*false_red_root, *false_red_check, *hole_root, *hole_check);
    assert_eq!(false_red_root, hole_root, "镜像相同，恢复走同一条根");
    assert!(
        false_red_root.is_some_and(|(_, txg)| txg >= CheckpointTxg(17)),
        "恢复自称的 txg ≥ l 那次发布：记录核对器要判 B 与 l 的单元在不在（{false_red_root:?}）"
    );
    assert_eq!(
        false_red_check,
        RecordCheck {
            root_without_record: false,
            claimed_state_missing_unit: false,
        },
        "X：u2 那一槽被落了的 l 合法盖住、u 那一槽被落了的 y 合法盖住，一个单元都不缺席（C561 的假红）"
    );
    assert_eq!(
        hole_check,
        RecordCheck {
            root_without_record: false,
            claimed_state_missing_unit: true,
        },
        "Y：u2 与盖住它的 l 都没落盘，u2 那一槽上的零不是任何一次落了的写写下的，u2 缺席（C507 那一格）"
    );
}

/// 那处假绿：l 没落、y 整份在位只盖住 l 的一半——l 的另一半（u2 那一槽）上是落了的 u2，不是 l 的，又没有落了的写解释，l 缺席。
#[test]
fn a_data_unit_only_half_covered_by_a_landed_later_node_is_missing() {
    let recorded = record_history("c561-half-covered", &HISTORY_UOOUOMSU);
    let chain = misaligned_reuse_chain(&recorded);
    let writes = &recorded.writes;
    let mut half_covered = persisted_before_the_units_of_sigma(&recorded, chain.sigma);
    set_both_copies(
        &mut half_covered,
        writes,
        chain.node_over_the_head_of_the_data_unit,
        true,
    );
    set_both_copies(&mut half_covered, writes, chain.data_unit, false);
    let engine = given_states_engine_of_this_test("half_covered");
    let Some((effective_root, check)) = judge(&engine, &recorded, "half-covered", &half_covered)
    else {
        return;
    };
    finish_the_engine(engine);
    assert!(
        effective_root.is_some_and(|(_, txg)| txg >= CheckpointTxg(17)),
        "恢复自称的 txg ≥ l 那次发布（{effective_root:?}）"
    );
    assert!(
        check.claimed_state_missing_unit,
        "l 的后一半不是它的字节、没有落了的写解释：l 缺席（按整份开脱的那一版在这里判绿）"
    );
}

/// 同偏移那一格：σ 的单元写全不落、u 与 l 都没落——u 那一槽的后写一次都没落，u 缺席。
#[test]
fn a_node_whose_every_later_write_to_the_same_offset_never_landed_is_missing() {
    let recorded = record_history("c561-same-offset", &HISTORY_UOOUOMSU);
    let chain = misaligned_reuse_chain(&recorded);
    let writes = &recorded.writes;
    let mut same_offset_hole = persisted_before_the_units_of_sigma(&recorded, chain.sigma);
    set_both_copies(&mut same_offset_hole, writes, chain.first_node_of_b, false);
    set_both_copies(&mut same_offset_hole, writes, chain.data_unit, false);
    let engine = given_states_engine_of_this_test("same_offset_hole");
    let Some((effective_root, check)) =
        judge(&engine, &recorded, "same-offset-hole", &same_offset_hole)
    else {
        return;
    };
    finish_the_engine(engine);
    assert!(
        effective_root.is_some_and(|(_, txg)| txg >= CheckpointTxg(17)),
        "恢复自称的 txg ≥ l 那次发布（{effective_root:?}）"
    );
    assert!(check.claimed_state_missing_unit, "u 与 l 都缺席");
}

/// σ 的原地写取任意子集 × y、x 两盘四份写取任意子集，σ 其余 12 个单元写不落：C577 之后 σ 里没有原地写（上一次发布的轮换在 σ 前一段、
/// 整段持久），16 个状态（改之前两个原地写、2^2 × 2^4 = 64 个），记录核对器一个都不判缺席。
/// 按整份判的那一版在 y、x 只落一部分的状态上判 u 或 u2 缺席（C561 的假红那一类）。
#[test]
fn partial_landing_of_the_two_later_nodes_never_reports_a_unit_missing() {
    let recorded = record_history("c561-partial-landing", &HISTORY_UOOUOMSU);
    let chain = misaligned_reuse_chain(&recorded);
    let writes = &recorded.writes;
    let in_place_writes_of_sigma: Vec<usize> = recorded.segments[chain.sigma]
        .iter()
        .copied()
        .filter(|write_index| writes[*write_index].kind != StepKind::UnitWrite)
        .collect();
    let later_node_copies = [
        chain.node_over_the_head_of_the_data_unit,
        copy_on_the_other_device(writes, chain.node_over_the_head_of_the_data_unit),
        chain.node_over_the_tail_of_the_data_unit,
        copy_on_the_other_device(writes, chain.node_over_the_tail_of_the_data_unit),
    ];
    let before_sigma = persisted_before_the_units_of_sigma(&recorded, chain.sigma);
    let mut states = 0u64;
    let mut masks_of_each_state: Vec<(u32, u32)> = Vec::new();
    let mut persisted_sets: Vec<Vec<bool>> = Vec::new();
    for in_place_mask in 0..1u32 << in_place_writes_of_sigma.len() {
        for later_node_mask in 0..1u32 << later_node_copies.len() {
            let mut persisted = before_sigma.clone();
            for (bit, write_index) in in_place_writes_of_sigma.iter().enumerate() {
                persisted[*write_index] = in_place_mask & (1 << bit) != 0;
            }
            for (bit, write_index) in later_node_copies.iter().enumerate() {
                persisted[*write_index] = later_node_mask & (1 << bit) != 0;
            }
            states += 1;
            masks_of_each_state.push((in_place_mask, later_node_mask));
            persisted_sets.push(persisted);
        }
    }
    let engine = given_states_engine_of_this_test("partial_landing_of_the_two_later_nodes");
    let Some(judged) = judge_the_states(
        &engine,
        &recorded,
        "partial-landing-of-the-two-later-nodes",
        persisted_sets,
    ) else {
        return;
    };
    finish_the_engine(engine);
    let reported_missing: Vec<(u32, u32)> = masks_of_each_state
        .into_iter()
        .zip(&judged)
        .filter(|(_, (_, check))| check.claimed_state_missing_unit)
        .map(|(masks, _)| masks)
        .collect();
    assert_eq!(
        (in_place_writes_of_sigma.len(), states),
        (0, 16),
        "2^0 × 2^4：C577 之后 σ 只有单元写"
    );
    assert_eq!(
        reported_missing,
        Vec::<(u32, u32)>::new(),
        "y、x 只落一部分时 u、u2 那两槽要么还是它们自己的、要么被落了的合法后写盖住（原地写掩码，y、x 四份写的掩码）"
    );
}

/// 一段的全部子集（这一段之前整段持久、之后一个都不落，`persisted_before` 给这一段之外的写）逐个评过之后的数：
/// 评了几个状态、闭式该有几个、切成几片、起了几个工作线程、线程数从哪来、记录核对器判缺席几次、几片照库里复用、用了多久。
#[cfg(feature = "verdict-store")]
struct SubsetsOfOneSegmentEnumerated {
    states: u64,
    /// `crash::closed_form_state_count` 只对这一段算：1 + (2^|段| − 1) = 2^|段|（整段全落那一个就是闭式里那个 1）。
    closed_form_states: u64,
    slices: usize,
    spawned_worker_threads: usize,
    parallelism: WorkerThreadsOfTheRun,
    reported_missing: u64,
    /// 判定库里这一把判器版本早先判过、这一趟照库里复用的片数（一片一张给定状态表）。
    slices_reused: u64,
    elapsed_seconds: f64,
}

/// 掩码区间切成几片：固定 [`SIGMA_TABLE_SLICES`] 片（与线程数无关：一片一张给定状态表，表名与内容不随线程数变，判过的片下一趟复用），
/// 不多于状态数。
#[cfg(feature = "verdict-store")]
const SIGMA_TABLE_SLICES: u64 = 64;

/// 一片是掩码区间 [起, 止)。
#[cfg(feature = "verdict-store")]
fn mask_slices(state_count: u64) -> Vec<std::ops::Range<u64>> {
    let slice_count = SIGMA_TABLE_SLICES.min(state_count).max(1);
    (0..slice_count)
        .map(|slice_index| {
            state_count * slice_index / slice_count..state_count * (slice_index + 1) / slice_count
        })
        .collect()
}

/// 把 `segment` 的全部子集按掩码区间切片、一片一张给定状态表、多线程交崩溃点重放的引擎（`crash_amplification::GivenStatesEngine`）两段判
/// （`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」；用户 2026-10-13 定「能走两段的都走两段」）：录入段把表写进
/// 给定状态表那一类的判定库，判段从库里读回判（每个状态两遍恢复、池级 checker、记录核对器；配了 GPU 走 GPU 判器），结论按块存、判过的片复用。
/// 工作线程按片号领片，调用线程收到一片就打一行 `C561_SIGMA_PROGRESS`（片号、掩码区间、已跑完的片数与状态数），再按片号次序并计数——
/// 结果与线程数、调度次序无关。只录、叫停、两台分片还没导回来的那一趟打「本次未跑」那一行、交 `None`。
///
/// # Panics
/// 引擎起不了、判不了一张表；评状态的线程 panic；有一片领了却没交回（按片号并不齐）；GPU 判不了某几个状态（判红）。
#[cfg(feature = "verdict-store")]
fn enumerate_every_subset_of_one_segment(
    recorded: &RecordedHistory,
    segment: &[usize],
    persisted_before: &[bool],
    parallelism: WorkerThreadsOfTheRun,
    item_name: &str,
) -> Option<SubsetsOfOneSegmentEnumerated> {
    use singlefs_checker_tier::crash_amplification::{GivenStatesEngine, GivenStatesItem};
    use singlefs_checker_tier::crash_given_states::{GivenStatesJudge, GivenStatesOfATable};
    let started = std::time::Instant::now();
    let state_count = 1u64 << segment.len();
    let slices = mask_slices(state_count);
    let spawned_worker_threads = parallelism.worker_threads.get().min(slices.len());
    let engine = {
        let _one_test_at_a_time =
            singlefs_checker_tier::crash_amplification::one_test_changes_the_environment_at_a_time(
            );
        GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: item_name.to_string(),
                setting_file: THIS_FILE.to_string(),
                seeds_and_scale: format!(
                    "history=UOOUOMSU segment_writes={} slices={}",
                    segment.len(),
                    slices.len()
                ),
            },
            &singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
            &singlefs_checker_tier::crash_library::repository_root_of_this_crate(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
    };
    let next_slice_index = std::sync::atomic::AtomicUsize::new(0);
    let finished_by_slice = std::thread::scope(|scope| {
        let (sender, receiver) = std::sync::mpsc::channel::<(usize, u64, u64)>();
        for _ in 0..spawned_worker_threads {
            let sender = sender.clone();
            let slices = &slices;
            let next_slice_index = &next_slice_index;
            let engine = &engine;
            scope.spawn(move || loop {
                let slice_index =
                    next_slice_index.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some(slice) = slices.get(slice_index) else {
                    break;
                };
                let persisted_sets: Vec<Vec<bool>> = slice
                    .clone()
                    .map(|mask| {
                        let mut persisted = persisted_before.to_vec();
                        for (bit, write_index) in segment.iter().enumerate() {
                            persisted[*write_index] = mask & (1 << bit) != 0;
                        }
                        persisted
                    })
                    .collect();
                let table = GivenStatesOfATable {
                    table_name: format!("masks={}..{}", slice.start, slice.end),
                    base: &recorded.base,
                    writes: &recorded.writes,
                    segments: &recorded.segments,
                    seams: &[],
                    persisted_sets,
                };
                let judged = engine
                    .judge_the_given_states(&table)
                    .unwrap_or_else(|reason| {
                        panic!("σ 那一片给定状态表 {} 判不了：{reason}", table.table_name)
                    });
                let evaluated = u64::try_from(judged.states.len()).expect("装得进 u64");
                let reported_missing = u64::try_from(
                    judged
                        .states
                        .iter()
                        .filter(|state| state.record_check().claimed_state_missing_unit)
                        .count(),
                )
                .expect("装得进 u64");
                if sender
                    .send((slice_index, evaluated, reported_missing))
                    .is_err()
                {
                    break;
                }
            });
        }
        drop(sender);
        let mut finished: std::collections::BTreeMap<usize, (u64, u64)> =
            std::collections::BTreeMap::new();
        let mut finished_states = 0u64;
        for (slice_index, evaluated, reported_missing) in receiver {
            finished_states += evaluated;
            finished.insert(slice_index, (evaluated, reported_missing));
            println!(
                "C561_SIGMA_PROGRESS slice={}/{} masks=[{},{}) finished_slices={}/{} finished_states={finished_states}/{state_count} elapsed_seconds={:.1}",
                slice_index + 1,
                slices.len(),
                slices[slice_index].start,
                slices[slice_index].end,
                finished.len(),
                slices.len(),
                started.elapsed().as_secs_f64()
            );
        }
        finished
    });
    assert_eq!(
        finished_by_slice.keys().copied().collect::<Vec<usize>>(),
        (0..slices.len()).collect::<Vec<usize>>(),
        "每一片都交回了：少了说明有工作线程领了片却没交回"
    );
    let run = engine.finish();
    if run.recorded_only() || run.stopped_early || run.left_tables_to_the_other_hosts() {
        println!(
            "{}",
            singlefs_harness::gpu_memory_ledger::gpu_not_run_line(
                item_name,
                &format!(
                    "σ 那几张给定状态表这一趟没判完（只录={}、叫停={}、归别的机器={}）：判过的在库里，判段那一趟接着判",
                    run.recorded_only(),
                    run.stopped_early,
                    run.tally.tables_left_to_the_other_hosts
                )
            )
        );
        return None;
    }
    assert_eq!(
        run.red_items(),
        Vec::<String>::new(),
        "GPU 判不了的状态照第三轮正文 H6 判红"
    );
    Some(SubsetsOfOneSegmentEnumerated {
        states: finished_by_slice
            .values()
            .map(|(evaluated, _)| evaluated)
            .sum(),
        closed_form_states: closed_form_state_count(&[segment.to_vec()]),
        slices: slices.len(),
        spawned_worker_threads,
        parallelism,
        reported_missing: finished_by_slice.values().map(|(_, missing)| missing).sum(),
        slices_reused: run.tally.tables_reused,
        elapsed_seconds: started.elapsed().as_secs_f64(),
    })
}

/// 计数行：`<前缀> states=… closed_form_states=… exhaustive=… record_claimed_state_missing_unit=…`。
/// `exhaustive=true` 只在评过的状态数等于闭式算的数时打（门禁 checker-tier-crash-point-replay 的 `exhaustive=` 判它，`scripts/gate/admission.py`）。
#[cfg(feature = "verdict-store")]
fn count_line(prefix: &str, enumerated: &SubsetsOfOneSegmentEnumerated) -> String {
    format!(
        "{prefix} states={} closed_form_states={} exhaustive={} record_claimed_state_missing_unit={}",
        enumerated.states,
        enumerated.closed_form_states,
        enumerated.states == enumerated.closed_form_states,
        enumerated.reported_missing
    )
}

/// 与崩溃点重放判段（`crash_state_enumeration`）打的那一行同形的线程行（门禁 checker-tier-crash-point-replay 的 `threads=` 按 `states=` 找它，
/// 读 `worker_threads=`、`slices=`、`resumed_slices=`、`freshly_run_slices=`，`admission.py` 的 `judge_worker_threads`）。
/// 这条用例经给定状态表那一类的判定库：复用的片是库里早先判过的，这一趟跑的片是其余的。
#[cfg(feature = "verdict-store")]
fn parallel_finished_line(enumerated: &SubsetsOfOneSegmentEnumerated) -> String {
    let worker_threads_source = enumerated.parallelism.worker_threads_source;
    let slices = u64::try_from(enumerated.slices).expect("片数装得进 u64");
    format!(
        "CRASH_POINT_REPLAY_PARALLEL_FINISHED states={} slices={} worker_threads={} configured_worker_threads={} worker_threads_source={worker_threads_source} resumed_slices={} freshly_run_slices={} progress_file_after_completion=none elapsed_seconds={:.1}",
        enumerated.states,
        enumerated.slices,
        enumerated.spawned_worker_threads,
        enumerated.parallelism.worker_threads,
        enumerated.slices_reused,
        slices - enumerated.slices_reused,
        enumerated.elapsed_seconds
    )
}

/// 一行里 `键=值` 那几段（与 `scripts/gate/admission.py` 的 `fields_of_line` 同一个切法：按空白切、第一个 `=` 分开）。
#[cfg(feature = "verdict-store")]
fn fields_of_line(line: &str) -> std::collections::BTreeMap<&str, &str> {
    line.split_whitespace()
        .filter_map(|token| token.split_once('='))
        .collect()
}

/// σ 之前整段持久、σ 与它之后的写一个都不落：σ 的全部子集拿它当底。
#[cfg(feature = "verdict-store")]
fn persisted_before_sigma(recorded: &RecordedHistory, sigma: usize) -> Vec<bool> {
    let mut before_sigma = persisted_before_the_units_of_sigma(recorded, sigma);
    for write_index in &recorded.segments[sigma] {
        before_sigma[*write_index] = false;
    }
    before_sigma
}

/// σ 那一段全部 2^16 个状态（σ 之前整段持久；C577 之前 σ 带上一次的轮换、2^18 个）：记录核对器一个都不判缺席。崩溃点重放规模第二轮攻方
/// 在冻结副本上量过按整份判的那一版在这里红 32768 个（那时 σ 18 个写；`research/prompts/m2-layer0-scale-r2-opus-output.md` 第一节，
/// 核查员复跑逐字一致）。每个状态一遍看 journal 的恢复加记录核对器，
/// 按掩码区间切成固定的几片、一片一张给定状态表经判定库两段判（线程数与崩溃点重放同一个来源）；打计数行 `C561_SIGMA_FULL`
/// （状态数等于闭式时带 `exhaustive=true`）与一行 `CRASH_POINT_REPLAY_PARALLEL_FINISHED` 同形的线程行（复用的片是库里早先判过的），
/// 门禁 checker-tier-crash-point-replay 按这两行判穷尽没有、线程用没用上。
#[cfg(feature = "verdict-store")]
#[test]
#[ignore = "崩溃枚举（65536 个状态、每个两遍恢复、池级 checker、记录核对器）：用户要求时由主 agent 经门禁 checker-tier-crash-point-replay --full 按输入哈希跑（release），提交时的快档只核它的全绿标记，平时不跑"]
fn every_crash_state_of_sigma_leaves_every_unit_of_the_claimed_publishes_present() {
    let recorded = record_history("c561-sigma-full", &HISTORY_UOOUOMSU);
    let chain = misaligned_reuse_chain(&recorded);
    let sigma_segment = recorded.segments[chain.sigma].clone();
    let Some(enumerated) = enumerate_every_subset_of_one_segment(
        &recorded,
        &sigma_segment,
        &persisted_before_sigma(&recorded, chain.sigma),
        WorkerThreadsOfTheRun::from_environment(),
        "c561-sigma-full",
    ) else {
        return;
    };
    println!("{}", parallel_finished_line(&enumerated));
    println!("{}", count_line("C561_SIGMA_FULL", &enumerated));
    assert_eq!(
        enumerated.states, 65_536,
        "σ 有 16 个写（C577 之前 18 个、262144）"
    );
    assert_eq!(
        enumerated.states, enumerated.closed_form_states,
        "σ 的每一个子集都评过"
    );
    assert_eq!(
        enumerated.reported_missing, 0,
        "σ 的任何一个子集落盘，自称的那几次发布的单元都不判缺席"
    );
}

/// σ 全量那两行的打法，拿同一条历史上 σ 前面一段只有几个写的小段核（全量本身标了 ignore，平时不跑）：
/// 评过的状态数等于闭式 2^|段| 时计数行带 `exhaustive=true`；线程行与崩溃点重放那一行同形、`states=` 与计数行相同，
/// 读回的片 + 这一趟跑的片 = 总片数、起了至少一个工作线程、不多于配的线程数与片数——门禁 checker-tier-crash-point-replay 按这几样判。
/// 两个工作线程与一个工作线程并出来的计数相同；后一趟的每一片都照库里那一趟的复用（经判定库，不在内存里现判）。
#[cfg(feature = "verdict-store")]
#[test]
fn the_sigma_enumeration_prints_exhaustive_true_and_a_thread_line_of_the_crash_point_replay_shape()
{
    let recorded = record_history("c561-sigma-lines", &HISTORY_UOOUOMSU);
    let chain = misaligned_reuse_chain(&recorded);
    let small_segment_index = (0..chain.sigma)
        .rev()
        .find(|segment_index| (2..=4).contains(&recorded.segments[*segment_index].len()))
        .expect("σ 之前有一段只有 2 到 4 个写");
    let small_segment = recorded.segments[small_segment_index].clone();
    let persisted_before = persisted_before_sigma(&recorded, small_segment_index);
    let on_threads = |worker_threads: usize| {
        enumerate_every_subset_of_one_segment(
            &recorded,
            &small_segment,
            &persisted_before,
            WorkerThreadsOfTheRun {
                worker_threads: std::num::NonZeroUsize::new(worker_threads).expect("至少 1 个"),
                worker_threads_source: "given_by_caller",
            },
            "c561-sigma-lines",
        )
    };
    let Some(on_two) = on_threads(2) else {
        return;
    };
    let Some(on_one) = on_threads(1) else {
        return;
    };
    assert_eq!(
        (on_two.states, on_two.reported_missing),
        (on_one.states, on_one.reported_missing),
        "线程数不改计数"
    );
    assert_eq!(
        on_one.slices_reused,
        u64::try_from(on_one.slices).expect("片数装得进 u64"),
        "后一趟的每一片都照库里前一趟判过的复用：片经判定库，不在内存里现判"
    );
    assert_eq!(on_two.states, 1u64 << small_segment.len());
    let counted = count_line("C561_SIGMA_LINES", &on_two);
    let count_fields = fields_of_line(&counted);
    assert_eq!(count_fields.get("exhaustive"), Some(&"true"), "{counted}");
    assert_eq!(
        count_fields.get("states"),
        Some(&on_two.states.to_string().as_str()),
        "{counted}"
    );
    let threads = parallel_finished_line(&on_two);
    assert!(
        threads.starts_with("CRASH_POINT_REPLAY_PARALLEL_FINISHED states="),
        "{threads}"
    );
    let thread_fields = fields_of_line(&threads);
    assert_eq!(
        thread_fields.get("states"),
        count_fields.get("states"),
        "{threads}"
    );
    let number = |key: &str| -> usize {
        thread_fields
            .get(key)
            .unwrap_or_else(|| panic!("线程行缺 {key}=：{threads}"))
            .parse()
            .unwrap_or_else(|error| panic!("线程行的 {key}= 不是整数（{error}）：{threads}"))
    };
    assert_eq!(
        number("resumed_slices") + number("freshly_run_slices"),
        number("slices"),
        "{threads}"
    );
    assert!(
        (1..=2.min(number("slices"))).contains(&number("worker_threads")),
        "{threads}"
    );
    let mut short_of_one_state = on_two;
    short_of_one_state.states -= 1;
    assert_eq!(
        fields_of_line(&count_line("C561_SIGMA_LINES", &short_of_one_state)).get("exhaustive"),
        Some(&"false"),
        "少评一个状态就不是全量"
    );
}

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 13 + seed) % 241).expect("小于 256"))
        .collect()
}

/// C513 那三个状态（复用窗口置 0 → 覆盖写 txg 4、txg 5，txg 5 的数据单元落回 txg 4 的那一对槽 50178，回收谓词过不了）：
/// txg 5 的单元落了、记录与根没落；除 txg 5 的根以外都落了；全部落了。三个状态上解释那一槽的后写（txg 5 的数据单元）都落了，
/// 但那次复用过不了回收谓词，txg 4 的数据单元照判缺席。
#[test]
fn an_illegal_reuse_does_not_explain_the_overwritten_unit_in_any_of_the_three_states() {
    let mut pool = build_pool("c561-illegal-reuse");
    pool.allocator.set_reuse_window(ReuseWindow::ForcedToZero);
    let first = pool.output.clone();
    let fourth = publish_overwrite_in_process(
        &mut pool,
        &first,
        &content_of(3100, 3),
        common::FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写 txg 4");
    let fifth = publish_overwrite_in_process(
        &mut pool,
        &fourth,
        &content_of(2900, 7),
        common::FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(1),
    )
    .expect("覆盖写 txg 5");
    let reused_slot = SlotNumber(50178);
    assert_eq!(
        (
            fourth.data_pointers[0].locations[0].slot,
            fifth.data_pointers[0].locations[0].slot
        ),
        (reused_slot, reused_slot),
        "复用窗口置 0：txg 5 的数据单元落回 txg 4 的那一对槽"
    );
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    let recorded = RecordedHistory {
        base: pool.memory_pool_after_mkfs(),
        writes,
        segments,
        versions: Vec::new(),
    };
    let root_indexes: Vec<usize> = recorded
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    let (fourth_root_index, fifth_root_index) = match root_indexes.as_slice() {
        [.., fourth_root_index, fifth_root_index] => (*fourth_root_index, *fifth_root_index),
        too_few => panic!("写表里至少有 txg 4、txg 5 的根槽写：{too_few:?}"),
    };
    let first_record_of_the_fifth_publish = (fourth_root_index + 1..fifth_root_index)
        .find(|index| recorded.writes[*index].kind == StepKind::JournalRecord)
        .expect("txg 5 写了 journal 记录");
    let write_count = recorded.writes.len();
    let three_states: [(&str, Vec<bool>); 3] = [
        (
            "txg 5 的单元落了、记录与根没落",
            (0..write_count)
                .map(|index| index < first_record_of_the_fifth_publish)
                .collect(),
        ),
        (
            "除 txg 5 的根以外都落了",
            (0..write_count)
                .map(|index| index < fifth_root_index)
                .collect(),
        ),
        ("全部落了", vec![true; write_count]),
    ];
    let engine = given_states_engine_of_this_test("illegal_reuse_three_states");
    let Some(judged) = judge_the_states(
        &engine,
        &recorded,
        "illegal-reuse-three-states",
        three_states
            .iter()
            .map(|(_, persisted)| persisted.clone())
            .collect(),
    ) else {
        return;
    };
    finish_the_engine(engine);
    for ((state_name, persisted), (effective_root, check)) in three_states.iter().zip(judged) {
        let image = CrashImage {
            base: &recorded.base,
            writes: &recorded.writes,
            persisted: persisted.clone(),
        };
        for (index, write) in recorded.writes.iter().enumerate() {
            if write.kind == StepKind::UnitWrite && write.offset == reused_slot.to_device_offset() {
                let length =
                    usize::try_from(write.length_in_bytes()).expect("单元写的长度装得进 usize");
                let on_disk = PoolReader::read(&image, write.device, write.offset, length)
                    .expect("50178 在两盘上都读得回来");
                assert_eq!(
                    write.contents.still_on_disk(&on_disk),
                    index > fourth_root_index,
                    "{state_name}：50178 上是 txg 5 那一份（写表下标 {index}）"
                );
            }
        }
        assert!(
            effective_root.is_some_and(|(_, txg)| txg >= CheckpointTxg(4)),
            "{state_name}：恢复自称的 txg ≥ 4（{effective_root:?}）"
        );
        assert!(
            check.claimed_state_missing_unit,
            "{state_name}：解释 50178 的那次后写落了、但过不了回收谓词，txg 4 的数据单元缺席（C513）"
        );
    }
}

/// σ 那次发布的写区间：单元写段起、到它根槽写所在的段末为止；紧跟的一段若只有系统配置槽轮换（没有单元写、没有根槽写），也归它。
#[cfg(feature = "verdict-store")]
fn sigma_publish_writes(recorded: &RecordedHistory) -> std::ops::Range<usize> {
    let chain = misaligned_reuse_chain(recorded);
    let sigma_first_write = *recorded.segments[chain.sigma]
        .first()
        .expect("σ 至少一个写");
    let sigma_root_write = (sigma_first_write..recorded.writes.len())
        .find(|index| recorded.writes[*index].kind == StepKind::RootRecordFua)
        .expect("σ 之后有根槽写");
    let segment_of = |write_index: usize| {
        recorded
            .segments
            .iter()
            .position(|segment| segment.contains(&write_index))
            .expect("每个写都在某一段里")
    };
    let mut last_segment_of_the_publish = segment_of(sigma_root_write);
    if let Some(next) = recorded.segments.get(last_segment_of_the_publish + 1) {
        if next.iter().all(|write_index| {
            recorded.writes[*write_index].kind == StepKind::SystemConfigurationSlot
        }) {
            last_segment_of_the_publish += 1;
        }
    }
    let end_of_the_publish = *recorded.segments[last_segment_of_the_publish]
        .last()
        .expect("段非空")
        + 1;
    sigma_first_write..end_of_the_publish
}

/// GPU 判器在 σ 那次发布上与 CPU 判器逐状态相同：这条历史把系统配置 0 号槽写了 34 次，判器表里一个落点的组合掩码要用到高 32 位
/// （它是今天唯一一条一个落点超过 32 次写的流）。只展开 σ 那次发布的段，其余段不展开；每个状态 GPU 的红项与 CPU 判器的
/// `red_items` 逐项同名、同序，GPU 一个状态都不许判不了。要一张空闲显存够的 NVIDIA 独显：本机没有 NVIDIA 卡时打一行
/// `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功，有卡而用不上判红（`gpu_unit_checks::gpu_test_start`）。
/// 卡经这个进程的卡池取，建好表之后按判器要的显存借（`crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool`），
/// 等满了借不到也报「本次未跑」。
#[cfg(all(feature = "gpu", feature = "verdict-store"))]
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_sigma_whose_configuration_slot_is_written_more_than_thirty_two_times(
) {
    use singlefs_checker_tier::crash::CrashPointReplaySegmentExpansion;
    use singlefs_checker_tier::crash_amplification::{
        gpu_recovery_words, gpu_verdict_items, recovery_words_of,
    };
    use singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
    use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
    use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    let recorded = record_history("gpu-judge-sigma", &HISTORY_UOOUOMSU);
    let sigma_publish = sigma_publish_writes(&recorded);
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment
            .iter()
            .all(|write_index| sigma_publish.contains(write_index))
        {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let judged_root_index = recorded
        .writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("历史末尾有根槽写");
    let tables = build_judge_tables(&JudgeFlow {
        base: &recorded.base,
        writes: &recorded.writes,
        segments: &recorded.segments,
        judged_root_index,
        versions: &recorded.versions,
        expansion: &expansion,
    })
    .expect("一个落点 64 次写以内，建得出表");
    let most_writes_on_one_location = tables
        .locations
        .iter()
        .map(|location| location.writes.len())
        .max()
        .expect("至少一个落点");
    assert!(
        most_writes_on_one_location > 32,
        "这条对拍要的是一个落点超过 32 次写的流，今天最多 {most_writes_on_one_location} 次"
    );
    // 高 32 位真起作用：有两个版本低 32 位相同、只差高位（只比低 32 位的内核会把它们认成同一个）
    let versions_told_apart_only_by_the_high_bits: usize = tables
        .locations
        .iter()
        .map(|location| {
            let mut by_low: std::collections::BTreeMap<u32, usize> =
                std::collections::BTreeMap::new();
            for version in &location.versions {
                *by_low
                    .entry(u32::try_from(version.combination & 0xFFFF_FFFF).expect("低 32 位"))
                    .or_insert(0) += 1;
            }
            by_low.values().filter(|count| **count > 1).count()
        })
        .sum();
    assert!(
        versions_told_apart_only_by_the_high_bits > 0,
        "这条对拍要有两个版本低 32 位相同、只差高位，不然只比低 32 位的内核也判得对"
    );
    const TEST_NAME: &str = "the_gpu_judge_agrees_with_the_cpu_judge_on_sigma_whose_configuration_slot_is_written_more_than_thirty_two_times";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(gpu) = judge_every_state_on_a_card_leased_from_the_pool(cards, TEST_NAME, &tables)
    else {
        return;
    };
    // CPU 判器按区间切片并行判（每个状态十几到几十毫秒，单线程判 6.5 万态要一个多小时）：线程数从
    // SINGLEFS_CRASH_JUDGE_CPU_THREADS 取，没设取每台默认重活线程数；每线程 8 片，按片序拼回，与单线程逐状态相同
    let threads = cpu_threads_from_the_value_of_the_variable(
        singlefs_harness::numeric_environment_variable::process_environment_variable(
            CPU_THREADS_ENVIRONMENT_VARIABLE,
        )
        .as_deref(),
        singlefs_harness::numeric_environment_variable::process_environment_variable(
            singlefs_harness::heavy_threads::THREAD_CAP_ENVIRONMENT_VARIABLE,
        )
        .as_deref(),
    )
    .get();
    let slice_count = u64::try_from(threads * 8).expect("装得进 u64");
    let slice_length = tables.state_count.div_ceil(slice_count).max(1);
    let slices: Vec<std::ops::Range<u64>> = (0..slice_count)
        .map(|index| {
            (index * slice_length).min(tables.state_count)
                ..((index + 1) * slice_length).min(tables.state_count)
        })
        .filter(|range| !range.is_empty())
        .collect();
    let next_slice = std::sync::atomic::AtomicUsize::new(0);
    /// 一个状态的 CPU 判定：红项与落到的根、两遍的结局（[`recovery_words_of`] 的四个数）。
    type CpuVerdict = (Vec<String>, [u32; 4]);
    let mut judged_slices: Vec<(usize, Vec<CpuVerdict>)> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut mine = Vec::new();
                    loop {
                        let index = next_slice.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        let Some(range) = slices.get(index) else {
                            break;
                        };
                        let mut items = Vec::new();
                        let Some(state_count) = sample_crash_states_on_the_cpu(
                            CrashPointReplayItem {
                                name: "record_checker::sigma_gpu_pairing_cpu_side",
                                setting_file: THIS_FILE,
                            },
                            &recorded.base,
                            &recorded.writes,
                            &recorded.segments,
                            judged_root_index,
                            &recorded.versions,
                            &expansion,
                            range.clone(),
                            &mut |_ordinal, _image, judgement| {
                                items.push((judgement.red_items(), recovery_words_of(judgement)));
                            },
                        )
                        .judged() else {
                            // 只录那一趟：流写进了判定库，这条线程不再领片
                            break;
                        };
                        assert_eq!(
                            state_count, tables.state_count,
                            "CPU 判器与判器的表数的状态总数相同"
                        );
                        assert_eq!(
                            u64::try_from(items.len()).expect("装得进 u64"),
                            range.end - range.start,
                            "这一片的状态都判了"
                        );
                        mine.push((index, items));
                    }
                    mine
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|worker| worker.join().expect("判器线程不 panic"))
            .collect()
    });
    if !singlefs_checker_tier::crash_library::RunStage::of_this_process().judges() {
        return;
    }
    judged_slices.sort_by_key(|(index, _)| *index);
    let cpu: Vec<CpuVerdict> = judged_slices
        .into_iter()
        .flat_map(|(_, items)| items)
        .collect();
    let states = u64::try_from(cpu.len()).expect("装得进 u64");
    assert_eq!(states, tables.state_count, "CPU 判器判了全部状态");
    // 红项与落到的根、两遍的结局都要相同：这条流全绿，判器读错一个落点的版本时红项照样为空，只露在落到的根上
    let mut mismatches = Vec::new();
    for (ordinal, (gpu_words, (cpu_items, cpu_recovery))) in gpu.iter().zip(&cpu).enumerate() {
        let gpu_items = gpu_verdict_items(gpu_words);
        let gpu_recovery = gpu_recovery_words(gpu_words);
        if (gpu_items.as_ref() != Some(cpu_items) || gpu_recovery != *cpu_recovery)
            && mismatches.len() < 5
        {
            mismatches.push(format!(
                "状态 {ordinal}：GPU {gpu_items:?} {gpu_recovery:?}，CPU {cpu_items:?} {cpu_recovery:?}"
            ));
        }
    }
    println!(
        "GPU_JUDGE_SIGMA states={} most_writes_on_one_location={most_writes_on_one_location} red_states={} mismatches={}",
        tables.state_count,
        cpu.iter().filter(|(items, _)| !items.is_empty()).count(),
        mismatches.len()
    );
    assert!(
        mismatches.is_empty(),
        "GPU 与 CPU 判器在 σ 上有不同：{mismatches:?}"
    );
}

/// σ 接进崩溃放量流水线（用户 2026-09-29 定：崩溃注入之外的崩溃枚举用例都进流水线、由 GPU 判）：整条 `UOOUOMSU` 历史一张写表；
/// σ 之前的段与 σ 那次发布之后的段各包成一个 ignore 的点（只留全持久那一个状态：是剪枝，也是别的流可复用的前缀），
/// σ 那次发布（单元写段起、到它的根槽写与系统配置轮换为止）一个点全部展开。判器是流水线的：两遍恢复、oracle、池级 checker、
/// 记录核对器，比 `every_crash_state_of_sigma_leaves_every_unit_of_the_claimed_publishes_present` 多判 oracle 与 checker。
/// 展开上限从环境取（`SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO`，没设 6：σ 16 个写平时不展开，全量脚本给 28 才展开）。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_of_sigma_on_the_misaligned_reuse_history_is_clean() {
    use singlefs_checker_tier::crash_amplification::{repository_relative_file, CrashPointSpan};
    let recorded = record_history("crash-amplification-sigma", &HISTORY_UOOUOMSU);
    let sigma_publish = sigma_publish_writes(&recorded);
    let (sigma_first_write, end_of_the_publish) = (sigma_publish.start, sigma_publish.end);
    let code_file = repository_relative_file(file!());
    let mut crash_points = vec![
        CrashPointSpan {
            name: "history_before_sigma".to_string(),
            code_file_in_repository: code_file.clone(),
            writes: 0..sigma_first_write,
            ignored: true,
        },
        CrashPointSpan {
            name: "sigma_publish".to_string(),
            code_file_in_repository: code_file.clone(),
            writes: sigma_first_write..end_of_the_publish,
            ignored: false,
        },
    ];
    if end_of_the_publish < recorded.writes.len() {
        crash_points.push(CrashPointSpan {
            name: "history_after_sigma".to_string(),
            code_file_in_repository: code_file,
            writes: end_of_the_publish..recorded.writes.len(),
            ignored: true,
        });
    }
    let judged_root_index = recorded
        .writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("历史末尾有根槽写");
    let run = common_crash_points::amplify_prepared_flow(
        "sigma-of-the-misaligned-reuse",
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        judged_root_index,
        &recorded.versions,
        crash_points,
        6,
    );
    assert!(
        run.recording.states_recorded + run.recording.states_already_judged >= 3,
        "三个点至少各一个状态"
    );
}
