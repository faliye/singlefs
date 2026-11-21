//! 虚机档（QEMU/KVM）：在两块真 virtio 盘上跑新池新建文件的整条写路，冷重开再恢复读回文件。
//!
//!   new_pool_file_creation_on_device /dev/vda /dev/vdb <direct | page-cache | skip-new-pool-file-creation-barrier | file-overwrite | second-instance | raise-rollback-floor>
//!
//! 由 `scripts/devices/vm-bench.sh` 送进虚机（`VM_DISKS=2`，设备路径排在参数前面）。结果行以 `E7RESULT` 打头、
//! 末行报条数（vm-bench.sh 的完整性闸）。这个二进制只判它自己判得了的（恢复读回文件、段序列）；
//! 「盘上实际收到的是不是程序以为的」由宿主上的 `new_pool_file_creation_device_log_check` 拿 QEMU 的设备侧日志判。
//! `skip-new-pool-file-creation-barrier`：新池新建文件在盘 0 上漏掉「单元 → journal 记录」那道屏障，而录制器照样记下它——
//! 程序以为发了，盘上没收到，宿主那一侧必须判红（C6（块层语义假设写错） 的「故意去掉一次屏障」）。
//! `file-overwrite`：新池新建文件之后同一个进程里再覆盖写一次（里程碑「覆盖写、释放、回退与复用」步 1 的发布 B），分别报两次发布的挂钟与
//! 程序交给设备的写请求数、字节、屏障、FUA——E152（按里程碑对比六家文件系统的文件性能） 里程碑二那一轮的 singlefs 臂用它量「第二次写」的稳态代价；
//! 冷重开读回的是第二版。
//! `second-instance`：发布 B 之后丢掉写的那一套句柄、同一对盘冷重开，走可写挂载（恢复、取号、写行、暖机，里程碑「覆盖写、释放、回退与复用」步 3），
//! 再发布一次（发布 C）；挂载一行、发布 C 一行，写行与挂载里的每次暖机、发布 C 各一行 `name=publish_writes`，挂载与发布 C 各一段窗口行；
//! 冷重开读回的是第三版（实例 2 的根）。前四个模式打的行一行不变。
//! `raise-rollback-floor`：`second-instance` 那条路走完，同一次挂载里再覆盖写一次（发布 D，第 4 新的非空有效根落到 A 上），接着把 F 抬到上限
//! （里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 58 行：抬 F 那一串发布的账在二进制这一侧与设备一层判相等）；发布 D 与发布 C 同样打一行、一行
//! `name=publish_writes`、一段窗口行，抬 F 一行、那一串空发布各一行 `name=publish_writes`、这一段一行窗口行；失败时照发布失败那样打失败账。
//! 冷重开读回的是第四版（最后一次抬 F 的空发布的根）；`name=recover_cold` 之后多打一行 `name=recover_cold_rollback_floor`：
//! 冷重开择到的根与它从盘上读回的 F（`rollback_floor_on_disk=` Z10）。前五个模式打的行一行不变。
//! 每次发布（两次暖机、新池新建文件、`file-overwrite` 模式下的发布 B）各打一行 `name=publish_writes`：写入口按结构种类记的写调用数与写字节
//! （里程碑「覆盖写、释放、回退与复用」增补 1 第 1 件）；每段窗口（两次暖机合一段、新池新建文件、发布 B）再打一行 `name=publish_writes_against_device`，
//! 按种类的合计与设备一层数的（`FaultInjectingDevice`，两盘相加）逐项比，对不上 `matches=false`、退出码 1。
//! 发布失败时不直接退出（里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 58 行）：写入口交出来的失败账一份一行 `name=failed_publish_writes`，
//! 再把它与这段窗口里设备一层数到的写比一行，打完 `name=run_failed` 与末行条数再以退出码 5 退出。
//! 模式名、两版内容与写入时刻、发布 B 与发布 C 的写路在 `singlefs_verification::on_device_modes`，宿主上的
//! `new_pool_file_creation_device_log_check` 跑同一份，逐项比设备侧日志。

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
use singlefs_verification::fault_injection::{
    FaultCounting, FaultDeviceCounts, FaultDeviceSelector, FaultInjectingBlockDevice,
    FaultOccurrence, FaultPlacement, FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_verification::on_device_modes::{
    allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process,
    publish_the_fourth_version, publish_the_second_version, publish_the_third_version,
    raise_the_rollback_floor_to_its_ceiling, FailedPublish, OnDeviceRunMode,
};
use singlefs_verification::scenario::{
    e142_parameters, run_new_pool_file_creation, NewPoolFileCreationPathFailure,
    NewPoolFileCreationPathStep, NewPoolFileCreationRun, ScenarioPoint,
};
use singlefs_verification::segments::{
    closed_form_state_count, segment_kinds_text, segment_sizes_text, split_into_segments,
    FixedGeometry,
};
use singlefs_verification::{RecordedOperation, RecordingBlockDevice, SharedStream};
use std::time::Instant;

mod publishes_after_the_new_pool_file_creation;

use publishes_after_the_new_pool_file_creation::*;

/// 这一档该报哪几段分段时间，按次序登记。跑批脚本与门禁按这张表点数；
/// 段丢了一条、次序反了，`name=segment_timing_registration` 那一行的 `matches` 就是 false、这一档退非 0。
fn registered_segments_of(mode: OnDeviceRunMode) -> &'static [&'static str] {
    match mode {
        OnDeviceRunMode::Direct
        | OnDeviceRunMode::PageCache
        | OnDeviceRunMode::SkipNewPoolFileCreationBarrier => &[
            "mkfs",
            "instance_acquisition",
            "warm_up",
            "new_pool_file_creation",
            "cold_reopen_and_recover",
        ],
        OnDeviceRunMode::FileOverwrite => &[
            "mkfs",
            "instance_acquisition",
            "warm_up",
            "new_pool_file_creation",
            "file_overwrite",
            "cold_reopen_and_recover",
        ],
        OnDeviceRunMode::SecondInstance => &[
            "mkfs",
            "instance_acquisition",
            "warm_up",
            "new_pool_file_creation",
            "file_overwrite",
            "reopen_and_writable_mount",
            "third_transaction",
            "cold_reopen_and_recover",
        ],
        OnDeviceRunMode::RaiseRollbackFloor => &[
            "mkfs",
            "instance_acquisition",
            "warm_up",
            "new_pool_file_creation",
            "file_overwrite",
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
    publish_writes_and_writes_outside_any_publish_against_device(window, publishes, None, device)
}

/// 同 [`publish_writes_against_device`]，窗口里另有一份不属于任何一次发布的写时（抬 F 那一串第一次发布之前先把新 F 写进每块盘系统配置那一步，
/// D16（发布语义） 已定项 1「抬 F 那一串」）把它并进按种类的合计，结果行在 `publishes=` 之后多一段 `writes_outside_any_publish_write_calls=`；
/// `publishes=` 仍只数发布。`None` 时结果行与 [`publish_writes_against_device`] 逐字相同。
fn publish_writes_and_writes_outside_any_publish_against_device(
    window: &str,
    publishes: &[&WritesByStructureKind],
    writes_outside_any_publish: Option<&WritesByStructureKind>,
    device: WriteCallsAndBytes,
) -> (String, bool) {
    let by_kind = publishes
        .iter()
        .chain(writes_outside_any_publish.iter())
        .fold(WriteCallsAndBytes::NONE, |sum, writes| {
            sum.plus(writes.total())
        });
    let matches = by_kind == device;
    let outside_field = writes_outside_any_publish.map_or(String::new(), |writes| {
        format!(
            " writes_outside_any_publish_write_calls={}",
            writes.total().write_calls
        )
    });
    (
        format!(
            "name=publish_writes_against_device window={window} publishes={}{outside_field} by_kind_write_calls={} by_kind_written_bytes={} device_write_calls={} device_written_bytes={} matches={matches}",
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
    describe_failed_window_after_writes_outside_any_publish(
        window,
        cause,
        None,
        writes_of_persisted_publishes,
        writes_of_failed_publishes,
        device,
    )
}

/// 同 [`describe_failed_window`]，窗口里第一次发布之前另有一份不属于任何一次发布的写时（抬 F 那一串先写系统配置那一步）：
/// 它先打一行 `name=writes_outside_any_publish`，再并进比的那一行（[`publish_writes_and_writes_outside_any_publish_against_device`]）。
/// `None` 时打出的行与 [`describe_failed_window`] 逐字相同。
fn describe_failed_window_after_writes_outside_any_publish(
    window: &str,
    cause: &str,
    writes_outside_any_publish: Option<&WritesByStructureKind>,
    writes_of_persisted_publishes: &[WritesByStructureKind],
    writes_of_failed_publishes: &[WritesByStructureKind],
    device: WriteCallsAndBytes,
) -> (Vec<String>, bool) {
    let mut lines: Vec<String> = writes_outside_any_publish
        .iter()
        .map(|writes| {
            format!(
                "name=writes_outside_any_publish window={window} {}",
                describe_writes_by_kind(writes)
            )
        })
        .collect();
    lines.extend(
        writes_of_persisted_publishes
            .iter()
            .enumerate()
            .map(|(index, writes)| {
                format!(
                    "name=persisted_publish_writes window={window} publish={} {}",
                    index + 1,
                    describe_writes_by_kind(writes)
                )
            }),
    );
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
    let (line, matches) = publish_writes_and_writes_outside_any_publish_against_device(
        &format!("{window}_failed"),
        &publishes_in_the_window,
        writes_outside_any_publish,
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

/// 新池新建文件那条路（mkfs → 取号 → 暖机 → 新池新建文件）停下时要打的结果行。`counts_at_failure` 是停下那一刻每块盘的计数；
/// 两段发布窗口的起点是 `run_new_pool_file_creation` 在 `ScenarioPoint` 上叫回来时拍的快照（没走到那一处是 None）。
fn describe_new_pool_file_creation_path_failure(
    failure: &NewPoolFileCreationPathFailure,
    counts_after_acquisition: Option<&[DeviceCallCounts]>,
    counts_before_new_pool_file_creation: Option<&[DeviceCallCounts]>,
    counts_at_failure: &[DeviceCallCounts],
) -> Vec<String> {
    let step = failure.failed_step.name();
    match failure.failed_step {
        NewPoolFileCreationPathStep::MakeFilesystem
        | NewPoolFileCreationPathStep::InstanceAcquisition => {
            vec![describe_run_failure(step, &failure.cause)]
        }
        NewPoolFileCreationPathStep::WarmUp => {
            describe_failed_window(
                step,
                &failure.cause,
                &failure.writes_of_persisted_publishes,
                &failure.writes_of_failed_publishes,
                pool_writes_between(
                    counts_after_acquisition
                        .expect("run_new_pool_file_creation 取号之后、暖机之前叫过取号之后那一处"),
                    counts_at_failure,
                ),
            )
            .0
        }
        NewPoolFileCreationPathStep::NewPoolFileCreation => {
            describe_failed_window(
                step,
                &failure.cause,
                &failure.writes_of_persisted_publishes,
                &failure.writes_of_failed_publishes,
                pool_writes_between(
                    counts_before_new_pool_file_creation
                        .expect("run_new_pool_file_creation 新池新建文件之前叫过那一处"),
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
/// `singlefs_verification::fault_injection`）。
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
/// （`.claude/singlefs-ai-sop/rules/show-me-test.md`「分段计时在被测进程里计，不在转发输出的循环里计」——
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

/// 录制器在外、按需注入的盘在里：录制流记的是程序发给这块盘的每一次调用，故障注入只决定这次调用有没有转发给
/// 它下面那块真设备——被吞掉的调用因此进录制流、不进真设备实际收到的那一份（里程碑「覆盖写、释放、回退与复用」收尾批
/// 「55 号装置包装次序」；模块头那句「包装的位置在录制器外面」说的是 `history.rs::HistoryDevice` 那一条独立的栈，
/// 不是这个二进制自己的 `CountedDevice`）。
type CountedDevice<Inner> = RecordingBlockDevice<FaultInjectingBlockDevice<Inner>>;

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
        PoolVersion::WithFile(file_version) | PoolVersion::WithTreesAndNoFile(file_version) => {
            &file_version.writes
        }
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
struct FileOverwriteRun {
    lines: Vec<String>,
    window_matches_device: bool,
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

/// 同一次挂载里一次覆盖写成功之后：这一版、这一段的结果行（发布一行、`name=publish_writes` 一行、窗口行一行）、
/// 这一段按种类的合计与设备一层是否相等。
struct OverwriteInTheMountedInstance {
    version: TransactionOutput,
    lines: Vec<String>,
    window_matches_device: bool,
}

/// 五段路径：mkfs、取号、暖机、新池新建文件、mkfs 之后整条。取号那一截取 mkfs 之后 [`ACQUISITION_OPERATIONS_RECORDED_AFTER_MKFS`] 步
/// （取号写两轮，最后一轮之后那道屏障归暖机）。暖机与事务的分界是 `operation_count_before_new_pool_file_creation`
/// ——`ScenarioPoint::BeforeNewPoolFileCreation` 处量到的流长度，暖机 `warm_up()` 已调完、`publish_first_file()`
/// 还没调，与 `new_pool_file_creation_publish.rs` 的 `warm_up_operation_count` 量的是同一处；
/// 不是从流尾往回数几步（里程碑「覆盖写、释放、回退与复用」收尾批「55 号装置步数」；`crates/mutations.tsv` 里点名这个函数的那条变异钉住这一点）。
fn new_pool_file_creation_paths(
    operations: &[RecordedOperation],
    mkfs_operation_count: usize,
    operation_count_before_new_pool_file_creation: usize,
) -> [(&'static str, &[RecordedOperation]); 5] {
    [
        ("mkfs", &operations[..mkfs_operation_count]),
        (
            "instance_acquisition",
            &operations[mkfs_operation_count
                ..mkfs_operation_count + ACQUISITION_OPERATIONS_RECORDED_AFTER_MKFS],
        ),
        (
            "warm_up",
            &operations[mkfs_operation_count + ACQUISITION_OPERATIONS_RECORDED_AFTER_MKFS
                ..operation_count_before_new_pool_file_creation],
        ),
        (
            "transaction",
            &operations[operation_count_before_new_pool_file_creation..],
        ),
        ("post_mkfs_stream", &operations[mkfs_operation_count..]),
    ]
}

#[allow(
    clippy::too_many_lines,
    reason = "虚机里只跑这一件事：写、数、重开、读回，结果行按次序打"
)]
/// 一趟虚机跑从 `main` 解析好的参数：两块盘的路径与模式参数的原文（结果行与分段登记照打原文）。
struct OnDeviceRunArguments {
    device_paths: [String; 2],
    mode_argument: String,
}

/// 新池新建文件之前要不要在盘 0 上吞掉下一道屏障：`skip-new-pool-file-creation-barrier` 那一档的入口交
/// [`swallow_the_next_barrier_on`]，别的档交 `None`（吞屏障那个函数只进那一档的闭包）。
type BarrierFaultBeforeTheNewPoolFileCreation = Option<fn(DeviceIdentity) -> FaultSchedule>;

/// 冷重开之后另打一行的读法：`raise-rollback-floor` 那一档的入口交 [`rollback_floor_read_back_from_the_chosen_root_line`]，别的档交 `None`。
type LineReadBackAfterTheColdReopen = Option<fn(&dyn PoolReader) -> String>;

/// 新池新建文件那一段跑完、它的账都打完之后这一趟的状态：各档的入口接着跑自己那几次发布，最后都交
/// [`finish_with_the_cold_reopen`]（冷重开、恢复、分段挂钟）。
struct OnDeviceRun {
    mode: OnDeviceRunMode,
    mode_argument: String,
    device_paths: [String; 2],
    policy: PageCachePolicy,
    emitter: Emitter,
    clock: SegmentClock,
    segment_timing_lines: Vec<String>,
    parameters: MakeFilesystemParameters,
    stream: SharedStream,
    plan: SharedFaultPlan,
    devices: Vec<(DeviceIdentity, CountedDevice<DirectInputOutputBlockDevice>)>,
    geometry: FixedGeometry,
    run: NewPoolFileCreationRun,
    every_window_matches_device: bool,
}

/// 量两块盘、建池、暖机、新池新建文件，打几何、段序列、设备调用数、每次发布的写与窗口、事务那几行（六档共用的那一段）。
/// `barrier_fault` 在新池新建文件之前布下（`skip-new-pool-file-creation-barrier` 那一档）；路上失败照失败账打完退出。
fn start_through_the_new_pool_file_creation(
    arguments: &OnDeviceRunArguments,
    mode: OnDeviceRunMode,
    policy: PageCachePolicy,
    barrier_fault: BarrierFaultBeforeTheNewPoolFileCreation,
) -> OnDeviceRun {
    let device_paths = arguments.device_paths.clone();
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
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        FaultInjectingBlockDevice::new(identity, open(path, policy), plan.clone()),
                        stream.clone(),
                    ),
                )
            })
            .collect();
    let device_bytes = devices[0].1.size_in_bytes();
    emitter.emit(&format!(
        "name=geometry mode={} physical_block_size={physical_block_size} minimum_io={minimum_input_output_bytes} spacing={} write_cache={} device_bytes={device_bytes}",
        arguments.mode_argument,
        parameters.geometry.fixed_structure_slot_spacing,
        write_cache.replace(' ', "_")
    ));
    let counters_before: Vec<_> = device_paths
        .iter()
        .map(|path| block_layer_counters(path))
        .collect();
    let mut counts_after_acquisition: Option<Vec<DeviceCallCounts>> = None;
    let mut counts_before_new_pool_file_creation: Option<Vec<DeviceCallCounts>> = None;
    // 暖机 warm_up() 已调完、publish_first_file() 还没调的那一处：与 new_pool_file_creation_publish.rs 的
    // warm_up_operation_count（该文件第 189、201 行）量的是同一个边界，不是从流尾往回数几步
    // （里程碑「覆盖写、释放、回退与复用」收尾批「55 号装置步数」）。
    let mut operation_count_before_new_pool_file_creation: Option<usize> = None;
    // 分段挂钟在这里起表：往下每一段末尾取一次，行先攒着、跑完一起打——
    // 打印会阻塞（虚机的串口），把它摆在两次取表之间就等于把打印积压算进那一段
    //（`test-discipline.md`「分段计时在被测进程里计，不在转发输出的循环里计」）。
    let mut clock = SegmentClock::start();
    let mut segment_timing_lines: Vec<String> = Vec::new();
    let path_run =
        run_new_pool_file_creation(&parameters, &mut devices, &stream, |point, devices| {
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
                ScenarioPoint::BeforeNewPoolFileCreation => {
                    counts_before_new_pool_file_creation = Some(counts);
                    operation_count_before_new_pool_file_creation = Some(stream.operation_count());
                    segment_timing_lines.push(clock.mark("warm_up"));
                    if let Some(barrier_fault) = barrier_fault {
                        plan.arm(barrier_fault(devices[0].0));
                    }
                }
            }
        });
    let run = match path_run {
        Ok(run) => run,
        Err(failure) => {
            let lines = describe_new_pool_file_creation_path_failure(
                &failure,
                counts_after_acquisition.as_deref(),
                counts_before_new_pool_file_creation.as_deref(),
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
    segment_timing_lines.push(clock.mark("new_pool_file_creation"));
    let counters_after: Vec<_> = device_paths
        .iter()
        .map(|path| block_layer_counters(path))
        .collect();
    let counts_after_new_pool_file_creation: Vec<DeviceCallCounts> = devices
        .iter()
        .map(|(identity, _)| DeviceCallCounts::of(plan.counts_of_device(*identity)))
        .collect();
    let counts_after_acquisition =
        counts_after_acquisition.expect("run_new_pool_file_creation 取号之后叫过一次");
    let counts_before_new_pool_file_creation = counts_before_new_pool_file_creation
        .expect("run_new_pool_file_creation 新池新建文件之前叫过一次");

    let operations = stream.operations();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let warm_up_end = operation_count_before_new_pool_file_creation.expect(
        "run_new_pool_file_creation 在新池新建文件之前叫过一次（ScenarioPoint::BeforeNewPoolFileCreation）：\
         这里就是暖机 warm_up() 已调完、publish_first_file() 还没调的那一处，\
         与 new_pool_file_creation_publish.rs 的 warm_up_operation_count 量的是同一个边界",
    );
    let paths = new_pool_file_creation_paths(&operations, run.mkfs_operation_count, warm_up_end);
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
        "new_pool_file_creation",
        run.output.root.checkpoint_txg.0,
        &run.output.writes,
    ));
    let warm_up_publishes: Vec<&WritesByStructureKind> = run.warm_up.writes.iter().collect();
    let (warm_up_line, warm_up_matches) = publish_writes_against_device(
        "warm_up",
        &warm_up_publishes,
        pool_writes_between(
            &counts_after_acquisition,
            &counts_before_new_pool_file_creation,
        ),
    );
    emitter.emit(&warm_up_line);
    let (new_pool_file_creation_line, new_pool_file_creation_matches) =
        publish_writes_against_device(
            "new_pool_file_creation",
            &[&run.output.writes],
            pool_writes_between(
                &counts_before_new_pool_file_creation,
                &counts_after_new_pool_file_creation,
            ),
        );
    emitter.emit(&new_pool_file_creation_line);
    let every_window_matches_device = warm_up_matches && new_pool_file_creation_matches;
    emitter.emit(&format!(
        "name=transaction key_order_mismatches={} root_txg={} back_chain={}",
        run.output.key_order_mismatches,
        run.output.root.checkpoint_txg.0,
        run.output.record.back_chain
    ));
    OnDeviceRun {
        mode,
        mode_argument: arguments.mode_argument.clone(),
        device_paths,
        policy,
        emitter,
        clock,
        segment_timing_lines,
        parameters,
        stream,
        plan,
        devices,
        geometry,
        run,
        every_window_matches_device,
    }
}

/// 各档最后同一个收尾：丢掉写的那一套句柄、按路径冷重开、恢复读回文件（`raise-rollback-floor` 那一档另打一行所选根从盘上读回的 F），
/// 打分段挂钟、分段登记与包含自检，末行条数；读回的不是这一档最后发布的那一版、哪一段窗口对不上、分段不对就退出码 1。
fn finish_with_the_cold_reopen(
    run: OnDeviceRun,
    line_read_back_after_the_cold_reopen: LineReadBackAfterTheColdReopen,
) {
    // 不再用的几样也绑着（`_` 起头的名字），活到收尾结束，与都在 `main` 里时一样的次序丢
    let OnDeviceRun {
        mode,
        mode_argument,
        device_paths,
        policy,
        mut emitter,
        mut clock,
        mut segment_timing_lines,
        parameters: _parameters,
        stream: _stream,
        plan: _plan,
        devices,
        geometry: _geometry,
        run: _new_pool_file_creation_run,
        every_window_matches_device,
    } = run;
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
    // 抬 F 模式多打一行：所选根从盘上读回的 F（Z10：`name=raise_rollback_floor` 那一行的 `requested_floor=`
    // 是内存里那一版的 F，冷重开择根并不看它，55 号要独立读回盘上的 F 才判得出「F 抬到 3」）。别的模式一行不多（入口交 `None`）。
    let rollback_floor_read_back_line =
        line_read_back_after_the_cold_reopen.map(|read_back| read_back(&reopened));
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
        clock.registration_check(&mode_argument, registered_segments_of(mode));
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

/// 发布 B（`file-overwrite` 与之后两档）：同一个进程里再覆盖写一次（`publish_the_second_version_and_describe`），打它的行、并进窗口账、记一段分段时间。
fn publish_the_second_version_step(run: &mut OnDeviceRun) {
    let second = publish_the_second_version_and_describe(
        &run.parameters,
        &mut run.devices,
        &run.plan,
        &run.stream,
        &run.geometry,
        &run.run,
    )
    .unwrap_or_else(|failed| exit_after_a_failed_run(&mut run.emitter, &failed));
    for line in &second.lines {
        run.emitter.emit(line);
    }
    run.every_window_matches_device &= second.window_matches_device;
    run.segment_timing_lines
        .push(run.clock.mark("file_overwrite"));
}

/// 冷重开时按路径重新打开两块盘（`second-instance` 与 `raise-rollback-floor` 两档换实例那一步交给它）。
fn reopen_by_path(
    device_paths: [String; 2],
    policy: PageCachePolicy,
) -> impl FnOnce(
    Vec<(DeviceIdentity, DirectInputOutputBlockDevice)>,
) -> Vec<(DeviceIdentity, DirectInputOutputBlockDevice)> {
    move |closed_devices: Vec<(DeviceIdentity, DirectInputOutputBlockDevice)>| {
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
    }
}

/// `second-instance`：发布 B 之后同一对盘冷重开、可写挂载，再发布 C；挂载与发布 C 的行并进来。
fn switch_instance_and_publish_the_third_version_step(run: &mut OnDeviceRun) {
    let devices = std::mem::take(&mut run.devices);
    let switched = switch_instance_and_publish_third_version(
        &run.parameters,
        devices,
        &run.stream,
        &run.geometry,
        &mut run.clock,
        reopen_by_path(run.device_paths.clone(), run.policy),
    )
    .unwrap_or_else(|failed| exit_after_a_failed_run(&mut run.emitter, &failed));
    emit_the_lines_after_the_second_version(
        &mut run.emitter,
        &mut run.segment_timing_lines,
        &switched.lines,
    );
    run.every_window_matches_device &= switched.every_window_matches_device;
    run.devices = switched.devices;
}

/// `raise-rollback-floor`：`second-instance` 那条路走完，同一次挂载里再发布 D、抬 F；这几段的行并进来。
fn switch_instance_publish_the_fourth_version_and_raise_the_rollback_floor_step(
    run: &mut OnDeviceRun,
) {
    let devices = std::mem::take(&mut run.devices);
    let raised = switch_instance_and_publish_third_version(
        &run.parameters,
        devices,
        &run.stream,
        &run.geometry,
        &mut run.clock,
        reopen_by_path(run.device_paths.clone(), run.policy),
    )
    .and_then(|switched| {
        publish_the_fourth_version_and_raise_the_rollback_floor(
            &run.parameters,
            switched,
            &run.stream,
            &run.geometry,
            &mut run.clock,
        )
    })
    .unwrap_or_else(|failed| exit_after_a_failed_run(&mut run.emitter, &failed));
    emit_the_lines_after_the_second_version(
        &mut run.emitter,
        &mut run.segment_timing_lines,
        &raised.lines,
    );
    run.every_window_matches_device &= raised.every_window_matches_device;
    run.devices = raised.devices;
}

// ============================== 每档一个入口（QEMU 门禁按档从各自的入口算来宾程序的闭包） ==============================

/// `direct`：绕开页缓存，新池新建文件之后冷重开读回第一版。
fn run_the_direct_mode(arguments: &OnDeviceRunArguments) {
    let run = start_through_the_new_pool_file_creation(
        arguments,
        OnDeviceRunMode::Direct,
        PageCachePolicy::BypassWithDirectInputOutput,
        None,
    );
    finish_with_the_cold_reopen(run, None);
}

/// `page-cache`：走页缓存（阳性对照），其余同 `direct`。
fn run_the_page_cache_mode(arguments: &OnDeviceRunArguments) {
    let run = start_through_the_new_pool_file_creation(
        arguments,
        OnDeviceRunMode::PageCache,
        PageCachePolicy::GoThroughPageCache,
        None,
    );
    finish_with_the_cold_reopen(run, None);
}

/// `skip-new-pool-file-creation-barrier`：新池新建文件之前在盘 0 上吞掉下一道屏障，其余同 `direct`。
fn run_the_skip_new_pool_file_creation_barrier_mode(arguments: &OnDeviceRunArguments) {
    let run = start_through_the_new_pool_file_creation(
        arguments,
        OnDeviceRunMode::SkipNewPoolFileCreationBarrier,
        PageCachePolicy::BypassWithDirectInputOutput,
        Some(swallow_the_next_barrier_on),
    );
    finish_with_the_cold_reopen(run, None);
}

/// `file-overwrite`：新池新建文件之后再发布 B，冷重开读回第二版。
fn run_the_file_overwrite_mode(arguments: &OnDeviceRunArguments) {
    let mut run = start_through_the_new_pool_file_creation(
        arguments,
        OnDeviceRunMode::FileOverwrite,
        PageCachePolicy::BypassWithDirectInputOutput,
        None,
    );
    publish_the_second_version_step(&mut run);
    finish_with_the_cold_reopen(run, None);
}

/// `second-instance`：发布 B 之后换实例、发布 C，冷重开读回第三版。
fn run_the_second_instance_mode(arguments: &OnDeviceRunArguments) {
    let mut run = start_through_the_new_pool_file_creation(
        arguments,
        OnDeviceRunMode::SecondInstance,
        PageCachePolicy::BypassWithDirectInputOutput,
        None,
    );
    publish_the_second_version_step(&mut run);
    switch_instance_and_publish_the_third_version_step(&mut run);
    finish_with_the_cold_reopen(run, None);
}

/// `raise-rollback-floor`：`second-instance` 那条路走完，再发布 D、抬 F，冷重开读回第四版、另打一行所选根从盘上读回的 F。
fn run_the_raise_rollback_floor_mode(arguments: &OnDeviceRunArguments) {
    let mut run = start_through_the_new_pool_file_creation(
        arguments,
        OnDeviceRunMode::RaiseRollbackFloor,
        PageCachePolicy::BypassWithDirectInputOutput,
        None,
    );
    publish_the_second_version_step(&mut run);
    switch_instance_publish_the_fourth_version_and_raise_the_rollback_floor_step(&mut run);
    finish_with_the_cold_reopen(
        run,
        Some(rollback_floor_read_back_from_the_chosen_root_line),
    );
}

/// 一档的入口 fn 与它的名字（名字是写在这里的那个标识符本身，`stringify!`，不另抄一份）。
macro_rules! entry_with_its_name {
    ($entry:ident) => {
        ($entry as fn(&OnDeviceRunArguments), stringify!($entry))
    };
}

/// 模式表 → 那一档的入口（名字照「run_the_<模式参数，- 换成 _>_mode」）：`main` 解析完参数只调它交回的那一个。
fn entry_of_the_mode(mode: OnDeviceRunMode) -> (fn(&OnDeviceRunArguments), &'static str) {
    match mode {
        OnDeviceRunMode::Direct => entry_with_its_name!(run_the_direct_mode),
        OnDeviceRunMode::PageCache => entry_with_its_name!(run_the_page_cache_mode),
        OnDeviceRunMode::SkipNewPoolFileCreationBarrier => {
            entry_with_its_name!(run_the_skip_new_pool_file_creation_barrier_mode)
        }
        OnDeviceRunMode::FileOverwrite => entry_with_its_name!(run_the_file_overwrite_mode),
        OnDeviceRunMode::SecondInstance => entry_with_its_name!(run_the_second_instance_mode),
        OnDeviceRunMode::RaiseRollbackFloor => {
            entry_with_its_name!(run_the_raise_rollback_floor_mode)
        }
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() != 4 {
        eprintln!(
            "用法：new_pool_file_creation_on_device <盘 0> <盘 1> <{}>",
            OnDeviceRunMode::every_argument_for_usage()
        );
        std::process::exit(2);
    }
    let Some(mode) = OnDeviceRunMode::from_argument(&arguments[3]) else {
        eprintln!("不认识的模式 {}", arguments[3]);
        std::process::exit(2)
    };
    let (entry, _entry_name) = entry_of_the_mode(mode);
    entry(&OnDeviceRunArguments {
        device_paths: [arguments[1].clone(), arguments[2].clone()],
        mode_argument: arguments[3].clone(),
    });
}

#[cfg(test)]
mod tests;
