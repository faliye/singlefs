//! 虚机档（QEMU/KVM）：在两块真 virtio 盘上跑第一个事务的整条写路，冷重开再恢复读回文件。
//!
//!   first_transaction_on_device /dev/vda /dev/vdb <direct | page-cache | skip-first-transaction-barrier | second-transaction | second-instance | raise-rollback-floor>
//!
//! 由 `research/scripts/vm-bench.sh` 送进虚机（`VM_DISKS=2`，设备路径排在参数前面）。结果行以 `E7RESULT` 打头、
//! 末行报条数（vm-bench.sh 的完整性闸）。这个二进制只判它自己判得了的（恢复读回文件、段序列）；
//! 「盘上实际收到的是不是程序以为的」由宿主上的 `first_transaction_device_log_check` 拿 QEMU 的设备侧日志判。
//! `skip-first-transaction-barrier`：第一个事务在盘 0 上漏掉「单元 → journal 记录」那道屏障，而录制器照样记下它——
//! 程序以为发了，盘上没收到，宿主那一侧必须判红（C6（块层语义假设写错） 的「故意去掉一次屏障」）。
//! `second-transaction`：第一个事务之后同一个进程里再覆盖写一次（里程碑「第二个事务」步 1 的发布 B），分别报两次发布的挂钟与
//! 程序交给设备的写请求数、字节、屏障、FUA——E152（按里程碑对比六家文件系统的文件性能） 里程碑二那一轮的 singlefs 臂用它量「第二次写」的稳态代价；
//! 冷重开读回的是第二版。
//! `second-instance`：发布 B 之后丢掉写的那一套句柄、同一对盘冷重开，走可写挂载（恢复、取号、写行、暖机，里程碑「第二个事务」步 3），
//! 再发布一次（发布 C）；挂载一行、发布 C 一行，写行与挂载里的每次暖机、发布 C 各一行 `name=publish_writes`，挂载与发布 C 各一段窗口行；
//! 冷重开读回的是第三版（实例 2 的根）。前四个模式打的行一行不变。
//! `raise-rollback-floor`：`second-instance` 那条路走完，同一次挂载里再覆盖写一次（发布 D，第 4 新的非空有效根落到 A 上），接着把 F 抬到上限
//! （里程碑「第二个事务」增补 2 收口表第 58 行：抬 F 那一串发布的账在二进制这一侧与设备一层判相等）；发布 D 与发布 C 同样打一行、一行
//! `name=publish_writes`、一段窗口行，抬 F 一行、那一串空发布各一行 `name=publish_writes`、这一段一行窗口行；失败时照发布失败那样打失败账。
//! 冷重开读回的是第四版（最后一次抬 F 的空发布的根）；`name=recover_cold` 之后多打一行 `name=recover_cold_rollback_floor`：
//! 冷重开择到的根与它从盘上读回的 F（`rollback_floor_on_disk=`，代码轮第二轮判决 Z10）。前五个模式打的行一行不变。
//! 每次发布（两次暖机、第一个事务、`second-transaction` 模式下的发布 B）各打一行 `name=publish_writes`：写入口按结构种类记的写调用数与写字节
//! （里程碑「第二个事务」增补 1 第 1 件）；每段窗口（两次暖机合一段、第一个事务、发布 B）再打一行 `name=publish_writes_against_device`，
//! 按种类的合计与设备一层数的（`FaultInjectingDevice`，两盘相加）逐项比，对不上 `matches=false`、退出码 1。
//! 发布失败时不直接退出（里程碑「第二个事务」增补 2 收口表第 58 行）：写入口交出来的失败账一份一行 `name=failed_publish_writes`，
//! 再把它与这段窗口里设备一层数到的写比一行，打完 `name=run_failed` 与末行条数再以退出码 5 退出。
//! 模式名、两版内容与写入时刻、发布 B 与发布 C 的写路在 `singlefs_harness::on_device_modes`，宿主上的
//! `first_transaction_device_log_check` 跑同一份，逐项比设备侧日志。

use std::path::Path;

use singlefs_core::address::DeviceIdentity;
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::{
    probe_queue_number_under, probe_queue_text_under, BlockDevice, DirectInputOutputBlockDevice,
    PageCachePolicy, PhysicalBlockSizeSource, SYSFS_CLASS_BLOCK,
};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{mount_writable, MountError, Mounted, PublishSequenceFailed};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryOutcome,
};
use singlefs_core::transaction::{PoolVersion, TransactionOutput};
use singlefs_core::write_accounting::{
    WriteCallsAndBytes, WritesByStructureKind, WrittenStructureKind,
};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceCounts, FaultDeviceSelector, FaultInjectingBlockDevice,
    FaultOccurrence, FaultPlacement, FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::on_device_modes::{
    allocator_rebuilt_from_the_records_of, publish_the_fourth_version, publish_the_second_version,
    publish_the_third_version, raise_the_rollback_floor_to_its_ceiling, FailedPublish,
    OnDeviceRunMode, PublishesAfterTheFirstTransaction,
};
use singlefs_harness::scenario::{
    e142_parameters, run_first_transaction, FirstTransactionPathFailure, FirstTransactionPathStep,
    ScenarioPoint,
};
use singlefs_harness::segments::{
    closed_form_state_count, segment_kinds_text, segment_sizes_text, split_into_segments,
    FixedGeometry,
};
use singlefs_harness::{RecordedOperation, RecordingBlockDevice, SharedStream};
use std::time::Instant;

/// 这一档该报哪几段分段时间，按次序登记。跑批脚本与门禁按这张表点数；
/// 段丢了一条、次序反了，`name=segment_timing_registration` 那一行的 `matches` 就是 false、这一档退非 0。
fn registered_segments_of(mode: OnDeviceRunMode) -> &'static [&'static str] {
    match mode {
        OnDeviceRunMode::Direct
        | OnDeviceRunMode::PageCache
        | OnDeviceRunMode::SkipFirstTransactionBarrier => &[
            "mkfs",
            "instance_acquisition",
            "warm_up",
            "first_transaction",
            "cold_reopen_and_recover",
        ],
        OnDeviceRunMode::SecondTransaction => &[
            "mkfs",
            "instance_acquisition",
            "warm_up",
            "first_transaction",
            "second_transaction",
            "cold_reopen_and_recover",
        ],
        OnDeviceRunMode::SecondInstance => &[
            "mkfs",
            "instance_acquisition",
            "warm_up",
            "first_transaction",
            "second_transaction",
            "reopen_and_writable_mount",
            "third_transaction",
            "cold_reopen_and_recover",
        ],
        OnDeviceRunMode::RaiseRollbackFloor => &[
            "mkfs",
            "instance_acquisition",
            "warm_up",
            "first_transaction",
            "second_transaction",
            "reopen_and_writable_mount",
            "third_transaction",
            "fourth_transaction",
            "raise_rollback_floor",
            "cold_reopen_and_recover",
        ],
    }
}

/// 一块盘上程序交给设备的调用计数快照，两次快照相减就是一次发布的代价。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DeviceCallCounts {
    write_calls: u64,
    written_bytes: u64,
    force_unit_access_writes: u64,
    barrier_calls: u64,
}

impl DeviceCallCounts {
    /// 从共用的注入计划里取这块盘此刻的计数（`FaultDeviceCounts` 的六项里，结果行只报这四项）。
    fn of(counts: FaultDeviceCounts) -> Self {
        Self {
            write_calls: counts.writes,
            written_bytes: counts.written_bytes,
            force_unit_access_writes: counts.force_unit_access_writes,
            barrier_calls: counts.barriers_forwarded,
        }
    }

    fn since(self, earlier: Self) -> Self {
        Self {
            write_calls: self.write_calls - earlier.write_calls,
            written_bytes: self.written_bytes - earlier.written_bytes,
            force_unit_access_writes: self.force_unit_access_writes
                - earlier.force_unit_access_writes,
            barrier_calls: self.barrier_calls - earlier.barrier_calls,
        }
    }

    fn describe(self, index: usize) -> String {
        format!(
            "device_{index}_writes={} device_{index}_written_bytes={} device_{index}_force_unit_access_writes={} device_{index}_barriers={}",
            self.write_calls, self.written_bytes, self.force_unit_access_writes, self.barrier_calls
        )
    }
}

/// 两次快照之间整个池（每块盘相加）交给设备的写调用与字节。
fn pool_writes_between(
    earlier: &[DeviceCallCounts],
    later: &[DeviceCallCounts],
) -> WriteCallsAndBytes {
    assert_eq!(earlier.len(), later.len(), "两次快照拍的是同一组盘");
    earlier.iter().zip(later).fold(
        WriteCallsAndBytes::NONE,
        |sum, (earlier_device, later_device)| {
            let difference = later_device.since(*earlier_device);
            sum.plus(WriteCallsAndBytes {
                write_calls: difference.write_calls,
                written_bytes: difference.written_bytes,
            })
        },
    )
}

/// 一次发布的 `name=publish_writes` 行。
fn describe_publish_writes(publish: &str, txg: u64, writes: &WritesByStructureKind) -> String {
    format!(
        "name=publish_writes publish={publish} txg={txg} {}",
        describe_writes_by_kind(writes)
    )
}

/// 一份按种类的账在结果行里的那一段：合计取 `total()`（从记过的每一笔加），各种按 `IN_REPORT_ORDER` 逐个列、没写过的列 0。
fn describe_writes_by_kind(writes: &WritesByStructureKind) -> String {
    let total = writes.total();
    let per_kind: String = WrittenStructureKind::IN_REPORT_ORDER
        .iter()
        .map(|kind| {
            let kind_writes = writes.of(*kind);
            format!(
                " {name}_write_calls={} {name}_written_bytes={}",
                kind_writes.write_calls,
                kind_writes.written_bytes,
                name = kind.report_name()
            )
        })
        .collect();
    format!(
        "write_calls={} written_bytes={}{per_kind}",
        total.write_calls, total.written_bytes
    )
}

/// 一段窗口里几次发布按种类记的合计与设备一层数的比：返回结果行与相不相等。
fn publish_writes_against_device(
    window: &str,
    publishes: &[&WritesByStructureKind],
    device: WriteCallsAndBytes,
) -> (String, bool) {
    let by_kind = publishes
        .iter()
        .fold(WriteCallsAndBytes::NONE, |sum, writes| {
            sum.plus(writes.total())
        });
    let matches = by_kind == device;
    (
        format!(
            "name=publish_writes_against_device window={window} publishes={} by_kind_write_calls={} by_kind_written_bytes={} device_write_calls={} device_written_bytes={} matches={matches}",
            publishes.len(),
            by_kind.write_calls,
            by_kind.written_bytes,
            device.write_calls,
            device.written_bytes
        ),
        matches,
    )
}

/// 一段窗口里的发布失败了：要打的结果行与判定（增补 2 收口表第 58 行）。写入口交得出的账有两样：这一段里失败之前已经落盘的发布
/// 各自的写（暖机两次空发布里的第一次、可写挂载里的写行与前几次暖机；单次发布的窗口里是空的），一份一行
/// `name=persisted_publish_writes`；落盘阶段失败的那一次已记的写，一份一行 `name=failed_publish_writes`。两样相加再与设备一层数到的
/// 比一行 `name=publish_writes_against_device window=<窗口>_failed`（与成功路径同一个比法）。末行 `name=run_failed` 带失败原因。
fn describe_failed_window(
    window: &str,
    cause: &str,
    writes_of_persisted_publishes: &[WritesByStructureKind],
    writes_of_failed_publishes: &[WritesByStructureKind],
    device: WriteCallsAndBytes,
) -> (Vec<String>, bool) {
    let mut lines: Vec<String> = writes_of_persisted_publishes
        .iter()
        .enumerate()
        .map(|(index, writes)| {
            format!(
                "name=persisted_publish_writes window={window} publish={} {}",
                index + 1,
                describe_writes_by_kind(writes)
            )
        })
        .collect();
    lines.extend(
        writes_of_failed_publishes
            .iter()
            .enumerate()
            .map(|(index, writes)| {
                format!(
                    "name=failed_publish_writes window={window} failure={} {}",
                    index + 1,
                    describe_writes_by_kind(writes)
                )
            }),
    );
    let failed_publishes: Vec<&WritesByStructureKind> = writes_of_failed_publishes.iter().collect();
    let publishes_in_the_window: Vec<&WritesByStructureKind> = writes_of_persisted_publishes
        .iter()
        .chain(failed_publishes)
        .collect();
    let (line, matches) = publish_writes_against_device(
        &format!("{window}_failed"),
        &publishes_in_the_window,
        device,
    );
    lines.push(line);
    lines.push(describe_run_failure(window, cause));
    (lines, matches)
}

/// 整条路停下的那一行：停在哪一段、什么错（错的 Debug 文本去掉空格，一行只有 `key=value`）。
fn describe_run_failure(step: &str, cause: &str) -> String {
    format!(
        "name=run_failed step={step} cause={}",
        cause.replace(' ', "")
    )
}

/// 第一个事务那条路（mkfs → 取号 → 暖机 → 第一个事务）停下时要打的结果行。`counts_at_failure` 是停下那一刻每块盘的计数；
/// 两段发布窗口的起点是 `run_first_transaction` 在 `ScenarioPoint` 上叫回来时拍的快照（没走到那一处是 None）。
fn describe_first_transaction_path_failure(
    failure: &FirstTransactionPathFailure,
    counts_after_acquisition: Option<&[DeviceCallCounts]>,
    counts_before_first_transaction: Option<&[DeviceCallCounts]>,
    counts_at_failure: &[DeviceCallCounts],
) -> Vec<String> {
    let step = failure.failed_step.name();
    match failure.failed_step {
        FirstTransactionPathStep::MakeFilesystem
        | FirstTransactionPathStep::InstanceAcquisition => {
            vec![describe_run_failure(step, &failure.cause)]
        }
        FirstTransactionPathStep::WarmUp => {
            describe_failed_window(
                step,
                &failure.cause,
                &failure.writes_of_persisted_publishes,
                &failure.writes_of_failed_publishes,
                pool_writes_between(
                    counts_after_acquisition
                        .expect("run_first_transaction 取号之后、暖机之前叫过取号之后那一处"),
                    counts_at_failure,
                ),
            )
            .0
        }
        FirstTransactionPathStep::FirstTransaction => {
            describe_failed_window(
                step,
                &failure.cause,
                &failure.writes_of_persisted_publishes,
                &failure.writes_of_failed_publishes,
                pool_writes_between(
                    counts_before_first_transaction
                        .expect("run_first_transaction 第一个事务之前叫过那一处"),
                    counts_at_failure,
                ),
            )
            .0
        }
    }
}

/// 整条路在某一段停下：到那一刻为止要打的结果行（含失败那几行）与给人看的一句原因。
#[derive(Debug)]
struct FailedRun {
    lines: Vec<String>,
    cause: String,
}

/// 停下之后：结果行照打、末行报条数（跑批脚本的完整性闸照样点得清条数），再以退出码 5 退出。
fn exit_after_a_failed_run(emitter: &mut Emitter, failed: &FailedRun) -> ! {
    for line in &failed.lines {
        emitter.emit(line);
    }
    emitter.finish();
    eprintln!("写路径失败：{}", failed.cause);
    std::process::exit(5)
}

/// 只吞掉这块盘接下来的第一道屏障（虚机档的「漏一道屏障」）：通用的故障注入包装（增补 3 第 4 件，
/// `singlefs_harness::fault_injection`；这里原先手写的 `FaultInjectingDevice` 2026-09-21 并进了它）。
fn swallow_the_next_barrier_on(device: DeviceIdentity) -> FaultSchedule {
    FaultSchedule {
        fault: InjectedFault::BarrierIsSwallowed,
        device: FaultDeviceSelector::OnlyDevice(device),
        placement: FaultPlacement::AnyOffset,
        counting: FaultCounting::PerDevice,
        occurrence: FaultOccurrence::TheNthMatchingCall(1),
    }
}

/// 分段挂钟：**在被测进程里计**，不许让外层按结果行到达时刻切
/// （`.claude/singlefs-ai-sop/rules/test-discipline.md`「分段计时在被测进程里计，不在转发输出的循环里计」——
/// 转打可能阻塞，虚机的串口尤其慢，按到达时刻切出来的段会带着前面几行的打印积压）。
///
/// 每段末尾叫一次 [`SegmentClock::mark`]，它交回那一段的挂钟与「从进程里第一次取表到这一刻」的单调读数；
/// 整条路跑完 [`SegmentClock::containment_check`] 报一次包含自检：各段之和不超过整条路的挂钟。
/// 各段首尾相接，所以两者本该只差取表本身的开销——差出来就是漏了一段或者某一段重复计了。
struct SegmentClock {
    started: Instant,
    previous_mark: Instant,
    segments: Vec<(String, u128)>,
}

impl SegmentClock {
    fn start() -> Self {
        let now = Instant::now();
        Self {
            started: now,
            previous_mark: now,
            segments: Vec::new(),
        }
    }
    /// 这一段结束：交回要打的结果行。`nanoseconds` 是这一段自己的挂钟，
    /// `monotonic_nanoseconds` 是同一个单调时钟从整条路开头算起的读数（跑批脚本按它对齐各段）。
    fn mark(&mut self, segment: &str) -> String {
        let now = Instant::now();
        let nanoseconds = now.duration_since(self.previous_mark).as_nanos();
        let since_start = now.duration_since(self.started).as_nanos();
        self.previous_mark = now;
        self.segments.push((segment.to_string(), nanoseconds));
        format!(
            "name=segment_timing segment={segment} nanoseconds={nanoseconds} monotonic_nanoseconds={since_start}"
        )
    }
    /// 段名按打标记的次序。
    fn marked_segments(&self) -> Vec<&str> {
        self.segments
            .iter()
            .map(|(name, _)| name.as_str())
            .collect()
    }
    /// 登记对账：这一档该打哪几段是写死登记的（[`registered_segments_of`]），真打出来的要与它逐项相同。
    /// 少打一段、多打一段、次序变了都判红——分段时间是拿去报中位与离散的，段丢了一条没人看得出来。
    fn registration_check(&self, mode_name: &str, registered: &[&str]) -> (String, bool) {
        let marked = self.marked_segments();
        let matches = marked == registered;
        (
            format!(
                "name=segment_timing_registration mode={mode_name} registered={} marked={} matches={matches}",
                registered.join(","),
                marked.join(",")
            ),
            matches,
        )
    }
    /// 包含自检：各段之和 ≤ 整条路的挂钟。交回结果行与判定。
    fn containment_check(&self) -> (String, bool) {
        let whole = self.started.elapsed().as_nanos();
        let sum: u128 = self.segments.iter().map(|(_, value)| *value).sum();
        let contained = sum <= whole;
        (
            format!(
                "name=segment_timing_check segments={} sum_nanoseconds={sum} whole_run_nanoseconds={whole} contained={contained}",
                self.segments.len()
            ),
            contained,
        )
    }
}

struct Emitter {
    emitted: u64,
}

impl Emitter {
    fn emit(&mut self, body: &str) {
        self.emitted += 1;
        println!("E7RESULT {body}");
    }
    fn finish(&mut self) {
        self.emitted += 1;
        println!("E7RESULT name=done emitted={}", self.emitted);
    }
}

/// 发布 B 之后那几段（可写挂载、发布 C、抬 F）攒下的结果行：分段时间那几行是在那几个函数里取的表，挑出来并进分段行
/// （跑完一起打），别的结果行照旧按次序打。
fn emit_the_lines_after_the_second_version(
    emitter: &mut Emitter,
    segment_timing_lines: &mut Vec<String>,
    lines: &[String],
) {
    for line in lines {
        if line.starts_with("name=segment_timing ") {
            segment_timing_lines.push(line.clone());
        } else {
            emitter.emit(line);
        }
    }
}

/// 块层计数（`/sys/class/block/<名>/stat`，内核 `Documentation/block/stat.rst` 的字段序）：写请求数（下标 4）、写扇区数（下标 6）、
/// FLUSH 请求数（下标 15，5.5 起才有，老内核没有就是 None）。
fn block_layer_counters(device_path: &str) -> Option<(u64, u64, Option<u64>)> {
    let name = Path::new(device_path).file_name()?.to_str()?;
    let text =
        std::fs::read_to_string(Path::new(SYSFS_CLASS_BLOCK).join(name).join("stat")).ok()?;
    let fields: Vec<u64> = text
        .split_whitespace()
        .filter_map(|field| field.parse().ok())
        .collect();
    if fields.len() < 8 {
        return None;
    }
    Some((fields[4], fields[6], fields.get(15).copied()))
}

fn describe_counters(
    before: Option<(u64, u64, Option<u64>)>,
    after: Option<(u64, u64, Option<u64>)>,
) -> String {
    match (before, after) {
        (
            Some((write_ios_before, sectors_before, flush_before)),
            Some((write_ios_after, sectors_after, flush_after)),
        ) => {
            let flushes = match (flush_before, flush_after) {
                (Some(start), Some(end)) => (end - start).to_string(),
                (_, _) => "NA".to_string(),
            };
            format!(
                "write_ios={} write_sectors={} flush_ios={flushes}",
                write_ios_after - write_ios_before,
                sectors_after - sectors_before
            )
        }
        (_, _) => "write_ios=NA write_sectors=NA flush_ios=NA".to_string(),
    }
}

fn open(path: &str, policy: PageCachePolicy) -> DirectInputOutputBlockDevice {
    DirectInputOutputBlockDevice::open_existing(
        Path::new(path),
        policy,
        PhysicalBlockSizeSource::ProbeSysfs,
    )
    .unwrap_or_else(|error| {
        eprintln!("打不开 {path}：{error}");
        std::process::exit(3)
    })
}

/// 录制器外面那一层数调用、按需注入的盘。
type CountedDevice<Inner> = FaultInjectingBlockDevice<RecordingBlockDevice<Inner>>;

/// 每块盘此刻的计数快照，按 `devices` 的次序。
fn device_call_counts<Inner: BlockDevice>(
    devices: &[(DeviceIdentity, CountedDevice<Inner>)],
    plan: &SharedFaultPlan,
) -> Vec<DeviceCallCounts> {
    devices
        .iter()
        .map(|(identity, _)| DeviceCallCounts::of(plan.counts_of_device(*identity)))
        .collect()
}

/// 一次发布按结构种类记的写：带文件的一版与树表 0 条的零单元发布各记在自己的输出里。
fn writes_of(version: &PoolVersion) -> &WritesByStructureKind {
    match version {
        PoolVersion::WithoutFile(version_without_file) => &version_without_file.writes,
        PoolVersion::WithFile(file_version) => &file_version.writes,
    }
}

fn root_text(version: &PoolVersion) -> String {
    format!(
        "{}:{}",
        version.root().instance.0,
        version.root().checkpoint_txg.0
    )
}

/// 发布 B 那一段要打的结果行，以及这一段按种类的合计与设备一层是否相等。
struct SecondTransactionRun {
    lines: Vec<String>,
    window_matches_device: bool,
}

/// 发布 B：同一个进程里再覆盖写一次，分配器从第一个事务的分配记录重建（与可写挂载同一条路）；两次快照之差就是这一次发布
/// 交给设备的调用数，挂钟只计发布本身。
///
/// # Errors
/// 发布 B 失败：交回失败账那几行与一句原因（增补 2 收口表第 58 行）。
fn publish_the_second_version_and_describe<Inner: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, CountedDevice<Inner>)],
    plan: &SharedFaultPlan,
    stream: &SharedStream,
    geometry: &FixedGeometry,
    first_version: &TransactionOutput,
) -> Result<SecondTransactionRun, FailedRun> {
    let before = device_call_counts(devices, plan);
    let operations_before = stream.operations().len();
    let mut allocator = allocator_rebuilt_from_the_records_of(devices, first_version);
    let started = Instant::now();
    let published = publish_the_second_version(parameters, devices, &mut allocator, first_version);
    let nanoseconds = started.elapsed().as_nanos();
    let after = device_call_counts(devices, plan);
    let second = match published {
        Ok(second) => second,
        Err(failed) => {
            let cause = format!("{:?}", failed.cause);
            let (lines, _) = describe_failed_window(
                "second_transaction",
                &cause,
                &[],
                &failed.writes_of_failed_publishes,
                pool_writes_between(&before, &after),
            );
            return Err(FailedRun {
                lines,
                cause: format!("第二个事务：{cause}"),
            });
        }
    };
    let after_operations = stream.operations();
    let second_segments = split_into_segments(&after_operations[operations_before..], geometry);
    let per_device: Vec<String> = after
        .iter()
        .enumerate()
        .map(|(index, later)| later.since(before[index]).describe(index))
        .collect();
    let mut lines = vec![
        format!(
            "name=second_transaction root_txg={} transaction={} released={} nanoseconds={nanoseconds} operations={} segments={} closed_form={} {}",
            second.root.checkpoint_txg.0,
            second.record.transaction,
            second.released.len(),
            after_operations.len() - operations_before,
            segment_sizes_text(&second_segments),
            closed_form_state_count(&second_segments),
            per_device.join(" ")
        ),
        describe_publish_writes(
            "second_transaction",
            second.root.checkpoint_txg.0,
            &second.writes,
        ),
    ];
    let (second_transaction_line, second_transaction_matches) = publish_writes_against_device(
        "second_transaction",
        &[&second.writes],
        pool_writes_between(&before, &after),
    );
    lines.push(second_transaction_line);
    Ok(SecondTransactionRun {
        lines,
        window_matches_device: second_transaction_matches,
    })
}

/// `second-instance` 模式在发布 B 之后写出的东西：重开之后的那一套句柄（冷恢复之前要丢掉）、挂载交回又被发布 C 推进过的分配器、
/// 现行那一版（发布 C；`raise-rollback-floor` 抬完 F 之后是最后一次抬 F 的空发布）、重开之后这一段的注入计划、按次序要打的结果行、
/// 每段窗口按种类的合计与设备一层是否都相等。
struct SecondInstanceRun<Inner: BlockDevice> {
    devices: Vec<(DeviceIdentity, CountedDevice<Inner>)>,
    allocator: PoolAllocator,
    current_version: TransactionOutput,
    plan: SharedFaultPlan,
    lines: Vec<String>,
    every_window_matches_device: bool,
}

/// `second-instance` 模式重开、可写挂载之后的样子：重开的那一套句柄、挂载交回的分配器与现行那一版、这一段的注入计划、
/// 挂载返回那一刻每块盘的计数（发布 C 那段窗口从这里算起）、按次序要打的结果行、挂载窗口按种类的合计与设备一层是否相等。
struct WritableMountRun<Inner: BlockDevice> {
    devices: Vec<(DeviceIdentity, CountedDevice<Inner>)>,
    mounted: Mounted,
    plan: SharedFaultPlan,
    counts_after_mount: Vec<DeviceCallCounts>,
    lines: Vec<String>,
    mount_window_matches_device: bool,
}

/// 发布 B 之后：丢掉写的那一套句柄，同一对盘冷重开（`reopen` 拿到关掉的那几块盘、交回重新打开的），走可写挂载（恢复、取号、写行、暖机），
/// 再发布 C。重开是新一轮：计数从零重数，所以这一段自己开一个注入计划（不接着上一段那个）。
///
/// # Errors
/// 可写挂载失败、取号那道屏障没数到、挂载之后现行那一版没有文件、发布 C 失败：交回到那一刻为止的结果行与一句原因。
fn switch_instance_and_publish_third_version<Inner, Reopen>(
    parameters: &MakeFilesystemParameters,
    devices_after_second_transaction: Vec<(DeviceIdentity, CountedDevice<Inner>)>,
    stream: &SharedStream,
    geometry: &FixedGeometry,
    clock: &mut SegmentClock,
    reopen: Reopen,
) -> Result<SecondInstanceRun<Inner>, FailedRun>
where
    Inner: BlockDevice,
    Reopen: FnOnce(Vec<(DeviceIdentity, Inner)>) -> Vec<(DeviceIdentity, Inner)>,
{
    let mounted = reopen_and_mount_writable(
        parameters,
        devices_after_second_transaction,
        stream,
        geometry,
        SharedFaultPlan::unarmed(*geometry),
        clock,
        reopen,
    )?;
    publish_the_third_version_and_describe(parameters, mounted, stream, geometry, clock)
}

/// 可写挂载里每块盘收到第一道屏障（取号那一道）那一刻的计数：挂载窗口（成功与失败两条路）从这里算起。
/// 某块盘一道屏障都没收到（取号那道屏障没发）：交回停下的那一行与原因。
fn counts_when_the_acquisition_barrier_arrived<Inner: BlockDevice>(
    devices: &[(DeviceIdentity, CountedDevice<Inner>)],
    plan: &SharedFaultPlan,
) -> Result<Vec<DeviceCallCounts>, FailedRun> {
    devices
        .iter()
        .map(|(identity, _)| {
            plan.counts_when_the_first_barrier_arrived_at(*identity)
                .map(DeviceCallCounts::of)
                .ok_or_else(|| {
                    format!(
                        "盘 {} 在可写挂载里一道屏障都没收到：取号那道屏障没发",
                        identity.0
                    )
                })
        })
        .collect::<Result<_, _>>()
        .map_err(|cause: String| FailedRun {
            lines: vec![describe_run_failure("reopen_and_writable_mount", &cause)],
            cause,
        })
}

/// 冷重开、可写挂载。挂载那一段窗口从每块盘收到第一道屏障（取号那一道）算起、到挂载返回为止，按种类的合计是写行与每次暖机之和：
/// 取号的系统配置槽写不是发布、不进按种类的账，与第一个事务那条路上暖机窗口从取号之后算起同一个口径。
///
/// # Errors
/// 可写挂载失败（取号之后那一串发布里失败的，挂载随错交回写入口的账，照成功路径的窗口口径与设备一层判相等；
/// 别的错只打停下的原因）、取号那道屏障没数到：交回到那一刻为止的结果行与一句原因。
fn reopen_and_mount_writable<Inner, Reopen>(
    parameters: &MakeFilesystemParameters,
    devices_after_second_transaction: Vec<(DeviceIdentity, CountedDevice<Inner>)>,
    stream: &SharedStream,
    geometry: &FixedGeometry,
    plan: SharedFaultPlan,
    clock: &mut SegmentClock,
    reopen: Reopen,
) -> Result<WritableMountRun<Inner>, FailedRun>
where
    Inner: BlockDevice,
    Reopen: FnOnce(Vec<(DeviceIdentity, Inner)>) -> Vec<(DeviceIdentity, Inner)>,
{
    let closed_devices: Vec<(DeviceIdentity, Inner)> = devices_after_second_transaction
        .into_iter()
        .map(|(identity, device)| (identity, device.into_inner().into_inner_and_operations().0))
        .collect();
    let mut devices: Vec<(DeviceIdentity, CountedDevice<Inner>)> = reopen(closed_devices)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                FaultInjectingBlockDevice::new(
                    identity,
                    RecordingBlockDevice::with_shared_stream(identity, inner, stream.clone()),
                    plan.clone(),
                ),
            )
        })
        .collect();
    let mut lines = Vec::new();

    let operations_before_mount = stream.operations().len();
    let mount_started = Instant::now();
    let mounted = match mount_writable(parameters, &mut devices) {
        Ok(mounted) => mounted,
        Err(failure) => {
            let cause = format!("{failure:?}");
            let failure_lines = match &failure {
                // 取号之后那一串发布里有一次没做成：挂载把写入口的账随错交回（已经落盘的写行与前几次暖机、失败那一次已记的写），
                // 窗口与成功路径的挂载窗口同一个起点——取号那道屏障，取号的两次系统配置槽写不是发布、不在账里。
                MountError::Publish(failed) => {
                    let counts_after_acquisition =
                        counts_when_the_acquisition_barrier_arrived(&devices, &plan)?;
                    describe_failed_window(
                        "reopen_and_writable_mount",
                        &cause,
                        &failed.writes_of_persisted_publishes,
                        &failed.writes_of_failed_publishes,
                        pool_writes_between(
                            &counts_after_acquisition,
                            &device_call_counts(&devices, &plan),
                        ),
                    )
                    .0
                }
                // 没走到取号之后的发布：恢复、准入、取号自己报的错（取号写的回卷在它自己的错里），这一段没有发布的账可比。
                MountError::Recovery(_)
                | MountError::FileVersionWithoutAnyJournalRecord
                | MountError::InstanceTableMalformed
                | MountError::Acquisition(_)
                | MountError::RaiseFloorSequencePublishFailed(_)
                | MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. }
                | MountError::RollbackTargetNotACandidate { .. }
                | MountError::RollbackFloorAboveCeiling { .. }
                | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
                | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
                | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
                | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
                | MountError::SpaceAdmissionRefusedBeforeAcquisition { .. }
                | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
                | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
                | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. }
                | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
                | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. } => {
                    vec![describe_run_failure("reopen_and_writable_mount", &cause)]
                }
            };
            return Err(FailedRun {
                lines: failure_lines,
                cause: format!("可写挂载：{cause}"),
            });
        }
    };
    let mount_nanoseconds = mount_started.elapsed().as_nanos();
    let counts_after_mount = device_call_counts(&devices, &plan);
    let counts_after_acquisition = counts_when_the_acquisition_barrier_arrived(&devices, &plan)?;
    let mount_operations = stream.operations();
    let mount_segments =
        split_into_segments(&mount_operations[operations_before_mount..], geometry);
    let output = &mounted.output;
    let warm_up_txgs: Vec<String> = output
        .warm_up_publishes
        .iter()
        .map(|publish| publish.root().checkpoint_txg.0.to_string())
        .collect();
    let per_device_mount: Vec<String> = counts_after_mount
        .iter()
        .enumerate()
        .map(|(index, counts)| counts.describe(index))
        .collect();
    lines.push(format!(
        "name=writable_mount instance={} chosen_root={}:{} rows_written={} row_publish_root={} warm_up_txgs={} nanoseconds={mount_nanoseconds} operations={} segments={} closed_form={} {}",
        output.instance.0,
        output.chosen_root.instance.0,
        output.chosen_root.checkpoint_txg.0,
        output.rows_written.len(),
        root_text(&output.row_publish),
        warm_up_txgs.join(","),
        mount_operations.len() - operations_before_mount,
        segment_sizes_text(&mount_segments),
        closed_form_state_count(&mount_segments),
        per_device_mount.join(" ")
    ));
    lines.push(describe_publish_writes(
        "instance_row",
        output.row_publish.root().checkpoint_txg.0,
        writes_of(&output.row_publish),
    ));
    let mut mount_publishes: Vec<&WritesByStructureKind> = vec![writes_of(&output.row_publish)];
    for warm_up_publish in &output.warm_up_publishes {
        lines.push(describe_publish_writes(
            "warm_up",
            warm_up_publish.root().checkpoint_txg.0,
            writes_of(warm_up_publish),
        ));
        mount_publishes.push(writes_of(warm_up_publish));
    }
    let (mount_window_line, mount_window_matches) = publish_writes_against_device(
        "writable_mount",
        &mount_publishes,
        pool_writes_between(&counts_after_acquisition, &counts_after_mount),
    );
    lines.push(mount_window_line);
    // 这一段从上一个标记算起，含冷重开那几块盘：名字照实写，不叫「可写挂载」。
    lines.push(clock.mark("reopen_and_writable_mount"));
    Ok(WritableMountRun {
        devices,
        mounted,
        plan,
        counts_after_mount,
        lines,
        mount_window_matches_device: mount_window_matches,
    })
}

/// 可写挂载之后发布 C，接在挂载交回的现行那一版上、用挂载取到的实例。
///
/// # Errors
/// 挂载之后现行那一版没有文件（发布 B 之后重开，上一版带文件）、发布 C 失败：交回到那一刻为止的结果行
/// （发布 C 失败时连同失败账那几行，增补 2 收口表第 58 行）与一句原因。
fn publish_the_third_version_and_describe<Inner: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    mounted_run: WritableMountRun<Inner>,
    stream: &SharedStream,
    geometry: &FixedGeometry,
    clock: &mut SegmentClock,
) -> Result<SecondInstanceRun<Inner>, FailedRun> {
    let WritableMountRun {
        mut devices,
        mut mounted,
        plan,
        counts_after_mount,
        mut lines,
        mount_window_matches_device,
    } = mounted_run;
    let Some(current) = mounted.current.file_version().cloned() else {
        let cause = "可写挂载之后现行那一版没有文件：发布 B 之后重开，上一版带文件".to_string();
        lines.push(describe_run_failure("third_transaction", &cause));
        return Err(FailedRun { lines, cause });
    };
    let instance = mounted.output.instance;
    let third = match publish_an_overwrite_in_the_mounted_instance_and_describe(
        "third_transaction",
        &mut devices,
        &plan,
        &counts_after_mount,
        stream,
        geometry,
        |devices| {
            publish_the_third_version(
                parameters,
                devices,
                &mut mounted.allocator,
                &current,
                instance,
            )
        },
    ) {
        Ok(third) => third,
        Err(failed) => {
            lines.extend(failed.lines);
            return Err(FailedRun {
                lines,
                cause: format!("发布 C：{}", failed.cause),
            });
        }
    };
    lines.extend(third.lines);
    lines.push(clock.mark("third_transaction"));

    Ok(SecondInstanceRun {
        devices,
        allocator: mounted.allocator,
        current_version: third.version,
        plan,
        lines,
        every_window_matches_device: mount_window_matches_device && third.window_matches_device,
    })
}

/// 同一次挂载里一次覆盖写成功之后：这一版、这一段的结果行（发布一行、`name=publish_writes` 一行、窗口行一行）、
/// 这一段按种类的合计与设备一层是否相等。
struct OverwriteInTheMountedInstance {
    version: TransactionOutput,
    lines: Vec<String>,
    window_matches_device: bool,
}

/// 同一次挂载里的一次覆盖写（发布 C；`raise-rollback-floor` 还有发布 D）：`publish` 发这一次，挂钟只计它；
/// 窗口从 `counts_before` 那一刻算起、到它返回为止，按种类的合计与设备一层逐项比。结果行以 `window` 为名（`name=<window>` 一行
/// 与 `publish=<window>`、`window=<window>` 各一行）。
///
/// # Errors
/// 发布失败：交回失败账那几行（增补 2 收口表第 58 行）与失败原因的 Debug 文本，调用方在前面接上它前面几段的结果行。
fn publish_an_overwrite_in_the_mounted_instance_and_describe<Inner, Publish>(
    window: &str,
    devices: &mut [(DeviceIdentity, CountedDevice<Inner>)],
    plan: &SharedFaultPlan,
    counts_before: &[DeviceCallCounts],
    stream: &SharedStream,
    geometry: &FixedGeometry,
    publish: Publish,
) -> Result<OverwriteInTheMountedInstance, FailedRun>
where
    Inner: BlockDevice,
    Publish: FnOnce(
        &mut [(DeviceIdentity, CountedDevice<Inner>)],
    ) -> Result<TransactionOutput, FailedPublish>,
{
    let operations_before = stream.operations().len();
    let started = Instant::now();
    let published = publish(devices);
    let nanoseconds = started.elapsed().as_nanos();
    let counts_after = device_call_counts(devices, plan);
    let version = match published {
        Ok(version) => version,
        Err(failed) => {
            let cause = format!("{:?}", failed.cause);
            let (failure_lines, _) = describe_failed_window(
                window,
                &cause,
                &[],
                &failed.writes_of_failed_publishes,
                pool_writes_between(counts_before, &counts_after),
            );
            return Err(FailedRun {
                lines: failure_lines,
                cause,
            });
        }
    };
    let operations = stream.operations();
    let segments = split_into_segments(&operations[operations_before..], geometry);
    let per_device: Vec<String> = counts_after
        .iter()
        .enumerate()
        .map(|(index, later)| later.since(counts_before[index]).describe(index))
        .collect();
    let mut lines = vec![
        format!(
            "name={window} root_txg={} transaction={} released={} nanoseconds={nanoseconds} operations={} segments={} closed_form={} {}",
            version.root.checkpoint_txg.0,
            version.record.transaction,
            version.released.len(),
            operations.len() - operations_before,
            segment_sizes_text(&segments),
            closed_form_state_count(&segments),
            per_device.join(" ")
        ),
        describe_publish_writes(window, version.root.checkpoint_txg.0, &version.writes),
    ];
    let (window_line, window_matches_device) = publish_writes_against_device(
        window,
        &[&version.writes],
        pool_writes_between(counts_before, &counts_after),
    );
    lines.push(window_line);
    Ok(OverwriteInTheMountedInstance {
        version,
        lines,
        window_matches_device,
    })
}

/// `raise-rollback-floor` 的发布 D：发布 C 之后、同一次挂载里接在发布 C 那一版上再覆盖写一次（`publish_the_fourth_version`，
/// 宿主检查重跑同一份），这一段窗口从发布 C 返回那一刻算起。结果行与发布 C 同形，名字是 `fourth_transaction`。
///
/// # Errors
/// 发布 D 失败：接在前面几段的结果行后面交回失败账那几行（增补 2 收口表第 58 行）与一句原因。
fn publish_the_fourth_version_and_describe<Inner: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    second_instance_run: SecondInstanceRun<Inner>,
    stream: &SharedStream,
    geometry: &FixedGeometry,
    clock: &mut SegmentClock,
) -> Result<SecondInstanceRun<Inner>, FailedRun> {
    let SecondInstanceRun {
        mut devices,
        mut allocator,
        current_version: third_version,
        plan,
        mut lines,
        every_window_matches_device,
    } = second_instance_run;
    let counts_after_third = device_call_counts(&devices, &plan);
    let fourth = match publish_an_overwrite_in_the_mounted_instance_and_describe(
        "fourth_transaction",
        &mut devices,
        &plan,
        &counts_after_third,
        stream,
        geometry,
        |devices| publish_the_fourth_version(parameters, devices, &mut allocator, &third_version),
    ) {
        Ok(fourth) => fourth,
        Err(failed) => {
            lines.extend(failed.lines);
            return Err(FailedRun {
                lines,
                cause: format!("发布 D：{}", failed.cause),
            });
        }
    };
    lines.extend(fourth.lines);
    lines.push(clock.mark("fourth_transaction"));

    Ok(SecondInstanceRun {
        devices,
        allocator,
        current_version: fourth.version,
        plan,
        lines,
        every_window_matches_device: every_window_matches_device && fourth.window_matches_device,
    })
}

/// `raise-rollback-floor` 在发布 C 之后的两段：同一次挂载里发布 D（第 4 新的非空有效根落到 A 上），再把 F 抬到上限。`main` 与用例走这同一个函数。
///
/// # Errors
/// 发布 D 或抬 F 失败：交回到那一刻为止的结果行（失败账那几行在内）与一句原因。
fn publish_the_fourth_version_and_raise_the_rollback_floor<Inner: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    third_version_run: SecondInstanceRun<Inner>,
    stream: &SharedStream,
    geometry: &FixedGeometry,
    clock: &mut SegmentClock,
) -> Result<SecondInstanceRun<Inner>, FailedRun> {
    let fourth_version_run = publish_the_fourth_version_and_describe(
        parameters,
        third_version_run,
        stream,
        geometry,
        clock,
    )?;
    raise_the_rollback_floor_and_describe(parameters, fourth_version_run, stream, geometry, clock)
}

/// `raise-rollback-floor` 模式冷重开之后打的那一行：所选根从盘上读回的回退下界 F（代码轮第二轮判决 Z10，
/// `research/prompts/m2-final-code-r2-main-verification.md` 第四节第 5 条）。所选根照恢复择根的同一套规则现读
/// （`choose_system_configuration` + `choose_root`：先跳过被回退见证抛弃的根，再按 (txg, 实例) 择新），F 取那条根记录自己的字段——
/// 不取 `name=raise_rollback_floor` 那一行里内存那一版的 `requested_floor`。择不出系统配置或根时两段都写 `none`。
fn rollback_floor_read_back_from_the_chosen_root_line(reader: &dyn PoolReader) -> String {
    let chosen_root = choose_system_configuration(reader)
        .ok()
        .and_then(|system_configuration| choose_root(reader, &system_configuration));
    match chosen_root {
        Some(root) => format!(
            "name=recover_cold_rollback_floor chosen_root={}:{} rollback_floor_on_disk={}",
            root.instance.0, root.checkpoint_txg.0, root.rollback_floor.0
        ),
        None => "name=recover_cold_rollback_floor chosen_root=none rollback_floor_on_disk=none"
            .to_string(),
    }
}

/// `raise-rollback-floor`：发布 D 之后、同一次挂载里把 F 抬到上限（`raise_the_rollback_floor_to_its_ceiling`，宿主检查重跑同一份）。
/// 这一段窗口从抬 F 之前那一刻算起、到它返回为止，按种类的合计是那一串空发布之和，与设备一层逐项比（增补 2 收口表第 58 行
/// 「二进制那一侧判相等」）。抬 F 只有测试入口（`raise_rollback_floor` 的文档注释），这一档就是那个入口在真设备上的一次。
/// 结果行里 `requested_floor` 是抬完之后最后一次空发布的根带的 F，`ceiling` 是 core 抬 F 时算的上限。
///
/// # Errors
/// 抬 F 失败：那一串空发布里有一次发不出去时，失败账那几行照可写挂载失败的判法打（已落盘的那几次各一行、失败那一次一行、
/// 两样相加与设备一层比一行）；别的错在任何写之前，只打停下的原因。都接在前面几段的结果行后面交回。
fn raise_the_rollback_floor_and_describe<Inner: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    second_instance_run: SecondInstanceRun<Inner>,
    stream: &SharedStream,
    geometry: &FixedGeometry,
    clock: &mut SegmentClock,
) -> Result<SecondInstanceRun<Inner>, FailedRun> {
    let SecondInstanceRun {
        mut devices,
        mut allocator,
        mut current_version,
        plan,
        mut lines,
        every_window_matches_device,
    } = second_instance_run;
    let floor_before_raise = current_version.root.rollback_floor;
    let counts_before_raise = device_call_counts(&devices, &plan);
    let operations_before_raise = stream.operations().len();
    let raise_started = Instant::now();
    let raised = raise_the_rollback_floor_to_its_ceiling(
        parameters,
        &mut devices,
        &mut allocator,
        &mut current_version,
    );
    let raise_nanoseconds = raise_started.elapsed().as_nanos();
    let counts_after_raise = device_call_counts(&devices, &plan);
    let raised = match raised {
        Ok(raised) => raised,
        Err(failure) => {
            let cause = format!("{failure:?}");
            let failure_lines = match &failure {
                // 抬 F 的写入口随错丢掉，这一串的账随错交回（已经落盘的那几次空发布、失败那一次已记的写）。
                MountError::RaiseFloorSequencePublishFailed(PublishSequenceFailed {
                    cause: _,
                    writes_of_persisted_publishes,
                    writes_of_failed_publishes,
                }) => {
                    describe_failed_window(
                        "raise_rollback_floor",
                        &cause,
                        writes_of_persisted_publishes,
                        writes_of_failed_publishes,
                        pool_writes_between(&counts_before_raise, &counts_after_raise),
                    )
                    .0
                }
                // 在第一次空发布之前就停下的：选系统配置、读实例表、算上限、超上限、整串预演里有一次报错，这一段没有发布的账可比。
                // 可写挂载与回退那几条（取号、写行、暖机、回退目标）抬 F 走不到，照样列全，不写通配臂。
                MountError::Recovery(_)
                | MountError::FileVersionWithoutAnyJournalRecord
                | MountError::InstanceTableMalformed
                | MountError::Acquisition(_)
                | MountError::Publish(_)
                | MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. }
                | MountError::RollbackTargetNotACandidate { .. }
                | MountError::RollbackFloorAboveCeiling { .. }
                | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
                | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
                | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
                | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
                | MountError::SpaceAdmissionRefusedBeforeAcquisition { .. }
                | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
                | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
                | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. }
                | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
                | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. } => {
                    vec![describe_run_failure("raise_rollback_floor", &cause)]
                }
            };
            lines.extend(failure_lines);
            return Err(FailedRun {
                lines,
                cause: format!("抬 F：{cause}"),
            });
        }
    };
    let raise_operations = stream.operations();
    let raise_segments =
        split_into_segments(&raise_operations[operations_before_raise..], geometry);
    let per_device_raise: Vec<String> = counts_after_raise
        .iter()
        .enumerate()
        .map(|(index, later)| later.since(counts_before_raise[index]).describe(index))
        .collect();
    let raise_txgs: Vec<String> = raised
        .publishes
        .iter()
        .map(|publish| publish.root.checkpoint_txg.0.to_string())
        .collect();
    lines.push(format!(
        "name=raise_rollback_floor floor_before={} requested_floor={} ceiling={} publishes={} root_txgs={} reclaimed={} abandoned_roots_unreadable={} nanoseconds={raise_nanoseconds} operations={} segments={} closed_form={} {}",
        floor_before_raise.0,
        current_version.root.rollback_floor.0,
        raised.ceiling.0,
        raised.publishes.len(),
        raise_txgs.join(","),
        raised.reclaimed.len(),
        raised.abandoned_roots_unreadable,
        raise_operations.len() - operations_before_raise,
        segment_sizes_text(&raise_segments),
        closed_form_state_count(&raise_segments),
        per_device_raise.join(" ")
    ));
    let mut raise_publishes: Vec<&WritesByStructureKind> = Vec::new();
    for raise_publish in &raised.publishes {
        lines.push(describe_publish_writes(
            "raise_rollback_floor",
            raise_publish.root.checkpoint_txg.0,
            &raise_publish.writes,
        ));
        raise_publishes.push(&raise_publish.writes);
    }
    let (raise_window_line, raise_window_matches) = publish_writes_against_device(
        "raise_rollback_floor",
        &raise_publishes,
        pool_writes_between(&counts_before_raise, &counts_after_raise),
    );
    lines.push(raise_window_line);
    lines.push(clock.mark("raise_rollback_floor"));

    Ok(SecondInstanceRun {
        devices,
        allocator,
        current_version,
        plan,
        lines,
        every_window_matches_device: every_window_matches_device && raise_window_matches,
    })
}

#[allow(
    clippy::too_many_lines,
    reason = "虚机里只跑这一件事：写、数、重开、读回，结果行按次序打"
)]
fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() != 4 {
        eprintln!(
            "用法：first_transaction_on_device <盘 0> <盘 1> <{}>",
            OnDeviceRunMode::every_argument_for_usage()
        );
        std::process::exit(2);
    }
    let device_paths = [arguments[1].clone(), arguments[2].clone()];
    let Some(mode) = OnDeviceRunMode::from_argument(&arguments[3]) else {
        eprintln!("不认识的模式 {}", arguments[3]);
        std::process::exit(2)
    };
    let policy = match mode {
        OnDeviceRunMode::Direct
        | OnDeviceRunMode::SkipFirstTransactionBarrier
        | OnDeviceRunMode::SecondTransaction
        | OnDeviceRunMode::SecondInstance
        | OnDeviceRunMode::RaiseRollbackFloor => PageCachePolicy::BypassWithDirectInputOutput,
        OnDeviceRunMode::PageCache => PageCachePolicy::GoThroughPageCache,
    };
    let mut emitter = Emitter { emitted: 0 };
    let probe = |attribute: &str| -> u32 {
        let values: Vec<Option<u32>> = device_paths
            .iter()
            .map(|path| {
                probe_queue_number_under(Path::new(SYSFS_CLASS_BLOCK), Path::new(path), attribute)
            })
            .collect();
        match (values[0], values[1]) {
            (Some(first), Some(second)) if first == second => first,
            (first, second) => {
                eprintln!("两块盘的 queue/{attribute} 读不到或不相等：{first:?} / {second:?}");
                std::process::exit(4)
            }
        }
    };
    let physical_block_size = probe("physical_block_size");
    let minimum_input_output_bytes = probe("minimum_io_size");
    let write_cache = probe_queue_text_under(
        Path::new(SYSFS_CLASS_BLOCK),
        Path::new(&device_paths[0]),
        "write_cache",
    )
    .unwrap_or_else(|| "NA".to_string());
    let parameters = e142_parameters(physical_block_size, minimum_input_output_bytes);
    let stream = SharedStream::new();
    let geometry_for_faults = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let plan = SharedFaultPlan::unarmed(geometry_for_faults);
    let mut devices: Vec<(DeviceIdentity, CountedDevice<DirectInputOutputBlockDevice>)> =
        device_paths
            .iter()
            .enumerate()
            .map(|(index, path)| {
                let identity = DeviceIdentity(u32::try_from(index).expect("设备号"));
                (
                    identity,
                    FaultInjectingBlockDevice::new(
                        identity,
                        RecordingBlockDevice::with_shared_stream(
                            identity,
                            open(path, policy),
                            stream.clone(),
                        ),
                        plan.clone(),
                    ),
                )
            })
            .collect();
    let device_bytes = devices[0].1.size_in_bytes();
    emitter.emit(&format!(
        "name=geometry mode={} physical_block_size={physical_block_size} minimum_io={minimum_input_output_bytes} spacing={} write_cache={} device_bytes={device_bytes}",
        arguments[3],
        parameters.geometry.fixed_structure_slot_spacing,
        write_cache.replace(' ', "_")
    ));
    let counters_before: Vec<_> = device_paths
        .iter()
        .map(|path| block_layer_counters(path))
        .collect();
    let mut counts_after_acquisition: Option<Vec<DeviceCallCounts>> = None;
    let mut counts_before_first_transaction: Option<Vec<DeviceCallCounts>> = None;
    // 分段挂钟在这里起表：往下每一段末尾取一次，行先攒着、跑完一起打——
    // 打印会阻塞（虚机的串口），把它摆在两次取表之间就等于把打印积压算进那一段
    //（`test-discipline.md`「分段计时在被测进程里计，不在转发输出的循环里计」）。
    let mut clock = SegmentClock::start();
    let mut segment_timing_lines: Vec<String> = Vec::new();
    let path_run = run_first_transaction(&parameters, &mut devices, &stream, |point, devices| {
        let counts: Vec<DeviceCallCounts> = devices
            .iter()
            .map(|(identity, _)| DeviceCallCounts::of(plan.counts_of_device(*identity)))
            .collect();
        match point {
            ScenarioPoint::AfterMakeFilesystem => {
                segment_timing_lines.push(clock.mark("mkfs"));
            }
            ScenarioPoint::AfterInstanceAcquisition => {
                counts_after_acquisition = Some(counts);
                segment_timing_lines.push(clock.mark("instance_acquisition"));
            }
            ScenarioPoint::BeforeFirstTransaction => {
                counts_before_first_transaction = Some(counts);
                segment_timing_lines.push(clock.mark("warm_up"));
                if mode == OnDeviceRunMode::SkipFirstTransactionBarrier {
                    plan.arm(swallow_the_next_barrier_on(devices[0].0));
                }
            }
        }
    });
    let run = match path_run {
        Ok(run) => run,
        Err(failure) => {
            let lines = describe_first_transaction_path_failure(
                &failure,
                counts_after_acquisition.as_deref(),
                counts_before_first_transaction.as_deref(),
                &device_call_counts(&devices, &plan),
            );
            exit_after_a_failed_run(
                &mut emitter,
                &FailedRun {
                    lines,
                    cause: failure.render(),
                },
            )
        }
    };
    segment_timing_lines.push(clock.mark("first_transaction"));
    let counters_after: Vec<_> = device_paths
        .iter()
        .map(|path| block_layer_counters(path))
        .collect();
    let counts_after_first_transaction: Vec<DeviceCallCounts> = devices
        .iter()
        .map(|(identity, _)| DeviceCallCounts::of(plan.counts_of_device(*identity)))
        .collect();
    let counts_after_acquisition =
        counts_after_acquisition.expect("run_first_transaction 取号之后叫过一次");
    let counts_before_first_transaction =
        counts_before_first_transaction.expect("run_first_transaction 第一个事务之前叫过一次");

    let operations = stream.operations();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let warm_up_end = operations.len() - 23;
    let paths: [(&str, &[RecordedOperation]); 5] = [
        ("mkfs", &operations[..run.mkfs_operation_count]),
        (
            "instance_acquisition",
            &operations[run.mkfs_operation_count..run.mkfs_operation_count + 2],
        ),
        (
            "warm_up",
            &operations[run.mkfs_operation_count + 2..warm_up_end],
        ),
        ("transaction", &operations[warm_up_end..]),
        ("post_mkfs_stream", &operations[run.mkfs_operation_count..]),
    ];
    for (name, slice) in paths {
        let segments = split_into_segments(slice, &geometry);
        emitter.emit(&format!(
            "name=segments path={name} operations={} segments={} closed_form={} kinds={}",
            slice.len(),
            segment_sizes_text(&segments),
            closed_form_state_count(&segments),
            segment_kinds_text(&segments)
        ));
    }
    for (index, (identity, _)) in devices.iter().enumerate() {
        let counted = plan.counts_of_device(*identity);
        emitter.emit(&format!(
            "name=device_calls device={} path={} writes={} written_bytes={} force_unit_access_writes={} barriers={} skipped_barriers={} {}",
            identity.0,
            device_paths[index],
            counted.writes,
            counted.written_bytes,
            counted.force_unit_access_writes,
            counted.barriers_forwarded,
            counted.barriers_swallowed,
            describe_counters(counters_before[index], counters_after[index])
        ));
    }
    for (warm_up_writes, warm_up_root) in run.warm_up.writes.iter().zip(&run.warm_up.roots) {
        emitter.emit(&describe_publish_writes(
            "warm_up",
            warm_up_root.checkpoint_txg.0,
            warm_up_writes,
        ));
    }
    emitter.emit(&describe_publish_writes(
        "first_transaction",
        run.output.root.checkpoint_txg.0,
        &run.output.writes,
    ));
    let warm_up_publishes: Vec<&WritesByStructureKind> = run.warm_up.writes.iter().collect();
    let (warm_up_line, warm_up_matches) = publish_writes_against_device(
        "warm_up",
        &warm_up_publishes,
        pool_writes_between(&counts_after_acquisition, &counts_before_first_transaction),
    );
    emitter.emit(&warm_up_line);
    let (first_transaction_line, first_transaction_matches) = publish_writes_against_device(
        "first_transaction",
        &[&run.output.writes],
        pool_writes_between(
            &counts_before_first_transaction,
            &counts_after_first_transaction,
        ),
    );
    emitter.emit(&first_transaction_line);
    let mut every_window_matches_device = warm_up_matches && first_transaction_matches;
    emitter.emit(&format!(
        "name=transaction key_order_mismatches={} root_txg={} back_chain={}",
        run.output.key_order_mismatches,
        run.output.root.checkpoint_txg.0,
        run.output.record.back_chain
    ));

    // 第二个事务（发布 B）：同一个进程里再覆盖写一次（`publish_the_second_version_and_describe`）。
    match mode.publishes_after_the_first_transaction() {
        PublishesAfterTheFirstTransaction::Nothing => {}
        PublishesAfterTheFirstTransaction::SecondVersion
        | PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance
        | PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstanceThenFourthVersionThenRaiseOfTheRollbackFloor => {
            let second = publish_the_second_version_and_describe(
                &parameters,
                &mut devices,
                &plan,
                &stream,
                &geometry,
                &run.output,
            )
            .unwrap_or_else(|failed| exit_after_a_failed_run(&mut emitter, &failed));
            for line in &second.lines {
                emitter.emit(line);
            }
            every_window_matches_device &= second.window_matches_device;
            segment_timing_lines.push(clock.mark("second_transaction"));
        }
    }

    // `second-instance`：发布 B 之后同一对盘冷重开、可写挂载，再发布 C；`raise-rollback-floor` 接着在同一次挂载里发布 D、抬 F。
    let reopen_by_path = |closed_devices: Vec<(DeviceIdentity, DirectInputOutputBlockDevice)>| {
        drop(closed_devices);
        device_paths
            .iter()
            .enumerate()
            .map(|(index, path)| {
                (
                    DeviceIdentity(u32::try_from(index).expect("设备号")),
                    open(path, policy),
                )
            })
            .collect::<Vec<_>>()
    };
    let devices = match mode.publishes_after_the_first_transaction() {
        PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance => {
            let switched = switch_instance_and_publish_third_version(
                &parameters,
                devices,
                &stream,
                &geometry,
                &mut clock,
                reopen_by_path,
            )
            .unwrap_or_else(|failed| exit_after_a_failed_run(&mut emitter, &failed));
            emit_the_lines_after_the_second_version(
                &mut emitter,
                &mut segment_timing_lines,
                &switched.lines,
            );
            every_window_matches_device &= switched.every_window_matches_device;
            switched.devices
        }
        PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstanceThenFourthVersionThenRaiseOfTheRollbackFloor => {
            let raised = switch_instance_and_publish_third_version(
                &parameters,
                devices,
                &stream,
                &geometry,
                &mut clock,
                reopen_by_path,
            )
            .and_then(|switched| {
                publish_the_fourth_version_and_raise_the_rollback_floor(
                    &parameters,
                    switched,
                    &stream,
                    &geometry,
                    &mut clock,
                )
            })
            .unwrap_or_else(|failed| exit_after_a_failed_run(&mut emitter, &failed));
            emit_the_lines_after_the_second_version(
                &mut emitter,
                &mut segment_timing_lines,
                &raised.lines,
            );
            every_window_matches_device &= raised.every_window_matches_device;
            raised.devices
        }
        PublishesAfterTheFirstTransaction::Nothing
        | PublishesAfterTheFirstTransaction::SecondVersion => devices,
    };
    let expected_content = mode.last_published_content();

    // 冷重开：丢掉写的那一套句柄，按路径重新打开，恢复只通过块设备读盘。
    drop(devices);
    let reopened: Vec<(DeviceIdentity, DirectInputOutputBlockDevice)> = device_paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            (
                DeviceIdentity(u32::try_from(index).expect("设备号")),
                open(path, policy),
            )
        })
        .collect();
    let report = recover(&reopened, JournalPolicy::Consult);
    // 抬 F 模式多打一行：所选根从盘上读回的 F（代码轮第二轮判决 Z10：`name=raise_rollback_floor` 那一行的 `requested_floor=`
    // 是内存里那一版的 F，冷重开择根并不看它，55 号要独立读回盘上的 F 才判得出「F 抬到 3」）。别的模式一行不多。
    let rollback_floor_read_back_line = match mode.publishes_after_the_first_transaction() {
        PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstanceThenFourthVersionThenRaiseOfTheRollbackFloor => {
            Some(rollback_floor_read_back_from_the_chosen_root_line(&reopened))
        }
        PublishesAfterTheFirstTransaction::Nothing
        | PublishesAfterTheFirstTransaction::SecondVersion
        | PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance => None,
    };
    let (outcome, root, content_matches) = match &report.outcome {
        RecoveryOutcome::FileRead { root, content } => (
            "file_read",
            format!("{}:{}", root.0 .0, root.1 .0),
            *content == expected_content,
        ),
        RecoveryOutcome::NoFile { root } => {
            ("no_file", format!("{}:{}", root.0 .0, root.1 .0), false)
        }
        RecoveryOutcome::Failed { root, failure } => {
            eprintln!("恢复失败：{failure:?}");
            (
                "failed",
                root.map_or_else(
                    || "none".to_string(),
                    |root| format!("{}:{}", root.0 .0, root.1 .0),
                ),
                false,
            )
        }
    };
    segment_timing_lines.push(clock.mark("cold_reopen_and_recover"));
    emitter.emit(&format!(
        "name=recover_cold outcome={outcome} root={root} content_matches={content_matches} valid_records={} above_water={} applied={} verification_passed={} mapping_fallbacks={}",
        report.journal.valid_records, report.journal.above_water, report.journal.prefix_applied, report.journal.verification_passed, report.mapping_fallbacks
    ));
    if let Some(line) = &rollback_floor_read_back_line {
        emitter.emit(line);
    }
    // 分段挂钟：每段一行，最后一行是包含自检（各段之和不超过整条路）。
    for line in &segment_timing_lines {
        emitter.emit(line);
    }
    let (registration_line, segment_timing_registered) =
        clock.registration_check(&arguments[3], registered_segments_of(mode));
    emitter.emit(&registration_line);
    let (containment_line, segment_timing_contained) = clock.containment_check();
    emitter.emit(&containment_line);
    emitter.finish();
    if !(outcome == "file_read"
        && content_matches
        && every_window_matches_device
        && segment_timing_registered
        && segment_timing_contained)
    {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        describe_publish_writes, pool_writes_between, publish_writes_against_device,
        registered_segments_of, switch_instance_and_publish_third_version, CountedDevice,
        DeviceCallCounts, SegmentClock,
    };
    use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
    use singlefs_core::block_device::PhysicalBlockSizeInBytes;
    use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
    use singlefs_core::write_accounting::{
        WriteCallsAndBytes, WritesByStructureKind, WrittenStructureKind,
    };
    use singlefs_harness::crash::SparseBlockDevice;
    use singlefs_harness::fault_injection::{FaultInjectingBlockDevice, SharedFaultPlan};
    use singlefs_harness::on_device_modes::{
        allocator_rebuilt_from_the_records_of, fourth_file_content, publish_the_second_version,
        third_file_content, OnDeviceRunMode,
    };
    use singlefs_harness::scenario::{e142_parameters, run_first_transaction};
    use singlefs_harness::segments::FixedGeometry;
    use singlefs_harness::{RecordingBlockDevice, SharedStream};

    /// 虚机里两块 virtio 盘的大小以外都照 E142 的几何：物理块 512、io_min 512。
    const SPARSE_DEVICE_BYTES: u64 = 4 << 30;

    /// 结果行里 `key=value` 那一段的值；没有这个字段是 `None`。
    fn field<'line>(line: &'line str, key: &str) -> Option<&'line str> {
        line.split(' ')
            .find_map(|part| part.strip_prefix(key)?.strip_prefix('='))
    }

    /// `second-instance` 模式在宿主上照同一条路跑一遍：稀疏内存盘代替 virtio 盘，「冷重开」把镜像交给新句柄。
    /// 发布 B 之后可写挂载取实例 2、写行 txg 5、暖机 txg 6 / 7，发布 C 是 txg 8；两段窗口按种类的合计都与设备一层相等；
    /// 冷恢复择实例 2 的根读回第三版。
    #[test]
    fn second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches(
    ) {
        let parameters = e142_parameters(512, 512);
        let stream = SharedStream::new();
        let geometry_for_faults = FixedGeometry {
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
            root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
        };
        let plan = SharedFaultPlan::unarmed(geometry_for_faults);
        let mut devices: Vec<(DeviceIdentity, CountedDevice<SparseBlockDevice>)> = (0..2u32)
            .map(|device_number| {
                let identity = DeviceIdentity(device_number);
                (
                    identity,
                    FaultInjectingBlockDevice::new(
                        identity,
                        RecordingBlockDevice::with_shared_stream(
                            identity,
                            SparseBlockDevice::new(
                                SPARSE_DEVICE_BYTES,
                                PhysicalBlockSizeInBytes(512),
                            ),
                            stream.clone(),
                        ),
                        plan.clone(),
                    ),
                )
            })
            .collect();
        let run = run_first_transaction(&parameters, &mut devices, &stream, |_point, _devices| {})
            .expect("第一个事务");
        let mut allocator = allocator_rebuilt_from_the_records_of(&devices, &run.output);
        let second =
            publish_the_second_version(&parameters, &mut devices, &mut allocator, &run.output)
                .expect("发布 B");
        assert_eq!(second.root.checkpoint_txg, CheckpointTxg(4));
        let geometry = FixedGeometry {
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
            root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
        };

        let mut clock = SegmentClock::start();
        let switched = switch_instance_and_publish_third_version(
            &parameters,
            devices,
            &stream,
            &geometry,
            &mut clock,
            |closed_devices| {
                closed_devices
                    .into_iter()
                    .map(|(identity, closed)| {
                        let mut reopened = SparseBlockDevice::new(
                            SPARSE_DEVICE_BYTES,
                            PhysicalBlockSizeInBytes(512),
                        );
                        reopened.image = closed.image;
                        (identity, reopened)
                    })
                    .collect()
            },
        )
        .expect("第二个实例");

        let line_named = |name: &str| -> Vec<&str> {
            switched
                .lines
                .iter()
                .map(String::as_str)
                .filter(|line| field(line, "name") == Some(name))
                .collect()
        };
        let mount_lines = line_named("writable_mount");
        assert_eq!(mount_lines.len(), 1, "{:#?}", switched.lines);
        let mount = mount_lines[0];
        assert_eq!(field(mount, "instance"), Some("2"), "{mount}");
        assert_eq!(field(mount, "chosen_root"), Some("1:4"), "B 的根：{mount}");
        assert_eq!(
            field(mount, "rows_written"),
            Some("1"),
            "实例 1 那一行：{mount}"
        );
        assert_eq!(field(mount, "row_publish_root"), Some("2:5"), "{mount}");
        assert_eq!(field(mount, "warm_up_txgs"), Some("6,7"), "{mount}");

        let publish_lines: Vec<(Option<&str>, Option<&str>)> = line_named("publish_writes")
            .into_iter()
            .map(|line| (field(line, "publish"), field(line, "txg")))
            .collect();
        assert_eq!(
            publish_lines,
            vec![
                (Some("instance_row"), Some("5")),
                (Some("warm_up"), Some("6")),
                (Some("warm_up"), Some("7")),
                (Some("third_transaction"), Some("8")),
            ]
        );
        let windows = line_named("publish_writes_against_device");
        let window_summary: Vec<(Option<&str>, Option<&str>, Option<&str>)> = windows
            .iter()
            .map(|line| {
                (
                    field(line, "window"),
                    field(line, "publishes"),
                    field(line, "matches"),
                )
            })
            .collect();
        assert_eq!(
            window_summary,
            vec![
                (Some("writable_mount"), Some("3"), Some("true")),
                (Some("third_transaction"), Some("1"), Some("true")),
            ],
            "两段窗口按种类的合计都等于设备一层数的：{windows:#?}"
        );
        assert!(switched.every_window_matches_device);
        let third_lines = line_named("third_transaction");
        assert_eq!(third_lines.len(), 1, "{:#?}", switched.lines);
        assert_eq!(field(third_lines[0], "root_txg"), Some("8"));
        assert_eq!(
            field(third_lines[0], "transaction"),
            Some("1"),
            "事务号按实例从 1 起"
        );
        assert_eq!(
            field(third_lines[0], "segments"),
            Some("16+2+1+2"),
            "发布 C 与发布 B 同型"
        );

        let cold: Vec<(DeviceIdentity, SparseBlockDevice)> = switched
            .devices
            .into_iter()
            .map(|(identity, device)| (identity, device.into_inner().into_inner_and_operations().0))
            .collect();
        let report = recover(&cold, JournalPolicy::Consult);
        match report.outcome {
            RecoveryOutcome::FileRead { root, content } => {
                assert_eq!(root, (InstanceGeneration(2), CheckpointTxg(8)));
                assert!(content == third_file_content(), "冷恢复读回第三版");
            }
            RecoveryOutcome::NoFile { root } => panic!("冷恢复择到 {root:?} 却没有文件"),
            RecoveryOutcome::Failed { root, failure } => {
                panic!("冷恢复失败：{root:?} {failure:?}")
            }
        }
    }

    fn device_counts(write_calls: u64, written_bytes: u64) -> DeviceCallCounts {
        DeviceCallCounts {
            write_calls,
            written_bytes,
            force_unit_access_writes: 0,
            barrier_calls: 0,
        }
    }

    #[test]
    fn publish_writes_line_gives_the_total_then_every_kind_in_report_order_with_zeros() {
        let mut writes = WritesByStructureKind::NOTHING_WRITTEN;
        writes.count_write_call(WrittenStructureKind::JournalRecord, &[0u8; 4096]);
        writes.count_write_call(WrittenStructureKind::JournalRecord, &[0u8; 4096]);
        writes.count_write_call(WrittenStructureKind::RootSlot, &[0u8; 512]);
        assert_eq!(
            describe_publish_writes("warm_up", 1, &writes),
            "name=publish_writes publish=warm_up txg=1 write_calls=3 written_bytes=8704 \
             data_unit_write_calls=0 data_unit_written_bytes=0 \
             extent_tree_node_write_calls=0 extent_tree_node_written_bytes=0 \
             inode_tree_leaf_container_write_calls=0 inode_tree_leaf_container_written_bytes=0 \
             inode_tree_root_write_calls=0 inode_tree_root_written_bytes=0 \
             allocation_record_tree_node_write_calls=0 allocation_record_tree_node_written_bytes=0 \
             accounting_tree_node_write_calls=0 accounting_tree_node_written_bytes=0 \
             central_mapping_tree_node_write_calls=0 central_mapping_tree_node_written_bytes=0 \
             tree_table_unit_write_calls=0 tree_table_unit_written_bytes=0 \
             instance_table_unit_write_calls=0 instance_table_unit_written_bytes=0 \
             journal_record_write_calls=2 journal_record_written_bytes=8192 \
             root_slot_write_calls=1 root_slot_written_bytes=512 \
             system_configuration_slot_write_calls=0 system_configuration_slot_written_bytes=0"
        );
    }

    #[test]
    fn device_window_check_matches_only_when_both_write_calls_and_bytes_are_equal() {
        let mut first = WritesByStructureKind::NOTHING_WRITTEN;
        first.count_write_call(WrittenStructureKind::SystemConfigurationSlot, &[0u8; 4096]);
        let mut second = WritesByStructureKind::NOTHING_WRITTEN;
        second.count_write_call(WrittenStructureKind::RootSlot, &[0u8; 512]);
        let (line, matches) = publish_writes_against_device(
            "warm_up",
            &[&first, &second],
            WriteCallsAndBytes {
                write_calls: 2,
                written_bytes: 4608,
            },
        );
        assert!(matches, "两次发布合起来 2 次、4608 字节，与设备一层相等");
        assert_eq!(
            line,
            "name=publish_writes_against_device window=warm_up publishes=2 by_kind_write_calls=2 by_kind_written_bytes=4608 device_write_calls=2 device_written_bytes=4608 matches=true"
        );
        let (_, bytes_differ) = publish_writes_against_device(
            "warm_up",
            &[&first, &second],
            WriteCallsAndBytes {
                write_calls: 2,
                written_bytes: 4096,
            },
        );
        assert!(!bytes_differ, "字节差一个根槽：判不相等");
        let (calls_differ_line, calls_differ) = publish_writes_against_device(
            "first_transaction",
            &[&first],
            WriteCallsAndBytes {
                write_calls: 2,
                written_bytes: 4096,
            },
        );
        assert!(!calls_differ, "写调用差一次：判不相等");
        assert!(calls_differ_line.ends_with(" matches=false"));
    }

    #[test]
    fn pool_writes_between_adds_the_differences_of_every_device() {
        let earlier = [device_counts(11, 172_544), device_counts(10, 172_032)];
        let later = [device_counts(22, 345_088), device_counts(20, 344_064)];
        assert_eq!(
            pool_writes_between(&earlier, &later),
            WriteCallsAndBytes {
                write_calls: 21,
                written_bytes: 344_576
            }
        );
    }

    #[test]
    fn device_call_counts_subtract_field_by_field_and_describe_with_the_device_index() {
        let earlier = DeviceCallCounts {
            write_calls: 11,
            written_bytes: 172_544,
            force_unit_access_writes: 1,
            barrier_calls: 2,
        };
        let later = DeviceCallCounts {
            write_calls: 22,
            written_bytes: 345_088,
            force_unit_access_writes: 2,
            barrier_calls: 4,
        };
        let delta = later.since(earlier);
        assert_eq!(
            delta,
            DeviceCallCounts {
                write_calls: 11,
                written_bytes: 172_544,
                force_unit_access_writes: 1,
                barrier_calls: 2,
            }
        );
        assert_eq!(
            delta.describe(1),
            "device_1_writes=11 device_1_written_bytes=172544 device_1_force_unit_access_writes=1 device_1_barriers=2"
        );
    }

    /// 分段标记少打一段就要被拒收：段名与登记的逐项比，少一段、次序反了都 `matches=false`。
    /// 「登记的」是 [`registered_segments_of`]，每一档写死一张表。
    #[test]
    fn segment_timing_registration_refuses_a_run_that_marked_one_segment_fewer() {
        let registered = registered_segments_of(OnDeviceRunMode::Direct);
        assert_eq!(
            registered,
            [
                "mkfs",
                "instance_acquisition",
                "warm_up",
                "first_transaction",
                "cold_reopen_and_recover"
            ]
        );

        let mut complete = SegmentClock::start();
        for segment in registered {
            let line = complete.mark(segment);
            assert!(
                line.starts_with(&format!(
                    "name=segment_timing segment={segment} nanoseconds="
                )),
                "每段末尾一行带单调时钟的标记：{line}"
            );
            assert!(line.contains(" monotonic_nanoseconds="));
        }
        let (line, matches) = complete.registration_check("direct", registered);
        assert!(matches, "五段都打了：{line}");
        assert_eq!(
            line,
            "name=segment_timing_registration mode=direct \
             registered=mkfs,instance_acquisition,warm_up,first_transaction,cold_reopen_and_recover \
             marked=mkfs,instance_acquisition,warm_up,first_transaction,cold_reopen_and_recover \
             matches=true"
        );

        let mut missing_one = SegmentClock::start();
        for segment in registered.iter().filter(|name| **name != "warm_up") {
            missing_one.mark(segment);
        }
        let (missing_line, missing_matches) = missing_one.registration_check("direct", registered);
        assert!(
            !missing_matches,
            "少打了 warm_up 那一段，必须拒收：{missing_line}"
        );
        assert!(missing_line.ends_with("matches=false"));

        let mut out_of_order = SegmentClock::start();
        for segment in ["instance_acquisition", "mkfs"] {
            out_of_order.mark(segment);
        }
        assert!(
            !out_of_order
                .registration_check("direct", &["mkfs", "instance_acquisition"])
                .1,
            "次序反了也算对不上"
        );

        // 包含自检：各段之和不超过整条路的挂钟（段首尾相接，超过就是某一段重复计了）。
        let (containment_line, contained) = complete.containment_check();
        assert!(contained, "{containment_line}");
        assert!(
            containment_line.starts_with("name=segment_timing_check segments=5 sum_nanoseconds=")
        );
    }

    /// 每一档登记的段都互不相同、而且都在这一档真会走到的那几段里：
    /// `second-instance` 比 `second-transaction` 多两段（重开可写挂载、发布 C），后者比 `direct` 多一段（发布 B）；
    /// `raise-rollback-floor` 比 `second-instance` 多两段（发布 D、抬 F）。
    #[test]
    fn every_mode_registers_its_own_segments_with_no_repeats() {
        for mode in OnDeviceRunMode::ALL {
            let registered = registered_segments_of(mode);
            let mut sorted = registered.to_vec();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), registered.len(), "{mode:?} 登记的段名有重复");
            assert_eq!(registered[0], "mkfs", "{mode:?} 第一段是 mkfs");
            assert_eq!(
                registered[registered.len() - 1],
                "cold_reopen_and_recover",
                "{mode:?} 最后一段是冷重开"
            );
        }
        assert_eq!(registered_segments_of(OnDeviceRunMode::Direct).len(), 5);
        assert_eq!(
            registered_segments_of(OnDeviceRunMode::SecondTransaction).len(),
            6
        );
        assert_eq!(
            registered_segments_of(OnDeviceRunMode::SecondInstance).len(),
            8
        );
        assert_eq!(
            registered_segments_of(OnDeviceRunMode::RaiseRollbackFloor),
            [
                "mkfs",
                "instance_acquisition",
                "warm_up",
                "first_transaction",
                "second_transaction",
                "reopen_and_writable_mount",
                "third_transaction",
                "fourth_transaction",
                "raise_rollback_floor",
                "cold_reopen_and_recover"
            ],
            "raise-rollback-floor 比 second-instance 多发布 D 与抬 F 两段，排在发布 C 之后、冷重开之前"
        );
    }

    use super::{
        describe_first_transaction_path_failure, device_call_counts,
        publish_the_fourth_version_and_describe,
        publish_the_fourth_version_and_raise_the_rollback_floor,
        publish_the_second_version_and_describe, publish_the_third_version_and_describe,
        raise_the_rollback_floor_and_describe, reopen_and_mount_writable, FailedRun,
        SecondInstanceRun,
    };
    use singlefs_harness::device_log::{expected_device_events, DeviceEvent};
    use singlefs_harness::fault_injection::{FaultSchedule, InjectedFault};
    use singlefs_harness::scenario::{FirstTransactionPathStep, ScenarioPoint};

    fn geometry_of(
        parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
    ) -> FixedGeometry {
        FixedGeometry {
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
            root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
        }
    }

    /// 两块稀疏内存盘，外面各包一层录制器、再包一层数调用的注入包装（与虚机里那一套同形）。
    fn counted_sparse_devices(
        stream: &SharedStream,
        plan: &SharedFaultPlan,
    ) -> Vec<(DeviceIdentity, CountedDevice<SparseBlockDevice>)> {
        (0..2u32)
            .map(|device_number| {
                let identity = DeviceIdentity(device_number);
                (
                    identity,
                    FaultInjectingBlockDevice::new(
                        identity,
                        RecordingBlockDevice::with_shared_stream(
                            identity,
                            SparseBlockDevice::new(
                                SPARSE_DEVICE_BYTES,
                                PhysicalBlockSizeInBytes(512),
                            ),
                            stream.clone(),
                        ),
                        plan.clone(),
                    ),
                )
            })
            .collect()
    }

    /// 宿主上的「冷重开」：镜像交给新句柄。
    fn reopen_sparse_devices(
        closed_devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    ) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
        closed_devices
            .into_iter()
            .map(|(identity, closed)| {
                let mut reopened =
                    SparseBlockDevice::new(SPARSE_DEVICE_BYTES, PhysicalBlockSizeInBytes(512));
                reopened.image = closed.image;
                (identity, reopened)
            })
            .collect()
    }

    fn lines_named<'lines>(lines: &'lines [String], name: &str) -> Vec<&'lines str> {
        lines
            .iter()
            .map(String::as_str)
            .filter(|line| field(line, "name") == Some(name))
            .collect()
    }

    fn failed_run_of<Success>(result: Result<Success, FailedRun>, what: &str) -> FailedRun {
        match result {
            Ok(_) => panic!("{what}：注入的写错该让它失败"),
            Err(failed) => failed,
        }
    }

    /// 失败那一段窗口的结果行：同一段里已经落盘的发布各一行（`persisted_write_calls` 按先后，各自的写调用数）、失败账一份
    /// （`failed_write_calls` 条写已落盘）、两样相加与设备一层逐项相等（`window_write_calls` 条），停下的原因是注入的写错。
    fn assert_the_failed_window_is_reconciled(
        lines: &[String],
        window: &str,
        persisted_write_calls: &[&str],
        failed_write_calls: &str,
        window_write_calls: &str,
    ) {
        let persisted_writes: Vec<(Option<&str>, Option<&str>, Option<&str>)> =
            lines_named(lines, "persisted_publish_writes")
                .into_iter()
                .map(|line| {
                    (
                        field(line, "window"),
                        field(line, "publish"),
                        field(line, "write_calls"),
                    )
                })
                .collect();
        let expected_persisted_writes: Vec<(Option<&str>, Option<&str>, Option<&str>)> =
            persisted_write_calls
                .iter()
                .enumerate()
                .map(|(index, write_calls)| {
                    (
                        Some(window),
                        Some(["1", "2", "3"][index]),
                        Some(*write_calls),
                    )
                })
                .collect();
        assert_eq!(
            persisted_writes, expected_persisted_writes,
            "同一段里失败之前已经落盘的发布各一行：{lines:#?}"
        );
        let failed_writes = lines_named(lines, "failed_publish_writes");
        assert_eq!(failed_writes.len(), 1, "失败账一份：{lines:#?}");
        assert_eq!(field(failed_writes[0], "window"), Some(window));
        assert_eq!(field(failed_writes[0], "failure"), Some("1"));
        assert_eq!(
            field(failed_writes[0], "write_calls"),
            Some(failed_write_calls),
            "失败之前已落盘的写：{}",
            failed_writes[0]
        );
        let failed_window = format!("{window}_failed");
        let windows: Vec<&str> = lines_named(lines, "publish_writes_against_device")
            .into_iter()
            .filter(|line| field(line, "window") == Some(failed_window.as_str()))
            .collect();
        assert_eq!(windows.len(), 1, "{lines:#?}");
        let publishes = (persisted_write_calls.len() + 1).to_string();
        assert_eq!(field(windows[0], "publishes"), Some(publishes.as_str()));
        assert_eq!(
            field(windows[0], "by_kind_write_calls"),
            Some(window_write_calls)
        );
        assert_eq!(
            field(windows[0], "device_write_calls"),
            Some(window_write_calls)
        );
        assert_eq!(
            field(windows[0], "by_kind_written_bytes"),
            field(windows[0], "device_written_bytes")
        );
        assert_eq!(
            field(windows[0], "matches"),
            Some("true"),
            "写入口交得出的账与设备一层逐项相等：{}",
            windows[0]
        );
        let stopped = lines_named(lines, "run_failed");
        assert_eq!(stopped.len(), 1, "{lines:#?}");
        assert_eq!(field(stopped[0], "step"), Some(window));
        assert!(
            stopped[0].contains("注入的写错"),
            "停下的原因是注入的那一次写错：{}",
            stopped[0]
        );
    }

    /// 增补 2 收口表第 58 行，发布 B 那一段：整池第 5 次写报错，前 4 次（头两个单元、每块盘各一份）已落盘。
    /// 二进制不直接退出：失败账那 4 次写照打，与设备一层数到的逐项相等，再打停下的原因。
    #[test]
    fn second_version_publish_failing_midway_reports_the_writes_it_landed_and_they_equal_the_device_layer_count(
    ) {
        let parameters = e142_parameters(512, 512);
        let geometry = geometry_of(&parameters);
        let stream = SharedStream::new();
        let plan = SharedFaultPlan::unarmed(geometry);
        let mut devices = counted_sparse_devices(&stream, &plan);
        let run = run_first_transaction(&parameters, &mut devices, &stream, |_point, _devices| {})
            .expect("第一个事务");
        plan.arm(FaultSchedule::the_nth_call_across_the_pool(
            InjectedFault::WriteFails,
            5,
        ));
        let failed = failed_run_of(
            publish_the_second_version_and_describe(
                &parameters,
                &mut devices,
                &plan,
                &stream,
                &geometry,
                &run.output,
            ),
            "发布 B",
        );
        assert_the_failed_window_is_reconciled(&failed.lines, "second_transaction", &[], "4", "4");
        assert!(
            lines_named(&failed.lines, "second_transaction").is_empty(),
            "失败的发布不打成功那一行"
        );
    }

    /// 增补 2 收口表第 58 行，发布 C 那一段：可写挂载做完之后装上注入，发布 C 整池第 5 次写报错。
    /// 挂载那几行照打在前面，失败账 4 次写与设备一层（重开之后那个注入计划数的）逐项相等。
    #[test]
    fn third_version_publish_failing_midway_reports_the_writes_it_landed_and_they_equal_the_device_layer_count(
    ) {
        let parameters = e142_parameters(512, 512);
        let geometry = geometry_of(&parameters);
        let stream = SharedStream::new();
        let plan = SharedFaultPlan::unarmed(geometry);
        let mut devices = counted_sparse_devices(&stream, &plan);
        let run = run_first_transaction(&parameters, &mut devices, &stream, |_point, _devices| {})
            .expect("第一个事务");
        let mut allocator = allocator_rebuilt_from_the_records_of(&devices, &run.output);
        publish_the_second_version(&parameters, &mut devices, &mut allocator, &run.output)
            .expect("发布 B");
        let mut clock = SegmentClock::start();
        let mounted = match reopen_and_mount_writable(
            &parameters,
            devices,
            &stream,
            &geometry,
            SharedFaultPlan::unarmed(geometry),
            &mut clock,
            reopen_sparse_devices,
        ) {
            Ok(mounted) => mounted,
            Err(failed) => panic!("可写挂载：{failed:?}"),
        };
        assert!(mounted.mount_window_matches_device);
        mounted
            .plan
            .arm(FaultSchedule::the_nth_call_across_the_pool(
                InjectedFault::WriteFails,
                5,
            ));
        let failed = failed_run_of(
            publish_the_third_version_and_describe(
                &parameters,
                mounted,
                &stream,
                &geometry,
                &mut clock,
            ),
            "发布 C",
        );
        assert_eq!(
            lines_named(&failed.lines, "writable_mount").len(),
            1,
            "挂载那一行在失败那几行前面照打：{:#?}",
            failed.lines
        );
        assert_the_failed_window_is_reconciled(&failed.lines, "third_transaction", &[], "4", "4");
        assert!(lines_named(&failed.lines, "third_transaction").is_empty());
    }

    /// 第一个事务那条路：第一个事务整池第 5 次写报错 ⇒ 失败账 4 次写，与设备一层逐项相等。
    /// 暖机第二次空发布的根槽写（整池第 8 次写：第一次空发布 5 次，第二次的两条 journal 记录之后）报错 ⇒
    /// 已经落盘的第一次空发布 5 次写由 `WarmUpFailed` 交回、失败账第二次那 2 次写，两样相加 7 次与设备一层逐项相等
    /// （增补 2 收口表第 58 行：此前第一次的账随错丢掉、只打两边的数不判）。
    #[test]
    fn first_transaction_path_failures_report_every_publish_of_the_failed_step_and_they_equal_the_device_layer_count(
    ) {
        let parameters = e142_parameters(512, 512);
        let geometry = geometry_of(&parameters);
        for (armed_at, failing_write, step) in [
            (
                ScenarioPoint::BeforeFirstTransaction,
                5,
                FirstTransactionPathStep::FirstTransaction,
            ),
            (
                ScenarioPoint::AfterInstanceAcquisition,
                8,
                FirstTransactionPathStep::WarmUp,
            ),
        ] {
            let stream = SharedStream::new();
            let plan = SharedFaultPlan::unarmed(geometry);
            let mut devices = counted_sparse_devices(&stream, &plan);
            let mut counts_after_acquisition: Option<Vec<DeviceCallCounts>> = None;
            let mut counts_before_first_transaction: Option<Vec<DeviceCallCounts>> = None;
            let result =
                run_first_transaction(&parameters, &mut devices, &stream, |point, devices| {
                    let counts = device_call_counts(devices, &plan);
                    let arm_here = matches!(
                        (&point, &armed_at),
                        (
                            ScenarioPoint::AfterInstanceAcquisition,
                            ScenarioPoint::AfterInstanceAcquisition
                        ) | (
                            ScenarioPoint::BeforeFirstTransaction,
                            ScenarioPoint::BeforeFirstTransaction
                        )
                    );
                    match point {
                        ScenarioPoint::AfterMakeFilesystem => {}
                        ScenarioPoint::AfterInstanceAcquisition => {
                            counts_after_acquisition = Some(counts);
                        }
                        ScenarioPoint::BeforeFirstTransaction => {
                            counts_before_first_transaction = Some(counts);
                        }
                    }
                    if arm_here {
                        plan.arm(FaultSchedule::the_nth_call_across_the_pool(
                            InjectedFault::WriteFails,
                            failing_write,
                        ));
                    }
                });
            let failure = match result {
                Ok(_) => panic!("{step:?}：注入的写错该让整条路停下"),
                Err(failure) => failure,
            };
            assert_eq!(failure.failed_step, step);
            let lines = describe_first_transaction_path_failure(
                &failure,
                counts_after_acquisition.as_deref(),
                counts_before_first_transaction.as_deref(),
                &device_call_counts(&devices, &plan),
            );
            match step {
                FirstTransactionPathStep::FirstTransaction => {
                    assert_the_failed_window_is_reconciled(
                        &lines,
                        "first_transaction",
                        &[],
                        "4",
                        "4",
                    );
                }
                FirstTransactionPathStep::WarmUp => {
                    assert_the_failed_window_is_reconciled(&lines, "warm_up", &["5"], "2", "7");
                }
                FirstTransactionPathStep::MakeFilesystem
                | FirstTransactionPathStep::InstanceAcquisition => {
                    unreachable!("这两格没有摆")
                }
            }
        }
    }

    /// 可写挂载失败（增补 2 收口表第 58 行）：挂载把这次写入口的账随 `MountError::Publish` 交回，窗口从取号那道屏障算起，
    /// 与设备一层逐项相等。发布 B 之后冷重开，注入摆在重开之后整池第 N 次写（取号那两次系统配置槽写是第 1、2 次）：
    /// 第 3 次是写行那次发布的第一个写——没有已经落盘的发布、失败账 0 次写、窗口 0 次；
    /// 第 33 次是第二次暖机的第 3 个写——写行 15 次与第一次暖机 13 次已经落盘、失败账 2 次写、窗口 30 次。
    #[test]
    fn failed_writable_mount_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count()
    {
        let parameters = e142_parameters(512, 512);
        let geometry = geometry_of(&parameters);
        for (failing_write, persisted_write_calls, failed_write_calls, window_write_calls) in
            [(3, &[][..], "0", "0"), (33, &["15", "13"][..], "2", "30")]
        {
            let stream = SharedStream::new();
            let plan = SharedFaultPlan::unarmed(geometry);
            let mut devices = counted_sparse_devices(&stream, &plan);
            let run =
                run_first_transaction(&parameters, &mut devices, &stream, |_point, _devices| {})
                    .expect("第一个事务");
            let mut allocator = allocator_rebuilt_from_the_records_of(&devices, &run.output);
            publish_the_second_version(&parameters, &mut devices, &mut allocator, &run.output)
                .expect("发布 B");
            let mut clock = SegmentClock::start();
            let failed = failed_run_of(
                reopen_and_mount_writable(
                    &parameters,
                    devices,
                    &stream,
                    &geometry,
                    SharedFaultPlan::armed(
                        geometry,
                        FaultSchedule::the_nth_call_across_the_pool(
                            InjectedFault::WriteFails,
                            failing_write,
                        ),
                    ),
                    &mut clock,
                    reopen_sparse_devices,
                ),
                "可写挂载",
            );
            assert_the_failed_window_is_reconciled(
                &failed.lines,
                "reopen_and_writable_mount",
                persisted_write_calls,
                failed_write_calls,
                window_write_calls,
            );
        }
    }

    /// `second-instance` 那条路在宿主上走到发布 C（实例 2，txg 8）：稀疏内存盘代替 virtio 盘，「冷重开」把镜像交给新句柄。
    /// 交回这一段的注入计划还没装注入。
    fn second_instance_run_up_to_the_third_version(
        parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
        geometry: &FixedGeometry,
        stream: &SharedStream,
        clock: &mut SegmentClock,
    ) -> SecondInstanceRun<SparseBlockDevice> {
        let plan = SharedFaultPlan::unarmed(*geometry);
        let mut devices = counted_sparse_devices(stream, &plan);
        let run = run_first_transaction(parameters, &mut devices, stream, |_point, _devices| {})
            .expect("第一个事务");
        let mut allocator = allocator_rebuilt_from_the_records_of(&devices, &run.output);
        publish_the_second_version(parameters, &mut devices, &mut allocator, &run.output)
            .expect("发布 B");
        let switched = match switch_instance_and_publish_third_version(
            parameters,
            devices,
            stream,
            geometry,
            clock,
            reopen_sparse_devices,
        ) {
            Ok(switched) => switched,
            Err(failed) => panic!("第二个实例：{failed:?}"),
        };
        assert_eq!(
            switched.current_version.root.checkpoint_txg,
            CheckpointTxg(8),
            "发布 C"
        );
        switched
    }

    /// `raise-rollback-floor` 模式在宿主上照同一条路跑一遍（增补 2 收口表第 58 行「二进制那一侧判相等」，成功那一路）：
    /// 发布 C（txg 8）之后同一次挂载里发布 D（txg 9，第四版），再把 F 抬到上限。发布 D 之后按新到旧数非空有效根是 D、C、B、A（txg 9、8、4、3），
    /// 上限 = min(每块盘上最新的有效根, 第 4 新的非空有效根) = min(盘 1 的 txg 7, txg 3) = 3（D16（发布语义） 已定项 1「抬 F 的上限」；
    /// 根环三个区域的设备归属是 0 / 1 / 0，按 txg 轮流落，盘 0 上最新的是 txg 9、盘 1 上是 txg 7）。推两次带 F = 3 的空发布 txg 10、11，
    /// 两块盘各落一条（生效）。F 的两个数排在最前：没有发布 D 时第 4 新的非空有效根够不到 A，上限是 0，红在这里。
    /// 发布 D 一行、抬 F 一行、三次发布各一行 `name=publish_writes`、两段窗口按种类的合计与设备一层逐项相等；录制流投到每块盘上的写与 FLUSH
    /// 与设备一层数到的相等（宿主检查拿这个投法当程序的信念，发布 D 与抬 F 两段也一样）；冷恢复择 (2, 11) 读回第四版。
    #[test]
    fn raise_rollback_floor_mode_publishes_the_fourth_version_raises_the_floor_to_its_ceiling_above_zero_and_every_window_matches_the_device_layer_count(
    ) {
        let parameters = e142_parameters(512, 512);
        let geometry = geometry_of(&parameters);
        let stream = SharedStream::new();
        let mut clock = SegmentClock::start();
        let switched = second_instance_run_up_to_the_third_version(
            &parameters,
            &geometry,
            &stream,
            &mut clock,
        );
        let lines_before_the_fourth_version = switched.lines.len();
        let operations_before_the_fourth_version = stream.operation_count();
        let counts_before_the_fourth_version =
            device_call_counts(&switched.devices, &switched.plan);
        let raised = match publish_the_fourth_version_and_raise_the_rollback_floor(
            &parameters,
            switched,
            &stream,
            &geometry,
            &mut clock,
        ) {
            Ok(raised) => raised,
            Err(failed) => panic!("发布 D、抬 F：{failed:?}"),
        };
        let new_lines = &raised.lines[lines_before_the_fourth_version..];

        let raise_line = lines_named(new_lines, "raise_rollback_floor");
        assert_eq!(raise_line.len(), 1, "{new_lines:#?}");
        let raise_line = raise_line[0];
        assert_eq!(
            field(raise_line, "ceiling"),
            Some("3"),
            "第 4 新的非空有效根是 A（txg 3），比盘 1 上最新的 txg 7 小：{raise_line}"
        );
        assert_eq!(
            field(raise_line, "requested_floor"),
            Some("3"),
            "F 抬到上限：{raise_line}"
        );
        assert_eq!(raised.current_version.root.rollback_floor, CheckpointTxg(3));
        assert_eq!(field(raise_line, "floor_before"), Some("0"), "{raise_line}");
        assert_eq!(field(raise_line, "publishes"), Some("2"), "{raise_line}");
        assert_eq!(
            field(raise_line, "root_txgs"),
            Some("10,11"),
            "{raise_line}"
        );
        assert_eq!(
            field(raise_line, "reclaimed"),
            Some("1"),
            "释放代 ≤ 3 的已释放落点（两盘同槽，按盘 0 报）：{raise_line}"
        );
        assert_eq!(
            raised.current_version.root.checkpoint_txg,
            CheckpointTxg(11)
        );

        let fourth_lines = lines_named(new_lines, "fourth_transaction");
        assert_eq!(fourth_lines.len(), 1, "{new_lines:#?}");
        assert_eq!(field(fourth_lines[0], "root_txg"), Some("9"));
        assert_eq!(
            field(fourth_lines[0], "transaction"),
            Some("2"),
            "同一个实例里发布 C 之后的下一个事务号"
        );
        assert_eq!(
            field(fourth_lines[0], "segments"),
            Some("16+2+1+2"),
            "发布 D 与发布 C 同型"
        );

        let publish_lines: Vec<(Option<&str>, Option<&str>)> =
            lines_named(new_lines, "publish_writes")
                .into_iter()
                .map(|line| (field(line, "publish"), field(line, "txg")))
                .collect();
        assert_eq!(
            publish_lines,
            vec![
                (Some("fourth_transaction"), Some("9")),
                (Some("raise_rollback_floor"), Some("10")),
                (Some("raise_rollback_floor"), Some("11")),
            ]
        );
        let windows = lines_named(new_lines, "publish_writes_against_device");
        let window_summary: Vec<(Option<&str>, Option<&str>, Option<&str>)> = windows
            .iter()
            .map(|line| {
                (
                    field(line, "window"),
                    field(line, "publishes"),
                    field(line, "matches"),
                )
            })
            .collect();
        assert_eq!(
            window_summary,
            vec![
                (Some("fourth_transaction"), Some("1"), Some("true")),
                (Some("raise_rollback_floor"), Some("2"), Some("true")),
            ],
            "发布 D 与抬 F 那一串空发布按种类的合计都等于设备一层数的：{windows:#?}"
        );
        for window in &windows {
            assert_eq!(
                field(window, "by_kind_write_calls"),
                field(window, "device_write_calls"),
                "{window}"
            );
            assert_eq!(
                field(window, "by_kind_written_bytes"),
                field(window, "device_written_bytes"),
                "{window}"
            );
        }
        assert!(raised.every_window_matches_device);
        assert_eq!(
            clock.marked_segments(),
            [
                "reopen_and_writable_mount",
                "third_transaction",
                "fourth_transaction",
                "raise_rollback_floor"
            ]
        );
        assert_eq!(
            lines_named(new_lines, "segment_timing").len(),
            2,
            "发布 D 与抬 F 各自的分段行：{new_lines:#?}"
        );

        let operations = stream.retained_operations();
        let new_operations = &operations[operations_before_the_fourth_version..];
        let counts_after_the_raise = device_call_counts(&raised.devices, &raised.plan);
        for (index, identity) in [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .enumerate()
        {
            let counted =
                counts_after_the_raise[index].since(counts_before_the_fourth_version[index]);
            let projected = expected_device_events(new_operations, identity);
            let projected_writes = projected
                .iter()
                .filter(|event| matches!(event, DeviceEvent::Write { .. }))
                .count();
            let projected_flushes = projected
                .iter()
                .filter(|event| matches!(event, DeviceEvent::Flush))
                .count();
            assert!(
                counted.write_calls > 0,
                "发布 D 与抬 F 两段盘 {} 收到了写",
                identity.0
            );
            assert_eq!(
                u64::try_from(projected_writes).expect("件数"),
                counted.write_calls,
                "发布 D 与抬 F 两段盘 {}：录制流投出的写与设备一层数到的",
                identity.0
            );
            assert_eq!(
                u64::try_from(projected_flushes).expect("件数"),
                counted.barrier_calls + counted.force_unit_access_writes,
                "发布 D 与抬 F 两段盘 {}：录制流投出的 FLUSH 与设备一层转发的屏障 + FUA 写",
                identity.0
            );
        }

        let cold: Vec<(DeviceIdentity, SparseBlockDevice)> = raised
            .devices
            .into_iter()
            .map(|(identity, device)| (identity, device.into_inner().into_inner_and_operations().0))
            .collect();
        let report = recover(&cold, JournalPolicy::Consult);
        match report.outcome {
            RecoveryOutcome::FileRead { root, content } => {
                assert_eq!(root, (InstanceGeneration(2), CheckpointTxg(11)));
                assert!(content == fourth_file_content(), "冷恢复读回第四版");
            }
            RecoveryOutcome::NoFile { root } => panic!("冷恢复择到 {root:?} 却没有文件"),
            RecoveryOutcome::Failed { root, failure } => {
                panic!("冷恢复失败：{root:?} {failure:?}")
            }
        }
    }

    /// 代码轮第二轮判决 Z10（`research/prompts/m2-final-code-r2-main-verification.md` 第四节第 5 条）：`raise-rollback-floor` 那条路在宿主上走到
    /// 抬 F 之后冷重开，多打的那一行 `name=recover_cold_rollback_floor` 报的是冷重开择到的根 (2, 11) 与它从盘上读回的 F = 3，
    /// 不是 `name=raise_rollback_floor` 里内存那一版的 `requested_floor`。同一段历史抬 F 之前的盘面上（发布 C 之后、择到 (2, 8)）读回的是 0：
    /// 这一行跟着盘上的根走，不是写死的数。
    #[test]
    fn raise_rollback_floor_mode_prints_the_floor_read_back_from_the_chosen_root_after_the_cold_reopen(
    ) {
        let parameters = e142_parameters(512, 512);
        let geometry = geometry_of(&parameters);
        let stream = SharedStream::new();
        let mut clock = SegmentClock::start();
        let switched = second_instance_run_up_to_the_third_version(
            &parameters,
            &geometry,
            &stream,
            &mut clock,
        );
        let before_the_raise: Vec<(DeviceIdentity, SparseBlockDevice)> = switched
            .devices
            .iter()
            .map(|(identity, device)| {
                let mut copy =
                    SparseBlockDevice::new(SPARSE_DEVICE_BYTES, PhysicalBlockSizeInBytes(512));
                copy.image = device.inner().inner().image.clone();
                (*identity, copy)
            })
            .collect();
        let line_before_the_raise =
            super::rollback_floor_read_back_from_the_chosen_root_line(&before_the_raise);
        assert_eq!(
            field(&line_before_the_raise, "chosen_root"),
            Some("2:8"),
            "发布 C 之后择到 (2, 8)：{line_before_the_raise}"
        );
        assert_eq!(
            field(&line_before_the_raise, "rollback_floor_on_disk"),
            Some("0"),
            "抬 F 之前盘上的 F 是 0：{line_before_the_raise}"
        );
        let raised = match publish_the_fourth_version_and_raise_the_rollback_floor(
            &parameters,
            switched,
            &stream,
            &geometry,
            &mut clock,
        ) {
            Ok(raised) => raised,
            Err(failed) => panic!("发布 D、抬 F：{failed:?}"),
        };
        let cold: Vec<(DeviceIdentity, SparseBlockDevice)> = raised
            .devices
            .into_iter()
            .map(|(identity, device)| (identity, device.into_inner().into_inner_and_operations().0))
            .collect();
        let line = super::rollback_floor_read_back_from_the_chosen_root_line(&cold);
        assert_eq!(
            field(&line, "name"),
            Some("recover_cold_rollback_floor"),
            "{line}"
        );
        assert_eq!(
            field(&line, "chosen_root"),
            Some("2:11"),
            "冷重开择到最后一次抬 F 的空发布的根：{line}"
        );
        assert_eq!(
            field(&line, "rollback_floor_on_disk"),
            Some("3"),
            "所选根从盘上读回的 F 是抬到的上限 3：{line}"
        );
    }

    /// 增补 2 收口表第 58 行，发布 D 那一段（`raise-rollback-floor`）：发布 C 之后装上注入，发布 D 整池第 5 次写报错。
    /// 挂载与发布 C 那几行照打在前面，失败账 4 次写与设备一层（重开之后那个注入计划数的）逐项相等；发布 D 没做成就不抬 F。
    #[test]
    fn fourth_version_publish_failing_midway_reports_the_writes_it_landed_and_they_equal_the_device_layer_count(
    ) {
        let parameters = e142_parameters(512, 512);
        let geometry = geometry_of(&parameters);
        let stream = SharedStream::new();
        let mut clock = SegmentClock::start();
        let switched = second_instance_run_up_to_the_third_version(
            &parameters,
            &geometry,
            &stream,
            &mut clock,
        );
        switched
            .plan
            .arm(FaultSchedule::the_nth_call_across_the_pool(
                InjectedFault::WriteFails,
                5,
            ));
        let failed = failed_run_of(
            publish_the_fourth_version_and_raise_the_rollback_floor(
                &parameters,
                switched,
                &stream,
                &geometry,
                &mut clock,
            ),
            "发布 D",
        );
        for earlier_line in ["writable_mount", "third_transaction"] {
            assert_eq!(
                lines_named(&failed.lines, earlier_line).len(),
                1,
                "{earlier_line} 那一行在失败那几行前面照打：{:#?}",
                failed.lines
            );
        }
        assert_the_failed_window_is_reconciled(&failed.lines, "fourth_transaction", &[], "4", "4");
        assert!(lines_named(&failed.lines, "fourth_transaction").is_empty());
        assert!(
            lines_named(&failed.lines, "raise_rollback_floor").is_empty(),
            "发布 D 没做成就不抬 F：{:#?}",
            failed.lines
        );
        assert!(failed.cause.starts_with("发布 D："), "{}", failed.cause);
    }

    /// 抬 F 失败（增补 2 收口表第 58 行「`raise_rollback_floor` 同形」）：抬 F 的写入口是它自己开的、随错丢掉，
    /// 已经落盘的那几次空发布与失败那一次已记的写随 `MountError::RaiseFloorSequencePublishFailed` 交回；`raise-rollback-floor`
    /// 模式走的那个函数（`raise_the_rollback_floor_and_describe`）照可写挂载那一段的判法打失败账、与设备一层逐项比。
    /// `raise-rollback-floor` 那条路走到发布 D（txg 9），装上注入，把 F 抬到上限（3）——推空发布直到两块盘上都有一条带这个 F 的根。
    /// 注入摆在之后整池第 16 次写：第一次空发布（txg 10）13 次写已经落盘（四个固定点单元与 journal 记录每盘一份、根槽一次、系统配置每盘一次），
    /// 第二次的第 3 个写报错 ⇒ 失败账 2 次写，两样相加 15 次与设备一层逐项相等。前面几段的结果行照打在失败那几行前面，成功那一行不打。
    #[test]
    fn failed_raise_of_the_rollback_floor_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count(
    ) {
        let parameters = e142_parameters(512, 512);
        let geometry = geometry_of(&parameters);
        let stream = SharedStream::new();
        let mut clock = SegmentClock::start();
        let switched = second_instance_run_up_to_the_third_version(
            &parameters,
            &geometry,
            &stream,
            &mut clock,
        );
        let published_the_fourth_version = match publish_the_fourth_version_and_describe(
            &parameters,
            switched,
            &stream,
            &geometry,
            &mut clock,
        ) {
            Ok(published) => published,
            Err(failed) => panic!("发布 D：{failed:?}"),
        };
        published_the_fourth_version
            .plan
            .arm(FaultSchedule::the_nth_call_across_the_pool(
                InjectedFault::WriteFails,
                16,
            ));
        let failed = failed_run_of(
            raise_the_rollback_floor_and_describe(
                &parameters,
                published_the_fourth_version,
                &stream,
                &geometry,
                &mut clock,
            ),
            "抬 F",
        );
        for earlier_line in ["writable_mount", "third_transaction", "fourth_transaction"] {
            assert_eq!(
                lines_named(&failed.lines, earlier_line).len(),
                1,
                "{earlier_line} 那一行在失败那几行前面照打：{:#?}",
                failed.lines
            );
        }
        assert_the_failed_window_is_reconciled(
            &failed.lines,
            "raise_rollback_floor",
            &["13"],
            "2",
            "15",
        );
        assert!(
            lines_named(&failed.lines, "raise_rollback_floor").is_empty(),
            "失败的抬 F 不打成功那一行"
        );
        assert!(failed.cause.starts_with("抬 F："), "{}", failed.cause);
    }

    /// 宿主检查（`first_transaction_device_log_check`）拿录制流投到每块盘上的事件当「程序的信念」：每个写一件、FUA 写之后一个 FLUSH、
    /// 每道池屏障在每块盘上各一个 FLUSH（录制器把连续几道并成一道）。这个投法在发布 B、可写挂载、发布 C 三段上是否就是设备收到的，
    /// 拿录制器外面那一层（注入包装，与录制器不共享计数）数到的核：每块盘、每段，写的件数相等，FLUSH 件数 = 转发的屏障 + FUA 写。
    /// 录制器并掉的屏障若在设备上是两次，这里先红，虚机档的逐项比对不会冤判。
    #[test]
    fn the_recorded_stream_projects_onto_each_device_the_writes_and_flushes_the_device_layer_counted(
    ) {
        let parameters = e142_parameters(512, 512);
        let geometry = geometry_of(&parameters);
        let stream = SharedStream::new();
        let plan = SharedFaultPlan::unarmed(geometry);
        let mut devices = counted_sparse_devices(&stream, &plan);
        let run = run_first_transaction(&parameters, &mut devices, &stream, |_point, _devices| {})
            .expect("第一个事务");
        let operations_after_the_first_transaction = stream.operation_count();
        let counts_after_the_first_transaction = device_call_counts(&devices, &plan);
        let second = publish_the_second_version_and_describe(
            &parameters,
            &mut devices,
            &plan,
            &stream,
            &geometry,
            &run.output,
        );
        assert!(second.is_ok_and(|second| second.window_matches_device));
        let operations_after_the_second_transaction = stream.operation_count();
        let counts_after_the_second_transaction = device_call_counts(&devices, &plan);
        let mut clock = SegmentClock::start();
        let switched = match switch_instance_and_publish_third_version(
            &parameters,
            devices,
            &stream,
            &geometry,
            &mut clock,
            reopen_sparse_devices,
        ) {
            Ok(switched) => switched,
            Err(failed) => panic!("第二个实例：{failed:?}"),
        };
        let operations = stream.retained_operations();
        let windows = [
            (
                "second_transaction",
                &operations[operations_after_the_first_transaction
                    ..operations_after_the_second_transaction],
            ),
            (
                "reopen_and_writable_mount_and_third_transaction",
                &operations[operations_after_the_second_transaction..],
            ),
        ];
        for (index, identity) in [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .enumerate()
        {
            let second_transaction_counted = counts_after_the_second_transaction[index]
                .since(counts_after_the_first_transaction[index]);
            let after_reopen_counted =
                DeviceCallCounts::of(switched.devices[index].1.plan().counts_of_device(identity));
            for ((window, slice), counted) in windows
                .iter()
                .zip([second_transaction_counted, after_reopen_counted])
            {
                let projected = expected_device_events(slice, identity);
                let projected_writes = projected
                    .iter()
                    .filter(|event| matches!(event, DeviceEvent::Write { .. }))
                    .count();
                let projected_flushes = projected
                    .iter()
                    .filter(|event| matches!(event, DeviceEvent::Flush))
                    .count();
                assert!(counted.write_calls > 0, "{window} 盘 {}", identity.0);
                assert_eq!(
                    u64::try_from(projected_writes).expect("件数"),
                    counted.write_calls,
                    "{window} 盘 {}：录制流投出的写与设备一层数到的",
                    identity.0
                );
                assert_eq!(
                    u64::try_from(projected_flushes).expect("件数"),
                    counted.barrier_calls + counted.force_unit_access_writes,
                    "{window} 盘 {}：录制流投出的 FLUSH 与设备一层转发的屏障 + FUA 写",
                    identity.0
                );
            }
        }
    }
}
