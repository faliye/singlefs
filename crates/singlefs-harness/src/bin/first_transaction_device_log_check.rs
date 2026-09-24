//! 宿主一侧：拿 QEMU blklogwrites 记下的两份设备侧日志，与宿主上重跑同一条写路（同参数同字节）得到的录制流逐盘逐项比。
//!
//!   first_transaction_device_log_check <模式> <log0.img> <log1.img> <设备字节数> <physical_block_size> <minimum_io> [<disk0.img> <disk1.img>]
//!
//! 模式就是虚机里那一次 `first_transaction_on_device` 的模式，宿主照它重跑同样几步（`singlefs_harness::on_device_modes`，两边跑同一份）：
//! 前三个模式停在第一个事务；`second-transaction` 接着发布 B；`second-instance` 再接着冷重开、可写挂载与发布 C
//! （里程碑「第二个事务」增补 2 收口表第 30 行：在这之前宿主只重跑第一个事务，后面三段的写逐项比不到）。
//! 三个数取虚机里那次跑的 `name=geometry` 行。判据：程序的事件是设备侧日志的逐项前缀，前缀之后至多一个 FLUSH
//! （虚机关机时补的那一个；2026-09-14 第一次跑 direct 时每块盘各多出这一个，见 records 十一·六）。
//! 每块盘另报程序每一段在这块盘上投出几件事（`expected_events_by_window`），与第一处不一致落在哪一段（`divergence_window`；
//! 程序的事件全对上、设备侧还多出东西时是 `after_the_program`）。
//! 给了两块数据盘就再在宿主上从盘上冷恢复一次，读回的要是这个模式最后发布的那一版。退出码：0 = 两块盘都对得上且（给了盘时）读回文件；
//! 1 = 有一块盘对不上（打印第一处）或读不回；2 = 用法、日志读不了、或宿主重跑那条写路失败。结果行同样以 `E7RESULT` 打头。

use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::{
    DirectInputOutputBlockDevice, PageCachePolicy, PhysicalBlockSizeInBytes,
    PhysicalBlockSizeSource,
};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::device_log::{
    compare_allowing_trailing_flushes, declared_zero_fills, expected_device_events,
    fold_declared_zero_fills, parse_device_log, DeviceEvent, DeviceLog,
};
use singlefs_harness::on_device_modes::{
    allocator_rebuilt_from_the_records_of, publish_the_second_version, publish_the_third_version,
    OnDeviceRunMode, PublishesAfterTheFirstTransaction,
};
use singlefs_harness::scenario::{e142_parameters, run_first_transaction, ScenarioPoint};
use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};

/// 前缀之后可以接受的 FLUSH 个数：虚机关机时每块盘至多补一个。
const ACCEPTED_TRAILING_FLUSHES: usize = 1;

/// 第一处不一致落在程序最后一件事之后（程序的事件全对上了，设备侧还多出东西）时 `divergence_window` 的写法。
const AFTER_THE_PROGRAM: &str = "after_the_program";

/// 程序那条写路按段切，段名与虚机档分段时间的段名相同。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProgramWindow {
    MakeFilesystem,
    InstanceAcquisition,
    WarmUp,
    FirstTransaction,
    SecondTransaction,
    ReopenAndWritableMount,
    ThirdTransaction,
}

impl ProgramWindow {
    const fn name(self) -> &'static str {
        match self {
            ProgramWindow::MakeFilesystem => "mkfs",
            ProgramWindow::InstanceAcquisition => "instance_acquisition",
            ProgramWindow::WarmUp => "warm_up",
            ProgramWindow::FirstTransaction => "first_transaction",
            ProgramWindow::SecondTransaction => "second_transaction",
            ProgramWindow::ReopenAndWritableMount => "reopen_and_writable_mount",
            ProgramWindow::ThirdTransaction => "third_transaction",
        }
    }
}

/// 宿主重跑的结果：整条录制流，与每一段在流里的终点（到这一段末尾为止流里有几步），按次序。
struct HostRerun {
    operations: Vec<RetainedOperation>,
    window_ends: Vec<(ProgramWindow, usize)>,
}

impl HostRerun {
    /// 每一段在流里占的那一截，按次序。
    fn windows(&self) -> Vec<(ProgramWindow, &[RetainedOperation])> {
        let mut start = 0usize;
        self.window_ends
            .iter()
            .map(|(window, end)| {
                let slice = &self.operations[start..*end];
                start = *end;
                (*window, slice)
            })
            .collect()
    }

    /// `name=host_rerun` 那一行：模式、跑了哪几段、每段几步。
    fn describe(&self, mode: OnDeviceRunMode) -> String {
        let windows = self.windows();
        let names: Vec<&str> = windows.iter().map(|(window, _)| window.name()).collect();
        let sizes: Vec<String> = windows
            .iter()
            .map(|(_, slice)| slice.len().to_string())
            .collect();
        format!(
            "name=host_rerun mode={} windows={} operations_by_window={}",
            mode.argument(),
            names.join(","),
            sizes.join("+")
        )
    }
}

/// 宿主上照这个模式把程序那条写路重跑一遍：稀疏内存盘代替 virtio 盘；`second-instance` 的「冷重开」把同一份镜像交给新的录制器，
/// 与虚机里按路径重新打开同一对盘同一个意思（读不进录制流，重开本身不发写）。
///
/// # Errors
/// 哪一步在宿主上没跑通，交回一句原因。
fn rerun_the_program_on_the_host(
    mode: OnDeviceRunMode,
    parameters: &MakeFilesystemParameters,
    device_bytes: u64,
    stream: &SharedStream,
) -> Result<HostRerun, String> {
    let physical_block_size = PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size);
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..2u32)
        .map(|index| {
            let identity = DeviceIdentity(index);
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    SparseBlockDevice::new(device_bytes, physical_block_size),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let mut window_ends: Vec<(ProgramWindow, usize)> = Vec::new();
    let run = run_first_transaction(parameters, &mut devices, stream, |point, _devices| {
        let finished_window = match point {
            ScenarioPoint::AfterMakeFilesystem => ProgramWindow::MakeFilesystem,
            ScenarioPoint::AfterInstanceAcquisition => ProgramWindow::InstanceAcquisition,
            ScenarioPoint::BeforeFirstTransaction => ProgramWindow::WarmUp,
        };
        window_ends.push((finished_window, stream.operation_count()));
    })
    .map_err(|failure| format!("第一个事务那条路：{}", failure.render()))?;
    window_ends.push((ProgramWindow::FirstTransaction, stream.operation_count()));

    let publishes = mode.publishes_after_the_first_transaction();
    match publishes {
        PublishesAfterTheFirstTransaction::Nothing => {}
        PublishesAfterTheFirstTransaction::SecondVersion
        | PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance => {
            let mut allocator = allocator_rebuilt_from_the_records_of(&devices, &run.output);
            publish_the_second_version(parameters, &mut devices, &mut allocator, &run.output)
                .map_err(|failed| format!("发布 B：{:?}", failed.cause))?;
            window_ends.push((ProgramWindow::SecondTransaction, stream.operation_count()));
        }
    }
    match publishes {
        PublishesAfterTheFirstTransaction::Nothing
        | PublishesAfterTheFirstTransaction::SecondVersion => {}
        PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance => {
            let mut reopened: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
                devices
                    .into_iter()
                    .map(|(identity, closed)| {
                        (
                            identity,
                            RecordingBlockDevice::with_shared_stream(
                                identity,
                                closed.into_inner_and_operations().0,
                                stream.clone(),
                            ),
                        )
                    })
                    .collect();
            let mut mounted = mount_writable(parameters, &mut reopened)
                .map_err(|failure| format!("可写挂载：{failure:?}"))?;
            window_ends.push((
                ProgramWindow::ReopenAndWritableMount,
                stream.operation_count(),
            ));
            let mounted_version = mounted.current.file_version().cloned().ok_or_else(|| {
                "可写挂载之后现行那一版没有文件：发布 B 之后重开，上一版带文件".to_string()
            })?;
            publish_the_third_version(
                parameters,
                &mut reopened,
                &mut mounted.allocator,
                &mounted_version,
                mounted.output.instance,
            )
            .map_err(|failed| format!("发布 C：{:?}", failed.cause))?;
            window_ends.push((ProgramWindow::ThirdTransaction, stream.operation_count()));
        }
    }
    Ok(HostRerun {
        operations: stream.retained_operations(),
        window_ends,
    })
}

fn parse_number(text: &str, what: &str) -> u64 {
    text.parse().unwrap_or_else(|_error| {
        eprintln!("{what} 不是数：{text}");
        std::process::exit(2)
    })
}

fn count_kinds(events: &[DeviceEvent]) -> (usize, usize) {
    let writes = events
        .iter()
        .filter(|event| matches!(event, DeviceEvent::Write { .. }))
        .count();
    let flushes = events
        .iter()
        .filter(|event| matches!(event, DeviceEvent::Flush))
        .count();
    (writes, flushes)
}

/// 一块盘上的比对：结果行，与这块盘对不对得上。
struct DeviceComparison {
    line: String,
    matches: bool,
}

/// 第一处不一致的下标落在程序的哪一段：`window_event_counts` 是每段在这块盘上投出的事件数，按次序；
/// 下标越过程序的全部事件就是 [`AFTER_THE_PROGRAM`]。
fn window_of_event(
    position: usize,
    window_event_counts: &[(ProgramWindow, usize)],
) -> &'static str {
    let mut end = 0usize;
    for (window, count) in window_event_counts {
        end += count;
        if position < end {
            return window.name();
        }
    }
    AFTER_THE_PROGRAM
}

/// 一块盘：程序的信念（宿主重跑的录制流投到这块盘上）与设备侧日志逐项比。
fn compare_one_device(
    rerun: &HostRerun,
    identity: DeviceIdentity,
    log: &DeviceLog,
) -> DeviceComparison {
    let window_event_counts: Vec<(ProgramWindow, usize)> = rerun
        .windows()
        .into_iter()
        .map(|(window, slice)| (window, expected_device_events(slice, identity).len()))
        .collect();
    let expected = expected_device_events(&rerun.operations, identity);
    assert_eq!(
        window_event_counts
            .iter()
            .map(|(_, count)| count)
            .sum::<usize>(),
        expected.len(),
        "录制流投到一块盘上是逐步各投各的：按段投再拼起来与整条投一次件数相同"
    );
    // 程序声明清零的那几段（mkfs 清 journal 环），盘上收到的若干条写先折回一件事再比：
    // 拆成几条由块层定（后端拆一次、来宾内核按 max_sectors_kb 再拆一次），程序管不着。
    // 折不起来（少一块、多一块、中间夹一条不是全 0 的写）就原样留着，照样逐项判红。
    let zero_fills = declared_zero_fills(&rerun.operations, identity);
    let observed = fold_declared_zero_fills(&log.events, &zero_fills);
    let (expected_writes, expected_flushes) = count_kinds(&expected);
    let (observed_writes, observed_flushes) = count_kinds(&observed);
    let comparison = compare_allowing_trailing_flushes(&expected, &observed);
    let divergence = if comparison.divergence.is_none()
        && comparison.trailing_flushes > ACCEPTED_TRAILING_FLUSHES
    {
        Some((
            expected.len() + ACCEPTED_TRAILING_FLUSHES,
            None,
            Some(DeviceEvent::Flush),
        ))
    } else {
        comparison.divergence.clone()
    };
    let (divergence_window, divergence_text) = match &divergence {
        None => ("none", "none".to_string()),
        Some((position, program, device)) => (
            window_of_event(*position, &window_event_counts),
            format!("at={position} program={program:?} device={device:?}").replace(' ', ""),
        ),
    };
    let events_by_window: Vec<String> = window_event_counts
        .iter()
        .map(|(window, count)| format!("{}:{count}", window.name()))
        .collect();
    // `zero_fill_*`：程序声明清了几段、共多少字节（`observed_writes` 里这几段各算一条，折过了）。
    // 盘上真收到这么多字节由折的条件保证：折不满就折不起来、比对判红。
    let zero_fill_bytes: u64 = zero_fills.iter().map(|(_, length)| *length).sum();
    DeviceComparison {
        line: format!(
            "name=device_log device={} declared_entries={} expected_writes={expected_writes} expected_flushes={expected_flushes} observed_writes={observed_writes} observed_flushes={observed_flushes} zero_fills={} zero_fill_bytes={zero_fill_bytes} trailing_flushes={} expected_events_by_window={} divergence_window={divergence_window} divergence={divergence_text}",
            identity.0,
            log.declared_entries
                .map_or_else(|| "NA".to_string(), |declared| declared.to_string()),
            zero_fills.len(),
            comparison.trailing_flushes,
            events_by_window.join(","),
        ),
        matches: divergence.is_none(),
    }
}

fn root_text(
    root: (
        singlefs_core::address::InstanceGeneration,
        singlefs_core::address::CheckpointTxg,
    ),
) -> String {
    format!("{}:{}", root.0 .0, root.1 .0)
}

/// 宿主从两块数据盘上冷恢复一次：读回文件、且内容是这个模式最后发布的那一版，才算读回。
fn recover_from_disk_images(
    paths: [&str; 2],
    physical_block_size: u32,
    expected_content: &[u8],
) -> (String, bool) {
    let disks: Vec<(DeviceIdentity, DirectInputOutputBlockDevice)> = paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let device = DirectInputOutputBlockDevice::open_existing(
                std::path::Path::new(path),
                PageCachePolicy::GoThroughPageCache,
                PhysicalBlockSizeSource::Declared(PhysicalBlockSizeInBytes(physical_block_size)),
            )
            .unwrap_or_else(|error| {
                eprintln!("打不开 {path}：{error}");
                std::process::exit(2)
            });
            (
                DeviceIdentity(u32::try_from(index).expect("设备号")),
                device,
            )
        })
        .collect();
    let report = recover(&disks, JournalPolicy::Consult);
    let (outcome, root, recovered) = match &report.outcome {
        RecoveryOutcome::FileRead { root, content } => (
            "file_read",
            root_text(*root),
            content.as_slice() == expected_content,
        ),
        RecoveryOutcome::NoFile { root } => ("no_file", root_text(*root), false),
        RecoveryOutcome::Failed { root, .. } => (
            "failed",
            root.map_or_else(|| "none".to_string(), root_text),
            false,
        ),
    };
    (
        format!("name=host_recover outcome={outcome} root={root} content_matches={recovered} valid_records={}", report.journal.valid_records),
        recovered,
    )
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() != 7 && arguments.len() != 9 {
        eprintln!(
            "用法：first_transaction_device_log_check <{}> <log0.img> <log1.img> <设备字节数> <physical_block_size> <minimum_io> [<disk0.img> <disk1.img>]",
            OnDeviceRunMode::every_argument_for_usage()
        );
        std::process::exit(2);
    }
    let Some(mode) = OnDeviceRunMode::from_argument(&arguments[1]) else {
        eprintln!("不认识的模式 {}", arguments[1]);
        std::process::exit(2)
    };
    let log_paths = [&arguments[2], &arguments[3]];
    let device_bytes = parse_number(&arguments[4], "设备字节数");
    let physical_block_size = u32::try_from(parse_number(&arguments[5], "physical_block_size"))
        .expect("物理块大小装得进 u32");
    let minimum_input_output_bytes =
        u32::try_from(parse_number(&arguments[6], "minimum_io")).expect("io_min 装得进 u32");
    let parameters = e142_parameters(physical_block_size, minimum_input_output_bytes);
    let rerun =
        rerun_the_program_on_the_host(mode, &parameters, device_bytes, &SharedStream::new())
            .unwrap_or_else(|reason| {
                eprintln!("宿主重跑写路失败：{reason}");
                std::process::exit(2)
            });
    println!("E7RESULT {}", rerun.describe(mode));
    let mut emitted = 1u64;
    let mut every_device_matches = true;
    for (index, log_path) in log_paths.iter().enumerate() {
        let log_bytes = std::fs::read(log_path).unwrap_or_else(|error| {
            eprintln!("读不了 {log_path}：{error}");
            std::process::exit(2)
        });
        let log = parse_device_log(&log_bytes).unwrap_or_else(|error| {
            eprintln!("{log_path} 解析不了：{error:?}");
            std::process::exit(2)
        });
        let identity = DeviceIdentity(u32::try_from(index).expect("设备号"));
        let comparison = compare_one_device(&rerun, identity, &log);
        every_device_matches &= comparison.matches;
        println!("E7RESULT {}", comparison.line);
        emitted += 1;
    }
    println!("E7RESULT name=device_log_verdict matches={every_device_matches}");
    emitted += 1;
    let mut recovered = true;
    if arguments.len() == 9 {
        let (line, read_back) = recover_from_disk_images(
            [&arguments[7], &arguments[8]],
            physical_block_size,
            &mode.last_published_content(),
        );
        println!("E7RESULT {line}");
        emitted += 1;
        recovered = read_back;
    }
    println!("E7RESULT name=done emitted={}", emitted + 1);
    if !(every_device_matches && recovered) {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compare_one_device, rerun_the_program_on_the_host, window_of_event, HostRerun,
        ProgramWindow, AFTER_THE_PROGRAM,
    };
    use singlefs_core::address::DeviceIdentity;
    use singlefs_harness::device_log::{
        declared_zero_fills, expected_device_events, DeviceEvent, DeviceLog,
    };
    use singlefs_harness::on_device_modes::OnDeviceRunMode;
    use singlefs_harness::scenario::e142_parameters;
    use singlefs_harness::{fnv1a_64_of_zeros, SharedStream};

    /// 门禁 55 号给虚机的两块 virtio 盘各 4096 MiB（`VM_DISK_MB=4096`）。
    const VIRTIO_DEVICE_BYTES: u64 = 4096 * 1024 * 1024;
    /// 设备侧日志里一段清零被块层拆成的块长（后端按 4 MiB 拆）。
    const ZERO_FILL_PIECE_BYTES: u64 = 4 * 1024 * 1024;

    /// 结果行里 `key=value` 那一段的值；没有这个字段是 `None`。
    fn field<'line>(line: &'line str, key: &str) -> Option<&'line str> {
        line.split(' ')
            .find_map(|part| part.strip_prefix(key)?.strip_prefix('='))
    }

    fn rerun_of(mode: OnDeviceRunMode) -> HostRerun {
        rerun_the_program_on_the_host(
            mode,
            &e142_parameters(512, 512),
            VIRTIO_DEVICE_BYTES,
            &SharedStream::new(),
        )
        .expect("宿主重跑")
    }

    /// 程序每一段投到这块盘上的事件，按段分开（改坏某一段里的一步用）。
    fn events_by_window(
        rerun: &HostRerun,
        identity: DeviceIdentity,
    ) -> Vec<(ProgramWindow, Vec<DeviceEvent>)> {
        rerun
            .windows()
            .into_iter()
            .map(|(window, slice)| (window, expected_device_events(slice, identity)))
            .collect()
    }

    /// 盘上收到的就是这些事件：程序声明清零的那几段按块层的拆法拆成几条全 0 的写，末尾关机补一个 FLUSH。
    fn device_log_that_received(
        rerun: &HostRerun,
        identity: DeviceIdentity,
        received: Vec<DeviceEvent>,
    ) -> DeviceLog {
        let zero_fills = declared_zero_fills(&rerun.operations, identity);
        let mut events = Vec::new();
        for event in received {
            let declared_zero_fill = match &event {
                DeviceEvent::Write {
                    offset,
                    length,
                    content_hash,
                } => zero_fills
                    .iter()
                    .find(|declared| {
                        **declared == (*offset, *length)
                            && *content_hash == fnv1a_64_of_zeros(*length)
                    })
                    .copied(),
                DeviceEvent::Flush | DeviceEvent::Discard { .. } | DeviceEvent::Mark => None,
            };
            let Some((offset, length)) = declared_zero_fill else {
                events.push(event);
                continue;
            };
            let mut cursor = 0u64;
            while cursor < length {
                let piece = ZERO_FILL_PIECE_BYTES.min(length - cursor);
                events.push(DeviceEvent::Write {
                    offset: singlefs_core::address::DeviceOffsetInBytes(offset.0 + cursor),
                    length: piece,
                    content_hash: fnv1a_64_of_zeros(piece),
                });
                cursor += piece;
            }
        }
        events.push(DeviceEvent::Flush);
        DeviceLog {
            sector_bytes: 512,
            declared_entries: None,
            events,
        }
    }

    fn flattened(windows: Vec<(ProgramWindow, Vec<DeviceEvent>)>) -> Vec<DeviceEvent> {
        windows.into_iter().flat_map(|(_, events)| events).collect()
    }

    /// 每个模式：宿主照模式重跑的段与虚机里那一次一样多；盘上收到的恰是程序发的（清零段被拆碎、末尾多一个关机 FLUSH）⇒ 两块盘都对得上。
    /// `second-transaction` / `second-instance` 在发布 B、可写挂载、发布 C 那几段上每块盘都投出了事件——那几段真在比对里。
    #[test]
    fn every_mode_reruns_its_own_windows_and_matches_a_log_that_received_exactly_the_program_events(
    ) {
        for mode in OnDeviceRunMode::ALL {
            let rerun = rerun_of(mode);
            let expected_windows = match mode {
                OnDeviceRunMode::Direct
                | OnDeviceRunMode::PageCache
                | OnDeviceRunMode::SkipFirstTransactionBarrier => {
                    "mkfs,instance_acquisition,warm_up,first_transaction"
                }
                OnDeviceRunMode::SecondTransaction => {
                    "mkfs,instance_acquisition,warm_up,first_transaction,second_transaction"
                }
                OnDeviceRunMode::SecondInstance => {
                    "mkfs,instance_acquisition,warm_up,first_transaction,second_transaction,reopen_and_writable_mount,third_transaction"
                }
            };
            let described = rerun.describe(mode);
            assert_eq!(field(&described, "mode"), Some(mode.argument()));
            assert_eq!(
                field(&described, "windows"),
                Some(expected_windows),
                "{described}"
            );
            for identity in [DeviceIdentity(0), DeviceIdentity(1)] {
                let windows = events_by_window(&rerun, identity);
                for (window, events) in &windows {
                    assert!(
                        !events.is_empty(),
                        "{mode:?} 盘 {} 的 {} 一件事都没投",
                        identity.0,
                        window.name()
                    );
                }
                let log = device_log_that_received(&rerun, identity, flattened(windows));
                let comparison = compare_one_device(&rerun, identity, &log);
                assert!(comparison.matches, "{mode:?}：{}", comparison.line);
                assert_eq!(field(&comparison.line, "trailing_flushes"), Some("1"));
                assert_eq!(field(&comparison.line, "divergence_window"), Some("none"));
                assert_eq!(field(&comparison.line, "divergence"), Some("none"));
            }
        }
    }

    /// 改掉一步就红，而且红在那一段：`second-instance` 的发布 B、可写挂载、发布 C 三段里各改一步
    /// （少一个写、少一个 FLUSH、一个写的内容变了、头两个写换了次序），两块盘都判不对，`divergence_window` 指着被改的那一段。
    #[test]
    fn one_step_changed_in_the_second_version_the_writable_mount_or_the_third_version_is_red_in_that_window(
    ) {
        let rerun = rerun_of(OnDeviceRunMode::SecondInstance);
        let changed_windows = [
            ProgramWindow::SecondTransaction,
            ProgramWindow::ReopenAndWritableMount,
            ProgramWindow::ThirdTransaction,
        ];
        type StepChange = fn(&mut Vec<DeviceEvent>) -> bool;
        let changes: [(&str, StepChange); 4] = [
            ("少一个写", |events| {
                match events
                    .iter()
                    .position(|event| matches!(event, DeviceEvent::Write { .. }))
                {
                    Some(position) => {
                        events.remove(position);
                        true
                    }
                    None => false,
                }
            }),
            ("少一个 FLUSH", |events| {
                match events
                    .iter()
                    .position(|event| matches!(event, DeviceEvent::Flush))
                {
                    Some(position) => {
                        events.remove(position);
                        true
                    }
                    None => false,
                }
            }),
            ("最后一个写的内容变了", |events| {
                match events
                    .iter_mut()
                    .rev()
                    .find(|event| matches!(event, DeviceEvent::Write { .. }))
                {
                    Some(DeviceEvent::Write { content_hash, .. }) => {
                        *content_hash ^= 1;
                        true
                    }
                    Some(DeviceEvent::Flush | DeviceEvent::Discard { .. } | DeviceEvent::Mark)
                    | None => false,
                }
            }),
            ("头两个写换了次序", |events| {
                let writes: Vec<usize> = events
                    .iter()
                    .enumerate()
                    .filter(|(_, event)| matches!(event, DeviceEvent::Write { .. }))
                    .map(|(position, _)| position)
                    .take(2)
                    .collect();
                if writes.len() < 2 || events[writes[0]] == events[writes[1]] {
                    return false;
                }
                events.swap(writes[0], writes[1]);
                true
            }),
        ];
        for identity in [DeviceIdentity(0), DeviceIdentity(1)] {
            for changed_window in changed_windows {
                for (change_name, change) in &changes {
                    let mut windows = events_by_window(&rerun, identity);
                    let (_, events) = windows
                        .iter_mut()
                        .find(|(window, _)| *window == changed_window)
                        .expect("second-instance 跑了这一段");
                    assert!(
                        change(events),
                        "盘 {} 的 {} 里找不到可改的那一步（{change_name}）",
                        identity.0,
                        changed_window.name()
                    );
                    let log = device_log_that_received(&rerun, identity, flattened(windows));
                    let comparison = compare_one_device(&rerun, identity, &log);
                    assert!(
                        !comparison.matches,
                        "盘 {} 的 {} {change_name}：必须判红\n{}",
                        identity.0,
                        changed_window.name(),
                        comparison.line
                    );
                    assert_eq!(
                        field(&comparison.line, "divergence_window"),
                        Some(changed_window.name()),
                        "{change_name}：{}",
                        comparison.line
                    );
                }
            }
        }
    }

    /// 模式送错也判红：`second-instance` 的盘拿 `second-transaction` 去比，程序的事件整段是日志的前缀、红在程序之后
    /// （今天门禁 55 号对后两档判的正是这个形态）；反过来 `second-transaction` 的盘拿 `second-instance` 去比，红在可写挂载那一段开头。
    /// 程序之后多出一个写、或关机之后多出第二个 FLUSH，都红在程序之后。
    #[test]
    fn device_log_from_another_mode_or_with_anything_after_the_program_is_red_after_the_program() {
        let second_transaction = rerun_of(OnDeviceRunMode::SecondTransaction);
        let second_instance = rerun_of(OnDeviceRunMode::SecondInstance);
        for identity in [DeviceIdentity(0), DeviceIdentity(1)] {
            let longer_log = device_log_that_received(
                &second_instance,
                identity,
                flattened(events_by_window(&second_instance, identity)),
            );
            let prefix_only = compare_one_device(&second_transaction, identity, &longer_log);
            assert!(!prefix_only.matches);
            assert_eq!(
                field(&prefix_only.line, "divergence_window"),
                Some(AFTER_THE_PROGRAM)
            );
            assert!(
                prefix_only.line.contains("program=Nonedevice=Some(Write"),
                "{}",
                prefix_only.line
            );

            let shorter_log = device_log_that_received(
                &second_transaction,
                identity,
                flattened(events_by_window(&second_transaction, identity)),
            );
            let missing_mount = compare_one_device(&second_instance, identity, &shorter_log);
            assert!(!missing_mount.matches);
            assert_eq!(
                field(&missing_mount.line, "divergence_window"),
                Some(ProgramWindow::ReopenAndWritableMount.name()),
                "{}",
                missing_mount.line
            );

            let mut received = flattened(events_by_window(&second_instance, identity));
            received.push(DeviceEvent::Write {
                offset: singlefs_core::address::DeviceOffsetInBytes(0),
                length: 4096,
                content_hash: 7,
            });
            let extra_write = compare_one_device(
                &second_instance,
                identity,
                &device_log_that_received(&second_instance, identity, received),
            );
            assert!(!extra_write.matches);
            assert_eq!(
                field(&extra_write.line, "divergence_window"),
                Some(AFTER_THE_PROGRAM)
            );

            let mut two_shutdown_flushes = longer_log.clone();
            two_shutdown_flushes.events.push(DeviceEvent::Flush);
            let extra_flush = compare_one_device(&second_instance, identity, &two_shutdown_flushes);
            assert!(!extra_flush.matches);
            assert_eq!(
                field(&extra_flush.line, "divergence_window"),
                Some(AFTER_THE_PROGRAM)
            );
        }
    }

    #[test]
    fn every_position_is_attributed_to_the_window_whose_events_contain_it() {
        let counts = [
            (ProgramWindow::MakeFilesystem, 3),
            (ProgramWindow::InstanceAcquisition, 0),
            (ProgramWindow::WarmUp, 2),
        ];
        assert_eq!(window_of_event(0, &counts), "mkfs");
        assert_eq!(window_of_event(2, &counts), "mkfs");
        assert_eq!(window_of_event(3, &counts), "warm_up", "空的一段不认领下标");
        assert_eq!(window_of_event(4, &counts), "warm_up");
        assert_eq!(window_of_event(5, &counts), AFTER_THE_PROGRAM);
    }
}
