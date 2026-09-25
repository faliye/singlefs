//! E142（第一个事务的干跑） 第 15 次跑步③：只读导出，比对侧。
//!
//! 在两块内存盘上跑与 `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`
//! 同一套参数的 mkfs → 取号 → 暖机 → 第一个事务，把「第一个事务窗口」（`ScenarioPoint::BeforeFirstTransaction`
//! 那一刻录制流的长度起，到发布真正结束为止）里**每一次写请求**按 (设备, 字节偏移, 字节长度, sha256, 整段十六进制)
//! 原样打出来。**不带任何位置寻址的布局知识**：不按名字归类、不假设哪个偏移属于哪棵树，
//! 区域清单就是写本身（跑前登记 `research/prompts/e142-r15-prereg.md` 第一节「装置写在哪」第 2 条）。
//!
//! 只驱动、只观测：不改 `singlefs-core`、`singlefs-checker`、`singlefs-format`。
//! 决定几何与负载的参数写成本地常量，值抄自模型的同名常量（同一份跑前登记同一条），
//! 每个常量另加一条断言与 `crate::scenario` 的公开常量、`singlefs_format::TEST_IMAGE_DEFAULT_BYTES` 回比。
//!
//!   e142_first_transaction_write_dump
//!
//! 不收参数、不读环境、不碰真设备、不写任何文件。退出码：0 = 正常；1 = 命令行给了参数。

use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_format::{FIRST_TRANSACTION_TXG, TEST_IMAGE_DEFAULT_BYTES};
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::scenario::{
    e142_parameters, run_first_transaction, ScenarioPoint, E142_FILESYSTEM_IDENTIFIER,
    FIRST_FILE_BYTES, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_harness::sha256::sha256_hexadecimal;
use singlefs_harness::{
    RecordedOperationKind, RecordingBlockDevice, RetainedOperation, SharedStream,
};

/// E142 那套几何：两块同构盘，`physical_block_size` = io_min = 512。
/// 值抄自跑前登记第一节「装置写在哪」第 2 条 ①：模型
/// `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs` 第 132 行 `DEVICE_BYTES`、
/// 第 15 行 `PHYSICAL_BLOCK_BYTES`（两者在这个装置里都是 512 那一档的推论，逐项断言见 `main` 开头）。
const DEVICE_COUNT: u32 = 2;
const PHYSICAL_BLOCK_SIZE_IN_BYTES: u32 = 512;
const MINIMUM_INPUT_OUTPUT_BYTES: u32 = 512;
/// 值抄自跑前登记同一条：模型第 245–247 行 `FIXED_FSID` / `FIXED_WRITE_TIME_SECONDS` / `FIRST_INODE_NUMBER`，
/// 文件 3000 字节、内容 `index % 251`、改动计数 = 3（改动计数不是这个导出要驱动的量，第一个事务只跑一次）。
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

/// 屏障没有字节内容（不算一次「写」，R2）；写清零的内容按长度铺 0——与 `lib.rs` 的模块注释同一个理由
/// （整段全 0 由 offset 与 length 全定，重放的人不用真的把它摆出来）。第一个事务窗口按跑前登记的几何
/// 不会出现大到扛不住的写清零，这里直接摆出来。
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

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if !arguments.is_empty() {
        eprintln!("e142_first_transaction_write_dump 不收参数，收到 {arguments:?}");
        std::process::exit(1);
    }

    // 绝对值断言（门禁 80 号）：4 GiB = 4 × 1024 × 1024 × 1024，独立算出的字面量，不从代码读回来。
    // 两边在编译期就定了，clippy 建议挪进 const 块（否则判「assertions_on_constants」）。
    const {
        assert!(
            TEST_IMAGE_DEFAULT_BYTES == 4_294_967_296,
            "跑前登记第一节写死两块 4 GiB 盘；singlefs_format::TEST_IMAGE_DEFAULT_BYTES 变了这一格要重登记"
        );
        assert!(FIRST_FILE_BYTES == 3000, "第一个文件字节数与跑前登记第一节抄的模型同名常量回比");
        assert!(EXPECTED_FILE_BYTES == 3000, "本地常量与上面这条断言钉的是同一个数");
    };

    let parameters = e142_parameters(PHYSICAL_BLOCK_SIZE_IN_BYTES, MINIMUM_INPUT_OUTPUT_BYTES);
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0
        ..DEVICE_COUNT)
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    SparseBlockDevice::new(
                        TEST_IMAGE_DEFAULT_BYTES,
                        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_SIZE_IN_BYTES),
                    ),
                    stream.clone(),
                ),
            )
        })
        .collect();

    // 窗口的起点：`ScenarioPoint::BeforeFirstTransaction` 那一次回调时录制流的长度
    // （与 `first_transaction_region_bytes.rs:94-101` 同一个取法，跑前登记第一节「装置写在哪」第 2 条末句）。
    let mut steps_before_first_transaction: Option<usize> = None;
    let run = run_first_transaction(&parameters, &mut devices, &stream, |point, _devices| {
        match point {
            ScenarioPoint::AfterMakeFilesystem | ScenarioPoint::AfterInstanceAcquisition => {}
            ScenarioPoint::BeforeFirstTransaction => {
                steps_before_first_transaction = Some(stream.operation_count());
            }
        }
    })
    .expect("整条路（mkfs → 取号 → 暖机 → 第一个事务）在内存盘上跑得通");
    let steps_before_first_transaction = steps_before_first_transaction
        .expect("run_first_transaction 一定走过 BeforeFirstTransaction");

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
        let bytes = operation_bytes(retained)
            .expect("write / write_fua / write_zeroes 都能重建出完整字节");
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
    emitter.emit(&format!(
        "name=device_region_bytes_summary window_operations={} writes={write_count} barriers={barrier_count}",
        window.len()
    ));
    emitter.finish();
}

#[cfg(test)]
mod tests {
    use super::{kind_name, operation_bytes};
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
            operation: RecordedOperation {
                device: DeviceIdentity(0),
                kind,
                offset: DeviceOffsetInBytes(0),
                length,
                content_hash: 0,
            },
            contents,
        }
    }

    #[test]
    fn operation_bytes_returns_the_retained_content_for_a_plain_write() {
        let write = retained(RecordedOperationKind::Write, 3, Some(vec![1, 2, 3]));
        assert_eq!(operation_bytes(&write), Some(vec![1, 2, 3]));
    }

    #[test]
    fn operation_bytes_returns_the_retained_content_for_a_force_unit_access_write() {
        let write = retained(RecordedOperationKind::WriteForceUnitAccess, 2, Some(vec![9, 9]));
        assert_eq!(operation_bytes(&write), Some(vec![9, 9]));
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
}
