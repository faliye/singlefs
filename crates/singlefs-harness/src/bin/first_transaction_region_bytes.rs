//! E142（第一个事务的干跑） 量 5 的实装一侧：在两块内存盘上跑 `scenario::run_first_transaction`
//! （与虚机档、与 E142 装置同参数：同 fsid、同写入时刻、同 3000 字节内容、两盘 4 GiB），
//! 把第一个事务写到的那 21 个区域的字节打成一行一个区域的结果行。
//!
//!   first_transaction_region_bytes
//!
//! 不收参数、不读环境、不碰真设备、不写任何文件：**只读导出**，写路径一个字节都不改。
//! 结果行形态照 `first_transaction_on_device`：`E7RESULT name=… `，末行报条数。
//! 区域清单与它的来历见 `singlefs_harness::first_transaction_regions` 的模块注释。
//!
//! 退出码：0 = 区域表与这一趟真正发出的 21 条写逐条对得上；1 = 对不上（表与实装分叉了，两边都要查）
//! 或者命令行给了参数。

use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_format::{FIRST_TRANSACTION_TXG, TEST_IMAGE_DEFAULT_BYTES};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::first_transaction_regions::{
    region_result_lines, region_table_against_writes, HexadecimalExtent, FIRST_TRANSACTION_REGIONS,
    FIRST_TRANSACTION_REGION_COUNT,
};
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::scenario::{
    e142_parameters, run_first_transaction, ScenarioPoint, E142_FILESYSTEM_IDENTIFIER,
    FIRST_FILE_BYTES, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// E142 那套几何：两块同构盘，`physical_block_size` = io_min = 512。
const DEVICE_COUNT: u32 = 2;
const PHYSICAL_BLOCK_SIZE_IN_BYTES: u32 = 512;
const MINIMUM_INPUT_OUTPUT_BYTES: u32 = 512;

/// 空清单在结果行里写 `none`，不留一个空值——空值与「这一格没打出来」在文本里分不开。
fn list_or_none(list: &str) -> &str {
    if list.is_empty() {
        "none"
    } else {
        list
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

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if !arguments.is_empty() {
        eprintln!("first_transaction_region_bytes 不收参数，收到 {arguments:?}");
        std::process::exit(1);
    }

    let parameters = e142_parameters(PHYSICAL_BLOCK_SIZE_IN_BYTES, MINIMUM_INPUT_OUTPUT_BYTES);
    let stream = SharedStream::new();
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

    // 第一个事务那一段在流里从哪一步起：暖机之后的那个口子上拍一次长度，之后的每一步都是这个事务发出的。
    let mut steps_before_first_transaction: Option<usize> = None;
    let run =
        run_first_transaction(
            &parameters,
            &mut devices,
            &stream,
            |point, _devices| match point {
                ScenarioPoint::AfterInstanceAcquisition => {}
                ScenarioPoint::BeforeFirstTransaction => {
                    steps_before_first_transaction = Some(stream.operations().len());
                }
            },
        )
        .expect("整条路（mkfs → 取号 → 暖机 → 第一个事务）在内存盘上跑得通");
    let steps_before_first_transaction = steps_before_first_transaction
        .expect("run_first_transaction 一定走过 BeforeFirstTransaction");

    let image = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner().image.clone()))
            .collect(),
        device_size_in_bytes: TEST_IMAGE_DEFAULT_BYTES,
    };
    let operations = stream.operations();
    let against_writes = region_table_against_writes(&operations[steps_before_first_transaction..]);

    let mut emitter = Emitter { emitted: 0 };
    emitter.emit(&format!(
        "name=impl_config devices={DEVICE_COUNT} image_bytes={TEST_IMAGE_DEFAULT_BYTES} physical_block_size={PHYSICAL_BLOCK_SIZE_IN_BYTES} minimum_input_output_bytes={MINIMUM_INPUT_OUTPUT_BYTES} file_bytes={FIRST_FILE_BYTES} write_time_seconds={FIXED_WRITE_TIME_SECONDS} filesystem_identifier={} checkpoint_txg={FIRST_TRANSACTION_TXG} mkfs_operations={} policy_mismatches={}",
        hexadecimal_text(&E142_FILESYSTEM_IDENTIFIER),
        run.mkfs_operation_count,
        run.policy_mismatches
    ));
    for line in region_result_lines(&image) {
        emitter.emit(&line);
    }
    let whole_region_rows = FIRST_TRANSACTION_REGIONS
        .iter()
        .filter(|region| region.hexadecimal_extent == HexadecimalExtent::WholeRegion)
        .count();
    emitter.emit(&format!(
        "name=impl_region_bytes_summary regions={FIRST_TRANSACTION_REGION_COUNT} whole_region={whole_region_rows} head_and_tail={}",
        FIRST_TRANSACTION_REGION_COUNT - whole_region_rows
    ));
    emitter.emit(&format!(
        "name=impl_region_table_against_writes regions={FIRST_TRANSACTION_REGION_COUNT} write_calls={} regions_without_a_write={} writes_outside_the_table={} matches={}",
        against_writes.write_calls,
        list_or_none(&against_writes.regions_without_a_write.join(",")),
        list_or_none(
            &against_writes
                .writes_outside_the_table
                .iter()
                .map(|(device, offset, length)| format!("device{}@{}+{length}", device.0, offset.0))
                .collect::<Vec<String>>()
                .join(",")
        ),
        against_writes.matches()
    ));
    emitter.finish();
    if !against_writes.matches() {
        eprintln!(
            "区域表与第一个事务真正发出的写对不上：见 name=impl_region_table_against_writes 那一行"
        );
        std::process::exit(1);
    }
}
