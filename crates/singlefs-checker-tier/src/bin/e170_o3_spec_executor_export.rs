//! E170（O3规约执行器第一版） 第一段：入库导出装置。
//!
//! 登记：`research/prompts/e170-preregistration.md` 5.3（导出格式）、5.4（取样三批）、5.7（装置写在哪）、5.8「第一段」、
//! 第十一节停机 S1、S4 与作废 V2、第九节 XE1–XE3。它回答问题单 Q2 的实现那一侧：两条层 0 流的录制写流按 5.3 的中性格式写出，
//! 按 5.4 取三批状态，每个状态调 `crates/` 的层 0 判定（`singlefs_checker_tier::crash::judge_crash_image`）记实现的判定与单线程耗时。
//!
//! 只读、只驱动、只观测：不改 `crates/singlefs-core`、`crates/singlefs-checker`、harness 与 format。
//! 流一（新池新建文件）经 `#[path]` 引 `crates/singlefs-harness/tests/common/mod.rs` 的 `format_pool`、`parameters` 与内容生成式，
//! 把 `build_pool` 里取号、暖机、新池新建文件三步照抄过来、每次调用返回时记流长（`calls.tsv` 的返回点），
//! 再拿 `build_pool` 整条流逐条回比（S1）；流二照抄 `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs`
//! 的 `prepare` 里 `Script::ReuseAfterRaisingFloorThenNormalUnmount` 那一支（第 218 行起）。
//! 决定几何与工作量的数写成本地常量（值抄自那两份测试文件与 `tests/common/mod.rs`，逐个注明），`main` 开头与 crates 的同名
//! 公开常量或现算值回比，不直接引（登记 5.7、S1）。
//!
//!   e170_o3_spec_executor_export dump --output-directory <目录> [--compare-with <上一次导出的目录>]
//!   e170_o3_spec_executor_export bad-states --dump-directory <dump 写出的目录> --seed-hex <第二段冻结的种子，16 个十六进制字符>
//!
//! 第三段（登记 5.5、5.8「第三段」）的 `bad-states` 模式见文件末尾「第三段：坏状态」一节：P 行、N 行、盲测族造在第一段导出的同一批
//! 流上，每行过一遍实现的判定，写进 `<目录>/<流名>/bad-*.tsv` 与 `bad-patches.bin`；回比没过同样退 3。
//!
//! 每条流写进 `<目录>/<流名>/`（流名 `new_pool_file_creation`、`fixed_script_through_e_then_unmount`），目录已存在就拒绝。
//! 给了 `--compare-with` 时逐个文件比 sha256（V2），另按登记第十二节修订比去掉耗时那一列之后的两个文件（V2b）。
//! 标准输出：`E7RESULT` 行（与跑几次无关，入库产物只留它们）与 `E7TIMING` 行（耗时与带耗时的文件摘要）；收尾 `name=done emitted=N`。
//! 退出码：0 正常；2 命令行不对或输出目录已存在；3 停机条款 S1 或 S4 没过（产物照打，交主 agent）。
//! 单线程：每个状态的 `judge_crash_image` 耗时在这个进程里用 `Instant` 计（登记 5.3 `impl-verdicts.tsv`）。

#[path = "../../../singlefs-harness/tests/common/mod.rs"]
mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker_tier::crash::{
    full_expansion, judge_crash_image, judge_layer0_state_range, layer0_enumeration_tables,
    layer0_plan_state_count, quick_tier_expansion, Layer0EnumerationTables, Layer0StateJudgement,
};
use singlefs_checker_tier::crash_facts::{extract_flow_facts, FlowFacts};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::journal::{
    back_chain_of, record_offset, JournalRecord, JournalRecordOrdinalWithinPublish,
    JournalRecordPlaceInPublish, JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET,
};
use singlefs_core::make_filesystem::{INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, RollbackTarget,
    ShadowLedger, UnmountRaisedTheFloor, Unmounted,
};
use singlefs_core::recovery::{PoolReader, RecoveryOutcome, RecoveryReport};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::system_configuration::{
    SystemConfiguration, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion,
    PoolWriter, TransactionOutput,
};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_harness::memory_pool::{
    publishes_of_one_recording, writes_and_segments_with_stream_indexes_and_entries, CrashImage,
    MemoryPool, PublishWrites, PublishedVersion, RetainedWrite, WrittenContents,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::{sha256_digest, sha256_hexadecimal, DIGEST_BYTES};
use singlefs_harness::{
    RecordedEntrySpan, RecordedOperationKind, RecordedPublishEntry, RetainedOperation, SharedStream,
};

// ───────────────────────────── 本地常量（登记 5.7、S1） ─────────────────────────────

/// 盘数：`tests/common/mod.rs` 第 152–184 行 `formatted_recorded_images` 建两块盘（`for device_number in 0..2u32`）。
const LOCAL_DEVICE_COUNT: usize = 2;
/// 每块盘的字节数：`tests/common/mod.rs` 第 29 行 `IMAGE_BYTES = 4 << 30`。
const LOCAL_DEVICE_BYTES: u64 = 4_294_967_296;
/// 物理块与 io_min：`tests/common/mod.rs` 第 54–55 行 `physical_block_size: 512`、`minimum_input_output_bytes: 512`。
const LOCAL_PHYSICAL_BLOCK_BYTES: u32 = 512;
const LOCAL_MINIMUM_INPUT_OUTPUT_BYTES: u32 = 512;
/// 导出格式的扇区宽（登记 5.3 `sector_bytes`（512））：与 `singlefs_harness::memory_pool::SECTOR_BYTES` 回比。
const LOCAL_SECTOR_BYTES: u64 = 512;
/// 固定 fsid：`tests/common/mod.rs` 第 33–35 行 `E142_FILESYSTEM_IDENTIFIER`。
const LOCAL_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
/// 固定写时刻：`tests/common/mod.rs` 第 37 行 `FIXED_WRITE_TIME_SECONDS = 1_788_000_000`。
const LOCAL_FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 第一个文件的字节数：`tests/common/mod.rs` 第 31 行 `FILE_BYTES = 3000`；内容 `index % 251`（第 71–75 行 `file_content`）。
const LOCAL_FIRST_FILE_BYTES: usize = 3000;
const LOCAL_FIRST_FILE_CONTENT_MODULUS: usize = 251;

/// 流一的段长：`tests/crash_enumeration_new_pool_file_creation_stream.rs` 第 466 行。
const LOCAL_FIRST_STREAM_SEGMENT_LENGTHS: [usize; 11] = [2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2];
/// 流一的写数：同一份文件第 471 行（取号 2 + 暖机两次各 5 + A 29）。
const LOCAL_FIRST_STREAM_WRITES: usize = 41;
/// 流一的根槽写数：暖机两次各一条 + A 一条。
const LOCAL_FIRST_STREAM_ROOT_WRITES: usize = 3;
/// 流一全量与快档的状态数：同一份文件第 446 行 `FULL_STATES`、第 450 行 `QUICK_TIER_STATES`。
const LOCAL_FIRST_STREAM_FULL_STATES: u64 = 16_777_260;
const LOCAL_FIRST_STREAM_QUICK_TIER_STATES: u64 = 46;
/// 流一每次写只取两态时的全量闭式：同一份文件第 442 行 `FULL_STATES_WITH_TWO_STATES_PER_WRITE`（单测拿段长表回算它）。
const LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM: u64 = 16_777_240;

/// 流二的段长（78 段）：`tests/crash_enumeration_fixed_script_stream.rs` 第 438–441 行。
const LOCAL_SECOND_STREAM_SEGMENT_LENGTHS: [usize; 78] = [
    2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2,
    16, // 取号到写行之后的暖机
    2, 1, 2, 24, 2, 1, 2, 20, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2,
    28, // C、回退 D、覆盖写
    2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1,
    2, // 抬 F、E、卸载
    16, 2, 1, 2, // 卸载第二次空发布与 txg 19 的轮换
];
/// 流二的写数：同一份文件第 448 行 `191 + 5 * 33 + (2 + 2 * 21) + 33 + (2 + 2 * 21)`。
const LOCAL_SECOND_STREAM_WRITES: usize = 477;
/// 流二的根槽写数：同一份文件第 482 行（`=> 19`）。
const LOCAL_SECOND_STREAM_ROOT_WRITES: usize = 19;
/// 流二全量与快档的状态数：同一份文件第 100 行 `FULL_STATES_THROUGH_THE_UNMOUNT`、第 114 行 `QUICK_TIER_STATES_THROUGH_THE_UNMOUNT`。
const LOCAL_SECOND_STREAM_FULL_STATES: u64 = 1_662_648_564;
const LOCAL_SECOND_STREAM_QUICK_TIER_STATES: u64 = 278;
/// 流二每次写只取两态时的全量闭式：同一份文件第 96 行（单测拿段长表回算它）。
const LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM: u64 = 1_662_648_449;
/// E 的数据单元落点：同一份文件第 318 行（`50176`）。
const LOCAL_SECOND_STREAM_E_DATA_UNIT_SLOT: u64 = 50_176;
/// 抬 F 推的空发布数：同一份文件第 306 行（`raised.publishes.len(), 2`）。
const LOCAL_SECOND_STREAM_RAISE_FLOOR_PUBLISHES: usize = 2;
/// 卸载推的两次空发布的 txg：同一份文件第 336 行（`vec![CheckpointTxg(18), CheckpointTxg(19)]`）。
const LOCAL_SECOND_STREAM_UNMOUNT_TXGS: [u64; 2] = [18, 19];

/// 取样：批 1 每段内点的分母、批 2 的分母、批 3 的分母（登记 5.4）。
const FIRST_BATCH_DIVISOR: u64 = 7;
const SECOND_BATCH_DIVISOR: u64 = 14;
const THIRD_BATCH_DIVISOR: u64 = 23;
/// 批 1 在带撕裂写的段里另取几个带「1」的状态、每段最多扫几个序号（登记 5.4）。
const TORN_STATES_PER_SEGMENT: usize = 4;
const TORN_SCAN_ORDINALS_PER_SEGMENT: u64 = 100_000;

/// 两条流的名字（登记 5.3 的目录名）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StreamName {
    NewPoolFileCreation,
    FixedScriptThroughEThenUnmount,
}

impl StreamName {
    fn directory_name(self) -> &'static str {
        match self {
            Self::NewPoolFileCreation => "new_pool_file_creation",
            Self::FixedScriptThroughEThenUnmount => "fixed_script_through_e_then_unmount",
        }
    }
}

const STREAM_NAMES: [StreamName; 2] = [
    StreamName::NewPoolFileCreation,
    StreamName::FixedScriptThroughEThenUnmount,
];

// ───────────────────────────── 流二的内容生成式（照抄测试文件） ─────────────────────────────

/// `tests/crash_enumeration_fixed_script_stream.rs` 第 50–56 行。
const SECOND_FILE_BYTES: usize = 4100;

fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 同一份文件第 131–143 行。
const THIRD_FILE_BYTES: usize = 2500;

fn later_content(seed: usize) -> Vec<u8> {
    (0..3000 + seed)
        .map(|index| u8::try_from((index * 5 + seed) % 251).expect("小于 256"))
        .collect()
}

fn third_content() -> Vec<u8> {
    (0..THIRD_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
        .collect()
}

/// 第一个文件的内容按本地常量现算，与 `common::file_content()` 回比（S1）。
fn local_first_file_content() -> Vec<u8> {
    (0..LOCAL_FIRST_FILE_BYTES)
        .map(|index| {
            u8::try_from(index % LOCAL_FIRST_FILE_CONTENT_MODULUS).expect("模 251 小于 256")
        })
        .collect()
}

// ───────────────────────────── 驱动方记的调用（calls.tsv） ─────────────────────────────

/// 驱动方调的一次 API：标签、它发出的第一条操作的流序下标、返回点（返回时录制流的长度），都从 mkfs 之后数。
/// 失败的调用在这个装置里一律 `expect` 停下，写进表的只有 `ok`。
#[derive(Clone, Debug, PartialEq, Eq)]
struct RecordedCall {
    label: &'static str,
    first_operation: usize,
    return_point: usize,
}

/// 调 `run` 并记一行：调之前与返回时各读一次流长。
fn record_call<Output>(
    calls: &mut Vec<RecordedCall>,
    stream: &SharedStream,
    operations_before_the_recording: usize,
    label: &'static str,
    run: impl FnOnce() -> Output,
) -> Output {
    let first_operation = stream.operation_count() - operations_before_the_recording;
    let output = run();
    let return_point = stream.operation_count() - operations_before_the_recording;
    calls.push(RecordedCall {
        label,
        first_operation,
        return_point,
    });
    output
}

// ───────────────────────────── 驱动两条流 ─────────────────────────────

/// 一条停机条款 S1 的回比：本地值与 crates 那一侧的值，两边按文本比。
#[derive(Clone, Debug, PartialEq, Eq)]
struct StopCheck {
    name: String,
    local: String,
    crates: String,
}

impl StopCheck {
    fn new(name: &str, local: impl std::fmt::Debug, crates: impl std::fmt::Debug) -> Self {
        Self {
            name: name.to_string(),
            local: format!("{local:?}"),
            crates: format!("{crates:?}"),
        }
    }
    fn holds(&self) -> bool {
        self.local == self.crates
    }
}

/// 流一的三次调用：照抄 `tests/common/mod.rs` 第 498–557 行 `build_pool`，每次调用返回时记流长。
/// mkfs 那一截由 `common::format_pool` 做（与 `build_pool` 同一个 `formatted_recorded_images`）。
fn drive_through_the_new_pool_file_creation(
    tag: &str,
    calls: &mut Vec<RecordedCall>,
) -> common::BuiltPool {
    let mut formatted = common::format_pool(tag);
    // 镜像路径交给 BuiltPool 删：FormattedPool 的 Drop 拿到的是空表，什么都不删。
    let paths = std::mem::take(&mut formatted.paths);
    let mut devices = formatted
        .devices
        .take()
        .expect("format_pool 交回开着的设备");
    let stream = formatted.stream.clone();
    let genesis = formatted.genesis.clone();
    let mkfs_operation_count = formatted.mkfs_operation_count;
    drop(formatted);
    let parameters = common::parameters();
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), LOCAL_DEVICE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), LOCAL_DEVICE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    let content = local_first_file_content();
    let (warmed, output, warm_up_operation_count) = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = record_call(calls, &stream, mkfs_operation_count, "acquire", || {
            acquire_instance(&mut pool)
        })
        .expect("取号");
        assert_eq!(instance, InstanceGeneration(1), "新池第一次取号取 1");
        let warmed = record_call(calls, &stream, mkfs_operation_count, "warm_up", || {
            warm_up(&mut pool, &genesis.root, instance)
        })
        .expect("暖机");
        let warm_up_operation_count = stream.operation_count();
        let output = record_call(calls, &stream, mkfs_operation_count, "publish", || {
            publish_first_file(
                &mut pool,
                &mut allocator,
                warmed.roots.last().expect("暖机两代根"),
                FirstFile {
                    content: &content,
                    write_time_seconds: LOCAL_FIXED_WRITE_TIME_SECONDS,
                },
                instance,
                &warmed.last_record_bytes,
            )
        })
        .expect("新池新建文件");
        (warmed, output, warm_up_operation_count)
    };
    common::BuiltPool {
        paths,
        devices: Some(devices),
        stream,
        genesis,
        warm_up: warmed,
        output,
        allocator,
        mkfs_operation_count,
        warm_up_operation_count,
    }
}

/// 照抄 `tests/crash_enumeration_fixed_script_stream.rs` 第 145–166 行 `overwrite`（写时刻取本地常量 + 60）。
fn overwrite(
    pool: &mut common::BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let parameters = common::parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &pool.output,
        FirstFile {
            content,
            write_time_seconds: LOCAL_FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写")
}

/// 照抄同一份文件第 168–196 行 `unmount_recorded`。
fn unmount_recorded(pool: &mut common::BuiltPool) -> UnmountRaisedTheFloor {
    let mut current = PoolVersion::WithFile(pool.output.clone());
    let stream = pool.stream.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let allocator = &mut pool.allocator;
    let unmounted = stream
        .record_entry(RecordedPublishEntry::Unmount, || {
            unmount(
                &common::parameters(),
                devices,
                allocator,
                &mut current,
                ShadowLedger::On,
            )
        })
        .expect("正常卸载");
    pool.output = current
        .into_file_version()
        .expect("带文件的一版卸载之后仍带文件");
    match unmounted {
        Unmounted::FloorRaisedToTheCurrentVersion(raised) => raised,
        Unmounted::NothingWrittenOnAVersionWithoutFile {
            current: reported_version,
        } => {
            panic!("带文件的一版上卸载报成「树表 0 条、一个字节都不写」：{reported_version:?}")
        }
    }
}

fn version(instance: u32, checkpoint_txg: u64, content: Vec<u8>) -> PublishedVersion {
    PublishedVersion {
        instance: InstanceGeneration(instance),
        checkpoint_txg: CheckpointTxg(checkpoint_txg),
        content,
    }
}

/// 流二：照抄同一份文件第 218–356 行 `prepare` 里 `Script::ReuseAfterRaisingFloorThenNormalUnmount` 那一支，每次调用返回时记流长。
/// 脚本里那几条 `assert_eq!`（抬 F 两次空发布、E 落回 50176、卸载 txg 18 与 19）改成 S1 的回比，不在这里停。
fn drive_through_e_then_unmount(
    calls: &mut Vec<RecordedCall>,
) -> (common::BuiltPool, Vec<PublishedVersion>, Vec<StopCheck>) {
    let mut checks = Vec::new();
    let mut pool = drive_through_the_new_pool_file_creation("e170-second-stream", calls);
    let stream = pool.stream.clone();
    let mkfs_operation_count = pool.mkfs_operation_count;
    let second = record_call(calls, &stream, mkfs_operation_count, "publish", || {
        overwrite(&mut pool, &second_content(), InstanceGeneration(1))
    });
    let mut versions = vec![
        version(1, 3, local_first_file_content()),
        version(1, 4, second_content()),
    ];
    pool.output = second;
    // 进程重开：取号 2、写行、暖机两次（txg 5 / 6 落盘 0、txg 7 落盘 1），文件还是第二次的内容。
    let mounted = record_call(
        calls,
        &stream,
        mkfs_operation_count,
        "mount_writable",
        || {
            let mut devices = pool.reopen_recorded();
            let mounted = mount_writable(&common::parameters(), &mut devices).expect("可写挂载");
            pool.devices = Some(devices);
            mounted
        },
    );
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    for checkpoint_txg in 5..=7 {
        versions.push(version(2, checkpoint_txg, second_content()));
    }
    let third = record_call(calls, &stream, mkfs_operation_count, "publish", || {
        overwrite(&mut pool, &third_content(), InstanceGeneration(2))
    });
    versions.push(version(2, 8, third_content()));
    pool.output = third;
    // 同一个会话里挂着的时候回退到 A 的根：一次向前发布 D（txg 9 落盘 0），文件回到第一次的内容。
    record_call(calls, &stream, mkfs_operation_count, "roll_back", || {
        let rollback_parameters = common::parameters();
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        roll_back_by_a_forward_publish(
            &rollback_parameters,
            open_devices,
            &mut pool.allocator,
            &mut pool.output,
            RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
            },
        )
        .expect("挂着的时候回退到 A")
    });
    versions.push(version(2, 9, local_first_file_content()));
    let mut latest = Vec::new();
    for (checkpoint_txg, seed) in [(10u64, 13usize), (11, 17), (12, 19), (13, 23), (14, 29)] {
        latest = later_content(seed);
        let written = record_call(calls, &stream, mkfs_operation_count, "publish", || {
            overwrite(&mut pool, &latest, InstanceGeneration(2))
        });
        pool.output = written;
        versions.push(version(2, checkpoint_txg, latest.clone()));
    }
    let mut current = pool.output.clone();
    let raised = record_call(calls, &stream, mkfs_operation_count, "raise_floor", || {
        let raise_devices = pool.devices.as_mut().expect("镜像还开着");
        raise_rollback_floor(
            &common::parameters(),
            raise_devices,
            &mut pool.allocator,
            &mut current,
            CheckpointTxg(11),
            ShadowLedger::On,
        )
        .expect("抬 F")
    });
    checks.push(StopCheck::new(
        "second_stream_raise_floor_publishes",
        LOCAL_SECOND_STREAM_RAISE_FLOOR_PUBLISHES,
        raised.publishes.len(),
    ));
    pool.output = current;
    for checkpoint_txg in 15..=16 {
        versions.push(version(2, checkpoint_txg, latest.clone()));
    }
    let reuse = record_call(calls, &stream, mkfs_operation_count, "publish", || {
        overwrite(&mut pool, &later_content(31), InstanceGeneration(2))
    });
    checks.push(StopCheck::new(
        "second_stream_e_data_unit_slot",
        LOCAL_SECOND_STREAM_E_DATA_UNIT_SLOT,
        reuse.data_pointers[0].locations[0].slot.0,
    ));
    versions.push(version(2, 17, later_content(31)));
    pool.output = reuse;
    let unmounted = record_call(calls, &stream, mkfs_operation_count, "unmount", || {
        unmount_recorded(&mut pool)
    });
    checks.push(StopCheck::new(
        "second_stream_unmount_txgs",
        LOCAL_SECOND_STREAM_UNMOUNT_TXGS.to_vec(),
        unmounted
            .publishes
            .iter()
            .map(|publish| publish.root.checkpoint_txg.0)
            .collect::<Vec<u64>>(),
    ));
    for checkpoint_txg in 18..=19 {
        versions.push(version(2, checkpoint_txg, later_content(31)));
    }
    (pool, versions, checks)
}

/// 一条流驱动完之后的样子：mkfs 之后的基镜像、mkfs 之后的录制流、写表与段、调用表、被判的根、版本表、S1 回比。
struct PreparedStream {
    name: StreamName,
    base: MemoryPool,
    operations: Vec<RetainedOperation>,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    calls: Vec<RecordedCall>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    checks: Vec<StopCheck>,
}

fn prepare_stream(name: StreamName) -> PreparedStream {
    let mut calls = Vec::new();
    let (pool, versions_of_the_script, mut checks) = match name {
        StreamName::NewPoolFileCreation => (
            drive_through_the_new_pool_file_creation("e170-first-stream", &mut calls),
            None,
            Vec::new(),
        ),
        StreamName::FixedScriptThroughEThenUnmount => {
            let (pool, versions, checks) = drive_through_e_then_unmount(&mut calls);
            (pool, Some(versions), checks)
        }
    };
    let all_operations = pool.retained_operations();
    let mkfs_operation_count = pool.mkfs_operation_count;
    // 照抄 `tests/crash_enumeration_fixed_script_stream.rs` 第 198–216 行：入口段（卸载入口发的那几段）一起交给切段（C557）。
    let entry_spans: Vec<RecordedEntrySpan> = pool
        .stream
        .entry_spans()
        .into_iter()
        .map(|span| RecordedEntrySpan {
            entry: span.entry,
            operations: span.operations.start - mkfs_operation_count
                ..span.operations.end - mkfs_operation_count,
        })
        .collect();
    let operations = all_operations[mkfs_operation_count..].to_vec();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &operations,
        &entry_spans,
        &common::geometry(),
    );
    let base = pool.memory_pool_after_mkfs();
    let non_barrier_operations: Vec<usize> = operations
        .iter()
        .enumerate()
        .filter(|(_, retained)| retained.operation.kind != RecordedOperationKind::Barrier)
        .map(|(stream_index, _)| stream_index)
        .collect();
    let label = name.directory_name();
    checks.push(StopCheck::new(
        &format!("{label}_write_order_is_the_non_barrier_operation_order"),
        true,
        non_barrier_operations == stream_indexes,
    ));
    let (local_segment_lengths, local_writes, local_root_writes): (&[usize], usize, usize) =
        match name {
            StreamName::NewPoolFileCreation => (
                &LOCAL_FIRST_STREAM_SEGMENT_LENGTHS,
                LOCAL_FIRST_STREAM_WRITES,
                LOCAL_FIRST_STREAM_ROOT_WRITES,
            ),
            StreamName::FixedScriptThroughEThenUnmount => (
                &LOCAL_SECOND_STREAM_SEGMENT_LENGTHS,
                LOCAL_SECOND_STREAM_WRITES,
                LOCAL_SECOND_STREAM_ROOT_WRITES,
            ),
        };
    checks.push(StopCheck::new(
        &format!("{label}_segment_lengths"),
        local_segment_lengths.to_vec(),
        segments.iter().map(Vec::len).collect::<Vec<usize>>(),
    ));
    checks.push(StopCheck::new(
        &format!("{label}_writes"),
        local_writes,
        writes.len(),
    ));
    let root_write_count = writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .count();
    checks.push(StopCheck::new(
        &format!("{label}_root_writes"),
        local_root_writes,
        root_write_count,
    ));
    checks.push(StopCheck::new(
        &format!("{label}_device_count"),
        LOCAL_DEVICE_COUNT,
        base.devices.len(),
    ));
    checks.push(StopCheck::new(
        &format!("{label}_device_bytes"),
        LOCAL_DEVICE_BYTES,
        base.device_size_in_bytes,
    ));
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("写流里有根槽写");
    let versions = versions_of_the_script.unwrap_or_else(|| {
        // 流一的版本表照 `tests/crash_enumeration_new_pool_file_creation_stream.rs` 第 391–399 行 `single_version`。
        let (instance, checkpoint_txg) =
            singlefs_harness::memory_pool::root_identity_of_write(&writes[judged_root_index]);
        vec![PublishedVersion {
            instance,
            checkpoint_txg,
            content: local_first_file_content(),
        }]
    });
    if name == StreamName::NewPoolFileCreation {
        // 流一的驱动是 `build_pool` 的照抄：整条流（含 mkfs）逐条与 `build_pool` 那一份比。
        let reference = common::build_pool("e170-first-stream-reference");
        checks.push(StopCheck::new(
            "new_pool_file_creation_operations_equal_build_pool",
            true,
            reference.retained_operations() == all_operations,
        ));
        checks.push(StopCheck::new(
            "new_pool_file_creation_mkfs_operations_equal_build_pool",
            mkfs_operation_count,
            reference.mkfs_operation_count,
        ));
        checks.push(StopCheck::new(
            "new_pool_file_creation_warm_up_return_point_equal_build_pool",
            pool.warm_up_operation_count,
            reference.warm_up_operation_count,
        ));
    }
    drop(pool);
    PreparedStream {
        name,
        base,
        operations,
        writes,
        segments,
        calls,
        judged_root_index,
        versions,
        checks,
    }
}

// ───────────────────────────── 取样（登记 5.4） ─────────────────────────────

/// 批 1 在一个非空区间 [a, b) 里取的点：a、b − 1 与 a + ⌊i·(b − 1 − a)/7⌋（i = 1..6）。
fn first_batch_points(state_range: &std::ops::Range<u64>) -> Vec<u64> {
    let first = state_range.start;
    let span = state_range.end - 1 - first;
    let mut points = vec![first, state_range.end - 1];
    for step in 1..FIRST_BATCH_DIVISOR {
        points.push(first + step * span / FIRST_BATCH_DIVISOR);
    }
    points
}

/// 批 2：a + ⌊(2i + 1)·(b − 1 − a)/14⌋（i = 0..6）。
fn second_batch_points(state_range: &std::ops::Range<u64>) -> Vec<u64> {
    let first = state_range.start;
    let span = state_range.end - 1 - first;
    (0..SECOND_BATCH_DIVISOR / 2)
        .map(|step| first + (2 * step + 1) * span / SECOND_BATCH_DIVISOR)
        .collect()
}

/// 批 3：a + ⌊i·(b − 1 − a)/23⌋（i = 0..23）。
fn third_batch_points(state_range: &std::ops::Range<u64>) -> Vec<u64> {
    let first = state_range.start;
    let span = state_range.end - 1 - first;
    (0..=THIRD_BATCH_DIVISOR)
        .map(|step| first + step * span / THIRD_BATCH_DIVISOR)
        .collect()
}

/// 带撕裂写的那一段扫了几个序号、取到几个带「1」的状态（登记 5.4「扫不到 4 个的照实报个数」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TornScan {
    segment_index: usize,
    scanned: u64,
    found: usize,
}

/// 三批的状态序号（各自升序、互不重叠）与撕裂扫描的记录。
struct SampledOrdinals {
    batches: [BTreeSet<u64>; 3],
    torn_scans: Vec<TornScan>,
}

fn sample_ordinals(
    state_ranges_by_segment: &[std::ops::Range<u64>],
    state_count: u64,
    segment_has_a_torn_write: &[bool],
    landing_vector_of_ordinal: &dyn Fn(u64) -> String,
) -> SampledOrdinals {
    let non_empty = || {
        state_ranges_by_segment
            .iter()
            .filter(|range| !range.is_empty())
    };
    let mut first: BTreeSet<u64> = non_empty().flat_map(first_batch_points).collect();
    first.insert(state_count - 1);
    let mut torn_scans = Vec::new();
    for (segment_index, state_range) in state_ranges_by_segment.iter().enumerate() {
        if state_range.is_empty() || !segment_has_a_torn_write[segment_index] {
            continue;
        }
        let scan_end = state_range
            .end
            .min(state_range.start + TORN_SCAN_ORDINALS_PER_SEGMENT);
        let mut found = 0usize;
        let mut scanned = 0u64;
        for ordinal in state_range.start..scan_end {
            if found == TORN_STATES_PER_SEGMENT {
                break;
            }
            scanned += 1;
            if landing_vector_of_ordinal(ordinal).contains('1') && first.insert(ordinal) {
                found += 1;
            }
        }
        torn_scans.push(TornScan {
            segment_index,
            scanned,
            found,
        });
    }
    let second: BTreeSet<u64> = non_empty()
        .flat_map(second_batch_points)
        .filter(|ordinal| !first.contains(ordinal))
        .collect();
    let third: BTreeSet<u64> = non_empty()
        .flat_map(third_batch_points)
        .filter(|ordinal| !first.contains(ordinal) && !second.contains(ordinal))
        .collect();
    SampledOrdinals {
        batches: [first, second, third],
        torn_scans,
    }
}

// ───────────────────────────── 落地向量与镜像摘要（登记 5.3） ─────────────────────────────

/// 把枚举写表上的持久集合折回按写序号的三值向量：原写持久记 `2`；原写没持久而它的撕裂镜像持久记 `1`；都没有记 `0`。
fn landing_vector_of(
    persisted: &[bool],
    recorded_write_count: usize,
    torn_image_index_by_write: &BTreeMap<usize, usize>,
) -> String {
    (0..recorded_write_count)
        .map(|write_index| {
            if persisted[write_index] {
                '2'
            } else if torn_image_index_by_write
                .get(&write_index)
                .is_some_and(|torn_image_index| persisted[*torn_image_index])
            {
                '1'
            } else {
                '0'
            }
        })
        .collect()
}

/// 镜像摘要的一个扇区：全零的不进摘要；进的依次是盘身份 u32、扇区号 u64、那 512 字节（小端）。
fn append_sector_to_digest_message(
    message: &mut Vec<u8>,
    device: u32,
    sector: u64,
    sector_bytes: &[u8],
) {
    if sector_bytes.iter().all(|byte| *byte == 0) {
        return;
    }
    message.extend_from_slice(&device.to_le_bytes());
    message.extend_from_slice(&sector.to_le_bytes());
    message.extend_from_slice(sector_bytes);
}

/// 从 `CrashImage` 直接算镜像摘要：每块盘（身份升序）上可能不全为零的扇区——基镜像写过的扇区与这一态里持久了的带字节的写罩住的扇区——
/// 合成连续的区间按升序读（读走 `CrashImage` 自己的 `PoolReader::read`），全零的扇区跳过。
fn crash_image_digest(image: &CrashImage<'_>) -> [u8; DIGEST_BYTES] {
    let mut message = Vec::new();
    let mut device_identities = image.device_identities();
    device_identities.sort_unstable();
    for device in device_identities {
        let mut sector_ranges: Vec<(u64, u64)> = image
            .base
            .devices
            .get(&device)
            .map(|sparse| {
                sparse
                    .written_sectors()
                    .keys()
                    .map(|sector| (*sector, sector + 1))
                    .collect()
            })
            .unwrap_or_default();
        for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
            let carries_bytes = match write.contents {
                WrittenContents::Bytes(_) => true,
                WrittenContents::Zeros { .. } => false,
            };
            if *is_persisted && carries_bytes && write.device == device {
                sector_ranges.push((
                    write.offset.0 / LOCAL_SECTOR_BYTES,
                    (write.offset.0 + write.length_in_bytes()).div_ceil(LOCAL_SECTOR_BYTES),
                ));
            }
        }
        for (first_sector, end_sector) in merged_sector_ranges(sector_ranges) {
            let bytes = PoolReader::read(
                image,
                device,
                DeviceOffsetInBytes(first_sector * LOCAL_SECTOR_BYTES),
                usize::try_from((end_sector - first_sector) * LOCAL_SECTOR_BYTES)
                    .expect("区间长装得进 usize"),
            )
            .expect("崩溃镜像在盘内、按扇区对齐的区间读得出");
            for (sector_offset, sector_bytes) in bytes
                .chunks_exact(usize::try_from(LOCAL_SECTOR_BYTES).expect("512"))
                .enumerate()
            {
                let sector = first_sector + u64::try_from(sector_offset).expect("扇区下标");
                append_sector_to_digest_message(&mut message, device.0, sector, sector_bytes);
            }
        }
    }
    sha256_digest(&message)
}

/// 扇区区间按起点排好、重叠或相接的合并。
fn merged_sector_ranges(mut sector_ranges: Vec<(u64, u64)>) -> Vec<(u64, u64)> {
    sector_ranges.sort_unstable();
    let mut merged: Vec<(u64, u64)> = Vec::new();
    for (first_sector, end_sector) in sector_ranges {
        match merged.last_mut() {
            Some(last) if first_sector <= last.1 => last.1 = last.1.max(end_sector),
            Some(_) | None => merged.push((first_sector, end_sector)),
        }
    }
    merged
}

// ───────────────────────────── 中性格式：写出与读回重建（登记 5.3、S4） ─────────────────────────────

const BASE_SECTORS_FILE: &str = "base-sectors.bin";
const OPERATIONS_FILE: &str = "ops.tsv";
const OPERATIONS_PAYLOAD_FILE: &str = "ops-payload.bin";
const CALLS_FILE: &str = "calls.tsv";
const TORN_FILE: &str = "torn.tsv";
const TORN_PAYLOAD_FILE: &str = "torn-payload.bin";
const STATES_FILE: &str = "states.tsv";
const IMPLEMENTATION_VERDICTS_FILE: &str = "impl-verdicts.tsv";
const QUICK_TIER_FILE: &str = "quick-tier.tsv";
const MANIFEST_FILE: &str = "manifest.tsv";
/// manifest 记 sha256 的那几个文件，按这个次序。
const FILES_LISTED_IN_THE_MANIFEST: [&str; 9] = [
    BASE_SECTORS_FILE,
    OPERATIONS_FILE,
    OPERATIONS_PAYLOAD_FILE,
    CALLS_FILE,
    TORN_FILE,
    TORN_PAYLOAD_FILE,
    STATES_FILE,
    IMPLEMENTATION_VERDICTS_FILE,
    QUICK_TIER_FILE,
];

fn operation_kind_name(kind: RecordedOperationKind) -> &'static str {
    match kind {
        RecordedOperationKind::Write => "write",
        RecordedOperationKind::WriteForceUnitAccess => "write_fua",
        RecordedOperationKind::WriteZeroes => "write_zeroes",
        RecordedOperationKind::Barrier => "barrier",
    }
}

/// 读回来的一条操作：盘身份、种类、偏移、长度、在 `ops-payload.bin` 里的起点（清零与屏障没有）。
struct NeutralOperation {
    device: u32,
    kind: RecordedOperationKind,
    offset: u64,
    length: u64,
    payload_start: Option<usize>,
}

/// 从导出目录读回一条流：基镜像扇区、操作、两份载荷、撕裂表。只读这几个文件，不碰内存里的 `CrashImage`。
struct NeutralStream {
    base_sectors: BTreeMap<(u32, u64), Vec<u8>>,
    operations: Vec<NeutralOperation>,
    payload: Vec<u8>,
    torn_bytes_by_write: BTreeMap<usize, (usize, usize)>,
    torn_payload: Vec<u8>,
}

fn read_file(directory: &Path, file_name: &str) -> Vec<u8> {
    std::fs::read(directory.join(file_name))
        .unwrap_or_else(|error| panic!("读 {}：{error}", directory.join(file_name).display()))
}

fn read_text_lines(directory: &Path, file_name: &str) -> Vec<String> {
    String::from_utf8(read_file(directory, file_name))
        .expect("导出的 tsv 是 UTF-8")
        .lines()
        .map(str::to_string)
        .collect()
}

fn parse_number<Number: std::str::FromStr>(field: &str, what: &str) -> Number {
    field
        .parse()
        .unwrap_or_else(|_| panic!("{what} 解析不了：{field:?}"))
}

fn read_neutral_stream(directory: &Path) -> NeutralStream {
    let base_bytes = read_file(directory, BASE_SECTORS_FILE);
    let record_bytes = 4 + 8 + usize::try_from(LOCAL_SECTOR_BYTES).expect("512");
    assert!(
        base_bytes.len().is_multiple_of(record_bytes),
        "base-sectors.bin 是整条记录"
    );
    let base_sectors = base_bytes
        .chunks_exact(record_bytes)
        .map(|record| {
            let device = u32::from_le_bytes(record[0..4].try_into().expect("4 字节"));
            let sector = u64::from_le_bytes(record[4..12].try_into().expect("8 字节"));
            ((device, sector), record[12..].to_vec())
        })
        .collect();
    let operations = read_text_lines(directory, OPERATIONS_FILE)
        .iter()
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert_eq!(fields.len(), 7, "ops.tsv 一行七列：{line:?}");
            let kind = match fields[2] {
                "write" => RecordedOperationKind::Write,
                "write_fua" => RecordedOperationKind::WriteForceUnitAccess,
                "write_zeroes" => RecordedOperationKind::WriteZeroes,
                "barrier" => RecordedOperationKind::Barrier,
                unknown => panic!("ops.tsv 里认不出的种类 {unknown:?}"),
            };
            NeutralOperation {
                device: parse_number(fields[1], "盘身份"),
                kind,
                offset: parse_number(fields[3], "偏移"),
                length: parse_number(fields[4], "长度"),
                payload_start: (fields[5] != "-").then(|| parse_number(fields[5], "载荷起点")),
            }
        })
        .collect();
    let torn_bytes_by_write = read_text_lines(directory, TORN_FILE)
        .iter()
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert_eq!(fields.len(), 4, "torn.tsv 一行四列：{line:?}");
            (
                parse_number(fields[0], "写序号"),
                (
                    parse_number(fields[1], "撕裂字节起点"),
                    parse_number(fields[2], "撕裂字节长度"),
                ),
            )
        })
        .collect();
    NeutralStream {
        base_sectors,
        operations,
        payload: read_file(directory, OPERATIONS_PAYLOAD_FILE),
        torn_bytes_by_write,
        torn_payload: read_file(directory, TORN_PAYLOAD_FILE),
    }
}

/// 从中性格式按落地向量重建一态的镜像（按流序叠：`2` 放新字节、`1` 放 `torn-payload.bin` 里那一段、`0` 不动），算镜像摘要。
fn rebuilt_image_digest(stream: &NeutralStream, landing_vector: &str) -> [u8; DIGEST_BYTES] {
    let sector_bytes = usize::try_from(LOCAL_SECTOR_BYTES).expect("512");
    let mut sectors = stream.base_sectors.clone();
    let mut landings = landing_vector.chars();
    let mut write_index = 0usize;
    for operation in &stream.operations {
        if operation.kind == RecordedOperationKind::Barrier {
            continue;
        }
        let landing = landings.next().expect("落地向量与写数一样长");
        let first_sector = operation.offset / LOCAL_SECTOR_BYTES;
        let length = usize::try_from(operation.length).expect("写长装得进 usize");
        let bytes: Option<&[u8]> = match (landing, operation.kind) {
            ('0', _) => None,
            ('2', RecordedOperationKind::WriteZeroes) => {
                let end_sector = (operation.offset + operation.length) / LOCAL_SECTOR_BYTES;
                let covered: Vec<(u32, u64)> = sectors
                    .range((operation.device, first_sector)..(operation.device, end_sector))
                    .map(|(key, _)| *key)
                    .collect();
                for key in covered {
                    sectors.remove(&key);
                }
                None
            }
            ('2', _) => {
                let start = operation.payload_start.expect("带字节的写有载荷起点");
                Some(&stream.payload[start..start + length])
            }
            ('1', _) => {
                let (start, torn_length) = stream.torn_bytes_by_write[&write_index];
                assert_eq!(torn_length, length, "撕裂那一段罩整次写");
                Some(&stream.torn_payload[start..start + torn_length])
            }
            (unknown, _) => panic!("落地向量里认不出的字符 {unknown:?}"),
        };
        if let Some(bytes) = bytes {
            for (sector_offset, chunk) in bytes.chunks_exact(sector_bytes).enumerate() {
                let sector = first_sector + u64::try_from(sector_offset).expect("扇区下标");
                sectors.insert((operation.device, sector), chunk.to_vec());
            }
        }
        write_index += 1;
    }
    assert!(landings.next().is_none(), "落地向量比写数长");
    let mut message = Vec::new();
    for ((device, sector), bytes) in &sectors {
        append_sector_to_digest_message(&mut message, *device, *sector, bytes);
    }
    sha256_digest(&message)
}

// ───────────────────────────── 实现的判定（登记 5.3 `impl-verdicts.tsv`） ─────────────────────────────

fn root_text(root: Option<(InstanceGeneration, CheckpointTxg)>) -> String {
    match root {
        Some((instance, checkpoint_txg)) => format!("{},{}", instance.0, checkpoint_txg.0),
        None => "none".to_string(),
    }
}

/// outcome 的种类；走不下去的那一种带上失败的名字（空白换成下划线，不破坏 tsv 与结果行）。
fn outcome_text(outcome: &RecoveryOutcome) -> String {
    match outcome {
        RecoveryOutcome::NoFile { .. } => "no_file".to_string(),
        RecoveryOutcome::FileRead { .. } => "file_read".to_string(),
        RecoveryOutcome::Failed { failure, .. } => format!("failed:{failure:?}")
            .chars()
            .map(|character| {
                if character.is_whitespace() {
                    '_'
                } else {
                    character
                }
            })
            .collect(),
    }
}

/// 一个状态上实现的判定（登记第一节读法 6：「实现都放行」= 池级 checker 一条 Violated 都没有，且记录核对器两项都为假）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ImplementationVerdict {
    consulted_root: String,
    consulted_outcome: String,
    ignored_root: String,
    ignored_outcome: String,
    red_items: Vec<String>,
    pool_checker_red: Vec<String>,
    root_without_record: bool,
    claimed_state_missing_unit: bool,
}

impl ImplementationVerdict {
    fn of(judgement: &Layer0StateJudgement) -> Self {
        let report_root = |report: &RecoveryReport| root_text(report.effective_root);
        Self {
            consulted_root: report_root(&judgement.consulted),
            consulted_outcome: outcome_text(&judgement.consulted.outcome),
            ignored_root: report_root(&judgement.ignored),
            ignored_outcome: outcome_text(&judgement.ignored.outcome),
            red_items: judgement.red_items(),
            pool_checker_red: judgement
                .pool_checker
                .iter()
                .filter(|(_, verdict)| match verdict {
                    InvariantVerdict::Violated(_) => true,
                    InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => false,
                })
                .map(|(invariant, _)| (*invariant).to_string())
                .collect(),
            root_without_record: judgement.records.root_without_record,
            claimed_state_missing_unit: judgement.records.claimed_state_missing_unit,
        }
    }

    fn pool_checker_and_record_checker_both_pass(&self) -> bool {
        self.pool_checker_red.is_empty()
            && !self.root_without_record
            && !self.claimed_state_missing_unit
    }
}

fn list_text(items: &[String]) -> String {
    if items.is_empty() {
        "-".to_string()
    } else {
        items.join(",")
    }
}

/// 一个取样状态上看到的：批号、层 0 序号、段号、落地向量、两份镜像摘要之一（直接从 `CrashImage` 算的）、实现判定与耗时、两条自检。
struct StateObservation {
    batch: u8,
    ordinal: u64,
    segment_index: usize,
    landing_vector: String,
    image_digest: String,
    verdict: ImplementationVerdict,
    judge_nanoseconds: u128,
    mask_matches_the_facts: bool,
    rejudge_matches: bool,
}

/// 按序号升序把取样的状态切成连续的几段，每段调一次 `judge_layer0_state_range`：回调里给的那个 `CrashImage` 折回落地向量、算摘要，
/// 再在同一个镜像上单独计时调一次 `judge_crash_image`（单线程耗时；两次判定逐项比，记 `rejudge_matches`）。
/// 回调的持久集合另与 `crash_facts` 按序号现算的掩码比（记 `mask_matches_the_facts`）。
fn observe_sampled_states(
    stream: &PreparedStream,
    tables: &Layer0EnumerationTables,
    facts: &FlowFacts,
    torn_image_index_by_write: &BTreeMap<usize, usize>,
    sampled: &SampledOrdinals,
) -> Vec<StateObservation> {
    let mut batch_of_ordinal: BTreeMap<u64, u8> = BTreeMap::new();
    for (batch_offset, batch) in sampled.batches.iter().enumerate() {
        for ordinal in batch {
            let batch_number = u8::try_from(batch_offset + 1).expect("三批");
            assert!(
                batch_of_ordinal.insert(*ordinal, batch_number).is_none(),
                "三批互不重叠"
            );
        }
    }
    let ordinals: Vec<u64> = batch_of_ordinal.keys().copied().collect();
    let mut runs: Vec<std::ops::Range<u64>> = Vec::new();
    for ordinal in ordinals {
        match runs.last_mut() {
            Some(run) if run.end == ordinal => run.end += 1,
            Some(_) | None => runs.push(ordinal..ordinal + 1),
        }
    }
    let recorded_write_count = stream.writes.len();
    let mut observations: Vec<StateObservation> = Vec::new();
    for run in runs {
        let state_count_of_the_stream = judge_layer0_state_range(
            &stream.base,
            &stream.writes,
            &stream.segments,
            stream.judged_root_index,
            &stream.versions,
            &full_expansion,
            run,
            &mut |ordinal, image, judgement| {
                let started = Instant::now();
                let timed = judge_crash_image(image, stream.judged_root_index, &stream.versions);
                let judge_nanoseconds = started.elapsed().as_nanos();
                let verdict = ImplementationVerdict::of(judgement);
                observations.push(StateObservation {
                    batch: batch_of_ordinal[&ordinal],
                    ordinal,
                    segment_index: tables
                        .state_ranges_by_segment
                        .partition_point(|state_range| state_range.end <= ordinal),
                    landing_vector: landing_vector_of(
                        &image.persisted,
                        recorded_write_count,
                        torn_image_index_by_write,
                    ),
                    image_digest: sha256_text(&crash_image_digest(image)),
                    rejudge_matches: ImplementationVerdict::of(&timed) == verdict,
                    verdict,
                    judge_nanoseconds,
                    mask_matches_the_facts: facts.persisted_writes_of_state(ordinal)
                        == image.persisted,
                });
            },
        );
        assert_eq!(
            state_count_of_the_stream, tables.state_count,
            "judge_layer0_state_range 交回的整条流状态数与枚举表的相同"
        );
    }
    observations.sort_by_key(|observation| (observation.batch, observation.ordinal));
    observations
}

fn sha256_text(digest: &[u8; DIGEST_BYTES]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ───────────────────────────── 一条流导出 ─────────────────────────────

/// 写出的一个文件：名字、字节数、sha256；带耗时的两个文件另算一份去掉耗时之后的 sha256（登记第十二节修订 V2b）。
struct WrittenFile {
    name: &'static str,
    bytes: usize,
    sha256: String,
    sha256_without_timing: Option<String>,
}

/// `impl-verdicts.tsv` 每行去掉最后一列（耗时）；`manifest.tsv` 去掉记 `impl-verdicts.tsv` 摘要的那一行。其余文件原样。
fn contents_without_timing(file_name: &str, contents: &[u8]) -> Option<Vec<u8>> {
    let text = std::str::from_utf8(contents).ok()?;
    let kept: Vec<String> = match file_name {
        IMPLEMENTATION_VERDICTS_FILE => text
            .lines()
            .map(|line| {
                line.rsplit_once('\t')
                    .map_or(line, |(without_timing, _)| without_timing)
                    .to_string()
            })
            .collect(),
        MANIFEST_FILE => text
            .lines()
            .filter(|line| !line.starts_with(&format!("sha256:{IMPLEMENTATION_VERDICTS_FILE}\t")))
            .map(str::to_string)
            .collect(),
        _ => return None,
    };
    Some(
        kept.iter()
            .flat_map(|line| format!("{line}\n").into_bytes())
            .collect(),
    )
}

fn write_export_file(directory: &Path, name: &'static str, contents: &[u8]) -> WrittenFile {
    std::fs::write(directory.join(name), contents)
        .unwrap_or_else(|error| panic!("写 {}：{error}", directory.join(name).display()));
    WrittenFile {
        name,
        bytes: contents.len(),
        sha256: sha256_hexadecimal(contents),
        sha256_without_timing: contents_without_timing(name, contents)
            .map(|kept| sha256_hexadecimal(&kept)),
    }
}

struct StreamExport {
    name: StreamName,
    checks: Vec<StopCheck>,
    calls: Vec<RecordedCall>,
    operation_count: usize,
    write_count: usize,
    segment_count: usize,
    tearable_write_count: usize,
    full_states: u64,
    quick_tier_states: u64,
    torn_scans: Vec<TornScan>,
    observations: Vec<StateObservation>,
    files: Vec<WrittenFile>,
    rebuilt_digest_mismatches: usize,
    rebuilt_states: usize,
    elapsed_nanoseconds: u128,
}

fn export_stream(name: StreamName, output_root: &Path) -> StreamExport {
    let started = Instant::now();
    let stream = prepare_stream(name);
    let mut checks = stream.checks.clone();
    let tables = layer0_enumeration_tables(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let full_states = layer0_plan_state_count(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let quick_tier_states = layer0_plan_state_count(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &quick_tier_expansion,
    );
    let facts = extract_flow_facts(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let label = name.directory_name();
    let (local_full, local_quick, local_closed_form) = match name {
        StreamName::NewPoolFileCreation => (
            LOCAL_FIRST_STREAM_FULL_STATES,
            LOCAL_FIRST_STREAM_QUICK_TIER_STATES,
            LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM,
        ),
        StreamName::FixedScriptThroughEThenUnmount => (
            LOCAL_SECOND_STREAM_FULL_STATES,
            LOCAL_SECOND_STREAM_QUICK_TIER_STATES,
            LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM,
        ),
    };
    checks.push(StopCheck::new(
        &format!("{label}_full_states"),
        local_full,
        full_states,
    ));
    checks.push(StopCheck::new(
        &format!("{label}_quick_tier_states"),
        local_quick,
        quick_tier_states,
    ));
    checks.push(StopCheck::new(
        &format!("{label}_two_state_closed_form"),
        local_closed_form,
        singlefs_harness::memory_pool::closed_form_state_count(&stream.segments),
    ));
    checks.push(StopCheck::new(
        &format!("{label}_enumeration_tables_state_count"),
        full_states,
        tables.state_count,
    ));
    checks.push(StopCheck::new(
        &format!("{label}_flow_facts_state_count"),
        full_states,
        facts.state_count,
    ));
    let torn_image_index_by_write: BTreeMap<usize, usize> = tables
        .torn_images
        .iter()
        .map(|row| (row.write_index, row.torn_image_index))
        .collect();
    let segment_has_a_torn_write: Vec<bool> = stream
        .segments
        .iter()
        .map(|segment| {
            segment
                .iter()
                .any(|write_index| tables.is_tearable_by_write[*write_index])
        })
        .collect();
    let recorded_write_count = stream.writes.len();
    let sampled = sample_ordinals(
        &tables.state_ranges_by_segment,
        tables.state_count,
        &segment_has_a_torn_write,
        &|ordinal| {
            landing_vector_of(
                &facts.persisted_writes_of_state(ordinal),
                recorded_write_count,
                &torn_image_index_by_write,
            )
        },
    );
    let observations = observe_sampled_states(
        &stream,
        &tables,
        &facts,
        &torn_image_index_by_write,
        &sampled,
    );
    let directory = output_root.join(label);
    std::fs::create_dir_all(&directory)
        .unwrap_or_else(|error| panic!("建 {}：{error}", directory.display()));
    let files = write_stream_files(
        &directory,
        &stream,
        &tables,
        &observations,
        full_states,
        quick_tier_states,
    );
    // S4：只读刚写出的文件，按 states.tsv 的落地向量重建，摘要与 states.tsv 里（直接从 CrashImage 算的）那一列比。
    let neutral = read_neutral_stream(&directory);
    let mut rebuilt_states = 0usize;
    let mut rebuilt_digest_mismatches = 0usize;
    for line in read_text_lines(&directory, STATES_FILE) {
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 6, "states.tsv 一行六列：{line:?}");
        rebuilt_states += 1;
        if sha256_text(&rebuilt_image_digest(&neutral, fields[4])) != fields[5] {
            rebuilt_digest_mismatches += 1;
        }
    }
    StreamExport {
        name,
        checks,
        calls: stream.calls.clone(),
        operation_count: stream.operations.len(),
        write_count: stream.writes.len(),
        segment_count: stream.segments.len(),
        tearable_write_count: torn_image_index_by_write.len(),
        full_states,
        quick_tier_states,
        torn_scans: sampled.torn_scans,
        observations,
        files,
        rebuilt_digest_mismatches,
        rebuilt_states,
        elapsed_nanoseconds: started.elapsed().as_nanos(),
    }
}

/// `base-sectors.bin` 的内容（登记 5.3）：基镜像里每个不全为零的扇区，盘身份 u32、扇区号 u64、512 字节。
/// 第一段写导出、第三段核导出与重驱动的流相同（`bad-states` 模式）共用这一份。
fn base_sectors_file_bytes(stream: &PreparedStream) -> Vec<u8> {
    let mut base_sectors = Vec::new();
    for (device, sparse) in &stream.base.devices {
        for (sector, bytes) in sparse.written_sectors() {
            if bytes.iter().all(|byte| *byte == 0) {
                continue;
            }
            base_sectors.extend_from_slice(&device.0.to_le_bytes());
            base_sectors.extend_from_slice(&sector.to_le_bytes());
            base_sectors.extend_from_slice(bytes);
        }
    }
    base_sectors
}

/// `ops.tsv` 的文本与 `ops-payload.bin` 的字节（登记 5.3），同上两种模式共用。
fn operations_file_text_and_payload(stream: &PreparedStream) -> (String, Vec<u8>) {
    let mut operations_text = String::new();
    let mut payload = Vec::new();
    for (stream_index, retained) in stream.operations.iter().enumerate() {
        let operation = &retained.operation;
        let (payload_start, content_digest) = match operation.kind {
            RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
                let contents = retained
                    .contents
                    .as_ref()
                    .expect("流开了内容保留，普通写与 FUA 写带字节");
                let start = payload.len();
                payload.extend_from_slice(contents);
                (start.to_string(), sha256_hexadecimal(contents))
            }
            RecordedOperationKind::WriteZeroes | RecordedOperationKind::Barrier => {
                ("-".to_string(), "-".to_string())
            }
        };
        operations_text.push_str(&format!(
            "{stream_index}\t{}\t{}\t{}\t{}\t{payload_start}\t{content_digest}\n",
            operation.device.0,
            operation_kind_name(operation.kind),
            operation.offset.0,
            operation.length,
        ));
    }
    (operations_text, payload)
}

/// `calls.tsv` 的文本（登记 5.3），同上两种模式共用。
fn calls_file_text(stream: &PreparedStream) -> String {
    stream
        .calls
        .iter()
        .enumerate()
        .map(|(call_index, call)| {
            format!(
                "{call_index}\t{}\t{}\t{}\tok\n",
                call.label, call.first_operation, call.return_point
            )
        })
        .collect()
}

/// 按登记 5.3 写一条流的十个文件，manifest 最后写（它记前九个的 sha256）。
fn write_stream_files(
    directory: &Path,
    stream: &PreparedStream,
    tables: &Layer0EnumerationTables,
    observations: &[StateObservation],
    full_states: u64,
    quick_tier_states: u64,
) -> Vec<WrittenFile> {
    let base_sectors = base_sectors_file_bytes(stream);
    let (operations_text, payload) = operations_file_text_and_payload(stream);
    let calls_text = calls_file_text(stream);
    let mut torn_text = String::new();
    let mut torn_payload = Vec::new();
    let mut torn_rows: Vec<(usize, usize)> = tables
        .torn_images
        .iter()
        .map(|row| (row.write_index, row.torn_image_index))
        .collect();
    torn_rows.sort_unstable();
    for (write_index, torn_image_index) in torn_rows {
        let torn_bytes = tables.enumerated_writes[torn_image_index]
            .bytes()
            .expect("撕裂镜像是带字节的写");
        torn_text.push_str(&format!(
            "{write_index}\t{}\t{}\t{}\n",
            torn_payload.len(),
            torn_bytes.len(),
            sha256_hexadecimal(torn_bytes)
        ));
        torn_payload.extend_from_slice(torn_bytes);
    }
    let mut states_text = String::new();
    let mut verdicts_text = String::new();
    for (state_number, observation) in observations.iter().enumerate() {
        states_text.push_str(&format!(
            "{state_number}\t{}\t{}\t{}\t{}\t{}\n",
            observation.ordinal,
            observation.segment_index,
            observation.batch,
            observation.landing_vector,
            observation.image_digest
        ));
        let verdict = &observation.verdict;
        verdicts_text.push_str(&format!(
            "{state_number}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            verdict.consulted_root,
            verdict.consulted_outcome,
            verdict.ignored_root,
            verdict.ignored_outcome,
            list_text(&verdict.red_items),
            list_text(&verdict.pool_checker_red),
            verdict.root_without_record,
            verdict.claimed_state_missing_unit,
            observation.judge_nanoseconds
        ));
    }
    let quick_tier_text =
        format!("quick_tier_states\t{quick_tier_states}\nfull_states\t{full_states}\n");
    let mut files = vec![
        write_export_file(directory, BASE_SECTORS_FILE, &base_sectors),
        write_export_file(directory, OPERATIONS_FILE, operations_text.as_bytes()),
        write_export_file(directory, OPERATIONS_PAYLOAD_FILE, &payload),
        write_export_file(directory, CALLS_FILE, calls_text.as_bytes()),
        write_export_file(directory, TORN_FILE, torn_text.as_bytes()),
        write_export_file(directory, TORN_PAYLOAD_FILE, &torn_payload),
        write_export_file(directory, STATES_FILE, states_text.as_bytes()),
        write_export_file(
            directory,
            IMPLEMENTATION_VERDICTS_FILE,
            verdicts_text.as_bytes(),
        ),
        write_export_file(directory, QUICK_TIER_FILE, quick_tier_text.as_bytes()),
    ];
    let device_list: Vec<String> = stream
        .base
        .devices
        .keys()
        .map(|device| device.0.to_string())
        .collect();
    let mut manifest_text = format!(
        "stream\t{}\ndevices\t{}\ndevice_bytes\t{}\nsector_bytes\t{LOCAL_SECTOR_BYTES}\nops\t{}\nwrites\t{}\ncalls\t{}\nsampled_states\t{}\n",
        stream.name.directory_name(),
        device_list.join(","),
        stream.base.device_size_in_bytes,
        stream.operations.len(),
        stream.writes.len(),
        stream.calls.len(),
        observations.len(),
    );
    for (listed_name, file) in FILES_LISTED_IN_THE_MANIFEST.iter().zip(&files) {
        assert_eq!(*listed_name, file.name, "manifest 的次序与写出的次序相同");
        manifest_text.push_str(&format!("sha256:{}\t{}\n", file.name, file.sha256));
    }
    files.push(write_export_file(
        directory,
        MANIFEST_FILE,
        manifest_text.as_bytes(),
    ));
    files
}

// ───────────────────────────── 结果行、V2 与 main ─────────────────────────────

/// `E7RESULT` 行计数（收尾 `name=done emitted=N`，N 把 done 自己也算进去）；`E7TIMING` 行不计数、不进入库产物。
struct ResultLines {
    emitted: u64,
}

impl ResultLines {
    fn result(&mut self, body: &str) {
        self.emitted += 1;
        println!("E7RESULT {body}");
    }
    fn timing(body: &str) {
        println!("E7TIMING {body}");
    }
    fn finish(&mut self) {
        self.emitted += 1;
        println!("E7RESULT name=done emitted={}", self.emitted);
    }
}

/// 结果行里的值不许带空白（按空格切字段）。
fn field_text(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

/// S1 里与流无关的几条：本地常量对 `tests/common/mod.rs` 与 harness 的同名公开常量或现算值。
fn stream_independent_checks() -> Vec<StopCheck> {
    let parameters = common::parameters();
    vec![
        StopCheck::new("device_bytes", LOCAL_DEVICE_BYTES, common::IMAGE_BYTES),
        StopCheck::new(
            "physical_block_bytes",
            LOCAL_PHYSICAL_BLOCK_BYTES,
            parameters.geometry.physical_block_size,
        ),
        StopCheck::new(
            "minimum_input_output_bytes",
            LOCAL_MINIMUM_INPUT_OUTPUT_BYTES,
            parameters.geometry.minimum_input_output_bytes,
        ),
        StopCheck::new(
            "sector_bytes",
            LOCAL_SECTOR_BYTES,
            singlefs_harness::memory_pool::SECTOR_BYTES,
        ),
        StopCheck::new(
            "filesystem_identifier",
            LOCAL_FILESYSTEM_IDENTIFIER,
            common::E142_FILESYSTEM_IDENTIFIER,
        ),
        StopCheck::new(
            "fixed_write_time_seconds",
            LOCAL_FIXED_WRITE_TIME_SECONDS,
            common::FIXED_WRITE_TIME_SECONDS,
        ),
        StopCheck::new(
            "first_file_bytes",
            LOCAL_FIRST_FILE_BYTES,
            common::FILE_BYTES,
        ),
        StopCheck::new(
            "first_file_content",
            true,
            local_first_file_content() == common::file_content(),
        ),
    ]
}

/// 中位取排好序之后下标 ⌊(n − 1)/2⌋ 那一个，p95 取下标 ⌈0.95·n⌉ − 1 那一个。
fn timing_summary(mut nanoseconds: Vec<u128>) -> (u128, u128, u128, u128) {
    if nanoseconds.is_empty() {
        return (0, 0, 0, 0);
    }
    nanoseconds.sort_unstable();
    let count = nanoseconds.len();
    let total: u128 = nanoseconds.iter().sum();
    let p95_index = (count * 95).div_ceil(100) - 1;
    (
        total,
        nanoseconds[(count - 1) / 2],
        nanoseconds[p95_index],
        nanoseconds[count - 1],
    )
}

/// V2：同一个文件两次导出逐字节比；V2b：带耗时的两个文件去掉耗时之后再比。交回（文件名, 逐字节相同, 去掉耗时之后相同）。
fn compare_with_previous_export(
    directory: &Path,
    previous_directory: &Path,
) -> Vec<(&'static str, bool, Option<bool>)> {
    FILES_LISTED_IN_THE_MANIFEST
        .iter()
        .chain(std::iter::once(&MANIFEST_FILE))
        .map(|file_name| {
            let this_run = read_file(directory, file_name);
            let previous_run = read_file(previous_directory, file_name);
            let without_timing = contents_without_timing(file_name, &this_run)
                .map(|kept| Some(kept) == contents_without_timing(file_name, &previous_run));
            (*file_name, this_run == previous_run, without_timing)
        })
        .collect()
}

struct CommandLine {
    output_directory: PathBuf,
    compare_with: Option<PathBuf>,
}

fn parse_command_line(arguments: &[String]) -> Option<CommandLine> {
    let mut words = arguments.iter();
    if words.next().map(String::as_str) != Some("dump") {
        return None;
    }
    let mut output_directory = None;
    let mut compare_with = None;
    while let Some(word) = words.next() {
        match word.as_str() {
            "--output-directory" => output_directory = Some(PathBuf::from(words.next()?)),
            "--compare-with" => compare_with = Some(PathBuf::from(words.next()?)),
            _ => return None,
        }
    }
    Some(CommandLine {
        output_directory: output_directory?,
        compare_with,
    })
}

/// 一条流的结果行（`E7RESULT` 与 `E7TIMING`）；交回 (S1 没过的条数, S4 对不上的态数, 掩码对不上的态数, 两次判定不同的态数, 实现没两判都放行的态数)。
fn report_stream(lines: &mut ResultLines, export: &StreamExport) -> [usize; 5] {
    let label = export.name.directory_name();
    let mut failed_checks = 0usize;
    for check in &export.checks {
        failed_checks += usize::from(!check.holds());
        lines.result(&format!(
            "name=s1 stream={label} check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    lines.result(&format!(
        "name=stream stream={label} operations={} writes={} segments={} tearable_writes={} calls={} full_states={} quick_tier_states={} sampled_states={}",
        export.operation_count,
        export.write_count,
        export.segment_count,
        export.tearable_write_count,
        export.calls.len(),
        export.full_states,
        export.quick_tier_states,
        export.observations.len()
    ));
    for (call_index, call) in export.calls.iter().enumerate() {
        lines.result(&format!(
            "name=call stream={label} call={call_index} label={} first_operation={} return_point={} status=ok",
            call.label, call.first_operation, call.return_point
        ));
    }
    for scan in &export.torn_scans {
        lines.result(&format!(
            "name=torn_scan stream={label} segment={} scanned={} found={} wanted={TORN_STATES_PER_SEGMENT}",
            scan.segment_index, scan.scanned, scan.found
        ));
    }
    let mut not_both_pass = 0usize;
    for batch in 1..=3u8 {
        let in_batch: Vec<&StateObservation> = export
            .observations
            .iter()
            .filter(|observation| observation.batch == batch)
            .collect();
        let count_where = |predicate: &dyn Fn(&StateObservation) -> bool| {
            in_batch
                .iter()
                .filter(|observation| predicate(observation))
                .count()
        };
        let red_item_states = |prefix: &str| {
            count_where(&|observation| {
                observation
                    .verdict
                    .red_items
                    .iter()
                    .any(|item| item.starts_with(prefix))
            })
        };
        let both_pass = count_where(&|observation| {
            observation
                .verdict
                .pool_checker_and_record_checker_both_pass()
        });
        not_both_pass += in_batch.len() - both_pass;
        let segments: BTreeSet<usize> = in_batch
            .iter()
            .map(|observation| observation.segment_index)
            .collect();
        lines.result(&format!(
            "name=batch stream={label} batch={batch} states={} segments_touched={} states_with_a_torn_write={} pool_checker_and_record_checker_both_pass={both_pass} pool_checker_red_states={} record_checker_red_states={} consulted_oracle_red_states={} ignored_oracle_red_states={} consulted_effective_root_none={} consulted_failed_outcomes={}",
            in_batch.len(),
            segments.len(),
            count_where(&|observation| observation.landing_vector.contains('1')),
            red_item_states("pool_checker:"),
            red_item_states("record_checker:"),
            red_item_states("journal_consulted_oracle:"),
            red_item_states("journal_ignored_oracle:"),
            count_where(&|observation| observation.verdict.consulted_root == "none"),
            count_where(&|observation| observation.verdict.consulted_outcome.starts_with("failed:")),
        ));
        let (total, median, p95, maximum) = timing_summary(
            in_batch
                .iter()
                .map(|observation| observation.judge_nanoseconds)
                .collect(),
        );
        ResultLines::timing(&format!(
            "name=judge_time stream={label} batch={batch} states={} total_ns={total} median_ns={median} p95_ns={p95} max_ns={maximum}",
            in_batch.len()
        ));
    }
    let mask_mismatches = export
        .observations
        .iter()
        .filter(|observation| !observation.mask_matches_the_facts)
        .count();
    let rejudge_mismatches = export
        .observations
        .iter()
        .filter(|observation| !observation.rejudge_matches)
        .count();
    lines.result(&format!(
        "name=s4 stream={label} rebuilt_states={} digest_mismatches={}",
        export.rebuilt_states, export.rebuilt_digest_mismatches
    ));
    lines.result(&format!(
        "name=crash_facts_mask stream={label} states={} mismatches={mask_mismatches}",
        export.observations.len()
    ));
    lines.result(&format!(
        "name=rejudge stream={label} states={} mismatches={rejudge_mismatches}",
        export.observations.len()
    ));
    for file in &export.files {
        match &file.sha256_without_timing {
            None => lines.result(&format!(
                "name=file stream={label} file={} bytes={} sha256={}",
                file.name, file.bytes, file.sha256
            )),
            Some(without_timing) => {
                lines.result(&format!(
                    "name=file stream={label} file={} sha256_without_timing={without_timing}",
                    file.name
                ));
                ResultLines::timing(&format!(
                    "name=file_with_timing stream={label} file={} bytes={} sha256={}",
                    file.name, file.bytes, file.sha256
                ));
            }
        }
    }
    ResultLines::timing(&format!(
        "name=stream_time stream={label} elapsed_ns={}",
        export.elapsed_nanoseconds
    ));
    [
        failed_checks,
        export.rebuilt_digest_mismatches,
        mask_mismatches,
        rejudge_mismatches,
        not_both_pass,
    ]
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let first_word = arguments.first().map(String::as_str);
    if first_word == Some(BadTableVersion::FirstRun.mode_word())
        || first_word == Some(BadTableVersion::SecondRun.mode_word())
    {
        let Some(bad_states_command_line) = parse_bad_states_command_line(&arguments) else {
            eprintln!(
                "用法：e170_o3_spec_executor_export bad-states（或 bad-states-second-run）--dump-directory <第一段的导出目录> --seed-hex <16 个十六进制字符>；收到 {arguments:?}"
            );
            std::process::exit(2);
        };
        std::process::exit(run_bad_states(&bad_states_command_line));
    }
    let Some(command_line) = parse_command_line(&arguments) else {
        eprintln!(
            "用法：e170_o3_spec_executor_export dump --output-directory <目录> [--compare-with <上一次导出的目录>]，或 bad-states --dump-directory <目录> --seed-hex <种子>；收到 {arguments:?}"
        );
        std::process::exit(2);
    };
    if command_line.output_directory.exists() {
        eprintln!(
            "输出目录 {} 已存在：导出不覆盖已有的文件，换一个目录",
            command_line.output_directory.display()
        );
        std::process::exit(2);
    }
    let mut lines = ResultLines { emitted: 0 };
    lines.result(&format!(
        "name=config segment=s1 streams={} sector_bytes={LOCAL_SECTOR_BYTES} first_batch_divisor={FIRST_BATCH_DIVISOR} second_batch_divisor={SECOND_BATCH_DIVISOR} third_batch_divisor={THIRD_BATCH_DIVISOR} torn_states_per_segment={TORN_STATES_PER_SEGMENT} torn_scan_ordinals_per_segment={TORN_SCAN_ORDINALS_PER_SEGMENT} judge_threads=1 compared_with_a_previous_export={}",
        STREAM_NAMES.len(),
        command_line.compare_with.is_some()
    ));
    let mut failed_checks = 0usize;
    for check in stream_independent_checks() {
        failed_checks += usize::from(!check.holds());
        lines.result(&format!(
            "name=s1 stream=both check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    let mut totals = [0usize; 5];
    let mut v2_differing_files = 0usize;
    let mut v2_differing_files_without_timing = 0usize;
    for name in STREAM_NAMES {
        let export = export_stream(name, &command_line.output_directory);
        let stream_totals = report_stream(&mut lines, &export);
        for (total, value) in totals.iter_mut().zip(stream_totals) {
            *total += value;
        }
        if let Some(previous) = &command_line.compare_with {
            let label = name.directory_name();
            let comparisons = compare_with_previous_export(
                &command_line.output_directory.join(label),
                &previous.join(label),
            );
            for (file_name, identical, without_timing) in comparisons {
                v2_differing_files += usize::from(!identical);
                lines.result(&format!(
                    "name=v2 stream={label} file={file_name} identical={identical}"
                ));
                if let Some(identical_without_timing) = without_timing {
                    v2_differing_files_without_timing += usize::from(!identical_without_timing);
                    lines.result(&format!(
                        "name=v2_without_timing stream={label} file={file_name} identical={identical_without_timing}"
                    ));
                }
            }
        }
    }
    failed_checks += totals[0];
    let [_, rebuilt_digest_mismatches, mask_mismatches, rejudge_mismatches, not_both_pass] = totals;
    lines.result(&format!(
        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={rebuilt_digest_mismatches} crash_facts_mask_mismatches={mask_mismatches} rejudge_mismatches={rejudge_mismatches} states_where_pool_checker_or_record_checker_is_red={not_both_pass} v2_compared={} v2_differing_files={v2_differing_files} v2_differing_files_without_timing={v2_differing_files_without_timing}",
        command_line.compare_with.is_some()
    ));
    lines.finish();
    if failed_checks > 0 || rebuilt_digest_mismatches > 0 {
        std::process::exit(3);
    }
}

// ───────────────────────────── 第三段：坏状态（登记 5.5、5.8「第三段」） ─────────────────────────────
//
// `bad-states --dump-directory <第一段的导出目录> --seed-hex <第二段冻结的种子>`：重驱动两条流（与 `dump` 同一份驱动），先核重驱动的流
// 与导出目录里的 base-sectors.bin、ops.tsv、ops-payload.bin、calls.tsv 逐字节相同（manifest 记的 sha256），再按登记 5.5 造 P 行、N 行与
// 盲测族：底态（F、Mk、Mk⁺、Mk⁻、盲测的 Mk 型调用边界）+ 字节补丁。补丁用 `crates/` 的格式知识造（根记录、journal 记录、系统配置槽的
// 编码与重封都调 core 的写者），补丁不算写（A(c) 按底态算，登记第一节读法 3）。每行同时过一遍实现的判定（`judge_crash_image`，补丁
// 接在写表末尾当持久的写，种类记成系统配置槽写：`publishes_of_one_recording` 不把它算进任何一次发布）。
// 每条流写进 `<导出目录>/<流名>/`：`bad-states.tsv` 与 `bad-patches.bin`（执行器那一侧读：行号、底态落地向量、两个摘要、补丁），
// `bad-expected.tsv`（期望，只给比对器读）、`bad-impl-verdicts.tsv`（实现判定）、`bad-seed.tsv`（种子）。已存在就拒绝，不覆盖。

/// 登记 5.5 写死的改法数值（本地常量，值抄自登记 5.5 的 P 表、N 表与盲测族一段）。
const BAD_ROOT_RING_BASE_SLOT_PATCHED: u64 = 65;
const BAD_ROOT_RING_BASE_SLOT_FIRST_VERSION: u64 = 64;
const BAD_PSEUDO_ROOT_TXG_STEP: u64 = 3;
const BAD_WATERMARK_STEP: u64 = 1000;
const BAD_LOWERED_TXG_STEP: u64 = 2;
const BAD_ROOT_TREE_IDENTIFIER_WATERMARK_FIRST_BYTE: usize = 122;
const BAD_UNIT_FLIP_OFFSET: usize = 200;
const BAD_FLIPPED_BIT: u8 = 0x01;
const BLIND_ROWS_PER_STREAM: usize = 20;
/// 第 1 次重跑多出来的那一行（P10）与它要对照底态的那一行（P9）的行名；S5b 按这两个名字找行。
const LOWER_BOUND_ROW_NAME: &str = "P10-Mk+";
const FIRST_RUN_MK_PLUS_ROW_NAME: &str = "P9-Mk+";
const BLIND_PSEUDO_ROOT_STEPS: u64 = 7;
const BLIND_LOWEST_WATERMARK_STEP: u64 = 1000;
const BLIND_HIGHEST_WATERMARK_STEP: u64 = 1_000_000;
/// 记录的点名项区起点 = 记录头字节数（D23（journal 的角色与格式） 已定项 4：记录头 487 字节）；新根段起点 = 本次发布内序号 4 + 反向链 4 + 载荷校验和 4
/// 之后。两个数写成本地常量，`bad-states` 开头与 core 的公开常量回比（S1 同形，回比不过照打、退 3）。
const BAD_RECORD_NAMED_ITEM_AREA_START: usize = 487;
const BAD_RECORD_NEW_ROOT_SEGMENT_START: usize = 99;
/// 系统配置槽里「根环起点」那 8 字节的偏移（`layout/01-first-txn.md` 一）：core 里是私有常量，补丁之后由 `SystemConfiguration::parse_slot`
/// 读回 65 来核这个偏移（`bad_s1` 行 `p1_root_ring_base_reads_back`）。
const BAD_SYSTEM_CONFIGURATION_ROOT_RING_BASE_OFFSET: usize = 371;

/// 坏状态表里一行属于哪一族（登记 5.5）：P 行与盲测族期望「不在」、决定 Q3；N 行期望「在」、附带。
/// 第 1 次重跑多一族 P10（`research/prompts/e170-r1-prereg.md` 5.5）：考 A(c) 下界的一行，附带 Q3-g、不进 Q3-a，所以不与 P 行共用标签。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BadFamily {
    Positive,
    Negative,
    Blind,
    AcknowledgementLowerBound,
}

impl BadFamily {
    fn label(self) -> &'static str {
        match self {
            Self::Positive => "P",
            Self::Negative => "N",
            Self::Blind => "B",
            Self::AcknowledgementLowerBound => "P10",
        }
    }
}

/// 坏状态表的哪一版：第一次跑的表（`bad-states`，P8 期望「不在」、没有 P10），或第 1 次重跑的表（`bad-states-second-run`，P8 期望「在」、
/// 每条流末尾多一行 P10；`research/prompts/e170-r1-prereg.md` 5.3、5.5）。两版的补丁、底态、盲测族抽样逐行相同（第十一节 S5）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BadTableVersion {
    FirstRun,
    SecondRun,
}

impl BadTableVersion {
    fn mode_word(self) -> &'static str {
        match self {
            Self::FirstRun => "bad-states",
            Self::SecondRun => "bad-states-second-run",
        }
    }

    /// `config` 结果行里 `mode=` 写的名字（第一次的模式照旧写 `bad_states`）。
    fn config_mode_text(self) -> &'static str {
        match self {
            Self::FirstRun => "bad_states",
            Self::SecondRun => "bad_states_second_run",
        }
    }

    /// P8 的期望判定。第一次的登记写「不在」；第 1 次重跑的登记 5.5 修订按读法 3 改成「在」（落点 v_{n−1} 仍在 A(c) 里）。
    fn p8_expected_verdict(self) -> &'static str {
        match self {
            Self::FirstRun => "not_in",
            Self::SecondRun => "in",
        }
    }
}

/// 一个底态：名字（F、Mk、Mk+、Mk-、Mk@<调用序号>）、它对应的调用（F 没有）、按写序号的持久向量（只有持久与没持久两值）。
#[derive(Clone, Debug)]
struct BaseState {
    label: String,
    call: Option<usize>,
    persisted: Vec<bool>,
    notes: String,
}

/// 一处字节补丁：在底态镜像上把 (盘, 偏移) 起的这几个字节换掉。
#[derive(Clone, Debug)]
struct BytePatch {
    device: DeviceIdentity,
    offset: u64,
    bytes: Vec<u8>,
}

/// 期望落到哪里（登记 5.5「期望与依据」一列；Q3-d、Q3-e 的比对用）。
#[derive(Clone, Debug)]
enum ExpectedLanding {
    Nothing,
    Version(String),
    OutsideTheVersionTable,
}

impl ExpectedLanding {
    fn text(&self) -> String {
        match self {
            Self::Nothing => "none".to_string(),
            Self::Version(projection) => projection.clone(),
            Self::OutsideTheVersionTable => "outside_version_table".to_string(),
        }
    }
}

/// 坏状态表的一行。
#[derive(Clone, Debug)]
struct BadRow {
    name: String,
    family: BadFamily,
    base: BaseState,
    patches: Vec<BytePatch>,
    expected_verdict: &'static str,
    expected_landing: ExpectedLanding,
}

/// splitmix64（盲测族的抽样，登记 5.5「当 u64 喂 splitmix64」）：状态先加黄金比常数，再两轮异或移位乘。
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn next_value(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    /// [0, choices) 里均匀取一个：取模，偏差至多 choices / 2⁶⁴（choices 不过一百万，可以不计）。
    fn below(&mut self, choices: u64) -> u64 {
        assert!(choices > 0, "至少一个可选");
        self.next_value() % choices
    }
}

/// 一条流在坏状态这一段要的几样：每次写的流序下标、按发布分好的写、发过根槽写的调用（调用序号升序）。
struct BadStreamFacts<'stream> {
    stream: &'stream PreparedStream,
    operation_index_of_write: Vec<usize>,
    publishes: Vec<PublishWrites>,
    root_issuing_calls: Vec<usize>,
}

fn bad_stream_facts(stream: &PreparedStream) -> BadStreamFacts<'_> {
    let operation_index_of_write: Vec<usize> = stream
        .operations
        .iter()
        .enumerate()
        .filter(|(_, retained)| retained.operation.kind != RecordedOperationKind::Barrier)
        .map(|(operation_index, _)| operation_index)
        .collect();
    assert_eq!(
        operation_index_of_write.len(),
        stream.writes.len(),
        "写表与流里的非屏障操作一一对应（第一段 S1 的 write_order_is_the_non_barrier_operation_order）"
    );
    let publishes = publishes_of_one_recording(&stream.writes, 0);
    let root_issuing_calls = stream
        .calls
        .iter()
        .enumerate()
        .filter(|(_, call)| {
            publishes.iter().any(|publish| {
                (call.first_operation..call.return_point)
                    .contains(&operation_index_of_write[publish.root])
            })
        })
        .map(|(call_index, _)| call_index)
        .collect();
    BadStreamFacts {
        stream,
        operation_index_of_write,
        publishes,
        root_issuing_calls,
    }
}

/// 第 ⌈K/2⌉ 个（从 1 数）在下标里是 ⌈K/2⌉ − 1（登记 5.5 Mk「共 K 个时取第 ⌈K/2⌉ 个」，Mk⁻ 同一取法）。
fn middle_position(count: usize) -> usize {
    assert!(count > 0, "至少一个可取");
    count.div_ceil(2) - 1
}

/// 只认 core 写者写出的根记录（fsid 取本地常量，第一段 S1 回比过）。
fn parse_root_write(write: &RetainedWrite) -> RootRecord {
    RootRecord::parse_slot(
        write.bytes().expect("根槽写带字节"),
        &LOCAL_FILESYSTEM_IDENTIFIER,
    )
    .expect("录制流里的根槽写是 core 写者写的、自证得过")
}

fn parse_record_write(write: &RetainedWrite) -> JournalRecord {
    JournalRecord::parse(
        write.bytes().expect("journal 记录写带字节"),
        unit_filesystem_identifier(&LOCAL_FILESYSTEM_IDENTIFIER),
    )
    .expect("录制流里的记录写是 core 写者写的、自证得过")
}

/// 一版的投影写成执行器判定行里 `landed=` 的同一种写法 (实例, txg, wm=树 ID 水位, floor=F)，比对器按文本比（指针两段不进这串）。
fn projection_text_of_root(root: &RootRecord) -> String {
    format!(
        "({},{},wm={},floor={})",
        root.instance.0,
        root.checkpoint_txg.0,
        root.tree_identifier_watermark,
        root.rollback_floor.0
    )
}

impl BadStreamFacts<'_> {
    fn persisted_before_operation(&self, operation_bound: usize) -> Vec<bool> {
        self.operation_index_of_write
            .iter()
            .map(|operation_index| *operation_index < operation_bound)
            .collect()
    }

    fn all_persisted(&self) -> BaseState {
        BaseState {
            label: "F".to_string(),
            call: None,
            persisted: vec![true; self.stream.writes.len()],
            notes: "all_writes_persisted".to_string(),
        }
    }

    /// 调用边界：流序下标小于这次调用返回点的写全记持久、其余全没持久（登记 5.5 Mk 的定义；盲测族的 Mk 型底态同一取法）。
    fn call_boundary(&self, call_index: usize, label: String) -> BaseState {
        let call = &self.stream.calls[call_index];
        BaseState {
            label,
            call: Some(call_index),
            persisted: self.persisted_before_operation(call.return_point),
            notes: format!(
                "call={call_index}:{}:return_point={}",
                call.label, call.return_point
            ),
        }
    }

    fn mk(&self) -> BaseState {
        let call_index = self.root_issuing_calls[middle_position(self.root_issuing_calls.len())];
        self.call_boundary(call_index, "Mk".to_string())
    }

    /// 底态里最新的那一版：根槽写持久了的发布里下标最大的那一次。
    fn newest_publish(&self, persisted: &[bool]) -> usize {
        self.publishes
            .iter()
            .rposition(|publish| persisted[publish.root])
            .expect("底态里至少一条根槽写持久")
    }

    /// Mk⁺：Mk 再加它之后第一次发布的第一次单元写持久；那次发布与 Mk 的最后一版同实例、且有单元写，不满足就顺延到下一个发过根槽写的调用。
    /// 一个都不满足时交 None（照报，P9 不造）。
    fn mk_plus(&self) -> Option<BaseState> {
        let middle = middle_position(self.root_issuing_calls.len());
        for (shifted, call_index) in self.root_issuing_calls[middle..].iter().enumerate() {
            let boundary = self.call_boundary(*call_index, "Mk+".to_string());
            let newest = &self.publishes[self.newest_publish(&boundary.persisted)];
            let return_point = self.stream.calls[*call_index].return_point;
            let Some(next) = self
                .publishes
                .iter()
                .find(|publish| self.operation_index_of_write[publish.root] >= return_point)
            else {
                continue;
            };
            let Some(first_unit_write) = next.units.iter().min().copied() else {
                continue;
            };
            if next.instance != newest.instance || boundary.persisted[first_unit_write] {
                continue;
            }
            let mut persisted = boundary.persisted;
            persisted[first_unit_write] = true;
            return Some(BaseState {
                label: "Mk+".to_string(),
                call: Some(*call_index),
                persisted,
                notes: format!(
                    "{}:shifted={shifted}:next_publish=({},{}):first_unit_write={first_unit_write}",
                    boundary.notes, next.instance, next.checkpoint_txg
                ),
            });
        }
        None
    }

    /// 一次发布带不带点名单元：有单元写，且它的记录里至少一条点名项数非 0。
    fn publish_names_units(&self, publish: &PublishWrites) -> bool {
        !publish.units.is_empty()
            && publish.records.iter().any(|record_write| {
                !parse_record_write(&self.stream.writes[*record_write])
                    .named
                    .is_empty()
            })
    }

    /// Mk⁻：带点名单元的发布里取居中那一次 p，流序在 p 的根槽写之前的写全记持久、其余全没持久。交回 (底态, p 在发布表里的下标)。
    fn mk_minus(&self) -> (BaseState, usize) {
        let naming: Vec<usize> = (0..self.publishes.len())
            .filter(|publish_index| self.publish_names_units(&self.publishes[*publish_index]))
            .collect();
        let chosen = naming[middle_position(naming.len())];
        let publish = &self.publishes[chosen];
        let root_operation = self.operation_index_of_write[publish.root];
        let owning_call =
            self.stream.calls.iter().position(|call| {
                (call.first_operation..call.return_point).contains(&root_operation)
            });
        (
            BaseState {
                label: "Mk-".to_string(),
                call: owning_call,
                persisted: self.persisted_before_operation(root_operation),
                notes: format!(
                    "publish=({},{}):publishes_naming_units={}:root_operation={root_operation}",
                    publish.instance,
                    publish.checkpoint_txg,
                    naming.len()
                ),
            },
            chosen,
        )
    }

    /// 底态镜像上读一段字节（基镜像 + 持久了的写，按写表顺序叠）。
    fn read_base_image(
        &self,
        persisted: &[bool],
        device: DeviceIdentity,
        offset: u64,
        length: usize,
    ) -> Vec<u8> {
        let image = CrashImage {
            base: &self.stream.base,
            writes: &self.stream.writes,
            persisted: persisted.to_vec(),
        };
        PoolReader::read(&image, device, DeviceOffsetInBytes(offset), length)
            .expect("底态镜像在盘内的区间读得出")
    }

    /// 每块盘两槽系统配置里自证得过、世代号最大的那一槽（D22（单元原子性怎么合成） 已定项 16 的择槽；解析与自证走 core 的读者）。
    fn chosen_system_configurations(&self, persisted: &[bool]) -> Vec<ChosenSystemConfiguration> {
        let slot_bytes = usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES)
            .expect("4096 装得进 usize");
        self.stream
            .base
            .devices
            .keys()
            .map(|device| {
                let slot_zero = self.read_base_image(persisted, *device, 0, slot_bytes);
                let parsed_zero = SystemConfiguration::parse_slot(&slot_zero)
                    .expect("底态只有持久与没持久两值、系统配置槽是整槽写：槽 0 自证得过");
                let slot_one_offset =
                    u64::from(parsed_zero.immutable.sizes.fixed_structure_slot_spacing);
                let slot_one =
                    self.read_base_image(persisted, *device, slot_one_offset, slot_bytes);
                let mut chosen = ChosenSystemConfiguration {
                    device: *device,
                    offset: 0,
                    bytes: slot_zero,
                    parsed: parsed_zero,
                };
                if let Ok(parsed_one) = SystemConfiguration::parse_slot(&slot_one) {
                    if parsed_one.quantities.slot_generation
                        > chosen.parsed.quantities.slot_generation
                    {
                        chosen = ChosenSystemConfiguration {
                            device: *device,
                            offset: slot_one_offset,
                            bytes: slot_one,
                            parsed: parsed_one,
                        };
                    }
                }
                chosen
            })
            .collect()
    }

    /// 底态里最新那一版 v_n：它的发布下标、根槽写、根记录、带末条标志的那条记录（记录 n）。
    fn newest_version(&self, persisted: &[bool]) -> NewestVersion {
        let publish_index = self.newest_publish(persisted);
        let publish = &self.publishes[publish_index];
        let record_write = publish
            .records
            .iter()
            .copied()
            .find(|record_write| {
                parse_record_write(&self.stream.writes[*record_write]).place_in_publish
                    == JournalRecordPlaceInPublish::LastRecordOfThePublish
            })
            .expect("每次发布都有一条带末条标志的记录（D23 已定项 17）");
        NewestVersion {
            publish_index,
            root_write: publish.root,
            root: parse_root_write(&self.stream.writes[publish.root]),
            record_write,
            record: parse_record_write(&self.stream.writes[record_write]),
        }
    }

    /// 发布表里第 `publish_index` 次发布之前那一版的投影文本；它是第一次发布时，前一版是基镜像上的现行那一版（mkfs 的第 0 代根，
    /// 按 txg 0 在根环里该落的位置读）。
    fn previous_version_text(&self, publish_index: usize) -> String {
        if let Some(previous) = publish_index.checked_sub(1) {
            return projection_text_of_root(&parse_root_write(
                &self.stream.writes[self.publishes[previous].root],
            ));
        }
        let nothing_persisted = vec![false; self.stream.writes.len()];
        let configuration = &self.chosen_system_configurations(&nothing_persisted)[0].parsed;
        let target = target_for_publish(
            CheckpointTxg(0),
            configuration.immutable.sizes.root_ring_slots_per_region,
        );
        let device = configuration.immutable.region_devices
            [usize::try_from(target.region).expect("区域号装得进 usize")];
        let offset = slot_offset(
            target,
            configuration.immutable.sizes.fixed_structure_slot_spacing,
        )
        .0;
        let width =
            usize::try_from(configuration.immutable.sizes.physical_block_size).expect("根槽宽");
        let bytes = self.read_base_image(&nothing_persisted, device, offset, width);
        projection_text_of_root(
            &RootRecord::parse_slot(&bytes, &LOCAL_FILESYSTEM_IDENTIFIER)
                .expect("基镜像上的第 0 代根自证得过"),
        )
    }
}

/// 一块盘上择到的系统配置槽：盘、槽在盘内的偏移、整槽字节、core 读者解出来的内容。
struct ChosenSystemConfiguration {
    device: DeviceIdentity,
    offset: u64,
    bytes: Vec<u8>,
    parsed: SystemConfiguration,
}

/// 底态里最新的那一版 v_n（登记 5.5 记号）。
struct NewestVersion {
    publish_index: usize,
    root_write: usize,
    root: RootRecord,
    record_write: usize,
    record: JournalRecord,
}

/// 一条伪记录的写法（登记 5.5「伪记录的头照记录 n 抄，改下面写明的字段」：点名项数 0、事务号 0、提交标记 1，反向链取逻辑前一条的头，
/// 载荷校验和与头校验和由 core 的写者重算，两块盘各写一份，落在计数器决定的环槽）。
struct PseudoRecord {
    counter: u64,
    checkpoint_txg: u64,
    ordinal: u32,
    is_last_of_the_publish: bool,
    watermark_step: u64,
}

impl BadStreamFacts<'_> {
    /// P1：两块盘上各自择到的那一槽，根环起点 64 → 65，照 core 的算法重封整槽校验和。另交回「补丁之后 core 读者读回 65、补丁之前读回 64」。
    fn root_ring_base_patches(&self, persisted: &[bool]) -> (Vec<BytePatch>, bool) {
        let mut reads_back = true;
        let patches = self
            .chosen_system_configurations(persisted)
            .into_iter()
            .map(|chosen| {
                reads_back &= chosen.parsed.immutable.root_ring_base_slot.0
                    == BAD_ROOT_RING_BASE_SLOT_FIRST_VERSION;
                let mut bytes = chosen.bytes;
                bytes[BAD_SYSTEM_CONFIGURATION_ROOT_RING_BASE_OFFSET
                    ..BAD_SYSTEM_CONFIGURATION_ROOT_RING_BASE_OFFSET + 8]
                    .copy_from_slice(&BAD_ROOT_RING_BASE_SLOT_PATCHED.to_le_bytes());
                let slot_bytes = bytes.len();
                let digest = wide_checksum_with_field_zeroed(
                    &bytes,
                    slot_bytes,
                    SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
                );
                bytes[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET
                    ..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + digest.len()]
                    .copy_from_slice(&digest);
                reads_back &= SystemConfiguration::parse_slot(&bytes).is_ok_and(|patched| {
                    patched.immutable.root_ring_base_slot.0 == BAD_ROOT_RING_BASE_SLOT_PATCHED
                });
                BytePatch {
                    device: chosen.device,
                    offset: chosen.offset,
                    bytes,
                }
            })
            .collect();
        (patches, reads_back)
    }

    /// P2 型：根 n 的 457 字节、txg 改成给定值、重封，落在区域公式给这个 txg 的那一槽（区域 txg mod R、槽 (txg div R) mod S，
    /// 盘按逐区域设备身份；几何取底态择到的系统配置，位置由 core 的 `target_for_publish`、`slot_offset` 算）。根 n 不动。
    fn pseudo_root_patch(
        &self,
        persisted: &[bool],
        newest: &NewestVersion,
        checkpoint_txg: u64,
    ) -> BytePatch {
        let configuration = &self.chosen_system_configurations(persisted)[0].parsed;
        let target = target_for_publish(
            CheckpointTxg(checkpoint_txg),
            configuration.immutable.sizes.root_ring_slots_per_region,
        );
        let slot_width = usize::try_from(self.stream.writes[newest.root_write].length_in_bytes())
            .expect("根槽宽装得进 usize");
        let mut root = newest.root;
        root.checkpoint_txg = CheckpointTxg(checkpoint_txg);
        BytePatch {
            device: configuration.immutable.region_devices
                [usize::try_from(target.region).expect("区域号装得进 usize")],
            offset: slot_offset(
                target,
                configuration.immutable.sizes.fixed_structure_slot_spacing,
            )
            .0,
            bytes: root.to_slot(slot_width),
        }
    }

    /// 根 n 原位改字段、重封（P5、N1）。
    fn root_in_place_patch(
        &self,
        newest: &NewestVersion,
        change: impl FnOnce(&mut RootRecord),
    ) -> BytePatch {
        let write = &self.stream.writes[newest.root_write];
        let mut root = newest.root;
        change(&mut root);
        BytePatch {
            device: write.device,
            offset: write.offset.0,
            bytes: root
                .to_slot(usize::try_from(write.length_in_bytes()).expect("根槽宽装得进 usize")),
        }
    }

    /// 不重封：底态镜像上那一个字节翻一位（P8、N4、N5）。
    fn flipped_byte_patch(
        &self,
        persisted: &[bool],
        device: DeviceIdentity,
        offset: u64,
    ) -> BytePatch {
        // 读者只收按扇区对齐的读（`check_request_against_the_device_contract`）：读整个扇区，取那一个字节。
        let sector_start = offset - offset % LOCAL_SECTOR_BYTES;
        let sector = self.read_base_image(
            persisted,
            device,
            sector_start,
            usize::try_from(LOCAL_SECTOR_BYTES).expect("512 装得进 usize"),
        );
        let original = sector[usize::try_from(offset - sector_start).expect("扇区内偏移")];
        BytePatch {
            device,
            offset,
            bytes: vec![original ^ BAD_FLIPPED_BIT],
        }
    }

    /// 伪记录（P3、P4、P9、N2、N3 与盲测 P3 型）：头照记录 n 抄，逐条改写法里的字段；第二条起反向链取前一条伪记录的头。
    fn pseudo_record_patches(
        &self,
        persisted: &[bool],
        newest: &NewestVersion,
        pseudo_records: &[PseudoRecord],
    ) -> Vec<BytePatch> {
        let ring_bytes = self.chosen_system_configurations(persisted)[0]
            .parsed
            .immutable
            .sizes
            .journal_ring_bytes;
        let template = &newest.record;
        let mut previous_bytes = self.stream.writes[newest.record_write]
            .bytes()
            .expect("记录写带字节")
            .to_vec();
        let mut patches = Vec::new();
        for pseudo in pseudo_records {
            let record = JournalRecord {
                instance: template.instance,
                counter: pseudo.counter,
                checkpoint_txg: CheckpointTxg(pseudo.checkpoint_txg),
                transaction: 0,
                is_commit: true,
                ordinal_within_publish: JournalRecordOrdinalWithinPublish(pseudo.ordinal),
                place_in_publish: if pseudo.is_last_of_the_publish {
                    JournalRecordPlaceInPublish::LastRecordOfThePublish
                } else {
                    JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
                },
                back_chain: back_chain_of(&previous_bytes),
                filesystem_identifier: template.filesystem_identifier,
                new_tree_table: template.new_tree_table,
                new_mapping_root: template.new_mapping_root,
                new_tree_identifier_watermark: template.new_tree_identifier_watermark
                    + pseudo.watermark_step,
                new_rollback_floor: template.new_rollback_floor,
                new_instance_table: template.new_instance_table,
                new_allocation_record_tree_root: template.new_allocation_record_tree_root,
                new_unmount_marker: template.new_unmount_marker,
                named: Vec::new(),
            };
            let bytes = record.to_bytes();
            let offset = record_offset(pseudo.counter, ring_bytes).0;
            for device in self.stream.base.devices.keys() {
                patches.push(BytePatch {
                    device: *device,
                    offset,
                    bytes: bytes.clone(),
                });
            }
            previous_bytes = bytes;
        }
        patches
    }
}

/// 一行坏状态的改法（登记 5.5 的 P 表、N 表与盲测族三型）。
#[derive(Clone, Copy, Debug)]
enum BadShape {
    /// P1：两块盘上择到的系统配置槽，根环起点 64 → 65，重封。
    RootRingBaseMisdirected,
    /// P2（盲测 P2 型 txg_step = 3j）：伪根 txg n + txg_step。
    PseudoRootAhead { txg_step: u64 },
    /// P3（盲测 P3 型）、P9：伪续接一条，水位 + watermark_step。
    PseudoContinuation { watermark_step: u64 },
    /// P4：伪续接两条，末条标志 0、1。
    PseudoContinuationOfTwo,
    /// P5（盲测 P5 型）：根 n 的实例代号 i → i − 1，重封。
    RootInstanceLowered,
    /// P8、P10：根 n 偏移 122 翻一位、记录 n 两份各翻一位，都不重封。补丁与落点两处同一个写法；期望判定由调用处给：
    /// 第一次的表 P8 写「不在」，第 1 次重跑的表 P8 写「在」、P10（Mk⁺ 底态）写「不在」。
    NewestRootAndRecordUnreadable { expected_verdict: &'static str },
    /// N1：根 n 的 txg n → n − 2，重封，原位。
    RootTxgLowered,
    /// N2：同 P3，计数器 c_n + 2。
    PseudoRecordAfterAGap,
    /// N3：同 P4，末条标志 0、0。
    PseudoPairWithoutLastFlag,
    /// N4：根 n 偏移 122 翻一位，不重封。
    NewestRootUnverified,
    /// N5：p 的第一次单元写在盘身份较小那块盘上的那一份，单元内偏移 200 翻一位。
    NamedUnitCopyCorrupted { publish_index: usize },
}

impl BadStreamFacts<'_> {
    /// 版本表里出现过的 (实例代号, txg)：基镜像的第 0 代 (0, 0) 加每一次根槽写（P5 的前提断言用）。
    fn version_identities(&self) -> BTreeSet<(u32, u64)> {
        std::iter::once((0, 0))
            .chain(
                self.publishes
                    .iter()
                    .map(|publish| (publish.instance, publish.checkpoint_txg)),
            )
            .collect()
    }

    /// 记录 n 在两块盘上的两份（写表下标）。
    fn record_copies_of(&self, newest: &NewestVersion) -> Vec<usize> {
        self.publishes[newest.publish_index]
            .records
            .iter()
            .copied()
            .filter(|record_write| {
                parse_record_write(&self.stream.writes[*record_write]).counter
                    == newest.record.counter
            })
            .collect()
    }
}

fn pseudo_continuation(
    newest: &NewestVersion,
    counter_step: u64,
    watermark_step: u64,
) -> PseudoRecord {
    PseudoRecord {
        counter: newest.record.counter + counter_step,
        checkpoint_txg: newest.root.checkpoint_txg.0 + 1,
        ordinal: 1,
        is_last_of_the_publish: true,
        watermark_step,
    }
}

fn pseudo_pair(newest: &NewestVersion, second_is_last: bool) -> [PseudoRecord; 2] {
    [
        PseudoRecord {
            is_last_of_the_publish: false,
            ..pseudo_continuation(newest, 1, BAD_WATERMARK_STEP)
        },
        PseudoRecord {
            counter: newest.record.counter + 2,
            ordinal: 2,
            is_last_of_the_publish: second_is_last,
            ..pseudo_continuation(newest, 1, BAD_WATERMARK_STEP)
        },
    ]
}

/// 照改法造一行：补丁、期望的在 / 不在、期望落到哪里。P1 的读回与 P5 的前提另记进 `checks`（照打，不停）。
fn build_bad_row(
    facts: &BadStreamFacts<'_>,
    name: String,
    family: BadFamily,
    base: BaseState,
    shape: BadShape,
    checks: &mut Vec<StopCheck>,
) -> BadRow {
    let persisted = &base.persisted;
    let newest = facts.newest_version(persisted);
    let newest_txg = newest.root.checkpoint_txg.0;
    let newest_text = projection_text_of_root(&newest.root);
    let root_write = &facts.stream.writes[newest.root_write];
    let root_watermark_byte = root_write.offset.0
        + u64::try_from(BAD_ROOT_TREE_IDENTIFIER_WATERMARK_FIRST_BYTE).expect("122");
    let (patches, expected_verdict, expected_landing) = match shape {
        BadShape::RootRingBaseMisdirected => {
            let (patches, reads_back) = facts.root_ring_base_patches(persisted);
            checks.push(StopCheck::new(
                &format!("{name}_root_ring_base_reads_back"),
                true,
                reads_back,
            ));
            (patches, "not_in", ExpectedLanding::Nothing)
        }
        BadShape::PseudoRootAhead { txg_step } => (
            vec![facts.pseudo_root_patch(persisted, &newest, newest_txg + txg_step)],
            "not_in",
            ExpectedLanding::OutsideTheVersionTable,
        ),
        BadShape::PseudoContinuation { watermark_step } => (
            facts.pseudo_record_patches(
                persisted,
                &newest,
                &[pseudo_continuation(&newest, 1, watermark_step)],
            ),
            "not_in",
            ExpectedLanding::OutsideTheVersionTable,
        ),
        BadShape::PseudoContinuationOfTwo => (
            facts.pseudo_record_patches(persisted, &newest, &pseudo_pair(&newest, true)),
            "not_in",
            ExpectedLanding::OutsideTheVersionTable,
        ),
        BadShape::RootInstanceLowered => {
            let lowered = newest
                .root
                .instance
                .0
                .checked_sub(1)
                .expect("v_n 的实例代号 ≥ 1");
            let absent = !facts.version_identities().contains(&(lowered, newest_txg));
            checks.push(StopCheck::new(
                &format!("{name}_version_table_lacks_lowered_instance"),
                true,
                absent,
            ));
            (
                vec![facts.root_in_place_patch(&newest, |root| {
                    root.instance = InstanceGeneration(lowered)
                })],
                "not_in",
                ExpectedLanding::OutsideTheVersionTable,
            )
        }
        BadShape::NewestRootAndRecordUnreadable { expected_verdict } => {
            let area = if newest.record.named.is_empty() {
                BAD_RECORD_NEW_ROOT_SEGMENT_START
            } else {
                BAD_RECORD_NAMED_ITEM_AREA_START
            };
            let mut patches =
                vec![facts.flipped_byte_patch(persisted, root_write.device, root_watermark_byte)];
            for copy in facts.record_copies_of(&newest) {
                let write = &facts.stream.writes[copy];
                let offset = write.offset.0 + u64::try_from(area).expect("记录内偏移");
                patches.push(facts.flipped_byte_patch(persisted, write.device, offset));
            }
            (
                patches,
                expected_verdict,
                ExpectedLanding::Version(facts.previous_version_text(newest.publish_index)),
            )
        }
        BadShape::RootTxgLowered => (
            vec![facts.root_in_place_patch(&newest, |root| {
                root.checkpoint_txg = CheckpointTxg(newest_txg - BAD_LOWERED_TXG_STEP);
            })],
            "in",
            ExpectedLanding::Version(newest_text),
        ),
        BadShape::PseudoRecordAfterAGap => (
            facts.pseudo_record_patches(
                persisted,
                &newest,
                &[pseudo_continuation(&newest, 2, BAD_WATERMARK_STEP)],
            ),
            "in",
            ExpectedLanding::Version(newest_text),
        ),
        BadShape::PseudoPairWithoutLastFlag => (
            facts.pseudo_record_patches(persisted, &newest, &pseudo_pair(&newest, false)),
            "in",
            ExpectedLanding::Version(newest_text),
        ),
        BadShape::NewestRootUnverified => (
            vec![facts.flipped_byte_patch(persisted, root_write.device, root_watermark_byte)],
            "in",
            ExpectedLanding::Version(newest_text),
        ),
        BadShape::NamedUnitCopyCorrupted { publish_index } => {
            let publish = &facts.publishes[publish_index];
            let writes = &facts.stream.writes;
            let first_unit = publish.units.iter().min().copied().expect("p 有单元写");
            let copy = publish
                .units
                .iter()
                .copied()
                .filter(|unit| writes[*unit].bytes() == writes[first_unit].bytes())
                .min_by_key(|unit| (writes[*unit].device, *unit))
                .expect("第一次单元写自己就在里面");
            let offset = writes[copy].offset.0 + u64::try_from(BAD_UNIT_FLIP_OFFSET).expect("200");
            (
                vec![facts.flipped_byte_patch(persisted, writes[copy].device, offset)],
                "in",
                ExpectedLanding::Version(facts.previous_version_text(publish_index)),
            )
        }
    };
    BadRow {
        name,
        family,
        base,
        patches,
        expected_verdict,
        expected_landing,
    }
}

/// 一条流的坏状态表：P 行（F、Mk 各 6 行，Mk⁺ 上 P9）、N 行（F 上 N1–N4，Mk⁻ 上 N5）、盲测族 20 行。交回 (底态, 行)。
/// 盲测族每行依次抽：底态（F 或任一个发过根槽写的调用边界，均匀）、形态（P2 型、P3 型、P5 型，均匀）、形态的参数（P2 型 j ∈ 1..7，
/// P3 型 d ∈ [1000, 1 000 000]）；两条流共用一串 splitmix64，流一的 20 行先抽。
/// 第 1 次重跑的表（`BadTableVersion::SecondRun`）与第一次的表只差两处：P8 的期望判定，和盲测族之后多一行 P10（Mk⁺ 底态上同 P8 的改法、
/// 期望「不在」；这条流没有 Mk⁺ 就没有这一行）。P10 排在最后、不抽随机数，所以别的行的行号、补丁与盲测族抽样都不变（第十一节 S5）。
fn bad_rows_of_stream(
    facts: &BadStreamFacts<'_>,
    generator: &mut SplitMix64,
    table_version: BadTableVersion,
    checks: &mut Vec<StopCheck>,
) -> (Vec<BaseState>, Vec<BadRow>) {
    let full = facts.all_persisted();
    let middle = facts.mk();
    let middle_plus = facts.mk_plus();
    let (middle_minus, named_publish) = facts.mk_minus();
    let mut rows = Vec::new();
    for base in [&full, &middle] {
        for (label, shape) in [
            ("P1", BadShape::RootRingBaseMisdirected),
            (
                "P2",
                BadShape::PseudoRootAhead {
                    txg_step: BAD_PSEUDO_ROOT_TXG_STEP,
                },
            ),
            (
                "P3",
                BadShape::PseudoContinuation {
                    watermark_step: BAD_WATERMARK_STEP,
                },
            ),
            ("P4", BadShape::PseudoContinuationOfTwo),
            ("P5", BadShape::RootInstanceLowered),
            (
                "P8",
                BadShape::NewestRootAndRecordUnreadable {
                    expected_verdict: table_version.p8_expected_verdict(),
                },
            ),
        ] {
            let name = format!("{label}-{}", base.label);
            rows.push(build_bad_row(
                facts,
                name,
                BadFamily::Positive,
                base.clone(),
                shape,
                checks,
            ));
        }
    }
    if let Some(plus) = &middle_plus {
        let shape = BadShape::PseudoContinuation {
            watermark_step: BAD_WATERMARK_STEP,
        };
        rows.push(build_bad_row(
            facts,
            "P9-Mk+".to_string(),
            BadFamily::Positive,
            plus.clone(),
            shape,
            checks,
        ));
    }
    for (label, shape) in [
        ("N1", BadShape::RootTxgLowered),
        ("N2", BadShape::PseudoRecordAfterAGap),
        ("N3", BadShape::PseudoPairWithoutLastFlag),
        ("N4", BadShape::NewestRootUnverified),
    ] {
        rows.push(build_bad_row(
            facts,
            format!("{label}-F"),
            BadFamily::Negative,
            full.clone(),
            shape,
            checks,
        ));
    }
    let named_unit_shape = BadShape::NamedUnitCopyCorrupted {
        publish_index: named_publish,
    };
    rows.push(build_bad_row(
        facts,
        "N5-Mk-".to_string(),
        BadFamily::Negative,
        middle_minus.clone(),
        named_unit_shape,
        checks,
    ));
    let base_choices = u64::try_from(facts.root_issuing_calls.len() + 1).expect("调用数装得进 u64");
    for row_number in 1..=BLIND_ROWS_PER_STREAM {
        let base_choice = generator.below(base_choices);
        let base = match base_choice.checked_sub(1) {
            None => full.clone(),
            Some(call_position) => {
                let call_index =
                    facts.root_issuing_calls[usize::try_from(call_position).expect("调用下标")];
                facts.call_boundary(call_index, format!("Mk@{call_index}"))
            }
        };
        let (shape_label, shape) = match generator.below(3) {
            0 => {
                let step = 1 + generator.below(BLIND_PSEUDO_ROOT_STEPS);
                let txg_step = BAD_PSEUDO_ROOT_TXG_STEP * step;
                (
                    format!("P2:j={step}"),
                    BadShape::PseudoRootAhead { txg_step },
                )
            }
            1 => {
                let watermark_step = BLIND_LOWEST_WATERMARK_STEP
                    + generator
                        .below(BLIND_HIGHEST_WATERMARK_STEP - BLIND_LOWEST_WATERMARK_STEP + 1);
                (
                    format!("P3:d={watermark_step}"),
                    BadShape::PseudoContinuation { watermark_step },
                )
            }
            _ => ("P5".to_string(), BadShape::RootInstanceLowered),
        };
        let name = format!("B{row_number:02}:{shape_label}:{}", base.label);
        rows.push(build_bad_row(
            facts,
            name,
            BadFamily::Blind,
            base,
            shape,
            checks,
        ));
    }
    let lower_bound_base = match table_version {
        BadTableVersion::FirstRun => None,
        BadTableVersion::SecondRun => middle_plus.clone(),
    };
    if let Some(lower_bound_base) = lower_bound_base {
        rows.push(build_bad_row(
            facts,
            LOWER_BOUND_ROW_NAME.to_string(),
            BadFamily::AcknowledgementLowerBound,
            lower_bound_base,
            BadShape::NewestRootAndRecordUnreadable {
                expected_verdict: "not_in",
            },
            checks,
        ));
    }
    let mut bases = vec![full, middle, middle_minus];
    bases.extend(middle_plus);
    (bases, rows)
}

/// 底态的落地向量（只有 `2` 与 `0`）在层 0 序号里找同一个：只找每段序号区间的两端与 0、T − 1（调用边界落在段的边界上时，同一个持久集合
/// 就是前一段区间的末端或后一段区间的起点；登记 5.5「找不到照实报，仍用它」）。交回 (找到的序号, 找过的候选数)。
fn layer0_ordinal_with_the_same_landing(
    recorded_write_count: usize,
    tables: &Layer0EnumerationTables,
    flow_facts: &FlowFacts,
    torn_image_index_by_write: &BTreeMap<usize, usize>,
    persisted: &[bool],
) -> (Option<u64>, usize) {
    let wanted: String = persisted
        .iter()
        .map(|is_persisted| if *is_persisted { '2' } else { '0' })
        .collect();
    let mut candidates: BTreeSet<u64> = BTreeSet::from([0, tables.state_count - 1]);
    for state_range in &tables.state_ranges_by_segment {
        if !state_range.is_empty() {
            candidates.insert(state_range.start);
            candidates.insert(state_range.end - 1);
        }
    }
    let found = candidates.iter().copied().find(|ordinal| {
        landing_vector_of(
            &flow_facts.persisted_writes_of_state(*ordinal),
            recorded_write_count,
            torn_image_index_by_write,
        ) == wanted
    });
    (found, candidates.len())
}

/// 一行坏状态上实现那一侧看到的：底态与补丁之后两个镜像摘要（同第一段的摘要算法）、实现判定（判定里 panic 的照打）。
struct BadObservation {
    base_digest: String,
    patched_digest: String,
    verdict: Result<ImplementationVerdict, String>,
}

fn observe_bad_row(stream: &PreparedStream, row: &BadRow) -> BadObservation {
    let base_image = CrashImage {
        base: &stream.base,
        writes: &stream.writes,
        persisted: row.base.persisted.clone(),
    };
    let base_digest = sha256_text(&crash_image_digest(&base_image));
    let mut writes = stream.writes.clone();
    writes.extend(row.patches.iter().map(|patch| RetainedWrite {
        device: patch.device,
        kind: StepKind::SystemConfigurationSlot,
        is_force_unit_access: false,
        offset: DeviceOffsetInBytes(patch.offset),
        contents: WrittenContents::Bytes(patch.bytes.clone()),
    }));
    let mut persisted = row.base.persisted.clone();
    persisted.resize(writes.len(), true);
    let image = CrashImage {
        base: &stream.base,
        writes: &writes,
        persisted,
    };
    let patched_digest = sha256_text(&crash_image_digest(&image));
    let verdict = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        judge_crash_image(&image, stream.judged_root_index, &stream.versions)
    }))
    .map(|judgement| ImplementationVerdict::of(&judgement))
    .map_err(|payload| {
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| {
                payload
                    .downcast_ref::<&str>()
                    .map(|text| (*text).to_string())
            })
            .unwrap_or_else(|| "non_text_panic".to_string())
    });
    BadObservation {
        base_digest,
        patched_digest,
        verdict,
    }
}

const BAD_STATES_FILE: &str = "bad-states.tsv";
const BAD_PATCHES_FILE: &str = "bad-patches.bin";
const BAD_EXPECTED_FILE: &str = "bad-expected.tsv";
const BAD_IMPLEMENTATION_VERDICTS_FILE: &str = "bad-impl-verdicts.tsv";
const BAD_SEED_FILE: &str = "bad-seed.tsv";
const BAD_FILES: [&str; 5] = [
    BAD_STATES_FILE,
    BAD_PATCHES_FILE,
    BAD_EXPECTED_FILE,
    BAD_IMPLEMENTATION_VERDICTS_FILE,
    BAD_SEED_FILE,
];

/// 重驱动的流与导出目录里那份逐字节相同：manifest 记的四个文件的 sha256 与重驱动现算的比（S1 同形，记进 `bad_s1`）。
fn dump_agreement_checks(stream: &PreparedStream, directory: &Path) -> Vec<StopCheck> {
    let manifest: BTreeMap<String, String> = read_text_lines(directory, MANIFEST_FILE)
        .iter()
        .filter_map(|line| line.split_once('\t'))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
    let (operations_text, payload) = operations_file_text_and_payload(stream);
    let label = stream.name.directory_name();
    [
        (BASE_SECTORS_FILE, base_sectors_file_bytes(stream)),
        (OPERATIONS_FILE, operations_text.into_bytes()),
        (OPERATIONS_PAYLOAD_FILE, payload),
        (CALLS_FILE, calls_file_text(stream).into_bytes()),
    ]
    .into_iter()
    .map(|(file_name, contents)| {
        StopCheck::new(
            &format!("{label}_redriven_{file_name}_matches_the_dump"),
            manifest
                .get(&format!("sha256:{file_name}"))
                .cloned()
                .unwrap_or_default(),
            sha256_hexadecimal(&contents),
        )
    })
    .collect()
}

/// 本地常量与 core 的回比（`bad_s1`）：点名项区起点、新根段起点、根记录树 ID 水位第一个字节、根 n 所在槽与区域公式给的槽。
fn bad_local_constant_checks(facts: &BadStreamFacts<'_>, full: &BaseState) -> Vec<StopCheck> {
    let label = facts.stream.name.directory_name();
    let newest = facts.newest_version(&full.persisted);
    let mut raised = newest.root;
    raised.tree_identifier_watermark += 1;
    let width =
        usize::try_from(facts.stream.writes[newest.root_write].length_in_bytes()).expect("根槽宽");
    let original_slot = newest.root.to_slot(width);
    let raised_slot = raised.to_slot(width);
    let first_differing_byte = original_slot
        .iter()
        .zip(&raised_slot)
        .position(|(original, raised_byte)| original != raised_byte);
    let configuration = &facts.chosen_system_configurations(&full.persisted)[0].parsed;
    let target = target_for_publish(
        newest.root.checkpoint_txg,
        configuration.immutable.sizes.root_ring_slots_per_region,
    );
    let root_write = &facts.stream.writes[newest.root_write];
    vec![
        StopCheck::new(
            &format!("{label}_record_named_item_area_start"),
            BAD_RECORD_NAMED_ITEM_AREA_START,
            usize::try_from(singlefs_format::JOURNAL_HEADER_BYTES)
                .expect("记录头 487 字节装得进 usize"),
        ),
        StopCheck::new(
            &format!("{label}_record_new_root_segment_start"),
            BAD_RECORD_NEW_ROOT_SEGMENT_START,
            JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET + 4 + 4 + 4,
        ),
        StopCheck::new(
            &format!("{label}_root_tree_identifier_watermark_first_byte"),
            Some(BAD_ROOT_TREE_IDENTIFIER_WATERMARK_FIRST_BYTE),
            first_differing_byte,
        ),
        StopCheck::new(
            &format!("{label}_root_n_reencodes_to_the_written_bytes"),
            true,
            root_write.bytes() == Some(original_slot.as_slice()),
        ),
        StopCheck::new(
            &format!("{label}_root_n_sits_where_the_region_formula_puts_it"),
            (root_write.device, root_write.offset.0),
            (
                configuration.immutable.region_devices
                    [usize::try_from(target.region).expect("区域号")],
                slot_offset(
                    target,
                    configuration.immutable.sizes.fixed_structure_slot_spacing,
                )
                .0,
            ),
        ),
    ]
}

/// 底态的落地向量文本：持久的写记 `2`、没持久的记 `0`（与 `states.tsv` 同一种写法）。
fn landing_vector_text(persisted: &[bool]) -> String {
    persisted
        .iter()
        .map(|is_persisted| if *is_persisted { '2' } else { '0' })
        .collect()
}

/// S5b（`research/prompts/e170-r1-prereg.md` 第十一节）：P10 那一行的底态落地向量与底态镜像摘要，与同一条流 P9-Mk+ 那一行的相同；
/// P10 行有没有，与 P9-Mk+ 行有没有一致（两行都由 Mk⁺ 给得出才造）。底态取错（例如取了 Mk）时落地向量与摘要都会不同。
fn lower_bound_row_base_checks(
    stream_label: &str,
    rows: &[BadRow],
    observations: &[BadObservation],
) -> Vec<StopCheck> {
    let position_of = |row_name: &str| rows.iter().position(|row| row.name == row_name);
    let lower_bound = position_of(LOWER_BOUND_ROW_NAME);
    let mk_plus_p9 = position_of(FIRST_RUN_MK_PLUS_ROW_NAME);
    let mut checks = vec![StopCheck::new(
        &format!("{stream_label}_p10_row_exists_exactly_when_p9_mk_plus_row_exists"),
        mk_plus_p9.is_some(),
        lower_bound.is_some(),
    )];
    if let (Some(lower_bound), Some(mk_plus_p9)) = (lower_bound, mk_plus_p9) {
        checks.push(StopCheck::new(
            &format!("{stream_label}_p10_base_landing_vector_equals_p9_mk_plus"),
            landing_vector_text(&rows[mk_plus_p9].base.persisted),
            landing_vector_text(&rows[lower_bound].base.persisted),
        ));
        checks.push(StopCheck::new(
            &format!("{stream_label}_p10_base_digest_equals_p9_mk_plus"),
            &observations[mk_plus_p9].base_digest,
            &observations[lower_bound].base_digest,
        ));
    }
    checks
}

/// 一条流造完、判完的坏状态：底态（带层 0 序号对照）、行、实现那一侧的观测、回比。
struct BadStreamExport {
    name: StreamName,
    bases: Vec<(BaseState, Option<u64>, usize)>,
    rows: Vec<BadRow>,
    observations: Vec<BadObservation>,
    checks: Vec<StopCheck>,
    files: Vec<WrittenFile>,
}

fn implementation_verdict_line(row_id: usize, observation: &BadObservation) -> String {
    match &observation.verdict {
        Ok(verdict) => format!(
            "{row_id}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            verdict.consulted_root,
            verdict.consulted_outcome,
            verdict.ignored_root,
            verdict.ignored_outcome,
            field_text(&list_text(&verdict.red_items)),
            list_text(&verdict.pool_checker_red),
            verdict.root_without_record,
            verdict.claimed_state_missing_unit,
            verdict.pool_checker_and_record_checker_both_pass()
        ),
        Err(panic) => format!(
            "{row_id}\tpanicked:{}\t-\t-\t-\t-\t-\t-\t-\tfalse\n",
            field_text(panic)
        ),
    }
}

/// 一条流：重驱动、核导出、造表、判、写五个文件（写之前 `run_bad_states` 已核过五个文件都不存在）。
fn bad_states_of_stream(
    name: StreamName,
    dump_root: &Path,
    seed_hex: &str,
    seed: u64,
    generator: &mut SplitMix64,
    table_version: BadTableVersion,
) -> BadStreamExport {
    let stream = prepare_stream(name);
    let directory = dump_root.join(name.directory_name());
    let mut checks = stream.checks.clone();
    checks.extend(dump_agreement_checks(&stream, &directory));
    let tables = layer0_enumeration_tables(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let flow_facts = extract_flow_facts(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let torn_image_index_by_write: BTreeMap<usize, usize> = tables
        .torn_images
        .iter()
        .map(|row| (row.write_index, row.torn_image_index))
        .collect();
    let facts = bad_stream_facts(&stream);
    checks.extend(bad_local_constant_checks(&facts, &facts.all_persisted()));
    let (bases, rows) = bad_rows_of_stream(&facts, generator, table_version, &mut checks);
    let mut matched_bases: Vec<(BaseState, Option<u64>, usize)> = Vec::new();
    for base in bases.iter().chain(rows.iter().map(|row| &row.base)) {
        if matched_bases
            .iter()
            .any(|(seen, _, _)| seen.label == base.label)
        {
            continue;
        }
        let (found, candidates) = layer0_ordinal_with_the_same_landing(
            stream.writes.len(),
            &tables,
            &flow_facts,
            &torn_image_index_by_write,
            &base.persisted,
        );
        matched_bases.push((base.clone(), found, candidates));
    }
    let observations: Vec<BadObservation> = rows
        .iter()
        .map(|row| observe_bad_row(&stream, row))
        .collect();
    if table_version == BadTableVersion::SecondRun {
        checks.extend(lower_bound_row_base_checks(
            name.directory_name(),
            &rows,
            &observations,
        ));
    }
    let mut states_text = String::new();
    let mut patches_bytes = Vec::new();
    let mut expected_text = String::new();
    let mut verdicts_text = String::new();
    for (row_id, (row, observation)) in rows.iter().zip(&observations).enumerate() {
        let landing: String = row
            .base
            .persisted
            .iter()
            .map(|is_persisted| if *is_persisted { '2' } else { '0' })
            .collect();
        let patch_list: Vec<String> = row
            .patches
            .iter()
            .map(|patch| {
                let start = patches_bytes.len();
                patches_bytes.extend_from_slice(&patch.bytes);
                format!(
                    "{}:{}:{start}:{}",
                    patch.device.0,
                    patch.offset,
                    patch.bytes.len()
                )
            })
            .collect();
        states_text.push_str(&format!(
            "{row_id}\t{landing}\t{}\t{}\t{}\n",
            observation.base_digest,
            observation.patched_digest,
            patch_list.join(",")
        ));
        let layer0 = matched_bases
            .iter()
            .find(|(seen, _, _)| seen.label == row.base.label)
            .and_then(|(_, found, _)| *found)
            .map_or_else(|| "none".to_string(), |ordinal| ordinal.to_string());
        expected_text.push_str(&format!(
            "{row_id}\t{}\t{}\t{}\t{}\t{}\t{}\t{layer0}\t{}\n",
            row.name,
            row.family.label(),
            row.base.label,
            row.base
                .call
                .map_or_else(|| "-".to_string(), |call| call.to_string()),
            row.expected_verdict,
            row.expected_landing.text(),
            row.base.notes
        ));
        verdicts_text.push_str(&implementation_verdict_line(row_id, observation));
    }
    let seed_text = format!("seed_hex\t{seed_hex}\nseed_u64\t{seed}\n");
    let files = vec![
        write_export_file(&directory, BAD_STATES_FILE, states_text.as_bytes()),
        write_export_file(&directory, BAD_PATCHES_FILE, &patches_bytes),
        write_export_file(&directory, BAD_EXPECTED_FILE, expected_text.as_bytes()),
        write_export_file(
            &directory,
            BAD_IMPLEMENTATION_VERDICTS_FILE,
            verdicts_text.as_bytes(),
        ),
        write_export_file(&directory, BAD_SEED_FILE, seed_text.as_bytes()),
    ];
    BadStreamExport {
        name,
        bases: matched_bases,
        rows,
        observations,
        checks,
        files,
    }
}

/// 一条流的坏状态结果行；交回 (回比没过的条数, 实现判定 panic 的行数)。
fn report_bad_stream(lines: &mut ResultLines, export: &BadStreamExport) -> (usize, usize) {
    let label = export.name.directory_name();
    let mut failed_checks = 0usize;
    for check in &export.checks {
        failed_checks += usize::from(!check.holds());
        lines.result(&format!(
            "name=bad_s1 stream={label} check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    for (base, found, candidates) in &export.bases {
        lines.result(&format!(
            "name=bad_base stream={label} base={} call={} persisted_writes={} layer0_match={} layer0_candidates={candidates} notes={}",
            base.label,
            base.call.map_or_else(|| "-".to_string(), |call| call.to_string()),
            base.persisted.iter().filter(|is_persisted| **is_persisted).count(),
            found.map_or_else(|| "none".to_string(), |ordinal| ordinal.to_string()),
            field_text(&base.notes)
        ));
    }
    let mut panics = 0usize;
    let mut both_pass_by_family: BTreeMap<&'static str, (usize, usize)> = BTreeMap::new();
    for (row_id, (row, observation)) in export.rows.iter().zip(&export.observations).enumerate() {
        let (verdict_text, both_pass) = match &observation.verdict {
            Ok(verdict) => (
                format!(
                    "impl_consulted_root={} impl_consulted_outcome={} impl_ignored_root={} impl_pool_checker_red={} impl_root_without_record={} impl_claimed_state_missing_unit={}",
                    verdict.consulted_root,
                    verdict.consulted_outcome,
                    verdict.ignored_root,
                    field_text(&list_text(&verdict.pool_checker_red)),
                    verdict.root_without_record,
                    verdict.claimed_state_missing_unit
                ),
                verdict.pool_checker_and_record_checker_both_pass(),
            ),
            Err(panic) => {
                panics += 1;
                (format!("impl_panicked={}", field_text(panic)), false)
            }
        };
        let tally = both_pass_by_family
            .entry(row.family.label())
            .or_insert((0, 0));
        tally.0 += 1;
        tally.1 += usize::from(both_pass);
        lines.result(&format!(
            "name=bad_row stream={label} row={row_id} name={} family={} base={} patches={} patched_bytes={} expected={} expected_landing={} base_digest={} patched_digest={} {verdict_text} impl_both_pass={both_pass}",
            row.name,
            row.family.label(),
            row.base.label,
            row.patches.len(),
            row.patches.iter().map(|patch| patch.bytes.len()).sum::<usize>(),
            row.expected_verdict,
            row.expected_landing.text(),
            observation.base_digest,
            observation.patched_digest
        ));
    }
    for (family, (rows, both_pass)) in &both_pass_by_family {
        lines.result(&format!(
            "name=bad_family stream={label} family={family} rows={rows} impl_both_pass={both_pass}"
        ));
    }
    for file in &export.files {
        lines.result(&format!(
            "name=bad_file stream={label} file={} bytes={} sha256={}",
            file.name, file.bytes, file.sha256
        ));
    }
    (failed_checks, panics)
}

struct BadStatesCommandLine {
    table_version: BadTableVersion,
    dump_directory: PathBuf,
    seed_hex: String,
}

/// 第一个词认两个模式：`bad-states`（第一次的表）、`bad-states-second-run`（第 1 次重跑的表）；别的词不认。
fn parse_bad_states_command_line(arguments: &[String]) -> Option<BadStatesCommandLine> {
    let mut words = arguments.iter();
    let table_version = match words.next().map(String::as_str) {
        Some("bad-states") => BadTableVersion::FirstRun,
        Some("bad-states-second-run") => BadTableVersion::SecondRun,
        Some(_) | None => return None,
    };
    let mut dump_directory = None;
    let mut seed_hex = None;
    while let Some(word) = words.next() {
        match word.as_str() {
            "--dump-directory" => dump_directory = Some(PathBuf::from(words.next()?)),
            "--seed-hex" => seed_hex = Some(words.next()?.clone()),
            _ => return None,
        }
    }
    let seed_hex = seed_hex?;
    (seed_hex.len() == 16
        && seed_hex
            .chars()
            .all(|character| character.is_ascii_hexdigit()))
    .then_some(BadStatesCommandLine {
        table_version,
        dump_directory: dump_directory?,
        seed_hex,
    })
}

/// `bad-states` 与 `bad-states-second-run` 模式（`BadTableVersion` 区分）：两条流各造各判，流一先（盲测族的 splitmix64 一串接着抽）。
/// 退出码：0 正常；2 命令行不对或五个文件里有已存在的；3 回比没过（产物照打，交主 agent）。`config` 行的 `mode=` 两个模式各写各的名字，
/// 第一次的 `bad-states` 模式除此之外输出一个字节不变。
fn run_bad_states(command_line: &BadStatesCommandLine) -> i32 {
    for name in STREAM_NAMES {
        for file_name in BAD_FILES {
            let path = command_line
                .dump_directory
                .join(name.directory_name())
                .join(file_name);
            if path.exists() {
                eprintln!(
                    "{} 已存在：坏状态文件不覆盖，换一份导出目录",
                    path.display()
                );
                return 2;
            }
        }
    }
    let seed =
        u64::from_str_radix(&command_line.seed_hex, 16).expect("16 个十六进制字符，命令行判过");
    let mut generator = SplitMix64 { state: seed };
    let mut lines = ResultLines { emitted: 0 };
    lines.result(&format!(
        "name=config segment=s3 mode={} seed_hex={} seed_u64={seed} blind_rows_per_stream={BLIND_ROWS_PER_STREAM} patched_writes_kind=system_configuration_slot judge_threads=1",
        command_line.table_version.config_mode_text(),
        command_line.seed_hex
    ));
    let mut failed_checks = 0usize;
    let mut panics = 0usize;
    let mut rows = 0usize;
    for name in STREAM_NAMES {
        let export = bad_states_of_stream(
            name,
            &command_line.dump_directory,
            &command_line.seed_hex,
            seed,
            &mut generator,
            command_line.table_version,
        );
        let (stream_failed, stream_panics) = report_bad_stream(&mut lines, &export);
        failed_checks += stream_failed;
        panics += stream_panics;
        rows += export.rows.len();
    }
    lines.result(&format!(
        "name=verdict bad_s1_failed_checks={failed_checks} bad_rows={rows} impl_panics={panics}"
    ));
    lines.finish();
    if failed_checks > 0 {
        3
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 空消息的 sha256（`python3 -c "import hashlib; print(hashlib.sha256(b'').hexdigest())"` 现算）。
    const EMPTY_MESSAGE_SHA256: &str =
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    /// 盘 1、扇区 2、512 个 0xAB 那一条的摘要（`struct.pack('<IQ', 1, 2) + b'\xab' * 512` 喂 hashlib 现算）。
    const ONE_SECTOR_OF_0XAB_SHA256: &str =
        "c71cae1b3fc678e4cabfa21d575c728edd44a4b372b3389ce763856273abecbf";

    #[test]
    fn operation_kind_name_covers_every_recorded_operation_kind() {
        assert_eq!(operation_kind_name(RecordedOperationKind::Write), "write");
        assert_eq!(
            operation_kind_name(RecordedOperationKind::WriteForceUnitAccess),
            "write_fua"
        );
        assert_eq!(
            operation_kind_name(RecordedOperationKind::WriteZeroes),
            "write_zeroes"
        );
        assert_eq!(
            operation_kind_name(RecordedOperationKind::Barrier),
            "barrier"
        );
    }

    #[test]
    fn a_torn_landing_folds_to_one_and_a_persisted_write_to_two() {
        // 写表：0 持久、1 没持久、2 原写没持久而撕裂镜像（枚举写表下标 3）持久。
        let persisted = vec![true, false, false, true];
        let torn_image_index_by_write = BTreeMap::from([(2usize, 3usize)]);
        assert_eq!(
            landing_vector_of(&persisted, 3, &torn_image_index_by_write),
            "201",
            "撕裂那一态折回 1，不是 2"
        );
        let persisted_with_the_torn_image_absent = vec![true, false, false, false];
        assert_eq!(
            landing_vector_of(
                &persisted_with_the_torn_image_absent,
                3,
                &torn_image_index_by_write
            ),
            "200"
        );
    }

    #[test]
    fn an_all_zero_sector_does_not_enter_the_image_digest() {
        let mut message = Vec::new();
        append_sector_to_digest_message(&mut message, 0, 7, &[0u8; 512]);
        assert!(message.is_empty(), "全零扇区不进摘要");
        assert_eq!(sha256_text(&sha256_digest(&message)), EMPTY_MESSAGE_SHA256);
        append_sector_to_digest_message(&mut message, 1, 2, &[0xAB; 512]);
        assert_eq!(message.len(), 4 + 8 + 512);
        assert_eq!(
            sha256_text(&sha256_digest(&message)),
            ONE_SECTOR_OF_0XAB_SHA256
        );
    }

    #[test]
    fn batch_points_follow_the_registered_formulas_on_a_hundred_state_range() {
        let state_range = 10u64..110;
        assert_eq!(
            first_batch_points(&state_range)
                .into_iter()
                .collect::<BTreeSet<u64>>(),
            BTreeSet::from([10, 24, 38, 52, 66, 80, 94, 109])
        );
        assert_eq!(
            second_batch_points(&state_range),
            vec![17, 31, 45, 59, 73, 87, 101]
        );
        let third = third_batch_points(&state_range);
        assert_eq!(third.len(), 24);
        assert_eq!(third[0], 10);
        assert_eq!(third[1], 14, "10 + ⌊99/23⌋");
        assert_eq!(third[23], 109);
    }

    #[test]
    fn batch_points_on_short_ranges_collapse_onto_the_first_batch() {
        let single = 5u64..6;
        assert_eq!(
            first_batch_points(&single)
                .into_iter()
                .collect::<BTreeSet<u64>>(),
            BTreeSet::from([5])
        );
        let eight = 0u64..8;
        assert_eq!(
            first_batch_points(&eight)
                .into_iter()
                .collect::<BTreeSet<u64>>(),
            (0..8).collect::<BTreeSet<u64>>()
        );
        assert_eq!(second_batch_points(&eight), vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn sampled_batches_are_disjoint_and_include_the_last_state_and_up_to_four_torn_states() {
        // 两段：[0, 8) 带撕裂写，序号为奇数的状态带 1；[8, 108) 不带。总数 109，最后那个是 108。
        let ranges = vec![0u64..8, 8..108];
        let landing = |ordinal: u64| {
            if ordinal % 2 == 1 {
                "21".to_string()
            } else {
                "20".to_string()
            }
        };
        let sampled = sample_ordinals(&ranges, 109, &[true, false], &landing);
        let [first, second, third] = &sampled.batches;
        assert!(first.contains(&108), "最后那个全持久状态进批 1");
        assert!(first.is_disjoint(second) && first.is_disjoint(third) && second.is_disjoint(third));
        assert_eq!(
            sampled.torn_scans,
            vec![TornScan {
                segment_index: 0,
                scanned: 8,
                found: 0
            }],
            "[0, 8) 的点批 1 已经全取了，扫完 8 个一个新的都没有"
        );
    }

    #[test]
    fn torn_scan_stops_after_four_new_states() {
        let ranges: Vec<std::ops::Range<u64>> = std::iter::once(0u64..1000).collect();
        let landing = |ordinal: u64| {
            if ordinal % 3 == 2 {
                "1".to_string()
            } else {
                "0".to_string()
            }
        };
        let sampled = sample_ordinals(&ranges, 1001, &[true], &landing);
        // 批 1 的格点 0、142、285、428、570、713、856、999；带 1 的是 2、5、8、11（都不在格点上）。
        assert_eq!(sampled.torn_scans[0].found, 4);
        assert_eq!(sampled.torn_scans[0].scanned, 12);
        for ordinal in [2u64, 5, 8, 11] {
            assert!(sampled.batches[0].contains(&ordinal));
        }
    }

    /// 两条流本地段长表的闭式 1 + Σ(2^n − 1) 与写数，与本地抄来的闭式、写数逐个相等（改段长表里任何一个数，这里都红）。
    #[test]
    fn local_segment_tables_reproduce_the_closed_forms_and_write_counts_copied_from_the_tests() {
        let closed_form = |lengths: &[usize]| -> u64 {
            1 + lengths
                .iter()
                .map(|length| (1u64 << length) - 1)
                .sum::<u64>()
        };
        assert_eq!(
            closed_form(&LOCAL_FIRST_STREAM_SEGMENT_LENGTHS),
            LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM
        );
        assert_eq!(
            LOCAL_FIRST_STREAM_SEGMENT_LENGTHS.iter().sum::<usize>(),
            LOCAL_FIRST_STREAM_WRITES
        );
        assert_eq!(
            closed_form(&LOCAL_SECOND_STREAM_SEGMENT_LENGTHS),
            LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM
        );
        assert_eq!(
            LOCAL_SECOND_STREAM_SEGMENT_LENGTHS.iter().sum::<usize>(),
            LOCAL_SECOND_STREAM_WRITES
        );
        assert_eq!(LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM, 1_662_648_449);
        assert_eq!(LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM, 16_777_240);
    }

    fn bytes_write(device: u32, offset: u64, bytes: Vec<u8>) -> RetainedWrite {
        RetainedWrite {
            device: DeviceIdentity(device),
            kind: StepKind::SystemConfigurationSlot,
            is_force_unit_access: false,
            offset: DeviceOffsetInBytes(offset),
            contents: WrittenContents::Bytes(bytes),
        }
    }

    fn neutral(
        device: u32,
        kind: RecordedOperationKind,
        offset: u64,
        length: u64,
        payload_start: Option<usize>,
    ) -> NeutralOperation {
        NeutralOperation {
            device,
            kind,
            offset,
            length,
            payload_start,
        }
    }

    /// 小池上同一个状态两条路算摘要：`CrashImage` 直接读（带撕裂镜像的枚举写表）与中性格式按落地向量重建，两份逐字节相同；
    /// 落地向量按写序号是「2 2 0 1」。
    #[test]
    fn rebuilding_from_the_neutral_format_matches_the_crash_image_digest_on_a_small_pool() {
        let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], 1 << 20);
        base.apply_writes(&[
            bytes_write(0, 0, vec![0x11; 512]),
            bytes_write(1, 8192, vec![0x22; 1024]),
        ]);
        let written_at_4096 = vec![0x33; 1024];
        let zeroes_over_the_base_sector = RetainedWrite {
            device: DeviceIdentity(0),
            kind: StepKind::ZeroFill,
            is_force_unit_access: false,
            offset: DeviceOffsetInBytes(0),
            contents: WrittenContents::Zeros { length: 512 },
        };
        let not_persisted = vec![0x44; 512];
        let overwrite_new = vec![0x55; 1024];
        let torn = [vec![0x55; 512], vec![0x22; 512]].concat();
        let writes = vec![
            bytes_write(0, 4096, written_at_4096.clone()),
            zeroes_over_the_base_sector,
            bytes_write(1, 0, not_persisted.clone()),
            bytes_write(1, 8192, overwrite_new.clone()),
            bytes_write(1, 8192, torn.clone()),
        ];
        let image = CrashImage {
            base: &base,
            writes: &writes,
            persisted: vec![true, true, false, false, true],
        };
        let landing = landing_vector_of(&image.persisted, 4, &BTreeMap::from([(3usize, 4usize)]));
        assert_eq!(landing, "2201");
        let payload = [written_at_4096, not_persisted, overwrite_new].concat();
        let stream = NeutralStream {
            base_sectors: BTreeMap::from([
                ((0u32, 0u64), vec![0x11; 512]),
                ((1, 16), vec![0x22; 512]),
                ((1, 17), vec![0x22; 512]),
            ]),
            operations: vec![
                neutral(0, RecordedOperationKind::Write, 4096, 1024, Some(0)),
                neutral(0, RecordedOperationKind::Barrier, 0, 0, None),
                neutral(0, RecordedOperationKind::WriteZeroes, 0, 512, None),
                neutral(1, RecordedOperationKind::Write, 0, 512, Some(1024)),
                neutral(
                    1,
                    RecordedOperationKind::WriteForceUnitAccess,
                    8192,
                    1024,
                    Some(1536),
                ),
            ],
            payload,
            torn_bytes_by_write: BTreeMap::from([(3usize, (0usize, 1024usize))]),
            torn_payload: torn,
        };
        let from_the_image = crash_image_digest(&image);
        assert_eq!(rebuilt_image_digest(&stream, &landing), from_the_image);
        assert_ne!(
            rebuilt_image_digest(&stream, "2202"),
            from_the_image,
            "撕裂那一态记成持久，重建出来的是新字节，摘要必须不同"
        );
    }

    #[test]
    fn contents_without_timing_drops_only_the_timing_column_and_the_verdicts_digest_line() {
        assert_eq!(
            contents_without_timing(
                IMPLEMENTATION_VERDICTS_FILE,
                b"0\ta\tb\t123\n1\tc\td\t4567\n"
            ),
            Some(b"0\ta\tb\n1\tc\td\n".to_vec())
        );
        assert_eq!(
            contents_without_timing(
                MANIFEST_FILE,
                b"stream\tx\nsha256:states.tsv\taa\nsha256:impl-verdicts.tsv\tbb\n"
            ),
            Some(b"stream\tx\nsha256:states.tsv\taa\n".to_vec())
        );
        assert_eq!(contents_without_timing(STATES_FILE, b"0\t1\n"), None);
    }

    #[test]
    fn timing_summary_takes_the_lower_middle_and_the_ninety_fifth_percentile() {
        assert_eq!(timing_summary(vec![]), (0, 0, 0, 0));
        assert_eq!(timing_summary(vec![4, 1, 3, 2]), (10, 2, 4, 4));
        let hundred: Vec<u128> = (1..=100).collect();
        assert_eq!(timing_summary(hundred), (5050, 50, 95, 100));
    }

    #[test]
    fn merged_sector_ranges_join_overlapping_and_touching_ranges_only() {
        assert_eq!(
            merged_sector_ranges(vec![(10, 12), (0, 2), (2, 3), (11, 15), (20, 21)]),
            vec![(0, 3), (10, 15), (20, 21)]
        );
    }

    /// splitmix64 从状态 0 起的头两个输出（公开参考实现的已知值，草稿目录里另用 Python 按同一式子算过）。
    #[test]
    fn splitmix64_from_state_zero_gives_the_reference_outputs() {
        let mut generator = SplitMix64 { state: 0 };
        assert_eq!(generator.next_value(), 0xE220_A839_7B1D_CDAF);
        assert_eq!(generator.next_value(), 0x6E78_9E6A_A1B9_65F4);
        assert!(generator.below(7) < 7);
    }

    #[test]
    fn the_middle_position_is_the_ceiling_of_half_counted_from_one() {
        assert_eq!(middle_position(1), 0);
        assert_eq!(middle_position(2), 0, "K = 2 取第 1 个");
        assert_eq!(middle_position(14), 6, "K = 14 取第 7 个");
        assert_eq!(middle_position(15), 7, "K = 15 取第 8 个");
    }

    #[test]
    fn the_bad_states_command_line_takes_a_dump_directory_and_a_sixteen_digit_seed() {
        let words = |text: &str| text.split(' ').map(str::to_string).collect::<Vec<String>>();
        let parsed = parse_bad_states_command_line(&words(
            "bad-states --dump-directory d --seed-hex 8546b1258e427c34",
        ))
        .expect("合法的命令行");
        assert_eq!(parsed.dump_directory, PathBuf::from("d"));
        assert_eq!(parsed.seed_hex, "8546b1258e427c34");
        assert!(parse_bad_states_command_line(&words(
            "bad-states --dump-directory d --seed-hex 8546"
        ))
        .is_none());
        assert!(
            parse_bad_states_command_line(&words("bad-states --seed-hex 8546b1258e427c34"))
                .is_none()
        );
        assert!(parse_bad_states_command_line(&words(
            "dump --dump-directory d --seed-hex 8546b1258e427c34"
        ))
        .is_none());
    }

    #[test]
    fn the_second_run_bad_states_mode_is_its_own_word_and_the_first_run_word_still_means_the_first_table(
    ) {
        let words = |text: &str| text.split(' ').map(str::to_string).collect::<Vec<String>>();
        let first_run = parse_bad_states_command_line(&words(
            "bad-states --dump-directory d --seed-hex 85441ae491419411",
        ))
        .expect("合法的命令行");
        assert_eq!(first_run.table_version, BadTableVersion::FirstRun);
        let second_run = parse_bad_states_command_line(&words(
            "bad-states-second-run --dump-directory d --seed-hex 85441ae491419411",
        ))
        .expect("合法的命令行");
        assert_eq!(second_run.table_version, BadTableVersion::SecondRun);
        assert_eq!(second_run.seed_hex, "85441ae491419411");
        assert!(parse_bad_states_command_line(&words(
            "bad-states-second-run-extra --dump-directory d --seed-hex 85441ae491419411"
        ))
        .is_none());
        assert_eq!(BadTableVersion::FirstRun.mode_word(), "bad-states");
        assert_eq!(
            BadTableVersion::SecondRun.mode_word(),
            "bad-states-second-run"
        );
    }

    /// 第 1 次重跑登记第十一节 S5 的单测形态：同一个种子、同一条流（流一，驱动快），两版的表逐行相同，只差 P8 的期望判定与末尾多出的 P10；
    /// P10 的底态与 P9-Mk+ 同一个、补丁与同底态族的 P8-Mk 同一个（同一个改法落在 Mk⁺ 上）。
    #[test]
    fn the_second_run_table_differs_from_the_first_only_in_the_p8_expectation_and_a_final_p10_row_on_the_p9_base(
    ) {
        let first_run_seed = 0x8546_b125_8e42_7c34_u64;
        let stream = prepare_stream(StreamName::NewPoolFileCreation);
        let facts = bad_stream_facts(&stream);
        let (_, first_run_rows) = bad_rows_of_stream(
            &facts,
            &mut SplitMix64 {
                state: first_run_seed,
            },
            BadTableVersion::FirstRun,
            &mut Vec::new(),
        );
        let (_, second_run_rows) = bad_rows_of_stream(
            &facts,
            &mut SplitMix64 {
                state: first_run_seed,
            },
            BadTableVersion::SecondRun,
            &mut Vec::new(),
        );
        let patch_texts = |row: &BadRow| -> Vec<(u32, u64, Vec<u8>)> {
            row.patches
                .iter()
                .map(|patch| (patch.device.0, patch.offset, patch.bytes.clone()))
                .collect()
        };
        assert_eq!(
            second_run_rows.len(),
            first_run_rows.len() + 1,
            "第 1 次重跑的表比第一次的多恰一行（P10）"
        );
        for (first_run_row, second_run_row) in first_run_rows.iter().zip(&second_run_rows) {
            assert_eq!(first_run_row.name, second_run_row.name);
            assert_eq!(first_run_row.family, second_run_row.family);
            assert_eq!(first_run_row.base.persisted, second_run_row.base.persisted);
            assert_eq!(patch_texts(first_run_row), patch_texts(second_run_row));
            assert_eq!(
                first_run_row.expected_landing.text(),
                second_run_row.expected_landing.text()
            );
            if second_run_row.name.starts_with("P8-") {
                assert_eq!(
                    first_run_row.expected_verdict, "not_in",
                    "{}",
                    first_run_row.name
                );
                assert_eq!(
                    second_run_row.expected_verdict, "in",
                    "{}",
                    second_run_row.name
                );
            } else {
                assert_eq!(
                    first_run_row.expected_verdict, second_run_row.expected_verdict,
                    "{}",
                    first_run_row.name
                );
            }
        }
        let lower_bound_row = second_run_rows.last().expect("表不空");
        assert_eq!(lower_bound_row.name, LOWER_BOUND_ROW_NAME);
        assert_eq!(lower_bound_row.family, BadFamily::AcknowledgementLowerBound);
        assert_eq!(lower_bound_row.expected_verdict, "not_in");
        let mk_plus_p9_row = second_run_rows
            .iter()
            .find(|row| row.name == FIRST_RUN_MK_PLUS_ROW_NAME)
            .expect("流一有 Mk⁺ 底态、有 P9 行");
        assert_eq!(
            lower_bound_row.base.persisted, mk_plus_p9_row.base.persisted,
            "P10 的底态与 P9-Mk+ 是同一个落地向量（S5b）；取了 Mk 就差一个写"
        );
        let mk_p8_row = second_run_rows
            .iter()
            .find(|row| row.name == "P8-Mk")
            .expect("有 P8-Mk 行");
        assert_eq!(patch_texts(lower_bound_row), patch_texts(mk_p8_row));
        assert_eq!(
            lower_bound_row.expected_landing.text(),
            mk_p8_row.expected_landing.text()
        );
        assert!(
            lower_bound_row.base.persisted != mk_p8_row.base.persisted,
            "Mk⁺ 比 Mk 多一个持久的写"
        );
    }

    #[test]
    fn the_command_line_takes_dump_an_output_directory_and_an_optional_previous_export() {
        let words = |text: &str| text.split(' ').map(str::to_string).collect::<Vec<String>>();
        let parsed = parse_command_line(&words("dump --output-directory a --compare-with b"))
            .expect("合法的命令行");
        assert_eq!(parsed.output_directory, PathBuf::from("a"));
        assert_eq!(parsed.compare_with, Some(PathBuf::from("b")));
        assert!(parse_command_line(&words("dump --compare-with b")).is_none());
        assert!(parse_command_line(&words("dump --output-directory a --extra")).is_none());
        assert!(parse_command_line(&words("list --output-directory a")).is_none());
    }
}
