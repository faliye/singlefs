//! 虚机档（QEMU/KVM）：在两块真 virtio 盘上跑第一个事务的整条写路，冷重开再恢复读回文件。
//!
//!   first_transaction_on_device /dev/vda /dev/vdb <direct | page-cache | skip-first-transaction-barrier | second-transaction | second-instance>
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
//! 每次发布（两次暖机、第一个事务、`second-transaction` 模式下的发布 B）各打一行 `name=publish_writes`：写入口按结构种类记的写调用数与写字节
//! （里程碑「第二个事务」增补 1 第 1 件）；每段窗口（两次暖机合一段、第一个事务、发布 B）再打一行 `name=publish_writes_against_device`，
//! 按种类的合计与设备一层数的（`FaultInjectingDevice`，两盘相加）逐项比，对不上 `matches=false`、退出码 1。

use std::path::Path;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator};
use singlefs_core::block_device::{
    probe_queue_number_under, probe_queue_text_under, BlockDevice, BlockDeviceError,
    DirectInputOutputBlockDevice, PageCachePolicy, PhysicalBlockSizeInBytes,
    PhysicalBlockSizeSource, WriteDurability, SYSFS_CLASS_BLOCK,
};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolVersion, PoolWriter};
use singlefs_core::write_accounting::{
    WriteCallsAndBytes, WritesByStructureKind, WrittenStructureKind,
};
use singlefs_harness::scenario::{
    e142_parameters, first_file_content, run_first_transaction, ScenarioPoint,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_harness::segments::{
    closed_form_state_count, segment_kinds_text, segment_sizes_text, split_into_segments,
    FixedGeometry,
};
use singlefs_harness::{RecordedOperation, RecordingBlockDevice, SharedStream};
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunMode {
    Direct,
    PageCache,
    SkipFirstTransactionBarrier,
    SecondTransaction,
    SecondInstance,
}

/// 第二版的内容：与第一版不同长度、不同字节，读回时分得开。
fn second_file_content() -> Vec<u8> {
    (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 第三版的内容（`second-instance` 模式的发布 C）：与前两版都不同长度、不同字节。
fn third_file_content() -> Vec<u8> {
    (0..2500usize)
        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
        .collect()
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
    fn of<Inner: BlockDevice>(device: &FaultInjectingDevice<Inner>) -> Self {
        Self {
            write_calls: device.write_calls,
            written_bytes: device.written_bytes,
            force_unit_access_writes: device.force_unit_access_writes,
            barrier_calls: device.barrier_calls,
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

/// 一次发布的 `name=publish_writes` 行：合计取 `total()`（从记过的每一笔加），各种按 `IN_REPORT_ORDER` 逐个列、没写过的列 0。
fn describe_publish_writes(publish: &str, txg: u64, writes: &WritesByStructureKind) -> String {
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
        "name=publish_writes publish={publish} txg={txg} write_calls={} written_bytes={}{per_kind}",
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

/// 包在真设备外面、录制器里面：数程序真正交给设备的调用，按需吞掉一道屏障。
struct FaultInjectingDevice<Inner: BlockDevice> {
    inner: Inner,
    skip_next_barrier: bool,
    skipped_barriers: u64,
    barrier_calls: u64,
    write_calls: u64,
    written_bytes: u64,
    force_unit_access_writes: u64,
    /// 这块盘收到第一道屏障那一刻的计数（屏障本身不算进去）：冷重开之后的可写挂载里第一道屏障是取号那一道，
    /// 挂载那一段窗口从这里算起（`second-instance` 模式；可写挂载在取号与写行之间没有给调用方的口子）。
    counts_when_first_barrier_arrived: Option<DeviceCallCounts>,
}

impl<Inner: BlockDevice> FaultInjectingDevice<Inner> {
    /// 只计数、不吞屏障。
    fn counting(inner: Inner) -> Self {
        Self {
            inner,
            skip_next_barrier: false,
            skipped_barriers: 0,
            barrier_calls: 0,
            write_calls: 0,
            written_bytes: 0,
            force_unit_access_writes: 0,
            counts_when_first_barrier_arrived: None,
        }
    }
}

impl<Inner: BlockDevice> BlockDevice for FaultInjectingDevice<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.write_calls += 1;
        self.written_bytes += u64::try_from(bytes.len()).expect("长度");
        if durability == WriteDurability::ForceUnitAccess {
            self.force_unit_access_writes += 1;
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        if self.counts_when_first_barrier_arrived.is_none() {
            self.counts_when_first_barrier_arrived = Some(DeviceCallCounts::of(self));
        }
        if self.skip_next_barrier {
            self.skip_next_barrier = false;
            self.skipped_barriers += 1;
            return Ok(());
        }
        self.barrier_calls += 1;
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
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

/// 录制器里面那一层计数的盘。
type CountedDevice<Inner> = RecordingBlockDevice<FaultInjectingDevice<Inner>>;

/// 每块盘此刻的计数快照。
fn device_call_counts<Inner: BlockDevice>(
    devices: &[(DeviceIdentity, CountedDevice<Inner>)],
) -> Vec<DeviceCallCounts> {
    devices
        .iter()
        .map(|(_, device)| DeviceCallCounts::of(device.inner()))
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

/// `second-instance` 模式在发布 B 之后写出的东西：重开之后的那一套句柄（冷恢复之前要丢掉）、按次序要打的结果行、
/// 每段窗口按种类的合计与设备一层是否都相等。
struct SecondInstanceRun<Inner: BlockDevice> {
    devices: Vec<(DeviceIdentity, CountedDevice<Inner>)>,
    lines: Vec<String>,
    every_window_matches_device: bool,
}

/// 发布 B 之后：丢掉写的那一套句柄，同一对盘冷重开（`reopen` 拿到关掉的那几块盘、交回重新打开的），走可写挂载（恢复、取号、写行、暖机），
/// 再发布 C。挂载那一段窗口从每块盘收到第一道屏障（取号那一道）算起、到挂载返回为止，按种类的合计是写行与每次暖机之和：取号的超级块槽写不是发布、
/// 不进按种类的账，与第一个事务那条路上暖机窗口从取号之后算起同一个口径。
///
/// # Errors
/// 可写挂载失败、挂载之后现行那一版没有文件、取号那道屏障没数到、发布 C 失败：交回一句原因。
fn switch_instance_and_publish_third_version<Inner, Reopen>(
    parameters: &MakeFilesystemParameters,
    devices_after_second_transaction: Vec<(DeviceIdentity, CountedDevice<Inner>)>,
    stream: &SharedStream,
    geometry: &FixedGeometry,
    reopen: Reopen,
) -> Result<SecondInstanceRun<Inner>, String>
where
    Inner: BlockDevice,
    Reopen: FnOnce(Vec<(DeviceIdentity, Inner)>) -> Vec<(DeviceIdentity, Inner)>,
{
    let closed_devices: Vec<(DeviceIdentity, Inner)> = devices_after_second_transaction
        .into_iter()
        .map(|(identity, device)| (identity, device.into_inner_and_operations().0.inner))
        .collect();
    let mut devices: Vec<(DeviceIdentity, CountedDevice<Inner>)> = reopen(closed_devices)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    FaultInjectingDevice::counting(inner),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let mut lines = Vec::new();

    let operations_before_mount = stream.operations().len();
    let mount_started = Instant::now();
    let mut mounted = mount_writable(parameters, &mut devices)
        .map_err(|failure| format!("可写挂载：{failure:?}"))?;
    let mount_nanoseconds = mount_started.elapsed().as_nanos();
    let counts_after_mount = device_call_counts(&devices);
    let counts_after_acquisition: Vec<DeviceCallCounts> = devices
        .iter()
        .map(|(identity, device)| {
            device
                .inner()
                .counts_when_first_barrier_arrived
                .ok_or_else(|| {
                    format!(
                        "盘 {} 在可写挂载里一道屏障都没收到：取号那道屏障没发",
                        identity.0
                    )
                })
        })
        .collect::<Result<_, _>>()?;
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

    let current = mounted
        .current
        .file_version()
        .ok_or_else(|| "可写挂载之后现行那一版没有文件：发布 B 之后重开，上一版带文件".to_string())?
        .clone();
    let instance = mounted.output.instance;
    let operations_before_third = stream.operations().len();
    let third_started = Instant::now();
    let third = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut pool,
            &mut mounted.allocator,
            &current,
            FirstFile {
                content: &third_file_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
            },
            instance,
        )
    }
    .map_err(|failure| format!("发布 C：{failure:?}"))?;
    let third_nanoseconds = third_started.elapsed().as_nanos();
    let counts_after_third = device_call_counts(&devices);
    let third_operations = stream.operations();
    let third_segments =
        split_into_segments(&third_operations[operations_before_third..], geometry);
    let per_device_third: Vec<String> = counts_after_third
        .iter()
        .enumerate()
        .map(|(index, later)| later.since(counts_after_mount[index]).describe(index))
        .collect();
    lines.push(format!(
        "name=third_transaction root_txg={} transaction={} released={} nanoseconds={third_nanoseconds} operations={} segments={} closed_form={} {}",
        third.root.checkpoint_txg.0,
        third.record.transaction,
        third.released.len(),
        third_operations.len() - operations_before_third,
        segment_sizes_text(&third_segments),
        closed_form_state_count(&third_segments),
        per_device_third.join(" ")
    ));
    lines.push(describe_publish_writes(
        "third_transaction",
        third.root.checkpoint_txg.0,
        &third.writes,
    ));
    let (third_window_line, third_window_matches) = publish_writes_against_device(
        "third_transaction",
        &[&third.writes],
        pool_writes_between(&counts_after_mount, &counts_after_third),
    );
    lines.push(third_window_line);

    Ok(SecondInstanceRun {
        devices,
        lines,
        every_window_matches_device: mount_window_matches && third_window_matches,
    })
}

#[allow(
    clippy::too_many_lines,
    reason = "虚机里只跑这一件事：写、数、重开、读回，结果行按次序打"
)]
fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() != 4 {
        eprintln!("用法：first_transaction_on_device <盘 0> <盘 1> <direct | page-cache | skip-first-transaction-barrier | second-transaction | second-instance>");
        std::process::exit(2);
    }
    let device_paths = [arguments[1].clone(), arguments[2].clone()];
    let mode = match arguments[3].as_str() {
        "direct" => RunMode::Direct,
        "page-cache" => RunMode::PageCache,
        "skip-first-transaction-barrier" => RunMode::SkipFirstTransactionBarrier,
        "second-transaction" => RunMode::SecondTransaction,
        "second-instance" => RunMode::SecondInstance,
        other => {
            eprintln!("不认识的模式 {other}");
            std::process::exit(2)
        }
    };
    let policy = match mode {
        RunMode::Direct
        | RunMode::SkipFirstTransactionBarrier
        | RunMode::SecondTransaction
        | RunMode::SecondInstance => PageCachePolicy::BypassWithDirectInputOutput,
        RunMode::PageCache => PageCachePolicy::GoThroughPageCache,
    };
    let publishes_second_version = match mode {
        RunMode::Direct | RunMode::PageCache | RunMode::SkipFirstTransactionBarrier => false,
        RunMode::SecondTransaction | RunMode::SecondInstance => true,
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
    let mut devices: Vec<(
        DeviceIdentity,
        RecordingBlockDevice<FaultInjectingDevice<DirectInputOutputBlockDevice>>,
    )> = device_paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let identity = DeviceIdentity(u32::try_from(index).expect("设备号"));
            let device = FaultInjectingDevice::counting(open(path, policy));
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
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
    let run = run_first_transaction(&parameters, &mut devices, &stream, |point, devices| {
        let counts: Vec<DeviceCallCounts> = devices
            .iter()
            .map(|(_, device)| DeviceCallCounts::of(device.inner()))
            .collect();
        match point {
            ScenarioPoint::AfterInstanceAcquisition => counts_after_acquisition = Some(counts),
            ScenarioPoint::BeforeFirstTransaction => {
                counts_before_first_transaction = Some(counts);
                if mode == RunMode::SkipFirstTransactionBarrier {
                    devices[0].1.inner_mut().skip_next_barrier = true;
                }
            }
        }
    })
    .unwrap_or_else(|reason| {
        eprintln!("写路径失败：{reason}");
        std::process::exit(5)
    });
    let counters_after: Vec<_> = device_paths
        .iter()
        .map(|path| block_layer_counters(path))
        .collect();
    let counts_after_first_transaction: Vec<DeviceCallCounts> = devices
        .iter()
        .map(|(_, device)| DeviceCallCounts::of(device.inner()))
        .collect();
    let counts_after_acquisition =
        counts_after_acquisition.expect("run_first_transaction 取号之后叫过一次");
    let counts_before_first_transaction =
        counts_before_first_transaction.expect("run_first_transaction 第一个事务之前叫过一次");

    let operations = stream.operations();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
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
    for (index, (identity, device)) in devices.iter().enumerate() {
        let counted = device.inner();
        emitter.emit(&format!(
            "name=device_calls device={} path={} writes={} written_bytes={} force_unit_access_writes={} barriers={} skipped_barriers={} {}",
            identity.0,
            device_paths[index],
            counted.write_calls,
            counted.written_bytes,
            counted.force_unit_access_writes,
            counted.barrier_calls,
            counted.skipped_barriers,
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
        "name=transaction policy_mismatches={} key_order_mismatches={} root_txg={} back_chain={}",
        run.policy_mismatches,
        run.output.key_order_mismatches,
        run.output.root.checkpoint_txg.0,
        run.output.record.back_chain
    ));

    // 第二个事务（发布 B）：同一个进程里再覆盖写一次，分配器从第一个事务的分配记录重建（与可写挂载同一条路）；
    // 两次快照之差就是这一次发布交给设备的调用数，挂钟单独计。
    let expected_content = if publishes_second_version {
        let before: Vec<DeviceCallCounts> = devices
            .iter()
            .map(|(_, device)| DeviceCallCounts::of(device.inner()))
            .collect();
        let operations_before = stream.operations().len();
        let device_maps: Vec<DeviceFreeMap> = devices
            .iter()
            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
            .collect();
        let mut allocator =
            PoolAllocator::rebuild_from_records(device_maps, run.output.allocation_records.clone());
        let started = Instant::now();
        let second = {
            let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
            publish_overwrite(
                &mut pool,
                &mut allocator,
                &run.output,
                FirstFile {
                    content: &second_file_content(),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
                },
                run.output.root.instance,
            )
        }
        .unwrap_or_else(|failure| {
            eprintln!("第二个事务失败：{failure:?}");
            std::process::exit(5)
        });
        let nanoseconds = started.elapsed().as_nanos();
        let after_operations = stream.operations();
        let second_segments =
            split_into_segments(&after_operations[operations_before..], &geometry);
        let after: Vec<DeviceCallCounts> = devices
            .iter()
            .map(|(_, device)| DeviceCallCounts::of(device.inner()))
            .collect();
        let per_device: Vec<String> = after
            .iter()
            .enumerate()
            .map(|(index, later)| later.since(before[index]).describe(index))
            .collect();
        emitter.emit(&format!(
            "name=second_transaction root_txg={} transaction={} released={} nanoseconds={nanoseconds} operations={} segments={} closed_form={} {}",
            second.root.checkpoint_txg.0,
            second.record.transaction,
            second.released.len(),
            after_operations.len() - operations_before,
            segment_sizes_text(&second_segments),
            closed_form_state_count(&second_segments),
            per_device.join(" ")
        ));
        emitter.emit(&describe_publish_writes(
            "second_transaction",
            second.root.checkpoint_txg.0,
            &second.writes,
        ));
        let (second_transaction_line, second_transaction_matches) = publish_writes_against_device(
            "second_transaction",
            &[&second.writes],
            pool_writes_between(&before, &after),
        );
        emitter.emit(&second_transaction_line);
        every_window_matches_device &= second_transaction_matches;
        second_file_content()
    } else {
        first_file_content()
    };

    // `second-instance`：发布 B 之后同一对盘冷重开、可写挂载，再发布 C。
    let (devices, expected_content) = match mode {
        RunMode::SecondInstance => {
            let switched = switch_instance_and_publish_third_version(
                &parameters,
                devices,
                &stream,
                &geometry,
                |closed_devices| {
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
                        .collect()
                },
            )
            .unwrap_or_else(|reason| {
                eprintln!("第二个实例失败：{reason}");
                std::process::exit(5)
            });
            for line in &switched.lines {
                emitter.emit(line);
            }
            every_window_matches_device &= switched.every_window_matches_device;
            (switched.devices, third_file_content())
        }
        RunMode::Direct
        | RunMode::PageCache
        | RunMode::SkipFirstTransactionBarrier
        | RunMode::SecondTransaction => (devices, expected_content),
    };

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
    emitter.emit(&format!(
        "name=recover_cold outcome={outcome} root={root} content_matches={content_matches} valid_records={} above_water={} applied={} verification_passed={} mapping_fallbacks={}",
        report.journal.valid_records, report.journal.above_water, report.journal.prefix_applied, report.journal.verification_passed, report.mapping_fallbacks
    ));
    emitter.finish();
    if !(outcome == "file_read" && content_matches && every_window_matches_device) {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        describe_publish_writes, pool_writes_between, publish_writes_against_device,
        second_file_content, switch_instance_and_publish_third_version, third_file_content,
        CountedDevice, DeviceCallCounts, FaultInjectingDevice,
    };
    use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
    use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator};
    use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
    use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
    use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
    use singlefs_core::write_accounting::{
        WriteCallsAndBytes, WritesByStructureKind, WrittenStructureKind,
    };
    use singlefs_harness::crash::SparseBlockDevice;
    use singlefs_harness::scenario::{
        e142_parameters, run_first_transaction, FIXED_WRITE_TIME_SECONDS,
    };
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
        let mut devices: Vec<(DeviceIdentity, CountedDevice<SparseBlockDevice>)> = (0..2u32)
            .map(|device_number| {
                let identity = DeviceIdentity(device_number);
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        FaultInjectingDevice::counting(SparseBlockDevice::new(
                            SPARSE_DEVICE_BYTES,
                            PhysicalBlockSizeInBytes(512),
                        )),
                        stream.clone(),
                    ),
                )
            })
            .collect();
        let run = run_first_transaction(&parameters, &mut devices, &stream, |_point, _devices| {})
            .expect("第一个事务");
        let mut allocator = PoolAllocator::rebuild_from_records(
            devices
                .iter()
                .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
                .collect(),
            run.output.allocation_records.clone(),
        );
        let second = {
            let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
            publish_overwrite(
                &mut pool,
                &mut allocator,
                &run.output,
                FirstFile {
                    content: &second_file_content(),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
                },
                run.output.root.instance,
            )
            .expect("发布 B")
        };
        assert_eq!(second.root.checkpoint_txg, CheckpointTxg(4));
        let geometry = FixedGeometry {
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        };

        let switched = switch_instance_and_publish_third_version(
            &parameters,
            devices,
            &stream,
            &geometry,
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
            .map(|(identity, device)| (identity, device.into_inner_and_operations().0.inner))
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
             superblock_slot_write_calls=0 superblock_slot_written_bytes=0"
        );
    }

    #[test]
    fn device_window_check_matches_only_when_both_write_calls_and_bytes_are_equal() {
        let mut first = WritesByStructureKind::NOTHING_WRITTEN;
        first.count_write_call(WrittenStructureKind::SuperblockSlot, &[0u8; 4096]);
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

    #[test]
    fn second_file_content_differs_from_the_first_in_length_and_bytes() {
        let second = second_file_content();
        assert_eq!(second.len(), 4100);
        assert_ne!(second, singlefs_harness::scenario::first_file_content());
    }
}
