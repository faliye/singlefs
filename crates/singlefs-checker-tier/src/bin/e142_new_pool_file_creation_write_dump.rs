//! E142（新池新建文件的干跑） 第 15 次跑步③：只读导出，比对侧。
//!
//! 在两块内存盘上跑与 `scripts/experiments/e142_new_pool_file_creation_dry_run.rs`
//! 同一套参数的 mkfs → 取号 → 暖机 → 新池新建文件，把「新池新建文件窗口」（`ScenarioPoint::BeforeNewPoolFileCreation`
//! 那一刻录制流的长度起，到发布真正结束为止）里**每一次写请求**按 (设备, 字节偏移, 字节长度, sha256, 整段十六进制)
//! 原样打出来。**不带任何位置寻址的布局知识**：不按名字归类、不假设哪个偏移属于哪棵树，
//! 区域清单就是写本身（跑前登记 `research/prompts/e142-r15-prereg.md` 第一节「装置写在哪」第 2 条）。
//!
//! 只驱动、只观测：不改 `singlefs-core`、`singlefs-checker`、`singlefs-format`。
//! 决定几何与负载的参数写成本地常量，值抄自模型的同名常量（同一份跑前登记同一条），
//! 每个常量另加一条断言与 `crate::scenario` 的公开常量、`singlefs_format::TEST_IMAGE_DEFAULT_BYTES` 回比。
//!
//!   e142_new_pool_file_creation_write_dump
//!   e142_new_pool_file_creation_write_dump --device-bytes <每盘字节数>
//!
//! 不带参数时两块盘各 `TEST_IMAGE_DEFAULT_BYTES`（4 GiB），输出与这个 bin 只收零个参数时逐字节相同；`--device-bytes` 换每盘字节数
//! （E142 第 22 次第二段的 P8G：两块 8 GiB 盘，`research/prompts/e142-r22-prereg.md` 读法 R67），别的参数照旧，
//! `make_filesystem` 收不收由它自己判（环不超过盘的四分之一）。`name=impl_config` 的 `image_bytes=` 打的是这一次的每盘字节数。
//! 不读环境、不碰真设备、不写任何文件。退出码：0 = 正常；1 = 命令行参数不是上面两种之一，或每盘字节数解得开而
//! 两块盘内存里的单元区空闲图合起来超过定死的上界或算不出（例：`--device-bytes 18446744073709551615`，建盘之前按算术判），
//! 或 `make_filesystem` 不收（例：`--device-bytes 0`），三种都在 stderr 打原因（最后一种打 mkfs 交回的错）；
//! 101 = mkfs 收了、后面某一步（取号、暖机、新池新建文件）在内存盘上没走通，panic 带那一步与原因。

use singlefs_core::address::DeviceIdentity;
use singlefs_core::allocator::{unit_area_slots_of_device_starting_at, UnitAreaStart};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_format::{NEW_POOL_FILE_CREATION_TXG, TEST_IMAGE_DEFAULT_BYTES};
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::scenario::{
    e142_parameters, run_new_pool_file_creation, NewPoolFileCreationPathFailure,
    NewPoolFileCreationPathStep, ScenarioPoint, E142_FILESYSTEM_IDENTIFIER, FIRST_FILE_BYTES,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_harness::sha256::sha256_hexadecimal;
use singlefs_harness::{
    RecordedOperationKind, RecordingBlockDevice, RetainedOperation, SharedStream,
};

/// E142 那套几何：两块同构盘，`physical_block_size` = io_min = 512。
/// 值抄自跑前登记第一节「装置写在哪」第 2 条 ①：模型
/// `scripts/experiments/e142_new_pool_file_creation_dry_run.rs` 的 `const DEVICE_BYTES`、
/// `const PHYSICAL_BLOCK_BYTES`（两者在这个装置里都是 512 那一档的推论，逐项断言见 `main` 开头）。
const DEVICE_COUNT: u32 = 2;
const PHYSICAL_BLOCK_SIZE_IN_BYTES: u32 = 512;
const MINIMUM_INPUT_OUTPUT_BYTES: u32 = 512;
/// 值抄自跑前登记同一条：模型的 `FIXED_FSID` / `FIXED_WRITE_TIME_SECONDS` / `FIRST_INODE_NUMBER`，
/// 文件 3000 字节、内容 `index % 251`、改动计数 = 3（改动计数不是这个导出要驱动的量，新池新建文件只跑一次）。
const EXPECTED_FILE_BYTES: usize = 3000;

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

/// 换每盘字节数的那个选项。
const DEVICE_BYTES_OPTION: &str = "--device-bytes";

/// 命令行给得不对时的退出码（文件头「退出码」那一句）：参数写法不对，或每盘字节数 mkfs 不收。
const EXIT_CODE_OF_A_REFUSED_COMMAND_LINE: i32 = 1;

/// 整条路没走完时这个 bin 怎么收场。几何与负载的参数都是本地常量，命令行只换每盘字节数，所以 mkfs 不收就是给的每盘字节数不对，
/// 照命令行给错了收场；mkfs 收了之后哪一步没走通，是实现在内存盘上的毛病，照旧 panic。
#[derive(Debug, PartialEq, Eq)]
enum EndingOfAnUnfinishedPath {
    /// 退 [`EXIT_CODE_OF_A_REFUSED_COMMAND_LINE`]，打 mkfs 交回的错。
    DeviceBytesRefusedByMakeFilesystem { reason: String },
    /// panic（退 101），带停在哪一步与那一步的账。
    StepAfterMakeFilesystemFailed { reason: String },
}

fn ending_of_an_unfinished_path(
    failure: &NewPoolFileCreationPathFailure,
) -> EndingOfAnUnfinishedPath {
    match failure.failed_step {
        NewPoolFileCreationPathStep::MakeFilesystem => {
            EndingOfAnUnfinishedPath::DeviceBytesRefusedByMakeFilesystem {
                reason: failure.render(),
            }
        }
        NewPoolFileCreationPathStep::InstanceAcquisition
        | NewPoolFileCreationPathStep::WarmUp
        | NewPoolFileCreationPathStep::NewPoolFileCreation => {
            EndingOfAnUnfinishedPath::StepAfterMakeFilesystemFailed {
                reason: format!("{failure:?}"),
            }
        }
    }
}

/// 两块同构内存盘，各包一层录制器、共用 `stream`。
fn two_recording_memory_devices(
    device_bytes: u64,
    stream: &SharedStream,
) -> Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> {
    (0..DEVICE_COUNT)
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    SparseBlockDevice::new(
                        device_bytes,
                        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_SIZE_IN_BYTES),
                    ),
                    stream.clone(),
                ),
            )
        })
        .collect()
}

/// 命令行给的每盘字节数：零个参数是 `TEST_IMAGE_DEFAULT_BYTES`，`--device-bytes <十进制字节数>` 是给的那个数；
/// 别的写法（多给、少给、数解不开）交回给人看的一句。盘够不够大不在这里判，交给 `make_filesystem`。
fn device_bytes_from_the_command_line(arguments: &[String]) -> Result<u64, String> {
    match arguments {
        [] => Ok(TEST_IMAGE_DEFAULT_BYTES),
        [option, value] if option == DEVICE_BYTES_OPTION => value.parse::<u64>().map_err(|error| {
            format!("{DEVICE_BYTES_OPTION} 后面要一个十进制字节数，收到 {value:?}：{error}")
        }),
        other => Err(format!(
            "只收零个参数或 {DEVICE_BYTES_OPTION} <每盘字节数>，收到 {other:?}"
        )),
    }
}

/// 一块盘的单元区每槽在内存空闲图（`singlefs_core::allocator::DeviceFreeMap::with_unit_area_start`）里至多占几个字节：
/// 每槽三个布尔数组（已分配、隔离、扣到 F 生效）3 字节，每 64 槽一段的四个 `u64` 计数数组合起来每槽半个字节，向上取整。
const FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST: u64 = 4;

/// 两块盘的单元区空闲图合起来至多这么多字节，超过的每盘字节数建盘之前就拒（1 GiB：每盘单元区至多 2²⁷ 槽、单元区 2 TiB，
/// 跑前登记过的 4 GiB 与 P8G 的 8 GiB 两档都远在里面；这个数是推的，没量过哪一档开始分配不出）。按算术判、不试分配：
/// 判据不随这台机器的内存与 overcommit 设定变（第 23 批 X 那一版试分配要不到才拒，主 agent 定改成按算术判）。
const FREE_MAP_BYTES_OF_THE_TWO_DEVICES_AT_MOST: u64 = 1 << 30;

/// 两块盘的单元区空闲图大小在 [`FREE_MAP_BYTES_OF_THE_TWO_DEVICES_AT_MOST`] 之内：mkfs 收下之后建分配器（`allocator_after_make_filesystem`）照每盘单元区槽数
/// 分配每槽一格的空闲图，那一步分配不出时进程当场 abort（退 134：`--device-bytes 18446744073709551615` 建盘就是这样，代码轮判决
/// `research/prompts/m3-batch20-code-r1-main-verification.md` 第二节 K1 那一格）。这里在建盘之前按 [`FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST`] 算：
/// 每盘单元区槽数 × 每槽字节数 × 盘数，用 checked 运算，乘法溢出或超过上界交回给人看的一句。
/// 单元区起点不在聚簇段边界上、盘末尾在单元区起点之前的不在这里判，mkfs 自己拒（`UnitAreaStartOffTheClusterSegmentBoundaryUnsupported`、
/// `UnitAreaBeyondDevice`），照旧打 mkfs 交回的错。
fn free_maps_of_the_two_devices_stay_within_the_bound(
    device_bytes: u64,
    journal_ring_bytes: u64,
) -> Result<(), String> {
    let Ok(unit_area_start) = UnitAreaStart::following_the_journal_ring(journal_ring_bytes) else {
        return Ok(());
    };
    let Ok(unit_area_slots) = unit_area_slots_of_device_starting_at(device_bytes, unit_area_start)
    else {
        return Ok(());
    };
    let refusal = |why: &str| {
        format!(
            "每盘 {device_bytes} 字节：单元区 {unit_area_slots} 槽，内存里的空闲图每盘至多要 {FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST} × {unit_area_slots} 字节，{DEVICE_COUNT} 块盘合起来{why}"
        )
    };
    let free_map_bytes_of_the_two_devices = unit_area_slots
        .checked_mul(FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST)
        .and_then(|bytes_per_device| bytes_per_device.checked_mul(u64::from(DEVICE_COUNT)))
        .ok_or_else(|| refusal("算不出（乘法溢出 u64）"))?;
    if free_map_bytes_of_the_two_devices > FREE_MAP_BYTES_OF_THE_TWO_DEVICES_AT_MOST {
        return Err(refusal(&format!(
            " {free_map_bytes_of_the_two_devices} 字节，超过上界 {FREE_MAP_BYTES_OF_THE_TWO_DEVICES_AT_MOST} 字节"
        )));
    }
    Ok(())
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let device_bytes = match device_bytes_from_the_command_line(&arguments) {
        Ok(device_bytes) => device_bytes,
        Err(refusal) => {
            eprintln!("e142_new_pool_file_creation_write_dump {refusal}");
            std::process::exit(EXIT_CODE_OF_A_REFUSED_COMMAND_LINE);
        }
    };

    // 绝对值断言（门禁 80 号）：4 GiB = 4 × 1024 × 1024 × 1024，独立算出的字面量，不从代码读回来。
    // 两边在编译期就定了，clippy 建议挪进 const 块（否则判「assertions_on_constants」）。
    const {
        assert!(
            TEST_IMAGE_DEFAULT_BYTES == 4_294_967_296,
            "跑前登记第一节写死两块 4 GiB 盘；singlefs_format::TEST_IMAGE_DEFAULT_BYTES 变了这一格要重登记"
        );
        assert!(
            FIRST_FILE_BYTES == 3000,
            "第一个文件字节数与跑前登记第一节抄的模型同名常量回比"
        );
        assert!(
            EXPECTED_FILE_BYTES == 3000,
            "本地常量与上面这条断言钉的是同一个数"
        );
    };

    let parameters = e142_parameters(PHYSICAL_BLOCK_SIZE_IN_BYTES, MINIMUM_INPUT_OUTPUT_BYTES);
    if let Err(refusal) = free_maps_of_the_two_devices_stay_within_the_bound(
        device_bytes,
        parameters.geometry.journal_ring_bytes,
    ) {
        eprintln!("e142_new_pool_file_creation_write_dump {refusal}");
        std::process::exit(EXIT_CODE_OF_A_REFUSED_COMMAND_LINE);
    }
    let stream = SharedStream::retaining_contents();
    let mut devices = two_recording_memory_devices(device_bytes, &stream);

    // 窗口的起点：`ScenarioPoint::BeforeNewPoolFileCreation` 那一次回调时录制流的长度
    // （与 `new_pool_file_creation_region_bytes.rs` 的 `main` 里 `ScenarioPoint::BeforeNewPoolFileCreation` 那一臂同一个取法，跑前登记第一节「装置写在哪」第 2 条末句）。
    let mut steps_before_new_pool_file_creation: Option<usize> = None;
    let path =
        run_new_pool_file_creation(
            &parameters,
            &mut devices,
            &stream,
            |point, _devices| match point {
                ScenarioPoint::AfterMakeFilesystem | ScenarioPoint::AfterInstanceAcquisition => {}
                ScenarioPoint::BeforeNewPoolFileCreation => {
                    steps_before_new_pool_file_creation = Some(stream.operation_count());
                }
            },
        );
    let run = match path {
        Ok(run) => run,
        Err(failure) => match ending_of_an_unfinished_path(&failure) {
            EndingOfAnUnfinishedPath::DeviceBytesRefusedByMakeFilesystem { reason } => {
                eprintln!(
                    "e142_new_pool_file_creation_write_dump 每盘 {device_bytes} 字节 mkfs 不收：{reason}"
                );
                std::process::exit(EXIT_CODE_OF_A_REFUSED_COMMAND_LINE);
            }
            EndingOfAnUnfinishedPath::StepAfterMakeFilesystemFailed { reason } => panic!(
                "整条路（mkfs → 取号 → 暖机 → 新池新建文件）mkfs 之后在内存盘上没走通：{reason}"
            ),
        },
    };
    let steps_before_new_pool_file_creation = steps_before_new_pool_file_creation
        .expect("run_new_pool_file_creation 一定走过 BeforeNewPoolFileCreation");

    let all_operations = stream.retained_operations();
    let before_window = &all_operations[..steps_before_new_pool_file_creation];
    let window = &all_operations[steps_before_new_pool_file_creation..];

    let mut emitter = Emitter { emitted: 0 };
    emitter.emit(&format!(
        "name=impl_config devices={DEVICE_COUNT} image_bytes={device_bytes} physical_block_size={PHYSICAL_BLOCK_SIZE_IN_BYTES} minimum_input_output_bytes={MINIMUM_INPUT_OUTPUT_BYTES} region_device_0={} region_device_1={} region_device_2={} slots_per_region={} journal_ring_bytes={} file_bytes={FIRST_FILE_BYTES} write_time_seconds={FIXED_WRITE_TIME_SECONDS} filesystem_identifier={} checkpoint_txg={NEW_POOL_FILE_CREATION_TXG} mkfs_operations={}",
        parameters.region_devices[0].0,
        parameters.region_devices[1].0,
        parameters.region_devices[2].0,
        parameters.geometry.root_ring_slots_per_region.count(),
        parameters.geometry.journal_ring_bytes,
        hexadecimal_text(&E142_FILESYSTEM_IDENTIFIER),
        run.mkfs_operation_count,
    ));

    // 窗口之前最后一次暖机的 5 次写的 (设备, 偏移, 长度)，给第五节 P5 用（跑前登记 P5 的字面是
    // 「最后一次暖机的 5 次写」——只数写，不数屏障；原样按发出次序，最早的在前）。
    let before_window_writes: Vec<&RetainedOperation> = before_window
        .iter()
        .filter(|retained| retained.operation.kind != RecordedOperationKind::Barrier)
        .collect();
    let last_five: Vec<String> = before_window_writes
        .iter()
        .rev()
        .take(5)
        .map(|retained| {
            format!(
                "device{}@{}+{}",
                retained.operation.device.0, retained.operation.offset.0, retained.operation.length
            )
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    emitter.emit(&format!(
        "name=before_window_summary operations={} writes={} last_five={}",
        before_window.len(),
        before_window_writes.len(),
        if last_five.is_empty() {
            "none".to_string()
        } else {
            last_five.join(",")
        }
    ));

    let mut write_count = 0u64;
    let mut barrier_count = 0u64;
    for (step_index, retained) in window.iter().enumerate() {
        let operation = &retained.operation;
        if operation.kind == RecordedOperationKind::Barrier {
            barrier_count += 1;
            emitter.emit(&format!(
                "name=window_barrier step={step_index} device={}",
                operation.device.0
            ));
            continue;
        }
        write_count += 1;
        // 屏障没有字节内容（不算一次「写」，R2）；新池新建文件窗口按跑前登记的几何不会出现大到扛不住的写清零，铺 0 直接摆出来。
        let bytes = retained
            .written_bytes_rebuilding_zero_fills()
            .expect("write / write_fua / write_zeroes 都能重建出完整字节");
        emitter.emit(&format!(
            "name=device_region_bytes step={step_index} device={} offset={} length={} kind={} sha256={} hexadecimal={}",
            operation.device.0,
            operation.offset.0,
            operation.length,
            operation.kind.name(),
            sha256_hexadecimal(&bytes),
            hexadecimal_text(&bytes),
        ));
    }
    emitter.emit(&format!(
        "name=device_region_bytes_summary window_operations={} writes={write_count} barriers={barrier_count}",
        window.len()
    ));
    emitter.finish();
}

#[cfg(test)]
mod tests {
    use super::{
        device_bytes_from_the_command_line, ending_of_an_unfinished_path,
        free_maps_of_the_two_devices_stay_within_the_bound, two_recording_memory_devices,
        EndingOfAnUnfinishedPath, DEVICE_COUNT, FREE_MAP_BYTES_OF_THE_TWO_DEVICES_AT_MOST,
        FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST, MINIMUM_INPUT_OUTPUT_BYTES,
        PHYSICAL_BLOCK_SIZE_IN_BYTES,
    };
    use singlefs_core::allocator::{unit_area_slots_of_device_starting_at, UnitAreaStart};
    use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES};
    use singlefs_harness::scenario::{
        e142_parameters, run_new_pool_file_creation, NewPoolFileCreationPathFailure,
        NewPoolFileCreationPathStep,
    };
    use singlefs_harness::SharedStream;

    fn arguments(words: &[&str]) -> Vec<String> {
        words.iter().map(|word| (*word).to_string()).collect()
    }

    #[test]
    fn no_argument_keeps_the_four_gibibyte_devices_of_the_first_registration() {
        assert_eq!(
            device_bytes_from_the_command_line(&arguments(&[])),
            Ok(4_294_967_296),
            "不带参数是跑前登记第一节那两块 4 GiB 盘"
        );
    }

    #[test]
    fn the_device_bytes_option_takes_the_eight_gibibyte_devices_of_the_second_geometry() {
        assert_eq!(
            device_bytes_from_the_command_line(&arguments(&["--device-bytes", "8589934592"])),
            Ok(8_589_934_592),
            "P8G 每盘 8 589 934 592 字节（第 22 次登记读法 R67）"
        );
    }

    #[test]
    fn an_unparsable_value_an_unknown_option_and_a_missing_value_are_each_refused() {
        for refused in [
            arguments(&["--device-bytes", "8GiB"]),
            arguments(&["--image-bytes", "8589934592"]),
            arguments(&["--device-bytes"]),
            arguments(&["--device-bytes", "8589934592", "extra"]),
        ] {
            assert!(
                device_bytes_from_the_command_line(&refused).is_err(),
                "{refused:?} 要拒"
            );
        }
    }

    /// `--device-bytes 0` 解得开（第二种写法），mkfs 不收：照命令行给错了收场、带 mkfs 的原因，不 panic（代码轮判决 N9）。
    #[test]
    fn zero_device_bytes_parse_but_make_filesystem_refuses_them_and_the_bin_ends_as_a_refused_command_line(
    ) {
        let device_bytes =
            device_bytes_from_the_command_line(&["--device-bytes".to_string(), "0".to_string()])
                .expect("0 是一个十进制字节数，解得开");
        let stream = SharedStream::retaining_contents();
        let mut devices = two_recording_memory_devices(device_bytes, &stream);
        let failure = match run_new_pool_file_creation(
            &e142_parameters(PHYSICAL_BLOCK_SIZE_IN_BYTES, MINIMUM_INPUT_OUTPUT_BYTES),
            &mut devices,
            &stream,
            |_point, _devices| {},
        ) {
            Ok(_) => panic!("每盘 0 字节 mkfs 不该收"),
            Err(failure) => failure,
        };
        match ending_of_an_unfinished_path(&failure) {
            EndingOfAnUnfinishedPath::DeviceBytesRefusedByMakeFilesystem { reason } => assert!(
                reason.contains("JournalRingTooLargeForDevice"),
                "原因要带 mkfs 交回的错：{reason}"
            ),
            EndingOfAnUnfinishedPath::StepAfterMakeFilesystemFailed { reason } => {
                panic!("mkfs 拒的值要照命令行给错了收场，不 panic：{reason}")
            }
        }
        assert_eq!(stream.operation_count(), 0, "mkfs 拒在任何写之前");
    }

    /// `--device-bytes 18446744073709551615` 解得开（第二种写法），两块盘的单元区空闲图按算术超过上界：建盘之前照命令行给错了收场、打原因，
    /// 不走到建分配器那一步 abort（退 134，代码轮判决 `research/prompts/m3-batch20-code-r1-main-verification.md` 第二节 K1 那一格）。
    /// 跑前登记的 4 GiB 与 P8G 的 8 GiB 两档照旧放过；每盘 0 字节不在这里拒，交给 mkfs 判（上一条）。
    #[test]
    fn the_largest_device_bytes_parse_but_their_free_maps_exceed_the_bound_and_the_bin_ends_as_a_refused_command_line(
    ) {
        let device_bytes = device_bytes_from_the_command_line(&arguments(&[
            "--device-bytes",
            "18446744073709551615",
        ]))
        .expect("2⁶⁴ − 1 是一个十进制字节数，解得开");
        match free_maps_of_the_two_devices_stay_within_the_bound(
            device_bytes,
            JOURNAL_RING_DEFAULT_BYTES,
        ) {
            Ok(()) => panic!("每盘 2⁶⁴ − 1 字节的单元区空闲图不该在上界之内"),
            Err(refusal) => assert!(
                refusal.contains("超过上界") && refusal.contains("每盘 18446744073709551615 字节"),
                "原因点名每盘字节数与超过上界：{refusal}"
            ),
        }
        for device_bytes_that_fit in [4_294_967_296, 8_589_934_592, 0] {
            assert_eq!(
                free_maps_of_the_two_devices_stay_within_the_bound(
                    device_bytes_that_fit,
                    JOURNAL_RING_DEFAULT_BYTES
                ),
                Ok(()),
                "每盘 {device_bytes_that_fit} 字节不在这里拒"
            );
        }
    }

    /// 上界那一格：两块盘的空闲图合起来恰好等于上界的每盘字节数放过，再多一个槽就拒（判据是算术，不随机器变）。
    #[test]
    fn device_bytes_at_the_free_map_bound_pass_and_one_slot_more_is_refused() {
        let unit_area_start = UnitAreaStart::following_the_journal_ring(JOURNAL_RING_DEFAULT_BYTES)
            .expect("默认环长的单元区起点在聚簇段边界上");
        let slot_bytes = SLOT_BYTES;
        let unit_area_slots_at_the_bound = FREE_MAP_BYTES_OF_THE_TWO_DEVICES_AT_MOST
            / (FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST * u64::from(DEVICE_COUNT));
        let device_bytes_at_the_bound =
            (unit_area_start.slot().0 + unit_area_slots_at_the_bound) * slot_bytes;
        assert_eq!(
            unit_area_slots_of_device_starting_at(device_bytes_at_the_bound, unit_area_start),
            Ok(unit_area_slots_at_the_bound),
            "这么多字节的盘单元区恰好是上界那么多槽"
        );
        assert_eq!(
            free_maps_of_the_two_devices_stay_within_the_bound(
                device_bytes_at_the_bound,
                JOURNAL_RING_DEFAULT_BYTES
            ),
            Ok(()),
            "恰好在上界上放过"
        );
        match free_maps_of_the_two_devices_stay_within_the_bound(
            device_bytes_at_the_bound + slot_bytes,
            JOURNAL_RING_DEFAULT_BYTES,
        ) {
            Ok(()) => panic!("比上界多一个槽要拒"),
            Err(refusal) => assert!(refusal.contains("超过上界"), "原因写超过上界：{refusal}"),
        }
    }

    /// mkfs 收了而后面某一步没走通的，照旧当实现的毛病（panic），不混进命令行给错了那一种。
    #[test]
    fn a_failure_after_make_filesystem_stays_an_implementation_failure() {
        for failed_step in [
            NewPoolFileCreationPathStep::InstanceAcquisition,
            NewPoolFileCreationPathStep::WarmUp,
            NewPoolFileCreationPathStep::NewPoolFileCreation,
        ] {
            let failure = NewPoolFileCreationPathFailure {
                failed_step,
                cause: "内存盘上没走通".to_string(),
                writes_of_failed_publishes: Vec::new(),
                writes_of_persisted_publishes: Vec::new(),
            };
            assert!(
                matches!(
                    ending_of_an_unfinished_path(&failure),
                    EndingOfAnUnfinishedPath::StepAfterMakeFilesystemFailed { .. }
                ),
                "{failed_step:?} 停下是实现的毛病"
            );
        }
    }
}
