//! E142（第一个事务的干跑） 第 15 次跑第五段：第八节 G4（几何敏感性，一盘，走整条写路，与模型比）。
//!
//! 跟 `e142_first_transaction_write_dump.rs` 同一套装置逻辑，唯一的差别是几何：只有一块盘、
//! 三个区域全部落在这一块盘上（`region_devices = [0, 0, 0]`，D2（RAID 条带策略） 已定项 6 明知违反、
//! 只作对照——跑前登记 `research/prompts/e142-r15-prereg.md` 第八节 8.2「G4」）。
//!
//! 只驱动、只观测：不改 `singlefs-core`、`singlefs-checker`、`singlefs-format`。
//! 决定几何与负载的参数写成本地常量，值抄自模型 `PoolParameters::control_one_device_no_barriers()`
//! （`research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`）与跑前登记同一条；
//! 每个另加一条断言与 `crate::scenario` 的公开常量、`singlefs_format::TEST_IMAGE_DEFAULT_BYTES` 回比。
//!
//!   e142_first_transaction_write_dump_one_device
//!
//! 不收参数、不读环境、不碰真设备、不写任何文件。退出码：0 = 正常；1 = 命令行给了参数。
//!
//! **已实测：这个装置跑不出产物**——`crates/singlefs-core/src/make_filesystem.rs:194` 有一条硬编码断言
//! 「第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）」，`assert_eq!(devices.len(), 2)`，不是 `MakeFilesystemError`
//! 走不到 `Result`，一盘调用必定 panic（2026-09-25 现跑过：`thread 'main' panicked at
//! crates/singlefs-core/src/make_filesystem.rs:194:5`）。这正是跑前登记第八节 G4 那一格允许的结局
//! 「`crates/` 侧造不出这个几何」——按只驱动、只观测的写范围，这个装置不改 `singlefs-core` 去放开那条断言，
//! 如实记「G4 只在一个几何上量过」，不停机、不作废（模型一侧的数已经由
//! `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs` 里
//! `positive_control_without_barriers_has_4092_violations_out_of_8192`（13 次写、10 个单元）与
//! `allocation_record_tree_shape_matches_the_five_geometry_points` 的 `one_device` 那一档（分配记录树 3 个节点）
//! 钉住，与跑前登记第八节 G4「该看到」一致）。这个文件留着，是为了让这条断言与它的行号可以被重新跑一次核实，
//! 不是留一个能跑通的装置。

use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_format::{FIRST_TRANSACTION_TXG, JOURNAL_RING_DEFAULT_BYTES, TEST_IMAGE_DEFAULT_BYTES};
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::scenario::{
    run_first_transaction, ScenarioPoint, E142_FILESYSTEM_IDENTIFIER, FIRST_FILE_BYTES,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_harness::sha256::sha256_hexadecimal;
use singlefs_harness::{
    RecordedOperationKind, RecordingBlockDevice, RetainedOperation, SharedStream,
};

/// G4 那套几何：一块盘，三个区域全落在盘 0 上；`physical_block_size` = io_min = 512（与主几何同一档）。
/// 值抄自模型 `PoolParameters::control_one_device_no_barriers()`（第 2029–2031 行：
/// `device_count = 1`、`region_devices = [DeviceIdentity(0); 3]`）。
const DEVICE_COUNT: u32 = 1;
const PHYSICAL_BLOCK_SIZE_IN_BYTES: u32 = 512;
const MINIMUM_INPUT_OUTPUT_BYTES: u32 = 512;
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

fn kind_name(kind: RecordedOperationKind) -> &'static str {
    match kind {
        RecordedOperationKind::Write => "write",
        RecordedOperationKind::WriteForceUnitAccess => "write_fua",
        RecordedOperationKind::WriteZeroes => "write_zeroes",
        RecordedOperationKind::Barrier => "barrier",
    }
}

/// 屏障没有字节内容（不算一次「写」）；写清零的内容按长度铺 0——与 `e142_first_transaction_write_dump.rs`
/// 同一个理由（整段全 0 由 offset 与 length 全定，重放的人不用真的把它摆出来）。
fn operation_bytes(retained: &RetainedOperation) -> Option<Vec<u8>> {
    match retained.operation.kind {
        RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
            retained.contents.clone()
        }
        RecordedOperationKind::WriteZeroes => Some(vec![
            0u8;
            usize::try_from(retained.operation.length)
                .expect("窗口里的写清零长度装得进 usize")
        ]),
        RecordedOperationKind::Barrier => None,
    }
}

/// 一盘对照的 mkfs 参数：`region_devices` 三个区域全指盘 0（D2 已定项 6 明知违反、只作对照臂），
/// 其余字段与 `scenario::e142_parameters` 同一份构造（那个函数把 `region_devices` 写死成两盘，
/// 这里不能借它，直接构造同一个结构体）。
fn one_device_parameters(physical_block_size: u32, minimum_input_output_bytes: u32) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: E142_FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(0), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size,
            minimum_input_output_bytes,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(minimum_input_output_bytes),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if !arguments.is_empty() {
        eprintln!("e142_first_transaction_write_dump_one_device 不收参数，收到 {arguments:?}");
        std::process::exit(1);
    }

    const {
        assert!(
            TEST_IMAGE_DEFAULT_BYTES == 4_294_967_296,
            "跑前登记第一节写死 4 GiB 一档；singlefs_format::TEST_IMAGE_DEFAULT_BYTES 变了这一格要重登记"
        );
        assert!(FIRST_FILE_BYTES == 3000, "第一个文件字节数与跑前登记第一节抄的模型同名常量回比");
        assert!(EXPECTED_FILE_BYTES == 3000, "本地常量与上面这条断言钉的是同一个数");
    };

    let parameters = one_device_parameters(PHYSICAL_BLOCK_SIZE_IN_BYTES, MINIMUM_INPUT_OUTPUT_BYTES);
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..DEVICE_COUNT)
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    SparseBlockDevice::new(TEST_IMAGE_DEFAULT_BYTES, PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_SIZE_IN_BYTES)),
                    stream.clone(),
                ),
            )
        })
        .collect();

    let mut steps_before_first_transaction: Option<usize> = None;
    let run = run_first_transaction(&parameters, &mut devices, &stream, |point, _devices| match point {
        ScenarioPoint::AfterMakeFilesystem | ScenarioPoint::AfterInstanceAcquisition => {}
        ScenarioPoint::BeforeFirstTransaction => {
            steps_before_first_transaction = Some(stream.operation_count());
        }
    });
    let run = match run {
        Ok(run) => run,
        Err(error) => {
            let mut emitter = Emitter { emitted: 0 };
            emitter.emit(&format!("name=g4_blocked reason={error:?}"));
            emitter.finish();
            return;
        }
    };
    let steps_before_first_transaction =
        steps_before_first_transaction.expect("run_first_transaction 一定走过 BeforeFirstTransaction");

    let all_operations = stream.retained_operations();
    let before_window = &all_operations[..steps_before_first_transaction];
    let window = &all_operations[steps_before_first_transaction..];

    let mut emitter = Emitter { emitted: 0 };
    emitter.emit(&format!(
        "name=impl_config devices={DEVICE_COUNT} image_bytes={TEST_IMAGE_DEFAULT_BYTES} physical_block_size={PHYSICAL_BLOCK_SIZE_IN_BYTES} minimum_input_output_bytes={MINIMUM_INPUT_OUTPUT_BYTES} region_device_0={} region_device_1={} region_device_2={} slots_per_region={} journal_ring_bytes={} file_bytes={FIRST_FILE_BYTES} write_time_seconds={FIXED_WRITE_TIME_SECONDS} filesystem_identifier={} checkpoint_txg={FIRST_TRANSACTION_TXG} mkfs_operations={}",
        parameters.region_devices[0].0,
        parameters.region_devices[1].0,
        parameters.region_devices[2].0,
        parameters.geometry.root_ring_slots_per_region.count(),
        parameters.geometry.journal_ring_bytes,
        hexadecimal_text(&E142_FILESYSTEM_IDENTIFIER),
        run.mkfs_operation_count,
    ));

    let before_window_writes: Vec<&RetainedOperation> = before_window
        .iter()
        .filter(|retained| retained.operation.kind != RecordedOperationKind::Barrier)
        .collect();
    let last_five: Vec<String> = before_window_writes
        .iter()
        .rev()
        .take(5)
        .map(|retained| format!("device{}@{}+{}", retained.operation.device.0, retained.operation.offset.0, retained.operation.length))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    emitter.emit(&format!(
        "name=before_window_summary operations={} writes={} last_five={}",
        before_window.len(),
        before_window_writes.len(),
        if last_five.is_empty() { "none".to_string() } else { last_five.join(",") }
    ));

    let mut write_count = 0u64;
    let mut barrier_count = 0u64;
    for (step_index, retained) in window.iter().enumerate() {
        let operation = &retained.operation;
        if operation.kind == RecordedOperationKind::Barrier {
            barrier_count += 1;
            emitter.emit(&format!("name=window_barrier step={step_index} device={}", operation.device.0));
            continue;
        }
        write_count += 1;
        let bytes = operation_bytes(retained).expect("write / write_fua / write_zeroes 都能重建出完整字节");
        emitter.emit(&format!(
            "name=device_region_bytes step={step_index} device={} offset={} length={} kind={} sha256={} hexadecimal={}",
            operation.device.0,
            operation.offset.0,
            operation.length,
            kind_name(operation.kind),
            sha256_hexadecimal(&bytes),
            hexadecimal_text(&bytes),
        ));
    }
    emitter.emit(&format!("name=device_region_bytes_summary window_operations={} writes={write_count} barriers={barrier_count}", window.len()));
    emitter.finish();
}

#[cfg(test)]
mod tests {
    use super::{kind_name, one_device_parameters, operation_bytes};
    use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
    use singlefs_harness::{RecordedOperation, RecordedOperationKind, RetainedOperation};

    #[test]
    fn kind_name_covers_every_recorded_operation_kind_without_a_wildcard_arm() {
        assert_eq!(kind_name(RecordedOperationKind::Write), "write");
        assert_eq!(kind_name(RecordedOperationKind::WriteForceUnitAccess), "write_fua");
        assert_eq!(kind_name(RecordedOperationKind::WriteZeroes), "write_zeroes");
        assert_eq!(kind_name(RecordedOperationKind::Barrier), "barrier");
    }

    fn retained(kind: RecordedOperationKind, length: u64, contents: Option<Vec<u8>>) -> RetainedOperation {
        RetainedOperation {
            operation: RecordedOperation { device: DeviceIdentity(0), kind, offset: DeviceOffsetInBytes(0), length, content_hash: 0 },
            contents,
        }
    }

    #[test]
    fn operation_bytes_returns_the_retained_content_for_a_plain_write() {
        let write = retained(RecordedOperationKind::Write, 3, Some(vec![1, 2, 3]));
        assert_eq!(operation_bytes(&write), Some(vec![1, 2, 3]));
    }

    #[test]
    fn operation_bytes_rebuilds_an_all_zero_buffer_for_write_zeroes_even_without_retained_content() {
        let write_zeroes = retained(RecordedOperationKind::WriteZeroes, 4, None);
        assert_eq!(operation_bytes(&write_zeroes), Some(vec![0, 0, 0, 0]));
    }

    #[test]
    fn operation_bytes_is_none_for_a_barrier_which_carries_no_bytes() {
        let barrier = retained(RecordedOperationKind::Barrier, 0, None);
        assert_eq!(operation_bytes(&barrier), None);
    }

    #[test]
    fn one_device_parameters_puts_every_region_on_device_zero() {
        let parameters = one_device_parameters(512, 512);
        assert_eq!(parameters.region_devices, [DeviceIdentity(0), DeviceIdentity(0), DeviceIdentity(0)]);
    }
}
